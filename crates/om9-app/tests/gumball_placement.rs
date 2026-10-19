// SPDX-License-Identifier: LGPL-2.1-or-later
//! Gumball placement: relocating the origin and aligning the axes to the selected object.
use om9_app::session::Controller;
use om9_core::{
    budget::Limits,
    geometry::{Geometry, Transform},
    gumball::{self, Axis},
    model::{Document, Entity},
};

fn controller_with_points() -> (Controller, Vec<om9_core::identity::ObjectId>) {
    let mut controller = Controller::new(Document::new("placement"), Limits::default()).expect("controller");
    for x in ["0,0,0", "10,0,0"] {
        controller.submit("Point").expect("Point");
        controller.submit(x).expect("coordinates");
    }
    let ids = controller.document().entities.keys().copied().collect();
    (controller, ids)
}

fn world(controller: &Controller, id: om9_core::identity::ObjectId) -> [f64; 3] {
    let entity = &controller.document().entities[&id];
    let Geometry::Point(point) = entity.geometry else {
        panic!("expected a point");
    };
    entity.placement.point(point).expect("placed point")
}

fn ends(controller: &Controller, ids: &[om9_core::identity::ObjectId]) -> ([f64; 3], [f64; 3]) {
    let mut points: Vec<[f64; 3]> = ids.iter().map(|id| world(controller, *id)).collect();
    points.sort_by(|left, right| left[0].total_cmp(&right[0]));
    (points[0], points[1])
}

#[test]
fn a_relocated_gumball_turns_about_the_new_origin() {
    let (mut controller, ids) = controller_with_points();
    controller.select_objects(&ids).expect("select");
    controller.submit("GumballOrigin 0,0,0").expect("relocate");
    assert_eq!(controller.gumball_origin(), Some([0.0, 0.0, 0.0]));
    controller.submit("RotateZ 90").expect("rotate");
    let (left, right) = ends(&controller, &ids);
    // About the world origin: (0,0,0) stays put and (10,0,0) goes to (0,10,0).
    assert!(left[0].abs() < 1e-9 && left[1].abs() < 1e-9, "{left:?}");
    assert!(right[0].abs() < 1e-9 && (right[1] - 10.0).abs() < 1e-9, "{right:?}");
}

#[test]
fn resetting_the_gumball_puts_the_pivot_back_at_the_selection_centre() {
    let (mut controller, ids) = controller_with_points();
    controller.select_objects(&ids).expect("select");
    controller.submit("GumballOrigin 0,0,0").expect("relocate");
    controller.submit("GumballReset").expect("reset");
    assert_eq!(controller.gumball_origin(), None);
    let pivot = controller.gumball_pivot().expect("pivot");
    assert!((pivot.origin[0] - 5.0).abs() < 1e-9, "{pivot:?}");
    controller.submit("RotateZ 90").expect("rotate");
    let mut points: Vec<[f64; 3]> = ids.iter().map(|id| world(&controller, *id)).collect();
    points.sort_by(|left, right| left[1].total_cmp(&right[1]));
    assert!((points[0][0] - 5.0).abs() < 1e-9 && (points[0][1] + 5.0).abs() < 1e-9, "{points:?}");
}

#[test]
fn object_alignment_follows_the_selected_objects_axes() {
    // The entity goes into the document before the controller builds its scene from it.
    let mut document = Document::new("aligned");
    let mut entity = Entity::new("turned", Geometry::Point([0.0, 0.0, 0.0]), None);
    entity.placement = gumball::rotation_about([0.0; 3], [0.0, 0.0, 1.0], 45f64.to_radians())
        .expect("rotation");
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut controller = Controller::new(document, Limits::default()).expect("controller");
    controller.select_objects(&[id]).expect("select");
    controller.submit("GumballObject").expect("align");
    let pivot = controller.gumball_pivot().expect("pivot");
    let diagonal = 45f64.to_radians().cos();
    assert!((pivot.x[0] - diagonal).abs() < 1e-9, "x axis turned with the object: {pivot:?}");
    assert!((pivot.x[1] - diagonal).abs() < 1e-9, "{pivot:?}");
    // World alignment puts them back.
    controller.submit("GumballWorld").expect("world");
    let pivot = controller.gumball_pivot().expect("pivot");
    assert_eq!(pivot.x, [1.0, 0.0, 0.0]);
    let _ = Axis::X;
    let _ = Transform::identity();
}

#[test]
fn cplane_alignment_follows_the_active_construction_plane_without_moving_the_object() {
    let (mut controller, ids) = controller_with_points();
    controller.select_objects(&ids).expect("select");
    let placement = controller.document().entities[&ids[0]].placement;
    // A plane turned 90 degrees about Z: its x axis points along world +Y.
    let plane = om9_scene::camera::CPlane::new([0.0; 3], [0.0, 1.0, 0.0], [-1.0, 0.0, 0.0])
        .expect("cplane");
    controller.set_cplane(plane).expect("set cplane");
    controller.submit("GumballCPlane").expect("align to cplane");
    let pivot = controller.gumball_pivot().expect("pivot");
    assert!((pivot.x[1] - 1.0).abs() < 1e-9, "the x handle follows the cplane: {pivot:?}");
    assert!((pivot.y[0] + 1.0).abs() < 1e-9, "the y handle follows the cplane: {pivot:?}");
    assert!((pivot.z[2] - 1.0).abs() < 1e-9, "z is x cross y: {pivot:?}");
    assert_eq!(
        controller.document().entities[&ids[0]].placement, placement,
        "changing the alignment does not touch the object"
    );
    controller.submit("GumballWorld").expect("back to world");
    let pivot = controller.gumball_pivot().expect("pivot");
    assert_eq!(pivot.x, [1.0, 0.0, 0.0]);
}
