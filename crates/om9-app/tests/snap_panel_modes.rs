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
    render_frame(app, context, events);
}

fn render_frame(app: &mut MatrixApp, context: &egui::Context, events: Vec<Event>) -> egui::FullOutput {
    let focused = !events.iter().any(|event| matches!(event, Event::WindowFocused(false)));
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 900.))),
            focused,
            events,
            ..Default::default()
        },
        |ui| app.show(ui),
    );
    output.textures_delta.clear();
    output
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

fn set_apparent_intersections(app: &mut MatrixApp, context: &egui::Context, enabled: bool) {
    app.focus_command_editor();
    frame(app, context, vec![]);
    let option = if enabled { "on" } else { "off" };
    let command = format!("ApparentIntersections {option}");
    frame(app, context, vec![Event::Text(command.clone())]);
    frame(app, context, vec![Event::Key {
        key: Key::Enter, physical_key: None, pressed: true, repeat: false, modifiers: Modifiers::NONE,
    }]);
    assert!(app.status_line().contains(&command));
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
        [12., -4., 7.],
        "Project applies to object snaps, not an unconstrained free point"
    );
    assert_eq!(controller.input_plane().origin, plane.origin);
    controller.set_planar(true);
    assert!(controller.planar() && controller.project());
    controller.set_project(false);
    assert!(!controller.project());
    assert!(controller.planar());
}

#[cfg(feature = "occt-brep")]
#[test]
fn surface_snap_override_commits_an_exact_sphere_source_point() {
    use om9_core::geometry::{Curve, Geometry};
    use om9_scene::camera::Viewport;
    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::with_template_tolerance("surface snap precision")).unwrap();
    app.controller.solid_sphere_points([0.; 3], 5.).unwrap();
    frame(&mut app, &context, vec![]);
    frame(&mut app, &context, vec![]);
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.submit("Line").unwrap();
    app.set_snap_override(Some(SnapKind::Surface));
    let rect = app.view_rects[&ViewId::LookingDown];
    let viewport = Viewport { origin: [rect.min.x as f64, rect.min.y as f64], size: [rect.width() as f64, rect.height() as f64] };
    let world = [2.13, 1.37, (25.0_f64 - 2.13_f64.powi(2) - 1.37_f64.powi(2)).sqrt()];
    let projected = app.controller.workspace.views[&ViewId::LookingDown].camera.project(world, viewport).unwrap().unwrap().screen;
    let point = Pos2::new(projected[0] as f32, projected[1] as f32);
    let display_hit = om9_scene::osnap::candidates(
        app.controller.scene().unwrap(), &app.controller.workspace.views[&ViewId::LookingDown].camera,
        viewport, projected, Default::default(), None,
    ).unwrap().into_iter().find(|candidate| candidate.kind == SnapKind::Surface).expect("display hit");
    let display_radius = display_hit.world.iter().map(|value| value * value).sum::<f64>().sqrt();
    assert!((display_radius - 5.).abs() > 1e-7, "fixture needs a display hit distinct from the source");
    frame(&mut app, &context, vec![Event::PointerMoved(point)]);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
    let snap = loop {
        if let Some(snap) = app.snap_candidate().filter(|snap| snap.kind == SnapKind::Surface) { break snap.clone(); }
        assert!(std::time::Instant::now() < deadline, "refined surface snap: {}", app.status_line());
        std::thread::sleep(std::time::Duration::from_millis(5));
        frame(&mut app, &context, vec![]);
    };
    let radius = snap.world.iter().map(|value| value * value).sum::<f64>().sqrt();
    assert!((radius - 5.).abs() < 1e-7, "snap must be on the canonical sphere, got {radius}");
    click(&mut app, &context, point);
    app.controller.submit("20,20,20").unwrap();
    let line = app.controller.document().entities.values().find_map(|entity| match &entity.geometry {
        Geometry::Curve(Curve::Line { start, .. }) => Some(*start), _ => None,
    }).expect("created line");
    assert!(line.iter().zip(snap.world).all(|(a, b)| (a-b).abs() < 1e-7));
}

#[cfg(feature = "occt-brep")]
#[test]
fn deferred_surface_click_ignores_key_release_and_cancels_on_window_focus_loss() {
    use om9_scene::camera::Viewport;
    use std::time::{Duration, Instant};

    for loses_focus in [true, false] {
        let context = egui::Context::default();
        let mut app = MatrixApp::new(Document::with_template_tolerance("deferred surface input")).unwrap();
        app.controller.solid_sphere_points([0.; 3], 5.).unwrap();
        frame(&mut app, &context, vec![]);
        frame(&mut app, &context, vec![]);
        app.controller.workspace.active = ViewId::LookingDown;
        app.controller.submit("Line").unwrap();
        app.set_snap_override(Some(SnapKind::Surface));
        let rect = app.view_rects[&ViewId::LookingDown];
        let viewport = Viewport {
            origin: [rect.min.x as f64, rect.min.y as f64],
            size: [rect.width() as f64, rect.height() as f64],
        };
        let [x, y] = app.controller.workspace.views[&ViewId::LookingDown].camera
            .project([0., 0., 5.], viewport).unwrap().unwrap().screen;
        let permit = app.controller.coordinator().external_memory_account()
            .try_acquire_surface_snap().unwrap();
        click(&mut app, &context, Pos2::new(x as f32, y as f32));
        assert!(app.snap_candidate().is_none());
        assert!(app.controller.relative_base().is_none());
        let event = if loses_focus {
            Event::WindowFocused(false)
        } else {
            Event::Key {
                key: Key::Enter, physical_key: None, pressed: false,
                repeat: false, modifiers: Modifiers::NONE,
            }
        };
        frame(&mut app, &context, vec![event]);
        drop(permit);

        let deadline = Instant::now() + Duration::from_secs(8);
        while app.snap_candidate().is_none() && app.controller.relative_base().is_none() {
            assert!(Instant::now() < deadline, "{}", app.status_line());
            std::thread::sleep(Duration::from_millis(5));
            frame(&mut app, &context, vec![]);
        }
        if loses_focus {
            assert!(app.controller.relative_base().is_none(), "a canceled click must not advance Line");
            assert_eq!(app.snap_override(), Some(SnapKind::Surface));
        } else {
            let point = app.controller.relative_base().expect("key release must preserve the queued click");
            assert!(point.xyz().iter().zip([0., 0., 5.]).all(|(actual, expected)| (actual - expected).abs() < 1e-4));
            assert_eq!(app.snap_override(), None);
        }
    }
}

#[test]
fn panel_updates_enabled_state_and_override_in_the_same_frame() {
    let mut app = MatrixApp::new(Document::new("snap panel")).unwrap();
    assert!(app.snap_enabled(SnapKind::End));
    assert!(app.snap_enabled(SnapKind::Mid));
    assert!(SnapKind::ALL.into_iter().filter(|kind| !matches!(kind, SnapKind::End | SnapKind::Mid))
        .all(|kind| !app.snap_enabled(kind)));
    app.set_object_snaps_enabled(false);
    assert!(!app.object_snaps_enabled());
    assert!(app.snap_enabled(SnapKind::End) && app.snap_enabled(SnapKind::Mid));
    app.set_object_snaps_enabled(true);
    assert!(app.object_snaps_enabled());
    assert!(app.snap_enabled(SnapKind::End) && app.snap_enabled(SnapKind::Mid));
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

#[cfg(feature = "occt-brep")]
#[test]
fn directional_nurbs_snaps_require_a_line_base_and_commit_the_refined_point() {
    use om9_core::{curve_ops::{derivative_at, point_at}, geometry::{Curve, Geometry, NurbsCurve, Transform}, model::Entity};
    use om9_scene::camera::Viewport;

    let curve = Curve::Nurbs(NurbsCurve {
        degree: 2, poles: vec![[-1., 0., 0.], [0., 2., 0.], [1., 0., 0.]],
        weights: vec![1.; 3], knots: vec![0., 0., 0., 1., 1., 1.],
        domain: [0., 1.], periodic: false, reversed: false, seam: None,
    });
    let placement = Transform([
        [0., -1., 0., 6.], [2., 0., 0., 2.], [0., 0., 1., 0.], [0., 0., 0., 1.],
    ]);
    let mut entity = Entity::new("directional source", Geometry::Curve(curve.clone()), None);
    entity.placement = placement;
    let source_id = entity.id;
    let mut document = Document::with_template_tolerance("directional NURBS snaps");
    document.entities.insert(source_id, entity);
    let context = egui::Context::default();
    let mut app = MatrixApp::new(document).unwrap();
    app.set_object_snaps_enabled(false);
    frame(&mut app, &context, vec![]);
    frame(&mut app, &context, vec![]);
    app.controller.workspace.active = ViewId::LookingDown;
    let rect = app.view_rects[&ViewId::LookingDown];
    let viewport = Viewport { origin: [rect.min.x as f64, rect.min.y as f64],
        size: [rect.width() as f64, rect.height() as f64] };
    let expected = placement.point(point_at(&curve, 0.5, &Limits::default()).unwrap()).unwrap();
    let projected = app.controller.workspace.views[&ViewId::LookingDown].camera
        .project(expected, viewport).unwrap().unwrap().screen;
    let cursor = Pos2::new(projected[0] as f32, projected[1] as f32);
    let before = app.controller.document().clone();
    let tolerance = before.effective_tolerance().unwrap().absolute_mm;
    for (kind, base) in [(SnapKind::Tangent, [5., 4., 0.]), (SnapKind::Perpendicular, [3., 2., 0.])] {
        app.controller.submit("Line").unwrap();
        app.set_snap_override(Some(kind));
        click(&mut app, &context, cursor);
        assert!(app.snap_candidate().is_none());
        assert!(app.controller.prompt().starts_with("Start of line"), "{}", app.controller.prompt());
        assert_eq!(app.snap_override(), Some(kind));
        assert_eq!(app.controller.document(), &before);

        app.controller.accept_world_point(base).unwrap();
        frame(&mut app, &context, vec![Event::PointerMoved(cursor)]);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
        let snap = loop {
            if let Some(snap) = app.snap_candidate().filter(|snap| snap.kind == kind) {
                break snap.clone();
            }
            assert!(std::time::Instant::now() < deadline, "{kind:?}: {}", app.status_line());
            std::thread::sleep(std::time::Duration::from_millis(5));
            frame(&mut app, &context, vec![]);
        };
        assert_eq!(snap.source.as_ref().unwrap().root, source_id);
        let parameter = snap.parameter.unwrap();
        let witness = placement.point(point_at(&curve, parameter, &Limits::default()).unwrap()).unwrap();
        assert!((0..3).all(|axis| (witness[axis] - snap.world[axis]).abs() <= tolerance));
        let offset = std::array::from_fn::<_, 3, _>(|axis| snap.world[axis] - base[axis]);
        let local = derivative_at(&curve, parameter, &Limits::default()).unwrap();
        let tangent = [-local[1], 2. * local[0], 0.];
        let residual = match kind {
            SnapKind::Tangent => offset[0] * tangent[1] - offset[1] * tangent[0],
            _ => offset[0] * tangent[0] + offset[1] * tangent[1],
        }.abs() / tangent[0].hypot(tangent[1]);
        assert!(residual <= tolerance, "{kind:?} residual {residual}");

        click(&mut app, &context, cursor);
        let line = app.controller.document().entities.values().find_map(|entity| match &entity.geometry {
            Geometry::Curve(Curve::Line { start, end, .. }) => Some((*start, *end)),
            _ => None,
        }).expect("Line committed from preview");
        assert_eq!(line, (base, snap.world));
        assert_eq!(app.snap_override(), None);
        assert_eq!(app.controller.coordinator().undo_len(), 1);
        let after = app.controller.document().clone();
        app.controller.undo().unwrap();
        assert_eq!(app.controller.document(), &before);
        app.controller.redo().unwrap();
        assert_eq!(app.controller.document(), &after);
        app.controller.undo().unwrap();
    }
}

#[test]
fn grid_ortho_and_planar_use_the_rotated_input_plane_in_mm() {
    use om9_core::{coords::{LocalFrame, WorldPoint}, geometry::Unit};
    use om9_scene::camera::CPlane;
    let mut document = Document::with_template_tolerance("rotated grid");
    document.page_unit = Unit::Inch;
    let mut controller = Controller::new(document, Limits::default()).unwrap();
    let plane = CPlane::new([10., -3., 4.], [0., 1., 0.], [-1., 0., 0.]).unwrap();
    controller.workspace.views.get_mut(&controller.workspace.active).unwrap().cplane = plane;
    controller.set_grid_snap(true);
    for step in [0.1, 0.25, 0.5, 1.] {
        controller.set_grid_snap_step_mm(step);
        let input = plane.point([0.37 * step, 1.61 * step, 2.]).unwrap();
        let result = controller.constrain_point_with_base(input, false, None).unwrap();
        let local = plane.to_local(WorldPoint::try_new(result).unwrap()).unwrap().xyz();
        assert!(local[0].abs() < 1e-9 && (local[1] - 2. * step).abs() < 1e-9 && (local[2] - 2.).abs() < 1e-9);
    }
    controller.set_grid_snap(false);
    controller.set_ortho(true);
    controller.set_planar(true);
    let base = WorldPoint::try_new(plane.point([0., 0., 3.]).unwrap()).unwrap();
    let input = plane.point([1., 0.45, 0.]).unwrap();
    let result = controller.constrain_point_with_base(input, false, Some(base)).unwrap();
    let local = plane.to_local(WorldPoint::try_new(result).unwrap()).unwrap().xyz();
    assert!((local[0] - 0.725).abs() < 1e-9 && (local[1] - 0.725).abs() < 1e-9 && (local[2] - 3.).abs() < 1e-9);
    let shifted = controller.constrain_point_with_base(input, true, Some(base)).unwrap();
    let local = plane.to_local(WorldPoint::try_new(shifted).unwrap()).unwrap().xyz();
    assert!((local[0] - 1.).abs() < 1e-9 && (local[1] - 0.45).abs() < 1e-9 && (local[2] - 3.).abs() < 1e-9);

    controller.set_grid_snap(true);
    controller.set_grid_snap_step_mm(0.5);
    for (base, input, shift, expected) in [
        ([0., 0., 3.], [3.22, 1.29, 0.], false, [2.5, 2.5, 3.]),
        ([0., 0., 3.], [-3.22, -1.29, 0.], false, [-2.5, -2.5, 3.]),
        ([0.10, 0.26, 3.], [2.30, 1.50, 0.], false, [1.84, 2., 3.]),
        ([0.10, 0.26, 3.], [-2.29, 0.30, 0.], false, [-2.5, 0.26, 3.]),
        ([0., 0.26, 3.], [-2.25, 0.30, 0.], false, [-2., 0.26, 3.]),
        ([0., 0., 3.], [3.22, 1.29, 0.], true, [3., 1.5, 3.]),
    ] {
        let base = WorldPoint::try_new(plane.point(base).unwrap()).unwrap();
        let result = controller.constrain_point_with_base(plane.point(input).unwrap(), shift, Some(base)).unwrap();
        let local = plane.to_local(WorldPoint::try_new(result).unwrap()).unwrap().xyz();
        assert!(local.into_iter().zip(expected).all(|(actual, expected)| (actual - expected).abs() < 1e-9),
            "input={input:?}, expected={expected:?}, actual={local:?}");
    }
}

#[test]
fn standalone_grid_snap_half_step_matches_matrix9() {
    use om9_core::geometry::{Curve, Geometry};
    use om9_scene::camera::{CPlane, Camera, Viewport};

    let mut app = MatrixApp::new(Document::with_template_tolerance("Grid Snap half-step")).unwrap();
    let context = egui::Context::default();
    let view_id = ViewId::LookingDown;
    app.controller.workspace.active = view_id;
    app.set_object_snaps_enabled(false);
    app.set_smarttrack_enabled(false);
    app.controller.set_grid_snap(true);
    app.controller.set_grid_snap_step_mm(0.5);
    assert!(!app.controller.ortho() && !app.controller.planar() && !app.controller.project());
    for _ in 0..3 { frame(&mut app, &context, vec![]); }

    let rect = app.view_rects[&view_id];
    let viewport = Viewport {
        origin: [f64::from(rect.min.x), f64::from(rect.min.y)],
        size: [f64::from(rect.width()), f64::from(rect.height())],
    };
    let plane = CPlane::world_xy();
    // Eight pixels per millimetre makes the pick and its neighbors exactly representable.
    let camera = Camera::orthographic([0., 0., 30.], [0.; 3], [0., 1., 0.], viewport.size[1] / 8.).unwrap();
    let view = app.controller.workspace.views.get_mut(&view_id).unwrap();
    view.camera = camera.clone();
    view.cplane = plane;
    let project = |world| {
        let [x, y] = camera.project(world, viewport).unwrap().unwrap().screen;
        Pos2::new(x as f32, y as f32)
    };
    let before = app.controller.document().clone();
    for (raw_x, expected_x) in [(-2.375, -2.5), (-2.25, -2.), (-2.125, -2.), (2.25, 2.5)] {
        let raw = [raw_x, 0., 0.];
        let expected = [expected_x, 0., 0.];
        let cursor = project(raw);
        let pixel = [f64::from(cursor.x), f64::from(cursor.y)];
        let traced = camera.ray(pixel, viewport).unwrap().intersect_plane(plane).unwrap().unwrap();
        assert_eq!(traced, raw);

        app.controller.submit("Line").unwrap();
        app.controller.submit("0,0,0").unwrap();
        let output = render_frame(&mut app, &context, vec![Event::PointerMoved(cursor)]);
        assert!(output.shapes.iter().any(|shape| matches!(&shape.shape,
            egui::Shape::LineSegment { points, stroke }
                if stroke.color == egui::Color32::from_rgb(0, 130, 85)
                    && points[0].distance(project([0.; 3])) < 0.05
                    && points[1].distance(project(expected)) < 0.05
        )), "preview for raw X={raw_x} must reach X={expected_x}");
        click(&mut app, &context, cursor);
        let mut entities = app.controller.document().entities.values();
        let Geometry::Curve(Curve::Line { start, end, .. }) = entities.next().unwrap().geometry else {
            panic!("expected one Line after clicking raw X={raw_x}");
        };
        assert_eq!(start, [0.; 3]);
        assert_eq!(end, expected, "raw X={raw_x}");
        assert!(entities.next().is_none());
        assert_eq!(app.controller.coordinator().undo_len(), 1);
        app.controller.undo().unwrap();
        assert_eq!(app.controller.document(), &before);
    }
}

#[test]
fn combined_constraints_keep_line_preview_pick_and_typed_coordinates_consistent() {
    use om9_core::{geometry::{Curve, Geometry, Unit}, model::Entity};
    use om9_scene::camera::{CPlane, Camera, Viewport};

    let plane = CPlane::new([10., -3., 4.], [0., 1., 0.], [-1., 0., 0.]).unwrap();
    let source_point = plane.point([7.31, 3.27, 9.]).unwrap();
    let source = Entity::new("Point above CPlane", Geometry::Point(source_point), None);
    let source_id = source.id;
    let mut document = Document::with_template_tolerance("combined point constraints");
    document.page_unit = Unit::Inch;
    document.entities.insert(source_id, source);
    let mut app = MatrixApp::new(document).unwrap();
    let context = egui::Context::default();
    let command_view = ViewId::LookingDown;
    let pointer_view = ViewId::Perspective;
    app.controller.workspace.active = command_view;
    app.controller.workspace.views.get_mut(&command_view).unwrap().cplane = plane;
    let camera = Camera::orthographic([50., -40., 70.], plane.origin, [0., 0., 1.], 42.).unwrap();
    app.controller.workspace.views.get_mut(&pointer_view).unwrap().camera = camera.clone();
    app.set_snap_enabled(SnapKind::Point, true);
    app.set_smarttrack_enabled(false);
    app.controller.set_grid_snap(true);
    app.controller.set_grid_snap_step_mm(0.5);
    app.controller.set_ortho(true);
    frame(&mut app, &context, vec![]);
    frame(&mut app, &context, vec![]);

    let rect = app.view_rects[&pointer_view];
    let viewport = Viewport {
        origin: [rect.min.x as f64, rect.min.y as f64],
        size: [rect.width() as f64, rect.height() as f64],
    };
    let project = |world| {
        let [x, y] = camera.project(world, viewport).unwrap().unwrap().screen;
        Pos2::new(x as f32, y as f32)
    };
    let before = app.controller.document().clone();
    let start = plane.point([0., 0., 3.]).unwrap();
    let free_cursor = plane.point([3.22, 1.29, 3.]).unwrap();
    for (cursor, expected, snapped, planar, project_snap) in [
        (Some(source_point), plane.point([7.31, 3.27, 3.]).unwrap(), true, true, true),
        (Some(source_point), plane.point([7.31, 3.27, 0.]).unwrap(), true, false, true),
        (Some(source_point), source_point, true, true, false),
        (Some(source_point), source_point, true, false, false),
        (Some(free_cursor), plane.point([2.5, 2.5, 3.]).unwrap(), false, true, true),
        (None, plane.point([2.54, 1.27, 5.08]).unwrap(), false, true, true),
    ] {
        app.controller.workspace.active = command_view;
        app.controller.submit("Line").unwrap();
        app.controller.accept_world_point(start).unwrap();
        app.controller.set_planar(planar);
        app.controller.set_project(project_snap);
        if let Some(cursor) = cursor {
            let cursor = project(cursor);
            let output = render_frame(&mut app, &context, vec![Event::PointerMoved(cursor)]);
            assert_eq!(app.controller.workspace.active, command_view);
            assert_eq!(app.snap_candidate().map(|hit| hit.root), snapped.then_some(Some(source_id)));
            assert_eq!(app.snap_candidate().map(|hit| hit.world), snapped.then_some(source_point));
            let preview_end = project(expected);
            let preview_start = project(start);
            let marker_at_end = |output: &egui::FullOutput| {
                output.shapes.iter().any(|shape| matches!(&shape.shape,
                    egui::Shape::Rect(marker)
                        if marker.stroke.color == egui::Color32::from_rgb(255, 220, 90)
                            && marker.rect.center().distance(preview_end) < 0.05
                ))
            };
            assert!(output.shapes.iter().any(|shape| matches!(&shape.shape,
                egui::Shape::LineSegment { points, stroke }
                    if stroke.color == egui::Color32::from_rgb(0, 130, 85)
                        && points[0].distance(preview_start) < 0.05
                        && points[1].distance(preview_end) < 0.05
            )), "preview must reach {expected:?}");
            if snapped {
                if project_snap { assert!(cursor.distance(preview_end) > 2.); }
                assert!(marker_at_end(&output), "snap marker must move to the projected point");
                let outside = render_frame(&mut app, &context, vec![Event::PointerGone]);
                assert!(!marker_at_end(&outside), "leaving the viewport must hide the marker");
            }
            click(&mut app, &context, cursor);
        } else {
            app.focus_command_editor();
            frame(&mut app, &context, vec![]);
            frame(&mut app, &context, vec![Event::Text("0.1,0.05,0.2".into())]);
            frame(&mut app, &context, vec![Event::Key {
                key: Key::Enter, physical_key: None, pressed: true, repeat: false,
                modifiers: Modifiers::NONE,
            }]);
        }
        let line = app.controller.document().entities.values().find_map(|entity| match &entity.geometry {
            Geometry::Curve(Curve::Line { start, end, .. }) => Some((*start, *end)),
            _ => None,
        }).expect("Line after point input");
        assert_eq!(line.0, start);
        assert!(line.1.iter().zip(expected).all(|(actual, expected)| (actual - expected).abs() < 1e-6),
            "expected {expected:?}, got {:?}", line.1);
        assert_eq!(app.controller.coordinator().undo_len(), 1);
        app.controller.undo().unwrap();
        assert_eq!(app.controller.document(), &before);
    }
}

#[test]
fn f8_toggles_ortho_during_a_point_prompt() {
    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::with_template_tolerance("F8 Ortho")).unwrap();
    frame(&mut app, &context, vec![]);
    app.controller.submit("Line").unwrap();
    assert!(!app.controller.ortho());
    let key = |pressed| Event::Key { key: Key::F8, physical_key: None, pressed, repeat: false, modifiers: Modifiers::NONE };
    frame(&mut app, &context, vec![key(true)]);
    assert!(app.controller.ortho());
    frame(&mut app, &context, vec![key(false)]);
    frame(&mut app, &context, vec![key(true)]);
    assert!(!app.controller.ortho());
    assert!(app.controller.is_running());
}

#[test]
fn shift_inverts_ortho_during_click_and_planar_uses_the_accepted_height() {
    use om9_core::geometry::{Curve, Geometry};
    use om9_scene::camera::Viewport;

    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::with_template_tolerance("Shift and Planar")).unwrap();
    frame(&mut app, &context, vec![]);
    frame(&mut app, &context, vec![]);
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.set_ortho(true);
    app.set_object_snaps_enabled(false);
    let rect = app.view_rects[&ViewId::LookingDown];
    let viewport = Viewport { origin: [rect.min.x as f64, rect.min.y as f64],
        size: [rect.width() as f64, rect.height() as f64] };
    let screen = app.controller.workspace.views[&ViewId::LookingDown].camera
        .project([10., 4.5, 0.], viewport).unwrap().unwrap().screen;
    let cursor = Pos2::new(screen[0] as f32, screen[1] as f32);
    let base = [0., 0., 3.];
    let before = app.controller.document().clone();
    for (shift, planar, expected) in [
        (true, false, [10., 4.5, 0.]),
        (true, true, [10., 4.5, 3.]),
        (false, true, [7.25, 7.25, 3.]),
    ] {
        app.controller.set_planar(false);
        app.controller.submit("Line").unwrap();
        app.controller.accept_world_point(base).unwrap();
        app.controller.set_planar(planar);
        let modifiers = Modifiers { shift, ..Modifiers::NONE };
        for pressed in [true, false] {
            frame(&mut app, &context, vec![
                Event::ModifiersChanged(modifiers),
                Event::PointerMoved(cursor),
                Event::PointerButton { pos: cursor, button: PointerButton::Primary, pressed, modifiers },
            ]);
        }
        frame(&mut app, &context, vec![Event::ModifiersChanged(Modifiers::NONE)]);
        let (start, end) = app.controller.document().entities.values().find_map(|entity| match &entity.geometry {
            Geometry::Curve(Curve::Line { start, end, .. }) => Some((*start, *end)),
            _ => None,
        }).expect("viewport Line");
        assert_eq!(start, base);
        assert!((0..3).all(|axis| (end[axis] - expected[axis]).abs() < 1e-4), "{end:?}");
        assert!(app.controller.ortho(), "Shift only changes this pointer pick");
        assert_eq!(app.controller.coordinator().undo_len(), 1);
        app.controller.undo().unwrap();
        assert_eq!(app.controller.document(), &before);
    }
}

#[test]
fn parallel_view_miss_does_not_reuse_previous_view_point() {
    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::with_template_tolerance("cross-view point")).unwrap();
    for _ in 0..3 { frame(&mut app, &context, vec![]); }
    app.controller.workspace.active = ViewId::Perspective;
    app.controller.submit("Line").unwrap();
    let perspective = app.view_rects[&ViewId::Perspective].center();
    let before = app.controller.document().clone();
    let rect = app.view_rects[&ViewId::ThroughFinger];
    let viewport = om9_scene::camera::Viewport { origin: [rect.min.x as f64, rect.min.y as f64],
        size: [rect.width() as f64, rect.height() as f64] };
    let front = rect.center() + Vec2::new(40., -30.);
    let ray = app.controller.workspace.views[&ViewId::ThroughFinger].camera
        .ray([front.x as f64, front.y as f64], viewport).unwrap();
    assert!(ray.intersect_plane(app.controller.pointer_input_plane()).unwrap().is_none(),
        "the front-view ray is parallel to the captured top input plane");
    click(&mut app, &context, front);
    assert!(app.controller.prompt().starts_with("Start of line"));
    assert!(app.status_line().contains("command plane"), "{}", app.status_line());
    assert_eq!(app.controller.document(), &before);
    click(&mut app, &context, perspective);
    assert!(app.controller.prompt().starts_with("End of line"));

    click(&mut app, &context, front);
    assert!(app.controller.is_running(), "a parallel-ray miss must leave the Line prompt active");
    assert_eq!(app.controller.document(), &before, "the previous viewport point must not be reused");
    assert!(app.status_line().contains("command plane"), "{}", app.status_line());
    let top = app.view_rects[&ViewId::LookingDown].center() + Vec2::new(60., -30.);
    click(&mut app, &context, top);
    assert!(!app.controller.is_running());
    assert_eq!(app.controller.document().entities.len(), 1);
}

#[test]
fn tab_locks_pointer_direction_for_line_commit() {
    use om9_core::geometry::{Curve, Geometry};
    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::with_template_tolerance("Tab direction lock")).unwrap();
    for _ in 0..3 { frame(&mut app, &context, vec![]); }
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.submit("Line").unwrap();
    let origin = app.view_rects[&ViewId::LookingDown].center();
    click(&mut app, &context, origin);
    frame(&mut app, &context, vec![Event::PointerMoved(origin + Vec2::new(80., 0.))]);
    assert!(!context.egui_wants_keyboard_input(), "viewport should own keyboard before Tab; focus={:?}", context.memory(|m| m.focused()));
    frame(&mut app, &context, vec![Event::Key { key: Key::Tab, physical_key: None,
        pressed: true, repeat: false, modifiers: Modifiers::NONE }]);
    assert!(app.status_line().contains("Direction lock on"), "{}; focus={:?}; wants_keyboard={}", app.status_line(), context.memory(|m| m.focused()), context.egui_wants_keyboard_input());
    click(&mut app, &context, origin + Vec2::new(60., 60.));
    let (start, end) = app.controller.document().entities.values().find_map(|e| match &e.geometry {
        Geometry::Curve(Curve::Line { start, end, .. }) => Some((*start, *end)), _ => None,
    }).expect("locked line");
    assert!((end[1] - start[1]).abs() < 1e-7 && (end[0] - start[0]).abs() > 1.,
        "Tab must preserve the horizontal direction: {start:?} -> {end:?}");
}

#[test]
fn tab_does_not_lock_direction_when_command_editor_owns_keyboard() {
    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::with_template_tolerance("Tab editor focus")).unwrap();
    for _ in 0..3 { frame(&mut app, &context, vec![]); }
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.submit("Line").unwrap();
    let origin = app.view_rects[&ViewId::LookingDown].center();
    click(&mut app, &context, origin);
    frame(&mut app, &context, vec![Event::PointerMoved(origin + Vec2::new(80., 0.))]);
    app.focus_command_editor();
    frame(&mut app, &context, vec![]);
    assert!(context.egui_wants_keyboard_input());
    frame(&mut app, &context, vec![Event::Key { key: Key::Tab, physical_key: None,
        pressed: true, repeat: false, modifiers: Modifiers::NONE }]);
    assert!(!app.status_line().contains("Direction lock"), "{}", app.status_line());
}

#[test]
fn ctrl_click_elevator_accepts_typed_height_as_one_line_point() {
    use om9_core::geometry::{Curve, Geometry};
    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::with_template_tolerance("Elevator click")).unwrap();
    for _ in 0..3 { frame(&mut app, &context, vec![]); }
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.submit("Line").unwrap();
    let at = app.view_rects[&ViewId::LookingDown].center();
    frame(&mut app, &context, vec![Event::PointerMoved(at)]);
    let ctrl = Modifiers { ctrl: true, ..Modifiers::NONE };
    for pressed in [true, false] {
        let mut output = context.run_ui(egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 900.))),
            events: vec![Event::ModifiersChanged(ctrl), Event::PointerMoved(at), Event::PointerButton {
                pos: at, button: PointerButton::Primary, pressed, modifiers: ctrl,
            }], ..Default::default()
        }, |ui| app.show(ui));
        output.textures_delta.clear();
    }
    frame(&mut app, &context, vec![Event::ModifiersChanged(Modifiers::NONE)]);
    assert!(app.status_line().contains("Elevator"), "{}", app.status_line());
    assert!(app.controller.document().entities.is_empty());
    assert!(app.transcript.history().contains("Elevator: height:\n"));
    app.focus_command_editor();
    frame(&mut app, &context, vec![]);
    let submit = |app: &mut MatrixApp, input: &str| {
        app.transcript.set_draft(input);
        for pressed in [true, false] {
            frame(app, &context, vec![Event::Key { key: Key::Enter, physical_key: None,
                pressed, repeat: false, modifiers: Modifiers::NONE }]);
        }
    };
    submit(&mut app, "wrong");
    assert!(app.transcript.history().contains("Elevator: height: wrong\n"));
    assert!(app.status_line().contains("invalid Elevator height"));
    assert!(app.transcript.prompt().starts_with("Elevator"));
    submit(&mut app, "3");
    assert!(app.controller.is_running());
    assert!(app.controller.document().entities.is_empty());
    submit(&mut app, "10,10,3");
    let history = app.transcript.history();
    let height = history.find("Elevator: height: 3\n").expect("accepted height in CMD history");
    assert!(history[height..].lines().any(|line| line.starts_with("End of line") && line.ends_with("10,10,3")), "{history}");
    let (start, end) = app.controller.document().entities.values().find_map(|e| match &e.geometry {
        Geometry::Curve(Curve::Line { start, end, .. }) => Some((*start, *end)), _ => None,
    }).expect("line from Elevator point");
    assert!((start[2] - 3.).abs() < 1e-7 && end == [10., 10., 3.], "{start:?} -> {end:?}");
    app.controller.undo().unwrap();
    assert!(app.controller.document().entities.is_empty());
}

#[test]
fn elevator_height_click_in_side_view_returns_keyboard_to_viewport() {
    use om9_core::{geometry::{Curve, Geometry}, model::Entity};
    use om9_scene::camera::Viewport;

    let context = egui::Context::default();
    let end = [8., 8., 7.];
    let reference = Entity::new("End reference", Geometry::Curve(Curve::Line {
        start: end,
        end: [8., 12., 7.],
        domain: [0., 1.],
    }), None);
    let reference_id = reference.id;
    let mut document = Document::with_template_tolerance("Elevator cross-view height");
    document.entities.insert(reference.id, reference);
    let mut app = MatrixApp::new(document).unwrap();
    app.controller.workspace.active = ViewId::LookingDown;
    app.set_object_snaps_enabled(false);
    for _ in 0..3 {
        frame(&mut app, &context, vec![]);
    }
    let before = app.controller.document().clone();
    let project = |app: &MatrixApp, view, point| {
        let rect = app.view_rects[&view];
        let viewport = Viewport {
            origin: [rect.min.x as f64, rect.min.y as f64],
            size: [rect.width() as f64, rect.height() as f64],
        };
        let [x, y] = app.controller.workspace.views[&view].camera
            .project(point, viewport).unwrap().unwrap().screen;
        Pos2::new(x as f32, y as f32)
    };

    app.controller.submit("Line").unwrap();
    let at = project(&app, ViewId::LookingDown, [2., 3., 0.]);
    let ctrl = Modifiers { ctrl: true, ..Modifiers::NONE };
    for pressed in [true, false] {
        frame(&mut app, &context, vec![
            Event::ModifiersChanged(ctrl),
            Event::PointerMoved(at),
            Event::PointerButton { pos: at, button: PointerButton::Primary, pressed, modifiers: ctrl },
        ]);
    }
    frame(&mut app, &context, vec![Event::ModifiersChanged(Modifiers::NONE)]);
    assert!(app.transcript.prompt().starts_with("Elevator"));
    assert!(context.memory(|memory| memory.has_focus(egui::Id::new("matrix-command-transcript"))));

    click(&mut app, &context, at + Vec2::new(20., 10.));
    assert!(app.transcript.prompt().starts_with("Elevator"));
    assert!(app.controller.point_constraint_base().is_none());
    let height = project(&app, ViewId::SideView, [2., 3., 5.]);
    click(&mut app, &context, height);
    assert!(app.controller.prompt().starts_with("End of line"));
    let history = app.transcript.history();
    let logged_height: f64 = history.lines().find_map(|line| line.strip_prefix("Elevator: height: "))
        .unwrap_or_else(|| panic!("missing picked height in {history}")).parse().unwrap();
    assert!((logged_height - 5.).abs() < 1e-5, "{history}");
    assert_eq!(app.controller.document(), &before);
    assert_eq!(app.controller.coordinator().undo_len(), 0);
    let start = app.controller.point_constraint_base().unwrap().xyz();
    assert!(start.iter().zip([2., 3., 5.]).all(|(actual, expected)| (actual - expected).abs() < 1e-5));

    frame(&mut app, &context, vec![
        Event::Key { key: Key::E, physical_key: None, pressed: true, repeat: false, modifiers: Modifiers::NONE },
        Event::Text("e".into()),
    ]);
    assert_eq!(app.snap_override(), Some(SnapKind::End));
    assert_eq!(app.transcript.draft(), "");
    let at = project(&app, ViewId::SideView, end);
    click(&mut app, &context, at);
    let after = app.controller.document().clone();
    let line = after.entities.values().find(|entity| entity.id != reference_id).unwrap();
    let Geometry::Curve(Curve::Line { start: committed_start, end: committed_end, .. }) = &line.geometry else {
        panic!("expected Line after Elevator");
    };
    assert_eq!((*committed_start, *committed_end), (start, end));
    assert_eq!(app.controller.coordinator().undo_len(), 1);
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &before);
    app.controller.redo().unwrap();
    assert_eq!(app.controller.document(), &after);
}

#[test]
fn escape_cancels_elevator_without_consuming_the_line_point() {
    use om9_core::geometry::{Curve, Geometry};
    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::with_template_tolerance("cancel Elevator")).unwrap();
    for _ in 0..3 { frame(&mut app, &context, vec![]); }
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.submit("Line").unwrap();
    let at = app.view_rects[&ViewId::LookingDown].center();
    frame(&mut app, &context, vec![Event::PointerMoved(at)]);
    let ctrl = Modifiers { ctrl: true, ..Modifiers::NONE };
    for pressed in [true, false] {
        let mut output = context.run_ui(egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 900.))),
            events: vec![Event::ModifiersChanged(ctrl), Event::PointerMoved(at), Event::PointerButton {
                pos: at, button: PointerButton::Primary, pressed, modifiers: ctrl,
            }], ..Default::default()
        }, |ui| app.show(ui));
        output.textures_delta.clear();
    }
    frame(&mut app, &context, vec![Event::ModifiersChanged(Modifiers::NONE)]);
    assert!(app.status_line().contains("Elevator"), "{}", app.status_line());
    frame(&mut app, &context, vec![Event::Key { key: Key::Escape, physical_key: None,
        pressed: true, repeat: false, modifiers: Modifiers::NONE }]);
    assert!(app.controller.is_running(), "Escape must leave the outer Line active");
    assert!(app.controller.document().entities.is_empty());
    assert!(app.transcript.history().contains("Elevator cancelled\n"));
    app.controller.submit("1,2,3").unwrap();
    assert!(app.controller.document().entities.is_empty(), "first point must not finish Line");
    app.controller.submit("4,5,6").unwrap();
    let (start, end) = app.controller.document().entities.values().find_map(|e| match &e.geometry {
        Geometry::Curve(Curve::Line { start, end, .. }) => Some((*start, *end)), _ => None,
    }).expect("Line after cancelling Elevator");
    assert_eq!((start, end), ([1., 2., 3.], [4., 5., 6.]));
    app.controller.undo().unwrap();
    assert!(app.controller.document().entities.is_empty());
}

#[test]
fn cmd_between_commits_only_the_outer_line_and_undoes_once() {
    use om9_core::geometry::{Curve, Geometry};
    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::with_template_tolerance("Between CMD")).unwrap();
    frame(&mut app, &context, vec![]);
    app.focus_command_editor();
    frame(&mut app, &context, vec![]);
    for text in ["Line", "Between", "0,0,0", "2,2,0", "4,1,0"] {
        frame(&mut app, &context, vec![Event::Text(text.into())]);
        frame(&mut app, &context, vec![Event::Key {
            key: Key::Enter, physical_key: None, pressed: true, repeat: false, modifiers: Modifiers::NONE,
        }]);
    }
    assert_eq!(app.controller.document().entities.len(), 1);
    let entity = app.controller.document().entities.values().next().unwrap();
    let Geometry::Curve(Curve::Line { start, end, .. }) = &entity.geometry else { panic!("expected line") };
    assert_eq!((*start, *end), ([1., 1., 0.], [4., 1., 0.]));
    assert_eq!(app.controller.coordinator().undo_len(), 1);
    app.controller.undo().unwrap();
    assert!(app.controller.document().entities.is_empty());
    app.controller.redo().unwrap();
    assert_eq!(app.controller.document().entities.len(), 1);
}

#[test]
fn between_viewport_picks_keep_end_shortcut_and_commit_only_midpoint() {
    use om9_core::{geometry::{Curve, Geometry}, model::Entity};
    use om9_scene::camera::Viewport;

    let context = egui::Context::default();
    let first = [-6., 0., 0.];
    let second = [4., 6., 0.];
    let mut document = Document::with_template_tolerance("Between End references");
    for start in [first, second] {
        let entity = Entity::new("End reference", Geometry::Curve(Curve::Line {
            start,
            end: [start[0], start[1] + 5., 0.],
            domain: [0., 1.],
        }), None);
        document.entities.insert(entity.id, entity);
    }
    let mut app = MatrixApp::new(document).unwrap();
    app.controller.workspace.active = ViewId::LookingDown;
    app.set_object_snaps_enabled(false);
    let before = app.controller.document().clone();
    let key = |key| Event::Key {
        key, physical_key: None, pressed: true, repeat: false, modifiers: Modifiers::NONE,
    };
    let submit = |app: &mut MatrixApp, text: &str| {
        app.focus_command_editor();
        frame(app, &context, vec![]);
        frame(app, &context, vec![Event::Text(text.into())]);
        frame(app, &context, vec![key(Key::Enter)]);
    };
    frame(&mut app, &context, vec![]);
    frame(&mut app, &context, vec![]);
    submit(&mut app, "Line");
    submit(&mut app, "Between");
    submit(&mut app, "End");
    assert_eq!(app.snap_override(), Some(SnapKind::End));
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

    click(&mut app, &context, project(first));
    assert!(app.transcript.prompt().contains("Between: second point"));
    assert!(app.controller.prompt().starts_with("Start of line"));
    assert_eq!(app.snap_override(), None);
    assert_eq!(app.controller.document(), &before);
    frame(&mut app, &context, vec![key(Key::E), Event::Text("e".into())]);
    assert_eq!(app.snap_override(), Some(SnapKind::End));
    assert_eq!(app.transcript.draft(), "");

    click(&mut app, &context, project(second));
    assert!(app.controller.prompt().starts_with("End of line"));
    assert_eq!(app.snap_override(), None);
    assert_eq!(app.controller.document(), &before);
    assert_eq!(app.controller.coordinator().undo_len(), 0);
    assert!(!context.memory(|memory| memory.has_focus(egui::Id::new("matrix-command-transcript"))));
    submit(&mut app, "0,7,1");
    let after = app.controller.document().clone();
    let line = after.entities.values().find(|entity| !before.entities.contains_key(&entity.id)).unwrap();
    let Geometry::Curve(Curve::Line { start, end, .. }) = &line.geometry else { panic!("expected line") };
    assert_eq!((*start, *end), ([-1., 3., 0.], [0., 7., 1.]));
    assert_eq!(app.controller.coordinator().undo_len(), 1);
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &before);
    app.controller.redo().unwrap();
    assert_eq!(app.controller.document(), &after);
}

#[cfg(feature = "occt-brep")]
#[test]
fn deferred_nurbs_end_preview_matches_line_commit_and_undo() {
    use om9_core::{geometry::{Curve, Geometry, NurbsCurve}, model::Entity};
    use om9_scene::camera::Viewport;
    let context = egui::Context::default();
    let mut document = Document::with_template_tolerance("deferred NURBS endpoint");
    let source = Entity::new("source", Geometry::Curve(Curve::Nurbs(NurbsCurve {
        degree: 2, poles: vec![[0., 0., 0.], [2., 3., 0.], [4., 0., 0.]],
        weights: vec![1.; 3], knots: vec![0., 0., 0., 1., 1., 1.], domain: [0., 1.],
        periodic: false, reversed: false, seam: None,
    })), None);
    let source_id = source.id;
    document.entities.insert(source_id, source);
    let mut app = MatrixApp::new(document).unwrap();
    frame(&mut app, &context, vec![]);
    frame(&mut app, &context, vec![]);
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.submit("Line").unwrap();
    app.set_snap_override(Some(SnapKind::End));
    let rect = app.view_rects[&ViewId::LookingDown];
    let viewport = Viewport { origin: [rect.min.x as f64, rect.min.y as f64],
        size: [rect.width() as f64, rect.height() as f64] };
    let projected = app.controller.workspace.views[&ViewId::LookingDown].camera
        .project([0., 0., 0.], viewport).unwrap().unwrap().screen;
    let cursor = Pos2::new(projected[0] as f32, projected[1] as f32);
    frame(&mut app, &context, vec![Event::PointerMoved(cursor)]);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
    let preview = loop {
        if let Some(candidate) = app.snap_candidate().filter(|candidate| candidate.kind == SnapKind::End) {
            break candidate.clone();
        }
        assert!(std::time::Instant::now() < deadline, "deferred endpoint: {}", app.status_line());
        std::thread::sleep(std::time::Duration::from_millis(5));
        frame(&mut app, &context, vec![]);
    };
    assert_eq!(preview.root, Some(source_id));
    assert_eq!(preview.world, [0., 0., 0.]);
    let before = app.controller.document().clone();
    click(&mut app, &context, cursor);
    app.controller.submit("10,10,0").unwrap();
    let line = app.controller.document().entities.values().find_map(|entity| match &entity.geometry {
        Geometry::Curve(Curve::Line { start, end, .. }) => Some((*start, *end)), _ => None,
    }).expect("outer line");
    assert_eq!(line, (preview.world, [10., 10., 0.]));
    assert_eq!(app.controller.coordinator().undo_len(), 1);
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &before);
    app.controller.redo().unwrap();
    assert_eq!(app.controller.document().entities.len(), 2);
}

#[cfg(feature = "occt-brep")]
#[test]
fn on_surface_target_uses_source_point_for_line_commit() {
    use om9_core::{geometry::{Curve, Geometry}, model::Entity, surface_kernel};
    use om9_scene::camera::Viewport;
    let context = egui::Context::default();
    let limits = Limits::default();
    let surface = surface_kernel::plane([0., 0., 5.], [1., 0., 0.], [0., 1., 0.], 10., 10., &limits).unwrap();
    let mut document = Document::with_template_tolerance("surface target");
    let source = Entity::new("surface", Geometry::Surface(surface), None);
    let source_id = source.id;
    document.entities.insert(source_id, source);
    let mut app = MatrixApp::new(document).unwrap();
    frame(&mut app, &context, vec![]);
    frame(&mut app, &context, vec![]);
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.submit("Line").unwrap();
    app.focus_command_editor();
    frame(&mut app, &context, vec![]);
    frame(&mut app, &context, vec![Event::Text("OnSurface".into())]);
    frame(&mut app, &context, vec![Event::Key { key: Key::Enter, physical_key: None,
        pressed: true, repeat: false, modifiers: Modifiers::NONE }]);
    let rect = app.view_rects[&ViewId::LookingDown];
    let viewport = Viewport { origin: [rect.min.x as f64, rect.min.y as f64],
        size: [rect.width() as f64, rect.height() as f64] };
    let camera = app.controller.workspace.views[&ViewId::LookingDown].camera.clone();
    let screen = |point| {
        let projected = camera.project(point, viewport).unwrap().unwrap().screen;
        Pos2::new(projected[0] as f32, projected[1] as f32)
    };
    let wait_for_surface = |app: &mut MatrixApp, point: Pos2| {
        frame(app, &context, vec![Event::PointerMoved(point)]);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
        loop {
            if let Some(candidate) = app.snap_candidate().filter(|candidate| candidate.kind == SnapKind::Surface) {
                break candidate.clone();
            }
            assert!(std::time::Instant::now() < deadline, "surface snap: {}", app.status_line());
            std::thread::sleep(std::time::Duration::from_millis(5));
            frame(app, &context, vec![]);
        }
    };
    let target_cursor = screen([2., 2., 5.]);
    let target = wait_for_surface(&mut app, target_cursor);
    assert_eq!(target.root, Some(source_id));
    click(&mut app, &context, target_cursor);
    assert!(app.controller.relative_base().is_none(), "target pick must not supply the Line start");
    let cursor = screen([4., 5., 5.]);
    let point = wait_for_surface(&mut app, cursor);
    assert_eq!(point.root, Some(source_id));
    assert!((point.world[2] - 5.).abs() < 0.001);
    click(&mut app, &context, cursor);
    click(&mut app, &context, screen([14., 12., 0.]));
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
    let (start, end) = loop {
        if let Some(line) = app.controller.document().entities.values().find_map(|entity| match &entity.geometry {
            Geometry::Curve(Curve::Line { start, end, .. }) => Some((*start, *end)), _ => None,
        }) { break line; }
        assert!(std::time::Instant::now() < deadline, "one-shot surface did not release: {}", app.status_line());
        std::thread::sleep(std::time::Duration::from_millis(5));
        frame(&mut app, &context, vec![]);
    };
    assert!((0..3).all(|i| (start[i] - point.world[i]).abs() <= 0.001));
    assert!(end[0] > 10. && end[1] > 10. && end[2].abs() < 0.001, "free second point {end:?}");
    assert_eq!(app.controller.coordinator().undo_len(), 1);
}

#[cfg(feature = "occt-brep")]
#[test]
fn project_on_surface_checks_captured_plane_before_committing_side_view_pick() {
    use om9_core::{budget::memory::MemoryClass, geometry::{Curve, Geometry}, model::Entity, surface_kernel};
    use om9_scene::camera::Viewport;

    for (lower_z, projection_is_on_surface) in [(-2., true), (2., false)] {
        let context = egui::Context::default();
        let surface = surface_kernel::plane(
            [3., -4., lower_z], [0., 1., 0.], [0., 0., 1.], 8., 8., &Limits::default(),
        ).unwrap();
        let target = Entity::new("vertical surface", Geometry::Surface(surface), None);
        let target_id = target.id;
        let mut document = Document::with_template_tolerance("Project surface witness");
        document.entities.insert(target_id, target);
        let mut app = MatrixApp::new(document).unwrap();
        app.controller.workspace.active = ViewId::LookingDown;
        app.controller.set_project(true);
        for _ in 0..3 {
            frame(&mut app, &context, vec![]);
        }
        let before = app.controller.document().clone();
        app.controller.submit("Line").unwrap();
        app.focus_command_editor();
        frame(&mut app, &context, vec![]);
        frame(&mut app, &context, vec![Event::Text("OnSurface".into())]);
        frame(&mut app, &context, vec![Event::Key {
            key: Key::Enter, physical_key: None, pressed: true, repeat: false, modifiers: Modifiers::NONE,
        }]);
        let rect = app.view_rects[&ViewId::SideView];
        let viewport = Viewport {
            origin: [rect.min.x as f64, rect.min.y as f64],
            size: [rect.width() as f64, rect.height() as f64],
        };
        let [x, y] = app.controller.workspace.views[&ViewId::SideView].camera
            .project([3., -2., 3.], viewport).unwrap().unwrap().screen;
        let at = Pos2::new(x as f32, y as f32);
        let step_until = |app: &mut MatrixApp, done: &dyn Fn(&MatrixApp) -> bool| {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
            while !done(app) {
                assert!(std::time::Instant::now() < deadline, "{}", app.status_line());
                std::thread::sleep(std::time::Duration::from_millis(5));
                frame(app, &context, vec![]);
            }
        };
        frame(&mut app, &context, vec![Event::PointerMoved(at)]);
        step_until(&mut app, &|app| app.snap_candidate().is_some_and(|snap| snap.root == Some(target_id)));
        assert!((app.snap_candidate().unwrap().world[2] - 3.).abs() < 1e-5);
        click(&mut app, &context, at);
        assert!(app.controller.point_constraint_base().is_none());
        assert!(app.controller.input_plane().normal()[2] > 0.99);

        let kernel_before = app.controller.coordinator().external_reserved_bytes(MemoryClass::Kernel);
        let permit = app.controller.coordinator().external_memory_account().try_acquire_surface_snap().unwrap();
        click(&mut app, &context, at);
        assert!(app.controller.coordinator().external_reserved_bytes(MemoryClass::Kernel) > kernel_before);
        assert!(app.controller.point_constraint_base().is_none());
        drop(permit);
        if projection_is_on_surface {
            step_until(&mut app, &|app| app.controller.prompt().starts_with("End of line"));
        } else {
            step_until(&mut app, &|app| app.status_line().contains("Error: unsupported could not verify Project on the selected surface"));
            assert!(app.controller.prompt().starts_with("Start of line"));
            assert!(app.controller.point_constraint_base().is_none());
            assert!(!context.memory(|memory| memory.has_focus(egui::Id::new("matrix-command-transcript"))));
            assert_eq!(app.controller.document(), &before);
            app.controller.set_project(false);
            step_until(&mut app, &|app| app.snap_candidate().is_some_and(|snap| snap.root == Some(target_id)));
            click(&mut app, &context, at);
            assert!(app.controller.prompt().starts_with("End of line"));
        }
        let start = app.controller.point_constraint_base().unwrap().xyz();
        let expected = [3., -2., if projection_is_on_surface { 0. } else { 3. }];
        assert!(start.iter().zip(expected).all(|(a, b)| (a - b).abs() < 1e-5), "{start:?}");
        assert_eq!(app.controller.document(), &before);
        app.controller.submit("10,10,1").unwrap();
        assert_eq!(app.controller.coordinator().undo_len(), 1);
        assert!(app.controller.document().entities.values().any(|entity| {
            matches!(&entity.geometry, Geometry::Curve(Curve::Line { start: accepted, end, .. })
                if *accepted == start && *end == [10., 10., 1.])
        }));
        app.controller.undo().unwrap();
        assert_eq!(app.controller.document(), &before);
    }
}

#[cfg(feature = "occt-brep")]
#[test]
fn typed_persistent_polysurface_points_survive_enter_release() {
    use om9_core::{budget::memory::MemoryClass, geometry::{Curve, Geometry}};
    use om9_scene::camera::Viewport;

    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::with_template_tolerance("typed polysurface points")).unwrap();
    app.controller.submit("Box 10 10 10").unwrap();
    let before = app.controller.document().clone();
    let target = *before.entities.keys().next().unwrap();
    let undo = app.controller.coordinator().undo_len();
    app.controller.workspace.active = ViewId::LookingDown;
    frame(&mut app, &context, vec![]);
    frame(&mut app, &context, vec![]);
    app.controller.submit("Line").unwrap();
    let enter = |pressed| Event::Key {
        key: Key::Enter, physical_key: None, pressed, repeat: false, modifiers: Modifiers::NONE,
    };
    let press_enter = |app: &mut MatrixApp, text| {
        app.transcript.set_draft(text);
        app.focus_command_editor();
        frame(app, &context, vec![]);
        frame(app, &context, vec![enter(true)]);
    };
    let submit_text = |app: &mut MatrixApp, text| {
        press_enter(app, text);
        frame(app, &context, vec![enter(false)]);
    };
    submit_text(&mut app, "PersistentOnPolysurface");

    let rect = app.view_rects[&ViewId::LookingDown];
    let viewport = Viewport {
        origin: [rect.min.x as f64, rect.min.y as f64],
        size: [rect.width() as f64, rect.height() as f64],
    };
    let [x, y] = app.controller.workspace.views[&ViewId::LookingDown].camera
        .project([4., 4., 10.], viewport).unwrap().unwrap().screen;
    let at = Pos2::new(x as f32, y as f32);
    let wait_for = |app: &mut MatrixApp, predicate: &dyn Fn(&MatrixApp) -> bool| {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(8);
        while !predicate(app) {
            assert!(std::time::Instant::now() < deadline, "{}", app.status_line());
            std::thread::sleep(std::time::Duration::from_millis(5));
            frame(app, &context, vec![]);
        }
    };
    let pick_target = |app: &mut MatrixApp| {
        frame(app, &context, vec![Event::PointerMoved(at)]);
        wait_for(app, &|app| app.snap_candidate().is_some_and(|hit| {
            hit.kind == SnapKind::Surface && hit.root == Some(target)
        }));
        click(app, &context, at);
        assert!(app.controller.prompt().starts_with("Start of line"));
    };
    pick_target(&mut app);
    assert_eq!(app.controller.document(), &before);

    let initial_reservation = app.controller.coordinator().external_reserved_bytes(MemoryClass::Kernel);
    let permit = app.controller.coordinator().external_memory_account().try_acquire_surface_snap().unwrap();
    press_enter(&mut app, "4,4,5");
    assert!(app.controller.coordinator().external_reserved_bytes(MemoryClass::Kernel) > initial_reservation);
    assert!(app.controller.prompt().starts_with("Start of line"));
    frame(&mut app, &context, vec![enter(false)]);
    drop(permit);
    wait_for(&mut app, &|app| app.status_line().contains("could not verify typed point on selected surface"));
    assert_eq!(app.controller.document(), &before);

    submit_text(&mut app, "4,4,10");
    wait_for(&mut app, &|app| app.controller.prompt().starts_with("End of line"));
    assert_eq!(app.controller.coordinator().undo_len(), undo);
    submit_text(&mut app, "4,4,0");
    wait_for(&mut app, &|app| !app.controller.is_running());
    let line = app.controller.document().entities.values().find_map(|entity| match &entity.geometry {
        Geometry::Curve(Curve::Line { start, end, .. }) => Some((*start, *end)),
        _ => None,
    }).expect("completed Line");
    assert_eq!(line, ([4., 4., 10.], [4., 4., 0.]));
    assert_eq!(app.controller.coordinator().undo_len(), undo + 1);
    let finished = app.controller.document().clone();
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &before);
    app.controller.redo().unwrap();
    assert_eq!(app.controller.document(), &finished);

    app.controller.submit("Line").unwrap();
    submit_text(&mut app, "PersistentOnPolysurface");
    pick_target(&mut app);
    let before_worker = app.controller.coordinator().external_reserved_bytes(MemoryClass::Kernel);
    let permit = app.controller.coordinator().external_memory_account().try_acquire_surface_snap().unwrap();
    submit_text(&mut app, "4,4,10");
    assert!(app.controller.coordinator().external_reserved_bytes(MemoryClass::Kernel) > before_worker);
    assert_eq!(app.controller.delete_objects(&[target]).unwrap(), 1);
    let after_delete = app.controller.coordinator().snapshot();
    frame(&mut app, &context, vec![]);
    wait_for(&mut app, &|app| app.controller.coordinator().external_reserved_bytes(MemoryClass::Kernel) == 0);
    drop(permit);
    assert_eq!(app.controller.coordinator().snapshot(), after_delete);
    assert!(!app.controller.prompt().starts_with("End of line"), "deleted target must not supply the Line start");
}

#[cfg(feature = "occt-brep")]
#[test]
fn on_surface_uses_exact_sphere_source_for_preview_and_line() {
    use om9_core::geometry::{Curve, Geometry};
    use om9_scene::camera::Viewport;
    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::with_template_tolerance("sphere surface target")).unwrap();
    app.controller.solid_sphere_points([0.; 3], 5.).unwrap();
    let source_id = app.controller.document().entities.values()
        .find(|entity| matches!(entity.geometry, Geometry::Brep(_))).unwrap().id;
    let scene = app.controller.scene().unwrap();
    let object = scene.objects.iter().find(|object| object.root == source_id).unwrap();
    scene.capture_brep_source_for(app.controller.coordinator(), object).unwrap();
    let handle = app.controller.document().entities[&source_id].geometry_handle();
    assert!(matches!(app.controller.coordinator().capture_brep_occurrence_source(
        &object.source, &object.path, handle, [0; 32]), Err(om9_core::Error::Stale(_))));
    let mut stale = handle;
    stale.geometry_revision += 1;
    let om9_scene::scene::BrepDisplayReceipt::Occt(binding) = object.brep_display_fingerprint.as_ref().unwrap() else { panic!("BRep source receipt") };
    assert!(matches!(app.controller.coordinator().capture_brep_occurrence_source(
        &object.source, &object.path, stale, *binding), Err(om9_core::Error::Stale(_))));
    let before = app.controller.document().clone();
    let undo_before = app.controller.coordinator().undo_len();
    frame(&mut app, &context, vec![]);
    frame(&mut app, &context, vec![]);
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.submit("Line").unwrap();
    app.focus_command_editor();
    frame(&mut app, &context, vec![]);
    frame(&mut app, &context, vec![Event::Text("OnSurface".into())]);
    frame(&mut app, &context, vec![Event::Key { key: Key::Enter, physical_key: None,
        pressed: true, repeat: false, modifiers: Modifiers::NONE }]);
    let rect = app.view_rects[&ViewId::LookingDown];
    let viewport = Viewport { origin: [rect.min.x as f64, rect.min.y as f64],
        size: [rect.width() as f64, rect.height() as f64] };
    let camera = app.controller.workspace.views[&ViewId::LookingDown].camera.clone();
    let screen = |point| {
        let p = camera.project(point, viewport).unwrap().unwrap().screen;
        Pos2::new(p[0] as f32, p[1] as f32)
    };
    let wait_for_surface = |app: &mut MatrixApp, at: Pos2| {
        frame(app, &context, vec![Event::PointerMoved(at)]);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
        loop {
            if let Some(candidate) = app.snap_candidate().filter(|candidate| candidate.kind == SnapKind::Surface) {
                break candidate.clone();
            }
            assert!(std::time::Instant::now() < deadline, "sphere surface: {}", app.status_line());
            std::thread::sleep(std::time::Duration::from_millis(5));
            frame(app, &context, vec![]);
        }
    };
    let target_cursor = screen([1., 1., 23_f64.sqrt()]);
    let target = wait_for_surface(&mut app, target_cursor);
    assert_eq!(target.root, Some(source_id));
    click(&mut app, &context, target_cursor);
    assert!(app.controller.relative_base().is_none());
    let cursor = screen([2., 1., 20_f64.sqrt()]);
    let preview = wait_for_surface(&mut app, cursor);
    assert_eq!(preview.root, Some(source_id));
    let radius = preview.world.iter().map(|value| value * value).sum::<f64>().sqrt();
    assert!((radius - 5.).abs() < 1e-6, "source radius {radius}");
    click(&mut app, &context, cursor);
    app.controller.submit("10,10,0").unwrap();
    let start = app.controller.document().entities.values().find_map(|entity| match &entity.geometry {
        Geometry::Curve(Curve::Line { start, .. }) => Some(*start), _ => None,
    }).expect("outer line");
    assert!((0..3).all(|i| (start[i] - preview.world[i]).abs() < 1e-6));
    assert_eq!(app.controller.coordinator().undo_len(), undo_before + 1);
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &before);
    app.controller.redo().unwrap();
    assert_eq!(app.controller.document().entities.len(), 2);
}

#[cfg(feature = "occt-brep")]
#[test]
fn reopened_trimmed_cylinder_surface_override_skips_front_hole() {
    use om9_core::{geometry::{Curve, Geometry}, model::Entity, rhino_semantics::{set_trimmed_surface, TrimmedSurface}};
    use om9_scene::{camera::{Camera, Viewport}, workspace::DisplayMode};
    let limits = Limits::default();
    let context = egui::Context::default();
    let surface = om9_core::surface_kernel::cylinder([0., 0., 0.], [1., 0., 0.], [0., 1., 0.], 2., 5., &limits).unwrap();
    let trim = TrimmedSurface { underlying: surface.clone(), outer: vec![[0., 0.], [1., 0.], [1., 1.], [0., 1.]],
        inners: vec![vec![[0.01, 0.4], [0.08, 0.4], [0.08, 0.6], [0.01, 0.6]]], reversed: false };
    let mut entity = Entity::new("trim cylinder", Geometry::Surface(surface), None);
    set_trimmed_surface(&mut entity, &trim, &limits).unwrap();
    let mut doc = Document::with_template_tolerance("trimmed snap line");
    doc.entities.insert(entity.id, entity);
    let path = std::env::temp_dir().join(format!("om9-trim-snap-line-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, &doc, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    let mut app = MatrixApp::from_file(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    for _ in 0..3 { frame(&mut app, &context, vec![]); }
    app.controller.workspace.active = ViewId::LookingDown;
    let view = app.controller.workspace.views.get_mut(&ViewId::LookingDown).unwrap();
    view.camera = Camera::orthographic([10., 0.4, 2.5], [0., 0.4, 2.5], [0., 0., 1.], 8.).unwrap();
    view.mode = DisplayMode::Shaded;
    frame(&mut app, &context, vec![]);
    let before = app.controller.document().clone();
    app.controller.submit("Line").unwrap();
    app.set_snap_override(Some(SnapKind::Surface));
    let rect = app.view_rects[&ViewId::LookingDown];
    let viewport = Viewport { origin: [rect.min.x as f64, rect.min.y as f64], size: [rect.width() as f64, rect.height() as f64] };
    let project = |app: &MatrixApp, world| {
        let p = app.controller.workspace.views[&ViewId::LookingDown].camera.project(world, viewport).unwrap().unwrap().screen;
        Pos2::new(p[0] as f32, p[1] as f32)
    };
    let point = project(&app, [0., 0.4, 2.5]);
    frame(&mut app, &context, vec![Event::PointerMoved(point)]);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
    let snap = loop {
        if let Some(snap) = app.snap_candidate().filter(|snap| snap.kind == SnapKind::Surface) { break snap.world; }
        assert!(std::time::Instant::now() < deadline, "trimmed rear snap: {}", app.status_line());
        std::thread::sleep(std::time::Duration::from_millis(5));
        frame(&mut app, &context, vec![]);
    };
    assert!(snap[0] < -1.9, "expected rear surface: {snap:?}");
    let miss = project(&app, [0., 3.5, 2.5]);
    frame(&mut app, &context, vec![Event::PointerMoved(miss)]);
    assert!(app.snap_candidate().is_none());
    click(&mut app, &context, miss);
    assert_eq!(app.controller.document(), &before, "miss must not commit a stale point");
    frame(&mut app, &context, vec![Event::PointerMoved(point)]);
    click(&mut app, &context, point);
    app.controller.submit("-3,2,2.5").unwrap();
    let start = app.controller.document().entities.values().find_map(|e| match &e.geometry {
        Geometry::Curve(Curve::Line { start, .. }) => Some(*start), _ => None,
    }).expect("created line");
    assert!(start.iter().zip(snap).all(|(a, b)| (a - b).abs() < 1e-7));
    let after = app.controller.document().clone();
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &before);
    app.controller.redo().unwrap();
    assert_eq!(app.controller.document(), &after);
}

#[cfg(feature = "occt-brep")]
#[test]
fn legacy_v4_coarse_and_fine_spheres_resolve_the_true_front_surface() {
    use om9_core::geometry::{Curve, Geometry};
    use om9_scene::camera::Viewport;
    use sha2::{Digest, Sha256};
    let limits = Limits::default();
    let coarse = om9_kernel::brep::sphere_brep(3., 0.5, &limits).unwrap();
    let fine = om9_kernel::brep::sphere_brep(3., 0.001, &limits).unwrap();
    let bytes = include_bytes!("../../../implementation/evidence/M03/solid-ui-followup-2026-10-23/review-fixtures/Boolean2-D-after.om9");
    let start = u64::from_le_bytes(bytes[40..48].try_into().unwrap()) as usize;
    let len = u64::from_le_bytes(bytes[48..56].try_into().unwrap()) as usize;
    let mut metadata: serde_json::Value = serde_json::from_slice(&bytes[start..start + len]).unwrap();
    metadata["tolerance"] = serde_json::to_value(om9_core::history::Tolerance::om9_template()).unwrap();
    let entities = metadata["entities"].as_object_mut().unwrap();
    let mut front = entities.values().next().unwrap().clone();
    let mut back = front.clone();
    let front_id = uuid::Uuid::new_v4();
    let back_id = uuid::Uuid::new_v4();
    front["id"] = serde_json::json!(front_id);
    front["geometry"]["Brep"] = serde_json::to_value(coarse).unwrap();
    front["placement"][2][3] = serde_json::json!(0.01);
    back["id"] = serde_json::json!(back_id);
    back["geometry"]["Brep"] = serde_json::to_value(fine).unwrap();
    entities.clear();
    entities.insert(front_id.to_string(), front);
    entities.insert(back_id.to_string(), back);
    let metadata = serde_json::to_vec(&metadata).unwrap();
    let mut fixture = bytes[..start].to_vec();
    fixture[48..56].copy_from_slice(&(metadata.len() as u64).to_le_bytes());
    fixture[56..88].copy_from_slice(&Sha256::digest(&metadata));
    fixture.extend_from_slice(&metadata);
    let path = std::env::temp_dir().join(format!("om9-v4-coarse-snap-{}.om9", uuid::Uuid::new_v4()));
    std::fs::write(&path, fixture).unwrap();
    let context = egui::Context::default();
    let mut app = MatrixApp::from_file(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    for _ in 0..3 { frame(&mut app, &context, vec![]); }
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.submit("Line").unwrap();
    app.set_snap_override(Some(SnapKind::Surface));
    let rect = app.view_rects[&ViewId::LookingDown];
    let viewport = Viewport { origin: [rect.min.x as f64, rect.min.y as f64], size: [rect.width() as f64, rect.height() as f64] };
    let camera = &app.controller.workspace.views[&ViewId::LookingDown].camera;
    let screen = camera.project([1.278, 0.822, 0.01 + (9_f64 - 1.278_f64.powi(2) - 0.822_f64.powi(2)).sqrt()], viewport).unwrap().unwrap().screen;
    let point = Pos2::new(screen[0] as f32, screen[1] as f32);
    let candidates = om9_scene::osnap::candidates(app.controller.scene().unwrap(), camera, viewport,
        [point.x as f64, point.y as f64], Default::default(), None).unwrap();
    let display_first = candidates.iter().find(|c| c.kind == SnapKind::Surface).unwrap();
    assert_eq!(display_first.root.unwrap().to_string(), back_id.to_string(),
        "the coarse front display must be behind the fine back display for this fixture");
    frame(&mut app, &context, vec![Event::PointerMoved(point)]);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
    let snap = loop {
        if let Some(snap) = app.snap_candidate().filter(|s| s.kind == SnapKind::Surface) { break snap.clone(); }
        assert!(std::time::Instant::now() < deadline, "coarse/fine snap: {}", app.status_line());
        std::thread::sleep(std::time::Duration::from_millis(5));
        frame(&mut app, &context, vec![]);
    };
    assert_eq!(snap.root.unwrap().to_string(), front_id.to_string());
    let radius = (snap.world[0].powi(2) + snap.world[1].powi(2) + (snap.world[2] - 0.01).powi(2)).sqrt();
    assert!((radius - 3.).abs() < 1e-6, "front source radius: {radius}");
    click(&mut app, &context, point);
    app.controller.submit("20,20,20").unwrap();
    let actual = app.controller.document().entities.values().find_map(|e| match &e.geometry {
        Geometry::Curve(Curve::Line { start, .. }) => Some(*start), _ => None,
    }).unwrap();
    assert!((0..3).all(|i| (actual[i] - snap.world[i]).abs() < 1e-6));
}

#[cfg(feature = "occt-brep")]
#[test]
fn legacy_v4_boolean_brep_reopens_and_snaps_to_source_face() {
    use om9_core::geometry::{Curve, Geometry};
    use om9_scene::camera::Viewport;
    use sha2::{Digest, Sha256};
    let bytes = include_bytes!("../../../implementation/evidence/M03/solid-ui-followup-2026-10-23/review-fixtures/Boolean2-D-after.om9");
    assert_eq!(u32::from_le_bytes(bytes[12..16].try_into().unwrap()), 1);
    let start = u64::from_le_bytes(bytes[40..48].try_into().unwrap()) as usize;
    let len = u64::from_le_bytes(bytes[48..56].try_into().unwrap()) as usize;
    let mut metadata: serde_json::Value = serde_json::from_slice(&bytes[start..start + len]).unwrap();
    metadata["tolerance"] = serde_json::to_value(om9_core::history::Tolerance::om9_template()).unwrap();
    let metadata = serde_json::to_vec(&metadata).unwrap();
    let mut fixture = bytes[..start].to_vec();
    fixture[48..56].copy_from_slice(&(metadata.len() as u64).to_le_bytes());
    fixture[56..88].copy_from_slice(&Sha256::digest(&metadata));
    fixture.extend_from_slice(&metadata);
    let path = std::env::temp_dir().join(format!("om9-v4-snap-{}.om9", uuid::Uuid::new_v4()));
    std::fs::write(&path, fixture).unwrap();
    let context = egui::Context::default();
    let mut app = MatrixApp::from_file(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    let id = app.controller.document().entities.values().find(|e| matches!(e.geometry, Geometry::Brep(_))).unwrap().id;
    for _ in 0..3 { frame(&mut app, &context, vec![]); }
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.submit("Line").unwrap();
    app.set_snap_override(Some(SnapKind::Surface));
    let rect = app.view_rects[&ViewId::LookingDown];
    let viewport = Viewport { origin: [rect.min.x as f64, rect.min.y as f64], size: [rect.width() as f64, rect.height() as f64] };
    let projected = app.controller.workspace.views[&ViewId::LookingDown].camera.project([2.5, 5., 10.], viewport).unwrap().unwrap().screen;
    let point = Pos2::new(projected[0] as f32, projected[1] as f32);
    frame(&mut app, &context, vec![Event::PointerMoved(point)]);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
    let snap = loop {
        if let Some(snap) = app.snap_candidate().filter(|snap| snap.kind == SnapKind::Surface) { break snap.clone(); }
        assert!(std::time::Instant::now() < deadline, "legacy BRep snap: {}", app.status_line());
        std::thread::sleep(std::time::Duration::from_millis(5));
        frame(&mut app, &context, vec![]);
    };
    assert_eq!(snap.root, Some(id));
    assert!((snap.world[2] - 10.).abs() < 1e-7, "source face: {:?}", snap.world);
    let before = app.controller.document().clone();
    click(&mut app, &context, point);
    app.controller.submit("20,20,20").unwrap();
    let start = app.controller.document().entities.values().find_map(|e| match &e.geometry {
        Geometry::Curve(Curve::Line { start, .. }) => Some(*start), _ => None,
    }).expect("created line");
    assert!(start.iter().zip(snap.world).all(|(a, b)| (a - b).abs() < 1e-7));
    let after = app.controller.document().clone();
    app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
    app.controller.redo().unwrap(); assert_eq!(app.controller.document(), &after);
}

#[cfg(feature = "occt-brep")]
#[test]
fn legacy_v4_drilled_brep_snap_respects_reopened_hole() {
    use om9_core::geometry::{Curve, Geometry};
    use om9_kernel::{creation::{self, Primitive, Profile}, editing::{self, BooleanOp, BooleanOutcome}};
    use om9_scene::camera::{Camera, Viewport};
    use sha2::{Digest, Sha256};
    let limits = Limits::default();
    let profile = Profile::new(vec![[-5., -5., 0.], [5., -5., 0.], [5., 5., 0.], [-5., 5., 0.]]).unwrap();
    let block = creation::extrude(&profile, [0., 0., 10.], true, 0.2, &limits).unwrap();
    let cutter = creation::primitive(Primitive::Cylinder { radius_mm: 2., height_mm: 10. }, 0.2, &limits).unwrap();
    let BooleanOutcome::Body(body) = editing::boolean(&block, &cutter, BooleanOp::Difference, 0.2, &limits).unwrap() else { panic!("drilled block") };
    let bytes = include_bytes!("../../../implementation/evidence/M03/solid-ui-followup-2026-10-23/review-fixtures/Boolean2-D-after.om9");
    assert_eq!(u32::from_le_bytes(bytes[12..16].try_into().unwrap()), 1);
    let start = u64::from_le_bytes(bytes[40..48].try_into().unwrap()) as usize;
    let len = u64::from_le_bytes(bytes[48..56].try_into().unwrap()) as usize;
    let mut metadata: serde_json::Value = serde_json::from_slice(&bytes[start..start + len]).unwrap();
    metadata["tolerance"] = serde_json::to_value(om9_core::history::Tolerance::om9_template()).unwrap();
    let entity = metadata["entities"].as_object_mut().unwrap().values_mut().next().unwrap();
    entity["geometry"]["Brep"] = serde_json::to_value(body).unwrap();
    let metadata = serde_json::to_vec(&metadata).unwrap();
    let mut fixture = bytes[..start].to_vec();
    fixture[48..56].copy_from_slice(&(metadata.len() as u64).to_le_bytes());
    fixture[56..88].copy_from_slice(&Sha256::digest(&metadata));
    fixture.extend_from_slice(&metadata);
    let path = std::env::temp_dir().join(format!("om9-v4-drilled-snap-{}.om9", uuid::Uuid::new_v4()));
    std::fs::write(&path, fixture).unwrap();
    let context = egui::Context::default();
    let mut app = MatrixApp::from_file(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    let id = app.controller.document().entities.values().find(|e| matches!(e.geometry, Geometry::Brep(_))).unwrap().id;
    for _ in 0..3 { frame(&mut app, &context, vec![]); }
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.workspace.views.get_mut(&ViewId::LookingDown).unwrap().camera =
        Camera::orthographic([0., 0., 30.], [0.; 3], [0., 1., 0.], 14.).unwrap();
    frame(&mut app, &context, vec![]);
    app.controller.submit("Line").unwrap();
    app.set_snap_override(Some(SnapKind::Surface));
    let rect = app.view_rects[&ViewId::LookingDown];
    let viewport = Viewport { origin: [rect.min.x as f64, rect.min.y as f64], size: [rect.width() as f64, rect.height() as f64] };
    let project = |app: &MatrixApp, world| {
        let p = app.controller.workspace.views[&ViewId::LookingDown].camera.project(world, viewport).unwrap().unwrap().screen;
        Pos2::new(p[0] as f32, p[1] as f32)
    };
    let hole = project(&app, [0., 0., 10.]);
    let hole_hits = om9_scene::osnap::candidates(app.controller.scene().unwrap(),
        &app.controller.workspace.views[&ViewId::LookingDown].camera, viewport,
        [hole.x as f64, hole.y as f64], Default::default(), None).unwrap();
    assert!(!hole_hits.iter().any(|candidate| candidate.kind == SnapKind::Surface && candidate.root == Some(id)),
        "the through-hole must not produce a display-face witness");
    let before = app.controller.document().clone();
    click(&mut app, &context, hole);
    assert!(app.snap_candidate().is_none(), "through-hole must not snap to the top face");
    assert_eq!(app.controller.document(), &before);
    let point = project(&app, [3., 1., 10.]);
    frame(&mut app, &context, vec![Event::PointerMoved(point)]);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
    let snap = loop {
        if let Some(snap) = app.snap_candidate().filter(|snap| snap.kind == SnapKind::Surface) { break snap.clone(); }
        assert!(std::time::Instant::now() < deadline, "drilled BRep snap: {}", app.status_line());
        std::thread::sleep(std::time::Duration::from_millis(5));
        frame(&mut app, &context, vec![]);
    };
    assert_eq!(snap.root, Some(id));
    assert!((snap.world[2] - 10.).abs() < 1e-7 && snap.world[0].hypot(snap.world[1]) > 2.);
    click(&mut app, &context, point);
    app.controller.submit("20,20,20").unwrap();
    let start = app.controller.document().entities.values().find_map(|e| match &e.geometry {
        Geometry::Curve(Curve::Line { start, .. }) => Some(*start), _ => None,
    }).expect("created line");
    assert!(start.iter().zip(snap.world).all(|(a, b)| (a - b).abs() < 1e-7));
    let after = app.controller.document().clone();
    app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
    app.controller.redo().unwrap(); assert_eq!(app.controller.document(), &after);
}

#[cfg(all(windows, feature = "occt-brep"))]
#[test]
#[ignore = "requires test-fixtures and the packaged canonical BRep worker fingerprint"]
fn reopened_boolean_brep_surface_override_commits_source_face() {
    use om9_app::file_lifecycle::{FileKind, FileOutcome};
    use om9_core::geometry::{Curve, Geometry};
    use om9_scene::camera::Viewport;
    use std::io::Read;

    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::with_template_tolerance("reopened trimmed BRep snap")).unwrap();
    app.controller.solid_sphere_points([0.; 3], 5.).unwrap();
    let left = *app.controller.document().entities.keys().next().unwrap();
    app.controller.solid_sphere_points([5., 0., 0.], 5.).unwrap();
    let right = *app.controller.document().entities.keys().find(|id| **id != left).unwrap();
    app.controller.select_objects(&[left]).unwrap();
    app.controller.submit("BooleanDifference").unwrap();
    app.controller.submit("DeleteInput=Yes").unwrap();
    app.controller.submit("").unwrap();
    app.controller.select_objects(&[right]).unwrap();
    app.controller.submit("").unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(45);
    loop {
        if let Some(result) = app.controller.poll_boolean_union_worker() { result.unwrap(); break; }
        assert!(std::time::Instant::now() < deadline, "Boolean worker timeout");
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    assert_eq!(app.controller.document().entities.len(), 1);
    let scene = app.controller.scene().unwrap();
    assert!(scene.brep_occurrence_has_interior_wires(&scene.objects[0]).unwrap());
    let id = scene.objects[0].root;
    let path = std::env::temp_dir().join(format!("om9-reopened-boolean-snap-{}.om9", uuid::Uuid::new_v4()));
    assert_eq!(app.controller.begin_file_operation(FileKind::SaveAs).unwrap(), FileOutcome::Pending);
    let token = app.controller.file_operation().unwrap().token();
    assert_eq!(app.controller.accept_file_path(token, &path).unwrap(), FileOutcome::Pending);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    loop {
        if let Some(result) = app.controller.poll_file_save_worker() {
            assert_eq!(result.unwrap(), FileOutcome::Saved);
            break;
        }
        assert!(std::time::Instant::now() < deadline, "BRep Save As worker timeout");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    let mut header = [0; 12];
    std::fs::File::open(&path).unwrap().read_exact(&mut header).unwrap();
    assert_eq!(u32::from_le_bytes(header[8..12].try_into().unwrap()), 6);
    let reopened = MatrixApp::from_file(&path);
    std::fs::remove_file(path).unwrap();
    let mut app = reopened.unwrap();
    assert!(app.controller.file_diagnostics().is_empty());
    assert!(matches!(app.controller.document().entities[&id].geometry, Geometry::Brep(_)));
    for _ in 0..3 { frame(&mut app, &context, vec![]); }
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.submit("Line").unwrap();
    app.set_snap_override(Some(SnapKind::Surface));
    let rect = app.view_rects[&ViewId::LookingDown];
    let viewport = Viewport { origin: [rect.min.x as f64, rect.min.y as f64], size: [rect.width() as f64, rect.height() as f64] };
    let expected = [1., -1., 23_f64.sqrt()];
    let projected = app.controller.workspace.views[&ViewId::LookingDown].camera.project(expected, viewport).unwrap().unwrap().screen;
    let point = Pos2::new(projected[0] as f32, projected[1] as f32);
    frame(&mut app, &context, vec![Event::PointerMoved(point)]);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
    let snap = loop {
        if let Some(snap) = app.snap_candidate().filter(|snap| snap.kind == SnapKind::Surface) { break snap.clone(); }
        assert!(std::time::Instant::now() < deadline, "reopened BRep snap: {}", app.status_line());
        std::thread::sleep(std::time::Duration::from_millis(5));
        frame(&mut app, &context, vec![]);
    };
    assert_eq!(snap.root, Some(id));
    let tolerance = app.controller.document().effective_tolerance().unwrap().absolute_mm;
    assert!(snap.world.iter().zip(expected).all(|(a, b)| (a - b).abs() <= tolerance),
        "expected {expected:?}, got {:?}", snap.world);
    let radius = snap.world.iter().map(|v| v * v).sum::<f64>().sqrt();
    assert!((radius - 5.).abs() < 1e-7, "source radius {radius}");
    let before = app.controller.document().clone();
    let undo_before = app.controller.coordinator().undo_len();
    click(&mut app, &context, point);
    app.controller.submit("10,10,10").unwrap();
    let start = app.controller.document().entities.values().find_map(|e| match &e.geometry {
        Geometry::Curve(Curve::Line { start, .. }) => Some(*start), _ => None,
    }).expect("created line");
    assert!(start.iter().zip(snap.world).all(|(a, b)| (a - b).abs() < 1e-7));
    assert_eq!(app.controller.coordinator().undo_len(), undo_before + 1);
    let after = app.controller.document().clone();
    app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
    app.controller.redo().unwrap(); assert_eq!(app.controller.document(), &after);
}

#[cfg(feature = "occt-brep")]
#[test]
fn surface_worker_rejects_stale_cursor_camera_document_and_cancel() {
    use om9_core::budget::memory::MemoryClass;
    use om9_scene::camera::{Camera, Viewport};
    let mut app = MatrixApp::new(Document::with_template_tolerance("surface worker freshness")).unwrap();
    app.controller.solid_sphere_points([0.; 3], 5.).unwrap();
    let id = *app.controller.document().entities.keys().next().unwrap();
    let controller = &mut app.controller;
    let camera = Camera::orthographic([0., 0., 20.], [0.; 3], [0., 1., 0.], 12.).unwrap();
    let viewport = Viewport { origin: [0., 0.], size: [600., 600.] };
    let cursor = camera.project([2., 1., 20_f64.sqrt()], viewport).unwrap().unwrap().screen;
    let candidates: Vec<_> = om9_scene::osnap::candidates(controller.scene().unwrap(), &camera, viewport, cursor, Default::default(), None).unwrap()
        .into_iter().filter(|c| c.kind == SnapKind::Surface).collect();
    assert!(!candidates.is_empty());
    let start = |controller: &Controller| {
        let guard = controller.coordinator().reserve_external_memory(MemoryClass::Kernel, 1024 * 1024, None).unwrap();
        om9_app::session::SurfaceSnapJob::start(controller, candidates.clone(), camera.clone(), viewport, cursor, guard).unwrap()
    };
    let receipt = |job: &mut om9_app::session::SurfaceSnapJob| {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        loop {
            if let Some(result) = job.try_result() { break result.unwrap(); }
            assert!(std::time::Instant::now() < deadline, "surface worker timed out");
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
    };
    let mut job = start(controller);
    let hit = receipt(&mut job).accept(controller, &camera, viewport, cursor).unwrap().unwrap();
    assert!((hit.world[2] - 20_f64.sqrt()).abs() < 1e-6);
    drop(job);
    let mut job = start(controller);
    assert!(receipt(&mut job).accept(controller, &camera, viewport, [cursor[0] + 1., cursor[1]]).is_err());
    drop(job);
    let changed_camera = Camera::orthographic([1., 0., 20.], [0.; 3], [0., 1., 0.], 12.).unwrap();
    let mut job = start(controller);
    assert!(receipt(&mut job).accept(controller, &changed_camera, viewport, cursor).is_err());
    drop(job);
    let mut job = start(controller);
    let other = Controller::new(Document::with_template_tolerance("another document"), Limits::default()).unwrap();
    assert!(receipt(&mut job).accept(&other, &camera, viewport, cursor).is_err());
    drop(job);
    let mut job = start(controller);
    let finished = receipt(&mut job);
    job.cancel();
    assert!(matches!(finished.accept(controller, &camera, viewport, cursor), Err(om9_core::Error::Canceled)));
    drop(job);
    let mut job = start(controller);
    let stale_selection = receipt(&mut job);
    controller.select_objects(&[id]).unwrap();
    assert!(stale_selection.accept(controller, &camera, viewport, cursor).is_err());
    drop(job);
    let mut job = start(controller);
    let stale_deleted_source = receipt(&mut job);
    assert_eq!(controller.delete_objects(&[id]).unwrap(), 1);
    assert!(stale_deleted_source.resolve(controller, &camera, viewport, cursor).is_err());
    controller.undo().unwrap();
    assert!(stale_deleted_source.accept(controller, &camera, viewport, cursor).is_err(),
        "undo must not revive a receipt captured before source deletion");
    drop(job);
    controller.select_objects(&[id]).unwrap();
    let candidates = om9_scene::osnap::candidates(controller.scene().unwrap(), &camera, viewport, cursor, Default::default(), None).unwrap()
        .into_iter().filter(|c| c.kind == SnapKind::Surface).collect();
    let guard = controller.coordinator().reserve_external_memory(MemoryClass::Kernel, 1024 * 1024, None).unwrap();
    let mut job = om9_app::session::SurfaceSnapJob::start(controller, candidates, camera.clone(), viewport, cursor, guard).unwrap();
    let stale_moved_source = receipt(&mut job);
    controller.submit("Move").unwrap();
    controller.accept_world_point([0.; 3]).unwrap();
    controller.accept_world_point([1., 0., 0.]).unwrap();
    assert!(stale_moved_source.accept(controller, &camera, viewport, cursor).is_err());
    drop(job);
    let moved_cursor = camera.project([3., 1., 20_f64.sqrt()], viewport).unwrap().unwrap().screen;
    let moved_candidates = om9_scene::osnap::candidates(controller.scene().unwrap(), &camera, viewport, moved_cursor, Default::default(), None).unwrap()
        .into_iter().filter(|c| c.kind == SnapKind::Surface).collect();
    let guard = controller.coordinator().reserve_external_memory(MemoryClass::Kernel, 1024 * 1024, None).unwrap();
    let mut job = om9_app::session::SurfaceSnapJob::start(controller, moved_candidates, camera.clone(), viewport, moved_cursor, guard).unwrap();
    let moved_hit = receipt(&mut job).accept(controller, &camera, viewport, moved_cursor).unwrap().unwrap();
    assert!((moved_hit.world[0] - 3.).abs() < 1e-6 && (moved_hit.world[2] - 20_f64.sqrt()).abs() < 1e-6);
    drop(job);
    assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel), 0);
}

#[cfg(feature = "occt-brep")]
#[test]
fn sphere_pole_end_snap_uses_topology_and_commits_the_preview() {
    use om9_core::geometry::{Curve, Geometry};
    use om9_scene::{camera::Viewport, workspace::DisplayMode};
    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::with_template_tolerance("sphere pole snap")).unwrap();
    app.controller.solid_sphere_points([0.; 3], 5.).unwrap();
    let source_id = app.controller.document().entities.values()
        .find(|entity| matches!(entity.geometry, Geometry::Brep(_))).unwrap().id;
    let before = app.controller.document().clone();
    let undo_before = app.controller.coordinator().undo_len();
    app.controller.workspace.views.get_mut(&ViewId::LookingDown).unwrap().mode = DisplayMode::Shaded;
    frame(&mut app, &context, vec![]);
    frame(&mut app, &context, vec![]);
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.submit("Line").unwrap();
    app.set_snap_override(Some(SnapKind::End));
    let rect = app.view_rects[&ViewId::LookingDown];
    let viewport = Viewport { origin: [rect.min.x as f64, rect.min.y as f64],
        size: [rect.width() as f64, rect.height() as f64] };
    let projected = app.controller.workspace.views[&ViewId::LookingDown].camera
        .project([0., 0., 5.], viewport).unwrap().unwrap().screen;
    let cursor = Pos2::new(projected[0] as f32, projected[1] as f32);
    frame(&mut app, &context, vec![Event::PointerMoved(cursor)]);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    let preview = loop {
        if let Some(candidate) = app.snap_candidate().filter(|candidate| candidate.kind == SnapKind::End) {
            break candidate.clone();
        }
        assert!(std::time::Instant::now() < deadline, "sphere End: {}", app.status_line());
        std::thread::sleep(std::time::Duration::from_millis(5));
        frame(&mut app, &context, vec![]);
    };
    assert_eq!(preview.root, Some(source_id));
    assert!((preview.world[0].abs() + preview.world[1].abs() + (preview.world[2] - 5.).abs()) < 1e-6,
        "front pole: {:?}", preview.world);
    click(&mut app, &context, cursor);
    app.controller.submit("10,10,0").unwrap();
    let start = app.controller.document().entities.values().find_map(|entity| match &entity.geometry {
        Geometry::Curve(Curve::Line { start, .. }) => Some(*start), _ => None,
    }).expect("outer line");
    assert!((0..3).all(|i| (start[i] - preview.world[i]).abs() < 1e-6));
    assert_eq!(app.controller.coordinator().undo_len(), undo_before + 1);
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &before);
}

#[cfg(feature = "occt-brep")]
#[test]
fn cylinder_edge_center_snap_uses_source_and_commits_the_preview() {
    use om9_core::geometry::{Curve, Geometry};
    use om9_scene::{camera::Viewport, workspace::DisplayMode};
    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::with_template_tolerance("cylinder center snap")).unwrap();
    app.controller.submit("Cylinder 2 10").unwrap();
    let source_id = app.controller.document().entities.values()
        .find(|entity| matches!(entity.geometry, Geometry::Brep(_))).unwrap().id;
    let before = app.controller.document().clone();
    let undo_before = app.controller.coordinator().undo_len();
    app.controller.workspace.views.get_mut(&ViewId::LookingDown).unwrap().mode = DisplayMode::Shaded;
    frame(&mut app, &context, vec![]);
    frame(&mut app, &context, vec![]);
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.submit("Line").unwrap();
    app.set_snap_override(Some(SnapKind::Center));
    let rect = app.view_rects[&ViewId::LookingDown];
    let viewport = Viewport { origin: [rect.min.x as f64, rect.min.y as f64],
        size: [rect.width() as f64, rect.height() as f64] };
    let projected = app.controller.workspace.views[&ViewId::LookingDown].camera
        .project([0., 0., 10.], viewport).unwrap().unwrap().screen;
    let cursor = Pos2::new(projected[0] as f32, projected[1] as f32);
    frame(&mut app, &context, vec![Event::PointerMoved(cursor)]);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    let preview = loop {
        if let Some(candidate) = app.snap_candidate().filter(|candidate| candidate.kind == SnapKind::Center) {
            break candidate.clone();
        }
        assert!(std::time::Instant::now() < deadline, "cylinder Center: {}", app.status_line());
        std::thread::sleep(std::time::Duration::from_millis(5));
        frame(&mut app, &context, vec![]);
    };
    assert_eq!(preview.root, Some(source_id));
    assert!((preview.world[0].abs() + preview.world[1].abs() + (preview.world[2] - 10.).abs()) < 1e-6,
        "front edge center: {:?}", preview.world);
    click(&mut app, &context, cursor);
    app.controller.submit("10,10,0").unwrap();
    let start = app.controller.document().entities.values().find_map(|entity| match &entity.geometry {
        Geometry::Curve(Curve::Line { start, .. }) => Some(*start), _ => None,
    }).expect("outer line");
    assert!((0..3).all(|i| (start[i] - preview.world[i]).abs() < 1e-6));
    assert_eq!(app.controller.coordinator().undo_len(), undo_before + 1);
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &before);
}

#[cfg(feature = "occt-brep")]
#[test]
fn intersection_keeps_isolated_hit_among_overlaps_and_refuses_projected_crossing() {
    use om9_core::{geometry::{Curve, Geometry}, model::Entity};
    use om9_scene::camera::Viewport;

    let mut document = Document::with_template_tolerance("intersection witness");
    let mut insert_line = |start, end| {
        let entity = Entity::new("reference", Geometry::Curve(Curve::Line {
            start, end, domain: [0., 1.],
        }), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        id
    };
    let first = insert_line([-9., 0., 0.], [7., 0., 0.]);
    let overlap = insert_line([-8., 0., 0.], [8., 0., 0.]);
    let crossing = insert_line([3., -4., 0.], [3., 4., 0.]);
    insert_line([9., 7., 0.], [15., 7., 0.]);
    insert_line([12., 4., 2.], [12., 10., 2.]);

    let context = egui::Context::default();
    let mut app = MatrixApp::new(document).unwrap();
    app.controller.workspace.active = ViewId::LookingDown;
    for _ in 0..3 {
        frame(&mut app, &context, vec![]);
    }
    set_apparent_intersections(&mut app, &context, false);
    let before = app.controller.document().clone();
    let rect = app.view_rects[&ViewId::LookingDown];
    let viewport = Viewport {
        origin: [rect.min.x as f64, rect.min.y as f64],
        size: [rect.width() as f64, rect.height() as f64],
    };
    let camera = app.controller.workspace.views[&ViewId::LookingDown].camera.clone();
    let at = |point| {
        let [x, y] = camera.project(point, viewport).unwrap().unwrap().screen;
        Pos2::new(x as f32, y as f32)
    };
    app.controller.submit("Line").unwrap();
    app.set_snap_override(Some(SnapKind::Intersection));

    // The two sources project to a common pixel, but they are 2 mm apart in Z.
    let projected = at([12., 7., 0.]);
    assert_eq!(projected, at([12., 7., 2.]));
    click(&mut app, &context, projected);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while app.status_line().contains("Waiting for edge snaps") {
        assert!(std::time::Instant::now() < deadline, "{}", app.status_line());
        std::thread::sleep(std::time::Duration::from_millis(5));
        frame(&mut app, &context, vec![]);
    }
    assert!(app.controller.prompt().starts_with("Start of line"));
    assert_eq!(app.snap_override(), Some(SnapKind::Intersection));

    frame(&mut app, &context, vec![Event::PointerMoved(at([-5., 0., 0.]))]);
    while !app.status_line().contains("overlapping line segments") {
        assert!(app.snap_candidate().is_none());
        assert!(std::time::Instant::now() < deadline, "{}", app.status_line());
        std::thread::sleep(std::time::Duration::from_millis(5));
        frame(&mut app, &context, vec![]);
    }
    assert!(app.snap_candidate().is_none());

    let cursor = at([3., 0., 0.]);
    frame(&mut app, &context, vec![Event::PointerMoved(cursor)]);
    let hit = loop {
        if let Some(hit) = app.snap_candidate() { break hit.clone(); }
        assert!(std::time::Instant::now() < deadline, "{}", app.status_line());
        std::thread::sleep(std::time::Duration::from_millis(5));
        frame(&mut app, &context, vec![]);
    };
    assert_eq!(hit.kind, SnapKind::Intersection);
    assert_eq!(hit.world, [3., 0., 0.]);
    let roots = [hit.source.unwrap().root, hit.second_source.unwrap().root];
    assert!(roots.contains(&crossing) && (roots.contains(&first) || roots.contains(&overlap)));
    click(&mut app, &context, cursor);
    assert_eq!(app.snap_override(), None);
    app.controller.submit("15,4,0").unwrap();
    assert_eq!(app.controller.coordinator().undo_len(), 1);
    assert!(app.controller.document().entities.values().any(|entity| {
        !before.entities.contains_key(&entity.id) && matches!(&entity.geometry,
            Geometry::Curve(Curve::Line { start, .. }) if *start == [3., 0., 0.])
    }));
}

#[test]
fn apparent_intersection_keeps_the_selected_perspective_source_before_project() {
    use om9_core::{geometry::{Curve, Geometry, NurbsCurve}, model::Entity};
    use om9_scene::{camera::{Camera, CPlane, Viewport}, workspace::DisplayMode};
    use std::f64::consts::PI;

    let lines = [
        Curve::Line { start: [5., 0., 0.], end: [13., 0., 4.], domain: [0., 1.] },
        Curve::Line { start: [6., -4., 8.], end: [6., 4., 8.], domain: [0., 1.] },
    ];
    let curves = [
        Curve::Circle {
            center: [9., 4., 2.], x: [1., 0., 0.], y: [0., 1., 0.],
            radius: 4., domain: [0., 1.], angle_domain: Some([PI, 2. * PI]),
        },
        Curve::Nurbs(NurbsCurve {
            degree: 2, poles: vec![[5., -4., 7.], [8., 0., 8.], [5., 4., 9.]],
            weights: vec![1., 0.5, 1.], knots: vec![0., 0., 0., 1., 1., 1.],
            domain: [0., 1.], periodic: false, reversed: false, seam: None,
        }),
    ];
    let clipped_witness = [6., 0., 37. / 3.];
    let far_x = clipped_witness[0] * 18. / (20. - clipped_witness[2]);
    let partly_visible = [
        Curve::Line {
            start: [far_x - 3., 0., 2.],
            end: [far_x + 3., 0., 2.],
            domain: [0., 1.],
        },
        Curve::Nurbs(NurbsCurve {
            degree: 2,
            poles: vec![[5., -4., 7.], [8., 0., 8.], [5., 4., 22.]],
            weights: vec![1., 0.5, 1.],
            knots: vec![0., 0., 0., 1., 1., 1.],
            domain: [0., 1.],
            periodic: false,
            reversed: false,
            seam: None,
        }),
    ];
    let ordinary_witness = [6., 0., 8.];
    for (sources, aim, project, offset_x, selected, expected_source) in [
        (lines.clone(), ordinary_witness, false, 0., 1, ordinary_witness),
        (lines.clone(), ordinary_witness, true, 0., 1, ordinary_witness),
        (lines, ordinary_witness, true, 3., 0, [9., 0., 2.]),
        (curves.clone(), ordinary_witness, false, 0., 1, ordinary_witness),
        (curves, ordinary_witness, true, 3., 0, [9., 0., 2.]),
        (partly_visible.clone(), clipped_witness, false, 0., 1, clipped_witness),
        (partly_visible, clipped_witness, true, 0., 1, clipped_witness),
    ] {
        let straight = sources.iter().all(|curve| matches!(curve, Curve::Line { .. }));
        if !straight && !cfg!(feature = "occt-brep") { continue; }
        let mut expected_start = expected_source;
        if project { expected_start[2] = 0.; }
        let mut document = Document::with_template_tolerance("apparent intersection");
        let point_tolerance = if straight { 1e-9 } else { 1e-6 };
        let roots = sources.map(|curve| {
            let entity = Entity::new("reference", Geometry::Curve(curve), None);
            let id = entity.id;
            document.entities.insert(id, entity);
            id
        });
        let context = egui::Context::default();
        let mut app = MatrixApp::new(document).unwrap();
        app.controller.workspace.active = ViewId::Perspective;
        for _ in 0..3 { frame(&mut app, &context, vec![]); }
        let rect = app.view_rects[&ViewId::Perspective];
        let viewport = Viewport {
            origin: [rect.min.x as f64, rect.min.y as f64],
            size: [rect.width() as f64, rect.height() as f64],
        };
        // Put the crossing on a pixel exactly representable by egui's f32 pointer.
        let focal_px = 100. * (20. - aim[2]) / aim[0];
        let fov = 2. * (viewport.size[1] / (2. * focal_px)).atan();
        let mut camera = Camera::perspective([0., 0., 20.], [0.; 3], [0., 1., 0.], fov).unwrap();
        camera.near_mm = 2.;
        let [x, y] = camera.project(aim, viewport).unwrap().unwrap().screen;
        let cursor = Pos2::new(x as f32 + offset_x, y as f32);
        let view = app.controller.workspace.views.get_mut(&ViewId::Perspective).unwrap();
        view.camera = camera;
        view.cplane = CPlane::world_xy();
        view.mode = DisplayMode::Wireframe;

        app.focus_command_editor();
        frame(&mut app, &context, vec![]);
        frame(&mut app, &context, vec![Event::Text("ApparentIntersections On".into())]);
        frame(&mut app, &context, vec![Event::Key {
            key: Key::Enter, physical_key: None, pressed: true, repeat: false, modifiers: Modifiers::NONE,
        }]);
        assert!(app.status_line().contains("ApparentIntersections on"), "{}", app.status_line());
        app.controller.set_project(project);
        app.controller.submit("Line").unwrap();
        app.set_snap_override(Some(SnapKind::Intersection));

        frame(&mut app, &context, vec![Event::PointerMoved(cursor)]);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let hit = loop {
            if let Some(hit) = app.snap_candidate() { break hit.clone(); }
            assert!(std::time::Instant::now() < deadline, "{}", app.status_line());
            std::thread::sleep(std::time::Duration::from_millis(5));
            frame(&mut app, &context, vec![]);
        };
        assert!(hit.apparent, "{hit:?}");
        assert_eq!(hit.source.as_ref().unwrap().root, roots[selected]);
        assert_eq!(hit.second_source.as_ref().unwrap().root, roots[1 - selected]);
        assert!(hit.world.into_iter().zip(expected_source)
            .all(|(actual, expected)| (actual - expected).abs() < point_tolerance), "{hit:?}");
        assert_eq!(hit.tooltip(), "Apparent intersection");
        assert_eq!(hit.marker_label(), "Apparent Int");
        click(&mut app, &context, cursor);
        assert_eq!(app.snap_override(), None);
        app.controller.submit("0,5,0").unwrap();
        let start = app.controller.document().entities.values().find_map(|entity| {
            match &entity.geometry {
                Geometry::Curve(Curve::Line { start, end, .. }) if *end == [0., 5., 0.] => Some(*start),
                _ => None,
            }
        }).expect("the accepted line");
        assert!(start.into_iter().zip(expected_start)
            .all(|(actual, expected)| (actual - expected).abs() < point_tolerance), "{start:?}");
        assert_eq!(app.controller.coordinator().undo_len(), 1);
    }
}

fn check_unordered_line_intersections(mut app: MatrixApp) {
    use om9_core::geometry::{Curve, Geometry};
    use om9_scene::{camera::{Camera, Viewport}, osnap::{self, SnapSettings}};
    use std::{thread, time::{Duration, Instant}};

    let before = app.controller.document().clone();
    let undo_before = app.controller.coordinator().undo_len();
    assert_eq!(before.entities.len(), 3);
    assert!(before.entities.values().all(|entity| entity.creation_order.is_none()));
    assert!(before.entities.values().all(|entity| matches!(entity.geometry, Geometry::Curve(Curve::Line { .. }))));

    let context = egui::Context::default();
    let view_id = ViewId::LookingDown;
    app.controller.workspace.active = view_id;
    for _ in 0..3 { frame(&mut app, &context, vec![]); }
    let rect = app.view_rects[&view_id];
    let viewport = Viewport {
        origin: [f64::from(rect.min.x), f64::from(rect.min.y)],
        size: [f64::from(rect.width()), f64::from(rect.height())],
    };
    let camera = Camera::orthographic([0., 0., 50.], [0.; 3], [0., 1., 0.], viewport.size[1] / 10.).unwrap();
    app.controller.workspace.views.get_mut(&view_id).unwrap().camera = camera.clone();
    let center = camera.project([0., 0., 2.], viewport).unwrap().unwrap().screen;
    assert_eq!(camera.project([0., 0., 8.], viewport).unwrap().unwrap().screen, center);
    assert_eq!(center.map(|coordinate| f64::from(coordinate as f32)), center);

    app.focus_command_editor();
    frame(&mut app, &context, vec![]);
    frame(&mut app, &context, vec![Event::Text("ApparentIntersections On".into())]);
    frame(&mut app, &context, vec![Event::Key {
        key: Key::Enter, physical_key: None, pressed: true, repeat: false, modifiers: Modifiers::NONE,
    }]);
    assert!(app.status_line().contains("ApparentIntersections on"), "{}", app.status_line());

    for (offset, z, apparent, project) in [
        ([0., 0.], 2., false, false),
        ([0., 4.], 8., true, false),
        ([4., 0.], 2., false, false),
        ([0., 0.], 2., false, true),
    ] {
        let screen = [center[0] + offset[0], center[1] + offset[1]];
        let cursor = Pos2::new(screen[0] as f32, screen[1] as f32);
        let expected_world = [0., 0., z];
        let expected_start = [0., 0., if project { 0. } else { z }];
        let settings = SnapSettings {
            one_shot: Some(SnapKind::Intersection), apparent_intersections: true, ..Default::default()
        };
        let hits = osnap::candidates(app.controller.scene().unwrap(), &camera, viewport, screen, settings, None).unwrap();
        let [direct] = hits.as_slice() else { panic!("expected one Int at {offset:?}: {hits:?}") };
        assert_eq!(direct.world, expected_world);
        assert_eq!(direct.apparent, apparent);

        app.controller.set_project(project);
        app.controller.submit("Line").unwrap();
        app.set_snap_override(Some(SnapKind::Intersection));
        frame(&mut app, &context, vec![Event::PointerMoved(cursor)]);
        let deadline = Instant::now() + Duration::from_secs(10);
        let hit = loop {
            if let Some(hit) = app.snap_candidate() { break hit.clone(); }
            assert!(Instant::now() < deadline, "{offset:?}: {}", app.status_line());
            thread::sleep(Duration::from_millis(5));
            frame(&mut app, &context, vec![]);
        };
        assert_eq!(hit.kind, SnapKind::Intersection);
        assert_eq!(hit.world, expected_world, "{offset:?}");
        assert_eq!(hit.apparent, apparent, "{offset:?}");
        let source = &before.entities[&hit.source.as_ref().unwrap().root];
        assert!(matches!(source.geometry, Geometry::Curve(Curve::Line { start, .. }) if start[2] == z));
        assert!(hit.second_source.is_some());

        click(&mut app, &context, cursor);
        assert_eq!(app.snap_override(), None);
        app.controller.submit("5,6,0").unwrap();
        let committed = app.controller.document().entities.values().find(|entity| !before.entities.contains_key(&entity.id)).unwrap();
        assert!(matches!(committed.geometry, Geometry::Curve(Curve::Line { start, end, .. })
            if start == expected_start && end == [5., 6., 0.]));
        assert_eq!(app.controller.coordinator().undo_len(), undo_before + 1);
        app.controller.undo().unwrap();
        assert_eq!(app.controller.document(), &before);
    }
}

#[test]
fn unordered_line_intersections_use_source_distance_then_true_intersection() {
    use om9_core::{geometry::{Curve, Geometry}, identity::ObjectId, model::Entity};

    let lines = [
        ([-10., 0., 2.], [10., 0., 2.]),
        ([0., -10., 8.], [0., 10., 8.]),
        ([-8., -8., 2.], [8., 8., 2.]),
    ];
    for order in [[0, 1, 2], [0, 2, 1], [1, 0, 2], [1, 2, 0], [2, 0, 1], [2, 1, 0]] {
        let mut document = Document::with_template_tolerance("unordered source lines");
        for (index, line) in order.into_iter().enumerate() {
            let (start, end) = lines[line];
            let mut entity = Entity::new("source line", Geometry::Curve(Curve::Line { start, end, domain: [0., 1.] }), None);
            entity.id = ObjectId(uuid::Uuid::from_u128(index as u128 + 1));
            document.entities.insert(entity.id, entity);
        }
        check_unordered_line_intersections(MatrixApp::new(document).unwrap());
    }
}

#[test]
#[ignore = "needs the two local Rhino 5 fixtures and a configured native exchange worker"]
fn int_after_repeated_import_of_both_rhino5_archives() {
    let fixtures = std::path::PathBuf::from(std::env::var_os("OM9_SNAPS_PAIR_ROOT").expect("Rhino 5 fixture directory"));
    for file in ["coincident-true-and-apparent-rhino5.3dm", "coincident-vertical-first-rhino5.3dm"] {
        let path = fixtures.join(file);
        for _ in 0..5 {
            let mut app = MatrixApp::new(Document::with_template_tolerance("imported source lines")).unwrap();
            app.controller.import_3dm(&path).unwrap();
            check_unordered_line_intersections(app);
        }
        println!("{file}: five fresh Imports verified");
    }
}

#[test]
#[cfg(feature = "occt-brep")]
fn polycurve_int_parameter_resolves_on_the_reversed_source() {
    use om9_core::{
        curve_ops::point_at,
        geometry::{Curve, Geometry},
        model::Entity,
    };
    use om9_scene::{
        camera::{Camera, Viewport},
        workspace::DisplayMode,
    };

    let line = |start, end| Curve::Line {
        start,
        end,
        domain: [0., 1.],
    };
    for (height, apparent, project, curved, expected_fraction) in [
        (2., false, false, true, 0.625),
        (8., true, false, false, 0.125),
        (8., true, false, true, 0.125),
        (8., true, true, true, 0.125),
    ] {
        let last_start = [if curved { 0. } else { 4. }, -4., height];
        let last = if curved {
            Curve::Circle {
                center: [0., 0., height],
                x: [1., 0., 0.],
                y: [0., 1., 0.],
                radius: 4.,
                domain: [0., 1.],
                angle_domain: Some([-std::f64::consts::FRAC_PI_2, std::f64::consts::FRAC_PI_2]),
            }
        } else {
            line(last_start, [4., 4., height])
        };
        let curve = Curve::Polycurve {
            segments: vec![
                line([-8., 0., 2.], [8., 0., 2.]),
                Curve::Polycurve {
                    segments: vec![line([8., 0., 2.], last_start), last],
                    reversed: false,
                },
            ],
            reversed: true,
        };
        let mut document = Document::with_template_tolerance("reversed Polycurve Int");
        let entity = Entity::new("joined reference", Geometry::Curve(curve.clone()), None);
        let source_id = entity.id;
        document.entities.insert(source_id, entity);
        let context = egui::Context::default();
        let mut app = MatrixApp::new(document).unwrap();
        app.controller.workspace.active = ViewId::LookingDown;
        for _ in 0..3 {
            frame(&mut app, &context, vec![]);
        }
        let rect = app.view_rects[&ViewId::LookingDown];
        let viewport = Viewport {
            origin: [rect.min.x as f64, rect.min.y as f64],
            size: [rect.width() as f64, rect.height() as f64],
        };
        // Keep the projection on a pixel representable by egui's f32 pointer
        // so source distance ties and depth chooses the visible branch.
        let camera = Camera::orthographic(
            [0., 0., 30.],
            [0.; 3],
            [0., 1., 0.],
            viewport.size[1] / 5.,
        )
        .unwrap();
        let expected_world = [4., 0., height];
        let [x, y] = camera
            .project(expected_world, viewport)
            .unwrap()
            .unwrap()
            .screen;
        let cursor = Pos2::new(x as f32, y as f32);
        let view = app
            .controller
            .workspace
            .views
            .get_mut(&ViewId::LookingDown)
            .unwrap();
        view.camera = camera;
        view.mode = DisplayMode::Wireframe;
        if apparent {
            app.focus_command_editor();
            frame(&mut app, &context, vec![]);
            frame(
                &mut app,
                &context,
                vec![Event::Text("ApparentIntersections On".into())],
            );
            frame(
                &mut app,
                &context,
                vec![Event::Key {
                    key: Key::Enter,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: Modifiers::NONE,
                }],
            );
            assert!(
                app.status_line().contains("ApparentIntersections on"),
                "{}",
                app.status_line()
            );
        }
        app.controller.set_project(project);
        app.controller.submit("Line").unwrap();
        app.set_snap_override(Some(SnapKind::Intersection));
        frame(&mut app, &context, vec![Event::PointerMoved(cursor)]);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let hit = loop {
            if let Some(hit) = app.snap_candidate() {
                break hit.clone();
            }
            assert!(
                std::time::Instant::now() < deadline,
                "{}",
                app.status_line()
            );
            std::thread::sleep(std::time::Duration::from_millis(5));
            frame(&mut app, &context, vec![]);
        };
        let matches_point = |actual: [f64; 3], expected| {
            actual
                .into_iter()
                .zip(expected)
                .all(|(a, b): (f64, f64)| (a - b).abs() < 1e-6)
        };
        assert_eq!(hit.apparent, apparent, "{hit:?}");
        assert_eq!(hit.source.as_ref().unwrap().root, source_id);
        assert_eq!(hit.second_source.as_ref().unwrap().root, source_id);
        assert!(matches_point(hit.world, expected_world), "{hit:?}");
        let fraction = hit.parameter.unwrap();
        assert!(
            (fraction - expected_fraction).abs() < 1e-6,
            "{fraction} != {expected_fraction}"
        );
        assert!(matches_point(
            point_at(&curve, fraction, &Limits::default()).unwrap(),
            hit.world
        ));
        click(&mut app, &context, cursor);
        let end = [0., 5., 0.];
        app.controller.submit("0,5,0").unwrap();
        let mut expected_start = expected_world;
        if project {
            expected_start[2] = 0.;
        }
        assert!(app.controller.document().entities.values().any(|entity| {
            matches!(&entity.geometry, Geometry::Curve(Curve::Line { start, end: actual_end, .. })
                if *actual_end == end && matches_point(*start, expected_start))
        }));
        assert_eq!(app.controller.coordinator().undo_len(), 1);
    }
}

#[test]
fn competing_intersections_follow_line_creation_order_and_keep_an_exact_tie() {
    use om9_core::{budget::Limits, geometry::{Curve, Geometry}, transaction::Cancellation};
    use om9_scene::camera::Viewport;

    for (order, from_view, vertical_x, expected_z) in [
        ([0, 1, 2], true, 0., 8.),
        ([1, 0, 2], true, 0., 2.),
        ([0, 1, 2], false, 0., 2.),
        ([1, 0, 2], false, 0., 2.),
        ([0, 1, 2], true, 0.25, 2.),
    ] {
        let context = egui::Context::default();
        let doc = Document::with_template_tolerance("competing intersections");
        let mut app = MatrixApp::new(doc).unwrap();
        app.controller.workspace.active = ViewId::LookingDown;
        let lines = [
            [[-8., 0., 2.], [8., 0., 2.]],
            [[vertical_x, -8., 8.], [vertical_x, 8., 8.]],
            [[-8., -8., 2.], [8., 8., 2.]],
        ];
        let mut creation_order = Vec::new();
        for index in order {
            app.controller.submit("Line").unwrap();
            for [x, y, z] in lines[index] {
                app.controller.submit(&format!("{x},{y},{z}")).unwrap();
            }
            let [start, end] = lines[index];
            let entity = app.controller.document().entities.values().find(|entity| {
                matches!(&entity.geometry, Geometry::Curve(Curve::Line { start: a, end: b, .. })
                    if *a == start && *b == end)
            }).expect("authored line");
            creation_order.push((entity.id, entity.creation_order.expect("document creation order")));
        }
        assert!(creation_order.windows(2).all(|pair| pair[0].1 < pair[1].1));
        app.controller.undo().unwrap();
        app.controller.redo().unwrap();
        let doc = app.controller.document();
        assert!(creation_order.iter().all(|(id, order)| doc.entities[id].creation_order == Some(*order)));
        if from_view && vertical_x == 0. {
            let path = std::env::temp_dir().join(format!("om9-int-order-{}.om9", uuid::Uuid::new_v4()));
            om9_archive::Archive::save(&path, doc, &Limits::default(), &Cancellation::default()).unwrap();
            let reopened = MatrixApp::from_file(&path);
            std::fs::remove_file(path).unwrap();
            app = reopened.unwrap();
            let doc = app.controller.document();
            assert!(creation_order.iter().all(|(id, order)| doc.entities[id].creation_order == Some(*order)));
        }
        app.controller.workspace.active = ViewId::LookingDown;
        for _ in 0..3 { frame(&mut app, &context, vec![]); }
        set_apparent_intersections(&mut app, &context, from_view);
        app.controller.submit("Line").unwrap();
        app.set_snap_override(Some(SnapKind::Intersection));
        let rect = app.view_rects[&ViewId::LookingDown];
        let viewport = Viewport {
            origin: [rect.min.x as f64, rect.min.y as f64],
            size: [rect.width() as f64, rect.height() as f64],
        };
        let camera = &app.controller.workspace.views[&ViewId::LookingDown].camera;
        let [x, y] = camera.project([0., 0., 2.], viewport).unwrap().unwrap().screen;
        let cursor = Pos2::new(x as f32, y as f32);
        frame(&mut app, &context, vec![Event::PointerMoved(cursor)]);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let hit = loop {
            if let Some(hit) = app.snap_candidate() { break hit.clone(); }
            assert!(std::time::Instant::now() < deadline, "{}", app.status_line());
            std::thread::sleep(std::time::Duration::from_millis(5));
            frame(&mut app, &context, vec![]);
        };
        let at_expected_start = |point: [f64; 3]| point.into_iter().zip([0., 0., expected_z])
            .all(|(actual, expected)| (actual - expected).abs() < 1e-12);
        assert!(at_expected_start(hit.world), "order {order:?}, apparent={from_view}, hit={hit:?}");
        assert_eq!(hit.apparent, expected_z == 8.);
        click(&mut app, &context, cursor);
        assert_eq!(app.snap_override(), None);
        app.controller.submit("5,6,0").unwrap();
        assert!(app.controller.document().entities.values().any(|entity| {
            matches!(&entity.geometry, Geometry::Curve(Curve::Line { start, end, .. })
                if at_expected_start(*start) && *end == [5., 6., 0.])
        }));
    }
}

#[test]
fn intersection_keeps_isolated_hit_when_a_line_and_nurbs_may_overlap() {
    use om9_core::{geometry::{Curve, Geometry, NurbsCurve}, model::Entity};
    use om9_scene::camera::Viewport;

    let mut document = Document::with_template_tolerance("line and NURBS overlap");
    let line = |start, end| Curve::Line { start, end, domain: [0., 1.] };
    let overlap_line = line([-8., 0., 2.], [8., 0., 2.]);
    let overlap_nurbs = Curve::Nurbs(NurbsCurve {
        degree: 3,
        poles: vec![[-8., 0., 2.], [-4., 0., 2.], [4., 0., 2.], [8., 0., 2.]],
        weights: vec![1.; 4],
        knots: vec![0., 0., 0., 0., 1., 1., 1., 1.],
        domain: [0., 1.],
        periodic: false,
        reversed: false,
        seam: None,
    });
    let mut insert = |curve| {
        let entity = Entity::new("reference", Geometry::Curve(curve), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        id
    };
    insert(overlap_line);
    insert(overlap_nurbs);
    let horizontal = insert(line([1., 0., 0.], [5., 0., 0.]));
    let vertical = insert(line([3., -2., 0.], [3., 2., 0.]));

    let mut app = MatrixApp::new(document).unwrap();
    let context = egui::Context::default();
    app.controller.workspace.active = ViewId::LookingDown;
    for _ in 0..3 { frame(&mut app, &context, vec![]); }
    set_apparent_intersections(&mut app, &context, false);
    let rect = app.view_rects[&ViewId::LookingDown];
    let viewport = Viewport {
        origin: [rect.min.x as f64, rect.min.y as f64],
        size: [rect.width() as f64, rect.height() as f64],
    };
    let camera = app.controller.workspace.views[&ViewId::LookingDown].camera.clone();
    let at = |point| {
        let [x, y] = camera.project(point, viewport).unwrap().unwrap().screen;
        Pos2::new(x as f32, y as f32)
    };
    app.controller.submit("Line").unwrap();
    app.set_snap_override(Some(SnapKind::Intersection));

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    frame(&mut app, &context, vec![Event::PointerMoved(at([-5., 0., 2.]))]);
    while !app.status_line().contains("line and NURBS may overlap") {
        assert!(app.snap_candidate().is_none(), "overlap is not an isolated hit");
        assert!(std::time::Instant::now() < deadline, "{}", app.status_line());
        std::thread::sleep(std::time::Duration::from_millis(5));
        frame(&mut app, &context, vec![]);
    }
    assert!(app.snap_candidate().is_none());
    assert_eq!(app.snap_override(), Some(SnapKind::Intersection));

    let crossing = [3., 0., 0.];
    let cursor = at(crossing);
    assert_eq!(cursor, at([3., 0., 2.]), "overlap must also be under the cursor");
    frame(&mut app, &context, vec![Event::PointerMoved(cursor)]);
    let hit = loop {
        if let Some(hit) = app.snap_candidate() { break hit.clone(); }
        assert!(std::time::Instant::now() < deadline, "{}", app.status_line());
        std::thread::sleep(std::time::Duration::from_millis(5));
        frame(&mut app, &context, vec![]);
    };
    assert_eq!(hit.kind, SnapKind::Intersection);
    assert_eq!(hit.world, crossing);
    let roots = [hit.source.unwrap().root, hit.second_source.unwrap().root];
    assert!(roots.contains(&horizontal) && roots.contains(&vertical));
    click(&mut app, &context, cursor);
    assert_eq!(app.snap_override(), None);
    app.controller.submit("6,4,0").unwrap();
    assert_eq!(app.controller.coordinator().undo_len(), 1);
    assert!(app.controller.document().entities.values().any(|entity| {
        matches!(&entity.geometry,
            Geometry::Curve(Curve::Line { start, end, .. }) if *start == crossing && *end == [6., 4., 0.])
    }));
}

#[cfg(feature = "occt-brep")]
#[test]
fn line_intersection_snap_uses_a_single_cubic_nurbs_crossing() {
    use om9_core::{
        budget::Limits,
        curve_ops::point_at,
        geometry::{Curve, Geometry, NurbsCurve},
        model::Entity,
    };
    use om9_scene::camera::Viewport;

    let context = egui::Context::default();
    let mut document = Document::with_template_tolerance("cubic self-intersection");
    // This polynomial Bézier has a transverse crossing at t=1/4 and t=3/4.
    let curve = Curve::Nurbs(NurbsCurve {
        degree: 3,
        poles: vec![[12., -5., 5.], [-4., 17., 5.], [-4., -9., 5.], [12., 13., 5.]],
        weights: vec![1.; 4],
        knots: vec![0., 0., 0., 0., 1., 1., 1., 1.],
        domain: [0., 1.],
        periodic: false,
        reversed: false,
        seam: None,
    });
    let crossing = [3., 4., 5.];
    let limits = Limits::default();
    for parameter in [0.25, 0.75] {
        assert_eq!(point_at(&curve, parameter, &limits).unwrap(), crossing);
    }
    let reference = Entity::new("Cubic reference", Geometry::Curve(curve.clone()), None);
    let source = reference.id;
    document.entities.insert(source, reference);
    let mut app = MatrixApp::new(document).unwrap();
    let before = app.controller.document().clone();
    app.controller.workspace.active = ViewId::LookingDown;
    for _ in 0..3 {
        frame(&mut app, &context, vec![]);
    }
    let rect = app.view_rects[&ViewId::LookingDown];
    let viewport = Viewport {
        origin: [rect.min.x as f64, rect.min.y as f64],
        size: [rect.width() as f64, rect.height() as f64],
    };
    let camera = &app.controller.workspace.views[&ViewId::LookingDown].camera;
    let [x, y] = camera.project(crossing, viewport).unwrap().unwrap().screen;
    let cursor = Pos2::new(x as f32, y as f32);
    let tolerance = before.effective_tolerance().unwrap().absolute_mm;

    for project in [false, true] {
        app.controller.set_project(project);
        app.controller.submit("Line").unwrap();
        app.set_snap_override(Some(SnapKind::Intersection));
        frame(&mut app, &context, vec![Event::PointerMoved(cursor)]);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let hit = loop {
            if let Some(hit) = app.snap_candidate() { break hit.clone(); }
            assert!(std::time::Instant::now() < deadline, "{}", app.status_line());
            std::thread::sleep(std::time::Duration::from_millis(5));
            frame(&mut app, &context, vec![]);
        };
        assert_eq!(hit.kind, SnapKind::Intersection);
        assert!(!hit.apparent);
        assert_eq!(hit.source.as_ref().unwrap().root, source);
        assert_eq!(hit.source, hit.second_source);
        let parameter = hit.parameter.unwrap();
        assert!((parameter - 0.25).abs() < 1e-5 || (parameter - 0.75).abs() < 1e-5, "{hit:?}");
        for point in [hit.world, point_at(&curve, parameter, &limits).unwrap()] {
            assert!(point.into_iter().zip(crossing)
                .all(|(actual, expected)| (actual - expected).abs() < tolerance), "{point:?}");
        }

        click(&mut app, &context, cursor);
        assert_eq!(app.snap_override(), None);
        app.controller.submit("8,9,0").unwrap();
        let start = app.controller.document().entities.values().find_map(|entity| match &entity.geometry {
            Geometry::Curve(Curve::Line { start, .. }) => Some(*start),
            _ => None,
        }).expect("accepted Line");
        let expected = [3., 4., if project { 0. } else { 5. }];
        assert!(start.into_iter().zip(expected)
            .all(|(actual, expected)| (actual - expected).abs() < tolerance), "{start:?}");
        assert_eq!(app.controller.coordinator().undo_len(), 1);
        app.controller.undo().unwrap();
        assert_eq!(app.controller.document(), &before);
    }
}

#[cfg(feature = "occt-brep")]
#[test]
fn brep_edge_and_line_intersection_snap_uses_both_sources() {
    use om9_core::geometry::{Curve, Geometry};
    use om9_scene::camera::{CPlane, Viewport};
    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::with_template_tolerance("BRep edge intersection")).unwrap();
    app.controller.solid_box_points([0., 0., 0.], [10., 10., 0.], 5., "Diagonal", None, CPlane::world_xy()).unwrap();
    let box_id = app.controller.document().entities.values()
        .find(|entity| matches!(entity.geometry, Geometry::Brep(_))).unwrap().id;
    app.controller.submit("Line").unwrap();
    app.controller.submit("5,-5,5").unwrap();
    app.controller.submit("5,5,5").unwrap();
    let line_id = app.controller.document().entities.values()
        .find(|entity| matches!(entity.geometry, Geometry::Curve(Curve::Line { .. }))).unwrap().id;
    let before = app.controller.document().clone();
    let undo_before = app.controller.coordinator().undo_len();
    frame(&mut app, &context, vec![]);
    frame(&mut app, &context, vec![]);
    app.controller.workspace.active = ViewId::LookingDown;
    set_apparent_intersections(&mut app, &context, false);
    app.controller.submit("Line").unwrap();
    app.set_snap_override(Some(SnapKind::Intersection));
    let rect = app.view_rects[&ViewId::LookingDown];
    let viewport = Viewport { origin: [rect.min.x as f64, rect.min.y as f64],
        size: [rect.width() as f64, rect.height() as f64] };
    let projected = app.controller.workspace.views[&ViewId::LookingDown].camera
        .project([5., 0., 5.], viewport).unwrap().unwrap().screen;
    let cursor = Pos2::new(projected[0] as f32, projected[1] as f32);
    frame(&mut app, &context, vec![Event::PointerMoved(cursor)]);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    let preview = loop {
        if let Some(candidate) = app.snap_candidate().filter(|candidate| candidate.kind == SnapKind::Intersection) {
            break candidate.clone();
        }
        assert!(std::time::Instant::now() < deadline, "BRep intersection: {}", app.status_line());
        std::thread::sleep(std::time::Duration::from_millis(5));
        frame(&mut app, &context, vec![]);
    };
    let first = preview.source.as_ref().unwrap().root;
    let second = preview.second_source.as_ref().unwrap().root;
    assert!(!preview.apparent);
    assert!((first == box_id && second == line_id) || (first == line_id && second == box_id));
    assert!((0..3).all(|i| (preview.world[i] - [5., 0., 5.][i]).abs() <= 0.001));
    click(&mut app, &context, cursor);
    app.controller.submit("20,20,0").unwrap();
    let start = app.controller.document().entities.values().find_map(|entity| match &entity.geometry {
        Geometry::Curve(Curve::Line { start, .. }) if entity.id != line_id => Some(*start), _ => None,
    }).expect("outer line");
    assert!((0..3).all(|i| (start[i] - preview.world[i]).abs() <= 0.001));
    assert_eq!(app.controller.coordinator().undo_len(), undo_before + 1);
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &before);
}

#[cfg(feature = "occt-brep")]
#[test]
fn intersection_on_a_trim_hole_uses_the_placed_surface_boundary() {
    use om9_core::{
        geometry::{Curve, Geometry, Transform},
        model::Entity,
        rhino_semantics::{set_trimmed_surface, TrimmedSurface},
        surface_kernel,
        transaction::Cancellation,
    };
    use om9_scene::{camera::Viewport, workspace::DisplayMode};

    let limits = Limits::default();
    let surface = surface_kernel::plane([0., 0., 0.], [1., 0., 0.], [0., 1., 0.], 10., 10., &limits).unwrap();
    let trim = TrimmedSurface {
        underlying: surface.clone(),
        outer: vec![[0.1, 0.1], [0.9, 0.1], [0.9, 0.9], [0.1, 0.9]],
        inners: vec![vec![[0.3, 0.4], [0.3, 0.6], [0.7, 0.6], [0.7, 0.4]]],
        reversed: false,
    };
    let placement = Transform([[0., -1.5, 0., 11.], [2., 0., 0., -7.], [0., 0., 1., 5.], [0., 0., 0., 1.]]);
    let world = |u, v| placement.point(surface_kernel::evaluate(&surface, u, v, &limits).unwrap()).unwrap();
    let crossing = world(0.5, 0.4);
    let mut surface_entity = Entity::new("Trimmed surface", Geometry::Surface(surface.clone()), None);
    surface_entity.placement = placement;
    set_trimmed_surface(&mut surface_entity, &trim, &limits).unwrap();
    let surface_id = surface_entity.id;
    let line = Entity::new("Crossing line", Geometry::Curve(Curve::Line {
        start: world(0.5, 0.3), end: world(0.5, 0.5), domain: [0., 1.],
    }), None);
    let line_id = line.id;
    let mut document = Document::with_template_tolerance("Trim hole intersection");
    document.entities.insert(surface_id, surface_entity);
    document.entities.insert(line_id, line);
    let context = egui::Context::default();
    let path = std::env::temp_dir().join(format!("om9-trim-hole-int-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, &document, &limits, &Cancellation::default()).unwrap();
    let reopened = MatrixApp::from_file(&path);
    std::fs::remove_file(path).unwrap();
    let mut app = reopened.unwrap();
    let before = app.controller.document().clone();
    let tolerance = before.effective_tolerance().unwrap().absolute_mm;
    app.controller.workspace.active = ViewId::LookingDown;
    for _ in 0..3 { frame(&mut app, &context, vec![]); }
    let rect = app.view_rects[&ViewId::LookingDown];
    let viewport = Viewport { origin: [rect.min.x as f64, rect.min.y as f64],
        size: [rect.width() as f64, rect.height() as f64] };
    let [x, y] = app.controller.workspace.views[&ViewId::LookingDown].camera
        .project(crossing, viewport).unwrap().unwrap().screen;
    let cursor = Pos2::new(x as f32, y as f32);

    for (mode, project) in [
        (DisplayMode::Wireframe, false),
        (DisplayMode::Wireframe, true),
        (DisplayMode::Shaded, false),
        (DisplayMode::Shaded, true),
    ] {
        app.controller.workspace.views.get_mut(&ViewId::LookingDown).unwrap().mode = mode;
        app.controller.set_project(project);
        app.controller.submit("Line").unwrap();
        app.set_snap_override(Some(SnapKind::Intersection));
        frame(&mut app, &context, vec![Event::PointerMoved(cursor)]);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let hit = loop {
            if let Some(hit) = app.snap_candidate() { break hit.clone(); }
            assert!(std::time::Instant::now() < deadline, "{mode:?}: {}", app.status_line());
            std::thread::sleep(std::time::Duration::from_millis(5));
            frame(&mut app, &context, vec![]);
        };
        assert_eq!(hit.kind, SnapKind::Intersection);
        assert!(!hit.apparent);
        let first = hit.source.as_ref().unwrap().root;
        let second = hit.second_source.as_ref().unwrap().root;
        assert!((first == surface_id && second == line_id) || (first == line_id && second == surface_id), "{hit:?}");
        assert!(hit.world.into_iter().zip(crossing).all(|(a, b)| (a - b).abs() < tolerance), "{hit:?}");

        click(&mut app, &context, cursor);
        assert_eq!(app.snap_override(), None);
        app.controller.submit("20,20,0").unwrap();
        let start = app.controller.document().entities.values().find_map(|entity| match &entity.geometry {
            Geometry::Curve(Curve::Line { start, .. }) if entity.id != line_id => Some(*start),
            _ => None,
        }).expect("committed Line");
        let expected = [crossing[0], crossing[1], if project { 0. } else { crossing[2] }];
        assert!(start.into_iter().zip(expected).all(|(a, b)| (a - b).abs() < tolerance), "{start:?}");
        assert_eq!(app.controller.coordinator().undo_len(), 1);
        app.controller.undo().unwrap();
        assert_eq!(app.controller.document(), &before);
    }
}

#[cfg(feature = "occt-brep")]
#[test]
fn apparent_intersection_uses_cylinder_rim_before_project() {
    use om9_core::geometry::{Curve, Geometry};
    use om9_scene::{
        camera::{Camera, CPlane, Viewport},
        workspace::DisplayMode,
    };

    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::with_template_tolerance("apparent rim intersection")).unwrap();
    app.controller.submit("Cylinder 2 10").unwrap();
    let cylinder = app.controller.document().entities.values()
        .find(|entity| matches!(entity.geometry, Geometry::Brep(_))).unwrap().id;
    app.controller.submit("Line").unwrap();
    app.controller.submit("1.8,1.8,2").unwrap();
    app.controller.submit("2.3,2.3,2").unwrap();
    let line = app.controller.document().entities.values()
        .find(|entity| matches!(entity.geometry, Geometry::Curve(Curve::Line { .. }))).unwrap().id;
    let before = app.controller.document().clone();
    let undo_before = app.controller.coordinator().undo_len();
    app.controller.workspace.active = ViewId::Perspective;
    for _ in 0..3 {
        frame(&mut app, &context, vec![]);
    }
    let rect = app.view_rects[&ViewId::Perspective];
    let viewport = Viewport {
        origin: [rect.min.x as f64, rect.min.y as f64],
        size: [rect.width() as f64, rect.height() as f64],
    };
    let rim_xy = 2f64.sqrt();
    let rim_point = [rim_xy, rim_xy, 10.];
    let focal_px = 60. * (30. - rim_point[2]) / rim_xy;
    let fov = 2. * (viewport.size[1] / (2. * focal_px)).atan();
    let camera = Camera::perspective([0., 0., 30.], [0.; 3], [0., 1., 0.], fov).unwrap();
    let [x, y] = camera.project(rim_point, viewport).unwrap().unwrap().screen;
    let lower_line_xy = rim_xy * (30. - 2.) / (30. - 10.);
    let lower_line_screen = camera.project([lower_line_xy, lower_line_xy, 2.], viewport)
        .unwrap().unwrap().screen;
    assert!((lower_line_screen[0] - x).abs() < 1e-9 && (lower_line_screen[1] - y).abs() < 1e-9);
    let cursor = Pos2::new(x as f32, y as f32);
    let view = app.controller.workspace.views.get_mut(&ViewId::Perspective).unwrap();
    view.camera = camera;
    view.cplane = CPlane::world_xy();

    app.focus_command_editor();
    frame(&mut app, &context, vec![]);
    frame(&mut app, &context, vec![Event::Text("ApparentIntersections On".into())]);
    frame(&mut app, &context, vec![Event::Key {
        key: Key::Enter, physical_key: None, pressed: true, repeat: false, modifiers: Modifiers::NONE,
    }]);
    assert!(app.status_line().contains("ApparentIntersections on"), "{}", app.status_line());
    let tolerance = before.effective_tolerance().unwrap().absolute_mm;
    for mode in [DisplayMode::Wireframe, DisplayMode::Shaded] {
        app.controller.workspace.views.get_mut(&ViewId::Perspective).unwrap().mode = mode;
        for project in [false, true] {
            app.controller.set_project(project);
            app.controller.submit("Line").unwrap();
            app.set_snap_override(Some(SnapKind::Intersection));
            frame(&mut app, &context, vec![Event::PointerMoved(cursor)]);
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
            let hit = loop {
                if let Some(hit) = app.snap_candidate() { break hit.clone(); }
                assert!(std::time::Instant::now() < deadline, "{mode:?}: {}", app.status_line());
                std::thread::sleep(std::time::Duration::from_millis(5));
                frame(&mut app, &context, vec![]);
            };
            assert_eq!(hit.marker_label(), "Apparent Int");
            assert_eq!(hit.source.as_ref().unwrap().root, cylinder, "{hit:?}");
            assert_eq!(hit.second_source.as_ref().unwrap().root, line, "{hit:?}");
            assert!(hit.world.into_iter().zip(rim_point)
                .all(|(actual, expected)| (actual - expected).abs() < tolerance), "{hit:?}");

            click(&mut app, &context, cursor);
            assert_eq!(app.snap_override(), None);
            app.controller.submit("5,6,0").unwrap();
            let start = app.controller.document().entities.values().find_map(|entity| {
                match &entity.geometry {
                    Geometry::Curve(Curve::Line { start, .. }) if entity.id != line => Some(*start),
                    _ => None,
                }
            }).expect("accepted line");
            let expected = [hit.world[0], hit.world[1], if project { 0. } else { hit.world[2] }];
            assert!(start.into_iter().zip(expected)
                .all(|(actual, expected)| (actual - expected).abs() < tolerance), "{start:?}");
            assert_eq!(app.controller.coordinator().undo_len(), undo_before + 1);
            app.controller.undo().unwrap();
            assert_eq!(app.controller.document(), &before);
        }
    }
}

#[test]
fn idle_viewport_shortcut_precedes_printable_command_input() {
    use om9_core::{geometry::Geometry, model::Entity};

    let context = egui::Context::default();
    let mut document = Document::with_template_tolerance("idle snap keys");
    for label in ["front point", "back point"] {
        let entity = Entity::new(label, Geometry::Point([0.; 3]), None);
        document.entities.insert(entity.id, entity);
    }
    let mut app = MatrixApp::new(document).unwrap();
    frame(&mut app, &context, vec![]);
    frame(&mut app, &context, vec![]);
    assert!(context.memory(|memory| memory.focused().is_none()));
    let at = viewport_center(&app);
    click(&mut app, &context, at);
    assert_eq!(app.ambiguous_candidates().len(), 2);
    app.choose_ambiguous(0).unwrap();
    frame(&mut app, &context, vec![]);
    frame(&mut app, &context, vec![Event::Key { key: Key::E, physical_key: None,
        pressed: true, repeat: false, modifiers: Modifiers::NONE }, Event::Text("e".into())]);
    assert_eq!(app.snap_override(), Some(SnapKind::End));
    assert_eq!(app.transcript.draft(), "");
    assert!(context.memory(|memory| memory.focused().is_none()));

    frame(&mut app, &context, vec![Event::Text("x".into())]);
    assert_eq!(app.transcript.draft(), "x");
    assert!(context.memory(|memory| memory.has_focus(egui::Id::new("matrix-command-transcript"))));
    assert!(!app.controller.is_running());
}

#[test]
fn pipe_curve_pick_focuses_the_radius_prompt() {
    use om9_core::{geometry::{Curve, Geometry}, model::Entity};
    let mut document = Document::with_template_tolerance("Pipe input focus");
    let rail = Entity::new("rail", Geometry::Curve(Curve::Line {
        start: [-5., 0., 0.], end: [5., 0., 0.], domain: [0., 1.],
    }), None);
    document.entities.insert(rail.id, rail);
    let context = egui::Context::default();
    let mut app = MatrixApp::new(document).unwrap();
    frame(&mut app, &context, vec![]);
    frame(&mut app, &context, vec![]);
    app.controller.dispatch("OM9-SOLID-011", om9_app::session::InvocationOrigin::Menu).unwrap();
    assert!(app.controller.solid_pipe_needs_curve());
    let at = app.view_rects[&ViewId::LookingDown].center();
    click(&mut app, &context, at);
    assert!(app.controller.prompt().starts_with("Start radius"), "{}", app.controller.prompt());
    assert!(context.memory(|memory| memory.has_focus(egui::Id::new("matrix-command-transcript"))));
    frame(&mut app, &context, vec![Event::Text("1".into())]);
    frame(&mut app, &context, vec![Event::Key { key: Key::Enter, physical_key: None,
        pressed: true, repeat: false, modifiers: Modifiers::NONE }]);
    assert!(app.controller.prompt().starts_with("End radius"), "{}", app.controller.prompt());
}

#[test]
fn e_and_m_select_end_and_mid_while_a_command_is_running() {
    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::with_template_tolerance("snap shortcut")).unwrap();
    frame(&mut app, &context, vec![]);
    frame(&mut app, &context, vec![]);
    app.controller.submit("Line").unwrap();
    app.focus_command_editor();
    frame(&mut app, &context, vec![]);
    assert_eq!(context.memory(|memory| memory.focused()), Some(egui::Id::new("matrix-command-transcript")));
    let at = viewport_center(&app);
    click(&mut app, &context, at);
    assert!(app.controller.prompt().starts_with("End of line"));
    assert!(
        !app.command_focus_requested(),
        "viewport click releases the command editor"
    );
    assert_ne!(context.memory(|memory| memory.focused()), Some(egui::Id::new("matrix-command-transcript")),
        "viewport click must also release actual egui keyboard focus");
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
        }, Event::Text("e".into())],
    );
    assert_eq!(app.snap_override(), Some(SnapKind::End));
    assert!(!app.command_focus_requested(), "shortcut text must not activate the command editor");
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
    assert_eq!(app.snap_override(), Some(SnapKind::Mid), "a miss must not consume the one-shot");
    assert!(app.status_line().contains("No Mid snap point under the pointer"), "{}", app.status_line());
    assert!(app.controller.document().entities.is_empty());
    assert_eq!(app.controller.coordinator().undo_len(), 0);
    frame(&mut app, &context, vec![Event::Key {
        key: Key::Escape, physical_key: None, pressed: true, repeat: false, modifiers: Modifiers::NONE,
    }]);
    assert_eq!(app.snap_override(), None, "Escape cancels the one-shot");
    assert!(app.controller.prompt().starts_with("End of line"));
}

#[test]
fn escape_release_without_a_press_cancels_only_the_viewport_one_shot() {
    let key = |key, pressed| Event::Key {
        key, physical_key: Some(key), pressed, repeat: false, modifiers: Modifiers::NONE,
    };
    for running in [false, true] {
        let context = egui::Context::default();
        let mut app = MatrixApp::new(Document::new("Windows Escape release")).unwrap();
        frame(&mut app, &context, vec![]);
        if running {
            app.controller.submit("Line").unwrap();
        }
        let before = app.controller.document().clone();
        let prompt = app.controller.prompt();
        let arm_end = |app: &mut MatrixApp| {
            frame(app, &context, vec![key(Key::E, true), key(Key::E, false)]);
            assert_eq!(app.snap_override(), Some(SnapKind::End));
        };
        arm_end(&mut app);
        frame(&mut app, &context, vec![Event::WindowFocused(false), key(Key::Escape, false)]);
        assert_eq!(app.snap_override(), Some(SnapKind::End));
        frame(&mut app, &context, vec![Event::WindowFocused(true), key(Key::Escape, false)]);
        assert_eq!(app.snap_override(), Some(SnapKind::End));

        frame(&mut app, &context, vec![key(Key::Escape, true)]);
        assert_eq!(app.snap_override(), None);
        arm_end(&mut app);
        frame(&mut app, &context, vec![key(Key::Escape, false)]);
        assert_eq!(app.snap_override(), Some(SnapKind::End), "the previous press already cancelled");

        frame(&mut app, &context, vec![key(Key::Backspace, true), key(Key::Escape, false)]);
        assert_eq!(app.snap_override(), None, "recover the standalone Escape release");
        assert!(app.status_line().contains("One-shot snap cancelled"));
        assert_eq!(app.controller.prompt(), prompt);
        assert_eq!(app.controller.document(), &before);
        assert_eq!(app.controller.coordinator().undo_len(), 0);
        frame(&mut app, &context, vec![key(Key::E, true), key(Key::E, false), key(Key::Escape, false)]);
        assert_eq!(app.snap_override(), Some(SnapKind::End), "keep a shortcut armed in this frame");
        if running {
            app.focus_command_editor();
            frame(&mut app, &context, vec![]);
            frame(&mut app, &context, vec![key(Key::Escape, false)]);
            assert_eq!(app.snap_override(), Some(SnapKind::End), "CMD owns the keyboard");
        }
    }
}

#[test]
fn a_focused_command_editor_does_not_consume_the_snap_shortcut() {
    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::new("snap focus")).unwrap();
    frame(&mut app, &context, vec![]);
    app.controller.submit("Line").unwrap();
    app.focus_command_editor();
    assert!(app.command_focus_requested());
    frame(&mut app, &context, vec![]);
    frame(
        &mut app,
        &context,
        vec![Event::Key {
            key: Key::E,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }, Event::Text("e".into())],
    );
    assert_eq!(app.snap_override(), None);
    assert_eq!(app.transcript.draft(), "e", "focused editor must receive the typed e");
}

#[cfg(feature = "occt-brep")]
#[test]
fn deferred_viewport_point_releases_command_editor_focus() {
    use om9_scene::{camera::Viewport, workspace::DisplayMode};
    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::with_template_tolerance("deferred snap focus")).unwrap();
    app.controller.solid_sphere_points([0.; 3], 5.).unwrap();
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.workspace.views.get_mut(&ViewId::LookingDown).unwrap().mode = DisplayMode::Wireframe;
    frame(&mut app, &context, vec![]);
    frame(&mut app, &context, vec![]);
    app.controller.submit("Line").unwrap();
    app.set_snap_override(Some(SnapKind::End));
    app.focus_command_editor();
    frame(&mut app, &context, vec![]);
    assert_eq!(context.memory(|memory| memory.focused()), Some(egui::Id::new("matrix-command-transcript")));
    let rect = app.view_rects[&ViewId::LookingDown];
    let viewport = Viewport { origin: [rect.min.x as f64, rect.min.y as f64],
        size: [rect.width() as f64, rect.height() as f64] };
    let projected = app.controller.workspace.views[&ViewId::LookingDown].camera
        .project([0., 0., 5.], viewport).unwrap().unwrap().screen;
    let permit = app.controller.coordinator().external_memory_account().try_acquire_surface_snap().unwrap();
    click(&mut app, &context, Pos2::new(projected[0] as f32, projected[1] as f32));
    assert!(app.controller.prompt().starts_with("Start of line"), "fixture must observe deferred click");
    assert!(app.status_line().contains("Waiting for edge snaps"), "{}", app.status_line());
    drop(permit);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
    while !app.controller.prompt().starts_with("End of line") {
        assert!(std::time::Instant::now() < deadline, "deferred point: {}", app.status_line());
        std::thread::sleep(std::time::Duration::from_millis(5));
        frame(&mut app, &context, vec![]);
    }
    assert_eq!(app.snap_override(), None, "the first override was consumed by the click");
    frame(&mut app, &context, vec![]);
    assert_ne!(context.memory(|memory| memory.focused()), Some(egui::Id::new("matrix-command-transcript")),
        "deferred point must release actual editor focus");
    frame(&mut app, &context, vec![Event::Key { key: Key::E, physical_key: None,
        pressed: true, repeat: false, modifiers: Modifiers::NONE }, Event::Text("e".into())]);
    assert_eq!(app.snap_override(), Some(SnapKind::End));
    assert_eq!(app.transcript.draft(), "");
}

#[cfg(feature = "occt-brep")]
#[test]
fn deferred_end_ctrl_click_keeps_the_elevator_base_after_ctrl_release() {
    use om9_core::geometry::{Curve, Geometry};
    use om9_scene::{camera::{Camera, Viewport}, workspace::DisplayMode};
    use std::time::{Duration, Instant};

    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::with_template_tolerance("deferred Elevator base")).unwrap();
    app.controller.solid_sphere_points([0.; 3], 5.).unwrap();
    let id = ViewId::LookingDown;
    app.controller.workspace.active = id;
    let view = app.controller.workspace.views.get_mut(&id).unwrap();
    // Separate the two sphere poles on screen while keeping the Top input plane.
    view.camera = Camera::orthographic([0., -40., 40.], [0.; 3], [0., 0., 1.], 40.).unwrap();
    view.mode = DisplayMode::Wireframe;
    app.set_object_snaps_enabled(false);
    for _ in 0..3 {
        frame(&mut app, &context, vec![]);
    }
    let before = app.controller.document().clone();
    let original_undo = app.controller.coordinator().undo_len();
    app.controller.submit("Line").unwrap();
    app.set_snap_override(Some(SnapKind::End));
    let rect = app.view_rects[&id];
    let viewport = Viewport {
        origin: [rect.min.x as f64, rect.min.y as f64],
        size: [rect.width() as f64, rect.height() as f64],
    };
    let [x, y] = app.controller.workspace.views[&id].camera
        .project([0., 0., 5.], viewport).unwrap().unwrap().screen;
    let at = Pos2::new(x as f32, y as f32);
    let permit = app.controller.coordinator().external_memory_account().try_acquire_surface_snap().unwrap();
    let ctrl = Modifiers { ctrl: true, ..Modifiers::NONE };
    for pressed in [true, false] {
        frame(&mut app, &context, vec![
            Event::ModifiersChanged(ctrl),
            Event::PointerMoved(at),
            Event::PointerButton { pos: at, button: PointerButton::Primary, pressed, modifiers: ctrl },
        ]);
    }
    assert!(app.status_line().contains("Waiting for edge snaps"), "{}", app.status_line());
    frame(&mut app, &context, vec![Event::ModifiersChanged(Modifiers::NONE)]);
    assert!(app.controller.point_constraint_base().is_none());
    assert_eq!(app.controller.document(), &before);
    assert_eq!(app.snap_override(), Some(SnapKind::End));

    drop(permit);
    let deadline = Instant::now() + Duration::from_secs(15);
    while !app.transcript.prompt().starts_with("Elevator:") {
        assert!(Instant::now() < deadline, "{}", app.status_line());
        std::thread::sleep(Duration::from_millis(5));
        frame(&mut app, &context, vec![]);
    }
    assert!(context.memory(|memory| memory.has_focus(egui::Id::new("matrix-command-transcript"))));
    assert_eq!(app.snap_override(), None);
    assert!(app.controller.point_constraint_base().is_none());
    assert_eq!(app.controller.coordinator().undo_len(), original_undo);

    let submit = |app: &mut MatrixApp, value: &str| {
        frame(app, &context, vec![Event::Text(value.into())]);
        frame(app, &context, vec![Event::Key { key: Key::Enter, physical_key: None,
            pressed: true, repeat: false, modifiers: Modifiers::NONE }]);
        frame(app, &context, vec![Event::Key { key: Key::Enter, physical_key: None,
            pressed: false, repeat: false, modifiers: Modifiers::NONE }]);
    };
    submit(&mut app, "2");
    let start = app.controller.point_constraint_base().unwrap().xyz();
    assert!(start.iter().zip([0., 0., 7.]).all(|(actual, expected)| (actual - expected).abs() < 1e-8), "{start:?}");
    assert_eq!(app.controller.document(), &before);
    submit(&mut app, "9,10,8");
    let after = app.controller.document().clone();
    assert!(after.entities.values().any(|entity| {
        matches!(&entity.geometry, Geometry::Curve(Curve::Line { start: a, end, .. }) if *a == start && *end == [9., 10., 8.])
    }));
    assert_eq!(app.controller.coordinator().undo_len(), original_undo + 1);
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &before);
    app.controller.redo().unwrap();
    assert_eq!(app.controller.document(), &after);
}

#[cfg(feature = "occt-brep")]
#[test]
fn deferred_end_miss_keeps_viewport_focus_for_retry() {
    use om9_core::geometry::{Curve, Geometry};
    use om9_scene::{camera::Viewport, workspace::DisplayMode};

    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::with_template_tolerance("deferred End retry")).unwrap();
    app.controller.solid_sphere_points([0.; 3], 5.).unwrap();
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.workspace.views.get_mut(&ViewId::LookingDown).unwrap().mode = DisplayMode::Wireframe;
    app.set_object_snaps_enabled(false);
    for _ in 0..3 {
        frame(&mut app, &context, vec![]);
    }
    let rect = app.view_rects[&ViewId::LookingDown];
    let viewport = Viewport {
        origin: [rect.min.x as f64, rect.min.y as f64],
        size: [rect.width() as f64, rect.height() as f64],
    };
    let camera = app.controller.workspace.views[&ViewId::LookingDown].camera.clone();
    let project = |world| {
        let [x, y] = camera.project(world, viewport).unwrap().unwrap().screen;
        Pos2::new(x as f32, y as f32)
    };
    let key = |key| Event::Key {
        key, physical_key: None, pressed: true, repeat: false, modifiers: Modifiers::NONE,
    };
    let before = app.controller.document().clone();
    let undo = app.controller.coordinator().undo_len();
    app.controller.submit("Line").unwrap();
    click(&mut app, &context, project([-10., -6., 0.]));
    assert!(app.controller.prompt().starts_with("End of line"));
    let permit = app.controller.coordinator().external_memory_account().try_acquire_surface_snap().unwrap();
    frame(&mut app, &context, vec![key(Key::E), Event::Text("e".into())]);
    assert_eq!(app.snap_override(), Some(SnapKind::End));

    // Inside the sphere's bounds but outside the End aperture of either pole.
    click(&mut app, &context, project([3., 2., 0.]));
    assert!(app.status_line().contains("Waiting for edge snaps"), "{}", app.status_line());
    drop(permit);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
    while !app.status_line().contains("snap point under the pointer") {
        assert!(std::time::Instant::now() < deadline, "{}", app.status_line());
        std::thread::sleep(std::time::Duration::from_millis(5));
        frame(&mut app, &context, vec![]);
    }
    assert!(app.status_line().contains("No End snap point under the pointer"), "{}", app.status_line());
    assert_eq!(app.snap_override(), Some(SnapKind::End));
    assert_eq!(app.controller.document(), &before);
    assert_eq!(app.controller.coordinator().undo_len(), undo);
    assert!(!context.memory(|memory| memory.has_focus(egui::Id::new("matrix-command-transcript"))));

    frame(&mut app, &context, vec![key(Key::Escape)]);
    assert_eq!(app.snap_override(), None);
    assert!(app.controller.prompt().starts_with("End of line"));
    frame(&mut app, &context, vec![key(Key::E), Event::Text("e".into())]);
    assert_eq!(app.snap_override(), Some(SnapKind::End));
    let pole = project([0., 0., 5.]);
    frame(&mut app, &context, vec![Event::PointerMoved(pole)]);
    let snapped_pole = loop {
        if let Some(snap) = app.snap_candidate().filter(|snap| snap.kind == SnapKind::End) {
            let [x, y, z] = snap.world;
            assert!(x.abs() < 1e-8 && y.abs() < 1e-8 && (z.abs() - 5.).abs() < 1e-8);
            break snap.world;
        }
        assert!(std::time::Instant::now() < deadline, "{}", app.status_line());
        std::thread::sleep(std::time::Duration::from_millis(5));
        frame(&mut app, &context, vec![]);
    };
    click(&mut app, &context, pole);
    assert_eq!(app.snap_override(), None);
    assert_eq!(app.controller.coordinator().undo_len(), undo + 1);
    assert!(app.controller.document().entities.values().any(|entity| {
        matches!(&entity.geometry, Geometry::Curve(Curve::Line { end, .. }) if *end == snapped_pole)
    }));
}
