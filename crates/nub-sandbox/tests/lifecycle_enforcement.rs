//! Native children are the enforcement oracle; matcher success alone does not pass these tests.
use nub_sandbox::{CommandSpec, CompileCtx, Homes, Sandbox, ScopeCapabilities, compile};
use serde_json::json;
use std::collections::BTreeMap;
use std::path::PathBuf;

const CASE: &str = "SANDBOX_FIXTURE_CASE";

#[test]
fn native_child() {
    let Ok(case) = std::env::var(CASE) else {
        return;
    };
    let root = PathBuf::from(std::env::var("SANDBOX_FIXTURE_ROOT").unwrap());
    match case.as_str() {
        "files" => {
            assert_eq!(
                std::fs::read(root.join("readable/input")).unwrap(),
                b"readable"
            );
            assert!(std::fs::write(root.join("readable/input"), b"forbidden").is_err());
            assert!(std::fs::read(root.join("omitted/.ssh/key")).is_err());
            assert!(std::fs::write(root.join("omitted/.ssh/key"), b"forbidden").is_err());
            std::fs::write(root.join("project/created"), b"created").unwrap();
            std::fs::create_dir_all(root.join("project/later/nested")).unwrap();
            std::fs::write(root.join("project/later/nested/output"), b"later").unwrap();
            assert!(std::env::var_os("SANDBOX_PARENT_SECRET").is_none());
        }
        "tmp" => {
            let tmp = std::env::temp_dir();
            let marker = tmp.join("session-marker");
            if marker.exists() {
                assert_eq!(std::fs::read(&marker).unwrap(), b"shared");
                std::fs::write(root.join("project/reused"), b"yes").unwrap();
            } else {
                std::fs::write(&marker, b"shared").unwrap();
                std::fs::write(
                    root.join("project/tmp-path"),
                    tmp.to_string_lossy().as_bytes(),
                )
                .unwrap();
            }
        }
        "network" => {
            let address = std::fs::read_to_string(root.join("project/address"))
                .unwrap()
                .parse()
                .unwrap();
            assert!(
                std::net::TcpStream::connect_timeout(&address, std::time::Duration::from_secs(2))
                    .is_err(),
                "net:false allowed a direct TCP connection"
            );
            if let Ok(socket) = std::net::UdpSocket::bind("127.0.0.1:0") {
                assert!(
                    socket.send_to(b"not-a-secret", address).is_err(),
                    "net:false allowed a UDP datagram"
                );
            }
        }
        "proc" => {
            #[cfg(target_os = "linux")]
            {
                let parent = std::env::var("SANDBOX_FIXTURE_PARENT").unwrap();
                for file in ["environ", "mem"] {
                    assert!(
                        std::fs::File::open(format!("/proc/{parent}/{file}")).is_err(),
                        "parent {file} was exposed"
                    );
                }
            }
        }
        _ => panic!("unknown native fixture {case}"),
    }
    println!("SANDBOX_NATIVE_OK:{case}");
}

fn fixture() -> tempfile::TempDir {
    // Host tmp may be part of the platform runtime baseline. Put the omitted canary outside it.
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .unwrap();
    tempfile::Builder::new()
        .prefix(".sandbox-enforcement-")
        .tempdir_in(home)
        .unwrap()
}

fn sandbox(root: &std::path::Path, case: &str) -> Sandbox {
    let project = root.join("project");
    let ctx = CompileCtx::new(
        Homes {
            home: root.join("omitted"),
            cache: root.join("cache"),
            tmp: root.join("tmp"),
            project: project.clone(),
        },
        project,
        ScopeCapabilities::approved(),
        BTreeMap::new(),
    );
    let mut policy = compile(&json!({"fs": {(root.join("project").to_string_lossy()): "rw", (root.join("readable").to_string_lossy()): "r", "$tmp": "rw"}, "net": false, "env": false}), &ctx).unwrap();
    for key in [
        "PATH",
        "SystemRoot",
        "SYSTEMROOT",
        "WINDIR",
        "COMSPEC",
        "PATHEXT",
    ] {
        if let Ok(value) = std::env::var(key) {
            policy.env.constructed.insert(key.into(), value);
        }
    }
    policy.env.constructed.insert(CASE.into(), case.into());
    policy
        .env
        .constructed
        .insert("SANDBOX_FIXTURE_ROOT".into(), root.to_string_lossy().into());
    policy.env.constructed.insert(
        "SANDBOX_FIXTURE_PARENT".into(),
        std::process::id().to_string(),
    );
    Sandbox::new(&policy).unwrap()
}

fn run(sandbox: &Sandbox, root: &std::path::Path, case: &str) {
    let command = CommandSpec::new(std::env::current_exe().unwrap())
        .args(["--exact", "native_child", "--nocapture"])
        .cwd(root.join("project"));
    let prepared = sandbox.prepare(command).unwrap();
    assert!(
        prepared.degradation.lost.is_empty(),
        "native fixture degraded: {:?}",
        prepared.degradation
    );
    let output = prepared.output().unwrap();
    assert!(
        output.status.success(),
        "{case}: {}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains(&format!("SANDBOX_NATIVE_OK:{case}")));
}

#[test]
fn positive_filesystem_grants_are_enforced_by_native_children() {
    let root = fixture();
    for path in ["project", "readable", "omitted/.ssh", "cache"] {
        std::fs::create_dir_all(root.path().join(path)).unwrap();
    }
    std::fs::write(root.path().join("readable/input"), b"readable").unwrap();
    std::fs::write(root.path().join("omitted/.ssh/key"), b"not-a-secret").unwrap();
    let sandbox = sandbox(root.path(), "files");
    for _ in 0..3 {
        run(&sandbox, root.path(), "files");
    }
    assert_eq!(
        std::fs::read(root.path().join("project/created")).unwrap(),
        b"created"
    );
    assert_eq!(
        std::fs::read(root.path().join("omitted/.ssh/key")).unwrap(),
        b"not-a-secret"
    );
}

#[cfg(target_os = "linux")]
#[test]
fn native_child_cannot_read_the_host_process_environment() {
    let root = fixture();
    for path in ["project", "readable", "cache"] {
        std::fs::create_dir_all(root.path().join(path)).unwrap();
    }
    let sandbox = sandbox(root.path(), "proc");
    run(&sandbox, root.path(), "proc");
}

#[test]
fn managed_temp_is_shared_by_commands_until_session_close() {
    let root = fixture();
    for path in ["project", "readable", "cache"] {
        std::fs::create_dir_all(root.path().join(path)).unwrap();
    }
    let sandbox = sandbox(root.path(), "tmp");
    run(&sandbox, root.path(), "tmp");
    run(&sandbox, root.path(), "tmp");
    assert_eq!(
        std::fs::read(root.path().join("project/reused")).unwrap(),
        b"yes"
    );
    let tmp = PathBuf::from(std::fs::read_to_string(root.path().join("project/tmp-path")).unwrap());
    assert!(tmp.join("session-marker").exists());
    sandbox.close();
    #[cfg(unix)]
    assert!(
        !tmp.exists(),
        "closed Unix session retained its managed temp"
    );
}

#[test]
fn net_false_blocks_native_tcp_and_udp() {
    let root = fixture();
    for path in ["project", "readable", "cache"] {
        std::fs::create_dir_all(root.path().join(path)).unwrap();
    }
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let control = std::net::TcpStream::connect(address).unwrap();
    let _accepted = listener.accept().unwrap();
    drop(control);
    std::fs::write(root.path().join("project/address"), address.to_string()).unwrap();
    let sandbox = sandbox(root.path(), "network");
    run(&sandbox, root.path(), "network");
}
