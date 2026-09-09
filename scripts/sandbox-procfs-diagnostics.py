"""Isolate procfs failures without changing a sandbox grant or backend."""

import hashlib
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import tempfile


reports = Path("reports").resolve()
reports.mkdir(exist_ok=True)
results = []


def invoke(label, command, env, cwd, injection=None):
    trace = reports / f"{label}.strace"
    argv = list(command)
    if injection is not None:
        options = ["strace", "-f", "-qq", "-s", "256", "-o", str(trace),
                   "-e", "trace=open,openat,openat2,mknod,mknodat"]
        if injection in ("maps", "stat"):
            options += ["-P", f"/proc/self/{injection}",
                        "-e", "inject=open,openat,openat2:error=EACCES"]
        elif injection == "fifo":
            options += ["-e", "inject=mknod,mknodat:error=EACCES"]
        argv = options + argv
    child = subprocess.Popen(argv, cwd=cwd, env=env, stdout=subprocess.PIPE,
                             stderr=subprocess.PIPE, start_new_session=True)
    try:
        stdout, stderr = child.communicate(timeout=120)
    except subprocess.TimeoutExpired:
        os.killpg(child.pid, signal.SIGKILL)
        stdout, stderr = child.communicate()
        raise AssertionError(f"{label} exceeded its deadline: {stdout!r} {stderr!r}")
    (reports / f"{label}.stdout").write_bytes(stdout)
    (reports / f"{label}.stderr").write_bytes(stderr)
    injected = [line for line in trace.read_text().splitlines() if "INJECTED" in line] if trace.exists() else []
    row = {"label": label, "command": command, "exit": child.returncode,
           "injection": injection, "injected_calls": injected}
    results.append(row)
    (reports / "results.json").write_text(json.dumps(results, indent=2) + "\n")
    print(json.dumps(row), flush=True)
    return child.returncode, stdout, injected


def environment(root):
    env = {key: os.environ[key] for key in ["PATH", "DOTNET_ROOT"] if key in os.environ}
    for key, leaf in [("HOME", "home"), ("TMPDIR", "tmp"), ("DOTNET_CLI_HOME", "dotnet-home"),
                      ("XDG_CACHE_HOME", "cache"), ("XDG_CONFIG_HOME", "config")]:
        path = root / leaf
        path.mkdir()
        env[key] = str(path)
    env.update(DOTNET_CLI_TELEMETRY_OPTOUT="1", DOTNET_SKIP_FIRST_TIME_EXPERIENCE="1",
               DOTNET_NOLOGO="1", BUN_INSTALL_CACHE_DIR=str(root / "cache"))
    return env


with tempfile.TemporaryDirectory(prefix="procfs-diagnostics-") as temporary:
    root = Path(temporary)
    probe = root / "filter-control"
    probe.mkdir()
    env = environment(probe)
    # Prove path filtering limits INJECTION, not only printed trace output.
    code = """import errno, pathlib
assert pathlib.Path('ordinary').read_text() == 'visible'
try:
    pathlib.Path('/proc/self/maps').read_text()
except PermissionError as error:
    assert error.errno == errno.EACCES
else:
    raise AssertionError('selected procfs open was not refused')
assert pathlib.Path('ordinary').read_text() == 'visible'
print('SELECTIVE_DENIAL_OK')
"""
    (probe / "ordinary").write_text("visible")
    status, output, calls = invoke("filter-control", [shutil.which("python3"), "-c", code], env, probe, "maps")
    assert status == 0 and b"SELECTIVE_DENIAL_OK" in output and calls
    assert all('"/proc/self/maps"' in line and "EACCES" in line for line in calls)

    programs = {"bun": str(Path(os.environ["BUN_DIAGNOSTIC_BIN"]).resolve()),
                "node": shutil.which("node"), "dotnet": shutil.which("dotnet")}
    bindings = {name: {"path": path, "sha256": hashlib.sha256(Path(path).read_bytes()).hexdigest()}
                for name, path in programs.items()}
    (reports / "binaries.json").write_text(json.dumps(bindings, indent=2) + "\n")
    for name, modes in [("bun", [None, "trace", "maps"]),
                        ("node", [None, "trace", "stat"]),
                        ("dotnet", [None, "trace", "maps", "fifo"])]:
        for mode in modes:
            label = f"{name}-{mode or 'plain'}"
            directory = root / label
            directory.mkdir()
            env = environment(directory)
            if name == "bun":
                (directory / "package").mkdir()
                (directory / "package/package.json").write_text('{"name":"local-fixture","version":"1.0.0"}')
                (directory / "package.json").write_text('{"name":"fixture","private":true,"dependencies":{"local-fixture":"file:./package"}}')
                command = [programs[name], "install", "--ignore-scripts"]
            elif name == "node":
                command = [programs[name], "-e", "console.log(process.memoryUsage().heapTotal)"]
            else:
                (directory / "fixture.csproj").write_text('<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><TargetFramework>net10.0</TargetFramework></PropertyGroup></Project>')
                (directory / "NuGet.Config").write_text('<configuration><packageSources><clear /></packageSources></configuration>')
                command = [programs[name], "restore", "--ignore-failed-sources"]
            status, _, calls = invoke(label, command, env, directory, mode)
            if mode in (None, "trace"):
                assert status == 0, f"{label} baseline failed; no causal conclusion is valid"
            else:
                assert calls, f"{label} did not exercise the selected denied operation"
                if mode in ("maps", "stat"):
                    assert all(f'"/proc/self/{mode}"' in line for line in calls)

