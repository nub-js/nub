use nub_sandbox::{Degradation, Sandbox, SandboxPolicy};

pub fn acquire(policy: &SandboxPolicy) -> Result<Sandbox, Degradation> {
    #[cfg(windows)]
    if std::env::var_os("NUB_NATIVE_EMBEDDED_ADAPTER").is_some() {
        return Sandbox::with_windows_native_compat(policy);
    }
    Sandbox::new(policy)
}
