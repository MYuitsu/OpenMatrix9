// SPDX-License-Identifier: LGPL-2.1-or-later
use super::*;
use crate::file_lifecycle::*;
use om9_archive::Archive;
use om9_core::budget::Usage;
use std::path::{Path, PathBuf};
const PATH_BYTES: usize = 32768;
impl Controller {
    pub fn from_file(path: &Path, limits: Limits) -> Result<Self> {
        let mut controller = Self::new(Document::new("Untitled"), limits)?;
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
        destination: Option<PathBuf>,
    ) {
        self.file.serial = next.serial;
        let operation = self.file.operation.as_mut().expect("active operation");
        operation.token = next;
        operation.phase = phase;
        operation.destination = destination;
    }
    fn advance_file_phase(
        &mut self,
        token: FileToken,
        phase: FilePhase,
        destination: Option<PathBuf>,
    ) -> Result<FileOutcome> {
        let next = self.next_file_token(token)?;
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
        let mut held = self.scene_cache_usage()?;
        held.add(Usage::array::<FileOperation>(1)?, self.coordinator.limits())?;
        held.add(
            Usage {
                bytes: 1024,
                ..Usage::default()
            },
            self.coordinator.limits(),
        )?;
        self.coordinator.retained_usage_with_held_usage(held)?;
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
                }
                result
            }
            FileIntent::Command(FileKind::Save) if self.file.path.is_some() => {
                let result = self
                    .copy_file_path(self.file.path.as_deref().expect("path"))
                    .and_then(|path| self.perform_file_save(token, path));
                if result.is_err() {
                    self.file.operation = None;
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
            let destination = self
                .file
                .operation
                .as_mut()
                .expect("checked")
                .destination
                .take();
            self.publish_file_phase(next, FilePhase::Overwrite, destination);
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
        let limits = *self.coordinator.limits();
        let mut auxiliary = self.scene_cache_usage()?;
        auxiliary.add(
            Usage {
                bytes: path.capacity() as u64,
                ..Usage::default()
            },
            &limits,
        )?;
        let total = self.coordinator.retained_usage_with_held_usage(auxiliary)?;
        // Archive owns the current document once; reserve all other live owners.
        let other = subtract_usage(total, self.document().validate(&limits)?)?;
        Archive::save(
            &path,
            self.document(),
            &limits.remaining_after(other)?,
            &om9_core::transaction::Cancellation::default(),
        )?;
        // Allocate/admit bookkeeping before filesystem acceptance.
        self.file.path = Some(path);
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
                Ok(FileOutcome::Saved)
            }
        }
    }
    fn replacement_hold(&self, extra: Usage) -> Result<Usage> {
        let mut auxiliary = self.scene_cache_usage()?;
        auxiliary.add(extra, self.coordinator.limits())?;
        self.coordinator.retained_usage_with_held_usage(auxiliary)
    }
    fn replace_with_new(&mut self, token: FileToken) -> Result<FileOutcome> {
        self.validate_file_operation(token)?;
        let limits = *self.coordinator.limits();
        let held = self.replacement_hold(Usage {
            bytes: 512,
            ..Usage::default()
        })?;
        let candidate = Self::new_with_held(Document::new("Untitled"), limits, held)?;
        *self = candidate;
        Ok(FileOutcome::Replaced)
    }
    fn replace_from_file(&mut self, token: FileToken, path: PathBuf) -> Result<FileOutcome> {
        self.file_phase(token, FilePhase::OpenPath)?;
        let limits = *self.coordinator.limits();
        let path_usage = Usage {
            bytes: path.capacity() as u64,
            ..Usage::default()
        };
        let held = self.replacement_hold(path_usage)?;
        let restored = Archive::open_restored(
            &path,
            &limits.remaining_after(held)?,
            &std::collections::BTreeSet::new(),
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
        let mut candidate = Self::new_with_held(restored.document, limits, held)?;
        candidate.file.path = Some(path);
        if !restored.diagnostics.is_empty() {
            candidate.file.saved = None;
        }
        candidate.file.diagnostics = restored.diagnostics;
        *self = candidate;
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
        Ok(())
    }
}
fn subtract_usage(total: Usage, owner: Usage) -> Result<Usage> {
    let sub = |a: u64, b: u64| {
        a.checked_sub(b)
            .ok_or(Error::Invalid("file retained owner accounting"))
    };
    Ok(Usage {
        bytes: sub(total.bytes, owner.bytes)?,
        objects: sub(total.objects, owner.objects)?,
        nodes: sub(total.nodes, owner.nodes)?,
        points: sub(total.points, owner.points)?,
        history: sub(total.history, owner.history)?,
        outputs: sub(total.outputs, owner.outputs)?,
    })
}
