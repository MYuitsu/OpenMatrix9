// SPDX-License-Identifier: LGPL-2.1-or-later
//! The jewelry ring family as an OCCT B-Rep solid (`OM9-L2-04-03`, `OM9-L3-04-03-007`).
//!
//! The mesh producer in `om9-core` and the OCCT producer registered in
//! [`om9_kernel::associative::history_registry`] sample the same axial sections; these tests
//! prove that the sewn B-Rep is one closed body whose volume matches the mesh body, that the
//! shared feasibility gate accepts it, and that a broken quad set is refused instead of being
//! published half open.
use om9_core::{
    associative::{InputSnapshot, RebuildRequest},
    budget::Limits,
    geometry::{Geometry, Point3},
    identity::{DocumentId, HistoryId, ObjectId},
    ring_builder::{self, RingBuilderKind, RingParams},
};
use om9_kernel::creation;

fn limits() -> Limits {
    Limits::default()
}

fn params(kind: RingBuilderKind) -> RingParams {
    kind.defaults()
}

#[test]
fn ring_feasibility_uses_source_not_viewport_triangles() {
    let body = creation::primitive(creation::Primitive::Sphere { radius_mm: 3.0 }, 0.5, &limits()).unwrap();
    let mut misleading = body.display().clone();
    for point in &mut misleading.points { for coordinate in point { *coordinate *= 2.0; } }
    let alternate = om9_core::brep::Brep::from_native(body.source().to_owned(), misleading, &limits()).unwrap();
    let source_measure = creation::measure(&body, &limits()).unwrap();
    let alternate_measure = creation::measure(&alternate, &limits()).unwrap();
    let report = om9_kernel::associative::inspect_ring_geometry(&Geometry::Brep(alternate), &limits()).unwrap();
    let expected = 36.0 * std::f64::consts::PI;
    assert!((source_measure.volume_mm3 - expected).abs() < 1e-6);
    assert!((alternate_measure.volume_mm3 - expected).abs() < 1e-6);
    assert!((report.volume_mm3 - expected).abs() < 1e-6);
    let mut invalid_display = body.clone();
    invalid_display.edit_display_for_validation(|display| display.points[0][0] = f64::NAN);
    assert!(invalid_display.validate(&limits()).is_err());
    assert!((creation::measure(&invalid_display, &limits()).unwrap().volume_mm3 - expected).abs() < 1e-6,
        "source measurement must not depend on an invalid viewport display");
    let mut tight = limits();
    tight.max_bytes = body.storage_usage(&limits()).unwrap().bytes;
    assert!(creation::measure(&body, &tight).is_err(), "live display and source-validation scratch both require admission");
    assert!(matches!(ring_builder::inspect_geometry(&Geometry::Brep(body), &limits()),
        Err(om9_core::Error::Unsupported(_))));
}

#[test]
fn sewed_ring_admission_covers_tables_snapshot_and_cancellation() {
    let (points, quads, _) = ring_builder::quad_mesh(&params(RingBuilderKind::Signet)).unwrap();
    let faces: Vec<[u32; 4]> = quads.iter().map(|q| [q[0], q[1], q[2], q[3]]).collect();
    let canceled = creation::sewed_solid_with_volume_admitted(&points, &faces, 1e-6, 0.1, &limits(), |_, _| Err(om9_core::Error::Canceled));
    assert!(matches!(canceled, Err(creation::SewSolidError::Admission(om9_core::Error::Canceled))));
    let mut calls = Vec::new();
    let (brep, volume) = creation::sewed_solid_with_volume_admitted(&points, &faces, 1e-6, 0.1, &limits(), |live, pending| {
        calls.push((live, pending)); Ok(())
    }).unwrap();
    assert!(volume > 0.0);
    let first_snapshot = calls.iter().enumerate().skip(3).find(|(_, (live, pending))| *live > 0 && *pending > 0)
        .map(|(index, _)| index).expect("snapshot allocation admission");
    assert_eq!(calls.last().unwrap().0, brep.storage_usage(&limits()).unwrap().bytes);
    assert_eq!(calls.last().unwrap().1, 0);
    let mut step = 0usize;
    let refused = creation::sewed_solid_with_volume_admitted(&points, &faces, 1e-6, 0.1, &limits(), |_, _| {
        let current = step; step += 1;
        if current == first_snapshot { Err(om9_core::Error::Canceled) } else { Ok(()) }
    });
    assert!(matches!(refused, Err(creation::SewSolidError::Admission(om9_core::Error::Canceled))),
        "snapshot refusal must preserve the caller's cancellation error");
    let mut tight = limits();
    tight.max_bytes = 1;
    let mut invoked = false;
    let refused = creation::sewed_solid_with_volume_admitted(&points, &faces, 1e-6, 0.1, &tight, |_, _| { invoked = true; Ok(()) });
    assert!(matches!(refused, Err(creation::SewSolidError::Creation(creation::CreationError::Budget(_)))));
    assert!(!invoked, "oversized tables are refused before allocation");
}

#[test]
fn occt_sewing_checks_cancellation_during_native_face_construction() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    let (points, quads, _) = ring_builder::quad_mesh(&params(RingBuilderKind::Signet)).unwrap();
    let faces: Vec<[u32; 4]> = quads.iter().map(|q| [q[0], q[1], q[2], q[3]]).collect();
    let checks = AtomicUsize::new(0);
    let result = creation::sewed_solid_with_volume_cancellable(&points, &faces, 1e-6, 0.1, &limits(),
        |_, _| Ok(()), || checks.fetch_add(1, Ordering::AcqRel) >= 10);
    assert!(matches!(result, Err(creation::SewSolidError::Admission(om9_core::Error::Canceled))));
    assert!(checks.load(Ordering::Acquire) > 10, "the native adapter must poll during construction");
}

#[test]
fn ring_producer_admission_covers_staging_and_retained_output() {
    let kind = RingBuilderKind::Signet;
    let request = RebuildRequest {
        recipe: HistoryId::new(),
        command: kind.command().to_string(),
        document: DocumentId::new(),
        parameters: params(kind).parameters().into_iter()
            .map(|(key, value)| (key.to_string(), value)).collect(),
        inputs: vec![],
        outputs: vec![ObjectId::new()],
    };
    let producer = om9_kernel::associative::RingFamilyBrepProducer { kind };
    let mut calls = Vec::new();
    let outputs = producer.produce_admitted(&request, &limits(), |live, pending| {
        calls.push((live, pending));
        Ok(())
    }).unwrap();
    assert_eq!(outputs.len(), 1);
    let Geometry::Brep(brep) = &outputs[0].geometry else { panic!("expected BRep") };
    assert_eq!(calls.first().unwrap().0, 0);
    assert!(calls.first().unwrap().1 > 0, "staging must be admitted before allocation");
    let retained = brep.storage_usage(&limits()).unwrap().bytes
        + (outputs.capacity() * std::mem::size_of::<om9_core::associative::RebuiltOutput>()) as u64;
    assert_eq!(*calls.last().unwrap(), (retained, 0));
    assert!(calls.iter().any(|(live, pending)| *live > 0 && *pending > 0));

    let mut step = 0;
    let refused = producer.produce_admitted(&request, &limits(), |_, _| {
        step += 1;
        if step == 2 { Err(om9_core::Error::Canceled) } else { Ok(()) }
    });
    assert!(matches!(refused, Err(om9_core::Error::Canceled)));
    let mut tight = limits();
    tight.max_bytes = 1;
    assert!(producer.produce_admitted(&request, &tight, |_, _| Ok(())).is_err());

    let mut last = (0, 0);
    let feature = producer.produce_feature_admitted(&request, &limits(), |live, pending| {
        last = (live, pending); Ok(())
    }).unwrap();
    assert_eq!(feature.outputs.len(), 1);
    assert_eq!(feature.subshapes.len(), 1);
    assert_eq!(feature.subshapes[0].output, request.outputs[0]);
    assert!(!feature.subshapes[0].subshapes.is_empty());
    let Geometry::Brep(body) = &feature.outputs[0].geometry else { panic!("expected BRep") };
    let names = &feature.subshapes[0].subshapes;
    use om9_core::producers::TopologyProvider;
    let direct = om9_kernel::associative::OcctTopology.subshapes(&feature.outputs[0].geometry, &limits()).unwrap();
    assert_eq!(names.len(), direct.len());
    for (worker, synchronous) in names.iter().zip(&direct) {
        assert!(worker.name.starts_with("source:"));
        assert_eq!((&worker.name, worker.kind), (&synchronous.name, synchronous.kind));
        assert!(worker.signature.deviation_from(&synchronous.signature) < 1e-8);
    }
    let remesh = om9_kernel::brep::read_brep(body.source(), 0.02, &limits()).unwrap();
    let remesh_names = om9_kernel::associative::OcctTopology.subshapes(&Geometry::Brep(remesh), &limits()).unwrap();
    assert_eq!(direct, remesh_names);
    let retained = body.storage_usage(&limits()).unwrap().bytes as usize
        + feature.outputs.capacity() * std::mem::size_of::<om9_core::associative::RebuiltOutput>()
        + feature.subshapes.capacity() * std::mem::size_of::<om9_core::associative::ProducedSubshapes>()
        + names.capacity() * std::mem::size_of::<om9_core::associative::Subshape>()
        + names.iter().map(|name| name.name.capacity()).sum::<usize>();
    assert_eq!(last, (retained as u64, 0));
    let mut feature_step = 0;
    let refused = producer.produce_feature_admitted(&request, &limits(), |_, _| {
        feature_step += 1;
        if feature_step == calls.len() + 3 { Err(om9_core::Error::Canceled) } else { Ok(()) }
    });
    assert!(matches!(refused, Err(om9_core::Error::Canceled)), "topology-stage cancellation must propagate");
}

#[test]
fn all_ring_source_signatures_match_the_synchronous_registry_policy() {
    use om9_core::producers::TopologyProvider;
    for kind in RingBuilderKind::ALL {
        let request = RebuildRequest { recipe: HistoryId::new(), command: kind.command().into(), document: DocumentId::new(),
            parameters: params(kind).parameters().into_iter().map(|(key, value)| (key.to_string(), value)).collect(),
            inputs: vec![], outputs: vec![ObjectId::new()] };
        let feature = om9_kernel::associative::RingFamilyBrepProducer { kind }
            .produce_feature_cancellable(&request, &limits(), |_, _| Ok(()), || false).unwrap();
        let direct = om9_kernel::associative::OcctTopology.subshapes(&feature.outputs[0].geometry, &limits()).unwrap();
        let worker = &feature.subshapes[0].subshapes;
        assert_eq!(worker.len(), direct.len(), "{kind:?}");
        for (a, b) in worker.iter().zip(&direct) {
            assert_eq!((&a.name, a.kind), (&b.name, b.kind), "{kind:?}");
            assert!(a.signature.deviation_from(&b.signature) < 1e-8, "{kind:?}: {}", a.name);
        }
    }
}

#[test]
fn display_topology_naming_admits_output_and_cancellation() {
    let brep = creation::primitive(creation::Primitive::Sphere { radius_mm: 3.0 }, 0.5, &limits()).unwrap();
    let topology = om9_core::producers::SnapshotTopology;
    let mut checkpoints = 0;
    let mut last = (0, 0);
    let actual = topology.brep_subshapes_admitted(brep.display(), &limits(), &mut |live, pending| {
        checkpoints += 1;
        last = (live, pending);
        Ok(())
    }).unwrap();
    assert_eq!(actual.iter().filter(|item| item.kind == om9_core::associative::SubshapeKind::Vertex).count(), brep.display().vertices.len());
    assert!(actual.iter().any(|item| item.kind == om9_core::associative::SubshapeKind::Face && item.signature.measure_mm > 0.0));
    let output_bytes = std::mem::size_of::<om9_core::associative::Subshape>() * actual.capacity()
        + actual.iter().map(|item| item.name.capacity()).sum::<usize>();
    assert_eq!(last, (output_bytes as u64, 0));
    assert!(checkpoints > 2);
    let mut checkpoints = 0;
    let refused = topology.brep_subshapes_admitted(brep.display(), &limits(), &mut |_, _| {
        checkpoints += 1;
        if checkpoints == 3 { Err(om9_core::Error::Canceled) } else { Ok(()) }
    });
    assert!(matches!(refused, Err(om9_core::Error::Canceled)));
    let mut tight = limits();
    tight.max_bytes = 1;
    assert!(topology.brep_subshapes_admitted(brep.display(), &tight, &mut |_, _| Ok(())).is_err());
}

#[test]
fn prepared_ring_feature_can_be_built_off_thread_and_published() {
    let document = om9_core::model::Document::new("worker ring");
    let mut session = om9_core::builder::BuilderSession::new(
        om9_core::builder::BuilderKind::SignetRing, &document, &[]).unwrap();
    let request = session.prepare_refresh(&document).unwrap();
    let owner = std::thread::current().id();
    let worker = std::thread::spawn(move || {
        let producer = om9_kernel::associative::RingFamilyBrepProducer { kind: RingBuilderKind::Signet };
        let produced = producer.produce_feature_admitted(request.request(), &limits(), |_, _| Ok(())).unwrap();
        (request, produced, std::thread::current().id())
    });
    let (request, produced, worker_id) = worker.join().unwrap();
    assert_ne!(worker_id, owner);
    session.finish_refresh(&document, request, produced).unwrap();
    assert!(matches!(session.preview().unwrap().geometry, Geometry::Brep(_)));
    assert!(session.plan(&document).is_ok());
}

/// Every kind sews into one closed B-Rep whose volume is the mesh volume.
#[test]
fn every_ring_kind_sews_into_one_closed_brep_matching_the_mesh_volume() {
    for kind in RingBuilderKind::ALL {
        let ring = params(kind);
        let (points, quads, _wall) = ring_builder::quad_mesh(&ring).unwrap();
        let mut faces: Vec<[u32; 4]> = Vec::with_capacity(quads.len());
        for quad in &quads {
            faces.push([quad[0], quad[1], quad[2], quad[3]]);
        }
        let (brep, receipt_volume) = creation::sewed_solid_with_volume(&points, &faces, 1.0e-6, 0.01, &limits())
            .unwrap_or_else(|error| panic!("{kind:?} must sew: {error}"));
        assert!(brep.source().contains("CASCADE Topology"));
        assert!(!brep.display().faces.is_empty());
        // The same gate the mesh path runs accepts the OCCT body and measures it.
        let geometry = Geometry::Brep(brep);
        let report = om9_kernel::associative::inspect_ring_geometry(&geometry, &limits()).unwrap();
        assert!(report.closed);
        assert!((receipt_volume - report.volume_mm3).abs() <= report.volume_mm3 * 1e-8,
            "same-call native receipt must match a fresh source measurement");
        let mesh = ring_builder::build(&ring).unwrap();
        assert!(mesh.closed);
        // Sewing keeps the analytic volume: one percent covers the display tessellation.
        let deviation = (report.volume_mm3 - mesh.volume_mm3).abs() / mesh.volume_mm3;
        assert!(
            deviation < 0.01,
            "{kind:?} B-Rep volume {} vs mesh {} ({deviation:.4})",
            report.volume_mm3,
            mesh.volume_mm3
        );
    }
}

/// A quad set with one face removed has a free border, so the adapter refuses it.
#[test]
fn a_ring_body_with_a_missing_face_is_refused_instead_of_published_open() {
    let ring = params(RingBuilderKind::Signet);
    let (points, mut quads, _wall) = ring_builder::quad_mesh(&ring).unwrap();
    quads.pop();
    let mut faces: Vec<[u32; 4]> = Vec::with_capacity(quads.len());
    for quad in &quads {
        faces.push([quad[0], quad[1], quad[2], quad[3]]);
    }
    let refused = creation::sewed_solid(&points, &faces, 1.0e-6, 0.01, &limits());
    assert!(
        refused.is_err(),
        "an open ring must not become a B-Rep body"
    );
    assert!(creation::sewed_solid_with_volume(&points, &faces, 1.0e-6, 0.01, &limits()).is_err(),
        "failure must not return a volume receipt");
}

/// The OCCT registry rebuilds a ring feature from its recipe into a B-Rep body.
#[test]
fn the_occt_registry_rebuilds_the_ring_family_as_brep() {
    let registry = om9_kernel::associative::history_registry().unwrap();
    for kind in RingBuilderKind::ALL {
        let ring = params(kind);
        let parameters = ring
            .parameters()
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect();
        let request = RebuildRequest {
            recipe: HistoryId::new(),
            command: kind.command().to_string(),
            document: DocumentId::new(),
            parameters,
            inputs: Vec::<InputSnapshot>::new(),
            outputs: vec![ObjectId::new()],
        };
        let rebuilt = registry.produce(&request).unwrap();
        assert_eq!(rebuilt.outputs.len(), 1);
        let Geometry::Brep(brep) = &rebuilt.outputs[0].geometry else {
            panic!("{kind:?} must rebuild a B-Rep body");
        };
        let report =
            om9_kernel::associative::inspect_ring_geometry(&rebuilt.outputs[0].geometry, &limits()).unwrap();
        assert!(report.closed && report.volume_mm3 > 0.0);
        assert!(brep.display().faces.len() >= 4);
        // The OCCT topology policy resolved the sewn body into published subshapes.
        assert!(!rebuilt.subshapes.is_empty());
    }
}

/// A ring body keeps points finite and never leaks a native handle into the document.
#[test]
fn a_brep_ring_snapshot_binds_to_its_source_and_budget() {
    let ring = params(RingBuilderKind::Eternity);
    let (points, quads, _wall) = ring_builder::quad_mesh(&ring).unwrap();
    let mut faces: Vec<[u32; 4]> = Vec::with_capacity(quads.len());
    for quad in &quads {
        faces.push([quad[0], quad[1], quad[2], quad[3]]);
    }
    let brep = creation::sewed_solid(&points, &faces, 1.0e-6, 0.01, &limits()).unwrap();
    for point in &brep.display().points {
        assert!(point.iter().all(|value| value.is_finite()), "{point:?}");
    }
    let usage = brep.validate(&limits()).unwrap();
    assert!(usage.points > 0);
    let mut refused = limits();
    refused.max_points = 1;
    assert!(creation::sewed_solid_with_volume(&points, &faces, 1.0e-6, 0.01, &refused).is_err(),
        "a failed snapshot must not return its earlier native volume");
    // The display snapshot names every face exactly once.
    let mut ids = brep.display().faces.clone();
    ids.sort_unstable();
    assert!(ids.windows(2).all(|pair| pair[0] != pair[1]));
    // A star-shaped fan of the same points would not sew into a solid: the adapter is the guard.
    let fan: Vec<[u32; 4]> = (0..8u32)
        .map(|index| [0, index + 1, index + 2, index + 3])
        .collect();
    let star: Vec<Point3> = points.clone();
    assert!(creation::sewed_solid(&star, &fan, 1.0e-6, 0.01, &limits()).is_err());
}
