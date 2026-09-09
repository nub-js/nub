//! Per-command owner-death reaping for Unix launchers.
//!
//! ## Integration
//!
//! Construct [`UnixGuardian`] **before** the workload, then call
//! [`UnixGuardian::join_command`] on the one `Command` that will `exec` the workload.
//! The caller must retain the guardian alongside that child and let it drop only after
//! the child has been reaped. Do not also install another `setpgid(0, 0)` callback on
//! that command: this module puts the workload in the guardian's private group.
//!
//! The guardian starts as the group leader before the workload exists. That ordering
//! closes the old `workload first, watcher later` race: an owner or owner-group death
//! before readiness leaves no workload to orphan. Its lifeline writer is owned only by
//! the host; EOF therefore means the host died, including `SIGKILL`, and the guardian
//! kills its own group. `Drop` is deliberately stronger: it kills and reaps the group
//! even on normal cancellation/teardown, so the reusable sandbox API does not leave a
//! background command behind.
//!
//! This is group, not kernel-tree, ownership. A workload that intentionally calls
//! `setsid()` or otherwise leaves the group cannot be reaped by this unprivileged
//! mechanism. That unsupported detachment is an explicit platform limit, not a reason
//! to scan same-UID processes after owner death.

#![cfg(any(target_os = "linux", target_os = "macos"))]

use std::io;
use std::os::fd::RawFd;
use std::process::Command;

/// A started private process-group guardian.
///
/// The value is intentionally non-cloneable: exactly one host owns the lifeline writer
/// and is responsible for either retaining it through the workload or dropping it to
/// terminate the group.
pub(crate) struct UnixGuardian {
    group_id: libc::pid_t,
    guardian_pid: libc::pid_t,
    lifeline_writer: RawFd,
}

impl UnixGuardian {
    /// Start a guardian group before spawning a workload.
    ///
    /// The child-side path after `fork` uses only raw async-signal-safe operations. On
    /// macOS the descriptor enumeration buffer is allocated and its lazy binding is
    /// resolved in the parent before `fork`; the guardian then closes every inherited
    /// descriptor except its duplicated lifeline and ready descriptors. That prevents a
    /// proxy, temporary resource, or another guardian's writer from pinning this guard.
    pub(crate) fn start() -> io::Result<Self> {
        let mut lifeline = [-1; 2];
        let mut ready = [-1; 2];
        create_cloexec_pipe(&mut lifeline)?;
        if let Err(error) = create_cloexec_pipe(&mut ready) {
            close_fd(lifeline[0]);
            close_fd(lifeline[1]);
            return Err(error);
        }

        let mut fd_sweep = match FdSweep::prepare() {
            Ok(sweep) => sweep,
            Err(error) => {
                close_fd(lifeline[0]);
                close_fd(lifeline[1]);
                close_fd(ready[0]);
                close_fd(ready[1]);
                return Err(error);
            }
        };
        // SAFETY: after this fork the child does not return into Rust. It performs only
        // the raw syscall helpers below, then exits with `_exit`.
        let guardian_pid = unsafe { libc::fork() };
        if guardian_pid < 0 {
            let error = io::Error::last_os_error();
            close_fd(lifeline[0]);
            close_fd(lifeline[1]);
            close_fd(ready[0]);
            close_fd(ready[1]);
            return Err(error);
        }
        if guardian_pid == 0 {
            // The guardian must become a distinct group before it reports readiness.
            // Until then there is no workload, so an owner-group SIGKILL cannot orphan one.
            if unsafe { libc::setpgid(0, 0) } != 0 {
                unsafe { libc::_exit(127) };
            }
            if guardian_prepare_fds(lifeline[0], ready[1], &mut fd_sweep) != 0 {
                unsafe { libc::_exit(127) };
            }
            // `guardian_prepare_fds` retains duplicated descriptors, whose numbers are
            // stable only in this child. Recover them from the return slots.
            let (lifeline_read, ready_write) = fd_sweep.kept_fds();
            if write_byte(ready_write, READY) != 0 {
                unsafe { libc::_exit(127) };
            }
            close_fd(ready_write);
            guardian_loop(lifeline_read);
        }

        close_fd(lifeline[0]);
        close_fd(ready[1]);
        match read_byte(ready[0]) {
            Ok(READY) => {
                close_fd(ready[0]);
                Ok(Self {
                    group_id: guardian_pid,
                    guardian_pid,
                    lifeline_writer: lifeline[1],
                })
            }
            Ok(_) | Err(_) => {
                close_fd(ready[0]);
                close_fd(lifeline[1]);
                kill_and_wait(guardian_pid);
                Err(io::Error::other(
                    "Unix owner-death guardian failed before readiness",
                ))
            }
        }
    }

    /// Install the workload's pre-exec join into this guardian's private group.
    ///
    /// This closes the lifeline writer first, so even an owner death in the narrow
    /// pre-exec interval cannot leave a descendant pinning the EOF event. The closure
    /// captures only two integer descriptors and calls only async-signal-safe functions.
    pub(crate) fn join_command(&self, command: &mut Command) {
        use std::os::unix::process::CommandExt;

        let group_id = self.group_id;
        let lifeline_writer = self.lifeline_writer;
        // SAFETY: `close` and `setpgid` are async-signal-safe, and the closure captures
        // only `Copy` integers. Returning the OS error makes `Command::spawn` fail before
        // the workload execs if it cannot join the private group.
        unsafe {
            command.pre_exec(move || {
                if child_join_raw(lifeline_writer, group_id) != 0 {
                    return Err(io::Error::last_os_error());
                }
                Ok(())
            });
        }
    }

    /// Join the current raw-fork child to this guardian's group without using `Command`.
    ///
    /// Call only in the child side of a raw `fork`, before any allocation, lock, logging,
    /// or `exec`. It closes the copied lifeline writer before `setpgid`; a descendant
    /// therefore cannot pin the owner-EOF event while it is still pre-exec. A nonzero
    /// return is the raw syscall failure signal, suitable for an immediate `_exit(127)`.
    #[cfg(target_os = "linux")]
    pub(crate) fn child_join(&self) -> libc::c_int {
        child_join_raw(self.lifeline_writer, self.group_id)
    }

    /// The private group id to use for foreground-TTY handoff or diagnostics.
    pub(crate) fn process_group_id(&self) -> libc::pid_t {
        self.group_id
    }

    /// Terminate every remaining member of this launch-owned group and reap its guardian.
    ///
    /// Idempotent best effort: `ESRCH` is the normal case when the group has already
    /// exited, and a guardian killed by the group signal remains reapable by this host.
    pub(crate) fn terminate_and_reap(&mut self) {
        if self.guardian_pid < 0 {
            return;
        }
        if self.lifeline_writer >= 0 {
            close_fd(self.lifeline_writer);
            self.lifeline_writer = -1;
        }
        // SIGKILL is intentional for explicit teardown and matches the guardian's
        // owner-EOF path: retained background work must not outlive this owner.
        unsafe {
            libc::kill(-self.group_id, libc::SIGKILL);
        }
        wait_pid(self.guardian_pid);
        self.guardian_pid = -1;
        self.group_id = -1;
    }
}

impl Drop for UnixGuardian {
    fn drop(&mut self) {
        self.terminate_and_reap();
    }
}

const READY: u8 = b'R';

/// The only descriptors a guardian keeps after the raw fork.
///
/// macOS uses a parent-sized `PROC_PIDLISTFDS` buffer, matching the sandbox
/// launcher's descriptor sweep: `RLIMIT_NOFILE` can be about one million, so a blind
/// close loop makes each launch needlessly expensive. Linux uses `close_range` and falls
/// back to its already-captured soft descriptor limit only on an older kernel.
enum FdSweep {
    #[cfg(target_os = "macos")]
    Mac {
        entries: Vec<libc::proc_fdinfo>,
        bytes: libc::c_int,
        lifeline_read: RawFd,
        ready_write: RawFd,
    },
    #[cfg(target_os = "linux")]
    Linux {
        max_fd: RawFd,
        lifeline_read: RawFd,
        ready_write: RawFd,
    },
}

impl FdSweep {
    fn prepare() -> io::Result<Self> {
        #[cfg(target_os = "macos")]
        {
            // Parent-side query both sizes the fixed child buffer and resolves the
            // lazy symbol binding before fork; the child must not allocate or bind dylibs.
            let probed = unsafe {
                libc::proc_pidinfo(
                    std::process::id() as libc::c_int,
                    libc::PROC_PIDLISTFDS,
                    0,
                    std::ptr::null_mut(),
                    0,
                )
            };
            let base = if probed > 0 {
                probed as usize / std::mem::size_of::<libc::proc_fdinfo>()
            } else {
                0
            };
            let len = base
                .checked_mul(2)
                .and_then(|value| value.checked_add(256))
                .ok_or_else(|| io::Error::other("guardian descriptor list is too large"))?;
            let bytes = len
                .checked_mul(std::mem::size_of::<libc::proc_fdinfo>())
                .and_then(|value| libc::c_int::try_from(value).ok())
                .ok_or_else(|| io::Error::other("guardian descriptor buffer exceeds C int"))?;
            Ok(Self::Mac {
                entries: vec![
                    libc::proc_fdinfo {
                        proc_fd: 0,
                        proc_fdtype: 0,
                    };
                    len
                ],
                bytes,
                lifeline_read: -1,
                ready_write: -1,
            })
        }

        #[cfg(target_os = "linux")]
        {
            let mut limit = libc::rlimit {
                rlim_cur: 0,
                rlim_max: 0,
            };
            // SAFETY: this runs in the parent, before fork.
            if unsafe { libc::getrlimit(libc::RLIMIT_NOFILE, &mut limit) } != 0 {
                return Err(io::Error::last_os_error());
            }
            let max_fd = limit.rlim_cur.min(libc::c_int::MAX as libc::rlim_t) as RawFd;
            Ok(Self::Linux {
                max_fd,
                lifeline_read: -1,
                ready_write: -1,
            })
        }
    }

    /// Duplicate the two retained descriptors, then close every other inherited one.
    /// Returns zero only after both descriptors are the sole surviving descriptors.
    fn prepare_guardian(&mut self, lifeline: RawFd, ready: RawFd) -> libc::c_int {
        let lifeline_read = unsafe { libc::fcntl(lifeline, libc::F_DUPFD, 3) };
        if lifeline_read < 0 {
            return -1;
        }
        let ready_write = unsafe { libc::fcntl(ready, libc::F_DUPFD, lifeline_read + 1) };
        if ready_write < 0 {
            close_fd(lifeline_read);
            return -1;
        }

        match self {
            #[cfg(target_os = "macos")]
            Self::Mac {
                entries,
                bytes,
                lifeline_read: retained_read,
                ready_write: retained_ready,
            } => {
                let got = unsafe {
                    libc::proc_pidinfo(
                        libc::getpid(),
                        libc::PROC_PIDLISTFDS,
                        0,
                        entries.as_mut_ptr().cast(),
                        *bytes,
                    )
                };
                if got <= 0 || got >= *bytes {
                    close_fd(lifeline_read);
                    close_fd(ready_write);
                    return -1;
                }
                let count = got as usize / std::mem::size_of::<libc::proc_fdinfo>();
                for info in &entries[..count] {
                    if info.proc_fd != lifeline_read && info.proc_fd != ready_write {
                        close_fd(info.proc_fd);
                    }
                }
                *retained_read = lifeline_read;
                *retained_ready = ready_write;
                0
            }
            #[cfg(target_os = "linux")]
            Self::Linux {
                max_fd,
                lifeline_read: retained_read,
                ready_write: retained_ready,
            } => {
                if close_other_fds_linux(lifeline_read, ready_write, *max_fd) != 0 {
                    close_fd(lifeline_read);
                    close_fd(ready_write);
                    return -1;
                }
                *retained_read = lifeline_read;
                *retained_ready = ready_write;
                0
            }
        }
    }

    fn kept_fds(&self) -> (RawFd, RawFd) {
        match self {
            #[cfg(target_os = "macos")]
            Self::Mac {
                lifeline_read,
                ready_write,
                ..
            } => (*lifeline_read, *ready_write),
            #[cfg(target_os = "linux")]
            Self::Linux {
                lifeline_read,
                ready_write,
                ..
            } => (*lifeline_read, *ready_write),
        }
    }
}

fn guardian_prepare_fds(lifeline: RawFd, ready: RawFd, sweep: &mut FdSweep) -> libc::c_int {
    sweep.prepare_guardian(lifeline, ready)
}

fn child_join_raw(lifeline_writer: RawFd, group_id: libc::pid_t) -> libc::c_int {
    close_fd(lifeline_writer);
    // SAFETY: this helper is called only in the spawned workload child, before exec.
    // `setpgid` is async-signal-safe and `group_id` is the ready guardian's private group.
    if unsafe { libc::setpgid(0, group_id) } == 0 {
        0
    } else {
        -1
    }
}

#[cfg(target_os = "linux")]
fn close_other_fds_linux(first_keep: RawFd, second_keep: RawFd, max_fd: RawFd) -> libc::c_int {
    debug_assert!(first_keep < second_keep);
    for (first, last) in [
        (0, first_keep.saturating_sub(1)),
        (first_keep.saturating_add(1), second_keep.saturating_sub(1)),
    ] {
        if first <= last && close_range_linux(first as u32, last as u32) != 0 {
            return close_other_fds_linux_fallback(first_keep, second_keep, max_fd);
        }
    }
    if close_range_linux(second_keep.saturating_add(1) as u32, u32::MAX) != 0 {
        return close_other_fds_linux_fallback(first_keep, second_keep, max_fd);
    }
    0
}

#[cfg(target_os = "linux")]
fn close_range_linux(first: u32, last: u32) -> libc::c_int {
    // SAFETY: raw Linux syscall, used only in the post-fork guardian; the caller has
    // arranged that the two protected descriptors lie outside this range.
    let result = unsafe { libc::syscall(libc::SYS_close_range, first, last, 0) };
    if result == 0 { 0 } else { -1 }
}

#[cfg(target_os = "linux")]
fn close_other_fds_linux_fallback(
    first_keep: RawFd,
    second_keep: RawFd,
    max_fd: RawFd,
) -> libc::c_int {
    for fd in 0..max_fd {
        if fd != first_keep && fd != second_keep {
            close_fd(fd);
        }
    }
    0
}

fn guardian_loop(lifeline_read: RawFd) -> ! {
    let mut byte = 0u8;
    loop {
        // SAFETY: raw read into a one-byte stack buffer. EINTR is the only retryable
        // result; EOF and any other error represent owner loss.
        let read = unsafe { libc::read(lifeline_read, (&raw mut byte).cast(), 1) };
        if read < 0 && io::Error::last_os_error().raw_os_error() == Some(libc::EINTR) {
            continue;
        }
        break;
    }
    close_fd(lifeline_read);
    // The owner-death contract is a hard stop: a process that ignores TERM must not
    // outlive a crashed owner. This also kills the guardian itself, which is correct.
    unsafe {
        libc::kill(-libc::getpgrp(), libc::SIGKILL);
        libc::_exit(0);
    }
}

#[cfg(target_os = "macos")]
fn set_cloexec(fd: RawFd) -> io::Result<()> {
    // SAFETY: `fcntl` acts on the valid descriptor returned by `pipe`.
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
    if flags < 0 {
        return Err(io::Error::last_os_error());
    }
    if unsafe { libc::fcntl(fd, libc::F_SETFD, flags | libc::FD_CLOEXEC) } < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

fn create_cloexec_pipe(fds: &mut [RawFd; 2]) -> io::Result<()> {
    #[cfg(target_os = "linux")]
    {
        // SAFETY: `pipe2` initializes the two stack-local slots atomically with
        // CLOEXEC, so another thread's concurrently spawned command cannot pin the
        // owner lifeline before the workload's pre-exec close runs.
        if unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC) } != 0 {
            return Err(io::Error::last_os_error());
        }
        lift_pipe_above_stdio(fds)
    }

    #[cfg(target_os = "macos")]
    {
        // Darwin has no `pipe2(O_CLOEXEC)`. The descriptors are closed in every
        // workload's pre-exec callback and the guardian closes all inherited fds, but a
        // foreign concurrent fork in the `pipe`→`fcntl` window is a Darwin residual.
        // Embedder-wide fork serialization is required to eliminate that process-wide
        // race; this module never mistakes it for a kernel ownership guarantee.
        // SAFETY: `pipe` initializes the two stack-local slots.
        if unsafe { libc::pipe(fds.as_mut_ptr()) } != 0 {
            return Err(io::Error::last_os_error());
        }
        if let Err(error) = set_cloexec(fds[0]).and_then(|()| set_cloexec(fds[1])) {
            close_fd(fds[0]);
            close_fd(fds[1]);
            return Err(error);
        }
        lift_pipe_above_stdio(fds)
    }
}

fn lift_pipe_above_stdio(fds: &mut [RawFd; 2]) -> io::Result<()> {
    // The workload's stdio setup precedes pre_exec. A low-numbered lifeline
    // writer would be replaced by dup2, then child_join_raw would close stdout
    // instead of the inherited writer. Keep all guardian descriptors above it.
    for index in 0..fds.len() {
        if fds[index] >= 3 {
            continue;
        }
        let moved = unsafe { libc::fcntl(fds[index], libc::F_DUPFD_CLOEXEC, 3) };
        if moved < 0 {
            let error = io::Error::last_os_error();
            for fd in fds.iter() {
                close_fd(*fd);
            }
            return Err(error);
        }
        close_fd(fds[index]);
        fds[index] = moved;
    }
    Ok(())
}

fn read_byte(fd: RawFd) -> io::Result<u8> {
    let mut byte = 0u8;
    loop {
        // SAFETY: read exactly one byte into stack storage.
        let count = unsafe { libc::read(fd, (&raw mut byte).cast(), 1) };
        if count == 1 {
            return Ok(byte);
        }
        if count == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "guardian ready pipe closed",
            ));
        }
        let error = io::Error::last_os_error();
        if error.raw_os_error() != Some(libc::EINTR) {
            return Err(error);
        }
    }
}

fn write_byte(fd: RawFd, byte: u8) -> libc::c_int {
    loop {
        // SAFETY: write one byte from stack storage to the guardian-ready pipe.
        let count = unsafe { libc::write(fd, (&raw const byte).cast(), 1) };
        if count == 1 {
            return 0;
        }
        if count < 0 && io::Error::last_os_error().raw_os_error() == Some(libc::EINTR) {
            continue;
        }
        return -1;
    }
}

fn close_fd(fd: RawFd) {
    if fd >= 0 {
        // SAFETY: close is idempotent only at this module's ownership boundaries; errors
        // are deliberately ignored during cleanup because an already-closed fd is benign.
        unsafe {
            libc::close(fd);
        }
    }
}

fn kill_and_wait(pid: libc::pid_t) {
    // SAFETY: this is only the freshly-forked guardian before workload readiness.
    unsafe {
        libc::kill(pid, libc::SIGKILL);
    }
    wait_pid(pid);
}

fn wait_pid(pid: libc::pid_t) {
    if pid < 0 {
        return;
    }
    loop {
        // SAFETY: `pid` is this owner's guardian child. No status storage is needed.
        let result = unsafe { libc::waitpid(pid, std::ptr::null_mut(), 0) };
        if result == pid
            || result < 0 && io::Error::last_os_error().raw_os_error() != Some(libc::EINTR)
        {
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::UnixGuardian;
    use std::process::Command;

    #[test]
    fn closed_standard_descriptors_do_not_alias_the_guardian_lifeline() {
        const FLAG: &str = "SANDBOX_GUARDIAN_CLOSED_STDIO";
        if std::env::var_os(FLAG).is_some() {
            let guardian = UnixGuardian::start().unwrap();
            let mut command = Command::new("/bin/sh");
            command.args(["-c", "printf guardian-output"]);
            guardian.join_command(&mut command);
            let output = command.output().unwrap();
            assert!(output.status.success());
            assert_eq!(output.stdout, b"guardian-output");
            return;
        }
        use std::os::unix::process::CommandExt;
        let mut command = Command::new(std::env::current_exe().unwrap());
        command.args(["--exact", "backend::unix_guardian::tests::closed_standard_descriptors_do_not_alias_the_guardian_lifeline"]);
        command.env(FLAG, "1");
        // Close only in this throwaway owner, never in the parallel test host.
        unsafe {
            command.pre_exec(|| {
                libc::close(0);
                libc::close(1);
                Ok(())
            });
        }
        let output = command.output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn drop_terminates_a_joined_workload() {
        let guardian = UnixGuardian::start().expect("start guardian");
        let mut command = Command::new("sh");
        command.args(["-c", "exec sleep 30"]);
        guardian.join_command(&mut command);
        let mut child = command.spawn().expect("spawn joined workload");
        drop(guardian);
        let status = child.wait().expect("reap joined workload");
        assert!(
            !status.success(),
            "guardian drop must terminate the workload"
        );
    }

    #[test]
    fn process_group_id_is_positive_after_readiness() {
        let guardian = UnixGuardian::start().expect("start guardian");
        assert!(
            guardian.process_group_id() > 0,
            "ready guardian has a private group id"
        );
    }

    #[test]
    fn terminate_and_reap_is_idempotent() {
        let mut guardian = UnixGuardian::start().expect("start guardian");
        guardian.terminate_and_reap();
        guardian.terminate_and_reap();
        assert!(
            guardian.process_group_id() < 0,
            "reaped guardian clears its group id"
        );
    }
}
