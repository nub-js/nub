use std::process::Command;

fn invoke(args: &[&str], cwd: &std::path::Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_nub"))
        .args(args)
        .current_dir(cwd)
        .output()
        .unwrap()
}

#[test]
fn cleanup_help_is_routable_without_project_configuration() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("nub.jsonc"), "not json").unwrap();
    for args in [
        vec!["sandbox"],
        vec!["sandbox", "--help"],
        vec!["help", "sandbox"],
        vec!["sandbox", "cleanup", "--help"],
    ] {
        let output = invoke(&args, root.path());
        assert!(output.status.success(), "{args:?}: {output:?}");
        assert!(String::from_utf8_lossy(&output.stdout).contains("nub sandbox cleanup"));
    }
}

#[test]
fn cleanup_rejects_unknown_operations_and_extra_arguments() {
    let root = tempfile::tempdir().unwrap();
    for args in [
        vec!["sandbox", "destroy"],
        vec!["sandbox", "cleanup", "all"],
        vec!["sandbox", "cleanup", "--force"],
    ] {
        assert!(!invoke(&args, root.path()).status.success());
    }
}

#[cfg(unix)]
#[test]
fn cleanup_is_a_noop_without_persistent_os_grants() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("nub.jsonc"), "not json").unwrap();
    let output = invoke(&["sandbox", "cleanup"], root.path());
    assert!(output.status.success(), "{output:?}");
    assert!(String::from_utf8_lossy(&output.stdout).contains("No persistent sandbox OS grants"));
}
