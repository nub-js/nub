//! Embedded native compatibility assets owned by the persistent Windows lease.

use super::windows_registry::{self, Acquired};
use sha2::{Digest as _, Sha256};
use std::io::{self, Write as _};
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
        format!("{:x}", hash.finalize())
    });
    &VERSION
}

pub(super) fn asset_path(profile: &str) -> io::Result<PathBuf> {
    windows_registry::native_assets_path(profile)
}

pub(super) fn install(resource: &mut Acquired, path: &Path) -> io::Result<()> {
    if ASSETS.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "native compatibility requires an MSVC build",
        ));
    }
    // The protected registry parent prevents an AppContainer from replacing the
    // directory or its DLLs. The launcher later grants only this leaf read/execute.
    resource.record_private_path(path)?;
    std::fs::create_dir(path)?;
    for (name, bytes) in ASSETS {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path.join(name))?;
        file.write_all(bytes)?;
        file.sync_all()?;
    }
    Ok(())
}

pub(super) fn inject(process: *mut std::ffi::c_void, path: &Path) -> io::Result<()> {
    #[cfg(target_env = "msvc")]
    {
        unsafe extern "C" {
            fn sandbox_native_inject(process: *mut std::ffi::c_void, directory: *const u16) -> u32;
        }
        let path = super::strip_verbatim_prefix(path.to_path_buf());
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
