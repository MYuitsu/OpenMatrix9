// SPDX-License-Identifier: LGPL-2.1-or-later
//! Private native output. A write acknowledgement is not an export capability.
use crate::staged::native;
use crate::{
    ColdRetainedProof, ExpectedRetainedOutput, NativeExchange, RecordAction, StagedRetainedRequest,
};
use om9_core::identity::SourceStamp;
use om9_core::{
    Error, Result,
    budget::Usage,
    transaction::{Cancellation, TransactionCoordinator},
};
use om9_native_owner::{ProcessStats, VerifiedFile};
use serde::{Deserialize, Deserializer};
use sha2::{Digest, Sha256};
use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};
struct Hash32([u8; 32]);
impl<'de> Deserialize<'de> for Hash32 {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Hash32;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("canonical lowercase SHA256")
            }
            fn visit_str<E: serde::de::Error>(self, value: &str) -> std::result::Result<Hash32, E> {
                if value.len() != 64 {
                    return Err(E::custom("native SHA256 length"));
                }
                let nibble = |byte| match byte {
                    b'0'..=b'9' => Some(byte - b'0'),
                    b'a'..=b'f' => Some(byte - b'a' + 10),
                    _ => None,
                };
                let mut result = [0; 32];
                for (slot, pair) in result.iter_mut().zip(value.as_bytes().as_chunks::<2>().0) {
                    *slot = nibble(pair[0])
                        .and_then(|a| nibble(pair[1]).map(|b| (a << 4) | b))
                        .ok_or_else(|| E::custom("native SHA256 alphabet"))?;
                }
                Ok(Hash32(result))
            }
        }
        d.deserialize_str(Visitor)
    }
}
fn required_option<'de, D: Deserializer<'de>, T: Deserialize<'de>>(
    d: D,
) -> std::result::Result<Option<T>, D::Error> {
    Option::deserialize(d)
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WriteAck<'a> {
    schema_version: u32,
    #[serde(borrow)]
    writer_policy: &'a str,
    #[serde(borrow)]
    bounds_policy: &'a str,
    #[serde(borrow)]
    status: &'a str,
    archive_version: u32,
    objects: usize,
    definitions: usize,
    layers: usize,
    #[serde(deserialize_with = "required_option")]
    primary_source_sha256: Option<Hash32>,
    cold_validation_required: bool,
    admission_required: bool,
    host_ready: bool,
    request_sha256: Hash32,
    output_sha256: Hash32,
    output_bytes: u64,
}
fn fingerprint(file: &File, limit: u64, cancel: &Cancellation) -> Result<([u8; 32], u64)> {
    let io = |e: std::io::Error| Error::Io(e.to_string());
    let expected = file.metadata().map_err(io)?.len();
    if expected <= 32 || expected > limit {
        return Err(Error::Invalid("private retained output bytes"));
    }
    let mut reader = file.try_clone().map_err(io)?;
    reader.seek(SeekFrom::Start(0)).map_err(io)?;
    let mut hash = Sha256::new();
    let mut bytes = 0u64;
    let mut buffer = [0; 8192];
    loop {
        if cancel.is_canceled() {
            return Err(Error::Canceled);
        }
        let n = reader.read(&mut buffer).map_err(io)?;
        if n == 0 {
            break;
        }
        bytes = bytes
            .checked_add(n as u64)
            .ok_or(Error::Overflow("private retained output length"))?;
        if bytes > expected {
            return Err(Error::Invalid("growing retained output"));
        }
        hash.update(&buffer[..n]);
    }
    if bytes != expected {
        return Err(Error::Invalid("retained output length changed"));
    }
    Ok((hash.finalize().into(), bytes))
}
pub struct RetainedWriteReceipt {
    pub program_sha256: [u8; 32],
    pub request_sha256: [u8; 32],
    pub output_sha256: [u8; 32],
    pub output_bytes: u64,
    pub objects: usize,
    pub definitions: usize,
    pub layers: usize,
    pub native: ProcessStats,
}
pub struct PrivateRetainedFile<'a> {
    stage: &'a StagedRetainedRequest<'a>,
    held: File,
    receipt: RetainedWriteReceipt,
    max_bytes: u64,
    write_cancel: Cancellation,
}
impl PrivateRetainedFile<'_> {
    pub fn receipt(&self) -> &RetainedWriteReceipt {
        &self.receipt
    }
    pub fn output_path(&self) -> &std::path::Path {
        self.stage.output_path()
    }
    pub fn revalidate(&self, c: &TransactionCoordinator, cancel: &Cancellation) -> Result<()> {
        let joined = self.native_cancellation(cancel)?;
        self.stage.revalidate(c, &joined)?;
        if fingerprint(&self.held, self.max_bytes, &joined)?
            != (self.receipt.output_sha256, self.receipt.output_bytes)
        {
            return Err(Error::Invalid("held retained output changed"));
        }
        self.stage.revalidate(c, &joined)
    }
    fn native_cancellation(&self, cancel: &Cancellation) -> Result<Cancellation> {
        self.stage
            .native_cancellation(&self.write_cancel)?
            .either(cancel)
    }
}
/// Fresh-process correspondence bound to the same held private output and
/// immutable source/request owners. A detached proof alone cannot publish.
pub struct ValidatedRetainedFile<'a> {
    file: PrivateRetainedFile<'a>,
    proof: ColdRetainedProof,
    inspect: ProcessStats,
    inspect_cancel: Cancellation,
}
/// OM9 containers and runtime ownership are not serialized by this selected
/// 3DM writer. Original native data proven retained is separate from these
/// explicit losses; this receipt never certifies an OM9 native save.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetainedExportLosses {
    pub om9_state_not_encoded: bool,
    pub om9_history_recipes: usize,
    pub om9_results: usize,
    pub om9_resources: usize,
    pub omitted_native_records: usize,
}
pub struct RetainedExportReceipt {
    pub path: PathBuf,
    pub proof: ColdRetainedProof,
    pub losses: RetainedExportLosses,
    pub write: RetainedWriteReceipt,
    pub inspect: ProcessStats,
}
impl ValidatedRetainedFile<'_> {
    pub fn proof(&self) -> &ColdRetainedProof {
        &self.proof
    }
    pub fn write_receipt(&self) -> &RetainedWriteReceipt {
        &self.file.receipt
    }
    pub fn inspect_stats(&self) -> &ProcessStats {
        &self.inspect
    }
    pub fn output_path(&self) -> &std::path::Path {
        self.file.output_path()
    }
    pub fn revalidate(&self, c: &TransactionCoordinator, cancel: &Cancellation) -> Result<()> {
        let joined = self.native_cancellation(cancel)?;
        self.file.revalidate(c, &joined)
    }
    fn native_cancellation(&self, cancel: &Cancellation) -> Result<Cancellation> {
        self.file
            .native_cancellation(&self.inspect_cancel)?
            .either(cancel)
    }
    pub fn losses(&self) -> Result<RetainedExportLosses> {
        let joined = self.native_cancellation(&self.inspect_cancel)?;
        let inputs = self.file.stage.wire().inputs();
        let document = inputs.capture().document();
        let mut omitted = 0usize;
        for ns in &inputs.plan().overlay.namespaces {
            for row in &ns.records {
                inputs.check_cancel(&joined)?;
                if row.action == RecordAction::Omit {
                    omitted = omitted
                        .checked_add(1)
                        .ok_or(Error::Overflow("retained omitted record count"))?;
                }
            }
        }
        Ok(RetainedExportLosses {
            om9_state_not_encoded: true,
            om9_history_recipes: document.history.recipes.len(),
            om9_results: document.results.len(),
            om9_resources: document.resources.len(),
            omitted_native_records: omitted,
        })
    }
    /// Replace only in the canonical staging parent. Caller acknowledgement
    /// must exactly match the displayed losses. The accepted OS handle hashes
    /// and atomically renames the same bytes after all source checks pass.
    pub fn publish(
        self,
        c: &TransactionCoordinator,
        target: &Path,
        cancel: &Cancellation,
        acknowledged_losses: Option<RetainedExportLosses>,
        mut external: Usage,
    ) -> Result<RetainedExportReceipt> {
        let joined = self.native_cancellation(cancel)?;
        self.revalidate(c, &joined)?;
        let losses = self.losses()?;
        if acknowledged_losses != Some(losses) {
            return Err(Error::Permission("retained export losses not acknowledged"));
        }
        if target.as_os_str().len() > 8192 {
            return Err(Error::Budget("retained export target path"));
        }
        let stage = self.file.stage;
        let limits = c.limits();
        external.add(stage.owned_usage(), limits)?;
        external.add(stage.wire().owned_usage(), limits)?;
        external.add(stage.wire().inputs().owned_usage(), limits)?;
        external.add(
            Usage::array::<SourceStamp>(stage.wire().inputs().capture().sources.capacity())?,
            limits,
        )?;
        external.add(Usage::array::<ValidatedRetainedFile>(1)?, limits)?;
        external.add(Usage::array::<RetainedExportReceipt>(1)?, limits)?;
        external.add(Usage::array::<VerifiedFile>(1)?, limits)?;
        external.add(
            Usage {
                bytes: 128 * 1024,
                ..Usage::default()
            },
            limits,
        )?;
        c.retained_usage_with_held_usage(external)?;
        let filename = target
            .file_name()
            .ok_or(Error::Invalid("retained export filename"))?;
        let parent = target
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."))
            .canonicalize()
            .map_err(|e| Error::Io(e.to_string()))?;
        if parent.as_os_str().len() > 8192
            || stage.output_path().parent().and_then(Path::parent) != Some(parent.as_path())
        {
            return Err(Error::Invalid("retained export staging parent mismatch"));
        }
        let target = parent.join(filename);
        crate::staged::destination(&target)?;
        let Self {
            file,
            proof,
            inspect,
            ..
        } = self;
        let PrivateRetainedFile {
            stage,
            held,
            receipt,
            max_bytes,
            ..
        } = file;
        // Native CRT reads need the read-only guard. Final publication needs
        // a DELETE-capable guard instead; reacquire and prove the same bytes.
        drop(held);
        let verified =
            VerifiedFile::open(stage.output_path(), max_bytes, &joined).map_err(native)?;
        if (verified.fingerprint(), verified.len()) != (proof.output_sha256(), proof.output_bytes())
            || receipt.output_sha256 != proof.output_sha256()
            || receipt.request_sha256 != proof.request_sha256()
        {
            return Err(Error::Invalid("retained output changed before acceptance"));
        }
        stage.revalidate(c, &joined)?;
        crate::staged::destination(&target)?;
        // No allocation or fallible receipt construction after OS acceptance.
        let result = RetainedExportReceipt {
            path: target,
            proof,
            losses,
            write: receipt,
            inspect,
        };
        verified.publish(&result.path, &joined).map_err(native)?;
        Ok(result)
    }
}
impl NativeExchange {
    pub fn write_retained_private<'a>(
        &self,
        c: &TransactionCoordinator,
        stage: &'a StagedRetainedRequest<'a>,
        cancel: &Cancellation,
        mut external: Usage,
    ) -> Result<PrivateRetainedFile<'a>> {
        stage.revalidate(c, cancel)?;
        let joined = stage.native_cancellation(cancel)?;
        let limits = c.limits();
        external.add(stage.owned_usage(), limits)?;
        external.add(stage.wire().owned_usage(), limits)?;
        external.add(stage.wire().inputs().owned_usage(), limits)?;
        external.add(
            Usage::array::<SourceStamp>(stage.wire().inputs().capture().sources.capacity())?,
            limits,
        )?;
        external.add(Usage::array::<PrivateRetainedFile>(1)?, limits)?;
        external.add(
            Usage {
                bytes: self
                    .limits
                    .reservation_bytes()
                    .map_err(native)?
                    .checked_add(128 * 1024)
                    .ok_or(Error::Overflow("retained write reservation"))?,
                ..Usage::default()
            },
            limits,
        )?;
        let held = c.retained_usage_with_held_usage(external)?;
        let output = self
            .program
            .run(
                &[
                    "--write-retained".into(),
                    stage.request_path().as_os_str().to_owned(),
                    stage.output_path().as_os_str().to_owned(),
                ],
                &self.limits,
                &joined,
            )
            .map_err(native)?;
        stage.revalidate(c, cancel)?;
        if !output.stats.exited || output.stats.active_processes != 0 {
            return Err(Error::Invalid("retained writer owner not reaped"));
        }
        let remaining = limits.remaining_after(held)?;
        om9_archive::admit_json(&output.stdout, &remaining)?;
        let ack: WriteAck = serde_json::from_slice(&output.stdout)
            .map_err(|_| Error::Invalid("retained write acknowledgement schema"))?;
        let primary = stage
            .wire()
            .inputs()
            .plan()
            .primary_namespace
            .map(|ns| {
                stage
                    .wire()
                    .inputs()
                    .source(ns)
                    .map(|s| s.source_sha256())
                    .ok_or(Error::Missing("retained primary acknowledgement source"))
            })
            .transpose()?;
        if ack.schema_version != 4
            || ack.writer_policy != "om9-retained-current-v1"
            || ack.bounds_policy != "cold_leaf_hull_v1"
            || ack.status != "written_retained_current"
            || ack.archive_version != 50
            || !ack.cold_validation_required
            || !ack.admission_required
            || ack.host_ready
            || ack.request_sha256.0 != stage.request_sha256()
            || ack.primary_source_sha256.map(|h| h.0) != primary
            || ack.objects != stage.wire().table_order(12).len()
            || ack.definitions != stage.wire().table_order(11).len()
            || ack.layers != stage.wire().table_order(5).len()
        {
            return Err(Error::Invalid(
                "retained write acknowledgement correspondence",
            ));
        }
        use std::os::windows::fs::OpenOptionsExt;
        let held = OpenOptions::new()
            .read(true)
            .share_mode(1)
            .open(stage.output_path())
            .map_err(|e| Error::Io(e.to_string()))?;
        let max_bytes = limits.max_bytes.min(16 * 1024 * 1024);
        if fingerprint(&held, max_bytes, &joined)? != (ack.output_sha256.0, ack.output_bytes) {
            return Err(Error::Invalid("retained write held bytes correspondence"));
        }
        let result = PrivateRetainedFile {
            stage,
            held,
            max_bytes,
            write_cancel: cancel.clone(),
            receipt: RetainedWriteReceipt {
                program_sha256: self.program.fingerprint(),
                request_sha256: ack.request_sha256.0,
                output_sha256: ack.output_sha256.0,
                output_bytes: ack.output_bytes,
                objects: ack.objects,
                definitions: ack.definitions,
                layers: ack.layers,
                native: output.stats,
            },
        };
        result.revalidate(c, cancel)?;
        Ok(result)
    }
    pub fn inspect_retained_private<'a>(
        &self,
        c: &TransactionCoordinator,
        file: PrivateRetainedFile<'a>,
        cancel: &Cancellation,
        mut external: Usage,
    ) -> Result<ValidatedRetainedFile<'a>> {
        file.revalidate(c, cancel)?;
        if self.program.fingerprint() != file.receipt.program_sha256 {
            return Err(Error::Invalid("retained writer/inspector program mismatch"));
        }
        let joined = file.native_cancellation(cancel)?;
        let stage = file.stage;
        let limits = c.limits();
        external.add(stage.owned_usage(), limits)?;
        external.add(stage.wire().owned_usage(), limits)?;
        external.add(stage.wire().inputs().owned_usage(), limits)?;
        external.add(
            Usage::array::<SourceStamp>(stage.wire().inputs().capture().sources.capacity())?,
            limits,
        )?;
        // Account the consumed input and constructed result together, plus
        // arguments, bounded child output and stream-hash scratch, prelaunch.
        external.add(Usage::array::<PrivateRetainedFile>(1)?, limits)?;
        external.add(Usage::array::<ValidatedRetainedFile>(1)?, limits)?;
        external.add(
            Usage {
                bytes: self
                    .limits
                    .reservation_bytes()
                    .map_err(native)?
                    .checked_add(128 * 1024)
                    .ok_or(Error::Overflow("retained cold reservation"))?,
                ..Usage::default()
            },
            limits,
        )?;
        let held = c.retained_usage_with_held_usage(external)?;
        let output = self
            .program
            .run(
                &[
                    "--inspect-retained-current".into(),
                    stage.request_path().as_os_str().to_owned(),
                    stage.output_path().as_os_str().to_owned(),
                ],
                &self.limits,
                &joined,
            )
            .map_err(native)?;
        file.revalidate(c, cancel)?;
        if !output.stats.exited || output.stats.active_processes != 0 {
            return Err(Error::Invalid("retained inspector owner not reaped"));
        }
        let proof = crate::retained_cold::verify(
            stage.wire(),
            &output.stdout,
            &file.held,
            ExpectedRetainedOutput {
                request_sha256: file.receipt.request_sha256,
                output_sha256: file.receipt.output_sha256,
                output_bytes: file.receipt.output_bytes,
            },
            &limits.remaining_after(held)?,
            &joined,
        )?;
        file.revalidate(c, cancel)?;
        let result = ValidatedRetainedFile {
            file,
            proof,
            inspect: output.stats,
            inspect_cancel: cancel.clone(),
        };
        result.revalidate(c, cancel)?;
        Ok(result)
    }
}
