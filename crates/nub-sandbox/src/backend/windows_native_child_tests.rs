use super::*;
use std::io::{Read, Write};
use std::time::{Duration, Instant};

const FIXTURE: &str = "backend::windows::native_child_tests::windows_native_child_fixture";
const MODE: &str = "__NUB_WINDOWS_NATIVE_FIXTURE";

fn plan(root: &Path, mode: &str) -> AppContainerLaunch {
    let program = std::env::current_exe().unwrap();
    let mut env: BTreeMap<String, String> = std::env::vars()
        .filter(|(key, _)| {
            [
                "SYSTEMROOT",
                "WINDIR",
                "TEMP",
                "TMP",
                "LOCALAPPDATA",
                "USERPROFILE",
                "PATH",
            ]
            .contains(&key.to_ascii_uppercase().as_str())
        })
        .collect();
    env.insert(MODE.to_string(), mode.to_string());
    env.insert(
        "__NUB_WINDOWS_FIXTURE_ROOT".to_string(),
        root.display().to_string(),
    );
    AppContainerLaunch {
        program: program.clone().into_os_string(),
        args: crate::backend::CommandArgs::Argv(vec![
            "--exact".into(),
            FIXTURE.into(),
            "--nocapture".into(),
            "--test-threads=1".into(),
        ]),
        cwd: Some(root.to_path_buf()),
        read_grants: vec![program],
        read_node_grants: Vec::new(),
        write_grants: vec![root.to_path_buf()],
        publishable_grants: Vec::new(),
        env: Some(env),
        allow_internet: false,
        egress_funnel: None,
        private_tmp: false,
        stdout: WindowsStdio::Piped,
        stderr: WindowsStdio::Piped,
    }
}

fn wait_for_file(path: &Path) {
    let deadline = Instant::now() + Duration::from_secs(30);
    while !path.exists() {
        assert!(
            Instant::now() < deadline,
            "child did not create {}",
            path.display()
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn is_running(pid: u32) -> bool {
    use windows_sys::Win32::Foundation::{CloseHandle, WAIT_TIMEOUT};
    use windows_sys::Win32::System::Threading::{OpenProcess, WaitForSingleObject};
    let handle = unsafe { OpenProcess(0x0010_0000, 0, pid) };
    if handle.is_null() {
        return false;
    }
    let running = unsafe { WaitForSingleObject(handle, 0) } == WAIT_TIMEOUT;
    unsafe {
        CloseHandle(handle);
    }
    running
}

#[test]
fn windows_native_child_fixture() {
    let Ok(mode) = std::env::var(MODE) else {
        return;
    };
    let root = PathBuf::from(std::env::var_os("__NUB_WINDOWS_FIXTURE_ROOT").unwrap());
    match mode.as_str() {
        "echo" => {
            let mut input = String::new();
            std::io::stdin().read_to_string(&mut input).unwrap();
            println!("native-stdout:{input}");
            eprintln!("native-stderr");
            println!("{}", super::windows_token_report());
        }
        "tmp" => {
            let tmp = std::env::var("TMP").unwrap();
            assert_eq!(std::env::var("TEMP").unwrap(), tmp);
            assert_eq!(std::env::var("TMPDIR").unwrap(), tmp);
            let path = std::env::temp_dir().join("managed-marker");
            std::fs::write(&path, b"managed-slot").unwrap();
            println!("managed-temp:{}", path.display());
            println!("{}", super::windows_token_report());
        }
        "tree" => {
            let child = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", FIXTURE, "--nocapture", "--test-threads=1"])
                .env(MODE, "hold")
                .spawn()
                .unwrap();
            std::fs::write(root.join("descendant"), child.id().to_string()).unwrap();
            // Return without waiting: the native Job, not this direct child, owns the tree.
        }
        "hold" => {
            std::fs::write(root.join(format!("ready-{}", std::process::id())), b"ready").unwrap();
            loop {
                std::thread::sleep(Duration::from_secs(60));
            }
        }
        "owner" => {
            let mut launch = plan(&root, "hold");
            launch.private_tmp = true;
            let resource = launch.acquire().unwrap();
            let child = resource
                .spawn_with_stdio(WindowsStdio::Null, WindowsStdio::Null, WindowsStdio::Null)
                .unwrap();
            wait_for_file(&root.join(format!("ready-{}", child.id())));
            let record = root.join(format!("owner-{}", std::process::id()));
            let pending = record.with_extension("pending");
            std::fs::write(
                &pending,
                format!(
                    "{}\n{}\n{}",
                    resource.profile_name(),
                    child.id(),
                    resource.private_tmp().unwrap().display()
                ),
            )
            .unwrap();
            std::fs::rename(pending, record).unwrap();
            let mut input = String::new();
            std::io::stdin().read_to_string(&mut input).unwrap();
            drop(child);
        }
        _ => panic!("unexpected fixture mode"),
    }
}

#[test]
fn windows_native_streams_are_caller_owned_and_status_is_cached() {
    let root = tempfile::tempdir().unwrap();
    let resource = plan(root.path(), "echo").acquire().unwrap();
    let mut child = resource
        .spawn_with_stdio(
            WindowsStdio::Piped,
            WindowsStdio::Piped,
            WindowsStdio::Piped,
        )
        .unwrap();
    assert!(
        child.try_wait().unwrap().is_none(),
        "stdin keeps the native child live"
    );
    let mut input = child.take_stdin().unwrap();
    input.write_all(b"pipe-contract").unwrap();
    drop(input);
    let mut out = child.take_stdout().unwrap();
    let mut err = child.take_stderr().unwrap();
    let stdout = std::thread::spawn(move || {
        let mut bytes = String::new();
        out.read_to_string(&mut bytes).unwrap();
        bytes
    });
    let stderr = std::thread::spawn(move || {
        let mut bytes = String::new();
        err.read_to_string(&mut bytes).unwrap();
        bytes
    });
    drop(resource);
    let status = child.wait().unwrap();
    assert!(status.success());
    assert_eq!(child.try_wait().unwrap(), Some(status));
    let stdout = stdout.join().unwrap();
    assert!(stdout.contains("native-stdout:pipe-contract"));
    assert!(stdout.contains("is_appcontainer=true"));
    assert!(stderr.join().unwrap().contains("native-stderr"));
}

#[test]
fn windows_native_equivalent_resources_reuse_identity_but_not_jobs() {
    let root = tempfile::tempdir().unwrap();
    let first = plan(root.path(), "hold").acquire().unwrap();
    let second = plan(root.path(), "hold").acquire().unwrap();
    assert_eq!(first.profile_name(), second.profile_name());
    let profile = first.profile_name().to_string();
    let mut a = first
        .spawn_with_stdio(WindowsStdio::Null, WindowsStdio::Null, WindowsStdio::Null)
        .unwrap();
    let mut b = second
        .spawn_with_stdio(WindowsStdio::Null, WindowsStdio::Null, WindowsStdio::Null)
        .unwrap();
    wait_for_file(&root.path().join(format!("ready-{}", a.id())));
    wait_for_file(&root.path().join(format!("ready-{}", b.id())));
    a.kill().unwrap();
    assert!(!a.wait().unwrap().success());
    assert!(b.try_wait().unwrap().is_none());
    assert!(is_running(b.id()));
    let pid = b.id();
    drop(b);
    assert!(!is_running(pid), "Drop must reap its native Job");
    drop(a);
    drop(first);
    drop(second);
    assert_eq!(
        plan(root.path(), "hold").acquire().unwrap().profile_name(),
        profile
    );
}

#[test]
fn windows_native_drop_reaps_a_handed_off_descendant() {
    let root = tempfile::tempdir().unwrap();
    let resource = plan(root.path(), "tree").acquire().unwrap();
    let mut child = resource
        .spawn_with_stdio(WindowsStdio::Null, WindowsStdio::Null, WindowsStdio::Null)
        .unwrap();
    wait_for_file(&root.path().join("descendant"));
    let descendant: u32 = std::fs::read_to_string(root.path().join("descendant"))
        .unwrap()
        .parse()
        .unwrap();
    wait_for_file(&root.path().join(format!("ready-{descendant}")));
    assert!(
        child.try_wait().unwrap().is_none(),
        "a root exit is not Job completion"
    );
    drop(child);
    assert!(!is_running(descendant));
}

#[test]
fn windows_native_spawn_failure_does_not_poison_the_resource() {
    let root = tempfile::tempdir().unwrap();
    let good = plan(root.path(), "hold");
    let mut bad = good.clone();
    bad.program = root.path().join("not-an-executable.exe").into_os_string();
    let resource = bad.acquire().unwrap();
    assert!(resource.spawn().is_err());
    let resource = good.acquire().unwrap();
    let mut child = resource
        .spawn_with_stdio(WindowsStdio::Null, WindowsStdio::Null, WindowsStdio::Null)
        .unwrap();
    child.kill().unwrap();
    assert!(!child.wait().unwrap().success());
}

#[test]
fn windows_native_independent_owner_death_reaps_only_its_command() {
    let root = tempfile::tempdir().unwrap();
    let owner = || {
        std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", FIXTURE, "--nocapture", "--test-threads=1"])
            .env(MODE, "owner")
            .env("__NUB_WINDOWS_FIXTURE_ROOT", root.path())
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::inherit())
            .spawn()
            .unwrap()
    };
    let mut a = owner();
    let mut b = owner();
    let record = |owner: &std::process::Child| {
        let path = root.path().join(format!("owner-{}", owner.id()));
        wait_for_file(&path);
        let text = std::fs::read_to_string(path).unwrap();
        let mut lines = text.lines();
        (
            lines.next().unwrap().to_string(),
            lines.next().unwrap().parse::<u32>().unwrap(),
            PathBuf::from(lines.next().unwrap()),
        )
    };
    let (profile_a, child_a, tmp_a) = record(&a);
    let (profile_b, child_b, tmp_b) = record(&b);
    assert_eq!(profile_a, profile_b);
    assert_eq!(tmp_a, tmp_b);
    a.kill().unwrap();
    a.wait().unwrap();
    let deadline = Instant::now() + Duration::from_secs(30);
    while is_running(child_a) {
        assert!(
            Instant::now() < deadline,
            "owner death left its command running"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(
        is_running(child_b),
        "another owner's same-policy Job must remain alive"
    );
    drop(b.stdin.take());
    assert!(b.wait().unwrap().success());
    assert!(!is_running(child_b));
    let mut launch = plan(root.path(), "hold");
    launch.private_tmp = true;
    let resource = launch.acquire().unwrap();
    assert_eq!(resource.profile_name(), profile_a);
    assert_eq!(resource.private_tmp(), Some(tmp_a.as_path()));
}

#[test]
fn windows_native_managed_tmp_reuses_slot_without_retaining_command_environment() {
    let root = tempfile::tempdir().unwrap();
    let mut first = plan(root.path(), "tmp");
    first.private_tmp = true;
    let mut second = first.clone();
    for (launch, value) in [(&mut first, "caller-one"), (&mut second, "caller-two")] {
        launch
            .env
            .as_mut()
            .unwrap()
            .insert("tMp".into(), value.into());
        launch
            .env
            .as_mut()
            .unwrap()
            .insert("TEMP".into(), value.into());
    }
    let first = first.acquire().unwrap();
    let slot = first.private_tmp().unwrap().to_path_buf();
    let identity = first.identity().to_string();
    let keepalive = first.lease();
    drop(first);
    assert!(
        keepalive.is_live(),
        "session retains the native registry lease"
    );
    assert!(
        slot.is_dir(),
        "session lease protects a resource between commands"
    );
    let second = second.acquire().unwrap();
    assert_eq!(second.identity(), identity);
    assert_eq!(second.private_tmp(), Some(slot.as_path()));
    let mut child = second
        .spawn_with_stdio(
            WindowsStdio::Null,
            WindowsStdio::Piped,
            WindowsStdio::Inherit,
        )
        .unwrap();
    let mut stdout = String::new();
    child
        .take_stdout()
        .unwrap()
        .read_to_string(&mut stdout)
        .unwrap();
    assert!(child.wait().unwrap().success());
    assert!(stdout.contains("is_appcontainer=true"));
    assert_eq!(
        std::fs::read(slot.join("managed-marker")).unwrap(),
        b"managed-slot"
    );
    drop(child);
    drop(second);
    drop(keepalive);
}

#[test]
fn windows_native_explicit_grant_changes_never_share_managed_tmp() {
    let root = tempfile::tempdir().unwrap();
    let unique = tempfile::tempdir().unwrap();
    let mut first = plan(root.path(), "hold");
    first.private_tmp = true;
    let mut second = first.clone();
    second.read_grants.push(unique.path().to_path_buf());
    let first = first.acquire().unwrap();
    let second = second.acquire().unwrap();
    assert_ne!(first.identity(), second.identity());
    assert_ne!(first.private_tmp(), second.private_tmp());
}
