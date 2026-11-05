// SPDX-License-Identifier: LGPL-2.1-or-later
//! Versioned standalone OM9 persistence.
#![forbid(unsafe_code)]
mod atomic_file;
mod container;
mod json_admission;
mod restore;
pub mod snapshot;
pub use restore::{ReplayKey, RestoredDocument};
pub use snapshot::DocumentSnapshot;

/// Structural admission for bounded JSON exchanged with an operation process.
/// Checks duplicate keys, strings, depth and estimated typed allocations before
/// a caller allocates a DOM or typed response. This does not activate a runtime.
pub fn admit_json(bytes: &[u8], limits: &om9_core::budget::Limits) -> om9_core::Result<()> {
    json_admission::check(bytes, bytes.len(), limits)
}
/// Conservative peak for the admitted JSON input/DOM, usable while a caller
/// reserves additional owned response strings. This is admission accounting.
pub fn admit_json_usage(
    bytes: &[u8],
    limits: &om9_core::budget::Limits,
) -> om9_core::Result<om9_core::budget::Usage> {
    json_admission::check_usage(bytes, bytes.len(), limits)
}
/// Admit external JSON incrementally before preflight allocations. The normal
/// external node/depth limits remain in force; the callback receives the
/// monotonically increasing estimated peak in bytes.
pub fn admit_json_usage_with_admission(
    bytes: &[u8],
    limits: &om9_core::budget::Limits,
    mut admit: impl FnMut(u64) -> om9_core::Result<()>,
) -> om9_core::Result<om9_core::budget::Usage> {
    json_admission::check_external_with_admission(bytes, limits, &mut admit)
}

use om9_core::{Result, budget::Limits, model::Document, transaction::Cancellation};
use std::path::Path;

pub struct Archive;
pub struct Migration {
    pub from_version: u32,
    pub to_version: u32,
    pub bytes: Vec<u8>,
}
impl Archive {
    pub fn encode(document: &Document, limits: &Limits) -> Result<Vec<u8>> {
        container::encode(document, limits)
    }
    pub fn decode(bytes: &[u8], limits: &Limits) -> Result<Document> {
        container::decode(bytes, limits)
    }
    /// Explicit schema-1 upgrade. This establishes empty resource ownership,
    /// never invented source provenance; durable document/object IDs stay put.
    pub fn migrate_from_v1(bytes: &[u8], limits: &Limits) -> Result<Migration> {
        let document = container::decode_legacy_v1(bytes, limits)?;
        Self::migration(bytes, limits, document, 1)
    }
    /// Schema 2 retains embedded bytes/IDs but has no source namespaces. The
    /// migration creates no provenance merely from a resource kind declaration.
    pub fn migrate_from_v2(bytes: &[u8], limits: &Limits) -> Result<Migration> {
        let document = container::decode_legacy_v2(bytes, limits)?;
        Self::migration(bytes, limits, document, 2)
    }
    /// Schema3 has no preset-slot authority or independent object Hidden.
    /// Preserve IDs/bytes and never infer a slot from a label/order.
    pub fn migrate_from_v3(bytes: &[u8], limits: &Limits) -> Result<Migration> {
        let document = container::decode_legacy_v3(bytes, limits)?;
        Self::migration(bytes, limits, document, 3)
    }
    fn migration(
        bytes: &[u8],
        limits: &Limits,
        document: Document,
        from_version: u32,
    ) -> Result<Migration> {
        let mut encoding = *limits;
        encoding.max_bytes = limits
            .max_bytes
            .checked_sub(bytes.len() as u64)
            .ok_or(om9_core::Error::Budget("migration input retention"))?;
        Ok(Migration {
            from_version,
            to_version: 4,
            bytes: container::encode(&document, &encoding)?,
        })
    }
    /// Explicit developer migration command; the original path is replaced
    /// only after schema-4 save/verification accepts it. Failure preserves it.
    pub fn upgrade_v1_in_place(
        path: &Path,
        limits: &Limits,
        cancel: &Cancellation,
    ) -> Result<Document> {
        if cancel.is_canceled() {
            return Err(om9_core::Error::Canceled);
        }
        let bytes = atomic_file::read_bounded(path, limits)?;
        let document = container::decode_legacy_v1(&bytes, limits)?;
        let mut saving = *limits;
        saving.max_bytes = limits
            .max_bytes
            .checked_sub(bytes.capacity() as u64)
            .ok_or(om9_core::Error::Budget("migration input retention"))?;
        atomic_file::save(path, &document, &saving, cancel)?;
        Ok(document)
    }
    /// Preflight runtime restoration before publishing the format upgrade.
    /// Missing modules yield owned unresolved diagnostics; restoration failure
    /// (including counter/budget overflow) leaves the schema-1 file intact.
    pub fn upgrade_v1_restored_in_place(
        path: &Path,
        limits: &Limits,
        cancel: &Cancellation,
        available: &std::collections::BTreeSet<ReplayKey>,
    ) -> Result<RestoredDocument> {
        Self::upgrade_restored(path, limits, cancel, available, 1)
    }
    pub fn upgrade_v2_restored_in_place(
        path: &Path,
        limits: &Limits,
        cancel: &Cancellation,
        available: &std::collections::BTreeSet<ReplayKey>,
    ) -> Result<RestoredDocument> {
        Self::upgrade_restored(path, limits, cancel, available, 2)
    }
    pub fn upgrade_v3_restored_in_place(
        path: &Path,
        limits: &Limits,
        cancel: &Cancellation,
        available: &std::collections::BTreeSet<ReplayKey>,
    ) -> Result<RestoredDocument> {
        Self::upgrade_restored(path, limits, cancel, available, 3)
    }
    fn upgrade_restored(
        path: &Path,
        limits: &Limits,
        cancel: &Cancellation,
        available: &std::collections::BTreeSet<ReplayKey>,
        version: u32,
    ) -> Result<RestoredDocument> {
        use om9_core::budget::Usage;
        if cancel.is_canceled() {
            return Err(om9_core::Error::Canceled);
        }
        let bytes = atomic_file::read_bounded(path, limits)?;
        let document = match version {
            1 => container::decode_legacy_v1(&bytes, limits)?,
            2 => container::decode_legacy_v2(&bytes, limits)?,
            3 => container::decode_legacy_v3(&bytes, limits)?,
            _ => return Err(om9_core::Error::Unsupported("legacy migration version")),
        };
        let input = Usage {
            bytes: bytes.capacity() as u64,
            ..Usage::default()
        };
        let mut held = input;
        held.add(document.validate(limits)?, limits)?;
        let restoration_limits = limits.remaining_after(held)?;
        // Prove the private clone fits before allocating it. Resource bytes in
        // schema2 share immutable ownership; admission conservatively counts
        // the complete distinct document clone until sharing is deduplicated.
        restore::admit(&document, &restoration_limits, available)?;
        let restored = restore::restore(document.clone(), &restoration_limits, available)?;
        let mut retained = input;
        retained.add(restored.document.validate(limits)?, limits)?;
        retained.add(
            Usage::array::<String>(restored.diagnostics.capacity())?,
            limits,
        )?;
        for diagnostic in &restored.diagnostics {
            retained.add(
                Usage {
                    bytes: diagnostic.capacity() as u64,
                    ..Usage::default()
                },
                limits,
            )?;
        }
        let saving = limits.remaining_after(retained)?;
        atomic_file::save(path, &document, &saving, cancel)?;
        Ok(restored)
    }
    pub fn save(
        path: &Path,
        document: &Document,
        limits: &Limits,
        cancel: &Cancellation,
    ) -> Result<()> {
        atomic_file::save(path, document, limits, cancel)
    }
    /// Save with per-stage admission for additional owned buffer capacity and
    /// pending scratch estimates. The original document is excluded. Refusal
    /// propagates before the next planned allocation/publication; the caller
    /// must retain its guard until this function returns, including on error.
    /// Codec estimates do not certify allocator/native or process peak memory.
    pub fn save_with_admission(
        path: &Path, document: &Document, limits: &Limits, cancel: &Cancellation,
        mut admit: impl FnMut(u64, u64) -> Result<()>,
    ) -> Result<()> {
        atomic_file::save_with_admission(path, document, limits, cancel, &mut admit)
    }
    pub fn open(path: &Path, limits: &Limits) -> Result<Document> {
        let bytes = atomic_file::read_bounded(path, limits)?;
        Self::decode(&bytes, limits)
    }
    /// Runtime restoration is distinct from validating a serialized document.
    /// `available` comes from the executable's registered replay modules.
    pub fn restore(
        bytes: &[u8],
        limits: &Limits,
        available: &std::collections::BTreeSet<ReplayKey>,
    ) -> Result<RestoredDocument> {
        let document = Self::decode(bytes, limits)?;
        let remaining = limits.remaining_after(om9_core::budget::Usage {
            bytes: bytes.len() as u64,
            ..om9_core::budget::Usage::default()
        })?;
        restore::restore(document, &remaining, available)
    }
    pub fn open_restored(
        path: &Path,
        limits: &Limits,
        available: &std::collections::BTreeSet<ReplayKey>,
    ) -> Result<RestoredDocument> {
        let bytes = atomic_file::read_bounded(path, limits)?;
        Self::restore(&bytes, limits, available)
    }
    /// Reserve the file input allocation before reading and keep the guard
    /// alive through decoding/restoration. Other restore allocations require
    /// their own admission; this reservation covers only the file bytes.
    pub fn open_restored_reserved<T>(
        path: &Path,
        limits: &Limits,
        available: &std::collections::BTreeSet<ReplayKey>,
        reserve: impl FnOnce(u64) -> Result<T>,
    ) -> Result<RestoredDocument> {
        Self::open_restored_reserved_staged(path, limits, available, reserve, |_, _| Ok(()))
    }
    /// The second callback may atomically grow the guard before the decoder
    /// starts; it receives the actual input length, not the sampled file size.
    pub fn open_restored_reserved_staged<T>(
        path: &Path,
        limits: &Limits,
        available: &std::collections::BTreeSet<ReplayKey>,
        reserve: impl FnOnce(u64) -> Result<T>,
        before_restore: impl FnOnce(&mut T, u64) -> Result<()>,
    ) -> Result<RestoredDocument> {
        Self::open_restored_reserved_preflight(path, limits, available, reserve, before_restore, |_, _| Ok(()))
    }
    /// Admit the structural JSON estimate incrementally during preflight.
    pub fn open_restored_reserved_preflight<T>(
        path: &Path,
        limits: &Limits,
        available: &std::collections::BTreeSet<ReplayKey>,
        reserve: impl FnOnce(u64) -> Result<T>,
        before_restore: impl FnOnce(&mut T, u64) -> Result<()>,
        mut admit_document: impl FnMut(&mut T, u64) -> Result<()>,
    ) -> Result<RestoredDocument> {
        Self::open_restored_retaining_reservation(path, limits, available, reserve, before_restore, &mut admit_document)
            .map(|(_reservation, document)| document)
    }
    /// Return the reservation to the caller with the restored document, so
    /// it can cover the document while the next owner is being constructed.
    pub fn open_restored_retaining_reservation<T>(
        path: &Path,
        limits: &Limits,
        available: &std::collections::BTreeSet<ReplayKey>,
        reserve: impl FnOnce(u64) -> Result<T>,
        before_restore: impl FnOnce(&mut T, u64) -> Result<()>,
        mut admit_document: impl FnMut(&mut T, u64) -> Result<()>,
    ) -> Result<(T, RestoredDocument)> {
        let (bytes, mut reservation) = atomic_file::read_bounded_reserved(path, limits, reserve)?;
        let restored = before_restore(&mut reservation, bytes.len() as u64)
            .and_then(|()| {
                let document = container::decode_with_admission(&bytes, limits, &mut |estimated| {
                    admit_document(&mut reservation, estimated)
                })?;
                let remaining = limits.remaining_after(om9_core::budget::Usage {
                    bytes: bytes.len() as u64, ..om9_core::budget::Usage::default()
                })?;
                restore::restore_with_admission(document, &remaining, available, &mut |estimated| {
                    let total = estimated.checked_add(bytes.len() as u64)
                        .ok_or(om9_core::Error::Overflow("archive restore admission"))?;
                    admit_document(&mut reservation, total)
                })
            });
        drop(bytes);
        restored.map(|document| (reservation, document))
    }
}
