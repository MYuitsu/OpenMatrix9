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

mod common;
use common::{DOCUMENTED_LEFT, dock_right_fixture};

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
    assert!(dock_right_fixture(&mut app));
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
fn trim_display_fallback_warning_is_visible_without_hiding_errors() {
    use om9_core::{budget::Limits,geometry::Geometry,model::Entity,rhino_semantics::{set_trimmed_surface,TrimmedSurface}};
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    let trim=TrimmedSurface {underlying:surface.clone(),outer:vec![[0.,0.],[1.,0.],[1.,1.],[0.,1.]],
        inners:vec![vec![[0.1,0.1],[0.9,0.1],[0.9,0.9],[0.1,0.9]],vec![[0.3,0.3],[0.7,0.3],[0.7,0.7],[0.3,0.7]]],reversed:false};
    let mut entity=Entity::new("trim",Geometry::Surface(surface),None);
    set_trimmed_surface(&mut entity,&trim,&limits).unwrap();
    let mut document=Document::new("trim warning");document.entities.insert(entity.id,entity);
    let mut app=MatrixApp::new(document).unwrap();let context=egui::Context::default();
    frame(&mut app,&context,vec![]);
    assert!(app.status_line().contains("Trim display unsupported"));
    assert!(app.status_line().contains("enclosing selection window"));
    app.status.error("important error");
    assert!(app.status_line().contains("important error"));
    app.status.acknowledge();
    assert!(app.status_line().contains("Trim display unsupported"));
}

#[test]
fn invalid_command_stays_visible_in_the_status_surface_and_the_transcript() {
    let (mut app, context) = startup(None);
    let editor = app.command_input_rect().expect("rendered command editor");
    click(&mut app, &context, editor.center());
    frame(&mut app, &context, vec![Event::Text("NotACommand".into())]);
    frame(&mut app, &context, vec![key(Key::Enter)]);
    assert_eq!(app.status.kind(), StatusKind::Error);
    let message = app.status.message().to_owned();
    assert!(!message.is_empty());
    assert!(app.transcript.text().contains(&message));

    frame(&mut app, &context, vec![key(Key::Escape)]);
    assert_eq!(app.status.kind(), StatusKind::Error);
    for text in ["Line", "0,0,0", "1,1,0"] {
        let editor = app.command_input_rect().unwrap();
        click(&mut app, &context, editor.right_center() - Vec2::new(6., 0.));
        frame(&mut app, &context, vec![key(Key::End)]);
        frame(&mut app, &context, vec![Event::Text(text.into())]);
        assert_eq!(app.transcript.draft(), text, "CMD draft before Enter: {:?}, focus {:?}", app.transcript.text(), context.memory(|m| m.focused()));
        frame(&mut app, &context, vec![key(Key::Enter)]);
        frame(&mut app, &context, vec![Event::Key {
            key: Key::Enter, physical_key: None, pressed: false, repeat: false, modifiers: Modifiers::NONE,
        }]);
    }
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


#[test]
fn ring_rail_toolbar_places_the_confirmed_region_size() {
    use om9_core::geometry::{Curve, Geometry};

    let mut app = MatrixApp::new(Document::with_template_tolerance("Top11 Ring Rail")).unwrap();
    let ctx = egui::Context::default();
    let before = app.controller.document().clone();
    let settle = |app: &mut MatrixApp| {
        for _ in 0..8 { frame(app, &ctx, vec![]); }
        frame(app, &ctx, vec![])
    };
    let text_at = |out: &egui::FullOutput, name: &str| -> Pos2 {
        out.shapes.iter().find_map(|shape| {
            let egui::Shape::Text(text) = &shape.shape else { return None };
            let center = text.pos + text.galley.size() / 2.;
            (text.galley.text() == name && shape.clip_rect.contains(center)).then_some(center)
        }).unwrap_or_else(|| {
            let matching: Vec<_> = out.shapes.iter().filter_map(|item| match &item.shape {
                egui::Shape::Text(text) if text.galley.text().contains(name) => Some((text.pos, item.clip_rect)),
                _ => None,
            }).collect();
            panic!("missing visible {name}: {matching:?}");
        })
    };
    settle(&mut app);
    let ring = ctx.data(|data| data.get_temp::<Rect>(egui::Id::new(("top11-sidebar", "RingRail")))).unwrap();
    click(&mut app, &ctx, ring.center());
    let out = settle(&mut app);
    let region = text_at(&out, "USA");
    assert_eq!(app.controller.ring_rail_size(), Some(("US", 7.)));
    click(&mut app, &ctx, region);
    let out = settle(&mut app);
    for name in ["Britain", "Japan", "Swiss", "European", "France", "German", "Custom Ring Size"] { text_at(&out, name); }
    click(&mut app, &ctx, text_at(&out, "VNM"));
    assert_eq!(app.controller.ring_rail_size(), Some(("US", 7.)));
    let out = settle(&mut app);
    let arrow_after = |out: &egui::FullOutput, label: &str| {
        let anchor = text_at(out, label);
        out.shapes.iter().find_map(|item| {
            let egui::Shape::Text(text) = &item.shape else { return None };
            let center = text.pos + text.galley.size() / 2.;
            (text.galley.text() == "▶" && item.clip_rect.contains(center)
                && (center.y - anchor.y).abs() < 6. && center.x > anchor.x).then_some(center)
        }).expect("visible green arrow in the requested row")
    };
    click(&mut app, &ctx, arrow_after(&out, "Set Region"));
    assert_eq!(app.controller.ring_rail_size(), Some(("VNM", 17.32)));
    app.controller.submit("RingSize=20").unwrap();
    assert_eq!(app.controller.document(), &before);
    let out = settle(&mut app);
    click(&mut app, &ctx, arrow_after(&out, "VNM"));
    let entity = app.controller.document().entities.values().next().expect("placed Ring Rail");
    let Geometry::Curve(Curve::Circle {radius, center, x, y, ..}) = &entity.geometry else { panic!("Front circle") };
    assert_eq!((*radius, *center, *x, *y), (10., [0.; 3], [0.,0.,1.], [1.,0.,0.]));
    assert_eq!(app.controller.coordinator().undo_len(), 1);
    let after = app.controller.document().clone();
    let out = settle(&mut app);
    text_at(&out, "VNM");
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &before);
    app.controller.redo().unwrap();
    assert_eq!(app.controller.document(), &after);
}

#[test]
fn gem_loader_drag_keeps_the_preview_through_both_docks_and_the_canvas() {
    use om9_app::session::{gem::{GemTool, LOADER}, InvocationOrigin};
    use om9_app::tool_panel::ToolPanel;
    let mut app = MatrixApp::new(Document::with_template_tolerance("gem dock replay")).unwrap();
    let ctx = egui::Context::default();
    let section = SectionId::Tool(ToolPanel::Gem(GemTool::Loader));
    app.controller.dispatch(LOADER, InvocationOrigin::Menu).unwrap();
    app.controller.submit("BRep=No").unwrap();
    app.controller.gem_loader_point([2., 3., 0.]).unwrap();
    let dimensions = app.controller.gem_session().unwrap().dimensions;
    for _ in 0..5 { frame(&mut app, &ctx, vec![]); }
    let press = |pos, pressed| Event::PointerButton {
        pos, pressed, button: egui::PointerButton::Primary, modifiers: Modifiers::NONE,
    };
    let drag = |app: &mut MatrixApp, target: Pos2, escape: bool| {
        let bar = app.section_rect(section.key()).unwrap().center();
        frame(app, &ctx, vec![Event::PointerMoved(bar), press(bar, true)]);
        frame(app, &ctx, vec![Event::PointerMoved(bar + Vec2::new(8., 0.))]);
        frame(app, &ctx, vec![Event::PointerMoved(target)]);
        assert_eq!(app.section_drag(), Some(section));
        if escape { frame(app, &ctx, vec![key(Key::Escape)]); }
        frame(app, &ctx, vec![press(target, false)]);
        for _ in 0..3 { frame(app, &ctx, vec![]); }
        let gem = app.controller.gem_session().expect("placement only changes presentation");
        assert_eq!(gem.dimensions, dimensions);
        assert_eq!(gem.points, [[2., 3., 0.]]);
        assert!(app.controller.document().entities.is_empty());
    };
    let right = canvas_rect(&app).right_center() - Vec2::new(8., 0.);
    drag(&mut app, right, true);
    assert_eq!(app.dock.section(section).dock, DockSide::Left);
    drag(&mut app, right, false);
    assert_eq!(app.dock.section(section).dock, DockSide::Right);
    let saved = app.dock.section(section);
    for width in [1199., 1200.] {
        for _ in 0..3 { frame_size(&mut app, &ctx, Vec2::new(width, 900.), vec![]); }
        assert_eq!(app.dock.section(section), saved);
        let center = app.section_rect(section.key()).unwrap().center();
        assert_eq!(app.right_dock_rect.is_some_and(|rect| rect.contains(center)), width >= 1200.);
    }
    frame(&mut app, &ctx, vec![]);
    let canvas = app.view_rects[&ViewId::Perspective].center();
    drag(&mut app, canvas, false);
    assert_eq!(app.dock.section(section).dock, DockSide::Floating);
    drag(&mut app, Pos2::new(70., 190.), false);
    assert_eq!(app.dock.section(section).dock, DockSide::Left);
    use om9_app::layout_profiles::ProfileAction;
    app.edit_layout_profile(ProfileAction::Create("Gem left")).unwrap();
    app.edit_layout_profile(ProfileAction::Duplicate("Gem right")).unwrap();
    assert!(app.dock_menu_section(section, DockSide::Right));
    for name in ["Gem left", "Gem right"] {
        app.edit_layout_profile(ProfileAction::Switch(name)).unwrap();
        frame(&mut app, &ctx, vec![]);
        assert_eq!(app.controller.gem_session().unwrap().points, [[2., 3., 0.]]);
        assert_eq!(app.controller.gem_session().unwrap().dimensions, dimensions);
        assert!(app.section_rect(section.key()).is_some());
    }
    app.controller.gem_apply().unwrap();
    assert_eq!(app.controller.document().entities.len(), 1);
    assert_eq!(app.controller.coordinator().undo_len(), 1);
    app.controller.undo().unwrap();
    assert!(app.controller.document().entities.is_empty());
}

#[test]
fn switching_profiles_restores_actual_rail_and_command_dimensions() {
    use om9_app::layout_profiles::ProfileAction;
    let (mut app, ctx) = startup(None);
    app.edit_layout_profile(ProfileAction::Create("Compact")).unwrap();
    let compact = app.dock.state(PanelId::LeftRail).size;
    let cmd = app.dock.state(PanelId::CommandLine).size;
    app.edit_layout_profile(ProfileAction::Duplicate("Wide")).unwrap();
    app.dock.set_size(PanelId::LeftRail, 405.);
    app.dock.set_size(PanelId::CommandLine, 171.);
    for (name, width, height) in [("Compact", compact, cmd), ("Wide", 405., 171.), ("Compact", compact, cmd)] {
        app.edit_layout_profile(ProfileAction::Switch(name)).unwrap();
        for _ in 0..3 { frame(&mut app, &ctx, vec![]); }
        assert_near(app.left_rail_outer_px, width, 1.);
        assert_near(app.dock.state(PanelId::CommandLine).size, height, 1.);
    }
}

#[test]
fn workspace_header_drags_keep_snap_project_layer_and_pending_line() {
    for scale in [1.0, 1.25, 1.5] {
        let (mut app, ctx) = startup(None);
        let input_frame = |app: &mut MatrixApp, events| {
            let mut raw = egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 900.))),
                events,
                ..Default::default()
            };
            raw.viewports.get_mut(&egui::ViewportId::ROOT).unwrap().native_pixels_per_point = Some(scale);
            ctx.run_ui(raw, |ui| app.show(ui)).textures_delta.clear();
        };
        app.controller.set_grid_snap(true);
        let row = app.project_manager.create_project("Bench", 0).unwrap();
        app.project_manager.select(row).unwrap();
        app.controller.submit("Line").unwrap();
        app.controller.submit("2,3,0").unwrap();
        let before = app.controller.document().clone();
        for section in [SectionId::Snaps, SectionId::InfoSettings, SectionId::Layers, SectionId::Projects] {
            for side in [DockSide::Right, DockSide::Floating, DockSide::Left] {
                for _ in 0..3 { input_frame(&mut app, vec![]); }
                let header = app.section_rect(section.key()).unwrap().center();
                let to = match side {
                    DockSide::Right => canvas_rect(&app).right_center() - Vec2::new(5., 0.),
                    DockSide::Floating => app.view_rects[&ViewId::Perspective].center(),
                    _ => Pos2::new(65., 190.),
                };
                let button = |pos, pressed| Event::PointerButton {
                    pos, pressed, button: egui::PointerButton::Primary, modifiers: Modifiers::NONE,
                };
                input_frame(&mut app, vec![Event::PointerMoved(header), button(header, true)]);
                input_frame(&mut app, vec![Event::PointerMoved(header + Vec2::new(9., 0.))]);
                input_frame(&mut app, vec![Event::PointerMoved(to)]);
                assert_eq!(app.section_drag(), Some(section));
                assert!(app.drop_preview.as_ref().is_some_and(|preview| preview.indicator.is_some()));
                input_frame(&mut app, vec![button(to, false)]);
                assert_eq!(app.dock.section(section).dock, side);
                assert!(app.controller.grid_snap());
                assert_eq!(app.project_manager.active().id, row);
                assert_eq!(app.controller.point_constraint_base().unwrap().xyz(), [2., 3., 0.]);
                assert_eq!(app.controller.document(), &before);
            }
        }
    }
}
