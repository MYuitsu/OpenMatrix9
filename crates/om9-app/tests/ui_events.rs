// SPDX-License-Identifier: LGPL-2.1-or-later
use eframe::egui::{self, Event, Key, Modifiers, Rect, Vec2};
use om9_app::ui::MatrixApp;
use om9_core::model::Document;
#[path = "common/gpu_frames.rs"]
mod gpu_frames;
fn frame(app: &mut MatrixApp, ctx: &egui::Context, events: Vec<Event>) -> egui::FullOutput {
    let input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(
            egui::Pos2::ZERO,
            Vec2::new(1280., 800.),
        )),
        events,
        ..Default::default()
    };
    let mut output = ctx.run_ui(input, |ui| app.show(ui));
    // This test exercises the widget reducer without a renderer. Native tests
    // separately apply these deltas through wgpu and inspect the screenshot.
    output.textures_delta.clear();
    output
}
fn key(key: Key, modifiers: Modifiers) -> Event {
    Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers,
    }
}
fn wait_for_history(app: &mut MatrixApp, ctx: &egui::Context) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while app.controller.history_worker_pending() {
        assert!(std::time::Instant::now() < deadline, "{}", app.status_line());
        frame(app, ctx, vec![]);
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}

fn text(app: &mut MatrixApp, ctx: &egui::Context, value: &str) {
    frame(app, ctx, vec![Event::Text(value.into())]);
    frame(app, ctx, vec![key(Key::Enter, Modifiers::NONE)]);
}

#[test]
fn clicking_dust_marquire_opens_its_sample_size() {
    use om9_app::session::{InvocationOrigin, gem::LOADER};
    use om9_core::gem::{GemCut, GemShape};

    let mut app = MatrixApp::new(Document::new("gem gallery")).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    app.controller.dispatch(LOADER, InvocationOrigin::Menu).unwrap();
    frame(&mut app, &ctx, vec![]);
    click_text(&mut app, &ctx, "Dust Marquire");
    let session = app.controller.gem_session().unwrap();
    assert_eq!(session.cut, GemCut::Diamond);
    assert_eq!(session.shape, GemShape::DustMarquire);
    assert_eq!(session.dimensions, [11.52, 6.04643, 3.93127]);
    let output = frame(&mut app, &ctx, vec![]);
    assert!(output.shapes.iter().any(|shape| matches!(&shape.shape,
        egui::Shape::Text(text) if text.galley.text() == "11.52 × 6.05 mm"
    )));
}

#[test]
fn ctrl_z_undoes_document_with_idle_command_focus_but_not_while_editing() {
    let mut app = MatrixApp::new(Document::new("ctrl-z")).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    text(&mut app, &ctx, "Point");
    text(&mut app, &ctx, "1,2,3");
    assert_eq!(app.controller.document().entities.len(), 1);
    let input = app.command_input_rect().unwrap().center();
    click_at(&mut app, &ctx, input);
    assert!(ctx.memory(|m| m.has_focus(egui::Id::new("matrix-command-transcript"))));
    let ctrl = Modifiers { ctrl: true, command: true, ..Modifiers::NONE };
    frame(&mut app, &ctx, vec![Event::Text("draft".into())]);
    assert_eq!(app.transcript.draft(), "draft");
    frame(&mut app, &ctx, vec![key(Key::Z, ctrl)]);
    assert!(!app.controller.history_worker_pending());
    assert_eq!(app.controller.document().entities.len(), 1);
    assert_eq!(app.transcript.draft(), "", "Ctrl+Z undoes the draft edit");

    let mut release = key(Key::Z, ctrl);
    if let Event::Key { pressed, .. } = &mut release {
        *pressed = false;
    }
    frame(&mut app, &ctx, vec![release]);
    frame(&mut app, &ctx, vec![key(Key::Z, ctrl)]);
    wait_for_history(&mut app, &ctx);
    assert!(app.controller.document().entities.is_empty());
    assert_eq!(app.controller.coordinator().redo_len(), 1);
}
#[test]
fn rendered_cmd_events_build_line_undo_redo_and_cancel_without_direct_model_writes() {
    let mut app = MatrixApp::new(Document::new("egui")).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![]);
    assert_eq!(app.view_rects.len(), 4);
    text(&mut app, &ctx, "Line");
    assert!(app.controller.prompt().starts_with("Start of line"));
    text(&mut app, &ctx, "0,0,0");
    assert!(app.controller.prompt().starts_with("End of line"));
    assert!(app.controller.document().entities.is_empty());
    text(&mut app, &ctx, "12,6,0");
    let document = app.controller.document();
    assert_eq!(document.entities.len(), 1);
    let line = document.entities.values().next().unwrap();
    assert!(matches!(
        line.geometry,
        om9_core::geometry::Geometry::Curve(om9_core::geometry::Curve::Line {
            start: [0., 0., 0.], end: [12., 6., 0.], ..
        })
    ));
    let line_id = line.id;
    assert_eq!(app.controller.coordinator().undo_len(), 1);
    text(&mut app, &ctx, "Undo");
    wait_for_history(&mut app, &ctx);
    assert!(app.controller.document().entities.is_empty());
    text(&mut app, &ctx, "Redo");
    wait_for_history(&mut app, &ctx);
    assert!(
        app.controller.document().entities.contains_key(&line_id),
        "{} / {:?}",
        app.status_line(),
        app.transcript.text()
    );
    text(&mut app, &ctx, "Line");
    text(&mut app, &ctx, "2,2,0");
    frame(&mut app, &ctx, vec![key(Key::Escape, Modifiers::NONE)]);
    assert!(!app.controller.is_running());
    assert_eq!(app.controller.coordinator().undo_len(), 1);
    assert_eq!(app.controller.document().entities.len(), 1);
}
#[test]
fn actual_editor_select_all_cut_paste_cannot_mutate_transcript_prefix_or_execute_paste() {
    let mut app = MatrixApp::new(Document::new("text")).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    text(&mut app, &ctx, "Point");
    text(&mut app, &ctx, "1,2,3");
    assert_eq!(app.controller.document().entities.len(), 1);
    let input = app.command_input_rect().unwrap().center();
    click_at(&mut app, &ctx, input);
    assert!(ctx.memory(|m| m.has_focus(egui::Id::new("matrix-command-transcript"))));
    frame(&mut app, &ctx, vec![Event::Text("draft".into())]);
    assert_eq!(app.transcript.draft(), "draft");
    let history = app.transcript.history().to_owned();
    let prompt = app.transcript.prompt().to_owned();
    frame(
        &mut app,
        &ctx,
        vec![
            key(
                Key::A,
                Modifiers {
                    ctrl: true,
                    command: true,
                    ..Modifiers::NONE
                },
            ),
            Event::Cut,
        ],
    );
    assert_eq!(app.transcript.history(), history);
    assert_eq!(app.transcript.prompt(), prompt);
    assert_eq!(app.transcript.draft(), "");
    assert_eq!(app.controller.document().entities.len(), 1);
    frame(
        &mut app,
        &ctx,
        vec![Event::Paste("Line\n0,0,0\n9,9,0".into())],
    );
    assert_eq!(app.transcript.draft(), "Line 0,0,0 9,9,0");
    assert_eq!(app.transcript.history(), history);
    assert_eq!(app.transcript.prompt(), prompt);
    assert_eq!(app.controller.document().entities.len(), 1);
    assert!(!app.controller.is_running());
}

fn click_f6_row(app: &mut MatrixApp, ctx: &egui::Context, label: &str) {
    frame(app, ctx, vec![]);
    frame(app, ctx, vec![key(Key::F6, Modifiers::NONE)]);
    frame(app, ctx, vec![]);
    click_text(app, ctx, label);
}

fn click_text(app: &mut MatrixApp, ctx: &egui::Context, label: &str) {
    let output = frame(app, ctx, vec![]);
    let point = output
        .shapes
        .iter()
        .find_map(|s| match &s.shape {
            egui::Shape::Text(t) if t.galley.text() == label => {
                let rect = Rect::from_min_size(t.pos, t.galley.size());
                s.clip_rect.contains(rect.center()).then_some(rect.center())
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("visible F6 row: {label}"));
    click_at(app, ctx, point);
}

fn click_at(app: &mut MatrixApp, ctx: &egui::Context, point: egui::Pos2) {
    frame(app, ctx, vec![Event::PointerMoved(point)]);
    for pressed in [true, false] {
        frame(
            app,
            ctx,
            vec![Event::PointerButton {
                pos: point,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: Modifiers::NONE,
            }],
        );
    }
}

#[test]
fn click_factory_editor_footer_cancels_and_saves_a_long_group() {
    let mut app = MatrixApp::new(Document::new("F6 editor")).unwrap();
    let ctx = egui::Context::default();
    let original = app.f6_custom.clone();
    click_f6_row(&mut app, &ctx, "⚙");
    click_text(&mut app, &ctx, "+");
    click_text(&mut app, &ctx, "Cancel");
    assert_eq!(app.f6_custom, original);
    click_text(&mut app, &ctx, "⚙");
    click_text(&mut app, &ctx, "Save");
    assert!(!app.f6_custom.replaced_groups.is_empty());
    assert!(app.f6_custom.entries.iter().any(|row| row.category.marker().is_some()));
}

#[test]
fn click_factory_points_off_selected_keeps_the_other_curve_grips() {
    use om9_core::{
        geometry::{Curve, Geometry},
        model::Entity,
    };
    use om9_scene::{camera::Viewport, workspace::ViewId};
    let mut doc = Document::with_template_tolerance("F6 grip owners");
    let mut ids = vec![];
    for y in [0., 8.] {
        let entity = Entity::new(
            "curve",
            Geometry::Curve(Curve::Line {
                domain: [0., 1.],
                start: [-5., y, 0.],
                end: [5., y, 0.],
            }),
            None,
        );
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut app = MatrixApp::new(doc).unwrap();
    app.controller.select_objects(&ids).unwrap();
    app.controller.submit("PointsOn").unwrap();
    let handles = app.controller.top11_grip_handles().unwrap();
    let first = *handles.iter().find(|g| g.object == ids[0]).unwrap();
    app.controller
        .select_grips(&[first], om9_app::session::GripPick::Replace);
    app.f6_mode = om9_app::f6::catalog::Mode::Curve;
    let ctx = egui::Context::default();
    click_f6_row(&mut app, &ctx, "Points Off Selected");
    assert!(!app.f6_open);
    assert!(app.controller.prompt().contains("turn points off"));
    let view = ViewId::LookingDown;
    let rect = app.view_rects[&view];
    let viewport = Viewport {
        origin: [rect.left() as f64, rect.top() as f64],
        size: [rect.width() as f64, rect.height() as f64],
    };
    let pixel = app.controller.workspace.views[&view]
        .camera
        .project([1., 0., 0.], viewport)
        .unwrap()
        .unwrap()
        .screen;
    let pos = egui::pos2(pixel[0] as f32, pixel[1] as f32);
    frame(&mut app, &ctx, vec![Event::PointerMoved(pos)]);
    for pressed in [true, false] {
        frame(
            &mut app,
            &ctx,
            vec![Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: Modifiers::NONE,
            }],
        );
    }
    assert_eq!(app.controller.selected_ordered(), [ids[0]]);
    frame(&mut app, &ctx, vec![key(Key::Enter, Modifiers::NONE)]);
    assert!(!app.controller.is_running());
    let remaining = app.controller.top11_grip_handles().unwrap();
    assert_eq!(remaining.len(), 2);
    assert!(remaining.iter().all(|g| g.object == ids[1]));
    assert_eq!(app.controller.coordinator().undo_len(), 0);
}

#[test]
fn click_factory_show_selected_then_pick_hidden_geometry_in_the_viewport() {
    use om9_core::{geometry::Geometry, model::Entity};
    use om9_scene::{camera::Viewport, workspace::ViewId};
    let mut doc = Document::with_template_tolerance("ShowSelected by mouse");
    let mut ids = vec![];
    for (label, x, hidden) in [
        ("visible at same pixel", 0., false),
        ("hidden", 0., true),
        ("other hidden", 8., true),
    ] {
        let mut entity = Entity::new(label, Geometry::Point([x, 0., 0.]), None);
        entity.attributes.hidden = hidden;
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut app = MatrixApp::new(doc).unwrap();
    let ctx = egui::Context::default();
    click_f6_row(&mut app, &ctx, "ShowSelected");
    assert!(!app.f6_open);
    assert!(app.controller.prompt().contains("objects to show"));
    let view = ViewId::LookingDown;
    let rect = app.view_rects[&view];
    let viewport = Viewport {
        origin: [rect.left() as f64, rect.top() as f64],
        size: [rect.width() as f64, rect.height() as f64],
    };
    let pixel = app.controller.workspace.views[&view]
        .camera
        .project([0.; 3], viewport)
        .unwrap()
        .unwrap()
        .screen;
    let pos = egui::pos2(pixel[0] as f32, pixel[1] as f32);
    frame(&mut app, &ctx, vec![Event::PointerMoved(pos)]);
    for pressed in [true, false] {
        frame(
            &mut app,
            &ctx,
            vec![Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: Modifiers::NONE,
            }],
        );
    }
    assert_eq!(app.controller.selected_ordered(), [ids[1]]);
    frame(&mut app, &ctx, vec![key(Key::Enter, Modifiers::NONE)]);
    assert!(!app.controller.is_running());
    assert!(
        !app.controller.document().entities[&ids[1]]
            .attributes
            .hidden
    );
    assert!(
        app.controller.document().entities[&ids[2]]
            .attributes
            .hidden
    );
    app.controller.undo().unwrap();
    assert!(
        app.controller.document().entities[&ids[1]]
            .attributes
            .hidden
    );
}

#[test]
fn click_factory_mesh_outline_in_perspective_matches_the_projected_box() {
    use om9_core::{
        cage::box_mesh,
        geometry::{Curve, Geometry},
        model::Entity,
    };
    use om9_scene::camera::Viewport;
    let corners = std::array::from_fn(|i| {
        std::array::from_fn(|axis| if i & (1 << axis) != 0 { 4. } else { -1. })
    });
    let mut doc = Document::with_template_tolerance("Mesh Outline perspective");
    let source = Entity::new("box", Geometry::Mesh(box_mesh(corners)), None);
    let id = source.id;
    doc.entities.insert(id, source);
    let mut app = MatrixApp::new(doc).unwrap();
    app.f6_mode = om9_app::f6::catalog::Mode::Curve;
    app.controller.activate_view("Perspective").unwrap();
    app.controller.select_objects(&[id]).unwrap();
    let (_, camera, _) = app
        .controller
        .view_cameras()
        .into_iter()
        .find(|v| v.0 == "Perspective")
        .unwrap();
    let viewport = Viewport {
        origin: [0.; 2],
        size: [700., 500.],
    };
    let project = |p| camera.project(p, viewport).unwrap().unwrap().screen;
    let projected = corners.map(project);
    let ctx = egui::Context::default();
    click_f6_row(&mut app, &ctx, "Mesh Outline");
    assert!(!app.f6_open);
    assert_eq!(app.controller.document().entities.len(), 2);
    let curve = app
        .controller
        .document()
        .entities
        .values()
        .find(|e| e.id != id)
        .unwrap();
    let Geometry::Curve(Curve::Nurbs(c)) = &curve.geometry else {
        panic!("outline");
    };
    assert_eq!(c.poles.len(), 7);
    let path = c.poles.iter().copied().map(project).collect::<Vec<_>>();
    for edge in path.windows(2) {
        let a = edge[0];
        let b = edge[1];
        assert!(
            projected
                .iter()
                .any(|p| (a[0] - p[0]).hypot(a[1] - p[1]) < 1e-5)
        );
        // An exterior edge of the convex box has every projected corner to the same side.
        let cross =
            projected.map(|p| (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0]));
        assert!(cross.iter().all(|v| *v < 1e-4) || cross.iter().all(|v| *v > -1e-4));
    }
    assert_eq!(app.controller.coordinator().undo_len(), 1);
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document().entities.len(), 1);
}

#[test]
fn click_factory_reduce_mesh_preview_and_ok_keep_the_object_id() {
    use om9_core::{
        budget::Limits, geometry::Geometry, model::Entity, surface_kernel,
        utilities::mesh_from_surface,
    };
    let limits = Limits::default();
    let plane =
        surface_kernel::plane([0.; 3], [1., 0., 0.], [0., 1., 0.], 36., 36., &limits).unwrap();
    let mesh = mesh_from_surface(&plane, 4, 4, &limits).unwrap();
    let mut doc = Document::with_template_tolerance("Reduce Mesh controls");
    let entity = Entity::new("selected mesh", Geometry::Mesh(mesh.clone()), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut app = MatrixApp::new(doc).unwrap();
    let ctx = egui::Context::default();
    app.controller.select_objects(&[id]).unwrap();
    click_f6_row(&mut app, &ctx, "Reduce Mesh");
    assert!(!app.f6_open);
    frame(&mut app, &ctx, vec![]);
    click_text(&mut app, &ctx, "Preview");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while app.controller.reduce_mesh_worker_pending() {
        assert!(std::time::Instant::now() < deadline);
        frame(&mut app, &ctx, vec![]);
    }
    assert!(!app.controller.addon_session().unwrap().lines.is_empty());
    assert_eq!(app.controller.coordinator().undo_len(), 0);
    if let Some(dir) = std::env::var_os("OM9_F6_SHOT_DIR") {
        om9_app::ui_replay::screenshot(
            &mut app,
            [1280., 800.],
            1.,
            40,
            &std::path::Path::new(&dir).join("reduce-mesh-preview.png"),
        )
        .unwrap();
    }
    click_text(&mut app, &ctx, "OK");
    let Geometry::Mesh(out) = &app.controller.document().entities[&id].geometry else {
        panic!("mesh")
    };
    assert!(out.faces.len() <= 16);
    assert_eq!(app.controller.coordinator().undo_len(), 1);
    app.controller.undo().unwrap();
    assert_eq!(
        app.controller.document().entities[&id].geometry,
        Geometry::Mesh(mesh)
    );
}

#[test]
fn f6_report_groups_selected_objects_metals_and_stones() {
    use om9_core::{
        exchange::RenderMaterial,
        gem::{GemRecord, GemSpec, build_mesh},
        geometry::Geometry,
        identity::MaterialId,
        model::{Entity, GEM_EXTENSION_SLOT},
    };
    let mut doc = Document::with_template_tolerance("F6 Report");
    let metal = MaterialId::new();
    doc.exchange.materials.insert(
        metal,
        RenderMaterial {
            id: metal,
            name: "Yellow Gold".into(),
            diffuse: [128; 4],
        },
    );
    let spec = GemSpec::round(2., 1.3).unwrap();
    let mesh = build_mesh(&spec).unwrap().mesh;
    let mut ids = Vec::new();
    for _ in 0..2 {
        let mut e = Entity::new("renamed stone", Geometry::Mesh(mesh.clone()), None);
        let mut record = GemRecord::new(spec.clone());
        record.carat = Some(0.032);
        record.material = "Diamond".into();
        e.extensions
            .insert(GEM_EXTENSION_SLOT.into(), record.as_extension().unwrap());
        ids.push(e.id);
        doc.entities.insert(e.id, e);
    }
    let mut e = Entity::new("renamed metal", Geometry::Mesh(mesh), None);
    e.attributes.material = Some(metal);
    e.attributes.material_source = 1;
    ids.push(e.id);
    doc.entities.insert(e.id, e);
    let plane = om9_core::surface_kernel::plane(
        [0.; 3],
        [1., 0., 0.],
        [0., 1., 0.],
        2.,
        3.,
        &Default::default(),
    )
    .unwrap();
    let mut layer = om9_core::model::Layer::new("mount", [180, 150, 120, 255]);
    layer.material = Some(metal);
    let surface = Entity::new("support", Geometry::Surface(plane), Some(layer.id));
    doc.layer_order.push(layer.id);
    doc.layers.insert(layer.id, layer);
    let surface_id = surface.id;
    doc.entities.insert(surface_id, surface);
    let mut app = MatrixApp::new(doc).unwrap();
    app.f6_mode = om9_app::f6::catalog::Mode::Report;
    app.controller.select_objects(&ids).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![key(Key::F6, Modifiers::NONE)]);
    let labels = |app: &mut MatrixApp| {
        frame(app, &ctx, vec![]);
        let output = frame(app, &ctx, vec![]);
        output
            .shapes
            .into_iter()
            .filter_map(|s| match s.shape {
                egui::Shape::Text(t) => Some(t.galley.text().to_owned()),
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    let all = labels(&mut app);
    for label in [
        "2 Gems",
        "1 Meshes",
        "Metal Info",
        "Yellow Gold",
        "Gem Info",
        "2 Gems, 1 Meshes",
    ] {
        assert!(all.iter().any(|t| t == label), "{label} absent");
    }
    assert!(
        all.iter()
            .any(|t| t.starts_with("2 × ") && t.contains("Diamond") && t.contains("0.032 ct"))
    );
    app.controller
        .bind_metal_material(metal, Some("gold_18k_yellow"))
        .unwrap();
    let quality = app
        .controller
        .metal_registry()
        .get("gold_18k_yellow")
        .unwrap()
        .name
        .clone();
    assert!(labels(&mut app).contains(&quality));
    if let Some(dir) = std::env::var_os("OM9_F6_SHOT_DIR") {
        om9_app::ui_replay::screenshot(
            &mut app,
            [1280., 800.],
            1.,
            40,
            &std::path::Path::new(&dir).join("report.png"),
        )
        .unwrap();
    }
    app.controller.select_objects(&ids[2..]).unwrap();
    let one = labels(&mut app);
    assert!(one.iter().any(|t| t == "1 Meshes"));
    assert!(!one.iter().any(|t| t.contains("0.032 ct") || t == "2 Gems"));
    app.controller.select_objects(&[surface_id]).unwrap();
    assert!(labels(&mut app).contains(&quality));
    app.controller
        .top11_invoke(om9_core::top11::Top11Action::ControlPoints)
        .unwrap();
    let handles = app.controller.top11_grip_handles().unwrap();
    app.controller
        .select_grips(&handles[..1], om9_app::session::GripPick::Replace);
    assert!(labels(&mut app).iter().any(|t| t == "1 Surface Points"));
    app.f6_mode = om9_app::f6::catalog::Mode::Curve;
    let rows = app.f6_context().actions;
    assert!(rows.iter().any(|a| a.label == "Set Point"));
    assert!(!rows.iter().any(|a| a.label == "Set Point Z"));
    assert_eq!(app.controller.coordinator().undo_len(), 0);
}

#[test]
fn f6_materials_applies_to_selected_objects_and_undo_restores_their_sources() {
    use om9_core::{
        exchange::RenderMaterial, geometry::Geometry, identity::MaterialId, model::Entity,
    };
    let mut doc = Document::with_template_tolerance("F6 material assignment");
    let material = MaterialId::new();
    doc.exchange.materials.insert(
        material,
        RenderMaterial {
            id: material,
            name: "Polished platinum".into(),
            diffuse: [180; 4],
        },
    );
    let ids = [0., 1., 2.].map(|x| {
        let entity = Entity::new("sample", Geometry::Point([x, 0., 0.]), None);
        let id = entity.id;
        doc.entities.insert(id, entity);
        id
    });
    let mut app = MatrixApp::new(doc).unwrap();
    app.f6_mode = om9_app::f6::catalog::Mode::Materials;
    app.controller.select_objects(&ids[..2]).unwrap();
    let before = app.controller.document().clone();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![key(Key::F6, Modifiers::NONE)]);
    frame(&mut app, &ctx, vec![]);
    click_text(&mut app, &ctx, "Polished platinum");
    click_text(&mut app, &ctx, "Apply material");
    for id in &ids[..2] {
        let attrs = &app.controller.document().entities[id].attributes;
        assert_eq!(attrs.material, Some(material));
        assert_eq!(attrs.material_source, 1);
    }
    assert_eq!(
        app.controller.document().entities[&ids[2]],
        before.entities[&ids[2]]
    );
    assert_eq!(app.controller.coordinator().undo_len(), 1);
    let limits = app.controller.coordinator().limits();
    let bytes = om9_archive::Archive::encode(app.controller.document(), limits).unwrap();
    let reopened = om9_archive::Archive::decode(&bytes, limits).unwrap();
    assert_eq!(reopened.entities, app.controller.document().entities);
    assert_eq!(
        reopened.exchange.materials,
        app.controller.document().exchange.materials
    );
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &before);
}

#[test]
fn f6_places_actions_at_pointer_and_keeps_the_pinned_position() {
    use om9_app::{dock_layout::SectionId, tool_panel::ToolPanel};
    let mut app = MatrixApp::new(Document::new("F6 anchor")).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    let open = |app: &mut MatrixApp, x, y| {
        frame(app, &ctx, vec![Event::PointerMoved(egui::pos2(x, y))]);
        frame(app, &ctx, vec![key(Key::F6, Modifiers::NONE)]);
        frame(app, &ctx, vec![]);
        app.dock
            .section(SectionId::Tool(ToolPanel::F6))
            .float
            .unwrap()
    };
    let path = std::env::temp_dir().join(format!("om9-f6-pin-{}.json", uuid::Uuid::new_v4()));
    app.set_dock_layout_path(Some(path.clone()));
    let first = open(&mut app, 620., 340.);
    assert!(
        (first.x - 610.).abs() < 2. && (first.y - 240.).abs() < 2.,
        "{first:?}"
    );
    click_text(&mut app, &ctx, "Pin");
    assert!(app.f6_pinned && app.dock.f6_pinned);
    let pinned = open(&mut app, 840., 350.);
    assert_eq!(first, pinned);
    // Other dock headers have close buttons too. Pick the one in the F6 header.
    let out = frame(&mut app, &ctx, vec![]);
    let close = out
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::Shape::Text(t) if t.galley.text() == "×" => {
                Some(Rect::from_min_size(t.pos, t.galley.size()).center())
            }
            _ => None,
        })
        .find(|p| p.x >= pinned.x && p.x <= pinned.x + pinned.width && (p.y - pinned.y).abs() < 32.)
        .expect("F6 close");
    frame(&mut app, &ctx, vec![Event::PointerMoved(close)]);
    for pressed in [true, false] {
        frame(
            &mut app,
            &ctx,
            vec![Event::PointerButton {
                pos: close,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: Modifiers::NONE,
            }],
        );
    }
    assert!(!app.f6_open && !app.f6_pinned && !app.dock.f6_pinned);
    assert!(path.is_file());
    let mut reopened = MatrixApp::new(Document::new("reopen closed F6")).unwrap();
    reopened.set_dock_layout_path(Some(path.clone()));
    assert!(reopened.load_dock_layout().is_empty());
    assert!(!reopened.f6_open && !reopened.f6_pinned);
    std::fs::remove_file(path).unwrap();
    let edge = open(&mut app, 1275., 795.);
    assert!(
        edge.x + edge.width <= 1281. && edge.y + edge.height <= 801.,
        "{edge:?}"
    );
}

#[test]
fn f6_eight_mode_buttons_follow_the_compiled_view_order() {
    use om9_app::{dock_layout::SectionId, f6::catalog::Mode, tool_panel::ToolPanel};
    let mut app = MatrixApp::new(Document::new("F6 mode strip")).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    frame(
        &mut app,
        &ctx,
        vec![
            Event::PointerMoved(egui::pos2(640., 320.)),
            key(Key::F6, Modifiers::NONE),
        ],
    );
    let output = frame(&mut app, &ctx, vec![]);
    let modes = output
        .shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            egui::Shape::Text(t) if t.galley.text() == "Modes" => {
                Some(Rect::from_min_size(t.pos, t.galley.size()))
            }
            _ => None,
        })
        .expect("mode header");
    let panel = app
        .dock
        .section(SectionId::Tool(ToolPanel::F6))
        .float
        .unwrap();
    let left = modes.left();
    let width = 2. * (panel.x + panel.width / 2. - left);
    for (index, mode) in [
        Mode::General,
        Mode::Curve,
        Mode::Gem,
        Mode::Surface,
        Mode::TSplines,
        Mode::User,
        Mode::Report,
        Mode::Materials,
    ]
    .into_iter()
    .enumerate()
    {
        let pos = egui::pos2(
            left + (index as f32 + 0.5) * width / 8.,
            modes.bottom() + 16.,
        );
        frame(&mut app, &ctx, vec![Event::PointerMoved(pos)]);
        for pressed in [true, false] {
            frame(
                &mut app,
                &ctx,
                vec![Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Modifiers::NONE,
                }],
            );
        }
        assert_eq!(app.f6_mode, mode, "button {index} at {pos:?}");
        assert!(app.f6_open);
    }
}

#[test]
fn click_factory_mesh_union_and_enter_publishes_the_measured_mesh() {
    use om9_core::{cage::box_mesh, geometry::Geometry, model::Entity};
    let mut doc = Document::with_template_tolerance("mesh menu click");
    let ids = [0., 1.].map(|lo| {
        let corners = std::array::from_fn(|i| {
            std::array::from_fn(|axis| if i & (1 << axis) != 0 { lo + 2. } else { lo })
        });
        let e = Entity::new("box mesh", Geometry::Mesh(box_mesh(corners)), None);
        let id = e.id;
        doc.entities.insert(id, e);
        id
    });
    let mut app = MatrixApp::new(doc).unwrap();
    let original = app.controller.document().clone();
    app.controller.select_objects(&ids).unwrap();
    app.f6_mode = om9_app::f6::catalog::Mode::Surface;
    let ctx = egui::Context::default();
    click_f6_row(&mut app, &ctx, "Mesh Union");
    assert!(!app.f6_open);
    assert_eq!(
        app.controller.addon_session().unwrap().code,
        "MeshBooleanUnion"
    );
    frame(&mut app, &ctx, vec![key(Key::Enter, Modifiers::NONE)]);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while app.controller.boolean_builder_worker_pending() {
        assert!(std::time::Instant::now() < deadline);
        frame(&mut app, &ctx, vec![]);
    }
    assert_eq!(app.controller.document().entities.len(), 1);
    let Geometry::Mesh(mesh) = &app
        .controller
        .document()
        .entities
        .values()
        .next()
        .unwrap()
        .geometry
    else {
        panic!("expected union mesh")
    };
    assert!((om9_core::cutters::volume_mm3(mesh).unwrap() - 15.).abs() < 1e-6);
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &original);
}

#[test]
fn click_factory_polyline_then_type_points_and_undo() {
    let mut app = MatrixApp::new(Document::with_template_tolerance("F6 click")).unwrap();
    let ctx = egui::Context::default();
    click_f6_row(&mut app, &ctx, "Polyline");
    assert!(!app.f6_open && app.controller.is_running());
    for input in ["0,0,0", "9,3,0", ""] {
        text(&mut app, &ctx, input);
    }
    assert_eq!(
        app.controller.document().entities.len(),
        1,
        "{}",
        app.transcript.text()
    );
    app.controller.undo().unwrap();
    assert!(app.controller.document().entities.is_empty());
}

#[test]
fn f6_remains_open_between_frames_and_unpinned_escape_restores_command_focus() {
    let mut app = MatrixApp::new(Document::new("F6")).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![key(Key::F6, Modifiers::NONE)]);
    assert!(app.f6_open);
    frame(&mut app, &ctx, vec![]);
    assert!(app.f6_open);
    frame(&mut app, &ctx, vec![key(Key::Escape, Modifiers::NONE)]);
    assert!(!app.f6_open);
    text(&mut app, &ctx, "Point");
    text(&mut app, &ctx, "3,4,5");
    assert_eq!(app.controller.document().entities.len(), 1);
}

#[test]
fn printable_viewport_input_routes_to_cmd_when_no_other_editor_owns_focus() {
    let mut app = MatrixApp::new(Document::new("focus")).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    let cmd = egui::Id::new("matrix-command-transcript");
    let other = egui::Id::new("other-editor");
    let mut value = String::new();
    ctx.run_ui(egui::RawInput::default(), |ui| {
        let field = ui.add(egui::TextEdit::singleline(&mut value).id(other));
        field.request_focus();
        app.show(ui);
    }).textures_delta.clear();
    ctx.run_ui(
        egui::RawInput {
            events: vec![Event::Text("other field".into())],
            ..Default::default()
        },
        |ui| {
            ui.add(egui::TextEdit::singleline(&mut value).id(other));
            app.show(ui);
        },
    ).textures_delta.clear();
    assert_eq!(value, "other field");
    assert!(app.transcript.draft().is_empty());
    ctx.memory_mut(|m| m.surrender_focus(other));
    assert!(ctx.memory(|m| m.focused().is_none()));
    for piece in ["P", "o", "i", "n", "t"] {
        frame(&mut app, &ctx, vec![Event::Text(piece.into())]);
        assert!(ctx.memory(|m| m.has_focus(cmd)));
    }
    assert_eq!(app.transcript.draft(), "Point");
    assert!(!app.controller.is_running());
    frame(&mut app, &ctx, vec![key(Key::ArrowDown, Modifiers::NONE)]);
    frame(&mut app, &ctx, vec![key(Key::Enter, Modifiers::NONE)]);
    assert!(app.controller.is_running());
    assert!(app.controller.prompt().starts_with("Location of point"));
    assert!(app.controller.document().entities.is_empty());
}

#[test]
fn reference_workspace_places_sidebar_left_and_through_finger_below_looking_down() {
    use om9_scene::workspace::ViewId;
    let mut app = MatrixApp::new(Document::new("reference layout")).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    let looking = app.view_rects[&ViewId::LookingDown];
    let perspective = app.view_rects[&ViewId::Perspective];
    let finger = app.view_rects[&ViewId::ThroughFinger];
    let side = app.view_rects[&ViewId::SideView];
    let plan = app.region_plan.as_ref().unwrap();
    let rail = plan.left_rail.unwrap();
    let history = app.command_history_rect().unwrap();
    let input = app.command_input_rect().unwrap();
    let same_edge = |a: f32, b: f32| (a - b).abs() <= 1.;
    assert!(same_edge(looking.left(), rail.right()));
    assert!(same_edge(plan.menu_bar.bottom(), plan.command_input.y));
    assert!(same_edge(plan.command_input.bottom(), plan.toolbar.y));
    assert!(same_edge(looking.top(), plan.toolbar.bottom()));
    assert!(same_edge(perspective.top(), looking.top()));
    assert!(history.top() >= plan.command_input.y);
    assert!(history.bottom() <= input.top());
    assert!(input.bottom() <= plan.command_input.bottom() + 1.);
    let font = egui::TextStyle::Monospace.resolve(&ctx.style_of(egui::Theme::Light));
    let row_height = ctx.fonts_mut(|fonts| fonts.row_height(&font));
    assert!(history.height() >= 2. * row_height);
    assert!(same_edge(input.height(), row_height + 2.));
    assert_eq!(looking.left(), finger.left());
    assert_eq!(perspective.left(), side.left());
    assert!(finger.top() > looking.top());
    assert!(side.top() > perspective.top());
    assert!(app.controller.document().entities.is_empty());
    assert_eq!(app.controller.coordinator().undo_len(), 0);
}

#[test]
fn construction_grid_renders_finite_axes_without_document_geometry() {
    use om9_scene::workspace::ViewId;
    let mut app = MatrixApp::new(Document::new("grid reference")).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    let Some(images) = gpu_frames::render(app.controller.document()) else { return };
    let red = |p: [u8; 4]| p[0] > 30 && u16::from(p[0]) > 2 * u16::from(p[1]) && u16::from(p[0]) > 2 * u16::from(p[2]);
    let cyan = |p: [u8; 4]| p[1] > 40 && p[2] > 40 && 2 * u16::from(p[0]) < u16::from(p[1].min(p[2]));
    for (point, predicate) in [
        ([-5., 0., 0.], red as fn([u8; 4]) -> bool),
        ([5., 0., 0.], red as fn([u8; 4]) -> bool),
        ([0., -5., 0.], cyan as fn([u8; 4]) -> bool),
        ([0., 5., 0.], cyan as fn([u8; 4]) -> bool),
    ] {
        let (x, y) = gpu_frames::looking_down_pixel(point);
        assert!((-1..=1).any(|dy| (-1..=1).any(|dx| {
            predicate(images[0].get_pixel((x as i32 + dx) as u32, (y as i32 + dy) as u32).0)
        })), "GPU axis colour at {point:?}");
    }
    let grid = om9_scene::grid::Grid::new(app.controller.workspace.views[&ViewId::LookingDown].cplane).unwrap();
    assert!(grid.lines.iter().flat_map(|line| [line.start, line.end]).flatten().all(|v| v.abs() <= 20.));
    assert!(app.controller.document().entities.is_empty());
    assert_eq!(app.controller.coordinator().undo_len(), 0);
}

fn top_pixel(app: &MatrixApp, world: [f64; 3]) -> egui::Pos2 {
    use om9_scene::workspace::ViewId;
    view_pixel(app, ViewId::LookingDown,world)
}
fn view_pixel(app: &MatrixApp, view: om9_scene::workspace::ViewId, world: [f64;3]) -> egui::Pos2 {
    use om9_scene::camera::Viewport;
    let rect = app.view_rects[&view];
    let viewport = Viewport {
        origin: [rect.left() as f64, rect.top() as f64],
        size: [rect.width() as f64, rect.height() as f64],
    };
    let location = app.controller.workspace.views[&view]
        .camera
        .project(world, viewport)
        .unwrap()
        .unwrap()
        .screen;
    egui::pos2(location[0] as f32, location[1] as f32)
}

fn click_top_component(app: &mut MatrixApp, ctx: &egui::Context, world: [f64; 3]) {
    frame(app, ctx, vec![]);
    frame(app, ctx, vec![]);
    let point = top_pixel(app, world);
    let modifiers = Modifiers {
        ctrl: true,
        shift: true,
        command: true,
        ..Modifiers::NONE
    };
    frame(
        app,
        ctx,
        vec![
            Event::ModifiersChanged(modifiers),
            Event::PointerMoved(point),
        ],
    );
    for pressed in [true, false] {
        frame(
            app,
            ctx,
            vec![
                Event::ModifiersChanged(modifiers),
                Event::PointerButton {
                    pos: point,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers,
                },
            ],
        );
    }
    frame(app, ctx, vec![Event::ModifiersChanged(Modifiers::NONE)]);
}

fn scroll_click_f6(app: &mut MatrixApp, ctx: &egui::Context, label: &str) {
    frame(app, ctx, vec![key(Key::F6, Modifiers::NONE)]);
    frame(app, ctx, vec![]);
    let mut visible = false;
    for _ in 0..20 {
        let out = frame(app, ctx, vec![]);
        let mut list = None;
        for shape in &out.shapes {
            if let egui::Shape::Text(t) = &shape.shape {
                let center = Rect::from_min_size(t.pos, t.galley.size()).center();
                if shape.clip_rect.contains(center) {
                    if t.galley.text() == label {
                        visible = true;
                        break;
                    }
                    if app
                        .f6_context()
                        .actions
                        .iter()
                        .any(|a| a.label == t.galley.text())
                    {
                        list = Some(center);
                    }
                }
            }
        }
        if visible {
            break;
        }
        let at = list.expect("visible factory actions");
        frame(
            app,
            ctx,
            vec![
                Event::PointerMoved(at),
                Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    phase: egui::TouchPhase::Move,
                    delta: Vec2::new(0., -100.),
                    modifiers: Modifiers::NONE,
                },
            ],
        );
    }
    assert!(visible);
    for _ in 0..30 {
        frame(app, ctx, vec![]);
    }
    click_text(app, ctx, label);
}

#[test]
fn click_factory_tspline_face_then_delete_keeps_the_other_quads() {
    use om9_core::geometry::Geometry;
    let mut app =
        MatrixApp::new(Document::with_template_tolerance("Delete box-cage face")).unwrap();
    app.controller.submit("tsPlane").unwrap();
    app.controller.submit("Center").unwrap();
    app.controller.session_point([0.; 3]).unwrap();
    app.controller.session_point([10., 10., 0.]).unwrap();
    let id = *app.controller.selected().first().unwrap();
    app.controller.select_objects(&[]).unwrap();
    app.f6_mode = om9_app::f6::catalog::Mode::TSplines;
    let ctx = egui::Context::default();
    click_top_component(&mut app, &ctx, [-4., -6., 0.]);
    assert!(app.controller.tspline_faces_selected());
    scroll_click_f6(&mut app,&ctx,"Delete Face(s)");
    assert!(!app.f6_open);
    let Geometry::Mesh(mesh) = &app.controller.document().entities[&id].geometry else {
        panic!("cage");
    };
    assert_eq!(mesh.faces.len(), 3);
    app.controller.undo().unwrap();
    app.controller
        .select_subobject(id, om9_scene::scene::Subelement::MeshFace(0))
        .unwrap();
    let mut raw = egui::RawInput {
        events: vec![key(Key::Delete, Modifiers::NONE)],
        ..Default::default()
    };
    eframe::App::raw_input_hook(&mut app, &ctx, &mut raw);
    frame(&mut app, &ctx, raw.events);
    let Geometry::Mesh(mesh) = &app.controller.document().entities[&id].geometry else {
        panic!("cage");
    };
    assert_eq!(mesh.faces.len(), 3);
}

#[test]
fn click_factory_ring_select_highlights_parallel_edges() {
    use om9_scene::scene::{PickKind, Subelement};
    let mut app = MatrixApp::new(Document::with_template_tolerance("edge ring by mouse")).unwrap();
    app.controller.submit("tsPlane").unwrap();
    app.controller.submit("Center").unwrap();
    app.controller.session_point([0.; 3]).unwrap();
    app.controller.session_point([10., 10., 0.]).unwrap();
    let id = *app.controller.selected().first().unwrap();
    let ctx = egui::Context::default();
    app.f6_mode = om9_app::f6::catalog::Mode::Report;
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![key(Key::F6, Modifiers::NONE)]);
    let report = frame(&mut app, &ctx, vec![]);
    assert!(report.shapes.iter().any(|s| matches!(&s.shape, egui::Shape::Text(t) if t.galley.text() == "1 T-Spline Meshes")));
    frame(&mut app, &ctx, vec![key(Key::Escape, Modifiers::NONE)]);
    app.controller.select_objects(&[]).unwrap();
    app.set_sub_object_kind(PickKind::Edge);
    app.f6_mode = om9_app::f6::catalog::Mode::General;
    click_top_component(&mut app, &ctx, [0., -5., 0.]);
    assert_eq!(app.controller.subselection().len(), 1);
    click_f6_row(&mut app, &ctx, "Ring Select");
    let edges = app
        .controller
        .subselection()
        .iter()
        .map(|key| {
            assert_eq!(key.root, id);
            let Subelement::MeshEdge { a, b } = key.subset else {
                panic!("edge");
            };
            (a, b)
        })
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        edges,
        std::collections::BTreeSet::from([(0, 1), (3, 4), (6, 7)])
    );
    assert_eq!(app.controller.coordinator().undo_len(), 1);
}

#[test]
fn double_click_and_f6_select_the_same_continuous_cage_edge_loop() {
    use om9_scene::scene::{PickKind, Subelement};
    let mut app =
        MatrixApp::new(Document::with_template_tolerance("T-Spline loop by mouse")).unwrap();
    app.controller.submit("tsPlane").unwrap();
    for option in ["Center", "XFaces=4", "YFaces=4"] {
        app.controller.submit(option).unwrap();
    }
    app.controller.session_point([0.; 3]).unwrap();
    app.controller.session_point([10., 10., 0.]).unwrap();
    app.controller.select_objects(&[]).unwrap();
    app.set_sub_object_kind(PickKind::Edge);
    app.f6_mode = om9_app::f6::catalog::Mode::General;
    let ctx = egui::Context::default();
    let point = [5., -2.5, 0.];
    click_top_component(&mut app, &ctx, point);
    click_f6_row(&mut app, &ctx, "Loop Select");
    let edges = |app: &MatrixApp| {
        app.controller
            .subselection()
            .iter()
            .map(|key| {
                let Subelement::MeshEdge { a, b } = key.subset else {
                    panic!("edge")
                };
                (a, b)
            })
            .collect::<std::collections::BTreeSet<_>>()
    };
    let expected = (15..19).map(|a| (a, a + 1)).collect();
    assert_eq!(edges(&app), expected);
    app.controller.clear_subselection();
    click_top_component(&mut app, &ctx, point);
    let pos = ctx.pointer_latest_pos().unwrap();
    let time = ctx.input(|i| i.time) + 1.0;
    for (i, pressed) in [true, false, true, false].into_iter().enumerate() {
        let raw = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(
                egui::Pos2::ZERO,
                Vec2::new(1280., 800.),
            )),
            time: Some(time + 0.03 * i as f64),
            events: vec![Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: Modifiers::NONE,
            }],
            ..Default::default()
        };
        let _ = ctx.run_ui(raw, |ui| app.show(ui));
    }
    assert_eq!(edges(&app), expected);
    assert_eq!(app.controller.coordinator().undo_len(), 1);
}

#[test]
fn click_factory_paint_selection_drags_across_faces_and_escape_keeps_the_picks() {
    use om9_scene::scene::{PickKind, Subelement};
    let mut app =
        MatrixApp::new(Document::with_template_tolerance("Paint T-Spline faces")).unwrap();
    app.controller.submit("tsPlane").unwrap();
    for option in ["Center", "XFaces=4", "YFaces=4"] {
        app.controller.submit(option).unwrap();
    }
    app.controller.session_point([0.; 3]).unwrap();
    app.controller.session_point([10., 10., 0.]).unwrap();
    app.controller.select_objects(&[]).unwrap();
    app.f6_mode = om9_app::f6::catalog::Mode::TSplines;
    let ctx = egui::Context::default();
    click_top_component(&mut app, &ctx, [-7.5, -7.5, 0.]);
    scroll_click_f6(&mut app, &ctx, "Paint Selection");
    assert_eq!(app.controller.cage_paint_mode(), Some(PickKind::Face));
    let a = top_pixel(&app, [-7.5, -2.5, 0.]);
    let b = top_pixel(&app, [7.5, -2.5, 0.]);
    let button = |pos, pressed| Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    };
    frame(&mut app, &ctx, vec![Event::PointerMoved(a)]);
    frame(&mut app, &ctx, vec![button(a, true)]);
    frame(&mut app, &ctx, vec![Event::PointerMoved(b)]);
    frame(
        &mut app,
        &ctx,
        vec![Event::PointerMoved(a), button(a, false)],
    );
    let ids = |app: &MatrixApp| {
        app.controller
            .subselection()
            .iter()
            .map(|r| {
                let Subelement::MeshFace(f) = r.subset else {
                    panic!("face")
                };
                f
            })
            .collect::<std::collections::BTreeSet<_>>()
    };
    let expected = std::collections::BTreeSet::from([0, 1, 5, 9, 13]);
    assert_eq!(ids(&app), expected);
    frame(&mut app, &ctx, vec![key(Key::Escape, Modifiers::NONE)]);
    assert!(app.controller.cage_paint_mode().is_none());
    assert_eq!(ids(&app), expected);
    for enabled in [true, false] {
        app.controller
            .submit("-_tsSelection _TogglePaintMode")
            .unwrap();
        assert_eq!(app.controller.cage_paint_mode().is_some(), enabled);
    }
    assert_eq!(app.controller.coordinator().undo_len(), 1);
}

#[test]
fn click_factory_object_manipulators_switch_visible_handles() {
    use om9_scene::{gumball_view::Mode, workspace::ViewId};
    let mut app =
        MatrixApp::new(Document::with_template_tolerance("F6 manipulator buttons")).unwrap();
    app.controller.submit("tsPlane").unwrap();
    app.controller.session_point([-4., -3., 0.]).unwrap();
    app.controller.session_point([4., 3., 0.]).unwrap();
    app.f6_mode = om9_app::f6::catalog::Mode::TSplines;
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    for (label, mode) in [
        ("Rotate", Mode::Rotate),
        ("Scale", Mode::Scale),
        ("Translate", Mode::Move),
    ] {
        click_f6_row(&mut app, &ctx, label);
        assert_eq!(app.gumball_mode(), mode);
        assert!(!app.f6_open);
        assert_eq!(
            app.gumball_scale_boxes(ViewId::LookingDown).is_empty(),
            mode != Mode::Scale
        );
    }
    assert_eq!(app.controller.coordinator().undo_len(), 1);
}

#[test]
fn click_factory_face_translate_drag_escape_then_switch_to_edge_picking() {
    use om9_app::f6::catalog::Mode;
    use om9_core::geometry::Geometry;
    use om9_scene::{
        scene::{PickKind, Subelement},
        workspace::ViewId,
    };
    let mut app = MatrixApp::new(Document::with_template_tolerance(
        "T-Spline component viewport",
    ))
    .unwrap();
    for command in ["tsPlane", "Center", "XFaces=4", "YFaces=4"] {
        app.controller.submit(command).unwrap();
    }
    app.controller.session_point([0.; 3]).unwrap();
    app.controller.session_point([10., 10., 0.]).unwrap();
    let id = *app.controller.selected().first().unwrap();
    let original = app.controller.document().clone();
    let ctx = egui::Context::default();
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.select_objects(&[]).unwrap();
    app.set_sub_object_kind(PickKind::Face);
    app.f6_mode = Mode::TSplines;
    click_top_component(&mut app, &ctx, [-2.5, -2.5, 0.]);
    click_f6_row(&mut app, &ctx, "Translate");
    assert_eq!(
        app.controller.subselection()[0].subset,
        Subelement::MeshFace(5)
    );
    let pivot = app.controller.gumball_pivot().unwrap().origin;
    assert_eq!(pivot, [-2.5, -2.5, 0.]);
    let p = top_pixel(&app, pivot);
    let start = p + Vec2::new(72., 0.);
    let end = p + Vec2::new(100., 0.);
    let button = |pos, pressed| Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    };
    let begin = |app: &mut MatrixApp| {
        frame(app, &ctx, vec![Event::PointerMoved(start)]);
        frame(app, &ctx, vec![button(start, true)]);
        frame(app, &ctx, vec![Event::PointerMoved(end)]);
    };
    begin(&mut app);
    assert!(!app.controller.cage_preview_edges().is_empty());
    assert_eq!(app.controller.document(), &original);
    frame(&mut app, &ctx, vec![key(Key::Escape, Modifiers::NONE)]);
    frame(&mut app, &ctx, vec![button(end, false)]);
    assert!(app.controller.cage_preview_edges().is_empty());
    assert_eq!(app.controller.coordinator().undo_len(), 1);
    assert_eq!(app.controller.subselection().len(), 1);
    begin(&mut app);
    frame(&mut app, &ctx, vec![button(end, false)]);
    let mesh = |doc: &Document| match &doc.entities[&id].geometry {
        Geometry::Mesh(m) => m.clone(),
        _ => panic!("mesh"),
    };
    let before = mesh(&original);
    let after = mesh(app.controller.document());
    for (i, (a, b)) in before.vertices.iter().zip(&after.vertices).enumerate() {
        assert_eq!(a != b, before.faces[5].contains(&(i as u32)));
    }
    assert_eq!(app.controller.coordinator().undo_len(), 2);
    scroll_click_f6(&mut app, &ctx, "Edge");
    assert!(app.controller.subselection().is_empty());
    assert!(app.controller.gumball_pivot().is_none());
    let pos = top_pixel(&app, [7.5, 5., 0.]);
    frame(&mut app, &ctx, vec![Event::PointerMoved(pos)]);
    for pressed in [true, false] {
        frame(&mut app, &ctx, vec![button(pos, pressed)]);
    }
    assert!(
        matches!(app.controller.subselection(),[pick] if matches!(pick.subset,Subelement::MeshEdge{..}))
    );
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &original);
}

#[test]
fn click_factory_set_pivot_and_double_click_a_cage_handle_accept_a_numeric_edit() {
    use om9_app::f6::catalog::Mode;
    use om9_core::geometry::Geometry;
    use om9_scene::workspace::ViewId;
    let mut app = MatrixApp::new(Document::with_template_tolerance(
        "F6 custom pivot and numeric axis",
    ))
    .unwrap();
    for command in ["tsPlane", "Center", "XFaces=4", "YFaces=4"] {
        app.controller.submit(command).unwrap();
    }
    app.controller.session_point([0.; 3]).unwrap();
    app.controller.session_point([10., 10., 0.]).unwrap();
    let id = *app.controller.selected().first().unwrap();
    app.f6_mode = Mode::TSplines;
    app.controller.workspace.active = ViewId::LookingDown;
    let ctx = egui::Context::default();
    click_top_component(&mut app, &ctx, [-2.5, -2.5, 0.]);
    click_f6_row(&mut app, &ctx, "Translate");
    scroll_click_f6(&mut app, &ctx, "Set Pivot");
    assert!(app.controller.session_wants_point());
    let target = [0., 0., 0.];
    let corner = top_pixel(&app, target);
    let press = |pos, pressed| Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    };
    frame(&mut app, &ctx, vec![Event::PointerMoved(corner)]);
    for down in [true, false] {
        frame(&mut app, &ctx, vec![press(corner, down)]);
    }
    let pivot = app.controller.gumball_pivot().unwrap().origin;
    assert!(pivot.iter().all(|v| v.abs() < 0.1), "{pivot:?}");
    assert!(app.controller.addon_session().is_none());
    let h = top_pixel(&app, pivot) + Vec2::new(74., 0.);
    let open = |app: &mut MatrixApp, point: egui::Pos2| {
        frame(app, &ctx, vec![Event::PointerMoved(point)]);
        let t = ctx.input(|i| i.time) + 1.;
        for (i, down) in [true, false, true, false].into_iter().enumerate() {
            let _ = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(
                        egui::Pos2::ZERO,
                        Vec2::new(1280., 800.),
                    )),
                    time: Some(t + i as f64 * 0.035),
                    events: vec![press(point, down)],
                    ..Default::default()
                },
                |ui| app.show(ui),
            );
        }
    };
    open(&mut app, h);
    assert_eq!(
        app.controller.addon_session().unwrap().reply.prompt,
        "Distance"
    );
    assert_eq!(app.controller.subselection().len(), 1);
    app.controller.submit("Handle=Z").unwrap();
    app.controller.submit("Style=Rotate").unwrap();
    let original = app.controller.document().clone();
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![Event::Text("90".into())]);
    frame(&mut app, &ctx, vec![key(Key::Enter, Modifiers::NONE)]);
    assert!(app.controller.addon_session().is_none());
    assert_eq!(app.controller.coordinator().undo_len(), 2);
    let Geometry::Mesh(m) = &app.controller.document().entities[&id].geometry else {
        panic!("mesh")
    };
    let Geometry::Mesh(old) = &original.entities[&id].geometry else {
        panic!("mesh")
    };
    for (i, p) in old.vertices.iter().enumerate() {
        if old.faces[5].contains(&(i as u32)) {
            let q = m.vertices[i];
            assert!(
                (q[0] - (pivot[0] + pivot[1] - p[1])).abs() < 1e-8
                    && (q[1] - (pivot[1] + p[0] - pivot[0])).abs() < 1e-8
            );
        } else {
            assert_eq!(m.vertices[i], *p);
        }
    }
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &original);
}

#[test]
fn click_factory_extract_single_edge_and_pick_offset_preview() {
    use om9_core::{budget::Limits, curve_ops::point_at, geometry::Geometry};
    use om9_scene::{scene::PickKind, workspace::ViewId};
    let mut app =
        MatrixApp::new(Document::with_template_tolerance("F6 extract edge replay")).unwrap();
    for command in [
        "tsPlane", "Center", "XFaces=4", "YFaces=4", "0,0,0", "10,10,0",
    ] {
        app.controller.submit(command).unwrap();
    }
    let original = app.controller.document().clone();
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.select_objects(&[]).unwrap();
    app.set_sub_object_kind(PickKind::Edge);
    app.f6_mode = om9_app::f6::catalog::Mode::TSplines;
    let ctx = egui::Context::default();
    click_top_component(&mut app, &ctx, [2.5, 0., 0.]);
    scroll_click_f6(&mut app, &ctx, "Extract Edges");
    assert!(app.controller.addon_session().is_none(), "focus={:?} draft={} reply={:?}", ctx.memory(|m|m.focused()),app.transcript.draft(),app.controller.addon_session().map(|s|(&s.reply.prompt,&s.reply.state)));
    let selected = *app.controller.selected().first().unwrap();
    let Geometry::Curve(curve) = &app.controller.document().entities[&selected].geometry else {
        panic!("SelLast curve")
    };
    let limits = Limits::default();
    let mid = point_at(curve, 0.5, &limits).unwrap();
    assert!((mid[0] - 2.5).abs() < 1e-8 && mid[1].abs() < 1e-8);
    app.controller.undo().unwrap();
    click_top_component(&mut app, &ctx, [2.5, 0., 0.]);
    scroll_click_f6(&mut app, &ctx, "Extract Edges Offset");
    assert!(app.controller.session_wants_point());
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![]);
    let pos = top_pixel(&app, [5., 2.5, 0.]);
    frame(&mut app, &ctx, vec![Event::PointerMoved(pos)]);
    frame(&mut app, &ctx, vec![]);
    let preview = &app.controller.addon_session().unwrap().hover_lines;
    assert!(!preview.is_empty());
    for pressed in [true, false] {
        frame(
            &mut app,
            &ctx,
            vec![Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: Modifiers::NONE,
            }],
        );
    }
    let preview = &app.controller.addon_session().unwrap().lines;
    assert!(!preview.is_empty());
    assert_eq!(app.controller.document(), &original);
    frame(&mut app, &ctx, vec![key(Key::Enter, Modifiers::NONE)]);
    assert!(app.controller.addon_session().is_none(), "focus={:?} draft={} reply={:?}", ctx.memory(|m|m.focused()),app.transcript.draft(),app.controller.addon_session().map(|s|(&s.reply.prompt,&s.reply.state)));
    let curve = app
        .controller
        .document()
        .entities
        .values()
        .find_map(|e| match &e.geometry {
            Geometry::Curve(c) => Some(c),
            _ => None,
        })
        .unwrap();
    let p = point_at(curve, 0.5, &limits).unwrap();
    assert!(
        (p[0] - 2.5).abs() < 1e-8 && (p[1] - 2.5).abs() < 0.15,
        "{p:?}"
    );
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &original);
    app.controller.select_objects(&[]).unwrap();
    app.controller.submit("tsExtractEdges").unwrap();
    assert!(app.controller.addon_collects_objects());
    for world in [[2.5, 0., 0.], [-2.5, -5., 0.]] {
        let pos = top_pixel(&app, world);
        frame(&mut app, &ctx, vec![Event::PointerMoved(pos)]);
        for pressed in [true, false] {
            frame(
                &mut app,
                &ctx,
                vec![Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Modifiers::NONE,
                }],
            );
        }
    }
    assert_eq!(app.controller.subselection().len(), 2);
    frame(&mut app, &ctx, vec![key(Key::Enter, Modifiers::NONE)]);
    assert!(app.controller.addon_session().is_none(), "focus={:?} draft={} reply={:?}", ctx.memory(|m|m.focused()),app.transcript.draft(),app.controller.addon_session().map(|s|(&s.reply.prompt,&s.reply.state)));
    assert_eq!(app.controller.document().entities.len(), 3);
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &original);
}

#[test]
fn click_factory_split_curves_updates_tolerance_display_and_commits_all_cuts() {
    use om9_app::f6::catalog::Mode;
    use om9_core::{
        geometry::{Curve, Geometry},
        history::Tolerance,
        model::Entity,
    };
    let mut doc = Document::with_template_tolerance("Split Curves controls");
    doc.tolerance = Some(Tolerance::try_new(0.001, 0.01, 0.01).unwrap());
    let mut ids = vec![];
    for (a, b) in [
        ([-9., 0., 0.], [9., 0., 0.]),
        ([-5., -4., 0.2], [-5., 4., 0.2]),
        ([5., -4., 0.2], [5., 4., 0.2]),
    ] {
        let e = Entity::new(
            "input",
            Geometry::Curve(Curve::Line {
                start: a,
                end: b,
                domain: [0., 1.],
            }),
            doc.current_layer,
        );
        ids.push(e.id);
        doc.entities.insert(e.id, e);
    }
    let mut app = MatrixApp::new(doc.clone()).unwrap();
    app.f6_mode = Mode::TSplines;
    app.controller.select_objects(&ids).unwrap();
    let ctx = egui::Context::default();
    let control = |app: &mut MatrixApp, name: &str| {
        frame(app, &ctx, vec![]);
        let pos = app
            .session_dialog_control_rect(&format!("Dialog:{name}"))
            .expect("visible dialog control")
            .center();
        frame(app, &ctx, vec![Event::PointerMoved(pos)]);
        for pressed in [true, false] {
            frame(
                app,
                &ctx,
                vec![Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Modifiers::NONE,
                }],
            );
        }
    };
    let wait = |app: &mut MatrixApp| {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while app.controller.split_curves_pending() {
            assert!(std::time::Instant::now() < deadline);
            frame(app, &ctx, vec![]);
        }
        frame(app, &ctx, vec![]);
    };
    scroll_click_f6(&mut app, &ctx, "ts Split Curves");
    assert!(!app.f6_open);
    frame(&mut app, &ctx, vec![]);
    control(&mut app, "Display");
    wait(&mut app);
    assert!(app.controller.addon_session().unwrap().lines.is_empty());
    control(&mut app, "UseFileTolerance");
    assert!(app.controller.addon_dialog().unwrap().enabled("Tolerance"));
    control(&mut app, "Tolerance");
    let ctrl = Modifiers {
        ctrl: true,
        command: true,
        ..Modifiers::NONE
    };
    frame(
        &mut app,
        &ctx,
        vec![key(Key::A, ctrl), Event::Text("0.3".into())],
    );
    control(&mut app, "Display");
    wait(&mut app);
    assert_eq!(
        app.controller.addon_session().unwrap().reply.preview.len(),
        4,
        "dialog={:?} focus={:?} draft={}",
        app.controller.addon_dialog(),
        ctx.memory(|m| m.focused()),
        app.transcript.draft()
    );
    assert_eq!(app.controller.document(), &doc);
    if let Some(dir) = std::env::var_os("OM9_F6_SHOT_DIR") {
        om9_app::ui_replay::screenshot(
            &mut app,
            [1280., 800.],
            1.,
            20,
            &std::path::Path::new(&dir).join("split-curves-display.png"),
        )
        .unwrap();
    }
    control(&mut app, "Cancel");
    assert!(app.controller.addon_session().is_none());
    assert_eq!(app.controller.document(), &doc);
    scroll_click_f6(&mut app, &ctx, "ts Split Curves");
    frame(&mut app, &ctx, vec![]);
    control(&mut app, "UseFileTolerance");
    control(&mut app, "Tolerance");
    frame(
        &mut app,
        &ctx,
        vec![key(Key::A, ctrl), key(Key::Backspace, Modifiers::NONE)],
    );
    control(&mut app, "OK");
    wait(&mut app);
    assert!(app.controller.addon_dialog().unwrap().enabled("Tolerance"));
    assert_eq!(app.controller.document(), &doc);
    control(&mut app, "Tolerance");
    frame(
        &mut app,
        &ctx,
        vec![key(Key::A, ctrl), Event::Text("0.3".into())],
    );
    control(&mut app, "OK");
    wait(&mut app);
    assert!(app.controller.addon_session().is_none());
    assert_eq!(app.controller.document().entities.len(), 7);
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &doc);
}

#[test]
fn click_factory_from_lines_removes_restores_and_publishes_faces() {
    use om9_app::{f6::catalog::Mode, session::from_lines::CageRecord};
    use om9_core::{curve_build::polyline, geometry::Geometry, model::Entity};
    let mut doc = Document::with_template_tolerance("From Lines controls");
    let a = [0., -6., 0.];
    let b = [0., 6., 0.];
    let mut ids = vec![];
    for points in [
        vec![a, b],
        vec![a, [-8., -6., 0.], [-8., 6., 0.], b],
        vec![a, [8., -6., 0.], [8., 6., 0.], b],
    ] {
        let e = Entity::new(
            "wire",
            Geometry::Curve(polyline(&points).unwrap()),
            doc.current_layer,
        );
        ids.push(e.id);
        doc.entities.insert(e.id, e);
    }
    let mut app = MatrixApp::new(doc.clone()).unwrap();
    app.f6_mode = Mode::TSplines;
    app.controller.select_objects(&ids).unwrap();
    let ctx = egui::Context::default();
    let wait = |app: &mut MatrixApp| {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
        while app.controller.from_lines_pending() {
            assert!(std::time::Instant::now() < deadline);
            frame(app, &ctx, vec![]);
        }
        for _ in 0..3 {
            frame(app, &ctx, vec![]);
        }
    };
    let click = |app: &mut MatrixApp, p| {
        frame(app, &ctx, vec![Event::PointerMoved(p)]);
        for pressed in [true, false] {
            frame(
                app,
                &ctx,
                vec![Event::PointerButton {
                    pos: p,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Modifiers::NONE,
                }],
            );
        }
    };
    scroll_click_f6(&mut app, &ctx, "ts From Lines");
    wait(&mut app);
    assert_eq!(app.controller.from_lines_layout().unwrap().faces().len(), 2);
    let p = top_pixel(&app, [4., 2., 0.]);
    click(&mut app, p);
    wait(&mut app);
    assert_eq!(app.controller.from_lines_layout().unwrap().faces().len(), 1);
    let p = top_pixel(&app, [8., 0., 0.]);
    click(&mut app, p);
    wait(&mut app);
    assert_eq!(app.controller.from_lines_choices().len(), 1);
    frame(&mut app, &ctx, vec![key(Key::Escape, Modifiers::NONE)]);
    wait(&mut app);
    assert!(app.controller.from_lines_choices().is_empty());
    assert_eq!(app.controller.from_lines_layout().unwrap().faces().len(), 1);
    click(&mut app, p);
    wait(&mut app);
    assert_eq!(app.controller.from_lines_choices().len(), 1);
    if let Some(dir) = std::env::var_os("OM9_F6_SHOT_DIR") {
        om9_app::ui_replay::screenshot(
            &mut app,
            [1280., 800.],
            1.,
            20,
            &std::path::Path::new(&dir).join("from-lines-chooser.png"),
        )
        .unwrap();
        frame(&mut app, &ctx, vec![]);
    }
    let row = app
        .session_dialog_control_rect("FromLines:Face1")
        .unwrap()
        .center();
    click(&mut app, row);
    wait(&mut app);
    assert_eq!(app.controller.from_lines_layout().unwrap().faces().len(), 2);
    frame(&mut app, &ctx, vec![key(Key::Enter, Modifiers::NONE)]);
    wait(&mut app);
    assert!(app.controller.addon_session().is_none());
    let id = app.controller.selected_ordered()[0];
    let cage = CageRecord::read(&app.controller.document().entities[&id]).unwrap();
    assert!(cage.smooth);
    assert_eq!(cage.polygons.len(), 2);
    assert_eq!(cage.display_face_ids.len(), 8);
    let archive=om9_archive::Archive::encode(app.controller.document(),&Default::default()).unwrap();
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &doc);
    let restored=om9_archive::Archive::decode(&archive,&Default::default()).unwrap();
    let mut reopened=MatrixApp::new(restored.clone()).unwrap();
    let ctx=egui::Context::default();
    reopened.f6_mode=Mode::TSplines;
    reopened.set_sub_object_kind(om9_scene::scene::PickKind::Face);
    click_top_component(&mut reopened,&ctx,[4.,2.,0.]);
    assert_eq!(reopened.controller.subselection().len(),4);
    scroll_click_f6(&mut reopened,&ctx,"Delete Face(s)");
    let cage=CageRecord::read(&reopened.controller.document().entities[&id]).unwrap();
    assert_eq!(cage.polygons.len(),1);
    assert_eq!(cage.display_face_ids.len(),4);
    reopened.controller.undo().unwrap();
    assert_eq!(reopened.controller.document(),&restored);
}

#[cfg(feature = "occt-brep")]
#[test]
fn extrude_lines_boundary_click_uses_visible_box_face_in_shaded_view() {
    use om9_core::{geometry::{Curve,Geometry},model::Entity};
    use om9_scene::workspace::{DisplayMode,ViewId};
    let mut doc=Document::with_template_tolerance("Boundary box face");
    let e=Entity::new("profile",Geometry::Curve(Curve::Line{start:[-1.,0.,0.],end:[1.,0.,0.],domain:[0.,1.]}),doc.current_layer);
    let id=e.id; doc.entities.insert(id,e);
    let mut app=MatrixApp::new(doc).unwrap(); let ctx=egui::Context::default();
    app.controller.activate_view("Looking Down").unwrap();
    for input in ["Box","-4,-4,4","4,4,4","2"] {app.controller.submit(input).unwrap();}
    let before=app.controller.document().clone();
    app.controller.workspace.views.get_mut(&ViewId::LookingDown).unwrap().mode=DisplayMode::Shaded;
    app.controller.select_objects(&[id]).unwrap();
    app.controller.submit("tsExtrudeLines").unwrap();
    app.controller.submit("ToBoundary").unwrap();
    frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);
    let pick=view_pixel(&app,ViewId::LookingDown,[2.,2.,6.]);
    click_at(&mut app,&ctx,pick);
    let deadline=std::time::Instant::now()+std::time::Duration::from_secs(5);
    while app.controller.addon_session().is_some() {
        assert!(std::time::Instant::now()<deadline,"{} {:?}",app.status_line(),app.controller.addon_session().unwrap().reply.messages);
        frame(&mut app,&ctx,vec![]);
    }
    let mut ends=Vec::new();
    for id in app.controller.selected_ordered() {
        if let Geometry::Curve(Curve::Line{start,end,..})=&app.controller.document().entities[&id].geometry
            && (start[0]-end[0]).abs()<1e-7 {ends.push(end[2]);}
    }
    assert_eq!(ends.len(),2);
    assert!(ends.iter().all(|&v|(v-6.).abs()<1e-6),"{ends:?}");
    app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
}

#[cfg(feature = "occt-brep")]
#[test]
fn click_factory_extrude_lines_to_boundary_picks_one_surface_and_closes() {
    use om9_core::{geometry::{Curve,Geometry},model::Entity,surface_kernel};
    use om9_scene::workspace::ViewId;
    let mut doc=Document::with_template_tolerance("ToBoundary mouse");
    let mut profiles=Vec::new();
    for y in [-2.,0.,2.] {
        let curve=Curve::Line{start:[-3.,y,0.],end:[3.,y,0.],domain:[0.,1.]};
        let e=Entity::new("curve",Geometry::Curve(curve),doc.current_layer);
        profiles.push(e.id);doc.entities.insert(e.id,e);
    }
    let mut surface=surface_kernel::plane([-5.,-4.,5.],[1.,0.,0.],[0.,1.,0.],10.,8.,&Default::default()).unwrap();
    for p in &mut surface.poles {p[2]+=0.5*p[0];}
    let face=Entity::new("boundary",Geometry::Surface(surface),doc.current_layer);
    let id=face.id;
    doc.entities.insert(id,face);
    let mut app=MatrixApp::new(doc.clone()).unwrap();
    let ctx=egui::Context::default();
    app.controller.workspace.active=ViewId::LookingDown;
    app.controller.select_objects(&profiles).unwrap();
    app.f6_mode=om9_app::f6::catalog::Mode::TSplines;
    scroll_click_f6(&mut app,&ctx,"ts Extrude Lines");
    frame(&mut app,&ctx,vec![]);
    let option=app.command_option_token_rect("ToBoundary").unwrap();
    click_at(&mut app,&ctx,option.center());
    assert!(app.controller.addon_collects_objects());
    assert_eq!(app.controller.document(),&doc);
    frame(&mut app,&ctx,vec![]);
    let pick=view_pixel(&app,ViewId::LookingDown,[0.,-4.,5.]);
    click_at(&mut app,&ctx,pick);
    for _ in 0..4 {frame(&mut app,&ctx,vec![]);}
    assert!(app.controller.addon_session().is_none(), "{}",app.status_line());
    assert_eq!(app.controller.document().entities[&id],doc.entities[&id]);
    assert_eq!(app.controller.selected_ordered().len(),9);
    let mut heights=Vec::new();
    for id in app.controller.selected_ordered() {
        if let Geometry::Curve(Curve::Line{start,end,..})=&app.controller.document().entities[&id].geometry
            && (start[0]-end[0]).abs()<1e-7 {
            heights.push(end[2]-start[2]);
        }
    }
    heights.sort_by(f64::total_cmp);
    assert_eq!(heights.len(),6);
    assert!(heights.iter().zip([3.5,3.5,3.5,6.5,6.5,6.5]).all(|(a,b)|(a-b).abs()<1e-6));
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(),&doc);
}

#[cfg(feature = "occt-brep")]
#[test]
fn click_factory_extrude_lines_solid_with_nested_and_spatial_profiles() {
    use om9_core::{
        curve_build::{circle_on_frame, polyline},
        geometry::Geometry,
        model::Entity,
    };
    use om9_scene::workspace::ViewId;
    let mut doc = Document::with_template_tolerance("Extrude Lines solid viewport");
    let circle = |r| circle_on_frame([0.; 3], [1., 0., 0.], [0., 1., 0.], r).unwrap();
    let spatial = polyline(&[
        [5., 0., 0.],
        [7., 0., 0.],
        [7., 2., 0.],
        [5., 2., 1.],
        [5., 0., 0.],
    ])
    .unwrap();
    let mut ids = Vec::new();
    // The factory 3Crv group matches the first three ordinary curves. A closed
    // planar curve has its own Matrix object class.
    let line_a = polyline(&[[10., 0., 0.], [12., 0., 0.]]).unwrap();
    let line_b = polyline(&[[10., 1., 0.], [12., 1., 0.]]).unwrap();
    for curve in [spatial, line_a, line_b, circle(3.), circle(1.)] {
        let entity = Entity::new("profile", Geometry::Curve(curve), doc.current_layer);
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut app = MatrixApp::new(doc.clone()).unwrap();
    let ctx = egui::Context::default();
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.select_objects(&ids).unwrap();
    app.f6_mode = om9_app::f6::catalog::Mode::TSplines;
    scroll_click_f6(&mut app, &ctx, "ts Extrude Lines");
    frame(&mut app, &ctx, vec![]);
    let split = app.command_option_token_rect("SplitAtTangents").expect("SplitAtTangents command option");
    click_at(&mut app, &ctx, split.center());
    assert!(app.controller.addon_session().unwrap().reply.options.iter()
        .any(|o| o.name == "SplitAtTangents" && o.value.as_deref() == Some("Yes")));
    frame(&mut app, &ctx, vec![]);
    let toggle = app.command_option_token_rect("Solid").expect("Solid command option");
    click_at(&mut app, &ctx, toggle.center());
    assert!(
        app.controller
            .addon_session()
            .unwrap()
            .reply
            .options
            .iter()
            .any(|o| o.name == "Solid" && o.value.as_deref() == Some("Yes"))
    );
    let pointer = view_pixel(&app, ViewId::ThroughFinger, [0., 0., 4.]);
    frame(&mut app, &ctx, vec![Event::PointerMoved(pointer)]);
    frame(&mut app, &ctx, vec![]);
    let preview = &app.controller.addon_session().unwrap().hover_lines;
    assert!(!preview.is_empty());
    assert_eq!(app.controller.document(), &doc);
    click_at(&mut app, &ctx, pointer);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while app.controller.addon_session().is_some() {
        assert!(std::time::Instant::now() < deadline, "{}", app.status_line());
        frame(&mut app, &ctx, vec![]);
        std::thread::yield_now();
    }
    let mut volumes = Vec::new();
    let mut wires = 0;
    for id in app.controller.selected_ordered() {
        match &app.controller.document().entities[&id].geometry {
            Geometry::Brep(body) => {
                assert_eq!(body.topology().faces.len(), 3);
                volumes.push(om9_kernel::brep::source_volume(body, &Default::default()).unwrap());
            }
            Geometry::Curve(_) => wires += 1,
            _ => panic!("unexpected extrusion geometry"),
        }
    }
    volumes.sort_by(f64::total_cmp);
    assert_eq!(volumes.len(), 2);
    assert!((volumes[0] / std::f64::consts::PI - 4.).abs() < 0.15);
    assert!((volumes[1] / volumes[0] - 9.).abs() < 1e-8);
    assert_eq!(wires, 14);
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &doc);
}

#[test]
fn click_factory_extrude_lines_hover_then_pick_through_finger_matches_cmd() {
    use om9_core::{
        geometry::{Curve, Geometry},
        model::Entity,
    };
    use om9_scene::workspace::ViewId;
    let mut doc = Document::with_template_tolerance("Extrude Lines viewport");
    let mut ids = Vec::new();
    for [start, end] in [
        [[0., 0., 0.], [4., 0., 0.]],
        [[4., 0., 0.], [4., 4., 0.]],
        [[4., 4., 0.], [0., 0., 0.]],
    ] {
        let e = Entity::new(
            "source",
            Geometry::Curve(Curve::Line {
                start,
                end,
                domain: [0., 1.],
            }),
            doc.current_layer,
        );
        ids.push(e.id);
        doc.entities.insert(e.id, e);
    }
    let mut app = MatrixApp::new(doc.clone()).unwrap();
    let ctx = egui::Context::default();
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.select_objects(&ids).unwrap();
    app.f6_mode = om9_app::f6::catalog::Mode::TSplines;
    scroll_click_f6(&mut app, &ctx, "ts Extrude Lines");
    frame(&mut app, &ctx, vec![]);
    let pointer = view_pixel(&app, ViewId::ThroughFinger, [0., 0., 5.]);
    frame(&mut app, &ctx, vec![Event::PointerMoved(pointer)]);
    frame(&mut app, &ctx, vec![]);
    let preview = app.controller.addon_session().unwrap().hover_lines.clone();
    assert_eq!(preview.len(), 6, "preview at {pointer:?}");
    let height = preview.iter().flatten().map(|p| p[2]).fold(0., f64::max);
    assert!((height - 5.).abs() < 0.15, "{height}");
    assert_eq!(app.controller.document(), &doc);
    for pressed in [true, false] {
        frame(
            &mut app,
            &ctx,
            vec![Event::PointerButton {
                pos: pointer,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: Modifiers::NONE,
            }],
        );
    }
    assert!(app.controller.addon_session().is_none());
    let output = |app: &MatrixApp| {
        let mut lines = Vec::new();
        for id in app.controller.selected_ordered() {
            let Geometry::Curve(Curve::Line { start, end, .. }) =
                &app.controller.document().entities[&id].geometry
            else {
                panic!("line")
            };
            lines.push([*start, *end]);
        }
        lines.sort_by(|a, b| {
            a.iter()
                .flatten()
                .zip(b.iter().flatten())
                .map(|(x, y)| x.total_cmp(y))
                .find(|o| !o.is_eq())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        lines
    };
    let actual = output(&app);
    assert_eq!(actual.len(), 6);
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &doc);
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.select_objects(&ids).unwrap();
    app.controller.submit_effect("tsExtrudeLines").unwrap();
    app.controller.submit_effect(&height.to_string()).unwrap();
    assert_eq!(output(&app), actual);
    app.controller.undo().unwrap();
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.select_objects(&ids).unwrap();
    scroll_click_f6(&mut app, &ctx, "ts Extrude Lines");
    text(&mut app, &ctx, "Direction");
    assert!(
        app.controller
            .addon_session()
            .unwrap()
            .reply
            .prompt
            .contains("Start of extrusion direction")
    );
    for (i, p) in [[-7., 0., -5.], [-3., 0., 3.], [6., 0., 12.]]
        .into_iter()
        .enumerate()
    {
        let pos = view_pixel(&app, ViewId::ThroughFinger, p);
        frame(&mut app, &ctx, vec![Event::PointerMoved(pos)]);
        frame(&mut app, &ctx, vec![]);
        if i == 2 {
            assert_eq!(app.controller.document(), &doc);
            assert!(
                app.controller
                    .addon_session()
                    .unwrap()
                    .reply
                    .prompt
                    .contains("Extrusion distance")
            );
            let preview = &app.controller.addon_session().unwrap().hover_lines;
            assert_eq!(preview.len(), 6);
            assert!(
                preview
                    .iter()
                    .any(|[a, b]| (b[0] - a[0] - 6.).abs() < 0.15
                        && (b[2] - a[2] - 12.).abs() < 0.15)
            );
            if let Some(dir) = std::env::var_os("OM9_F6_SHOT_DIR") {
                om9_app::ui_replay::screenshot_at_pointer(
                    &mut app,
                    [1280., 800.],
                    1.,
                    12,
                    Some(pos),
                    &std::path::Path::new(&dir).join("extrude-lines-direction.png"),
                )
                .unwrap();
                frame(&mut app, &ctx, vec![Event::PointerMoved(pos)]);
            }
        }
        for pressed in [true, false] {
            frame(
                &mut app,
                &ctx,
                vec![Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Modifiers::NONE,
                }],
            );
        }
    }
    assert!(app.controller.addon_session().is_none());
    assert_eq!(
        output(&app)
            .iter()
            .filter(|[a, b]| (b[0] - a[0] - 6.).abs() < 0.15 && (b[2] - a[2] - 12.).abs() < 0.15)
            .count(),
        3
    );
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &doc);
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.select_objects(&ids).unwrap();
    scroll_click_f6(&mut app, &ctx, "ts Extrude Lines");
    text(&mut app, &ctx, "-2.25");
    assert!(app.controller.addon_session().is_none());
    assert_eq!(
        output(&app)
            .iter()
            .filter(|[a, b]| (b[2] - a[2] + 2.25).abs() < 1e-9)
            .count(),
        3
    );
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &doc);
}

#[test]
fn f6_click_viewport_to_file_asks_for_path_and_escape_preserves_document() {
    let doc = Document::with_template_tolerance("Capture");
    let mut app = MatrixApp::new(doc.clone()).unwrap();
    let ctx = egui::Context::default();
    app.f6_mode = om9_app::f6::catalog::Mode::General;
    frame(&mut app, &ctx, vec![]);
    scroll_click_f6(&mut app, &ctx, "Viewport to File");
    assert!(app.controller.is_running());
    frame(&mut app, &ctx, vec![key(Key::Escape, Modifiers::NONE)]);
    assert!(!app.controller.is_running());
    assert_eq!(app.controller.document(), &doc);
}

#[test]
fn curve_direction_factory_click_flips_only_the_clicked_curve_and_escape_discards() {
    use om9_core::{
        geometry::{Curve, Geometry},
        model::Entity,
    };
    use om9_scene::workspace::ViewId;
    let mut doc = Document::with_template_tolerance("Two curve directions");
    let mut ids = Vec::new();
    for y in [-4., 4.] {
        let e = Entity::new(
            "line",
            Geometry::Curve(Curve::Line {
                start: [-8., y, 0.],
                end: [8., y, 0.],
                domain: [2., 8.],
            }),
            doc.current_layer,
        );
        ids.push(e.id);
        doc.entities.insert(e.id, e);
    }
    let mut app = MatrixApp::new(doc.clone()).unwrap();
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.select_objects(&ids).unwrap();
    app.f6_mode = om9_app::f6::catalog::Mode::TSplines;
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    scroll_click_f6(&mut app, &ctx, "Curve Direction");
    assert!(app.controller.prompt().contains("FlipAll"));
    let pointer = view_pixel(&app, ViewId::LookingDown, [2., 4., 0.]);
    let arrows = |frame: egui::FullOutput| {
        frame
            .shapes
            .iter()
            .filter_map(|s| match &s.shape {
                egui::Shape::LineSegment { points, stroke }
                    if stroke.color == egui::Color32::WHITE
                        && (stroke.width - 1.6).abs() < 0.01
                        && (points[0].distance(points[1]) - 40.).abs() < 0.1 =>
                {
                    Some(*points)
                }
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    frame(&mut app, &ctx, vec![Event::PointerMoved(pointer)]);
    let before = arrows(frame(&mut app, &ctx, vec![]));
    for x in [-8.,0.,8.] {
        let anchor=view_pixel(&app,ViewId::LookingDown,[x,4.,0.]);
        assert!(before.iter().any(|[a,b]|a.distance(anchor)<2. && b.x>a.x+39.), "stationary direction arrow at {anchor:?}");
    }
    assert!(
        before
            .iter()
            .any(|[a, b]| a.distance(pointer) < 3. && b.x > a.x + 20.),
        "hover arrow at {pointer:?}: {before:?}; pick {:?}",
        {
            let r = app.view_rects[&ViewId::LookingDown];
            let v = om9_scene::camera::Viewport {
                origin: [r.left() as f64, r.top() as f64],
                size: [r.width() as f64, r.height() as f64],
            };
            app.controller
                .scene()
                .unwrap()
                .pick(
                    [pointer.x as f64, pointer.y as f64],
                    &app.controller.workspace.views[&ViewId::LookingDown].camera,
                    v,
                    6.,
                    om9_scene::scene::PickIntent::Select,
                )
                .map(|h| h.map(|h| (h.path, h.world_point)))
        }
    );
    click_at(&mut app, &ctx, pointer);
    let after = arrows(frame(&mut app, &ctx, vec![Event::PointerMoved(pointer)]));
    assert!(
        after
            .iter()
            .any(|[a, b]| a.distance(pointer) < 3. && b.x < a.x - 20.),
        "flipped arrow {after:?}"
    );
    assert_eq!(app.controller.document(), &doc);
    app.controller.submit("").unwrap();
    assert_eq!(
        app.controller.document().entities[&ids[0]],
        doc.entities[&ids[0]]
    );
    assert!(
        matches!(app.controller.document().entities[&ids[1]].geometry,Geometry::Curve(Curve::Line{start,domain,..}) if start==[8.,4.,0.] && domain==[-8.,-2.])
    );
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &doc);
    app.controller.select_objects(&ids).unwrap();
    app.controller.submit("noecho !_Dir").unwrap();
    app.controller.submit("FlipAll").unwrap();
    frame(&mut app, &ctx, vec![key(Key::Escape, Modifiers::NONE)]);
    assert_eq!(app.controller.document(), &doc);
    assert!(!app.controller.is_running());
    app.controller.select_objects(&[]).unwrap();
    app.controller.submit("_Dir").unwrap();
    click_at(&mut app, &ctx, pointer);
    app.controller.submit("").unwrap();
    assert!(app.controller.prompt().contains("( Flip )"));
    app.controller.submit("Flip").unwrap();
    app.controller.submit("").unwrap();
    assert!(
        matches!(app.controller.document().entities[&ids[1]].geometry,Geometry::Curve(Curve::Line{domain,..}) if domain==[-8.,-2.])
    );
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &doc);
}

#[cfg(feature = "occt-brep")]
#[test]
fn extrude_to_point_factory_click_previews_and_commits_the_picked_apex() {
    use om9_core::{curve_build::circle_on_frame, geometry::Geometry, model::Entity};
    use om9_scene::workspace::ViewId;
    let mut doc = Document::with_template_tolerance("F6 apex");
    let curve = circle_on_frame([0.; 3], [1., 0., 0.], [0., 1., 0.], 2.).unwrap();
    let source = Entity::new("circle", Geometry::Curve(curve), doc.current_layer);
    let id = source.id;
    doc.entities.insert(id, source);
    let mut app = MatrixApp::new(doc.clone()).unwrap();
    let ctx = egui::Context::default();
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.select_objects(&[id]).unwrap();
    app.f6_mode = om9_app::f6::catalog::Mode::General;
    frame(&mut app, &ctx, vec![]);
    scroll_click_f6(&mut app, &ctx, "Extrude to Point");
    frame(&mut app, &ctx, vec![]);
    let solid = app.command_option_token_rect("Solid").unwrap().center();
    click_at(&mut app, &ctx, solid);
    let pointer = view_pixel(&app, ViewId::ThroughFinger, [1., 0., 6.]);
    frame(&mut app, &ctx, vec![Event::PointerMoved(pointer)]);
    frame(&mut app, &ctx, vec![]);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while app
        .controller
        .addon_session()
        .unwrap()
        .hover_lines
        .is_empty()
    {
        assert!(
            std::time::Instant::now() < deadline,
            "no apex preview: {}",
            app.status_line()
        );
        frame(&mut app, &ctx, vec![]);
        std::thread::yield_now();
    }
    assert_eq!(app.controller.document(), &doc);
    click_at(&mut app, &ctx, pointer);
    frame(&mut app, &ctx, vec![]);
    assert!(
        app.controller.addon_session().is_none(),
        "{}",
        app.status_line()
    );
    let output = app.controller.selected_ordered()[0];
    let Geometry::Brep(body) = &app.controller.document().entities[&output].geometry else {
        panic!("expected cone")
    };
    assert_eq!(body.topology().faces.len(), 2);
    let volume = om9_kernel::brep::source_volume(body, &Default::default()).unwrap();
    assert!((volume - 8. * std::f64::consts::PI).abs() < 1.0, "{volume}");
    assert!(app.controller.document().entities.contains_key(&id));
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &doc);
}

#[cfg(feature = "occt-brep")]
#[test]
fn tapered_factory_row_previews_and_extrudes_three_curves_at_picked_height() {
    use om9_core::{
        geometry::{Curve, Geometry},
        model::Entity,
    };
    use om9_scene::workspace::ViewId;
    let mut doc = Document::with_template_tolerance("Surface F6 tapered extrusion");
    let mut ids = vec![];
    for y in [-4., 0., 4.] {
        let line = Curve::Line {
            start: [-2., y, 0.],
            end: [2., y, 0.],
            domain: [0., 1.],
        };
        let e = Entity::new("line", Geometry::Curve(line), doc.current_layer);
        ids.push(e.id);
        doc.entities.insert(e.id, e);
    }
    let mut app = MatrixApp::new(doc.clone()).unwrap();
    let ctx = egui::Context::default();
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.select_objects(&ids).unwrap();
    app.f6_mode = om9_app::f6::catalog::Mode::Surface;
    frame(&mut app, &ctx, vec![]);
    scroll_click_f6(&mut app, &ctx, "Extrude Tapered");
    frame(&mut app, &ctx, vec![]);
    let flip = app.command_option_token_rect("FlipAngle").unwrap().center();
    click_at(&mut app, &ctx, flip);
    let opt = &app.controller.addon_session().unwrap().reply.options;
    assert!(
        opt.iter()
            .any(|o| o.name == "DraftAngle" && o.value.as_deref() == Some("-5"))
    );
    let pointer = view_pixel(&app, ViewId::ThroughFinger, [0., 0., 3.]);
    frame(&mut app, &ctx, vec![Event::PointerMoved(pointer)]);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(12);
    while app
        .controller
        .addon_session()
        .unwrap()
        .hover_lines
        .is_empty()
    {
        assert!(
            std::time::Instant::now() < deadline,
            "no taper preview: {}",
            app.status_line()
        );
        frame(&mut app, &ctx, vec![]);
        std::thread::yield_now();
    }
    assert_eq!(app.controller.document(), &doc);
    click_at(&mut app, &ctx, pointer);
    while app.controller.addon_session().is_some() {
        assert!(
            std::time::Instant::now() < deadline,
            "{}",
            app.status_line()
        );
        frame(&mut app, &ctx, vec![]);
        std::thread::yield_now();
    }
    let mut high = Vec::new();
    for id in app.controller.selected_ordered() {
        let Geometry::Brep(b) = &app.controller.document().entities[&id].geometry else {
            panic!("expected ruled face")
        };
        assert_eq!(b.topology().faces.len(), 1);
        let bounds=om9_kernel::brep::world_bounds(b,&om9_core::geometry::Transform::identity(),&Default::default()).unwrap();
        assert!((bounds[5]-3.).abs()<0.08);
        high.push(bounds[1]);
    }
    high.sort_by(f64::total_cmp);
    assert_eq!(high.len(), 3);
    for (y, base) in high.into_iter().zip([-4., 0., 4.]) {
        assert!((y - base + 3. * 5f64.to_radians().tan()).abs() < 0.02);
    }
    assert_eq!(app.controller.coordinator().undo_len(), 1);
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &doc);
}
