// SPDX-License-Identifier: LGPL-2.1-or-later
use super::{Attributes, Entity, Extension};
use crate::{
    geometry::{Geometry, Transform},
    identity::ObjectId,
};
use serde::ser::{SerializeMap, SerializeStruct};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;

pub(super) const CREATION_ORDER_SLOT: &str = "om9.object.creation-order";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SavedEntity {
    id: ObjectId,
    label: String,
    generation: u64,
    geometry_revision: u64,
    attributes_revision: u64,
    // Read files saved by interim builds that wrote the order as a field.
    #[serde(default)]
    creation_order: Option<[u64; 2]>,
    geometry: Geometry,
    placement: Transform,
    attributes: Attributes,
    #[serde(default, deserialize_with = "crate::identity::unique_named_slot_map")]
    extensions: BTreeMap<String, Extension>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CreationOrder {
    order: [u64; 2],
}

impl<'de> Deserialize<'de> for Entity {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let mut saved = SavedEntity::deserialize(deserializer)?;
        let stored_order = saved
            .extensions
            .get(CREATION_ORDER_SLOT)
            .filter(|slot| slot.version == 1)
            .and_then(|slot| serde_json::from_str::<CreationOrder>(&slot.payload).ok())
            .map(|payload| payload.order);
        if let Some(order) = stored_order {
            if saved.creation_order.is_some_and(|inline| inline != order) {
                return Err(serde::de::Error::custom(
                    "conflicting object creation order",
                ));
            }
            saved.extensions.remove(CREATION_ORDER_SLOT);
        }
        Ok(Self {
            id: saved.id,
            label: saved.label,
            generation: saved.generation,
            geometry_revision: saved.geometry_revision,
            attributes_revision: saved.attributes_revision,
            creation_order: stored_order.or(saved.creation_order),
            geometry: saved.geometry,
            placement: saved.placement,
            attributes: saved.attributes,
            extensions: saved.extensions,
        })
    }
}

impl Serialize for Entity {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let extensions = SavedExtensions(self);
        let has_extensions = !self.extensions.is_empty() || self.creation_order.is_some();
        let mut record = serializer.serialize_struct("Entity", 8 + usize::from(has_extensions))?;
        record.serialize_field("id", &self.id)?;
        record.serialize_field("label", &self.label)?;
        record.serialize_field("generation", &self.generation)?;
        record.serialize_field("geometry_revision", &self.geometry_revision)?;
        record.serialize_field("attributes_revision", &self.attributes_revision)?;
        record.serialize_field("geometry", &self.geometry)?;
        record.serialize_field("placement", &self.placement)?;
        record.serialize_field("attributes", &self.attributes)?;
        if has_extensions {
            record.serialize_field("extensions", &extensions)?;
        }
        record.end()
    }
}

struct SavedExtensions<'a>(&'a Entity);

impl Serialize for SavedExtensions<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let entity = self.0;
        let has_order = entity.creation_order.is_some();
        if has_order && entity.extensions.contains_key(CREATION_ORDER_SLOT) {
            return Err(serde::ser::Error::custom(
                "occupied object creation order slot",
            ));
        }
        let mut extensions =
            serializer.serialize_map(Some(entity.extensions.len() + usize::from(has_order)))?;
        for (slot, value) in &entity.extensions {
            extensions.serialize_entry(slot, value)?;
        }
        // The ordinary extension envelope is readable and preserved by v5
        // readers that predate this metadata. It stays outside runtime tools.
        if let Some(order) = entity.creation_order {
            let payload = serde_json::to_string(&CreationOrder { order })
                .map_err(serde::ser::Error::custom)?;
            extensions.serialize_entry(
                CREATION_ORDER_SLOT,
                &Extension {
                    version: 1,
                    payload,
                },
            )?;
        }
        extensions.end()
    }
}
