//! Native Windows lifecycle regressions.  These deliberately drive a second test
//! process: a process-local cache cannot prove the persistent registry's lease and
//! cleanup rules.

use super::*;
use crate::backend::CommandArgs;
use std::collections::BTreeMap;
use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const FIXTURE: &str = "backend::windows::windows_cleanup_tests::windows_cleanup_fixture";
const MODE: &str = "__NUB_WINDOWS_CLEANUP_FIXTURE";
const ROOT: &str = "__NUB_WINDOWS_CLEANUP_ROOT";

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
                "PATH",
            ]
            .contains(&key.to_ascii_uppercase().as_str())
        })
        .collect();
    env.insert(MODE.into(), mode.into());
    env.insert(ROOT.into(), root.display().to_string());
    AppContainerLaunch {
        program: program.clone().into_os_string(),
        args: CommandArgs::Argv(vec![
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
        private_tmp: true,
        stdout: WindowsStdio::Null,
        stderr: WindowsStdio::Null,
    }
}

fn wait_for(path: &Path) {
    let deadline = Instant::now() + Duration::from_secs(30);
    while !path.exists() {
        assert!(
            Instant::now() < deadline,
            "timed out waiting for {}",
            path.display()
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn running(pid: u32) -> bool {
    use windows_sys::Win32::Foundation::{CloseHandle, WAIT_TIMEOUT};
    use windows_sys::Win32::System::Threading::{OpenProcess, WaitForSingleObject};
    let handle = unsafe { OpenProcess(0x0010_0000, 0, pid) };
    if handle.is_null() {
        return false;
    }
    let live = unsafe { WaitForSingleObject(handle, 0) } == WAIT_TIMEOUT;
    unsafe { CloseHandle(handle) };
    live
}

struct OwnerFixture(std::process::Child);

impl Drop for OwnerFixture {
    fn drop(&mut self) {
        self.0.stdin.take();
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// Keeps an unrelated SID ACE installed while cleanup removes the resource's ACE.
/// The synthetic profile need not be registered: Windows derives its SID from its
/// deterministic name, which makes this a genuinely distinct DACL entry.
struct ForeignProfileAce {
    profile: String,
    path: PathBuf,
}

impl ForeignProfileAce {
    fn grant(path: &Path) -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let profile = format!("nub-test-foreign-{}-{stamp}", std::process::id());
        super::launch::test_set_profile_ace(&profile, path, true).unwrap();
        let owned = Self {
            profile,
            path: path.to_path_buf(),
        };
        assert!(
            super::launch::test_profile_has_ace(&owned.profile, path).unwrap(),
            "test setup did not install unrelated SID ACE"
        );
        owned
    }
}

impl Drop for ForeignProfileAce {
    fn drop(&mut self) {
        let _ = super::launch::test_set_profile_ace(&self.profile, &self.path, false);
    }
}

struct CleanupAfterTest;

impl Drop for CleanupAfterTest {
    fn drop(&mut self) {
        let _ = cleanup_resources();
    }
}

#[test]
fn windows_cleanup_fixture() {
    let Ok(mode) = std::env::var(MODE) else {
        return;
    };
    let root = PathBuf::from(std::env::var_os(ROOT).unwrap());
    match mode.as_str() {
        "hold" => {
            std::fs::write(root.join(format!("ready-{}", std::process::id())), b"ready").unwrap();
            // Do not share the fixture's inherited stdin with the confined child:
            // the parent fixture owns that pipe as its deterministic lifetime gate.
            loop {
                std::thread::sleep(Duration::from_secs(60));
            }
        }
        "owner" => {
            let resource = plan(&root, "hold").acquire().unwrap();
            let child = resource
                .spawn_with_stdio(WindowsStdio::Null, WindowsStdio::Null, WindowsStdio::Null)
                .unwrap();
            let pid = child.id();
            wait_for(&root.join(format!("ready-{pid}")));
            std::fs::write(
                root.join(format!("owner-{}", std::process::id())),
                format!("{}\n{pid}", resource.profile_name()),
            )
            .unwrap();
            let mut input = String::new();
            std::io::stdin().read_to_string(&mut input).unwrap();
            drop(child);
        }
        other => panic!("unknown cleanup fixture mode {other}"),
    }
}

#[test]
fn windows_cleanup_skips_a_live_independent_lease_and_preserves_caller_files() {
    let root = tempfile::tempdir().unwrap();
    let marker = root.path().join("caller-owned.txt");
    std::fs::write(&marker, b"must survive cleanup").unwrap();
    let mut owner = OwnerFixture(
        std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", FIXTURE, "--nocapture", "--test-threads=1"])
            .env(MODE, "owner")
            .env(ROOT, root.path())
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::inherit())
            .spawn()
            .unwrap(),
    );
    let record = root.path().join(format!("owner-{}", owner.0.id()));
    wait_for(&record);
    let lines = std::fs::read_to_string(record).unwrap();
    let pid: u32 = lines.lines().nth(1).unwrap().parse().unwrap();
    cleanup_resources().unwrap();
    assert!(
        running(pid),
        "explicit cleanup evicted a live cross-process lease"
    );
    assert_eq!(std::fs::read(&marker).unwrap(), b"must survive cleanup");
    drop(owner.0.stdin.take());
    assert!(owner.0.wait().unwrap().success());
}

#[test]
fn windows_cleanup_reuses_equivalent_profile_across_processes_without_killing_survivor() {
    let root = tempfile::tempdir().unwrap();
    let owner = || {
        std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", FIXTURE, "--nocapture", "--test-threads=1"])
            .env(MODE, "owner")
            .env(ROOT, root.path())
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .spawn()
            .unwrap()
    };
    let mut first = OwnerFixture(owner());
    let mut second = OwnerFixture(owner());
    let read = |child: &std::process::Child| {
        let p = root.path().join(format!("owner-{}", child.id()));
        wait_for(&p);
        std::fs::read_to_string(p).unwrap()
    };
    let a = read(&first.0);
    let b = read(&second.0);
    assert_eq!(
        a.lines().next(),
        b.lines().next(),
        "equivalent policies must share a profile"
    );
    let survivor: u32 = b.lines().nth(1).unwrap().parse().unwrap();
    first.0.kill().unwrap();
    first.0.wait().unwrap();
    cleanup_resources().unwrap();
    assert!(
        running(survivor),
        "cleanup after one owner died killed another owner’s command"
    );
    drop(second.0.stdin.take());
    assert!(second.0.wait().unwrap().success());
}

#[test]
fn windows_idle_cleanup_removes_only_owned_private_state_and_aces() {
    let root = tempfile::tempdir().unwrap();
    let caller_marker = root.path().join("caller-output.txt");
    std::fs::write(&caller_marker, b"caller-owned").unwrap();
    let _cleanup = CleanupAfterTest;
    let foreign = ForeignProfileAce::grant(root.path());

    let resource = plan(root.path(), "hold").acquire().unwrap();
    let profile = resource.profile_name().to_string();
    let owned_temp = resource.private_tmp().unwrap().to_path_buf();
    assert!(
        super::launch::test_profile_has_ace(&profile, root.path()).unwrap(),
        "resource acquisition did not install its tracked root ACE"
    );
    let mut child = resource
        .spawn_with_stdio(WindowsStdio::Null, WindowsStdio::Null, WindowsStdio::Null)
        .unwrap();
    wait_for(&root.path().join(format!("ready-{}", child.id())));
    child.kill().unwrap();
    let _ = child.wait().unwrap();
    drop(child);
    drop(resource);

    cleanup_resources().unwrap();
    assert!(
        !super::launch::test_profile_has_ace(&profile, root.path()).unwrap(),
        "idle cleanup retained the resource ACE for {profile}"
    );
    assert!(
        super::launch::test_profile_has_ace(&foreign.profile, root.path()).unwrap(),
        "idle cleanup removed an unrelated external ACE"
    );
    assert!(
        !owned_temp.exists(),
        "idle cleanup retained the registered private directory for {profile}: {}",
        owned_temp.display()
    );
    assert_eq!(std::fs::read(&caller_marker).unwrap(), b"caller-owned");
}
