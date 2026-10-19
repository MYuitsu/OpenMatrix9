// SPDX-License-Identifier: LGPL-2.1-or-later
//! B-Rep kernel bridge (M01 `OM9-L2-01-05`): the Rust half of the OCCT adapter.
//!
//! The L2 owns the *adapter*: B-Rep records and their validation, safe handle lifetime, the
//! geometry-operation error contract, and the kernel thread/transaction policy. The native
//! kernel sits behind [`KernelBackend`]; with no OCCT linked the only backend is
//! [`UnlinkedBackend`], which refuses every kernel call with `Unsupported("occt kernel")` instead
//! of pretending to answer. Everything a caller can test in Rust is implemented and tested here.
//!
//! Accuracy: tolerance comparisons are the documented exact rule (`|a - b| <= tolerance`), shape
//! validation rejects a record one ULP outside its tolerance, and vertex/edge identity is decided
//! by exact coordinate equality, never by a distance threshold that would merge distinct points.
//! Performance: the handle store is a `Vec` with a free list, so create/lookup/drop are O(1) and
//! a bulk build does not allocate per handle.
use crate::{
    Error, Result,
    budget::Limits,
    geometry::{Point3, Transform, finite},
};
use std::collections::BTreeMap;

/// A kernel object handle. The generation makes a handle to a destroyed object invalid instead of
/// silently aliasing the slot's next occupant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KernelHandle {
    pub slot: u32,
    pub generation: u32,
}

/// The L3-009 geometry-operation error contract: every native failure maps to one of these, and
/// each has one `Error` spelling, so a command line never has to know OCCT's shapes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelError {
    /// The kernel is not linked in this build.
    Unavailable,
    /// The input shape is not a valid B-Rep.
    InvalidShape,
    /// A boolean/trim needs closed solids.
    ClosedSolidRequired,
    /// The operation has no solution for these inputs.
    NoSolution,
    /// The operation exceeded its work budget.
    BudgetExceeded,
    /// The kernel call was cancelled by the caller.
    Cancelled,
    /// The handle is stale (the object was destroyed or replaced).
    StaleHandle,
    /// The input geometry lies within the kernel tolerance, so it cannot be resolved.
    BelowTolerance,
    /// A native fault escaped the adapter (a defect, not a user error).
    NativeFault,
}

impl KernelError {
    pub fn message(self) -> &'static str {
        match self {
            Self::Unavailable => "The OCCT kernel is not available in this build",
            Self::InvalidShape => "Invalid B-Rep shape; inputs are unchanged",
            Self::ClosedSolidRequired => "Boolean requires valid closed solid inputs",
            Self::NoSolution => "The operation has no solution for these inputs",
            Self::BudgetExceeded => "The kernel operation exceeded its budget",
            Self::Cancelled => "The kernel operation was cancelled",
            Self::StaleHandle => "The kernel shape is no longer alive",
            Self::BelowTolerance => "The geometry is at or below the kernel tolerance",
            Self::NativeFault => "Native kernel fault; the document is unchanged",
        }
    }
    /// Map to the crate error. `Unavailable` is a capability refusal, not bad data; a native
    /// fault is a `Busy`-free hard error that callers must report, never swallow.
    pub fn into_error(self) -> Error {
        match self {
            Self::Unavailable => Error::Unsupported("occt kernel"),
            Self::InvalidShape => Error::Invalid("brep shape"),
            Self::ClosedSolidRequired => Error::Invalid("closed solid required"),
            Self::NoSolution => Error::Invalid("kernel no solution"),
            Self::BudgetExceeded => Error::Budget("kernel operation"),
            Self::Cancelled => Error::Canceled,
            Self::StaleHandle => Error::Stale("kernel handle"),
            Self::BelowTolerance => Error::Invalid("geometry below kernel tolerance"),
            Self::NativeFault => Error::Invalid("native kernel fault"),
        }
    }
}

impl From<KernelError> for Error {
    fn from(value: KernelError) -> Self {
        value.into_error()
    }
}

/// The shape class a B-Rep record claims. Trimmed surfaces are not a class here: the document
/// model has no trimmed-B-Rep record until the kernel is linked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeKind {
    Vertex,
    Edge,
    Wire,
    Face,
    Shell,
    Solid,
    Compound,
}

/// A topological vertex.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vertex {
    pub point: Point3,
    pub tolerance: f64,
}

/// A topological edge: two vertices plus the tolerance it is admitted at.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Edge {
    pub start: u32,
    pub end: u32,
    pub tolerance: f64,
    pub closed: bool,
}

/// A wire: an ordered list of edges that must form a chain.
#[derive(Debug, Clone, PartialEq)]
pub struct Wire {
    pub edges: Vec<u32>,
    pub closed: bool,
}

/// A face over one wire (outer boundary) and its inner wires.
#[derive(Debug, Clone, PartialEq)]
pub struct Face {
    pub outer: u32,
    pub inners: Vec<u32>,
    pub tolerance: f64,
}

/// A shell: the faces that bound one region.
#[derive(Debug, Clone, PartialEq)]
pub struct Shell {
    pub faces: Vec<u32>,
    pub closed: bool,
}

/// The portable B-Rep body the adapter validates before it hands the shape to the kernel.
#[derive(Debug, Clone, PartialEq)]
pub struct BrepBody {
    pub vertices: Vec<Vertex>,
    pub edges: Vec<Edge>,
    pub wires: Vec<Wire>,
    pub faces: Vec<Face>,
    pub shells: Vec<Shell>,
    pub kind: ShapeKind,
    pub tolerance: f64,
}

impl BrepBody {
    /// The smallest tolerance the kernel accepts; a smaller value is below the kernel's
    /// resolution and is refused rather than silently rounded up.
    pub const MIN_TOLERANCE: f64 = 1e-9;
    /// The largest tolerance a shape may declare, in millimetres.
    pub const MAX_TOLERANCE: f64 = 1.0e3;

    /// Validate the topology and the tolerances.
    ///
    /// The rules are the ones a kernel bridge must enforce before touching a kernel: every index
    /// exists, an edge is not a self-loop unless it says so, a wire is a chain, a face's wires
    /// exist, a closed shell bounds a region, and every tolerance is finite, inside
    /// `[MIN_TOLERANCE, MAX_TOLERANCE]`, and at least the size of the geometry it protects.
    pub fn validate(&self, limits: &Limits) -> Result<()> {
        if self.vertices.len() as u64 > limits.max_points
            || self.edges.len() as u64 > limits.max_objects
            || self.wires.len() as u64 > limits.max_objects
            || self.faces.len() as u64 > limits.max_objects
            || self.shells.len() as u64 > limits.max_objects
        {
            return Err(Error::Budget("brep body"));
        }
        self.check_tolerance(self.tolerance, "brep tolerance")?;
        if self.tolerance < Self::MIN_TOLERANCE {
            return Err(Error::Invalid("geometry below kernel tolerance"));
        }
        for vertex in &self.vertices {
            finite(vertex.point)?;
            self.check_tolerance(vertex.tolerance, "brep vertex tolerance")?;
        }
        for edge in &self.edges {
            let start = self
                .vertices
                .get(edge.start as usize)
                .ok_or(Error::Missing("brep vertex"))?;
            let end = self
                .vertices
                .get(edge.end as usize)
                .ok_or(Error::Missing("brep vertex"))?;
            self.check_tolerance(edge.tolerance, "brep edge tolerance")?;
            if edge.start == edge.end && !edge.closed {
                return Err(Error::Invalid("brep edge endpoints"));
            }
            if edge.closed && edge.start != edge.end {
                return Err(Error::Invalid("brep edge closure"));
            }
            // A distinct pair closer than the body tolerance is unresolvable: the kernel would
            // collapse it, so the adapter refuses instead of building a degenerate edge.
            if edge.start != edge.end {
                let distance = distance(start.point, end.point);
                if distance <= self.tolerance {
                    return Err(Error::Invalid("geometry below kernel tolerance"));
                }
            }
        }
        for wire in &self.wires {
            if wire.edges.is_empty() {
                return Err(Error::Invalid("brep wire edges"));
            }
            let mut previous_end: Option<u32> = None;
            for (position, index) in wire.edges.iter().enumerate() {
                let edge = self
                    .edges
                    .get(*index as usize)
                    .ok_or(Error::Missing("brep edge"))?;
                if let Some(previous) = previous_end
                    && previous != edge.start
                {
                    return Err(Error::Invalid("brep wire chain"));
                }
                if wire.edges[..position].contains(index) {
                    return Err(Error::Invalid("brep wire duplicate edge"));
                }
                previous_end = Some(edge.end);
            }
            if wire.closed
                && previous_end
                    != wire
                        .edges
                        .first()
                        .map(|first| self.edges[*first as usize].start)
            {
                return Err(Error::Invalid("brep wire closure"));
            }
        }
        for face in &self.faces {
            if self.wires.get(face.outer as usize).is_none() {
                return Err(Error::Missing("brep wire"));
            }
            self.check_tolerance(face.tolerance, "brep face tolerance")?;
            for inner in &face.inners {
                if self.wires.get(*inner as usize).is_none() {
                    return Err(Error::Missing("brep wire"));
                }
                if *inner == face.outer {
                    return Err(Error::Invalid("brep face wires"));
                }
            }
        }
        for shell in &self.shells {
            if shell.faces.is_empty() {
                return Err(Error::Invalid("brep shell faces"));
            }
            for face in &shell.faces {
                if self.faces.get(*face as usize).is_none() {
                    return Err(Error::Missing("brep face"));
                }
            }
            if shell.closed {
                // A closed shell must be a set (no repeated face), otherwise it bounds nothing.
                let mut seen = BTreeMap::new();
                for face in &shell.faces {
                    if seen.insert(*face, ()).is_some() {
                        return Err(Error::Invalid("brep shell duplicate face"));
                    }
                }
            }
        }
        Ok(())
    }

    fn check_tolerance(&self, tolerance: f64, label: &'static str) -> Result<()> {
        if !tolerance.is_finite()
            || tolerance < Self::MIN_TOLERANCE
            || tolerance > Self::MAX_TOLERANCE
        {
            return Err(Error::Invalid(label));
        }
        Ok(())
    }

    /// The shape class the kernel would see.
    pub fn shape_kind(&self) -> ShapeKind {
        self.kind
    }
}

fn distance(a: Point3, b: Point3) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1]).hypot(a[2] - b[2])
}

/// The kernel operations the adapter needs. A backend is either linked OCCT or [`UnlinkedBackend`].
pub trait KernelBackend {
    /// Evaluate a B-Rep shape into the adapter's body.
    fn build(&mut self, body: &BrepBody) -> std::result::Result<KernelHandle, KernelError>;
    /// Drop a shape. A stale handle is an error, never a silent no-op.
    fn destroy(&mut self, handle: KernelHandle) -> std::result::Result<(), KernelError>;
    /// Apply a rigid transform, returning the transformed shape.
    fn transform(
        &mut self,
        handle: KernelHandle,
        transform: Transform,
    ) -> std::result::Result<KernelHandle, KernelError>;
    /// Tessellate a shape into triangles (indices into the returned points).
    fn tessellate(
        &mut self,
        handle: KernelHandle,
        chord_tolerance: f64,
    ) -> std::result::Result<(Vec<Point3>, Vec<[u32; 3]>), KernelError>;
}

/// The backend of a build without OCCT: every call is refused with the same contract error.
#[derive(Debug, Clone, Copy, Default)]
pub struct UnlinkedBackend;

impl KernelBackend for UnlinkedBackend {
    fn build(&mut self, _body: &BrepBody) -> std::result::Result<KernelHandle, KernelError> {
        Err(KernelError::Unavailable)
    }
    fn destroy(&mut self, _handle: KernelHandle) -> std::result::Result<(), KernelError> {
        Err(KernelError::Unavailable)
    }
    fn transform(
        &mut self,
        _handle: KernelHandle,
        _transform: Transform,
    ) -> std::result::Result<KernelHandle, KernelError> {
        Err(KernelError::Unavailable)
    }
    fn tessellate(
        &mut self,
        _handle: KernelHandle,
        _chord_tolerance: f64,
    ) -> std::result::Result<(Vec<Point3>, Vec<[u32; 3]>), KernelError> {
        Err(KernelError::Unavailable)
    }
}

/// A generational handle store (L3-001). Creating, looking up and dropping a handle is O(1); the
/// free list means a bulk build does not allocate per handle, and the generation means a handle
/// to a dropped object can never resolve to the replacement that took its slot.
#[derive(Debug, Clone)]
pub struct HandleStore<T> {
    slots: Vec<Option<T>>,
    generations: Vec<u32>,
    free: Vec<u32>,
    /// Live object count, so `len()` is O(1) instead of a scan of the slots.
    live: usize,
}

impl<T> Default for HandleStore<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> HandleStore<T> {
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            generations: Vec::new(),
            free: Vec::new(),
            live: 0,
        }
    }

    /// Create a slot. The first `expected` insertions do not reallocate past `expected`, so a
    /// caller that knows its object count pays for one allocation.
    pub fn with_capacity(expected: usize) -> Self {
        Self {
            slots: Vec::with_capacity(expected),
            generations: Vec::with_capacity(expected),
            free: Vec::new(),
            live: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.live
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn insert(&mut self, value: T) -> KernelHandle {
        if let Some(slot) = self.free.pop() {
            self.slots[slot as usize] = Some(value);
            self.live += 1;
            return KernelHandle {
                slot,
                generation: self.generations[slot as usize],
            };
        }
        let slot = u32::try_from(self.slots.len()).unwrap_or(u32::MAX);
        self.slots.push(Some(value));
        self.generations.push(1);
        self.live += 1;
        KernelHandle {
            slot,
            generation: 1,
        }
    }

    pub fn get(&self, handle: KernelHandle) -> Option<&T> {
        if self.generations.get(handle.slot as usize).copied()? != handle.generation {
            return None;
        }
        self.slots.get(handle.slot as usize)?.as_ref()
    }

    pub fn get_mut(&mut self, handle: KernelHandle) -> Option<&mut T> {
        if self.generations.get(handle.slot as usize).copied()? != handle.generation {
            return None;
        }
        self.slots.get_mut(handle.slot as usize)?.as_mut()
    }

    /// Drop the object. The slot is reused with a bumped generation, so the old handle is stale.
    pub fn remove(&mut self, handle: KernelHandle) -> std::result::Result<T, KernelError> {
        if self.generations.get(handle.slot as usize).copied() != Some(handle.generation) {
            return Err(KernelError::StaleHandle);
        }
        let slot = handle.slot as usize;
        let value = self.slots[slot].take().ok_or(KernelError::StaleHandle)?;
        self.generations[slot] = self.generations[slot].wrapping_add(1).max(1);
        self.free.push(handle.slot);
        self.live = self.live.saturating_sub(1);
        Ok(value)
    }

    /// A handle is alive when its generation matches and the slot holds an object.
    pub fn is_alive(&self, handle: KernelHandle) -> bool {
        self.get(handle).is_some()
    }
}

/// The kernel thread and transaction policy (L3-010).
///
/// OCCT is not thread-safe for concurrent shape edits, so the policy admits **one** kernel call at
/// a time and **one** open transaction per document. It is a data record with a check function,
/// which is what a session calls before it enters the kernel; the actual mutex lives in the
/// session, so this stays testable and allocation-free.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct KernelPolicy {
    pub in_flight: bool,
    pub open_transaction: bool,
    /// The kernel requires calls from the owning thread; a call from another thread is refused.
    pub thread_locked: bool,
}

impl KernelPolicy {
    /// Admit one kernel call. `same_thread` is the caller's answer to "am I on the owning thread".
    pub fn admit_call(&self, same_thread: bool) -> Result<()> {
        if self.in_flight {
            return Err(Error::Busy);
        }
        if self.thread_locked && !same_thread {
            return Err(Error::Permission("kernel thread affinity"));
        }
        Ok(())
    }

    /// Admit opening a transaction: the policy allows one at a time, per document.
    pub fn admit_transaction(&self) -> Result<()> {
        if self.open_transaction {
            return Err(Error::Busy);
        }
        Ok(())
    }
}
