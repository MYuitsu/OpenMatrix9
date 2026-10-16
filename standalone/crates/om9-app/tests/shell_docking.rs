// SPDX-License-Identifier: LGPL-2.1-or-later
//! Shell docking evidence for plan module OM9-L2-01-01 (L3-002/003/004/005/007/008).
//!
//! The documented layout comes from the module's own reference, Matrix8 book 1
//! "A Tour of the Interface": the left column is divided into Icon History, Main
//! Menu, Display, Snaps, Info & Settings, Layers and Projects, builders open in
//! the space below the Project Manager, menus can be rolled up, reordered,
//! floated or docked to the right ~300-pixel mirror column, and the
//! configuration is kept with Set as Default / Reset Menus / Reload Menus.
//!
//! These tests drive the real `MatrixApp::show` frame the same way the other UI
//! tests do: no mock shell, no synthetic document mutation.
use eframe::egui::{self, Event, Key, Modifiers, Pos2, Rect, Vec2};
use om9_app::dock_layout::{DockLayout, DockSide, FloatRect, PanelId, RIGHT_DOCK_MIN, SectionId};
use om9_app::status_surface::StatusKind;
use om9_app::ui::MatrixApp;
use om9_core::model::Document;
use om9_scene::workspace::ViewId;

const DOCUMENTED_LEFT: [SectionId; 8] = [
    SectionId::IconHistory,
    SectionId::MainMenu,
    SectionId::Display,
    SectionId::Snaps,
    SectionId::InfoSettings,
    SectionId::Layers,
    SectionId::Projects,
    SectionId::BuilderSlot,
];

fn frame_size(
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

fn frame(app: &mut MatrixApp, context: &egui::Context, events: Vec<Event>) -> egui::FullOutput {
    frame_size(app, context, Vec2::new(1280., 900.), events)
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

fn canvas_rect(app: &MatrixApp) -> Rect {
    let mut bounds: Option<Rect> = None;
    for rect in app.view_rects.values() {
        bounds = Some(match bounds {
            Some(current) => current.union(*rect),
            None => *rect,
        });
    }
    bounds.expect("four viewport rectangles")
}

fn owned_layout_path(case: &str) -> std::path::PathBuf {
    let directory =
        std::env::temp_dir().join(format!("om9-shell-docking-{case}-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    directory.join("ui-layout.json")
}

fn assert_near(actual: f32, expected: f32, tolerance: f32) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "expected {expected} ± {tolerance}, got {actual}"
    );
}

fn startup(path: Option<std::path::PathBuf>) -> (MatrixApp, egui::Context) {
    let mut app = MatrixApp::new(Document::new("shell docking")).unwrap();
    if let Some(path) = path {
        app.set_dock_layout_path(Some(path));
        app.load_dock_layout();
    }
    let context = egui::Context::default();
    frame(&mut app, &context, vec![]);
    frame(&mut app, &context, vec![]);
    (app, context)
}

#[test]
fn left_column_renders_the_documented_sections_with_builders_below_projects() {
    let (app, _context) = startup(None);
    assert_eq!(app.dock.left_sections(), DOCUMENTED_LEFT.to_vec());
    assert!(app.dock.right_sections().is_empty());
    assert!(
        app.right_dock_rect.is_none(),
        "no menu is docked to the right"
    );
    assert!(
        app.status_rect.is_some(),
        "status surface is always visible"
    );
    assert_eq!(app.view_rects.len(), 4);

    for id in DOCUMENTED_LEFT {
        assert!(
            app.section_rect(id.key()).is_some(),
            "{} title bar must render",
            id.title()
        );
    }
    let projects = app.section_rect(SectionId::Projects.key()).unwrap();
    let builders = app
        .builder_slot_rect
        .expect("the reserved Builder area renders in the left column");
    assert!(
        builders.min.y > projects.min.y,
        "builders open in the space below the Project Manager"
    );
    assert!(SectionId::BuilderSlot.is_reserved());
    assert!(!SectionId::BuilderSlot.keeps_controls_while_collapsed());
    assert!(SectionId::Snaps.keeps_controls_while_collapsed());
}

#[test]
fn rolling_up_a_title_bar_keeps_its_state_and_persists() {
    let path = owned_layout_path("rollup");
    let _ = std::fs::remove_file(&path);
    {
        let (mut app, context) = startup(Some(path.clone()));
        let bar = app
            .section_rect(SectionId::Projects.key())
            .expect("projects title bar");
        click(&mut app, &context, bar.center());
        assert!(app.dock.section(SectionId::Projects).collapsed);
        assert_eq!(
            app.dock.left_sections(),
            DOCUMENTED_LEFT.to_vec(),
            "rolling up must not move the menu"
        );
        app.set_menus_as_default();
        assert!(path.exists(), "Set as Default writes the configuration");
    }

    let (app, _context) = startup(Some(path.clone()));
    assert!(
        app.dock.section(SectionId::Projects).collapsed,
        "the rolled-up state survives the session"
    );
    std::fs::remove_file(&path).unwrap();
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn menus_dock_to_the_right_mirror_column_and_back() {
    let (mut app, context) = startup(None);
    assert!(app.dock_menu_section(SectionId::Snaps, DockSide::Right));
    frame(&mut app, &context, vec![]);
    assert_eq!(app.dock.right_sections(), vec![SectionId::Snaps]);
    assert!(!app.dock.left_sections().contains(&SectionId::Snaps));
    let dock = app.right_dock_rect.expect("right menu column renders");
    assert!(dock.width() >= RIGHT_DOCK_MIN);
    let bar = app
        .section_rect(SectionId::Snaps.key())
        .expect("docked menu bar");
    assert!(dock.contains(bar.center()));
    assert_eq!(
        app.view_rects.len(),
        4,
        "docking must not remove a viewport"
    );

    assert!(app.dock_menu_section(SectionId::Snaps, DockSide::Left));
    frame(&mut app, &context, vec![]);
    assert!(app.right_dock_rect.is_none());
    // Re-docking appends the menu, and the documented Projects/Builders order
    // stays adjacent.
    let left = app.dock.left_sections();
    assert_eq!(left.len(), DOCUMENTED_LEFT.len());
    assert!(left.contains(&SectionId::Snaps));
    let projects = left
        .iter()
        .position(|id| *id == SectionId::Projects)
        .unwrap();
    assert_eq!(left[projects + 1], SectionId::BuilderSlot);
}

#[test]
fn a_floated_menu_keeps_its_rectangle_and_docks_back() {
    let (mut app, context) = startup(None);
    assert!(app.float_menu_section(
        SectionId::Display,
        FloatRect {
            x: 600.,
            y: 300.,
            width: 320.,
            height: 240.,
        }
    ));
    frame(&mut app, &context, vec![]);
    assert_eq!(app.dock.floating_sections(), vec![SectionId::Display]);
    let stored = app.dock.section(SectionId::Display).float.unwrap();
    assert_near(stored.x, 600., 24.);
    assert_near(stored.width, 320., 24.);
    assert!(!app.dock.left_sections().contains(&SectionId::Display));

    assert!(app.dock_menu_section(SectionId::Display, DockSide::Left));
    frame(&mut app, &context, vec![]);
    assert!(app.dock.floating_sections().is_empty());
    assert!(app.dock.left_sections().contains(&SectionId::Display));
}

#[test]
fn narrow_window_yields_the_right_column_and_keeps_command_line_and_canvas() {
    let (mut app, context) = startup(None);
    assert!(app.dock_menu_section(SectionId::Layers, DockSide::Right));
    frame(&mut app, &context, vec![]);
    assert!(app.right_dock_rect.is_some());

    frame_size(&mut app, &context, Vec2::new(800., 600.), vec![]);
    assert!(
        app.right_dock_rect.is_none(),
        "narrow window must drop the optional right column"
    );
    assert!(app.command_input_rect().is_some(), "command line kept");
    let canvas = canvas_rect(&app);
    assert!(canvas.width() >= 320., "canvas width {}", canvas.width());
    assert_eq!(app.view_rects.len(), 4);
}

#[test]
fn reset_and_reload_menus_follow_the_documented_commands() {
    let path = owned_layout_path("menus");
    let _ = std::fs::remove_file(&path);
    let (mut app, context) = startup(Some(path.clone()));
    app.dock_menu_section(SectionId::Projects, DockSide::Right);
    frame(&mut app, &context, vec![]);
    app.set_menus_as_default();
    assert_eq!(app.dock.right_sections(), vec![SectionId::Projects]);

    // Reset Menus returns to the documented default mode in place.
    app.reset_menus(Some(&context));
    frame(&mut app, &context, vec![]);
    assert_eq!(app.dock.left_sections(), DOCUMENTED_LEFT.to_vec());
    assert!(app.dock.right_sections().is_empty());
    assert_eq!(app.view_rects.len(), 4);

    // Reload Menus brings back the configuration the program opens with.
    app.reload_menus();
    frame(&mut app, &context, vec![]);
    assert_eq!(app.dock.right_sections(), vec![SectionId::Projects]);

    std::fs::remove_file(&path).unwrap();
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn a_schema_one_layout_file_resets_with_a_diagnostic_instead_of_crashing() {
    let path = owned_layout_path("schema1");
    std::fs::write(
        &path,
        br#"{"schema_version": 1, "panels": {"right-dock.project-manager": {"visible": true, "collapsed": false, "size": 260.0}}}"#,
    )
    .unwrap();
    let mut app = MatrixApp::new(Document::new("schema one")).unwrap();
    app.set_dock_layout_path(Some(path.clone()));
    let diagnostics = app.load_dock_layout();
    assert!(
        diagnostics.iter().any(|line| line.contains("version 1")),
        "{diagnostics:?}"
    );
    assert_eq!(app.dock, DockLayout::matrix_default());
    std::fs::remove_file(&path).unwrap();
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn builder_area_stays_reserved_and_non_interactive() {
    let (mut app, context) = startup(None);
    let slot = app.builder_slot_rect.expect("reserved builder area");
    let entities = app.controller.document().entities.len();
    let undo = app.controller.coordinator().undo_len();
    let order = app.dock.left_sections();
    click(&mut app, &context, slot.center());
    assert_eq!(app.controller.document().entities.len(), entities);
    assert_eq!(app.controller.coordinator().undo_len(), undo);
    assert_eq!(app.dock.left_sections(), order);
}

#[test]
fn invalid_command_stays_visible_in_the_status_surface_and_the_transcript() {
    let (mut app, context) = startup(None);
    frame(&mut app, &context, vec![Event::Text("NotACommand".into())]);
    frame(&mut app, &context, vec![key(Key::Enter)]);
    assert_eq!(app.status.kind(), StatusKind::Error);
    let message = app.status.message().to_owned();
    assert!(!message.is_empty());
    assert!(app.transcript.text().contains(&message));

    frame(&mut app, &context, vec![key(Key::Escape)]);
    assert_eq!(app.status.kind(), StatusKind::Error);
    frame(&mut app, &context, vec![Event::Text("Line".into())]);
    frame(&mut app, &context, vec![key(Key::Enter)]);
    frame(&mut app, &context, vec![Event::Text("0,0,0".into())]);
    frame(&mut app, &context, vec![key(Key::Enter)]);
    frame(&mut app, &context, vec![Event::Text("1,1,0".into())]);
    frame(&mut app, &context, vec![key(Key::Enter)]);
    assert_eq!(app.controller.document().entities.len(), 1);
    assert_eq!(app.status.kind(), StatusKind::Error, "{message}");

    app.status.acknowledge();
    assert_eq!(app.status.kind(), StatusKind::Ready);
    assert!(app.status_line().starts_with("Ready: "));
}

#[test]
fn the_left_rail_and_command_line_keep_their_documented_bounds() {
    let (mut app, context) = startup(None);
    app.dock.set_size(PanelId::LeftRail, 5_000.);
    app.dock.set_size(PanelId::CommandLine, 5_000.);
    frame(&mut app, &context, vec![]);
    assert_eq!(app.regions.left_rail, om9_app::dock_layout::LEFT_RAIL_MAX);
    assert_eq!(app.regions.command_line, om9_app::dock_layout::COMMAND_MAX);
    app.dock.set_size(PanelId::LeftRail, -5.);
    app.dock.set_size(PanelId::CommandLine, -5.);
    frame(&mut app, &context, vec![]);
    assert_eq!(app.regions.left_rail, om9_app::dock_layout::LEFT_RAIL_MIN);
    assert_eq!(app.regions.command_line, om9_app::dock_layout::COMMAND_MIN);
    assert_eq!(app.view_rects.len(), 4);
    assert!(app.view_rects.contains_key(&ViewId::Perspective));
}
