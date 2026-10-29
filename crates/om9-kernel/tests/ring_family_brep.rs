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
        let brep = creation::sewed_solid(&points, &faces, 1.0e-6, 0.01, &limits())
            .unwrap_or_else(|error| panic!("{kind:?} must sew: {error}"));
        assert!(brep.source().contains("CASCADE Topology"));
        assert!(!brep.display.faces.is_empty());
        // The same gate the mesh path runs accepts the OCCT body and measures it.
        let geometry = Geometry::Brep(brep);
        let report = ring_builder::inspect_geometry(&geometry, &limits()).unwrap();
        assert!(report.closed);
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
            ring_builder::inspect_geometry(&rebuilt.outputs[0].geometry, &limits()).unwrap();
        assert!(report.closed && report.volume_mm3 > 0.0);
        assert!(brep.display.faces.len() >= 4);
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
    for point in &brep.display.points {
        assert!(point.iter().all(|value| value.is_finite()), "{point:?}");
    }
    let usage = brep.validate(&limits()).unwrap();
    assert!(usage.points > 0);
    // The display snapshot names every face exactly once.
    let mut ids = brep.display.faces.clone();
    ids.sort_unstable();
    assert!(ids.windows(2).all(|pair| pair[0] != pair[1]));
    // A star-shaped fan of the same points would not sew into a solid: the adapter is the guard.
    let fan: Vec<[u32; 4]> = (0..8u32)
        .map(|index| [0, index + 1, index + 2, index + 3])
        .collect();
    let star: Vec<Point3> = points.clone();
    assert!(creation::sewed_solid(&star, &fan, 1.0e-6, 0.01, &limits()).is_err());
}
