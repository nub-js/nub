//! Native Cargo/rustup, Go, JVM, NuGet, and Composer tool-directory controls.

#[path = "common/tool_output.rs"]
mod tool_output;

use nub_sandbox::{CommandSpec, CompileCtx, Homes, Sandbox, ScopeCapabilities, compile};
use serde::Deserialize;
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[derive(Deserialize)]
struct Tool {
    name: String,
    program: PathBuf,
    prefix: Vec<String>,
    version: String,
    #[serde(rename = "toolRoot")]
    tool_root: PathBuf,
    #[serde(rename = "runtimeRoots")]
    runtime_roots: Vec<PathBuf>,
    #[serde(rename = "toolEnv", default)]
    tool_env: BTreeMap<String, String>,
    #[serde(default)]
    shell: bool,
    #[serde(rename = "mavenSeed")]
    maven_seed: Option<PathBuf>,
}

fn tool(name: &str) -> Tool {
    let matrix = std::env::var("NUB_SANDBOX_NATIVE_TOOL_MATRIX_FILE").expect(
        "native tool matrix missing: run `node scripts/sandbox-native-tool-fixtures.mjs` before native tests",
    );
    let text = std::fs::read_to_string(&matrix)
        .unwrap_or_else(|error| panic!("native tool matrix `{matrix}` unreadable: {error}"));
    serde_json::from_str::<Vec<Tool>>(&text)
        .unwrap_or_else(|error| panic!("native tool matrix `{matrix}` is invalid: {error}"))
        .into_iter()
        .find(|tool| tool.name == name)
        .unwrap_or_else(|| panic!("native tool matrix omitted {name}"))
}

fn fixture() -> tempfile::TempDir {
    let parent = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .expect("runner home");
    let root = tempfile::Builder::new()
        .prefix("sandbox-native-tool-")
        .tempdir_in(parent)
        .expect("fixture root");
    for path in ["home", "home/.m2", "project/java-tmp", "cache", "tmp"] {
        std::fs::create_dir_all(root.path().join(path)).expect("fixture directory");
    }
    root
}

fn env_for(root: &Path, tool: &Tool) -> BTreeMap<String, String> {
    let mut env = BTreeMap::new();
    for key in [
        "PATH",
        "SystemRoot",
        "SYSTEMROOT",
        "WINDIR",
        "COMSPEC",
        "PATHEXT",
        "ProgramFiles",
        "ProgramFiles(x86)",
        "ProgramData",
        "ALLUSERSPROFILE",
    ] {
        if let Ok(value) = std::env::var(key) {
            env.insert(key.into(), value);
        }
    }
    let home = root.join("home");
    for (key, path) in [
        ("HOME", home.clone()),
        ("USERPROFILE", home.clone()),
        ("APPDATA", home.join("AppData/Roaming")),
        ("LOCALAPPDATA", home.join("AppData/Local")),
        ("XDG_CACHE_HOME", home.join("cache")),
        ("XDG_CONFIG_HOME", home.join("config")),
        ("XDG_DATA_HOME", home.join("data")),
        ("CARGO_HOME", home.join("cargo-home")),
        ("RUSTUP_HOME", home.join("rustup-home")),
        ("CARGO_TARGET_DIR", root.join("cargo-target")),
        ("GOMODCACHE", home.join("go-mod")),
        ("GOCACHE", home.join("go-cache")),
        ("GOBIN", home.join("go-bin")),
        ("GOTMPDIR", root.join("go-tmp")),
        ("GRADLE_USER_HOME", home.join("gradle")),
        ("NUGET_PACKAGES", home.join("nuget-packages")),
        ("NUGET_HTTP_CACHE_PATH", home.join("nuget-http")),
        ("NUGET_SCRATCH", home.join("nuget-scratch")),
        ("NUGET_PLUGINS_CACHE_PATH", home.join("nuget-plugins")),
        ("COMPOSER_HOME", home.join("composer-home")),
        ("COMPOSER_CACHE_DIR", home.join("composer-cache")),
        ("COMPOSER_VENDOR_DIR", root.join("project/vendor")),
        ("COMPOSER_BIN_DIR", root.join("project/vendor/bin")),
        ("DOTNET_CLI_HOME", home.join("dotnet")),
    ] {
        std::fs::create_dir_all(&path).expect("configured root precondition");
        env.insert(key.into(), path.to_string_lossy().into());
    }
    env.insert("GOPROXY".into(), "off".into());
    env.insert("GOSUMDB".into(), "off".into());
    env.insert(
        "JAVA_TOOL_OPTIONS".into(),
        format!(
            "-Djava.io.tmpdir={} -Duser.home={}",
            root.join("project/java-tmp").display(),
            home.display()
        ),
    );
    env.extend(tool.tool_env.clone());
    if let Some(seed) = &tool.maven_seed {
        copy_directory(seed, &home.join(".m2/repository"));
    }
    env
}

fn copy_directory(source: &Path, destination: &Path) {
    std::fs::create_dir_all(destination).expect("Maven seed destination");
    for entry in std::fs::read_dir(source).expect("Maven seed source") {
        let entry = entry.expect("Maven seed entry");
        let destination = destination.join(entry.file_name());
        if entry.file_type().expect("Maven seed type").is_dir() {
            copy_directory(&entry.path(), &destination);
        } else {
            std::fs::copy(entry.path(), destination).expect("Maven seed file");
        }
    }
}

fn policy(
    root: &Path,
    tool: &Tool,
    env: BTreeMap<String, String>,
    tooldirs: bool,
) -> nub_sandbox::SandboxPolicy {
    let mut fs = Map::new();
    if tooldirs {
        fs.insert("$tooldirs".into(), Value::String("rw".into()));
    } else {
        for key in [
            "CARGO_HOME",
            "RUSTUP_HOME",
            "CARGO_TARGET_DIR",
            "GOMODCACHE",
            "GOCACHE",
            "GOBIN",
            "GOTMPDIR",
            "GRADLE_USER_HOME",
            "NUGET_PACKAGES",
            "NUGET_HTTP_CACHE_PATH",
            "NUGET_SCRATCH",
            "NUGET_PLUGINS_CACHE_PATH",
            "COMPOSER_HOME",
            "COMPOSER_CACHE_DIR",
            "COMPOSER_VENDOR_DIR",
            "COMPOSER_BIN_DIR",
            "DOTNET_CLI_HOME",
        ] {
            if let Some(path) = env.get(key) {
                fs.insert(path.clone(), Value::String("rw".into()));
            }
        }
        fs.insert(
            root.join("home/.m2").to_string_lossy().into(),
            Value::String("rw".into()),
        );
    }
    fs.insert("./".into(), Value::String("rw".into()));
    fs.insert("$tmp".into(), Value::String("rw".into()));
    insert_read(&mut fs, &tool.tool_root);
    for root in &tool.runtime_roots {
        insert_read(&mut fs, root);
    }
    insert_read(&mut fs, tool.program.parent().expect("tool program parent"));
    let homes = Homes {
        home: root.join("home"),
        cache: root.join("cache"),
        tmp: root.join("tmp"),
        project: root.join("project"),
    };
    let ctx = CompileCtx::new(
        homes,
        root.join("project"),
        ScopeCapabilities::approved(),
        env.clone(),
    );
    // Gradle's file-lock service uses sockets even for an offline local build.
    // This matrix tests filesystem grants; network enforcement has separate tests.
    let network = tool.name == "gradle";
    if network {
        println!("NATIVE TOOL CAPABILITY gradle network=true (file-lock service)");
    }
    let mut policy =
        compile(&json!({"fs": fs, "net": network}), &ctx).expect("native tool policy compiles");
    policy.env.constructed = env;
    policy
}

fn insert_read(fs: &mut Map<String, Value>, path: &Path) {
    fs.entry(path.to_string_lossy().into_owned())
        .or_insert_with(|| Value::String("r".into()));
}

fn run(
    tool: &Tool,
    args: &[&str],
    root: &Path,
    env: &BTreeMap<String, String>,
    policy: Option<&nub_sandbox::SandboxPolicy>,
) -> Output {
    let args = tool
        .prefix
        .iter()
        .cloned()
        .chain(args.iter().map(|arg| (*arg).to_string()))
        .collect::<Vec<_>>();
    match policy {
        Some(policy) => {
            let sandbox = Sandbox::new(policy).expect("sandbox acquires");
            let spec = if tool.shell {
                CommandSpec::new(std::env::var_os("COMSPEC").unwrap_or_else(|| "cmd.exe".into()))
                    .verbatim_command_line(format!(
                        "/d /s /c \"{}\"",
                        command_line(&tool.program, &args)
                    ))
                    .cwd(root.join("project"))
                    .redact_stdout(true)
                    .redact_stderr(true)
            } else {
                CommandSpec::new(tool.program.to_string_lossy().into_owned())
                    .args(args.iter().cloned())
                    .cwd(root.join("project"))
                    .redact_stdout(true)
                    .redact_stderr(true)
            };
            let prepared = sandbox.prepare(spec).expect("tool prepares");
            assert!(
                prepared.degradation.lost.is_empty(),
                "{} degraded: {:?}",
                tool.name,
                prepared.degradation
            );
            tool_output::output(prepared)
        }
        None => {
            let mut command = if tool.shell {
                let mut command =
                    Command::new(std::env::var_os("COMSPEC").unwrap_or_else(|| "cmd.exe".into()));
                #[cfg(windows)]
                command.raw_arg(format!(
                    "/d /s /c \"{}\"",
                    command_line(&tool.program, &args)
                ));
                #[cfg(not(windows))]
                command.args(["/d", "/s", "/c", &command_line(&tool.program, &args)]);
                command
            } else {
                let mut command = Command::new(&tool.program);
                command.args(&args);
                command
            };
            command
                .current_dir(root.join("project"))
                .env_clear()
                .envs(env);
            command.output().expect("unconfined tool launches")
        }
    }
}

fn command_line(program: &Path, args: &[String]) -> String {
    std::iter::once(program.to_string_lossy().into_owned())
        .chain(args.iter().cloned())
        .map(|arg| format!("\"{}\"", arg.replace('"', "\\\"")))
        .collect::<Vec<_>>()
        .join(" ")
}

fn assert_ok(tool: &Tool, phase: &str, output: Output) {
    assert!(
        output.status.success(),
        "{} {phase} failed:\nstdout:\n{}\nstderr:\n{}",
        tool.name,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn write_projects(root: &Path) {
    let project = root.join("project");
    std::fs::write(
        project.join("Cargo.toml"),
        "[package]\nname = \"native_fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .unwrap();
    std::fs::create_dir_all(project.join("src")).unwrap();
    std::fs::write(
        project.join("src/main.rs"),
        "fn main() { println!(\"ok\"); }\n",
    )
    .unwrap();
    std::fs::write(
        project.join("go.mod"),
        "module example.invalid/nativefixture\ngo 1.25\n",
    )
    .unwrap();
    std::fs::write(project.join("main.go"), "package main\nfunc main() {}\n").unwrap();
    std::fs::write(
        project.join("build.gradle"),
        "tasks.register('fixture') { doLast { println 'ok' } }\n",
    )
    .unwrap();
    std::fs::write(
        project.join("settings.gradle"),
        "rootProject.name = 'native-fixture'\n",
    )
    .unwrap();
    std::fs::write(
        project.join("global.json"),
        "{\"sdk\":{\"version\":\"10.0.100\",\"rollForward\":\"disable\"}}\n",
    )
    .unwrap();
    std::fs::write(project.join("pom.xml"), "<project xmlns=\"http://maven.apache.org/POM/4.0.0\"><modelVersion>4.0.0</modelVersion><groupId>example</groupId><artifactId>fixture</artifactId><version>1</version><build><plugins><plugin><groupId>org.apache.maven.plugins</groupId><artifactId>maven-clean-plugin</artifactId><version>3.4.1</version></plugin></plugins></build></project>\n").unwrap();
    std::fs::write(project.join("fixture.csproj"), "<Project Sdk=\"Microsoft.NET.Sdk\"><PropertyGroup><TargetFramework>net10.0</TargetFramework><OutputType>Exe</OutputType><ImplicitUsings>enable</ImplicitUsings></PropertyGroup></Project>\n").unwrap();
    std::fs::write(
        project.join("Program.cs"),
        "System.Console.WriteLine(\"ok\");\n",
    )
    .unwrap();
    std::fs::write(
        project.join("composer.json"),
        "{\"name\":\"example/native-fixture\",\"require\":{}}\n",
    )
    .unwrap();
}

fn operations(
    name: &str,
    tool: &Tool,
    root: &Path,
    env: &BTreeMap<String, String>,
    policy: Option<&nub_sandbox::SandboxPolicy>,
) {
    match name {
        "cargo" => {
            assert_ok(
                tool,
                "cold build",
                run(tool, &["build", "--offline"], root, env, policy),
            );
            assert_ok(
                tool,
                "warm build",
                run(tool, &["build", "--offline"], root, env, policy),
            );
            assert_ok(
                tool,
                "target cleanup",
                run(tool, &["clean"], root, env, policy),
            );
        }
        "rustup" => assert_ok(
            tool,
            "home query",
            run(tool, &["show", "home"], root, env, policy),
        ),
        "go" => {
            assert_ok(tool, "cold build", run(tool, &["build"], root, env, policy));
            assert_ok(tool, "warm build", run(tool, &["build"], root, env, policy));
            assert_ok(
                tool,
                "user install",
                run(tool, &["install"], root, env, policy),
            );
            assert_ok(
                tool,
                "cache cleanup",
                run(tool, &["clean", "-cache"], root, env, policy),
            );
        }
        "gradle" => {
            assert_ok(
                tool,
                "cold task",
                run(
                    tool,
                    &["--offline", "--no-daemon", "fixture"],
                    root,
                    env,
                    policy,
                ),
            );
            assert_ok(
                tool,
                "warm task",
                run(
                    tool,
                    &["--offline", "--no-daemon", "fixture"],
                    root,
                    env,
                    policy,
                ),
            );
            assert_ok(
                tool,
                "daemon cleanup",
                run(tool, &["--stop"], root, env, policy),
            );
        }
        "maven" => {
            assert_ok(
                tool,
                "cold validate",
                run(tool, &["-o", "validate"], root, env, policy),
            );
            assert_ok(
                tool,
                "warm clean",
                run(tool, &["-o", "clean"], root, env, policy),
            );
        }
        "nuget" => {
            assert_ok(
                tool,
                "offline restore",
                run(
                    tool,
                    &["restore", "--ignore-failed-sources"],
                    root,
                    env,
                    policy,
                ),
            );
            assert_ok(
                tool,
                "build",
                run(tool, &["build", "--no-restore"], root, env, policy),
            );
            assert_ok(
                tool,
                "cache cleanup",
                run(
                    tool,
                    &["nuget", "locals", "all", "--clear"],
                    root,
                    env,
                    policy,
                ),
            );
        }
        "composer" => {
            assert_ok(
                tool,
                "cold install",
                run(
                    tool,
                    &[
                        "install",
                        "--no-interaction",
                        "--no-plugins",
                        "--no-scripts",
                    ],
                    root,
                    env,
                    policy,
                ),
            );
            assert_ok(
                tool,
                "warm install",
                run(
                    tool,
                    &[
                        "install",
                        "--no-interaction",
                        "--no-plugins",
                        "--no-scripts",
                    ],
                    root,
                    env,
                    policy,
                ),
            );
            assert_ok(
                tool,
                "cache cleanup",
                run(
                    tool,
                    &["clear-cache", "--no-interaction"],
                    root,
                    env,
                    policy,
                ),
            );
        }
        _ => unreachable!("matrix tool name"),
    }
}

fn run_case(name: &str, tooldirs: Option<bool>) {
    let tool = tool(name);
    eprintln!("NATIVE TOOL {} {}", tool.name, tool.version);
    let root = fixture();
    let env = env_for(root.path(), &tool);
    write_projects(root.path());
    let policy = tooldirs.map(|value| policy(root.path(), &tool, env.clone(), value));
    operations(name, &tool, root.path(), &env, policy.as_ref());
}

macro_rules! tool_cases {
    ($name:literal, $u:ident, $e:ident, $d:ident) => {
        #[test]
        #[ignore = "requires the pinned native tool matrix"]
        fn $u() {
            run_case($name, None);
        }
        #[test]
        #[ignore = "requires the pinned native tool matrix"]
        fn $e() {
            run_case($name, Some(false));
        }
        #[test]
        #[ignore = "requires the pinned native tool matrix"]
        fn $d() {
            run_case($name, Some(true));
        }
    };
}
tool_cases!("cargo", cargo_unconfined, cargo_exact, cargo_tooldirs);
tool_cases!("rustup", rustup_unconfined, rustup_exact, rustup_tooldirs);
tool_cases!("go", go_unconfined, go_exact, go_tooldirs);
tool_cases!("gradle", gradle_unconfined, gradle_exact, gradle_tooldirs);
tool_cases!("maven", maven_unconfined, maven_exact, maven_tooldirs);
tool_cases!("nuget", nuget_unconfined, nuget_exact, nuget_tooldirs);
tool_cases!(
    "composer",
    composer_unconfined,
    composer_exact,
    composer_tooldirs
);
