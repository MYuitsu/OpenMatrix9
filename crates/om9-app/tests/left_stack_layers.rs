// SPDX-License-Identifier: LGPL-2.1-or-later
//! Left-column parity evidence for plan task OM9-L3-01-01-002 (FEAT-002).
//!
//! The user's Matrix9 screenshot (2026-10-16) shows one left column, top to
//! bottom: ICON HISTORY, MAIN MENU, DISPLAY, SNAPS, INFO & SETTINGS, LAYERS,
//! PROJECTS, then the builder area. There is no right dock. These tests drive
//! the real `MatrixApp::show` frame in the DEFAULT layout and prove:
//!
//! - Layers sits directly below Info & Settings and above Projects at every
//!   supported frame size, after Reset Menus from a perturbed layout and after a
//!   save/reload round trip;
//! - the Layers title bar is visible without scrolling at rail 300 and 460;
//! - no right dock exists by default;
//! - the rail's OUTER width is 300 and its inner content is `LEFT_RAIL_CHROME_PX`
//!   narrower;
//! - the sidebar icons follow the rail width (`rail_icon_edge`) without
//!   overflowing, while toolbar icons stay at 22;
//! - the widest rail keeps the 320 pt canvas minimum.
//!
//! No test docks Layers to the right: the only right-dock fixture is
//! `RIGHT_DOCK_FIXTURE` (Snaps), a user-initiated OM9-extension layout.
//!
//! SIMULATED/AUTOMATED evidence. It does not replace the human M01-UI sign-off.
use eframe::egui::{self, Vec2};
use om9_app::dock_layout::{
    CANVAS_MIN_WIDTH, FloatRect, LEFT_RAIL_CHROME_PX, LEFT_RAIL_DEFAULT, LEFT_RAIL_MAX,
    LEFT_RAIL_MIN, NARROW_WINDOW_WIDTH, PanelId, RIGHT_DOCK_FIXTURE, RIGHT_DOCK_MAX, SectionId,
    rail_icon_edge,
};
use om9_app::file_lifecycle::FileKind;
use om9_app::ui::MatrixApp;
use om9_app::workbench_regions::{RegionNote, RegionPlan, YieldReason};
use om9_core::model::Document;

mod common;
use common::{
    DOCUMENTED_LEFT, assert_layers_between_info_and_projects, dock_right_fixture, settle,
};

/// Window sizes in logical points: the nominal minimum, 1366x768 at 100%
/// (client area), at 125% and at 150% after the window-level cap, and a roomy
/// window.
const SUPPORTED: [(f32, f32); 5] = [
    (1280., 720.),
    (1280., 688.),
    (1093., 585.),
    (911., 480.),
    (1600., 1000.),
];

/// A fresh app on a fresh egui context. egui remembers a live panel's size in
/// its own memory, so a width other than the default must be set BEFORE the
/// first frame (FEAT-001 finding).
fn fresh(
    size: Vec2,
    rail: Option<f32>,
    right: Option<f32>,
    fixture: bool,
) -> (MatrixApp, egui::Context) {
    let mut app = MatrixApp::new(Document::new("left stack layers")).unwrap();
    if let Some(width) = rail {
        app.dock.set_size(PanelId::LeftRail, width);
    }
    if let Some(width) = right {
        app.dock.set_size(PanelId::RightDock, width);
    }
    if fixture {
        assert!(dock_right_fixture(&mut app));
    }
    let context = egui::Context::default();
    settle(&mut app, &context, size, 4);
    (app, context)
}

fn plan(app: &MatrixApp) -> &RegionPlan {
    app.region_plan.as_ref().expect("a validated region plan")
}

fn yielded(plan: &RegionPlan, reason: YieldReason) -> bool {
    plan.notes
        .iter()
        .any(|note| matches!(note, RegionNote::RightDockYielded { reason: r } if *r == reason))
}

fn any_yield(plan: &RegionPlan) -> bool {
    plan.notes
        .iter()
        .any(|note| matches!(note, RegionNote::RightDockYielded { .. }))
}

fn owned_layout_path(case: &str) -> std::path::PathBuf {
    let directory =
        std::env::temp_dir().join(format!("om9-left-stack-{case}-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    directory.join("ui-layout.json")
}

fn cleanup(path: &std::path::Path) {
    let _ = std::fs::remove_file(path);
    if let Some(parent) = path.parent() {
        let _ = std::fs::remove_dir_all(parent);
    }
}

fn assert_default_left_stack(app: &MatrixApp, label: &str) {
    assert_eq!(
        app.dock.left_sections(),
        DOCUMENTED_LEFT.to_vec(),
        "{label}: left column order"
    );
    assert!(
        app.dock.right_sections().is_empty(),
        "{label}: nothing is docked right by default"
    );
    assert!(
        app.dock.floating_sections().is_empty(),
        "{label}: nothing floats by default"
    );
    assert!(app.right_dock_rect.is_none(), "{label}: right dock rect");
    assert_layers_between_info_and_projects(app, label);
}

#[test]
fn layers_sits_between_info_and_projects_at_supported_frames() {
    for (width, height) in SUPPORTED {
        let label = format!("{width}x{height}");
        let (app, _context) = fresh(Vec2::new(width, height), None, None, false);
        assert!(
            app.region_error.is_none(),
            "{label}: region error {:?}",
            app.region_error
        );
        assert_default_left_stack(&app, &label);
        let info = app.section_rect(SectionId::InfoSettings.key()).unwrap();
        let layers = app.section_rect(SectionId::Layers.key()).unwrap();
        let projects = app.section_rect(SectionId::Projects.key()).unwrap();
        println!(
            "OM9_SWEEP left_stack window={label} info_y={} layers_y={} projects_y={}",
            info.min.y, layers.min.y, projects.min.y
        );
    }
}

#[test]
fn layers_stays_in_the_left_column_after_reset_menus_from_a_perturbed_layout() {
    for (width, height) in SUPPORTED {
        let label = format!("reset {width}x{height}");
        let size = Vec2::new(width, height);
        let (mut app, context) = fresh(size, None, None, false);

        // Perturb: collapse Layers, move it up, dock the fixture right, float
        // Projects. Every one of these is a legal user action.
        app.dock.set_section_collapsed(SectionId::Layers, true);
        assert!(app.move_menu_section(SectionId::Layers, -1));
        assert!(dock_right_fixture(&mut app));
        assert!(app.float_menu_section(
            SectionId::Projects,
            FloatRect {
                x: 500.,
                y: 200.,
                width: 300.,
                height: 200.,
            }
        ));
        settle(&mut app, &context, size, 3);
        assert_ne!(
            app.dock.left_sections(),
            DOCUMENTED_LEFT.to_vec(),
            "{label}: the perturbation must change the layout"
        );

        app.reset_menus(Some(&context));
        settle(&mut app, &context, size, 4);
        assert!(
            !app.dock.section(SectionId::Layers).collapsed,
            "{label}: Layers is expanded again"
        );
        assert_default_left_stack(&app, &label);
    }
}

#[test]
fn layers_stays_in_the_left_column_after_a_save_and_reload_round_trip() {
    let size = Vec2::new(1280., 720.);
    // Default layout, and the user-initiated fixture-right layout.
    for fixture in [false, true] {
        let case = if fixture { "fixture-right" } else { "default" };
        let path = owned_layout_path(case);
        let _ = std::fs::remove_file(&path);
        {
            let mut app = MatrixApp::new(Document::new("save")).unwrap();
            app.set_dock_layout_path(Some(path.clone()));
            let context = egui::Context::default();
            settle(&mut app, &context, size, 3);
            if fixture {
                assert!(dock_right_fixture(&mut app));
                settle(&mut app, &context, size, 3);
            }
            app.set_menus_as_default();
            assert!(path.exists(), "{case}: Set as Default writes the file");
        }

        let mut app = MatrixApp::new(Document::new("reload")).unwrap();
        app.set_dock_layout_path(Some(path.clone()));
        let diagnostics = app.load_dock_layout();
        assert!(diagnostics.is_empty(), "{case}: {diagnostics:?}");
        let context = egui::Context::default();
        settle(&mut app, &context, size, 4);
        if fixture {
            let mut expected = DOCUMENTED_LEFT.to_vec();
            expected.retain(|id| *id != RIGHT_DOCK_FIXTURE);
            assert_eq!(app.dock.left_sections(), expected, "{case}: left order");
            assert_eq!(
                app.dock.right_sections(),
                vec![RIGHT_DOCK_FIXTURE],
                "{case}: right column"
            );
            assert_layers_between_info_and_projects(&app, case);
        } else {
            assert_default_left_stack(&app, case);
        }
        cleanup(&path);
    }
}

#[test]
fn layers_title_bar_is_visible_without_scrolling_at_1280x720() {
    let size = Vec2::new(1280., 720.);
    for rail in [LEFT_RAIL_DEFAULT, LEFT_RAIL_MAX] {
        let label = format!("rail {rail}");
        let (app, _context) = fresh(size, Some(rail), None, false);
        assert!(
            (app.left_rail_outer_px - rail).abs() <= 0.5,
            "{label}: outer {}",
            app.left_rail_outer_px
        );
        let sidebar = app.sidebar_rect.expect("sidebar rect");
        let layers = app.section_rect(SectionId::Layers.key()).unwrap();
        let projects = app.section_rect(SectionId::Projects.key()).unwrap();
        // Visible without scrolling: the whole bar lies inside the viewport of
        // the scroll area, which starts at the sidebar top and ends at the
        // window bottom.
        assert!(
            layers.min.y >= sidebar.min.y && layers.max.y <= sidebar.max.y.min(size.y),
            "{label}: Layers bar {layers:?} not inside the visible sidebar {sidebar:?}"
        );
        assert_layers_between_info_and_projects(&app, &label);
        let content = app.sidebar_content_size.expect("scroll content size");
        println!(
            "OM9_SWEEP layers_visible window=1280x720 rail={rail} layers_y={} projects_y={} \
             sidebar_y={}..{} content_h={} visible=true",
            layers.min.y, projects.min.y, sidebar.min.y, sidebar.max.y, content.y
        );
        // The Projects bar below Layers also starts inside the viewport, so the
        // Info & Settings / Layers / Projects trio is readable without a scroll.
        assert!(
            projects.max.y <= sidebar.max.y.min(size.y),
            "{label}: Projects bar {projects:?} below the visible area"
        );
    }
}

#[test]
fn default_frame_has_no_right_dock() {
    for (width, height) in SUPPORTED {
        let label = format!("{width}x{height}");
        let (app, _context) = fresh(Vec2::new(width, height), None, None, false);
        assert!(
            !app.regions.right_dock_visible,
            "{label}: regions.right_dock_visible"
        );
        assert!(app.right_dock_rect.is_none(), "{label}: right_dock_rect");
        assert_eq!(app.right_dock_outer_px, 0., "{label}: right dock outer");
        let plan = plan(&app);
        assert!(plan.right_dock.is_none(), "{label}: plan right dock");
        assert!(
            !any_yield(plan),
            "{label}: nothing was docked, so nothing can yield: {:?}",
            plan.notes
        );
        assert!(app.dock.right_sections().is_empty(), "{label}");
    }
}

#[test]
fn rail_outer_width_is_300_and_inner_is_chrome_less() {
    for (width, height) in [(1280., 720.), (1600., 1000.)] {
        let label = format!("{width}x{height}");
        let (app, _context) = fresh(Vec2::new(width, height), None, None, false);
        let outer = app.left_rail_outer_px;
        let inner = app.sidebar_rect.expect("sidebar rect").width();
        assert!(
            (outer - LEFT_RAIL_DEFAULT).abs() <= 0.5,
            "{label}: outer {outer}"
        );
        assert!(
            (inner - (outer - LEFT_RAIL_CHROME_PX)).abs() <= 0.5,
            "{label}: inner {inner} vs outer {outer} - {LEFT_RAIL_CHROME_PX}"
        );
        assert_eq!(
            app.dock.state(PanelId::LeftRail).size,
            app.regions.left_rail,
            "{label}: stored size is the planner's rail"
        );
        assert!(
            (app.dock.state(PanelId::LeftRail).size - outer).abs() <= 0.5,
            "{label}: stored PanelState.size {} is the OUTER width {outer}",
            app.dock.state(PanelId::LeftRail).size
        );
        println!("OM9_SWEEP rail_width window={label} outer={outer} inner={inner}");
    }
}

#[test]
fn icon_edge_follows_the_rail_width() {
    let size = Vec2::new(1600., 1000.);
    let mut previous: Option<f32> = None;
    for rail in [LEFT_RAIL_MIN, LEFT_RAIL_DEFAULT, 380., LEFT_RAIL_MAX] {
        let label = format!("rail {rail}");
        let (app, _context) = fresh(size, Some(rail), None, false);
        let outer = app.left_rail_outer_px;
        assert!((outer - rail).abs() <= 0.5, "{label}: outer {outer}");
        let edge = app.left_icon_edge;
        let expected = (rail_icon_edge(outer) * app.screen_icon_scale())
            .clamp(23., 42.)
            .round();
        assert_eq!(edge, expected, "{label}: edge follows outer and screen");
        if let Some(previous) = previous {
            assert!(edge > previous, "{label}: {edge} not above {previous}");
        }
        previous = Some(edge);

        let sidebar = app.sidebar_rect.expect("sidebar rect");
        assert!(!app.rail_icon_rects.is_empty(), "{label}: icons drawn");
        for rect in &app.rail_icon_rects {
            assert!(
                rect.width() <= edge + 0.5
                    && rect.width() >= 16.
                    && (rect.width() - rect.height()).abs() <= 0.5,
                "{label}: icon {rect:?} does not fit the {edge} rail edge"
            );
            assert!(
                rect.min.x >= sidebar.min.x - 0.5 && rect.max.x <= sidebar.max.x + 0.5,
                "{label}: icon {rect:?} overflows the sidebar {sidebar:?}"
            );
        }
        // Toolbar icons follow the same screen scale, not the rail column rule.
        let toolbar = app
            .file_toolbar_rect(FileKind::New)
            .expect("New toolbar icon");
        assert!(
            (toolbar.width() - app.toolbar_icon_edge()).abs() <= 0.5,
            "{label}: toolbar icon {toolbar:?} must match the responsive edge"
        );
        println!(
            "OM9_SWEEP icon_edge rail={rail} outer={outer} edge={edge} icons={} toolbar={}",
            app.rail_icon_rects.len(),
            toolbar.width()
        );
    }
    // Anchors of the rule at the three documented widths.
    assert_eq!(rail_icon_edge(LEFT_RAIL_MIN), 23.);
    assert_eq!(rail_icon_edge(LEFT_RAIL_DEFAULT), 28.);
    assert_eq!(rail_icon_edge(LEFT_RAIL_MAX), 42.);
}

#[test]
fn max_rail_keeps_the_canvas_minimum() {
    // Rail 460 with no right dock at the nominal minimum and the smallest
    // plannable capped window.
    for (width, height) in [(1280., 720.), (911., 480.)] {
        let label = format!("rail 460 {width}x{height}");
        let (app, _context) = fresh(Vec2::new(width, height), Some(LEFT_RAIL_MAX), None, false);
        assert!(
            app.region_error.is_none(),
            "{label}: {:?}",
            app.region_error
        );
        let plan = plan(&app);
        assert!(
            plan.center.width >= CANVAS_MIN_WIDTH,
            "{label}: {:?}",
            plan.center
        );
        let canvas = app
            .view_rects
            .values()
            .copied()
            .reduce(|a, b| a.union(b))
            .expect("four viewports");
        assert!(
            canvas.width() >= CANVAS_MIN_WIDTH,
            "{label}: canvas {}",
            canvas.width()
        );
        assert_eq!(app.view_rects.len(), 4, "{label}");
        assert_layers_between_info_and_projects(&app, &label);
    }

    // At exactly NARROW_WINDOW_WIDTH the widest rail and the widest right dock
    // leave exactly the canvas minimum and the dock is shown; one point less
    // and the dock yields.
    let (app, _context) = fresh(
        Vec2::new(NARROW_WINDOW_WIDTH, 720.),
        Some(LEFT_RAIL_MAX),
        Some(RIGHT_DOCK_MAX),
        true,
    );
    let at_threshold = plan(&app);
    assert!(app.region_error.is_none(), "{:?}", app.region_error);
    assert!(at_threshold.right_dock.is_some(), "dock shown at 1200");
    assert!(app.right_dock_rect.is_some(), "dock rendered at 1200");
    assert_eq!(at_threshold.left_rail.unwrap().width, LEFT_RAIL_MAX);
    assert_eq!(at_threshold.right_dock.unwrap().width, RIGHT_DOCK_MAX);
    assert_eq!(
        at_threshold.center.width, CANVAS_MIN_WIDTH,
        "1200 - 460 - 420 leaves exactly the canvas minimum"
    );
    assert!(!any_yield(at_threshold), "{:?}", at_threshold.notes);

    let (app, _context) = fresh(
        Vec2::new(NARROW_WINDOW_WIDTH - 1., 720.),
        Some(LEFT_RAIL_MAX),
        Some(RIGHT_DOCK_MAX),
        true,
    );
    let below = plan(&app);
    assert!(app.region_error.is_none(), "{:?}", app.region_error);
    assert!(below.right_dock.is_none(), "dock yields at 1199");
    assert!(app.right_dock_rect.is_none(), "dock not rendered at 1199");
    assert!(
        yielded(below, YieldReason::NarrowWindow),
        "{:?}",
        below.notes
    );
    assert!(
        !yielded(below, YieldReason::NotEnoughRoom),
        "{:?}",
        below.notes
    );
    assert!(below.center.width >= CANVAS_MIN_WIDTH);
    // The Snaps fixture left the column; Layers is still between Info and
    // Projects.
    assert_layers_between_info_and_projects(&app, "1199 fixture right");
}
