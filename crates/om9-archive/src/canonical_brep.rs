// SPDX-License-Identifier: LGPL-2.1-or-later
//! Preflight and decode a standalone source/topology record, not a project codec.
use om9_core::{Error, Result, brep::CanonicalBrepRecord, budget::{Limits, Usage, memory::MemoryClass},
    transaction::{ExternalMemoryAccount, ExternalMemoryReservation}};

/// The decoded record and the account reservation that owns its allocations.
/// Its borrowed record cannot outlive this guard.
pub struct ReservedCanonicalBrep {
    record: CanonicalBrepRecord,
    reservation: ExternalMemoryReservation,
}
impl ReservedCanonicalBrep {
    pub fn record(&self) -> &CanonicalBrepRecord { &self.record }
    pub fn reserved_bytes(&self) -> u64 { self.reservation.bytes() }
}

/// Decode under a caller-supplied, already admitted peak reservation. The
/// input must remain accounted separately by its owner. The conservative
/// preflight also includes the input, so this may temporarily double-charge.
/// Cancellation and all failures drop the reservation without publishing a
/// record. The caller must not share an unaccounted clone of the record.
pub fn decode_reserved(input: &[u8], limits: &Limits, account: &ExternalMemoryAccount,
    mut reservation: ExternalMemoryReservation, mut cancelled: impl FnMut() -> bool) -> Result<ReservedCanonicalBrep> {
    if reservation.class() != MemoryClass::CpuCache || !reservation.belongs_to(account)
        || reservation.pending_bytes() != reservation.bytes() || !reservation.physical_sampled() {
        return Err(Error::Invalid("canonical BRep reservation"));
    }
    let (record, usage) = decode_with_admission(input, limits, |peak| {
        if cancelled() { return Err(Error::Canceled); }
        if peak > reservation.bytes() { return Err(Error::Budget("canonical BRep reservation")); }
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
    fn reserved_record_keeps_charge_and_failures_release_it() {
        use om9_core::{model::Document, transaction::{TransactionCoordinator, World}};
        let input = fixture();
        let limits = Limits::default();
        let owner = TransactionCoordinator::new(World::new(vec![Document::new("owner")], &limits).unwrap(), limits).unwrap();
        let mut peer = TransactionCoordinator::new(World::new(vec![Document::new("peer")], &limits).unwrap(), limits).unwrap();
        peer.adopt_external_captures(&owner).unwrap();
        let physical = Some((8 * 1024 * 1024 * 1024, 6 * 1024 * 1024 * 1024));
        let reserve = |bytes| owner.reserve_external_memory(MemoryClass::CpuCache, bytes, physical).unwrap();
        let record = decode_reserved(&input, &limits, &owner.external_memory_account(), reserve(64 * 1024), || false).unwrap();
        assert_eq!(record.record().source(), "CASCADE Topology fixture");
        assert_eq!(peer.external_reserved_bytes(MemoryClass::CpuCache), record.reserved_bytes());
        drop(record);
        assert_eq!(peer.external_reserved_bytes(MemoryClass::CpuCache), 0);
        assert!(matches!(decode_reserved(&input, &limits, &owner.external_memory_account(), reserve(1), || false), Err(Error::Budget(_))));
        assert_eq!(peer.external_reserved_bytes(MemoryClass::CpuCache), 0);
        assert!(matches!(decode_reserved(&input, &limits, &owner.external_memory_account(), reserve(64 * 1024), || true), Err(Error::Canceled)));
        assert_eq!(peer.external_reserved_bytes(MemoryClass::CpuCache), 0);
        let foreign = TransactionCoordinator::new(World::new(vec![Document::new("foreign")], &limits).unwrap(), limits).unwrap();
        assert!(matches!(decode_reserved(&input, &limits, &foreign.external_memory_account(), reserve(64 * 1024), || false), Err(Error::Invalid("canonical BRep reservation"))));
        assert_eq!(peer.external_reserved_bytes(MemoryClass::CpuCache), 0);
    }
}
