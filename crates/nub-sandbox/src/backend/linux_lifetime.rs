//! Keep commands in their guardian's group without exposing terminal injection.
use seccompiler::{
    BpfProgram, SeccompAction, SeccompCmpArgLen, SeccompCmpOp, SeccompCondition, SeccompFilter,
    SeccompRule, TargetArch,
};
use std::collections::BTreeMap;
use std::io;

pub(super) fn program(freeze_group: bool) -> io::Result<BpfProgram> {
    let arch: TargetArch = std::env::consts::ARCH
        .try_into()
        .map_err(io::Error::other)?;
    let mut rules = BTreeMap::new();
    // ioctl's request argument is visible to seccomp. Block both terminal-input
    // injection interfaces; ordinary terminal reads, writes and sizing remain usable.
    let ioctls = [0x5412_u64, 0x541c_u64]
        .into_iter()
        .map(|request| {
            SeccompRule::new(vec![SeccompCondition::new(
                1,
                SeccompCmpArgLen::Dword,
                SeccompCmpOp::Eq,
                request,
            )?])
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(io::Error::other)?;
    rules.insert(libc::SYS_ioctl, ioctls);
    if freeze_group {
        rules.insert(libc::SYS_setsid, Vec::new());
        rules.insert(libc::SYS_setpgid, Vec::new());
    }
    let program = SeccompFilter::new(
        rules,
        SeccompAction::Allow,
        SeccompAction::Errno(libc::EPERM as u32),
        arch,
    )
    .map_err(io::Error::other)?
    .try_into()
    .map_err(io::Error::other)?;
    if arch == TargetArch::x86_64 {
        super::linux::prepend_x86_64_unsupported_abi_guard(program).map_err(io::Error::other)
    } else {
        Ok(program)
    }
}

/// Called only between fork and exec, with all BPF allocation done by the parent.
pub(super) fn install(program: &[seccompiler::sock_filter]) -> Result<(), i32> {
    if unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) } != 0 {
        return Err(unsafe { *libc::__errno_location() });
    }
    super::linux_supervisor::install_target_seccomp(program)
}

pub(super) fn attach(command: &mut std::process::Command) -> io::Result<()> {
    use std::os::unix::process::CommandExt;
    let program = program(true)?;
    // The preceding guardian hook joins the group; the filter then makes that
    // membership inherited and non-detachable for every descendant.
    unsafe {
        command.pre_exec(move || install(&program).map_err(io::Error::from_raw_os_error));
    }
    Ok(())
}
