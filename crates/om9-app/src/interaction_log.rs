// SPDX-License-Identifier: LGPL-2.1-or-later
//! Opt-in interaction log for viewport performance work.
//!
//! `OM9_INTERACTION_LOG=<path>` appends one JSON object per line for the camera gestures (orbit, pan,
//! zoom) and for each gumball move/rotate/scale step: the frame time seen by the UI, the camera identity
//! and the scene counters (objects, visible, culled, draw calls). The file is written only when the
//! variable names a path, so a normal session pays nothing.
//!
//! The log is diagnostic evidence, not document data: it never enters a project file and a write error
//! never interrupts the gesture.

use serde_json::{Value, json};
use std::{
    fs::OpenOptions,
    io::{BufWriter, Read, Write},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
        mpsc::{self, SyncSender},
    },
    time::Instant,
};

pub struct InteractionLog {
    sender: SyncSender<LogMessage>,
    dropped: Arc<AtomicU64>,
    path: PathBuf,
    started: Instant,
    worker: Option<std::thread::JoinHandle<()>>,
    session: String,
}
enum LogMessage {
    Line(Vec<u8>),
    Flush(mpsc::Sender<()>),
    Stop,
}

impl InteractionLog {
    /// Opens the log named by `OM9_INTERACTION_LOG`, creating its directory when needed.
    pub fn from_env() -> Option<Self> {
        let path = PathBuf::from(std::env::var_os("OM9_INTERACTION_LOG")?);
        Self::open(path)
    }
    pub fn open(path: PathBuf) -> Option<Self> {
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            let _ = std::fs::create_dir_all(parent);
        }
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .ok()?;
        let (sender, receiver) = mpsc::sync_channel(4096);
        let dropped = Arc::new(AtomicU64::new(0));
        let lost = dropped.clone();
        let worker = std::thread::Builder::new()
            .name("om9-diagnostics".into())
            .spawn(move || {
                let mut writer = BufWriter::with_capacity(64 * 1024, file);
                let mut flushed = Instant::now();
                loop {
                    let message = match receiver.recv_timeout(std::time::Duration::from_millis(100))
                    {
                        Ok(message) => message,
                        Err(mpsc::RecvTimeoutError::Timeout) => {
                            let _ = writer.flush();
                            continue;
                        }
                        Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    };
                    match message {
                        LogMessage::Line(bytes) => {
                            if writer.write_all(&bytes).is_err() {
                                lost.fetch_add(1, Ordering::Relaxed);
                            }
                        }
                        LogMessage::Flush(done) => {
                            let _ = writer.flush();
                            let _ = done.send(());
                        }
                        LogMessage::Stop => break,
                    }
                    if flushed.elapsed() >= std::time::Duration::from_millis(100) {
                        let _ = writer.flush();
                        flushed = Instant::now();
                    }
                }
                let _ = writeln!(
                    writer,
                    "{}",
                    json!({"event":"diagnostic_end","dropped_records":lost.load(Ordering::Relaxed)})
                );
                let _ = writer.flush();
            })
            .ok()?;
        let session = format!(
            "{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .ok()?
                .as_nanos()
        );
        let mut log = Self {
            sender,
            dropped,
            path,
            started: Instant::now(),
            worker: Some(worker),
            session,
        };
        use sha2::{Digest, Sha256};
        let mut source = Sha256::new();
        for bytes in [
            include_str!("navigation.rs"),
            include_str!("frame_scheduler.rs"),
            include_str!("ui.rs"),
            include_str!("gpu_viewport.rs"),
        ] {
            source.update(bytes.as_bytes());
        }
        let executable = std::env::current_exe().ok().and_then(|path| {
            let mut file = std::fs::File::open(path).ok()?;
            let mut hash = Sha256::new();
            let mut buffer = [0u8; 65536];
            loop {
                let n = file.read(&mut buffer).ok()?;
                if n == 0 {
                    break;
                }
                hash.update(&buffer[..n]);
            }
            Some(format!("{:x}", hash.finalize()))
        });
        log.record(json!({"event":"diagnostic_start","schema_version":2,"executable_sha256":executable,"source_fingerprint":format!("{:x}",source.finalize()),"source_scope":"navigation/frame_scheduler/ui/gpu_viewport","version":env!("CARGO_PKG_VERSION"),"dropped_records":0}));
        Some(log)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Appends one line and stamps it with elapsed milliseconds since the log was opened.
    pub fn record(&mut self, mut value: Value) {
        if let Some(object) = value.as_object_mut() {
            object.insert("t_ms".into(), json!(self.started.elapsed().as_millis()));
            object.insert("session_id".into(), json!(self.session));
            object.insert(
                "wall_unix_ns".into(),
                json!(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_nanos())
                        .unwrap_or_default()
                        .to_string()
                ),
            );
            object.insert("dropped_records".into(), json!(self.dropped_records()));
        }
        if let Ok(mut bytes) = serde_json::to_vec(&value) {
            bytes.push(b'\n');
            if self.sender.try_send(LogMessage::Line(bytes)).is_err() {
                self.dropped.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
    pub fn dropped_records(&self) -> u64 {
        self.dropped.load(Ordering::Relaxed)
    }
    /// Explicit diagnostic barrier; never called on the interactive frame path.
    pub fn flush(&self) {
        let (tx, rx) = mpsc::channel();
        if self.sender.send(LogMessage::Flush(tx)).is_ok() {
            let _ = rx.recv();
        }
    }
}
impl Drop for InteractionLog {
    fn drop(&mut self) {
        let _ = self.sender.send(LogMessage::Stop);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

/// A frame time in milliseconds as egui saw it, clamped to a finite positive value.
pub fn frame_ms(stable_dt: f32) -> f64 {
    if !stable_dt.is_finite() || stable_dt <= 0. {
        return 0.001;
    }
    (stable_dt as f64 * 1000.0).clamp(0.001, 3_600_000.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_frame_time_is_finite_and_positive() {
        assert_eq!(frame_ms(0.0), 0.001);
        assert!(frame_ms(f32::NAN).is_finite());
        assert!(frame_ms(f32::INFINITY).is_finite());
        assert!((frame_ms(0.016) - 16.0).abs() < 0.01);
    }

    #[test]
    fn records_are_json_lines_with_a_timestamp() {
        let directory =
            std::env::temp_dir().join(format!("om9-interaction-log-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).expect("log directory");
        let path = directory.join("interaction.jsonl");
        let mut log = InteractionLog::open(path.clone()).expect("log file");
        log.record(json!({"event": "camera", "action": "orbit", "frame_ms": 16.5}));
        log.flush();
        let line = std::fs::read_to_string(&path).expect("written line");
        let value: Value = serde_json::from_str(line.lines().last().unwrap()).expect("json line");
        assert_eq!(value["event"], "camera");
        assert_eq!(value["action"], "orbit");
        assert!(value["t_ms"].is_u64());
        let _ = std::fs::remove_dir_all(&directory);
    }
}
