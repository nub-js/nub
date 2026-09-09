//! Explicit own-process metadata reads. Ordinary opens still run under Landlock.

use super::*;

const SECCOMP_ADDFD_FLAG_SEND: u32 = 2;

pub(super) fn check_atomic_addfd(nfd: RawFd) -> io::Result<()> {
    // The child is still behind the launch barrier: there are no notifications.
    // A supported ioctl validates flags and srcfd before looking up this absent id.
    let mut add = SeccompNotifAddfd {
        id: u64::MAX,
        flags: SECCOMP_ADDFD_FLAG_SEND,
        srcfd: nfd as u32,
        ..Default::default()
    };
    if ioctl_notif(nfd, notif_addfd(), &mut add as *mut _ as *mut libc::c_void) < 0
        && errno() == libc::ENOENT
    {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "self-process metadata requires atomic seccomp FD injection (Linux 5.14+)",
        ))
    }
}

pub(super) fn handle_open(state: &SupState, nfd: RawFd, req: &SeccompNotif) -> bool {
    let nr = req.data.nr as libc::c_long;
    let args = &req.data.args;
    let (pointer, flags) = if nr == libc::SYS_openat {
        (args[1], Some(args[2]))
    } else if nr == libc::SYS_openat2 {
        let mut how = [0u8; 24];
        let read = unsafe { read_child_mem(req.pid, args[2], &mut how) };
        let flags = (args[3] == 24 && read == 24)
            .then(|| u64::from_ne_bytes(how[..8].try_into().unwrap()))
            .filter(|_| how[8..].iter().all(|byte| *byte == 0));
        (args[1], flags)
    } else {
        #[cfg(target_arch = "x86_64")]
        if nr == libc::SYS_open {
            return open_path(state, nfd, req, args[0], Some(args[1]));
        }
        return false;
    };
    open_path(state, nfd, req, pointer, flags)
}

fn open_path(
    state: &SupState,
    nfd: RawFd,
    req: &SeccompNotif,
    pointer: u64,
    flags: Option<u64>,
) -> bool {
    let mut path = [0u8; 16];
    let read = unsafe { read_child_mem(req.pid, pointer, &mut path) };
    let file = match (read, &path) {
        (16, b"/proc/self/maps\0") => Some(SelfProcFile::Maps),
        (16, b"/proc/self/stat\0") => Some(SelfProcFile::Stat),
        _ => None,
    };
    let Some(file) = file else {
        // CONTINUE cannot grant a racing replacement path: Landlock remains the
        // authority for every ordinary open, including all other procfs paths.
        if state.write_matcher.is_some()
            && flags.is_none_or(|flags| flags & u64::from(write_open_mask()) != 0)
        {
            return false;
        }
        reply_continue(nfd, req.id);
        return true;
    };
    let permitted_flags = (libc::O_CLOEXEC
        | libc::O_LARGEFILE
        | libc::O_NONBLOCK
        | libc::O_NOCTTY
        | libc::O_NOFOLLOW) as u64;
    let Some(flags) = flags.filter(|flags| flags & !permitted_flags == 0) else {
        reply(nfd, req.id, -libc::EACCES);
        return true;
    };
    if !state.self_proc.contains(&file) {
        reply(nfd, req.id, -libc::EACCES);
        return true;
    }
    let mut id = req.id;
    if ioctl_notif(
        nfd,
        notif_id_valid(),
        &mut id as *mut _ as *mut libc::c_void,
    ) < 0
    {
        return true;
    }
    // The notification's kernel-supplied TID identifies the requesting process,
    // never a PID supplied in the path. Revalidation after opening prevents PID
    // recycling from returning another process's file; ADDFD_SEND is atomic.
    let pid = std::fs::read_to_string(format!("/proc/{}/status", req.pid))
        .ok()
        .and_then(|text| {
            text.lines().find_map(|line| {
                line.strip_prefix("Tgid:")
                    .and_then(|value| value.trim().parse::<u32>().ok())
            })
        });
    let Some(pid) = pid else {
        reply(nfd, req.id, -libc::ESRCH);
        return true;
    };
    let path = cstr(&format!("/proc/{pid}/{}", file.name()));
    let fd = unsafe { libc::open(path.as_ptr(), flags as i32 | libc::O_CLOEXEC) };
    if fd < 0 {
        reply(nfd, req.id, -errno());
        return true;
    }
    let fd = unsafe { OwnedFd::from_raw_fd(fd) };
    if ioctl_notif(
        nfd,
        notif_id_valid(),
        &mut id as *mut _ as *mut libc::c_void,
    ) < 0
    {
        return true;
    }
    let mut add = SeccompNotifAddfd {
        id: req.id,
        flags: SECCOMP_ADDFD_FLAG_SEND,
        srcfd: fd.as_raw_fd() as u32,
        newfd_flags: (flags & libc::O_CLOEXEC as u64) as u32,
        ..Default::default()
    };
    if ioctl_notif(nfd, notif_addfd(), &mut add as *mut _ as *mut libc::c_void) < 0 {
        reply(nfd, req.id, -errno());
    }
    true
}
