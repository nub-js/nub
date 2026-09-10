//! Native compatibility controls through both the isolated probe and embedded adapter.

use super::*;
use crate::{CompileCtx, Homes, ScopeCapabilities, compile};
use serde_json::json;
use std::io::{Read, Write};
use std::path::Path;
use std::process::Command;

const CHILD: &str = "backend::windows_native_adapter_probe::native_adapter_child";

fn embedded_assets_protected() -> Option<bool> {
    use std::os::windows::ffi::OsStringExt as _;
    use windows_sys::Win32::System::LibraryLoader::{GetModuleFileNameW, GetModuleHandleW};
    for name in ["compat-x64.dll", "compat-arm64.dll"] {
        let name: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
        let module = unsafe { GetModuleHandleW(name.as_ptr()) };
        if module.is_null() {
            continue;
        }
        let mut path = vec![0u16; 32768];
        let length = unsafe { GetModuleFileNameW(module, path.as_mut_ptr(), path.len() as u32) };
        assert!(length > 0 && (length as usize) < path.len());
        let path =
            std::path::PathBuf::from(std::ffi::OsString::from_wide(&path[..length as usize]));
        let root = path.parent().unwrap();
        let registry = root.parent().unwrap().join("registry.json");
        let denied = |result: std::io::Result<std::fs::File>| {
            result.is_err_and(|e| e.kind() == std::io::ErrorKind::PermissionDenied)
        };
        let protected = std::fs::read(&path).is_ok()
            && denied(
                std::fs::OpenOptions::new()
                    .write(true)
                    .open(root.join("compat-x64.dll")),
            )
            && denied(
                std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(root.join("tamper")),
            )
            && denied(std::fs::File::open(registry));
        return Some(protected);
    }
    None
}

pub(crate) fn inject_probe(pid: u32) -> std::io::Result<()> {
    let adapter = std::env::var_os("NUB_NATIVE_ADAPTER_PROBE_DIR")
        .ok_or_else(|| std::io::Error::other("native adapter directory missing"))?;
    let output = Command::new(Path::new(&adapter).join("injector.exe"))
        .arg(pid.to_string())
        .arg(&adapter)
        .output()?;
    eprintln!("ADAPTER_INJECT {output:?}");
    if !output.status.success() {
        return Err(std::io::Error::other("native adapter injection failed"));
    }
    Ok(())
}

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
    let absolute_nul = Path::new(&file).parent().unwrap().join("NUL");
    let absolute_nul = std::fs::OpenOptions::new()
        .write(true)
        .open(absolute_nul)
        .and_then(|mut f| f.write(b"discarded"));
    let denied = std::fs::read(canary);
    let assets = embedded_assets_protected();
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
            "absolute_nul": absolute_nul.as_ref().is_ok_and(|n| *n == 9),
            "canonical": canonical.is_ok(),
            "canary_denied": denied.as_ref().is_err_and(|error| error.kind() == std::io::ErrorKind::PermissionDenied),
            "nested": nested,
            "assets_protected": assets,
        })
    );
    eprintln!("ADAPTER_ERRORS read={nul_read:?} write={nul_write:?} canonical={canonical:?}");
    if std::env::var_os("NUB_ADAPTER_PROBE_REQUIRE").is_some() {
        assert!(nul_read.is_ok_and(|n| n == 0));
        assert!(nul_write.is_ok_and(|n| n == 9));
        assert!(absolute_nul.is_ok_and(|n| n == 9));
        assert!(canonical.is_ok());
        assert!(denied.is_err_and(|error| error.kind() == std::io::ErrorKind::PermissionDenied));
        assert!(nested.is_none_or(|ok| ok));
        assert!(assets.is_none_or(|ok| ok));
    }
}

#[test]
#[ignore = "requires the separately built native API probe DLL and injector"]
fn native_adapter_primitives_with_raw_and_plain_controls() {
    native_adapter_primitives(true);
}

#[test]
fn embedded_native_adapter_primitives_with_raw_and_plain_controls() {
    native_adapter_primitives(false);
}

fn native_adapter_primitives(probe: bool) {
    use super::windows::WindowsStdio;
    let adapter = probe.then(|| std::env::var("NUB_NATIVE_ADAPTER_PROBE_DIR").unwrap());
    let binary = std::env::current_exe().unwrap();
    let root = tempfile::Builder::new()
        .prefix("sandbox-native-adapter-")
        .tempdir_in(std::env::var_os("USERPROFILE").unwrap())
        .unwrap();
    let project = root.path().join("project");
    std::fs::create_dir(&project).unwrap();
    let file = project.join("allowed");
    std::fs::write(&file, "allowed").unwrap();
    let canary = root.path().join("withheld-NUL");
    std::fs::write(&canary, "withheld").unwrap();
    let mut ambient: std::collections::BTreeMap<String, String> = std::env::vars().collect();
    ambient.insert(
        "NUB_ADAPTER_PROBE_FILE".into(),
        file.to_string_lossy().into_owned(),
    );
    ambient.insert(
        "NUB_ADAPTER_PROBE_CANARY".into(),
        canary.to_string_lossy().into_owned(),
    );
    let ctx = CompileCtx::new(
        Homes {
            home: root.path().join("home"),
            cache: root.path().join("cache"),
            tmp: root.path().join("tmp"),
            project: project.clone(),
        },
        project.clone(),
        ScopeCapabilities::approved(),
        ambient,
    );
    for mode in ["plain", "raw", if probe { "adapter" } else { "embedded" }] {
        let mut config = json!({
            "fs": {"./": "rw", "$tmp": "rw", binary.parent().unwrap().to_str().unwrap(): "r"},
            "vars": {"NUB_ADAPTER_PROBE_FILE": true, "NUB_ADAPTER_PROBE_CANARY": true},
            "net": false,
        });
        if let Some(adapter) = &adapter {
            config["fs"][adapter] = json!("r");
        }
        let mut policy = compile(&config, &ctx).unwrap();
        if matches!(mode, "adapter" | "embedded") {
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
            let sandbox = if mode == "embedded" {
                Sandbox::with_windows_native_compat(&policy)
            } else {
                Sandbox::acquire(&policy)
            }
            .unwrap();
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
            let assets = (mode == "embedded").then(|| {
                let second = Sandbox::with_windows_native_compat(&policy).unwrap();
                let mut pending = second
                    .prepare(CommandSpec::new(&binary).cwd(&project))
                    .unwrap();
                let plan = pending.launch.take().unwrap();
                let shared = pending.acquire_windows_resource(plan).unwrap();
                assert_eq!(resource.profile_name(), shared.profile_name());
                let path =
                    super::windows_native_compat::asset_path(resource.profile_name()).unwrap();
                assert!(path.join("compat-x64.dll").is_file());
                drop(shared);
                drop(pending);
                second.close();
                assert!(path.is_dir());
                path
            });
            let mut child = resource
                .spawn_before_resume(
                    WindowsStdio::Null,
                    WindowsStdio::Piped,
                    WindowsStdio::Piped,
                    |pid| {
                        if mode == "adapter" {
                            inject_probe(pid)?;
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
            let mut timed_out = false;
            let status = loop {
                if let Some(status) = child.try_wait().unwrap() {
                    break status;
                }
                if start.elapsed() > std::time::Duration::from_secs(30) {
                    child.kill().unwrap();
                    timed_out = true;
                    break child.wait().unwrap();
                }
                std::thread::sleep(std::time::Duration::from_millis(20));
            };
            drop(child);
            drop(resource);
            drop(prepared);
            sandbox.close();
            if let Some(path) = &assets {
                assert!(path.is_dir(), "idle asset retention");
            }
            cleanup().unwrap();
            if let Some(path) = &assets {
                assert!(!path.exists(), "explicit cleanup removes owned adapters");
            }
            let stdout = stdout.join().unwrap();
            let stderr = stderr.join().unwrap();
            assert!(
                !timed_out,
                "native adapter child timed out: {}",
                String::from_utf8_lossy(&stderr)
            );
            std::process::Output {
                status,
                stdout,
                stderr,
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
        if mode == "embedded" {
            assert_eq!(result["assets_protected"], true);
        }
        if mode == "raw" {
            assert_eq!(result["canonical"], false);
        }
        if mode != "raw" {
            for property in [
                "nul_read",
                "nul_write",
                "absolute_nul",
                "canonical",
                "nested",
            ] {
                assert_eq!(result[property], true, "{mode} {property}: {result}");
            }
        }
    }
}
