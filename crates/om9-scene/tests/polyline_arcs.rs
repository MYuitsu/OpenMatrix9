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
fn nested_mixed_segments_render_reversed_without_invented_connections() {
    let arc = arc_by_center([10.,5.,0.], [0.,-1.,0.], [1.,0.,0.], 5., 0., std::f64::consts::FRAC_PI_2).unwrap();
    let line = |start, end| Curve::Line { start, end, domain: [0.,1.] };
    for reversed in [false,true] { for nurbs_reversed in [false,true] {
        let nurbs = Curve::Nurbs(om9_core::geometry::NurbsCurve {
            degree: 2, poles: if nurbs_reversed { vec![[20.,5.,0.],[17.5,8.,0.],[15.,5.,0.]] } else { vec![[15.,5.,0.],[17.5,8.,0.],[20.,5.,0.]] }, weights: vec![1.;3],
            knots: vec![0.,0.,0.,1.,1.,1.], domain: [0.,1.], periodic: false, reversed: nurbs_reversed, seam: None,
        });
        let inner = Curve::Polycurve { segments: vec![line([0.,0.,0.],[10.,0.,0.]),arc.clone(),nurbs.clone()], reversed: true };
        let curve = Curve::Polycurve { segments: vec![line([22.,5.,0.],[20.,5.,0.]),inner], reversed };
        let mut doc = Document::new("nested mixed");
        let entity = Entity::new("profile", Geometry::Curve(curve), None);
        doc.entities.insert(entity.id,entity);
        let source = doc.validate(&Limits::default()).unwrap().points;
        let scene = Scene::capture(Arc::new(doc.clone()),1,&Limits::default()).unwrap();
        assert!(scene.diagnostics.is_empty(),"{:?}",scene.diagnostics);
        let RenderBuffers::Curve { points, .. } = scene.objects[0].buffers.as_ref() else { panic!("curve") };
        assert_eq!(points.first().unwrap(),if reversed { &[0.,0.,0.] } else { &[22.,5.,0.] });
        assert_eq!(points.last().unwrap(),if reversed { &[22.,5.,0.] } else { &[0.,0.,0.] });
        assert!(points.iter().any(|p| (p[0]-17.5).abs()<1e-9 && (p[1]-6.5).abs()<1e-9));
        assert!(points.iter().any(|p| (p[0]-10.).abs()<1e-9 && p[1].abs()<1e-9));
        assert!(scene.objects[0].has_pick_geometry());
        let display = scene.cache_usage().unwrap().points;
        let limits = Limits { max_points: source.max(display), ..Limits::default() };
        Scene::capture(Arc::new(doc.clone()),1,&limits).unwrap();
        assert!(matches!(Scene::capture(Arc::new(doc),1,&Limits { max_points: limits.max_points-1, ..limits }),Err(Error::Budget(_))));
    }}
}

#[test]
fn nested_full_circle_and_periodic_nurbs_are_sampled_in_both_directions() {
    let circle = Curve::Circle { center:[0.;3],x:[1.,0.,0.],y:[0.,1.,0.],radius:2.,domain:[0.,std::f64::consts::TAU],angle_domain:None };
    let periodic = Curve::Nurbs(om9_core::geometry::NurbsCurve {
        degree:2,poles:vec![[0.,0.,0.],[2.,0.,0.],[0.,2.,0.],[-2.,0.,0.],[0.,0.,0.],[2.,0.,0.]],weights:vec![1.;6],
        knots:vec![-2.,-1.,0.,1.,2.,3.,4.,5.,6.],domain:[0.,4.],periodic:true,reversed:false,seam:None,
    });
    for leaf in [circle,periodic] { for reversed in [false,true] {
        let inner=Curve::Polycurve{segments:vec![leaf.clone()],reversed};
        let curve=Curve::Polycurve{segments:vec![inner],reversed:false};
        let mut doc=Document::new("nested closed curve");
        let entity=Entity::new("closed",Geometry::Curve(curve.clone()),None);doc.entities.insert(entity.id,entity);
        let scene=Scene::capture(Arc::new(doc.clone()),1,&Limits::default()).unwrap();
        assert!(scene.diagnostics.is_empty(),"{:?}",scene.diagnostics);
        let RenderBuffers::Curve{points,..}=scene.objects[0].buffers.as_ref() else {panic!("curve")};
        let expected=om9_core::curve_ops::point_at(&curve,0.25,&Limits::default()).unwrap();
        assert!(points.windows(2).any(|pair| {
            let d: [f64;3]=std::array::from_fn(|i|pair[1][i]-pair[0][i]);
            let delta: [f64;3]=std::array::from_fn(|i|expected[i]-pair[0][i]);
            let denom: f64=d.iter().map(|v|v*v).sum();
            let t=if denom==0. {0.} else {(0..3).map(|i|delta[i]*d[i]).sum::<f64>()/denom}.clamp(0.,1.);
            (0..3).map(|i|(delta[i]-t*d[i]).powi(2)).sum::<f64>().sqrt()<0.006
        }),"{expected:?}");
        assert!((0..3).all(|i|(points[0][i]-points.last().unwrap()[i]).abs()<1e-10));
        assert!(scene.objects[0].has_pick_geometry());
        let source=doc.validate(&Limits::default()).unwrap().points;let display=scene.cache_usage().unwrap().points;
        let limits=Limits{max_points:source.max(display),..Limits::default()};Scene::capture(Arc::new(doc.clone()),1,&limits).unwrap();
        assert!(matches!(Scene::capture(Arc::new(doc),1,&Limits{max_points:limits.max_points-1,..limits}),Err(Error::Budget(_))));
    }}
}

#[test]
fn nested_reseamed_nurbs_samples_each_knot_interval_without_a_false_bridge() {
    for seam in [0.,1.,1.24,4.] { for reversed in [false,true] { for outer_reversed in [false,true] {
        let leaf=Curve::Nurbs(om9_core::geometry::NurbsCurve {
            degree:2,poles:vec![[0.,0.,0.],[2.,0.,0.],[0.,2.,0.],[-2.,0.,0.],[0.,0.,0.],[2.,0.,0.]],weights:vec![1.;6],
            knots:vec![-2.,-1.,0.,1.,2.,3.,4.,5.,6.],domain:[0.,4.],periodic:true,reversed,seam:Some(seam),
        });
        let curve=Curve::Polycurve {segments:vec![Curve::Polycurve{segments:vec![leaf],reversed:outer_reversed}],reversed:false};
        let mut doc=Document::new("nested reseam");let e=Entity::new("curve",Geometry::Curve(curve.clone()),None);doc.entities.insert(e.id,e);
        let scene=Scene::capture(Arc::new(doc.clone()),1,&Limits::default()).unwrap();assert!(scene.diagnostics.is_empty(),"{:?}",scene.diagnostics);
        let RenderBuffers::Curve{points,..}=scene.objects[0].buffers.as_ref() else{panic!("curve")};
        let expected=om9_core::curve_ops::point_at(&curve,0.,&Limits::default()).unwrap();
        assert!((0..3).all(|i|(points[0][i]-expected[i]).abs()<1e-12),"seam={seam} reverse={reversed} outer={outer_reversed}");
        assert!((0..3).all(|i|(points.last().unwrap()[i]-expected[i]).abs()<1e-12));
        assert_eq!(points.len(),if seam==1.24 {1281}else{1025});
        assert!(points.windows(2).all(|p|(0..3).map(|i|(p[1][i]-p[0][i]).powi(2)).sum::<f64>().sqrt()<0.02));
        let source=doc.validate(&Limits::default()).unwrap().points;let display=scene.cache_usage().unwrap().points;
        let limits=Limits{max_points:source.max(display),..Limits::default()};Scene::capture(Arc::new(doc.clone()),1,&limits).unwrap();
        assert!(matches!(Scene::capture(Arc::new(doc),1,&Limits{max_points:limits.max_points-1,..limits}),Err(Error::Budget(_))));
    }}}
}

#[test]
fn reseamed_nonperiodic_closed_nurbs_is_displayed_but_open_is_refused() {
    for closed in [false,true] {
        let end=if closed {[0.,0.,0.]}else{[0.,1.,0.]};
        let nurbs=Curve::Nurbs(om9_core::geometry::NurbsCurve {degree:1,poles:vec![[0.,0.,0.],[2.,0.,0.],[2.,2.,0.],[0.,2.,0.],end],weights:vec![1.;5],knots:vec![0.,0.,1.,2.,3.,4.,4.],domain:[0.,4.],periodic:false,reversed:false,seam:Some(1.5)});
        let curve=Curve::Polycurve{segments:vec![Curve::Polycurve{segments:vec![nurbs],reversed:false}],reversed:false};
        let mut doc=Document::new("reseamed nonperiodic");let e=Entity::new("curve",Geometry::Curve(curve.clone()),None);doc.entities.insert(e.id,e);
        let scene=Scene::capture(Arc::new(doc),1,&Limits::default()).unwrap();
        if closed {
            assert!(scene.diagnostics.is_empty(),"{:?}",scene.diagnostics);
            let RenderBuffers::Curve{points,..}=scene.objects[0].buffers.as_ref() else{panic!("curve")};
            assert_eq!(points.len(),1281);assert_eq!(points[0],[2.,1.,0.]);assert_eq!(points.last().unwrap(),&[2.,1.,0.]);
        } else {
            assert!(scene.objects.is_empty());assert!(scene.diagnostics.iter().any(|s|s.contains("requires closed NURBS")));
        }
    }
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
        max_points: source.max(display),
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
    for nested in [false,true] {
        let inner = Curve::Polycurve { segments: vec![arch.clone()], reversed: false };
        let curve = if nested { Curve::Polycurve { segments: vec![inner], reversed: true } } else { inner };
        let mut doc = Document::new("discontinuous polycurve");
        let entity = Entity::new("profile", Geometry::Curve(curve), None);
        doc.entities.insert(entity.id, entity);
        let scene = Scene::capture(Arc::new(doc), 1, &Limits::default()).unwrap();
        assert!(scene.objects.is_empty());
        assert!(scene.diagnostics[0].contains("discontinuous"));
    }
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
