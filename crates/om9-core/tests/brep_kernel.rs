// SPDX-License-Identifier: LGPL-2.1-or-later
//! OM9-L2-01-05: the Rust half of the OCCT B-Rep bridge.
//!
//! What is testable without OCCT is tested here: B-Rep validation, generational handle lifetime,
//! the geometry-operation error contract, the thread/transaction policy, and the adapter contract
//! itself against a fixture backend. The native kernel is *not* linked, so the unlinked backend
//! must refuse every call with one contract error instead of answering.
use om9_core::{
    Error,
    brep_kernel::{
        BrepBody, Edge, Face, HandleStore, KernelBackend, KernelError, KernelHandle, KernelPolicy,
        ShapeKind, Shell, UnlinkedBackend, Vertex, Wire,
    },
    budget::Limits,
    geometry::{Point3, Transform},
};

fn limits() -> Limits {
    Limits::default()
}

/// One triangle: three distinct vertices, one wire of three edges, one face, one open shell.
fn triangle() -> BrepBody {
    BrepBody {
        vertices: vec![
            Vertex {
                point: [0.0, 0.0, 0.0],
                tolerance: 0.001,
            },
            Vertex {
                point: [10.0, 0.0, 0.0],
                tolerance: 0.001,
            },
            Vertex {
                point: [0.0, 10.0, 0.0],
                tolerance: 0.001,
            },
        ],
        edges: vec![
            Edge {
                start: 0,
                end: 1,
                tolerance: 0.001,
                closed: false,
            },
            Edge {
                start: 1,
                end: 2,
                tolerance: 0.001,
                closed: false,
            },
            Edge {
                start: 2,
                end: 0,
                tolerance: 0.001,
                closed: false,
            },
        ],
        wires: vec![Wire {
            edges: vec![0, 1, 2],
            closed: true,
        }],
        faces: vec![Face {
            outer: 0,
            inners: vec![],
            tolerance: 0.001,
        }],
        shells: vec![Shell {
            faces: vec![0],
            closed: false,
        }],
        kind: ShapeKind::Shell,
        tolerance: 0.001,
    }
}

/* ------------------------- B-Rep validation (L3-002) ------------------------- */

#[test]
fn a_valid_triangle_passes_brep_validation() {
    triangle().validate(&limits()).unwrap();
    assert_eq!(triangle().shape_kind(), ShapeKind::Shell);
}

#[test]
fn a_brep_body_refuses_broken_topology() {
    let limits = limits();
    // An edge that points at a missing vertex.
    let mut body = triangle();
    body.edges[0].end = 9;
    assert_eq!(body.validate(&limits), Err(Error::Missing("brep vertex")));
    // A wire whose edges do not chain.
    let mut body = triangle();
    body.wires[0].edges = vec![0, 2, 1];
    assert_eq!(
        body.validate(&limits),
        Err(Error::Invalid("brep wire chain"))
    );
    // A duplicated edge in one wire (the chain still closes, so only the duplicate is wrong).
    let mut body = triangle();
    body.wires[0].edges = vec![0, 1, 2, 0];
    body.wires[0].closed = false;
    assert_eq!(
        body.validate(&limits),
        Err(Error::Invalid("brep wire duplicate edge"))
    );
    // A closed wire that does not return to its start.
    let mut body = triangle();
    body.wires[0].edges = vec![0, 1];
    assert_eq!(
        body.validate(&limits),
        Err(Error::Invalid("brep wire closure"))
    );
    // A face over a missing wire.
    let mut body = triangle();
    body.faces[0].outer = 4;
    assert_eq!(body.validate(&limits), Err(Error::Missing("brep wire")));
    // A face that names its own outer wire as inner.
    let mut body = triangle();
    body.faces[0].inners = vec![0];
    assert_eq!(
        body.validate(&limits),
        Err(Error::Invalid("brep face wires"))
    );
    // A shell over a missing face, an empty shell, and a duplicated face in a closed shell.
    let mut body = triangle();
    body.shells[0].faces = vec![3];
    assert_eq!(body.validate(&limits), Err(Error::Missing("brep face")));
    let mut body = triangle();
    body.shells[0].faces.clear();
    assert_eq!(
        body.validate(&limits),
        Err(Error::Invalid("brep shell faces"))
    );
    let mut body = triangle();
    body.shells[0].closed = true;
    body.shells[0].faces = vec![0, 0];
    assert_eq!(
        body.validate(&limits),
        Err(Error::Invalid("brep shell duplicate face"))
    );
    // An edge that claims closure but has distinct endpoints, and one that collapses.
    let mut body = triangle();
    body.edges[0].closed = true;
    assert_eq!(
        body.validate(&limits),
        Err(Error::Invalid("brep edge closure"))
    );
    let mut body = triangle();
    body.edges[0].end = 0;
    assert_eq!(
        body.validate(&limits),
        Err(Error::Invalid("brep edge endpoints"))
    );
}

#[test]
fn tolerance_rules_are_exact_at_their_boundaries() {
    let limits = limits();
    // Exactly at the minimum is admitted, one ULP below is refused.
    let mut body = triangle();
    body.tolerance = BrepBody::MIN_TOLERANCE;
    body.validate(&limits).unwrap();
    body.tolerance = f64::from_bits(BrepBody::MIN_TOLERANCE.to_bits() - 1);
    assert_eq!(
        body.validate(&limits),
        Err(Error::Invalid("brep tolerance"))
    );
    // Above the maximum, non-finite, and a negative tolerance are refused.
    let mut body = triangle();
    body.tolerance = BrepBody::MAX_TOLERANCE * 2.0;
    assert_eq!(
        body.validate(&limits),
        Err(Error::Invalid("brep tolerance"))
    );
    let mut body = triangle();
    body.tolerance = f64::NAN;
    assert_eq!(
        body.validate(&limits),
        Err(Error::Invalid("brep tolerance"))
    );
    // A distinct vertex pair closer than the body tolerance is unresolvable, so the adapter
    // refuses it instead of letting the kernel collapse the edge.
    let mut body = triangle();
    body.vertices[1].point = [1.0e-12, 0.0, 0.0];
    body.tolerance = 1.0e-6;
    assert_eq!(
        body.validate(&limits),
        Err(Error::Invalid("geometry below kernel tolerance"))
    );
    // The same pair with a tolerance far below the distance is fine.
    body.tolerance = BrepBody::MIN_TOLERANCE;
    body.vertices[1].point = [0.5, 0.0, 0.0];
    body.validate(&limits).unwrap();
}

#[test]
fn a_body_over_the_object_budget_is_refused() {
    let mut body = triangle();
    let tight = Limits {
        max_objects: 2,
        ..limits()
    };
    body.edges = vec![
        Edge {
            start: 0,
            end: 1,
            tolerance: 0.001,
            closed: false,
        };
        3
    ];
    assert_eq!(body.validate(&tight), Err(Error::Budget("brep body")));
}

/* ------------------------- handle lifecycle (L3-001) ------------------------- */

#[test]
fn a_handle_store_rejects_stale_and_double_dropped_handles() {
    let mut store: HandleStore<&'static str> = HandleStore::with_capacity(4);
    let first = store.insert("first");
    assert_eq!(store.get(first), Some(&"first"));
    assert!(store.is_alive(first));
    // Drop it: the handle is dead and a second drop is stale, never a silent no-op.
    assert_eq!(store.remove(first), Ok("first"));
    assert!(!store.is_alive(first));
    assert_eq!(store.get(first), None);
    assert_eq!(store.remove(first), Err(KernelError::StaleHandle));
    // The slot is reused with a new generation, so the old handle cannot alias the new object.
    let second = store.insert("second");
    assert_eq!(second.slot, first.slot, "the slot is reused");
    assert_ne!(
        second.generation, first.generation,
        "the generation moves on"
    );
    assert_eq!(store.get(first), None, "the old handle stays dead");
    assert_eq!(store.get(second), Some(&"second"));
    assert_eq!(store.len(), 1);
}

#[test]
fn a_handle_store_round_trips_many_objects_without_leaking() {
    let mut store: HandleStore<u32> = HandleStore::with_capacity(1000);
    let handles: Vec<KernelHandle> = (0..1000).map(|value| store.insert(value)).collect();
    assert_eq!(store.len(), 1000);
    for (index, handle) in handles.iter().enumerate() {
        assert_eq!(store.get(*handle), Some(&(index as u32)));
    }
    // Drop every other object, then reuse the slots: no slot is lost.
    for handle in handles.iter().step_by(2) {
        assert!(store.remove(*handle).is_ok());
    }
    assert_eq!(store.len(), 500);
    for _ in 0..500 {
        let reused = store.insert(7);
        assert!(store.is_alive(reused));
    }
    assert_eq!(store.len(), 1000);
}

/* ------------------------- error contract (L3-009) ------------------------- */

#[test]
fn every_kernel_error_maps_to_one_contract_error() {
    let cases = [
        (KernelError::Unavailable, Error::Unsupported("occt kernel")),
        (KernelError::InvalidShape, Error::Invalid("brep shape")),
        (
            KernelError::ClosedSolidRequired,
            Error::Invalid("closed solid required"),
        ),
        (
            KernelError::NoSolution,
            Error::Invalid("kernel no solution"),
        ),
        (
            KernelError::BudgetExceeded,
            Error::Budget("kernel operation"),
        ),
        (KernelError::Cancelled, Error::Canceled),
        (KernelError::StaleHandle, Error::Stale("kernel handle")),
        (
            KernelError::BelowTolerance,
            Error::Invalid("geometry below kernel tolerance"),
        ),
        (
            KernelError::NativeFault,
            Error::Invalid("native kernel fault"),
        ),
    ];
    for (error, expected) in cases {
        assert_eq!(error.into_error(), expected, "{error:?}");
        assert_eq!(Error::from(error), expected, "{error:?}");
        assert!(!error.message().is_empty());
    }
}

/* ------------------------- policy (L3-010) ------------------------- */

#[test]
fn the_kernel_policy_serializes_calls_and_transactions() {
    let idle = KernelPolicy::default();
    assert_eq!(idle.admit_call(true), Ok(()));
    assert_eq!(idle.admit_transaction(), Ok(()));
    // One call at a time.
    let busy = KernelPolicy {
        in_flight: true,
        ..KernelPolicy::default()
    };
    assert_eq!(busy.admit_call(true), Err(Error::Busy));
    // Thread affinity: a call from another thread is refused only when the policy locks it.
    let locked = KernelPolicy {
        thread_locked: true,
        ..KernelPolicy::default()
    };
    assert_eq!(
        locked.admit_call(false),
        Err(Error::Permission("kernel thread affinity"))
    );
    assert_eq!(locked.admit_call(true), Ok(()));
    // One open transaction per document.
    let open = KernelPolicy {
        open_transaction: true,
        ..KernelPolicy::default()
    };
    assert_eq!(open.admit_transaction(), Err(Error::Busy));
}

/* ------------------------- adapter contract ------------------------- */

/// A fixture backend: it proves the adapter contract without OCCT. It keeps the same
/// generational store the real backend would use.
#[derive(Default)]
struct FixtureBackend {
    shapes: HandleStore<BrepBody>,
}

impl KernelBackend for FixtureBackend {
    fn build(&mut self, body: &BrepBody) -> std::result::Result<KernelHandle, KernelError> {
        if body.kind == ShapeKind::Compound {
            return Err(KernelError::InvalidShape);
        }
        Ok(self.shapes.insert(body.clone()))
    }
    fn destroy(&mut self, handle: KernelHandle) -> std::result::Result<(), KernelError> {
        self.shapes.remove(handle).map(|_| ())
    }
    fn transform(
        &mut self,
        handle: KernelHandle,
        transform: Transform,
    ) -> std::result::Result<KernelHandle, KernelError> {
        let body = self
            .shapes
            .get(handle)
            .ok_or(KernelError::StaleHandle)?
            .clone();
        let mut moved = body;
        for vertex in &mut moved.vertices {
            vertex.point = transform
                .point(vertex.point)
                .map_err(|_| KernelError::NativeFault)?;
        }
        Ok(self.shapes.insert(moved))
    }
    fn tessellate(
        &mut self,
        handle: KernelHandle,
        chord_tolerance: f64,
    ) -> std::result::Result<(Vec<Point3>, Vec<[u32; 3]>), KernelError> {
        if !chord_tolerance.is_finite() || chord_tolerance <= 0.0 {
            return Err(KernelError::BudgetExceeded);
        }
        let body = self.shapes.get(handle).ok_or(KernelError::StaleHandle)?;
        let points = body.vertices.iter().map(|vertex| vertex.point).collect();
        Ok((points, vec![[0, 1, 2]]))
    }
}

#[test]
fn the_unlinked_backend_refuses_every_kernel_call_with_one_contract_error() {
    let body = triangle();
    let mut unlinked = UnlinkedBackend;
    for result in [
        unlinked.build(&body).map(|_| ()),
        unlinked.destroy(KernelHandle {
            slot: 0,
            generation: 1,
        }),
        unlinked
            .transform(
                KernelHandle {
                    slot: 0,
                    generation: 1,
                },
                Transform::identity(),
            )
            .map(|_| ()),
        unlinked
            .tessellate(
                KernelHandle {
                    slot: 0,
                    generation: 1,
                },
                0.01,
            )
            .map(|_| ()),
    ] {
        assert_eq!(result, Err(KernelError::Unavailable));
    }
    // And the contract spelling is the capability refusal, not bad data.
    assert_eq!(
        KernelError::Unavailable.into_error(),
        Error::Unsupported("occt kernel")
    );
}

#[test]
fn the_adapter_contract_holds_against_a_fixture_backend() {
    let body = triangle();
    body.validate(&limits()).unwrap();
    let mut backend = FixtureBackend::default();
    let handle = backend.build(&body).unwrap();
    // Transform through the shared `Transform` type: the triangle moves as a whole.
    let moved = backend
        .transform(handle, Transform::translation([0.0, 0.0, 5.0]))
        .unwrap();
    let (points, triangles) = backend.tessellate(moved, 0.01).unwrap();
    assert_eq!(points.len(), 3);
    assert_eq!(triangles.len(), 1);
    assert!((points[0][2] - 5.0).abs() < 1e-12);
    // A bad chord tolerance is a budget refusal, and a stale handle is stale.
    assert_eq!(
        backend.tessellate(moved, 0.0).unwrap_err(),
        KernelError::BudgetExceeded
    );
    backend.destroy(moved).unwrap();
    assert_eq!(backend.destroy(moved), Err(KernelError::StaleHandle));
    // Compound shapes are refused at build by the fixture contract.
    let mut compound = triangle();
    compound.kind = ShapeKind::Compound;
    assert_eq!(
        backend.build(&compound).unwrap_err(),
        KernelError::InvalidShape
    );
}

/* ------------------------- performance ------------------------- */

#[test]
fn handle_and_validation_throughput_is_stated() {
    // Handle store: create/lookup/drop a burst and then reuse the slots.
    let mut store: HandleStore<u32> = HandleStore::with_capacity(100_000);
    const COUNT: u32 = 100_000;
    let started = std::time::Instant::now();
    let handles: Vec<KernelHandle> = (0..COUNT).map(|value| store.insert(value)).collect();
    let inserted = started.elapsed();
    let started = std::time::Instant::now();
    let mut sink = 0u64;
    for handle in &handles {
        sink += u64::from(*store.get(*handle).unwrap());
    }
    let looked_up = started.elapsed();
    assert_eq!(store.len() as u32, COUNT);
    println!(
        "HandleStore: {COUNT} insert in {inserted:?}, {COUNT} get in {looked_up:?} (sink {sink})"
    );
    assert!(
        inserted.as_millis() < 2_000,
        "insert must stay fast: {inserted:?}"
    );
    assert!(
        looked_up.as_millis() < 2_000,
        "lookup must stay fast: {looked_up:?}"
    );

    // B-Rep validation: the adapter runs this before every kernel call, so it must stay cheap
    // for the small topologies a jewellery model actually has.
    let body = triangle();
    let limits = limits();
    const VALIDATIONS: u32 = 20_000;
    let started = std::time::Instant::now();
    for _ in 0..VALIDATIONS {
        body.validate(&limits).unwrap();
    }
    let elapsed = started.elapsed();
    println!(
        "BrepBody::validate: {VALIDATIONS} triangle bodies in {elapsed:?} ({} ns/body)",
        elapsed.as_nanos() / u128::from(VALIDATIONS)
    );
    assert!(
        elapsed.as_millis() < 5_000,
        "validation must stay fast: {elapsed:?}"
    );
}
