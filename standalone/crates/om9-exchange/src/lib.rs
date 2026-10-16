// SPDX-License-Identifier: LGPL-2.1-or-later
//! Owned operation-scoped exchange requests. No archive selects a program.
#![forbid(unsafe_code)]
use om9_core::{
    Error, Result,
    budget::{Limits, Usage},
    identity::{DocumentId, SourceStamp},
    model::Document,
    transaction::{OperationContext, TransactionCoordinator},
};
use serde::Serialize;
use std::{
    io::{self, Write},
    sync::Arc,
};
mod export_document;
pub mod import_capture;
mod inspection;
mod staged;
pub use import_capture::ImportCapture;
mod current_overlay;
mod dependency_cache;
mod output_identity;
mod source_preparation;
pub use source_preparation::{
    PreparedArchiveSpan, PreparedNativeUnits, PreparedSourceRecord, PreservationFlags,
    SourcePreparation, WireIdentity,
};
mod merge_inputs;
pub use merge_inputs::PreparedMergeInputs;
pub mod retained_cold;
mod retained_packet;
pub use retained_cold::{ColdRetainedProof, ExpectedRetainedOutput};
#[cfg(windows)]
mod retained_stage;
#[cfg(windows)]
pub use retained_stage::StagedRetainedRequest;
#[cfg(windows)]
mod retained_write;
#[cfg(windows)]
pub use retained_write::{
    PrivateRetainedFile, RetainedExportLosses, RetainedExportReceipt, RetainedWriteReceipt,
    ValidatedRetainedFile,
};
mod wire_output;
pub use output_identity::{NativeIdentity, OutputPlan, ScopedNativeKey};
pub use wire_output::{OutputWireIdentity, WireOutputPlan};
mod import_dto;
mod import_graph;
mod imported;
pub use current_overlay::{
    ChangedFields, CurrentOverlay, HostValue, NamespaceOverlay, RecordAction, RecordOverlay,
};
pub use imported::{
    ImportMode, ImportOptions, ImportReceipt, PreparedNativeImport, UnitResolution,
};
pub use staged::{ExportReceipt, NativeExchange, ValidatedExport};
pub struct ExportCapture {
    document: Arc<Document>,
    context: OperationContext,
    sources: Vec<SourceStamp>,
}
#[derive(Serialize)]
struct Packet<'a> {
    schema_version: u32,
    mode: &'static str,
    document: ExportDocument<'a>,
}
#[derive(Serialize)]
struct ExportDocument<'a> {
    id: DocumentId,
    name: &'a str,
    page_unit: om9_core::geometry::Unit,
    current_layer: Option<om9_core::identity::LayerId>,
    layer_order: &'a [om9_core::identity::LayerId],
    layers: export_document::Layers<'a>,
    entities: export_document::Entities<'a>,
    definitions: export_document::Definitions<'a>,
}
struct BoundedWriter {
    bytes: Vec<u8>,
    limit: usize,
    failed: bool,
}
impl Write for BoundedWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self
            .bytes
            .len()
            .checked_add(bytes.len())
            .is_none_or(|n| n > self.limit)
        {
            self.failed = true;
            return Err(io::Error::other("exchange packet budget"));
        }
        self.bytes
            .try_reserve_exact(bytes.len())
            .map_err(|_| io::Error::other("exchange packet allocation"))?;
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
impl ExportCapture {
    pub fn all(coordinator: &TransactionCoordinator, id: DocumentId) -> Result<Self> {
        let (context, world) = coordinator.capture_context(None)?;
        let document = world
            .documents
            .get(&id)
            .ok_or(Error::Missing("export document"))?;
        let mut usage = coordinator.retained_usage()?;
        usage.add(
            Usage::array::<SourceStamp>(document.entities.len())?,
            coordinator.limits(),
        )?;
        let mut sources = Vec::new();
        sources
            .try_reserve_exact(document.entities.len())
            .map_err(|_| Error::Budget("export source stamps"))?;
        for id in document.entities.keys() {
            sources.push(document.stamp(*id)?);
        }
        Ok(Self {
            document: document.clone(),
            context,
            sources,
        })
    }
    pub fn document(&self) -> &Document {
        &self.document
    }
    pub fn revalidate(&self, coordinator: &TransactionCoordinator) -> Result<()> {
        if coordinator.authority() != self.context.authority() {
            return Err(Error::WrongOwner);
        }
        let (context, world) = coordinator.capture_context(None)?;
        if context != self.context {
            return Err(Error::Stale("export publication context"));
        }
        let current = world
            .documents
            .get(&self.document.id)
            .ok_or(Error::Missing("export owner"))?;
        if !Arc::ptr_eq(current, &self.document) {
            return Err(Error::Stale("export document snapshot"));
        }
        for stamp in &self.sources {
            current.resolve(stamp)?;
        }
        Ok(())
    }
    pub fn encode(&self, limits: &Limits) -> Result<Vec<u8>> {
        let mut usage = self.document.validate(limits)?;
        usage.add(Usage::array::<SourceStamp>(self.sources.len())?, limits)?;
        let remaining = limits
            .max_bytes
            .checked_sub(usage.bytes)
            .ok_or(Error::Budget("export serialization peak"))?;
        // Reserve transient old/new Vec storage together with the immutable
        // source. Native DOM/geometry memory is separately capped by its owner.
        let limit = usize::try_from((remaining / 2).min(16 * 1024 * 1024))
            .map_err(|_| Error::Overflow("exchange packet address space"))?;
        self.encode_up_to(limit)
    }
    fn encode_up_to(&self, limit: usize) -> Result<Vec<u8>> {
        let d = &self.document;
        let document = ExportDocument {
            id: d.id,
            name: &d.name,
            page_unit: d.page_unit,
            current_layer: d.current_layer,
            layer_order: &d.layer_order,
            layers: export_document::Layers(&d.layers),
            entities: export_document::Entities(&d.entities),
            definitions: export_document::Definitions(&d.definitions),
        };
        let mut writer = BoundedWriter {
            bytes: vec![],
            limit,
            failed: false,
        };
        if let Err(error) = serde_json::to_writer(
            &mut writer,
            &Packet {
                schema_version: 1,
                mode: "geometry_only_current",
                document,
            },
        ) {
            return Err(if writer.failed {
                Error::Budget("exchange packet bytes")
            } else {
                Error::Io(error.to_string())
            });
        }
        Ok(writer.bytes)
    }
}
