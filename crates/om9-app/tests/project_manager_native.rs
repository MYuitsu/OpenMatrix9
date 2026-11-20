// SPDX-License-Identifier: LGPL-2.1-or-later
#![cfg(windows)]

use om9_app::session::Controller;
use om9_core::{budget::{Limits, Usage}, model::Document, retained::HostRef, transaction::{Cancellation, TransactionCoordinator, World}};
use om9_exchange::{ExportCapture, ImportCapture, ImportOptions, NativeExchange};
use om9_native_owner::{NativeProgram, ProcessLimits};
use sha2::{Digest, Sha256};
use std::path::PathBuf;

fn admitted(name: &str) -> NativeProgram {
    let path = PathBuf::from(std::env::var_os(name).expect("native executable path"));
    let hash = Sha256::digest(std::fs::read(&path).unwrap()).into();
    NativeProgram::admit(&path, hash).unwrap()
}

fn occurrence_points(document: &Document, root: &om9_core::model::Entity) -> Vec<[f64; 3]> {
    fn visit(document: &Document, entity: &om9_core::model::Entity, parent: om9_core::geometry::Transform, points: &mut Vec<[f64; 3]>, depth: usize) {
        assert!(depth < 16, "unexpected definition cycle");
        let world = parent.compose(entity.placement).unwrap();
        match &entity.geometry {
            om9_core::geometry::Geometry::Point(point) => points.push(world.point(*point).unwrap()),
            om9_core::geometry::Geometry::Instance(definition) => {
                for member in &document.definitions[definition].members { visit(document, member, world, points, depth + 1); }
            }
            _ => panic!("unexpected fixture member geometry"),
        }
    }
    let mut points = Vec::new();
    visit(document, root, om9_core::geometry::Transform::identity(), &mut points, 0);
    points.sort_by(|a, b| a[0].total_cmp(&b[0]).then(a[1].total_cmp(&b[1])).then(a[2].total_cmp(&b[2])));
    points
}

#[test]
#[ignore = "requires configured native exchange, SDK fixture, and scratch directory"]
fn selected_native_point_survives_jobbag_and_native_publication() {
    selected_native_round_trip("20000000-0000-4000-8000-00000000000a", false);
}

#[test]
#[ignore = "requires configured native exchange, SDK fixture, and scratch directory"]
fn selected_native_nested_instance_survives_jobbag_and_native_publication() {
    selected_native_round_trip("20000000-0000-4000-8000-000000000012", true);
}

#[test]
#[ignore = "requires configured native exchange, SDK fixture, and scratch directory"]
fn selected_native_line_survives_jobbag_and_native_publication() {
    selected_native_round_trip("20000000-0000-4000-8000-00000000000b", false);
}

#[test]
#[ignore = "requires configured native exchange, SDK fixture, and scratch directory"]
fn selected_native_rational_curve_survives_jobbag_and_native_publication() {
    selected_native_round_trip("20000000-0000-4000-8000-00000000000d", false);
}

#[test]
#[ignore = "requires configured native exchange, SDK fixture, and scratch directory"]
fn selected_native_rational_surface_survives_jobbag_and_native_publication() {
    selected_native_round_trip("20000000-0000-4000-8000-00000000000e", false);
}

#[test]
#[ignore = "requires configured native exchange, SDK fixture, and scratch directory"]
fn selected_native_mesh_survives_jobbag_and_native_publication() {
    selected_native_round_trip("20000000-0000-4000-8000-00000000000f", false);
}

#[test]
#[ignore = "requires configured native exchange, SDK fixture, and scratch directory"]
fn mixed_native_selection_survives_jobbag_and_native_publication() {
    selected_native_round_trip_set(&[
        "20000000-0000-4000-8000-00000000000a",
        "20000000-0000-4000-8000-00000000000b",
        "20000000-0000-4000-8000-00000000000d",
        "20000000-0000-4000-8000-00000000000e",
        "20000000-0000-4000-8000-00000000000f",
    ], false);
}

fn selected_native_round_trip(native_id: &str, instance: bool) {
    selected_native_round_trip_set(&[native_id], instance);
}

fn selected_native_round_trip_set(native_ids: &[&str], instance: bool) {
    let base = PathBuf::from(std::env::var_os("OM9_NATIVE_IMPORT_SCRATCH").unwrap());
    let dir = base.join(format!("project-jobbag-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&dir).unwrap();
    let source_path = dir.join("source.3dm");
    let cancel = Cancellation::default();
    admitted("OM9_SOURCE_FIXTURE").run(&["clean-mm".into(), source_path.as_os_str().to_owned()], &ProcessLimits::default(), &cancel).unwrap();
    let exchange = NativeExchange::new(admitted("OM9_EXCHANGE_NATIVE"), ProcessLimits::default()).unwrap();
    let document = Document::new("native source");
    let id = document.id;
    let mut source = TransactionCoordinator::new(World::new(vec![document], &Limits::default()).unwrap(), Limits::default()).unwrap();
    let imported = exchange.prepare_import(&source, ImportCapture::target(&source, id).unwrap(), &source_path, &dir, &ImportOptions::default(), &cancel).unwrap();
    let (prepared, receipt) = imported.prepare(&source, &cancel).unwrap();
    source.commit(prepared).unwrap();
    let doc = source.world().documents[&id].as_ref().clone();
    let selected: Vec<_> = native_ids.iter().map(|native_id| {
        let native_id = uuid::Uuid::parse_str(native_id).unwrap();
        let host = doc.import_namespaces[&receipt.namespace].records.iter().find(|r| r.key.id == native_id).unwrap().host.unwrap();
        let HostRef::Object(id) = host else { panic!("expected a root object") };
        id
    }).collect();
    let expected: Vec<_> = selected.iter().map(|id| (doc.entities[id].geometry.clone(), doc.entities[id].placement)).collect();
    let expected_points = if instance { occurrence_points(&doc, &doc.entities[&selected[0]]) } else { Vec::new() };
    std::fs::remove_file(&source_path).unwrap();
    let controller = Controller::new(doc, Limits::default()).unwrap();
    let bag = controller.copy_jobbag_objects_om9(&selected).unwrap();
    let mut target = Controller::new(Document::new("native bag target"), Limits::default()).unwrap();
    target.paste_om9(&bag).unwrap();
    if instance { assert!(target.document().definitions.len() >= 2); }
    assert_eq!(target.coordinator().undo_len(), 1);
    target.undo().unwrap();
    assert!(target.document().entities.is_empty());
    target.redo().unwrap();
    let namespace = *target.document().import_namespaces.keys().next().unwrap();
    let capture = ExportCapture::all(target.coordinator(), target.document().id).unwrap();
    let inputs = exchange.prepare_merge_inputs(target.coordinator(), &capture, Some(namespace), &dir, &cancel, Usage::default()).unwrap();
    let wire = inputs.output_wire_plan(target.coordinator(), &cancel, Usage::default()).unwrap();
    let stage = wire.stage_retained_request(target.coordinator(), &dir, &cancel, Usage::default()).unwrap();
    let written = exchange.write_retained_private(target.coordinator(), &stage, &cancel, Usage::default()).unwrap();
    let validated = exchange.inspect_retained_private(target.coordinator(), written, &cancel, Usage::default()).unwrap();
    let losses = validated.losses().unwrap();
    let output = dir.join("result.3dm");
    validated.publish(target.coordinator(), &output, &cancel, Some(losses), Usage::default()).unwrap();
    assert!(output.metadata().unwrap().len() > 0);
    let reopened = Document::new("reopened native bag");
    let reopened_id = reopened.id;
    let mut reopened = TransactionCoordinator::new(World::new(vec![reopened], &Limits::default()).unwrap(), Limits::default()).unwrap();
    let imported = exchange.prepare_import(&reopened, ImportCapture::target(&reopened, reopened_id).unwrap(), &output, &dir, &ImportOptions::default(), &cancel).unwrap();
    let (prepared, _) = imported.prepare(&reopened, &cancel).unwrap();
    reopened.commit(prepared).unwrap();
    let entities = &reopened.world().documents[&reopened_id].entities;
    assert_eq!(entities.len(), target.document().entities.len());
    if instance {
        assert!(reopened.world().documents[&reopened_id].definitions.len() >= 2);
        let root = entities.values().find(|entity| matches!(entity.geometry, om9_core::geometry::Geometry::Instance(_)) && entity.placement == expected[0].1).unwrap();
        let actual_points = occurrence_points(&reopened.world().documents[&reopened_id], root);
        assert!(!actual_points.is_empty());
        assert_eq!(actual_points.len(), expected_points.len());
        for (actual, expected) in actual_points.iter().zip(expected_points) {
            for axis in 0..3 { assert!((actual[axis] - expected[axis]).abs() < 1e-8); }
        }
    } else {
        for (geometry, placement) in expected {
            assert!(entities.values().any(|entity| entity.geometry == geometry && entity.placement == placement));
        }
    }
    println!("native selected Job Bag result: {}", output.display());
}
