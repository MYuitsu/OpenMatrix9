// SPDX-License-Identifier: LGPL-2.1-or-later
//! `OM9-L2-02-02`: world/local/relative point entry, CPlane presets and cursor constraints.
use om9_app::session::{CmdPhase, Controller};
use om9_core::{Error, budget::Limits, geometry::Geometry, model::Document};
use om9_scene::camera::{CPlane, CPlanePreset};
use om9_scene::workspace::ViewId;

fn controller() -> Controller {
    Controller::new(Document::new("cplane input"), Limits::default()).unwrap()
}

fn points(controller: &Controller) -> Vec<[f64; 3]> {
    controller
        .document()
        .entities
        .values()
        .filter_map(|entity| match &entity.geometry {
            Geometry::Point(point) => Some(*point),
            _ => None,
        })
        .collect()
}

fn point_world(controller: &Controller) -> [f64; 3] {
    let points = points(controller);
    assert_eq!(points.len(), 1, "exactly one point entity");
    points[0]
}

fn point_count(controller: &Controller) -> usize {
    points(controller).len()
}

#[test]
fn local_entry_uses_the_active_view_cplane() {
    let mut c = controller();
    c.workspace.active = ViewId::LookingDown;
    c.set_cplane(CPlane::new([0.; 3], [0., 1., 0.], [-1., 0., 0.]).unwrap())
        .unwrap();
    c.submit("Point").unwrap();
    c.submit("5,0,0").unwrap();
    assert_eq!(point_world(&c), [0., 5., 0.]);
}

#[test]
fn front_view_local_input_uses_the_front_cplane_not_top() {
    let mut c = controller();
    c.workspace.active = ViewId::ThroughFinger;
    c.submit("Point").unwrap();
    c.submit("0,5,0").unwrap();
    // Front x=world X, y=world Z: local (0,5,0) is world (0,0,5), not Top's (0,5,0).
    assert_eq!(point_world(&c), [0., 0., 5.]);
}

#[test]
fn world_entry_ignores_the_cplane_and_relative_uses_the_previous_point() {
    let mut c = controller();
    c.set_cplane(CPlane::new([10., 20., 30.], [0., 1., 0.], [0., 0., 1.]).unwrap())
        .unwrap();
    c.submit("Point").unwrap();
    c.submit("W10,20,30").unwrap();
    assert_eq!(point_world(&c), [10., 20., 30.]);

    c.submit("Point").unwrap();
    c.submit("@5,0,0").unwrap();
    // The relative displacement is along the active CPlane's x axis (world +Y here),
    // while the base is the previously accepted world point.
    let points = points(&c);
    assert!(points.contains(&[10., 20., 30.]));
    assert!(points.contains(&[10., 25., 30.]));
}

#[test]
fn relative_entry_without_a_base_is_refused_without_a_document_write() {
    let mut c = controller();
    c.submit("Point").unwrap();
    assert_eq!(
        c.submit("@5,0,0"),
        Err(Error::Missing("relative base point"))
    );
    assert_eq!(point_count(&c), 0);
    assert_eq!(c.phase(), CmdPhase::AwaitPoint);
}

#[test]
fn malformed_point_text_keeps_the_prompt_and_the_document_unchanged() {
    let mut c = controller();
    c.submit("Line").unwrap();
    c.submit("0,0,0").unwrap();
    let before = c.coordinator().snapshot();
    for text in ["NaN,0,0", "1e309,0,0", "1 2 3", "1,2,3,4"] {
        assert!(c.submit(text).is_err(), "{text:?} must be refused");
        assert_eq!(c.phase(), CmdPhase::AwaitEnd, "{text:?}");
        assert_eq!(c.coordinator().snapshot(), before, "{text:?}");
    }
    assert_eq!(
        c.document().entities.len(),
        0,
        "the line has not been committed"
    );
}

#[test]
fn f4_inserts_the_world_origin_through_the_point_prompt() {
    let mut c = controller();
    c.submit("Line").unwrap();
    c.submit_world_origin().unwrap();
    assert_eq!(c.phase(), CmdPhase::AwaitEnd);
    c.submit("W0,0,5").unwrap();
    let Geometry::Curve(curve) = &c.document().entities.values().next().unwrap().geometry else {
        panic!("line entity");
    };
    let om9_core::geometry::Curve::Line { start, end, .. } = curve else {
        panic!("line curve");
    };
    assert_eq!(*start, [0.; 3]);
    assert_eq!(*end, [0., 0., 5.]);

    let mut idle = controller();
    assert_eq!(
        idle.submit_world_origin(),
        Err(Error::Missing("point prompt"))
    );
    assert_eq!(point_count(&idle), 0);
}

#[test]
fn ortho_snaps_to_the_dominant_cplane_axis_from_the_previous_point() {
    let mut c = controller();
    c.set_cplane(CPlane::preset(CPlanePreset::Top)).unwrap();
    c.submit("Line").unwrap();
    c.submit("0,0,0").unwrap();
    c.set_ortho(true);
    assert_eq!(c.constrain_point([3., 1., 2.]).unwrap(), [3., 0., 0.]);
    assert_eq!(c.constrain_point([1., 4., 2.]).unwrap(), [0., 4., 0.]);
}

#[test]
fn planar_projects_onto_the_active_plane_and_both_modes_stay_on_it() {
    let mut c = controller();
    c.set_cplane(CPlane::new([10., 20., 0.], [1., 0., 0.], [0., 0., 1.]).unwrap())
        .unwrap();
    c.submit("Line").unwrap();
    c.submit("10,20,0").unwrap();
    c.set_planar(true);
    let planar = c.constrain_point([15., 27., 9.]).unwrap();
    assert_eq!(planar, [15., 20., 9.]);
    c.set_ortho(true);
    let both = c.constrain_point([15., 27., 9.]).unwrap();
    let local = <CPlane as om9_core::coords::LocalFrame>::to_local(
        &c.input_plane(),
        om9_core::coords::WorldPoint::try_new(both).unwrap(),
    )
    .unwrap();
    assert_eq!(local.xyz()[2], 0., "both modes must stay on the plane");
}

#[test]
fn cplane_origin_rotation_keeps_world_geometry_and_rejects_collinear_input() {
    let mut c = controller();
    c.submit("Point").unwrap();
    c.submit("W1,2,3").unwrap();
    let before = (
        c.document().entities.values().next().unwrap().placement,
        c.document()
            .entities
            .values()
            .next()
            .unwrap()
            .geometry
            .clone(),
    );
    c.set_cplane_origin_rotation([10., 20., 0.], 30f64.to_radians())
        .unwrap();
    assert_eq!(c.input_plane().origin, [10., 20., 0.]);
    assert!((c.input_plane().x[0] - 30f64.to_radians().cos()).abs() < 1e-12);
    assert_eq!(
        (
            c.document().entities.values().next().unwrap().placement,
            c.document()
                .entities
                .values()
                .next()
                .unwrap()
                .geometry
                .clone(),
        ),
        before,
        "changing the construction plane never moves world geometry"
    );
    let plane = c.input_plane();
    assert_eq!(
        c.set_cplane_from_three_points([0.; 3], [1., 0., 0.], [2., 0., 0.]),
        Err(Error::Invalid("CPlane frame points"))
    );
    assert_eq!(
        c.input_plane(),
        plane,
        "a failed frame leaves the plane alone"
    );
}

#[test]
fn switching_views_reads_the_other_views_plane_without_moving_the_world_point() {
    let mut c = controller();
    c.workspace.active = ViewId::LookingDown;
    c.set_cplane(CPlane::new([10., 20., 0.], [0., 1., 0.], [-1., 0., 0.]).unwrap())
        .unwrap();
    c.submit("Point").unwrap();
    c.submit("W10,20,30").unwrap();
    let world = point_world(&c);
    let top_plane = c.input_plane();

    c.workspace
        .views
        .get_mut(&ViewId::LookingDown)
        .unwrap()
        .camera
        .pan([12., -3.])
        .unwrap();
    assert_eq!(point_world(&c), world, "camera pan never moves the point");

    c.workspace.active = ViewId::ThroughFinger;
    assert_ne!(c.input_plane(), top_plane, "each view owns its CPlane");
    let front = c.input_plane();
    let expected = CPlane::preset(CPlanePreset::Front);
    assert_eq!(front.origin, expected.origin);
    assert_eq!(front.x, expected.x);
    assert_eq!(front.y, expected.y);
    assert_eq!(point_world(&c), world, "the stored point is world-space");

    c.workspace.active = ViewId::Perspective;
    c.set_cplane_preset(CPlanePreset::Top).unwrap();
    assert_eq!(point_world(&c), world);
    let top = c.input_plane();
    let expected = CPlane::preset(CPlanePreset::Top);
    assert_eq!(top.origin, expected.origin);
    assert_eq!(top.x, expected.x);
    assert_eq!(top.y, expected.y);
}
