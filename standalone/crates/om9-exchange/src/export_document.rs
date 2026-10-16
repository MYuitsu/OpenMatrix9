// SPDX-License-Identifier: LGPL-2.1-or-later
//! Borrowed closed native ABI projection; OM9-only state never leaks into it.
use om9_core::{
    geometry::{Geometry, Transform},
    identity::{DefinitionId, LayerId, ObjectId},
    model::{Definition, Entity, Layer},
};
use serde::{
    Serialize, Serializer,
    ser::{SerializeMap, SerializeSeq},
};
use std::collections::BTreeMap;
#[derive(Serialize)]
struct Attributes {
    layer: Option<LayerId>,
    visible: bool,
    selectable: bool,
    locked: bool,
    color: Option<[u8; 4]>,
}
#[derive(Serialize)]
struct Object<'a> {
    id: ObjectId,
    label: &'a str,
    generation: u64,
    geometry_revision: u64,
    attributes_revision: u64,
    geometry: &'a Geometry,
    placement: Transform,
    attributes: Attributes,
}
impl<'a> From<&'a Entity> for Object<'a> {
    fn from(e: &'a Entity) -> Self {
        Self {
            id: e.id,
            label: &e.label,
            generation: e.generation,
            geometry_revision: e.geometry_revision,
            attributes_revision: e.attributes_revision,
            geometry: &e.geometry,
            placement: e.placement,
            attributes: Attributes {
                layer: e.attributes.layer,
                visible: e.attributes.visible && !e.attributes.hidden,
                selectable: e.attributes.selectable,
                locked: e.attributes.locked,
                color: e.attributes.color,
            },
        }
    }
}
#[derive(Serialize)]
struct LayerRow<'a> {
    id: LayerId,
    name: &'a str,
    parent: Option<LayerId>,
    visible: bool,
    selectable: bool,
    locked: bool,
    color: [u8; 4],
    revision: u64,
}
pub(crate) struct Layers<'a>(pub &'a BTreeMap<LayerId, Layer>);
impl Serialize for Layers<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(Some(self.0.len()))?;
        for (id, l) in self.0 {
            map.serialize_entry(
                id,
                &LayerRow {
                    id: l.id,
                    name: &l.name,
                    parent: l.parent,
                    visible: l.visible,
                    selectable: l.selectable,
                    locked: l.locked,
                    color: l.color,
                    revision: l.revision,
                },
            )?;
        }
        map.end()
    }
}
pub(crate) struct Entities<'a>(pub &'a BTreeMap<ObjectId, Entity>);
impl Serialize for Entities<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(Some(self.0.len()))?;
        for (id, e) in self.0 {
            map.serialize_entry(id, &Object::from(e))?;
        }
        map.end()
    }
}
struct Members<'a>(&'a [Entity]);
impl Serialize for Members<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut seq = s.serialize_seq(Some(self.0.len()))?;
        for e in self.0 {
            seq.serialize_element(&Object::from(e))?;
        }
        seq.end()
    }
}
pub(crate) struct Definitions<'a>(pub &'a BTreeMap<DefinitionId, Definition>);
impl Serialize for Definitions<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct Row<'a> {
            id: DefinitionId,
            label: &'a str,
            revision: u64,
            members: Members<'a>,
        }
        let mut map = s.serialize_map(Some(self.0.len()))?;
        for (id, d) in self.0 {
            map.serialize_entry(
                id,
                &Row {
                    id: d.id,
                    label: &d.label,
                    revision: d.revision,
                    members: Members(&d.members),
                },
            )?;
        }
        map.end()
    }
}
