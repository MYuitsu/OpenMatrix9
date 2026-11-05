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
    io::{BufWriter, Read, Write},
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

const PACKET_BUFFER_BYTES: u64 = 64 * 1024;

struct StreamingPacketWriter<'a, W> {
    output: W,
    hash: Sha256,
    bytes: u64,
    limit: u64,
    cancel: &'a Cancellation,
    error: Option<Error>,
}
impl<W: Write> Write for StreamingPacketWriter<'_, W> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if self.cancel.is_canceled() {
            self.error = Some(Error::Canceled);
            return Err(std::io::Error::other("exchange packet canceled"));
        }
        let Some(next) = self.bytes.checked_add(bytes.len() as u64).filter(|n| *n <= self.limit) else {
            self.error = Some(Error::Budget("exchange packet bytes"));
            return Err(std::io::Error::other("exchange packet budget"));
        };
        self.output.write_all(bytes)?;
        self.hash.update(bytes);
        self.bytes = next;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> { self.output.flush() }
}

fn stream_packet<W: Write>(capture: &ExportCapture, output: W, limit: u64, cancel: &Cancellation)
    -> Result<(W, [u8; 32])> {
    let mut writer = StreamingPacketWriter { output, hash: Sha256::new(), bytes: 0, limit, cancel, error: None };
    if let Err(error) = serde_json::to_writer(&mut writer, &capture.packet()?) {
        return Err(writer.error.take().unwrap_or_else(|| Error::Io(error.to_string())));
    }
    writer.flush().map_err(io)?;
    Ok((writer.output, writer.hash.finalize().into()))
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
    pub brep_fingerprints: Option<ProcessStats>,
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
        capture.reserve_brep_projection(coordinator)?;
        capture.convert_breps(bridge, coordinator.limits())?;
        let mut held = coordinator.retained_usage()?;
        held.add(
            Usage::array::<SourceStamp>(capture.sources.len())?,
            coordinator.limits(),
        )?;
        if let Some(projected) = &capture.converted {
            held.add(projected.validate(coordinator.limits())?, coordinator.limits())?;
        }
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
        // The native process allowance must be visible to other operations on
        // this coordinator, not just subtracted from a local estimate. Keep it
        // through every native write/inspection call, including error paths.
        let mut auxiliary = Usage::array::<SourceStamp>(capture.sources.len())?;
        if let Some(projected) = &capture.converted && capture.projection_reservation.is_none() {
            let usage = projected.validate(coordinator.limits())?;
            auxiliary.add(Usage { bytes: usage.bytes, ..Usage::default() }, coordinator.limits())?;
        }
        let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
        let _native_reservation = coordinator.reserve_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::Kernel,
            self.limits.reservation_bytes().map_err(native)?, physical, auxiliary,
        ).map_err(|error| match error {
            Error::Budget("bytes") => Error::Budget("3DM native export reservation"),
            other => other,
        })?;
        let expected_needed = capture.document().entities.values()
            .chain(capture.document().definitions.values().flat_map(|d| &d.members))
            .any(|e| matches!(e.geometry, om9_core::geometry::Geometry::NativeBrep(_))
                && e.placement != om9_core::geometry::Transform::identity());
        let filename = path.file_name().ok_or(Error::Invalid("export filename"))?;
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let parent = parent.canonicalize().map_err(io)?;
        let target = parent.join(filename);
        destination(&target)?;
        let staging = Staging::new(&parent)?;
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(staging.packet())
            .map_err(io)?;
        let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
        let packet_buffer = coordinator.reserve_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::CpuCache, PACKET_BUFFER_BYTES, physical, auxiliary,
        )?;
        let buffered = BufWriter::with_capacity(PACKET_BUFFER_BYTES as usize, file);
        let (buffered, packet_hash) = stream_packet(&capture, buffered, packet_limit as u64, cancel)?;
        buffered.get_ref().sync_all().map_err(io)?;
        drop(buffered);
        drop(packet_buffer);
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
        let (expected_breps, brep_fingerprints) = if expected_needed {
            if fingerprint(&staging.packet(), packet_limit as u64, cancel)?.0 != packet_hash {
                return Err(Error::Stale("export packet changed"));
            }
            let expected = self.program.run(
                &["--expected-brep-fingerprints".into(), staging.packet().into_os_string()],
                &self.limits, cancel,
            ).map_err(native)?;
            if !expected.stats.exited || expected.stats.active_processes != 0 {
                return Err(Error::Invalid("native BRep expectation lifetime"));
            }
            if fingerprint(&staging.packet(), packet_limit as u64, cancel)?.0 != packet_hash {
                return Err(Error::Stale("export packet changed"));
            }
            (inspection::expected_breps(capture.document(), &expected.stdout, &response_limits)?, Some(expected.stats))
        } else {
            (inspection::ExpectedBreps::new(), None)
        };
        let mut inspect_limits = response_limits;
        if !expected_breps.is_empty() {
            let held = Usage::btree::<om9_core::identity::ObjectId, (u32, usize)>(expected_breps.len())?;
            inspect_limits.max_bytes = inspect_limits.max_bytes.checked_sub(held.bytes)
                .ok_or(Error::Budget("export BRep fingerprints"))?;
        }
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
        let mut diagnostics = inspection::verify_with_brep(capture.document(), &output.stdout, &inspect_limits, &expected_breps)?;
        let dropped_retained_sources = capture.document().retained_sources.len();
        let dropped_native_records: usize = capture.document().import_namespaces.values().map(|ns| ns.records.len()).sum();
        if dropped_retained_sources > 0 || dropped_native_records > 0 {
            diagnostics.push(format!(
                "EXPORT_RETAINED_OMITTED: {dropped_native_records} original native records and {dropped_retained_sources} retained sources were not written to the 3DM; original source bytes remain in the project"
            ));
        }
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
            dropped_retained_sources,
            dropped_native_records,
            diagnostics,
            dropped_om9_hidden_distinctions,
            dropped_om9_preset_slots,
            write,
            inspect: output.stats,
            brep_fingerprints,
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

#[cfg(test)]
mod streaming_packet_tests {
    use super::*;
    use om9_core::{geometry::{Cloud, Geometry}, model::{Document, Entity}, transaction::World};

    #[test]
    fn streamed_packet_matches_existing_encoding_and_checks_limit_and_cancel() {
        let limits = Limits::default();
        let mut document = Document::with_template_tolerance("streamed packet");
        let cloud = Entity::new("cloud", Geometry::Cloud(Cloud {
            points: vec![[1., 2., 3.]; 20_000], normals: vec![], colors: vec![], hidden: vec![],
        }), None);
        document.entities.insert(cloud.id, cloud);
        let id = document.id;
        let coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        let capture = ExportCapture::all(&coordinator, id).unwrap();
        let expected = capture.encode(&limits).unwrap();
        let cancel = Cancellation::default();
        let (streamed, digest) = stream_packet(&capture, Vec::new(), expected.len() as u64, &cancel).unwrap();
        assert_eq!(streamed, expected);
        assert_eq!(digest, <[u8; 32]>::from(Sha256::digest(&expected)));
        assert!(matches!(stream_packet(&capture, Vec::new(), expected.len() as u64 - 1, &cancel), Err(Error::Budget("exchange packet bytes"))));
        cancel.cancel();
        assert!(matches!(stream_packet(&capture, Vec::new(), expected.len() as u64, &cancel), Err(Error::Canceled)));
    }
}
