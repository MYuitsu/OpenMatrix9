// SPDX-License-Identifier: LGPL-2.1-or-later
//! Headless egui coverage of the Metal Weights builder controls.
#![cfg(feature = "occt-brep")]
use eframe::egui::{self, Event, Modifiers, Pos2, Rect, Shape, Vec2};
use om9_app::{session::InvocationOrigin, ui::MatrixApp};
use om9_core::model::Document;

fn frame(app: &mut MatrixApp, ctx: &egui::Context, events: Vec<Event>) -> egui::FullOutput {
    let mut output = ctx.run_ui(egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 900.))),
        events, ..Default::default()
    }, |ui| app.show(ui));
    output.textures_delta.clear();
    output
}

fn find_text(shape: &Shape, needle: &str) -> Option<Pos2> {
    match shape {
        Shape::Text(text) if text.galley.job.text == needle => Some(text.pos + text.galley.size() / 2.),
        Shape::Vec(shapes) => shapes.iter().find_map(|shape| find_text(shape, needle)),
        _ => None,
    }
}

fn texts(shape: &Shape, out: &mut Vec<String>) {
    match shape {
        Shape::Text(text) => out.push(text.galley.job.text.clone()),
        Shape::Vec(shapes) => for shape in shapes { texts(shape, out); },
        _ => {}
    }
}

fn click(app: &mut MatrixApp, ctx: &egui::Context, point: Pos2) {
    for pressed in [true, false] {
        frame(app, ctx, vec![Event::PointerMoved(point), Event::PointerButton {
            pos: point, button: egui::PointerButton::Primary, pressed, modifiers: Modifiers::NONE,
        }]);
    }
}

fn wait_metal(app: &mut MatrixApp, ctx: &egui::Context) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while app.controller.metal_worker_pending() {
        frame(app, ctx, vec![]);
        assert!(std::time::Instant::now() < deadline, "metal worker timeout");
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
}

#[test]
fn builder_opens_without_selection_and_calculates_after_selecting_a_solid() {
    let mut app = MatrixApp::new(Document::with_template_tolerance("metal panel")).unwrap();
    app.controller.submit("Box 10 20 30").unwrap();
    let id = *app.controller.document().entities.keys().next().unwrap();
    app.controller.clear_selection();
    app.controller.dispatch("OM9-TOOLS-006", InvocationOrigin::Menu).unwrap();
    assert!(app.controller.metal_panel_open());
    assert!(app.controller.metal_report().is_none());
    app.controller.select_objects(&[id]).unwrap();
    let before = app.controller.document().clone();
    let undo = app.controller.coordinator().undo_len();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    let out = frame(&mut app, &ctx, vec![]);
    let point = out.shapes.iter().find_map(|item| find_text(&item.shape, "Calculate"))
        .unwrap_or_else(|| { let mut found = Vec::new(); for item in &out.shapes { texts(&item.shape, &mut found); } panic!("Calculate button visible in builder: {found:?}") });
    click(&mut app, &ctx, point);
    assert!(app.controller.metal_worker_pending());
    wait_metal(&mut app, &ctx);
    let report = app.controller.metal_catalog_report().expect("catalog report");
    let gold = report.estimates.iter().find(|(_, result)| result.entries[0].material_id == "gold_18k_yellow").unwrap();
    assert!((gold.1.total_volume_mm3 - 6000.).abs() < 1e-6);
    assert!((gold.1.total_grams - 93.).abs() < 1e-6);
    let previous = report.clone();
    let out = frame(&mut app, &ctx, vec![]);
    let reference = out.shapes.iter().find_map(|item| find_text(&item.shape, "OM9 reference")).unwrap();
    click(&mut app, &ctx, reference);
    app.controller.submit("Calculate").unwrap();
    wait_metal(&mut app, &ctx);
    assert_eq!(app.controller.metal_catalog_report(), Some(&previous));
    click(&mut app, &ctx, reference);
    let out = frame(&mut app, &ctx, vec![]);
    let dropdown = out.shapes.iter().find_map(|item| find_text(&item.shape, "All visible materials")).unwrap();
    click(&mut app, &ctx, dropdown);
    let out = frame(&mut app, &ctx, vec![]);
    let platinum = out.shapes.iter().find_map(|item| find_text(&item.shape, "Platinum 950")).expect("material dropdown");
    click(&mut app, &ctx, platinum);
    assert_eq!(app.controller.metal_panel_material(), Some("platinum_950"));
    app.controller.submit("Calculate").unwrap();
    wait_metal(&mut app, &ctx);
    assert!((app.controller.metal_report().unwrap().grams - 124.8).abs() < 1e-6);
    app.controller.submit("MetalWeights all").unwrap();
    assert!(app.controller.metal_worker_pending());
    wait_metal(&mut app, &ctx);
    assert_eq!(app.controller.metal_catalog_report().unwrap().estimates.len(), app.controller.metal_registry().len());
    app.controller.submit("MetalWeights material=platinum_950").unwrap();
    assert!(app.controller.metal_worker_pending());
    wait_metal(&mut app, &ctx);
    assert!((app.controller.metal_report().unwrap().grams - 124.8).abs() < 1e-6);
    assert_eq!(app.controller.document(), &before);
    assert_eq!(app.controller.coordinator().undo_len(), undo);
}

#[cfg(feature = "test-fixtures")]
#[test]
fn object_material_panel_binds_density_and_calculates_selected_instance() {
    use om9_core::{exchange::RenderMaterial, geometry::Geometry, identity::MaterialId, model::{Definition, Entity}};
    let render = MaterialId::new();
    let mut document = Document::with_template_tolerance("metal object panel");
    document.exchange.materials.insert(render, RenderMaterial { id: render, name: "My render".into(), diffuse: [128; 4] });
    let seed = document.clone();
    let dir = std::env::temp_dir().join(format!("om9-metal-ui-binding-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&dir).unwrap();
    let path = dir.join("catalog.json");
    let mut app = MatrixApp::new(document).unwrap();
    app.load_user_metal_catalog(&path).unwrap();
    app.controller.submit("Box 10 5 2").unwrap();
    let source = app.controller.document().entities.values().next().unwrap().geometry.clone();
    let mut leaf = Entity::new("solid", source, None);
    leaf.attributes.material_source = 1;
    leaf.attributes.material = Some(render);
    let definition = Definition::new("definition", vec![leaf]);
    let root = Entity::new("instance", Geometry::Instance(definition.id), None);
    let id = root.id;
    app.controller.test_insert_metal_definitions(vec![definition], vec![root]).unwrap();
    app.controller.select_objects(&[id]).unwrap();
    app.controller.dispatch("OM9-TOOLS-006", InvocationOrigin::Menu).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    let out = frame(&mut app, &ctx, vec![]);
    let checkbox = out.shapes.iter().find_map(|item| find_text(&item.shape, "Use Object Materials")).unwrap();
    click(&mut app, &ctx, checkbox);
    assert!(app.controller.metal_panel_object_materials());
    let out = frame(&mut app, &ctx, vec![]);
    let dropdown = out.shapes.iter().find_map(|item| find_text(&item.shape, "Unassigned")).unwrap();
    click(&mut app, &ctx, dropdown);
    let out = frame(&mut app, &ctx, vec![]);
    let platinum = out.shapes.iter().find_map(|item| find_text(&item.shape, "Platinum 950")).unwrap();
    click(&mut app, &ctx, platinum);
    assert_eq!(app.controller.bound_metal_material(render), Some("platinum_950"));
    app.controller.submit("Calculate").unwrap();
    wait_metal(&mut app, &ctx);
    let report = app.controller.metal_report().unwrap();
    assert!((report.volume_mm3 - 100.).abs() < 1e-6);
    assert!((report.grams - 2.08).abs() < 1e-6);
    let mut reopened = MatrixApp::new(seed).unwrap();
    reopened.load_user_metal_catalog(&path).unwrap();
    assert_eq!(reopened.controller.bound_metal_material(render), Some("platinum_950"));
    std::fs::remove_file(path).unwrap();
    std::fs::remove_dir(dir).unwrap();
}

#[test]
fn builder_creates_a_persistent_user_material() {
    let dir = std::env::temp_dir().join(format!("om9-metal-ui-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&dir).unwrap();
    let path = dir.join("metal-catalog.json");
    let mut app = MatrixApp::new(Document::new("user material")).unwrap();
    app.load_user_metal_catalog(&path).unwrap();
    app.controller.dispatch("OM9-TOOLS-006", InvocationOrigin::Menu).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    let out = frame(&mut app, &ctx, vec![]);
    let header = out.shapes.iter().find_map(|item| find_text(&item.shape, "Add user material")).unwrap();
    click(&mut app, &ctx, header);
    frame(&mut app, &ctx, vec![Event::PointerMoved(Pos2::new(100., 600.)), Event::MouseWheel {
        unit: egui::MouseWheelUnit::Point, delta: Vec2::new(0., -1000.),
        modifiers: Modifiers::NONE, phase: egui::TouchPhase::Move,
    }]);
    frame(&mut app, &ctx, vec![]);
    for (key, value) in [("id", "my_alloy"), ("name", "My alloy"), ("density", "12.5"), ("provenance", "supplier data")] {
        let rect: Rect = ctx.memory(|memory| memory.data.get_temp(egui::Id::new(("metal-user-field", key)))).unwrap();
        click(&mut app, &ctx, rect.center());
        frame(&mut app, &ctx, vec![Event::Text(value.into())]);
    }
    let out = frame(&mut app, &ctx, vec![]);
    let save = out.shapes.iter().find_map(|item| find_text(&item.shape, "Save material")).unwrap();
    click(&mut app, &ctx, save);
    assert_eq!(app.controller.metal_registry().get("my_alloy").unwrap().density_g_cm3, 12.5);
    frame(&mut app, &ctx, vec![]);
    let edit: Rect = ctx.memory(|m| m.data.get_temp(egui::Id::new(("metal-user-edit", "my_alloy")))).unwrap();
    click(&mut app, &ctx, edit.center());
    frame(&mut app, &ctx, vec![Event::PointerMoved(Pos2::new(100., 600.)), Event::MouseWheel {
        unit: egui::MouseWheelUnit::Point, delta: Vec2::new(0., -1000.),
        modifiers: Modifiers::NONE, phase: egui::TouchPhase::Move,
    }]);
    frame(&mut app, &ctx, vec![]);
    let field: Rect = ctx.memory(|m| m.data.get_temp(egui::Id::new(("metal-user-field", "density")))).unwrap();
    click(&mut app, &ctx, field.center());
    let ctrl = Modifiers { ctrl: true, command: true, ..Modifiers::NONE };
    frame(&mut app, &ctx, vec![Event::Key { key: egui::Key::A, physical_key: None, pressed: true, repeat: false, modifiers: ctrl }]);
    frame(&mut app, &ctx, vec![Event::Text("13.5".into())]);
    let out = frame(&mut app, &ctx, vec![]);
    let save = out.shapes.iter().find_map(|item| find_text(&item.shape, "Save changes")).unwrap();
    click(&mut app, &ctx, save);
    assert_eq!(app.controller.metal_registry().get("my_alloy").unwrap().density_g_cm3, 13.5);
    let mut reopened = MatrixApp::new(Document::new("reopened")).unwrap();
    reopened.load_user_metal_catalog(&path).unwrap();
    assert_eq!(reopened.controller.metal_registry().get("my_alloy").unwrap().density_g_cm3, 13.5);
    let delete: Rect = ctx.memory(|m| m.data.get_temp(egui::Id::new(("metal-user-delete", "my_alloy")))).unwrap();
    click(&mut app, &ctx, delete.center());
    assert!(app.controller.metal_registry().get("my_alloy").is_err());
    reopened.load_user_metal_catalog(&path).unwrap();
    assert!(reopened.controller.metal_registry().get("my_alloy").is_err());
    std::fs::remove_file(path).unwrap();
    std::fs::remove_dir(dir).unwrap();
}
