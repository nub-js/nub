//! Native Git operations under unconfined, exact-grant, and `$tooldirs` controls.
//!
//! These tests intentionally use a synthetic HOME outside the OS temporary directory. They are
//! opt-in native tests: the dedicated CI job runs them after selecting the runner's Git binary.

#[path = "common/tool_output.rs"]
mod tool_output;

use nub_sandbox::{CommandSpec, CompileCtx, Homes, Sandbox, ScopeCapabilities, compile};
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Control {
    Unconfined,
    Exact,
    ToolDirs,
}

impl Control {
    fn name(self) -> &'static str {
        match self {
            Self::Unconfined => "unconfined",
            Self::Exact => "exact grants",
            Self::ToolDirs => "$tooldirs",
        }
    }
}

fn fixture() -> tempfile::TempDir {
    let runner_home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .expect("runner home");
    let root = tempfile::Builder::new()
        .prefix("sandbox-git-tool-")
        .tempdir_in(runner_home)
        .expect("fixture root");
    for path in ["home", "project", "cache"] {
        std::fs::create_dir_all(root.path().join(path)).expect("fixture directory");
    }
    root
}

fn environment(root: &Path, extra: &[(&str, String)]) -> BTreeMap<String, String> {
    let mut env = BTreeMap::new();
    for key in [
        "PATH",
        "SystemRoot",
        "SYSTEMROOT",
        "WINDIR",
        "COMSPEC",
        "PATHEXT",
    ] {
        if let Ok(value) = std::env::var(key) {
            env.insert(key.to_owned(), value);
        }
    }
    env.insert("HOME".into(), root.join("home").to_string_lossy().into());
    env.insert(
        "USERPROFILE".into(),
        root.join("home").to_string_lossy().into(),
    );
    env.insert(
        "XDG_CONFIG_HOME".into(),
        root.join("home/.config").to_string_lossy().into(),
    );
    for (key, value) in extra {
        env.insert((*key).to_owned(), value.clone());
    }
    env
}

fn exact_grants(paths: &[(&Path, &str)]) -> Value {
    let mut entries = Map::new();
    for (path, access) in paths {
        entries.insert(
            path.to_string_lossy().into_owned(),
            Value::String((*access).into()),
        );
    }
    Value::Object(entries)
}

fn policy(
    root: &Path,
    control: Control,
    extra_env: &[(&str, String)],
    extra_exact: &[(&Path, &str)],
) -> nub_sandbox::SandboxPolicy {
    let home = root.join("home");
    let remote = root.join("remote.git");
    let mut grants = match control {
        Control::Unconfined => unreachable!("an unconfined control has no policy"),
        Control::Exact => exact_grants(&[
            (&remote, "rw"),
            // Git replaces this conventional global config through this adjacent lock. These
            // are deliberately files rather than a writable synthetic HOME.
            (&home.join(".gitconfig"), "rw"),
            (&home.join(".gitconfig.lock"), "rw"),
        ]),
        Control::ToolDirs => {
            let mut paths = Map::new();
            paths.insert("$tooldirs".into(), Value::String("rw".into()));
            paths.insert(
                remote.to_string_lossy().into_owned(),
                Value::String("rw".into()),
            );
            Value::Object(paths)
        }
    };
    let Value::Object(ref mut entries) = grants else {
        unreachable!("all Git grants are object-form");
    };
    for (path, access) in extra_exact {
        entries.insert(
            path.to_string_lossy().into_owned(),
            Value::String((*access).into()),
        );
    }
    entries.insert("./".into(), Value::String("rw".into()));
    entries.insert("$tmp".into(), Value::String("rw".into()));

    let env = environment(root, extra_env);
    let homes = Homes {
        home,
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
    let mut policy =
        compile(&json!({"fs": grants, "net": false}), &ctx).expect("Git fixture policy compiles");
    policy.env.constructed = env;
    policy
}

fn output_message(output: &Output) -> String {
    format!(
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn assert_success(phase: &str, output: Output) -> Output {
    assert!(
        output.status.success(),
        "{phase} failed:\n{}",
        output_message(&output)
    );
    output
}

fn host(root: &Path, cwd: &Path, args: &[&str]) -> Output {
    eprintln!("SETUP git {args:?}");
    let mut command = Command::new("git");
    command
        .args(args)
        .current_dir(cwd)
        .env_clear()
        .envs(environment(root, &[]));
    command.output().expect("Git setup command launches")
}

fn invoke(
    root: &Path,
    cwd: &Path,
    args: &[&str],
    control: Control,
    policy: Option<&nub_sandbox::SandboxPolicy>,
    extra_env: &[(&str, String)],
) -> Output {
    match control {
        Control::Unconfined => {
            eprintln!("UNCONFINED git {args:?}");
            let mut command = Command::new("git");
            command
                .args(args)
                .current_dir(cwd)
                .env_clear()
                .envs(environment(root, extra_env));
            command.output().expect("unconfined Git command launches")
        }
        Control::Exact | Control::ToolDirs => {
            eprintln!("CONFINED {} git {args:?}", control.name());
            let sandbox =
                Sandbox::new(policy.expect("confined Git policy")).expect("Git sandbox acquires");
            let prepared = sandbox
                .prepare(
                    CommandSpec::new("git")
                        .args(args)
                        .cwd(cwd)
                        .redact_stdout(true)
                        .redact_stderr(true),
                )
                .expect("Git command prepares without degradation");
            assert!(
                prepared.degradation.lost.is_empty(),
                "{} Git fixture degraded: {:?}",
                control.name(),
                prepared.degradation
            );
            tool_output::output(prepared)
        }
    }
}

fn require_git() {
    let output = Command::new("git")
        .arg("--version")
        .output()
        .expect("standard GitHub runner supplies Git");
    let output = assert_success("git --version", output);
    eprintln!(
        "native Git: {}",
        String::from_utf8_lossy(&output.stdout).trim()
    );
}

fn prepare_remote(root: &Path) {
    let remote = root.join("remote.git");
    let publisher = root.join("publisher");
    assert_success(
        "create bare remote",
        host(root, root, &["init", "--bare", remote.to_str().unwrap()]),
    );
    assert_success(
        "create publisher",
        host(root, root, &["init", publisher.to_str().unwrap()]),
    );
    std::fs::write(publisher.join("README.md"), "initial\n").expect("initial source");
    assert_success(
        "stage initial source",
        host(root, &publisher, &["add", "README.md"]),
    );
    assert_success(
        "commit initial source",
        host(
            root,
            &publisher,
            &[
                "-c",
                "user.name=Sandbox Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "commit",
                "-m",
                "initial",
            ],
        ),
    );
    assert_success(
        "add fixture remote",
        host(
            root,
            &publisher,
            &["remote", "add", "origin", remote.to_str().unwrap()],
        ),
    );
    assert_success(
        "push initial source",
        host(root, &publisher, &["push", "origin", "HEAD:main"]),
    );
    assert_success(
        "set fixture remote default branch",
        host(
            root,
            root,
            &[
                "--git-dir",
                remote.to_str().unwrap(),
                "symbolic-ref",
                "HEAD",
                "refs/heads/main",
            ],
        ),
    );
}

fn advance_remote(root: &Path) {
    let remote = root.join("remote.git");
    let upstream = root.join("upstream");
    assert_success(
        "clone upstream fixture",
        host(
            root,
            root,
            &[
                "clone",
                remote.to_str().unwrap(),
                upstream.to_str().unwrap(),
            ],
        ),
    );
    std::fs::write(upstream.join("upstream.txt"), "upstream\n").expect("upstream source");
    assert_success(
        "stage upstream source",
        host(root, &upstream, &["add", "upstream.txt"]),
    );
    assert_success(
        "commit upstream source",
        host(
            root,
            &upstream,
            &[
                "-c",
                "user.name=Sandbox Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "commit",
                "-m",
                "upstream",
            ],
        ),
    );
    assert_success(
        "push upstream source",
        host(root, &upstream, &["push", "origin", "HEAD:main"]),
    );
}

fn conventional_global_config(root: &Path) -> PathBuf {
    let config = root.join("home/.gitconfig");
    std::fs::write(
        &config,
        "[user]\n\tname = Sandbox Fixture\n\temail = fixture@example.invalid\n",
    )
    .expect("conventional global config fixture");
    config
}

fn run_conventional_config_write(control: Control) {
    require_git();
    let root = fixture();
    let config = conventional_global_config(root.path());
    let policy = (control != Control::Unconfined).then(|| policy(root.path(), control, &[], &[]));
    assert_success(
        "write conventional global config through its adjacent lock",
        invoke(
            root.path(),
            &root.path().join("project"),
            &[
                "config",
                "--global",
                "user.email",
                "updated@example.invalid",
            ],
            control,
            policy.as_ref(),
            &[],
        ),
    );
    assert!(
        std::fs::read_to_string(&config)
            .unwrap()
            .contains("updated@example.invalid")
    );
    assert!(
        !config.with_extension("gitconfig.lock").exists(),
        "Git must clean up its adjacent lock"
    );
}

fn run_operations(control: Control) {
    require_git();
    let root = fixture();
    prepare_remote(root.path());
    conventional_global_config(root.path());
    let policy = (control != Control::Unconfined).then(|| policy(root.path(), control, &[], &[]));
    let project = root.path().join("project");
    let remote = root.path().join("remote.git");
    let clone = project.join("clone");

    let configured = assert_success(
        "read conventional global config",
        invoke(
            root.path(),
            &project,
            &["config", "--global", "--get", "user.email"],
            control,
            policy.as_ref(),
            &[],
        ),
    );
    assert_eq!(
        String::from_utf8_lossy(&configured.stdout).trim(),
        "fixture@example.invalid"
    );
    assert!(root.path().join("home/.gitconfig").exists());

    assert_success(
        "local clone",
        invoke(
            root.path(),
            &project,
            &["clone", remote.to_str().unwrap(), clone.to_str().unwrap()],
            control,
            policy.as_ref(),
            &[],
        ),
    );
    std::fs::write(clone.join("change.txt"), "change\n").expect("untracked fixture source");
    let status = assert_success(
        "status untracked source",
        invoke(
            root.path(),
            &clone,
            &["status", "--porcelain"],
            control,
            policy.as_ref(),
            &[],
        ),
    );
    assert!(String::from_utf8_lossy(&status.stdout).contains("?? change.txt"));
    assert_success(
        "stage source",
        invoke(
            root.path(),
            &clone,
            &["add", "change.txt"],
            control,
            policy.as_ref(),
            &[],
        ),
    );
    assert_success(
        "commit source",
        invoke(
            root.path(),
            &clone,
            &["commit", "-m", "sandbox change"],
            control,
            policy.as_ref(),
            &[],
        ),
    );
    assert_success(
        "push source",
        invoke(
            root.path(),
            &clone,
            &["push", "origin", "HEAD:main"],
            control,
            policy.as_ref(),
            &[],
        ),
    );

    advance_remote(root.path());
    assert_success(
        "fetch remote advance",
        invoke(
            root.path(),
            &clone,
            &["fetch", "origin"],
            control,
            policy.as_ref(),
            &[],
        ),
    );

    let linked = project.join("linked");
    assert_success(
        "create linked worktree",
        invoke(
            root.path(),
            &clone,
            &[
                "worktree",
                "add",
                "-b",
                "sandbox-linked",
                linked.to_str().unwrap(),
            ],
            control,
            policy.as_ref(),
            &[],
        ),
    );
    assert!(
        linked.join(".git").is_file(),
        "linked worktree must use a .git indirection file"
    );
    assert!(
        std::fs::read_dir(clone.join(".git/worktrees"))
            .expect("linked-worktree common directory")
            .next()
            .is_some(),
        "linked worktree must create common-dir metadata"
    );
    assert_success(
        "linked worktree status",
        invoke(
            root.path(),
            &linked,
            &["status", "--porcelain"],
            control,
            policy.as_ref(),
            &[],
        ),
    );
}

#[test]
#[ignore = "requires native Git tool functionality job"]
fn git_native_unconfined_conventional_global_config_write() {
    run_conventional_config_write(Control::Unconfined);
}

#[test]
#[ignore = "requires native Git tool functionality job"]
fn git_native_exact_conventional_global_config_write() {
    run_conventional_config_write(Control::Exact);
}

#[test]
#[ignore = "requires native Git tool functionality job"]
fn git_native_tooldirs_conventional_global_config_write() {
    run_conventional_config_write(Control::ToolDirs);
}

#[test]
#[ignore = "requires native Git tool functionality job"]
fn git_native_unconfined_status_add_commit_clone_fetch_push_and_worktree() {
    run_operations(Control::Unconfined);
}

#[test]
#[ignore = "requires native Git tool functionality job"]
fn git_native_exact_status_add_commit_clone_fetch_push_and_worktree() {
    run_operations(Control::Exact);
}

#[test]
#[ignore = "requires native Git tool functionality job"]
fn git_native_tooldirs_status_add_commit_clone_fetch_push_and_worktree() {
    run_operations(Control::ToolDirs);
}

#[test]
#[ignore = "requires native Git tool functionality job"]
fn git_documented_global_config_relocation_needs_an_explicit_grant() {
    require_git();
    let root = fixture();
    let project = root.path().join("project");
    let config = root.path().join("explicit/global.gitconfig");
    std::fs::create_dir_all(config.parent().unwrap()).expect("explicit config parent");
    std::fs::write(&config, "[user]\n\tname = Before\n").expect("explicit config fixture");
    let env = [("GIT_CONFIG_GLOBAL", config.to_string_lossy().into_owned())];
    let args = [
        "config",
        "--global",
        "user.email",
        "fixture@example.invalid",
    ];

    assert_success(
        "unconfined relocated global config",
        invoke(
            root.path(),
            &project,
            &args,
            Control::Unconfined,
            None,
            &env,
        ),
    );
    let exact = policy(
        root.path(),
        Control::Exact,
        &env,
        &[(config.parent().unwrap(), "rw")],
    );
    assert_success(
        "explicitly granted relocated global config",
        invoke(
            root.path(),
            &project,
            &args,
            Control::Exact,
            Some(&exact),
            &env,
        ),
    );
    assert!(
        std::fs::read_to_string(&config)
            .unwrap()
            .contains("fixture@example.invalid")
    );
    assert!(!config.with_extension("gitconfig.lock").exists());
}

fn git_lfs_program() -> PathBuf {
    let executable = if cfg!(windows) {
        "git-lfs.exe"
    } else {
        "git-lfs"
    };
    let available = Command::new("git")
        .args(["lfs", "version"])
        .output()
        .expect("Git LFS must be provisioned for the native Git fixture");
    let available = assert_success(
        "git lfs version (required native fixture provision)",
        available,
    );
    eprintln!(
        "native Git LFS: {}",
        String::from_utf8_lossy(&available.stdout).trim()
    );

    let path_candidate = std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .map(|path| path.join(executable))
            .find(|candidate| candidate.is_file())
    });
    let exec_path_candidate = Command::new("git")
        .arg("--exec-path")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| PathBuf::from(String::from_utf8_lossy(&output.stdout).trim()))
        .map(|path| path.join(executable))
        .filter(|candidate| candidate.is_file());
    path_candidate
        .or(exec_path_candidate)
        .expect("Git LFS is available but its executable is not discoverable for an exact grant")
}

fn run_lfs(control: Control) {
    require_git();
    let lfs = git_lfs_program();
    let root = fixture();
    prepare_remote(root.path());
    let policy =
        (control != Control::Unconfined).then(|| policy(root.path(), control, &[], &[(&lfs, "r")]));
    let project = root.path().join("project");
    let clone = project.join("clone");
    assert_success(
        "clone LFS fixture",
        invoke(
            root.path(),
            &project,
            &[
                "clone",
                root.path().join("remote.git").to_str().unwrap(),
                clone.to_str().unwrap(),
            ],
            control,
            policy.as_ref(),
            &[],
        ),
    );
    assert_success(
        "install local LFS hooks",
        invoke(
            root.path(),
            &clone,
            &["lfs", "install", "--local"],
            control,
            policy.as_ref(),
            &[],
        ),
    );
    assert_success(
        "track LFS fixture",
        invoke(
            root.path(),
            &clone,
            &["lfs", "track", "*.bin"],
            control,
            policy.as_ref(),
            &[],
        ),
    );
    assert!(
        std::fs::read_to_string(clone.join(".gitattributes"))
            .unwrap()
            .contains("*.bin filter=lfs")
    );
}

#[test]
#[ignore = "requires native Git tool functionality job"]
fn git_lfs_native_unconfined_when_available() {
    run_lfs(Control::Unconfined);
}

#[test]
#[ignore = "requires native Git tool functionality job"]
fn git_lfs_native_exact_when_available() {
    run_lfs(Control::Exact);
}

#[test]
#[ignore = "requires native Git tool functionality job"]
fn git_lfs_native_tooldirs_when_available() {
    run_lfs(Control::ToolDirs);
}
