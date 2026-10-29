// SPDX-License-Identifier: LGPL-2.1-or-later
//! Metal Weights (`OM9-L2-04-19`): closed volumes, material density, grams/dwt, multi-material
//! breakdown and the open-shell refusal that leaves the previous report intact.
#![cfg(feature = "occt-brep")]

use om9_app::session::{Controller, InvocationOrigin};
use om9_core::{
    Error,
    budget::Limits,
    geometry::Geometry,
    metal_weight::{DEFAULT_MATERIAL_ID, MetalMaterial},
    model::{Document, Entity},
};
use om9_kernel::creation::{self, Primitive};

fn controller_with(entities: Vec<Entity>) -> Controller {
    let mut document = Document::new("metal-weights");
    for entity in entities {
        document.entities.insert(entity.id, entity);
    }
    Controller::new(document, Limits::default()).expect("controller")
}

fn box_entity(label: &str, size: [f64; 3]) -> Entity {
    let brep = creation::primitive(
        Primitive::Box {
            x_mm: size[0],
            y_mm: size[1],
            z_mm: size[2],
        },
        creation::DEFAULT_DEFLECTION_MM,
        &Limits::default(),
    )
    .expect("closed box");
    Entity::new(label, Geometry::Brep(brep), None)
}

fn open_shell_entity() -> Entity {
    let shell = creation::extrude_wire(
        &[
            [0.0, 0.0, 0.0],
            [10.0, 0.0, 0.0],
            [10.0, 2.0, 0.0],
        ],
        [0.0, 0.0, 2.0],
        creation::DEFAULT_DEFLECTION_MM,
        &Limits::default(),
    )
    .expect("open side shell");
    Entity::new("open-shell", Geometry::Brep(shell), None)
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 1e-6 * expected.abs().max(1.0),
        "expected {expected}, got {actual}"
    );
}

#[test]
fn the_icon_reports_a_closed_box_in_grams_and_pennyweights_without_editing() {
    let entity = box_entity("box", [10.0, 5.0, 2.0]);
    let id = entity.id;
    let mut controller = controller_with(vec![entity]);
    let before = controller.document().clone();
    controller.select_objects(&[id]).unwrap();

    controller
        .dispatch("OM9-TOOLS-006", InvocationOrigin::Menu)
        .expect("metal weights icon");

    let report = controller.metal_report().expect("report");
    assert_close(report.volume_mm3, 100.0);
    assert_eq!(report.material_id, DEFAULT_MATERIAL_ID);
    assert_close(report.grams, 100.0 / 1000.0 * 15.5);
    assert_close(report.dwt, report.grams / 1.555_173_84);
    assert_eq!(report.parts.len(), 1);
    assert_eq!(report.parts[0].object, id);
    assert_close(report.parts[0].volume_mm3, 100.0);
    assert!(report.text.contains("g"));
    assert!(report.text.contains("dwt"));
    assert_eq!(controller.document(), &before, "read-only report");
    assert_eq!(controller.coordinator().undo_len(), 0);
}

#[test]
fn the_command_line_matches_the_icon_and_accepts_a_material() {
    let entity = box_entity("box", [10.0, 5.0, 2.0]);
    let id = entity.id;
    let mut controller = controller_with(vec![entity]);
    controller.select_objects(&[id]).unwrap();

    controller.submit("MetalWeights").expect("typed command");
    let default = controller.metal_report().expect("report").clone();
    controller
        .submit("Metal Weights material=platinum_950")
        .expect("typed material");
    let platinum = controller.metal_report().expect("report");
    assert_eq!(platinum.material_id, "platinum_950");
    assert!(platinum.grams > default.grams);
    assert!(platinum.text.contains("Platinum 950"));
    assert_eq!(
        controller.submit("MetalWeights material="),
        Err(Error::Missing("metal material id"))
    );
    assert_eq!(
        controller.submit("MetalWeights wrong=1"),
        Err(Error::Invalid("metal weights option"))
    );
}

#[test]
fn an_open_shell_is_refused_and_the_previous_report_is_preserved() {
    let closed = box_entity("closed", [10.0, 5.0, 2.0]);
    let closed_id = closed.id;
    let open = open_shell_entity();
    let open_id = open.id;
    let mut controller = controller_with(vec![closed, open]);
    controller.select_objects(&[closed_id]).unwrap();
    controller
        .dispatch("OM9-TOOLS-006", InvocationOrigin::Menu)
        .expect("closed report");
    let previous = controller.metal_report().expect("report").clone();
    let document = controller.document().clone();

    controller.select_objects(&[open_id]).unwrap();
    assert_eq!(
        controller.dispatch("OM9-TOOLS-006", InvocationOrigin::Menu),
        Err(Error::Invalid("metal weight requires a closed solid"))
    );
    assert_eq!(controller.metal_report(), Some(&previous));
    assert_eq!(controller.document(), &document);
    assert_eq!(controller.coordinator().undo_len(), 0);
}

#[test]
fn a_missing_density_is_a_typed_refusal_not_a_zero_mass() {
    let entity = box_entity("box", [10.0, 5.0, 2.0]);
    let id = entity.id;
    let mut controller = controller_with(vec![entity]);
    controller.select_objects(&[id]).unwrap();
    controller
        .dispatch("OM9-TOOLS-006", InvocationOrigin::Menu)
        .expect("valid report");
    let previous = controller.metal_report().expect("report").clone();

    assert_eq!(
        controller.calculate_metal_weights(Some("missing_material")),
        Err(Error::Missing("metal material density"))
    );
    assert_eq!(controller.metal_report(), Some(&previous));
    assert_eq!(controller.coordinator().undo_len(), 0);
}

#[test]
fn a_multi_material_selection_reports_each_body_and_the_total() {
    let a = box_entity("body-a", [10.0, 5.0, 2.0]); // 100 mm^3
    let b = box_entity("body-b", [10.0, 10.0, 2.0]); // 200 mm^3
    let (a_id, b_id) = (a.id, b.id);
    let mut controller = controller_with(vec![a, b]);
    controller
        .metal_registry_mut()
        .insert(MetalMaterial::test_only("synthetic_material", 10.0, "test fixture").unwrap())
        .unwrap();
    controller
        .metal_registry_mut()
        .insert(MetalMaterial::test_only("light_material", 8.0, "test fixture").unwrap())
        .unwrap();
    controller.select_objects(&[a_id, b_id]).unwrap();

    let report = controller
        .calculate_metal_weights_with(&[
            (a_id, "synthetic_material"),
            (b_id, "light_material"),
        ])
        .expect("breakdown");
    assert_eq!(report.material_id, "mixed");
    assert_eq!(report.parts.len(), 2);
    assert_close(report.parts[0].grams, 1.0);
    assert_close(report.parts[1].grams, 1.6);
    assert_close(report.grams, 2.6);
    assert_close(report.volume_mm3, 300.0);
    assert_eq!(report.parts[0].material_id, "synthetic_material");
    assert_eq!(report.parts[1].material_id, "light_material");
    assert_eq!(controller.coordinator().undo_len(), 0);
}

#[test]
fn an_assignment_that_omits_a_selected_body_is_refused() {
    let a = box_entity("body-a", [10.0, 5.0, 2.0]);
    let b = box_entity("body-b", [10.0, 10.0, 2.0]);
    let a_id = a.id;
    let mut controller = controller_with(vec![a, b]);
    controller.select_objects(&[a_id]).unwrap();
    // The selection has one body, but the assignment names a different pair of ids.
    let other = controller.document().entities.keys().copied().find(|id| *id != a_id);
    let Some(other) = other else {
        panic!("second body");
    };
    assert_eq!(
        controller.calculate_metal_weights_with(&[(a_id, "gold_18k_yellow"), (other, "gold_18k_yellow")]),
        Err(Error::Invalid("metal weight material assignment"))
    );
}
