// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    Error,
    budget::Limits,
    curve_build::arc_by_center,
    geometry::{Curve, Geometry},
    model::{Document, Entity},
};
use om9_scene::scene::{RenderBuffers, Scene};
use std::sync::Arc;

fn document(gap: bool) -> Document {
    let mut doc = Document::new("mixed polyline");
    let curve = Curve::Polycurve {
        segments: vec![
            Curve::Line {
                start: [0., 0., 0.],
                end: [10., 0., 0.],
                domain: [0., 1.],
            },
            arc_by_center(
                [10., 5., 0.],
                [0., -1., 0.],
                [1., 0., 0.],
                5.,
                0.,
                std::f64::consts::FRAC_PI_2,
            )
            .unwrap(),
            Curve::Line {
                start: [if gap { 16. } else { 15. }, 5., 0.],
                end: [20., 5., 0.],
                domain: [0., 1.],
            },
        ],
        reversed: false,
    };
    let entity = Entity::new("Polyline", Geometry::Curve(curve), doc.current_layer);
    doc.entities.insert(entity.id, entity);
    doc
}
#[test]
fn mixed_curve_draws_arc_without_losing_exact_line_corners() {
    let scene = Scene::capture(Arc::new(document(false)), 1, &Limits::default()).unwrap();
    assert!(scene.diagnostics.is_empty());
    let RenderBuffers::Curve { points, parameters } = scene.objects[0].buffers.as_ref() else {
        panic!("curve");
    };
    assert_eq!(points[0], [0., 0., 0.]);
    assert_eq!(points[1], [10., 0., 0.]);
    assert_eq!(*points.last().unwrap(), [20., 5., 0.]);
    assert!(points.len() > 18);
    for point in &points[1..points.len() - 1] {
        let radius = (point[0] - 10.).hypot(point[1] - 5.);
        assert!((radius - 5.).abs() < 1e-10);
    }
    assert!(parameters.is_empty());
    assert!(scene.objects[0].has_pick_geometry());
}
#[test]
fn mixed_curve_preflight_and_cache_agree_on_the_point_budget() {
    let doc = document(false);
    let source = doc.validate(&Limits::default()).unwrap().points;
    let scene = Scene::capture(Arc::new(doc.clone()), 1, &Limits::default()).unwrap();
    let display = scene.cache_usage().unwrap().points;
    let limits = Limits {
        max_points: source + display,
        ..Limits::default()
    };
    Scene::capture(Arc::new(doc.clone()), 1, &limits).unwrap();
    let smaller = Limits {
        max_points: limits.max_points - 1,
        ..limits
    };
    assert!(matches!(
        Scene::capture(Arc::new(doc), 1, &smaller),
        Err(Error::Budget(_))
    ));
}

#[test]
fn incremental_addition_reserves_all_arc_display_points() {
    let doc = document(false);
    let entity = doc.entities.values().next().unwrap();
    let rendered = Scene::capture(Arc::new(doc.clone()), 1, &Limits::default()).unwrap();
    let display_points = rendered.cache_usage().unwrap().points;
    let empty = Scene::capture(Arc::new(Document::new("empty")), 1, &Limits::default()).unwrap();
    let base = empty.update_usage(None).unwrap();
    let added = empty.update_usage(Some(entity)).unwrap();
    assert!(display_points > 18);
    assert_eq!(added.points - base.points, display_points);
    assert!(added.bytes - base.bytes >= display_points * std::mem::size_of::<[f64; 3]>() as u64);
    let too_small = Limits { max_points: display_points - 1, ..Limits::default() };
    assert!(matches!(added.check(&too_small), Err(Error::Budget(_))));
}
#[test]
fn mixed_curve_gap_is_rejected_without_inventing_a_connector() {
    let scene = Scene::capture(Arc::new(document(true)), 1, &Limits::default()).unwrap();
    assert!(scene.objects.is_empty());
    assert!(scene.diagnostics[0].contains("connected"));
}

#[test]
fn connected_nurbs_and_line_display_without_invented_connectors() {
    let arch = Curve::Nurbs(om9_core::geometry::NurbsCurve {
        degree: 2, poles: vec![[3.,5.,0.],[5.,9.,0.],[7.,5.,0.]], weights: vec![1.;3],
        knots: vec![0.,0.,0.,1.,1.,1.], domain: [0.,1.], periodic: false, reversed: false, seam: None,
    });
    for reversed in [false, true] {
        let mut doc = Document::new("NURBS polycurve");
        let line = Curve::Line { start: [7.,5.,0.], end: [3.,5.,0.], domain: [0.,1.] };
        let entity = Entity::new("profile", Geometry::Curve(Curve::Polycurve { segments: vec![arch.clone(), line], reversed }), None);
        doc.entities.insert(entity.id, entity);
        let scene = Scene::capture(Arc::new(doc), 1, &Limits::default()).unwrap();
        assert!(scene.diagnostics.is_empty(), "{:?}", scene.diagnostics);
        let RenderBuffers::Curve { points, .. } = scene.objects[0].buffers.as_ref() else { panic!() };
        assert_eq!(points.len(), 258);
        assert_eq!(points.first(), points.last());
        assert!(points.iter().any(|p| (p[0]-5.).abs() < 1e-10 && (p[1]-7.).abs() < 1e-10));
        assert!(scene.objects[0].has_pick_geometry());
    }
}

#[test]
fn discontinuous_nurbs_segment_is_not_connected_by_display() {
    let arch = Curve::Nurbs(om9_core::geometry::NurbsCurve {
        degree: 1, poles: vec![[0.,0.,0.],[1.,0.,0.],[3.,0.,0.],[4.,0.,0.]], weights: vec![1.;4],
        knots: vec![0.,0.,0.5,0.5,1.,1.], domain: [0.,1.], periodic: false, reversed: false, seam: None,
    });
    let mut doc = Document::new("discontinuous polycurve");
    let entity = Entity::new("profile", Geometry::Curve(Curve::Polycurve { segments: vec![arch], reversed: false }), None);
    doc.entities.insert(entity.id, entity);
    let scene = Scene::capture(Arc::new(doc), 1, &Limits::default()).unwrap();
    assert!(scene.objects.is_empty());
    assert!(scene.diagnostics[0].contains("discontinuous"));
}

#[test]
fn nurbs_polycurve_samples_short_knot_spans_and_honors_segment_reversal() {
    for reversed in [false, true] {
        let arch = Curve::Nurbs(om9_core::geometry::NurbsCurve {
            degree: 1, poles: vec![[0.,0.,0.],[0.,10.,0.],[1.,0.,0.],[2.,0.,0.]], weights: vec![1.;4],
            knots: vec![0.,0.,0.001,0.002,1.,1.], domain: [0.,1.], periodic: false, reversed, seam: None,
        });
        let line = Curve::Line { start: if reversed { [0.,0.,0.] } else { [2.,0.,0.] }, end: if reversed { [2.,0.,0.] } else { [0.,0.,0.] }, domain: [0.,1.] };
        let mut doc = Document::new("short NURBS spans");
        let entity = Entity::new("profile", Geometry::Curve(Curve::Polycurve { segments: vec![arch, line], reversed: false }), None);
        let empty = Scene::capture(Arc::new(Document::new("empty")), 1, &Limits::default()).unwrap();
        let added = empty.update_usage(Some(&entity)).unwrap();
        let base = empty.update_usage(None).unwrap();
        doc.entities.insert(entity.id, entity);
        let scene = Scene::capture(Arc::new(doc), 1, &Limits::default()).unwrap();
        assert!(scene.diagnostics.is_empty(), "{:?}", scene.diagnostics);
        let RenderBuffers::Curve { points, .. } = scene.objects[0].buffers.as_ref() else { panic!() };
        assert_eq!(points.len(), 770);
        assert_eq!(added.points - base.points, points.len() as u64);
        let too_small = Limits { max_points: added.points - 1, ..Limits::default() };
        assert!(matches!(added.check(&too_small), Err(Error::Budget(_))));
        assert_eq!(points.first().unwrap(), if reversed { &[2.,0.,0.] } else { &[0.,0.,0.] });
        assert!(points.iter().any(|p| (p[0]).abs() < 1e-9 && (p[1]-10.).abs() < 1e-9));
        assert!(scene.objects[0].has_pick_geometry());
    }
}
