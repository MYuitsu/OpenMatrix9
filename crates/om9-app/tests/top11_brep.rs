// SPDX-License-Identifier: LGPL-2.1-or-later
//! BRep mirror through the Top 11 adapter (OCCT kernel feature).
#![cfg(feature = "occt-brep")]

use om9_app::{command_options::CommandOptionInterface, session::Controller};
use om9_core::{
    budget::Limits,
    geometry::{Curve, Geometry, NurbsCurve, Transform},
    model::{Document, Entity},
};
use om9_kernel::creation::{self, Primitive};

fn box_brep() -> om9_core::brep::Brep {
    creation::primitive(
        Primitive::Box {
            x_mm: 2.0,
            y_mm: 3.0,
            z_mm: 4.0,
        },
        creation::DEFAULT_DEFLECTION_MM,
        &Limits::default(),
    )
    .expect("box")
}

#[test]
fn trim_closed_curve_with_brep_cutter_at_seam_keeps_correct_interval() {
    use om9_core::{curve_ops,top11::Top11Action};
    let limits=Limits::default();let body=creation::primitive(Primitive::Box{x_mm:6.,y_mm:20.,z_mm:2.},creation::DEFAULT_DEFLECTION_MM,&limits).unwrap();
    for kind in 0..5 {for toolbar in [false,true] {
        let curve=if kind==0 {Curve::Circle{center:[0.;3],x:[1.,0.,0.],y:[0.,1.,0.],radius:5.,domain:[0.,std::f64::consts::TAU],angle_domain:None}} else {let mut n=om9_core::curve_build::ellipse([0.;3],[1.,0.,0.],[0.,1.,0.],5.,5.,&limits).unwrap();n.reversed=kind==2||kind==4;let curve=Curve::Nurbs(n);if kind>=3 {let(f,_,d)=om9_core::curve_surface_ops::closest_point_on_curve(&curve,[-1.,24_f64.sqrt(),0.],&limits).unwrap();assert!(d<1e-7);curve_ops::set_seam(&curve,f,1e-9,&limits).unwrap()}else{curve}};
        let target=Entity::new("closed",Geometry::Curve(curve),None);let mut cutter=Entity::new("box",Geometry::Brep(body.clone()),None);cutter.placement=Transform::translation([-1.,-10.,-1.]);let ids=[target.id,cutter.id];
        let mut doc=Document::with_template_tolerance("closed curve brep seam");doc.entities.insert(ids[0],target);doc.entities.insert(ids[1],cutter);let mut c=Controller::new(doc,limits).unwrap();c.select_objects(&ids).unwrap();let before=c.document().clone();
        if toolbar {c.top11_invoke(Top11Action::Trim).unwrap_or_else(|e|panic!("invoke kind={kind} {e:?}"));}else{c.submit("Trim").unwrap_or_else(|e|panic!("invoke kind={kind} {e:?}"));}let d=5./2_f64.sqrt();c.top11_trim_many_accept_pick(ids[0],[d,d,0.]).unwrap_or_else(|e|panic!("pick kind={kind} {e:?}"));c.submit("").unwrap();
        let after=c.document().clone();let pieces:Vec<_>=after.entities.iter().filter(|(id,_)|**id!=ids[1]).collect();assert_eq!(pieces.len(),1);let Geometry::Curve(curve)=&pieces[0].1.geometry else{panic!("curve")};
        let ends=curve_ops::curve_endpoints(curve,&limits).unwrap();for p in [[5.,0.],[-1.,24_f64.sqrt()]]{assert!(ends.iter().any(|q|(q[0]-p[0]).hypot(q[1]-p[1])<1e-6),"kind={kind} {ends:?}");}
        let expected=5.*(std::f64::consts::TAU-(-0.2_f64).acos());assert!((curve_ops::arc_length(curve,&limits).unwrap()-expected).abs()<1e-5);
        let path=std::env::temp_dir().join(format!("om9-trim-brep-seam-{}.om9",uuid::Uuid::new_v4()));om9_archive::Archive::save(&path,&after,&limits,&om9_core::transaction::Cancellation::default()).unwrap();assert_eq!(om9_archive::Archive::open(&path,&limits).unwrap(),after);std::fs::remove_file(path).unwrap();c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }}
}

#[test]
fn brep_cutter_splits_long_line_near_endpoint_without_losing_faces() {
    use om9_core::top11::Top11Action;
    let limits=Limits::default();let brep=creation::primitive(Primitive::Box{x_mm:0.05,y_mm:2.,z_mm:2.},creation::DEFAULT_DEFLECTION_MM,&limits).unwrap();
    let curve=Curve::Line{start:[-0.05,1.,1.],end:[999_999.95,1.,1.],domain:[0.,1.]};
    let fractions=om9_kernel::intersect::curve_brep_fractions(&brep,&curve,0.001,&limits).unwrap();assert_eq!(fractions.len(),2,"{fractions:?}");
    for toolbar in [false,true] {
        let target=Entity::new("long line",Geometry::Curve(curve.clone()),None);let id=target.id;let cutter=Entity::new("box",Geometry::Brep(brep.clone()),None);let cutter_id=cutter.id;
        let mut doc=Document::with_template_tolerance("near endpoint brep");doc.entities.insert(id,target);doc.entities.insert(cutter_id,cutter);
        let mut c=Controller::new(doc,limits).unwrap();c.select_objects(&[id]).unwrap();let before=c.document().clone();
        if toolbar {c.top11_invoke(Top11Action::Split).unwrap();}else{c.submit("Split").unwrap();}c.top11_accept_object(cutter_id).unwrap();c.submit("").unwrap();
        let after=c.document().clone();assert_eq!(after.entities.len(),4);
        let mut lengths:Vec<_>=after.entities.iter().filter(|(id,_)|**id!=cutter_id).map(|(_,e)|match &e.geometry{Geometry::Curve(curve)=>om9_core::curve_ops::arc_length(curve,&limits).unwrap(),_=>panic!("curve")}).collect();
        lengths.sort_by(f64::total_cmp);assert!((lengths[0]-0.05).abs()<1e-7);assert!((lengths[1]-0.05).abs()<1e-7);
        c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }
}

#[test]
fn split_isocurve_directions_use_surface_parameters_and_one_undo() {
    use om9_core::top11::Top11Action;
    let limits = Limits::default();
    for (direction, expected) in [("U", 2), ("V", 2), ("Both", 4)] {
        let surface = om9_core::surface_kernel::plane([0., 0., 0.], [1., 0., 0.], [0., 1., 0.], 10., 20., &limits).unwrap();
        let mut target = Entity::new("surface", Geometry::Surface(surface), None);
        target.placement = Transform::translation([30., 40., 5.]);
        let id = target.id;
        let mut doc = Document::with_template_tolerance("split isocurve");
        doc.entities.insert(id, target);
        let mut c = Controller::new(doc, limits).unwrap();
        let before = c.document().clone();
        c.select_objects(&[id]).unwrap();
        c.top11_invoke(Top11Action::Split).unwrap();
        assert!(c.command_options().iter().any(|o| o.id.name() == "Isocurve"));
        c.submit("Isocurve").unwrap();
        c.submit(direction).unwrap();
        assert!(c.submit("W32,46,100").is_err());
        assert!(c.submit("W100,100,5").is_err());
        assert_eq!(c.document(), &before);
        c.submit("W32,46,5").unwrap();
        let pieces: Vec<_> = c.document().entities.values().collect();
        assert_eq!(pieces.len(), expected);
        for piece in &pieces {
            assert_eq!(piece.placement, Transform::translation([30., 40., 5.]));
            let Geometry::Surface(surface) = &piece.geometry else { panic!("expected surface") };
            let [u, v] = surface.domain;
            if direction != "U" { assert!(u[1] <= 0.2 + 1e-6 || u[0] >= 0.2 - 1e-6, "{u:?}"); }
            if direction != "V" { assert!(v[1] <= 0.3 + 1e-6 || v[0] >= 0.3 - 1e-6, "{v:?}"); }
        }
        let after = c.document().clone();
        c.undo().unwrap(); assert_eq!(c.document(), &before);
        c.redo().unwrap(); assert_eq!(c.document(), &after);
    }
}

#[test]
fn split_isocurve_option_toggle_and_cancel_keep_the_document() {
    use om9_app::command_options::CommandValue;
    let limits = Limits::default();
    let surface = om9_core::surface_kernel::plane([0., 0., 0.], [1., 0., 0.], [0., 1., 0.], 10., 20., &limits).unwrap();
    let target = Entity::new("surface", Geometry::Surface(surface), None);
    let id = target.id;
    let mut doc = Document::with_template_tolerance("isocurve option"); doc.entities.insert(id, target);
    let mut c = Controller::new(doc, limits).unwrap();
    let before = c.document().clone();
    c.submit("Split").unwrap(); c.top11_accept_object(id).unwrap(); c.submit("").unwrap();
    let option = c.command_options().into_iter().find(|o| o.id.name() == "Isocurve").unwrap();
    c.activate_command_option(option.id).unwrap();
    let direction = c.command_options().into_iter().find(|o| o.id.name() == "Direction").unwrap();
    assert_eq!(direction.value, CommandValue::Choice("U"));
    c.activate_command_option(direction.id).unwrap();
    let direction = c.command_options().into_iter().find(|o| o.id.name() == "Direction").unwrap();
    assert_eq!(direction.value, CommandValue::Choice("V"));
    c.submit("Both").unwrap();
    assert_eq!(c.command_options().into_iter().find(|o| o.id.name() == "Direction").unwrap().value, CommandValue::Choice("Both"));
    c.cancel();
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), 0);
}

#[test]
fn split_isocurve_preserves_rational_surface_under_affine_placement() {
    use om9_core::geometry::NurbsSurface;
    let limits = Limits::default();
    let surface = NurbsSurface {
        degree: [2, 2], size: [3, 3],
        poles: (0..3).flat_map(|u| (0..3).map(move |v| [u as f64 * 5., v as f64 * 4., (u * v) as f64])).collect(),
        weights: vec![1., 0.8, 1., 0.9, 0.6, 0.9, 1., 0.8, 1.],
        knots: [vec![0., 0., 0., 1., 1., 1.], vec![0., 0., 0., 1., 1., 1.]],
        domain: [[0., 1.], [0., 1.]], periodic: [false, false], reversed: false,
    };
    let pick = om9_core::surface_kernel::evaluate(&surface, 0.35, 0.7, &limits).unwrap();
    let mut target = Entity::new("rational", Geometry::Surface(surface.clone()), None);
    target.placement = Transform::scale([2., 0.5, 3.]).unwrap();
    let pick = target.placement.point(pick).unwrap();
    let id = target.id;
    let mut doc = Document::with_template_tolerance("rational split"); doc.entities.insert(id, target);
    let mut c = Controller::new(doc, limits).unwrap();
    let before = c.document().clone();
    c.select_objects(&[id]).unwrap(); c.submit("Split").unwrap(); c.submit("Isocurve").unwrap(); c.submit("Both").unwrap();
    c.accept_world_point(pick).unwrap();
    assert_eq!(c.document().entities.len(), 4);
    for entity in c.document().entities.values() {
        let Geometry::Surface(piece) = &entity.geometry else { panic!("surface") };
        assert_eq!(piece.poles, surface.poles);
        assert_eq!(piece.weights, surface.weights);
        assert_eq!(piece.knots, surface.knots);
        let uv = [(piece.domain[0][0] + piece.domain[0][1]) / 2., (piece.domain[1][0] + piece.domain[1][1]) / 2.];
        let a = om9_core::surface_kernel::evaluate(&surface, uv[0], uv[1], &limits).unwrap();
        let b = om9_core::surface_kernel::evaluate(piece, uv[0], uv[1], &limits).unwrap();
        for i in 0..3 { assert!((a[i] - b[i]).abs() < 1e-10); }
    }
    let path = std::env::temp_dir().join(format!("om9-isocurve-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, c.document(), &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    let loaded = om9_archive::Archive::open(&path, &limits).unwrap();
    std::fs::remove_file(path).unwrap(); assert_eq!(&loaded, c.document());
    c.undo().unwrap(); assert_eq!(c.document(), &before);
}

#[test]
fn split_isocurve_shrink_yes_restricts_rational_surface_exactly() {
    use om9_core::geometry::NurbsSurface;
    use om9_app::command_options::CommandValue;
    let limits = Limits::default();
    let surface = NurbsSurface {
        degree: [2, 2], size: [3, 3],
        poles: (0..3).flat_map(|u| (0..3).map(move |v| [u as f64 * 5., v as f64 * 4., (u * v) as f64])).collect(),
        weights: vec![1., 0.8, 1., 0.9, 0.6, 0.9, 1., 0.8, 1.],
        knots: [vec![0., 0., 0., 1., 1., 1.], vec![0., 0., 0., 1., 1., 1.]],
        domain: [[0., 1.], [0., 1.]], periodic: [false, false], reversed: false,
    };
    let pick = om9_core::surface_kernel::evaluate(&surface, 0.35, 0.7, &limits).unwrap();
    let target = Entity::new("surface", Geometry::Surface(surface.clone()), None); let id = target.id;
    let mut doc = Document::with_template_tolerance("isocurve shrink"); doc.entities.insert(id, target);
    let mut c = Controller::new(doc, limits).unwrap(); let before = c.document().clone();
    c.select_objects(&[id]).unwrap(); c.submit("Split").unwrap(); c.submit("Isocurve").unwrap(); c.submit("Both").unwrap();
    let shrink = c.command_options().into_iter().find(|o| o.id.name() == "Shrink").unwrap();
    assert_eq!(shrink.value, CommandValue::Flag(false));
    assert!(c.submit("Shrink=Maybe").is_err());
    c.activate_command_option(shrink.id).unwrap();
    assert_eq!(c.command_options().into_iter().find(|o| o.id.name() == "Shrink").unwrap().value, CommandValue::Flag(true));
    c.accept_world_point(pick).unwrap();
    assert_eq!(c.document().entities.len(), 4);
    for entity in c.document().entities.values() {
        let Geometry::Surface(piece) = &entity.geometry else { panic!("surface") };
        for axis in 0..2 {
            assert!((piece.knots[axis][0] - piece.domain[axis][0]).abs() < 1e-9);
            assert!((*piece.knots[axis].last().unwrap() - piece.domain[axis][1]).abs() < 1e-9);
        }
        let uv = [(piece.domain[0][0] + piece.domain[0][1]) / 2., (piece.domain[1][0] + piece.domain[1][1]) / 2.];
        let a = om9_core::surface_kernel::evaluate(&surface, uv[0], uv[1], &limits).unwrap();
        let b = om9_core::surface_kernel::evaluate(piece, uv[0], uv[1], &limits).unwrap();
        for i in 0..3 { assert!((a[i] - b[i]).abs() < 1e-9); }
    }
    let after = c.document().clone();
    let path = std::env::temp_dir().join(format!("om9-isocurve-shrink-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, &after, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    let loaded = om9_archive::Archive::open(&path, &limits).unwrap();
    std::fs::remove_file(path).unwrap(); assert_eq!(loaded, after);
    c.undo().unwrap(); assert_eq!(c.document(), &before);
    c.redo().unwrap(); assert_eq!(c.document(), &after);
}

#[test]
fn split_isocurve_on_placed_single_face_brep_preserves_area_and_undo() {
    let limits = Limits::default();
    for direction in ["U", "V", "Both"] {
        let profile = creation::Profile::new(vec![[0., 0., 0.], [10., 0., 0.], [10., 20., 0.], [0., 20., 0.]]).unwrap();
        let face = creation::planar(&profile, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
        let mut target = Entity::new("face", Geometry::Brep(face), None);
        target.placement = Transform::translation([20., 30., 5.]);
        let id = target.id;
        let mut doc = Document::with_template_tolerance("split BRep isocurve"); doc.entities.insert(id, target);
        let mut c = Controller::new(doc, limits).unwrap();
        let before = c.document().clone();
        c.select_objects(&[id]).unwrap(); c.submit("Split").unwrap();
        assert!(c.command_options().iter().any(|o| o.id.name() == "Isocurve"));
        c.submit("Isocurve").unwrap(); c.submit(direction).unwrap();
        assert!(c.submit("W23,37,20").is_err());
        c.submit("W23,37,5").unwrap();
        assert_eq!(c.document().entities.len(), if direction == "Both" { 4 } else { 2 });
        let mut area = 0.;
        let mut areas = Vec::new();
        for entity in c.document().entities.values() {
            assert_eq!(entity.placement, Transform::translation([20., 30., 5.]));
            let Geometry::Brep(brep) = &entity.geometry else { panic!("expected BRep") };
            let measured = creation::measure(brep, &limits).unwrap().area_mm2;
            area += measured;
            areas.push(measured);
        }
        assert!((area - 200.).abs() < 1e-6, "{area}");
        if direction == "Both" {
            areas.sort_by(f64::total_cmp);
            for (actual, expected) in areas.iter().zip([21., 39., 49., 91.]) { assert!((actual - expected).abs() < 1e-6, "{areas:?}"); }
        }
        let after = c.document().clone();
        if direction == "Both" {
            let path = std::env::temp_dir().join(format!("om9-brep-isocurve-{}.om9", uuid::Uuid::new_v4()));
            om9_archive::Archive::save(&path, &after, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
            let loaded = om9_archive::Archive::open(&path, &limits).unwrap();
            std::fs::remove_file(path).unwrap(); assert_eq!(loaded, after);
        }
        c.undo().unwrap(); assert_eq!(c.document(), &before);
        c.redo().unwrap(); assert_eq!(c.document(), &after);
    }
}

#[test]
fn split_brep_isocurve_shrink_restricts_underlying_surface() {
    use om9_app::command_options::CommandValue;
    let limits = Limits::default();
    let profile = creation::Profile::new(vec![[0., 0., 0.], [10., 0., 0.], [10., 20., 0.], [0., 20., 0.]]).unwrap();
    let face = creation::planar(&profile, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
    let target = Entity::new("face", Geometry::Brep(face), None); let id = target.id;
    let mut doc = Document::with_template_tolerance("BRep isocurve shrink"); doc.entities.insert(id, target);
    let mut c = Controller::new(doc, limits).unwrap(); let before = c.document().clone();
    c.select_objects(&[id]).unwrap(); c.submit("Split").unwrap(); c.submit("Isocurve").unwrap(); c.submit("Both").unwrap();
    let shrink = c.command_options().into_iter().find(|o| o.id.name() == "Shrink").unwrap();
    assert_eq!(shrink.value, CommandValue::Flag(false)); c.activate_command_option(shrink.id).unwrap();
    c.accept_world_point([3., 7., 0.]).unwrap();
    assert_eq!(c.document().entities.len(), 4);
    for entity in c.document().entities.values() {
        let Geometry::Brep(brep) = &entity.geometry else { panic!("BRep") };
        let bounds = om9_kernel::measurement::face_underlying_bounds(brep, &limits).unwrap();
        assert!((bounds[1]-bounds[0]) <= 10.0001 && (bounds[3]-bounds[2]) <= 20.0001, "{bounds:?}");
    }
    let after = c.document().clone(); c.undo().unwrap(); assert_eq!(c.document(), &before);
    c.redo().unwrap(); assert_eq!(c.document(), &after);
}

#[test]
fn shrink_brep_nurbs_face_preserves_exact_geometry() {
    use om9_core::geometry::NurbsSurface;
    use om9_core::surface_kernel::SurfaceIso;
    let limits = Limits::default();
    let surface = NurbsSurface {
        degree: [2, 2], size: [3, 3],
        poles: (0..3).flat_map(|u| (0..3).map(move |v| [u as f64 * 5., v as f64 * 4., (u * v) as f64])).collect(),
        weights: vec![1., 0.8, 1., 0.9, 0.6, 0.9, 1., 0.8, 1.],
        knots: [vec![0., 0., 0., 1., 1., 1.], vec![0., 0., 0., 1., 1., 1.]],
        domain: [[0., 1.], [0., 1.]], periodic: [false, false], reversed: false,
    };
    let iso = Geometry::Curve(Curve::Nurbs(om9_core::surface_kernel::isocurve(&surface, SurfaceIso::U, 0.5, &limits).unwrap()));
    let partition = om9_kernel::measurement::split_surface_by_geometry(&Geometry::Surface(surface.clone()), Transform::identity(),
        &[(&iso, Transform::identity())], creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
    assert_eq!(partition.pieces, 2);
    for index in 0..2 {
        let face = om9_kernel::editing::extract_face(&partition.brep, index, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
        let before = om9_kernel::measurement::face_underlying_bounds(&face, &limits).unwrap();
        let result = om9_kernel::measurement::shrink_face(&face, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
        let after = om9_kernel::measurement::face_underlying_bounds(&result, &limits).unwrap();
        assert!((after[3] - after[2]) < (before[3] - before[2]) - 0.1, "{before:?} {after:?}");
        let area_a = creation::measure(&face, &limits).unwrap().area_mm2;
        let area_b = creation::measure(&result, &limits).unwrap().area_mm2;
        assert!((area_a - area_b).abs() < 1e-6);
        let uv = [(after[0] + after[1]) / 2., (after[2] + after[3]) / 2.];
        let point = om9_core::surface_kernel::evaluate(&surface, uv[0], uv[1], &limits).unwrap();
        let distance = om9_kernel::measurement::distance(&Geometry::Brep(result), Transform::identity(),
            &Geometry::Point(point), Transform::identity(), &limits).unwrap().millimetres;
        assert!(distance < 1e-6, "{distance}");
    }
}

#[test]
fn split_isocurve_preserves_trimmed_face_hole() {
    use std::f64::consts::{PI, TAU};
    let limits = Limits::default();
    let profile = creation::Profile::new(vec![[0., 0., 0.], [10., 0., 0.], [10., 20., 0.], [0., 20., 0.]]).unwrap();
    let target = Entity::new("face", Geometry::Brep(creation::planar(&profile, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap()), None);
    let cutter = Entity::new("circle", Geometry::Curve(Curve::Circle { center: [5., 10., 0.], x: [1., 0., 0.], y: [0., 1., 0.], radius: 2., domain: [0., TAU], angle_domain: None }), None);
    let ids = [target.id, cutter.id];
    let mut doc = Document::with_template_tolerance("isocurve trimmed hole");
    for entity in [target, cutter] { doc.entities.insert(entity.id, entity); }
    let mut c = Controller::new(doc, limits).unwrap();
    c.select_objects(&ids).unwrap(); c.submit("Trim").unwrap(); c.accept_world_point([5., 10., 0.]).unwrap(); c.submit("").unwrap();
    let before = c.document().clone();
    let trimmed = &c.document().entities[&ids[0]];
    assert!(om9_kernel::measurement::surface_ray_hit(&trimmed.geometry, trimmed.placement,
        [5., 10., 10.], [0., 0., -1.], [5., 10., 0.], 0.001, &limits).is_err());
    c.select_objects(&[ids[0]]).unwrap(); c.submit("Split").unwrap(); c.submit("Isocurve").unwrap(); c.submit("Both").unwrap();
    assert!(c.accept_world_point([5., 10., 0.]).is_err());
    assert_eq!(c.document(), &before);
    c.accept_world_point([5., 5., 0.]).unwrap();
    let pieces: Vec<_> = c.document().entities.values().filter(|e| e.id != ids[1]).collect();
    assert_eq!(pieces.len(), 4);
    let area: f64 = pieces.iter().map(|e| {
        let Geometry::Brep(brep) = &e.geometry else { panic!("BRep") };
        creation::measure(brep, &limits).unwrap().area_mm2
    }).sum();
    assert!((area - (200. - 4. * PI)).abs() < 1e-6, "{area}");
    c.undo().unwrap(); assert_eq!(c.document(), &before);
}

#[test]
fn split_brep_isocurve_shrink_keeps_trimmed_hole_and_persistence() {
    use std::f64::consts::{PI, TAU};
    let limits = Limits::default();
    let profile = creation::Profile::new(vec![[0., 0., 0.], [10., 0., 0.], [10., 20., 0.], [0., 20., 0.]]).unwrap();
    let target = Entity::new("face", Geometry::Brep(creation::planar(&profile, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap()), None);
    let cutter = Entity::new("circle", Geometry::Curve(Curve::Circle { center: [5., 10., 0.], x: [1., 0., 0.], y: [0., 1., 0.], radius: 2., domain: [0., TAU], angle_domain: None }), None);
    let ids = [target.id, cutter.id];
    let mut doc = Document::with_template_tolerance("shrink hole");
    for entity in [target, cutter] { doc.entities.insert(entity.id, entity); }
    let mut c = Controller::new(doc, limits).unwrap();
    c.select_objects(&ids).unwrap(); c.submit("Trim").unwrap(); c.accept_world_point([5., 10., 0.]).unwrap(); c.submit("").unwrap();
    let before = c.document().clone();
    c.select_objects(&[ids[0]]).unwrap(); c.submit("Split").unwrap(); c.submit("Isocurve").unwrap(); c.submit("Both").unwrap(); c.submit("Shrink=Yes").unwrap();
    c.accept_world_point([5., 5., 0.]).unwrap();
    let pieces: Vec<_> = c.document().entities.values().filter(|e| e.id != ids[1]).collect();
    assert_eq!(pieces.len(), 4);
    let area: f64 = pieces.iter().map(|e| {
        let Geometry::Brep(brep) = &e.geometry else { panic!("BRep") };
        let bounds = om9_kernel::measurement::face_underlying_bounds(brep, &limits).unwrap();
        assert!((bounds[1]-bounds[0]) <= 10.0001 && (bounds[3]-bounds[2]) <= 20.0001);
        creation::measure(brep, &limits).unwrap().area_mm2
    }).sum();
    assert!((area - (200. - 4. * PI)).abs() < 1e-6);
    let distance = pieces.iter().map(|e| om9_kernel::measurement::distance(&e.geometry, e.placement,
        &Geometry::Point([5., 10., 0.]), Transform::identity(), &limits).unwrap().millimetres)
        .fold(f64::INFINITY, f64::min);
    assert!((distance - 2.).abs() < 1e-6, "{distance}");
    let after = c.document().clone();
    let path = std::env::temp_dir().join(format!("om9-brep-shrink-hole-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, &after, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    let loaded = om9_archive::Archive::open(&path, &limits).unwrap(); std::fs::remove_file(path).unwrap(); assert_eq!(loaded, after);
    c.undo().unwrap(); assert_eq!(c.document(), &before); c.redo().unwrap(); assert_eq!(c.document(), &after);
}

#[test]
fn split_brep_isocurve_shrink_preserves_nonuniform_placement() {
    let limits = Limits::default();
    let profile = creation::Profile::new(vec![[0., 0., 0.], [10., 0., 0.], [10., 20., 0.], [0., 20., 0.]]).unwrap();
    let mut target = Entity::new("scaled face", Geometry::Brep(creation::planar(&profile, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap()), None);
    target.placement = Transform([[2., 0., 0., 30.], [0., 0.5, 0., -7.], [0., 0., 3., 4.], [0., 0., 0., 1.]]);
    let id = target.id;
    let mut doc = Document::with_template_tolerance("scaled BRep shrink"); doc.entities.insert(id, target);
    let mut c = Controller::new(doc, limits).unwrap(); let before = c.document().clone();
    c.select_objects(&[id]).unwrap(); c.submit("Split").unwrap(); c.submit("Isocurve").unwrap(); c.submit("Both").unwrap(); c.submit("Shrink=Yes").unwrap();
    c.accept_world_point([36., -3.5, 4.]).unwrap();
    assert_eq!(c.document().entities.len(), 4);
    let mut areas: Vec<_> = c.document().entities.values().map(|e| {
        assert_eq!(e.placement, before.entities[&id].placement);
        let Geometry::Brep(brep) = &e.geometry else { panic!("BRep") };
        creation::measure(brep, &limits).unwrap().area_mm2
    }).collect();
    areas.sort_by(f64::total_cmp);
    for (actual, expected) in areas.iter().zip([21., 39., 49., 91.]) { assert!((actual-expected).abs() < 1e-5, "{areas:?}"); }
    for point in [[1., 1., 0.], [8., 15., 0.], [3., 7., 0.]] {
        let distance = c.document().entities.values().map(|e| om9_kernel::measurement::distance(&e.geometry, Transform::identity(),
            &Geometry::Point(point), Transform::identity(), &limits).unwrap().millimetres).fold(f64::INFINITY, f64::min);
        assert!(distance < 1e-6, "{point:?} {distance}");
    }
    c.undo().unwrap(); assert_eq!(c.document(), &before);
}

#[test]
fn split_isocurve_on_cylindrical_brep_keeps_exact_area() {
    let limits = Limits::default();
    let body = creation::primitive(Primitive::Cylinder { radius_mm: 5., height_mm: 10. }, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
    let body_measure = creation::measure(&body, &limits).unwrap();
    let side = (0..body_measure.faces).map(|index| om9_kernel::editing::extract_face(&body, index, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap())
        .find(|face| (creation::measure(face, &limits).unwrap().area_mm2 - 100. * std::f64::consts::PI).abs() < 1e-5).unwrap();
    let target = Entity::new("cylinder wall", Geometry::Brep(side), None);
    let id = target.id;
    let mut doc = Document::with_template_tolerance("cylinder isocurve"); doc.entities.insert(id, target);
    let mut c = Controller::new(doc, limits).unwrap();
    let before = c.document().clone();
    c.select_objects(&[id]).unwrap(); c.submit("Split").unwrap(); c.submit("Isocurve").unwrap(); c.submit("U").unwrap();
    assert!(!c.command_options().iter().any(|o| o.id.name() == "Shrink"));
    let a = 5. / 2_f64.sqrt();
    c.accept_world_point([a, a, 5.]).unwrap();
    assert_eq!(c.document().entities.len(), 2);
    for entity in c.document().entities.values() {
        let Geometry::Brep(brep) = &entity.geometry else { panic!("BRep") };
        assert!((creation::measure(brep, &limits).unwrap().area_mm2 - 50. * std::f64::consts::PI).abs() < 1e-5);
    }
    c.undo().unwrap(); assert_eq!(c.document(), &before);
}

#[test]
fn split_isocurve_viewport_refines_cylinder_hit_along_ray() {
    let limits = Limits::default();
    let body = creation::primitive(Primitive::Cylinder { radius_mm: 5., height_mm: 10. }, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
    let side = (0..creation::measure(&body, &limits).unwrap().faces)
        .map(|i| om9_kernel::editing::extract_face(&body, i, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap())
        .find(|face| (creation::measure(face, &limits).unwrap().area_mm2 - 100. * std::f64::consts::PI).abs() < 1e-5).unwrap();
    let target = Entity::new("wall", Geometry::Brep(side), None); let id = target.id;
    let mut doc = Document::with_template_tolerance("isocurve ray"); doc.entities.insert(id, target);
    let mut c = Controller::new(doc, limits).unwrap(); let before = c.document().clone();
    c.select_objects(&[id]).unwrap(); c.submit("Split").unwrap(); c.submit("Isocurve").unwrap(); c.submit("U").unwrap();
    let a = 5. / 2_f64.sqrt();
    let ray = om9_scene::camera::Ray { origin: [10., 10., 0.], direction: [a - 10., a - 10., 5.] };
    let display = [a + 0.02 * ray.direction[0], a + 0.02 * ray.direction[1], 5. + 0.02 * ray.direction[2]];
    let miss = om9_scene::camera::Ray { origin: [10., 10., 0.], direction: [1., 1., 0.] };
    assert!(c.top11_split_iso_accept_viewport_pick(id, display, miss).is_err());
    assert_eq!(c.document(), &before); assert!(c.is_running());
    let ambiguous = om9_scene::camera::Ray { origin: [-10., 0., 5.], direction: [1., 0., 0.] };
    assert!(c.top11_split_iso_accept_viewport_pick(id, [0., 0., 5.], ambiguous).is_err());
    assert_eq!(c.document(), &before); assert!(c.is_running());
    c.top11_split_iso_accept_viewport_pick(id, display, ray).unwrap();
    assert_eq!(c.document().entities.len(), 2);
    for entity in c.document().entities.values() {
        let Geometry::Brep(brep) = &entity.geometry else { panic!("BRep") };
        assert!((creation::measure(brep, &limits).unwrap().area_mm2 - 50. * std::f64::consts::PI).abs() < 1e-5);
    }
    c.undo().unwrap(); assert_eq!(c.document(), &before);
}

#[test]
fn split_isocurve_viewport_refines_typed_nurbs_hit_along_ray() {
    use om9_core::geometry::NurbsSurface;
    let limits = Limits::default();
    let surface = NurbsSurface {
        degree: [2, 2], size: [3, 3],
        poles: (0..3).flat_map(|u| (0..3).map(move |v| [u as f64 * 5., v as f64 * 4., (u * v) as f64])).collect(),
        weights: vec![1., 0.8, 1., 0.9, 0.6, 0.9, 1., 0.8, 1.],
        knots: [vec![0., 0., 0., 1., 1., 1.], vec![0., 0., 0., 1., 1., 1.]],
        domain: [[0., 1.], [0., 1.]], periodic: [false, false], reversed: false,
    };
    for placement in [Transform::identity(), Transform([[2., 0., 0., 20.], [0., 0.5, 0., -7.], [0., 0., 3., 4.], [0., 0., 0., 1.]])] {
    let point = placement.point(om9_core::surface_kernel::evaluate(&surface, 0.35, 0.7, &limits).unwrap()).unwrap();
    let mut target = Entity::new("surface", Geometry::Surface(surface.clone()), None); target.placement = placement; let id = target.id;
    let mut doc = Document::with_template_tolerance("typed isocurve ray"); doc.entities.insert(id, target);
    let mut c = Controller::new(doc, limits).unwrap(); let before = c.document().clone();
    c.select_objects(&[id]).unwrap(); c.submit("Split").unwrap(); c.submit("Isocurve").unwrap(); c.submit("Both").unwrap();
    let ray = om9_scene::camera::Ray { origin: [point[0], point[1], point[2] + 10.], direction: [0., 0., -1.] };
    let display = [point[0], point[1], point[2] - 0.5];
    c.top11_split_iso_accept_viewport_pick(id, display, ray).unwrap();
    assert_eq!(c.document().entities.len(), 4);
    for entity in c.document().entities.values() {
        let Geometry::Surface(surface) = &entity.geometry else { panic!("surface") };
        assert!(surface.domain[0].iter().any(|u| (*u - 0.35).abs() < 1e-6));
        assert!(surface.domain[1].iter().any(|v| (*v - 0.7).abs() < 1e-6));
    }
    c.undo().unwrap(); assert_eq!(c.document(), &before);
    }
}

#[test]
fn split_isocurve_on_nonuniformly_placed_brep_uses_world_point() {
    let limits = Limits::default();
    let profile = creation::Profile::new(vec![[0., 0., 0.], [10., 0., 0.], [10., 20., 0.], [0., 20., 0.]]).unwrap();
    let mut target = Entity::new("scaled face", Geometry::Brep(creation::planar(&profile, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap()), None);
    target.placement = Transform([[2., 0., 0., 30.], [0., 0.5, 0., -7.], [0., 0., 3., 4.], [0., 0., 0., 1.]]);
    let id = target.id;
    let mut doc = Document::with_template_tolerance("scaled BRep isocurve"); doc.entities.insert(id, target);
    let mut c = Controller::new(doc, limits).unwrap();
    let before = c.document().clone();
    c.select_objects(&[id]).unwrap(); c.submit("Split").unwrap(); c.submit("Isocurve").unwrap(); c.submit("Both").unwrap();
    c.accept_world_point([36., -3.5, 4.]).unwrap();
    assert_eq!(c.document().entities.len(), 4);
    let mut areas: Vec<_> = c.document().entities.values().map(|e| {
        assert_eq!(e.placement, before.entities[&id].placement);
        let Geometry::Brep(brep) = &e.geometry else { panic!("BRep") };
        creation::measure(brep, &limits).unwrap().area_mm2
    }).collect();
    areas.sort_by(f64::total_cmp);
    for (actual, expected) in areas.iter().zip([21., 39., 49., 91.]) { assert!((actual - expected).abs() < 1e-5, "{areas:?}"); }
    c.undo().unwrap(); assert_eq!(c.document(), &before);
}

#[test]
fn split_does_not_offer_isocurve_for_polysurface_or_multiple_targets() {
    let limits = Limits::default();
    let solid = Entity::new("box", Geometry::Brep(box_brep()), None);
    let a = Entity::new("a", Geometry::Surface(om9_core::surface_kernel::plane([0., 0., 0.], [1., 0., 0.], [0., 1., 0.], 10., 20., &limits).unwrap()), None);
    let b = Entity::new("b", a.geometry.clone(), None);
    let cylinder = Entity::new("periodic solid", Geometry::Brep(creation::primitive(Primitive::Cylinder { radius_mm: 5., height_mm: 10. }, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap()), None);
    let ids = [solid.id, a.id, b.id, cylinder.id];
    let mut doc = Document::with_template_tolerance("isocurve availability");
    for entity in [solid, a, b, cylinder] { doc.entities.insert(entity.id, entity); }
    let mut c = Controller::new(doc, limits).unwrap();
    for (selection, shrink_expected) in [(&ids[..1], true), (&ids[1..3], true), (&ids[3..], false)] {
        c.select_objects(selection).unwrap(); c.submit("Split").unwrap();
        assert!(!c.command_options().iter().any(|o| o.id.name() == "Isocurve"));
        assert_eq!(c.command_options().iter().any(|o| o.id.name() == "Shrink"), shrink_expected);
        c.cancel();
    }
}

#[test]
fn split_placed_surface_with_two_cutters_is_atomic_and_undoable() {
    use om9_core::top11::Top11Action;
    let limits = Limits::default();
    for toolbar in [false, true] {
        let profile = creation::Profile::new(vec![[0., 0., 0.], [10., 0., 0.], [10., 20., 0.], [0., 20., 0.]]).unwrap();
        let face = creation::planar(&profile, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
        let mut target = Entity::new("face", Geometry::Brep(face), None);
        target.placement = Transform::translation([20., 4., 2.]);
        let mut a = Entity::new("a", Geometry::Curve(Curve::Line { start: [3., -1., 0.], end: [3., 21., 0.], domain: [0., 1.] }), None);
        let mut b = Entity::new("b", Geometry::Curve(Curve::Line { start: [7., -1., 0.], end: [7., 21., 0.], domain: [0., 1.] }), None);
        a.placement = target.placement;
        b.placement = target.placement;
        let ids = [target.id, a.id, b.id];
        let mut doc = Document::with_template_tolerance("split face");
        for entity in [target, a, b] { doc.entities.insert(entity.id, entity); }
        let mut c = Controller::new(doc, limits.clone()).unwrap();
        let before = c.document().clone();
        if toolbar {
            c.select_objects(&[ids[0]]).unwrap();
            c.top11_invoke(Top11Action::Split).unwrap();
        } else {
            c.submit("Split").unwrap();
            c.top11_accept_object(ids[0]).unwrap();
            c.submit("").unwrap();
        }
        assert!(!c.command_options().iter().any(|o| o.id.name() == "Point"));
        c.top11_accept_object(ids[1]).unwrap();
        c.top11_accept_object(ids[2]).unwrap();
        assert_eq!(c.document(), &before);
        c.submit("").unwrap();
        assert!(!c.document().entities.contains_key(&ids[0]));
        let mut areas: Vec<_> = c.document().entities.values().filter(|e| !ids[1..].contains(&e.id)).map(|e| {
            assert_eq!(e.placement, Transform::translation([20., 4., 2.]));
            let Geometry::Brep(brep) = &e.geometry else { panic!("expected BRep") };
            creation::measure(brep, &limits).unwrap().area_mm2
        }).collect();
        areas.sort_by(|a, b| a.total_cmp(b));
        assert_eq!(areas.len(), 3);
        for (actual, expected) in areas.iter().zip([60., 60., 80.]) { assert!((actual - expected).abs() < 1e-6); }
        assert_eq!(c.document().entities[&ids[1]], before.entities[&ids[1]]);
        assert_eq!(c.document().entities[&ids[2]], before.entities[&ids[2]]);
        let after = c.document().clone();
        c.undo().unwrap();
        assert_eq!(c.document(), &before);
        c.redo().unwrap();
        assert_eq!(c.document(), &after);
    }
}

#[test]
fn split_mixed_targets_skips_missed_face_and_cancel_is_nonmutating() {
    use om9_core::geometry::NurbsSurface;
    let limits = Limits::default();
    let surface = || NurbsSurface {
        degree: [1, 1], size: [2, 2],
        poles: vec![[0., 0., 0.], [0., 20., 0.], [10., 0., 0.], [10., 20., 0.]],
        weights: vec![1.; 4], knots: [vec![0., 0., 1., 1.], vec![0., 0., 1., 1.]],
        domain: [[0., 1.], [0., 1.]], periodic: [false, false], reversed: false,
    };
    let target = Entity::new("surface", Geometry::Surface(surface()), None);
    let mut missed = Entity::new("missed", Geometry::Surface(surface()), None);
    missed.placement = Transform::translation([30., 0., 0.]);
    let curve = Entity::new("curve", Geometry::Curve(Curve::Line { start: [0., 10., 0.], end: [10., 10., 0.], domain: [0., 1.] }), None);
    let cutter = Entity::new("cutter", Geometry::Curve(Curve::Line { start: [5., -1., 0.], end: [5., 21., 0.], domain: [0., 1.] }), None);
    let ids = [target.id, missed.id, curve.id, cutter.id];
    let mut doc = Document::with_template_tolerance("split mixed targets");
    for entity in [target, missed, curve, cutter] { doc.entities.insert(entity.id, entity); }
    let mut c = Controller::new(doc, limits.clone()).unwrap();
    let before = c.document().clone();
    c.select_objects(&ids[..3]).unwrap();
    c.submit("Split").unwrap();
    c.top11_accept_object(ids[3]).unwrap();
    c.cancel();
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), 0);
    c.submit("Split").unwrap();
    c.top11_accept_object(ids[3]).unwrap();
    c.submit("").unwrap();
    assert_eq!(c.document().entities[&ids[1]], before.entities[&ids[1]]);
    assert_eq!(c.document().entities[&ids[3]], before.entities[&ids[3]]);
    assert!(!c.document().entities.contains_key(&ids[0]));
    assert!(!c.document().entities.contains_key(&ids[2]));
    let pieces: Vec<_> = c.document().entities.values().filter(|e| ![ids[1], ids[3]].contains(&e.id)).collect();
    assert_eq!(pieces.len(), 4);
    assert_eq!(pieces.iter().filter(|e| matches!(e.geometry, Geometry::Curve(_))).count(), 2);
    let area: f64 = pieces.iter().filter_map(|e| match &e.geometry {
        Geometry::Brep(brep) => Some(creation::measure(brep, &limits).unwrap().area_mm2), _ => None,
    }).sum();
    assert!((area - 200.).abs() < 1e-6);
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
}

#[test]
fn split_cutters_shrink_mixed_surface_targets_in_one_transaction() {
    use om9_app::command_options::CommandValue;
    let limits = Limits::default();
    for select_after_command in [false, true] {
    let typed = om9_core::surface_kernel::plane([0., 0., 0.], [1., 0., 0.], [0., 1., 0.], 10., 20., &limits).unwrap();
    let profile = creation::Profile::new(vec![[0., 0., 0.], [10., 0., 0.], [10., 20., 0.], [0., 20., 0.]]).unwrap();
    let face = creation::planar(&profile, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
    let a = Entity::new("typed", Geometry::Surface(typed), None);
    let b = Entity::new("face", Geometry::Brep(face), None);
    let curve = Entity::new("curve", Geometry::Curve(Curve::Line { start: [0., 10., 0.], end: [10., 10., 0.], domain: [0., 1.] }), None);
    let cutter = Entity::new("cutter", Geometry::Curve(Curve::Line { start: [5., -1., 0.], end: [5., 21., 0.], domain: [0., 1.] }), None);
    let ids = [a.id, b.id, curve.id, cutter.id];
    let mut doc = Document::with_template_tolerance("split cutter shrink mixed");
    for e in [a,b,curve,cutter] { doc.entities.insert(e.id,e); }
    let mut c = Controller::new(doc, limits).unwrap(); let before = c.document().clone();
    if select_after_command {
        c.submit("Split").unwrap();
        for id in &ids[..3] { c.top11_accept_object(*id).unwrap(); }
        c.submit("").unwrap();
    } else {
        c.select_objects(&ids[..3]).unwrap(); c.submit("Split").unwrap();
    }
    let option = c.command_options().into_iter().find(|o| o.id.name() == "Shrink").unwrap();
    assert_eq!(option.value, CommandValue::Flag(false));
    c.activate_command_option(option.id).unwrap();
    assert!(c.submit("Shrink=Maybe").is_err());
    c.top11_accept_object(ids[3]).unwrap();
    assert_eq!(c.command_options().into_iter().find(|o| o.id.name() == "Shrink").unwrap().value, CommandValue::Flag(true));
    c.top11_remove_object(ids[3]).unwrap();
    assert_eq!(c.command_options().into_iter().find(|o| o.id.name() == "Shrink").unwrap().value, CommandValue::Flag(true));
    c.top11_accept_object(ids[3]).unwrap();
    c.submit("").unwrap();
    assert_eq!(c.document().entities.len(), 7);
    assert_eq!(c.document().entities[&ids[3]], before.entities[&ids[3]]);
    let mut faces = 0; let mut curves = 0; let mut area = 0.;
    for e in c.document().entities.values().filter(|e| e.id != ids[3]) {
        match &e.geometry {
            Geometry::Curve(_) => curves += 1,
            Geometry::Brep(brep) => {
                faces += 1;
                let bounds = om9_kernel::measurement::face_underlying_bounds(brep, &limits).unwrap();
                assert!((bounds[1]-bounds[0]).abs() < 25. && (bounds[3]-bounds[2]).abs() < 25., "{bounds:?}");
                area += creation::measure(brep, &limits).unwrap().area_mm2;
            }
            _ => panic!("unexpected result"),
        }
    }
    assert_eq!((faces,curves),(4,2)); assert!((area-400.).abs() < 1e-6);
    let after = c.document().clone();
    let path = std::env::temp_dir().join(format!("om9-cutter-shrink-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, &after, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    let loaded = om9_archive::Archive::open(&path, &limits).unwrap(); std::fs::remove_file(path).unwrap(); assert_eq!(loaded, after);
    c.undo().unwrap(); assert_eq!(c.document(), &before); c.redo().unwrap(); assert_eq!(c.document(), &after);
    }
}

#[test]
fn split_cutter_shrink_fails_atomically_after_preparing_curve() {
    let limits = Limits::default();
    let profile = creation::Profile::new(vec![[0., 0., 0.], [10., 0., 0.], [10., 20., 0.], [0., 20., 0.]]).unwrap();
    let face = creation::planar(&profile, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
    let curve = Geometry::Curve(Curve::Line { start: [5., 10., -5.], end: [5., 10., 5.], domain: [0., 1.] });
    let mut a = Entity::new("first", curve.clone(), None);
    let mut b = Entity::new("second", curve.clone(), None);
    if a.id < b.id { b.geometry = Geometry::Brep(face.clone()); } else { a.geometry = Geometry::Brep(face.clone()); }
    let cutter = Entity::new("overlap", Geometry::Brep(face), None);
    let ids = [a.id,b.id,cutter.id];
    let mut doc = Document::with_template_tolerance("split shrink atomic failure");
    for entity in [a,b,cutter] { doc.entities.insert(entity.id, entity); }
    let mut c = Controller::new(doc, limits).unwrap(); let before = c.document().clone();
    c.select_objects(&ids[..2]).unwrap(); c.submit("Split").unwrap(); c.submit("Shrink=Yes").unwrap();
    c.top11_accept_object(ids[2]).unwrap(); assert!(c.submit("").is_err());
    assert_eq!(c.document(), &before); assert_eq!(c.coordinator().undo_len(), 0); assert!(c.is_running());
    c.cancel(); assert_eq!(c.document(), &before);
}

#[test]
fn split_shrink_option_carries_into_isocurve_mode() {
    use om9_app::command_options::CommandValue;
    let limits = Limits::default();
    let surface = om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,20.,&limits).unwrap();
    let target = Entity::new("surface", Geometry::Surface(surface), None); let id = target.id;
    let mut doc = Document::with_template_tolerance("shrink option transition"); doc.entities.insert(id,target);
    let mut c = Controller::new(doc, limits).unwrap();
    c.select_objects(&[id]).unwrap(); c.submit("Split").unwrap(); c.submit("Shrink=Yes").unwrap(); c.submit("Isocurve").unwrap();
    assert_eq!(c.command_options().into_iter().find(|o| o.id.name() == "Shrink").unwrap().value, CommandValue::Flag(true));
    c.accept_world_point([2.,6.,0.]).unwrap();
    for entity in c.document().entities.values() {
        let Geometry::Surface(surface) = &entity.geometry else { panic!("surface") };
        for axis in 0..2 {
            assert!((surface.knots[axis][0]-surface.domain[axis][0]).abs()<1e-8);
            assert!((*surface.knots[axis].last().unwrap()-surface.domain[axis][1]).abs()<1e-8);
        }
    }
}

#[test]
fn split_missed_cutter_does_not_mutate() {
    let limits = Limits::default();
    for solid in [false, true] {
        let profile = creation::Profile::new(vec![[0., 0., 0.], [10., 0., 0.], [10., 20., 0.], [0., 20., 0.]]).unwrap();
        let brep = if solid { box_brep() } else { creation::planar(&profile, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap() };
        let target = Entity::new("target", Geometry::Brep(brep), None);
        let cutter = Entity::new("cutter", Geometry::Curve(Curve::Line { start: [30., -1., 0.], end: [30., 21., 0.], domain: [0., 1.] }), None);
        let ids = [target.id, cutter.id];
        let mut doc = Document::with_template_tolerance("split errors");
        for entity in [target, cutter] { doc.entities.insert(entity.id, entity); }
        let mut c = Controller::new(doc, limits.clone()).unwrap();
        c.select_objects(&[ids[0]]).unwrap();
        let before = c.document().clone();
        c.submit("Split").unwrap();
        c.top11_accept_object(ids[1]).unwrap();
        assert!(c.submit("").is_err());
        assert!(c.is_running());
        assert_eq!(c.document(), &before);
        assert_eq!(c.coordinator().undo_len(), 0);
        c.cancel();
        assert_eq!(c.document(), &before);
    }
}

#[test]
fn split_joined_faces_preserves_existing_seam_and_undo() {
    let limits = Limits::default();
    let face = |points| creation::planar(&creation::Profile::new(points).unwrap(), creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
    let left = face(vec![[0., 0., 0.], [5., 0., 0.], [5., 20., 0.], [0., 20., 0.]]);
    let right = face(vec![[5., 0., 0.], [10., 0., 0.], [10., 20., 0.], [5., 20., 0.]]);
    let joined = om9_kernel::editing::sew(&left, &right, 0.001, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
    let target = Entity::new("joined", Geometry::Brep(joined), None);
    let cutter = Entity::new("cutter", Geometry::Curve(Curve::Line { start: [-1., 10., 0.], end: [11., 10., 0.], domain: [0., 1.] }), None);
    let limited = Limits { max_outputs: 1, ..limits };
    assert!(om9_kernel::measurement::split_boundary_by_geometry(
        &target.geometry, target.placement, &[(&cutter.geometry, cutter.placement)],
        creation::DEFAULT_DEFLECTION_MM, &limited,
    ).is_err());
    let ids = [target.id, cutter.id];
    let mut doc = Document::with_template_tolerance("split joined faces");
    for entity in [target, cutter] { doc.entities.insert(entity.id, entity); }
    let mut c = Controller::new(doc, limits.clone()).unwrap();
    let before = c.document().clone();
    c.select_objects(&[ids[0]]).unwrap();
    c.submit("Split").unwrap();
    c.top11_accept_object(ids[1]).unwrap();
    c.submit("").unwrap();
    let pieces: Vec<_> = c.document().entities.values().filter(|e| e.id != ids[1]).collect();
    assert_eq!(pieces.len(), 2);
    for piece in pieces {
        let Geometry::Brep(brep) = &piece.geometry else { panic!("expected BRep") };
        let m = creation::measure(brep, &limits).unwrap();
        assert_eq!(m.faces, 2);
        assert!((m.area_mm2 - 100.).abs() < 1e-6);
    }
    let after = c.document().clone();
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
    c.redo().unwrap();
    assert_eq!(c.document(), &after);
}

#[test]
fn split_joined_faces_with_shrink_preserves_shared_topology() {
    let limits = Limits::default();
    let face = |points| creation::planar(&creation::Profile::new(points).unwrap(), creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
    let left = face(vec![[0., 0., 0.], [5., 0., 0.], [5., 20., 0.], [0., 20., 0.]]);
    let right = face(vec![[5., 0., 0.], [10., 0., 0.], [10., 20., 0.], [5., 20., 0.]]);
    let joined = om9_kernel::editing::sew(&left, &right, 0.001, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
    let target = Entity::new("joined", Geometry::Brep(joined), None);
    let cutter = Entity::new("cutter", Geometry::Curve(Curve::Line { start: [-1., 10., 0.], end: [11., 10., 0.], domain: [0., 1.] }), None);
    let ids = [target.id, cutter.id];
    let mut doc = Document::with_template_tolerance("split shrink joined");
    for entity in [target,cutter] { doc.entities.insert(entity.id, entity); }
    let mut c = Controller::new(doc, limits).unwrap(); let before = c.document().clone();
    c.select_objects(&[ids[0]]).unwrap(); c.submit("Split").unwrap();
    assert!(c.command_options().iter().any(|o| o.id.name() == "Shrink"));
    c.submit("Shrink=Yes").unwrap(); c.top11_accept_object(ids[1]).unwrap(); c.submit("").unwrap();
    let pieces: Vec<_> = c.document().entities.values().filter(|e| e.id != ids[1]).collect();
    assert_eq!(pieces.len(), 2);
    for piece in pieces {
        let Geometry::Brep(brep) = &piece.geometry else { panic!("BRep") };
        let m = creation::measure(brep, &limits).unwrap();
        assert_eq!((m.faces, m.shells), (2,1));
        assert!((m.area_mm2-100.).abs() < 1e-6);
        assert_eq!(m.naked_edges, 6);
        for index in 0..2 {
            let face = om9_kernel::editing::extract_face(brep, index, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
            let bounds = om9_kernel::measurement::face_underlying_bounds(&face, &limits).unwrap();
            assert!((bounds[1]-bounds[0]) <= 10.0001 && (bounds[3]-bounds[2]) <= 10.0001, "{bounds:?}");
        }
    }
    let after = c.document().clone(); c.undo().unwrap(); assert_eq!(c.document(), &before);
    c.redo().unwrap(); assert_eq!(c.document(), &after);
}

#[test]
fn split_shrink_nonplanar_shell_with_surface_cutter() {
    let limits = Limits::default();
    let face = |points| creation::planar(&creation::Profile::new(points).unwrap(), creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
    let bottom = face(vec![[0., 0., 0.], [5., 0., 0.], [5., 20., 0.], [0., 20., 0.]]);
    let wall = face(vec![[5., 0., 0.], [5., 0., 5.], [5., 20., 5.], [5., 20., 0.]]);
    let joined = om9_kernel::editing::sew(&bottom, &wall, 0.001, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
    let cutter = face(vec![[-1., 10., -1.], [6., 10., -1.], [6., 10., 6.], [-1., 10., 6.]]);
    let target = Entity::new("fold", Geometry::Brep(joined), None);
    let cutting = Entity::new("plane", Geometry::Brep(cutter), None);
    let ids = [target.id, cutting.id]; let mut doc = Document::with_template_tolerance("nonplanar shell shrink");
    for entity in [target,cutting] { doc.entities.insert(entity.id,entity); }
    let mut c = Controller::new(doc, limits).unwrap(); let before = c.document().clone();
    c.select_objects(&[ids[0]]).unwrap(); c.submit("Split").unwrap(); c.submit("Shrink=Yes").unwrap();
    c.top11_accept_object(ids[1]).unwrap(); c.submit("").unwrap();
    let pieces: Vec<_> = c.document().entities.values().filter(|e| e.id != ids[1]).collect();
    assert_eq!(pieces.len(),2);
    for entity in pieces {
        let Geometry::Brep(brep) = &entity.geometry else { panic!("BRep") };
        let m = creation::measure(brep,&limits).unwrap();
        assert_eq!((m.faces,m.shells,m.solids,m.naked_edges),(2,1,0,6));
        assert!((m.area_mm2-100.).abs()<1e-6);
        for index in 0..2 {
            let sub = om9_kernel::editing::extract_face(brep,index,creation::DEFAULT_DEFLECTION_MM,&limits).unwrap();
            let bounds = om9_kernel::measurement::face_underlying_bounds(&sub,&limits).unwrap();
            assert!((bounds[1]-bounds[0]) < 11. && (bounds[3]-bounds[2]) < 11., "{bounds:?}");
        }
    }
    c.undo().unwrap(); assert_eq!(c.document(),&before);
}

#[test]
fn split_joined_faces_along_existing_seam_separates_the_faces() {
    let limits = Limits::default();
    for surface_cutter in [false, true] {
    for shrink in [false, true] {
    let face = |points| creation::planar(&creation::Profile::new(points).unwrap(), creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
    let left = face(vec![[0., 0., 0.], [5., 0., 0.], [5., 20., 0.], [0., 20., 0.]]);
    let right = face(vec![[5., 0., 0.], [10., 0., 0.], [10., 20., 0.], [5., 20., 0.]]);
    let joined = om9_kernel::editing::sew(&left, &right, 0.001, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
    let target = Entity::new("joined", Geometry::Brep(joined), None);
    let geometry = if surface_cutter {
        Geometry::Brep(face(vec![[5., -1., -2.], [5., 21., -2.], [5., 21., 2.], [5., -1., 2.]]))
    } else {
        Geometry::Curve(Curve::Line { start: [5., -1., 0.], end: [5., 21., 0.], domain: [0., 1.] })
    };
    let cutter = Entity::new("seam cutter", geometry, None);
    let ids = [target.id, cutter.id];
    let mut doc = Document::with_template_tolerance("split existing seam");
    for entity in [target, cutter] { doc.entities.insert(entity.id, entity); }
    let mut c = Controller::new(doc, limits.clone()).unwrap();
    let before = c.document().clone();
    c.select_objects(&[ids[0]]).unwrap(); c.submit("Split").unwrap();
    if shrink { c.submit("Shrink=Yes").unwrap(); }
    c.top11_accept_object(ids[1]).unwrap(); c.submit("").unwrap();
    let pieces: Vec<_> = c.document().entities.values().filter(|e| e.id != ids[1]).collect();
    assert_eq!(pieces.len(), 2);
    for piece in pieces {
        let Geometry::Brep(brep) = &piece.geometry else { panic!("expected BRep") };
        let m = creation::measure(brep, &limits).unwrap();
        assert_eq!(m.faces, 1);
        assert!((m.area_mm2 - 100.).abs() < 1e-6);
    }
    c.undo().unwrap(); assert_eq!(c.document(), &before);
    }
    }
}

#[test]
fn split_rejects_overlapping_surface_cutter_atomically() {
    let limits = Limits::default();
    let face = |points| creation::planar(&creation::Profile::new(points).unwrap(), creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
    let left = face(vec![[0., 0., 0.], [5., 0., 0.], [5., 20., 0.], [0., 20., 0.]]);
    let right = face(vec![[5., 0., 0.], [10., 0., 0.], [10., 20., 0.], [5., 20., 0.]]);
    let joined = om9_kernel::editing::sew(&left, &right, 0.001, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
    let target = Entity::new("joined", Geometry::Brep(joined), None);
    let cutter = Entity::new("overlap", Geometry::Brep(left), None);
    let ids = [target.id, cutter.id];
    let mut doc = Document::with_template_tolerance("overlap");
    for entity in [target, cutter] { doc.entities.insert(entity.id, entity); }
    let mut c = Controller::new(doc, limits).unwrap();
    let before = c.document().clone();
    c.select_objects(&[ids[0]]).unwrap(); c.submit("Split").unwrap();
    c.top11_accept_object(ids[1]).unwrap();
    assert!(c.submit("").is_err());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), 0);
    assert!(c.is_running());
}

#[test]
fn split_partial_seam_does_not_disconnect_a_joined_surface() {
    let limits = Limits::default();
    let face = |points| creation::planar(&creation::Profile::new(points).unwrap(), creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
    let left = face(vec![[0., 0., 0.], [5., 0., 0.], [5., 20., 0.], [0., 20., 0.]]);
    let right = face(vec![[5., 0., 0.], [10., 0., 0.], [10., 20., 0.], [5., 20., 0.]]);
    let joined = om9_kernel::editing::sew(&left, &right, 0.001, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
    let target = Entity::new("joined", Geometry::Brep(joined), None);
    let cutter = Entity::new("partial seam", Geometry::Curve(Curve::Line { start: [5., 0., 0.], end: [5., 10., 0.], domain: [0., 1.] }), None);
    let ids = [target.id, cutter.id];
    let mut doc = Document::with_template_tolerance("partial seam");
    for entity in [target, cutter] { doc.entities.insert(entity.id, entity); }
    let mut c = Controller::new(doc, limits).unwrap();
    let before = c.document().clone();
    c.select_objects(&[ids[0]]).unwrap(); c.submit("Split").unwrap();
    c.top11_accept_object(ids[1]).unwrap(); assert!(c.submit("").is_err());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), 0);
}

#[test]
fn split_seam_and_crossing_cutter_make_four_regions_in_one_undo() {
    let limits = Limits::default();
    let face = |points| creation::planar(&creation::Profile::new(points).unwrap(), creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
    let left = face(vec![[0., 0., 0.], [5., 0., 0.], [5., 20., 0.], [0., 20., 0.]]);
    let right = face(vec![[5., 0., 0.], [10., 0., 0.], [10., 20., 0.], [5., 20., 0.]]);
    let joined = om9_kernel::editing::sew(&left, &right, 0.001, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
    let target = Entity::new("joined", Geometry::Brep(joined), None);
    let seam = Entity::new("seam", Geometry::Curve(Curve::Line { start: [5., -1., 0.], end: [5., 21., 0.], domain: [0., 1.] }), None);
    let cross = Entity::new("cross", Geometry::Curve(Curve::Line { start: [-1., 10., 0.], end: [11., 10., 0.], domain: [0., 1.] }), None);
    let ids = [target.id, seam.id, cross.id];
    let mut doc = Document::with_template_tolerance("seam and cross");
    for entity in [target, seam, cross] { doc.entities.insert(entity.id, entity); }
    let mut c = Controller::new(doc, limits.clone()).unwrap();
    let before = c.document().clone();
    c.select_objects(&[ids[0]]).unwrap(); c.submit("Split").unwrap();
    c.top11_accept_object(ids[1]).unwrap(); c.top11_accept_object(ids[2]).unwrap(); c.submit("").unwrap();
    let pieces: Vec<_> = c.document().entities.values().filter(|e| !ids[1..].contains(&e.id)).collect();
    assert_eq!(pieces.len(), 4);
    for piece in pieces {
        let Geometry::Brep(brep) = &piece.geometry else { panic!("expected BRep") };
        let m = creation::measure(brep, &limits).unwrap();
        assert_eq!(m.faces, 1);
        assert!((m.area_mm2 - 50.).abs() < 1e-6);
    }
    c.undo().unwrap(); assert_eq!(c.document(), &before);
}

#[test]
fn split_closed_polysurface_by_plane_keeps_connected_open_regions() {
    let limits = Limits::default();
    let profile = creation::Profile::new(vec![[-1., -1., 2.], [3., -1., 2.], [3., 4., 2.], [-1., 4., 2.]]).unwrap();
    let mut target = Entity::new("box", Geometry::Brep(box_brep()), None);
    let mut cutter = Entity::new("plane", Geometry::Brep(creation::planar(&profile, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap()), None);
    target.placement = Transform::translation([10., -5., 3.]);
    cutter.placement = target.placement;
    let ids = [target.id, cutter.id];
    let mut doc = Document::with_template_tolerance("split box boundary");
    for entity in [target, cutter] { doc.entities.insert(entity.id, entity); }
    let mut c = Controller::new(doc, limits.clone()).unwrap();
    let before = c.document().clone();
    c.select_objects(&[ids[0]]).unwrap();
    c.submit("Split").unwrap();
    c.top11_accept_object(ids[1]).unwrap();
    c.submit("").unwrap();
    let pieces: Vec<_> = c.document().entities.values().filter(|e| e.id != ids[1]).collect();
    assert_eq!(pieces.len(), 2);
    for piece in pieces {
        assert_eq!(piece.placement, Transform::translation([10., -5., 3.]));
        let Geometry::Brep(brep) = &piece.geometry else { panic!("expected BRep") };
        let m = creation::measure(brep, &limits).unwrap();
        assert_eq!(m.faces, 5);
        assert_eq!(m.solids, 0);
        assert!((m.area_mm2 - 26.).abs() < 1e-6, "{}", m.area_mm2);
    }
    let after = c.document().clone();
    let path = std::env::temp_dir().join(format!("om9-split-polysurface-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, &after, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    let loaded = om9_archive::Archive::open(&path, &limits).unwrap();
    std::fs::remove_file(path).unwrap();
    assert_eq!(loaded, after);
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
    c.redo().unwrap();
    assert_eq!(c.document(), &after);
}

#[test]
fn split_box_boundary_with_shrink_keeps_connected_regions() {
    let limits = Limits::default();
    let profile = creation::Profile::new(vec![[-1., -1., 2.], [3., -1., 2.], [3., 4., 2.], [-1., 4., 2.]]).unwrap();
    let target = Entity::new("box", Geometry::Brep(box_brep()), None);
    let cutter = Entity::new("plane", Geometry::Brep(creation::planar(&profile, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap()), None);
    let ids = [target.id,cutter.id]; let mut doc = Document::with_template_tolerance("box shrink");
    for entity in [target,cutter] { doc.entities.insert(entity.id,entity); }
    let mut c = Controller::new(doc, limits).unwrap(); let before = c.document().clone();
    c.select_objects(&[ids[0]]).unwrap(); c.submit("Split").unwrap(); c.submit("Shrink=Yes").unwrap();
    c.top11_accept_object(ids[1]).unwrap(); c.submit("").unwrap();
    let pieces: Vec<_> = c.document().entities.values().filter(|e| e.id != ids[1]).collect();
    assert_eq!(pieces.len(),2);
    for entity in pieces {
        let Geometry::Brep(brep) = &entity.geometry else { panic!("BRep") };
        let m = creation::measure(brep, &limits).unwrap();
        assert_eq!((m.faces,m.shells,m.solids),(5,1,0));
        assert_eq!(m.naked_edges,4);
        assert!((m.area_mm2-26.).abs()<1e-6);
        for index in 0..5 {
            let face = om9_kernel::editing::extract_face(brep,index,creation::DEFAULT_DEFLECTION_MM,&limits).unwrap();
            let bounds = om9_kernel::measurement::face_underlying_bounds(&face,&limits).unwrap();
            assert!((bounds[1]-bounds[0]) < 5. && (bounds[3]-bounds[2]) < 5., "{bounds:?}");
        }
    }
    let after = c.document().clone();
    let path = std::env::temp_dir().join(format!("om9-box-shrink-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, &after, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    let loaded = om9_archive::Archive::open(&path,&limits).unwrap(); std::fs::remove_file(path).unwrap(); assert_eq!(loaded, after);
    c.undo().unwrap(); assert_eq!(c.document(),&before); c.redo().unwrap(); assert_eq!(c.document(),&after);
}

#[test]
fn split_sphere_boundary_by_plane_produces_two_hemispheres() {
    let limits = Limits::default();
    let sphere = creation::primitive(Primitive::Sphere { radius_mm: 5. }, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
    let profile = creation::Profile::new(vec![[-6., -6., 0.], [6., -6., 0.], [6., 6., 0.], [-6., 6., 0.]]).unwrap();
    let target = Entity::new("sphere", Geometry::Brep(sphere), None);
    let cutter = Entity::new("plane", Geometry::Brep(creation::planar(&profile, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap()), None);
    let ids = [target.id, cutter.id];
    let mut doc = Document::with_template_tolerance("split sphere boundary");
    for entity in [target, cutter] { doc.entities.insert(entity.id, entity); }
    let mut c = Controller::new(doc, limits.clone()).unwrap();
    let before = c.document().clone();
    c.select_objects(&[ids[0]]).unwrap();
    c.submit("Split").unwrap();
    c.top11_accept_object(ids[1]).unwrap();
    c.submit("").unwrap();
    let pieces: Vec<_> = c.document().entities.values().filter(|e| e.id != ids[1]).collect();
    assert_eq!(pieces.len(), 2);
    for piece in pieces {
        let Geometry::Brep(brep) = &piece.geometry else { panic!("expected BRep") };
        let m = creation::measure(brep, &limits).unwrap();
        assert_eq!(m.solids, 0);
        assert!((m.area_mm2 - 50. * std::f64::consts::PI).abs() < 1e-5);
    }
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
}

#[test]
fn split_surface_with_brep_cutter_produces_two_faces() {
    let limits = Limits::default();
    let profile = creation::Profile::new(vec![[0., 0., 0.], [10., 0., 0.], [10., 20., 0.], [0., 20., 0.]]).unwrap();
    let cutter_profile = creation::Profile::new(vec![[5., -1., -2.], [5., 21., -2.], [5., 21., 2.], [5., -1., 2.]]).unwrap();
    let target = Entity::new("target", Geometry::Brep(creation::planar(&profile, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap()), None);
    let cutter = Entity::new("cutter", Geometry::Brep(creation::planar(&cutter_profile, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap()), None);
    let ids = [target.id, cutter.id];
    let mut doc = Document::with_template_tolerance("split surface with surface");
    for entity in [target, cutter] { doc.entities.insert(entity.id, entity); }
    let mut c = Controller::new(doc, limits.clone()).unwrap();
    let before = c.document().clone();
    c.select_objects(&[ids[0]]).unwrap();
    c.submit("Split").unwrap();
    c.top11_accept_object(ids[1]).unwrap();
    c.submit("").unwrap();
    let areas: Vec<_> = c.document().entities.values().filter(|e| e.id != ids[1]).map(|e| {
        let Geometry::Brep(brep) = &e.geometry else { panic!("expected face") };
        creation::measure(brep, &limits).unwrap().area_mm2
    }).collect();
    assert_eq!(areas.len(), 2);
    assert!(areas.iter().all(|area| (area - 100.).abs() < 1e-6));
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
}

#[test]
fn trim_line_and_placed_face_through_command_and_toolbar_with_undo() {
    use om9_core::top11::Top11Action;
    let limits = Limits::default();
    let profile = creation::Profile::new(vec![[0., 0., 0.], [10., 0., 0.], [10., 20., 0.], [0., 20., 0.]]).unwrap();
    for toolbar in [false, true] {
        let face = creation::planar(&profile, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
        let mut doc = Document::with_template_tolerance("trim placed face");
        let mut target = Entity::new("face", Geometry::Brep(face), None);
        target.placement = Transform::translation([20., 4., 2.]);
        let mut cutter = Entity::new("cutter", Geometry::Curve(Curve::Line {
            start: [0., -5., 0.], end: [0., 25., 0.], domain: [0., 1.],
        }), None);
        cutter.placement = Transform::translation([23., 4., 2.]);
        let (target_id, cutter_id) = (target.id, cutter.id);
        doc.entities.insert(target_id, target);
        doc.entities.insert(cutter_id, cutter);
        let mut c = Controller::new(doc, limits.clone()).unwrap();
        let before = c.document().clone();
        if toolbar {
            c.select_objects(&[target_id, cutter_id]).unwrap();
            c.top11_invoke(Top11Action::Trim).unwrap();
        } else {
            c.submit("Trim").unwrap();
            c.top11_accept_object(target_id).unwrap();
            c.top11_accept_object(cutter_id).unwrap();
            c.submit("").unwrap();
        }
        assert!(c.top11_trim_many_has_target(target_id));
        assert!(c.top11_trim_many_accept_pick(target_id, [23., 14., 2.]).is_err());
        assert_eq!(c.document(), &before);
        assert_eq!(c.coordinator().undo_len(), 0);
        c.top11_trim_many_accept_pick(target_id, [21., 14., 2.]).unwrap();
        let result = &c.document().entities[&target_id];
        assert_eq!(result.placement, Transform::translation([20., 4., 2.]));
        let Geometry::Brep(brep) = &result.geometry else { panic!("expected BRep") };
        assert!((creation::measure(brep, &limits).unwrap().area_mm2 - 140.).abs() < 1e-6);
        assert_eq!(c.document().entities[&cutter_id], before.entities[&cutter_id]);
        assert!(c.top11_trim_many_has_target(target_id));
        c.submit("").unwrap();
        c.undo().unwrap();
        assert_eq!(c.document(), &before);
        c.redo().unwrap();
        let Geometry::Brep(brep) = &c.document().entities[&target_id].geometry else { panic!("expected BRep") };
        assert!((creation::measure(brep, &limits).unwrap().area_mm2 - 140.).abs() < 1e-6);
    }
}

#[test]
fn trim_face_missed_cutter_and_cancel_leave_document_untouched() {
    let limits = Limits::default();
    let profile = creation::Profile::new(vec![[0., 0., 0.], [10., 0., 0.], [10., 20., 0.], [0., 20., 0.]]).unwrap();
    for x in [3., 30.] {
        let face = creation::planar(&profile, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
        let mut doc = Document::with_template_tolerance("trim face cancel");
        let target = Entity::new("face", Geometry::Brep(face), None);
        let cutter = Entity::new("cutter", Geometry::Curve(Curve::Line {
            start: [x, -5., 0.], end: [x, 25., 0.], domain: [0., 1.],
        }), None);
        let ids = [target.id, cutter.id];
        doc.entities.insert(target.id, target);
        doc.entities.insert(cutter.id, cutter);
        let mut c = Controller::new(doc, limits.clone()).unwrap();
        c.select_objects(&ids).unwrap();
        let before = c.document().clone();
        if x == 3. {
            c.submit("Trim").unwrap();
            c.cancel();
        } else {
            assert!(c.submit("Trim").is_err());
        }
        assert_eq!(c.document(), &before);
        assert_eq!(c.coordinator().undo_len(), 0);
    }
}

#[test]
fn trim_surface_with_multiple_curves_removes_regions_repeatedly_with_undo() {
    use om9_core::geometry::NurbsSurface;
    let surface = NurbsSurface {
        degree: [1, 1], size: [2, 2],
        poles: vec![[0., 0., 0.], [0., 20., 0.], [10., 0., 0.], [10., 20., 0.]],
        weights: vec![1.; 4], knots: [vec![0., 0., 1., 1.], vec![0., 0., 1., 1.]],
        domain: [[0., 1.], [0., 1.]], periodic: [false, false], reversed: false,
    };
    let mut doc = Document::with_template_tolerance("trim multiple surface regions");
    let target = Entity::new("surface", Geometry::Surface(surface), None);
    let a = Entity::new("vertical", Geometry::Curve(Curve::Line {
        start: [5., -1., 0.], end: [5., 21., 0.], domain: [0., 1.],
    }), None);
    let b = Entity::new("horizontal", Geometry::Curve(Curve::Line {
        start: [-1., 10., 0.], end: [11., 10., 0.], domain: [0., 1.],
    }), None);
    let ids = [target.id, a.id, b.id];
    for entity in [target, a, b] { doc.entities.insert(entity.id, entity); }
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    let before = c.document().clone();
    c.select_objects(&ids).unwrap();
    c.submit("Trim").unwrap();
    assert!(c.top11_trim_many_accept_pick(ids[0], [5., 10., 0.]).is_err());
    assert_eq!(c.document(), &before);
    for (point, area, faces) in [([1., 1., 0.], 150., 1), ([9., 19., 0.], 100., 2), ([1., 19., 0.], 50., 1)] {
        c.top11_trim_many_accept_pick(ids[0], point).unwrap();
        let Geometry::Brep(brep) = &c.document().entities[&ids[0]].geometry else { panic!("expected BRep") };
        let measured = creation::measure(brep, &Limits::default()).unwrap();
        assert!((measured.area_mm2 - area).abs() < 1e-6);
        assert_eq!(measured.faces, faces);
    }
    c.top11_trim_many_accept_pick(ids[0], [9., 1., 0.]).unwrap();
    assert!(!c.document().entities.contains_key(&ids[0]));
    c.submit("").unwrap();
    for _ in 0..4 { c.undo().unwrap(); }
    assert_eq!(c.document(), &before);
    for _ in 0..4 { c.redo().unwrap(); }
    assert!(!c.document().entities.contains_key(&ids[0]));
    assert!(ids[1..].iter().all(|id| c.document().entities.contains_key(id)));
}

#[test]
fn trim_surface_and_curve_repartitions_in_both_edit_orders() {
    use om9_core::top11::Top11Action;
    use om9_core::geometry::NurbsSurface;
    let limits = Limits::default();
    let profile = creation::Profile::new(vec![[0., 0., 0.], [10., 0., 0.], [10., 20., 0.], [0., 20., 0.]]).unwrap();
    for (face_first, typed_surface) in [(false, false), (true, false), (false, true), (true, true)] {
        let geometry = if typed_surface {
            Geometry::Surface(NurbsSurface { degree: [1, 1], size: [2, 2],
                poles: vec![[0., 0., 0.], [0., 20., 0.], [10., 0., 0.], [10., 20., 0.]], weights: vec![1.; 4],
                knots: [vec![0., 0., 1., 1.], vec![0., 0., 1., 1.]], domain: [[0., 1.], [0., 1.]], periodic: [false, false], reversed: false })
        } else { Geometry::Brep(creation::planar(&profile, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap()) };
        let face = Entity::new("face", geometry, None);
        let vertical = Entity::new("vertical", Geometry::Curve(Curve::Line { start: [5., -1., 0.], end: [5., 21., 0.], domain: [0., 1.] }), None);
        let horizontal = Entity::new("horizontal", Geometry::Curve(Curve::Line { start: [-1., 10., 0.], end: [11., 10., 0.], domain: [0., 1.] }), None);
        let ids = [face.id, vertical.id, horizontal.id];
        let mut doc = Document::with_template_tolerance("mixed trim order");
        for entity in [face, vertical, horizontal] { doc.entities.insert(entity.id, entity); }
        let mut c = Controller::new(doc, limits.clone()).unwrap();
        let before = c.document().clone();
        c.select_objects(&ids).unwrap();
        c.top11_invoke(Top11Action::Trim).unwrap();
        assert!(c.top11_trim_many_has_target(ids[0]));
        assert!(c.top11_trim_many_has_target(ids[1]));
        assert!(c.accept_world_point([5., 5., 0.]).is_err());
        assert_eq!(c.document(), &before);
        assert_eq!(c.coordinator().undo_len(), 0);
        if face_first {
            c.top11_trim_many_accept_pick(ids[0], [1., 1., 0.]).unwrap();
            c.top11_trim_many_accept_pick(ids[1], [5., 15., 0.]).unwrap();
            c.top11_trim_many_accept_pick(ids[0], [1., 19., 0.]).unwrap();
        } else {
            c.top11_trim_many_accept_pick(ids[1], [5., 5., 0.]).unwrap();
            c.top11_trim_many_accept_pick(ids[0], [1., 1., 0.]).unwrap();
        }
        let Geometry::Brep(brep) = &c.document().entities[&ids[0]].geometry else { panic!("expected face") };
        let expected = if face_first { 50. } else { 100. };
        let area = creation::measure(brep, &limits).unwrap().area_mm2;
        assert!((area - expected).abs() < 1e-6, "face_first={face_first}, area={area}");
        c.submit("").unwrap();
        for _ in 0..if face_first { 3 } else { 2 } { c.undo().unwrap(); }
        assert_eq!(c.document(), &before);
        for _ in 0..if face_first { 3 } else { 2 } { c.redo().unwrap(); }
        let Geometry::Brep(brep) = &c.document().entities[&ids[0]].geometry else { panic!("expected face") };
        assert!((creation::measure(brep, &limits).unwrap().area_mm2 - expected).abs() < 1e-6);
    }
}

#[test]
fn trim_two_intersecting_surfaces_recomputes_after_each_removal() {
    let limits = Limits::default();
    let first = creation::Profile::new(vec![[-5., -5., 0.], [5., -5., 0.], [5., 5., 0.], [-5., 5., 0.]]).unwrap();
    let second = creation::Profile::new(vec![[-5., 0., -5.], [5., 0., -5.], [5., 0., 5.], [-5., 0., 5.]]).unwrap();
    for (pick_first, pick_other) in [(true, false), (false, false), (true, true), (false, true)] {
        let a = Entity::new("horizontal", Geometry::Brep(creation::planar(&first, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap()), None);
        let b = Entity::new("vertical", Geometry::Brep(creation::planar(&second, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap()), None);
        let ids = [a.id, b.id];
        let mut doc = Document::with_template_tolerance("mutual surfaces");
        for entity in [a, b] { doc.entities.insert(entity.id, entity); }
        let mut c = Controller::new(doc, limits.clone()).unwrap();
        let before = c.document().clone();
        c.select_objects(&ids).unwrap();
        c.submit("Trim").unwrap();
        assert!(c.top11_trim_many_has_target(ids[0]));
        assert!(c.top11_trim_many_has_target(ids[1]));
        assert!(c.accept_world_point([0., 0., 0.]).is_err());
        assert_eq!(c.document(), &before);
        let (id, first_pick, remaining_pick, other) = if pick_first {
            (ids[0], [0., -2., 0.], [0., 2., 0.], ids[1])
        } else {
            (ids[1], [0., 0., -2.], [0., 0., 2.], ids[0])
        };
        c.top11_trim_many_accept_pick(id, first_pick).unwrap();
        let Geometry::Brep(brep) = &c.document().entities[&id].geometry else { panic!("expected BRep") };
        assert!((creation::measure(brep, &limits).unwrap().area_mm2 - 50.).abs() < 1e-6);
        assert!(c.top11_trim_many_has_target(other));
        if pick_other {
            let other_pick = if pick_first { [0., 0., -2.] } else { [0., -2., 0.] };
            c.top11_trim_many_accept_pick(other, other_pick).unwrap();
            let Geometry::Brep(brep) = &c.document().entities[&other].geometry else { panic!("expected BRep") };
            assert!((creation::measure(brep, &limits).unwrap().area_mm2 - 50.).abs() < 1e-6);
        } else {
            c.top11_trim_many_accept_pick(id, remaining_pick).unwrap();
            assert!(!c.document().entities.contains_key(&id));
            assert!(!c.top11_trim_many_has_target(other));
        }
        c.submit("").unwrap();
        c.undo().unwrap(); c.undo().unwrap();
        assert_eq!(c.document(), &before);
        c.redo().unwrap(); c.redo().unwrap();
        if !pick_other { assert!(!c.document().entities.contains_key(&id)); }
    }
}

#[test]
fn trim_two_typed_surfaces_can_be_selected_after_command() {
    use om9_core::geometry::NurbsSurface;
    let surface = |poles| Geometry::Surface(NurbsSurface {
        degree: [1, 1], size: [2, 2], poles, weights: vec![1.; 4],
        knots: [vec![0., 0., 1., 1.], vec![0., 0., 1., 1.]],
        domain: [[0., 1.], [0., 1.]], periodic: [false, false], reversed: false,
    });
    let a = Entity::new("horizontal", surface(vec![[-5., -5., 0.], [-5., 5., 0.], [5., -5., 0.], [5., 5., 0.]]), None);
    let b = Entity::new("vertical", surface(vec![[-5., 0., -5.], [-5., 0., 5.], [5., 0., -5.], [5., 0., 5.]]), None);
    let ids = [a.id, b.id];
    let mut doc = Document::with_template_tolerance("mutual typed surfaces");
    for entity in [a, b] { doc.entities.insert(entity.id, entity); }
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    let before = c.document().clone();
    c.submit("Trim").unwrap();
    for id in ids { c.top11_accept_object(id).unwrap(); }
    c.submit("").unwrap();
    c.top11_trim_many_accept_pick(ids[0], [0., -2., 0.]).unwrap();
    c.top11_trim_many_accept_pick(ids[1], [0., 0., -2.]).unwrap();
    for id in ids {
        let Geometry::Brep(brep) = &c.document().entities[&id].geometry else { panic!("expected BRep") };
        assert!((creation::measure(brep, &Limits::default()).unwrap().area_mm2 - 50.).abs() < 1e-6);
    }
    c.submit("").unwrap();
    c.undo().unwrap(); c.undo().unwrap();
    assert_eq!(c.document(), &before);
}

#[test]
fn trim_three_surfaces_recomputes_other_targets() {
    let limits = Limits::default();
    let profiles = [
        vec![[-5., -5., 0.], [5., -5., 0.], [5., 5., 0.], [-5., 5., 0.]],
        vec![[-5., 0., -5.], [5., 0., -5.], [5., 0., 5.], [-5., 0., 5.]],
        vec![[0., -5., -5.], [0., 5., -5.], [0., 5., 5.], [0., -5., 5.]],
    ];
    let mut doc = Document::with_template_tolerance("three mutual surfaces");
    let mut ids = Vec::new();
    for (index, points) in profiles.into_iter().enumerate() {
        let profile = creation::Profile::new(points).unwrap();
        let face = creation::planar(&profile, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
        let entity = Entity::new(format!("face {index}"), Geometry::Brep(face), None);
        ids.push(entity.id); doc.entities.insert(entity.id, entity);
    }
    let mut c = Controller::new(doc, limits.clone()).unwrap();
    let before = c.document().clone();
    c.select_objects(&ids).unwrap(); c.submit("Trim").unwrap();
    c.top11_trim_many_accept_pick(ids[0], [2., 2., 0.]).unwrap();
    c.top11_trim_many_accept_pick(ids[1], [-2., 0., -2.]).unwrap();
    for id in &ids[..2] {
        let Geometry::Brep(face) = &c.document().entities[id].geometry else { panic!("face") };
        assert!((creation::measure(face, &limits).unwrap().area_mm2 - 75.).abs() < 1e-6);
    }
    c.submit("").unwrap(); c.undo().unwrap(); c.undo().unwrap();
    assert_eq!(c.document(), &before);
}

#[test]
fn trim_multiple_surfaces_and_curve_in_both_orders() {
    use om9_core::geometry::NurbsSurface;
    let limits = Limits::default();
    let horizontal = creation::Profile::new(vec![[-5., -5., 0.], [5., -5., 0.], [5., 5., 0.], [-5., 5., 0.]]).unwrap();
    let vertical = creation::Profile::new(vec![[-5., 0., -5.], [5., 0., -5.], [5., 0., 5.], [-5., 0., 5.]]).unwrap();
    for (face_first, typed, placed) in [(false, false, false), (true, false, false), (false, true, false), (true, true, false), (true, false, true), (false, true, true)] {
        let surface = |poles| Geometry::Surface(NurbsSurface {
            degree: [1, 1], size: [2, 2], poles, weights: vec![1.; 4],
            knots: [vec![0., 0., 1., 1.], vec![0., 0., 1., 1.]], domain: [[0., 1.], [0., 1.]], periodic: [false, false], reversed: false,
        });
        let ag = if typed { surface(vec![[-5., -5., 0.], [-5., 5., 0.], [5., -5., 0.], [5., 5., 0.]]) }
            else { Geometry::Brep(creation::planar(&horizontal, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap()) };
        let bg = if typed { surface(vec![[-5., 0., -5.], [-5., 0., 5.], [5., 0., -5.], [5., 0., 5.]]) }
            else { Geometry::Brep(creation::planar(&vertical, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap()) };
        let mut a = Entity::new("horizontal", ag, None);
        let mut b = Entity::new("vertical", bg, None);
        let mut curve = Entity::new("curve", Geometry::Curve(Curve::Line { start: [0., -6., 0.], end: [0., 6., 0.], domain: [0., 1.] }), None);
        let placement = if placed { Transform::translation([10., -3., 4.]) } else { Transform::identity() };
        a.placement = placement; b.placement = placement; curve.placement = placement;
        let world = |point| placement.point(point).unwrap();
        let ids = [a.id, b.id, curve.id];
        let mut doc = Document::with_template_tolerance("mixed multiple surfaces");
        for entity in [a, b, curve] { doc.entities.insert(entity.id, entity); }
        let mut c = Controller::new(doc, limits.clone()).unwrap();
        let before = c.document().clone();
        c.select_objects(&ids).unwrap(); c.submit("Trim").unwrap();
        assert!(ids.iter().all(|id| c.top11_trim_many_has_target(*id)));
        assert!(c.accept_world_point(world([0., 0., 0.])).is_err());
        assert_eq!(c.document(), &before);
        assert_eq!(c.coordinator().undo_len(), 0);
        if face_first {
            c.top11_trim_many_accept_pick(ids[0], world([-2., -2., 0.])).unwrap();
            c.top11_trim_many_accept_pick(ids[2], world([0., 2., 0.])).unwrap();
            c.top11_trim_many_accept_pick(ids[0], world([-2., 2., 0.])).unwrap();
        } else {
            c.top11_trim_many_accept_pick(ids[2], world([0., -2., 0.])).unwrap();
            c.top11_trim_many_accept_pick(ids[0], world([-2., -2., 0.])).unwrap();
            c.top11_trim_many_accept_pick(ids[1], world([2., 0., -2.])).unwrap();
        }
        let Geometry::Brep(face) = &c.document().entities[&ids[0]].geometry else { panic!("face") };
        let area = creation::measure(face, &limits).unwrap().area_mm2;
        assert!((area - if face_first { 25. } else { 50. }).abs() < 1e-6, "area={area}");
        c.submit("").unwrap();
        for _ in 0..3 { c.undo().unwrap(); }
        assert_eq!(c.document(), &before);
        for _ in 0..3 { c.redo().unwrap(); }
        let Geometry::Brep(face) = &c.document().entities[&ids[0]].geometry else { panic!("face") };
        assert!((creation::measure(face, &limits).unwrap().area_mm2 - area).abs() < 1e-6);
    }
}

#[test]
fn trim_face_and_surface_sessions_report_no_change_on_empty_finish() {
    use om9_app::session::DispatchOutcome;
    let limits=Limits::default();
    let face=|points| creation::planar(&creation::Profile::new(points).unwrap(),creation::DEFAULT_DEFLECTION_MM,&limits).unwrap();
    let a=Entity::new("base",Geometry::Brep(face(vec![[0.,0.,0.],[10.,0.,0.],[10.,10.,0.],[0.,10.,0.]])),None);
    let line=Entity::new("line",Geometry::Curve(Curve::Line{start:[5.,-1.,0.],end:[5.,11.,0.],domain:[0.,1.]}),None);
    let crossing=Entity::new("crossing",Geometry::Brep(face(vec![[0.,5.,-5.],[10.,5.,-5.],[10.,5.,5.],[0.,5.,5.]])),None);
    let ids=[a.id,line.id,crossing.id];let mut doc=Document::with_template_tolerance("trim no change");
    for e in [a,line,crossing] {doc.entities.insert(e.id,e);}
    for selection in [&ids[..2],&[ids[0],ids[2]][..]] {
        let mut c=Controller::new(doc.clone(),limits.clone()).unwrap();c.select_objects(selection).unwrap();let before=c.document().clone();let undo=c.coordinator().undo_len();
        c.submit("Trim").unwrap();let DispatchOutcome::Top11(result)=c.submit_effect("").unwrap() else {panic!("missing feedback")};
        assert_eq!(result.message(),"Trim: no changes");assert!(!c.is_running());assert_eq!(c.document(),&before);assert_eq!(c.coordinator().undo_len(),undo);
    }
}

#[test]
fn trim_multiface_brep_and_curve_keeps_original_face_boundary() {
    let limits = Limits::default();
    let left = creation::Profile::new(vec![[0., 0., 0.], [5., 0., 0.], [5., 20., 0.], [0., 20., 0.]]).unwrap();
    let right = creation::Profile::new(vec![[5., 0., 0.], [10., 0., 0.], [10., 20., 0.], [5., 20., 0.]]).unwrap();
    let left = creation::planar(&left, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
    let right = creation::planar(&right, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
    let original = om9_kernel::editing::sew(&left, &right, 1e-6, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
    let face = Entity::new("two faces", Geometry::Brep(original), None);
    let vertical = Entity::new("vertical", Geometry::Curve(Curve::Line { start: [2., -1., 0.], end: [2., 21., 0.], domain: [0., 1.] }), None);
    let horizontal = Entity::new("horizontal", Geometry::Curve(Curve::Line { start: [-1., 10., 0.], end: [11., 10., 0.], domain: [0., 1.] }), None);
    let ids = [face.id, vertical.id, horizontal.id];
    let mut doc = Document::with_template_tolerance("preserve authored face seam");
    for entity in [face, vertical, horizontal] { doc.entities.insert(entity.id, entity); }
    let mut c = Controller::new(doc, limits.clone()).unwrap();
    let before = c.document().clone();
    c.select_objects(&ids).unwrap(); c.submit("Trim").unwrap();
    c.top11_trim_many_accept_pick(ids[0], [1., 1., 0.]).unwrap();
    c.top11_trim_many_accept_pick(ids[1], [2., 15., 0.]).unwrap();
    c.top11_trim_many_accept_pick(ids[0], [1., 15., 0.]).unwrap();
    let Geometry::Brep(result) = &c.document().entities[&ids[0]].geometry else { panic!("face") };
    let measure = creation::measure(result, &limits).unwrap();
    assert_eq!(measure.faces, 2);
    assert!((measure.area_mm2 - 130.).abs() < 1e-6);
    let mut areas: Vec<_> = (0..2).map(|i| {
        let face = om9_kernel::editing::extract_face(result, i, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
        creation::measure(&face, &limits).unwrap().area_mm2
    }).collect();
    areas.sort_by(f64::total_cmp);
    assert!((areas[0] - 30.).abs() < 1e-6 && (areas[1] - 100.).abs() < 1e-6, "{areas:?}");
    c.submit("").unwrap();
    for _ in 0..3 { c.undo().unwrap(); }
    assert_eq!(c.document(), &before);
    for _ in 0..3 { c.redo().unwrap(); }
    let Geometry::Brep(result) = &c.document().entities[&ids[0]].geometry else { panic!("face") };
    assert_eq!(creation::measure(result, &limits).unwrap().faces, 2);
}

#[test]
fn trim_multiface_brep_with_other_surface_and_curve_preserves_seam() {
    let limits = Limits::default();
    let face = |points| creation::planar(&creation::Profile::new(points).unwrap(), creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
    let left = face(vec![[0., 0., 0.], [5., 0., 0.], [5., 20., 0.], [0., 20., 0.]]);
    let right = face(vec![[5., 0., 0.], [10., 0., 0.], [10., 20., 0.], [5., 20., 0.]]);
    let multi = om9_kernel::editing::sew(&left, &right, 1e-6, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
    let crossing_left = face(vec![[0., 10., -5.], [5., 10., -5.], [5., 10., 5.], [0., 10., 5.]]);
    let crossing_right = face(vec![[5., 10., -5.], [10., 10., -5.], [10., 10., 5.], [5., 10., 5.]]);
    let crossing = om9_kernel::editing::sew(&crossing_left, &crossing_right, 1e-6, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
    let a = Entity::new("multi", Geometry::Brep(multi), None);
    let b = Entity::new("crossing", Geometry::Brep(crossing), None);
    let curve = Entity::new("curve", Geometry::Curve(Curve::Line { start: [2., -1., 0.], end: [2., 21., 0.], domain: [0., 1.] }), None);
    let ids = [a.id, b.id, curve.id];
    let mut doc = Document::with_template_tolerance("mixed multi-face Trim");
    for entity in [a, b, curve] { doc.entities.insert(entity.id, entity); }
    let mut c = Controller::new(doc, limits.clone()).unwrap();
    let before = c.document().clone();
    c.select_objects(&ids).unwrap(); c.submit("Trim").unwrap();
    assert!(ids.iter().all(|id| c.top11_trim_many_has_target(*id)));
    c.top11_trim_many_accept_pick(ids[1], [2., 10., -2.]).unwrap();
    let Geometry::Brep(result) = &c.document().entities[&ids[1]].geometry else { panic!("face") };
    let measure = creation::measure(result, &limits).unwrap();
    assert_eq!(measure.faces, 2);
    assert!((measure.area_mm2 - 75.).abs() < 1e-6);
    c.top11_trim_many_accept_pick(ids[0], [1., 1., 0.]).unwrap();
    c.top11_trim_many_accept_pick(ids[2], [2., 15., 0.]).unwrap();
    c.top11_trim_many_accept_pick(ids[0], [1., 15., 0.]).unwrap();
    let Geometry::Brep(result) = &c.document().entities[&ids[0]].geometry else { panic!("face") };
    let measure = creation::measure(result, &limits).unwrap();
    assert_eq!(measure.faces, 2);
    assert!((measure.area_mm2 - 130.).abs() < 1e-6, "{}", measure.area_mm2);
    let mut areas: Vec<_> = (0..2).map(|i| {
        let face = om9_kernel::editing::extract_face(result, i, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
        creation::measure(&face, &limits).unwrap().area_mm2
    }).collect();
    areas.sort_by(f64::total_cmp);
    assert!((areas[0] - 30.).abs() < 1e-6 && (areas[1] - 100.).abs() < 1e-6, "{areas:?}");
    c.submit("").unwrap();
    for _ in 0..4 { c.undo().unwrap(); }
    assert_eq!(c.document(), &before);
    for _ in 0..4 { c.redo().unwrap(); }
    let Geometry::Brep(result) = &c.document().entities[&ids[0]].geometry else { panic!("face") };
    assert_eq!(creation::measure(result, &limits).unwrap().faces, 2);
}

#[test]
fn trim_solid_boundary_with_surface_and_undo() {
    let limits = Limits::default();
    let solid = creation::primitive(Primitive::Box { x_mm: 10., y_mm: 10., z_mm: 10. }, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
    let profile = creation::Profile::new(vec![[5., -5., -5.], [5., 15., -5.], [5., 15., 15.], [5., -5., 15.]]).unwrap();
    let cutter = creation::planar(&profile, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
    let a = Entity::new("box", Geometry::Brep(solid), None);
    let b = Entity::new("cutting face", Geometry::Brep(cutter), None);
    let ids = [a.id, b.id];
    let mut doc = Document::with_template_tolerance("trim solid boundary");
    for entity in [a, b] { doc.entities.insert(entity.id, entity); }
    let mut c = Controller::new(doc, limits.clone()).unwrap();
    let before = c.document().clone();
    c.submit("Trim").unwrap();
    for id in ids { c.top11_accept_object(id).unwrap(); }
    c.submit("").unwrap();
    assert!(c.top11_trim_many_has_target(ids[0]));
    assert!(c.top11_trim_many_has_target(ids[1]));
    c.top11_trim_many_accept_pick(ids[0], [2., 0., 2.]).unwrap();
    let Geometry::Brep(result) = &c.document().entities[&ids[0]].geometry else { panic!("BRep") };
    let measure = creation::measure(result, &limits).unwrap();
    assert_eq!(measure.solids, 0);
    assert_eq!(measure.faces, 6);
    assert!(!measure.closed && measure.naked_edges > 0);
    assert!((measure.area_mm2 - 550.).abs() < 1e-6, "{}", measure.area_mm2);
    assert_eq!(c.document().entities[&ids[1]], before.entities[&ids[1]]);
    c.submit("").unwrap(); c.undo().unwrap();
    assert_eq!(c.document(), &before);
    c.redo().unwrap();
    let Geometry::Brep(result) = &c.document().entities[&ids[0]].geometry else { panic!("BRep") };
    assert!((creation::measure(result, &limits).unwrap().area_mm2 - 550.).abs() < 1e-6);
}

#[test]
fn trim_solid_boundary_with_line_through_shared_session() {
    let limits = Limits::default();
    let solid = creation::primitive(Primitive::Box { x_mm: 10., y_mm: 10., z_mm: 10. }, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
    let a = Entity::new("box", Geometry::Brep(solid), None);
    let line = Entity::new("line", Geometry::Curve(Curve::Line { start: [5., 0., -1.], end: [5., 0., 11.], domain: [0., 1.] }), None);
    let ids = [a.id, line.id];
    let mut doc = Document::with_template_tolerance("solid line Trim");
    for entity in [a, line] { doc.entities.insert(entity.id, entity); }
    let mut c = Controller::new(doc, limits.clone()).unwrap();
    let before = c.document().clone();
    c.select_objects(&ids).unwrap();
    c.top11_invoke(om9_core::top11::Top11Action::Trim).unwrap();
    c.top11_trim_many_accept_pick(ids[0], [2., 0., 2.]).unwrap();
    let Geometry::Brep(result) = &c.document().entities[&ids[0]].geometry else { panic!("BRep") };
    let measure = creation::measure(result, &limits).unwrap();
    assert_eq!(measure.solids, 0);
    assert!((measure.area_mm2 - 550.).abs() < 1e-6, "{}", measure.area_mm2);
    c.submit("").unwrap(); c.undo().unwrap();
    assert_eq!(c.document(), &before);
}

#[test]
fn trim_two_intersecting_solids_updates_selected_boundary() {
    let limits = Limits::default();
    let solid = creation::primitive(Primitive::Box { x_mm: 10., y_mm: 10., z_mm: 10. }, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
    let a = Entity::new("first", Geometry::Brep(solid.clone()), None);
    let mut b = Entity::new("second", Geometry::Brep(solid), None);
    b.placement = Transform::translation([5., 5., 5.]);
    let ids = [a.id, b.id];
    let mut doc = Document::with_template_tolerance("two solid Trim");
    for entity in [a, b] { doc.entities.insert(entity.id, entity); }
    let mut c = Controller::new(doc, limits.clone()).unwrap();
    let before = c.document().clone();
    c.select_objects(&ids).unwrap(); c.submit("Trim").unwrap();
    assert!(c.top11_trim_many_has_target(ids[0]));
    assert!(c.top11_trim_many_has_target(ids[1]));
    c.top11_trim_many_accept_pick(ids[0], [10., 7., 7.]).unwrap();
    let Geometry::Brep(result) = &c.document().entities[&ids[0]].geometry else { panic!("BRep") };
    let measure = creation::measure(result, &limits).unwrap();
    assert_eq!(measure.solids, 0);
    assert!(!measure.closed);
    assert!((measure.area_mm2 - 575.).abs() < 1e-6, "{}", measure.area_mm2);
    assert_eq!(c.document().entities[&ids[1]], before.entities[&ids[1]]);
    assert!(c.top11_trim_many_has_target(ids[1]));
    c.top11_trim_many_accept_pick(ids[1], [5., 7., 7.]).unwrap();
    let Geometry::Brep(result) = &c.document().entities[&ids[1]].geometry else { panic!("BRep") };
    let measure = creation::measure(result, &limits).unwrap();
    assert_eq!(measure.solids, 0);
    assert!((measure.area_mm2 - 575.).abs() < 1e-6, "{}", measure.area_mm2);
    c.submit("").unwrap(); c.undo().unwrap(); c.undo().unwrap();
    assert_eq!(c.document(), &before);
}

#[test]
fn trim_curved_solid_boundary_preserves_expected_area_and_undo() {
    use std::f64::consts::PI;
    let limits = Limits::default();
    for (primitive, z, pick, normal, expected, expected_distance) in [
        (Primitive::Sphere { radius_mm: 5. }, 0., [0., 3., 4.], [0., 0.6, 0.8], 50. * PI, 20_f64.sqrt()),
        (Primitive::Cylinder { radius_mm: 5., height_mm: 10. }, 5., [0., 5., 7.], [0., 1., 0.], 100. * PI, 2.),
    ] {
        let solid = creation::primitive(primitive, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
        let profile = creation::Profile::new(vec![[-10., -10., z], [10., -10., z], [10., 10., z], [-10., 10., z]]).unwrap();
        let cutter = creation::planar(&profile, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
        for placed in [false, true] {
            let mut a = Entity::new("solid", Geometry::Brep(solid.clone()), None);
            let mut b = Entity::new("plane", Geometry::Brep(cutter.clone()), None);
            let placement = if placed { Transform::translation([20., -5., 3.]) } else { Transform::identity() };
            a.placement = placement; b.placement = placement;
            let ids = [a.id, b.id];
            let mut doc = Document::with_template_tolerance("curved solid Trim");
            for entity in [a, b] { doc.entities.insert(entity.id, entity); }
            let mut c = Controller::new(doc, limits.clone()).unwrap();
            let before = c.document().clone();
            c.select_objects(&ids).unwrap(); c.submit("Trim").unwrap();
            let ray = om9_scene::camera::Ray { origin: placement.point([0., 0., 30.]).unwrap(), direction: [1., 0., 0.] };
            assert!(c.top11_trim_many_accept_viewport_pick(ids[0], placement.point([10., 0., 30.]).unwrap(), ray).is_err());
            assert_eq!(c.document(), &before);
            assert!(c.top11_trim_many_has_target(ids[0]));
            if placed {
                let origin = std::array::from_fn(|i| pick[i] + 10. * normal[i]);
                let approximate = std::array::from_fn(|i| pick[i] - 0.05 * normal[i]);
                let ray = om9_scene::camera::Ray { origin: placement.point(origin).unwrap(), direction: normal.map(|x| -x) };
                c.top11_trim_many_accept_viewport_pick(ids[0], placement.point(approximate).unwrap(), ray).unwrap();
            } else {
                c.top11_trim_many_accept_pick(ids[0], placement.point(pick).unwrap()).unwrap();
            }
            let Geometry::Brep(result) = &c.document().entities[&ids[0]].geometry else { panic!("BRep") };
            let measure = creation::measure(result, &limits).unwrap();
            assert_eq!(measure.solids, 0);
            assert!(!measure.closed && measure.naked_edges > 0);
            assert!((measure.area_mm2 - expected).abs() < 1e-5, "{} vs {expected}", measure.area_mm2);
            let distance = om9_kernel::measurement::distance(&Geometry::Brep(result.clone()), Transform::identity(),
                &Geometry::Point(pick), Transform::identity(), &limits).unwrap().millimetres;
            assert!((distance - expected_distance).abs() < 1e-5, "distance {distance} vs {expected_distance}");
            c.submit("").unwrap();
            let path = std::env::temp_dir().join(format!("om9-trim-curved-{}.om9", uuid::Uuid::new_v4()));
            om9_archive::Archive::save(&path, c.document(), &limits, &om9_core::transaction::Cancellation::default()).unwrap();
            let loaded = om9_archive::Archive::open(&path, &limits).unwrap();
            std::fs::remove_file(path).unwrap();
            assert_eq!(&loaded, c.document());
            let reopened = Controller::new(loaded, limits.clone()).unwrap();
            let Geometry::Brep(reopened_brep) = &reopened.document().entities[&ids[0]].geometry else { panic!("BRep") };
            assert!((creation::measure(reopened_brep, &limits).unwrap().area_mm2 - expected).abs() < 1e-5);
            assert_eq!(reopened.document().entities[&ids[0]].placement, placement);
            c.undo().unwrap();
            assert_eq!(c.document(), &before);
            c.redo().unwrap();
            let Geometry::Brep(result) = &c.document().entities[&ids[0]].geometry else { panic!("BRep") };
            assert!((creation::measure(result, &limits).unwrap().area_mm2 - expected).abs() < 1e-5);
        }
    }
}

#[test]
fn trim_disjoint_surfaces_fails_without_a_transaction() {
    let limits = Limits::default();
    let profile = creation::Profile::new(vec![[-5., -5., 0.], [5., -5., 0.], [5., 5., 0.], [-5., 5., 0.]]).unwrap();
    let face = creation::planar(&profile, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
    let a = Entity::new("first", Geometry::Brep(face.clone()), None);
    let mut b = Entity::new("second", Geometry::Brep(face), None);
    b.placement = Transform::translation([0., 0., 5.]);
    let ids = [a.id, b.id];
    let mut doc = Document::with_template_tolerance("disjoint surfaces");
    for entity in [a, b] { doc.entities.insert(entity.id, entity); }
    let mut c = Controller::new(doc, limits).unwrap();
    c.select_objects(&ids).unwrap();
    let before = c.document().clone();
    assert!(c.submit("Trim").is_err());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), 0);
}

#[test]
fn trim_face_inside_closed_circle_preserves_the_exact_hole() {
    use std::f64::consts::{PI, TAU};
    let limits = Limits::default();
    let profile = creation::Profile::new(vec![[0., 0., 0.], [10., 0., 0.], [10., 20., 0.], [0., 20., 0.]]).unwrap();
    let target = Entity::new("face", Geometry::Brep(creation::planar(&profile, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap()), None);
    let cutter = Entity::new("circle", Geometry::Curve(Curve::Circle {
        center: [5., 10., 0.], x: [1., 0., 0.], y: [0., 1., 0.], radius: 2., domain: [0., TAU], angle_domain: None,
    }), None);
    let ids = [target.id, cutter.id];
    let mut doc = Document::with_template_tolerance("trim circular hole");
    for entity in [target, cutter] { doc.entities.insert(entity.id, entity); }
    let mut c = Controller::new(doc, limits.clone()).unwrap();
    let before = c.document().clone();
    c.select_objects(&ids).unwrap();
    c.submit("Trim").unwrap();
    c.accept_world_point([5., 10., 0.]).unwrap();
    let Geometry::Brep(brep) = &c.document().entities[&ids[0]].geometry else { panic!("expected BRep") };
    assert!((creation::measure(brep, &limits).unwrap().area_mm2 - (200. - 4. * PI)).abs() < 1e-6);
    c.submit("").unwrap();
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
}

#[test]
fn trim_translated_face_by_translated_circle_keeps_exact_area() {
    use std::f64::consts::{PI, TAU};
    let limits = Limits::default();
    let profile = creation::Profile::new(vec![[0., 0., 0.], [10., 0., 0.], [10., 20., 0.], [0., 20., 0.]]).unwrap();
    let mut target = Entity::new("face", Geometry::Brep(creation::planar(&profile, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap()), None);
    target.placement = Transform::translation([30., -7., 4.]);
    let mut cutter = Entity::new("circle", Geometry::Curve(Curve::Circle {
        center: [5., 10., 0.], x: [1., 0., 0.], y: [0., 1., 0.], radius: 2., domain: [0., TAU], angle_domain: None,
    }), None);
    cutter.placement = target.placement;
    let ids = [target.id, cutter.id];
    let mut doc = Document::with_template_tolerance("trim translated circular hole");
    for entity in [target, cutter] { doc.entities.insert(entity.id, entity); }
    let mut c = Controller::new(doc, limits.clone()).unwrap();
    c.select_objects(&ids).unwrap();
    c.submit("Trim").unwrap();
    c.accept_world_point([35., 3., 4.]).unwrap();
    let Geometry::Brep(brep) = &c.document().entities[&ids[0]].geometry else { panic!("expected BRep") };
    let area = creation::measure(brep, &limits).unwrap().area_mm2;
    assert!((area - (200. - 4. * PI)).abs() < 1e-6, "area={area}");
}

#[test]
fn trim_nonuniformly_placed_face_by_circle_preserves_local_area() {
    use std::f64::consts::{PI, TAU};
    let limits = Limits::default();
    let profile = creation::Profile::new(vec![[0., 0., 0.], [10., 0., 0.], [10., 20., 0.], [0., 20., 0.]]).unwrap();
    let mut target = Entity::new("face", Geometry::Brep(creation::planar(&profile, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap()), None);
    target.placement = Transform([[2., 0., 0., 30.], [0., 1., 0., -7.], [0., 0., 1., 4.], [0., 0., 0., 1.]]);
    let mut cutter = Entity::new("circle", Geometry::Curve(Curve::Circle {
        center: [5., 10., 0.], x: [1., 0., 0.], y: [0., 1., 0.], radius: 2., domain: [0., TAU], angle_domain: None,
    }), None);
    cutter.placement = target.placement;
    let ids = [target.id, cutter.id];
    let mut doc = Document::with_template_tolerance("trim affine circular hole");
    for entity in [target, cutter] { doc.entities.insert(entity.id, entity); }
    let mut c = Controller::new(doc, limits.clone()).unwrap();
    let before = c.document().clone();
    c.select_objects(&ids).unwrap();
    c.submit("Trim").unwrap();
    c.accept_world_point([40., 3., 4.]).unwrap();
    let Geometry::Brep(brep) = &c.document().entities[&ids[0]].geometry else { panic!("expected BRep") };
    let area = creation::measure(brep, &limits).unwrap().area_mm2;
    assert!((area - (200. - 4. * PI)).abs() < 1e-5, "area={area}");
    let geometry = Geometry::Brep(brep.clone());
    let distance = |point| om9_kernel::measurement::distance(
        &geometry, Transform::identity(), &Geometry::Point(point), Transform::identity(), &limits,
    ).unwrap().millimetres;
    assert!((distance([5., 10., 0.]) - 2.).abs() < 1e-6);
    assert!(distance([8., 10., 0.]) < 1e-7);
    assert_eq!(c.document().entities[&ids[0]].placement, before.entities[&ids[0]].placement);
    c.submit("").unwrap();
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
}

#[test]
fn trim_curve_piercing_surface_or_brep_keeps_surface_and_recomputes() {
    use om9_core::geometry::NurbsSurface;
    let limits = Limits::default();
    let profile = creation::Profile::new(vec![[0., 0., 0.], [10., 0., 0.], [10., 20., 0.], [0., 20., 0.]]).unwrap();
    let surfaces = [
        Geometry::Brep(creation::planar(&profile, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap()),
        Geometry::Surface(NurbsSurface {
            degree: [1, 1], size: [2, 2], poles: vec![[0., 0., 0.], [0., 20., 0.], [10., 0., 0.], [10., 20., 0.]],
            weights: vec![1.; 4], knots: [vec![0., 0., 1., 1.], vec![0., 0., 1., 1.]],
            domain: [[0., 1.], [0., 1.]], periodic: [false, false], reversed: false,
        }),
    ];
    for (kind, surface) in surfaces.into_iter().enumerate() {
        let mut doc = Document::with_template_tolerance("mutual trim piercing curve");
        let face = Entity::new("face", surface, None);
        let line = Entity::new("line", Geometry::Curve(Curve::Line {
            start: [2., 2., -5.], end: [2., 2., 5.], domain: [0., 1.],
        }), None);
        let ids = [face.id, line.id];
        for entity in [face, line] { doc.entities.insert(entity.id, entity); }
        let mut c = Controller::new(doc, limits.clone()).unwrap();
        let before = c.document().clone();
        c.submit("Trim").unwrap();
        c.top11_accept_object(ids[0]).unwrap();
        c.top11_accept_object(ids[1]).unwrap();
        c.submit("").unwrap();
        assert!(c.top11_trim_many_has_target(ids[1]), "initial target kind {kind}");
        assert!(!c.top11_trim_many_has_target(ids[0]));
        c.top11_trim_many_accept_pick(ids[1], [2., 2., -2.]).unwrap();
        let Geometry::Curve(curve) = &c.document().entities[&ids[1]].geometry else { panic!("expected curve") };
        assert!((om9_core::curve_ops::arc_length(curve, &limits).unwrap() - 5.).abs() < 1e-6);
        assert_eq!(c.document().entities[&ids[0]], before.entities[&ids[0]]);
        assert!(c.top11_trim_many_has_target(ids[1]), "remaining target kind {kind}");
        c.top11_trim_many_accept_pick(ids[1], [2., 2., 2.]).unwrap();
        assert!(!c.document().entities.contains_key(&ids[1]));
        c.submit("").unwrap();
        c.undo().unwrap();
        c.undo().unwrap();
        assert_eq!(c.document(), &before);
    }
}

#[test]
fn brep_intersection_can_include_endpoints_without_changing_split_filter() {
    let limits = Limits::default();
    let profile = creation::Profile::new(vec![[0., 0., 0.], [10., 0., 0.], [10., 10., 0.], [0., 10., 0.]]).unwrap();
    let face = creation::planar(&profile, creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
    let curve = Curve::Line { start: [2., 2., 0.], end: [2., 2., 5.], domain: [0., 1.] };
    assert!(om9_kernel::intersect::curve_brep_fractions(&face, &curve, 1e-6, &limits).unwrap().is_empty());
    let hits = om9_kernel::intersect::curve_brep_fractions_including_endpoints(&face, &curve, 1e-6, &limits).unwrap();
    assert_eq!(hits, vec![0.]);
}

#[test]
fn trim_curve_crossing_solid_brep_can_remove_the_interior_segment() {
    let mut doc = Document::with_template_tolerance("trim curve through solid");
    let solid = Entity::new("box", Geometry::Brep(box_brep()), None);
    let line = Entity::new("line", Geometry::Curve(Curve::Line {
        start: [1., 1., -2.], end: [1., 1., 6.], domain: [0., 1.],
    }), None);
    let ids = [solid.id, line.id];
    for entity in [solid, line] { doc.entities.insert(entity.id, entity); }
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    let before = c.document().clone();
    c.select_objects(&ids).unwrap();
    c.submit("Trim").unwrap();
    c.top11_trim_many_accept_pick(ids[1], [1., 1., 2.]).unwrap();
    assert!(!c.document().entities.contains_key(&ids[1]));
    assert_eq!(c.document().entities.len(), 3);
    let mut lengths: Vec<_> = c.document().entities.values().filter_map(|entity| match &entity.geometry {
        Geometry::Curve(curve) => Some(om9_core::curve_ops::arc_length(curve, &Limits::default()).unwrap()),
        _ => None,
    }).collect();
    lengths.sort_by(f64::total_cmp);
    assert!(lengths.iter().all(|length| (*length - 2.).abs() < 1e-6), "{lengths:?}");
    assert_eq!(c.document().entities[&ids[0]], before.entities[&ids[0]]);
    c.submit("").unwrap();
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
}

fn controller_with_box() -> (Controller, om9_core::identity::ObjectId) {
    let mut document = Document::new("top11-brep");
    let mut entity = Entity::new("box", Geometry::Brep(box_brep()), None);
    entity.placement = Transform::translation([5.0, 0.0, 0.0]);
    let id = entity.id;
    document.entities.insert(id, entity);
    (
        Controller::new(document, Limits::default()).expect("controller"),
        id,
    )
}

#[test]
fn top11_explode_brep_creates_faces_in_one_undo() {
    let (mut c, id) = controller_with_box();
    c.select_objects(&[id]).unwrap();
    c.top11_invoke(om9_core::top11::Top11Action::Explode)
        .unwrap();
    assert!(!c.document().entities.contains_key(&id));
    assert_eq!(c.document().entities.len(), 6);
    let mut total_area = 0.0;
    for face in c.document().entities.values() {
        assert_eq!(face.placement, Transform::translation([5.0, 0.0, 0.0]));
        let Geometry::Brep(brep) = &face.geometry else {
            panic!("expected face BRep")
        };
        let measure = creation::measure(brep, c.coordinator().limits()).unwrap();
        assert_eq!(measure.faces, 1);
        total_area += measure.area_mm2;
    }
    assert!((total_area - 52.0).abs() < 1e-6, "{total_area}");
    assert_eq!(c.coordinator().undo_len(), 1);
    c.undo().unwrap();
    assert_eq!(c.document().entities.len(), 1);
    assert!(c.document().entities.contains_key(&id));
}

#[test]
fn explode_retained_native_brep_uses_exact_converted_faces() {
    use om9_core::top11::Top11Action;
    let limits = Limits::default();
    let native = om9_kernel::brep::to_3dm(&box_brep(), 0.001, &limits).unwrap();
    for toolbar in [false, true] {
        let mut source = Entity::new("retained box", Geometry::NativeBrep(native.clone()), None);
        source.placement = Transform::translation([5., 0., 0.]); let id = source.id;
        let mut doc = Document::with_template_tolerance("retained explode"); doc.entities.insert(id, source);
        let mut c = Controller::new(doc, limits).unwrap(); c.select_objects(&[id]).unwrap();
        let before = c.document().clone();
        if toolbar { c.top11_invoke(Top11Action::Explode).unwrap(); } else { c.submit("Explode").unwrap(); }
        assert!(!c.document().entities.contains_key(&id)); assert_eq!(c.document().entities.len(), 6);
        let mut area = 0.;
        for entity in c.document().entities.values() {
            assert_eq!(entity.placement, Transform::translation([5., 0., 0.]));
            let Geometry::Brep(face) = &entity.geometry else { panic!("expected exact BRep face") };
            let measure = creation::measure(face, &limits).unwrap(); assert_eq!(measure.faces, 1); area += measure.area_mm2;
        }
        assert!((area - 52.).abs() < 1e-6, "{area}");
        let after = c.document().clone();
        let path = std::env::temp_dir().join(format!("om9-native-explode-{}.om9", uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&path, &after, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
        assert_eq!(om9_archive::Archive::open(&path, &limits).unwrap(), after); std::fs::remove_file(path).unwrap();
        c.undo().unwrap(); assert_eq!(c.document(), &before); c.redo().unwrap(); assert_eq!(c.document(), &after);
    }
}

#[test]
fn explode_single_face_rejects_without_mutating_document() {
    let face = om9_kernel::editing::extract_face(
        &box_brep(),
        0,
        creation::DEFAULT_DEFLECTION_MM,
        &Limits::default(),
    )
    .unwrap();
    let mut doc = Document::new("single face");
    let entity = Entity::new("face", Geometry::Brep(face), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    let before = c.document().clone();
    assert!(c.submit("Explode").is_err());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), 0);
}

#[test]
fn top11_join_adjacent_brep_faces_and_undo() {
    let source = box_brep();
    let mut doc = Document::with_template_tolerance("join adjacent faces");
    let mut ids = Vec::new();
    for index in [0, 2] {
        let face = om9_kernel::editing::extract_face(
            &source,
            index,
            creation::DEFAULT_DEFLECTION_MM,
            &Limits::default(),
        )
        .unwrap();
        let mut entity = Entity::new("face", Geometry::Brep(face), None);
        entity.placement = Transform::translation([5., 0., 0.]);
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&ids).unwrap();
    c.top11_invoke(om9_core::top11::Top11Action::Join).unwrap();
    assert_eq!(c.document().entities.len(), 1);
    let joined = c.document().entities.values().next().unwrap();
    assert_eq!(joined.placement, Transform::translation([5., 0., 0.]));
    let Geometry::Brep(brep) = &joined.geometry else {
        panic!("expected BRep")
    };
    let measure = creation::measure(brep, c.coordinator().limits()).unwrap();
    assert_eq!(measure.faces, 2);
    assert_eq!(measure.naked_edges, 6);
    assert_eq!(c.coordinator().undo_len(), 1);
    c.undo().unwrap();
    assert_eq!(c.document().entities.len(), 2);
    assert!(ids.iter().all(|id| c.document().entities.contains_key(id)));
    c.redo().unwrap();
    assert_eq!(c.document().entities.len(), 1);
    assert!(ids.iter().all(|id| !c.document().entities.contains_key(id)));
}

#[test]
fn top11_join_disconnected_faces_keeps_both_without_undo() {
    let source = box_brep();
    let mut doc = Document::with_template_tolerance("disconnected faces");
    let mut ids = Vec::new();
    for index in [0, 1] {
        let face = om9_kernel::editing::extract_face(
            &source,
            index,
            creation::DEFAULT_DEFLECTION_MM,
            &Limits::default(),
        )
        .unwrap();
        let entity = Entity::new("face", Geometry::Brep(face), None);
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&ids).unwrap();
    let before = c.document().clone();
    assert!(c.submit("Join").is_err());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), 0);
}

#[test]
fn top11_join_faces_with_different_placements_and_undo() {
    let source = box_brep();
    let limits = Limits::default();
    let face_a =
        om9_kernel::editing::extract_face(&source, 0, creation::DEFAULT_DEFLECTION_MM, &limits)
            .unwrap();
    let face_b =
        om9_kernel::editing::extract_face(&source, 2, creation::DEFAULT_DEFLECTION_MM, &limits)
            .unwrap();
    let shifted = om9_kernel::transform_ops::transform_brep(
        &face_b,
        &Transform::translation([-5., 0., 0.]),
        creation::DEFAULT_DEFLECTION_MM,
        &limits,
    )
    .unwrap();
    let a = Entity::new("face a", Geometry::Brep(face_a), None);
    let mut b = Entity::new("face b", Geometry::Brep(shifted), None);
    b.placement = Transform::translation([5., 0., 0.]);
    let ids = [a.id, b.id];
    let mut doc = Document::with_template_tolerance("join placed faces");
    doc.entities.insert(a.id, a);
    doc.entities.insert(b.id, b);
    let mut c = Controller::new(doc, limits).unwrap();
    c.select_objects(&ids).unwrap();
    c.submit("Join").unwrap();
    let joined = c.document().entities.values().next().unwrap();
    let Geometry::Brep(brep) = &joined.geometry else {
        panic!("BRep")
    };
    let world = om9_kernel::transform_ops::transform_brep(
        brep,
        &joined.placement,
        creation::DEFAULT_DEFLECTION_MM,
        c.coordinator().limits(),
    )
    .unwrap();
    let measures = creation::measure(&world, c.coordinator().limits()).unwrap();
    assert_eq!(measures.faces, 2);
    assert_eq!(measures.naked_edges, 6);
    c.undo().unwrap();
    assert_eq!(c.document().entities.len(), 2);
    assert!(ids.iter().all(|id| c.document().entities.contains_key(id)));
}

#[test]
fn top11_join_without_document_tolerance_is_atomic() {
    let source = box_brep();
    let mut doc = Document::new("missing tolerance");
    let mut ids = Vec::new();
    for index in [0, 2] {
        let face = om9_kernel::editing::extract_face(
            &source,
            index,
            creation::DEFAULT_DEFLECTION_MM,
            &Limits::default(),
        )
        .unwrap();
        let entity = Entity::new("face", Geometry::Brep(face), None);
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&ids).unwrap();
    let before = c.document().clone();
    assert!(c.submit("Join").is_err());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), 0);
}

#[test]
fn top11_join_three_faces_finds_connected_order_and_undo_redo() {
    let source = box_brep();
    let mut doc = Document::with_template_tolerance("three faces");
    let mut ids = Vec::new();
    for index in [0, 1, 2] {
        let face = om9_kernel::editing::extract_face(
            &source,
            index,
            creation::DEFAULT_DEFLECTION_MM,
            &Limits::default(),
        )
        .unwrap();
        let entity = Entity::new("face", Geometry::Brep(face), None);
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&ids).unwrap();
    c.submit("Join").unwrap();
    assert_eq!(c.document().entities.len(), 1);
    let joined = c.document().entities.values().next().unwrap();
    let Geometry::Brep(brep) = &joined.geometry else {
        panic!("BRep")
    };
    let measure = creation::measure(brep, c.coordinator().limits()).unwrap();
    assert_eq!(measure.faces, 3);
    assert_eq!(measure.naked_edges, 8);
    assert_eq!(c.coordinator().undo_len(), 1);
    c.undo().unwrap();
    assert_eq!(c.document().entities.len(), 3);
    assert!(ids.iter().all(|id| c.document().entities.contains_key(id)));
    c.redo().unwrap();
    assert_eq!(c.document().entities.len(), 1);
}

#[test]
fn top11_join_three_faces_with_disconnected_member_is_atomic() {
    let source = box_brep();
    let mut doc = Document::with_template_tolerance("disconnected member");
    let mut ids = Vec::new();
    for index in [0, 1, 2] {
        let face = om9_kernel::editing::extract_face(
            &source,
            index,
            creation::DEFAULT_DEFLECTION_MM,
            &Limits::default(),
        )
        .unwrap();
        let mut entity = Entity::new("face", Geometry::Brep(face), None);
        if index == 1 {
            entity.placement = Transform::translation([100., 0., 0.]);
        }
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&ids).unwrap();
    let before = c.document().clone();
    assert!(c.submit("Join").is_err());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), 0);
}

#[test]
fn brep_mirror_deletes_input_in_place_and_keeps_one_undo() {
    let (mut c, id) = controller_with_box();
    c.select_objects(&[id]).unwrap();
    c.top11_mirror(false).unwrap();
    c.accept_world_point([0.0, -5.0, 0.0]).unwrap();
    c.accept_world_point([0.0, 5.0, 0.0]).unwrap();
    assert_eq!(c.document().entities.len(), 1);
    let Geometry::Brep(brep) = &c.document().entities[&id].geometry else {
        panic!("mirror lost the BRep");
    };
    let measures = creation::measure(brep, c.coordinator().limits()).unwrap();
    assert!(
        (measures.volume_mm3 - 24.0).abs() < 1e-6,
        "{}",
        measures.volume_mm3
    );
    assert!(
        (measures.bounds_mm[0] + 12.0).abs() < 1e-4,
        "{:?}",
        measures.bounds_mm
    );
    assert!(
        (measures.bounds_mm[3] + 10.0).abs() < 1e-4,
        "{:?}",
        measures.bounds_mm
    );
    assert!(measures.closed);
    assert_eq!(c.coordinator().undo_len(), 1);
    c.undo().unwrap();
    assert_eq!(c.document().entities.len(), 1);
    println!("brep_mirror_volume_mm3={:.6}", measures.volume_mm3);
    println!("brep_mirror_local_bounds={:?}", measures.bounds_mm);
}

#[test]
fn brep_three_point_mirror_accepts_small_plane_definition() {
    let (mut c, id) = controller_with_box();
    c.select_objects(&[id]).unwrap();
    let before = c.document().clone();
    c.top11_mirror(false).unwrap(); c.submit("3Point").unwrap();
    c.accept_world_point([0.; 3]).unwrap(); c.accept_world_point([0., 1e-9, 0.]).unwrap();
    let target = [0., 0., 1e-9];
    assert!(!c.transform_preview(target, "").unwrap().is_empty());
    c.accept_world_point(target).unwrap();
    let Geometry::Brep(brep) = &c.document().entities[&id].geometry else { panic!("BRep") };
    let measures = creation::measure(brep, c.coordinator().limits()).unwrap();
    assert!((measures.volume_mm3 - 24.).abs() < 1e-6 && measures.closed);
    assert!((measures.bounds_mm[0] + 12.).abs() < 1e-4 && (measures.bounds_mm[3] + 10.).abs() < 1e-4, "{:?}", measures.bounds_mm);
    c.undo().unwrap(); assert_eq!(c.document(), &before);
}

#[test]
fn brep_mirror_copy_leaves_the_source_in_place() {
    let (mut c, id) = controller_with_box();
    c.select_objects(&[id]).unwrap();
    c.top11_mirror(true).unwrap();
    c.accept_world_point([0.0, -5.0, 0.0]).unwrap();
    c.accept_world_point([0.0, 5.0, 0.0]).unwrap();
    assert_eq!(c.document().entities.len(), 2);
    let copy_id = *c.selected().iter().next().unwrap();
    assert_ne!(copy_id, id);
    let Geometry::Brep(copy) = &c.document().entities[&copy_id].geometry else {
        panic!("mirror did not copy the BRep");
    };
    let measures = creation::measure(copy, c.coordinator().limits()).unwrap();
    assert!(
        (measures.bounds_mm[0] + 7.0).abs() < 1e-4,
        "{:?}",
        measures.bounds_mm
    );
    assert!(
        (measures.bounds_mm[3] + 5.0).abs() < 1e-4,
        "{:?}",
        measures.bounds_mm
    );
    println!("brep_mirror_copy_bounds={:?}", measures.bounds_mm);
}

#[test]
fn brep_can_be_a_cutting_object_for_split() {
    let mut document = Document::with_template_tolerance("brep-cutter");
    let line = Entity::new(
        "line",
        Geometry::Curve(Curve::Line {
            start: [-5.0, 1.0, 1.0],
            end: [5.0, 1.0, 1.0],
            domain: [0.0, 10.0],
        }),
        None,
    );
    let line_id = line.id;
    document.entities.insert(line_id, line);
    let box_entity = Entity::new("box", Geometry::Brep(box_brep()), None);
    let box_id = box_entity.id;
    document.entities.insert(box_id, box_entity);
    let mut c = Controller::new(document, Limits::default()).unwrap();
    c.select_objects(&[line_id]).unwrap();
    c.top11_invoke(om9_core::top11::Top11Action::Split).unwrap();
    c.top11_accept_object(box_id).unwrap();
    c.submit("").unwrap();
    assert_eq!(c.document().entities.len(), 4);
    assert_eq!(c.selected().len(), 3);
    println!("brep_cutter_entities={}", c.document().entities.len());
}

#[test]
fn split_periodic_nurbs_with_brep_cutter_keeps_exact_pieces() {
    use om9_core::{curve_ops,geometry::NurbsCurve,top11::Top11Action,transaction::Cancellation};
    let limits=Limits::default();
    let body=creation::primitive(Primitive::Box{x_mm:10.,y_mm:20.,z_mm:10.},creation::DEFAULT_DEFLECTION_MM,&limits).unwrap();
    let mut n=NurbsCurve{degree:2,poles:vec![[-8.,5.,5.],[18.,5.,5.],[18.,15.,5.],[-8.,15.,5.],[-8.,5.,5.],[18.,5.,5.]],
        weights:vec![1.,0.7,1.2,0.9,1.,0.7],knots:vec![-2.,-1.,0.,1.,2.,3.,4.,5.,6.],domain:[0.,4.],periodic:true,reversed:false,seam:None};
    for reversed in [false,true] {for shifted in [false,true] {for toolbar in [false,true] {
        n.reversed=reversed;let mut curve=Curve::Nurbs(n.clone());if shifted {curve=curve_ops::set_seam(&curve,0.31,1e-9,&limits).unwrap();}
        let mut target=Entity::new("periodic",Geometry::Curve(curve.clone()),None);target.placement=Transform::translation([3.,-2.,1.]);
        let mut cutter=Entity::new("box",Geometry::Brep(body.clone()),None);cutter.placement=target.placement;
        let ids=[target.id,cutter.id];let mut doc=Document::with_template_tolerance("periodic BRep split");
        doc.entities.insert(target.id,target);doc.entities.insert(cutter.id,cutter);let mut c=Controller::new(doc,limits).unwrap();
        c.select_objects(&[ids[0]]).unwrap();let before=c.document().clone();
        if toolbar {c.top11_invoke(Top11Action::Split).unwrap();} else {c.submit("Split").unwrap();}
        c.top11_accept_object(ids[1]).unwrap();c.submit("").unwrap();assert_eq!(c.selected().len(),4);
        let sum:f64=c.selected().iter().map(|id|{let Geometry::Curve(piece)=&c.document().entities[id].geometry else {panic!("curve")};curve_ops::arc_length(piece,&limits).unwrap()}).sum();
        assert!((sum-curve_ops::arc_length(&curve,&limits).unwrap()).abs()<1e-5);
        for id in c.selected() {let Geometry::Curve(piece)=&c.document().entities[id].geometry else {panic!("curve")};
            for i in 0..=10 {let p=curve_ops::point_at(piece,i as f64/10.,&limits).unwrap();
                assert!(om9_core::curve_surface_ops::closest_point_on_curve(&curve,p,&limits).unwrap().2<1e-6);}}
        let after=c.document().clone();let path=std::env::temp_dir().join(format!("om9-periodic-brep-split-{}.om9",uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&path,&after,&limits,&Cancellation::default()).unwrap();assert_eq!(om9_archive::Archive::open(&path,&limits).unwrap(),after);std::fs::remove_file(path).unwrap();
        c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }}}
}

#[test]
fn trim_periodic_nurbs_against_brep_removes_picked_interval() {
    use om9_core::{curve_ops,geometry::NurbsCurve,top11::Top11Action};
    let limits=Limits::default();
    let body=creation::primitive(Primitive::Box{x_mm:10.,y_mm:20.,z_mm:10.},creation::DEFAULT_DEFLECTION_MM,&limits).unwrap();
    let mut n=NurbsCurve{degree:2,poles:vec![[-8.,5.,5.],[18.,5.,5.],[18.,15.,5.],[-8.,15.,5.],[-8.,5.,5.],[18.,5.,5.]],
        weights:vec![1.,0.7,1.2,0.9,1.,0.7],knots:vec![-2.,-1.,0.,1.,2.,3.,4.,5.,6.],domain:[0.,4.],periodic:true,reversed:false,seam:None};
    for reversed in [false,true] {for shifted in [false,true] {for toolbar in [false,true] {
        n.reversed=reversed;let mut curve=Curve::Nurbs(n.clone());if shifted {curve=curve_ops::set_seam(&curve,0.31,1e-9,&limits).unwrap();}
        let p=curve_ops::point_at(&curve,0.25,&limits).unwrap();
        let a=Entity::new("periodic",Geometry::Curve(curve.clone()),None);let b=Entity::new("box",Geometry::Brep(body.clone()),None);let ids=[a.id,b.id];
        let mut doc=Document::with_template_tolerance("periodic BRep trim");doc.entities.insert(a.id,a);doc.entities.insert(b.id,b);
        let mut c=Controller::new(doc,limits).unwrap();c.select_objects(&ids).unwrap();let before=c.document().clone();
        if toolbar {c.top11_invoke(Top11Action::Trim).unwrap();} else {c.submit("Trim").unwrap();}
        c.top11_trim_many_accept_pick(ids[0],p).unwrap();c.submit("").unwrap();
        let results:Vec<_>=c.document().entities.values().filter_map(|e|if let Geometry::Curve(curve)=&e.geometry {Some(curve)} else {None}).collect();
        assert!(!results.is_empty());assert!(results.iter().all(|curve|om9_core::curve_surface_ops::closest_point_on_curve(curve,p,&limits).unwrap().2>1e-4));
        let total:f64=results.iter().map(|curve|curve_ops::arc_length(curve,&limits).unwrap()).sum();assert!(total<curve_ops::arc_length(&curve,&limits).unwrap());
        let after=c.document().clone();c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }}}
}

#[test]
fn trim_surface_inside_periodic_nurbs_boundary() {
    use om9_core::{geometry::NurbsCurve,top11::Top11Action};
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,20.,&limits).unwrap();
    let curve=Curve::Nurbs(NurbsCurve{degree:2,poles:vec![[3.,8.,0.],[7.,8.,0.],[7.,12.,0.],[3.,12.,0.],[3.,8.,0.],[7.,8.,0.]],
        weights:vec![1.,0.7,1.2,0.9,1.,0.7],knots:vec![-2.,-1.,0.,1.,2.,3.,4.,5.,6.],domain:[0.,4.],periodic:true,reversed:false,seam:None});
    for toolbar in [false,true] {
        let a=Entity::new("surface",Geometry::Surface(surface.clone()),None);let b=Entity::new("periodic",Geometry::Curve(curve.clone()),None);let ids=[a.id,b.id];
        let mut doc=Document::with_template_tolerance("periodic boundary trim");doc.entities.insert(a.id,a);doc.entities.insert(b.id,b);
        let mut c=Controller::new(doc,limits).unwrap();c.select_objects(&ids).unwrap();let before=c.document().clone();
        if toolbar {c.top11_invoke(Top11Action::Trim).unwrap();} else {c.submit("Trim").unwrap();}
        c.top11_trim_many_accept_pick(ids[0],[5.,10.,0.]).unwrap();c.submit("").unwrap();
        let Geometry::Brep(result)=&c.document().entities[&ids[0]].geometry else {panic!("trimmed face")};
        let m=creation::measure(result,&limits).unwrap();assert_eq!(m.faces,1);assert!(m.area_mm2>180. && m.area_mm2<199.);
        let after=c.document().clone();c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }
}

#[test]
fn placed_brep_cuts_a_differently_placed_line() {
    let mut document = Document::with_template_tolerance("placed brep cutter");
    let mut line = Entity::new(
        "line",
        Geometry::Curve(Curve::Line {
            start: [-5., 1., 1.],
            end: [5., 1., 1.],
            domain: [0., 10.],
        }),
        None,
    );
    line.placement = Transform::translation([10., 0., 0.]);
    let line_id = line.id;
    let mut cutter = Entity::new("box", Geometry::Brep(box_brep()), None);
    cutter.placement = Transform::translation([11., 0., 0.]);
    let cutter_id = cutter.id;
    document.entities.insert(line_id, line);
    document.entities.insert(cutter_id, cutter);
    let mut c = Controller::new(document, Limits::default()).unwrap();
    c.select_objects(&[line_id]).unwrap();
    c.top11_invoke(om9_core::top11::Top11Action::Split).unwrap();
    c.top11_accept_object(cutter_id).unwrap();
    c.submit("").unwrap();
    let mut bounds: Vec<_> = c
        .selected()
        .iter()
        .map(|id| {
            let entity = &c.document().entities[id];
            let Geometry::Curve(curve) = &entity.geometry else {
                panic!("curve")
            };
            let ends = om9_core::curve_ops::curve_endpoints(curve, &Limits::default()).unwrap();
            (
                entity.placement.point(ends[0]).unwrap()[0],
                entity.placement.point(ends[1]).unwrap()[0],
            )
        })
        .collect();
    bounds.sort_by(|a, b| a.0.total_cmp(&b.0));
    assert_eq!(bounds.len(), 3);
    for (actual, expected) in bounds.iter().zip([(5., 11.), (11., 13.), (13., 15.)]) {
        assert!(
            (actual.0 - expected.0).abs() < 1e-5 && (actual.1 - expected.1).abs() < 1e-5,
            "{bounds:?}"
        );
    }
    c.undo().unwrap();
    assert!(c.document().entities.contains_key(&line_id));
    assert!(c.document().entities.contains_key(&cutter_id));
    c.redo().unwrap();
    assert!(!c.document().entities.contains_key(&line_id));
    assert_eq!(c.document().entities.len(), 4);
}

#[test]
fn brep_mirror_accepts_a_non_rigid_placement() {
    let mut document = Document::new("brep-non-rigid");
    let mut entity = Entity::new("box", Geometry::Brep(box_brep()), None);
    entity.placement = Transform::scale([2.0, 1.0, 1.0]).unwrap();
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut c = Controller::new(document, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.top11_mirror(true).unwrap();
    c.accept_world_point([0.0, -5.0, 0.0]).unwrap();
    c.accept_world_point([0.0, 5.0, 0.0]).unwrap();
    let copy_id = *c.selected().iter().next().unwrap();
    let Geometry::Brep(copy) = &c.document().entities[&copy_id].geometry else {
        panic!("non-rigid BRep mirror lost the BRep");
    };
    let measures = creation::measure(copy, c.coordinator().limits()).unwrap();
    assert!(
        (measures.bounds_mm[0] + 4.0).abs() < 1e-4,
        "{:?}",
        measures.bounds_mm
    );
    assert!(
        (measures.bounds_mm[3] - 0.0).abs() < 1e-4,
        "{:?}",
        measures.bounds_mm
    );
    println!("brep_non_rigid_mirror_bounds={:?}", measures.bounds_mm);
}

#[test]
fn exact_brep_intersection_uses_the_analytic_sphere_surface() {
    let limits = Limits::default();
    let sphere = creation::primitive(
        Primitive::Sphere { radius_mm: 5.0 },
        creation::DEFAULT_DEFLECTION_MM,
        &limits,
    )
    .expect("sphere");
    let line = Curve::Line {
        start: [-10.0, 0.0, 0.0],
        end: [10.0, 0.0, 0.0],
        domain: [0.0, 20.0],
    };
    let fractions =
        om9_kernel::intersect::curve_brep_fractions(&sphere, &line, 1e-7, &limits).unwrap();
    assert_eq!(fractions.len(), 2, "{fractions:?}");
    assert!((fractions[0] - 0.25).abs() < 1e-6, "{fractions:?}");
    assert!((fractions[1] - 0.75).abs() < 1e-6, "{fractions:?}");
    println!("exact_brep_fractions={fractions:?}");
}

#[test]
fn exact_brep_intersection_handles_nurbs_and_polycurve_targets() {
    let limits = Limits::default();
    let sphere = creation::primitive(
        Primitive::Sphere { radius_mm: 5.0 },
        creation::DEFAULT_DEFLECTION_MM,
        &limits,
    )
    .expect("sphere");
    let nurbs = Curve::Nurbs(NurbsCurve {
        degree: 1,
        poles: vec![[-10.0, 0.0, 0.0], [10.0, 0.0, 0.0]],
        weights: vec![1.0, 1.0],
        knots: vec![0.0, 0.0, 1.0, 1.0],
        domain: [0.0, 1.0],
        periodic: false,
        reversed: false,
        seam: None,
    });
    let fractions =
        om9_kernel::intersect::curve_brep_fractions(&sphere, &nurbs, 1e-7, &limits).unwrap();
    assert_eq!(fractions.len(), 2, "{fractions:?}");
    assert!((fractions[0] - 0.25).abs() < 1e-6, "{fractions:?}");

    let polycurve = Curve::Polycurve {
        segments: vec![
            Curve::Line {
                start: [-10.0, 0.0, 0.0],
                end: [0.0, 0.0, 0.0],
                domain: [0.0, 10.0],
            },
            Curve::Line {
                start: [0.0, 0.0, 0.0],
                end: [10.0, 0.0, 0.0],
                domain: [0.0, 10.0],
            },
        ],
        reversed: false,
    };
    let fractions =
        om9_kernel::intersect::curve_brep_fractions(&sphere, &polycurve, 1e-7, &limits).unwrap();
    assert_eq!(fractions.len(), 2, "{fractions:?}");
    assert!((fractions[0] - 0.25).abs() < 1e-6, "{fractions:?}");
    assert!((fractions[1] - 0.75).abs() < 1e-6, "{fractions:?}");
    println!("exact_nurbs_polycurve_fractions={fractions:?}");
}

#[test]
fn explode_mixed_group_preserves_membership_and_atomic_history() {
    use om9_core::geometry::Mesh;
    use om9_core::model::Group;
    use om9_core::top11::Top11Action;
    for via_toolbar in [false,true] {
        let line=|start,end|Curve::Line{start,end,domain:[0.,1.]};
        let curve=Entity::new("curve",Geometry::Curve(Curve::Polycurve{segments:vec![line([0.,0.,0.],[1.,0.,0.]),line([1.,0.,0.],[2.,0.,0.])],reversed:false}),None);
        let mesh=Entity::new("mesh",Geometry::Mesh(Mesh{vertices:vec![[0.,0.,0.],[1.,0.,0.],[0.,1.,0.],[3.,0.,0.],[4.,0.,0.],[3.,1.,0.]],faces:vec![vec![0,1,2],vec![3,4,5]],normals:vec![],colors:vec![],corner_uvs:vec![]}),None);
        let solid=Entity::new("solid",Geometry::Brep(box_brep()),None);
        let other=Entity::new("other",Geometry::Point([9.,9.,9.]),None);
        let ids=[curve.id,mesh.id,solid.id];let other_id=other.id;
        let mut doc=Document::with_template_tolerance("mixed explode groups");for e in [curve,mesh,solid,other] {doc.entities.insert(e.id,e);}
        let group=Group::new("mixed",vec![ids[0],ids[1],ids[2],other_id]);let group_id=group.id;doc.groups.insert(group_id,group);
        let mut c=Controller::new(doc,Limits::default()).unwrap();c.select_objects(&ids).unwrap();let before=c.document().clone();
        if via_toolbar {c.top11_invoke(Top11Action::Explode).unwrap();}else{c.submit("Explode").unwrap();}
        let after=c.document().clone();assert_eq!(after.entities.len(),11);let members=&after.groups[&group_id].members;assert_eq!(members.len(),11);assert!(after.entities.keys().all(|id|members.contains(id)));assert!(ids.iter().all(|id|!members.contains(id)));
        assert_eq!(after.entities.values().filter(|e|matches!(e.geometry,Geometry::Curve(_))).count(),2);
        assert_eq!(after.entities.values().filter(|e|matches!(e.geometry,Geometry::Mesh(_))).count(),2);
        assert_eq!(after.entities.values().filter(|e|matches!(e.geometry,Geometry::Brep(_))).count(),6);
        let path=std::env::temp_dir().join(format!("om9-mixed-explode-{}.om9",uuid::Uuid::new_v4()));om9_archive::Archive::save(&path,&after,&Limits::default(),&om9_core::transaction::Cancellation::default()).unwrap();let loaded=om9_archive::Archive::open(&path,&Limits::default()).unwrap();std::fs::remove_file(path).unwrap();assert_eq!(loaded,after);
        c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }
}

#[test]
fn join_adjacent_nurbs_surfaces_through_cmd_and_toolbar() {
    use om9_core::top11::Top11Action;
    let limits=Limits::default();
    for via_toolbar in [false,true] {
        let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,3.,&limits).unwrap();
        let mut a=Entity::new("a",Geometry::Surface(surface.clone()),None);a.placement=Transform::translation([10.,0.,0.]);
        let mut b=Entity::new("b",Geometry::Surface(surface.clone()),None);b.placement=Transform::translation([12.,0.,0.]);let ids=[a.id,b.id];
        let mut doc=Document::with_template_tolerance("surface join");doc.entities.insert(a.id,a);doc.entities.insert(b.id,b);let mut c=Controller::new(doc,limits).unwrap();c.select_objects(&ids).unwrap();let before=c.document().clone();
        if via_toolbar {c.top11_invoke(Top11Action::Join).unwrap();}else{c.submit("Join").unwrap();}
        let after=c.document().clone();assert_eq!(after.entities.len(),1);let joined=after.entities.values().next().unwrap();let Geometry::Brep(brep)=&joined.geometry else{panic!("expected BRep")};let m=creation::measure(brep,&limits).unwrap();assert_eq!(m.faces,2);assert_eq!(m.shells,1);assert!((m.area_mm2-12.).abs()<1e-6);assert_eq!(m.naked_edges,6);
        let path=std::env::temp_dir().join(format!("om9-join-surfaces-{}.om9",uuid::Uuid::new_v4()));om9_archive::Archive::save(&path,&after,&limits,&om9_core::transaction::Cancellation::default()).unwrap();let loaded=om9_archive::Archive::open(&path,&limits).unwrap();std::fs::remove_file(path).unwrap();assert_eq!(loaded,after);c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }
}

#[test]
fn join_mixed_surfaces_and_brep_finds_connected_order() {
    use om9_core::top11::Top11Action;
    let limits=Limits::default();let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,3.,&limits).unwrap();
    let mut a=Entity::new("a",Geometry::Surface(surface.clone()),None);a.placement=Transform::translation([10.,0.,0.]);
    let face=om9_kernel::measurement::surface_to_face(&surface,Transform::identity(),creation::DEFAULT_DEFLECTION_MM,&limits).unwrap();
    let mut b=Entity::new("b",Geometry::Brep(face),None);b.placement=Transform::translation([12.,0.,0.]);
    let mut c=Entity::new("c",Geometry::Surface(surface),None);c.placement=Transform::translation([14.,0.,0.]);
    let ids=[a.id,c.id,b.id];let mut doc=Document::with_template_tolerance("mixed join");for e in [a,b,c]{doc.entities.insert(e.id,e);}let mut controller=Controller::new(doc,limits).unwrap();controller.select_objects(&ids).unwrap();let before=controller.document().clone();controller.top11_invoke(Top11Action::Join).unwrap();let after=controller.document().clone();let Geometry::Brep(result)=&after.entities.values().next().unwrap().geometry else{panic!("BRep")};let m=creation::measure(result,&limits).unwrap();assert_eq!(m.faces,3);assert_eq!(m.shells,1);assert_eq!(m.naked_edges,8);assert!((m.area_mm2-18.).abs()<1e-6);controller.undo().unwrap();assert_eq!(controller.document(),&before);controller.redo().unwrap();assert_eq!(controller.document(),&after);
}

#[test]
fn join_surface_disconnected_refuses_atomically() {
    let limits=Limits::default();for toolbar in [false,true] {
        let s=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,3.,&limits).unwrap();let a=Entity::new("a",Geometry::Surface(s.clone()),None);let mut b=Entity::new("b",Geometry::Surface(s),None);b.placement=Transform::translation([10.,0.,0.]);let ids=[a.id,b.id];let mut doc=Document::with_template_tolerance("refuse surface join");doc.entities.insert(a.id,a);doc.entities.insert(b.id,b);let mut c=Controller::new(doc,limits).unwrap();c.select_objects(&ids).unwrap();let before=c.document().clone();let result=if toolbar {c.top11_invoke(om9_core::top11::Top11Action::Join).map(|_|())}else{c.submit("Join").map(|_|())};assert!(result.is_err());assert_eq!(c.document(),&before);assert_eq!(c.coordinator().undo_len(),0);
    }
}

#[test]
fn join_adjacent_curved_rational_surfaces_preserves_geometry() {
    use om9_core::geometry::NurbsSurface;
    use om9_core::top11::Top11Action;
    let limits=Limits::default();
    let surface=|y:f64|NurbsSurface{degree:[2,1],size:[3,2],poles:vec![[0.,y,0.],[0.,y+1.,0.],[1.,y,1.],[1.,y+1.,1.],[2.,y,0.],[2.,y+1.,0.]],weights:vec![1.,1.,0.7,0.7,1.2,1.2],knots:[vec![0.,0.,0.,1.,1.,1.],vec![0.,0.,1.,1.]],domain:[[0.,1.],[0.,1.]],periodic:[false,false],reversed:false};
    for toolbar in [false,true] {
        let a=Entity::new("a",Geometry::Surface(surface(0.)),None);let b=Entity::new("b",Geometry::Surface(surface(1.)),None);let ids=[a.id,b.id];let mut doc=Document::with_template_tolerance("curved join");doc.entities.insert(a.id,a);doc.entities.insert(b.id,b);let mut c=Controller::new(doc,limits).unwrap();c.select_objects(&ids).unwrap();let before=c.document().clone();
        if toolbar {c.top11_invoke(Top11Action::Join).unwrap();}else{c.submit("Join").unwrap();}let after=c.document().clone();let Geometry::Brep(result)=&after.entities.values().next().unwrap().geometry else{panic!("BRep")};let measured=creation::measure(result,&limits).unwrap();assert_eq!(measured.faces,2);assert_eq!(measured.shells,1);assert_eq!(measured.naked_edges,6);
        for y in [0.,1.] {for (u,v) in [(0.2,0.3),(0.5,0.5),(0.8,0.7)] {let p=om9_core::surface_kernel::evaluate(&surface(y),u,v,&limits).unwrap();let origin=[p[0],p[1],p[2]+3.];let mut matched=false;for index in 0..2 {let face=om9_kernel::editing::extract_face(result,index,creation::DEFAULT_DEFLECTION_MM,&limits).unwrap();if let Ok(hit)=om9_kernel::measurement::surface_ray_hit(&Geometry::Brep(face),Transform::identity(),origin,[0.,0.,-1.],p,1e-8,&limits) {if p.iter().zip(hit).all(|(a,b)|(a-b).abs()<1e-7){matched=true;}}}assert!(matched,"{p:?}");}}
        c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }
}

#[test]
fn join_surface_gap_uses_document_tolerance_without_partial_commit() {
    use om9_core::top11::Top11Action;
    let limits=Limits::default();
    for (gap,success,toolbar) in [(0.0005,true,false),(0.0005,true,true),(0.002,false,false),(0.002,false,true)] {
        let s=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,3.,&limits).unwrap();let mut a=Entity::new("a",Geometry::Surface(s.clone()),None);a.placement=Transform::translation([10.,0.,0.]);let mut b=Entity::new("b",Geometry::Surface(s),None);b.placement=Transform::translation([12.+gap,0.,0.]);let ids=[a.id,b.id];let mut d=Document::with_template_tolerance("surface tolerance");d.entities.insert(a.id,a);d.entities.insert(b.id,b);let mut c=Controller::new(d,limits).unwrap();c.select_objects(&ids).unwrap();let before=c.document().clone();
        let result=if toolbar {c.top11_invoke(Top11Action::Join).map(|_|())}else{c.submit("Join").map(|_|())};
        if success {result.unwrap();let after=c.document().clone();let Geometry::Brep(b)=&after.entities.values().next().unwrap().geometry else{panic!("BRep")};let m=creation::measure(b,&limits).unwrap();assert_eq!(m.faces,2);assert_eq!(m.shells,1);assert!((m.area_mm2-(12.+3.*gap)).abs()<1e-6,"area {} gap {gap}",m.area_mm2); let bounds:Vec<_>=(0..2).map(|i|{let f=om9_kernel::editing::extract_face(b,i,creation::DEFAULT_DEFLECTION_MM,&limits).unwrap();om9_kernel::measurement::face_underlying_bounds(&f,&limits).unwrap()}).collect(); assert!(bounds.iter().all(|v|*v==[0.,1.,0.,1.]));c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);}else{assert!(result.is_err());assert_eq!(c.document(),&before);assert_eq!(c.coordinator().undo_len(),0);}
    }
}

#[test]
fn join_reversed_parameter_surfaces_through_cmd_and_toolbar() {
    use om9_core::{surface_kernel::SurfaceIso,top11::Top11Action};
    let limits=Limits::default();let original=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,3.,&limits).unwrap();
    for axis in [SurfaceIso::U,SurfaceIso::V] {for toolbar in [false,true] {
        let reversed=om9_core::surface_ops::reverse_surface(&original,axis,&limits).unwrap();let a=Entity::new("a",Geometry::Surface(original.clone()),None);let mut b=Entity::new("b",Geometry::Surface(reversed),None);b.placement=Transform::translation([2.,0.,0.]);let ids=[a.id,b.id];let mut doc=Document::with_template_tolerance("reverse join");doc.entities.insert(a.id,a);doc.entities.insert(b.id,b);let mut c=Controller::new(doc,limits).unwrap();c.select_objects(&ids).unwrap();let before=c.document().clone();if toolbar {c.top11_invoke(Top11Action::Join).unwrap();}else{c.submit("Join").unwrap();}let after=c.document().clone();let Geometry::Brep(b)=&after.entities.values().next().unwrap().geometry else{panic!("BRep")};let m=creation::measure(b,&limits).unwrap();assert_eq!(m.faces,2);assert_eq!(m.shells,1);assert_eq!(m.naked_edges,6);assert!((m.area_mm2-12.).abs()<1e-6);c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }}
}

#[test]
fn join_cylindrical_nurbs_surfaces_along_closed_edge() {
    use om9_core::top11::Top11Action;
    let limits=Limits::default();let surface=om9_core::surface_kernel::cylinder([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,3.,&limits).unwrap();
    for toolbar in [false,true] {let a=Entity::new("a",Geometry::Surface(surface.clone()),None);let mut b=Entity::new("b",Geometry::Surface(surface.clone()),None);b.placement=Transform::translation([0.,0.,3.]);let ids=[a.id,b.id];let mut doc=Document::with_template_tolerance("cylinder join");doc.entities.insert(a.id,a);doc.entities.insert(b.id,b);let mut c=Controller::new(doc,limits).unwrap();c.select_objects(&ids).unwrap();let before=c.document().clone();if toolbar{c.top11_invoke(Top11Action::Join).unwrap();}else{c.submit("Join").unwrap();}let after=c.document().clone();let Geometry::Brep(result)=&after.entities.values().next().unwrap().geometry else{panic!("BRep")};let m=creation::measure(result,&limits).unwrap();assert_eq!(m.faces,2);assert_eq!(m.shells,1);assert!((m.area_mm2-24.*std::f64::consts::PI).abs()<1e-5,"{}",m.area_mm2);c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);}
}

#[test]
fn join_repeated_periodic_surfaces_at_closed_boundary() {
    use om9_core::{geometry::NurbsSurface,top11::Top11Action};
    let limits=Limits::default();let rows=[[0.,0.],[2.,0.],[2.,2.],[0.,2.],[0.,0.],[2.,0.]];let mut poles=Vec::new();let mut weights=Vec::new();for (i,[x,y]) in rows.into_iter().enumerate(){for z in [0.,2.] {poles.push([x,y,z]);weights.push([1.,0.7,1.2,0.9,1.,0.7][i]);}}
    let surface=NurbsSurface{degree:[2,1],size:[6,2],poles,weights,knots:[vec![-2.,-1.,0.,1.,2.,3.,4.,5.,6.],vec![0.,0.,1.,1.]],domain:[[0.,4.],[0.,1.]],periodic:[true,false],reversed:false};
    let single=om9_kernel::measurement::surface_to_face(&surface,Transform::identity(),creation::DEFAULT_DEFLECTION_MM,&limits).unwrap();let area=creation::measure(&single,&limits).unwrap().area_mm2;
    for toolbar in [false,true] {let a=Entity::new("a",Geometry::Surface(surface.clone()),None);let mut b=Entity::new("b",Geometry::Surface(surface.clone()),None);b.placement=Transform::translation([0.,0.,2.]);let ids=[a.id,b.id];let mut doc=Document::with_template_tolerance("periodic join");doc.entities.insert(a.id,a);doc.entities.insert(b.id,b);let mut c=Controller::new(doc,limits).unwrap();c.select_objects(&ids).unwrap();let before=c.document().clone();if toolbar{c.top11_invoke(Top11Action::Join).unwrap();}else{c.submit("Join").unwrap();}let after=c.document().clone();let Geometry::Brep(b)=&after.entities.values().next().unwrap().geometry else{panic!("BRep")};let m=creation::measure(b,&limits).unwrap();assert_eq!(m.faces,2);assert_eq!(m.shells,1);assert_eq!(m.naked_edges,2);assert_eq!(m.edges,5);assert!((m.area_mm2-2.*area).abs()<1e-6);let path=std::env::temp_dir().join(format!("om9-periodic-surface-{}.om9",uuid::Uuid::new_v4()));om9_archive::Archive::save(&path,&after,&limits,&om9_core::transaction::Cancellation::default()).unwrap();let loaded=om9_archive::Archive::open(&path,&limits).unwrap();std::fs::remove_file(path).unwrap();assert_eq!(loaded,after);for doc in [&after,&loaded] {let Geometry::Brep(joined)=&doc.entities.values().next().unwrap().geometry else{panic!("BRep")};for index in 0..2 {let face=om9_kernel::editing::extract_face(joined,index,creation::DEFAULT_DEFLECTION_MM,&limits).unwrap();assert_eq!(om9_kernel::measurement::face_periodicity(&face,&limits).unwrap(),[true,false]);}}c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);}
}

#[test]
fn join_uncertified_periodic_surface_refuses_without_mutation() {
    use om9_core::geometry::NurbsSurface;
    let limits=Limits::default();let rows=[[0.,0.],[2.,0.],[2.,2.],[0.,2.],[0.,0.],[2.1,0.]];let mut poles=Vec::new();for [x,y] in rows {for z in [0.,2.] {poles.push([x,y,z]);}}
    let surface=NurbsSurface{degree:[2,1],size:[6,2],poles,weights:vec![1.;12],knots:[vec![-2.,-1.,0.,1.,2.,3.,4.,5.,6.],vec![0.,0.,1.,1.]],domain:[[0.,4.],[0.,1.]],periodic:[true,false],reversed:false};let a=Entity::new("invalid periodic",Geometry::Surface(surface.clone()),None);let mut b=Entity::new("other",Geometry::Surface(surface),None);b.placement=Transform::translation([0.,0.,2.]);let ids=[a.id,b.id];let mut doc=Document::with_template_tolerance("invalid periodic");doc.entities.insert(a.id,a);doc.entities.insert(b.id,b);let mut c=Controller::new(doc,limits).unwrap();c.select_objects(&ids).unwrap();let before=c.document().clone();assert!(c.submit("Join").is_err());assert_eq!(c.document(),&before);assert_eq!(c.coordinator().undo_len(),0);
}
