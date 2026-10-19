// SPDX-License-Identifier: LGPL-2.1-or-later
use super::*;
use std::{
    ffi::{OsStr, c_void},
    mem::{ManuallyDrop, size_of},
    os::windows::{
        ffi::OsStrExt,
        io::{AsRawHandle, FromRawHandle},
    },
    ptr::{null, null_mut},
    thread,
    time::Instant,
};
use windows_sys::Win32::{
    Foundation::*,
    Security::SECURITY_ATTRIBUTES,
    Storage::FileSystem::*,
    System::{
        JobObjects::*,
        Pipes::{CreatePipe, PeekNamedPipe},
        Threading::*,
    },
};
struct Handle(HANDLE);
impl Handle {
    fn new(raw: HANDLE) -> Result<Self> {
        if raw.is_null() || raw == INVALID_HANDLE_VALUE {
            Err(io(std::io::Error::last_os_error()))
        } else {
            Ok(Self(raw))
        }
    }
    fn file(self) -> File {
        let this = ManuallyDrop::new(self);
        // SAFETY: one valid owned handle is transferred exactly once to File.
        unsafe { File::from_raw_handle(this.0) }
    }
}
impl Drop for Handle {
    fn drop(&mut self) {
        // SAFETY: Handle owns a non-null live Win32 handle; no other owner closes it.
        unsafe {
            CloseHandle(self.0);
        }
    }
}
fn bool_result(value: i32) -> Result<()> {
    if value == 0 {
        Err(io(std::io::Error::last_os_error()))
    } else {
        Ok(())
    }
}
/// Show one owned startup-failure dialog and return whether the call was made.
///
/// The application owns the decision to display it; audits and automation
/// suppress it through `OM9_STARTUP_DIALOG=0`. Text is bounded before it reaches
/// Win32 and no application state crosses this boundary.
pub fn report_startup_failure(title: &str, message: &str) -> bool {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        MB_ICONERROR, MB_OK, MB_SETFOREGROUND, MessageBoxW,
    };
    const MAX_CHARS: usize = 512;
    fn wide(text: &str) -> Vec<u16> {
        text.chars()
            .take(MAX_CHARS)
            .collect::<String>()
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect()
    }
    let caption = wide(title);
    let body = wide(message);
    // SAFETY: both UTF-16 buffers are NUL terminated and outlive the call; a null
    // owner window is valid for a task-modal message box that returns no data.
    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            body.as_ptr(),
            caption.as_ptr(),
            MB_OK | MB_ICONERROR | MB_SETFOREGROUND,
        );
    }
    true
}
fn pipe() -> Result<(Handle, Handle)> {
    let attributes = SECURITY_ATTRIBUTES {
        nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: null_mut(),
        bInheritHandle: 1,
    };
    let mut read = null_mut();
    let mut write = null_mut();
    // SAFETY: initialized attributes and two writable HANDLE outputs live for
    // the call. Returned handles become unique RAII owners immediately.
    bool_result(unsafe { CreatePipe(&mut read, &mut write, &attributes, 8192) })?;
    let read = Handle::new(read)?;
    let write = Handle::new(write)?;
    // SAFETY: only child write ends are inheritable; read end remains parent-owned.
    bool_result(unsafe { SetHandleInformation(read.0, HANDLE_FLAG_INHERIT, 0) })?;
    Ok((read, write))
}
struct Attributes {
    storage: Vec<usize>,
}
impl Attributes {
    fn new(handles: &mut [HANDLE]) -> Result<Self> {
        let mut bytes = 0;
        // SAFETY: null first call queries required storage; no buffer is accessed.
        unsafe {
            InitializeProcThreadAttributeList(null_mut(), 1, 0, &mut bytes);
        }
        if bytes == 0 {
            return Err(io(std::io::Error::last_os_error()));
        }
        let mut storage = vec![0usize; bytes.div_ceil(size_of::<usize>())];
        let pointer = storage.as_mut_ptr().cast();
        // SAFETY: word-aligned allocation contains at least queried bytes.
        bool_result(unsafe { InitializeProcThreadAttributeList(pointer, 1, 0, &mut bytes) })?;
        let result = Self { storage };
        // SAFETY: initialized list and inheritable live handle array remain
        // alive until CreateProcess returns; only these three handles are inherited.
        bool_result(unsafe {
            UpdateProcThreadAttribute(
                pointer,
                0,
                PROC_THREAD_ATTRIBUTE_HANDLE_LIST as usize,
                handles.as_mut_ptr().cast(),
                std::mem::size_of_val(handles),
                null_mut(),
                null(),
            )
        })?;
        Ok(result)
    }
    fn pointer(&mut self) -> LPPROC_THREAD_ATTRIBUTE_LIST {
        self.storage.as_mut_ptr().cast()
    }
}
impl Drop for Attributes {
    fn drop(&mut self) {
        // SAFETY: successful initialization is required to construct this owner.
        unsafe {
            DeleteProcThreadAttributeList(self.pointer());
        }
    }
}
fn wide(value: &OsStr) -> Result<Vec<u16>> {
    let value: Vec<_> = value.encode_wide().collect();
    if value.contains(&0) {
        return Err(Failure::new(FailureKind::Invalid("NUL in native argument")));
    }
    Ok(value)
}
fn command_line(program: &Path, args: &[OsString]) -> Result<Vec<u16>> {
    let values: Vec<_> = std::iter::once(program.as_os_str())
        .chain(args.iter().map(OsString::as_os_str))
        .collect();
    let bound = values
        .iter()
        .try_fold(0usize, |n, s| {
            s.encode_wide()
                .count()
                .checked_mul(2)
                .and_then(|m| m.checked_add(3))
                .and_then(|m| n.checked_add(m))
        })
        .ok_or_else(|| Failure::new(FailureKind::Invalid("native command line size")))?;
    if bound > 32766 {
        return Err(Failure::new(FailureKind::Invalid(
            "native command line size",
        )));
    }
    let mut command = Vec::with_capacity(bound + 1);
    for value in values {
        if !command.is_empty() {
            command.push(b' ' as u16);
        }
        command.push(b'"' as u16);
        let mut slashes = 0;
        for c in wide(value)? {
            if c == b'\\' as u16 {
                slashes += 1;
                continue;
            }
            if c == b'"' as u16 {
                command.extend(std::iter::repeat_n(b'\\' as u16, slashes * 2 + 1));
            } else {
                command.extend(std::iter::repeat_n(b'\\' as u16, slashes));
            }
            command.push(c);
            slashes = 0;
        }
        command.extend(std::iter::repeat_n(b'\\' as u16, slashes * 2));
        command.push(b'"' as u16);
    }
    command.push(0);
    Ok(command)
}
fn environment() -> Result<Vec<u16>> {
    let mut environment = vec![];
    for name in ["SystemRoot", "TEMP", "TMP"] {
        if let Some(value) = std::env::var_os(name) {
            environment.extend(wide(OsStr::new(name))?);
            environment.push(b'=' as u16);
            environment.extend(wide(&value)?);
            environment.push(0);
        }
    }
    if environment.is_empty() {
        environment.push(0);
    }
    environment.push(0);
    Ok(environment)
}
#[derive(Default)]
struct Capture {
    bytes: Vec<u8>,
    error: Option<FailureKind>,
}
struct PipeCapture {
    read: File,
    capture: Capture,
    limit: usize,
    closed: bool,
}
impl PipeCapture {
    fn new(read: Handle, limit: usize) -> Self {
        Self {
            read: read.file(),
            capture: Capture::default(),
            limit,
            closed: false,
        }
    }
    // This owner is the only reader and never starts a pending read. Peek and
    // Read operate on the same exclusive handle; bytes already observed cannot
    // be consumed by another reader. Each pump reads at most one bounded chunk.
    fn pump(&mut self) -> bool {
        if self.closed {
            return false;
        }
        let mut available = 0;
        // SAFETY: the private read handle has no competing/pending I/O, and the
        // writable count lives for the call. No data buffer is requested.
        if unsafe {
            PeekNamedPipe(
                self.read.as_raw_handle(),
                null_mut(),
                0,
                null_mut(),
                &mut available,
                null_mut(),
            )
        } == 0
        {
            let error = std::io::Error::last_os_error();
            if error.raw_os_error() != Some(ERROR_BROKEN_PIPE as i32) {
                self.capture
                    .error
                    .get_or_insert(FailureKind::Io(error.to_string()));
            }
            self.closed = true;
            return true;
        }
        if available == 0 {
            return false;
        }
        let mut buffer = [0; 8192];
        let n = (available as usize).min(buffer.len());
        // Available bytes remain owned by this read end even if the writer
        // closes. Reading no more than that count cannot wait for future writes.
        let n = match self.read.read(&mut buffer[..n]) {
            Ok(n) if n > 0 => n,
            Ok(_) => {
                self.capture
                    .error
                    .get_or_insert(FailureKind::Io("native pipe short read".into()));
                self.closed = true;
                return true;
            }
            Err(error) => {
                self.capture
                    .error
                    .get_or_insert(FailureKind::Io(error.to_string()));
                self.closed = true;
                return true;
            }
        };
        if self.capture.error.is_none() {
            let keep = n.min(self.limit - self.capture.bytes.len());
            if self.capture.bytes.try_reserve_exact(keep).is_err() {
                self.capture.error = Some(FailureKind::OutputBudget);
            } else {
                self.capture.bytes.extend_from_slice(&buffer[..keep]);
                if keep != n {
                    self.capture.error = Some(FailureKind::OutputBudget);
                }
            }
        }
        true
    }
}
struct Streams {
    stdout_write: Option<Handle>,
    stderr_write: Option<Handle>,
    stdout: PipeCapture,
    stderr: PipeCapture,
}
impl Streams {
    fn new(limits: &ProcessLimits) -> Result<Self> {
        let (stdout_read, stdout_write) = pipe()?;
        let (stderr_read, stderr_write) = pipe()?;
        Ok(Self {
            stdout_write: Some(stdout_write),
            stderr_write: Some(stderr_write),
            stdout: PipeCapture::new(stdout_read, limits.stdout_bytes),
            stderr: PipeCapture::new(stderr_read, limits.stderr_bytes),
        })
    }
    fn close_writes(&mut self) {
        self.stdout_write.take();
        self.stderr_write.take();
    }
    fn pump(&mut self) -> bool {
        let stderr = self.stderr.pump();
        let stdout = self.stdout.pump();
        stdout || stderr
    }
    fn closed(&self) -> bool {
        self.stdout.closed && self.stderr.closed
    }
    fn error(&self) -> Option<FailureKind> {
        self.stdout
            .capture
            .error
            .clone()
            .or_else(|| self.stderr.capture.error.clone())
    }
}
#[derive(Clone, Copy)]
struct TeardownClock {
    end: Option<Instant>,
    limit: Duration,
}
impl TeardownClock {
    fn begin(&mut self, now: Instant) -> Instant {
        *self.end.get_or_insert_with(|| now + self.limit)
    }
    fn remaining(&self, now: Instant) -> Option<Duration> {
        self.end.and_then(|end| end.checked_duration_since(now))
    }
}
struct Child {
    job: Handle,
    process: Handle,
    thread: Handle,
    pid: u32,
    exited: bool,
    teardown: TeardownClock,
}
impl Child {
    fn terminate(&self) {
        // SAFETY: this dedicated job/process are live owned handles. Both calls
        // request termination without waiting; process covers failed assignment.
        unsafe {
            TerminateJobObject(self.job.0, 0xE0000001);
            if !self.exited {
                TerminateProcess(self.process.0, 0xE0000001);
            }
        }
    }
    fn poll_exit(&mut self) -> Result<()> {
        if !self.exited {
            // SAFETY: a zero-time probe never waits or grants a cleanup budget.
            match unsafe { WaitForSingleObject(self.process.0, 0) } {
                WAIT_OBJECT_0 => self.exited = true,
                WAIT_TIMEOUT => {}
                _ => return Err(io(std::io::Error::last_os_error())),
            }
        }
        Ok(())
    }
    fn stats(&self, elapsed: Duration) -> Result<ProcessStats> {
        let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        let mut accounting = JOBOBJECT_BASIC_ACCOUNTING_INFORMATION::default();
        // SAFETY: correctly typed and sized writable outputs; job stays alive.
        bool_result(unsafe {
            QueryInformationJobObject(
                self.job.0,
                JobObjectExtendedLimitInformation,
                (&mut limits as *mut JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
                null_mut(),
            )
        })?;
        // SAFETY: as above, for accounting rather than extended limits.
        bool_result(unsafe {
            QueryInformationJobObject(
                self.job.0,
                JobObjectBasicAccountingInformation,
                (&mut accounting as *mut JOBOBJECT_BASIC_ACCOUNTING_INFORMATION).cast(),
                size_of::<JOBOBJECT_BASIC_ACCOUNTING_INFORMATION>() as u32,
                null_mut(),
            )
        })?;
        Ok(ProcessStats {
            pid: self.pid,
            peak_committed_bytes: limits.PeakJobMemoryUsed as u64,
            active_processes: accounting.ActiveProcesses,
            exited: self.exited,
            elapsed,
        })
    }
}
impl Drop for Child {
    fn drop(&mut self) {
        // No waits, reader joins or new deadlines in Drop. The last job handle
        // has KILL_ON_JOB_CLOSE; error cleanup cannot create live resource owners.
        self.terminate();
    }
}
// A signaled process does not promise synchronous Job ActiveProcesses cleanup.
// Query the actual job state, rather than replacing it with an assumed zero.
// The clock/pause seams let tests exercise deadline accounting without sleeps.
fn drain_job(
    mut query: impl FnMut() -> Result<(ProcessStats, bool)>,
    mut remaining: impl FnMut() -> Option<Duration>,
    mut pause: impl FnMut(Duration),
) -> Result<ProcessStats> {
    let (mut stats, mut pipes_closed) = query()?;
    loop {
        // Check after each query too: scheduling inside the Win32 query must
        // not turn an observation that arrived after expiry into late success.
        let Some(wait) = remaining().filter(|wait| !wait.is_zero()) else {
            return Err(Failure {
                kind: FailureKind::Teardown,
                stats: Some(stats),
                stderr: vec![],
            });
        };
        if stats.exited && stats.active_processes == 0 && pipes_closed {
            return Ok(stats);
        }
        pause(wait.min(Duration::from_millis(10)));
        (stats, pipes_closed) = query().map_err(|mut error| {
            error.stats.get_or_insert_with(|| stats.clone());
            error
        })?;
    }
}
pub(super) fn run(
    program: &Path,
    args: &[OsString],
    limits: &ProcessLimits,
    cancel: &Cancellation,
) -> Result<ProcessOutput> {
    let started = Instant::now();
    let mut command = command_line(program, args)?;
    let mut application = wide(program.as_os_str())?;
    application.push(0);
    let env = environment()?;
    // SAFETY: unnamed job has no inherited security attributes or shared owner.
    let job = Handle::new(unsafe { CreateJobObjectW(null(), null()) })?;
    let mut information = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
    information.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
        | JOB_OBJECT_LIMIT_PROCESS_MEMORY
        | JOB_OBJECT_LIMIT_JOB_MEMORY
        | JOB_OBJECT_LIMIT_ACTIVE_PROCESS;
    information.BasicLimitInformation.ActiveProcessLimit = 1;
    information.ProcessMemoryLimit = limits.native_committed_bytes as usize;
    information.JobMemoryLimit = limits.native_committed_bytes as usize;
    // SAFETY: initialized typed limit structure with exact byte length.
    bool_result(unsafe {
        SetInformationJobObject(
            job.0,
            JobObjectExtendedLimitInformation,
            (&information as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
            size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        )
    })?;
    let mut streams = Streams::new(limits)?;
    let security = SECURITY_ATTRIBUTES {
        nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: null_mut(),
        bInheritHandle: 1,
    };
    let nul = wide(OsStr::new("NUL"))?;
    let mut nul = nul;
    nul.push(0);
    // SAFETY: constant NUL device name and live security attributes. No desktop
    // or parent console input is inherited by the native operation.
    let stdin = Handle::new(unsafe {
        CreateFileW(
            nul.as_ptr(),
            GENERIC_READ,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            &security,
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            null_mut(),
        )
    })?;
    let mut inherited = [
        stdin.0,
        streams.stdout_write.as_ref().expect("stdout write").0,
        streams.stderr_write.as_ref().expect("stderr write").0,
    ];
    let mut attributes = Attributes::new(&mut inherited)?;
    let mut startup = STARTUPINFOEXW::default();
    startup.StartupInfo.cb = size_of::<STARTUPINFOEXW>() as u32;
    startup.StartupInfo.dwFlags = STARTF_USESTDHANDLES | STARTF_USESHOWWINDOW;
    startup.StartupInfo.wShowWindow = 0;
    startup.StartupInfo.hStdInput = inherited[0];
    startup.StartupInfo.hStdOutput = inherited[1];
    startup.StartupInfo.hStdError = inherited[2];
    startup.lpAttributeList = attributes.pointer();
    let mut process = PROCESS_INFORMATION::default();
    if cancel.is_canceled() {
        return Err(Failure::new(FailureKind::Canceled));
    }
    // SAFETY: all UTF-16 buffers are owned, NUL-terminated and live; command is
    // mutable as required. Only explicit stdio handles are inherited. Suspension
    // prevents any native entry before successful assignment to the limited job.
    bool_result(unsafe {
        CreateProcessW(
            application.as_ptr(),
            command.as_mut_ptr(),
            null(),
            null(),
            1,
            CREATE_SUSPENDED
                | CREATE_NO_WINDOW
                | CREATE_UNICODE_ENVIRONMENT
                | EXTENDED_STARTUPINFO_PRESENT,
            env.as_ptr().cast::<c_void>(),
            null(),
            &startup.StartupInfo,
            &mut process,
        )
    })?;
    let mut child = Child {
        job,
        process: Handle::new(process.hProcess)?,
        thread: Handle::new(process.hThread)?,
        pid: process.dwProcessId,
        exited: false,
        teardown: TeardownClock {
            end: None,
            limit: limits.teardown,
        },
    };
    streams.close_writes();
    // SAFETY: suspended live process and configured job are uniquely owned.
    let mut failure =
        bool_result(unsafe { AssignProcessToJobObject(child.job.0, child.process.0) })
            .err()
            .map(|error| error.kind);
    // SAFETY: the job is assigned and limits are effective before resuming entry.
    if failure.is_none() && unsafe { ResumeThread(child.thread.0) } == u32::MAX {
        failure = Some(FailureKind::Io(std::io::Error::last_os_error().to_string()));
    }
    while failure.is_none() {
        if cancel.is_canceled() {
            failure = Some(FailureKind::Canceled);
            break;
        }
        if started.elapsed() >= limits.deadline {
            failure = Some(FailureKind::Deadline);
            break;
        }
        let progress = streams.pump();
        if let Some(error) = streams.error() {
            failure = Some(error);
            break;
        }
        // SAFETY: process handle remains live and this bounded wait cannot block
        // the owning worker beyond the configured cancellation polling interval.
        let wait = if progress {
            0
        } else {
            limits
                .deadline
                .saturating_sub(started.elapsed())
                .min(Duration::from_millis(10))
                .as_millis() as u32
        };
        match unsafe { WaitForSingleObject(child.process.0, wait) } {
            WAIT_OBJECT_0 => {
                child.exited = true;
                break;
            }
            WAIT_TIMEOUT => {}
            _ => {
                failure = Some(FailureKind::Io(std::io::Error::last_os_error().to_string()));
                break;
            }
        }
    }
    // All cleanup uses one stored absolute deadline. No termination call waits
    // first, and no failure return can start a second budget in Drop.
    child.teardown.begin(Instant::now());
    let teardown_clock = child.teardown;
    if failure.is_some() {
        child.terminate();
    }
    let stats = match drain_job(
        || {
            let exit = child.poll_exit();
            streams.pump();
            let stats = child.stats(started.elapsed())?;
            if let Err(mut error) = exit {
                error.stats = Some(stats);
                return Err(error);
            }
            Ok((stats, streams.closed()))
        },
        || teardown_clock.remaining(Instant::now()),
        thread::sleep,
    ) {
        Ok(stats) => stats,
        Err(mut error) => {
            error.stderr = streams.stderr.capture.bytes;
            return Err(error);
        }
    };
    if failure.is_none() {
        failure = streams.error();
    }
    let mut code = 0;
    // SAFETY: exited process handle remains live, and code is a writable DWORD.
    if let Err(mut error) = bool_result(unsafe { GetExitCodeProcess(child.process.0, &mut code) }) {
        error.stats = Some(stats);
        error.stderr = streams.stderr.capture.bytes;
        return Err(error);
    }
    if failure.is_none() && cancel.is_canceled() {
        failure = Some(FailureKind::Canceled);
    }
    if failure.is_none() && started.elapsed() >= limits.deadline {
        failure = Some(FailureKind::Deadline);
    }
    if failure.is_none() && code != 0 {
        failure = Some(FailureKind::NativeExit(code));
    }
    if let Some(kind) = failure {
        return Err(Failure {
            kind,
            stats: Some(stats),
            stderr: streams.stderr.capture.bytes,
        });
    }
    Ok(ProcessOutput {
        stdout: streams.stdout.capture.bytes,
        stderr: streams.stderr.capture.bytes,
        stats,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::Cell, collections::VecDeque, io::Write};

    fn stats(active_processes: u32, elapsed: Duration) -> ProcessStats {
        ProcessStats {
            pid: 23,
            peak_committed_bytes: 4096,
            active_processes,
            exited: true,
            elapsed,
        }
    }

    #[test]
    fn delayed_accounting_requires_observed_zero_without_fixed_sleep() {
        let elapsed = Cell::new(Duration::ZERO);
        let mut sequence = VecDeque::from([1, 1, 0]);
        let result = drain_job(
            || {
                Ok((
                    stats(sequence.pop_front().expect("extra query"), elapsed.get()),
                    true,
                ))
            },
            || Duration::from_millis(30).checked_sub(elapsed.get()),
            |wait| elapsed.set(elapsed.get() + wait),
        )
        .unwrap();
        assert_eq!(result.active_processes, 0);
        assert_eq!(result.elapsed, Duration::from_millis(20));
        assert!(sequence.is_empty());
    }

    #[test]
    fn persistent_active_job_expires_at_the_remaining_teardown_budget() {
        // Termination/readers have already consumed part of the shared budget.
        let elapsed = Cell::new(Duration::from_millis(25));
        let queries = Cell::new(0);
        let failure = drain_job(
            || {
                queries.set(queries.get() + 1);
                Ok((stats(1, elapsed.get()), true))
            },
            || Duration::from_millis(40).checked_sub(elapsed.get()),
            |wait| elapsed.set(elapsed.get() + wait),
        )
        .unwrap_err();
        assert_eq!(failure.kind, FailureKind::Teardown);
        assert_eq!(failure.stats.unwrap().active_processes, 1);
        assert_eq!(elapsed.get(), Duration::from_millis(40));
        assert_eq!(queries.get(), 3);
    }

    #[test]
    fn accounting_zero_observed_only_after_deadline_cannot_be_success() {
        let elapsed = Cell::new(Duration::ZERO);
        let mut sequence = VecDeque::from([1, 0]);
        let failure = drain_job(
            || {
                Ok((
                    stats(sequence.pop_front().expect("extra query"), elapsed.get()),
                    true,
                ))
            },
            || Duration::from_millis(15).checked_sub(elapsed.get()),
            |_| elapsed.set(Duration::from_millis(16)),
        )
        .unwrap_err();
        assert_eq!(failure.kind, FailureKind::Teardown);
        assert_eq!(failure.stats.unwrap().active_processes, 0);
        assert!(sequence.is_empty());
    }

    #[test]
    fn accounting_query_that_finishes_after_deadline_cannot_return_success() {
        let elapsed = Cell::new(Duration::ZERO);
        let mut first = true;
        let failure = drain_job(
            || {
                if std::mem::take(&mut first) {
                    Ok((stats(1, elapsed.get()), true))
                } else {
                    // Querying is also a boundary at which scheduling may
                    // consume the remaining deadline, before zero is observed.
                    elapsed.set(Duration::from_millis(16));
                    Ok((stats(0, elapsed.get()), true))
                }
            },
            || Duration::from_millis(15).checked_sub(elapsed.get()),
            |wait| elapsed.set(elapsed.get() + wait),
        )
        .unwrap_err();
        assert_eq!(failure.kind, FailureKind::Teardown);
        assert_eq!(failure.stats.unwrap().active_processes, 0);
    }

    #[test]
    fn query_failure_during_drain_preserves_error_instead_of_accepting_last_sample() {
        let mut first = true;
        let failure = drain_job(
            || {
                if std::mem::take(&mut first) {
                    Ok((stats(1, Duration::ZERO), true))
                } else {
                    Err(Failure::new(FailureKind::Io("accounting query".into())))
                }
            },
            || Some(Duration::from_millis(30)),
            |_| {},
        )
        .unwrap_err();
        assert_eq!(failure.kind, FailureKind::Io("accounting query".into()));
    }

    #[test]
    fn reader_not_ready_cannot_publish_zero_accounting_as_success() {
        let elapsed = Cell::new(Duration::ZERO);
        let failure = drain_job(
            || Ok((stats(0, elapsed.get()), false)),
            || Duration::from_millis(15).checked_sub(elapsed.get()),
            |wait| elapsed.set(elapsed.get() + wait),
        )
        .unwrap_err();
        assert_eq!(failure.kind, FailureKind::Teardown);
        assert!(failure.stats.unwrap().exited);
        assert_eq!(elapsed.get(), Duration::from_millis(15));
    }

    #[test]
    fn unsuccessful_stop_cannot_publish_unreaped_process_as_success() {
        let elapsed = Cell::new(Duration::from_millis(25));
        let failure = drain_job(
            || {
                let mut state = stats(0, elapsed.get());
                state.exited = false;
                Ok((state, true))
            },
            || Duration::from_millis(40).checked_sub(elapsed.get()),
            |wait| elapsed.set(elapsed.get() + wait),
        )
        .unwrap_err();
        assert_eq!(failure.kind, FailureKind::Teardown);
        assert!(!failure.stats.unwrap().exited);
        assert_eq!(elapsed.get(), Duration::from_millis(40));
    }

    #[test]
    fn controlled_open_empty_pipe_collection_is_bounded_by_the_same_deadline() {
        let (read, write) = pipe().unwrap();
        let mut capture = PipeCapture::new(read, 1024);
        let elapsed = Cell::new(Duration::ZERO);
        let failure = drain_job(
            || {
                assert!(!capture.pump());
                Ok((stats(0, elapsed.get()), capture.closed))
            },
            || Duration::from_millis(15).checked_sub(elapsed.get()),
            |wait| elapsed.set(elapsed.get() + wait),
        )
        .unwrap_err();
        assert_eq!(failure.kind, FailureKind::Teardown);
        assert_eq!(elapsed.get(), Duration::from_millis(15));
        assert!(capture.capture.bytes.is_empty());
        drop(write);
        assert!(capture.pump() && capture.closed);
        assert!(capture.capture.error.is_none());
    }

    #[test]
    fn exclusively_available_pipe_bytes_are_captured_without_waiting_for_writer_close() {
        let (read, write) = pipe().unwrap();
        let mut writer = write.file();
        writer.write_all(b"ready bytes").unwrap();
        let mut capture = PipeCapture::new(read, 1024);
        assert!(capture.pump());
        assert_eq!(capture.capture.bytes, b"ready bytes");
        assert!(!capture.pump() && !capture.closed);
        drop(writer);
        assert!(capture.pump() && capture.closed);
    }

    #[test]
    fn pipe_overflow_keeps_bounded_diagnostics_and_drains_after_writer_close() {
        let (read, write) = pipe().unwrap();
        let mut writer = write.file();
        writer.write_all(&[b'x'; 1024]).unwrap();
        let mut capture = PipeCapture::new(read, 512);
        assert!(capture.pump());
        assert_eq!(capture.capture.bytes, vec![b'x'; 512]);
        assert_eq!(capture.capture.error, Some(FailureKind::OutputBudget));
        drop(writer);
        assert!(capture.pump() && capture.closed);
        assert_eq!(capture.capture.bytes.len(), 512);
    }

    #[test]
    fn failed_stop_and_repeated_cleanup_cannot_refresh_the_absolute_deadline() {
        let start = Instant::now();
        let mut clock = TeardownClock {
            end: None,
            limit: Duration::from_millis(40),
        };
        let deadline = clock.begin(start);
        // A stop failure after consuming the budget re-enters cleanup. The
        // stored deadline stays expired instead of granting another 40ms.
        assert_eq!(clock.begin(start + Duration::from_millis(45)), deadline);
        assert!(clock.remaining(start + Duration::from_millis(45)).is_none());
        assert_eq!(
            clock.remaining(start + Duration::from_millis(25)),
            Some(Duration::from_millis(15))
        );
    }
}
