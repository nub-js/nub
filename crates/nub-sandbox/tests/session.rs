use nub_sandbox::{
    CommandSpec, CompileCtx, Homes, Sandbox, SandboxPolicy, ScopeCapabilities, compile,
};
use std::collections::BTreeMap;
use std::path::Path;

fn resolved_policy(root: &Path) -> SandboxPolicy {
    let environment: BTreeMap<String, String> = ["PATH", "SystemRoot", "WINDIR", "COMSPEC"]
        .into_iter()
        .filter_map(|key| std::env::var(key).ok().map(|value| (key.into(), value)))
        .collect();
    let context = CompileCtx::new(
        Homes {
            home: root.into(),
            cache: root.into(),
            tmp: root.into(),
            project: root.into(),
        },
        root.into(),
        ScopeCapabilities::approved(),
        environment.clone(),
    );
    let mut policy = compile(
        &serde_json::json!({"fs": {"./": "rw", "$tmp": "rw"}, "net": false}),
        &context,
    )
    .unwrap();
    policy.env.constructed = environment;
    policy
}

#[cfg(unix)]
fn echo_command(value: &str) -> CommandSpec {
    CommandSpec::new("/bin/sh")
        .arg("-c")
        .arg(format!("printf '{value}\\n'"))
}

#[cfg(windows)]
fn echo_command(value: &str) -> CommandSpec {
    CommandSpec::new("cmd.exe")
        .args(["/d", "/s", "/c"])
        .arg(format!("echo {value}"))
}

fn expected_echo(value: &str) -> Vec<u8> {
    if cfg!(windows) {
        format!("{value}\r\n").into_bytes()
    } else {
        format!("{value}\n").into_bytes()
    }
}

#[test]
fn acquired_sandbox_submits_independent_commands() {
    let root = tempfile::tempdir().unwrap();
    let sandbox =
        Sandbox::acquire(&resolved_policy(root.path())).expect("resolved policy acquires");
    let first = sandbox
        .prepare(echo_command("one").cwd(root.path()))
        .expect("first command prepares")
        .output()
        .expect("first command runs");
    let second = sandbox
        .prepare(echo_command("two").cwd(root.path()))
        .expect("second command prepares")
        .output()
        .expect("second command runs");

    assert!(first.status.success());
    assert!(second.status.success());
    assert_eq!(first.stdout, expected_echo("one"));
    assert_eq!(second.stdout, expected_echo("two"));
}

#[test]
fn prepared_command_keeps_acquired_resources_alive_after_close() {
    let root = tempfile::tempdir().unwrap();
    let sandbox = Sandbox::new(&resolved_policy(root.path())).expect("resolved policy acquires");
    let prepared = sandbox
        .prepare(echo_command("live").cwd(root.path()))
        .expect("command prepares before close");
    sandbox.close();

    let output = prepared
        .output()
        .expect("prepared command still owns its lease");
    assert!(output.status.success());
    assert_eq!(output.stdout, expected_echo("live"));
}

#[test]
fn acquisition_requires_a_resolved_policy() {
    let error = match Sandbox::new(&SandboxPolicy::default()) {
        Ok(_) => panic!("unresolved policy must fail closed"),
        Err(error) => error,
    };
    assert_eq!(error.lost, vec!["env-unresolved"]);
}
