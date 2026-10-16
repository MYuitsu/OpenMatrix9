// SPDX-License-Identifier: LGPL-2.1-or-later
//! Original-wire provenance admission, separate from current host state.
use om9_core::{
    Error, Result,
    budget::{Limits, Usage},
    retained::{ImportNamespace, NativeRecordKey, NativeUnits, ResolvedBy, RetainedSource},
    transaction::Cancellation,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

#[derive(Debug)]
pub struct SourcePreparation {
    dto: PreparedDto,
    records: BTreeMap<NativeRecordKey, usize>,
    owned: Usage,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PreparedDto {
    schema_version: u32,
    status: String,
    decoder_policy: String,
    admission_required: bool,
    host_ready: bool,
    archive: Archive,
    original_units: Units,
    records: Vec<PreparedSourceRecord>,
    top_level_spans: Vec<PreparedArchiveSpan>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Archive {
    version: u32,
    opennurbs_version: u32,
    file_bytes: u64,
    endmark_bytes: u64,
    #[serde(deserialize_with = "sha256")]
    source_sha256: [u8; 32],
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Units {
    model: PreparedNativeUnits,
    page: PreparedNativeUnits,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreparedNativeUnits {
    pub native_enum: u32,
    pub name: String,
    #[serde(deserialize_with = "required_option")]
    pub millimeters_per_unit: Option<f64>,
    #[serde(deserialize_with = "required_option")]
    pub meters_per_unit: Option<f64>,
    pub requires_resolution: bool,
    pub absolute_tolerance: f64,
    pub angle_tolerance_radians: f64,
    pub relative_tolerance: f64,
    // Original display settings belong to the retained settings span.
    pub distance_display_mode: u32,
    pub distance_display_precision: u32,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceIdentity {
    native_component_type: u32,
    #[serde(deserialize_with = "crate::import_dto::uuid")]
    id: Uuid,
    #[serde(deserialize_with = "required_option")]
    native_index: Option<i32>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreparedSourceRecord {
    source: SourceIdentity,
    pub wire: WireIdentity,
    pub native_kind: String,
    #[serde(deserialize_with = "crate::import_dto::uuid")]
    pub native_class_uuid: Uuid,
    pub name: String,
    userdata: Vec<UserData>,
    attribute_userdata: Vec<UserData>,
    #[serde(deserialize_with = "required_option")]
    definition_units: Option<DefinitionUnits>,
    pub preservation_flags: PreservationFlags,
}
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum UserData {
    Unknown(UnknownUserData),
    Known(KnownUserData),
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UnknownUserData {
    #[serde(deserialize_with = "crate::import_dto::uuid")]
    application_id: Uuid,
    #[serde(deserialize_with = "crate::import_dto::uuid")]
    class_id: Uuid,
    #[serde(deserialize_with = "crate::import_dto::uuid")]
    item_id: Uuid,
    archive_version: u32,
    opennurbs_version: u32,
    copy_count: u32,
    byte_length: u64,
    native_sizeof_bytes: u64,
    unknown: bool,
    opaque_references: bool,
    #[serde(deserialize_with = "sha1")]
    payload_sha1: [u8; 20],
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct KnownUserData {
    #[serde(deserialize_with = "crate::import_dto::uuid")]
    application_id: Uuid,
    #[serde(deserialize_with = "crate::import_dto::uuid")]
    class_id: Uuid,
    #[serde(deserialize_with = "crate::import_dto::uuid")]
    item_id: Uuid,
    copy_count: u32,
    native_sizeof_bytes: u64,
    unknown: bool,
    opaque_references: bool,
}
impl UserData {
    fn unknown(&self) -> bool {
        matches!(self, Self::Unknown(_))
    }
    fn validate(&self) -> Result<()> {
        let (app, class, item, size, opaque) = match self {
            Self::Unknown(u) => {
                if !u.unknown
                    || u.archive_version == 0
                    || u.opennurbs_version == 0
                    || u.byte_length == 0
                {
                    return Err(invalid());
                }
                let _ = (u.copy_count, u.payload_sha1);
                (
                    u.application_id,
                    u.class_id,
                    u.item_id,
                    u.native_sizeof_bytes,
                    u.opaque_references,
                )
            }
            Self::Known(u) => {
                if u.unknown {
                    return Err(invalid());
                }
                let _ = u.copy_count;
                (
                    u.application_id,
                    u.class_id,
                    u.item_id,
                    u.native_sizeof_bytes,
                    u.opaque_references,
                )
            }
        };
        if app.is_nil() || class.is_nil() || item.is_nil() || size == 0 || !opaque {
            return Err(invalid());
        }
        Ok(())
    }
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DefinitionUnits {
    native_enum: u32,
    name: String,
    #[serde(deserialize_with = "required_option")]
    declared_mm_per_unit: Option<f64>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreservationFlags {
    pub master_userdata_present: bool,
    pub attribute_userdata_present: bool,
    pub unknown_userdata_present: bool,
    pub unrepresented_record_chunks_present: bool,
    pub opaque_identity_remap_requires_admission: bool,
    pub opaque_length_remap_requires_admission: bool,
    pub selected_serializer_context_required: bool,
}
/// These three index contexts are intentionally separate. SourceRecord keys
/// name the decoded host projection; none is an object-table ordinal.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireIdentity {
    #[serde(deserialize_with = "crate::import_dto::uuid")]
    pub id: Uuid,
    #[serde(deserialize_with = "required_option")]
    pub serialized_index: Option<i32>,
    #[serde(deserialize_with = "required_option")]
    pub archive_reference_index: Option<i32>,
    #[serde(deserialize_with = "required_option")]
    pub sdk_destination_index: Option<i32>,
    pub table_type: u32,
    pub record_type: u32,
    pub offset: u64,
    pub byte_length: u64,
    #[serde(deserialize_with = "sha256")]
    pub sha256: [u8; 32],
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreparedArchiveSpan {
    pub table_type: u32,
    pub offset: u64,
    pub byte_length: u64,
    #[serde(deserialize_with = "sha256")]
    pub sha256: [u8; 32],
    pub anonymous_user_table: bool,
}
fn invalid() -> Error {
    Error::Invalid("original-wire source correspondence")
}
fn required_option<'de, D: serde::Deserializer<'de>, T: Deserialize<'de>>(
    d: D,
) -> std::result::Result<Option<T>, D::Error> {
    Option::<T>::deserialize(d)
}
fn hash<'de, D: serde::Deserializer<'de>, const N: usize>(
    d: D,
) -> std::result::Result<[u8; N], D::Error> {
    let s = String::deserialize(d)?;
    if s.len() != 2 * N {
        return Err(serde::de::Error::custom("native digest length"));
    }
    let mut result = [0; N];
    fn nibble(b: u8) -> Option<u8> {
        match b {
            b'0'..=b'9' => Some(b - b'0'),
            b'a'..=b'f' => Some(b - b'a' + 10),
            _ => None,
        }
    }
    for (out, pair) in result.iter_mut().zip(s.as_bytes().as_chunks::<2>().0) {
        *out = nibble(pair[0])
            .and_then(|hi| nibble(pair[1]).map(|lo| hi * 16 + lo))
            .ok_or_else(|| serde::de::Error::custom("noncanonical native digest"))?;
    }
    Ok(result)
}
fn sha256<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<[u8; 32], D::Error> {
    hash(d)
}
fn sha1<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<[u8; 20], D::Error> {
    hash(d)
}
fn range(
    offset: u64,
    length: u64,
    bytes: &[u8],
    expected: [u8; 32],
    cancel: &Cancellation,
) -> Result<u64> {
    let end = offset
        .checked_add(length)
        .ok_or(Error::Overflow("source span end"))?;
    let (a, b) = (
        usize::try_from(offset).map_err(|_| invalid())?,
        usize::try_from(end).map_err(|_| invalid())?,
    );
    let slice = bytes
        .get(a..b)
        .filter(|s| !s.is_empty())
        .ok_or_else(invalid)?;
    let mut digest = Sha256::new();
    for block in slice.chunks(8192) {
        crate::current_overlay::canceled(cancel)?;
        digest.update(block);
    }
    if <[u8; 32]>::from(digest.finalize()) != expected {
        return Err(invalid());
    }
    Ok(end)
}
fn unit(actual: &PreparedNativeUnits, stored: &NativeUnits) -> Result<()> {
    if actual.native_enum != stored.native_enum
        || actual.name != stored.name
        || actual.millimeters_per_unit != stored.declared_mm_per_unit
        || actual.absolute_tolerance != stored.absolute_tolerance
        || actual.angle_tolerance_radians != stored.angular_tolerance_radians
        || actual.relative_tolerance != stored.relative_tolerance
        || actual.requires_resolution != matches!(actual.native_enum, 0 | 255)
        || !actual.absolute_tolerance.is_finite()
        || actual.absolute_tolerance <= 0.
        || !actual.angle_tolerance_radians.is_finite()
        || actual.angle_tolerance_radians < 0.
        || !actual.relative_tolerance.is_finite()
        || actual.relative_tolerance < 0.
        || actual.distance_display_mode > 2
        || actual.distance_display_precision > 15
    {
        return Err(invalid());
    }
    match (
        actual.millimeters_per_unit,
        actual.meters_per_unit,
        stored.resolved_by,
    ) {
        (Some(mm), Some(m), ResolvedBy::Declared)
            if !actual.requires_resolution
                && mm == stored.millimeters_per_unit
                && mm.is_finite()
                && mm > 0.
                && m.is_finite()
                && m > 0.
                && (m * 1000. - mm).abs() <= mm * 1e-12 => {}
        (None, None, ResolvedBy::Explicit)
            if actual.requires_resolution
                && stored.millimeters_per_unit.is_finite()
                && stored.millimeters_per_unit > 0. => {}
        _ => return Err(invalid()),
    }
    Ok(())
}
// Pinned public opennurbs_3dm.h table/record constants. The selected source
// preparer currently admits V5 (50) only; later codecs require a new ABI gate.
fn codes(ty: u32) -> Result<(u32, u32)> {
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
        _ => return Err(Error::Unsupported("selected original-wire component codec")),
    };
    Ok((0x10000000 | table, 0x20008000 | record))
}
impl PreparedSourceRecord {
    pub fn key(&self) -> Result<NativeRecordKey> {
        Ok(NativeRecordKey {
            kind: crate::import_dto::kind(self.source.native_component_type)?,
            id: self.source.id,
            native_index: self.source.native_index,
        })
    }
    fn indices(&self) -> Result<()> {
        let s = &self.source;
        let w = &self.wire;
        if [
            s.native_index,
            w.serialized_index,
            w.archive_reference_index,
            w.sdk_destination_index,
        ]
        .iter()
        .flatten()
        .any(|i| *i < 0)
            || s.id != w.id
            || (s.id.is_nil() && s.native_index.is_none())
        {
            return Err(invalid());
        }
        let valid = match s.native_component_type {
            5 => {
                s.native_index.is_some()
                    && s.native_index == w.archive_reference_index
                    && s.native_index == w.sdk_destination_index
                    && w.serialized_index.is_some()
            }
            6 => {
                s.native_index.is_some()
                    && s.native_index == w.sdk_destination_index
                    && w.serialized_index.is_some()
                    && w.serialized_index == w.archive_reference_index
            }
            9 => {
                s.native_index.is_none()
                    && w.archive_reference_index.is_some()
                    && w.serialized_index.is_some()
                    && w.serialized_index == w.sdk_destination_index
            }
            11 => {
                s.native_index.is_some()
                    && s.native_index == w.archive_reference_index
                    && s.native_index == w.sdk_destination_index
                    && w.serialized_index.is_none()
            }
            12 | 13 => {
                s.native_index.is_none()
                    && w.serialized_index.is_none()
                    && w.archive_reference_index.is_none()
                    && w.sdk_destination_index.is_none()
            }
            _ => {
                s.native_index.is_some()
                    && s.native_index == w.sdk_destination_index
                    && w.archive_reference_index.is_some()
            }
        };
        if !valid {
            return Err(invalid());
        }
        Ok(())
    }
    fn flags(&self) -> Result<()> {
        let f = &self.preservation_flags;
        let unknown = self
            .userdata
            .iter()
            .chain(&self.attribute_userdata)
            .any(UserData::unknown);
        let conservative = f.master_userdata_present
            || f.attribute_userdata_present
            || f.unrepresented_record_chunks_present
            || f.selected_serializer_context_required;
        if f.master_userdata_present != !self.userdata.is_empty()
            || f.attribute_userdata_present != !self.attribute_userdata.is_empty()
            || f.unknown_userdata_present != unknown
            || (conservative
                && (!f.opaque_identity_remap_requires_admission
                    || !f.opaque_length_remap_requires_admission))
        {
            return Err(invalid());
        }
        for u in self.userdata.iter().chain(&self.attribute_userdata) {
            u.validate()?;
            // copy_count=0 remains a real original attachment; it is never an
            // instruction to skip it. Payload SHA1 is the SDK's owned oracle.
        }
        Ok(())
    }
}
impl SourcePreparation {
    pub fn decode(
        response: &[u8],
        namespace: &ImportNamespace,
        source: &RetainedSource,
        source_bytes: &[u8],
        limits: &Limits,
        cancel: &Cancellation,
    ) -> Result<Self> {
        crate::current_overlay::canceled(cancel)?;
        om9_archive::admit_json(response, limits)?;
        let dto: PreparedDto = serde_json::from_slice(response).map_err(|_| invalid())?;
        if dto.schema_version != 4
            || dto.status != "retained_source_prepared"
            || dto.decoder_policy != "om9-retained-source-v1"
            || !dto.admission_required
            || dto.host_ready
            || dto.archive.version != 50
            || source.archive_version != 50
            || dto.archive.opennurbs_version == 0
            || namespace.source != source.id
            || dto.archive.file_bytes != source_bytes.len() as u64
            || dto.archive.endmark_bytes != dto.archive.file_bytes
            || dto.archive.source_sha256 != source.sha256
            || dto.records.len() != namespace.records.len()
        {
            return Err(invalid());
        }
        range(
            0,
            source_bytes.len() as u64,
            source_bytes,
            source.sha256,
            cancel,
        )?;
        unit(&dto.original_units.model, &source.units.model)?;
        unit(&dto.original_units.page, &source.units.page)?;
        let mut owned = Usage::array::<Self>(1)?;
        owned.add(
            Usage::array::<PreparedSourceRecord>(dto.records.capacity())?,
            limits,
        )?;
        owned.add(
            Usage::array::<PreparedArchiveSpan>(dto.top_level_spans.capacity())?,
            limits,
        )?;
        owned.add(
            Usage::btree::<NativeRecordKey, usize>(dto.records.len())?,
            limits,
        )?;
        for s in [
            &dto.status,
            &dto.decoder_policy,
            &dto.original_units.model.name,
            &dto.original_units.page.name,
        ] {
            owned.add(
                Usage {
                    bytes: s.capacity() as u64,
                    ..Usage::default()
                },
                limits,
            )?;
        }
        for r in &dto.records {
            for s in [&r.native_kind, &r.name] {
                owned.add(
                    Usage {
                        bytes: s.capacity() as u64,
                        ..Usage::default()
                    },
                    limits,
                )?;
            }
            owned.add(Usage::array::<UserData>(r.userdata.capacity())?, limits)?;
            owned.add(
                Usage::array::<UserData>(r.attribute_userdata.capacity())?,
                limits,
            )?;
            if let Some(u) = &r.definition_units {
                owned.add(
                    Usage {
                        bytes: u.name.capacity() as u64,
                        ..Usage::default()
                    },
                    limits,
                )?;
            }
        }
        // The response stays live during admission. Reserve all lookup and
        // uniqueness scratch before allocating any map, not after it is built.
        let mut peak = owned;
        peak.add(
            Usage {
                bytes: response.len() as u64,
                ..Usage::default()
            },
            limits,
        )?;
        peak.add(Usage::btree::<(u32, i32), ()>(dto.records.len())?, limits)?;
        peak.add(
            Usage::btree::<u32, (u64, u64)>(dto.top_level_spans.len())?,
            limits,
        )?;
        peak.add(
            Usage::btree::<NativeRecordKey, ()>(namespace.records.len())?,
            limits,
        )?;
        let mut tables = BTreeMap::new();
        let mut cursor = 0;
        if dto
            .top_level_spans
            .first()
            .is_none_or(|s| s.table_type != 0)
            || dto
                .top_level_spans
                .last()
                .is_none_or(|s| s.table_type != 0x7fff)
        {
            return Err(invalid());
        }
        for (i, span) in dto.top_level_spans.iter().enumerate() {
            crate::current_overlay::canceled(cancel)?;
            if span.offset != cursor
                || span.anonymous_user_table != (span.table_type == 0x10000017)
                || (i != 0
                    && i + 1 != dto.top_level_spans.len()
                    && !(0x10000010..=0x10000026).contains(&span.table_type))
            {
                return Err(invalid());
            }
            cursor = range(
                span.offset,
                span.byte_length,
                source_bytes,
                span.sha256,
                cancel,
            )?;
            if !matches!(span.table_type, 0 | 0x7fff | 0x10000017)
                && tables
                    .insert(span.table_type, (span.offset, cursor))
                    .is_some()
            {
                return Err(invalid());
            }
        }
        if cursor != dto.archive.file_bytes {
            return Err(invalid());
        }
        let mut records = BTreeMap::new();
        let mut refs = BTreeSet::new();
        let mut previous_end = 0;
        let mut definitions = 0;
        for (i, r) in dto.records.iter().enumerate() {
            crate::current_overlay::canceled(cancel)?;
            r.indices()?;
            r.flags()?;
            if r.native_kind.is_empty()
                || r.native_class_uuid.is_nil()
                || (r.wire.table_type, r.wire.record_type) != codes(r.source.native_component_type)?
            {
                return Err(invalid());
            }
            let (start, end) = *tables.get(&r.wire.table_type).ok_or_else(invalid)?;
            let record_end = range(
                r.wire.offset,
                r.wire.byte_length,
                source_bytes,
                r.wire.sha256,
                cancel,
            )?;
            if r.wire.offset < previous_end || r.wire.offset < start + 12 || record_end >= end {
                return Err(invalid());
            }
            previous_end = record_end;
            if let Some(index) = r.wire.archive_reference_index
                && !refs.insert((r.source.native_component_type, index))
            {
                return Err(invalid());
            }
            if records.insert(r.key()?, i).is_some() {
                return Err(invalid());
            }
            match (&r.definition_units, r.source.native_component_type) {
                (Some(actual), 11) => {
                    definitions += 1;
                    let stored = source
                        .units
                        .definitions
                        .get(&r.source.id)
                        .ok_or_else(invalid)?;
                    if actual.native_enum != stored.native_enum
                        || actual.name != stored.name
                        || actual.declared_mm_per_unit != stored.declared_mm_per_unit
                        || actual
                            .declared_mm_per_unit
                            .is_some_and(|f| !f.is_finite() || f <= 0.)
                    {
                        return Err(invalid());
                    }
                }
                (None, ty) if ty != 11 => {}
                _ => return Err(invalid()),
            }
        }
        let mut catalog = BTreeSet::new();
        if definitions != source.units.definitions.len()
            || namespace
                .records
                .iter()
                .any(|r| !records.contains_key(&r.key) || !catalog.insert(r.key))
        {
            return Err(invalid());
        }
        crate::current_overlay::canceled(cancel)?;
        Ok(Self {
            dto,
            records,
            owned,
        })
    }
    pub fn record(&self, key: NativeRecordKey) -> Option<&PreparedSourceRecord> {
        self.records.get(&key).map(|i| &self.dto.records[*i])
    }
    pub fn owned_usage(&self) -> Usage {
        self.owned
    }
    pub fn records(&self) -> impl Iterator<Item = &PreparedSourceRecord> {
        self.dto.records.iter()
    }
    pub fn top_level_spans(&self) -> &[PreparedArchiveSpan] {
        &self.dto.top_level_spans
    }
    pub fn original_model_units(&self) -> &PreparedNativeUnits {
        &self.dto.original_units.model
    }
    pub fn original_page_units(&self) -> &PreparedNativeUnits {
        &self.dto.original_units.page
    }
    pub fn source_sha256(&self) -> [u8; 32] {
        self.dto.archive.source_sha256
    }
}
