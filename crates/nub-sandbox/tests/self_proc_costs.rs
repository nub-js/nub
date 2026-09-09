//! Release controls using only the policy API shared with the pre-metadata engine.
#![cfg(target_os = "linux")]

use nub_sandbox::{CommandSpec, CompileCtx, Homes, Sandbox, ScopeCapabilities, compile};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::process::Command;
use std::time::Instant;

#[test]
fn cost_child() {
    let Some(loops) = std::env::var("SELF_PROC_COST_LOOPS").ok() else {
        return;
    };
    if std::env::var("SELF_PROC_EXPECT").as_deref() == Ok("true") {
        let stat = std::fs::read_to_string("/proc/self/stat").unwrap();
        assert_eq!(
            stat.split_whitespace().next().unwrap(),
            std::process::id().to_string()
        );
        assert!(
            std::fs::read_to_string("/proc/self/maps")
                .unwrap()
                .contains("self_proc_costs")
        );
    }
    let start = Instant::now();
    let loops: usize = loops.parse().unwrap();
    for _ in 0..loops {
        assert_eq!(std::fs::read("allowed").unwrap(), b"ALLOWED");
    }
    println!("OPEN_LOOP_MS {}", start.elapsed().as_secs_f64() * 1000.0);
}

#[test]
#[ignore = "serialized release performance controls"]
fn metadata_costs() {
    if cfg!(debug_assertions) {
        panic!("measure a release binary");
    }
    let exe = std::env::current_exe().unwrap();
    let hash: String = Sha256::digest(std::fs::read(&exe).unwrap())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    println!("SELF_PROC_BINARY {}", json!({"path": exe, "sha256": hash}));
    for metadata in [false, true] {
        if metadata && std::env::var_os("SELF_PROC_BASELINE").is_some() {
            continue;
        }
        for loops in [0, 2000] {
            let root = tempfile::tempdir().unwrap();
            std::fs::write(root.path().join("allowed"), "ALLOWED").unwrap();
            let ctx = CompileCtx::new(
                Homes {
                    home: root.path().into(),
                    cache: root.path().into(),
                    tmp: root.path().into(),
                    project: root.path().into(),
                },
                root.path().into(),
                ScopeCapabilities::approved(),
                BTreeMap::new(),
            );
            let mut value = json!({"fs":{"./":"rw", "$tmp":"rw"}, "net": false});
            value["fs"][exe.parent().unwrap().to_string_lossy().as_ref()] = json!("r");
            if metadata {
                value["fs"]["/proc/self/maps"] = json!("r");
                value["fs"]["/proc/self/stat"] = json!("r");
            }
            let mut policy = compile(&value, &ctx).unwrap();
            policy.env.constructed.extend([
                ("SELF_PROC_COST_LOOPS".into(), loops.to_string()),
                ("SELF_PROC_EXPECT".into(), metadata.to_string()),
            ]);
            let acquire = Instant::now();
            let sandbox = Sandbox::acquire(&policy).unwrap();
            println!(
                "SELF_PROC_ACQUIRE {}",
                json!({"metadata":metadata,"ms":acquire.elapsed().as_secs_f64()*1000.0})
            );
            for sample in 0..24 {
                // Alternate order rather than putting every plain control before its subject.
                for confined in if sample % 2 == 0 {
                    [false, true]
                } else {
                    [true, false]
                } {
                    let start = Instant::now();
                    let output = if confined {
                        let prepared = sandbox
                            .prepare(
                                CommandSpec::new(&exe)
                                    .args(["--exact", "cost_child", "--nocapture"])
                                    .cwd(root.path()),
                            )
                            .unwrap();
                        assert!(prepared.degradation.lost.is_empty());
                        prepared.output().unwrap()
                    } else {
                        Command::new(&exe)
                            .args(["--exact", "cost_child", "--nocapture"])
                            .env("SELF_PROC_COST_LOOPS", loops.to_string())
                            .env("SELF_PROC_EXPECT", metadata.to_string())
                            .current_dir(root.path())
                            .output()
                            .unwrap()
                    };
                    let total_ms = start.elapsed().as_secs_f64() * 1000.0;
                    assert!(output.status.success(), "{output:?}");
                    let text = String::from_utf8(output.stdout).unwrap();
                    let open_ms: f64 = text
                        .lines()
                        .find_map(|line| line.strip_prefix("OPEN_LOOP_MS "))
                        .unwrap()
                        .parse()
                        .unwrap();
                    println!(
                        "SELF_PROC_COST {}",
                        json!({"metadata":metadata,"confined":confined,"sample":sample,"total_ms":total_ms,"opens":loops,"open_ms":open_ms})
                    );
                }
            }
            let close = Instant::now();
            sandbox.close();
            nub_sandbox::cleanup().unwrap();
            println!(
                "SELF_PROC_CLOSE {}",
                json!({"metadata":metadata,"ms":close.elapsed().as_secs_f64()*1000.0})
            );
        }
    }
}
