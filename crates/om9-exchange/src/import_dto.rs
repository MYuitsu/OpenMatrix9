// SPDX-License-Identifier: LGPL-2.1-or-later
//! Strict owned DTO for an independently parsed, hash-bound native source.
#![allow(dead_code)] // Strictly decode preserved fields; original bytes own unsupported semantics.
use om9_core::{
    Error, Result,
    budget::{Limits, Usage},
    geometry::{Geometry, Transform},
    retained::{NativeRecordKey, NativeRecordKind, NativeUnits, ResolvedBy, SourceUnits},
};
use serde::{Deserialize, de::IgnoredAny};
use serde_json::Value;
use std::collections::BTreeMap;
use uuid::Uuid;

fn invalid() -> Error {
    Error::Invalid("native source projection correspondence")
}
pub(crate) fn uuid<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Uuid, D::Error> {
    let s = String::deserialize(d)?;
    let id = Uuid::parse_str(&s).map_err(serde::de::Error::custom)?;
    let mut buffer = [0; 36];
    if id.hyphenated().encode_lower(&mut buffer).as_bytes() != s.as_bytes() {
        return Err(serde::de::Error::custom("noncanonical native UUID"));
    }
    Ok(id)
}
fn optional_uuid<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Option<Uuid>, D::Error> {
    #[derive(Deserialize)]
    struct Canonical(#[serde(deserialize_with = "uuid")] Uuid);
    Option::<Canonical>::deserialize(d).map(|value| value.map(|id| id.0))
}
fn uuid_list<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Vec<Uuid>, D::Error> {
    #[derive(Deserialize)]
    struct Canonical(#[serde(deserialize_with = "uuid")] Uuid);
    Vec::<Canonical>::deserialize(d).map(|values| values.into_iter().map(|value| value.0).collect())
}
pub(crate) fn kind(value: u32) -> Result<NativeRecordKind> {
    Ok(match value {
        1 => NativeRecordKind::Image,
        2 => NativeRecordKind::TextureMapping,
        3 => NativeRecordKind::Material,
        4 => NativeRecordKind::LineType,
        5 => NativeRecordKind::Layer,
        6 => NativeRecordKind::Group,
        7 | 17 => NativeRecordKind::Other { native_type: value },
        8 => NativeRecordKind::DimStyle,
        9 => NativeRecordKind::Light,
        10 => NativeRecordKind::HatchPattern,
        11 => NativeRecordKind::Definition,
        12 => NativeRecordKind::Geometry,
        13 => NativeRecordKind::History,
        14 => NativeRecordKind::RenderContent,
        15 => NativeRecordKind::EmbeddedFile,
        _ => return Err(invalid()),
    })
}
pub(crate) fn type_number(name: &str) -> Result<u32> {
    Ok(match name {
        "Image" => 1,
        "TextureMapping" => 2,
        "Material" | "RenderMaterial" => 3,
        "LinePattern" => 4,
        "Layer" => 5,
        "Group" => 6,
        "TextStyle" => 7,
        "DimStyle" => 8,
        "RenderLight" => 9,
        "HatchPattern" => 10,
        "InstanceDefinition" => 11,
        "ModelGeometry" => 12,
        "HistoryRecord" => 13,
        "RenderContent" => 14,
        "EmbeddedFile" => 15,
        "SectionStyle" => 17,
        _ => return Err(invalid()),
    })
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Archive {
    pub version: u32,
    pub opennurbs_version: u32,
    pub file_bytes: u64,
    pub endmark_bytes: u64,
    pub index_semantics: String,
    pub source_sha256: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct OriginalUnits {
    pub native_enum: u32,
    pub name: String,
    pub millimeters_per_unit: Option<f64>,
    pub meters_per_unit: Option<f64>,
    pub requires_resolution: bool,
    pub absolute_tolerance: f64,
    pub angle_tolerance_radians: f64,
    pub relative_tolerance: f64,
    pub distance_display_mode: u32,
    pub distance_display_precision: u32,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Units {
    pub model: OriginalUnits,
    pub page: OriginalUnits,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ResolvedUnit {
    pub millimeters_per_unit: f64,
    pub resolved_by: String,
    pub original_native_enum: u32,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Resolutions {
    pub model: ResolvedUnit,
    pub page: ResolvedUnit,
    pub definitions: BTreeMap<String, Option<ResolvedUnit>>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Relation {
    pub component_type: String,
    #[serde(deserialize_with = "optional_uuid")]
    pub id: Option<Uuid>,
    pub native_index: Option<i32>,
    pub role: String,
    pub scope: String,
    #[serde(default, deserialize_with = "optional_uuid")]
    pub application_id: Option<Uuid>,
    #[serde(default)]
    pub channel_id: Option<i32>,
    #[serde(default)]
    pub transform: Option<Transform>,
}
#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub(crate) struct Preservation {
    pub admission_required: bool,
    pub default_fields: BTreeMap<String, Value>,
    pub nondefault_fields: BTreeMap<String, Value>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ComponentTable {
    #[serde(deserialize_with = "uuid")]
    pub id: Uuid,
    pub native_index: Option<i32>,
    pub native_component_type: u32,
    pub component_type: String,
    pub native_kind: String,
    pub native_class_uuid: String,
    pub name: String,
    pub capability: String,
    pub admission_required: bool,
    pub relations: Vec<Relation>,
    pub userdata: Vec<IgnoredAny>,
    #[serde(default, deserialize_with = "optional_uuid")]
    pub render_material_instance_id: Option<Uuid>,
    #[serde(default)]
    pub texture_count: Option<u64>,
    #[serde(default)]
    pub settings: Option<Value>,
    #[serde(default)]
    pub preservation: Option<Preservation>,
    #[serde(default)]
    pub native_component_body: Option<IgnoredAny>,
    #[serde(default)]
    pub opaque_history_values: Option<bool>,
}
#[derive(Deserialize)]
#[serde(untagged)]
enum ComponentPacket {
    Table(Box<ComponentTable>),
    Light(Box<Object>),
}
#[derive(Deserialize)]
#[serde(try_from = "ComponentPacket")]
pub(crate) struct Component {
    table: ComponentTable,
    pub light: Option<Box<Object>>,
}
impl std::ops::Deref for Component {
    type Target = ComponentTable;
    fn deref(&self) -> &Self::Target {
        &self.table
    }
}
impl TryFrom<ComponentPacket> for Component {
    type Error = &'static str;
    fn try_from(value: ComponentPacket) -> std::result::Result<Self, Self::Error> {
        match value {
            ComponentPacket::Table(table) => Ok(Self {
                table: *table,
                light: None,
            }),
            ComponentPacket::Light(mut light) => {
                if light.native_component_type != 9
                    || light.component_type.as_deref() != Some("RenderLight")
                    || light.geometry.is_some()
                    || light.capability != "retained_only"
                {
                    return Err("native light row");
                }
                let table = ComponentTable {
                    id: light.id,
                    native_index: light.native_index,
                    native_component_type: 9,
                    component_type: light.component_type.take().unwrap(),
                    name: std::mem::take(&mut light.label),
                    native_kind: std::mem::take(&mut light.native_kind),
                    native_class_uuid: std::mem::take(&mut light.native_class_uuid),
                    capability: light.capability.clone(),
                    admission_required: light.admission_required,
                    relations: std::mem::take(&mut light.relations),
                    userdata: std::mem::take(&mut light.userdata),
                    preservation: Some(std::mem::take(&mut light.preservation)),
                    render_material_instance_id: None,
                    texture_count: None,
                    settings: None,
                    native_component_body: None,
                    opaque_history_values: None,
                };
                Ok(Self {
                    table,
                    light: Some(light),
                })
            }
        }
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Attributes {
    #[serde(deserialize_with = "optional_uuid")]
    pub layer: Option<Uuid>,
    pub native_layer_index: i32,
    pub color: Option<[u8; 4]>,
    pub color_source: u32,
    pub visible: bool,
    pub locked: bool,
    pub mode: u32,
    pub space: u32,
    #[serde(deserialize_with = "optional_uuid")]
    pub viewport_id: Option<Uuid>,
    pub material_source: u32,
    pub native_material_index: i32,
    pub line_pattern_source: u32,
    pub native_line_pattern_index: i32,
    pub groups: Vec<i32>,
    pub display_order: i32,
    pub plot_weight_mm: f64,
    pub plot_color: Option<[u8; 4]>,
    pub plot_color_source: u32,
    pub plot_weight_source: u32,
    pub userdata: Vec<IgnoredAny>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Object {
    #[serde(default)]
    pub component_type: Option<String>,
    #[serde(deserialize_with = "uuid")]
    pub id: Uuid,
    pub native_index: Option<i32>,
    pub native_component_type: u32,
    pub label: String,
    pub native_kind: String,
    pub native_class_uuid: String,
    pub capability: String,
    pub admission_required: bool,
    pub geometry: Option<Geometry>,
    pub placement: Option<Transform>,
    pub native_placement: Transform,
    #[serde(default)]
    pub projection_units_basis: Option<String>,
    pub attributes: Attributes,
    pub relations: Vec<Relation>,
    pub is_definition_member: bool,
    pub channels: IgnoredAny,
    pub userdata: Vec<IgnoredAny>,
    pub diagnostics: Vec<String>,
    pub unrepresented_attributes: Vec<String>,
    pub unsupported_fields: Vec<String>,
    pub preservation: Preservation,
    #[serde(default)]
    pub dependency_readiness: Option<IgnoredAny>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Layer {
    #[serde(deserialize_with = "uuid")]
    pub id: Uuid,
    pub native_index: i32,
    pub native_component_type: u32,
    pub admission_required: bool,
    pub name: String,
    #[serde(deserialize_with = "optional_uuid")]
    pub parent: Option<Uuid>,
    pub color: [u8; 4],
    pub plot_color: [u8; 4],
    pub plot_weight_mm: f64,
    pub visible: bool,
    pub locked: bool,
    pub relations: Vec<Relation>,
    pub userdata: Vec<IgnoredAny>,
    pub preservation: Preservation,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Definition {
    #[serde(deserialize_with = "uuid")]
    pub id: Uuid,
    pub native_component_type: u32,
    pub label: String,
    #[serde(deserialize_with = "uuid_list")]
    pub members: Vec<Uuid>,
    pub linked: bool,
    pub units: DefinitionUnits,
    pub relations: Vec<Relation>,
    pub userdata: Vec<IgnoredAny>,
    pub admission_required: bool,
    pub dependency_readiness: IgnoredAny,
    pub preservation: Preservation,
    #[serde(default)]
    pub native_bbox: Option<IgnoredAny>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DefinitionUnits {
    pub native_enum: u32,
    pub name: String,
    pub meters_per_unit: Option<f64>,
    pub millimeters_per_unit: Option<f64>,
    pub requires_resolution: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Table {
    #[serde(default)]
    pub present: Option<bool>,
    pub table: String,
    pub crc_errors: u64,
    pub critical_errors: u64,
    pub recoverable_errors: u64,
    pub read_records: u64,
    pub retained_records: u64,
    pub derived_records: u64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Inventory {
    pub schema_version: u32,
    pub status: String,
    pub decoder_policy: String,
    pub archive: Archive,
    pub complete_read: bool,
    pub admission_required: bool,
    pub units: Units,
    pub unit_resolution: Resolutions,
    pub components: Vec<Component>,
    pub objects: Vec<Object>,
    pub layers: Vec<Layer>,
    pub definitions: Vec<Definition>,
    pub settings: IgnoredAny,
    pub user_tables: Vec<IgnoredAny>,
    pub table_receipts: Vec<Table>,
    pub diagnostics: Vec<String>,
}
fn resolved(
    original: &OriginalUnits,
    chosen: &ResolvedUnit,
    requested: Option<f64>,
) -> Result<NativeUnits> {
    let factor = chosen.millimeters_per_unit;
    if chosen.original_native_enum != original.native_enum
        || original.requires_resolution != matches!(original.native_enum, 0 | 255)
        || !factor.is_finite()
        || factor <= 0.
        || original.name.is_empty()
        || !original.absolute_tolerance.is_finite()
        || original.absolute_tolerance <= 0.
        || !original.angle_tolerance_radians.is_finite()
        || original.angle_tolerance_radians < 0.
        || !original.relative_tolerance.is_finite()
        || original.relative_tolerance < 0.
        || !(original.absolute_tolerance * factor).is_finite()
        || original.absolute_tolerance * factor <= 0.
    {
        return Err(invalid());
    }
    let by = if original.requires_resolution {
        if original.millimeters_per_unit.is_some()
            || chosen.resolved_by != "explicit"
            || requested != Some(factor)
        {
            return Err(invalid());
        }
        ResolvedBy::Explicit
    } else {
        if original.millimeters_per_unit != Some(factor)
            || chosen.resolved_by != "declared"
            || requested.is_some_and(|r| r != factor)
            || !original.meters_per_unit.is_some_and(|f| {
                f.is_finite() && f > 0. && (f * 1000. - factor).abs() <= factor * 1e-12
            })
        {
            return Err(invalid());
        }
        ResolvedBy::Declared
    };
    Ok(NativeUnits {
        native_enum: original.native_enum,
        name: original.name.clone(),
        declared_mm_per_unit: original.millimeters_per_unit,
        millimeters_per_unit: factor,
        resolved_by: by,
        absolute_tolerance: original.absolute_tolerance,
        angular_tolerance_radians: original.angle_tolerance_radians,
        relative_tolerance: original.relative_tolerance,
    })
}
impl Inventory {
    pub fn decode(bytes: &[u8], l: &Limits) -> Result<Self> {
        om9_archive::admit_json(bytes, l)?;
        let value: Self = serde_json::from_slice(bytes).map_err(|_| invalid())?;
        if value.schema_version != 3
            || value.status != "source_projected"
            || value.decoder_policy != "om9-source-projection-v1"
            || !value.complete_read
            || !value.admission_required
            || value.archive.index_semantics != "decoded_native"
            || value.archive.version == 0
            || value.archive.opennurbs_version == 0
            || value.archive.file_bytes != value.archive.endmark_bytes
            || value.table_receipts.is_empty()
            || value
                .table_receipts
                .iter()
                .any(|t| t.crc_errors != 0 || t.critical_errors != 0 || t.recoverable_errors != 0)
        {
            return Err(invalid());
        }
        Ok(value)
    }
    pub fn source_units(&self, model: Option<f64>, page: Option<f64>) -> Result<SourceUnits> {
        Ok(SourceUnits {
            model: resolved(&self.units.model, &self.unit_resolution.model, model)?,
            page: resolved(&self.units.page, &self.unit_resolution.page, page)?,
            definitions: BTreeMap::new(),
        })
    }
    pub fn definition_units(
        &self,
        def: &Definition,
        requested: Option<f64>,
    ) -> Result<om9_core::retained::DefinitionUnits> {
        let original = &def.units;
        let chosen = self
            .unit_resolution
            .definitions
            .get(&def.id.to_string())
            .ok_or_else(invalid)?;
        if original.name.is_empty()
            || original.requires_resolution != matches!(original.native_enum, 0 | 255)
            || original.requires_resolution != original.millimeters_per_unit.is_none()
        {
            return Err(invalid());
        }
        let (factor, by) = match chosen {
            Some(chosen) => {
                let f = chosen.millimeters_per_unit;
                if !f.is_finite() || f <= 0. || chosen.original_native_enum != original.native_enum
                {
                    return Err(invalid());
                }
                let by = if original.requires_resolution {
                    if chosen.resolved_by != "explicit"
                        || requested != Some(f)
                        || original.meters_per_unit.is_some()
                    {
                        return Err(invalid());
                    }
                    ResolvedBy::Explicit
                } else {
                    if chosen.resolved_by != "declared"
                        || original.millimeters_per_unit != Some(f)
                        || requested.is_some_and(|r| r != f)
                        || !original.meters_per_unit.is_some_and(|m| {
                            m.is_finite() && m > 0. && (m * 1000. - f).abs() <= f * 1e-12
                        })
                    {
                        return Err(invalid());
                    }
                    ResolvedBy::Declared
                };
                (Some(f), Some(by))
            }
            None => {
                if !original.requires_resolution
                    || requested.is_some()
                    || original.meters_per_unit.is_some()
                {
                    return Err(invalid());
                }
                (None, None)
            }
        };
        Ok(om9_core::retained::DefinitionUnits {
            native_enum: original.native_enum,
            name: original.name.clone(),
            declared_mm_per_unit: original.millimeters_per_unit,
            millimeters_per_unit: factor,
            resolved_by: by,
        })
    }
    pub fn owned_usage(&self, l: &Limits) -> Result<Usage> {
        let mut u = Usage::array::<Self>(1)?;
        let strings = [
            &self.status,
            &self.decoder_policy,
            &self.archive.index_semantics,
            &self.archive.source_sha256,
            &self.units.model.name,
            &self.units.page.name,
            &self.unit_resolution.model.resolved_by,
            &self.unit_resolution.page.resolved_by,
        ];
        for s in strings {
            u.add(string_usage(s)?, l)?;
        }
        u.add(
            Usage::btree::<String, Option<ResolvedUnit>>(self.unit_resolution.definitions.len())?,
            l,
        )?;
        for (s, r) in &self.unit_resolution.definitions {
            u.add(string_usage(s)?, l)?;
            if let Some(r) = r {
                u.add(string_usage(&r.resolved_by)?, l)?;
            }
        }
        for v in [
            Usage::array::<Component>(self.components.capacity())?,
            Usage::array::<Object>(self.objects.capacity())?,
            Usage::array::<Layer>(self.layers.capacity())?,
            Usage::array::<Definition>(self.definitions.capacity())?,
            Usage::array::<Table>(self.table_receipts.capacity())?,
        ] {
            u.add(v, l)?;
        }
        for c in &self.components {
            for s in [
                &c.component_type,
                &c.native_kind,
                &c.native_class_uuid,
                &c.name,
                &c.capability,
            ] {
                u.add(string_usage(s)?, l)?;
            }
            u.add(relations_usage(&c.relations, l)?, l)?;
            if let Some(p) = &c.preservation {
                u.add(preservation_usage(p, l)?, l)?;
            }
            if let Some(o) = &c.light {
                u.add(Usage::array::<Object>(1)?, l)?;
                u.add(object_usage(o, l)?, l)?;
            }
        }
        for o in &self.objects {
            u.add(object_usage(o, l)?, l)?;
        }
        for layer in &self.layers {
            u.add(string_usage(&layer.name)?, l)?;
            u.add(relations_usage(&layer.relations, l)?, l)?;
            u.add(preservation_usage(&layer.preservation, l)?, l)?;
        }
        for d in &self.definitions {
            u.add(string_usage(&d.label)?, l)?;
            u.add(string_usage(&d.units.name)?, l)?;
            u.add(Usage::array::<Uuid>(d.members.capacity())?, l)?;
            u.add(relations_usage(&d.relations, l)?, l)?;
            u.add(preservation_usage(&d.preservation, l)?, l)?;
        }
        for t in &self.table_receipts {
            u.add(string_usage(&t.table)?, l)?;
        }
        u.add(strings_usage(&self.diagnostics, l)?, l)?;
        Ok(u)
    }
}
fn object_usage(o: &Object, l: &Limits) -> Result<Usage> {
    let mut u = Usage::default();
    for s in [
        &o.label,
        &o.native_kind,
        &o.native_class_uuid,
        &o.capability,
    ] {
        u.add(string_usage(s)?, l)?;
    }
    for s in [&o.component_type, &o.projection_units_basis]
        .into_iter()
        .flatten()
    {
        u.add(string_usage(s)?, l)?;
    }
    u.add(relations_usage(&o.relations, l)?, l)?;
    u.add(preservation_usage(&o.preservation, l)?, l)?;
    u.add(Usage::array::<i32>(o.attributes.groups.capacity())?, l)?;
    for a in [
        &o.diagnostics,
        &o.unrepresented_attributes,
        &o.unsupported_fields,
    ] {
        u.add(strings_usage(a, l)?, l)?;
    }
    if let Some(g) = &o.geometry {
        u.add(g.validate(l)?, l)?;
    }
    Ok(u)
}
fn string_usage(s: &String) -> Result<Usage> {
    Ok(Usage {
        bytes: s.capacity() as u64,
        ..Usage::default()
    })
}
fn strings_usage(a: &Vec<String>, l: &Limits) -> Result<Usage> {
    let mut u = Usage::array::<String>(a.capacity())?;
    for s in a {
        u.add(string_usage(s)?, l)?;
    }
    Ok(u)
}
fn relations_usage(a: &Vec<Relation>, l: &Limits) -> Result<Usage> {
    let mut u = Usage::array::<Relation>(a.capacity())?;
    for r in a {
        for s in [&r.component_type, &r.role, &r.scope] {
            u.add(string_usage(s)?, l)?;
        }
    }
    Ok(u)
}
fn preservation_usage(p: &Preservation, l: &Limits) -> Result<Usage> {
    let mut u = Usage::default();
    for a in [&p.default_fields, &p.nondefault_fields] {
        u.add(Usage::btree::<String, Value>(a.len())?, l)?;
        for (s, v) in a {
            u.add(string_usage(s)?, l)?;
            u.add(value_usage(v, l)?, l)?;
        }
    }
    Ok(u)
}
pub(crate) fn key(ty: u32, id: Uuid, index: Option<i32>) -> Result<NativeRecordKey> {
    if index.is_some_and(|i| i < 0) || (id.is_nil() && index.is_none()) {
        return Err(invalid());
    }
    Ok(NativeRecordKey {
        kind: kind(ty)?,
        id,
        native_index: index,
    })
}
// Diagnostic preservation field values remain owned until admission/conversion.
pub(crate) fn value_usage(value: &Value, l: &Limits) -> Result<Usage> {
    let mut u = Usage::array::<Value>(1)?;
    match value {
        Value::String(s) => u.add(
            Usage {
                bytes: s.capacity() as u64,
                ..Usage::default()
            },
            l,
        )?,
        Value::Array(a) => {
            u.add(Usage::array::<Value>(a.capacity())?, l)?;
            for v in a {
                u.add(value_usage(v, l)?, l)?;
            }
        }
        Value::Object(a) => {
            u.add(Usage::btree::<String, Value>(a.len())?, l)?;
            for (s, v) in a {
                u.add(
                    Usage {
                        bytes: s.capacity() as u64,
                        ..Usage::default()
                    },
                    l,
                )?;
                u.add(value_usage(v, l)?, l)?;
            }
        }
        _ => {}
    }
    Ok(u)
}
