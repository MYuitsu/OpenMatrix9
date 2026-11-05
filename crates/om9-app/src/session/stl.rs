// SPDX-License-Identifier: LGPL-2.1-or-later
//! File ▸ Export Selected as binary STL (`OM9-L3-01-12-008`, target M07).
//!
//! The document stores millimetres and editable OCCT B-Reps. Export re-tessellates each selected
//! solid through the shared kernel snapshot at the document's export tolerance, applies the entity
//! placement, resolves per-face nodes, checks indexed topology, and writes a binary STL.
//! Nothing here mutates the document: no transaction, no Undo, no selection change.
use super::*;
use om9_core::budget::Usage;
use om9_core::geometry::{Mesh, Transform};
use om9_exchange::stl::{StlOptions, TriangleSoup, write_binary_stl_to_with_report_and_admission, StlStreamError, StlWriteReport};
#[cfg(test)]
use om9_exchange::stl::write_binary_stl_to;
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}},
};

/// Feature id of the File menu `FileExportSelected` row.
pub const STL_EXPORT: &str = "OM9-FILE-001";
type StlAdmission<'a> = dyn FnMut(u64, u64) -> Result<()> + 'a;

fn stl_soup_bytes(points: &Vec<[f64; 3]>, triangles: &Vec<[u32; 3]>) -> Result<u64> {
    om9_core::budget::Usage::array::<[f64; 3]>(points.capacity())?.bytes
        .checked_add(om9_core::budget::Usage::array::<[u32; 3]>(triangles.capacity())?.bytes)
        .ok_or(Error::Overflow("STL soup capacity"))
}
fn stl_sum(a: u64, b: u64) -> Result<u64> { a.checked_add(b).ok_or(Error::Overflow("STL live memory")) }

#[cfg(feature = "occt-brep")]
fn stl_kernel_admitted<T>(admit: &mut StlAdmission<'_>, context: &'static str,
    call: impl FnOnce(&mut StlAdmission<'_>) -> std::result::Result<T, om9_kernel::KernelError>) -> Result<T> {
    let mut failure = None;
    let result = call(&mut |live, pending| match admit(live, pending) {
        Ok(()) => Ok(()),
        Err(error) => { failure = Some(error); Err(Error::Budget("STL kernel copy admission")) }
    });
    if let Some(error) = failure { return Err(error); }
    result.map_err(|error| Error::Io(format!("{context}: {error}")))
}

/// The controller owns validity; captures hold only the shared token. Dropping
/// the controller invalidates outstanding work even outside the normal UI.
pub(super) struct StlSessionOwner {
    pub(super) token: Arc<Mutex<bool>>,
}
impl StlSessionOwner {
    pub(super) fn new() -> Self { Self { token: Arc::new(Mutex::new(true)) } }
}
impl Drop for StlSessionOwner {
    fn drop(&mut self) { *self.token.lock().unwrap_or_else(|e| e.into_inner()) = false; }
}
/// Maximum weld distance in millimetres (one nanometre), further limited by
/// the document tolerance. The actual geometry error is not certified by this cap.
#[cfg(feature = "occt-brep")]
const WELD_MM: f64 = 1.0e-6;

/// Per-export linear quality request, measured in output millimetres.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StlExportProfile {
    pub linear_tolerance_mm: f64,
    pub angular_tolerance_rad: f64,
}

impl StlExportProfile {
    pub fn validate(self) -> Result<()> {
        if !self.linear_tolerance_mm.is_finite() || self.linear_tolerance_mm <= 0.0 {
            return Err(Error::Invalid("STL export tolerance"));
        }
        if !self.angular_tolerance_rad.is_finite() || self.angular_tolerance_rad <= 0.0 || self.angular_tolerance_rad > std::f64::consts::PI {
            return Err(Error::Invalid("STL angular tolerance"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StlExportReceipt {
    pub path: PathBuf,
    pub triangles: usize,
    pub bytes: usize,
    /// Requested linear tolerance, not a certified global deviation.
    pub requested_tolerance_mm: f64,
    pub requested_angle_rad: f64,
    /// Maximum placement-scaled upper bound of OCCT topology tolerance
    /// metadata across exported B-reps/surfaces; not measured mesh error.
    pub max_occt_topology_tolerance_upper_mm: Option<f64>,
    /// Identity of the captured source document.
    pub source_document_id: om9_core::identity::DocumentId,
    /// Coordinator epoch captured with the source document.
    pub source_epoch: u64,
    /// Boundary edges after welding. Other topology defects are counted separately.
    pub open_edges: usize,
    pub non_manifold_edges: usize,
    pub inconsistent_edges: usize,
    pub disconnected_vertices: usize,
    /// Source-scoped triangle components sharing indexed edges, prior to STL encoding.
    pub edge_connected_components: usize,
    pub duplicate_faces: usize,
    /// Distinct indexed input vertices encode to the same position.
    pub encoded_vertex_identity_ambiguous: bool,
    /// Selected objects that carry no triangles (points, curves, unsupported geometry).
    pub skipped: Vec<String>,
    /// Honest notes such as a retained native display mesh or an open surface.
    pub warnings: Vec<String>,
}

/// Immutable source and selection retained independently of later document edits.
pub struct StlExportCapture {
    document: om9_core::transaction::CapturedDocument,
    selection: Vec<om9_core::identity::ObjectId>,
    _selection_reservation: om9_core::transaction::ExternalMemoryReservation,
    limits: Limits,
    epoch: u64,
    account: om9_core::transaction::ExternalMemoryAccount,
    session: Arc<Mutex<bool>>,
    /// Empty peer shares the live retained-document registry and reservations.
    admission_peer: om9_core::transaction::TransactionCoordinator,
    /// Legacy unregistered auxiliary and structural allowance at capture.
    /// Scene bytes belong to their live owners, not to this estimate. Other
    /// changing auxiliary still needs live ownership before closing R1a.
    caller_held_at_capture: om9_core::budget::Usage,
}

/// Owns one background export and requests cancellation on drop. The caller
/// must obtain a sufficient reservation from the shared budget before launch,
/// admitted with a physical-RAM sample. Sampling and the supplied reservation
/// do not themselves prove an upper bound on native or total worker memory.
pub struct StlExportJob {
    cancellation: StlCancellation,
    result: Option<std::sync::mpsc::Receiver<Result<StlExportReceipt>>>,
}

impl StlExportJob {
    pub fn start(
        capture: StlExportCapture,
        target: PathBuf,
        profile: StlExportProfile,
        reservation: om9_core::transaction::ExternalMemoryReservation,
    ) -> Result<Self> {
        Self::start_with_sample(capture, target, profile, reservation, || {
            om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes))
        })
    }

    fn start_with_sample(
        mut capture: StlExportCapture,
        target: PathBuf,
        profile: StlExportProfile,
        reservation: om9_core::transaction::ExternalMemoryReservation,
        sample: impl FnOnce() -> Option<(u64, u64)>,
    ) -> Result<Self> {
        profile.validate()?;
        if !*capture.session.lock().unwrap_or_else(|e| e.into_inner()) {
            return Err(Error::Canceled);
        }
        if reservation.class() != om9_core::budget::memory::MemoryClass::Export || reservation.bytes() == 0 || !reservation.belongs_to(&capture.account) {
            return Err(Error::Invalid("STL export reservation"));
        }
        let target = stl_target(&target)?;
        let permit = capture.account.try_acquire_export()?;
        if !reservation.physical_sampled() {
            return Err(Error::Invalid("STL export requires sampled physical RAM"));
        }
        let (total, available) = sample().ok_or(Error::Unsupported("physical RAM availability is unavailable"))?;
        reservation.recheck_physical(total, available)?;
        // The native snapshot adapter must not admit an owned snapshot larger
        // than the worker's reservation. This is a necessary bound only: OCCT
        // transients and other concurrently live export scratch are separate.
        let reservation_bytes = reservation.bytes();
        capture.limits.max_bytes = capture.limits.max_bytes.min(reservation_bytes);
        Self::spawn(reservation, permit, move |cancel| {
            let cancel = cancel.with_session(capture.session.clone());
            if reservation_bytes < capture.authored_mesh_array_lower_bound(&cancel)? {
                return Err(Error::Budget("STL mesh reservation lower bound"));
            }
            capture.export_admitted(&target, profile, &cancel)
        })
    }

    fn spawn(
        reservation: om9_core::transaction::ExternalMemoryReservation,
        permit: om9_core::transaction::ExternalExportPermit,
        work: impl FnOnce(&StlCancellation) -> Result<StlExportReceipt> + Send + 'static,
    ) -> Result<Self> {
        let cancellation = StlCancellation::default();
        let worker_cancel = cancellation.clone();
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        std::thread::Builder::new().name("om9-stl-export".into()).spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| work(&worker_cancel)))
                .unwrap_or_else(|_| Err(Error::Io("STL worker panicked".into())));
            drop(reservation);
            drop(permit);
            let _ = sender.send(result);
        }).map_err(|error| Error::Io(format!("STL worker start: {error}")))?;
        Ok(Self { cancellation, result: Some(receiver) })
    }

    pub fn cancel(&self) -> bool { self.cancellation.cancel() }

    /// Nonblocking. A completed result is returned only once.
    pub fn try_result(&mut self) -> Option<Result<StlExportReceipt>> {
        let receiver = self.result.as_ref()?;
        match receiver.try_recv() {
            Ok(result) => { self.result = None; Some(result) }
            Err(std::sync::mpsc::TryRecvError::Empty) => None,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.result = None;
                Some(Err(Error::Io("STL worker exited without a result".into())))
            }
        }
    }
}

impl Drop for StlExportJob {
    fn drop(&mut self) { self.cancellation.cancel(); }
}

/// Cancellation is serialized with final publication. A successful cancellation
/// guarantees that this export will not subsequently replace the destination.
#[derive(Clone, Default)]
pub struct StlCancellation {
    state: Arc<StlCancellationState>,
    session: Option<Arc<Mutex<bool>>>,
}
#[derive(Default)]
struct StlCancellationState {
    canceled: AtomicBool,
    published: Mutex<bool>,
}
impl StlCancellation {
    pub fn cancel(&self) -> bool {
        let published = self.state.published.lock().unwrap_or_else(|e| e.into_inner());
        if *published { return false; }
        self.state.canceled.store(true, Ordering::Release);
        true
    }
    fn with_session(&self, session: Arc<Mutex<bool>>) -> Self {
        Self { state: self.state.clone(), session: Some(session) }
    }
    pub fn is_canceled(&self) -> bool {
        self.state.canceled.load(Ordering::Acquire)
            || self.session.as_ref().is_some_and(|session| !*session.lock().unwrap_or_else(|e| e.into_inner()))
    }
    fn check(&self) -> Result<()> { if self.is_canceled() { Err(Error::Canceled) } else { Ok(()) } }
}

impl StlExportReceipt {
    /// Checks the indexed input soup; STL does not retain source topology and this does not check self-intersections.
    pub fn watertight(&self) -> bool {
        self.open_edges == 0 && self.non_manifold_edges == 0 && self.inconsistent_edges == 0 && self.disconnected_vertices == 0 && self.duplicate_faces == 0
    }

    /// Closed under edge, orientation, vertex-fan and duplicate checks after
    /// exact-coordinate indexing of the encoded triangles. Unknown when two
    /// distinct input indices encode to one position; self-intersection and
    /// shell containment are never checked here.
    pub fn encoded_closed_by_topology_checks(&self) -> Option<bool> {
        (!self.encoded_vertex_identity_ambiguous).then(|| self.watertight())
    }

    pub fn message(&self) -> String {
        let mut text = format!(
            "Exported STL: {} ({} triangles, {} bytes, assumed millimetres, {})",
            crate::transcript::redact_paths(&self.path.display().to_string()),
            self.triangles,
            self.bytes,
            if self.watertight() {
                "closed by source-scoped indexed topology checks".to_owned()
            } else {
                format!("{} boundary, {} non-manifold, {} inconsistent edges, {} disconnected vertices, {} duplicate faces", self.open_edges, self.non_manifold_edges, self.inconsistent_edges, self.disconnected_vertices, self.duplicate_faces)
            }
        );
        text.push_str(&format!("; requested tolerance {} mm, angle {} rad; global deviation unverified; self-intersection unchecked; STL has no unit metadata", self.requested_tolerance_mm, self.requested_angle_rad));
        text.push_str(&format!("; {} source-scoped edge-connected triangle components", self.edge_connected_components));
        if self.encoded_vertex_identity_ambiguous {
            text.push_str("; distinct indexed vertices share encoded positions; STL cannot preserve their topology identity");
        }
        match self.encoded_closed_by_topology_checks() {
            Some(true) => text.push_str("; encoded coordinate-indexed edge/fan checks: closed"),
            Some(false) => text.push_str("; encoded coordinate-indexed edge/fan checks: not closed or invalid"),
            None => text.push_str("; encoded coordinate-indexed edge/fan checks: unknown"),
        }
        if !self.watertight() { text.push_str("; source-scoped indexed topology checks"); }
        if !self.skipped.is_empty() {
            text.push_str(&format!("; skipped {}", self.skipped.join(", ")));
        }
        for warning in &self.warnings {
            text.push_str(&format!("; {warning}"));
        }
        text
    }
}

/// Expand selected definition occurrences using composed placements. Each
/// occurrence remains distinct even when it shares the same definition.
fn stl_occurrences<'a>(
    document: &'a om9_core::model::Document,
    selection: impl Iterator<Item = om9_core::identity::ObjectId>,
    limits: &Limits,
    cancel: &StlCancellation,
    mut handle: impl FnMut(&'a om9_core::model::Entity, Transform, bool) -> Result<()>,
) -> Result<()> {
    struct Ancestor<'a> {
        id: om9_core::identity::DefinitionId,
        parent: Option<&'a Ancestor<'a>>,
    }
    fn visit<'a>(document: &'a om9_core::model::Document, entity: &'a om9_core::model::Entity,
        parent: Transform, parent_visible: bool, limits: &Limits, cancel: &StlCancellation,
        ancestors: Option<&Ancestor<'_>>, depth: usize, count: &mut u64,
        handle: &mut impl FnMut(&'a om9_core::model::Entity, Transform, bool) -> Result<()>) -> Result<()> {
        cancel.check()?;
        *count = count.checked_add(1).ok_or(Error::Overflow("STL occurrences"))?;
        if *count > limits.max_nodes { return Err(Error::Budget("STL occurrences")); }
        let placement = parent.compose(entity.placement)?;
        let visible = parent_visible && document.permissions_for_entity(entity, true)?.visible;
        if let Geometry::Instance(id) = entity.geometry {
            if depth >= limits.max_depth { return Err(Error::Budget("STL instance depth")); }
            let mut cursor = ancestors;
            while let Some(ancestor) = cursor {
                if ancestor.id == id { return Err(Error::Invalid("STL instance cycle")); }
                cursor = ancestor.parent;
            }
            let definition = document.definitions.get(&id).ok_or(Error::Missing("STL instance definition"))?;
            let ancestor = Ancestor { id, parent: ancestors };
            for member in &definition.members { visit(document, member, placement, visible, limits, cancel, Some(&ancestor), depth + 1, count, handle)?; }
        } else {
            handle(entity, placement, visible)?;
        }
        Ok(())
    }
    let mut count = 0;
    for id in selection {
        let entity = document.entities.get(&id).ok_or(Error::Stale("STL source"))?;
        visit(document, entity, Transform::identity(), true, limits, cancel, None, 0, &mut count, &mut handle)?;
    }
    Ok(())
}

impl Controller {
    /// A close request cancels the current export generation. If the user
    /// cancels closing, later exports use the new live generation.
    pub(crate) fn invalidate_stl_session_for_close(&mut self) {
        let previous = std::mem::replace(&mut self.stl_session, StlSessionOwner::new());
        drop(previous);
    }

    /// Files a typed name or the menu feature id reaches.
    pub fn stl_export_feature(text: &str) -> Option<&'static str> {
        let text = text.trim();
        if text.eq_ignore_ascii_case("exportstl")
            || text.eq_ignore_ascii_case("export stl")
            || text.eq_ignore_ascii_case("export selected")
            || text.eq_ignore_ascii_case("filesexportselected")
            || text.eq_ignore_ascii_case("FileExportSelected")
        {
            Some(STL_EXPORT)
        } else {
            None
        }
    }

    /// Whether the current selection has at least one supported triangle source.
    pub fn stl_export_ready(&self) -> Result<()> {
        if self.selection.is_empty() {
            return Err(Error::Missing(
                "select meshes, B-Reps or surfaces to export",
            ));
        }
        let mut supported = false;
        stl_occurrences(self.document(), self.selection.iter().copied(), self.coordinator.limits(), &StlCancellation::default(), |entity, _, _| {
                supported |= matches!(
                    entity.geometry,
                    Geometry::Mesh(_)
                        | Geometry::Brep(_)
                        | Geometry::NativeBrep(_)
                        | Geometry::Surface(_)
                );
                Ok(())
        })?;
        if supported {
            Ok(())
        } else {
            Err(Error::Missing(
                "selected objects have no triangles: meshes, B-Reps, retained native BReps and surfaces export",
            ))
        }
    }

    /// Export the selected meshes, editable/retained B-Reps and surfaces to one binary STL in
    /// millimetres. The file replaces `target` atomically; a failed export leaves the destination
    /// untouched. An open set (a surface, or an open mesh) is written with its open-edge count in
    /// the receipt instead of being refused, because a surface has no closed volume to claim.
    pub fn export_stl_selected(&self, target: &Path) -> Result<StlExportReceipt> {
        let profile = StlExportProfile { linear_tolerance_mm: self.document().effective_tolerance()?.absolute_mm, angular_tolerance_rad: 0.3 };
        self.export_stl_selected_with_profile(target, profile)
    }

    pub fn export_stl_selected_with_profile(&self, target: &Path, profile: StlExportProfile) -> Result<StlExportReceipt> {
        self.export_captured_stl(self.capture_stl_selected()?, target, profile)
    }

    /// Use the same phase admission as detached export. OCCT internal native
    /// allocations remain outside the owned-copy callbacks.
    pub fn export_captured_stl(&self, capture: StlExportCapture, target: &Path, profile: StlExportProfile) -> Result<StlExportReceipt> {
        self.export_captured_stl_with_sample(capture, target, profile,
            || om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes)))
    }

    fn export_captured_stl_with_sample(
        &self, capture: StlExportCapture, target: &Path, profile: StlExportProfile,
        sample: impl FnOnce() -> Option<(u64, u64)>,
    ) -> Result<StlExportReceipt> {
        profile.validate()?;
        if !capture.belongs_to_session(self.document()) {
            return Err(Error::Stale("STL export document session"));
        }
        if !capture._selection_reservation.belongs_to(&self.coordinator.external_memory_account()) {
            return Err(Error::Invalid("STL source account"));
        }
        capture.export_staged(target, profile, &StlCancellation::default(), sample,
            || self.scene_cache_usage(), |_| Ok(()))
    }

    /// Hold the current immutable document allocation and selected object IDs.
    pub fn capture_stl_selected(&self) -> Result<StlExportCapture> {
        self.capture_stl_selected_with_sample(|| {
            om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes))
        })
    }

    fn capture_stl_selected_with_sample(&self, mut sample: impl FnMut() -> Option<(u64, u64)>) -> Result<StlExportCapture> {
        self.stl_export_ready()?;
        let limits = *self.coordinator.limits();
        if self.selection.len() as u64 > limits.max_outputs {
            return Err(Error::Budget("STL source selection"));
        }
        let scene = self.scene.as_ref().ok_or(Error::Missing("STL capture scene"))?;
        if scene.cache_held_usage()?.bytes != self.registered_scene_buffer_bytes(scene) {
            return Err(Error::Invalid("STL scene ownership"));
        }
        let selection_bytes = om9_core::budget::Usage::array::<om9_core::identity::ObjectId>(self.selection.len())?.bytes;
        let held = self.scene_cache_usage()?;
        let mut selection_reservation = self.coordinator.reserve_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::Export, selection_bytes.max(1), sample(), held,
        )?;
        let mut selection = Vec::new();
        selection.try_reserve_exact(self.selection.len()).map_err(|_| Error::Budget("STL source selection"))?;
        let actual = om9_core::budget::Usage::array::<om9_core::identity::ObjectId>(selection.capacity())?.bytes;
        if actual > selection_reservation.bytes() {
            let additional = actual - selection_reservation.bytes();
            self.coordinator.grow_external_memory_with_held(&mut selection_reservation, additional, sample(), held)?;
        }
        selection.extend(self.selection.iter().copied());
        selection_reservation.mark_allocated(actual)?;
        let document = self.coordinator.capture_tracked_document(self.document)?;
        let admission_peer = self.coordinator.memory_admission_peer()?;
        Ok(StlExportCapture { document, selection, _selection_reservation: selection_reservation, limits, epoch: self.coordinator.epoch(), account: self.coordinator.external_memory_account(), session: self.stl_session.token.clone(), admission_peer, caller_held_at_capture: held })
    }
}

impl StlExportCapture {
    pub fn source_epoch(&self) -> u64 { self.epoch }
    pub fn document_id(&self) -> om9_core::identity::DocumentId { self.document.id }
    /// Normal edits retain the session; New/Open renew it even when the same
    /// persistent document UUID is reopened.
    pub fn belongs_to_session(&self, document: &om9_core::model::Document) -> bool {
        *self.session.lock().unwrap_or_else(|e| e.into_inner())
            && self.document.id == document.id
            && self.document.owner_session == document.owner_session
            && self.document.session_generation == document.session_generation
    }

    /// Known minimum for authored-mesh soup and the coexisting topology
    /// union-find arrays. Hash tables, native mesh, capacity growth, writer and
    /// worker allocations are excluded; this is not a sufficient reservation.
    fn authored_mesh_array_lower_bound(&self, cancel: &StlCancellation) -> Result<u64> {
        use om9_core::budget::Usage;
        let mut bytes = 0u64;
        stl_occurrences(&self.document, self.selection.iter().copied(), &self.limits, cancel, |entity, _, _| {
            if let Geometry::Mesh(mesh) = &entity.geometry {
                let triangles = authored_mesh_triangle_count(mesh, || cancel.check())?;
                for usage in [Usage::array::<[f64; 3]>(mesh.vertices.len())?, Usage::array::<[u32; 3]>(triangles)?, Usage::array::<usize>(triangles)?, Usage::array::<u8>(triangles)?] {
                    bytes = bytes.checked_add(usage.bytes).ok_or(Error::Overflow("STL mesh reservation lower bound"))?;
                }
            }
            Ok(())
        })?;
        Ok(bytes)
    }

    pub fn export_with_profile(&self, target: &Path, profile: StlExportProfile) -> Result<StlExportReceipt> {
        self.export_with_cancellation(target, profile, &StlCancellation::default())
    }

    pub fn export_with_cancellation(&self, target: &Path, profile: StlExportProfile, cancel: &StlCancellation) -> Result<StlExportReceipt> {
        self.export_with_sample(target, profile, cancel, || {
            om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes))
        })
    }

    fn export_with_sample(&self, target: &Path, profile: StlExportProfile, cancel: &StlCancellation,
        sample: impl FnOnce() -> Option<(u64, u64)>) -> Result<StlExportReceipt> {
        self.export_with_sample_guard(target, profile, cancel, sample, |_| Ok(()))
    }

    fn export_with_sample_guard(&self, target: &Path, profile: StlExportProfile, cancel: &StlCancellation,
        sample: impl FnOnce() -> Option<(u64, u64)>,
        admitted: impl FnOnce(&om9_core::transaction::ExternalMemoryReservation) -> Result<()>) -> Result<StlExportReceipt> {
        self.export_staged(target, profile, cancel, sample, || Ok(self.caller_held_at_capture), admitted)
    }

    fn export_staged(&self, target: &Path, profile: StlExportProfile, cancel: &StlCancellation,
        sample: impl FnOnce() -> Option<(u64, u64)>, held: impl Fn() -> Result<Usage>,
        admitted: impl FnOnce(&om9_core::transaction::ExternalMemoryReservation) -> Result<()>) -> Result<StlExportReceipt> {
        profile.validate()?;
        let cancel = cancel.with_session(self.session.clone());
        cancel.check()?;
        let _permit = self.account.try_acquire_export()?;
        let physical = sample().ok_or(Error::Unsupported("physical RAM availability is unavailable"))?;
        let initial = self.authored_mesh_array_lower_bound(&cancel)?.max(1);
        let mut reservation = self.admission_peer.reserve_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::Export, initial, Some(physical), held()?,
        )?;
        admitted(&reservation)?;
        self.export_with_admission(target, profile, &cancel, &self.limits, &mut |allocated, pending| {
            cancel.check()?;
            let requested = stl_sum(allocated, pending)?;
            if pending != 0 && requested <= reservation.bytes()
                && reservation.bytes() - reservation.pending_bytes() == allocated { return Ok(()); }
            reservation.set_allocated_bytes(allocated.min(reservation.bytes()))?;
            if requested > reservation.bytes() {
                let additional = requested - reservation.bytes();
                let batch = additional.max(64 * 1024);
                let physical = om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
                let held = held()?;
                match self.admission_peer.grow_external_memory_with_held(&mut reservation, batch, physical, held) {
                    Err(Error::Budget(_)) if batch > additional => self.admission_peer.grow_external_memory_with_held(
                        &mut reservation, additional, physical, held)?,
                    other => other?,
                }
            } else { reservation.shrink_to(requested)?; }
            reservation.set_allocated_bytes(allocated)?;
            if pending != 0 { reservation.recheck_pending()?; }
            Ok(())
        })
    }

    /// Caller owns the export permit; worker and controller routes also hold
    /// their managed-memory reservation around this operation.
    fn export_admitted(&self, target: &Path, profile: StlExportProfile, cancel: &StlCancellation) -> Result<StlExportReceipt> {
        self.export_admitted_with_limits(target, profile, cancel, &self.limits)
    }

    fn export_admitted_with_limits(&self, target: &Path, profile: StlExportProfile, cancel: &StlCancellation, limits: &Limits) -> Result<StlExportReceipt> {
        self.export_with_admission(target, profile, cancel, limits, &mut |_, _| Ok(()))
    }
    fn export_with_admission(&self, target: &Path, profile: StlExportProfile, cancel: &StlCancellation, limits: &Limits,
        admit: &mut StlAdmission<'_>) -> Result<StlExportReceipt> {
        profile.validate()?;
        cancel.check()?;
        let (soup, skipped, warnings, max_occt_tolerance) = self.stl_soup_admitted(profile, cancel, limits, admit)?;
        cancel.check()?;
        let stage_limits = limits_after_stl_notes(limits, &skipped, &warnings)?;
        let notes = limits.max_bytes - stage_limits.max_bytes;
        let topology = validate_stl_topology_admitted(&soup, &stage_limits, cancel,
            &mut |live, pending| admit(stl_sum(live, notes)?, pending))?;
        admit(stl_sum(stl_soup_bytes(&soup.positions, &soup.triangles)?, notes)?, 0)?;
        cancel.check()?;
        let open_edges = topology.boundary;
        let options = StlOptions {
            // Topology was already checked above and is reported in the receipt.
            // Avoid building its large maps a second time inside the writer.
            strict: false,
            ..StlOptions::default()
        };
        let max_quantization = profile.linear_tolerance_mm / 10.0;
        let triangles = soup.triangles.len();
        let bytes = triangles.checked_mul(50).and_then(|n| n.checked_add(84)).ok_or(Error::Overflow("STL file size"))?;
        let path = stl_target(target)?;
        let writer_limits = stage_limits.remaining_after(om9_core::budget::Usage { bytes: path.capacity() as u64, ..Default::default() })?;
        let prefix = stl_sum(notes, path.capacity() as u64)?;
        admit(stl_sum(stl_soup_bytes(&soup.positions, &soup.triangles)?, prefix)?, 0)?;
        let encoded = write_stl_atomic_admitted(&path, &soup, options, max_quantization, &writer_limits, cancel,
            &mut |live, pending| admit(stl_sum(live, prefix)?, pending))?;
        Ok(StlExportReceipt {
            path,
            triangles,
            bytes,
            requested_tolerance_mm: profile.linear_tolerance_mm,
            requested_angle_rad: profile.angular_tolerance_rad,
            max_occt_topology_tolerance_upper_mm: max_occt_tolerance,
            source_document_id: self.document.id,
            source_epoch: self.epoch,
            open_edges,
            non_manifold_edges: topology.non_manifold,
            inconsistent_edges: topology.inconsistent_winding,
            disconnected_vertices: topology.disconnected_vertices,
            edge_connected_components: topology.edge_connected_components,
            duplicate_faces: topology.duplicate_faces,
            encoded_vertex_identity_ambiguous: encoded.coincident_distinct_indices,
            skipped,
            warnings,
        })
    }

    #[cfg(test)]
    fn stl_soup(&self, profile: StlExportProfile, cancel: &StlCancellation) -> Result<(TriangleSoup, Vec<String>, Vec<String>, Option<f64>)> {
        self.stl_soup_with_limits(profile, cancel, &self.limits)
    }

    #[cfg(test)]
    fn stl_soup_with_limits(&self, profile: StlExportProfile, cancel: &StlCancellation, limits: &Limits) -> Result<(TriangleSoup, Vec<String>, Vec<String>, Option<f64>)> {
        self.stl_soup_admitted(profile, cancel, limits, &mut |_, _| Ok(()))
    }
    fn stl_soup_admitted(&self, profile: StlExportProfile, cancel: &StlCancellation, limits: &Limits,
        admit: &mut StlAdmission<'_>) -> Result<(TriangleSoup, Vec<String>, Vec<String>, Option<f64>)> {
        if self.selection.is_empty() {
            return Err(Error::Missing(
                "select meshes, B-Reps or surfaces to export",
            ));
        }
        #[cfg(feature = "occt-brep")]
        let export_tolerance = profile.linear_tolerance_mm;
        #[cfg(not(feature = "occt-brep"))]
        let _ = profile;
        let mut points: Vec<[f64; 3]> = Vec::new();
        let mut triangles: Vec<[u32; 3]> = Vec::new();
        let mut skipped = Vec::new();
        let mut warnings = Vec::new();
        #[cfg(feature = "occt-brep")]
        let mut max_occt_tolerance: Option<f64> = None;
        #[cfg(not(feature = "occt-brep"))]
        let max_occt_tolerance = None;
        stl_occurrences(&self.document, self.selection.iter().copied(), limits, cancel, |entity, placement, visible| {
            cancel.check()?;
            let triangle_start = triangles.len();
            // Existing receipt strings stay live while the next source is built.
            let geometry_limits = limits_after_stl_notes(limits, &skipped, &warnings)?;
            let notes = limits.max_bytes - geometry_limits.max_bytes;
            let mut geometry_admit = |live, pending| admit(stl_sum(live, notes)?, pending);
            match &entity.geometry {
                Geometry::Mesh(mesh) => {
                    append_mesh_admitted(&placement, mesh, &mut points, &mut triangles, &geometry_limits, cancel, &mut geometry_admit)?;
                }
                Geometry::Brep(brep) => {
                    #[cfg(feature = "occt-brep")]
                    {
                        let source_tolerance = append_brep_source_admitted(&placement, brep.source(), &mut points, &mut triangles, export_tolerance, profile.angular_tolerance_rad, &geometry_limits, cancel, &mut geometry_admit)?;
                        max_occt_tolerance = Some(max_occt_tolerance.map_or(source_tolerance.1, |current| current.max(source_tolerance.1)));
                        push_source_tolerance_note(&mut warnings, &skipped, &entity.label, source_tolerance, export_tolerance, &points, &triangles, limits, admit)?;
                        push_stl_note_admitted(&mut warnings, &skipped, format_args!("{}: requested OCCT deflection; global deviation unverified", entity.label), &points, &triangles, limits, admit)?;
                    }
                    #[cfg(not(feature = "occt-brep"))]
                    {
                        let _ = brep;
                        return Err(Error::Unsupported("STL export of B-Rep requires OCCT"));
                    }
                }
                Geometry::NativeBrep(native) => {
                    #[cfg(feature = "occt-brep")]
                    {
                        // The payload is the authority; never export the display mesh as
                        // replacement geometry. Re-import the ON_Brep payload through the
                        // admitted OCCT bridge, then use that canonical tessellation.
                        let remaining = limits_after_stl_soup(&geometry_limits, &points, &triangles)?;
                        let soup_bytes = stl_soup_bytes(&points, &triangles)?;
                        let source = stl_kernel_admitted(&mut |live, pending| geometry_admit(stl_sum(live, soup_bytes)?, pending),
                            "STL native B-Rep payload", |admit| om9_kernel::brep::from_3dm_source_admitted(
                                &native.payload, self.document.effective_tolerance().map_err(|e| om9_kernel::KernelError::InvalidInput(e.to_string()))?.absolute_mm,
                                &remaining, admit))?;
                        let source_limits = geometry_limits.remaining_after(om9_core::budget::Usage {
                            bytes: source.capacity() as u64, ..Default::default()
                        })?;
                        let source_tolerance = append_brep_source_admitted(&placement, &source, &mut points, &mut triangles,
                            export_tolerance, profile.angular_tolerance_rad, &source_limits, cancel,
                            &mut |live, pending| geometry_admit(stl_sum(live, source.capacity() as u64)?, pending))?;
                        // The converted source is no longer used; release it before
                        // allocating receipt notes while the resulting soup is live.
                        drop(source);
                        max_occt_tolerance = Some(max_occt_tolerance.map_or(source_tolerance.1, |current| current.max(source_tolerance.1)));
                        push_source_tolerance_note(&mut warnings, &skipped, &entity.label, source_tolerance, export_tolerance, &points, &triangles, limits, admit)?;
                        push_stl_note_admitted(&mut warnings, &skipped, format_args!("{}: retained native payload; requested OCCT deflection, global deviation unverified", entity.label), &points, &triangles, limits, admit)?;
                    }
                    #[cfg(not(feature = "occt-brep"))]
                    {
                        let _ = native;
                        return Err(Error::Unsupported(
                            "STL export of a retained native B-Rep requires OCCT",
                        ));
                    }
                }
                Geometry::Surface(surface) => {
                    #[cfg(feature = "occt-brep")]
                    {
                        let remaining = limits_after_stl_soup(&geometry_limits, &points, &triangles)?;
                        let soup_bytes = stl_soup_bytes(&points, &triangles)?;
                        let source = stl_kernel_admitted(&mut |live, pending| geometry_admit(stl_sum(live, soup_bytes)?, pending),
                            "STL NURBS conversion", |admit| om9_kernel::measurement::surface_to_face_source_admitted(
                                surface, Transform::identity(), &remaining, admit))?;
                        let source_limits = geometry_limits.remaining_after(om9_core::budget::Usage {
                            bytes: source.capacity() as u64, ..Default::default()
                        })?;
                        let source_tolerance = append_brep_source_admitted(&placement, &source, &mut points, &mut triangles,
                            export_tolerance, profile.angular_tolerance_rad, &source_limits, cancel,
                            &mut |live, pending| geometry_admit(stl_sum(live, source.capacity() as u64)?, pending))?;
                        drop(source);
                        max_occt_tolerance = Some(max_occt_tolerance.map_or(source_tolerance.1, |current| current.max(source_tolerance.1)));
                        push_source_tolerance_note(&mut warnings, &skipped, &entity.label, source_tolerance, export_tolerance, &points, &triangles, limits, admit)?;
                        if surface.reversed {
                            for triangle in &mut triangles[triangle_start..] { triangle.swap(1, 2); }
                        }
                        push_stl_note_admitted(&mut warnings, &skipped, format_args!("{}: open surface; requested OCCT deflection, global deviation unverified", entity.label), &points, &triangles, limits, admit)?;
                    }
                    #[cfg(not(feature = "occt-brep"))]
                    {
                        let _ = surface;
                        return Err(Error::Unsupported("STL export of a NURBS surface requires OCCT"));
                    }
                }
                other => push_stl_note_admitted(&mut skipped, &warnings, format_args!("{} ({})", entity.label, other.kind().tag()), &points, &triangles, limits, admit)?,
            }
            if !visible && triangles.len() > triangle_start {
                push_stl_note_admitted(&mut warnings, &skipped, format_args!("{}: hidden or invisible selected geometry included in STL", entity.label), &points, &triangles, limits, admit)?;
            }
            // A reflected placement reverses orientation in world space. Repair
            // each source independently, including open surfaces and native B-reps.
            let m = placement.0;
            let determinant = m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
                - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
                + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
            if determinant < 0.0 {
                for triangle in &mut triangles[triangle_start..] {
                    triangle.swap(1, 2);
                }
            }
            mark_collapses(points.len(), &mut triangles[triangle_start..], !matches!(&entity.geometry, Geometry::Mesh(_)), cancel)?;
            let notes = limits.max_bytes - limits_after_stl_notes(limits, &skipped, &warnings)?.max_bytes;
            admit(stl_sum(stl_soup_bytes(&points, &triangles)?, notes)?, 0)?;
            Ok(())
        })?;
        if triangles.is_empty() {
            return Err(Error::Missing(if skipped.is_empty() {
                "selected objects have no triangles"
            } else {
                "selected objects have no triangles: points and curves do not export"
            }));
        }
        if points.len() as u64 > limits.max_points {
            return Err(Error::Budget("STL vertices"));
        }
        cancel.check()?;
        let (soup, omitted) = finalize_soup(points, triangles, cancel)?;
        if omitted > 0 {
            push_stl_note_admitted(&mut warnings, &skipped, format_args!("{omitted} exactly collapsed generated triangles omitted"), &soup.positions, &soup.triangles, limits, admit)?;
        }
        Ok((soup, skipped, warnings, max_occt_tolerance))
    }
}

fn authored_mesh_triangle_count(mesh: &Mesh, mut check: impl FnMut() -> Result<()>) -> Result<usize> {
    mesh.faces.iter().enumerate().try_fold(0usize, |count, (index, face)| {
        if index % 1024 == 0 { check()?; }
        let additional = match face.len() {
            3 => 1,
            4 => 2,
            _ => return Err(Error::Invalid("STL mesh face count")),
        };
        count.checked_add(additional).ok_or(Error::Overflow("STL mesh reservation triangles"))
    })
}

/// Count the formatted bytes without first allocating the string.
struct StlNoteLength(usize);
impl std::fmt::Write for StlNoteLength {
    fn write_str(&mut self, text: &str) -> std::fmt::Result {
        self.0 = self.0.checked_add(text.len()).ok_or(std::fmt::Error)?;
        Ok(())
    }
}

#[cfg(test)]
fn push_stl_note(
    notes: &mut Vec<String>, other: &Vec<String>, args: std::fmt::Arguments<'_>,
    points: &Vec<[f64; 3]>, triangles: &Vec<[u32; 3]>, limits: &Limits,
) -> Result<()> {
    push_stl_note_admitted(notes, other, args, points, triangles, limits, &mut |_, _| Ok(()))
}
fn push_stl_note_admitted(
    notes: &mut Vec<String>, other: &Vec<String>, args: std::fmt::Arguments<'_>,
    points: &Vec<[f64; 3]>, triangles: &Vec<[u32; 3]>, limits: &Limits,
    admit: &mut StlAdmission<'_>,
) -> Result<()> {
    use om9_core::budget::Usage;
    let mut length = StlNoteLength(0);
    std::fmt::write(&mut length, args).map_err(|_| Error::Overflow("STL note length"))?;
    let remaining = limits_after_stl_notes(limits, notes, other)?;
    let mut held = Usage::array::<[f64; 3]>(points.capacity())?;
    held.add(Usage::array::<[u32; 3]>(triangles.capacity())?, &remaining)?;
    if notes.len() == notes.capacity() {
        let requested = stl_growth_capacity(notes.capacity(), notes.len().checked_add(1).ok_or(Error::Overflow("STL notes"))?);
        held.add(Usage::array::<String>(requested)?, &remaining)?;
        let live = stl_sum(stl_soup_bytes(points, triangles)?, limits.max_bytes - remaining.max_bytes)?;
        admit(live, Usage::array::<String>(requested)?.bytes)?;
        notes.try_reserve_exact(requested - notes.len()).map_err(|_| Error::Budget("STL notes"))?;
    }
    let remaining = limits_after_stl_notes(limits, notes, other)?;
    let mut held = Usage::array::<[f64; 3]>(points.capacity())?;
    held.add(Usage::array::<[u32; 3]>(triangles.capacity())?, &remaining)?;
    held.add(Usage { bytes: length.0 as u64, ..Usage::default() }, &remaining)?;
    let live = stl_sum(stl_soup_bytes(points, triangles)?, limits.max_bytes - remaining.max_bytes)?;
    admit(live, length.0 as u64)?;
    let mut text = String::new();
    text.try_reserve_exact(length.0).map_err(|_| Error::Budget("STL note"))?;
    let mut held = Usage::array::<[f64; 3]>(points.capacity())?;
    held.add(Usage::array::<[u32; 3]>(triangles.capacity())?, &remaining)?;
    held.add(Usage { bytes: text.capacity() as u64, ..Usage::default() }, &remaining)?;
    std::fmt::write(&mut text, args).map_err(|_| Error::Invalid("STL note formatting"))?;
    notes.push(text);
    let notes_bytes = limits.max_bytes - limits_after_stl_notes(limits, notes, other)?.max_bytes;
    admit(stl_sum(stl_soup_bytes(points, triangles)?, notes_bytes)?, 0)?;
    Ok(())
}

#[cfg(feature = "occt-brep")]
fn push_source_tolerance_note(
    warnings: &mut Vec<String>, skipped: &Vec<String>, label: &str,
    (source, upper): (f64, f64), requested: f64,
    points: &Vec<[f64; 3]>, triangles: &Vec<[u32; 3]>, limits: &Limits, admit: &mut StlAdmission<'_>,
) -> Result<()> {
    push_stl_note_admitted(warnings, skipped,
        format_args!("{label}: OCCT maximum face/edge/vertex tolerance {source} source units; placement-scaled upper bound {upper} mm{}; metadata is not measured mesh deviation",
            if upper > requested { " exceeds requested STL tolerance" } else { "" }),
        points, triangles, limits, admit)
}

/// Admit known soup and receipt storage before growing a skipped-object note.
/// Allocator metadata and other live export owners remain outside this estimate.
#[cfg(test)]
fn push_skipped_note(
    skipped: &mut Vec<String>, warnings: &Vec<String>, label: &str, tag: &str,
    points: &Vec<[f64; 3]>, triangles: &Vec<[u32; 3]>, limits: &Limits,
) -> Result<()> {
    use om9_core::budget::Usage;
    let bytes = label.len().checked_add(tag.len()).and_then(|n| n.checked_add(3))
        .ok_or(Error::Overflow("STL skipped note"))?;
    let remaining = limits_after_stl_notes(limits, skipped, warnings)?;
    let mut held = Usage::array::<[f64; 3]>(points.capacity())?;
    held.add(Usage::array::<[u32; 3]>(triangles.capacity())?, &remaining)?;
    if skipped.len() == skipped.capacity() {
        let requested = stl_growth_capacity(skipped.capacity(), skipped.len().checked_add(1).ok_or(Error::Overflow("STL skipped notes"))?);
        held.add(Usage::array::<String>(requested)?, &remaining)?;
        skipped.try_reserve_exact(requested - skipped.len()).map_err(|_| Error::Budget("STL skipped notes"))?;
        let remaining = limits_after_stl_notes(limits, skipped, warnings)?;
        let mut soup = Usage::array::<[f64; 3]>(points.capacity())?;
        soup.add(Usage::array::<[u32; 3]>(triangles.capacity())?, &remaining)?;
        soup.add(Usage { bytes: bytes as u64, ..Usage::default() }, &remaining)?;
    } else {
        held.add(Usage { bytes: bytes as u64, ..Usage::default() }, &remaining)?;
    }
    let mut text = String::new();
    text.try_reserve_exact(bytes).map_err(|_| Error::Budget("STL skipped note"))?;
    let remaining = limits_after_stl_notes(limits, skipped, warnings)?;
    let mut soup = Usage::array::<[f64; 3]>(points.capacity())?;
    soup.add(Usage::array::<[u32; 3]>(triangles.capacity())?, &remaining)?;
    soup.add(Usage { bytes: text.capacity() as u64, ..Usage::default() }, &remaining)?;
    text.push_str(label);
    text.push_str(" (");
    text.push_str(tag);
    text.push(')');
    skipped.push(text);
    Ok(())
}

/// Account already-created receipt strings while topology and writing run.
/// Their earlier construction and allocator overhead need separate admission.
fn limits_after_stl_notes(limits: &Limits, skipped: &Vec<String>, warnings: &Vec<String>) -> Result<Limits> {
    use om9_core::budget::Usage;
    let mut held = Usage::array::<String>(skipped.capacity())?;
    held.add(Usage::array::<String>(warnings.capacity())?, limits)?;
    for text in skipped.iter().chain(warnings) {
        held.add(Usage { bytes: text.capacity() as u64, ..Usage::default() }, limits)?;
    }
    limits.remaining_after(held)
}

fn stl_growth_capacity(current: usize, needed: usize) -> usize {
    if needed <= current { current } else { current.checked_mul(2).unwrap_or(needed).max(needed).max(4) }
}

/// Admit the soup and known lower-bound topology storage together. Hash bucket
/// metadata and allocator overhead remain separate.
#[cfg(test)]
fn validate_stl_topology(soup: &TriangleSoup, limits: &Limits, cancel: &StlCancellation) -> Result<om9_exchange::stl::EdgeTopology> {
    validate_stl_topology_admitted(soup, limits, cancel, &mut |_, _| Ok(()))
}
fn validate_stl_topology_admitted(soup: &TriangleSoup, limits: &Limits, cancel: &StlCancellation,
    admit: &mut StlAdmission<'_>) -> Result<om9_exchange::stl::EdgeTopology> {
    use om9_core::budget::Usage;
    cancel.check()?;
    let mut held = Usage::array::<[f64; 3]>(soup.positions.capacity())?;
    held.add(Usage::array::<[u32; 3]>(soup.triangles.capacity())?, limits)?;
    om9_exchange::stl::try_edge_topology_with_admission(
        soup, || cancel.check(), || Error::Budget("STL topology validation"), |bytes| {
            let mut combined = held;
            combined.add(Usage { bytes, ..Usage::default() }, limits)?;
            admit(held.bytes, bytes)
        },
    )
}

/// Check both sequential vector growth peaks. Old and requested new buffers
/// may coexist during reallocation; the caller keeps source mesh ownership.
fn admit_mesh_output_growth(
    points: &Vec<[f64; 3]>, triangles: &Vec<[u32; 3]>,
    additional_points: usize, additional_triangles: usize, limits: &Limits,
) -> Result<(usize, usize)> {
    use om9_core::budget::Usage;
    let next_points = points.len().checked_add(additional_points).ok_or(Error::Overflow("STL mesh vertices"))?;
    let next_triangles = triangles.len().checked_add(additional_triangles).ok_or(Error::Overflow("STL mesh triangles"))?;
    let check = |point_capacity: usize, triangle_capacity: usize| -> Result<()> {
        let mut point_peak = Usage::array::<[f64; 3]>(points.capacity())?;
        point_peak.add(Usage::array::<[u32; 3]>(triangles.capacity())?, limits)?;
        if point_capacity > points.capacity() { point_peak.add(Usage::array::<[f64; 3]>(point_capacity)?, limits)?; }
        let mut triangle_peak = Usage::array::<[f64; 3]>(point_capacity)?;
        triangle_peak.add(Usage::array::<[u32; 3]>(triangles.capacity())?, limits)?;
        if triangle_capacity > triangles.capacity() { triangle_peak.add(Usage::array::<[u32; 3]>(triangle_capacity)?, limits)?; }
        Ok(())
    };
    let planned = (stl_growth_capacity(points.capacity(), next_points), stl_growth_capacity(triangles.capacity(), next_triangles));
    match check(planned.0, planned.1) {
        Ok(()) => Ok(planned),
        Err(Error::Budget(_)) => {
            let exact = (points.capacity().max(next_points), triangles.capacity().max(next_triangles));
            check(exact.0, exact.1)?;
            Ok(exact)
        }
        Err(error) => Err(error),
    }
}

#[cfg(test)]
fn append_mesh(
    placement: &Transform,
    mesh: &Mesh,
    points: &mut Vec<[f64; 3]>,
    triangles: &mut Vec<[u32; 3]>,
    limits: &Limits,
    cancel: &StlCancellation,
) -> Result<()> {
    append_mesh_admitted(placement, mesh, points, triangles, limits, cancel, &mut |_, _| Ok(()))
}
fn append_mesh_admitted(
    placement: &Transform, mesh: &Mesh, points: &mut Vec<[f64; 3]>, triangles: &mut Vec<[u32; 3]>,
    limits: &Limits, cancel: &StlCancellation, admit: &mut StlAdmission<'_>,
) -> Result<()> {
    cancel.check()?;
    let base = checked_mesh_base(points.len(), mesh.vertices.len(), limits)?;
    let mut triangle_count = 0usize;
    for (index, face) in mesh.faces.iter().enumerate() {
        if index % 1024 == 0 { cancel.check()?; }
        if face.iter().any(|corner| *corner as usize >= mesh.vertices.len()) {
            return Err(Error::Invalid("STL mesh face index"));
        }
        triangle_count = triangle_count.checked_add(match face.len() {
            0..=2 => 0,
            3 => 1,
            4 => 2,
            _ => return Err(Error::Unsupported("STL polygon with more than four corners")),
        }).ok_or(Error::Overflow("STL triangle count"))?;
    }
    check_triangle_count(triangles.len(), triangle_count, limits)?;
    let (point_capacity, _) = admit_mesh_output_growth(points, triangles, mesh.vertices.len(), triangle_count, limits)?;
    admit(stl_soup_bytes(points, triangles)?, if point_capacity > points.capacity() {
        om9_core::budget::Usage::array::<[f64; 3]>(point_capacity)?.bytes } else { 0 })?;
    points.try_reserve_exact(point_capacity - points.len()).map_err(|_| Error::Budget("STL mesh vertices"))?;
    admit(stl_soup_bytes(points, triangles)?, 0)?;
    let (_, triangle_capacity) = admit_mesh_output_growth(points, triangles, mesh.vertices.len(), triangle_count, limits)?;
    admit(stl_soup_bytes(points, triangles)?, if triangle_capacity > triangles.capacity() {
        om9_core::budget::Usage::array::<[u32; 3]>(triangle_capacity)?.bytes } else { 0 })?;
    triangles.try_reserve_exact(triangle_capacity - triangles.len()).map_err(|_| Error::Budget("STL mesh triangles"))?;
    admit(stl_soup_bytes(points, triangles)?, 0)?;
    admit_mesh_output_growth(points, triangles, mesh.vertices.len(), triangle_count, limits)?;
    for (index, vertex) in mesh.vertices.iter().enumerate() {
        if index % 1024 == 0 { cancel.check()?; }
        points.push(placement.point(*vertex)?);
    }
    for (index, face) in mesh.faces.iter().enumerate() {
        if index % 1024 == 0 { cancel.check()?; }
        if face.len() < 3 {
            continue;
        }
        match face.as_slice() {
            [a, b, c] => triangles.push([base + *a, base + *b, base + *c]),
            [a, b, c, d] => {
                let vertices = [*a, *b, *c, *d].map(|i| mesh.vertices[i as usize]);
                let diagonal_ac = quad_diagonal_ac(vertices)?;
                let indices = if diagonal_ac { [[*a, *b, *c], [*a, *c, *d]] } else { [[*a, *b, *d], [*b, *c, *d]] };
                triangles.extend(indices.map(|tri| tri.map(|i| base + i)));
            }
            _ => return Err(Error::Unsupported("STL polygon with more than four corners")),
        }
    }
    Ok(())
}

fn check_triangle_count(existing: usize, additional: usize, limits: &Limits) -> Result<()> {
    let total = existing.checked_add(additional).ok_or(Error::Overflow("STL triangle count"))?;
    if total as u128 > u128::from(u32::MAX) { return Err(Error::Overflow("STL triangle count")); }
    if total as u128 > u128::from(limits.max_nodes) { return Err(Error::Budget("STL triangles")); }
    Ok(())
}

fn checked_mesh_base(existing: usize, additional: usize, limits: &Limits) -> Result<u32> {
    let end = existing.checked_add(additional).ok_or(Error::Overflow("STL mesh vertices"))?;
    if end as u128 > u128::from(limits.max_points) { return Err(Error::Budget("STL mesh vertices")); }
    if end as u128 > u128::from(u32::MAX) { return Err(Error::Overflow("STL mesh vertices")); }
    u32::try_from(existing).map_err(|_| Error::Overflow("STL mesh base"))
}

use om9_core::geometry::quad_diagonal_ac;

#[cfg(feature = "occt-brep")]
fn local_deflection(placement: &Transform, world_tolerance: f64) -> Result<f64> {
    let scale = placement_scale_upper_bound(placement)?;
    // Reserve half the requested tolerance for other error sources. OCCT's
    // deflection parameter is a request, not a global error certificate.
    let local = (world_tolerance * 0.5) / scale;
    if !local.is_finite() || local <= 0.0 { return Err(Error::Invalid("STL placement tolerance")); }
    Ok(local)
}

#[cfg(feature = "occt-brep")]
fn placement_scale_upper_bound(placement: &Transform) -> Result<f64> {
    placement.validate()?;
    let m = &placement.0;
    let row = (0..3).map(|i| (0..3).map(|j| m[i][j].abs()).sum::<f64>()).fold(0.0_f64, f64::max);
    let col = (0..3).map(|j| (0..3).map(|i| m[i][j].abs()).sum::<f64>()).fold(0.0_f64, f64::max);
    // sqrt(||A||_1 ||A||_inf) is an upper bound for the Euclidean operator norm.
    let scale = row.sqrt() * col.sqrt();
    if !scale.is_finite() || scale <= 0.0 { return Err(Error::Invalid("STL placement tolerance")); }
    Ok(scale)
}

#[cfg(all(feature = "occt-brep", test))]
fn source_tolerance_note(label: &str, (source, upper): (f64, f64), requested: f64) -> String {
    format!("{label}: OCCT maximum face/edge/vertex tolerance {source} source units; placement-scaled upper bound {upper} mm{}; metadata is not measured mesh deviation",
        if upper > requested { " exceeds requested STL tolerance" } else { "" })
}

#[cfg(feature = "occt-brep")]
fn limits_after_stl_soup(limits: &Limits, points: &Vec<[f64; 3]>, triangles: &Vec<[u32; 3]>) -> Result<Limits> {
    let mut held = om9_core::budget::Usage::array::<[f64; 3]>(points.capacity())?;
    held.add(om9_core::budget::Usage::array::<[u32; 3]>(triangles.capacity())?, limits)?;
    limits.remaining_after(held)
}

#[cfg(feature = "occt-brep")]
fn admit_brep_output_peak(
    limits: &Limits, points: &Vec<[f64; 3]>, triangles: &Vec<[u32; 3]>,
    source_points: &Vec<[f64; 3]>, source_triangles: &Vec<[u32; 4]>, remap: &Vec<u32>,
    additional: om9_core::budget::Usage,
) -> Result<()> {
    use om9_core::budget::Usage;
    let mut usage = additional;
    for held in [Usage::array::<[f64; 3]>(points.capacity())?, Usage::array::<[u32; 3]>(triangles.capacity())?,
        Usage::array::<[f64; 3]>(source_points.capacity())?, Usage::array::<[u32; 4]>(source_triangles.capacity())?,
        Usage::array::<u32>(remap.capacity())?] {
        usage.add(held, limits)?;
    }
    Ok(())
}

#[cfg(feature = "occt-brep")]
fn plan_brep_output_capacity(
    limits: &Limits, points: &Vec<[f64; 3]>, triangles: &Vec<[u32; 3]>,
    source_points: &Vec<[f64; 3]>, source_triangles: &Vec<[u32; 4]>, remap: &Vec<u32>,
    next_points: usize, next_triangles: usize,
) -> Result<(usize, usize)> {
    use om9_core::budget::Usage;
    let check = |point_capacity: usize, triangle_capacity: usize| -> Result<()> {
        let triangle_growth = if triangle_capacity > triangles.capacity() { Usage::array::<[u32; 3]>(triangle_capacity)? } else { Usage::default() };
        admit_brep_output_peak(limits, points, triangles, source_points, source_triangles, remap, triangle_growth)?;
        let mut point_peak = Usage::array::<[f64; 3]>(points.capacity())?;
        for held in [Usage::array::<[u32; 3]>(triangle_capacity)?, Usage::array::<[f64; 3]>(source_points.capacity())?,
            Usage::array::<[u32; 4]>(source_triangles.capacity())?, Usage::array::<u32>(remap.capacity())?] {
            point_peak.add(held, limits)?;
        }
        if !points.is_empty() && point_capacity > points.capacity() { point_peak.add(Usage::array::<[f64; 3]>(point_capacity)?, limits)?; }
        Ok(())
    };
    let point_plan = if points.is_empty() { points.capacity() } else { stl_growth_capacity(points.capacity(), next_points) };
    let planned = (point_plan, stl_growth_capacity(triangles.capacity(), next_triangles));
    match check(planned.0, planned.1) {
        Ok(()) => Ok(planned),
        Err(Error::Budget(_)) => {
            let point_exact = if points.is_empty() { points.capacity() } else { points.capacity().max(next_points) };
            let exact = (point_exact, triangles.capacity().max(next_triangles));
            check(exact.0, exact.1)?;
            Ok(exact)
        }
        Err(error) => Err(error),
    }
}

#[cfg(all(feature = "occt-brep", test))]
fn append_brep_source(
    placement: &Transform,
    source: &str,
    points: &mut Vec<[f64; 3]>,
    triangles: &mut Vec<[u32; 3]>,
    world_tolerance: f64,
    angular_tolerance: f64,
    limits: &Limits,
    cancel: &StlCancellation,
) -> Result<(f64, f64)> {
    append_brep_source_admitted(placement, source, points, triangles, world_tolerance, angular_tolerance,
        limits, cancel, &mut |_, _| Ok(()))
}
#[cfg(feature = "occt-brep")]
fn append_brep_source_admitted(
    placement: &Transform, source: &str, points: &mut Vec<[f64; 3]>, triangles: &mut Vec<[u32; 3]>,
    world_tolerance: f64, angular_tolerance: f64, limits: &Limits, cancel: &StlCancellation,
    admit: &mut StlAdmission<'_>,
) -> Result<(f64, f64)> {
    // Re-tessellate canonical source independently of the viewport display profile.
    let remaining = limits_after_stl_soup(limits, points, triangles)?;
    let soup_bytes = stl_soup_bytes(points, triangles)?;
    let deflection = local_deflection(placement, world_tolerance)?;
    cancel.check()?;
    let topology = stl_kernel_admitted(&mut |live, pending| {
        cancel.check()?;
        admit(stl_sum(live, soup_bytes)?, pending)
    }, "STL tessellation", |admit| om9_kernel::brep::read_brep_with_shell_groups_and_edges_admitted(
        source, deflection, angular_tolerance, &remaining, admit))?;
    let source_tolerance = topology.max_source_tolerance();
    let scaled_tolerance = source_tolerance * placement_scale_upper_bound(placement)?;
    if !scaled_tolerance.is_finite() { return Err(Error::Invalid("STL source tolerance scaling")); }
    cancel.check()?;
    let snapshot = topology.brep();
    let base = u32::try_from(points.len()).map_err(|_| Error::Overflow("STL B-Rep base"))?;
    let end = points.len().checked_add(snapshot.display.points.len()).ok_or(Error::Overflow("STL B-Rep vertices"))?;
    if end as u64 > limits.max_points { return Err(Error::Budget("STL B-Rep vertices")); }
    if end > u32::MAX as usize { return Err(Error::Overflow("STL B-Rep vertices")); }
    check_triangle_count(triangles.len(), snapshot.display.triangles.len(), limits)?;
    let retained = topology.retained_usage(&remaining)?.bytes;
    admit(stl_sum(soup_bytes, retained)?, topology.topology_remap_estimated_usage()?.bytes)?;
    let remap = topology.topology_remap_with_budget(*placement, WELD_MM.min(world_tolerance / 10.0), &remaining, || cancel.check())?;
    let om9_core::brep::BrepDisplay { points: source_points, triangles: source_triangles, .. } = topology.into_display()?;
    let next_triangles = triangles.len().checked_add(source_triangles.len()).ok_or(Error::Overflow("STL B-Rep triangles"))?;
    let next_points = points.len().checked_add(source_points.len()).ok_or(Error::Overflow("STL B-Rep vertices"))?;
    let (_, triangle_capacity) = plan_brep_output_capacity(limits, points, triangles, &source_points, &source_triangles, &remap, next_points, next_triangles)?;
    let source_bytes = stl_sum(om9_core::budget::Usage::array::<[f64; 3]>(source_points.capacity())?.bytes,
        stl_sum(om9_core::budget::Usage::array::<[u32; 4]>(source_triangles.capacity())?.bytes,
            om9_core::budget::Usage::array::<u32>(remap.capacity())?.bytes)?)?;
    admit(stl_sum(stl_soup_bytes(points, triangles)?, source_bytes)?, if triangle_capacity > triangles.capacity() {
        om9_core::budget::Usage::array::<[u32; 3]>(triangle_capacity)?.bytes } else { 0 })?;
    triangles.try_reserve_exact(triangle_capacity - triangles.len()).map_err(|_| Error::Budget("STL B-Rep triangles"))?;
    admit(stl_sum(stl_soup_bytes(points, triangles)?, source_bytes)?, 0)?;
    if !points.is_empty() {
        let (point_capacity, _) = plan_brep_output_capacity(limits, points, triangles, &source_points, &source_triangles, &remap, next_points, next_triangles)?;
        admit(stl_sum(stl_soup_bytes(points, triangles)?, source_bytes)?, if point_capacity > points.capacity() {
            om9_core::budget::Usage::array::<[f64; 3]>(point_capacity)?.bytes } else { 0 })?;
        points.try_reserve_exact(point_capacity - points.len()).map_err(|_| Error::Budget("STL B-Rep vertices"))?;
        admit(stl_sum(stl_soup_bytes(points, triangles)?, source_bytes)?, 0)?;
    }
    admit_brep_output_peak(limits, points, triangles, &source_points, &source_triangles, &remap, Default::default())?;
    append_brep_points(placement, source_points, points, cancel)?;
    for (index, [a, b, c, _]) in source_triangles.iter().enumerate() {
        if index % 1024 == 0 { cancel.check()?; }
        let before = [*a, *b, *c];
        let after = before.map(|point| remap[point as usize]);
        let original = before.map(|point| points[base as usize + point as usize]);
        let result = after.map(|point| points[base as usize + point as usize]);
        if after[0] == after[1] || after[1] == after[2] || after[0] == after[2] {
            if original[0] != original[1] && original[1] != original[2] && original[0] != original[2] {
                return Err(Error::Invalid("STL topology weld collapsed a triangle"));
            }
        } else if !same_triangle_orientation(original, result) {
            return Err(Error::Invalid("STL topology weld reversed or collapsed a triangle"));
        }
        triangles.push(after.map(|point| base + point));
    }
    Ok((source_tolerance, scaled_tolerance))
}

#[cfg(feature = "occt-brep")]
fn append_brep_points(placement: &Transform, mut source: Vec<[f64; 3]>, points: &mut Vec<[f64; 3]>, cancel: &StlCancellation) -> Result<()> {
    if points.is_empty() {
        for (index, point) in source.iter_mut().enumerate() {
            if index % 1024 == 0 { cancel.check()?; }
            *point = placement.point(*point)?;
        }
        *points = source;
    } else {
        points.try_reserve_exact(source.len()).map_err(|_| Error::Budget("STL B-Rep vertices"))?;
        for (index, point) in source.into_iter().enumerate() {
            if index % 1024 == 0 { cancel.check()?; }
            points.push(placement.point(point)?);
        }
    }
    Ok(())
}

#[cfg(feature = "occt-brep")]
fn same_triangle_orientation(a: [[f64; 3]; 3], b: [[f64; 3]; 3]) -> bool {
    let normal = |p: [[f64; 3]; 3]| {
        let u = [p[1][0]-p[0][0], p[1][1]-p[0][1], p[1][2]-p[0][2]];
        let v = [p[2][0]-p[0][0], p[2][1]-p[0][1], p[2][2]-p[0][2]];
        let n = [u[1]*v[2]-u[2]*v[1], u[2]*v[0]-u[0]*v[2], u[0]*v[1]-u[1]*v[0]];
        let length = n[0].hypot(n[1]).hypot(n[2]);
        (length.is_finite() && length > 0.).then(|| n.map(|value| value / length))
    };
    match (normal(a), normal(b)) {
        (Some(x), Some(y)) => (0..3).map(|i| x[i] * y[i]).sum::<f64>() > 0.,
        _ => false,
    }
}

// u32::MAX cannot be a valid index: the allowed vertex count is at most
// u32::MAX, so its largest index is u32::MAX - 1. Mark generated collapses in
// place instead of retaining a separate source flag for every triangle.
const OMIT_GENERATED_TRIANGLE: [u32; 3] = [u32::MAX; 3];

fn mark_collapses(point_count: usize, triangles: &mut [[u32; 3]], generated: bool, cancel: &StlCancellation) -> Result<()> {
    for (index, triangle) in triangles.iter_mut().enumerate() {
        if index % 1024 == 0 { cancel.check()?; }
        if triangle.iter().any(|point| *point as usize >= point_count) { return Err(Error::Invalid("STL triangle index")); }
        if triangle[0] == triangle[1] || triangle[1] == triangle[2] || triangle[0] == triangle[2] {
            if !generated { return Err(Error::Invalid("STL degenerate authored triangle")); }
            *triangle = OMIT_GENERATED_TRIANGLE;
        }
    }
    Ok(())
}

fn finalize_soup(points: Vec<[f64; 3]>, mut triangles: Vec<[u32; 3]>, cancel: &StlCancellation) -> Result<(TriangleSoup, usize)> {
    cancel.check()?;
    let mut omitted = 0usize;
    let mut retained = 0usize;
    for index in 0..triangles.len() {
        if index % 1024 == 0 { cancel.check()?; }
        let triangle = triangles[index];
        if triangle == OMIT_GENERATED_TRIANGLE { omitted += 1; continue; }
        if triangle.iter().any(|point| *point as usize >= points.len()) { return Err(Error::Invalid("STL triangle index")); }
        if triangle[0] == triangle[1] || triangle[1] == triangle[2] || triangle[0] == triangle[2] {
            return Err(Error::Invalid("STL degenerate authored triangle"));
        }
        triangles[retained] = triangle;
        retained += 1;
    }
    cancel.check()?;
    triangles.truncate(retained);
    Ok((TriangleSoup::new_f64(points, triangles), omitted))
}
#[cfg(all(test, feature = "occt-brep"))]
mod topology_orientation_tests {
    use super::{append_brep_points, append_brep_source, limits_after_stl_soup, plan_brep_output_capacity, same_triangle_orientation, finalize_soup, mark_collapses, source_tolerance_note, StlCancellation};
    use om9_core::geometry::Transform;
    #[test]
    fn source_copy_refusal_keeps_output_empty_and_preserves_typed_budget_error() {
        let limits = om9_core::budget::Limits::default();
        let body = om9_kernel::brep::box_brep([10.; 3], 0.1, &limits).unwrap();
        let mut points = Vec::new();
        let mut triangles = Vec::new();
        let mut refused = false;
        let result = super::append_brep_source_admitted(&Transform::identity(), body.source(), &mut points, &mut triangles,
            0.1, 0.3, &limits, &StlCancellation::default(), &mut |_, pending| {
                if pending > 0 { refused = true; Err(om9_core::Error::Budget("injected source phase")) } else { Ok(()) }
            });
        assert!(refused);
        assert!(matches!(result, Err(om9_core::Error::Budget("injected source phase"))));
        assert_eq!((points.capacity(), triangles.capacity()), (0, 0));
    }
    #[test]
    fn brep_growth_plans_both_buffers_before_falling_back_to_exact() {
        let points = vec![[0.; 3]; 4];
        let triangles = vec![[0; 3]; 4];
        let source_points = vec![[0.; 3]];
        let source_triangles = vec![[0; 4]];
        let remap = vec![0u32];
        let base = (points.capacity() * std::mem::size_of::<[f64; 3]>()
            + 5 * std::mem::size_of::<[u32; 3]>()
            + source_points.capacity() * std::mem::size_of::<[f64; 3]>()
            + source_triangles.capacity() * std::mem::size_of::<[u32; 4]>()
            + remap.capacity() * std::mem::size_of::<u32>()
            + 5 * std::mem::size_of::<[f64; 3]>()) as u64;
        let limits = om9_core::budget::Limits { max_bytes: base, ..Default::default() };
        assert_eq!(plan_brep_output_capacity(&limits, &points, &triangles, &source_points, &source_triangles, &remap, 5, 5).unwrap(), (5, 5));
        let low = om9_core::budget::Limits { max_bytes: base - 1, ..limits };
        assert!(plan_brep_output_capacity(&low, &points, &triangles, &source_points, &source_triangles, &remap, 5, 5).is_err());
    }
    #[test]
    fn repeated_brep_appends_use_amortized_capacity_and_preserve_geometry() {
        let limits = om9_core::budget::Limits::default();
        let body = om9_kernel::brep::box_brep([2.; 3], 0.1, &limits).unwrap();
        let mut points = Vec::new();
        let mut triangles = Vec::new();
        let mut point_growths = 0;
        let mut triangle_growths = 0;
        let cancel = StlCancellation::default();
        let mut first_points = Vec::new();
        let mut first_triangles = Vec::new();
        for index in 0..64 {
            let before = (points.capacity(), triangles.capacity());
            append_brep_source(&Transform::identity(), body.source(), &mut points, &mut triangles,
                0.1, 0.3, &limits, &cancel).unwrap();
            point_growths += usize::from(points.capacity() != before.0);
            triangle_growths += usize::from(triangles.capacity() != before.1);
            if index == 0 { first_points = points.clone(); first_triangles = triangles.clone(); }
        }
        assert!(point_growths <= 10 && triangle_growths <= 10, "growths: {point_growths}/{triangle_growths}");
        assert_eq!(points.len(), first_points.len() * 64);
        assert_eq!(triangles.len(), first_triangles.len() * 64);
        assert_eq!(&points[first_points.len() * 63..], first_points.as_slice());
        let base = (first_points.len() * 63) as u32;
        for (actual, first) in triangles[first_triangles.len() * 63..].iter().zip(&first_triangles) {
            assert_eq!(*actual, first.map(|index| index + base));
        }
    }
    #[test]
    fn output_growth_rejects_when_old_and_new_triangle_buffers_cannot_coexist() {
        let default = om9_core::budget::Limits::default();
        let body = om9_kernel::brep::box_brep([10.; 3], 0.1, &default).unwrap();
        let snapshot = om9_kernel::brep::read_brep_with_shell_groups_and_edges(body.source(), 0.05, 0.3, &default).unwrap();
        let mut points = vec![[0., 0., 0.]];
        let mut triangles = vec![[0, 0, 0]; 100_000];
        let held = (points.capacity() * std::mem::size_of::<[f64; 3]>()
            + triangles.capacity() * std::mem::size_of::<[u32; 3]>()) as u64;
        let limit = held + snapshot.retained_usage(&default).unwrap().bytes
            + snapshot.topology_remap_estimated_usage().unwrap().bytes;
        drop(snapshot);
        let limits = om9_core::budget::Limits { max_bytes: limit, ..default };
        let before = (points.as_ptr(), points.len(), triangles.as_ptr(), triangles.len());
        let result = append_brep_source(&Transform::identity(), body.source(), &mut points, &mut triangles,
            0.1, 0.3, &limits, &StlCancellation::default());
        assert!(matches!(result, Err(om9_core::Error::Budget(_))), "output growth must be refused: {result:?}");
        assert_eq!(before, (points.as_ptr(), points.len(), triangles.as_ptr(), triangles.len()));
    }
    #[test]
    fn output_growth_rejects_when_old_and_new_point_buffers_cannot_coexist() {
        let default = om9_core::budget::Limits::default();
        let body = om9_kernel::brep::box_brep([10.; 3], 0.1, &default).unwrap();
        let snapshot = om9_kernel::brep::read_brep_with_shell_groups_and_edges(body.source(), 0.05, 0.3, &default).unwrap();
        let mut points = vec![[0., 0., 0.]; 100_000];
        let mut triangles = Vec::with_capacity(snapshot.brep().display.triangles.len());
        let held = (points.capacity() * std::mem::size_of::<[f64; 3]>()
            + triangles.capacity() * std::mem::size_of::<[u32; 3]>()) as u64;
        let limit = held + snapshot.retained_usage(&default).unwrap().bytes
            + snapshot.topology_remap_estimated_usage().unwrap().bytes;
        drop(snapshot);
        let limits = om9_core::budget::Limits { max_bytes: limit, ..default };
        let before = (points.as_ptr(), points.len(), triangles.as_ptr(), triangles.len());
        let result = append_brep_source(&Transform::identity(), body.source(), &mut points, &mut triangles,
            0.1, 0.3, &limits, &StlCancellation::default());
        assert!(matches!(result, Err(om9_core::Error::Budget(_))), "point growth must be refused: {result:?}");
        assert_eq!(before, (points.as_ptr(), points.len(), triangles.as_ptr(), triangles.len()));
    }
    #[test]
    fn existing_soup_capacity_reduces_next_native_snapshot_budget() {
        let default = om9_core::budget::Limits::default();
        let body = om9_kernel::brep::box_brep([10.; 3], 0.1, &default).unwrap();
        assert!(body.source().len() > 128);
        let mut points = Vec::with_capacity(50_000);
        let mut triangles = Vec::with_capacity(100);
        let held = (points.capacity() * std::mem::size_of::<[f64; 3]>()
            + triangles.capacity() * std::mem::size_of::<[u32; 3]>()) as u64;
        let limits = om9_core::budget::Limits { max_bytes: held + 128, ..default };
        assert_eq!(limits_after_stl_soup(&limits, &points, &triangles).unwrap().max_bytes, 128);
        let original_storage = points.as_ptr();
        let result = append_brep_source(&Transform::identity(), body.source(), &mut points, &mut triangles,
            0.1, 0.3, &limits, &StlCancellation::default());
        assert!(result.is_err(), "snapshot must fit alongside retained soup capacity");
        assert!(points.is_empty() && triangles.is_empty());
        assert_eq!(points.as_ptr(), original_storage);
    }
    #[test]
    fn first_brep_reuses_point_storage_and_later_brep_appends() {
        let source = vec![[0., 1., 2.], [4., 5., 6.]];
        let storage = source.as_ptr();
        let mut output = Vec::new();
        let cancel = StlCancellation::default();
        let placement = Transform::translation([10., -2., 0.5]);
        append_brep_points(&placement, source, &mut output, &cancel).unwrap();
        assert_eq!(output.as_ptr(), storage);
        assert_eq!(output, [[10., -1., 2.5], [14., 3., 6.5]]);
        append_brep_points(&placement, vec![[1., 0., 0.]], &mut output, &cancel).unwrap();
        assert_eq!(output, [[10., -1., 2.5], [14., 3., 6.5], [11., -2., 0.5]]);
        cancel.cancel();
        assert!(append_brep_points(&placement, vec![[0.; 3]], &mut output, &cancel).is_err());
    }
    #[test]
    fn source_tolerance_note_distinguishes_metadata_from_measured_error() {
        let note = source_tolerance_note("test", (0.002, 0.004), 0.001);
        assert!(note.contains("exceeds requested STL tolerance"));
        assert!(note.contains("metadata is not measured mesh deviation"));
        assert!(!source_tolerance_note("test", (0.0001, 0.001), 0.001).contains("exceeds requested"));
    }
    #[test]
    fn topology_remap_rejects_flipped_or_collapsed_faces() {
        let source = [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]];
        assert!(same_triangle_orientation(source, [[0., 0., 0.], [1.001, 0., 0.], [0., 1., 0.]]));
        assert!(!same_triangle_orientation(source, [source[0], source[2], source[1]]));
        assert!(!same_triangle_orientation(source, [source[0], source[0], source[2]]));
    }
    #[test]
    fn finalization_omits_only_repeated_generated_corners() {
        let points = vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]];
        let cancel = StlCancellation::default();
        let mut generated = vec![[0, 0, 1], [0, 1, 2]];
        mark_collapses(points.len(), &mut generated, true, &cancel).unwrap();
        let (soup, omitted) = finalize_soup(points.clone(), generated, &cancel).unwrap();
        assert_eq!(omitted, 1);
        assert_eq!(soup.triangles, vec![[0, 1, 2]]);
        assert!(mark_collapses(points.len(), &mut [[0, 0, 1]], false, &cancel).is_err());
        assert!(mark_collapses(points.len(), &mut [[0, 1, 99]], true, &cancel).is_err());
        assert!(mark_collapses(points.len(), &mut [[u32::MAX; 3]], false, &cancel).is_err());
    }
    #[test]
    fn finalization_reuses_triangle_storage_and_preserves_order() {
        let points = vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.], [0.,0.,1.]];
        let mut triangles = vec![[0,1,2], [0,0,1], [0,2,3], [1,1,2], [0,3,1]];
        let storage = triangles.as_ptr();
        let capacity = triangles.capacity();
        let cancel = StlCancellation::default();
        mark_collapses(points.len(), &mut triangles[1..2], true, &cancel).unwrap();
        mark_collapses(points.len(), &mut triangles[3..4], true, &cancel).unwrap();
        let (soup, omitted) = finalize_soup(points, triangles, &cancel).unwrap();
        assert_eq!(omitted, 2);
        assert_eq!(soup.triangles, vec![[0,1,2], [0,2,3], [0,3,1]]);
        assert_eq!(soup.triangles.as_ptr(), storage);
        assert_eq!(soup.triangles.capacity(), capacity);
    }
}

#[cfg(test)]
mod cancellation_tests {
    use super::*;

    #[test]
    fn authored_mesh_preflight_checks_cancellation_during_face_scan() {
        let mesh = Mesh { vertices: vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]],
            faces: vec![vec![0,1,2]; 4096], normals: vec![], colors: vec![], corner_uvs: vec![] };
        let mut checks = 0;
        let result = authored_mesh_triangle_count(&mesh, || {
            checks += 1;
            if checks == 2 { Err(Error::Canceled) } else { Ok(()) }
        });
        assert!(matches!(result, Err(Error::Canceled)));
        assert_eq!(checks, 2);
        assert_eq!(authored_mesh_triangle_count(&mesh, || Ok(())).unwrap(), 4096);
    }

    #[test]
    fn topology_minimum_is_admitted_with_live_soup_before_scratch_allocation() {
        let soup = TriangleSoup::new_f64(vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]], vec![[0,1,2]]);
        let required = (soup.positions.capacity() * std::mem::size_of::<[f64; 3]>()
            + soup.triangles.capacity() * std::mem::size_of::<[u32; 3]>()
            + soup.triangles.len() * (std::mem::size_of::<usize>() + std::mem::size_of::<u8>())) as u64;
        let low = Limits { max_bytes: required - 1, ..Limits::default() };
        let cancel = StlCancellation::default();
        assert!(matches!(validate_stl_topology(&soup, &low, &cancel), Err(Error::Budget(_))));
        let arrays_only = Limits { max_bytes: required, ..low };
        assert!(matches!(validate_stl_topology(&soup, &arrays_only, &cancel), Err(Error::Budget(_))), "face table must also fit");
        let actual = validate_stl_topology(&soup, &Limits::default(), &cancel).unwrap();
        assert_eq!(actual, om9_exchange::stl::edge_topology(&soup));
        assert_eq!(actual.boundary, 3);
        assert!(cancel.cancel());
        assert!(matches!(validate_stl_topology(&soup, &low, &cancel), Err(Error::Canceled)));
    }

    #[test]
    fn receipt_notes_are_charged_before_stl_topology_and_preserve_destination() {
        let mesh = Mesh { vertices: vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]],
            faces: vec![vec![0,1,2]], normals: vec![], colors: vec![], corner_uvs: vec![] };
        let mut document = om9_core::model::Document::with_template_tolerance("STL note budget");
        let mesh = Entity::new("mesh", Geometry::Mesh(mesh), None);
        let mut ids = vec![mesh.id];
        document.entities.insert(mesh.id, mesh);
        for index in 0..60 {
            let point = Entity::new(format!("{index}{}", "x".repeat(300)), Geometry::Point([index as f64, 0., 0.]), None);
            ids.push(point.id);
            document.entities.insert(point.id, point);
        }
        let mut controller = Controller::new(document, Limits::default()).unwrap();
        controller.select_objects(&ids).unwrap();
        let mut capture = controller.capture_stl_selected().unwrap();
        capture.limits.max_bytes = 10_000;
        let profile = StlExportProfile { linear_tolerance_mm: 0.01, angular_tolerance_rad: 0.3 };
        assert!(matches!(capture.stl_soup(profile, &StlCancellation::default()), Err(Error::Budget(_))));
        let target = std::env::temp_dir().join(format!("om9-stl-notes-budget-{}.stl", uuid::Uuid::new_v4()));
        std::fs::write(&target, b"original destination").unwrap();
        let result = capture.export_with_profile(&target, profile);
        let bytes = std::fs::read(&target).unwrap();
        std::fs::remove_file(target).unwrap();
        assert!(matches!(result, Err(Error::Budget(_))));
        assert_eq!(bytes, b"original destination");
    }

    #[test]
    fn skipped_note_rejects_before_allocating_when_budget_is_too_small() {
        let mut skipped = Vec::new();
        let warnings = Vec::new();
        let limits = Limits { max_bytes: 1, ..Limits::default() };
        let result = push_skipped_note(&mut skipped, &warnings, "point", "point", &Vec::new(), &Vec::new(), &limits);
        assert!(matches!(result, Err(Error::Budget(_))));
        assert_eq!(skipped.capacity(), 0);
    }

    #[test]
    fn warning_budget_is_checked_before_format_allocation() {
        let mut warnings = Vec::new();
        let skipped = Vec::new();
        let low = Limits { max_bytes: 1, ..Limits::default() };
        let result = push_stl_note(&mut warnings, &skipped, format_args!("{}: {}", "surface", 0.125), &Vec::new(), &Vec::new(), &low);
        assert!(matches!(result, Err(Error::Budget(_))));
        assert_eq!(warnings.capacity(), 0);
        push_stl_note(&mut warnings, &skipped, format_args!("{}: {}", "surface", 0.125), &Vec::new(), &Vec::new(), &Limits::default()).unwrap();
        assert_eq!(warnings, ["surface: 0.125"]);
    }

    #[test]
    fn existing_notes_reduce_budget_for_later_geometry() {
        let mesh = Mesh { vertices: vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]],
            faces: vec![vec![0,1,2]], normals: vec![], colors: vec![], corner_uvs: vec![] };
        let mut document = om9_core::model::Document::with_template_tolerance("STL note and mesh budget");
        let point = Entity::new("x".repeat(250), Geometry::Point([0.,0.,0.]), None);
        let solid = Entity::new("mesh", Geometry::Mesh(mesh.clone()), None);
        let ids = [point.id, solid.id];
        document.entities.insert(point.id, point);
        document.entities.insert(solid.id, solid);
        let mut controller = Controller::new(document, Limits::default()).unwrap();
        controller.select_objects(&ids).unwrap();
        let mut capture = controller.capture_stl_selected().unwrap();
        capture.selection = ids.to_vec();
        capture.limits.max_bytes = 400;
        let cancel = StlCancellation::default();
        let mut skipped = Vec::new();
        let warnings = Vec::new();
        let mut points = Vec::new();
        let mut triangles = Vec::new();
        push_skipped_note(&mut skipped, &warnings, &"x".repeat(250), "point", &points, &triangles, &capture.limits).unwrap();
        let available = limits_after_stl_notes(&capture.limits, &skipped, &warnings).unwrap();
        assert!(matches!(append_mesh(&Transform::identity(), &mesh, &mut points, &mut triangles, &available, &cancel), Err(Error::Budget(_))));
        assert!(points.is_empty() && triangles.is_empty());
        assert!(append_mesh(&Transform::identity(), &mesh, &mut points, &mut triangles, &capture.limits, &cancel).is_ok());
        let profile = StlExportProfile { linear_tolerance_mm: 0.01, angular_tolerance_rad: 0.3 };
        assert!(matches!(capture.stl_soup(profile, &cancel), Err(Error::Budget(_))));
    }

    #[test]
    fn topology_budget_failure_does_not_replace_stl_destination() {
        let mesh = Mesh { vertices: vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]],
            faces: vec![vec![0,1,2]], normals: vec![], colors: vec![], corner_uvs: vec![] };
        let mut document = om9_core::model::Document::with_template_tolerance("topology budget");
        let entity = Entity::new("mesh", Geometry::Mesh(mesh), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        let mut controller = Controller::new(document, Limits::default()).unwrap();
        controller.select_objects(&[id]).unwrap();
        let mut capture = controller.capture_stl_selected().unwrap();
        capture.limits.max_bytes = (4 * std::mem::size_of::<[f64; 3]>() + 4 * std::mem::size_of::<[u32; 3]>()) as u64;
        let profile = StlExportProfile { linear_tolerance_mm: 0.01, angular_tolerance_rad: 0.3 };
        let (soup, _, _, _) = capture.stl_soup(profile, &StlCancellation::default()).unwrap();
        assert_eq!(soup.positions.len(), 3);
        assert_eq!(soup.triangles.len(), 1);
        drop(soup);
        let target = std::env::temp_dir().join(format!("om9-stl-topology-budget-{}.stl", uuid::Uuid::new_v4()));
        std::fs::write(&target, b"original destination").unwrap();
        let result = capture.export_with_profile(&target, profile);
        let bytes = std::fs::read(&target).unwrap();
        std::fs::remove_file(target).unwrap();
        assert!(matches!(result, Err(Error::Budget(_))), "topology budget must refuse: {result:?}");
        assert_eq!(bytes, b"original destination");
    }

    #[cfg(feature = "occt-brep")]
    #[test]
    fn worker_caps_native_snapshot_budget_to_its_reservation() {
        use om9_core::budget::memory::MemoryClass;
        const MIB: u64 = 1024 * 1024;
        let limits = Limits::default();
        let brep = om9_kernel::brep::box_brep([10.; 3], 0.1, &limits).unwrap();
        let small = brep.source().len() as u64 + 64;
        let mut document = om9_core::model::Document::with_template_tolerance("native worker budget");
        let entity = Entity::new("box", Geometry::Brep(brep), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        let mut controller = Controller::new(document, limits).unwrap();
        controller.select_objects(&[id]).unwrap();
        let capture = controller.capture_stl_selected().unwrap();
        let reservation = controller.coordinator().reserve_external_memory(MemoryClass::Export, small, Some((1024 * MIB, 1024 * MIB))).unwrap();
        let target = std::env::temp_dir().join(format!("om9-stl-native-limit-{}.stl", uuid::Uuid::new_v4()));
        std::fs::write(&target, b"unchanged").unwrap();
        let mut job = StlExportJob::start_with_sample(capture, target.clone(),
            StlExportProfile { linear_tolerance_mm: 0.01, angular_tolerance_rad: 0.3 }, reservation,
            || Some((1024 * MIB, 1024 * MIB))).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        let result = loop {
            if let Some(result) = job.try_result() { break result; }
            assert!(std::time::Instant::now() < deadline, "STL worker timed out");
            std::thread::yield_now();
        };
        let target_bytes = std::fs::read(&target).unwrap();
        std::fs::remove_file(target).unwrap();
        let error = result.expect_err("native output must not exceed its small worker budget");
        assert!(error.to_string().contains("BRep snapshot bytes"), "unexpected failure: {error}");
        assert_eq!(target_bytes, b"unchanged");
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Export), 0);
    }

    #[test]
    fn detached_stl_sees_live_root_selection_storage() {
        use om9_core::budget::{Usage, memory::MemoryClass};
        let mut document = om9_core::model::Document::with_template_tolerance("live selection storage");
        let mesh = Entity::new("triangle", Geometry::Mesh(Mesh {
            vertices: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]], faces: vec![vec![0, 1, 2]],
            normals: vec![], colors: vec![], corner_uvs: vec![],
        }), None);
        let mesh_id = mesh.id;
        document.entities.insert(mesh_id, mesh);
        let mut ids = vec![mesh_id];
        for index in 0..100 {
            let point = Entity::new("point", Geometry::Point([index as f64, 2., 0.]), None);
            ids.push(point.id);
            document.entities.insert(point.id, point);
        }
        let mut controller = Controller::new(document, Limits::default()).unwrap();
        controller.select_objects(&[mesh_id]).unwrap();
        let capture = controller.capture_stl_selected().unwrap();
        let (request, requested) = std::sync::mpsc::sync_channel(1);
        let (response, observed) = std::sync::mpsc::sync_channel(1);
        let worker = std::thread::spawn(move || {
            for _ in 0..4 {
                requested.recv_timeout(std::time::Duration::from_secs(10)).unwrap();
                let bytes = capture.admission_peer.retained_usage().unwrap().bytes;
                let cpu = capture.admission_peer.external_reserved_bytes(MemoryClass::CpuCache);
                response.send((bytes, cpu)).unwrap();
            }
            capture
        });
        let observe = || {
            request.send(()).unwrap();
            observed.recv_timeout(std::time::Duration::from_secs(10)).unwrap()
        };
        let initial = observe().0;
        let initial_auxiliary = controller.auxiliary_usage().unwrap();
        let initial_selection = Usage::btree::<om9_core::identity::ObjectId, ()>(1).unwrap().bytes
            + Usage::array::<om9_core::identity::ObjectId>(controller.selection_order.capacity()).unwrap().bytes;
        controller.select_objects(&ids).unwrap();
        let live_selection = Usage::btree::<om9_core::identity::ObjectId, ()>(ids.len()).unwrap().bytes
            + Usage::array::<om9_core::identity::ObjectId>(controller.selection_order.capacity()).unwrap().bytes;
        let after_growth = observe().0;
        assert_eq!(after_growth - initial, live_selection - initial_selection,
            "detached admission must see current selection storage, not capture-time auxiliary bytes");
        assert_eq!(controller.auxiliary_usage().unwrap(), initial_auxiliary,
            "registered selection must not also be charged in caller-held auxiliary");
        controller.clear_selection();
        let retained_order = Usage::array::<om9_core::identity::ObjectId>(controller.selection_order.capacity()).unwrap().bytes;
        assert_eq!(observe().0,
            initial - initial_selection + retained_order,
            "clearing releases tree nodes but keeps the actual order Vec capacity charged");
        drop(controller);
        assert_eq!(observe().1, 0,
            "a detached source capture must not keep the controller selection alive");
        let capture = worker.join().unwrap();
        assert_eq!(capture.document.entities.len(), ids.len());
        assert_eq!(capture.selection, vec![mesh_id]);
    }

    #[test]
    fn captured_stl_peer_preserves_existing_account_headroom() {
        use om9_core::budget::memory::MemoryClass;
        const MIB: u64 = 1024 * 1024;
        let mut document = om9_core::model::Document::with_template_tolerance("STL peer headroom");
        let entity = Entity::new("triangle", Geometry::Mesh(Mesh {
            vertices: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
            faces: vec![vec![0, 1, 2]], normals: vec![], colors: vec![], corner_uvs: vec![],
        }), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        let mut controller = Controller::new(document, Limits { max_bytes: 32 * MIB, ..Limits::default() }).unwrap();
        controller.select_objects(&[id]).unwrap();
        let reference = controller.coordinator().memory_admission_peer().unwrap();
        let before = controller.coordinator().snapshot();
        let capture = controller.capture_stl_selected().unwrap();
        assert_eq!(controller.coordinator().snapshot(), before);
        let held = controller.scene_cache_usage().unwrap();
        let physical = Some((1024 * MIB, 1024 * MIB));
        let capacity = reference.reserve_remaining_external_memory_with_held(MemoryClass::Kernel, physical, held).unwrap();
        let capacity_bytes = capacity.bytes();
        drop(capacity);
        let cache_bytes = controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache);
        // The source, selection, cache allowances and account all still exist.
        // Capturing must not add an unrelated identity ledger to admission.
        let admitted = capture.admission_peer.reserve_external_memory_with_held(
            MemoryClass::Kernel, capacity_bytes, physical, held,
        ).expect("a capture peer must admit the same request as its shared-account peer");
        assert_eq!(admitted.bytes(), capacity_bytes);
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache), cache_bytes,
            "the equivalent request must not reclaim cache to fund an unrelated peer ledger");
        assert_eq!(capture.admission_peer.authority(), controller.coordinator().authority());
        assert!(capture.admission_peer.world().documents.is_empty());
        assert_eq!(capture.admission_peer.undo_len(), 0);
        assert_eq!(capture.admission_peer.redo_len(), 0);
        drop(admitted);
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel), 0);
    }

    #[test]
    fn captured_stl_peers_share_current_registry_after_history_changes() {
        use om9_core::budget::memory::MemoryClass;
        let mut document = om9_core::model::Document::with_template_tolerance("STL peer history");
        let entity = Entity::new("triangle", Geometry::Mesh(Mesh {
            vertices: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
            faces: vec![vec![0, 1, 2]], normals: vec![], colors: vec![], corner_uvs: vec![],
        }), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        let mut controller = Controller::new(document, Limits::default()).unwrap();
        controller.select_objects(&[id]).unwrap();
        let first = controller.capture_stl_selected().unwrap();
        controller.submit("Point").unwrap();
        controller.submit("2,3,4").unwrap();
        controller.undo().unwrap();
        controller.redo().unwrap();
        controller.select_objects(&[id]).unwrap();
        let second = controller.capture_stl_selected().unwrap();
        assert_eq!(first.document.entities.len(), 1);
        assert_eq!(second.document.entities.len(), 2);
        assert!(first.source_epoch() < second.source_epoch());
        let reference = controller.coordinator().memory_admission_peer().unwrap();
        let held = controller.scene_cache_usage().unwrap();
        let expected = reference.retained_usage_with_held_usage(held).unwrap();
        assert_eq!(first.admission_peer.retained_usage_with_held_usage(held).unwrap(), expected);
        assert_eq!(second.admission_peer.retained_usage_with_held_usage(held).unwrap(), expected);
        let source = controller.coordinator().capture_retained_document(controller.document().id).unwrap();
        assert!(std::ptr::eq::<om9_core::model::Document>(&*second.document, &*source));
        let target = std::env::temp_dir().join(format!("om9-stl-peer-history-{}.stl", uuid::Uuid::new_v4()));
        let receipt = first.export_with_profile(&target,
            StlExportProfile { linear_tolerance_mm: 0.001, angular_tolerance_rad: 0.3 }).unwrap();
        assert_eq!(receipt.triangles, 1);
        assert_eq!(receipt.source_epoch, first.source_epoch());
        assert_eq!(controller.document().entities.len(), 2);
        drop(first);
        let lease = second.admission_peer.reserve_external_memory_with_held(MemoryClass::Kernel, 4096, None, held).unwrap();
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel), 4096);
        drop(lease);
        drop(second);
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Export), 0);
        std::fs::remove_file(target).unwrap();
    }

    #[test]
    fn captured_selection_is_reserved_until_capture_is_dropped() {
        use om9_core::budget::memory::MemoryClass;
        let limits = Limits::default();
        let mut document = om9_core::model::Document::with_template_tolerance("selection ownership");
        let mesh = Mesh { vertices: vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]],
            faces: vec![vec![0,1,2]], normals: vec![], colors: vec![], corner_uvs: vec![] };
        let entity = Entity::new("mesh", Geometry::Mesh(mesh), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        let mut controller = Controller::new(document, limits).unwrap();
        controller.select_objects(&[id]).unwrap();
        let before = controller.coordinator().snapshot();
        let guard = controller.coordinator().reserve_remaining_external_memory_with_held(
            MemoryClass::CpuCache, None, controller.scene_cache_usage().unwrap(),
        ).unwrap();
        assert!(matches!(controller.capture_stl_selected(), Err(Error::Budget(_))));
        assert_eq!(controller.coordinator().snapshot(), before);
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Export), 0);
        drop(guard);
        let capture = controller.capture_stl_selected().unwrap();
        assert!(controller.coordinator().external_reserved_bytes(MemoryClass::Export)
            >= std::mem::size_of::<om9_core::identity::ObjectId>() as u64);
        drop(capture);
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Export), 0);
        assert_eq!(controller.coordinator().snapshot(), before);
    }

    #[test]
    fn capture_checks_available_physical_ram_before_allocating_selection() {
        use om9_core::budget::memory::MemoryClass;
        const MIB: u64 = 1024 * 1024;
        let mesh = Mesh { vertices: vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]],
            faces: vec![vec![0,1,2]], normals: vec![], colors: vec![], corner_uvs: vec![] };
        let mut document = om9_core::model::Document::with_template_tolerance("capture physical headroom");
        let entity = Entity::new("mesh", Geometry::Mesh(mesh), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        let mut controller = Controller::new(document, Limits::default()).unwrap();
        controller.select_objects(&[id]).unwrap();
        let before = controller.coordinator().snapshot();
        let refused = controller.capture_stl_selected_with_sample(|| Some((1024 * MIB, 256 * MIB)));
        assert!(matches!(refused, Err(Error::Budget("available physical RAM"))));
        assert_eq!(controller.coordinator().snapshot(), before);
        assert!(controller.selection.contains(&id));
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Export), 0);
        let capture = controller.capture_stl_selected_with_sample(|| Some((1024 * MIB, 1024 * MIB))).unwrap();
        assert!(capture._selection_reservation.physical_sampled());
        drop(capture);
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Export), 0);
    }

    #[test]
    fn synchronous_export_reserves_headroom_and_preserves_target_if_unavailable() {
        use om9_core::budget::memory::MemoryClass;
        const MIB: u64 = 1024 * 1024;
        let mesh = Mesh { vertices: vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]],
            faces: vec![vec![0,1,2]], normals: vec![], colors: vec![], corner_uvs: vec![] };
        let mut document = om9_core::model::Document::with_template_tolerance("synchronous STL budget");
        let entity = Entity::new("mesh", Geometry::Mesh(mesh), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        let mut controller = Controller::new(document, Limits::default()).unwrap();
        controller.select_objects(&[id]).unwrap();
        let capture = controller.capture_stl_selected().unwrap();
        let guard = controller.coordinator().reserve_remaining_external_memory_with_held(
            MemoryClass::CpuCache, None, controller.scene_cache_usage().unwrap()).unwrap();
        let target = std::env::temp_dir().join(format!("om9-stl-sync-lease-{}.stl", uuid::Uuid::new_v4()));
        std::fs::write(&target, b"unchanged").unwrap();
        let profile = StlExportProfile { linear_tolerance_mm: 0.01, angular_tolerance_rad: 0.3 };
        let blocked = controller.export_captured_stl_with_sample(capture, &target, profile, || Some((1024 * MIB, 1024 * MIB)));
        assert!(matches!(blocked, Err(Error::Budget(_))));
        assert_eq!(std::fs::read(&target).unwrap(), b"unchanged");
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Export), 0);
        drop(guard);
        let capture = controller.capture_stl_selected().unwrap();
        let receipt = controller.export_captured_stl_with_sample(capture, &target, profile, || Some((1024 * MIB, 1024 * MIB))).unwrap();
        assert_eq!(receipt.triangles, 1);
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Export), 0);
        std::fs::remove_file(target).unwrap();
    }

    #[test]
    fn synchronous_export_shares_worker_gate_and_releases_it_after_failure() {
        const MIB: u64 = 1024 * 1024;
        let mesh = Mesh { vertices: vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]],
            faces: vec![vec![0,1,2]], normals: vec![], colors: vec![], corner_uvs: vec![] };
        let mut document = om9_core::model::Document::with_template_tolerance("STL shared gate");
        let entity = Entity::new("mesh", Geometry::Mesh(mesh), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        let mut controller = Controller::new(document, Limits::default()).unwrap();
        controller.select_objects(&[id]).unwrap();
        let account = controller.coordinator().external_memory_account();
        let worker_permit = account.try_acquire_export().unwrap();
        let target = std::env::temp_dir().join(format!("om9-stl-shared-gate-{}.stl", uuid::Uuid::new_v4()));
        std::fs::write(&target, b"unchanged").unwrap();
        let profile = StlExportProfile { linear_tolerance_mm: 0.01, angular_tolerance_rad: 0.3 };
        let result = controller.export_captured_stl_with_sample(controller.capture_stl_selected().unwrap(), &target, profile, || Some((1024 * MIB, 1024 * MIB)));
        assert!(matches!(result, Err(Error::Invalid("export already active"))));
        let direct = controller.capture_stl_selected().unwrap();
        assert!(matches!(direct.export_with_profile(&target, profile), Err(Error::Invalid("export already active"))));
        assert_eq!(std::fs::read(&target).unwrap(), b"unchanged");
        drop(worker_permit);
        drop(direct);
        let result = controller.export_captured_stl_with_sample(controller.capture_stl_selected().unwrap(), &target, profile, || None);
        assert!(matches!(result, Err(Error::Unsupported(_))));
        drop(account.try_acquire_export().expect("failure must release the export gate"));
        let receipt = controller.export_captured_stl_with_sample(controller.capture_stl_selected().unwrap(), &target, profile, || Some((1024 * MIB, 1024 * MIB))).unwrap();
        assert_eq!(receipt.triangles, 1);
        std::fs::remove_file(target).unwrap();
    }

    #[test]
    fn running_worker_excludes_both_capture_and_synchronous_exports() {
        use om9_core::budget::memory::MemoryClass;
        const MIB: u64 = 1024 * 1024;
        let mesh = Mesh { vertices: vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]],
            faces: vec![vec![0,1,2]], normals: vec![], colors: vec![], corner_uvs: vec![] };
        let mut document = om9_core::model::Document::with_template_tolerance("running STL worker gate");
        let entity = Entity::new("mesh", Geometry::Mesh(mesh), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        let mut controller = Controller::new(document, Limits::default()).unwrap();
        controller.select_objects(&[id]).unwrap();
        let mut worker_capture = controller.capture_stl_selected().unwrap();
        let competing_capture = controller.capture_stl_selected().unwrap();
        let reservation = controller.coordinator().reserve_external_memory(MemoryClass::Export, MIB, Some((1024 * MIB, 1024 * MIB))).unwrap();
        worker_capture.limits.max_bytes = reservation.bytes();
        let permit = controller.coordinator().external_memory_account().try_acquire_export().unwrap();
        let worker_target = std::env::temp_dir().join(format!("om9-worker-gate-{}.stl", uuid::Uuid::new_v4()));
        let target = std::env::temp_dir().join(format!("om9-competitor-gate-{}.stl", uuid::Uuid::new_v4()));
        std::fs::write(&target, b"unchanged").unwrap();
        let profile = StlExportProfile { linear_tolerance_mm: 0.01, angular_tolerance_rad: 0.3 };
        let (started_tx, started_rx) = std::sync::mpsc::sync_channel(1);
        let (release_tx, release_rx) = std::sync::mpsc::sync_channel(1);
        let output = worker_target.clone();
        let mut job = StlExportJob::spawn(reservation, permit, move |cancel| {
            started_tx.send(()).map_err(|_| Error::Canceled)?;
            release_rx.recv().map_err(|_| Error::Canceled)?;
            worker_capture.export_admitted(&output, profile, cancel)
        }).unwrap();
        started_rx.recv_timeout(std::time::Duration::from_secs(10)).unwrap();
        assert!(matches!(competing_capture.export_with_profile(&target, profile), Err(Error::Invalid("export already active"))));
        let blocked = controller.export_captured_stl_with_sample(controller.capture_stl_selected().unwrap(), &target, profile, || Some((1024 * MIB, 1024 * MIB)));
        assert!(matches!(blocked, Err(Error::Invalid("export already active"))));
        assert_eq!(std::fs::read(&target).unwrap(), b"unchanged");
        release_tx.send(()).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let receipt = loop {
            if let Some(result) = job.try_result() { break result.unwrap(); }
            assert!(std::time::Instant::now() < deadline, "worker timed out");
            std::thread::yield_now();
        };
        assert_eq!(receipt.triangles, 1);
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Export),
            competing_capture._selection_reservation.bytes());
        assert_eq!(competing_capture.export_with_profile(&target, profile).unwrap().triangles, 1);
        std::fs::remove_file(target).unwrap();
        std::fs::remove_file(worker_target).unwrap();
    }

    #[test]
    fn worker_rejects_reservation_smaller_than_authored_mesh_arrays() {
        use om9_core::budget::memory::MemoryClass;
        const MIB: u64 = 1024 * 1024;
        let mut document = om9_core::model::Document::with_template_tolerance("mesh RAM floor");
        let mesh = Mesh { vertices: vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]],
            faces: vec![vec![0,1,2]], normals: vec![], colors: vec![], corner_uvs: vec![] };
        let entity = Entity::new("mesh", Geometry::Mesh(mesh), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        let mut controller = Controller::new(document, Limits::default()).unwrap();
        controller.select_objects(&[id]).unwrap();
        let target = std::env::temp_dir().join(format!("om9-stl-small-reservation-{}.stl", uuid::Uuid::new_v4()));
        std::fs::write(&target, b"unchanged").unwrap();
        // This holds the old soup/flag minimum but not the coexisting
        // topology parent/rank arrays for the output triangle.
        let old_floor = (3 * std::mem::size_of::<[f64; 3]>() + std::mem::size_of::<[u32; 3]>() + std::mem::size_of::<bool>()) as u64;
        let reservation = controller.coordinator().reserve_external_memory(MemoryClass::Export, old_floor, Some((1024 * MIB, 1024 * MIB))).unwrap();
        let mut job = StlExportJob::start_with_sample(controller.capture_stl_selected().unwrap(), target.clone(),
            StlExportProfile { linear_tolerance_mm: 0.001, angular_tolerance_rad: 0.3 }, reservation,
            || Some((1024 * MIB, 1024 * MIB))).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let result = loop {
            if let Some(result) = job.try_result() { break result; }
            assert!(std::time::Instant::now() < deadline, "worker timed out");
            std::thread::yield_now();
        };
        let remaining = controller.coordinator().external_reserved_bytes(MemoryClass::Export);
        let contents = std::fs::read(&target).unwrap();
        std::fs::remove_file(target).unwrap();
        assert!(matches!(result, Err(Error::Budget("STL mesh reservation lower bound"))), "the old minimum cannot hold the topology scratch arrays");
        assert_eq!(remaining, 0);
        assert_eq!(contents, b"unchanged");
    }

    #[test]
    fn authored_mesh_lower_bound_counts_each_shared_instance() {
        use om9_core::model::Definition;
        let mut document = om9_core::model::Document::with_template_tolerance("repeated mesh RAM");
        let mesh = Mesh { vertices: vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]],
            faces: vec![vec![0,1,2]], normals: vec![], colors: vec![], corner_uvs: vec![] };
        let definition = Definition::new("mesh", vec![Entity::new("mesh", Geometry::Mesh(mesh), None)]);
        let definition_id = definition.id;
        document.definitions.insert(definition_id, definition);
        let mut ids = Vec::new();
        for x in [0., 10.] {
            let mut entity = Entity::new("instance", Geometry::Instance(definition_id), None);
            entity.placement = Transform::translation([x, 0., 0.]);
            ids.push(entity.id);
            document.entities.insert(entity.id, entity);
        }
        let mut controller = Controller::new(document, Limits::default()).unwrap();
        controller.select_objects(&ids).unwrap();
        let bytes = controller.capture_stl_selected().unwrap().authored_mesh_array_lower_bound(&StlCancellation::default()).unwrap();
        assert_eq!(bytes, 2 * (3 * std::mem::size_of::<[f64; 3]>() + std::mem::size_of::<[u32; 3]>() + std::mem::size_of::<usize>() + std::mem::size_of::<u8>()) as u64);
    }

    #[test]
    fn authored_mesh_lower_bound_counts_both_quad_triangles() {
        let mut document = om9_core::model::Document::with_template_tolerance("quad RAM");
        let mesh = Mesh { vertices: vec![[0.,0.,0.], [1.,0.,0.], [1.,1.,0.], [0.,1.,0.]],
            faces: vec![vec![0,1,2,3]], normals: vec![], colors: vec![], corner_uvs: vec![] };
        let entity = Entity::new("quad", Geometry::Mesh(mesh), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        let mut controller = Controller::new(document, Limits::default()).unwrap();
        controller.select_objects(&[id]).unwrap();
        let bytes = controller.capture_stl_selected().unwrap().authored_mesh_array_lower_bound(&StlCancellation::default()).unwrap();
        assert_eq!(bytes, (4 * std::mem::size_of::<[f64; 3]>() + 2 * (std::mem::size_of::<[u32; 3]>() + std::mem::size_of::<usize>() + std::mem::size_of::<u8>())) as u64);
    }

    #[test]
    fn worker_rechecks_launch_sample_and_preserves_destination_on_failure() {
        use om9_core::budget::memory::MemoryClass;
        const MIB: u64 = 1024 * 1024;
        let mut document = om9_core::model::Document::with_template_tolerance("launch RAM");
        let mesh = Mesh { vertices: vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]],
            faces: vec![vec![0,1,2]], normals: vec![], colors: vec![], corner_uvs: vec![] };
        let entity = Entity::new("mesh", Geometry::Mesh(mesh), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        let mut controller = Controller::new(document, Limits::default()).unwrap();
        controller.select_objects(&[id]).unwrap();
        let before = controller.coordinator().snapshot();
        let target = std::env::temp_dir().join(format!("om9-stl-ram-recheck-{}.stl", uuid::Uuid::new_v4()));
        std::fs::write(&target, b"unchanged").unwrap();
        for launch in [Some((1024 * MIB, 0)), None] {
            let reservation = controller.coordinator().reserve_external_memory(MemoryClass::Export, MIB, Some((1024 * MIB, 1024 * MIB))).unwrap();
            let capture = controller.capture_stl_selected().unwrap();
            let result = StlExportJob::start_with_sample(capture, target.clone(),
                StlExportProfile { linear_tolerance_mm: 0.001, angular_tolerance_rad: 0.3 }, reservation, || launch);
            assert!(result.is_err());
            assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Export), 0);
            assert!(controller.coordinator().external_memory_account().try_acquire_export().is_ok());
            assert_eq!(std::fs::read(&target).unwrap(), b"unchanged");
            assert_eq!(controller.coordinator().snapshot(), before);
        }
        std::fs::remove_file(target).unwrap();
    }

    #[test]
    fn instance_enumeration_checks_cycle_depth_and_cancel() {
        use om9_core::model::Definition;
        let mut document = om9_core::model::Document::new("instance bounds");
        let leaf = Entity::new("point", Geometry::Point([0., 0., 0.]), None);
        let inner = Definition::new("inner", vec![leaf]);
        let inner_id = inner.id;
        document.definitions.insert(inner_id, inner);
        let outer = Definition::new("outer", vec![Entity::new("inner", Geometry::Instance(inner_id), None)]);
        let outer_id = outer.id;
        document.definitions.insert(outer_id, outer);
        let root = Entity::new("root", Geometry::Instance(outer_id), None);
        let root_id = root.id;
        document.entities.insert(root_id, root);
        let token = StlCancellation::default();
        let shallow = Limits { max_depth: 1, ..Limits::default() };
        assert!(matches!(stl_occurrences(&document, [root_id].into_iter(), &shallow, &token, |_, _, _| Ok(())), Err(Error::Budget(_))));
        document.definitions.get_mut(&inner_id).unwrap().members[0].geometry = Geometry::Instance(outer_id);
        assert!(matches!(stl_occurrences(&document, [root_id].into_iter(), &Limits::default(), &token, |_, _, _| Ok(())), Err(Error::Invalid(_))));
        assert!(token.cancel());
        assert!(matches!(stl_occurrences(&document, [root_id].into_iter(), &Limits::default(), &token, |_, _, _| Ok(())), Err(Error::Canceled)));
    }

    #[test]
    fn instance_enumeration_preserves_inherited_visibility() {
        use om9_core::model::Definition;
        let mut document = om9_core::model::Document::new("instance visibility");
        let leaf = Entity::new("point", Geometry::Point([0., 0., 0.]), None);
        let definition = Definition::new("nested", vec![leaf]);
        let definition_id = definition.id;
        document.definitions.insert(definition_id, definition);
        let mut root = Entity::new("root", Geometry::Instance(definition_id), None);
        root.attributes.hidden = true;
        let root_id = root.id;
        document.entities.insert(root_id, root);
        let mut visibility = Vec::new();
        stl_occurrences(&document, [root_id].into_iter(), &Limits::default(), &StlCancellation::default(), |_, _, visible| {
            visibility.push(visible);
            Ok(())
        }).unwrap();
        assert_eq!(visibility, vec![false]);
    }

    #[test]
    fn staged_stl_inspection_detects_damage_truncation_count_and_cancellation() {
        use sha2::{Digest, Sha256};
        let target = std::env::temp_dir().join(format!("om9-stl-inspection-{}.tmp", uuid::Uuid::new_v4()));
        let soup = TriangleSoup::new_f64(vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]], vec![[0,1,2]]);
        let mut encoded = Vec::new();
        write_binary_stl_to(&mut encoded, &soup, StlOptions { strict: false, ..Default::default() }, Some(0.001)).unwrap();
        let hash: [u8; 32] = Sha256::digest(&encoded).into();
        let token = StlCancellation::default();
        std::fs::write(&target, &encoded).unwrap();
        verify_staged_stl(&target, 1, hash, &token).unwrap();

        let mut changed = encoded.clone();
        changed[100] ^= 1;
        std::fs::write(&target, &changed).unwrap();
        assert!(matches!(verify_staged_stl(&target, 1, hash, &token), Err(Error::Invalid("STL staging content mismatch"))));
        let destination = target.with_extension("stl");
        std::fs::write(&destination, b"previous file").unwrap();
        assert!(matches!(publish_staged_stl(&target, &destination, 1, hash, &token), Err(Error::Invalid("STL staging content mismatch"))));
        assert_eq!(std::fs::read(&destination).unwrap(), b"previous file");
        changed = encoded.clone();
        changed[80] = 2;
        std::fs::write(&target, &changed).unwrap();
        assert!(matches!(verify_staged_stl(&target, 1, Sha256::digest(&changed).into(), &token), Err(Error::Invalid("STL staging triangle count"))));
        std::fs::write(&target, &encoded[..encoded.len() - 1]).unwrap();
        assert!(matches!(verify_staged_stl(&target, 1, hash, &token), Err(Error::Invalid("STL staging file length"))));
        std::fs::write(&target, &encoded).unwrap();
        assert!(token.cancel());
        assert!(matches!(verify_staged_stl(&target, 1, hash, &token), Err(Error::Canceled)));
        std::fs::remove_file(target).unwrap();
        std::fs::remove_file(destination).unwrap();
    }

    #[test]
    fn staged_stl_decodes_triangles_even_when_the_digest_matches() {
        use sha2::{Digest, Sha256};
        let path = std::env::temp_dir().join(format!("om9-stl-decoding-{}.tmp", uuid::Uuid::new_v4()));
        let soup = TriangleSoup::new_f64(vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]], vec![[0,1,2]]);
        let mut encoded = Vec::new();
        write_binary_stl_to(&mut encoded, &soup, StlOptions { strict: false, ..Default::default() }, Some(0.001)).unwrap();
        let token = StlCancellation::default();
        let check = |bytes: &[u8]| {
            std::fs::write(&path, bytes).unwrap();
            verify_staged_stl(&path, 1, Sha256::digest(bytes).into(), &token)
        };
        check(&encoded).unwrap();
        let mut invalid = encoded.clone();
        invalid[96..100].copy_from_slice(&f32::NAN.to_le_bytes());
        assert!(matches!(check(&invalid), Err(Error::Invalid("STL staging non-finite triangle"))));
        invalid = encoded.clone();
        let first_vertex: [u8; 12] = invalid[96..108].try_into().unwrap();
        invalid[108..120].copy_from_slice(&first_vertex);
        assert!(matches!(check(&invalid), Err(Error::Invalid("STL staging degenerate triangle"))));
        invalid = encoded.clone();
        invalid[92..96].copy_from_slice(&(-1.0_f32).to_le_bytes());
        assert!(matches!(check(&invalid), Err(Error::Invalid("STL staging triangle normal"))));
        std::fs::remove_file(path).unwrap();
    }

    fn job_coordinator() -> om9_core::transaction::TransactionCoordinator {
        let limits = Limits::default();
        let world = om9_core::transaction::World::new(vec![om9_core::model::Document::new("worker test")], &limits).unwrap();
        om9_core::transaction::TransactionCoordinator::new(world, limits).unwrap()
    }

    #[test]
    fn mesh_base_checks_limits_and_index_overflow_without_allocating() {
        let limits = Limits { max_points: u64::MAX, ..Limits::default() };
        assert_eq!(checked_mesh_base(10, 20, &limits).unwrap(), 10);
        if let Ok(above_u32) = usize::try_from(u64::from(u32::MAX) + 1) {
            assert!(matches!(checked_mesh_base(above_u32 - 1, 1, &limits), Err(Error::Overflow(_))));
        }
        assert!(matches!(checked_mesh_base(usize::MAX, 1, &limits), Err(Error::Overflow(_))));
        let small = Limits { max_points: 5, ..limits };
        assert!(matches!(checked_mesh_base(3, 3, &small), Err(Error::Budget(_))));
    }

    #[test]
    fn mesh_append_checks_cancellation_and_indices_before_changing_output() {
        let mesh = Mesh { vertices: vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]],
            faces: vec![vec![0,1,3]], normals: vec![], colors: vec![], corner_uvs: vec![] };
        let mut points = vec![[9.,9.,9.]];
        let mut triangles = vec![[0,0,0]];
        let token = StlCancellation::default();
        assert!(matches!(append_mesh(&Transform::identity(), &mesh, &mut points, &mut triangles, &Limits::default(), &token), Err(Error::Invalid(_))));
        assert_eq!(points, vec![[9.,9.,9.]]);
        assert_eq!(triangles, vec![[0,0,0]]);
        assert!(token.cancel());
        assert!(matches!(append_mesh(&Transform::identity(), &mesh, &mut points, &mut triangles, &Limits::default(), &token), Err(Error::Canceled)));
        assert_eq!(points.len(), 1);
        assert_eq!(triangles.len(), 1);
    }

    #[test]
    fn mesh_append_rejects_cumulative_triangle_limit_before_mutation() {
        let mesh = Mesh { vertices: vec![[0.,0.,0.], [1.,0.,0.], [1.,1.,0.], [0.,1.,0.]],
            faces: vec![vec![0,1,2,3]], normals: vec![], colors: vec![], corner_uvs: vec![] };
        let mut points = vec![[9.,9.,9.]];
        let mut triangles = vec![[0,0,0]];
        let limits = Limits { max_nodes: 2, ..Limits::default() };
        let token = StlCancellation::default();
        assert!(matches!(append_mesh(&Transform::identity(), &mesh, &mut points, &mut triangles, &limits, &token), Err(Error::Budget(_))));
        assert_eq!(points, vec![[9.,9.,9.]]);
        assert_eq!(triangles, vec![[0,0,0]]);
    }

    #[test]
    fn mesh_growth_checks_old_and_new_buffers_before_mutating_output() {
        let mesh = Mesh { vertices: vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]],
            faces: vec![vec![0,1,2]], normals: vec![], colors: vec![], corner_uvs: vec![] };
        for grow_points in [true, false] {
            let (mut points, mut triangles) = if grow_points {
                (vec![[9.,9.,9.]; 100_000], Vec::with_capacity(4))
            } else {
                let mut points = Vec::with_capacity(16);
                points.push([9.,9.,9.]);
                (points, vec![[0,0,0]; 100_000])
            };
            let held = (points.capacity() * std::mem::size_of::<[f64; 3]>()
                + triangles.capacity() * std::mem::size_of::<[u32; 3]>()) as u64;
            let limits = Limits { max_bytes: held + 128, ..Limits::default() };
            let before = (points.as_ptr(), points.len(), triangles.as_ptr(), triangles.len());
            let result = append_mesh(&Transform::identity(), &mesh, &mut points, &mut triangles, &limits, &StlCancellation::default());
            assert!(matches!(result, Err(Error::Budget(_))), "growth must be refused: {result:?}");
            assert_eq!(before, (points.as_ptr(), points.len(), triangles.as_ptr(), triangles.len()));
        }
    }

    #[test]
    fn mesh_growth_falls_back_to_exact_capacity_when_headroom_is_tight() {
        let points = vec![[9., 9., 9.]];
        let triangles = Vec::new();
        let limits = Limits { max_bytes: 120, ..Limits::default() };
        assert_eq!(admit_mesh_output_growth(&points, &triangles, 3, 1, &limits).unwrap(), (4, 1));
    }

    #[test]
    fn repeated_mesh_appends_use_amortized_capacity_and_keep_geometry() {
        let mesh = Mesh { vertices: vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]],
            faces: vec![vec![0,1,2]], normals: vec![], colors: vec![], corner_uvs: vec![] };
        let mut points = Vec::new();
        let mut triangles = Vec::new();
        let mut point_growths = 0;
        let mut triangle_growths = 0;
        for _ in 0..256 {
            let previous = (points.capacity(), triangles.capacity());
            append_mesh(&Transform::identity(), &mesh, &mut points, &mut triangles, &Limits::default(), &StlCancellation::default()).unwrap();
            point_growths += usize::from(points.capacity() != previous.0);
            triangle_growths += usize::from(triangles.capacity() != previous.1);
        }
        assert!(point_growths <= 12 && triangle_growths <= 12, "growths: {point_growths}/{triangle_growths}");
        assert_eq!(points.len(), 768);
        assert_eq!(triangles.len(), 256);
        assert_eq!(triangles[255], [765, 766, 767]);
        assert_eq!(&points[765..], mesh.vertices.as_slice());
    }

    #[test]
    fn cumulative_triangle_count_checks_boundary_and_overflow() {
        let limits = Limits { max_nodes: 3, ..Limits::default() };
        assert!(check_triangle_count(1, 2, &limits).is_ok());
        assert!(matches!(check_triangle_count(2, 2, &limits), Err(Error::Budget(_))));
        assert!(matches!(check_triangle_count(usize::MAX, 1, &limits), Err(Error::Overflow(_))));
        if usize::BITS > 32 {
            assert!(matches!(check_triangle_count(u32::MAX as usize, 1, &limits), Err(Error::Overflow(_))));
        }
    }

    #[test]
    fn capture_requires_a_registered_scene_before_allocating_export_selection() {
        use om9_core::budget::memory::MemoryClass;
        let mut document = om9_core::model::Document::with_template_tolerance("capture ownership guard");
        let entity = Entity::new("triangle", Geometry::Mesh(Mesh {
            vertices: vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]], faces: vec![vec![0,1,2]],
            normals: vec![], colors: vec![], corner_uvs: vec![],
        }), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        let mut controller = Controller::new(document, Limits::default()).unwrap();
        controller.select_objects(&[id]).unwrap();
        controller.refresh_scene().unwrap();
        let before = controller.coordinator.snapshot();
        let selected = controller.selected().clone();
        assert!(matches!(controller.capture_stl_selected(), Err(Error::Invalid("STL scene ownership"))));
        assert_eq!(controller.coordinator.snapshot(), before);
        assert_eq!(controller.selected(), &selected);
        assert_eq!(controller.coordinator.external_reserved_bytes(MemoryClass::Export), 0);
        controller.share_scene_buffer_owner(controller.scene().unwrap()).unwrap();
        let capture = controller.capture_stl_selected().unwrap();
        assert_eq!(capture.caller_held_at_capture.bytes, controller.auxiliary_usage().unwrap().bytes,
            "registered viewport bytes must not be added to the detached caller allowance");
        drop(capture);
        assert_eq!(controller.coordinator.external_reserved_bytes(MemoryClass::Export), 0);
    }

    #[test]
    fn detached_capture_does_not_retain_a_scene_peak_after_buffers_are_registered() {
        use om9_core::budget::memory::MemoryClass;
        const MIB: u64 = 1024 * 1024;
        let mut vertices = vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]];
        vertices.extend((0..30_000).map(|i| [i as f64, 2., 0.]));
        let mut document = om9_core::model::Document::with_template_tolerance("obsolete scene peak");
        let entity = Entity::new("triangle", Geometry::Mesh(Mesh { vertices, faces: vec![vec![0,1,2]],
            normals: vec![], colors: vec![], corner_uvs: vec![] }), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        let mut controller = Controller::new(document, Limits { max_bytes: 64 * MIB, ..Limits::default() }).unwrap();
        controller.select_objects(&[id]).unwrap();
        let capture = controller.capture_stl_selected().unwrap();
        // This fixture-only path publishes new buffers before registering them.
        // Production publication paths already transfer their buffer owners.
        controller.refresh_scene().unwrap();
        controller.share_scene_buffer_owner(controller.scene().unwrap()).unwrap();
        let scene = controller.scene().unwrap();
        assert_eq!(scene.cache_held_usage().unwrap().bytes, controller.registered_scene_buffer_bytes(scene));
        let held = controller.scene_cache_usage().unwrap();
        let initial = capture.authored_mesh_array_lower_bound(&StlCancellation::default()).unwrap().max(1);
        let mut pressure = capture.admission_peer.reserve_remaining_external_memory_with_held(
            MemoryClass::Kernel, Some((1024 * MIB, 1024 * MIB)), held).unwrap();
        pressure.shrink_to(pressure.bytes().checked_sub(initial).unwrap()).unwrap();
        let target = std::env::temp_dir().join(format!("om9-stl-no-scene-peak-{}.stl", uuid::Uuid::new_v4()));
        std::fs::write(&target, b"existing destination").unwrap();
        let before = controller.coordinator.snapshot();
        let result = capture.export_with_sample_guard(&target,
            StlExportProfile { linear_tolerance_mm: 0.001, angular_tolerance_rad: 0.3 }, &StlCancellation::default(),
            || Some((1024 * MIB, 1024 * MIB)), |_| Err(Error::Canceled));
        assert!(matches!(result, Err(Error::Canceled)), "only current owners should constrain admission: {result:?}");
        assert_eq!(controller.coordinator.snapshot(), before);
        assert_eq!(std::fs::read(&target).unwrap(), b"existing destination");
        std::fs::remove_file(target).unwrap();
    }

    #[test]
    fn detached_export_reconciles_scene_growth_during_admission() {
        use om9_core::{budget::memory::MemoryClass, model::Entity};
        const MIB: u64 = 1024 * 1024;
        let mut vertices = vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]];
        vertices.extend((0..30_000).map(|i| [i as f64, 2., 0.]));
        let mesh = Mesh { vertices, faces: vec![vec![0,1,2]], normals: vec![], colors: vec![], corner_uvs: vec![] };
        let mut document = om9_core::model::Document::with_template_tolerance("scene growth during STL admission");
        let entity = Entity::new("mesh", Geometry::Mesh(mesh), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        let limits = Limits { max_bytes: 64 * MIB, ..Limits::default() };
        let mut controller = Controller::new(document, limits).unwrap();
        controller.select_objects(&[id]).unwrap();
        let capture = controller.capture_stl_selected().unwrap();
        let clipboard = controller.copy_selected_om9().unwrap();
        let initial_scene = {
            let scene = controller.scene().unwrap();
            scene.cache_held_usage().unwrap().bytes - controller.registered_scene_buffer_bytes(scene)
        };
        let initial_registered = controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache);
        let held = capture.caller_held_at_capture;
        let baseline = capture.admission_peer.reserve_remaining_external_memory_with_held(
            MemoryClass::Export, Some((1024 * MIB, 1024 * MIB)), held,
        ).unwrap();
        let baseline_bytes = baseline.bytes();
        drop(baseline);
        let growth = std::cell::Cell::new(0u64);
        let unregistered_growth = std::cell::Cell::new(0u64);
        let target = std::env::temp_dir().join(format!("om9-stl-scene-growth-{}.stl", uuid::Uuid::new_v4()));
        std::fs::write(&target, b"existing destination").unwrap();
        let result = capture.export_with_sample_guard(&target,
            StlExportProfile { linear_tolerance_mm: 0.001, angular_tolerance_rad: 0.3 }, &StlCancellation::default(),
            || {
                controller.paste_om9(&clipboard).unwrap();
                let scene = controller.scene().unwrap();
                let latest = scene.cache_held_usage().unwrap().bytes - controller.registered_scene_buffer_bytes(scene);
                unregistered_growth.set(latest.checked_sub(initial_scene).unwrap());
                let registered = controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache);
                growth.set(unregistered_growth.get() + registered.checked_sub(initial_registered).unwrap());
                Some((1024 * MIB, 1024 * MIB))
            },
            |reservation| {
                assert!(growth.get() > 30_000 * 24, "new scene must grow: {}", growth.get());
                let current = capture.caller_held_at_capture;
                let remaining = capture.admission_peer.reserve_remaining_external_memory_with_held(
                    MemoryClass::Kernel, Some((1024 * MIB, 1024 * MIB)), current)?;
                assert!(remaining.bytes() + reservation.bytes() <= baseline_bytes - growth.get(),
                    "registered scene growth must reduce other available capacity, not inflate the export lease");
                Err(Error::Canceled)
            });
        assert!(matches!(result, Err(Error::Canceled)));
        assert_eq!(std::fs::read(&target).unwrap(), b"existing destination");
        controller.undo().unwrap();
        let scene = controller.scene().unwrap();
        assert_eq!(scene.cache_held_usage().unwrap().bytes, controller.registered_scene_buffer_bytes(scene),
            "Undo must publish a fully owned scene while the old export source lives");
        controller.share_scene_buffer_owner(controller.scene().unwrap()).unwrap();
        drop(capture);
        controller.select_objects(&[id]).unwrap();
        let fresh = controller.capture_stl_selected().unwrap();
        let scene = controller.scene().unwrap();
        assert_eq!(fresh.caller_held_at_capture, controller.scene_cache_usage().unwrap(),
            "a new capture keeps only its current unregistered caller allowance");
        assert_eq!(initial_scene, 0, "published scene metadata and buffers already have owners");
        assert_eq!(unregistered_growth.get(), 0, "growth is charged through the shared registry");
        assert_eq!(scene.cache_held_usage().unwrap().bytes - controller.registered_scene_buffer_bytes(scene), 0);
        std::fs::remove_file(target).unwrap();
    }

    #[test]
    fn detached_export_reserves_its_phases_and_tracks_later_document_versions() {
        use om9_core::{budget::memory::MemoryClass, model::Entity};
        const MIB: u64 = 1024 * 1024;
        let mut vertices = vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]];
        vertices.extend((0..30_000).map(|i| [i as f64, 2., 0.]));
        let mesh = Mesh { vertices, faces: vec![vec![0,1,2]], normals: vec![], colors: vec![], corner_uvs: vec![] };
        let mut document = om9_core::model::Document::with_template_tolerance("detached export accounting");
        let entity = Entity::new("mesh", Geometry::Mesh(mesh), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        let limits = Limits { max_bytes: 32 * MIB, ..Limits::default() };
        let mut controller = Controller::new(document, limits).unwrap();
        controller.select_objects(&[id]).unwrap();
        let capture = controller.capture_stl_selected().unwrap();
        let before = capture.admission_peer.retained_usage().unwrap().bytes;
        controller.submit("Point").unwrap();
        controller.submit("2,3,4").unwrap();
        let after = capture.admission_peer.retained_usage().unwrap().bytes;
        assert!(after > before + 30_000 * 24, "new retained document version must be visible: {before} -> {after}");
        let target = std::env::temp_dir().join(format!("om9-detached-admission-{}.stl", uuid::Uuid::new_v4()));
        std::fs::write(&target, b"existing destination").unwrap();
        let profile = StlExportProfile { linear_tolerance_mm: 0.001, angular_tolerance_rad: 0.3 };
        let cancel = StlCancellation::default();
        let before_export = controller.coordinator().external_reserved_bytes(MemoryClass::Export);
        assert!(matches!(capture.export_with_sample(&target, profile, &cancel, || None), Err(Error::Unsupported(_))));
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Export), before_export);
        drop(controller.coordinator().external_memory_account().try_acquire_export().unwrap());
        assert_eq!(std::fs::read(&target).unwrap(), b"existing destination");
        let held = controller.scene_cache_usage().unwrap();
        let pressure = controller.coordinator().reserve_remaining_external_memory_with_held(MemoryClass::Kernel, None, held).unwrap();
        assert!(matches!(capture.export_with_sample(&target, profile, &cancel, || Some((1024 * MIB, 1024 * MIB))), Err(Error::Budget(_))));
        assert_eq!(std::fs::read(&target).unwrap(), b"existing destination");
        drop(pressure);
        let result = capture.export_with_sample_guard(&target, profile, &cancel, || Some((1024 * MIB, 1024 * MIB)), |reservation| {
            assert!(reservation.bytes() > 0);
            assert!(controller.coordinator().external_reserved_bytes(MemoryClass::Export) >= reservation.bytes());
            let peer = controller.coordinator().reserve_external_memory_with_held(MemoryClass::Kernel, 8 * MIB, None, held)
                .expect("staged STL must leave unused headroom for other work");
            drop(peer);
            Err(Error::Canceled)
        });
        assert!(matches!(result, Err(Error::Canceled)));
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Export), before_export);
        drop(controller.coordinator().external_memory_account().try_acquire_export().unwrap());
        assert_eq!(std::fs::read(&target).unwrap(), b"existing destination");
        let receipt = capture.export_with_sample(&target, profile, &cancel, || Some((1024 * MIB, 1024 * MIB))).unwrap();
        assert_eq!(receipt.triangles, 1, "export remains bound to the captured version");
        std::fs::remove_file(target).unwrap();
    }

    #[test]
    fn staged_stl_allows_edit_and_competing_work_and_preserves_captured_output() {
        use om9_core::{budget::memory::MemoryClass, model::Entity};
        const MIB: u64 = 1024 * 1024;
        let mut document = om9_core::model::Document::with_template_tolerance("STL staged concurrency");
        let entity = Entity::new("triangle", Geometry::Mesh(Mesh { vertices: vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]],
            faces: vec![vec![0,1,2]], normals: vec![], colors: vec![], corner_uvs: vec![] }), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        let limits = Limits { max_bytes: 32 * MIB, ..Default::default() };
        let mut controller = Controller::new(document, limits).unwrap();
        controller.select_objects(&[id]).unwrap();
        let baseline = controller.coordinator().external_reserved_bytes(MemoryClass::Export);
        let capture = controller.capture_stl_selected().unwrap();
        assert_eq!(controller.coordinator().external_pending_bytes(MemoryClass::Export), 0,
            "the captured selection exists and must not remain a pending allocation");
        let profile = StlExportProfile { linear_tolerance_mm: 0.001, angular_tolerance_rad: 0.3 };
        let target = std::env::temp_dir().join(format!("om9-stl-stages-{}.stl", uuid::Uuid::new_v4()));
        let second = target.with_extension("second.stl");
        let (ready, waiting) = std::sync::mpsc::sync_channel(1);
        let (resume, proceed) = std::sync::mpsc::sync_channel(1);
        let worker_target = target.clone();
        let worker = std::thread::spawn(move || capture.export_with_sample_guard(&worker_target, profile, &StlCancellation::default(),
            || Some((1024 * MIB, 1024 * MIB)), |reservation| {
                assert!(reservation.bytes() < MIB);
                ready.send(()).map_err(|e| Error::Io(e.to_string()))?;
                proceed.recv_timeout(std::time::Duration::from_secs(10)).map_err(|e| Error::Io(e.to_string()))?;
                Ok(())
            }));
        waiting.recv_timeout(std::time::Duration::from_secs(10)).unwrap();
        controller.submit("Point").unwrap();
        controller.submit("2,3,4").unwrap();
        controller.undo().unwrap();
        controller.redo().unwrap();
        controller.select_objects(&[id]).unwrap();
        let competing = controller.coordinator().reserve_external_memory_with_held(
            MemoryClass::Kernel, 16 * MIB, None, controller.scene_cache_usage().unwrap()).unwrap();
        resume.send(()).unwrap();
        let receipt = worker.join().unwrap().unwrap();
        assert_eq!(receipt.triangles, 1);
        assert_eq!(controller.document().entities.len(), 2);
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel), 16 * MIB);
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Export), baseline);
        // Controller entrypoint shares this implementation and must produce
        // identical bytes from the same capture, even with peer capacity held.
        controller.export_captured_stl(controller.capture_stl_selected().unwrap(), &second, profile).unwrap();
        assert_eq!(std::fs::read(&target).unwrap(), std::fs::read(&second).unwrap());
        drop(competing);
        std::fs::remove_file(target).unwrap();
        std::fs::remove_file(second).unwrap();
    }

    #[test]
    fn staged_stl_growth_refusal_preserves_destination_history_and_permit() {
        use om9_core::{budget::memory::MemoryClass, model::Entity};
        let mut document = om9_core::model::Document::with_template_tolerance("STL phase refusal");
        let entity = Entity::new("triangle", Geometry::Mesh(Mesh { vertices: vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]],
            faces: vec![vec![0,1,2]], normals: vec![], colors: vec![], corner_uvs: vec![] }), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        let mut controller = Controller::new(document, Limits { max_bytes: 32 * 1024 * 1024, ..Default::default() }).unwrap();
        controller.submit("Point").unwrap(); controller.submit("2,3,4").unwrap(); controller.undo().unwrap();
        controller.select_objects(&[id]).unwrap();
        let capture = controller.capture_stl_selected().unwrap();
        let before = controller.coordinator().snapshot();
        let baseline = controller.coordinator().external_reserved_bytes(MemoryClass::Export);
        let target = std::env::temp_dir().join(format!("om9-stl-stage-refusal-{}.stl", uuid::Uuid::new_v4()));
        std::fs::write(&target, b"existing destination").unwrap();
        let mut pressure = None;
        let result = capture.export_with_sample_guard(&target,
            StlExportProfile { linear_tolerance_mm: 0.001, angular_tolerance_rad: 0.3 }, &StlCancellation::default(),
            || Some((1024 * 1024 * 1024, 1024 * 1024 * 1024)), |_| {
                pressure = Some(controller.coordinator().reserve_remaining_external_memory_with_held(
                    MemoryClass::Kernel, None, controller.scene_cache_usage()?)?);
                Ok(())
            });
        assert!(matches!(result, Err(Error::Budget(_))), "{result:?}");
        assert_eq!(std::fs::read(&target).unwrap(), b"existing destination");
        assert_eq!(controller.coordinator().snapshot(), before);
        assert_eq!(controller.coordinator().redo_len(), 1);
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Export), baseline);
        drop(controller.coordinator().external_memory_account().try_acquire_export().unwrap());
        drop(pressure);
        std::fs::remove_file(target).unwrap();
    }

    #[test]
    fn dropping_controller_cancels_detached_capture_and_preserves_destination() {
        use om9_core::model::Entity;
        let mesh = Mesh { vertices: vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]],
            faces: vec![vec![0,1,2]], normals: vec![], colors: vec![], corner_uvs: vec![] };
        let mut document = om9_core::model::Document::with_template_tolerance("controller dropped");
        let entity = Entity::new("triangle", Geometry::Mesh(mesh), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        let capture = {
            let mut controller = Controller::new(document, Limits::default()).unwrap();
            controller.select_objects(&[id]).unwrap();
            controller.capture_stl_selected().unwrap()
        };
        let target = std::env::temp_dir().join(format!("om9-stl-controller-drop-{}.stl", uuid::Uuid::new_v4()));
        std::fs::write(&target, b"previous destination").unwrap();
        assert!(matches!(capture.export_with_profile(&target,
            StlExportProfile { linear_tolerance_mm: 0.001, angular_tolerance_rad: 0.3 }), Err(Error::Canceled)));
        assert_eq!(std::fs::read(&target).unwrap(), b"previous destination");
        std::fs::remove_file(target).unwrap();
    }

    #[test]
    fn failed_new_keeps_export_session_and_existing_capture_valid() {
        use om9_core::{budget::memory::MemoryClass, model::Entity};
        let mesh = Mesh { vertices: vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]],
            faces: vec![vec![0,1,2]], normals: vec![], colors: vec![], corner_uvs: vec![] };
        let mut document = om9_core::model::Document::with_template_tolerance("failed New");
        let entity = Entity::new("triangle", Geometry::Mesh(mesh), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        let mut controller = Controller::new(document, Limits::default()).unwrap();
        controller.select_objects(&[id]).unwrap();
        let capture = controller.capture_stl_selected().unwrap();
        let held = controller.scene_cache_usage().unwrap();
        let pressure = controller.coordinator().reserve_remaining_external_memory_with_held(MemoryClass::Kernel, None, held).unwrap();
        assert!(controller.begin_file_operation(crate::file_lifecycle::FileKind::New).is_err());
        assert!(capture.belongs_to_session(controller.document()));
        drop(pressure);
        let target = std::env::temp_dir().join(format!("om9-stl-failed-new-{}.stl", uuid::Uuid::new_v4()));
        capture.export_with_profile(&target, StlExportProfile { linear_tolerance_mm: 0.001, angular_tolerance_rad: 0.3 }).unwrap();
        std::fs::remove_file(target).unwrap();
    }

    #[test]
    fn document_restore_invalidates_detached_stl_capture() {
        use om9_core::model::Entity;
        for master in [false, true] {
            let mesh = Mesh { vertices: vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]],
                faces: vec![vec![0,1,2]], normals: vec![], colors: vec![], corner_uvs: vec![] };
            let mut document = om9_core::model::Document::with_template_tolerance("STL restore");
            let entity = Entity::new("triangle", Geometry::Mesh(mesh), None);
            let id = entity.id;
            document.entities.insert(id, entity);
            let mut controller = Controller::new(document, Limits::default()).unwrap();
            controller.select_objects(&[id]).unwrap();
            let capture = controller.capture_stl_selected().unwrap();
            if master {
                controller.restore_master_document(controller.document().clone()).unwrap();
            } else {
                let snapshot = om9_archive::DocumentSnapshot::capture(controller.document(), &Limits::default()).unwrap();
                controller.restore_snapshot(&snapshot).unwrap();
            }
            assert!(!capture.belongs_to_session(controller.document()));
            let target = std::env::temp_dir().join(format!("om9-stl-restore-{}.stl", uuid::Uuid::new_v4()));
            std::fs::write(&target, b"existing destination").unwrap();
            assert!(matches!(capture.export_with_profile(&target,
                StlExportProfile { linear_tolerance_mm: 0.001, angular_tolerance_rad: 0.3 }), Err(Error::Canceled)));
            assert_eq!(std::fs::read(&target).unwrap(), b"existing destination");
            std::fs::remove_file(target).unwrap();
        }
    }

    fn invalidate_between_stl_verification_and_publication_preserves_target(close: bool) {
        use om9_core::{budget::memory::MemoryClass, model::Entity};
        use sha2::{Digest, Sha256};
        use std::sync::mpsc;
        let mesh = Mesh { vertices: vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]],
            faces: vec![vec![0,1,2]], normals: vec![], colors: vec![], corner_uvs: vec![] };
        let mut document = om9_core::model::Document::with_template_tolerance("STL replacement race");
        let entity = Entity::new("triangle", Geometry::Mesh(mesh), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        let mut controller = Controller::new(document, Limits::default()).unwrap();
        controller.select_objects(&[id]).unwrap();
        let capture = controller.capture_stl_selected().unwrap();
        let soup = TriangleSoup::new_f64(vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]], vec![[0,1,2]]);
        let bytes = om9_exchange::stl::write_binary_stl(&soup, StlOptions { strict: false, ..Default::default() }).unwrap();
        let digest: [u8; 32] = Sha256::digest(&bytes).into();
        let target = std::env::temp_dir().join(format!("om9-stl-session-target-{}.stl", uuid::Uuid::new_v4()));
        let staging = target.with_extension("stage");
        std::fs::write(&target, b"existing destination").unwrap();
        std::fs::write(&staging, &bytes).unwrap();
        let reservation = controller.coordinator().reserve_external_memory(MemoryClass::Export, 4096, None).unwrap();
        let permit = controller.coordinator().external_memory_account().try_acquire_export().unwrap();
        let (verified_tx, verified_rx) = mpsc::channel();
        let (resume_tx, resume_rx) = mpsc::channel();
        let worker_target = target.clone();
        let worker_staging = staging.clone();
        let mut job = StlExportJob::spawn(reservation, permit, move |cancel| {
            let capture = capture;
            let cancel = cancel.with_session(capture.session.clone());
            publish_staged_stl_guard(&worker_staging, &worker_target, 1, digest, &cancel, || {
                verified_tx.send(()).unwrap();
                resume_rx.recv().unwrap();
            })?;
            Err(Error::Invalid("unexpected publication"))
        }).unwrap();
        verified_rx.recv_timeout(std::time::Duration::from_secs(10)).unwrap();
        if close {
            controller.invalidate_stl_session_for_close();
        } else {
            let outcome = controller.begin_file_operation(crate::file_lifecycle::FileKind::New).unwrap();
            if outcome == crate::file_lifecycle::FileOutcome::Pending {
                let token = controller.file_operation().unwrap().token();
                controller.decide_unsaved(token, crate::file_lifecycle::UnsavedDecision::Discard).unwrap();
            }
        }
        resume_tx.send(()).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let result = loop {
            if let Some(result) = job.try_result() { break result; }
            assert!(std::time::Instant::now() < deadline, "STL worker did not finish");
            std::thread::yield_now();
        };
        assert!(matches!(result, Err(Error::Canceled)));
        assert_eq!(std::fs::read(&target).unwrap(), b"existing destination");
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Export), 0);
        std::fs::remove_file(staging).unwrap();
        std::fs::remove_file(target).unwrap();
    }

    #[test]
    fn new_document_between_stl_verification_and_publication_preserves_target() {
        invalidate_between_stl_verification_and_publication_preserves_target(false);
    }

    #[test]
    fn close_between_stl_verification_and_publication_preserves_target() {
        invalidate_between_stl_verification_and_publication_preserves_target(true);
    }

    #[test]
    fn dropped_job_cancels_but_holds_reservation_until_worker_stops() {
        use om9_core::budget::memory::MemoryClass;
        use std::sync::mpsc;
        let coordinator = job_coordinator();
        let reservation = coordinator.reserve_external_memory(MemoryClass::Export, 4096, None).unwrap();
        let account = coordinator.external_memory_account();
        let permit = account.try_acquire_export().unwrap();
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let (done_tx, done_rx) = mpsc::channel();
        let job = StlExportJob::spawn(reservation, permit, move |cancel| {
            started_tx.send(()).unwrap();
            release_rx.recv().unwrap();
            let canceled = cancel.is_canceled();
            done_tx.send(canceled).unwrap();
            Err(Error::Canceled)
        }).unwrap();
        started_rx.recv_timeout(std::time::Duration::from_secs(10)).unwrap();
        drop(job);
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::Export), 4096);
        assert!(account.try_acquire_export().is_err(), "cancellation must not free a running worker slot");
        release_tx.send(()).unwrap();
        assert!(done_rx.recv_timeout(std::time::Duration::from_secs(10)).unwrap());
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while coordinator.external_reserved_bytes(MemoryClass::Export) != 0 {
            assert!(std::time::Instant::now() < deadline, "worker reservation was not released");
            std::thread::yield_now();
        }
        loop {
            if let Ok(permit) = account.try_acquire_export() { drop(permit); break; }
            assert!(std::time::Instant::now() < deadline, "worker slot was not released");
            std::thread::yield_now();
        }
    }

    #[test]
    fn worker_panic_returns_error_and_releases_reservation() {
        use om9_core::budget::memory::MemoryClass;
        let coordinator = job_coordinator();
        let reservation = coordinator.reserve_external_memory(MemoryClass::Export, 4096, None).unwrap();
        let account = coordinator.external_memory_account();
        let permit = account.try_acquire_export().unwrap();
        let mut job = StlExportJob::spawn(reservation, permit, |_| panic!("simulated worker failure")).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let result = loop {
            if let Some(result) = job.try_result() { break result; }
            assert!(std::time::Instant::now() < deadline, "worker did not report failure");
            std::thread::yield_now();
        };
        assert!(matches!(result, Err(Error::Io(message)) if message.contains("worker panicked")));
        assert_eq!(coordinator.external_reserved_bytes(MemoryClass::Export), 0);
        assert!(account.try_acquire_export().is_ok());
    }

    #[test]
    fn cancelled_soup_finalization_refuses_work_before_allocating_output() {
        let token = StlCancellation::default();
        assert!(token.cancel());
        assert!(matches!(finalize_soup(vec![[0.,0.,0.]], vec![], &token), Err(Error::Canceled)));
    }

    #[test]
    fn cancel_during_streaming_preserves_destination_and_removes_temporary_file() {
        let directory = std::env::temp_dir().join(format!("om9-stl-cancel-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&directory).unwrap();
        let target = directory.join("target.stl");
        std::fs::write(&target, b"old destination").unwrap();
        let token = StlCancellation::default();
        let worker_token = token.clone();
        let worker_target = target.clone();
        let worker = std::thread::spawn(move || {
            let soup = TriangleSoup::new_f64(vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]], vec![[0,1,2]; 200_000]);
            write_stl_atomic(&worker_target, &soup, StlOptions { strict: false, ..StlOptions::default() }, 0.001, &Limits::default(), &worker_token)
        });
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while std::fs::read_dir(&directory).unwrap().count() < 2 && !worker.is_finished() && std::time::Instant::now() < deadline {
            std::thread::yield_now();
        }
        let temporary_was_created = std::fs::read_dir(&directory).unwrap().count() == 2;
        assert!(token.cancel(), "cancellation must happen before publication");
        assert_eq!(worker.join().unwrap(), Err(Error::Canceled));
        assert!(temporary_was_created, "test must cancel after temporary file creation");
        assert_eq!(std::fs::read(&target).unwrap(), b"old destination");
        assert_eq!(std::fs::read_dir(&directory).unwrap().count(), 1);
        std::fs::remove_file(target).unwrap();
        std::fs::remove_dir(directory).unwrap();
    }

    #[test]
    fn encoded_position_budget_failure_preserves_destination_and_removes_temporary_file() {
        let directory = std::env::temp_dir().join(format!("om9-stl-encoded-budget-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&directory).unwrap();
        let target = directory.join("target.stl");
        std::fs::write(&target, b"old destination").unwrap();
        let soup = TriangleSoup::new_f64(vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]], vec![[0,1,2]]);
        let held = (soup.positions.capacity() * std::mem::size_of::<[f64; 3]>() + soup.triangles.capacity() * std::mem::size_of::<[u32; 3]>()) as u64;
        let temporary = directory.join(".om9-stl-00000000-0000-0000-0000-000000000000.tmp");
        let limits = Limits { max_bytes: held + 8 * 1024 + temporary.capacity() as u64 + 1, ..Limits::default() };
        let result = write_stl_atomic(&target, &soup, StlOptions { strict: false, ..Default::default() }, 0.001, &limits, &StlCancellation::default());
        assert!(matches!(result, Err(Error::Budget(_))));
        assert_eq!(std::fs::read(&target).unwrap(), b"old destination");
        assert_eq!(std::fs::read_dir(&directory).unwrap().count(), 1);
        std::fs::remove_file(target).unwrap();
        std::fs::remove_dir(directory).unwrap();
    }

    #[test]
    fn stl_writer_buffer_budget_failure_preserves_destination_without_temporary_file() {
        let directory = std::env::temp_dir().join(format!("om9-stl-buffer-budget-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&directory).unwrap();
        let target = directory.join("target.stl");
        std::fs::write(&target, b"old destination").unwrap();
        let soup = TriangleSoup::new_f64(vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]], vec![[0,1,2]]);
        let held = (soup.positions.capacity() * std::mem::size_of::<[f64; 3]>() + soup.triangles.capacity() * std::mem::size_of::<[u32; 3]>()) as u64;
        let limits = Limits { max_bytes: held + 8 * 1024 - 1, ..Limits::default() };
        let result = write_stl_atomic(&target, &soup, StlOptions { strict: false, ..Default::default() }, 0.001, &limits, &StlCancellation::default());
        assert!(matches!(result, Err(Error::Budget(_))));
        assert_eq!(std::fs::read(&target).unwrap(), b"old destination");
        assert_eq!(std::fs::read_dir(&directory).unwrap().count(), 1);
        std::fs::remove_file(target).unwrap();
        std::fs::remove_dir(directory).unwrap();
    }

    #[test]
    fn stl_temporary_path_budget_failure_does_not_create_file() {
        let directory = std::env::temp_dir().join(format!("om9-stl-path-budget-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&directory).unwrap();
        let target = directory.join("target.stl");
        std::fs::write(&target, b"old destination").unwrap();
        let soup = TriangleSoup::new_f64(vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]], vec![[0,1,2]]);
        let held = (soup.positions.capacity() * std::mem::size_of::<[f64; 3]>() + soup.triangles.capacity() * std::mem::size_of::<[u32; 3]>()) as u64;
        let limits = Limits { max_bytes: held + 8 * 1024, ..Limits::default() };
        let result = write_stl_atomic(&target, &soup, StlOptions { strict: false, ..Default::default() }, 0.001, &limits, &StlCancellation::default());
        assert!(matches!(result, Err(Error::Budget(_))));
        assert_eq!(std::fs::read(&target).unwrap(), b"old destination");
        assert_eq!(std::fs::read_dir(&directory).unwrap().count(), 1);
        std::fs::remove_file(target).unwrap();
        std::fs::remove_dir(directory).unwrap();
    }

    #[test]
    fn quantization_collision_preserves_destination_and_removes_temporary_file() {
        let directory = std::env::temp_dir().join(format!("om9-stl-collision-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&directory).unwrap();
        let target = directory.join("target.stl");
        std::fs::write(&target, b"old destination").unwrap();
        let soup = TriangleSoup::new_f64(
            vec![[0., 0., 1.], [1., 0., 1.], [0., 1., 1.],
                 [0., 0., 1.00000002], [1., 0., 1.00000002], [0., 1., 1.00000002]],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let result = write_stl_atomic(&target, &soup, StlOptions { strict: false, ..Default::default() }, 0.0001, &Limits::default(), &StlCancellation::default());
        assert!(matches!(result, Err(Error::Invalid(message)) if message.contains("merges distinct vertices")));
        assert_eq!(std::fs::read(&target).unwrap(), b"old destination");
        assert_eq!(std::fs::read_dir(&directory).unwrap().count(), 1);
        std::fs::remove_file(target).unwrap();
        std::fs::remove_dir(directory).unwrap();
    }

    #[test]
    fn cancellation_after_publication_reports_it_was_too_late() {
        let target = std::env::temp_dir().join(format!("om9-stl-published-{}.stl", uuid::Uuid::new_v4()));
        let token = StlCancellation::default();
        let soup = TriangleSoup::new_f64(vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]], vec![[0,1,2]]);
        write_stl_atomic(&target, &soup, StlOptions { strict: false, ..StlOptions::default() }, 0.001, &Limits::default(), &token).unwrap();
        assert!(!token.cancel());
        assert_eq!(std::fs::metadata(&target).unwrap().len(), 134);
        std::fs::remove_file(target).unwrap();
    }
}

#[cfg(test)]
mod quad_tests {
    use super::quad_diagonal_ac;

    #[test]
    fn concave_quad_diagonal_is_stable_across_axes_and_winding() {
        let points = [[0.,0.,0.], [1.,0.5,0.], [2.,0.,0.], [0.,2.,0.]];
        for rotate in 0..3 {
            let p = points.map(|p| [p[rotate], p[(rotate + 1) % 3], p[(rotate + 2) % 3]]);
            assert!(!quad_diagonal_ac(p).unwrap());
            assert!(!quad_diagonal_ac([p[0], p[3], p[2], p[1]]).unwrap());
        }
    }
}

fn stl_target(target: &Path) -> Result<PathBuf> {
    let encoded = target.as_os_str().as_encoded_bytes();
    if encoded.is_empty() || encoded.contains(&0) {
        return Err(Error::Invalid("STL file path"));
    }
    if encoded.len() > 32768 {
        return Err(Error::Budget("STL file path bytes"));
    }
    if !target
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("stl"))
    {
        return Err(Error::Unsupported("selected file codec; choose .stl"));
    }
    if target.file_name().is_none() {
        return Err(Error::Invalid("STL file name"));
    }
    let path = std::path::absolute(target).map_err(|error| Error::Io(error.to_string()))?;
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    if !parent.is_dir() {
        return Err(Error::Io("STL destination directory".into()));
    }
    Ok(path)
}

/// Decode and check each staged triangle, then compare the file to the encoded stream.
/// This does not certify source deviation, self-intersection or encoded topology.
fn verify_staged_stl(path: &Path, triangles: usize, expected_hash: [u8; 32], cancel: &StlCancellation) -> Result<()> {
    use sha2::{Digest, Sha256};
    use std::io::Read;
    cancel.check()?;
    let count = u32::try_from(triangles).map_err(|_| Error::Overflow("STL triangle count"))?;
    let expected_len = u64::from(count).checked_mul(50).and_then(|n| n.checked_add(84))
        .ok_or(Error::Overflow("STL file size"))?;
    let file = std::fs::File::open(path).map_err(|error| Error::Io(error.to_string()))?;
    if file.metadata().map_err(|error| Error::Io(error.to_string()))?.len() != expected_len {
        return Err(Error::Invalid("STL staging file length"));
    }
    // Records are already read in 16,000-byte blocks below; an additional
    // heap-allocated BufReader is unnecessary for this sequential pass.
    let mut reader = file;
    let mut header = [0_u8; 84];
    reader.read_exact(&mut header).map_err(|error| Error::Io(error.to_string()))?;
    if u32::from_le_bytes(header[80..84].try_into().expect("four-byte STL count")) != count {
        return Err(Error::Invalid("STL staging triangle count"));
    }
    let mut digest = Sha256::new();
    digest.update(header);
    // Whole 50-byte records allow validation and hashing without retaining the STL.
    let mut buffer = [0_u8; 50 * 320];
    let mut remaining = count as usize;
    while remaining > 0 {
        cancel.check()?;
        let records = remaining.min(buffer.len() / 50);
        let bytes = &mut buffer[..records * 50];
        reader.read_exact(bytes).map_err(|error| Error::Io(error.to_string()))?;
        digest.update(&*bytes);
        for record in bytes.chunks_exact(50) {
            let read_float = |offset: usize| f64::from(f32::from_le_bytes(record[offset..offset + 4].try_into().expect("four-byte STL float")));
            let normal = [read_float(0), read_float(4), read_float(8)];
            let vertices: [[f64; 3]; 3] = std::array::from_fn(|i| std::array::from_fn(|axis| read_float(12 + i * 12 + axis * 4)));
            if normal.iter().chain(vertices.iter().flatten()).any(|value| !value.is_finite()) {
                return Err(Error::Invalid("STL staging non-finite triangle"));
            }
            let ab: [f64; 3] = std::array::from_fn(|axis| vertices[1][axis] - vertices[0][axis]);
            let ac: [f64; 3] = std::array::from_fn(|axis| vertices[2][axis] - vertices[0][axis]);
            let cross = [ab[1] * ac[2] - ab[2] * ac[1], ab[2] * ac[0] - ab[0] * ac[2], ab[0] * ac[1] - ab[1] * ac[0]];
            let area = cross[0].hypot(cross[1]).hypot(cross[2]);
            let normal_length = normal[0].hypot(normal[1]).hypot(normal[2]);
            if !area.is_finite() || area == 0. || !normal_length.is_finite() || normal_length == 0. {
                return Err(Error::Invalid("STL staging degenerate triangle"));
            }
            let alignment = (0..3).map(|axis| (cross[axis] / area) * (normal[axis] / normal_length)).sum::<f64>();
            if !alignment.is_finite() || alignment <= 0. {
                return Err(Error::Invalid("STL staging triangle normal"));
            }
        }
        remaining -= records;
    }
    let mut trailing = [0_u8; 1];
    if reader.read(&mut trailing).map_err(|error| Error::Io(error.to_string()))? != 0 {
        return Err(Error::Invalid("STL staging file length"));
    }
    if <[u8; 32]>::from(digest.finalize()) != expected_hash {
        return Err(Error::Invalid("STL staging content mismatch"));
    }
    Ok(())
}

fn publish_staged_stl(temporary: &Path, target: &Path, triangles: usize, digest: [u8; 32], cancel: &StlCancellation) -> Result<()> {
    publish_staged_stl_guard(temporary, target, triangles, digest, cancel, || {})
}

fn publish_staged_stl_guard(temporary: &Path, target: &Path, triangles: usize, digest: [u8; 32], cancel: &StlCancellation, verified: impl FnOnce()) -> Result<()> {
    verify_staged_stl(temporary, triangles, digest, cancel)?;
    verified();
    let session = cancel.session.as_ref().map(|session| session.lock().unwrap_or_else(|e| e.into_inner()));
    let mut published = cancel.state.published.lock().unwrap_or_else(|e| e.into_inner());
    if cancel.state.canceled.load(Ordering::Acquire) || session.as_ref().is_some_and(|session| !**session) {
        return Err(Error::Canceled);
    }
    std::fs::rename(temporary, target).map_err(|error| Error::Io(error.to_string()))?;
    *published = true;
    Ok(())
}

#[cfg(test)]
fn write_stl_atomic(path: &Path, soup: &TriangleSoup, options: StlOptions, max_quantization: f64, limits: &Limits, cancel: &StlCancellation) -> Result<StlWriteReport> {
    write_stl_atomic_admitted(path, soup, options, max_quantization, limits, cancel, &mut |_, _| Ok(()))
}
fn write_stl_atomic_admitted(path: &Path, soup: &TriangleSoup, options: StlOptions, max_quantization: f64, limits: &Limits, cancel: &StlCancellation,
    admit: &mut StlAdmission<'_>) -> Result<StlWriteReport> {
    use std::io::Write;
    use sha2::{Digest, Sha256};
    struct TempFile(Option<PathBuf>);
    impl Drop for TempFile {
        fn drop(&mut self) { if let Some(path) = self.0.take() { let _ = std::fs::remove_file(path); } }
    }
    struct CancellableWriter<'a, W> { inner: W, cancel: &'a StlCancellation }
    impl<W: Write> Write for CancellableWriter<'_, W> {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if self.cancel.is_canceled() { return Err(std::io::Error::other("STL export canceled")); }
            self.inner.write(bytes)
        }
        fn flush(&mut self) -> std::io::Result<()> { self.inner.flush() }
    }
    // Hash buffered file writes, rather than each small scalar sent by the STL encoder.
    struct DigestWriter<W> { inner: W, digest: Sha256 }
    impl<W: Write> Write for DigestWriter<W> {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            let written = self.inner.write(bytes)?;
            self.digest.update(&bytes[..written]);
            Ok(written)
        }
        fn flush(&mut self) -> std::io::Result<()> { self.inner.flush() }
    }
    const WRITE_BUFFER_BYTES: usize = 8 * 1024;
    cancel.check()?;
    let mut held = om9_core::budget::Usage::array::<[f64; 3]>(soup.positions.capacity())?;
    held.add(om9_core::budget::Usage::array::<[u32; 3]>(soup.triangles.capacity())?, limits)?;
    held.add(om9_core::budget::Usage::array::<u8>(WRITE_BUFFER_BYTES)?, limits)?;
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let filename_bytes = ".om9-stl-".len() + 36 + ".tmp".len();
    let path_growth = stl_sum(parent.as_os_str().as_encoded_bytes().len() as u64,
        (1 + 2 * filename_bytes) as u64)?;
    admit(stl_soup_bytes(&soup.positions, &soup.triangles)?, stl_sum(WRITE_BUFFER_BYTES as u64, path_growth)?)?;
    let temporary = parent.join(format!(".om9-stl-{}.tmp", uuid::Uuid::new_v4()));
    // The cleanup guard owns this same path; do not keep a second clone live.
    held.add(om9_core::budget::Usage { bytes: temporary.capacity() as u64, ..Default::default() }, limits)?;
    let file = std::fs::OpenOptions::new().write(true).create_new(true).open(&temporary)
        .map_err(|error| Error::Io(error.to_string()))?;
    let mut cleanup = TempFile(Some(temporary));
    let result = (|| -> Result<([u8; 32], StlWriteReport)> {
        let mut writer = std::io::BufWriter::with_capacity(WRITE_BUFFER_BYTES, DigestWriter { inner: file, digest: Sha256::new() });
        if writer.capacity() > WRITE_BUFFER_BYTES {
            held.add(om9_core::budget::Usage::array::<u8>(writer.capacity() - WRITE_BUFFER_BYTES)?, limits)?;
        }
        admit(held.bytes, 0)?;
        let mut admission_error = None;
        let report = {
            let mut checked = CancellableWriter { inner: &mut writer, cancel };
            write_binary_stl_to_with_report_and_admission(&mut checked, soup, options, Some(max_quantization), || {
                if cancel.is_canceled() { Err(std::io::Error::other("STL export canceled")) } else { Ok(()) }
            }, |bytes| {
                let mut combined = held;
                let result = combined.add(om9_core::budget::Usage { bytes, ..Default::default() }, limits)
                    .and_then(|()| admit(held.bytes, bytes));
                if let Err(error) = result { admission_error = Some(error); return Err(om9_exchange::stl::StlError::Allocation); }
                Ok(())
            }).map_err(|error| admission_error.take().unwrap_or_else(|| match error {
                StlStreamError::Geometry(error) => Error::from(error),
                StlStreamError::Io(_) if cancel.is_canceled() => Error::Canceled,
                StlStreamError::Io(error) => Error::Io(error.to_string()),
            }))?
        };
        cancel.check()?;
        writer.flush().map_err(|error| Error::Io(error.to_string()))?;
        writer.get_ref().inner.sync_all().map_err(|error| Error::Io(error.to_string()))?;
        Ok((writer.get_ref().digest.clone().finalize().into(), report))
    })();
    let (digest, report) = result?;
    admit(stl_sum(stl_soup_bytes(&soup.positions, &soup.triangles)?, cleanup.0.as_ref().expect("temporary path").capacity() as u64)?, 0)?;
    publish_staged_stl(cleanup.0.as_deref().expect("temporary STL path"), path, soup.triangles.len(), digest, cancel)?;
    cleanup.0 = None;
    Ok(report)
}
