// SPDX-License-Identifier: LGPL-2.1-or-later
//! Focused acceptance tests for M03 `OM9-L2-01-10` boolean / split / cap / fillet / offset /
//! extract / health operations.
//!
//! Every assertion measures the exact OCCT B-Rep (volume/area/topology/naked edges), never a
//! display mesh. Run one owner at a time, for example:
//!
//! `cargo test -p om9-kernel --features 3dm-brep --test l2_01_10 m03_01_10_001 -- --exact`
use om9_core::{brep::Brep, budget::Limits, geometry::Point3};
use om9_kernel::{
    creation::{self, DEFAULT_DEFLECTION_MM, Primitive, Profile},
    editing::{self, BooleanOp, BooleanOutcome, EditingError},
};
use std::f64::consts::PI;

fn limits() -> Limits {
    Limits::default()
}

fn deflection() -> f64 {
    DEFAULT_DEFLECTION_MM
}

fn close(actual: f64, expected: f64, tolerance: f64) -> bool {
    (actual - expected).abs() <= tolerance
}

fn box_shape(dx: f64, dy: f64, dz: f64) -> Brep {
    creation::primitive(
        Primitive::Box {
            x_mm: dx,
            y_mm: dy,
            z_mm: dz,
        },
        deflection(),
        &limits(),
    )
    .expect("box")
}

fn box_at(origin: Point3, dx: f64, dy: f64, dz: f64) -> Brep {
    let profile = Profile::new(vec![
        [origin[0], origin[1], origin[2]],
        [origin[0] + dx, origin[1], origin[2]],
        [origin[0] + dx, origin[1] + dy, origin[2]],
        [origin[0], origin[1] + dy, origin[2]],
    ])
    .expect("box profile");
    creation::extrude(&profile, [0.0, 0.0, dz], true, deflection(), &limits()).expect("box prism")
}

fn rect_face(width: f64, height: f64) -> Brep {
    let profile = Profile::new(vec![
        [0.0, 0.0, 0.0],
        [width, 0.0, 0.0],
        [width, height, 0.0],
        [0.0, height, 0.0],
    ])
    .expect("rectangle profile");
    creation::planar(&profile, deflection(), &limits()).expect("planar face")
}

#[test]
fn boss_smooth_nurbs_line_polycurve_on_cylinder_matches_independent_integral() {
    use om9_core::{evaluation::{CurveEvaluator,Parameter},geometry::{Curve,NurbsCurve,Transform}};
    let cylinder=creation::primitive(Primitive::Cylinder {radius_mm:5.,height_mm:10.},deflection(),&limits()).unwrap();
    let boundary=(0..measured(&cylinder).faces).map(|i| editing::extract_face(&cylinder,i,deflection(),&limits()).unwrap())
        .find(|f| close(measured(f).area_mm2,100.*PI,1e-6)).unwrap();
    let arch=Curve::Nurbs(NurbsCurve {degree:2,poles:vec![[-1.,0.,3.],[-1.,0.,4.],[1.,0.,4.],[1.,0.,3.]],weights:vec![1.,0.7,1.2,1.],
        knots:vec![0.,0.,0.,0.5,1.,1.,1.],domain:[0.,1.],periodic:false,reversed:false,seam:None});
    let line=Curve::Line {start:[1.,0.,3.],end:[-1.,0.,3.],domain:[0.,1.]};
    let evaluator=CurveEvaluator::capture(&arch,&limits()).unwrap(); let count=8192; let dt=1./count as f64;
    let mut wall=24.0_f64.sqrt()+25.*0.2_f64.asin(); let mut signed_hole=0.;
    for i in 0..count { let t=(i as f64+0.5)*dt; let d=evaluator.evaluate(Parameter::Native(t),1,Transform::identity()).unwrap(); let x=d[0][0];
        wall+=(25.-x*x).sqrt()*d[1][0].hypot(d[1][2])*dt; signed_hole+=5.*(x/5.).asin()*d[1][2]*dt; }
    let expected=100.*PI-signed_hole.abs()+wall;
    let mut baseline=None;
    for (outer,inner) in [(false,false),(false,true),(true,false),(true,true)] {
        let mut part=arch.clone(); if let Curve::Nurbs(n)=&mut part {n.reversed=inner;}
        let chord=if inner {Curve::Line {start:[-1.,0.,3.],end:[1.,0.,3.],domain:[0.,1.]}} else {line.clone()};
        let curve=Curve::Polycurve {segments:vec![part,chord],reversed:outer};
        let body=editing::boss_cylindrical_sections(&[editing::BossSection::Curve(curve)],&boundary,deflection(),&limits()).unwrap();
        let m=measured(&body); assert!(close(m.area_mm2,expected,2e-5),"outer={outer} inner={inner} {m:?} expected {expected}"); assert_eq!(m.faces,4); assert_eq!(m.shells,1);
        let plus=box_at([-0.05,4.8,3.1],0.1,0.4,0.1); let minus=box_at([-0.05,-5.2,3.1],0.1,0.4,0.1);
        let plus_empty=matches!(editing::intersect_surface_with_solid(&body,&plus,deflection(),&limits()).unwrap(),BooleanOutcome::Empty);
        let minus_empty=matches!(editing::intersect_surface_with_solid(&body,&minus,deflection(),&limits()).unwrap(),BooleanOutcome::Empty);
        assert_ne!(plus_empty,minus_empty);
        let first=*baseline.get_or_insert(plus_empty); assert_eq!(plus_empty,if outer^inner {!first} else {first});
    }
}

#[test]
fn boss_smooth_closed_nurbs_on_cylinder_matches_independent_integrals() {
    use om9_core::{evaluation::{CurveEvaluator,Parameter},geometry::{Curve,NurbsCurve,Transform}};
    let cylinder=creation::primitive(Primitive::Cylinder {radius_mm:5.,height_mm:10.},deflection(),&limits()).unwrap();
    let boundary=(0..measured(&cylinder).faces).map(|i| editing::extract_face(&cylinder,i,deflection(),&limits()).unwrap())
        .find(|f| close(measured(f).area_mm2,100.*PI,1e-6)).unwrap();
    for knots in [vec![0.,0.,0.,0.25,0.5,0.75,1.,1.,1.],vec![0.,0.,0.,0.13,0.41,0.86,1.,1.,1.]] {
    let n=NurbsCurve {degree:2,poles:vec![[1.,0.,3.],[1.,0.,4.],[-1.,0.,4.],[-1.,0.,2.],[1.,0.,2.],[1.,0.,3.]],weights:vec![1.,0.7,1.2,0.8,1.3,1.],
        knots:knots.clone(),domain:[0.,1.],periodic:false,reversed:false,seam:None};
    let mut refined=n.clone(); for knot in [knots[3],knots[4],knots[5]] {refined=om9_core::curve_build::insert_knot(&refined,knot,1,&limits()).unwrap();}
    let curve=Curve::Nurbs(n); let evaluator=CurveEvaluator::capture(&curve,&limits()).unwrap();
    let refined_curve=Curve::Nurbs(refined); let refined_eval=CurveEvaluator::capture(&refined_curve,&limits()).unwrap();
    for i in 0..=128 { let t=i as f64/128.; let a=evaluator.evaluate(Parameter::Native(t),0,Transform::identity()).unwrap()[0]; let b=refined_eval.evaluate(Parameter::Native(t),0,Transform::identity()).unwrap()[0];
        assert!((0..3).all(|axis|(a[axis]-b[axis]).abs()<1e-10),"t={t} {a:?} {b:?}"); }
    let count=8192; let dt=1./count as f64;
    let mut wall=0.; let mut signed_hole=0.;
    for i in 0..count { let t=(i as f64+0.5)*dt; let d=evaluator.evaluate(Parameter::Native(t),1,Transform::identity()).unwrap();
        let x=d[0][0]; wall+=(25.-x*x).sqrt()*d[1][0].hypot(d[1][2])*dt; signed_hole+=5.*(x/5.).asin()*d[1][2]*dt; }
    let expected=100.*PI-signed_hole.abs()+wall;
    let restricted=Limits {max_outputs:3,..limits()}; let before=boundary.clone();
    assert!(editing::boss_cylindrical_sections(&[editing::BossSection::Curve(curve.clone())],&boundary,deflection(),&restricted).is_err());
    assert_eq!(boundary,before);
    for reversed in [false,true] {
        let mut source=curve.clone(); if let Curve::Nurbs(n)=&mut source {n.reversed=reversed;}
        let body=editing::boss_cylindrical_sections(&[editing::BossSection::Curve(source)],&boundary,deflection(),&limits()).unwrap();
        let m=measured(&body); assert!(close(m.area_mm2,expected,2e-5),"{m:?} expected {expected}"); assert_eq!(m.shells,1); assert_eq!(m.faces,5);
        let plus=box_at([-0.05,4.8,2.95],0.1,0.4,0.1); let minus=box_at([-0.05,-5.2,2.95],0.1,0.4,0.1);
        let (hole,opposite)=if reversed {(&plus,&minus)} else {(&minus,&plus)};
        assert!(matches!(editing::intersect_surface_with_solid(&body,hole,deflection(),&limits()).unwrap(),BooleanOutcome::Empty),"reversed={reversed}");
        assert!(matches!(editing::intersect_surface_with_solid(&body,opposite,deflection(),&limits()).unwrap(),BooleanOutcome::Body(_)));
    }
    }
}

#[test]
fn boss_reversed_nurbs_ellipse_changes_cylinder_side() {
    use om9_core::geometry::Curve;
    let cylinder=creation::primitive(Primitive::Cylinder {radius_mm:5.,height_mm:10.},deflection(),&limits()).unwrap();
    let boundary=(0..measured(&cylinder).faces).map(|i| editing::extract_face(&cylinder,i,deflection(),&limits()).unwrap())
        .find(|f| close(measured(f).area_mm2,100.*PI,1e-6)).unwrap();
    let plus=box_at([-0.1,4.8,2.9],0.2,0.4,0.2); let minus=box_at([-0.1,-5.2,2.9],0.2,0.4,0.2);
    let restricted=Limits {max_outputs:3,..limits()};
    let ellipse=om9_core::curve_build::ellipse([0.,0.,3.],[1.,0.,0.],[0.,0.,1.],1.,0.5,&limits()).unwrap();
    let original=boundary.clone();
    assert!(editing::boss_cylindrical_sections(&[editing::BossSection::Curve(Curve::Nurbs(ellipse))],&boundary,deflection(),&restricted).is_err());
    assert_eq!(boundary,original);
    for reversed in [false,true] {
        let mut ellipse=om9_core::curve_build::ellipse([0.,0.,3.],[1.,0.,0.],[0.,0.,1.],1.,0.5,&limits()).unwrap(); ellipse.reversed=reversed;
        let body=editing::boss_cylindrical_sections(&[editing::BossSection::Curve(Curve::Nurbs(ellipse))],&boundary,deflection(),&limits()).unwrap();
        let (hole,opposite)=if reversed {(&plus,&minus)} else {(&minus,&plus)};
        assert!(matches!(editing::intersect_surface_with_solid(&body,hole,deflection(),&limits()).unwrap(),BooleanOutcome::Empty),"reversed={reversed}");
        assert!(matches!(editing::intersect_surface_with_solid(&body,opposite,deflection(),&limits()).unwrap(),BooleanOutcome::Body(_)));
    }
}

#[test]
fn boss_piecewise_nurbs_ellipse_on_cylinder_matches_independent_integral() {
    use om9_core::geometry::{Curve,NurbsCurve};
    let cylinder=creation::primitive(Primitive::Cylinder {radius_mm:5.,height_mm:10.},deflection(),&limits()).unwrap();
    let boundary=(0..measured(&cylinder).faces).map(|i| editing::extract_face(&cylinder,i,deflection(),&limits()).unwrap())
        .find(|f| close(measured(f).area_mm2,100.*PI,1e-6)).unwrap();
    let ellipse=om9_core::curve_build::ellipse([0.,0.,3.],[1.,0.,0.],[0.,0.,1.],1.,0.5,&limits()).unwrap();
    let segments=(0..4).map(|i| Curve::Nurbs(NurbsCurve {degree:2,poles:ellipse.poles[2*i..=2*i+2].to_vec(),weights:ellipse.weights[2*i..=2*i+2].to_vec(),knots:vec![0.,0.,0.,1.,1.,1.],domain:[0.,1.],periodic:false,reversed:false,seam:None})).collect();
    let curve=Curve::Polycurve {segments,reversed:false};
    let result=editing::boss_cylindrical_sections(&[editing::BossSection::Curve(curve)],&boundary,deflection(),&limits()).unwrap();
    let m=measured(&result); let count=32768; let dt=2.*PI/count as f64; let a=1.0_f64; let b=0.5_f64;
    let wall:f64=(0..count).map(|i| {let t=(i as f64+0.5)*dt; (25.-(a*t.cos()).powi(2)).sqrt()*((a*t.sin()).powi(2)+(b*t.cos()).powi(2)).sqrt()*dt}).sum();
    let du=PI/count as f64; let hole:f64=(0..count).map(|i| {let t=-PI/2.+(i as f64+0.5)*du; 10.*a*b*t.cos().powi(2)/(25.-(a*t.sin()).powi(2)).sqrt()*du}).sum();
    let expected=100.*PI-hole+wall;
    assert!(close(m.area_mm2,expected,1e-5),"{m:?} expected {expected}"); assert_eq!(m.faces,5); assert_eq!(m.shells,1); assert_eq!(m.naked_edges,6);
}

#[test]
fn boss_cylinder_mixes_circle_and_polygon_without_overlap() {
    let cylinder=creation::primitive(Primitive::Cylinder {radius_mm:5.,height_mm:10.},deflection(),&limits()).unwrap();
    let boundary=(0..measured(&cylinder).faces).map(|i| editing::extract_face(&cylinder,i,deflection(),&limits()).unwrap())
        .find(|f| close(measured(f).area_mm2,100.*PI,1e-6)).unwrap();
    let circle=om9_core::curve_build::circle_on_frame([0.,0.,2.],[1.,0.,0.],[0.,0.,1.],0.5).unwrap();
    let p=Profile::new(vec![[-1.,0.,5.],[1.,0.,5.],[1.,0.,6.],[-1.,0.,6.]]).unwrap();
    let body=editing::boss_cylindrical_sections(&[editing::BossSection::Circle(circle),editing::BossSection::Polygon(p)],&boundary,deflection(),&limits()).unwrap();
    let m=measured(&body); let count=32768; let dt=2.*PI/count as f64;
    let wall:f64=(0..count).map(|i| {let t=(i as f64+0.5)*dt; (25.-(0.5*t.cos()).powi(2)).sqrt()*0.5*dt}).sum();
    let du=PI/count as f64;
    let hole:f64=(0..count).map(|i| {let t=-PI/2.+(i as f64+0.5)*du; 2.5*t.cos().powi(2)/(25.-(0.5*t.sin()).powi(2)).sqrt()*du}).sum();
    let expected=100.*PI+wall-hole+4.*24.0_f64.sqrt()+40.*0.2_f64.asin();
    assert!(close(m.area_mm2,expected,1e-5),"{m:?} expected {expected}"); assert_eq!(m.faces,6); assert_eq!(m.shells,1); assert_eq!(m.naked_edges,7);
}

#[test]
fn boss_circle_and_ellipse_on_cylinder_match_independent_integral() {
    let cylinder=creation::primitive(Primitive::Cylinder {radius_mm:5.,height_mm:10.},deflection(),&limits()).unwrap();
    let boundary=(0..measured(&cylinder).faces).map(|i| editing::extract_face(&cylinder,i,deflection(),&limits()).unwrap())
        .find(|f| close(measured(f).area_mm2,100.*PI,1e-6)).unwrap();
    for (circle,nonuniform) in [(true,false),(false,false),(false,true)] {
        let curve=if circle { om9_core::curve_build::circle_on_frame([0.,0.,3.],[1.,0.,0.],[0.,0.,1.],1.).unwrap() }
            else { let mut e=om9_core::curve_build::ellipse([0.,0.,3.],[1.,0.,0.],[0.,0.,1.],1.,0.5,&limits()).unwrap();
                if nonuniform { e.knots=vec![0.,0.,0.,0.1,0.1,0.35,0.35,0.8,0.8,1.,1.,1.]; }
                om9_core::geometry::Curve::Nurbs(e) };
        let section=if circle { editing::BossSection::Circle(curve) } else { editing::BossSection::Curve(curve) };
        let saved=boundary.clone();
        let result=editing::boss_cylindrical_sections(&[section],&boundary,deflection(),&limits());
        let body=result.unwrap();
        let m=measured(&body); let a=1.0_f64; let b=if circle {1.} else {0.5}; let count=32768; let dt=2.*PI/count as f64;
        let wall:f64=(0..count).map(|i| {let t=(i as f64+0.5)*dt; (25.-(a*t.cos()).powi(2)).sqrt()*((a*t.sin()).powi(2)+(b*t.cos()).powi(2)).sqrt()*dt}).sum();
        let du=PI/count as f64;
        let hole:f64=(0..count).map(|i| {let t=-PI/2.+(i as f64+0.5)*du; 10.*a*b*t.cos().powi(2)/(25.-(a*t.sin()).powi(2)).sqrt()*du}).sum();
        let expected=100.*PI-hole+wall;
        assert!(close(m.area_mm2,expected,1e-5),"circle={circle} {m:?} expected {expected}");
        assert_eq!(m.faces,if circle {2} else {5}); assert_eq!(m.shells,1); assert_eq!(m.naked_edges,if circle {3} else {6});
        assert_eq!(boundary,saved);
        if !circle { continue; }
        let plus=box_at([-0.1,4.8,2.9],0.2,0.4,0.2); let minus=box_at([-0.1,-5.2,2.9],0.2,0.4,0.2);
        assert!(matches!(editing::intersect_surface_with_solid(&body,&minus,deflection(),&limits()).unwrap(),BooleanOutcome::Empty));
        assert!(matches!(editing::intersect_surface_with_solid(&body,&plus,deflection(),&limits()).unwrap(),BooleanOutcome::Body(_)));
        let flipped=om9_core::curve_build::circle_on_frame([0.,0.,3.],[1.,0.,0.],[0.,0.,-1.],1.).unwrap();
        let flipped=editing::boss_cylindrical_sections(&[editing::BossSection::Circle(flipped)],&boundary,deflection(),&limits()).unwrap();
        assert!(matches!(editing::intersect_surface_with_solid(&flipped,&plus,deflection(),&limits()).unwrap(),BooleanOutcome::Empty));
        assert!(matches!(editing::intersect_surface_with_solid(&flipped,&minus,deflection(),&limits()).unwrap(),BooleanOutcome::Body(_)));
    }
}

#[test]
fn boss_oblique_circle_and_ellipse_on_cylinder_match_independent_integrals() {
    use om9_core::geometry::Curve;
    let cylinder=creation::primitive(Primitive::Cylinder {radius_mm:5.,height_mm:10.},deflection(),&limits()).unwrap();
    let boundary=(0..measured(&cylinder).faces).map(|i| editing::extract_face(&cylinder,i,deflection(),&limits()).unwrap())
        .find(|f| close(measured(f).area_mm2,100.*PI,1e-6)).unwrap();
    let saved=boundary.clone(); let c=3.0_f64.sqrt()/2.; let a=1.0_f64;
    for (circle,b) in [(true,1.0_f64),(false,0.5_f64)] {
        let count=32768; let dt=2.*PI/count as f64;
        let wall:f64=(0..count).map(|i| {let t=(i as f64+0.5)*dt; (25.-(a*t.cos()).powi(2)).sqrt()*((a*t.sin()).powi(2)+(b*t.cos()).powi(2)).sqrt()*dt}).sum();
        let du=PI/count as f64;
        let hole:f64=(0..count).map(|i| {let t=-PI/2.+(i as f64+0.5)*du; 10.*a*b*t.cos().powi(2)/(25.-(a*t.sin()).powi(2)).sqrt()*du}).sum();
        let expected=100.*PI+(wall-hole)/c;
        for reversed in [false,true] {
            let v=if reversed {[0.,-0.5,-c]} else {[0.,0.5,c]};
            let curve=if circle {om9_core::curve_build::circle_on_frame([0.,0.,5.],[1.,0.,0.],v,a).unwrap()}
                else {Curve::Nurbs(om9_core::curve_build::ellipse([0.,0.,5.],[1.,0.,0.],v,a,b,&limits()).unwrap())};
            let section=if circle {editing::BossSection::Circle(curve)} else {editing::BossSection::Curve(curve)};
            let body=editing::boss_cylindrical_sections(&[section],&boundary,deflection(),&limits()).unwrap_or_else(|e| panic!("circle={circle} reversed={reversed}: {e:?}"));
            let m=measured(&body); assert!(close(m.area_mm2,expected,2e-5),"circle={circle} reversed={reversed}: {m:?} expected={expected}");
            assert_eq!(m.shells,1); assert_eq!(m.faces,if circle {2} else {5});
        }
    }
    // This reversed circle reaches slightly beyond the cylinder's lower end.
    let crossing=om9_core::curve_build::circle_on_frame([0.,0.,4.],[1.,0.,0.],[0.,-0.5,-c],1.).unwrap();
    assert!(editing::boss_cylindrical_sections(&[editing::BossSection::Circle(crossing)],&boundary,deflection(),&limits()).is_err());
    assert_eq!(boundary,saved);
}

#[test]
fn boss_oblique_polygon_on_cylinder_matches_independent_integral() {
    let cylinder=creation::primitive(Primitive::Cylinder {radius_mm:5.,height_mm:10.},deflection(),&limits()).unwrap();
    let boundary=(0..measured(&cylinder).faces).map(|i| editing::extract_face(&cylinder,i,deflection(),&limits()).unwrap())
        .find(|f| close(measured(f).area_mm2,100.*PI,1e-6)).unwrap();
    let before=boundary.clone();
    let c=3.0_f64.sqrt()/2.;
    let points=vec![[-1.,0.,4.],[1.,0.,4.],[1.,0.5,4.+c],[-1.,0.5,4.+c]];
    for (reverse,sign) in [(false,1.),(true,-1.)] {
        let mut p=points.clone(); if reverse { p.reverse(); }
        let profile=Profile::new(p).unwrap();
        let body=editing::boss_cylindrical_polygons(&[&profile],&boundary,deflection(),&limits()).unwrap();
        let m=measured(&body);
        let expected=100.*PI+(4.*24.0_f64.sqrt()+40.*0.2_f64.asin()+sign*1.5)/c;
        assert!(close(m.area_mm2,expected,1e-5),"{m:?} expected {expected}");
        assert_eq!((m.faces,m.shells,m.naked_edges),(5,1,6));
    }
    let outside=Profile::new(vec![[-1.,0.,8.],[1.,0.,8.],[1.,0.5,8.+c],[-1.,0.5,8.+c]]).unwrap();
    assert!(editing::boss_cylindrical_polygons(&[&outside],&boundary,deflection(),&limits()).is_err());
    let partial=Profile::new(vec![[-1.,0.,6.8],[1.,0.,6.8],[1.,0.5,6.8+c],[-1.,0.5,6.8+c]]).unwrap();
    assert!(editing::boss_cylindrical_polygons(&[&partial],&boundary,deflection(),&limits()).is_err());
    let axial=Profile::new(vec![[-1.,-1.,3.],[1.,-1.,3.],[1.,1.,3.],[-1.,1.,3.]]).unwrap();
    assert!(editing::boss_cylindrical_polygons(&[&axial],&boundary,deflection(),&limits()).is_err());
    assert_eq!(boundary,before);
}

#[test]
fn boss_cylinder_multiple_polygons_join_atomically_and_refuse_overlap() {
    let cylinder=creation::primitive(Primitive::Cylinder {radius_mm:5.,height_mm:10.},deflection(),&limits()).unwrap();
    let boundary=(0..measured(&cylinder).faces).map(|i| editing::extract_face(&cylinder,i,deflection(),&limits()).unwrap())
        .find(|f| close(measured(f).area_mm2,100.*PI,1e-6)).unwrap();
    let before=boundary.clone();
    let a=Profile::new(vec![[-1.,0.,2.],[1.,0.,2.],[1.,0.,3.],[-1.,0.,3.]]).unwrap();
    let b=Profile::new(vec![[-1.,0.,6.],[1.,0.,6.],[1.,0.,7.],[-1.,0.,7.]]).unwrap();
    let body=editing::boss_cylindrical_polygons(&[&a,&b],&boundary,deflection(),&limits()).unwrap();
    let m=measured(&body); let expected=100.*PI+2.*(4.*24.0_f64.sqrt()+40.*0.2_f64.asin());
    assert!(close(m.area_mm2,expected,1e-5),"{m:?} expected {expected}");
    assert_eq!(m.faces,9); assert_eq!(m.shells,1); assert_eq!(m.naked_edges,10);
    let overlap=Profile::new(vec![[-1.,0.,2.5],[1.,0.,2.5],[1.,0.,3.5],[-1.,0.,3.5]]).unwrap();
    assert!(editing::boss_cylindrical_polygons(&[&a,&overlap],&boundary,deflection(),&limits()).is_err());
    let opposite=Profile::new(vec![[-1.,0.,3.],[1.,0.,3.],[1.,0.,2.],[-1.,0.,2.]]).unwrap();
    // Coincident profile curves on opposite sides produce a nonmanifold join.
    assert!(editing::boss_cylindrical_polygons(&[&a,&opposite],&boundary,deflection(),&limits()).is_err());
    let opposite_disjoint=Profile::new(vec![[-1.,0.,7.],[1.,0.,7.],[1.,0.,6.],[-1.,0.,6.]]).unwrap();
    let both_sides=editing::boss_cylindrical_polygons(&[&a,&opposite_disjoint],&boundary,deflection(),&limits()).unwrap();
    let both=measured(&both_sides); assert_eq!(both.faces,9); assert_eq!(both.shells,1); assert!(close(both.area_mm2,expected,1e-5));
    let restricted=Limits {max_outputs:1,..limits()};
    assert!(editing::boss_cylindrical_polygons(&[&a,&b],&boundary,deflection(),&restricted).is_err());
    assert!(editing::boss_cylindrical_polygons(&[],&boundary,deflection(),&limits()).is_err());
    assert_eq!(boundary,before);
}

#[test]
fn boss_cylindrical_boundary_kernel_construction_uses_exact_carrier() {
    let cylinder=creation::primitive(Primitive::Cylinder { radius_mm:5.,height_mm:10. },deflection(),&limits()).unwrap();
    let cylinder_before=cylinder.clone();
    let boundary=(0..measured(&cylinder).faces).map(|i| editing::extract_face(&cylinder,i,deflection(),&limits()).unwrap())
        .find(|face| close(measured(face).area_mm2,100.*PI,1e-6)).unwrap();
    let before=boundary.clone();
    let (carrier,axis)=editing::boss_cylinder_carrier(&boundary,deflection(),&limits()).unwrap();
    assert!(close(axis[6],5.,1e-10)); assert!(close(axis[5].abs(),1.,1e-10));
    assert!(close(measured(&carrier).volume_mm3,300.*PI,1e-3));
    assert!(editing::boss_cylinder_carrier(&rect_face(4.,4.),deflection(),&limits()).is_err());
    let translated=om9_kernel::transform_ops::transform_brep(&boundary,&om9_core::geometry::Transform::translation([10.,20.,30.]),deflection(),&limits()).unwrap();
    // Current general affine BRep transform converts the cylinder to a spline surface.
    assert!(editing::boss_cylinder_carrier(&translated,deflection(),&limits()).is_err());
    let profile=Profile::new(vec![[-1.,0.,2.],[1.,0.,2.],[1.,0.,4.],[-1.,0.,4.]]).unwrap();
    let tool=creation::extrude(&profile,[0.,10.,0.],true,deflection(),&limits()).unwrap();
    let wall=creation::extrude(&profile,[0.,10.,0.],false,deflection(),&limits()).unwrap();
    let trimmed=editing::trim_surface_by_solid(&boundary,&tool,deflection(),&limits()).unwrap().body().unwrap().clone();
    let clipped=editing::intersect_surface_with_solid(&wall,&carrier,deflection(),&limits()).unwrap().body().unwrap().clone();
    let far=box_at([20.,20.,20.],1.,1.,1.);
    assert!(matches!(editing::intersect_surface_with_solid(&wall,&far,deflection(),&limits()).unwrap(),BooleanOutcome::Empty));
    assert!(editing::intersect_surface_with_solid(&cylinder,&wall,deflection(),&limits()).is_err());
    let joined=editing::sew(&trimmed,&clipped,1e-7,deflection(),&limits()).unwrap();
    let m=measured(&joined); let expected=100.*PI+6.*24.0_f64.sqrt()+30.*0.2_f64.asin();
    assert!(close(m.area_mm2,expected,1e-5),"{m:?} expected {expected}");
    assert_eq!(m.faces,5); assert_eq!(m.shells,1); assert_eq!(m.naked_edges,6);
    assert_eq!(boundary,before); assert_eq!(cylinder,cylinder_before);
    for (reversed,points) in [vec![[-1.,0.,2.],[1.,0.,2.],[1.,0.,4.],[-1.,0.,4.]], vec![[-1.,0.,4.],[1.,0.,4.],[1.,0.,2.],[-1.,0.,2.]]].into_iter().enumerate() {
        let profile=Profile::new(points).unwrap();
        let body=editing::boss_cylindrical_polygon(&profile,&boundary,deflection(),&limits()).unwrap();
        let m=measured(&body);
        assert!(close(m.area_mm2,expected,1e-5),"{m:?}"); assert_eq!(m.faces,5); assert_eq!(m.shells,1); assert_eq!(m.naked_edges,6);
        let plus=box_at([-0.2,4.8,2.5],0.4,0.4,1.0);
        let minus=box_at([-0.2,-5.2,2.5],0.4,0.4,1.0);
        let (hole,opposite)=if reversed==0 { (&minus,&plus) } else { (&plus,&minus) };
        assert!(matches!(editing::intersect_surface_with_solid(&body,hole,deflection(),&limits()).unwrap(),BooleanOutcome::Empty));
        assert!(matches!(editing::intersect_surface_with_solid(&body,opposite,deflection(),&limits()).unwrap(),BooleanOutcome::Body(_)));
    }
    let outside=Profile::new(vec![[5.1,0.,2.],[6.1,0.,2.],[6.1,0.,4.],[5.1,0.,4.]]).unwrap();
    assert!(editing::boss_cylindrical_polygon(&outside,&boundary,deflection(),&limits()).is_err());
    let beyond=Profile::new(vec![[-1.,0.,20.],[1.,0.,20.],[1.,0.,22.],[-1.,0.,22.]]).unwrap();
    assert!(editing::boss_cylindrical_polygon(&beyond,&boundary,deflection(),&limits()).is_err());
    let along_axis=Profile::new(vec![[-1.,-1.,2.],[1.,-1.,2.],[1.,1.,2.],[-1.,1.,2.]]).unwrap();
    assert!(editing::boss_cylindrical_polygon(&along_axis,&boundary,deflection(),&limits()).is_err());
    assert!(editing::boss_cylindrical_polygon(&profile,&trimmed,deflection(),&limits()).is_err());
}

#[test]
fn boss_piecewise_nurbs_ellipse_near_inclined_boundary() {
    use om9_core::geometry::{Curve,NurbsCurve};
    let h=std::f64::consts::FRAC_1_SQRT_2;
    let ellipse=om9_core::curve_build::ellipse([5.,5.,0.],[h,h,0.],[-h,h,0.],2.,1.,&limits()).unwrap();
    let segments=(0..4).map(|i| Curve::Nurbs(NurbsCurve { degree:2, poles:ellipse.poles[2*i..=2*i+2].to_vec(),
        weights:ellipse.weights[2*i..=2*i+2].to_vec(), knots:vec![0.,0.,0.,1.,1.,1.], domain:[0.,1.], periodic:false,reversed:false,seam:None })).collect();
    let curve=Curve::Polycurve { segments,reversed:false };
    let p=Profile::new(vec![[0.,0.,-1.38],[10.,0.,0.62],[10.,10.,2.62],[0.,10.,0.62]]).unwrap();
    let boundary=creation::planar(&p,deflection(),&limits()).unwrap();
    let result=editing::boss_parallel_planar_sections(&[editing::BossSection::Curve(curve.clone())],&boundary,deflection(),&limits()).unwrap();
    let m=measured(&result); let dt=2.*PI/16384.;
    let perimeter: f64=(0..16384).map(|i| {let t=(i as f64+0.5)*dt; ((2.*t.sin()).powi(2)+t.cos().powi(2)).sqrt()*dt}).sum();
    let expected=(100.-2.*PI)*1.08_f64.sqrt()+0.62*perimeter;
    assert!(close(m.area_mm2,expected,1e-5),"{m:?} expected {expected}");
    assert_eq!(m.faces,5); assert_eq!(m.shells,1);
    let crossing=Profile::new(vec![[0.,0.,-2.],[10.,0.,0.],[10.,10.,2.],[0.,10.,0.]]).unwrap();
    let face=creation::planar(&crossing,deflection(),&limits()).unwrap(); let before=face.clone();
    assert!(editing::boss_parallel_planar_sections(&[editing::BossSection::Curve(curve)],&face,deflection(),&limits()).is_err());
    assert_eq!(face,before);
}

#[test]
fn boss_mixed_nurbs_polycurve_near_inclined_boundary_uses_projected_extent() {
    use om9_core::geometry::{Curve,NurbsCurve};
    let h=std::f64::consts::FRAC_1_SQRT_2;
    let arch=Curve::Nurbs(NurbsCurve { degree:2,
        poles:vec![[5.+2.*h,5.+2.*h,0.],[5.-2.*h,5.+2.*h,0.],[5.-2.*h,5.-2.*h,0.]],
        weights:vec![1.;3],knots:vec![0.,0.,0.,1.,1.,1.],domain:[0.,1.],periodic:false,reversed:false,seam:None });
    let chord=Curve::Line { start:[5.-2.*h,5.-2.*h,0.],end:[5.+2.*h,5.+2.*h,0.],domain:[0.,1.] };
    let curve=Curve::Polycurve { segments:vec![arch,chord],reversed:false };
    let p=Profile::new(vec![[0.,0.,-1.38],[10.,0.,0.62],[10.,10.,2.62],[0.,10.,0.62]]).unwrap();
    let boundary=creation::planar(&p,deflection(),&limits()).unwrap();
    let result=editing::boss_parallel_planar_sections(&[editing::BossSection::Curve(curve)],&boundary,deflection(),&limits()).unwrap();
    let m=measured(&result); let perimeter=4.+2.*2.0_f64.sqrt()+2.*1.0_f64.asinh();
    let expected=(100.-8./3.)*1.08_f64.sqrt()+0.62*perimeter;
    assert!(close(m.area_mm2,expected,1e-5),"{m:?} expected {expected}");
    assert_eq!(m.faces,3); assert_eq!(m.shells,1);
}

#[test]
fn boss_rotated_ellipse_near_inclined_boundary_uses_projected_extent() {
    let h = std::f64::consts::FRAC_1_SQRT_2;
    let ellipse = om9_core::geometry::Curve::Nurbs(om9_core::curve_build::ellipse([5.,5.,0.],[h,h,0.],[-h,h,0.],2.,1.,&limits()).unwrap());
    let p = Profile::new(vec![[0.,0.,-1.38],[10.,0.,0.62],[10.,10.,2.62],[0.,10.,0.62]]).unwrap();
    let boundary = creation::planar(&p, deflection(), &limits()).unwrap();
    let result = editing::boss_parallel_planar_sections(&[editing::BossSection::Curve(ellipse)], &boundary, deflection(), &limits()).unwrap();
    let m = measured(&result); let dt=2.*PI/16384.;
    let perimeter: f64 = (0..16384).map(|i| { let t=(i as f64+0.5)*dt; ((2.*t.sin()).powi(2)+t.cos().powi(2)).sqrt()*dt }).sum();
    let expected = (100.-2.*PI)*1.08_f64.sqrt()+0.62*perimeter;
    assert!(close(m.area_mm2, expected, 1e-5), "{m:?} expected {expected}");
    assert_eq!(m.faces,2); assert_eq!(m.shells,1);
    let crossing = Profile::new(vec![[0.,0.,-2.],[10.,0.,0.],[10.,10.,2.],[0.,10.,0.]]).unwrap();
    let cross_face = creation::planar(&crossing, deflection(), &limits()).unwrap();
    let original = cross_face.clone();
    let circle = om9_core::geometry::Curve::Nurbs(om9_core::curve_build::ellipse([5.,5.,0.],[h,h,0.],[-h,h,0.],2.,1.,&limits()).unwrap());
    assert!(editing::boss_parallel_planar_sections(&[editing::BossSection::Curve(circle)], &cross_face, deflection(), &limits()).is_err());
    assert_eq!(cross_face, original);
}

#[test]
fn boss_nurbs_ellipse_joins_inclined_planar_boundary() {
    for sign in [-1.,1.] {
    let profile = Profile::new(vec![[0.,0.,5.*sign],[10.,0.,7.*sign],[10.,10.,7.*sign],[0.,10.,5.*sign]]).unwrap();
    let boundary = creation::planar(&profile, deflection(), &limits()).unwrap();
    let ellipse = om9_core::geometry::Curve::Nurbs(om9_core::curve_build::ellipse([5.,5.,0.],[1.,0.,0.],[0.,1.,0.],2.,1.,&limits()).unwrap());
    let result = editing::boss_parallel_planar_sections(&[editing::BossSection::Curve(ellipse)], &boundary, deflection(), &limits()).unwrap();
    let m = measured(&result);
    let count = 16384; let dt = 2.*PI/count as f64;
    let perimeter: f64 = (0..count).map(|i| { let t=(i as f64+0.5)*dt; ((2.*t.sin()).powi(2)+t.cos().powi(2)).sqrt()*dt }).sum();
    let expected = (100.-2.*PI)*1.04_f64.sqrt()+6.*perimeter;
    assert!(close(m.area_mm2, expected, 1e-5), "{m:?} expected {expected}");
    assert_eq!(m.faces, 2); assert_eq!(m.shells, 1); assert_eq!(m.naked_edges, 5);
    }
}

#[test]
fn boss_circle_joins_inclined_planar_boundary() {
    for sign in [-1., 1.] {
        let profile = Profile::new(vec![[0.,0.,5.*sign],[10.,0.,7.*sign],[10.,10.,7.*sign],[0.,10.,5.*sign]]).unwrap();
        let boundary = creation::planar(&profile, deflection(), &limits()).unwrap();
        let circle = om9_core::curve_build::circle_on_frame([2.,3.,0.],[1.,0.,0.],[0.,1.,0.],1.).unwrap();
        let result = editing::boss_parallel_planar_sections(&[editing::BossSection::Circle(circle)], &boundary, deflection(), &limits()).unwrap();
        let m = measured(&result);
        let expected = (100.-PI)*1.04_f64.sqrt()+10.8*PI;
        assert!(close(m.area_mm2, expected, 1e-5), "{m:?} expected {expected}");
        assert_eq!(m.faces, 2); assert_eq!(m.shells, 1); assert_eq!(m.naked_edges, 5);
    }
    let crossing = Profile::new(vec![[0.,0.,-0.4],[10.,0.,1.6],[10.,10.,1.6],[0.,10.,-0.4]]).unwrap();
    let boundary = creation::planar(&crossing, deflection(), &limits()).unwrap();
    let before = boundary.clone();
    let circle = om9_core::curve_build::circle_on_frame([2.,3.,0.],[1.,0.,0.],[0.,1.,0.],1.).unwrap();
    assert!(editing::boss_parallel_planar_sections(&[editing::BossSection::Circle(circle)], &boundary, deflection(), &limits()).is_err());
    assert_eq!(boundary, before);
}

#[test]
fn boss_polygon_joins_inclined_planar_boundary() {
    for sign in [-1., 1.] {
        let boundary_profile = Profile::new(vec![[0.,0.,5.*sign],[10.,0.,7.*sign],[10.,10.,7.*sign],[0.,10.,5.*sign]]).unwrap();
        let boundary = creation::planar(&boundary_profile, deflection(), &limits()).unwrap();
        let before = boundary.clone();
        let profile = Profile::new(vec![[1.,2.,0.],[3.,2.,0.],[3.,4.,0.],[1.,4.,0.]]).unwrap();
        let result = editing::boss_parallel_planar_face(&profile, &boundary, deflection(), &limits()).unwrap();
        let m = measured(&result);
        assert!(close(m.area_mm2, 96.*1.04_f64.sqrt()+43.2, 1e-5), "{m:?}");
        assert_eq!(m.faces, 5); assert_eq!(m.shells, 1); assert_eq!(m.naked_edges, 8);
        assert_eq!(boundary, before);
    }
    let profile = Profile::new(vec![[1.,2.,0.],[3.,2.,0.],[3.,4.,0.],[1.,4.,0.]]).unwrap();
    let crossing = Profile::new(vec![[0.,0.,-2.],[10.,0.,8.],[10.,10.,8.],[0.,10.,-2.]]).unwrap();
    let boundary = creation::planar(&crossing, deflection(), &limits()).unwrap();
    let before = boundary.clone();
    assert!(editing::boss_parallel_planar_face(&profile, &boundary, deflection(), &limits()).is_err());
    assert_eq!(boundary, before);
    let partial = Profile::new(vec![[0.,0.,5.],[2.,0.,5.4],[2.,10.,5.4],[0.,10.,5.]]).unwrap();
    let partial_face = creation::planar(&partial, deflection(), &limits()).unwrap();
    assert!(editing::boss_parallel_planar_face(&profile, &partial_face, deflection(), &limits()).is_err());
}

#[test]
fn rib_straight_planar_foundation_builds_joined_shell() {
    let boundary_profile=Profile::new(vec![[0.,0.,5.],[10.,0.,5.],[10.,10.,5.],[0.,10.,5.]]).unwrap();
    let boundary=creation::planar(&boundary_profile,deflection(),&limits()).unwrap();let before=boundary.clone();
    for distance in [-2.,2.] {for reversed in [false,true] {
        let points=if reversed {[[6.,5.,0.],[2.,5.,0.]]} else {[[2.,5.,0.],[6.,5.,0.]]};
        let result=editing::rib_straight_planar_boundary(&points,[0.,0.,1.],distance,&boundary,deflection(),&limits()).unwrap();
        let m=measured(&result);assert_eq!(m.faces,5);assert_eq!(m.shells,1);assert_eq!(m.naked_edges,8);
        assert!(close(m.area_mm2,152.,1e-6),"{distance} {m:?}");assert_eq!(boundary,before);
    }}
    assert!(editing::rib_straight_planar_boundary(&[[2.,5.,0.],[6.,5.,0.]],[0.,0.,1.],0.,&boundary,deflection(),&limits()).is_err());
    assert!(editing::rib_straight_planar_boundary(&[[20.,5.,0.],[24.,5.,0.]],[0.,0.,1.],2.,&boundary,deflection(),&limits()).is_err());
    assert_eq!(boundary,before);
}

#[test]
fn rib_straight_planar_foundation_accepts_disjoint_curves_atomically() {
    let profile=Profile::new(vec![[0.,0.,5.],[10.,0.,5.],[10.,10.,5.],[0.,10.,5.]]).unwrap();
    let boundary=creation::planar(&profile,deflection(),&limits()).unwrap();let before=boundary.clone();
    let first=[[2.,3.,0.],[6.,3.,0.]];let second=[[2.,8.,0.],[6.,8.,0.]];
    let curves=[(&first[..],[0.,0.,1.]),(&second[..],[0.,0.,1.])];
    let result=editing::rib_straight_planar_boundary_many(&curves,1.,&boundary,deflection(),&limits()).unwrap();
    let m=measured(&result);assert_eq!(m.faces,9);assert_eq!(m.shells,1);assert!(close(m.area_mm2,192.,1e-6),"{m:?}");
    assert_eq!(boundary,before);
    let outside=[[20.,3.,0.],[24.,3.,0.]];let invalid=[(&first[..],[0.,0.,1.]),(&outside[..],[0.,0.,1.])];
    assert!(editing::rib_straight_planar_boundary_many(&invalid,1.,&boundary,deflection(),&limits()).is_err());
    assert_eq!(boundary,before);
}

#[test]
fn boss_wall_can_be_trimmed_by_an_inclined_plane() {
    let p = Profile::new(vec![[1.,2.,0.],[3.,2.,0.],[3.,4.,0.],[1.,4.,0.]]).unwrap();
    let wall = creation::extrude(&p, [0.,0.,10.], false, deflection(), &limits()).unwrap();
    let before = wall.clone();
    let clipped = editing::trim_surface_by_plane(&wall, [0.,0.,5.], [-0.2,0.,1.], [0.,0.,10.], deflection(), &limits()).unwrap();
    let m = measured(&clipped);
    assert!(close(m.area_mm2, 43.2, 1e-6), "{m:?}");
    assert_eq!(m.faces, 4); assert_eq!(m.shells, 1);
    assert!(close(m.bounds_mm[5], 5.6, 1e-5)); assert_eq!(wall, before);
    let other = editing::trim_surface_by_plane(&wall, [0.,0.,5.], [-0.2,0.,1.], [0.,0.,0.], deflection(), &limits()).unwrap();
    assert!(close(measured(&other).area_mm2, 36.8, 1e-6));
    assert!(editing::trim_surface_by_plane(&wall, [0.,0.,5.], [0.,0.,1.], [f64::NAN,0.,0.], deflection(), &limits()).is_err());
    assert!(editing::trim_surface_by_plane(&wall, [0.,0.,5.], [0.,0.,0.], [0.,0.,10.], deflection(), &limits()).is_err());
    assert!(editing::trim_surface_by_plane(&wall, [0.,0.,5.], [0.,0.,1.], [0.,0.,5.], deflection(), &limits()).is_err());
}

#[test]
fn boss_joins_coplanar_two_face_boundary_and_rejects_bent_boundary() {
    let left = Profile::new(vec![[0.,0.,5.],[5.,0.,5.],[5.,10.,5.],[0.,10.,5.]]).unwrap();
    let right = Profile::new(vec![[5.,0.,5.],[10.,0.,5.],[10.,10.,5.],[5.,10.,5.]]).unwrap();
    let a = creation::planar(&left, deflection(), &limits()).unwrap();
    let b = creation::planar(&right, deflection(), &limits()).unwrap();
    let boundary = editing::sew(&a, &b, 1e-7, deflection(), &limits()).unwrap();
    let before = boundary.clone(); let original = measured(&boundary);
    assert_eq!(original.faces, 2); assert_eq!(original.shells, 1);
    let profile = Profile::new(vec![[1.,2.,0.],[3.,2.,0.],[3.,4.,0.],[1.,4.,0.]]).unwrap();
    let result = editing::boss_parallel_planar_face(&profile, &boundary, deflection(), &limits()).unwrap();
    let m = measured(&result);
    assert_eq!(m.faces, 6); assert_eq!(m.shells, 1); assert_eq!(m.naked_edges, original.naked_edges + 4);
    assert!(close(m.area_mm2, 136., 1e-6), "{m:?}"); assert_eq!(boundary, before);
    let crossing = Profile::new(vec![[4.,2.,0.],[6.,2.,0.],[6.,4.,0.],[4.,4.,0.]]).unwrap();
    let crossing_result = editing::boss_parallel_planar_face(&crossing, &boundary, deflection(), &limits()).unwrap();
    let cross_measures = measured(&crossing_result);
    assert!(close(cross_measures.area_mm2, 136., 1e-6)); assert_eq!(cross_measures.shells, 1);
    assert_eq!(cross_measures.naked_edges, original.naked_edges + 4);
    let upright = Profile::new(vec![[5.,0.,5.],[5.,10.,5.],[5.,10.,9.],[5.,0.,9.]]).unwrap();
    let bent = editing::sew(&a, &creation::planar(&upright, deflection(), &limits()).unwrap(), 1e-7, deflection(), &limits()).unwrap();
    let saved = bent.clone();
    assert!(editing::boss_parallel_planar_face(&profile, &bent, deflection(), &limits()).is_err());
    assert_eq!(bent, saved);
    let distant_profile = Profile::new(vec![[20.,0.,5.],[25.,0.,5.],[25.,10.,5.],[20.,10.,5.]]).unwrap();
    let distant = creation::planar(&distant_profile, deflection(), &limits()).unwrap();
    let disconnected = editing::sew(&boundary, &distant, 1e-7, deflection(), &limits()).unwrap();
    assert!(editing::boss_parallel_planar_face(&profile, &disconnected, deflection(), &limits()).is_err());
}

#[test]
fn boss_bezier_line_area_components_against_independent_oracle() {
    use om9_core::geometry::{Curve, NurbsCurve};
    let arch = Curve::Nurbs(NurbsCurve {
        degree: 2, poles: vec![[3.,5.,0.],[5.,9.,0.],[7.,5.,0.]], weights: vec![1.;3],
        knots: vec![0.,0.,0.,1.,1.,1.], domain: [0.,1.], periodic: false, reversed: false, seam: None,
    });
    let line = Curve::Line { start: [7.,5.,0.], end: [3.,5.,0.], domain: [0.,1.] };
    let curve = Curve::Polycurve { segments: vec![arch, line], reversed: false };
    let boundary_profile = Profile::new(vec![[0.,0.,5.],[10.,0.,5.],[10.,10.,5.],[0.,10.,5.]]).unwrap();
    let boundary = creation::planar(&boundary_profile, deflection(), &limits()).unwrap();
    let face = om9_kernel::extrusion::planar_face(&curve, deflection(), &limits()).unwrap();
    let wall = om9_kernel::extrusion::shell(&curve, [0.,0.,5.], deflection(), &limits()).unwrap();
    let wall_before = wall.clone();
    let result = editing::boss_parallel_planar_sections(&[editing::BossSection::Curve(curve)], &boundary, deflection(), &limits()).unwrap();
    let perimeter = 4. + 2. * 5.0_f64.sqrt() + 2.0_f64.asinh();
    let face_area = measured(&face).area_mm2;
    let wall_area = measured(&wall).area_mm2;
    let result_area = measured(&result).area_mm2;
    println!("face={face_area:.12} expected={:.12}; wall={wall_area:.12} expected={:.12}; result={result_area:.12} expected={:.12}", 16./3., 5.*perimeter, 100.-16./3.+5.*perimeter);
    assert!(close(face_area, 16./3., 1e-7));
    assert!(close(wall_area, 5.*perimeter, 1e-7));
    assert!(close(result_area, 100.-16./3.+5.*perimeter, 1e-7));
    assert_eq!(wall, wall_before);
}

#[test]
fn boss_mixed_arc_line_profile_is_exact_and_connected() {
    let boundary_profile = Profile::new(vec![[0.0,0.0,5.0],[10.0,0.0,5.0],[10.0,10.0,5.0],[0.0,10.0,5.0]]).unwrap();
    let boundary = creation::planar(&boundary_profile, deflection(), &limits()).unwrap();
    let arc = om9_core::curve_build::arc_by_center([5.0,5.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0],2.0,0.0,PI).unwrap();
    let line = om9_core::geometry::Curve::Line { start: [3.0,5.0,0.0], end: [7.0,5.0,0.0], domain: [0.0,1.0] };
    let curve = om9_core::geometry::Curve::Polycurve { segments: vec![arc,line], reversed: false };
    let result = editing::boss_parallel_planar_sections(&[editing::BossSection::Curve(curve)], &boundary, deflection(), &limits()).unwrap();
    let m = measured(&result);
    assert_eq!(m.faces, 3); assert_eq!(m.shells, 1); assert_eq!(m.naked_edges, 6);
    assert!(close(m.area_mm2, 120.0+8.0*PI, 1e-5), "{m:?}");
}

#[test]
fn rational_ellipse_extrusion_area_matches_independent_perimeter() {
    for (rx, ry, height) in [(2.0_f64,3.0_f64,5.0_f64), (1.0,4.0,2.0), (3.0,3.0,7.0)] {
        let ellipse = om9_core::geometry::Curve::Nurbs(om9_core::curve_build::ellipse([0.0,0.0,0.0], [1.0,0.0,0.0], [0.0,1.0,0.0], rx, ry, &limits()).unwrap());
        let shell = om9_kernel::extrusion::shell(&ellipse, [0.0,0.0,height], deflection(), &limits()).unwrap();
        let before = shell.clone(); let m = measured(&shell);
        let count = 8192;
        let step = 2.0*PI / count as f64;
        let perimeter: f64 = (0..count).map(|i| { let t = (i as f64+0.5)*step; ((rx*t.sin()).powi(2)+(ry*t.cos()).powi(2)).sqrt()*step }).sum();
        println!("ellipse {rx} {ry} {height}: measured {:.12}, reference {:.12}, difference {:.12}", m.area_mm2, perimeter*height, m.area_mm2-perimeter*height);
        assert!(close(m.area_mm2, perimeter*height, 1e-7), "{rx} {ry} {height}: {m:?} expected {}", perimeter*height);
        assert_eq!(shell, before);
    }
}

#[test]
fn boss_rational_ellipse_uses_exact_closed_curve() {
    let boundary_profile = Profile::new(vec![[0.0,0.0,5.0],[12.0,0.0,5.0],[12.0,12.0,5.0],[0.0,12.0,5.0]]).unwrap();
    let boundary = creation::planar(&boundary_profile, deflection(), &limits()).unwrap();
    let ellipse = om9_core::geometry::Curve::Nurbs(om9_core::curve_build::ellipse([6.0,6.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0],2.0,3.0,&limits()).unwrap());
    let face = om9_kernel::extrusion::planar_face(&ellipse, deflection(), &limits()).unwrap();
    assert!(close(measured(&face).area_mm2, 6.0*PI, 1e-6));
    let wall = om9_kernel::extrusion::shell(&ellipse, [0.0,0.0,5.0], deflection(), &limits()).unwrap();
    for p in &wall.display.points {
        let on_ellipse = ((p[0]-6.0)/2.0).powi(2) + ((p[1]-6.0)/3.0).powi(2);
        assert!((on_ellipse-1.0).abs() < 1e-7, "{p:?}");
    }
    let expected = 144.0 - 6.0*PI + measured(&wall).area_mm2;
    let result = editing::boss_parallel_planar_sections(&[editing::BossSection::Curve(ellipse)], &boundary, deflection(), &limits()).unwrap();
    let m = measured(&result);
    assert_eq!(m.faces, 2); assert_eq!(m.shells, 1); assert_eq!(m.naked_edges, 5);
    assert!(close(m.area_mm2, expected, 1e-5), "{m:?}");
}

#[test]
fn boss_exact_circle_and_polygon_join_planar_boundary() {
    let boundary_profile = Profile::new(vec![[0.0,0.0,5.0],[10.0,0.0,5.0],[10.0,10.0,5.0],[0.0,10.0,5.0]]).unwrap();
    let boundary = creation::planar(&boundary_profile, deflection(), &limits()).unwrap();
    let circle = om9_core::curve_build::circle_on_frame([2.0,2.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0],1.0).unwrap();
    let polygon = Profile::new(vec![[6.0,6.0,0.0],[8.0,6.0,0.0],[8.0,8.0,0.0],[6.0,8.0,0.0]]).unwrap();
    let sections = [editing::BossSection::Circle(circle), editing::BossSection::Polygon(polygon)];
    let result = editing::boss_parallel_planar_sections(&sections, &boundary, deflection(), &limits()).unwrap();
    let m = measured(&result);
    assert_eq!(m.faces, 6); assert_eq!(m.shells, 1); assert_eq!(m.naked_edges, 9);
    assert!(close(m.area_mm2, 136.0+9.0*PI, 1e-6), "{m:?}");
}

#[test]
fn exact_circle_extrusion_shell_has_one_uncapped_cylindrical_wall() {
    let circle = om9_core::curve_build::circle_on_frame([0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0],2.0).unwrap();
    let shell = om9_kernel::extrusion::shell(&circle, [0.0,0.0,5.0], deflection(), &limits()).unwrap();
    let m = measured(&shell);
    assert_eq!(m.faces, 1); assert_eq!(m.solids, 0); assert_eq!(m.naked_edges, 2);
    assert!(close(m.area_mm2, 20.0*PI, 1e-6), "{m:?}");
}

#[test]
fn boss_multiple_profiles_join_atomically_and_reject_overlap() {
    let boundary_profile = Profile::new(vec![[0.0,0.0,5.0],[10.0,0.0,5.0],[10.0,10.0,5.0],[0.0,10.0,5.0]]).unwrap();
    let boundary = creation::planar(&boundary_profile, deflection(), &limits()).unwrap();
    let before = boundary.clone();
    let first = Profile::new(vec![[1.0,1.0,0.0],[3.0,1.0,0.0],[3.0,3.0,0.0],[1.0,3.0,0.0]]).unwrap();
    let second = Profile::new(vec![[6.0,6.0,0.0],[8.0,6.0,0.0],[8.0,8.0,0.0],[6.0,8.0,0.0]]).unwrap();
    let result = editing::boss_parallel_planar_face_many(&[first.clone(),second.clone()], &boundary, deflection(), &limits()).unwrap();
    let m = measured(&result);
    assert_eq!(m.faces, 9); assert_eq!(m.shells, 1); assert_eq!(m.naked_edges, 12);
    assert!(close(m.area_mm2, 172.0, 1e-6), "{m:?}");
    let overlap = Profile::new(vec![[2.0,2.0,0.0],[4.0,2.0,0.0],[4.0,4.0,0.0],[2.0,4.0,0.0]]).unwrap();
    assert!(editing::boss_parallel_planar_face_many(&[first, overlap], &boundary, deflection(), &limits()).is_err());
    assert!(editing::boss_parallel_planar_face_many(&[], &boundary, deflection(), &limits()).is_err());
    assert_eq!(boundary, before);
}

#[test]
fn boss_parallel_planar_face_preserves_existing_boundary_hole() {
    let outer = Profile::new(vec![[0.0,0.0,5.0], [10.0,0.0,5.0], [10.0,10.0,5.0], [0.0,10.0,5.0]]).unwrap();
    let surface = creation::planar(&outer, deflection(), &limits()).unwrap();
    let hole_tool = box_at([1.0,1.0,4.0], 1.0, 1.0, 2.0);
    let boundary = editing::trim_surface_by_solid(&surface, &hole_tool, deflection(), &limits()).unwrap().body().unwrap().clone();
    let before = boundary.clone();
    let profile = Profile::new(vec![[4.0,4.0,0.0], [6.0,4.0,0.0], [6.0,6.0,0.0], [4.0,6.0,0.0]]).unwrap();
    let result = editing::boss_parallel_planar_face(&profile, &boundary, deflection(), &limits()).unwrap();
    let m = measured(&result);
    assert_eq!(m.faces, 5); assert_eq!(m.shells, 1); assert_eq!(m.naked_edges, 12); assert_eq!(m.solids, 0);
    assert!(close(m.area_mm2, 135.0, 1e-6), "{m:?}");
    assert_eq!(boundary, before);
    let over_hole = Profile::new(vec![[0.5,0.5,0.0], [2.5,0.5,0.0], [2.5,2.5,0.0], [0.5,2.5,0.0]]).unwrap();
    assert!(editing::boss_parallel_planar_face(&over_hole, &boundary, deflection(), &limits()).is_err());
    assert!(editing::boss_parallel_planar_face(&profile, &hole_tool, deflection(), &limits()).is_err());
    let cylinder = creation::primitive(Primitive::Cylinder { radius_mm: 4.0, height_mm: 10.0 }, deflection(), &limits()).unwrap();
    let curved = (0..measured(&cylinder).faces).map(|i| editing::extract_face(&cylinder, i, deflection(), &limits()).unwrap())
        .find(|f| close(measured(f).area_mm2, 80.0*PI, 1e-6)).unwrap();
    assert!(editing::boss_parallel_planar_face(&profile, &curved, deflection(), &limits()).is_err());
}

#[test]
fn closed_profile_uncapped_extrusion_keeps_every_side() {
    let profile = Profile::new(vec![[0.0,0.0,0.0], [2.0,0.0,0.0], [2.0,2.0,0.0], [0.0,2.0,0.0]]).unwrap();
    let shell = creation::extrude(&profile, [0.0, 0.0, 5.0], false, deflection(), &limits()).unwrap();
    let m = measured(&shell);
    assert_eq!(m.faces, 4); assert_eq!(m.solids, 0); assert_eq!(m.naked_edges, 8);
    assert!(close(m.area_mm2, 40.0, 1e-7), "{m:?}");
    let open = creation::extrude_wire(&[[0.0,0.0,0.0], [2.0,0.0,0.0], [2.0,2.0,0.0]], [0.0,0.0,5.0], deflection(), &limits()).unwrap();
    let m = measured(&open);
    assert_eq!(m.faces, 2); assert!(close(m.area_mm2, 20.0, 1e-7), "{m:?}");
}

#[test]
fn boss_parallel_profiles_join_walls_to_finite_boundary() {
    for z in [-5.0, 5.0] {
        let profile = Profile::new(vec![[4.0,4.0,0.0], [6.0,4.0,0.0], [6.0,6.0,0.0], [4.0,6.0,0.0]]).unwrap();
        let boundary = Profile::new(vec![[0.0,0.0,z], [10.0,0.0,z], [10.0,10.0,z], [0.0,10.0,z]]).unwrap();
        let result = editing::boss_parallel_profiles(&profile, &boundary, deflection(), &limits()).unwrap();
        let m = measured(&result);
        assert_eq!(m.faces, 5); assert_eq!(m.shells, 1); assert_eq!(m.solids, 0); assert_eq!(m.naked_edges, 8);
        assert!(close(m.area_mm2, 136.0, 1e-6), "{m:?}");
        assert!(close(m.bounds_mm[2], z.min(0.0), 1e-5) && close(m.bounds_mm[5], z.max(0.0), 1e-5), "{m:?}");
    }
}

#[test]
fn boss_parallel_profiles_rejects_invalid_boundary_without_mutation() {
    let profile = Profile::new(vec![[4.0,4.0,0.0], [6.0,4.0,0.0], [6.0,6.0,0.0], [4.0,6.0,0.0]]).unwrap();
    let original = profile.clone();
    let outside = Profile::new(vec![[20.0,20.0,5.0], [30.0,20.0,5.0], [30.0,30.0,5.0], [20.0,30.0,5.0]]).unwrap();
    let clipping = Profile::new(vec![[5.0,0.0,5.0], [10.0,0.0,5.0], [10.0,10.0,5.0], [5.0,10.0,5.0]]).unwrap();
    let coplanar = Profile::new(vec![[0.0,0.0,0.0], [10.0,0.0,0.0], [10.0,10.0,0.0], [0.0,10.0,0.0]]).unwrap();
    for boundary in [&outside, &clipping, &coplanar] {
        assert!(editing::boss_parallel_profiles(&profile, boundary, deflection(), &limits()).is_err());
    }
    assert_eq!(profile, original);
}

#[test]
fn surface_trim_by_solid_preserves_boundary_geometry_and_reports_empty() {
    let surface = rect_face(10.0, 10.0);
    let before = surface.clone();
    let cutter = box_at([4.0, 4.0, -1.0], 2.0, 2.0, 2.0);
    let trimmed = editing::trim_surface_by_solid(&surface, &cutter, deflection(), &limits()).unwrap();
    let result = measured(trimmed.body().unwrap());
    assert_eq!(result.solids, 0); assert_eq!(result.faces, 1);
    assert!(close(result.area_mm2, 96.0, 1e-7), "{result:?}");
    assert_eq!(result.naked_edges, 8);
    let disjoint = box_at([20.0, 20.0, -1.0], 2.0, 2.0, 2.0);
    let unchanged = editing::trim_surface_by_solid(&surface, &disjoint, deflection(), &limits()).unwrap();
    assert!(close(measured(unchanged.body().unwrap()).area_mm2, 100.0, 1e-7));
    let covering = box_at([-1.0, -1.0, -1.0], 12.0, 12.0, 2.0);
    assert!(matches!(editing::trim_surface_by_solid(&surface, &covering, deflection(), &limits()).unwrap(), BooleanOutcome::Empty));
    assert!(editing::trim_surface_by_solid(&cutter, &surface, deflection(), &limits()).is_err());
    assert!(editing::trim_surface_by_solid(&surface, &surface, deflection(), &limits()).is_err());
    assert_eq!(surface, before);
}

#[test]
fn surface_trim_by_solid_handles_curved_boundary() {
    let cylinder = creation::primitive(Primitive::Cylinder { radius_mm: 4.0, height_mm: 10.0 }, deflection(), &limits()).unwrap();
    let face = (0..measured(&cylinder).faces).map(|i| editing::extract_face(&cylinder, i, deflection(), &limits()).unwrap())
        .find(|b| close(measured(b).area_mm2, 80.0 * PI, 1e-6)).expect("cylinder side");
    let before = face.clone();
    let cutter = box_at([3.0, -1.0, 2.0], 2.0, 2.0, 2.0);
    let trimmed = editing::trim_surface_by_solid(&face, &cutter, deflection(), &limits()).unwrap();
    let after = measured(trimmed.body().unwrap());
    assert_eq!(after.solids, 0);
    assert!(after.area_mm2 > 0.0 && after.area_mm2 < 80.0 * PI, "{after:?}");
    assert_eq!(face, before);
}

#[test]
fn solid_split_partitions_target_and_preserves_exact_volume() {
    let target = box_shape(10.0, 10.0, 10.0);
    let before = target.clone();
    for (cutter, expected) in [
        (
            box_at([5.0, 0.0, 0.0], 10.0, 10.0, 10.0),
            vec![500.0, 500.0],
        ),
        (box_at([2.0, 2.0, 2.0], 4.0, 4.0, 4.0), vec![64.0, 936.0]),
        (box_at([20.0, 0.0, 0.0], 10.0, 10.0, 10.0), vec![1000.0]),
        (target.clone(), vec![1000.0]),
    ] {
        let split = editing::split_solid(&target, &cutter, deflection(), &limits()).unwrap();
        let m = measured(&split.brep);
        assert_eq!(split.pieces, expected.len());
        assert_eq!(m.solids, expected.len());
        assert!(m.closed);
        assert!(close(m.volume_mm3, 1000.0, 1e-6), "{}", m.volume_mm3);
        let mut volumes: Vec<_> = (0..split.pieces)
            .map(|index| {
                let piece =
                    editing::split_solid_piece(&split, index, deflection(), &limits()).unwrap();
                let measure = measured(&piece);
                assert!(measure.closed);
                assert_eq!(measure.solids, 1);
                measure.volume_mm3
            })
            .collect();
        volumes.sort_by(f64::total_cmp);
        for (actual, expected) in volumes.into_iter().zip(expected) {
            assert!(close(actual, expected, 1e-6), "{actual} != {expected}");
        }
        assert!(editing::split_solid_piece(&split, split.pieces, deflection(), &limits()).is_err());
        assert_eq!(target, before);
    }
    assert!(
        editing::split_solid(&target, &rect_face(10.0, 10.0), deflection(), &limits()).is_err()
    );
    assert!(
        editing::split_solid(&rect_face(10.0, 10.0), &target, deflection(), &limits()).is_err()
    );
}

#[test]
fn solid_split_sets_partition_overlaps_and_deduplicate_shared_cells() {
    let a = box_shape(10.0, 10.0, 10.0);
    let b = box_at([5.0, 0.0, 0.0], 10.0, 10.0, 10.0);
    for (targets, cutters) in [
        (vec![a.clone(), b.clone()], vec![a.clone(), b.clone()]),
        (vec![b.clone(), a.clone()], vec![b.clone(), a.clone()]),
        (vec![a.clone(), b.clone(), a.clone()], vec![b.clone()]),
    ] {
        let pieces =
            editing::split_solid_sets(&targets, &cutters, deflection(), &limits()).unwrap();
        assert_eq!(pieces.len(), 3);
        let mut mins = Vec::new();
        for piece in pieces {
            let m = measured(&piece);
            assert!(m.closed);
            assert_eq!(m.solids, 1);
            assert!(close(m.volume_mm3, 500.0, 1e-6));
            mins.push(m.bounds_mm[0]);
        }
        mins.sort_by(f64::total_cmp);
        for (actual, expected) in mins.into_iter().zip([0.0, 5.0, 10.0]) {
            assert!(close(actual, expected, 1e-6));
        }
    }
    let mut constrained = limits();
    constrained.max_outputs = 2;
    assert!(
        editing::split_solid_sets(
            &[a.clone(), b.clone()],
            &[b.clone()],
            deflection(),
            &constrained
        )
        .is_err()
    );
    let face = rect_face(10.0, 10.0);
    assert!(editing::split_solid_sets(&[face.clone()], &[face], deflection(), &limits()).is_err());
    assert!(editing::split_solid_sets(&[], &[b], deflection(), &limits()).is_err());
}

#[test]
fn solid_split_sets_applies_each_cutter_to_every_result() {
    let target = box_shape(10.0, 10.0, 10.0);
    let cutters = [
        box_at([2.0, 0.0, 0.0], 2.0, 10.0, 10.0),
        box_at([6.0, 0.0, 0.0], 2.0, 10.0, 10.0),
    ];
    for order in [
        vec![cutters[0].clone(), cutters[1].clone()],
        vec![cutters[1].clone(), cutters[0].clone()],
    ] {
        let pieces = editing::split_solid_sets(
            std::slice::from_ref(&target),
            &order,
            deflection(),
            &limits(),
        )
        .unwrap();
        assert_eq!(pieces.len(), 5);
        let mut starts: Vec<_> = pieces
            .iter()
            .map(|piece| {
                let m = measured(piece);
                assert_eq!(m.solids, 1);
                assert!(m.closed);
                assert!(close(m.volume_mm3, 200.0, 1e-6));
                m.bounds_mm[0]
            })
            .collect();
        starts.sort_by(f64::total_cmp);
        for (actual, expected) in starts.into_iter().zip([0.0, 2.0, 4.0, 6.0, 8.0]) {
            assert!(close(actual, expected, 1e-6));
        }
    }
}

#[test]
fn create_solid_regions_from_intersecting_faces_and_reject_open_set() {
    let boxes = [
        box_shape(10.0, 10.0, 10.0),
        box_at([5.0, 0.0, 0.0], 10.0, 10.0, 10.0),
    ];
    let mut faces = Vec::new();
    for body in &boxes {
        for index in 0..measured(body).faces {
            faces.push(editing::extract_face(body, index, deflection(), &limits()).unwrap());
        }
    }
    let before = faces.clone();
    let regions = editing::create_solid_regions(&faces, deflection(), &limits())
        .unwrap()
        .unwrap();
    assert_eq!(regions.pieces, 3);
    let mut starts = Vec::new();
    for index in 0..regions.pieces {
        let piece = editing::split_solid_piece(&regions, index, deflection(), &limits()).unwrap();
        let m = measured(&piece);
        assert!(m.closed && m.solids == 1 && close(m.volume_mm3, 500.0, 1e-6));
        starts.push(m.bounds_mm[0]);
    }
    starts.sort_by(f64::total_cmp);
    for (actual, expected) in starts.into_iter().zip([0.0, 5.0, 10.0]) {
        assert!(close(actual, expected, 1e-6));
    }
    assert_eq!(faces, before);
    let mut limited = limits();
    limited.max_outputs = 2;
    assert!(editing::create_solid_regions(&faces, deflection(), &limited).is_err());
    assert!(
        editing::create_solid_regions(&[rect_face(10.0, 10.0)], deflection(), &limits())
            .unwrap()
            .is_none()
    );
    assert!(editing::create_solid_regions(&[], deflection(), &limits()).is_err());
}

fn measured(brep: &Brep) -> creation::Measures {
    creation::measure(brep, &limits()).expect("measure")
}

fn vertical_edge(brep: &Brep) -> usize {
    for index in 0..brep.display.edges.len() {
        if let Ok((a, b)) = editing::edge_endpoints(brep, index) {
            let dx = (a[0] - b[0]).abs();
            let dy = (a[1] - b[1]).abs();
            let dz = (a[2] - b[2]).abs();
            if dx < 1.0e-6 && dy < 1.0e-6 && dz > 1.0 {
                return index;
            }
        }
    }
    panic!("no vertical edge in the box");
}

fn face_area(brep: &Brep, index: usize) -> f64 {
    let face = editing::extract_face(brep, index, deflection(), &limits()).expect("extract face");
    measured(&face).area_mm2
}

fn face_min_z(brep: &Brep, index: usize) -> f64 {
    let mut min_z = f64::INFINITY;
    for [a, b, c, face] in &brep.display.triangles {
        if *face as usize != index {
            continue;
        }
        for point in [a, b, c] {
            min_z = min_z.min(brep.display.points[*point as usize][2]);
        }
    }
    min_z
}

fn face_max_z(brep: &Brep, index: usize) -> f64 {
    let mut max_z = f64::NEG_INFINITY;
    for [a, b, c, face] in &brep.display.triangles {
        if *face as usize != index {
            continue;
        }
        for point in [a, b, c] {
            max_z = max_z.max(brep.display.points[*point as usize][2]);
        }
    }
    max_z
}

fn top_face_index(brep: &Brep) -> usize {
    let mut global_max = f64::NEG_INFINITY;
    for index in 0..brep.display.faces.len() {
        global_max = global_max.max(face_max_z(brep, index));
    }
    // The top face is the horizontal face whose every vertex is at the global maximum z; a side
    // face also reaches that z but its minimum z is lower.
    for index in 0..brep.display.faces.len() {
        if close(face_max_z(brep, index), global_max, 1e-9)
            && close(face_min_z(brep, index), global_max, 1e-9)
        {
            return index;
        }
    }
    panic!("no horizontal top face");
}

/* ------------------------- 001 union ------------------------- */

#[test]
fn m03_01_10_001() {
    let limits = limits();
    let a = box_shape(10.0, 10.0, 10.0);
    let b = box_at([5.0, 0.0, 0.0], 10.0, 10.0, 10.0);
    let union = editing::boolean(&a, &b, BooleanOp::Union, deflection(), &limits).expect("union");
    let body = union.body().expect("union body");
    let union_measures = measured(body);
    assert!(
        close(union_measures.volume_mm3, 1500.0, 1e-6),
        "union volume {}",
        union_measures.volume_mm3
    );
    assert_eq!(union_measures.solids, 1);
    assert!(union_measures.closed);
    assert_eq!(union_measures.naked_edges, 0);

    // Disjoint union is a compound of two solids; its volume is the sum and no phantom body exists.
    let far = box_at([50.0, 0.0, 0.0], 10.0, 10.0, 10.0);
    let disjoint = editing::boolean(&a, &far, BooleanOp::Union, deflection(), &limits)
        .expect("disjoint union")
        .body()
        .cloned()
        .expect("disjoint body");
    let disjoint_measures = measured(&disjoint);
    assert!(close(disjoint_measures.volume_mm3, 2000.0, 1e-6));
    assert_eq!(disjoint_measures.solids, 2);

    // Negative: an open shell is not a boolean operand; a tampered source is refused.
    let open = creation::extrude_wire(
        &[
            [0.0, 0.0, 0.0],
            [10.0, 0.0, 0.0],
            [10.0, 10.0, 0.0],
            [0.0, 10.0, 0.0],
        ],
        [0.0, 0.0, 10.0],
        deflection(),
        &limits,
    )
    .expect("open shell");
    assert!(matches!(
        editing::boolean(&open, &a, BooleanOp::Union, deflection(), &limits),
        Err(EditingError::Invalid(_))
    ));
    let mut json = serde_json::to_value(&a).expect("serialize");
    json["source"] = serde_json::Value::String("not a brep".into());
    let tampered: Brep = serde_json::from_value(json).expect("deserialize");
    assert!(matches!(
        editing::boolean(&tampered, &a, BooleanOp::Union, deflection(), &limits),
        Err(EditingError::Invalid(_))
    ));
}

/* ------------------------- 002 difference ------------------------- */

#[test]
fn m03_01_10_002() {
    let limits = limits();
    let a = box_shape(10.0, 10.0, 10.0);
    let b = box_at([5.0, 0.0, 0.0], 10.0, 10.0, 10.0);
    let cut = editing::boolean(&a, &b, BooleanOp::Difference, deflection(), &limits)
        .expect("difference")
        .body()
        .cloned()
        .expect("difference body");
    let cut_measures = measured(&cut);
    assert!(
        close(cut_measures.volume_mm3, 500.0, 1e-6),
        "difference volume {}",
        cut_measures.volume_mm3
    );
    assert_eq!(cut_measures.solids, 1);
    assert!(cut_measures.closed);
    // The cut face at x = 5 is part of the result: the box's extent must stop there.
    assert!(close(cut_measures.bounds_mm[3], 5.0, 1e-6));

    // A tool outside the argument leaves the argument unchanged (explicit, not a phantom body).
    let outside = box_at([50.0, 0.0, 0.0], 10.0, 10.0, 10.0);
    let unchanged = editing::boolean(&a, &outside, BooleanOp::Difference, deflection(), &limits)
        .expect("outside difference")
        .body()
        .cloned()
        .expect("unchanged body");
    assert!(close(measured(&unchanged).volume_mm3, 1000.0, 1e-6));

    let mut json = serde_json::to_value(&a).expect("serialize");
    json["source"] = serde_json::Value::String("not a brep".into());
    let tampered: Brep = serde_json::from_value(json).expect("deserialize");
    assert!(matches!(
        editing::boolean(&a, &tampered, BooleanOp::Difference, deflection(), &limits),
        Err(EditingError::Invalid(_))
    ));
}

/* ------------------------- 003 intersection ------------------------- */

#[test]
fn m03_01_10_003() {
    let limits = limits();
    let a = box_shape(10.0, 10.0, 10.0);
    let b = box_at([5.0, 0.0, 0.0], 10.0, 10.0, 10.0);
    let common = editing::boolean(&a, &b, BooleanOp::Intersection, deflection(), &limits)
        .expect("intersection")
        .body()
        .cloned()
        .expect("intersection body");
    let common_measures = measured(&common);
    assert!(
        close(common_measures.volume_mm3, 500.0, 1e-6),
        "intersection volume {}",
        common_measures.volume_mm3
    );
    assert_eq!(common_measures.solids, 1);

    // Disjoint solids have an empty intersection: a typed empty outcome, never a phantom body.
    let far = box_at([50.0, 0.0, 0.0], 10.0, 10.0, 10.0);
    assert_eq!(
        editing::boolean(&a, &far, BooleanOp::Intersection, deflection(), &limits)
            .expect("empty intersection"),
        BooleanOutcome::Empty
    );

    // Borderline coplanar contact must not silently produce a wrong solid body.
    let touching = box_at([10.0, 0.0, 0.0], 10.0, 10.0, 10.0);
    let contact = editing::boolean(
        &a,
        &touching,
        BooleanOp::Intersection,
        deflection(),
        &limits,
    )
    .expect("contact intersection");
    match contact {
        BooleanOutcome::Empty => {}
        BooleanOutcome::Body(body) => {
            let measures = measured(&body);
            assert!(
                measures.volume_mm3 <= 1e-9,
                "coplanar contact produced volume {}",
                measures.volume_mm3
            );
            assert_eq!(measures.solids, 0);
        }
    }
}

/* ------------------------- 004 split ------------------------- */

#[test]
fn m03_01_10_004() {
    let limits = limits();
    let face = rect_face(10.0, 20.0);
    let original = measured(&face);
    let split = editing::split_face(
        &face,
        &[[5.0, -5.0, 0.0], [5.0, 25.0, 0.0]],
        deflection(),
        &limits,
    )
    .expect("split face");
    assert_eq!(split.pieces, 2);
    let split_measures = measured(&split.brep);
    assert_eq!(split_measures.faces, 2);
    assert!(
        close(split_measures.area_mm2, original.area_mm2, 1e-6),
        "split area {} vs {}",
        split_measures.area_mm2,
        original.area_mm2
    );
    // Each piece is exactly half the original face.
    for index in 0..split_measures.faces {
        assert!(close(face_area(&split.brep, index), 100.0, 1e-6));
    }

    // Negative: a trim that misses the face, a repeated point, and a non-face input.
    assert!(matches!(
        editing::split_face(
            &face,
            &[[50.0, -5.0, 0.0], [50.0, 25.0, 0.0]],
            deflection(),
            &limits
        ),
        Err(EditingError::Invalid(_))
    ));
    assert!(matches!(
        editing::split_face(
            &face,
            &[[1.0, 1.0, 0.0], [1.0, 1.0, 0.0]],
            deflection(),
            &limits
        ),
        Err(EditingError::Invalid(_))
    ));
    assert!(matches!(
        editing::split_face(
            &box_shape(10.0, 10.0, 10.0),
            &[[5.0, -5.0, 0.0], [5.0, 25.0, 0.0]],
            deflection(),
            &limits
        ),
        Err(EditingError::Invalid(_))
    ));
}

/* ------------------------- 005 cap ------------------------- */

#[test]
fn remove_multiple_faces_uses_original_indices() {
    let limits = limits();
    let body = box_shape(10.0, 10.0, 10.0);
    let remaining = editing::remove_faces(&body, &[1, 4], deflection(), &limits).unwrap();
    let m = measured(&remaining);
    assert_eq!(m.faces, 4);
    assert!(!m.closed);
    assert!(editing::remove_faces(&body, &[1, 1], deflection(), &limits).is_err());
    assert!(editing::remove_faces(&body, &[1, 99], deflection(), &limits).is_err());
    assert!(editing::remove_faces(&body, &[0, 1, 2, 3, 4, 5], deflection(), &limits).is_err());
}

#[test]
fn cap_planar_multiple_loops_and_circular_boundary() {
    let limits = limits();
    let boxed = box_shape(10.0, 10.0, 10.0);
    let open_top =
        editing::remove_face(&boxed, top_face_index(&boxed), deflection(), &limits).unwrap();
    let bottom = (0..open_top.display.faces.len())
        .find(|&i| {
            close(face_min_z(&open_top, i), 0.0, 1e-9) && close(face_max_z(&open_top, i), 0.0, 1e-9)
        })
        .expect("bottom face");
    let open_both = editing::remove_face(&open_top, bottom, deflection(), &limits).unwrap();
    assert_eq!(measured(&open_both).naked_edges, 8);
    let capped =
        editing::cap_planar(&open_both, 1e-6, deflection(), &limits).expect("two planar loops");
    let measure = measured(&capped);
    assert!(measure.closed);
    assert_eq!(measure.solids, 1);
    assert_eq!(measure.naked_edges, 0);
    assert!(close(measure.volume_mm3, 1000.0, 1e-6));

    let cylinder = creation::primitive(
        Primitive::Cylinder {
            radius_mm: 5.0,
            height_mm: 10.0,
        },
        deflection(),
        &limits,
    )
    .unwrap();
    let open =
        editing::remove_face(&cylinder, top_face_index(&cylinder), deflection(), &limits).unwrap();
    let capped =
        editing::cap_planar(&open, 1e-6, deflection(), &limits).expect("circular planar loop");
    let measure = measured(&capped);
    assert!(measure.closed);
    assert_eq!(measure.naked_edges, 0);
    assert!(close(measure.volume_mm3, 250.0 * PI, 1e-5));
}

#[test]
fn cap_planar_closes_nested_coplanar_loops() {
    let limits = limits();
    let cylinder = |radius_mm| {
        creation::primitive(
            Primitive::Cylinder {
                radius_mm,
                height_mm: 10.0,
            },
            deflection(),
            &limits,
        )
        .unwrap()
    };
    let ring = editing::boolean(
        &cylinder(5.0),
        &cylinder(2.0),
        BooleanOp::Difference,
        deflection(),
        &limits,
    )
    .unwrap()
    .body()
    .unwrap()
    .clone();
    let open = editing::remove_face(&ring, top_face_index(&ring), deflection(), &limits).unwrap();
    let before = open.clone();
    let capped =
        editing::cap_planar(&open, 1e-6, deflection(), &limits).expect("annular planar cap");
    let measure = measured(&capped);
    assert!(measure.closed);
    assert_eq!(measure.solids, 1);
    assert_eq!(measure.naked_edges, 0);
    assert!(close(measure.volume_mm3, 210.0 * PI, 1e-5));
    assert_eq!(open, before);
}

#[test]
fn cap_planar_two_holes_in_one_opening() {
    let limits = limits();
    let mut body = box_shape(10.0, 10.0, 10.0);
    for x in [2.0, 6.0] {
        body = editing::boolean(
            &body,
            &box_at([x, 4.0, -1.0], 2.0, 2.0, 12.0),
            BooleanOp::Difference,
            deflection(),
            &limits,
        )
        .unwrap()
        .body()
        .unwrap()
        .clone();
    }
    let open = editing::remove_face(&body, top_face_index(&body), deflection(), &limits).unwrap();
    let capped = editing::cap_planar(&open, 1e-6, deflection(), &limits).expect("two-hole cap");
    let measure = measured(&capped);
    assert!(measure.closed);
    assert_eq!(measure.solids, 1);
    assert_eq!(measure.naked_edges, 0);
    assert!(
        close(measure.volume_mm3, 920.0, 1e-6),
        "{}",
        measure.volume_mm3
    );
}

#[test]
fn cap_planar_closes_planar_loop_on_partially_nonplanar_shell() {
    let limits = limits();
    let tall = box_at([0.0, 0.0, 0.0], 10.0, 10.0, 10.0);
    let short = box_at([10.0, 0.0, 0.0], 10.0, 10.0, 5.0);
    let body = editing::boolean(&tall, &short, BooleanOp::Union, deflection(), &limits)
        .unwrap()
        .body()
        .unwrap()
        .clone();
    let mut open = body;
    for z in [10.0, 5.0] {
        loop {
            let face = (0..open.display.faces.len()).find(|&i| {
                close(face_min_z(&open, i), z, 1e-9) && close(face_max_z(&open, i), z, 1e-9)
            });
            let Some(face) = face else { break };
            open = editing::remove_face(&open, face, deflection(), &limits).unwrap();
        }
    }
    let step = (0..open.display.faces.len())
        .find(|&i| {
            let vertices: Vec<_> = open
                .display
                .triangles
                .iter()
                .filter(|triangle| triangle[3] as usize == i)
                .flat_map(|triangle| {
                    triangle[..3]
                        .iter()
                        .map(|index| open.display.points[*index as usize])
                })
                .collect();
            !vertices.is_empty()
                && vertices.iter().all(|point| close(point[0], 10.0, 1e-9))
                && close(face_min_z(&open, i), 5.0, 1e-9)
                && close(face_max_z(&open, i), 10.0, 1e-9)
        })
        .expect("vertical step");
    open = editing::remove_face(&open, step, deflection(), &limits).unwrap();
    loop {
        let face = (0..open.display.faces.len()).find(|&i| {
            close(face_min_z(&open, i), 0.0, 1e-9) && close(face_max_z(&open, i), 0.0, 1e-9)
        });
        let Some(face) = face else { break };
        open = editing::remove_face(&open, face, deflection(), &limits).unwrap();
    }
    let before = measured(&open);
    let capped = editing::cap_planar(&open, 1e-6, deflection(), &limits)
        .expect("cap planar bottom while leaving nonplanar upper boundary");
    let after = measured(&capped);
    assert!(!after.closed);
    assert!(after.naked_edges > 0 && after.naked_edges < before.naked_edges);
    assert!(after.faces > before.faces);
}

#[test]
fn m03_01_10_005() {
    let limits = limits();
    let boxed = box_shape(10.0, 10.0, 10.0);
    let top = top_face_index(&boxed);
    let open = editing::remove_face(&boxed, top, deflection(), &limits).expect("open shell");
    let open_measures = measured(&open);
    assert_eq!(open_measures.naked_edges, 4);
    assert!(!open_measures.closed);

    let capped = editing::cap_planar(&open, 1.0e-6, deflection(), &limits).expect("cap");
    let capped_measures = measured(&capped);
    assert!(capped_measures.closed);
    assert_eq!(capped_measures.solids, 1);
    assert_eq!(capped_measures.naked_edges, 0);
    assert!(
        close(capped_measures.volume_mm3, 1000.0, 1e-6),
        "capped volume {}",
        capped_measures.volume_mm3
    );

    // Negative: capping an already closed body and capping a non-planar naked loop.
    assert!(matches!(
        editing::cap_planar(&boxed, 1.0e-6, deflection(), &limits),
        Err(EditingError::Invalid(_))
    ));
    let skew = creation::extrude_wire(
        &[
            [0.0, 0.0, 0.0],
            [10.0, 0.0, 0.0],
            [10.0, 10.0, 3.0],
            [0.0, 10.0, 0.0],
        ],
        [0.0, 0.0, 10.0],
        deflection(),
        &limits,
    )
    .expect("skewed open shell");
    assert!(editing::cap_planar(&skew, 1.0e-6, deflection(), &limits).is_err());
}

/* ------------------------- 006 fillet and chamfer ------------------------- */

#[test]
fn fillet_tangent_split_edge_fixture() {
    let profile = Profile::new(vec![
        [0.0, 0.0, 0.0], [5.0, 0.0, 0.0], [10.0, 0.0, 0.0],
        [10.0, 10.0, 0.0], [0.0, 10.0, 0.0],
    ]).unwrap();
    let body = creation::extrude(&profile, [0.0, 0.0, 10.0], true, deflection(), &limits()).unwrap();
    let mut split = Vec::new();
    for index in 0..body.display.edges.len() {
        if let Ok((a, b)) = editing::oriented_edge_endpoints(&body, index, &limits()) {
            if close(a[1], 0.0, 1e-6) && close(b[1], 0.0, 1e-6)
                && close(a[2], 10.0, 1e-6) && close(b[2], 10.0, 1e-6)
                && close((a[0] - b[0]).abs(), 5.0, 1e-6) {
                split.push((index, a, b));
            }
        }
    }
    assert_eq!(split.len(), 2, "split top edge must survive extrusion");
    let side = if close(split[0].1[0], 5.0, 1e-6) { 0 } else { 1 };
    let mask = editing::fillet_shared_endpoint(&body, split[0].0, side, split[1].0, &limits()).unwrap();
    assert_eq!(mask, if close(split[1].1[0], 5.0, 1e-6) { 1 } else { 2 });
    let other = (0..body.display.edges.len()).find(|&i| {
        if i == split[0].0 || i == split[1].0 { return false; }
        let Ok((a, b)) = editing::oriented_edge_endpoints(&body, i, &limits()) else { return false; };
        let outer = if side == 0 { split[0].2 } else { split[0].1 };
        a == outer || b == outer
    }).unwrap();
    assert_eq!(editing::fillet_shared_endpoint(&body, split[0].0, 1 - side, other, &limits()).unwrap(), 0);
    let batch = editing::fillet_shared_endpoints(&body, split[0].0, side, &[split[0].0, split[1].0, other], &limits()).unwrap();
    assert_eq!(batch, vec![1 << side, mask, 0]);
    assert!(editing::fillet_shared_endpoints(&body, split[0].0, side, &[usize::MAX], &limits()).is_err());
    assert!(editing::fillet_shared_endpoints(&body, split[0].0, side, &[], &limits()).is_err());
    let constant = editing::fillet_with_end_radii(&body, &[(split[0].0, 1.0, 1.0), (split[1].0, 1.0, 1.0)], deflection(), &limits()).unwrap();
    assert!(measured(&constant).closed);
    let controlled = editing::fillet_with_radius_controls(&body,
        &[(split[0].0, 1.0, 1.0), (split[1].0, 1.0, 1.0)], &[(split[0].0, 0.5, 1.3)], deflection(), &limits()).unwrap();
    assert!(measured(&controlled).closed && measured(&controlled).volume_mm3 < measured(&constant).volume_mm3);
    assert!(editing::fillet_with_radius_controls(&body, &[(split[0].0, 1.0, 1.0)], &[(split[0].0, 0.5, 1.3)], deflection(), &limits()).is_err());
    let expected = 1000.0 - (1.0 - PI / 4.0) * 10.0;
    assert!(close(measured(&constant).volume_mm3, expected, 1e-5));
    let radii: Vec<_> = split.iter().map(|(index, a, b)| (*index, 1.0 + a[0] / 10.0, 1.0 + b[0] / 10.0)).collect();
    let reverse: Vec<_> = split.iter().map(|(index, a, b)| (*index, 2.0 - a[0] / 10.0, 2.0 - b[0] / 10.0)).collect();
    let variable = editing::fillet_with_end_radii(&body, &radii, deflection(), &limits()).unwrap();
    let reversed = editing::fillet_with_end_radii(&body, &reverse, deflection(), &limits()).unwrap();
    assert!(measured(&variable).closed && measured(&reversed).closed);
    let high = editing::fillet_with_end_radii(&body, &[(split[0].0, 2.0, 2.0), (split[1].0, 2.0, 2.0)], deflection(), &limits()).unwrap();
    assert!(measured(&variable).volume_mm3 < measured(&constant).volume_mm3);
    assert!(measured(&variable).volume_mm3 > measured(&high).volume_mm3);
    let slice = |shape: &Brep, x| {
        let tool = box_at([x, 0.0, 0.0], 1.0, 10.0, 10.0);
        let result = editing::boolean(shape, &tool, BooleanOp::Intersection, deflection(), &limits()).unwrap();
        measured(result.body().unwrap()).volume_mm3
    };
    assert!(slice(&variable, 0.0) > slice(&variable, 9.0));
    assert!(slice(&reversed, 0.0) < slice(&reversed, 9.0));
    assert!(editing::fillet_with_end_radii(&body, &radii[..1], deflection(), &limits()).is_err());
    let mut inconsistent = radii.clone();
    let shared_start = close(split[1].1[0], 5.0, 1e-6);
    if shared_start { inconsistent[1].1 += 0.25; } else { inconsistent[1].2 += 0.25; }
    assert!(editing::fillet_with_end_radii(&body, &inconsistent, deflection(), &limits()).is_err());
}

#[test]
fn fillet_independent_edges_use_distinct_radii() {
    let boxed = box_shape(10.0, 10.0, 10.0);
    let vertical: Vec<_> = (0..boxed.display.edges.len())
        .filter(|&index| {
            editing::edge_endpoints(&boxed, index).is_ok_and(|(a, b)| {
                (a[0] - b[0]).abs() < 1e-6
                    && (a[1] - b[1]).abs() < 1e-6
                    && (a[2] - b[2]).abs() > 9.0
            })
        })
        .collect();
    assert!(vertical.len() >= 2);
    let result = editing::fillet_with_radii(
        &boxed,
        &[(vertical[0], 1.0), (vertical[1], 2.0)],
        deflection(),
        &limits(),
    )
    .unwrap();
    let m = measured(&result);
    let expected = 1000.0 - (1.0 - PI / 4.0) * 10.0 * (1.0 + 4.0);
    assert!(
        m.closed && close(m.volume_mm3, expected, 1e-5),
        "{} != {expected}",
        m.volume_mm3
    );
    assert!(
        editing::fillet_with_radii(
            &boxed,
            &[(vertical[0], 1.0), (vertical[0], 2.0)],
            deflection(),
            &limits()
        )
        .is_err()
    );
    assert!(
        editing::fillet_with_radii(&boxed, &[(vertical[0], 0.0)], deflection(), &limits()).is_err()
    );
}

#[test]
fn fillet_open_joined_shell_internal_edge() {
    let boxed = box_shape(10.0, 10.0, 10.0);
    let open =
        editing::remove_face(&boxed, top_face_index(&boxed), deflection(), &limits()).unwrap();
    let before = measured(&open);
    assert!(!before.closed && before.faces == 5);
    let edge = vertical_edge(&open);
    let filleted = editing::fillet_with_radii(&open, &[(edge, 1.0)], deflection(), &limits())
        .expect("internal edge on open joined shell");
    let after = measured(&filleted);
    assert!(!after.closed);
    assert_eq!(after.solids, 0);
    assert!(after.faces > before.faces);
    assert!(after.naked_edges > 0);
    let expected_area = 500.0 - 20.0 + 5.0 * PI - (1.0 - PI / 4.0);
    assert!(
        close(after.area_mm2, expected_area, 1e-5),
        "{} != {expected_area}",
        after.area_mm2
    );
    let boundary = (0..open.display.edges.len())
        .find(|&index| {
            editing::edge_endpoints(&open, index)
                .is_ok_and(|(a, b)| close(a[2], 10.0, 1e-6) && close(b[2], 10.0, 1e-6))
        })
        .expect("open top boundary");
    assert!(
        editing::fillet_with_radii(&open, &[(boundary, 1.0)], deflection(), &limits()).is_err()
    );
}

#[test]
fn fillet_endpoint_query_distinguishes_closed_edges_from_errors() {
    let cylinder = creation::primitive(
        Primitive::Cylinder { radius_mm: 5.0, height_mm: 10.0 },
        deflection(),
        &limits(),
    ).unwrap();
    let mut closed = Vec::new();
    let mut open = 0;
    for index in 0..cylinder.display.edges.len() {
        match editing::oriented_edge_endpoints_or_closed(&cylinder, index, &limits()).unwrap() {
            Some((a, b)) => { assert_ne!(a, b); open += 1; }
            None => closed.push(index),
        }
    }
    assert!(open > 0 && closed.len() >= 2);
    assert!(editing::oriented_edge_endpoints_or_closed(&cylinder, usize::MAX, &limits()).is_err());
    assert!(editing::oriented_edge_endpoints(&cylinder, closed[0], &limits()).is_err());
    let result = editing::fillet_with_end_radii(&cylinder, &[(closed[0], 1.0, 1.0)], deflection(), &limits()).unwrap();
    assert!(measured(&result).closed);
}

#[test]
fn fillet_interior_radius_control_changes_local_shape() {
    let body = box_shape(10.0, 10.0, 10.0);
    let edge = vertical_edge(&body);
    let (start, end) = editing::oriented_edge_endpoints(&body, edge, &limits()).unwrap();
    for fraction in [0.2, 0.5, 0.8] {
        let point = std::array::from_fn(|i| start[i] + fraction * (end[i] - start[i]));
        let (actual, projected) = editing::fillet_project_edge(&body, edge, point, &limits()).unwrap();
        assert!(close(actual, fraction, 1e-8));
        for i in 0..3 { assert!(close(projected[i], point[i], 1e-8)); }
    }
    assert!(editing::fillet_project_edge(&body, usize::MAX, [0.0; 3], &limits()).is_err());
    assert!(editing::fillet_project_edge(&body, edge, [f64::NAN, 0.0, 0.0], &limits()).is_err());
    let uniform = editing::fillet_with_end_radii(&body, &[(edge, 1.0, 1.0)], deflection(), &limits()).unwrap();
    let controlled = editing::fillet_with_radius_controls(&body, &[(edge, 1.0, 1.0)], &[(edge, 0.5, 1.5)], deflection(), &limits()).unwrap();
    assert!(measured(&controlled).closed);
    assert!(measured(&controlled).volume_mm3 < measured(&uniform).volume_mm3);
    let slice = |shape: &Brep, z| {
        let tool = box_at([0.0, 0.0, z], 10.0, 10.0, 1.0);
        let result = editing::boolean(shape, &tool, BooleanOp::Intersection, deflection(), &limits()).unwrap();
        measured(result.body().unwrap()).volume_mm3
    };
    assert!(slice(&controlled, 4.5) < slice(&uniform, 4.5) - 0.05);
    assert!(slice(&controlled, 4.5) < slice(&controlled, 0.0) - 0.05);
    let near_start = editing::fillet_with_radius_controls(&body, &[(edge, 1.0, 1.0)], &[(edge, 0.25, 1.5)], deflection(), &limits()).unwrap();
    let near_end = editing::fillet_with_radius_controls(&body, &[(edge, 1.0, 1.0)], &[(edge, 0.75, 1.5)], deflection(), &limits()).unwrap();
    let z_start = if start[2] < end[2] { 2.0 } else { 7.0 };
    let z_end = if start[2] < end[2] { 7.0 } else { 2.0 };
    assert!(slice(&near_start, z_start) < slice(&near_end, z_start) - 0.05);
    assert!(slice(&near_end, z_end) < slice(&near_start, z_end) - 0.05);
    for controls in [vec![(edge, 0.0, 1.5)], vec![(edge, 1.0, 1.5)], vec![(edge, 0.5, 0.0)],
        vec![(edge, 0.5, 1.5), (edge, 0.5, 2.0)], vec![(usize::MAX, 0.5, 1.5)]] {
        assert!(editing::fillet_with_radius_controls(&body, &[(edge, 1.0, 1.0)], &controls, deflection(), &limits()).is_err());
    }
}

#[test]
fn fillet_linear_radius_changes_between_edge_endpoints() {
    let boxed = box_shape(10.0, 10.0, 10.0);
    let edge = vertical_edge(&boxed);
    let variable =
        editing::fillet_with_end_radii(&boxed, &[(edge, 1.0, 2.0)], deflection(), &limits())
            .unwrap();
    let reverse =
        editing::fillet_with_end_radii(&boxed, &[(edge, 2.0, 1.0)], deflection(), &limits())
            .unwrap();
    let m = measured(&variable);
    assert!(m.closed);
    let low = editing::fillet(&boxed, &[edge], 1.0, deflection(), &limits()).unwrap();
    let high = editing::fillet(&boxed, &[edge], 2.0, deflection(), &limits()).unwrap();
    assert!(m.volume_mm3 < measured(&low).volume_mm3);
    assert!(m.volume_mm3 > measured(&high).volume_mm3);
    assert!(close(m.volume_mm3, measured(&reverse).volume_mm3, 1e-5));
    let slice = |shape: &Brep, z| {
        let tool = box_at([0.0, 0.0, z], 10.0, 10.0, 1.0);
        let body = editing::boolean(
            shape,
            &tool,
            BooleanOp::Intersection,
            deflection(),
            &limits(),
        )
        .unwrap()
        .body()
        .unwrap()
        .clone();
        measured(&body).volume_mm3
    };
    let variable_difference = slice(&variable, 0.0) - slice(&variable, 9.0);
    let reverse_difference = slice(&reverse, 0.0) - slice(&reverse, 9.0);
    assert!(variable_difference.abs() > 0.3);
    assert!(variable_difference * reverse_difference < 0.0);
    let (start, end) = editing::oriented_edge_endpoints(&boxed, edge, &limits()).unwrap();
    assert!((start[2] - end[2]).abs() > 9.0);
    assert_eq!(variable_difference.is_sign_positive(), start[2] < end[2]);
    assert!(editing::oriented_edge_endpoints(&boxed, usize::MAX, &limits()).is_err());
    assert!(
        editing::fillet_with_end_radii(&boxed, &[(edge, 0.0, 2.0)], deflection(), &limits())
            .is_err()
    );
}

#[test]
fn m03_01_10_006() {
    let limits = limits();
    let boxed = box_shape(10.0, 10.0, 10.0);
    let edge = vertical_edge(&boxed);

    let filleted =
        editing::fillet(&boxed, &[edge], 1.0, deflection(), &limits).expect("fillet edge");
    let fillet_measures = measured(&filleted);
    let expected = 1000.0 - (1.0 - PI / 4.0) * 10.0;
    assert!(
        close(fillet_measures.volume_mm3, expected, 1e-5),
        "fillet volume {} != {expected}",
        fillet_measures.volume_mm3
    );
    assert!(fillet_measures.closed);
    assert_eq!(fillet_measures.naked_edges, 0);
    assert!(fillet_measures.faces > 6, "the fillet adds a face");
    // The unaffected extent is preserved.
    assert!(close(fillet_measures.bounds_mm[3], 10.0, 1e-6));

    let chamfered =
        editing::chamfer(&boxed, &[edge], 1.0, deflection(), &limits).expect("chamfer edge");
    let chamfer_measures = measured(&chamfered);
    assert!(
        close(chamfer_measures.volume_mm3, 995.0, 1e-5),
        "chamfer volume {}",
        chamfer_measures.volume_mm3
    );
    assert!(chamfer_measures.closed);

    // Negative: oversized radius fails in the kernel and leaves the input untouched; zero radius
    // and an unknown edge index are refused before the kernel.
    assert!(
        editing::fillet(&boxed, &[edge], 1000.0, deflection(), &limits).is_err(),
        "an oversized fillet radius must fail"
    );
    assert!(matches!(
        editing::fillet(&boxed, &[edge], 0.0, deflection(), &limits),
        Err(EditingError::Invalid(_))
    ));
    assert!(matches!(
        editing::fillet(&boxed, &[9999], 1.0, deflection(), &limits),
        Err(EditingError::Invalid(_))
    ));
    let open = creation::extrude_wire(
        &[
            [0.0, 0.0, 0.0],
            [10.0, 0.0, 0.0],
            [10.0, 10.0, 0.0],
            [0.0, 10.0, 0.0],
        ],
        [0.0, 0.0, 10.0],
        deflection(),
        &limits,
    )
    .expect("open shell");
    assert!(matches!(
        editing::fillet(&open, &[0], 1.0, deflection(), &limits),
        Err(EditingError::Invalid(_))
    ));
    // The original body is unchanged by a refused edit.
    assert!(close(measured(&boxed).volume_mm3, 1000.0, 1e-9));
}

/* ------------------------- 007 offset and thicken ------------------------- */

#[test]
fn shell_removes_two_opposite_faces_with_inward_walls() {
    let solid = box_shape(10.0, 10.0, 10.0);
    let top = top_face_index(&solid);
    let bottom = (0..solid.display.faces.len()).find(|&i|
        close(face_min_z(&solid, i), 0.0, 1e-9) && close(face_max_z(&solid, i), 0.0, 1e-9)
    ).unwrap();
    let result = editing::shell_faces(&solid, &[top, bottom], 1.0, deflection(), &limits()).unwrap();
    let m = measured(&result);
    assert!(m.closed && m.solids == 1);
    assert!(close(m.volume_mm3, 360.0, 1e-5), "{}", m.volume_mm3);
    let reverse = editing::shell_faces(&solid, &[bottom, top], 1.0, deflection(), &limits()).unwrap();
    assert!(close(measured(&reverse).volume_mm3, 360.0, 1e-5));
    assert!(editing::shell_faces(&solid, &[], 1.0, deflection(), &limits()).is_err());
    assert!(editing::shell_faces(&solid, &[top, top], 1.0, deflection(), &limits()).is_err());
    assert!(editing::shell_faces(&solid, &[usize::MAX], 1.0, deflection(), &limits()).is_err());
    let all: Vec<_> = (0..solid.display.faces.len()).collect();
    assert!(editing::shell_faces(&solid, &all, 1.0, deflection(), &limits()).is_err());
    assert!(editing::shell_faces(&solid, &[top], 0.0, deflection(), &limits()).is_err());
    let open = editing::remove_face(&solid, top, deflection(), &limits()).unwrap();
    assert!(editing::shell_faces(&open, &[0], 1.0, deflection(), &limits()).is_err());
}

#[test]
fn m03_01_10_007() {
    let limits = limits();
    let face = rect_face(10.0, 20.0);
    let thick = editing::thicken(&face, 2.0, deflection(), &limits).expect("thicken");
    let thick_measures = measured(&thick);
    assert!(
        close(thick_measures.volume_mm3, 400.0, 1e-5),
        "thicken volume {}",
        thick_measures.volume_mm3
    );
    assert!(thick_measures.closed);
    assert_eq!(thick_measures.solids, 1);
    assert!(
        close(thick_measures.size_mm()[2], 2.0, 1e-6),
        "thicken size {}",
        thick_measures.size_mm()[2]
    );

    let offset = editing::offset_shape(&face, 2.0, deflection(), &limits).expect("offset face");
    let offset_measures = measured(&offset);
    assert_eq!(offset_measures.faces, 1);
    assert!(close(offset_measures.area_mm2, 200.0, 1e-6));
    assert!(
        close(offset_measures.bounds_mm[2].abs(), 2.0, 1e-6),
        "offset plane z {}",
        offset_measures.bounds_mm[2]
    );

    // Negative: zero thickness/offset and thickening a solid.
    assert!(matches!(
        editing::thicken(&face, 0.0, deflection(), &limits),
        Err(EditingError::Invalid(_))
    ));
    assert!(matches!(
        editing::offset_shape(&face, 0.0, deflection(), &limits),
        Err(EditingError::Invalid(_))
    ));
    assert!(matches!(
        editing::thicken(&box_shape(10.0, 10.0, 10.0), 2.0, deflection(), &limits),
        Err(EditingError::Invalid(_))
    ));
}

/* ------------------------- 008 extract, unjoin, rejoin ------------------------- */

#[test]
fn straight_edge_endpoints_reject_curved_topology() {
    let limits = limits();
    let boxed = box_shape(10.0, 20.0, 30.0);
    for index in 0..boxed.display.edges.len() {
        let (a, b) = editing::straight_edge_endpoints(&boxed, index, &limits).unwrap();
        assert!(a != b);
    }
    let cylinder = creation::primitive(creation::Primitive::Cylinder { radius_mm: 3.0, height_mm: 5.0 }, deflection(), &limits).unwrap();
    let mut straight = 0;
    let mut curved = 0;
    for index in 0..cylinder.display.edges.len() {
        if editing::straight_edge_endpoints(&cylinder, index, &limits).is_ok() { straight += 1; } else { curved += 1; }
    }
    assert!(straight > 0 && curved >= 2, "straight={straight}, curved={curved}");
}

#[test]
fn m03_01_10_008() {
    let limits = limits();
    let boxed = box_shape(10.0, 20.0, 30.0);
    let before = measured(&boxed);

    // Pick a 10×20 face (area 600 mm²) by exact extraction, then unjoin and rejoin it.
    let mut selected = None;
    for index in 0..before.faces {
        if close(face_area(&boxed, index), 600.0, 1e-6) {
            selected = Some(index);
            break;
        }
    }
    let index = selected.expect("10×20 face");
    let face = editing::extract_face(&boxed, index, deflection(), &limits).expect("extract");
    let face_measures = measured(&face);
    assert_eq!(face_measures.faces, 1);
    assert!(close(face_measures.area_mm2, 600.0, 1e-6));
    // Extract never mutates the source body.
    assert!(close(measured(&boxed).volume_mm3, before.volume_mm3, 1e-9));

    let open = editing::remove_face(&boxed, index, deflection(), &limits).expect("unjoin");
    let open_measures = measured(&open);
    assert_eq!(open_measures.faces, 5);
    assert_eq!(open_measures.naked_edges, 4);
    assert!(!open_measures.closed);

    let rejoined = editing::sew(&open, &face, 1.0e-6, deflection(), &limits).expect("rejoin");
    let joined_measures = measured(&rejoined);
    assert!(joined_measures.closed);
    assert_eq!(joined_measures.faces, 6);
    assert!(close(joined_measures.volume_mm3, 6000.0, 1e-6));

    // Negative: unknown index; two faces from unrelated places stay an open join, not a solid.
    assert!(matches!(
        editing::extract_face(&boxed, 9999, deflection(), &limits),
        Err(EditingError::Invalid(_))
    ));
    assert!(matches!(
        editing::remove_face(&boxed, 9999, deflection(), &limits),
        Err(EditingError::Invalid(_))
    ));
    let other = rect_face(1.0, 1.0);
    let distant = editing::sew(&face, &other, 1.0e-6, deflection(), &limits).expect("open join");
    let distant_measures = measured(&distant);
    assert!(!distant_measures.closed);
    assert_eq!(distant_measures.solids, 0);
}

/* ------------------------- 009 health ------------------------- */

#[test]
fn m03_01_10_009() {
    let limits = limits();
    let boxed = box_shape(10.0, 10.0, 10.0);
    let closed = editing::geometry_health(&boxed, &limits).expect("closed health");
    assert!(closed.valid);
    assert!(closed.closed);
    assert_eq!(closed.naked_edges, 0);
    assert_eq!(closed.solids, 1);
    assert_eq!(closed.faces, 6);
    assert_eq!(closed.source_sha256, boxed.display.source_sha256);
    assert!(closed.is_solid());

    let top = top_face_index(&boxed);
    let open = editing::remove_face(&boxed, top, deflection(), &limits).expect("open shell");
    let open_health = editing::geometry_health(&open, &limits).expect("open health");
    assert!(open_health.valid);
    assert!(!open_health.closed);
    assert_eq!(open_health.naked_edges, 4);
    assert_eq!(open_health.faces, 5);
    assert_eq!(open_health.solids, 0);
    assert!(!open_health.is_solid());

    // The count is exact topology: removing one face adds exactly its four boundary edges, no
    // matter how the face is tessellated (a mesh boundary is never consulted).
    assert_eq!(open_health.naked_edges, closed.naked_edges + 4);
    let tessellated = creation::measure(&boxed, &limits).expect("tessellated measure");
    assert_eq!(tessellated.naked_edges, 0);
}

/* ------------------------- regression and stress ------------------------- */

#[test]
fn m03_01_10_regression_boolean_identity_and_round_trip() {
    let limits = limits();
    let a = box_shape(10.0, 10.0, 10.0);
    let b = box_at([5.0, 0.0, 0.0], 10.0, 10.0, 10.0);
    let union = editing::boolean(&a, &b, BooleanOp::Union, deflection(), &limits)
        .expect("union")
        .body()
        .cloned()
        .expect("body");
    let common = editing::boolean(&a, &b, BooleanOp::Intersection, deflection(), &limits)
        .expect("common")
        .body()
        .cloned()
        .expect("body");
    let union_volume = measured(&union).volume_mm3;
    let common_volume = measured(&common).volume_mm3;
    // Inclusion–exclusion: |A∪B| + |A∩B| = |A| + |B|.
    assert!(close(union_volume + common_volume, 2000.0, 1e-6));
    // Every result reopens from its canonical source to the same measured solid.
    for brep in [&union, &common] {
        let reopened =
            om9_kernel::brep::read_brep(brep.source(), deflection(), &limits).expect("reopen");
        assert!(close(
            measured(&reopened).volume_mm3,
            measured(brep).volume_mm3,
            1e-6
        ));
    }
}

#[test]
fn m03_01_10_stress_union_chain_and_fillet_family() {
    let limits = limits();
    // A chain of 20 unit cubes overlapping by 0.5 mm: the union volume is 20 − 19×0.5 = 10.5.
    let mut chain = box_at([0.0, 0.0, 0.0], 1.0, 1.0, 1.0);
    for step in 1..20 {
        let next = box_at([step as f64 * 0.5, 0.0, 0.0], 1.0, 1.0, 1.0);
        chain = editing::boolean(&chain, &next, BooleanOp::Union, deflection(), &limits)
            .expect("chain union")
            .body()
            .cloned()
            .expect("chain body");
    }
    let chain_measures = measured(&chain);
    assert!(
        close(chain_measures.volume_mm3, 10.5, 1e-6),
        "chain volume {}",
        chain_measures.volume_mm3
    );
    assert_eq!(chain_measures.solids, 1);

    // Fillet radius family: volume decreases monotonically and every result stays valid.
    let boxed = box_shape(10.0, 10.0, 10.0);
    let edge = vertical_edge(&boxed);
    let mut previous = measured(&boxed).volume_mm3;
    for radius in [0.5, 1.0, 1.5, 2.0] {
        let filleted =
            editing::fillet(&boxed, &[edge], radius, deflection(), &limits).expect("fillet step");
        let volume = measured(&filleted).volume_mm3;
        assert!(volume < previous, "radius {radius}: {volume} !< {previous}");
        assert_eq!(measured(&filleted).naked_edges, 0);
        previous = volume;
    }
}

/* ------------------------- measured receipt ------------------------- */

/// Prints the measured oracle for every L3 fixture as one JSON object. The same test asserts the
/// values, so a printed line is also a pass.
#[test]
fn m03_01_10_receipt() {
    let limits = limits();
    let mut report = String::from("{\"measured\":{");
    let mut push = |name: &str, value: f64| {
        report.push_str(&format!("\"{name}\":{value:.12},"));
    };

    let a = box_shape(10.0, 10.0, 10.0);
    let b = box_at([5.0, 0.0, 0.0], 10.0, 10.0, 10.0);
    let union = editing::boolean(&a, &b, BooleanOp::Union, deflection(), &limits)
        .expect("union")
        .body()
        .cloned()
        .expect("body");
    let union_volume = measured(&union).volume_mm3;
    push("union_volume_mm3", union_volume);
    assert!(close(union_volume, 1500.0, 1e-6));
    let difference = editing::boolean(&a, &b, BooleanOp::Difference, deflection(), &limits)
        .expect("difference")
        .body()
        .cloned()
        .expect("body");
    let difference_volume = measured(&difference).volume_mm3;
    push("difference_volume_mm3", difference_volume);
    assert!(close(difference_volume, 500.0, 1e-6));
    let intersection = editing::boolean(&a, &b, BooleanOp::Intersection, deflection(), &limits)
        .expect("intersection")
        .body()
        .cloned()
        .expect("body");
    let intersection_volume = measured(&intersection).volume_mm3;
    push("intersection_volume_mm3", intersection_volume);
    assert!(close(intersection_volume, 500.0, 1e-6));

    let face = rect_face(10.0, 20.0);
    let split = editing::split_face(
        &face,
        &[[5.0, -5.0, 0.0], [5.0, 25.0, 0.0]],
        deflection(),
        &limits,
    )
    .expect("split");
    let split_area = measured(&split.brep).area_mm2;
    push("split_area_mm2", split_area);
    assert!(close(split_area, 200.0, 1e-6));

    let boxed = box_shape(10.0, 10.0, 10.0);
    let open = editing::remove_face(&boxed, top_face_index(&boxed), deflection(), &limits)
        .expect("open shell");
    let capped = editing::cap_planar(&open, 1.0e-6, deflection(), &limits).expect("cap");
    let capped_volume = measured(&capped).volume_mm3;
    push("capped_volume_mm3", capped_volume);
    assert!(close(capped_volume, 1000.0, 1e-6));

    let edge = vertical_edge(&boxed);
    let filleted = editing::fillet(&boxed, &[edge], 1.0, deflection(), &limits).expect("fillet");
    let fillet_volume = measured(&filleted).volume_mm3;
    push("fillet_one_edge_volume_mm3", fillet_volume);
    assert!(close(fillet_volume, 1000.0 - (1.0 - PI / 4.0) * 10.0, 1e-5));
    let chamfered = editing::chamfer(&boxed, &[edge], 1.0, deflection(), &limits).expect("chamfer");
    let chamfer_volume = measured(&chamfered).volume_mm3;
    push("chamfer_one_edge_volume_mm3", chamfer_volume);
    assert!(close(chamfer_volume, 995.0, 1e-5));

    let thick = editing::thicken(&face, 2.0, deflection(), &limits).expect("thicken");
    let thick_volume = measured(&thick).volume_mm3;
    push("thicken_volume_mm3", thick_volume);
    assert!(close(thick_volume, 400.0, 1e-5));
    let offset = editing::offset_shape(&face, 2.0, deflection(), &limits).expect("offset");
    let offset_area = measured(&offset).area_mm2;
    push("offset_area_mm2", offset_area);
    assert!(close(offset_area, 200.0, 1e-6));

    let boxed_3d = box_shape(10.0, 20.0, 30.0);
    let mut selected = 0;
    for index in 0..measured(&boxed_3d).faces {
        if close(face_area(&boxed_3d, index), 600.0, 1e-6) {
            selected = index;
            break;
        }
    }
    let extracted =
        editing::extract_face(&boxed_3d, selected, deflection(), &limits).expect("extract");
    let extracted_area = measured(&extracted).area_mm2;
    push("extracted_face_area_mm2", extracted_area);
    assert!(close(extracted_area, 600.0, 1e-6));

    let health = editing::geometry_health(&boxed, &limits).expect("health");
    push("closed_naked_edges", health.naked_edges as f64);
    assert_eq!(health.naked_edges, 0);
    let open_health = editing::geometry_health(&open, &limits).expect("open health");
    push("open_naked_edges", open_health.naked_edges as f64);
    assert_eq!(open_health.naked_edges, 4);

    report.pop();
    report.push_str("}}");
    println!("{report}");
}
