use nub_sandbox::policy::SelfProcFile;
use nub_sandbox::{CompileCtx, Homes, SandboxPolicy, ScopeCapabilities, compile};
use serde_json::json;
use std::collections::BTreeMap;
use std::path::Path;

fn context(root: &Path) -> CompileCtx {
    CompileCtx::new(
        Homes {
            home: root.into(),
            cache: root.into(),
            tmp: root.into(),
            project: root.into(),
        },
        root.into(),
        ScopeCapabilities::approved(),
        BTreeMap::new(),
    )
}

#[test]
fn metadata_grants_are_explicit_resolved_capabilities() {
    let root = tempfile::tempdir().unwrap();
    let value = json!({"fs": {"/proc/self/maps": "r", "/proc/self/stat": "r"}, "net": false});
    let policy = compile(&value, &context(root.path())).unwrap();
    assert_eq!(
        policy.fs.self_proc,
        [SelfProcFile::Maps, SelfProcFile::Stat].into()
    );
    assert!(
        policy.fs.rules.entries.is_empty(),
        "no compiler-process PID in ordinary grants"
    );
    let encoded = serde_json::to_value(&policy).unwrap();
    let decoded: SandboxPolicy = serde_json::from_value(encoded).unwrap();
    assert_eq!(decoded.fs.self_proc, policy.fs.self_proc);
    let plain = compile(&json!({"fs": false}), &context(root.path())).unwrap();
    assert!(plain.fs.self_proc.is_empty());
    assert!(
        serde_json::to_value(&plain).unwrap()["fs"]
            .get("self_proc")
            .is_none()
    );
}

#[test]
fn writable_metadata_is_rejected_including_reused_grants() {
    let root = tempfile::tempdir().unwrap();
    for value in [
        json!({"fs": {"/proc/self/maps": "rw"}}),
        json!({"fs": {"/proc/self/stat": true}}),
        json!({"fs": ["/proc/self/maps"]}),
        json!({"shared": {"/proc/self/stat": "rw"}, "fs": {"...:#/shared": true}}),
    ] {
        assert!(compile(&value, &context(root.path())).is_err(), "{value}");
    }
    let document =
        json!({"shared": {"/proc/self/stat": "r"}, "sandbox": {"fs": {"...:#/shared": true}}});
    let policy = compile(
        &document["sandbox"],
        &context(root.path()).with_document(document.clone()),
    )
    .unwrap();
    assert_eq!(policy.fs.self_proc, [SelfProcFile::Stat].into());
}

#[cfg(not(target_os = "linux"))]
#[test]
fn unsupported_hosts_refuse_metadata_at_acquisition() {
    let root = tempfile::tempdir().unwrap();
    let policy = compile(
        &json!({"fs": {"/proc/self/stat": "r"}}),
        &context(root.path()),
    )
    .unwrap();
    let error = nub_sandbox::Sandbox::acquire(&policy)
        .err()
        .expect("must refuse");
    assert_eq!(error.lost, ["fs-self-proc"]);
}

#[cfg(target_os = "linux")]
mod linux {
    use super::*;
    use nub_sandbox::{CommandSpec, Sandbox};
    use std::ffi::CString;
    use std::io::Read;
    use std::os::fd::{AsRawFd, FromRawFd};
    use std::os::unix::ffi::OsStrExt;
    use std::process::Command;
    use std::time::{Duration, Instant};

    fn fixture(files: &[&str]) -> (tempfile::TempDir, Sandbox) {
        let root = tempfile::tempdir().unwrap();
        let project = root.path().join("project");
        std::fs::create_dir(&project).unwrap();
        std::fs::write(root.path().join("secret"), "WITHHELD").unwrap();
        std::fs::write(project.join("allowed"), "ALLOWED").unwrap();
        std::fs::write(
            project.join(std::ffi::OsStr::from_bytes(b"nonutf8-\xff")),
            "BYTES",
        )
        .unwrap();
        let exe = std::env::current_exe().unwrap();
        let mut fs = json!({"./": "rw", "$tmp": "rw"});
        fs[exe.parent().unwrap().to_string_lossy().as_ref()] = json!("r");
        for file in files {
            fs[format!("/proc/self/{file}")] = json!("r");
        }
        let mut policy = compile(&json!({"fs": fs, "net": false}), &context(&project)).unwrap();
        policy.env.constructed.extend([
            ("SELF_PROC_CASE".into(), files.join(",")),
            ("SELF_PROC_ROOT".into(), root.path().display().to_string()),
            ("SELF_PROC_EXE".into(), exe.display().to_string()),
            ("SELF_PROC_OWNER".into(), std::process::id().to_string()),
            ("SELF_PROC_LOOP".into(), "1".into()),
        ]);
        (root, Sandbox::acquire(&policy).unwrap())
    }

    fn spec(root: &Path, name: &str) -> CommandSpec {
        CommandSpec::new(std::env::current_exe().unwrap())
            .args(["--exact", name, "--nocapture"])
            .cwd(root.join("project"))
    }

    fn check_metadata() {
        let selected = std::env::var("SELF_PROC_CASE").unwrap();
        for name in ["maps", "stat"] {
            let read = std::fs::read_to_string(format!("/proc/self/{name}"));
            if selected.split(',').any(|file| file == name) {
                let text = read.unwrap();
                if name == "stat" {
                    assert_eq!(
                        text.split_whitespace().next().unwrap(),
                        std::process::id().to_string()
                    );
                } else {
                    assert!(
                        text.contains("self_proc"),
                        "maps describes this executable: {text}"
                    );
                }
            } else {
                assert_eq!(
                    read.unwrap_err().kind(),
                    std::io::ErrorKind::PermissionDenied
                );
            }
        }
    }

    #[test]
    fn metadata_child() {
        if std::env::var_os("SELF_PROC_CASE").is_none() {
            return;
        }
        check_metadata();
        std::thread::spawn(check_metadata).join().unwrap();
        let root = std::path::PathBuf::from(std::env::var_os("SELF_PROC_ROOT").unwrap());
        let owner = std::env::var("SELF_PROC_OWNER").unwrap();
        for path in [
            root.join("secret"),
            "/proc/self/environ".into(),
            "/proc/self/cmdline".into(),
            "/proc/thread-self/stat".into(),
            format!("/proc/{owner}/maps").into(),
            format!("/proc/{owner}/environ").into(),
            format!("/proc/{}/stat", std::process::id()).into(),
        ] {
            assert_eq!(
                std::fs::read(&path).unwrap_err().kind(),
                std::io::ErrorKind::PermissionDenied,
                "{}",
                path.display()
            );
        }
        assert!(std::env::var_os("WITHHELD_AMBIENT_SECRET").is_none());
        assert_eq!(std::fs::read_to_string("allowed").unwrap(), "ALLOWED");
        assert_eq!(
            std::fs::read(std::ffi::OsStr::from_bytes(b"nonutf8-\xff")).unwrap(),
            b"BYTES"
        );
        assert!(
            std::fs::OpenOptions::new()
                .write(true)
                .open("/proc/self/stat")
                .is_err()
        );
        if std::env::var_os("SELF_PROC_GRANDCHILD").is_none() {
            let output = Command::new(std::env::var_os("SELF_PROC_EXE").unwrap())
                .args(["--exact", "linux::metadata_child", "--nocapture"])
                .env("SELF_PROC_GRANDCHILD", "1")
                .output()
                .unwrap();
            assert!(output.status.success(), "grandchild: {output:?}");
        }
    }

    #[test]
    fn retained_metadata_is_per_command_thread_and_descendant() {
        for files in [&[][..], &["maps"][..], &["stat"][..], &["maps", "stat"][..]] {
            let (root, sandbox) = fixture(files);
            for _ in 0..3 {
                let output = sandbox
                    .prepare(spec(root.path(), "linux::metadata_child"))
                    .unwrap()
                    .output()
                    .unwrap();
                assert!(output.status.success(), "{files:?}: {output:?}");
            }
            sandbox.close();
        }
    }

    #[test]
    fn syscall_child() {
        if std::env::var_os("SELF_PROC_CASE").is_none() {
            return;
        }
        let path = CString::new("/proc/self/stat").unwrap();
        for (close_exec, nonblock) in [(0, 0), (libc::O_CLOEXEC, libc::O_NONBLOCK)] {
            let flags = close_exec | nonblock;
            let how = [flags as u64, 0, 0];
            let mut fds = vec![
                unsafe { libc::syscall(libc::SYS_openat, libc::AT_FDCWD, path.as_ptr(), flags, 0) },
                unsafe {
                    libc::syscall(
                        libc::SYS_openat2,
                        libc::AT_FDCWD,
                        path.as_ptr(),
                        how.as_ptr(),
                        24,
                    )
                },
            ];
            #[cfg(target_arch = "x86_64")]
            fds.push(unsafe { libc::syscall(libc::SYS_open, path.as_ptr(), flags, 0) });
            for fd in fds {
                assert!(fd >= 0, "open: {}", std::io::Error::last_os_error());
                let mut file = unsafe { std::fs::File::from_raw_fd(fd as i32) };
                let status = unsafe { libc::fcntl(file.as_raw_fd(), libc::F_GETFL) };
                assert_eq!(status & libc::O_NONBLOCK, nonblock);
                let descriptor = unsafe { libc::fcntl(file.as_raw_fd(), libc::F_GETFD) };
                assert_eq!(descriptor & libc::FD_CLOEXEC, i32::from(close_exec != 0));
                let mut text = String::new();
                file.read_to_string(&mut text).unwrap();
                assert_eq!(
                    text.split_whitespace().next().unwrap(),
                    std::process::id().to_string()
                );
            }
        }
        for flags in [libc::O_PATH, libc::O_DIRECTORY, libc::O_RDWR, libc::O_CREAT] {
            let fd = unsafe { libc::open(path.as_ptr(), flags, 0o600) };
            assert_eq!(fd, -1, "unsupported flags: {flags}");
        }
        let fd = unsafe {
            libc::syscall(
                libc::SYS_openat,
                libc::AT_FDCWD,
                std::ptr::null::<u8>(),
                0,
                0,
            )
        };
        assert_eq!(fd, -1);
        assert_eq!(
            std::io::Error::last_os_error().raw_os_error(),
            Some(libc::EFAULT)
        );
    }

    #[test]
    fn read_only_syscalls_preserve_descriptor_flags() {
        let (root, sandbox) = fixture(&["stat"]);
        let output = sandbox
            .prepare(spec(root.path(), "linux::syscall_child"))
            .unwrap()
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
    }

    #[test]
    fn reading_child() {
        if std::env::var_os("SELF_PROC_CASE").is_none() {
            return;
        }
        std::fs::write("ready", "ready").unwrap();
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(30) {
            check_metadata();
        }
    }

    #[test]
    fn cancellation_reclaims_metadata_workers_and_descriptors() {
        let (root, sandbox) = fixture(&["maps", "stat"]);
        let counts = || {
            (
                std::fs::read_dir("/proc/self/fd").unwrap().count(),
                std::fs::read_dir("/proc/self/task").unwrap().count(),
            )
        };
        // Warm the process-global owner guardian before establishing the baseline.
        assert!(
            sandbox
                .prepare(CommandSpec::new("/bin/true"))
                .unwrap()
                .status()
                .unwrap()
                .success()
        );
        let baseline = counts();
        for _ in 0..20 {
            let ready = root.path().join("project/ready");
            let _ = std::fs::remove_file(&ready);
            let child = sandbox
                .prepare(spec(root.path(), "linux::reading_child"))
                .unwrap()
                .spawn()
                .unwrap();
            let start = Instant::now();
            while !ready.exists() {
                assert!(
                    start.elapsed() < Duration::from_secs(10),
                    "reader never became ready"
                );
                std::thread::sleep(Duration::from_millis(10));
            }
            drop(child);
        }
        assert_eq!(
            counts(),
            baseline,
            "all per-command resources return to baseline"
        );
    }

    #[test]
    fn read_loop_child() {
        if std::env::var_os("SELF_PROC_LOOP").is_none() {
            return;
        }
        let start = Instant::now();
        for _ in 0..2000 {
            assert_eq!(std::fs::read("allowed").unwrap(), b"ALLOWED");
        }
        println!("OPEN_LOOP_MS {}", start.elapsed().as_secs_f64() * 1000.0);
    }

    #[test]
    #[ignore = "serialized release measurement with plain and unchanged-policy controls"]
    fn metadata_costs() {
        use sha2::{Digest, Sha256};
        assert!(!cfg!(debug_assertions), "measure a release binary");
        let exe = std::env::current_exe().unwrap();
        let hash: String = Sha256::digest(std::fs::read(&exe).unwrap())
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        println!("SELF_PROC_BINARY {}", json!({"path": exe, "sha256": hash}));
        for files in [&[][..], &["maps", "stat"][..]] {
            let (root, sandbox) = fixture(files);
            for sample in 0..12 {
                for confined in [false, true] {
                    let start = Instant::now();
                    let output = if confined {
                        sandbox
                            .prepare(spec(root.path(), "linux::read_loop_child"))
                            .unwrap()
                            .output()
                            .unwrap()
                    } else {
                        Command::new(&exe)
                            .args(["--exact", "linux::read_loop_child", "--nocapture"])
                            .env("SELF_PROC_LOOP", "1")
                            .current_dir(root.path().join("project"))
                            .output()
                            .unwrap()
                    };
                    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
                    assert!(output.status.success(), "{output:?}");
                    let text = String::from_utf8(output.stdout).unwrap();
                    let inner: f64 = text
                        .lines()
                        .find_map(|line| line.strip_prefix("OPEN_LOOP_MS "))
                        .unwrap()
                        .parse()
                        .unwrap();
                    println!(
                        "SELF_PROC_COST {}",
                        json!({"files": files, "confined": confined, "sample": sample, "total_ms": elapsed, "opens": 2000, "open_ms": inner})
                    );
                }
            }
        }
    }
}
