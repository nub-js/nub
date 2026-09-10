//! Opt-in native API experiment; no adapter is installed by production launches.

use super::*;
use crate::{CompileCtx, Homes, ScopeCapabilities, compile};
use serde_json::json;
use std::io::{Read, Write};
use std::path::Path;
use std::process::Command;

const CHILD: &str = "backend::windows_native_adapter_probe::native_adapter_child";

#[test]
fn native_adapter_child() {
    let Ok(file) = std::env::var("NUB_ADAPTER_PROBE_FILE") else {
        return;
    };
    let canary = std::env::var("NUB_ADAPTER_PROBE_CANARY").unwrap();
    let nul_read = std::fs::File::open("NUL").and_then(|mut f| f.read(&mut [0u8]));
    let nul_write = std::fs::OpenOptions::new()
        .write(true)
        .open("NUL")
        .and_then(|mut f| f.write(b"discarded"));
    let canonical = std::fs::canonicalize(&file);
    let denied = std::fs::read(canary);
    let mut nested = None;
    if std::env::var_os("NUB_ADAPTER_PROBE_NESTED").is_none() {
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", CHILD, "--nocapture"])
            .env("NUB_ADAPTER_PROBE_NESTED", "1")
            .output();
        eprintln!("ADAPTER_NESTED {output:?}");
        nested = Some(output.is_ok_and(|output| {
            output.status.success()
                && String::from_utf8_lossy(&output.stdout).contains("ADAPTER_PRIMITIVES")
        }));
    }
    println!(
        "ADAPTER_PRIMITIVES {}",
        json!({
            "nul_read": nul_read.as_ref().is_ok_and(|n| *n == 0),
            "nul_write": nul_write.as_ref().is_ok_and(|n| *n == 9),
            "canonical": canonical.is_ok(),
            "canary_denied": denied.as_ref().is_err_and(|error| error.kind() == std::io::ErrorKind::PermissionDenied),
            "nested": nested,
        })
    );
    eprintln!("ADAPTER_ERRORS read={nul_read:?} write={nul_write:?} canonical={canonical:?}");
    if std::env::var_os("NUB_ADAPTER_PROBE_REQUIRE").is_some() {
        assert!(nul_read.is_ok_and(|n| n == 0));
        assert!(nul_write.is_ok_and(|n| n == 9));
        assert!(canonical.is_ok());
        assert!(denied.is_err_and(|error| error.kind() == std::io::ErrorKind::PermissionDenied));
        assert!(nested.is_none_or(|ok| ok));
    }
}

#[test]
#[ignore = "requires the separately built native API probe DLL and injector"]
fn native_adapter_primitives_with_raw_and_plain_controls() {
    use super::windows::WindowsStdio;
    let adapter = std::env::var("NUB_NATIVE_ADAPTER_PROBE_DIR").unwrap();
    let binary = std::env::current_exe().unwrap();
    let root = tempfile::Builder::new()
        .prefix("sandbox-native-adapter-")
        .tempdir_in(std::env::var_os("USERPROFILE").unwrap())
        .unwrap();
    let project = root.path().join("project");
    std::fs::create_dir(&project).unwrap();
    let file = project.join("allowed");
    std::fs::write(&file, "allowed").unwrap();
    let canary = root.path().join("withheld");
    std::fs::write(&canary, "withheld").unwrap();
    let ctx = CompileCtx::new(
        Homes {
            home: root.path().join("home"),
            cache: root.path().join("cache"),
            tmp: root.path().join("tmp"),
            project: project.clone(),
        },
        project.clone(),
        ScopeCapabilities::approved(),
        std::env::vars().collect(),
    );
    for mode in ["plain", "raw", "adapter"] {
        let mut policy = compile(&json!({
            "fs": {"./": "rw", "$tmp": "rw", binary.parent().unwrap().to_str().unwrap(): "r", adapter.as_str(): "r"},
            "vars": {"NUB_ADAPTER_PROBE_FILE": file.to_str().unwrap(), "NUB_ADAPTER_PROBE_CANARY": canary.to_str().unwrap()},
            "net": false,
        }), &ctx).unwrap();
        if mode == "adapter" {
            policy
                .env
                .constructed
                .insert("NUB_ADAPTER_PROBE_REQUIRE".into(), "1".into());
        }
        let output = if mode == "plain" {
            Command::new(&binary)
                .args(["--exact", CHILD, "--nocapture"])
                .env_clear()
                .envs(&policy.env.constructed)
                .current_dir(&project)
                .output()
                .unwrap()
        } else {
            let sandbox = Sandbox::acquire(&policy).unwrap();
            let mut prepared = sandbox
                .prepare(
                    CommandSpec::new(&binary)
                        .args(["--exact", CHILD, "--nocapture"])
                        .cwd(&project),
                )
                .unwrap();
            assert!(prepared.degradation.lost.is_empty());
            let launch = prepared.launch.take().unwrap();
            let resource = prepared.acquire_windows_resource(launch).unwrap();
            let mut child = resource
                .spawn_before_resume(
                    WindowsStdio::Null,
                    WindowsStdio::Piped,
                    WindowsStdio::Piped,
                    |pid| {
                        if mode == "adapter" {
                            let output = Command::new(Path::new(&adapter).join("injector.exe"))
                                .arg(pid.to_string())
                                .arg(Path::new(&adapter).join("probe.dll"))
                                .output()?;
                            eprintln!("ADAPTER_INJECT {output:?}");
                            if !output.status.success() {
                                return Err(std::io::Error::other(
                                    "native adapter injection failed",
                                ));
                            }
                        }
                        Ok(())
                    },
                )
                .unwrap();
            let stdout = child.take_stdout().unwrap();
            let stderr = child.take_stderr().unwrap();
            let read = |mut stream: Box<dyn Read + Send>| {
                let mut bytes = Vec::new();
                stream.read_to_end(&mut bytes).unwrap();
                bytes
            };
            let stdout = std::thread::spawn(move || read(Box::new(stdout)));
            let stderr = std::thread::spawn(move || read(Box::new(stderr)));
            let start = std::time::Instant::now();
            let status = loop {
                if let Some(status) = child.try_wait().unwrap() {
                    break status;
                }
                if start.elapsed() > std::time::Duration::from_secs(30) {
                    child.kill().unwrap();
                    child.wait().unwrap();
                    panic!("native adapter child timed out");
                }
                std::thread::sleep(std::time::Duration::from_millis(20));
            };
            drop(child);
            drop(resource);
            drop(prepared);
            sandbox.close();
            cleanup().unwrap();
            std::process::Output {
                status,
                stdout: stdout.join().unwrap(),
                stderr: stderr.join().unwrap(),
            }
        };
        eprintln!("ADAPTER_CONTROL {mode} {output:?}");
        assert!(output.status.success(), "{mode}: {output:?}");
        let text = String::from_utf8_lossy(&output.stdout);
        let marker = text
            .split("ADAPTER_PRIMITIVES ")
            .nth(1)
            .unwrap()
            .lines()
            .next()
            .unwrap();
        let result: serde_json::Value = serde_json::from_str(marker).unwrap();
        assert_eq!(result["canary_denied"], mode != "plain");
        if mode == "raw" {
            assert_eq!(result["canonical"], false);
        }
        if mode == "plain" || mode == "adapter" {
            for property in ["nul_read", "nul_write", "canonical", "nested"] {
                assert_eq!(result[property], true, "{mode} {property}: {result}");
            }
        }
    }
}
