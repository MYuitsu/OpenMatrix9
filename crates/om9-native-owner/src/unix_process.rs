// SPDX-License-Identifier: LGPL-2.1-or-later
//! Native workers run in their own process group. The parent bounds output
//! and checks the kernel's lifetime memory peak before accepting a result.
use super::*;
use std::{
    io::{ErrorKind, Read},
    os::{fd::AsRawFd, unix::process::{CommandExt, ExitStatusExt}},
    process::{Child, Command, Stdio},
    thread,
    time::Instant,
};

/// Open a regular input without following the final symlink. The shared lock
/// coordinates cooperating readers and writers for the life of the handle.
pub fn open_regular_read_locked(path: &Path) -> std::io::Result<File> {
    use std::{fs::OpenOptions, os::unix::fs::OpenOptionsExt};
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC)
        .open(path)?;
    if !file.metadata()?.is_file() {
        return Err(std::io::Error::new(ErrorKind::InvalidInput, "expected a regular file"));
    }
    file.try_lock_shared()?;
    Ok(file)
}

struct OwnedChild(Child, bool);

impl OwnedChild {
    fn pid(&self) -> libc::pid_t {
        self.0.id() as libc::pid_t
    }

    fn reap(&mut self) -> std::io::Result<std::process::ExitStatus> {
        // SAFETY: the unreaped child owns this dedicated process group ID.
        unsafe { libc::kill(-self.pid(), libc::SIGKILL); }
        let status = self.0.wait()?;
        self.1 = true;
        Ok(status)
    }

    fn exited(&self) -> Result<bool> {
        // SAFETY: a live writable siginfo_t receives status for our owned
        // child. WNOWAIT keeps the child ID reserved until group cleanup.
        let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
        if unsafe { libc::waitid(libc::P_PID, self.pid() as libc::id_t, &mut info,
                                 libc::WNOHANG | libc::WEXITED | libc::WNOWAIT) } == -1 {
            let error = std::io::Error::last_os_error();
            return if error.kind() == ErrorKind::Interrupted { Ok(false) } else { Err(io(error)) };
        }
        #[cfg(target_os = "macos")]
        let pid = info.si_pid;
        #[cfg(target_os = "linux")]
        // SAFETY: waitid filled the child-status variant or left the buffer zeroed.
        let pid = unsafe { info.si_pid() };
        Ok(pid != 0)
    }

    #[cfg(target_os = "macos")]
    fn peak(&self) -> Option<u64> {
        // SAFETY: the writable rusage v4 object is the exact layout selected by
        // the flavor. The view is only used after a successful query.
        let mut info: libc::rusage_info_v4 = unsafe { std::mem::zeroed() };
        let result = unsafe {
            libc::proc_pid_rusage(self.pid(), libc::RUSAGE_INFO_V4,
                                  (&mut info as *mut libc::rusage_info_v4).cast())
        };
        (result == 0).then_some(info.ri_lifetime_max_phys_footprint.max(info.ri_phys_footprint))
    }

    #[cfg(target_os = "linux")]
    fn peak(&self) -> Option<u64> {
        let status = std::fs::read_to_string(format!("/proc/{}/status", self.pid())).ok()?;
        let peak = status.lines().find_map(|line| line.strip_prefix("VmHWM:"))?;
        peak.split_whitespace().next()?.parse::<u64>().ok()?.checked_mul(1024)
    }
}

impl Drop for OwnedChild {
    fn drop(&mut self) {
        if !self.1 {
            let _ = self.reap();
        }
    }
}

fn nonblocking(pipe: &impl AsRawFd) -> Result<()> {
    let fd = pipe.as_raw_fd();
    // SAFETY: fd belongs to the live pipe. F_GETFL/F_SETFL use scalar options.
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags == -1 || unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } == -1 {
        return Err(Failure::new(FailureKind::Io(format!(
            "configure worker pipe: {}", std::io::Error::last_os_error(),
        ))));
    }
    Ok(())
}

fn drain(pipe: &mut impl Read, output: &mut Vec<u8>, limit: usize) -> Result<bool> {
    let mut buffer = [0; 8192];
    loop {
        let count = match pipe.read(&mut buffer) {
            Ok(0) => return Ok(true),
            Ok(n) => n,
            Err(error) if error.kind() == ErrorKind::WouldBlock => return Ok(false),
            Err(error) if error.kind() == ErrorKind::Interrupted => continue,
            Err(error) => return Err(io(error)),
        };
        if count > limit.saturating_sub(output.len()) {
            return Err(Failure::new(FailureKind::OutputBudget));
        }
        output.try_reserve_exact(count).map_err(|_| Failure::new(FailureKind::OutputBudget))?;
        output.extend_from_slice(&buffer[..count]);
    }
}

pub(super) fn run(program: &NativeProgram, args: &[OsString], limits: &ProcessLimits,
                  cancel: &Cancellation) -> Result<ProcessOutput> {
    use std::os::unix::fs::MetadataExt;
    let held = program._image.metadata().map_err(io)?;
    let named = std::fs::symlink_metadata(&program.path).map_err(io)?;
    if !named.is_file() || (held.dev(), held.ino(), held.len(), held.ctime(), held.ctime_nsec())
        != (named.dev(), named.ino(), named.len(), named.ctime(), named.ctime_nsec())
    {
        return Err(Failure::new(FailureKind::Fingerprint));
    }
    // Rehash the current bundled image before launch. The application bundle is
    // a trusted installed input. A native child never chooses an executable.
    if NativeProgram::admit(&program.path, program.sha256).is_err() {
        return Err(Failure::new(FailureKind::Fingerprint));
    }
    let data_limit = limits.native_committed_bytes;
    let mut command = Command::new(&program.path);
    command.args(args).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped())
        .process_group(0);
    #[cfg(target_os = "linux")]
    {
        // Linux applies RLIMIT_DATA to brk and writable private mmap. Set the
        // ceiling in the child only, before exec. A wait4 peak can include the
        // parent's pre-exec mappings, so telemetry uses post-exec VmHWM.
        let limit = libc::rlimit { rlim_cur: data_limit, rlim_max: data_limit };
        // SAFETY: the hook uses only setrlimit and errno in the child before exec.
        unsafe {
            command.pre_exec(move || {
                if libc::setrlimit(libc::RLIMIT_DATA, &limit) == 0 {
                    Ok(())
                } else {
                    Err(std::io::Error::last_os_error())
                }
            });
        }
    }
    // Do not inherit loader overrides from a developer shell.
    for (name, _) in std::env::vars_os() {
        if name.as_encoded_bytes().starts_with(b"DYLD_") || name.as_encoded_bytes().starts_with(b"LD_") {
            command.env_remove(name);
        }
    }
    let start = Instant::now();
    let mut child = OwnedChild(command.spawn().map_err(|error| Failure::new(FailureKind::Io(
        format!("launch worker: {error}"),
    )))?, false);
    let mut out = child.0.stdout.take().expect("piped stdout");
    let mut err = child.0.stderr.take().expect("piped stderr");
    nonblocking(&out)?;
    nonblocking(&err)?;
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let mut peak = 0;
    let mut exited = false;
    let result = (|| {
        loop {
            if cancel.is_canceled() {
                return Err(Failure::new(FailureKind::Canceled));
            }
            if start.elapsed() >= limits.deadline {
                return Err(Failure::new(FailureKind::Deadline));
            }
            if let Some(sample) = child.peak() {
                peak = peak.max(sample);
            }
            if peak > data_limit {
                return Err(Failure::new(FailureKind::Invalid("native memory budget")));
            }
            exited = child.exited()?;
            let out_done = drain(&mut out, &mut stdout, limits.stdout_bytes)?;
            let err_done = drain(&mut err, &mut stderr, limits.stderr_bytes)?;
            if exited && out_done && err_done {
                // Darwin keeps the lifetime footprint on the exited child
                // until reap. Include allocations between parent samples.
                #[cfg(target_os = "macos")]
                {
                    let final_peak = child.peak().ok_or_else(|| Failure::new(
                        FailureKind::Io("cannot read final Darwin worker footprint".into()),
                    ))?;
                    peak = peak.max(final_peak);
                }
                let status = child.reap().map_err(io)?;
                if peak > data_limit {
                    return Err(Failure::new(FailureKind::Invalid("native memory budget")));
                }
                return Ok(status);
            }
            thread::sleep(Duration::from_millis(2));
        }
    })();
    if !child.1 {
        exited = child.reap().is_ok();
    }
    let stats = ProcessStats {
        pid: child.0.id(), peak_committed_bytes: peak,
        active_processes: u32::from(!exited), exited, elapsed: start.elapsed(),
    };
    match result {
        Ok(status) => {
            if status.success() {
                Ok(ProcessOutput { stdout, stderr, stats })
            } else {
                let code = status.code().or_else(|| status.signal()).unwrap_or(1) as u32;
                Err(Failure { kind: FailureKind::NativeExit(code), stats: Some(stats), stderr })
            }
        }
        Err(mut error) => {
            error.stats = Some(stats);
            error.stderr = stderr;
            Err(error)
        }
    }
}
