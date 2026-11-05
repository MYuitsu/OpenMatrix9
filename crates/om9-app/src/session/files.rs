// SPDX-License-Identifier: LGPL-2.1-or-later
use super::*;
use crate::file_lifecycle::*;
use om9_archive::Archive;
use om9_core::budget::Usage;
use std::path::{Path, PathBuf};
const PATH_BYTES: usize = 32768;

// The path is already allocated on entry; keep it before its lease on error.
struct ReservedFilePath {
    path: PathBuf,
    reservation: om9_core::transaction::ExternalMemoryReservation,
}

#[cfg(test)]
mod file_memory_tests {
    use super::*;
    use om9_core::{budget::memory::MemoryClass, geometry::Mesh};

    fn fixture() -> Controller {
        let mut document = Document::with_template_tolerance("file memory");
        let entity = Entity::new("triangle", Geometry::Mesh(Mesh {
            vertices: vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]], faces: vec![vec![0,1,2]],
            normals: vec![], colors: vec![], corner_uvs: vec![],
        }), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        let mut controller = Controller::new(document, Limits::default()).unwrap();
        controller.select_objects(&[id]).unwrap();
        controller
    }

    #[test]
    fn detached_capture_observes_file_destination_growth_and_cancel_release() {
        let mut controller = fixture();
        let capture = controller.capture_stl_selected().unwrap();
        controller.begin_file_operation(FileKind::SaveAs).unwrap();
        let token = controller.file_operation().unwrap().token();
        let peer = controller.coordinator.memory_admission_peer().unwrap();
        let (request, requested) = std::sync::mpsc::sync_channel(1);
        let (response, observed) = std::sync::mpsc::sync_channel(1);
        let epoch = capture.source_epoch();
        let worker = std::thread::spawn(move || {
            for _ in 0..5 {
                requested.recv_timeout(std::time::Duration::from_secs(10)).unwrap();
                response.send((peer.retained_usage().unwrap().bytes, peer.external_reserved_bytes(MemoryClass::CpuCache))).unwrap();
            }
            capture.source_epoch()
        });
        let observe = || { request.send(()).unwrap(); observed.recv_timeout(std::time::Duration::from_secs(10)).unwrap() };
        let initial = observe().0;
        let held = controller.auxiliary_usage().unwrap();
        let target = std::env::temp_dir().join(format!("om9-file-owner-{}.om9",uuid::Uuid::new_v4()));
        std::fs::write(&target,b"keep destination").unwrap();
        controller.accept_file_path(token,&target).unwrap();
        let bytes = controller.file.operation.as_ref().unwrap().destination.as_ref().unwrap().capacity() as u64;
        assert_eq!(observe().0-initial,bytes,"detached peer must see the live overwrite destination");
        assert_eq!(controller.auxiliary_usage().unwrap(),held,"owned paths must not also enter caller-held snapshots");
        let token = controller.file_operation().unwrap().token();
        controller.confirm_file_overwrite(token,false).unwrap();
        assert_eq!(observe().0,initial);
        let token = controller.file_operation().unwrap().token();
        controller.cancel_file_operation(token).unwrap();
        assert_eq!(observe().0,initial);
        assert_eq!(std::fs::read(&target).unwrap(),b"keep destination");
        drop(controller);
        assert_eq!(observe().1,0);
        assert_eq!(worker.join().unwrap(),epoch);
        std::fs::remove_file(target).unwrap();
    }

    #[test]
    fn save_path_is_globally_owned_before_writer_and_released_on_failure() {
        let mut controller = fixture();
        controller.submit("Point").unwrap(); controller.submit("2,3,4").unwrap(); controller.undo().unwrap();
        controller.begin_file_operation(FileKind::SaveAs).unwrap();
        let token = controller.file_operation().unwrap().token();
        let peer = controller.coordinator.memory_admission_peer().unwrap();
        let baseline = peer.external_reserved_bytes(MemoryClass::CpuCache);
        let before = controller.coordinator.snapshot();
        let target = std::env::temp_dir().join(format!("om9-file-prewrite-{}.om9",uuid::Uuid::new_v4()));
        std::fs::write(&target,b"keep destination").unwrap();
        let path = target.clone();
        let bytes = path.capacity() as u64;
        let result = controller.perform_file_save_guard(token,path,|| {
            assert!(peer.external_reserved_bytes(MemoryClass::CpuCache) >= baseline+bytes,
                "path must have a live owner while writer work is pending");
            Err(Error::Canceled)
        });
        assert!(matches!(result,Err(Error::Canceled)));
        assert_eq!(peer.external_reserved_bytes(MemoryClass::CpuCache),baseline);
        assert_eq!(controller.coordinator.snapshot(),before);
        assert_eq!(controller.coordinator.redo_len(),1);
        assert!(controller.active_file_path().is_none());
        assert_eq!(std::fs::read(&target).unwrap(),b"keep destination");
        std::fs::remove_file(target).unwrap();
    }

    #[test]
    fn refused_save_path_admission_keeps_destination_and_existing_state() {
        let mut controller = fixture();
        controller.submit("Point").unwrap(); controller.submit("2,3,4").unwrap(); controller.undo().unwrap();
        controller.begin_file_operation(FileKind::SaveAs).unwrap();
        let token = controller.file_operation().unwrap().token();
        let spheres = controller.scene().unwrap().object_spheres().unwrap();
        let index = controller.scene().unwrap().spatial_index().unwrap();
        let before = controller.coordinator.snapshot();
        let pressure = controller.coordinator.reserve_remaining_external_memory_with_held(
            MemoryClass::Kernel,None,controller.scene_cache_usage().unwrap()).unwrap();
        let target = std::env::temp_dir().join(format!("om9-file-refused-{}.om9",uuid::Uuid::new_v4()));
        std::fs::write(&target,b"keep destination").unwrap();
        let result = controller.perform_file_save_guard(token,target.clone(),|| panic!("refused path must not reach writer"));
        assert!(matches!(result,Err(Error::Budget(_))));
        assert_eq!(controller.coordinator.snapshot(),before);
        assert_eq!(controller.file_operation().unwrap().token(),token);
        assert!(controller.file_save_receipt().is_none());
        assert!(controller.active_file_path().is_none());
        assert_eq!(std::fs::read(&target).unwrap(),b"keep destination");
        drop((pressure,spheres,index));
        std::fs::remove_file(target).unwrap();
    }

    #[test]
    fn save_snapshot_restore_and_open_keep_only_live_file_storage() {
        let mut controller = fixture();
        let target = std::env::temp_dir().join(format!("om9-file-lifetime-{}.om9",uuid::Uuid::new_v4()));
        controller.begin_file_operation(FileKind::SaveAs).unwrap();
        let token = controller.file_operation().unwrap().token();
        assert_eq!(controller.accept_file_path(token,&target).unwrap(),FileOutcome::Saved);
        let owned = controller.file.storage_usage().unwrap().bytes;
        assert!(owned>0);
        assert_eq!(controller.file.memory.as_ref().unwrap().bytes(),owned);
        assert_eq!(controller.file.memory.as_ref().unwrap().pending_bytes(),0);
        assert_eq!(controller.file.usage(controller.coordinator.limits()).unwrap().bytes,0);
        let snapshot = controller.capture_snapshot().unwrap();
        controller.submit("Point").unwrap(); controller.submit("10,20,30").unwrap();
        controller.restore_snapshot(&snapshot).unwrap();
        assert_eq!(controller.document().entities.len(),1);
        assert_eq!(controller.file.memory.as_ref().unwrap().bytes(),controller.file.storage_usage().unwrap().bytes);
        assert_eq!(controller.file.usage(controller.coordinator.limits()).unwrap().bytes,0);
        assert_eq!(controller.active_file_path(),Some(target.as_path()));
        let opened = Controller::from_file(&target,Limits::default()).unwrap();
        assert_eq!(opened.file.memory.as_ref().unwrap().bytes(),opened.file.storage_usage().unwrap().bytes);
        assert_eq!(opened.file.usage(opened.coordinator.limits()).unwrap().bytes,0);
        let peer = opened.coordinator.memory_admission_peer().unwrap();
        drop(opened);
        assert_eq!(peer.external_reserved_bytes(MemoryClass::CpuCache),0);
        std::fs::remove_file(target).unwrap();
    }

    #[test]
    fn candidate_diagnostic_buffers_transfer_to_file_owner_before_replacement() {
        let mut controller = fixture();
        let peer = controller.coordinator.memory_admission_peer().unwrap();
        let document = Document::with_template_tolerance("restored diagnostics");
        let held = controller.replacement_hold(Usage::default()).unwrap();
        let auxiliary = controller.scene_cache_usage().unwrap();
        let reservation = controller.coordinator.reserve_external_memory_with_held(
            MemoryClass::CpuCache,document.validation_peak_estimate(controller.coordinator.limits(),true).unwrap().bytes.max(1),None,auxiliary).unwrap();
        let (mut candidate,reservation) = controller.build_reserved_document_replacement(document,held,auxiliary,reservation).unwrap();
        // Exercise the same candidate fields populated by the archive decoder.
        candidate.file.path = Some(PathBuf::from("restored-diagnostics.om9"));
        let mut diagnostic = String::with_capacity(64*1024);
        diagnostic.push_str("restored input warning");
        candidate.file.diagnostics = vec![diagnostic];
        candidate.file.saved = None;
        let bytes = candidate.file.storage_usage().unwrap().bytes;
        assert!(bytes>=64*1024);
        controller.publish_reserved_file_replacement(reservation,candidate).unwrap();
        assert_eq!(controller.file.memory.as_ref().unwrap().bytes(),bytes);
        assert_eq!(controller.file.memory.as_ref().unwrap().pending_bytes(),0);
        assert_eq!(controller.file.usage(controller.coordinator.limits()).unwrap().bytes,0);
        assert_eq!(controller.file_diagnostics(),&["restored input warning"]);
        assert!(peer.external_reserved_bytes(MemoryClass::CpuCache)>=bytes);
        drop(controller);
        assert_eq!(peer.external_reserved_bytes(MemoryClass::CpuCache),0);
    }
}

/// The single user-visible summary line for a completed 3DM import.
///
/// It is a pure function of the receipt so the UI can display it and a test can assert the
/// exact counts without opening a window or a native dialog.
pub fn format_import_summary(receipt: &om9_exchange::ImportReceipt) -> String {
    format!(
        "3DM import summary: {} editable BRep, {} opaque, {} displayable, {} display unavailable, {} annotation, {} hatch, {} light, {} group, {} line pattern, {} page-space",
        receipt.breps_editable,
        receipt.breps_opaque,
        receipt.breps_displayable,
        receipt.breps_display_unavailable,
        receipt.annotations,
        receipt.hatches,
        receipt.lights,
        receipt.groups,
        receipt.line_patterns,
        receipt.page_space_objects
    )
}

impl Controller {
    pub fn from_file(path: &Path, limits: Limits) -> Result<Self> {
        let mut controller = Self::new(Document::with_template_tolerance("Untitled"), limits)?;
        controller.begin_file_operation(FileKind::Open)?;
        let token = controller.file_operation().expect("Open path").token();
        controller.accept_file_path(token, path)?;
        Ok(controller)
    }
    pub fn file_operation(&self) -> Option<&FileOperation> {
        self.file.operation.as_ref()
    }
    pub fn active_file_path(&self) -> Option<&Path> {
        self.file.path.as_deref()
    }
    pub fn file_diagnostics(&self) -> &[String] {
        &self.file.diagnostics
    }
    /// Accepted filesystem work is observable even if a later replacement fails.
    pub fn file_save_receipt(&self) -> Option<&FileSaveReceipt> {
        self.file.accepted_save.as_ref()
    }
    fn reserve_file_path(&self, path: PathBuf) -> Result<ReservedFilePath> {
        let bytes = path.capacity() as u64;
        let reservation = self.coordinator.reserve_external_memory_with_allocated(
            om9_core::budget::memory::MemoryClass::CpuCache, bytes.max(1), bytes,
            om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes)), self.scene_cache_usage()?)?;
        Ok(ReservedFilePath { path, reservation })
    }
    /// Admission is complete before filesystem acceptance/publication. Only
    /// same-account materialized ownership moves here, never a new allocation.
    fn accept_file_path_owner(&mut self, mut incoming: ReservedFilePath) -> PathBuf {
        if let Some(current) = self.file.memory.as_mut() {
            current.transfer_allocated_from(&mut incoming.reservation, incoming.path.capacity() as u64)
                .expect("pre-admitted file path belongs to the same CPU account");
        } else {
            self.file.memory = Some(incoming.reservation);
        }
        incoming.path
    }
    fn release_file_storage(&mut self) {
        let bytes = self.file.storage_usage().expect("file storage already fit its admitted owner").bytes;
        if let Some(memory) = self.file.memory.as_mut() {
            memory.set_allocated_bytes(bytes).expect("file publication stays within its owner");
            memory.shrink_to(bytes).expect("file release cannot grow storage");
        }
    }
    fn bind_file_storage_from(&mut self, reservation: &mut om9_core::transaction::ExternalMemoryReservation) -> Result<u64> {
        let bytes = self.file.storage_usage()?.bytes;
        if bytes != 0 {
            let mut owner = reservation.split_off(bytes)?;
            owner.set_allocated_bytes(bytes)?;
            self.file.memory = Some(owner);
        }
        Ok(bytes)
    }
    fn next_file_token(&self, token: FileToken) -> Result<FileToken> {
        self.validate_file_operation(token)?;
        Ok(FileToken {
            serial: self
                .file
                .serial
                .checked_add(1)
                .ok_or(Error::Overflow("file response serial"))?,
            ..token
        })
    }
    fn publish_file_phase(
        &mut self,
        next: FileToken,
        phase: FilePhase,
        destination: Option<ReservedFilePath>,
    ) {
        let destination = destination.map(|path| self.accept_file_path_owner(path));
        self.reissue_file_phase(next, phase);
        self.file.operation.as_mut().expect("active operation").destination = destination;
        self.release_file_storage();
    }
    fn reissue_file_phase(&mut self, next: FileToken, phase: FilePhase) {
        self.file.serial = next.serial;
        let operation = self.file.operation.as_mut().expect("active operation");
        operation.token = next;
        operation.phase = phase;
    }
    fn advance_file_phase(
        &mut self,
        token: FileToken,
        phase: FilePhase,
        destination: Option<PathBuf>,
    ) -> Result<FileOutcome> {
        let next = self.next_file_token(token)?;
        let destination = destination.map(|path| self.reserve_file_path(path)).transpose()?;
        self.publish_file_phase(next, phase, destination);
        Ok(FileOutcome::Pending)
    }
    fn current_file_hash(&self) -> Result<[u8; 32]> {
        let epoch = self.coordinator.epoch();
        if let Some((cached, hash)) = self.file.cache.get()
            && cached == epoch
        {
            return Ok(hash);
        }
        let hash = fingerprint(self.document())?;
        self.file.cache.set(Some((epoch, hash)));
        Ok(hash)
    }
    pub fn is_dirty(&self) -> Result<bool> {
        Ok(self.file.saved != Some(self.current_file_hash()?))
    }
    fn file_owner(
        &self,
    ) -> (
        om9_core::identity::DocumentId,
        om9_core::identity::SessionId,
        u64,
        u64,
        u64,
        u64,
    ) {
        let d = self.document();
        (
            d.id,
            d.owner_session,
            d.epoch,
            d.session_generation,
            d.revision,
            d.context_revision,
        )
    }
    pub fn validate_file_operation(&self, token: FileToken) -> Result<()> {
        if self
            .file
            .operation
            .as_ref()
            .is_none_or(|op| op.token != token)
        {
            return Err(Error::Stale("file operation token"));
        }
        let (context, _) = self.coordinator.capture_context(None)?;
        if context != token.context || self.file_owner() != token.owner {
            return Err(Error::Stale("file operation document/context"));
        }
        Ok(())
    }
    fn file_phase(&self, token: FileToken, phase: FilePhase) -> Result<()> {
        self.validate_file_operation(token)?;
        if self.file.operation.as_ref().expect("checked").phase != phase {
            return Err(Error::Invalid("file operation phase"));
        }
        Ok(())
    }
    pub fn begin_file_operation(&mut self, kind: FileKind) -> Result<FileOutcome> {
        self.begin_file_intent(FileIntent::Command(kind))
    }
    pub fn begin_window_close(&mut self) -> Result<FileOutcome> {
        self.begin_file_intent(FileIntent::WindowClose)
    }
    fn begin_file_intent(&mut self, intent: FileIntent) -> Result<FileOutcome> {
        if self.layer_dialog.is_some() || self.file.operation.is_some() {
            return Err(Error::Busy);
        }
        // A rebuild that owns the document, or a recipe still waiting for its output
        // insert, must finish before the document can be replaced or closed (`-006`).
        if self.history_rebuilding || self.history_has_pending() {
            return Err(Error::Busy);
        }
        if self.is_running() {
            self.validate_session()?;
        }
        let dirty = self.is_dirty()?;
        let serial = self
            .file
            .serial
            .checked_add(1)
            .ok_or(Error::Overflow("file operation serial"))?;
        let (context, _) = self.coordinator.capture_context(None)?;
        let token = FileToken {
            context,
            owner: self.file_owner(),
            serial,
        };
        let phase = match intent {
            FileIntent::WindowClose if !dirty => FilePhase::CloseReady,
            FileIntent::WindowClose | FileIntent::Command(FileKind::New) => FilePhase::Unsaved,
            FileIntent::Command(FileKind::Open) if dirty => FilePhase::Unsaved,
            FileIntent::Command(FileKind::Open) => FilePhase::OpenPath,
            FileIntent::Command(FileKind::Save | FileKind::SaveAs) => FilePhase::SavePath,
        };
        self.file.operation = Some(FileOperation {
            token,
            intent,
            phase,
            destination: None,
        });
        self.file.serial = serial;
        self.file.accepted_save = None;
        match intent {
            FileIntent::WindowClose if !dirty => Ok(FileOutcome::CloseReady),
            FileIntent::Command(FileKind::New) if !dirty => {
                let result = self.replace_with_new(token);
                if result.is_err() {
                    self.file.operation = None;
                    self.release_file_storage();
                }
                result
            }
            FileIntent::Command(FileKind::Save) if self.file.path.is_some() => {
                let result = self
                    .copy_file_path(self.file.path.as_deref().expect("path"))
                    .and_then(|path| self.perform_file_save(token, path));
                if result.is_err() {
                    self.file.operation = None;
                    self.release_file_storage();
                }
                result
            }
            _ => Ok(FileOutcome::Pending),
        }
    }
    pub fn decide_unsaved(
        &mut self,
        token: FileToken,
        decision: UnsavedDecision,
    ) -> Result<FileOutcome> {
        self.file_phase(token, FilePhase::Unsaved)?;
        match decision {
            UnsavedDecision::Discard => self.after_replacement_permission(token),
            UnsavedDecision::Save => {
                if let Some(path) = self.file.path.as_deref() {
                    let path = self.copy_file_path(path)?;
                    self.perform_file_save(token, path)
                } else {
                    self.advance_file_phase(token, FilePhase::SavePath, None)
                }
            }
        }
    }
    fn after_replacement_permission(&mut self, token: FileToken) -> Result<FileOutcome> {
        self.validate_file_operation(token)?;
        match self.file.operation.as_ref().expect("checked").intent {
            FileIntent::WindowClose => {
                self.advance_file_phase(token, FilePhase::CloseReady, None)?;
                Ok(FileOutcome::CloseReady)
            }
            FileIntent::Command(FileKind::New) => self.replace_with_new(token),
            FileIntent::Command(FileKind::Open) => {
                self.advance_file_phase(token, FilePhase::OpenPath, None)
            }
            _ => Err(Error::Invalid("replacement intent")),
        }
    }
    fn path_scratch(&self) -> Result<Usage> {
        let mut held = self.scene_cache_usage()?;
        held.add(
            Usage {
                bytes: (PATH_BYTES * 3) as u64,
                ..Usage::default()
            },
            self.coordinator.limits(),
        )?;
        self.coordinator.retained_usage_with_held_usage(held)
    }
    fn copy_file_path(&self, path: &Path) -> Result<PathBuf> {
        if path.as_os_str().as_encoded_bytes().len() > PATH_BYTES {
            return Err(Error::Budget("file path bytes"));
        }
        self.path_scratch()?;
        let mut owned = PathBuf::new();
        owned
            .try_reserve_exact(path.as_os_str().len())
            .map_err(|_| Error::Budget("file path allocation"))?;
        owned.push(path);
        Ok(owned)
    }
    fn native_file_path(&self, path: &Path) -> Result<PathBuf> {
        let encoded = path.as_os_str().as_encoded_bytes();
        if encoded.len() > PATH_BYTES {
            return Err(Error::Budget("file path bytes"));
        }
        if encoded.is_empty() || encoded.contains(&0) {
            return Err(Error::Invalid("file path"));
        }
        if !path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("om9"))
        {
            return Err(Error::Unsupported(
                "selected file codec; choose native .om9",
            ));
        }
        if path.file_name().is_none() {
            return Err(Error::Invalid("file name"));
        }
        self.path_scratch()?;
        let path = std::path::absolute(path).map_err(|e| Error::Io(e.to_string()))?;
        if path.as_os_str().as_encoded_bytes().len() > PATH_BYTES {
            return Err(Error::Budget("absolute file path bytes"));
        }
        Ok(path)
    }
    pub fn accept_file_path(&mut self, token: FileToken, path: &Path) -> Result<FileOutcome> {
        self.validate_file_operation(token)?;
        let phase = self.file.operation.as_ref().expect("checked").phase;
        if !matches!(phase, FilePhase::OpenPath | FilePhase::SavePath) {
            return Err(Error::Invalid("file path phase"));
        }
        let next = self.next_file_token(token)?;
        let result = self.accept_file_path_inner(token, path);
        if result.is_err()
            && self
                .file
                .operation
                .as_ref()
                .is_some_and(|operation| operation.token == token)
        {
            self.publish_file_phase(next, phase, None);
        }
        result
    }
    fn accept_file_path_inner(&mut self, token: FileToken, path: &Path) -> Result<FileOutcome> {
        self.validate_file_operation(token)?;
        let phase = self.file.operation.as_ref().expect("checked").phase;
        if !matches!(phase, FilePhase::OpenPath | FilePhase::SavePath) {
            return Err(Error::Invalid("file path phase"));
        }
        let path = self.native_file_path(path)?;
        if phase == FilePhase::OpenPath {
            return self.replace_from_file(token, path);
        }
        match std::fs::symlink_metadata(&path) {
            Ok(meta) => {
                if !meta.is_file() || meta.permissions().readonly() {
                    return Err(Error::Permission("save destination"));
                }
                self.advance_file_phase(token, FilePhase::Overwrite, Some(path))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                self.perform_file_save(token, path)
            }
            Err(e) => Err(Error::Io(e.to_string())),
        }
    }
    pub fn confirm_file_overwrite(&mut self, token: FileToken, yes: bool) -> Result<FileOutcome> {
        self.file_phase(token, FilePhase::Overwrite)?;
        let next = self.next_file_token(token)?;
        let result = self.confirm_file_overwrite_inner(token, yes);
        if result.is_err()
            && self
                .file
                .operation
                .as_ref()
                .is_some_and(|operation| operation.token == token)
        {
            self.reissue_file_phase(next, FilePhase::Overwrite);
        }
        result
    }
    fn confirm_file_overwrite_inner(&mut self, token: FileToken, yes: bool) -> Result<FileOutcome> {
        self.file_phase(token, FilePhase::Overwrite)?;
        if !yes {
            return self.advance_file_phase(token, FilePhase::SavePath, None);
        }
        let path = self.copy_file_path(
            self.file
                .operation
                .as_ref()
                .expect("checked")
                .destination
                .as_deref()
                .expect("overwrite target"),
        )?;
        self.perform_file_save(token, path)
    }
    fn perform_file_save(&mut self, token: FileToken, path: PathBuf) -> Result<FileOutcome> {
        self.perform_file_save_guard(token, path, || Ok(()))
    }
    fn perform_file_save_guard(&mut self, token: FileToken, path: PathBuf, guard: impl FnOnce() -> Result<()>) -> Result<FileOutcome> {
        self.validate_file_operation(token)?;
        let intent = self.file.operation.as_ref().expect("checked").intent;
        let next = if matches!(
            intent,
            FileIntent::WindowClose | FileIntent::Command(FileKind::New | FileKind::Open)
        ) {
            Some(self.next_file_token(token)?)
        } else {
            None
        };
        let hash = self.current_file_hash()?;
        let path = self.reserve_file_path(path)?;
        self.save_archive_for_batch_guard(&path.path, 0, guard)?;
        let path = self.accept_file_path_owner(path);
        self.file.path = Some(path);
        self.release_file_storage();
        self.file.saved = Some(hash);
        self.file.accepted_save = Some(FileSaveReceipt {
            token,
            fingerprint: hash,
        });
        match intent {
            FileIntent::WindowClose => {
                self.publish_file_phase(next.expect("preflight"), FilePhase::CloseReady, None);
                Ok(FileOutcome::SavedAndCloseReady)
            }
            FileIntent::Command(FileKind::Open) => {
                self.publish_file_phase(next.expect("preflight"), FilePhase::OpenPath, None);
                Ok(FileOutcome::SavedPending)
            }
            FileIntent::Command(FileKind::New) => match self.replace_with_new(token) {
                Ok(_) => Ok(FileOutcome::SavedAndReplaced),
                Err(error) => {
                    self.publish_file_phase(next.expect("preflight"), FilePhase::Unsaved, None);
                    Err(error)
                }
            },
            FileIntent::Command(FileKind::Save | FileKind::SaveAs) => {
                self.file.operation = None;
                self.release_file_storage();
                Ok(FileOutcome::Saved)
            }
        }
    }
    /// Save a batch result using the same admission as interactive Save.
    pub fn save_archive_for_batch(&self, path: &Path) -> Result<()> {
        self.save_archive_for_batch_guard(path, path.as_os_str().len() as u64, || Ok(()))
    }
    fn save_archive_for_batch_guard(&self, path: &Path, path_bytes: u64, guard: impl FnOnce() -> Result<()>) -> Result<()> {
        let limits = *self.coordinator.limits();
        let mut auxiliary = self.scene_cache_usage()?;
        auxiliary.add(
            Usage {
                bytes: path_bytes,
                ..Usage::default()
            },
            &limits,
        )?;
        let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
        // Admit only the next phase, retaining actual buffers and old/new
        // overlap. Other shared-account work may use the remaining headroom.
        let mut reservation = Some(self.coordinator.reserve_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::CpuCache,
            self.document().validation_scratch_usage(&limits)?.bytes.max(1), physical, auxiliary,
        )?);
        guard()?;
        Archive::save_with_admission(
            &path,
            self.document(),
            &limits,
            &om9_core::transaction::Cancellation::default(),
            |allocated, pending| {
                let requested = allocated.checked_add(pending).ok_or(Error::Overflow("save memory budget"))?;
                if requested == 0 { reservation = None; return Ok(()); }
                if let Some(lease) = reservation.as_mut() {
                    // JSON preflight reports each token. Reuse an admitted
                    // window within this phase instead of scanning all owners
                    // and sampling the OS for every scalar.
                    if pending != 0 && requested <= lease.bytes()
                        && lease.bytes() - lease.pending_bytes() == allocated { return Ok(()); }
                    lease.set_allocated_bytes(allocated.min(lease.bytes()))?;
                    if requested > lease.bytes() {
                        let additional = requested - lease.bytes();
                        let physical = om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
                        // A bounded admission quantum, not a model/native peak
                        // multiplier. Near the ceiling retry the exact request.
                        let batch = additional.max(64 * 1024);
                        match self.coordinator.grow_external_memory_with_held(lease, batch, physical, auxiliary) {
                            Err(Error::Budget(_)) if batch > additional =>
                                self.coordinator.grow_external_memory_with_held(lease, additional, physical, auxiliary)?,
                            other => other?,
                        }
                    } else {
                        lease.shrink_to(requested)?;
                    }
                    lease.set_allocated_bytes(allocated)?;
                    if pending != 0 { lease.recheck_pending()?; }
                } else {
                    let physical = om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
                    reservation = Some(self.coordinator.reserve_external_memory_with_allocated(
                        om9_core::budget::memory::MemoryClass::CpuCache, requested, allocated, physical, auxiliary)?);
                }
                Ok(())
            },
        )?;
        drop(reservation);
        Ok(())
    }
    pub(super) fn replacement_hold(&self, extra: Usage) -> Result<Usage> {
        let mut auxiliary = self.scene_cache_usage()?;
        auxiliary.add(extra, self.coordinator.limits())?;
        self.coordinator.retained_usage_with_held_usage(auxiliary)
    }
    fn candidate_controller_overhead(extra: Usage, limits: &Limits) -> Result<Usage> {
        let mut usage = extra;
        usage.add(Workspace::startup_usage()?, limits)?;
        usage.add(Usage::array::<Self>(1)?, limits)?;
        usage.add(Usage::btree::<DocumentId, Arc<Document>>(1)?, limits)?;
        usage.add(Usage::array::<Document>(1)?, limits)?;
        Ok(usage)
    }
    fn publish_reserved_file_replacement(
        &mut self,
        reservation: om9_core::transaction::ExternalMemoryReservation,
        candidate: Self,
    ) -> Result<()> {
        self.publish_reserved_replacement(reservation, candidate, om9_core::changes::ChangeCause::Replace, Usage::default())
    }
    pub(super) fn publish_reserved_replacement(
        &mut self,
        mut reservation: om9_core::transaction::ExternalMemoryReservation,
        mut candidate: Self,
        cause: om9_core::changes::ChangeCause,
        extra_held: Usage,
    ) -> Result<()> {
        candidate.coordinator.set_undo_policy(self.coordinator.undo_policy())?;
        let candidate_bytes = candidate.replacement_hold(Usage::default())?.bytes;
        let scene = candidate.scene.as_ref().ok_or(Error::Missing("replacement scene"))?;
        let buffer_bytes = scene.shared_buffer_bytes();
        let arena = scene.picking_arena_bytes()?;
        let metadata = scene.metadata_storage_usage()?.bytes;
        let owner_overhead = if buffer_bytes == 0 { 0 } else { super::scene_buffer_lease_overhead() };
        let required = candidate_bytes.checked_add(owner_overhead).and_then(|bytes| bytes.checked_add(arena))
            .ok_or(Error::Overflow("replacement scene owner"))?;
        let mut auxiliary = self.scene_cache_usage()?;
        auxiliary.add(extra_held, self.coordinator.limits())?;
        let additional = required.saturating_sub(reservation.bytes());
        if additional != 0 {
            let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
            self.coordinator.grow_external_memory_with_held(&mut reservation, additional, physical, auxiliary)?;
        }
        candidate.coordinator.adopt_external_captures(&self.coordinator)?;
        candidate.scene.as_ref().ok_or(Error::Missing("replacement scene"))?.transfer_memory_allowance(&mut reservation, &candidate.coordinator, auxiliary)?;
        if buffer_bytes != 0 {
            let bytes = buffer_bytes.checked_add(owner_overhead).ok_or(Error::Overflow("replacement scene owner"))?;
            let buffer_reservation = reservation.split_off(bytes)?;
            let scene = candidate.scene.as_ref().ok_or(Error::Missing("replacement scene"))?;
            let lease = super::SceneBufferLease::new(None, buffer_reservation)?;
            *candidate.scene_buffer_owner.lock().unwrap_or_else(|e| e.into_inner()) = Some(super::SceneBufferOwner {
                epoch: scene.epoch, document: Arc::downgrade(&scene.document), bytes: buffer_bytes, lease,
            });
        }
        let file_bytes = candidate.bind_file_storage_from(&mut reservation)?;
        let candidate_bytes = candidate_bytes.checked_sub(buffer_bytes).and_then(|bytes| bytes.checked_sub(metadata))
            .and_then(|bytes| bytes.checked_sub(file_bytes))
            .ok_or(Error::Invalid("replacement scene accounting"))?;
        let admission = self.coordinator.replacement_admission();
        let mut old = None;
        let retired;
        let old_stl_session = self.stl_session.token.clone();
        {
            let mut guard = admission.lock();
            let transfer = guard.prepare_candidate_reservation(&self.coordinator, &mut reservation,
                Usage { bytes: candidate_bytes, ..Usage::default() }, auxiliary)?;
            retired = transfer.publish_retaining_old(|| {
                // Serialize replacement with the final STL rename: whichever
                // acquires this guard first publishes before the other.
                let mut session = old_stl_session.lock().unwrap_or_else(|e| e.into_inner());
                candidate.move_contract_state(self);
                old = Some(std::mem::replace(self, candidate));
                *session = false;
            });
        }
        drop(old);
        drop(retired);
        self.sync_published(cause);
        Ok(())
    }
    pub(super) fn build_reserved_document_replacement(
        &self,
        document: Document,
        held: Usage,
        auxiliary: Usage,
        mut reservation: om9_core::transaction::ExternalMemoryReservation,
    ) -> Result<(Self, om9_core::transaction::ExternalMemoryReservation)> {
        let limits = *self.coordinator.limits();
        let mut estimated = Self::candidate_controller_overhead(Usage::default(), &limits)?;
        estimated.add(document.validation_peak_estimate(&limits, true)?, &limits)?;
        estimated.add(Scene::candidate_preflight_scratch(&document, &limits)?, &limits)?;
        if estimated.bytes > reservation.bytes() {
            let additional = estimated.bytes - reservation.bytes();
            let physical = om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
            self.coordinator.grow_external_memory_with_held(&mut reservation, additional, physical, auxiliary)?;
        }
        let scene_peak = Scene::candidate_scene_peak(&document, &limits)?;
        let required = estimated.bytes.checked_add(scene_peak).ok_or(Error::Overflow("replacement scene peak"))?;
        if required > reservation.bytes() {
            let additional = required - reservation.bytes();
            let physical = om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
            self.coordinator.grow_external_memory_with_held(&mut reservation, additional, physical, auxiliary)?;
        }
        let candidate = Self::new_with_held(document, limits, held)?;
        Ok((candidate, reservation))
    }
    fn replace_with_new(&mut self, token: FileToken) -> Result<FileOutcome> {
        self.replace_with_new_guard(token, || Ok(()))
    }
    fn replace_with_new_guard<F>(&mut self, token: FileToken, guard: F) -> Result<FileOutcome>
    where F: FnOnce() -> Result<()> {
        self.replace_with_new_guards(token, || Ok(()), guard)
    }
    fn replace_with_new_guards<P, F>(&mut self, token: FileToken, preparing: P, guard: F) -> Result<FileOutcome>
    where P: FnOnce() -> Result<()>, F: FnOnce() -> Result<()> {
        self.validate_file_operation(token)?;
        let limits = *self.coordinator.limits();
        let held = self.replacement_hold(Usage {
            bytes: 512,
            ..Usage::default()
        })?;
        let document = Document::with_template_tolerance("Untitled");
        let auxiliary = self.scene_cache_usage()?;
        let mut estimated = Self::candidate_controller_overhead(Usage::default(), &limits)?;
        estimated.add(document.validation_peak_estimate(&limits, true)?, &limits)?;
        estimated.add(Scene::candidate_planning_scratch(&document)?, &limits)?;
        let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
        let mut reservation = self.coordinator.reserve_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::CpuCache, estimated.bytes.max(1), physical, auxiliary,
        )?;
        let scene_peak = Scene::candidate_scene_peak(&document, &limits)?;
        if scene_peak != 0 {
            let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
            self.coordinator.grow_external_memory_with_held(&mut reservation, scene_peak, physical, auxiliary)?;
        }
        preparing()?;
        let candidate = Self::new_with_held(document, limits, held)?;
        guard()?;
        // Observers and the published change ring survive a document
        // replacement: a caller that tracks the document keeps its subscription.
        self.publish_reserved_file_replacement(reservation, candidate)?;
        Ok(FileOutcome::Replaced)
    }
    fn replace_from_file(&mut self, token: FileToken, path: PathBuf) -> Result<FileOutcome> {
        self.replace_from_file_guard(token, path, || Ok(()))
    }
    /// Load a batch input through the reserved Open path, preserving its saved
    /// document name and accepting an archive regardless of filename extension.
    pub fn open_archive_for_batch(path: &Path, limits: Limits) -> Result<Self> {
        let mut controller = Self::new(Document::with_template_tolerance("Batch input"), limits)?;
        if controller.begin_file_operation(FileKind::Open)? != FileOutcome::Pending {
            return Err(Error::Invalid("batch open state"));
        }
        let token = controller.file_operation().ok_or(Error::Missing("batch open"))?.token();
        let result = controller.replace_from_file_with_planning_sample(token, path.to_path_buf(), || Ok(()), || {
            om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes))
        }, false)?;
        if result != FileOutcome::Replaced { return Err(Error::Invalid("batch open outcome")); }
        Ok(controller)
    }
    fn replace_from_file_guard<F>(&mut self, token: FileToken, path: PathBuf, guard: F) -> Result<FileOutcome>
    where F: FnOnce() -> Result<()> {
        self.replace_from_file_with_planning_sample(token, path, guard, || {
            om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes))
        }, true)
    }
    fn replace_from_file_with_planning_sample<F, S>(&mut self, token: FileToken, path: PathBuf, guard: F, sample: S, name_from_path: bool) -> Result<FileOutcome>
    where F: FnOnce() -> Result<()>, S: FnOnce() -> Option<(u64, u64)> {
        self.file_phase(token, FilePhase::OpenPath)?;
        let limits = *self.coordinator.limits();
        let path_usage = Usage {
            bytes: path.capacity() as u64,
            ..Usage::default()
        };
        let held = self.replacement_hold(path_usage)?;
        let mut auxiliary = self.scene_cache_usage()?;
        auxiliary.add(path_usage, &limits)?;
        let (mut archive_reservation, restored) = Archive::open_restored_retaining_reservation(
            &path,
            &limits.remaining_after(held)?,
            &std::collections::BTreeSet::new(),
            |bytes| {
                let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
                self.coordinator.reserve_external_memory_with_held(
                    om9_core::budget::memory::MemoryClass::CpuCache, bytes.max(1), physical, auxiliary,
                )
            },
            |reservation, bytes| {
                if bytes == 0 { return Ok(()); }
                let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
                self.coordinator.grow_external_memory_with_held(reservation, bytes, physical, auxiliary)
            },
            |reservation, estimated| {
                let additional = estimated.saturating_sub(reservation.bytes());
                if additional == 0 { return Ok(()); }
                let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
                // Batch growth so streaming JSON does not query the OS or lock
                // the account for every node. Near the limit, retry exact size.
                let batch = additional.max(64 * 1024);
                match self.coordinator.grow_external_memory_with_held(reservation, batch, physical, auxiliary) {
                    Err(Error::Budget(_)) if batch != additional =>
                        self.coordinator.grow_external_memory_with_held(reservation, additional, physical, auxiliary),
                    result => result,
                }
            },
        )?;
        let mut extra = path_usage;
        extra.add(
            Usage::array::<String>(restored.diagnostics.capacity())?,
            &limits,
        )?;
        for diagnostic in &restored.diagnostics {
            extra.add(
                Usage {
                    bytes: diagnostic.capacity() as u64,
                    ..Usage::default()
                },
                &limits,
            )?;
        }
        let held = self.replacement_hold(extra)?;
        let candidate_overhead = Self::candidate_controller_overhead(extra, &limits)?;
        if candidate_overhead.bytes != 0 {
            let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
            self.coordinator.grow_external_memory_with_held(&mut archive_reservation, candidate_overhead.bytes, physical, auxiliary)?;
        }
        // The document name follows the file it was opened from: the title, the
        // status line and a later Save As then name the same document. Without a
        // file name the archive's own name is kept, and the name is bounded so a
        // very long path cannot grow the document without limit.
        let mut document = restored.document;
        if name_from_path && let Some(name) = path.file_name() {
            let bounded: String = name.to_string_lossy().chars().take(240).collect();
            if !bounded.is_empty() {
                document.name = bounded;
            }
        }
        // The candidate controller builds its scene in a separate account.
        // Hold its estimated planner and construction peak on the shared
        // account too, before either phase starts.
        let scratch = Scene::candidate_preflight_scratch(&document, &limits)?;
        let planning = self.coordinator.reserve_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::CpuCache, scratch.bytes.max(1), sample(), auxiliary,
        )?;
        let scene_peak = Scene::candidate_scene_peak(&document, &limits)?;
        drop(planning);
        if scene_peak != 0 {
            let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
            self.coordinator.grow_external_memory_with_held(&mut archive_reservation, scene_peak, physical, auxiliary)?;
        }
        let mut candidate = Self::new_with_held(document, limits, held)?;
        candidate.file.path = Some(path);
        if !restored.diagnostics.is_empty() {
            candidate.file.saved = None;
        }
        candidate.file.diagnostics = restored.diagnostics;
        guard()?;
        // Observers and the change ring survive an Open exactly like a New.
        self.publish_reserved_file_replacement(archive_reservation, candidate)?;
        Ok(FileOutcome::Replaced)
    }
    pub fn cancel_file_operation(&mut self, token: FileToken) -> Result<()> {
        if self
            .file
            .operation
            .as_ref()
            .is_none_or(|op| op.token != token)
        {
            return Err(Error::Stale("file operation token"));
        }
        self.file.operation = None;
        self.release_file_storage();
        Ok(())
    }
    /// Import a Rhino 3DM into the open document as one transaction. The native executable is
    /// admitted by the host adapter from trusted configuration, never from the file.
    pub fn import_3dm(&mut self, source: &std::path::Path) -> Result<Vec<String>> {
        self.import_3dm_with_clipboard_metadata(source, None)
    }
    pub(super) fn import_3dm_with_clipboard_metadata(
        &mut self,
        source: &std::path::Path,
        metadata: Option<&om9_exchange::three_dm::ThreeDmLayerSnapshot>,
    ) -> Result<Vec<String>> {
        let host = crate::three_dm_host::ThreeDmHost::configured()?;
        let document = self.document;
        let next_selection_revision = self.selection_revision
            .checked_add(1).ok_or(Error::Overflow("selection revision"))?;
        let mut held = self.scene_cache_usage()?;
        let (prepared, receipt) = host.prepare_import_with_metadata(
            &self.coordinator,
            document,
            source,
            &om9_exchange::ImportOptions {
                mode: om9_exchange::ImportMode::Geometry,
                brep: if cfg!(feature = "occt-brep") { om9_exchange::BrepMode::EditableBestEffort } else { om9_exchange::BrepMode::RetainOpaque },
                ..Default::default()
            },
            metadata,
            &om9_core::transaction::Cancellation::default(),
        )?;
        held.add(prepared.held_usage(), self.coordinator.limits())?;
        self.commit_with_candidate_scene(prepared, held)?;
        self.selection.clear();
        self.selection_order.clear();
        self.release_removed_root_selection();
        self.selection_revision = next_selection_revision;
        let summary = format_import_summary(&receipt);
        let mut diagnostics = receipt.diagnostics;
        diagnostics.push(summary);
        Ok(diagnostics)
    }
    /// Export the current document to Rhino 3DM without changing the document, selection or Undo.
    pub fn export_3dm(&self, target: &std::path::Path) -> Result<Vec<String>> {
        let host = crate::three_dm_host::ThreeDmHost::configured()?;
        let receipt = host.export(
            self.coordinator(),
            self.document,
            target,
            &om9_core::transaction::Cancellation::default(),
        )?;
        Ok(receipt.diagnostics)
    }
}
#[cfg(test)]
mod replacement_budget_tests {
    use super::*;
    #[test]
    fn late_restore_publication_refusal_keeps_observers_and_history() {
        struct Observer;
        impl super::super::contract::DocumentObserver for Observer {
            fn document_changed(&mut self, _: &om9_core::changes::ChangeBatch) -> Result<()> { Ok(()) }
        }
        let limits = Limits { max_bytes: 64 * 1024 * 1024, ..Limits::default() };
        let mut controller = Controller::new(Document::with_template_tolerance("current"), limits).unwrap();
        controller.submit("Point").unwrap(); controller.submit("1,2,3").unwrap();
        controller.add_document_observer(Box::new(Observer));
        let before = controller.coordinator().snapshot();
        let baseline = controller.coordinator().external_reserved_bytes(om9_core::budget::memory::MemoryClass::CpuCache);
        let candidate_document = Document::with_template_tolerance("restored");
        let held = controller.replacement_hold(Usage::default()).unwrap();
        let auxiliary = controller.scene_cache_usage().unwrap();
        let reservation = controller.coordinator().reserve_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::CpuCache, 1, None, auxiliary,
        ).unwrap();
        let (candidate, reservation) = controller.build_reserved_document_replacement(candidate_document.clone(), held, auxiliary, reservation).unwrap();
        let pressure = controller.coordinator().reserve_remaining_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::Export, None, auxiliary,
        ).unwrap();
        let result = controller.publish_reserved_replacement(reservation, candidate, om9_core::changes::ChangeCause::Restore, Usage::default());
        assert!(matches!(result, Err(Error::Budget(_))), "{result:?}");
        assert_eq!(controller.coordinator().snapshot(), before);
        assert_eq!(controller.observer_count(), 1);
        assert_eq!(controller.coordinator().external_reserved_bytes(om9_core::budget::memory::MemoryClass::CpuCache), baseline);
        drop(pressure);
        controller.restore_master_document(candidate_document).unwrap();
        assert_eq!(controller.observer_count(), 1);
        assert!(controller.document().entities.is_empty());
    }
    use om9_core::{budget::memory::MemoryClass, geometry::{Cloud, Geometry}, model::Entity,
        transaction::{TransactionCoordinator, World}};

    #[test]
    fn save_reserves_only_its_phases_and_allows_competing_work() {
        const MIB: u64 = 1024 * 1024;
        let limits = Limits { max_bytes: 32 * MIB, ..Limits::default() };
        let mut document = Document::new("save reservation");
        let resource = om9_core::resource::Resource::from_bytes("save payload",
            om9_core::resource::ResourceKind::OpaqueBinary, &vec![0x5a; MIB as usize], &limits).unwrap();
        document.resources.insert(resource.id, resource);
        let mut controller = Controller::new(document, limits).unwrap();
        let mut peer = TransactionCoordinator::new(World::new(vec![Document::new("peer")], &limits).unwrap(), limits).unwrap();
        peer.adopt_external_captures(controller.coordinator()).unwrap();
        let path = std::env::temp_dir().join(format!("om9-save-lease-{}.om9", uuid::Uuid::new_v4()));
        std::fs::write(&path, b"original destination").unwrap();
        controller.begin_file_operation(FileKind::SaveAs).unwrap();
        let token = controller.file_operation().unwrap().token();
        let before = controller.coordinator().snapshot();
        let mut competing = None;
        let result = controller.perform_file_save_guard(token, path.clone(), || {
            assert!(peer.external_reserved_bytes(MemoryClass::CpuCache) > 0);
            competing = Some(peer.reserve_external_memory(MemoryClass::Export, 16 * MIB, None)?);
            Err(Error::Canceled)
        });
        assert!(matches!(result, Err(Error::Canceled)));
        assert_eq!(std::fs::read(&path).unwrap(), b"original destination");
        assert_eq!(controller.coordinator().snapshot(), before);
        assert_eq!(peer.external_reserved_bytes(MemoryClass::CpuCache), controller.scene.as_ref().unwrap().memory_allowance_bytes());
        drop(competing.take());
        assert!(matches!(controller.perform_file_save_guard(token, path.clone(), || {
            assert!(peer.external_reserved_bytes(MemoryClass::CpuCache) > 0);
            competing = Some(peer.reserve_external_memory(MemoryClass::Export, 16 * MIB, None)?);
            Ok(())
        }), Ok(FileOutcome::Saved)));
        assert_eq!(peer.external_reserved_bytes(MemoryClass::Export), 16 * MIB,
            "Save must finish while competing work retains its own admitted capacity");
        assert_eq!(peer.external_reserved_bytes(MemoryClass::CpuCache),
            controller.scene.as_ref().unwrap().memory_allowance_bytes() + controller.file.storage_usage().unwrap().bytes);
        assert_eq!(Archive::open(&path, &limits).unwrap(), *controller.document());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn save_phase_growth_refusal_preserves_destination_document_and_redo() {
        let limits = Limits { max_bytes: 32 * 1024 * 1024, ..Limits::default() };
        let mut controller = Controller::new(Document::new("save failure"), limits).unwrap();
        for point in ["1,2,3", "4,5,6"] { controller.submit("Point").unwrap(); controller.submit(point).unwrap(); }
        controller.undo().unwrap();
        let mut peer = TransactionCoordinator::new(World::new(vec![], &limits).unwrap(), limits).unwrap();
        peer.adopt_external_captures(controller.coordinator()).unwrap();
        let path = std::env::temp_dir().join(format!("om9-save-growth-{}.om9", uuid::Uuid::new_v4()));
        std::fs::write(&path, b"original destination").unwrap();
        let before = controller.coordinator().snapshot();
        let baseline = peer.external_reserved_bytes(MemoryClass::CpuCache);
        let mut competing = None;
        let result = controller.save_archive_for_batch_guard(&path, path.capacity() as u64, || {
            competing = Some(peer.reserve_remaining_external_memory_with_held(
                MemoryClass::Export, None, controller.scene_cache_usage()?)?);
            Ok(())
        });
        assert!(matches!(result, Err(Error::Budget(_))), "{result:?}");
        assert_eq!(std::fs::read(&path).unwrap(), b"original destination");
        assert_eq!(controller.coordinator().snapshot(), before);
        assert_eq!(controller.coordinator().redo_len(), 1);
        assert_eq!(peer.external_reserved_bytes(MemoryClass::CpuCache), baseline - controller.scene.as_ref().unwrap().picking_arena_bytes().unwrap());
        drop(competing);
        controller.redo().unwrap();
        assert_eq!(controller.document().entities.len(), 2);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn open_scene_planning_checks_fresh_physical_headroom_and_preserves_redo() {
        const MIB: u64 = 1024 * 1024;
        let limits = Limits { max_bytes: 64 * MIB, ..Limits::default() };
        let path = std::env::temp_dir().join(format!("om9-open-planning-{}.om9", uuid::Uuid::new_v4()));
        Archive::save(&path, &Document::new("opened"), &limits, &om9_core::transaction::Cancellation::default()).unwrap();
        let mut controller = Controller::new(Document::with_template_tolerance("existing"), limits).unwrap();
        for point in ["1,2,3", "4,5,6"] { controller.submit("Point").unwrap(); controller.submit(point).unwrap(); }
        controller.undo().unwrap();
        controller.begin_file_operation(FileKind::Open).unwrap();
        let token = controller.file_operation().unwrap().token();
        controller.decide_unsaved(token, UnsavedDecision::Discard).unwrap();
        let token = controller.file_operation().unwrap().token();
        let before = controller.coordinator().snapshot();
        let mut sampled = false;
        let baseline = controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache);
        let mut published = false;
        let result = controller.replace_from_file_with_planning_sample(token, path.clone(), || { published = true; Ok(()) }, || {
            sampled = true;
            Some((1024 * MIB, 256 * MIB))
        }, true);
        assert!(sampled, "fixture must reach scene planning");
        assert!(matches!(result, Err(Error::Budget("available physical RAM"))));
        assert!(!published, "refused candidate must not reach publication");
        assert_eq!(controller.coordinator().snapshot(), before);
        assert_eq!(controller.coordinator().redo_len(), 1);
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache), baseline - controller.scene.as_ref().unwrap().picking_arena_bytes().unwrap());
        assert!(matches!(controller.replace_from_file_with_planning_sample(token, path.clone(), || Ok(()), || Some((1024 * MIB, 1024 * MIB)), true), Ok(FileOutcome::Replaced)));
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache),
            controller.scene.as_ref().unwrap().memory_allowance_bytes() + controller.file.storage_usage().unwrap().bytes);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn replacement_inventory_counts_file_path_and_diagnostic_capacity() {
        let limits = Limits::default();
        let mut controller = Controller::new(Document::new("file accounting"), limits).unwrap();
        let before = controller.replacement_hold(Usage::default()).unwrap().bytes;
        controller.file.path = Some(PathBuf::from("a-longer-file-path-for-accounting.om9"));
        controller.file.diagnostics = vec!["one diagnostic".repeat(100)];
        let file = controller.file.usage(&limits).unwrap().bytes;
        let after = controller.replacement_hold(Usage::default()).unwrap().bytes;
        assert!(file >= controller.file.path.as_ref().unwrap().capacity() as u64 + controller.file.diagnostics[0].capacity() as u64);
        assert_eq!(after - before, file);
    }

    #[test]
    fn archive_input_reservation_blocks_competing_peer_until_restore_finishes() {
        const MIB: u64 = 1024 * 1024;
        let limits = Limits { max_bytes: 64 * MIB, ..Limits::default() };
        let mut input = Document::new("archive input");
        let cloud = Entity::new("cloud", Geometry::Cloud(Cloud {
            points: vec![[1., 2., 3.]; 50_000], normals: vec![], colors: vec![], hidden: vec![],
        }), None);
        input.entities.insert(cloud.id, cloud);
        let path = std::env::temp_dir().join(format!("om9-open-input-reservation-{}.om9", uuid::Uuid::new_v4()));
        Archive::save(&path, &input, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
        let controller = Controller::new(Document::new("current"), limits).unwrap();
        let mut peer = TransactionCoordinator::new(World::new(vec![Document::new("peer")], &limits).unwrap(), limits).unwrap();
        peer.adopt_external_captures(controller.coordinator()).unwrap();
        let held = controller.replacement_hold(Usage::default()).unwrap();
        let result = Archive::open_restored_reserved(&path, &limits.remaining_after(held).unwrap(), &Default::default(), |size| {
            assert!(size > 100_000);
            let probe = limits.max_bytes - peer.retained_usage()?.bytes - size / 2 - 8192;
            drop(peer.reserve_external_memory(MemoryClass::Export, probe, None)?);
            let guard = controller.coordinator().reserve_external_memory(MemoryClass::CpuCache, size, None)?;
            assert!(matches!(peer.reserve_external_memory(MemoryClass::Export, probe, None), Err(Error::Budget(_))));
            Ok(guard)
        });
        assert!(result.is_ok(), "archive restore must succeed");
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn new_replacement_refuses_competing_reservation_without_losing_redo() {
        const MIB: u64 = 1024 * 1024;
        let limits = Limits { max_bytes: 64 * MIB, ..Limits::default() };
        let mut document = Document::with_template_tolerance("existing cloud");
        let cloud = Cloud { points: vec![[0., 0., 0.]; 100_000], normals: vec![], colors: vec![], hidden: vec![] };
        let cloud = Entity::new("cloud", Geometry::Cloud(cloud), None);
        document.entities.insert(cloud.id, cloud);
        let mut controller = Controller::new(document, limits).unwrap();
        for point in ["1,2,3", "4,5,6"] { controller.submit("Point").unwrap(); controller.submit(point).unwrap(); }
        controller.undo().unwrap();
        assert_eq!(controller.coordinator().redo_len(), 1);
        controller.begin_file_operation(FileKind::New).unwrap();
        let token = controller.file_operation().unwrap().token();
        let before = controller.coordinator().snapshot();
        let held_before = controller.replacement_hold(Usage::default()).unwrap().bytes;
        let baseline = controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache);
        assert!(held_before > 4 * MIB && held_before < limits.max_bytes / 2);
        let mut peer = TransactionCoordinator::new(World::new(vec![Document::new("peer")], &limits).unwrap(), limits).unwrap();
        peer.adopt_external_captures(controller.coordinator()).unwrap();
        let interrupted = controller.replace_with_new_guards(token, || {
            assert!(peer.external_reserved_bytes(MemoryClass::CpuCache) > 0,
                "New's candidate must be reserved before controller construction");
            Err(Error::Canceled)
        }, || panic!("canceled preparation must not reach publication"));
        assert!(matches!(interrupted, Err(Error::Canceled)));
        assert_eq!(controller.coordinator().snapshot(), before);
        assert_eq!(controller.coordinator().redo_len(), 1);
        assert_eq!(peer.external_reserved_bytes(MemoryClass::CpuCache), baseline);
        let mut reservation = None;
        let result = controller.replace_with_new_guard(token, || {
            assert!(peer.external_reserved_bytes(MemoryClass::CpuCache) > 0,
                "candidate preparation must be reserved before constructing New");
            reservation = Some(peer.reserve_external_memory(MemoryClass::Export, limits.max_bytes - held_before / 2, None)?);
            Ok(())
        });
        assert!(matches!(result, Err(Error::Budget(_))), "New must reject a competing reservation: {result:?}");
        assert_eq!(controller.coordinator().snapshot(), before);
        assert_eq!(controller.coordinator().redo_len(), 1);
        assert_eq!(peer.external_reserved_bytes(MemoryClass::CpuCache), baseline - controller.scene.as_ref().unwrap().picking_arena_bytes().unwrap(),
            "failed New returns its temporary owners and unused cache quota, retaining source/history");
        drop(reservation);
        assert!(matches!(controller.replace_with_new_guard(token, || Ok(())), Ok(FileOutcome::Replaced)));
        assert_eq!(controller.coordinator().redo_len(), 0);
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache), controller.scene.as_ref().unwrap().memory_allowance_bytes(), "New retains only its published scene owner");
    }

    #[test]
    fn open_replacement_refuses_competing_reservation_without_losing_redo() {
        const MIB: u64 = 1024 * 1024;
        let limits = Limits { max_bytes: 64 * MIB, ..Limits::default() };
        let path = std::env::temp_dir().join(format!("om9-replacement-race-{}.om9", uuid::Uuid::new_v4()));
        Archive::save(&path, &Document::new("opened"), &limits, &om9_core::transaction::Cancellation::default()).unwrap();
        let scene_peak = Scene::candidate_scene_peak(&Document::new("opened"), &limits).unwrap();
        let overhead = Controller::candidate_controller_overhead(Usage::default(), &limits).unwrap().bytes;
        let minimum_reserved = std::fs::metadata(&path).unwrap().len() * 2 + scene_peak + overhead;
        let mut document = Document::with_template_tolerance("existing cloud");
        let cloud = Cloud { points: vec![[0., 0., 0.]; 100_000], normals: vec![], colors: vec![], hidden: vec![] };
        let cloud = Entity::new("cloud", Geometry::Cloud(cloud), None);
        document.entities.insert(cloud.id, cloud);
        let mut controller = Controller::new(document, limits).unwrap();
        for point in ["1,2,3", "4,5,6"] { controller.submit("Point").unwrap(); controller.submit(point).unwrap(); }
        controller.undo().unwrap();
        controller.begin_file_operation(FileKind::Open).unwrap();
        let token = controller.file_operation().unwrap().token();
        controller.decide_unsaved(token, UnsavedDecision::Discard).unwrap();
        let token = controller.file_operation().unwrap().token();
        let before = controller.coordinator().snapshot();
        let held_before = controller.replacement_hold(Usage::default()).unwrap().bytes;
        let baseline = controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache);
        let mut peer = TransactionCoordinator::new(World::new(vec![Document::new("peer")], &limits).unwrap(), limits).unwrap();
        peer.adopt_external_captures(controller.coordinator()).unwrap();
        let mut reservation = None;
        let result = controller.replace_from_file_guard(token, path.clone(), || {
            assert!(peer.external_reserved_bytes(MemoryClass::CpuCache) >= minimum_reserved,
                "archive input and candidate scene peak must remain reserved through construction");
            reservation = Some(peer.reserve_external_memory(MemoryClass::Export, limits.max_bytes - held_before / 2, None)?);
            Ok(())
        });
        assert!(matches!(result, Err(Error::Budget(_))), "Open must reject a competing reservation: {result:?}");
        assert_eq!(controller.coordinator().snapshot(), before);
        assert_eq!(controller.coordinator().redo_len(), 1);
        assert_eq!(peer.external_reserved_bytes(MemoryClass::CpuCache), baseline - controller.scene.as_ref().unwrap().picking_arena_bytes().unwrap(),
            "failed Open returns its temporary owners and unused cache quota, retaining source/history");
        drop(reservation);
        assert!(matches!(controller.replace_from_file_guard(token, path.clone(), || Ok(())), Ok(FileOutcome::Replaced)));
        assert_eq!(controller.coordinator().redo_len(), 0);
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache),
            controller.scene.as_ref().unwrap().memory_allowance_bytes() + controller.file.storage_usage().unwrap().bytes,
            "Open retains its published scene and actual file heap owners");
        std::fs::remove_file(path).unwrap();
    }
}
