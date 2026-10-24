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
    c.submit("ExtractSrf 0").unwrap();
    assert_eq!(c.document().entities.len(), before + 1);
}

#[test]
fn solid_menu_usage_and_invalid_input_are_visible_errors() {
    let mut c = controller();
    assert!(c.submit("Box").is_ok());
    assert!(c.prompt().starts_with("First corner of base"));
    c.submit("0,0,0").unwrap();
    assert!(c.prompt().starts_with("Other corner of base"));
    c.submit("10,20,0").unwrap();
    assert!(c.prompt().starts_with("Height"));
    c.submit("30").unwrap();
    assert_eq!(c.document().entities.len(), 1);
    c.cancel();
    assert!(c.submit("Box 1 2").is_err());
    assert!(c.submit("Box 1 2 -3").is_err());
    assert!(c.submit("BooleanUnion").is_err());
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
fn solid_box_cube_mode_uses_the_diagonal_as_a_cube() {
    let mut c = controller();
    c.submit("Box").unwrap();
    c.submit("Cube").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([10.0, 10.0, 10.0]).unwrap();
    c.submit("").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    assert!((measures(&c, id).volume_mm3 - 1000.0).abs() < 1e-6);
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
    assert!(
        (measures(&c, id).volume_mm3 - 4.0 * std::f64::consts::PI / 3.0 * 125.0).abs() < 1e-6
    );
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
    let id = *c.document().entities.keys().find(|id| **id != curve_id).unwrap();
    assert!((measures(&c, id).volume_mm3 - 4.0 * std::f64::consts::PI / 3.0 * 125.0).abs() < 1e-6);
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
    c.submit("").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    assert!((measures(&c, id).volume_mm3 - 1000.0).abs() < 1e-6);
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
    let id = *c.document().entities.keys().next().unwrap();
    assert!((measures(&c, id).volume_mm3 - 4.0 * std::f64::consts::PI / 3.0 * 27.0).abs() < 1e-6);
    assert_eq!(
        c.document().entities[&id].placement,
        om9_core::geometry::Transform::translation([2.0, 5.0_f64.sqrt(), 0.0])
    );
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
    let id = *c.document().entities.keys().find(|id| **id != curve_id).unwrap();
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
    let id = *c.document().entities.keys().find(|id| **id != curve_id).unwrap();
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
    let id = *c.document().entities.keys().find(|id| **id != curve_id).unwrap();
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
    let id = *c.document().entities.keys().find(|id| **id != curve_id).unwrap();
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
    assert!(tokens.iter().any(|(_, command)| *command == "Mode=Diagonal"));
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
    for command in ["Mode=2Point", "Mode=3Point", "Mode=4Point", "Mode=Tangent", "Cancel"] {
        assert!(tokens.iter().any(|(_, value)| *value == command), "{command}");
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
    let horizontal = om9_core::curve_build::polyline(&[[-20.0, 0.0, 0.0], [20.0, 0.0, 0.0]]).unwrap();
    let vertical = om9_core::curve_build::polyline(&[[10.0, -20.0, 0.0], [10.0, 20.0, 0.0]]).unwrap();
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
    let id = *c.document().entities.keys().find(|id| **id != ida && **id != idb).unwrap();
    let placement = c.document().entities[&id].placement;
    assert!((placement.0[0][3] - 7.0).abs() < 1e-6, "{placement:?}");
    assert!((placement.0[1][3].abs() - 3.0).abs() < 1e-6, "{placement:?}");
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
    let id = *c.document().entities.keys().find(|id| **id != i1 && **id != i2 && **id != i3).unwrap();
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
    let id = *c.document().entities.keys().find(|id| **id != curve_id).unwrap();
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
    let id = *c.document().entities.keys().find(|id| **id != curve_id).unwrap();
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
    let horizontal = om9_core::curve_build::polyline(&[[-20.0, 0.0, 0.0], [20.0, 0.0, 0.0]]).unwrap();
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
    let id = *c.document().entities.keys().find(|id| **id != ida && **id != idb).unwrap();
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
    let id = *c.document().entities.keys().find(|id| **id != curve_id).unwrap();
    assert_eq!(
        c.document().entities[&id].placement,
        om9_core::geometry::Transform::translation([10.0, 0.0, 0.0])
    );
}

#[test]
fn solid_sphere_fit_points_accepts_a_selection_batch() {
    let mut doc = Document::new("fit selection");
    let mut ids = Vec::new();
    for point in [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0], [1.0, 1.0, 1.0]] {
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
    let id = *c.document().entities.keys().find(|id| **id != curve_id).unwrap();
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
    assert!(c
        .solid_option_tokens()
        .iter()
        .any(|(_, command)| *command == "Length="));
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
