use super::*;
use std::io::Read;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const FIXTURE: &str = "backend::windows::replacement_tests::windows_replacement_fixture";
const MODE: &str = "__NUB_WINDOWS_REPLACEMENT_FIXTURE";
const ROOT: &str = "__NUB_WINDOWS_REPLACEMENT_ROOT";

fn plan(root: &Path, mode: &str) -> AppContainerLaunch {
    let mut launch = super::native_child_tests::plan(root, mode);
    launch.read_grants.push(root.join("package.json"));
    launch
}

fn command(root: &Path, mode: &str) -> Command {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args(["--exact", FIXTURE, "--nocapture", "--test-threads=1"])
        .env(MODE, mode)
        .env(ROOT, root)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    command
}

struct Owner(std::process::Child);
impl Drop for Owner {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn wait_for(path: &Path) {
    let deadline = Instant::now() + Duration::from_secs(30);
    while !path.exists() {
        assert!(Instant::now() < deadline, "waiting for {}", path.display());
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn isolated(mode: &str) {
    let root = tempfile::tempdir().unwrap();
    let caller = root.path().join("caller");
    let state = root.path().join("registry");
    std::fs::create_dir(&caller).unwrap();
    std::fs::create_dir(&state).unwrap();
    std::fs::write(caller.join("package.json"), b"original").unwrap();
    let lock_root = windows_registry::test_registry_root().unwrap();
    let mut child = Owner(
        command(&caller, mode)
            .env("ProgramData", &state)
            .env("__NUB_WINDOWS_CLEANUP_ACL_LOCK_ROOT", lock_root)
            .spawn()
            .unwrap(),
    );
    let status = child.0.wait().unwrap();
    if !status.success() {
        for directory in std::fs::read_dir(&state).unwrap() {
            let journal = directory.unwrap().path().join("registry.json");
            if journal.is_file() {
                // The runner's temp directory is not an uploaded artifact. Emit
                // ownership records into the captured log before retaining it.
                eprintln!(
                    "REPLACEMENT_JOURNAL {}\n{}",
                    journal.display(),
                    std::fs::read_to_string(&journal).unwrap()
                );
            }
        }
        let root = root.keep();
        panic!(
            "{mode} failed ({status}); journal retained at {}",
            root.display()
        );
    }
}

fn lookup_states(root: &Path) {
    use std::os::windows::fs::OpenOptionsExt;
    use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
    use windows_sys::Win32::Storage::FileSystem::{
        DeleteFileW, FILE_FLAG_BACKUP_SEMANTICS, FILE_ID_DESCRIPTOR, FILE_ID_DESCRIPTOR_0,
        FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE, FileIdType, OpenFileById,
    };

    let path = root.join("package.json");
    let moved = root.join("moved.json");
    let link = root.join("link.json");
    let expected = windows_registry::object_id(&path).unwrap().unwrap();
    let parts: Vec<u32> = expected
        .split(':')
        .map(|part| part.parse().unwrap())
        .collect();
    let lookup = |stage: &str, malformed: bool| {
        as_file_owner(|| {
            let hint = std::fs::OpenOptions::new()
                .access_mode(0)
                .custom_flags(FILE_FLAG_BACKUP_SEMANTICS)
                .open(root)
                .unwrap();
            let descriptor = FILE_ID_DESCRIPTOR {
                dwSize: if malformed {
                    0
                } else {
                    std::mem::size_of::<FILE_ID_DESCRIPTOR>() as u32
                },
                Type: FileIdType,
                Anonymous: FILE_ID_DESCRIPTOR_0 {
                    FileId: ((u64::from(parts[1]) << 32) | u64::from(parts[2])) as i64,
                },
            };
            let handle = unsafe {
                OpenFileById(
                    hint.as_raw_handle(),
                    &descriptor,
                    0x0002_0000 | 0x0004_0000,
                    FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
                    std::ptr::null(),
                    FILE_FLAG_BACKUP_SEMANTICS,
                )
            };
            let result = if handle == INVALID_HANDLE_VALUE {
                Err(std::io::Error::last_os_error().raw_os_error().unwrap())
            } else {
                let file = unsafe { std::fs::File::from_raw_handle(handle) };
                assert_eq!(
                    windows_registry::object_handle_id(file.as_raw_handle()).unwrap(),
                    expected
                );
                Ok(())
            };
            eprintln!(
                "FILE_ID_LOOKUP stage={stage} id={expected} malformed={malformed} result={result:?}"
            );
            result
        })
    };
    assert_eq!(lookup("live", false), Ok(()));
    assert_eq!(lookup("invalid-descriptor", true), Err(87));
    std::fs::rename(&path, &moved).unwrap();
    assert_eq!(lookup("renamed", false), Ok(()));
    std::fs::hard_link(&moved, &link).unwrap();
    std::fs::remove_file(&moved).unwrap();
    assert_eq!(lookup("hardlink-only", false), Ok(()));
    let held = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
        .open(&link)
        .unwrap();
    let wide: Vec<_> = link.as_os_str().encode_wide().chain(Some(0)).collect();
    assert_ne!(
        unsafe { DeleteFileW(wide.as_ptr()) },
        0,
        "{}",
        std::io::Error::last_os_error()
    );
    // Native Windows can still reopen this object while the named handle is
    // held. Record either outcome; the lookup validates and closes every handle.
    let _pending = lookup("delete-pending", false);
    drop(held);
    assert!(lookup("deleted", false).is_err());
    let recovered = super::launch::open_recorded_acl_file(&path, &expected);
    eprintln!("FILE_ID_RECOVERY deleted={recovered:?}");
    assert!(recovered.unwrap().is_none());

    let directory = root.join("retired-directory");
    std::fs::create_dir(&directory).unwrap();
    let directory_id = windows_registry::object_id(&directory).unwrap().unwrap();
    std::fs::remove_dir(&directory).unwrap();
    let recovered = super::launch::open_recorded_acl_file(&directory, &directory_id);
    eprintln!("FILE_ID_RECOVERY deleted-directory={recovered:?}");
    assert!(recovered.unwrap().is_none());
}

fn lookup_control_failure(root: &Path) {
    let path = root.join("package.json");
    let identity =
        windows_registry::PolicyIdentity::new([path.clone()], [], [], None, false, false)
            .unwrap()
            .with_objects([path.clone()])
            .unwrap();
    let mut resource = windows_registry::acquire(identity).unwrap();
    let profile = resource.entry.profile_name.clone();
    resource
        .record_mutation(windows_registry::AclMutation {
            path: path.display().to_string(),
            kind: windows_registry::AclKind::Subtree,
            access: 0x0012_0089,
        })
        .unwrap();
    let expected = resource.entry.object_ids.clone();
    super::launch::test_set_profile_ace(&profile, &path, true).unwrap();
    std::fs::remove_file(&path).unwrap();
    drop(resource);
    let error = cleanup_resources().unwrap_err();
    assert!(
        error.to_string().contains("OpenFileById live control"),
        "{error}"
    );
    let retained = windows_registry::test_entry(&profile).unwrap().unwrap();
    assert_eq!(retained.state, windows_registry::EntryState::RecoveryNeeded);
    assert_eq!(retained.object_ids, expected);
    assert!(!retained.mutations.is_empty());
}

fn replace(root: &Path, value: &str) {
    use windows_sys::Win32::Storage::FileSystem::{MOVEFILE_REPLACE_EXISTING, MoveFileExW};
    let pending = root.join("package.pending");
    let destination = root.join("package.json");
    std::fs::write(&pending, value).unwrap();
    let wide = |path: &Path| {
        path.as_os_str()
            .encode_wide()
            .chain(Some(0))
            .collect::<Vec<_>>()
    };
    assert_ne!(
        unsafe {
            MoveFileExW(
                wide(&pending).as_ptr(),
                wide(&destination).as_ptr(),
                MOVEFILE_REPLACE_EXISTING,
            )
        },
        0,
        "atomic replacement: {}",
        std::io::Error::last_os_error()
    );
}

fn read_package(launch: AppContainerLaunch, expected: &str) {
    let resource = launch.acquire().unwrap();
    let mut child = resource
        .spawn_with_stdio(
            WindowsStdio::Null,
            WindowsStdio::Piped,
            WindowsStdio::Inherit,
        )
        .unwrap();
    let mut output = String::new();
    child
        .take_stdout()
        .unwrap()
        .read_to_string(&mut output)
        .unwrap();
    assert!(child.wait().unwrap().success(), "{output}");
    assert!(output.contains(&format!("package:{expected}")), "{output}");
}

fn replacement_with_live_owner(root: &Path) {
    let path = root.join("package.json");
    let mut owner = Owner(
        command(root, "owner")
            .stdin(Stdio::piped())
            .spawn()
            .unwrap(),
    );
    wait_for(&root.join("owner.json"));
    let old: String =
        serde_json::from_slice(&std::fs::read(root.join("owner.json")).unwrap()).unwrap();
    let original = root.parent().unwrap().join("original.json");
    let hard_link = root.parent().unwrap().join("original-link.json");
    std::fs::rename(&path, &original).unwrap();
    std::fs::hard_link(&original, &hard_link).unwrap();
    replace(root, "replacement");
    // Model an atomic writer preserving the retired profile's explicit ACE.
    super::launch::test_set_profile_ace(&old, &path, true).unwrap();
    let foreign = format!("nub-replacement-foreign-{}", std::process::id());
    for file in [&original, &path] {
        super::launch::test_set_profile_ace(&foreign, file, true).unwrap();
    }
    let resource = plan(root, "hold").acquire().unwrap();
    let new = resource.profile_name().to_string();
    assert_ne!(old, new);
    let same = plan(root, "hold").acquire().unwrap();
    assert_eq!(same.profile_name(), new);
    let mut child = resource
        .spawn_with_stdio(WindowsStdio::Null, WindowsStdio::Null, WindowsStdio::Null)
        .unwrap();
    wait_for(&root.join(format!("ready-{}", child.id())));
    cleanup_resources().unwrap();
    assert!(super::launch::test_profile_has_ace(&old, &original).unwrap());
    drop(owner.0.stdin.take());
    assert!(owner.0.wait().unwrap().success());
    cleanup_resources().unwrap();
    assert!(windows_registry::test_entry(&old).unwrap().is_none());
    for file in [&original, &hard_link, &path] {
        assert!(!super::launch::test_profile_has_ace(&old, file).unwrap());
        assert!(super::launch::test_profile_has_ace(&foreign, file).unwrap());
    }
    assert!(super::launch::test_profile_has_ace(&new, &path).unwrap());
    assert!(child.try_wait().unwrap().is_none());
    read_package(plan(root, "read-package"), "replacement");
    std::fs::write(&path, b"in-place").unwrap();
    let unchanged = plan(root, "hold").acquire().unwrap();
    assert_eq!(unchanged.profile_name(), new);
    child.kill().unwrap();
    child.wait().unwrap();
    drop((child, unchanged, same, resource));
    cleanup_resources().unwrap();
    assert_eq!(std::fs::read(&original).unwrap(), b"original");
    assert_eq!(std::fs::read(&path).unwrap(), b"in-place");
    for file in [&original, &path] {
        super::launch::test_set_profile_ace(&foreign, file, false).unwrap();
    }
}

fn churn(root: &Path) {
    let mut profiles = std::collections::BTreeSet::new();
    for index in 0..windows_registry::MAX_IDLE_ENTRIES + 3 {
        replace(root, &index.to_string());
        let resource = plan(root, "hold").acquire().unwrap();
        profiles.insert(resource.profile_name().to_string());
        drop(resource);
        let registry: serde_json::Value = serde_json::from_slice(
            &std::fs::read(
                windows_registry::test_registry_root()
                    .unwrap()
                    .join("registry.json"),
            )
            .unwrap(),
        )
        .unwrap();
        assert!(
            registry["entries"].as_object().unwrap().len() <= windows_registry::MAX_IDLE_ENTRIES
        );
    }
    assert!(profiles.len() > windows_registry::MAX_IDLE_ENTRIES);
    cleanup_resources().unwrap();
    for profile in profiles {
        assert!(windows_registry::test_entry(&profile).unwrap().is_none());
    }
    assert!(root.join("package.json").exists());
}

fn crash_recovery(root: &Path) {
    let status = command(root, "fault")
        .env("__NUB_WINDOWS_CLEANUP_FIXTURE", "fault-acquire")
        .env("__NUB_WINDOWS_CLEANUP_ROOT", root)
        .env("__NUB_WINDOWS_CLEANUP_FAULT", "acl-installed-before-ready")
        .status()
        .unwrap();
    assert_eq!(status.code(), Some(91));
    let (_, profile, _): (String, String, PathBuf) =
        serde_json::from_slice(&std::fs::read(root.join("crash-transition.json")).unwrap())
            .unwrap();
    assert_eq!(
        windows_registry::test_entry(&profile)
            .unwrap()
            .unwrap()
            .state,
        windows_registry::EntryState::Preparing
    );
    std::fs::rename(root.join("package.json"), root.join("original.json")).unwrap();
    replace(root, "after-crash");
    cleanup_resources().unwrap();
    assert!(windows_registry::test_entry(&profile).unwrap().is_none());
    assert!(!super::launch::test_profile_has_ace(&profile, &root.join("original.json")).unwrap());
    read_package(plan(root, "read-package"), "after-crash");
    cleanup_resources().unwrap();
}

// Restrict the impersonating thread to its user's own SID and remove privileges
// (except the standard traverse bypass). An administrator group cannot satisfy
// the restricted access check, even on an elevated CI runner.
fn as_file_owner<T>(operation: impl FnOnce() -> T) -> T {
    use windows_sys::Win32::Security::{
        CreateRestrictedToken, DISABLE_MAX_PRIVILEGE, GetTokenInformation, ImpersonateLoggedOnUser,
        IsTokenRestricted, RevertToSelf, SID_AND_ATTRIBUTES, TOKEN_DUPLICATE, TOKEN_QUERY,
        TOKEN_USER, TokenUser,
    };
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
    let mut token = std::ptr::null_mut();
    assert_ne!(
        unsafe {
            OpenProcessToken(
                GetCurrentProcess(),
                TOKEN_DUPLICATE | TOKEN_QUERY,
                &mut token,
            )
        },
        0
    );
    let token = unsafe { OwnedHandle::from_raw_handle(token) };
    let mut bytes = 0;
    unsafe {
        GetTokenInformation(
            token.as_raw_handle(),
            TokenUser,
            std::ptr::null_mut(),
            0,
            &mut bytes,
        )
    };
    let mut buffer = vec![0usize; (bytes as usize).div_ceil(std::mem::size_of::<usize>())];
    assert_ne!(
        unsafe {
            GetTokenInformation(
                token.as_raw_handle(),
                TokenUser,
                buffer.as_mut_ptr().cast(),
                bytes,
                &mut bytes,
            )
        },
        0
    );
    let user = unsafe { &*buffer.as_ptr().cast::<TOKEN_USER>() };
    let sid = SID_AND_ATTRIBUTES {
        Sid: user.User.Sid,
        Attributes: 0,
    };
    let mut restricted = std::ptr::null_mut();
    assert_ne!(
        unsafe {
            CreateRestrictedToken(
                token.as_raw_handle(),
                DISABLE_MAX_PRIVILEGE,
                0,
                std::ptr::null(),
                0,
                std::ptr::null(),
                1,
                &sid,
                &mut restricted,
            )
        },
        0
    );
    let restricted = unsafe { OwnedHandle::from_raw_handle(restricted) };
    assert_ne!(unsafe { IsTokenRestricted(restricted.as_raw_handle()) }, 0);
    assert_ne!(
        unsafe { ImpersonateLoggedOnUser(restricted.as_raw_handle()) },
        0
    );
    struct Revert;
    impl Drop for Revert {
        fn drop(&mut self) {
            assert_ne!(unsafe { RevertToSelf() }, 0);
        }
    }
    let _revert = Revert;
    operation()
}

fn opened_object_binding(root: &Path) {
    let path = root.join("package.json");
    let identity =
        windows_registry::PolicyIdentity::new([path.clone()], [], [], None, false, false)
            .unwrap()
            .with_objects([path.clone()])
            .unwrap();
    let mut resource = windows_registry::acquire(identity).unwrap();
    let profile = resource.entry.profile_name.clone();
    let expected = windows_registry::object_id(&path).unwrap().unwrap();
    let file = super::launch::open_recorded_acl_file(&path, &expected)
        .unwrap()
        .unwrap();
    resource.validate_admitted_object(&path, &expected).unwrap();
    resource
        .record_mutation_id(
            windows_registry::AclMutation {
                path: path.display().to_string(),
                kind: windows_registry::AclKind::Subtree,
                access: 0x0012_0089,
            },
            Some(expected.clone()),
        )
        .unwrap();
    std::fs::rename(&path, root.join("original.json")).unwrap();
    replace(root, "replacement");
    super::launch::test_set_profile_ace_on_handle(&profile, &file, true).unwrap();
    assert!(!super::launch::test_profile_has_ace(&profile, &path).unwrap());
    assert!(resource.ready().is_err());
    drop(file);
    as_file_owner(|| {
        let original = super::launch::open_recorded_acl_file(&path, &expected)
            .unwrap()
            .unwrap();
        assert_eq!(
            windows_registry::object_handle_id(original.as_raw_handle()).unwrap(),
            expected
        );
        super::launch::test_set_profile_ace_on_handle(&profile, &original, false).unwrap();
    });
    assert!(!super::launch::test_profile_has_ace(&profile, &root.join("original.json")).unwrap());
    drop(resource);
    cleanup_resources().unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), b"replacement");
}

#[test]
fn windows_replacement_fixture() {
    let Ok(mode) = std::env::var(MODE) else {
        return;
    };
    let root = PathBuf::from(std::env::var_os(ROOT).unwrap());
    match mode.as_str() {
        "replace" => replacement_with_live_owner(&root),
        "churn" => churn(&root),
        "crash" => crash_recovery(&root),
        "binding" => opened_object_binding(&root),
        "lookup-states" => lookup_states(&root),
        "lookup-control-failure" => lookup_control_failure(&root),
        "fault" => {
            let _resource = plan(&root, "hold").acquire().unwrap();
            panic!("fault was not reached");
        }
        "owner" => {
            let resource = plan(&root, "hold").acquire().unwrap();
            let mut child = resource
                .spawn_with_stdio(WindowsStdio::Null, WindowsStdio::Null, WindowsStdio::Null)
                .unwrap();
            wait_for(&root.join(format!("ready-{}", child.id())));
            let pending = root.join("owner.pending");
            std::fs::write(
                &pending,
                serde_json::to_vec(resource.profile_name()).unwrap(),
            )
            .unwrap();
            std::fs::rename(pending, root.join("owner.json")).unwrap();
            std::io::stdin().read_to_end(&mut Vec::new()).unwrap();
            child.kill().unwrap();
            child.wait().unwrap();
        }
        _ => panic!("unknown replacement scenario {mode}"),
    }
}

#[test]
fn windows_fresh_acquisition_replaces_files_without_changing_live_owners() {
    isolated("replace");
}

#[test]
fn windows_replacement_churn_stays_inside_the_idle_cache_bound() {
    isolated("churn");
}

#[test]
fn windows_replacement_recovers_a_crashed_preparing_owner() {
    isolated("crash");
}

#[test]
fn windows_recorded_object_lookup_and_acl_write_need_only_file_owner_access() {
    isolated("binding");
}

#[test]
fn windows_recorded_object_lookup_distinguishes_link_and_deletion_states() {
    isolated("lookup-states");
}

#[test]
fn windows_recorded_object_lookup_keeps_ownership_when_live_control_fails() {
    isolated("lookup-control-failure");
}
