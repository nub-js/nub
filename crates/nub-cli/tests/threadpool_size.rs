//! libuv threadpool sizing — `UV_THREADPOOL_SIZE` set to `max(4, cores)` on an
//! augmented run.
//!
//! Node reads the variable once at startup, so the spawn is the only place it
//! can be set; nub sets it when the user has not, and leaves a user value alone.
//! Under `--node` / `NODE_COMPAT` the variable is absent, the plain-Node
//! fingerprint (libuv's own default of 4).

use std::path::{Path, PathBuf};
use std::process::Command;

fn nub_binary() -> PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // deps/
    path.pop(); // debug/ or release/
    path.push("nub");
    path
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
/// cores Node itself reports (a cgroup quota can make the two counts differ, so
/// the exact value is not pinned).
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

/// A value the user set is theirs, whatever the core count.
#[test]
fn user_value_is_never_overwritten() {
    let v = run(&[], &[("UV_THREADPOOL_SIZE", "3")]);
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
