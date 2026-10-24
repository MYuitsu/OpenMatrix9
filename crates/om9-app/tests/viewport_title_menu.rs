// SPDX-License-Identifier: LGPL-2.1-or-later
use eframe::egui::{self, Event, Modifiers, Pos2, Rect, Vec2};
use om9_app::ui::MatrixApp;
use om9_app::{
    session::Controller,
    viewport_menu::{DisplayPreset, MenuAction},
};
use om9_core::model::Document;
use om9_core::{Error, budget::Limits};
use om9_scene::workspace::{DisplayMode, ViewId};

fn frame(app: &mut MatrixApp, ctx: &egui::Context, events: Vec<Event>) -> egui::FullOutput {
    frame_size(app, ctx, events, Vec2::new(1280., 900.))
}
fn frame_size(
    app: &mut MatrixApp,
    ctx: &egui::Context,
    events: Vec<Event>,
    size: Vec2,
) -> egui::FullOutput {
    let mut out = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, size)),
            events,
            ..Default::default()
        },
        |ui| app.show(ui),
    );
    out.textures_delta.clear();
    out
}
fn click(app: &mut MatrixApp, ctx: &egui::Context, pos: Pos2) {
    for pressed in [true, false] {
        frame(
            app,
            ctx,
            vec![
                Event::PointerMoved(pos),
                Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Modifiers::NONE,
                },
            ],
        );
    }
}
fn labels(output: &egui::FullOutput) -> Vec<String> {
    output
        .shapes
        .iter()
        .filter_map(|shape| {
            if let egui::Shape::Text(t) = &shape.shape {
                Some(t.galley.job.text.clone())
            } else {
                None
            }
        })
        .collect()
}
#[test]
fn title_popup_has_reference_actions_instead_of_the_old_three_entry_stub() {
    let mut app = MatrixApp::new(Document::new("title menu source fixture")).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![]);
    let before = app.controller.coordinator().snapshot();
    let rect = app.view_rects[&ViewId::LookingDown];
    // Title text is about78logical pixels wide; its separate arrow is next.
    let title = frame(&mut app, &ctx, vec![])
        .shapes
        .into_iter()
        .find_map(|s| {
            if let egui::Shape::Text(t) = s.shape {
                (t.galley.job.text == "Looking Down"
                    && rect.contains(t.visual_bounding_rect().center()))
                .then(|| t.visual_bounding_rect())
            } else {
                None
            }
        })
        .unwrap();
    click(
        &mut app,
        &ctx,
        Pos2::new(title.right() + 8., rect.top() + 9.),
    );
    // Let the native popup's fade reach full opacity before contrast checks.
    for _ in 0..20 {
        frame(&mut app, &ctx, vec![]);
    }
    let output = frame(&mut app, &ctx, vec![]);
    let menu = labels(&output);
    assert!(
        menu.iter().any(|s| s == "Maximize"),
        "S6 Maximize label missing: {menu:?}"
    );
    let first = menu.iter().position(|s| s == "Maximize").unwrap();
    assert_eq!(
        &menu[first..first + 8],
        &[
            "Maximize",
            "Wireframe",
            "Shaded",
            "Rendered",
            "Ghosted",
            "X-Ray",
            "Technical",
            "Art Color"
        ]
    );
    assert_eq!(app.controller.workspace.active, ViewId::LookingDown);
    assert_eq!(app.controller.coordinator().snapshot(), before);
    let maximize = output
        .shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            egui::Shape::Text(text) if text.galley.job.text == "Maximize" => Some(text),
            _ => None,
        })
        .unwrap();
    let label_color = maximize
        .override_text_color
        .unwrap_or(maximize.fallback_color);
    assert!(label_color.r() < 64 && label_color.g() < 64 && label_color.b() < 64);
    fn light_menu(shape: &egui::Shape, point: Pos2) -> bool {
        match shape {
            egui::Shape::Vec(shapes) => shapes.iter().any(|shape| light_menu(shape, point)),
            egui::Shape::Rect(rect) => {
                rect.rect.contains(point)
                    && rect.rect.width() >= 230.
                    && rect.fill.r() > 220
                    && rect.fill.g() > 220
                    && rect.fill.b() > 220
            }
            _ => false,
        }
    }
    assert!(
        output
            .shapes
            .iter()
            .any(|shape| light_menu(&shape.shape, maximize.visual_bounding_rect().center())),
        "S6 title menu must use a light background with dark labels"
    );
}

#[test]
fn captured_maximize_preserves_model_redo_pending_point_and_all_camera_poses() {
    let mut c = Controller::new(Document::new("captured menu"), Limits::default()).unwrap();
    c.submit("Point").unwrap();
    c.submit("1,2,3").unwrap();
    c.undo().unwrap();
    c.submit("Line").unwrap();
    c.submit("2,3,4").unwrap();
    let before = c.coordinator().snapshot();
    let views = c.workspace.views.clone();
    let menu = c.capture_view_menu(ViewId::LookingDown).unwrap();
    c.apply_view_menu(&menu, MenuAction::Maximize).unwrap();
    assert_eq!(c.workspace.maximized, Some(ViewId::LookingDown));
    assert_eq!(c.workspace.active, ViewId::LookingDown);
    for (id, view) in &views {
        assert_eq!(c.workspace.views[id].camera, view.camera);
        assert_eq!(c.workspace.views[id].cplane, view.cplane);
        assert_eq!(c.workspace.views[id].mode, view.mode);
    }
    assert_eq!(c.coordinator().snapshot(), before);
    assert!(c.is_running());
    assert_eq!(
        c.preview([6., 7., 8.]).unwrap(),
        Some(([2., 3., 4.], [6., 7., 8.]))
    );
    let fresh = c.capture_view_menu(ViewId::LookingDown).unwrap();
    c.apply_view_menu(&fresh, MenuAction::Maximize).unwrap();
    assert_eq!(c.workspace.maximized, None);
    assert_eq!(c.coordinator().snapshot(), before);
    c.cancel();
    c.redo().unwrap();
    assert_eq!(c.document().entities.len(), 1);
}

#[test]
fn render_plugin_and_unimplemented_modes_refuse_without_changing_any_view_or_model() {
    let mut c = Controller::new(Document::new("mode gate"), Limits::default()).unwrap();
    let before = c.coordinator().snapshot();
    let workspace = c.workspace.clone();
    let menu = c.capture_view_menu(ViewId::SideView).unwrap();
    for mode in [DisplayPreset::Rendered, DisplayPreset::ClayooSculpt] {
        assert!(matches!(
            c.apply_view_menu(&menu, MenuAction::Preset(mode)),
            Err(Error::Unsupported(_))
        ));
        assert_eq!(c.workspace, workspace);
        assert_eq!(c.coordinator().snapshot(), before);
    }
}

#[test]
fn foreign_closed_and_replaced_view_capture_rejects_even_with_equal_visible_state() {
    let d = Document::new("view owner");
    let mut first = Controller::new(d.clone(), Limits::default()).unwrap();
    let menu = first.capture_view_menu(ViewId::SideView).unwrap();
    let mut second = Controller::new(d, Limits::default()).unwrap();
    assert!(second.apply_view_menu(&menu, MenuAction::Maximize).is_err());
    let old = first.workspace.views.remove(&ViewId::SideView).unwrap();
    assert!(first.apply_view_menu(&menu, MenuAction::Maximize).is_err());
    first.workspace.views.insert(
        ViewId::SideView,
        om9_scene::workspace::ViewState::new(old.id, old.camera, old.cplane),
    );
    let before = first.workspace.clone();
    assert!(first.apply_view_menu(&menu, MenuAction::Maximize).is_err());
    assert_eq!(first.workspace, before);
}

#[test]
fn changed_document_or_target_view_invalidates_popup_without_resetting_prior_state() {
    let mut c = Controller::new(Document::new("menu stale"), Limits::default()).unwrap();
    let menu = c.capture_view_menu(ViewId::LookingDown).unwrap();
    c.submit("Point").unwrap();
    c.submit("1,2,3").unwrap();
    let before = c.coordinator().snapshot();
    assert!(c.apply_view_menu(&menu, MenuAction::Maximize).is_err());
    assert_eq!(c.coordinator().snapshot(), before);
    let menu = c.capture_view_menu(ViewId::LookingDown).unwrap();
    c.workspace
        .views
        .get_mut(&ViewId::LookingDown)
        .unwrap()
        .camera
        .zoom(1.5)
        .unwrap();
    let views = c.workspace.clone();
    assert!(c.apply_view_menu(&menu, MenuAction::Maximize).is_err());
    assert_eq!(c.workspace, views);
}

#[test]
fn wireframe_changes_only_captured_view_and_no_op_preserves_revision() {
    use om9_scene::workspace::DisplayMode;
    let mut c =
        Controller::new(Document::new("independent view modes"), Limits::default()).unwrap();
    c.workspace
        .views
        .get_mut(&ViewId::LookingDown)
        .unwrap()
        .mode = DisplayMode::Shaded;
    c.workspace
        .views
        .get_mut(&ViewId::Perspective)
        .unwrap()
        .mode = DisplayMode::Shaded;
    let before = c.coordinator().snapshot();
    let menu = c.capture_view_menu(ViewId::LookingDown).unwrap();
    let neighbor = c.workspace.views[&ViewId::Perspective].clone();
    c.apply_view_menu(&menu, MenuAction::Preset(DisplayPreset::Wireframe))
        .unwrap();
    assert_eq!(
        c.workspace.views[&ViewId::LookingDown].mode,
        DisplayMode::Wireframe
    );
    assert_eq!(c.workspace.views[&ViewId::Perspective], neighbor);
    let menu = c.capture_view_menu(ViewId::LookingDown).unwrap();
    let workspace = c.workspace.clone();
    c.apply_view_menu(&menu, MenuAction::Preset(DisplayPreset::Wireframe))
        .unwrap();
    assert_eq!(c.workspace, workspace);
    assert_eq!(c.coordinator().snapshot(), before);
}

#[test]
fn wireframe_with_exhausted_revision_refuses_before_any_layout_or_camera_change() {
    let mut c =
        Controller::new(Document::new("view revision boundary"), Limits::default()).unwrap();
    c.workspace
        .views
        .get_mut(&ViewId::LookingDown)
        .unwrap()
        .revision = u64::MAX;
    c.workspace
        .views
        .get_mut(&ViewId::LookingDown)
        .unwrap()
        .mode = om9_scene::workspace::DisplayMode::Shaded;
    let menu = c.capture_view_menu(ViewId::LookingDown).unwrap();
    let before = c.workspace.clone();
    assert!(matches!(
        c.apply_view_menu(&menu, MenuAction::Preset(DisplayPreset::Wireframe)),
        Err(Error::Overflow(_))
    ));
    assert_eq!(c.workspace, before);
}

#[test]
fn escape_closes_title_popup_and_preserves_numeric_draft_and_pending_command() {
    use eframe::egui::Key;
    let mut app = MatrixApp::new(Document::new("popup escape")).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![]);
    app.controller.submit("Line").unwrap();
    app.transcript.set_draft("0,0,");
    let output = frame(&mut app, &ctx, vec![]);
    let rect = app.view_rects[&ViewId::LookingDown];
    let title = output
        .shapes
        .into_iter()
        .find_map(|s| {
            if let egui::Shape::Text(t) = s.shape {
                (t.galley.job.text == "Looking Down"
                    && rect.contains(t.visual_bounding_rect().center()))
                .then(|| t.visual_bounding_rect())
            } else {
                None
            }
        })
        .unwrap();
    click(
        &mut app,
        &ctx,
        Pos2::new(title.right() + 8., rect.top() + 9.),
    );
    frame(&mut app, &ctx, vec![]);
    assert!(egui::Popup::is_any_open(&ctx));
    frame(
        &mut app,
        &ctx,
        vec![Event::Key {
            key: Key::Escape,
            physical_key: Some(Key::Escape),
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
    );
    assert!(!egui::Popup::is_any_open(&ctx));
    assert!(app.controller.is_running());
    assert_eq!(app.transcript.draft(), "0,0,");
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![Event::Text("0".into())]);
    assert_eq!(app.transcript.draft(), "0,0,0");
}

#[test]
fn long_title_menu_scrolls_to_last_reference_item_in_a_short_window() {
    let size = Vec2::new(1280., 500.);
    let mut app = MatrixApp::new(Document::new("small viewport menu")).unwrap();
    let ctx = egui::Context::default();
    frame_size(&mut app, &ctx, vec![], size);
    let output = frame_size(&mut app, &ctx, vec![], size);
    let rect = app.view_rects[&ViewId::LookingDown];
    let title = output
        .shapes
        .into_iter()
        .find_map(|s| {
            if let egui::Shape::Text(t) = s.shape {
                (t.galley.job.text == "Looking Down"
                    && rect.contains(t.visual_bounding_rect().center()))
                .then(|| t.visual_bounding_rect())
            } else {
                None
            }
        })
        .unwrap();
    let point = Pos2::new(title.right() + 8., rect.top() + 9.);
    for pressed in [true, false] {
        frame_size(
            &mut app,
            &ctx,
            vec![
                Event::PointerMoved(point),
                Event::PointerButton {
                    pos: point,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Modifiers::NONE,
                },
            ],
            size,
        );
    }
    let opened = frame_size(&mut app, &ctx, vec![], size);
    let maximum = opened
        .shapes
        .iter()
        .find_map(|s| {
            if let egui::Shape::Text(t) = &s.shape {
                (t.galley.job.text == "Maximize").then(|| t.visual_bounding_rect().center())
            } else {
                None
            }
        })
        .unwrap();
    for _ in 0..4 {
        frame_size(
            &mut app,
            &ctx,
            vec![
                Event::PointerMoved(maximum),
                Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: Vec2::new(0., -2000.),
                    phase: egui::TouchPhase::Move,
                    modifiers: Modifiers::NONE,
                },
            ],
            size,
        );
    }
    let end = frame_size(&mut app, &ctx, vec![], size);
    let last = end
        .shapes
        .iter()
        .find_map(|s| {
            if let egui::Shape::Text(t) = &s.shape {
                (t.galley.job.text == "Viewport Properties..."
                    && s.clip_rect.intersects(t.visual_bounding_rect()))
                .then(|| t.visual_bounding_rect())
            } else {
                None
            }
        })
        .expect("last S6 menu action clipped/unreachable");
    assert!(last.top() >= 0. && last.bottom() <= size.y);
    assert!(app.controller.document().entities.is_empty());
}

#[test]
fn mismatched_view_key_cannot_redirect_menu_to_a_neighbor() {
    let mut c = Controller::new(Document::new("view table alias"), Limits::default()).unwrap();
    let neighbor = c.workspace.views[&ViewId::LookingDown].clone();
    c.workspace.views.insert(ViewId::SideView, neighbor);
    let before = c.workspace.clone();
    assert!(c.capture_view_menu(ViewId::SideView).is_err());
    assert_eq!(c.workspace, before);
    assert!(c.workspace.toggle_maximize(ViewId::SideView).is_err());
    assert_eq!(c.workspace, before);
}

#[test]
fn layout_roundtrip_keeps_views_exact_but_does_not_revive_a_previous_capture() {
    let mut c = Controller::new(Document::new("layout sequence"), Limits::default()).unwrap();
    let menu = c.capture_view_menu(ViewId::LookingDown).unwrap();
    let views = c.workspace.views.clone();
    c.workspace.toggle_maximize(ViewId::LookingDown).unwrap();
    c.workspace.toggle_maximize(ViewId::LookingDown).unwrap();
    assert_eq!(c.workspace.views, views);
    assert_eq!(c.workspace.maximized, None);
    let before = c.workspace.clone();
    assert!(c.apply_view_menu(&menu, MenuAction::Maximize).is_err());
    assert_eq!(c.workspace, before);
}

#[test]
fn workspace_snapshot_restore_cannot_revive_a_used_menu() {
    let mut c = Controller::new(Document::new("snapshot rollback"), Limits::default()).unwrap();
    let old = c.workspace.clone();
    let menu = c.capture_view_menu(ViewId::LookingDown).unwrap();
    c.workspace.toggle_maximize(ViewId::LookingDown).unwrap();
    c.workspace = old;
    assert!(c.apply_view_menu(&menu, MenuAction::Maximize).is_err());
    c.workspace
        .views
        .get_mut(&ViewId::LookingDown)
        .unwrap()
        .mode = om9_scene::workspace::DisplayMode::Shaded;
    let old = c.workspace.clone();
    let menu = c.capture_view_menu(ViewId::LookingDown).unwrap();
    c.apply_view_menu(&menu, MenuAction::Preset(DisplayPreset::Wireframe))
        .unwrap();
    c.workspace = old;
    assert!(
        c.apply_view_menu(&menu, MenuAction::Preset(DisplayPreset::Wireframe))
            .is_err()
    );
}

#[test]
fn fresh_layout_clock_with_copied_view_states_cannot_reset_menu_authority() {
    let mut c = Controller::new(Document::new("new layout owner"), Limits::default()).unwrap();
    let old = c.workspace.clone();
    let menu = c.capture_view_menu(ViewId::LookingDown).unwrap();
    c.workspace.toggle_maximize(ViewId::LookingDown).unwrap();
    let mut fresh = om9_scene::workspace::Workspace::new();
    fresh.views = old.views;
    fresh.active = old.active;
    fresh.maximized = old.maximized;
    c.workspace = fresh;
    assert!(c.apply_view_menu(&menu, MenuAction::Maximize).is_err());
}

#[test]
fn popup_keyboard_cannot_commit_command_after_tab_reaches_editor() {
    let mut app = MatrixApp::new(Document::new("popup keyboard ownership")).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![]);
    app.controller.submit("Point").unwrap();
    app.transcript.set_draft("1,2,3");
    let output = frame(&mut app, &ctx, vec![]);
    let rect = app.view_rects[&ViewId::LookingDown];
    let title = output
        .shapes
        .into_iter()
        .find_map(|s| {
            if let egui::Shape::Text(t) = s.shape {
                (t.galley.job.text == "Looking Down"
                    && rect.contains(t.visual_bounding_rect().center()))
                .then(|| t.visual_bounding_rect())
            } else {
                None
            }
        })
        .unwrap();
    click(
        &mut app,
        &ctx,
        Pos2::new(title.right() + 8., rect.top() + 9.),
    );
    let before = app.controller.coordinator().snapshot();
    for _ in 0..80 {
        frame(
            &mut app,
            &ctx,
            vec![Event::Key {
                key: egui::Key::Tab,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
        );
        if ctx.memory(|m| m.has_focus(egui::Id::new("matrix-command-transcript"))) {
            frame(
                &mut app,
                &ctx,
                vec![Event::Key {
                    key: egui::Key::Enter,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: Modifiers::NONE,
                }],
            );
        }
        assert_eq!(app.controller.coordinator().snapshot(), before);
        assert_eq!(app.transcript.draft(), "1,2,3");
        assert!(app.controller.is_running());
    }
}

#[test]
fn replacing_target_view_dismisses_popup_before_command_input_resumes() {
    let mut app = MatrixApp::new(Document::new("popup target closed")).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![]);
    app.controller.submit("Line").unwrap();
    app.transcript.set_draft("0,0,");
    let output = frame(&mut app, &ctx, vec![]);
    let rect = app.view_rects[&ViewId::LookingDown];
    let title = output
        .shapes
        .into_iter()
        .find_map(|s| {
            if let egui::Shape::Text(t) = s.shape {
                (t.galley.job.text == "Looking Down"
                    && rect.contains(t.visual_bounding_rect().center()))
                .then(|| t.visual_bounding_rect())
            } else {
                None
            }
        })
        .unwrap();
    click(
        &mut app,
        &ctx,
        Pos2::new(title.right() + 8., rect.top() + 9.),
    );
    assert!(egui::Popup::is_any_open(&ctx));
    let old = app
        .controller
        .workspace
        .views
        .remove(&ViewId::LookingDown)
        .unwrap();
    app.controller.workspace.views.insert(
        ViewId::LookingDown,
        om9_scene::workspace::ViewState::new(old.id, old.camera, old.cplane),
    );
    frame(
        &mut app,
        &ctx,
        vec![Event::Key {
            key: egui::Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
    );
    assert!(!egui::Popup::is_any_open(&ctx));
    assert!(app.controller.document().entities.is_empty());
    assert!(app.controller.is_running());
    assert_eq!(app.transcript.draft(), "0,0,");
}

#[test]
fn keyboard_enter_activates_focused_menu_item_and_returns_to_command_draft() {
    let mut app = MatrixApp::new(Document::new("keyboard menu activation")).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![]);
    app.controller.submit("Point").unwrap();
    app.transcript.set_draft("1,2,");
    let output = frame(&mut app, &ctx, vec![]);
    let rect = app.view_rects[&ViewId::LookingDown];
    let title = output
        .shapes
        .into_iter()
        .find_map(|s| {
            if let egui::Shape::Text(t) = s.shape {
                (t.galley.job.text == "Looking Down"
                    && rect.contains(t.visual_bounding_rect().center()))
                .then(|| t.visual_bounding_rect())
            } else {
                None
            }
        })
        .unwrap();
    click(
        &mut app,
        &ctx,
        Pos2::new(title.right() + 8., rect.top() + 9.),
    );
    frame(&mut app, &ctx, vec![]);
    frame(
        &mut app,
        &ctx,
        vec![Event::Key {
            key: egui::Key::Enter,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
    );
    assert_eq!(
        app.controller.workspace.maximized,
        Some(ViewId::LookingDown)
    );
    assert!(!egui::Popup::is_any_open(&ctx));
    assert!(app.controller.is_running());
    assert_eq!(app.transcript.draft(), "1,2,");
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![Event::Text("3".into())]);
    assert_eq!(app.transcript.draft(), "1,2,3");
    assert!(app.controller.document().entities.is_empty());
}

#[test]
fn outside_canvas_click_only_dismisses_popup_and_preserves_pending_point() {
    let mut app = MatrixApp::new(Document::new("popup dismissal click")).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![]);
    app.controller.submit("Point").unwrap();
    app.transcript.set_draft("1,2,3");
    let output = frame(&mut app, &ctx, vec![]);
    let rect = app.view_rects[&ViewId::LookingDown];
    let title = output
        .shapes
        .into_iter()
        .find_map(|s| {
            if let egui::Shape::Text(t) = s.shape {
                (t.galley.job.text == "Looking Down"
                    && rect.contains(t.visual_bounding_rect().center()))
                .then(|| t.visual_bounding_rect())
            } else {
                None
            }
        })
        .unwrap();
    click(
        &mut app,
        &ctx,
        Pos2::new(title.right() + 8., rect.top() + 9.),
    );
    let before = app.controller.coordinator().snapshot();
    let outside = app.view_rects[&ViewId::Perspective].center();
    click(&mut app, &ctx, outside);
    assert!(!egui::Popup::is_any_open(&ctx));
    assert_eq!(app.controller.coordinator().snapshot(), before);
    assert!(app.controller.is_running());
    assert_eq!(app.transcript.draft(), "1,2,3");
    click(&mut app, &ctx, outside);
    assert_eq!(app.controller.document().entities.len(), 1);
    assert!(!app.controller.is_running());
}

#[test]
fn shaded_switches_only_the_view_mode_and_keeps_the_model_and_journal() {
    let mut c = Controller::new(Document::new("shaded mode"), Limits::default()).unwrap();
    let menu = c.capture_view_menu(ViewId::Perspective).unwrap();
    let before = c.coordinator().snapshot();
    let entities = c.document().entities.clone();
    c.apply_view_menu(&menu, MenuAction::Preset(DisplayPreset::Shaded))
        .unwrap();
    assert_eq!(
        c.workspace.views[&ViewId::Perspective].mode,
        DisplayMode::Shaded
    );
    assert_eq!(c.document().entities, entities);
    assert_eq!(c.coordinator().snapshot(), before);
    let menu = c.capture_view_menu(ViewId::Perspective).unwrap();
    c.apply_view_menu(&menu, MenuAction::Preset(DisplayPreset::Wireframe))
        .unwrap();
    assert_eq!(
        c.workspace.views[&ViewId::Perspective].mode,
        DisplayMode::Wireframe
    );
}
