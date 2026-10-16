// SPDX-License-Identifier: LGPL-2.1-or-later
//! Dedicated native operation process ownership. Win32 handle code is isolated
//! here; model/scene/archive stay safe Rust. No document or native shape pointer
//! crosses this boundary. This is resource containment, not a security sandbox
//! or a certificate of native memory safety.
#![deny(unsafe_op_in_unsafe_fn)]
use om9_core::transaction::Cancellation;
use sha2::{Digest, Sha256};
use std::{
    ffi::OsString,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    time::Duration,
};
#[cfg(windows)]
pub mod dialog_probe;
#[cfg(windows)]
pub mod file_dialog;
mod verified_file;
#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::report_startup_failure;
/// Non-Windows builds have no owned window shell: the caller keeps the diagnostic
/// receipt and stderr text instead of claiming a dialog was shown.
#[cfg(not(windows))]
pub fn report_startup_failure(_title: &str, _message: &str) -> bool {
    false
}
pub use verified_file::VerifiedFile;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FailureKind {
    Invalid(&'static str),
    Fingerprint,
    Io(String),
    Canceled,
    Deadline,
    OutputBudget,
    NativeExit(u32),
    Teardown,
}
#[derive(Debug, Clone)]
pub struct ProcessStats {
    pub pid: u32,
    pub peak_committed_bytes: u64,
    pub active_processes: u32,
    pub exited: bool,
    pub elapsed: Duration,
}
#[derive(Debug)]
pub struct Failure {
    pub kind: FailureKind,
    pub stats: Option<ProcessStats>,
    pub stderr: Vec<u8>,
}
impl Failure {
    fn new(kind: FailureKind) -> Self {
        Self {
            kind,
            stats: None,
            stderr: vec![],
        }
    }
}
impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "native process: {:?}", self.kind)
    }
}
impl std::error::Error for Failure {}
pub type Result<T> = std::result::Result<T, Failure>;
fn io(error: std::io::Error) -> Failure {
    Failure::new(FailureKind::Io(error.to_string()))
}
#[derive(Debug, Clone)]
pub struct ProcessLimits {
    pub native_committed_bytes: u64,
    pub stdout_bytes: usize,
    pub stderr_bytes: usize,
    pub deadline: Duration,
    pub teardown: Duration,
}
impl Default for ProcessLimits {
    fn default() -> Self {
        Self {
            native_committed_bytes: 128 * 1024 * 1024,
            stdout_bytes: 16 * 1024 * 1024,
            stderr_bytes: 16 * 1024,
            deadline: Duration::from_secs(30),
            teardown: Duration::from_secs(3),
        }
    }
}
impl ProcessLimits {
    pub fn validate(&self) -> Result<()> {
        if self.native_committed_bytes == 0
            || usize::try_from(self.native_committed_bytes).is_err()
            || self.stdout_bytes == 0
            || self.stderr_bytes == 0
            || self.deadline.is_zero()
            || self.teardown.is_zero()
            || self.teardown.as_millis() > u32::MAX as u128
        {
            return Err(Failure::new(FailureKind::Invalid("process limits")));
        }
        self.reservation_bytes()?;
        Ok(())
    }
    /// Native private commit cap plus conservative parent capture buffers,
    /// reallocation overlap, pipe bookkeeping and platform overhead.
    /// Caller snapshots/packets and shared app resources are separate owners.
    pub fn reservation_bytes(&self) -> Result<u64> {
        (self.stdout_bytes as u64)
            .checked_add(self.stderr_bytes as u64)
            .and_then(|n| n.checked_mul(2))
            .and_then(|n| n.checked_add(2 * (64 * 1024 + 8192 + 8192)))
            .and_then(|n| n.checked_add(self.native_committed_bytes))
            .ok_or_else(|| Failure::new(FailureKind::Invalid("process reservation overflow")))
    }
}
#[derive(Debug)]
pub struct ProcessOutput {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub stats: ProcessStats,
}
pub struct NativeProgram {
    path: PathBuf,
    _image: File,
    sha256: [u8; 32],
}
impl NativeProgram {
    pub fn fingerprint(&self) -> [u8; 32] {
        self.sha256
    }
    /// Trusted application registry supplies the expected fingerprint; archive
    /// data never chooses an executable. Keep the admitted image open without
    /// write/delete sharing while this owner is alive.
    pub fn admit(path: &Path, expected: [u8; 32]) -> Result<Self> {
        if !path.is_absolute() {
            return Err(Failure::new(FailureKind::Invalid(
                "absolute native program path",
            )));
        }
        let path = path.canonicalize().map_err(io)?;
        let mut options = std::fs::OpenOptions::new();
        options.read(true);
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            options.share_mode(1);
        }
        let mut image = options.open(&path).map_err(io)?;
        let metadata = image.metadata().map_err(io)?;
        if !metadata.is_file() || metadata.len() > 128 * 1024 * 1024 {
            return Err(Failure::new(FailureKind::Invalid("native executable size")));
        }
        let mut digest = Sha256::new();
        let mut buffer = [0; 8192];
        let mut bytes = 0u64;
        loop {
            let n = image.read(&mut buffer).map_err(io)?;
            if n == 0 {
                break;
            }
            bytes = bytes
                .checked_add(n as u64)
                .ok_or_else(|| Failure::new(FailureKind::Invalid("native executable size")))?;
            if bytes > 128 * 1024 * 1024 {
                return Err(Failure::new(FailureKind::Invalid("native executable size")));
            }
            digest.update(&buffer[..n]);
        }
        let sha256: [u8; 32] = digest.finalize().into();
        if sha256 != expected {
            return Err(Failure::new(FailureKind::Fingerprint));
        }
        Ok(Self {
            path,
            _image: image,
            sha256,
        })
    }
    pub fn run(
        &self,
        args: &[OsString],
        limits: &ProcessLimits,
        cancel: &Cancellation,
    ) -> Result<ProcessOutput> {
        limits.validate()?;
        if cancel.is_canceled() {
            return Err(Failure::new(FailureKind::Canceled));
        }
        #[cfg(windows)]
        {
            windows::run(&self.path, args, limits, cancel)
        }
        #[cfg(not(windows))]
        {
            let _ = (args, limits, cancel);
            Err(Failure::new(FailureKind::Invalid(
                "native process containment requires Windows",
            )))
        }
    }
}
