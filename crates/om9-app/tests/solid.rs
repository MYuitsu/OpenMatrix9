// SPDX-License-Identifier: LGPL-2.1-or-later
//! Solid menu workflow bindings (`OM9-L2-03-18`); every enabled command dispatches to
//! the OCCT owner in `om9-kernel`.
#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::{
    budget::Limits,
    geometry::Geometry,
    model::{Document, Entity},
};
use om9_kernel::creation;

fn controller() -> Controller {
    Controller::new(Document::new("solid"), Limits::default()).unwrap()
}

fn new_id(
    c: &Controller,
    before: &std::collections::BTreeSet<om9_core::identity::ObjectId>,
) -> om9_core::identity::ObjectId {
    *c.document()
        .entities
        .keys()
        .find(|id| !before.contains(id))
        .expect("new solid")
}

fn measures(c: &Controller, id: om9_core::identity::ObjectId) -> creation::Measures {
    let Geometry::Brep(brep) = &c.document().entities[&id].geometry else {
        panic!("not a B-Rep");
    };
    creation::measure(brep, c.coordinator().limits()).unwrap()
}

#[test]
fn solid_rib_development_stages_selection_distance_and_undo() {
    use om9_kernel::creation::{self,Profile,DEFAULT_DEFLECTION_MM};
    let limits=Limits::default();let boundary=creation::planar(&Profile::new(vec![[0.,0.,5.],[10.,0.,5.],[10.,10.,5.],[0.,10.,5.]]).unwrap(),DEFAULT_DEFLECTION_MM,&limits).unwrap();
    let mut doc=Document::new("Rib development session");let source=Entity::new("rail",Geometry::Curve(om9_core::curve_build::polyline(&[[2.,5.,0.],[6.,5.,0.]]).unwrap()),None);
    let source_id=source.id;doc.entities.insert(source_id,source);let face=Entity::new("boundary",Geometry::Brep(boundary),None);let face_id=face.id;doc.entities.insert(face_id,face);
    let mut c=Controller::new(doc,limits).unwrap();let before=c.document().clone();
    c.begin_rib_development().unwrap();assert!(c.solid_pipe_needs_curve());c.solid_pipe_pick_curve(source_id).unwrap();c.submit("").unwrap();assert!(c.prompt().starts_with("Select Rib boundary"));
    assert!(c.solid_boolean_needs_object());c.solid_boolean_pick(face_id,false).unwrap();c.submit("").unwrap();assert!(c.prompt().starts_with("Rib signed distance"));
    assert!(!c.solid_boolean_needs_object());assert!(c.solid_boolean_pick(face_id,false).is_err());
    assert!(c.submit("Distance=0").is_err());assert_eq!(c.document(),&before);assert_eq!(c.coordinator().undo_len(),0);
    c.submit("Undo").unwrap();assert!(c.prompt().starts_with("Select Rib boundary"));c.select_objects(&[face_id]).unwrap();c.submit("").unwrap();
    c.submit("dIsTaNcE=2").unwrap();let m=measures(&c,face_id);assert_eq!(m.faces,5);assert!((m.area_mm2-152.).abs()<1e-6,"{m:?}");
    c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();c.undo().unwrap();
    c.select_objects(&[source_id]).unwrap();c.begin_rib_development().unwrap();c.submit("").unwrap();c.cancel();
    assert_eq!(c.document(),&before);assert_eq!(c.coordinator().undo_len(),0);
}

#[test]
fn solid_rib_development_multiple_curves_commit_as_one() {
    use om9_kernel::creation::{self,Profile,DEFAULT_DEFLECTION_MM};
    let limits=Limits::default();let boundary=creation::planar(&Profile::new(vec![[0.,0.,5.],[10.,0.,5.],[10.,10.,5.],[0.,10.,5.]]).unwrap(),DEFAULT_DEFLECTION_MM,&limits).unwrap();
    let mut doc=Document::new("Rib multiple curves");
    let a=Entity::new("a",Geometry::Curve(om9_core::curve_build::polyline(&[[2.,3.,0.],[6.,3.,0.]]).unwrap()),None);
    let b=Entity::new("b",Geometry::Curve(om9_core::curve_build::polyline(&[[2.,8.,0.],[6.,8.,0.]]).unwrap()),None);
    let face=Entity::new("boundary",Geometry::Brep(boundary),None);let (a,b,f)=(a,b,face);
    let (a_id,b_id,f_id)=(a.id,b.id,f.id);doc.entities.insert(a_id,a);doc.entities.insert(b_id,b);doc.entities.insert(f_id,f);
    let mut c=Controller::new(doc,limits).unwrap();let before=c.document().clone();c.begin_rib_development().unwrap();
    c.solid_pipe_pick_curve(a_id).unwrap();c.solid_pipe_pick_curve(b_id).unwrap();c.submit("").unwrap();
    c.solid_boolean_pick(f_id,false).unwrap();c.submit("").unwrap();c.submit("Distance=1").unwrap();
    let m=measures(&c,f_id);assert_eq!(m.faces,9);assert!((m.area_mm2-192.).abs()<1e-6,"{m:?}");
    assert_eq!(c.coordinator().undo_len(),1);c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();
}

#[test]
fn solid_rib_internal_straight_publication_is_atomic() {
    use om9_core::geometry::Transform;
    use om9_kernel::creation::{self,Profile,DEFAULT_DEFLECTION_MM};
    let limits=Limits::default();
    let face=creation::planar(&Profile::new(vec![[0.,0.,5.],[10.,0.,5.],[10.,10.,5.],[0.,10.,5.]]).unwrap(),DEFAULT_DEFLECTION_MM,&limits).unwrap();
    for reflected in [false,true] {
        let transform=if reflected {Transform([[-2.,0.,0.,10.],[0.,2.,0.,20.],[0.,0.,2.,3.],[0.,0.,0.,1.]])}
            else {Transform::translation([10.,20.,3.])};
        let mut doc=Document::new("Rib internal");
        let mut source=Entity::new("rail",Geometry::Curve(om9_core::curve_build::polyline(&[[2.,5.,0.],[6.,5.,0.]]).unwrap()),None);source.placement=transform;
        let source_id=source.id;doc.entities.insert(source_id,source);
        let mut target=Entity::new("boundary",Geometry::Brep(face.clone()),None);target.placement=transform;
        let boundary_id=target.id;doc.entities.insert(boundary_id,target);let mut c=Controller::new(doc,limits).unwrap();
        let before=c.document().clone();let source_stamp=c.document().stamp(source_id).unwrap();let target_stamp=c.document().stamp(boundary_id).unwrap();
        c.solid_rib_commit_straight(source_stamp,target_stamp,[0.,0.,1.],if reflected {4.} else {2.}).unwrap();
        let m=measures(&c,boundary_id);assert_eq!(m.faces,5);assert_eq!(m.shells,1);assert!((m.area_mm2-152.).abs()<1e-6,"{m:?}");
        assert_eq!(c.coordinator().undo_len(),1);c.undo().unwrap();assert_eq!(c.document(),&before);
        c.redo().unwrap();c.undo().unwrap();
        assert!(c.solid_rib_commit_straight(source_stamp,target_stamp,[0.,0.,1.],0.).is_err());
        assert_eq!(c.document(),&before);assert_eq!(c.coordinator().undo_len(),0);
    }
}

#[test]
fn solid_slab_development_uses_source_plane_and_validates_incplane() {
    let mut doc=Document::new("Slab source plane");
    let rail=Entity::new("vertical rail",Geometry::Curve(om9_core::curve_build::polyline(&[[0.,0.,0.],[0.,4.,0.],[0.,4.,3.]]).unwrap()),None);
    let id=rail.id;doc.entities.insert(id,rail);let mut c=Controller::new(doc,Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();let before=c.document().clone();c.begin_slab_development().unwrap();
    assert!(c.submit("InCPlane=Yes").is_err());assert_eq!(c.document(),&before);
    c.submit("InCPlane=No").unwrap();
    let plane=c.slab_offset_input_plane().unwrap();assert!(plane.normal()[0].abs()>0.999);
    let side=[0.,6.,2.];let outline=c.solid_preview(side);assert_eq!(outline.len(),6);
    c.accept_world_point(side).unwrap();let preview=c.solid_preview([3.,0.,0.]);assert_eq!(preview.len(),18);
    c.accept_world_point([3.,0.,0.]).unwrap();let e=c.document().entities.values().find(|e|e.id!=id).unwrap();
    let m=measures(&c,e.id);assert!((m.volume_mm3-54.).abs()<1e-7,"{m:?}");
    c.undo().unwrap();assert_eq!(c.document(),&before);
}

#[test]
fn solid_slab_development_distance_requires_side_and_throughpoint_uses_pick() {
    for mode in ["positive", "negative", "through"] {
        let mut doc=Document::new("Slab distance side");
        let rail=Entity::new("rail",Geometry::Curve(om9_core::curve_build::polyline(&[[0.,0.,0.],[4.,0.,0.]]).unwrap()),None);
        let id=rail.id;doc.entities.insert(id,rail);
        let mut c=Controller::new(doc,Limits::default()).unwrap();c.select_objects(&[id]).unwrap();let before=c.document().clone();
        c.begin_slab_development().unwrap();c.submit("Distance=2").unwrap();
        assert!(c.prompt().starts_with("Pick Slab offset side"));assert_eq!(c.document(),&before);
        if mode=="through" { c.submit("ThroughPoint").unwrap(); }
        let side=if mode=="positive" {-7.} else if mode=="negative" {7.} else {3.};
        let edges=c.solid_preview([0.,side,0.]);assert_eq!(edges.len(),4);
        c.accept_world_point([0.,side,0.]).unwrap();
        if mode=="positive" { c.submit("Undo").unwrap();assert!(c.prompt().starts_with("Pick Slab offset side"));c.accept_world_point([0.,side,0.]).unwrap(); }
        c.submit("Height=3").unwrap();
        let entity=c.document().entities.values().find(|e|e.id!=id).unwrap();let m=measures(&c,entity.id);
        assert!((m.volume_mm3-if mode=="through" {36.} else {24.}).abs()<1e-7,"{mode} {m:?}");
        if mode=="positive" {assert!((m.bounds_mm[1]+2.).abs()<1e-7);} else {assert!(m.bounds_mm[4]>1.9);}
        c.undo().unwrap();assert_eq!(c.document(),&before);
    }
}

#[test]
fn solid_slab_development_pick_uses_nearest_polyline_segment() {
    let mut doc=Document::new("Slab L pick");
    let rail=Entity::new("rail",Geometry::Curve(om9_core::curve_build::polyline(&[[0.,0.,0.],[4.,0.,0.],[4.,3.,0.]]).unwrap()),None);
    let id=rail.id;doc.entities.insert(id,rail);
    let mut c=Controller::new(doc,Limits::default()).unwrap();c.select_objects(&[id]).unwrap();
    let before=c.document().clone();c.begin_slab_development().unwrap();
    assert!(c.accept_world_point([5.,-2.,0.]).is_err());
    assert_eq!(c.document(),&before);assert!(c.prompt().starts_with("Slab offset"));
    let outline=c.solid_preview([6.,2.,0.]);assert_eq!(outline.len(),6);
    c.accept_world_point([6.,2.,0.]).unwrap();
    let preview=c.solid_preview([0.,0.,3.]);assert_eq!(preview.len(),18);
    c.accept_world_point([0.,0.,3.]).unwrap();
    let result=c.document().entities.values().find(|e|e.id!=id).unwrap();
    let measured=measures(&c,result.id);assert!((measured.volume_mm3-54.).abs()<1e-7,"{measured:?}");
    assert!((measured.bounds_mm[3]-6.).abs()<1e-7 && (measured.bounds_mm[1]+2.).abs()<1e-7,"{measured:?}");
    c.undo().unwrap();assert_eq!(c.document(),&before);
}

#[test]
fn solid_slab_development_points_preview_and_commit_agree() {
    for (side, height, both) in [(-2.,3.,false),(2.,-3.,false),(-2.,3.,true)] {
        let mut doc=Document::new("Slab point preview");
        let rail=Entity::new("line",Geometry::Curve(om9_core::curve_build::polyline(&[[0.,0.,0.],[4.,0.,0.]]).unwrap()),None);
        let id=rail.id; doc.entities.insert(id,rail);
        let mut c=Controller::new(doc,Limits::default()).unwrap();c.select_objects(&[id]).unwrap();
        let before=c.document().clone();c.begin_slab_development().unwrap();
        if both { c.submit("BothSides=Yes").unwrap(); }
        assert_eq!(c.solid_preview([0.,side,0.]).len(),4);
        c.accept_world_point([0.,side,0.]).unwrap();
        let edges=c.solid_preview([0.,0.,height]);assert_eq!(edges.len(),12);
        let mut low=[f64::INFINITY;3];let mut high=[f64::NEG_INFINITY;3];
        for edge in &edges {for p in edge {for i in 0..3 {low[i]=low[i].min(p[i]);high[i]=high[i].max(p[i]);}}}
        c.accept_world_point([0.,0.,height]).unwrap();
        let output=c.document().entities.values().find(|e| e.id!=id).unwrap();
        let measured=measures(&c,output.id);
        for i in 0..3 {assert!((measured.bounds_mm[i]-low[i]).abs()<1e-6);assert!((measured.bounds_mm[i+3]-high[i]).abs()<1e-6);}
        assert!((measured.volume_mm3-if both {48.} else {24.}).abs()<1e-6);
        c.undo().unwrap();assert_eq!(c.document(),&before);
    }
}

#[test]
fn solid_slab_development_session_units_undo_cancel_and_retry() {
    use om9_core::geometry::Unit;
    let mut doc = Document::new("Slab staged input"); doc.page_unit = Unit::Centimeter;
    let source = Entity::new("line",Geometry::Curve(om9_core::curve_build::polyline(&[[0.,0.,0.],[40.,0.,0.]]).unwrap()),None);
    let id = source.id; doc.entities.insert(id,source);
    let mut c = Controller::new(doc,Limits::default()).unwrap(); c.select_objects(&[id]).unwrap();
    let before = c.document().clone(); c.begin_slab_development().unwrap();
    c.submit("InCPlane=Yes").unwrap(); c.submit("InCPlane=No").unwrap();
    assert!(c.submit("Distance=0").is_err()); assert_eq!(c.document(),&before);
    c.submit("Distance=0.2").unwrap(); c.submit("Undo").unwrap();
    assert!(c.prompt().starts_with("Slab offset"));
    assert!(c.submit("Distance=-0.2").is_err());
    c.submit("Distance=0.2").unwrap(); c.accept_world_point([0.,2.,0.]).unwrap(); c.submit("BothSides=Yes").unwrap();
    assert!(c.submit("Height=0").is_err()); assert_eq!(c.document(),&before);
    c.submit("Height=3").unwrap();
    let output = c.document().entities.values().find(|e| e.id != id).unwrap();
    assert!((measures(&c,output.id).volume_mm3-4800.).abs()<1e-6);
    assert_eq!(c.coordinator().undo_len(),1); c.undo().unwrap(); assert_eq!(c.document(),&before);
    c.select_objects(&[id]).unwrap(); c.begin_slab_development().unwrap(); c.submit("1").unwrap(); c.cancel();
    assert_eq!(c.document(),&before);
    c.select_objects(&[id]).unwrap(); c.begin_slab_development().unwrap(); c.submit("1").unwrap(); c.accept_world_point([0.,2.,0.]).unwrap();
    c.delete_objects(&[id]).unwrap(); let after = c.document().clone(); let count = c.coordinator().undo_len();
    assert!(c.submit("2").is_err()); assert_eq!(c.document(),&after); assert_eq!(c.coordinator().undo_len(),count);
    c.cancel();
}

#[test]
fn solid_slab_development_open_arc_uses_bounded_angles_and_preview() {
    use om9_core::geometry::Curve;
    let pi=std::f64::consts::PI;
    for (angles,pick) in [([0.,pi/2.],[7./2_f64.sqrt(),7./2_f64.sqrt(),0.]),([7.*pi/4.,9.*pi/4.],[7.,0.,0.])] {
        let mut doc=Document::new("Slab arc session");
        let rail=Entity::new("arc",Geometry::Curve(Curve::Circle {center:[0.,0.,0.],x:[1.,0.,0.],y:[0.,1.,0.],radius:5.,domain:angles,angle_domain:None}),None);
        let id=rail.id;doc.entities.insert(id,rail);let mut c=Controller::new(doc,Limits::default()).unwrap();
        c.select_objects(&[id]).unwrap();let before=c.document().clone();c.begin_slab_development().unwrap();
        assert_eq!(c.solid_preview(pick).len(),128);c.accept_world_point(pick).unwrap();
        assert_eq!(c.solid_preview([0.,0.,3.]).len(),264);c.accept_world_point([0.,0.,3.]).unwrap();
        let output=c.document().entities.values().find(|e|e.id!=id).unwrap();let m=measures(&c,output.id);
        assert!((m.volume_mm3-18.*pi).abs()<1e-6,"{m:?}");c.undo().unwrap();assert_eq!(c.document(),&before);
    }
}

#[test]
fn solid_slab_development_circle_points_and_preview_match_publication() {
    use om9_core::geometry::Curve;
    for (pick,expected) in [([7.,0.,0.],72.),([3.,0.,0.],48.)] {
        let mut doc=Document::new("Slab circle session");
        let rail=Entity::new("circle",Geometry::Curve(Curve::Circle {center:[0.,0.,0.],x:[1.,0.,0.],y:[0.,1.,0.],radius:5.,domain:[0.,std::f64::consts::TAU],angle_domain:None}),None);
        let id=rail.id;doc.entities.insert(id,rail);let mut c=Controller::new(doc,Limits::default()).unwrap();
        c.select_objects(&[id]).unwrap();let before=c.document().clone();c.begin_slab_development().unwrap();
        c.submit("InCPlane=Yes").unwrap();
        assert!(c.accept_world_point([0.,0.,0.]).is_err());assert_eq!(c.document(),&before);
        let outline=c.solid_preview(pick);assert_eq!(outline.len(),128);
        c.accept_world_point(pick).unwrap();let preview=c.solid_preview([0.,0.,3.]);assert_eq!(preview.len(),272);
        c.accept_world_point([0.,0.,3.]).unwrap();
        let out=c.document().entities.values().find(|e|e.id!=id).unwrap();let m=measures(&c,out.id);
        assert!((m.volume_mm3-expected*std::f64::consts::PI).abs()<1e-6,"{m:?}");
        let outer=if expected==72. {7.} else {5.};
        assert!(preview.iter().flat_map(|edge|edge.iter()).all(|p|p[0].hypot(p[1])<=outer+1e-8));
        c.undo().unwrap();assert_eq!(c.document(),&before);
    }
}

fn slab_quadratic_polyline(points: &[[f64;3]], breaks: &[f64], reversed: bool) -> om9_core::geometry::Curve {
    use om9_core::geometry::{Curve,NurbsCurve};
    assert_eq!(breaks.len(),points.len()-2);
    let mut poles=vec![points[0]];
    for segment in points.windows(2) {
        poles.push(std::array::from_fn(|i|(segment[0][i]+segment[1][i])*0.5));
        poles.push(segment[1]);
    }
    let mut knots=vec![0.;3];for k in breaks {knots.extend([*k,*k]);}knots.extend([1.;3]);
    Curve::Nurbs(NurbsCurve {degree:2,weights:vec![1.;poles.len()],poles,knots,domain:[0.,1.],periodic:false,reversed,seam:None})
}

#[test]
fn solid_slab_degree_one_nurbs_uses_mitered_polyline_publication() {
    use om9_core::geometry::{Curve,NurbsCurve,Transform};
    for reversed in [false,true] {
        let curve=Curve::Nurbs(NurbsCurve {degree:1,poles:vec![[0.,0.,0.],[4.,0.,0.],[4.,3.,0.]],
            weights:vec![1.,2.,1.],knots:vec![0.,0.,0.5,1.,1.],domain:[0.,1.],periodic:false,reversed,seam:None});
        let mut doc=Document::new("Slab degree-one NURBS");
        let mut source=Entity::new("curve",Geometry::Curve(curve),None);
        source.placement=Transform::translation([10.,20.,5.]);let id=source.id;doc.entities.insert(id,source);
        let mut c=Controller::new(doc,Limits::default()).unwrap();c.select_objects(&[id]).unwrap();
        let before=c.document().clone();c.begin_slab_development().unwrap();
        let preview=c.solid_preview([16.,22.,5.]);assert_eq!(preview.len(),6);
        c.accept_world_point([16.,22.,5.]).unwrap();c.accept_world_point([10.,20.,8.]).unwrap();
        let output=c.document().entities.values().find(|e|e.id!=id).unwrap();let m=measures(&c,output.id);
        assert!((m.volume_mm3-54.).abs()<1e-7,"{m:?}");
        c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document().entities.len(),2);
    }
    let discontinuous=Curve::Nurbs(NurbsCurve {degree:1,poles:vec![[0.,0.,0.],[1.,0.,0.],[2.,0.,0.],[3.,0.,0.]],
        weights:vec![1.;4],knots:vec![0.,0.,0.5,0.5,1.,1.],domain:[0.,1.],periodic:false,reversed:false,seam:None});
    assert!(om9_core::slab::placed_open_polyline(&discontinuous,Transform::identity(),&Limits::default()).is_err());
    for reversed in [false,true] {
        let trimmed=Curve::Nurbs(NurbsCurve {degree:1,poles:vec![[0.,0.,0.],[4.,0.,0.],[4.,3.,0.]],
            weights:vec![1.,2.,1.],knots:vec![0.,0.,0.5,1.,1.],domain:[0.25,0.75],periodic:false,reversed,seam:None});
        let limits=Limits::default();let points=om9_core::slab::placed_open_polyline(&trimmed,Transform::identity(),&limits).unwrap();
        assert_eq!(points.len(),3);
        assert_eq!(points[0],om9_core::curve_ops::point_at(&trimmed,0.,&limits).unwrap());
        assert_eq!(points[2],om9_core::curve_ops::point_at(&trimmed,1.,&limits).unwrap());
        assert!((points[1][0]-4.).abs()<1e-10 && points[1][1].abs()<1e-10);
    }
}

#[test]
fn solid_slab_nurbs_pick_rejects_conflicting_equidistant_spans() {
    for reversed in [false,true] {
        let curve=slab_quadratic_polyline(&[[0.,0.,0.],[10.,0.,0.],[10.,4.,0.],[0.,4.,0.],[0.,1.,0.],[10.,1.,0.]], &[0.001,0.002,0.003,0.004], reversed);
        let mut doc=Document::new("Slab ambiguous NURBS pick");
        let entity=Entity::new("curve",Geometry::Curve(curve),None);let id=entity.id;doc.entities.insert(id,entity);
        let mut c=Controller::new(doc,Limits::default()).unwrap();c.select_objects(&[id]).unwrap();
        let before=c.document().clone();c.begin_slab_development().unwrap();
        assert!(c.solid_preview([2.3,0.5,0.]).is_empty());
        assert!(c.accept_world_point([2.3,0.5,0.]).is_err());
        assert_eq!(c.document(),&before);assert_eq!(c.coordinator().undo_len(),0);
        assert!(!c.solid_preview([2.3,0.4,0.]).is_empty());
    }
}

#[test]
fn solid_slab_nurbs_pick_checks_competing_knot_spans() {
    for reversed in [false,true] {
    let curve=slab_quadratic_polyline(&[[0.,0.,0.],[10.,0.,0.],[10.,1.,0.],[0.,1.,0.]], &[0.001,0.002], reversed);
    let mut doc=Document::new("Slab competing NURBS spans");
    let entity=Entity::new("curve",Geometry::Curve(curve),None);let id=entity.id;doc.entities.insert(id,entity);
    let mut c=Controller::new(doc,Limits::default()).unwrap();c.select_objects(&[id]).unwrap();
    let before=c.document().clone();c.begin_slab_development().unwrap();
    let preview=c.solid_preview([2.3,0.4,0.]);
    let outer_start=preview.last().unwrap()[0];
    assert!((outer_start[1]-if reversed {0.6} else {0.4}).abs()<1e-4,
        "offset must use the nearest short span: {outer_start:?}");
    assert!(!c.solid_preview([2.3,0.5,0.]).is_empty(), "equal distances with matching signed offsets are not conflicting");
    assert_eq!(c.document(),&before);assert_eq!(c.coordinator().undo_len(),0);
    }
}

#[test]
fn solid_slab_rejects_unsampled_out_of_plane_nurbs_without_transaction() {
    use om9_core::geometry::Curve;
    let curve = slab_quadratic_polyline(&[[0.,0.,0.], [0.001,0.,0.], [0.002,0.,1.], [0.003,0.,0.], [1.,0.,0.]], &[0.001,0.002,0.003], false);
    let limits = Limits::default();
    let sampled = om9_core::curve_ops::sample_curve(&curve, 65, &limits).unwrap();
    assert!(sampled.iter().all(|p| p[2].abs() < 1e-12));
    assert!(om9_core::curve_ops::point_at(&curve, 0.002, &limits).unwrap()[2] > 0.9);
    assert!(matches!(om9_core::slab::nurbs_slab(&curve,[0.,0.,1.],0.1,1.,false,1e-4,&limits),
        Err(om9_core::Error::Invalid("Slab NURBS control hull must be planar"))));
    let mut doc = Document::new("Slab narrow nonplanar span");
    let entity = Entity::new("curve", Geometry::Curve(curve), None);
    let id = entity.id; doc.entities.insert(id,entity);
    let mut c = Controller::new(doc,limits).unwrap();
    let before = c.document().clone();
    let stamp = c.document().stamp(id).unwrap();
    assert!(c.solid_slab_commit_polyline(stamp,[0.,0.,1.],0.1,1.,false).is_err());
    assert_eq!(c.document(),&before); assert_eq!(c.coordinator().undo_len(),0);
    c.select_objects(&[id]).unwrap();
    c.begin_slab_development().unwrap();
    let normal = c.slab_offset_input_plane().unwrap().normal();
    assert!(normal[1].abs() > 0.999, "narrow span defines the XZ plane: {normal:?}");
    assert!(c.submit("InCPlane=Yes").is_err());
    let preview = c.solid_preview([0.5,0.,2.]);
    assert!(preview.iter().flat_map(|edge| edge.iter()).any(|p| p[2] > 0.99 && p[0] < 0.01), "preview must include the narrow knot span");
    let near_spike = c.solid_preview([0.1015,0.,0.5]);
    assert!(near_spike.iter().flat_map(|edge|edge.iter()).any(|p|
        (p[0]-0.1015).abs()<0.002 && (p[2]-0.5).abs()<0.002), "offset must be measured against the short span: {near_spike:?}");
    assert_eq!(c.document(),&before); assert_eq!(c.coordinator().undo_len(),0);
    c.submit("Cancel").unwrap();

    let Geometry::Curve(Curve::Nurbs(mut spatial)) = c.document().entities[&id].geometry.clone() else { panic!("NURBS source") };
    spatial.poles[1][1] = 0.5;
    let mut doc = Document::new("Slab nonplanar narrow spans");
    let entity = Entity::new("curve",Geometry::Curve(Curve::Nurbs(spatial)),None);
    let id = entity.id; doc.entities.insert(id,entity);
    let mut c = Controller::new(doc,Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap(); let before = c.document().clone();
    assert!(c.begin_slab_development().is_err());
    assert_eq!(c.document(),&before); assert_eq!(c.coordinator().undo_len(),0);
}

#[test]
fn solid_slab_development_reversed_nurbs_preserves_selected_side() {
    use om9_core::geometry::{Curve,NurbsCurve};
    let arc=Curve::Nurbs(NurbsCurve {degree:2,poles:vec![[5.,0.,0.],[5.,5.,0.],[0.,5.,0.]],
        weights:vec![1.,std::f64::consts::FRAC_1_SQRT_2,1.],knots:vec![0.,0.,0.,1.,1.,1.],domain:[0.,1.],periodic:false,reversed:true,seam:None});
    let mut doc=Document::new("Slab reversed NURBS");let source=Entity::new("arc",Geometry::Curve(arc),None);
    let id=source.id;doc.entities.insert(id,source);let mut c=Controller::new(doc,Limits::default()).unwrap();c.select_objects(&[id]).unwrap();
    let before=c.document().clone();c.begin_slab_development().unwrap();
    let pick=[3./2_f64.sqrt(),3./2_f64.sqrt(),0.];assert!(!c.solid_preview(pick).is_empty());
    c.accept_world_point(pick).unwrap();assert!(!c.solid_preview([0.,0.,3.]).is_empty());
    c.accept_world_point([0.,0.,3.]).unwrap();let output=c.document().entities.values().find(|e|e.id!=id).unwrap();
    let m=measures(&c,output.id);assert!((m.volume_mm3-12.*std::f64::consts::PI).abs()<0.01,"{m:?}");
    c.undo().unwrap();assert_eq!(c.document(),&before);
}

#[test]
fn solid_slab_development_nurbs_point_preview_and_commit() {
    use om9_core::geometry::{Curve,NurbsCurve};
    let arc=Curve::Nurbs(NurbsCurve {degree:2,poles:vec![[5.,0.,0.],[5.,5.,0.],[0.,5.,0.]],
        weights:vec![1.,std::f64::consts::FRAC_1_SQRT_2,1.],knots:vec![0.,0.,0.,1.,1.,1.],domain:[0.,1.],periodic:false,reversed:false,seam:None});
    let mut doc=Document::new("Slab NURBS session");let source=Entity::new("arc",Geometry::Curve(arc),None);
    let id=source.id;doc.entities.insert(id,source);let mut c=Controller::new(doc,Limits::default()).unwrap();c.select_objects(&[id]).unwrap();
    let before=c.document().clone();c.begin_slab_development().unwrap();
    let pick=[7./2_f64.sqrt(),7./2_f64.sqrt(),0.];assert!(!c.solid_preview(pick).is_empty());
    c.accept_world_point(pick).unwrap();assert!(!c.solid_preview([0.,0.,3.]).is_empty());
    c.accept_world_point([0.,0.,3.]).unwrap();let output=c.document().entities.values().find(|e|e.id!=id).unwrap();
    let m=measures(&c,output.id);assert!((m.volume_mm3-18.*std::f64::consts::PI).abs()<0.01,"{m:?}");
    c.undo().unwrap();assert_eq!(c.document(),&before);
}

#[test]
fn solid_slab_internal_nurbs_publication_checks_fit_and_volume() {
    use om9_core::geometry::{Curve,NurbsCurve,Transform};
    let arc=Curve::Nurbs(NurbsCurve {degree:2,poles:vec![[5.,0.,0.],[5.,5.,0.],[0.,5.,0.]],
        weights:vec![1.,std::f64::consts::FRAC_1_SQRT_2,1.],knots:vec![0.,0.,0.,1.,1.,1.],domain:[0.,1.],periodic:false,reversed:false,seam:None});
    let mut doc=Document::new("Slab NURBS");let mut source=Entity::new("arc",Geometry::Curve(arc),None);
    source.placement=Transform::translation([10.,20.,5.]);let id=source.id;doc.entities.insert(id,source);
    let mut c=Controller::new(doc,Limits::default()).unwrap();let before=c.document().clone();let stamp=c.document().stamp(id).unwrap();
    assert!(c.solid_slab_commit_polyline(stamp,[0.,1.,0.],2.,3.,false).is_err());
    assert_eq!(c.document(),&before);assert_eq!(c.coordinator().undo_len(),0);
    c.solid_slab_commit_polyline(stamp,[0.,0.,1.],2.,3.,false).unwrap();
    let output=c.document().entities.values().find(|e|e.id!=id).unwrap();let m=measures(&c,output.id);
    assert!(m.closed && m.solids==1);assert!((m.volume_mm3-18.*std::f64::consts::PI).abs()<0.01,"{m:?}");
    c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document().entities.len(),2);
}

#[test]
fn solid_slab_internal_arc_publication_measures_sector_and_undo() {
    use om9_core::geometry::{Curve,Transform};
    for (offset,height,both,factor) in [(2.,3.,false,18.),(-2.,3.,false,12.),(2.,-3.,true,36.)] {
        let mut doc=Document::new("Slab arc");
        let mut source=Entity::new("arc",Geometry::Curve(Curve::Circle {center:[0.,0.,0.],x:[1.,0.,0.],y:[0.,1.,0.],radius:5.,domain:[0.,std::f64::consts::FRAC_PI_2],angle_domain:None}),None);
        source.placement=Transform([[0.,-1.,0.,10.],[1.,0.,0.,20.],[0.,0.,1.,5.],[0.,0.,0.,1.]]);
        let id=source.id;doc.entities.insert(id,source);let mut c=Controller::new(doc,Limits::default()).unwrap();
        let before=c.document().clone();let stamp=c.document().stamp(id).unwrap();
        assert!(c.solid_slab_commit_polyline(stamp,[0.,0.,1.],-5.,height,both).is_err());assert_eq!(c.document(),&before);
        c.solid_slab_commit_polyline(stamp,[0.,0.,1.],offset,height,both).unwrap();
        let e=c.document().entities.values().find(|e|e.id!=id).unwrap();let m=measures(&c,e.id);
        assert!(m.closed && m.solids==1);assert!((m.volume_mm3-factor*std::f64::consts::PI).abs()<1e-6,"{m:?}");
        c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document().entities.len(),2);
    }
}

#[test]
fn solid_slab_internal_circle_publication_is_exact_and_atomic() {
    use om9_core::geometry::{Curve, Transform};
    let sample=Curve::Circle {center:[0.,0.,0.],x:[1.,0.,0.],y:[0.,1.,0.],radius:5.,domain:[0.,std::f64::consts::TAU],angle_domain:None};
    let nonuniform=Transform([[2.,0.,0.,0.],[0.,1.,0.,0.],[0.,0.,1.,0.],[0.,0.,0.,1.]]);
    assert!(om9_core::slab::circle_slab(&sample,nonuniform,[0.,0.,1.],1.,3.,false,&Limits::default()).is_err());
    for (offset,height,both,expected_factor) in [(2.,3.,false,72.),(-2.,3.,false,48.),(2.,-3.,true,144.)] {
        let mut doc=Document::new("Slab circle");
        let mut rail=Entity::new("circle",Geometry::Curve(Curve::Circle {center:[0.,0.,0.],x:[1.,0.,0.],y:[0.,1.,0.],radius:5.,domain:[0.,std::f64::consts::TAU],angle_domain:None}),None);
        rail.placement=Transform([[1.,0.,0.,10.],[0.,0.,-1.,20.],[0.,1.,0.,5.],[0.,0.,0.,1.]]);
        let id=rail.id;doc.entities.insert(id,rail);let mut c=Controller::new(doc,Limits::default()).unwrap();
        let before=c.document().clone();let stamp=c.document().stamp(id).unwrap();
        assert!(c.solid_slab_commit_polyline(stamp,[0.,-1.,0.],-5.,height,both).is_err());assert_eq!(c.document(),&before);
        c.solid_slab_commit_polyline(stamp,[0.,-1.,0.],offset,height,both).unwrap();
        let output=c.document().entities.values().find(|e|e.id!=id).unwrap();let m=measures(&c,output.id);
        assert!(m.closed && m.solids==1);assert!((m.volume_mm3-expected_factor*std::f64::consts::PI).abs()<1e-6,"{m:?}");
        let base=output.placement.point([0.,0.,0.]).unwrap();
        let expected_y=if both {20.+height} else {20.};assert!((base[1]-expected_y).abs()<1e-8,"{base:?}");
        c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document().entities.len(),2);
    }
}

#[test]
fn solid_slab_internal_publication_checks_source_and_is_atomic() {
    use om9_core::geometry::Transform;
    let mut doc = Document::new("Slab source");
    let mut source = Entity::new("rail", Geometry::Curve(om9_core::curve_build::polyline(&[[0.,0.,0.],[4.,0.,0.],[4.,3.,0.]]).unwrap()), None);
    source.placement = Transform::translation([10.,20.,5.]);
    let id = source.id; doc.entities.insert(id,source);
    let mut c = Controller::new(doc,Limits::default()).unwrap();
    let before = c.document().clone();
    let stamp = c.document().stamp(id).unwrap();
    c.solid_slab_commit_polyline(stamp,[0.,0.,1.],1.,3.,false).unwrap();
    assert_eq!(c.coordinator().undo_len(),1);
    let output = c.document().entities.values().find(|e| e.id != id).unwrap();
    let measured = measures(&c,output.id);
    assert!((measured.volume_mm3-24.).abs() < 1e-7);
    let expected = [10.,19.,5.,15.,23.,8.];
    assert!((0..6).all(|i| (measured.bounds_mm[i] - expected[i]).abs() < 1e-6), "{:?}", measured.bounds_mm);
    c.undo().unwrap(); assert_eq!(c.document(),&before);
    c.redo().unwrap(); assert_eq!(c.document().entities.len(),2);
    c.undo().unwrap();
    c.delete_objects(&[id]).unwrap();
    let after = c.document().clone(); let undo = c.coordinator().undo_len();
    assert!(c.solid_slab_commit_polyline(stamp,[0.,0.,1.],1.,3.,false).is_err());
    assert_eq!(c.document(),&after); assert_eq!(c.coordinator().undo_len(),undo);
}

#[test]
fn solid_slab_internal_publication_rejects_collapsed_closed_source_without_commit() {
    let mut doc = Document::new("closed Slab source");
    let source = Entity::new("closed", Geometry::Curve(om9_core::curve_build::polyline(&[[0.,0.,0.],[4.,0.,0.],[4.,3.,0.],[0.,0.,0.]]).unwrap()), None);
    let id = source.id; doc.entities.insert(id,source);
    let mut c = Controller::new(doc,Limits::default()).unwrap();
    let before = c.document().clone(); let stamp = c.document().stamp(id).unwrap();
    assert!(c.solid_slab_commit_polyline(stamp,[0.,0.,1.],-100.,3.,false).is_err());
    assert_eq!(c.document(),&before); assert_eq!(c.coordinator().undo_len(),0);
}

#[test]
fn solid_slab_development_periodic_double_seam_creates_measured_body() {
    use om9_core::geometry::{Curve,NurbsCurve};
    let limits=Limits::default();let n=NurbsCurve {degree:3,
        poles:vec![[4.,0.,0.],[0.,4.,0.],[-4.,0.,0.],[0.,-4.,0.],[4.,0.,0.],[0.,4.,0.],[-4.,0.,0.]],
        weights:vec![1.,0.7,1.3,0.9,1.,0.7,1.3],knots:vec![0.,1.,2.,2.,4.,5.,6.,6.,8.,9.,10.],
        domain:[2.,6.],periodic:true,reversed:false,seam:None};
    let curve=Curve::Nurbs(n);let mut original=Vec::new();let mut shifted=Vec::new();
    for i in 0..4096 {let f=(i as f64+0.5)/4096.;let p=om9_core::curve_ops::point_at(&curve,f,&limits).unwrap();
        let t=om9_core::curve_ops::tangent_at(&curve,f,&limits).unwrap();original.push(p);shifted.push([p[0]+0.25*t[1],p[1]-0.25*t[0],0.]);}
    let area=|ring:&Vec<[f64;3]>|0.5*(0..ring.len()).map(|i| {let a=ring[i];let b=ring[(i+1)%ring.len()];a[0]*b[1]-b[0]*a[1]}).sum::<f64>().abs();
    let expected=2.*(area(&shifted)-area(&original));
    let mut doc=Document::new("Slab periodic double seam");let source=Entity::new("rail",Geometry::Curve(curve),None);
    let id=source.id;doc.entities.insert(id,source);let mut c=Controller::new(doc,limits).unwrap();c.select_objects(&[id]).unwrap();
    let before=c.document().clone();c.begin_slab_development().unwrap();c.submit("Distance=0.25").unwrap();
    assert!(!c.solid_preview([5.,0.,0.]).is_empty());c.accept_world_point([5.,0.,0.]).unwrap();c.submit("Height=2").unwrap();
    let result=c.document().entities.values().find(|e|e.id!=id).unwrap();let m=measures(&c,result.id);
    assert!(m.closed && m.solids==1);assert!((m.volume_mm3-expected).abs()<0.02,"{m:?} expected {expected}");
    c.undo().unwrap();assert_eq!(c.document(),&before);
}

#[test]
fn solid_slab_certified_periodic_nurbs_foundation() {
    use om9_core::geometry::{Curve,NurbsCurve};
    let limits=Limits::default();
    let source=NurbsCurve {degree:2,poles:vec![[4.,0.,0.],[0.,4.,0.],[-4.,0.,0.],[0.,-4.,0.],[4.,0.,0.],[0.,4.,0.]],
        weights:vec![1.;6],knots:(0..=8).map(|v|v as f64).collect(),domain:[2.,6.],periodic:true,reversed:false,seam:Some(3.)};
    let normalized=om9_core::slab::normalize_closed_nurbs_for_slab(&source,&limits).unwrap();
    assert!(!normalized.periodic && normalized.seam.is_none());
    let curve=Curve::Nurbs(normalized);let step=1./4096.;
    let periodic_curve=Curve::Nurbs(source.clone());
    for f in [0.,0.13,0.5,1.] {
        let a=om9_core::curve_ops::point_at(&periodic_curve,f,&limits).unwrap();
        let b=om9_core::curve_ops::point_at(&curve,f,&limits).unwrap();
        assert!((0..3).all(|i|(a[i]-b[i]).abs()<1e-10));
    }
    let t0=om9_core::curve_ops::tangent_at(&periodic_curve,0.,&limits).unwrap();
    let t1=om9_core::curve_ops::tangent_at(&periodic_curve,1.,&limits).unwrap();
    assert!((0..3).all(|i|(t0[i]-t1[i]).abs()<1e-10));
    assert!(om9_core::curve_ops::fraction_derivatives(&periodic_curve,0.,2,&limits).is_err());
    let perimeter=(0..4096).map(|i| {let t=om9_core::curve_ops::derivative_at(&curve,(i as f64+0.5)*step,&limits).unwrap();t[0].hypot(t[1])*step}).sum::<f64>();
    let slab=om9_core::slab::closed_nurbs_slab(&Curve::Nurbs(source.clone()),[0.,0.,1.],0.25,2.,false,1e-4,&limits).unwrap();
    let outside=om9_kernel::extrusion::solid(&slab.outer,slab.direction,0.05,&limits).unwrap();
    let inside=om9_kernel::extrusion::solid(&slab.inner,slab.direction,0.05,&limits).unwrap();
    let body=om9_kernel::editing::boolean(&outside,&inside,om9_kernel::editing::BooleanOp::Difference,0.05,&limits).unwrap();
    let m=om9_kernel::creation::measure(body.body().unwrap(),&limits).unwrap();
    let expected=2.*(0.25*perimeter+std::f64::consts::PI*0.25_f64.powi(2));
    assert!((m.volume_mm3-expected).abs()<0.02,"{m:?} expected {expected}");
    let mut invalid=source.clone();invalid.poles[4][0]+=0.1;
    assert!(om9_core::slab::normalize_closed_nurbs_for_slab(&invalid,&limits).is_err());
    let mut invalid=source.clone();invalid.knots[0]=-0.5;
    assert!(om9_core::slab::normalize_closed_nurbs_for_slab(&invalid,&limits).is_err());
    let mut tiny=source.clone();for k in &mut tiny.knots {*k*=1e-9;}tiny.domain=tiny.domain.map(|u|u*1e-9);tiny.seam=None;
    assert!(tiny.certify_repeated_periodic(&limits).is_ok());
    tiny.knots[1]+=1e-13;
    assert!(tiny.certify_repeated_periodic(&limits).is_err(),"non-periodic tiny knot intervals must not pass an absolute tolerance");
    let mut unresolvable=source.clone();
    for k in &mut unresolvable.knots {*k=1e6+*k*1e-9;}unresolvable.domain=unresolvable.domain.map(|u|1e6+u*1e-9);unresolvable.seam=None;
    assert!(matches!(unresolvable.certify_repeated_periodic(&limits),Err(om9_core::Error::Unsupported("periodic NURBS knot precision is insufficient"))));
    for reversed in [false,true] {
        let mut periodic=source.clone();periodic.reversed=reversed;
        let mut doc=Document::new("Slab certified periodic session");let entity=Entity::new("periodic",Geometry::Curve(Curve::Nurbs(periodic)),None);
        let id=entity.id;doc.entities.insert(id,entity);let mut c=Controller::new(doc,limits).unwrap();c.select_objects(&[id]).unwrap();
        let before=c.document().clone();c.begin_slab_development().unwrap();c.submit("Distance=0.25").unwrap();
        assert!(!c.solid_preview([5.,0.,0.]).is_empty());c.accept_world_point([5.,0.,0.]).unwrap();c.submit("Height=2").unwrap();
        let output=c.document().entities.values().find(|e|e.id!=id).unwrap();let actual=measures(&c,output.id);
        assert!((actual.volume_mm3-expected).abs()<0.02,"{reversed} {actual:?}");c.undo().unwrap();assert_eq!(c.document(),&before);
    }
}

#[test]
fn solid_slab_development_shifted_closed_nurbs_seam() {
    use om9_core::geometry::Curve;
    let limits=Limits::default();let step=std::f64::consts::TAU/10000.;
    let perimeter=(0..10000).map(|i| {let a=(i as f64+0.5)*step;(25.*a.sin().powi(2)+9.*a.cos().powi(2)).sqrt()*step}).sum::<f64>();
    for reversed in [false,true] {
        let mut ellipse=om9_core::curve_build::ellipse([0.;3],[1.,0.,0.],[0.,1.,0.],5.,3.,&limits).unwrap();
        ellipse.reversed=reversed;ellipse.seam=Some(0.17);
        let mut doc=Document::new("Slab shifted seam");let source=Entity::new("ellipse",Geometry::Curve(Curve::Nurbs(ellipse)),None);
        let id=source.id;doc.entities.insert(id,source);let mut c=Controller::new(doc,limits).unwrap();c.select_objects(&[id]).unwrap();
        let before=c.document().clone();c.begin_slab_development().unwrap();let pick=[5.5,0.,0.];
        assert!(!c.solid_preview(pick).is_empty());c.accept_world_point(pick).unwrap();c.accept_world_point([0.,0.,2.]).unwrap();
        let output=c.document().entities.values().find(|e|e.id!=id).unwrap();let m=measures(&c,output.id);
        let expected=2.*(0.5*perimeter+std::f64::consts::PI*0.25);
        assert!((m.volume_mm3-expected).abs()<0.02,"{reversed} {m:?}");c.undo().unwrap();assert_eq!(c.document(),&before);
    }
    let mut open=om9_core::curve_build::ellipse([0.;3],[1.,0.,0.],[0.,1.,0.],5.,3.,&limits).unwrap();
    open.poles[8]=[4.,0.,0.];open.seam=Some(0.17);let curve=Curve::Nurbs(open);
    assert!(om9_core::slab::closed_nurbs_slab(&curve,[0.,0.,1.],0.5,2.,false,1e-4,&limits).is_err());
    let mut doc=Document::new("Slab invalid seam");let source=Entity::new("open",Geometry::Curve(curve),None);
    let id=source.id;doc.entities.insert(id,source);let mut c=Controller::new(doc,limits).unwrap();
    c.select_objects(&[id]).unwrap();let before=c.document().clone();assert!(c.begin_slab_development().is_err());
    assert_eq!(c.document(),&before);assert_eq!(c.coordinator().undo_len(),0);
}

#[test]
fn solid_slab_development_closed_nurbs_suppresses_collapsed_preview() {
    use om9_core::geometry::Curve;
    let limits=Limits::default();
    let ellipse=om9_core::curve_build::ellipse([0.;3],[1.,0.,0.],[0.,1.,0.],5.,3.,&limits).unwrap();
    let mut doc=Document::new("Slab collapsed preview");let source=Entity::new("ellipse",Geometry::Curve(Curve::Nurbs(ellipse)),None);
    let id=source.id;doc.entities.insert(id,source);let mut c=Controller::new(doc,limits).unwrap();
    c.select_objects(&[id]).unwrap();let before=c.document().clone();c.begin_slab_development().unwrap();
    c.submit("Distance=4").unwrap();let pick=[4.5,0.,0.];assert!(c.solid_preview(pick).is_empty());
    assert!(c.accept_world_point(pick).is_err());assert_eq!(c.document(),&before);assert_eq!(c.coordinator().undo_len(),0);
    c.submit("Distance=0.5").unwrap();assert!(!c.solid_preview(pick).is_empty());
    c.cancel();assert_eq!(c.document(),&before);assert_eq!(c.coordinator().undo_len(),0);
}

#[test]
fn solid_slab_development_closed_nurbs_preview_and_commit() {
    use om9_core::geometry::Curve;
    let limits=Limits::default();let step=std::f64::consts::TAU/10000.;
    let perimeter=(0..10000).map(|i| {let a=(i as f64+0.5)*step;(25.*a.sin().powi(2)+9.*a.cos().powi(2)).sqrt()*step}).sum::<f64>();
    for reversed in [false,true] {for (pick,both,sign) in [([5.5,0.,0.],false,1.),([4.5,0.,0.],true,-1.)] {
        let mut ellipse=om9_core::curve_build::ellipse([0.;3],[1.,0.,0.],[0.,1.,0.],5.,3.,&limits).unwrap();ellipse.reversed=reversed;
        let mut doc=Document::new("Slab closed NURBS session");let source=Entity::new("ellipse",Geometry::Curve(Curve::Nurbs(ellipse)),None);
        let id=source.id;doc.entities.insert(id,source);let mut c=Controller::new(doc,limits).unwrap();c.select_objects(&[id]).unwrap();
        let before=c.document().clone();c.begin_slab_development().unwrap();if both {c.submit("BothSides=Yes").unwrap();}
        let flat=c.solid_preview(pick);assert!(!flat.is_empty());
        assert!(!flat.iter().any(|edge|(0..3).map(|i|(edge[0][i]-edge[1][i]).powi(2)).sum::<f64>().sqrt()>1.),"preview rings must not be joined");
        c.accept_world_point(pick).unwrap();let preview=c.solid_preview([0.,0.,2.]);assert!(!preview.is_empty());
        c.accept_world_point([0.,0.,2.]).unwrap();let result=c.document().entities.values().find(|e|e.id!=id).unwrap();let m=measures(&c,result.id);
        let expected=2.*if both {2.} else {1.}*(0.5*perimeter+sign*std::f64::consts::PI*0.25);
        assert!((m.volume_mm3-expected).abs()<0.02,"{reversed} {m:?}");c.undo().unwrap();assert_eq!(c.document(),&before);
    }}
}

#[test]
fn solid_slab_internal_closed_nurbs_publication_is_atomic() {
    use om9_core::geometry::{Curve,Transform};
    let limits=Limits::default();let h=std::f64::consts::TAU/10000.;
    let perimeter=(0..10000).map(|i| {let a=(i as f64+0.5)*h;(25.*a.sin().powi(2)+9.*a.cos().powi(2)).sqrt()*h}).sum::<f64>();
    for reversed in [false,true] {for (offset,both) in [(0.5,false),(-0.5,true)] {
        let mut ellipse=om9_core::curve_build::ellipse([0.;3],[1.,0.,0.],[0.,1.,0.],5.,3.,&limits).unwrap();ellipse.reversed=reversed;
        let mut doc=Document::new("Slab closed NURBS publication");let mut source=Entity::new("ellipse",Geometry::Curve(Curve::Nurbs(ellipse)),None);
        source.placement=Transform::translation([10.,20.,5.]);let id=source.id;doc.entities.insert(id,source);
        let mut c=Controller::new(doc,limits).unwrap();let before=c.document().clone();let stamp=c.document().stamp(id).unwrap();
        let effective=if reversed {-offset} else {offset};
        c.solid_slab_commit_polyline(stamp,[0.,0.,1.],effective,-2.,both).unwrap();
        let output=c.document().entities.values().find(|e|e.id!=id).unwrap();let m=measures(&c,output.id);
        let expected=2.*if both {2.} else {1.}*(0.5*perimeter+offset.signum()*std::f64::consts::PI*0.25);
        assert!(m.closed && m.solids==1);assert!((m.volume_mm3-expected).abs()<0.02,"{m:?}");
        let shift=output.placement.point([0.;3]).unwrap();assert!((shift[2]-if both {2.} else {0.}).abs()<1e-9);
        assert_eq!(c.coordinator().undo_len(),1);c.undo().unwrap();assert_eq!(c.document(),&before);
        c.redo().unwrap();assert_eq!(c.document().entities.len(),2);c.undo().unwrap();
        assert!(c.solid_slab_commit_polyline(stamp,[0.,0.,1.],if reversed {4.} else {-4.},2.,false).is_err());
        assert_eq!(c.document(),&before);assert_eq!(c.coordinator().undo_len(),0);
    }}
}

#[test]
fn solid_slab_closed_ellipse_nurbs_foundation() {
    use om9_core::geometry::Curve;
    let limits=Limits::default();
    let ellipse=Curve::Nurbs(om9_core::curve_build::ellipse([0.;3],[1.,0.,0.],[0.,1.,0.],5.,3.,&limits).unwrap());
    let h=std::f64::consts::TAU/10000.;
    let perimeter=(0..10000).map(|i| {let a=(i as f64+0.5)*h;(25.*a.sin().powi(2)+9.*a.cos().powi(2)).sqrt()*h}).sum::<f64>();
    for reversed in [false,true] {for (offset,both) in [(0.5,false),(-0.5,true)] {
        let curve=if reversed {om9_core::curve_ops::reverse_curve(&ellipse,&limits).unwrap()} else {ellipse.clone()};
        let slab=om9_core::slab::closed_nurbs_slab(&curve,[0.,0.,1.],if reversed {-offset} else {offset},2.,both,1e-4,&limits).unwrap();
        let outside=om9_kernel::extrusion::solid(&slab.outer,slab.direction,0.05,&limits).unwrap();
        let inside=om9_kernel::extrusion::solid(&slab.inner,slab.direction,0.05,&limits).unwrap();
        let outcome=om9_kernel::editing::boolean(&outside,&inside,om9_kernel::editing::BooleanOp::Difference,0.05,&limits).unwrap();
        let body=outcome.body().unwrap();let m=om9_kernel::creation::measure(body,&limits).unwrap();
        let expected=2.*if both {2.} else {1.}*(0.5*perimeter+offset.signum()*std::f64::consts::PI*0.25);
        assert!((m.volume_mm3-expected).abs()<0.02,"{reversed} {offset} {m:?} expected {expected}");
    }}
    assert!(om9_core::slab::closed_nurbs_slab(&ellipse,[0.,1.,0.],0.5,2.,false,1e-4,&limits).is_err());
    assert!(om9_core::slab::closed_nurbs_slab(&ellipse,[0.,0.,1.],-4.,2.,false,1e-4,&limits).is_err());
}

#[test]
fn solid_slab_development_concave_closed_options_and_preview() {
    for (pick,through,expected) in [([2.,-2.,0.],false,18.),([2.,0.25,0.],true,7.5)] {
        let points=[[0.,0.,0.],[4.,0.,0.],[4.,1.,0.],[1.,1.,0.],[1.,4.,0.],[0.,4.,0.],[0.,0.,0.]];
        let mut doc=Document::new("Slab closed concave options");
        let source=Entity::new("rail",Geometry::Curve(om9_core::curve_build::polyline(&points).unwrap()),None);
        let id=source.id;doc.entities.insert(id,source);let mut c=Controller::new(doc,Limits::default()).unwrap();
        c.select_objects(&[id]).unwrap();let before=c.document().clone();c.begin_slab_development().unwrap();
        c.submit("Distance=0.5").unwrap();if through {c.submit("ThroughPoint").unwrap();}
        assert!(!c.solid_preview(pick).is_empty());c.accept_world_point(pick).unwrap();
        c.submit("Undo").unwrap();assert_eq!(c.document(),&before);assert_eq!(c.coordinator().undo_len(),0);
        c.accept_world_point(pick).unwrap();let preview=c.solid_preview([0.,0.,2.]);assert!(!preview.is_empty());
        let mut low=[f64::INFINITY;3];let mut high=[f64::NEG_INFINITY;3];
        for edge in &preview {for p in edge {for i in 0..3 {low[i]=low[i].min(p[i]);high[i]=high[i].max(p[i]);}}}
        c.submit("Height=2").unwrap();let output=c.document().entities.values().find(|e|e.id!=id).unwrap();let m=measures(&c,output.id);
        assert!((m.volume_mm3-expected).abs()<1e-6,"{through} {m:?}");
        for i in 0..3 {assert!((m.bounds_mm[i]-low[i]).abs()<1e-6 && (m.bounds_mm[i+3]-high[i]).abs()<1e-6);}
        c.undo().unwrap();assert_eq!(c.document(),&before);
        c.select_objects(&[id]).unwrap();c.begin_slab_development().unwrap();c.accept_world_point(pick).unwrap();c.cancel();
        assert_eq!(c.document(),&before);assert_eq!(c.coordinator().undo_len(),0);
    }
}

#[test]
fn solid_slab_development_closed_polyline_preview_and_commit() {
    use om9_core::geometry::{Curve,NurbsCurve};
    for (pick,both,expected) in [([2.,-1.,0.],false,36.),([2.,0.5,0.],true,24.)] {
        for reversed in [false,true] {
            for nurbs in [false,true] {
                let points=vec![[0.,0.,0.],[4.,0.,0.],[4.,3.,0.],[0.,3.,0.],[0.,0.,0.]];
                let curve=if nurbs {Curve::Nurbs(NurbsCurve {degree:1,poles:points.clone(),weights:vec![1.,2.,1.,3.,1.],
                    knots:vec![0.,0.,0.25,0.5,0.75,1.,1.],domain:[0.,1.],periodic:false,reversed,seam:None})}
                    else {let mut p=points.clone();if reversed {p.reverse();}om9_core::curve_build::polyline(&p).unwrap()};
                let mut doc=Document::new("Slab closed session");let source=Entity::new("rail",Geometry::Curve(curve),None);
                let id=source.id;doc.entities.insert(id,source);let mut c=Controller::new(doc,Limits::default()).unwrap();
                c.select_objects(&[id]).unwrap();let before=c.document().clone();c.begin_slab_development().unwrap();
                if both {c.submit("BothSides=Yes").unwrap();}
                assert_eq!(c.solid_preview(pick).len(),8);c.accept_world_point(pick).unwrap();
                assert_eq!(c.solid_preview([0.,0.,2.]).len(),24);c.accept_world_point([0.,0.,2.]).unwrap();
                let output=c.document().entities.values().find(|e|e.id!=id).unwrap();let m=measures(&c,output.id);
                assert!((m.volume_mm3-expected).abs()<1e-6,"{m:?}");c.undo().unwrap();assert_eq!(c.document(),&before);
            }
        }
    }
}

#[test]
fn solid_slab_internal_closed_polyline_publication_is_atomic() {
    use om9_core::geometry::{Curve,NurbsCurve,Transform};
    for reversed in [false,true] {
        for nurbs in [false,true] {
            let pts=vec![[0.,0.,0.],[4.,0.,0.],[4.,3.,0.],[0.,3.,0.],[0.,0.,0.]];
            let curve=if nurbs {Curve::Nurbs(NurbsCurve {degree:1,poles:pts.clone(),weights:vec![1.,2.,1.,3.,1.],
                knots:vec![0.,0.,0.25,0.5,0.75,1.,1.],domain:[0.,1.],periodic:false,reversed,seam:None})}
                else {let mut p=pts.clone();if reversed {p.reverse();}om9_core::curve_build::polyline(&p).unwrap()};
            let mut doc=Document::new("Slab closed publication");let mut source=Entity::new("rail",Geometry::Curve(curve),None);
            source.placement=Transform::translation([10.,20.,5.]);let id=source.id;doc.entities.insert(id,source);
            let mut c=Controller::new(doc,Limits::default()).unwrap();let before=c.document().clone();let stamp=c.document().stamp(id).unwrap();
            let offset=if reversed {-1.} else {1.};
            c.solid_slab_commit_polyline(stamp,[0.,0.,1.],offset,2.,false).unwrap();
            let result=c.document().entities.values().find(|e|e.id!=id).unwrap();let m=measures(&c,result.id);
            assert!(m.closed && m.solids==1);assert!((m.volume_mm3-36.).abs()<1e-6,"{m:?}");
            assert!((m.bounds_mm[0]-9.).abs()<1e-6 && (m.bounds_mm[2]-5.).abs()<1e-6);
            assert_eq!(c.coordinator().undo_len(),1);c.undo().unwrap();assert_eq!(c.document(),&before);
            c.redo().unwrap();assert_eq!(c.document().entities.len(),2);c.undo().unwrap();
            assert!(c.solid_slab_commit_polyline(stamp,[0.,0.,1.],-100.*offset,2.,false).is_err());
            assert_eq!(c.document(),&before);assert_eq!(c.coordinator().undo_len(),0);
        }
    }
}

#[test]
fn solid_slab_closed_polyline_foundation_measures_nested_rings() {
    use om9_kernel::{creation::{self, Profile, DEFAULT_DEFLECTION_MM},editing::{self,BooleanOp,BooleanOutcome}};
    let limits=Limits::default();
    let square=vec![[0.,0.,0.],[4.,0.,0.],[4.,3.,0.],[0.,3.,0.],[0.,0.,0.]];
    let concave=vec![[0.,0.,0.],[4.,0.,0.],[4.,1.,0.],[1.,1.,0.],[1.,4.,0.],[0.,4.,0.],[0.,0.,0.]];
    for (points,offset,height,both,expected) in [(square.clone(),1.,2.,false,36.),(square.clone(),-0.5,2.,false,12.),(square,-0.5,2.,true,24.),(concave,0.5,2.,false,18.)] {
        for reversed in [false,true] {
            let mut input=points.clone();if reversed {input.reverse();}
            let (outer,inner,direction)=om9_core::slab::closed_polyline_slab(&input,[0.,0.,1.],if reversed {-offset} else {offset},height,both,&limits).unwrap();
            let outside=creation::extrude(&Profile::new(outer).unwrap(),direction,true,DEFAULT_DEFLECTION_MM,&limits).unwrap();
            let inside=creation::extrude(&Profile::new(inner).unwrap(),direction,true,DEFAULT_DEFLECTION_MM,&limits).unwrap();
            let BooleanOutcome::Body(body)=editing::boolean(&outside,&inside,BooleanOp::Difference,DEFAULT_DEFLECTION_MM,&limits).unwrap() else {panic!("empty Slab")};
            let m=creation::measure(&body,&limits).unwrap();assert!(m.closed && m.solids==1);
            assert!((m.volume_mm3-expected).abs()<1e-6,"{offset} {reversed} {m:?}");
        }
    }
    let square=[[0.,0.,0.],[4.,0.,0.],[4.,3.,0.],[0.,3.,0.],[0.,0.,0.]];
    assert!(om9_core::slab::closed_polyline_slab(&square,[0.,0.,1.],-2.,2.,false,&limits).is_err());
    let equal_square=[[0.,0.,0.],[4.,0.,0.],[4.,4.,0.],[0.,4.,0.],[0.,0.,0.]];
    assert!(om9_core::slab::closed_polyline_slab(&equal_square,[0.,0.,1.],-3.,2.,false,&limits).is_err());
    let vertical=square.map(|p|[5.,p[0],p[1]]);
    let (outside,inside,_)=om9_core::slab::closed_polyline_slab(&vertical,[1.,0.,0.],1.,2.,false,&limits).unwrap();
    assert!(outside.iter().any(|p|p[1] < -0.9));assert!(inside.iter().all(|p|p[1]>=0.));
    let crossing=[[0.,0.,0.],[4.,4.,0.],[0.,4.,0.],[4.,0.,0.],[0.,0.,0.]];
    assert!(om9_core::slab::closed_polyline_slab(&crossing,[0.,0.,1.],0.5,2.,false,&limits).is_err());
}

#[test]
fn solid_slab_polyline_accepts_disjoint_collinear_segments() {
    use om9_kernel::creation::{self, Profile, DEFAULT_DEFLECTION_MM};
    let limits=Limits::default();
    let points=[[0.,0.,0.],[4.,0.,0.],[4.,2.,0.],[8.,2.,0.],[8.,0.,0.],[12.,0.,0.]];
    for vertical in [false,true] {
        let placed=points.map(|p|if vertical {[5.,p[0],p[1]]} else {p});
        let normal=if vertical {[1.,0.,0.]} else {[0.,0.,1.]};
        let (boundary,direction)=om9_core::slab::polyline_slab(&placed,normal,0.5,3.,false,&limits).unwrap();
        let body=creation::extrude(&Profile::new(boundary).unwrap(),direction,true,DEFAULT_DEFLECTION_MM,&limits).unwrap();
        let measured=creation::measure(&body,&limits).unwrap();
        assert!(measured.closed && measured.solids==1);
        assert!((measured.volume_mm3-24.).abs()<1e-7,"{measured:?}");
    }
    let overlap=[[0.,0.,0.],[4.,0.,0.],[4.,2.,0.],[2.,2.,0.],[2.,0.,0.],[6.,0.,0.]];
    assert!(om9_core::slab::polyline_slab(&overlap,[0.,0.,1.],0.5,3.,false,&limits).is_err());
}

#[test]
fn solid_slab_polyline_foundation_miters_and_rejects_invalid_boundaries() {
    use om9_core::slab::polyline_slab;
    use om9_kernel::creation::{self, Profile, DEFAULT_DEFLECTION_MM};
    let limits = Limits::default();
    let points = [[0.,0.,0.],[4.,0.,0.],[4.,3.,0.]];
    for (offset, expected_area) in [(1.,8.), (-1.,6.)] {
        let (boundary, extrusion) = polyline_slab(&points,[0.,0.,1.],offset,3.,false,&limits).unwrap();
        let profile = Profile::new(boundary).unwrap();
        let body = creation::extrude(&profile,extrusion,true,DEFAULT_DEFLECTION_MM,&limits).unwrap();
        let m = creation::measure(&body,&limits).unwrap();
        assert_eq!(m.solids,1);
        assert!((m.volume_mm3-expected_area*3.).abs()<1e-7,"{offset} {m:?}");
    }
    assert!(polyline_slab(&[[0.,0.,0.],[1.,0.,0.],[0.,0.,0.]],[0.,0.,1.],1.,1.,false,&limits).is_err());
    assert!(polyline_slab(&[[0.,0.,0.],[1.,0.,0.],[1.,1.,1.]],[0.,0.,1.],1.,1.,false,&limits).is_err());
    assert!(polyline_slab(&[[0.,0.,0.],[4.,4.,0.],[0.,4.,0.],[4.,0.,0.]],[0.,0.,1.],0.5,1.,false,&limits).is_err());
    let (both, direction) = polyline_slab(&points,[0.,0.,1.],1.,3.,true,&limits).unwrap();
    assert!(both.iter().all(|p| (p[2]+3.).abs() < 1e-9));
    assert_eq!(direction,[0.,0.,6.]);
    let restricted = Limits { max_bytes: 1, ..limits };
    assert!(polyline_slab(&points,[0.,0.,1.],1.,1.,false,&restricted).is_err());
}

#[test]
fn solid_slab_line_foundation_builds_measured_solid_without_enabling_command() {
    use om9_core::slab::line_slab;
    use om9_kernel::creation::{self, DEFAULT_DEFLECTION_MM, Profile};
    let limits = Limits::default();
    for (offset, height, both) in [(2.,3.,false),(-2.,3.,false),(2.,-3.,false),(2.,3.,true)] {
        let slab = line_slab([10.,20.,30.], [14.,20.,30.], [0.,0.,1.], offset, height, both).unwrap();
        let profile = Profile::new(slab.perimeter.to_vec()).unwrap();
        let body = creation::extrude(&profile, slab.extrusion, true, DEFAULT_DEFLECTION_MM, &limits).unwrap();
        let m = creation::measure(&body, &limits).unwrap();
        assert_eq!(m.solids, 1);
        assert!((m.volume_mm3 - if both {48.} else {24.}).abs() < 1e-7, "{m:?}");
        assert_eq!(slab.perimeter[0][2], if both {27.} else {30.});
    }
    assert!(line_slab([0.;3],[0.;3],[0.,0.,1.],1.,1.,false).is_err());
    assert!(line_slab([0.;3],[1.,0.,1.],[0.,0.,1.],1.,1.,false).is_err());
    assert!(line_slab([0.;3],[1.,0.,0.],[0.,0.,1.],0.,1.,false).is_err());
    let feature = om9_app::icons::feature_for("SolidSlab").expect("Slab icon binding");
    assert!(om9_app::session::Controller::availability(feature).is_err());
}

#[test]
fn solid_primitives_are_menu_bound_and_closed() {
    let mut c = controller();
    let cases: &[(&str, f64)] = &[
        ("Box 10 20 30", 6000.0),
        ("Sphere 5", 523.598_775_598_298_8),
        ("Ellipsoid 2 3 4", 4.0 * std::f64::consts::PI / 3.0 * 24.0),
        ("Torus 5 1", 98.696_044_010_364_66),
        ("Cylinder 2 10", 125.663_706_143_591_86),
        ("Cone 3 6", 56.548_667_766_616_34),
        ("Tube 5 3 10", 502.654_824_574_367_4),
    ];
    for (command, expected) in cases {
        let before = c.document().entities.keys().copied().collect();
        c.submit(command).unwrap();
        let id = new_id(&c, &before);
        let measured = measures(&c, id);
        assert_eq!(measured.solids, 1, "{command}");
        assert!(measured.closed, "{command}");
        assert!(
            (measured.volume_mm3 - expected).abs() < expected.abs().max(1.0) * 1e-10,
            "{command}: {} vs {expected}",
            measured.volume_mm3
        );
        println!(
            "solid_{}_volume={:.6}",
            command.split_whitespace().next().unwrap(),
            measured.volume_mm3
        );
    }
}

#[test]
fn solid_ellipsoid_menu_cmd_undo_and_archive_roundtrip() {
    use om9_app::session::InvocationOrigin;
    use om9_archive::Archive;
    use om9_core::transaction::Cancellation;
    let expected = 4.0 * std::f64::consts::PI / 3.0 * 24.0;
    let mut c = controller();
    c.dispatch("OM9-SOLID-013", InvocationOrigin::Menu).unwrap();
    assert!(c.document().entities.is_empty());
    c.cancel();
    assert_eq!(c.coordinator().undo_len(), 0);
    for command in ["Ellipsoid 0 3 4", "Ellipsoid 2 -3 4", "Ellipsoid 2 3 NaN"] {
        assert!(c.submit(command).is_err());
        assert!(c.document().entities.is_empty());
        assert_eq!(c.coordinator().undo_len(), 0);
    }
    c.dispatch("OM9-SOLID-013", InvocationOrigin::Menu).unwrap();
    c.submit("2 3 4").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    assert_eq!(c.coordinator().undo_len(), 1);
    let source = match &c.document().entities[&id].geometry {
        Geometry::Brep(brep) => brep.source().to_owned(),
        _ => panic!("ellipsoid must be a B-Rep"),
    };
    assert!((measures(&c, id).volume_mm3 / expected - 1.0).abs() < 1e-10);
    c.undo().unwrap();
    assert!(c.document().entities.is_empty());
    c.redo().unwrap();
    assert!((measures(&c, id).volume_mm3 / expected - 1.0).abs() < 1e-10);
    let path = std::env::temp_dir().join(format!("om9-ellipsoid-{}.om9", uuid::Uuid::new_v4()));
    Archive::save(
        &path,
        c.document(),
        c.coordinator().limits(),
        &Cancellation::default(),
    )
    .unwrap();
    let restored =
        Archive::open_restored(&path, c.coordinator().limits(), &Default::default()).unwrap();
    std::fs::remove_file(&path).unwrap();
    let Geometry::Brep(brep) = &restored.document.entities[&id].geometry else {
        panic!("stored B-Rep");
    };
    assert_eq!(brep.source(), source);
    assert!(
        (creation::measure(brep, c.coordinator().limits())
            .unwrap()
            .volume_mm3
            / expected
            - 1.0)
            .abs()
            < 1e-10
    );
    c.submit("Ellipsoid 2 3 4").unwrap();
    assert_eq!(c.document().entities.len(), 2);
    println!("ellipsoid_menu_cmd_undo_redo_om9=PASS");
}

#[test]
fn solid_ellipsoid_center_axes_menu_and_cmd() {
    use om9_app::session::InvocationOrigin;
    for menu in [true, false] {
        let mut c = controller();
        if menu {
            c.dispatch("OM9-SOLID-013", InvocationOrigin::Menu).unwrap();
        } else {
            c.submit("Ellipsoid").unwrap();
        }
        assert!(c.prompt().starts_with("Center"));
        c.submit("10,20,30").unwrap();
        c.accept_world_point([10.0, 22.0, 30.0]).unwrap();
        assert!(c.accept_world_point([10.0, 24.0, 30.0]).is_err());
        assert!(c.prompt().starts_with("End of second"));
        assert_eq!(c.coordinator().undo_len(), 0);
        c.accept_world_point([7.0, 20.0, 30.0]).unwrap();
        c.submit("Undo").unwrap();
        assert!(c.prompt().starts_with("End of second"));
        c.submit("3").unwrap();
        assert!(c.accept_world_point([10.0, 20.0, 30.0]).is_err());
        c.submit("4").unwrap();
        let id = *c.document().entities.keys().next().unwrap();
        let entity = &c.document().entities[&id];
        let transform = entity.placement;
        for (local, expected) in [
            ([0.0, 0.0, 0.0], [10.0, 20.0, 30.0]),
            ([2.0, 0.0, 0.0], [10.0, 22.0, 30.0]),
            ([0.0, 3.0, 0.0], [7.0, 20.0, 30.0]),
            ([0.0, 0.0, 4.0], [10.0, 20.0, 34.0]),
        ] {
            let actual = transform.point(local).unwrap();
            assert!((0..3).all(|i| (actual[i] - expected[i]).abs() < 1e-9));
        }
        assert!((measures(&c, id).volume_mm3 - 32.0 * std::f64::consts::PI).abs() < 1e-7);
        assert_eq!(c.coordinator().undo_len(), 1);
        c.undo().unwrap();
        assert!(c.document().entities.is_empty());
    }
}

#[test]
fn solid_ellipsoid_captures_tilted_construction_plane() {
    let mut c = controller();
    c.set_cplane_from_three_points([0., 0., 0.], [0., 1., 0.], [0., 0., 1.])
        .unwrap();
    c.submit("Ellipsoid").unwrap();
    c.accept_world_point([5., 6., 7.]).unwrap();
    assert!(
        c.set_cplane_from_three_points([0., 0., 0.], [1., 0., 0.], [0., 1., 0.])
            .is_err()
    );
    c.accept_world_point([5., 8., 7.]).unwrap();
    c.accept_world_point([5., 6., 10.]).unwrap();
    c.accept_world_point([9., 6., 7.]).unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    let t = c.document().entities[&id].placement;
    for (local, expected) in [
        ([2., 0., 0.], [5., 8., 7.]),
        ([0., 3., 0.], [5., 6., 10.]),
        ([0., 0., 4.], [9., 6., 7.]),
    ] {
        let actual = t.point(local).unwrap();
        assert!((0..3).all(|i| (actual[i] - expected[i]).abs() < 1e-9));
    }
}

#[test]
fn solid_ellipsoid_diameter_from_menu_and_cmd() {
    use om9_app::session::InvocationOrigin;
    for route in 0..3 {
        let mut c = controller();
        match route {
            0 => {
                c.dispatch("EllipsoidDiameter", InvocationOrigin::Menu)
                    .unwrap();
            }
            1 => {
                c.submit("Ellipsoid").unwrap();
                assert!(
                    c.solid_option_tokens()
                        .iter()
                        .any(|(label, _)| *label == "Diameter")
                );
                c.submit("Diameter").unwrap();
            }
            _ => {
                c.submit("_Ellipsoid _D").unwrap();
            }
        }
        assert!(c.prompt().starts_with("First end"));
        c.accept_world_point([8., 20., 30.]).unwrap();
        assert!(c.accept_world_point([8., 20., 32.]).is_err());
        assert!(c.prompt().starts_with("Other end"));
        c.accept_world_point([12., 20., 32.]).unwrap();
        c.submit("Undo").unwrap();
        assert!(c.prompt().starts_with("Other end"));
        c.accept_world_point([12., 20., 30.]).unwrap();
        assert!(c.accept_world_point([12., 20., 30.]).is_err());
        c.accept_world_point([10., 23., 30.]).unwrap();
        c.accept_world_point([10., 20., 34.]).unwrap();
        let id = *c.document().entities.keys().next().unwrap();
        let t = c.document().entities[&id].placement;
        assert_eq!(t.point([0., 0., 0.]).unwrap(), [10., 20., 30.]);
        assert_eq!(t.point([2., 0., 0.]).unwrap(), [12., 20., 30.]);
        assert!((measures(&c, id).volume_mm3 - 32. * std::f64::consts::PI).abs() < 1e-7);
        assert_eq!(c.coordinator().undo_len(), 1);
        c.undo().unwrap();
        assert!(c.document().entities.is_empty());
    }
}

#[test]
fn solid_ellipsoid_diameter_rotated_first_axis() {
    let mut c = controller();
    c.submit("Ellipsoid Diameter").unwrap();
    for p in [
        [10., 18., 30.],
        [10., 22., 30.],
        [7., 20., 30.],
        [10., 20., 34.],
    ] {
        c.accept_world_point(p).unwrap();
    }
    let id = *c.document().entities.keys().next().unwrap();
    let t = c.document().entities[&id].placement;
    assert_eq!(t.point([2., 0., 0.]).unwrap(), [10., 22., 30.]);
    assert_eq!(t.point([0., 3., 0.]).unwrap(), [7., 20., 30.]);
    assert!((measures(&c, id).volume_mm3 - 32. * std::f64::consts::PI).abs() < 1e-7);
}

#[test]
fn solid_ellipsoid_from_foci_geometry_and_atomic_markers() {
    use om9_app::session::InvocationOrigin;
    for route in 0..3 {
        let mut c = controller();
        match route {
            0 => {
                c.dispatch("EllipsoidFromFoci", InvocationOrigin::Menu)
                    .unwrap();
            }
            1 => {
                c.submit("Ellipsoid").unwrap();
                c.submit("FromFoci").unwrap();
            }
            _ => {
                c.submit("_Ellipsoid _FromFoci").unwrap();
            }
        }
        if route != 0 {
            c.submit("MarkFoci=Yes").unwrap();
        }
        c.accept_world_point([7., 20., 30.]).unwrap();
        assert!(c.accept_world_point([7., 20., 31.]).is_err());
        assert_eq!(c.prompt(), "Second focus");
        c.accept_world_point([13., 20., 30.]).unwrap();
        assert!(c.accept_world_point([10., 20., 30.]).is_err());
        assert_eq!(c.prompt(), "Point on base ellipse");
        c.accept_world_point([10., 24., 30.]).unwrap();
        c.submit("Undo").unwrap();
        assert_eq!(c.prompt(), "Point on base ellipse");
        c.accept_world_point([10., 24., 30.]).unwrap();
        c.submit("2").unwrap();
        assert_eq!(c.document().entities.len(), if route == 0 { 1 } else { 3 });
        let entity = c
            .document()
            .entities
            .values()
            .find(|e| matches!(e.geometry, Geometry::Brep(_)))
            .unwrap();
        let id = entity.id;
        assert_eq!(
            entity.placement.point([5., 0., 0.]).unwrap(),
            [15., 20., 30.]
        );
        assert_eq!(
            entity.placement.point([0., 4., 0.]).unwrap(),
            [10., 24., 30.]
        );
        assert!((measures(&c, id).volume_mm3 - 160. * std::f64::consts::PI / 3.).abs() < 1e-7);
        if route != 0 {
            let points: Vec<_> = c
                .document()
                .entities
                .values()
                .filter_map(|e| {
                    if let Geometry::Point(p) = e.geometry {
                        Some(p)
                    } else {
                        None
                    }
                })
                .collect();
            assert!(points.contains(&[7., 20., 30.]));
            assert!(points.contains(&[13., 20., 30.]));
        }
        assert_eq!(c.coordinator().undo_len(), 1);
        c.undo().unwrap();
        assert!(c.document().entities.is_empty());
        c.redo().unwrap();
        assert_eq!(c.document().entities.len(), if route == 0 { 1 } else { 3 });
    }
}

#[test]
fn solid_ellipsoid_from_foci_marker_limit_retries_without_partial_commit() {
    let mut limits = Limits::default();
    limits.max_outputs = 1;
    let mut c = Controller::new(Document::new("foci output limit"), limits).unwrap();
    c.submit("Ellipsoid FromFoci").unwrap();
    c.submit("MarkFoci=Yes").unwrap();
    for p in [[-3., 0., 0.], [3., 0., 0.], [0., 4., 0.]] {
        c.accept_world_point(p).unwrap();
    }
    assert!(c.accept_world_point([0., 0., 2.]).is_err());
    assert!(c.document().entities.is_empty());
    assert_eq!(c.coordinator().undo_len(), 0);
    assert_eq!(c.prompt(), "End of third axis");
    c.submit("MarkFoci=No").unwrap();
    c.accept_world_point([0., 0., 2.]).unwrap();
    assert_eq!(c.document().entities.len(), 1);
}

#[test]
fn solid_ellipsoid_corner_validates_box_and_commits_oriented_axes() {
    for reverse in [false, true] {
        let mut c = controller();
        c.set_cplane_from_three_points([0., 0., 0.], [0., 1., 0.], [0., 0., 1.])
            .unwrap();
        c.submit("Ellipsoid Corner").unwrap();
        let a = [5., 8., 12.];
        let b = [5., 12., 18.];
        let (first, second) = if reverse { (b, a) } else { (a, b) };
        c.accept_world_point(first).unwrap();
        assert!(c.accept_world_point([5., first[1], second[2]]).is_err());
        assert_eq!(c.prompt(), "Opposite corner of ellipse box");
        c.accept_world_point(second).unwrap();
        c.submit("Undo").unwrap();
        assert_eq!(c.prompt(), "Opposite corner of ellipse box");
        c.accept_world_point(second).unwrap();
        assert!(c.accept_world_point([5., 10., 15.]).is_err());
        c.submit("4").unwrap();
        let entity = c.document().entities.values().next().unwrap();
        let id = entity.id;
        assert_eq!(
            entity.placement.point([0., 0., 0.]).unwrap(),
            [5., 10., 15.]
        );
        assert_eq!(
            entity.placement.point([2., 0., 0.]).unwrap(),
            [5., 12., 15.]
        );
        assert_eq!(
            entity.placement.point([0., 3., 0.]).unwrap(),
            [5., 10., 18.]
        );
        assert_eq!(
            entity.placement.point([0., 0., 4.]).unwrap(),
            [9., 10., 15.]
        );
        assert!((measures(&c, id).volume_mm3 - 32. * std::f64::consts::PI).abs() < 1e-7);
        assert_eq!(c.coordinator().undo_len(), 1);
        c.undo().unwrap();
        assert!(c.document().entities.is_empty());
    }
}

#[test]
fn solid_ellipsoid_around_curve_uses_placed_tangent_and_retains_source() {
    use om9_core::geometry::{Curve, Transform};
    for typed in [false, true] {
        let mut rail = Entity::new(
            "rail",
            Geometry::Curve(Curve::Line {
                start: [0., 0., 0.],
                end: [10., 0., 0.],
                domain: [0., 1.],
            }),
            None,
        );
        rail.placement = Transform::translation([10., 20., 4.]);
        let id = rail.id;
        let mut doc = Document::new("ellipsoid around curve");
        doc.entities.insert(id, rail);
        let mut c = Controller::new(doc, Limits::default()).unwrap();
        let before = c.document().clone();
        c.submit("Ellipsoid AroundCurve").unwrap();
        assert!(c.solid_pipe_needs_curve());
        c.solid_pipe_pick_curve(id).unwrap();
        if typed {
            c.submit("OnCurve=0.5").unwrap();
        } else {
            c.accept_world_point([15., 21., 4.]).unwrap();
        }
        assert!(c.prompt().starts_with("End of first"));
        c.submit("2").unwrap();
        c.submit("3").unwrap();
        c.submit("4").unwrap();
        let entity = c.document().entities.values().find(|e| e.id != id).unwrap();
        let transform = entity.placement;
        assert_eq!(transform.point([0., 0., 0.]).unwrap(), [15., 20., 4.]);
        assert_eq!(transform.point([0., 0., 4.]).unwrap(), [19., 20., 4.]);
        assert!((measures(&c, entity.id).volume_mm3 - 32. * std::f64::consts::PI).abs() < 1e-7);
        c.undo().unwrap();
        assert_eq!(c.document(), &before);
    }
}

#[test]
fn solid_ellipsoid_around_curve_rejects_deleted_source_without_mutation() {
    use om9_core::geometry::Curve;
    let rail = Entity::new(
        "rail",
        Geometry::Curve(Curve::Line {
            start: [0., 0., 0.],
            end: [10., 0., 0.],
            domain: [0., 1.],
        }),
        None,
    );
    let id = rail.id;
    let mut doc = Document::new("stale ellipsoid rail");
    doc.entities.insert(id, rail);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("Ellipsoid AroundCurve").unwrap();
    c.submit("OnCurve=0.5").unwrap();
    c.submit("2").unwrap();
    c.delete_objects(&[id]).unwrap();
    let after = c.document().clone();
    let count = c.coordinator().undo_len();
    assert!(c.submit("3").is_err());
    assert_eq!(c.document(), &after);
    assert_eq!(c.coordinator().undo_len(), count);
}

#[test]
fn solid_ellipsoid_around_circle_tracks_tangent() {
    use om9_core::geometry::Transform;
    let curve =
        om9_core::curve_build::circle_on_frame([0., 0., 0.], [1., 0., 0.], [0., 1., 0.], 5.)
            .unwrap();
    let mut rail = Entity::new("circle", Geometry::Curve(curve), None);
    rail.placement = Transform::translation([10., 20., 4.]);
    let id = rail.id;
    let mut doc = Document::new("ellipsoid around circle");
    doc.entities.insert(id, rail);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("Ellipsoid AroundCurve").unwrap();
    c.submit("OnCurve=0.25").unwrap();
    c.submit("2").unwrap();
    c.submit("3").unwrap();
    c.submit("4").unwrap();
    let entity = c.document().entities.values().find(|e| e.id != id).unwrap();
    let a = entity.placement.point([0., 0., 0.]).unwrap();
    let b = entity.placement.point([0., 0., 4.]).unwrap();
    assert!((a[0] - 10.).abs() < 1e-7 && (a[1] - 25.).abs() < 1e-7 && (a[2] - 4.).abs() < 1e-7);
    assert!(
        (b[0] - a[0] + 4.).abs() < 1e-7 && (b[1] - a[1]).abs() < 1e-7 && (b[2] - a[2]).abs() < 1e-7
    );
}

#[test]
fn solid_ellipsoid_wire_preview_matches_commit_for_four_modes() {
    for mode in ["Center", "Diameter", "Corner", "FromFoci"] {
        let mut c = controller();
        c.submit("Ellipsoid").unwrap();
        if mode != "Center" {
            c.submit(mode).unwrap();
        }
        let f = 5.0_f64.sqrt();
        let (points, radii): (Vec<[f64; 3]>, [f64; 3]) = match mode {
            "Center" => (vec![[0., 0., 0.], [2., 0., 0.], [0., 3., 0.]], [2., 3., 4.]),
            "Diameter" => (
                vec![[-2., 0., 0.], [2., 0., 0.], [0., 3., 0.]],
                [2., 3., 4.],
            ),
            "Corner" => (vec![[-2., -3., 0.], [2., 3., 0.]], [2., 3., 4.]),
            _ => (vec![[-f, 0., 0.], [f, 0., 0.], [0., 2., 0.]], [3., 2., 4.]),
        };
        for p in points {
            c.accept_world_point(p).unwrap();
        }
        assert!(c.solid_preview([0., 0., 0.]).is_empty());
        let wire = c.solid_preview([0., 0., 4.]);
        assert_eq!(wire.len(), 192, "{mode}");
        for [p, _] in wire {
            let q =
                (p[0] / radii[0]).powi(2) + (p[1] / radii[1]).powi(2) + (p[2] / radii[2]).powi(2);
            assert!((q - 1.).abs() < 1e-9, "{mode}: {p:?}");
        }
        assert!(c.document().entities.is_empty());
        assert_eq!(c.coordinator().undo_len(), 0);
        c.accept_world_point([0., 0., 4.]).unwrap();
        let id = *c.document().entities.keys().next().unwrap();
        let expected = 4. * std::f64::consts::PI / 3. * radii.iter().product::<f64>();
        assert!(
            (measures(&c, id).volume_mm3 - expected).abs() < 1e-7,
            "{mode}"
        );
        c.undo().unwrap();
        assert!(c.document().entities.is_empty());
    }
}

#[test]
fn solid_ellipsoid_around_curve_wire_preview_is_stale_safe() {
    use om9_core::geometry::Curve;
    let rail = Entity::new(
        "rail",
        Geometry::Curve(Curve::Line {
            start: [0., 0., 0.],
            end: [10., 0., 0.],
            domain: [0., 1.],
        }),
        None,
    );
    let id = rail.id;
    let mut doc = Document::new("stale ellipsoid preview");
    doc.entities.insert(id, rail);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("Ellipsoid AroundCurve").unwrap();
    c.submit("OnCurve=0.5").unwrap();
    c.submit("2").unwrap();
    c.submit("3").unwrap();
    assert_eq!(c.solid_preview([9., 0., 0.]).len(), 192);
    c.delete_objects(&[id]).unwrap();
    assert!(c.solid_preview([9., 0., 0.]).is_empty());
    assert!(c.accept_world_point([9., 0., 0.]).is_err());
}

#[test]
fn solid_torus_center_and_two_point_geometry_and_retry() {
    use om9_app::session::InvocationOrigin;
    for route in 0..3 {
        let mut c = controller();
        c.set_cplane_from_three_points([0., 0., 0.], [0., 1., 0.], [0., 0., 1.])
            .unwrap();
        if route == 0 {
            c.dispatch("OM9-SOLID-015", InvocationOrigin::Menu).unwrap();
        } else if route == 1 {
            c.dispatch("Torus2Point", InvocationOrigin::Menu).unwrap();
        } else {
            c.submit("_Torus _P").unwrap();
        }
        let first = if route == 0 {
            [5., 10., 15.]
        } else {
            [5., 5., 15.]
        };
        c.accept_world_point(first).unwrap();
        assert!(c.accept_world_point(first).is_err());
        c.accept_world_point([5., 15., 15.]).unwrap();
        assert!(c.accept_world_point([5., 20., 15.]).is_err());
        assert_eq!(c.prompt(), "Tube radius or point");
        c.submit("Undo").unwrap();
        c.accept_world_point([5., 15., 15.]).unwrap();
        c.accept_world_point([7., 15., 15.]).unwrap();
        let entity = c.document().entities.values().next().unwrap();
        let id = entity.id;
        assert_eq!(
            entity.placement.point([0., 0., 0.]).unwrap(),
            [5., 10., 15.]
        );
        assert_eq!(
            entity.placement.point([0., 0., 2.]).unwrap(),
            [7., 10., 15.]
        );
        assert!((measures(&c, id).volume_mm3 - 40. * std::f64::consts::PI.powi(2)).abs() < 1e-6);
        assert_eq!(c.coordinator().undo_len(), 1);
        c.undo().unwrap();
        assert!(c.document().entities.is_empty());
    }
}

#[test]
fn solid_torus_numeric_inputs_keep_existing_and_point_routes() {
    for mode in [0, 1, 2] {
        let mut c = controller();
        c.submit(if mode == 2 { "Torus 2Point" } else { "Torus" })
            .unwrap();
        if mode == 0 {
            c.submit("5").unwrap();
            c.submit("1").unwrap();
        } else {
            c.submit(if mode == 2 { "-5,0,0" } else { "0,0,0" })
                .unwrap();
            c.submit(if mode == 2 { "10" } else { "5" }).unwrap();
            assert!(c.submit("5").is_err());
            c.submit("1").unwrap();
        }
        let id = *c.document().entities.keys().next().unwrap();
        assert!((measures(&c, id).volume_mm3 - 10. * std::f64::consts::PI.powi(2)).abs() < 1e-6);
    }
}

#[test]
fn solid_torus_three_points_retries_and_preserves_circle() {
    for clockwise in [false, true] {
        let mut c = controller();
        c.set_cplane_from_three_points([0., 0., 0.], [0., 1., 0.], [0., 0., 1.])
            .unwrap();
        c.submit("Torus 3Point").unwrap();
        c.accept_world_point([5., 15., 15.]).unwrap();
        c.accept_world_point([5., 5., 15.]).unwrap();
        assert!(c.accept_world_point([5., 10., 15.]).is_err());
        assert_eq!(c.prompt(), "Third point on ring");
        assert_eq!(c.coordinator().undo_len(), 0);
        let third = if clockwise {
            [5., 10., 10.]
        } else {
            [5., 10., 20.]
        };
        c.accept_world_point(third).unwrap();
        c.submit("Undo").unwrap();
        assert_eq!(c.prompt(), "Third point on ring");
        c.accept_world_point(third).unwrap();
        c.submit("2").unwrap();
        let entity = c.document().entities.values().next().unwrap();
        let id = entity.id;
        assert_eq!(
            entity.placement.point([0., 0., 0.]).unwrap(),
            [5., 10., 15.]
        );
        assert!((measures(&c, id).volume_mm3 - 40. * std::f64::consts::PI.powi(2)).abs() < 1e-6);
        c.undo().unwrap();
        assert!(c.document().entities.is_empty());
    }
}

#[test]
fn solid_torus_three_point_radius_selects_both_centers_and_undo() {
    for sign in [-1., 1.] {
        let mut c = controller();
        c.submit("Torus 3Point").unwrap();
        c.accept_world_point([-2., 0., 0.]).unwrap();
        c.accept_world_point([2., 0., 0.]).unwrap();
        assert!(c.submit("Radius=1").is_err());
        assert_eq!(c.prompt(), "Third point on ring");
        c.submit("Radius").unwrap();
        assert_eq!(c.prompt(), "Base circle radius");
        c.submit("3").unwrap();
        assert_eq!(c.prompt(), "Choose base circle side");
        assert!(c.accept_world_point([0., 0., 0.]).is_err());
        c.accept_world_point([0., sign, 0.]).unwrap();
        assert_eq!(c.prompt(), "Tube radius or point");
        c.submit("Undo").unwrap();
        assert_eq!(c.prompt(), "Choose base circle side");
        c.accept_world_point([0., sign, 0.]).unwrap();
        c.submit("1").unwrap();
        let entity = c.document().entities.values().next().unwrap();
        let center = entity.placement.point([0., 0., 0.]).unwrap();
        assert!(center[0].abs() < 1e-8 && (center[1] - sign * 5.0_f64.sqrt()).abs() < 1e-8);
        assert!(
            (measures(&c, entity.id).volume_mm3 - 6. * std::f64::consts::PI.powi(2)).abs() < 1e-6
        );
        c.undo().unwrap();
        assert!(c.document().entities.is_empty());
    }
}

#[test]
fn solid_torus_three_point_radius_can_be_picked() {
    let mut c = controller();
    c.submit("Torus 3Point").unwrap();
    c.accept_world_point([-2., 0., 0.]).unwrap();
    c.accept_world_point([2., 0., 0.]).unwrap();
    c.submit("Radius").unwrap();
    c.accept_world_point([-2., 3., 0.]).unwrap();
    assert_eq!(c.prompt(), "Choose base circle side");
    c.accept_world_point([0., 1., 0.]).unwrap();
    c.submit("1").unwrap();
    let entity = c.document().entities.values().next().unwrap();
    assert!((entity.placement.point([0., 0., 0.]).unwrap()[1] - 5.0_f64.sqrt()).abs() < 1e-8);
    assert!((measures(&c, entity.id).volume_mm3 - 6. * std::f64::consts::PI.powi(2)).abs() < 1e-6);
}

#[test]
fn solid_torus_around_curve_captures_tangent_and_preserves_source() {
    use om9_core::geometry::{Curve, Transform};
    for circle in [false, true] {
        let curve = if circle {
            om9_core::curve_build::circle_on_frame([0., 0., 0.], [1., 0., 0.], [0., 1., 0.], 5.)
                .unwrap()
        } else {
            Curve::Line {
                start: [0., 0., 0.],
                end: [10., 0., 0.],
                domain: [0., 1.],
            }
        };
        let mut rail = Entity::new("rail", Geometry::Curve(curve), None);
        rail.placement = Transform::translation([10., 20., 4.]);
        let id = rail.id;
        let mut doc = Document::new("torus around curve");
        doc.entities.insert(id, rail);
        let mut c = Controller::new(doc, Limits::default()).unwrap();
        let before = c.document().clone();
        c.submit("Torus AroundCurve").unwrap();
        assert!(c.solid_pipe_needs_curve());
        c.solid_pipe_pick_curve(id).unwrap();
        c.submit(if circle {
            "OnCurve=0.25"
        } else {
            "OnCurve=0.5"
        })
        .unwrap();
        c.submit("5").unwrap();
        c.submit("1").unwrap();
        let entity = c.document().entities.values().find(|e| e.id != id).unwrap();
        let a = entity.placement.point([0., 0., 0.]).unwrap();
        let b = entity.placement.point([0., 0., 1.]).unwrap();
        let expected = if circle {
            [10., 25., 4.]
        } else {
            [15., 20., 4.]
        };
        assert!((0..3).all(|i| (a[i] - expected[i]).abs() < 1e-7));
        assert!((b[0] - a[0] - if circle { -1. } else { 1. }).abs() < 1e-7);
        assert!(
            (measures(&c, entity.id).volume_mm3 - 10. * std::f64::consts::PI.powi(2)).abs() < 1e-6
        );
        c.undo().unwrap();
        assert_eq!(c.document(), &before);
    }
}

#[test]
fn solid_torus_around_curve_rejects_deleted_source() {
    use om9_core::geometry::Curve;
    let rail = Entity::new(
        "rail",
        Geometry::Curve(Curve::Line {
            start: [0., 0., 0.],
            end: [10., 0., 0.],
            domain: [0., 1.],
        }),
        None,
    );
    let id = rail.id;
    let mut doc = Document::new("stale torus source");
    doc.entities.insert(id, rail);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("Torus AroundCurve").unwrap();
    c.accept_world_point([5., 1., 0.]).unwrap();
    c.submit("5").unwrap();
    c.delete_objects(&[id]).unwrap();
    let after = c.document().clone();
    let count = c.coordinator().undo_len();
    assert!(c.submit("1").is_err());
    assert_eq!(c.document(), &after);
    assert_eq!(c.coordinator().undo_len(), count);
}

#[test]
fn solid_torus_wire_preview_matches_ring_and_tube_across_modes() {
    for mode in ["Center", "2Point", "3Point", "Radius"] {
        let mut c = controller();
        c.submit("Torus").unwrap();
        if mode != "Center" {
            c.submit(if mode == "Radius" { "3Point" } else { mode })
                .unwrap();
        }
        let final_point = match mode {
            "Center" => {
                c.accept_world_point([0., 0., 0.]).unwrap();
                c.accept_world_point([5., 0., 0.]).unwrap();
                [6., 0., 0.]
            }
            "2Point" => {
                c.accept_world_point([-5., 0., 0.]).unwrap();
                c.accept_world_point([5., 0., 0.]).unwrap();
                [6., 0., 0.]
            }
            "3Point" => {
                for p in [[5., 0., 0.], [-5., 0., 0.], [0., 5., 0.]] {
                    c.accept_world_point(p).unwrap();
                }
                [0., 6., 0.]
            }
            _ => {
                c.accept_world_point([-5., 0., 0.]).unwrap();
                c.accept_world_point([5., 0., 0.]).unwrap();
                c.submit("Radius=5").unwrap();
                c.accept_world_point([0., 1., 0.]).unwrap();
                [0., 6., 0.]
            }
        };
        let wire = c.solid_preview(final_point);
        assert_eq!(wire.len(), 256, "{mode}");
        for [p, _] in wire {
            let q = (p[0].hypot(p[1]) - 5.).powi(2) + p[2].powi(2);
            assert!((q - 1.).abs() < 1e-8, "{mode}: {p:?}");
        }
        assert!(c.document().entities.is_empty());
        assert_eq!(c.coordinator().undo_len(), 0);
        c.accept_world_point(final_point).unwrap();
        let id = *c.document().entities.keys().next().unwrap();
        assert!((measures(&c, id).volume_mm3 - 10. * std::f64::consts::PI.powi(2)).abs() < 1e-6);
    }
}

#[test]
fn solid_torus_around_curve_preview_rejects_stale_source() {
    use om9_core::geometry::Curve;
    let rail = Entity::new(
        "rail",
        Geometry::Curve(Curve::Line {
            start: [0., 0., 0.],
            end: [10., 0., 0.],
            domain: [0., 1.],
        }),
        None,
    );
    let id = rail.id;
    let mut doc = Document::new("torus preview stale");
    doc.entities.insert(id, rail);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("Torus AroundCurve").unwrap();
    c.submit("OnCurve=0.5").unwrap();
    c.submit("5").unwrap();
    assert_eq!(c.solid_preview([5., 6., 0.]).len(), 256);
    c.delete_objects(&[id]).unwrap();
    assert!(c.solid_preview([5., 6., 0.]).is_empty());
    assert!(c.accept_world_point([5., 6., 0.]).is_err());
}

#[test]
fn solid_torus_vertical_reorients_and_undo_restores_input_plane() {
    let mut c = controller();
    c.submit("Torus Vertical").unwrap();
    assert!(c.submit("5 1").is_err());
    assert!(c.document().entities.is_empty());
    c.accept_world_point([10., 20., 30.]).unwrap();
    c.accept_world_point([10., 25., 30.]).unwrap();
    assert_eq!(c.solid_preview([11., 25., 30.]).len(), 256);
    c.submit("Undo").unwrap();
    assert_eq!(c.prompt(), "Ring radius or point");
    c.accept_world_point([15., 20., 30.]).unwrap();
    assert_eq!(c.solid_preview([15., 19., 30.]).len(), 256);
    c.accept_world_point([15., 19., 30.]).unwrap();
    let entity = c.document().entities.values().next().unwrap();
    let a = entity.placement.point([0., 0., 0.]).unwrap();
    let b = entity.placement.point([0., 0., 1.]).unwrap();
    assert_eq!(a, [10., 20., 30.]);
    assert_eq!(b, [10., 19., 30.]);
    assert!((measures(&c, entity.id).volume_mm3 - 10. * std::f64::consts::PI.powi(2)).abs() < 1e-6);
    c.undo().unwrap();
    assert!(c.document().entities.is_empty());
}

#[test]
fn solid_tube_center_and_two_and_three_point_workflows() {
    use om9_app::session::InvocationOrigin;
    for mode in 0..3 {
        let mut c = controller();
        c.set_cplane_from_three_points([0., 0., 0.], [0., 1., 0.], [0., 0., 1.])
            .unwrap();
        let before = c.document().clone();
        if mode == 1 {
            c.dispatch("Tube2Point", InvocationOrigin::Menu).unwrap();
        } else if mode == 2 {
            c.submit("Tube 3Point").unwrap();
        } else {
            c.submit("Tube").unwrap();
        }
        if mode == 0 {
            c.accept_world_point([4., 3., 7.]).unwrap();
            c.accept_world_point([4., 6., 7.]).unwrap();
        } else if mode == 1 {
            c.accept_world_point([4., 0., 7.]).unwrap();
            c.accept_world_point([4., 6., 7.]).unwrap();
        } else {
            for p in [[4., 6., 7.], [4., 0., 7.], [4., 3., 10.]] {
                c.accept_world_point(p).unwrap();
            }
        }
        assert_eq!(c.prompt(), "Second radius or point");
        c.submit("1").unwrap();
        assert_eq!(c.prompt(), "Height or point");
        assert!(c.submit("0").is_err());
        assert_eq!(c.document(), &before);
        c.accept_world_point([9., 3., 7.]).unwrap();
        let id = *c.document().entities.keys().next().unwrap();
        let e = &c.document().entities[&id];
        let a = e.placement.point([0., 0., 0.]).unwrap();
        let b = e.placement.point([0., 0., 5.]).unwrap();
        assert!((a[0] - 4.).abs() < 1e-8 && (a[1] - 3.).abs() < 1e-8 && (a[2] - 7.).abs() < 1e-8);
        assert!((b[0] - 9.).abs() < 1e-8);
        let m = measures(&c, id);
        assert!(m.closed && (m.volume_mm3 - 40. * std::f64::consts::PI).abs() < 1e-6);
        c.undo().unwrap();
        assert_eq!(c.document(), &before);
    }
}

#[test]
fn solid_pyramid_center_corner_and_signed_height() {
    for (sides, height) in [(4, 6.), (5, -6.)] {
        let mut c = controller();
        c.set_cplane_from_three_points([0., 0., 0.], [0., 1., 0.], [0., 0., 1.])
            .unwrap();
        let before = c.document().clone();
        c.submit("Pyramid").unwrap();
        c.submit(&format!("NumSides={sides}")).unwrap();
        assert!(c.submit("NumSides=2").is_err());
        c.accept_world_point([10., 20., 30.]).unwrap();
        assert!(c.accept_world_point([10., 20., 30.]).is_err());
        c.accept_world_point([10., 20., 32.]).unwrap();
        assert_eq!(c.prompt(), "Apex height or point");
        let wire = c.solid_preview([10. + height, 20., 30.]);
        assert_eq!(wire.len(), 2 * sides as usize);
        for [a, b] in wire {
            assert!((a[0] - 10.).abs() < 1e-8);
            assert!((b[0] - 10.).abs() < 1e-8 || (b[0] - (10. + height)).abs() < 1e-8);
        }
        assert_eq!(c.coordinator().undo_len(), 0);
        assert!(c.submit("0").is_err());
        assert_eq!(c.document(), &before);
        c.accept_world_point([10. + height, 20., 30.]).unwrap();
        let id = *c.document().entities.keys().next().unwrap();
        let e = &c.document().entities[&id];
        let a = e.placement.point([0., 0., 0.]).unwrap();
        let b = e.placement.point([0., 0., height.abs()]).unwrap();
        let corner = e.placement.point([2., 0., 0.]).unwrap();
        assert_eq!(a, [10., 20., 30.]);
        assert_eq!(b, [10. + height, 20., 30.]);
        assert!((corner[2] - 32.).abs() < 1e-8);
        let expected =
            sides as f64 * 2. * (std::f64::consts::TAU / sides as f64).sin() * height.abs() / 3.;
        assert!((measures(&c, id).volume_mm3 - expected).abs() < 1e-6);
        c.undo().unwrap();
        assert_eq!(c.document(), &before);
    }
}

#[test]
fn solid_pyramid_open_base_for_center_edge_and_star() {
    for mode in ["center", "edge", "star", "circumscribed"] {
        let mut c = controller();
        let before = c.document().clone();
        c.submit("Pyramid").unwrap();
        c.submit("Solid=No").unwrap();
        assert!(c.submit("4 3 5").is_err());
        assert!(c.submit("Solid=maybe").is_err());
        if mode == "edge" {
            c.submit("Edge").unwrap();
        }
        if mode == "star" {
            c.submit("Star").unwrap();
        }
        if mode == "circumscribed" {
            c.submit("Circumscribed").unwrap();
        }
        c.accept_world_point([0., 0., 0.]).unwrap();
        if mode == "edge" {
            c.accept_world_point([4., 0., 0.]).unwrap();
        } else {
            c.submit("3").unwrap();
        }
        if mode == "star" {
            c.submit("2").unwrap();
        }
        assert_eq!(c.document(), &before);
        c.submit("-5").unwrap();
        let id = *c.document().entities.keys().next().unwrap();
        let m = measures(&c, id);
        let faces = if mode == "star" { 8 } else { 4 };
        assert!(
            !m.closed && m.faces == faces && m.naked_edges == faces,
            "{mode}: {m:?}"
        );
        c.undo().unwrap();
        assert_eq!(c.document(), &before);
    }
}

#[test]
fn solid_pyramid_around_curve_center_star_and_source_staleness() {
    use om9_core::geometry::{Curve, Transform};
    for (star, typed) in [(false, true), (false, false), (true, true)] {
        let mut rail = Entity::new(
            "rail",
            Geometry::Curve(Curve::Line {
                start: [0., 0., 0.],
                end: [10., 0., 0.],
                domain: [0., 1.],
            }),
            None,
        );
        rail.placement = Transform::translation([10., 20., 4.]);
        let id = rail.id;
        let mut doc = Document::new("pyramid around curve");
        doc.entities.insert(id, rail);
        let mut c = Controller::new(doc, Limits::default()).unwrap();
        let before = c.document().clone();
        c.submit("Pyramid AroundCurve").unwrap();
        if star {
            c.submit("Star").unwrap();
            c.submit("Solid=No").unwrap();
        }
        assert!(c.solid_pipe_needs_curve());
        c.solid_pipe_pick_curve(id).unwrap();
        assert!(c.submit("OnCurve=2").is_err());
        if typed {
            c.submit("OnCurve=0.5").unwrap();
        } else {
            c.accept_world_point([15., 21., 4.]).unwrap();
        }
        c.submit("3").unwrap();
        if star {
            c.submit("2").unwrap();
        }
        assert_eq!(c.coordinator().undo_len(), 0);
        assert_eq!(
            c.solid_preview([20., 20., 4.]).len(),
            if star { 16 } else { 8 }
        );
        c.submit("5").unwrap();
        let e = c.document().entities.values().find(|e| e.id != id).unwrap();
        assert_eq!(e.placement.point([0., 0., 0.]).unwrap(), [15., 20., 4.]);
        assert_eq!(e.placement.point([0., 0., 5.]).unwrap(), [20., 20., 4.]);
        let m = measures(&c, e.id);
        assert_eq!(m.closed, !star);
        c.undo().unwrap();
        assert_eq!(c.document(), &before);
    }
    let rail = Entity::new(
        "rail",
        Geometry::Curve(Curve::Line {
            start: [0., 0., 0.],
            end: [10., 0., 0.],
            domain: [0., 1.],
        }),
        None,
    );
    let id = rail.id;
    let mut doc = Document::new("stale pyramid rail");
    doc.entities.insert(id, rail);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("Pyramid AroundCurve").unwrap();
    c.submit("OnCurve=0.5").unwrap();
    c.submit("3").unwrap();
    c.delete_objects(&[id]).unwrap();
    let after = c.document().clone();
    let count = c.coordinator().undo_len();
    assert!(c.solid_preview([10., 0., 0.]).is_empty());
    assert!(c.submit("5").is_err());
    assert_eq!(c.document(), &after);
    assert_eq!(c.coordinator().undo_len(), count);
}

#[test]
fn solid_pyramid_around_circle_undo_and_switch_back() {
    let curve =
        om9_core::curve_build::circle_on_frame([0., 0., 0.], [1., 0., 0.], [0., 1., 0.], 5.)
            .unwrap();
    let rail = Entity::new("circle", Geometry::Curve(curve), None);
    let id = rail.id;
    let mut doc = Document::new("pyramid around circle");
    doc.entities.insert(id, rail);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    let before = c.document().clone();
    c.submit("Pyramid").unwrap();
    c.submit("AroundCurve").unwrap();
    c.submit("OnCurve=0.25").unwrap();
    c.submit("Undo").unwrap();
    assert_eq!(c.prompt(), "Pick center on curve or OnCurve=");
    c.submit("Center").unwrap();
    assert_eq!(c.prompt(), "Center of pyramid base");
    c.submit("AroundCurve").unwrap();
    c.submit("OnCurve=0.25").unwrap();
    c.submit("Circumscribed").unwrap();
    c.submit("3").unwrap();
    c.submit("5").unwrap();
    let e = c.document().entities.values().find(|e| e.id != id).unwrap();
    let a = e.placement.point([0., 0., 0.]).unwrap();
    let b = e.placement.point([0., 0., 5.]).unwrap();
    assert!(
        (a[0]).abs() < 1e-7
            && (a[1] - 5.).abs() < 1e-7
            && (b[0] + 5.).abs() < 1e-7
            && (b[1] - 5.).abs() < 1e-7
    );
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
}

#[test]
fn solid_cone_center_two_three_and_fit_workflows() {
    use om9_app::session::InvocationOrigin;
    for mode in 0..4 {
        let mut c = controller();
        let before = c.document().clone();
        match mode {
            0 => {
                c.dispatch("OM9-SOLID-019", InvocationOrigin::Menu).unwrap();
            }
            1 => {
                c.submit("_Cone _P").unwrap();
            }
            2 => {
                c.submit("Cone 3Point").unwrap();
            }
            _ => {
                c.submit("Cone FitPoints").unwrap();
            }
        }
        match mode {
            0 => {
                c.accept_world_point([0., 0., 0.]).unwrap();
                c.submit(&format!("Area={}", 9. * std::f64::consts::PI))
                    .unwrap();
            }
            1 => {
                c.accept_world_point([-3., 0., 0.]).unwrap();
                c.accept_world_point([3., 0., 0.]).unwrap();
            }
            2 => {
                c.accept_world_point([-3., 0., 0.]).unwrap();
                c.accept_world_point([3., 0., 0.]).unwrap();
                c.accept_world_point([0., 3., 0.]).unwrap();
            }
            _ => {
                for p in [[3., 0., 0.], [0., 3., 0.], [-3., 0., 0.], [0., -3., 0.]] {
                    c.accept_world_point(p).unwrap();
                }
                c.submit("").unwrap();
            }
        }
        assert_eq!(c.prompt(), "Height or point");
        let preview = c.solid_preview([0., 0., 5.]);
        assert_eq!(preview.len(), 68);
        assert_eq!(c.document(), &before);
        c.submit("5").unwrap();
        let id = *c.document().entities.keys().next().unwrap();
        let m = measures(&c, id);
        assert!(
            m.closed && (m.volume_mm3 - 15. * std::f64::consts::PI).abs() < 1e-6,
            "{m:?}"
        );
        c.undo().unwrap();
        assert_eq!(c.document(), &before);
    }
}

#[test]
fn solid_cone_three_point_radius_selects_both_sides() {
    for side in [-1., 1.] {
        let mut c = controller();
        let before = c.document().clone();
        c.submit("Cone 3Point").unwrap();
        c.accept_world_point([0., 0., 0.]).unwrap();
        c.accept_world_point([4., 0., 0.]).unwrap();
        assert!(c.submit("Radius=1").is_err());
        c.submit("Radius=3").unwrap();
        c.accept_world_point([2., side, 0.]).unwrap();
        c.submit("-5").unwrap();
        let id = *c.document().entities.keys().next().unwrap();
        let e = &c.document().entities[&id];
        let center = e.placement.point([0., 0., 0.]).unwrap();
        assert!((center[1] - side * 5.0_f64.sqrt()).abs() < 1e-8);
        assert!((measures(&c, id).volume_mm3 - 15. * std::f64::consts::PI).abs() < 1e-6);
        c.undo().unwrap();
        assert_eq!(c.document(), &before);
    }
}

#[test]
fn solid_cone_rejects_deleted_around_curve_source() {
    use om9_core::geometry::Curve;
    let e = Entity::new(
        "rail",
        Geometry::Curve(Curve::Line {
            start: [0., 0., 0.],
            end: [10., 0., 0.],
            domain: [0., 1.],
        }),
        None,
    );
    let id = e.id;
    let mut doc = Document::new("stale cone");
    doc.entities.insert(id, e);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("Cone AroundCurve").unwrap();
    c.submit("OnCurve=0.5").unwrap();
    c.submit("2").unwrap();
    assert!(!c.solid_preview([10., 0., 0.]).is_empty());
    c.delete_objects(&[id]).unwrap();
    let after = c.document().clone();
    let count = c.coordinator().undo_len();
    assert!(c.solid_preview([10., 0., 0.]).is_empty());
    assert!(c.submit("5").is_err());
    assert_eq!(c.document(), &after);
    assert_eq!(c.coordinator().undo_len(), count);
}

#[test]
fn solid_cone_open_and_negative_height() {
    let mut c = controller();
    c.set_cplane_from_three_points([0., 0., 0.], [0., 1., 0.], [0., 0., 1.])
        .unwrap();
    let before = c.document().clone();
    c.submit("Cone").unwrap();
    c.submit("Solid=No").unwrap();
    c.accept_world_point([10., 20., 30.]).unwrap();
    c.submit("2").unwrap();
    c.submit("-5").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    let e = &c.document().entities[&id];
    let a = e.placement.point([0., 0., 0.]).unwrap();
    let b = e.placement.point([0., 0., 5.]).unwrap();
    assert_eq!(a, [10., 20., 30.]);
    assert_eq!(b, [5., 20., 30.]);
    let m = measures(&c, id);
    assert!(
        !m.closed && m.solids == 0 && m.faces == 1 && m.naked_edges == 1,
        "{m:?}"
    );
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
}

#[test]
fn solid_cone_around_curve_and_tangent_point() {
    use om9_core::geometry::Curve;
    for tangent in [false, true] {
        let rail = Entity::new(
            "rail",
            Geometry::Curve(Curve::Line {
                start: [0., 0., 0.],
                end: [10., 0., 0.],
                domain: [0., 1.],
            }),
            None,
        );
        let id = rail.id;
        let mut doc = Document::new("cone curve");
        doc.entities.insert(id, rail);
        let mut c = Controller::new(doc, Limits::default()).unwrap();
        let before = c.document().clone();
        c.submit(if tangent {
            "Cone Tangent"
        } else {
            "Cone AroundCurve"
        })
        .unwrap();
        c.solid_pipe_pick_curve(id).unwrap();
        c.submit("OnCurve=0.5").unwrap();
        let r = if tangent {
            c.submit("Point").unwrap();
            c.accept_world_point([8., 2., 0.]).unwrap();
            3.25_f64
        } else {
            c.submit("2").unwrap();
            2.
        };
        c.submit("5").unwrap();
        let e = c.document().entities.values().find(|e| e.id != id).unwrap();
        assert!(
            (measures(&c, e.id).volume_mm3 - std::f64::consts::PI * r * r * 5. / 3.).abs() < 1e-6
        );
        c.undo().unwrap();
        assert_eq!(c.document(), &before);
    }
}

#[test]
fn solid_pyramid_original_star_macro_starts_five_point_star() {
    let mut c = controller();
    c.submit("_Pyramid _Numsides=5 _S").unwrap();
    c.accept_world_point([0., 0., 0.]).unwrap();
    c.submit("5").unwrap();
    c.submit("2").unwrap();
    c.submit("6").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    let expected = 5. * 5. * 2. * (std::f64::consts::PI / 5.).sin() * 2.;
    assert!((measures(&c, id).volume_mm3 - expected).abs() < 1e-6);
}

#[test]
fn solid_pyramid_star_radii_automatic_and_signed_height() {
    for automatic in [false, true] {
        let mut c = controller();
        c.set_cplane_from_three_points([0., 0., 0.], [0., 1., 0.], [0., 0., 1.])
            .unwrap();
        let before = c.document().clone();
        c.submit("Pyramid Star").unwrap();
        c.submit("NumSides=5").unwrap();
        c.accept_world_point([10., 20., 30.]).unwrap();
        c.accept_world_point([10., 25., 30.]).unwrap();
        assert_eq!(c.prompt(), "Second star radius or point (Automatic)");
        assert!(c.submit("0").is_err());
        if automatic {
            c.submit("").unwrap();
        } else {
            c.accept_world_point([10., 22., 30.]).unwrap();
        }
        assert_eq!(c.prompt(), "Apex height or point");
        c.submit("Undo").unwrap();
        assert_eq!(c.prompt(), "Second star radius or point (Automatic)");
        if automatic {
            c.submit("Automatic").unwrap();
        } else {
            c.submit("2").unwrap();
        }
        let edges = c.solid_preview([4., 20., 30.]);
        assert_eq!(edges.len(), 20);
        assert_eq!(c.document(), &before);
        assert_eq!(c.coordinator().undo_len(), 0);
        c.accept_world_point([4., 20., 30.]).unwrap();
        let id = *c.document().entities.keys().next().unwrap();
        let e = &c.document().entities[&id];
        assert_eq!(e.placement.point([0., 0., 0.]).unwrap(), [10., 20., 30.]);
        assert_eq!(e.placement.point([0., 0., 6.]).unwrap(), [4., 20., 30.]);
        let inner = if automatic { 5. } else { 2. };
        let expected = 5. * 5. * inner * (std::f64::consts::PI / 5.).sin() * 2.;
        assert!((measures(&c, id).volume_mm3 - expected).abs() < 1e-6);
        c.undo().unwrap();
        assert_eq!(c.document(), &before);
    }
}

#[test]
fn solid_pyramid_edge_flip_before_and_after_base() {
    for flip_before in [false, true] {
        let mut c = controller();
        let before = c.document().clone();
        c.submit("Pyramid Edge").unwrap();
        assert_eq!(c.prompt(), "First point of base edge");
        assert!(c.submit("4 2 5").is_err());
        c.accept_world_point([0., 0., 0.]).unwrap();
        assert_eq!(c.prompt(), "Second point of base edge");
        assert!(c.submit("2").is_err());
        assert!(c.accept_world_point([0., 0., 0.]).is_err());
        if flip_before {
            c.submit("Flip").unwrap();
        }
        let preview = c.solid_preview([4., 0., 0.]);
        assert_eq!(preview.len(), 4);
        c.accept_world_point([4., 0., 0.]).unwrap();
        if !flip_before {
            c.submit("Flip").unwrap();
        }
        assert_eq!(c.prompt(), "Apex height or point");
        let preview = c.solid_preview([2., -2., 6.]);
        assert_eq!(preview.len(), 8);
        assert_eq!(c.document(), &before);
        c.accept_world_point([2., -2., 6.]).unwrap();
        let id = *c.document().entities.keys().next().unwrap();
        let e = &c.document().entities[&id];
        let center = e.placement.point([0., 0., 0.]).unwrap();
        assert!((center[0] - 2.).abs() < 1e-8 && (center[1] + 2.).abs() < 1e-8);
        assert!((measures(&c, id).volume_mm3 - 32.).abs() < 1e-6);
        c.undo().unwrap();
        assert_eq!(c.document(), &before);
    }
}

#[test]
fn solid_pyramid_edge_five_sides_on_rotated_plane() {
    let mut c = controller();
    c.set_cplane_from_three_points([0., 0., 0.], [0., 1., 0.], [0., 0., 1.])
        .unwrap();
    let before = c.document().clone();
    c.submit("Pyramid Edge").unwrap();
    c.submit("NumSides=5").unwrap();
    c.accept_world_point([10., 20., 30.]).unwrap();
    c.accept_world_point([10., 24., 30.]).unwrap();
    c.accept_world_point([4., 22., 32.]).unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    let e = &c.document().entities[&id];
    let center = e.placement.point([0., 0., 0.]).unwrap();
    let expected = 30. + 2. / (std::f64::consts::PI / 5.).tan();
    assert!(
        (center[0] - 10.).abs() < 1e-8
            && (center[1] - 22.).abs() < 1e-8
            && (center[2] - expected).abs() < 1e-8
    );
    assert!((measures(&c, id).volume_mm3 - 40. / (std::f64::consts::PI / 5.).tan()).abs() < 1e-6);
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
}

#[test]
fn solid_pyramid_edge_undo_returns_to_first_endpoint() {
    let mut c = controller();
    c.submit("Pyramid Edge").unwrap();
    c.accept_world_point([0., 0., 0.]).unwrap();
    c.accept_world_point([4., 0., 0.]).unwrap();
    c.submit("Undo").unwrap();
    assert_eq!(c.prompt(), "Second point of base edge");
    c.submit("Flip").unwrap();
    c.accept_world_point([4., 0., 0.]).unwrap();
    c.submit("-6").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    let e = &c.document().entities[&id];
    let p = e.placement.point([0., 0., 0.]).unwrap();
    assert!((p[1] + 2.).abs() < 1e-8);
    assert!((measures(&c, id).volume_mm3 - 32.).abs() < 1e-6);
}

#[test]
fn solid_pyramid_circumscribed_midpoint_and_signed_preview() {
    for (sides, height) in [(4, 5.), (5, -5.)] {
        let mut c = controller();
        let before = c.document().clone();
        c.submit("Pyramid").unwrap();
        c.submit(&format!("NumSides={sides}")).unwrap();
        c.submit("Circumscribed").unwrap();
        c.accept_world_point([0., 0., 0.]).unwrap();
        assert_eq!(c.prompt(), "Base edge midpoint or radius");
        c.accept_world_point([3., 0., 0.]).unwrap();
        let wire = c.solid_preview([0., 0., height]);
        assert_eq!(wire.len(), 2 * sides as usize);
        let [a, b] = wire[0];
        assert!(((a[0] + b[0]) / 2. - 3.).abs() < 1e-8 && ((a[1] + b[1]) / 2.).abs() < 1e-8);
        assert_eq!(c.document(), &before);
        assert_eq!(c.coordinator().undo_len(), 0);
        c.submit(&height.to_string()).unwrap();
        let id = *c.document().entities.keys().next().unwrap();
        let e = &c.document().entities[&id];
        let r = 3. / (std::f64::consts::PI / sides as f64).cos();
        assert!((0..sides).any(|i| {
            let a = std::f64::consts::TAU * i as f64 / sides as f64;
            let b = std::f64::consts::TAU * (i + 1) as f64 / sides as f64;
            let p = e.placement.point([r * a.cos(), r * a.sin(), 0.]).unwrap();
            let q = e.placement.point([r * b.cos(), r * b.sin(), 0.]).unwrap();
            ((p[0] + q[0]) / 2. - 3.).abs() < 1e-8 && ((p[1] + q[1]) / 2.).abs() < 1e-8
        }));
        let m = measures(&c, id);
        let expected =
            sides as f64 * 9. * (std::f64::consts::PI / sides as f64).tan() * height.abs() / 3.;
        assert!((m.volume_mm3 - expected).abs() < 1e-6, "{m:?}");
        c.undo().unwrap();
        assert_eq!(c.document(), &before);
    }
}

#[test]
fn solid_pyramid_base_style_switch_and_undo() {
    let mut c = controller();
    c.submit("Pyramid").unwrap();
    c.submit("Circumscribed").unwrap();
    c.accept_world_point([0., 0., 0.]).unwrap();
    c.submit("Inscribed").unwrap();
    assert_eq!(c.prompt(), "Base corner or radius");
    c.submit("2").unwrap();
    assert!(c.submit("Circumscribed").is_err());
    c.submit("Undo").unwrap();
    c.submit("Circumscribed").unwrap();
    assert_eq!(c.prompt(), "Base edge midpoint or radius");
}

#[test]
fn solid_pyramid_side_budget_rejects_before_preview_or_commit() {
    let limits = Limits {
        max_points: 3,
        ..Limits::default()
    };
    let mut c = Controller::new(Document::new("pyramid budget"), limits).unwrap();
    let before = c.document().clone();
    c.submit("Pyramid").unwrap();
    assert!(c.submit("NumSides=4").is_err());
    c.accept_world_point([0., 0., 0.]).unwrap();
    c.submit("2").unwrap();
    assert!(c.solid_preview([0., 0., 5.]).is_empty());
    assert!(c.submit("5").is_err());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), 0);
    c.cancel();
    assert!(c.submit("Pyramid 4 2 5").is_err());
    assert_eq!(c.document(), &before);
}

#[test]
fn solid_pyramid_default_and_undo_before_commit() {
    let mut c = controller();
    let before = c.document().clone();
    c.submit("Pyramid").unwrap();
    c.submit("1,2,3").unwrap();
    c.submit("2").unwrap();
    c.submit("Undo").unwrap();
    assert_eq!(c.prompt(), "Base corner or radius");
    c.submit("2").unwrap();
    c.submit("6").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    assert!((measures(&c, id).volume_mm3 - 16.).abs() < 1e-6);
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
}

#[test]
fn solid_tube_area_uses_square_document_units_and_can_switch_modes() {
    let mut doc = Document::new("tube area units");
    doc.page_unit = om9_core::geometry::Unit::Centimeter;
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    let before = c.document().clone();
    c.submit("Tube").unwrap();
    c.accept_world_point([10., 20., 30.]).unwrap();
    assert!(c.solid_option_tokens().iter().any(|(n, _)| *n == "Area"));
    c.submit("Diameter").unwrap();
    c.submit("Area").unwrap();
    assert_eq!(c.prompt(), "Base area");
    assert!(c.submit("-1").is_err());
    assert_eq!(c.prompt(), "Base area");
    c.submit("Radius").unwrap();
    assert_eq!(c.prompt(), "First radius or point");
    c.submit(&format!("Area={}", 9. * std::f64::consts::PI))
        .unwrap();
    assert_eq!(c.prompt(), "Second radius or point");
    c.submit("1").unwrap();
    c.submit("0.5").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    assert!((measures(&c, id).volume_mm3 - 4000. * std::f64::consts::PI).abs() < 1e-5);
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
}

#[test]
fn solid_tube_wall_thickness_value_and_point_modes() {
    for picked in [0, 1, 2] {
        let mut c = controller();
        let before = c.document().clone();
        c.submit("Tube").unwrap();
        c.submit("Diameter").unwrap();
        c.accept_world_point([0., 0., 0.]).unwrap();
        c.submit("6").unwrap();
        assert!(
            c.solid_option_tokens()
                .iter()
                .any(|(_, v)| *v == "WallThickness=")
        );
        c.submit("WallThickness").unwrap();
        assert_eq!(c.prompt(), "Wall thickness or point");
        assert!(c.submit("3").is_err());
        assert_eq!(c.document(), &before);
        c.submit("Undo").unwrap();
        assert_eq!(c.prompt(), "Second radius or point");
        if picked != 0 {
            c.submit("WallThickness").unwrap();
            c.accept_world_point([if picked == 1 { 1. } else { 5. }, 0., 0.])
                .unwrap();
        } else {
            c.submit("WallThickness=2").unwrap();
        }
        assert_eq!(c.prompt(), "Height or point");
        c.submit("5").unwrap();
        let id = *c.document().entities.keys().next().unwrap();
        let expected = if picked == 2 { 80. } else { 40. };
        assert!((measures(&c, id).volume_mm3 - expected * std::f64::consts::PI).abs() < 1e-6);
        c.undo().unwrap();
        assert_eq!(c.document(), &before);
    }
}

#[test]
fn solid_tube_wall_thickness_with_both_sides() {
    let mut c = controller();
    let before = c.document().clone();
    c.submit("Tube").unwrap();
    c.submit("BothSides=Yes").unwrap();
    c.accept_world_point([0., 0., 0.]).unwrap();
    c.submit("3").unwrap();
    c.submit("WallThickness=2").unwrap();
    assert_eq!(c.solid_preview([0., 0., -5.]).len(), 264);
    c.submit("-5").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    assert!((measures(&c, id).volume_mm3 - 80. * std::f64::consts::PI).abs() < 1e-6);
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
}

#[test]
fn solid_tube_both_sides_matches_signed_preview_and_undo() {
    for height in [-5., 5.] {
        let mut c = controller();
        c.set_cplane_from_three_points([0., 0., 0.], [0., 1., 0.], [0., 0., 1.])
            .unwrap();
        let before = c.document().clone();
        c.submit("Tube").unwrap();
        c.submit("BothSides=Yes").unwrap();
        assert!(c.submit("3 1 5").is_err());
        c.accept_world_point([10., 20., 30.]).unwrap();
        c.submit("3").unwrap();
        c.submit("1").unwrap();
        let edges = c.solid_preview([10. + height, 20., 30.]);
        assert_eq!(edges.len(), 264);
        for edge in edges {
            for p in edge {
                assert!(
                    (p[0] - (10. + height)).abs() < 1e-8 || (p[0] - (10. - height)).abs() < 1e-8
                );
            }
        }
        assert_eq!(c.document(), &before);
        assert_eq!(c.coordinator().undo_len(), 0);
        c.submit(&height.to_string()).unwrap();
        let id = *c.document().entities.keys().next().unwrap();
        let e = &c.document().entities[&id];
        let a = e.placement.point([0., 0., 0.]).unwrap();
        let b = e.placement.point([0., 0., 10.]).unwrap();
        assert!((a[0] - (10. - height)).abs() < 1e-8 && (b[0] - (10. + height)).abs() < 1e-8);
        assert!((measures(&c, id).volume_mm3 - 80. * std::f64::consts::PI).abs() < 1e-6);
        c.undo().unwrap();
        assert_eq!(c.document(), &before);
    }
}

#[test]
fn solid_tube_tangent_point_both_sides_and_stale_source() {
    use om9_core::geometry::Curve;
    for side in [-1., 1.] {
        let rail = Entity::new(
            "rail",
            Geometry::Curve(Curve::Line {
                start: [0., 0., 0.],
                end: [10., 0., 0.],
                domain: [0., 1.],
            }),
            None,
        );
        let id = rail.id;
        let mut doc = Document::new("tangent tube");
        doc.entities.insert(id, rail);
        let mut c = Controller::new(doc, Limits::default()).unwrap();
        let before = c.document().clone();
        c.submit("Tube Tangent").unwrap();
        assert!(c.solid_pipe_needs_curve());
        c.solid_pipe_pick_curve(id).unwrap();
        c.submit("OnCurve=0.5").unwrap();
        assert_eq!(c.prompt(), "Choose Point for a non-tangent constraint");
        c.submit("Point").unwrap();
        assert!(c.accept_world_point([8., 0., 0.]).is_err());
        c.accept_world_point([8., 2. * side, 0.]).unwrap();
        assert_eq!(c.prompt(), "Second radius or point");
        c.submit("1").unwrap();
        c.submit("5").unwrap();
        let e = c.document().entities.values().find(|e| e.id != id).unwrap();
        let center = e.placement.point([0., 0., 0.]).unwrap();
        assert!((center[0] - 5.).abs() < 1e-8 && (center[1] - side * 3.25).abs() < 1e-8);
        assert!(
            (measures(&c, e.id).volume_mm3 - std::f64::consts::PI * (3.25_f64.powi(2) - 1.) * 5.)
                .abs()
                < 1e-6
        );
        c.undo().unwrap();
        assert_eq!(c.document(), &before);
    }
    let rail = Entity::new(
        "rail",
        Geometry::Curve(Curve::Line {
            start: [0., 0., 0.],
            end: [10., 0., 0.],
            domain: [0., 1.],
        }),
        None,
    );
    let id = rail.id;
    let mut doc = Document::new("stale tangent tube");
    doc.entities.insert(id, rail);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("Tube Tangent").unwrap();
    c.submit("OnCurve=0.5").unwrap();
    c.submit("Point").unwrap();
    c.accept_world_point([8., 2., 0.]).unwrap();
    c.submit("1").unwrap();
    c.delete_objects(&[id]).unwrap();
    let after = c.document().clone();
    let n = c.coordinator().undo_len();
    assert!(c.solid_preview([5., 3.25, 5.]).is_empty());
    assert!(c.submit("5").is_err());
    assert_eq!(c.document(), &after);
    assert_eq!(c.coordinator().undo_len(), n);
}

#[test]
fn solid_tube_around_curve_orients_and_rejects_stale_source() {
    use om9_core::geometry::{Curve, Transform};
    for stale in [false, true] {
        let mut rail = Entity::new(
            "rail",
            Geometry::Curve(Curve::Line {
                start: [0., 0., 0.],
                end: [10., 0., 0.],
                domain: [0., 1.],
            }),
            None,
        );
        rail.placement = Transform::translation([10., 20., 4.]);
        let id = rail.id;
        let mut doc = Document::new("tube around");
        doc.entities.insert(id, rail);
        let mut c = Controller::new(doc, Limits::default()).unwrap();
        let before = c.document().clone();
        c.submit("Tube AroundCurve").unwrap();
        assert!(c.solid_pipe_needs_curve());
        c.solid_pipe_pick_curve(id).unwrap();
        c.submit("OnCurve=0.5").unwrap();
        if stale {
            c.submit("3").unwrap();
        } else {
            c.submit(&format!("Area={}", 9. * std::f64::consts::PI))
                .unwrap();
        }
        c.submit("1").unwrap();
        assert_eq!(c.solid_preview([20., 20., 4.]).len(), 264);
        if stale {
            c.delete_objects(&[id]).unwrap();
            let after = c.document().clone();
            let count = c.coordinator().undo_len();
            assert!(c.solid_preview([20., 20., 4.]).is_empty());
            assert!(c.submit("5").is_err());
            assert_eq!(c.document(), &after);
            assert_eq!(c.coordinator().undo_len(), count);
        } else {
            c.submit("5").unwrap();
            let e = c.document().entities.values().find(|e| e.id != id).unwrap();
            assert_eq!(e.placement.point([0., 0., 0.]).unwrap(), [15., 20., 4.]);
            assert_eq!(e.placement.point([0., 0., 5.]).unwrap(), [20., 20., 4.]);
            assert!((measures(&c, e.id).volume_mm3 - 40. * std::f64::consts::PI).abs() < 1e-6);
            c.undo().unwrap();
            assert_eq!(c.document(), &before);
        }
    }
}

#[test]
fn solid_tube_fit_points_direct_and_selected_with_undo() {
    use om9_core::geometry::Transform;
    for selected in [false, true] {
        let points = [[3., 0., 0.], [0., 3., 0.], [-3., 0., 0.], [0., -3., 0.]];
        let mut doc = Document::new("tube fit");
        let mut ids = Vec::new();
        if selected {
            for point in points {
                let mut e = Entity::new("fit point", Geometry::Point(point), None);
                e.placement = Transform::translation([10., 20., 4.]);
                ids.push(e.id);
                doc.entities.insert(e.id, e);
            }
        }
        let mut c = Controller::new(doc, Limits::default()).unwrap();
        if selected {
            c.select_objects(&ids).unwrap();
        }
        let before = c.document().clone();
        c.submit("Tube FitPoints").unwrap();
        if !selected {
            for p in points {
                c.accept_world_point([p[0] + 10., p[1] + 20., 4.]).unwrap();
            }
        }
        c.submit("").unwrap();
        assert_eq!(c.prompt(), "Second radius or point");
        c.submit("Undo").unwrap();
        assert_eq!(c.prompt(), "Add fit points or Enter to fit");
        c.submit("").unwrap();
        c.submit("1").unwrap();
        c.submit("5").unwrap();
        let e = c
            .document()
            .entities
            .values()
            .find(|e| !ids.contains(&e.id))
            .unwrap();
        assert!((measures(&c, e.id).volume_mm3 - 40. * std::f64::consts::PI).abs() < 1e-6);
        c.undo().unwrap();
        assert_eq!(c.document(), &before);
    }
}

#[test]
fn solid_tube_fit_points_invalid_is_atomic() {
    let mut c = controller();
    let before = c.document().clone();
    c.submit("Tube FitPoints").unwrap();
    for p in [[0., 0., 0.], [1., 0., 0.], [2., 0., 0.]] {
        c.accept_world_point(p).unwrap();
    }
    assert!(c.submit("").is_err());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), 0);
    c.submit("Undo").unwrap();
    c.accept_world_point([0., 2., 0.]).unwrap();
    c.submit("").unwrap();
    assert_eq!(c.prompt(), "Second radius or point");
}

#[test]
fn solid_tube_preview_tracks_both_radii_and_signed_height() {
    let mut c = controller();
    let before = c.document().clone();
    c.submit("Tube").unwrap();
    c.accept_world_point([2., 3., 4.]).unwrap();
    assert_eq!(c.solid_preview([5., 3., 4.]).len(), 64);
    c.submit("3").unwrap();
    assert_eq!(c.solid_preview([3., 3., 4.]).len(), 128);
    c.submit("1").unwrap();
    let edges = c.solid_preview([2., 3., -1.]);
    assert_eq!(edges.len(), 264);
    for edge in edges {
        for p in edge {
            let radius = (p[0] - 2.).hypot(p[1] - 3.);
            assert!((radius - 1.).abs() < 1e-8 || (radius - 3.).abs() < 1e-8);
            assert!((p[2] - 4.).abs() < 1e-8 || (p[2] + 1.).abs() < 1e-8);
        }
    }
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), 0);
    c.submit("-5").unwrap();
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
}

#[test]
fn solid_tube_three_point_radius_selects_both_sides_and_undo() {
    for side in [-1., 1.] {
        let mut c = controller();
        let before = c.document().clone();
        c.submit("Tube 3Point").unwrap();
        c.accept_world_point([0., 0., 0.]).unwrap();
        c.accept_world_point([4., 0., 0.]).unwrap();
        assert!(c.solid_option_tokens().iter().any(|(_, v)| *v == "Radius="));
        assert!(c.submit("Radius=1").is_err());
        c.submit("Radius=3").unwrap();
        assert_eq!(c.prompt(), "Choose base circle side");
        c.accept_world_point([2., side, 0.]).unwrap();
        assert_eq!(c.prompt(), "Second radius or point");
        c.submit("Undo").unwrap();
        assert_eq!(c.prompt(), "Choose base circle side");
        c.accept_world_point([2., side, 0.]).unwrap();
        c.submit("1").unwrap();
        c.submit("5").unwrap();
        let id = *c.document().entities.keys().next().unwrap();
        let e = &c.document().entities[&id];
        let center = e.placement.point([0., 0., 0.]).unwrap();
        assert!((center[1] - side * 5.0_f64.sqrt()).abs() < 1e-8);
        assert!((measures(&c, id).volume_mm3 - 40. * std::f64::consts::PI).abs() < 1e-6);
        c.undo().unwrap();
        assert_eq!(c.document(), &before);
    }
}

#[test]
fn solid_tube_reversed_radii_diameter_and_negative_height() {
    let mut c = controller();
    let before = c.document().clone();
    c.submit("Tube").unwrap();
    c.submit("Diameter").unwrap();
    c.accept_world_point([2., 3., 4.]).unwrap();
    c.submit("2").unwrap();
    assert!(c.submit("2").is_err());
    assert_eq!(c.prompt(), "Second radius or point");
    c.submit("6").unwrap();
    c.submit("-5").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    let e = &c.document().entities[&id];
    assert_eq!(e.placement.point([0., 0., 5.]).unwrap(), [2., 3., -1.]);
    assert!((measures(&c, id).volume_mm3 - 40. * std::f64::consts::PI).abs() < 1e-6);
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
}

#[test]
fn solid_cylinder_center_and_two_point_with_open_option() {
    use om9_app::session::InvocationOrigin;
    for route in 0..3 {
        let mut c = controller();
        c.set_cplane_from_three_points([0., 0., 0.], [0., 1., 0.], [0., 0., 1.])
            .unwrap();
        match route {
            0 => {
                c.dispatch("OM9-SOLID-016", InvocationOrigin::Menu).unwrap();
            }
            1 => {
                c.dispatch("Cylinder2Point", InvocationOrigin::Menu)
                    .unwrap();
            }
            _ => {
                c.submit("_Cylinder _P").unwrap();
            }
        }
        if route == 2 {
            c.submit("Solid=No").unwrap();
        }
        let first = if route == 0 {
            [5., 10., 15.]
        } else {
            [5., 8., 15.]
        };
        c.accept_world_point(first).unwrap();
        assert!(c.accept_world_point(first).is_err());
        c.accept_world_point([5., 12., 15.]).unwrap();
        assert_eq!(c.prompt(), "Height or point");
        assert!(c.accept_world_point([5., 10., 15.]).is_err());
        c.accept_world_point([2., 10., 15.]).unwrap();
        let entity = c.document().entities.values().next().unwrap();
        let id = entity.id;
        assert_eq!(
            entity.placement.point([0., 0., 0.]).unwrap(),
            [5., 10., 15.]
        );
        assert_eq!(
            entity.placement.point([0., 0., 3.]).unwrap(),
            [2., 10., 15.]
        );
        let m = measures(&c, id);
        if route == 2 {
            assert!(!m.closed);
            assert_eq!((m.solids, m.faces, m.naked_edges), (0, 1, 2));
        } else {
            assert!(m.closed);
            assert!((m.volume_mm3 - 12. * std::f64::consts::PI).abs() < 1e-6);
        }
        assert_eq!(c.coordinator().undo_len(), 1);
        c.undo().unwrap();
        assert!(c.document().entities.is_empty());
    }
}

#[test]
fn solid_cylinder_direct_dimensions_preserve_open_option() {
    let mut c = controller();
    c.submit("Cylinder").unwrap();
    c.submit("Solid=No").unwrap();
    assert!(c.submit("2 10").is_err());
    assert!(c.document().entities.is_empty());
    assert_eq!(c.coordinator().undo_len(), 0);
    c.accept_world_point([0., 0., 0.]).unwrap();
    c.submit("2").unwrap();
    c.submit("10").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    let m = measures(&c, id);
    assert_eq!((m.solids, m.faces, m.naked_edges), (0, 1, 2));
    c.undo().unwrap();
    c.submit("Cylinder 2 10").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    assert!(measures(&c, id).closed);
}

#[test]
fn solid_cylinder_three_point_and_radius_branches() {
    for branch in [0, 1, 2] {
        let mut c = controller();
        c.submit(if branch == 0 {
            "_Cylinder _3Point"
        } else {
            "Cylinder"
        })
        .unwrap();
        if branch != 0 {
            c.submit("3Point").unwrap();
        }
        c.accept_world_point([-2., 0., 0.]).unwrap();
        c.accept_world_point([2., 0., 0.]).unwrap();
        assert!(c.accept_world_point([0., 0., 0.]).is_err());
        assert_eq!(c.prompt(), "Third point on base circle");
        if branch == 0 {
            c.accept_world_point([0., 2., 0.]).unwrap();
        } else {
            c.submit("Radius=3").unwrap();
            assert_eq!(c.prompt(), "Choose base circle side");
            c.accept_world_point([0., if branch == 1 { 1. } else { -1. }, 0.])
                .unwrap();
        }
        assert_eq!(c.prompt(), "Height or point");
        if branch == 2 {
            c.submit("Solid=No").unwrap();
        }
        c.submit("5").unwrap();
        let id = *c.document().entities.keys().next().unwrap();
        let m = measures(&c, id);
        if branch == 2 {
            assert_eq!((m.solids, m.faces, m.naked_edges), (0, 1, 2));
        } else {
            let radius = if branch == 0 { 2. } else { 3. };
            assert!((m.volume_mm3 - std::f64::consts::PI * radius * radius * 5.).abs() < 1e-6);
        }
        c.undo().unwrap();
        assert!(c.document().entities.is_empty());
    }
}

#[test]
fn solid_cylinder_around_curve_uses_tangent_and_preserves_source() {
    use om9_core::geometry::{Curve, Transform};
    for circle in [false, true] {
        let curve = if circle {
            om9_core::curve_build::circle_on_frame([0., 0., 0.], [1., 0., 0.], [0., 1., 0.], 5.)
                .unwrap()
        } else {
            Curve::Line {
                start: [0., 0., 0.],
                end: [10., 0., 0.],
                domain: [0., 1.],
            }
        };
        let mut rail = Entity::new("rail", Geometry::Curve(curve), None);
        rail.placement = Transform::translation([10., 20., 4.]);
        let id = rail.id;
        let mut doc = Document::new("cylinder around");
        doc.entities.insert(id, rail);
        let mut c = Controller::new(doc, Limits::default()).unwrap();
        let before = c.document().clone();
        c.submit("Cylinder AroundCurve").unwrap();
        assert!(c.solid_pipe_needs_curve());
        c.solid_pipe_pick_curve(id).unwrap();
        c.submit(if circle {
            "OnCurve=0.25"
        } else {
            "OnCurve=0.5"
        })
        .unwrap();
        c.submit("2").unwrap();
        c.submit("5").unwrap();
        let entity = c.document().entities.values().find(|e| e.id != id).unwrap();
        let a = entity.placement.point([0., 0., 0.]).unwrap();
        let b = entity.placement.point([0., 0., 5.]).unwrap();
        let expected = if circle {
            [10., 25., 4.]
        } else {
            [15., 20., 4.]
        };
        assert!((0..3).all(|i| (a[i] - expected[i]).abs() < 1e-7));
        assert!((b[0] - a[0] - if circle { -5. } else { 5. }).abs() < 1e-7);
        assert!((measures(&c, entity.id).volume_mm3 - 20. * std::f64::consts::PI).abs() < 1e-6);
        c.undo().unwrap();
        assert_eq!(c.document(), &before);
    }
}

#[test]
fn solid_cylinder_around_curve_rejects_deleted_source() {
    use om9_core::geometry::Curve;
    let rail = Entity::new(
        "rail",
        Geometry::Curve(Curve::Line {
            start: [0., 0., 0.],
            end: [10., 0., 0.],
            domain: [0., 1.],
        }),
        None,
    );
    let id = rail.id;
    let mut doc = Document::new("stale cylinder");
    doc.entities.insert(id, rail);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("Cylinder AroundCurve").unwrap();
    c.accept_world_point([5., 1., 0.]).unwrap();
    c.submit("2").unwrap();
    assert!(!c.solid_preview([10., 0., 0.]).is_empty());
    c.delete_objects(&[id]).unwrap();
    let after = c.document().clone();
    let count = c.coordinator().undo_len();
    assert!(c.solid_preview([10., 0., 0.]).is_empty());
    assert!(c.submit("5").is_err());
    assert_eq!(c.document(), &after);
    assert_eq!(c.coordinator().undo_len(), count);
}

#[test]
fn solid_cylinder_fit_points_direct_and_selected_sources() {
    use om9_core::geometry::Transform;
    for selected in [false, true] {
        let points = [[3., 0., 0.], [0., 3., 0.], [-3., 0., 0.], [0., -3., 0.]];
        let mut doc = Document::new("cylinder fit points");
        let mut ids = Vec::new();
        if selected {
            for p in points {
                let mut entity = Entity::new("fit point", Geometry::Point(p), None);
                entity.placement = Transform::translation([10., 20., 4.]);
                ids.push(entity.id);
                doc.entities.insert(entity.id, entity);
            }
        }
        let mut c = Controller::new(doc, Limits::default()).unwrap();
        if selected {
            c.select_objects(&ids).unwrap();
        }
        let before = c.document().clone();
        c.submit("Cylinder FitPoints").unwrap();
        if !selected {
            for p in points {
                c.accept_world_point([p[0] + 10., p[1] + 20., 4.]).unwrap();
            }
        }
        c.submit("").unwrap();
        assert_eq!(c.prompt(), "Height or point");
        c.submit("5").unwrap();
        let entity = c
            .document()
            .entities
            .values()
            .find(|e| !ids.contains(&e.id))
            .unwrap();
        let m = measures(&c, entity.id);
        assert!((m.volume_mm3 - 45. * std::f64::consts::PI).abs() < 1e-6);
        c.undo().unwrap();
        assert_eq!(c.document(), &before);
    }
}

#[test]
fn solid_cylinder_fit_points_invalid_and_limit_are_atomic() {
    let mut c = controller();
    let before = c.document().clone();
    c.submit("Cylinder FitPoints").unwrap();
    for p in [[0., 0., 0.], [1., 0., 0.], [2., 0., 0.]] {
        c.accept_world_point(p).unwrap();
    }
    assert!(c.submit("").is_err());
    assert_eq!(c.prompt(), "Add fit points or Enter to fit");
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), 0);
    c.submit("Undo").unwrap();
    c.accept_world_point([0., 2., 0.]).unwrap();
    c.submit("").unwrap();
    c.submit("Solid=No").unwrap();
    c.submit("5").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    let m = measures(&c, id);
    assert_eq!((m.solids, m.faces, m.naked_edges), (0, 1, 2));
    let limits = Limits {
        max_points: 3,
        ..Limits::default()
    };
    let mut c = Controller::new(Document::new("fit limit"), limits).unwrap();
    let before = c.document().clone();
    c.submit("Cylinder FitPoints").unwrap();
    for p in [[3., 0., 0.], [0., 3., 0.], [-3., 0., 0.]] {
        c.accept_world_point(p).unwrap();
    }
    assert!(c.accept_world_point([0., -3., 0.]).is_err());
    assert!(c.submit("").is_err());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), 0);
}

#[test]
fn solid_cylinder_tangent_point_both_sides_and_stale_source() {
    use om9_core::geometry::Curve;
    for sign in [-1., 1.] {
        let rail = Entity::new(
            "rail",
            Geometry::Curve(Curve::Line {
                start: [0., 0., 0.],
                end: [10., 0., 0.],
                domain: [0., 1.],
            }),
            None,
        );
        let id = rail.id;
        let mut doc = Document::new("tangent cylinder");
        doc.entities.insert(id, rail);
        let mut c = Controller::new(doc, Limits::default()).unwrap();
        let before = c.document().clone();
        c.submit("Cylinder Tangent").unwrap();
        assert!(c.solid_pipe_needs_curve());
        c.solid_pipe_pick_curve(id).unwrap();
        c.submit("OnCurve=0.5").unwrap();
        assert_eq!(c.prompt(), "Choose Point for a non-tangent constraint");
        c.submit("Point").unwrap();
        assert!(c.accept_world_point([8., 0., 0.]).is_err());
        c.accept_world_point([8., 2. * sign, 0.]).unwrap();
        assert_eq!(c.prompt(), "Height or point");
        c.submit("5").unwrap();
        let entity = c.document().entities.values().find(|e| e.id != id).unwrap();
        let center = entity.placement.point([0., 0., 0.]).unwrap();
        assert!((center[0] - 5.).abs() < 1e-8 && (center[1] - sign * 3.25).abs() < 1e-8);
        assert!(
            (measures(&c, entity.id).volume_mm3 - std::f64::consts::PI * 3.25_f64.powi(2) * 5.)
                .abs()
                < 1e-6
        );
        c.undo().unwrap();
        assert_eq!(c.document(), &before);
    }
    let rail = Entity::new(
        "rail",
        Geometry::Curve(Curve::Line {
            start: [0., 0., 0.],
            end: [10., 0., 0.],
            domain: [0., 1.],
        }),
        None,
    );
    let id = rail.id;
    let mut doc = Document::new("stale cylinder tangent");
    doc.entities.insert(id, rail);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("Cylinder Tangent").unwrap();
    c.submit("OnCurve=0.5").unwrap();
    c.submit("Point").unwrap();
    c.accept_world_point([8., 2., 0.]).unwrap();
    c.delete_objects(&[id]).unwrap();
    let after = c.document().clone();
    let count = c.coordinator().undo_len();
    assert!(c.submit("5").is_err());
    assert_eq!(c.document(), &after);
    assert_eq!(c.coordinator().undo_len(), count);
}

#[test]
fn solid_cylinder_wire_preview_matches_signed_height_and_radius() {
    for (two_point, sign, open) in [(false, 1., false), (true, -1., true)] {
        let mut c = controller();
        c.submit("Cylinder").unwrap();
        if two_point {
            c.submit("2Point").unwrap();
        }
        if open {
            c.submit("Solid=No").unwrap();
        }
        if two_point {
            c.accept_world_point([8., 20., 30.]).unwrap();
            c.accept_world_point([12., 20., 30.]).unwrap();
        } else {
            c.accept_world_point([10., 20., 30.]).unwrap();
            c.accept_world_point([12., 20., 30.]).unwrap();
        }
        let wire = c.solid_preview([10., 20., 30. + sign * 5.]);
        assert_eq!(wire.len(), 132);
        for [a, b] in wire {
            for p in [a, b] {
                let radius = (p[0] - 10.).hypot(p[1] - 20.);
                assert!((radius - 2.).abs() < 1e-9);
                assert!((p[2] - 30.).abs() < 1e-9 || (p[2] - (30. + sign * 5.)).abs() < 1e-9);
            }
        }
        assert_eq!(c.coordinator().undo_len(), 0);
        assert!(c.document().entities.is_empty());
        c.accept_world_point([10., 20., 30. + sign * 5.]).unwrap();
        let id = *c.document().entities.keys().next().unwrap();
        let m = measures(&c, id);
        if open {
            assert_eq!((m.solids, m.faces, m.naked_edges), (0, 1, 2));
        } else {
            assert!((m.volume_mm3 - 20. * std::f64::consts::PI).abs() < 1e-6);
        }
        c.undo().unwrap();
        assert!(c.document().entities.is_empty());
    }
}

#[test]
#[ignore = "requires an ellipsoid .om9 saved by the native UI; set OM9_ELLIPSOID_UI_FIXTURE"]
fn solid_ellipsoid_native_ui_saved_fixture() {
    let path = std::env::var_os("OM9_ELLIPSOID_UI_FIXTURE").expect("native UI fixture path");
    let limits = Limits::default();
    let restored = om9_archive::Archive::open_restored(
        std::path::Path::new(&path),
        &limits,
        &Default::default(),
    )
    .unwrap();
    assert_eq!(restored.document.entities.len(), 1);
    let entity = restored.document.entities.values().next().unwrap();
    let Geometry::Brep(brep) = &entity.geometry else {
        panic!("native UI must store B-Rep");
    };
    let measured = creation::measure(brep, &limits).unwrap();
    assert_eq!(measured.solids, 1);
    assert!(measured.closed);
    assert_eq!(measured.naked_edges, 0);
    let expected = 4.0 * std::f64::consts::PI / 3.0 * 24.0;
    assert!((measured.volume_mm3 / expected - 1.0).abs() < 1e-10);
    println!(
        "native_ui_saved_ellipsoid_volume={:.12} solids={} naked_edges={}",
        measured.volume_mm3, measured.solids, measured.naked_edges
    );
}

#[test]
fn solid_boolean_menu_commands_commit_one_transaction() {
    let mut c = controller();
    c.submit("Box 10 10 10").unwrap();
    c.submit("Box 10 10 10").unwrap();
    let ids: Vec<_> = c.document().entities.keys().copied().collect();
    c.select_objects(&[ids[1]]).unwrap();
    c.submit("Move").unwrap();
    c.accept_world_point([0.0; 3]).unwrap();
    c.accept_world_point([5.0, 0.0, 0.0]).unwrap();
    c.select_objects(&ids).unwrap();
    let undo_before = c.coordinator().undo_len();
    c.submit("BooleanUnion").unwrap();
    c.submit("DeleteInput=Yes").unwrap();
    c.submit("").unwrap();
    assert_eq!(c.document().entities.len(), 1);
    assert_eq!(c.coordinator().undo_len(), undo_before + 1);
    let result = *c.document().entities.keys().next().unwrap();
    let measured = measures(&c, result);
    assert_eq!(measured.solids, 1);
    assert!(
        (measured.volume_mm3 - 1500.0).abs() < 1e-6,
        "{}",
        measured.volume_mm3
    );
    c.undo().unwrap();
    assert_eq!(c.document().entities.len(), 2);
    println!("solid_boolean_union_volume={:.6}", measured.volume_mm3);
}

#[test]
fn solid_extrude_pipe_extract_and_fillet_dispatch() {
    let mut doc = Document::new("solid-ops");
    let rectangle =
        om9_core::curve_build::rectangle([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 10.0, 10.0)
            .unwrap();
    let curve = Entity::new("rectangle", Geometry::Curve(rectangle), None);
    let curve_id = curve.id;
    doc.entities.insert(curve_id, curve);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[curve_id]).unwrap();
    c.submit("ExtrudeCrv 5").unwrap();
    let extrude_id = *c
        .document()
        .entities
        .keys()
        .find(|id| **id != curve_id)
        .unwrap();
    let measured = measures(&c, extrude_id);
    assert!(
        (measured.volume_mm3 - 500.0).abs() < 5.0,
        "{}",
        measured.volume_mm3
    );
    println!("solid_extrude_volume={:.6}", measured.volume_mm3);

    let mut c = controller();
    c.submit("Box 10 10 10").unwrap();
    let box_id = *c.document().entities.keys().next().unwrap();
    let before = c.document().entities.len();
    c.select_objects(&[box_id]).unwrap();
    c.submit("ExtractSrf 0").unwrap();
    assert_eq!(c.document().entities.len(), before + 1);
    c.undo().unwrap();
    c.select_objects(&[box_id]).unwrap();
    c.submit("FilletEdge 0 1").unwrap();
    let measured = measures(&c, box_id);
    assert!(measured.volume_mm3 > 0.0);
    println!("solid_fillet_volume={:.6}", measured.volume_mm3);

    let mut c = controller();
    c.submit("Sphere 5").unwrap();
    let box_id = *c.document().entities.keys().next().unwrap();
    c.select_objects(&[box_id]).unwrap();
    let before = c.document().entities.len();
    c.submit("ExtractSrf 0 Copy=Yes").unwrap();
    assert_eq!(c.document().entities.len(), before + 1);
}

#[test]
fn solid_menu_usage_and_invalid_input_are_visible_errors() {
    let mut c = controller();
    assert!(c.submit("Box").is_ok());
    assert!(!c.prompt().is_empty());
    c.submit("0,0,0").unwrap();
    assert!(c.prompt().starts_with("Other corner of base"));
    c.submit("10,20,0").unwrap();
    assert!(c.prompt().starts_with("Height"));
    c.submit("30").unwrap();
    assert_eq!(c.document().entities.len(), 1);
    c.cancel();
    assert!(c.submit("Box 1 2").is_err());
    assert!(c.submit("Box 1 2 -3").is_err());
    c.submit("BooleanUnion").unwrap();
    assert!(c.submit("").is_err());
    c.cancel();
    assert_eq!(c.document().entities.len(), 1);
    c.submit("Box 2 2 2").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    c.select_objects(&[id]).unwrap();
    assert!(c.submit("ExtractSrf 99").is_err());
    assert!(c.submit("FilletEdge 99 1").is_err());
}

#[test]
fn solid_shell_removes_one_face_and_stays_closed() {
    let mut c = controller();
    c.submit("Box 10 10 10").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("Shell 0 1").unwrap();
    let measured = measures(&c, id);
    assert_eq!(measured.solids, 1);
    assert!(measured.closed);
    assert!(measured.volume_mm3 > 0.0 && measured.volume_mm3 < 1000.0);
    println!("solid_shell_volume={:.6}", measured.volume_mm3);
}

#[test]
fn solid_text_builds_exact_outline_solids() {
    let mut c = controller();
    c.submit("Text H 5").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    let measured = measures(&c, id);
    assert!(measured.solids >= 1);
    assert!(measured.closed);
    assert!(measured.volume_mm3 > 0.0);
    println!("solid_text_volume={:.6}", measured.volume_mm3);
}

#[test]
fn solid_box_uses_the_freecad_point_prompts() {
    let mut c = controller();
    c.submit("Box").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([10.0, 20.0, 0.0]).unwrap();
    assert!(c.prompt().starts_with("Height"));
    c.submit("30").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    let measured = measures(&c, id);
    assert!((measured.volume_mm3 - 6000.0).abs() < 1e-6);
    assert_eq!(
        c.document().entities[&id].placement,
        om9_core::geometry::Transform::translation([0.0, 0.0, 0.0])
    );
}

#[test]
fn solid_box_center_mode_uses_the_center_and_doubles_the_base() {
    let mut c = controller();
    c.submit("Box").unwrap();
    c.submit("Center").unwrap();
    assert!(c.prompt().starts_with("Center of base"));
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([10.0, 10.0, 0.0]).unwrap();
    c.submit("20").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    let measured = measures(&c, id);
    assert!((measured.volume_mm3 - 8000.0).abs() < 1e-6);
    assert_eq!(
        c.document().entities[&id].placement,
        om9_core::geometry::Transform::translation([-10.0, -10.0, 0.0])
    );
}

#[test]
fn solid_box_signed_height_preview_and_commit_match_world_xy() {
    for center in [false, true] {
        for typed_dimensions in [false, true] {
            if typed_dimensions && !center {
                continue;
            }
            for sign in [-1., 1.] {
                let mut c = controller();
                let before = c.document().clone();
                c.submit("Box").unwrap();
                if center {
                    c.submit("Center").unwrap();
                }
                c.accept_world_point([10., 20., 3.]).unwrap();
                if typed_dimensions {
                    c.submit("8").unwrap();
                    c.submit("6").unwrap();
                } else {
                    c.accept_world_point([14., 23., 3.]).unwrap();
                }
                let pointer = [10., 20., 3. + sign * 5.];
                let wire = c.solid_preview(pointer);
                assert_eq!(wire.len(), 12);
                let zs: Vec<_> = wire.iter().flat_map(|[a, b]| [a[2], b[2]]).collect();
                assert!(
                    (zs.iter().copied().fold(f64::INFINITY, f64::min) - (3. + (sign * 5.).min(0.)))
                        .abs()
                        < 1e-8
                );
                assert!(
                    (zs.iter().copied().fold(f64::NEG_INFINITY, f64::max)
                        - (3. + (sign * 5.).max(0.)))
                    .abs()
                        < 1e-8
                );
                assert!(c.accept_world_point([10., 20., 3.]).is_err());
                assert_eq!(c.document(), &before);
                c.accept_world_point(pointer).unwrap();
                let e = c.document().entities.values().next().unwrap();
                let min = e.placement.point([0., 0., 0.]).unwrap();
                assert!((min[2] - (3. + (sign * 5.).min(0.))).abs() < 1e-8);
                let m = measures(&c, e.id);
                let area = if center { 48. } else { 12. };
                assert!((m.volume_mm3 - area * 5.).abs() < 1e-6);
                c.undo().unwrap();
                assert_eq!(c.document(), &before);
            }
        }
    }
}

#[test]
fn solid_box_rotated_cplane_is_captured_for_preview_and_commit() {
    for center in [false, true] {
        for dimensions in [false, true] {
            if dimensions && !center {
                continue;
            }
            for sign in [-1., 1.] {
                let mut c = controller();
                c.set_cplane_from_three_points([0., 0., 0.], [0., 1., 0.], [0., 0., 1.])
                    .unwrap();
                let before = c.document().clone();
                c.submit("Box").unwrap();
                if center {
                    c.submit("Center").unwrap();
                }
                c.accept_world_point([10., 20., 30.]).unwrap();
                if dimensions {
                    c.submit("8").unwrap();
                    c.submit("6").unwrap();
                } else {
                    c.accept_world_point([10., 24., 33.]).unwrap();
                }
                let p = [10. + sign * 5., 20., 30.];
                let wire = c.solid_preview(p);
                assert_eq!(wire.len(), 12);
                let xs: Vec<_> = wire.iter().flat_map(|[a, b]| [a[0], b[0]]).collect();
                assert!(
                    (xs.iter().copied().fold(f64::INFINITY, f64::min)
                        - (10. + (sign * 5.).min(0.)))
                    .abs()
                        < 1e-8
                );
                assert!(
                    (xs.iter().copied().fold(f64::NEG_INFINITY, f64::max)
                        - (10. + (sign * 5.).max(0.)))
                    .abs()
                        < 1e-8
                );
                c.accept_world_point(p).unwrap();
                let e = c.document().entities.values().next().unwrap();
                let origin = e.placement.point([0., 0., 0.]).unwrap();
                let top = e.placement.point([0., 0., 5.]).unwrap();
                assert!(
                    (origin[0] - (10. + (sign * 5.).min(0.))).abs() < 1e-8
                        && (top[0] - (10. + (sign * 5.).max(0.))).abs() < 1e-8
                );
                let expected = if center { 240. } else { 60. };
                assert!((measures(&c, e.id).volume_mm3 - expected).abs() < 1e-6);
                c.undo().unwrap();
                assert_eq!(c.document(), &before);
            }
        }
    }
}

#[test]
fn solid_box_rotated_cplane_cmd_coordinates_and_ray() {
    let mut c = controller();
    c.set_cplane_from_three_points([10., 20., 30.], [10., 21., 30.], [10., 20., 31.])
        .unwrap();
    c.submit("Box").unwrap();
    c.submit("0").unwrap();
    c.submit("4,3,0").unwrap();
    let ray = om9_scene::camera::Ray {
        origin: [15., 20., -100.],
        direction: [0., 0., 1.],
    };
    let p = c.box_height_ray_point(&ray).unwrap();
    assert!((p[0] - 15.).abs() < 1e-8);
    c.submit("5").unwrap();
    let e = c.document().entities.values().next().unwrap();
    let origin = e.placement.point([0., 0., 0.]).unwrap();
    assert_eq!(origin, [10., 20., 30.]);
    assert!((measures(&c, e.id).volume_mm3 - 60.).abs() < 1e-6);
}

#[test]
fn solid_box_numeric_dimensions_use_document_units_once() {
    use om9_core::geometry::Unit;
    for mode in ["default", "three", "center", "inline", "enter"] {
        let mut doc = Document::new("box centimetres");
        doc.page_unit = Unit::Centimeter;
        let mut c = Controller::new(doc, Limits::default()).unwrap();
        let before = c.document().clone();
        if mode == "inline" {
            c.submit("Box 1 2 3").unwrap();
        } else {
            c.submit("Box").unwrap();
            if mode == "three" {
                c.submit("3Point").unwrap();
            }
            if mode == "center" {
                c.submit("Center").unwrap();
            }
            c.submit("0").unwrap();
            match mode {
                "default" | "enter" => {
                    assert!(c.submit("3").is_err());
                    c.submit("1,2,0").unwrap();
                    if mode == "enter" {
                        c.submit("").unwrap();
                    } else {
                        c.submit("Height=3").unwrap();
                    }
                }
                "three" => {
                    c.submit("1,0,0").unwrap();
                    c.submit("Width=-2").unwrap();
                    c.submit("3").unwrap();
                }
                "center" => {
                    c.submit("Length=2").unwrap();
                    c.submit("Width=4").unwrap();
                    let preview = c.solid_preview([0., 0., -30.]);
                    assert_eq!(preview.len(), 12);
                    c.submit("Height=-3").unwrap();
                }
                _ => unreachable!(),
            }
        }
        let e = c.document().entities.values().next().unwrap();
        let expected = match mode {
            "center" => 24000.,
            "enter" => 4000.,
            _ => 6000.,
        };
        assert!(
            (measures(&c, e.id).volume_mm3 - expected).abs() < 1e-6,
            "{mode}"
        );
        if mode == "three" {
            assert_eq!(e.placement.point([0., 0., 0.]).unwrap()[1], -20.);
        }
        if mode == "center" {
            assert_eq!(e.placement.point([0., 0., 0.]).unwrap()[2], -30.);
        }
        c.undo().unwrap();
        assert_eq!(c.document(), &before);
    }
}

#[test]
fn solid_box_retries_bad_base_after_numeric_height() {
    for text in ["3", "Height=3"] {
        let mut c = controller();
        let before = c.document().clone();
        c.submit("Box").unwrap();
        c.submit("0").unwrap();
        c.submit("0,2,0").unwrap();
        assert!(c.submit(text).is_err());
        assert!(c.is_running());
        assert_eq!(c.document(), &before);
        assert_eq!(c.coordinator().undo_len(), 0);
        c.submit("Undo").unwrap();
        c.submit("1,2,0").unwrap();
        c.submit(text).unwrap();
        let id = *c.document().entities.keys().next().unwrap();
        assert!((measures(&c, id).volume_mm3 - 6.).abs() < 1e-6);
    }
}

#[test]
fn solid_box_numeric_routes_scale_for_all_document_units() {
    use om9_core::geometry::Unit;
    for unit in [
        Unit::Millimeter,
        Unit::Centimeter,
        Unit::Meter,
        Unit::Inch,
        Unit::Foot,
        Unit::Micrometer,
    ] {
        let expected = 6_000_000. * unit.factor().powi(3);
        for mode in ["inline", "3Point", "Center"] {
            let mut doc = Document::new("unit matrix");
            doc.page_unit = unit;
            let mut c = Controller::new(doc, Limits::default()).unwrap();
            if mode == "inline" {
                c.submit("Box 100 200 300").unwrap();
            } else {
                c.submit("Box").unwrap();
                c.submit(mode).unwrap();
                c.submit("0").unwrap();
                if mode == "3Point" {
                    c.submit("100,0,0").unwrap();
                    c.submit("Width=200").unwrap();
                } else {
                    c.submit("Length=100").unwrap();
                    c.submit("Width=200").unwrap();
                }
                c.submit("Height=300").unwrap();
            }
            let id = *c.document().entities.keys().next().unwrap();
            let actual = measures(&c, id).volume_mm3;
            assert!(
                (actual - expected).abs() / expected < 1e-8,
                "{unit:?} {mode}: {actual} vs {expected}"
            );
        }
    }
}

#[test]
fn solid_box_center_typed_dimensions_accept_negative_height() {
    let mut c = controller();
    c.submit("Box").unwrap();
    c.submit("Center").unwrap();
    c.accept_world_point([0., 0., 3.]).unwrap();
    c.submit("8").unwrap();
    c.submit("6").unwrap();
    assert!(c.submit("0").is_err());
    c.submit("-5").unwrap();
    let e = c.document().entities.values().next().unwrap();
    assert_eq!(e.placement.point([0., 0., 0.]).unwrap()[2], -2.);
}

#[test]
fn solid_box_rejects_mode_changes_after_the_first_point() {
    let mut c = controller();
    c.submit("Box").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    assert!(c.submit("Diagonal").is_err());
    assert_eq!(c.prompt(), "Other corner of base or length ( 3Point ) :");
}

#[test]
fn solid_box_three_point_mode_uses_edge_then_width_then_height() {
    let mut c = controller();
    c.submit("Box").unwrap();
    c.submit("3Point").unwrap();
    assert!(c.prompt().starts_with("Start of edge"));
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([10.0, 0.0, 0.0]).unwrap();
    assert!(c.prompt().starts_with("Width"));
    c.submit("5").unwrap();
    assert!(c.prompt().starts_with("Height"));
    c.submit("4").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    assert!((measures(&c, id).volume_mm3 - 200.0).abs() < 1e-6);
}

#[test]
fn solid_box_three_point_and_vertical_signed_picks_follow_cplane() {
    for vertical in [false, true] {
        for width_sign in [-1., 1.] {
            for height_sign in [-1., 1.] {
                let mut c = controller();
                c.set_cplane_from_three_points([0., 0., 0.], [0., 1., 0.], [0., 0., 1.])
                    .unwrap();
                let before = c.document().clone();
                c.submit("Box").unwrap();
                c.submit(if vertical { "Vertical" } else { "3Point" })
                    .unwrap();
                let first = [10., 20., 30.];
                let second = [10., 24., 30.];
                c.accept_world_point(first).unwrap();
                assert!(c.accept_world_point(first).is_err());
                c.accept_world_point(second).unwrap();
                let width_axis = if vertical { [1., 0., 0.] } else { [0., 0., 1.] };
                let height_axis = if vertical {
                    [0., 0., -1.]
                } else {
                    [1., 0., 0.]
                };
                let width_pick =
                    std::array::from_fn(|i| first[i] + width_axis[i] * width_sign * 3.);
                let height_pick =
                    std::array::from_fn(|i| first[i] + height_axis[i] * height_sign * 5.);
                assert_eq!(c.solid_preview(width_pick).len(), 12);
                c.accept_world_point(width_pick).unwrap();
                let wire = c.solid_preview(height_pick);
                assert_eq!(wire.len(), 12);
                let has_point = |target: [f64; 3]| {
                    wire.iter()
                        .flat_map(|[a, b]| [a, b])
                        .any(|p| (0..3).all(|i| (p[i] - target[i]).abs() < 1e-8))
                };
                assert!(has_point(first) && has_point(width_pick) && has_point(height_pick));
                c.accept_world_point(height_pick).unwrap();
                let e = c.document().entities.values().next().unwrap();
                assert!((measures(&c, e.id).volume_mm3 - 60.).abs() < 1e-6);
                let corners: Vec<_> = (0..8)
                    .map(|bits| {
                        e.placement
                            .point([
                                if bits & 1 != 0 { 4. } else { 0. },
                                if bits & 2 != 0 { 3. } else { 0. },
                                if bits & 4 != 0 { 5. } else { 0. },
                            ])
                            .unwrap()
                    })
                    .collect();
                for expected in [first, width_pick, height_pick] {
                    assert!(
                        corners
                            .iter()
                            .any(|p| (0..3).all(|i| (p[i] - expected[i]).abs() < 1e-8)),
                        "{vertical} {width_sign} {height_sign} {expected:?} {corners:?}"
                    );
                }
                c.undo().unwrap();
                assert_eq!(c.document(), &before);
            }
        }
    }
}

#[test]
fn solid_box_three_point_and_vertical_accept_signed_text_and_width_points() {
    for mode in ["3Point", "Vertical"] {
        for point in [false, true] {
            let mut c = controller();
            c.submit("Box").unwrap();
            c.submit(mode).unwrap();
            c.submit("0,0,0").unwrap();
            c.submit("4,0,0").unwrap();
            if point {
                c.submit(if mode == "Vertical" {
                    "0,0,-3"
                } else {
                    "0,-3,0"
                })
                .unwrap();
            } else {
                c.submit("Width=-3").unwrap();
            }
            c.submit("-5").unwrap();
            let e = c.document().entities.values().next().unwrap();
            assert!((measures(&c, e.id).volume_mm3 - 60.).abs() < 1e-6);
        }
    }
}

#[test]
fn solid_box_recovered_macros_and_variant_ids_share_geometry() {
    use om9_app::session::InvocationOrigin;
    for (mode, id, macro_text) in [
        ("Diagonal", "SolidBoxDiagonal", "_Box _Diagonal"),
        ("3Point", "SolidBox3PointsHeight", "_Box _3Point"),
        ("3Point", "SolidBox3PointsHeight", "_Box _P"),
        ("Vertical", "SolidBoxVerticalBase", "_Box _Vertical"),
        ("Center", "SolidBoxCenterofBaseCornerHeight", "_Box _Center"),
    ] {
        for from_menu in [false, true] {
            let mut c = controller();
            let before = c.document().clone();
            if from_menu {
                c.dispatch(id, InvocationOrigin::Menu).unwrap();
            } else {
                c.submit(macro_text).unwrap();
            }
            assert!(c.dispatch("Box3Point", InvocationOrigin::Menu).is_err());
            assert_eq!(c.coordinator().undo_len(), 0);
            c.accept_world_point([0., 0., 0.]).unwrap();
            if mode == "Diagonal" || mode == "Center" {
                c.accept_world_point([10., 5., 0.]).unwrap();
            } else {
                c.accept_world_point([10., 0., 0.]).unwrap();
                c.submit("5").unwrap();
            }
            c.submit("4").unwrap();
            let entity = c.document().entities.values().next().unwrap();
            let expected = if mode == "Center" { 800. } else { 200. };
            assert!(
                (measures(&c, entity.id).volume_mm3 - expected).abs() < 1e-6,
                "{mode} {from_menu}"
            );
            c.undo().unwrap();
            assert_eq!(c.document(), &before);
        }
    }
}

#[test]
fn solid_box_cube_mode_uses_the_diagonal_as_a_cube() {
    let mut c = controller();
    c.submit("Box").unwrap();
    c.submit("Cube").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([10.0, 10.0, 10.0]).unwrap();
    assert_eq!(c.coordinator().undo_len(), 1);
    let id = *c.document().entities.keys().next().unwrap();
    assert!((measures(&c, id).volume_mm3 - 1000.0).abs() < 1e-6);
}

#[test]
fn solid_box_cube_preview_matches_commit_on_rotated_plane_and_half_turn() {
    for delta in [[-4., -4., -4.], [-4., -4., -4.000001], [2., -3., 5.]] {
        let mut c = controller();
        c.set_cplane_from_three_points([0., 0., 0.], [0., 1., 0.], [0., 0., 1.])
            .unwrap();
        c.submit("Box").unwrap();
        c.submit("Cube").unwrap();
        let first = [10., 20., 30.];
        let target = [
            first[0] + delta[2],
            first[1] + delta[0],
            first[2] + delta[1],
        ];
        c.accept_world_point(first).unwrap();
        let preview = c.solid_preview(target);
        assert_eq!(preview.len(), 12);
        assert!(
            preview
                .iter()
                .flat_map(|[a, b]| [a, b])
                .any(|p| (0..3).all(|i| (p[i] - target[i]).abs() < 1e-7))
        );
        assert!(c.accept_world_point(first).is_err());
        assert_eq!(c.coordinator().undo_len(), 0);
        c.accept_world_point(target).unwrap();
        let e = c.document().entities.values().next().unwrap();
        let side = delta[0].hypot(delta[1]).hypot(delta[2]) / 3_f64.sqrt();
        for bits in 0..8 {
            let local = std::array::from_fn(|i| if bits & (1 << i) != 0 { side } else { 0. });
            let vertex = e.placement.point(local).unwrap();
            assert!(
                preview
                    .iter()
                    .flat_map(|[a, b]| [a, b])
                    .any(|p| (0..3).all(|i| (p[i] - vertex[i]).abs() < 1e-7)),
                "{delta:?} {vertex:?}"
            );
        }
        assert!((measures(&c, e.id).volume_mm3 - side.powi(3)).abs() < 1e-6);
    }
}

#[test]
fn solid_box_edge_preview_uses_captured_plane_projection() {
    for mode in ["3Point", "Vertical"] {
        let mut c = controller();
        c.set_cplane_from_three_points([0., 0., 0.], [0., 1., 0.], [0., 0., 1.])
            .unwrap();
        c.submit("Box").unwrap();
        c.submit(mode).unwrap();
        c.accept_world_point([10., 20., 30.]).unwrap();
        let preview = c.solid_preview([15., 24., 33.]);
        assert_eq!(preview, vec![[[10., 20., 30.], [10., 24., 33.]]]);
        c.accept_world_point([15., 24., 33.]).unwrap();
        c.submit("3").unwrap();
        c.submit("5").unwrap();
        let e = c.document().entities.values().next().unwrap();
        assert!((measures(&c, e.id).volume_mm3 - 75.).abs() < 1e-6);
    }
}

#[test]
fn solid_box_vertical_mode_uses_a_vertical_edge() {
    let mut c = controller();
    c.submit("Box").unwrap();
    c.submit("Vertical").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([10.0, 0.0, 0.0]).unwrap();
    c.submit("5").unwrap();
    c.submit("4").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    assert!((measures(&c, id).volume_mm3 - 200.0).abs() < 1e-6);
}

#[test]
fn solid_sphere_center_then_radius_point_matches_freecad() {
    let mut c = controller();
    c.submit("Sphere").unwrap();
    assert!(c.prompt().starts_with("Center of sphere"));
    c.accept_world_point([1.0, 2.0, 3.0]).unwrap();
    assert!(c.prompt().starts_with("Radius"));
    c.accept_world_point([4.0, 2.0, 3.0]).unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    let measured = measures(&c, id);
    assert!((measured.volume_mm3 - 4.0 * std::f64::consts::PI / 3.0 * 27.0).abs() < 1e-6);
    assert_eq!(
        c.document().entities[&id].placement,
        om9_core::geometry::Transform::translation([1.0, 2.0, 3.0])
    );
}

#[test]
fn solid_sphere_recovered_macros_and_variant_ids_share_modes() {
    use om9_app::session::InvocationOrigin;
    for (id, mac, prompt) in [
        (
            "SolidSphere2Points",
            "_Sphere _2Point",
            "First diameter endpoint",
        ),
        (
            "SolidSphere2Points",
            "_Sphere _P",
            "First diameter endpoint",
        ),
        ("SolidSphere3Points", "_Sphere _3Point", "First point"),
        ("SolidSphere4Points", "_Sphere _4Point", "First point"),
        (
            "SolidSphereAroundCurve",
            "_Sphere _AroundCurve",
            "Select curve or B-Rep edge",
        ),
        (
            "SolidSphereTangenttoCurves",
            "_Sphere _Tangent",
            "Select tangent curve",
        ),
        (
            "SolidSphereFitSpheretoPoints",
            "_Sphere _FitPoints",
            "Add points",
        ),
    ] {
        for menu in [false, true] {
            let mut c = controller();
            let before = c.document().clone();
            if menu {
                c.dispatch(id, InvocationOrigin::Menu).unwrap();
            } else {
                c.submit(mac).unwrap();
            }
            assert!(c.prompt().starts_with(prompt), "{id}: {}", c.prompt());
            assert_eq!(c.coordinator().undo_len(), 0);
            c.cancel();
            assert_eq!(c.document(), &before);
        }
    }
}

#[test]
fn solid_sphere_numeric_radius_is_not_a_mode_shortcut_after_center() {
    for radius in [2_f64, 3., 4.] {
        let mut c = controller();
        c.submit("Sphere").unwrap();
        c.accept_world_point([0., 0., 0.]).unwrap();
        c.submit(&radius.to_string()).unwrap();
        let id = *c.document().entities.keys().next().unwrap();
        assert!(
            (measures(&c, id).volume_mm3 - 4. * std::f64::consts::PI * radius.powi(3) / 3.).abs()
                < 1e-6
        );
    }
}

#[test]
fn solid_sphere_center_and_two_point_numeric_units_match_picked_geometry() {
    use om9_core::geometry::Unit;
    for unit in [
        Unit::Millimeter,
        Unit::Centimeter,
        Unit::Meter,
        Unit::Inch,
        Unit::Foot,
        Unit::Micrometer,
    ] {
        let radius = 5_f64;
        let input = radius / unit.factor();
        let expected = 4. * std::f64::consts::PI * radius.powi(3) / 3.;
        for mode in ["direct", "radius", "diameter", "two_point", "picked"] {
            let mut doc = Document::new("sphere units");
            doc.page_unit = unit;
            let mut c = Controller::new(doc, Limits::default()).unwrap();
            if mode == "direct" {
                c.submit(&format!("Sphere {input:.12}")).unwrap();
            } else if mode == "two_point" {
                c.submit("_Sphere _2Point").unwrap();
                c.submit("0").unwrap();
                c.submit(&format!("Diameter={:.12}", input * 2.)).unwrap();
            } else {
                c.submit("Sphere").unwrap();
                c.submit("0").unwrap();
                match mode {
                    "radius" => c.submit(&format!("{input:.12}")).unwrap(),
                    "diameter" => c.submit(&format!("Diameter={:.12}", input * 2.)).unwrap(),
                    "picked" => c.accept_world_point([radius, 0., 0.]).unwrap(),
                    _ => unreachable!(),
                }
            }
            let id = *c.document().entities.keys().next().unwrap();
            let actual = measures(&c, id).volume_mm3;
            assert!(
                (actual - expected).abs() / expected < 1e-8,
                "{unit:?} {mode}: {actual} vs {expected}"
            );
        }
    }
}

#[test]
fn solid_sphere_typed_center_uses_document_units_and_cplane() {
    for input in ["2,3,4", "2 3 4"] {
        let mut doc = Document::new("sphere rotated cm");
        doc.page_unit = om9_core::geometry::Unit::Centimeter;
        let mut c = Controller::new(doc, Limits::default()).unwrap();
        c.set_cplane_from_three_points([10., 20., 30.], [10., 21., 30.], [10., 20., 31.])
            .unwrap();
        c.submit("Sphere").unwrap();
        c.submit(input).unwrap();
        c.submit("Radius=0.5").unwrap();
        let e = c.document().entities.values().next().unwrap();
        assert_eq!(e.placement.point([0., 0., 0.]).unwrap(), [50., 40., 60.]);
        assert!(
            (measures(&c, e.id).volume_mm3 - 4. * std::f64::consts::PI * 125. / 3.).abs() < 1e-6
        );
    }
}

#[test]
fn solid_sphere_two_point_diameter_uses_captured_cplane_x_and_units() {
    let mut doc = Document::new("sphere diameter rotated");
    doc.page_unit = om9_core::geometry::Unit::Centimeter;
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.set_cplane_from_three_points([10., 20., 30.], [10., 21., 30.], [10., 20., 31.])
        .unwrap();
    c.submit("_Sphere _2Point").unwrap();
    c.submit("0").unwrap();
    c.submit("Diameter=2").unwrap();
    let e = c.document().entities.values().next().unwrap();
    assert_eq!(e.placement.point([0., 0., 0.]).unwrap(), [10., 30., 30.]);
    assert!((measures(&c, e.id).volume_mm3 - 4. * std::f64::consts::PI * 1000. / 3.).abs() < 1e-6);
}

#[test]
fn solid_sphere_two_point_mode_uses_a_diameter() {
    let mut c = controller();
    c.submit("Sphere").unwrap();
    c.submit("2Point").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([6.0, 0.0, 0.0]).unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    let measured = measures(&c, id);
    assert!((measured.volume_mm3 - 4.0 * std::f64::consts::PI / 3.0 * 27.0).abs() < 1e-6);
}

#[test]
fn solid_sphere_three_point_mode_uses_the_circumcircle() {
    let mut c = controller();
    c.submit("Sphere").unwrap();
    c.submit("3Point").unwrap();
    assert_eq!(c.prompt(), "First point");
    c.accept_world_point([1.0, 0.0, 0.0]).unwrap();
    assert_eq!(c.prompt(), "Second point");
    c.accept_world_point([0.0, 1.0, 0.0]).unwrap();
    assert_eq!(c.prompt(), "Third point");
    c.accept_world_point([0.0, 0.0, 1.0]).unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    let measured = measures(&c, id);
    let radius = (2.0_f64 / 3.0).sqrt();
    assert!((measured.volume_mm3 - 4.0 * std::f64::consts::PI / 3.0 * radius.powi(3)).abs() < 1e-6);
}

#[test]
fn solid_sphere_four_point_mode_uses_the_fourth_point() {
    let mut c = controller();
    c.submit("Sphere").unwrap();
    c.submit("4Point").unwrap();
    for point in [
        [1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
    ] {
        c.accept_world_point(point).unwrap();
    }
    let id = *c.document().entities.keys().next().unwrap();
    let measured = measures(&c, id);
    assert!((measured.volume_mm3 - 4.0 * std::f64::consts::PI / 3.0).abs() < 1e-6);
}

#[test]
fn solid_sphere_four_point_preview_and_invalid_point_retry() {
    let mut c = controller();
    c.submit("Sphere").unwrap();
    c.submit("4Point").unwrap();
    c.accept_world_point([11.0, 20.0, 30.0]).unwrap();
    c.accept_world_point([9.0, 20.0, 30.0]).unwrap();
    assert!(c.accept_world_point([10.0, 20.0, 30.0]).is_err());
    assert!(c.prompt().starts_with("Third point"));
    assert!(c.document().entities.is_empty());
    c.accept_world_point([10.0, 21.0, 30.0]).unwrap();
    assert!(c.solid_preview([10.0, 22.0, 30.0]).is_empty());
    assert!(c.accept_world_point([10.0, 22.0, 30.0]).is_err());
    assert!(c.prompt().starts_with("Fourth point"));
    assert!(c.document().entities.is_empty());
    let fourth = [10.0, 20.0, 32.0];
    let wire = c.solid_preview(fourth);
    assert_eq!(wire.len(), 192);
    for [a, _] in wire {
        let radius =
            ((a[0] - 10.0).powi(2) + (a[1] - 20.0).powi(2) + (a[2] - 30.75).powi(2)).sqrt();
        assert!((radius - 1.25).abs() < 1e-9);
    }
    c.accept_world_point(fourth).unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    assert!(
        (measures(&c, id).volume_mm3 - 4.0 * std::f64::consts::PI / 3.0 * 1.25_f64.powi(3)).abs()
            < 1e-6
    );
    c.submit("Undo").unwrap();
    assert!(c.document().entities.is_empty());
}

#[test]
fn solid_sphere_three_point_retries_a_collinear_point() {
    let mut c = controller();
    c.submit("Sphere").unwrap();
    c.submit("3Point").unwrap();
    c.accept_world_point([1.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([-1.0, 0.0, 0.0]).unwrap();
    assert!(c.accept_world_point([0.0, 0.0, 0.0]).is_err());
    assert!(c.prompt().starts_with("Third point"));
    assert!(c.document().entities.is_empty());
    c.accept_world_point([0.0, 1.0, 0.0]).unwrap();
    assert_eq!(c.document().entities.len(), 1);
}

#[test]
fn solid_sphere_fit_points_fits_the_collected_points() {
    let mut c = controller();
    c.submit("Sphere").unwrap();
    c.submit("FitPoints").unwrap();
    for point in [
        [1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
    ] {
        c.accept_world_point(point).unwrap();
    }
    c.submit("").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    let measured = measures(&c, id);
    assert!((measured.volume_mm3 - 4.0 * std::f64::consts::PI / 3.0).abs() < 1e-6);
}

#[test]
fn solid_sphere_vertical_mode_uses_center_radius() {
    let mut c = controller();
    c.submit("Sphere").unwrap();
    c.submit("Vertical").unwrap();
    assert!(c.prompt().starts_with("Center of sphere"));
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([2.0, 0.0, 0.0]).unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    assert!((measures(&c, id).volume_mm3 - 4.0 * std::f64::consts::PI / 3.0 * 8.0).abs() < 1e-6);
}

#[test]
fn solid_sphere_around_curve_uses_the_selected_curve() {
    let mut doc = Document::new("sphere around curve");
    let curve = om9_core::curve_build::polyline(&[[0.0, 0.0, 0.0], [20.0, 0.0, 0.0]]).unwrap();
    let entity = Entity::new("path", Geometry::Curve(curve), None);
    let curve_id = entity.id;
    doc.entities.insert(curve_id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[curve_id]).unwrap();
    c.submit("Sphere").unwrap();
    c.submit("AroundCurve").unwrap();
    c.accept_world_point([20.0, 0.0, 0.0]).unwrap();
    assert!(c.prompt().starts_with("Radius"));
    c.accept_world_point([20.0, 5.0, 0.0]).unwrap();
    let id = *c
        .document()
        .entities
        .keys()
        .find(|id| **id != curve_id)
        .unwrap();
    assert!((measures(&c, id).volume_mm3 - 4.0 * std::f64::consts::PI / 3.0 * 125.0).abs() < 1e-6);
    assert_eq!(
        c.document().entities[&id].placement,
        om9_core::geometry::Transform::translation([20.0, 0.0, 0.0])
    );
}

#[test]
fn solid_sphere_tangent_uses_the_curve_normal_and_radius() {
    let mut doc = Document::new("sphere tangent");
    let curve = om9_core::curve_build::polyline(&[[0.0, 0.0, 0.0], [20.0, 0.0, 0.0]]).unwrap();
    let entity = Entity::new("path", Geometry::Curve(curve), None);
    let curve_id = entity.id;
    doc.entities.insert(curve_id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[curve_id]).unwrap();
    c.submit("Sphere").unwrap();
    c.submit("Tangent").unwrap();
    c.accept_world_point([10.0, 0.0, 0.0]).unwrap();
    c.submit("5").unwrap();
    let id = *c
        .document()
        .entities
        .keys()
        .find(|id| **id != curve_id)
        .unwrap();
    assert!((measures(&c, id).volume_mm3 - 4.0 * std::f64::consts::PI / 3.0 * 125.0).abs() < 1e-6);
}

#[test]
fn solid_sphere_tangent_radius_modes_use_centimeters() {
    use om9_core::geometry::Unit;
    for mode in ["one", "two", "three", "curve_point", "point"] {
        let mut doc = Document::new("sphere tangent cm");
        doc.page_unit = Unit::Centimeter;
        let mut ids = Vec::new();
        let segments: Vec<[[f64; 3]; 2]> = match mode {
            "one" | "curve_point" => vec![[[-20., 0., 0.], [20., 0., 0.]]],
            "two" => vec![
                [[-20., 0., 0.], [20., 0., 0.]],
                [[10., -20., 0.], [10., 20., 0.]],
            ],
            "three" => vec![
                [[-10., -20., 0.], [-10., 20., 0.]],
                [[-20., 10., 0.], [20., 10., 0.]],
                [[10., -20., 0.], [10., 20., 0.]],
            ],
            _ => vec![],
        };
        for segment in segments {
            let curve = om9_core::curve_build::polyline(&segment).unwrap();
            let e = Entity::new("rail", Geometry::Curve(curve), None);
            ids.push(e.id);
            doc.entities.insert(e.id, e);
        }
        let mut c = Controller::new(doc, Limits::default()).unwrap();
        if !ids.is_empty() {
            c.select_objects(&ids).unwrap();
        }
        let before = c.document().clone();
        c.submit("Sphere").unwrap();
        c.submit("Tangent").unwrap();
        match mode {
            "one" => c.accept_world_point([0., 0., 0.]).unwrap(),
            "curve_point" => {
                c.submit("Point").unwrap();
                c.accept_world_point([0., 5., 0.]).unwrap();
            }
            "point" => {
                c.submit("Point").unwrap();
                c.accept_world_point([0., 0., 0.]).unwrap();
                c.accept_world_point([4., 0., 0.]).unwrap();
            }
            _ => {}
        }
        assert!(c.submit("radius=0").is_err());
        assert_eq!(c.document(), &before);
        let r = if mode == "one" {
            5.
        } else if mode == "three" {
            10.
        } else {
            3.
        };
        c.submit(&format!("radius={}", r / 10.)).unwrap();
        let e = c
            .document()
            .entities
            .values()
            .find(|e| !ids.contains(&e.id))
            .unwrap();
        let measured = measures(&c, e.id);
        assert!(
            (measured.volume_mm3 - 4. * std::f64::consts::PI * r * r * r / 3.).abs() < 1e-6,
            "{mode}: {measured:?}"
        );
        let p = e.placement.point([0., 0., 0.]).unwrap();
        match mode {
            "one" | "curve_point" => assert!((p[1].abs() - r).abs() < 1e-6, "{mode}: {p:?}"),
            "two" => assert!(
                (p[1].abs() - r).abs() < 1e-6 && ((p[0] - 10.).abs() - r).abs() < 1e-6,
                "{p:?}"
            ),
            "three" => assert!(
                ((p[0] + 10.).abs() - r).abs() < 1e-6
                    && ((p[1] - 10.).abs() - r).abs() < 1e-6
                    && ((p[0] - 10.).abs() - r).abs() < 1e-6,
                "{p:?}"
            ),
            "point" => assert!(
                ((p[0].hypot(p[1])).hypot(p[2]) - r).abs() < 1e-6
                    && (((p[0] - 4.).hypot(p[1])).hypot(p[2]) - r).abs() < 1e-6,
                "{p:?}"
            ),
            _ => unreachable!(),
        }
        c.undo().unwrap();
        assert_eq!(c.document(), &before);
    }
}

#[test]
fn solid_sphere_curve_modes_use_placed_sources_and_guard_stale() {
    use om9_core::geometry::Transform;
    for mode in ["around","path","tangent","two"] {
        let mut doc=Document::new("placed sphere curves");
        let mut a=Entity::new("rail",Geometry::Curve(om9_core::curve_build::polyline(&[[-20.,0.,0.],[20.,0.,0.]]).unwrap()),None);
        a.placement=Transform::translation([10.,20.,0.]);let ia=a.id;doc.entities.insert(ia,a);
        let mut ids=vec![ia];
        if mode=="two" {let mut b=Entity::new("cross",Geometry::Curve(om9_core::curve_build::polyline(&[[10.,-20.,0.],[10.,20.,0.]]).unwrap()),None);b.placement=Transform::translation([10.,20.,0.]);ids.push(b.id);doc.entities.insert(b.id,b);}
        let mut c=Controller::new(doc,Limits::default()).unwrap();let before=c.document().clone();
        if mode!="path" {c.select_objects(&ids).unwrap();}
        c.submit("Sphere").unwrap();c.submit(if matches!(mode,"around"|"path") {"AroundCurve"} else {"Tangent"}).unwrap();
        if mode=="path" {c.submit("Path=rail.Edge1").unwrap();}
        if matches!(mode,"around"|"path") {c.submit("OnCurve=0.5").unwrap();}
        else if mode=="tangent" {c.accept_world_point([10.,20.,0.]).unwrap();}
        c.submit("3").unwrap();let e=c.document().entities.values().find(|e|!ids.contains(&e.id)).unwrap();let p=e.placement.point([0.,0.,0.]).unwrap();
        if matches!(mode,"around"|"path") {assert!((p[0]-10.).abs()<1e-7 && (p[1]-20.).abs()<1e-7,"{mode} {p:?}");}
        else if mode=="tangent" {assert!((p[0]-10.).abs()<1e-6 && ((p[1]-20.).abs()-3.).abs()<1e-6,"{p:?}");}
        else {assert!(((p[1]-20.).abs()-3.).abs()<1e-6 && ((p[0]-20.).abs()-3.).abs()<1e-6,"{p:?}");}
        c.undo().unwrap();assert_eq!(c.document(),&before);
    }
    let mut rotated = Entity::new("rotated", Geometry::Curve(om9_core::curve_build::polyline(&[[-20., 0., 0.], [20., 0., 0.]]).unwrap()), None);
    rotated.placement = Transform([[0., -1., 0., 10.], [1., 0., 0., 20.], [0., 0., 1., 0.], [0., 0., 0., 1.]]);
    let rotated_id = rotated.id;
    let mut rotated_doc = Document::new("rotated sphere curve");
    rotated_doc.entities.insert(rotated_id, rotated);
    let mut rotated_controller = Controller::new(rotated_doc, Limits::default()).unwrap();
    rotated_controller.select_objects(&[rotated_id]).unwrap();
    rotated_controller.submit("Sphere").unwrap();
    rotated_controller.submit("Tangent").unwrap();
    rotated_controller.accept_world_point([10., 20., 0.]).unwrap();
    rotated_controller.submit("3").unwrap();
    let sphere = rotated_controller.document().entities.values().find(|e| e.id != rotated_id).unwrap();
    let center = sphere.placement.point([0., 0., 0.]).unwrap();
    assert!(((center[0] - 10.).abs() - 3.).abs() < 1e-6 && (center[1] - 20.).abs() < 1e-6, "{center:?}");
    let rail=Entity::new("rail",Geometry::Curve(om9_core::curve_build::polyline(&[[-10.,0.,0.],[10.,0.,0.]]).unwrap()),None);let id=rail.id;
    let mut doc=Document::new("stale sphere curve");doc.entities.insert(id,rail);let mut c=Controller::new(doc,Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();c.submit("Sphere").unwrap();c.submit("Tangent").unwrap();c.accept_world_point([0.,0.,0.]).unwrap();c.delete_objects(&[id]).unwrap();
    let after=c.document().clone();let undo=c.coordinator().undo_len();assert!(c.submit("3").is_err());assert_eq!(c.document(),&after);assert_eq!(c.coordinator().undo_len(),undo);c.cancel();
}

#[test]
fn solid_sphere_accepts_diameter_option() {
    let mut c = controller();
    c.submit("Sphere").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.submit("Diameter=10").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    assert!((measures(&c, id).volume_mm3 - 4.0 * std::f64::consts::PI / 3.0 * 125.0).abs() < 1e-6);
}

#[test]
fn solid_box_accepts_width_and_height_options() {
    let mut c = controller();
    c.submit("Box").unwrap();
    c.submit("3Point").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([10.0, 0.0, 0.0]).unwrap();
    c.submit("Width=5").unwrap();
    c.submit("Height=4").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    assert!((measures(&c, id).volume_mm3 - 200.0).abs() < 1e-6);
}

#[test]
fn solid_box_exposes_a_twelve_edge_wire_preview() {
    let mut c = controller();
    c.submit("Box").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    let wire = c.solid_preview([10.0, 20.0, 0.0]);
    assert_eq!(wire.len(), 12);
    assert!(wire.iter().all(|[a, b]| a != b));
}

#[test]
fn solid_box_height_can_be_picked_in_the_viewport() {
    let mut c = controller();
    c.submit("Box").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([10.0, 20.0, 0.0]).unwrap();
    c.accept_world_point([0.0, 0.0, 30.0]).unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    assert!((measures(&c, id).volume_mm3 - 6000.0).abs() < 1e-6);
}

#[test]
fn solid_box_cube_is_available_after_the_diagonal_first_point() {
    let mut c = controller();
    c.submit("Box").unwrap();
    c.submit("Diagonal").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.submit("Cube").unwrap();
    c.accept_world_point([10.0, 10.0, 10.0]).unwrap();
    assert_eq!(c.coordinator().undo_len(), 1);
    let id = *c.document().entities.keys().next().unwrap();
    assert!((measures(&c, id).volume_mm3 - 1000.0).abs() < 1e-6);
}

#[test]
fn solid_box_cube_typed_second_corner_commits_without_enter() {
    let mut c = controller();
    let before = c.document().clone();
    c.submit("Box").unwrap();
    c.submit("Cube").unwrap();
    c.submit("0").unwrap();
    assert!(c.submit("0,0,0").is_err());
    assert_eq!(c.document(), &before);
    c.submit("4,4,4").unwrap();
    assert_eq!(c.coordinator().undo_len(), 1);
    let e = c.document().entities.values().next().unwrap();
    assert!((measures(&c, e.id).volume_mm3 - 64.).abs() < 1e-6);
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
}

#[test]
fn solid_box_accepts_mode_prefix_and_rejects_corners() {
    let mut c = controller();
    c.submit("Box").unwrap();
    assert!(c.submit("Corners").is_err());
    c.submit("Mode=Diagonal").unwrap();
    assert!(c.prompt().starts_with("First corner"));
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([10.0, 20.0, 0.0]).unwrap();
    c.submit("30").unwrap();
    assert_eq!(c.document().entities.len(), 1);
}

#[test]
fn solid_sphere_tangent_point_constraints_use_two_points() {
    let mut c = controller();
    c.submit("Sphere").unwrap();
    c.submit("Tangent").unwrap();
    c.submit("Point").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([4.0, 0.0, 0.0]).unwrap();
    c.submit("3").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    let measured = measures(&c, id);
    assert!((measured.volume_mm3 - 4.0 * std::f64::consts::PI / 3.0 * 27.0).abs() < 1e-6);
    assert_eq!(
        c.document().entities[&id].placement,
        om9_core::geometry::Transform::translation([2.0, 5.0_f64.sqrt(), 0.0])
    );
}

#[test]
fn solid_sphere_two_point_accepts_diameter_input() {
    let mut c = controller();
    c.submit("Sphere").unwrap();
    c.submit("2Point").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.submit("Diameter=10").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    assert!((measures(&c, id).volume_mm3 - 4.0 * std::f64::consts::PI / 3.0 * 125.0).abs() < 1e-6);
    assert_eq!(
        c.document().entities[&id].placement,
        om9_core::geometry::Transform::translation([5.0, 0.0, 0.0])
    );
}

#[test]
fn solid_sphere_three_point_accepts_radius_for_the_chord() {
    let mut c = controller();
    c.submit("Sphere").unwrap();
    c.submit("3Point").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([4.0, 0.0, 0.0]).unwrap();
    c.submit("Radius=3").unwrap();
    assert_eq!(c.coordinator().undo_len(), 0);
    c.accept_world_point([2., 5., 0.]).unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    assert!((measures(&c, id).volume_mm3 - 4.0 * std::f64::consts::PI / 3.0 * 27.0).abs() < 1e-6);
    assert_eq!(
        c.document().entities[&id].placement,
        om9_core::geometry::Transform::translation([2.0, 5.0_f64.sqrt(), 0.0])
    );
}

#[test]
fn solid_sphere_three_point_radius_waits_for_direction_and_respects_units() {
    for sign in [-1., 1.] {
        let mut doc = Document::new("sphere chord cm");
        doc.page_unit = om9_core::geometry::Unit::Centimeter;
        let mut c = Controller::new(doc, Limits::default()).unwrap();
        let before = c.document().clone();
        c.set_cplane_from_three_points([0., 0., 0.], [0., 1., 0.], [0., 0., 1.])
            .unwrap();
        c.submit("Sphere 3Point").unwrap();
        c.accept_world_point([10., 20., 30.]).unwrap();
        c.accept_world_point([10., 20., 34.]).unwrap();
        c.submit("Radius").unwrap();
        assert_eq!(c.prompt(), "Radius");
        assert!(c.submit("0.1").is_err());
        c.submit("0.3").unwrap();
        assert_eq!(c.prompt(), "Choose direction away from chord");
        assert_eq!(c.document(), &before);
        assert!(c.accept_world_point([10., 20., 35.]).is_err());
        assert!(c.solid_preview([10., 20., 35.]).is_empty());
        c.submit("Undo").unwrap();
        assert_eq!(c.prompt(), "Third point");
        c.submit("Radius=0.3").unwrap();
        let pick = [10., 20. + sign * 5., 32.];
        let wire = c.solid_preview(pick);
        assert!(!wire.is_empty());
        c.accept_world_point(pick).unwrap();
        let e = c.document().entities.values().next().unwrap();
        let center = e.placement.point([0., 0., 0.]).unwrap();
        assert!(
            (center[0] - 10.).abs() < 1e-8
                && (center[1] - (20. + sign * 5_f64.sqrt())).abs() < 1e-8
                && (center[2] - 32.).abs() < 1e-8,
            "{center:?}"
        );
        assert!((measures(&c, e.id).volume_mm3 - 36. * std::f64::consts::PI).abs() < 1e-6);
        c.undo().unwrap();
        assert_eq!(c.document(), &before);
    }
}

#[test]
fn solid_sphere_tangent_handles_an_out_of_plane_curve() {
    let mut doc = Document::new("sphere tangent z");
    let curve = om9_core::curve_build::polyline(&[[0.0, 0.0, 0.0], [0.0, 0.0, 20.0]]).unwrap();
    let entity = Entity::new("path", Geometry::Curve(curve), None);
    let curve_id = entity.id;
    doc.entities.insert(curve_id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[curve_id]).unwrap();
    c.submit("Sphere").unwrap();
    c.submit("Tangent").unwrap();
    c.accept_world_point([0.0, 0.0, 10.0]).unwrap();
    c.submit("5").unwrap();
    let id = *c
        .document()
        .entities
        .keys()
        .find(|id| **id != curve_id)
        .unwrap();
    let placement = c.document().entities[&id].placement;
    assert!((placement.0[0][3] - 0.0).abs() < 1e-9);
    assert!((placement.0[1][3].abs() - 5.0).abs() < 1e-9);
    assert!((placement.0[2][3] - 10.0).abs() < 1e-9, "{placement:?}");
}

#[test]
fn solid_box_enter_height_uses_the_base_width() {
    let mut c = controller();
    c.submit("Box").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([10.0, 20.0, 0.0]).unwrap();
    c.submit("").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    assert!((measures(&c, id).volume_mm3 - 4000.0).abs() < 1e-6);
}

#[test]
fn solid_box_center_accepts_length_width_height_sequence() {
    let mut c = controller();
    c.submit("Box").unwrap();
    c.submit("Center").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.submit("Length=10").unwrap();
    assert!(c.prompt().starts_with("Width"));
    c.submit("Width=20").unwrap();
    assert!(c.prompt().starts_with("Height"));
    c.submit("Height=5").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    assert!((measures(&c, id).volume_mm3 - 1000.0).abs() < 1e-6);
    assert_eq!(
        c.document().entities[&id].placement,
        om9_core::geometry::Transform::translation([-5.0, -10.0, 0.0])
    );
}

#[test]
fn solid_box_center_undo_steps_back_through_length_and_width() {
    let mut c = controller();
    c.submit("Box").unwrap();
    c.submit("Center").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.submit("Length=10").unwrap();
    c.submit("Undo").unwrap();
    assert!(c.prompt().starts_with("Other corner"));
    c.submit("Length=10").unwrap();
    c.submit("Width=20").unwrap();
    c.submit("Undo").unwrap();
    assert!(c.prompt().starts_with("Width"));
}

#[test]
fn solid_sphere_four_point_undo_returns_to_third_point() {
    let mut c = controller();
    c.submit("Sphere").unwrap();
    c.submit("4Point").unwrap();
    c.accept_world_point([1.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([-1.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([0.0, 1.0, 0.0]).unwrap();
    assert!(c.prompt().starts_with("Fourth point"));
    c.submit("Undo").unwrap();
    assert!(c.prompt().starts_with("Third point"));
}

#[test]
fn solid_sphere_around_curve_accepts_oncurve_fraction() {
    let mut doc = Document::new("sphere oncurve");
    let curve = om9_core::curve_build::polyline(&[[0.0, 0.0, 0.0], [20.0, 0.0, 0.0]]).unwrap();
    let entity = Entity::new("path", Geometry::Curve(curve), None);
    let curve_id = entity.id;
    doc.entities.insert(curve_id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[curve_id]).unwrap();
    c.submit("Sphere").unwrap();
    c.submit("AroundCurve").unwrap();
    c.submit("OnCurve=0.5").unwrap();
    c.submit("5").unwrap();
    let id = *c
        .document()
        .entities
        .keys()
        .find(|id| **id != curve_id)
        .unwrap();
    assert_eq!(
        c.document().entities[&id].placement,
        om9_core::geometry::Transform::translation([10.0, 0.0, 0.0])
    );
}

#[test]
fn solid_sphere_tangent_point_solution_two_uses_the_other_branch() {
    let mut c = controller();
    c.submit("Sphere").unwrap();
    c.submit("Tangent").unwrap();
    c.submit("Point").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([4.0, 0.0, 0.0]).unwrap();
    c.submit("Solution=2").unwrap();
    c.submit("3").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    assert_eq!(
        c.document().entities[&id].placement,
        om9_core::geometry::Transform::translation([2.0, -5.0_f64.sqrt(), 0.0])
    );
}

#[test]
fn solid_sphere_tangent_curve_solution_two_flips_the_normal_side() {
    let mut doc = Document::new("sphere tangent curve solution");
    let curve = om9_core::curve_build::polyline(&[[0.0, 0.0, 0.0], [20.0, 0.0, 0.0]]).unwrap();
    let entity = Entity::new("path", Geometry::Curve(curve), None);
    let curve_id = entity.id;
    doc.entities.insert(curve_id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[curve_id]).unwrap();
    c.submit("Sphere").unwrap();
    c.submit("Tangent").unwrap();
    c.accept_world_point([10.0, 0.0, 0.0]).unwrap();
    c.submit("Solution=2").unwrap();
    c.submit("5").unwrap();
    let id = *c
        .document()
        .entities
        .keys()
        .find(|id| **id != curve_id)
        .unwrap();
    assert_eq!(
        c.document().entities[&id].placement,
        om9_core::geometry::Transform::translation([10.0, 5.0, 0.0])
    );
}

#[test]
fn solid_sphere_around_curve_accepts_path_by_label() {
    let mut doc = Document::new("sphere path label");
    let curve = om9_core::curve_build::polyline(&[[0.0, 0.0, 0.0], [20.0, 0.0, 0.0]]).unwrap();
    let entity = Entity::new("path", Geometry::Curve(curve), None);
    let curve_id = entity.id;
    doc.entities.insert(curve_id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.submit("Sphere").unwrap();
    c.submit("AroundCurve").unwrap();
    c.submit("Path=path.Edge1").unwrap();
    c.submit("OnCurve=0.5").unwrap();
    c.submit("5").unwrap();
    let id = *c
        .document()
        .entities
        .keys()
        .find(|id| **id != curve_id)
        .unwrap();
    assert_eq!(
        c.document().entities[&id].placement,
        om9_core::geometry::Transform::translation([10.0, 0.0, 0.0])
    );
}

#[test]
fn solid_box_exposes_clickable_mode_tokens() {
    let mut c = controller();
    c.submit("Box").unwrap();
    let tokens = c.solid_option_tokens();
    assert!(
        tokens
            .iter()
            .any(|(_, command)| *command == "Mode=Diagonal")
    );
    assert!(tokens.iter().any(|(_, command)| *command == "Mode=Cube"));
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    let tokens = c.solid_option_tokens();
    assert!(tokens.iter().any(|(_, command)| *command == "Undo"));
    assert!(tokens.iter().any(|(_, command)| *command == "Cancel"));
}

#[test]
fn solid_sphere_exposes_clickable_mode_tokens() {
    let mut c = controller();
    c.submit("Sphere").unwrap();
    let tokens = c.solid_option_tokens();
    for command in [
        "Mode=2Point",
        "Mode=3Point",
        "Mode=4Point",
        "Mode=Tangent",
        "Cancel",
    ] {
        assert!(
            tokens.iter().any(|(_, value)| *value == command),
            "{command}"
        );
    }
}

#[test]
fn solid_sphere_tangent_curve_exposes_solution_tokens() {
    let mut doc = Document::new("tangent tokens");
    let curve = om9_core::curve_build::polyline(&[[0.0, 0.0, 0.0], [20.0, 0.0, 0.0]]).unwrap();
    let entity = Entity::new("path", Geometry::Curve(curve), None);
    let curve_id = entity.id;
    doc.entities.insert(curve_id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[curve_id]).unwrap();
    c.submit("Sphere").unwrap();
    c.submit("Tangent").unwrap();
    c.accept_world_point([10.0, 0.0, 0.0]).unwrap();
    let tokens = c.solid_option_tokens();
    assert!(tokens.iter().any(|(_, value)| *value == "Solution=2"));
}

#[test]
fn solid_sphere_tangent_offers_the_point_constraint_token() {
    let mut c = controller();
    c.submit("Sphere").unwrap();
    c.submit("Tangent").unwrap();
    let tokens = c.solid_option_tokens();
    assert!(tokens.iter().any(|(_, value)| *value == "Point"));
}

#[test]
fn solid_box_center_length_width_then_height_click() {
    let mut c = controller();
    c.submit("Box").unwrap();
    c.submit("Center").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.submit("Length=10").unwrap();
    c.submit("Width=20").unwrap();
    c.accept_world_point([0.0, 0.0, 5.0]).unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    assert!((measures(&c, id).volume_mm3 - 1000.0).abs() < 1e-6);
}

#[test]
fn solid_right_click_defaults_start_box_3point_and_sphere_2point() {
    use om9_app::session::InvocationOrigin;
    let mut c = controller();
    c.dispatch("Box3Point", InvocationOrigin::Menu).unwrap();
    assert!(c.prompt().starts_with("Start of edge"));
    c.cancel();
    c.dispatch("Sphere2Point", InvocationOrigin::Menu).unwrap();
    assert_eq!(c.prompt(), "First diameter endpoint");
}

#[test]
fn solid_box_accepts_undo_alias() {
    let mut c = controller();
    c.submit("Box").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.submit("U").unwrap();
    assert!(c.prompt().starts_with("First corner"));
}

#[test]
fn solid_options_accept_short_key_aliases() {
    let mut c = controller();
    c.submit("Sphere").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.submit("D=10").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    assert!((measures(&c, id).volume_mm3 - 4.0 * std::f64::consts::PI / 3.0 * 125.0).abs() < 1e-6);

    let mut c = controller();
    c.submit("Box").unwrap();
    c.submit("Center").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.submit("L=10").unwrap();
    c.submit("W=20").unwrap();
    c.submit("H=5").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    assert!((measures(&c, id).volume_mm3 - 1000.0).abs() < 1e-6);
}

#[test]
fn solid_sphere_tangent_curve_tokens_replace_the_mode_list() {
    let mut doc = Document::new("tangent token list");
    let curve = om9_core::curve_build::polyline(&[[0.0, 0.0, 0.0], [20.0, 0.0, 0.0]]).unwrap();
    let entity = Entity::new("path", Geometry::Curve(curve), None);
    let curve_id = entity.id;
    doc.entities.insert(curve_id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[curve_id]).unwrap();
    c.submit("Sphere").unwrap();
    c.submit("Tangent").unwrap();
    let tokens = c.solid_option_tokens();
    assert!(tokens.iter().any(|(_, value)| *value == "Cancel"));
    assert!(!tokens.iter().any(|(_, value)| *value == "Mode=Center"));
}

#[test]
fn solid_sphere_tangent_two_curves_finds_a_circle() {
    let mut doc = Document::new("tangent two curves");
    let horizontal =
        om9_core::curve_build::polyline(&[[-20.0, 0.0, 0.0], [20.0, 0.0, 0.0]]).unwrap();
    let vertical =
        om9_core::curve_build::polyline(&[[10.0, -20.0, 0.0], [10.0, 20.0, 0.0]]).unwrap();
    let a = Entity::new("h", Geometry::Curve(horizontal), None);
    let b = Entity::new("v", Geometry::Curve(vertical), None);
    let (ida, idb) = (a.id, b.id);
    doc.entities.insert(ida, a);
    doc.entities.insert(idb, b);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[ida, idb]).unwrap();
    c.submit("Sphere").unwrap();
    c.submit("Tangent").unwrap();
    c.submit("3").unwrap();
    let id = *c
        .document()
        .entities
        .keys()
        .find(|id| **id != ida && **id != idb)
        .unwrap();
    let placement = c.document().entities[&id].placement;
    assert!((placement.0[0][3] - 7.0).abs() < 1e-6, "{placement:?}");
    assert!(
        (placement.0[1][3].abs() - 3.0).abs() < 1e-6,
        "{placement:?}"
    );
}

#[test]
fn solid_box_exposes_preview_markers_for_picked_points() {
    let mut c = controller();
    c.submit("Box").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    assert_eq!(c.solid_preview_markers().len(), 1);
    c.accept_world_point([10.0, 20.0, 0.0]).unwrap();
    assert_eq!(c.solid_preview_markers().len(), 2);
}

#[test]
fn solid_sphere_tangent_three_curves_finds_the_incircle() {
    let mut doc = Document::new("tangent three curves");
    let c1 = om9_core::curve_build::polyline(&[[-10.0, -20.0, 0.0], [-10.0, 20.0, 0.0]]).unwrap();
    let c2 = om9_core::curve_build::polyline(&[[-20.0, 10.0, 0.0], [20.0, 10.0, 0.0]]).unwrap();
    let c3 = om9_core::curve_build::polyline(&[[10.0, -20.0, 0.0], [10.0, 20.0, 0.0]]).unwrap();
    let e1 = Entity::new("a", Geometry::Curve(c1), None);
    let e2 = Entity::new("b", Geometry::Curve(c2), None);
    let e3 = Entity::new("c", Geometry::Curve(c3), None);
    let (i1, i2, i3) = (e1.id, e2.id, e3.id);
    doc.entities.insert(i1, e1);
    doc.entities.insert(i2, e2);
    doc.entities.insert(i3, e3);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[i1, i2, i3]).unwrap();
    c.submit("Sphere").unwrap();
    c.submit("Tangent").unwrap();
    c.submit("10").unwrap();
    let id = *c
        .document()
        .entities
        .keys()
        .find(|id| **id != i1 && **id != i2 && **id != i3)
        .unwrap();
    let placement = c.document().entities[&id].placement;
    assert!(placement.0[0][3].abs() < 1e-6, "{placement:?}");
    assert!(placement.0[1][3].abs() < 1e-6, "{placement:?}");
}

#[test]
fn solid_sphere_tangent_curve_and_point_finds_a_circle() {
    let mut doc = Document::new("tangent curve point");
    let curve = om9_core::curve_build::polyline(&[[-20.0, 0.0, 0.0], [20.0, 0.0, 0.0]]).unwrap();
    let entity = Entity::new("path", Geometry::Curve(curve), None);
    let curve_id = entity.id;
    doc.entities.insert(curve_id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[curve_id]).unwrap();
    c.submit("Sphere").unwrap();
    c.submit("Tangent").unwrap();
    c.submit("Point").unwrap();
    c.accept_world_point([0.0, 5.0, 0.0]).unwrap();
    c.submit("3").unwrap();
    let id = *c
        .document()
        .entities
        .keys()
        .find(|id| **id != curve_id)
        .unwrap();
    let placement = c.document().entities[&id].placement;
    assert!(
        (placement.0[0][3].abs() - 5.0_f64.sqrt()).abs() < 1e-6,
        "{placement:?}"
    );
    assert!((placement.0[1][3] - 3.0).abs() < 1e-6, "{placement:?}");
}

#[test]
fn solid_sphere_tangent_two_curves_exposes_undo_tokens() {
    let mut doc = Document::new("tangent2 tokens");
    let a = om9_core::curve_build::polyline(&[[-10.0, 0.0, 0.0], [10.0, 0.0, 0.0]]).unwrap();
    let b = om9_core::curve_build::polyline(&[[0.0, -10.0, 0.0], [0.0, 10.0, 0.0]]).unwrap();
    let ea = Entity::new("a", Geometry::Curve(a), None);
    let eb = Entity::new("b", Geometry::Curve(b), None);
    let (ia, ib) = (ea.id, eb.id);
    doc.entities.insert(ia, ea);
    doc.entities.insert(ib, eb);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[ia, ib]).unwrap();
    c.submit("Sphere").unwrap();
    c.submit("Tangent").unwrap();
    let tokens = c.solid_option_tokens();
    assert!(tokens.iter().any(|(_, value)| *value == "Cancel"));
    assert!(!tokens.iter().any(|(_, value)| *value == "Mode=Center"));
}

#[test]
fn solid_sphere_tangent_curve_two_points_finds_a_circle() {
    let mut doc = Document::new("tangent curve two points");
    let curve = om9_core::curve_build::polyline(&[[-20.0, 0.0, 0.0], [20.0, 0.0, 0.0]]).unwrap();
    let entity = Entity::new("path", Geometry::Curve(curve), None);
    let curve_id = entity.id;
    doc.entities.insert(curve_id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[curve_id]).unwrap();
    c.submit("Sphere").unwrap();
    c.submit("Tangent").unwrap();
    c.submit("Point").unwrap();
    c.accept_world_point([0.0, 5.0, 0.0]).unwrap();
    c.accept_world_point([10.0, 5.0, 0.0]).unwrap();
    c.submit("").unwrap();
    let id = *c
        .document()
        .entities
        .keys()
        .find(|id| **id != curve_id)
        .unwrap();
    let placement = c.document().entities[&id].placement;
    assert!((placement.0[0][3] - 5.0).abs() < 1e-6, "{placement:?}");
    assert!((placement.0[1][3] - 5.0).abs() < 1e-6, "{placement:?}");
}

#[test]
fn solid_box_negative_height_shifts_the_base_down() {
    let mut c = controller();
    c.submit("Box").unwrap();
    c.accept_world_point([0.0, 0.0, 10.0]).unwrap();
    c.accept_world_point([10.0, 20.0, 10.0]).unwrap();
    c.submit("Height=-5").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    assert!((measures(&c, id).volume_mm3 - 1000.0).abs() < 1e-6);
    assert!((c.document().entities[&id].placement.0[2][3] - 5.0).abs() < 1e-6);
}

#[test]
fn solid_sphere_tangent_two_curves_and_point_finds_a_circle() {
    let mut doc = Document::new("tangent two curves point");
    let horizontal =
        om9_core::curve_build::polyline(&[[-20.0, 0.0, 0.0], [20.0, 0.0, 0.0]]).unwrap();
    let vertical = om9_core::curve_build::polyline(&[[0.0, -20.0, 0.0], [0.0, 20.0, 0.0]]).unwrap();
    let a = Entity::new("h", Geometry::Curve(horizontal), None);
    let b = Entity::new("v", Geometry::Curve(vertical), None);
    let (ida, idb) = (a.id, b.id);
    doc.entities.insert(ida, a);
    doc.entities.insert(idb, b);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[ida, idb]).unwrap();
    c.submit("Sphere").unwrap();
    c.submit("Tangent").unwrap();
    c.submit("Point").unwrap();
    c.accept_world_point([5.0, 5.0, 0.0]).unwrap();
    c.submit("").unwrap();
    let id = *c
        .document()
        .entities
        .keys()
        .find(|id| **id != ida && **id != idb)
        .unwrap();
    let placement = c.document().entities[&id].placement;
    let expected = 5.0 * (2.0 - 2.0_f64.sqrt());
    assert!((placement.0[0][3] - expected).abs() < 1e-6, "{placement:?}");
    assert!((placement.0[1][3] - expected).abs() < 1e-6, "{placement:?}");
}

#[test]
fn solid_sphere_tangent_three_points_use_the_circumcircle() {
    let mut c = controller();
    c.submit("Sphere").unwrap();
    c.submit("Tangent").unwrap();
    c.submit("Point").unwrap();
    c.accept_world_point([1.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([-1.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([0.0, 1.0, 0.0]).unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    let measured = measures(&c, id);
    assert!((measured.volume_mm3 - 4.0 * std::f64::consts::PI / 3.0).abs() < 1e-6);
    assert_eq!(
        c.document().entities[&id].placement,
        om9_core::geometry::Transform::translation([0.0, 0.0, 0.0])
    );
}

#[test]
fn solid_sphere_aroundcurve_trimmed_circular_edge_preserves_orientation_and_placement() {
    use om9_core::geometry::{Curve, Transform};
    use om9_kernel::{creation::{Primitive, Profile, DEFAULT_DEFLECTION_MM}, editing::{self, BooleanOp}};
    let limits = Limits::default();
    let cylinder = creation::primitive(Primitive::Cylinder { radius_mm: 5., height_mm: 10. }, DEFAULT_DEFLECTION_MM, &limits).unwrap();
    let profile = Profile::new(vec![[0.,-10.,0.],[10.,-10.,0.],[10.,10.,0.],[0.,10.,0.]]).unwrap();
    let cutter = creation::extrude(&profile, [0.,0.,10.], true, DEFAULT_DEFLECTION_MM, &limits).unwrap();
    let body = editing::boolean(&cylinder, &cutter, BooleanOp::Intersection, DEFAULT_DEFLECTION_MM, &limits).unwrap().body().unwrap().clone();
    let (index, curve) = (0..body.display.edges.len()).find_map(|i| {
        let c = editing::circular_edge(&body, i, &limits).ok()?;
        match &c { Curve::Circle { domain, .. } if domain[1] < 6.0 => Some((i,c)), _ => None }
    }).expect("trimmed circular edge");
    let (a,b) = editing::oriented_edge_endpoints(&body, index, &limits).unwrap();
    let start = om9_core::curve_ops::point_at(&curve, 0., &limits).unwrap();
    let end = om9_core::curve_ops::point_at(&curve, 1., &limits).unwrap();
    for (actual, expected) in [(start,a),(end,b)] {
        assert!((0..3).all(|i| (actual[i]-expected[i]).abs() < 1e-7), "{actual:?} {expected:?}");
    }
    let middle = om9_core::curve_ops::point_at(&curve, 0.5, &limits).unwrap();
    let sum = [a[0]+b[0], a[1]+b[1]];
    let length = sum[0].hypot(sum[1]);
    assert!(length > 1e-6);
    assert!((middle[0]-5.*sum[0]/length).abs() < 1e-7 && (middle[1]-5.*sum[1]/length).abs() < 1e-7, "{middle:?}");
    let placement = Transform([[0.,-1.,0.,20.],[1.,0.,0.,30.],[0.,0.,1.,7.],[0.,0.,0.,1.]]);
    for fraction in [0., 0.5, 1., 2.] {
        let mut source = Entity::new("trimmed", Geometry::Brep(body.clone()), None); source.placement = placement;
        let id = source.id; let mut doc = Document::new("trimmed arc sphere"); doc.entities.insert(id, source);
        let mut c = Controller::new(doc, limits.clone()).unwrap();
        c.submit("Sphere AroundCurve").unwrap();
        if fraction == 2. {
            let pick = placement.point([middle[0]*1.5, middle[1]*1.5, middle[2]]).unwrap();
            c.submit(&format!("Path=trimmed.Edge{}@{},{},{}", index+1, pick[0], pick[1], pick[2])).unwrap();
        } else {
            c.submit(&format!("Path=trimmed.Edge{}", index+1)).unwrap();
            c.submit(&format!("OnCurve={fraction}")).unwrap();
        }
        c.submit("3").unwrap();
        let sphere = c.document().entities.values().find(|e| e.id != id).unwrap();
        let actual = sphere.placement.point([0.,0.,0.]).unwrap();
        let local = match fraction { f if f == 0. => a, f if f == 1. => b, _ => middle };
        let expected = placement.point(local).unwrap();
        assert!((0..3).all(|i| (actual[i]-expected[i]).abs() < 1e-7), "{actual:?} {expected:?}");
        c.undo().unwrap(); assert_eq!(c.document().entities.len(),1);
    }
}

#[test]
fn solid_sphere_aroundcurve_accepts_named_circular_brep_edge() {
    for mode in ["fraction", "pick", "stale"] {
        let mut c = controller();
        c.submit("Cylinder 5 10").unwrap();
        let body_id = *c.document().entities.keys().next().unwrap();
        let body = &c.document().entities[&body_id];
        let Geometry::Brep(brep) = &body.geometry else { panic!("cylinder") };
        let (index, curve) = (0..brep.display.edges.len()).find_map(|i| om9_kernel::editing::circular_edge(brep, i, c.coordinator().limits()).ok().map(|curve| (i, curve))).unwrap();
        let expected = om9_core::curve_ops::point_at(&curve, 0.25, c.coordinator().limits()).unwrap();
        assert!((expected[0].hypot(expected[1]) - 5.).abs() < 1e-8);
        assert!(expected[2].abs() < 1e-8 || (expected[2] - 10.).abs() < 1e-8);
        let label = body.label.clone();
        c.clear_selection();
        c.submit("Sphere").unwrap();
        c.submit("AroundCurve").unwrap();
        c.submit(&format!("Path={label}.Edge{}", index + 1)).unwrap();
        if mode == "fraction" { c.submit("OnCurve=0.25").unwrap(); }
        else { c.accept_world_point(expected).unwrap(); }
        if mode == "stale" {
            c.delete_objects(&[body_id]).unwrap();
            let after = c.document().clone();
            assert!(c.submit("3").is_err());
            assert_eq!(c.document(), &after);
            c.cancel();
            continue;
        }
        assert_eq!(c.solid_preview([expected[0], expected[1], expected[2] + 3.]).len(), 192);
        c.submit("3").unwrap();
        let sphere = c.document().entities.values().find(|e| e.id != body_id).unwrap();
        let center = sphere.placement.point([0., 0., 0.]).unwrap();
        assert!((0..3).all(|i| (center[i] - expected[i]).abs() < 1e-7), "{mode}: {center:?} {expected:?}");
        c.undo().unwrap();
        assert_eq!(c.document().entities.len(), 1);
    }
}

#[test]
fn solid_sphere_aroundcurve_brep_edge_selection_stages_and_undo() {
    use om9_app::session::SelectedSubobject;
    use om9_scene::scene::Subelement;
    let mut c = controller();
    c.submit("Box 10 20 30").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    let Geometry::Brep(brep) = &c.document().entities[&id].geometry else { panic!("box") };
    let edge = brep.display.edges[0];
    let wrong_edge = brep.display.edges[1];
    let (a, b) = om9_kernel::editing::straight_edge_endpoints(brep, 0, c.coordinator().limits()).unwrap();
    let center = std::array::from_fn(|i| (a[i] + b[i]) * 0.5);
    c.clear_selection();
    c.submit("Sphere").unwrap();
    c.submit("AroundCurve").unwrap();
    let scene = c.scene().unwrap();
    let object = scene.objects.iter().find(|o| o.root == id && o.path.is_empty()).unwrap();
    let reference = SelectedSubobject { root: id, path: vec![], subset: Subelement::BrepEdge(edge), source: object.source, world_epoch: scene.epoch };
    let mut wrong = reference.clone();
    wrong.subset = Subelement::BrepEdge(wrong_edge);
    assert_eq!(c.solid_sphere_brep_edge_stage(), 1);
    c.solid_sphere_pick_brep_edge(reference.clone(), center).unwrap();
    assert_eq!(c.solid_sphere_brep_edge_stage(), 2);
    assert!(c.solid_sphere_pick_brep_edge(wrong, center).is_err());
    c.submit("Undo").unwrap();
    assert_eq!(c.solid_sphere_brep_edge_stage(), 1);
    c.solid_sphere_pick_brep_edge(reference.clone(), center).unwrap();
    c.solid_sphere_pick_brep_edge(reference, center).unwrap();
    assert_eq!(c.solid_sphere_brep_edge_stage(), 0);
    c.submit("3").unwrap();
    let sphere = c.document().entities.values().find(|e| e.id != id).unwrap();
    let actual = sphere.placement.point([0., 0., 0.]).unwrap();
    assert!((0..3).all(|i| (center[i] - actual[i]).abs() < 1e-7));
    c.undo().unwrap();
    assert_eq!(c.document().entities.len(), 1);
}

#[test]
fn solid_sphere_aroundcurve_accepts_named_straight_brep_edge() {
    for mode in ["fraction", "pick", "stale"] {
        let mut c = controller();
        c.submit("Box 10 20 30").unwrap();
        let body_id = *c.document().entities.keys().next().unwrap();
        let body = &c.document().entities[&body_id];
        let Geometry::Brep(brep) = &body.geometry else { panic!("box") };
        let (a, b) = om9_kernel::editing::straight_edge_endpoints(brep, 0, c.coordinator().limits()).unwrap();
        let expected = std::array::from_fn::<_, 3, _>(|i| a[i] * 0.75 + b[i] * 0.25);
        let label = body.label.clone();
        c.clear_selection();
        c.submit("Sphere").unwrap();
        c.submit("AroundCurve").unwrap();
        assert!(c.submit(&format!("Path={label}")).is_err());
        assert!(c.submit(&format!("Path={label}.Edge999")).is_err());
        c.submit(&format!("Path={label}.Edge1")).unwrap();
        if mode == "fraction" { c.submit("OnCurve=0.25").unwrap(); }
        else { c.accept_world_point(expected).unwrap(); }
        if mode == "stale" {
            c.delete_objects(&[body_id]).unwrap();
            let after = c.document().clone();
            assert!(c.submit("3").is_err());
            assert_eq!(c.document(), &after);
            c.cancel();
            continue;
        }
        assert_eq!(c.solid_preview([expected[0] + 3., expected[1], expected[2]]).len(), 192);
        c.submit("3").unwrap();
        let sphere = c.document().entities.values().find(|e| e.id != body_id).unwrap();
        let center = sphere.placement.point([0., 0., 0.]).unwrap();
        assert!((0..3).all(|i| (center[i] - expected[i]).abs() < 1e-7), "{mode}: {center:?} {expected:?}");
        c.undo().unwrap();
        assert_eq!(c.document().entities.len(), 1);
    }
}

#[test]
fn solid_sphere_aroundcurve_fraction_and_preview_match_committed_geometry() {
    use om9_core::geometry::{Curve, Transform};
    let mut rail = Entity::new("arc", Geometry::Curve(Curve::Circle {
        center: [0., 0., 0.], x: [1., 0., 0.], y: [0., 1., 0.], radius: 10.,
        domain: [0., std::f64::consts::TAU], angle_domain: None,
    }), None);
    rail.placement = Transform::translation([100., 20., 0.]);
    let id = rail.id;
    let mut doc = Document::new("aroundcurve fraction and preview");
    doc.entities.insert(id, rail);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("Sphere").unwrap();
    c.submit("AroundCurve").unwrap();
    c.submit("OnCurve=0.125").unwrap();
    let component = 10. / 2_f64.sqrt();
    let center = [100. + component, 20. + component, 0.];
    let picked = [center[0], center[1], 5.];
    let preview = c.solid_preview(picked);
    assert_eq!(preview.len(), 192);
    for segment in &preview {
        for p in segment {
            let radius = (0..3).map(|i| (p[i] - center[i]).powi(2)).sum::<f64>().sqrt();
            assert!((radius - 5.).abs() < 1e-8, "{radius}");
        }
    }
    c.accept_world_point(picked).unwrap();
    let sphere = c.document().entities.values().find(|e| e.id != id).unwrap();
    let actual = sphere.placement.point([0., 0., 0.]).unwrap();
    assert!((0..3).all(|i| (actual[i] - center[i]).abs() < 1e-8), "{actual:?}");
    assert!((measures(&c, sphere.id).volume_mm3 - 4. * std::f64::consts::PI / 3. * 125.).abs() < 1e-6);
    c.undo().unwrap();
    assert_eq!(c.document().entities.len(), 1);
}

#[test]
fn solid_sphere_around_curve_projects_world_pick_for_all_routes() {
    use om9_core::geometry::{Curve, Transform};
    for route in ["viewport", "path", "curve", "endpoint", "circle"] {
        let geometry = if route == "circle" {
            Curve::Circle { center: [0., 0., 0.], x: [1., 0., 0.], y: [0., 1., 0.], radius: 10., domain: [0., std::f64::consts::TAU], angle_domain: None }
        } else {
            om9_core::curve_build::polyline(&[[0., 0., 0.], [20., 0., 0.]]).unwrap()
        };
        let mut rail = Entity::new("rail", Geometry::Curve(geometry), None);
        rail.placement = Transform::translation([100., 20., 0.]);
        let id = rail.id;
        let mut doc = Document::new("sphere projected pick");
        doc.entities.insert(id, rail);
        let mut c = Controller::new(doc, Limits::default()).unwrap();
        if route == "viewport" { c.select_objects(&[id]).unwrap(); }
        c.submit("Sphere").unwrap();
        c.submit("AroundCurve").unwrap();
        match route {
            "viewport" => c.accept_world_point([107.123, 23., 0.]).unwrap(),
            "path" => c.submit("Path=rail.Edge1@107.123,23,0").unwrap(),
            "curve" => c.submit("Curve=rail.Edge1@107.123,23,0").unwrap(),
            "endpoint" => c.submit("Path=rail.Edge1@130,23,0").unwrap(),
            _ => c.submit("Path=rail.Edge1@109,32,0").unwrap(),
        }
        c.submit("5").unwrap();
        let sphere = c.document().entities.values().find(|e| e.id != id).unwrap();
        let center = sphere.placement.point([0., 0., 0.]).unwrap();
        let expected = match route { "endpoint" => [120., 20., 0.], "circle" => [106., 28., 0.], _ => [107.123, 20., 0.] };
        assert!((0..3).all(|i| (center[i] - expected[i]).abs() < 1e-6), "{route}: {center:?}");
        c.undo().unwrap();
        assert_eq!(c.document().entities.len(), 1);
    }
}

#[test]
fn solid_sphere_path_accepts_a_world_pick() {
    let mut doc = Document::new("path pick");
    let curve = om9_core::curve_build::polyline(&[[0.0, 0.0, 0.0], [20.0, 0.0, 0.0]]).unwrap();
    let entity = Entity::new("path", Geometry::Curve(curve), None);
    let curve_id = entity.id;
    doc.entities.insert(curve_id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.submit("Sphere").unwrap();
    c.submit("AroundCurve").unwrap();
    c.submit("Path=path.Edge1@10,0,0").unwrap();
    c.submit("5").unwrap();
    let id = *c
        .document()
        .entities
        .keys()
        .find(|id| **id != curve_id)
        .unwrap();
    assert_eq!(
        c.document().entities[&id].placement,
        om9_core::geometry::Transform::translation([10.0, 0.0, 0.0])
    );
}

#[test]
fn solid_sphere_fit_points_accepts_a_selection_batch() {
    let mut doc = Document::new("fit selection");
    let mut ids = Vec::new();
    for point in [
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, 1.0, 1.0],
    ] {
        let entity = Entity::new("p", Geometry::Point(point), None);
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&ids).unwrap();
    c.submit("Sphere").unwrap();
    c.submit("FitPoints").unwrap();
    c.submit("Selection").unwrap();
    c.submit("").unwrap();
    assert_eq!(c.document().entities.len(), 5);
}

#[test]
fn solid_sphere_exposes_a_three_circle_wire_preview() {
    let mut c = controller();
    c.submit("Sphere").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    let wire = c.solid_preview([3.0, 0.0, 0.0]);
    assert_eq!(wire.len(), 192);
    for [a, _] in wire {
        let radius = (a[0].powi(2) + a[1].powi(2) + a[2].powi(2)).sqrt();
        assert!((radius - 3.0).abs() < 1e-9);
    }
}

#[test]
fn solid_sphere_two_point_previews_the_diameter_sphere() {
    let mut c = controller();
    c.submit("Sphere").unwrap();
    c.submit("2Point").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    let wire = c.solid_preview([10.0, 0.0, 0.0]);
    assert_eq!(wire.len(), 192);
    for [a, _] in wire {
        let distance = ((a[0] - 5.0).powi(2) + a[1].powi(2) + a[2].powi(2)).sqrt();
        assert!((distance - 5.0).abs() < 1e-9);
    }
}

#[test]
fn solid_sphere_three_point_previews_the_circumcircle_sphere() {
    let mut c = controller();
    c.submit("Sphere").unwrap();
    c.submit("3Point").unwrap();
    c.accept_world_point([1.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([-1.0, 0.0, 0.0]).unwrap();
    let wire = c.solid_preview([0.0, 1.0, 0.0]);
    assert_eq!(wire.len(), 192);
    for [a, _] in wire {
        let radius = (a[0].powi(2) + a[1].powi(2) + a[2].powi(2)).sqrt();
        assert!((radius - 1.0).abs() < 1e-9);
    }
}

#[test]
fn solid_box_center_dimensions_preview_uses_length_and_width() {
    let mut c = controller();
    c.submit("Box").unwrap();
    c.submit("Center").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.submit("Length=10").unwrap();
    c.submit("Width=20").unwrap();
    let wire = c.solid_preview([0.0, 0.0, 5.0]);
    assert_eq!(wire.len(), 12);
    let xs: Vec<f64> = wire.iter().flat_map(|[a, b]| [a[0], b[0]]).collect();
    let ys: Vec<f64> = wire.iter().flat_map(|[a, b]| [a[1], b[1]]).collect();
    assert!((xs.iter().cloned().fold(f64::MIN, f64::max) - 5.0).abs() < 1e-9);
    assert!((xs.iter().cloned().fold(f64::MAX, f64::min) + 5.0).abs() < 1e-9);
    assert!((ys.iter().cloned().fold(f64::MIN, f64::max) - 10.0).abs() < 1e-9);
    assert!((ys.iter().cloned().fold(f64::MAX, f64::min) + 10.0).abs() < 1e-9);
}

#[test]
fn solid_sphere_curve_alias_accepts_a_world_pick() {
    let mut doc = Document::new("curve pick");
    let curve = om9_core::curve_build::polyline(&[[0.0, 0.0, 0.0], [20.0, 0.0, 0.0]]).unwrap();
    let entity = Entity::new("path", Geometry::Curve(curve), None);
    let curve_id = entity.id;
    doc.entities.insert(curve_id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.submit("Sphere").unwrap();
    c.submit("AroundCurve").unwrap();
    c.submit("Curve=path.Edge1@10,0,0").unwrap();
    c.submit("5").unwrap();
    let id = *c
        .document()
        .entities
        .keys()
        .find(|id| **id != curve_id)
        .unwrap();
    assert_eq!(
        c.document().entities[&id].placement,
        om9_core::geometry::Transform::translation([10.0, 0.0, 0.0])
    );
}

#[test]
fn solid_sphere_exposes_radius_and_diameter_tokens() {
    let mut c = controller();
    c.submit("Sphere").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    let tokens = c.solid_option_tokens();
    for expected in ["Radius=", "Diameter="] {
        assert!(tokens.iter().any(|(_, command)| *command == expected));
    }
}

#[test]
fn solid_box_exposes_width_and_height_tokens() {
    let mut c = controller();
    c.submit("Box").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    let tokens = c.solid_option_tokens();
    for expected in ["Width=", "Height="] {
        assert!(tokens.iter().any(|(_, command)| *command == expected));
    }
}

#[test]
fn solid_box_center_exposes_length_token() {
    let mut c = controller();
    c.submit("Box").unwrap();
    c.submit("Center").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    assert!(
        c.solid_option_tokens()
            .iter()
            .any(|(_, command)| *command == "Length=")
    );
}

#[test]
fn solid_box_height_can_be_picked_from_a_vertical_plane() {
    let mut c = controller();
    c.submit("Box").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([10.0, 20.0, 0.0]).unwrap();
    let ray = om9_scene::camera::Ray {
        origin: [5.0, -100.0, 7.0],
        direction: [0.0, 1.0, 0.0],
    };
    let point = c.box_height_ray_point(&ray).unwrap();
    assert!((point[2] - 7.0).abs() < 1e-9);
    c.accept_world_point(point).unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    assert!((measures(&c, id).volume_mm3 - 1400.0).abs() < 1e-6);
}

#[test]
fn box_center_mode_uses_the_shared_choice_option() {
    use om9_app::command_options::{CommandOptionEffect, CommandOptionInterface};
    let mut c = controller();
    c.submit("Box").unwrap();
    let center = c
        .command_options()
        .into_iter()
        .find(|option| option.id.name() == "Center")
        .expect("Center option");
    assert_eq!(
        c.activate_command_option(center.id).unwrap(),
        CommandOptionEffect::Applied
    );
    assert!(c.prompt().contains("Center"));
    c.cancel();
    assert!(c.document().entities.is_empty());
    assert_eq!(c.coordinator().undo_len(), 0);
}

#[test]
fn sphere_two_point_mode_uses_the_shared_choice_option() {
    use om9_app::command_options::{CommandOptionEffect, CommandOptionInterface};
    let mut c = controller();
    c.submit("Sphere").unwrap();
    let two = c
        .command_options()
        .into_iter()
        .find(|option| option.id.name() == "2Point")
        .expect("2Point option");
    assert_eq!(
        c.activate_command_option(two.id).unwrap(),
        CommandOptionEffect::Applied
    );
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([6.0, 0.0, 0.0]).unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    let measured = measures(&c, id);
    assert!((measured.volume_mm3 - 4.0 * std::f64::consts::PI / 3.0 * 27.0).abs() < 1e-6);
}
