//! Serialized native measurements; each sample includes a real confined child.
use nub_sandbox::{
    CommandSpec, CompileCtx, Homes, Sandbox, SandboxPolicy, ScopeCapabilities, compile,
};
use serde_json::json;
use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

#[test]
fn cost_child() {
    println!("SANDBOX_COST_CHILD_OK");
}

fn policy(root: &Path) -> SandboxPolicy {
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
    let mut policy = compile(
        &json!({"fs": {"./": "rw", "$tmp": "rw"}, "env": false, "net": false}),
        &context,
    )
    .unwrap();
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
        })
    );
}

#[test]
#[ignore = "serialized native performance and persistent-cache cleanup probe"]
fn native_session_costs_and_unique_policy_churn() {
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
    for (scenario, count) in [("empty", 0), ("populated-1000", 1000)] {
        let root = fixture.path().join(scenario);
        std::fs::create_dir(&root).unwrap();
        for index in 0..count {
            std::fs::write(root.join(format!("file-{index}")), b"fixture").unwrap();
        }
        for sample in 0..12 {
            nub_sandbox::cleanup().unwrap();
            let total = Instant::now();
            let start = Instant::now();
            let policy = policy(&root);
            record(scenario, "resolve", sample, elapsed(start));
            let start = Instant::now();
            let session = Sandbox::acquire(&policy).unwrap();
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
        let session = Sandbox::acquire(&policy(&root)).unwrap();
        run(&session, &root);
        for sample in 0..32 {
            let start = Instant::now();
            run(&session, &root);
            record(scenario, "reused-command", sample, elapsed(start));
        }
        session.close();
        nub_sandbox::cleanup().unwrap();
    }
    // Exceed the persistent idle-count limit with genuinely different grant roots.
    // These are caller-owned outputs: cleanup must never remove them.
    for sample in 0..72 {
        let root = fixture.path().join(format!("unique-{sample}"));
        std::fs::create_dir(&root).unwrap();
        std::fs::write(root.join("caller-output"), b"keep").unwrap();
        let start = Instant::now();
        let session = Sandbox::acquire(&policy(&root)).unwrap();
        run(&session, &root);
        session.close();
        record("unique-policy", "create-run-close", sample, elapsed(start));
    }
    nub_sandbox::cleanup().unwrap();
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
