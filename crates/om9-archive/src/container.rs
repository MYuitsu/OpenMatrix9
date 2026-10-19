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
struct BoundedWriter {
    data: Vec<u8>,
    limit: usize,
    failed: bool,
}
impl Write for BoundedWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self
            .data
            .len()
            .checked_add(bytes.len())
            .is_none_or(|n| n > self.limit)
        {
            self.failed = true;
            return Err(io::Error::other("serialization budget"));
        }
        self.data
            .try_reserve_exact(bytes.len())
            .map_err(|_| io::Error::other("serialization allocation"))?;
        self.data.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
pub fn encode(document: &Document, limits: &Limits) -> Result<Vec<u8>> {
    let source = document.validate(limits)?;
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
    };
    if let Err(error) = serde_json::to_writer(&mut writer, document) {
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
    let mut out = Vec::new();
    out.try_reserve_exact(size)
        .map_err(|_| Error::Budget("archive allocation"))?;
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
    decode_schema(bytes, limits, VERSION)
}
pub fn decode_legacy_v1(bytes: &[u8], limits: &Limits) -> Result<Document> {
    decode_schema(bytes, limits, 1)
}
pub fn decode_legacy_v2(bytes: &[u8], limits: &Limits) -> Result<Document> {
    decode_schema(bytes, limits, 2)
}
pub fn decode_legacy_v3(bytes: &[u8], limits: &Limits) -> Result<Document> {
    decode_schema(bytes, limits, 3)
}
fn decode_schema(bytes: &[u8], limits: &Limits, version: u32) -> Result<Document> {
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
        crate::json_admission::check(metadata, bytes.len(), limits)?;
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
