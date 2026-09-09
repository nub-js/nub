use nub_sandbox::{CommandSpec, Sandbox, SandboxPolicy};

fn resolved_policy() -> SandboxPolicy {
    let mut policy = SandboxPolicy::default();
    policy.env.resolved = true;
    policy
}

#[cfg(unix)]
fn echo_command(value: &str) -> CommandSpec {
    CommandSpec::new("/bin/sh")
        .arg("-c")
        .arg(format!("printf {value}"))
}

#[cfg(windows)]
fn echo_command(value: &str) -> CommandSpec {
    CommandSpec::new("cmd.exe")
        .args(["/d", "/s", "/c"])
        .arg(format!("<nul set /p ={value}"))
}

#[test]
fn acquired_sandbox_submits_independent_commands() {
    let sandbox = Sandbox::acquire(&resolved_policy()).expect("resolved policy acquires");
    let first = sandbox
        .prepare(echo_command("one"))
        .expect("first command prepares")
        .output()
        .expect("first command runs");
    let second = sandbox
        .prepare(echo_command("two"))
        .expect("second command prepares")
        .output()
        .expect("second command runs");

    assert!(first.status.success());
    assert!(second.status.success());
    assert_eq!(first.stdout, b"one");
    assert_eq!(second.stdout, b"two");
}

#[test]
fn prepared_command_keeps_acquired_resources_alive_after_close() {
    let sandbox = Sandbox::new(&resolved_policy()).expect("resolved policy acquires");
    let prepared = sandbox
        .prepare(echo_command("live"))
        .expect("command prepares before close");
    sandbox.close();

    let output = prepared
        .output()
        .expect("prepared command still owns its lease");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"live");
}

#[test]
fn acquisition_requires_a_resolved_policy() {
    let error = match Sandbox::new(&SandboxPolicy::default()) {
        Ok(_) => panic!("unresolved policy must fail closed"),
        Err(error) => error,
    };
    assert_eq!(error.lost, vec!["env-unresolved"]);
}
