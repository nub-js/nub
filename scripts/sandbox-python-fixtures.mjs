#!/usr/bin/env node
/**
 * Provision the pinned Python tools used by
 * `crates/nub-sandbox/tests/python_tool_functionality.rs`.
 *
 * The pins were checked against PyPI's JSON endpoint and Astral's release API
 * before this fixture was authored. Both distributions are installed from PyPI
 * into isolated target directories. Pip runs through the Python executable and
 * `-m pip`; Windows never relies on a generated `.cmd` wrapper.
 */
import { appendFileSync, existsSync, mkdirSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { execFileSync } from 'node:child_process';

const root = resolve(process.env.NUB_SANDBOX_PYTHON_TOOL_ROOT ?? '.sandbox-python-fixtures');
const pins = {
  pip: '26.2.1',
  uv: '0.12.11',
};

function command(command, args, options = {}) {
  return execFileSync(command, args, {
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'inherit'],
    ...options,
  }).trim();
}

function python() {
  const candidates = [
    process.env.NUB_SANDBOX_PYTHON,
    process.platform === 'win32' ? 'python' : 'python3',
    'python',
  ].filter(Boolean);
  for (const candidate of candidates) {
    try {
      const details = JSON.parse(command(candidate, [
        '-c',
        'import json, sys; print(json.dumps({"executable": sys.executable, "prefix": sys.prefix, "version": sys.version.split()[0]}))',
      ]));
      if (details.executable && details.prefix) return { command: candidate, ...details };
    } catch {
      // Try the next documented runner spelling.
    }
  }
  throw new Error(`Python was not found; set NUB_SANDBOX_PYTHON (tried ${candidates.join(', ')})`);
}

function installTarget(python, target, spec) {
  mkdirSync(target, { recursive: true });
  execFileSync(python, [
    '-m', 'pip', 'install', '--disable-pip-version-check', '--no-input', '--upgrade',
    '--target', target, spec,
  ], {
    stdio: 'inherit',
    env: { ...process.env, PIP_DISABLE_PIP_VERSION_CHECK: '1' },
  });
}

function wheel(python, destination) {
  const program = String.raw`import base64, hashlib, pathlib, sys, zipfile
out = pathlib.Path(sys.argv[1])
out.parent.mkdir(parents=True, exist_ok=True)
files = {
  'sandbox_python_fixture/__init__.py': 'def main():\n    print("sandbox-python-fixture-ok")\n',
  'sandbox_python_fixture-1.0.0.dist-info/METADATA': 'Metadata-Version: 2.1\nName: sandbox-python-fixture\nVersion: 1.0.0\n',
  'sandbox_python_fixture-1.0.0.dist-info/WHEEL': 'Wheel-Version: 1.0\nGenerator: nub sandbox fixture\nRoot-Is-Purelib: true\nTag: py3-none-any\n',
  'sandbox_python_fixture-1.0.0.dist-info/entry_points.txt': '[console_scripts]\nsandbox-python-fixture = sandbox_python_fixture:main\n',
}
records = []
for name, content in files.items():
    payload = content.encode()
    digest = base64.urlsafe_b64encode(hashlib.sha256(payload).digest()).decode().rstrip('=')
    records.append(f'{name},sha256={digest},{len(payload)}')
files['sandbox_python_fixture-1.0.0.dist-info/RECORD'] = '\n'.join(records) + '\n' + 'sandbox_python_fixture-1.0.0.dist-info/RECORD,,\n'
with zipfile.ZipFile(out, 'w', zipfile.ZIP_DEFLATED) as archive:
    for name, content in files.items(): archive.writestr(name, content)
`;
  command(python, ['-c', program, destination]);
}

function uvExecutable(target) {
  const candidates = process.platform === 'win32'
    ? [join(target, 'Scripts', 'uv.exe')]
    : [join(target, 'bin', 'uv')];
  const executable = candidates.find(existsSync);
  if (!executable) throw new Error(`uv@${pins.uv} did not provide an executable (looked in ${candidates.join(', ')})`);
  if (executable.endsWith('.cmd')) throw new Error('uv fixture must not use a .cmd wrapper');
  return executable;
}

mkdirSync(root, { recursive: true });
const runtime = python();
const pipRoot = join(root, 'pip');
const uvRoot = join(root, 'uv');
const fixtureWheel = join(root, 'wheel', 'sandbox_python_fixture-1.0.0-py3-none-any.whl');
installTarget(runtime.command, pipRoot, `pip==${pins.pip}`);
installTarget(runtime.command, uvRoot, `uv==${pins.uv}`);
wheel(runtime.command, fixtureWheel);

const pipVersion = command(runtime.command, ['-c', 'import pip; print(pip.__version__)'], {
  env: { ...process.env, PYTHONPATH: pipRoot },
});
if (pipVersion !== pins.pip) throw new Error(`expected pip ${pins.pip}, got ${pipVersion}`);
const uvProgram = uvExecutable(uvRoot);
const uvVersion = command(uvProgram, ['--version']).split(/\s+/).at(-1);
if (uvVersion !== pins.uv) throw new Error(`expected uv ${pins.uv}, got ${uvVersion}`);

const matrix = [
  {
    name: 'pip',
    spec: `pip==${pins.pip}`,
    program: runtime.executable,
    prefix: ['-m', 'pip'],
    toolRoot: pipRoot,
    runtimeRoot: runtime.prefix,
    python: runtime.executable,
    version: pipVersion,
    wheel: fixtureWheel,
  },
  {
    name: 'uv',
    spec: `uv==${pins.uv}`,
    program: uvProgram,
    prefix: [],
    toolRoot: uvRoot,
    runtimeRoot: runtime.prefix,
    python: runtime.executable,
    version: uvVersion,
    wheel: fixtureWheel,
  },
];
const matrixFile = join(root, 'matrix.json');
writeFileSync(matrixFile, `${JSON.stringify(matrix, null, 2)}\n`);
const lines = [
  `NUB_SANDBOX_PYTHON_TOOL_MATRIX_FILE=${matrixFile}`,
  `NUB_SANDBOX_PYTHON=${runtime.executable}`,
];
if (process.env.GITHUB_ENV) appendFileSync(process.env.GITHUB_ENV, `${lines.join('\n')}\n`);
else process.stdout.write(`${lines.join('\n')}\n`);
