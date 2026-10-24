// SPDX-License-Identifier: LGPL-2.1-or-later
//! Actual native SDK-consumer checks. These run only against an explicitly built
//! and packaged hello-plugin, never against a mock or an inferred binary path.

use om9_app::command_events::{EventKind, EventOutcome};
use om9_app::file_lifecycle::{FileKind, FileOutcome};
use om9_app::session::Controller;
use om9_archive::Archive;
use om9_core::{
    budget::Limits,
    geometry::{Curve, Geometry},
    identity::ObjectId,
    model::Document,
    transaction::Cancellation,
};
use std::{collections::BTreeSet, path::PathBuf};

fn example_package() -> PathBuf {
    let package = std::env::var_os("OM9_TEST_PLUGIN_PACKAGE")
        .map(PathBuf::from)
        .expect("set OM9_TEST_PLUGIN_PACKAGE to a built hello-plugin .om9p package");
    assert!(package.is_file(), "missing package: {}", package.display());
    package
}

fn load_example(controller: &mut Controller) {
    let path_json =
        serde_json::to_string(example_package().to_str().expect("UTF-8 package path")).unwrap();
    controller
        .submit(&format!("PluginLoad {path_json}"))
        .unwrap();
    let plugins = controller.plugin_descriptors();
    assert_eq!(plugins.len(), 1);
    assert_eq!(plugins[0].id, "org.openmatrix9.hello");
    assert!(
        plugins[0]
            .commands
            .iter()
            .any(|command| command.id == "CreateProfile")
    );
}

fn controller_with_example() -> Controller {
    let mut controller = Controller::new(
        Document::with_template_tolerance("Native SDK integration"),
        Limits::default(),
    )
    .unwrap();
    load_example(&mut controller);
    controller
}

fn profile_ids(controller: &Controller) -> BTreeSet<om9_core::identity::ObjectId> {
    controller.document().entities.keys().copied().collect()
}

fn assert_profile(controller: &Controller) {
    let document = controller.document();
    assert_eq!(document.entities.len(), 3);
    let geometry = |label: &str| {
        &document
            .entities
            .values()
            .find(|entity| entity.label == label)
            .unwrap()
            .geometry
    };
    assert!(matches!(geometry("SDK line"), Geometry::Curve(Curve::Line {
        start, end, ..
    }) if *start == [0.0; 3] && *end == [10.0, 0.0, 0.0]));
    assert!(
        matches!(geometry("SDK circle"), Geometry::Curve(Curve::Circle {
        center, radius, ..
    }) if *center == [0.0; 3] && *radius == 5.0)
    );
    assert!(
        matches!(geometry("SDK profile"), Geometry::Curve(Curve::Polycurve {
        segments, reversed: false,
    }) if segments.len() == 4)
    );
    let scene = controller.scene().expect("published plugin scene");
    assert_eq!(scene.epoch, controller.coordinator().epoch());
    let scene_ids: BTreeSet<_> = scene.objects.iter().map(|object| object.root).collect();
    assert_eq!(scene_ids, profile_ids(controller));
}

#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_cmd_geometry_undo_redo_and_archive_roundtrip() {
    let mut controller = controller_with_example();
    controller
        .submit("PluginRun org.openmatrix9.hello CreateProfile {\"radius_mm\":5}")
        .unwrap();
    assert_profile(&controller);
    let ids = profile_ids(&controller);
    assert_eq!(controller.coordinator().undo_len(), 1);
    controller.undo().unwrap();
    assert!(controller.document().entities.is_empty());
    assert!(controller.scene().unwrap().objects.is_empty());
    controller.redo().unwrap();
    assert_profile(&controller);
    assert_eq!(profile_ids(&controller), ids);

    struct TemporaryProject(PathBuf);
    impl Drop for TemporaryProject {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }
    let path = TemporaryProject(
        std::env::temp_dir().join(format!("om9-native-plugin-{}.om9", uuid::Uuid::new_v4(),)),
    );
    Archive::save(
        &path.0,
        controller.document(),
        &Limits::default(),
        &Cancellation::default(),
    )
    .unwrap();
    let restored = Archive::open_restored(&path.0, &Limits::default(), &BTreeSet::new()).unwrap();
    assert_eq!(restored.document.entities, controller.document().entities);
    let reopened = Controller::new(restored.document, Limits::default()).unwrap();
    assert_profile(&reopened);
    assert_eq!(profile_ids(&reopened), ids);
}

#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_cancel_and_unknown_command_preserve_scene_and_existing_redo() {
    let mut controller = controller_with_example();
    controller
        .submit("PluginRun org.openmatrix9.hello CreateProfile {}")
        .unwrap();
    let ids = profile_ids(&controller);
    controller.undo().unwrap();
    let before = controller.document().clone();
    let scene_epoch = controller.scene().unwrap().epoch;
    for command in [
        "PluginRun org.openmatrix9.hello CreateProfile {\"cancel\":true}",
        "PluginRun org.openmatrix9.hello UnknownCommand {}",
    ] {
        assert!(controller.submit(command).is_err());
        assert_eq!(controller.document(), &before);
        assert_eq!(controller.scene().unwrap().epoch, scene_epoch);
        assert!(controller.scene().unwrap().objects.is_empty());
        assert_eq!(controller.coordinator().undo_len(), 0);
        assert_eq!(controller.coordinator().redo_len(), 1);
    }
    controller.redo().unwrap();
    assert_profile(&controller);
    assert_eq!(profile_ids(&controller), ids);
}

#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_reads_units_tolerance_and_typed_numbers_through_the_sdk() {
    let mut controller = controller_with_example();
    let before = controller.document().clone();
    let report = controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "UnitsReport",
            serde_json::json!({}),
        )
        .unwrap();
    assert_eq!(report["unit"], "mm");
    assert_eq!(report["millimeters_per_unit"], 1.0);
    assert_eq!(report["tolerance"]["absolute_mm"], 0.001);
    assert_eq!(report["tolerance"]["relative"], 1.0e-4);
    assert_eq!(
        report["tolerance"]["angular_rad"],
        std::f64::consts::PI / 180.0
    );
    assert_eq!(report["one_inch_mm"], 25.4);
    assert_eq!(report["bare_mm"], 2.5);
    assert_eq!(report["right_degrees"], 90.0);
    assert_eq!(report["right_radians"], std::f64::consts::FRAC_PI_2);
    assert_eq!(
        report["coordinates_mm"],
        serde_json::json!([10.0, 20.0, 30.0])
    );
    let capabilities = report["capabilities"].as_array().expect("capability list");
    for capability in [
        "document_units",
        "parse_length",
        "parse_angle",
        "parse_coordinates",
    ] {
        assert!(
            capabilities.iter().any(|value| value == capability),
            "missing advertised capability {capability}"
        );
    }
    // A read-only command publishes nothing and keeps the document byte-identical.
    assert_eq!(controller.document(), &before);
    assert_eq!(controller.coordinator().undo_len(), 0);
    assert_eq!(controller.coordinator().redo_len(), 0);

    // A document that states no tolerance must be reported as null, not defaulted.
    let mut plain = Controller::new(Document::new("no tolerance"), Limits::default()).unwrap();
    load_example(&mut plain);
    let report = plain
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "UnitsReport",
            serde_json::json!({}),
        )
        .unwrap();
    assert!(report["tolerance"].is_null());
}

#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_selected_geometry_move_delete_and_undo_redo() {
    let mut controller = controller_with_example();
    controller
        .submit("PluginRun org.openmatrix9.hello CreateProfile {}")
        .unwrap();
    let ids = profile_ids(&controller);
    controller
        .select_objects(&ids.iter().copied().collect::<Vec<_>>())
        .unwrap();
    assert_eq!(controller.selected(), &ids);
    controller
        .submit("PluginRun org.openmatrix9.hello MoveSelection {\"dx_mm\":2}")
        .unwrap();
    assert_profile(&controller);
    assert!(
        controller
            .document()
            .entities
            .values()
            .all(|entity| entity.placement.0[0][3] == 2.0)
    );
    assert_eq!(controller.coordinator().undo_len(), 2);
    controller
        .submit("PluginRun org.openmatrix9.hello DeleteSelection {}")
        .unwrap();
    assert!(controller.document().entities.is_empty());
    assert!(controller.scene().unwrap().objects.is_empty());
    assert!(controller.selected().is_empty());
    assert_eq!(controller.coordinator().undo_len(), 3);
    controller.undo().unwrap();
    assert_profile(&controller);
    assert_eq!(profile_ids(&controller), ids);
    assert!(
        controller
            .document()
            .entities
            .values()
            .all(|entity| entity.placement.0[0][3] == 2.0)
    );
    controller.redo().unwrap();
    assert!(controller.document().entities.is_empty());
    assert!(controller.scene().unwrap().objects.is_empty());
    controller.undo().unwrap();
    controller.undo().unwrap();
    assert_profile(&controller);
    assert!(
        controller
            .document()
            .entities
            .values()
            .all(|entity| entity.placement.0[0][3] == 0.0)
    );
}

#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_edits_object_attributes_and_queries_through_the_sdk() {
    let mut controller = controller_with_example();
    controller
        .submit("PluginRun org.openmatrix9.hello CreateProfile {}")
        .unwrap();
    let after_profile = controller.document().clone();
    let report = controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "AttributesDemo",
            serde_json::json!({}),
        )
        .unwrap();
    assert_eq!(report["total"], 3);
    let first = report["first"].as_object().expect("attribute record");
    assert_eq!(first["color"], serde_json::json!([255, 136, 0, 255]));
    assert_eq!(first["locked"], true);
    // A locked object stays visible but is no longer editable.
    assert_eq!(first["effective_visible"], true);
    assert_eq!(first["effective_editable"], false);
    let first_id = ObjectId(
        uuid::Uuid::parse_str(first["id"].as_str().expect("first object id")).expect("uuid"),
    );
    assert_eq!(report["second_label"], "SDK renamed");
    let document = controller.document();
    let entity = &document.entities[&first_id];
    assert_eq!(entity.attributes.color, Some([255, 136, 0, 255]));
    assert!(entity.attributes.locked);
    assert_eq!(
        entity.attributes_revision,
        after_profile.entities[&first_id].attributes_revision + 1
    );
    // CreateProfile and the attribute demo are two ordinary Undo units.
    assert_eq!(controller.coordinator().undo_len(), 2);
    controller.undo().unwrap();
    assert_eq!(controller.document(), &after_profile);
    controller.redo().unwrap();
    assert_eq!(
        controller.document().entities[&first_id].attributes.color,
        Some([255, 136, 0, 255])
    );
}

#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_creates_a_layer_and_assigns_an_object_through_the_sdk() {
    let mut controller = controller_with_example();
    controller
        .submit("PluginRun org.openmatrix9.hello CreateProfile {}")
        .unwrap();
    let after_profile = controller.document().clone();
    let report = controller
        .run_loaded_plugin("org.openmatrix9.hello", "LayerDemo", serde_json::json!({}))
        .unwrap();
    let layer_id = report["layer"].as_str().expect("created layer id");
    let object_id = report["object"].as_str().expect("assigned object id");
    assert_eq!(report["object_layer"], layer_id);
    assert!(report["object_color"].is_null());
    assert_eq!(report["new_object_layer"], layer_id);
    assert_eq!(report["layer_count"], 1);
    let layer_id =
        om9_core::identity::LayerId(uuid::Uuid::parse_str(layer_id).expect("layer uuid"));
    let object_id = ObjectId(uuid::Uuid::parse_str(object_id).expect("object uuid"));
    let new_object_id = ObjectId(
        uuid::Uuid::parse_str(report["new_object"].as_str().expect("new object id"))
            .expect("new object uuid"),
    );
    let document = controller.document();
    assert_eq!(document.layers[&layer_id].name, "SDK Gems");
    assert_eq!(document.layers[&layer_id].color, [12, 200, 140, 255]);
    assert_eq!(document.current_layer, Some(layer_id));
    assert_eq!(
        document.entities[&object_id].attributes.layer,
        Some(layer_id)
    );
    assert_eq!(document.entities[&object_id].attributes.color, None);
    // The layer created by the same command accepted a newly added object.
    assert_eq!(
        document.entities[&new_object_id].attributes.layer,
        Some(layer_id)
    );
    assert_eq!(document.entities.len(), 4);
    assert_eq!(controller.coordinator().undo_len(), 2);
    controller.undo().unwrap();
    assert_eq!(controller.document(), &after_profile);
    controller.redo().unwrap();
    assert_eq!(
        controller.document().entities[&object_id].attributes.layer,
        Some(layer_id)
    );
    assert_eq!(
        controller.document().entities[&new_object_id]
            .attributes
            .layer,
        Some(layer_id)
    );
}

#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_validates_typed_parameters_through_the_sdk() {
    let mut controller = controller_with_example();
    let before = controller.document().clone();
    let report = controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "ParameterDemo",
            serde_json::json!({"radius": {"kind": "length", "millimeters": 3}}),
        )
        .unwrap();
    assert_eq!(
        report["parameters"]["radius"],
        serde_json::json!({"kind": "length", "millimeters": 3.0})
    );
    assert_eq!(
        report["parameters"]["sides"],
        serde_json::json!({"kind": "count", "value": 6})
    );
    assert_eq!(
        report["parameters"]["profile"],
        serde_json::json!({"kind": "choice", "value": "round"})
    );
    // Reading and validating parameters stages no mutation.
    assert_eq!(controller.document(), &before);
    assert_eq!(controller.coordinator().undo_len(), 0);

    for arguments in [
        serde_json::json!({"radius": 3.0}),
        serde_json::json!({"radius": {"kind": "angle", "degrees": 3.0}}),
        serde_json::json!({"radius": {"kind": "length", "millimeters": 80.0}}),
        serde_json::json!({}),
    ] {
        assert!(
            controller
                .run_loaded_plugin("org.openmatrix9.hello", "ParameterDemo", arguments)
                .is_err()
        );
        assert_eq!(controller.document(), &before);
        assert_eq!(controller.coordinator().undo_len(), 0);
    }
}

#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_reports_progress_and_warnings_into_the_command_event_log() {
    let mut controller = controller_with_example();
    let before = controller.document().clone();
    let report = controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "FeedbackDemo",
            serde_json::json!({"steps": 2}),
        )
        .unwrap();
    assert_eq!(report["reports"], 3);
    let label = "org.openmatrix9.hello:FeedbackDemo";
    let run: Vec<_> = controller
        .events
        .events()
        .iter()
        .filter(|event| event.command == label)
        .cloned()
        .collect();
    assert_eq!(
        run.iter().map(|event| event.kind).collect::<Vec<_>>(),
        vec![
            EventKind::Started,
            EventKind::Progress,
            EventKind::Progress,
            EventKind::Warn,
            EventKind::Finished
        ]
    );
    assert_eq!(run[1].detail, "feedback demo started");
    assert_eq!(run[2].detail, "feedback demo half way");
    assert_eq!(run[3].detail, "feedback demo warning");
    assert_eq!(run[4].outcome, Some(EventOutcome::Committed));
    // Reports are not document state.
    assert_eq!(controller.document(), &before);
    assert_eq!(controller.coordinator().undo_len(), 0);

    // A refused command keeps its reports, changes nothing and keeps the old Redo.
    controller
        .submit("PluginRun org.openmatrix9.hello CreateProfile {}")
        .unwrap();
    controller.undo().unwrap();
    let after_undo = controller.document().clone();
    let epoch = controller.scene().unwrap().epoch;
    assert!(
        controller
            .run_loaded_plugin(
                "org.openmatrix9.hello",
                "FeedbackDemo",
                serde_json::json!({"steps": 2, "fail": true})
            )
            .is_err()
    );
    let events: Vec<_> = controller
        .events
        .events()
        .iter()
        .filter(|event| event.command == label)
        .cloned()
        .collect();
    // A refused invocation publishes nothing, including its staged reports: the
    // Finished event carries the refusal.
    let last = &events[events.len() - 2..];
    assert_eq!(
        last.iter().map(|event| event.kind).collect::<Vec<_>>(),
        vec![EventKind::Started, EventKind::Finished]
    );
    assert_eq!(last[1].outcome, Some(EventOutcome::Refused));
    assert_eq!(controller.document(), &after_undo);
    assert_eq!(controller.scene().unwrap().epoch, epoch);
    assert_eq!(controller.coordinator().redo_len(), 1);
}

#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_writes_the_selection_through_the_sdk() {
    let mut controller = controller_with_example();
    controller
        .submit("PluginRun org.openmatrix9.hello CreateProfile {}")
        .unwrap();
    let after_profile = controller.document().clone();
    let report = controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "SelectionDemo",
            serde_json::json!({}),
        )
        .unwrap();
    let ids: Vec<ObjectId> = report["ids"]
        .as_array()
        .expect("selected ids")
        .iter()
        .map(|value| ObjectId(uuid::Uuid::parse_str(value.as_str().unwrap()).unwrap()))
        .collect();
    assert_eq!(ids.len(), 2);
    assert_eq!(
        controller.selected().iter().copied().collect::<Vec<_>>(),
        ids
    );
    // Selection is not document state.
    assert_eq!(controller.document(), &after_profile);
    assert_eq!(controller.coordinator().undo_len(), 1);

    // Adding keeps the entries the selection already had.
    controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "SelectionDemo",
            serde_json::json!({"add": true}),
        )
        .unwrap();
    assert_eq!(controller.selected().len(), 2);

    // Toggle removes the first root, then remove drops the remaining one.
    controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "SelectionDemo",
            serde_json::json!({"toggle": true}),
        )
        .unwrap();
    assert_eq!(controller.selected().len(), 1);
    controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "SelectionDemo",
            serde_json::json!({"remove": true}),
        )
        .unwrap();
    assert!(controller.selected().is_empty());

    // A canonical but unknown root refuses the command and keeps the selection.
    let before: Vec<ObjectId> = controller.selected().iter().copied().collect();
    assert!(
        controller
            .run_loaded_plugin(
                "org.openmatrix9.hello",
                "SelectionDemo",
                serde_json::json!({"invalid": true})
            )
            .is_err()
    );
    assert_eq!(
        controller.selected().iter().copied().collect::<Vec<_>>(),
        before
    );
    assert_eq!(controller.document(), &after_profile);
    assert_eq!(controller.coordinator().undo_len(), 1);
}

#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_reads_curve_information_and_samples_through_the_sdk() {
    let mut controller = controller_with_example();
    let report = controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "CreateProfile",
            serde_json::json!({"radius_mm": 5}),
        )
        .unwrap();
    let objects: Vec<String> = report["objects"]
        .as_array()
        .expect("created objects")
        .iter()
        .map(|value| value.as_str().unwrap().to_string())
        .collect();
    let line_id = objects[0].clone();
    let circle_id = objects[1].clone();
    let polyline_id = objects[2].clone();
    let after_profile = controller.document().clone();

    // Line: local domain [0, 1], midpoint [5, 0, 0], derivative (10, 0, 0).
    let line = controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "CurveDemo",
            serde_json::json!({"id": line_id}),
        )
        .unwrap();
    assert_eq!(line["info"]["kind"], "line");
    assert_eq!(line["info"]["domain"], serde_json::json!([0.0, 1.0]));
    assert_eq!(
        line["samples"][1]["point"],
        serde_json::json!([5.0, 0.0, 0.0])
    );
    assert_eq!(
        line["samples"][1]["derivatives"],
        serde_json::json!([[10.0, 0.0, 0.0]])
    );

    // Circle: domain [0, tau], midpoint at -x, derivative at the midpoint -y*radius.
    let circle = controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "CurveDemo",
            serde_json::json!({"id": circle_id}),
        )
        .unwrap();
    assert_eq!(circle["info"]["kind"], "circle");
    assert_eq!(
        circle["info"]["domain"],
        serde_json::json!([0.0, std::f64::consts::TAU])
    );
    let midpoint = circle["samples"][1]["point"]
        .as_array()
        .expect("circle midpoint");
    assert!((midpoint[0].as_f64().unwrap() + 5.0).abs() < 1e-12);
    assert!(midpoint[1].as_f64().unwrap().abs() < 1e-12);
    assert_eq!(midpoint[2].as_f64().unwrap(), 0.0);
    let derivative = circle["samples"][1]["derivatives"][0]
        .as_array()
        .expect("circle derivative");
    assert!(derivative[0].as_f64().unwrap().abs() < 1e-12);
    assert!((derivative[1].as_f64().unwrap() + 5.0).abs() < 1e-12);

    // Definitions expose no NURBS control data for a line; the circle has no arc domain.
    let line_definition = controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "CurveDemo",
            serde_json::json!({"id": line_id, "definition": true}),
        )
        .unwrap();
    assert_eq!(line_definition["definition"]["kind"], "line");
    assert!(line_definition["definition"]["poles"].is_null());
    assert!(line_definition["definition"]["knots"].is_null());
    let circle_definition = controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "CurveDemo",
            serde_json::json!({"id": circle_id, "definition": true}),
        )
        .unwrap();
    assert_eq!(circle_definition["definition"]["kind"], "circle");
    assert!(circle_definition["definition"]["angle_domain"].is_null());

    // Semantics use the document's stated tolerance: line open, square polyline closed.
    let line_semantics = controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "CurveDemo",
            serde_json::json!({"id": line_id, "semantics": true}),
        )
        .unwrap();
    assert_eq!(line_semantics["semantics"]["closed"], false);
    assert_eq!(line_semantics["semantics"]["planar"], true);
    assert_eq!(line_semantics["semantics"]["tolerance_mm"], 0.001);
    let polyline_semantics = controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "CurveDemo",
            serde_json::json!({"id": polyline_id, "semantics": true}),
        )
        .unwrap();
    assert_eq!(polyline_semantics["semantics"]["kind"], "polyline");
    assert_eq!(polyline_semantics["semantics"]["closed"], true);

    // A polycurve has no single parameter domain and evaluation is refused.
    assert!(
        controller
            .run_loaded_plugin(
                "org.openmatrix9.hello",
                "CurveDemo",
                serde_json::json!({"id": polyline_id})
            )
            .is_err()
    );
    assert!(
        controller
            .run_loaded_plugin(
                "org.openmatrix9.hello",
                "CurveDemo",
                serde_json::json!({"id": polyline_id, "sample": true})
            )
            .is_err()
    );
    // Reading curves stages no mutation.
    assert_eq!(controller.document(), &after_profile);
    assert_eq!(controller.coordinator().undo_len(), 1);
}

#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_builds_arc_and_spline_curves_through_the_sdk() {
    let mut controller = controller_with_example();
    controller
        .submit("PluginRun org.openmatrix9.hello CreateProfile {}")
        .unwrap();
    let after_profile = controller.document().clone();
    let report = controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "CurveBuildDemo",
            serde_json::json!({}),
        )
        .unwrap();
    let arc_id = report["arc"].as_str().expect("arc id");
    let spline_id = report["spline"].as_str().expect("spline id");
    assert_eq!(report["arc_info"]["kind"], "circle");
    let midpoint = report["arc_midpoint"][0]["point"].as_array().unwrap();
    assert!(midpoint[0].as_f64().unwrap().abs() < 1e-12);
    assert!((midpoint[1].as_f64().unwrap() - 5.0).abs() < 1e-12);
    assert_eq!(report["spline_definition"]["kind"], "nurbs");
    assert_eq!(report["spline_definition"]["degree"], 3);
    assert_eq!(
        report["spline_definition"]["poles"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
    let control_id = report["control"].as_str().expect("control nurbs id");
    assert_eq!(report["control_definition"]["kind"], "nurbs");
    assert_eq!(report["control_definition"]["degree"], 3);
    assert_eq!(
        report["control_definition"]["poles"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
    let document = controller.document();
    assert_eq!(document.entities.len(), 6);
    assert_eq!(controller.coordinator().undo_len(), 2);
    let arc_id = ObjectId(uuid::Uuid::parse_str(arc_id).unwrap());
    assert!(matches!(
        document.entities[&arc_id].geometry,
        Geometry::Curve(Curve::Circle {
            angle_domain: Some(_),
            ..
        })
    ));
    let spline_id = ObjectId(uuid::Uuid::parse_str(spline_id).unwrap());
    assert!(matches!(
        document.entities[&spline_id].geometry,
        Geometry::Curve(Curve::Nurbs(_))
    ));
    let control_id = ObjectId(uuid::Uuid::parse_str(control_id).unwrap());
    assert!(matches!(
        document.entities[&control_id].geometry,
        Geometry::Curve(Curve::Nurbs(_))
    ));
    controller.undo().unwrap();
    assert_eq!(controller.document(), &after_profile);
    controller.redo().unwrap();
    assert_eq!(controller.document().entities.len(), 6);
    assert!(matches!(
        controller.document().entities[&control_id].geometry,
        Geometry::Curve(Curve::Nurbs(_))
    ));
}

#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_measures_distance_and_angle_through_the_sdk() {
    let mut controller = controller_with_example();
    controller
        .submit("PluginRun org.openmatrix9.hello CreateProfile {}")
        .unwrap();
    let after_profile = controller.document().clone();
    let report = controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "MeasureDemo",
            serde_json::json!({}),
        )
        .unwrap();
    assert_eq!(report["distance"]["millimeters"], 5.0);
    assert_eq!(report["angle"]["degrees"], 90.0);
    // Measuring stages no mutation.
    assert_eq!(controller.document(), &after_profile);
    assert_eq!(controller.coordinator().undo_len(), 1);
}

#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_builds_ellipse_and_rectangle_through_the_sdk() {
    let mut controller = controller_with_example();
    controller
        .submit("PluginRun org.openmatrix9.hello CreateProfile {}")
        .unwrap();
    let after_profile = controller.document().clone();
    let report = controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "CurveShapeDemo",
            serde_json::json!({}),
        )
        .unwrap();
    assert_eq!(report["ellipse_info"]["kind"], "nurbs");
    let sample = report["ellipse_mid"][0]["point"].as_array().unwrap();
    assert!(sample[0].as_f64().unwrap().abs() < 1e-9);
    assert!((sample[1].as_f64().unwrap() - 2.0).abs() < 1e-9);
    assert_eq!(report["rectangle_info"]["kind"], "polycurve");
    assert_eq!(report["rectangle_info"]["segments"], 4);
    assert_eq!(report["rectangle_semantics"]["closed"], true);
    assert_eq!(report["polygon_info"]["kind"], "polycurve");
    assert_eq!(report["polygon_info"]["segments"], 4);
    assert_eq!(report["polygon_semantics"]["closed"], true);
    assert_eq!(report["framed_info"]["kind"], "circle");
    assert_eq!(report["rect_segments"].as_array().unwrap().len(), 4);
    assert_eq!(
        report["rect_segment_mid"][0]["point"],
        serde_json::json!([5.0, 0.0, 0.0])
    );
    assert_eq!(controller.document().entities.len(), 7);
    assert_eq!(controller.coordinator().undo_len(), 2);
    let ellipse_id = report["ellipse"].as_str().unwrap();
    let ellipse_id = ObjectId(uuid::Uuid::parse_str(ellipse_id).unwrap());
    assert!(matches!(
        controller.document().entities[&ellipse_id].geometry,
        Geometry::Curve(Curve::Nurbs(_))
    ));
    let rectangle_id = report["rectangle"].as_str().unwrap();
    let rectangle_id = ObjectId(uuid::Uuid::parse_str(rectangle_id).unwrap());
    assert!(matches!(
        controller.document().entities[&rectangle_id].geometry,
        Geometry::Curve(Curve::Polycurve { .. })
    ));
    controller.undo().unwrap();
    assert_eq!(controller.document(), &after_profile);
    controller.redo().unwrap();
    assert_eq!(controller.document().entities.len(), 7);
}

#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_reads_project_state_and_requests_an_atomic_save() {
    let mut controller = controller_with_example();
    let untitled = controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "ProjectDemo",
            serde_json::json!({}),
        )
        .unwrap();
    assert!(untitled["project"]["path"].is_null());
    assert_eq!(untitled["project"]["dirty"], false);
    // An untitled project refuses the save request before any commit.
    assert!(
        controller
            .run_loaded_plugin(
                "org.openmatrix9.hello",
                "ProjectDemo",
                serde_json::json!({"save": true})
            )
            .is_err()
    );

    controller
        .submit("PluginRun org.openmatrix9.hello CreateProfile {}")
        .unwrap();
    let dirty = controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "ProjectDemo",
            serde_json::json!({}),
        )
        .unwrap();
    assert_eq!(dirty["project"]["dirty"], true);

    // Save the project through the host file workflow, then dirty it again.
    let dir = std::env::temp_dir().join(format!("om9-plugin-save-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("plugin-save.om9");
    assert_eq!(
        controller.begin_file_operation(FileKind::SaveAs).unwrap(),
        FileOutcome::Pending
    );
    let token = controller.file_operation().unwrap().token();
    controller.accept_file_path(token, &path).unwrap();
    assert!(!controller.is_dirty().unwrap());
    controller
        .submit("PluginRun org.openmatrix9.hello CreateProfile {}")
        .unwrap();
    assert!(controller.is_dirty().unwrap());

    let saved = controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "ProjectDemo",
            serde_json::json!({"save": true}),
        )
        .unwrap();
    // The plugin saw the state before the save; the host then saved atomically.
    assert_eq!(saved["project"]["dirty"], true);
    assert!(!controller.is_dirty().unwrap());
    let restored = Archive::open_restored(
        &path,
        &om9_core::budget::Limits::default(),
        &BTreeSet::new(),
    )
    .unwrap();
    assert_eq!(
        restored.document.entities.len(),
        controller.document().entities.len()
    );
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_requests_save_as_and_the_host_dialog_saves_atomically() {
    let mut controller = controller_with_example();
    controller
        .submit("PluginRun org.openmatrix9.hello CreateProfile {}")
        .unwrap();
    assert!(controller.is_dirty().unwrap());
    controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "ProjectDemo",
            serde_json::json!({"save_as": true}),
        )
        .unwrap();
    // The command committed; the host now waits for the Save As destination.
    let operation = controller.file_operation().expect("save-as operation");
    assert_eq!(operation.kind(), Some(FileKind::SaveAs));
    let dir = std::env::temp_dir().join(format!("om9-plugin-save-as-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("plugin-save-as.om9");
    let token = operation.token();
    assert_eq!(
        controller.accept_file_path(token, &path).unwrap(),
        FileOutcome::Saved
    );
    assert!(path.exists());
    assert!(!controller.is_dirty().unwrap());
    let restored = Archive::open_restored(
        &path,
        &om9_core::budget::Limits::default(),
        &BTreeSet::new(),
    )
    .unwrap();
    assert_eq!(
        restored.document.entities.len(),
        controller.document().entities.len()
    );
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_tags_an_object_with_an_extension_slot() {
    let mut controller = controller_with_example();
    let profile = controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "CreateProfile",
            serde_json::json!({}),
        )
        .unwrap();
    let target = profile["objects"][0].as_str().expect("profile object");
    let report = controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "ExtensionDemo",
            serde_json::json!({"id": target}),
        )
        .unwrap();
    let id = report["id"].as_str().expect("tagged object");
    assert_eq!(report["slots"]["org.openmatrix9.demo"]["version"], 1);
    assert_eq!(
        report["slots"]["org.openmatrix9.demo"]["payload"],
        "{\"weight_g\":4.2}"
    );
    let document = controller.document();
    let id = ObjectId(uuid::Uuid::parse_str(id).unwrap());
    let extension = &document.entities[&id].extensions["org.openmatrix9.demo"];
    assert_eq!(extension.version, 1);
    assert_eq!(extension.payload, "{\"weight_g\":4.2}");
    assert_eq!(controller.coordinator().undo_len(), 2);
    controller.undo().unwrap();
    assert!(controller.document().entities[&id].extensions.is_empty());
    controller.redo().unwrap();
    assert_eq!(
        controller.document().entities[&id].extensions["org.openmatrix9.demo"].version,
        1
    );
}

#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_sets_an_object_placement_matrix() {
    let mut controller = controller_with_example();
    let profile = controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "CreateProfile",
            serde_json::json!({}),
        )
        .unwrap();
    let target = profile["objects"][0]
        .as_str()
        .expect("profile object")
        .to_string();
    let before = controller.document().entities[&ObjectId(uuid::Uuid::parse_str(&target).unwrap())]
        .placement;
    controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "PlacementDemo",
            serde_json::json!({"id": target}),
        )
        .unwrap();
    let id = ObjectId(uuid::Uuid::parse_str(&target).unwrap());
    let placement = controller.document().entities[&id].placement;
    assert_eq!(placement.0[0][3], 10.0);
    assert_eq!(placement.0[1][3], 20.0);
    assert_eq!(placement.0[2][3], 30.0);
    assert_eq!(controller.coordinator().undo_len(), 2);
    controller.undo().unwrap();
    assert_eq!(controller.document().entities[&id].placement, before);
    controller.redo().unwrap();
    assert_eq!(controller.document().entities[&id].placement.0[2][3], 30.0);
}

#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_creates_a_mesh_object() {
    let mut controller = controller_with_example();
    let report = controller
        .run_loaded_plugin("org.openmatrix9.hello", "MeshDemo", serde_json::json!({}))
        .unwrap();
    let id = ObjectId(uuid::Uuid::parse_str(report["id"].as_str().unwrap()).unwrap());
    let document = controller.document();
    match &document.entities[&id].geometry {
        Geometry::Mesh(mesh) => {
            assert_eq!(mesh.vertices.len(), 4);
            assert_eq!(mesh.faces, vec![vec![0, 1, 2, 3]]);
        }
        other => panic!("expected a mesh, got {other:?}"),
    }
    assert_eq!(controller.coordinator().undo_len(), 1);
    controller.undo().unwrap();
    assert!(controller.document().entities.is_empty());
    controller.redo().unwrap();
    assert!(matches!(
        controller.document().entities[&id].geometry,
        Geometry::Mesh(_)
    ));
}

#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_reads_the_recorded_change_batch() {
    let mut controller = controller_with_example();
    let empty = controller
        .run_loaded_plugin("org.openmatrix9.hello", "ChangeDemo", serde_json::json!({}))
        .unwrap();
    assert!(empty["change"].is_null());
    controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "CreateProfile",
            serde_json::json!({}),
        )
        .unwrap();
    let created = controller
        .run_loaded_plugin("org.openmatrix9.hello", "ChangeDemo", serde_json::json!({}))
        .unwrap();
    let change = &created["change"];
    assert_eq!(change["cause"], "edit");
    assert!(change["to_revision"].as_u64().unwrap() > change["from_revision"].as_u64().unwrap());
    let changes = change["changes"].as_array().unwrap();
    assert_eq!(changes.len(), 3);
    for entry in changes {
        assert_eq!(entry["kind"], "entity_added");
        assert!(entry["id"].is_string());
    }
    // Reading the feed stages no mutation.
    assert_eq!(controller.coordinator().undo_len(), 1);
}

#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_measures_object_bounds() {
    let mut controller = controller_with_example();
    let profile = controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "CreateProfile",
            serde_json::json!({}),
        )
        .unwrap();
    let line = profile["objects"][0].as_str().expect("profile object");
    let report = controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "BoundsDemo",
            serde_json::json!({"id": line}),
        )
        .unwrap();
    assert_eq!(report["bounds"]["min"], serde_json::json!([0.0, 0.0, 0.0]));
    assert_eq!(report["bounds"]["max"], serde_json::json!([10.0, 0.0, 0.0]));
    assert_eq!(
        report["bounds"]["size"],
        serde_json::json!([10.0, 0.0, 0.0])
    );
    assert_eq!(
        report["bounds"]["center"],
        serde_json::json!([5.0, 0.0, 0.0])
    );
    assert_eq!(controller.coordinator().undo_len(), 1);
}

#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_measures_a_curve_length() {
    let mut controller = controller_with_example();
    let profile = controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "CreateProfile",
            serde_json::json!({}),
        )
        .unwrap();
    let line = profile["objects"][0].as_str().expect("profile object");
    let report = controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "LengthDemo",
            serde_json::json!({"id": line}),
        )
        .unwrap();
    assert_eq!(report["length"]["millimeters"], 10.0);
}

#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_measures_a_circle_radius() {
    let mut controller = controller_with_example();
    let profile = controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "CreateProfile",
            serde_json::json!({}),
        )
        .unwrap();
    let circle = profile["objects"][1].as_str().expect("profile circle");
    let report = controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "RadiusDemo",
            serde_json::json!({"id": circle}),
        )
        .unwrap();
    assert_eq!(report["radius"]["millimeters"], 5.0);
}

#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_counts_objects_by_geometry_kind() {
    let mut controller = controller_with_example();
    controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "CreateProfile",
            serde_json::json!({}),
        )
        .unwrap();
    let report = controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "SummaryDemo",
            serde_json::json!({}),
        )
        .unwrap();
    assert_eq!(report["summary"]["total"], 3);
    assert_eq!(report["summary"]["by_kind"]["Curve"], 3);
}

#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_reports_a_measurement_label() {
    let mut controller = controller_with_example();
    let profile = controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "CreateProfile",
            serde_json::json!({}),
        )
        .unwrap();
    let line = profile["objects"][0].as_str().expect("profile object");
    let report = controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "LabelDemo",
            serde_json::json!({"id": line}),
        )
        .unwrap();
    assert_eq!(report["length"]["millimeters"], 10.0);
    let labelled = controller
        .events
        .events()
        .iter()
        .any(|event| format!("{event:?}").contains("length 10.00 mm"));
    assert!(labelled);
}

#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_reads_layer_order() {
    let mut controller = controller_with_example();
    controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "CreateProfile",
            serde_json::json!({}),
        )
        .unwrap();
    let created = controller
        .run_loaded_plugin("org.openmatrix9.hello", "LayerDemo", serde_json::json!({}))
        .unwrap();
    let layer = created["layer"].as_str().expect("created layer");
    let report = controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "LayerOrderDemo",
            serde_json::json!({}),
        )
        .unwrap();
    let order: Vec<String> = report["order"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().to_string())
        .collect();
    assert!(order.contains(&layer.to_string()));
}

#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_measures_a_mesh_area() {
    let mut controller = controller_with_example();
    let mesh = controller
        .run_loaded_plugin("org.openmatrix9.hello", "MeshDemo", serde_json::json!({}))
        .unwrap();
    let id = mesh["id"].as_str().expect("mesh id");
    let report = controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "AreaDemo",
            serde_json::json!({"id": id}),
        )
        .unwrap();
    assert_eq!(report["area"]["square_millimeters"], 100.0);
}

#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_reads_a_host_undo_in_the_change_feed() {
    let mut controller = controller_with_example();
    controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "CreateProfile",
            serde_json::json!({}),
        )
        .unwrap();
    controller.undo().unwrap();
    let report = controller
        .run_loaded_plugin("org.openmatrix9.hello", "ChangeDemo", serde_json::json!({}))
        .unwrap();
    let change = &report["change"];
    assert_eq!(change["cause"], "undo");
    let changes = change["changes"].as_array().unwrap();
    assert_eq!(changes.len(), 3);
    for entry in changes {
        assert_eq!(entry["kind"], "entity_removed");
    }
}

#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_lists_document_groups() {
    let mut controller = controller_with_example();
    controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "CreateProfile",
            serde_json::json!({}),
        )
        .unwrap();
    let report = controller
        .run_loaded_plugin("org.openmatrix9.hello", "GroupsDemo", serde_json::json!({}))
        .unwrap();
    // The example document has no group records yet; the command still round-trips.
    assert_eq!(report["groups"].as_array().unwrap().len(), 0);
}

#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_lists_block_definitions() {
    let mut controller = controller_with_example();
    controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "CreateProfile",
            serde_json::json!({}),
        )
        .unwrap();
    let report = controller
        .run_loaded_plugin("org.openmatrix9.hello", "BlocksDemo", serde_json::json!({}))
        .unwrap();
    assert_eq!(report["definitions"].as_array().unwrap().len(), 0);
}

#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_clears_the_current_layer() {
    let mut controller = controller_with_example();
    controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "CreateProfile",
            serde_json::json!({}),
        )
        .unwrap();
    controller
        .run_loaded_plugin("org.openmatrix9.hello", "LayerDemo", serde_json::json!({}))
        .unwrap();
    assert!(controller.document().current_layer.is_some());
    controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "ClearLayerDemo",
            serde_json::json!({}),
        )
        .unwrap();
    assert!(controller.document().current_layer.is_none());
}

#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_reads_an_object_placement() {
    let mut controller = controller_with_example();
    let profile = controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "CreateProfile",
            serde_json::json!({}),
        )
        .unwrap();
    let id = profile["objects"][0].as_str().expect("profile object");
    let report = controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "PlacementReadDemo",
            serde_json::json!({"id": id}),
        )
        .unwrap();
    let matrix = report["placement"].as_array().unwrap();
    assert_eq!(matrix.len(), 4);
    assert_eq!(matrix[0][0], 1.0);
    assert_eq!(matrix[0][3], 0.0);
}

#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_reads_one_object_record() {
    let mut controller = controller_with_example();
    let profile = controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "CreateProfile",
            serde_json::json!({}),
        )
        .unwrap();
    let id = profile["objects"][0].as_str().expect("profile object");
    let report = controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "InfoDemo",
            serde_json::json!({"id": id}),
        )
        .unwrap();
    assert_eq!(report["info"]["id"], id);
    assert_eq!(report["info"]["geometry_kind"], "Curve");
    assert!(report["info"]["placement"][3][3].as_f64().unwrap() == 1.0);
}

#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_creates_a_group() {
    let mut controller = controller_with_example();
    let profile = controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "CreateProfile",
            serde_json::json!({}),
        )
        .unwrap();
    let members = profile["objects"].as_array().unwrap()[..2].to_vec();
    let report = controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "GroupCreateDemo",
            serde_json::json!({"members": members}),
        )
        .unwrap();
    let group = report["group"].as_str().expect("group id");
    let document = controller.document();
    assert_eq!(document.groups.len(), 1);
    let created = document
        .groups
        .values()
        .find(|g| g.id.to_string() == group)
        .expect("group");
    assert_eq!(created.members.len(), 2);
    assert_eq!(controller.coordinator().undo_len(), 2);
    controller.undo().unwrap();
    assert!(controller.document().groups.is_empty());
}

#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_deletes_a_group() {
    let mut controller = controller_with_example();
    let profile = controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "CreateProfile",
            serde_json::json!({}),
        )
        .unwrap();
    let members = profile["objects"].as_array().unwrap()[..1].to_vec();
    let created = controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "GroupCreateDemo",
            serde_json::json!({"members": members}),
        )
        .unwrap();
    let group = created["group"].as_str().expect("group id");
    controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "GroupDeleteDemo",
            serde_json::json!({"id": group}),
        )
        .unwrap();
    assert!(controller.document().groups.is_empty());
    assert_eq!(controller.document().entities.len(), 3);
    controller.undo().unwrap();
    assert_eq!(controller.document().groups.len(), 1);
}

#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_reads_one_layer_record() {
    let mut controller = controller_with_example();
    controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "CreateProfile",
            serde_json::json!({}),
        )
        .unwrap();
    let created = controller
        .run_loaded_plugin("org.openmatrix9.hello", "LayerDemo", serde_json::json!({}))
        .unwrap();
    let layer = created["layer"].as_str().expect("layer id");
    let report = controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "LayerInfoDemo",
            serde_json::json!({"id": layer}),
        )
        .unwrap();
    assert_eq!(report["layer"]["id"], layer);
    assert_eq!(report["layer"]["current"], true);
    assert_eq!(report["layer"]["locked"], false);
}

#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_sets_the_document_tolerance() {
    let mut controller = controller_with_example();
    controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "ToleranceDemo",
            serde_json::json!({}),
        )
        .unwrap();
    let tolerance = controller
        .document()
        .effective_tolerance()
        .expect("tolerance");
    assert_eq!(tolerance.absolute_mm, 0.01);
    assert_eq!(controller.coordinator().undo_len(), 1);
    controller.undo().unwrap();
    let restored = controller
        .document()
        .effective_tolerance()
        .expect("restored tolerance");
    assert_eq!(restored.absolute_mm, 0.001);
}

#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_reports_no_provenance_for_a_native_object() {
    let mut controller = controller_with_example();
    let profile = controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "CreateProfile",
            serde_json::json!({}),
        )
        .unwrap();
    let id = profile["objects"][0].as_str().expect("profile object");
    let report = controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "ProvenanceDemo",
            serde_json::json!({"id": id}),
        )
        .unwrap();
    assert!(report["provenance"].is_null());
}

#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_reports_pickability() {
    let mut controller = controller_with_example();
    let profile = controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "CreateProfile",
            serde_json::json!({}),
        )
        .unwrap();
    let id = profile["objects"][0].as_str().expect("profile object");
    let visible = controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "PickableDemo",
            serde_json::json!({"id": id}),
        )
        .unwrap();
    assert_eq!(visible["pickable"], true);
    controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "AttributesDemo",
            serde_json::json!({"id": id}),
        )
        .unwrap();
    let after = controller
        .run_loaded_plugin(
            "org.openmatrix9.hello",
            "PickableDemo",
            serde_json::json!({"id": id}),
        )
        .unwrap();
    assert_eq!(after["pickable"], true);
}
