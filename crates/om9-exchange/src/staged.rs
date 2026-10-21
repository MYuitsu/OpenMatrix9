// SPDX-License-Identifier: LGPL-2.1-or-later
use crate::{ExportCapture, inspection};
use om9_core::{
    Error, Result,
    budget::{Limits, Usage},
    identity::SourceStamp,
    transaction::{Cancellation, TransactionCoordinator},
};
use om9_native_owner::{FailureKind, NativeProgram, ProcessLimits, ProcessStats, VerifiedFile};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

fn io(error: std::io::Error) -> Error {
    Error::Io(error.to_string())
}
fn canceled(token: &Cancellation) -> Result<()> {
    if token.is_canceled() {
        Err(Error::Canceled)
    } else {
        Ok(())
    }
}
pub(crate) fn native(error: om9_native_owner::Failure) -> Error {
    if error.kind == FailureKind::Canceled {
        Error::Canceled
    } else {
        Error::Io(format!(
            "{error}: {}",
            String::from_utf8_lossy(&error.stderr)
        ))
    }
}
pub(crate) fn destination(path: &Path) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(m) if !m.is_file() || m.permissions().readonly() => {
            Err(Error::Permission("export destination"))
        }
        Ok(_) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(io(e)),
    }
}
fn fingerprint(path: &Path, max_bytes: u64, cancel: &Cancellation) -> Result<([u8; 32], u64)> {
    let m = fs::symlink_metadata(path).map_err(io)?;
    if !m.is_file() || m.len() == 0 || m.len() > max_bytes {
        return Err(Error::Invalid("staged 3DM file"));
    }
    let mut f = File::open(path).map_err(io)?;
    let mut buffer = [0; 8192];
    let mut hash = Sha256::new();
    let mut size = 0u64;
    loop {
        canceled(cancel)?;
        let n = f.read(&mut buffer).map_err(io)?;
        if n == 0 {
            break;
        }
        size = size
            .checked_add(n as u64)
            .ok_or(Error::Overflow("export file bytes"))?;
        if size > max_bytes {
            return Err(Error::Budget("growing export file"));
        }
        hash.update(&buffer[..n]);
    }
    if size != m.len() {
        return Err(Error::Invalid("changing export file"));
    }
    Ok((hash.finalize().into(), size))
}
struct Staging(PathBuf);
impl Staging {
    fn new(parent: &Path) -> Result<Self> {
        let path = parent.join(format!(".om9-export-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&path).map_err(io)?;
        Ok(Self(path))
    }
    fn packet(&self) -> PathBuf {
        self.0.join("request.json")
    }
    fn file(&self) -> PathBuf {
        self.0.join("current.3dm")
    }
}
impl Drop for Staging {
    fn drop(&mut self) {
        let _ = fs::remove_file(self.packet());
        let _ = fs::remove_file(self.file());
        let _ = fs::remove_dir(&self.0);
    }
}
#[derive(Debug)]
pub struct ExportReceipt {
    pub path: PathBuf,
    pub sha256: [u8; 32],
    pub bytes: u64,
    pub root_objects: usize,
    pub definition_members: usize,
    pub dropped_history_recipes: usize,
    pub dropped_results: usize,
    pub dropped_resources: usize,
    pub dropped_retained_sources: usize,
    pub dropped_native_records: usize,
    pub dropped_om9_hidden_distinctions: usize,
    pub dropped_om9_preset_slots: usize,
    pub diagnostics: Vec<String>,
    pub write: ProcessStats,
    pub inspect: ProcessStats,
}
/// Trusted application code admits the binary. Serialized documents cannot
/// supply its executable path, digest or process limits.
pub struct NativeExchange {
    pub(crate) program: NativeProgram,
    pub(crate) limits: ProcessLimits,
}
pub struct ValidatedExport {
    capture: ExportCapture,
    staging: Staging,
    receipt: ExportReceipt,
    max_file_bytes: u64,
    cancel: Cancellation,
}
impl NativeExchange {
    pub fn new(program: NativeProgram, limits: ProcessLimits) -> Result<Self> {
        limits.validate().map_err(native)?;
        Ok(Self { program, limits })
    }
    pub fn prepare(
        &self,
        coordinator: &TransactionCoordinator,
        capture: ExportCapture,
        path: &Path,
        cancel: &Cancellation,
    ) -> Result<ValidatedExport> {
        self.prepare_with_brep(coordinator, capture, path, cancel, None)
    }
    pub fn prepare_with_brep(
        &self,
        coordinator: &TransactionCoordinator,
        mut capture: ExportCapture,
        path: &Path,
        cancel: &Cancellation,
        bridge: Option<&dyn crate::BrepExport>,
    ) -> Result<ValidatedExport> {
        canceled(cancel)?;
        capture.revalidate(coordinator)?;
        capture.convert_breps(bridge, coordinator.limits())?;
        let mut held = coordinator.retained_usage()?;
        held.add(
            Usage::array::<SourceStamp>(capture.sources.len())?,
            coordinator.limits(),
        )?;
        let base = held;
        held.add(
            Usage {
                bytes: self.limits.reservation_bytes().map_err(native)?,
                ..Usage::default()
            },
            coordinator.limits(),
        )?;
        let available = coordinator
            .limits()
            .max_bytes
            .checked_sub(held.bytes)
            .ok_or(Error::Budget("export operation aggregate"))?;
        let packet_limit = usize::try_from((available / 2).min(16 * 1024 * 1024))
            .map_err(|_| Error::Overflow("export packet address space"))?;
        // Account both old and new buffer storage before every serialization
        // growth. Native geometry/DOM/temporary allocations share its Job cap.
        let packet = capture.encode_up_to(packet_limit)?;
        let filename = path.file_name().ok_or(Error::Invalid("export filename"))?;
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let parent = parent.canonicalize().map_err(io)?;
        let target = parent.join(filename);
        destination(&target)?;
        let staging = Staging::new(&parent)?;
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(staging.packet())
            .map_err(io)?;
        file.write_all(&packet).map_err(io)?;
        file.flush().map_err(io)?;
        file.sync_all().map_err(io)?;
        drop(file);
        drop(packet);
        canceled(cancel)?;
        let l = coordinator.limits();
        let remaining = |max: u64, used: u64| {
            max.checked_sub(used)
                .ok_or(Error::Budget("export response aggregate"))
        };
        let response_limits = Limits {
            max_bytes: remaining(l.max_bytes, base.bytes)?,
            max_points: remaining(l.max_points, base.points)?,
            max_objects: remaining(l.max_objects, base.objects)?,
            max_nodes: remaining(l.max_nodes, base.nodes)?,
            max_history: remaining(l.max_history, base.history)?,
            max_outputs: remaining(l.max_outputs, base.outputs)?,
            ..*l
        };
        let output = self
            .program
            .run(
                &[
                    "--write".into(),
                    staging.packet().into_os_string(),
                    staging.file().into_os_string(),
                ],
                &self.limits,
                cancel,
            )
            .map_err(native)?;
        om9_archive::admit_json(&output.stdout, &response_limits)?;
        let response: Value = serde_json::from_slice(&output.stdout)
            .map_err(|_| Error::Invalid("native write response"))?;
        let expected = capture.document().entities.len()
            + capture
                .document()
                .definitions
                .values()
                .map(|d| d.members.len())
                .sum::<usize>();
        if response["schema_version"] != 1
            || response["status"] != "written"
            || response["objects"].as_u64() != Some(expected as u64)
            || !output.stats.exited
            || output.stats.active_processes != 0
        {
            return Err(Error::Invalid("native write acknowledgement"));
        }
        let write = output.stats;
        drop(response);
        drop(output.stdout);
        drop(output.stderr);
        fs::remove_file(staging.packet()).map_err(io)?;
        // Sync the native output before its cold read. Hash the actual file on
        // both sides of inspection; never validate cached input geometry.
        OpenOptions::new()
            .write(true)
            .open(staging.file())
            .map_err(io)?
            .sync_all()
            .map_err(io)?;
        let max_file_bytes = coordinator.limits().max_bytes.min(16 * 1024 * 1024);
        let (sha256, bytes) = fingerprint(&staging.file(), max_file_bytes, cancel)?;
        let output = self
            .program
            .run(
                &["--inspect".into(), staging.file().into_os_string()],
                &self.limits,
                cancel,
            )
            .map_err(native)?;
        if !output.stats.exited || output.stats.active_processes != 0 {
            return Err(Error::Invalid("native inspect lifetime"));
        }
        let diagnostics = inspection::verify(capture.document(), &output.stdout, &response_limits)?;
        let dropped_om9_hidden_distinctions = capture
            .document
            .entities
            .values()
            .chain(
                capture
                    .document
                    .definitions
                    .values()
                    .flat_map(|d| &d.members),
            )
            .filter(|e| e.attributes.hidden)
            .count();
        let dropped_om9_preset_slots = capture
            .document
            .layers
            .values()
            .filter(|l| l.preset_slot.is_some())
            .count();
        if fingerprint(&staging.file(), max_file_bytes, cancel)? != (sha256, bytes) {
            return Err(Error::Invalid("native inspection changed file"));
        }
        capture.revalidate(coordinator)?;
        canceled(cancel)?;
        let receipt = ExportReceipt {
            path: target,
            sha256,
            bytes,
            root_objects: capture.document().entities.len(),
            definition_members: expected - capture.document().entities.len(),
            dropped_history_recipes: capture.document().history.recipes.len(),
            dropped_results: capture.document().results.len(),
            dropped_resources: capture.document().resources.len(),
            dropped_retained_sources: capture.document().retained_sources.len(),
            dropped_native_records: capture
                .document()
                .import_namespaces
                .values()
                .map(|ns| ns.records.len())
                .sum(),
            diagnostics,
            dropped_om9_hidden_distinctions,
            dropped_om9_preset_slots,
            write,
            inspect: output.stats,
        };
        Ok(ValidatedExport {
            capture,
            staging,
            receipt,
            max_file_bytes,
            cancel: cancel.clone(),
        })
    }
}
impl ValidatedExport {
    pub fn receipt(&self) -> &ExportReceipt {
        &self.receipt
    }
    /// Acceptance runs on the owning coordinator thread. A source change,
    /// close, Undo, cancellation or changed staged file rejects before rename.
    /// Export does not mutate the document, selection or Undo/Redo stacks.
    pub fn publish(
        self,
        coordinator: &TransactionCoordinator,
        cancel: &Cancellation,
    ) -> Result<ExportReceipt> {
        let combined = self.cancel.either(cancel)?;
        let cancel = &combined;
        self.capture.revalidate(coordinator)?;
        canceled(cancel)?;
        destination(&self.receipt.path)?;
        let verified = VerifiedFile::open(&self.staging.file(), self.max_file_bytes, cancel)
            .map_err(native)?;
        if (verified.fingerprint(), verified.len()) != (self.receipt.sha256, self.receipt.bytes) {
            return Err(Error::Invalid("staged export changed before acceptance"));
        }
        self.capture.revalidate(coordinator)?;
        canceled(cancel)?;
        // The same guarded handle supplies the fingerprint and performs the
        // atomic rename; ordinary writes/deletes/replacement cannot race this
        // final acceptance window.
        verified
            .publish(&self.receipt.path, cancel)
            .map_err(native)?;
        Ok(self.receipt)
    }
}
