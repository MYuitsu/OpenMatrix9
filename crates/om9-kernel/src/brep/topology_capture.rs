// SPDX-License-Identifier: LGPL-2.1-or-later
//! Opt-in, in-process source snapshot plus bounded topology descriptor capture.
use om9_core::brep::BrepSourceAccess;
use super::*;
use om9_core::{Error, Result, budget::Usage, brep::{BrepTopology, topology_mapping::{Descriptor, Incidence, IndexMapping, Manifest, match_manifests_admitted}}};
use sha2::{Digest, Sha256};
use std::sync::OnceLock;
use om9_core::{budget::memory::MemoryClass, identity::SourceStamp,
    transaction::{CapturedBrepSource, ExternalMemoryReservation, OperationContext, TransactionCoordinator}};

// Deliberately process-scoped. This is not a persistent kernel-build identity:
// saved data must not reuse it after restart or serialize it as an attestation.
fn session_key() -> &'static str {
    static KEY: OnceLock<[u8; 36]> = OnceLock::new();
    let key = KEY.get_or_init(|| {
        let mut bytes = [0; 36];
        Uuid::new_v4().hyphenated().encode_lower(&mut bytes);
        bytes
    });
    std::str::from_utf8(key).expect("UUID is ASCII")
}

/// Owned descriptors captured in the same native callback as the returned
/// display. These are not serializable and are valid for this process only.
pub struct CapturedTopologySnapshot {
    brep: Brep,
    source_sha256: [u8; 32],
    faces: Vec<Descriptor>, edges: Vec<Descriptor>, vertices: Vec<Descriptor>,
    face_edges: Vec<Incidence>, edge_vertices: Vec<Incidence>,
}
/// A B-rep and a shape-set packet captured from the same native snapshot.
/// UUID associations are the snapshot's face/edge/vertex order; callers must
/// keep both allocations charged and verify them on any later restore.
pub struct NamedTopologySnapshot {
    snapshot: CapturedTopologySnapshot,
    shape_set: Vec<u8>,
}
impl NamedTopologySnapshot {
    pub fn snapshot(&self) -> &CapturedTopologySnapshot { &self.snapshot }
    pub fn shape_set(&self) -> &[u8] { &self.shape_set }
    pub fn storage_bytes(&self, limits: &Limits) -> Result<u64> {
        self.snapshot.manifest_bytes()?.checked_add(self.snapshot.brep.storage_usage(limits)?.bytes)
            .and_then(|v| v.checked_add(self.shape_set.capacity() as u64))
            .ok_or(Error::Overflow("named topology storage"))
    }
    /// Bind the packet's native references to the IDs of an existing,
    /// attested B-rep. The packet and candidate came from the same snapshot;
    /// the existing proof is checked before IDs can be carried across.
    /// Unattested legacy displays must continue using their original mesh.
    pub fn into_rebound_for_brep(self, original: &Brep, proof: &TopologyAttestation,
        limits: &Limits, mut admit: impl FnMut(u64, u64) -> Result<()>) -> Result<Self> {
        let Self { snapshot, shape_set } = self;
        let packet_bytes = Usage::array::<u8>(shape_set.capacity())?.bytes;
        let remaining = limits.remaining_after(Usage { bytes: packet_bytes, ..Usage::default() })?;
        let snapshot = snapshot.into_rebound_for_brep(original, proof, &remaining, |live, pending| {
            admit(packet_bytes.checked_add(live).ok_or(Error::Overflow("named topology admission"))?, pending)
        })?;
        Ok(Self { snapshot, shape_set })
    }

    /// Bind a packet captured from the same native snapshot to an existing
    /// tracked, attested source without materializing its old display. The
    /// caller keeps the source lease and proof charged, and must verify the
    /// current session and object before publishing the resulting packet.
    pub fn into_rebound_for_source(self, original: &RegisteredTopologySource,
        limits: &Limits, mut admit: impl FnMut(u64, u64) -> Result<()>) -> Result<Self> {
        let Self { snapshot, shape_set } = self;
        let packet_bytes = Usage::array::<u8>(shape_set.capacity())?.bytes;
        let remaining = limits.remaining_after(Usage { bytes: packet_bytes, ..Usage::default() })?;
        let snapshot = snapshot.into_rebound_for_source(original, &remaining, |live, pending| {
            admit(packet_bytes.checked_add(live).ok_or(Error::Overflow("named topology admission"))?, pending)
        })?;
        Ok(Self { snapshot, shape_set })
    }
    /// Restore only the packet owned by this same capture. Its UUIDs and
    /// packet are inseparable here; arbitrary file input is not supported.
    /// Native OCCT parsing and meshing allocations are outside Rust admission.
    pub fn restore(&self, deflection: f64, angular: f64, limits: &Limits,
        admit: impl FnMut(u64, u64) -> Result<()>) -> std::result::Result<Brep, KernelError> {
        let held = self.storage_bytes(limits).map_err(|error| KernelError::InvalidInput(error.to_string()))?;
        restore_shape_set(&self.shape_set, self.snapshot.brep.source(), self.snapshot.brep.topology(), held, deflection, angular, limits, admit)
    }
    /// Transfer the exact shape packet and IDs without retaining the display.
    /// The caller must retain its existing reservation through the transfer.
    pub fn into_source(self) -> NamedTopologySource {
        NamedTopologySource { shape_set: self.shape_set, source: self.snapshot.brep.into_owned_source() }
    }
}

/// Source and topology captured together. It never owns a display mesh.
/// Arbitrary external packets cannot be used to construct this type.
pub struct NamedTopologySource {
    shape_set: Vec<u8>,
    source: om9_core::brep::OwnedBrepSource,
}
impl NamedTopologySource {
    pub fn source(&self) -> &str { self.source.source() }
    pub fn topology(&self) -> &BrepTopology { self.source.topology() }
    /// Borrow the source record paired with this capture's shape set.
    pub fn canonical_record(&self) -> om9_core::brep::CanonicalBrepRef<'_> { self.source.canonical_record() }
    pub fn shape_set(&self) -> &[u8] { &self.shape_set }
    pub fn storage_bytes(&self) -> Result<u64> {
        Usage::array::<u8>(self.shape_set.capacity())?.bytes.checked_add(self.source.storage_bytes()?)
            .ok_or(Error::Overflow("named topology source"))
    }
    /// Measure the private shape-set root and rebind its explicit native
    /// references to this owner's IDs, without creating a display mesh. The
    /// packet and IDs must remain the inseparable pair created by capture.
    /// Keep the output allocation reserved while the returned vector lives.
    /// OCCT parser, validity checks and property integration are not covered
    /// by Rust admission and are not interruptible within each native call.
    pub fn signatures(&self, limits: &Limits, mut admit: impl FnMut(u64, u64) -> Result<()>,
        canceled: impl Fn() -> bool) -> Result<Vec<BoundSourceSignature>> {
        preflight_shape_set(&self.shape_set, self.topology(), limits)?;
        let held = self.storage_bytes()?;
        limits.remaining_after(Usage { bytes: held, ..Usage::default() })?;
        admit(held, 0)?;
        let admission = RefCell::new(&mut admit as &mut dyn FnMut(u64, u64) -> Result<()>);
        let mut mapping = [Vec::<u32>::new(), Vec::new(), Vec::new()];
        let live = Cell::new(held);
        let values = {
            let mut receive_mapping = |kind: c_int, old: u32, new: u32, orientation: c_int| -> Result<()> {
                if canceled() { return Err(Error::Canceled); }
                if !(0..=2).contains(&kind) || !(0..=3).contains(&orientation) { return Err(Error::Invalid("shape set signature mapping")); }
                let group = &mut mapping[kind as usize];
                if group.len() != old as usize { return Err(Error::Invalid("shape set signature mapping order")); }
                let mut current = live.get();
                append(group, new, &mut current, limits, &admission)?;
                live.set(current);
                Ok(())
            };
            let mut signature_admit = |allocated: u64, pending: u64| -> Result<()> {
                let total = live.get().checked_add(allocated).ok_or(Error::Overflow("shape set signature admission"))?;
                limits.remaining_after(Usage { bytes: total.checked_add(pending).ok_or(Error::Overflow("shape set signature admission"))?, ..Usage::default() })?;
                (admission.borrow_mut())(total, pending)
            };
            super::source_signatures::shape_set_signatures(&self.shape_set, limits, &mut signature_admit,
                &canceled, &mut receive_mapping)?
        };
        let signature_bytes = Usage::array::<super::source_signatures::SourceSignature>(values.capacity())?.bytes;
        let base = live.get().checked_add(signature_bytes).ok_or(Error::Overflow("shape set signature storage"))?;
        let planned = Usage::array::<BoundSourceSignature>(values.len())?.bytes;
        limits.remaining_after(Usage { bytes: base.checked_add(planned).ok_or(Error::Overflow("shape set signature output"))?, ..Usage::default() })?;
        (admission.borrow_mut())(base, planned)?;
        let mut output = Vec::new();
        output.try_reserve_exact(values.len()).map_err(|_| Error::Budget("shape set signature output"))?;
        let actual = Usage::array::<BoundSourceSignature>(output.capacity())?.bytes;
        limits.remaining_after(Usage { bytes: base.checked_add(actual).ok_or(Error::Overflow("shape set signature output"))?, ..Usage::default() })?;
        (admission.borrow_mut())(base + actual, 0)?;
        use om9_core::associative::SubshapeKind;
        let topology = self.topology();
        let groups = [(&topology.faces, &mapping[0], 0usize, SubshapeKind::Face),
            (&topology.edges, &mapping[1], mapping[0].len(), SubshapeKind::Edge),
            (&topology.vertex_ids, &mapping[2], mapping[0].len() + mapping[1].len(), SubshapeKind::Vertex)];
        for (ids, indices, offset, kind) in groups {
            if ids.len() != indices.len() { return Err(Error::Invalid("shape set signature identity count")); }
            for (id, index) in ids.iter().zip(indices) {
                let value = values.get(offset.checked_add(*index as usize).ok_or(Error::Overflow("shape set signature index"))?)
                    .ok_or(Error::Invalid("shape set signature index"))?;
                if value.kind != kind || value.source_index != *index as usize { return Err(Error::Invalid("shape set signature order")); }
                output.push(BoundSourceSignature { id: *id, kind, signature: value.signature });
            }
        }
        if canceled() { return Err(Error::Canceled); }
        drop(mapping); drop(values);
        (admission.borrow_mut())(held.checked_add(actual).ok_or(Error::Overflow("shape set signature output"))?, 0)?;
        Ok(output)
    }
    pub fn restore(&self, deflection: f64, angular: f64, limits: &Limits,
        admit: impl FnMut(u64, u64) -> Result<()>) -> std::result::Result<Brep, KernelError> {
        let held = self.storage_bytes().map_err(|error| KernelError::InvalidInput(error.to_string()))?;
        restore_shape_set(&self.shape_set, self.source(), self.topology(), held, deflection, angular, limits, admit)
    }
}

/// Check the bounded envelope before invoking the native ShapeSet parser.
/// This cannot bound OCCT's internal parsing or certify untrusted input.
fn preflight_shape_set(packet: &[u8], topology: &BrepTopology, limits: &Limits) -> Result<()> {
    if packet.len() as u64 > limits.max_bytes { return Err(Error::Budget("shape set packet")); }
    let rest = packet.strip_prefix(b"OM9-SHAPESET 1\n").ok_or(Error::Invalid("shape set version"))?;
    let newline = rest.iter().take(128).position(|byte| *byte == b'\n').ok_or(Error::Invalid("shape set counts"))?;
    let line = std::str::from_utf8(&rest[..newline]).map_err(|_| Error::Invalid("shape set counts"))?;
    let mut parts = line.split_ascii_whitespace();
    let expected = [topology.faces.len(), topology.edges.len(), topology.vertex_ids.len()];
    let mut total = 0u64;
    for expected_count in expected {
        let count = parts.next().ok_or(Error::Invalid("shape set counts"))?.parse::<u64>()
            .map_err(|_| Error::Invalid("shape set counts"))?;
        if count > i32::MAX as u64 { return Err(Error::Budget("shape set topology")); }
        total = total.checked_add(count).ok_or(Error::Overflow("shape set topology"))?;
        if count != expected_count as u64 { return Err(Error::Invalid("shape set topology count")); }
    }
    if parts.next().is_some() || expected[0] == 0 { return Err(Error::Invalid("shape set counts")); }
    if total > limits.max_nodes { return Err(Error::Budget("shape set topology")); }
    if rest.len() == newline + 1 { return Err(Error::Invalid("shape set body")); }
    Ok(())
}

/// Test-only entry point for an internally generated packet transferred to a
/// fresh process. This is not admission for arbitrary user-supplied files.
#[cfg(feature = "test-fixtures")]
pub fn test_restore_captured_shape_set(packet: &[u8], source: &str, topology: &BrepTopology, held: u64,
    deflection: f64, angular: f64, limits: &Limits,
    admit: impl FnMut(u64, u64) -> Result<()>) -> std::result::Result<Brep, KernelError> {
    restore_shape_set(packet, source, topology, held, deflection, angular, limits, admit)
}

/// Native parsing entry for the separately built restore worker. The caller
/// must run this in an owned process with enforced memory, output and lifetime
/// limits; this API does not certify arbitrary input or authorize publication.
#[cfg(feature = "isolated-brep-worker")]
pub fn restore_isolated_worker_input(packet: &[u8], source: &str, topology: &BrepTopology, held: u64,
    deflection: f64, angular: f64, limits: &Limits,
    admit: impl FnMut(u64, u64) -> Result<()>) -> std::result::Result<Brep, KernelError> {
    restore_shape_set(packet, source, topology, held, deflection, angular, limits, admit)
}

fn restore_shape_set(packet: &[u8], source: &str, topology: &BrepTopology, held: u64,
    deflection: f64, angular: f64, limits: &Limits,
    mut admit: impl FnMut(u64, u64) -> Result<()>) -> std::result::Result<Brep, KernelError> {
        if !deflection.is_finite() || deflection <= 0. || !angular.is_finite()
            || angular <= 0. || angular > std::f64::consts::PI {
            return Err(KernelError::InvalidInput("shape set restore profile".into()));
        }
        preflight_shape_set(packet, topology, limits).map_err(|error| KernelError::InvalidInput(error.to_string()))?;
        let operation = (|| -> Result<Brep> {
            limits.remaining_after(Usage { bytes: held, ..Usage::default() })?;
            admit(held, 0)?;
            let admission = RefCell::new(&mut admit as &mut dyn FnMut(u64, u64) -> Result<()>);
            let mut state = ShapeSetRestore { limits, admission: &admission, live: held,
                mapping: [Vec::new(), Vec::new(), Vec::new()], deflection, result: None, proof: None, error: None };
            let mut message = [0i8; 512];
            let context = (&mut state as *mut ShapeSetRestore<'_, '_>).cast();
            let status = unsafe { om9_occt_read_snapshot_shape_set(packet.as_ptr(), packet.len(),
                deflection, angular, bound(limits.max_points), bound(limits.max_nodes), bound(limits.max_bytes),
                receive_shape_mapping, context, receive_shape_snapshot, context, message.as_mut_ptr(), message.len() as c_int) };
            if let Some(error) = state.error.take() { return Err(error); }
            if status != 0 {
                return Err(Error::Io(unsafe { CStr::from_ptr(message.as_ptr()) }.to_string_lossy().into_owned()));
            }
            let candidate = state.result.take().ok_or(Error::Invalid("shape set snapshot missing"))??;
            let mut proof = state.proof.take().ok_or(Error::Invalid("shape set proof missing"))?;
            let [faces, edges, vertices] = state.mapping;
            let mapping = IndexMapping::from_reference_indices(faces, edges, vertices);
            let mapping_bytes = mapping.storage_bytes()?;
            let candidate_bytes = candidate.storage_usage(limits)?.bytes;
            let proof_bytes = proof.storage_bytes()?;
            let base = held.checked_add(mapping_bytes).and_then(|v| v.checked_add(candidate_bytes))
                .and_then(|v| v.checked_add(proof_bytes))
                .ok_or(Error::Overflow("shape set restore"))?;
            let remaining = limits.remaining_after(Usage { bytes: base, ..Usage::default() })?;
            let topology = mapping.reorder_topology_admitted(topology,
                &candidate.topology().face_edges, &remaining, &mut |live, pending| {
                    (admission.borrow_mut())(base.checked_add(live).ok_or(Error::Overflow("shape set restore"))?, pending)
                })?;
            drop(mapping);
            let proof_held = held.checked_add(proof_bytes).ok_or(Error::Overflow("shape set restore"))?;
            let remaining = limits.remaining_after(Usage { bytes: proof_held, ..Usage::default() })?;
            let candidate = candidate.rebind_candidate_topology(topology, &remaining, |live, pending| {
                (admission.borrow_mut())(proof_held.checked_add(live).ok_or(Error::Overflow("shape set restore"))?, pending)
            })?;
            proof.display_sha256 = candidate.display_fingerprint();
            proof.topology_sha256 = om9_core::brep::source_topology_fingerprint(proof.source_sha256, candidate.topology());
            // Downstream native commands read the retained canonical source.
            // Match that independent read to the packet's verified references
            // rather than assuming either enumeration or serialized bytes match.
            let reread_base = proof_held.checked_add(candidate.storage_usage(limits)?.bytes)
                .ok_or(Error::Overflow("shape set restore"))?;
            let reread_limits = limits.remaining_after(Usage { bytes: reread_base, ..Usage::default() })?;
            let reread = capture_source_topology(source, deflection, angular, &reread_limits, |live, pending| {
                (admission.borrow_mut())(reread_base.checked_add(live).ok_or(Error::Overflow("shape set restore"))?, pending)
            }).map_err(|error| Error::Io(error.to_string()))?;
            if reread.brep().source() != source { return Err(Error::Invalid("shape set canonical source changed")); }
            let final_limits = limits.remaining_after(Usage { bytes: held, ..Usage::default() })?;
            let rebound = reread.into_rebound_for_packet(&candidate, &proof, &final_limits, |live, pending| {
                (admission.borrow_mut())(held.checked_add(live).ok_or(Error::Overflow("shape set restore"))?, pending)
            })?;
            let (result, rebound_proof) = rebound.into_parts();
            drop(rebound_proof);
            drop(proof);
            drop(candidate);
            (admission.borrow_mut())(held.checked_add(result.storage_usage(limits)?.bytes)
                .ok_or(Error::Overflow("shape set restore"))?, 0)?;
            Ok(result)
        })();
        operation.map_err(|error| KernelError::InvalidInput(error.to_string()))
}
/// In-process proof that descriptors came from the same native snapshot as a
/// specific validated B-rep display and topology. It does not hold the B-rep
/// or its mesh. The owner must reserve its vector capacities for its lifetime.
pub struct TopologyAttestation {
    source_sha256: [u8; 32], topology_sha256: [u8; 32], display_sha256: [u8; 32],
    faces: Vec<Descriptor>, edges: Vec<Descriptor>, vertices: Vec<Descriptor>,
    face_edges: Vec<Incidence>, edge_vertices: Vec<Incidence>,
}

/// A proof adopted by the coordinator's memory account and bound to its
/// current session and root-object stamp. No document or display is retained.
/// The reservation must have covered the proof while it was being built.
pub struct RegisteredTopologyProof {
    proof: TopologyAttestation,
    // Immutable, accounted source/topology captured at attestation time. It
    // does not retain a document or display and permits exact comparison.
    source: CapturedBrepSource,
    stamp: SourceStamp,
    context: OperationContext,
    document: std::sync::Weak<om9_core::model::Document>,
    reservation: ExternalMemoryReservation,
}
/// Shared proof allocation and its independently charged Arc owner.
pub struct SharedTopologyProof {
    proof: RegisteredTopologyProof,
    binding: std::sync::Mutex<ProofBinding>,
    _owner_reservation: ExternalMemoryReservation,
    worker_active: std::sync::atomic::AtomicBool,
}
#[derive(Clone)]
struct ProofBinding {
    stamp: SourceStamp,
    context: OperationContext,
    document: std::sync::Weak<om9_core::model::Document>,
    display_verified: bool,
}
pub struct TopologyWorkerPermit(std::sync::Arc<SharedTopologyProof>);
impl Drop for TopologyWorkerPermit {
    fn drop(&mut self) { self.0.worker_active.store(false, std::sync::atomic::Ordering::Release); }
}
impl SharedTopologyProof {
    pub fn attestation(&self) -> &TopologyAttestation { self.proof.attestation() }
    pub fn source_identity(&self) -> om9_core::brep::BrepSourceIdentity { self.proof.source.source_identity() }
    /// Includes the source lease retained by this proof itself.
    pub fn source_strong_count(&self) -> usize { self.proof.source.source_strong_count() }
    pub fn same_source_allocations(&self, other: &Self) -> bool {
        self.proof.source.same_canonical_allocations(&other.proof.source)
    }
    /// Reservations released when the last strong owner of this proof dies.
    pub fn reserved_bytes(&self) -> Result<u64> {
        self.proof.reservation.bytes().checked_add(self._owner_reservation.bytes())
            .ok_or(Error::Overflow("shared topology proof reservation"))
    }
    pub fn allocation_bytes() -> u64 {
        (std::mem::size_of::<Self>() + 2 * std::mem::size_of::<usize>()) as u64
    }
    pub fn new(proof: RegisteredTopologyProof, coordinator: &TransactionCoordinator,
        mut reservation: ExternalMemoryReservation) -> Result<std::sync::Arc<Self>> {
        if reservation.class() != MemoryClass::CpuCache
            || !reservation.belongs_to(&coordinator.external_memory_account())
            || !proof.reservation.belongs_to(&coordinator.external_memory_account())
            || reservation.bytes() != Self::allocation_bytes()
            || reservation.pending_bytes() != reservation.bytes() {
            return Err(Error::Invalid("shared topology proof reservation"));
        }
        reservation.recheck_pending()?;
        reservation.set_allocated_bytes(reservation.bytes())?;
        let binding = ProofBinding { stamp: proof.stamp, context: proof.context, document: proof.document.clone(), display_verified: true };
        Ok(std::sync::Arc::new(Self { proof, binding: std::sync::Mutex::new(binding), _owner_reservation: reservation,
            worker_active: std::sync::atomic::AtomicBool::new(false) }))
    }
    pub fn acquire_worker(self: &std::sync::Arc<Self>) -> Result<TopologyWorkerPermit> {
        self.worker_active.compare_exchange(false, true, std::sync::atomic::Ordering::AcqRel,
            std::sync::atomic::Ordering::Acquire).map_err(|_| Error::Busy)?;
        Ok(TopologyWorkerPermit(std::sync::Arc::clone(self)))
    }
    pub fn stamp(&self) -> SourceStamp {
        self.binding.lock().unwrap_or_else(|e| e.into_inner()).stamp
    }
    /// Cheap cache-retention filter only. A match is not an attestation:
    /// rebind must still validate the complete current source and display.
    pub fn may_match_display(&self, brep: &Brep) -> bool {
        brep.display_fingerprint() == self.proof.proof.display_sha256
    }
    /// A candidate filter, not a new native attestation. The proof was
    /// adopted from a native-attested source and retains that exact source.
    pub fn may_match_source(&self, brep: &dyn BrepSourceAccess) -> bool {
        self.proof.source.shares_brep_canonical_allocations(brep)
            || (brep.source() == self.proof.source.source() && brep.topology() == self.proof.source.topology())
    }
    /// Identity filter for reusing this already-attested immutable source
    /// through another occurrence. Content equality alone is not sufficient.
    pub fn shares_canonical_allocations(&self, brep: &dyn BrepSourceAccess) -> bool {
        self.proof.source.shares_brep_canonical_allocations(brep)
    }
    /// Cheap lookup filter; worker validation must compare the full source.
    pub fn may_candidate_source(&self, brep: &dyn BrepSourceAccess) -> bool {
        brep.canonical_fingerprints() == (self.proof.proof.source_sha256, self.proof.proof.topology_sha256)
    }
    /// Cache deduplication only; use still requires current-document validation.
    pub fn same_snapshot(&self, other: &Self) -> bool {
        let a = self.stamp();
        let b = other.stamp();
        a.document == b.document && a.owner_session == b.owner_session
            && a.session_generation == b.session_generation && a.object == b.object
            && a.generation == b.generation
            && self.proof.proof.source_sha256 == other.proof.proof.source_sha256
            && self.proof.proof.topology_sha256 == other.proof.proof.topology_sha256
            && self.proof.proof.display_sha256 == other.proof.proof.display_sha256
            && (self.proof.source.same_canonical_allocations(&other.proof.source)
                || (self.proof.source.source() == other.proof.source.source()
                    && self.proof.source.topology() == other.proof.source.topology()))
    }
    /// Fast check for the currently bound immutable document allocation.
    pub fn verify_unchanged_owner<'a>(&self, coordinator: &'a TransactionCoordinator) -> Result<&'a om9_core::brep::CanonicalBrep> {
        if !self.proof.reservation.belongs_to(&coordinator.external_memory_account()) {
            return Err(Error::Stale("topology proof account"));
        }
        let binding = self.binding.lock().unwrap_or_else(|e| e.into_inner());
        if !binding.display_verified { return Err(Error::Stale("topology proof display")); }
        let (context, world) = coordinator.capture_context(None)?;
        if context != binding.context { return Err(Error::Stale("topology proof context")); }
        let document = world.documents.get(&binding.stamp.document).ok_or(Error::Stale("topology proof document"))?;
        if !binding.document.ptr_eq(&std::sync::Arc::downgrade(document)) {
            return Err(Error::Stale("topology proof document owner"));
        }
        let entity = document.resolve(&binding.stamp)?;
        let om9_core::geometry::Geometry::Brep(brep) = &entity.geometry else { return Err(Error::Stale("topology proof geometry")); };
        if coordinator.brep_representation(brep)?.display_fingerprint() != self.proof.proof.display_sha256 { return Err(Error::Stale("topology proof display")); }
        Ok(brep)
    }
    /// Rebind after a document change: check the same account, runtime
    /// session and object generation, then fully revalidate source and display
    /// even when the geometry revision changed (for example, a placement edit).
    /// Active workers block
    /// rebinding so their source receipt cannot silently change underneath them.
    pub fn rebind_unchanged(self: &std::sync::Arc<Self>, coordinator: &TransactionCoordinator,
        mut admit_scratch: impl FnMut(u64, u64) -> Result<()>) -> Result<()> {
        let _permit = self.acquire_worker()?;
        if !self.proof.reservation.belongs_to(&coordinator.external_memory_account()) {
            return Err(Error::Stale("topology proof account"));
        }
        let mut binding = self.binding.lock().unwrap_or_else(|e| e.into_inner());
        let (context, world) = coordinator.capture_context(None)?;
        if context.authority() != binding.context.authority() { return Err(Error::Stale("topology proof authority")); }
        let document = world.documents.get(&binding.stamp.document).ok_or(Error::Stale("topology proof document"))?;
        let stamp = document.stamp(binding.stamp.object)?;
        if stamp.owner_session != binding.stamp.owner_session || stamp.session_generation != binding.stamp.session_generation
            || stamp.generation != binding.stamp.generation {
            return Err(Error::Stale("topology proof source revision"));
        }
        let entity = document.resolve(&stamp)?;
        let om9_core::geometry::Geometry::Brep(brep) = &entity.geometry else { return Err(Error::Stale("topology proof geometry")); };
        if !self.proof.source.shares_brep_canonical_allocations(brep)
            && (brep.source() != self.proof.source.source() || brep.topology() != self.proof.source.topology()) {
            return Err(Error::Stale("topology proof source"));
        }
        let limits = coordinator.limits();
        let baseline = self.proof.proof.storage_bytes()?.checked_add(brep.storage_usage(limits)?.bytes)
            .ok_or(Error::Overflow("topology proof admission"))?;
        self.proof.proof.verify_brep(&coordinator.materialize_brep(brep)?, limits, |live, pending| {
            let extra = live.checked_sub(baseline).ok_or(Error::Invalid("topology proof admission"))?;
            admit_scratch(extra, pending)
        })?;
        *binding = ProofBinding { stamp, context, document: std::sync::Arc::downgrade(document), display_verified: true };
        Ok(())
    }
    /// Rebind the previously native-attested canonical source without
    /// asserting anything about the new document's display. A source-only
    /// binding cannot satisfy `verify_unchanged_owner` until full validation.
    pub fn rebind_source_unchanged(self: &std::sync::Arc<Self>, coordinator: &TransactionCoordinator,
        mut admit_scratch: impl FnMut(u64, u64) -> Result<()>) -> Result<()> {
        let _permit = self.acquire_worker()?;
        if !self.proof.reservation.belongs_to(&coordinator.external_memory_account()) {
            return Err(Error::Stale("topology proof account"));
        }
        let mut binding = self.binding.lock().unwrap_or_else(|e| e.into_inner());
        let (context, world) = coordinator.capture_context(None)?;
        if context.authority() != binding.context.authority() { return Err(Error::Stale("topology proof authority")); }
        let document = world.documents.get(&binding.stamp.document).ok_or(Error::Stale("topology proof document"))?;
        let stamp = document.stamp(binding.stamp.object)?;
        if stamp.owner_session != binding.stamp.owner_session || stamp.session_generation != binding.stamp.session_generation
            || stamp.generation != binding.stamp.generation { return Err(Error::Stale("topology proof source revision")); }
        let entity = document.resolve(&stamp)?;
        let om9_core::geometry::Geometry::Brep(brep) = &entity.geometry else { return Err(Error::Stale("topology proof geometry")); };
        if !self.may_match_source(brep) { return Err(Error::Stale("topology proof source")); }
        let baseline = brep.source_storage_bytes()?;
        brep.validate_source_admitted(coordinator.limits(), |live, pending| {
            let extra = live.checked_sub(baseline).ok_or(Error::Invalid("topology source admission"))?;
            admit_scratch(extra, pending)
        })?;
        self.proof.proof.verify_source_binding(self.proof.source.source_fingerprint(), brep.topology())?;
        *binding = ProofBinding { stamp, context, document: std::sync::Arc::downgrade(document), display_verified: false };
        Ok(())
    }
    /// Reuse the already-attested immutable allocations after a document-only
    /// change. No source scan or scratch allocation is needed: both source
    /// and topology are the exact live allocations validated at adoption.
    /// Returns false without changing the binding for independent owners.
    pub fn rebind_shared_source(self: &std::sync::Arc<Self>, coordinator: &TransactionCoordinator) -> Result<bool> {
        let _permit = self.acquire_worker()?;
        if !self.proof.reservation.belongs_to(&coordinator.external_memory_account()) {
            return Err(Error::Stale("topology proof account"));
        }
        let mut binding = self.binding.lock().unwrap_or_else(|e| e.into_inner());
        let (context, world) = coordinator.capture_context(None)?;
        if context.authority() != binding.context.authority() { return Err(Error::Stale("topology proof authority")); }
        let document = world.documents.get(&binding.stamp.document).ok_or(Error::Stale("topology proof document"))?;
        let stamp = document.stamp(binding.stamp.object)?;
        if stamp.owner_session != binding.stamp.owner_session || stamp.session_generation != binding.stamp.session_generation
            || stamp.generation != binding.stamp.generation { return Err(Error::Stale("topology proof source revision")); }
        let entity = document.resolve(&stamp)?;
        let om9_core::geometry::Geometry::Brep(brep) = &entity.geometry else { return Err(Error::Stale("topology proof geometry")); };
        if !self.proof.source.shares_brep_canonical_allocations(brep) { return Ok(false); }
        *binding = ProofBinding { stamp, context, document: std::sync::Arc::downgrade(document), display_verified: false };
        Ok(true)
    }
    /// Capture immutable owners and lifecycle tokens only; expensive content
    /// comparison and validation are deferred to the worker.
    pub fn prepare_source_rebind(self: &std::sync::Arc<Self>, coordinator: &TransactionCoordinator) -> Result<PendingSourceRebind> {
        if !self.proof.reservation.belongs_to(&coordinator.external_memory_account()) { return Err(Error::Stale("topology proof account")); }
        let original = self.binding.lock().unwrap_or_else(|e| e.into_inner()).clone();
        let (context, world) = coordinator.capture_context(None)?;
        if context.authority() != original.context.authority() { return Err(Error::Stale("topology proof authority")); }
        let document = world.documents.get(&original.stamp.document).ok_or(Error::Stale("topology proof document"))?;
        let stamp = document.stamp(original.stamp.object)?;
        if stamp.owner_session != original.stamp.owner_session || stamp.session_generation != original.stamp.session_generation
            || stamp.generation != original.stamp.generation { return Err(Error::Stale("topology proof source revision")); }
        let source = coordinator.capture_brep_source(&stamp)?;
        Ok(PendingSourceRebind { proof: self.clone(), source, original, context, stamp, document: std::sync::Arc::downgrade(document) })
    }
    pub fn verify_unchanged_source<'a>(&self, coordinator: &'a TransactionCoordinator) -> Result<&'a om9_core::brep::CanonicalBrep> {
        if !self.proof.reservation.belongs_to(&coordinator.external_memory_account()) { return Err(Error::Stale("topology proof account")); }
        let binding = self.binding.lock().unwrap_or_else(|e| e.into_inner());
        let (context, world) = coordinator.capture_context(None)?;
        if context != binding.context { return Err(Error::Stale("topology proof context")); }
        let document = world.documents.get(&binding.stamp.document).ok_or(Error::Stale("topology proof document"))?;
        if !binding.document.ptr_eq(&std::sync::Arc::downgrade(document)) { return Err(Error::Stale("topology proof document owner")); }
        let entity = document.resolve(&binding.stamp)?;
        let om9_core::geometry::Geometry::Brep(brep) = &entity.geometry else { return Err(Error::Stale("topology proof geometry")); };
        Ok(brep)
    }
    pub fn capture_source(self: &std::sync::Arc<Self>, coordinator: &TransactionCoordinator) -> Result<RegisteredTopologySource> {
        self.verify_unchanged_source(coordinator)?;
        let source = self.proof.source.clone();
        self.bind_captured_source(coordinator, source)
    }
    /// Bind retained evidence to an unpublished history source. The caller
    /// owns the history-head and publication checks; this only verifies the
    /// immutable source, native attestation and original session identity.
    pub fn bind_history_candidate(self: &std::sync::Arc<Self>, coordinator: &TransactionCoordinator,
        stamp: &SourceStamp, canonical: &om9_core::brep::CanonicalBrep,
        limits: &Limits, mut admit: impl FnMut(u64, u64) -> Result<()>) -> Result<RegisteredTopologySource> {
        if !self.proof.reservation.belongs_to(&coordinator.external_memory_account())
            || !self.proof.source.belongs_to(coordinator) {
            return Err(Error::Stale("history topology account"));
        }
        let binding = self.binding.lock().unwrap_or_else(|e| e.into_inner()).clone();
        let (context, _) = coordinator.capture_context(None)?;
        if context.authority() != binding.context.authority() || stamp.document != binding.stamp.document
            || stamp.object != binding.stamp.object || stamp.generation != binding.stamp.generation
            || stamp.owner_session != binding.stamp.owner_session || stamp.session_generation != binding.stamp.session_generation {
            return Err(Error::Stale("history topology identity"));
        }
        let source = &self.proof.source;
        if !source.shares_brep_canonical_allocations(canonical)
            && (source.source() != canonical.source() || source.topology() != canonical.topology()) {
            return Err(Error::Stale("history topology source"));
        }
        canonical.validate_source_admitted(limits, &mut admit)?;
        self.proof.proof.verify_source_binding(source.source_fingerprint(), canonical.topology())?;
        Ok(RegisteredTopologySource { proof: std::sync::Arc::clone(self), source: source.clone() })
    }
    /// Reuse a tracked source lease only after checking this account, current
    /// document binding and the proof's source/topology integrity.
    pub fn bind_captured_source(self: &std::sync::Arc<Self>, coordinator: &TransactionCoordinator,
        source: CapturedBrepSource) -> Result<RegisteredTopologySource> {
        self.verify_unchanged_source(coordinator)?;
        if !source.belongs_to(coordinator) { return Err(Error::Stale("topology source account")); }
        if !source.same_canonical_allocations(&self.proof.source)
            && (source.source() != self.proof.source.source() || source.topology() != self.proof.source.topology()) {
            return Err(Error::Stale("topology source content"));
        }
        self.proof.proof.verify_source_binding(source.source_fingerprint(), source.topology())?;
        Ok(RegisteredTopologySource { proof: std::sync::Arc::clone(self), source })
    }
}
/// Private candidate; the captured source is accounted independently of a document.
pub struct PendingSourceRebind {
    proof: std::sync::Arc<SharedTopologyProof>, source: CapturedBrepSource, original: ProofBinding,
    context: OperationContext, stamp: SourceStamp, document: std::sync::Weak<om9_core::model::Document>,
}
pub struct ValidatedSourceRebind {
    source: RegisteredTopologySource, original: ProofBinding, context: OperationContext,
    stamp: SourceStamp, document: std::sync::Weak<om9_core::model::Document>,
}
impl PendingSourceRebind {
    pub fn validate(self, limits: &Limits, mut admit: impl FnMut(u64, u64) -> Result<()>) -> Result<ValidatedSourceRebind> {
        if !self.source.same_canonical_allocations(&self.proof.proof.source)
            && (self.source.source() != self.proof.proof.source.source() || self.source.topology() != self.proof.proof.source.topology()) {
            return Err(Error::Stale("topology proof source"));
        }
        let baseline = self.source.canonical_storage_bytes()?;
        self.source.validate_canonical_admitted(limits, |live, pending| {
            admit(live.checked_sub(baseline).ok_or(Error::Invalid("topology source admission"))?, pending)
        })?;
        self.proof.proof.proof.verify_source_binding(self.source.source_fingerprint(), self.source.topology())?;
        Ok(ValidatedSourceRebind { source: RegisteredTopologySource { proof: self.proof, source: self.source },
            original: self.original, context: self.context, stamp: self.stamp, document: self.document })
    }
}
impl ValidatedSourceRebind {
    pub fn source(&self) -> &RegisteredTopologySource { &self.source }
    pub fn proof_handle(&self) -> std::sync::Arc<SharedTopologyProof> { self.source.proof.clone() }
    pub fn validate_current(&self, coordinator: &TransactionCoordinator) -> Result<()> {
        if !self.source.source.belongs_to(coordinator) { return Err(Error::Stale("topology source account")); }
        let (context, world) = coordinator.capture_context(None)?;
        if context != self.context { return Err(Error::Stale("topology proof context")); }
        let document = world.documents.get(&self.stamp.document).ok_or(Error::Stale("topology proof document"))?;
        if !self.document.ptr_eq(&std::sync::Arc::downgrade(document)) || document.resolve(&self.stamp).is_err() {
            return Err(Error::Stale("topology proof document owner"));
        }
        let binding = self.source.proof.binding.lock().unwrap_or_else(|e| e.into_inner());
        if binding.context != self.original.context || binding.stamp != self.original.stamp
            || !binding.document.ptr_eq(&self.original.document) || binding.display_verified != self.original.display_verified {
            return Err(Error::Stale("topology proof binding"));
        }
        Ok(())
    }
    pub fn publish(self, coordinator: &TransactionCoordinator) -> Result<RegisteredTopologySource> {
        self.validate_current(coordinator)?;
        let mut binding = self.source.proof.binding.lock().unwrap_or_else(|e| e.into_inner());
        if binding.context != self.original.context || binding.stamp != self.original.stamp
            || !binding.document.ptr_eq(&self.original.document) || binding.display_verified != self.original.display_verified {
            return Err(Error::Stale("topology proof binding"));
        }
        *binding = ProofBinding { stamp: self.stamp, context: self.context, document: self.document, display_verified: false };
        drop(binding);
        Ok(self.source)
    }
}
/// A source-only lease captured while the proof still matched the coordinator.
/// Publication must still revalidate the proof against the current document.
pub struct RegisteredTopologySource {
    proof: std::sync::Arc<SharedTopologyProof>,
    source: CapturedBrepSource,
}
impl RegisteredTopologySource {
    pub fn source(&self) -> &str { self.source.source() }
    pub fn topology(&self) -> &BrepTopology { self.source.topology() }
    pub fn proof(&self) -> &SharedTopologyProof { &self.proof }
    pub fn into_proof(self) -> std::sync::Arc<SharedTopologyProof> { self.proof }
    pub fn into_source(self) -> CapturedBrepSource { self.source }
    #[doc(hidden)]
    pub fn shares_source_lease(&self, source: &CapturedBrepSource) -> bool { self.source.same_lease(source) }

    /// Prepare an owned source and its explicit native topology references
    /// from this already-accounted source/proof pair. The returned allocations
    /// belong to the caller's admission; retain that reservation for their
    /// entire lifetime. Recheck the document and object before publication.
    /// Cancellation is observed between native calls, not within OCCT.
    pub fn capture_named_source(&self, deflection: f64, angular: f64, limits: &Limits,
        mut admit: impl FnMut(u64, u64) -> Result<()>, canceled: impl Fn() -> bool) -> Result<NamedTopologySource> {
        let _permit = self.proof.acquire_worker()?;
        if canceled() { return Err(Error::Canceled); }
        let refusal = RefCell::new(None);
        let snapshot = capture_source_with_shape_set(self.source(), deflection, angular, limits, |live, pending| {
            if canceled() { *refusal.borrow_mut() = Some(Error::Canceled); return Err(Error::Canceled); }
            match admit(live, pending) {
                Ok(()) => Ok(()),
                Err(error) => { *refusal.borrow_mut() = Some(error.clone()); Err(error) }
            }
        }).map_err(|error| refusal.into_inner().unwrap_or_else(|| Error::Io(error.to_string())))?;
        if canceled() { return Err(Error::Canceled); }
        let rebound = snapshot.into_rebound_for_source(self, limits, |live, pending| {
            if canceled() { return Err(Error::Canceled); }
            admit(live, pending)
        })?;
        if canceled() { return Err(Error::Canceled); }
        Ok(rebound.into_source())
    }
}
impl RegisteredTopologyProof {
    /// Adopt an already materialized, exclusively reserved proof. The caller
    /// reserves any owner container separately. `admit_scratch` covers only
    /// transient validation allocations; proof and document are already held.
    pub fn adopt(coordinator: &TransactionCoordinator, stamp: SourceStamp, proof: TopologyAttestation,
        reservation: ExternalMemoryReservation,
        mut admit_scratch: impl FnMut(u64, u64) -> Result<()>) -> Result<Self> {
        let bytes = proof.storage_bytes()?;
        if bytes == 0 || reservation.class() != MemoryClass::CpuCache
            || !reservation.belongs_to(&coordinator.external_memory_account())
            || reservation.bytes() != bytes || reservation.pending_bytes() != 0 {
            return Err(Error::Invalid("topology proof reservation"));
        }
        let (context, world) = coordinator.capture_context(None)?;
        let document = world.documents.get(&stamp.document).ok_or(Error::Stale("topology proof document"))?;
        let source = coordinator.capture_brep_source(&stamp)?;
        let owned = Self { proof, source, stamp, context, document: std::sync::Arc::downgrade(document), reservation };
        owned.verify_current(coordinator, &mut admit_scratch)?;
        Ok(owned)
    }
    pub fn stamp(&self) -> SourceStamp { self.stamp }
    pub fn attestation(&self) -> &TopologyAttestation { &self.proof }
    pub fn storage_bytes(&self) -> u64 { self.reservation.bytes() }
    /// The full source/display/topology was validated during adoption. An
    /// unchanged immutable document allocation plus its session and stamp is
    /// sufficient to reuse that proof without rescanning the mesh. A Weak also
    /// forces Arc::make_mut to detach before the document can be mutated.
    pub fn verify_unchanged_owner<'a>(&self, coordinator: &'a TransactionCoordinator) -> Result<&'a om9_core::brep::CanonicalBrep> {
        if !self.reservation.belongs_to(&coordinator.external_memory_account()) {
            return Err(Error::Stale("topology proof account"));
        }
        let (context, world) = coordinator.capture_context(None)?;
        if context != self.context { return Err(Error::Stale("topology proof context")); }
        let document = world.documents.get(&self.stamp.document).ok_or(Error::Stale("topology proof document"))?;
        if !self.document.ptr_eq(&std::sync::Arc::downgrade(document)) {
            return Err(Error::Stale("topology proof document owner"));
        }
        let entity = document.resolve(&self.stamp)?;
        let om9_core::geometry::Geometry::Brep(brep) = &entity.geometry else { return Err(Error::Stale("topology proof geometry")); };
        if coordinator.brep_representation(brep)?.display_fingerprint() != self.proof.display_sha256 { return Err(Error::Stale("topology proof display")); }
        Ok(brep)
    }
    /// Recheck against the coordinator's current view before scheduling or
    /// publication. Even an identical reopened document has a different
    /// session/context. `admit_scratch` accounts transient validation only.
    pub fn verify_current<'a>(&self, coordinator: &'a TransactionCoordinator,
        mut admit_scratch: impl FnMut(u64, u64) -> Result<()>) -> Result<&'a om9_core::brep::CanonicalBrep> {
        if !self.reservation.belongs_to(&coordinator.external_memory_account()) {
            return Err(Error::Stale("topology proof account"));
        }
        let (context, world) = coordinator.capture_context(None)?;
        if context != self.context { return Err(Error::Stale("topology proof context")); }
        let document = world.documents.get(&self.stamp.document).ok_or(Error::Stale("topology proof document"))?;
        let entity = document.resolve(&self.stamp)?;
        let om9_core::geometry::Geometry::Brep(brep) = &entity.geometry else { return Err(Error::Stale("topology proof geometry")); };
        let limits = coordinator.limits();
        let baseline = self.proof.storage_bytes()?.checked_add(brep.storage_usage(limits)?.bytes)
            .ok_or(Error::Overflow("topology proof admission"))?;
        self.proof.verify_brep(&coordinator.materialize_brep(brep)?, limits, |live, pending| {
            let extra = live.checked_sub(baseline).ok_or(Error::Invalid("topology proof admission"))?;
            admit_scratch(extra, pending)
        })?;
        Ok(brep)
    }
}
impl TopologyAttestation {
    fn verify_source_binding(&self, source_sha256: [u8; 32], topology: &BrepTopology) -> Result<()> {
        if source_sha256 != self.source_sha256
            || om9_core::brep::source_topology_fingerprint(source_sha256, topology) != self.topology_sha256 {
            return Err(Error::Stale("topology source binding"));
        }
        Ok(())
    }
    /// Actual allocated descriptor/adjacency payload, not the B-rep storage.
    pub fn storage_bytes(&self) -> Result<u64> {
        let capacities = [Usage::array::<Descriptor>(self.faces.capacity())?.bytes,
            Usage::array::<Descriptor>(self.edges.capacity())?.bytes, Usage::array::<Descriptor>(self.vertices.capacity())?.bytes,
            Usage::array::<Incidence>(self.face_edges.capacity())?.bytes, Usage::array::<Incidence>(self.edge_vertices.capacity())?.bytes];
        capacities.into_iter().try_fold(0u64, |sum, value| sum.checked_add(value).ok_or(Error::Overflow("topology capture")))
    }
    fn manifest(&self) -> Manifest<'_> {
        Manifest { source_sha256: self.source_sha256, kernel: session_key(), faces: &self.faces, edges: &self.edges,
            vertices: &self.vertices, face_edges: &self.face_edges, edge_vertices: &self.edge_vertices }
    }
    /// Validate the exact B-rep before trusting its IDs. An unchanged pointer
    /// or object ID alone is insufficient: the display/topology hash is bound.
    pub fn verify_brep(&self, brep: &Brep, limits: &Limits,
        mut admit: impl FnMut(u64, u64) -> Result<()>) -> Result<()> {
        let held = self.storage_bytes()?;
        let remaining = limits.remaining_after(Usage { bytes: held, ..Usage::default() })?;
        brep.validate_admitted(&remaining, &mut |live, pending| {
            admit(held.checked_add(live).ok_or(Error::Overflow("topology admission"))?, pending)
        })?;
        let source: [u8; 32] = Sha256::digest(brep.source().as_bytes()).into();
        self.verify_source_binding(source, brep.topology()).map_err(|_| Error::Stale("topology attestation"))?;
        if brep.display_fingerprint() != self.display_sha256 {
            return Err(Error::Stale("topology attestation"));
        }
        Ok(())
    }
}

/// A numerical source signature bound to a previously attested topology ID.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundSourceSignature {
    pub id: Uuid,
    pub kind: om9_core::associative::SubshapeKind,
    pub signature: om9_core::associative::SubshapeSignature,
}

/// Match source-derived measurements to existing IDs only through a complete,
/// unambiguous oriented topology manifest captured from the *same* native root
/// as those measurements. Source/topology storage is charged by the caller;
/// this function accounts the retained proof, temporary manifests and output.
/// Neither the proof nor this binding is persistent across processes.
pub fn attested_source_signatures(source: &str, topology: &BrepTopology, proof: &TopologyAttestation,
    limits: &Limits, mut admit: impl FnMut(u64, u64) -> Result<()>, canceled: impl Fn() -> bool,
) -> Result<Vec<BoundSourceSignature>> {
    let source_hash: [u8; 32] = Sha256::digest(source.as_bytes()).into();
    proof.verify_source_binding(source_hash, topology)?;
    let proof_bytes = proof.storage_bytes()?;
    let remaining = limits.remaining_after(Usage { bytes: proof_bytes, ..Usage::default() })?;
    let admission = RefCell::new(&mut admit as &mut dyn FnMut(u64, u64) -> Result<()>);
    let mut descriptor_admit = |live: u64, pending: u64| {
        (admission.borrow_mut())(proof_bytes.checked_add(live).ok_or(Error::Overflow("source signature admission"))?, pending)
    };
    let descriptor_account = RefCell::new(&mut descriptor_admit as &mut dyn FnMut(u64, u64) -> Result<()>);
    let mut collector = Collector { limits: &remaining, admission: &descriptor_account, live: 0,
        faces: vec![], edges: vec![], vertices: vec![], face_edges: vec![], edge_vertices: vec![],
        active: None, error: None, shape_set: None };
    let descriptor_live = Cell::new(0u64);
    let values = {
        let mut capture = |handle: *const c_void| -> Result<()> {
            if canceled() { return Err(Error::Canceled); }
            if handle.is_null() { return Err(Error::Invalid("source topology handle")); }
            let status = unsafe { om9_occt_visit_snapshot_topology(handle, bound(remaining.max_nodes),
                bound(remaining.max_bytes), visit, (&mut collector as *mut Collector<'_, '_>).cast()) };
            if let Some(error) = collector.error.take() { return Err(error); }
            if status != 0 || collector.active.is_some() { return Err(Error::Invalid("source topology visitor")); }
            descriptor_live.set(collector.live);
            if canceled() { return Err(Error::Canceled); }
            Ok(())
        };
        let mut signature_admit = |live: u64, pending: u64| {
            let total = proof_bytes.checked_add(descriptor_live.get()).and_then(|v| v.checked_add(live))
                .ok_or(Error::Overflow("source signature admission"))?;
            if total.checked_add(pending).ok_or(Error::Overflow("source signature admission"))? > limits.max_bytes {
                return Err(Error::Budget("source signature bytes"));
            }
            (admission.borrow_mut())(total, pending)
        };
        super::source_signatures::source_signatures_with_handle(source, &remaining, &mut signature_admit,
            &canceled, Some(&mut capture))?
    };
    let signature_bytes = Usage::array::<super::source_signatures::SourceSignature>(values.capacity())?.bytes;
    let held = proof_bytes.checked_add(collector.live).and_then(|v| v.checked_add(signature_bytes))
        .ok_or(Error::Overflow("source signature storage"))?;
    let scratch = limits.remaining_after(Usage { bytes: held, ..Usage::default() })?;
    let manifest = Manifest { source_sha256: source_hash, kernel: session_key(), faces: &collector.faces,
        edges: &collector.edges, vertices: &collector.vertices, face_edges: &collector.face_edges,
        edge_vertices: &collector.edge_vertices };
    let mapping = match_manifests_admitted(&proof.manifest(), &manifest, &scratch, &mut |live, pending| {
        (admission.borrow_mut())(held.checked_add(live).ok_or(Error::Overflow("source signature admission"))?, pending)
    })?;
    if canceled() { return Err(Error::Canceled); }
    let base = held.checked_add(mapping.storage_bytes()?).ok_or(Error::Overflow("source signature storage"))?;
    let planned = Usage::array::<BoundSourceSignature>(values.len())?.bytes;
    limits.remaining_after(Usage { bytes: base.checked_add(planned).ok_or(Error::Overflow("source signature output"))?, ..Usage::default() })?;
    (admission.borrow_mut())(base, planned)?;
    let mut output = Vec::new();
    output.try_reserve_exact(values.len()).map_err(|_| Error::Budget("source signature output"))?;
    let actual = Usage::array::<BoundSourceSignature>(output.capacity())?.bytes;
    limits.remaining_after(Usage { bytes: base.checked_add(actual).ok_or(Error::Overflow("source signature output"))?, ..Usage::default() })?;
    (admission.borrow_mut())(base + actual, 0)?;
    use om9_core::associative::SubshapeKind;
    let groups = [(&topology.faces, mapping.faces(), 0usize, SubshapeKind::Face),
        (&topology.edges, mapping.edges(), collector.faces.len(), SubshapeKind::Edge),
        (&topology.vertex_ids, mapping.vertices(), collector.faces.len() + collector.edges.len(), SubshapeKind::Vertex)];
    for (ids, indices, offset, kind) in groups {
        if ids.len() != indices.len() { return Err(Error::Invalid("source signature identity count")); }
        for (id, index) in ids.iter().zip(indices) {
            let value = values.get(offset.checked_add(*index as usize).ok_or(Error::Overflow("source signature index"))?)
                .ok_or(Error::Invalid("source signature index"))?;
            if value.kind != kind || value.source_index != *index as usize { return Err(Error::Invalid("source signature order")); }
            output.push(BoundSourceSignature { id: *id, kind: value.kind, signature: value.signature });
        }
    }
    drop(mapping); drop(values); drop(collector);
    (admission.borrow_mut())(proof_bytes.checked_add(actual).ok_or(Error::Overflow("source signature output"))?, 0)?;
    Ok(output)
}
impl CapturedTopologySnapshot {
    pub fn brep(&self) -> &Brep { &self.brep }
    pub fn manifest(&self) -> Manifest<'_> {
        Manifest { source_sha256: self.source_sha256, kernel: session_key(), faces: &self.faces,
            edges: &self.edges, vertices: &self.vertices, face_edges: &self.face_edges, edge_vertices: &self.edge_vertices }
    }
    pub fn manifest_bytes(&self) -> Result<u64> {
        [Usage::array::<Descriptor>(self.faces.capacity())?.bytes, Usage::array::<Descriptor>(self.edges.capacity())?.bytes,
            Usage::array::<Descriptor>(self.vertices.capacity())?.bytes, Usage::array::<Incidence>(self.face_edges.capacity())?.bytes,
            Usage::array::<Incidence>(self.edge_vertices.capacity())?.bytes].into_iter()
            .try_fold(0u64, |sum, value| sum.checked_add(value).ok_or(Error::Overflow("topology capture")))
    }
    /// Move the actual snapshot into its owner while retaining only its proof.
    /// The caller must account the proof separately and check document stamps
    /// as well as `verify_brep` before publishing any later result.
    pub fn into_parts(self) -> (Brep, TopologyAttestation) {
        let Self { brep, source_sha256, faces, edges, vertices, face_edges, edge_vertices } = self;
        let attestation = TopologyAttestation { source_sha256,
            topology_sha256: om9_core::brep::source_topology_fingerprint(source_sha256, brep.topology()),
            display_sha256: brep.display_fingerprint(),
            faces, edges, vertices, face_edges, edge_vertices };
        (brep, attestation)
    }
    /// Produce candidate-order IDs only when the two source-bound snapshots
    /// match. This does not publish a display or authorize a stale document.
    /// `limits` and `admit` must include both captures and all other live data.
    pub fn match_topology(&self, candidate: &Self, limits: &Limits,
        mut admit: impl FnMut(u64, u64) -> Result<()>) -> Result<BrepTopology> {
        let own = self.manifest_bytes()?.checked_add(self.brep.storage_usage(limits)?.bytes)
            .ok_or(Error::Overflow("topology captures"))?;
        let other = if std::ptr::eq(self, candidate) { 0 } else {
            candidate.manifest_bytes()?.checked_add(candidate.brep.storage_usage(limits)?.bytes)
                .ok_or(Error::Overflow("topology captures"))?
        };
        let held = own.checked_add(other).ok_or(Error::Overflow("topology captures"))?;
        let scratch = limits.remaining_after(Usage { bytes: held, ..Usage::default() })?;
        let mapping = match_manifests_admitted(&self.manifest(), &candidate.manifest(), &scratch,
            &mut |live, pending| admit(held.checked_add(live).ok_or(Error::Overflow("topology admission"))?, pending))?;
        let mapping_bytes = mapping.storage_bytes()?;
        let base = held.checked_add(mapping_bytes).ok_or(Error::Overflow("topology admission"))?;
        let remaining = scratch.remaining_after(Usage { bytes: mapping_bytes, ..Usage::default() })?;
        let topology = mapping.reorder_topology_admitted(self.brep.topology(), &candidate.brep.topology().face_edges,
            &remaining, &mut |live, pending| admit(base.checked_add(live).ok_or(Error::Overflow("topology admission"))?, pending))?;
        let output = [Usage::array::<Uuid>(topology.faces.capacity())?.bytes, Usage::array::<Uuid>(topology.edges.capacity())?.bytes,
            Usage::array::<Uuid>(topology.vertex_ids.capacity())?.bytes, Usage::array::<[u32; 2]>(topology.face_edges.capacity())?.bytes]
            .into_iter().try_fold(0u64, |sum, value| sum.checked_add(value).ok_or(Error::Overflow("topology output")))?;
        drop(mapping);
        admit(held.checked_add(output).ok_or(Error::Overflow("topology admission"))?, 0)?;
        Ok(topology)
    }
    /// Consume an exclusive candidate and attach the verified original IDs.
    /// Keep its in-process manifest with the result for a later remesh. The
    /// candidate is still private: document/session freshness must be rechecked
    /// by the caller before publication or picking can use it.
    pub fn into_rebound_snapshot(self, original: &Self, limits: &Limits,
        mut admit: impl FnMut(u64, u64) -> Result<()>) -> Result<Self> {
        let topology = original.match_topology(&self, limits, &mut admit)?;
        let original_bytes = original.manifest_bytes()?.checked_add(original.brep.storage_usage(limits)?.bytes)
            .ok_or(Error::Overflow("topology original"))?;
        let held = original_bytes.checked_add(self.manifest_bytes()?).ok_or(Error::Overflow("topology admission"))?;
        let remaining = limits.remaining_after(Usage { bytes: held, ..Usage::default() })?;
        let Self { brep, source_sha256, faces, edges, vertices, face_edges, edge_vertices } = self;
        let brep = brep.rebind_candidate_topology(topology, &remaining, |live, pending| {
            admit(held.checked_add(live).ok_or(Error::Overflow("topology admission"))?, pending)
        })?;
        Ok(Self { brep, source_sha256, faces, edges, vertices, face_edges, edge_vertices })
    }
    /// Rebind to IDs of the exact B-rep that supplied an in-process proof.
    /// This creates a private candidate only; the owner must still check the
    /// current document/session stamp at publication and account the proof.
    pub fn into_rebound_for_brep(self, original: &Brep, proof: &TopologyAttestation, limits: &Limits,
        mut admit: impl FnMut(u64, u64) -> Result<()>) -> Result<Self> {
        self.into_rebound_for_brep_impl(original, proof, limits, &mut admit, false)
    }

    // The private ShapeSet path has already paired this packet with its source
    // at capture and verified its explicit native references. OCCT may write a
    // different byte representation after reading that packet; compare the
    // full topology manifests while retaining the original source identity.
    fn into_rebound_for_packet(self, original: &Brep, proof: &TopologyAttestation, limits: &Limits,
        mut admit: impl FnMut(u64, u64) -> Result<()>) -> Result<Self> {
        self.into_rebound_for_brep_impl(original, proof, limits, &mut admit, true)
    }

    fn into_rebound_for_brep_impl(self, original: &Brep, proof: &TopologyAttestation, limits: &Limits,
        admit: &mut dyn FnMut(u64, u64) -> Result<()>, paired_packet: bool) -> Result<Self> {
        let candidate_bytes = self.manifest_bytes()?.checked_add(self.brep.storage_usage(limits)?.bytes)
            .ok_or(Error::Overflow("topology candidate"))?;
        let for_original = limits.remaining_after(Usage { bytes: candidate_bytes, ..Usage::default() })?;
        proof.verify_brep(original, &for_original, |live, pending| {
            admit(candidate_bytes.checked_add(live).ok_or(Error::Overflow("topology admission"))?, pending)
        })?;
        let original_bytes = proof.storage_bytes()?.checked_add(original.storage_usage(limits)?.bytes)
            .ok_or(Error::Overflow("topology original"))?;
        let held = original_bytes.checked_add(candidate_bytes).ok_or(Error::Overflow("topology captures"))?;
        let scratch = limits.remaining_after(Usage { bytes: held, ..Usage::default() })?;
        let mut original_manifest = proof.manifest();
        if paired_packet { original_manifest.source_sha256 = self.source_sha256; }
        let mapping = match_manifests_admitted(&original_manifest, &self.manifest(), &scratch,
            &mut |live, pending| admit(held.checked_add(live).ok_or(Error::Overflow("topology admission"))?, pending))?;
        let mapping_bytes = mapping.storage_bytes()?;
        let base = held.checked_add(mapping_bytes).ok_or(Error::Overflow("topology admission"))?;
        let reorder_limits = scratch.remaining_after(Usage { bytes: mapping_bytes, ..Usage::default() })?;
        let topology = mapping.reorder_topology_admitted(original.topology(), &self.brep.topology().face_edges,
            &reorder_limits, &mut |live, pending| admit(base.checked_add(live).ok_or(Error::Overflow("topology admission"))?, pending))?;
        drop(mapping);
        let keep = original_bytes.checked_add(self.manifest_bytes()?).ok_or(Error::Overflow("topology admission"))?;
        let remaining = limits.remaining_after(Usage { bytes: keep, ..Usage::default() })?;
        let Self { brep, source_sha256, faces, edges, vertices, face_edges, edge_vertices } = self;
        let brep = brep.rebind_candidate_topology(topology, &remaining, |live, pending| {
            admit(keep.checked_add(live).ok_or(Error::Overflow("topology admission"))?, pending)
        })?;
        Ok(Self { brep, source_sha256, faces, edges, vertices, face_edges, edge_vertices })
    }

    /// Map a candidate using an already validated, tracked source-only lease.
    /// The lease and proof are charged separately by the coordinator.
    pub fn into_rebound_for_source(self, original: &RegisteredTopologySource, limits: &Limits,
        mut admit: impl FnMut(u64, u64) -> Result<()>) -> Result<Self> {
        original.proof.attestation().verify_source_binding(original.source.source_fingerprint(), original.source.topology())?;
        let candidate_bytes = self.manifest_bytes()?.checked_add(self.brep.storage_usage(limits)?.bytes)
            .ok_or(Error::Overflow("topology candidate"))?;
        let scratch = limits.remaining_after(Usage { bytes: candidate_bytes, ..Usage::default() })?;
        let mapping = match_manifests_admitted(&original.proof.attestation().manifest(), &self.manifest(), &scratch,
            &mut |live, pending| admit(candidate_bytes.checked_add(live).ok_or(Error::Overflow("topology admission"))?, pending))?;
        let mapping_bytes = mapping.storage_bytes()?;
        let base = candidate_bytes.checked_add(mapping_bytes).ok_or(Error::Overflow("topology admission"))?;
        let reorder_limits = scratch.remaining_after(Usage { bytes: mapping_bytes, ..Usage::default() })?;
        let topology = mapping.reorder_topology_admitted(original.source.topology(), &self.brep.topology().face_edges,
            &reorder_limits, &mut |live, pending| admit(base.checked_add(live).ok_or(Error::Overflow("topology admission"))?, pending))?;
        drop(mapping);
        let held = self.manifest_bytes()?;
        let remaining = limits.remaining_after(Usage { bytes: held, ..Usage::default() })?;
        let Self { brep, source_sha256, faces, edges, vertices, face_edges, edge_vertices } = self;
        let brep = brep.rebind_candidate_topology(topology, &remaining, |live, pending| {
            admit(held.checked_add(live).ok_or(Error::Overflow("topology admission"))?, pending)
        })?;
        Ok(Self { brep, source_sha256, faces, edges, vertices, face_edges, edge_vertices })
    }

    /// Transfer a freshly captured display to the canonical owner after a
    /// native-attested correspondence. The returned representation still needs
    /// an allocation owner and a guarded publication by its caller.
    pub fn into_representation_for_source(self, original: &RegisteredTopologySource,
        canonical: &om9_core::brep::CanonicalBrep, limits: &Limits,
        mut admit: impl FnMut(u64, u64) -> Result<()>) -> Result<om9_core::brep::BrepRepresentation> {
        original.proof.attestation().verify_source_binding(original.source.source_fingerprint(), original.source.topology())?;
        if canonical.source() != original.source.source() || canonical.topology() != original.source.topology() {
            return Err(Error::Stale("topology representation source"));
        }
        let candidate_bytes = self.manifest_bytes()?.checked_add(self.brep.storage_usage(limits)?.bytes)
            .ok_or(Error::Overflow("topology representation candidate"))?;
        let source_bytes = canonical.storage_usage(limits)?.bytes;
        let held = candidate_bytes.checked_add(source_bytes).ok_or(Error::Overflow("topology representation"))?;
        let scratch = limits.remaining_after(Usage { bytes: held, ..Usage::default() })?;
        let mapping = match_manifests_admitted(&original.proof.attestation().manifest(), &self.manifest(), &scratch,
            &mut |live, pending| admit(held.checked_add(live).ok_or(Error::Overflow("topology representation"))?, pending))?;
        let mapping_bytes = mapping.storage_bytes()?;
        let base = held.checked_add(mapping_bytes).ok_or(Error::Overflow("topology representation"))?;
        let remaining = scratch.remaining_after(Usage { bytes: mapping_bytes, ..Usage::default() })?;
        let topology = mapping.reorder_topology_admitted(canonical.topology(), &self.brep.topology().face_edges,
            &remaining, &mut |live, pending| admit(base.checked_add(live).ok_or(Error::Overflow("topology representation"))?, pending))?;
        let Self { brep, faces, edges, vertices, face_edges, edge_vertices, .. } = self;
        drop((faces, edges, vertices, face_edges, edge_vertices));
        let held = source_bytes.checked_add(mapping_bytes).ok_or(Error::Overflow("topology representation"))?;
        let remaining = limits.remaining_after(Usage { bytes: held, ..Usage::default() })?;
        let brep = brep.rebind_candidate_topology(topology, &remaining, |live, pending| {
            admit(held.checked_add(live).ok_or(Error::Overflow("topology representation"))?, pending)
        })?;
        let remaining = limits.remaining_after(Usage { bytes: mapping_bytes, ..Usage::default() })?;
        brep.into_representation_for_canonical(canonical, &mapping, &remaining, |live, pending| {
            admit(mapping_bytes.checked_add(live).ok_or(Error::Overflow("topology representation"))?, pending)
        })
    }
}

struct Collector<'a, 'b> {
    limits: &'a Limits, admission: &'a SnapshotAdmission<'b>, live: u64,
    faces: Vec<Descriptor>, edges: Vec<Descriptor>, vertices: Vec<Descriptor>,
    face_edges: Vec<Incidence>, edge_vertices: Vec<Incidence>,
    active: Option<(i32, u32, i32, Sha256)>, error: Option<Error>,
    shape_set: Option<Vec<u8>>,
}

fn append<T>(vec: &mut Vec<T>, item: T, live: &mut u64, limits: &Limits, admission: &SnapshotAdmission<'_>) -> Result<()> {
    if vec.len() == vec.capacity() {
        let target = vec.len().checked_mul(2).unwrap_or(usize::MAX).max(4).min(bound(limits.max_nodes));
        if target <= vec.len() { return Err(Error::Budget("topology capture nodes")); }
        let pending = Usage::array::<T>(target)?.bytes;
        if live.checked_add(pending).ok_or(Error::Overflow("topology capture"))? > limits.max_bytes {
            return Err(Error::Budget("topology capture bytes"));
        }
        (admission.borrow_mut())(*live, pending)?;
        let old = Usage::array::<T>(vec.capacity())?.bytes;
        vec.try_reserve_exact(target - vec.len()).map_err(|_| Error::Budget("topology capture allocation"))?;
        let new = Usage::array::<T>(vec.capacity())?.bytes;
        *live = live.checked_sub(old).and_then(|v| v.checked_add(new)).ok_or(Error::Overflow("topology capture"))?;
        if *live > limits.max_bytes { return Err(Error::Budget("topology capture capacity")); }
        (admission.borrow_mut())(*live, 0)?;
    }
    vec.push(item);
    Ok(())
}

unsafe extern "C" fn visit(context: *mut c_void, event: c_int, kind: c_int, index: u32,
    child: u32, orientation: c_int, data: *const u8, len: usize) -> c_int {
    let state = unsafe { &mut *context.cast::<Collector<'_, '_>>() };
    if state.error.is_some() { return 1; }
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<()> {
        match event {
            0 => {
                if !(0..=2).contains(&kind) || !(0..=3).contains(&orientation) || state.active.is_some() {
                    return Err(Error::Invalid("topology descriptor begin"));
                }
                let mut hash = Sha256::new(); hash.update([kind as u8, orientation as u8]);
                state.active = Some((kind, index, orientation, hash));
            }
            1 => {
                if len > isize::MAX as usize || (data.is_null() && len != 0) { return Err(Error::Invalid("topology descriptor bytes")); }
                let bytes = if len == 0 { &[] } else { unsafe { std::slice::from_raw_parts(data, len) } };
                state.active.as_mut().ok_or(Error::Invalid("topology descriptor state"))?.3.update(bytes);
            }
            2 => {
                let (begin_kind, begin_index, begin_orientation, hash) = state.active.take().ok_or(Error::Invalid("topology descriptor state"))?;
                if (kind, index, orientation) != (begin_kind, begin_index, begin_orientation) { return Err(Error::Invalid("topology descriptor end")); }
                let vec = match kind { 0 => &mut state.faces, 1 => &mut state.edges, 2 => &mut state.vertices, _ => return Err(Error::Invalid("topology descriptor kind")) };
                if vec.len() != index as usize { return Err(Error::Invalid("topology descriptor index")); }
                append(vec, hash.finalize().into(), &mut state.live, state.limits, state.admission)?;
            }
            3 => {
                if state.active.is_some() || !(0..=3).contains(&orientation) { return Err(Error::Invalid("topology incidence")); }
                let vec = match kind { 0 => &mut state.face_edges, 1 => &mut state.edge_vertices, _ => return Err(Error::Invalid("topology incidence kind")) };
                append(vec, Incidence { parent: index, child, orientation: orientation as u8 }, &mut state.live, state.limits, state.admission)?;
            }
            _ => return Err(Error::Invalid("topology event")),
        }
        Ok(())
    })).unwrap_or(Err(Error::Invalid("topology capture panic")));
    if let Err(error) = result { state.error = Some(error); 1 } else { 0 }
}

unsafe extern "C" fn receive_shape_set(context: *mut c_void, event: c_int, _kind: c_int, _index: u32,
    _child: u32, _orientation: c_int, data: *const u8, len: usize) -> c_int {
    let state = unsafe { &mut *context.cast::<Collector<'_, '_>>() };
    if state.error.is_some() { return 1; }
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<()> {
        if event != 1 || len > isize::MAX as usize || (data.is_null() && len != 0) {
            return Err(Error::Invalid("shape set stream"));
        }
        let bytes = if len == 0 { &[] } else { unsafe { std::slice::from_raw_parts(data, len) } };
        let packet = state.shape_set.as_mut().ok_or(Error::Invalid("shape set capture"))?;
        let needed = packet.len().checked_add(len).ok_or(Error::Overflow("shape set packet"))?;
        if needed as u128 > u128::from(state.limits.max_bytes) { return Err(Error::Budget("shape set packet")); }
        if needed > packet.capacity() {
            let limit = bound(state.limits.max_bytes);
            let target = packet.capacity().saturating_mul(2).max(needed).max(256).min(limit);
            let pending = Usage::array::<u8>(target)?.bytes;
            if state.live.checked_add(pending).ok_or(Error::Overflow("shape set packet"))? > state.limits.max_bytes {
                return Err(Error::Budget("shape set packet"));
            }
            (state.admission.borrow_mut())(state.live, pending)?;
            let before = packet.capacity() as u64;
            packet.try_reserve_exact(target - packet.len()).map_err(|_| Error::Budget("shape set packet"))?;
            state.live = state.live.checked_sub(before).and_then(|v| v.checked_add(packet.capacity() as u64))
                .ok_or(Error::Overflow("shape set packet"))?;
            if state.live > state.limits.max_bytes { return Err(Error::Budget("shape set packet")); }
            (state.admission.borrow_mut())(state.live, 0)?;
        }
        packet.extend_from_slice(bytes);
        Ok(())
    })).unwrap_or(Err(Error::Invalid("shape set capture panic")));
    if let Err(error) = result { state.error = Some(error); 1 } else { 0 }
}

struct ShapeSetRestore<'a, 'b> {
    limits: &'a Limits,
    admission: &'a SnapshotAdmission<'b>,
    live: u64,
    mapping: [Vec<u32>; 3],
    deflection: f64,
    result: Option<Result<Brep>>,
    proof: Option<TopologyAttestation>,
    error: Option<Error>,
}
unsafe extern "C" fn receive_shape_mapping(context: *mut c_void, event: c_int, kind: c_int, index: u32,
    mapped: u32, orientation: c_int, _data: *const u8, len: usize) -> c_int {
    let state = unsafe { &mut *context.cast::<ShapeSetRestore<'_, '_>>() };
    if state.error.is_some() { return 1; }
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<()> {
        if event != 0 || !(0..=2).contains(&kind) || !(0..=3).contains(&orientation) || len != 0 {
            return Err(Error::Invalid("shape set mapping"));
        }
        let values = &mut state.mapping[kind as usize];
        if values.len() != index as usize { return Err(Error::Invalid("shape set mapping order")); }
        append(values, mapped, &mut state.live, state.limits, state.admission)
    })).unwrap_or(Err(Error::Invalid("shape set mapping panic")));
    if let Err(error) = result { state.error = Some(error); 1 } else { 0 }
}
unsafe extern "C" fn receive_shape_snapshot(context: *mut c_void, snapshot: *const Snapshot) -> c_int {
    let state = unsafe { &mut *context.cast::<ShapeSetRestore<'_, '_>>() };
    if state.error.is_some() || state.result.is_some() || snapshot.is_null() { return 1; }
    let snapshot_ref = unsafe { &*snapshot };
    if snapshot_ref.topology_handle.is_null() {
        state.error = Some(Error::Invalid("shape set topology handle"));
        return 1;
    }
    let mut collector = Collector { limits: state.limits, admission: state.admission, live: state.live,
        faces: vec![], edges: vec![], vertices: vec![], face_edges: vec![], edge_vertices: vec![],
        active: None, error: None, shape_set: None };
    let status = unsafe { om9_occt_visit_snapshot_topology(snapshot_ref.topology_handle,
        bound(state.limits.max_nodes), bound(state.limits.max_bytes), visit,
        (&mut collector as *mut Collector<'_, '_>).cast()) };
    if let Some(error) = collector.error.take() { state.error = Some(error); return 1; }
    if status != 0 || collector.active.is_some() || collector.faces.len() != snapshot_ref.faces_count
        || collector.edges.len() != snapshot_ref.edges_count || collector.vertices.len() != snapshot_ref.vertices_len {
        state.error = Some(Error::Invalid("shape set topology descriptors"));
        return 1;
    }
    let account = Cell::new(collector.live);
    let mut base = Context { limits: state.limits, deflection: state.deflection, result: None,
        grouped_bytes: Some(&account), admission: Some(state.admission) };
    let status = unsafe { receive((&mut base as *mut Context<'_, '_>).cast(), snapshot) };
    state.live = account.get();
    if let Some(Ok(brep)) = base.result.as_ref() {
        let source_sha256 = Sha256::digest(brep.source().as_bytes()).into();
        state.proof = Some(TopologyAttestation { source_sha256,
            topology_sha256: om9_core::brep::source_topology_fingerprint(source_sha256, brep.topology()),
            display_sha256: brep.display_fingerprint(), faces: collector.faces, edges: collector.edges,
            vertices: collector.vertices, face_edges: collector.face_edges, edge_vertices: collector.edge_vertices });
    }
    state.result = base.result;
    status
}

struct Capture<'a, 'b> {
    collector: Collector<'a, 'b>, deflection: f64,
    result: Option<Result<Brep>>,
}
unsafe extern "C" fn receive_capture(context: *mut c_void, snapshot: *const Snapshot) -> c_int {
    let state = unsafe { &mut *context.cast::<Capture<'_, '_>>() };
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<()> {
        if snapshot.is_null() || state.result.is_some() { return Err(Error::Invalid("topology snapshot")); }
        let snapshot = unsafe { &*snapshot };
        if snapshot.topology_handle.is_null() { return Err(Error::Unsupported("snapshot topology receipt")); }
        let status = unsafe { om9_occt_visit_snapshot_topology(snapshot.topology_handle, bound(state.collector.limits.max_nodes),
            bound(state.collector.limits.max_bytes), visit, (&mut state.collector as *mut Collector<'_, '_>).cast()) };
        if let Some(error) = state.collector.error.take() { return Err(error); }
        if status != 0 || state.collector.active.is_some() { return Err(Error::Invalid("topology visitor")); }
        if state.collector.faces.len() != snapshot.faces_count || state.collector.edges.len() != snapshot.edges_count
            || state.collector.vertices.len() != snapshot.vertices_len { return Err(Error::Invalid("topology descriptor count")); }
        if state.collector.shape_set.is_some() {
            let status = unsafe { om9_occt_write_snapshot_shape_set(snapshot.topology_handle,
                bound(state.collector.limits.max_nodes), bound(state.collector.limits.max_bytes),
                receive_shape_set, (&mut state.collector as *mut Collector<'_, '_>).cast()) };
            if let Some(error) = state.collector.error.take() { return Err(error); }
            if status != 0 { return Err(Error::Invalid("shape set writer")); }
        }
        let account = Cell::new(state.collector.live);
        let mut base = Context { limits: state.collector.limits, deflection: state.deflection, result: None,
            grouped_bytes: Some(&account), admission: Some(state.collector.admission) };
        let status = unsafe { receive((&mut base as *mut Context<'_, '_>).cast(), snapshot) };
        state.result = base.result;
        if status != 0 { return Err(Error::Invalid("topology snapshot copy")); }
        Ok(())
    })).unwrap_or(Err(Error::Invalid("topology snapshot panic")));
    if let Err(error) = result {
        if state.result.as_ref().is_none_or(|r| r.is_ok()) { state.result = Some(Err(error)); }
        1
    } else { 0 }
}

/// Capture the display and topology in one synchronous source read. The caller
/// must reserve all other live allocations and use `admit` to account this
/// capture; native OCCT internals remain outside the managed byte ceiling.
/// No returned descriptor is a persistent or cross-process attestation.
pub fn capture_source_topology(source: &str, deflection: f64, angular: f64, limits: &Limits,
    admit: impl FnMut(u64, u64) -> Result<()>) -> std::result::Result<CapturedTopologySnapshot, KernelError> {
    if source.is_empty() || source.len() as u64 > limits.max_bytes || !deflection.is_finite() || deflection <= 0.
        || !angular.is_finite() || angular <= 0. || angular > std::f64::consts::PI {
        return Err(KernelError::InvalidInput("topology capture input".into()));
    }
    let dimensions = [0.; 3];
    capture_via_topology(deflection, limits, admit, |sink, context, message, capacity| unsafe {
        om9_occt_brep_snapshot(source.as_ptr(), source.len(), 0, dimensions.as_ptr(), deflection, angular,
            bound(limits.max_points), bound(limits.max_nodes), bound(limits.max_bytes), sink, context, message, capacity)
    })
}

/// Capture a native shape set and its UUID-bearing display in one callback.
/// Native OCCT allocations are not covered by this Rust admission; the packet
/// is not yet a trusted archive format and no project codec uses it.
pub fn capture_source_with_shape_set(source: &str, deflection: f64, angular: f64, limits: &Limits,
    admit: impl FnMut(u64, u64) -> Result<()>) -> std::result::Result<NamedTopologySnapshot, KernelError> {
    if source.is_empty() || source.len() as u64 > limits.max_bytes || !deflection.is_finite() || deflection <= 0.
        || !angular.is_finite() || angular <= 0. || angular > std::f64::consts::PI {
        return Err(KernelError::InvalidInput("shape set capture input".into()));
    }
    let dimensions = [0.; 3];
    let (snapshot, shape_set) = capture_via_topology_impl(deflection, limits, admit, true,
        |sink, context, message, capacity| unsafe {
            om9_occt_brep_snapshot(source.as_ptr(), source.len(), 0, dimensions.as_ptr(), deflection, angular,
                bound(limits.max_points), bound(limits.max_nodes), bound(limits.max_bytes), sink, context, message, capacity)
        })?;
    let shape_set = shape_set.ok_or_else(|| KernelError::InvalidInput("shape set missing".into()))?;
    Ok(NamedTopologySnapshot { snapshot, shape_set })
}

/// Capture proof while a native constructor's exact snapshot is still live.
/// The native operation is synchronous and must invoke the sink at most once.
pub(crate) fn capture_via_topology(deflection: f64, limits: &Limits,
    admit: impl FnMut(u64, u64) -> Result<()>, invoke: impl FnOnce(
        unsafe extern "C" fn(*mut c_void, *const Snapshot) -> c_int, *mut c_void, *mut c_char, c_int) -> c_int,
) -> std::result::Result<CapturedTopologySnapshot, KernelError> {
    capture_via_topology_impl(deflection, limits, admit, false, invoke).map(|(snapshot, _)| snapshot)
}
fn capture_via_topology_impl(deflection: f64, limits: &Limits,
    mut admit: impl FnMut(u64, u64) -> Result<()>, named: bool, invoke: impl FnOnce(
        unsafe extern "C" fn(*mut c_void, *const Snapshot) -> c_int, *mut c_void, *mut c_char, c_int) -> c_int,
) -> std::result::Result<(CapturedTopologySnapshot, Option<Vec<u8>>), KernelError> {
    if !deflection.is_finite() || deflection <= 0. { return Err(KernelError::InvalidInput("topology capture input".into())); }
    let admission = RefCell::new(&mut admit as &mut dyn FnMut(u64, u64) -> Result<()>);
    let mut capture = Capture { collector: Collector { limits, admission: &admission, live: 0,
        faces: vec![], edges: vec![], vertices: vec![], face_edges: vec![], edge_vertices: vec![], active: None, error: None,
        shape_set: named.then(Vec::new) },
        deflection, result: None };
    let mut message = [0i8; 512];
    let status = invoke(receive_capture, (&mut capture as *mut Capture<'_, '_>).cast(), message.as_mut_ptr(), 512);
    if let Some(Err(error)) = capture.result.as_ref() { return Err(KernelError::InvalidInput(error.to_string())); }
    if status != 0 { return Err(KernelError::Failure(unsafe { CStr::from_ptr(message.as_ptr()) }.to_string_lossy().into_owned())); }
    let brep = capture.result.ok_or_else(|| KernelError::InvalidInput("topology snapshot missing".into()))?
        .map_err(|error| KernelError::InvalidInput(error.to_string()))?;
    let source_sha256 = Sha256::digest(brep.source().as_bytes()).into();
    let collector = capture.collector;
    Ok((CapturedTopologySnapshot { brep, source_sha256, faces: collector.faces, edges: collector.edges,
        vertices: collector.vertices, face_edges: collector.face_edges, edge_vertices: collector.edge_vertices }, collector.shape_set))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn named_source_measures_original_ids_without_display() {
        let limits = Limits::default();
        let seed = super::super::box_brep([2., 3., 4.], 0.1, &limits).unwrap();
        let original = capture_source_topology(seed.source(), 0.1, 0.3, &limits, |_, _| Ok(())).unwrap();
        let (brep, proof) = original.into_parts();
        let expected_vertices: Vec<_> = brep.topology().vertex_ids.iter().copied()
            .zip(brep.display().vertices.iter().map(|index| brep.display().points[*index as usize])).collect();
        let expected_ids: Vec<_> = brep.topology().faces.iter().chain(&brep.topology().edges)
            .chain(&brep.topology().vertex_ids).copied().collect();
        let named = capture_source_with_shape_set(brep.source(), 0.02, 0.3, &limits, |_, _| Ok(())).unwrap()
            .into_rebound_for_brep(&brep, &proof, &limits, |_, _| Ok(())).unwrap();
        let display = named.snapshot().brep().display_liveness();
        let source = named.into_source();
        assert!(!display.is_alive());
        drop(brep); drop(seed); drop(proof);
        let values = source.signatures(&limits, |_, _| Ok(()), || false).unwrap();
        assert_eq!(values.iter().map(|value| value.id).collect::<Vec<_>>(), expected_ids);
        for (id, point) in expected_vertices {
            let value = values.iter().find(|value| value.id == id).unwrap();
            assert_eq!(value.kind, om9_core::associative::SubshapeKind::Vertex);
            assert_eq!(value.signature.start, point);
        }
        let area: f64 = values.iter().filter(|value| value.kind == om9_core::associative::SubshapeKind::Face)
            .map(|value| value.signature.measure_mm).sum();
        assert!((area - 52.).abs() < 1e-8);
        assert!(matches!(source.signatures(&limits, |_, _| Ok(()), || true), Err(Error::Canceled)));
        assert!(matches!(source.signatures(&limits, |_, _| Err(Error::Budget("test")), || false), Err(Error::Budget("test"))));
        let calls = Cell::new(0);
        assert!(matches!(source.signatures(&limits, |_, _| Ok(()), || {
            calls.set(calls.get() + 1); calls.get() > 4
        }), Err(Error::Canceled)));
        assert!(calls.get() > 4);
        let initial = Cell::new(true);
        assert!(matches!(source.signatures(&limits, |_, _| {
            if initial.replace(false) { Ok(()) } else { Err(Error::Budget("mapping")) }
        }, || false), Err(Error::Budget("mapping"))));
        let mut corrupt = source;
        corrupt.shape_set[0] ^= 1;
        assert!(corrupt.signatures(&limits, |_, _| Ok(()), || false).is_err());
    }

    #[test]
    fn named_sphere_source_area_matches_original_without_display() {
        let limits = Limits::default();
        let seed = super::super::sphere_brep(5., 0.1, &limits).unwrap();
        let captured = capture_source_with_shape_set(seed.source(), 0.05, 0.3, &limits, |_, _| Ok(())).unwrap();
        let face_id = captured.snapshot().brep().topology().faces[0];
        let display = captured.snapshot().brep().display_liveness();
        let source = captured.into_source();
        drop(seed);
        assert!(!display.is_alive());
        let values = source.signatures(&limits, |_, _| Ok(()), || false).unwrap();
        let face = values.iter().find(|value| value.id == face_id).unwrap();
        assert_eq!(face.kind, om9_core::associative::SubshapeKind::Face);
        assert!((face.signature.measure_mm - 100. * std::f64::consts::PI).abs() < 1e-7);
    }

    #[test]
    fn source_signatures_refuse_postmesh_proof_without_retaining_display() {
        let limits = Limits::default();
        let seed = super::super::box_brep([2., 3., 4.], 0.1, &limits).unwrap();
        let captured = capture_source_topology(seed.source(), 0.1, 0.3, &limits, |_, _| Ok(())).unwrap();
        drop(seed);
        let (brep, proof) = captured.into_parts();
        let display = brep.display_liveness();
        let source = brep.into_owned_source();
        assert!(!display.is_alive());
        assert!(matches!(attested_source_signatures(source.source(), source.topology(), &proof, &limits,
            |_, _| Ok(()), || false), Err(Error::Invalid("topology descriptor mismatch"))));
        let mut foreign = source.topology().clone();
        foreign.edges[0] = Uuid::new_v4();
        assert!(matches!(attested_source_signatures(source.source(), &foreign, &proof, &limits, |_, _| Ok(()), || false), Err(Error::Stale(_))));
        assert!(matches!(attested_source_signatures("foreign", source.topology(), &proof, &limits, |_, _| Ok(()), || false), Err(Error::Stale(_))));
        assert!(matches!(attested_source_signatures(source.source(), source.topology(), &proof, &limits,
            |_, _| Err(Error::Budget("test")), || false), Err(Error::Budget("test"))));
        assert!(matches!(attested_source_signatures(source.source(), source.topology(), &proof, &limits,
            |_, _| Ok(()), || true), Err(Error::Canceled)));
    }

    #[test]
    fn source_signatures_mapping_mechanics_with_synthetic_manifest() {
        let limits = Limits::default();
        let seed = super::super::box_brep([2., 3., 4.], 0.1, &limits).unwrap();
        let captured = capture_source_topology(seed.source(), 0.1, 0.3, &limits, |_, _| Ok(())).unwrap();
        let (brep, mut proof) = captured.into_parts();
        let mut admission_fn = |_, _| Ok(());
        let admission = RefCell::new(&mut admission_fn as &mut dyn FnMut(u64, u64) -> Result<()>);
        let mut collector = Collector { limits: &limits, admission: &admission, live: 0,
            faces: vec![], edges: vec![], vertices: vec![], face_edges: vec![], edge_vertices: vec![],
            active: None, error: None, shape_set: None };
        let mut visitor = |handle| {
            let status = unsafe { om9_occt_visit_snapshot_topology(handle, bound(limits.max_nodes), bound(limits.max_bytes),
                visit, (&mut collector as *mut Collector<'_, '_>).cast()) };
            if status != 0 { return Err(Error::Invalid("test source visitor")); }
            Ok(())
        };
        super::super::source_signatures::source_signatures_with_handle(seed.source(), &limits,
            &mut |_, _| Ok(()), &|| false, Some(&mut visitor)).unwrap();
        // Deliberately fabricated only to test mapping mechanics. Replacing a
        // real proof's descriptors this way is not a valid identity binding.
        proof.faces = collector.faces; proof.edges = collector.edges; proof.vertices = collector.vertices;
        proof.face_edges = collector.face_edges; proof.edge_vertices = collector.edge_vertices;
        let values = attested_source_signatures(brep.source(), brep.topology(), &proof, &limits,
            |_, _| Ok(()), || false).unwrap();
        assert_eq!(values.len(), 26);
        assert_eq!(values.iter().map(|v| v.id).collect::<Vec<_>>(),
            brep.topology().faces.iter().chain(&brep.topology().edges).chain(&brep.topology().vertex_ids).copied().collect::<Vec<_>>());
        let area: f64 = values.iter().filter(|v| v.kind == om9_core::associative::SubshapeKind::Face)
            .map(|v| v.signature.measure_mm).sum();
        assert!((area - 52.).abs() < 1e-8);
    }

    #[test]
    fn attestation_checks_source_and_topology_without_retaining_display() {
        let limits = Limits::default();
        let captured = crate::creation::primitive_with_topology(
            crate::creation::Primitive::Sphere { radius_mm: 5. }, 0.1, &limits, |_, _| Ok(())).unwrap();
        let (brep, proof) = captured.into_parts();
        let display = brep.display_liveness();
        let topology = brep.topology().clone();
        let source = proof.source_sha256;
        drop(brep);
        assert!(!display.is_alive());
        assert!(proof.verify_source_binding(source, &topology).is_ok());
        let mut foreign = topology.clone();
        foreign.faces[0] = Uuid::new_v4();
        assert!(matches!(proof.verify_source_binding(source, &foreign), Err(Error::Stale("topology source binding"))));
        let mut changed_source = source;
        changed_source[0] ^= 1;
        assert!(matches!(proof.verify_source_binding(changed_source, &topology), Err(Error::Stale("topology source binding"))));
    }
    use om9_core::brep::topology_mapping::match_manifests;
    #[test]
    fn shape_set_source_rebinds_existing_attested_ids() {
        let limits = Limits::default();
        let seed = super::super::sphere_brep(3., 0.1, &limits).unwrap();
        let reread = capture_source_topology(seed.source(), 0.1, 0.3, &limits, |_, _| Ok(())).unwrap().into_parts();
        let created = crate::creation::primitive_with_topology(
            crate::creation::Primitive::Sphere { radius_mm: 5. }, 0.1, &limits, |_, _| Ok(())).unwrap().into_parts();
        for (existing, proof) in [reread, created] {
            let named = capture_source_with_shape_set(existing.source(), 0.02, 0.3, &limits, |_, _| Ok(())).unwrap();
            let expected = existing.topology().clone();
            let named = named.into_rebound_for_brep(&existing, &proof, &limits, |_, _| Ok(())).unwrap();
            let old_display = named.snapshot().brep().display_liveness();
            let source = named.into_source();
            assert!(!old_display.is_alive());
            assert_eq!(source.source(), existing.source());
            let restored = source.restore(0.05, 0.3, &limits, |_, _| Ok(())).unwrap();
            assert_eq!(restored.source(), existing.source(), "remesh must preserve the captured source");
            let ids = |values: &[Uuid]| values.iter().copied().collect::<std::collections::BTreeSet<_>>();
            assert_eq!(ids(&restored.topology().faces), ids(&expected.faces));
            assert_eq!(ids(&restored.topology().edges), ids(&expected.edges));
            assert_eq!(ids(&restored.topology().vertex_ids), ids(&expected.vertex_ids));
            let other = super::super::sphere_brep(4., 0.1, &limits).unwrap();
            assert!(restore_shape_set(source.shape_set(), other.source(), source.topology(), source.storage_bytes().unwrap(),
                0.05, 0.3, &limits, |_, _| Ok(())).is_err(), "a foreign source must not inherit packet IDs");
            assert_eq!(source.source(), existing.source());
            let named = capture_source_with_shape_set(other.source(), 0.1, 0.3, &limits, |_, _| Ok(())).unwrap();
            assert!(named.into_rebound_for_brep(&existing, &proof, &limits, |_, _| Ok(())).is_err());
        }
    }
    #[test]
    fn shape_set_packet_is_captured_with_the_same_brep_and_budget() {
        let limits = Limits::default();
        for seed in [super::super::box_brep([4., 5., 6.], 0.1, &limits).unwrap(),
            super::super::sphere_brep(3., 0.1, &limits).unwrap()] {
            let mut peak = 0u64;
            let named = capture_source_with_shape_set(seed.source(), 0.1, 0.3, &limits, |live, pending| {
                peak = peak.max(live.checked_add(pending).ok_or(Error::Overflow("test admission"))?);
                Ok(())
            }).unwrap();
            assert!(named.shape_set().starts_with(b"OM9-SHAPESET 1\n"));
            assert_eq!(named.snapshot().brep().source(), seed.source());
            assert_eq!(named.snapshot().brep().display().points, seed.display().points);
            assert!(peak >= named.storage_bytes(&limits).unwrap());
            let restored = named.restore(0.02, 0.3, &limits, |_, _| Ok(())).unwrap();
            let ids = |values: &[Uuid]| values.iter().copied().collect::<std::collections::BTreeSet<_>>();
            assert_eq!(ids(&restored.topology().faces), ids(&named.snapshot().brep().topology().faces));
            assert_eq!(ids(&restored.topology().edges), ids(&named.snapshot().brep().topology().edges));
            assert_eq!(ids(&restored.topology().vertex_ids), ids(&named.snapshot().brep().topology().vertex_ids));
            let original_measures = crate::creation::measure(named.snapshot().brep(), &limits).unwrap();
            let restored_measures = crate::creation::measure(&restored, &limits).unwrap();
            assert!((original_measures.volume_mm3 - restored_measures.volume_mm3).abs() < 1e-9);
            assert!((original_measures.area_mm2 - restored_measures.area_mm2).abs() < 1e-9);
            assert_eq!((original_measures.faces, original_measures.edges, original_measures.vertices),
                (restored_measures.faces, restored_measures.edges, restored_measures.vertices));
            assert!(named.restore(0.02, 0.3, &limits, |_, _| Err(Error::Budget("test refusal"))).is_err());
            assert!(capture_source_with_shape_set(seed.source(), 0.1, 0.3, &limits,
                |_, _| Err(Error::Budget("test refusal"))).is_err());
            let old_display = named.snapshot().brep().display_liveness();
            let expected_faces = ids(&named.snapshot().brep().topology().faces);
            let expected_edges = ids(&named.snapshot().brep().topology().edges);
            let expected_vertices = ids(&named.snapshot().brep().topology().vertex_ids);
            let mut face_probes = Vec::new();
            let original = named.snapshot().brep();
            for (index, face) in original.topology().faces.iter().enumerate() {
                let triangle = original.display().triangles.iter().find(|t| t[3] as usize == index).unwrap();
                let [a, b, c] = [triangle[0], triangle[1], triangle[2]].map(|i| original.display().points[i as usize]);
                let u = std::array::from_fn::<_, 3, _>(|i| b[i] - a[i]);
                let v = std::array::from_fn::<_, 3, _>(|i| c[i] - a[i]);
                let normal = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]];
                let length = normal[0].hypot(normal[1]).hypot(normal[2]);
                assert!(length > 0.);
                let direction = normal.map(|value| -value / length);
                let center = std::array::from_fn(|i| (a[i] + b[i] + c[i]) / 3.);
                let origin = std::array::from_fn(|i| center[i] - direction[i]);
                let hit = crate::measurement::brep_face_ray_hit(original, *face,
                    om9_core::geometry::Transform::identity(), origin, direction, center, 1e-6, &limits).unwrap();
                face_probes.push((*face, origin, direction, center, hit));
            }
            let before = named.storage_bytes(&limits).unwrap();
            let source = named.into_source();
            assert!(!old_display.is_alive(), "source-only owner must release its display");
            assert_eq!(source.source(), seed.source());
            assert!(source.storage_bytes().unwrap() < before);
            let rebuilt = source.restore(0.02, 0.3, &limits, |_, _| Ok(())).unwrap();
            assert_eq!(ids(&rebuilt.topology().faces), expected_faces);
            assert_eq!(ids(&rebuilt.topology().edges), expected_edges);
            assert_eq!(ids(&rebuilt.topology().vertex_ids), expected_vertices);
            let rebuilt_measures = crate::creation::measure(&rebuilt, &limits).unwrap();
            assert!((original_measures.volume_mm3 - rebuilt_measures.volume_mm3).abs() < 1e-9);
            assert!((original_measures.area_mm2 - rebuilt_measures.area_mm2).abs() < 1e-9);
            for (face, origin, direction, center, expected) in face_probes {
                let hit = crate::measurement::brep_face_ray_hit(&rebuilt, face,
                    om9_core::geometry::Transform::identity(), origin, direction, center, 1e-6, &limits).unwrap();
                assert!((0..3).map(|i| (hit[i] - expected[i]).powi(2)).sum::<f64>().sqrt() < 1e-7);
            }
        }
    }
    #[test]
    fn shape_set_restore_rejects_corruption_without_changing_capture() {
        let limits = Limits::default();
        let seed = super::super::box_brep([4., 5., 6.], 0.1, &limits).unwrap();
        let mut named = capture_source_with_shape_set(seed.source(), 0.1, 0.3, &limits, |_, _| Ok(())).unwrap();
        let fingerprint = named.snapshot().brep().display_fingerprint();
        named.shape_set[0] ^= 1;
        assert!(named.restore(0.1, 0.3, &limits, |_, _| Ok(())).is_err());
        assert_eq!(named.snapshot().brep().display_fingerprint(), fingerprint);
    }
    #[test]
    fn source_capture_binds_real_snapshots_and_admits_owned_storage() {
        let limits = Limits::default();
        for seed in [super::super::box_brep([4., 5., 6.], 0.1, &limits).unwrap(),
            super::super::sphere_brep(3., 0.1, &limits).unwrap()] {
            let mut peak = 0;
            let first = capture_source_topology(seed.source(), 0.1, 0.3, &limits, |live, pending| {
                peak = peak.max(live.checked_add(pending).ok_or(Error::Overflow("test admission"))?);
                Ok(())
            }).unwrap();
            assert!(peak >= first.manifest_bytes().unwrap() + first.brep().storage_usage(&limits).unwrap().bytes);
            let second = capture_source_topology(seed.source(), 0.02, 0.3, &limits, |_, _| Ok(())).unwrap();
            let mapping = match_manifests(&first.manifest(), &second.manifest(), &limits).unwrap();
            let topology = mapping.reorder_topology(first.brep().topology(), &second.brep().topology().face_edges, &limits).unwrap();
            assert_eq!(topology.faces, first.brep().topology().faces);
            assert_eq!(topology.edges, first.brep().topology().edges);
            assert_eq!(topology.vertex_ids, first.brep().topology().vertex_ids);
            let mut matching_peak = 0;
            let rebound = first.match_topology(&second, &limits, |live, pending| {
                matching_peak = matching_peak.max(live.checked_add(pending).ok_or(Error::Overflow("test admission"))?);
                Ok(())
            }).unwrap();
            assert_eq!(rebound.faces, first.brep().topology().faces);
            assert_eq!(rebound.edges, first.brep().topology().edges);
            assert_eq!(rebound.vertex_ids, first.brep().topology().vertex_ids);
            assert!(matching_peak > first.manifest_bytes().unwrap() + second.manifest_bytes().unwrap());
            assert!(first.match_topology(&second, &limits, |_, _| Err(Error::Budget("mapping refusal"))).is_err());
            let candidate_points = second.brep().display().points.clone();
            let candidate_triangles = second.brep().display().triangles.clone();
            let rebound_snapshot = second.into_rebound_snapshot(&first, &limits, |_, _| Ok(())).unwrap();
            let rebound = rebound_snapshot.brep();
            assert_eq!(rebound.source(), seed.source());
            assert_eq!(rebound.display().points, candidate_points);
            assert_eq!(rebound.display().triangles, candidate_triangles);
            assert_eq!(rebound.topology().faces, first.brep().topology().faces);
            assert_eq!(rebound.topology().edges, first.brep().topology().edges);
            assert_eq!(rebound.topology().vertex_ids, first.brep().topology().vertex_ids);
            rebound.validate(&limits).unwrap();
            for (index, face) in rebound.topology().faces.iter().enumerate() {
                let triangle = rebound.display().triangles.iter().find(|t| t[3] as usize == index).unwrap();
                let [a, b, c] = [triangle[0], triangle[1], triangle[2]].map(|i| rebound.display().points[i as usize]);
                let u = std::array::from_fn::<_, 3, _>(|i| b[i] - a[i]);
                let v = std::array::from_fn::<_, 3, _>(|i| c[i] - a[i]);
                let normal = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]];
                let length = normal[0].hypot(normal[1]).hypot(normal[2]);
                assert!(length > 0.);
                let direction = normal.map(|value| -value / length);
                let center = std::array::from_fn(|i| (a[i] + b[i] + c[i]) / 3.);
                let origin = std::array::from_fn(|i| center[i] - direction[i]);
                let hit = crate::measurement::brep_face_ray_hit(&rebound, *face, om9_core::geometry::Transform::identity(),
                    origin, direction, center, 1e-6, &limits).unwrap();
                let distance = (0..3).map(|i| (hit[i] - center[i]).powi(2)).sum::<f64>().sqrt();
                assert!(distance <= rebound.display().deflection_mm + 1e-6);
            }
            let third = capture_source_topology(seed.source(), 0.05, 0.3, &limits, |_, _| Ok(())).unwrap();
            let third = third.into_rebound_snapshot(&rebound_snapshot, &limits, |_, _| Ok(())).unwrap();
            assert_eq!(third.brep().topology().faces, first.brep().topology().faces);
            assert_eq!(third.brep().topology().edges, first.brep().topology().edges);
            assert_eq!(third.brep().topology().vertex_ids, first.brep().topology().vertex_ids);
            let mut calls = 0;
            let refused = capture_source_topology(seed.source(), 0.1, 0.3, &limits, |_, _| {
                calls += 1; Err(Error::Budget("test refusal"))
            });
            assert!(refused.is_err());
            assert_eq!(calls, 1);
        }
    }
    #[test]
    fn distinct_sources_cannot_reuse_topology_ids() {
        let limits = Limits::default();
        let a = super::super::box_brep([4., 5., 6.], 0.1, &limits).unwrap();
        let b = super::super::box_brep([4., 5., 7.], 0.1, &limits).unwrap();
        let first = capture_source_topology(a.source(), 0.1, 0.3, &limits, |_, _| Ok(())).unwrap();
        let second = capture_source_topology(b.source(), 0.1, 0.3, &limits, |_, _| Ok(())).unwrap();
        assert!(first.match_topology(&second, &limits, |_, _| Ok(())).is_err());
    }
    #[test]
    fn shared_candidate_and_admission_refusal_leave_original_unchanged() {
        let limits = Limits::default();
        let seed = super::super::box_brep([4., 5., 6.], 0.1, &limits).unwrap();
        let first = capture_source_topology(seed.source(), 0.1, 0.3, &limits, |_, _| Ok(())).unwrap();
        let candidate = capture_source_topology(seed.source(), 0.02, 0.3, &limits, |_, _| Ok(())).unwrap();
        let old_ids = first.brep().topology().faces.clone();
        let shared = candidate.brep().clone();
        assert!(candidate.into_rebound_snapshot(&first, &limits, |_, _| Ok(())).is_err());
        assert_eq!(shared.source(), seed.source());
        assert_eq!(first.brep().topology().faces, old_ids);
        let candidate = capture_source_topology(seed.source(), 0.02, 0.3, &limits, |_, _| Ok(())).unwrap();
        let mut match_calls = 0usize;
        first.match_topology(&candidate, &limits, |_, _| { match_calls += 1; Ok(()) }).unwrap();
        let mut calls = 0usize;
        let result = candidate.into_rebound_snapshot(&first, &limits, |_, _| {
            calls += 1;
            if calls > match_calls { Err(Error::Budget("candidate refusal")) } else { Ok(()) }
        });
        assert!(matches!(result, Err(Error::Budget("candidate refusal"))));
        assert_eq!(first.brep().topology().faces, old_ids);
        first.brep().validate(&limits).unwrap();
    }
    #[test]
    fn document_brep_proof_preserves_document_ids_and_rejects_stale_display() {
        use om9_core::{geometry::Geometry, model::{Document, Entity}};
        let limits = Limits::default();
        let seed = super::super::box_brep([4., 5., 6.], 0.1, &limits).unwrap();
        let snapshot = capture_source_topology(seed.source(), 0.1, 0.3, &limits, |_, _| Ok(())).unwrap();
        let (body, proof) = snapshot.into_parts();
        let weak_display = body.display_liveness();
        let entity = Entity::new("captured body", Geometry::Brep(body), None);
        let id = entity.id;
        let mut document = Document::new("attested document");
        document.entities.insert(id, entity);
        document.validate(&limits).unwrap();
        let Geometry::Brep(original) = &document.entities.get(&id).unwrap().geometry else { panic!("B-rep") };
        proof.verify_brep(original, &limits, |_, _| Ok(())).unwrap();
        assert!(matches!(proof.verify_brep(original, &limits, |_, _| Err(Error::Budget("proof refusal"))),
            Err(Error::Budget("proof refusal"))));
        let candidate = capture_source_topology(seed.source(), 0.02, 0.3, &limits, |_, _| Ok(())).unwrap();
        let rebound = candidate.into_rebound_for_brep(original, &proof, &limits, |_, _| Ok(())).unwrap();
        assert_eq!(rebound.brep().topology().faces, original.topology().faces);
        assert_eq!(rebound.brep().topology().edges, original.topology().edges);
        assert_eq!(rebound.brep().topology().vertex_ids, original.topology().vertex_ids);
        let replacement = super::super::read_brep(seed.source(), 0.1, &limits).unwrap();
        assert!(proof.verify_brep(&replacement, &limits, |_, _| Ok(())).is_err());
        let candidate = capture_source_topology(seed.source(), 0.02, 0.3, &limits, |_, _| Ok(())).unwrap();
        assert!(matches!(candidate.into_rebound_for_brep(&replacement, &proof, &limits, |_, _| Ok(())),
            Err(Error::Stale("topology attestation"))));
        let mut changed = original.clone();
        changed.edit_display_for_validation(|display| display.points[0][0] += 1.);
        assert!(proof.verify_brep(&changed, &limits, |_, _| Ok(())).is_err());
        drop(changed); drop(document);
        assert!(!weak_display.is_alive(), "proof must not retain the display");
    }
    #[test]
    fn registered_proof_is_accounted_and_rejects_changed_transaction_context() {
        use om9_core::{geometry::{Geometry, Transform}, model::{Document, Entity},
            transaction::{Mutation, TransactionRequest, World}};
        let limits = Limits::default();
        let document = Document::new("proof transaction");
        let doc = document.id;
        let mut coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        let mut reservation = coordinator.reserve_external_memory(MemoryClass::CpuCache, 4 * 1024 * 1024, None).unwrap();
        let snapshot = crate::creation::primitive_with_topology(crate::creation::Primitive::Box {
            x_mm: 4., y_mm: 5., z_mm: 6.,
        }, 0.1, &limits, |live, pending| {
            if live.checked_add(pending).ok_or(Error::Overflow("test reservation"))? > reservation.bytes() {
                return Err(Error::Budget("test reservation"));
            }
            reservation.set_allocated_bytes(live)?;
            reservation.recheck_pending()
        }).unwrap();
        let (body, proof) = snapshot.into_parts();
        let proof_bytes = proof.storage_bytes().unwrap();
        let total = proof_bytes + body.storage_usage(&limits).unwrap().bytes;
        reservation.set_allocated_bytes(total).unwrap();
        reservation.shrink_to(total).unwrap();
        let proof_reservation = reservation.split_off(proof_bytes).unwrap();
        let entity = Entity::new("body", Geometry::Brep(body), None);
        let id = entity.id;
        let prepared = coordinator.prepare(TransactionRequest::new("insert", vec![Mutation::Insert { document: doc, entity }])).unwrap();
        coordinator.commit(prepared).unwrap();
        drop(reservation);
        let stamp = coordinator.world().documents[&doc].stamp(id).unwrap();
        let mut scratch = coordinator.reserve_external_memory(MemoryClass::CpuCache, 64 * 1024, None).unwrap();
        let mut scratch_admit = |live: u64, pending: u64| {
            if live.checked_add(pending).ok_or(Error::Overflow("scratch"))? > scratch.bytes() { return Err(Error::Budget("scratch")); }
            scratch.set_allocated_bytes(live)?;
            scratch.recheck_pending()
        };
        let owned = RegisteredTopologyProof::adopt(&coordinator, stamp, proof, proof_reservation, &mut scratch_admit).unwrap();
        owned.verify_current(&coordinator, &mut scratch_admit).unwrap();
        assert_eq!(owned.verify_unchanged_owner(&coordinator).unwrap().display_fingerprint(),
            owned.verify_current(&coordinator, &mut scratch_admit).unwrap().display_fingerprint());
        let document_owner = std::sync::Arc::clone(&coordinator.world().documents[&doc]);
        let weak = std::sync::Arc::downgrade(&document_owner);
        let mut detached = document_owner.clone();
        std::sync::Arc::make_mut(&mut detached).name.push_str(" changed");
        assert!(!weak.ptr_eq(&std::sync::Arc::downgrade(&detached)), "mutation detaches immutable document owner");
        drop(detached);
        drop(document_owner);
        let mut unique = std::sync::Arc::new(Document::new("weak-only owner"));
        let weak_only = std::sync::Arc::downgrade(&unique);
        assert_eq!(std::sync::Arc::strong_count(&unique), 1);
        std::sync::Arc::make_mut(&mut unique).name.push_str(" changed");
        assert!(!weak_only.ptr_eq(&std::sync::Arc::downgrade(&unique)));
        assert!(weak_only.upgrade().is_none(), "weak owner must not survive in-place document mutation");
        let moved = coordinator.prepare(TransactionRequest::new("move", vec![Mutation::Placement { source: stamp,
            placement: Transform::translation([1., 0., 0.]) }])).unwrap();
        coordinator.commit(moved).unwrap();
        assert!(matches!(owned.verify_current(&coordinator, &mut scratch_admit), Err(Error::Stale(_))));
        assert!(matches!(owned.verify_unchanged_owner(&coordinator), Err(Error::Stale(_))));
        coordinator.undo().unwrap();
        assert!(matches!(owned.verify_current(&coordinator, &mut scratch_admit), Err(Error::Stale(_))));
        assert!(matches!(owned.verify_unchanged_owner(&coordinator), Err(Error::Stale(_))));
        drop(owned); drop(scratch_admit); drop(scratch);
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::CpuCache), 0);
    }
    #[test]
    #[ignore = "release-only proof-validation timing probe"]
    fn registered_proof_validation_release_cost() {
        use om9_core::{geometry::Geometry, model::{Document, Entity}, transaction::{Mutation, TransactionRequest, World}};
        let limits = Limits::default();
        let document = Document::new("proof timing");
        let doc = document.id;
        let mut coordinator = TransactionCoordinator::new(World::new(vec![document], &limits).unwrap(), limits).unwrap();
        let mut reservation = coordinator.reserve_external_memory(MemoryClass::CpuCache, 64 * 1024 * 1024, None).unwrap();
        let snapshot = crate::creation::primitive_with_topology(crate::creation::Primitive::Sphere { radius_mm: 3. }, 0.005,
            &limits, |live, pending| {
                if live.checked_add(pending).ok_or(Error::Overflow("timing"))? > reservation.bytes() { return Err(Error::Budget("timing")); }
                reservation.set_allocated_bytes(live)?;
                reservation.recheck_pending()
            }).unwrap();
        let triangles = snapshot.brep().display().triangles.len();
        let (body, proof) = snapshot.into_parts();
        let proof_bytes = proof.storage_bytes().unwrap();
        let total = proof_bytes + body.storage_usage(&limits).unwrap().bytes;
        reservation.set_allocated_bytes(total).unwrap();
        reservation.shrink_to(total).unwrap();
        let proof_reservation = reservation.split_off(proof_bytes).unwrap();
        let entity = Entity::new("sphere", Geometry::Brep(body), None);
        let id = entity.id;
        let prepared = coordinator.prepare(TransactionRequest::new("insert", vec![Mutation::Insert { document: doc, entity }])).unwrap();
        coordinator.commit(prepared).unwrap();
        drop(reservation);
        let stamp = coordinator.world().documents[&doc].stamp(id).unwrap();
        let proof = RegisteredTopologyProof::adopt(&coordinator, stamp, proof, proof_reservation, |_, _| Ok(())).unwrap();
        let mut full = Vec::new();
        let mut owner = Vec::new();
        for _ in 0..31 {
            let start = std::time::Instant::now();
            std::hint::black_box(proof.verify_current(&coordinator, |_, _| Ok(())).unwrap());
            full.push(start.elapsed().as_nanos());
            let start = std::time::Instant::now();
            std::hint::black_box(proof.verify_unchanged_owner(&coordinator).unwrap());
            owner.push(start.elapsed().as_nanos());
        }
        full.sort_unstable();
        owner.sort_unstable();
        println!("proof_check triangles={triangles} samples=31 full_median_ns={} full_p95_ns={} owner_median_ns={} owner_p95_ns={}",
            full[15], full[29], owner[15], owner[29]);
    }
    #[test]
    #[ignore = "cross-process OCCT topology descriptor stability probe"]
    fn topology_descriptor_cross_process_probe() {
        fn signature(snapshot: &CapturedTopologySnapshot) -> [u8; 32] {
            let mut digest = Sha256::new();
            for group in [&snapshot.faces, &snapshot.edges, &snapshot.vertices] {
                digest.update((group.len() as u64).to_le_bytes());
                for item in group { digest.update(item); }
            }
            for group in [&snapshot.face_edges, &snapshot.edge_vertices] {
                digest.update((group.len() as u64).to_le_bytes());
                for item in group {
                    digest.update(item.parent.to_le_bytes());
                    digest.update(item.child.to_le_bytes());
                    digest.update([item.orientation]);
                }
            }
            digest.finalize().into()
        }
        let limits = Limits::default();
        if let Some(source_path) = std::env::var_os("OM9_P1_TOPOLOGY_PROBE_SOURCE") {
            let source = std::fs::read_to_string(source_path).unwrap();
            let snapshot = capture_source_topology(&source, 0.1, 0.3, &limits, |_, _| Ok(())).unwrap();
            let result = std::env::var_os("OM9_P1_TOPOLOGY_PROBE_RESULT").unwrap();
            std::fs::write(result, signature(&snapshot)).unwrap();
            return;
        }
        struct Cleanup(Vec<std::path::PathBuf>);
        impl Drop for Cleanup { fn drop(&mut self) { for path in &self.0 { let _ = std::fs::remove_file(path); } } }
        let nonce = uuid::Uuid::new_v4();
        let paths = Cleanup(vec![std::env::temp_dir().join(format!("om9-topology-source-{nonce}.txt")),
            std::env::temp_dir().join(format!("om9-topology-result-{nonce}.bin"))]);
        for (name, shape) in [("box", crate::creation::Primitive::Box { x_mm: 4., y_mm: 5., z_mm: 6. }),
            ("sphere", crate::creation::Primitive::Sphere { radius_mm: 3. })] {
            let original = crate::creation::primitive(shape, 0.1, &limits).unwrap();
            std::fs::write(&paths.0[0], original.source()).unwrap();
            let expected = capture_source_topology(original.source(), 0.1, 0.3, &limits, |_, _| Ok(())).unwrap();
            let expected_signature = signature(&expected);
            for (kind, descriptors) in [("faces", &expected.faces), ("edges", &expected.edges), ("vertices", &expected.vertices)] {
                let unique = descriptors.iter().collect::<std::collections::BTreeSet<_>>().len();
                assert_eq!(unique, descriptors.len(), "ambiguous {kind} descriptors in {name}");
            }
            for run in 0..3 {
                let output = std::process::Command::new(std::env::current_exe().unwrap())
                    .args(["--ignored", "--exact", "brep::topology_capture::tests::topology_descriptor_cross_process_probe"])
                    .env("OM9_P1_TOPOLOGY_PROBE_SOURCE", &paths.0[0]).env("OM9_P1_TOPOLOGY_PROBE_RESULT", &paths.0[1])
                    .output().unwrap();
                assert!(output.status.success(), "child {name}/{run}: {}", String::from_utf8_lossy(&output.stderr));
                let actual = std::fs::read(&paths.0[1]).unwrap();
                assert_eq!(actual.as_slice(), expected_signature, "descriptor mismatch for {name} in child {run}");
            }
            println!("cross_process {name} children=3 faces={} edges={} vertices={}", expected.faces.len(), expected.edges.len(), expected.vertices.len());
        }
    }
    #[test]
    fn primitive_proof_uses_the_same_single_snapshot_as_normal_creation() {
        use crate::creation::{primitive, primitive_with_topology, Primitive};
        let limits = Limits::default();
        for shape in [Primitive::Box { x_mm: 4., y_mm: 5., z_mm: 6. },
            Primitive::Sphere { radius_mm: 3. }, Primitive::Ellipsoid { x_mm: 3., y_mm: 4., z_mm: 5. }] {
            let normal = primitive(shape, 0.1, &limits).unwrap();
            let captured = primitive_with_topology(shape, 0.1, &limits, |_, _| Ok(())).unwrap();
            assert_eq!(normal.source(), captured.brep().source());
            assert_eq!(normal.display().points, captured.brep().display().points);
            assert_eq!(normal.display().triangles, captured.brep().display().triangles);
            let (body, proof) = captured.into_parts();
            proof.verify_brep(&body, &limits, |_, _| Ok(())).unwrap();
        }
    }
    #[test]
    #[ignore = "isolated release timing probe; not a desktop benchmark"]
    fn primitive_topology_capture_release_cost() {
        use crate::creation::{primitive, primitive_with_topology, Primitive};
        let limits = Limits::default();
        for (name, shape) in [("box", Primitive::Box { x_mm: 10., y_mm: 10., z_mm: 10. }),
            ("sphere", Primitive::Sphere { radius_mm: 5. })] {
            let _ = primitive(shape, 0.1, &limits).unwrap();
            let _ = primitive_with_topology(shape, 0.1, &limits, |_, _| Ok(())).unwrap();
            let mut plain = Vec::new();
            let mut attested = Vec::new();
            for _ in 0..10 {
                let start = std::time::Instant::now();
                std::hint::black_box(primitive(shape, 0.1, &limits).unwrap());
                plain.push(start.elapsed().as_micros());
                let start = std::time::Instant::now();
                std::hint::black_box(primitive_with_topology(shape, 0.1, &limits, |_, _| Ok(())).unwrap());
                attested.push(start.elapsed().as_micros());
            }
            plain.sort_unstable(); attested.sort_unstable();
            println!("{name} samples=10 plain_median_us={} plain_max_us={} proof_median_us={} proof_max_us={}",
                (plain[4] + plain[5]) / 2, plain[9], (attested[4] + attested[5]) / 2, attested[9]);
        }
    }
}
