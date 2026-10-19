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

fn text_positions(output: &egui::FullOutput, label: &str) -> Vec<Pos2> {
    output
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
fn begin_numeric_draft(app: &mut MatrixApp, context: &egui::Context) {
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

#[test]
fn snaps_are_noninvoking_even_where_the_old_sidebar_displayed_point_and_line() {
    let (mut app, context, output) = startup();
    let mut in_snaps = false;
    let mut positions = Vec::new();
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
            positions.push(mesh.calc_bounds().center());
        }
    }
    assert!(positions.len() >= 4, "snap icons were not rendered");
    for position in positions {
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
fn main_menu_category_tabs_switch_palette_without_document_mutation() {
    let (mut app, context, _) = startup();
    let gems = app.main_category_rect(8).expect("Gems category").center();
    click(&mut app, &context, gems);
    assert_eq!(app.main_menu_category(), 8);
    assert!(!app.controller.is_running());
    assert!(app.controller.document().entities.is_empty());
    assert_eq!(app.controller.coordinator().undo_len(), 0);
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
fn view_tab_restores_cmd_focus_and_preserves_numeric_draft() {
    let (mut app, context, _) = startup();
    begin_numeric_draft(&mut app, &context);
    let output = frame(&mut app, &context, vec![]);
    let toolbar = app.toolbar_rect.unwrap();
    let point = text_positions(&output, ViewId::SideView.title())
        .into_iter()
        .find(|p| toolbar.contains(*p))
        .unwrap();
    click(&mut app, &context, point);
    assert_eq!(app.controller.workspace.active, ViewId::SideView);
    finish_numeric_draft(&mut app, &context);
}

#[test]
fn view_tab_switches_active_and_maximized_view() {
    let (mut app, context, _) = startup();
    app.controller.workspace.maximized = Some(ViewId::LookingDown);
    app.controller.workspace.active = ViewId::LookingDown;
    let output = frame(&mut app, &context, vec![]);
    let toolbar = app.toolbar_rect.unwrap();
    let point = text_positions(&output, ViewId::SideView.title())
        .into_iter()
        .find(|p| toolbar.contains(*p))
        .unwrap();
    click(&mut app, &context, point);
    assert_eq!(app.controller.workspace.active, ViewId::SideView);
    assert_eq!(app.controller.workspace.maximized, Some(ViewId::SideView));
    assert_eq!(
        app.controller.workspace.visible_views(),
        vec![ViewId::SideView]
    );
    assert_eq!(app.controller.coordinator().undo_len(), 0);
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
    let toolbar_top = app.toolbar_rect.unwrap().top();
    let selection = text_positions(&opened, ViewId::SideView.title())
        .into_iter()
        .find(|p| p.y < toolbar_top)
        .unwrap();
    click(&mut app, &context, selection);
    assert_eq!(app.controller.workspace.active, ViewId::SideView);
    finish_numeric_draft(&mut app, &context);
}

#[test]
fn display_toolbar_tab_does_not_reuse_enabled_geometry_palette() {
    let (mut app, context, output) = startup();
    let display = text_positions(&output, "Display")[0];
    click(&mut app, &context, display);
    let output = frame(&mut app, &context, vec![]);
    let toolbar = app.toolbar_rect.unwrap();
    let images: Vec<_> = output
        .shapes
        .iter()
        .filter_map(|s| {
            if let egui::Shape::Mesh(mesh) = &s.shape {
                let center = mesh.calc_bounds().center();
                (mesh.texture_id != egui::TextureId::default()
                    && toolbar.contains(center)
                    && s.clip_rect.contains(center))
                .then_some(center)
            } else {
                None
            }
        })
        .collect();
    assert!(!images.is_empty());
    for position in images {
        click(&mut app, &context, position);
        assert!(
            !app.controller.is_running(),
            "Display toolbar invoked geometry"
        );
        assert!(app.controller.document().entities.is_empty());
    }
}
