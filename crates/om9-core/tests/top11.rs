// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    budget::Limits,
    curve_build::{polyline, rectangle},
    geometry::{Curve, Geometry, Mesh, NurbsSurface, Point3, Transform},
    model::{Document, Entity},
    top11::{
        GripMode, Top11Action, Top11Context, availability, duplicate_entities,
        duplicate_entities_offset, explode_entities, grip_handles, grip_points, join_entities,
        mirror_entities, move_grip_geometry, rotate_placements, split_by_cutter, split_entity,
        toolbar, translate_placements, trim_by_cutter, trim_entity,
    },
};
use std::collections::BTreeSet;

fn line(a: Point3, b: Point3) -> Curve {
    let length = ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2) + (b[2] - a[2]).powi(2)).sqrt();
    Curve::Line {
        start: a,
        end: b,
        domain: [0.0, length],
    }
}

#[test]
fn grips_ignore_incompatible_objects_only_when_a_compatible_object_exists() {
    let mut doc=Document::new("mixed grips");let point=Entity::new("point",Geometry::Point([0.;3]),None);let curve=Entity::new("line",Geometry::Curve(line([0.;3],[1.,0.,0.])),None);let ids=[point.id,curve.id];doc.entities.insert(ids[0],point);doc.entities.insert(ids[1],curve);
    for mode in [GripMode::Control,GripMode::Edit] {
        let only_point=BTreeSet::from([ids[0]]);assert!(grip_handles(&doc,&only_point,mode,&Limits::default()).is_err());
        let both=BTreeSet::from(ids);let handles=grip_handles(&doc,&both,mode,&Limits::default()).unwrap();assert_eq!(handles.len(),2);assert!(handles.iter().all(|h|h.object==ids[1]));
        let missing=BTreeSet::from([ids[1],om9_core::identity::ObjectId::new()]);assert!(grip_handles(&doc,&missing,mode,&Limits::default()).is_err());
    }
}

#[test]
fn join_availability_rejects_mixed_curve_and_point_selection() {
    let mut doc=Document::new("mixed join");let a=Entity::new("a",Geometry::Curve(line([0.;3],[1.,0.,0.])),None);let b=Entity::new("b",Geometry::Curve(line([1.,0.,0.],[2.,0.,0.])),None);let p=Entity::new("p",Geometry::Point([0.,1.,0.]),None);let ids=[a.id,b.id,p.id];for e in [a,b,p]{doc.entities.insert(e.id,e);}
    let mixed=Top11Context::capture(&doc,&BTreeSet::from(ids),false);assert!(!availability(Top11Action::Join,&mixed).enabled);
    let curves=Top11Context::capture(&doc,&BTreeSet::from([ids[0],ids[1]]),false);assert!(availability(Top11Action::Join,&curves).enabled);
}

#[test]
fn split_certified_periodic_nurbs_preserves_each_active_interval() {
    use om9_core::{curve_ops,geometry::NurbsCurve};
    let limits=Limits::default();
    let mut n=NurbsCurve{degree:2,poles:vec![[0.,0.,0.],[2.,0.,0.],[2.,2.,0.],[0.,2.,0.],[0.,0.,0.],[2.,0.,0.]],
        weights:vec![1.,0.7,1.2,0.9,1.,0.7],knots:vec![-2.,-1.,0.,1.,2.,3.,4.,5.,6.],
        domain:[0.,4.],periodic:true,reversed:false,seam:None};
    for domain in [[0.,4.],[0.3,3.7]] { for reversed in [false,true] {
        n.domain=domain;n.reversed=reversed;let original=Curve::Nurbs(n.clone());
        let (left,right)=curve_ops::split_curve(&original,0.37,&limits).unwrap();
        for (piece,start,end) in [(&left,0.,0.37),(&right,0.37,1.)] {
            for i in 0..=100 {let t=i as f64/100.;
                let a=curve_ops::point_at(&original,start+(end-start)*t,&limits).unwrap();
                let b=curve_ops::point_at(piece,t,&limits).unwrap();
                assert!(a.iter().zip(b).all(|(x,y)|(x-y).abs()<1e-9),"{domain:?} {reversed} {t}: {a:?} {b:?}");
            }
        }
        if domain == [0.,4.] {
            let pieces=om9_core::top11::split_closed_at_fractions(&original,&[0.25,0.7],&limits).unwrap();
            assert_eq!(pieces.len(),2);
            let Curve::Polycurve{segments,..}=&pieces[1] else {panic!("seam-spanning piece")};
            assert_eq!(segments.len(),2);
            for (part,start,end) in [(&pieces[0],0.25,0.7),(&segments[0],0.7,1.),(&segments[1],0.,0.25)] {
                for i in 0..=100 {let t=i as f64/100.;
                    let a=curve_ops::point_at(&original,start+(end-start)*t,&limits).unwrap();
                    let b=curve_ops::point_at(part,t,&limits).unwrap();
                    assert!(a.iter().zip(b).all(|(x,y)|(x-y).abs()<1e-9));
                }
            }
        }
    }}
    n.domain=[0.,4.];n.poles[5][0]+=0.01;assert!(curve_ops::split_curve(&Curve::Nurbs(n),0.37,&limits).is_err());
}

#[test]
fn reseamed_nurbs_evaluation_projection_and_split_agree() {
    use om9_core::{curve_ops,curve_surface_ops,geometry::NurbsCurve};
    let limits=Limits::default();
    let n=NurbsCurve{degree:2,poles:vec![[0.,0.,0.],[2.,0.,0.],[2.,2.,0.],[0.,2.,0.],[0.,0.,0.],[2.,0.,0.]],
        weights:vec![1.,0.7,1.2,0.9,1.,0.7],knots:vec![-2.,-1.,0.,1.,2.,3.,4.,5.,6.],
        domain:[0.,4.],periodic:true,reversed:false,seam:None};
    for reversed in [false,true] {
        let mut n=n.clone();n.reversed=reversed;let original=Curve::Nurbs(n);
        let shifted=curve_ops::set_seam(&original,0.3,1e-9,&limits).unwrap();
        let shifted_again=curve_ops::set_seam(&shifted,0.2,1e-9,&limits).unwrap();
        for (curve,offset) in [(&shifted,0.3),(&shifted_again,0.5)] {
            for i in 0..=100 {let f=i as f64/100.;
                let a=curve_ops::point_at(curve,f,&limits).unwrap();let b=curve_ops::point_at(&original,(offset+f).rem_euclid(1.),&limits).unwrap();
                assert!(a.iter().zip(b).all(|(x,y)|(x-y).abs()<1e-9),"{reversed} {f}: {a:?} {b:?}");
            }
            for f in [0.07,0.3,0.73] {let p=curve_ops::point_at(curve,f,&limits).unwrap();let (found,_,distance)=curve_surface_ops::closest_point_on_curve(curve,p,&limits).unwrap();
                assert!(distance<1e-7 && (found-f).abs()<1e-6,"{found} {f} {distance}");}
            let pieces=om9_core::top11::split_closed_at_fractions(curve,&[0.2,0.65],&limits).unwrap();assert_eq!(pieces.len(),2);
            for piece in &pieces {let ends=curve_ops::curve_endpoints(piece,&limits).unwrap();
                for p in ends {let (_,_,distance)=curve_surface_ops::closest_point_on_curve(curve,p,&limits).unwrap();assert!(distance<1e-7);}
                for i in 0..=40 {let p=curve_ops::point_at(piece,i as f64/40.,&limits).unwrap();
                    let (f,_,distance)=curve_surface_ops::closest_point_on_curve(curve,p,&limits).unwrap();assert!(distance<1e-7,"reversed={reversed} offset={offset} i={i} point={p:?} found={f} distance={distance}");}}
            for i in 0..=40 {let p=curve_ops::point_at(curve,i as f64/40.,&limits).unwrap();
                assert!(pieces.iter().any(|piece|curve_surface_ops::closest_point_on_curve(piece,p,&limits).unwrap().2<1e-7));}
            let total:f64=pieces.iter().map(|p|curve_ops::arc_length(p,&limits).unwrap()).sum();
            assert!((total-curve_ops::arc_length(curve,&limits).unwrap()).abs()<1e-5);
        }
    }
}

#[test]
fn reseamed_closed_nonperiodic_nurbs_is_split_without_refitting() {
    use om9_core::curve_ops;
    let limits=Limits::default();
    for reversed in [false,true] {
        let mut n=om9_core::curve_build::ellipse([0.;3],[1.,0.,0.],[0.,1.,0.],4.,2.,&limits).unwrap();n.reversed=reversed;
        let original=Curve::Nurbs(n.clone());let shifted=curve_ops::set_seam(&original,0.31,1e-9,&limits).unwrap();
        for i in 0..=100 {let f=i as f64/100.;let a=curve_ops::point_at(&shifted,f,&limits).unwrap();
            let b=curve_ops::point_at(&original,(0.31+f).rem_euclid(1.),&limits).unwrap();
            assert!(a.iter().zip(b).all(|(x,y)|(x-y).abs()<1e-8));}
        let pieces=om9_core::top11::split_closed_at_fractions(&shifted,&[0.15,0.72],&limits).unwrap();
        assert_eq!(pieces.len(),2);let total:f64=pieces.iter().map(|c|curve_ops::arc_length(c,&limits).unwrap()).sum();
        assert!((total-curve_ops::arc_length(&original,&limits).unwrap()).abs()<1e-5);
        n.poles[8]=[3.,0.,0.];n.seam=Some(0.3);let invalid=Curve::Nurbs(n);
        assert!(!curve_ops::is_closed(&invalid,1e-9,&limits).unwrap());
        assert!(curve_ops::split_curve(&invalid,0.5,&limits).is_err());
    }
}

fn doc_with(entities: Vec<Entity>) -> (Document, BTreeSet<om9_core::identity::ObjectId>) {
    let mut document = Document::new("top11");
    let mut ids = BTreeSet::new();
    for entity in entities {
        ids.insert(entity.id);
        document.entities.insert(entity.id, entity);
    }
    (document, ids)
}

#[test]
fn mirror_instance_composes_arbitrary_world_plane_and_placement() {
    use om9_core::model::Definition;
    let definition=Definition::new("shared",vec![Entity::new("point",Geometry::Point([1.,2.,3.]),None)]);let definition_id=definition.id;
    let mut instance=Entity::new("instance",Geometry::Instance(definition_id),None);
    instance.placement=Transform([[2.,0.3,0.,4.],[0.,1.5,0.2,-1.],[0.,0.,0.8,2.],[0.,0.,0.,1.]]);
    let id=instance.id;let placement=instance.placement;let (mut doc,ids)=doc_with(vec![instance]);doc.definitions.insert(definition_id,definition);
    let origin=[1.,2.,3.];let normal=[1.,2.,2.];let n=[1./3.,2./3.,2./3.];
    for copy in [false,true] {
        let result=mirror_entities(&doc,&ids,origin,normal,copy,&Limits::default()).unwrap();
        let transformed=if copy {assert!(result.placements.is_empty());result.copies[0].placement} else {assert!(result.copies.is_empty());result.placements[0].1};
        for local in [[0.,0.,0.],[1.,2.,3.],[-2.,0.3,1.]] {
            let before=placement.point(local).unwrap();let signed=(0..3).map(|i|(before[i]-origin[i])*n[i]).sum::<f64>();
            let expected=std::array::from_fn::<_,3,_>(|i|before[i]-2.*signed*n[i]);let actual=transformed.point(local).unwrap();
            assert!(actual.iter().zip(expected).all(|(a,b)|(a-b).abs()<1e-10),"{id:?} {actual:?} {expected:?}");
        }
    }
}

#[test]
fn availability_matches_the_selection_and_the_deferred_ring_rail() {
    let empty = Top11Context {
        running: false,
        selected: 0,
        curves: 0,
        points: 0,
        meshes: 0,
        surfaces: 0,
        breps: 0,
        other: 0,
        mirrorable_other: 0,
    };
    for state in toolbar(&empty) {
        assert!(!state.enabled);
    }
    assert!(!availability(Top11Action::RingRail, &empty).enabled);
    let two_curves = Top11Context {
        running: false,
        selected: 2,
        curves: 2,
        points: 0,
        meshes: 0,
        surfaces: 0,
        breps: 0,
        other: 0,
        mirrorable_other: 0,
    };
    let states = toolbar(&two_curves);
    for action in [
        Top11Action::Duplicate,
        Top11Action::Mirror,
        Top11Action::Move,
        Top11Action::Rotate,
        Top11Action::Explode,
        Top11Action::Join,
    ] {
        let state = states
            .iter()
            .find(|state| state.action == action)
            .copied()
            .unwrap();
        assert!(state.enabled, "{action:?}");
    }
    assert!(!availability(Top11Action::Split, &two_curves).enabled);
    assert!(!availability(Top11Action::Trim, &two_curves).enabled);
    assert!(!availability(Top11Action::RingRail, &two_curves).enabled);
    let running = Top11Context {
        running: true,
        ..two_curves
    };
    assert!(!availability(Top11Action::Duplicate, &running).enabled);
    assert_eq!(Top11Action::ControlPoints.shortcut(), Some("F10"));
    assert_eq!(Top11Action::Duplicate.command(), "Copy");
}

#[test]
fn duplicate_is_in_place_with_fresh_identity_and_same_geometry() {
    let limits = Limits::default();
    let source = Entity::new(
        "line",
        Geometry::Curve(line([0.0; 3], [10.0, 0.0, 0.0])),
        None,
    );
    let (document, ids) = doc_with(vec![source.clone()]);
    let copies = duplicate_entities(&document, &ids, &limits).unwrap();
    assert_eq!(copies.len(), 1);
    assert_ne!(copies[0].id, source.id);
    assert_eq!(copies[0].geometry, source.geometry);
    assert_eq!(copies[0].placement, source.placement);
    assert_eq!(copies[0].generation, 1);
}

#[test]
fn move_and_rotate_change_only_placements() {
    let source = Entity::new(
        "line",
        Geometry::Curve(line([0.0; 3], [10.0, 0.0, 0.0])),
        None,
    );
    let (document, ids) = doc_with(vec![source.clone()]);
    let moved = translate_placements(&document, &ids, [5.0, -1.0, 2.0]).unwrap();
    assert_eq!(moved.len(), 1);
    let placement = moved[0].1;
    assert_eq!(placement.point([0.0; 3]).unwrap(), [5.0, -1.0, 2.0]);
    let rotated = rotate_placements(&document, &ids, [0.0; 3], [0.0, 0.0, 1.0], 90.0).unwrap();
    let point = rotated[0].1.point([10.0, 0.0, 0.0]).unwrap();
    assert!(point[0].abs() < 1e-9 && (point[1] - 10.0).abs() < 1e-9);
    assert_eq!(document.entities[&source.id].placement, source.placement);
}

#[test]
fn mirror_reflects_line_mesh_and_surface_without_mutating_the_source() {
    let limits = Limits::default();
    let line_entity = Entity::new(
        "line",
        Geometry::Curve(line([1.0, 2.0, 0.0], [4.0, 2.0, 0.0])),
        None,
    );
    let mesh = Mesh {
        vertices: vec![[1.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 1.0, 0.0]],
        faces: vec![vec![0, 1, 2]],
        normals: Vec::new(),
        colors: Vec::new(),

        corner_uvs: Vec::new(),
    };
    let mesh_entity = Entity::new("mesh", Geometry::Mesh(mesh), None);
    let surface = om9_core::surface_kernel::plane(
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        2.0,
        2.0,
        &limits,
    )
    .unwrap();
    let surface_entity = Entity::new("surface", Geometry::Surface(surface), None);
    let (document, ids) = doc_with(vec![
        line_entity.clone(),
        mesh_entity.clone(),
        surface_entity.clone(),
    ]);
    let outcome =
        mirror_entities(&document, &ids, [0.0; 3], [1.0, 0.0, 0.0], true, &limits).unwrap();
    assert_eq!(outcome.copies.len(), 3);
    for copy in &outcome.copies {
        match &copy.geometry {
            Geometry::Curve(Curve::Line { start, end, .. }) => {
                assert!((start[0] + 1.0).abs() < 1e-9);
                assert!((end[0] + 4.0).abs() < 1e-9);
            }
            Geometry::Mesh(mesh) => {
                assert!((mesh.vertices[0][0] + 1.0).abs() < 1e-9);
                assert_eq!(mesh.faces[0], vec![0, 2, 1]);
            }
            Geometry::Surface(surface) => {
                assert!((surface.poles[0][0] + 1.0).abs() < 1e-9);
            }
            other => panic!("unexpected mirror geometry {other:?}"),
        }
    }
    assert_eq!(
        document.entities[&line_entity.id].geometry,
        line_entity.geometry
    );
    assert_eq!(
        document.entities[&surface_entity.id].geometry,
        surface_entity.geometry
    );
}

#[test]
fn explode_join_split_and_trim_round_trip_a_polyline() {
    let limits = Limits::default();
    let polyline = polyline(&[[0.0; 3], [10.0, 0.0, 0.0], [10.0, 10.0, 0.0]]).unwrap();
    let entity = Entity::new("polyline", Geometry::Curve(polyline), None);
    let (document, ids) = doc_with(vec![entity.clone()]);
    let pieces = explode_entities(&document, &ids, &limits).unwrap();
    assert_eq!(pieces.len(), 2);
    let joined_ids: BTreeSet<_> = pieces.iter().map(|entity| entity.id).collect();
    let joined_document = doc_with(pieces.clone()).0;
    let joined = join_entities(&joined_document, &joined_ids, 1e-9, &limits).unwrap();
    let split = split_entity(&document, entity.id, 0.5, &limits).unwrap();
    assert_eq!(split.len(), 2);
    let trimmed = trim_entity(&document, entity.id, 0.25, 0.75, &limits).unwrap();
    let Geometry::Curve(curve) = joined.geometry else {
        panic!("join did not return a curve");
    };
    assert!((om9_core::curve_ops::arc_length(&curve, &limits).unwrap() - 20.0).abs() < 1e-9);
    let Geometry::Curve(curve) = trimmed else {
        panic!("trim did not return a curve");
    };
    assert!((om9_core::curve_ops::arc_length(&curve, &limits).unwrap() - 10.0).abs() < 1e-9);
    let plain = Entity::new(
        "line",
        Geometry::Curve(line([0.0; 3], [1.0, 0.0, 0.0])),
        None,
    );
    let (document, ids) = doc_with(vec![plain]);
    assert!(explode_entities(&document, &ids, &limits).is_err());
}

#[test]
fn grips_expose_edit_and_control_points_in_world_space() {
    let limits = Limits::default();
    let square = rectangle([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 10.0, 10.0).unwrap();
    let mut entity = Entity::new("square", Geometry::Curve(square), None);
    entity.placement = om9_core::geometry::Transform::translation([5.0, 0.0, 0.0]);
    let (document, ids) = doc_with(vec![entity]);
    let control = grip_points(&document, &ids, GripMode::Control, &limits).unwrap();
    let edit = grip_points(&document, &ids, GripMode::Edit, &limits).unwrap();
    assert_eq!(control.len(), 8);
    assert_eq!(edit.len(), 8);
    assert_eq!(control[0], [5.0, 0.0, 0.0]);
    let nurbs = om9_core::curve_build::interpolate(
        &[[0.0; 3], [2.0, 3.0, 0.0], [5.0, 2.0, 0.0], [8.0, 0.0, 0.0]],
        3,
        om9_core::curve_build::KnotMode::ChordLength,
        &limits,
    )
    .unwrap();
    let entity = Entity::new("nurbs", Geometry::Curve(Curve::Nurbs(nurbs)), None);
    let (document, ids) = doc_with(vec![entity]);
    assert_eq!(
        grip_points(&document, &ids, GripMode::Control, &limits)
            .unwrap()
            .len(),
        4
    );
    assert_eq!(
        grip_points(&document, &ids, GripMode::Edit, &limits)
            .unwrap()
            .len(),
        4
    );
}

#[test]
fn base_point_duplicate_offsets_the_copy_placement() {
    let limits = Limits::default();
    let source = Entity::new(
        "line",
        Geometry::Curve(line([0.0; 3], [1.0, 0.0, 0.0])),
        None,
    );
    let (document, ids) = doc_with(vec![source]);
    let copies = duplicate_entities_offset(&document, &ids, [5.0, -2.0, 0.0], &limits).unwrap();
    assert_eq!(copies.len(), 1);
    assert_eq!(
        copies[0].placement.point([0.0; 3]).unwrap(),
        [5.0, -2.0, 0.0]
    );
}

#[test]
fn grip_handles_move_control_and_edit_points() {
    let limits = Limits::default();
    let source = Entity::new(
        "line",
        Geometry::Curve(line([0.0; 3], [10.0, 0.0, 0.0])),
        None,
    );
    let (document, ids) = doc_with(vec![source]);
    let handles = grip_handles(&document, &ids, GripMode::Control, &limits).unwrap();
    assert_eq!(handles.len(), 2);
    let geometry = move_grip_geometry(&document, handles[1], [15.0, 2.0, 0.0], &limits).unwrap();
    let Geometry::Curve(Curve::Line { end, .. }) = geometry else {
        panic!("line grip lost the line");
    };
    assert!((end[0] - 15.0).abs() < 1e-9 && (end[1] - 2.0).abs() < 1e-9);

    let points = [[0.0; 3], [2.0, 3.0, 0.0], [5.0, 2.0, 0.0], [8.0, 0.0, 0.0]];
    let nurbs = om9_core::curve_build::interpolate(
        &points,
        3,
        om9_core::curve_build::KnotMode::ChordLength,
        &limits,
    )
    .unwrap();
    let entity = Entity::new("nurbs", Geometry::Curve(Curve::Nurbs(nurbs)), None);
    let (document, ids) = doc_with(vec![entity]);
    let handles = grip_handles(&document, &ids, GripMode::Edit, &limits).unwrap();
    assert_eq!(handles.len(), 4);
    let target = [1.0, 6.0, 0.0];
    let geometry = move_grip_geometry(&document, handles[1], target, &limits).unwrap();
    let Geometry::Curve(curve) = geometry else {
        panic!("edit grip lost the curve");
    };
    let mut nearest = f64::INFINITY;
    for step in 0..=512 {
        let point = om9_core::curve_ops::point_at(&curve, step as f64 / 512.0, &limits).unwrap();
        nearest = nearest.min(
            (0..3)
                .map(|axis| (point[axis] - target[axis]).powi(2))
                .sum::<f64>()
                .sqrt(),
        );
    }
    assert!(nearest < 0.05, "edited curve misses target by {nearest}");
}

#[test]
fn split_and_trim_by_a_cutting_curve_use_the_measured_intersections() {
    let limits = Limits::default();
    let horizontal = line([-10.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    let vertical = line([0.0, -10.0, 0.0], [0.0, 10.0, 0.0]);
    let pieces = split_by_cutter(&horizontal, &vertical, 1e-6, &limits).unwrap();
    assert_eq!(pieces.len(), 2);
    assert!((om9_core::curve_ops::arc_length(&pieces[0], &limits).unwrap() - 10.0).abs() < 1e-6);
    let kept = trim_by_cutter(&horizontal, &vertical, [-5.0, 0.0, 0.0], 1e-6, &limits).unwrap();
    assert_eq!(kept.len(), 1);
    let start = om9_core::curve_ops::curve_endpoints(&kept[0], &limits).unwrap()[0];
    assert!(start[0].abs() < 1e-6, "{start:?}");
}

#[test]
fn trim_removes_the_picked_interval_near_an_uneven_cut() {
    let limits = Limits::default();
    let target = line([0.0, 0.0, 0.0], [100.0, 0.0, 0.0]);
    let cutter = line([10.0, -1.0, 0.0], [10.0, 1.0, 0.0]);
    let kept = trim_by_cutter(&target, &cutter, [11.0, 0.0, 0.0], 1e-6, &limits).unwrap();
    let ends = om9_core::curve_ops::curve_endpoints(&kept[0], &limits).unwrap();
    assert!(
        (ends[0][0] - 0.0).abs() < 1e-6 && (ends[1][0] - 10.0).abs() < 1e-6,
        "{ends:?}"
    );
}

#[test]
fn trim_keeps_adjacent_unremoved_intervals_connected() {
    let limits = Limits::default();
    let target = line([0., 0., 0.], [100., 0., 0.]);
    let pieces =
        om9_core::top11::trim_by_fractions(&target, &[0.25, 0.5, 0.75], [37., 0., 0.], &limits)
            .unwrap();
    assert_eq!(
        pieces.len(),
        2,
        "unremoved adjacent intervals must remain joined"
    );
    let endpoints: Vec<_> = pieces
        .iter()
        .map(|p| om9_core::curve_ops::curve_endpoints(p, &limits).unwrap())
        .collect();
    assert!(
        (endpoints[0][0][0]).abs() < 1e-7 && (endpoints[0][1][0] - 25.).abs() < 1e-7,
        "{endpoints:?}"
    );
    assert!(
        (endpoints[1][0][0] - 50.).abs() < 1e-7 && (endpoints[1][1][0] - 100.).abs() < 1e-7,
        "{endpoints:?}"
    );
}

#[test]
fn normalize_reseamed_nurbs_handles_unrepresentable_endpoint_interval() {
    let limits=Limits::default();let n=om9_core::curve_build::ellipse([0.;3],[1.,0.,0.],[0.,1.,0.],5.,5.,&limits).unwrap();let curve=om9_core::curve_ops::set_seam(&Curve::Nurbs(n),0.2842816612614377,1e-9,&limits).unwrap();
    let near=f64::from_bits(1.0_f64.to_bits()-1);let cuts=om9_core::top11::normalize_cut_fractions(&curve,om9_core::geometry::Transform::identity(),vec![near],0.001,true,&limits).unwrap();assert_eq!(cuts,vec![1.]);
    let distinct=om9_core::top11::normalize_cut_fractions(&curve,om9_core::geometry::Transform::identity(),vec![0.,0.5,1.],0.001,true,&limits).unwrap();assert_eq!(distinct,vec![0.,0.5,1.]);
}

#[test]
fn cut_closure_distinguishes_tiny_open_nurbs_and_polycurves() {
    use om9_core::geometry::NurbsCurve;let limits=Limits::default();
    let nurbs=Curve::Nurbs(NurbsCurve{degree:1,poles:vec![[0.,0.,0.],[1e-10,0.,0.]],weights:vec![1.;2],knots:vec![0.,0.,1.,1.],domain:[0.,1.],periodic:false,reversed:false,seam:None});
    let poly=Curve::Polycurve{segments:vec![line([0.,0.,0.],[5e-11,0.,0.]),line([5e-11,0.,0.],[1e-10,0.,0.])],reversed:false};
    for curve in [&nurbs,&poly] {
        assert!(!om9_core::top11::curve_closed_for_cut(curve,&limits).unwrap());
        let pieces=om9_core::top11::trim_by_fractions(curve,&[0.25,0.75],[1e-11,0.,0.],&limits).unwrap();assert_eq!(pieces.len(),1);
        let ends=om9_core::curve_ops::curve_endpoints(&pieces[0],&limits).unwrap();assert!((ends[0][0]-2.5e-11).abs()<1e-20&&(ends[1][0]-1e-10).abs()<1e-20,"{ends:?}");
        let reverse=om9_core::curve_ops::reverse_curve(curve,&limits).unwrap();assert!(!om9_core::top11::curve_closed_for_cut(&reverse,&limits).unwrap());
        let pieces=om9_core::top11::trim_by_fractions(&reverse,&[0.25,0.75],[1e-11,0.,0.],&limits).unwrap();let ends=om9_core::curve_ops::curve_endpoints(&pieces[0],&limits).unwrap();assert!((ends[0][0]-1e-10).abs()<1e-20&&(ends[1][0]-2.5e-11).abs()<1e-20,"{ends:?}");
    }
    let closed=Curve::Nurbs(NurbsCurve{degree:2,poles:vec![[0.,0.,0.],[1e-10,0.,0.],[0.,0.,0.]],weights:vec![1.;3],knots:vec![0.,0.,0.,1.,1.,1.],domain:[0.,1.],periodic:false,reversed:false,seam:None});
    assert!(om9_core::top11::curve_closed_for_cut(&closed,&limits).unwrap());
    let square=Curve::Polycurve{segments:vec![line([0.,0.,0.],[1e-10,0.,0.]),line([1e-10,0.,0.],[1e-10,1e-10,0.]),line([1e-10,1e-10,0.],[0.,1e-10,0.]),line([0.,1e-10,0.],[0.,0.,0.])],reversed:false};
    assert!(om9_core::top11::curve_closed_for_cut(&square,&limits).unwrap());
    for size in [1.,1e-6,1e-10] { for (gap,expected) in [(1e-10,true),(1e-8,false)] {
        let curve=Curve::Polycurve{segments:vec![line([0.,0.,0.],[size,0.,0.]),line([size,0.,0.],[size,size,0.]),line([size,size,0.],[0.,size*gap,0.])],reversed:false};
        assert_eq!(om9_core::top11::curve_closed_for_cut(&curve,&limits).unwrap(),expected,"size={size} gap={gap}");
    }}
}

#[test]
fn cut_closure_distinguishes_tiny_arcs_and_full_circles() {
    let limits=Limits::default();let tau=std::f64::consts::TAU;
    let circle=|angles|Curve::Circle{center:[0.;3],x:[1.,0.,0.],y:[0.,1.,0.],radius:1e-10,domain:[7.,12.],angle_domain:Some(angles)};
    assert!(!om9_core::top11::curve_closed_for_cut(&circle([0.,std::f64::consts::PI]),&limits).unwrap());
    assert!(om9_core::top11::split_closed_at_fractions(&circle([0.,std::f64::consts::PI]),&[0.25,0.75],&limits).is_err());
    assert!(!om9_core::top11::curve_closed_for_cut(&circle([0.,1e-12]),&limits).unwrap());
    assert!(om9_core::top11::curve_closed_for_cut(&circle([0.,tau]),&limits).unwrap());
    assert!(om9_core::top11::curve_closed_for_cut(&circle([1.,1.+2.*tau]),&limits).unwrap());
}

#[test]
fn trim_tiny_open_arc_is_not_mistaken_for_a_closed_curve() {
    let limits=Limits::default();let curve=Curve::Circle{center:[0.;3],x:[1.,0.,0.],y:[0.,1.,0.],radius:1e-10,domain:[0.,std::f64::consts::PI],angle_domain:None};
    let pick=om9_core::curve_ops::point_at(&curve,0.125,&limits).unwrap();let pieces=om9_core::top11::trim_by_fractions(&curve,&[0.25,0.75],pick,&limits).unwrap();assert_eq!(pieces.len(),1);
    let ends=om9_core::curve_ops::curve_endpoints(&pieces[0],&limits).unwrap();let expected=om9_core::curve_ops::point_at(&curve,1.,&limits).unwrap();assert!(ends.iter().any(|p|(p[0]-expected[0]).hypot(p[1]-expected[1])<1e-20),"{ends:?}");
    let length=om9_core::curve_ops::arc_length(&pieces[0],&limits).unwrap();assert!((length-0.75*std::f64::consts::PI*1e-10).abs()<1e-20,"{length}");
}

#[test]
fn trim_very_short_line_remains_open_and_preserves_nonzero_piece() {
    let limits=Limits::default();let curve=Curve::Line{start:[0.,0.,0.],end:[1e-10,0.,0.],domain:[0.,1.]};
    let pieces=om9_core::top11::trim_by_fractions(&curve,&[0.25,0.75],[1e-11,0.,0.],&limits).unwrap();assert_eq!(pieces.len(),1);
    let ends=om9_core::curve_ops::curve_endpoints(&pieces[0],&limits).unwrap();assert!((ends[0][0]-2.5e-11).abs()<1e-20&&(ends[1][0]-1e-10).abs()<1e-20,"{ends:?}");
}

#[test]
fn trim_closed_circle_respects_a_cutter_at_the_parameter_seam() {
    let limits=Limits::default();let curve=Curve::Circle{center:[0.;3],x:[1.,0.,0.],y:[0.,1.,0.],radius:5.,domain:[0.,std::f64::consts::TAU],angle_domain:None};
    for seam in [0.0,1.0] {for (pick_fraction,remaining,endpoint) in [(0.125,0.75,[0.,5.,0.]),(0.75,0.5,[-5.,0.,0.])] {
        let pick=om9_core::curve_ops::point_at(&curve,pick_fraction,&limits).unwrap();let pieces=om9_core::top11::trim_by_fractions(&curve,&[seam,0.25,0.5],pick,&limits).unwrap();assert_eq!(pieces.len(),1);
        assert!((om9_core::curve_ops::arc_length(&pieces[0],&limits).unwrap()-remaining*10.*std::f64::consts::PI).abs()<1e-7);
        let ends=om9_core::curve_ops::curve_endpoints(&pieces[0],&limits).unwrap();for p in [[5.,0.,0.],endpoint] {assert!(ends.iter().any(|q|(q[0]-p[0]).hypot(q[1]-p[1])<1e-8),"{ends:?}");}
    }}
    let pick=om9_core::curve_ops::point_at(&curve,0.375,&limits).unwrap();let pieces=om9_core::top11::trim_by_fractions(&curve,&[0.,0.25,0.5],pick,&limits).unwrap();assert_eq!(pieces.len(),1);
    assert!((om9_core::curve_ops::arc_length(&pieces[0],&limits).unwrap()-7.5*std::f64::consts::PI).abs()<1e-7);
    let ends=om9_core::curve_ops::curve_endpoints(&pieces[0],&limits).unwrap();for p in [[0.,5.],[-5.,0.]]{assert!(ends.iter().any(|q|(q[0]-p[0]).hypot(q[1]-p[1])<1e-8),"{ends:?}");}
}

#[test]
fn trim_by_polycurve_cutter_preserves_a_closed_curve_seam_cut() {
    let limits=Limits::default();let curve=Curve::Circle{center:[0.;3],x:[1.,0.,0.],y:[0.,1.,0.],radius:5.,domain:[0.,std::f64::consts::TAU],angle_domain:None};
    let cutter=Curve::Polycurve{segments:vec![line([6.,0.,0.],[0.,0.,0.]),line([0.,0.,0.],[0.,6.,0.]),line([0.,6.,0.],[0.,0.,0.]),line([0.,0.,0.],[-6.,0.,0.])],reversed:false};
    let pick=om9_core::curve_ops::point_at(&curve,0.125,&limits).unwrap();let pieces=om9_core::top11::trim_by_cutter(&curve,&cutter,pick,1e-6,&limits).unwrap();assert_eq!(pieces.len(),1);
    assert!((om9_core::curve_ops::arc_length(&pieces[0],&limits).unwrap()-7.5*std::f64::consts::PI).abs()<1e-7);
    let ends=om9_core::curve_ops::curve_endpoints(&pieces[0],&limits).unwrap();for p in [[5.,0.],[0.,5.]] {assert!(ends.iter().any(|q|(q[0]-p[0]).hypot(q[1]-p[1])<1e-8),"{ends:?}");}
}

#[test]
fn trim_closed_circle_removes_the_whole_interval_across_its_seam() {
    let limits=Limits::default();let curve=Curve::Circle{center:[0.;3],x:[1.,0.,0.],y:[0.,1.,0.],radius:5.,domain:[0.,std::f64::consts::TAU],angle_domain:None};
    for pick_fraction in [0.125,0.875] {
        let pick=om9_core::curve_ops::point_at(&curve,pick_fraction,&limits).unwrap();
        let pieces=om9_core::top11::trim_by_fractions(&curve,&[0.25,0.75],pick,&limits).unwrap();assert_eq!(pieces.len(),1);
        assert!((om9_core::curve_ops::arc_length(&pieces[0],&limits).unwrap()-5.*std::f64::consts::PI).abs()<1e-6);
        let ends=om9_core::curve_ops::curve_endpoints(&pieces[0],&limits).unwrap();assert!(ends.iter().any(|p|p[0].abs()<1e-6&&(p[1]-5.).abs()<1e-6));assert!(ends.iter().any(|p|p[0].abs()<1e-6&&(p[1]+5.).abs()<1e-6));
    }
}

#[test]
fn trim_closed_circle_keeps_the_complement_across_the_seam() {
    let limits = Limits::default();
    let curve = Curve::Circle {
        center: [0.; 3],
        x: [1., 0., 0.],
        y: [0., 1., 0.],
        radius: 5.,
        domain: [0., std::f64::consts::TAU],
        angle_domain: None,
    };
    let pick = om9_core::curve_ops::point_at(&curve, 0.375, &limits).unwrap();
    let pieces =
        om9_core::top11::trim_by_fractions(&curve, &[0.25, 0.5, 0.75], pick, &limits).unwrap();
    assert_eq!(pieces.len(), 1);
    assert!(
        (om9_core::curve_ops::arc_length(&pieces[0], &limits).unwrap()
            - 7.5 * std::f64::consts::PI)
            .abs()
            < 1e-6
    );
    let ends = om9_core::curve_ops::curve_endpoints(&pieces[0], &limits).unwrap();
    assert!(
        (ends[0][0] + 5.).abs() < 1e-6 && ends[0][1].abs() < 1e-6,
        "{ends:?}"
    );
    assert!(
        ends[1][0].abs() < 1e-6 && (ends[1][1] - 5.).abs() < 1e-6,
        "{ends:?}"
    );
}

#[test]
fn mirror_without_copy_keeps_the_placement_and_returns_local_geometry() {
    let limits = Limits::default();
    let mut source = Entity::new(
        "line",
        Geometry::Curve(line([0.0; 3], [1.0, 0.0, 0.0])),
        None,
    );
    source.placement = om9_core::geometry::Transform::translation([5.0, 0.0, 0.0]);
    let (document, ids) = doc_with(vec![source.clone()]);
    let outcome =
        mirror_entities(&document, &ids, [0.0; 3], [1.0, 0.0, 0.0], false, &limits).unwrap();
    assert!(outcome.copies.is_empty());
    assert_eq!(outcome.replacements.len(), 1);
    let (id, Geometry::Curve(Curve::Line { start, end, .. })) = &outcome.replacements[0] else {
        panic!("mirror did not replace the line");
    };
    assert_eq!(*id, source.id);
    assert!((start[0] + 10.0).abs() < 1e-9, "{start:?}");
    assert!((end[0] + 11.0).abs() < 1e-9, "{end:?}");
}

#[test]
fn circle_and_polycurve_grips_are_editable() {
    let limits = Limits::default();
    let circle = Curve::Circle {
        center: [0.0; 3],
        x: [1.0, 0.0, 0.0],
        y: [0.0, 1.0, 0.0],
        radius: 2.0,
        domain: [0.0, std::f64::consts::TAU],
        angle_domain: None,
    };
    let entity = Entity::new("circle", Geometry::Curve(circle), None);
    let (document, ids) = doc_with(vec![entity]);
    let handles = grip_handles(&document, &ids, GripMode::Control, &limits).unwrap();
    assert_eq!(handles.len(), 5);
    let geometry = move_grip_geometry(&document, handles[1], [5.0, 0.0, 0.0], &limits).unwrap();
    let Geometry::Curve(Curve::Circle { radius, center, .. }) = geometry else {
        panic!("circle grip lost the circle");
    };
    assert!((radius - 5.0).abs() < 1e-9);
    assert_eq!(center, [0.0; 3]);
    println!("circle_grip_radius={radius:.12}");

    let polycurve = Curve::Polycurve {
        segments: vec![
            line([0.0; 3], [5.0, 0.0, 0.0]),
            line([5.0, 0.0, 0.0], [5.0, 5.0, 0.0]),
        ],
        reversed: false,
    };
    let entity = Entity::new("poly", Geometry::Curve(polycurve), None);
    let (mut document, ids) = doc_with(vec![entity]);
    document.tolerance = Some(om9_core::history::Tolerance::om9_template());
    let handles = grip_handles(&document, &ids, GripMode::Control, &limits).unwrap();
    assert_eq!(handles.len(), 4);
    let geometry = move_grip_geometry(&document, handles[1], [5.0, 1.0, 0.0], &limits).unwrap();
    let Geometry::Curve(Curve::Polycurve { segments, .. }) = geometry else {
        panic!("polycurve grip lost the polycurve");
    };
    let Curve::Line { end, .. } = &segments[0] else {
        panic!("first segment is not a line");
    };
    assert!((end[1] - 1.0).abs() < 1e-9);
}

#[test]
fn surface_control_and_edit_grips_move_the_surface_without_reparameterising() {
    let limits = Limits::default();
    let expected = NurbsSurface {
        degree: [1, 1],
        size: [2, 2],
        poles: vec![
            [0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
        ],
        weights: vec![1.0; 4],
        knots: [vec![0.0, 0.0, 1.0, 1.0], vec![0.0, 0.0, 1.0, 1.0]],
        domain: [[0.0, 1.0], [0.0, 1.0]],
        periodic: [false, false],
        reversed: false,
    };
    let entity = Entity::new("surface", Geometry::Surface(expected.clone()), None);
    let (document, ids) = doc_with(vec![entity]);
    let handles = grip_handles(&document, &ids, GripMode::Control, &limits).unwrap();
    assert_eq!(handles.len(), 4);
    let geometry = move_grip_geometry(&document, handles[3], [2.0, 2.0, 1.0], &limits).unwrap();
    let Geometry::Surface(edited) = geometry else {
        panic!("surface grip lost the surface");
    };
    assert_eq!(edited.poles[3], [2.0, 2.0, 1.0]);
    let edit_handles = grip_handles(&document, &ids, GripMode::Edit, &limits).unwrap();
    assert_eq!(edit_handles.len(), 4);
    let geometry =
        move_grip_geometry(&document, edit_handles[3], [3.0, 3.0, 2.0], &limits).unwrap();
    let Geometry::Surface(edited) = geometry else {
        panic!("surface edit grip lost the surface");
    };
    let moved = om9_core::surface_kernel::evaluate(&edited, 1.0, 1.0, &limits).unwrap();
    assert!((moved[0] - 3.0).abs() < 1e-9, "{moved:?}");
    assert!((moved[2] - 2.0).abs() < 1e-9, "{moved:?}");
    println!("surface_control_grips=4");
    println!("surface_edit_grips=4");
}

#[test]
fn line_intersections_remain_stable_for_short_and_nearly_parallel_segments() {
    let limits=Limits::default();let line=|a,b|Curve::Line{start:a,end:b,domain:[0.,1.]};
    let cases=[
        (line([0.,0.,0.],[1e-8,0.,0.]),line([3.71234e-9,-4e-9,0.],[3.71234e-9,6e-9,0.]),1e-13,0.371234),
        (line([0.,0.,0.],[1e10,0.,0.]),line([0.,-0.74,0.],[1e10,1.26,0.]),1e-4,0.37),
    ];
    for (target,cutter,tolerance,expected) in cases {
        let hits=om9_core::curve_surface_ops::curve_curve_intersections(&target,&cutter,tolerance,&limits).unwrap();
        assert_eq!(hits.len(),1,"{hits:?}");assert!((hits[0].first_fraction-expected).abs()<1e-10,"{hits:?}");assert!(hits[0].distance<=tolerance);
    }
    let target=line([0.,0.,0.],[1e10,0.,0.]);let skew=line([0.,-0.74,0.01],[1e10,1.26,0.01]);
    assert!(om9_core::curve_surface_ops::curve_curve_intersections(&target,&skew,0.001,&limits).unwrap().is_empty());
    let hits=om9_core::curve_surface_ops::curve_curve_intersections(&target,&skew,0.02,&limits).unwrap();
    assert_eq!(hits.len(),1);assert!((hits[0].distance-0.01).abs()<1e-10);
    let swapped=om9_core::curve_surface_ops::curve_curve_intersections(&skew,&target,0.02,&limits).unwrap();
    assert_eq!(swapped.len(),1);assert!((swapped[0].first_fraction-hits[0].second_fraction).abs()<1e-10);
}

#[test]
fn cutter_fraction_dedup_uses_world_distance_along_curve() {
    let limits=Limits::default();let line=Curve::Line{start:[0.,0.,0.],end:[10.,0.,0.],domain:[0.,1.]};
    let scale=Transform::scale([1000.,1.,1.]).unwrap();
    let cuts=om9_core::top11::normalize_cut_fractions(&line,scale,vec![0.5,0.50000005,0.5000002],0.001,false,&limits).unwrap();
    assert_eq!(cuts,vec![0.5,0.5000002]);
    let endpoints=om9_core::top11::normalize_cut_fractions(&line,scale,vec![0.00000005,0.5,0.99999995],0.001,true,&limits).unwrap();
    assert_eq!(endpoints,vec![0.,0.5,1.]);
    let retraced=Curve::Polycurve{segments:vec![
        Curve::Line{start:[0.,0.,0.],end:[1.,0.,0.],domain:[0.,1.]},
        Curve::Line{start:[1.,0.,0.],end:[0.,0.,0.],domain:[0.,1.]},
        Curve::Line{start:[0.,0.,0.],end:[1.,0.,0.],domain:[0.,1.]},
    ],reversed:false};
    let cuts=om9_core::top11::normalize_cut_fractions(&retraced,Transform::identity(),vec![1./6.,5./6.],0.001,false,&limits).unwrap();
    assert_eq!(cuts.len(),2,"same world point at distant curve parameters must remain two cuts");
}

#[test]
fn reversed_line_and_arc_keep_domains_and_reverse_fractional_points() {
    let curves=[
        Curve::Line{start:[1.,2.,0.],end:[4.,5.,0.],domain:[2.,5.]},
        Curve::Circle{center:[1.,2.,0.],x:[1.,0.,0.],y:[0.,1.,0.],radius:2.,domain:[7.,12.],angle_domain:Some([-0.8,2.])},
    ];
    let limits=Limits::default();
    for original in curves {
        let flipped=om9_core::curve_ops::reverse_curve(&original,&limits).unwrap();
        let domain=|curve:&Curve|match curve {Curve::Line{domain,..}|Curve::Circle{domain,..}=>*domain,_=>panic!("curve")};
        assert_eq!(domain(&flipped),domain(&original));
        for f in [0.,0.1,0.37,0.9,1.] {
            let actual=om9_core::curve_ops::point_at(&flipped,f,&limits).unwrap();
            let expected=om9_core::curve_ops::point_at(&original,1.-f,&limits).unwrap();
            assert!(actual.iter().zip(expected).all(|(a,b)|(a-b).abs()<1e-12),"{f}: {actual:?} {expected:?}");
        }
    }
}

#[test]
fn circle_bake_does_not_approximate_nearly_elliptic_transforms() {
    let curve=Curve::Circle {center:[0.;3],x:[1.,0.,0.],y:[0.,1.,0.],radius:1.,domain:[2.,5.],angle_domain:Some([0.,std::f64::consts::TAU])};
    let transform=Transform::scale([1.,1.+1e-10,1.]).unwrap();
    let actual=om9_core::top11::bake_curve(&curve,transform).unwrap();
    assert!(matches!(actual,Curve::Nurbs(_)),"anisotropy must not be rounded into a circle");
    let p=om9_core::curve_ops::point_at(&actual,0.25,&Limits::default()).unwrap();
    assert!((p[1]-(1.+1e-10)).abs()<1e-14,"{p:?}");
}

#[test]
fn non_rigid_mirror_bakes_a_line_and_turns_a_circle_into_an_ellipse() {
    let limits = Limits::default();
    let mut line_entity = Entity::new(
        "line",
        Geometry::Curve(line([0.0; 3], [1.0, 0.0, 0.0])),
        None,
    );
    line_entity.placement = Transform::scale([2.0, 1.0, 1.0]).unwrap();
    let (document, ids) = doc_with(vec![line_entity]);
    let outcome =
        mirror_entities(&document, &ids, [0.0; 3], [1.0, 0.0, 0.0], true, &limits).unwrap();
    let Geometry::Curve(Curve::Line { start, end, .. }) = &outcome.copies[0].geometry else {
        panic!("non-rigid mirror lost the line");
    };
    assert!((start[0] + 0.0).abs() < 1e-9);
    assert!((end[0] + 2.0).abs() < 1e-9);

    let circle_entity = Entity::new(
        "circle",
        Geometry::Curve(Curve::Circle {
            center: [0.0; 3],
            x: [1.0, 0.0, 0.0],
            y: [0.0, 1.0, 0.0],
            radius: 2.0,
            domain: [0.0, std::f64::consts::TAU],
            angle_domain: None,
        }),
        None,
    );
    let mut circle_entity = circle_entity;
    circle_entity.placement = Transform::scale([2.0, 1.0, 1.0]).unwrap();
    let (document, ids) = doc_with(vec![circle_entity]);
    let outcome =
        mirror_entities(&document, &ids, [0.0; 3], [0.0, 1.0, 0.0], true, &limits).unwrap();
    assert!(matches!(
        outcome.copies[0].geometry,
        Geometry::Curve(Curve::Nurbs(_))
    ));
}

#[test]
fn non_rigid_mirror_transforms_mesh_vertices_normals_and_winding() {
    let limits = Limits::default();
    let mut mesh_entity = Entity::new(
        "mesh",
        Geometry::Mesh(Mesh {
            vertices: vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            faces: vec![vec![0, 1, 2]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            colors: Vec::new(),
            corner_uvs: Vec::new(),
        }),
        None,
    );
    mesh_entity.placement = Transform::scale([2.0, 1.0, 1.0]).unwrap();
    let (document, ids) = doc_with(vec![mesh_entity]);
    let outcome =
        mirror_entities(&document, &ids, [0.0; 3], [1.0, 0.0, 0.0], true, &limits).unwrap();
    let Geometry::Mesh(mesh) = &outcome.copies[0].geometry else {
        panic!("non-rigid mirror lost the mesh");
    };
    assert!(
        (mesh.vertices[1][0] + 2.0).abs() < 1e-9,
        "{:?}",
        mesh.vertices
    );
    assert_eq!(mesh.faces, vec![vec![0, 2, 1]]);
    assert!((mesh.normals[0][2] - 1.0).abs() < 1e-9);
}

#[test]
fn surface_edit_grips_keep_a_reversed_or_periodic_surface_record() {
    let limits = Limits::default();
    let mut surface = om9_core::surface_kernel::cylinder(
        [0.0; 3],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        5.0,
        10.0,
        &limits,
    )
    .unwrap();
    surface.reversed = true;
    let entity = Entity::new("cylinder", Geometry::Surface(surface.clone()), None);
    let (document, ids) = doc_with(vec![entity]);
    let handles = grip_handles(&document, &ids, GripMode::Edit, &limits).unwrap();
    let geometry = move_grip_geometry(&document, handles[10], handles[10].point, &limits).unwrap();
    let Geometry::Surface(edited) = geometry else {
        panic!("surface edit lost the surface");
    };
    assert!(edited.reversed);
    assert_eq!(edited.poles.len(), surface.poles.len());

    surface.periodic = [true, false];
    let entity = Entity::new("periodic", Geometry::Surface(surface), None);
    let (document, ids) = doc_with(vec![entity]);
    let handles = grip_handles(&document, &ids, GripMode::Edit, &limits).unwrap();
    let target = [
        handles[0].point[0],
        handles[0].point[1],
        handles[0].point[2] + 1.0,
    ];
    let Geometry::Surface(edited) =
        move_grip_geometry(&document, handles[0], target, &limits).unwrap()
    else {
        panic!("periodic surface edit lost the surface");
    };
    assert!(edited.periodic[0]);
}

fn large_surface(size: usize) -> NurbsSurface {
    let degree = 3usize;
    let mut knots = vec![0.0; degree + 1];
    let spans = size - degree;
    for step in 1..spans {
        knots.push(step as f64 / spans as f64);
    }
    knots.extend(std::iter::repeat_n(1.0, degree + 1));
    let mut poles = Vec::with_capacity(size * size);
    for i in 0..size {
        for j in 0..size {
            let u = i as f64 / (size - 1) as f64;
            let v = j as f64 / (size - 1) as f64;
            poles.push([
                u * 100.0,
                v * 100.0,
                (u * 8.0).sin() * (v * 8.0).cos() * 5.0,
            ]);
        }
    }
    NurbsSurface {
        degree: [degree, degree],
        size: [size, size],
        weights: vec![1.0; size * size],
        poles,
        knots: [knots.clone(), knots],
        domain: [[0.0, 1.0], [0.0, 1.0]],
        periodic: [false, false],
        reversed: false,
    }
}

#[test]
fn surface_edit_grid_is_fast_on_a_large_surface() {
    let limits = Limits::default();
    let surface = large_surface(96);
    let started = std::time::Instant::now();
    let points = om9_core::surface_kernel::edit_points(&surface, &limits).unwrap();
    let grid_ms = started.elapsed().as_secs_f64() * 1000.0;
    assert_eq!(points.len(), 96 * 96);
    assert!(grid_ms < 1500.0, "edit-point grid took {grid_ms:.3} ms");
    let entity = Entity::new("large", Geometry::Surface(surface), None);
    let (document, ids) = doc_with(vec![entity]);
    let started = std::time::Instant::now();
    let handles = grip_handles(&document, &ids, GripMode::Edit, &limits).unwrap();
    let handles_ms = started.elapsed().as_secs_f64() * 1000.0;
    assert_eq!(handles.len(), 96 * 96);
    assert!(handles_ms < 1500.0, "edit handles took {handles_ms:.3} ms");
    let started = std::time::Instant::now();
    let target = [
        handles[48 * 96 + 48].point[0] + 1.0,
        handles[48 * 96 + 48].point[1],
        handles[48 * 96 + 48].point[2],
    ];
    move_grip_geometry(&document, handles[48 * 96 + 48], target, &limits).unwrap();
    let move_ms = started.elapsed().as_secs_f64() * 1000.0;
    assert!(move_ms < 500.0, "one edit move took {move_ms:.3} ms");
    println!("surface_edit_points_96_ms={grid_ms:.3}");
    println!("surface_edit_handles_96_ms={handles_ms:.3}");
    println!("surface_edit_move_96_ms={move_ms:.3}");
}

#[test]
fn surface_edit_grips_are_capped_for_large_grids() {
    let limits = Limits::default();
    let entity = Entity::new("huge", Geometry::Surface(large_surface(101)), None);
    let (document, ids) = doc_with(vec![entity]);
    let error = grip_handles(&document, &ids, GripMode::Edit, &limits).unwrap_err();
    assert!(error.to_string().contains("10,000"), "{error}");
    assert!(grip_handles(&document, &ids, GripMode::Control, &limits).is_ok());
}

#[test]
fn polycurve_joint_tolerance_is_measured_in_world_coordinates() {
    use om9_core::geometry::Transform;
    let limits = Limits::default();
    for mode in [GripMode::Edit, GripMode::Control] {
        for (gap, joined) in [([0.,0.005,0.], true), ([0.0005,0.,0.], false), ([0.00005,0.,0.], true)] {
            let curve = Curve::Polycurve { segments: vec![line([0.,0.,0.], [5.,0.,0.]), line([5. + gap[0], gap[1], gap[2]], [5.,5.,0.])], reversed: false };
            let mut entity = Entity::new("placed", Geometry::Curve(curve), None);
            entity.placement = Transform::scale([10.,0.1,1.]).unwrap();
            let (mut document, ids) = doc_with(vec![entity]);
            document.tolerance = Some(om9_core::history::Tolerance::om9_template());
            let handle = grip_handles(&document, &ids, mode, &limits).unwrap()[1];
            let target = [60.,0.1,2.];
            let geometry = move_grip_geometry(&document, handle, target, &limits).unwrap();
            let Geometry::Curve(Curve::Polycurve { segments, .. }) = geometry else { panic!("lost polycurve") };
            let first = om9_core::curve_ops::curve_endpoints(&segments[0], &limits).unwrap()[1];
            let second = om9_core::curve_ops::curve_endpoints(&segments[1], &limits).unwrap()[0];
            assert!((0..3).all(|i| (first[i] - [6.,1.,2.][i]).abs() < 1e-9));
            if joined { assert!((0..3).all(|i| (second[i] - first[i]).abs() < 1e-9)); }
            else { assert!((second[0] - (5. + gap[0])).abs() < 1e-9 && second[1] == gap[1]); }
            document.tolerance = None;
            assert!(move_grip_geometry(&document, handle, target, &limits).is_err());
        }
    }
}

#[test]
fn smooth_control_fusion_rejects_output_over_point_budget() {
    let limits = Limits { max_points: 2, ..Limits::default() };
    let curve = Curve::Polycurve { segments: vec![line([0.,0.,0.], [5.,0.,0.]), line([5.,0.,0.], [10.,0.,0.])], reversed: false };
    let entity = Entity::new("smooth", Geometry::Curve(curve), None);
    let (mut document, ids) = doc_with(vec![entity]);
    document.tolerance = Some(om9_core::history::Tolerance::om9_template());
    let before = document.clone();
    let handle = grip_handles(&document, &ids, GripMode::Control, &limits).unwrap()[1];
    let error = move_grip_geometry(&document, handle, [5.,1.,0.], &limits).unwrap_err();
    assert!(matches!(error, om9_core::Error::Budget(_)), "{error:?}");
    assert_eq!(document, before);
}

#[test]
fn join_loop_tail_accepts_all_edge_orders_and_rejects_impossible_graphs() {
    let limits=Limits::default();
    let edges=[([0.,0.,0.],[1.,0.,0.]),([1.,0.,0.],[2.,0.,0.]),([1.,0.,0.],[1.,1.,0.]),([1.,1.,0.],[0.,1.,0.]),([0.,1.,0.],[1.,0.,0.])];
    for a in 0..5 { for b in 0..5 { for c in 0..5 { for d in 0..5 { for e in 0..5 {
        let order=[a,b,c,d,e]; if order.iter().copied().collect::<BTreeSet<_>>().len()!=5 {continue;}
        let mut entities: Vec<_> = (0..5).map(|_| Entity::new("edge",Geometry::Curve(line([0.;3],[1.,0.,0.])),None)).collect(); entities.sort_by_key(|entity| entity.id);
        for (entity,index) in entities.iter_mut().zip(order) { entity.geometry=Geometry::Curve(line(edges[index].0,edges[index].1)); }
        let (document,ids)=doc_with(entities); let joined=join_entities(&document,&ids,1e-9,&limits).unwrap();
        let Geometry::Curve(Curve::Polycurve {segments,..})=joined.geometry else {panic!("polycurve")}; assert_eq!(segments.len(),5);
        for pair in segments.windows(2) { assert_eq!(om9_core::curve_ops::curve_endpoints(&pair[0],&limits).unwrap()[1],om9_core::curve_ops::curve_endpoints(&pair[1],&limits).unwrap()[0]); }
    }}}}}
    let t=[line([0.,0.,0.],[1.,0.,0.]),line([1.,0.,0.],[2.,0.,0.]),line([1.,0.,0.],[1.,1.,0.])];
    let entities=t.into_iter().map(|curve|Entity::new("edge",Geometry::Curve(curve),None)).collect(); let (document,ids)=doc_with(entities);
    assert!(join_entities(&document,&ids,1e-9,&limits).is_err());
    let loops=[line([0.,0.,0.],[1.,0.,0.]),line([1.,0.,0.],[0.,0.,0.]),line([3.,0.,0.],[4.,0.,0.]),line([4.,0.,0.],[3.,0.,0.])];
    let entities=loops.into_iter().map(|curve|Entity::new("edge",Geometry::Curve(curve),None)).collect(); let (document,ids)=doc_with(entities);
    assert!(join_entities(&document,&ids,1e-9,&limits).is_err());
}

#[test]
fn join_trail_scratch_obeys_node_budget() {
    let entities=(0..5).map(|i|Entity::new("edge",Geometry::Curve(line([i as f64,0.,0.],[(i+1) as f64,0.,0.])),None)).collect();
    let (document,ids)=doc_with(entities); let limits=Limits { max_nodes: 9, ..Limits::default() };
    assert!(matches!(join_entities(&document,&ids,1e-9,&limits),Err(om9_core::Error::Budget(_))));
}

#[test]
fn join_large_spread_chain_preserves_every_segment() {
    let count=2000usize;
    let entities=(0..count).map(|i|Entity::new("edge",Geometry::Curve(line([i as f64,0.,0.],[(i+1) as f64,0.,0.])),None)).collect();
    let (document,ids)=doc_with(entities); let start=std::time::Instant::now();
    let joined=join_entities(&document,&ids,0.001,&Limits::default()).unwrap();
    let elapsed=start.elapsed(); let Geometry::Curve(Curve::Polycurve {segments,..})=joined.geometry else {panic!("polycurve")};
    assert_eq!(segments.len(),count);
    for pair in segments.windows(2) { assert_eq!(om9_core::curve_ops::curve_endpoints(&pair[0],&Limits::default()).unwrap()[1],om9_core::curve_ops::curve_endpoints(&pair[1],&Limits::default()).unwrap()[0]); }
    println!("join_spread_segments={count} elapsed_ms={:.3}",elapsed.as_secs_f64()*1000.);
}

#[test]
fn join_sweep_respects_tolerance_on_all_axes_and_tiny_separation() {
    let limits=Limits::default();
    for axis in 0..3 {
        for gap in [0.,0.0005,0.0015] {
            let mut a=[0.;3]; a[axis]=1.; let mut b=a; b[(axis+1)%3]=gap; let mut c=b; c[axis]=2.;
            let entities=vec![Entity::new("first",Geometry::Curve(line([0.;3],a)),None),Entity::new("second",Geometry::Curve(line(b,c)),None)];
            let (document,ids)=doc_with(entities); let result=join_entities(&document,&ids,0.001,&limits);
            assert_eq!(result.is_ok(),gap<=0.001,"axis {axis} gap {gap}");
        }
    }
    let tiny = |x,y| Curve::Line { start:[x,0.,0.],end:[y,0.,0.],domain:[0.,1.] };
    let entities=vec![Entity::new("a",Geometry::Curve(tiny(0.,1e-200)),None),Entity::new("b",Geometry::Curve(tiny(2e-200,3e-200)),None)];
    let (document,ids)=doc_with(entities); assert!(join_entities(&document,&ids,0.,&limits).is_err());
}
