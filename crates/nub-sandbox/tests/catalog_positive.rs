use nub_sandbox::policy::{Effect, FsAccess};
use nub_sandbox::{Homes, compile_build_jail};
use std::collections::BTreeMap;
use std::path::PathBuf;

fn policy_for(
    package: &str,
    version: &str,
) -> (tempfile::TempDir, nub_sandbox::SandboxPolicy, Homes) {
    let root = tempfile::tempdir().expect("fixture root");
    let home = root.path().join("home");
    let project = root.path().join("project");
    let cache = root.path().join("cache");
    let package_dir = project.join("node_modules").join("fixture");
    for path in [&home, &project, &cache, &package_dir] {
        std::fs::create_dir_all(path).expect("fixture directory");
    }
    let homes = Homes {
        home,
        cache,
        tmp: root.path().join("tmp"),
        project,
    };
    let policy = compile_build_jail(
        homes.clone(),
        &package_dir,
        Some(package),
        Some(version),
        vec![PathBuf::from(if cfg!(windows) {
            "C:/Windows/System32/cmd.exe"
        } else {
            "/bin/sh"
        })],
        Vec::new(),
        BTreeMap::new(),
    )
    .expect("catalog build-jail policy compiles");
    (root, policy, homes)
}

#[test]
fn catalog_user_home_grant_includes_a_credential_canary() {
    let (_root, policy, homes) = policy_for("@pulumi/aws-native", "1.0.0");
    let canary = homes.home.join(".npmrc");
    let matcher = nub_sandbox::matcher::PathMatcher::new(&policy.fs.rules);

    assert_eq!(
        matcher.decide(&canary).effect,
        Effect::Allow,
        "the real catalog userHome grant must be literal, including credential-bearing files"
    );
    assert!(
        policy.fs.rules.entries.iter().any(|rule| {
            rule.effect == Effect::Allow
                && rule.access == FsAccess::ReadWrite
                && rule
                    .matcher
                    .as_str()
                    .contains(homes.home.to_string_lossy().as_ref())
        }),
        "the userHome catalog grant must remain writable"
    );
}

#[cfg(target_os = "linux")]
#[test]
fn catalog_disk_read_is_a_literal_read_only_root_grant() {
    let (_root, policy, _homes) = policy_for("@mui/x-telemetry", "1.0.0");

    assert!(
        policy.fs.rules.entries.iter().any(|rule| {
            rule.effect == Effect::Allow
                && rule.access == FsAccess::Read
                && rule.matcher.as_str() == "**"
        }),
        "the real read:disk catalog entry must emit a literal root read grant"
    );
    assert_eq!(
        policy.fs.rules.default_effect,
        Effect::Deny,
        "read:disk must not make the filesystem writable"
    );
}
