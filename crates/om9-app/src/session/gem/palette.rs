// SPDX-License-Identifier: LGPL-2.1-or-later
use super::*;
use om9_core::{
    layers::{LayerAction, LayerPlan},
    model::Layer,
};

// Matrix palette order, including the extra User rows after the primary four.
const SLOTS: [u8; 24] = [
    5, 6, 7, 8, 9, 10, 11, 12, 17, 18, 19, 20, 25, 26, 27, 28, 21, 22, 23, 24, 29, 30, 31, 32,
];

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct GemSizeKey {
    shape: String,
    cut: String,
    material: String,
    length_um: i64,
    width_um: i64,
}

impl GemSizeKey {
    fn new(shape: &str, cut: &str, material: &str, length: f64, width: f64) -> Self {
        Self {
            shape: shape.to_ascii_lowercase(),
            cut: cut.to_ascii_lowercase(),
            material: material.trim().to_ascii_lowercase(),
            length_um: (length * 1000.).round() as i64,
            width_um: (width * 1000.).round() as i64,
        }
    }

    fn from_entity(entity: &Entity) -> Option<Self> {
        let ext = entity.extension(GEM_EXTENSION_SLOT)?;
        if let Some(record) = gem_record(entity) {
            let s = record.spec;
            return Some(Self::new(
                s.shape.tag(),
                s.cut.tag(),
                &record.material,
                s.length_mm,
                s.width_mm,
            ));
        }
        Self::from_sdk(&ext.payload)
    }

    fn from_sdk(payload: &str) -> Option<Self> {
        // SDK gems carry the outline name in "cut" and outer dimensions in mm.
        let value: serde_json::Value = serde_json::from_str(payload).ok()?;
        let shape = value.get("cut")?.as_str()?;
        let length = value
            .get("length_mm")
            .or(value.get("diameter_mm"))?
            .as_f64()?;
        let width = value
            .get("width_mm")
            .and_then(|v| v.as_f64())
            .unwrap_or(length);
        Some(Self::new(
            shape,
            "diamond",
            value
                .get("material")
                .and_then(|v| v.as_str())
                .unwrap_or("diamond"),
            length,
            width,
        ))
    }
}

/// Some add-ons attach generic tool metadata in the same slot. Only records
/// describing a stone inherit the gem palette in the viewport.
pub(crate) fn gem_wire_profile(entity: &Entity) -> Option<bool> {
    let ext = entity.extension(GEM_EXTENSION_SLOT)?;
    if let Some(record) = gem_record(entity) {
        let spec = record.spec;
        return Some(
            spec.shape == GemShape::Round
                && spec.cut == GemCut::Diamond
                && spec.profile.is_none()
                && spec.facet_count == spec.cut.default_facet_count(),
        );
    }
    GemSizeKey::from_sdk(&ext.payload).map(|_| false)
}

#[derive(Default)]
struct GemPalette {
    assignments: BTreeMap<GemSizeKey, PresetSlot>,
}

impl GemPalette {
    fn from_document(doc: &om9_core::model::Document) -> Self {
        let mut palette = Self::default();
        for entity in doc.entities.values() {
            let Some(slot) = entity
                .attributes
                .layer
                .and_then(|id| doc.layers.get(&id))
                .and_then(|l| l.preset_slot)
                .filter(|s| SLOTS.contains(&s.number()))
            else {
                continue;
            };
            let key = GemSizeKey::from_entity(entity).or_else(|| {
                let om9_core::geometry::Geometry::Instance(id) = entity.geometry else {
                    return None;
                };
                doc.definitions
                    .get(&id)?
                    .members
                    .iter()
                    .find_map(GemSizeKey::from_entity)
            });
            if let Some(key) = key {
                palette
                    .assignments
                    .entry(key)
                    .and_modify(|old| *old = (*old).min(slot))
                    .or_insert(slot);
            }
        }
        palette
    }

    fn slot(&mut self, key: GemSizeKey) -> PresetSlot {
        if let Some(slot) = self.assignments.get(&key) {
            return *slot;
        }
        let mut usage = [0usize; 33];
        for assigned in self.assignments.values() {
            usage[assigned.number() as usize] += 1;
        }
        let number = SLOTS
            .iter()
            .copied()
            .find(|&n| usage[n as usize] == 0)
            .unwrap_or_else(|| {
                *SLOTS[4..]
                    .iter()
                    .min_by_key(|&&n| usage[n as usize])
                    .unwrap()
            });
        let slot = PresetSlot::new(number).expect("palette slot");
        self.assignments.insert(key, slot);
        slot
    }
}

impl Controller {
    pub(crate) fn gem_preview_color(&self, session: &GemSession) -> [u8; 4] {
        let doc = self.document();
        if let Some(layer) = session
            .layer
            .or_else(|| {
                session
                    .replace
                    .or(session.target)
                    .and_then(|id| doc.entities.get(&id)?.attributes.layer)
            })
            .and_then(|id| doc.layers.get(&id))
        {
            return layer.color;
        }
        let key = GemSizeKey::new(
            session.shape.tag(),
            session.cut.tag(),
            &session.material,
            session.dimensions[0],
            session.dimensions[1],
        );
        let slot = GemPalette::from_document(doc).slot(key);
        self.layer_id(slot)
            .and_then(|id| doc.layers.get(&id))
            .map_or(slot.color(), |l| l.color)
    }

    /// Allocate from committed gems and earlier inserts in this transaction. Creating a
    /// missing preset is part of the same undo step as placing the first stone on it.
    pub(in crate::session) fn route_gem_inserts(
        &self,
        mutations: &mut Vec<Mutation>,
        explicit: &std::collections::BTreeSet<ObjectId>,
    ) -> Result<()> {
        let mut palette = None;
        let mut layers = BTreeMap::new();
        let mut creates = Vec::new();
        for mutation in mutations.iter_mut() {
            let Mutation::Insert { entity, .. } = mutation else {
                continue;
            };
            if explicit.contains(&entity.id) {
                continue;
            }
            let Some(key) = GemSizeKey::from_entity(entity) else {
                continue;
            };
            let slot = palette
                .get_or_insert_with(|| GemPalette::from_document(self.document()))
                .slot(key);
            let id = if let Some(id) = layers.get(&slot) {
                *id
            } else if let Some(id) = self.layer_id(slot) {
                self.ensure_layer_writable(Some(id))?;
                layers.insert(slot, id);
                id
            } else {
                let layer = Layer::preset(slot);
                let id = layer.id;
                creates.push(Mutation::Layers(Box::new(LayerPlan::capture(
                    &self.coordinator,
                    self.document,
                    LayerAction::Create {
                        layer,
                        make_current: false,
                    },
                    None,
                )?)));
                layers.insert(slot, id);
                id
            };
            entity.attributes.layer = Some(id);
            entity.attributes.color = None;
        }
        mutations.splice(0..0, creates);
        Ok(())
    }
}
