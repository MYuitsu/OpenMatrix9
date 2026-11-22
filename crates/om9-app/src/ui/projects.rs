// SPDX-License-Identifier: LGPL-2.1-or-later
//! Compact Matrix Projects: project rows, numbered bags and named master bags.
use super::*;
use crate::project_manager::{BagFilter, DirectoryEntry, DirectoryKind, MASTER_NAMES};

pub(super) struct DirectoryEditor { kind: DirectoryKind, id: Option<u64>, name: String }

fn directory_choice(ui: &mut egui::Ui, label: &str, kind: DirectoryKind, entries: &[DirectoryEntry], id: &mut Option<u64>, name: &mut String) {
    ui.horizontal(|ui| {
        ui.label(label);
        egui::ComboBox::from_id_salt(("project-directory-choice", kind as u8)).selected_text(if id.is_some() { "Record" } else { "Manual" }).show_ui(ui, |ui| {
            if ui.selectable_label(id.is_none(), "Manual entry").clicked() { *id = None; }
            for entry in entries {
                if ui.selectable_label(*id == Some(entry.id), format!("{} ({})", entry.name, entry.id)).clicked() {
                    *id = Some(entry.id); *name = entry.name.clone();
                }
            }
        });
        if ui.text_edit_singleline(name).changed() { *id = None; }
    });
}

const BAG_FILTERS: [(&str, BagFilter); 6] = [
    ("All", BagFilter::All), ("Gems", BagFilter::Gems), ("Curves", BagFilter::Curves),
    ("Surfaces", BagFilter::Surfaces), ("Polysurfaces", BagFilter::Polysurfaces), ("Meshes", BagFilter::Meshes),
];

fn overlay_button(ui: &mut egui::Ui, rect: Rect, id: &'static str, text: &'static str) -> egui::Response {
    let mut child = ui.new_child(egui::UiBuilder::new().id_salt(id).max_rect(rect)
        .layout(egui::Layout::top_down(egui::Align::Min)));
    child.add_sized(rect.size(), egui::Button::new(egui::RichText::new(text).color(Color32::WHITE))
        .fill(Color32::from_rgb(55, 59, 57)).stroke(egui::Stroke::new(1., Color32::from_gray(115))).corner_radius(2))
}

fn jobbag_capture_error(error: om9_core::Error) -> om9_core::Error {
    match error {
        om9_core::Error::Unsupported("BRep archive migration to v5 is not available") =>
            om9_core::Error::Unsupported("This selection cannot be saved in the current Job Bag format"),
        other => other,
    }
}

fn project_timer_text(row: &crate::project_manager::ProjectRow) -> String {
    let elapsed = match (row.timer_started_unix, timer_now()) {
        (Some(_), Ok(now)) => row.elapsed_at(now),
        (Some(_), Err(_)) => return "Timer unavailable".into(),
        (None, _) => row.elapsed_seconds,
    };
    let state = if row.timer_started_unix.is_some() { "running" } else { "stopped" };
    format!("Timer {:02}:{:02}:{:02} ({state})", elapsed / 3600, elapsed / 60 % 60, elapsed % 60)
}
use sha2::{Digest, Sha256};

pub(super) struct ProjectThumbnailWorker {
    receiver: std::sync::mpsc::Receiver<(crate::project_manager::ProjectId, crate::project_manager::SnapshotId, std::sync::Arc<[u8]>, Result<Vec<u8>>, om9_core::transaction::ExternalMemoryReservation)>,
    cancel: om9_core::transaction::Cancellation,
}
impl ProjectThumbnailWorker { pub(super) fn cancel(&self) { self.cancel.cancel(); } }
pub(super) struct ProjectCatalogWorker {
    receiver: std::sync::mpsc::Receiver<(Result<[u8; 32]>, om9_core::transaction::ExternalMemoryReservation)>,
    revision: u64,
    path: std::path::PathBuf,
}
pub(super) struct ProjectExportWorker {
    receiver: std::sync::mpsc::Receiver<(Result<crate::project_catalog::PreparedProjectFile>, om9_core::transaction::ExternalMemoryReservation)>,
    cancel: om9_core::transaction::Cancellation,
    generation: u64,
    revision: u64,
}
impl ProjectExportWorker { pub(super) fn cancel(&self) { self.cancel.cancel(); } }
pub(super) struct ProjectRootWorker {
    receiver: std::sync::mpsc::Receiver<(Result<(crate::project_catalog::PreparedCatalogRoot, Option<crate::project_catalog::SourceCatalogGuard>)>, om9_core::transaction::ExternalMemoryReservation)>,
    cancel: om9_core::transaction::Cancellation,
    generation: u64,
    revision: u64,
    old_path: Option<std::path::PathBuf>,
}
impl ProjectRootWorker { pub(super) fn cancel(&self) { self.cancel.cancel(); } }
/// Keeps a charge while an immutable snapshot archive has any strong owner.
pub(super) struct ProjectAssetLease {
    payload: std::sync::Weak<[u8]>,
    _reservation: om9_core::transaction::ExternalMemoryReservation,
}

pub(super) fn retain_manager_assets(leases: &mut Vec<ProjectAssetLease>, manager: &crate::project_manager::ProjectManager,
    reservation: &mut om9_core::transaction::ExternalMemoryReservation) -> Result<()> {
    let mut pending: Vec<std::sync::Arc<[u8]>> = Vec::new();
    let mut add = |asset: &std::sync::Arc<[u8]>| -> Result<()> {
        let weak = std::sync::Arc::downgrade(asset);
        if leases.iter().any(|lease| lease.payload.ptr_eq(&weak)) || pending.iter().any(|old| std::sync::Arc::ptr_eq(old, asset)) {
            return Ok(());
        }
        pending.try_reserve(1).map_err(|_| om9_core::Error::Budget("project asset owners"))?;
        pending.push(asset.clone());
        Ok(())
    };
    for row in manager.rows() {
        for snapshot in row.slots.iter().filter_map(|slot| slot.snapshot.as_ref()) {
            if let Some(asset) = &snapshot.archive { add(asset)?; }
            if let Some(asset) = &snapshot.thumbnail.png { add(asset)?; }
        }
        for snapshot in row.master.iter().chain(row.named_masters.values()).chain(row.named_versions.values().flatten()) {
            if let Some(asset) = &snapshot.archive { add(asset)?; }
            if let Some(asset) = &snapshot.thumbnail_png { add(asset)?; }
        }
    }
    let overhead = (2 * std::mem::size_of::<usize>()) as u64;
    let total = pending.iter().try_fold(0u64, |sum, asset| sum.checked_add(asset.len() as u64)
        .and_then(|n| n.checked_add(overhead)).ok_or(om9_core::Error::Overflow("project asset owners")))?;
    if total > reservation.bytes() { return Err(om9_core::Error::Budget("project asset owners")); }
    leases.try_reserve(pending.len()).map_err(|_| om9_core::Error::Budget("project asset owners"))?;
    for asset in pending {
        let bytes = (asset.len() as u64).checked_add(overhead).ok_or(om9_core::Error::Overflow("project asset owner"))?;
        let mut owner = reservation.split_off(bytes)?;
        let _ = owner.set_allocated_bytes(bytes);
        leases.push(ProjectAssetLease { payload: std::sync::Arc::downgrade(&asset), _reservation: owner });
    }
    Ok(())
}
pub(super) struct ProjectCatalogLoadWorker {
    receiver: std::sync::mpsc::Receiver<(Result<Option<(crate::project_manager::ProjectManager, [u8; 32])>>, om9_core::transaction::ExternalMemoryReservation)>,
    cancel: om9_core::transaction::Cancellation,
    path: std::path::PathBuf,
    generation: u64,
    revision: u64,
    startup: bool,
}
impl ProjectCatalogLoadWorker { pub(super) fn cancel(&self) { self.cancel.cancel(); } }

type CaptureResult = (om9_exchange::ExportCapture, Vec<om9_core::model::Entity>, Result<Vec<u8>>, om9_core::transaction::ExternalMemoryReservation);
pub(super) struct ProjectCaptureWorker {
    receiver: std::sync::mpsc::Receiver<CaptureResult>,
    cancel: om9_core::transaction::Cancellation,
    project: crate::project_manager::ProjectId,
    slot: usize,
    previous: Option<crate::project_manager::SnapshotId>,
    document: om9_core::identity::DocumentId,
    revision: u64,
    selection_revision: u64,
    generation: u64,
}
impl ProjectCaptureWorker { pub(super) fn cancel(&self) { self.cancel.cancel(); } }
#[cfg(feature = "occt-brep")]
pub(super) struct ProjectBrepInsertWorker {
    receiver: std::sync::mpsc::Receiver<Result<crate::session::PreparedBrepJobbag>>,
    cancel: om9_core::transaction::Cancellation,
    project: crate::project_manager::ProjectId,
    slot: usize,
    snapshot: crate::project_manager::SnapshotId,
    generation: u64,
    revision: u64,
    archive: std::sync::Weak<[u8]>,
    selection_revision: u64,
    sub_revision: u64,
    scene_publication: uuid::Uuid,
    #[cfg(feature = "test-fixtures")]
    ready: std::sync::Arc<std::sync::atomic::AtomicBool>,
}
#[cfg(feature = "occt-brep")]
impl Drop for ProjectBrepInsertWorker { fn drop(&mut self) { self.cancel.cancel(); } }
#[cfg(feature = "occt-brep")]
impl ProjectBrepInsertWorker { pub(super) fn cancel(&self) { self.cancel.cancel(); } }
#[cfg(feature = "occt-brep")]
pub(super) struct ProjectBrepOpenWorker {
    receiver: std::sync::mpsc::Receiver<Result<crate::session::PreparedNamedBrepOpen>>,
    cancel: om9_core::transaction::Cancellation,
    project: crate::project_manager::ProjectId,
    name: &'static str,
    snapshot: crate::project_manager::SnapshotId,
    generation: u64,
    token: crate::file_lifecycle::FileToken,
    revision: u64,
    selection_revision: u64,
    sub_revision: u64,
    scene_publication: uuid::Uuid,
    #[cfg(feature = "test-fixtures")]
    ready: std::sync::Arc<std::sync::atomic::AtomicBool>,
}
#[cfg(feature = "occt-brep")]
impl Drop for ProjectBrepOpenWorker { fn drop(&mut self) { self.cancel.cancel(); } }
#[cfg(feature = "occt-brep")]
impl ProjectBrepOpenWorker { pub(super) fn cancel(&self) { self.cancel.cancel(); } }
type DocumentCaptureResult = (om9_exchange::ExportCapture, usize, usize, Result<Vec<u8>>, om9_core::transaction::ExternalMemoryReservation);
pub(super) struct ProjectDocumentWorker {
    receiver: std::sync::mpsc::Receiver<DocumentCaptureResult>,
    cancel: om9_core::transaction::Cancellation,
    project: crate::project_manager::ProjectId,
    name: &'static str,
    previous: Option<crate::project_manager::SnapshotId>,
    replace: Option<crate::project_manager::SnapshotId>,
    document: om9_core::identity::DocumentId,
    revision: u64,
    generation: u64,
}
impl ProjectDocumentWorker { pub(super) fn cancel(&self) { self.cancel.cancel(); } }

enum ProjectImportResult {
    Imported(crate::project_manager::ProjectManager, Vec<std::fs::File>),
    Prepared(Vec<std::fs::File>, u64),
}
enum ProjectImportSource { Folder(std::path::PathBuf), Files(Vec<std::path::PathBuf>) }
pub(super) struct ProjectImportWorker {
    receiver: std::sync::mpsc::Receiver<(Result<ProjectImportResult>, om9_core::transaction::ExternalMemoryReservation)>,
    cancel: om9_core::transaction::Cancellation,
    revision: u64,
    generation: u64,
    embedded_source: Option<(om9_core::identity::DocumentId, u64)>,
    #[cfg(feature = "test-fixtures")]
    ready: std::sync::Arc<std::sync::atomic::AtomicBool>,
    #[cfg(feature = "test-fixtures")]
    preparing: bool,
}
impl ProjectImportWorker { pub(super) fn cancel(&self) { self.cancel.cancel(); } }

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ProjectFilePreview {
    Numbered(usize, crate::project_manager::SnapshotId),
    Named(&'static str, crate::project_manager::SnapshotId),
}

impl ProjectFilePreview {
    fn snapshot_id(self) -> crate::project_manager::SnapshotId {
        match self { Self::Numbered(_, id) | Self::Named(_, id) => id }
    }
}

#[derive(Clone, Default)]
pub(super) struct ProjectFilters {
    name: String, customer: String, designer: String, status: String, tag: String, job_number: String,
    created_from: String, created_to: String, due_from: String, due_to: String,
    metal_min: String, metal_max: String, gem_min: String, gem_max: String,
    cad_min: String, cad_max: String, cam_min: String, cam_max: String,
    radius_min: String, radius_max: String, visibility: Option<bool>,
}
struct CompiledProjectFilters {
    name: String, customer: String, designer: String, status: String, tag: String, job_number: String,
    created: (Option<String>, Option<String>), due: (Option<String>, Option<String>),
    metal: (Option<f64>, Option<f64>), gem: (Option<f64>, Option<f64>),
    cad: (Option<f64>, Option<f64>), cam: (Option<f64>, Option<f64>),
    radius: (Option<f64>, Option<f64>), visibility: Option<bool>,
}
impl ProjectFilters {
    fn compile(&self) -> Result<CompiledProjectFilters> {
        fn date(value: &str) -> Result<Option<String>> {
            let value = value.trim();
            if value.is_empty() { return Ok(None); }
            if !crate::project_manager::valid_calendar_date(value) { return Err(om9_core::Error::Invalid("search date")); }
            Ok(Some(value.to_owned()))
        }
        fn number(value: &str) -> Result<Option<f64>> {
            let value = value.trim();
            if value.is_empty() { return Ok(None); }
            let value = value.parse::<f64>().map_err(|_| om9_core::Error::Invalid("search measurement"))?;
            if !value.is_finite() || value < 0.0 { return Err(om9_core::Error::Invalid("search measurement")); }
            Ok(Some(value))
        }
        fn ordered<T: PartialOrd>(range: (Option<T>, Option<T>)) -> Result<(Option<T>, Option<T>)> {
            if range.0.as_ref().zip(range.1.as_ref()).is_some_and(|(min, max)| min > max) {
                return Err(om9_core::Error::Invalid("search range"));
            }
            Ok(range)
        }
        Ok(CompiledProjectFilters {
            name: self.name.trim().to_lowercase(), customer: self.customer.trim().to_lowercase(),
            designer: self.designer.trim().to_lowercase(), status: self.status.trim().to_lowercase(),
            tag: self.tag.trim().to_lowercase(), job_number: self.job_number.trim().to_lowercase(),
            created: ordered((date(&self.created_from)?, date(&self.created_to)?))?,
            due: ordered((date(&self.due_from)?, date(&self.due_to)?))?,
            metal: ordered((number(&self.metal_min)?, number(&self.metal_max)?))?,
            gem: ordered((number(&self.gem_min)?, number(&self.gem_max)?))?,
            cad: ordered((number(&self.cad_min)?, number(&self.cad_max)?))?,
            cam: ordered((number(&self.cam_min)?, number(&self.cam_max)?))?,
            radius: ordered((number(&self.radius_min)?, number(&self.radius_max)?))?,
            visibility: self.visibility,
        })
    }
}
impl CompiledProjectFilters {
    fn matches(&self, row: &crate::project_manager::ProjectRow) -> bool {
        fn range<T: PartialOrd>(value: Option<&T>, limits: &(Option<T>, Option<T>)) -> bool {
            if limits.0.is_none() && limits.1.is_none() { return true; }
            value.is_some_and(|value| limits.0.as_ref().is_none_or(|min| value >= min)
                && limits.1.as_ref().is_none_or(|max| value <= max))
        }
        row.name.to_lowercase().contains(&self.name)
            && row.customer.to_lowercase().contains(&self.customer)
            && row.metadata.designer.to_lowercase().contains(&self.designer)
            && row.metadata.status.to_lowercase().contains(&self.status)
            && row.metadata.job_number.to_lowercase().contains(&self.job_number)
            && (self.tag.is_empty() || row.metadata.tags.iter().any(|tag| tag.to_lowercase().contains(&self.tag)))
            && range(row.metadata.created_date.as_ref(), &self.created)
            && range(row.metadata.due_date.as_ref(), &self.due)
            && range(row.metadata.metal_weight_grams.as_ref(), &self.metal)
            && range(row.metadata.gem_weight_carats.as_ref(), &self.gem)
            && range(row.metadata.cad_labor_hours.as_ref(), &self.cad)
            && range(row.metadata.cam_labor_hours.as_ref(), &self.cam)
            && range(row.metadata.ring_rail_radius_mm.as_ref(), &self.radius)
            && self.visibility.is_none_or(|visible| row.visible == visible)
    }
}

pub(super) struct ProjectDraft {
    id: crate::project_manager::ProjectId,
    generation: u64,
    name: String,
    customer: String,
    notes: String,
    metadata: crate::project_manager::ProjectMetadata,
    tags: String,
    created: String,
    due: String,
}
impl ProjectDraft {
    fn from_row(row: &crate::project_manager::ProjectRow, generation: u64) -> Self {
        Self {
            id: row.id, generation, name: row.name.clone(), customer: row.customer.clone(), notes: row.notes.clone(),
            tags: row.metadata.tags.join(", "),
            created: row.metadata.created_date.clone().unwrap_or_default(),
            due: row.metadata.due_date.clone().unwrap_or_default(),
            metadata: row.metadata.clone(),
        }
    }
    fn prepare_metadata(&self) -> crate::project_manager::ProjectMetadata {
        let mut metadata = self.metadata.clone();
        metadata.tags = self.tags.split(',').map(str::trim).filter(|tag| !tag.is_empty()).map(str::to_owned).collect();
        metadata.created_date = (!self.created.trim().is_empty()).then(|| self.created.trim().to_owned());
        metadata.due_date = (!self.due.trim().is_empty()).then(|| self.due.trim().to_owned());
        metadata
    }

    fn sync_directory(&mut self, manager: &crate::project_manager::ProjectManager) {
        fn sync(entries: &[DirectoryEntry], id: &mut Option<u64>, name: &mut String) {
            if let Some(current) = *id {
                if let Some(entry) = entries.iter().find(|entry| entry.id == current) {
                    name.clone_from(&entry.name);
                } else {
                    *id = None;
                }
            }
        }
        sync(manager.directory_entries(DirectoryKind::Customer), &mut self.metadata.customer_id, &mut self.customer);
        sync(manager.directory_entries(DirectoryKind::Designer), &mut self.metadata.designer_id, &mut self.metadata.designer);
        sync(manager.directory_entries(DirectoryKind::Status), &mut self.metadata.status_id, &mut self.metadata.status);
    }
}

fn optional_measure(ui: &mut egui::Ui, label: &str, value: &mut Option<f64>) {
    ui.horizontal(|ui| {
        let mut enabled = value.is_some();
        if ui.checkbox(&mut enabled, label).changed() {
            *value = enabled.then_some(0.0);
        }
        if let Some(value) = value { ui.add(egui::DragValue::new(value).range(0.0..=1.0e12).speed(0.1)); }
    });
}

fn filter_range(ui: &mut egui::Ui, label: &str, min: &mut String, max: &mut String) -> (Rect, Rect) {
    ui.vertical(|ui| {
        ui.label(label);
        let width = ((ui.available_width() - ui.spacing().item_spacing.x) * 0.5).max(1.);
        ui.horizontal(|ui| {
            let min = ui.add(egui::TextEdit::singleline(min).desired_width(width).hint_text("from"));
            let max = ui.add(egui::TextEdit::singleline(max).desired_width(width).hint_text("to"));
            (min.rect, max.rect)
        }).inner
    }).inner
}

#[derive(Clone, Copy)]
pub(super) enum ProjectConfirmation {
    Delete(crate::project_manager::ProjectId),
    ResetTimer(crate::project_manager::ProjectId),
    ClearSlot(crate::project_manager::ProjectId, usize, crate::project_manager::SnapshotId),
    ClearVersion(crate::project_manager::ProjectId, &'static str, crate::project_manager::SnapshotId),
    OverwriteSlot(crate::project_manager::ProjectId, usize, crate::project_manager::SnapshotId, om9_core::identity::DocumentId, u64, u64, BagFilter),
    OverwriteVersion(crate::project_manager::ProjectId, &'static str, crate::project_manager::SnapshotId, om9_core::identity::DocumentId, u64),
    OverwriteFile,
    ReloadCatalog,
    RemoveDirectory(DirectoryKind, u64),
    CloseUnsavedCatalog(u64),
}

#[derive(Clone, Copy)]
pub(super) enum ProjectFileAction {
    ImportRow,
    ImportFolder,
    ChangeRoot,
    ExportRow(crate::project_manager::ProjectId),
    ExportSlot(crate::project_manager::ProjectId, usize, crate::project_manager::SnapshotId),
    ExportVersion(crate::project_manager::ProjectId, &'static str, crate::project_manager::SnapshotId),
}

fn timer_now() -> Result<u64> {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|_| om9_core::Error::Invalid("project timer clock"))
}

impl MatrixApp {
    #[cfg(feature = "test-fixtures")]
    pub fn test_start_project_import(&mut self, files: Vec<std::path::PathBuf>) -> Result<()> { self.start_project_import(files) }

    #[cfg(feature = "test-fixtures")]
    pub fn test_import_project_folder(&mut self, path: &std::path::Path) -> Result<()> {
        self.execute_project_file(ProjectFileAction::ImportFolder, path)
    }

    #[cfg(feature = "test-fixtures")]
    pub fn test_export_project(&mut self, path: &std::path::Path) -> Result<()> {
        self.execute_project_file(ProjectFileAction::ExportRow(self.project_manager.active().id), path)
    }

    #[cfg(feature = "test-fixtures")]
    pub fn test_export_jobbag(&mut self, slot: usize, path: &std::path::Path) -> Result<()> {
        let project = self.project_manager.active().id;
        let snapshot = self.project_manager.active().slot(slot).and_then(|slot| slot.snapshot.as_ref())
            .ok_or(om9_core::Error::Missing("jobbag slot"))?.id;
        self.execute_project_file(ProjectFileAction::ExportSlot(project, slot, snapshot), path)
    }

    #[cfg(feature = "test-fixtures")]
    pub fn test_export_named_jobbag(&mut self, name: &'static str, path: &std::path::Path) -> Result<()> {
        let project = self.project_manager.active().id;
        let snapshot = self.project_manager.named_master_snapshot(name).ok_or(om9_core::Error::Missing("named jobbag"))?.id;
        self.execute_project_file(ProjectFileAction::ExportVersion(project, name, snapshot), path)
    }

    #[cfg(feature = "test-fixtures")]
    pub fn test_project_export_pending(&self) -> bool { self.project_export_worker.is_some() }

    #[cfg(feature = "test-fixtures")]
    pub fn test_start_project_root_change(&mut self, root: &std::path::Path, settings: &std::path::Path) -> Result<()> {
        self.start_project_root_change(root, settings)
    }

    #[cfg(feature = "test-fixtures")]
    pub fn test_project_root_pending(&self) -> bool { self.project_root_worker.is_some() }

    #[cfg(feature = "test-fixtures")]
    pub fn test_cancel_project_root(&self) { if let Some(worker) = &self.project_root_worker { worker.cancel(); } }

    #[cfg(feature = "test-fixtures")]
    pub fn test_cancel_project_export(&self) { if let Some(worker) = &self.project_export_worker { worker.cancel(); } }

    #[cfg(feature = "test-fixtures")]
    pub fn test_take_project_file_request(&mut self) -> bool { self.project_file_request.take().is_some() }

    #[cfg(feature = "test-fixtures")]
    pub fn test_take_current_project_file_request(&mut self) -> Result<bool> {
        self.take_current_project_file_request().map(|request| request.is_some())
    }

    #[cfg(feature = "test-fixtures")]
    pub fn test_complete_project_file_request(&mut self, path: &std::path::Path) -> Result<()> {
        let action = self.take_current_project_file_request()?.ok_or(om9_core::Error::Missing("project file request"))?;
        self.execute_project_file(action, path)
    }

    #[cfg(feature = "test-fixtures")]
    pub fn test_take_project_file_request_kind(&mut self) -> Option<&'static str> {
        self.project_file_request.take().map(|(action, _)| match action {
            ProjectFileAction::ImportRow => "import-project",
            ProjectFileAction::ImportFolder => "import-folder",
            ProjectFileAction::ChangeRoot => "change-root",
            ProjectFileAction::ExportRow(_) => "export-project",
            ProjectFileAction::ExportSlot(..) => "export-slot",
            ProjectFileAction::ExportVersion(..) => "export-version",
        })
    }

    #[cfg(feature = "test-fixtures")]
    pub fn test_start_embedded_project_import(&mut self) -> Result<()> { self.start_embedded_project_import() }

    #[cfg(feature = "test-fixtures")]
    pub fn test_project_import_pending(&self) -> bool { self.project_import_worker.is_some() }

    #[cfg(feature = "test-fixtures")]
    pub fn test_project_import_result_ready(&self) -> bool {
        self.project_import_worker.as_ref().is_some_and(|worker| worker.ready.load(std::sync::atomic::Ordering::Acquire))
    }

    #[cfg(feature = "test-fixtures")]
    pub fn test_project_import_preparing(&self) -> bool {
        self.project_import_worker.as_ref().is_some_and(|worker| worker.preparing)
    }

    #[cfg(feature = "test-fixtures")]
    pub fn test_service_project_import(&mut self, context: &egui::Context) { self.update_project_import(context); }

    #[cfg(feature = "test-fixtures")]
    pub fn test_project_catalog_load_pending(&self) -> bool { self.project_catalog_load_worker.is_some() }

    #[cfg(feature = "test-fixtures")]
    pub fn test_start_project_catalog_at(&mut self, path: std::path::PathBuf) -> Result<()> {
        self.project_catalog_path = Some(path.clone());
        self.start_project_catalog_load(path, true)
    }

    #[cfg(feature = "test-fixtures")]
    pub fn test_set_unpersisted_catalog_path(&mut self, path: std::path::PathBuf) {
        self.project_catalog_path = Some(path);
        self.project_catalog_fingerprint = None;
    }

    #[cfg(feature = "test-fixtures")]
    pub fn test_project_catalog_save_pending(&self) -> bool { self.project_catalog_worker.is_some() }

    #[cfg(feature = "test-fixtures")]
    pub fn test_cancel_project_import(&self) {
        if let Some(worker) = &self.project_import_worker { worker.cancel(); }
    }

    #[cfg(feature = "test-fixtures")]
    pub fn test_start_jobbag_capture(&mut self, slot: usize) -> Result<()> { self.capture_project_slot(slot, BagFilter::All) }

    #[cfg(feature = "test-fixtures")]
    pub fn test_capture_filtered_jobbag(&mut self, slot: usize, filter: BagFilter) -> Result<()> { self.capture_project_slot(slot, filter) }

    #[cfg(feature = "test-fixtures")]
    pub fn test_insert_filtered_jobbag(&mut self, slot: usize, filter: BagFilter) -> Result<()> { self.insert_project_slot(slot, filter) }

    #[cfg(feature = "test-fixtures")]
    pub fn test_start_named_capture(&mut self, name: &'static str, replace: Option<crate::project_manager::SnapshotId>) -> Result<()> {
        self.start_document_snapshot(name, replace)
    }

    #[cfg(feature = "test-fixtures")]
    pub fn test_begin_named_open(&mut self, name: &'static str, snapshot: crate::project_manager::SnapshotId) -> Result<()> {
        self.begin_named_jobbag_open(name, snapshot)
    }

    #[cfg(feature = "test-fixtures")]
    pub fn test_service_named_open(&mut self, context: &egui::Context) {
        self.file_ui(context);
        self.update_project_brep_open(context);
    }

    #[cfg(all(feature = "test-fixtures", feature = "occt-brep"))]
    pub fn test_named_open_result_ready(&self) -> bool {
        self.project_brep_open_worker.as_ref().is_some_and(|worker| worker.ready.load(std::sync::atomic::Ordering::Acquire))
    }

    #[cfg(all(feature = "test-fixtures", feature = "occt-brep"))]
    pub fn test_brep_insert_result_ready(&self) -> bool {
        self.project_brep_insert_worker.as_ref().is_some_and(|worker| worker.ready.load(std::sync::atomic::Ordering::Acquire))
    }

    #[cfg(feature = "test-fixtures")]
    pub fn test_capture_pending(&self) -> bool { self.project_capture_worker.is_some() || self.project_document_worker.is_some() || self.project_brep_insert_pending() || self.project_brep_open_pending() }

    #[cfg(feature = "test-fixtures")]
    pub fn test_project_asset_lease_count(&self) -> usize { self.project_asset_leases.len() }

    #[cfg(feature = "test-fixtures")]
    pub fn test_cancel_capture(&self) {
        if let Some(worker) = &self.project_capture_worker { worker.cancel(); }
        if let Some(worker) = &self.project_document_worker { worker.cancel(); }
        #[cfg(feature = "occt-brep")]
        if let Some(worker) = &self.project_brep_insert_worker { worker.cancel.cancel(); }
        #[cfg(feature = "occt-brep")]
        if let Some(worker) = &self.project_brep_open_worker { worker.cancel.cancel(); }
    }

    #[cfg(feature = "test-fixtures")]
    pub fn test_service_capture(&mut self, context: &egui::Context) {
        self.update_project_capture(context);
        self.update_project_document_capture(context);
        self.update_project_brep_insert(context);
        self.update_project_brep_open(context);
    }

    #[cfg(feature = "test-fixtures")]
    pub fn test_service_thumbnails(&mut self, context: &egui::Context, allow_start: bool) { self.update_project_thumbnails(context, allow_start); }

    #[cfg(feature = "test-fixtures")]
    pub fn test_thumbnail_pending(&self) -> bool { self.project_thumbnail_worker.is_some() }

    #[cfg(feature = "test-fixtures")]
    pub fn test_manager_preview_is_numbered(&self, slot: usize) -> bool {
        matches!(self.project_file_preview, Some((_, ProjectFilePreview::Numbered(index, _))) if index == slot)
    }

    #[cfg(feature = "test-fixtures")]
    pub fn test_thumbnail_succeeded(&self) -> Option<bool> { self.project_test_thumbnail_succeeded }

    #[cfg(feature = "test-fixtures")]
    pub fn test_thumbnail_attempted(&self, project: crate::project_manager::ProjectId, snapshot: crate::project_manager::SnapshotId) -> bool {
        self.project_thumbnail_attempts.contains_key(&(project, snapshot.0))
    }
    pub(super) fn project_asset_is_leased(&self, asset: &std::sync::Arc<[u8]>) -> bool {
        let weak = std::sync::Arc::downgrade(asset);
        self.project_asset_leases.iter().any(|lease| lease.payload.ptr_eq(&weak)
            && lease._reservation.belongs_to(&self.controller.coordinator().external_memory_account()))
    }
    pub(super) fn project_brep_insert_pending(&self) -> bool {
        #[cfg(feature = "occt-brep")]
        { self.project_brep_insert_worker.is_some() }
        #[cfg(not(feature = "occt-brep"))]
        { false }
    }
    pub(super) fn project_brep_open_pending(&self) -> bool {
        #[cfg(feature = "occt-brep")]
        { self.project_brep_open_worker.is_some() }
        #[cfg(not(feature = "occt-brep"))]
        { false }
    }

    #[cfg(feature = "occt-brep")]
    pub(super) fn start_named_brep_open(&mut self, token: crate::file_lifecycle::FileToken,
        project: crate::project_manager::ProjectId, name: &'static str,
        snapshot: crate::project_manager::SnapshotId, archive: std::sync::Arc<[u8]>) -> Result<()> {
        use om9_core::budget::memory::MemoryClass;
        if self.project_brep_open_worker.is_some() || self.project_brep_insert_worker.is_some()
            || self.project_document_worker.is_some() || self.project_capture_worker.is_some() {
            return Err(om9_core::Error::Busy);
        }
        self.controller.validate_file_operation(token)?;
        let held = self.controller.scene_cache_usage()?;
        let replacement_held = self.controller.named_brep_replacement_hold()?;
        let scene_publication = self.controller.scene().ok_or(om9_core::Error::Missing("project open scene"))?.publication_id();
        let coordinator = self.controller.coordinator();
        let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
        let input = coordinator.reserve_external_memory_with_allocated(MemoryClass::CpuCache,
            (archive.len() as u64).max(1), archive.len() as u64, physical, held)?;
        let manager_bytes = self.project_manager.storage_bytes_estimate(coordinator.limits())?.checked_mul(3)
            .and_then(|bytes| bytes.checked_add(1024 * 1024)).ok_or(om9_core::Error::Overflow("BRep named snapshot manager"))?;
        let manager_owner = coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache, manager_bytes.max(1), physical, held)?;
        let manager = self.project_manager.clone();
        let peer = coordinator.capture_transaction_preparation_peer(held)?;
        let host = crate::brep_restore_host::BrepRestoreHost::admit_packaged(om9_native_owner::ProcessLimits::default())?;
        let cancel = om9_core::transaction::Cancellation::default();
        let worker_cancel = cancel.clone();
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        #[cfg(feature = "test-fixtures")]
        let ready = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        #[cfg(feature = "test-fixtures")]
        let worker_ready = ready.clone();
        let worker = ProjectBrepOpenWorker { receiver, cancel, project, name, snapshot,
            generation: self.project_manager_generation, revision: self.project_manager.change_revision(), token,
            selection_revision: self.controller.selection_state().revision, sub_revision: self.controller.sub_revision(), scene_publication,
            #[cfg(feature = "test-fixtures")] ready };
        std::thread::Builder::new().name("om9-brep-named-open".into()).spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let bound = host.restore_bound_archive(peer.coordinator(), &archive, true, held, &worker_cancel)?;
                let (bound, proof) = crate::session::project_state::prepare_bound_jobbag_project(peer.coordinator(), bound, &manager, manager_owner, held, &worker_cancel)?;
                crate::session::Controller::prepare_named_brep_replacement(peer.coordinator(), bound, proof, archive, input,
                    replacement_held, held, &worker_cancel)
            })).unwrap_or_else(|_| Err(om9_core::Error::Io("BRep named snapshot worker panicked".into())));
            if result.is_err() {
                peer.coordinator().prune_orphaned_brep_representations();
                peer.coordinator().prune_orphaned_native_representations();
            }
            #[cfg(feature = "test-fixtures")]
            let prepared = result.is_ok();
            if let Err(unsent) = sender.send(result) {
                drop(unsent);
                peer.coordinator().prune_orphaned_brep_representations();
            }
            #[cfg(feature = "test-fixtures")]
            worker_ready.store(prepared, std::sync::atomic::Ordering::Release);
        }).map_err(|error| om9_core::Error::Io(format!("BRep named snapshot worker start: {error}")))?;
        self.project_brep_open_worker = Some(worker);
        self.status.info("Opening named snapshot…");
        Ok(())
    }

    #[cfg(feature = "occt-brep")]
    pub(super) fn update_project_brep_open(&mut self, context: &egui::Context) {
        let Some(worker) = self.project_brep_open_worker.take() else { return; };
        let result = match worker.receiver.try_recv() {
            Ok(result) => result,
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                self.project_brep_open_worker = Some(worker);
                context.request_repaint_after(std::time::Duration::from_millis(100));
                return;
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => Err(om9_core::Error::Io("BRep named snapshot worker exited".into())),
        };
        let current = self.project_manager.active().id == worker.project && self.project_manager_generation == worker.generation
            && self.project_manager.change_revision() == worker.revision
            && self.project_manager.named_master_by_id(worker.name, worker.snapshot).is_some()
            && self.controller.file_operation().is_some_and(|operation| operation.token() == worker.token
                && operation.phase() == crate::file_lifecycle::FilePhase::OpenPath)
            && self.controller.validate_file_operation(worker.token).is_ok()
            && self.controller.scene().is_some_and(|scene| scene.publication_id() == worker.scene_publication)
            && self.controller.selection_state().revision == worker.selection_revision && self.controller.sub_revision() == worker.sub_revision;
        let outcome = match result {
            Ok(prepared) if worker.cancel.is_canceled() || !current => {
                drop(prepared);
                self.controller.coordinator().prune_orphaned_brep_representations();
                Err(if worker.cancel.is_canceled() { om9_core::Error::Canceled } else { om9_core::Error::Stale("named BRep snapshot changed") })
            }
            Ok(prepared) => {
                let summary = self.project_manager.named_master_by_id(worker.name, worker.snapshot).expect("checked");
                if summary.object_count.is_some_and(|count| count != prepared.object_count())
                    || summary.layer_count.is_some_and(|count| count != prepared.layer_count()) {
                    drop(prepared);
                    self.controller.coordinator().prune_orphaned_brep_representations();
                    Err(om9_core::Error::Invalid("named jobbag archive summary"))
                } else {
                    self.controller.cancel_file_operation(worker.token)
                        .and_then(|_| self.controller.publish_named_brep_replacement(prepared, &worker.cancel))
                }
            }
            Err(error) => Err(error),
        };
        if outcome.is_err() && self.controller.file_operation().is_some_and(|operation| operation.token() == worker.token) {
            let _ = self.controller.cancel_file_operation(worker.token);
        }
        if outcome.is_err() { self.controller.coordinator().prune_orphaned_brep_representations(); }
        let _ = self.file_feedback(context, outcome.map(|_| crate::file_lifecycle::FileOutcome::Replaced));
        context.request_repaint();
    }

    #[cfg(not(feature = "occt-brep"))]
    pub(super) fn update_project_brep_open(&mut self, _context: &egui::Context) {}
    fn project_icon_button(&mut self, ui: &mut egui::Ui, name: &str, size: Vec2, enabled: bool, tooltip: &str) -> egui::Response {
        let response = ui.add_enabled(enabled, egui::Button::new("").min_size(size)
            .fill(Color32::from_rgb(55, 59, 57)).stroke(egui::Stroke::new(1., Color32::from_gray(110))).corner_radius(2));
        let response = if enabled { response.on_hover_text(tooltip) } else { response.on_disabled_hover_text(tooltip) };
        self.icons.paint(ui, response.rect.shrink(3.), name);
        if !enabled { ui.painter().rect_filled(response.rect, 2, Color32::from_black_alpha(110)); }
        response
    }
    pub(super) fn apply_project_action(&mut self, action: crate::project_manager::ProjectAction) {
        use crate::project_manager::ProjectAction;
        let section = crate::dock_layout::SectionId::Projects;
        self.dock.set_section_visible(section, true);
        self.dock.set_section_collapsed(section, false);
        self.dock_dirty = true;
        if action == ProjectAction::OpenManager {
            self.project_manager_open = true;
        }
    }
    pub(super) fn projects_ui(&mut self, ui: &mut egui::Ui) {
        self.update_project_thumbnails(ui.ctx(), true);
        self.project_row_rects.clear();
        self.jobbag_slot_rects.clear();
        self.project_action_rects.clear();
        self.project_bag_rects.clear();
        egui::Frame::new()
            .fill(super::theme::BODY)
            .inner_margin(2.)
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = Vec2::new(3., 3.);
                ui.spacing_mut().button_padding = Vec2::new(2., 0.);
                ui.spacing_mut().scroll = egui::style::ScrollStyle::solid();
                ui.spacing_mut().scroll.bar_width = 10.;
                ui.spacing_mut().scroll.bar_inner_margin = 2.;
                ui.style_mut().override_font_id = Some(egui::FontId::proportional(10.));
                if let Some(worker) = &self.project_capture_worker {
                    ui.horizontal(|ui| {
                        ui.label("Capturing bag…");
                        if ui.small_button("Cancel").clicked() { worker.cancel.cancel(); }
                    });
                }
                #[cfg(feature = "occt-brep")]
                if let Some(worker) = &self.project_brep_insert_worker {
                    ui.horizontal(|ui| {
                        ui.label("Restoring bag…");
                        if ui.small_button("Cancel").clicked() { worker.cancel.cancel(); }
                    });
                }
                #[cfg(feature = "occt-brep")]
                if let Some(worker) = &self.project_brep_open_worker {
                    ui.horizontal(|ui| {
                        ui.label("Opening snapshot…");
                        if ui.small_button("Cancel").clicked() { worker.cancel.cancel(); }
                    });
                }
                if let Some(worker) = &self.project_document_worker {
                    ui.horizontal(|ui| {
                        ui.label("Capturing document…");
                        if ui.small_button("Cancel").clicked() { worker.cancel.cancel(); }
                    });
                }
                if let Some(worker) = &self.project_import_worker {
                    ui.horizontal(|ui| {
                        ui.label("Importing projects…");
                        if ui.small_button("Cancel").clicked() { worker.cancel.cancel(); }
                    });
                }
                if let Some(worker) = &self.project_export_worker {
                    ui.horizontal(|ui| {
                        ui.label("Exporting archive…");
                        if ui.small_button("Cancel").clicked() { worker.cancel(); }
                    });
                }
                if let Some(worker) = &self.project_root_worker {
                    ui.horizontal(|ui| {
                        ui.label("Changing project folder…");
                        if ui.small_button("Cancel").clicked() { worker.cancel(); }
                    });
                }
                if let Some(worker) = &self.project_catalog_load_worker {
                    ui.horizontal(|ui| {
                        ui.label("Loading projects…");
                        if ui.small_button("Cancel").clicked() { worker.cancel.cancel(); }
                    });
                }
                ui.horizontal_top(|ui| {
                    let row_width = (ui.available_width() * 0.32).clamp(62., 110.);
                    ui.allocate_ui_with_layout(Vec2::new(row_width, 120.), egui::Layout::top_down(egui::Align::Min), |ui| {
                        egui::ScrollArea::vertical().id_salt("project-rows").max_height(88.).min_scrolled_height(88.).show(ui, |ui| {
                            let active = self.project_manager.active().id;
                            let rows: Vec<_> = self.project_manager.rows().iter().filter(|row| row.visible).map(|row| (row.id, row.name.clone(), project_timer_text(row), !row.has_legacy_bags())).collect();
                            if self.project_manager.rows().iter().any(|row| row.visible && row.timer_started_unix.is_some()) {
                                ui.ctx().request_repaint_after(std::time::Duration::from_secs(1));
                            }
                            for (id, name, timer, exportable) in rows {
                                let response = ui.add_enabled_ui(self.project_draft.is_none(), |ui| {
                                    ui.add_sized([(row_width - 6.).max(1.), 16.], egui::Button::new(name).selected(id == active))
                                }).inner.on_hover_text(timer.as_str());
                                self.project_row_rects.insert(id, response.rect);
                                if response.clicked() {
                                    let result = self.project_manager.select(id);
                                    self.feedback(result);
                                }
                                response.context_menu(|ui| {
                                    ui.label(&timer);
                                    ui.separator();
                                    if ui.add_enabled(exportable, egui::Button::new("Export project…"))
                                        .on_disabled_hover_text("Legacy bags need native archives before project export").clicked() {
                                        self.project_file_request = Some((ProjectFileAction::ExportRow(id), self.project_manager_generation)); ui.close();
                                    }
                                    if ui.button("Import project…").clicked() {
                                        self.project_file_request = Some((ProjectFileAction::ImportRow, self.project_manager_generation)); ui.close();
                                    }
                                    if ui.button("Rename / Info…").clicked() {
                                        let result = if self.project_draft.is_some() { Err(om9_core::Error::Busy) }
                                            else { self.project_manager.select(id) };
                                        if result.is_ok() {
                                            self.apply_project_action(crate::project_manager::ProjectAction::OpenManager);
                                            self.project_draft = Some(ProjectDraft::from_row(self.project_manager.active(), self.project_manager_generation));
                                        }
                                        self.feedback(result); ui.close();
                                    }
                                    if ui.button("Update catalog").clicked() {
                                        self.project_catalog_attempted = None;
                                        self.persist_project_catalog(); ui.close();
                                    }
                                    if ui.button("Compact bags").clicked() {
                                        let result = self.project_manager.select(id);
                                        if result.is_ok() { self.project_manager.compact_slots(); }
                                        self.feedback(result);
                                        ui.close();
                                    }
                                    if ui.button("Start timer").clicked() {
                                        let result = timer_now().and_then(|now| self.project_manager.timer_start(id, now));
                                        self.feedback(result); ui.close();
                                    }
                                    if ui.button("Stop timer").clicked() {
                                        let result = timer_now().and_then(|now| self.project_manager.timer_stop(id, now));
                                        self.feedback(result); ui.close();
                                    }
                                    if ui.button("Reset timer…").clicked() {
                                        self.project_confirmation = Some((ProjectConfirmation::ResetTimer(id), self.project_manager_generation)); ui.close();
                                    }
                                    if ui.button("Hide").clicked() {
                                        let result = self.project_manager.set_visible(id, false);
                                        self.feedback(result); ui.close();
                                    }
                                    let delete = ui.button("Delete…");
                                    self.project_action_rects.insert("project-row-delete", delete.rect);
                                    if delete.clicked() {
                                        self.project_confirmation = Some((ProjectConfirmation::Delete(id), self.project_manager_generation)); ui.close();
                                    }
                                });
                            }
                        });
                        ui.horizontal_wrapped(|ui| {
                            let add = self.project_icon_button(ui, "ProjectAdd", Vec2::splat(23.), self.project_draft.is_none(), "Create a project row");
                            self.project_action_rects.insert("new-project", add.rect);
                            if add.clicked() {
                                self.project_new_name = Some((String::new(), self.project_manager_generation));
                            }
                            if self.project_icon_button(ui, "ProjectIn", Vec2::splat(23.), true, "Import project archive").clicked() {
                                self.project_file_request = Some((ProjectFileAction::ImportRow, self.project_manager_generation));
                            }
                            let manager = ui.small_button("Mngr").on_hover_text("Project name, customer and notes");
                            self.project_action_rects.insert("manager", manager.rect);
                            if manager.clicked() {
                                self.apply_project_action(crate::project_manager::ProjectAction::OpenManager);
                            }
                        });
                    });
                    let strip = egui::ScrollArea::horizontal().id_salt("numbered-jobbags").scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible).max_height(120.).show(ui, |ui| {
                        ui.horizontal_top(|ui| {
                            ui.spacing_mut().item_spacing.x = 4.;
                            for slot in 0..self.project_manager.active().slots.len() { self.project_slot_ui(ui, slot); }
                        });
                    });
                    self.project_action_rects.insert("numbered-strip", strip.inner_rect);
                    if let Some(bar) = ui.ctx().read_response(strip.id.with(0usize)) {
                        self.project_action_rects.insert("numbered-scrollbar", bar.rect);
                    }
                });
                let (divider, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 14.), Sense::hover());
                ui.painter().line_segment([divider.left_center(), divider.right_center()], egui::Stroke::new(1., Color32::from_gray(130)));
                let label = if self.project_show_named { "▴" } else { "▾" };
                let toggle_rect = Rect::from_center_size(divider.center(), Vec2::new(20., 14.));
                self.project_action_rects.insert("toggle-named", toggle_rect);
                if overlay_button(ui, toggle_rect, "named-toggle", label).on_hover_text("Show or hide document snapshots").clicked() {
                    self.project_show_named = !self.project_show_named;
                }
                if self.project_show_named {
                    let named = egui::ScrollArea::horizontal().id_salt("named-jobbags").scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible).show(ui, |ui| {
                        ui.horizontal_top(|ui| {
                            ui.spacing_mut().item_spacing.x = 4.;
                            for name in MASTER_NAMES { self.project_master_ui(ui, name); }
                        });
                    });
                    self.project_action_rects.insert("named-strip", named.inner_rect);
                }
            });
        self.persist_project_catalog();
    }

    pub(super) fn update_project_capture(&mut self, context: &egui::Context) {
        let Some(worker) = self.project_capture_worker.take() else { return; };
        match worker.receiver.try_recv() {
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                self.project_capture_worker = Some(worker);
                context.request_repaint_after(std::time::Duration::from_millis(100));
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => self.feedback(Err(om9_core::Error::Io("jobbag capture worker stopped".into()))),
            Ok((capture, entities, result, mut reservation)) => {
                let current = self.project_manager.active().id == worker.project
                    && self.project_manager_generation == worker.generation
                    && self.project_manager.active().slot(worker.slot).map(|slot| slot.snapshot.as_ref().map(|bag| bag.id)) == Some(worker.previous)
                    && self.controller.document().id == worker.document && self.controller.document().revision == worker.revision
                    && self.controller.selection_state().revision == worker.selection_revision;
                let result = if worker.cancel.is_canceled() { Err(om9_core::Error::Canceled) }
                    else if !current { Err(om9_core::Error::Stale("jobbag capture changed")) }
                    else { result.and_then(|archive| {
                        capture.revalidate(self.controller.coordinator())?;
                        self.project_asset_leases.try_reserve(1).map_err(|_| om9_core::Error::Budget("project asset leases"))?;
                        let bytes = (archive.len() as u64).checked_add((2 * std::mem::size_of::<usize>()) as u64)
                            .ok_or(om9_core::Error::Overflow("project asset owner"))?;
                        if bytes > reservation.bytes() { return Err(om9_core::Error::Budget("project asset owner")); }
                        let mut owner = reservation.split_off(bytes)?;
                        self.project_manager.capture_archive_with_limits(worker.slot, entities, self.controller.document(), archive, self.controller.coordinator().limits())?;
                        let asset = self.project_manager.active().slot(worker.slot).and_then(|slot| slot.snapshot.as_ref())
                            .and_then(|snapshot| snapshot.archive.as_ref()).ok_or(om9_core::Error::Missing("captured jobbag asset"))?;
                        let _ = owner.set_allocated_bytes(bytes);
                        self.project_asset_leases.push(ProjectAssetLease { payload: std::sync::Arc::downgrade(asset), _reservation: owner });
                        Ok(())
                    }) };
                self.feedback(result.map_err(jobbag_capture_error));
            }
        }
    }

    pub(super) fn update_project_document_capture(&mut self, context: &egui::Context) {
        let Some(worker) = self.project_document_worker.take() else { return; };
        match worker.receiver.try_recv() {
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                self.project_document_worker = Some(worker);
                context.request_repaint_after(std::time::Duration::from_millis(100));
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => self.feedback(Err(om9_core::Error::Io("document snapshot worker stopped".into()))),
            Ok((capture, objects, layers, result, mut reservation)) => {
                let current = self.project_manager.active().id == worker.project
                    && self.project_manager_generation == worker.generation
                    && self.project_manager.named_master_snapshot(worker.name).map(|bag| bag.id) == worker.previous
                    && worker.replace.is_none_or(|id| self.project_manager.named_master_by_id(worker.name, id).is_some())
                    && self.controller.document().id == worker.document && self.controller.document().revision == worker.revision;
                let result = if worker.cancel.is_canceled() { Err(om9_core::Error::Canceled) }
                    else if !current { Err(om9_core::Error::Stale("document snapshot changed")) }
                    else { result.and_then(|archive| {
                        capture.revalidate(self.controller.coordinator())?;
                        self.project_asset_leases.try_reserve(1).map_err(|_| om9_core::Error::Budget("project asset leases"))?;
                        let bytes = (archive.len() as u64).checked_add((2 * std::mem::size_of::<usize>()) as u64)
                            .ok_or(om9_core::Error::Overflow("project asset owner"))?;
                        if bytes > reservation.bytes() { return Err(om9_core::Error::Budget("project asset owner")); }
                        let mut owner = reservation.split_off(bytes)?;
                        let id = match worker.replace {
                            Some(old) => self.project_manager.replace_named_version_summary_with_limits(worker.name, old, objects, layers, archive, self.controller.coordinator().limits())?,
                            None => self.project_manager.capture_named_version_summary_with_limits(worker.name, objects, layers, archive, self.controller.coordinator().limits())?,
                        };
                        let asset = self.project_manager.named_master_by_id(worker.name, id).and_then(|snapshot| snapshot.archive.as_ref())
                            .ok_or(om9_core::Error::Missing("captured document asset"))?;
                        let _ = owner.set_allocated_bytes(bytes);
                        self.project_asset_leases.push(ProjectAssetLease { payload: std::sync::Arc::downgrade(asset), _reservation: owner });
                        self.project_selected_versions.insert((worker.project, worker.name), id);
                        Ok(())
                    }) };
                self.feedback(result.map_err(jobbag_capture_error));
            }
        }
    }

    pub(super) fn update_project_import(&mut self, context: &egui::Context) {
        let Some(worker) = self.project_import_worker.take() else { return; };
        match worker.receiver.try_recv() {
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                self.project_import_worker = Some(worker);
                context.request_repaint_after(std::time::Duration::from_millis(100));
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => self.feedback(Err(om9_core::Error::Io("project import worker stopped".into()))),
            Ok((result, mut reservation)) => {
                let result = if worker.cancel.is_canceled() { Err(om9_core::Error::Canceled) }
                    else if self.project_draft.is_some() || self.project_manager_generation != worker.generation || self.project_manager.change_revision() != worker.revision
                        || worker.embedded_source.is_some_and(|(id, revision)| self.controller.document().id != id || self.controller.document().revision != revision) {
                        Err(om9_core::Error::Stale("project catalog changed during import"))
                    } else { result.and_then(|result| match result {
                        ProjectImportResult::Prepared(files, bytes) => {
                            drop(reservation);
                            self.start_prepared_project_import(files, bytes)
                        }
                        ProjectImportResult::Imported(manager, _source_files) => {
                            retain_manager_assets(&mut self.project_asset_leases, &manager, &mut reservation)?;
                            self.project_manager = manager;
                            self.project_manager_generation = self.project_manager_generation.wrapping_add(1);
                            self.project_file_preview = None;
                            self.project_selected_versions.clear();
                            Ok(())
                        }
                    }) };
                self.feedback(result);
            }
        }
    }

    pub(super) fn update_project_export(&mut self, context: &egui::Context) {
        let Some(worker) = self.project_export_worker.take() else { return; };
        match worker.receiver.try_recv() {
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                self.project_export_worker = Some(worker);
                context.request_repaint_after(std::time::Duration::from_millis(100));
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => self.feedback(Err(om9_core::Error::Io("project export worker stopped".into()))),
            Ok((result, _reservation)) => {
                let result = if worker.cancel.is_canceled() { Err(om9_core::Error::Canceled) }
                    else if self.project_manager_generation != worker.generation || self.project_manager.change_revision() != worker.revision {
                        Err(om9_core::Error::Stale("project catalog changed during export"))
                    } else { result.and_then(|prepared| prepared.publish(&worker.cancel)) };
                self.feedback(result);
            }
        }
    }

    pub(super) fn update_project_root(&mut self, context: &egui::Context) {
        let Some(worker) = self.project_root_worker.take() else { return; };
        match worker.receiver.try_recv() {
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                self.project_root_worker = Some(worker);
                context.request_repaint_after(std::time::Duration::from_millis(100));
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => self.feedback(Err(om9_core::Error::Io("project root worker stopped".into()))),
            Ok((result, _reservation)) => {
                let result = if worker.cancel.is_canceled() { Err(om9_core::Error::Canceled) }
                    else if self.project_manager_generation != worker.generation || self.project_manager.change_revision() != worker.revision
                        || self.project_catalog_path != worker.old_path {
                        Err(om9_core::Error::Stale("project catalog changed during root change"))
                    } else { result.and_then(|(prepared, source_guard)| {
                        if let Some(guard) = &source_guard { guard.verify_absent_source()?; }
                        prepared.publish(&worker.cancel)
                    }) };
                let result = result.map(|(target, fingerprint)| {
                    self.project_catalog_path = Some(target);
                    self.project_catalog_fingerprint = Some(fingerprint);
                    self.project_catalog_revision = Some(worker.revision);
                    self.project_catalog_attempted = self.project_catalog_revision;
                    self.project_catalog_error = None;
                    self.project_catalog_ready = true;
                });
                self.feedback(result);
            }
        }
    }

    fn start_project_root_change(&mut self, root: &std::path::Path, settings: &std::path::Path) -> Result<()> {
        if self.project_root_worker.is_some() || self.project_catalog_worker.is_some() || self.project_catalog_load_worker.is_some() {
            return Err(om9_core::Error::Busy);
        }
        let root = std::path::absolute(root).map_err(|error| om9_core::Error::Io(error.to_string()))?;
        if !root.is_dir() { return Err(om9_core::Error::Invalid("project root folder")); }
        if self.project_catalog_path.as_ref() == Some(&root.join("project-catalog.json")) { return Ok(()); }
        let settings_before = crate::project_catalog::root_settings_fingerprint(settings)?;
        let limits = *self.controller.coordinator().limits();
        let estimate = self.project_manager.storage_bytes_estimate(&limits)?;
        let bytes = estimate.checked_mul(3).and_then(|n| n.checked_add(64 * 1024 * 1024))
            .ok_or(om9_core::Error::Overflow("project root admission"))?;
        let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
        let reservation = self.controller.coordinator().reserve_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::CpuCache, bytes, physical, self.controller.scene_cache_usage()?)?;
        let manager = self.project_manager.clone();
        let source_path = self.project_catalog_path.clone();
        let source_fingerprint = self.project_catalog_fingerprint;
        let settings = settings.to_path_buf();
        let cancel = om9_core::transaction::Cancellation::default();
        let thread_cancel = cancel.clone();
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        let worker = ProjectRootWorker { receiver, cancel, generation: self.project_manager_generation,
            revision: self.project_manager.change_revision(), old_path: self.project_catalog_path.clone() };
        std::thread::Builder::new().name("om9-project-root".into()).spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let guard = source_path.as_deref().map(|path| crate::project_catalog::guard_source_catalog(path, source_fingerprint)).transpose()?;
                let prepared = crate::project_catalog::prepare_catalog_root(&root, &settings, &manager, &limits, &thread_cancel, settings_before)?;
                Ok((prepared, guard))
            }))
                .unwrap_or(Err(om9_core::Error::Io("project root worker failed".into())));
            let _ = sender.send((result, reservation));
        }).map_err(|error| om9_core::Error::Io(format!("project root worker: {error}")))?;
        self.project_root_worker = Some(worker);
        self.status.info("Changing project folder…");
        Ok(())
    }

    fn start_jobbag_export(&mut self, path: &std::path::Path, archive: std::sync::Arc<[u8]>) -> Result<()> {
        if self.project_export_worker.is_some() { return Err(om9_core::Error::Busy); }
        let held = self.controller.scene_cache_usage()?;
        let input_bytes = if self.project_asset_is_leased(&archive) { 1 } else { (archive.len() as u64).checked_add(2 * std::mem::size_of::<usize>() as u64)
            .ok_or(om9_core::Error::Overflow("jobbag export owner"))? };
        let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
        let reservation = self.controller.coordinator().reserve_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::CpuCache, input_bytes, physical, held)?;
        let coordinator = self.controller.coordinator().memory_admission_peer_with_held(held)?;
        let path = path.to_path_buf();
        let cancel = om9_core::transaction::Cancellation::default();
        let thread_cancel = cancel.clone();
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        let worker = ProjectExportWorker { receiver, cancel, generation: self.project_manager_generation, revision: self.project_manager.change_revision() };
        std::thread::Builder::new().name("om9-jobbag-export".into()).spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> Result<_> {
                let (decode_reservation, document) = crate::session::project_state::validate_project_archive_reserved(
                    &coordinator, &archive, held, Some(&thread_cancel), true)?;
                drop(document);
                drop(decode_reservation);
                crate::project_catalog::prepare_bytes(&path, &archive, &thread_cancel)
            })).unwrap_or(Err(om9_core::Error::Io("jobbag export worker failed".into())));
            let _ = sender.send((result, reservation));
        }).map_err(|error| om9_core::Error::Io(format!("jobbag export worker: {error}")))?;
        self.project_export_worker = Some(worker);
        self.status.info("Exporting archive…");
        Ok(())
    }

    fn start_project_import_scan(&mut self, source: ProjectImportSource) -> Result<()> {
        if self.project_import_worker.is_some() || self.project_draft.is_some() { return Err(om9_core::Error::Busy); }
        if let ProjectImportSource::Files(files) = &source {
            if files.is_empty() || files.len() > 64 { return Err(om9_core::Error::Budget("project import files")); }
        }
        let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
        let reservation = self.controller.coordinator().reserve_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::CpuCache, 8 * 1024 * 1024, physical, self.controller.scene_cache_usage()?)?;
        let cancel = om9_core::transaction::Cancellation::default();
        let thread_cancel = cancel.clone();
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        #[cfg(feature = "test-fixtures")]
        let ready = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        #[cfg(feature = "test-fixtures")]
        let worker_ready = ready.clone();
        let worker = ProjectImportWorker { receiver, cancel, revision: self.project_manager.change_revision(),
            generation: self.project_manager_generation, embedded_source: None, #[cfg(feature = "test-fixtures")] ready, #[cfg(feature = "test-fixtures")] preparing: true };
        std::thread::Builder::new().name("om9-project-folder-scan".into()).spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> Result<_> {
                let mut files = Vec::new();
                let mut path_bytes = 0usize;
                let mut add = |path: std::path::PathBuf| -> Result<()> {
                    path_bytes = path_bytes.checked_add(path.as_os_str().len()).ok_or(om9_core::Error::Overflow("project folder paths"))?;
                    if files.len() >= 64 || path_bytes > 4 * 1024 * 1024 { return Err(om9_core::Error::Budget("project folder entries")); }
                    files.try_reserve(1).map_err(|_| om9_core::Error::Budget("project folder entries"))?;
                    files.push(path);
                    Ok(())
                };
                match source {
                    ProjectImportSource::Folder(folder) => for entry in std::fs::read_dir(&folder).map_err(|error| om9_core::Error::Io(error.to_string()))? {
                        if thread_cancel.is_canceled() { return Err(om9_core::Error::Canceled); }
                        let path = entry.map_err(|error| om9_core::Error::Io(error.to_string()))?.path();
                        if path.extension().and_then(|ext| ext.to_str()).is_some_and(|ext| ext.eq_ignore_ascii_case("om9project")) { add(path)?; }
                    },
                    ProjectImportSource::Files(paths) => for path in paths { add(path)?; },
                }
                if thread_cancel.is_canceled() { return Err(om9_core::Error::Canceled); }
                if files.is_empty() { return Err(om9_core::Error::Missing("project archives in folder")); }
                files.sort();
                let mut bytes = 0u64;
                let mut inputs = Vec::new();
                inputs.try_reserve_exact(files.len()).map_err(|_| om9_core::Error::Budget("project import files"))?;
                for path in files {
                    if thread_cancel.is_canceled() { return Err(om9_core::Error::Canceled); }
                    let metadata = std::fs::symlink_metadata(&path).map_err(|error| om9_core::Error::Io(error.to_string()))?;
                    if !metadata.is_file() || metadata.file_type().is_symlink() { return Err(om9_core::Error::Invalid("project import file")); }
                    bytes = bytes.checked_add(metadata.len()).ok_or(om9_core::Error::Overflow("project import bytes"))?;
                    inputs.push(crate::project_catalog::open_project_for_import(&path, metadata.len())?);
                }
                Ok(ProjectImportResult::Prepared(inputs, bytes))
            })).unwrap_or(Err(om9_core::Error::Io("project folder scan worker failed".into())));
            #[cfg(feature = "test-fixtures")]
            worker_ready.store(true, std::sync::atomic::Ordering::Release);
            let _ = sender.send((result, reservation));
        }).map_err(|error| om9_core::Error::Io(format!("project folder scan worker: {error}")))?;
        self.project_import_worker = Some(worker);
        self.status.info("Preparing project import…");
        Ok(())
    }

    fn start_project_import(&mut self, files: Vec<std::path::PathBuf>) -> Result<()> {
        self.start_project_import_scan(ProjectImportSource::Files(files))
    }

    fn start_prepared_project_import(&mut self, inputs: Vec<std::fs::File>, input_bytes: u64) -> Result<()> {
        if self.project_import_worker.is_some() || self.project_draft.is_some() { return Err(om9_core::Error::Busy); }
        let held = self.controller.scene_cache_usage()?;
        let coordinator = self.controller.coordinator().memory_admission_peer_with_held(held)?;
        let limits = *self.controller.coordinator().limits();
        let existing = self.project_manager.storage_bytes_estimate(&limits)?;
        let bytes = input_bytes.checked_mul(3).and_then(|n| existing.checked_mul(2).and_then(|old| n.checked_add(old)))
            .and_then(|n| n.checked_add(64 * 1024 * 1024)).ok_or(om9_core::Error::Overflow("project import admission"))?;
        let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
        let reservation = self.controller.coordinator().reserve_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::CpuCache, bytes, physical, self.controller.scene_cache_usage()?)?;
        let mut candidate = self.project_manager.clone();
        let cancel = om9_core::transaction::Cancellation::default();
        let thread_cancel = cancel.clone();
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        #[cfg(feature = "test-fixtures")]
        let ready = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        #[cfg(feature = "test-fixtures")]
        let worker_ready = ready.clone();
        let worker = ProjectImportWorker { receiver, cancel, revision: self.project_manager.change_revision(), generation: self.project_manager_generation, embedded_source: None, #[cfg(feature = "test-fixtures")] ready, #[cfg(feature = "test-fixtures")] preparing: false };
        std::thread::Builder::new().name("om9-project-import".into()).spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> Result<_> {
                for owner in &inputs {
                    if thread_cancel.is_canceled() { return Err(om9_core::Error::Canceled); }
                    let reader = owner.try_clone().map_err(|error| om9_core::Error::Io(error.to_string()))?;
                    let (manifest, assets) = crate::project_catalog::read_project_file_with_limits(reader, &thread_cancel, &limits)?;
                    candidate.import_project_bundle_with_validator(&manifest, &assets, &limits, &thread_cancel, false, &mut |archive| {
                        crate::session::project_state::validate_project_archive_reserved(&coordinator, archive, held, Some(&thread_cancel), true)
                    })?;
                }
                if thread_cancel.is_canceled() { return Err(om9_core::Error::Canceled); }
                Ok(ProjectImportResult::Imported(candidate, inputs))
            })).unwrap_or(Err(om9_core::Error::Io("project import worker failed".into())));
            #[cfg(feature = "test-fixtures")]
            worker_ready.store(true, std::sync::atomic::Ordering::Release);
            let _ = sender.send((result, reservation));
        }).map_err(|error| om9_core::Error::Io(format!("project import worker: {error}")))?;
        self.project_import_worker = Some(worker);
        self.status.info("Importing project archives…");
        Ok(())
    }

    fn start_embedded_project_import(&mut self) -> Result<()> {
        if self.project_import_worker.is_some() || self.project_draft.is_some() { return Err(om9_core::Error::Busy); }
        if !self.preserve_embedded_project_state() { return Err(om9_core::Error::Missing("separate document projects")); }
        let (_embedded_reservation, embedded) = self.controller.embedded_project_manager_reserved()?
            .ok_or(om9_core::Error::Missing("document project state"))?;
        let held = self.controller.scene_cache_usage()?;
        let coordinator = self.controller.coordinator().memory_admission_peer_with_held(held)?;
        let limits = *self.controller.coordinator().limits();
        let existing = self.project_manager.storage_bytes_estimate(&limits)?;
        let incoming = embedded.storage_bytes_estimate(&limits)?;
        let bytes = existing.checked_mul(2).and_then(|n| incoming.checked_mul(2).and_then(|v| n.checked_add(v)))
            .and_then(|n| n.checked_add(64 * 1024 * 1024)).ok_or(om9_core::Error::Overflow("embedded project import admission"))?;
        let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
        let reservation = self.controller.coordinator().reserve_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::CpuCache, bytes, physical, self.controller.scene_cache_usage()?)?;
        let mut candidate = self.project_manager.clone();
        let cancel = om9_core::transaction::Cancellation::default();
        let thread_cancel = cancel.clone();
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        #[cfg(feature = "test-fixtures")]
        let ready = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        #[cfg(feature = "test-fixtures")]
        let worker_ready = ready.clone();
        let worker = ProjectImportWorker { receiver, cancel, revision: self.project_manager.change_revision(),
            generation: self.project_manager_generation, embedded_source: Some((self.controller.document().id, self.controller.document().revision)), #[cfg(feature = "test-fixtures")] ready, #[cfg(feature = "test-fixtures")] preparing: false };
        std::thread::Builder::new().name("om9-embedded-project-import".into()).spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                candidate.merge_embedded_with_validator(&embedded, &limits, &thread_cancel, &mut |archive| {
                    crate::session::project_state::validate_project_archive_reserved(&coordinator, archive, held, Some(&thread_cancel), true)
                }).map(|()| candidate)
            })).unwrap_or(Err(om9_core::Error::Io("embedded project import worker failed".into())));
            #[cfg(feature = "test-fixtures")]
            worker_ready.store(true, std::sync::atomic::Ordering::Release);
            let _ = sender.send((result.map(|manager| ProjectImportResult::Imported(manager, Vec::new())), reservation));
        }).map_err(|error| om9_core::Error::Io(format!("embedded project import worker: {error}")))?;
        self.project_import_worker = Some(worker);
        self.status.info("Importing document projects…");
        Ok(())
    }

    fn start_document_snapshot(&mut self, name: &'static str, replace: Option<crate::project_manager::SnapshotId>) -> Result<()> {
        if self.project_document_worker.is_some() || self.project_capture_worker.is_some() || self.controller.is_running() {
            return Err(om9_core::Error::Busy);
        }
        let capture = om9_exchange::ExportCapture::all(self.controller.coordinator(), self.controller.document().id)?;
        let limits = *self.controller.coordinator().limits();
        let source = capture.document().validate(&limits)?;
        let copied = capture.document().clone_owned_bytes_estimate(&limits)?;
        let bytes = source.bytes.checked_mul(4).and_then(|n| n.checked_add(copied))
            .and_then(|n| n.checked_add(16 * 1024 * 1024)).ok_or(om9_core::Error::Overflow("document snapshot admission"))?;
        let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
        let reservation = self.controller.coordinator().reserve_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::CpuCache, bytes, physical, self.controller.scene_cache_usage()?)?;
        #[cfg(feature = "occt-brep")]
        let brep = if capture.document().entities.values().chain(capture.document().definitions.values().flat_map(|definition| &definition.members))
            .map(|entity| &entity.geometry).chain(capture.document().results.values().map(|result| &result.geometry_snapshot))
            .any(|geometry| matches!(geometry, om9_core::geometry::Geometry::Brep(_) | om9_core::geometry::Geometry::NativeBrep(_))) {
            Some((self.controller.capture_brep_sources_for_document(capture.document(), om9_core::budget::Usage::default())?,
                crate::brep_restore_host::BrepRestoreHost::admit_packaged(om9_native_owner::ProcessLimits::default())?))
        } else { None };
        let cancel = om9_core::transaction::Cancellation::default();
        let thread_cancel = cancel.clone();
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        let worker = ProjectDocumentWorker {
            receiver, cancel, project: self.project_manager.active().id, name,
            previous: self.project_manager.named_master_snapshot(name).map(|bag| bag.id),
            replace,
            document: self.controller.document().id, revision: self.controller.document().revision,
            generation: self.project_manager_generation,
        };
        std::thread::Builder::new().name("om9-document-snapshot".into()).spawn(move || {
            let mut reservation = reservation;
            let objects = capture.document().entities.len();
            let layers = capture.document().layer_order.len();
            let result = if thread_cancel.is_canceled() { Err(om9_core::Error::Canceled) } else {
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    let mut document = capture.document().clone();
                    document.resources.retain(|_, resource| resource.name != crate::session::project_state::PROJECT_STATE_RESOURCE
                        && !resource.name.starts_with(crate::session::project_state::PROJECT_ASSET_PREFIX));
                    #[cfg(feature = "occt-brep")]
                    if let Some((brep_capture, host)) = brep {
                        let archive = brep_capture.encode_document(&host, &document, &thread_cancel)?;
                        let (bytes, mut owner) = archive.into_parts();
                        let allocated = owner.bytes().checked_sub(owner.pending_bytes()).ok_or(om9_core::Error::Invalid("BRep archive owner"))?;
                        reservation.transfer_allocated_from(&mut owner, allocated)?;
                        return Ok(bytes);
                    }
                    om9_archive::Archive::encode_with_admission(&document, &limits, |live, pending| {
                        if thread_cancel.is_canceled() { return Err(om9_core::Error::Canceled); }
                        let required = copied.checked_add(live).and_then(|n| n.checked_add(pending))
                            .ok_or(om9_core::Error::Overflow("document snapshot admission"))?;
                        if required > reservation.bytes() { return Err(om9_core::Error::Budget("document snapshot admission")); }
                        Ok(())
                    })
                })).unwrap_or(Err(om9_core::Error::Io("document snapshot worker failed".into())))
            };
            let _ = sender.send((capture, objects, layers, result, reservation));
        }).map_err(|error| om9_core::Error::Io(format!("document snapshot worker: {error}")))?;
        self.project_document_worker = Some(worker);
        self.status.info("Capturing document snapshot…");
        Ok(())
    }

    pub(super) fn update_project_thumbnails(&mut self, context: &egui::Context, allow_start: bool) {
        // Thumbnail publication changes the catalog revision. Keep completed
        // derived images queued while a catalog operation is checking that
        // revision, so an unrelated preview cannot invalidate the operation.
        if self.project_import_worker.is_some() || self.project_export_worker.is_some()
            || self.project_root_worker.is_some() || self.project_catalog_load_worker.is_some() {
            if self.project_thumbnail_worker.is_some() { context.request_repaint_after(std::time::Duration::from_millis(100)); }
            return;
        }
        if let Some(worker) = &self.project_thumbnail_worker {
            match worker.receiver.try_recv() {
                Ok((project, snapshot, archive, result, mut reservation)) => {
                    #[cfg(feature = "test-fixtures")]
                    { self.project_test_thumbnail_succeeded = Some(result.is_ok()); }
                    let canceled = worker.cancel.is_canceled();
                    self.project_thumbnail_worker = None;
                    let current = self.project_manager.snapshot_archive_owner(project, snapshot)
                        .is_some_and(|current| std::sync::Arc::ptr_eq(current, &archive));
                    let key = (project, snapshot.0);
                    let mut retry = false;
                    if current && !canceled { if let Ok(png) = result {
                        let attach = (|| -> Result<()> {
                            self.project_asset_leases.try_reserve(1).map_err(|_| om9_core::Error::Budget("thumbnail owner"))?;
                            let bytes = (png.len() as u64).checked_add((2 * std::mem::size_of::<usize>()) as u64)
                                .ok_or(om9_core::Error::Overflow("thumbnail owner"))?;
                            if bytes > reservation.bytes() { return Err(om9_core::Error::Budget("thumbnail owner")); }
                            let mut owner = reservation.split_off(bytes)?;
                            self.project_manager.set_thumbnail_with_limits(project, snapshot, png, self.controller.coordinator().limits())?;
                            let row = self.project_manager.rows().iter().find(|row| row.id == project).ok_or(om9_core::Error::Missing("thumbnail project"))?;
                            let asset = row.slots.iter().filter_map(|slot| slot.snapshot.as_ref())
                                .find(|bag| bag.id == snapshot).and_then(|bag| bag.thumbnail.png.as_ref())
                                .or_else(|| row.master.iter().chain(row.named_masters.values()).chain(row.named_versions.values().flatten())
                                    .find(|bag| bag.id == snapshot).and_then(|bag| bag.thumbnail_png.as_ref()))
                                .ok_or(om9_core::Error::Missing("thumbnail asset"))?;
                            let _ = owner.set_allocated_bytes(bytes);
                            self.project_asset_leases.push(ProjectAssetLease { payload: std::sync::Arc::downgrade(asset), _reservation: owner });
                            Ok(())
                        })();
                        retry = attach.is_err();
                    } else { retry = true; } }
                    if retry {
                        self.project_thumbnail_attempts.insert(key, (std::sync::Arc::downgrade(&archive), std::time::Instant::now() + std::time::Duration::from_secs(5)));
                    } else {
                        self.project_thumbnail_attempts.remove(&key);
                    }
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    context.request_repaint_after(std::time::Duration::from_millis(150));
                    return;
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => self.project_thumbnail_worker = None,
            }
        }
        if !allow_start || self.project_capture_close_wait { return; }
        let active = self.project_manager.active().id;
        let now = std::time::Instant::now();
        let mut retry_at: Option<std::time::Instant> = None;
        let rows = self.project_manager.rows();
        let candidates = rows.iter().filter(|row| row.id == active)
            .chain(rows.iter().filter(|row| {
                self.project_manager_open && row.id != active
                    && !row.slots.iter().filter_map(|slot| slot.snapshot.as_ref()).any(|snapshot| snapshot.thumbnail.png.is_some())
                    && !row.master.iter().chain(row.named_masters.values()).chain(row.named_versions.values().flatten())
                        .any(|snapshot| snapshot.thumbnail_png.is_some())
            }));
        let mut candidate = None;
        for row in candidates {
            let project = row.id;
            let found = row.slots.iter().filter_map(|slot| slot.snapshot.as_ref())
                .filter_map(|snapshot| snapshot.archive.as_ref().filter(|_| snapshot.thumbnail.png.is_none()).map(|archive| (snapshot.id, archive)))
                .chain(row.master.iter().chain(row.named_masters.values()).chain(row.named_versions.values().flatten())
                    .filter_map(|snapshot| snapshot.archive.as_ref().filter(|_| snapshot.thumbnail_png.is_none()).map(|archive| (snapshot.id, archive))))
                .find(|(id, archive)| {
                    let key = (project, id.0);
                    match self.project_thumbnail_attempts.get(&key) {
                        Some((attempted, deadline)) if attempted.ptr_eq(&std::sync::Arc::downgrade(archive)) && *deadline > now => {
                            retry_at = Some(retry_at.map_or(*deadline, |prior| prior.min(*deadline)));
                            false
                        }
                        _ => true,
                    }
                });
            if let Some((snapshot, archive)) = found { candidate = Some((project, snapshot, archive)); break; }
        }
        let Some((project, snapshot, archive)) = candidate else {
            if let Some(deadline) = retry_at { context.request_repaint_after(deadline.saturating_duration_since(now)); }
            return;
        };
        if self.project_thumbnail_attempts.len() >= 1024 { self.project_thumbnail_attempts.clear(); }
        let Some(bytes) = (archive.len() as u64).checked_mul(2).and_then(|size| size.checked_add(128 * 1024 * 1024)) else { return; };
        let Ok(held) = self.controller.scene_cache_usage() else {
            context.request_repaint_after(std::time::Duration::from_secs(1));
            return;
        };
        let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
        let Ok(mut reservation) = self.controller.coordinator().reserve_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::CpuCache, bytes, physical, held,
        ) else {
            context.request_repaint_after(std::time::Duration::from_secs(1));
            return;
        };
        let attempted = std::sync::Arc::downgrade(archive);
        let archive = archive.clone();
        let limits = *self.controller.coordinator().limits();
        let Ok(coordinator) = self.controller.coordinator().memory_admission_peer_with_held(held) else {
            context.request_repaint_after(std::time::Duration::from_secs(1));
            return;
        };
        let cancel = om9_core::transaction::Cancellation::default();
        let thread_cancel = cancel.clone();
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        if std::thread::Builder::new().name("om9-jobbag-thumbnail".into()).spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| crate::headless::render_jobbag_thumbnail(&archive, limits, &coordinator, held, &thread_cancel, |required| {
                if required > reservation.bytes() {
                    let growth = required - reservation.bytes();
                    let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
                    coordinator.grow_external_memory_with_held(&mut reservation, growth, physical, held)?;
                }
                Ok(())
            })))
                .unwrap_or(Err(om9_core::Error::Io("thumbnail worker failed".into())));
            let _ = sender.send((project, snapshot, archive, result, reservation));
        }).is_ok() {
            self.project_thumbnail_attempts.insert((project, snapshot.0), (attempted, std::time::Instant::now() + std::time::Duration::from_secs(5)));
            self.project_thumbnail_worker = Some(ProjectThumbnailWorker { receiver, cancel });
            context.request_repaint_after(std::time::Duration::from_millis(150));
        } else {
            context.request_repaint_after(std::time::Duration::from_secs(1));
        }
    }

    fn project_thumbnail_texture(&mut self, context: &egui::Context, project: crate::project_manager::ProjectId,
        snapshot: crate::project_manager::SnapshotId, bytes: Option<&[u8]>) -> Option<egui::TextureId> {
        let bytes = bytes?;
        let key = (project, snapshot.0, Sha256::digest(bytes).into());
        if let Some(texture) = self.project_thumbnail_textures.get(&key) { return Some(texture.id()); }
        // Check PNG dimensions before asking the decoder to allocate memory.
        if bytes.len() > 128 * 1024 || bytes.len() < 24 || &bytes[..8] != b"\x89PNG\r\n\x1a\n"
            || &bytes[12..16] != b"IHDR" || bytes[16..24] != [0, 0, 0, 96, 0, 0, 0, 96] { return None; }
        let image = image::load_from_memory_with_format(bytes, image::ImageFormat::Png).ok()?.to_rgba8();
        if image.width() != 96 || image.height() != 96 { return None; }
        if self.project_thumbnail_textures.len() >= 64 {
            if let Some(oldest) = self.project_thumbnail_textures.keys().next().copied() { self.project_thumbnail_textures.remove(&oldest); }
        }
        let texture = context.load_texture(format!("project-thumb-{}-{}", project.0, snapshot.0),
            egui::ColorImage::from_rgba_unmultiplied([96, 96], image.as_raw()), egui::TextureOptions::LINEAR);
        let id = texture.id();
        self.project_thumbnail_textures.insert(key, texture);
        Some(id)
    }

    pub(super) fn project_dialogs(&mut self, context: &egui::Context) {
        let draft_current = self.project_draft.as_ref().is_some_and(|draft| {
            draft.generation == self.project_manager_generation && draft.id == self.project_manager.active().id
        });
        if let Some(mut editor) = self.project_directory_editor.take() {
            let mut save = false;
            let mut cancel = false;
            super::theme::modal(context, egui::Id::new("project-directory-editor")).show(context, |ui| {
                ui.heading(match editor.kind { DirectoryKind::Customer => "Customer", DirectoryKind::Designer => "Designer", DirectoryKind::Status => "Status" });
                if !draft_current { ui.colored_label(super::theme::ERROR, "The project or catalog changed. Cancel this editor."); }
                let name = ui.text_edit_singleline(&mut editor.name);
                self.project_action_rects.insert("directory-editor-name", name.rect);
                ui.horizontal(|ui| {
                    let button = ui.add_enabled(draft_current, egui::Button::new("Save"));
                    self.project_action_rects.insert("directory-editor-save", button.rect);
                    save = button.clicked(); cancel = ui.button("Cancel").clicked();
                });
            });
            if save {
                let result = match editor.id {
                    Some(id) => self.project_manager.rename_directory_entry(editor.kind, id, &editor.name),
                    None => self.project_manager.create_directory_entry(editor.kind, &editor.name).map(|_| ()),
                };
                if result.is_err() { self.project_directory_editor = Some(editor); }
                self.feedback(result);
            } else if !cancel { self.project_directory_editor = Some(editor); }
        }
        if let Some((mut name, generation)) = self.project_new_name.take() {
            let mut create = false;
            let mut cancel = false;
            super::theme::modal(context, egui::Id::new("new-project-dialog")).show(context, |ui| {
                ui.heading("Create project");
                if generation != self.project_manager_generation { ui.colored_label(super::theme::ERROR, "The project catalog changed. Copy the name, then cancel."); }
                let field = ui.text_edit_singleline(&mut name);
                self.project_action_rects.insert("new-project-name", field.rect);
                ui.horizontal(|ui| {
                    let button = ui.add_enabled(generation == self.project_manager_generation && self.project_draft.is_none(), egui::Button::new("Create"));
                    self.project_action_rects.insert("new-project-create", button.rect);
                    create = button.clicked();
                    cancel = ui.button("Cancel").clicked();
                });
            });
            if create {
                let result = self.project_manager.create_project(&name, self.controller.document().revision).map(|_| ());
                if result.is_err() { self.project_new_name = Some((name, generation)); }
                self.feedback(result);
            } else if !cancel { self.project_new_name = Some((name, generation)); }
        }
        if let Some((confirmation, generation)) = self.project_confirmation.take() {
            let mut confirm = false;
            let mut cancel = false;
            let mut retry = false;
            super::theme::modal(context, egui::Id::new("project-confirmation")).show(context, |ui| {
                ui.label(match confirmation {
                    ProjectConfirmation::Delete(_) => "Remove this project from the catalog? Files on disk are kept.",
                    ProjectConfirmation::ResetTimer(_) => "Reset this project's timer?",
                    ProjectConfirmation::ClearSlot(..) | ProjectConfirmation::ClearVersion(..) => "Clear this saved bag? Objects in the drawing are kept.",
                    ProjectConfirmation::OverwriteSlot(..) => "Replace this saved bag with the selected objects?",
                    ProjectConfirmation::OverwriteVersion(..) => "Replace this saved document version with the current document?",
                    ProjectConfirmation::OverwriteFile => "Replace the selected file?",
                    ProjectConfirmation::ReloadCatalog => "Reload the workspace catalog and discard its unsaved in-memory changes?",
                    ProjectConfirmation::RemoveDirectory(..) => "Remove this record? Existing projects keep their current text.",
                    ProjectConfirmation::CloseUnsavedCatalog(..) => "The project catalog has unsaved changes. Retry saving, discard those changes, or cancel closing.",
                });
                ui.horizontal(|ui| {
                    if matches!(confirmation, ProjectConfirmation::CloseUnsavedCatalog(..)) {
                        retry = ui.add_enabled(self.project_catalog_ready, egui::Button::new("Retry save")).clicked();
                    }
                    let button = ui.button(if matches!(confirmation, ProjectConfirmation::CloseUnsavedCatalog(..)) { "Discard" } else { "Confirm" });
                    self.project_action_rects.insert("project-confirm", button.rect);
                    confirm = button.clicked();
                    cancel = ui.button("Cancel").clicked();
                });
            });
            if confirm {
                let result = if generation != self.project_manager_generation {
                    if matches!(confirmation, ProjectConfirmation::OverwriteFile) { self.project_file_pending = None; }
                    Err(om9_core::Error::Stale("project catalog changed"))
                } else { match confirmation {
                    ProjectConfirmation::Delete(id) => self.project_manager.delete_project(id),
                    ProjectConfirmation::ResetTimer(id) => timer_now().and_then(|now| self.project_manager.timer_reset(id, now)),
                    ProjectConfirmation::ClearSlot(project, slot, snapshot) => {
                        if self.project_manager.active().id != project || self.project_manager.active().slot(slot).and_then(|bag| bag.snapshot.as_ref()).is_none_or(|bag| bag.id != snapshot) {
                            Err(om9_core::Error::Stale("jobbag changed"))
                        } else { self.project_manager.clear_slot(slot) }
                    }
                    ProjectConfirmation::ClearVersion(project, name, snapshot) => {
                        if self.project_manager.active().id != project { Err(om9_core::Error::Stale("project changed")) }
                        else { self.project_manager.clear_named_version(name, snapshot) }
                    }
                    ProjectConfirmation::OverwriteSlot(project, slot, snapshot, document, revision, selection, filter) => {
                        if self.project_manager.active().id != project
                            || self.project_manager.active().slot(slot).and_then(|bag| bag.snapshot.as_ref()).is_none_or(|bag| bag.id != snapshot)
                            || self.controller.document().id != document || self.controller.document().revision != revision
                            || self.controller.selection_state().revision != selection {
                            Err(om9_core::Error::Stale("jobbag capture changed"))
                        } else { self.capture_project_slot(slot, filter) }
                    }
                    ProjectConfirmation::OverwriteVersion(project, name, snapshot, document, revision) => {
                        if self.project_manager.active().id != project || self.project_manager.named_master_by_id(name, snapshot).is_none()
                            || self.controller.document().id != document || self.controller.document().revision != revision {
                            Err(om9_core::Error::Stale("document snapshot changed"))
                        } else { self.start_document_snapshot(name, Some(snapshot)) }
                    }
                    ProjectConfirmation::OverwriteFile => match self.project_file_pending.take() {
                        Some((action, path)) => self.execute_project_file(action, &path),
                        None => Err(om9_core::Error::Missing("project export destination")),
                    },
                    ProjectConfirmation::ReloadCatalog => self.reload_project_catalog(),
                    ProjectConfirmation::RemoveDirectory(kind, id) => {
                        if !draft_current { Err(om9_core::Error::Stale("project catalog changed")) }
                        else { self.project_manager.remove_directory_entry(kind, id) }
                    },
                    ProjectConfirmation::CloseUnsavedCatalog(revision) => {
                        if revision != self.project_manager.change_revision() { Err(om9_core::Error::Stale("project catalog changed")) }
                        else {
                            self.project_catalog_discard_revision = Some(revision);
                            context.send_viewport_cmd(egui::ViewportCommand::Close);
                            Ok(())
                        }
                    }
                }};
                self.feedback(result);
            } else if retry {
                if generation != self.project_manager_generation {
                    self.feedback(Err(om9_core::Error::Stale("project catalog changed")));
                } else {
                    self.project_catalog_attempted = None;
                    self.project_catalog_close_wait = true;
                    self.persist_project_catalog();
                    if !self.project_catalog_ready { self.project_catalog_close_wait = false; }
                }
            } else if !cancel { self.project_confirmation = Some((confirmation, generation)); }
            else if matches!(confirmation, ProjectConfirmation::OverwriteFile) { self.project_file_pending = None; }
        }
    }

    fn start_project_catalog_load(&mut self, path: std::path::PathBuf, startup: bool) -> Result<()> {
        if self.project_catalog_load_worker.is_some() || self.project_catalog_worker.is_some() || self.project_root_worker.is_some()
            || self.project_capture_worker.is_some() || self.project_document_worker.is_some() || self.project_import_worker.is_some() {
            return Err(om9_core::Error::Busy);
        }
        let held = self.controller.scene_cache_usage()?;
        let physical = || om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
        let coordinator = self.controller.coordinator().memory_admission_peer_with_held(held)?;
        let limits = *self.controller.coordinator().limits();
        let mut reservation = coordinator.reserve_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::CpuCache, 1, physical(), held,
        )?;
        let name = self.controller.document().name.clone();
        let cancel = om9_core::transaction::Cancellation::default();
        let thread_cancel = cancel.clone();
        let thread_path = path.clone();
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        let worker = ProjectCatalogLoadWorker { receiver, cancel, path, generation: self.project_manager_generation,
            revision: self.project_manager.change_revision(), startup };
        std::thread::Builder::new().name("om9-project-catalog-load".into()).spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> Result<_> {
                let loaded = crate::project_catalog::load_with_limits_and_admission(&thread_path, &name, &limits, |estimated| {
                    if thread_cancel.is_canceled() { return Err(om9_core::Error::Canceled); }
                    if estimated > reservation.bytes() {
                        let growth = estimated - reservation.bytes();
                        coordinator.grow_external_memory_with_held(&mut reservation, growth, physical(), held)?;
                    }
                    Ok(())
                })?;
                if thread_cancel.is_canceled() { return Err(om9_core::Error::Canceled); }
                if let Some((manager, _)) = &loaded {
                    crate::session::project_state::validate_catalog_project_archives(&coordinator, manager, held, Some(&thread_cancel))?;
                }
                if thread_cancel.is_canceled() { return Err(om9_core::Error::Canceled); }
                Ok(loaded)
            })).unwrap_or(Err(om9_core::Error::Io("project catalog load worker failed".into())));
            let _ = sender.send((result, reservation));
        }).map_err(|error| om9_core::Error::Io(format!("project catalog load worker: {error}")))?;
        self.project_catalog_load_worker = Some(worker);
        Ok(())
    }

    pub(super) fn open_project_catalog(&mut self) {
        let path = match crate::project_catalog::configured_path() {
            Ok(Some(path)) => path,
            Ok(None) => return,
            Err(error) => {
                self.project_catalog_path = crate::project_catalog::default_path();
                self.project_catalog_ready = false;
                self.project_catalog_error = Some(error.to_string());
                self.transcript.append_history(&format!("Project root was not opened: {error}"));
                self.status.info("Project root could not be opened; existing data was retained.");
                return;
            }
        };
        self.project_catalog_path = Some(path.clone());
        if let Err(error) = self.start_project_catalog_load(path, true) {
                self.project_catalog_ready = false;
                self.project_catalog_error = Some(error.to_string());
                self.transcript.append_history(&format!("Project catalog was not opened: {error}"));
                self.status.info("Project catalog could not be opened; existing data was retained.");
        }
    }

    fn reload_project_catalog(&mut self) -> Result<()> {
        if self.project_draft.is_some() { return Err(om9_core::Error::Busy); }
        let path = self.project_catalog_path.clone().ok_or(om9_core::Error::Missing("project catalog path"))?;
        self.start_project_catalog_load(path, false)
    }

    fn open_project_folder(&self) -> Result<()> {
        let folder = self.project_catalog_path.as_deref().and_then(std::path::Path::parent)
            .ok_or(om9_core::Error::Missing("project catalog folder"))?;
        #[cfg(windows)]
        { std::process::Command::new("explorer.exe").arg(folder).spawn().map_err(|error| om9_core::Error::Io(error.to_string()))?; Ok(()) }
        #[cfg(not(windows))]
        { let _ = folder; Err(om9_core::Error::Unsupported("open project folder")) }
    }

    fn persist_project_catalog(&mut self) {
        if !self.project_catalog_ready { return; }
        if self.project_root_worker.is_some() { return; }
        if self.project_catalog_load_worker.is_some() { return; }
        if self.project_catalog_worker.is_some() { return; }
        let revision = self.project_manager.change_revision();
        if self.project_catalog_discard_revision == Some(revision) { return; }
        if self.project_catalog_revision == Some(revision) || self.project_catalog_attempted == Some(revision) { return; }
        let Some(path) = self.project_catalog_path.clone() else { return; };
        // Record the attempt so an I/O failure does not retry on every frame.
        self.project_catalog_attempted = Some(revision);
        let result = (|| -> Result<ProjectCatalogWorker> {
            let limits = *self.controller.coordinator().limits();
            let estimate = self.project_manager.storage_bytes_estimate(&limits)?;
            let bytes = estimate.checked_mul(3).and_then(|n| n.checked_add(64 * 1024 * 1024))
                .ok_or(om9_core::Error::Overflow("project catalog admission"))?;
            let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
            let reservation = self.controller.coordinator().reserve_external_memory_with_held(
                om9_core::budget::memory::MemoryClass::CpuCache, bytes, physical, self.controller.scene_cache_usage()?)?;
            let manager = self.project_manager.clone();
            let fingerprint = self.project_catalog_fingerprint;
            let worker_path = path.clone();
            let (sender, receiver) = std::sync::mpsc::sync_channel(1);
            std::thread::Builder::new().name("om9-project-catalog".into()).spawn(move || {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| crate::project_catalog::save_with_limits(&worker_path, &manager, fingerprint, &limits)))
                    .unwrap_or(Err(om9_core::Error::Io("project catalog worker failed".into())));
                let _ = sender.send((result, reservation));
            }).map_err(|error| om9_core::Error::Io(format!("project catalog worker: {error}")))?;
            Ok(ProjectCatalogWorker { receiver, revision, path })
        })();
        match result {
            Ok(worker) => self.project_catalog_worker = Some(worker),
            Err(error) => self.project_catalog_failure(error),
        }
    }

    fn project_catalog_failure(&mut self, error: om9_core::Error) {
        self.project_catalog_error = Some(error.to_string());
        self.transcript.append_history(&format!("Project catalog was not saved: {error}"));
        self.status.warning("Project catalog has unsaved changes. Open Mngr to retry or change root.");
    }

    pub(super) fn service_project_catalog(&mut self, context: &egui::Context) {
        self.project_asset_leases.retain(|lease| lease.payload.strong_count() != 0);
        if let Some(worker) = self.project_catalog_load_worker.take() {
            match worker.receiver.try_recv() {
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    self.project_catalog_load_worker = Some(worker);
                    context.request_repaint_after(std::time::Duration::from_millis(100));
                    return;
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.project_catalog_error = Some("Project catalog load worker stopped".into());
                    if worker.startup { self.project_catalog_ready = false; }
                }
                Ok((result, mut reservation)) => {
                    let stale = self.project_draft.is_some() || self.project_catalog_path.as_ref() != Some(&worker.path)
                        || self.project_manager_generation != worker.generation
                        || self.project_manager.change_revision() != worker.revision;
                    let result = if worker.cancel.is_canceled() { Err(om9_core::Error::Canceled) }
                        else if stale { Err(om9_core::Error::Stale("project catalog changed during load")) }
                        else { result };
                    let result = result.and_then(|loaded| {
                        if let Some((manager, _)) = &loaded {
                            retain_manager_assets(&mut self.project_asset_leases, manager, &mut reservation)?;
                        }
                        Ok(loaded)
                    });
                    match result {
                        Ok(Some((manager, fingerprint))) => {
                            if worker.startup { self.project_preserve_embedded = self.document_has_embedded_projects(); }
                            self.project_manager = manager;
                            self.project_manager_generation = self.project_manager_generation.wrapping_add(1);
                            self.project_file_preview = None;
                            self.project_selected_versions.clear();
                            self.project_draft = None;
                            self.project_catalog_ready = true;
                            self.project_catalog_revision = Some(self.project_manager.change_revision());
                            self.project_catalog_attempted = self.project_catalog_revision;
                            self.project_catalog_fingerprint = Some(fingerprint);
                            self.project_catalog_error = None;
                        }
                        Ok(None) if worker.startup => {
                            self.project_catalog_ready = true;
                            self.project_catalog_revision = None;
                            self.project_catalog_attempted = None;
                        }
                        Ok(None) => self.project_catalog_error = Some("Project catalog not found".into()),
                        Err(error) => {
                            if worker.startup { self.project_catalog_ready = false; }
                            self.project_catalog_error = Some(error.to_string());
                            self.status.warning("Project catalog was not loaded; current projects were retained.");
                        }
                    }
                }
            }
        }
        if let Some(worker) = self.project_catalog_worker.take() {
            match worker.receiver.try_recv() {
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    self.project_catalog_worker = Some(worker);
                    context.request_repaint_after(std::time::Duration::from_millis(150));
                    return;
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => self.project_catalog_failure(om9_core::Error::Io("project catalog worker stopped".into())),
                Ok((result, _reservation)) => {
                    if self.project_catalog_path.as_ref() == Some(&worker.path) {
                        match result {
                            Ok(fingerprint) => {
                                self.project_catalog_fingerprint = Some(fingerprint);
                                self.project_catalog_revision = Some(worker.revision);
                                self.project_catalog_error = None;
                            }
                            Err(error) => self.project_catalog_failure(error),
                        }
                    }
                }
            }
        }
        self.persist_project_catalog();
        if self.project_catalog_worker.is_some() { context.request_repaint_after(std::time::Duration::from_millis(150)); }
        else if self.project_catalog_close_wait {
            self.project_catalog_close_wait = false;
            if self.project_catalog_ready && self.project_catalog_revision == Some(self.project_manager.change_revision()) {
                context.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }

    fn take_current_project_file_request(&mut self) -> Result<Option<ProjectFileAction>> {
        let Some((action, generation)) = self.project_file_request.take() else { return Ok(None); };
        if generation != self.project_manager_generation { return Err(om9_core::Error::Stale("project catalog changed")); }
        Ok(Some(action))
    }

    pub(super) fn service_project_file_picker(&mut self, frame: &eframe::Frame) {
        let action = match self.take_current_project_file_request() {
            Ok(Some(action)) => action,
            Ok(None) => return,
            Err(error) => { self.feedback(Err(error)); return; }
        };
        let generation = self.project_manager_generation;
        let picked = match action {
            ProjectFileAction::ImportRow => crate::native_picker::pick_project_archive(frame, false),
            ProjectFileAction::ImportFolder | ProjectFileAction::ChangeRoot => crate::native_picker::pick_project_folder(frame),
            ProjectFileAction::ExportRow(_) => crate::native_picker::pick_project_archive(frame, true),
            ProjectFileAction::ExportSlot(..) | ProjectFileAction::ExportVersion(..) => crate::native_picker::pick(frame, crate::file_lifecycle::FilePhase::SavePath, None),
        };
        match picked {
            Ok(Some(path)) => {
                if generation != self.project_manager_generation {
                    self.feedback(Err(om9_core::Error::Stale("project catalog changed")));
                    return;
                }
                if matches!(action, ProjectFileAction::ExportRow(_) | ProjectFileAction::ExportSlot(..) | ProjectFileAction::ExportVersion(..)) && path.exists() {
                    self.project_file_pending = Some((action, path));
                    self.project_confirmation = Some((ProjectConfirmation::OverwriteFile, self.project_manager_generation));
                } else {
                    let result = self.execute_project_file(action, &path);
                    self.feedback(result);
                }
            }
            Ok(None) => {}
            Err(error) => self.feedback(Err(error)),
        }
    }

    fn execute_project_file(&mut self, action: ProjectFileAction, path: &std::path::Path) -> Result<()> {
        let extension = path.extension().and_then(|extension| extension.to_str()).unwrap_or("");
        match action {
            ProjectFileAction::ImportFolder => {
                self.start_project_import_scan(ProjectImportSource::Folder(path.to_path_buf()))?;
            }
            ProjectFileAction::ChangeRoot => {
                let settings = crate::project_catalog::root_settings_path()
                    .ok_or(om9_core::Error::Unsupported("project catalog path is overridden by environment"))?;
                self.start_project_root_change(path, &settings)?;
            }
            ProjectFileAction::ImportRow => {
                if !extension.eq_ignore_ascii_case("om9project") { return Err(om9_core::Error::Unsupported("project archive extension")); }
                self.start_project_import(vec![path.to_path_buf()])?;
            }
            ProjectFileAction::ExportRow(id) => {
                if !extension.eq_ignore_ascii_case("om9project") { return Err(om9_core::Error::Unsupported("project archive extension")); }
                if self.project_export_worker.is_some() { return Err(om9_core::Error::Busy); }
                let estimate = self.project_manager.storage_bytes_estimate(self.controller.coordinator().limits())?;
                let bytes = estimate.checked_mul(3).and_then(|n| n.checked_add(64 * 1024 * 1024))
                    .ok_or(om9_core::Error::Overflow("project export admission"))?;
                let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
                let reservation = self.controller.coordinator().reserve_external_memory_with_held(
                    om9_core::budget::memory::MemoryClass::CpuCache, bytes, physical, self.controller.scene_cache_usage()?)?;
                let manager = self.project_manager.clone();
                let limits = *self.controller.coordinator().limits();
                let path = path.to_path_buf();
                let cancel = om9_core::transaction::Cancellation::default();
                let thread_cancel = cancel.clone();
                let (sender, receiver) = std::sync::mpsc::sync_channel(1);
                let worker = ProjectExportWorker { receiver, cancel, generation: self.project_manager_generation, revision: self.project_manager.change_revision() };
                std::thread::Builder::new().name("om9-project-export".into()).spawn(move || {
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> Result<_> {
                        if thread_cancel.is_canceled() { return Err(om9_core::Error::Canceled); }
                        let bytes = crate::project_catalog::encode_project_with_check(&manager, id, &limits, || {
                            if thread_cancel.is_canceled() { Err(om9_core::Error::Canceled) } else { Ok(()) }
                        })?;
                        crate::project_catalog::prepare_bytes(&path, &bytes, &thread_cancel)
                    })).unwrap_or(Err(om9_core::Error::Io("project export worker failed".into())));
                    let _ = sender.send((result, reservation));
                }).map_err(|error| om9_core::Error::Io(format!("project export worker: {error}")))?;
                self.project_export_worker = Some(worker);
                self.status.info("Exporting archive…");
            }
            ProjectFileAction::ExportSlot(project, slot, snapshot) => {
                if !extension.eq_ignore_ascii_case("om9") { return Err(om9_core::Error::Unsupported("jobbag archive extension")); }
                let bag = self.project_manager.rows().iter().find(|row| row.id == project)
                    .and_then(|row| row.slot(slot)).and_then(|bag| bag.snapshot.as_ref())
                    .filter(|bag| bag.id == snapshot).ok_or(om9_core::Error::Stale("jobbag changed"))?;
                let archive = bag.archive.as_ref().ok_or(om9_core::Error::Unsupported("legacy jobbag archive"))?.clone();
                self.start_jobbag_export(path, archive)?;
            }
            ProjectFileAction::ExportVersion(project, name, snapshot) => {
                if !extension.eq_ignore_ascii_case("om9") { return Err(om9_core::Error::Unsupported("jobbag archive extension")); }
                let row = self.project_manager.rows().iter().find(|row| row.id == project).ok_or(om9_core::Error::Stale("project changed"))?;
                let legacy = if name == "Master" { row.master.as_ref() } else { row.named_masters.get(name) };
                let bag = legacy.into_iter().chain(row.named_versions.get(name).into_iter().flatten())
                    .find(|bag| bag.id == snapshot).ok_or(om9_core::Error::Stale("jobbag changed"))?;
                let archive = bag.archive.as_ref().ok_or(om9_core::Error::Unsupported("legacy jobbag archive"))?.clone();
                self.start_jobbag_export(path, archive)?;
            }
        }
        self.persist_project_catalog();
        Ok(())
    }

    fn project_slot_ui(&mut self, ui: &mut egui::Ui, slot: usize) {
        ui.push_id(slot, |ui| {
            ui.vertical(|ui| {
                let available = self.project_manager.slot_available(slot, self.controller.document());
                let label = self.project_manager.active().slot(slot).and_then(|bag| bag.snapshot.as_ref()).map(|snapshot| {
                    if available { format!("{}\n{} {}", snapshot.thumbnail.label, snapshot.thumbnail.count, snapshot.thumbnail.kind) }
                    else { "Unavailable".to_owned() }
                }).unwrap_or_default();
                let project = self.project_manager.active().id;
                let thumbnail = self.project_manager.active().slot(slot).and_then(|bag| bag.snapshot.as_ref())
                    .map(|snapshot| (snapshot.id, snapshot.thumbnail.png.clone()));
                let saveable = self.project_manager.active().slot(slot).and_then(|bag| bag.snapshot.as_ref())
                    .is_some_and(|snapshot| snapshot.archive.is_some());
                let texture = thumbnail.as_ref().and_then(|(id, png)| self.project_thumbnail_texture(ui.ctx(), project, *id, png.as_deref()));
                let (rect, response) = ui.allocate_exact_size(Vec2::new(82., 80.), Sense::click());
                self.jobbag_slot_rects.insert(slot, rect);
                paint_bag(ui, rect, &format!("{}", slot + 1), &label, true, texture);
                let load = overlay_button(ui, Rect::from_min_size(rect.right_top() - Vec2::new(16., 0.), Vec2::splat(16.)), "load", "▶");
                let restore = load.on_hover_text("Insert captured objects (one Undo)").clicked() || response.clicked();
                if available && restore {
                    let result = self.insert_project_slot(slot, BagFilter::All);
                    self.feedback(result);
                }
                response.context_menu(|ui| {
                    ui.label("Load");
                    for (label, filter) in BAG_FILTERS {
                        let item = ui.add_enabled(available, egui::Button::new(label));
                        if slot == 1 {
                            let key = match filter {
                                BagFilter::Curves => Some("load-filter-curves"),
                                BagFilter::Surfaces => Some("load-filter-surfaces"),
                                BagFilter::Meshes => Some("load-filter-meshes"),
                                BagFilter::Gems => Some("load-filter-gems"),
                                _ => None,
                            };
                            if let Some(key) = key { self.project_action_rects.insert(key, item.rect); }
                        }
                        if item.clicked() {
                            let result = self.insert_project_slot(slot, filter); self.feedback(result); ui.close();
                        }
                    }
                });
                ui.horizontal(|ui| {
                    let capture = self.project_icon_button(ui, "ProjectOut", Vec2::new(25., 22.), true, "Capture selected objects");
                    if slot == 0 { self.project_action_rects.insert("capture-first-slot", capture.rect); }
                    if slot == 1 { self.project_action_rects.insert("capture-second-slot", capture.rect); }
                    if slot == 2 { self.project_action_rects.insert("capture-third-slot", capture.rect); }
                    if slot == 3 { self.project_action_rects.insert("capture-fourth-slot", capture.rect); }
                    if slot == 4 { self.project_action_rects.insert("capture-fifth-slot", capture.rect); }
                    if capture.clicked() { self.request_project_capture(slot, BagFilter::All); }
                    capture.context_menu(|ui| {
                        ui.label("Capture");
                        for (label, filter) in BAG_FILTERS {
                            let item = ui.button(label);
                            if slot == 0 && filter == BagFilter::Curves { self.project_action_rects.insert("capture-filter-curves", item.rect); }
                            if slot == 2 && filter == BagFilter::Surfaces { self.project_action_rects.insert("capture-filter-surfaces", item.rect); }
                            if slot == 3 && filter == BagFilter::Meshes { self.project_action_rects.insert("capture-filter-meshes", item.rect); }
                            if slot == 4 && filter == BagFilter::Gems { self.project_action_rects.insert("capture-filter-gems", item.rect); }
                            if item.clicked() { self.request_project_capture(slot, filter); ui.close(); }
                        }
                    });
                    let save = self.project_icon_button(ui, "ProjectSave", Vec2::new(25., 22.), saveable,
                        if saveable { "Save bag as an OM9 file" } else { "No native archive is available for this bag" });
                    if slot == 0 { self.project_action_rects.insert("save-first-slot", save.rect); }
                    if save.clicked() {
                        if let Some(snapshot) = self.project_manager.active().slot(slot).and_then(|bag| bag.snapshot.as_ref()) {
                            self.project_file_request = Some((ProjectFileAction::ExportSlot(self.project_manager.active().id, slot, snapshot.id), self.project_manager_generation));
                        }
                    }
                    let occupied = self.project_manager.active().slot(slot).is_some_and(|bag| !bag.is_empty());
                    if self.project_icon_button(ui, "ProjectDelete", Vec2::new(25., 22.), occupied, "Clear this bag; keep viewport objects").clicked() {
                        if let Some(snapshot) = self.project_manager.active().slot(slot).and_then(|bag| bag.snapshot.as_ref()) {
                            self.project_confirmation = Some((ProjectConfirmation::ClearSlot(self.project_manager.active().id, slot, snapshot.id), self.project_manager_generation));
                        }
                    }
                });
            });
        });
    }

    fn request_project_capture(&mut self, slot: usize, filter: BagFilter) {
        if let Some(snapshot) = self.project_manager.active().slot(slot).and_then(|bag| bag.snapshot.as_ref()) {
            self.project_confirmation = Some((ProjectConfirmation::OverwriteSlot(
                self.project_manager.active().id, slot, snapshot.id, self.controller.document().id,
                self.controller.document().revision, self.controller.selection_state().revision, filter), self.project_manager_generation));
        } else {
            let result = self.capture_project_slot(slot, filter); self.feedback(result);
        }
    }

    fn capture_project_slot(&mut self, slot: usize, filter: BagFilter) -> Result<()> {
        if self.project_capture_worker.is_some() || self.project_document_worker.is_some() { return Err(om9_core::Error::Busy); }
        let mut ids = Vec::new();
        ids.try_reserve_exact(self.controller.selected().len()).map_err(|_| om9_core::Error::Budget("jobbag selection"))?;
        let mut visited = 0;
        for id in self.controller.selected() {
            let entity = self.controller.document().entities.get(id).ok_or(om9_core::Error::Missing("selected jobbag object"))?;
            if filter.matches_in_document_counted(entity, self.controller.document(), self.controller.coordinator().limits(), &mut visited)? { ids.push(*id); }
        }
        if ids.is_empty() { return Err(om9_core::Error::Invalid("jobbag filter matches no objects")); }
        let capture = self.controller.prepare_jobbag_capture(&ids)?;
        let limits = *self.controller.coordinator().limits();
        let usage = capture.document().validate(&limits)?;
        let bytes = usage.bytes.checked_mul(4).and_then(|size| size.checked_add(16 * 1024 * 1024))
            .ok_or(om9_core::Error::Overflow("jobbag capture admission"))?;
        let held = self.controller.scene_cache_usage()?;
        let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
        let reservation = self.controller.coordinator().reserve_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::CpuCache, bytes, physical, held)?;
        let mut entities = Vec::new();
        entities.try_reserve_exact(ids.len()).map_err(|_| om9_core::Error::Budget("jobbag selection"))?;
        for id in &ids {
            entities.push(capture.document().entities.get(id).ok_or(om9_core::Error::Missing("selected jobbag object"))?.clone());
        }
        #[cfg(feature = "occt-brep")]
        let brep = if capture.document().entities.values().chain(capture.document().definitions.values().flat_map(|definition| &definition.members))
            .map(|entity| &entity.geometry).chain(capture.document().results.values().map(|result| &result.geometry_snapshot))
            .any(|geometry| matches!(geometry, om9_core::geometry::Geometry::Brep(_) | om9_core::geometry::Geometry::NativeBrep(_))) {
            Some((self.controller.capture_brep_sources_for_document(capture.document(), om9_core::budget::Usage::default())?,
                crate::brep_restore_host::BrepRestoreHost::admit_packaged(om9_native_owner::ProcessLimits::default())?))
        } else { None };
        let cancel = om9_core::transaction::Cancellation::default();
        let thread_cancel = cancel.clone();
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        let previous = self.project_manager.active().slot(slot).ok_or(om9_core::Error::Invalid("jobbag slot"))?.snapshot.as_ref().map(|bag| bag.id);
        let worker = ProjectCaptureWorker {
            receiver, cancel, project: self.project_manager.active().id, slot, previous,
            document: self.controller.document().id, revision: self.controller.document().revision,
            selection_revision: self.controller.selection_state().revision,
            generation: self.project_manager_generation,
        };
        std::thread::Builder::new().name("om9-jobbag-capture".into()).spawn(move || {
            let mut reservation = reservation;
            let result = if thread_cancel.is_canceled() { Err(om9_core::Error::Canceled) } else {
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(||
                    {
                    #[cfg(feature = "occt-brep")]
                    if let Some((brep_capture, host)) = brep {
                        let archive = brep_capture.encode_document(&host, capture.document(), &thread_cancel)?;
                        let (bytes, mut owner) = archive.into_parts();
                        let allocated = owner.bytes().checked_sub(owner.pending_bytes()).ok_or(om9_core::Error::Invalid("BRep archive owner"))?;
                        reservation.transfer_allocated_from(&mut owner, allocated)?;
                        return Ok(bytes);
                    }
                    om9_archive::Archive::encode_with_admission(capture.document(), &limits, |live, pending| {
                        if thread_cancel.is_canceled() { return Err(om9_core::Error::Canceled); }
                        let required = usage.bytes.checked_add(live).and_then(|n| n.checked_add(pending))
                            .ok_or(om9_core::Error::Overflow("jobbag capture admission"))?;
                        if required > reservation.bytes() { return Err(om9_core::Error::Budget("jobbag capture admission")); }
                        Ok(())
                    })
                    }))
                    .unwrap_or(Err(om9_core::Error::Io("jobbag capture worker failed".into())))
            };
            let _ = sender.send((capture, entities, result, reservation));
        }).map_err(|error| om9_core::Error::Io(format!("jobbag capture worker: {error}")))?;
        self.project_capture_worker = Some(worker);
        self.status.info("Capturing jobbag…");
        Ok(())
    }

    fn insert_project_slot(&mut self, slot: usize, filter: BagFilter) -> Result<()> {
        let snapshot = self.project_manager.active().slot(slot).and_then(|bag| bag.snapshot.as_ref())
            .ok_or(om9_core::Error::Missing("jobbag slot"))?;
        if let Some(archive) = snapshot.archive.clone() {
            #[cfg(feature = "occt-brep")]
            if crate::brep_restore_host::is_brep_archive(&archive) {
                return self.start_project_brep_insert(slot, snapshot.id, archive, filter);
            }
            let accounted = self.project_asset_is_leased(&archive);
            if filter == BagFilter::All { self.controller.insert_jobbag_archive_accounted(&archive, snapshot, accounted).map(|_| ()) }
            else { self.controller.insert_jobbag_matching_filter(&archive, snapshot, filter, accounted).map(|_| ()) }
        } else {
            let ids = snapshot.filtered_ids(filter)?;
            if ids.is_empty() { return Err(om9_core::Error::Invalid("jobbag filter matches no objects")); }
            let selected: std::collections::BTreeSet<_> = ids.into_iter().collect();
            let keep: Vec<_> = snapshot.entities.iter().map(|entity| selected.contains(&entity.id)).collect();
            let entities: Vec<_> = self.project_manager.restore_entities(slot)?.into_iter().zip(keep)
                .filter_map(|(entity, keep)| keep.then_some(entity)).collect();
            if entities.iter().any(|entity| entity.attributes.layer.is_some() || entity.attributes.material.is_some()
                || entity.attributes.line_pattern.is_some() || !entity.extensions.is_empty()
                || matches!(&entity.geometry, om9_core::geometry::Geometry::Instance(_))) {
                return Err(om9_core::Error::Unsupported("legacy jobbag dependencies were not captured"));
            }
            self.controller.insert_jobbag_entities(entities).map(|_| ())
        }
    }

    #[cfg(feature = "occt-brep")]
    fn start_project_brep_insert(&mut self, slot: usize, snapshot: crate::project_manager::SnapshotId,
        archive: std::sync::Arc<[u8]>, filter: BagFilter) -> Result<()> {
        use om9_core::budget::memory::MemoryClass;
        if self.project_brep_insert_worker.is_some() || self.project_capture_worker.is_some()
            || self.project_document_worker.is_some() || self.controller.is_running() {
            return Err(om9_core::Error::Busy);
        }
        let held = self.controller.scene_cache_usage()?;
        let projection_held = self.controller.named_brep_replacement_hold()?;
        let target = self.controller.document().id;
        let scene_publication = self.controller.scene().ok_or(om9_core::Error::Missing("jobbag scene"))?.publication_id();
        let coordinator = self.controller.coordinator();
        let physical = om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes));
        let input = coordinator.reserve_external_memory_with_allocated(MemoryClass::CpuCache,
            (archive.len() as u64).max(1), archive.len() as u64, physical, held)?;
        let current_snapshot = self.project_manager.active().slot(slot).and_then(|slot| slot.snapshot.as_ref())
            .filter(|current| current.id == snapshot).ok_or(om9_core::Error::Stale("BRep Job Bag changed"))?;
        let snapshot_owner = coordinator.reserve_external_memory_with_held(MemoryClass::CpuCache,
            current_snapshot.clone_owned_bytes_estimate()?.max(1), physical, held)?;
        let snapshot_copy = current_snapshot.clone();
        let peer = coordinator.capture_transaction_preparation_peer(held)?;
        let cleanup = coordinator.memory_admission_peer_with_held(held)?;
        let host = crate::brep_restore_host::BrepRestoreHost::admit_packaged(om9_native_owner::ProcessLimits::default())?;
        let cancel = om9_core::transaction::Cancellation::default();
        let worker_cancel = cancel.clone();
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        #[cfg(feature = "test-fixtures")]
        let ready = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        #[cfg(feature = "test-fixtures")]
        let worker_ready = ready.clone();
        let worker = ProjectBrepInsertWorker { receiver, cancel, project: self.project_manager.active().id,
            slot, snapshot, generation: self.project_manager_generation, revision: self.project_manager.change_revision(),
            archive: std::sync::Arc::downgrade(&archive),
            selection_revision: self.controller.selection_state().revision, sub_revision: self.controller.sub_revision(), scene_publication,
            #[cfg(feature = "test-fixtures")] ready };
        std::thread::Builder::new().name("om9-brep-jobbag-restore".into()).spawn(move || {
            let _snapshot_owner = snapshot_owner;
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let bound = host.restore_bound_archive(peer.coordinator(), &archive, true, held, &worker_cancel)?;
                let cache = crate::session::shared_brep_archive_cache(peer.coordinator(), bound.document(), archive.clone(), input, held)?;
                Ok(crate::session::Controller::prepare_bound_jobbag_document(peer.coordinator(), bound, &snapshot_copy,
                    filter, target, projection_held, held, &worker_cancel)?.retain_cache(cache))
            })).unwrap_or_else(|_| Err(om9_core::Error::Io("BRep Job Bag worker panicked".into())));
            if result.is_err() {
                peer.coordinator().prune_orphaned_brep_representations();
                peer.coordinator().prune_orphaned_native_representations();
            }
            #[cfg(feature = "test-fixtures")]
            let prepared = result.is_ok();
            let result = result.map(|value| value.retain_peer(peer));
            if let Err(unsent) = sender.send(result) {
                drop(unsent);
                cleanup.prune_orphaned_brep_representations();
                cleanup.prune_orphaned_native_representations();
            }
            #[cfg(feature = "test-fixtures")]
            worker_ready.store(prepared, std::sync::atomic::Ordering::Release);
        }).map_err(|error| om9_core::Error::Io(format!("BRep Job Bag worker start: {error}")))?;
        self.project_brep_insert_worker = Some(worker);
        self.status.info("Restoring jobbag…");
        Ok(())
    }

    #[cfg(feature = "occt-brep")]
    pub(super) fn update_project_brep_insert(&mut self, context: &egui::Context) {
        let Some(worker) = self.project_brep_insert_worker.take() else { return; };
        let result = match worker.receiver.try_recv() {
            Ok(result) => result,
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                self.project_brep_insert_worker = Some(worker);
                context.request_repaint_after(std::time::Duration::from_millis(100));
                return;
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => Err(om9_core::Error::Io("BRep Job Bag worker exited".into())),
        };
        let current = self.project_manager.active().id == worker.project && self.project_manager_generation == worker.generation
            && self.project_manager.change_revision() == worker.revision
            && self.project_manager.active().slot(worker.slot).and_then(|slot| slot.snapshot.as_ref()).is_some_and(|snapshot|
                snapshot.id == worker.snapshot && snapshot.archive.as_ref().is_some_and(|archive| std::sync::Weak::ptr_eq(&worker.archive, &std::sync::Arc::downgrade(archive))))
            && self.controller.scene().is_some_and(|scene| scene.publication_id() == worker.scene_publication)
            && self.controller.selection_state().revision == worker.selection_revision && self.controller.sub_revision() == worker.sub_revision;
        let outcome = match result {
            Ok(prepared) if worker.cancel.is_canceled() || !current => {
                drop(prepared);
                self.controller.coordinator().prune_orphaned_brep_representations();
                self.controller.coordinator().prune_orphaned_native_representations();
                Err(if worker.cancel.is_canceled() { om9_core::Error::Canceled } else { om9_core::Error::Stale("BRep Job Bag changed") })
            }
            Ok(prepared) => self.controller.insert_prepared_brep_jobbag(prepared, &worker.cancel).map(|_| ()),
            Err(error) => Err(error),
        };
        self.feedback(outcome);
        context.request_repaint();
    }

    #[cfg(not(feature = "occt-brep"))]
    pub(super) fn update_project_brep_insert(&mut self, _context: &egui::Context) {}

    fn project_master_ui(&mut self, ui: &mut egui::Ui, name: &'static str) {
        ui.push_id(name, |ui| {
            ui.vertical(|ui| {
                let key = (self.project_manager.active().id, name);
                let versions: Vec<_> = self.project_manager.named_master_versions(name).iter().map(|snapshot| {
                    (snapshot.id,
                        snapshot.object_count.or_else(|| snapshot.document.as_ref().map(|doc| doc.entities.len())).unwrap_or(0),
                        snapshot.layer_count.or_else(|| snapshot.document.as_ref().map(|doc| doc.layer_order.len())).unwrap_or(0),
                        snapshot.thumbnail_png.clone(), snapshot.archive.is_some())
                }).collect();
                let selected = self.project_selected_versions.get(&key).copied()
                    .filter(|id| versions.iter().any(|(candidate, _, _, _, _)| candidate == id))
                    .or_else(|| versions.last().map(|(id, _, _, _, _)| *id));
                let version = versions.iter().find(|(id, _, _, _, _)| Some(*id) == selected);
                let summary = version.map(|(_, objects, layers, _, _)| format!("{} objects\n{} layers", objects, layers)).unwrap_or_default();
                let texture = version.and_then(|(id, _, _, png, _)| self.project_thumbnail_texture(ui.ctx(), key.0, *id, png.as_deref()));
                let available = selected.is_some();
                let saveable = version.is_some_and(|(_, _, _, _, has_archive)| *has_archive);
                let (rect, _) = ui.allocate_exact_size(Vec2::new(68., 60.), Sense::hover());
                self.project_bag_rects.insert(name, rect);
                paint_bag(ui, rect, name, &summary, false, texture);
                let restore = overlay_button(ui, Rect::from_min_size(rect.right_top() - Vec2::new(18., 0.), Vec2::new(18., 18.)), "restore", "▶");
                if name == "Master" { self.project_action_rects.insert("restore-master", restore.rect); }
                if restore.on_hover_text("Open the captured document").clicked() && available {
                    let result = self.begin_named_jobbag_open(name, selected.unwrap());
                    self.feedback(result);
                }
                let capture = overlay_button(ui, Rect::from_min_size(rect.right_top() + Vec2::new(-18., 19.), Vec2::new(18., 18.)), "capture", "+");
                if name == "Master" { self.project_action_rects.insert("capture-master", capture.rect); }
                if capture.on_hover_text("Capture the whole document, including hidden objects and layers").clicked() {
                    let result = self.start_document_snapshot(name, None);
                    self.feedback(result);
                }
                if versions.len() > 1 {
                    let selected_index = versions.iter().position(|(id, _, _, _, _)| Some(*id) == selected).unwrap_or(versions.len() - 1);
                    egui::ComboBox::from_id_salt(("named-version", key)).width(68.).selected_text(format!("{} / {}", selected_index + 1, versions.len())).show_ui(ui, |ui| {
                        for (index, (id, _, _, _, _)) in versions.iter().enumerate() {
                            if ui.selectable_label(Some(*id) == selected, format!("Version {}", index + 1)).clicked() {
                                self.project_selected_versions.insert(key, *id);
                            }
                        }
                    });
                }
                ui.horizontal(|ui| {
                    if self.project_icon_button(ui, "ProjectOut", Vec2::new(20., 20.), true, "Capture the whole document into this version").clicked() {
                        if let Some(snapshot) = selected {
                            self.project_confirmation = Some((ProjectConfirmation::OverwriteVersion(key.0, name, snapshot, self.controller.document().id, self.controller.document().revision), self.project_manager_generation));
                        } else {
                            let result = self.start_document_snapshot(name, None); self.feedback(result);
                        }
                    }
                    let save = self.project_icon_button(ui, "ProjectSave", Vec2::new(20., 20.), saveable,
                        if saveable { "Save snapshot as an OM9 file" } else { "No native archive is available for this snapshot" });
                    if name == "Master" { self.project_action_rects.insert("save-master", save.rect); }
                    if save.clicked() {
                        self.project_file_request = Some((ProjectFileAction::ExportVersion(key.0, name, selected.unwrap()), self.project_manager_generation));
                    }
                    if self.project_icon_button(ui, "ProjectDelete", Vec2::new(20., 20.), available, "Clear this bag; keep viewport objects").clicked() {
                        self.project_confirmation = Some((ProjectConfirmation::ClearVersion(key.0, name, selected.unwrap()), self.project_manager_generation));
                    }
                });
            });
        });
    }

    pub(super) fn project_metadata_ui(&mut self, ui: &mut egui::Ui) {
        ui.scope(|ui| {
            if let Some(worker) = &self.project_export_worker {
                ui.horizontal(|ui| {
                    ui.label("Exporting archive…");
                    if ui.small_button("Cancel").clicked() { worker.cancel(); }
                });
            }
            if let Some(worker) = &self.project_root_worker {
                ui.horizontal(|ui| {
                    ui.label("Changing project folder…");
                    if ui.small_button("Cancel").clicked() { worker.cancel(); }
                });
            }
            if let Some(error) = self.project_catalog_error.clone() {
                ui.colored_label(super::theme::ERROR, format!("Catalog not saved/opened: {error}"));
                if self.project_catalog_ready && ui.button("Retry saving catalog").clicked() {
                    self.project_catalog_attempted = None;
                    self.persist_project_catalog();
                }
            }
            ui.horizontal(|ui| {
                let mut all_visible = self.project_manager.rows().iter().all(|row| row.visible);
                if ui.checkbox(&mut all_visible, "Show all").changed() {
                    let ids: Vec<_> = self.project_manager.rows().iter().map(|row| row.id).collect();
                    for id in ids { let _ = self.project_manager.set_visible(id, all_visible); }
                }
                ui.label(format!("{} projects", self.project_manager.rows().len()));
                if self.preserve_embedded_project_state() {
                    let import = ui.button("Import document projects…")
                        .on_hover_text("Add the document's projects as separate workspace rows. Repeating this creates additional rows.");
                    self.project_action_rects.insert("import-document-projects", import.rect);
                    if import.clicked() { let result = self.start_embedded_project_import(); self.feedback(result); }
                }
            });
            ui.separator();
            // Reserve space for wrapping actions and keep the footer inside
            // the visible screen even while egui sizes the window initially.
            let body_height = (ui.available_height() - 100.).clamp(100., (ui.ctx().content_rect().height() - 240.).max(100.));
            ui.horizontal_top(|ui| {
                let grid_width = (ui.available_width() - 180.).max(150.);
                ui.allocate_ui_with_layout(Vec2::new(grid_width, body_height), egui::Layout::top_down(egui::Align::Min), |ui| {
                    let query = self.project_applied_search.to_lowercase();
                    let filters = self.project_applied_filters.compile();
                    let rows: Vec<_> = self.project_manager.rows().iter().filter(|row| {
                        let general = query.is_empty() || [&row.name, &row.customer, &row.notes, &row.metadata.designer, &row.metadata.status, &row.metadata.job_number, &row.metadata.description]
                            .iter().any(|value| value.to_lowercase().contains(&query)) || row.metadata.tags.iter().any(|tag| tag.to_lowercase().contains(&query));
                        general && filters.as_ref().is_ok_and(|filter| filter.matches(row))
                    }).map(|row| {
                        let numbered = row.slots.iter().filter_map(|slot| slot.snapshot.as_ref())
                            .find_map(|snapshot| snapshot.thumbnail.png.clone().map(|png| (snapshot.id, png)));
                        let named = || row.master.iter().chain(row.named_masters.values()).chain(row.named_versions.values().flatten())
                            .filter_map(|snapshot| snapshot.thumbnail_png.clone().map(|png| (snapshot.id, png)))
                            .max_by_key(|(snapshot, _)| snapshot.0);
                        (row.id, row.name.clone(), row.visible, numbered.or_else(named))
                    }).collect();
                    egui::ScrollArea::vertical().id_salt("manager-project-grid").max_height(body_height).show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            for (id, name, mut visible, thumbnail) in rows {
                                ui.vertical(|ui| {
                                    let texture = thumbnail.as_ref().and_then(|(snapshot, png)| self.project_thumbnail_texture(ui.ctx(), id, *snapshot, Some(png)));
                                    let (rect, response) = ui.allocate_exact_size(Vec2::splat(120.), Sense::click());
                                    self.project_action_rects.insert("manager-project", response.rect);
                                    paint_bag(ui, rect, "", "", false, texture);
                                    if response.clicked() && self.project_draft.is_none() {
                                        let result = self.project_manager.select(id);
                                        if result.is_ok() { self.project_draft = Some(ProjectDraft::from_row(self.project_manager.active(), self.project_manager_generation)); }
                                        self.feedback(result);
                                    }
                                    if ui.checkbox(&mut visible, &name).changed() {
                                        let result = self.project_manager.set_visible(id, visible); self.feedback(result);
                                    }
                                });
                            }
                        });
                    });
                });
                let (separator, _) = ui.allocate_exact_size(Vec2::new(1., body_height), Sense::hover());
                ui.painter().vline(separator.center().x, separator.y_range(), ui.visuals().widgets.noninteractive.bg_stroke);
                ui.allocate_ui_with_layout(Vec2::new(170., body_height), egui::Layout::top_down(egui::Align::Min), |ui| {
                    self.project_action_rects.insert("manager-filter-panel", ui.max_rect());
                    ui.strong("Text Search");
                    let search = ui.text_edit_singleline(&mut self.project_search);
                    self.project_action_rects.insert("manager-search", search.rect);
                    if let Err(error) = self.project_filters.compile() { ui.colored_label(super::theme::ERROR, format!("{error}")); }
                    ui.separator(); ui.strong("Search Filters");
                    let filter_scroll = egui::ScrollArea::vertical().id_salt("manager-overview-filters").max_height((body_height - 70.).max(50.)).show(ui, |ui| {
                        let filters = &mut self.project_filters;
                        for (label, value) in [("Name", &mut filters.name), ("Customer Name", &mut filters.customer), ("Designer", &mut filters.designer), ("Tags", &mut filters.tag), ("Status", &mut filters.status), ("Job number", &mut filters.job_number)] {
                            ui.collapsing(label, |ui| { ui.text_edit_singleline(value); });
                        }
                        let gem = ui.collapsing("Gem Total Weight", |ui| filter_range(ui, "ct", &mut filters.gem_min, &mut filters.gem_max));
                        self.project_action_rects.insert("manager-gem-filter", gem.header_response.rect);
                        if let Some((min, max)) = gem.body_returned {
                            self.project_action_rects.insert("manager-gem-min", min);
                            self.project_action_rects.insert("manager-gem-max", max);
                        }
                        ui.collapsing("Metal Total Weight", |ui| filter_range(ui, "g", &mut filters.metal_min, &mut filters.metal_max));
                        ui.collapsing("CAD Labor", |ui| filter_range(ui, "hours", &mut filters.cad_min, &mut filters.cad_max));
                        ui.collapsing("CAM Labor", |ui| filter_range(ui, "hours", &mut filters.cam_min, &mut filters.cam_max));
                        ui.collapsing("Ring Rail Radius", |ui| filter_range(ui, "mm", &mut filters.radius_min, &mut filters.radius_max));
                        ui.collapsing("Creation Date", |ui| filter_range(ui, "YYYY-MM-DD", &mut filters.created_from, &mut filters.created_to));
                        ui.collapsing("Due Date", |ui| filter_range(ui, "YYYY-MM-DD", &mut filters.due_from, &mut filters.due_to));
                        let jobs = ui.collapsing("Jobs On/Off", |ui| {
                            ui.selectable_value(&mut filters.visibility, None, "All");
                            ui.selectable_value(&mut filters.visibility, Some(true), "Visible");
                            let hidden = ui.selectable_value(&mut filters.visibility, Some(false), "Hidden");
                            self.project_action_rects.insert("manager-hidden-filter", hidden.rect);
                        });
                        self.project_action_rects.insert("manager-jobs-filter", jobs.header_response.rect);
                    });
                    self.project_action_rects.insert("manager-filters-viewport", filter_scroll.inner_rect);
                    ui.horizontal(|ui| {
                        let search = ui.button("Search");
                        self.project_action_rects.insert("manager-search-submit", search.rect);
                        if search.clicked() && self.project_filters.compile().is_ok() {
                            self.project_applied_search = self.project_search.clone();
                            self.project_applied_filters = self.project_filters.clone();
                        }
                        let reset = ui.button("Reset");
                        self.project_action_rects.insert("manager-search-reset", reset.rect);
                        if reset.clicked() {
                            self.project_search.clear(); self.project_filters = Default::default();
                            self.project_applied_search.clear(); self.project_applied_filters = Default::default();
                        }
                    });
                });
            });
            ui.separator();
            ui.horizontal_wrapped(|ui| {
                let refresh = ui.button("Refresh…");
                self.project_action_rects.insert("manager-refresh", refresh.rect);
                if refresh.clicked() { self.project_confirmation = Some((ProjectConfirmation::ReloadCatalog, self.project_manager_generation)); }
                let import_project = ui.button("Import Project…");
                self.project_action_rects.insert("manager-import-project", import_project.rect);
                if import_project.clicked() { self.project_file_request = Some((ProjectFileAction::ImportRow, self.project_manager_generation)); }
                let import_folder = ui.button("Import Folder…");
                self.project_action_rects.insert("manager-import-folder", import_folder.rect);
                if import_folder.clicked() { self.project_file_request = Some((ProjectFileAction::ImportFolder, self.project_manager_generation)); }
                let change_root = ui.button("Change…");
                self.project_action_rects.insert("manager-change-root", change_root.rect);
                if change_root.clicked() { self.project_file_request = Some((ProjectFileAction::ChangeRoot, self.project_manager_generation)); }
                let export = ui.add_enabled(!self.project_manager.active().has_legacy_bags(), egui::Button::new("Export…"))
                    .on_disabled_hover_text("Legacy bags need native archives before project export");
                self.project_action_rects.insert("manager-export", export.rect);
                if export.clicked() { self.project_file_request = Some((ProjectFileAction::ExportRow(self.project_manager.active().id), self.project_manager_generation)); }
                let open_folder = ui.button("Open folder");
                self.project_action_rects.insert("manager-open-folder", open_folder.rect);
                if open_folder.clicked() { let result = self.open_project_folder(); self.feedback(result); }
                if let Some(path) = self.project_catalog_path.as_deref().and_then(std::path::Path::parent) { ui.label(format!("Projects Location: {}", path.display())); }
            });
        });

    }

    pub(super) fn project_details_ui(&mut self, ui: &mut egui::Ui) {
        let context = ui.ctx().clone();
        let draft_current = self.project_draft.as_ref().is_some_and(|draft| {
            draft.generation == self.project_manager_generation && draft.id == self.project_manager.active().id
        });
        if !draft_current { self.project_files_tab = false; }
        ui.scope(|ui| {
            if let Some(error) = self.project_catalog_error.clone() {
                ui.colored_label(super::theme::ERROR, format!("Catalog not saved/opened: {error}"));
                if self.project_catalog_ready && ui.button("Retry saving catalog").clicked() {
                    self.project_catalog_attempted = None;
                    self.persist_project_catalog();
                }
            }
            if draft_current { ui.heading(&self.project_manager.active().name); }
            else {
                ui.heading("Project draft");
                ui.colored_label(super::theme::ERROR, "The project or catalog changed. Copy any needed edits, then cancel this draft.");
            }
            if draft_current {
                let timer = ui.label(project_timer_text(self.project_manager.active()));
                self.project_action_rects.insert("manager-timer", timer.rect);
                if self.project_manager.active().timer_started_unix.is_some() {
                    context.request_repaint_after(std::time::Duration::from_secs(1));
                }
            }
            let directory = ui.add_enabled_ui(draft_current, |ui| ui.collapsing("Customers, designers and statuses", |ui| {
                for (kind, label) in [(DirectoryKind::Customer, "Customers"), (DirectoryKind::Designer, "Designers"), (DirectoryKind::Status, "Statuses")] {
                    ui.horizontal(|ui| {
                        ui.strong(label);
                        if ui.small_button("Add…").clicked() { self.project_directory_editor = Some(DirectoryEditor { kind, id: None, name: String::new() }); }
                    });
                    let entries = self.project_manager.directory_entries(kind).to_vec();
                    for entry in entries {
                        ui.horizontal(|ui| {
                            ui.label(&entry.name);
                            let edit = ui.small_button("Edit…");
                            if kind == DirectoryKind::Customer { self.project_action_rects.insert("directory-customer-edit", edit.rect); }
                            if edit.clicked() { self.project_directory_editor = Some(DirectoryEditor { kind, id: Some(entry.id), name: entry.name.clone() }); }
                            let remove = ui.small_button("Remove…");
                            if kind == DirectoryKind::Customer { self.project_action_rects.insert("directory-customer-remove", remove.rect); }
                            if remove.clicked() { self.project_confirmation = Some((ProjectConfirmation::RemoveDirectory(kind, entry.id), self.project_manager_generation)); }
                        });
                    }
                }
            }));
            self.project_action_rects.insert("manager-directory", directory.inner.header_response.rect);
            ui.separator();
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.project_files_tab, false, "Info");
                let files = ui.add_enabled(draft_current, egui::Button::new("Files").selected(self.project_files_tab));
                if files.clicked() { self.project_files_tab = true; }
                self.project_action_rects.insert("manager-files", files.rect);
            });
            if self.project_files_tab {
                self.project_files_preview_ui(ui);
            } else if let Some(mut draft) = self.project_draft.take() {
                if draft_current { draft.sync_directory(&self.project_manager); }
                let mut save = false;
                let mut cancel = false;
                let info = egui::ScrollArea::vertical().id_salt("project-info-fields").max_height(350.).show(ui, |ui| {
                    ui.label("Info (weights and labor are entered manually)");
                    ui.label("Name");
                    let name = ui.text_edit_singleline(&mut draft.name);
                    self.project_action_rects.insert("manager-info-name", name.rect);
                    directory_choice(ui, "Customer", DirectoryKind::Customer, self.project_manager.directory_entries(DirectoryKind::Customer), &mut draft.metadata.customer_id, &mut draft.customer);
                    directory_choice(ui, "Designer", DirectoryKind::Designer, self.project_manager.directory_entries(DirectoryKind::Designer), &mut draft.metadata.designer_id, &mut draft.metadata.designer);
                    directory_choice(ui, "Status", DirectoryKind::Status, self.project_manager.directory_entries(DirectoryKind::Status), &mut draft.metadata.status_id, &mut draft.metadata.status);
                    ui.label("Tags (comma separated)"); ui.text_edit_singleline(&mut draft.tags);
                    ui.label("Created (YYYY-MM-DD)"); ui.text_edit_singleline(&mut draft.created);
                    ui.label("Due (YYYY-MM-DD)"); ui.text_edit_singleline(&mut draft.due);
                    ui.label("Job number"); ui.text_edit_singleline(&mut draft.metadata.job_number);
                    ui.label("Ring size and standard"); ui.text_edit_singleline(&mut draft.metadata.ring_size);
                    optional_measure(ui, "Ring rail radius (mm)", &mut draft.metadata.ring_rail_radius_mm);
                    optional_measure(ui, "CAD labor (hours)", &mut draft.metadata.cad_labor_hours);
                    optional_measure(ui, "CAM labor (hours)", &mut draft.metadata.cam_labor_hours);
                    optional_measure(ui, "Metal weight (g)", &mut draft.metadata.metal_weight_grams);
                    optional_measure(ui, "Gem weight (ct)", &mut draft.metadata.gem_weight_carats);
                    ui.label("Description"); ui.text_edit_multiline(&mut draft.metadata.description);
                    ui.label("Notes"); ui.text_edit_multiline(&mut draft.notes);
                    ui.horizontal(|ui| {
                        let save_button = ui.add_enabled(draft_current, egui::Button::new("Save"));
                        self.project_action_rects.insert("manager-info-save", save_button.rect);
                        save = save_button.clicked();
                        let cancel_button = ui.button("Cancel");
                        self.project_action_rects.insert("manager-info-cancel", cancel_button.rect);
                        cancel = cancel_button.clicked();
                    });
                });
                self.project_action_rects.insert("manager-info-viewport", info.inner_rect);
                if save {
                    let metadata = draft.prepare_metadata();
                    let result = self.project_manager.update_project(draft.id, &draft.name, &draft.customer, &draft.notes, metadata);
                    if result.is_err() { self.project_draft = Some(draft); }
                    self.feedback(result);
                } else if !cancel { self.project_draft = Some(draft); }
            } else { ui.label("Choose a project to edit its Info."); }
        });
    }

    fn project_files_preview_ui(&mut self, ui: &mut egui::Ui) {
        let project = self.project_manager.active().id;
        let mut items = Vec::new();
        for name in MASTER_NAMES {
            for (index, snapshot) in self.project_manager.named_master_versions(name).iter().enumerate() {
                let label = format!("{name} {}", index + 1);
                let summary = format!("{} objects", snapshot.object_count.or_else(|| snapshot.document.as_ref().map(|doc| doc.entities.len())).unwrap_or(0));
                items.push((ProjectFilePreview::Named(name, snapshot.id), label, summary, snapshot.thumbnail_png.clone()));
            }
        }
        for (slot, bag) in self.project_manager.active().slots.iter().enumerate() {
            if let Some(snapshot) = &bag.snapshot {
                items.push((ProjectFilePreview::Numbered(slot, snapshot.id), format!("{}", slot + 1),
                    format!("{} {}", snapshot.thumbnail.count, snapshot.thumbnail.kind), snapshot.thumbnail.png.clone()));
            }
        }
        let selected = self.project_file_preview.filter(|(id, key)| *id == project && items.iter().any(|item| item.0 == *key))
            .map(|(_, key)| key).or_else(|| items.iter().rev().find(|item| matches!(item.0, ProjectFilePreview::Named("Master", _))).map(|item| item.0))
            .or_else(|| items.first().map(|item| item.0));
        if let Some(key) = selected { self.project_file_preview = Some((project, key)); }
        ui.horizontal(|ui| {
            ui.label(format!("Saved bags for {}", self.project_manager.active().name));
            let export = ui.add_enabled(!self.project_manager.active().has_legacy_bags(), egui::Button::new("Export project…"))
                .on_disabled_hover_text("Legacy bags need native archives before project export");
            self.project_action_rects.insert("manager-files-export", export.rect);
            if export.clicked() { self.project_file_request = Some((ProjectFileAction::ExportRow(project), self.project_manager_generation)); }
            if ui.button("Open folder").clicked() { let result = self.open_project_folder(); self.feedback(result); }
        });
        ui.horizontal_top(|ui| {
            ui.vertical(|ui| {
                let (rect, _) = ui.allocate_exact_size(Vec2::splat(190.), Sense::hover());
                self.project_action_rects.insert("manager-file-preview", rect);
                if let Some((key, label, summary, png)) = selected.and_then(|key| items.iter().find(|item| item.0 == key)) {
                    let texture = self.project_thumbnail_texture(ui.ctx(), project, key.snapshot_id(), png.as_deref());
                    paint_bag(ui, rect, label, summary, false, texture);
                } else { paint_bag(ui, rect, "Master", "No saved bags", false, None); }
            });
            egui::ScrollArea::vertical().id_salt("manager-file-previews").max_height(260.).show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for (key, label, summary, png) in &items {
                        let texture = self.project_thumbnail_texture(ui.ctx(), project, key.snapshot_id(), png.as_deref());
                        let (rect, response) = ui.allocate_exact_size(Vec2::new(76., 74.), Sense::click());
                        self.project_action_rects.insert("manager-file-thumbnail", rect);
                        paint_bag(ui, rect, label, summary, matches!(key, ProjectFilePreview::Numbered(..)), texture);
                        if Some(*key) == selected { ui.painter().rect_stroke(rect, 0, egui::Stroke::new(2., Color32::YELLOW), egui::StrokeKind::Inside); }
                        if response.clicked() { self.project_file_preview = Some((project, *key)); }
                        if response.double_clicked() {
                            let result = match *key {
                                ProjectFilePreview::Numbered(slot, _) => self.insert_project_slot(slot, BagFilter::All),
                                ProjectFilePreview::Named(name, id) => self.begin_named_jobbag_open(name, id),
                            };
                            self.feedback(result);
                        }
                    }
                });
            });
        });
    }
}

fn paint_bag(ui: &egui::Ui, rect: Rect, title: &str, summary: &str, numbered: bool, texture: Option<egui::TextureId>) {
    let painter = ui.painter();
    painter.rect_filled(rect, 0, if numbered { Color32::from_rgb(0, 10, 70) } else { Color32::from_rgb(65, 67, 66) });
    painter.rect_stroke(rect, 0, egui::Stroke::new(1., Color32::from_gray(145)), egui::StrokeKind::Inside);
    painter.text(rect.left_top() + Vec2::new(2., 2.), egui::Align2::LEFT_TOP, title, egui::FontId::proportional(10.), Color32::WHITE);
    if let Some(texture) = texture {
        let image_rect = Rect::from_min_max(rect.left_top() + Vec2::new(3., 16.), rect.right_bottom() - Vec2::new(3., 3.));
        painter.image(texture, image_rect, Rect::from_min_max(egui::pos2(0., 0.), egui::pos2(1., 1.)), Color32::WHITE);
    } else {
        painter.text(rect.center() + Vec2::new(0., 8.), egui::Align2::CENTER_CENTER, summary, egui::FontId::proportional(9.), Color32::WHITE);
    }
}

#[cfg(test)]
mod project_filter_tests {
    use super::*;

    #[test]
    fn close_cancels_and_drains_thumbnail_worker_with_panel_hidden() {
        let mut app = MatrixApp::new(om9_core::model::Document::new("thumbnail close")).unwrap();
        let reservation = app.controller.coordinator().reserve_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::CpuCache, 1, None, app.controller.scene_cache_usage().unwrap(),
        ).unwrap();
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        let cancel = om9_core::transaction::Cancellation::default();
        app.project_thumbnail_worker = Some(ProjectThumbnailWorker { receiver, cancel: cancel.clone() });
        let context = egui::Context::default();
        let mut input = egui::RawInput::default();
        input.viewports.get_mut(&egui::ViewportId::ROOT).unwrap().events.push(egui::ViewportEvent::Close);
        let _ = context.run_logic(&input, |ctx| { assert!(!app.service_window_close(ctx)); });
        assert!(cancel.is_canceled());
        assert!(app.project_capture_close_wait);
        assert!(sender.send((crate::project_manager::ProjectId(1), crate::project_manager::SnapshotId(1), [0; 32].into(), Err(om9_core::Error::Canceled), reservation)).is_ok());
        app.service_project_background(&context);
        assert!(app.project_thumbnail_worker.is_none());
        assert!(!app.project_capture_close_wait);
    }

    #[test]
    fn captured_archive_reservation_follows_shared_asset_lifetime() {
        let mut app = MatrixApp::new(om9_core::model::Document::new("asset lifetime")).unwrap();
        app.controller.submit("Point").unwrap();
        app.controller.submit("1,2,3").unwrap();
        let id = *app.controller.document().entities.keys().next().unwrap();
        app.controller.select_objects(&[id]).unwrap();
        app.capture_project_slot(0, BagFilter::All).unwrap();
        let context = egui::Context::default();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while app.project_capture_worker.is_some() && std::time::Instant::now() < deadline {
            app.update_project_capture(&context);
            std::thread::yield_now();
        }
        assert!(app.project_capture_worker.is_none());
        assert_eq!(app.project_asset_leases.len(), 1);
        app.stage_project_state().unwrap();
        let reopened = MatrixApp::new(app.controller.document().clone()).unwrap();
        assert!(!reopened.project_asset_leases.is_empty());
        assert!(reopened.project_manager.active().slot(0).unwrap().snapshot.is_some());
        drop(reopened);
        let shared = app.project_manager.active().slot(0).unwrap().snapshot.as_ref().unwrap().archive.clone().unwrap();
        assert!(app.project_asset_is_leased(&shared));
        app.project_manager.clear_slot(0).unwrap();
        app.service_project_catalog(&context);
        assert_eq!(app.project_asset_leases.len(), 1);
        drop(shared);
        app.service_project_catalog(&context);
        assert!(app.project_asset_leases.is_empty());
    }

    #[test]
    fn refresh_rejects_catalog_with_corrupt_bag_and_keeps_current_projects() {
        let folder = std::env::temp_dir().join(format!("om9-project-refresh-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&folder).unwrap();
        let path = folder.join("catalog.json");
        let mut app = MatrixApp::new(om9_core::model::Document::new("current")).unwrap();
        app.controller.submit("Point").unwrap();
        app.controller.submit("1,2,3").unwrap();
        let selected = app.controller.document().entities.values().cloned().collect();
        let mut corrupt = crate::project_manager::ProjectManager::new("corrupt catalog");
        corrupt.capture_archive(0, selected, app.controller.document(), b"bad archive".to_vec()).unwrap();
        crate::project_catalog::save(&path, &corrupt, None).unwrap();
        app.project_catalog_path = Some(path);
        let before = app.project_manager.active().name.clone();
        app.reload_project_catalog().unwrap();
        let context = egui::Context::default();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while app.project_catalog_load_worker.is_some() && std::time::Instant::now() < deadline {
            app.service_project_catalog(&context);
            std::thread::yield_now();
        }
        assert!(app.project_catalog_load_worker.is_none());
        assert!(app.project_catalog_error.is_some());
        assert_eq!(app.project_manager.active().name, before);
        std::fs::remove_dir_all(folder).unwrap();
    }

    #[test]
    fn refresh_does_not_overwrite_edits_made_while_loading() {
        let folder = std::env::temp_dir().join(format!("om9-project-stale-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&folder).unwrap();
        let path = folder.join("catalog.json");
        let saved = crate::project_manager::ProjectManager::new("on disk");
        crate::project_catalog::save(&path, &saved, None).unwrap();
        let mut app = MatrixApp::new(om9_core::model::Document::new("current")).unwrap();
        app.project_catalog_path = Some(path);
        app.reload_project_catalog().unwrap();
        let id = app.project_manager.active().id;
        app.project_manager.rename_project(id, "edited while loading").unwrap();
        let context = egui::Context::default();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while app.project_catalog_load_worker.is_some() && std::time::Instant::now() < deadline {
            app.service_project_catalog(&context);
            std::thread::yield_now();
        }
        assert!(app.project_catalog_load_worker.is_none());
        assert_eq!(app.project_manager.active().name, "edited while loading");
        assert!(app.project_catalog_error.is_some());
        std::fs::remove_dir_all(folder).unwrap();
    }

    #[test]
    fn typed_ranges_require_present_values_and_valid_dates() {
        let filters = ProjectFilters { created_from: "2026-01-01".into(), created_to: "2026-12-31".into(),
            metal_min: "4".into(), metal_max: "6".into(), ..Default::default() };
        let mut row = crate::project_manager::ProjectRow::new(crate::project_manager::ProjectId(1), "Ring", 1);
        row.metadata.created_date = Some("2026-06-01".into());
        assert!(!filters.compile().unwrap().matches(&row));
        row.metadata.metal_weight_grams = Some(5.0);
        assert!(filters.compile().unwrap().matches(&row));
        row.metadata.metal_weight_grams = Some(7.0);
        assert!(!filters.compile().unwrap().matches(&row));
        let bad = ProjectFilters { due_from: "2025-02-29".into(), ..Default::default() };
        assert!(bad.compile().is_err());
    }
}
