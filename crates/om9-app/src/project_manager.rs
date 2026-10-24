// SPDX-License-Identifier: LGPL-2.1-or-later
//! Matrix Project Manager and Job Bag state (OM9-L2-03-15).
//!
//! This module owns project rows, three job-bag slots per project, the Master
//! job bag and project notes. It is deliberately independent of egui and of the
//! document transaction path: snapshots clone entities; restoring hands fresh
//! entities back to the caller, which keeps Undo/Redo in the Controller.
use om9_core::{
    Error, Result,
    identity::ObjectId,
    model::{Document, Entity},
};
use serde::{Deserialize, Serialize};

pub const SLOT_COUNT: usize = 3;
const MAX_PROJECTS: usize = 64;
const MAX_CAPTURE: usize = 64;
const MAX_NAME: usize = 64;
const MAX_NOTES: usize = 4096;
const MAX_CUSTOMER: usize = 128;

/// Stable, human-readable project identity. Duplicate labels are allowed; IDs
/// are never derived from the label.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ProjectId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Thumbnail {
    pub label: String,
    pub count: usize,
    /// Geometry kind summary (`Mesh`, `Brep`, `Curve`, ...); no pixels are
    /// fabricated when the source geometry is unavailable.
    pub kind: String,
    pub source_revision: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: SnapshotId,
    pub source_document: om9_core::identity::DocumentId,
    pub source_revision: u64,
    pub captured_ids: Vec<ObjectId>,
    pub entities: Vec<Entity>,
    pub thumbnail: Thumbnail,
}

impl Snapshot {
    fn new(id: SnapshotId, document: &Document, entities: Vec<Entity>) -> Result<Self> {
        if entities.is_empty() {
            return Err(Error::Invalid("jobbag capture selection"));
        }
        if entities.len() > MAX_CAPTURE {
            return Err(Error::Budget("jobbag capture selection"));
        }
        let mut ids = Vec::with_capacity(entities.len());
        for entity in &entities {
            if ids.contains(&entity.id) {
                return Err(Error::Invalid("duplicate jobbag object"));
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
            },
            entities,
        })
    }

    /// A slot is available only while every captured object still exists and
    /// belongs to the same document. A deleted object must show unavailable,
    /// never a stale thumbnail of different geometry.
    pub fn available(&self, document: &Document) -> bool {
        self.source_document == document.id
            && self
                .captured_ids
                .iter()
                .all(|id| document.entities.contains_key(id))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
pub struct MasterSnapshot {
    pub id: SnapshotId,
    pub document: Document,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectRow {
    pub id: ProjectId,
    pub name: String,
    pub notes: String,
    /// Customer/job label; free text, never used as a path.
    pub customer: String,
    pub created_revision: u64,
    pub slots: Vec<JobbagSlot>,
    pub master: Option<MasterSnapshot>,
}

impl ProjectRow {
    pub fn new(id: ProjectId, name: impl Into<String>, revision: u64) -> Self {
        Self {
            id,
            name: name.into(),
            notes: String::new(),
            customer: String::new(),
            created_revision: revision,
            slots: (0..SLOT_COUNT).map(JobbagSlot::empty).collect(),
            master: None,
        }
    }
    pub fn slot(&self, index: usize) -> Option<&JobbagSlot> {
        self.slots.get(index)
    }
    pub fn thumbnail(&self, index: usize, document: &Document) -> Option<&Thumbnail> {
        self.slot(index).and_then(|slot| slot.thumbnail(document))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectManager {
    rows: Vec<ProjectRow>,
    active: usize,
    next_project: u64,
    next_snapshot: u64,
}

impl ProjectManager {
    pub fn new(name: impl Into<String>) -> Self {
        let row = ProjectRow::new(ProjectId(1), name, 1);
        Self {
            rows: vec![row],
            active: 0,
            next_project: 2,
            next_snapshot: 1,
        }
    }

    /// True when there is user-created project state worth embedding. A fresh
    /// default row with empty slots does not dirty a plain document.
    pub fn has_persistent_state(&self) -> bool {
        self.rows.len() > 1
            || self.rows.iter().any(|row| {
                !row.notes.is_empty()
                    || !row.customer.is_empty()
                    || row.master.is_some()
                    || row.slots.iter().any(|slot| slot.snapshot.is_some())
            })
    }

    pub fn rows(&self) -> &[ProjectRow] {
        &self.rows
    }

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
        self.active = index;
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
        Ok(id)
    }

    pub fn rename_project(&mut self, id: ProjectId, name: &str) -> Result<()> {
        let name = validate_text(name, MAX_NAME, "project name")?;
        let row = self
            .rows
            .iter_mut()
            .find(|row| row.id == id)
            .ok_or(Error::Missing("project row"))?;
        row.name = name;
        Ok(())
    }

    pub fn set_notes(&mut self, notes: &str) -> Result<()> {
        let notes = validate_text(notes, MAX_NOTES, "project notes")?;
        self.active_mut().notes = notes;
        Ok(())
    }

    pub fn set_customer(&mut self, customer: &str) -> Result<()> {
        let customer = validate_text(customer, MAX_CUSTOMER, "customer label")?;
        if customer.contains(['/', '\\']) && (customer.starts_with('/') || customer.contains(":\\"))
        {
            return Err(Error::Invalid("customer label must not be a file path"));
        }
        self.active_mut().customer = customer;
        Ok(())
    }

    pub fn capture(
        &mut self,
        slot: usize,
        selected: Vec<Entity>,
        document: &Document,
    ) -> Result<SnapshotId> {
        if slot >= SLOT_COUNT {
            return Err(Error::Invalid("jobbag slot index"));
        }
        if selected.is_empty() {
            return Err(Error::Invalid("jobbag capture selection"));
        }
        let id = SnapshotId(self.next_snapshot);
        let snapshot = Snapshot::new(id, document, selected)?;
        self.next_snapshot = self
            .next_snapshot
            .checked_add(1)
            .ok_or(Error::Overflow("jobbag snapshot id"))?;
        let row = self.active_mut();
        row.slots[slot].snapshot = Some(snapshot);
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
        let mut entities = Vec::with_capacity(selection.len());
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

    /// Return fresh entities for a job-bag slot. IDs are always new, and the
    /// placement is offset so repeated inserts do not overlap exactly.
    pub fn restore_entities(&self, slot: usize) -> Result<Vec<Entity>> {
        if slot >= SLOT_COUNT {
            return Err(Error::Invalid("jobbag slot index"));
        }
        let snapshot = self
            .active()
            .slot(slot)
            .and_then(|slot| slot.snapshot.as_ref())
            .ok_or(Error::Missing("jobbag slot"))?;
        let mut restored = Vec::with_capacity(snapshot.entities.len());
        for (index, source) in snapshot.entities.iter().enumerate() {
            let mut entity = source.clone();
            entity.id = ObjectId::new();
            entity.placement = om9_core::geometry::Transform::translation([
                (slot as f64 + 1.) * 2. + index as f64 * 0.01,
                0.,
                0.,
            ]);
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
        if document.entities.is_empty() {
            return Err(Error::Invalid("master jobbag empty document"));
        }
        let id = SnapshotId(self.next_snapshot);
        self.next_snapshot = self
            .next_snapshot
            .checked_add(1)
            .ok_or(Error::Overflow("master snapshot id"))?;
        self.active_mut().master = Some(MasterSnapshot {
            id,
            document: document.clone(),
        });
        Ok(id)
    }

    pub fn master_document(&self) -> Option<&Document> {
        self.active().master.as_ref().map(|master| &master.document)
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
    let value = value.trim();
    if value.is_empty() {
        return Err(Error::Invalid(what));
    }
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
        let bytes = serde_json::to_vec(self)
            .map_err(|error| Error::Io(format!("project manager serialization: {error}")))?;
        if bytes.len() > 16 * 1024 * 1024 {
            return Err(Error::Budget("project manager state bytes"));
        }
        Ok(bytes)
    }

    pub fn from_json(bytes: &[u8], document_name: &str) -> Result<Self> {
        if bytes.is_empty() || bytes.len() > 16 * 1024 * 1024 {
            return Err(Error::Invalid("project manager state bytes"));
        }
        let manager: Self = serde_json::from_slice(bytes)
            .map_err(|error| Error::Io(format!("project manager decode: {error}")))?;
        if manager.rows.is_empty()
            || manager.active >= manager.rows.len()
            || manager.rows.iter().any(|row| row.slots.len() != SLOT_COUNT)
        {
            return Err(Error::Invalid("project manager rows"));
        }
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
    fn deleting_a_captured_object_makes_the_slot_unavailable() {
        let mut document = document_with_points(1);
        let id = *document.entities.keys().next().unwrap();
        let mut manager = ProjectManager::new("Default");
        manager
            .capture_selection(0, &[id], &document)
            .expect("capture");
        assert!(manager.slot_available(0, &document));
        document.entities.remove(&id);
        assert!(!manager.slot_available(0, &document));
        assert!(manager.active().thumbnail(0, &document).is_none());
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
        assert!(manager.set_notes("").is_err());
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
