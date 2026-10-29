// SPDX-License-Identifier: LGPL-2.1-or-later
//! BRep mirror through the Top 11 adapter (OCCT kernel feature).
#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
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
    let mut document = Document::new("brep-cutter");
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
