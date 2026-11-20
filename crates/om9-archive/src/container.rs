// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    Error, Result,
    budget::{Limits, Usage},
    geometry::{Geometry, GeometryKind},
    model::Document,
};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    io::{self, Write},
};

const MAGIC: &[u8; 8] = b"OM9\0\r\n\x1a\n";
pub(crate) const VERSION: u32 = 5;

fn has_unmigrated_brep(document: &Document) -> bool {
    let is_brep = |geometry: &Geometry| matches!(geometry, Geometry::Brep(_) | Geometry::NativeBrep(_));
    document.entities.values().chain(document.definitions.values().flat_map(|definition| &definition.members))
        .any(|entity| is_brep(&entity.geometry))
        || document.results.values().any(|result| is_brep(&result.geometry_snapshot))
}
const HEADER: usize = 16;
const ENTRY: usize = 72;
struct BoundedWriter<'a> {
    data: Vec<u8>,
    limit: usize,
    failed: bool,
    admission: &'a mut dyn FnMut(u64, u64) -> Result<()>,
    admission_error: Option<Error>,
    writes_since_check: usize,
    bytes_since_check: usize,
}

/// Serialize a bounded auxiliary record while charging its live buffer.
pub(crate) fn encode_auxiliary_json<T: serde::Serialize>(value: &T, limits: &Limits,
    admit: &mut dyn FnMut(u64, u64) -> Result<()>) -> Result<Vec<u8>> {
    let limit = usize::try_from(limits.max_bytes).map_err(|_| Error::Overflow("archive address space"))?;
    let mut bounded_admit = |live: u64, pending: u64| {
        if live.checked_add(pending).ok_or(Error::Overflow("archive auxiliary JSON"))? > limits.max_bytes {
            return Err(Error::Budget("archive auxiliary JSON"));
        }
        admit(live, pending)
    };
    let mut writer = BoundedWriter { data: Vec::new(), limit, failed: false, admission: &mut bounded_admit,
        admission_error: None, writes_since_check: 0, bytes_since_check: 0 };
    if let Err(error) = serde_json::to_writer(&mut writer, value) {
        if let Some(error) = writer.admission_error.take() { return Err(error); }
        return Err(if writer.failed { Error::Budget("archive auxiliary JSON") } else { Error::Io(error.to_string()) });
    }
    (writer.admission)(writer.data.capacity() as u64, 0)?;
    Ok(writer.data)
}
impl Write for BoundedWriter<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let Some(needed) = self.data.len().checked_add(bytes.len()) else {
            self.failed = true;
            return Err(io::Error::other("serialization budget"));
        };
        if needed > self.limit {
            self.failed = true;
            return Err(io::Error::other("serialization budget"));
        }
        if needed > self.data.capacity() {
            // JSON emits many tiny writes. Grow geometrically, bounded by the
            // existing serialization budget, instead of reallocating per token.
            let capacity = self.data.capacity().saturating_mul(2).max(needed).max(256).min(self.limit);
            // Reallocation may retain the old allocation until the new one is
            // obtained. Admit that overlap, then reconcile actual capacity.
            if let Err(error) = (self.admission)(self.data.capacity() as u64, capacity as u64) {
                self.admission_error = Some(error);
                return Err(io::Error::other("serialization admission"));
            }
            if self.data.try_reserve_exact(capacity - self.data.len()).is_err()
                || self.data.capacity() > self.limit
            {
                self.failed = true;
                return Err(io::Error::other("serialization allocation"));
            }
            if let Err(error) = (self.admission)(self.data.capacity() as u64, 0) {
                self.admission_error = Some(error);
                return Err(io::Error::other("serialization capacity admission"));
            }
        }
        for chunk in bytes.chunks(64 * 1024) {
            if self.writes_since_check >= 1024 || self.bytes_since_check >= 64 * 1024 {
                if let Err(error) = (self.admission)(self.data.capacity() as u64, 0) {
                    self.admission_error = Some(error);
                    return Err(io::Error::other("serialization checkpoint"));
                }
                self.writes_since_check = 0;
                self.bytes_since_check = 0;
            }
            self.data.extend_from_slice(chunk);
            self.bytes_since_check += chunk.len();
            self.writes_since_check += 1;
        }
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod bounded_writer_tests {
    use super::BoundedWriter;
    use std::io::Write;

    #[test]
    fn small_writes_grow_amortized_without_exceeding_the_limit() {
        let mut admission = |_, _| Ok(());
        let mut writer = BoundedWriter { data: Vec::new(), limit: 10_000, failed: false,
            admission: &mut admission, admission_error: None, writes_since_check: 0, bytes_since_check: 0 };
        let mut changes = 0;
        let mut capacity = 0;
        for _ in 0..10_000 {
            writer.write_all(b"x").unwrap();
            if writer.data.capacity() != capacity {
                changes += 1;
                capacity = writer.data.capacity();
            }
            assert!(capacity <= writer.limit);
        }
        assert!(changes <= 8, "expected geometric growth, got {changes} reallocations");
        assert_eq!(writer.data, vec![b'x'; 10_000]);
        assert!(writer.write_all(b"y").is_err());
        assert!(writer.failed);
        assert_eq!(writer.data.len(), 10_000);
    }
}
pub fn encode(document: &Document, limits: &Limits) -> Result<Vec<u8>> {
    encode_with_admission(document, limits, &mut |_, _| Ok(()))
}
/// Callback reports existing owned buffer capacity and additional pending
/// storage, excluding the caller's original document. Decoder estimates are
/// separate from these exact buffer capacities.
pub(crate) fn encode_with_admission(document: &Document, limits: &Limits,
    admit: &mut dyn FnMut(u64, u64) -> Result<()>) -> Result<Vec<u8>> {
    // The v5 container is ready for geometry whose source is self-contained.
    // BRep output remains disabled until source-only Open has a durable topology
    // binding and opaque native payloads have an explicit safe disposition.
    if has_unmigrated_brep(document) {
        return Err(Error::Unsupported("BRep archive migration to v5 is not available"));
    }
    encode_schema_with_admission(document, limits, admit, VERSION, &[])
}

/// Serialize a prepared BRep candidate. The caller must already have checked
/// the complete source/packet association and all borrowed input ownership.
pub(crate) fn encode_brep_candidate(document: &Document, limits: &Limits,
    admit: &mut dyn FnMut(u64, u64) -> Result<()>, packets: &[([u8; 16], &[u8])]) -> Result<Vec<u8>> {
    encode_schema_with_admission(document, limits, admit, 6, packets)
}

/// Structural v7 candidate. Native payloads are retained in metadata; callers
/// must validate them in an isolated adapter before publishing this container.
pub(crate) fn encode_mixed_brep_candidate(document: &Document, limits: &Limits,
    admit: &mut dyn FnMut(u64, u64) -> Result<()>, packets: &[([u8; 16], &[u8])]) -> Result<Vec<u8>> {
    encode_schema_with_admission(document, limits, admit, 7, packets)
}

fn encode_schema_with_admission(document: &Document, limits: &Limits,
    admit: &mut dyn FnMut(u64, u64) -> Result<()>, version: u32, packets: &[([u8; 16], &[u8])]) -> Result<Vec<u8>> {
    admit(0, document.validation_scratch_usage(limits)?.bytes)?;
    let source = document.validate(limits)?;
    admit(0, 0)?;
    let count = document
        .resources
        .len()
        .checked_add(1).and_then(|n| n.checked_add(packets.len()))
        .ok_or(Error::Overflow("resource chunks"))?;
    if count as u64 > limits.max_nodes {
        return Err(Error::Budget("archive chunks"));
    }
    let count_u32 = u32::try_from(count).map_err(|_| Error::Overflow("archive chunk count"))?;
    let table = count
        .checked_mul(ENTRY)
        .and_then(|n| n.checked_add(HEADER))
        .ok_or(Error::Overflow("archive table"))?;
    let resource_bytes = document.resources.values().try_fold(0u64, |n, r| {
        n.checked_add(r.byte_length())
            .ok_or(Error::Overflow("resource total"))
    })?;
    let packet_bytes = packets.iter().try_fold(0u64, |n, (_, packet)| {
        n.checked_add(packet.len() as u64).ok_or(Error::Overflow("BRep packet total"))
    })?;
    let available = limits
        .max_bytes
        .checked_sub(source.bytes)
        .and_then(|n| n.checked_sub(table as u64))
        .and_then(|n| n.checked_sub(resource_bytes))
        .and_then(|n| n.checked_sub(packet_bytes))
        .ok_or(Error::Budget("archive serialization peak"))?;
    // Both JSON serialization and its final container coexist at this boundary.
    let limit =
        usize::try_from(available / 2).map_err(|_| Error::Overflow("archive address space"))?;
    let mut writer = BoundedWriter {
        data: vec![],
        limit,
        failed: false,
        admission: admit,
        admission_error: None,
        writes_since_check: 0,
        bytes_since_check: 0,
    };
    if let Err(error) = serde_json::to_writer(&mut writer, document) {
        if let Some(error) = writer.admission_error.take() { return Err(error); }
        return Err(if writer.failed {
            Error::Budget("archive serialization peak")
        } else {
            Error::Io(error.to_string())
        });
    }
    let size = table
        .checked_add(writer.data.len())
        .and_then(|n| n.checked_add(usize::try_from(resource_bytes).ok()?))
        .and_then(|n| n.checked_add(usize::try_from(packet_bytes).ok()?))
        .ok_or(Error::Overflow("archive size"))?;
    let mut peak = source;
    peak.add(
        Usage {
            bytes: writer.data.capacity() as u64,
            ..Usage::default()
        },
        limits,
    )?;
    peak.add(Usage::array::<u8>(size)?, limits)?;
    (writer.admission)(writer.data.capacity() as u64, size as u64)?;
    let mut out = Vec::new();
    out.try_reserve_exact(size)
        .map_err(|_| Error::Budget("archive allocation"))?;
    (writer.admission)((writer.data.capacity() as u64).checked_add(out.capacity() as u64)
        .ok_or(Error::Overflow("archive buffer capacity"))?, 0)?;
    out.resize(table, 0);
    out[..8].copy_from_slice(MAGIC);
    out[8..12].copy_from_slice(&version.to_le_bytes());
    out[12..16].copy_from_slice(&count_u32.to_le_bytes());
    out[16..20].copy_from_slice(&1u32.to_le_bytes());
    out[40..48].copy_from_slice(&(table as u64).to_le_bytes());
    out[48..56].copy_from_slice(&(writer.data.len() as u64).to_le_bytes());
    let live = (writer.data.capacity() as u64).checked_add(out.capacity() as u64)
        .ok_or(Error::Overflow("archive buffer capacity"))?;
    let mut digest = Sha256::new();
    for chunk in writer.data.chunks(1024 * 1024) {
        (writer.admission)(live, 0)?;
        digest.update(chunk);
    }
    out[56..88].copy_from_slice(&digest.finalize());
    for chunk in writer.data.chunks(1024 * 1024) {
        (writer.admission)(live, 0)?;
        out.extend_from_slice(chunk);
    }
    for (i, resource) in document.resources.values().enumerate() {
        let position = HEADER + (i + 1) * ENTRY;
        let start = out.len() as u64;
        out[position..position + 4].copy_from_slice(&2u32.to_le_bytes());
        out[position + 8..position + 24].copy_from_slice(resource.id.0.as_bytes());
        out[position + 24..position + 32].copy_from_slice(&start.to_le_bytes());
        out[position + 32..position + 40].copy_from_slice(&resource.byte_length().to_le_bytes());
        out[position + 40..position + 72].copy_from_slice(&resource.sha256());
        for chunk in resource.payload()?.chunks(1024 * 1024) {
            (writer.admission)(live, 0)?;
            out.extend_from_slice(chunk);
        }
    }
    for (i, (id, packet)) in packets.iter().enumerate() {
        let position = HEADER + (1 + document.resources.len() + i) * ENTRY;
        let start = out.len() as u64;
        out[position..position + 4].copy_from_slice(&3u32.to_le_bytes());
        out[position + 8..position + 24].copy_from_slice(id);
        out[position + 24..position + 32].copy_from_slice(&start.to_le_bytes());
        out[position + 32..position + 40].copy_from_slice(&(packet.len() as u64).to_le_bytes());
        let mut hash = Sha256::new();
        for chunk in packet.chunks(1024 * 1024) { (writer.admission)(live, 0)?; hash.update(chunk); }
        out[position + 40..position + 72].copy_from_slice(&hash.finalize());
        for chunk in packet.chunks(1024 * 1024) { (writer.admission)(live, 0)?; out.extend_from_slice(chunk); }
    }
    drop(writer);
    admit(out.capacity() as u64, 0)?;
    Ok(out)
}
fn u32_at(bytes: &[u8], offset: usize) -> Result<u32> {
    Ok(u32::from_le_bytes(
        bytes
            .get(offset..offset + 4)
            .ok_or(Error::Invalid("truncated header"))?
            .try_into()
            .map_err(|_| Error::Invalid("header integer"))?,
    ))
}
fn u64_at(bytes: &[u8], offset: usize) -> Result<u64> {
    Ok(u64::from_le_bytes(
        bytes
            .get(offset..offset + 8)
            .ok_or(Error::Invalid("truncated header"))?
            .try_into()
            .map_err(|_| Error::Invalid("header integer"))?,
    ))
}
/// The stored variant name serde reports for a value this build does not
/// declare. serde states it before its position suffix, for example
/// ``unknown variant `Brep`, expected one of ... at line 1 column 42``.
fn unknown_variant(message: &str) -> Option<&str> {
    message
        .strip_prefix("unknown variant `")?
        .split_once('`')
        .map(|(tag, _)| tag)
}
struct Chunk<'a> {
    kind: u32,
    flags: u32,
    id: &'a [u8],
    start: usize,
    end: usize,
    hash: &'a [u8],
}
pub fn decode(bytes: &[u8], limits: &Limits) -> Result<Document> {
    decode_with_admission(bytes, limits, &mut |_| Ok(()))
}
pub(crate) fn decode_with_admission(bytes: &[u8], limits: &Limits, admit: &mut impl FnMut(u64) -> Result<()>) -> Result<Document> {
    let (document, displays) = decode_open_candidate(bytes, limits, admit)?;
    if !displays.occt.is_empty() || !displays.native.is_empty() {
        return Err(Error::Unsupported("legacy BRep Open requires retained display migration"));
    }
    Ok(document)
}
pub fn decode_legacy_v1(bytes: &[u8], limits: &Limits) -> Result<Document> {
    decode_schema(bytes, limits, 1, &mut |_| Ok(()))
}
pub fn decode_legacy_v2(bytes: &[u8], limits: &Limits) -> Result<Document> {
    decode_schema(bytes, limits, 2, &mut |_| Ok(()))
}
pub fn decode_legacy_v3(bytes: &[u8], limits: &Limits) -> Result<Document> {
    decode_schema(bytes, limits, 3, &mut |_| Ok(()))
}
fn decode_schema(bytes: &[u8], limits: &Limits, version: u32, admit: &mut impl FnMut(u64) -> Result<()>) -> Result<Document> {
    decode_schema_inner(bytes, limits, version, admit, false, None).map(|(document, _)| document)
}

/// Decode v4 into a private source/display migration candidate. The caller
/// must retain the returned displays and account for their ownership before
/// publishing a scene; this does not attest native topology.
pub fn decode_v4_with_displays(bytes: &[u8], limits: &Limits, admit: &mut impl FnMut(u64) -> Result<()>)
    -> Result<(Document, om9_core::legacy_v4::LegacyDisplays)> {
    decode_schema_inner(bytes, limits, 4, admit, true, None)
}

/// Open either current canonical metadata or an explicit v4 migration
/// candidate, retaining legacy displays for scene construction.
pub(crate) fn decode_open_candidate(bytes: &[u8], limits: &Limits, admit: &mut impl FnMut(u64) -> Result<()>)
    -> Result<(Document, om9_core::legacy_v4::LegacyDisplays)> {
    if bytes.get(..8) != Some(MAGIC.as_slice()) { return Err(Error::Invalid("OM9 magic")); }
    match u32_at(bytes, 8)? {
        4 => decode_v4_with_displays(bytes, limits, admit),
        VERSION => decode_schema_inner(bytes, limits, VERSION, admit, false, None),
        _ => Err(Error::Unsupported("OM9 schema version; no implicit migration")),
    }
}

pub(crate) fn decode_brep_candidate<'a>(bytes: &'a [u8], limits: &Limits, admit: &mut impl FnMut(u64) -> Result<()>)
    -> Result<(Document, Vec<([u8; 16], &'a [u8])>)> {
    let mut packets = Vec::new();
    let (document, _) = decode_schema_inner(bytes, limits, 6, admit, false, Some(&mut packets))?;
    if packets.is_empty() || !has_unmigrated_brep(&document) { return Err(Error::Invalid("BRep document packet coverage")); }
    if document.entities.values().chain(document.definitions.values().flat_map(|definition| &definition.members))
        .map(|entity| &entity.geometry).chain(document.results.values().map(|result| &result.geometry_snapshot))
        .any(|geometry| matches!(geometry, Geometry::NativeBrep(_))) {
        return Err(Error::Unsupported("native BRep packet disposition"));
    }
    Ok((document, packets))
}

/// Decode a v7 candidate without establishing native payload validity.
pub(crate) fn decode_mixed_brep_candidate<'a>(bytes: &'a [u8], limits: &Limits, admit: &mut impl FnMut(u64) -> Result<()>)
    -> Result<(Document, Vec<([u8; 16], &'a [u8])>)> {
    let mut packets = Vec::new();
    let (document, _) = decode_schema_inner(bytes, limits, 7, admit, false, Some(&mut packets))?;
    if !document.entities.values().chain(document.definitions.values().flat_map(|definition| &definition.members))
        .map(|entity| &entity.geometry).chain(document.results.values().map(|result| &result.geometry_snapshot))
        .any(|geometry| matches!(geometry, Geometry::NativeBrep(_))) {
        return Err(Error::Invalid("mixed BRep document requires native source"));
    }
    Ok((document, packets))
}

fn decode_schema_inner<'a>(bytes: &'a [u8], limits: &Limits, version: u32, admit: &mut impl FnMut(u64) -> Result<()>,
    migrate_v4: bool, mut packet_sink: Option<&mut Vec<([u8; 16], &'a [u8])>>) -> Result<(Document, om9_core::legacy_v4::LegacyDisplays)> {
    if bytes.get(..8) != Some(MAGIC.as_slice()) {
        return Err(Error::Invalid("OM9 magic"));
    }
    if u32_at(bytes, 8)? != version {
        return Err(Error::Unsupported(
            "OM9 schema version; no implicit migration",
        ));
    }
    if bytes.len() as u64 > limits.max_bytes {
        return Err(Error::Budget("serialized archive bytes"));
    }
    let count = u32_at(bytes, 12)? as usize;
    if count == 0 || count as u64 > limits.max_nodes {
        return Err(Error::Budget("archive chunks"));
    }
    let table_end = count
        .checked_mul(ENTRY)
        .and_then(|n| n.checked_add(HEADER))
        .ok_or(Error::Overflow("chunk table"))?;
    if table_end > bytes.len() {
        return Err(Error::Invalid("truncated chunk table"));
    }
    let mut usage = Usage::array::<Chunk<'_>>(count)?;
    if packet_sink.is_some() { usage.add(Usage::array::<([u8; 16], &[u8])>(count)?, limits)?; }
    usage.add(
        Usage {
            bytes: bytes.len() as u64,
            nodes: count as u64,
            ..Usage::default()
        },
        limits,
    )?;
    admit(usage.bytes)?;
    if let Some(sink) = packet_sink.as_mut() {
        sink.try_reserve_exact(count).map_err(|_| Error::Budget("BRep document packet index"))?;
        let extra = sink.capacity().checked_sub(count).ok_or(Error::Overflow("BRep document packet index"))?;
        usage.add(Usage::array::<([u8; 16], &[u8])>(extra)?, limits)?;
        admit(usage.bytes)?;
    }
    let mut chunks = Vec::new();
    chunks
        .try_reserve_exact(count)
        .map_err(|_| Error::Budget("chunk table allocation"))?;
    for i in 0..count {
        let position = HEADER + i * ENTRY;
        let start = u64_at(bytes, position + 24)?;
        let length = u64_at(bytes, position + 32)?;
        let end = start
            .checked_add(length)
            .ok_or(Error::Overflow("chunk range"))?;
        if start < table_end as u64 || end > bytes.len() as u64 {
            return Err(Error::Invalid("chunk bounds"));
        }
        let start = usize::try_from(start).map_err(|_| Error::Overflow("chunk offset"))?;
        let end = usize::try_from(end).map_err(|_| Error::Overflow("chunk end"))?;
        chunks.push(Chunk {
            kind: u32_at(bytes, position)?,
            flags: u32_at(bytes, position + 4)?,
            id: &bytes[position + 8..position + 24],
            start,
            end,
            hash: &bytes[position + 40..position + 72],
        });
    }
    // Empty resource chunks occupy a real declared identity but no payload
    // interval. Sort them before nonempty chunks at the same offset.
    chunks.sort_unstable_by_key(|c| (c.start, c.end));
    let mut cursor = table_end;
    for chunk in &chunks {
        if chunk.start < cursor {
            return Err(Error::Invalid("chunk overlap"));
        }
        if chunk.start > cursor {
            return Err(Error::Invalid("unreferenced archive gap"));
        }
        cursor = chunk.end;
    }
    if cursor != bytes.len() {
        return Err(Error::Invalid("unreferenced trailing archive bytes"));
    }
    let mut ids = BTreeSet::new();
    let mut metadata = None;
    usage.add(Usage::btree::<&[u8], ()>(count)?, limits)?;
    admit(usage.bytes)?;
    for chunk in &chunks {
        if !ids.insert(chunk.id) {
            return Err(Error::Invalid("duplicate chunk ID"));
        }
        if chunk.flags != 0 {
            return Err(Error::Unsupported("chunk compression/flags"));
        }
        let mut hash = Sha256::new();
        for block in bytes[chunk.start..chunk.end].chunks(1024 * 1024) {
            admit(usage.bytes)?;
            hash.update(block);
        }
        if hash.finalize().as_slice() != chunk.hash {
            return Err(Error::Invalid("chunk checksum"));
        }
        match chunk.kind {
            1 => {
                if metadata.is_some()
                    || chunk.id.iter().any(|b| *b != 0)
                    || chunk.start == chunk.end
                {
                    return Err(Error::Invalid("metadata chunk identity"));
                }
                metadata = Some(&bytes[chunk.start..chunk.end]);
            }
            2 if version >= 2 && chunk.id.iter().any(|b| *b != 0) => {}
            3 if (version == 6 || version == 7) && chunk.id.iter().any(|b| *b != 0) && chunk.start != chunk.end => {
                let sink = packet_sink.as_mut().ok_or(Error::Invalid("BRep document packet owner"))?;
                let id: [u8; 16] = chunk.id.try_into().map_err(|_| Error::Invalid("BRep document chunk identity"))?;
                sink.push((id, &bytes[chunk.start..chunk.end]));
            }
            _ => return Err(Error::Unsupported("chunk kind without a schema/adapter")),
        }
    }
    let metadata = metadata.ok_or(Error::Missing("document metadata chunk"))?;
    let mut preflight_bytes = usage.bytes;
    if version < 4 {
        crate::json_admission::check_legacy_document(metadata, bytes.len(), limits)?;
    } else {
        let chunk_bytes = usage.bytes.checked_sub(bytes.len() as u64)
            .ok_or(Error::Overflow("archive chunk admission"))?;
        crate::json_admission::check_document_usage_with_admission(metadata, bytes.len(), limits, &mut |estimated| {
            let total = estimated.checked_add(chunk_bytes).ok_or(Error::Overflow("archive preflight admission"))?;
            if total > limits.max_bytes { return Err(Error::Budget("archive preflight bytes")); }
            preflight_bytes = preflight_bytes.max(total);
            admit(total)
        })?;
    }
    let map_decode_error = |error: serde_json::Error| {
        let message = error.to_string();
        if message.starts_with("noncanonical UUID map key") {
            Error::Invalid("noncanonical UUID map key")
        } else if message.starts_with("duplicate typed UUID map key") {
            Error::Invalid("duplicate typed UUID map key")
        } else if let Some(tag) = unknown_variant(&message) {
            // A declared entity kind this build cannot construct (Brep/BRep/
            // B-rep) is a capability limit, not malformed metadata, and is never
            // silently downgraded to a nearby kind. Any other unrecognised
            // variant is an unsupported document variant rather than bad shape.
            if matches!(
                GeometryKind::from_tag(tag),
                Err(Error::Unsupported("brep geometry"))
            ) {
                Error::Unsupported("brep geometry")
            } else {
                Error::Unsupported("document variant")
            }
        } else {
            Error::Invalid("typed document metadata/schema")
        }
    };
    let mut migration_peak = 0u64;
    let (mut document, displays) = if migrate_v4 {
        let legacy: om9_core::legacy_v4::DocumentV4 = serde_json::from_slice(metadata).map_err(map_decode_error)?;
        let overlap = legacy.split_allocation_usage(limits)?.bytes;
        let (occt_count, native_count) = legacy.display_counts();
        let requested_vectors = Usage::array::<(om9_core::brep::CanonicalBrep, om9_core::brep::BrepRepresentation)>(occt_count)?.bytes
            .checked_add(Usage::array::<(om9_core::geometry::NativeBrepSource, om9_core::geometry::NativeBrepRepresentation)>(native_count)?.bytes)
            .ok_or(Error::Overflow("legacy display vectors"))?;
        let total = preflight_bytes.checked_add(overlap).ok_or(Error::Overflow("legacy document migration"))?;
        if total > limits.max_bytes { return Err(Error::Budget("legacy document migration")); }
        admit(total)?;
        legacy.validate_displays_admitted(limits, |live, pending| {
            let peak = total.checked_add(live).and_then(|n| n.checked_add(pending))
                .ok_or(Error::Overflow("legacy display validation"))?;
            if peak > limits.max_bytes { return Err(Error::Budget("legacy display validation")); }
            admit(peak)
        })?;
        let (document, displays) = legacy.split(om9_core::legacy_v4::LegacyDisplays::default())?;
        let extra = displays.vector_storage_bytes()?.saturating_sub(requested_vectors);
        migration_peak = total.checked_add(extra).ok_or(Error::Overflow("legacy display vector capacity"))?;
        if migration_peak > limits.max_bytes { return Err(Error::Budget("legacy document migration")); }
        admit(migration_peak)?;
        (document, displays)
    } else {
        (serde_json::from_slice(metadata).map_err(map_decode_error)?, om9_core::legacy_v4::LegacyDisplays::default())
    };
    // A current-version file cannot establish the durable source/topology
    // binding or native payload disposition merely by containing a BRep.
    // Keep v4 migration separate so its retained displays are not discarded.
    if version == VERSION && has_unmigrated_brep(&document) {
        return Err(Error::Unsupported("BRep archive migration to v5 is not available"));
    }
    if version == 1 && !document.resources.is_empty() {
        return Err(Error::Invalid("legacy schema has no resource ownership"));
    }
    if version < 3
        && (!document.retained_sources.is_empty() || !document.import_namespaces.is_empty())
    {
        return Err(Error::Invalid(
            "legacy schema has no native source ownership",
        ));
    }
    // Admit the entire hydrated document alongside the serialized input and
    // retained table bookkeeping before any resource allocation.
    let mut validation_peak = usage;
    validation_peak.add(document.validation_peak_estimate(limits, false)?, limits)?;
    let admitted_validation = validation_peak.bytes.checked_add(migration_peak)
        .ok_or(Error::Overflow("legacy document validation overlap"))?;
    if admitted_validation > limits.max_bytes { return Err(Error::Budget("legacy document validation overlap")); }
    admit(admitted_validation)?;
    usage.add(document.validate_manifest(limits)?, limits)?;
    for chunk in chunks.iter().filter(|c| c.kind == 2) {
        let id = om9_core::identity::ResourceId(
            uuid::Uuid::from_slice(chunk.id)
                .map_err(|_| Error::Invalid("resource chunk identity"))?,
        );
        let resource = document
            .resources
            .get_mut(&id)
            .ok_or(Error::Invalid("unreferenced resource chunk"))?;
        if chunk.hash != resource.sha256()
            || (chunk.end - chunk.start) as u64 != resource.byte_length()
        {
            return Err(Error::Invalid("resource descriptor/chunk mismatch"));
        }
        resource.attach_payload(&bytes[chunk.start..chunk.end], limits)?;
    }
    document.validate(limits)?;
    Ok((document, displays))
}
