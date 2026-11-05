// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    Error, Result,
    budget::{Limits, Usage},
    geometry::GeometryKind,
    model::Document,
};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    io::{self, Write},
};

const MAGIC: &[u8; 8] = b"OM9\0\r\n\x1a\n";
const VERSION: u32 = 4;
const HEADER: usize = 16;
const ENTRY: usize = 72;
struct BoundedWriter<'a> {
    data: Vec<u8>,
    limit: usize,
    failed: bool,
    admission: &'a mut dyn FnMut(u64, u64) -> Result<()>,
    admission_error: Option<Error>,
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
        self.data.extend_from_slice(bytes);
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
            admission: &mut admission, admission_error: None };
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
    admit(0, document.validation_scratch_usage(limits)?.bytes)?;
    let source = document.validate(limits)?;
    admit(0, 0)?;
    let count = document
        .resources
        .len()
        .checked_add(1)
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
    let available = limits
        .max_bytes
        .checked_sub(source.bytes)
        .and_then(|n| n.checked_sub(table as u64))
        .and_then(|n| n.checked_sub(resource_bytes))
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
    out[8..12].copy_from_slice(&VERSION.to_le_bytes());
    out[12..16].copy_from_slice(&count_u32.to_le_bytes());
    out[16..20].copy_from_slice(&1u32.to_le_bytes());
    out[40..48].copy_from_slice(&(table as u64).to_le_bytes());
    out[48..56].copy_from_slice(&(writer.data.len() as u64).to_le_bytes());
    out[56..88].copy_from_slice(&Sha256::digest(&writer.data));
    out.extend_from_slice(&writer.data);
    for (i, resource) in document.resources.values().enumerate() {
        let position = HEADER + (i + 1) * ENTRY;
        let start = out.len() as u64;
        out[position..position + 4].copy_from_slice(&2u32.to_le_bytes());
        out[position + 8..position + 24].copy_from_slice(resource.id.0.as_bytes());
        out[position + 24..position + 32].copy_from_slice(&start.to_le_bytes());
        out[position + 32..position + 40].copy_from_slice(&resource.byte_length().to_le_bytes());
        out[position + 40..position + 72].copy_from_slice(&resource.sha256());
        out.extend_from_slice(resource.payload()?);
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
    decode_schema(bytes, limits, VERSION, &mut |_| Ok(()))
}
pub(crate) fn decode_with_admission(bytes: &[u8], limits: &Limits, admit: &mut impl FnMut(u64) -> Result<()>) -> Result<Document> {
    decode_schema(bytes, limits, VERSION, admit)
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
    usage.add(
        Usage {
            bytes: bytes.len() as u64,
            nodes: count as u64,
            ..Usage::default()
        },
        limits,
    )?;
    admit(usage.bytes)?;
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
        if Sha256::digest(&bytes[chunk.start..chunk.end]).as_slice() != chunk.hash {
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
            _ => return Err(Error::Unsupported("chunk kind without a schema/adapter")),
        }
    }
    let metadata = metadata.ok_or(Error::Missing("document metadata chunk"))?;
    if version < 4 {
        crate::json_admission::check_legacy_document(metadata, bytes.len(), limits)?;
    } else {
        let chunk_bytes = usage.bytes.checked_sub(bytes.len() as u64)
            .ok_or(Error::Overflow("archive chunk admission"))?;
        crate::json_admission::check_usage_with_admission(metadata, bytes.len(), limits, &mut |estimated| {
            let total = estimated.checked_add(chunk_bytes).ok_or(Error::Overflow("archive preflight admission"))?;
            if total > limits.max_bytes { return Err(Error::Budget("archive preflight bytes")); }
            admit(total)
        })?;
    }
    let mut document: Document = serde_json::from_slice(metadata).map_err(|error| {
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
    })?;
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
    admit(validation_peak.bytes)?;
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
    Ok(document)
}
