// SPDX-License-Identifier: LGPL-2.1-or-later
//! Owned-process source preparation. This does not write or publish a 3DM.
use crate::{
    ExportCapture, NativeExchange, OutputPlan, SourcePreparation, current_overlay::canceled,
    staged::native,
};
use om9_core::{
    Error, Result,
    budget::Usage,
    identity::{ImportNamespaceId, SourceStamp},
    transaction::{Cancellation, TransactionCoordinator},
};
use om9_native_owner::ProcessStats;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

pub struct PreparedMergeInputs<'a> {
    capture: &'a ExportCapture,
    cancel: Cancellation,
    plan: OutputPlan<'a>,
    sources: BTreeMap<ImportNamespaceId, SourcePreparation>,
    native: BTreeMap<ImportNamespaceId, ProcessStats>,
    owned_usage: Usage,
}
impl PreparedMergeInputs<'_> {
    pub(crate) fn capture(&self) -> &ExportCapture {
        self.capture
    }
    pub fn plan(&self) -> &OutputPlan<'_> {
        &self.plan
    }
    pub fn source(&self, id: ImportNamespaceId) -> Option<&SourcePreparation> {
        self.sources.get(&id)
    }
    pub fn native_stats(&self) -> &BTreeMap<ImportNamespaceId, ProcessStats> {
        &self.native
    }
    pub fn owned_usage(&self) -> Usage {
        self.owned_usage
    }
    pub fn revalidate(&self, c: &TransactionCoordinator, cancel: &Cancellation) -> Result<()> {
        self.check_cancel(cancel)?;
        self.capture.revalidate(c)
    }
    pub(crate) fn check_cancel(&self, cancel: &Cancellation) -> Result<()> {
        canceled(&self.cancel)?;
        canceled(cancel)
    }
    pub(crate) fn native_cancellation(&self, cancel: &Cancellation) -> Result<Cancellation> {
        self.check_cancel(cancel)?;
        self.cancel.either(cancel)
    }
    /// Extract original-wire data for diagnostics. Consuming the prepared
    /// operation also drops its current-state intent and publication authority.
    pub fn into_source(mut self, id: ImportNamespaceId) -> Option<SourcePreparation> {
        self.sources.remove(&id)
    }
}
fn io(e: std::io::Error) -> Error {
    Error::Io(e.to_string())
}
struct SourceStage {
    dir: PathBuf,
    path: PathBuf,
    held: Option<File>,
}
impl SourceStage {
    fn new(parent: &Path, bytes: &[u8], hash: [u8; 32], cancel: &Cancellation) -> Result<Self> {
        canceled(cancel)?;
        let dir = parent.join(format!(".om9-merge-source-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&dir).map_err(io)?;
        let path = dir.join("original.3dm");
        let mut stage = Self {
            dir,
            path,
            held: None,
        };
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&stage.path)
            .map_err(io)?;
        for block in bytes.chunks(8192) {
            canceled(cancel)?;
            file.write_all(block).map_err(io)?;
        }
        file.flush().map_err(io)?;
        file.sync_all().map_err(io)?;
        drop(file);
        let mut options = OpenOptions::new();
        options.read(true);
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            options.share_mode(1);
        }
        // Preparation needs a read owner. The publication guard requests
        // DELETE access for atomic rename and cannot share the CRT reader.
        let mut held = options.open(&stage.path).map_err(io)?;
        let mut digest = Sha256::new();
        let mut buffer = [0; 8192];
        let mut count = 0usize;
        loop {
            canceled(cancel)?;
            let n = held.read(&mut buffer).map_err(io)?;
            if n == 0 {
                break;
            }
            count = count
                .checked_add(n)
                .ok_or(Error::Overflow("merge staging bytes"))?;
            if count > bytes.len() {
                return Err(Error::Invalid("growing merge source staging"));
            }
            digest.update(&buffer[..n]);
        }
        if count != bytes.len() || <[u8; 32]>::from(digest.finalize()) != hash {
            return Err(Error::Invalid("merge source staging digest"));
        }
        stage.held = Some(held);
        Ok(stage)
    }
}
impl Drop for SourceStage {
    fn drop(&mut self) {
        self.held.take();
        let _ = fs::remove_file(&self.path);
        let _ = fs::remove_dir(&self.dir);
    }
}
impl NativeExchange {
    pub fn prepare_merge_inputs<'a>(
        &self,
        coordinator: &TransactionCoordinator,
        capture: &'a ExportCapture,
        primary: Option<ImportNamespaceId>,
        staging_parent: &Path,
        cancel: &Cancellation,
        mut external: Usage,
    ) -> Result<PreparedMergeInputs<'a>> {
        canceled(cancel)?;
        capture.revalidate(coordinator)?;
        let l = coordinator.limits();
        if staging_parent.as_os_str().len() > 8192 {
            return Err(Error::Budget("merge staging path bytes"));
        }
        let count = capture.document().import_namespaces.len();
        let mut owned = Usage::array::<PreparedMergeInputs>(1)?;
        // Cancellation can hold four immutable shared flags. Retain their
        // control blocks even when a later caller supplies another token.
        owned.add(Usage::array::<usize>(12)?, l)?;
        owned.add(
            Usage::btree::<ImportNamespaceId, SourcePreparation>(count)?,
            l,
        )?;
        owned.add(Usage::btree::<ImportNamespaceId, ProcessStats>(count)?, l)?;
        external.add(owned, l)?;
        // Reserve process, stdout/stderr and native heap before any document
        // validation scratch. The per-source file writes borrow immutable bytes.
        let mut running = Usage {
            bytes: self.limits.reservation_bytes().map_err(native)?,
            ..Usage::default()
        };
        running.add(
            Usage {
                // Parent, directory, file and process-argument paths can each
                // retain a geometrically grown buffer near the 8192-byte cap.
                // Admit old/new path copies and the bounded hashing buffer.
                bytes: 128 * 1024,
                ..Usage::default()
            },
            l,
        )?;
        external.add(running, l)?;
        let plan = capture.current_output_plan(coordinator, primary, cancel, external)?;
        owned.add(plan.owned_usage, l)?;
        // owned maps were included in external above; add only the new plan.
        external.add(plan.owned_usage, l)?;
        external.add(Usage::array::<SourceStamp>(capture.sources.capacity())?, l)?;
        let mut held = coordinator.retained_usage_with_held_usage(external)?;
        let parent = staging_parent.canonicalize().map_err(io)?;
        if parent.as_os_str().len() > 8192 || !parent.is_dir() {
            return Err(Error::Invalid("merge staging parent"));
        }
        let mut sources = BTreeMap::new();
        let mut stats = BTreeMap::new();
        for namespace in &plan.overlay.namespaces {
            canceled(cancel)?;
            capture.revalidate(coordinator)?;
            let resource = capture
                .document()
                .resources
                .get(&namespace.source.resource)
                .ok_or(Error::Missing("merge source resource"))?;
            let bytes = resource.payload()?;
            if resource.sha256() != namespace.source.sha256 {
                return Err(Error::Invalid("merge original resource identity"));
            }
            let stage = SourceStage::new(&parent, bytes, namespace.source.sha256, cancel)?;
            let output = self
                .program
                .run(
                    &[
                        "--prepare-retained-source".into(),
                        stage.path.as_os_str().to_owned(),
                    ],
                    &self.limits,
                    cancel,
                )
                .map_err(native)?;
            if !output.stats.exited || output.stats.active_processes != 0 {
                return Err(Error::Invalid("merge preparation owner not reaped"));
            }
            // The running reservation already charges stdout/stderr capacities.
            // Keep it charged during typed JSON and lookup allocations too.
            let prepared = SourcePreparation::decode(
                &output.stdout,
                namespace.namespace,
                namespace.source,
                bytes,
                &l.remaining_after(held)?,
                cancel,
            )?;
            owned.add(prepared.owned_usage(), l)?;
            held.add(prepared.owned_usage(), l)?;
            sources.insert(namespace.namespace.id, prepared);
            stats.insert(namespace.namespace.id, output.stats);
            drop(output.stdout);
            drop(output.stderr);
            drop(stage);
        }
        canceled(cancel)?;
        capture.revalidate(coordinator)?;
        Ok(PreparedMergeInputs {
            capture,
            cancel: cancel.clone(),
            plan,
            sources,
            native: stats,
            owned_usage: owned,
        })
    }
}
