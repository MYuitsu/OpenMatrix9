// SPDX-License-Identifier: LGPL-2.1-or-later
//! Metal Weights (`OM9-L2-04-19`): closed volumes, material density, grams/dwt, multi-material
//! breakdown and the open-shell refusal that leaves the previous report intact.
#![cfg(feature = "occt-brep")]

use om9_app::session::{Controller, InvocationOrigin, metal::{measure_metal_source_volume, measure_metal_source_volume_cancellable_admitted}};
use om9_core::{
    Error,
    brep::{Brep, BrepSourceAccess},
    budget::{Limits, Usage, memory::MemoryClass},
    geometry::Geometry,
    metal_weight::{DEFAULT_MATERIAL_ID, MetalMaterial},
    model::{Definition, Document, Entity},
};
use om9_kernel::creation::{self, Primitive};

struct Fixture {
    id: om9_core::identity::ObjectId,
    label: String,
    brep: Brep,
    placement: om9_core::geometry::Transform,
}

fn controller_with(entities: Vec<Fixture>) -> Controller {
    let limits = Limits::default();
    let mut controller =
        Controller::new(Document::new("metal-weights"), limits).expect("controller");
    for fixture in entities {
        let bytes = fixture.brep.storage_usage(&limits).unwrap().bytes;
        let mut reservation = controller
            .coordinator()
            .reserve_external_memory(MemoryClass::Kernel, bytes, None)
            .unwrap();
        reservation.set_allocated_bytes(bytes).unwrap();
        let source = controller
            .coordinator()
            .register_reserved_brep_candidate(fixture.brep, &mut reservation, Usage::default())
            .unwrap();
        let mut entity = Entity::new(fixture.label, Geometry::Brep(source), None);
        entity.id = fixture.id;
        entity.placement = fixture.placement;
        controller
            .commit_entity_reserved_brep("fixture", "fixture", entity, &mut reservation)
            .unwrap();
    }
    controller
}

fn box_entity(label: &str, size: [f64; 3]) -> Fixture {
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
    Fixture {
        id: om9_core::identity::ObjectId::new(),
        label: label.into(),
        brep,
        placement: om9_core::geometry::Transform::identity(),
    }
}

fn open_shell_entity() -> Fixture {
    let shell = creation::extrude_wire(
        &[[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [10.0, 2.0, 0.0]],
        [0.0, 0.0, 2.0],
        creation::DEFAULT_DEFLECTION_MM,
        &Limits::default(),
    )
    .expect("open side shell");
    Fixture {
        id: om9_core::identity::ObjectId::new(),
        label: "open-shell".into(),
        brep: shell,
        placement: om9_core::geometry::Transform::identity(),
    }
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 1e-6 * expected.abs().max(1.0),
        "expected {expected}, got {actual}"
    );
}

fn calculate_gold(controller: &mut Controller) -> om9_core::Result<()> {
    controller.set_metal_panel_material(Some(DEFAULT_MATERIAL_ID))?;
    controller.submit("Calculate")
}

#[test]
fn source_volume_counts_nested_placed_instances_and_rejects_partial_results() {
    use om9_core::geometry::Transform;
    let source = box_entity("box", [10.0, 5.0, 2.0]).brep.to_canonical();
    let leaf = Entity::new("leaf", Geometry::Brep(source), None);
    let inner = Definition::new("inner", vec![leaf]);
    let mut first = Entity::new("first", Geometry::Instance(inner.id), None);
    first.placement = Transform::translation([2.0, 0.0, 0.0]);
    let mut second = Entity::new("second", Geometry::Instance(inner.id), None);
    second.placement = Transform([[-2., 0., 0., 5.], [0., 3., 0., 0.], [0., 0., 1., 0.], [0., 0., 0., 1.]]);
    let outer = Definition::new("outer", vec![first, second]);
    let outer_id = outer.id;
    let mut root = Entity::new("root", Geometry::Instance(outer_id), None);
    root.placement = Transform::translation([10.0, 0.0, 0.0]);
    let mut document = Document::new("nested metal");
    document.definitions.insert(inner.id, inner);
    document.definitions.insert(outer_id, outer);
    assert_close(measure_metal_source_volume(&document, &root, &Limits::default()).unwrap(), 700.0);
    let cancel = om9_core::transaction::Cancellation::default();
    let mut calls = 0;
    assert_eq!(measure_metal_source_volume_cancellable_admitted(&document, &root, &Limits::default(), &cancel, &mut |_, _| {
        calls += 1; cancel.cancel(); Ok(())
    }), Err(Error::Canceled));
    assert_eq!(calls, 1, "cancellation stops traversal at the first admitted source");
    let admitted = om9_core::transaction::Cancellation::default();
    assert_eq!(measure_metal_source_volume_cancellable_admitted(&document, &root, &Limits::default(), &admitted,
        &mut |_, _| Err(Error::Budget("fixture admission"))), Err(Error::Budget("fixture admission")));
    let canceled = om9_core::transaction::Cancellation::default();
    canceled.cancel();
    assert_eq!(measure_metal_source_volume_cancellable_admitted(&document, &root, &Limits::default(), &canceled, &mut |_, _| panic!("pre-canceled traversal")), Err(Error::Canceled));

    let mut deep_root = root.clone();
    for _ in 0..4 {
        let wrapper = Definition::new("wrapper", vec![deep_root]);
        deep_root = Entity::new("wrapper instance", Geometry::Instance(wrapper.id), None);
        document.definitions.insert(wrapper.id, wrapper);
    }
    let tight = Limits { max_nodes: 4, ..Limits::default() };
    assert_eq!(measure_metal_source_volume(&document, &deep_root, &tight), Err(Error::Budget("metal weight occurrences")));
    let shallow = Limits { max_depth: 1, ..Limits::default() };
    assert!(matches!(measure_metal_source_volume(&document, &root, &shallow), Err(Error::Budget(_))));
    document.definitions.get_mut(&outer_id).unwrap().members.push(Entity::new("point", Geometry::Point([0.; 3]), None));
    assert!(matches!(measure_metal_source_volume(&document, &root, &Limits::default()), Err(Error::Invalid(_))));
    document.definitions.get_mut(&outer_id).unwrap().members.pop();
    document.definitions.get_mut(&outer_id).unwrap().members.push(Entity::new(
        "open shell", Geometry::Brep(open_shell_entity().brep.to_canonical()), None));
    assert_eq!(measure_metal_source_volume(&document, &root, &Limits::default()), Err(Error::Invalid("metal weight requires a closed solid")));
    document.definitions.get_mut(&outer_id).unwrap().members.pop();
    document.definitions.get_mut(&outer_id).unwrap().members.push(Entity::new("cycle", Geometry::Instance(outer_id), None));
    assert_eq!(measure_metal_source_volume(&document, &root, &Limits::default()), Err(Error::Invalid("metal weight instance cycle")));
}

#[cfg(feature = "test-fixtures")]
#[test]
fn object_materials_resolve_instance_parent_object_and_layer_without_partial_reports() {
    use om9_core::{exchange::RenderMaterial, identity::MaterialId, model::Layer};
    let gold = MaterialId::new();
    let platinum = MaterialId::new();
    let mut document = Document::with_template_tolerance("metal material binding");
    for (id, name) in [(gold, "gold render"), (platinum, "platinum render")] {
        document.exchange.materials.insert(id, RenderMaterial { id, name: name.into(), diffuse: [128; 4] });
    }
    let mut layer = Layer::new("platinum layer", [255; 4]);
    layer.material = Some(platinum);
    let layer_id = layer.id;
    document.layer_order.push(layer_id);
    document.layers.insert(layer_id, layer);
    let mut controller = Controller::new(document, Limits::default()).unwrap();
    controller.submit("Box 10 5 2").unwrap();
    let geometry = controller.document().entities.values().next().unwrap().geometry.clone();
    let mut inherited = Entity::new("inherited", geometry.clone(), None);
    inherited.attributes.material_source = 3;
    let mut direct = Entity::new("direct", geometry.clone(), None);
    direct.attributes.material_source = 1;
    direct.attributes.material = Some(platinum);
    let mut from_layer = Entity::new("layer", geometry, Some(layer_id));
    from_layer.attributes.material_source = 0;
    let definition = Definition::new("three materials", vec![inherited, direct, from_layer]);
    let mut root = Entity::new("root", Geometry::Instance(definition.id), None);
    root.attributes.material_source = 1;
    root.attributes.material = Some(gold);
    let root_id = root.id;
    controller.test_insert_metal_definitions(vec![definition], vec![root]).unwrap();
    controller.select_objects(&[root_id]).unwrap();
    let before = controller.document().clone();
    let undo = controller.coordinator().undo_len();
    controller.dispatch("OM9-TOOLS-006", InvocationOrigin::Menu).unwrap();
    controller.calculate_metal_weights(None).unwrap();
    let prior = controller.metal_report().unwrap().clone();
    controller.bind_metal_material(gold, Some("gold_18k_yellow")).unwrap();
    controller.set_metal_panel_object_materials(true);
    assert_eq!(controller.submit("Calculate"), Err(Error::Missing("metal material density binding")));
    assert_eq!(controller.metal_report(), Some(&prior));
    controller.bind_metal_material(platinum, Some("platinum_950")).unwrap();
    controller.submit("Calculate").unwrap();
    let report = controller.metal_report().unwrap().clone();
    assert_eq!(report.parts.len(), 3);
    assert_close(report.volume_mm3, 300.);
    assert_close(report.grams, 5.71);
    assert_eq!(report.parts[0].material_id, "gold_18k_yellow");
    assert_eq!(report.parts[1].material_id, "platinum_950");
    assert_eq!(report.parts[2].material_id, "platinum_950");
    assert!(report.parts.iter().all(|part| part.object == root_id));
    controller.bind_metal_material(platinum, None).unwrap();
    assert_eq!(controller.submit("Calculate"), Err(Error::Missing("metal material density binding")));
    assert_eq!(controller.metal_report(), Some(&report));
    controller.bind_metal_material(platinum, Some("platinum_950")).unwrap();
    controller.enable_background_metal_weights();
    controller.submit("Calculate").unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    loop {
        if let Some(result) = controller.poll_metal_worker() { result.unwrap(); break; }
        assert!(std::time::Instant::now() < deadline, "object-material worker timeout");
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    assert_eq!(controller.metal_report(), Some(&report));
    controller.submit("Calculate").unwrap();
    controller.bind_metal_material(platinum, None).unwrap();
    assert_eq!(controller.poll_metal_worker(), Some(Err(Error::Stale("metal weight context"))));
    assert_eq!(controller.metal_report(), Some(&report));
    assert_eq!(controller.document(), &before);
    assert_eq!(controller.coordinator().undo_len(), undo);
}

#[cfg(feature = "test-fixtures")]
#[test]
fn nested_instance_selection_routes_and_refusals_keep_document_and_undo() {
    use om9_core::geometry::Transform;
    let mut controller = Controller::new(Document::with_template_tolerance("metal instance"), Limits::default()).unwrap();
    controller.submit("Box 10 5 2").unwrap();
    let source_id = *controller.document().entities.keys().next().unwrap();
    let source = controller.document().entities[&source_id].geometry.clone();
    let inner = Definition::new("inner", vec![Entity::new("solid", source, None)]);
    let first = Entity::new("first", Geometry::Instance(inner.id), None);
    let mut second = Entity::new("second", Geometry::Instance(inner.id), None);
    second.placement = Transform([[-2., 0., 0., 20.], [0., 3., 0., 0.], [0., 0., 1., 0.], [0., 0., 0., 1.]]);
    let outer = Definition::new("outer", vec![first, second]);
    let mut root = Entity::new("nested", Geometry::Instance(outer.id), None);
    root.placement = Transform::translation([10., 0., 0.]);
    let root_id = root.id;
    let mixed = Definition::new("mixed", vec![
        Entity::new("valid", Geometry::Instance(inner.id), None),
        Entity::new("point", Geometry::Point([0.; 3]), None),
    ]);
    let bad = Entity::new("mixed root", Geometry::Instance(mixed.id), None);
    let bad_id = bad.id;
    controller.test_insert_metal_definitions(vec![inner, outer, mixed], vec![root, bad]).unwrap();
    controller.select_objects(&[root_id]).unwrap();
    let before = controller.document().clone();
    let undo = controller.coordinator().undo_len();
    controller.dispatch("OM9-TOOLS-006", InvocationOrigin::Menu).unwrap();
    assert!(controller.metal_report().is_none());
    calculate_gold(&mut controller).unwrap();
    let report = controller.metal_report().unwrap().clone();
    assert_close(report.volume_mm3, 700.);
    assert_close(report.grams, 10.85);
    controller.submit("gvMetalWeights").unwrap();
    calculate_gold(&mut controller).unwrap();
    assert_eq!(controller.metal_report(), Some(&report));
    controller.request_viewport_detail(om9_app::display::ViewportDetail::High).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while controller.viewport_detail_pending() {
        if let Some(result) = controller.poll_viewport_detail() { result.unwrap(); }
        assert!(std::time::Instant::now() < deadline, "viewport detail timed out");
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    controller.submit("gvMetalWeights").unwrap();
    calculate_gold(&mut controller).unwrap();
    assert_eq!(controller.metal_report(), Some(&report));
    controller.select_objects(&[bad_id]).unwrap();
    assert!(controller.calculate_metal_weights(None).is_err());
    assert_eq!(controller.metal_report(), Some(&report));
    assert_eq!(controller.document(), &before);
    assert_eq!(controller.coordinator().undo_len(), undo);
}


#[test]
fn the_icon_reports_a_closed_box_in_grams_and_pennyweights_without_editing() {
    let entity = box_entity("box", [10.0, 5.0, 2.0]);
    let id = entity.id;
    let mut controller = controller_with(vec![entity]);
    let undo = controller.coordinator().undo_len();
    let before = controller.document().clone();
    controller.select_objects(&[id]).unwrap();

    controller
        .dispatch("OM9-TOOLS-006", InvocationOrigin::Menu)
        .expect("metal weights icon");
    assert!(controller.metal_report().is_none(), "opening waits for Calculate");
    calculate_gold(&mut controller).unwrap();

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
    assert_eq!(controller.coordinator().undo_len(), undo);
}

#[test]
fn the_command_line_matches_the_icon_and_accepts_a_material() {
    let entity = box_entity("box", [10.0, 5.0, 2.0]);
    let id = entity.id;
    let mut controller = controller_with(vec![entity]);
    controller.select_objects(&[id]).unwrap();

    controller.submit("MetalWeights").expect("typed command");
    assert!(controller.metal_report().is_none());
    calculate_gold(&mut controller).unwrap();
    let default = controller.metal_report().expect("report").clone();
    controller.close_metal_panel();
    controller.submit("MetalWeights calculate").unwrap();
    assert_eq!(controller.metal_report(), Some(&default));
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
    let undo = controller.coordinator().undo_len();
    controller.select_objects(&[closed_id]).unwrap();
    controller
        .dispatch("OM9-TOOLS-006", InvocationOrigin::Menu)
        .expect("closed report");
    calculate_gold(&mut controller).unwrap();
    let previous = controller.metal_report().expect("report").clone();
    let document = controller.document().clone();

    controller.select_objects(&[open_id]).unwrap();
    assert_eq!(
        calculate_gold(&mut controller),
        Err(Error::Invalid("metal weight requires a closed solid"))
    );
    assert_eq!(controller.metal_report(), Some(&previous));
    assert_eq!(controller.document(), &document);
    assert_eq!(controller.coordinator().undo_len(), undo);
}

#[test]
fn a_missing_density_is_a_typed_refusal_not_a_zero_mass() {
    let entity = box_entity("box", [10.0, 5.0, 2.0]);
    let id = entity.id;
    let mut controller = controller_with(vec![entity]);
    let undo = controller.coordinator().undo_len();
    controller.select_objects(&[id]).unwrap();
    controller
        .dispatch("OM9-TOOLS-006", InvocationOrigin::Menu)
        .expect("valid report");
    calculate_gold(&mut controller).unwrap();
    let previous = controller.metal_report().expect("report").clone();

    assert_eq!(
        controller.calculate_metal_weights(Some("missing_material")),
        Err(Error::Missing("metal material density"))
    );
    assert_eq!(controller.metal_report(), Some(&previous));
    assert_eq!(controller.coordinator().undo_len(), undo);
}

#[test]
fn a_multi_material_selection_reports_each_body_and_the_total() {
    let a = box_entity("body-a", [10.0, 5.0, 2.0]); // 100 mm^3
    let b = box_entity("body-b", [10.0, 10.0, 2.0]); // 200 mm^3
    let (a_id, b_id) = (a.id, b.id);
    let mut controller = controller_with(vec![a, b]);
    let undo = controller.coordinator().undo_len();
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
        .calculate_metal_weights_with(&[(a_id, "synthetic_material"), (b_id, "light_material")])
        .expect("breakdown");
    assert_eq!(report.material_id, "mixed");
    assert_eq!(report.parts.len(), 2);
    assert_close(report.parts[0].grams, 1.0);
    assert_close(report.parts[1].grams, 1.6);
    assert_close(report.grams, 2.6);
    assert_close(report.volume_mm3, 300.0);
    assert_eq!(report.parts[0].material_id, "synthetic_material");
    assert_eq!(report.parts[1].material_id, "light_material");
    assert_eq!(controller.coordinator().undo_len(), undo);
}

#[test]
fn an_assignment_that_omits_a_selected_body_is_refused() {
    let a = box_entity("body-a", [10.0, 5.0, 2.0]);
    let b = box_entity("body-b", [10.0, 10.0, 2.0]);
    let a_id = a.id;
    let mut controller = controller_with(vec![a, b]);
    controller.select_objects(&[a_id]).unwrap();
    // The selection has one body, but the assignment names a different pair of ids.
    let other = controller
        .document()
        .entities
        .keys()
        .copied()
        .find(|id| *id != a_id);
    let Some(other) = other else {
        panic!("second body");
    };
    assert_eq!(
        controller
            .calculate_metal_weights_with(&[(a_id, "gold_18k_yellow"), (other, "gold_18k_yellow")]),
        Err(Error::Invalid("metal weight material assignment"))
    );
}

#[test]
fn a_translated_solid_keeps_its_source_volume_and_undo() {
    let mut solid = box_entity("placed", [10.0, 20.0, 30.0]);
    solid.placement = om9_core::geometry::Transform::translation([17.0, -8.0, 4.0]);
    let id = solid.id;
    let mut controller = controller_with(vec![solid]);
    controller.select_objects(&[id]).unwrap();
    let before = controller.document().clone();
    let undo = controller.coordinator().undo_len();
    controller.submit("gvMetalWeights").unwrap();
    calculate_gold(&mut controller).unwrap();
    assert_close(controller.metal_report().unwrap().volume_mm3, 6000.0);
    assert_close(controller.metal_report().unwrap().grams, 93.0);
    assert_eq!(controller.document(), &before);
    assert_eq!(controller.coordinator().undo_len(), undo);
}

#[test]
fn a_scaled_solid_uses_placed_volume_without_editing() {
    let mut solid = box_entity("scaled", [10.0, 5.0, 2.0]);
    solid.placement = om9_core::geometry::Transform([
        [-2., 0., 0., 5.], [0., 3., 0., 0.], [0., 0., 1., 0.], [0., 0., 0., 1.],
    ]);
    let id = solid.id;
    let mut controller = controller_with(vec![solid]);
    controller.select_objects(&[id]).unwrap();
    let before = controller.document().clone();
    let undo = controller.coordinator().undo_len();
    controller.submit("gvMetalWeights").unwrap();
    calculate_gold(&mut controller).unwrap();
    assert_close(controller.metal_report().unwrap().volume_mm3, 600.0);
    assert_close(controller.metal_report().unwrap().grams, 9.3);
    assert_eq!(controller.document(), &before);
    assert_eq!(controller.coordinator().undo_len(), undo);
}

#[test]
fn catalog_command_reports_all_densities_and_refuses_without_losing_report() {
    let first = box_entity("first", [10., 5., 2.]);
    let first_id = first.id;
    let second = box_entity("second", [10., 10., 2.]);
    let second_id = second.id;
    let open = open_shell_entity();
    let open_id = open.id;
    let mut controller = controller_with(vec![first, second, open]);
    controller.metal_registry_mut().insert(MetalMaterial::test_only("fixture_10", 10., "fixture").unwrap()).unwrap();
    controller.select_objects(&[first_id, second_id]).unwrap();
    let document = controller.document().clone();
    let undo = controller.coordinator().undo_len();
    controller.submit("gvMetalWeights all").unwrap();
    let report = controller.metal_catalog_report().unwrap().clone();
    assert_eq!(report.estimates.len(), controller.metal_registry().len());
    let (_, fixture) = report.estimates.iter().find(|(_, b)| b.entries[0].material_id == "fixture_10").unwrap();
    assert_eq!(fixture.entries.len(), 2);
    assert_close(fixture.total_volume_mm3, 300.);
    assert_close(fixture.total_grams, 3.);
    assert_close(fixture.total_dwt, 3. / 1.555_173_84);
    assert!(report.text.contains("fixture_10"));
    controller.dispatch("OM9-TOOLS-006", InvocationOrigin::Menu).unwrap();
    controller.set_metal_panel_filters(false, true);
    controller.submit("Calculate").unwrap();
    let filtered = controller.metal_catalog_report().unwrap().clone();
    assert_eq!(filtered.estimates.len(), 1);
    assert_eq!(filtered.estimates[0].1.entries[0].material_id, "fixture_10");
    controller.set_metal_panel_filters(false, false);
    assert_eq!(controller.submit("Calculate"), Err(Error::Missing("metal material density")));
    assert_eq!(controller.metal_catalog_report(), Some(&filtered));
    controller.set_metal_panel_filters(true, true);
    controller.submit("Calculate").unwrap();
    controller.select_objects(&[first_id, open_id]).unwrap();
    assert!(controller.submit("MetalWeights all").is_err());
    assert_eq!(controller.metal_catalog_report(), Some(&report));
    assert!(controller.submit("MetalWeights all material=gold_18k_yellow").is_err());
    assert_eq!(controller.metal_catalog_report(), Some(&report));
    assert_eq!(controller.document(), &document);
    assert_eq!(controller.coordinator().undo_len(), undo);
}

#[test]
fn background_measurement_publishes_only_current_selection_and_cancels_cleanly() {
    let first = box_entity("first", [10., 20., 30.]);
    let first_id = first.id;
    let second = box_entity("second", [10., 5., 2.]);
    let second_id = second.id;
    let mut controller = controller_with(vec![first, second]);
    controller.select_objects(&[first_id]).unwrap();
    controller.dispatch("OM9-TOOLS-006", InvocationOrigin::Menu).unwrap();
    controller.set_metal_panel_material(Some(DEFAULT_MATERIAL_ID)).unwrap();
    controller.enable_background_metal_weights();
    let document = controller.document().clone();
    let undo = controller.coordinator().undo_len();
    controller.submit("Calculate").unwrap();
    assert!(controller.metal_worker_pending());
    assert!(controller.metal_report().is_none(), "no result before worker publication");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    loop {
        if let Some(result) = controller.poll_metal_worker() { result.unwrap(); break; }
        assert!(std::time::Instant::now() < deadline, "worker timeout");
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    let previous = controller.metal_report().unwrap().clone();
    assert_close(previous.volume_mm3, 6000.);
    controller.submit("Calculate").unwrap();
    controller.select_objects(&[second_id]).unwrap();
    assert_eq!(controller.poll_metal_worker(), Some(Err(Error::Stale("metal weight context"))));
    assert_eq!(controller.metal_report(), Some(&previous));
    // The native call may still be draining after cancellation; do not start a second
    // native operation until its exclusive permit has actually been released.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    loop {
        match controller.submit("Calculate") {
            Ok(()) => break,
            Err(Error::Busy) => { assert!(std::time::Instant::now() < deadline); std::thread::sleep(std::time::Duration::from_millis(2)); }
            Err(error) => panic!("worker restart: {error}"),
        }
    }
    controller.cancel_metal_worker();
    assert!(!controller.metal_worker_pending());
    assert_eq!(controller.metal_report(), Some(&previous));
    assert_eq!(controller.document(), &document);
    assert_eq!(controller.coordinator().undo_len(), undo);
}

#[test]
fn background_measurement_rejects_changed_material_state() {
    let body = box_entity("body", [10., 5., 2.]);
    let id = body.id;
    let mut controller = controller_with(vec![body]);
    controller.select_objects(&[id]).unwrap();
    controller.dispatch("OM9-TOOLS-006", InvocationOrigin::Menu).unwrap();
    controller.enable_background_metal_weights();
    controller.submit("Calculate").unwrap();
    controller.set_metal_panel_material(Some("platinum_950")).unwrap();
    assert_eq!(controller.poll_metal_worker(), Some(Err(Error::Stale("metal weight context"))));
    assert!(controller.metal_catalog_report().is_none());
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    loop {
        match controller.submit("Calculate") {
            Ok(()) => break,
            Err(Error::Busy) => { assert!(std::time::Instant::now() < deadline); std::thread::sleep(std::time::Duration::from_millis(2)); }
            Err(error) => panic!("worker restart: {error}"),
        }
    }
    controller.metal_registry_mut().insert(MetalMaterial::test_only("new_density", 10., "fixture").unwrap()).unwrap();
    assert_eq!(controller.poll_metal_worker(), Some(Err(Error::Stale("metal weight context"))));
    assert!(controller.metal_report().is_none());
}

#[test]
fn background_measurement_rejects_document_changes_before_publication() {
    let body = box_entity("body", [10., 5., 2.]);
    let id = body.id;
    let mut controller = controller_with(vec![body]);
    controller.select_objects(&[id]).unwrap();
    controller.dispatch("OM9-TOOLS-006", InvocationOrigin::Menu).unwrap();
    controller.calculate_metal_weights(None).unwrap();
    let previous = controller.metal_report().unwrap().clone();
    controller.enable_background_metal_weights();
    controller.submit("Calculate").unwrap();
    let extra = box_entity("new body", [1., 1., 1.]);
    let limits = Limits::default();
    let bytes = extra.brep.storage_usage(&limits).unwrap().bytes;
    let mut reservation = controller.coordinator().reserve_external_memory(MemoryClass::Kernel, bytes, None).unwrap();
    reservation.set_allocated_bytes(bytes).unwrap();
    let source = controller.coordinator().register_reserved_brep_candidate(extra.brep, &mut reservation, Usage::default()).unwrap();
    let mut entity = Entity::new(extra.label, Geometry::Brep(source), None);
    entity.id = extra.id;
    controller.commit_entity_reserved_brep("fixture", "fixture", entity, &mut reservation).unwrap();
    assert_eq!(controller.poll_metal_worker(), Some(Err(Error::Stale("metal weight context"))));
    assert_eq!(controller.metal_report(), Some(&previous));
}

#[test]
fn background_result_does_not_overwrite_a_newer_synchronous_report() {
    let body = box_entity("body", [10., 5., 2.]);
    let id = body.id;
    let mut controller = controller_with(vec![body]);
    controller.select_objects(&[id]).unwrap();
    controller.dispatch("OM9-TOOLS-006", InvocationOrigin::Menu).unwrap();
    controller.enable_background_metal_weights();
    controller.submit("Calculate").unwrap();
    let newer = controller.calculate_metal_weights(Some("platinum_950")).unwrap();
    assert_eq!(controller.poll_metal_worker(), Some(Err(Error::Stale("metal weight context"))));
    assert_eq!(controller.metal_report(), Some(&newer));
    assert!(controller.metal_catalog_report().is_none());
}

#[test]
fn background_measurement_refuses_unavailable_reservation_before_starting() {
    let point = Entity::new("point", Geometry::Point([0.; 3]), None);
    let id = point.id;
    let mut document = Document::new("tight metal worker");
    document.entities.insert(id, point);
    let limits = Limits { max_bytes: 32 * 1024 * 1024, ..Limits::default() };
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&[id]).unwrap();
    let before = controller.document().clone();
    controller.dispatch("OM9-TOOLS-006", InvocationOrigin::Menu).unwrap();
    controller.enable_background_metal_weights();
    assert!(matches!(controller.submit("Calculate"), Err(Error::Budget(_))));
    assert!(!controller.metal_worker_pending());
    assert!(controller.metal_catalog_report().is_none());
    assert_eq!(controller.document(), &before);
    assert_eq!(controller.coordinator().undo_len(), 0);
}
