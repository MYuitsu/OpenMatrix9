// SPDX-License-Identifier: LGPL-2.1-or-later
//! Owned OCCT source plus a revision-bound, disposable display/topology snapshot.
//! Native indices never serve as persistent IDs. Replacing this snapshot creates new UUIDs;
//! publication/source stamps invalidate previous picks instead of guessing topology continuity.
use crate::{
    Error, Result,
    budget::{Limits, Usage},
    geometry::{Point3, finite},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::{Arc, Weak};
use uuid::Uuid;

/// An external account kept alive for every owner of an admitted candidate.
#[derive(Clone)]
struct AllocationOwner { _owner: Arc<dyn std::any::Any + Send + Sync> }
impl std::fmt::Debug for AllocationOwner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AllocationOwner(..)")
    }
}

pub mod topology_mapping;

/// Process-local canonical owner identity. It is not a native topology proof
/// and is never persisted; decoding a source creates a fresh identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BrepSourceIdentity(Uuid);

/// Document geometry. Derived viewport data belongs to a representation owner.
#[derive(Debug, Clone)]
pub struct CanonicalBrep(BrepSource, Option<Arc<CoupledDisplayDiagnostic>>);

// Opt-in benchmark reconstruction of the former document-owned source+display
// pair. It retains the SAME admitted mesh, never generates work per frame.
// No Weak source receipt here: exclusive source handoff still has one receipt.
#[derive(Debug)]
struct CoupledDisplayDiagnostic {
    pin: BrepDisplayPin,
    fingerprint: [u8; 32],
    identity: Uuid,
    validations: std::sync::atomic::AtomicU64,
}
impl PartialEq for CanonicalBrep {
    fn eq(&self, other: &Self) -> bool { self.0 == other.0 }
}

impl Serialize for CanonicalBrep {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        // The benchmark control also restores the former complete-pair
        // serialization/fingerprinting boundary. It is throwaway-only, not an
        // additional supported project format or a persistence migration.
        if let Some(display) = &self.1 { return self.coupled_pair(display).serialize(serializer); }
        self.0.canonical_record().serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for CanonicalBrep {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let record = CanonicalBrepRecord::deserialize(deserializer)?;
        Ok(Self(BrepSource { source: Arc::new(record.source), topology: Arc::new(record.topology),
            source_sha256: record.source_sha256, binding_sha256: record.binding_sha256,
            allocation_owner: None, topology_allocation_owner: None, identity: BrepSourceIdentity(Uuid::new_v4()) }, None))
    }
}
impl CanonicalBrep {
    pub fn source(&self) -> &str { self.0.source() }
    pub fn topology(&self) -> &BrepTopology { self.0.topology() }
    pub fn identity(&self) -> BrepSourceIdentity { self.0.identity }
    pub fn canonical_record(&self) -> CanonicalBrepRef<'_> { self.0.canonical_record() }
    pub fn canonical_fingerprints(&self) -> ([u8; 32], [u8; 32]) { (self.0.source_sha256, self.0.binding_sha256) }
    pub fn validate(&self, limits: &Limits) -> Result<Usage> { self.validate_admitted(limits, &mut |_, _| Ok(())) }
    pub fn validate_source(&self, limits: &Limits) -> Result<Usage> { self.0.validate_admitted(limits, &mut |_, _| Ok(())) }
    pub fn validate_admitted(&self, limits: &Limits, admit: &mut dyn FnMut(u64, u64) -> Result<()>) -> Result<Usage> {
        if let Some(display) = &self.1 {
            display.validations.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let pair = self.coupled_pair(display);
            // Restore combined source/display integrity validation at the
            // document boundary. Keep the modern memory policy in both arms:
            // the retained display lease already accounts for its allocation.
            let full = pair.storage_usage(limits)?.bytes;
            let canonical = self.storage_usage(limits)?;
            pair.validate_admitted(limits, &mut |live, pending| {
                let scratch = live.checked_sub(full).ok_or(Error::Invalid("coupled validation accounting"))?;
                admit(canonical.bytes.checked_add(scratch).ok_or(Error::Overflow("coupled validation accounting"))?, pending)
            })?;
            return Ok(canonical);
        }
        self.0.validate_admitted(limits, admit)
    }
    fn coupled_pair(&self, display: &CoupledDisplayDiagnostic) -> Brep {
        Brep { canonical: self.0.clone(), display: display.pin.display.clone(),
            display_allocation_owner: display.pin._candidate_owner.clone(),
            display_sha256: display.fingerprint, display_identity: display.identity }
    }
    #[doc(hidden)]
    pub fn has_coupled_display_diagnostic(&self) -> bool { self.1.is_some() }
    #[doc(hidden)]
    pub fn coupled_diagnostic_validation_count(&self) -> u64 {
        self.1.as_ref().map_or(0, |d|d.validations.load(std::sync::atomic::Ordering::Relaxed))
    }
    pub(crate) fn coupled_diagnostic_bytes() -> u64 {
        (std::mem::size_of::<CoupledDisplayDiagnostic>() + 2 * std::mem::size_of::<usize>()) as u64
    }
    pub(crate) fn attach_coupled_diagnostic(&mut self, representation: &BrepRepresentation) -> Result<()> {
        if self.1.is_some() || !representation.matches(self) || representation.display_allocation_owner.is_none() {
            return Err(Error::Invalid("coupled diagnostic source/display"));
        }
        let owner = representation.display_allocation_owner.as_ref().expect("checked display owner")._owner.clone();
        self.1 = Some(Arc::new(CoupledDisplayDiagnostic { pin: representation.pin(owner),
            fingerprint: representation.display_sha256, identity: representation.display_identity,
            validations: std::sync::atomic::AtomicU64::new(0) }));
        Ok(())
    }
    pub fn validate_source_admitted(&self, limits: &Limits, mut admit: impl FnMut(u64, u64) -> Result<()>) -> Result<Usage> {
        self.0.validate_admitted(limits, &mut admit)
    }
    pub fn source_storage_bytes(&self) -> Result<u64> {
        self.0.source_bytes()?.checked_add(self.0.topology_bytes()?).ok_or(Error::Overflow("canonical BRep storage"))
    }
    pub fn storage_usage(&self, limits: &Limits) -> Result<Usage> {
        let nodes = self.topology().faces.len().checked_add(self.topology().edges.len())
            .and_then(|n| n.checked_add(self.topology().vertex_ids.len())).ok_or(Error::Overflow("BRep nodes"))?;
        let usage = Usage { bytes: self.source_storage_bytes()?, nodes: u64::try_from(nodes).map_err(|_| Error::Overflow("BRep nodes"))?, ..Usage::default() };
        usage.check(limits)?; Ok(usage)
    }
    pub fn shares_validation_payload(&self, other: &Self) -> bool {
        self.0.identity == other.0.identity && self.canonical_fingerprints() == other.canonical_fingerprints()
            && Arc::ptr_eq(&self.0.source, &other.0.source) && Arc::ptr_eq(&self.0.topology, &other.0.topology)
            && match (&self.1, &other.1) { (None,None) => true, (Some(a),Some(b)) => Arc::ptr_eq(a,b), _ => false }
    }
    pub(crate) fn tracked_source_snapshot(&self) -> BrepSource { self.0.clone() }
    pub(crate) fn source_allocation_key(&self) -> usize { self.0.source_key() }
    pub(crate) fn topology_allocation_key(&self) -> usize { self.0.topology_key() }
    pub fn source_allocation_bytes(&self) -> Result<u64> { self.0.source_bytes() }
    pub fn topology_allocation_bytes(&self) -> Result<u64> { self.0.topology_bytes() }
    pub fn has_allocation_owner(&self) -> bool { self.0.allocation_owner.is_some() || self.0.topology_allocation_owner.is_some() }
    pub(crate) fn clone_for_document_staging(&self) -> Self {
        let mut source = self.0.clone(); source.allocation_owner = None; source.topology_allocation_owner = None;
        Self(source, self.1.clone())
    }
    pub fn has_exclusive_storage(&self) -> bool { Arc::strong_count(&self.0.source) == 1 && Arc::weak_count(&self.0.source) == 0 }
    pub(crate) fn has_exclusive_storage_with_one_receipt(&self) -> bool {
        Arc::strong_count(&self.0.source) == 1 && Arc::weak_count(&self.0.source) == 1
    }
    pub fn allocation_covered_by(&self, account: &crate::transaction::ExternalMemoryAccount, bytes: u64) -> bool {
        let mut covered = 0u64;
        for lease in [self.0.allocation_owner.as_ref(), self.0.topology_allocation_owner.as_ref()].into_iter().flatten() {
            let Some(owner) = lease._owner.downcast_ref::<std::sync::Mutex<crate::transaction::ExternalMemoryReservation>>() else { return false; };
            let guard = owner.lock().unwrap_or_else(|e| e.into_inner());
            if !guard.belongs_to(account) { return false; }
            covered = covered.saturating_add(guard.bytes().saturating_sub(guard.pending_bytes()));
        }
        covered >= bytes
    }
}

/// Exact-source input shared by canonical document shapes and transient native results.
/// Geometry algorithms receive no viewport representation through this interface.
pub trait BrepSourceAccess {
    fn source(&self) -> &str;
    fn to_canonical(&self) -> CanonicalBrep;
    fn topology(&self) -> &BrepTopology;
    fn canonical_fingerprints(&self) -> ([u8;32],[u8;32]);
    fn source_storage_bytes(&self) -> Result<u64>;
    fn canonical_owner_keys(&self) -> (usize, usize);
    fn shares_validation_payload(&self, other:&dyn BrepSourceAccess)->bool {
        self.canonical_owner_keys()==other.canonical_owner_keys() && self.canonical_fingerprints()==other.canonical_fingerprints()
    }
    fn validate_source_admitted(&self, limits:&Limits, admit:&mut dyn FnMut(u64,u64)->Result<()>) -> Result<Usage>;
    fn validate_source(&self, limits:&Limits) -> Result<Usage> {
        self.validate_source_admitted(limits,&mut |_,_|Ok(()))
    }
    fn validate(&self, limits:&Limits) -> Result<Usage> { self.validate_source(limits) }
    fn storage_usage(&self, limits:&Limits) -> Result<Usage> {
        let topology=self.topology();
        let nodes=topology.faces.len().checked_add(topology.edges.len()).and_then(|n|n.checked_add(topology.vertex_ids.len()))
            .ok_or(Error::Overflow("BRep source nodes"))?;
        let usage=Usage {bytes:self.source_storage_bytes()?,nodes:nodes as u64,..Usage::default()};
        usage.check(limits)?;Ok(usage)
    }
}
impl BrepSourceAccess for CanonicalBrep {
    fn to_canonical(&self)->CanonicalBrep {self.clone()}
    fn source(&self)->&str { self.source() }
    fn topology(&self)->&BrepTopology { self.topology() }
    fn canonical_fingerprints(&self)->([u8;32],[u8;32]) { self.canonical_fingerprints() }
    fn source_storage_bytes(&self)->Result<u64> { self.source_storage_bytes() }
    fn canonical_owner_keys(&self)->(usize,usize) { (self.source_allocation_key(),self.topology_allocation_key()) }
    fn validate_source_admitted(&self,limits:&Limits,admit:&mut dyn FnMut(u64,u64)->Result<()>) -> Result<Usage> {
        self.0.validate_admitted(limits,admit)
    }
}
impl BrepSourceAccess for Brep {
    fn to_canonical(&self)->CanonicalBrep {CanonicalBrep(self.canonical.clone(), None)}
    fn source(&self)->&str { self.source() }
    fn topology(&self)->&BrepTopology { self.topology() }
    fn canonical_fingerprints(&self)->([u8;32],[u8;32]) { self.canonical_fingerprints() }
    fn source_storage_bytes(&self)->Result<u64> { self.source_storage_bytes() }
    fn canonical_owner_keys(&self)->(usize,usize) { (self.source_allocation_key(),self.topology_allocation_key()) }
    fn validate_source_admitted(&self,limits:&Limits,admit:&mut dyn FnMut(u64,u64)->Result<()>) -> Result<Usage> {
        self.canonical.validate_admitted(limits,admit)
    }
}

impl<T: BrepSourceAccess + ?Sized> BrepSourceAccess for &T {
    fn source(&self)->&str { (**self).source() }
    fn to_canonical(&self)->CanonicalBrep { (**self).to_canonical() }
    fn topology(&self)->&BrepTopology { (**self).topology() }
    fn canonical_fingerprints(&self)->([u8;32],[u8;32]) { (**self).canonical_fingerprints() }
    fn source_storage_bytes(&self)->Result<u64> { (**self).source_storage_bytes() }
    fn canonical_owner_keys(&self)->(usize,usize) { (**self).canonical_owner_keys() }
    fn validate_source_admitted(&self,limits:&Limits,admit:&mut dyn FnMut(u64,u64)->Result<()>) -> Result<Usage> {
        (**self).validate_source_admitted(limits,admit)
    }
}

/// Runtime cache profile. An unspecified adapter profile is never treated as
/// equal to another display, even if its linear deflection happens to match.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BrepRepresentationProfile {
    Unspecified { linear: u64, display: Uuid },
    Certified { linear: u64, angular: u64, adapter_version: u32 },
}

/// Prepared display transferred independently from canonical document data.
#[derive(Debug, Clone)]
pub struct BrepRepresentation {
    source: BrepSourceIdentity,
    source_liveness: Weak<String>,
    display: Arc<BrepDisplay>,
    display_allocation_owner: Option<AllocationOwner>,
    source_allocation_owner: Option<AllocationOwner>,
    topology_allocation_owner: Option<AllocationOwner>,
    display_sha256: [u8; 32],
    display_identity: Uuid,
    profile: BrepRepresentationProfile,
}
impl BrepRepresentation {
    /// Arc allocation retained by the weak source receipt after the source
    /// value dies. Cache owners must include it in their metadata charge.
    pub fn source_receipt_bytes() -> u64 {
        (std::mem::size_of::<String>() + 2 * std::mem::size_of::<usize>()) as u64
    }
    pub fn source_identity(&self) -> BrepSourceIdentity { self.source }
    pub fn display(&self) -> &BrepDisplay { &self.display }
    pub fn display_identity(&self) -> BrepDisplayIdentity { BrepDisplayIdentity(self.display_identity) }
    pub fn display_fingerprint(&self) -> [u8; 32] { self.display_sha256 }
    pub fn display_allocation_bytes(&self) -> Result<u64> { self.display.allocation_bytes() }
    pub(crate) fn display_owned_by(&self, account: &crate::transaction::ExternalMemoryAccount) -> bool {
        let Some(owner) = self.display_allocation_owner.as_ref().and_then(|owner|
            owner._owner.downcast_ref::<std::sync::Mutex<crate::transaction::ExternalMemoryReservation>>()) else { return false; };
        let guard = owner.lock().unwrap_or_else(|error| error.into_inner());
        guard.belongs_to(account) && self.display_allocation_bytes().is_ok_and(|bytes|
            guard.bytes().saturating_sub(guard.pending_bytes()) >= bytes)
    }
    pub fn profile(&self) -> BrepRepresentationProfile { self.profile }
    /// Call only when the adapter knows the settings used for this exact
    /// display; this records a cache key and is not topology attestation.
    pub fn set_certified_profile(&mut self, angular: f64, adapter_version: u32) -> Result<()> {
        if !angular.is_finite() || angular <= 0. || angular > std::f64::consts::PI || adapter_version == 0 {
            return Err(Error::Invalid("BRep representation profile"));
        }
        self.profile = BrepRepresentationProfile::Certified { linear: self.display.deflection_mm.to_bits(),
            angular: angular.to_bits(), adapter_version };
        Ok(())
    }
    pub fn matches(&self, source: &CanonicalBrep) -> bool {
        self.source == source.identity() && std::ptr::eq(self.source_liveness.as_ptr(), Arc::as_ptr(&source.0.source))
    }
    fn same_registration(&self, other: &Self) -> bool {
        fn same_owner(a: &Option<AllocationOwner>, b: &Option<AllocationOwner>) -> bool {
            match (a, b) {
                (None, None) => true,
                (Some(a), Some(b)) => Arc::ptr_eq(&a._owner, &b._owner),
                _ => false,
            }
        }
        self.source == other.source && self.source_liveness.ptr_eq(&other.source_liveness)
            && self.profile == other.profile && self.display_identity == other.display_identity
            && self.display_sha256 == other.display_sha256 && Arc::ptr_eq(&self.display, &other.display)
            && same_owner(&self.display_allocation_owner, &other.display_allocation_owner)
            && same_owner(&self.source_allocation_owner, &other.source_allocation_owner)
            && same_owner(&self.topology_allocation_owner, &other.topology_allocation_owner)
    }
    pub fn materialize(&self, source: &CanonicalBrep, limits: &Limits) -> Result<Brep> {
        Brep::from_canonical_and_representation(source, self, limits)
    }
    pub fn materialize_admitted(&self, source: &CanonicalBrep, limits: &Limits,
        admit: &mut dyn FnMut(u64, u64) -> Result<()>) -> Result<Brep> {
        Brep::from_canonical_and_representation_admitted(source, self, limits, admit)
    }
    /// Adopt a charged display allocation before publishing a rebuilt entry.
    /// Source and topology leases remain independent of this display owner.
    pub fn bind_display_owner(&mut self, source: &CanonicalBrep,
        owner: Arc<std::sync::Mutex<crate::transaction::ExternalMemoryReservation>>,
        account: &crate::transaction::ExternalMemoryAccount,
        limits: &Limits) -> Result<()> {
        if self.display_allocation_owner.is_some() || !self.is_unpinned() || !self.matches(source) {
            return Err(Error::Invalid("BRep representation owner"));
        }
        let bytes = self.display.allocation_bytes()?;
        Usage { bytes, ..Usage::default() }.check(limits)?;
        {
            let guard = owner.lock().unwrap_or_else(|error| error.into_inner());
            if !guard.belongs_to(account) || guard.bytes().saturating_sub(guard.pending_bytes()) < bytes {
                return Err(Error::Invalid("BRep representation owner accounting"));
            }
        }
        self.display_allocation_owner = Some(AllocationOwner { _owner: owner });
        Ok(())
    }
    pub fn is_unpinned(&self) -> bool { Arc::strong_count(&self.display) == 1 }
    fn exclusively_owned_display_bytes(&self, account: &crate::transaction::ExternalMemoryAccount) -> Option<u64> {
        if !self.is_unpinned() { return None; }
        let owner = self.display_allocation_owner.as_ref()?;
        if Arc::strong_count(&owner._owner) != 1 { return None; }
        let guard = owner._owner.downcast_ref::<std::sync::Mutex<crate::transaction::ExternalMemoryReservation>>()?
            .lock().unwrap_or_else(|error| error.into_inner());
        guard.belongs_to(account).then_some(guard.bytes())
    }
    pub fn pin(&self, owner: Arc<dyn std::any::Any + Send + Sync>) -> BrepDisplayPin {
        BrepDisplayPin { display: Arc::clone(&self.display), _owner: owner,
            _candidate_owner: self.display_allocation_owner.clone(), _source_owner: self.source_allocation_owner.clone(),
            _topology_owner: self.topology_allocation_owner.clone() }
    }
}

/// Runtime-only representations, owned separately from document snapshots.
#[derive(Debug, Default)]
pub struct BrepRepresentationStore {
    entries: std::collections::BTreeMap<(BrepSourceIdentity, BrepRepresentationProfile), BrepRepresentation>,
    preferred: std::collections::BTreeMap<BrepSourceIdentity, BrepRepresentationProfile>,
    native: crate::geometry::NativeBrepRepresentationStore,
    coupled_diagnostic: bool,
}
impl BrepRepresentationStore {
    /// Benchmark-only: choose before registering any geometry. Product startup
    /// never enables this; it is deliberately not persisted.
    #[doc(hidden)]
    pub fn enable_coupled_diagnostic(&mut self) -> Result<()> {
        if !self.entries.is_empty() || !self.preferred.is_empty() {
            return Err(Error::Invalid("coupled diagnostic requires empty store"));
        }
        self.coupled_diagnostic = true;
        Ok(())
    }
    pub(crate) fn coupled_diagnostic_enabled(&self) -> bool { self.coupled_diagnostic }
    /// The coupled control renders from its document-owned mesh; the product
    /// resolves an independently owned representation. Both reuse GPU buffers.
    pub fn display_for<'a>(&'a self, source: &'a CanonicalBrep) -> Result<&'a BrepDisplay> {
        if let Some(attached) = &source.1 { return Ok(attached.pin.display()); }
        Ok(self.get(source)?.display())
    }
    pub fn pin_for(&self, source: &CanonicalBrep, owner: Arc<dyn std::any::Any + Send + Sync>) -> Result<BrepDisplayPin> {
        if let Some(attached) = &source.1 {
            let mut pin = attached.pin.clone(); pin._owner = owner; return Ok(pin);
        }
        Ok(self.get(source)?.pin(owner))
    }
    fn entries_for_source(&self, source: BrepSourceIdentity)
        -> impl Iterator<Item = (&(BrepSourceIdentity, BrepRepresentationProfile), &BrepRepresentation)> {
        // Unspecified is the first profile variant; zero/nil is its minimum.
        let first = BrepRepresentationProfile::Unspecified { linear: 0, display: Uuid::nil() };
        self.entries.range((source, first)..).take_while(move |((id, _), _)| *id == source)
    }
    pub(crate) fn is_empty(&self) -> bool { self.entries.is_empty() && self.native.is_empty() }
    pub(crate) fn entry_count(&self) -> usize { self.entries.len().saturating_add(self.native.len()) }
    /// Opaque native payloads retain their original representation until an
    /// adapter proves that they can be reconstructed from source.
    pub fn insert_native(&mut self, source: &crate::geometry::NativeBrepSource,
        representation: crate::geometry::NativeBrepRepresentation, limits: &Limits) -> Result<()> {
        self.native.insert(source, representation, limits)
    }
    pub(crate) fn insert_native_prevalidated(&mut self, source: &crate::geometry::NativeBrepSource,
        representation: crate::geometry::NativeBrepRepresentation) -> Result<()> {
        self.native.insert_prevalidated(source, representation)
    }
    pub fn get_native(&self, source: &crate::geometry::NativeBrepSource)
        -> Result<&crate::geometry::NativeBrepRepresentation> {
        self.native.get(source)
    }
    pub(crate) fn contains_native(&self, source: crate::geometry::NativeBrepSourceIdentity) -> bool {
        self.native.contains(source)
    }
    pub(crate) fn remove_native_exact(&mut self, source: crate::geometry::NativeBrepSourceIdentity,
        display: crate::geometry::NativeBrepRepresentationIdentity) -> Option<crate::geometry::NativeBrepRepresentation> {
        self.native.remove_exact(source, display)
    }
    pub(crate) fn take_orphaned_native(&mut self) -> Option<crate::geometry::NativeBrepRepresentation> {
        self.native.take_orphaned()
    }
    pub(crate) fn take_orphaned(&mut self) -> Option<BrepRepresentation> {
        let (source, profile, display) = self.entries.iter().find_map(|((source, profile), value)|
            (value.source_liveness.strong_count() == 0)
                .then_some((*source, *profile, value.display_identity())))?;
        self.remove_exact(source, profile, display)
    }
    #[doc(hidden)]
    pub fn has_rebuildable_unpinned(&self, source: BrepSourceIdentity,
        account: &crate::transaction::ExternalMemoryAccount) -> bool {
        self.rebuildable_unpinned(source, account).is_some()
    }
    fn rebuildable_unpinned(&self, source: BrepSourceIdentity,
        account: &crate::transaction::ExternalMemoryAccount)
        -> Option<(BrepRepresentationProfile, BrepDisplayIdentity, u64)> {
        let preferred = self.preferred.get(&source).copied();
        let multiple = self.entries_for_source(source).take(2).count() > 1;
        self.entries_for_source(source).find_map(|((_, profile), value)| {
            // Keeping the preferred representation until it is the only one
            // avoids silently switching history to a different display profile.
            if (multiple && Some(*profile) == preferred) || value.source_liveness.strong_count() == 0 { return None; }
            value.exclusively_owned_display_bytes(account).map(|bytes| (*profile, value.display_identity(), bytes))
        })
    }
    /// The caller must retain independently accounted, source-bound rebuild
    /// evidence before using this. The returned owner must be dropped after
    /// releasing the store lock; live scene/worker pins prevent removal.
    #[doc(hidden)]
    pub fn take_rebuildable_unpinned(&mut self, source: BrepSourceIdentity,
        account: &crate::transaction::ExternalMemoryAccount) -> Option<(BrepRepresentation, u64)> {
        let (profile, display, bytes) = self.rebuildable_unpinned(source, account)?;
        Some((self.remove_exact(source, profile, display)?, bytes))
    }
    pub(crate) fn remove_exact(&mut self, source: BrepSourceIdentity, profile: BrepRepresentationProfile,
        display: BrepDisplayIdentity) -> Option<BrepRepresentation> {
        let key = (source, profile);
        if self.entries.get(&key).is_none_or(|entry| entry.display_identity() != display) { return None; }
        let removed = self.entries.remove(&key);
        if self.preferred.get(&source) == Some(&profile) {
            let replacement = self.entries_for_source(source).next().map(|((_, profile), _)| *profile);
            if let Some(replacement) = replacement {
                *self.preferred.get_mut(&source).expect("preferred entry") = replacement;
            } else {
                self.preferred.remove(&source);
            }
        }
        removed
    }
    pub(crate) fn contains_profile(&self, source: BrepSourceIdentity, profile: BrepRepresentationProfile) -> bool {
        self.entries.contains_key(&(source, profile))
    }
    #[doc(hidden)]
    pub fn preferred_profile(&self, source: BrepSourceIdentity) -> Option<BrepRepresentationProfile> {
        self.preferred.get(&source).copied()
    }
    pub(crate) fn existing_registration(&mut self, source: &CanonicalBrep, representation: &BrepRepresentation) -> Option<bool> {
        let key = (source.identity(), representation.profile());
        let same = self.entries.get(&key)?.same_registration(representation);
        if same { self.preferred.insert(source.identity(), representation.profile()); }
        Some(same)
    }
    pub(crate) fn has_single_accounted_receipt(&self, source: &CanonicalBrep,
        account: &crate::transaction::ExternalMemoryAccount) -> bool {
        let mut entries = self.entries_for_source(source.identity());
        let Some((_, representation)) = entries.next() else { return false; };
        if entries.next().is_some() || !representation.matches(source)
            || !representation.source_liveness.ptr_eq(&Arc::downgrade(&source.0.source)) { return false; }
        let Some(owner) = representation.display_allocation_owner.as_ref().and_then(|value|
            value._owner.downcast_ref::<std::sync::Mutex<crate::transaction::ExternalMemoryReservation>>()) else { return false; };
        let guard = owner.lock().unwrap_or_else(|error| error.into_inner());
        let Some(required) = representation.display_allocation_bytes().ok()
            .and_then(|bytes| bytes.checked_add(BrepRepresentation::source_receipt_bytes()))
            .and_then(|bytes| bytes.checked_add((std::mem::size_of::<std::sync::Mutex<crate::transaction::ExternalMemoryReservation>>() + 2 * std::mem::size_of::<usize>()) as u64))
            .and_then(|bytes| bytes.checked_add(Usage::btree::<(BrepSourceIdentity, BrepRepresentationProfile), BrepRepresentation>(1).ok()?.bytes))
            .and_then(|bytes| bytes.checked_add(Usage::btree::<BrepSourceIdentity, BrepRepresentationProfile>(1).ok()?.bytes)) else { return false; };
        guard.belongs_to(account) && guard.bytes().saturating_sub(guard.pending_bytes()) >= required
    }
    pub fn insert(&mut self, source: &CanonicalBrep, representation: BrepRepresentation, limits: &Limits) -> Result<()> {
        representation.materialize(source, limits)?;
        self.insert_prevalidated(source, representation)
    }
    /// The caller has just validated this exact immutable source/display pair.
    pub(crate) fn insert_prevalidated(&mut self, source: &CanonicalBrep, representation: BrepRepresentation) -> Result<()> {
        if !representation.matches(source) { return Err(Error::Stale("BRep representation source")); }
        let profile = representation.profile();
        if self.entries.contains_key(&(source.identity(), profile)) {
            return Err(Error::Invalid("duplicate BRep representation profile"));
        }
        self.entries.insert((source.identity(), profile), representation);
        self.preferred.insert(source.identity(), profile);
        Ok(())
    }
    /// A staged transaction must not change the display selected by existing
    /// readers of a live source. A source with no entry needs its staged
    /// representation for private scene construction; rollback removes it.
    pub(crate) fn insert_pending_prevalidated(&mut self, source: &CanonicalBrep, representation: BrepRepresentation) -> Result<()> {
        if !representation.matches(source) { return Err(Error::Stale("BRep representation source")); }
        let profile = representation.profile();
        if self.entries.contains_key(&(source.identity(), profile)) {
            return Err(Error::Invalid("duplicate BRep representation profile"));
        }
        self.entries.insert((source.identity(), profile), representation);
        self.preferred.entry(source.identity()).or_insert(profile);
        Ok(())
    }
    pub fn get(&self, source: &CanonicalBrep) -> Result<&BrepRepresentation> {
        let profile = self.preferred.get(&source.identity()).ok_or(Error::Missing("BRep representation requires rebuild"))?;
        self.entries.get(&(source.identity(), *profile)).filter(|representation| representation.matches(source))
            .ok_or(Error::Missing("BRep representation requires rebuild"))
    }
    pub fn get_profile(&self, source: &CanonicalBrep, deflection: f64, angular: f64,
        adapter_version: u32) -> Result<&BrepRepresentation> {
        let profile = BrepRepresentationProfile::Certified { linear: deflection.to_bits(), angular: angular.to_bits(), adapter_version };
        self.entries.get(&(source.identity(), profile)).filter(|representation| representation.matches(source))
            .ok_or(Error::Missing("BRep profile requires rebuild"))
    }
}

#[derive(Debug, Clone)]
pub struct Brep {
    // Drop the display before the canonical owner releases any candidate lease.
    display: Arc<BrepDisplay>,
    display_allocation_owner: Option<AllocationOwner>,
    canonical: BrepSource,
    display_sha256: [u8; 32],
    display_identity: Uuid,
}
impl PartialEq for Brep {
    fn eq(&self, other: &Self) -> bool {
        // The process-local receipt identifies an allocation, not document data.
        self.canonical == other.canonical && self.display_sha256 == other.display_sha256 && self.display == other.display
    }
}

/// A display retained together with the owner that accounts for its lifetime.
/// Clones keep both allocations alive and never expose an independently
/// clonable display Arc.
#[derive(Clone)]
pub struct BrepDisplayPin {
    display: Arc<BrepDisplay>,
    _owner: Arc<dyn std::any::Any + Send + Sync>,
    _candidate_owner: Option<AllocationOwner>,
    _source_owner: Option<AllocationOwner>,
    _topology_owner: Option<AllocationOwner>,
}
impl std::fmt::Debug for BrepDisplayPin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BrepDisplayPin").finish_non_exhaustive()
    }
}
impl BrepDisplayPin {
    pub fn display(&self) -> &BrepDisplay { &self.display }
    #[doc(hidden)]
    pub fn owner<T: std::any::Any>(&self) -> Option<&T> { self._owner.downcast_ref::<T>() }
    #[doc(hidden)]
    pub fn topology_is_exclusive(&self) -> bool {
        Arc::strong_count(&self.display.topology.0) == 1 && Arc::weak_count(&self.display.topology.0) == 0
    }
    /// True only when no scene, renderer, worker, or other pin retains this
    /// display. Callers must synchronize creation of new pins when testing it.
    #[doc(hidden)]
    pub fn is_exclusive(&self) -> bool {
        Arc::strong_count(&self.display) == 1 && Arc::weak_count(&self.display) == 0
            && Arc::strong_count(&self._owner) == 1
            && self.topology_is_exclusive()
    }
    #[doc(hidden)]
    pub fn storage_bytes(&self) -> Result<u64> {
        let d = &self.display;
        let t = &d.topology;
        [Usage::array::<BrepDisplay>(1)?.bytes, Usage::array::<usize>(2)?.bytes,
            Usage::array::<Point3>(d.points.capacity())?.bytes,
            Usage::array::<[u32; 4]>(d.triangles.capacity())?.bytes,
            Usage::array::<[u32; 3]>(d.segments.capacity())?.bytes,
            Usage::array::<u32>(d.vertices.capacity())?.bytes,
            Usage::array::<BrepTopology>(1)?.bytes, Usage::array::<usize>(2)?.bytes,
            Usage::array::<Uuid>(t.faces.capacity())?.bytes, Usage::array::<Uuid>(t.edges.capacity())?.bytes,
            Usage::array::<Uuid>(t.vertex_ids.capacity())?.bytes, Usage::array::<[u32; 2]>(t.face_edges.capacity())?.bytes]
            .into_iter().try_fold(0u64, |sum, bytes| sum.checked_add(bytes).ok_or(Error::Overflow("BRep display storage")))
    }
}

/// Process-local identity of one immutable display version. Clones preserve
/// it; replacement or editing creates a new identity. It does not retain an
/// Arc header and is not serialized into the document.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BrepDisplayIdentity(Uuid);
impl BrepDisplayIdentity {
    pub fn same(&self, other: &Self) -> bool { self == other }
}
/// Diagnostic liveness witness; its weak header must be accounted for if it
/// outlives the display. Production identity receipts do not use this type.
#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct BrepDisplayLiveness(Weak<BrepDisplay>);
impl BrepDisplayLiveness {
    pub fn is_alive(&self) -> bool { self.0.strong_count() != 0 }
}
impl Serialize for Brep {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut stored = serializer.serialize_struct("Brep", 4)?;
        stored.serialize_field("source", &self.canonical.source)?;
        stored.serialize_field("source_sha256", &self.canonical.source_sha256)?;
        stored.serialize_field("display_sha256", &self.display_sha256)?;
        stored.serialize_field("display", &self.display)?;
        stored.end()
    }
}
impl<'de> Deserialize<'de> for Brep {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct StoredBrep {
            source: Arc<String>,
            source_sha256: [u8; 32],
            display_sha256: [u8; 32],
            display: Arc<BrepDisplay>,
        }
        let stored = StoredBrep::deserialize(deserializer)?;
        let binding_sha256 = source_topology_fingerprint(stored.source_sha256, &stored.display.topology);
        Ok(Self {
            canonical: BrepSource { topology: stored.display.topology.0.clone(), source: stored.source,
                source_sha256: stored.source_sha256, binding_sha256, allocation_owner: None, topology_allocation_owner:None, identity: BrepSourceIdentity(Uuid::new_v4()) },
            display_sha256: stored.display_sha256,
            display: stored.display,
            display_allocation_owner: None,
            display_identity: Uuid::new_v4(),
        })
    }
}
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BrepDisplay {
    pub source_sha256: [u8; 32],
    pub deflection_mm: f64,
    pub points: Vec<Point3>,
    /// Three display vertex offsets and a snapshot-local face offset.
    pub triangles: Vec<[u32; 4]>,
    /// Two display vertex offsets and a snapshot-local edge offset.
    pub segments: Vec<[u32; 3]>,
    /// A display vertex offset for each actual topological vertex.
    pub vertices: Vec<u32>,
    #[serde(flatten)]
    pub topology: BrepDisplayTopology,
}

/// Topology owned by a display. Borrowing exposes only the value, never the
/// independently clonable Arc; cloning a display copies its topology, and the
/// caller must admit and account for that independent allocation.
///
/// ```compile_fail
/// use std::sync::Arc;
/// use om9_core::brep::{BrepDisplay, BrepTopology};
/// fn escape(display: &BrepDisplay) -> Arc<BrepTopology> {
///     Arc::clone(&display.topology)
/// }
/// ```
#[repr(transparent)]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BrepDisplayTopology(pub(crate) Arc<BrepTopology>);
impl From<BrepTopology> for BrepDisplayTopology {
    fn from(topology: BrepTopology) -> Self { Self(Arc::new(topology)) }
}
impl std::ops::Deref for BrepDisplayTopology {
    type Target = BrepTopology;
    fn deref(&self) -> &BrepTopology { &self.0 }
}
impl std::ops::DerefMut for BrepDisplayTopology {
    fn deref_mut(&mut self) -> &mut BrepTopology { Arc::make_mut(&mut self.0) }
}
impl AsRef<BrepTopology> for BrepDisplayTopology {
    fn as_ref(&self) -> &BrepTopology { &self.0 }
}
impl Clone for BrepDisplay {
    fn clone(&self) -> Self {
        Self { source_sha256: self.source_sha256, deflection_mm: self.deflection_mm,
            points: self.points.clone(), triangles: self.triangles.clone(), segments: self.segments.clone(),
            vertices: self.vertices.clone(), topology: BrepTopology::clone(&self.topology).into() }
    }
}
impl BrepDisplay {
    fn allocation_bytes(&self) -> Result<u64> {
        let sizes = [
            Usage::array::<BrepDisplay>(1)?.bytes,
            Usage::array::<usize>(2)?.bytes,
            Usage::array::<Point3>(self.points.capacity())?.bytes,
            Usage::array::<[u32; 4]>(self.triangles.capacity())?.bytes,
            Usage::array::<[u32; 3]>(self.segments.capacity())?.bytes,
            Usage::array::<u32>(self.vertices.capacity())?.bytes,
        ];
        sizes.into_iter().try_fold(0u64, |sum, bytes| sum.checked_add(bytes).ok_or(Error::Overflow("BRep display allocation")))
    }
}

/// IDs and adjacency associated with a canonical B-rep source.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BrepTopology {
    pub faces: Vec<Uuid>,
    pub edges: Vec<Uuid>,
    pub vertex_ids: Vec<Uuid>,
    /// Face/edge adjacency, retaining closed boundaries and inner wires.
    pub face_edges: Vec<[u32; 2]>,
}

/// Detect accidental changes to source/ID/adjacency associations. This hash
/// is not an attestation that these IDs belong to native OCCT subshapes.
pub fn source_topology_fingerprint(source_sha256: [u8; 32], topology: &BrepTopology) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"OM9 canonical BRep topology 1\0");
    hash.update(source_sha256);
    for group in [&topology.faces, &topology.edges, &topology.vertex_ids] {
        hash.update((group.len() as u64).to_le_bytes());
        for id in group { hash.update(id.as_bytes()); }
    }
    hash.update((topology.face_edges.len() as u64).to_le_bytes());
    for [face, edge] in &topology.face_edges {
        hash.update(face.to_le_bytes()); hash.update(edge.to_le_bytes());
    }
    hash.finalize().into()
}

/// Portable source-only wire view. It borrows the tracked owner and never
/// includes tessellation; a codec must account its own output buffer.
#[derive(Serialize)]
pub struct CanonicalBrepRef<'a> {
    source: &'a str,
    source_sha256: [u8; 32],
    binding_sha256: [u8; 32],
    topology: &'a BrepTopology,
}

/// Untrusted source-only record for a future archive decoder. Deserialization
/// itself must be preceded by the codec's bounded admission/preflight; this
/// type does not claim that OCCT topology or persistent IDs are attested.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CanonicalBrepRecord {
    source: String,
    source_sha256: [u8; 32],
    binding_sha256: [u8; 32],
    #[serde(deserialize_with = "deserialize_canonical_topology")]
    topology: BrepTopology,
}
fn deserialize_canonical_topology<'de, D: serde::Deserializer<'de>>(deserializer: D)
    -> std::result::Result<BrepTopology, D::Error> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct TopologyRecord {
        faces: Vec<Uuid>, edges: Vec<Uuid>, vertex_ids: Vec<Uuid>, face_edges: Vec<[u32; 2]>,
    }
    let record = TopologyRecord::deserialize(deserializer)?;
    Ok(BrepTopology { faces: record.faces, edges: record.edges, vertex_ids: record.vertex_ids, face_edges: record.face_edges })
}
impl CanonicalBrepRecord {
    pub fn source(&self) -> &str { &self.source }
    pub fn topology(&self) -> &BrepTopology { &self.topology }
    pub fn fingerprint(&self) -> [u8; 32] { self.source_sha256 }
    /// Check source integrity, portable IDs and adjacency without a display.
    /// `admit` sees this record's actual capacities plus transient scratch;
    /// it does not cover the original encoded input or native OCCT memory.
    pub fn validate_admitted(&self, limits: &Limits,
        mut admit: impl FnMut(u64, u64) -> Result<()>) -> Result<Usage> {
        if self.source.is_empty() || !self.source.contains("CASCADE Topology")
            || <[u8; 32]>::from(Sha256::digest(self.source.as_bytes())) != self.source_sha256 {
            return Err(Error::Invalid("BRep source binding"));
        }
        if source_topology_fingerprint(self.source_sha256, &self.topology) != self.binding_sha256 {
            return Err(Error::Invalid("BRep source/topology integrity"));
        }
        if self.topology.faces.is_empty() { return Err(Error::Invalid("BRep source topology")); }
        let nodes = self.topology.faces.len().checked_add(self.topology.edges.len())
            .and_then(|n| n.checked_add(self.topology.vertex_ids.len())).ok_or(Error::Overflow("BRep topology count"))?;
        let allocations = [Usage::array::<u8>(self.source.capacity())?.bytes,
            Usage::array::<Uuid>(self.topology.faces.capacity())?.bytes,
            Usage::array::<Uuid>(self.topology.edges.capacity())?.bytes,
            Usage::array::<Uuid>(self.topology.vertex_ids.capacity())?.bytes,
            Usage::array::<[u32; 2]>(self.topology.face_edges.capacity())?.bytes];
        let bytes = allocations.into_iter().try_fold(std::mem::size_of::<Self>() as u64,
            |sum, bytes| sum.checked_add(bytes).ok_or(Error::Overflow("BRep source bytes")))?;
        let mut usage = Usage { bytes, nodes: u64::try_from(nodes).map_err(|_| Error::Overflow("BRep topology count"))?, ..Usage::default() };
        usage.check(limits)?;
        validate_topology_admitted(&self.topology, &mut usage, limits, &mut admit)?;
        Ok(usage)
    }
}
/// Source and topology owners, without the derived display mesh. A document
/// owns this through its BRep; independent captures require a tracked lease.
#[derive(Debug, Clone)]
pub(crate) struct BrepSource {
    identity: BrepSourceIdentity,
    source: Arc<String>,
    topology: Arc<BrepTopology>,
    source_sha256: [u8; 32],
    // Integrity of this in-memory association only; this is not a native
    // topology attestation and cannot authorize v4 display eviction.
    binding_sha256: [u8; 32],
    allocation_owner: Option<AllocationOwner>,
    topology_allocation_owner: Option<AllocationOwner>,
}
impl PartialEq for BrepSource {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source && self.topology == other.topology
            && self.source_sha256 == other.source_sha256 && self.binding_sha256 == other.binding_sha256
    }
}

/// Source and topology transferred from a B-rep candidate without its display.
/// The caller must account for this owner for its entire lifetime. Its shared
/// allocations cannot be cloned independently of the owner.
///
/// ```compile_fail
/// use std::sync::Arc;
/// use om9_core::brep::{OwnedBrepSource, BrepTopology};
/// fn escape(source: &OwnedBrepSource) -> Arc<BrepTopology> {
///     Arc::clone(source.topology())
/// }
/// ```
#[derive(Debug)]
pub struct OwnedBrepSource(BrepSource);
impl OwnedBrepSource {
    pub fn source(&self) -> &str { self.0.source() }
    pub fn topology(&self) -> &BrepTopology { self.0.topology() }
    pub fn storage_bytes(&self) -> Result<u64> {
        self.0.source_bytes()?.checked_add(self.0.topology_bytes()?)
            .ok_or(Error::Overflow("BRep source storage"))
    }
}

/// Diagnostic weak owner; account for the weak allocation header when held.
#[doc(hidden)]
pub struct BrepSourceLiveness(Weak<String>);
impl BrepSourceLiveness {
    pub fn is_alive(&self) -> bool { self.0.strong_count() != 0 }
}
impl BrepSource {
    pub(crate) fn source(&self) -> &str { &self.source }
    pub(crate) fn identity(&self) -> BrepSourceIdentity { self.identity }
    pub(crate) fn source_strong_count(&self) -> usize { Arc::strong_count(&self.source) }
    pub(crate) fn topology(&self) -> &BrepTopology { &self.topology }
    pub(crate) fn canonical_record(&self) -> CanonicalBrepRef<'_> {
        CanonicalBrepRef { source: &self.source, source_sha256: self.source_sha256,
            binding_sha256: self.binding_sha256, topology: &self.topology }
    }
    pub(crate) fn fingerprint(&self) -> [u8; 32] { self.source_sha256 }
    pub(crate) fn source_key(&self) -> usize { Arc::as_ptr(&self.source) as usize }
    pub(crate) fn topology_key(&self) -> usize { Arc::as_ptr(&self.topology) as usize }
    pub(crate) fn source_bytes(&self) -> Result<u64> {
        Usage::array::<u8>(self.source.capacity())?.bytes
            .checked_add((std::mem::size_of::<String>() + 2 * std::mem::size_of::<usize>()) as u64)
            .ok_or(Error::Overflow("BRep source allocation"))
    }
    pub(crate) fn topology_bytes(&self) -> Result<u64> {
        let d = &self.topology;
        [Usage::array::<BrepTopology>(1)?.bytes, Usage::array::<usize>(2)?.bytes,
            Usage::array::<Uuid>(d.faces.capacity())?.bytes, Usage::array::<Uuid>(d.edges.capacity())?.bytes,
            Usage::array::<Uuid>(d.vertex_ids.capacity())?.bytes, Usage::array::<[u32; 2]>(d.face_edges.capacity())?.bytes]
            .into_iter().try_fold(0u64, |sum, bytes| sum.checked_add(bytes).ok_or(Error::Overflow("BRep topology allocation")))
    }
    /// Validate canonical source and IDs without requiring any display mesh.
    /// `admit` includes the already-owned source and temporary UUID scratch.
    pub(crate) fn validate_admitted(&self, limits: &Limits, admit: &mut dyn FnMut(u64, u64) -> Result<()>) -> Result<Usage> {
        let digest: [u8; 32] = Sha256::digest(self.source.as_bytes()).into();
        if self.source.is_empty() || !self.source.contains("CASCADE Topology") || digest != self.source_sha256 {
            return Err(Error::Invalid("BRep source binding"));
        }
        if source_topology_fingerprint(self.source_sha256, &self.topology) != self.binding_sha256 {
            return Err(Error::Invalid("BRep source/topology integrity"));
        }
        if self.topology.faces.is_empty() { return Err(Error::Invalid("BRep source topology")); }
        let nodes = self.topology.faces.len().checked_add(self.topology.edges.len())
            .and_then(|n| n.checked_add(self.topology.vertex_ids.len())).ok_or(Error::Overflow("BRep topology count"))?;
        let mut usage = Usage { bytes: self.source_bytes()?.checked_add(self.topology_bytes()?).ok_or(Error::Overflow("BRep source bytes"))?,
            nodes: u64::try_from(nodes).map_err(|_| Error::Overflow("BRep topology count"))?, ..Usage::default() };
        usage.check(limits)?;
        validate_topology_admitted(&self.topology, &mut usage, limits, admit)?;
        Ok(usage)
    }
}

fn validate_topology_admitted(topology: &BrepTopology, usage: &mut Usage, limits: &Limits,
    admit: &mut dyn FnMut(u64, u64) -> Result<()>) -> Result<()> {
    if topology.face_edges.iter().any(|[face, edge]| *face as usize >= topology.faces.len() || *edge as usize >= topology.edges.len()) {
        return Err(Error::Invalid("BRep boundary topology"));
    }
    let count = topology.faces.len().checked_add(topology.edges.len())
        .and_then(|n| n.checked_add(topology.vertex_ids.len())).ok_or(Error::Overflow("BRep topology count"))?;
    let pending = Usage::array::<Uuid>(count)?;
    let before = usage.bytes;
    usage.add(pending, limits)?;
    admit(before, pending.bytes)?;
    let mut ids = Vec::new();
    ids.try_reserve_exact(count).map_err(|_| Error::Budget("BRep topology identities"))?;
    let actual = Usage::array::<Uuid>(ids.capacity())?.bytes;
    let extra = actual.checked_sub(pending.bytes).ok_or(Error::Overflow("BRep topology identities"))?;
    usage.add(Usage { bytes: extra, ..Usage::default() }, limits)?;
    admit(usage.bytes, 0)?;
    for keys in [&topology.faces, &topology.edges, &topology.vertex_ids] { ids.extend_from_slice(keys); }
    ids.sort_unstable();
    if ids.iter().any(Uuid::is_nil) || ids.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(Error::Invalid("BRep topology identity"));
    }
    usage.bytes = usage.bytes.checked_sub(actual).ok_or(Error::Overflow("BRep topology identities"))?;
    drop(ids);
    admit(usage.bytes, 0)?;
    Ok(())
}

impl std::ops::Deref for BrepDisplay {
    type Target = BrepTopology;
    fn deref(&self) -> &BrepTopology {
        &self.topology
    }
}
impl std::ops::DerefMut for BrepDisplay {
    fn deref_mut(&mut self) -> &mut BrepTopology {
        &mut self.topology
    }
}
impl Brep {
    pub fn into_canonical_and_representation(self) -> (CanonicalBrep, BrepRepresentation) {
        let profile = BrepRepresentationProfile::Unspecified {
            linear: self.display.deflection_mm.to_bits(), display: self.display_identity,
        };
        let representation = BrepRepresentation { source: self.canonical.identity,
            source_liveness: Arc::downgrade(&self.canonical.source),
            display: self.display, display_allocation_owner: self.display_allocation_owner,
            source_allocation_owner: self.canonical.allocation_owner.clone(),
            topology_allocation_owner: self.canonical.topology_allocation_owner.clone(),
            display_sha256: self.display_sha256, display_identity: self.display_identity, profile };
        (CanonicalBrep(self.canonical, None), representation)
    }
    /// Transfer an independently rebuilt candidate to an existing canonical
    /// source after the adapter has attested the topology mapping. The mapping
    /// runs from canonical indices to candidate indices; this method checks its
    /// structural consistency but does not prove its native provenance.
    pub fn into_representation_for_canonical(self, source: &CanonicalBrep,
        mapping: &topology_mapping::IndexMapping, limits: &Limits,
        mut admit: impl FnMut(u64, u64) -> Result<()>) -> Result<BrepRepresentation> {
        if self.canonical.allocation_owner.is_some() || self.canonical.topology_allocation_owner.is_some()
            || self.display_allocation_owner.is_some() || !self.has_exclusive_storage() {
            return Err(Error::Invalid("shared BRep representation candidate"));
        }
        if self.source() != source.source() { return Err(Error::Stale("BRep representation source")); }
        let held = self.storage_usage(limits)?.bytes.checked_add(source.storage_usage(limits)?.bytes)
            .ok_or(Error::Overflow("BRep representation transfer"))?;
        let remaining = limits.remaining_after(Usage { bytes: held, ..Usage::default() })?;
        let topology = mapping.reorder_topology_admitted(source.topology(), &self.topology().face_edges,
            &remaining, &mut |live, pending| admit(held.checked_add(live)
                .ok_or(Error::Overflow("BRep representation transfer"))?, pending))?;
        if &topology != self.topology() { return Err(Error::Invalid("BRep representation topology")); }
        drop(topology);

        // Build inverse face/edge maps for display references and an ordered
        // vertex array. No native or document state changes before validation.
        let counts = [mapping.faces().len(), mapping.edges().len(), mapping.vertices().len()];
        let scratch = counts.into_iter().try_fold(0u64, |sum, count| {
            sum.checked_add(Usage::array::<u32>(count)?.bytes).ok_or(Error::Overflow("BRep representation mapping"))
        })?;
        remaining.remaining_after(Usage { bytes: scratch, ..Usage::default() })?;
        admit(held, scratch)?;
        fn inverse(indices: &[u32]) -> Result<Vec<u32>> {
            let mut result = Vec::new();
            result.try_reserve_exact(indices.len()).map_err(|_| Error::Budget("BRep representation mapping"))?;
            result.resize(indices.len(), u32::MAX);
            for (old, &new) in indices.iter().enumerate() {
                let slot = result.get_mut(new as usize).ok_or(Error::Invalid("BRep representation mapping"))?;
                if *slot != u32::MAX { return Err(Error::Invalid("BRep representation mapping")); }
                *slot = u32::try_from(old).map_err(|_| Error::Overflow("BRep representation mapping"))?;
            }
            Ok(result)
        }
        let faces = inverse(mapping.faces())?;
        let edges = inverse(mapping.edges())?;
        let mut vertices = Vec::new();
        vertices.try_reserve_exact(counts[2]).map_err(|_| Error::Budget("BRep representation vertices"))?;
        for &index in mapping.vertices() {
            vertices.push(*self.display.vertices.get(index as usize).ok_or(Error::Invalid("BRep representation vertex"))?);
        }
        let actual = [faces.capacity(), edges.capacity(), vertices.capacity()].into_iter().try_fold(0u64, |sum, count| {
            sum.checked_add(Usage::array::<u32>(count)?.bytes).ok_or(Error::Overflow("BRep representation mapping"))
        })?;
        remaining.remaining_after(Usage { bytes: actual, ..Usage::default() })?;
        admit(held.checked_add(actual).ok_or(Error::Overflow("BRep representation transfer"))?, 0)?;
        let mut display = Arc::try_unwrap(self.display).map_err(|_| Error::Invalid("shared BRep representation candidate"))?;
        for triangle in &mut display.triangles {
            triangle[3] = *faces.get(triangle[3] as usize).ok_or(Error::Invalid("BRep representation face"))?;
        }
        for segment in &mut display.segments {
            segment[2] = *edges.get(segment[2] as usize).ok_or(Error::Invalid("BRep representation edge"))?;
        }
        display.vertices = vertices;
        display.topology = BrepDisplayTopology(Arc::clone(&source.0.topology));
        let display_sha256 = crate::retained::fingerprint(&display)?;
        let display_identity = Uuid::new_v4();
        let profile = BrepRepresentationProfile::Unspecified { linear: display.deflection_mm.to_bits(), display: display_identity };
        let result = BrepRepresentation { source: source.identity(), source_liveness: Arc::downgrade(&source.0.source), display: Arc::new(display),
            display_allocation_owner: None, source_allocation_owner: source.0.allocation_owner.clone(),
            topology_allocation_owner: source.0.topology_allocation_owner.clone(),
            display_sha256, display_identity, profile };
        let candidate = Brep { canonical: source.0.clone(), display: Arc::clone(&result.display),
            display_allocation_owner: None, display_sha256: result.display_sha256,
            display_identity: result.display_identity };
        candidate.validate_admitted(limits, &mut |live, pending| {
            let base = held.checked_add(actual).and_then(|n| n.checked_add(live))
                .ok_or(Error::Overflow("BRep representation validation"))?;
            limits.remaining_after(Usage { bytes: base.checked_add(pending)
                .ok_or(Error::Overflow("BRep representation validation"))?, ..Usage::default() })?;
            admit(base, pending)
        })?;
        Ok(result)
    }
    /// Temporary kernel/legacy adapter. Neither Document nor Undo owns this pair.
    pub fn from_canonical_and_representation(source: &CanonicalBrep, representation: &BrepRepresentation,
        limits: &Limits) -> Result<Self> {
        Self::from_canonical_and_representation_admitted(source, representation, limits, &mut |_, _| Ok(()))
    }
    pub fn from_canonical_and_representation_admitted(source: &CanonicalBrep, representation: &BrepRepresentation,
        limits: &Limits, admit: &mut dyn FnMut(u64, u64) -> Result<()>) -> Result<Self> {
        if !representation.matches(source) { return Err(Error::Stale("BRep representation source")); }
        let candidate = Self { canonical: source.0.clone(), display: Arc::clone(&representation.display),
            display_allocation_owner: representation.display_allocation_owner.clone(),
            display_sha256: representation.display_sha256, display_identity: representation.display_identity };
        candidate.validate_admitted(limits, admit)?;
        Ok(candidate)
    }
    /// True only for the same immutable allocations and every binding field
    /// relevant to validation. This is valid for a single borrowed traversal;
    /// it is not a persistent proof or a native topology attestation.
    #[doc(hidden)]
    pub fn shares_validation_payload(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.canonical.source, &other.canonical.source)
            && Arc::ptr_eq(&self.canonical.topology, &other.canonical.topology)
            && Arc::ptr_eq(&self.display, &other.display)
            && self.canonical.source_sha256 == other.canonical.source_sha256
            && self.canonical.binding_sha256 == other.canonical.binding_sha256
            && self.display_sha256 == other.display_sha256
            && self.display_identity == other.display_identity
    }
    /// Borrow the display currently associated with this B-rep.
    pub fn display(&self) -> &BrepDisplay { &self.display }

    /// Consume an exclusively owned candidate without exposing a shared Arc.
    pub fn try_into_display(self) -> Result<BrepDisplay> {
        if self.canonical.allocation_owner.is_some() {
            return Err(Error::Invalid("owned BRep display transfer"));
        }
        Arc::try_unwrap(self.display).map_err(|_| Error::Busy)
    }

    /// Retain a display together with the reservation owner supplied by the
    /// caller. The owner must cover this display until the last pin is dropped.
    #[doc(hidden)]
    pub fn pin_display(&self, owner: Arc<dyn std::any::Any + Send + Sync>) -> BrepDisplayPin {
        BrepDisplayPin { display: self.display.clone(), _owner: owner,
            _candidate_owner: self.display_allocation_owner.clone(),
            _source_owner: if self.canonical.topology_allocation_owner.is_some() {None} else {self.canonical.allocation_owner.clone()},
            _topology_owner:self.canonical.topology_allocation_owner.clone() }
    }

    /// A display pin clones this independent topology owner. Only an allocated
    /// lease in the same account can replace a separate topology reservation.
    pub fn topology_owner_covers_allocation(&self, account: &crate::transaction::ExternalMemoryAccount) -> Result<bool> {
        let Some(owner) = self.canonical.topology_allocation_owner.as_ref().and_then(|owner|
            owner._owner.downcast_ref::<std::sync::Mutex<crate::transaction::ExternalMemoryReservation>>()) else {
            return Ok(false);
        };
        let guard = owner.lock().unwrap_or_else(|error| error.into_inner());
        Ok(guard.belongs_to(account)
            && guard.bytes().saturating_sub(guard.pending_bytes()) >= self.topology_allocation_bytes()?)
    }

    /// Identity receipts do not retain display data or its Arc allocation.
    #[doc(hidden)]
    pub fn display_identity(&self) -> BrepDisplayIdentity { BrepDisplayIdentity(self.display_identity) }

    /// Diagnostic-only liveness observation; unlike an identity this keeps a
    /// weak allocation header and must be covered by its caller's accounting.
    #[doc(hidden)]
    pub fn display_liveness(&self) -> BrepDisplayLiveness { BrepDisplayLiveness(Arc::downgrade(&self.display)) }

    /// Direct mutation invalidates the display fingerprint and is only useful
    /// to exercise validation of damaged inputs.
    #[doc(hidden)]
    pub fn edit_display_for_validation(&mut self, edit: impl FnOnce(&mut BrepDisplay)) {
        self.display_identity = Uuid::new_v4();
        edit(Arc::make_mut(&mut self.display));
    }
    /// Adapter publication boundary. The kernel validates the shape before calling this constructor.
    /// Portable validation checks ownership/bounds and source binding, not OCCT validity.
    pub fn from_native(source: String, mut display: BrepDisplay, limits: &Limits) -> Result<Self> {
        let digest: [u8; 32] = Sha256::digest(source.as_bytes()).into();
        if display.source_sha256 != [0; 32] && display.source_sha256 != digest {
            return Err(Error::Invalid("BRep source/cache binding"));
        }
        display.source_sha256 = digest;
        let display_sha256 = crate::retained::fingerprint(&display)?;
        let shape = Self {
            canonical: BrepSource { topology: display.topology.0.clone(), source: Arc::new(source), source_sha256: digest,
                binding_sha256: source_topology_fingerprint(digest, &display.topology), allocation_owner: None, topology_allocation_owner:None,
                identity: BrepSourceIdentity(Uuid::new_v4()) },
            display_sha256,
            display: Arc::new(display),
            display_allocation_owner: None,
            display_identity: Uuid::new_v4(),
        };
        shape.validate(limits)?;
        Ok(shape)
    }
    pub fn source(&self) -> &str {
        self.canonical.source()
    }
    /// Bind a caller's allocation owner before the candidate can be cloned or published.
    /// The caller must ensure the owner accounts for every shared allocation it covers.
    /// This is a transient owner: do not insert the BRep into Document/Undo before a
    /// checked ownership transfer, because the lease can otherwise drop under account locks.
    #[doc(hidden)]
    pub fn bind_allocation_owner(&mut self, owner: Arc<dyn std::any::Any + Send + Sync>) -> Result<()> {
        if self.canonical.allocation_owner.is_some() || !self.has_exclusive_storage() {
            return Err(Error::Invalid("shared BRep allocation owner"));
        }
        self.canonical.allocation_owner = Some(AllocationOwner { _owner: owner });
        Ok(())
    }
    /// Attach a display-only lease while canonical source aliases already exist.
    /// No other display reader may exist before this handoff.
    pub fn bind_display_allocation_owner(&mut self,
        owner: Arc<std::sync::Mutex<crate::transaction::ExternalMemoryReservation>>,
        account: &crate::transaction::ExternalMemoryAccount) -> Result<()> {
        if self.canonical.allocation_owner.is_some() || self.display_allocation_owner.is_some()
            || Arc::strong_count(&self.display) != 1 || Arc::weak_count(&self.display) != 0 {
            return Err(Error::Invalid("shared BRep display owner"));
        }
        let bytes = self.display_allocation_bytes()?;
        {
            let guard = owner.lock().unwrap_or_else(|error| error.into_inner());
            if !guard.belongs_to(account) || guard.bytes().saturating_sub(guard.pending_bytes()) < bytes {
                return Err(Error::Invalid("BRep display owner accounting"));
            }
        }
        self.display_allocation_owner = Some(AllocationOwner { _owner: owner });
        Ok(())
    }
    /// Return the existing tracked display lease, if this candidate has one.
    pub fn display_owner_reservation(&self) -> Option<Arc<std::sync::Mutex<crate::transaction::ExternalMemoryReservation>>> {
        Arc::downcast::<std::sync::Mutex<crate::transaction::ExternalMemoryReservation>>(
            self.display_allocation_owner.as_ref()?._owner.clone()).ok()
    }
    /// Bind independently accounted source, shared topology and display owners.
    /// All three must use the same account and class, and contain allocated
    /// charges sufficient for their respective allocations.
    #[doc(hidden)]
    pub fn bind_split_allocation_owners(&mut self,source:Arc<std::sync::Mutex<crate::transaction::ExternalMemoryReservation>>,
        topology:Arc<std::sync::Mutex<crate::transaction::ExternalMemoryReservation>>,
        display:Arc<std::sync::Mutex<crate::transaction::ExternalMemoryReservation>>)->Result<()> {
        if self.canonical.allocation_owner.is_some() || self.canonical.topology_allocation_owner.is_some()
            || self.display_allocation_owner.is_some() || !self.has_exclusive_storage()
            || Arc::ptr_eq(&source,&display) || Arc::ptr_eq(&source,&topology) || Arc::ptr_eq(&topology,&display)
            {return Err(Error::Invalid("shared BRep allocation owner"));}
        {
            let mut order=[(Arc::as_ptr(&source) as usize,0,&source),(Arc::as_ptr(&topology) as usize,1,&topology),
                (Arc::as_ptr(&display) as usize,2,&display)];
            order.sort_unstable_by_key(|item|item.0);
            let first=order[0].2.lock().unwrap_or_else(|error|error.into_inner());
            let second=order[1].2.lock().unwrap_or_else(|error|error.into_inner());
            let third=order[2].2.lock().unwrap_or_else(|error|error.into_inner());
            let guards=[&first,&second,&third];
            let source_guard=guards[order.iter().position(|item|item.1==0).expect("source guard")];
            let topology_guard=guards[order.iter().position(|item|item.1==1).expect("topology guard")];
            let display_guard=guards[order.iter().position(|item|item.1==2).expect("display guard")];
            if !source_guard.shares_account_and_class(topology_guard) || !source_guard.shares_account_and_class(display_guard)
                || source_guard.bytes().saturating_sub(source_guard.pending_bytes())<self.source_allocation_bytes()?
                || topology_guard.bytes().saturating_sub(topology_guard.pending_bytes())<self.topology_allocation_bytes()?
                || display_guard.bytes().saturating_sub(display_guard.pending_bytes())<self.display_allocation_bytes()? {
                return Err(Error::Invalid("BRep split owner accounting"));
            }
        }
        self.canonical.allocation_owner=Some(AllocationOwner {_owner:source});
        self.canonical.topology_allocation_owner=Some(AllocationOwner {_owner:topology});
        self.display_allocation_owner=Some(AllocationOwner {_owner:display});Ok(())
    }
    #[doc(hidden)]
    pub fn has_allocation_owner(&self) -> bool { self.canonical.allocation_owner.is_some() }
    /// Only for coordinator staging while the original guarded owner is borrowed.
    pub(crate) fn clone_for_document_staging(&self) -> Self {
        let mut clone = self.clone();
        clone.canonical.allocation_owner = None;
        clone.canonical.topology_allocation_owner = None;
        clone.display_allocation_owner = None;
        clone
    }
    #[doc(hidden)]
    pub fn allocation_covered_by(&self, account: &crate::transaction::ExternalMemoryAccount, bytes: u64) -> bool {
        let Some(owner) = self.canonical.allocation_owner.as_ref().and_then(|owner|
            owner._owner.downcast_ref::<std::sync::Mutex<crate::transaction::ExternalMemoryReservation>>()) else { return false; };
        let guard = owner.lock().unwrap_or_else(|error| error.into_inner());
        if !guard.belongs_to(account) {return false;}
        let mut covered=guard.bytes().saturating_sub(guard.pending_bytes());drop(guard);
        for extra in [self.canonical.topology_allocation_owner.as_ref(),self.display_allocation_owner.as_ref()].into_iter().flatten() {
            let Some(owner)=extra._owner.downcast_ref::<std::sync::Mutex<crate::transaction::ExternalMemoryReservation>>() else {return false;};
            let guard=owner.lock().unwrap_or_else(|error|error.into_inner());
            if !guard.belongs_to(account) {return false;}
            covered=covered.saturating_add(guard.bytes().saturating_sub(guard.pending_bytes()));
        }
        covered>=bytes
    }
    /// Fast candidate filter only; hashes do not attest native topology.
    pub fn canonical_fingerprints(&self) -> ([u8; 32], [u8; 32]) {
        (self.canonical.source_sha256, self.canonical.binding_sha256)
    }
    pub fn topology(&self) -> &BrepTopology {
        self.canonical.topology()
    }
    /// Check the immutable source and IDs without traversing the derived mesh.
    /// This checks portable integrity, not native OCCT shape validity or ID provenance.
    pub fn validate_source(&self, limits: &Limits) -> Result<Usage> {
        self.canonical.validate_admitted(limits, &mut |_, _| Ok(()))
    }
    /// Account source/topology validation scratch separately from the already
    /// retained canonical allocations.
    pub fn validate_source_admitted(&self, limits: &Limits,
        mut admit: impl FnMut(u64, u64) -> Result<()>) -> Result<Usage> {
        self.canonical.validate_admitted(limits, &mut admit)
    }
    pub fn source_storage_bytes(&self) -> Result<u64> {
        self.canonical.source_bytes()?.checked_add(self.canonical.topology_bytes()?)
            .ok_or(Error::Overflow("BRep source storage"))
    }
    /// Transfer source and topology together, without retaining the display.
    /// Callers must keep the returned owner under their existing reservation.
    pub fn into_owned_source(self) -> OwnedBrepSource { OwnedBrepSource(self.canonical) }
    /// Diagnostic-only liveness observation; the caller accounts for its weak header.
    #[doc(hidden)]
    pub fn source_liveness(&self) -> BrepSourceLiveness { BrepSourceLiveness(Arc::downgrade(&self.canonical.source)) }
    pub(crate) fn tracked_source_snapshot(&self) -> BrepSource {
        self.canonical.clone()
    }
    pub(crate) fn tracked_display_snapshot(&self) -> Arc<BrepDisplay> { Arc::clone(&self.display) }
    /// Fingerprint of the validated source-bound display and topology snapshot.
    pub fn display_fingerprint(&self) -> [u8; 32] {
        self.display_sha256
    }
    /// Owned allocations needed when a scene retains only this display and
    /// its topology, after the candidate source has been released.
    pub fn display_storage_bytes(&self) -> Result<u64> {
        if !Arc::ptr_eq(&self.canonical.topology, &self.display.topology.0) {
            return Err(Error::Invalid("BRep display topology owner"));
        }
        self.display_allocation_bytes()?.checked_add(self.topology_allocation_bytes()?)
            .ok_or(Error::Overflow("BRep display storage"))
    }
    /// Whether this value exclusively owns all allocations transferable to an
    /// inserted document. Weak owners also retain Arc allocation headers.
    pub(crate) fn has_exclusive_storage(&self) -> bool {
        Arc::strong_count(&self.canonical.source) == 1 && Arc::weak_count(&self.canonical.source) == 0
            && Arc::strong_count(&self.display) == 1 && Arc::weak_count(&self.display) == 0
            && Arc::strong_count(&self.canonical.topology) == 2 && Arc::weak_count(&self.canonical.topology) == 0
    }
    pub(crate) fn can_adopt_unowned_storage(&self) -> bool {
        self.has_exclusive_storage() && self.canonical.allocation_owner.is_none()
            && self.canonical.topology_allocation_owner.is_none() && self.display_allocation_owner.is_none()
    }
    /// Existing leases may be shared with previews, but they cannot be used to
    /// account a representation in another coordinator's memory account.
    pub(crate) fn allocation_owners_belong_to(&self, account: &crate::transaction::ExternalMemoryAccount) -> bool {
        [self.canonical.allocation_owner.as_ref(), self.canonical.topology_allocation_owner.as_ref(),
            self.display_allocation_owner.as_ref()].into_iter().flatten().all(|owner| {
            owner._owner.downcast_ref::<std::sync::Mutex<crate::transaction::ExternalMemoryReservation>>()
                .is_some_and(|reservation| reservation.lock().unwrap_or_else(|error| error.into_inner()).belongs_to(account))
        })
    }
    pub(crate) fn display_allocation_covered_by(&self, account: &crate::transaction::ExternalMemoryAccount) -> Result<bool> {
        let Some(owner) = self.display_allocation_owner.as_ref().and_then(|owner|
            owner._owner.downcast_ref::<std::sync::Mutex<crate::transaction::ExternalMemoryReservation>>()) else {
            return Ok(false);
        };
        let guard = owner.lock().unwrap_or_else(|error| error.into_inner());
        Ok(guard.belongs_to(account)
            && guard.bytes().saturating_sub(guard.pending_bytes()) >= self.display.allocation_bytes()?)
    }
    /// Replace topology on an exclusively owned candidate display after the
    /// caller has verified source-bound native descriptors. This method checks
    /// structural validity; it does not itself prove descriptor provenance.
    /// `limits` excludes other retained allocations, and `admit` must account
    /// for the old candidate, new topology, Arc overlap and validation scratch.
    pub fn rebind_candidate_topology(self, topology: BrepTopology, limits: &Limits,
        mut admit: impl FnMut(u64, u64) -> Result<()>) -> Result<Self> {
        if self.canonical.allocation_owner.is_some() {
            return Err(Error::Invalid("owned BRep topology rebind"));
        }
        if Arc::strong_count(&self.display) != 1 || Arc::weak_count(&self.display) != 0
            || Arc::strong_count(&self.canonical.topology) != 2 || Arc::weak_count(&self.canonical.topology) != 0 {
            return Err(Error::Invalid("shared BRep candidate"));
        }
        let arrays = [Usage::array::<Uuid>(topology.faces.capacity())?.bytes,
            Usage::array::<Uuid>(topology.edges.capacity())?.bytes, Usage::array::<Uuid>(topology.vertex_ids.capacity())?.bytes,
            Usage::array::<[u32; 2]>(topology.face_edges.capacity())?.bytes].into_iter()
            .try_fold(0u64, |sum, value| sum.checked_add(value).ok_or(Error::Overflow("BRep topology candidate")))?;
        let held = self.storage_usage(limits)?.bytes.checked_add(arrays).ok_or(Error::Overflow("BRep topology candidate"))?;
        let pending = Usage::array::<BrepTopology>(1)?.bytes.checked_add(Usage::array::<BrepDisplay>(1)?.bytes)
            .and_then(|v| v.checked_add(Usage::array::<usize>(4).ok()?.bytes))
            .ok_or(Error::Overflow("BRep topology candidate"))?;
        if held.checked_add(pending).ok_or(Error::Overflow("BRep topology candidate"))? > limits.max_bytes {
            return Err(Error::Budget("BRep topology candidate"));
        }
        admit(held, pending)?;
        let Self { canonical: BrepSource { source, source_sha256, topology: old_topology, allocation_owner, .. }, display, .. } = self;
        let mut display = Arc::try_unwrap(display).map_err(|_| Error::Invalid("shared BRep candidate"))?;
        let topology = Arc::new(topology);
        display.topology = BrepDisplayTopology(topology.clone());
        drop(old_topology);
        let display_sha256 = crate::retained::fingerprint(&display)?;
        let binding_sha256 = source_topology_fingerprint(source_sha256, &topology);
        let shape = Self { canonical: BrepSource { source, source_sha256, topology, binding_sha256, allocation_owner, topology_allocation_owner:None,
            identity: BrepSourceIdentity(Uuid::new_v4()) }, display_sha256, display: Arc::new(display), display_allocation_owner:None, display_identity: Uuid::new_v4() };
        shape.validate_admitted(limits, &mut admit)?;
        Ok(shape)
    }
    pub(crate) fn source_allocation_key(&self) -> usize {
        self.canonical.source_key()
    }
    pub fn source_allocation_bytes(&self) -> Result<u64> {
        self.canonical.source_bytes()
    }
    pub(crate) fn display_allocation_key(&self) -> usize {
        Arc::as_ptr(&self.display) as usize
    }
    pub fn display_allocation_bytes(&self) -> Result<u64> {
        self.display.allocation_bytes()
    }
    pub(crate) fn topology_allocation_key(&self) -> usize {
        self.canonical.topology_key()
    }
    pub fn topology_allocation_bytes(&self) -> Result<u64> {
        let d = &self.canonical.topology;
        let sizes = [
            Usage::array::<BrepTopology>(1)?.bytes,
            Usage::array::<usize>(2)?.bytes,
            Usage::array::<[u32; 2]>(d.face_edges.capacity())?.bytes,
            Usage::array::<Uuid>(d.faces.capacity())?.bytes,
            Usage::array::<Uuid>(d.edges.capacity())?.bytes,
            Usage::array::<Uuid>(d.vertex_ids.capacity())?.bytes,
        ];
        sizes.into_iter().try_fold(0u64, |sum, bytes| sum.checked_add(bytes).ok_or(Error::Overflow("BRep topology allocation")))
    }
    /// Inventory owned buffers without hashing, topology checks, or scratch allocation.
    /// Callers must still validate untrusted data before publication.
    pub fn storage_usage(&self, limits: &Limits) -> Result<Usage> {
        let d = &self.display;
        let mut usage = Usage { bytes: self.source_allocation_bytes()?, ..Usage::default() };
        usage.add(Usage { bytes: self.display_allocation_bytes()?, ..Usage::default() }, limits)?;
        usage.add(Usage { bytes: self.topology_allocation_bytes()?, ..Usage::default() }, limits)?;
        if !Arc::ptr_eq(&self.canonical.topology, &self.display.topology.0) {
            let d = &self.display.topology;
            let mut extra = Usage::array::<BrepTopology>(1)?;
            extra.add(Usage::array::<usize>(2)?, limits)?;
            extra.add(Usage::array::<[u32; 2]>(d.face_edges.capacity())?, limits)?;
            extra.add(Usage::array::<Uuid>(d.faces.capacity())?, limits)?;
            extra.add(Usage::array::<Uuid>(d.edges.capacity())?, limits)?;
            extra.add(Usage::array::<Uuid>(d.vertex_ids.capacity())?, limits)?;
            usage.add(extra, limits)?;
        }
        usage.points = d.points.len() as u64;
        usage.nodes = d
            .faces
            .len()
            .checked_add(d.edges.len())
            .and_then(|n| n.checked_add(d.vertices.len()))
            .ok_or(Error::Overflow("BRep topology count"))? as u64;
        usage.check(limits)?;
        Ok(usage)
    }
    pub fn validate(&self, limits: &Limits) -> Result<Usage> {
        self.validate_admitted(limits, &mut |_, _| Ok(()))
    }
    pub fn validate_admitted(&self, limits: &Limits, admit: &mut dyn FnMut(u64, u64) -> Result<()>) -> Result<Usage> {
        let d = &self.display;
        let digest: [u8; 32] = Sha256::digest(self.canonical.source.as_bytes()).into();
        if self.canonical.source.is_empty()
            || !self.canonical.source.contains("CASCADE Topology")
            || digest != self.canonical.source_sha256
            || source_topology_fingerprint(digest, &self.canonical.topology) != self.canonical.binding_sha256
            || digest != d.source_sha256
            || !Arc::ptr_eq(&self.canonical.topology, &d.topology.0)
            || crate::retained::fingerprint(d)? != self.display_sha256
        {
            return Err(Error::Invalid("BRep source/cache binding"));
        }
        if !d.deflection_mm.is_finite()
            || d.deflection_mm <= 0.
            || d.faces.is_empty()
            || d.triangles.is_empty()
            || d.vertices.len() != d.vertex_ids.len()
        {
            return Err(Error::Invalid("BRep display topology"));
        }
        let mut usage = self.storage_usage(limits)?;
        for p in &d.points {
            finite(*p)?;
        }
        for [a, b, c, f] in &d.triangles {
            if [a, b, c].iter().any(|i| **i as usize >= d.points.len())
                || *f as usize >= d.faces.len()
                || a == b
                || b == c
                || a == c
            {
                return Err(Error::Invalid("BRep triangle topology"));
            }
        }
        if d.segments.iter().any(|[a, b, e]| {
            *a as usize >= d.points.len()
                || *b as usize >= d.points.len()
                || *e as usize >= d.edges.len()
        }) || d.vertices.iter().any(|i| *i as usize >= d.points.len())
        {
            return Err(Error::Invalid("BRep boundary topology"));
        }
        validate_topology_admitted(&self.canonical.topology, &mut usage, limits, admit)?;
        Ok(usage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "release scaling probe; run explicitly"]
    fn document_validation_many_small_breps() {
        use crate::{geometry::Geometry, model::{Document, Entity}};
        let limits = Limits::default();
        for count in [200, 800, 1600] {
            let mut distinct = Document::new("independent small B-reps");
            let mut shared = Document::new("shared small B-reps");
            let display = BrepDisplay {
                source_sha256: [0; 32], deflection_mm: 0.01,
                points: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
                triangles: vec![[0, 1, 2, 0]], segments: vec![], vertices: vec![],
                topology: BrepTopology { faces: vec![Uuid::from_u128(10)], edges: vec![], vertex_ids: vec![], face_edges: vec![] }.into(),
            };
            let original = Brep::from_native("CASCADE Topology fixture".into(), display.clone(), &limits).unwrap();
            for index in 0..count {
                let mut independent_display = display.clone();
                independent_display.topology = (*display.topology).clone().into();
                let unique = Brep::from_native(format!("CASCADE Topology fixture {index}"), independent_display, &limits).unwrap();
                let entity = Entity::new(format!("independent {index}"), Geometry::Brep(unique), None);
                distinct.entities.insert(entity.id, entity);
                let entity = Entity::new(format!("shared {index}"), Geometry::Brep(original.clone()), None);
                shared.entities.insert(entity.id, entity);
            }
            for (name, document) in [("distinct", &distinct), ("shared", &shared)] {
                let mut elapsed = Vec::new();
                for _ in 0..3 {
                    let start = std::time::Instant::now();
                    std::hint::black_box(document.validate(&limits).unwrap());
                    elapsed.push(start.elapsed().as_micros());
                }
                eprintln!("{name}_{count}_validation_us={elapsed:?}");
            }
        }
    }

    #[test]
    fn document_rechecks_shared_brep_when_any_binding_or_owner_differs() {
        use crate::{geometry::Geometry, identity::ObjectId, model::{Definition, Document, Entity}};
        let limits = Limits::default();
        let display = BrepDisplay {
            source_sha256: [0; 32], deflection_mm: 0.01,
            points: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
            triangles: vec![[0, 1, 2, 0]], segments: vec![], vertices: vec![],
            topology: BrepTopology { faces: vec![Uuid::from_u128(10)], edges: vec![], vertex_ids: vec![], face_edges: vec![] }.into(),
        };
        let original = Brep::from_native("CASCADE Topology fixture".into(), display, &limits).unwrap();
        let make_document = |second: Brep| {
            let mut document = Document::new("shared integrity");
            let mut first = Entity::new("first", Geometry::Brep(original.clone()), None);
            first.id = ObjectId(Uuid::from_u128(1));
            let mut second = Entity::new("second", Geometry::Brep(second), None);
            second.id = ObjectId(Uuid::from_u128(2));
            document.entities.insert(first.id, first); document.entities.insert(second.id, second);
            document
        };
        let valid = make_document(original.clone());
        assert_eq!(valid.validate(&limits).unwrap(), valid.storage_usage(&limits).unwrap());
        let mut different_display = original.clone();
        Arc::make_mut(&mut different_display.display).deflection_mm = 0.02;
        Arc::make_mut(&mut different_display.display).topology = BrepDisplayTopology(different_display.canonical.topology.clone());
        different_display.display_sha256 = crate::retained::fingerprint(&*different_display.display).unwrap();
        different_display.display_identity = Uuid::new_v4();
        assert!(!original.shares_validation_payload(&different_display));
        let mixed = make_document(different_display);
        assert_eq!(mixed.validate(&limits).unwrap(), mixed.storage_usage(&limits).unwrap());
        let constrained = Limits { max_points: 5, ..limits };
        assert!(matches!(valid.validate(&constrained), Err(Error::Budget(_))), "shared geometry still counts each occurrence's point limit");
        for field in 0..5 {
            let mut bad = original.clone();
            match field {
                0 => bad.canonical.source_sha256[0] ^= 1,
                1 => bad.canonical.binding_sha256[0] ^= 1,
                2 => bad.display_sha256[0] ^= 1,
                3 => Arc::make_mut(&mut bad.display).triangles[0][0] = u32::MAX,
                _ => Arc::make_mut(&mut bad.canonical.topology).faces[0] = Uuid::from_u128(11),
            }
            assert!(!original.shares_validation_payload(&bad));
            assert!(make_document(bad).validate(&limits).is_err(), "corrupt field {field}");
        }
        let mut invalid_entity = make_document(original.clone());
        invalid_entity.entities.get_mut(&ObjectId(Uuid::from_u128(2))).unwrap().generation = 0;
        assert!(invalid_entity.validate(&limits).is_err(), "each entity must still be validated");
        let mut with_definition = make_document(original.clone());
        let member = Entity::new("shared member", Geometry::Brep(original.clone()), None);
        let definition = Definition::new("definition", vec![member]);
        let definition_id = definition.id;
        with_definition.definitions.insert(definition_id, definition);
        assert_eq!(with_definition.validate(&limits).unwrap(), with_definition.storage_usage(&limits).unwrap());
        let Geometry::Brep(member) = &mut with_definition.definitions.get_mut(&definition_id).unwrap().members[0].geometry else { panic!() };
        member.display_sha256[0] ^= 1;
        assert!(with_definition.validate(&limits).is_err(), "definition member binding must be checked on every traversal");
        let mut modified_after_validation = valid.clone();
        let Geometry::Brep(changed) = &mut modified_after_validation.entities.get_mut(&ObjectId(Uuid::from_u128(2))).unwrap().geometry else { panic!() };
        Arc::make_mut(&mut changed.display).points[0][0] = f64::NAN;
        assert!(modified_after_validation.validate(&limits).is_err(), "a later traversal must recheck changed data");
    }

    #[test]
    fn display_identity_never_retains_an_arc_allocation() {
        assert!(!std::mem::needs_drop::<BrepDisplayIdentity>());
        assert_eq!(std::mem::size_of::<BrepDisplayIdentity>(), std::mem::size_of::<Uuid>());
    }

    #[test]
    fn allocation_owner_follows_brep_source_and_display_pins() {
        let limits = Limits::default();
        let make = || Brep::from_native("CASCADE Topology fixture".into(), BrepDisplay {
            source_sha256: [0; 32], deflection_mm: 0.1,
            points: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
            triangles: vec![[0, 1, 2, 0]], segments: vec![], vertices: vec![],
            topology: BrepTopology { faces: vec![Uuid::from_u128(1)], edges: vec![],
                vertex_ids: vec![], face_edges: vec![] }.into(),
        }, &limits).unwrap();
        let owner = Arc::new(());
        let weak = Arc::downgrade(&owner);
        let mut brep = make();
        brep.bind_allocation_owner(owner.clone()).unwrap();
        assert!(brep.bind_allocation_owner(Arc::new(())).is_err());
        {
            use crate::{geometry::Geometry, model::{Document, Entity}};
            let geometry = Geometry::Brep(brep.clone());
            assert!(matches!(geometry.validate(&limits), Err(Error::Unsupported("transient leased BRep geometry"))));
            let entity = Entity::new("transient", geometry, None);
            let mut document = Document::new("must reject transient geometry");
            document.entities.insert(entity.id, entity);
            assert!(matches!(document.validate(&limits), Err(Error::Unsupported("transient leased BRep geometry"))));
        }
        drop(owner);
        let clone = brep.clone();
        let pin = clone.pin_display(Arc::new(7u32));
        assert_eq!(pin.owner::<u32>(), Some(&7));
        let source = brep.into_owned_source();
        drop(clone);
        assert!(weak.upgrade().is_some());
        drop(source);
        assert!(weak.upgrade().is_some(), "display pin must retain the candidate owner");
        drop(pin);
        assert!(weak.upgrade().is_none());

        let mut shared = make();
        let alias = shared.clone();
        assert!(shared.bind_allocation_owner(Arc::new(())).is_err());
        drop(alias);
        shared.bind_allocation_owner(Arc::new(())).unwrap();
        assert!(matches!(shared.try_into_display(), Err(Error::Invalid("owned BRep display transfer"))));
    }

    #[test]
    fn split_candidate_owner_releases_display_charge_while_source_lives() {
        use crate::{budget::memory::MemoryClass, transaction::{TransactionCoordinator,World}};
        let limits=Limits::default();
        let mut shape=Brep::from_native("CASCADE Topology fixture".into(),BrepDisplay {
            source_sha256:[0;32],deflection_mm:0.1,points:vec![[0.,0.,0.],[1.,0.,0.],[0.,1.,0.]],
            triangles:vec![[0,1,2,0]],segments:vec![],vertices:vec![],
            topology:BrepTopology {faces:vec![Uuid::from_u128(1)],edges:vec![],vertex_ids:vec![],face_edges:vec![]}.into(),
        },&limits).unwrap();
        let coordinator=TransactionCoordinator::new(World::new(vec![],&limits).unwrap(),limits).unwrap();
        let source_bytes=shape.source_allocation_bytes().unwrap();let topology_bytes=shape.topology_allocation_bytes().unwrap();
        let display_bytes=shape.display_allocation_bytes().unwrap();
        let mut source=coordinator.reserve_external_memory(MemoryClass::Kernel,source_bytes,None).unwrap();source.set_allocated_bytes(source_bytes).unwrap();
        let mut topology=coordinator.reserve_external_memory(MemoryClass::Kernel,topology_bytes,None).unwrap();topology.set_allocated_bytes(topology_bytes).unwrap();
        let mut display=coordinator.reserve_external_memory(MemoryClass::Kernel,display_bytes,None).unwrap();display.set_allocated_bytes(display_bytes).unwrap();
        let source=Arc::new(std::sync::Mutex::new(source));let topology=Arc::new(std::sync::Mutex::new(topology));let display=Arc::new(std::sync::Mutex::new(display));
        shape.bind_split_allocation_owners(source.clone(),topology.clone(),display.clone()).unwrap();
        assert!(shape.allocation_covered_by(&coordinator.external_memory_account(),source_bytes+topology_bytes+display_bytes));
        let live=shape.display_liveness();drop(source);drop(topology);drop(display);
        let source=shape.into_owned_source();
        assert!(!live.is_alive());
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::Kernel),source_bytes+topology_bytes);
        assert_eq!(source.storage_bytes().unwrap(),source_bytes+topology_bytes);drop(source);
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::Kernel),0);
    }

    #[test]
    fn split_candidate_pin_releases_source_and_rejects_foreign_owner() {
        use crate::{budget::memory::MemoryClass, transaction::{TransactionCoordinator,World}};
        let limits=Limits::default();
        let mut shape=Brep::from_native("CASCADE Topology fixture".into(),BrepDisplay {
            source_sha256:[0;32],deflection_mm:0.1,points:vec![[0.,0.,0.],[1.,0.,0.],[0.,1.,0.]],
            triangles:vec![[0,1,2,0]],segments:vec![],vertices:vec![],
            topology:BrepTopology {faces:vec![Uuid::from_u128(1)],edges:vec![],vertex_ids:vec![],face_edges:vec![]}.into(),
        },&limits).unwrap();
        let coordinator=TransactionCoordinator::new(World::new(vec![],&limits).unwrap(),limits).unwrap();
        let foreign=TransactionCoordinator::new(World::new(vec![],&limits).unwrap(),limits).unwrap();
        let source_bytes=shape.source_allocation_bytes().unwrap();let topology_bytes=shape.topology_allocation_bytes().unwrap();
        let display_bytes=shape.display_allocation_bytes().unwrap();
        let reserve=|owner:&TransactionCoordinator,bytes| {let mut r=owner.reserve_external_memory(MemoryClass::Kernel,bytes,None).unwrap();
            r.set_allocated_bytes(bytes).unwrap();Arc::new(std::sync::Mutex::new(r))};
        let source=reserve(&coordinator,source_bytes);let topology=reserve(&coordinator,topology_bytes);let wrong=reserve(&foreign,display_bytes);
        assert!(matches!(shape.bind_split_allocation_owners(source.clone(),topology.clone(),wrong.clone()),Err(Error::Invalid(_))));
        drop(wrong);assert_eq!(foreign.external_reserved_bytes(MemoryClass::Kernel),0);
        let pending=Arc::new(std::sync::Mutex::new(coordinator.reserve_external_memory(MemoryClass::Kernel,display_bytes,None).unwrap()));
        assert!(matches!(shape.bind_split_allocation_owners(source.clone(),topology.clone(),pending.clone()),Err(Error::Invalid(_))));
        drop(pending);assert_eq!(coordinator.external_pending_bytes(MemoryClass::Kernel),0);
        let display=reserve(&coordinator,display_bytes);shape.bind_split_allocation_owners(source.clone(),topology.clone(),display.clone()).unwrap();
        let pin=shape.pin_display(Arc::new(()));let live=shape.display_liveness();let source_live=shape.source_liveness();
        drop(source);drop(topology);drop(display);drop(shape);
        assert!(live.is_alive());assert!(!source_live.is_alive());
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::Kernel),topology_bytes+display_bytes);
        drop(pin);assert!(!live.is_alive());assert_eq!(coordinator.external_reserved_bytes(MemoryClass::Kernel),0);
    }

    #[test]
    fn native_constructor_rejects_display_bound_to_another_source() {
        let limits = Limits::default();
        let display = BrepDisplay {
            source_sha256: [0; 32],
            deflection_mm: 0.01,
            points: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
            triangles: vec![[0, 1, 2, 0]],
            segments: vec![],
            vertices: vec![],
            topology: crate::brep::BrepTopology {
                faces: vec![Uuid::from_u128(1)],
                edges: vec![],
                vertex_ids: vec![],
                face_edges: vec![],
            }.into(),
        };
        let shape = Brep::from_native("CASCADE Topology A".into(), display, &limits).unwrap();
        #[derive(Serialize)]
        struct LegacyDisplay<'a> {
            source_sha256: [u8; 32],
            deflection_mm: f64,
            points: &'a [Point3],
            triangles: &'a [[u32; 4]],
            segments: &'a [[u32; 3]],
            vertices: &'a [u32],
            faces: &'a [Uuid],
            edges: &'a [Uuid],
            vertex_ids: &'a [Uuid],
            face_edges: &'a [[u32; 2]],
        }
        let display = &shape.display;
        let legacy = LegacyDisplay {
            source_sha256: display.source_sha256,
            deflection_mm: display.deflection_mm,
            points: &display.points,
            triangles: &display.triangles,
            segments: &display.segments,
            vertices: &display.vertices,
            faces: &display.faces,
            edges: &display.edges,
            vertex_ids: &display.vertex_ids,
            face_edges: &display.face_edges,
        };
        let old_bytes = serde_json::to_vec(&legacy).unwrap();
        assert_eq!(serde_json::to_vec(display).unwrap(), old_bytes);
        let restored: BrepDisplay = serde_json::from_slice(&old_bytes).unwrap();
        assert_eq!(&restored, display.as_ref());
        let mut invalid = serde_json::to_value(&legacy).unwrap();
        invalid["unknown"] = serde_json::json!(1);
        assert!(serde_json::from_value::<BrepDisplay>(invalid).is_err());
        assert!(Arc::ptr_eq(&shape.canonical.topology, &shape.display.topology.0));
        let copied = shape.display().clone();
        assert_eq!(&*copied.topology, shape.topology());
        assert!(!Arc::ptr_eq(&copied.topology.0, &shape.display.topology.0), "display clone must not escape the original topology owner");
        drop(copied);
        let mut detached = shape.clone();
        detached.edit_display_for_validation(|display| display.topology = (*shape.canonical.topology).clone().into());
        assert!(detached.storage_usage(&limits).unwrap().bytes > shape.storage_usage(&limits).unwrap().bytes);
        assert!(matches!(detached.display_storage_bytes(), Err(Error::Invalid("BRep display topology owner"))));
        assert!(matches!(detached.validate(&limits), Err(Error::Invalid("BRep source/cache binding"))));
        let topology = shape.canonical.topology.clone();
        let source = shape.tracked_source_snapshot();
        let display = Arc::downgrade(&shape.display);
        drop(detached);
        assert!(Brep::from_native("CASCADE Topology A".into(), (*shape.display).clone(), &limits).is_ok());
        assert!(matches!(
            Brep::from_native("CASCADE Topology B".into(), (*shape.display).clone(), &limits),
            Err(Error::Invalid("BRep source/cache binding"))
        ));
        drop(shape);
        assert!(display.upgrade().is_none());
        assert_eq!(topology.faces, [Uuid::from_u128(1)]);
        let usage = source.validate_admitted(&limits, &mut |_, _| Ok(())).unwrap();
        assert_eq!(usage.points, 0);
        let encoded = serde_json::to_vec(&source.canonical_record()).unwrap();
        let wire: serde_json::Value = serde_json::from_slice(&encoded).unwrap();
        assert!(wire.get("display").is_none() && wire.get("triangles").is_none());
        let record: CanonicalBrepRecord = serde_json::from_slice(&encoded).unwrap();
        let record_usage = record.validate_admitted(&limits, |_, _| Ok(())).unwrap();
        assert_eq!(record_usage.points, 0);
        assert_eq!(record.source(), source.source());
        assert_eq!(record.topology(), source.topology());
        assert_eq!(record.fingerprint(), source.fingerprint());
        assert!(matches!(record.validate_admitted(&limits, |_, pending| {
            if pending > 0 { Err(Error::Budget("record scratch")) } else { Ok(()) }
        }), Err(Error::Budget("record scratch"))));
        let mut invalid_wire = wire.clone();
        invalid_wire["source"] = serde_json::json!("CASCADE Topology altered");
        let invalid: CanonicalBrepRecord = serde_json::from_value(invalid_wire).unwrap();
        assert!(matches!(invalid.validate_admitted(&limits, |_, _| Ok(())), Err(Error::Invalid("BRep source binding"))));
        let mut invalid_wire = wire.clone();
        invalid_wire["topology"]["faces"][0] = serde_json::json!(Uuid::from_u128(2));
        let invalid: CanonicalBrepRecord = serde_json::from_value(invalid_wire).unwrap();
        assert!(matches!(invalid.validate_admitted(&limits, |_, _| Ok(())), Err(Error::Invalid("BRep source/topology integrity"))));
        let mut invalid_wire = wire.clone();
        invalid_wire["topology"]["unknown"] = serde_json::json!(1);
        assert!(serde_json::from_value::<CanonicalBrepRecord>(invalid_wire).is_err());
        let mut invalid_wire = wire;
        invalid_wire["display"] = serde_json::json!({});
        assert!(serde_json::from_value::<CanonicalBrepRecord>(invalid_wire).is_err());
        assert!(matches!(source.validate_admitted(&limits, &mut |_, pending| {
            if pending > 0 { Err(Error::Budget("test scratch")) } else { Ok(()) }
        }), Err(Error::Budget("test scratch"))));
        let mut rebound_without_proof = source.clone();
        Arc::make_mut(&mut rebound_without_proof.topology).faces[0] = Uuid::from_u128(2);
        assert!(matches!(rebound_without_proof.validate_admitted(&limits, &mut |_, _| Ok(())),
            Err(Error::Invalid("BRep source/topology integrity"))));
        let encoded = serde_json::to_vec(&rebound_without_proof.canonical_record()).unwrap();
        let record: CanonicalBrepRecord = serde_json::from_slice(&encoded).unwrap();
        assert!(matches!(record.validate_admitted(&limits, |_, _| Ok(())),
            Err(Error::Invalid("BRep source/topology integrity"))));
        let mut invalid = source.clone();
        invalid.source_sha256 = [0; 32];
        assert!(matches!(invalid.validate_admitted(&limits, &mut |_, _| Ok(())), Err(Error::Invalid("BRep source binding"))));
        let mut invalid = source.clone();
        Arc::make_mut(&mut invalid.topology).faces.push(Uuid::from_u128(1));
        invalid.binding_sha256 = source_topology_fingerprint(invalid.source_sha256, &invalid.topology);
        assert!(matches!(invalid.validate_admitted(&limits, &mut |_, _| Ok(())), Err(Error::Invalid("BRep topology identity"))));
        let mut invalid = source.clone();
        Arc::make_mut(&mut invalid.topology).face_edges.push([0, 0]);
        invalid.binding_sha256 = source_topology_fingerprint(invalid.source_sha256, &invalid.topology);
        assert!(matches!(invalid.validate_admitted(&limits, &mut |_, _| Ok(())), Err(Error::Invalid("BRep boundary topology"))));
    }

    #[test]
    fn inventory_matches_validation_and_does_not_waive_scratch_budget() {
        let limits = Limits::default();
        let shape = Brep::from_native(
            "CASCADE Topology fixture".into(),
            BrepDisplay {
                source_sha256: [0; 32],
                deflection_mm: 0.01,
                points: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
                triangles: vec![[0, 1, 2, 0]],
                segments: vec![],
                vertices: vec![],
                topology: crate::brep::BrepTopology {
                    faces: vec![Uuid::from_u128(1)],
                    edges: vec![],
                    vertex_ids: vec![],
                    face_edges: vec![],
                }.into(),
            },
            &limits,
        )
        .unwrap();
        let cloned = shape.clone();
        assert_eq!(shape.source().as_ptr(), cloned.source().as_ptr(), "immutable source should be shared across clones");
        assert!(Arc::ptr_eq(&shape.display, &cloned.display));
        assert!(shape.display_identity().same(&cloned.display_identity()));
        let serialized = serde_json::to_vec(&shape).unwrap();
        #[derive(Serialize)]
        struct LegacyBrep<'a> {
            source: &'a str,
            source_sha256: [u8; 32],
            display_sha256: [u8; 32],
            display: &'a BrepDisplay,
        }
        let legacy = LegacyBrep { source: shape.source(), source_sha256: shape.canonical.source_sha256,
            display_sha256: shape.display_sha256, display: shape.display.as_ref() };
        assert_eq!(serialized, serde_json::to_vec(&legacy).unwrap(), "v4 B-rep bytes must not change");
        let packet: serde_json::Value = serde_json::from_slice(&serialized).unwrap();
        assert_eq!(packet["source"].as_str(), Some(shape.source()));
        assert_eq!(packet["display"], serde_json::to_value(shape.display.as_ref()).unwrap());
        let restored: Brep = serde_json::from_slice(&serialized).unwrap();
        assert_eq!(restored, shape);
        assert!(!restored.display_identity().same(&shape.display_identity()), "decoded allocation gets a new runtime identity");
        restored.validate(&limits).unwrap();
        let mut changed = cloned;
        changed.edit_display_for_validation(|display| display.points[0][0] = 42.);
        assert!(!Arc::ptr_eq(&shape.display, &changed.display));
        assert!(!shape.display_identity().same(&changed.display_identity()));
        assert!(changed.validate(&limits).is_err(), "modified snapshot must fail source binding");
        assert_eq!(shape.display.points[0][0], 0.);
        let usage = shape.storage_usage(&limits).unwrap();
        assert_eq!(usage, shape.validate(&limits).unwrap());
        let tight = Limits {
            max_bytes: usage.bytes,
            ..limits
        };
        assert!(shape.storage_usage(&tight).is_ok());
        assert!(matches!(shape.validate(&tight), Err(Error::Budget(_))));
    }
}
