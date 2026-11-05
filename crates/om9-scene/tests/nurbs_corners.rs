// SPDX-License-Identifier: LGPL-2.1-or-later
//! A degree-1 NURBS (a polygon as add-ons create it) is drawn through its exact corners:
//! uniform display samples alone would land beside a kink and cut the corner.
use om9_core::{
    budget::Limits,
    geometry::{Curve, Geometry, NurbsCurve},
    model::{Document, Entity},
};
use om9_scene::scene::{RenderBuffers, Scene};
use std::sync::Arc;

#[test]
fn degree_one_nurbs_keeps_its_corners() {
    // Chord-length knots that no 1/256 step hits.
    let poles = vec![
        [0., 0., 0.],
        [7., 0., 0.],
        [7., 3., 0.],
        [0., 3., 0.],
        [0., 0., 0.],
    ];
    let knots = vec![0., 0., 0.35, 0.5, 0.85, 1., 1.];
    let curve = Curve::Nurbs(NurbsCurve {
        degree: 1,
        poles: poles.clone(),
        weights: vec![1.; 5],
        knots,
        domain: [0., 1.],
        periodic: false,
        reversed: false,
        seam: None,
    });
    let mut doc = Document::new("polygon");
    let entity = Entity::new("Polygon", Geometry::Curve(curve), doc.current_layer);
    doc.entities.insert(entity.id, entity);
    let scene = Scene::capture(Arc::new(doc), 1, &Limits::default()).unwrap();
    let RenderBuffers::Curve { points, parameters } = scene.objects[0].buffers.as_ref() else {
        panic!("curve");
    };
    for corner in &poles {
        assert!(
            points
                .iter()
                .any(|p| (0..3).all(|i| (p[i] - corner[i]).abs() < 1e-12)),
            "corner {corner:?} is drawn"
        );
    }
    assert!(parameters.windows(2).all(|w| w[0] < w[1]), "increasing");
}
