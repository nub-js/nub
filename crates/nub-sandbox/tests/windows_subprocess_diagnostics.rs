#![cfg(windows)]

#[path = "common/tool_output.rs"]
mod tool_output;

use nub_sandbox::{CommandSpec, CompileCtx, Homes, Sandbox, ScopeCapabilities, compile};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Clone, Deserialize, Serialize)]
struct Target {
    name: String,
    program: PathBuf,
    args: Vec<String>,
    root: PathBuf,
}

#[test]
fn subprocess_leaf() {
    println!("SUBPROCESS_LEAF_OK");
}

#[test]
fn subprocess_owner() {
    let Ok(target) = std::env::var("SANDBOX_DIAG_TARGET") else {
        return;
    };
    let target: Target = serde_json::from_str(&target).unwrap();
    let mode = std::env::var("SANDBOX_DIAG_MODE").unwrap();
    let project = PathBuf::from(std::env::var_os("SANDBOX_DIAG_PROJECT").unwrap());
    let canary = PathBuf::from(std::env::var_os("SANDBOX_DIAG_CANARY").unwrap());
    let null_read = File::open("NUL").map(|_| ()).map_err(|e| e.raw_os_error());
    let null_write = OpenOptions::new()
        .write(true)
        .open("NUL")
        .map(|_| ())
        .map_err(|e| e.raw_os_error());
    let mut command = Command::new(&target.program);
    command.args(&target.args).current_dir(&project);
    let captured = matches!(mode.as_str(), "output" | "file_stdin_output");
    match mode.as_str() {
        "inherit" => {}
        "null_stdin" => {
            command.stdin(Stdio::null());
        }
        "null_stdout" => {
            command.stdout(Stdio::null());
        }
        "pipe_stdin" => {
            command.stdin(Stdio::piped());
        }
        "output" => {}
        "file_stdin_output" => {
            command.stdin(File::open(project.join("input")).unwrap());
        }
        "file_output" => {
            command
                .stdin(File::open(project.join("input")).unwrap())
                .stdout(File::create(project.join("stdout")).unwrap())
                .stderr(File::create(project.join("stderr")).unwrap());
        }
        _ => panic!("unknown diagnostic mode"),
    }
    println!("SANDBOX_SUBPROCESS_START {} {mode}", target.name);
    let result = if captured {
        command.output().map(|output| {
            json!({"success": output.status.success(), "code": output.status.code(),
                "stdout": String::from_utf8_lossy(&output.stdout),
                "stderr": String::from_utf8_lossy(&output.stderr)})
        })
    } else {
        command.status().map(|status| {
            let mut result = json!({"success": status.success(), "code": status.code()});
            if mode == "file_output" {
                for name in ["stdout", "stderr"] {
                    result[name] = json!(String::from_utf8_lossy(
                        &std::fs::read(project.join(name)).unwrap()
                    ));
                }
            }
            result
        })
    };
    let result = result.unwrap_or_else(|error| {
        json!({"success": false, "os_error": error.raw_os_error(), "error": error.to_string()})
    });
    println!(
        "SANDBOX_SUBPROCESS_RESULT {}",
        json!({"target": target.name, "mode": mode, "result": result,
            "null_read": null_read, "null_write": null_write,
            "canary_readable": std::fs::read(canary).is_ok()})
    );
}

fn environment(target: &Target, mode: &str, root: &Path) -> BTreeMap<String, String> {
    let mut env: BTreeMap<_, _> = std::env::vars()
        .filter(|(key, _)| {
            ["PATH", "SYSTEMROOT", "WINDIR", "COMSPEC", "PATHEXT"]
                .contains(&key.to_ascii_uppercase().as_str())
        })
        .collect();
    env.insert(
        "SANDBOX_DIAG_TARGET".into(),
        serde_json::to_string(target).unwrap(),
    );
    env.insert("SANDBOX_DIAG_MODE".into(), mode.into());
    env.insert(
        "SANDBOX_DIAG_PROJECT".into(),
        root.join("project").to_str().unwrap().into(),
    );
    env.insert(
        "SANDBOX_DIAG_CANARY".into(),
        root.join("canary").to_str().unwrap().into(),
    );
    env
}

fn record(stdout: &[u8], stderr: &[u8]) -> Value {
    let stdout = String::from_utf8_lossy(stdout);
    stdout
        .lines()
        .find_map(|line| line.split_once("SANDBOX_SUBPROCESS_RESULT "))
        .map(|(_, json)| serde_json::from_str(json).unwrap())
        .unwrap_or_else(|| {
            panic!(
                "no diagnostic result\n{stdout}\n{}",
                String::from_utf8_lossy(stderr)
            )
        })
}

#[test]
#[ignore = "requires real Windows and an explicit interpreter matrix"]
fn windows_subprocess_startup_controls() {
    let matrix = std::env::var_os("SANDBOX_DIAG_MATRIX").expect("interpreter matrix required");
    let mut targets: Vec<Target> = serde_json::from_slice(&std::fs::read(matrix).unwrap()).unwrap();
    assert!(
        targets.len() >= 2,
        "at least two independent program controls must be supplied"
    );
    let exe = std::env::current_exe().unwrap();
    targets.push(Target {
        name: "test-binary".into(),
        program: exe.clone(),
        args: vec![
            "--exact".into(),
            "subprocess_leaf".into(),
            "--nocapture".into(),
        ],
        root: exe.parent().unwrap().into(),
    });
    let root = tempfile::Builder::new()
        .prefix("sandbox-subprocess-")
        .tempdir_in(std::env::var_os("USERPROFILE").unwrap())
        .unwrap();
    let project = root.path().join("project");
    std::fs::create_dir(&project).unwrap();
    std::fs::write(project.join("input"), "").unwrap();
    std::fs::write(root.path().join("canary"), "synthetic-canary").unwrap();
    let mut failed_descriptor_controls = Vec::new();
    for target in &targets {
        for mode in [
            "inherit",
            "null_stdin",
            "null_stdout",
            "pipe_stdin",
            "output",
            "file_output",
            "file_stdin_output",
        ] {
            let env = environment(target, mode, root.path());
            let control = Command::new(&exe)
                .args(["--exact", "subprocess_owner", "--nocapture"])
                .env_clear()
                .envs(&env)
                .current_dir(&project)
                .output()
                .unwrap();
            assert!(
                control.status.success(),
                "unconfined owner failed: {control:?}"
            );
            let control = record(&control.stdout, &control.stderr);
            assert_eq!(
                control["result"]["success"], true,
                "unconfined control: {control}"
            );
            assert_eq!(control["canary_readable"], true);
            println!("SUBPROCESS_CONTROL {control}");

            let ctx = CompileCtx::new(
                Homes {
                    home: root.path().join("home"),
                    cache: root.path().join("cache"),
                    tmp: root.path().join("tmp"),
                    project: project.clone(),
                },
                project.clone(),
                ScopeCapabilities::approved(),
                env.clone(),
            );
            let mut input = json!({"fs": {"./": "rw", "$tmp": "rw"}, "net": false});
            input["fs"][target.root.to_str().unwrap()] = json!("r");
            input["fs"][exe.parent().unwrap().to_str().unwrap()] = json!("r");
            let mut policy = compile(&input, &ctx).unwrap();
            policy.env.constructed = env;
            let sandbox = Sandbox::new(&policy).unwrap();
            let prepared = sandbox
                .prepare(
                    CommandSpec::new(&exe)
                        .args(["--exact", "subprocess_owner", "--nocapture"])
                        .cwd(&project)
                        .redact_stdout(true)
                        .redact_stderr(true),
                )
                .unwrap();
            assert!(
                prepared.degradation.lost.is_empty(),
                "{:?}",
                prepared.degradation
            );
            let output = tool_output::output(prepared);
            assert!(output.status.success(), "confined owner failed: {output:?}");
            let mut result = record(&output.stdout, &output.stderr);
            result["owner_stderr"] = json!(String::from_utf8_lossy(&output.stderr));
            assert_eq!(
                result["canary_readable"], false,
                "confinement control: {result}"
            );
            println!("SUBPROCESS_CONFINED {result}");
            if (mode == "inherit" || mode == "file_output") && result["result"]["success"] != true {
                failed_descriptor_controls.push(result);
            }
            sandbox.close();
        }
    }
    nub_sandbox::cleanup().unwrap();
    assert!(
        failed_descriptor_controls.is_empty(),
        "descriptor-backed controls failed: {failed_descriptor_controls:?}"
    );
}
