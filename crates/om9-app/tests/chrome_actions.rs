// SPDX-License-Identifier: LGPL-2.1-or-later
use eframe::egui::{self, Event, Key, Modifiers, Pos2, Rect, Vec2};
use om9_app::ui::MatrixApp;
use om9_core::model::Document;
use om9_scene::workspace::ViewId;

fn frame(app: &mut MatrixApp, context: &egui::Context, events: Vec<Event>) -> egui::FullOutput {
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 900.))),
            events,
            ..Default::default()
        },
        |ui| app.show(ui),
    );
    output.textures_delta.clear();
    output
}
fn key(key: Key) -> Event {
    Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    }
}
fn click(app: &mut MatrixApp, context: &egui::Context, at: Pos2) -> egui::FullOutput {
    frame(
        app,
        context,
        vec![
            Event::PointerMoved(at),
            Event::PointerButton {
                pos: at,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Modifiers::NONE,
            },
        ],
    );
    frame(
        app,
        context,
        vec![
            Event::PointerMoved(at),
            Event::PointerButton {
                pos: at,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Modifiers::NONE,
            },
        ],
    )
}
fn frame_at(
    app: &mut MatrixApp,
    context: &egui::Context,
    size: Vec2,
    events: Vec<Event>,
) -> egui::FullOutput {
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, size)),
            events,
            ..Default::default()
        },
        |ui| app.show(ui),
    );
    output.textures_delta.clear();
    output
}

fn secondary_click(app: &mut MatrixApp, context: &egui::Context, at: Pos2) -> egui::FullOutput {
    frame(
        app,
        context,
        vec![
            Event::PointerMoved(at),
            Event::PointerButton {
                pos: at,
                button: egui::PointerButton::Secondary,
                pressed: true,
                modifiers: Modifiers::NONE,
            },
        ],
    );
    frame(
        app,
        context,
        vec![
            Event::PointerMoved(at),
            Event::PointerButton {
                pos: at,
                button: egui::PointerButton::Secondary,
                pressed: false,
                modifiers: Modifiers::NONE,
            },
        ],
    )
}

/// Click one authored Solid palette icon by name. The page follows the icon's position in
/// the authored menu, so a menu reorder cannot silently test a different icon.
fn solid_icon(
    app: &mut MatrixApp,
    context: &egui::Context,
    history: usize,
    name: &str,
    secondary: bool,
) {
    let index = om9_app::menu_catalog::SOLID
        .iter()
        .position(|icon| *icon == name)
        .unwrap_or_else(|| panic!("no Solid icon {name}"));
    context.memory_mut(|memory| {
        memory
            .data
            .insert_temp(egui::Id::new(("main-menu-page", "Solid")), index / 20)
    });
    frame(app, context, vec![]);
    let at = app.rail_icon_rects[history + 2 + index % 20].center();
    if secondary {
        secondary_click(app, context, at);
    } else {
        click(app, context, at);
    }
}

fn text_positions(output: &egui::FullOutput, label: &str) -> Vec<Pos2> {    output
        .shapes
        .iter()
        .filter_map(|clipped| {
            if let egui::Shape::Text(text) = &clipped.shape {
                (text.galley.job.text == label).then(|| text.visual_bounding_rect().center())
            } else {
                None
            }
        })
        .collect()
}
fn datum_strokes(output: &egui::FullOutput) -> usize {
    let color = egui::Color32::from_rgb(255, 170, 40);
    output
        .shapes
        .iter()
        .filter(|clipped| match &clipped.shape {
            egui::Shape::LineSegment { stroke, .. } => stroke.color == color,
            egui::Shape::Path(path) => {
                matches!(path.stroke.color, eframe::epaint::ColorMode::Solid(c) if c == color)
            }
            _ => false,
        })
        .count()
}

fn startup() -> (MatrixApp, egui::Context, egui::FullOutput) {
    let mut app = MatrixApp::new(Document::new("chrome pointer fixtures")).unwrap();
    let context = egui::Context::default();
    frame(&mut app, &context, vec![]);
    let output = frame(&mut app, &context, vec![]);
    (app, context, output)
}

#[test]
fn line_history_keeps_both_prompts_after_typed_or_viewport_start() {
    use om9_core::geometry::{Curve, Geometry};
    use om9_scene::camera::Viewport;

    for (typed_command, typed_start) in [(false, false), (true, false), (false, true), (true, true)] {
        let mut app = MatrixApp::new(Document::with_template_tolerance("Line history")).unwrap();
        let context = egui::Context::default();
        frame(&mut app, &context, vec![]);
        frame(&mut app, &context, vec![]);
        if typed_command {
            let editor = app.command_input_rect().unwrap().center();
            click(&mut app, &context, editor);
            frame(&mut app, &context, vec![Event::Text("Line".into())]);
            frame(&mut app, &context, vec![key(Key::Enter)]);
            frame(&mut app, &context, vec![Event::Key {
                key: Key::Enter, physical_key: None, pressed: false, repeat: false, modifiers: Modifiers::NONE,
            }]);
        } else {
            let category = app.main_category_rect(0).expect("Curve category").center();
            click(&mut app, &context, category);
            frame(&mut app, &context, vec![]);
            let index = om9_app::menu_catalog::CURVE.iter()
                .position(|icon| *icon == "CurveLineSingleLine").unwrap();
            let icon = app.rail_icon_rects[2 + index].center();
            click(&mut app, &context, icon);
        }
        assert_eq!(app.controller.running_feature(), Some(om9_app::session::LINE));
        let rect = app.view_rects[&ViewId::LookingDown];
        let viewport = Viewport {
            origin: [rect.min.x as f64, rect.min.y as f64],
            size: [rect.width() as f64, rect.height() as f64],
        };
        let camera = app.controller.workspace.views[&ViewId::LookingDown].camera.clone();
        let start = [4., 5., 0.];
        let end = [7., 8., 0.];
        for (index, point) in [start, end].into_iter().enumerate() {
            if index == 0 && typed_start {
                frame(&mut app, &context, vec![Event::Text("W4,5,0".into())]);
                frame(&mut app, &context, vec![key(Key::Enter)]);
            } else {
                let [x, y] = camera.project(point, viewport).unwrap().unwrap().screen;
                let at = Pos2::new(x as f32, y as f32);
                frame(&mut app, &context, vec![Event::PointerMoved(at)]);
                click(&mut app, &context, at);
            }
            assert_eq!(app.controller.is_running(), index == 0,
                "input {index}, typed_command={typed_command}, typed_start={typed_start}: {}", app.status_line());
        }
        let entity = app.controller.document().entities.values().next().expect("committed Line");
        let Geometry::Curve(Curve::Line { start: actual_start, end: actual_end, .. }) = entity.geometry else {
            panic!("expected a Line");
        };
        for (actual, target) in actual_start.into_iter().chain(actual_end).zip(start.into_iter().chain(end)) {
            assert!((actual - target).abs() < 1e-5);
        }
        assert_eq!(app.controller.coordinator().undo_len(), 1);
        let history: Vec<_> = app.transcript.history().lines().map(str::trim_end).collect();
        let mut expected = vec!["Command: Line", "Start of line (BothSides=No):"];
        if typed_start { expected.push("Start of line (BothSides=No): W4,5,0"); }
        expected.push("End of line:");
        assert!(history.ends_with(&expected), "{history:?}");
    }
}

#[test]
fn command_history_opens_from_cmd_icon_and_f2_without_consuming_point_input() {
    let (mut app, context, _) = startup();
    app.transcript.append_history("Between midpoint (World mm): 1,1,0");
    for row in 1..=48 {
        app.transcript.append_history(&format!("SNAPS history row {row}"));
    }
    let output = frame(&mut app, &context, vec![]);
    let top = app.section_rect("info-settings").unwrap().bottom();
    let bottom = app.section_rect("layers").unwrap().top();
    let history_icon = output.shapes.iter().filter_map(|clipped| {
        let egui::Shape::Mesh(mesh) = &clipped.shape else { return None };
        let center = mesh.calc_bounds().center();
        (mesh.texture_id != egui::TextureId::default() && center.x < 180.
            && center.y > top && center.y < bottom).then_some(center)
    }).nth(4).expect("Command History icon");
    let visible = |output: &egui::FullOutput| !text_positions(output, "Command History").is_empty();
    let editor = app.command_input_rect().unwrap().center();
    click(&mut app, &context, editor);
    frame(&mut app, &context, vec![Event::Text("CommandHistory".to_owned())]);
    frame(&mut app, &context, vec![key(Key::Enter)]);
    let opened = frame(&mut app, &context, vec![]);
    assert!(visible(&opened));
    assert!(opened.shapes.iter().any(|shape| {
        matches!(&shape.shape, egui::Shape::Text(text)
            if text.galley.job.text == app.transcript.history())
    }));
    assert!(!app.transcript.history().contains("History report"));
    let f2 = || vec![key(Key::F2), Event::Key {
        key: Key::F2,
        physical_key: None,
        pressed: false,
        repeat: false,
        modifiers: Modifiers::NONE,
    }];
    frame(&mut app, &context, f2());
    assert!(!visible(&frame(&mut app, &context, vec![])));

    click(&mut app, &context, history_icon);
    assert!(visible(&frame(&mut app, &context, vec![])));
    click(&mut app, &context, history_icon);
    assert!(!visible(&frame(&mut app, &context, vec![])));

    app.controller.submit("Line").unwrap();
    app.transcript.set_draft("4,5");
    frame(&mut app, &context, f2());
    let opened = frame(&mut app, &context, vec![]);
    assert!(visible(&opened));
    assert_eq!(app.transcript.draft(), "4,5");
    let history = app.transcript.history().to_owned();
    assert!(history.contains("Between midpoint (World mm): 1,1,0"));
    assert!(history.contains("SNAPS history row 48"));
    let history_text = opened.shapes.iter().find_map(|clipped| {
        let egui::Shape::Text(text) = &clipped.shape else { return None };
        if text.galley.job.text != history {
            return None;
        }
        let bounds = text.visual_bounding_rect();
        assert!(bounds.height() > clipped.clip_rect.height(), "fixture should extend beyond the scroll area");
        Some(bounds.intersect(clipped.clip_rect).center())
    }).expect("visible transcript text");
    click(&mut app, &context, history_text);
    let copied = frame(&mut app, &context, vec![
        Event::Key {
            key: Key::A,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers { ctrl: true, command: true, ..Modifiers::NONE },
        },
        Event::Copy,
    ]);
    assert!(copied.platform_output.commands.iter().any(|command| {
        matches!(command, egui::OutputCommand::CopyText(text) if text == &history)
    }), "Ctrl+A and Copy should return the complete transcript");
    frame(&mut app, &context, vec![Event::Cut, Event::Paste("invalid input".to_owned())]);
    assert_eq!(app.transcript.history(), history);
    assert_eq!(app.transcript.draft(), "4,5");
    assert!(app.controller.is_running());
    assert_eq!(app.controller.coordinator().undo_len(), 0);
}

fn begin_numeric_draft(app: &mut MatrixApp, context: &egui::Context) {
    let editor = app.command_input_rect().expect("command editor").center();
    click(app, context, editor);
    frame(app, context, vec![Event::Text("Line".into())]);
    frame(app, context, vec![key(Key::Enter)]);
    frame(app, context, vec![Event::Text("0,0,".into())]);
    assert!(app.controller.is_running());
    assert_eq!(app.transcript.draft(), "0,0,");
}
fn finish_numeric_draft(app: &mut MatrixApp, context: &egui::Context) {
    frame(app, context, vec![]);
    frame(app, context, vec![Event::Text("0".into())]);
    assert_eq!(
        app.transcript.draft(),
        "0,0,0",
        "view selection lost the editable numeric draft/focus"
    );
    frame(app, context, vec![key(Key::Enter)]);
    frame(app, context, vec![Event::Text("4,5,0".into())]);
    frame(app, context, vec![key(Key::Enter)]);
    assert_eq!(app.controller.document().entities.len(), 1);
    assert_eq!(app.controller.coordinator().undo_len(), 1);
}

fn snaps_icons(output: &egui::FullOutput) -> Vec<(Pos2, egui::TextureId, egui::Color32)> {
    let mut in_snaps = false;
    let mut icons = Vec::new();
    for clipped in &output.shapes {
        if let egui::Shape::Text(text) = &clipped.shape {
            if text.galley.job.text == "SNAPS" {
                in_snaps = true;
            }
            if text.galley.job.text == "INFO & SETTINGS" {
                in_snaps = false;
            }
        }
        if in_snaps
            && let egui::Shape::Mesh(mesh) = &clipped.shape
            && mesh.texture_id != egui::TextureId::default()
            && clipped.clip_rect.contains(mesh.calc_bounds().center())
        {
            icons.push((mesh.calc_bounds().center(), mesh.texture_id, mesh.vertices[0].color));
        }
    }
    icons
}

fn active_osnap_master_frame(output: &egui::FullOutput, end: Pos2) -> Rect {
    output.shapes.iter().find_map(|shape| match &shape.shape {
        egui::Shape::Rect(rect) if rect.rect.left() > end.x
            && (rect.rect.center().y - end.y).abs() < 0.5
            && rect.stroke.color == egui::Color32::from_rgb(90, 190, 245) => Some(rect.rect),
        _ => None,
    }).expect("active O-Snap button frame to the right of End")
}

#[test]
fn apparent_intersection_option_uses_the_osnap_context_menu() {
    let (mut app, context, output) = startup();
    let master = active_osnap_master_frame(&output, snaps_icons(&output)[8].0).center();
    for (master_enabled, status) in [(false, "ApparentIntersections on"), (true, "ApparentIntersections off")] {
        app.set_object_snaps_enabled(master_enabled);
        frame(&mut app, &context, vec![Event::PointerMoved(master)]);
        let opened = secondary_click(&mut app, &context, master);
        assert!(egui::Popup::is_any_open(&context), "{status} at {master:?}");
        let output = frame(&mut app, &context, vec![]);
        let label = "Snap to apparent intersections";
        let option = text_positions(&output, label).into_iter()
            .chain(text_positions(&opened, label))
            .next().unwrap_or_else(|| panic!("missing apparent intersection option for {status}"));
        click(&mut app, &context, option);
        assert!(app.status_line().contains(status), "{}", app.status_line());
        assert_eq!(app.object_snaps_enabled(), master_enabled);
        assert!(app.controller.document().entities.is_empty());
    }
}

#[test]
fn snaps_are_noninvoking_even_where_the_old_sidebar_displayed_point_and_line() {
    let (mut app, context, output) = startup();
    let icons = snaps_icons(&output);
    assert!(icons.len() >= 4, "snap icons were not rendered");
    for (position, _, _) in icons {
        click(&mut app, &context, position);
        assert!(
            !app.controller.is_running(),
            "sidebar snap icon invoked geometry at {position:?}"
        );
        assert!(app.controller.document().entities.is_empty());
        assert_eq!(app.controller.coordinator().undo_len(), 0);
    }
}

#[test]
fn remaining_snap_icons_and_cmd_aliases_commit_the_same_source_point() {
    use om9_core::geometry::{Curve, Geometry};
    use om9_core::model::Entity;
    use om9_scene::{camera::Viewport, osnap::SnapKind};

    let circle = Geometry::Curve(Curve::Circle {
        center: [0., 0., 5.],
        x: [1., 0., 0.],
        y: [0., 1., 0.],
        radius: 5.,
        domain: [0., std::f64::consts::TAU],
        angle_domain: None,
    });
    let tangent = [2.5, 18.75_f64.sqrt(), 5.];
    let cases = [
        (SnapKind::Perpendicular, "Perp", 0, circle.clone(), [5., 0., 5.], true),
        (SnapKind::Tangent, "Tan", 1, circle.clone(), tangent, true),
        (SnapKind::Point, "Point", 3, Geometry::Point([-8., 6., 7.]), [-8., 6., 7.], false),
        (SnapKind::Near, "Near", 4, Geometry::Curve(Curve::Line {
            start: [-12., 1., 5.], end: [-2., 1., 5.], domain: [0., 1.],
        }), [-7., 1., 5.], false),
        (SnapKind::Quadrant, "Quad", 5, circle.clone(), [5., 0., 5.], false),
        (SnapKind::Center, "Cen", 6, circle, [0., 0., 5.], false),
    ];

    for (kind, command, icon, geometry, witness, directional) in cases {
        let mut document = Document::with_template_tolerance(format!("{command} point input"));
        let source = Entity::new("Reference", geometry, None);
        let id = source.id;
        document.entities.insert(id, source);
        let mut app = MatrixApp::new(document).unwrap();
        app.set_snap_enabled(SnapKind::End, false);
        app.set_snap_enabled(SnapKind::Mid, false);
        app.set_smarttrack_enabled(false);
        app.controller.workspace.active = ViewId::LookingDown;
        let context = egui::Context::default();
        frame(&mut app, &context, vec![]);
        let output = frame(&mut app, &context, vec![]);
        let button = snaps_icons(&output)[icon].0;
        let rect = app.view_rects[&ViewId::LookingDown];
        let viewport = Viewport {
            origin: [rect.min.x as f64, rect.min.y as f64],
            size: [rect.width() as f64, rect.height() as f64],
        };
        let mut pointer = witness;
        if kind == SnapKind::Near { pointer[1] += 0.2; }
        let [x, y] = app.controller.workspace.views[&ViewId::LookingDown].camera
            .project(pointer, viewport).unwrap().unwrap().screen;
        let cursor = Pos2::new(x as f32, y as f32);
        let before = app.controller.document().clone();
        let tolerance = before.effective_tolerance().unwrap().absolute_mm;

        for from_cmd in [false, true] {
            if !from_cmd {
                click(&mut app, &context, button);
                assert!(app.snap_enabled(kind), "{command} palette binding");
            }
            app.controller.submit("Line").unwrap();
            if directional { app.controller.submit("10,0,5").unwrap(); }
            if from_cmd {
                let editor = app.command_input_rect().unwrap().center();
                click(&mut app, &context, editor);
                frame(&mut app, &context, vec![Event::Text(command.into())]);
                frame(&mut app, &context, vec![key(Key::Enter)]);
                assert_eq!(app.snap_override(), Some(kind),
                    "{command} one-shot, draft={:?}, prompt={:?}, status={}",
                    app.transcript.draft(), app.transcript.prompt(), app.status_line());
            }
            frame(&mut app, &context, vec![Event::PointerMoved(cursor)]);
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(8);
            let hit = loop {
                if let Some(hit) = app.snap_candidate() { break hit.clone(); }
                assert!(std::time::Instant::now() < deadline, "{command}: {}", app.status_line());
                std::thread::sleep(std::time::Duration::from_millis(5));
                frame(&mut app, &context, vec![]);
            };
            assert_eq!(hit.kind, kind);
            assert_eq!(hit.source.as_ref().unwrap().root, id);
            assert!(hit.world.into_iter().zip(witness).all(|(a, b)| (a - b).abs() < tolerance), "{command}: {hit:?}");
            click(&mut app, &context, cursor);
            assert_eq!(app.snap_override(), None);
            if !directional { app.controller.submit("20,15,0").unwrap(); }
            let created = app.controller.document().entities.values()
                .find(|entity| entity.id != id).expect("committed Line");
            let Geometry::Curve(Curve::Line { start, end, .. }) = &created.geometry else {
                panic!("expected Line after {command}");
            };
            assert_eq!(if directional { *end } else { *start }, hit.world);
            assert_eq!(app.controller.coordinator().undo_len(), 1);
            app.controller.undo().unwrap();
            assert_eq!(app.controller.document(), &before);
            app.set_snap_enabled(kind, false);
        }
    }
}

#[test]
fn between_icon_focuses_its_prompt_and_escape_preserves_the_outer_line() {
    use om9_core::geometry::{Curve, Geometry};
    use om9_scene::camera::Viewport;

    let mut app = MatrixApp::new(Document::with_template_tolerance("Between palette")).unwrap();
    let context = egui::Context::default();
    frame(&mut app, &context, vec![]);
    let between_icon = snaps_icons(&frame(&mut app, &context, vec![]))[11].0;
    let editor = app.command_input_rect().expect("command editor").center();
    click(&mut app, &context, editor);
    let submit = |app: &mut MatrixApp, text: &str| {
        frame(app, &context, vec![Event::Text(text.into())]);
        frame(app, &context, vec![key(Key::Enter)]);
    };
    submit(&mut app, "Line");
    let rect = app.view_rects[&ViewId::LookingDown];
    let viewport = Viewport {
        origin: [rect.min.x as f64, rect.min.y as f64],
        size: [rect.width() as f64, rect.height() as f64],
    };
    let camera = app.controller.workspace.views[&ViewId::LookingDown].camera.clone();
    let project = |point| {
        let [x, y] = camera.project(point, viewport).unwrap().unwrap().screen;
        Pos2::new(x as f32, y as f32)
    };
    let first_point = [4., 5., 0.];
    click(&mut app, &context, project(first_point));
    let command_editor = egui::Id::new("matrix-command-transcript");
    assert!(!context.memory(|memory| memory.has_focus(command_editor)));
    assert!(app.controller.relative_base().is_some(), "{}", app.status_line());
    let outer_prompt = app.controller.prompt_with_options();
    click(&mut app, &context, between_icon);
    assert!(app.transcript.prompt().starts_with("Between: first point"));
    assert!(context.memory(|memory| memory.has_focus(command_editor)));
    submit(&mut app, "1,2,0");
    assert!(app.transcript.prompt().starts_with("Between: second point"));

    frame(&mut app, &context, vec![key(Key::Escape)]);
    assert!(app.controller.is_running());
    assert_eq!(app.transcript.prompt(), outer_prompt);
    assert!(app.controller.document().entities.is_empty());
    assert_eq!(app.controller.coordinator().undo_len(), 0);
    assert!(!context.memory(|memory| memory.has_focus(command_editor)));
    assert!(app.transcript.history().contains(
        "Between: first point:\nBetween: first point: 1,2,0\nBetween: second point:\nBetween cancelled\n"
    ));

    let last_point = [7., 8., 0.];
    click(&mut app, &context, project(last_point));
    let document = app.controller.document();
    assert_eq!(document.entities.len(), 1, "{}", app.status_line());
    let Geometry::Curve(Curve::Line { start, end, .. }) =
        &document.entities.values().next().unwrap().geometry
    else {
        panic!("expected the outer Line");
    };
    for (coordinate, expected) in start.iter().zip(first_point) {
        assert!((coordinate - expected).abs() < 1e-4, "start was {start:?}");
    }
    for (coordinate, expected) in end.iter().zip(last_point) {
        assert!((coordinate - expected).abs() < 1e-4, "end was {end:?}");
    }
    assert_eq!(app.controller.coordinator().undo_len(), 1);

    frame(&mut app, &context, vec![key(Key::Space)]);
    assert!(context.memory(|memory| memory.has_focus(command_editor)));
    click(&mut app, &context, project([10., 5., 0.]));
    click(&mut app, &context, between_icon);
    submit(&mut app, "0,0,0");
    assert!(context.memory(|memory| memory.has_focus(command_editor)));
    submit(&mut app, "2,2,0");
    assert!(!app.controller.is_running());
    assert!(app.transcript.history().contains("Between: second point: 2,2,0\n"));
    assert!(app.transcript.history().contains("Between midpoint (World mm): 1,1,0\n"));
    assert_eq!(app.controller.coordinator().undo_len(), 2);
    assert!(app.controller.document().entities.values().any(|entity| {
        matches!(&entity.geometry, Geometry::Curve(Curve::Line { end, .. }) if *end == [1., 1., 0.])
    }));
    assert!(!context.memory(|memory| memory.has_focus(command_editor)));
    frame(&mut app, &context, vec![key(Key::E), Event::Text("e".into())]);
    assert_eq!(app.snap_override(), Some(om9_scene::osnap::SnapKind::End));
    assert!(app.transcript.draft().is_empty());
}

#[cfg(feature = "occt-brep")]
#[test]
fn deferred_polysurface_target_pick_focuses_the_typed_point_prompt() {
    use om9_core::geometry::{Curve, Geometry};
    use om9_scene::camera::Viewport;
    use std::time::{Duration, Instant};

    let mut app = MatrixApp::new(Document::with_template_tolerance("Polysurface palette")).unwrap();
    app.controller.submit("Box 10 10 10").unwrap();
    let initial_undo = app.controller.coordinator().undo_len();
    let context = egui::Context::default();
    frame(&mut app, &context, vec![]);
    let polysurface_icon = snaps_icons(&frame(&mut app, &context, vec![]))[10].0;
    app.controller.submit("Line").unwrap();
    secondary_click(&mut app, &context, polysurface_icon);
    assert!(app.transcript.prompt().starts_with("Select polysurface"));

    let rect = app.view_rects[&ViewId::LookingDown];
    let viewport = Viewport {
        origin: [rect.min.x as f64, rect.min.y as f64],
        size: [rect.width() as f64, rect.height() as f64],
    };
    let [x, y] = app.controller.workspace.views[&ViewId::LookingDown].camera
        .project([4., 4., 10.], viewport).unwrap().unwrap().screen;
    let cursor = Pos2::new(x as f32, y as f32);
    let permit = app.controller.coordinator().external_memory_account()
        .try_acquire_surface_snap().unwrap();
    click(&mut app, &context, cursor);
    assert!(app.snap_candidate().is_none(), "held permit must defer the target pick");
    assert!(app.transcript.prompt().starts_with("Select polysurface"));
    drop(permit);

    let wait_for = |app: &mut MatrixApp, check: &dyn Fn(&MatrixApp) -> bool| {
        let deadline = Instant::now() + Duration::from_secs(8);
        while !check(app) {
            assert!(Instant::now() < deadline, "{}", app.status_line());
            std::thread::sleep(Duration::from_millis(5));
            frame(app, &context, vec![]);
        }
    };
    wait_for(&mut app, &|app| app.transcript.prompt().starts_with("Start of line"));
    assert!(app.transcript.history().contains("Select polysurface:\nSurface target selected; pick a point on it\n"));
    assert!(context.memory(|memory| memory.has_focus(egui::Id::new("matrix-command-transcript"))));
    for (text, next_prompt) in [("4,4,10", "End of line"), ("4,4,0", "Command")] {
        frame(&mut app, &context, vec![Event::Text(text.into())]);
        frame(&mut app, &context, vec![key(Key::Enter)]);
        wait_for(&mut app, &|app| app.transcript.prompt().starts_with(next_prompt));
    }
    let line = app.controller.document().entities.values().find_map(|entity| match &entity.geometry {
        Geometry::Curve(Curve::Line { start, end, .. }) => Some((*start, *end)),
        _ => None,
    }).expect("completed Line");
    assert_eq!(line, ([4., 4., 10.], [4., 4., 0.]));
    assert_eq!(app.controller.coordinator().undo_len(), initial_undo + 1);
    let editor = egui::Id::new("matrix-command-transcript");
    assert!(!context.memory(|memory| memory.has_focus(editor)));
    frame(&mut app, &context, vec![key(Key::E), Event::Text("e".into())]);
    assert_eq!(app.snap_override(), Some(om9_scene::osnap::SnapKind::End));
    assert!(app.transcript.draft().is_empty());
}

#[cfg(feature = "occt-brep")]
#[test]
fn on_surface_icon_buttons_control_how_many_line_points_snap_to_a_sphere() {
    use om9_core::geometry::{Curve, Geometry};
    use om9_scene::{camera::Viewport, osnap::SnapKind};
    use std::time::{Duration, Instant};

    for persistent in [false, true] {
        let mut app = MatrixApp::new(Document::with_template_tolerance("On Surface pointer modes")).unwrap();
        app.controller.solid_sphere_points([0.; 3], 5.).unwrap();
        app.controller.workspace.active = ViewId::LookingDown;
        app.set_object_snaps_enabled(false);
        app.set_smarttrack_enabled(false);
        let before = app.controller.document().clone();
        let sphere = *before.entities.keys().next().unwrap();
        let undo = app.controller.coordinator().undo_len();
        let context = egui::Context::default();
        frame(&mut app, &context, vec![]);
        let surface_icon = snaps_icons(&frame(&mut app, &context, vec![]))[9].0;
        app.controller.submit("Line").unwrap();
        if persistent {
            secondary_click(&mut app, &context, surface_icon);
        } else {
            click(&mut app, &context, surface_icon);
        }
        assert!(app.transcript.prompt().starts_with("Select surface"));

        let rect = app.view_rects[&ViewId::LookingDown];
        let viewport = Viewport { origin: [rect.min.x as f64, rect.min.y as f64], size: [rect.width() as f64, rect.height() as f64] };
        let camera = app.controller.workspace.views[&ViewId::LookingDown].camera.clone();
        let screen = |x, y| {
            let [x, y] = camera.project([x, y, 0.], viewport).unwrap().unwrap().screen;
            Pos2::new(x as f32, y as f32)
        };
        let surface_at = |app: &mut MatrixApp, at: Pos2| {
            frame(app, &context, vec![Event::PointerMoved(at)]);
            let deadline = Instant::now() + Duration::from_secs(8);
            loop {
                if let Some(hit) = app.snap_candidate().filter(|hit| hit.kind == SnapKind::Surface && hit.root == Some(sphere)) {
                    return hit.world;
                }
                assert!(Instant::now() < deadline, "{}", app.status_line());
                std::thread::sleep(Duration::from_millis(5));
                frame(app, &context, vec![]);
            }
        };
        let target = screen(-2., 1.);
        surface_at(&mut app, target);
        click(&mut app, &context, target);
        assert!(app.transcript.prompt().starts_with("Start of line"));
        assert!(app.controller.relative_base().is_none());

        let first = screen(-1., 2.);
        let start = surface_at(&mut app, first);
        click(&mut app, &context, first);
        assert!(app.transcript.prompt().starts_with("End of line"));
        assert_eq!(app.controller.coordinator().undo_len(), undo);

        let second = screen(-3., -1.);
        let end = if persistent {
            let outside = screen(-7., 0.);
            let miss = "Surface snap unavailable: no surface candidate under the pointer";
            frame(&mut app, &context, vec![Event::PointerMoved(outside)]);
            assert!(!app.transcript.history().contains(miss), "hover must not log a miss");
            click(&mut app, &context, outside);
            assert_eq!(app.transcript.history().matches(miss).count(), 1);
            assert!(app.transcript.prompt().starts_with("End of line"));
            assert_eq!(app.controller.document(), &before);
            surface_at(&mut app, second)
        } else {
            [-3., -1., 0.]
        };
        click(&mut app, &context, second);
        let line = app.controller.document().entities.values().find_map(|entity| match &entity.geometry {
            Geometry::Curve(Curve::Line { start, end, .. }) => Some((*start, *end)),
            _ => None,
        }).expect("completed Line");
        let assert_point = |actual: [f64; 3], expected: [f64; 3]| {
            assert!(actual.into_iter().zip(expected).all(|(a, b)| (a - b).abs() < 1e-5), "{actual:?} expected {expected:?}");
        };
        assert_point(line.0, start);
        assert_point(line.0, [-1., 2., 20_f64.sqrt()]);
        assert_point(line.1, end);
        assert_point(line.1, [-3., -1., if persistent { 15_f64.sqrt() } else { 0. }]);
        assert_eq!(app.controller.coordinator().undo_len(), undo + 1);
        let history = app.transcript.history();
        assert!(history.contains("Select surface:\nSurface target selected; pick a point on it\n"));
        if persistent {
            frame(&mut app, &context, vec![key(Key::Escape)]);
            assert!(app.transcript.history().contains("Surface constraint cancelled\n"));
        }
        app.controller.undo().unwrap();
        assert_eq!(app.controller.document(), &before);
    }
}

#[test]
fn reopening_the_same_archive_drops_the_persistent_surface_target() {
    use om9_app::{file_lifecycle::FileKind, ui::FileControl};
    use om9_core::{budget::Limits, geometry::{Curve, Geometry}, model::Entity, surface_kernel};
    use om9_scene::{camera::Viewport, osnap::SnapKind};
    use std::time::{Duration, Instant};

    let plane = surface_kernel::plane(
        [0., 0., 5.], [1., 0., 0.], [0., 1., 0.], 10., 10., &Limits::default(),
    ).unwrap();
    let mut document = Document::with_template_tolerance("Surface reopen");
    let entity = Entity::new("plane target", Geometry::Surface(plane), None);
    let target = entity.id;
    document.entities.insert(target, entity);
    let path = std::env::temp_dir().join(format!("om9-surface-reopen-{}.om9", uuid::Uuid::new_v4()));
    MatrixApp::new(document).unwrap().controller.save_archive_for_batch(&path).unwrap();
    let mut app = MatrixApp::from_file(&path).unwrap();
    let original_source = app.controller.document().stamp(target).unwrap();
    let context = egui::Context::default();
    frame(&mut app, &context, vec![]);
    let surface_icon = snaps_icons(&frame(&mut app, &context, vec![]))[9].0;
    let wait_for = |app: &mut MatrixApp, check: &dyn Fn(&MatrixApp) -> bool| {
        let deadline = Instant::now() + Duration::from_secs(8);
        while !check(app) {
            assert!(Instant::now() < deadline, "{}", app.status_line());
            std::thread::sleep(Duration::from_millis(5));
            frame(app, &context, vec![]);
        }
    };

    app.controller.submit("Line").unwrap();
    secondary_click(&mut app, &context, surface_icon);
    assert!(app.transcript.prompt().starts_with("Select surface"));
    let rect = app.view_rects[&ViewId::LookingDown];
    let viewport = Viewport {
        origin: [rect.min.x as f64, rect.min.y as f64],
        size: [rect.width() as f64, rect.height() as f64],
    };
    let [x, y] = app.controller.workspace.views[&ViewId::LookingDown].camera
        .project([2., 2., 5.], viewport).unwrap().unwrap().screen;
    let cursor = Pos2::new(x as f32, y as f32);
    frame(&mut app, &context, vec![Event::PointerMoved(cursor)]);
    wait_for(&mut app, &|app| app.snap_candidate().is_some_and(|candidate| {
        candidate.kind == SnapKind::Surface && candidate.root == Some(target)
    }));
    click(&mut app, &context, cursor);
    let submit = |app: &mut MatrixApp, text: &str| {
        frame(app, &context, vec![Event::Text(text.into())]);
        frame(app, &context, vec![key(Key::Enter)]);
    };
    submit(&mut app, "2,2,0");
    wait_for(&mut app, &|app| app.status_line().contains("could not verify typed point on selected surface"));
    assert!(app.controller.relative_base().is_none());
    app.transcript.set_draft("");
    submit(&mut app, "2,2,5");
    wait_for(&mut app, &|app| app.transcript.prompt().starts_with("End of line"));
    submit(&mut app, "4,2,5");
    wait_for(&mut app, &|app| !app.controller.is_running());
    let before_open = frame(&mut app, &context, vec![]);
    assert_eq!(snaps_icons(&before_open)[9].2, egui::Color32::WHITE);

    let file_menu = app.file_menu_rect().unwrap().center();
    click(&mut app, &context, file_menu);
    frame(&mut app, &context, vec![]);
    let open = app.file_menu_item_rect(FileKind::Open).unwrap().center();
    click(&mut app, &context, open);
    frame(&mut app, &context, vec![]);
    let discard = app.file_control_rect(FileControl::Discard).unwrap().center();
    click(&mut app, &context, discard);
    wait_for(&mut app, &|app| app.pending_file_picker().is_some());
    let token = app.pending_file_picker().unwrap();
    app.accept_file_picker_reply(&context, token, Some(path.clone())).unwrap();
    wait_for(&mut app, &|app| !app.controller.file_open_worker_pending());
    std::fs::remove_file(path).unwrap();
    let reopened_source = app.controller.document().stamp(target).unwrap();
    assert_ne!(original_source.owner_session, reopened_source.owner_session);
    assert_eq!(original_source.object, reopened_source.object);
    assert!(!app.controller.is_running());
    assert_eq!(app.controller.document().entities.len(), 1);
    let after_open = frame(&mut app, &context, vec![]);
    assert!(snaps_icons(&after_open)[9].2.a() < egui::Color32::WHITE.a());

    let editor = app.command_input_rect().expect("command editor").center();
    click(&mut app, &context, editor);
    for text in ["Line", "2,2,0", "4,2,0"] {
        submit(&mut app, text);
    }
    let line = app.controller.document().entities.values().find_map(|entity| match &entity.geometry {
        Geometry::Curve(Curve::Line { start, end, .. }) => Some((*start, *end)),
        _ => None,
    }).expect("off-surface Line after reopen");
    assert_eq!(line, ([2., 2., 0.], [4., 2., 0.]));
    assert_eq!(app.controller.coordinator().undo_len(), 1);
}

#[test]
fn collapsed_snaps_use_the_expanded_icons_and_follow_cmd_state() {
    let (mut app, context, output) = startup();
    let expanded = snaps_icons(&output);
    assert_eq!(expanded.len(), 16);
    let active = |output: &egui::FullOutput, position: Pos2| {
        output.shapes.iter().any(|shape| {
            matches!(&shape.shape, egui::Shape::Rect(rect)
                if rect.rect.contains(position) && rect.stroke.color == egui::Color32::from_rgb(90, 190, 245))
        })
    };
    // The master switch and four grid steps complete the 21-control palette.
    let header = app.section_rect("snaps").unwrap().center();
    let body_end = app.section_rect("info-settings").unwrap().top();
    let end = expanded[8].0;
    let mid = expanded[7].0;
    let master_button = active_osnap_master_frame(&output, end);
    let master = master_button.center();
    assert!(master_button.width() >= 2. * master_button.height());
    assert!(master_button.contains(Pos2::new(expanded[15].0.x, master.y)));
    let state_marks = |output: &egui::FullOutput| {
        output.shapes.iter().filter_map(|shape| match &shape.shape {
            egui::Shape::LineSegment { points, .. }
                if points.iter().all(|point| master_button.contains(*point)) => Some('I'),
            egui::Shape::Circle(circle) if master_button.contains(circle.center) => Some('O'),
            _ => None,
        }).collect::<Vec<_>>()
    };
    assert_eq!(state_marks(&output), ['I']);
    assert!(active(&output, end) && active(&output, mid));
    assert_eq!(expanded[8].2, egui::Color32::WHITE);
    assert_eq!(expanded[7].2, egui::Color32::WHITE);
    click(&mut app, &context, Pos2::new(master_button.right() - 4., master.y));
    let disabled = frame(&mut app, &context, vec![]);
    assert!(!app.object_snaps_enabled());
    assert!(!active(&disabled, end) && !active(&disabled, mid) && !active(&disabled, master));
    assert_eq!(state_marks(&disabled), ['O']);
    let disabled_icons = snaps_icons(&disabled);
    for index in [7, 8] {
        assert_eq!(disabled_icons[index].1, expanded[index].1);
        assert!(disabled_icons[index].2.a() < expanded[index].2.a(), "inactive snap icon {index} must dim");
    }
    click(&mut app, &context, end);
    assert!(!app.snap_enabled(om9_scene::osnap::SnapKind::End));
    click(&mut app, &context, end);
    assert!(app.snap_enabled(om9_scene::osnap::SnapKind::End));
    click(&mut app, &context, Pos2::new(master_button.left() + 4., master.y));
    let enabled = frame(&mut app, &context, vec![]);
    assert!(app.object_snaps_enabled());
    assert!(active(&enabled, end) && active(&enabled, mid) && active(&enabled, master));
    assert_eq!(state_marks(&enabled), ['I']);
    assert_eq!(snaps_icons(&enabled)[8].2, egui::Color32::WHITE);
    for (label, step) in [("0.1", 0.1), ("0.25", 0.25), ("0.5", 0.5), ("1.0", 1.0)] {
        let position = text_positions(&output, label).into_iter()
            .find(|point| point.y > header.y && point.y < body_end)
            .expect("grid step button");
        click(&mut app, &context, position);
        assert_eq!(app.controller.grid_snap_step_mm(), step);
    }
    click(&mut app, &context, header);
    let output = frame(&mut app, &context, vec![]);
    let collapsed = snaps_icons(&output);
    assert_eq!(collapsed.len(), 4);
    for (index, &expanded_index) in [15, 12, 13, 14].iter().enumerate() {
        assert_eq!(collapsed[index].1, expanded[expanded_index].1, "collapsed icon {index}");
    }

    for theme in [egui::Theme::Light, egui::Theme::Dark] {
        context.style_mut_of(theme, |style| style.interaction.tooltip_delay = 0.0);
    }
    frame(&mut app, &context, vec![Event::PointerMoved(collapsed[0].0)]);
    // Tooltips fade in after the pointer settles and the header click expires.
    let tooltip = (0..12).map(|_| frame(&mut app, &context, vec![])).last().unwrap();
    assert!(!text_positions(&tooltip, "Grid Snap: round pointer input to the 1 mm step").is_empty());

    for (index, command) in ["GridSnap On", "Ortho On", "Planar On", "Project On"].iter().enumerate() {
        let editor = app.command_input_rect().unwrap().center();
        click(&mut app, &context, editor);
        frame(&mut app, &context, vec![Event::Text((*command).into())]);
        frame(&mut app, &context, vec![key(Key::Enter)]);
        let output = frame(&mut app, &context, vec![]);
        let position = snaps_icons(&output)[index].0;
        assert!(active(&output, position), "{command} did not highlight the compact icon");
        assert_eq!(snaps_icons(&output)[index].2, egui::Color32::WHITE);
        click(&mut app, &context, position);
        let output = frame(&mut app, &context, vec![]);
        assert!(!active(&output, position), "{command} icon stayed active after its toggle");
        assert!(snaps_icons(&output)[index].2.a() < 255, "{command} icon did not dim");
    }
    assert!(!app.controller.grid_snap());
    assert!(!app.controller.ortho());
    assert!(!app.controller.planar());
    assert!(!app.controller.project());

    use om9_app::dock_layout::{DockSide, SectionId};
    assert!(app.dock_menu_section(SectionId::Snaps, DockSide::Right));
    frame(&mut app, &context, vec![]);
    assert!(app.dock.section(SectionId::Snaps).collapsed);
    assert_eq!(app.right_dock_icon_rects.len(), 4, "right dock lost compact controls");
    let project = app.right_dock_icon_rects[3].center();
    click(&mut app, &context, project);
    assert!(app.controller.project());
    assert_eq!(app.controller.coordinator().undo_len(), 0);
}

#[test]
fn help_popup_contains_help_and_about_without_feature_catalog_actions() {
    let (mut app, context, output) = startup();
    let help = text_positions(&output, "Help")[0];
    click(&mut app, &context, help);
    let opened = frame(&mut app, &context, vec![]);
    assert!(!text_positions(&opened, "Command help").is_empty());
    assert!(!text_positions(&opened, "About OpenMatrix9").is_empty());
    assert!(text_positions(&opened, "Matrix Interface").is_empty());
    assert!(text_positions(&opened, "Line").is_empty());
    assert!(!app.controller.is_running());
}

#[test]
fn icons_scale_with_the_screen_size() {
    let context = egui::Context::default();
    let mut small = MatrixApp::new(Document::new("small screen")).unwrap();
    frame_at(&mut small, &context, Vec2::new(1280., 720.), Vec::new());
    let mut large = MatrixApp::new(Document::new("large screen")).unwrap();
    frame_at(&mut large, &context, Vec2::new(1600., 1000.), Vec::new());
    assert!(large.screen_icon_scale() > small.screen_icon_scale());
    assert!(small.toolbar_rect.is_none());
    assert!(large.toolbar_rect.is_none());
    assert!(large.left_icon_edge >= small.left_icon_edge);
}

#[test]
fn pinned_menu_lists_have_expected_counts() {
    for (name, expected) in [
        ("Curve", 58),
        ("Surface", 40),
        ("Solid", 43),
        ("Transform", 40),
        ("SubD", 58),
        ("Art", 7),
        ("Builder", 16),
        ("Tools", 31),
        ("Gems", 35),
        ("Setting", 12),
        ("Cutters", 11),
        ("Render", 23),
        ("File", 13),
        ("View", 15),
        ("Utilities", 22),
        ("Measure", 17),
        ("Custom", 0),
    ] {
        assert_eq!(
            om9_app::menu_catalog::category(name).unwrap().len(),
            expected
        );
    }
}

#[test]
fn sphere_variant_icons_start_the_recovered_modes() {
    for (name, prompt) in [
        ("SolidSphere2Points", "First diameter endpoint"),
        ("SolidSphere3Points", "First point"),
        ("SolidSphere4Points", "First point"),
        ("SolidSphereAroundCurve", "Select curve or B-Rep edge"),
        ("SolidSphereTangenttoCurves", "Select tangent curve"),
        ("SolidSphereFitSpheretoPoints", "Add points"),
    ] {
        let (mut app, context, _) = startup();
        let solid = app.main_category_rect(2).unwrap().center();
        click(&mut app, &context, solid);
        frame(&mut app, &context, vec![]);
        solid_icon(&mut app, &context, 0, name, false);
        assert_eq!(app.controller.active_feature(), Some("OM9-SOLID-014"));
        assert!(
            app.controller.prompt().starts_with(prompt),
            "{}",
            app.controller.prompt()
        );
        assert_eq!(app.controller.coordinator().undo_len(), 0);
    }
}

#[test]
fn box_variant_icons_start_the_recovered_modes() {
    for (name, prompt) in [
        ("SolidBoxDiagonal", "First corner"),
        ("SolidBox3PointsHeight", "Start of edge"),
        ("SolidBoxVerticalBase", "Start of edge"),
        ("SolidBoxCenterofBaseCornerHeight", "Center of base"),
    ] {
        let (mut app, context, _) = startup();
        let solid = app.main_category_rect(2).unwrap().center();
        click(&mut app, &context, solid);
        frame(&mut app, &context, vec![]);
        solid_icon(&mut app, &context, 0, name, false);
        assert_eq!(app.controller.active_feature(), Some("OM9-SOLID-012"));
        assert!(
            app.controller.prompt().starts_with(prompt),
            "{}",
            app.controller.prompt()
        );
        frame(&mut app, &context, vec![]);
        assert!(
            context.memory(|memory| memory.has_focus(egui::Id::new("matrix-command-transcript"))),
            "{name} did not focus its command prompt"
        );
        assert_eq!(app.controller.coordinator().undo_len(), 0);
    }
}

#[test]
fn multi_pipe_menu_icon_starts_multiple_selection() {
    let (mut app, context, _) = startup();
    let solid = app.main_category_rect(2).expect("Solid category").center();
    click(&mut app, &context, solid);
    frame(&mut app, &context, vec![]);
    solid_icon(&mut app, &context, 0, "SolidGVDMultiPipe", false);
    assert_eq!(app.controller.active_feature(), Some("OM9-SOLID-011"));
    assert_eq!(app.controller.prompt(), "Select curves for Pipe (Enter when done)");
    assert_eq!(app.controller.coordinator().undo_len(), 0);
}

#[test]
fn create_solid_menu_icon_starts_shared_command() {
    let (mut app, context, _) = startup();
    let solid = app.main_category_rect(2).expect("Solid category").center();
    click(&mut app, &context, solid);
    assert_eq!(app.main_menu_category(), 2);
    solid_icon(&mut app, &context, 0, "SolidCreateSolid", false);
    assert_eq!(app.controller.active_feature(), Some("SolidCreateSolid"));
    assert!(app.controller.document().entities.is_empty());
    assert_eq!(app.controller.coordinator().undo_len(), 0);
}

#[test]
fn boolean_split_menu_icon_starts_shared_command() {
    let (mut app, context, _) = startup();
    let solid = app.main_category_rect(2).expect("Solid category").center();
    click(&mut app, &context, solid);
    solid_icon(&mut app, &context, 0, "SolidBooleanSplit", false);
    assert_eq!(app.controller.active_feature(), Some("SolidBooleanSplit"));
    assert!(app.controller.prompt().contains("first set"));
    assert!(app.controller.document().entities.is_empty());
    assert_eq!(app.controller.coordinator().undo_len(), 0);
}

#[test]
fn truncated_cone_right_click_starts_two_point_variant() {
    let (mut app, context, _) = startup();
    let solid = app.main_category_rect(2).expect("Solid category").center();
    click(&mut app, &context, solid);
    frame(&mut app, &context, vec![]);
    solid_icon(&mut app, &context, 0, "SolidTruncatedCone", true);
    assert_eq!(app.controller.active_feature(), Some("OM9-SOLID-020"));
    assert_eq!(app.controller.prompt(), "First diameter endpoint");
    assert_eq!(app.controller.coordinator().undo_len(), 0);
}

#[test]
fn ellipsoid_right_click_starts_diameter_variant() {
    let (mut app, context, _) = startup();
    let solid = app.main_category_rect(2).expect("Solid category").center();
    click(&mut app, &context, solid);
    frame(&mut app, &context, vec![]);
    solid_icon(&mut app, &context, 0, "SolidEllipsoidFromCenter", true);
    assert_eq!(app.controller.active_feature(), Some("OM9-SOLID-013"));
    assert_eq!(app.controller.prompt(), "First end of first axis");
    assert_eq!(app.controller.coordinator().undo_len(), 0);
}

#[test]
fn torus_right_click_starts_two_point_variant() {
    let (mut app, context, _) = startup();
    let solid = app.main_category_rect(2).expect("Solid category").center();
    click(&mut app, &context, solid);
    frame(&mut app, &context, vec![]);
    solid_icon(&mut app, &context, 0, "SolidTorus", true);
    assert_eq!(app.controller.active_feature(), Some("OM9-SOLID-015"));
    assert_eq!(app.controller.prompt(), "First diameter endpoint");
    assert_eq!(app.controller.coordinator().undo_len(), 0);
}

#[test]
fn cone_right_click_starts_two_point_variant() {
    let (mut app, context, _) = startup();
    let solid = app.main_category_rect(2).expect("Solid category").center();
    click(&mut app, &context, solid);
    frame(&mut app, &context, vec![]);
    solid_icon(&mut app, &context, 0, "SolidCone", true);
    assert_eq!(app.controller.active_feature(), Some("OM9-SOLID-019"));
    assert_eq!(app.controller.prompt(), "First diameter endpoint");
    assert_eq!(app.controller.coordinator().undo_len(), 0);
}

#[test]
fn pyramid_right_click_starts_five_point_star() {
    let (mut app, context, _) = startup();
    let solid = app.main_category_rect(2).expect("Solid category").center();
    click(&mut app, &context, solid);
    frame(&mut app, &context, vec![]);
    solid_icon(&mut app, &context, 0, "SolidPyramid", true);
    assert_eq!(app.controller.active_feature(), Some("OM9-SOLID-018"));
    app.controller.accept_world_point([0., 0., 0.]).unwrap();
    app.controller.submit("5").unwrap();
    assert_eq!(app.controller.prompt(), "Second star radius or point (Automatic)");
    app.controller.submit("2").unwrap();
    app.controller.submit("6").unwrap();
    let e = app.controller.document().entities.values().next().unwrap();
    let om9_core::geometry::Geometry::Brep(body) = &e.geometry else {
        panic!()
    };
    let m = om9_kernel::creation::measure(body, app.controller.coordinator().limits()).unwrap();
    assert!(
        (m.volume_mm3 - 5. * 5. * 2. * (std::f64::consts::PI / 5.).sin() * 2.).abs() < 1e-6
    );
}

#[test]
fn tube_right_click_starts_two_point_variant() {
    let (mut app, context, _) = startup();
    let solid = app.main_category_rect(2).expect("Solid category").center();
    click(&mut app, &context, solid);
    frame(&mut app, &context, vec![]);
    solid_icon(&mut app, &context, 0, "SolidTube", true);
    assert_eq!(app.controller.active_feature(), Some("OM9-SOLID-017"));
    assert_eq!(app.controller.prompt(), "First diameter endpoint");
    assert_eq!(app.controller.coordinator().undo_len(), 0);
}

#[test]
fn cylinder_right_click_starts_two_point_variant() {
    let (mut app, context, _) = startup();
    let solid = app.main_category_rect(2).expect("Solid category").center();
    click(&mut app, &context, solid);
    frame(&mut app, &context, vec![]);
    solid_icon(&mut app, &context, 0, "SolidCylinder", true);
    assert_eq!(app.controller.active_feature(), Some("OM9-SOLID-016"));
    assert_eq!(app.controller.prompt(), "First diameter endpoint");
}

#[test]
fn pipe_right_click_starts_chain_variant() {
    let (mut app, context, _) = startup();
    let solid = app.main_category_rect(2).expect("Solid category").center();
    click(&mut app, &context, solid);
    frame(&mut app, &context, vec![]);
    solid_icon(&mut app, &context, 0, "SolidPipe", true);
    assert_eq!(app.controller.active_feature(), Some("OM9-SOLID-011"));
    assert!(app.controller.prompt().contains("connected curve segments"));
    assert_eq!(app.controller.coordinator().undo_len(), 0);
}

#[test]
fn extract_surface_right_click_starts_copy_variant() {
    use om9_app::command_options::CommandOptionInterface;
    let (mut app, context, _) = startup();
    let solid = app.main_category_rect(2).expect("Solid category").center();
    click(&mut app, &context, solid);
    assert_eq!(app.main_menu_category(), 2);
    frame(&mut app, &context, vec![]);
    frame(&mut app, &context, vec![]);
    solid_icon(&mut app, &context, 0, "SolidExtractSurface", true);
    assert_eq!(app.controller.active_feature(), Some("OM9-SOLID-006"));
    let copy = app
        .controller
        .command_options()
        .into_iter()
        .find(|o| o.id.name() == "Copy")
        .unwrap();
    assert_eq!(copy.label, "Copy=Yes");
    assert_eq!(app.controller.coordinator().undo_len(), 0);
}

#[test]
fn extract_surface_right_click_with_preselection() {
    use om9_app::command_options::CommandOptionInterface;
    let (mut app, context, _) = startup();
    app.controller.submit("Box 10 10 10").unwrap();
    let id = *app.controller.document().entities.keys().next().unwrap();
    app.controller.select_objects(&[id]).unwrap();
    let before = app.controller.document().entities[&id].clone();
    let history_count = app.controller.recent().len();
    let undo_before = app.controller.coordinator().undo_len();
    frame(&mut app, &context, vec![]);
    let solid = app.main_category_rect(2).unwrap().center();
    click(&mut app, &context, solid);
    solid_icon(&mut app, &context, history_count, "SolidExtractSurface", true);
    assert_eq!(app.controller.active_feature(), Some("OM9-SOLID-006"));
    let copy = app
        .controller
        .command_options()
        .into_iter()
        .find(|o| o.id.name() == "Copy")
        .unwrap();
    assert_eq!(copy.label, "Copy=Yes");
    app.controller.submit("0").unwrap();
    assert_eq!(app.controller.document().entities[&id], before);
    assert_eq!(app.controller.document().entities.len(), 2);
    assert_eq!(app.controller.coordinator().undo_len(), undo_before + 1);
}

#[test]
fn top_menu_labels_are_interactive_and_return_to_categories() {
    let (mut app, context, _) = startup();
    let file = app.top_menu_rect(0).expect("File top menu").center();
    click(&mut app, &context, file);
    assert_eq!(app.top_menu_selected(), Some(0));
    let curve = app.main_category_rect(0).expect("Curve category").center();
    click(&mut app, &context, curve);
    assert_eq!(app.top_menu_selected(), None);
    assert!(!app.controller.is_running());
    assert!(app.controller.document().entities.is_empty());
}

#[test]
fn main_menu_categories_wrap_at_the_rail_edge() {
    let (app, _context, _) = startup();
    let sidebar = app.sidebar_rect.expect("sidebar rect");
    let rects: Vec<Rect> = (0..12)
        .map(|i| app.main_category_rect(i).expect("all twelve categories"))
        .collect();
    assert_eq!(rects.len(), 12);
    for rect in &rects {
        assert!(
            rect.max.x <= sidebar.max.x + 0.5,
            "category {rect:?} overflowed the rail edge {sidebar:?}"
        );
    }
    let rows = rects
        .iter()
        .filter(|rect| (rect.center().y - rects[0].center().y).abs() <= 0.5)
        .count();
    assert!(rows >= 2, "only one category fits the first row");
    let distinct_rows = rects
        .iter()
        .map(|rect| rect.center().y.round() as i32)
        .collect::<std::collections::BTreeSet<_>>();
    assert!(
        distinct_rows.len() >= 2,
        "categories did not wrap at the rail edge: {distinct_rows:?}"
    );
}

#[test]
fn main_menu_category_tabs_switch_palette_without_document_mutation() {
    let (mut app, context, _) = startup();
    let surface = app
        .main_category_rect(1)
        .expect("Surface category")
        .center();
    click(&mut app, &context, surface);
    assert_eq!(app.main_menu_category(), 1);
    assert!(!app.controller.is_running());
    assert!(app.controller.document().entities.is_empty());
    assert_eq!(app.controller.coordinator().undo_len(), 0);
    frame(&mut app, &context, vec![Event::Text("x".into())]);
    assert!(!context.memory(|memory| memory.has_focus(egui::Id::new("matrix-command-transcript"))));
    frame(&mut app, &context, vec![key(Key::E), Event::Text("e".into())]);
    assert_eq!(app.snap_override(), Some(om9_scene::osnap::SnapKind::End));
    assert!(app.transcript.draft().is_empty());
}

#[test]
fn unregistered_right_click_slot_never_executes_a_previous_action() {
    let (mut app, context, _) = startup();
    let icon = *app.rail_icon_rects.first().expect("sidebar icon");
    let before = app.controller.coordinator().snapshot();
    secondary_click(&mut app, &context, icon.center());
    assert!(!app.controller.is_running());
    assert!(app.controller.document().entities.is_empty());
    assert_eq!(app.controller.coordinator().snapshot(), before);
    assert_eq!(app.controller.coordinator().undo_len(), 0);
}

#[test]
fn f4_toggles_world_datum_without_mutating_document() {
    let (mut app, context, output) = startup();
    assert!(app.world_datum_visible());
    assert!(datum_strokes(&output) > 0, "F4 datum must paint");
    let before = app.controller.coordinator().snapshot();
    let hidden = frame(&mut app, &context, vec![key(Key::F4)]);
    assert!(!app.world_datum_visible());
    assert_eq!(datum_strokes(&hidden), 0);
    let shown = frame(&mut app, &context, vec![key(Key::F4)]);
    assert!(app.world_datum_visible());
    assert!(datum_strokes(&shown) > 0);
    assert_eq!(app.controller.coordinator().snapshot(), before);
    assert_eq!(app.controller.coordinator().undo_len(), 0);
    assert!(app.view_rects.contains_key(&ViewId::LookingDown));
    assert!(app.view_rects.contains_key(&ViewId::Perspective));
    assert!(app.view_rects.contains_key(&ViewId::SideView));
    assert!(app.view_rects.contains_key(&ViewId::ThroughFinger));
}

#[test]
fn left_main_menu_restores_cmd_focus_and_preserves_numeric_draft() {
    let (mut app, context, _) = startup();
    begin_numeric_draft(&mut app, &context);
    frame(&mut app, &context, vec![]);
    let point = app.main_category_rect(1).expect("Surface category").center();
    click(&mut app, &context, point);
    assert_eq!(app.main_menu_category(), 1);
    assert!(app.toolbar_rect.is_none());
    finish_numeric_draft(&mut app, &context);
}

#[test]
fn view_menu_switches_active_and_maximized_view() {
    let (mut app, context, _) = startup();
    app.controller.workspace.maximized = Some(ViewId::LookingDown);
    app.controller.workspace.active = ViewId::LookingDown;
    let output = frame(&mut app, &context, vec![]);
    let menu = text_positions(&output, "View")
        .into_iter()
        .find(|p| p.y < 25.)
        .unwrap();
    click(&mut app, &context, menu);
    let opened = frame(&mut app, &context, vec![]);
    let point = text_positions(&opened, ViewId::SideView.title())[0];
    click(&mut app, &context, point);
    assert_eq!(app.controller.workspace.active, ViewId::SideView);
    assert_eq!(app.controller.workspace.maximized, Some(ViewId::SideView));
    assert_eq!(
        app.controller.workspace.visible_views(),
        vec![ViewId::SideView]
    );
    assert_eq!(app.controller.coordinator().undo_len(), 0);
    frame(&mut app, &context, vec![Event::Text("x".into())]);
    assert!(!context.memory(|memory| memory.has_focus(egui::Id::new("matrix-command-transcript"))));
    frame(&mut app, &context, vec![key(Key::E), Event::Text("e".into())]);
    assert_eq!(app.snap_override(), Some(om9_scene::osnap::SnapKind::End));
    assert!(app.transcript.draft().is_empty());
}

#[test]
fn view_menu_restores_cmd_focus_and_preserves_numeric_draft() {
    let (mut app, context, _) = startup();
    begin_numeric_draft(&mut app, &context);
    let output = frame(&mut app, &context, vec![]);
    let menu = text_positions(&output, "View")
        .into_iter()
        .find(|p| p.y < 25.)
        .unwrap();
    click(&mut app, &context, menu);
    let opened = frame(&mut app, &context, vec![]);
    let selection = text_positions(&opened, ViewId::SideView.title())
        .into_iter()
        .next()
        .unwrap();
    click(&mut app, &context, selection);
    assert_eq!(app.controller.workspace.active, ViewId::SideView);
    finish_numeric_draft(&mut app, &context);
}

#[test]
fn left_display_palette_does_not_invoke_geometry() {
    let (mut app, context, _) = startup();
    frame(&mut app, &context, vec![]);
    let display = app.section_rect("display").unwrap();
    let snaps = app.section_rect("snaps").unwrap();
    let images: Vec<_> = app
        .rail_icon_rects
        .iter()
        .map(|rect| rect.center())
        .filter(|point| point.y > display.bottom() && point.y < snaps.top())
        .collect();
    assert!(!images.is_empty());
    for position in images {
        click(&mut app, &context, position);
        assert!(
            !app.controller.is_running(),
            "Display palette invoked geometry"
        );
        assert!(app.controller.document().entities.is_empty());
    }
}
#[test]
fn box_command_line_records_the_command_once_without_option_values() {
    let (mut app, context, _) = startup();
    let solid = app.main_category_rect(2).expect("Solid category").center();
    click(&mut app, &context, solid);
    frame(&mut app, &context, vec![]);
    solid_icon(&mut app, &context, 0, "SolidBoxCornertoCornerHeight", false);
    frame(&mut app, &context, vec![]);
    let prompt = app.controller.prompt();
    assert_eq!(
        prompt,
        "First corner of base ( Diagonal 3Point Vertical Center ):"
    );
    // One command echo plus the live prompt; the option values never repeat the prompt.
    assert_eq!(app.transcript.text(), format!("Command: Box\n{prompt} "));
    for name in ["Diagonal", "3Point", "Vertical", "Center"] {
        assert!(
            app.command_option_token_rect(name).is_some(),
            "{name} must be clickable inside the prompt"
        );
    }
    assert!(app.command_option_token_rect("Cube").is_none());
}

#[test]
fn box_typed_stages_match_the_matrix_command_log() {
    let (mut app, context, _) = startup();
    app.focus_command_editor();
    frame(&mut app, &context, vec![]);
    let enter = || egui::Event::Key {
        key: egui::Key::Enter,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    };
    for text in ["box", "0", "10"] {
        frame(&mut app, &context, vec![egui::Event::Text(text.into())]);
        frame(&mut app, &context, vec![enter()]);
    }
    let text = app.transcript.text();
    // Matrix logs the command and one prompt+value line per input; no feature-id line.
    assert!(!text.contains("[OM9-"), "{text}");
    assert!(text.contains("Command: box\n"), "{text}");
    assert!(
        text.contains("First corner of base ( Diagonal 3Point Vertical Center ): 0\n"),
        "{text}"
    );
    assert!(
        text.contains("Other corner of base or length ( 3Point ): 10\n"),
        "{text}"
    );
    assert!(app.transcript.draft().is_empty(), "{text}");
    assert_eq!(
        app.controller.prompt(),
        "Width. Press Enter to use length ( 3Point ):"
    );
}
