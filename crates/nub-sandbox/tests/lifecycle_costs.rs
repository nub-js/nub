//! Serialized native measurements; each sample includes a real confined child.
use nub_sandbox::{
    CommandSpec, CompileCtx, Homes, Sandbox, SandboxPolicy, ScopeCapabilities, compile,
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

#[path = "common/tool_sandbox.rs"]
mod tool_sandbox;

#[test]
fn cost_child() {
    println!("SANDBOX_COST_CHILD_OK");
}

#[test]
fn cost_owner() {
    let Some(root) = std::env::var_os("SANDBOX_COST_OWNER_ROOT") else {
        return;
    };
    let root = Path::new(&root);
    let tree = std::env::var_os("SANDBOX_COST_OWNER_TREE");
    let start = Instant::now();
    let session = tool_sandbox::acquire(&policy(root, tree.as_deref().map(Path::new))).unwrap();
    run(&session, root);
    println!("SANDBOX_COST_OWNER_MS {}", elapsed(start));
    if std::env::var_os("SANDBOX_COST_OWNER_CRASH").is_some() {
        // Bypass resource destructors after successful native setup and execution.
        std::process::exit(91);
    }
    session.close();
}

fn independent_owner(root: &Path, tree: Option<&Path>, crash: bool) -> f64 {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args(["--exact", "cost_owner", "--nocapture"])
        .env("SANDBOX_COST_OWNER_ROOT", root)
        .env_remove("SANDBOX_COST_OWNER_TREE")
        .env_remove("SANDBOX_COST_OWNER_CRASH");
    if let Some(tree) = tree {
        command.env("SANDBOX_COST_OWNER_TREE", tree);
    }
    if crash {
        command.env("SANDBOX_COST_OWNER_CRASH", "1");
    }
    let output = command.output().unwrap();
    assert_eq!(
        output.status.code(),
        Some(if crash { 91 } else { 0 }),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .find_map(|line| line.strip_prefix("SANDBOX_COST_OWNER_MS "))
        .expect("owner emitted a timing after its confined command succeeded")
        .parse()
        .unwrap()
}

fn policy(root: &Path, tool_tree: Option<&Path>) -> SandboxPolicy {
    let mut environment = BTreeMap::new();
    for name in [
        "PATH",
        "SystemRoot",
        "SYSTEMROOT",
        "WINDIR",
        "COMSPEC",
        "PATHEXT",
    ] {
        if let Ok(value) = std::env::var(name) {
            environment.insert(name.into(), value);
        }
    }
    let context = CompileCtx::new(
        Homes {
            home: root.join("home"),
            cache: root.join("cache"),
            tmp: root.join("tmp"),
            project: root.into(),
        },
        root.into(),
        ScopeCapabilities::approved(),
        environment.clone(),
    );
    let mut input = json!({"fs": {"./": "rw", "$tmp": "rw"}, "net": false});
    if let Some(tree) = tool_tree {
        input["fs"][tree.to_str().unwrap()] = json!("rw");
    }
    let mut policy = compile(&input, &context).unwrap();
    policy.env.constructed = environment;
    policy
}

fn run(session: &Sandbox, root: &Path) {
    let prepared = session
        .prepare(
            CommandSpec::new(std::env::current_exe().unwrap())
                .args(["--exact", "cost_child", "--nocapture"])
                .cwd(root),
        )
        .unwrap();
    assert!(
        prepared.degradation.lost.is_empty(),
        "{:?}",
        prepared.degradation
    );
    let output = prepared.output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("SANDBOX_COST_CHILD_OK"));
}

fn elapsed(start: Instant) -> f64 {
    start.elapsed().as_secs_f64() * 1000.0
}

fn record(scenario: &str, phase: &str, sample: usize, ms: f64) {
    println!(
        "SANDBOX_COST {}",
        json!({
            "os": std::env::consts::OS, "arch": std::env::consts::ARCH,
            "scenario": scenario, "phase": phase, "sample": sample, "ms": ms,
            "native_adapter": cfg!(windows) && std::env::var_os("NUB_NATIVE_EMBEDDED_ADAPTER").is_some(),
        })
    );
}

fn tree_size(root: &Path) -> (u64, u64) {
    let mut pending = vec![root.to_path_buf()];
    let (mut files, mut bytes) = (0, 0);
    while let Some(path) = pending.pop() {
        let metadata = std::fs::symlink_metadata(&path).unwrap();
        if metadata.is_dir() {
            pending.extend(
                std::fs::read_dir(path)
                    .unwrap()
                    .map(|entry| entry.unwrap().path()),
            );
        } else if metadata.is_file() {
            files += 1;
            bytes += metadata.len();
        }
    }
    (files, bytes)
}

#[test]
#[ignore = "serialized native performance and persistent-cache cleanup probe"]
fn native_session_costs_and_unique_policy_churn() {
    let binary = std::env::current_exe().unwrap();
    let digest = Sha256::digest(std::fs::read(&binary).unwrap());
    let hash: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();
    println!(
        "SANDBOX_COST_BINARY {}",
        json!({"path": binary, "sha256": hash, "debug_assertions": cfg!(debug_assertions)})
    );
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .unwrap();
    let fixture = tempfile::Builder::new()
        .prefix(".sandbox-cost-")
        .tempdir_in(home)
        .unwrap();
    nub_sandbox::cleanup().unwrap();
    for sample in 0..16 {
        let start = Instant::now();
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "cost_child", "--nocapture"])
            .output()
            .unwrap();
        assert!(output.status.success());
        record("unconfined", "command", sample, elapsed(start));
    }
    let real_tree =
        std::env::var_os("SANDBOX_COST_TOOL_TREE").map(|path| std::fs::canonicalize(path).unwrap());
    let original_tree_size = real_tree.as_deref().map(tree_size);
    let mut scenarios = vec![("empty", 0, None), ("populated-1000", 1000, None)];
    if let Some(tree) = real_tree.as_deref() {
        let (files, bytes) = tree_size(tree);
        assert!(
            files >= 500,
            "the real tool fixture must contain a populated package tree"
        );
        println!(
            "SANDBOX_COST_TREE {}",
            json!({"path": tree, "files": files, "bytes": bytes})
        );
        scenarios.push(("real-tool-tree", 0, Some(tree)));
    }
    for (scenario, count, tool_tree) in scenarios {
        let root = fixture.path().join(scenario);
        std::fs::create_dir(&root).unwrap();
        for index in 0..count {
            std::fs::write(root.join(format!("file-{index}")), b"fixture").unwrap();
        }
        for sample in 0..12 {
            nub_sandbox::cleanup().unwrap();
            let total = Instant::now();
            let start = Instant::now();
            let policy = policy(&root, tool_tree);
            record(scenario, "resolve", sample, elapsed(start));
            let start = Instant::now();
            let session = tool_sandbox::acquire(&policy).unwrap();
            record(scenario, "acquire", sample, elapsed(start));
            let start = Instant::now();
            run(&session, &root);
            // Native Windows acquisition is lazy until a command supplies its
            // executable closure; include that cost in this first-command phase.
            record(scenario, "first-command", sample, elapsed(start));
            let start = Instant::now();
            session.close();
            record(scenario, "close", sample, elapsed(start));
            let start = Instant::now();
            nub_sandbox::cleanup().unwrap();
            record(scenario, "evict", sample, elapsed(start));
            record(scenario, "fresh-total", sample, elapsed(total));
        }
        let session = tool_sandbox::acquire(&policy(&root, tool_tree)).unwrap();
        run(&session, &root);
        for sample in 0..32 {
            let start = Instant::now();
            run(&session, &root);
            record(scenario, "reused-command", sample, elapsed(start));
        }
        for sample in 0..12 {
            let start = Instant::now();
            let native_ms = independent_owner(&root, tool_tree, false);
            record(scenario, "cross-process-acquire-command", sample, native_ms);
            record(scenario, "cross-process-total", sample, elapsed(start));
        }
        session.close();
        for sample in 0..12 {
            let start = Instant::now();
            let native_ms = independent_owner(&root, tool_tree, false);
            record(scenario, "idle-reopen-acquire-command", sample, native_ms);
            record(scenario, "idle-reopen-total", sample, elapsed(start));
        }
        nub_sandbox::cleanup().unwrap();
        #[cfg(windows)]
        for sample in 0..3 {
            independent_owner(&root, tool_tree, true);
            let start = Instant::now();
            nub_sandbox::cleanup().unwrap();
            record(scenario, "abandoned-owner-recovery", sample, elapsed(start));
        }
    }
    // Exceed the persistent idle-count limit with genuinely different grant roots.
    // These are caller-owned outputs: cleanup must never remove them.
    for sample in 0..72 {
        let root = fixture.path().join(format!("unique-{sample}"));
        std::fs::create_dir(&root).unwrap();
        std::fs::write(root.join("caller-output"), b"keep").unwrap();
        let start = Instant::now();
        let session = tool_sandbox::acquire(&policy(&root, None)).unwrap();
        run(&session, &root);
        session.close();
        record("unique-policy", "create-run-close", sample, elapsed(start));
    }
    nub_sandbox::cleanup().unwrap();
    assert_eq!(real_tree.as_deref().map(tree_size), original_tree_size);
    for sample in 0..72 {
        assert_eq!(
            std::fs::read(
                fixture
                    .path()
                    .join(format!("unique-{sample}/caller-output"))
            )
            .unwrap(),
            b"keep"
        );
    }
}
