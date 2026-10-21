// SPDX-License-Identifier: LGPL-2.1-or-later
//! Cold correspondence; native process and publication authority stay outside.
use crate::{
    HostValue, OutputWireIdentity, PreparedNativeUnits, RecordAction, ScopedNativeKey,
    WireOutputPlan,
};
use om9_core::{
    Error, Result,
    budget::{Limits, Usage},
    geometry::{Curve, Geometry, Transform, Unit},
    retained::HostRef,
    transaction::Cancellation,
};
use serde::{
    Deserialize,
    de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor},
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    fs::File,
    io::{Read, Seek, SeekFrom},
};
use uuid::Uuid;
#[derive(Debug, Clone, Copy)]
pub struct ExpectedRetainedOutput {
    pub request_sha256: [u8; 32],
    pub output_sha256: [u8; 32],
    pub output_bytes: u64,
}
#[derive(Debug)]
pub struct ColdRetainedProof {
    request_sha256: [u8; 32],
    output_sha256: [u8; 32],
    output_bytes: u64,
    live_records: u64,
    represented_bounds: u64,
    native_only_bounds: u64,
}
impl ColdRetainedProof {
    pub fn request_sha256(&self) -> [u8; 32] {
        self.request_sha256
    }
    pub fn output_sha256(&self) -> [u8; 32] {
        self.output_sha256
    }
    pub fn output_bytes(&self) -> u64 {
        self.output_bytes
    }
    pub fn live_records(&self) -> u64 {
        self.live_records
    }
    pub fn represented_bounds_checked(&self) -> u64 {
        self.represented_bounds
    }
    /// Pinned native conservative-hull checks, never independent tight BRep bounds.
    pub fn native_only_bounds_checked(&self) -> u64 {
        self.native_only_bounds
    }
    pub fn owned_usage(&self) -> Usage {
        Usage {
            bytes: std::mem::size_of::<Self>() as u64,
            ..Usage::default()
        }
    }
}
fn invalid() -> Error {
    Error::Invalid("cold retained correspondence")
}
fn option<'de, D: de::Deserializer<'de>, T: Deserialize<'de>>(
    d: D,
) -> std::result::Result<Option<T>, D::Error> {
    Option::<T>::deserialize(d)
}
fn optional_uuid<'de, D: de::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Option<Uuid>, D::Error> {
    #[derive(Deserialize)]
    struct Canonical(#[serde(deserialize_with = "crate::import_dto::uuid")] Uuid);
    Option::<Canonical>::deserialize(d).map(|v| v.map(|v| v.0))
}
fn uuid_list<'de, D: de::Deserializer<'de>>(d: D) -> std::result::Result<Vec<Uuid>, D::Error> {
    #[derive(Deserialize)]
    struct Canonical(#[serde(deserialize_with = "crate::import_dto::uuid")] Uuid);
    Vec::<Canonical>::deserialize(d).map(|v| v.into_iter().map(|v| v.0).collect())
}
fn sha<'de, D: de::Deserializer<'de>>(d: D) -> std::result::Result<[u8; 32], D::Error> {
    struct H;
    impl<'de> Visitor<'de> for H {
        type Value = [u8; 32];
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("canonical SHA256")
        }
        fn visit_str<E: de::Error>(self, s: &str) -> std::result::Result<Self::Value, E> {
            if s.len() != 64 {
                return Err(E::custom("SHA256 length"));
            }
            let mut out = [0; 32];
            for (x, pair) in out.iter_mut().zip(s.as_bytes().as_chunks::<2>().0) {
                let digit = |b| match b {
                    b'0'..=b'9' => Some(b - b'0'),
                    b'a'..=b'f' => Some(b - b'a' + 10),
                    _ => None,
                };
                *x = digit(pair[0])
                    .zip(digit(pair[1]))
                    .map(|(a, b)| a * 16 + b)
                    .ok_or_else(|| E::custom("SHA256 spelling"))?;
            }
            Ok(out)
        }
    }
    d.deserialize_str(H)
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Report {
    schema_version: u32,
    writer_policy: String,
    bounds_policy: String,
    status: String,
    archive_version: u32,
    objects: u64,
    definitions: u64,
    layers: u64,
    #[serde(deserialize_with = "option_sha")]
    primary_source_sha256: Option<[u8; 32]>,
    cold_validation_required: bool,
    admission_required: bool,
    host_ready: bool,
    #[serde(deserialize_with = "sha")]
    request_sha256: [u8; 32],
    #[serde(deserialize_with = "sha")]
    output_sha256: [u8; 32],
    output_bytes: u64,
    checks: Checks,
    actual: Actual,
    raw_spans: Vec<RawSpan>,
}
fn option_sha<'de, D: de::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Option<[u8; 32]>, D::Error> {
    #[derive(Deserialize)]
    struct S(#[serde(deserialize_with = "sha")] [u8; 32]);
    Option::<S>::deserialize(d).map(|v| v.map(|v| v.0))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Checks {
    live_records: u64,
    raw_records: u64,
    raw_tables: u64,
    opaque_chunks: u64,
    bounds: u64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Actual {
    model_units: PreparedNativeUnits,
    page_units: PreparedNativeUnits,
    current_layer: CurrentLayer,
    objects: Vec<Object>,
    layers: Vec<Layer>,
    definitions: Vec<Definition>,
    bounds: Vec<Bound>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CurrentLayer {
    #[serde(deserialize_with = "optional_uuid")]
    id: Option<Uuid>,
    #[serde(deserialize_with = "option")]
    archive_reference_index: Option<i32>,
    #[serde(deserialize_with = "option")]
    sdk_destination_index: Option<i32>,
}
#[derive(Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct Wire {
    #[serde(deserialize_with = "option")]
    serialized_index: Option<i32>,
    #[serde(deserialize_with = "option")]
    archive_reference_index: Option<i32>,
    #[serde(deserialize_with = "option")]
    sdk_destination_index: Option<i32>,
}
impl Wire {
    fn matches(&self, o: OutputWireIdentity) -> bool {
        (
            self.serialized_index,
            self.archive_reference_index,
            self.sdk_destination_index,
        ) == (
            o.serialized_index,
            o.archive_reference_index,
            o.sdk_destination_index,
        )
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Attributes {
    #[serde(deserialize_with = "optional_uuid")]
    layer: Option<Uuid>,
    visible: bool,
    locked: bool,
    #[serde(deserialize_with = "option")]
    color: Option<[u8; 4]>,
    color_source: u32,
    material_source: u32,
    line_pattern_source: u32,
    mode: u32,
    native_layer_index: i32,
    native_material_index: i32,
    native_line_pattern_index: i32,
    groups: Vec<i32>,
    #[serde(deserialize_with = "optional_uuid")]
    viewport_id: Option<Uuid>,
    space: u32,
    display_order: i32,
    plot_weight_mm: f64,
    userdata: Vec<Value>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Object {
    #[serde(deserialize_with = "crate::import_dto::uuid")]
    id: Uuid,
    native_component_type: u32,
    native_kind: String,
    #[serde(deserialize_with = "crate::import_dto::uuid")]
    native_class_uuid: Uuid,
    wire: Wire,
    label: String,
    is_definition_member: bool,
    #[serde(deserialize_with = "option")]
    geometry: Option<Value>,
    placement: Transform,
    attributes: Attributes,
    preservation: Value,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Layer {
    #[serde(deserialize_with = "crate::import_dto::uuid")]
    id: Uuid,
    native_component_type: u32,
    native_kind: String,
    #[serde(deserialize_with = "crate::import_dto::uuid")]
    native_class_uuid: Uuid,
    wire: Wire,
    name: String,
    #[serde(deserialize_with = "optional_uuid")]
    parent: Option<Uuid>,
    visible: bool,
    locked: bool,
    color: [u8; 4],
    preservation: Value,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Definition {
    #[serde(deserialize_with = "crate::import_dto::uuid")]
    id: Uuid,
    native_component_type: u32,
    native_kind: String,
    #[serde(deserialize_with = "crate::import_dto::uuid")]
    native_class_uuid: Uuid,
    wire: Wire,
    label: String,
    #[serde(deserialize_with = "uuid_list")]
    members: Vec<Uuid>,
    units: DefinitionUnits,
    linked: bool,
    preservation: Value,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DefinitionUnits {
    native_enum: u32,
    name: String,
    #[serde(deserialize_with = "option")]
    millimeters_per_unit: Option<f64>,
    #[serde(deserialize_with = "option")]
    meters_per_unit: Option<f64>,
    requires_resolution: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Bound {
    #[serde(deserialize_with = "crate::import_dto::uuid")]
    id: Uuid,
    native_component_type: u32,
    stored: [[f64; 3]; 2],
    computed: [[f64; 3]; 2],
    matched: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSpan {
    kind: String,
    #[serde(deserialize_with = "option")]
    native_component_type: Option<u32>,
    #[serde(deserialize_with = "optional_uuid")]
    id: Option<Uuid>,
    table_type: u32,
    #[serde(deserialize_with = "option")]
    record_type: Option<u32>,
    offset: u64,
    byte_length: u64,
    #[serde(deserialize_with = "sha")]
    sha256: [u8; 32],
}

// A nonallocating second structural pass admits DOM/map growth and typed row
// buffers before serde creates them. Shared admission has already rejected
// duplicate keys. String unescaping scratch remains charged throughout.
struct Admission<'a> {
    l: &'a Limits,
    u: Usage,
    error: Option<Error>,
    check: &'a dyn Fn() -> Result<()>,
}
impl Admission<'_> {
    fn add<E: de::Error>(&mut self, u: Usage) -> std::result::Result<(), E> {
        if let Err(e) = (self.check)() {
            self.error = Some(e);
            return Err(E::custom("cold canceled"));
        }
        self.u.add(u, self.l).map_err(|e| {
            self.error = Some(e);
            E::custom("cold allocation admission")
        })
    }
}
struct Node<'a, 'b> {
    a: &'a mut Admission<'b>,
    depth: usize,
}
struct Text<'a, 'b>(&'a mut Admission<'b>);
impl<'de> DeserializeSeed<'de> for Text<'_, '_> {
    type Value = ();
    fn deserialize<D: de::Deserializer<'de>>(self, d: D) -> std::result::Result<(), D::Error> {
        d.deserialize_str(self)
    }
}
impl<'de> Visitor<'de> for Text<'_, '_> {
    type Value = ();
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("bounded cold string")
    }
    fn visit_str<E: de::Error>(self, s: &str) -> std::result::Result<(), E> {
        if let Err(e) = om9_core::budget::name(s, self.0.l) {
            self.0.error = Some(e);
            return Err(E::custom("cold string admission"));
        }
        let n = (s.len() as u64)
            .checked_mul(3)
            .ok_or_else(|| E::custom("cold string overflow"))?;
        self.0.add(Usage {
            bytes: n + 32,
            ..Usage::default()
        })
    }
}
impl<'de> DeserializeSeed<'de> for Node<'_, '_> {
    type Value = ();
    fn deserialize<D: de::Deserializer<'de>>(self, d: D) -> std::result::Result<(), D::Error> {
        if self.depth > self.a.l.max_depth {
            self.a.error = Some(Error::Budget("cold nesting"));
            return Err(de::Error::custom("cold nesting"));
        }
        self.a.add::<D::Error>(Usage {
            bytes: 256,
            nodes: 1,
            ..Usage::default()
        })?;
        d.deserialize_any(self)
    }
}
impl<'de> Visitor<'de> for Node<'_, '_> {
    type Value = ();
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("bounded cold value")
    }
    fn visit_bool<E: de::Error>(self, _: bool) -> std::result::Result<(), E> {
        Ok(())
    }
    fn visit_i64<E: de::Error>(self, _: i64) -> std::result::Result<(), E> {
        Ok(())
    }
    fn visit_u64<E: de::Error>(self, _: u64) -> std::result::Result<(), E> {
        Ok(())
    }
    fn visit_f64<E: de::Error>(self, n: f64) -> std::result::Result<(), E> {
        if n.is_finite() {
            Ok(())
        } else {
            Err(E::custom("nonfinite cold value"))
        }
    }
    fn visit_unit<E: de::Error>(self) -> std::result::Result<(), E> {
        Ok(())
    }
    fn visit_str<E: de::Error>(self, s: &str) -> std::result::Result<(), E> {
        Text(self.a).visit_str(s)
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> std::result::Result<(), A::Error> {
        while seq
            .next_element_seed(Node {
                a: self.a,
                depth: self.depth + 1,
            })?
            .is_some()
        {}
        Ok(())
    }
    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> std::result::Result<(), A::Error> {
        let mut count = 0usize;
        let mut prior = Usage::btree::<String, Value>(0)
            .map_err(de::Error::custom)?
            .bytes;
        self.a.add::<A::Error>(Usage {
            bytes: prior,
            ..Usage::default()
        })?;
        while map.next_key_seed(Text(self.a))?.is_some() {
            count = count
                .checked_add(1)
                .ok_or_else(|| de::Error::custom("cold map count"))?;
            let now = Usage::btree::<String, Value>(count)
                .map_err(de::Error::custom)?
                .bytes;
            self.a.add::<A::Error>(Usage {
                bytes: now - prior,
                ..Usage::default()
            })?;
            prior = now;
            map.next_value_seed(Node {
                a: self.a,
                depth: self.depth + 1,
            })?;
        }
        Ok(())
    }
}
fn preflight(bytes: &[u8], l: &Limits, check: &dyn Fn() -> Result<()>) -> Result<()> {
    let n = (bytes.len() as u64)
        .checked_mul(3)
        .ok_or(Error::Overflow("cold JSON input"))?;
    let mut a = Admission {
        l,
        u: Usage {
            bytes: n,
            ..Usage::default()
        },
        error: None,
        check,
    };
    a.u.check(l)?;
    let mut d = serde_json::Deserializer::from_slice(bytes);
    let result = Node {
        a: &mut a,
        depth: 0,
    }
    .deserialize(&mut d)
    .and_then(|_| d.end());
    if result.is_err() {
        return Err(a.error.unwrap_or_else(invalid));
    }
    Ok(())
}
fn value_usage(v: &Value, l: &Limits) -> Result<Usage> {
    let mut u = Usage::default();
    match v {
        Value::String(s) => u.bytes = s.capacity() as u64,
        Value::Array(a) => {
            u.add(Usage::array::<Value>(a.capacity())?, l)?;
            for v in a {
                u.add(value_usage(v, l)?, l)?;
            }
        }
        Value::Object(m) => {
            u.add(Usage::btree::<String, Value>(m.len())?, l)?;
            for (k, v) in m {
                u.add(
                    Usage {
                        bytes: k.capacity() as u64,
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
fn report_usage(r: &Report, l: &Limits) -> Result<Usage> {
    let mut u = Usage::array::<Report>(1)?;
    for s in [
        &r.writer_policy,
        &r.bounds_policy,
        &r.status,
        &r.actual.model_units.name,
        &r.actual.page_units.name,
    ] {
        u.add(
            Usage {
                bytes: s.capacity() as u64,
                ..Usage::default()
            },
            l,
        )?;
    }
    for part in [
        Usage::array::<Object>(r.actual.objects.capacity())?,
        Usage::array::<Layer>(r.actual.layers.capacity())?,
        Usage::array::<Definition>(r.actual.definitions.capacity())?,
        Usage::array::<Bound>(r.actual.bounds.capacity())?,
        Usage::array::<RawSpan>(r.raw_spans.capacity())?,
    ] {
        u.add(part, l)?;
    }
    for o in &r.actual.objects {
        u.add(
            Usage {
                bytes: (o.native_kind.capacity() + o.label.capacity()) as u64,
                ..Usage::default()
            },
            l,
        )?;
        u.add(Usage::array::<i32>(o.attributes.groups.capacity())?, l)?;
        u.add(Usage::array::<Value>(o.attributes.userdata.capacity())?, l)?;
        for v in &o.attributes.userdata {
            u.add(value_usage(v, l)?, l)?;
        }
        if let Some(v) = &o.geometry {
            u.add(value_usage(v, l)?, l)?;
        }
        u.add(value_usage(&o.preservation, l)?, l)?;
    }
    for v in &r.actual.layers {
        u.add(
            Usage {
                bytes: (v.native_kind.capacity() + v.name.capacity()) as u64,
                ..Usage::default()
            },
            l,
        )?;
        u.add(value_usage(&v.preservation, l)?, l)?;
    }
    for v in &r.actual.definitions {
        u.add(
            Usage {
                bytes: (v.native_kind.capacity() + v.label.capacity() + v.units.name.capacity())
                    as u64,
                ..Usage::default()
            },
            l,
        )?;
        u.add(Usage::array::<Uuid>(v.members.capacity())?, l)?;
        u.add(value_usage(&v.preservation, l)?, l)?;
    }
    for v in &r.raw_spans {
        u.add(
            Usage {
                bytes: v.kind.capacity() as u64,
                ..Usage::default()
            },
            l,
        )?;
    }
    Ok(u)
}
#[derive(Clone, Copy)]
enum Origin<'a> {
    Source {
        ns: &'a crate::NamespaceOverlay<'a>,
        row: &'a crate::RecordOverlay<'a>,
        original: &'a crate::PreparedSourceRecord,
    },
    Addition(HostValue<'a>),
    Default,
}
#[derive(Clone, Copy)]
struct Expected<'a> {
    output: OutputWireIdentity,
    origin: Origin<'a>,
    refresh: bool,
}
impl Expected<'_> {
    fn current(&self) -> Option<HostValue<'_>> {
        match self.origin {
            Origin::Source { row, .. } => row.current,
            Origin::Addition(v) => Some(v),
            Origin::Default => None,
        }
    }
}
fn expected_rows<'a>(
    w: &'a WireOutputPlan<'a>,
    cancel: &Cancellation,
) -> Result<BTreeMap<Uuid, Expected<'a>>> {
    let mut m = BTreeMap::new();
    for ns in &w.inputs().plan().overlay.namespaces {
        let p = w.inputs().source(ns.namespace.id).ok_or_else(invalid)?;
        for row in &ns.records {
            w.inputs().check_cancel(cancel)?;
            let key = ScopedNativeKey {
                namespace: ns.namespace.id,
                key: row.source.key,
            };
            if let Some(output) = w.source(key).ok_or_else(invalid)? {
                let original = p.record(row.source.key).ok_or_else(invalid)?;
                if output.id.is_nil()
                    || m.insert(
                        output.id,
                        Expected {
                            output,
                            origin: Origin::Source { ns, row, original },
                            refresh: row.needs_cache_refresh,
                        },
                    )
                    .is_some()
                {
                    return Err(invalid());
                }
            }
        }
    }
    for v in &w.inputs().plan().overlay.additions {
        w.inputs().check_cancel(cancel)?;
        let output = w.host(v.host()).ok_or_else(invalid)?;
        let refresh = matches!(v, HostValue::Definition(_))
            || matches!(
                v,
                HostValue::Entity {
                    entity: om9_core::model::Entity {
                        geometry: Geometry::Instance(_),
                        ..
                    },
                    ..
                }
            );
        if output.id.is_nil()
            || m.insert(
                output.id,
                Expected {
                    output,
                    origin: Origin::Addition(*v),
                    refresh,
                },
            )
            .is_some()
        {
            return Err(invalid());
        }
    }
    if let Some(output) = w.default_layer()
        && m.insert(
            output.id,
            Expected {
                output,
                origin: Origin::Default,
                refresh: false,
            },
        )
        .is_some()
    {
        return Err(invalid());
    }

    Ok(m)
}
fn equal(a: f64, b: f64) -> bool {
    a.is_finite() && b.is_finite() && (a - b).abs() <= 1e-9 * (1. + b.abs())
}
fn matrix(a: Transform, b: Transform) -> bool {
    a.0.iter()
        .flatten()
        .zip(b.0.iter().flatten())
        .all(|(a, b)| equal(*a, *b))
}
fn fields(v: &Value, names: &[&str]) -> Result<()> {
    let m = v.as_object().ok_or_else(invalid)?;
    if m.len() != names.len() || names.iter().any(|n| !m.contains_key(*n)) {
        return Err(invalid());
    }
    Ok(())
}
const ENTITY_FIELDS: &[&str] = &[
    "color_source",
    "native_color",
    "url",
    "material_source",
    "native_material_index",
    "line_pattern_source",
    "native_line_pattern_index",
    "groups",
    "viewport_id",
    "space",
    "display_order",
    "plot_color",
    "plot_color_source",
    "plot_weight_mm",
    "plot_weight_source",
    "wire_density",
    "object_decoration",
    "display_mode_overrides",
    "render_material_references",
    "texture_mapping_channels",
    "casts_shadows",
    "receives_shadows",
    "object_frame",
    "section_attributes_source",
    "linetype_scale",
    "custom_linetype_present",
    "custom_section_style_present",
    "hatch_background_fill",
    "hatch_boundary_visible",
    "clipping_plane_label_style",
    "userdata",
];
const LAYER_FIELDS: &[&str] = &[
    "native_material_index",
    "native_line_pattern_index",
    "plot_color",
    "plot_weight_mm",
    "display_material_id",
    "iges_level",
    "expanded",
    "persistent_visibility",
    "persistent_locking",
    "per_viewport_settings",
    "render_material_references",
    "userdata",
];
const DEFINITION_FIELDS: &[&str] = &[
    "description",
    "url",
    "url_tag",
    "native_definition_type",
    "linked_file_path",
    "has_linked_component_settings",
    "skip_nested_linked_definitions",
    "linked_component_appearance",
    "userdata",
];
fn userdata(v: &Value) -> Result<()> {
    for u in v.as_array().ok_or_else(invalid)? {
        userdata_entry(u)?;
    }
    Ok(())
}
fn userdata_entry(u: &Value) -> Result<()> {
    let unknown = u
        .get("unknown")
        .and_then(Value::as_bool)
        .ok_or_else(invalid)?;
    let known = &[
        "class_id",
        "application_id",
        "item_id",
        "unknown",
        "copy_count",
        "native_sizeof_bytes",
        "opaque_references",
    ][..];
    let other = &[
        "class_id",
        "application_id",
        "item_id",
        "unknown",
        "copy_count",
        "native_sizeof_bytes",
        "opaque_references",
        "byte_length",
        "payload_sha1",
        "archive_version",
        "opennurbs_version",
    ][..];
    fields(u, if unknown { other } else { known })?;
    let class = value_uuid(&u["class_id"])?.ok_or_else(invalid)?;
    if class.is_nil()
        || u["copy_count"].as_u64().is_none()
        || u["native_sizeof_bytes"].as_u64().is_none_or(|n| n == 0)
        || u["opaque_references"] != true
    {
        return Err(invalid());
    }
    for k in ["application_id", "item_id"] {
        value_uuid(&u[k])?;
    }
    if unknown {
        let hash = u["payload_sha1"].as_str().ok_or_else(invalid)?;
        if hash.len() != 40
            || !hash
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            || u["byte_length"].as_u64().is_none()
            || u["archive_version"].as_u64().is_none()
            || u["opennurbs_version"].as_u64().is_none()
        {
            return Err(invalid());
        }
    }
    Ok(())
}

fn value_uuid(v: &Value) -> Result<Option<Uuid>> {
    if v.is_null() {
        return Ok(None);
    }
    let s = v.as_str().ok_or_else(invalid)?;
    let id = Uuid::parse_str(s).map_err(|_| invalid())?;
    let mut canonical = [0; 36];
    if id.hyphenated().encode_lower(&mut canonical).as_bytes() != s.as_bytes() {
        return Err(invalid());
    }
    Ok(Some(id))
}
fn rgba(v: &Value) -> Option<[u8; 4]> {
    let a = v.as_array()?;
    if a.len() != 4 {
        return None;
    }
    let mut c = [0; 4];
    for (n, v) in c.iter_mut().zip(a) {
        *n = u8::try_from(v.as_u64()?).ok()?;
    }
    Some(c)
}
fn new_entity_preservation(v: &Value) -> Result<()> {
    for k in ["url"] {
        if v[k] != "" {
            return Err(invalid());
        }
    }
    for k in [
        "material_source",
        "line_pattern_source",
        "display_order",
        "plot_color_source",
        "plot_weight_source",
        "object_decoration",
        "section_attributes_source",
        "clipping_plane_label_style",
    ] {
        if v[k] != 0 {
            return Err(invalid());
        }
    }
    for k in ["native_material_index", "native_line_pattern_index"] {
        if v[k] != -1 {
            return Err(invalid());
        }
    }
    for k in [
        "groups",
        "display_mode_overrides",
        "render_material_references",
        "texture_mapping_channels",
        "userdata",
    ] {
        if v[k].as_array().is_none_or(|a| !a.is_empty()) {
            return Err(invalid());
        }
    }
    for k in ["casts_shadows", "receives_shadows"] {
        if v[k] != true {
            return Err(invalid());
        }
    }
    for k in [
        "custom_linetype_present",
        "custom_section_style_present",
        "hatch_boundary_visible",
    ] {
        if v[k] != false {
            return Err(invalid());
        }
    }
    if !v["viewport_id"].is_null()
        || !v["object_frame"].is_null()
        || v["space"] != 1
        || v["wire_density"] != 1
        || v["linetype_scale"] != 1.
        || v["plot_weight_mm"] != 0.
        || rgba(&v["plot_color"]) != Some([0, 0, 0, 255])
        || rgba(&v["hatch_background_fill"]) != Some([255, 255, 255, 0])
    {
        return Err(invalid());
    }
    Ok(())
}

fn new_layer_preservation(v: &Value, visible: bool, locked: bool) -> Result<()> {
    if v["native_material_index"] != -1
        || v["native_line_pattern_index"] != -1
        || rgba(&v["plot_color"]) != Some([255, 255, 255, 0])
        || v["plot_weight_mm"] != 0.
        || !v["display_material_id"].is_null()
        || v["iges_level"] != -1
        || v["expanded"] != true
        || v["persistent_visibility"] != visible
        || v["persistent_locking"] != locked
        || v["per_viewport_settings"]["present"] != false
        || v["per_viewport_settings"]["crc"] != 0
        || v["render_material_references"]
            .as_array()
            .is_none_or(|a| !a.is_empty())
        || v["userdata"].as_array().is_none_or(|a| !a.is_empty())
    {
        return Err(invalid());
    }
    Ok(())
}
fn new_definition_preservation(v: &Value) -> Result<()> {
    if ["description", "url", "url_tag", "linked_file_path"]
        .iter()
        .any(|k| v[*k] != "")
        || v["native_definition_type"] != 1
        || v["has_linked_component_settings"] != false
        || v["skip_nested_linked_definitions"] != false
        || v["linked_component_appearance"] != 0
        || v["userdata"].as_array().is_none_or(|a| !a.is_empty())
    {
        return Err(invalid());
    }
    Ok(())
}
fn rich(v: &Value, names: &[&str]) -> Result<()> {
    fields(v, names)?;
    userdata(&v["userdata"])?;
    for (k, v) in v.as_object().ok_or_else(invalid)? {
        let good = match k.as_str() {
            "userdata" => true,
            "url" | "description" | "url_tag" | "linked_file_path" => v.is_string(),
            "expanded"
            | "persistent_visibility"
            | "persistent_locking"
            | "casts_shadows"
            | "receives_shadows"
            | "custom_linetype_present"
            | "custom_section_style_present"
            | "hatch_boundary_visible"
            | "has_linked_component_settings"
            | "skip_nested_linked_definitions" => v.is_boolean(),
            "native_color" | "plot_color" | "hatch_background_fill" => {
                v.as_array().is_some_and(|a| {
                    a.len() == 4 && a.iter().all(|n| n.as_u64().is_some_and(|n| n <= 255))
                })
            }
            "groups" => v
                .as_array()
                .is_some_and(|a| a.iter().all(|n| n.as_i64().is_some())),
            "display_material_id" | "viewport_id" => {
                value_uuid(v)?;
                true
            }
            "per_viewport_settings" => {
                fields(v, &["present", "crc"])?;
                v["present"].is_boolean() && v["crc"].as_u64().is_some()
            }
            "object_frame" => {
                if v.is_null() {
                    true
                } else {
                    fields(v, &["origin", "x", "y"])?;
                    ["origin", "x", "y"].iter().all(|k| {
                        v[k].as_array().is_some_and(|a| {
                            a.len() == 3 && a.iter().all(|n| n.as_f64().is_some_and(f64::is_finite))
                        })
                    })
                }
            }
            "display_mode_overrides" => {
                for entry in v.as_array().ok_or_else(invalid)? {
                    fields(entry, &["viewport_id", "display_material_id"])?;
                    value_uuid(&entry["viewport_id"])?;
                    value_uuid(&entry["display_material_id"])?;
                }
                true
            }
            "render_material_references" => {
                for entry in v.as_array().ok_or_else(invalid)? {
                    fields(
                        entry,
                        &[
                            "role",
                            "component_type",
                            "id",
                            "native_index",
                            "application_id",
                        ],
                    )?;
                    if !matches!(
                        entry["role"].as_str(),
                        Some("render_material" | "backface_material")
                    ) || entry["component_type"].as_u64() != Some(3)
                        || value_uuid(&entry["id"])?.is_none_or(|id| id.is_nil())
                        || (!entry["native_index"].is_null()
                            && entry["native_index"].as_i64().is_none())
                    {
                        return Err(invalid());
                    }
                    value_uuid(&entry["application_id"])?;
                }
                true
            }
            "texture_mapping_channels" => {
                for entry in v.as_array().ok_or_else(invalid)? {
                    fields(
                        entry,
                        &["application_id", "channel_id", "mapping_id", "transform"],
                    )?;
                    value_uuid(&entry["application_id"])?;
                    value_uuid(&entry["mapping_id"])?;
                    let rows = entry["transform"].as_array().ok_or_else(invalid)?;
                    if entry["channel_id"].as_i64().is_none()
                        || rows.len() != 4
                        || rows.iter().any(|r| {
                            r.as_array().is_none_or(|r| {
                                r.len() != 4
                                    || r.iter().any(|x| x.as_f64().is_none_or(|x| !x.is_finite()))
                            })
                        })
                    {
                        return Err(invalid());
                    }
                }
                true
            }
            "plot_weight_mm" | "linetype_scale" => v.as_f64().is_some_and(f64::is_finite),
            _ => v.as_i64().is_some(),
        };
        if !good {
            return Err(invalid());
        }
    }
    Ok(())
}
fn units_valid(u: &PreparedNativeUnits) -> Result<()> {
    if !u.absolute_tolerance.is_finite()
        || u.absolute_tolerance <= 0.
        || !u.angle_tolerance_radians.is_finite()
        || u.angle_tolerance_radians <= 0.
        || u.angle_tolerance_radians > std::f64::consts::PI
        || !u.relative_tolerance.is_finite()
        || u.relative_tolerance <= 0.
        || u.relative_tolerance >= 1.
        || u.name.is_empty()
    {
        return Err(invalid());
    }
    match (
        u.requires_resolution,
        u.millimeters_per_unit,
        u.meters_per_unit,
    ) {
        (true, None, None) if matches!(u.native_enum, 0 | 255) => {}
        (false, Some(mm), Some(m)) if mm > 0. && m > 0. && equal(mm, m * 1000.) => {}
        _ => return Err(invalid()),
    }
    Ok(())
}
fn units_equal(a: &PreparedNativeUnits, b: &PreparedNativeUnits) -> bool {
    a.native_enum == b.native_enum
        && a.name == b.name
        && a.millimeters_per_unit == b.millimeters_per_unit
        && a.meters_per_unit == b.meters_per_unit
        && a.requires_resolution == b.requires_resolution
        && a.absolute_tolerance == b.absolute_tolerance
        && a.angle_tolerance_radians == b.angle_tolerance_radians
        && a.relative_tolerance == b.relative_tolerance
        && a.distance_display_mode == b.distance_display_mode
        && a.distance_display_precision == b.distance_display_precision
}
fn declared(u: &PreparedNativeUnits, unit: Unit) -> bool {
    let n = match unit {
        Unit::Millimeter => 2,
        Unit::Centimeter => 3,
        Unit::Meter => 4,
        Unit::Inch => 8,
        Unit::Foot => 9,
        Unit::Micrometer => 1,
    };
    u.native_enum == n
        && !u.requires_resolution
        && u.millimeters_per_unit
            .is_some_and(|v| equal(v, unit.factor()))
}
fn geometry_class(g: &Geometry) -> Result<(&'static str, &'static str)> {
    let pair = match g {
        Geometry::Hatch(_) => return Err(Error::Unsupported("hatch wire class")),
        Geometry::Light(_) => return Err(Error::Unsupported("light wire class")),
        Geometry::Point(_) => ("ON_Point", "c3101a1d-f157-11d3-bfe7-0010830122f0"),
        Geometry::Curve(Curve::Line { .. }) => {
            ("ON_LineCurve", "4ed7d4db-e947-11d3-bfe5-0010830122f0")
        }
        Geometry::Curve(Curve::Circle { .. }) => {
            ("ON_ArcCurve", "cf33be2a-09b4-11d4-bffb-0010830122f0")
        }
        Geometry::Curve(Curve::Nurbs(_)) => {
            ("ON_NurbsCurve", "4ed7d4dd-e947-11d3-bfe5-0010830122f0")
        }
        Geometry::Curve(Curve::Polycurve { .. }) => {
            ("ON_PolyCurve", "4ed7d4e0-e947-11d3-bfe5-0010830122f0")
        }
        Geometry::Surface(_) => ("ON_NurbsSurface", "4ed7d4de-e947-11d3-bfe5-0010830122f0"),
        Geometry::Mesh(_) => ("ON_Mesh", "4ed7d4e4-e947-11d3-bfe5-0010830122f0"),
        Geometry::Cloud(_) => ("ON_PointCloud", "2488f347-f8fa-11d3-bfec-0010830122f0"),
        Geometry::Instance(_) => ("ON_InstanceRef", "f9cfb638-b9d4-4340-87e3-c56e7865d96a"),
        // The cold retained writer has no native class binding for a BRep source; refuse instead of
        // inventing an archive class UUID.
        Geometry::Brep(_) | Geometry::NativeBrep(_) => {
            return Err(Error::Unsupported("cold brep export"));
        }
    };
    Ok(pair)
}
fn object(o: &Object, e: Expected<'_>, w: &WireOutputPlan<'_>) -> Result<()> {
    if o.native_component_type != 12 || !o.wire.matches(e.output) || o.native_class_uuid.is_nil() {
        return Err(invalid());
    }
    rich(&o.preservation, ENTITY_FIELDS)?;
    if matches!(e.origin, Origin::Addition(_))
        || matches!(e.origin,Origin::Source{row,..} if row.action==RecordAction::Regenerate)
    {
        new_entity_preservation(&o.preservation)?;
    }
    for u in &o.attributes.userdata {
        userdata_entry(u)?;
    }
    let a = &o.attributes;
    if o.preservation["userdata"].as_array() != Some(&a.userdata)
        || o.preservation["groups"].as_array().is_none_or(|v| {
            v.len() != a.groups.len()
                || v.iter()
                    .zip(&a.groups)
                    .any(|(v, n)| v.as_i64() != Some(*n as i64))
        })
    {
        return Err(invalid());
    }
    if a.color
        .is_some_and(|c| rgba(&o.preservation["native_color"]) != Some(c))
    {
        return Err(invalid());
    }
    if a.mode > 3
        || a.native_layer_index < 0
        || a.groups.iter().any(|n| *n < 0)
        || !a.plot_weight_mm.is_finite()
    {
        return Err(invalid());
    }
    // Redundant inventory/preservation channels must agree, not merely exist.
    for (k, v) in [
        ("color_source", Value::from(a.color_source)),
        ("material_source", Value::from(a.material_source)),
        ("line_pattern_source", Value::from(a.line_pattern_source)),
        (
            "native_material_index",
            Value::from(a.native_material_index),
        ),
        (
            "native_line_pattern_index",
            Value::from(a.native_line_pattern_index),
        ),
        ("space", Value::from(a.space)),
        ("display_order", Value::from(a.display_order)),
        ("plot_weight_mm", Value::from(a.plot_weight_mm)),
    ] {
        if o.preservation[k] != v {
            return Err(invalid());
        }
    }
    if a.viewport_id != value_uuid(&o.preservation["viewport_id"])? {
        return Err(invalid());
    }
    if let Some(HostValue::Entity { host, entity }) = e.current() {
        if !entity.attributes.selectable {
            return Err(Error::Unsupported("independent native selectable"));
        }
        let (kind, class) = geometry_class(&entity.geometry)?;
        let layer = match entity.attributes.layer {
            Some(id) => w.host(HostRef::Layer(id)),
            None => w.default_layer(),
        }
        .ok_or_else(invalid)?;
        if o.label != entity.label
            || o.native_kind != kind
            || o.native_class_uuid != Uuid::parse_str(class).map_err(|_| invalid())?
            || o.is_definition_member != matches!(host, HostRef::DefinitionMember { .. })
            || a.layer != Some(layer.id)
            || Some(a.native_layer_index) != layer.archive_reference_index
            || a.visible != (entity.attributes.visible && !entity.attributes.hidden)
            || a.locked != entity.attributes.locked
            || a.color != entity.attributes.color
            || a.color_source != u32::from(entity.attributes.color.is_some())
            || a.space != 1
        {
            return Err(invalid());
        }
        let native = w.inputs().plan().native_placement(host, entity)?;
        let geometry = o.geometry.as_ref().ok_or_else(invalid)?;
        if let Geometry::Instance(id) = entity.geometry {
            fields(geometry, &["Instance"])?;
            let target = w.host(HostRef::Definition(id)).ok_or_else(invalid)?.id;
            if geometry["Instance"].as_str() != Some(target.to_string().as_str())
                || !matrix(o.placement, native)
            {
                return Err(invalid());
            }
        } else {
            if !matrix(o.placement, Transform::identity()) {
                return Err(invalid());
            }
            crate::inspection::geometry(geometry, &entity.geometry, native)?;
        }
    } else if let Origin::Source { original, .. } = e.origin
        && (o.native_kind != original.native_kind
            || o.native_class_uuid != original.native_class_uuid
            || o.label != original.name)
    {
        return Err(invalid());
    }
    Ok(())
}
fn layer(v: &Layer, e: Expected<'_>, w: &WireOutputPlan<'_>) -> Result<()> {
    if v.native_component_type != 5
        || !v.wire.matches(e.output)
        || v.native_kind != "ON_Layer"
        || v.native_class_uuid
            != Uuid::parse_str("95809813-e985-11d3-bfe5-0010830122f0").map_err(|_| invalid())?
    {
        return Err(invalid());
    }
    rich(&v.preservation, LAYER_FIELDS)?;
    if matches!(e.origin, Origin::Addition(_) | Origin::Default)
        || matches!(e.origin,Origin::Source{row,..} if row.action==RecordAction::Regenerate)
    {
        new_layer_preservation(&v.preservation, v.visible, v.locked)?;
    }
    if let Some(HostValue::Layer(l)) = e.current() {
        if !l.selectable {
            return Err(Error::Unsupported("independent native layer selectable"));
        }
        let parent = l
            .parent
            .map(|id| w.host(HostRef::Layer(id)).map(|o| o.id).ok_or_else(invalid))
            .transpose()?;
        if v.name != l.name
            || v.parent != parent
            || v.visible != l.visible
            || v.locked != l.locked
            || v.color != l.color
        {
            return Err(invalid());
        }
    } else if matches!(e.origin, Origin::Default)
        && (v.name != w.default_layer_name().ok_or_else(invalid)?
            || v.parent.is_some()
            || !v.visible
            || v.locked
            || v.color != [128, 128, 128, 255])
    {
        return Err(invalid());
    }
    Ok(())
}
fn definition(v: &Definition, e: Expected<'_>, w: &WireOutputPlan<'_>) -> Result<()> {
    if v.native_component_type != 11
        || !v.wire.matches(e.output)
        || v.native_kind != "ON_InstanceDefinition"
        || v.native_class_uuid
            != Uuid::parse_str("26f8bff6-2618-417f-a158-153d64a94989").map_err(|_| invalid())?
    {
        return Err(invalid());
    }
    rich(&v.preservation, DEFINITION_FIELDS)?;
    if matches!(e.origin, Origin::Addition(_))
        || matches!(e.origin,Origin::Source{row,..} if row.action==RecordAction::Regenerate)
    {
        new_definition_preservation(&v.preservation)?;
    }
    let u = &v.units;
    match (
        u.requires_resolution,
        u.millimeters_per_unit,
        u.meters_per_unit,
    ) {
        (true, None, None) if matches!(u.native_enum, 0 | 255) => {}
        (false, Some(mm), Some(m)) if mm > 0. && m > 0. && equal(mm, m * 1000.) => {}
        _ => return Err(invalid()),
    }
    if let Some(HostValue::Definition(d)) = e.current() {
        if v.label != d.label || v.members.len() != d.members.len() || v.linked {
            return Err(invalid());
        }
        for (id, member) in v.members.iter().zip(&d.members) {
            if *id
                != w.host(HostRef::DefinitionMember {
                    definition: d.id,
                    object: member.id,
                })
                .ok_or_else(invalid)?
                .id
            {
                return Err(invalid());
            }
        }
    }
    match e.origin {
        Origin::Source { ns, row, .. }
            if Some(ns.namespace.id) == w.inputs().plan().primary_namespace =>
        {
            let original = ns
                .source
                .units
                .definitions
                .get(&row.source.key.id)
                .ok_or_else(invalid)?;
            if u.native_enum != original.native_enum
                || u.name != original.name
                || u.millimeters_per_unit != original.declared_mm_per_unit
                || u.requires_resolution != original.declared_mm_per_unit.is_none()
            {
                return Err(invalid());
            }
        }
        _ => {
            if u.native_enum != 2 || u.requires_resolution || u.millimeters_per_unit != Some(1.) {
                return Err(invalid());
            }
        }
    }
    Ok(())
}
fn codes(ty: u32) -> Result<(u32, u32)> {
    // Pinned V5 TCODE values, shared with original-wire admission.
    let (table, record) = match ty {
        1 => (0x16, 0x90),
        2 => (0x25, 0x7a),
        3 => (0x10, 0x40),
        4 => (0x23, 0x78),
        5 => (0x11, 0x50),
        6 => (0x18, 0x73),
        7 => (0x19, 0x74),
        8 => (0x20, 0x75),
        9 => (0x12, 0x60),
        10 => (0x22, 0x77),
        11 => (0x21, 0x76),
        12 => (0x13, 0x70),
        13 => (0x26, 0x7b),
        _ => return Err(invalid()),
    };
    Ok((0x10000000 | table, 0x20008000 | record))
}

fn span_hash(
    reader: &mut File,
    offset: u64,
    len: u64,
    w: &WireOutputPlan<'_>,
    cancel: &Cancellation,
) -> Result<[u8; 32]> {
    reader
        .seek(SeekFrom::Start(offset))
        .map_err(|e| Error::Io(e.to_string()))?;
    let mut left = len;
    let mut hash = Sha256::new();
    let mut buf = [0; 8192];
    while left > 0 {
        w.inputs().check_cancel(cancel)?;
        let n = usize::try_from(left.min(buf.len() as u64)).map_err(|_| invalid())?;
        reader
            .read_exact(&mut buf[..n])
            .map_err(|e| Error::Io(e.to_string()))?;
        hash.update(&buf[..n]);
        left -= n as u64;
    }
    Ok(hash.finalize().into())
}
fn current_layer(
    w: &WireOutputPlan<'_>,
    v: &CurrentLayer,
    expected: &BTreeMap<Uuid, Expected<'_>>,
) -> Result<()> {
    let d = w.inputs().capture().document();
    let explicit = if let Some(id) = d.current_layer {
        Some(w.host(HostRef::Layer(id)).ok_or_else(invalid)?)
    } else if w.inputs().plan().primary_namespace.is_some() {
        None
    } else {
        w.default_layer().or_else(|| {
            w.table_order(5)
                .first()
                .and_then(|id| expected.get(id).map(|e| e.output))
        })
    };
    if let Some(e) = explicit {
        if v.id != Some(e.id)
            || v.archive_reference_index != e.archive_reference_index
            || v.sdk_destination_index != e.sdk_destination_index
        {
            return Err(invalid());
        }
    } else {
        let matches =
            v.id.and_then(|id| expected.get(&id))
                .map(|e| e.output)
                .or_else(|| {
                    expected
                        .values()
                        .find(|e| {
                            e.output.native_component_type == 5
                                && e.output.archive_reference_index == v.archive_reference_index
                        })
                        .map(|e| e.output)
                });
        let e = matches.ok_or_else(invalid)?;
        if e.native_component_type != 5
            || v.archive_reference_index != e.archive_reference_index
            || v.sdk_destination_index != e.sdk_destination_index
        {
            return Err(invalid());
        }
    }
    Ok(())
}
pub fn verify(
    wire: &WireOutputPlan<'_>,
    response: &[u8],
    held_output: &File,
    expected: ExpectedRetainedOutput,
    limits: &Limits,
    cancel: &Cancellation,
) -> Result<ColdRetainedProof> {
    wire.inputs().check_cancel(cancel)?;
    let reserved = Usage {
        bytes: 8192 + std::mem::size_of::<ColdRetainedProof>() as u64,
        ..Usage::default()
    };
    let remaining = limits.remaining_after(reserved)?;
    om9_archive::admit_json(response, &remaining)?;
    preflight(response, &remaining, &|| wire.inputs().check_cancel(cancel))?;
    wire.inputs().check_cancel(cancel)?;
    let r: Report = serde_json::from_slice(response).map_err(|_| invalid())?;
    wire.inputs().check_cancel(cancel)?;
    let count = (1..=13).try_fold(0usize, |n, ty| {
        n.checked_add(wire.table_order(ty).len())
            .ok_or(Error::Overflow("cold live count"))
    })?;
    let mut peak = reserved;
    peak.add(report_usage(&r, limits)?, limits)?;
    peak.add(
        Usage {
            bytes: response.len() as u64,
            ..Usage::default()
        },
        limits,
    )?;
    for u in [
        Usage::btree::<Uuid, Expected>(count)?,
        Usage::btree::<Uuid, ()>(count)?,
        Usage::btree::<Uuid, &RawSpan>(r.raw_spans.len())?,
        Usage::btree::<u64, &RawSpan>(r.raw_spans.len())?,
        Usage::btree::<(u32, usize), &RawSpan>(r.raw_spans.len())?,
        Usage::btree::<u32, usize>(26)?,
        Usage::btree::<u32, usize>(26)?,
        Usage::btree::<u32, usize>(26)?,
        Usage::btree::<Uuid, ()>(r.actual.bounds.len())?,
    ] {
        peak.add(u, limits)?;
    }
    let primary = wire.inputs().plan().primary_namespace.and_then(|id| {
        wire.inputs()
            .plan()
            .overlay
            .namespaces
            .iter()
            .find(|ns| ns.namespace.id == id)
    });
    if r.schema_version != 4
        || r.writer_policy != "om9-retained-current-v1"
        || r.bounds_policy != "cold_leaf_hull_v1"
        || r.status != "validated_retained_current"
        || r.archive_version != 50
        || !r.cold_validation_required
        || !r.admission_required
        || r.host_ready
        || r.primary_source_sha256 != primary.map(|ns| ns.source.sha256)
        || r.request_sha256 != expected.request_sha256
        || r.output_sha256 != expected.output_sha256
        || r.output_bytes != expected.output_bytes
        || expected.output_bytes == 0
        || expected.output_bytes > 64 * 1024 * 1024
        || r.objects != wire.table_order(12).len() as u64
        || r.layers != wire.table_order(5).len() as u64
        || r.definitions != wire.table_order(11).len() as u64
        || r.checks.live_records != count as u64
        || r.actual.objects.len() as u64 != r.objects
        || r.actual.layers.len() as u64 != r.layers
        || r.actual.definitions.len() as u64 != r.definitions
        || r.checks.opaque_chunks > limits.max_nodes
    {
        return Err(invalid());
    }
    units_valid(&r.actual.model_units)?;
    units_valid(&r.actual.page_units)?;
    if let Some(ns) = primary {
        let p = wire.inputs().source(ns.namespace.id).ok_or_else(invalid)?;
        if !units_equal(&r.actual.model_units, p.original_model_units())
            || !units_equal(&r.actual.page_units, p.original_page_units())
        {
            return Err(invalid());
        }
    } else if !declared(&r.actual.model_units, Unit::Millimeter)
        || !declared(
            &r.actual.page_units,
            wire.inputs().capture().document().page_unit,
        )
    {
        return Err(invalid());
    }
    if held_output
        .metadata()
        .map_err(|e| Error::Io(e.to_string()))?
        .len()
        != expected.output_bytes
    {
        return Err(invalid());
    }
    let mut reader = held_output
        .try_clone()
        .map_err(|e| Error::Io(e.to_string()))?;
    if span_hash(&mut reader, 0, expected.output_bytes, wire, cancel)? != expected.output_sha256 {
        return Err(invalid());
    }
    let expected_rows = expected_rows(wire, cancel)?;
    if expected_rows.len() != count {
        return Err(invalid());
    }
    let mut tops = BTreeMap::new();
    let mut top_keys = BTreeMap::new();
    let mut occurrences = BTreeMap::<u32, usize>::new();
    let mut raw = BTreeMap::new();
    let mut ordinal = BTreeMap::<u32, usize>::new();
    let mut cursor = 0u64;
    let mut saw_record = false;
    let mut prior_end = 0;
    for s in &r.raw_spans {
        wire.inputs().check_cancel(cancel)?;
        let end = s
            .offset
            .checked_add(s.byte_length)
            .filter(|end| *end <= expected.output_bytes && s.byte_length > 0)
            .ok_or_else(invalid)?;
        if span_hash(&mut reader, s.offset, s.byte_length, wire, cancel)? != s.sha256 {
            return Err(invalid());
        }
        if s.kind == "top_level" {
            if saw_record
                || s.native_component_type.is_some()
                || s.id.is_some()
                || s.record_type.is_some()
                || s.offset != cursor
                || (!matches!(s.table_type, 0 | 0x7fff)
                    && !(0x10000010..=0x10000026).contains(&s.table_type))
            {
                return Err(invalid());
            }
            if (cursor == 0) != (s.table_type == 0)
                || ((end == expected.output_bytes) != (s.table_type == 0x7fff))
            {
                return Err(invalid());
            }
            let n = occurrences.entry(s.table_type).or_insert(0);
            if *n > 0 && s.table_type != 0x10000017 {
                return Err(invalid());
            }
            top_keys.insert((s.table_type, *n), s);
            *n += 1;
            tops.insert(s.offset, s);
            cursor = end;
        } else if s.kind == "record" {
            saw_record = true;
            let ty = s.native_component_type.ok_or_else(invalid)?;
            let id = s.id.ok_or_else(invalid)?;
            let e = expected_rows.get(&id).ok_or_else(invalid)?;
            let (table, kind) = codes(ty)?;
            let top = tops
                .range(..=s.offset)
                .next_back()
                .map(|(_, s)| *s)
                .ok_or_else(invalid)?;
            if ty != e.output.native_component_type
                || s.table_type != table
                || s.record_type != Some(kind)
                || top.table_type != table
                || s.offset < top.offset + 12
                || end >= top.offset + top.byte_length
                || s.offset < prior_end
                || raw.insert(id, s).is_some()
            {
                return Err(invalid());
            }
            prior_end = end;
            let n = ordinal.entry(ty).or_insert(0);
            if wire.table_order(ty).get(*n) != Some(&id) {
                return Err(invalid());
            }
            *n += 1;
        } else {
            return Err(invalid());
        }
    }
    if cursor != expected.output_bytes || raw.len() != count {
        return Err(invalid());
    }
    let mut raw_records = 0u64;
    for (id, e) in &expected_rows {
        wire.inputs().check_cancel(cancel)?;
        if let Origin::Source { row, original, .. } = e.origin
            && row.action == RecordAction::Retain
            && !e.refresh
        {
            let s = raw.get(id).ok_or_else(invalid)?;
            if s.sha256 != original.wire.sha256 || s.byte_length != original.wire.byte_length {
                return Err(invalid());
            }
            raw_records += 1;
        }
    }
    let mut raw_tables = 0u64;
    if let Some(ns) = primary {
        let p = wire.inputs().source(ns.namespace.id).ok_or_else(invalid)?;
        let mut position = BTreeMap::<u32, usize>::new();
        for original in p.top_level_spans() {
            wire.inputs().check_cancel(cancel)?;
            if matches!(
                original.table_type,
                0x7fff | 0x10000011 | 0x10000021 | 0x10000013
            ) {
                continue;
            }
            let n = position.entry(original.table_type).or_insert(0);
            let actual = top_keys
                .get(&(original.table_type, *n))
                .ok_or_else(invalid)?;
            *n += 1;
            if actual.sha256 != original.sha256 || actual.byte_length != original.byte_length {
                return Err(invalid());
            }
            raw_tables += 1;
        }
        for (ty, n) in top_keys.keys() {
            if !matches!(*ty, 0x7fff | 0x10000011 | 0x10000021 | 0x10000013)
                && position.get(ty).is_none_or(|count| *n >= *count)
            {
                return Err(invalid());
            }
        }
    }
    if r.checks.raw_records != raw_records || r.checks.raw_tables != raw_tables {
        return Err(invalid());
    }
    let mut seen = BTreeSet::new();
    for (index, v) in r.actual.objects.iter().enumerate() {
        wire.inputs().check_cancel(cancel)?;
        if wire.table_order(12).get(index) != Some(&v.id) || !seen.insert(v.id) {
            return Err(invalid());
        }
        object(v, *expected_rows.get(&v.id).ok_or_else(invalid)?, wire)?;
    }
    for (index, v) in r.actual.layers.iter().enumerate() {
        wire.inputs().check_cancel(cancel)?;
        if wire.table_order(5).get(index) != Some(&v.id) || !seen.insert(v.id) {
            return Err(invalid());
        }
        layer(v, *expected_rows.get(&v.id).ok_or_else(invalid)?, wire)?;
    }
    for (index, v) in r.actual.definitions.iter().enumerate() {
        wire.inputs().check_cancel(cancel)?;
        if wire.table_order(11).get(index) != Some(&v.id) || !seen.insert(v.id) {
            return Err(invalid());
        }
        definition(v, *expected_rows.get(&v.id).ok_or_else(invalid)?, wire)?;
    }
    current_layer(wire, &r.actual.current_layer, &expected_rows)?;
    let dirty = expected_rows.values().filter(|e| e.refresh).count();
    if dirty != r.actual.bounds.len() || r.checks.bounds != dirty as u64 {
        return Err(invalid());
    }
    let mut seen_bounds = BTreeSet::new();
    let mut represented_bounds = 0;
    let mut native_only_bounds = 0;
    let mut work = 0;
    for b in &r.actual.bounds {
        wire.inputs().check_cancel(cancel)?;
        let e = expected_rows.get(&b.id).ok_or_else(invalid)?;
        if !e.refresh
            || e.output.native_component_type != b.native_component_type
            || !seen_bounds.insert(b.id)
            || !b.matched
            || !valid_box(b.stored)
            || !valid_box(b.computed)
            || !box_equal(b.stored, b.computed)
        {
            return Err(invalid());
        }
        if let Some(host) = e.current() {
            let hull = captured_hull(
                wire,
                host,
                Transform::identity(),
                0,
                &mut work,
                limits,
                cancel,
            )?
            .ok_or_else(invalid)?;
            if !box_equal(b.computed, hull) {
                return Err(invalid());
            }
            represented_bounds += 1;
        } else {
            native_only_bounds += 1;
        }
    }
    if held_output
        .metadata()
        .map_err(|e| Error::Io(e.to_string()))?
        .len()
        != expected.output_bytes
        || span_hash(&mut reader, 0, expected.output_bytes, wire, cancel)? != expected.output_sha256
    {
        return Err(invalid());
    }
    wire.inputs().check_cancel(cancel)?;
    Ok(ColdRetainedProof {
        request_sha256: expected.request_sha256,
        output_sha256: expected.output_sha256,
        output_bytes: expected.output_bytes,
        live_records: count as u64,
        represented_bounds,
        native_only_bounds,
    })
}

type Box3 = [[f64; 3]; 2];
fn valid_box(b: Box3) -> bool {
    b.iter().flatten().all(|n| n.is_finite()) && (0..3).all(|i| b[0][i] <= b[1][i])
}
fn box_equal(a: Box3, b: Box3) -> bool {
    // Translation must not enlarge a cache-error allowance. Per-axis hull span
    // permits small relative arithmetic error; a fixed ULP allowance handles
    // rounding at a far-away origin without erasing a thin finite hull.
    (0..3).all(|i| {
        let span = (a[1][i] - a[0][i]).max(b[1][i] - b[0][i]);
        (0..2).all(|end| {
            let (a, b) = (a[end][i], b[end][i]);
            if !a.is_finite() || !b.is_finite() {
                return false;
            }
            if a == b {
                return true;
            }
            let ordered = |v: f64| {
                let bits = v.to_bits();
                if bits & (1u64 << 63) != 0 {
                    !bits
                } else {
                    bits | (1u64 << 63)
                }
            };
            let ulps = ordered(a).abs_diff(ordered(b));
            ulps <= 32 || (span.is_finite() && (a - b).abs() <= span * 1e-9)
        })
    })
}
fn union(a: Option<Box3>, b: Option<Box3>) -> Option<Box3> {
    match (a, b) {
        (Some(a), Some(b)) => Some([
            std::array::from_fn(|i| a[0][i].min(b[0][i])),
            std::array::from_fn(|i| a[1][i].max(b[1][i])),
        ]),
        (a, b) => a.or(b),
    }
}
fn points_hull(
    points: &[[f64; 3]],
    t: Transform,
    check: &dyn Fn() -> Result<()>,
) -> Result<Option<Box3>> {
    let mut b = None;
    for p in points {
        check()?;
        let p = t.point(*p)?;
        b = union(b, Some([p, p]));
    }
    Ok(b)
}
fn curve_hull(
    c: &Curve,
    t: Transform,
    depth: usize,
    l: &Limits,
    check: &dyn Fn() -> Result<()>,
) -> Result<Option<Box3>> {
    check()?;
    if depth > l.max_depth {
        return Err(Error::Budget("cold curve depth"));
    }
    Ok(match c {
        Curve::Line { start, end, .. } => points_hull(&[*start, *end], t, check)?,
        Curve::Circle {
            center,
            x,
            y,
            radius,
            ..
        } => {
            let center = t.point(*center)?;
            let extent = std::array::from_fn::<_, 3, _>(|i| {
                let a = (0..3).map(|j| t.0[i][j] * x[j] * radius).sum::<f64>();
                let b = (0..3).map(|j| t.0[i][j] * y[j] * radius).sum::<f64>();
                a.hypot(b)
            });
            let b = [
                std::array::from_fn(|i| center[i] - extent[i]),
                std::array::from_fn(|i| center[i] + extent[i]),
            ];
            if !valid_box(b) {
                return Err(invalid());
            }
            Some(b)
        }
        Curve::Nurbs(n) => {
            if n.weights.iter().any(|w| *w <= 0.) {
                return Err(Error::Unsupported("cold nonpositive rational hull"));
            }
            points_hull(&n.poles, t, check)?
        }
        Curve::Polycurve { segments, .. } => {
            let mut b = None;
            for c in segments {
                b = union(b, curve_hull(c, t, depth + 1, l, check)?);
            }
            b
        }
    })
}
fn captured_hull(
    w: &WireOutputPlan<'_>,
    host: HostValue<'_>,
    t: Transform,
    depth: usize,
    work: &mut u64,
    l: &Limits,
    cancel: &Cancellation,
) -> Result<Option<Box3>> {
    w.inputs().check_cancel(cancel)?;
    *work = work
        .checked_add(1)
        .ok_or(Error::Overflow("cold hull work"))?;
    if depth > l.max_depth || *work > l.max_nodes {
        return Err(Error::Budget("cold hull traversal"));
    }
    match host {
        HostValue::Definition(d) => {
            let mut b = None;
            for member in &d.members {
                b = union(
                    b,
                    captured_hull(
                        w,
                        HostValue::Entity {
                            host: HostRef::DefinitionMember {
                                definition: d.id,
                                object: member.id,
                            },
                            entity: member,
                        },
                        t,
                        depth + 1,
                        work,
                        l,
                        cancel,
                    )?,
                );
            }
            Ok(b)
        }
        HostValue::Entity { host, entity } => {
            let t = t.compose(w.inputs().plan().native_placement(host, entity)?)?;
            match &entity.geometry {
                Geometry::Instance(id) => {
                    let d = w
                        .inputs()
                        .capture()
                        .document()
                        .definitions
                        .get(id)
                        .ok_or_else(invalid)?;
                    captured_hull(w, HostValue::Definition(d), t, depth + 1, work, l, cancel)
                }
                Geometry::Point(p) => points_hull(&[*p], t, &|| w.inputs().check_cancel(cancel)),
                Geometry::Curve(c) => curve_hull(c, t, 0, l, &|| w.inputs().check_cancel(cancel)),
                Geometry::Surface(s) => {
                    if s.weights.iter().any(|w| *w <= 0.) {
                        return Err(Error::Unsupported("cold nonpositive rational hull"));
                    }
                    points_hull(&s.poles, t, &|| w.inputs().check_cancel(cancel))
                }
                Geometry::NativeBrep(b) => {
                    points_hull(&b.display.vertices, t, &|| w.inputs().check_cancel(cancel))
                }
                Geometry::Mesh(m) => {
                    points_hull(&m.vertices, t, &|| w.inputs().check_cancel(cancel))
                }
                Geometry::Brep(b) => {
                    points_hull(&b.display.points, t, &|| w.inputs().check_cancel(cancel))
                }
                Geometry::Cloud(c) => {
                    points_hull(&c.points, t, &|| w.inputs().check_cancel(cancel))
                }
                Geometry::Hatch(h) => {
                    points_hull(&h.display.vertices, t, &|| w.inputs().check_cancel(cancel))
                }
                Geometry::Light(light) => {
                    points_hull(&[light.display], t, &|| w.inputs().check_cancel(cancel))
                }
            }
        }
        HostValue::Layer(_) => Err(invalid()),
    }
}

#[cfg(test)]
mod strict_metadata_tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn bounds_tolerance_does_not_erase_small_or_translated_thin_hulls() {
        assert!(!box_equal([[0.; 3]; 2], [[1e-12, 2e-12, 3e-12]; 2]));
        let thin = [[1e9, 1e9, 1e9], [1e9 + 1e-3, 1e9 + 1e-3, 1e9 + 1e-3]];
        assert!(!box_equal(thin, [[1e9; 3]; 2]));
    }
    #[test]
    fn malformed_preserved_reference_entries_reject() {
        // Shape helper covers preserved opaque metadata too, where a new-master
        // empty-array policy cannot stand in for entry validation.
        let cases = [
            ("display_mode_overrides", json!([{"unrequested":0}])),
            ("render_material_references", json!([{"unrequested":0}])),
            ("texture_mapping_channels", json!([{"unrequested":0}])),
        ];
        for (key, entries) in cases {
            let v = json!({key:entries,"userdata":[]});
            assert!(rich(&v, &[key, "userdata"]).is_err(), "entry {key}");
        }
    }
}

