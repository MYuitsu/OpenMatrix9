// SPDX-License-Identifier: LGPL-2.1-or-later
//! `OM9-L3-03-04-002/003/004/005`: Project mode, one-shot feedback, SmartTrack state and
//! keyboard snap shortcuts.
use eframe::egui::{self, Event, Key, Modifiers, PointerButton, Pos2, Rect, Vec2};
use om9_app::session::Controller;
use om9_app::ui::MatrixApp;
use om9_core::{budget::Limits, model::Document};
use om9_scene::osnap::SnapKind;
use om9_scene::workspace::ViewId;

fn frame(app: &mut MatrixApp, context: &egui::Context, events: Vec<Event>) {
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 900.))),
            events,
            ..Default::default()
        },
        |ui| app.show(ui),
    );
    output.textures_delta.clear();
}

fn click(app: &mut MatrixApp, context: &egui::Context, at: Pos2) {
    frame(app, context, vec![Event::PointerMoved(at)]);
    frame(
        app,
        context,
        vec![
            Event::PointerMoved(at),
            Event::PointerButton {
                pos: at,
                button: PointerButton::Primary,
                pressed: true,
                modifiers: Modifiers::NONE,
            },
        ],
    );
    frame(
        app,
        context,
        vec![Event::PointerButton {
            pos: at,
            button: PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        }],
    );
}

fn viewport_center(app: &MatrixApp) -> Pos2 {
    app.view_rects[&ViewId::Perspective].center()
}

#[test]
fn project_projects_to_the_active_cplane_without_changing_it() {
    let mut controller = Controller::new(Document::new("project"), Limits::default()).unwrap();
    let plane = controller.input_plane();
    controller.set_project(true);
    assert!(controller.project());
    assert_eq!(
        controller.project_to_plane([12., -4., 7.]).unwrap(),
        [12., -4., 0.]
    );
    assert_eq!(
        controller.constrain_point([12., -4., 7.]).unwrap(),
        [12., -4., 0.]
    );
    assert_eq!(controller.input_plane().origin, plane.origin);
    controller.set_planar(true);
    assert!(controller.planar() && controller.project());
    controller.set_project(false);
    assert!(!controller.project());
    assert!(controller.planar());
}

#[test]
fn panel_updates_enabled_state_and_override_in_the_same_frame() {
    let mut app = MatrixApp::new(Document::new("snap panel")).unwrap();
    assert!(app.snap_enabled(SnapKind::End));
    app.set_snap_enabled(SnapKind::End, false);
    assert!(!app.snap_enabled(SnapKind::End));
    app.set_snap_enabled(SnapKind::End, true);
    assert!(app.snap_enabled(SnapKind::End));
    app.set_snap_override(Some(SnapKind::End));
    assert_eq!(app.snap_override(), Some(SnapKind::End));
    app.set_snap_override(None);
    assert_eq!(app.snap_override(), None);

    assert!(app.smarttrack_enabled());
    app.set_smarttrack_enabled(false);
    assert!(!app.smarttrack_enabled());
    assert!(app.smarttrack_guides().is_empty());
}

#[test]
fn e_and_m_select_end_and_mid_while_a_command_is_running() {
    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::new("snap shortcut")).unwrap();
    frame(&mut app, &context, vec![]);
    frame(&mut app, &context, vec![]);
    app.controller.submit("Line").unwrap();
    let at = viewport_center(&app);
    click(&mut app, &context, at);
    assert!(
        app.controller.is_running(),
        "Line is waiting for its second point"
    );
    assert!(
        !app.command_focus_requested(),
        "viewport click releases the command editor"
    );
    app.set_snap_override(None);
    frame(
        &mut app,
        &context,
        vec![Event::Key {
            key: Key::E,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
    );
    assert_eq!(app.snap_override(), Some(SnapKind::End));
    frame(
        &mut app,
        &context,
        vec![Event::Key {
            key: Key::M,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
    );
    assert_eq!(app.snap_override(), Some(SnapKind::Mid));
    let second = viewport_center(&app) + Vec2::new(40., 20.);
    click(&mut app, &context, second);
    assert_eq!(app.snap_override(), None, "one pick consumes the one-shot");
}

#[test]
fn a_focused_command_editor_does_not_consume_the_snap_shortcut() {
    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::new("snap focus")).unwrap();
    frame(&mut app, &context, vec![]);
    app.controller.submit("Line").unwrap();
    app.focus_command_editor();
    assert!(app.command_focus_requested());
    frame(
        &mut app,
        &context,
        vec![Event::Key {
            key: Key::E,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
    );
    assert_eq!(app.snap_override(), None);
}
