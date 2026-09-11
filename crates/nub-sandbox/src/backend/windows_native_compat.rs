//! Embedded native compatibility assets owned by the persistent Windows lease.

use super::windows::windows_registry::{self, Acquired};
use sha2::{Digest as _, Sha256};
use std::io;
#[cfg(target_env = "msvc")]
use std::io::Write as _;
#[cfg(target_env = "msvc")]
use std::os::windows::ffi::OsStrExt as _;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

#[cfg(target_env = "msvc")]
const ASSETS: &[(&str, &[u8])] = &[
    (
        "compat-x64.dll",
        include_bytes!(concat!(env!("OUT_DIR"), "/compat-x64.dll")),
    ),
    (
        "compat-arm64.dll",
        include_bytes!(concat!(env!("OUT_DIR"), "/compat-arm64.dll")),
    ),
];
#[cfg(not(target_env = "msvc"))]
const ASSETS: &[(&str, &[u8])] = &[];

pub(super) fn version() -> &'static str {
    static VERSION: LazyLock<String> = LazyLock::new(|| {
        let mut hash = Sha256::new();
        for (name, bytes) in ASSETS {
            hash.update(name.as_bytes());
            hash.update(bytes);
        }
        hash.finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    });
    &VERSION
}

pub(super) fn asset_path(profile: &str) -> io::Result<PathBuf> {
    windows_registry::native_assets_path(profile)
}

#[cfg(target_env = "msvc")]
pub(super) fn install(resource: &mut Acquired, path: &Path) -> io::Result<()> {
    // The protected registry parent prevents an AppContainer from replacing the
    // directory or its DLLs. The launcher later grants only this leaf read/execute.
    resource.record_private_path(path)?;
    #[cfg(test)]
    super::windows::launch::test_crash_transition(
        "native-assets-journaled",
        &resource.entry.profile_name,
        path,
    );
    std::fs::create_dir(path)?;
    #[cfg(test)]
    super::windows::launch::test_crash_transition(
        "native-assets-before-identity",
        &resource.entry.profile_name,
        path,
    );
    resource.record_mutation(windows_registry::AclMutation {
        path: path.to_string_lossy().into_owned(),
        kind: windows_registry::AclKind::Subtree,
        access: windows_sys::Win32::Foundation::GENERIC_READ
            | windows_sys::Win32::Foundation::GENERIC_EXECUTE,
    })?;
    #[cfg(test)]
    super::windows::launch::test_crash_transition(
        "native-assets-created",
        &resource.entry.profile_name,
        path,
    );
    for (name, bytes) in ASSETS {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path.join(name))?;
        file.write_all(bytes)?;
        file.sync_all()?;
        #[cfg(test)]
        super::windows::launch::test_crash_transition(
            "native-asset-written",
            &resource.entry.profile_name,
            path,
        );
    }
    #[cfg(test)]
    super::windows::launch::test_crash_transition(
        "native-assets-installed",
        &resource.entry.profile_name,
        path,
    );
    Ok(())
}

#[cfg(not(target_env = "msvc"))]
pub(super) fn install(_resource: &mut Acquired, _path: &Path) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "native compatibility requires an MSVC build",
    ))
}

pub(super) fn inject(process: *mut std::ffi::c_void, path: &Path) -> io::Result<()> {
    #[cfg(target_env = "msvc")]
    {
        unsafe extern "C" {
            fn sandbox_native_inject(process: *mut std::ffi::c_void, directory: *const u16) -> u32;
        }
        let path = super::windows::strip_verbatim_prefix(path.to_path_buf());
        let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
        // SAFETY: the launcher owns this suspended process handle, and the FFI
        // copies the terminated directory string before returning.
        let code = unsafe { sandbox_native_inject(process, wide.as_ptr()) };
        if code != 0 {
            return Err(io::Error::other(format!(
                "native sandbox compatibility injection: {}",
                io::Error::from_raw_os_error(code as i32)
            )));
        }
        Ok(())
    }
    #[cfg(not(target_env = "msvc"))]
    {
        let _ = (process, path);
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "native compatibility requires an MSVC build",
        ))
    }
}
