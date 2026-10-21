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
    line_pattern: Option<om9_core::identity::LinePatternId>,
    line_pattern_source: u32,
    material: Option<om9_core::identity::MaterialId>,
    material_source: u32,
    plot_color: Option<[u8; 4]>,
    plot_weight_mm: Option<f64>,
    plot_color_source: Option<u32>,
    plot_weight_source: Option<u32>,
    display_order: i32,
    space: u32,
    viewport_id: Option<om9_core::identity::ViewportId>,
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
                line_pattern: e.attributes.line_pattern,
                line_pattern_source: e.attributes.line_pattern_source,
                material: e.attributes.material,
                material_source: e.attributes.material_source,
                plot_color: e.attributes.plot_color,
                plot_weight_mm: e.attributes.plot_weight_mm,
                plot_color_source: e.attributes.plot_color_source,
                plot_weight_source: e.attributes.plot_weight_source,
                display_order: e.attributes.display_order,
                space: e.attributes.space,
                viewport_id: e.attributes.viewport_id,
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
    line_pattern: Option<om9_core::identity::LinePatternId>,
    line_pattern_source: u32,
    material: Option<om9_core::identity::MaterialId>,
    plot_color: Option<[u8; 4]>,
    plot_weight_mm: Option<f64>,
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
                    line_pattern: l.line_pattern,
                    line_pattern_source: l.line_pattern_source,
                    material: l.material,
                    plot_color: l.plot_color,
                    plot_weight_mm: l.plot_weight_mm,
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
pub(crate) struct Groups<'a>(pub &'a BTreeMap<om9_core::identity::GroupId, om9_core::model::Group>);
impl Serialize for Groups<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct Row<'a> {
            id: om9_core::identity::GroupId,
            name: &'a str,
            members: &'a [ObjectId],
        }
        let mut map = s.serialize_map(Some(self.0.len()))?;
        for (id, group) in self.0 {
            map.serialize_entry(
                id,
                &Row {
                    id: group.id,
                    name: &group.name,
                    members: &group.members,
                },
            )?;
        }
        map.end()
    }
}
pub(crate) struct LinePatterns<'a>(
    pub &'a BTreeMap<om9_core::identity::LinePatternId, om9_core::exchange::LinePattern>,
);
impl Serialize for LinePatterns<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct Row<'a> {
            id: om9_core::identity::LinePatternId,
            name: &'a str,
            segments: &'a [om9_core::exchange::LinePatternSegment],
        }
        let mut map = s.serialize_map(Some(self.0.len()))?;
        for (id, pattern) in self.0 {
            map.serialize_entry(
                id,
                &Row {
                    id: pattern.id,
                    name: &pattern.name,
                    segments: &pattern.segments,
                },
            )?;
        }
        map.end()
    }
}
pub(crate) struct TextStyles<'a>(
    pub &'a BTreeMap<om9_core::identity::TextStyleId, om9_core::exchange::TextStyle>,
);
impl Serialize for TextStyles<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct Row<'a> {
            id: om9_core::identity::TextStyleId,
            name: &'a str,
        }
        let mut map = s.serialize_map(Some(self.0.len()))?;
        for (id, style) in self.0 {
            map.serialize_entry(
                id,
                &Row {
                    id: style.id,
                    name: &style.name,
                },
            )?;
        }
        map.end()
    }
}
pub(crate) struct DimStyles<'a>(
    pub &'a BTreeMap<om9_core::identity::DimStyleId, om9_core::exchange::DimStyle>,
);
impl Serialize for DimStyles<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct Row<'a> {
            id: om9_core::identity::DimStyleId,
            name: &'a str,
            text_height: f64,
            length_resolution: f64,
        }
        let mut map = s.serialize_map(Some(self.0.len()))?;
        for (id, style) in self.0 {
            map.serialize_entry(
                id,
                &Row {
                    id: style.id,
                    name: &style.name,
                    text_height: style.text_height,
                    length_resolution: style.length_resolution,
                },
            )?;
        }
        map.end()
    }
}
pub(crate) struct Materials<'a>(
    pub &'a BTreeMap<om9_core::identity::MaterialId, om9_core::exchange::RenderMaterial>,
);
impl Serialize for Materials<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct Row<'a> {
            id: om9_core::identity::MaterialId,
            name: &'a str,
            diffuse: [u8; 4],
        }
        let mut map = s.serialize_map(Some(self.0.len()))?;
        for (id, material) in self.0 {
            map.serialize_entry(
                id,
                &Row {
                    id: material.id,
                    name: &material.name,
                    diffuse: material.diffuse,
                },
            )?;
        }
        map.end()
    }
}
pub(crate) struct HatchPatterns<'a>(
    pub &'a BTreeMap<om9_core::identity::HatchPatternId, om9_core::exchange::HatchPattern>,
);
impl Serialize for HatchPatterns<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct Row<'a> {
            id: om9_core::identity::HatchPatternId,
            name: &'a str,
            fill_type: u32,
            lines: &'a [om9_core::exchange::HatchPatternLine],
        }
        let mut map = s.serialize_map(Some(self.0.len()))?;
        for (id, pattern) in self.0 {
            map.serialize_entry(
                id,
                &Row {
                    id: pattern.id,
                    name: &pattern.name,
                    fill_type: pattern.fill_type,
                    lines: &pattern.lines,
                },
            )?;
        }
        map.end()
    }
}
pub(crate) struct Lights<'a>(
    pub &'a BTreeMap<om9_core::identity::LightId, om9_core::exchange::RenderLight>,
);
impl Serialize for Lights<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct Row<'a> {
            id: om9_core::identity::LightId,
            name: &'a str,
            style: u32,
            location: [f64; 3],
            direction: [f64; 3],
            diffuse: [u8; 4],
            intensity: f64,
        }
        let mut map = s.serialize_map(Some(self.0.len()))?;
        for (id, light) in self.0 {
            map.serialize_entry(
                id,
                &Row {
                    id: light.id,
                    name: &light.name,
                    style: light.style,
                    location: light.location,
                    direction: light.direction,
                    diffuse: light.diffuse,
                    intensity: light.intensity,
                },
            )?;
        }
        map.end()
    }
}
pub(crate) struct Exchange<'a>(pub &'a om9_core::exchange::ExchangeMetadata);
impl Serialize for Exchange<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct Row<'a> {
            line_patterns: LinePatterns<'a>,
            text_styles: TextStyles<'a>,
            materials: Materials<'a>,
            dim_styles: DimStyles<'a>,
            hatch_patterns: HatchPatterns<'a>,
            lights: Lights<'a>,
        }
        Row {
            line_patterns: LinePatterns(&self.0.line_patterns),
            text_styles: TextStyles(&self.0.text_styles),
            materials: Materials(&self.0.materials),
            dim_styles: DimStyles(&self.0.dim_styles),
            hatch_patterns: HatchPatterns(&self.0.hatch_patterns),
            lights: Lights(&self.0.lights),
        }
        .serialize(s)
    }
}
