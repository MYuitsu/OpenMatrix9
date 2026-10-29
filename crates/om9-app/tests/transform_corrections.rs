// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_app::session::{Controller, transform};
use om9_core::{
    budget::Limits,
    geometry::{Curve, Geometry, Transform},
    model::{Document, Entity},
};
fn controller(entities: Vec<Entity>) -> Controller {
    let mut d = Document::new("transform corrections");
    for e in entities {
        d.entities.insert(e.id, e);
    }
    Controller::new(d, Limits::default()).unwrap()
}
fn path() -> Entity {
    Entity::new(
        "path",
        Geometry::Curve(Curve::Line {
            start: [0.; 3],
            end: [30., 0., 0.],
            domain: [0., 30.],
        }),
        None,
    )
}
#[test]
fn spacing_is_in_mm_and_preserves_the_original_in_one_undo() {
    let source = Entity::new("source", Geometry::Point([0.; 3]), None);
    let id = source.id;
    let path = path();
    let p = path.id;
    let mut c = controller(vec![source, path]);
    c.select_objects(&[id, p]).unwrap();
    let before = c.document().clone();
    c.submit("ArrayCrv").unwrap();
    c.submit("Spacing=10mm").unwrap();
    assert!(c.is_running());
    let preview = c.transform_preview([0.; 3], "").unwrap();
    assert!(!preview.is_empty());
    assert_eq!(c.document(), &before);
    c.submit("").unwrap();
    assert_eq!(c.document().entities.len(), 5);
    let mut x: Vec<_> = c
        .document()
        .entities
        .values()
        .filter(|e| matches!(e.geometry, Geometry::Point(_)))
        .map(|e| e.placement.point([0.; 3]).unwrap()[0])
        .collect();
    x.sort_by(f64::total_cmp);
    for (a, b) in x.iter().zip([0., 10., 20., 30.]) {
        assert!((a - b).abs() < 1e-7, "{x:?}");
    }
    assert_eq!(c.coordinator().undo_len(), 1);
    c.undo().unwrap();
    assert_eq!(c.document().entities.len(), 2);
}
#[test]
fn roadlike_follows_the_tangent_and_preserves_cplane_up() {
    let source = Entity::new("source", Geometry::Point([0.; 3]), None);
    let id = source.id;
    let arc = Entity::new(
        "arc",
        Geometry::Curve(Curve::Circle {
            center: [0.; 3],
            x: [1., 0., 0.],
            y: [0., 1., 0.],
            radius: 10.,
            domain: [0., std::f64::consts::FRAC_PI_2],
            angle_domain: None,
        }),
        None,
    );
    let p = arc.id;
    let mut c = controller(vec![source, arc]);
    c.select_objects(&[id, p]).unwrap();
    c.submit("ArrayCrv").unwrap();
    c.submit("Roadlike").unwrap();
    c.submit("2").unwrap();
    let copy = c
        .document()
        .entities
        .values()
        .find(|e| e.id != id && matches!(e.geometry, Geometry::Point(_)))
        .unwrap();
    let origin = copy.placement.point([0.; 3]).unwrap();
    let up = copy.placement.point([0., 0., 1.]).unwrap();
    assert!((up[2] - origin[2] - 1.).abs() < 1e-8);
    let x = copy.placement.point([1., 0., 0.]).unwrap();
    assert!((x[0] - origin[0]).abs() < 1e-7);
    assert!((x[1] - origin[1] - 1.).abs() < 1e-7);
}
#[test]
fn setpt_edits_only_the_requested_control_coordinates_and_keeps_placement() {
    let mut entity = Entity::new(
        "line",
        Geometry::Curve(Curve::Line {
            start: [0., 0., 1.],
            end: [10., 0., 3.],
            domain: [0., 10.],
        }),
        None,
    );
    entity.placement = Transform::translation([5., 7., 2.]);
    let id = entity.id;
    let placement = entity.placement;
    let mut c = controller(vec![entity]);
    c.select_objects(&[id]).unwrap();
    let before = c.document().clone();
    c.submit("SetPt").unwrap();
    c.submit("Axes=Z").unwrap();
    c.submit("Indices=1").unwrap();
    let preview = c.transform_preview([0., 0., 9.], "").unwrap();
    assert!(!preview.is_empty());
    assert_eq!(c.document(), &before);
    c.submit("W0,0,9").unwrap();
    let entity = &c.document().entities[&id];
    assert_eq!(entity.placement, placement);
    let Geometry::Curve(Curve::Line { start, end, .. }) = &entity.geometry else {
        panic!("line")
    };
    assert_eq!(*start, [0., 0., 1.]);
    assert_eq!(*end, [10., 0., 7.]);
    c.undo().unwrap();
    assert_eq!(
        c.document().entities[&id].geometry,
        before.entities[&id].geometry
    );
}
#[test]
fn setpt_cplane_axes_and_cancel_are_read_only_until_commit() {
    let entity = Entity::new("point", Geometry::Point([1., 2., 3.]), None);
    let id = entity.id;
    let mut c = controller(vec![entity]);
    c.select_objects(&[id]).unwrap();
    c.set_cplane_preset(om9_scene::camera::CPlanePreset::Front)
        .unwrap();
    c.submit("SetPt").unwrap();
    c.submit("Axes=Y").unwrap();
    c.submit("CPlane").unwrap();
    c.submit("W9,9,8").unwrap();
    assert_eq!(
        c.document().entities[&id].geometry,
        Geometry::Point([1., 2., 8.])
    );
    let before = c.document().clone();
    c.submit("SetPt").unwrap();
    c.submit("Axes=Z").unwrap();
    assert!(c.submit("Indices=bad").is_err());
    c.submit("Cancel").unwrap();
    assert_eq!(c.document(), &before);
}
#[test]
fn blockers_refuse_instead_of_reusing_unrelated_grips_or_inventing_facing() {
    for id in [
        "OM9-TRANSFORM-019",
        "OM9-TRANSFORM-026",
        "OM9-TRANSFORM-037",
    ] {
        assert!(Controller::availability(id).is_err());
    }
    let mut c = controller(vec![]);
    assert!(
        c.dispatch(
            transform::SET_POINTS,
            om9_app::session::InvocationOrigin::Menu
        )
        .is_err()
    );
}
#[test]
fn inverse_surface_routines_exist_and_return_measured_projection() {
    let surface = om9_core::surface_kernel::plane(
        [0.; 3],
        [1., 0., 0.],
        [0., 1., 0.],
        10.,
        10.,
        &Limits::default(),
    )
    .unwrap();
    let projection = om9_core::curve_surface_ops::closest_point_on_surface(
        &surface,
        [3., 4., 2.],
        &Limits::default(),
    )
    .unwrap();
    assert!((projection.distance - 2.).abs() < 1e-8);
    assert!((projection.point[2]).abs() < 1e-8);
}
#[test]
fn spacing_with_no_copy_refuses_and_the_same_run_can_retry() {
    let e = Entity::new("source", Geometry::Point([0.; 3]), None);
    let id = e.id;
    let p = path();
    let path_id = p.id;
    let mut c = controller(vec![e, p]);
    c.select_objects(&[id, path_id]).unwrap();
    let before = c.document().clone();
    c.submit("ArrayCrv").unwrap();
    c.submit("Spacing=40mm").unwrap();
    assert!(c.submit("").is_err());
    assert!(c.is_running());
    assert_eq!(c.document(), &before);
    c.submit("Spacing=10mm").unwrap();
    c.submit("").unwrap();
    assert_eq!(c.document().entities.len(), 5);
}
