// SPDX-License-Identifier: LGPL-2.1-or-later
//! Preflight and decode a standalone source/topology record, not a project codec.
use om9_core::{Error, Result, brep::{CanonicalBrepRecord, CanonicalBrepRef}, budget::{Limits, Usage, memory::MemoryClass},
    transaction::{ExternalMemoryAccount, ExternalMemoryReservation}};
use sha2::{Digest, Sha256};

const PACKET_MAGIC: &[u8; 8] = b"OM9BRP\x01\0";
const PACKET_HEADER: usize = 88;
pub use crate::atomic_file::PreparedArchive;

/// Stage and atomically publish an already encoded v6 candidate. The caller
/// must verify every native packet against its source, both before staging and
/// when this callback is invoked on the independently read staging copy.
/// This is a low-level publication primitive, not native attestation.
#[doc(hidden)]
pub fn publish_verified_document_candidate(path: &std::path::Path, bytes: &Vec<u8>, limits: &Limits,
    cancel: &om9_core::transaction::Cancellation, mut admit: impl FnMut(u64, u64) -> Result<()>,
    mut verify: impl FnMut(&[u8]) -> Result<()>) -> Result<()> {
    prepare_verified_document_candidate(path, bytes, limits, cancel, &mut admit, &mut verify)?.publish(cancel)
}

/// Verify and stage a v6 archive without changing its destination. The
/// caller must revalidate its live operation before publishing the result.
#[doc(hidden)]
pub fn prepare_verified_document_candidate(path: &std::path::Path, bytes: &Vec<u8>, limits: &Limits,
    cancel: &om9_core::transaction::Cancellation, mut admit: impl FnMut(u64, u64) -> Result<()>,
    mut verify: impl FnMut(&[u8]) -> Result<()>) -> Result<PreparedArchive> {
    prepare_verified_candidate_version(path, bytes, limits, cancel, &mut admit, &mut verify, 6)
}

/// Stage a v7 candidate only after the caller validates both kinds of native
/// source on the original bytes and on the independent staging readback.
#[doc(hidden)]
pub fn prepare_verified_mixed_document_candidate(path: &std::path::Path, bytes: &Vec<u8>, limits: &Limits,
    cancel: &om9_core::transaction::Cancellation, mut admit: impl FnMut(u64, u64) -> Result<()>,
    mut verify: impl FnMut(&[u8]) -> Result<()>) -> Result<PreparedArchive> {
    prepare_verified_candidate_version(path, bytes, limits, cancel, &mut admit, &mut verify, 7)
}

fn prepare_verified_candidate_version(path: &std::path::Path, bytes: &Vec<u8>, limits: &Limits,
    cancel: &om9_core::transaction::Cancellation, admit: &mut dyn FnMut(u64, u64) -> Result<()>,
    verify: &mut dyn FnMut(&[u8]) -> Result<()>, version: u32) -> Result<PreparedArchive> {
    #[cfg(not(windows))]
    {
        let _ = (path, bytes, limits, cancel, admit, verify, version);
        return Err(Error::Unsupported("verified BRep staging requires Windows"));
    }
    #[cfg(windows)]
    {
    if bytes.len() < 12 || &bytes[..8] != b"OM9\0\r\n\x1a\n"
        || u32::from_le_bytes(bytes[8..12].try_into().expect("header checked")) != version {
        return Err(Error::Invalid("BRep document candidate version"));
    }
    if bytes.capacity() as u64 > limits.max_bytes { return Err(Error::Budget("BRep document candidate")); }
    if cancel.is_canceled() { return Err(Error::Canceled); }
    verify(bytes)?;
    if cancel.is_canceled() { return Err(Error::Canceled); }
        crate::atomic_file::prepare_encoded(path, bytes, limits, cancel, admit, &mut |reread, _| verify(reread))
    }
}

/// Serialize an auxiliary source/ShapeSet pair. The caller owns and accounts
/// for both inputs, and must retain the returned buffer's reservation. This
/// envelope is not a document archive and does not attest native topology.
pub fn encode_packet_with_admission(source: &CanonicalBrepRef<'_>, packet: &[u8], limits: &Limits,
    mut admit: impl FnMut(u64, u64) -> Result<()>) -> Result<Vec<u8>> {
    if packet.is_empty() { return Err(Error::Invalid("BRep packet")); }
    let json = crate::container::encode_auxiliary_json(source, limits, &mut admit)?;
    let size = PACKET_HEADER.checked_add(json.len()).and_then(|n| n.checked_add(packet.len()))
        .ok_or(Error::Overflow("BRep packet envelope"))?;
    if size as u64 > limits.max_bytes { return Err(Error::Budget("BRep packet envelope")); }
    if (json.capacity() as u64).checked_add(size as u64).ok_or(Error::Overflow("BRep packet envelope"))? > limits.max_bytes {
        return Err(Error::Budget("BRep packet envelope"));
    }
    admit(json.capacity() as u64, size as u64)?;
    let mut output = Vec::new();
    output.try_reserve_exact(size).map_err(|_| Error::Budget("BRep packet envelope"))?;
    let live = (json.capacity() as u64).checked_add(output.capacity() as u64)
        .ok_or(Error::Overflow("BRep packet envelope"))?;
    if live > limits.max_bytes { return Err(Error::Budget("BRep packet envelope")); }
    admit(live, 0)?;
    output.extend_from_slice(PACKET_MAGIC);
    output.extend_from_slice(&(json.len() as u64).to_le_bytes());
    output.extend_from_slice(&(packet.len() as u64).to_le_bytes());
    let mut hashes = [[0u8; 32]; 2];
    for (index, input) in [json.as_slice(), packet].into_iter().enumerate() {
        let mut hash = Sha256::new();
        for chunk in input.chunks(1024 * 1024) { admit(live, 0)?; hash.update(chunk); }
        hashes[index] = hash.finalize().into();
    }
    output.extend_from_slice(&hashes[0]);
    output.extend_from_slice(&hashes[1]);
    for input in [json.as_slice(), packet] {
        for chunk in input.chunks(1024 * 1024) { admit(live, 0)?; output.extend_from_slice(chunk); }
    }
    drop(json);
    admit(output.capacity() as u64, 0)?;
    Ok(output)
}

/// A validated portable record and a borrowed packet. The caller keeps the
/// envelope bytes accounted and alive; the contained record owns its receipt.
pub struct ReservedBrepPacket<'a> { record: ReservedCanonicalBrep, packet: &'a [u8], envelope: &'a [u8] }
impl ReservedBrepPacket<'_> {
    pub fn record(&self) -> &ReservedCanonicalBrep { &self.record }
    pub fn packet(&self) -> &[u8] { self.packet }
    pub fn envelope(&self) -> &[u8] { self.envelope }
}

/// Parse only the portable envelope and source record. Native parsing and the
/// association of UUIDs to native shapes must still be checked by the worker.
pub fn decode_packet_reserved<'a>(input: &'a [u8], limits: &Limits, account: &ExternalMemoryAccount,
    reservation: ExternalMemoryReservation, cancelled: impl FnMut() -> bool) -> Result<ReservedBrepPacket<'a>> {
    decode_packet_growing(input, limits, account, reservation, |_, _| Err(Error::Budget("canonical BRep reservation")), cancelled)
}

/// Decode under an account-owned reservation that grows before each admitted
/// parser allocation. The caller keeps the input separately accounted.
pub fn decode_packet_growing<'a>(input: &'a [u8], limits: &Limits, account: &ExternalMemoryAccount,
    reservation: ExternalMemoryReservation, grow: impl FnMut(&mut ExternalMemoryReservation, u64) -> Result<()>,
    mut cancelled: impl FnMut() -> bool) -> Result<ReservedBrepPacket<'a>> {
    if cancelled() { return Err(Error::Canceled); }
    if input.len() as u64 > limits.max_bytes { return Err(Error::Budget("BRep packet envelope")); }
    if input.len() < PACKET_HEADER || &input[..8] != PACKET_MAGIC { return Err(Error::Invalid("BRep packet envelope")); }
    let read_len = |offset: usize| -> Result<usize> {
        let raw = u64::from_le_bytes(input[offset..offset + 8].try_into().expect("header checked"));
        usize::try_from(raw).map_err(|_| Error::Overflow("BRep packet length"))
    };
    let json_len = read_len(8)?;
    let packet_len = read_len(16)?;
    let split = PACKET_HEADER.checked_add(json_len).ok_or(Error::Overflow("BRep packet length"))?;
    let end = split.checked_add(packet_len).ok_or(Error::Overflow("BRep packet length"))?;
    if json_len == 0 || packet_len == 0 || end != input.len() { return Err(Error::Invalid("BRep packet length")); }
    let json = &input[PACKET_HEADER..split];
    let packet = &input[split..end];
    for (data, expected) in [(json, &input[24..56]), (packet, &input[56..88])] {
        let mut hash = Sha256::new();
        for chunk in data.chunks(1024 * 1024) {
            if cancelled() { return Err(Error::Canceled); }
            hash.update(chunk);
        }
        if hash.finalize().as_slice() != expected { return Err(Error::Invalid("BRep packet checksum")); }
    }
    let record = decode_reserved_growing(json, limits, account, reservation, grow, cancelled)?;
    Ok(ReservedBrepPacket { record, packet, envelope: input })
}

/// Prepare an experimental v6 document container with one verified portable
/// packet per distinct canonical BRep association. The input document,
/// packet envelopes and decoded records remain caller-owned and accounted.
/// This only checks portable integrity, not native topology provenance; it is
/// not an Open implementation or permission to publish a document.
#[doc(hidden)]
pub fn encode_document_candidate(document: &om9_core::model::Document, packets: &[ReservedBrepPacket<'_>],
    account: &ExternalMemoryAccount, limits: &Limits, mut admit: impl FnMut(u64, u64) -> Result<()>) -> Result<Vec<u8>> {
    encode_document_candidate_inner(document, packets, account, limits, &mut admit, false)
}

/// Structural v7 candidate only. The caller must independently validate each
/// distinct native payload in a contained adapter before using these bytes.
#[doc(hidden)]
pub fn encode_mixed_document_candidate(document: &om9_core::model::Document, packets: &[ReservedBrepPacket<'_>],
    account: &ExternalMemoryAccount, limits: &Limits, mut admit: impl FnMut(u64, u64) -> Result<()>) -> Result<Vec<u8>> {
    encode_document_candidate_inner(document, packets, account, limits, &mut admit, true)
}

fn encode_document_candidate_inner(document: &om9_core::model::Document, packets: &[ReservedBrepPacket<'_>],
    account: &ExternalMemoryAccount, limits: &Limits, admit: &mut dyn FnMut(u64, u64) -> Result<()>, mixed: bool) -> Result<Vec<u8>> {
    use om9_core::{brep::CanonicalBrep, geometry::Geometry};
    use std::collections::{BTreeMap, BTreeSet};
    let geometry = || document.entities.values().chain(document.definitions.values().flat_map(|definition| &definition.members))
        .map(|entity| &entity.geometry).chain(document.results.values().map(|result| &result.geometry_snapshot));
    let count = geometry().filter(|item| matches!(item, Geometry::Brep(_) | Geometry::NativeBrep(_))).count();
    if count == 0 || count as u64 > limits.max_nodes || packets.len() > count {
        return Err(Error::Invalid("BRep document packet count"));
    }
    let mut index_bytes = Usage::btree::<[u8; 32], &CanonicalBrep>(count)?.bytes
        .checked_add(Usage::btree::<[u8; 16], ()>(packets.len())?.bytes)
        .and_then(|n| n.checked_add(Usage::array::<([u8; 16], &[u8])>(packets.len()).ok()?.bytes))
        .ok_or(Error::Overflow("BRep document packet index"))?;
    if index_bytes > limits.max_bytes { return Err(Error::Budget("BRep document packet index")); }
    admit(0, index_bytes)?;
    let mut sources = BTreeMap::<[u8; 32], &CanonicalBrep>::new();
    let mut native_count = 0usize;
    for item in geometry() {
        match item {
            Geometry::NativeBrep(_) if mixed => { native_count += 1; }
            Geometry::NativeBrep(_) => return Err(Error::Unsupported("native BRep packet disposition")),
            Geometry::Brep(source) => {
                if source.has_coupled_display_diagnostic() { return Err(Error::Unsupported("coupled BRep diagnostic archive")); }
                let binding = source.canonical_fingerprints().1;
                if let Some(previous) = sources.insert(binding, source) {
                    if previous.source() != source.source() || previous.topology() != source.topology() {
                        return Err(Error::Invalid("BRep document binding collision"));
                    }
                }
            }
            _ => {}
        }
    }
    if mixed && native_count == 0 { return Err(Error::Invalid("mixed BRep document requires native source")); }
    if sources.len() != packets.len() { return Err(Error::Invalid("BRep document packet coverage")); }
    let mut chunks = Vec::new();
    chunks.try_reserve_exact(packets.len()).map_err(|_| Error::Budget("BRep document packet index"))?;
    let extra = Usage::array::<([u8; 16], &[u8])>(chunks.capacity().saturating_sub(packets.len()))?.bytes;
    index_bytes = index_bytes.checked_add(extra).ok_or(Error::Overflow("BRep document packet index"))?;
    if index_bytes > limits.max_bytes { return Err(Error::Budget("BRep document packet index")); }
    admit(index_bytes, 0)?;
    let mut chunk_ids = BTreeSet::new();
    for packet in packets {
        if !packet.record().belongs_to(account) { return Err(Error::Invalid("BRep record account")); }
        let record = packet.record().record();
        let binding = record.binding_fingerprint();
        let source = sources.remove(&binding).ok_or(Error::Invalid("BRep document packet binding"))?;
        if source.source() != record.source() || source.topology() != record.topology()
            || source.canonical_fingerprints().0 != record.fingerprint() {
            return Err(Error::Invalid("BRep document packet binding"));
        }
        let mut id = [0u8; 16]; id.copy_from_slice(&binding[..16]);
        if id == [0; 16] || !chunk_ids.insert(id)
            || document.resources.contains_key(&om9_core::identity::ResourceId(uuid::Uuid::from_bytes(id))) {
            return Err(Error::Invalid("BRep document chunk identity"));
        }
        chunks.push((id, packet.envelope()));
    }
    if !sources.is_empty() { return Err(Error::Invalid("BRep document packet coverage")); }
    let remaining = limits.remaining_after(Usage { bytes: index_bytes, ..Usage::default() })?;
    let mut admit_container = |live, pending| {
        admit(index_bytes.checked_add(live).ok_or(Error::Overflow("BRep document admission"))?, pending)
    };
    let result = if mixed {
        crate::container::encode_mixed_brep_candidate(document, &remaining, &mut admit_container, &chunks)?
    } else {
        crate::container::encode_brep_candidate(document, &remaining, &mut admit_container, &chunks)?
    };
    drop((sources, chunk_ids, chunks));
    admit(result.capacity() as u64, 0)?;
    Ok(result)
}

/// A structural v6 candidate. Its document has no restored displays or native
/// topology proof; the packet chunks are untrusted slices of the input owner.
pub struct BrepDocumentCandidate<'a> {
    document: om9_core::model::Document,
    packets: Vec<([u8; 16], &'a [u8])>,
}
impl<'a> BrepDocumentCandidate<'a> {
    pub fn document(&self) -> &om9_core::model::Document { &self.document }
    pub fn packets(&self) -> &[([u8; 16], &'a [u8])] { &self.packets }
    /// Transfer the detached document only; callers still need restored,
    /// accounted sources and displays before publication.
    pub fn into_document(self) -> om9_core::model::Document { self.document }
    /// The caller must admit validation scratch before invoking this method.
    pub fn owned_storage_bytes(&self, limits: &Limits) -> Result<u64> {
        self.document.validate(limits)?.bytes.checked_add(Usage::array::<([u8; 16], &[u8])>(self.packets.capacity())?.bytes)
            .and_then(|bytes| bytes.checked_add(std::mem::size_of::<Self>() as u64))
            .ok_or(Error::Overflow("BRep document candidate storage"))
    }

    /// Check every separately decoded, account-owned packet against the exact
    /// chunk and every canonical source in the document. This still does not
    /// attest the native payload or authorize scene publication.
    pub fn validate_packets(&self, packets: &[ReservedBrepPacket<'_>], account: &ExternalMemoryAccount,
        limits: &Limits, mut admit: impl FnMut(u64, u64) -> Result<()>) -> Result<()> {
        use om9_core::{brep::CanonicalBrep, geometry::Geometry};
        use std::collections::BTreeMap;
        if packets.len() != self.packets.len() {
            return Err(Error::Invalid("BRep document packet coverage"));
        }
        let geometry = || self.document.entities.values().chain(self.document.definitions.values().flat_map(|definition| &definition.members))
            .map(|entity| &entity.geometry).chain(self.document.results.values().map(|result| &result.geometry_snapshot));
        let count = geometry().filter(|item| matches!(item, Geometry::Brep(_))).count();
        if count as u64 > limits.max_nodes { return Err(Error::Invalid("BRep document packet count")); }
        let scratch = Usage::btree::<[u8; 32], &CanonicalBrep>(count)?.bytes;
        if scratch > limits.max_bytes { return Err(Error::Budget("BRep document packet index")); }
        admit(0, scratch)?;
        let mut sources = BTreeMap::<[u8; 32], &CanonicalBrep>::new();
        for item in geometry() {
            if let Geometry::Brep(source) = item {
                if let Some(previous) = sources.insert(source.canonical_fingerprints().1, source) {
                    if previous.source() != source.source() || previous.topology() != source.topology() {
                        return Err(Error::Invalid("BRep document binding collision"));
                    }
                }
            }
        }
        admit(scratch, 0)?;
        if sources.len() != packets.len() { return Err(Error::Invalid("BRep document packet coverage")); }
        for ((id, bytes), packet) in self.packets.iter().zip(packets) {
            admit(scratch, 0)?;
            if !packet.record().belongs_to(account) { return Err(Error::Invalid("BRep record account")); }
            if !std::ptr::eq(bytes.as_ptr(), packet.envelope().as_ptr()) || bytes.len() != packet.envelope().len() {
                return Err(Error::Invalid("BRep document packet owner"));
            }
            let record = packet.record().record();
            let binding = record.binding_fingerprint();
            if id.as_slice() != &binding[..16] { return Err(Error::Invalid("BRep document chunk identity")); }
            let source = sources.remove(&binding).ok_or(Error::Invalid("BRep document packet binding"))?;
            if source.source() != record.source() || source.topology() != record.topology()
                || source.canonical_fingerprints().0 != record.fingerprint() {
                return Err(Error::Invalid("BRep document packet binding"));
            }
        }
        if !sources.is_empty() { return Err(Error::Invalid("BRep document packet coverage")); }
        drop(sources);
        admit(0, 0)
    }
}

/// Decode only a detached structural candidate. The caller accounts for the
/// input and decoded candidate, validates every packet/source association, and
/// obtains contained native restore results before any scene or publication.
#[doc(hidden)]
pub fn decode_document_candidate_with_admission<'a>(input: &'a [u8], limits: &Limits,
    mut admit: impl FnMut(u64) -> Result<()>) -> Result<BrepDocumentCandidate<'a>> {
    let (document, packets) = crate::container::decode_brep_candidate(input, limits, &mut admit)?;
    Ok(BrepDocumentCandidate { document, packets })
}

/// Decode v7 structure only; validate the packet/source association and every
/// native payload in a contained adapter before publication or scene use.
#[doc(hidden)]
pub fn decode_mixed_document_candidate_with_admission<'a>(input: &'a [u8], limits: &Limits,
    mut admit: impl FnMut(u64) -> Result<()>) -> Result<BrepDocumentCandidate<'a>> {
    let (document, packets) = crate::container::decode_mixed_brep_candidate(input, limits, &mut admit)?;
    Ok(BrepDocumentCandidate { document, packets })
}

/// The decoded record and the account reservation that owns its allocations.
/// Its borrowed record cannot outlive this guard.
pub struct ReservedCanonicalBrep {
    record: CanonicalBrepRecord,
    reservation: ExternalMemoryReservation,
}
impl ReservedCanonicalBrep {
    pub fn record(&self) -> &CanonicalBrepRecord { &self.record }
    pub fn reserved_bytes(&self) -> u64 { self.reservation.bytes() }
    /// A record may only participate in work charged to its owning account.
    pub fn belongs_to(&self, account: &ExternalMemoryAccount) -> bool { self.reservation.belongs_to(account) }
}

/// Decode under a caller-supplied, already admitted peak reservation. The
/// input must remain accounted separately by its owner. The conservative
/// preflight also includes the input, so this may temporarily double-charge.
/// Cancellation and all failures drop the reservation without publishing a
/// record. The caller must not share an unaccounted clone of the record.
pub fn decode_reserved(input: &[u8], limits: &Limits, account: &ExternalMemoryAccount,
    reservation: ExternalMemoryReservation, cancelled: impl FnMut() -> bool) -> Result<ReservedCanonicalBrep> {
    decode_reserved_growing(input, limits, account, reservation, |_, _| Err(Error::Budget("canonical BRep reservation")), cancelled)
}

fn decode_reserved_growing(input: &[u8], limits: &Limits, account: &ExternalMemoryAccount,
    mut reservation: ExternalMemoryReservation, mut grow: impl FnMut(&mut ExternalMemoryReservation, u64) -> Result<()>,
    mut cancelled: impl FnMut() -> bool) -> Result<ReservedCanonicalBrep> {
    if reservation.class() != MemoryClass::CpuCache || !reservation.belongs_to(account)
        || reservation.pending_bytes() != reservation.bytes() || !reservation.physical_sampled() {
        return Err(Error::Invalid("canonical BRep reservation"));
    }
    let (record, usage) = decode_with_admission(input, limits, |peak| {
        if cancelled() { return Err(Error::Canceled); }
        if peak > reservation.bytes() {
            let extra = peak - reservation.bytes();
            grow(&mut reservation, extra)?;
            if peak > reservation.bytes() { return Err(Error::Budget("canonical BRep reservation")); }
        }
        reservation.recheck_pending()
    })?;
    if cancelled() { return Err(Error::Canceled); }
    reservation.set_allocated_bytes(usage.bytes)?;
    reservation.shrink_to(usage.bytes)?;
    Ok(ReservedCanonicalBrep { record, reservation })
}

/// The caller owns `input` and must keep a reservation for the returned
/// record. `admit` receives the combined input, parser and record peak; it
/// must fail before a requested peak can exceed the caller's live budget.
/// This checks portable structure and integrity, not OCCT validity or ID
/// provenance, and does not establish a v5 project format.
fn decode_with_admission(input: &[u8], limits: &Limits,
    mut admit: impl FnMut(u64) -> Result<()>) -> Result<(CanonicalBrepRecord, Usage)> {
    let estimated = crate::json_admission::check_usage_with_admission(input, input.len(), limits, &mut admit)?;
    let record: CanonicalBrepRecord = serde_json::from_slice(input)
        .map_err(|_| Error::Invalid("canonical BRep record schema"))?;
    let input_bytes = u64::try_from(input.len()).map_err(|_| Error::Overflow("canonical BRep input"))?;
    let usage = record.validate_admitted(limits, |live, pending| {
        let peak = input_bytes.checked_add(live).and_then(|n| n.checked_add(pending))
            .ok_or(Error::Overflow("canonical BRep admission"))?;
        if peak > limits.max_bytes { return Err(Error::Budget("canonical BRep bytes")); }
        admit(peak.max(estimated.bytes))
    })?;
    Ok((record, usage))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

    fn fixture() -> Vec<u8> {
        let source = "CASCADE Topology fixture";
        let digest: [u8; 32] = Sha256::digest(source.as_bytes()).into();
        let topology = om9_core::brep::BrepTopology { faces: vec![uuid::Uuid::from_u128(1)],
            edges: vec![], vertex_ids: vec![], face_edges: vec![] };
        let binding = om9_core::brep::source_topology_fingerprint(digest, &topology);
        serde_json::to_vec(&serde_json::json!({
            "source": source, "source_sha256": digest, "binding_sha256": binding,
            "topology": topology
        })).unwrap()
    }
    #[test]
    fn preflight_refuses_before_typed_decode_and_rejects_corruption() {
        let input = fixture();
        let limits = Limits::default();
        let mut calls = 0;
        let (record, usage) = decode_with_admission(&input, &limits, |_| { calls += 1; Ok(()) }).unwrap();
        assert!(calls > 1 && usage.points == 0);
        assert_eq!(record.topology().faces, [uuid::Uuid::from_u128(1)]);
        assert!(matches!(decode_with_admission(&input, &limits, |_| Err(Error::Budget("test refusal"))), Err(Error::Budget("test refusal"))));
        let mut corrupt: serde_json::Value = serde_json::from_slice(&input).unwrap();
        corrupt["source"] = serde_json::json!("CASCADE Topology altered");
        assert!(matches!(decode_with_admission(&serde_json::to_vec(&corrupt).unwrap(), &limits, |_| Ok(())), Err(Error::Invalid("BRep source binding"))));
        let mut corrupt: serde_json::Value = serde_json::from_slice(&input).unwrap();
        corrupt["topology"]["faces"][0] = serde_json::json!(uuid::Uuid::from_u128(2));
        assert!(matches!(decode_with_admission(&serde_json::to_vec(&corrupt).unwrap(), &limits, |_| Ok(())), Err(Error::Invalid("BRep source/topology integrity"))));
        let duplicate = br#"{"source":"a","source":"b"}"#;
        assert!(matches!(decode_with_admission(duplicate, &limits, |_| Ok(())), Err(Error::Invalid("duplicate JSON key"))));
    }

    #[test]
    fn source_integrity_checks_admission_between_large_chunks() {
        let mut wire: serde_json::Value = serde_json::from_slice(&fixture()).unwrap();
        let mut source = "a".repeat(1024 * 1024 - 4);
        source.push_str("CASCADE Topology");
        source.push_str(&"b".repeat(1024 * 1024));
        let digest: [u8; 32] = Sha256::digest(source.as_bytes()).into();
        let topology = om9_core::brep::BrepTopology { faces: vec![uuid::Uuid::from_u128(1)],
            edges: vec![], vertex_ids: vec![], face_edges: vec![] };
        wire["source"] = serde_json::json!(source);
        wire["source_sha256"] = serde_json::json!(digest);
        wire["binding_sha256"] = serde_json::json!(om9_core::brep::source_topology_fingerprint(digest, &topology));
        let record: CanonicalBrepRecord = serde_json::from_value(wire).unwrap();
        let mut calls = 0;
        assert!(matches!(record.validate_admitted(&Limits::default(), |_, _| {
            calls += 1;
            if calls == 3 { Err(Error::Canceled) } else { Ok(()) }
        }), Err(Error::Canceled)));
        record.validate_admitted(&Limits::default(), |_, _| Ok(())).unwrap();
        let limits = Limits { max_bytes: 1024, ..Limits::default() };
        let mut checked = false;
        assert!(matches!(record.validate_admitted(&limits, |_, _| { checked = true; Ok(()) }), Err(Error::Budget(_))));
        assert!(!checked);
    }

    #[test]
    fn reserved_record_keeps_charge_and_failures_release_it() {
        use om9_core::{model::Document, transaction::{TransactionCoordinator, World}};
        let input = fixture();
        let limits = Limits::default();
        let owner = TransactionCoordinator::new(World::new(vec![Document::new("owner")], &limits).unwrap(), limits).unwrap();
        let mut peer = TransactionCoordinator::new(World::new(vec![Document::new("peer")], &limits).unwrap(), limits).unwrap();
        peer.adopt_external_captures(&owner).unwrap();
        let baseline = peer.external_reserved_bytes(MemoryClass::CpuCache);
        let physical = Some((8 * 1024 * 1024 * 1024, 6 * 1024 * 1024 * 1024));
        let reserve = |bytes| owner.reserve_external_memory(MemoryClass::CpuCache, bytes, physical).unwrap();
        let record = decode_reserved(&input, &limits, &owner.external_memory_account(), reserve(64 * 1024), || false).unwrap();
        assert_eq!(record.record().source(), "CASCADE Topology fixture");
        assert_eq!(peer.external_reserved_bytes(MemoryClass::CpuCache), baseline + record.reserved_bytes());
        drop(record);
        assert_eq!(peer.external_reserved_bytes(MemoryClass::CpuCache), baseline);
        assert!(matches!(decode_reserved(&input, &limits, &owner.external_memory_account(), reserve(1), || false), Err(Error::Budget(_))));
        assert_eq!(peer.external_reserved_bytes(MemoryClass::CpuCache), baseline);
        let mut grows = 0;
        let growing = decode_reserved_growing(&input, &limits, &owner.external_memory_account(), reserve(1), |receipt, extra| {
            grows += 1;
            owner.grow_external_memory_with_held(receipt, extra, physical, Usage::default())
        }, || false).unwrap();
        assert!(grows > 0 && growing.reserved_bytes() < 64 * 1024);
        drop(growing);
        assert_eq!(peer.external_reserved_bytes(MemoryClass::CpuCache), baseline);
        assert!(matches!(decode_reserved_growing(&input, &limits, &owner.external_memory_account(), reserve(1),
            |_, _| Err(Error::Budget("injected growth refusal")), || false), Err(Error::Budget("injected growth refusal"))));
        assert_eq!(peer.external_reserved_bytes(MemoryClass::CpuCache), baseline);
        assert!(matches!(decode_reserved(&input, &limits, &owner.external_memory_account(), reserve(64 * 1024), || true), Err(Error::Canceled)));
        assert_eq!(peer.external_reserved_bytes(MemoryClass::CpuCache), baseline);
        let foreign = TransactionCoordinator::new(World::new(vec![Document::new("foreign")], &limits).unwrap(), limits).unwrap();
        assert!(matches!(decode_reserved(&input, &limits, &foreign.external_memory_account(), reserve(64 * 1024), || false), Err(Error::Invalid("canonical BRep reservation"))));
        assert_eq!(peer.external_reserved_bytes(MemoryClass::CpuCache), baseline);
    }
}
