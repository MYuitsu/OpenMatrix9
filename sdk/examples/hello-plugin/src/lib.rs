// SPDX-License-Identifier: LGPL-2.1-or-later
//! A real native SDK consumer. No core, UI, kernel or Rhino dependency.
use om9_sdk::{
    AttributesPatch, CommandDescriptor, Dimension, Host, KnotMode, ObjectQuery, ParameterDomain,
    ParameterSpec, ParameterValue, Plugin, PluginDescriptor, Result, SdkError, Value,
};
use std::collections::BTreeMap;

#[derive(Default)]
struct HelloPlugin;
fn number(arguments: &Value, key: &str, default: f64) -> Result<f64> {
    match arguments.get(key) {
        None => Ok(default),
        Some(value) => value
            .as_f64()
            .filter(|n| n.is_finite())
            .ok_or_else(|| SdkError::new("argument", format!("{key} must be a finite number"))),
    }
}
impl Plugin for HelloPlugin {
    fn descriptor() -> PluginDescriptor {
        PluginDescriptor {
            id: "org.openmatrix9.hello".into(),
            name: "OM9 SDK example".into(),
            version: "0.1.0".into(),
            sdk_version: 1,
            commands: vec![
                CommandDescriptor {
                    id: "CreateProfile".into(),
                    title: "Create example profile".into(),
                },
                CommandDescriptor {
                    id: "MoveSelection".into(),
                    title: "Move selected objects".into(),
                },
                CommandDescriptor {
                    id: "DeleteSelection".into(),
                    title: "Delete selected objects".into(),
                },
                CommandDescriptor {
                    id: "UnitsReport".into(),
                    title: "Report typed units and tolerance".into(),
                },
                CommandDescriptor {
                    id: "AttributesDemo".into(),
                    title: "Edit attributes of two objects".into(),
                },
                CommandDescriptor {
                    id: "LayerDemo".into(),
                    title: "Create a layer and assign an object".into(),
                },
                CommandDescriptor {
                    id: "ParameterDemo".into(),
                    title: "Validate typed builder parameters".into(),
                },
                CommandDescriptor {
                    id: "FeedbackDemo".into(),
                    title: "Report progress and warnings".into(),
                },
                CommandDescriptor {
                    id: "SelectionDemo".into(),
                    title: "Write the selection".into(),
                },
                CommandDescriptor {
                    id: "CurveDemo".into(),
                    title: "Read curve information and samples".into(),
                },
                CommandDescriptor {
                    id: "CurveBuildDemo".into(),
                    title: "Build an arc and a spline".into(),
                },
                CommandDescriptor {
                    id: "MeasureDemo".into(),
                    title: "Measure a distance and an angle".into(),
                },
                CommandDescriptor {
                    id: "CurveShapeDemo".into(),
                    title: "Build an ellipse and a rectangle".into(),
                },
                CommandDescriptor {
                    id: "ProjectDemo".into(),
                    title: "Read project state and request a save".into(),
                },
                CommandDescriptor {
                    id: "ExtensionDemo".into(),
                    title: "Tag an object with an extension slot".into(),
                },
                CommandDescriptor {
                    id: "PlacementDemo".into(),
                    title: "Set an object placement matrix".into(),
                },
                CommandDescriptor {
                    id: "MeshDemo".into(),
                    title: "Create a quad mesh".into(),
                },
                CommandDescriptor {
                    id: "ChangeDemo".into(),
                    title: "Read the recorded change batch".into(),
                },
                CommandDescriptor {
                    id: "BoundsDemo".into(),
                    title: "Measure an object bounding box".into(),
                },
                CommandDescriptor {
                    id: "LengthDemo".into(),
                    title: "Measure a curve length".into(),
                },
                CommandDescriptor {
                    id: "RadiusDemo".into(),
                    title: "Measure a circle radius".into(),
                },
                CommandDescriptor {
                    id: "SummaryDemo".into(),
                    title: "Count objects by geometry kind".into(),
                },
                CommandDescriptor {
                    id: "LabelDemo".into(),
                    title: "Report a measurement label".into(),
                },
                CommandDescriptor {
                    id: "LayerOrderDemo".into(),
                    title: "Read the layer order".into(),
                },
                CommandDescriptor {
                    id: "AreaDemo".into(),
                    title: "Measure a mesh area".into(),
                },
                CommandDescriptor {
                    id: "GroupsDemo".into(),
                    title: "List document groups".into(),
                },
                CommandDescriptor {
                    id: "BlocksDemo".into(),
                    title: "List block definitions".into(),
                },
                CommandDescriptor {
                    id: "ClearLayerDemo".into(),
                    title: "Clear the current layer".into(),
                },
                CommandDescriptor {
                    id: "PlacementReadDemo".into(),
                    title: "Read an object placement".into(),
                },
                CommandDescriptor {
                    id: "InfoDemo".into(),
                    title: "Read one object record".into(),
                },
                CommandDescriptor {
                    id: "GroupCreateDemo".into(),
                    title: "Create a group from members".into(),
                },
                CommandDescriptor {
                    id: "GroupDeleteDemo".into(),
                    title: "Delete a group".into(),
                },
                CommandDescriptor {
                    id: "LayerInfoDemo".into(),
                    title: "Read one layer record".into(),
                },
                CommandDescriptor {
                    id: "ToleranceDemo".into(),
                    title: "Set the document tolerance".into(),
                },
                CommandDescriptor {
                    id: "ProvenanceDemo".into(),
                    title: "Read import provenance".into(),
                },
                CommandDescriptor {
                    id: "PickableDemo".into(),
                    title: "Test if an object is pickable".into(),
                },
                CommandDescriptor {
                    id: "HistoryDemo".into(),
                    title: "Read associative history and its rebuild plan".into(),
                },
            ],
        }
    }
    fn execute(&mut self, host: &mut Host<'_>, command: &str, arguments: Value) -> Result<Value> {
        if command == "HistoryDemo" {
            let features = host.history_list()?;
            let dependencies = host.history_dependencies()?;
            let rebuild_plan = host.history_rebuild_plan()?;
            // Provenance is optional: a legacy record without a unit-safe payload is
            // reported by the host as an error, so the demo only asks for the first typed
            // feature.
            let provenance = match features.first() {
                Some(feature) => Some(host.history_provenance(feature.id.clone())?),
                None => None,
            };
            return Ok(serde_json::json!({
                "features": features.len(),
                "commands": features.iter().map(|feature| feature.command.clone()).collect::<Vec<_>>(),
                "dependencies": dependencies.len(),
                "rebuild_plan": rebuild_plan,
                "provenance_command": provenance.map(|provenance| provenance.command),
            }));
        }
        if command == "CurveShapeDemo" {
            let ellipse = host.add_ellipse(
                "SDK ellipse",
                [0., 0., 0.],
                [1., 0., 0.],
                [0., 1., 0.],
                5.,
                2.,
            )?;
            let rectangle = host.add_rectangle(
                "SDK rectangle",
                [0., 0., 0.],
                [1., 0., 0.],
                [0., 1., 0.],
                10.,
                5.,
            )?;
            let polygon = host.add_polygon(
                "SDK polygon",
                vec![[0., 0., 0.], [4., 0., 0.], [4., 3., 0.], [0., 3., 0.]],
            )?;
            let framed = host.add_circle_on_frame(
                "SDK XZ circle",
                [0., 0., 0.],
                [1., 0., 0.],
                [0., 0., 1.],
                3.,
            )?;
            let ellipse_info = host.curve_info(&ellipse)?;
            let ellipse_mid = host.evaluate_curve(&ellipse, vec![0.25], 0)?;
            let rectangle_info = host.curve_info(&rectangle)?;
            let rectangle_semantics = host.curve_semantics(&rectangle, None)?;
            let polygon_info = host.curve_info(&polygon)?;
            let polygon_semantics = host.curve_semantics(&polygon, None)?;
            let framed_info = host.curve_info(&framed)?;
            let rect_segments = host.curve_segments(&rectangle)?;
            let rect_segment_mid = host.evaluate_curve_segment(&rectangle, 0, vec![0.5], 1)?;
            return Ok(serde_json::json!({
                "ellipse": ellipse,
                "rectangle": rectangle,
                "polygon": polygon,
                "polygon_info": polygon_info,
                "polygon_semantics": polygon_semantics,
                "framed": framed,
                "framed_info": framed_info,
                "rect_segments": rect_segments,
                "rect_segment_mid": rect_segment_mid,
                "ellipse_info": ellipse_info,
                "ellipse_mid": ellipse_mid,
                "rectangle_info": rectangle_info,
                "rectangle_semantics": rectangle_semantics,
            }));
        }
        if command == "PickableDemo" {
            let id = arguments
                .get("id")
                .and_then(Value::as_str)
                .ok_or_else(|| SdkError::new("argument", "id is required"))?
                .to_string();
            return Ok(serde_json::json!({"pickable": host.is_object_pickable(&id)?}));
        }
        if command == "ProvenanceDemo" {
            let id = arguments
                .get("id")
                .and_then(Value::as_str)
                .ok_or_else(|| SdkError::new("argument", "id is required"))?
                .to_string();
            return Ok(serde_json::json!({"provenance": host.object_provenance(&id)?}));
        }
        if command == "ToleranceDemo" {
            host.set_document_tolerance(0.01, 1e-6, 1e-8)?;
            return Ok(serde_json::json!({"ok": true}));
        }
        if command == "LayerInfoDemo" {
            let id = arguments
                .get("id")
                .and_then(Value::as_str)
                .ok_or_else(|| SdkError::new("argument", "id is required"))?
                .to_string();
            return Ok(serde_json::json!({"layer": host.layer_info(&id)?}));
        }
        if command == "GroupDeleteDemo" {
            let id = arguments
                .get("id")
                .and_then(Value::as_str)
                .ok_or_else(|| SdkError::new("argument", "id is required"))?
                .to_string();
            host.delete_group(&id)?;
            return Ok(serde_json::json!({"groups": host.groups()?}));
        }
        if command == "GroupCreateDemo" {
            let members = arguments
                .get("members")
                .and_then(Value::as_array)
                .map(|items| {
                    items
                        .iter()
                        .filter_map(Value::as_str)
                        .map(str::to_string)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            return Ok(serde_json::json!({
                "group": host.create_group("SDK set", members)?,
            }));
        }
        if command == "InfoDemo" {
            let id = arguments
                .get("id")
                .and_then(Value::as_str)
                .ok_or_else(|| SdkError::new("argument", "id is required"))?
                .to_string();
            return Ok(serde_json::json!({"info": host.object_info(&id)?}));
        }
        if command == "PlacementReadDemo" {
            let id = arguments
                .get("id")
                .and_then(Value::as_str)
                .ok_or_else(|| SdkError::new("argument", "id is required"))?
                .to_string();
            return Ok(serde_json::json!({"placement": host.object_placement(&id)?}));
        }
        if command == "ClearLayerDemo" {
            host.clear_current_layer()?;
            return Ok(serde_json::json!({"ok": true}));
        }
        if command == "BlocksDemo" {
            return Ok(serde_json::json!({"definitions": host.block_definitions()?}));
        }
        if command == "GroupsDemo" {
            return Ok(serde_json::json!({"groups": host.groups()?}));
        }
        if command == "AreaDemo" {
            let id = arguments
                .get("id")
                .and_then(Value::as_str)
                .ok_or_else(|| SdkError::new("argument", "id is required"))?
                .to_string();
            return Ok(serde_json::json!({"area": host.measure_mesh_area(&id)?}));
        }
        if command == "LayerOrderDemo" {
            return Ok(serde_json::json!({"order": host.layer_order()?}));
        }
        if command == "LabelDemo" {
            let id = arguments
                .get("id")
                .and_then(Value::as_str)
                .ok_or_else(|| SdkError::new("argument", "id is required"))?
                .to_string();
            let length = host.measure_curve_length(&id)?;
            host.report_measurement(format!("length {:.2} mm", length.millimeters))?;
            return Ok(serde_json::json!({"length": length}));
        }
        if command == "SummaryDemo" {
            return Ok(serde_json::json!({"summary": host.object_summary()?}));
        }
        if command == "RadiusDemo" {
            let id = arguments
                .get("id")
                .and_then(Value::as_str)
                .ok_or_else(|| SdkError::new("argument", "id is required"))?
                .to_string();
            return Ok(serde_json::json!({"radius": host.measure_curve_radius(&id)?}));
        }
        if command == "LengthDemo" {
            let id = arguments
                .get("id")
                .and_then(Value::as_str)
                .ok_or_else(|| SdkError::new("argument", "id is required"))?
                .to_string();
            return Ok(serde_json::json!({"length": host.measure_curve_length(&id)?}));
        }
        if command == "BoundsDemo" {
            let id = arguments
                .get("id")
                .and_then(Value::as_str)
                .ok_or_else(|| SdkError::new("argument", "id is required"))?
                .to_string();
            return Ok(serde_json::json!({"bounds": host.object_bounds(&id)?}));
        }
        if command == "ChangeDemo" {
            return Ok(serde_json::json!({"change": host.last_change()?}));
        }
        if command == "MeshDemo" {
            let id = host.add_mesh(
                "SDK quad",
                vec![[0., 0., 0.], [10., 0., 0.], [10., 10., 0.], [0., 10., 0.]],
                vec![vec![0, 1, 2, 3]],
            )?;
            return Ok(serde_json::json!({"id": id}));
        }
        if command == "PlacementDemo" {
            let id = arguments
                .get("id")
                .and_then(Value::as_str)
                .ok_or_else(|| SdkError::new("argument", "id is required"))?
                .to_string();
            let mut matrix = [[0.0; 4]; 4];
            matrix[0][0] = 1.0;
            matrix[1][1] = 1.0;
            matrix[2][2] = 1.0;
            matrix[3][3] = 1.0;
            matrix[0][3] = 10.0;
            matrix[1][3] = 20.0;
            matrix[2][3] = 30.0;
            host.set_object_placement(&id, matrix)?;
            return Ok(serde_json::json!({"id": id}));
        }
        if command == "ExtensionDemo" {
            let id = arguments
                .get("id")
                .and_then(Value::as_str)
                .ok_or_else(|| SdkError::new("argument", "id is required"))?
                .to_string();
            host.set_object_extension(&id, "org.openmatrix9.demo", 1, "{\"weight_g\":4.2}")?;
            let slots = host.object_extensions(&id)?;
            return Ok(serde_json::json!({"id": id, "slots": slots}));
        }
        if command == "ProjectDemo" {
            let project = host.project_state()?;
            if arguments.get("save").and_then(Value::as_bool) == Some(true) {
                host.save_project()?;
            }
            if arguments.get("save_as").and_then(Value::as_bool) == Some(true) {
                host.save_project_as()?;
            }
            return Ok(serde_json::json!({"project": project}));
        }
        if command == "MeasureDemo" {
            let distance = host.measure_distance([0., 0., 0.], [3., 4., 0.])?;
            let angle =
                host.measure_angle([0., 0., 0.], [1., 0., 0.], [0., 0., 0.], [0., 1., 0.])?;
            return Ok(serde_json::json!({
                "distance": distance,
                "angle": angle,
            }));
        }
        if command == "CurveBuildDemo" {
            let arc = host.add_arc("SDK arc", [5., 0., 0.], [0., 5., 0.], [-5., 0., 0.])?;
            let spline = host.add_interpolated_curve(
                "SDK spline",
                vec![[0., 0., 0.], [1., 5., 0.], [3., 5., 0.], [4., 0., 0.]],
                3,
                KnotMode::ChordLength,
            )?;
            let control = host.add_nurbs_curve(
                "SDK nurbs",
                3,
                vec![[0., 0., 0.], [1., 4., 0.], [3., 4., 0.], [4., 0., 0.]],
                vec![1.; 4],
                vec![0., 0., 0., 0., 1., 1., 1., 1.],
            )?;
            let arc_info = host.curve_info(&arc)?;
            let spline_definition = host.curve_definition(&spline)?;
            let control_definition = host.curve_definition(&control)?;
            let arc_midpoint = host.evaluate_curve(&arc, vec![0.5], 0)?;
            return Ok(serde_json::json!({
                "arc": arc,
                "spline": spline,
                "control": control,
                "arc_info": arc_info,
                "spline_definition": spline_definition,
                "control_definition": control_definition,
                "arc_midpoint": arc_midpoint,
            }));
        }
        if command == "CurveDemo" {
            let id = arguments
                .get("id")
                .and_then(Value::as_str)
                .ok_or_else(|| SdkError::new("argument", "id is required"))?
                .to_string();
            if arguments.get("sample").and_then(Value::as_bool) == Some(true) {
                let samples = host.evaluate_curve(&id, vec![0.0], 0)?;
                return Ok(serde_json::json!({"samples": samples}));
            }
            if arguments.get("definition").and_then(Value::as_bool) == Some(true) {
                return Ok(serde_json::json!({
                    "definition": host.curve_definition(&id)?,
                }));
            }
            if arguments.get("semantics").and_then(Value::as_bool) == Some(true) {
                return Ok(serde_json::json!({
                    "semantics": host.curve_semantics(&id, None)?,
                }));
            }
            let info = host.curve_info(&id)?;
            let domain = info
                .domain
                .ok_or_else(|| SdkError::new("curve", "curve has no single parameter domain"))?;
            let middle = domain[0] + (domain[1] - domain[0]) / 2.0;
            let samples = host.evaluate_curve(&id, vec![domain[0], middle, domain[1]], 1)?;
            return Ok(serde_json::json!({"info": info, "samples": samples}));
        }
        if command == "SelectionDemo" {
            let page = host.query_objects(ObjectQuery {
                limit: 2,
                ..ObjectQuery::default()
            })?;
            let ids: Vec<String> = page
                .objects
                .iter()
                .map(|object| object.id.clone())
                .collect();
            if ids.is_empty() {
                return Err(SdkError::new(
                    "document",
                    "SelectionDemo needs at least one object",
                ));
            }
            if arguments.get("invalid").and_then(Value::as_bool) == Some(true) {
                // A canonical but unknown root: the host must refuse the command.
                host.set_selection(vec!["00000000-0000-4000-8000-000000000001".into()])?;
            }
            if arguments.get("remove").and_then(Value::as_bool) == Some(true) {
                host.remove_from_selection(ids.clone())?;
            } else if arguments.get("toggle").and_then(Value::as_bool) == Some(true) {
                host.toggle_selection(ids[..1].to_vec())?;
            } else if arguments.get("add").and_then(Value::as_bool) == Some(true) {
                host.add_to_selection(ids.clone())?;
            } else {
                host.set_selection(ids.clone())?;
            }
            return Ok(serde_json::json!({
                "ids": ids,
                "selected": host.selected_objects()?,
            }));
        }
        if command == "FeedbackDemo" {
            // Reports go to the host command event log; a refused command keeps
            // its reports but publishes nothing.
            host.progress("feedback demo started")?;
            let mut reports = 2;
            if arguments.get("steps").and_then(Value::as_u64).unwrap_or(2) >= 2 {
                host.progress("feedback demo half way")?;
                reports += 1;
            }
            host.warning("feedback demo warning")?;
            if arguments.get("fail").and_then(Value::as_bool) == Some(true) {
                return Err(SdkError::new("canceled", "feedback demo refused"));
            }
            return Ok(serde_json::json!({"reports": reports}));
        }
        if command == "ParameterDemo" {
            // The plugin declares its own schema; the host applies defaults and the
            // document's typed-option rule. Typed values only - a bare number is refused.
            let specs = vec![
                ParameterSpec {
                    name: "radius".into(),
                    domain: ParameterDomain::Number {
                        dimension: Dimension::Length,
                        min: Some(0.5),
                        min_inclusive: false,
                        max: Some(50.0),
                        step: None,
                    },
                    default: None,
                    required: true,
                },
                ParameterSpec {
                    name: "sides".into(),
                    domain: ParameterDomain::Number {
                        dimension: Dimension::Count,
                        min: Some(3.0),
                        min_inclusive: true,
                        max: Some(64.0),
                        step: None,
                    },
                    default: Some(ParameterValue::Count { value: 6 }),
                    required: false,
                },
                ParameterSpec {
                    name: "profile".into(),
                    domain: ParameterDomain::Choice {
                        choices: vec!["round".into(), "flat".into()],
                    },
                    default: Some(ParameterValue::Choice {
                        value: "round".into(),
                    }),
                    required: false,
                },
            ];
            let mut supplied = BTreeMap::new();
            if let Some(radius) = arguments.get("radius") {
                supplied.insert("radius".to_string(), radius.clone());
            }
            let applied = host.apply_parameters(specs, supplied)?;
            return Ok(serde_json::json!({"parameters": applied}));
        }
        if command == "LayerDemo" {
            // One command: create a layer, make it current, add one object into it,
            // assign an existing object to it and read the staged layer list back.
            let page = host.query_objects(ObjectQuery {
                limit: 1,
                ..ObjectQuery::default()
            })?;
            let Some(object) = page.objects.first() else {
                return Err(SdkError::new(
                    "document",
                    "LayerDemo needs at least one object",
                ));
            };
            let object_id = object.id.clone();
            let layer = host.layer_create("SDK Gems", [12, 200, 140, 255], true)?;
            let new_object = host.add_point("SDK gem", [0., 10., 0.])?;
            host.assign_objects_to_layer(vec![object_id.clone()], &layer)?;
            let updated = host.object_attributes(&object_id)?;
            let created = host.object_attributes(&new_object)?;
            let layers = host.layers()?;
            return Ok(serde_json::json!({
                "layer": layer,
                "object": object_id,
                "object_layer": updated.layer,
                "object_color": updated.color,
                "attributes_revision": updated.attributes_revision,
                "new_object": new_object,
                "new_object_layer": created.layer,
                "layer_count": layers.len(),
                "layers": layers,
            }));
        }
        if command == "AttributesDemo" {
            // Two captured objects: one attribute edit and one rename, then a
            // read-back of the staged records. Both end up in one transaction.
            let page = host.query_objects(ObjectQuery {
                limit: 2,
                ..ObjectQuery::default()
            })?;
            if page.objects.len() < 2 {
                return Err(SdkError::new(
                    "document",
                    "AttributesDemo needs at least two objects",
                ));
            }
            let first = page.objects[0].id.clone();
            let second = page.objects[1].id.clone();
            let first_after = host.set_object_attributes(
                &first,
                AttributesPatch {
                    color: Some("#ff8800".into()),
                    locked: Some(true),
                    ..AttributesPatch::default()
                },
            )?;
            host.rename_object(&second, "SDK renamed")?;
            let refreshed = host.query_objects(ObjectQuery::default())?;
            return Ok(serde_json::json!({
                "first": first_after,
                "second_label": host.object_attributes(&second)?.label,
                "total": refreshed.total,
                "objects": refreshed
                    .objects
                    .iter()
                    .map(|object| serde_json::json!({
                        "id": object.id,
                        "label": object.label,
                        "attributes_revision": object.attributes_revision,
                    }))
                    .collect::<Vec<_>>(),
            }));
        }
        if command == "UnitsReport" {
            // Read-only: the document's own unit/tolerance and typed numeric parsing.
            let units = host.document_units()?;
            let one_inch = host.parse_length("1 in")?;
            let bare = host.parse_length("2.5")?;
            let right = host.parse_angle("90")?;
            let radian = host.parse_angle("1rad")?;
            let coordinates = host.parse_coordinates("10, 20, 30")?;
            return Ok(serde_json::json!({
                "unit": units.unit,
                "millimeters_per_unit": units.millimeters_per_unit,
                "tolerance": units.tolerance,
                "one_inch_mm": one_inch.millimeters,
                "bare_mm": bare.millimeters,
                "right_degrees": right.degrees,
                "right_radians": right.radians,
                "one_radian_degrees": radian.degrees,
                "coordinates_mm": coordinates.millimeters,
                "capabilities": host.capabilities()?,
            }));
        }
        if command == "MoveSelection" || command == "DeleteSelection" {
            let ids = host.selected_objects()?;
            let dx = number(&arguments, "dx_mm", 2.)?;
            let matrix = [
                [1., 0., 0., dx],
                [0., 1., 0., 0.],
                [0., 0., 1., 0.],
                [0., 0., 0., 1.],
            ];
            for id in &ids {
                if command == "DeleteSelection" {
                    host.delete_object(id)?;
                } else {
                    let _geometry = host.object_geometry(id)?;
                    host.transform_object(id, matrix)?;
                }
            }
            return Ok(serde_json::json!({"objects":ids}));
        }
        if command != "CreateProfile" {
            return Err(SdkError::new("command", "unknown example command"));
        }
        let _context = (
            host.document_info()?,
            host.capabilities()?,
            host.layers()?,
            host.objects()?,
        );
        let radius = number(&arguments, "radius_mm", 5.)?;
        if !radius.is_finite() || radius <= 0. {
            return Err(SdkError::new(
                "radius",
                "radius_mm must be positive and finite",
            ));
        }
        let line = host.add_line("SDK line", [0., 0., 0.], [10., 0., 0.])?;
        let circle = host.add_circle(
            "SDK circle",
            [0., 0., 0.],
            [1., 0., 0.],
            [0., 1., 0.],
            radius,
        )?;
        let polyline = host.add_polyline(
            "SDK profile",
            vec![
                [0., 0., 0.],
                [10., 0., 0.],
                [10., 20., 0.],
                [0., 20., 0.],
                [0., 0., 0.],
            ],
        )?;
        // Demonstrates that a later plugin error rolls back all earlier SDK calls.
        if arguments.get("cancel").and_then(Value::as_bool) == Some(true) {
            return Err(SdkError::new("canceled", "example command canceled"));
        }
        Ok(serde_json::json!({"objects":[line,circle,polyline],"radius_mm":radius}))
    }
}
om9_sdk::export_plugin!(HelloPlugin);
