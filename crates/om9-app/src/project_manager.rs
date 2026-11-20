// SPDX-License-Identifier: LGPL-2.1-or-later
//! Matrix Project Manager and Job Bag state (OM9-L2-03-15).
//!
//! This module owns project rows, numbered job-bag slots, the Master
//! job bag and project notes. It is deliberately independent of egui and of the
//! document transaction path: snapshots clone entities; restoring hands fresh
//! entities back to the caller, which keeps Undo/Redo in the Controller.
use om9_core::{
    Error, Result,
    identity::ObjectId,
    model::{Document, Entity},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;

pub const SLOT_COUNT: usize = 10;
pub const MAX_SLOTS: usize = 200;
pub const MASTER_NAMES: [&str; 5] = ["Master", "Creation", "Parts", "Render", "Output"];
const MAX_PROJECTS: usize = 64;
const MAX_NAME: usize = 64;
const MAX_NOTES: usize = 4096;
const MAX_CUSTOMER: usize = 128;
const MAX_MANAGER_JSON_BYTES: usize = 16 * 1024 * 1024;

struct ManagerJsonWriter { bytes: Vec<u8>, refused: bool }
impl std::io::Write for ManagerJsonWriter {
    fn write(&mut self, input: &[u8]) -> std::io::Result<usize> {
        let required = self.bytes.len().checked_add(input.len()).filter(|n| *n <= MAX_MANAGER_JSON_BYTES);
        let Some(required) = required else {
            self.refused = true;
            return Err(std::io::Error::other("project manager state budget"));
        };
        if required > self.bytes.capacity() {
            let capacity = required.max(self.bytes.capacity().saturating_mul(2).max(1024)).min(MAX_MANAGER_JSON_BYTES);
            if self.bytes.try_reserve_exact(capacity - self.bytes.len()).is_err() {
                self.refused = true;
                return Err(std::io::Error::other("project manager state allocation"));
            }
        }
        self.bytes.extend_from_slice(input);
        Ok(input.len())
    }
    fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
}

/// Stable, human-readable project identity. Duplicate labels are allowed; IDs
/// are never derived from the label.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ProjectId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectAction { OpenPanel, OpenManager }
impl ProjectAction {
    pub fn from_command(command: &str) -> Option<Self> {
        if command.trim().eq_ignore_ascii_case("gvProjectManager") { Some(Self::OpenPanel) }
        else if command.trim().eq_ignore_ascii_case("gvOpenProjectManager") { Some(Self::OpenManager) }
        else { None }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BagFilter { All, Gems, Curves, Surfaces, Polysurfaces, Meshes }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum BagObjectClass { Gem, Curve, Surface, Polysurface, Mesh, Other, UnknownBrep, UnknownInstance }
impl BagObjectClass {
    fn of(entity: &Entity) -> Self {
        use om9_core::geometry::Geometry;
        if entity.extensions.contains_key(om9_core::model::GEM_EXTENSION_SLOT) { return Self::Gem; }
        match &entity.geometry {
            Geometry::Curve(_) => Self::Curve,
            Geometry::Surface(_) => Self::Surface,
            Geometry::Mesh(_) => Self::Mesh,
            Geometry::Brep(shape) => match shape.topology().faces.len() { 0 => Self::UnknownBrep, 1 => Self::Surface, _ => Self::Polysurface },
            Geometry::NativeBrep(_) => Self::UnknownBrep,
            Geometry::Instance(_) => Self::UnknownInstance,
            _ => Self::Other,
        }
    }

    // A homogeneous instance can be filtered without flattening it. Mixed or
    // unclassifiable definitions remain unknown rather than silently dropping
    // part of a block or importing unrelated members.
    fn resolved(entity: &Entity, document: &Document, limits: &om9_core::budget::Limits, visited: &mut u64) -> Result<Self> {
        fn visit(entity: &Entity, document: &Document, limits: &om9_core::budget::Limits,
            depth: usize, visited: &mut u64) -> Result<BagObjectClass> {
            *visited = visited.checked_add(1).ok_or(Error::Overflow("jobbag classification"))?;
            if *visited > limits.max_nodes || depth > limits.max_depth { return Err(Error::Budget("jobbag classification")); }
            let class = BagObjectClass::of(entity);
            if class != BagObjectClass::UnknownInstance { return Ok(class); }
            let om9_core::geometry::Geometry::Instance(id) = entity.geometry else { unreachable!() };
            let definition = document.definitions.get(&id).ok_or(Error::Missing("jobbag instance definition"))?;
            let mut result = None;
            for member in &definition.members {
                let class = visit(member, document, limits, depth + 1, visited)?;
                if matches!(class, BagObjectClass::UnknownBrep | BagObjectClass::UnknownInstance)
                    || result.is_some_and(|previous| previous != class) {
                    return Ok(BagObjectClass::UnknownInstance);
                }
                result = Some(class);
            }
            Ok(result.unwrap_or(BagObjectClass::UnknownInstance))
        }
        visit(entity, document, limits, 0, visited)
    }
}
impl BagFilter {
    pub fn matches(self, entity: &Entity) -> Result<bool> {
        self.matches_class(BagObjectClass::of(entity))
    }
    pub fn matches_in_document(self, entity: &Entity, document: &Document, limits: &om9_core::budget::Limits) -> Result<bool> {
        self.matches_in_document_counted(entity, document, limits, &mut 0)
    }
    pub(crate) fn matches_in_document_counted(self, entity: &Entity, document: &Document,
        limits: &om9_core::budget::Limits, visited: &mut u64) -> Result<bool> {
        if self == Self::All { return Ok(true); }
        self.matches_class(BagObjectClass::resolved(entity, document, limits, visited)?)
    }
    fn matches_class(self, class: BagObjectClass) -> Result<bool> {
        if self == Self::All { return Ok(true); }
        if class == BagObjectClass::UnknownInstance { return Err(Error::Unsupported("jobbag filter requires resolved object classification")); }
        if self == Self::Gems { return Ok(class == BagObjectClass::Gem); }
        Ok(match (class, self) {
            (BagObjectClass::Curve, Self::Curves) | (BagObjectClass::Surface, Self::Surfaces)
                | (BagObjectClass::Polysurface, Self::Polysurfaces) | (BagObjectClass::Mesh, Self::Meshes) => true,
            (BagObjectClass::UnknownBrep, Self::Surfaces | Self::Polysurfaces) | (BagObjectClass::UnknownInstance, _) => {
                return Err(Error::Unsupported("jobbag filter requires resolved object classification"));
            }
            _ => false,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Thumbnail {
    pub label: String,
    pub count: usize,
    /// Geometry kind summary (`Mesh`, `Brep`, `Curve`, ...); no pixels are
    /// fabricated when the source geometry is unavailable.
    pub kind: String,
    pub source_revision: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub png: Option<Arc<[u8]>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    png_asset: Option<ProjectAssetRef>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub id: SnapshotId,
    pub source_document: om9_core::identity::DocumentId,
    pub source_revision: u64,
    pub captured_ids: Vec<ObjectId>,
    pub entities: Arc<[Entity]>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    root_classes: Vec<BagObjectClass>,
    /// Detached native selection archive; absent in legacy three-slot state.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archive: Option<Arc<[u8]>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    archive_asset: Option<ProjectAssetRef>,
    pub thumbnail: Thumbnail,
}

impl Snapshot {
    #[cfg(feature = "occt-brep")]
    pub(crate) fn clone_owned_bytes_estimate(&self) -> Result<u64> {
        use om9_core::budget::Usage;
        (std::mem::size_of::<Self>() as u64)
            .checked_add(Usage::array::<ObjectId>(self.captured_ids.len())?.bytes)
            .and_then(|n| n.checked_add(Usage::array::<BagObjectClass>(self.root_classes.len()).ok()?.bytes))
            .and_then(|n| n.checked_add(self.thumbnail.label.len() as u64))
            .and_then(|n| n.checked_add(self.thumbnail.kind.len() as u64))
            .and_then(|n| n.checked_add(1024))
            .ok_or(Error::Overflow("jobbag snapshot clone"))
    }

    fn new(id: SnapshotId, document: &Document, entities: Vec<Entity>, limits: &om9_core::budget::Limits) -> Result<Self> {
        if entities.is_empty() {
            return Err(Error::Invalid("jobbag capture selection"));
        }
        check_capture_roots(entities.len(), limits)?;
        let mut ids = Vec::new();
        ids.try_reserve_exact(entities.len()).map_err(|_| Error::Budget("jobbag object identities"))?;
        let mut seen = std::collections::BTreeSet::new();
        for entity in &entities {
            if !seen.insert(entity.id) {
                return Err(Error::Invalid("duplicate jobbag object"));
            }
            if document.entities.get(&entity.id) != Some(entity) {
                return Err(Error::Stale("jobbag selection source"));
            }
            ids.push(entity.id);
        }
        let kind = entities
            .iter()
            .map(|e| e.geometry.kind().tag())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>()
            .join("+");
        let label = if entities.len() == 1 {
            entities[0].label.clone()
        } else {
            format!("{} objects", entities.len())
        };
        Ok(Self {
            id,
            source_document: document.id,
            source_revision: document.revision,
            captured_ids: ids,
            thumbnail: Thumbnail {
                label,
                count: entities.len(),
                kind,
                source_revision: document.revision,
                png: None,
                png_asset: None,
            },
            entities: entities.into(),
            root_classes: Vec::new(),
            archive: None,
            archive_asset: None,
        })
    }

    /// Availability follows captured content; source identity is provenance.
    pub fn available(&self, _document: &Document) -> bool {
        !self.captured_ids.is_empty()
    }

    pub fn filtered_ids(&self, filter: BagFilter) -> Result<Vec<ObjectId>> {
        let classes: Vec<_> = if self.root_classes.len() == self.captured_ids.len() {
            self.root_classes.clone()
        } else if self.entities.len() == self.captured_ids.len() {
            self.entities.iter().map(BagObjectClass::of).collect()
        } else { return Err(Error::Invalid("jobbag object classes")); };
        let mut ids = Vec::new();
        for (id, class) in self.captured_ids.iter().zip(classes) {
            if filter.matches_class(class)? { ids.push(*id); }
        }
        Ok(ids)
    }

    /// Resolve filtering from the validated archived definitions. In
    /// particular, old manifests may have recorded an instance as unknown.
    pub fn filtered_ids_in_document(&self, filter: BagFilter, document: &Document,
        limits: &om9_core::budget::Limits) -> Result<Vec<ObjectId>> {
        self.validate_archive_roots(document, limits)?;
        let mut ids = Vec::new();
        ids.try_reserve_exact(self.captured_ids.len()).map_err(|_| Error::Budget("jobbag filter roots"))?;
        let mut visited = 0;
        for id in &self.captured_ids {
            let entity = document.entities.get(id).ok_or(Error::Missing("jobbag archive root"))?;
            if filter.matches_in_document_counted(entity, document, limits, &mut visited)? { ids.push(*id); }
        }
        Ok(ids)
    }

    /// Check the manifest's selected-root metadata against the decoded source
    /// before a filtered or unfiltered insert publishes anything.
    pub fn validate_archive_roots(&self, document: &Document, limits: &om9_core::budget::Limits) -> Result<()> {
        if document.id != self.source_document || document.revision != self.source_revision {
            return Err(Error::Invalid("jobbag archive provenance mismatch"));
        }
        // A projection may contain extra top-level objects only when they are
        // required by the selected roots' history or other dependencies.
        check_capture_roots(self.captured_ids.len(), limits)?;
        if self.captured_ids.is_empty() || document.entities.len() < self.captured_ids.len() {
            return Err(Error::Invalid("jobbag archive root set mismatch"));
        }
        let unique: std::collections::BTreeSet<_> = self.captured_ids.iter().copied().collect();
        if unique.len() != self.captured_ids.len() {
            return Err(Error::Invalid("jobbag archive root set mismatch"));
        }
        let mut visited = 0;
        for (index, id) in self.captured_ids.iter().enumerate() {
            let entity = document.entities.get(id).ok_or(Error::Missing("jobbag archive root"))?;
            if let Some(class) = self.root_classes.get(index) {
                if *class != BagObjectClass::of(entity) && *class != BagObjectClass::resolved(entity, document, limits, &mut visited)? {
                    return Err(Error::Invalid("jobbag archive root mismatch"));
                }
            }
            if self.entities.get(index).is_some_and(|old| old != entity) {
                return Err(Error::Invalid("jobbag archive root mismatch"));
            }
        }
        let closure = crate::session::jobbag_selection_closure(document, &self.captured_ids, limits)?;
        if closure.len() != document.entities.len() || document.entities.keys().any(|id| closure.binary_search(id).is_err()) {
            return Err(Error::Invalid("jobbag archive root set mismatch"));
        }
        Ok(())
    }
}

fn check_capture_roots(count: usize, limits: &om9_core::budget::Limits) -> Result<()> {
    use om9_core::budget::Usage;
    let mut usage = Usage { objects: u64::try_from(count).map_err(|_| Error::Overflow("jobbag roots"))?, ..Usage::default() };
    usage.add(Usage::array::<ObjectId>(count)?, limits)?;
    usage.add(Usage::array::<BagObjectClass>(count)?, limits)?;
    usage.add(Usage::btree::<ObjectId, ()>(count)?, limits)?;
    // Persisted state must remain loadable with the standard manager limits.
    usage.check(&om9_core::budget::Limits::default())?;
    // IDs and classes also need to fit the independently bounded JSON manifest.
    if count.checked_mul(96).is_none_or(|bytes| bytes > 16 * 1024 * 1024) {
        return Err(Error::Budget("jobbag manifest roots"));
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JobbagSlot {
    pub index: usize,
    pub snapshot: Option<Snapshot>,
}

impl JobbagSlot {
    pub fn empty(index: usize) -> Self {
        Self {
            index,
            snapshot: None,
        }
    }
    pub fn is_empty(&self) -> bool {
        self.snapshot.is_none()
    }
    pub fn thumbnail(&self, document: &Document) -> Option<&Thumbnail> {
        self.snapshot
            .as_ref()
            .filter(|snapshot| snapshot.available(document))
            .map(|snapshot| &snapshot.thumbnail)
    }
}

/// Full-design Master snapshot. It is a deep clone so a later edit cannot
/// rewrite the captured scene.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MasterSnapshot {
    pub id: SnapshotId,
    /// Legacy inline snapshots keep a document. New versions use the archive
    /// as their sole document payload, avoiding a second serialized copy.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub document: Option<Arc<Document>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object_count: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layer_count: Option<usize>,
    /// Native container with hydrated resource chunks; absent in legacy state.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archive: Option<Arc<[u8]>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    archive_asset: Option<ProjectAssetRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumbnail_png: Option<Arc<[u8]>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    thumbnail_asset: Option<ProjectAssetRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectAssetRef { sha256: [u8; 32], length: u64 }

pub(crate) type ProjectAssets = std::collections::BTreeMap<[u8; 32], Arc<[u8]>>;

fn externalize_asset(payload: &mut Option<Arc<[u8]>>, reference: &mut Option<ProjectAssetRef>, assets: &mut ProjectAssets,
    check: &impl Fn() -> Result<()>) -> Result<()> {
    check()?;
    if reference.is_some() { return Err(Error::Invalid("unhydrated project asset")); }
    if let Some(bytes) = payload.take() {
        let mut digest = Sha256::new();
        for chunk in bytes.chunks(1024 * 1024) { check()?; digest.update(chunk); }
        let sha256: [u8; 32] = digest.finalize().into();
        let length = u64::try_from(bytes.len()).map_err(|_| Error::Overflow("project asset length"))?;
        if let Some(existing) = assets.get(&sha256) {
            if existing.len() != bytes.len() { return Err(Error::Invalid("project asset hash collision")); }
            for (a, b) in existing.chunks(1024 * 1024).zip(bytes.chunks(1024 * 1024)) {
                check()?;
                if a != b { return Err(Error::Invalid("project asset hash collision")); }
            }
        } else { assets.insert(sha256, bytes); }
        *reference = Some(ProjectAssetRef { sha256, length });
    }
    Ok(())
}

fn hydrate_asset(payload: &mut Option<Arc<[u8]>>, reference: &mut Option<ProjectAssetRef>, assets: &ProjectAssets,
    check: &mut impl FnMut() -> Result<()>) -> Result<()> {
    check()?;
    if let Some(asset) = reference.take() {
        if payload.is_some() { return Err(Error::Invalid("duplicate project asset")); }
        let bytes = assets.get(&asset.sha256).ok_or(Error::Missing("project asset"))?;
        if bytes.len() as u64 != asset.length { return Err(Error::Invalid("project asset integrity")); }
        let mut digest = Sha256::new();
        for chunk in bytes.chunks(1024 * 1024) { check()?; digest.update(chunk); }
        if <[u8; 32]>::from(digest.finalize()) != asset.sha256 {
            return Err(Error::Invalid("project asset integrity"));
        }
        *payload = Some(bytes.clone());
    }
    Ok(())
}

/// User-entered catalog data. Numeric values are not calculated from a mesh.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ProjectMetadata {
    pub customer_id: Option<u64>,
    pub designer_id: Option<u64>,
    pub status_id: Option<u64>,
    pub designer: String,
    pub status: String,
    pub tags: Vec<String>,
    /// ISO calendar dates (YYYY-MM-DD), when supplied.
    pub created_date: Option<String>,
    pub due_date: Option<String>,
    pub job_number: String,
    pub cad_labor_hours: Option<f64>,
    pub cam_labor_hours: Option<f64>,
    pub ring_size: String,
    pub ring_rail_radius_mm: Option<f64>,
    pub metal_weight_grams: Option<f64>,
    pub gem_weight_carats: Option<f64>,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectoryKind { Customer, Designer, Status }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DirectoryEntry { pub id: u64, pub name: String }

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ProjectDirectory {
    pub customers: Vec<DirectoryEntry>,
    pub designers: Vec<DirectoryEntry>,
    pub statuses: Vec<DirectoryEntry>,
}
impl ProjectDirectory {
    fn entries(&self, kind: DirectoryKind) -> &[DirectoryEntry] {
        match kind { DirectoryKind::Customer => &self.customers, DirectoryKind::Designer => &self.designers, DirectoryKind::Status => &self.statuses }
    }
    fn entries_mut(&mut self, kind: DirectoryKind) -> &mut Vec<DirectoryEntry> {
        match kind { DirectoryKind::Customer => &mut self.customers, DirectoryKind::Designer => &mut self.designers, DirectoryKind::Status => &mut self.statuses }
    }
}

impl ProjectMetadata {
    fn validate(&self) -> Result<()> {
        for text in [&self.designer, &self.status, &self.job_number, &self.ring_size] {
            validate_optional_text(text, MAX_CUSTOMER, "project metadata")?;
        }
        validate_optional_text(&self.description, MAX_NOTES, "project description")?;
        if self.tags.len() > 64 { return Err(Error::Budget("project tags")); }
        for tag in &self.tags { validate_text(tag, MAX_NAME, "project tag")?; }
        for value in [self.cad_labor_hours, self.cam_labor_hours, self.ring_rail_radius_mm, self.metal_weight_grams, self.gem_weight_carats].into_iter().flatten() {
            if !value.is_finite() || value < 0.0 { return Err(Error::Invalid("project measurement")); }
        }
        for date in [&self.created_date, &self.due_date].into_iter().flatten() {
            if !valid_calendar_date(date) { return Err(Error::Invalid("project date")); }
        }
        Ok(())
    }
}

pub(crate) fn valid_calendar_date(date: &str) -> bool {
    let bytes = date.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-'
        || bytes.iter().enumerate().any(|(i, b)| i != 4 && i != 7 && !b.is_ascii_digit()) { return false; }
    let number = |range: std::ops::Range<usize>| -> u32 {
        bytes[range].iter().fold(0, |n, b| n * 10 + u32::from(*b - b'0'))
    };
    let (year, month, day) = (number(0..4), number(5..7), number(8..10));
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days = match month { 1 | 3 | 5 | 7 | 8 | 10 | 12 => 31, 4 | 6 | 9 | 11 => 30, 2 if leap => 29, 2 => 28, _ => 0 };
    year != 0 && day >= 1 && day <= days
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectRow {
    pub id: ProjectId,
    pub name: String,
    pub notes: String,
    /// Customer/job label; free text, never used as a path.
    pub customer: String,
    #[serde(default)]
    pub metadata: ProjectMetadata,
    pub created_revision: u64,
    #[serde(default = "default_visible")]
    pub visible: bool,
    #[serde(default)]
    pub elapsed_seconds: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timer_started_unix: Option<u64>,
    pub slots: Vec<JobbagSlot>,
    pub master: Option<MasterSnapshot>,
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub named_masters: std::collections::BTreeMap<String, MasterSnapshot>,
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub named_versions: std::collections::BTreeMap<String, Vec<MasterSnapshot>>,
}

impl ProjectRow {
    pub fn new(id: ProjectId, name: impl Into<String>, revision: u64) -> Self {
        Self {
            id,
            name: name.into(),
            notes: String::new(),
            customer: String::new(),
            metadata: ProjectMetadata::default(),
            created_revision: revision,
            visible: true,
            elapsed_seconds: 0,
            timer_started_unix: None,
            slots: (0..SLOT_COUNT).map(JobbagSlot::empty).collect(),
            master: None,
            named_masters: Default::default(),
            named_versions: Default::default(),
        }
    }
    pub fn slot(&self, index: usize) -> Option<&JobbagSlot> {
        self.slots.get(index)
    }
    pub fn has_legacy_bags(&self) -> bool {
        self.slots.iter().filter_map(|slot| slot.snapshot.as_ref()).any(|snapshot| snapshot.archive.is_none())
            || self.master.iter().chain(self.named_masters.values()).chain(self.named_versions.values().flatten())
                .any(|snapshot| snapshot.archive.is_none())
    }
    pub fn elapsed_at(&self, now: u64) -> u64 {
        self.elapsed_seconds.saturating_add(self.timer_started_unix.map_or(0, |start| now.saturating_sub(start)))
    }
    pub fn thumbnail(&self, index: usize, document: &Document) -> Option<&Thumbnail> {
        self.slot(index).and_then(|slot| slot.thumbnail(document))
    }
}

fn default_visible() -> bool { true }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectManager {
    #[serde(default = "legacy_project_schema")]
    schema_version: u32,
    rows: Vec<ProjectRow>,
    active: usize,
    next_project: u64,
    next_snapshot: u64,
    #[serde(default)]
    directory: ProjectDirectory,
    #[serde(default = "initial_directory_id")]
    next_directory: u64,
    #[serde(default)]
    metadata_changed: bool,
    #[serde(skip)]
    change_revision: u64,
}

fn initial_directory_id() -> u64 { 1 }
fn legacy_project_schema() -> u32 { 1 }
const PROJECT_SCHEMA: u32 = 2;

impl ProjectManager {
    /// Encode a manifest plus deduplicated immutable blobs. The live manager is
    /// untouched; the caller publishes the manifest and blobs atomically.
    pub(crate) fn externalized(&self) -> Result<(Vec<u8>, ProjectAssets)> {
        self.externalized_with_check(&|| Ok(()))
    }

    fn externalized_with_check(&self, check: &impl Fn() -> Result<()>) -> Result<(Vec<u8>, ProjectAssets)> {
        check()?;
        let mut manager = self.clone();
        let mut assets = ProjectAssets::new();
        for row in &mut manager.rows {
            for snapshot in row.slots.iter_mut().filter_map(|slot| slot.snapshot.as_mut()) {
                externalize_asset(&mut snapshot.archive, &mut snapshot.archive_asset, &mut assets, check)?;
                externalize_asset(&mut snapshot.thumbnail.png, &mut snapshot.thumbnail.png_asset, &mut assets, check)?;
            }
            for snapshot in row.master.iter_mut().chain(row.named_masters.values_mut()).chain(row.named_versions.values_mut().flatten()) {
                externalize_asset(&mut snapshot.archive, &mut snapshot.archive_asset, &mut assets, check)?;
                externalize_asset(&mut snapshot.thumbnail_png, &mut snapshot.thumbnail_asset, &mut assets, check)?;
            }
        }
        check()?;
        let manifest = manager.to_json()?;
        check()?;
        Ok((manifest, assets))
    }

    pub(crate) fn hydrate_assets(self, assets: &ProjectAssets) -> Result<Self> {
        self.hydrate_assets_with_check(assets, &mut || Ok(()))
    }

    pub(crate) fn hydrate_assets_with_check(mut self, assets: &ProjectAssets, check: &mut impl FnMut() -> Result<()>) -> Result<Self> {
        for row in &mut self.rows {
            for snapshot in row.slots.iter_mut().filter_map(|slot| slot.snapshot.as_mut()) {
                hydrate_asset(&mut snapshot.archive, &mut snapshot.archive_asset, assets, check)?;
                hydrate_asset(&mut snapshot.thumbnail.png, &mut snapshot.thumbnail.png_asset, assets, check)?;
            }
            for snapshot in row.master.iter_mut().chain(row.named_masters.values_mut()).chain(row.named_versions.values_mut().flatten()) {
                hydrate_asset(&mut snapshot.archive, &mut snapshot.archive_asset, assets, check)?;
                hydrate_asset(&mut snapshot.thumbnail_png, &mut snapshot.thumbnail_asset, assets, check)?;
            }
        }
        check()?;
        Ok(self)
    }

    pub(crate) fn required_assets(&self) -> Result<std::collections::BTreeMap<[u8; 32], u64>> {
        let mut needed = std::collections::BTreeMap::new();
        let mut add = |reference: &Option<ProjectAssetRef>| -> Result<()> {
            if let Some(asset) = reference {
                if asset.length == 0 { return Err(Error::Invalid("empty project asset")); }
                if needed.insert(asset.sha256, asset.length).is_some_and(|length| length != asset.length) {
                    return Err(Error::Invalid("project asset length conflict"));
                }
            }
            Ok(())
        };
        for row in &self.rows {
            for snapshot in row.slots.iter().filter_map(|slot| slot.snapshot.as_ref()) {
                add(&snapshot.archive_asset)?; add(&snapshot.thumbnail.png_asset)?;
            }
            for snapshot in row.master.iter().chain(row.named_masters.values()).chain(row.named_versions.values().flatten()) {
                add(&snapshot.archive_asset)?; add(&snapshot.thumbnail_asset)?;
            }
        }
        Ok(needed)
    }

    /// Decode archived snapshots through the caller's resource and admission
    /// owner, and verify that their summaries describe the decoded document.
    pub(crate) fn validate_snapshot_archives<R>(
        &self,
        limits: &om9_core::budget::Limits,
        mut decode: impl FnMut(Option<&[u8]>, Option<([u8; 32], u64)>) -> Result<(R, Document)>,
    ) -> Result<()> {
        for row in &self.rows {
            for snapshot in row.slots.iter().filter_map(|slot| slot.snapshot.as_ref()) {
                if snapshot.archive.is_none() && snapshot.archive_asset.is_none() { continue; }
                let reference = snapshot.archive_asset.as_ref().map(|asset| (asset.sha256, asset.length));
                let (_reservation, document) = decode(snapshot.archive.as_deref(), reference)?;
                snapshot.validate_archive_roots(&document, limits)?;
            }
            for snapshot in row.master.iter().chain(row.named_masters.values()).chain(row.named_versions.values().flatten()) {
                if snapshot.archive.is_none() && snapshot.archive_asset.is_none() { continue; }
                let reference = snapshot.archive_asset.as_ref().map(|asset| (asset.sha256, asset.length));
                let (_reservation, document) = decode(snapshot.archive.as_deref(), reference)?;
                if snapshot.object_count.is_some_and(|count| count != document.entities.len())
                    || snapshot.layer_count.is_some_and(|count| count != document.layer_order.len()) {
                    return Err(Error::Invalid("named jobbag archive summary"));
                }
            }
        }
        Ok(())
    }

    fn validate_storage_budget(&self, limits: &om9_core::budget::Limits) -> Result<()> {
        let mut owners = std::collections::BTreeSet::new();
        let mut size = 0u64;
        let mut add = |payload: &Option<Arc<[u8]>>, reference: &Option<ProjectAssetRef>| -> Result<()> {
            if reference.is_some() { return Err(Error::Invalid("unhydrated project asset")); }
            if let Some(bytes) = payload {
                // Count immutable allocations once; hashing belongs to the catalog worker.
                if owners.insert(Arc::as_ptr(bytes) as *const u8 as usize) {
                    size = size.checked_add(bytes.len() as u64).ok_or(Error::Overflow("project asset bytes"))?;
                    if size > limits.max_bytes { return Err(Error::Budget("project asset bytes")); }
                }
            }
            Ok(())
        };
        for row in &self.rows {
            for snapshot in row.slots.iter().filter_map(|slot| slot.snapshot.as_ref()) {
                add(&snapshot.archive, &snapshot.archive_asset)?;
                add(&snapshot.thumbnail.png, &snapshot.thumbnail.png_asset)?;
            }
            for snapshot in row.master.iter().chain(row.named_masters.values()).chain(row.named_versions.values().flatten()) {
                add(&snapshot.archive, &snapshot.archive_asset)?;
                add(&snapshot.thumbnail_png, &snapshot.thumbnail_asset)?;
            }
        }
        Ok(())
    }

    pub fn new(name: impl Into<String>) -> Self {
        let row = ProjectRow::new(ProjectId(1), name, 1);
        Self {
            schema_version: PROJECT_SCHEMA,
            rows: vec![row],
            active: 0,
            next_project: 2,
            next_snapshot: 1,
            directory: ProjectDirectory::default(),
            next_directory: 1,
            metadata_changed: false,
            change_revision: 0,
        }
    }

    /// True when there is user-created project state worth embedding. A fresh
    /// default row with empty slots does not dirty a plain document.
    pub fn has_persistent_state(&self) -> bool {
        self.metadata_changed || self.rows.len() > 1 || self.directory != ProjectDirectory::default()
            || self.rows.iter().any(|row| {
                !row.notes.is_empty()
                    || !row.customer.is_empty()
                    || row.metadata != ProjectMetadata::default()
                    || row.master.is_some()
                    || !row.named_masters.is_empty()
                    || !row.named_versions.is_empty()
                    || row.slots.iter().any(|slot| slot.snapshot.is_some())
            })
    }

    pub fn rows(&self) -> &[ProjectRow] {
        &self.rows
    }

    pub fn directory_entries(&self, kind: DirectoryKind) -> &[DirectoryEntry] { self.directory.entries(kind) }

    pub fn create_directory_entry(&mut self, kind: DirectoryKind, name: &str) -> Result<u64> {
        let name = validate_text(name, MAX_CUSTOMER, "directory name")?;
        if self.directory.entries(kind).len() >= 256 { return Err(Error::Budget("project directory")); }
        let id = self.next_directory;
        self.next_directory = id.checked_add(1).ok_or(Error::Overflow("directory id"))?;
        self.directory.entries_mut(kind).push(DirectoryEntry { id, name });
        self.metadata_changed = true;
        self.changed();
        Ok(id)
    }

    pub fn rename_directory_entry(&mut self, kind: DirectoryKind, id: u64, name: &str) -> Result<()> {
        let name = validate_text(name, MAX_CUSTOMER, "directory name")?;
        let entry = self.directory.entries_mut(kind).iter_mut().find(|entry| entry.id == id).ok_or(Error::Missing("directory entry"))?;
        if entry.name == name { return Ok(()); }
        entry.name = name.clone();
        for row in &mut self.rows {
            match kind {
                DirectoryKind::Customer if row.metadata.customer_id == Some(id) => row.customer = name.clone(),
                DirectoryKind::Designer if row.metadata.designer_id == Some(id) => row.metadata.designer = name.clone(),
                DirectoryKind::Status if row.metadata.status_id == Some(id) => row.metadata.status = name.clone(),
                _ => {}
            }
        }
        self.metadata_changed = true;
        self.changed();
        Ok(())
    }

    pub fn remove_directory_entry(&mut self, kind: DirectoryKind, id: u64) -> Result<()> {
        let entries = self.directory.entries_mut(kind);
        let index = entries.iter().position(|entry| entry.id == id).ok_or(Error::Missing("directory entry"))?;
        entries.remove(index);
        for row in &mut self.rows {
            let binding = match kind { DirectoryKind::Customer => &mut row.metadata.customer_id, DirectoryKind::Designer => &mut row.metadata.designer_id, DirectoryKind::Status => &mut row.metadata.status_id };
            if *binding == Some(id) { *binding = None; }
        }
        self.metadata_changed = true;
        self.changed();
        Ok(())
    }

    pub fn change_revision(&self) -> u64 { self.change_revision }

    /// Conservative input for asynchronous catalog serialization admission.
    /// Native/GPU allocations and allocator overhead remain separate.
    pub fn storage_bytes_estimate(&self, limits: &om9_core::budget::Limits) -> Result<u64> {
        use om9_core::budget::Usage;
        let mut total = 0u64;
        let mut add = |bytes: u64| -> Result<()> {
            total = total.checked_add(bytes).ok_or(Error::Overflow("project manager storage"))?;
            Ok(())
        };
        add(Usage::array::<Self>(1)?.bytes)?;
        add(Usage::array::<ProjectRow>(self.rows.capacity())?.bytes)?;
        for entries in [&self.directory.customers, &self.directory.designers, &self.directory.statuses] {
            add(Usage::array::<DirectoryEntry>(entries.capacity())?.bytes)?;
            for entry in entries { add(entry.name.capacity() as u64)?; }
        }
        for row in &self.rows {
            add((row.name.capacity() + row.customer.capacity() + row.notes.capacity()) as u64)?;
            for text in [&row.metadata.designer, &row.metadata.status, &row.metadata.job_number,
                &row.metadata.ring_size, &row.metadata.description] { add(text.capacity() as u64)?; }
            for date in [&row.metadata.created_date, &row.metadata.due_date].into_iter().flatten() {
                add(date.capacity() as u64)?;
            }
            add(Usage::array::<String>(row.metadata.tags.capacity())?.bytes)?;
            for tag in &row.metadata.tags { add(tag.capacity() as u64)?; }
            add(Usage::array::<JobbagSlot>(row.slots.capacity())?.bytes)?;
            add(Usage::btree::<String, MasterSnapshot>(row.named_masters.len())?.bytes)?;
            add(Usage::btree::<String, Vec<MasterSnapshot>>(row.named_versions.len())?.bytes)?;
            for key in row.named_masters.keys().chain(row.named_versions.keys()) { add(key.capacity() as u64)?; }
            for versions in row.named_versions.values() { add(Usage::array::<MasterSnapshot>(versions.capacity())?.bytes)?; }
            for snapshot in row.slots.iter().filter_map(|slot| slot.snapshot.as_ref()) {
                add(Usage::array::<ObjectId>(snapshot.captured_ids.capacity())?.bytes)?;
                add(Usage::array::<BagObjectClass>(snapshot.root_classes.capacity())?.bytes)?;
                add(Usage::array::<Entity>(snapshot.entities.len())?.bytes)?;
                add((snapshot.thumbnail.label.capacity() + snapshot.thumbnail.kind.capacity()) as u64)?;
                add(snapshot.archive.as_ref().map_or(0, |bytes| bytes.len()) as u64)?;
                add(snapshot.thumbnail.png.as_ref().map_or(0, |bytes| bytes.len()) as u64)?;
                for entity in snapshot.entities.iter() {
                    add(entity.geometry.validate(limits)?.bytes)?;
                    add(entity.label.capacity() as u64)?;
                    add(Usage::btree::<String, om9_core::model::Extension>(entity.extensions.len())?.bytes)?;
                    for (name, extension) in &entity.extensions { add((name.capacity() + extension.payload.capacity()) as u64)?; }
                }
            }
            for snapshot in row.master.iter().chain(row.named_masters.values()).chain(row.named_versions.values().flatten()) {
                add(snapshot.archive.as_ref().map_or(0, |bytes| bytes.len()) as u64)?;
                add(snapshot.thumbnail_png.as_ref().map_or(0, |bytes| bytes.len()) as u64)?;
                if let Some(document) = &snapshot.document { add(document.clone_owned_bytes_estimate(limits)?)?; }
            }
        }
        Ok(total)
    }

    fn changed(&mut self) { self.change_revision = self.change_revision.wrapping_add(1); }

    pub fn active(&self) -> &ProjectRow {
        &self.rows[self.active]
    }

    pub fn active_mut(&mut self) -> &mut ProjectRow {
        &mut self.rows[self.active]
    }

    pub fn active_index(&self) -> usize {
        self.active
    }

    pub fn select(&mut self, id: ProjectId) -> Result<()> {
        let index = self
            .rows
            .iter()
            .position(|row| row.id == id)
            .ok_or(Error::Missing("project row"))?;
        if self.active != index {
            self.active = index;
            self.changed();
        }
        Ok(())
    }

    pub fn create_project(&mut self, name: &str, revision: u64) -> Result<ProjectId> {
        let name = validate_text(name, MAX_NAME, "project name")?;
        if self.rows.len() >= MAX_PROJECTS {
            return Err(Error::Budget("project rows"));
        }
        let id = ProjectId(self.next_project);
        self.next_project = self
            .next_project
            .checked_add(1)
            .ok_or(Error::Overflow("project id"))?;
        self.rows.push(ProjectRow::new(id, name, revision));
        self.active = self.rows.len() - 1;
        self.changed();
        Ok(id)
    }

    pub fn rename_project(&mut self, id: ProjectId, name: &str) -> Result<()> {
        let name = validate_text(name, MAX_NAME, "project name")?;
        let row = self
            .rows
            .iter_mut()
            .find(|row| row.id == id)
            .ok_or(Error::Missing("project row"))?;
        if row.name != name {
            row.name = name;
            self.metadata_changed = true;
            self.changed();
        }
        Ok(())
    }

    pub fn delete_project(&mut self, id: ProjectId) -> Result<()> {
        if self.rows.len() <= 1 { return Err(Error::Invalid("cannot delete last project")); }
        let index = self.rows.iter().position(|row| row.id == id).ok_or(Error::Missing("project row"))?;
        self.rows.remove(index);
        if self.active > index { self.active -= 1; }
        else if self.active == index { self.active = index.min(self.rows.len() - 1); }
        self.metadata_changed = true;
        self.changed();
        Ok(())
    }

    /// Export exactly one row with native snapshot archives and stable local
    /// identities. A legacy incomplete bag is refused rather than lost.
    pub fn export_project(&self, id: ProjectId) -> Result<Vec<u8>> {
        self.single_project(id)?.to_json()
    }

    pub(crate) fn export_project_bundle_with_check(&self, id: ProjectId, check: &impl Fn() -> Result<()>) -> Result<(Vec<u8>, ProjectAssets)> {
        check()?;
        self.single_project(id)?.externalized_with_check(check)
    }

    fn single_project(&self, id: ProjectId) -> Result<Self> {
        self.single_project_with_legacy(id, false)
    }

    fn single_project_with_legacy(&self, id: ProjectId, allow_legacy: bool) -> Result<Self> {
        let row = self.rows.iter().find(|row| row.id == id).ok_or(Error::Missing("project row"))?;
        if !allow_legacy && row.has_legacy_bags() {
            return Err(Error::Unsupported("legacy project bags need native archives before export"));
        }
        let single = Self {
            schema_version: PROJECT_SCHEMA, rows: vec![row.clone()], active: 0, next_project: self.next_project,
            next_snapshot: self.next_snapshot, directory: self.directory.clone(), next_directory: self.next_directory,
            metadata_changed: true, change_revision: 0,
        };
        Ok(single)
    }

    /// Import as a new row and remap local row/snapshot identifiers before
    /// publication. Existing rows are never overwritten by an imported name.
    pub fn import_project(&mut self, bytes: &[u8], limits: &om9_core::budget::Limits) -> Result<ProjectId> {
        self.import_project_bundle(bytes, &ProjectAssets::new(), limits)
    }

    pub(crate) fn import_project_bundle(&mut self, bytes: &[u8], assets: &ProjectAssets, limits: &om9_core::budget::Limits) -> Result<ProjectId> {
        self.import_project_bundle_with_cancel(bytes, assets, limits, &om9_core::transaction::Cancellation::default())
    }

    pub(crate) fn import_project_bundle_with_cancel(&mut self, bytes: &[u8], assets: &ProjectAssets, limits: &om9_core::budget::Limits, cancel: &om9_core::transaction::Cancellation) -> Result<ProjectId> {
        self.import_project_bundle_with_policy(bytes, assets, limits, cancel, false)
    }

    fn import_project_bundle_with_policy(&mut self, bytes: &[u8], assets: &ProjectAssets, limits: &om9_core::budget::Limits, cancel: &om9_core::transaction::Cancellation, allow_legacy: bool) -> Result<ProjectId> {
        self.import_project_bundle_with_validator(bytes, assets, limits, cancel, allow_legacy, &mut |archive| {
            om9_archive::Archive::decode_with_admission(archive, limits, |_| {
                if cancel.is_canceled() { Err(Error::Canceled) } else { Ok(()) }
            }).map(|document| ((), document))
        })
    }

    pub(crate) fn import_project_bundle_with_validator<R>(&mut self, bytes: &[u8], assets: &ProjectAssets,
        limits: &om9_core::budget::Limits, cancel: &om9_core::transaction::Cancellation, allow_legacy: bool,
        validate: &mut impl FnMut(&[u8]) -> Result<(R, om9_core::model::Document)>) -> Result<ProjectId> {
        if cancel.is_canceled() { return Err(Error::Canceled); }
        let manifest = Self::from_json(bytes, "import")?;
        let needed = manifest.required_assets()?;
        if needed.len() != assets.len() || needed.iter().any(|(hash, length)| assets.get(hash).is_none_or(|bytes| bytes.len() as u64 != *length)) {
            return Err(Error::Invalid("project archive assets"));
        }
        let mut incoming = manifest.hydrate_assets_with_check(assets, &mut || {
            if cancel.is_canceled() { Err(Error::Canceled) } else { Ok(()) }
        })?;
        if incoming.rows.len() != 1 { return Err(Error::Invalid("project archive row count")); }
        if self.rows.len() >= MAX_PROJECTS { return Err(Error::Budget("project rows")); }
        let mut row = incoming.rows.remove(0);
        for snapshot in row.slots.iter().filter_map(|slot| slot.snapshot.as_ref()) {
            let Some(archive) = snapshot.archive.as_ref() else {
                if allow_legacy { continue; }
                return Err(Error::Unsupported("legacy project bag archive"));
            };
            let (_owner, document) = validate(archive)?;
            snapshot.validate_archive_roots(&document, limits)?;
        }
        for snapshot in row.master.iter().chain(row.named_masters.values()).chain(row.named_versions.values().flatten()) {
            let Some(archive) = snapshot.archive.as_ref() else {
                if allow_legacy && snapshot.document.is_some() { continue; }
                return Err(Error::Unsupported("legacy project snapshot archive"));
            };
            let (_owner, document) = validate(archive)?;
            if snapshot.object_count.is_some_and(|count| count != document.entities.len())
                || snapshot.layer_count.is_some_and(|count| count != document.layer_order.len()) {
                return Err(Error::Invalid("named jobbag archive summary"));
            }
        }
        let mut candidate = self.clone();
        for (kind, binding) in [
            (DirectoryKind::Customer, &mut row.metadata.customer_id),
            (DirectoryKind::Designer, &mut row.metadata.designer_id),
            (DirectoryKind::Status, &mut row.metadata.status_id),
        ] {
            if let Some(old_id) = *binding {
                let source = incoming.directory.entries(kind).iter().find(|entry| entry.id == old_id).ok_or(Error::Missing("import directory entry"))?;
                let same = candidate.directory.entries(kind).iter().any(|entry| entry.id == old_id && entry.name == source.name);
                *binding = Some(if same { old_id } else { candidate.create_directory_entry(kind, &source.name)? });
            }
        }
        row.id = ProjectId(candidate.next_project);
        candidate.next_project = candidate.next_project.checked_add(1).ok_or(Error::Overflow("project id"))?;
        let mut assign = |id: &mut SnapshotId| -> Result<()> {
            *id = SnapshotId(candidate.next_snapshot);
            candidate.next_snapshot = candidate.next_snapshot.checked_add(1).ok_or(Error::Overflow("jobbag snapshot id"))?;
            Ok(())
        };
        for snapshot in row.slots.iter_mut().filter_map(|slot| slot.snapshot.as_mut()) { assign(&mut snapshot.id)?; }
        for snapshot in row.master.iter_mut().chain(row.named_masters.values_mut()).chain(row.named_versions.values_mut().flatten()) { assign(&mut snapshot.id)?; }
        let id = row.id;
        candidate.rows.push(row);
        candidate.active = candidate.rows.len() - 1;
        candidate.metadata_changed = true;
        candidate.changed();
        candidate.validate_storage_budget(limits)?;
        if cancel.is_canceled() { return Err(Error::Canceled); }
        *self = candidate;
        Ok(id)
    }

    /// Import embedded document rows as new workspace rows, preserving legacy
    /// inline bags while remapping workspace-local identities atomically.
    pub(crate) fn merge_embedded_with_cancel(&mut self, embedded: &Self, limits: &om9_core::budget::Limits, cancel: &om9_core::transaction::Cancellation) -> Result<()> {
        self.merge_embedded_with_validator(embedded, limits, cancel, &mut |archive| {
            om9_archive::Archive::decode_with_admission(archive, limits, |_| {
                if cancel.is_canceled() { Err(Error::Canceled) } else { Ok(()) }
            }).map(|document| ((), document))
        })
    }

    pub(crate) fn merge_embedded_with_validator<R>(&mut self, embedded: &Self, limits: &om9_core::budget::Limits,
        cancel: &om9_core::transaction::Cancellation,
        validate: &mut impl FnMut(&[u8]) -> Result<(R, om9_core::model::Document)>) -> Result<()> {
        let mut candidate = self.clone();
        for row in &embedded.rows {
            if cancel.is_canceled() { return Err(Error::Canceled); }
            let (manifest, assets) = embedded.single_project_with_legacy(row.id, true)?.externalized_with_check(&|| {
                if cancel.is_canceled() { Err(Error::Canceled) } else { Ok(()) }
            })?;
            candidate.import_project_bundle_with_validator(&manifest, &assets, limits, cancel, true, validate)?;
        }
        if cancel.is_canceled() { return Err(Error::Canceled); }
        *self = candidate;
        Ok(())
    }

    pub fn set_visible(&mut self, id: ProjectId, visible: bool) -> Result<()> {
        let row = self.rows.iter_mut().find(|row| row.id == id).ok_or(Error::Missing("project row"))?;
        if row.visible != visible {
            row.visible = visible;
            self.metadata_changed = true;
            self.changed();
        }
        Ok(())
    }

    pub fn timer_start(&mut self, id: ProjectId, now: u64) -> Result<()> {
        let row = self.rows.iter_mut().find(|row| row.id == id).ok_or(Error::Missing("project row"))?;
        if row.timer_started_unix.is_none() {
            row.timer_started_unix = Some(now);
            self.metadata_changed = true;
            self.changed();
        }
        Ok(())
    }

    pub fn timer_stop(&mut self, id: ProjectId, now: u64) -> Result<()> {
        let row = self.rows.iter_mut().find(|row| row.id == id).ok_or(Error::Missing("project row"))?;
        if let Some(start) = row.timer_started_unix {
            let elapsed = row.elapsed_seconds.checked_add(now.saturating_sub(start)).ok_or(Error::Overflow("project timer"))?;
            row.elapsed_seconds = elapsed;
            row.timer_started_unix = None;
            self.metadata_changed = true;
            self.changed();
        }
        Ok(())
    }

    pub fn timer_reset(&mut self, id: ProjectId, now: u64) -> Result<()> {
        let row = self.rows.iter_mut().find(|row| row.id == id).ok_or(Error::Missing("project row"))?;
        row.elapsed_seconds = 0;
        if row.timer_started_unix.is_some() { row.timer_started_unix = Some(now); }
        self.metadata_changed = true;
        self.changed();
        Ok(())
    }

    pub fn set_notes(&mut self, notes: &str) -> Result<()> {
        let notes = validate_optional_text(notes, MAX_NOTES, "project notes")?;
        if self.active().notes != notes {
            self.active_mut().notes = notes;
            self.changed();
        }
        Ok(())
    }

    pub fn set_customer(&mut self, customer: &str) -> Result<()> {
        let customer = validate_optional_text(customer, MAX_CUSTOMER, "customer label")?;
        if customer.contains(['/', '\\']) && (customer.starts_with('/') || customer.contains(":\\"))
        {
            return Err(Error::Invalid("customer label must not be a file path"));
        }
        if self.active().customer != customer {
            self.active_mut().customer = customer;
            self.active_mut().metadata.customer_id = None;
            self.changed();
        }
        Ok(())
    }

    /// Validate the complete draft before applying any field to a row.
    pub fn update_project(&mut self, id: ProjectId, name: &str, customer: &str, notes: &str, metadata: ProjectMetadata) -> Result<()> {
        let name = validate_text(name, MAX_NAME, "project name")?;
        let customer = validate_optional_text(customer, MAX_CUSTOMER, "customer label")?;
        if customer.contains(['/', '\\']) && (customer.starts_with('/') || customer.contains(":\\")) {
            return Err(Error::Invalid("customer label must not be a file path"));
        }
        let notes = validate_optional_text(notes, MAX_NOTES, "project notes")?;
        metadata.validate()?;
        self.validate_directory_link(DirectoryKind::Customer, metadata.customer_id, &customer)?;
        self.validate_directory_link(DirectoryKind::Designer, metadata.designer_id, &metadata.designer)?;
        self.validate_directory_link(DirectoryKind::Status, metadata.status_id, &metadata.status)?;
        let row = self.rows.iter_mut().find(|row| row.id == id).ok_or(Error::Missing("project row"))?;
        if row.name != name || row.customer != customer || row.notes != notes || row.metadata != metadata {
            row.name = name;
            row.customer = customer;
            row.notes = notes;
            row.metadata = metadata;
            self.metadata_changed = true;
            self.changed();
        }
        Ok(())
    }

    fn validate_directory_link(&self, kind: DirectoryKind, id: Option<u64>, name: &str) -> Result<()> {
        if let Some(id) = id {
            let entry = self.directory.entries(kind).iter().find(|entry| entry.id == id).ok_or(Error::Missing("directory entry"))?;
            if entry.name != name { return Err(Error::Stale("directory entry name changed")); }
        }
        Ok(())
    }

    pub fn capture(
        &mut self,
        slot: usize,
        selected: Vec<Entity>,
        document: &Document,
    ) -> Result<SnapshotId> {
        self.capture_with_limits(slot, selected, document, &om9_core::budget::Limits::default())
    }

    fn capture_with_limits(&mut self, slot: usize, selected: Vec<Entity>, document: &Document, limits: &om9_core::budget::Limits) -> Result<SnapshotId> {
        if slot >= self.active().slots.len() {
            return Err(Error::Invalid("jobbag slot index"));
        }
        if selected.is_empty() {
            return Err(Error::Invalid("jobbag capture selection"));
        }
        let id = SnapshotId(self.next_snapshot);
        let snapshot = Snapshot::new(id, document, selected, limits)?;
        if slot + 1 == self.active().slots.len() && self.active().slots.len() < MAX_SLOTS {
            self.active_mut().slots.try_reserve(1).map_err(|_| Error::Budget("jobbag slots"))?;
        }
        self.next_snapshot = self
            .next_snapshot
            .checked_add(1)
            .ok_or(Error::Overflow("jobbag snapshot id"))?;
        let row = self.active_mut();
        row.slots[slot].snapshot = Some(snapshot);
        if slot + 1 == row.slots.len() && row.slots.len() < MAX_SLOTS {
            row.slots.push(JobbagSlot::empty(slot + 1));
        }
        self.changed();
        Ok(id)
    }

    pub fn capture_selection(
        &mut self,
        slot: usize,
        selection: &[ObjectId],
        document: &Document,
    ) -> Result<SnapshotId> {
        if selection.is_empty() {
            return Err(Error::Invalid("jobbag capture selection"));
        }
        check_capture_roots(selection.len(), &om9_core::budget::Limits::default())?;
        let mut entities = Vec::new();
        entities.try_reserve_exact(selection.len()).map_err(|_| Error::Budget("jobbag selection"))?;
        for id in selection {
            let entity = document
                .entities
                .get(id)
                .ok_or(Error::Missing("selected jobbag object"))?
                .clone();
            entities.push(entity);
        }
        self.capture(slot, entities, document)
    }

    /// Install an already validated detached selection archive only after the
    /// capture succeeds, so failures leave the previous bag untouched.
    pub fn capture_archive(
        &mut self,
        slot: usize,
        selected: Vec<Entity>,
        document: &Document,
        archive: Vec<u8>,
    ) -> Result<SnapshotId> {
        self.capture_archive_with_limits(slot, selected, document, archive, &om9_core::budget::Limits::default())
    }

    pub fn capture_archive_with_limits(&mut self, slot: usize, selected: Vec<Entity>, document: &Document, archive: Vec<u8>, limits: &om9_core::budget::Limits) -> Result<SnapshotId> {
        if archive.is_empty() {
            return Err(Error::Invalid("empty jobbag archive"));
        }
        if slot >= self.active().slots.len() {
            return Err(Error::Invalid("jobbag slot index"));
        }
        let old_next = self.next_snapshot;
        let old_revision = self.change_revision;
        let old_len = self.active().slots.len();
        let old = self.active_mut().slots[slot].snapshot.take();
        let result = self.capture_with_limits(slot, selected, document, limits).and_then(|id| {
            let snapshot = self.active_mut().slots[slot].snapshot.as_mut().unwrap();
            let mut visited = 0;
            snapshot.root_classes = snapshot.entities.iter().map(|entity| BagObjectClass::resolved(entity, document, limits, &mut visited)).collect::<Result<Vec<_>>>()?;
            snapshot.entities = Arc::from([]);
            snapshot.archive = Some(archive.into());
            self.validate_storage_budget(limits)?;
            Ok(id)
        });
        if result.is_err() {
            self.active_mut().slots.truncate(old_len);
            self.active_mut().slots[slot].snapshot = old;
            self.next_snapshot = old_next;
            self.change_revision = old_revision;
        }
        result
    }

    /// Return fresh entities while preserving captured placement.
    pub fn restore_entities(&self, slot: usize) -> Result<Vec<Entity>> {
        if slot >= self.active().slots.len() {
            return Err(Error::Invalid("jobbag slot index"));
        }
        let snapshot = self
            .active()
            .slot(slot)
            .and_then(|slot| slot.snapshot.as_ref())
            .ok_or(Error::Missing("jobbag slot"))?;
        if snapshot.archive.is_some() || snapshot.archive_asset.is_some() {
            return Err(Error::Unsupported("jobbag archive requires transactional insertion"));
        }
        if snapshot.entities.iter().any(|entity| {
            entity.attributes.layer.is_some() || entity.attributes.material.is_some()
                || entity.attributes.line_pattern.is_some() || !entity.extensions.is_empty()
                || matches!(&entity.geometry, om9_core::geometry::Geometry::Instance(_))
        }) {
            return Err(Error::Unsupported("legacy jobbag dependencies were not captured"));
        }
        let mut restored = Vec::with_capacity(snapshot.entities.len());
        for source in snapshot.entities.iter() {
            let mut entity = source.clone();
            entity.id = ObjectId::new();
            restored.push(entity);
        }
        Ok(restored)
    }

    /// Insert a restored slot directly into an in-memory document. The UI path
    /// goes through the Controller transaction instead so Undo sees one action.
    pub fn insert_into(&self, slot: usize, document: &mut Document) -> Result<Vec<ObjectId>> {
        let entities = self.restore_entities(slot)?;
        let mut ids = Vec::with_capacity(entities.len());
        for entity in entities {
            ids.push(entity.id);
            document.entities.insert(entity.id, entity);
        }
        document.revision = document
            .revision
            .checked_add(1)
            .ok_or(Error::Overflow("document revision"))?;
        Ok(ids)
    }

    pub fn capture_master(&mut self, document: &Document) -> Result<SnapshotId> {
        self.capture_named_master("Master", document)
    }

    /// The five named bags all capture the whole document, including layers.
    pub fn capture_named_master(&mut self, name: &str, document: &Document) -> Result<SnapshotId> {
        if !MASTER_NAMES.contains(&name) {
            return Err(Error::Invalid("named master jobbag"));
        }
        let id = SnapshotId(self.next_snapshot);
        self.next_snapshot = self
            .next_snapshot
            .checked_add(1)
            .ok_or(Error::Overflow("master snapshot id"))?;
        let snapshot = MasterSnapshot {
            id,
            document: Some(Arc::new(document.clone())),
            object_count: None,
            layer_count: None,
            archive: None,
            archive_asset: None,
            thumbnail_png: None,
            thumbnail_asset: None,
        };
        if name == "Master" {
            self.active_mut().master = Some(snapshot);
        } else {
            self.active_mut().named_masters.insert(name.to_owned(), snapshot);
        }
        self.changed();
        Ok(id)
    }

    pub fn named_master_document(&self, name: &str) -> Option<&Document> {
        self.named_master_snapshot(name).and_then(|snapshot| snapshot.document.as_deref())
    }

    pub fn named_master_snapshot(&self, name: &str) -> Option<&MasterSnapshot> {
        self.active().named_versions.get(name).and_then(|versions| versions.last())
            .or_else(|| if name == "Master" { self.active().master.as_ref() }
                else { self.active().named_masters.get(name) })
    }

    pub fn named_master_versions(&self, name: &str) -> Vec<&MasterSnapshot> {
        let row = self.active();
        let legacy = if name == "Master" { row.master.as_ref() } else { row.named_masters.get(name) };
        legacy.into_iter().chain(row.named_versions.get(name).into_iter().flatten()).collect()
    }

    pub fn named_master_by_id(&self, name: &str, id: SnapshotId) -> Option<&MasterSnapshot> {
        self.named_master_versions(name).into_iter().find(|snapshot| snapshot.id == id)
    }

    pub fn clear_named_version(&mut self, name: &str, id: SnapshotId) -> Result<()> {
        let row = self.active_mut();
        let mut found = false;
        if name == "Master" && row.master.as_ref().is_some_and(|bag| bag.id == id) {
            row.master = None; found = true;
        } else if row.named_masters.get(name).is_some_and(|bag| bag.id == id) {
            row.named_masters.remove(name); found = true;
        } else if let Some(versions) = row.named_versions.get_mut(name) {
            if let Some(index) = versions.iter().position(|bag| bag.id == id) {
                versions.remove(index); found = true;
            }
            if versions.is_empty() { row.named_versions.remove(name); }
        }
        if !found { return Err(Error::Missing("named jobbag version")); }
        self.changed();
        Ok(())
    }

    /// Append a full-document version, rolling back if the embedded state is
    /// too large to save. The archive is prepared and validated by the codec.
    pub fn capture_named_version(&mut self, name: &str, document: Document, archive: Vec<u8>) -> Result<SnapshotId> {
        self.capture_named_version_summary(name, document.entities.len(), document.layer_order.len(), archive)
    }

    pub fn capture_named_version_summary(&mut self, name: &str, object_count: usize, layer_count: usize, archive: Vec<u8>) -> Result<SnapshotId> {
        self.capture_named_version_summary_with_limits(name, object_count, layer_count, archive, &om9_core::budget::Limits::default())
    }

    pub fn capture_named_version_summary_with_limits(&mut self, name: &str, object_count: usize, layer_count: usize, archive: Vec<u8>, limits: &om9_core::budget::Limits) -> Result<SnapshotId> {
        if !MASTER_NAMES.contains(&name) || archive.is_empty() {
            return Err(Error::Invalid("named jobbag archive"));
        }
        let id = SnapshotId(self.next_snapshot);
        let next = self.next_snapshot.checked_add(1).ok_or(Error::Overflow("master snapshot id"))?;
        let versions = self.active_mut().named_versions.entry(name.to_owned()).or_default();
        if versions.try_reserve(1).is_err() {
            if versions.is_empty() { self.active_mut().named_versions.remove(name); }
            return Err(Error::Budget("named jobbag versions"));
        }
        versions.push(MasterSnapshot {
            id, object_count: Some(object_count), layer_count: Some(layer_count),
            document: None, archive: Some(archive.into()), archive_asset: None, thumbnail_png: None, thumbnail_asset: None,
        });
        self.next_snapshot = next;
        if let Err(error) = self.validate_storage_budget(limits) {
            let versions = self.active_mut().named_versions.get_mut(name).unwrap();
            versions.pop();
            if versions.is_empty() { self.active_mut().named_versions.remove(name); }
            self.next_snapshot = id.0;
            return Err(error);
        }
        self.changed();
        Ok(id)
    }

    pub fn replace_named_version_summary(&mut self, name: &str, old: SnapshotId, object_count: usize, layer_count: usize, archive: Vec<u8>) -> Result<SnapshotId> {
        self.replace_named_version_summary_with_limits(name, old, object_count, layer_count, archive, &om9_core::budget::Limits::default())
    }

    pub fn replace_named_version_summary_with_limits(&mut self, name: &str, old: SnapshotId, object_count: usize, layer_count: usize, archive: Vec<u8>, limits: &om9_core::budget::Limits) -> Result<SnapshotId> {
        if !MASTER_NAMES.contains(&name) || archive.is_empty() { return Err(Error::Invalid("named jobbag archive")); }
        let mut candidate = self.clone();
        let id = SnapshotId(candidate.next_snapshot);
        candidate.next_snapshot = candidate.next_snapshot.checked_add(1).ok_or(Error::Overflow("master snapshot id"))?;
        let snapshot = MasterSnapshot { id, object_count: Some(object_count), layer_count: Some(layer_count), document: None,
            archive: Some(archive.into()), archive_asset: None, thumbnail_png: None, thumbnail_asset: None };
        let row = candidate.active_mut();
        if name == "Master" && row.master.as_ref().is_some_and(|bag| bag.id == old) {
            row.master = Some(snapshot);
        } else if row.named_masters.get(name).is_some_and(|bag| bag.id == old) {
            row.named_masters.insert(name.to_owned(), snapshot);
        } else {
            let version = row.named_versions.get_mut(name).and_then(|versions| versions.iter_mut().find(|bag| bag.id == old))
                .ok_or(Error::Stale("named jobbag version"))?;
            *version = snapshot;
        }
        candidate.changed();
        candidate.validate_storage_budget(limits)?;
        *self = candidate;
        Ok(id)
    }

    pub fn clear_named_master(&mut self, name: &str) {
        if let Some(versions) = self.active_mut().named_versions.get_mut(name) {
            versions.pop();
            if versions.is_empty() { self.active_mut().named_versions.remove(name); }
            self.changed();
            return;
        }
        if name == "Master" {
            self.active_mut().master = None;
        } else {
            self.active_mut().named_masters.remove(name);
        }
        self.changed();
    }

    /// Attach a derived thumbnail only to the still-current immutable snapshot.
    pub fn set_thumbnail(&mut self, project: ProjectId, snapshot: SnapshotId, png: Vec<u8>) -> Result<()> {
        self.set_thumbnail_with_limits(project, snapshot, png, &om9_core::budget::Limits::default())
    }

    pub fn set_thumbnail_with_limits(&mut self, project: ProjectId, snapshot: SnapshotId, png: Vec<u8>, limits: &om9_core::budget::Limits) -> Result<()> {
        if png.is_empty() || png.len() > 128 * 1024 { return Err(Error::Budget("thumbnail bytes")); }
        let png: Arc<[u8]> = png.into();
        let row = self.rows.iter_mut().find(|row| row.id == project).ok_or(Error::Stale("thumbnail project"))?;
        let mut attached = false;
        for bag in row.slots.iter_mut().filter_map(|slot| slot.snapshot.as_mut()) {
            if bag.id == snapshot {
                if bag.thumbnail.png.is_some() { return Err(Error::Stale("thumbnail already exists")); }
                bag.thumbnail.png = Some(png.clone()); attached = true; break;
            }
        }
        if !attached {
            for bag in row.master.iter_mut().chain(row.named_masters.values_mut()).chain(row.named_versions.values_mut().flatten()) {
                if bag.id == snapshot {
                    if bag.thumbnail_png.is_some() { return Err(Error::Stale("thumbnail already exists")); }
                    bag.thumbnail_png = Some(png); attached = true; break;
                }
            }
        }
        if !attached { return Err(Error::Stale("thumbnail snapshot")); }
        if let Err(error) = self.validate_storage_budget(limits) {
            let row = self.rows.iter_mut().find(|row| row.id == project).unwrap();
            for bag in row.slots.iter_mut().filter_map(|slot| slot.snapshot.as_mut()) {
                if bag.id == snapshot { bag.thumbnail.png = None; }
            }
            for bag in row.master.iter_mut().chain(row.named_masters.values_mut()).chain(row.named_versions.values_mut().flatten()) {
                if bag.id == snapshot { bag.thumbnail_png = None; }
            }
            return Err(error);
        }
        self.changed();
        Ok(())
    }

    pub fn snapshot_archive(&self, project: ProjectId, snapshot: SnapshotId) -> Option<&[u8]> {
        self.snapshot_archive_owner(project, snapshot).map(AsRef::as_ref)
    }

    pub(crate) fn snapshot_archive_owner(&self, project: ProjectId, snapshot: SnapshotId) -> Option<&Arc<[u8]>> {
        let row = self.rows.iter().find(|row| row.id == project)?;
        row.slots.iter().filter_map(|slot| slot.snapshot.as_ref()).find(|bag| bag.id == snapshot)
            .and_then(|bag| bag.archive.as_ref())
            .or_else(|| row.master.iter().chain(row.named_masters.values()).chain(row.named_versions.values().flatten())
                .find(|bag| bag.id == snapshot).and_then(|bag| bag.archive.as_ref()))
    }

    pub fn clear_slot(&mut self, index: usize) -> Result<()> {
        self.active_mut().slots.get_mut(index)
            .ok_or(Error::Invalid("jobbag slot"))?.snapshot = None;
        self.changed();
        Ok(())
    }

    /// Move occupied bags left while preserving snapshot identities and order.
    pub fn compact_slots(&mut self) {
        let row = self.active_mut();
        row.slots.retain(|slot| slot.snapshot.is_some());
        for (index, slot) in row.slots.iter_mut().enumerate() {
            slot.index = index;
        }
        let target = SLOT_COUNT.max((row.slots.len() + 1).min(MAX_SLOTS));
        while row.slots.len() < target {
            row.slots.push(JobbagSlot::empty(row.slots.len()));
        }
        self.changed();
    }

    pub fn master_document(&self) -> Option<&Document> {
        self.named_master_snapshot("Master").and_then(|master| master.document.as_deref())
    }

    /// Layer names captured with the master snapshot, exposed for the UI/tests.
    pub fn master_layer_names(&self) -> Vec<String> {
        self.master_document()
            .map(|document| {
                document
                    .layer_order
                    .iter()
                    .filter_map(|id| document.layers.get(id))
                    .map(|layer| layer.name.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn slot_available(&self, slot: usize, document: &Document) -> bool {
        self.active()
            .slot(slot)
            .and_then(|slot| slot.snapshot.as_ref())
            .is_some_and(|snapshot| snapshot.available(document))
    }
}

fn validate_text(value: &str, max: usize, what: &'static str) -> Result<String> {
    let value = validate_optional_text(value, max, what)?;
    if value.is_empty() {
        return Err(Error::Invalid(what));
    }
    Ok(value)
}

fn validate_optional_text(value: &str, max: usize, what: &'static str) -> Result<String> {
    let value = value.trim();
    if value.len() > max {
        return Err(Error::Budget(what));
    }
    if value.contains('\0') {
        return Err(Error::Invalid(what));
    }
    Ok(value.to_owned())
}
impl ProjectManager {
    /// Serialize the whole project manager for embedding in the `.om9` project
    /// resource. The caller supplies the owning document name for validation.
    pub fn to_json(&self) -> Result<Vec<u8>> {
        if self.rows.is_empty() || self.active >= self.rows.len() {
            return Err(Error::Invalid("project manager state"));
        }
        let mut output = ManagerJsonWriter { bytes: Vec::new(), refused: false };
        if let Err(error) = serde_json::to_writer(&mut output, self) {
            return if output.refused { Err(Error::Budget("project manager state bytes")) }
                else { Err(Error::Io(format!("project manager serialization: {error}"))) };
        }
        Ok(output.bytes)
    }

    pub fn from_json(bytes: &[u8], document_name: &str) -> Result<Self> {
        if bytes.is_empty() {
            return Err(Error::Invalid("project manager state bytes"));
        }
        if bytes.len() > MAX_MANAGER_JSON_BYTES {
            return Err(Error::Budget("project manager state bytes"));
        }
        // Typed map deserializers can otherwise silently replace a duplicate
        // key before the manager's cross-reference checks see it.
        om9_archive::admit_json(bytes, &om9_core::budget::Limits::default())?;
        let mut manager: Self = serde_json::from_slice(bytes)
            .map_err(|_| Error::Invalid("project manager schema"))?;
        if !matches!(manager.schema_version, 1 | PROJECT_SCHEMA) {
            return Err(Error::Unsupported("project manager schema"));
        }
        if manager.rows.is_empty()
            || manager.rows.len() > MAX_PROJECTS
            || manager.active >= manager.rows.len()
            || manager.next_project == 0 || manager.next_snapshot == 0 || manager.next_directory == 0
            || manager.rows.iter().any(|row| row.slots.is_empty() || row.slots.len() > MAX_SLOTS)
        {
            return Err(Error::Invalid("project manager rows"));
        }
        let mut project_ids = std::collections::BTreeSet::new();
        let mut snapshot_ids = std::collections::BTreeSet::new();
        let mut directory_ids = std::collections::BTreeSet::new();
        for kind in [DirectoryKind::Customer, DirectoryKind::Designer, DirectoryKind::Status] {
            let entries = manager.directory.entries(kind);
            if entries.len() > 256 { return Err(Error::Budget("project directory")); }
            for entry in entries {
                validate_text(&entry.name, MAX_CUSTOMER, "directory name")?;
                if entry.id == 0 || entry.id >= manager.next_directory || !directory_ids.insert(entry.id) {
                    return Err(Error::Invalid("directory id"));
                }
            }
        }
        for row in &manager.rows {
            validate_text(&row.name, MAX_NAME, "project name")?;
            validate_optional_text(&row.notes, MAX_NOTES, "project notes")?;
            validate_optional_text(&row.customer, MAX_CUSTOMER, "customer label")?;
            row.metadata.validate()?;
            manager.validate_directory_link(DirectoryKind::Customer, row.metadata.customer_id, &row.customer)?;
            manager.validate_directory_link(DirectoryKind::Designer, row.metadata.designer_id, &row.metadata.designer)?;
            manager.validate_directory_link(DirectoryKind::Status, row.metadata.status_id, &row.metadata.status)?;
            if row.id.0 == 0 || row.id.0 >= manager.next_project || !project_ids.insert(row.id) {
                return Err(Error::Invalid("project id"));
            }
            for (index, slot) in row.slots.iter().enumerate() {
                if slot.index != index {
                    return Err(Error::Invalid("jobbag slot index"));
                }
                if let Some(snapshot) = &slot.snapshot {
                    if (snapshot.archive.is_some() && snapshot.archive_asset.is_some())
                        || (snapshot.thumbnail.png.is_some() && snapshot.thumbnail.png_asset.is_some()) {
                        return Err(Error::Invalid("duplicate project asset"));
                    }
                    if snapshot.thumbnail.png_asset.as_ref().is_some_and(|asset| asset.length > 128 * 1024) {
                        return Err(Error::Budget("thumbnail bytes"));
                    }
                    if snapshot.thumbnail.png.as_ref().is_some_and(|png| png.is_empty() || png.len() > 128 * 1024) {
                        return Err(Error::Budget("thumbnail bytes"));
                    }
                    let count = snapshot.captured_ids.len();
                    if count == 0 { return Err(Error::Invalid("jobbag snapshot entities")); }
                    check_capture_roots(count, &om9_core::budget::Limits::default())?;
                    if snapshot.source_document.is_nil() || snapshot.source_revision == 0
                        || snapshot.thumbnail.source_revision != snapshot.source_revision
                        || snapshot.thumbnail.count != count {
                        return Err(Error::Invalid("jobbag snapshot provenance"));
                    }
                    if (!snapshot.root_classes.is_empty() && snapshot.root_classes.len() != count)
                        || (snapshot.entities.is_empty() && (snapshot.root_classes.len() != count || (snapshot.archive.is_none() && snapshot.archive_asset.is_none())))
                        || (!snapshot.entities.is_empty() && (snapshot.entities.len() != count
                            || snapshot.captured_ids.iter().zip(snapshot.entities.iter()).any(|(id, entity)| *id != entity.id)
                            || (!snapshot.root_classes.is_empty() && snapshot.entities.iter().map(BagObjectClass::of).ne(snapshot.root_classes.iter().copied()))))
                        || snapshot.captured_ids.iter().collect::<std::collections::BTreeSet<_>>().len() != count {
                        return Err(Error::Invalid("jobbag snapshot ids"));
                    }
                    if snapshot.id.0 == 0 || snapshot.id.0 >= manager.next_snapshot || !snapshot_ids.insert(snapshot.id.0) {
                        return Err(Error::Invalid("jobbag snapshot id"));
                    }
                }
            }
            for snapshot in row.master.iter().chain(row.named_masters.values()).chain(row.named_versions.values().flatten()) {
                if snapshot.document.is_none() && snapshot.archive.is_none() && snapshot.archive_asset.is_none() {
                    return Err(Error::Invalid("named jobbag has no document"));
                }
                if (snapshot.archive.is_some() && snapshot.archive_asset.is_some())
                    || (snapshot.thumbnail_png.is_some() && snapshot.thumbnail_asset.is_some()) {
                    return Err(Error::Invalid("duplicate project asset"));
                }
                if snapshot.thumbnail_asset.as_ref().is_some_and(|asset| asset.length > 128 * 1024) {
                    return Err(Error::Budget("thumbnail bytes"));
                }
                if snapshot.thumbnail_png.as_ref().is_some_and(|png| png.is_empty() || png.len() > 128 * 1024) {
                    return Err(Error::Budget("thumbnail bytes"));
                }
                if snapshot.id.0 == 0 || snapshot.id.0 >= manager.next_snapshot || !snapshot_ids.insert(snapshot.id.0) {
                    return Err(Error::Invalid("master snapshot id"));
                }
            }
            if row.named_masters.keys().any(|name| name == "Master" || !MASTER_NAMES.contains(&name.as_str())) {
                return Err(Error::Invalid("named master jobbag"));
            }
            if row.named_versions.iter().any(|(name, versions)| !MASTER_NAMES.contains(&name.as_str()) || versions.is_empty()) {
                return Err(Error::Invalid("named jobbag versions"));
            }
        }
        manager.required_assets()?;
        for row in &mut manager.rows {
            while row.slots.len() < SLOT_COUNT {
                row.slots.push(JobbagSlot::empty(row.slots.len()));
            }
        }
        manager.schema_version = PROJECT_SCHEMA;
        let _ = document_name;
        Ok(manager)
    }

    /// Clone the captured Master document for an atomic live restore.
    pub fn restore_master_document(&self) -> Option<Document> {
        self.master_document().cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use om9_core::{geometry::Geometry, model::Document};

    fn document_with_points(count: usize) -> Document {
        let mut document = Document::new("Jobbag test");
        for index in 0..count {
            let entity = Entity::new(
                format!("point {index}"),
                Geometry::Point([index as f64, 0., 0.]),
                None,
            );
            let id = entity.id;
            document.entities.insert(id, entity);
        }
        document
    }

    #[test]
    fn projects_have_stable_ids_and_duplicate_labels_are_allowed() {
        let mut manager = ProjectManager::new("Default");
        let first = manager.active().id;
        let second = manager.create_project("Default", 2).expect("create");
        assert_ne!(first, second);
        assert_eq!(manager.rows().len(), 2);
        assert_eq!(manager.active().id, second);
        manager.rename_project(first, "Renamed").expect("rename");
        assert_eq!(manager.rows()[0].name, "Renamed");
        manager.select(first).expect("select");
        assert_eq!(manager.active().id, first);
        assert!(manager.rename_project(ProjectId(999), "x").is_err());
    }

    #[test]
    fn empty_capture_is_explicit_and_does_not_create_a_slot() {
        let document = document_with_points(0);
        let mut manager = ProjectManager::new("Default");
        assert!(manager.capture_selection(0, &[], &document).is_err());
        assert!(manager.active().slot(0).expect("slot").is_empty());
        assert!(manager.active().thumbnail(0, &document).is_none());
    }

    #[test]
    fn capture_uses_object_budget_instead_of_fixed_small_selection_limit() {
        let document = document_with_points(65);
        let ids: Vec<_> = document.entities.keys().copied().collect();
        let mut manager = ProjectManager::new("Default");
        manager.capture_selection(0, &ids, &document).unwrap();
        let encoded = manager.to_json().unwrap();
        let restored = ProjectManager::from_json(&encoded, "Default").unwrap();
        assert_eq!(restored.active().slot(0).unwrap().snapshot.as_ref().unwrap().captured_ids.len(), 65);
        let original = manager.active().slot(0).unwrap().snapshot.as_ref().unwrap().id;
        let limits = om9_core::budget::Limits { max_objects: 64, ..Default::default() };
        let selected = document.entities.values().cloned().collect();
        assert!(matches!(manager.capture_with_limits(0, selected, &document, &limits), Err(Error::Budget(_))));
        assert_eq!(manager.active().slot(0).unwrap().snapshot.as_ref().unwrap().id, original);
        let mut stale = document.entities.values().next().unwrap().clone();
        stale.label.push_str(" changed");
        assert!(matches!(manager.capture(0, vec![stale], &document), Err(Error::Stale(_))));
        assert_eq!(manager.active().slot(0).unwrap().snapshot.as_ref().unwrap().id, original);
    }

    #[test]
    fn capture_two_objects_does_not_mutate_geometry_and_keeps_revision_thumbnail() {
        let document = document_with_points(2);
        let ids: Vec<_> = document.entities.keys().copied().collect();
        let before = document.clone();
        let mut manager = ProjectManager::new("Default");
        let snapshot = manager
            .capture_selection(1, &ids, &document)
            .expect("capture");
        assert_eq!(before.entities, document.entities);
        assert_eq!(
            manager
                .active()
                .slot(1)
                .expect("slot")
                .snapshot
                .as_ref()
                .unwrap()
                .id,
            snapshot
        );
        let thumb = manager.active().thumbnail(1, &document).expect("thumbnail");
        assert_eq!(thumb.count, 2);
        assert_eq!(thumb.source_revision, document.revision);
        assert_eq!(thumb.kind, "Point");
        assert_eq!(manager.active().slot(0).expect("slot").snapshot, None);
    }

    #[test]
    fn deleting_a_captured_object_keeps_the_slot_available() {
        let mut document = document_with_points(1);
        let id = *document.entities.keys().next().unwrap();
        let mut manager = ProjectManager::new("Default");
        manager
            .capture_selection(0, &[id], &document)
            .expect("capture");
        assert!(manager.slot_available(0, &document));
        document.entities.remove(&id);
        assert!(manager.slot_available(0, &document));
        assert!(manager.active().thumbnail(0, &document).is_some());
    }

    #[test]
    fn restore_uses_new_ids_and_repeated_restore_does_not_collide() {
        let mut document = document_with_points(2);
        let ids: Vec<_> = document.entities.keys().copied().collect();
        let mut manager = ProjectManager::new("Default");
        manager
            .capture_selection(0, &ids, &document)
            .expect("capture");
        let first = manager.restore_entities(0).expect("restore");
        let second = manager.restore_entities(0).expect("restore twice");
        assert_eq!(first.len(), 2);
        assert_eq!(second.len(), 2);
        for entity in &first {
            assert!(!ids.contains(&entity.id));
        }
        let first_ids: std::collections::BTreeSet<_> = first.iter().map(|e| e.id).collect();
        let second_ids: std::collections::BTreeSet<_> = second.iter().map(|e| e.id).collect();
        assert!(first_ids.is_disjoint(&second_ids));
        let inserted = manager.insert_into(0, &mut document).expect("insert");
        assert_eq!(inserted.len(), 2);
        assert_eq!(document.entities.len(), 4);
    }

    #[test]
    fn restore_preserves_placement_and_rename_only_is_persistent() {
        let mut document = document_with_points(1);
        let entity = document.entities.values_mut().next().unwrap();
        entity.placement = om9_core::geometry::Transform::translation([12., 23., 34.]);
        let id = entity.id;
        let placement = entity.placement.clone();
        let mut manager = ProjectManager::new("Default");
        assert!(!manager.has_persistent_state());
        manager.rename_project(manager.active().id, "Renamed").unwrap();
        assert!(manager.has_persistent_state());
        manager.capture_selection(0, &[id], &document).unwrap();
        assert_eq!(manager.restore_entities(0).unwrap()[0].placement, placement);
    }

    #[test]
    fn decode_rejects_duplicate_project_ids() {
        let mut manager = ProjectManager::new("Default");
        manager.create_project("Second", 1).unwrap();
        manager.rows[1].id = manager.rows[0].id;
        let bytes = serde_json::to_vec(&manager).unwrap();
        assert!(ProjectManager::from_json(&bytes, "document").is_err());
        let valid = String::from_utf8(ProjectManager::new("Default").to_json().unwrap()).unwrap();
        let duplicate = valid.replacen("\"slots\":", "\"named_versions\":{\"Master\":[],\"Master\":[]},\"slots\":", 1);
        assert_ne!(duplicate, valid);
        assert!(matches!(ProjectManager::from_json(duplicate.as_bytes(), "document"), Err(Error::Invalid("duplicate JSON key"))));
        let mut wrong_type = serde_json::to_value(ProjectManager::new("Default")).unwrap();
        wrong_type["active"] = serde_json::json!("not an index");
        assert!(matches!(ProjectManager::from_json(&serde_json::to_vec(&wrong_type).unwrap(), "document"), Err(Error::Invalid("project manager schema"))));
        let mut oversized = ProjectManager::new("Default");
        oversized.active_mut().notes = "x".repeat(MAX_MANAGER_JSON_BYTES);
        assert!(matches!(oversized.to_json(), Err(Error::Budget("project manager state bytes"))));
    }

    #[test]
    fn slots_expand_compact_and_migrate_legacy_rows() {
        let document = document_with_points(1);
        let id = *document.entities.keys().next().unwrap();
        let mut manager = ProjectManager::new("Default");
        manager.active_mut().slots.truncate(3);
        let mut legacy = serde_json::to_value(&manager).unwrap();
        legacy.as_object_mut().unwrap().remove("schema_version");
        let legacy = serde_json::to_vec(&legacy).unwrap();
        let mut manager = ProjectManager::from_json(&legacy, "test").unwrap();
        assert_eq!(manager.schema_version, PROJECT_SCHEMA);
        assert_eq!(manager.active().slots.len(), 10);
        let snapshot = manager.capture_selection(9, &[id], &document).unwrap();
        assert_eq!(manager.active().slots.len(), 11);
        manager.compact_slots();
        assert_eq!(manager.active().slots.len(), 10);
        assert_eq!(manager.active().slots[0].snapshot.as_ref().unwrap().id, snapshot);
    }

    #[test]
    fn unsupported_project_schema_is_rejected() {
        let manager = ProjectManager::new("Default");
        let mut value = serde_json::to_value(manager).unwrap();
        value["schema_version"] = serde_json::json!(PROJECT_SCHEMA + 1);
        assert!(matches!(ProjectManager::from_json(&serde_json::to_vec(&value).unwrap(), "test"), Err(Error::Unsupported(_))));
    }

    #[test]
    fn row_visibility_and_timer_survive_serialization() {
        let mut manager = ProjectManager::new("Default");
        let row = manager.create_project("Customer", 1).unwrap();
        manager.timer_start(row, 100).unwrap();
        manager.timer_stop(row, 135).unwrap();
        manager.timer_start(row, 150).unwrap();
        manager.set_visible(row, false).unwrap();
        let bytes = manager.to_json().unwrap();
        let mut reopened = ProjectManager::from_json(&bytes, "unused").unwrap();
        assert!(!reopened.active().visible);
        assert_eq!(reopened.active().elapsed_seconds, 35);
        reopened.timer_stop(row, 170).unwrap();
        assert_eq!(reopened.active().elapsed_seconds, 55);
        reopened.timer_reset(row, 180).unwrap();
        assert_eq!(reopened.active().elapsed_seconds, 0);
        reopened.delete_project(row).unwrap();
        assert_eq!(reopened.rows().len(), 1);
    }

    #[test]
    fn metadata_draft_validates_atomically_and_import_remaps_ids() {
        let mut manager = ProjectManager::new("Original");
        let id = manager.active().id;
        let bad = ProjectMetadata { due_date: Some("2025-02-29".into()), ..Default::default() };
        assert!(manager.update_project(id, "Changed", "", "", bad).is_err());
        assert_eq!(manager.active().name, "Original");
        let good = ProjectMetadata { due_date: Some("2028-02-29".into()), metal_weight_grams: Some(5.5), ..Default::default() };
        manager.update_project(id, "Changed", "Customer", "Note", good).unwrap();
        let archive = manager.export_project(id).unwrap();
        let imported = manager.import_project(&archive, &om9_core::budget::Limits::default()).unwrap();
        assert_ne!(imported, id);
        assert_eq!(manager.active().name, "Changed");
        assert_eq!(manager.active().metadata.metal_weight_grams, Some(5.5));
    }

    #[test]
    fn directory_links_survive_rename_and_import_collisions() {
        let mut source = ProjectManager::new("Source");
        let customer = source.create_directory_entry(DirectoryKind::Customer, "Alice").unwrap();
        let id = source.active().id;
        let metadata = ProjectMetadata { customer_id: Some(customer), ..Default::default() };
        source.update_project(id, "Source", "Alice", "", metadata).unwrap();
        source.rename_directory_entry(DirectoryKind::Customer, customer, "Alicia").unwrap();
        assert_eq!(source.active().customer, "Alicia");
        let archive = source.export_project(id).unwrap();
        let mut target = ProjectManager::new("Target");
        let collision = target.create_directory_entry(DirectoryKind::Customer, "Bob").unwrap();
        assert_eq!(collision, customer);
        target.import_project(&archive, &om9_core::budget::Limits::default()).unwrap();
        let imported = target.active().metadata.customer_id.unwrap();
        assert_ne!(imported, collision);
        assert_eq!(target.active().customer, "Alicia");
        target.remove_directory_entry(DirectoryKind::Customer, imported).unwrap();
        assert_eq!(target.active().customer, "Alicia");
        assert_eq!(target.active().metadata.customer_id, None);
    }

    #[test]
    fn replacing_named_version_preserves_position_and_changes_identity() {
        let mut manager = ProjectManager::new("Versions");
        let first = manager.capture_named_version_summary("Creation", 1, 2, vec![1]).unwrap();
        let second = manager.capture_named_version_summary("Creation", 3, 4, vec![2]).unwrap();
        let replacement = manager.replace_named_version_summary("Creation", first, 5, 6, vec![3]).unwrap();
        let versions = manager.named_master_versions("Creation");
        assert_eq!(versions.len(), 2);
        assert_eq!(versions[0].id, replacement);
        assert_eq!(versions[0].object_count, Some(5));
        assert_eq!(versions[1].id, second);
        assert!(manager.replace_named_version_summary("Creation", first, 1, 1, vec![4]).is_err());
    }

    #[test]
    fn snapshot_root_manifest_must_match_decoded_source() {
        let document = document_with_points(1);
        let id = *document.entities.keys().next().unwrap();
        let mut manager = ProjectManager::new("roots");
        manager.capture_selection(0, &[id], &document).unwrap();
        let snapshot = manager.active_mut().slots[0].snapshot.as_mut().unwrap();
        snapshot.root_classes = vec![BagObjectClass::Other];
        assert!(snapshot.validate_archive_roots(&document, &om9_core::budget::Limits::default()).is_ok());
        snapshot.root_classes[0] = BagObjectClass::Curve;
        assert!(snapshot.validate_archive_roots(&document, &om9_core::budget::Limits::default()).is_err());
        snapshot.root_classes[0] = BagObjectClass::Other;
        let mut wrong_source = document.clone();
        wrong_source.id = om9_core::identity::DocumentId::new();
        assert!(snapshot.validate_archive_roots(&wrong_source, &om9_core::budget::Limits::default()).is_err());
        wrong_source.id = document.id;
        wrong_source.revision += 1;
        assert!(snapshot.validate_archive_roots(&wrong_source, &om9_core::budget::Limits::default()).is_err());
        snapshot.captured_ids[0] = ObjectId::new();
        assert!(snapshot.validate_archive_roots(&document, &om9_core::budget::Limits::default()).is_err());
    }

    #[test]
    fn master_snapshot_is_a_deep_whole_document_copy() {
        let mut document = document_with_points(2);
        let layer = om9_core::model::Layer::new("Metal", [1, 2, 3, 255]);
        let layer_id = layer.id;
        document.layers.insert(layer_id, layer);
        document.layer_order.push(layer_id);
        let mut manager = ProjectManager::new("Default");
        manager.capture_master(&document).expect("master");
        let before = manager.master_document().expect("snapshot").clone();
        document.name = "Changed".into();
        document.entities.clear();
        assert_eq!(manager.master_document().unwrap().name, before.name);
        assert_eq!(
            manager.master_document().unwrap().entities.len(),
            before.entities.len()
        );
        assert_eq!(manager.master_layer_names(), vec!["Metal".to_string()]);
    }

    #[test]
    fn notes_and_customer_preserve_unicode_and_reject_file_paths() {
        let mut manager = ProjectManager::new("Default");
        manager
            .set_notes("Khách hàng: Nguyễn Văn A")
            .expect("unicode notes");
        manager.set_customer("Nhẫn cưới").expect("unicode customer");
        assert_eq!(manager.active().notes, "Khách hàng: Nguyễn Văn A");
        assert_eq!(manager.active().customer, "Nhẫn cưới");
        manager.set_notes("").expect("clear notes");
        assert_eq!(manager.active().notes, "");
        assert!(manager.set_customer("C:\\\\secret.txt").is_err());
    }

    #[test]
    fn invalid_slot_index_is_rejected() {
        let document = document_with_points(1);
        let id = *document.entities.keys().next().unwrap();
        let mut manager = ProjectManager::new("Default");
        assert!(
            manager
                .capture_selection(SLOT_COUNT, &[id], &document)
                .is_err()
        );
        assert!(manager.restore_entities(SLOT_COUNT).is_err());
    }
}
