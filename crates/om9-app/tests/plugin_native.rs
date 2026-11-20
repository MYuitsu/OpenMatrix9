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

#[cfg(feature = "occt-brep")]
#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_project_uses_current_plane_after_stale_line() {
    use om9_core::model::Entity;
    use om9_scene::{camera::{CPlane, Viewport}, workspace::ViewId};

    let mut document = Document::with_template_tolerance("SDK Project after stale point prompt");
    let start = [5., 2., 7.];
    let line = Entity::new("source", Geometry::Curve(Curve::Line {
        start, end: [5., 8., 7.], domain: [0., 1.],
    }), None);
    let source = line.id;
    let edit_target = Entity::new("unrelated point", Geometry::Point([0.; 3]), None);
    let edit_id = edit_target.id;
    for entity in [line, edit_target] {
        document.entities.insert(entity.id, entity);
    }
    let mut controller = Controller::new(document, Limits::default()).unwrap();
    controller.workspace.active = ViewId::LookingDown;
    load_example(&mut controller);
    controller.submit("Line").unwrap();
    assert!(controller.is_running());
    // Publishing a model edit invalidates the old command's captured plane.
    assert_eq!(controller.delete_objects(&[edit_id]).unwrap(), 1);
    assert!(!controller.is_running());
    controller.set_cplane(CPlane::world_xy().with_origin([0., 0., 2.]).unwrap()).unwrap();
    controller.set_project(true);
    controller.set_planar(true);

    let before = controller.coordinator().snapshot();
    let viewport = Viewport { origin: [0., 0.], size: [800., 600.] };
    let [x, y] = controller.workspace.views[&ViewId::LookingDown].camera
        .project(start, viewport).unwrap().unwrap().screen;
    let result = controller.run_loaded_plugin("org.openmatrix9.hello", "SnapDemo",
        serde_json::json!({"kind": "end", "cursor_x": x, "cursor_y": y})).unwrap();
    assert_eq!(result["hit"]["root_id"], source.to_string());
    assert_eq!(result["hit"]["point_mm"], serde_json::json!(start));
    assert_eq!(result["point_mm"], serde_json::json!([5., 2., 2.]));
    assert_eq!(controller.coordinator().snapshot(), before);
}

#[cfg(feature = "occt-brep")]
#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_smarttrack_two_anchors_follow_rotated_cplane() {
    use om9_scene::{camera::{CPlane, Viewport}, workspace::ViewId};
    let mut controller = Controller::new(Document::with_template_tolerance("SDK rotated SmartTrack"), Limits::default()).unwrap();
    controller.workspace.active = ViewId::LookingDown;
    let q = std::f64::consts::FRAC_1_SQRT_2;
    controller.set_cplane(CPlane::new([2., -1., 0.], [q, q, 0.], [-q, q, 0.]).unwrap()).unwrap();
    load_example(&mut controller);
    let before = controller.document().clone();
    let viewport = Viewport { origin: [0., 0.], size: [800., 600.] };
    let cursor = controller.workspace.views[&ViewId::LookingDown].camera.project([2.04, 2.06, 0.], viewport).unwrap().unwrap().screen;
    let query = serde_json::json!({"kind": "smarttrack", "tracking_anchors_mm": [[0., 0., 0.], [4., 0., 0.]],
        "cursor_x": cursor[0], "cursor_y": cursor[1]});
    let result = controller.run_loaded_plugin("org.openmatrix9.hello", "SnapDemo", query).unwrap();
    assert_eq!(result["hit"]["kind"], "SmartTrack");
    assert!(result["hit"]["root_id"].is_null());
    let point = result["point_mm"].as_array().unwrap();
    assert!((point[0].as_f64().unwrap() - 2.).abs() < 1e-6 && (point[1].as_f64().unwrap() - 2.).abs() < 1e-6);
    assert_eq!(controller.document(), &before);
    assert_eq!(controller.coordinator().undo_len(), 0);
}

#[cfg(feature = "occt-brep")]
#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_smarttrack_guide_curve_query_uses_source_identity() {
    use om9_core::model::Entity;
    use om9_scene::{camera::Viewport, workspace::ViewId};
    let mut document = Document::with_template_tolerance("SDK SmartTrack curve");
    let line = Entity::new("source line", Geometry::Curve(Curve::Line {
        start: [3., -5., 0.], end: [3., 5., 0.], domain: [0., 10.],
    }), None);
    let id = line.id;
    document.entities.insert(id, line);
    let mut controller = Controller::new(document, Limits::default()).unwrap();
    controller.workspace.active = ViewId::LookingDown;
    load_example(&mut controller);
    let before = controller.document().clone();
    let viewport = Viewport { origin: [0., 0.], size: [800., 600.] };
    let cursor = controller.workspace.views[&ViewId::LookingDown].camera.project([3., 0., 0.], viewport).unwrap().unwrap().screen;
    let query = serde_json::json!({"kind": "smarttrack", "tracking_anchors_mm": [[0., 0., 0.]],
        "cursor_x": cursor[0], "cursor_y": cursor[1], "create_point": true});
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    let error = controller.run_loaded_plugin("org.openmatrix9.hello", "SnapDemo", query.clone()).unwrap_err();
    assert!(error.to_string().contains("pending"), "{error}");
    let alternate = serde_json::json!({"kind": "smarttrack", "tracking_anchors_mm": [[0., 5., 0.]],
        "cursor_x": cursor[0], "cursor_y": cursor[1], "create_point": true});
    loop {
        match controller.run_loaded_plugin("org.openmatrix9.hello", "SnapDemo", alternate.clone()) {
            Ok(result) => { assert!(result["hit"].is_null(), "old guides must not be reused"); break; }
            Err(error) if error.to_string().contains("pending") => {
                assert!(std::time::Instant::now() < deadline);
                assert_eq!(controller.document(), &before);
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            Err(error) => panic!("changed SmartTrack guides: {error}"),
        }
    }
    assert_eq!(controller.document(), &before);
    let mut pending = 1;
    let result = loop {
        match controller.run_loaded_plugin("org.openmatrix9.hello", "SnapDemo", query.clone()) {
            Ok(result) => break result,
            Err(error) if error.to_string().contains("pending") => {
                pending += 1;
                assert!(std::time::Instant::now() < deadline);
                assert_eq!(controller.document(), &before);
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            Err(error) => panic!("SmartTrack query: {error}"),
        }
    };
    assert!(pending > 0);
    assert_eq!(result["hit"]["kind"], "SmartTrack");
    assert_eq!(result["hit"]["root_id"], id.to_string());
    let point = result["point_mm"].as_array().unwrap();
    assert!((point[0].as_f64().unwrap() - 3.).abs() < 1e-6 && point[1].as_f64().unwrap().abs() < 1e-6);
    assert!(result["created"].is_string());
    controller.undo().unwrap(); assert_eq!(controller.document(), &before);
}

#[cfg(feature = "occt-brep")]
#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_directional_nurbs_snaps_require_a_base_and_commit_source_point() {
    use om9_core::{curve_ops::{derivative_at, point_at}, geometry::{NurbsCurve, Transform}, model::Entity};
    use om9_scene::{camera::Viewport, workspace::ViewId};

    let curve = Curve::Nurbs(NurbsCurve {
        degree: 2, poles: vec![[-1., 0., 0.], [0., 2., 0.], [1., 0., 0.]],
        weights: vec![1.; 3], knots: vec![0., 0., 0., 1., 1., 1.],
        domain: [0., 1.], periodic: false, reversed: false, seam: None,
    });
    let placement = Transform([
        [0., -1., 0., 6.], [2., 0., 0., 2.], [0., 0., 1., 0.], [0., 0., 0., 1.],
    ]);
    let mut entity = Entity::new("directional source", Geometry::Curve(curve.clone()), None);
    entity.placement = placement;
    let source_id = entity.id;
    let mut document = Document::with_template_tolerance("SDK directional NURBS snaps");
    document.entities.insert(source_id, entity);
    let limits = Limits::default();
    let mut controller = Controller::new(document, limits).unwrap();
    controller.workspace.active = ViewId::LookingDown;
    load_example(&mut controller);
    let before = controller.document().clone();
    let tolerance = before.effective_tolerance().unwrap().absolute_mm;
    let viewport = Viewport { origin: [0.; 2], size: [800., 600.] };
    let expected = placement.point(point_at(&curve, 0.5, &limits).unwrap()).unwrap();
    let cursor = controller.workspace.views[&ViewId::LookingDown].camera
        .project(expected, viewport).unwrap().unwrap().screen;
    let run = |controller: &mut Controller, query| {
        controller.run_loaded_plugin("org.openmatrix9.hello", "SnapDemo", query)
    };

    for (kind, label, base) in [
        ("tangent", "Tangent", [5., 4., 0.]),
        ("perpendicular", "Perpendicular", [3., 2., 0.]),
    ] {
        let mut query = serde_json::json!({"kind": kind, "create_point": true,
            "cursor_x": cursor[0], "cursor_y": cursor[1]});
        let missing_base = run(&mut controller, query.clone()).unwrap();
        assert!(missing_base["hit"].is_null() && missing_base["created"].is_null());
        assert_eq!(controller.document(), &before);

        query["from_mm"] = serde_json::json!(base);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        let mut pending = false;
        let result = loop {
            match run(&mut controller, query.clone()) {
                Ok(result) => break result,
                Err(error) if error.to_string().contains("pending") => {
                    pending = true;
                    assert_eq!(controller.document(), &before);
                    assert!(std::time::Instant::now() < deadline);
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
                Err(error) => panic!("{kind} snap: {error}"),
            }
        };
        assert!(pending, "expected native source work for {kind}");
        assert_eq!(result["hit"]["kind"], label);
        assert_eq!(result["hit"]["root_id"], source_id.to_string());
        assert_eq!(result["point_mm"], result["hit"]["point_mm"]);
        let point: [f64; 3] = serde_json::from_value(result["point_mm"].clone()).unwrap();
        let parameter = point[1] / 4.; // The fixture's world Y equals 4t.
        let witness = placement.point(point_at(&curve, parameter, &limits).unwrap()).unwrap();
        assert!((0..3).all(|axis| (point[axis] - witness[axis]).abs() <= tolerance));
        let local = derivative_at(&curve, parameter, &limits).unwrap();
        let tangent = [-local[1], 2. * local[0]];
        let offset = [point[0] - base[0], point[1] - base[1]];
        let residual = if kind == "tangent" {
            offset[0] * tangent[1] - offset[1] * tangent[0]
        } else {
            offset[0] * tangent[0] + offset[1] * tangent[1]
        }.abs() / tangent[0].hypot(tangent[1]);
        assert!(residual <= tolerance, "{kind} residual {residual}");
        let created = result["created"].as_str().expect("created point");
        assert!(controller.document().entities.values().any(|entity|
            entity.id.to_string() == created && matches!(entity.geometry, Geometry::Point(p) if p == point)));
        assert_eq!(controller.coordinator().undo_len(), 1);
        controller.undo().unwrap();
        assert_eq!(controller.document(), &before);
    }
}

#[cfg(feature = "occt-brep")]
#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_smarttrack_reuses_a_source_anchor_until_its_scene_changes() {
    use om9_core::model::Entity;
    use om9_scene::{camera::Viewport, workspace::ViewId};

    let mut document = Document::with_template_tolerance("SDK source-tracked anchor");
    let anchor = Entity::new("anchor", Geometry::Point([0.; 3]), None);
    let anchor_id = anchor.id;
    document.entities.insert(anchor_id, anchor);
    let curve = Entity::new("guide crossing", Geometry::Curve(Curve::Line {
        start: [3., -5., 0.], end: [3., 5., 0.], domain: [0., 10.],
    }), None);
    let curve_id = curve.id;
    document.entities.insert(curve_id, curve);
    let mut controller = Controller::new(document, Limits::default()).unwrap();
    controller.workspace.active = ViewId::LookingDown;
    load_example(&mut controller);
    let before = controller.document().clone();
    let viewport = Viewport { origin: [0., 0.], size: [800., 600.] };
    let camera = &controller.workspace.views[&ViewId::LookingDown].camera;
    let cursor = |world| camera.project(world, viewport).unwrap().unwrap().screen;
    let anchor_cursor = cursor([0.; 3]);
    let guide_cursor = cursor([3., 0., 0.]);
    let anchor_query = serde_json::json!({"kind": "point", "retain_anchor": true,
        "cursor_x": anchor_cursor[0], "cursor_y": anchor_cursor[1]});
    let run = |controller: &mut Controller, query| {
        controller.run_loaded_plugin("org.openmatrix9.hello", "SnapDemo", query)
    };
    let result = run(&mut controller, anchor_query.clone()).unwrap();
    assert_eq!(result["hit"]["root_id"], anchor_id.to_string());
    let anchor = result["anchor"].as_str().expect("tracked anchor token");
    let query = serde_json::json!({"kind": "smarttrack", "tracking_anchors": [anchor],
        "cursor_x": guide_cursor[0], "cursor_y": guide_cursor[1]});

    let error = run(&mut controller, query.clone()).unwrap_err();
    assert!(error.to_string().contains("pending"), "{error}");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    let tracked = loop {
        match run(&mut controller, query.clone()) {
            Ok(result) => break result,
            Err(error) if error.to_string().contains("pending") => {
                assert!(std::time::Instant::now() < deadline);
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            Err(error) => panic!("tracked snap query failed: {error}"),
        }
    };
    assert_eq!(tracked["hit"]["root_id"], curve_id.to_string());
    let point: [f64; 3] = serde_json::from_value(tracked["point_mm"].clone()).unwrap();
    assert!((0..3).all(|axis| (point[axis] - [3., 0., 0.][axis]).abs() <= 1e-9), "{point:?}");
    assert_eq!(tracked["point_mm"], tracked["hit"]["point_mm"]);
    assert_eq!(controller.document(), &before);
    assert_eq!(controller.coordinator().undo_len(), 0);

    controller.select_objects(&[anchor_id]).unwrap();
    controller.numeric_move([0., 20., 0.]).unwrap();
    let moved = controller.document().clone();
    let mut commit_with_stale_anchor = query;
    commit_with_stale_anchor["create_point"] = serde_json::json!(true);
    let error = run(&mut controller, commit_with_stale_anchor.clone()).unwrap_err();
    assert!(error.to_string().contains("SmartTrack anchor expired"), "{error}");
    assert_eq!(controller.document(), &moved);
    assert_eq!(controller.coordinator().undo_len(), 1);
    controller.undo().unwrap();
    assert_eq!(controller.document(), &before);
    let error = run(&mut controller, commit_with_stale_anchor.clone()).unwrap_err();
    assert!(error.to_string().contains("SmartTrack anchor expired"), "{error}");
    let renewed = run(&mut controller, anchor_query).unwrap();
    assert_ne!(renewed["anchor"].as_str().expect("renewed token"), anchor);

    commit_with_stale_anchor["tracking_anchors"] = serde_json::json!([renewed["anchor"]]);
    let plane = controller.workspace.views[&ViewId::LookingDown].cplane;
    controller.set_cplane(plane.with_origin([0., 0., 2.]).unwrap()).unwrap();
    let error = run(&mut controller, commit_with_stale_anchor).unwrap_err();
    assert!(error.to_string().contains("SmartTrack anchor expired"), "{error}");
    assert_eq!(controller.document(), &before);
}

#[cfg(feature = "occt-brep")]
#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_smarttrack_retains_the_curve_end_direction() {
    use om9_core::model::Entity;
    use om9_scene::{camera::Viewport, workspace::ViewId};
    use std::f64::consts::{FRAC_1_SQRT_2, FRAC_PI_2, FRAC_PI_4};

    let mut document = Document::with_template_tolerance("SDK End direction");
    let entity = Entity::new("arc", Geometry::Curve(Curve::Circle {
        center: [0., 0., 12.], x: [1., 0., 0.], y: [0., 1., 0.], radius: 4.,
        domain: [0., 1.], angle_domain: Some([FRAC_PI_4, FRAC_PI_2]),
    }), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut controller = Controller::new(document, Limits::default()).unwrap();
    controller.workspace.active = ViewId::LookingDown;
    load_example(&mut controller);
    let before = controller.document().clone();
    let viewport = Viewport { origin: [0.; 2], size: [800., 600.] };
    let project = |controller: &Controller, point| controller.workspace.views[&ViewId::LookingDown]
        .camera.project(point, viewport).unwrap().unwrap().screen;
    let diagonal = 4. * FRAC_1_SQRT_2;
    let at = project(&controller, [diagonal, diagonal, 12.]);
    let acquired = controller.run_loaded_plugin("org.openmatrix9.hello", "SnapDemo", serde_json::json!({
        "kind": "end", "cursor_x": at[0], "cursor_y": at[1], "retain_anchor": true,
    })).unwrap();
    assert_eq!(acquired["hit"]["root_id"], id.to_string());
    let anchor = acquired["anchor"].as_str().expect("issued End anchor");

    let expected = [diagonal + 6., diagonal - 6., 12.];
    let cursor = project(&controller, expected);
    let query = serde_json::json!({"kind": "smarttrack", "tracking_anchors": [anchor],
        "cursor_x": cursor[0], "cursor_y": cursor[1]});
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
    let result = loop {
        match controller.run_loaded_plugin("org.openmatrix9.hello", "SnapDemo", query.clone()) {
            Ok(result) => break result,
            Err(error) if error.to_string().contains("pending") => {
                assert!(std::time::Instant::now() < deadline, "SDK SmartTrack timed out");
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            Err(error) => panic!("SDK SmartTrack: {error}"),
        }
    };
    assert_eq!(result["hit"]["kind"], "SmartTrack");
    assert!(result["hit"]["root_id"].is_null());
    let point: [f64; 3] = serde_json::from_value(result["point_mm"].clone()).unwrap();
    assert!(point.iter().zip(expected).all(|(a, b)| (a - b).abs() < 1e-6));
    assert_eq!(controller.document(), &before);
}

#[cfg(feature = "occt-brep")]
#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_smarttrack_free_guide_does_not_claim_a_source() {
    use om9_scene::{camera::Viewport, workspace::ViewId};
    let mut controller = Controller::new(Document::with_template_tolerance("SDK SmartTrack guide"), Limits::default()).unwrap();
    controller.workspace.active = ViewId::Perspective;
    load_example(&mut controller);
    let viewport = Viewport { origin: [0., 0.], size: [800., 600.] };
    let expected = [12., 0., 12.];
    let cursor = controller.workspace.views[&ViewId::Perspective].camera.project(expected, viewport).unwrap().unwrap().screen;
    let query = serde_json::json!({"kind": "smarttrack", "tracking_anchors_mm": [[0., 0., 12.]],
        "cursor_x": cursor[0], "cursor_y": cursor[1]});
    let result = controller.run_loaded_plugin("org.openmatrix9.hello", "SnapDemo", query.clone()).unwrap();
    assert_eq!(result["hit"]["kind"], "SmartTrack");
    assert!(result["hit"]["root_id"].is_null());
    let point = result["point_mm"].as_array().unwrap();
    assert!(point.iter().zip(expected).all(|(actual, expected)|
        (actual.as_f64().unwrap() - expected).abs() < 1e-6), "guide point was {point:?}");
    assert!(controller.document().entities.is_empty());
    let mut retain = query;
    retain["retain_anchor"] = serde_json::json!(true);
    let error = controller.run_loaded_plugin("org.openmatrix9.hello", "SnapDemo", retain).unwrap_err();
    assert!(error.to_string().contains("source-backed snap hit"), "{error}");
    for anchors in [serde_json::json!([]), serde_json::json!([[0., 0., 0.], [1., 0., 0.], [2., 0., 0.]])] {
        let invalid = serde_json::json!({"kind": "smarttrack", "tracking_anchors_mm": anchors,
            "cursor_x": cursor[0], "cursor_y": cursor[1], "create_point": true});
        assert!(controller.run_loaded_plugin("org.openmatrix9.hello", "SnapDemo", invalid).is_err());
        assert!(controller.document().entities.is_empty());
    }

    let mut perpendicular_query = serde_json::json!({
        "kinds": ["perpendicular", "smarttrack"],
        "tracking_anchors_mm": [[0., 0., 12.]],
        "from_mm": [12., 4., 0.],
        "cursor_x": cursor[0],
        "cursor_y": cursor[1],
    });
    let result = controller.run_loaded_plugin("org.openmatrix9.hello", "SnapDemo", perpendicular_query.clone()).unwrap();
    assert_eq!(result["hit"]["kind"], "Perpendicular");
    assert_eq!(result["point_mm"], serde_json::json!(expected));
    assert!(result["hit"]["root_id"].is_null());
    perpendicular_query["retain_anchor"] = serde_json::json!(true);
    let error = controller.run_loaded_plugin("org.openmatrix9.hello", "SnapDemo", perpendicular_query).unwrap_err();
    assert!(error.to_string().contains("source-backed snap hit"), "{error}");
    assert!(controller.document().entities.is_empty());
}

#[cfg(feature = "occt-brep")]
#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_surface_pending_is_invalidated_by_source_move() {
    use om9_core::budget::memory::MemoryClass;
    use om9_scene::{camera::Viewport, workspace::ViewId};
    let mut controller = Controller::new(Document::with_template_tolerance("plugin stale surface"), Limits::default()).unwrap();
    controller.solid_sphere_points([0.; 3], 5.).unwrap();
    let id = *controller.document().entities.keys().next().unwrap();
    controller.workspace.active = ViewId::LookingDown;
    load_example(&mut controller);
    let viewport = Viewport { origin: [0., 0.], size: [800., 600.] };
    let cursor = controller.workspace.views[&ViewId::LookingDown].camera.project([2., 1., 20_f64.sqrt()], viewport).unwrap().unwrap().screen;
    let query = serde_json::json!({"kind": "surface", "cursor_x": cursor[0], "cursor_y": cursor[1], "create_point": true});
    let baseline = controller.coordinator().external_reserved_bytes(MemoryClass::Kernel);
    let permit = controller.coordinator().external_memory_account().try_acquire_surface_snap().unwrap();
    let error = controller.run_loaded_plugin("org.openmatrix9.hello", "SnapDemo", query.clone()).unwrap_err();
    assert!(error.to_string().contains("pending"), "{error}");
    assert!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel) > baseline);
    controller.select_objects(&[id]).unwrap();
    controller.numeric_move([100., 0., 0.]).unwrap();
    assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel), baseline,
        "publishing the move must clear the queued snap without a later plugin call");
    drop(permit);
    let moved = controller.document().clone();
    let undo_after_move = controller.coordinator().undo_len();
    let miss = controller.run_loaded_plugin("org.openmatrix9.hello", "SnapDemo", query.clone()).unwrap();
    assert!(miss["hit"].is_null(), "old surface hit must not survive the source move");
    assert_eq!(controller.document(), &moved);
    assert_eq!(controller.coordinator().undo_len(), undo_after_move);
    controller.undo().unwrap();
    let restored = controller.document().clone();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    let result = loop {
        match controller.run_loaded_plugin("org.openmatrix9.hello", "SnapDemo", query.clone()) {
            Ok(result) => break result,
            Err(error) if error.to_string().contains("pending") => {
                assert!(std::time::Instant::now() < deadline);
                assert_eq!(controller.document(), &restored);
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            Err(error) => panic!("restored source query: {error}"),
        }
    };
    assert_eq!(result["hit"]["root_id"], id.to_string());
    let values = result["point_mm"].as_array().unwrap();
    assert!((values[2].as_f64().unwrap() - 20_f64.sqrt()).abs() < 1e-6);
    assert!(result["created"].is_string());
}

#[cfg(feature = "occt-brep")]
#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_surface_miss_cancels_and_drains_previous_request() {
    use om9_core::budget::memory::MemoryClass;
    use om9_scene::{camera::Viewport, workspace::ViewId};
    let mut controller = Controller::new(Document::with_template_tolerance("plugin cancel surface"), Limits::default()).unwrap();
    controller.solid_sphere_points([0.; 3], 5.).unwrap();
    controller.workspace.active = ViewId::LookingDown;
    load_example(&mut controller);
    let before = controller.document().clone();
    let baseline = controller.coordinator().external_reserved_bytes(MemoryClass::Kernel);
    let viewport = Viewport { origin: [0., 0.], size: [800., 600.] };
    let cursor = controller.workspace.views[&ViewId::LookingDown].camera.project([2., 1., 20_f64.sqrt()], viewport).unwrap().unwrap().screen;
    let query = serde_json::json!({"kind": "surface", "cursor_x": cursor[0], "cursor_y": cursor[1], "create_point": true});
    let error = controller.run_loaded_plugin("org.openmatrix9.hello", "SnapDemo", query).unwrap_err();
    assert!(error.to_string().contains("pending"), "{error}");
    assert!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel) > baseline);
    let miss = serde_json::json!({"kind": "surface", "cursor_x": 0., "cursor_y": 0., "create_point": true});
    let result = controller.run_loaded_plugin("org.openmatrix9.hello", "SnapDemo", miss).unwrap();
    assert!(result["hit"].is_null());
    assert_eq!(controller.document(), &before);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while controller.coordinator().external_reserved_bytes(MemoryClass::Kernel) != baseline {
        assert!(std::time::Instant::now() < deadline, "canceled worker reservation did not drain");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(controller.document(), &before);
}

#[cfg(feature = "occt-brep")]
#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_intersection_follows_shaded_visibility() {
    use om9_core::model::{Definition, Entity};
    use om9_scene::{camera::Viewport, workspace::{DisplayMode, ViewId}};

    let mut document = Document::with_template_tolerance("SDK intersection visibility");
    let first = Entity::new("first", Geometry::Curve(Curve::Line {
        start: [-5., 0., 0.], end: [5., 0., 0.], domain: [0., 1.],
    }), None);
    let second = Entity::new("second", Geometry::Curve(Curve::Line {
        start: [0., -5., 0.], end: [0., 5., 0.], domain: [0., 1.],
    }), None);
    let leaves = [first.id.to_string(), second.id.to_string()];
    let definition = Definition::new("crossing lines", vec![first, second]);
    let instance = Entity::new("block", Geometry::Instance(definition.id), None);
    let root = instance.id.to_string();
    document.definitions.insert(definition.id, definition);
    let mesh = Entity::new("occluder", Geometry::Mesh(om9_core::geometry::Mesh {
        vertices: vec![[-3., -3., 5.], [3., -3., 5.], [3., 3., 5.], [-3., 3., 5.]],
        faces: vec![vec![0, 1, 2, 3]],
        normals: vec![], colors: vec![], corner_uvs: vec![],
    }), None);
    for entity in [instance, mesh] {
        document.entities.insert(entity.id, entity);
    }
    let mut controller = Controller::new(document, Limits::default()).unwrap();
    controller.workspace.active = ViewId::LookingDown;
    load_example(&mut controller);
    let before = controller.coordinator().snapshot();
    let viewport = Viewport { origin: [0., 0.], size: [800., 600.] };
    let [x, y] = controller.workspace.views[&ViewId::LookingDown].camera
        .project([0.; 3], viewport).unwrap().unwrap().screen;
    for (mode, kind) in [
        (DisplayMode::Wireframe, "intersection"),
        (DisplayMode::Wireframe, "apparent_intersection"),
        (DisplayMode::Shaded, "intersection"),
        (DisplayMode::Shaded, "apparent_intersection"),
    ] {
        let query = serde_json::json!({"kind": kind, "cursor_x": x, "cursor_y": y});
        controller.workspace.views.get_mut(&ViewId::LookingDown).unwrap().mode = mode;
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let answer = loop {
            match controller.run_loaded_plugin("org.openmatrix9.hello", "SnapDemo", query.clone()) {
                Ok(answer) => break answer,
                Err(error) if error.to_string().contains("pending") => {
                    assert!(std::time::Instant::now() < deadline, "{error}");
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
                Err(error) => panic!("intersection query in {mode:?}: {error}"),
            }
        };
        if mode == DisplayMode::Wireframe {
            assert_eq!(answer["hit"]["kind"], "Intersection");
            assert_eq!(answer["point_mm"], serde_json::json!([0., 0., 0.]));
            let first_source = &answer["hit"];
            let second_source = &answer["hit"]["second_source"];
            assert_eq!(first_source["root_id"], root);
            assert_eq!(second_source["root_id"], root);
            let first = first_source["leaf_id"].as_str().unwrap();
            let second = second_source["leaf_id"].as_str().unwrap();
            assert_ne!(first, second);
            assert!(leaves.iter().any(|id| id == first) && leaves.iter().any(|id| id == second));
            assert_eq!(first_source["member_path"], serde_json::json!([first]));
            assert_eq!(second_source["member_path"], serde_json::json!([second]));
        } else {
            assert!(answer["hit"].is_null(), "{answer}");
        }
        assert_eq!(controller.coordinator().snapshot(), before);
    }
}

#[cfg(feature = "occt-brep")]
#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_apparent_int_checks_source_visibility_before_project() {
    use om9_core::model::Entity;
    use om9_scene::{camera::Viewport, workspace::{DisplayMode, ViewId}};
    use serde_json::json;

    for (cover_z, visible) in [(5., true), (10., false)] {
        let mut document = Document::with_template_tolerance("SDK apparent intersection visibility");
        for (start, end) in [([-5., 0., 0.], [5., 0., 0.]), ([0., -5., 8.], [0., 5., 8.])] {
            let line = Entity::new(
                "crossing",
                Geometry::Curve(Curve::Line { start, end, domain: [0., 1.] }),
                None,
            );
            document.entities.insert(line.id, line);
        }
        let mesh = Entity::new("occluder", Geometry::Mesh(om9_core::geometry::Mesh {
            vertices: vec![[-3., -3., cover_z], [3., -3., cover_z], [3., 3., cover_z], [-3., 3., cover_z]],
            faces: vec![vec![0, 1, 2, 3]],
            normals: vec![], colors: vec![], corner_uvs: vec![],
        }), None);
        document.entities.insert(mesh.id, mesh);
        let mut controller = Controller::new(document, Limits::default()).unwrap();
        controller.workspace.active = ViewId::LookingDown;
        controller.set_project(true);
        let view = controller.workspace.views.get_mut(&ViewId::LookingDown).unwrap();
        view.mode = DisplayMode::Shaded;
        let viewport = Viewport { origin: [0., 0.], size: [800., 600.] };
        let [x, y] = view.camera.project([0., 0., 8.], viewport).unwrap().unwrap().screen;
        load_example(&mut controller);
        let before = controller.coordinator().snapshot();
        let query = json!({ "kind": "apparent_intersection", "cursor_x": x, "cursor_y": y });
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let answer = loop {
            match controller.run_loaded_plugin("org.openmatrix9.hello", "SnapDemo", query.clone()) {
                Ok(answer) => break answer,
                Err(error) if error.to_string().contains("pending") => {
                    assert!(std::time::Instant::now() < deadline, "{error}");
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
                Err(error) => panic!("apparent intersection under cover at Z={cover_z}: {error}"),
            }
        };
        if visible {
            assert_eq!(answer["hit"]["kind"], "Apparent Intersection");
            assert_eq!(answer["hit"]["point_mm"], json!([0., 0., 8.]));
            assert_eq!(answer["point_mm"], json!([0., 0., 0.]));
        } else {
            assert!(answer["hit"].is_null(), "{answer}");
            assert!(answer["point_mm"].is_null(), "{answer}");
        }
        assert_eq!(controller.coordinator().snapshot(), before);
        assert_eq!(controller.coordinator().undo_len(), 0);
    }
}

#[cfg(not(feature = "occt-brep"))]
#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_synchronous_snap_distinguishes_hit_miss_and_unsupported() {
    use om9_core::model::Entity;
    use om9_scene::workspace::ViewId;
    use serde_json::json;

    let mut document = Document::with_template_tolerance("SDK synchronous snaps");
    for (start, end) in [([-5., 0., 2.], [5., 0., 2.]), ([0., -5., 8.], [0., 5., 8.])] {
        let line = Entity::new(
            "crossing",
            Geometry::Curve(Curve::Line { start, end, domain: [0., 1.] }),
            None,
        );
        document.entities.insert(line.id, line);
    }
    let with_example = |document| {
        let mut controller = Controller::new(document, Limits::default()).unwrap();
        controller.workspace.active = ViewId::LookingDown;
        controller.set_project(true);
        load_example(&mut controller);
        controller
    };
    let query = |controller: &mut Controller, kind| {
        let anchors: &[[f64; 3]] = if kind == "smarttrack" { &[[0.; 3]] } else { &[] };
        controller.run_loaded_plugin(
            "org.openmatrix9.hello",
            "SnapDemo",
            json!({ "kind": kind, "tracking_anchors_mm": anchors, "create_point": true }),
        )
    };

    let mut controller = with_example(document.clone());
    let before = controller.coordinator().snapshot();
    assert!(query(&mut controller, "intersection").unwrap()["hit"].is_null());
    for kind in ["surface", "smarttrack"] {
        let error = query(&mut controller, kind).unwrap_err();
        assert!(error.to_string().contains("unsupported"), "{error}");
    }
    assert_eq!(controller.coordinator().snapshot(), before);
    assert_eq!(controller.coordinator().undo_len(), 0);

    let result = query(&mut controller, "apparent_intersection").unwrap();
    let hit = &result["hit"];
    assert_eq!(hit["kind"], "Apparent Intersection");
    assert_eq!(hit["point_mm"], json!([0., 0., 8.]));
    let source_ids = [
        hit["root_id"].as_str().unwrap(),
        hit["second_source"]["root_id"].as_str().unwrap(),
    ];
    let expected_ids: BTreeSet<_> = document.entities.keys().map(ToString::to_string).collect();
    assert_eq!(
        source_ids.into_iter().map(str::to_owned).collect::<BTreeSet<_>>(),
        expected_ids,
    );
    assert_eq!(result["point_mm"], json!([0., 0., 0.]));
    let created = result["created"].as_str().unwrap();
    assert!(controller.document().entities.values().any(|entity| {
        entity.id.to_string() == created && entity.geometry == Geometry::Point([0.; 3])
    }));
    assert_eq!(controller.coordinator().undo_len(), 1);
    controller.undo().unwrap();
    assert_eq!(controller.document().entities, document.entities);

    let circle = Entity::new(
        "curved candidate",
        Geometry::Curve(Curve::Circle {
            center: [2., 0., 3.],
            x: [1., 0., 0.],
            y: [0., 1., 0.],
            radius: 2.,
            domain: [0., std::f64::consts::TAU],
            angle_domain: None,
        }),
        None,
    );
    document.entities.insert(circle.id, circle);
    let mut controller = with_example(document);
    let before = controller.coordinator().snapshot();
    let error = query(&mut controller, "apparent_intersection").unwrap_err();
    assert!(error.to_string().contains("unsupported"), "{error}");
    assert_eq!(controller.coordinator().snapshot(), before);
    assert_eq!(controller.coordinator().undo_len(), 0);
}

#[cfg(feature = "occt-brep")]
#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_apparent_intersection_applies_project_to_the_issued_source() {
    use om9_core::{geometry::NurbsCurve, model::{Definition, Entity}};
    use om9_scene::{camera::{Camera, CPlane, Viewport}, workspace::{DisplayMode, ViewId}};
    use std::f64::consts::PI;

    let mut document = Document::with_template_tolerance("SDK apparent intersection");
    let arc = Entity::new("far arc", Geometry::Curve(Curve::Circle {
        center: [9., 4., 2.], x: [1., 0., 0.], y: [0., 1., 0.],
        radius: 4., domain: [0., 1.], angle_domain: Some([PI, 2. * PI]),
    }), None);
    let nurbs = Entity::new("near rational curve", Geometry::Curve(Curve::Nurbs(NurbsCurve {
        degree: 2, poles: vec![[5., -4., 7.], [8., 0., 8.], [5., 4., 9.]],
        weights: vec![1., 0.5, 1.], knots: vec![0., 0., 0., 1., 1., 1.],
        domain: [0., 1.], periodic: false, reversed: false, seam: None,
    })), None);
    let ids = [arc.id.to_string(), nurbs.id.to_string()];
    let definition = Definition::new("apparent sources", vec![arc, nurbs]);
    let instance = Entity::new("block", Geometry::Instance(definition.id), None);
    let root = instance.id.to_string();
    document.definitions.insert(definition.id, definition);
    document.entities.insert(instance.id, instance);
    let mut controller = Controller::new(document, Limits::default()).unwrap();
    controller.workspace.active = ViewId::Perspective;
    let viewport = Viewport { origin: [0., 0.], size: [800., 600.] };
    let mut camera = Camera::perspective(
        [0., 0., 20.], [0.; 3], [0., 1., 0.], 2. * (viewport.size[1] / 400.).atan(),
    ).unwrap();
    camera.near_mm = 2.;
    let [x, y] = camera.project([6., 0., 8.], viewport).unwrap().unwrap().screen;
    let view = controller.workspace.views.get_mut(&ViewId::Perspective).unwrap();
    view.camera = camera;
    view.cplane = CPlane::world_xy();
    view.mode = DisplayMode::Wireframe;
    load_example(&mut controller);
    let before = controller.document().entities.clone();
    let undo_before = controller.coordinator().undo_len();

    let query = |controller: &mut Controller, kind, offset, create, from_mm: Option<[f64; 3]>| {
        let arguments = serde_json::json!({
            "kind": kind, "cursor_x": x + offset, "cursor_y": y, "create_point": create, "from_mm": from_mm,
        });
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        loop {
            match controller.run_loaded_plugin("org.openmatrix9.hello", "SnapDemo", arguments.clone()) {
                Ok(result) => break result,
                Err(error) if error.to_string().contains("pending") => {
                    assert!(std::time::Instant::now() < deadline, "{error}");
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
                Err(error) => panic!("apparent intersection SDK query: {error}"),
            }
        }
    };
    let assert_point = |value: &serde_json::Value, expected: [f64; 3]| {
        let actual: [f64; 3] = serde_json::from_value(value.clone()).unwrap();
        for (actual, expected) in actual.into_iter().zip(expected) {
            assert!((actual - expected).abs() < 1e-6, "{value}");
        }
    };
    let assert_sources = |hit: &serde_json::Value, primary: &str, secondary: &str| {
        assert_eq!(hit["kind"], "Apparent Intersection");
        assert_eq!(hit["root_id"], root);
        assert_eq!(hit["leaf_id"], primary);
        assert_eq!(hit["member_path"], serde_json::json!([primary]));
        assert_eq!(hit["second_source"]["root_id"], root);
        assert_eq!(hit["second_source"]["leaf_id"], secondary);
        assert_eq!(hit["second_source"]["member_path"], serde_json::json!([secondary]));
    };
    assert!(query(&mut controller, "intersection", 0., false, None)["hit"].is_null());
    let near = query(&mut controller, "apparent_intersection", 0., false, None);
    assert_sources(&near["hit"], &ids[1], &ids[0]);
    assert_point(&near["hit"]["point_mm"], [6., 0., 8.]);
    assert_point(&near["point_mm"], [6., 0., 8.]);
    assert_eq!(controller.document().entities, before);
    assert_eq!(controller.coordinator().undo_len(), undo_before);

    controller.set_project(true);
    let base = Some([1., -2., 3.]);
    let project_only = query(&mut controller, "apparent_intersection", 3., false, base);
    assert_sources(&project_only["hit"], &ids[0], &ids[1]);
    assert_point(&project_only["hit"]["point_mm"], [9., 0., 2.]);
    assert_point(&project_only["point_mm"], [9., 0., 0.]);
    controller.set_planar(true);
    let without_base = query(&mut controller, "apparent_intersection", 3., false, None);
    assert_point(&without_base["point_mm"], [9., 0., 0.]);
    assert_eq!(controller.document().entities, before);
    assert_eq!(controller.coordinator().undo_len(), undo_before);
    let far = query(&mut controller, "apparent_intersection", 3., true, base);
    assert_sources(&far["hit"], &ids[0], &ids[1]);
    assert_point(&far["hit"]["point_mm"], [9., 0., 2.]);
    assert_point(&far["point_mm"], [9., 0., 3.]);
    let created = far["created"].as_str().expect("created point id");
    let point = controller.document().entities.values()
        .find(|entity| entity.id.to_string() == created).expect("committed SDK point");
    let Geometry::Point(position) = point.geometry else { panic!("expected an SDK point") };
    assert_point(&serde_json::json!(position), [9., 0., 3.]);
    let committed = controller.document().entities.clone();
    assert_eq!(controller.coordinator().undo_len(), undo_before + 1);
    controller.undo().unwrap();
    assert_eq!(controller.document().entities, before);
    controller.redo().unwrap();
    assert_eq!(controller.document().entities, committed);
}

#[cfg(feature = "occt-brep")]
#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_surface_query_refuses_shaded_occlusion() {
    use om9_core::model::Entity;
    use om9_scene::{camera::Viewport, workspace::{DisplayMode, ViewId}};
    let limits = Limits::default();
    let plane = om9_core::surface_kernel::plane([-5., -5., 0.], [1., 0., 0.], [0., 1., 0.], 10., 10., &limits).unwrap();
    let surface = Entity::new("surface", Geometry::Surface(plane), None);
    let mesh = om9_core::geometry::Mesh { vertices: vec![[-5., -5., 5.], [5., -5., 5.], [5., 5., 5.], [-5., 5., 5.]],
        faces: vec![vec![0, 1, 2, 3]], normals: vec![], colors: vec![], corner_uvs: vec![] };
    let occluder = Entity::new("occluder", Geometry::Mesh(mesh), None);
    let mut document = Document::with_template_tolerance("plugin shaded surface");
    document.entities.insert(surface.id, surface);
    document.entities.insert(occluder.id, occluder);
    let mut controller = Controller::new(document, limits).unwrap();
    controller.workspace.active = ViewId::LookingDown;
    controller.workspace.views.get_mut(&ViewId::LookingDown).unwrap().mode = DisplayMode::Shaded;
    load_example(&mut controller);
    let before = controller.document().clone();
    let viewport = Viewport { origin: [0., 0.], size: [800., 600.] };
    let cursor = controller.workspace.views[&ViewId::LookingDown].camera.project([1., 1., 0.], viewport).unwrap().unwrap().screen;
    let arguments = serde_json::json!({"kind": "surface", "cursor_x": cursor[0], "cursor_y": cursor[1], "create_point": true});
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    loop {
        match controller.run_loaded_plugin("org.openmatrix9.hello", "SnapDemo", arguments.clone()) {
            Err(error) if error.to_string().contains("pending") => {
                assert!(std::time::Instant::now() < deadline);
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            Err(error) => { assert!(error.to_string().contains("occluded"), "{error}"); break; }
            Ok(value) => panic!("occluded surface unexpectedly accepted: {value}"),
        }
    }
    assert_eq!(controller.document(), &before);
    assert_eq!(controller.coordinator().undo_len(), 0);
}

#[cfg(feature = "occt-brep")]
#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_surface_query_retries_and_commits_the_source_point() {
    use om9_scene::{camera::Viewport, workspace::ViewId};
    let mut controller = Controller::new(Document::with_template_tolerance("plugin surface snap"), Limits::default()).unwrap();
    controller.solid_sphere_points([0.; 3], 5.).unwrap();
    let id = *controller.document().entities.keys().next().unwrap();
    controller.workspace.active = ViewId::LookingDown;
    load_example(&mut controller);
    let before = controller.document().clone();
    let undo_before = controller.coordinator().undo_len();
    let viewport = Viewport { origin: [0., 0.], size: [800., 600.] };
    let view = &controller.workspace.views[&ViewId::LookingDown];
    let cursor = view.camera.project([2., 1., 20_f64.sqrt()], viewport).unwrap().unwrap().screen;
    let arguments = serde_json::json!({"kind": "surface", "cursor_x": cursor[0], "cursor_y": cursor[1], "create_point": true});
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    let mut pending = 0;
    let result = loop {
        match controller.run_loaded_plugin("org.openmatrix9.hello", "SnapDemo", arguments.clone()) {
            Ok(result) => break result,
            Err(error) if error.to_string().contains("pending") => {
                pending += 1;
                assert!(std::time::Instant::now() < deadline, "surface query did not finish");
                assert_eq!(controller.document(), &before);
                assert_eq!(controller.coordinator().undo_len(), undo_before);
                if pending == 1 {
                    let miss = serde_json::json!({"kind": "surface", "cursor_x": 0., "cursor_y": 0., "create_point": true});
                    let result = controller.run_loaded_plugin("org.openmatrix9.hello", "SnapDemo", miss).unwrap();
                    assert!(result["hit"].is_null(), "a different cursor must not reuse the pending surface hit");
                    assert_eq!(controller.document(), &before);
                }
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            Err(error) => panic!("plugin surface query failed: {error}"),
        }
    };
    assert!(pending > 0);
    assert_eq!(result["hit"]["root_id"], id.to_string());
    let values = result["point_mm"].as_array().unwrap();
    let point = [values[0].as_f64().unwrap(), values[1].as_f64().unwrap(), values[2].as_f64().unwrap()];
    assert!((point[2] - 20_f64.sqrt()).abs() < 1e-6, "{point:?}");
    assert_eq!(controller.coordinator().undo_len(), undo_before + 1);
    let after = controller.document().clone();
    controller.undo().unwrap(); assert_eq!(controller.document(), &before);
    controller.redo().unwrap(); assert_eq!(controller.document(), &after);
}

#[cfg(feature = "occt-brep")]
#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_point_query_cancels_queued_edge_source() {
    use om9_core::{budget::memory::MemoryClass, geometry::NurbsCurve, model::Entity};
    use om9_scene::{camera::Viewport, workspace::ViewId};

    let mut document = Document::with_template_tolerance("SDK snap kind switch");
    let curve = Entity::new("curve", Geometry::Curve(Curve::Nurbs(NurbsCurve {
        degree: 2,
        poles: vec![[0., 0., 0.], [2., 3., 0.], [4., 0., 0.]],
        weights: vec![1.; 3],
        knots: vec![0., 0., 0., 1., 1., 1.],
        domain: [0., 1.],
        periodic: false,
        reversed: false,
        seam: None,
    })), None);
    let point = Entity::new("point", Geometry::Point([0.; 3]), None);
    let point_id = point.id;
    document.entities.insert(curve.id, curve);
    document.entities.insert(point.id, point);
    let mut controller = Controller::new(document, Limits::default()).unwrap();
    controller.workspace.active = ViewId::LookingDown;
    load_example(&mut controller);
    let before = controller.coordinator().snapshot();
    let baseline = controller.coordinator().external_reserved_bytes(MemoryClass::Kernel);
    let viewport = Viewport { origin: [0., 0.], size: [800., 600.] };
    let [x, y] = controller.workspace.views[&ViewId::LookingDown].camera
        .project([0.; 3], viewport).unwrap().unwrap().screen;
    let end_query = serde_json::json!({"kind": "end", "cursor_x": x, "cursor_y": y});
    let first = controller.run_loaded_plugin("org.openmatrix9.hello", "SnapDemo", end_query.clone()).unwrap_err();
    assert!(first.to_string().contains("pending"), "{first}");

    let account = controller.coordinator().external_memory_account();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
    let permit = loop {
        match account.try_acquire_surface_snap() {
            Ok(permit) => break permit,
            Err(om9_core::Error::Invalid("surface snap already active")) => {
                assert!(std::time::Instant::now() < deadline, "edge bounds did not finish");
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            Err(error) => panic!("failed to hold the source worker slot: {error}"),
        }
    };
    loop {
        let pending = controller.run_loaded_plugin("org.openmatrix9.hello", "SnapDemo", end_query.clone()).unwrap_err();
        assert!(pending.to_string().contains("pending"), "{pending}");
        if controller.coordinator().external_reserved_bytes(MemoryClass::Kernel) >= baseline + 32 * 1024 * 1024 {
            break;
        }
        assert!(std::time::Instant::now() < deadline, "edge extraction was never queued");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    let point_query = serde_json::json!({"kind": "point", "cursor_x": x, "cursor_y": y});
    let result = controller.run_loaded_plugin("org.openmatrix9.hello", "SnapDemo", point_query).unwrap();
    assert_eq!(result["hit"]["kind"], "Point");
    assert_eq!(result["hit"]["root_id"], point_id.to_string());
    assert_eq!(result["point_mm"], serde_json::json!([0., 0., 0.]));
    assert_eq!(controller.coordinator().snapshot(), before);
    assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel), baseline,
        "taking a Point result must release the superseded extraction while its slot is still held");
    drop(permit);
}

#[cfg(feature = "occt-brep")]
#[test]
#[ignore = "requires built native hello-plugin .om9p and OM9_TEST_PLUGIN_PACKAGE"]
fn native_plugin_retries_pending_source_snap_with_a_fresh_invocation() {
    use om9_core::{geometry::NurbsCurve, model::Entity};
    use om9_scene::camera::Viewport;
    let curve = Curve::Nurbs(NurbsCurve {
        degree: 2, poles: vec![[0., 0., 0.], [2., 3., 0.], [4., 0., 0.]],
        weights: vec![1.; 3], knots: vec![0., 0., 0., 1., 1., 1.],
        domain: [0., 1.], periodic: false, reversed: false, seam: None,
    });
    let mut document = Document::with_template_tolerance("plugin deferred snap");
    let entity = Entity::new("source curve", Geometry::Curve(curve), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut controller = Controller::new(document, Limits::default()).unwrap();
    load_example(&mut controller);
    let before = controller.coordinator().snapshot();
    let before_document = controller.document().clone();
    let undo_before = controller.coordinator().undo_len();
    let viewport = Viewport { origin: [0., 0.], size: [800., 600.] };
    let view = &controller.workspace.views[&controller.workspace.active];
    let cursor = view.camera.project([0., 0., 0.], viewport).unwrap().unwrap().screen;
    let arguments = serde_json::json!({"cursor_x": cursor[0], "cursor_y": cursor[1], "create_point": true});
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    let mut pending_count = 0;
    let result = loop {
        match controller.run_loaded_plugin("org.openmatrix9.hello", "SnapDemo", arguments.clone()) {
            Ok(result) => break result,
            Err(error) if error.to_string().contains("pending") => {
                pending_count += 1;
                assert!(std::time::Instant::now() < deadline, "deferred snap did not finish");
                assert_eq!(controller.coordinator().snapshot(), before);
                assert_eq!(controller.coordinator().undo_len(), undo_before);
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            Err(error) => panic!("plugin snap failed: {error}"),
        }
    };
    assert!(pending_count > 0, "fixture must exercise the deferred path");
    assert_eq!(result["hit"]["root_id"], id.to_string());
    assert_eq!(result["point_mm"], serde_json::json!([0., 0., 0.]));
    let new_id = result["created"].as_str().expect("created point id");
    assert!(controller.document().entities.values().any(|entity| entity.id.to_string() == new_id
        && matches!(&entity.geometry, Geometry::Point(point) if *point == [0., 0., 0.])));
    assert_eq!(controller.coordinator().undo_len(), undo_before + 1);
    controller.undo().unwrap();
    assert_eq!(controller.document(), &before_document);
    controller.redo().unwrap();
    assert!(controller.document().entities.values().any(|entity| entity.id.to_string() == new_id));
}
