//! libuv threadpool sizing — `UV_THREADPOOL_SIZE` set to `max(4, cores)` on an
//! augmented run.
//!
//! Node reads the variable once at startup, so the spawn is the only place it
//! can be set; nub sets it when the user has not, and leaves a user value alone.
//! Under `--node` / `NODE_COMPAT` the variable is absent, the plain-Node
//! fingerprint (libuv's own default of 4). A value from an env file is the
//! user's too, on every launch path, while a shell value still beats the file.

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

fn nub_binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_nub"))
}

fn fixture() -> PathBuf {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    Path::new(&manifest).join("../../tests/fixtures/threadpool/size.js")
}

/// Run the fixture under `nub [extra_args] [env]` and parse its JSON.
fn run(extra_args: &[&str], env: &[(&str, &str)]) -> serde_json::Value {
    let f = fixture();
    let mut cmd = Command::new(nub_binary());
    cmd.args(extra_args)
        .arg(&f)
        .current_dir(f.parent().unwrap())
        .env_remove("UV_THREADPOOL_SIZE");
    for (k, v) in env {
        cmd.env(k, v);
    }
    let output = cmd.output().expect("failed to spawn nub");
    assert!(
        output.status.success(),
        "nub exited {:?}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_str(String::from_utf8_lossy(&output.stdout).trim())
        .expect("fixture must emit valid JSON")
}

/// Augmented run: the pool is at least libuv's default of 4 and never exceeds the
/// parallelism Node itself reports (the two runtimes read a cgroup quota
/// differently, so the exact value is not pinned).
#[test]
fn augmented_sizes_pool_to_cores() {
    let v = run(&[], &[]);
    let size: usize = v["size"]
        .as_str()
        .expect("UV_THREADPOOL_SIZE must be set on an augmented run")
        .parse()
        .expect("UV_THREADPOOL_SIZE must be an integer");
    let cores = v["cores"].as_u64().unwrap() as usize;
    assert!(
        size >= 4,
        "pool must be at least libuv's default, got {size}"
    );
    assert!(
        size <= cores.max(4),
        "pool must not exceed max(4, cores={cores}), got {size}"
    );
}

/// `nub run` goes through the shared script-runner environment rather than the
/// direct spawn, so it is covered on its own: the script's `node` child carries
/// the same value a direct run gets.
#[test]
fn run_script_children_get_the_same_pool() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::copy(fixture(), dir.path().join("size.js")).unwrap();
    std::fs::write(
        dir.path().join("package.json"),
        r#"{ "name": "tp", "private": true, "scripts": { "probe": "node size.js" } }"#,
    )
    .unwrap();
    let mut cmd = Command::new(nub_binary());
    cmd.args(["run", "probe"])
        .current_dir(dir.path())
        .env_remove("UV_THREADPOOL_SIZE");
    let output = cmd.output().expect("failed to spawn nub");
    assert!(
        output.status.success(),
        "nub run exited {:?}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json = stdout
        .lines()
        .rev()
        .find(|l| l.trim_start().starts_with('{'))
        .expect("fixture JSON line in `nub run` output");
    let v: serde_json::Value = serde_json::from_str(json.trim()).unwrap();
    let direct = run(&[], &[]);
    assert_eq!(
        v["size"], direct["size"],
        "`nub run` must size the pool exactly as a direct run does"
    );
}

/// A value the user set is theirs, whatever the core count.
#[test]
fn user_value_is_never_overwritten() {
    let v = run(&[], &[("UV_THREADPOOL_SIZE", "3")]);
    assert_eq!(v["size"].as_str(), Some("3"));
}

/// A project whose `.env` sets the pool, with both fixtures and a `probe` script.
fn project_with_env_file() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    for name in ["size.js", "watch-size.js"] {
        std::fs::copy(fixture().with_file_name(name), dir.path().join(name)).unwrap();
    }
    std::fs::write(
        dir.path().join("package.json"),
        r#"{ "name": "tp", "private": true, "scripts": { "probe": "node size.js" } }"#,
    )
    .unwrap();
    std::fs::write(dir.path().join(".env"), "UV_THREADPOOL_SIZE=3\n").unwrap();
    dir
}

/// The first JSON line `nub <args>` prints from `dir`, killed if it outlives
/// `limit` (a watch supervisor never exits on its own).
fn first_json_line(dir: &Path, args: &[&str], env: &[(&str, &str)], limit: Duration) -> String {
    let mut cmd = Command::new(nub_binary());
    cmd.args(args)
        .current_dir(dir)
        .env_remove("UV_THREADPOOL_SIZE")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (k, v) in env {
        cmd.env(k, v);
    }
    let mut child = cmd.spawn().expect("failed to spawn nub");
    let stdout = child.stdout.take().unwrap();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            if line.trim_start().starts_with('{') {
                let _ = tx.send(line);
                break;
            }
        }
    });
    let line = rx.recv_timeout(limit);
    let _ = child.kill();
    let _ = child.wait();
    line.unwrap_or_else(|_| {
        panic!(
            "no JSON line from `nub {}` within {limit:?}",
            args.join(" ")
        )
    })
}

/// `.env` is the user's value on a direct run, and the shell still beats it.
#[test]
fn env_file_value_wins_on_a_direct_run_but_not_over_the_shell() {
    let dir = project_with_env_file();
    let from_file = first_json_line(dir.path(), &["size.js"], &[], Duration::from_secs(60));
    let v: serde_json::Value = serde_json::from_str(&from_file).unwrap();
    assert_eq!(
        v["size"].as_str(),
        Some("3"),
        "the .env value must reach the script"
    );
    let from_shell = first_json_line(
        dir.path(),
        &["size.js"],
        &[("UV_THREADPOOL_SIZE", "7")],
        Duration::from_secs(60),
    );
    let v: serde_json::Value = serde_json::from_str(&from_shell).unwrap();
    assert_eq!(
        v["size"].as_str(),
        Some("7"),
        "a shell value must beat the .env value"
    );
}

/// `nub run` installs nub's default, then the script's `node` re-enters nub
/// through the shim, where `.env` is loaded: the installed default must read as
/// nub's, not as a shell value the file may not touch.
#[test]
fn env_file_value_beats_the_installed_default_under_run() {
    let dir = project_with_env_file();
    let line = first_json_line(dir.path(), &["run", "probe"], &[], Duration::from_secs(60));
    let v: serde_json::Value = serde_json::from_str(&line).unwrap();
    assert_eq!(v["size"].as_str(), Some("3"));
}

/// A non-Node local bin under `nub exec` has its `--env-file` values staged
/// before augmentation runs; the pool default must not land on top of them.
#[cfg(unix)]
#[test]
fn env_file_value_survives_exec_of_a_non_node_bin() {
    use std::os::unix::fs::PermissionsExt as _;
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("package.json"),
        r#"{ "name": "tp", "private": true }"#,
    )
    .unwrap();
    std::fs::write(dir.path().join("custom.env"), "UV_THREADPOOL_SIZE=5\n").unwrap();
    let bin_dir = dir.path().join("node_modules").join(".bin");
    std::fs::create_dir_all(&bin_dir).unwrap();
    let bin = bin_dir.join("pool-echo");
    std::fs::write(
        &bin,
        r#"#!/bin/sh
echo "{\"size\":\"$UV_THREADPOOL_SIZE\"}"
"#,
    )
    .unwrap();
    std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o755)).unwrap();
    let from_file = first_json_line(
        dir.path(),
        &["--env-file=custom.env", "exec", "pool-echo"],
        &[],
        Duration::from_secs(60),
    );
    let v: serde_json::Value = serde_json::from_str(&from_file).unwrap();
    assert_eq!(
        v["size"].as_str(),
        Some("5"),
        "the --env-file value must reach the bin"
    );
    let plain = first_json_line(
        dir.path(),
        &["exec", "pool-echo"],
        &[],
        Duration::from_secs(60),
    );
    let v: serde_json::Value = serde_json::from_str(&plain).unwrap();
    let size: usize = v["size"]
        .as_str()
        .unwrap()
        .parse()
        .expect("nub's default must be a number");
    assert!(
        size >= 4,
        "without a file the bin gets nub's default, got {size}"
    );
}

/// Windows environment keys are case-insensitive, so a differently cased key in
/// `.env` is the same user value, at the nested `nub run` boundary as well.
#[cfg(windows)]
#[test]
fn env_file_key_case_is_folded_on_windows() {
    let dir = project_with_env_file();
    std::fs::write(dir.path().join(".env"), "uv_threadpool_size=3\n").unwrap();
    for args in [&["size.js"][..], &["run", "probe"][..]] {
        let line = first_json_line(dir.path(), args, &[], Duration::from_secs(60));
        let v: serde_json::Value = serde_json::from_str(&line).unwrap();
        assert_eq!(
            v["size"].as_str(),
            Some("3"),
            "under `nub {}`",
            args.join(" ")
        );
    }
}

/// Watch forwards `.env` to Node's own `--env-file`, which never overrides a
/// value already in the command environment, so nub must not pre-install one.
#[test]
fn env_file_value_wins_in_watch_mode() {
    let dir = project_with_env_file();
    let line = first_json_line(
        dir.path(),
        &["watch", "watch-size.js"],
        &[],
        Duration::from_secs(90),
    );
    let v: serde_json::Value = serde_json::from_str(&line).unwrap();
    assert_eq!(v["size"].as_str(), Some("3"));
}

/// `--node` is plain Node: the variable is absent.
#[test]
fn node_compat_flag_leaves_pool_alone() {
    let v = run(&["--node"], &[]);
    assert!(
        v["size"].is_null(),
        "--node must not set UV_THREADPOOL_SIZE, got {:?}",
        v["size"]
    );
}

/// `NODE_COMPAT=1` is the tree-wide opt-out; same contract as `--node`.
#[test]
fn node_compat_env_leaves_pool_alone() {
    let v = run(&[], &[("NODE_COMPAT", "1")]);
    assert!(
        v["size"].is_null(),
        "NODE_COMPAT=1 must not set UV_THREADPOOL_SIZE, got {:?}",
        v["size"]
    );
}
