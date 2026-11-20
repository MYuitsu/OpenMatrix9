// SPDX-License-Identifier: LGPL-2.1-or-later
//! Project-manager state embedded as an opaque `.om9` resource (OM9-L2-03-15).
use om9_core::{
    Error, Result,
    budget::Usage,
    resource::{Resource, ResourceKind},
    transaction::{Mutation, TransactionRequest},
};

pub const PROJECT_STATE_RESOURCE: &str = "om9.project-manager.v1";
pub const PROJECT_ASSET_PREFIX: &str = "om9.project-asset.v1.";

fn asset_name(hash: &[u8; 32]) -> String {
    use std::fmt::Write;
    let mut name = String::from(PROJECT_ASSET_PREFIX);
    for byte in hash { let _ = write!(name, "{byte:02x}"); }
    name
}

#[cfg(all(feature = "test-fixtures", feature = "occt-brep"))]
impl super::Controller {
    pub fn test_project_proof_rejects_replaced_or_missing_resource() {
        let limits = om9_core::budget::Limits::default();
        let controller = super::Controller::new(om9_core::model::Document::new("proof"), limits).unwrap();
        let owner = controller.coordinator().reserve_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::CpuCache, 1024 * 1024, None, Usage::default()).unwrap();
        let mut document = om9_core::model::Document::new("proof document");
        let resource = Resource::from_bytes(PROJECT_STATE_RESOURCE, ResourceKind::OpaqueBinary, b"state", &limits).unwrap();
        let id = resource.id;
        document.resources.insert(id, resource.clone());
        let proof = ValidatedProjectResources { resources: vec![resource], _owner: owner };
        proof.verify(&document).unwrap();
        let mut replacement = Resource::from_bytes(PROJECT_STATE_RESOURCE, ResourceKind::OpaqueBinary, b"state", &limits).unwrap();
        replacement.id = id;
        document.resources.insert(id, replacement);
        assert!(proof.verify(&document).is_err());
        document.resources.remove(&id);
        assert!(proof.verify(&document).is_err());
    }
}

fn attach_project_manager(document: &mut om9_core::model::Document, manager: &crate::project_manager::ProjectManager,
    limits: &om9_core::budget::Limits) -> Result<()> {
    let (state, assets) = manager.externalized()?;
    let resource = Resource::from_bytes(PROJECT_STATE_RESOURCE, ResourceKind::OpaqueBinary, &state, limits)?;
    document.resources.retain(|_, item| item.name != PROJECT_STATE_RESOURCE && !item.name.starts_with(PROJECT_ASSET_PREFIX));
    document.resources.insert(resource.id, resource);
    for (hash, bytes) in assets {
        let resource = Resource::from_bytes(&asset_name(&hash), ResourceKind::OpaqueBinary, &bytes, limits)?;
        document.resources.insert(resource.id, resource);
    }
    Ok(())
}

#[cfg(feature = "occt-brep")]
pub(crate) struct ValidatedProjectResources {
    resources: Vec<Resource>,
    _owner: om9_core::transaction::ExternalMemoryReservation,
}

#[cfg(feature = "occt-brep")]
impl ValidatedProjectResources {
    pub(super) fn verify(&self, document: &om9_core::model::Document) -> Result<()> {
        let managed = |resource: &&Resource| resource.name == PROJECT_STATE_RESOURCE || resource.name.starts_with(PROJECT_ASSET_PREFIX);
        if document.resources.values().filter(managed).count() != self.resources.len() {
            return Err(Error::Invalid("validated project resources changed"));
        }
        for expected in &self.resources {
            let actual = document.resources.get(&expected.id).ok_or(Error::Invalid("validated project resource missing"))?;
            let a = actual.payload()?;
            let b = expected.payload()?;
            if actual.name != expected.name || actual.kind != expected.kind || actual.byte_length() != expected.byte_length()
                || actual.sha256() != expected.sha256() || a.len() != b.len() || a.as_ptr() != b.as_ptr() {
                return Err(Error::Invalid("validated project resource changed"));
            }
        }
        Ok(())
    }
}

#[cfg(feature = "occt-brep")]
pub(crate) fn prepare_bound_jobbag_project(coordinator: &om9_core::transaction::TransactionCoordinator,
    mut bound: crate::brep_restore_host::BoundBrepDocument, manager: &crate::project_manager::ProjectManager,
    owner: om9_core::transaction::ExternalMemoryReservation, held: Usage, cancel: &om9_core::transaction::Cancellation)
    -> Result<(crate::brep_restore_host::BoundBrepDocument, ValidatedProjectResources)> {
    if cancel.is_canceled() { return Err(Error::Canceled); }
    if !owner.belongs_to(&coordinator.external_memory_account()) { return Err(Error::Invalid("project validation owner")); }
    attach_project_manager(bound.document_mut(), manager, coordinator.limits())?;
    validate_replacement_project_state_on(coordinator, bound.document(), held, Some(cancel))?;
    let managed = || bound.document().resources.values().filter(|resource|
        resource.name == PROJECT_STATE_RESOURCE || resource.name.starts_with(PROJECT_ASSET_PREFIX));
    let mut resources = Vec::new();
    resources.try_reserve_exact(managed().count()).map_err(|_| Error::Budget("project validation resources"))?;
    resources.extend(managed().cloned());
    let proof = ValidatedProjectResources { resources, _owner: owner };
    proof.verify(bound.document())?;
    if cancel.is_canceled() { return Err(Error::Canceled); }
    Ok((bound, proof))
}

pub(crate) fn decode_project_archive_reserved(coordinator: &om9_core::transaction::TransactionCoordinator, archive: &[u8], held: Usage, cancel: Option<&om9_core::transaction::Cancellation>, input_accounted: bool) -> Result<(om9_core::transaction::ExternalMemoryReservation, om9_core::model::Document)> {
        let limits = *coordinator.limits();
        let physical = || om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
        let mut reservation = coordinator.reserve_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::CpuCache, 1, physical(), held,
        )?;
        let decoded = om9_archive::Archive::decode_with_admission(archive, &limits, |estimated| {
            if cancel.is_some_and(|cancel| cancel.is_canceled()) { return Err(Error::Canceled); }
            let required = if input_accounted { estimated.saturating_sub(archive.len() as u64) } else { estimated }.max(1);
            if required > reservation.bytes() {
                let growth = required - reservation.bytes();
                coordinator.grow_external_memory_with_held(&mut reservation, growth, physical(), held)?;
            }
            Ok(())
        })?;
        Ok((reservation, decoded))
}

/// Validate a project asset for metadata checks. V6 native sources are checked
/// in the contained worker; its unbound structural result must never be used
/// as a scene or pasted without the separate binding path.
pub(crate) fn validate_project_archive_reserved(coordinator: &om9_core::transaction::TransactionCoordinator,
    archive: &[u8], held: Usage, cancel: Option<&om9_core::transaction::Cancellation>, input_accounted: bool)
    -> Result<(om9_core::transaction::ExternalMemoryReservation, om9_core::model::Document)> {
    if archive.len() >= 12 && &archive[..8] == b"OM9\0\r\n\x1a\n"
        && matches!(u32::from_le_bytes(archive[8..12].try_into().expect("header checked")), 6 | 7) {
        #[cfg(feature = "occt-brep")]
        {
            let fallback = om9_core::transaction::Cancellation::default();
            let token = cancel.unwrap_or(&fallback);
            let host = crate::brep_restore_host::BrepRestoreHost::admit_packaged(om9_native_owner::ProcessLimits::default())?;
            return host.validate_unbound_archive(coordinator, archive, input_accounted, held, token);
        }
        #[cfg(not(feature = "occt-brep"))]
        return Err(Error::Unsupported("BRep project archive validation"));
    }
    decode_project_archive_reserved(coordinator, archive, held, cancel, input_accounted)
}

pub(crate) fn validate_catalog_project_archives(coordinator: &om9_core::transaction::TransactionCoordinator, manager: &crate::project_manager::ProjectManager, held: Usage, cancel: Option<&om9_core::transaction::Cancellation>) -> Result<()> {
        manager.validate_snapshot_archives(coordinator.limits(), |inline, reference| {
            let archive = match (inline, reference) {
                (Some(bytes), None) => bytes,
                _ => return Err(Error::Invalid("unhydrated catalog archive")),
            };
            validate_project_archive_reserved(coordinator, archive, held, cancel, true)
        })
}

/// Validate embedded manager metadata and its native archives before a
/// replacement is published. The caller retains the document owner.
pub(crate) fn validate_replacement_project_state_on(coordinator: &om9_core::transaction::TransactionCoordinator,
    document: &om9_core::model::Document, held: Usage, cancel: Option<&om9_core::transaction::Cancellation>) -> Result<()> {
        if cancel.is_some_and(|token| token.is_canceled()) { return Err(Error::Canceled); }
        let mut states = document.resources.values().filter(|resource| resource.name == PROJECT_STATE_RESOURCE);
        let Some(state) = states.next() else { return Ok(()); };
        if states.next().is_some() || !matches!(state.kind, ResourceKind::OpaqueBinary) {
            return Err(Error::Invalid("project state resource"));
        }
        let bytes = state.payload()?;
        let allowance = (bytes.len() as u64).checked_mul(16).and_then(|n| n.checked_add(1024 * 1024))
            .ok_or(Error::Overflow("project state validation"))?;
        let physical = om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
        let _reservation = coordinator.reserve_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::CpuCache, allowance, physical, held,
        )?;
        let manager = crate::project_manager::ProjectManager::from_json(bytes, &document.name)?;
        for (hash, length) in manager.required_assets()? {
            if cancel.is_some_and(|token| token.is_canceled()) { return Err(Error::Canceled); }
            let name = asset_name(&hash);
            let mut matches = document.resources.values().filter(|resource| resource.name == name);
            let resource = matches.next().ok_or(Error::Missing("project asset resource"))?;
            if matches.next().is_some() || !matches!(resource.kind, ResourceKind::OpaqueBinary)
                || resource.byte_length() != length || resource.sha256() != hash {
                return Err(Error::Invalid("project asset resource"));
            }
            resource.payload()?;
        }
        manager.validate_snapshot_archives(coordinator.limits(), |inline, reference| {
            let archive = match (inline, reference) {
                (Some(bytes), None) => bytes,
                (None, Some((hash, length))) => {
                    let name = asset_name(&hash);
                    let resource = document.resources.values().find(|resource| resource.name == name)
                        .ok_or(Error::Missing("project asset resource"))?;
                    if resource.byte_length() != length || resource.sha256() != hash {
                        return Err(Error::Invalid("project asset resource"));
                    }
                    resource.payload()?
                }
                _ => return Err(Error::Invalid("project archive reference")),
            };
            validate_project_archive_reserved(coordinator, archive, held, cancel, true)
        })?;
        Ok(())
}

impl super::Controller {
    pub(super) fn validate_replacement_project_state(&self, document: &om9_core::model::Document, held: Usage) -> Result<()> {
        validate_replacement_project_state_on(&self.coordinator, document, held, None)
    }

    /// Capture a native whole-document archive without recursively embedding the
    /// manager that will own it. The archive codec owns resource chunk hashes.
    pub fn capture_jobbag_document(&self) -> Result<(om9_core::model::Document, Vec<u8>)> {
        if self.is_running() {
            return Err(Error::Busy);
        }
        let mut document = self.document().clone();
        document.resources.retain(|_, resource| resource.name != PROJECT_STATE_RESOURCE && !resource.name.starts_with(PROJECT_ASSET_PREFIX));
        let archive = om9_archive::Archive::encode(&document, self.coordinator.limits())?;
        Ok((document, archive))
    }

    /// Validate and hydrate an archive. The returned document is owned by the
    /// caller; UI replacement paths retain a reservation through publication.
    pub fn decode_jobbag_document(&self, archive: &[u8]) -> Result<om9_core::model::Document> {
        self.decode_jobbag_document_reserved(archive).map(|(_, document)| document)
    }

    pub(crate) fn decode_jobbag_document_reserved(&self, archive: &[u8]) -> Result<(om9_core::transaction::ExternalMemoryReservation, om9_core::model::Document)> {
        self.decode_jobbag_document_reserved_with_owner(archive, false)
    }

    pub(crate) fn decode_jobbag_document_reserved_with_owner(&self, archive: &[u8], input_accounted: bool) -> Result<(om9_core::transaction::ExternalMemoryReservation, om9_core::model::Document)> {
        decode_project_archive_reserved(&self.coordinator, archive, self.scene_cache_usage()?, None, input_accounted)
    }

    /// Add current workspace metadata to the candidate before publishing it.
    /// Both metadata and the candidate scene must be admitted together.
    pub fn restore_jobbag_document(&mut self, mut document: om9_core::model::Document, manager: &crate::project_manager::ProjectManager) -> Result<()> {
        let limits = *self.coordinator.limits();
        attach_project_manager(&mut document, manager, &limits)?;
        self.restore_master_document(document)
    }

    pub fn embedded_project_manager(&self) -> Result<Option<crate::project_manager::ProjectManager>> {
        Ok(self.embedded_project_manager_reserved()?.map(|(_, manager)| manager))
    }

    pub(crate) fn embedded_project_manager_reserved(&self) -> Result<Option<(om9_core::transaction::ExternalMemoryReservation, crate::project_manager::ProjectManager)>> {
        let mut states = self.document().resources.values().filter(|resource| resource.name == PROJECT_STATE_RESOURCE);
        let Some(state) = states.next() else { return Ok(None); };
        if states.next().is_some() || !matches!(state.kind, ResourceKind::OpaqueBinary) {
            return Err(Error::Invalid("project state resource"));
        }
        let bytes = state.payload()?;
        let manifest_peak = (bytes.len() as u64).checked_mul(16).and_then(|n| n.checked_add(1024 * 1024))
            .ok_or(Error::Overflow("embedded project admission"))?;
        let held = self.scene_cache_usage()?;
        let physical = || om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
        let mut reservation = self.coordinator.reserve_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::CpuCache, manifest_peak, physical(), held,
        )?;
        let manager = crate::project_manager::ProjectManager::from_json(bytes, &self.document().name)?;
        let needed = manager.required_assets()?;
        let asset_bytes = needed.values().try_fold(0u64, |sum, length| sum.checked_add(*length)
            .and_then(|n| n.checked_add((2 * std::mem::size_of::<usize>()) as u64))
            .ok_or(Error::Overflow("embedded project assets")))?;
        if asset_bytes != 0 { self.coordinator.grow_external_memory_with_held(&mut reservation, asset_bytes, physical(), held)?; }
        let mut assets = crate::project_manager::ProjectAssets::new();
        for (hash, length) in needed {
            let name = asset_name(&hash);
            let mut matches = self.document().resources.values().filter(|resource| resource.name == name);
            let resource = matches.next().ok_or(Error::Missing("project asset resource"))?;
            if matches.next().is_some() || !matches!(resource.kind, ResourceKind::OpaqueBinary)
                || resource.byte_length() != length || resource.sha256() != hash {
                return Err(Error::Invalid("project asset resource"));
            }
            assets.insert(hash, std::sync::Arc::from(resource.payload()?));
        }
        Ok(Some((reservation, manager.hydrate_assets(&assets)?)))
    }

    /// Replace manifest and all owned chunks in one metadata transaction.
    pub fn store_project_manager(&mut self, manager: &crate::project_manager::ProjectManager) -> Result<()> {
        let (state, assets) = manager.externalized()?;
        let mut desired = std::collections::BTreeMap::new();
        desired.insert(PROJECT_STATE_RESOURCE.to_owned(), std::sync::Arc::<[u8]>::from(state));
        for (hash, bytes) in assets { desired.insert(asset_name(&hash), bytes); }
        let mut seen = std::collections::BTreeSet::new();
        let mut equal = true;
        for resource in self.document().resources.values().filter(|resource| resource.name == PROJECT_STATE_RESOURCE || resource.name.starts_with(PROJECT_ASSET_PREFIX)) {
            if !seen.insert(resource.name.clone()) || !matches!(resource.kind, ResourceKind::OpaqueBinary)
                || desired.get(&resource.name).is_none_or(|bytes| resource.payload().ok() != Some(bytes.as_ref())) {
                equal = false;
            }
        }
        if equal && seen.len() == desired.len() { return Ok(()); }
        let revision = self.document().revision;
        let mut mutations = Vec::new();
        for resource in self.document().resources.values().filter(|resource| resource.name == PROJECT_STATE_RESOURCE || resource.name.starts_with(PROJECT_ASSET_PREFIX)) {
            mutations.push(Mutation::ResourceDelete { document: self.document, expected_revision: revision, resource: resource.id });
        }
        for (name, bytes) in desired {
            let resource = Resource::from_bytes(&name, ResourceKind::OpaqueBinary, &bytes, self.coordinator.limits())?;
            mutations.push(Mutation::ResourceInsert { document: self.document, expected_revision: revision, resource });
        }
        let mut request = TransactionRequest::new("Project State", mutations);
        request.recording = false;
        self.commit_ui_metadata(request)
    }
    /// Read the project manager JSON embedded in the open document. A missing
    /// resource is `Ok(None)`; a malformed/foreign resource is an error.
    pub fn embedded_project_state(&self) -> Result<Option<Vec<u8>>> {
        let mut found: Option<Vec<u8>> = None;
        for resource in self.document().resources.values() {
            if resource.name != PROJECT_STATE_RESOURCE {
                continue;
            }
            if !matches!(resource.kind, ResourceKind::OpaqueBinary) {
                return Err(Error::Invalid("project state resource kind"));
            }
            if found.is_some() {
                return Err(Error::Invalid("duplicate project state resource"));
            }
            found = Some(resource.payload()?.to_vec());
        }
        Ok(found)
    }

    /// Replace the embedded project-manager resource without adding an Undo
    /// entry. This runs immediately before Save/Save As, so the archive contains
    /// the current rows/slots/notes.
    pub fn store_project_state(&mut self, bytes: &[u8]) -> Result<()> {
        if bytes.len() > 16 * 1024 * 1024 {
            return Err(Error::Budget("project state bytes"));
        }
        let mut existing = self
            .document()
            .resources
            .values()
            .filter(|r| r.name == PROJECT_STATE_RESOURCE);
        if let Some(old) = existing.next()
            && existing.next().is_none()
            && matches!(old.kind, ResourceKind::OpaqueBinary)
            && old.payload()? == bytes
        {
            return Ok(());
        }
        let limits = *self.coordinator.limits();
        let resource = Resource::from_bytes(
            PROJECT_STATE_RESOURCE,
            ResourceKind::OpaqueBinary,
            bytes,
            &limits,
        )?;
        let mut mutations = Vec::new();
        for old in self.document().resources.values() {
            if old.name == PROJECT_STATE_RESOURCE {
                mutations.push(Mutation::ResourceDelete {
                    document: self.document,
                    expected_revision: self.document().revision,
                    resource: old.id,
                });
            }
        }
        mutations.push(Mutation::ResourceInsert {
            document: self.document,
            expected_revision: self.document().revision,
            resource,
        });
        let mut request = TransactionRequest::new("Project State", mutations);
        request.recording = false;
        self.commit_ui_metadata(request)
    }

    /// Publish resource-only edits with the document's matching scene metadata.
    /// Geometry buffers stay shared; a failed admission keeps both old versions.
    pub(super) fn commit_ui_metadata(&mut self, request: TransactionRequest) -> Result<()> {
        let old_scene = self.scene.as_ref().ok_or(Error::Missing("UI metadata scene"))?;
        if old_scene.epoch != self.coordinator.epoch() {
            return Err(Error::Stale("UI metadata scene"));
        }
        let mut held = self.scene_cache_usage()?;
        let increment = old_scene.update_held_usage(None)?;
        held.add(increment, self.coordinator.limits())?;
        let prepared = self.coordinator.prepare_with_held_usage(request, held)?;
        let receipt = self.commit_incremental_scene(prepared, held, increment, None, false)?;
        if receipt.published {
            self.sync_published(om9_core::changes::ChangeCause::Edit);
        }
        Ok(())
    }

    /// Atomically replace the live document from a captured Master snapshot.
    /// Validation/construction of the candidate happens before the assignment;
    /// a failure leaves the current document untouched.
    pub fn restore_master_document(&mut self, document: om9_core::model::Document) -> Result<()> {
        let limits = *self.coordinator.limits();
        let held = self.replacement_hold(Usage::default())?;
        let auxiliary = self.scene_cache_usage()?;
        let initial = document.validation_peak_estimate(&limits, true)?.bytes.max(1);
        let physical = om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
        let reservation = self.coordinator.reserve_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::CpuCache, initial, physical, auxiliary,
        )?;
        let (candidate, reservation) = self.build_reserved_document_replacement(document, held, auxiliary, reservation)?;
        self.publish_reserved_replacement(reservation, candidate, om9_core::changes::ChangeCause::Replace, Usage::default())
    }
}
