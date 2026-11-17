// SPDX-License-Identifier: LGPL-2.1-or-later
//! `OM9-L3-02-03-007`: the viewport marker/point path consumes the osnap candidate.
use eframe::egui::{self, Event, Modifiers, Pos2, Rect, Vec2};
use om9_app::ui::MatrixApp;
use om9_core::{
    geometry::{Curve, Geometry},
    model::{Document, Entity},
};
use om9_scene::camera::Viewport;
use om9_scene::osnap::SnapKind;
use om9_scene::workspace::ViewId;

fn screen_rect() -> Rect {
    Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 900.))
}

fn frame(app: &mut MatrixApp, context: &egui::Context, events: Vec<Event>) {
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(screen_rect()),
            events,
            ..Default::default()
        },
        |ui| app.show(ui),
    );
    output.textures_delta.clear();
}

fn viewport(app: &MatrixApp) -> Viewport {
    let rect = app.view_rects[&ViewId::LookingDown];
    Viewport {
        origin: [rect.min.x as f64, rect.min.y as f64],
        size: [rect.width() as f64, rect.height() as f64],
    }
}

fn screen_of(app: &MatrixApp, point: [f64; 3]) -> Pos2 {
    let projected = app.controller.workspace.views[&ViewId::LookingDown]
        .camera
        .project(point, viewport(app))
        .unwrap()
        .unwrap();
    Pos2::new(projected.screen[0] as f32, projected.screen[1] as f32)
}

fn key(key: egui::Key) -> Event {
    Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    }
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
                button: egui::PointerButton::Primary,
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
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        }],
    );
}

#[cfg(feature = "occt-brep")]
#[test]
fn surface_snap_ui_commits_an_exact_sphere_point_instead_of_the_display_triangle() {
    let context = egui::Context::default();
    let limits = om9_core::budget::Limits::default();
    let sphere = om9_kernel::brep::sphere_brep(5., 0.5, &limits).unwrap();
    let mut document = Document::with_template_tolerance("surface snap precision");
    let entity = Entity::new("sphere", Geometry::Brep(sphere), None);
    document.entities.insert(entity.id, entity);
    let mut app = MatrixApp::new(document).unwrap();
    frame(&mut app, &context, vec![]);
    frame(&mut app, &context, vec![]);
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.submit("Line").unwrap();
    app.set_snap_override(Some(SnapKind::Surface));
    let point = screen_of(&app, [2.13, 1.37, (25.0_f64 - 2.13_f64.powi(2) - 1.37_f64.powi(2)).sqrt()]);
    frame(&mut app, &context, vec![Event::PointerMoved(point)]);
    let snap = app.snap_candidate().expect("refined surface snap").clone();
    assert_eq!(snap.kind, SnapKind::Surface);
    let radius = snap.world.iter().map(|value| value * value).sum::<f64>().sqrt();
    assert!((radius - 5.).abs() < 1e-7, "snap must be on the canonical sphere, got {radius}");
    click(&mut app, &context, point);
    app.controller.submit("20,20,20").unwrap();
    let line = app.controller.document().entities.values().find_map(|entity| match &entity.geometry {
        Geometry::Curve(Curve::Line { start, .. }) => Some(*start), _ => None,
    }).expect("created line");
    assert!(line.iter().zip(snap.world).all(|(a,b)| (a-b).abs() < 1e-7));
}

#[cfg(feature = "occt-brep")]
#[test]
fn grouped_box_surface_snap_commits_front_face_for_line_and_gumball() {
    for gumball in [false, true] {
        let context = egui::Context::default();
        let limits = om9_core::budget::Limits::default();
        let mut box_entity = Entity::new("box", Geometry::Brep(om9_kernel::brep::box_brep([10., 10., 10.], 0.1, &limits).unwrap()), None);
        box_entity.placement = om9_core::geometry::Transform::translation([10., 10., 0.]);
        let box_id = box_entity.id;
        let mut document = Document::with_template_tolerance("grouped surface UI");
        document.entities.insert(box_id, box_entity);
        let line = Entity::new("moving line", Geometry::Curve(Curve::Line {
            start: [-1., 0., 0.], end: [1., 0., 0.], domain: [0., 2.],
        }), None);
        let line_id = line.id;
        if gumball { document.entities.insert(line_id, line); }
        let mut app = MatrixApp::new(document).unwrap();
        for _ in 0..3 { frame(&mut app, &context, vec![]); }
        app.controller.workspace.active = ViewId::LookingDown;
        if gumball {
            app.controller.select_objects(&[line_id]).unwrap();
            app.controller.submit("Gumball Snappy").unwrap();
        } else { app.controller.submit("Line").unwrap(); }
        app.set_snap_override(Some(SnapKind::Surface));
        frame(&mut app, &context, vec![]);
        let end = screen_of(&app, [14., 14., 10.]);
        let camera = &app.controller.workspace.views[&ViewId::LookingDown].camera;
        let candidates = om9_scene::osnap::candidates(app.controller.scene().unwrap(), camera, viewport(&app),
            [end.x as f64, end.y as f64], Default::default(), None).unwrap();
        assert_eq!(candidates.iter().filter(|c| c.kind == SnapKind::Surface && c.root == Some(box_id)).count(), 2);
        if gumball {
            let start = screen_of(&app, [0., 0., 0.]);
            frame(&mut app, &context, vec![Event::PointerMoved(start), Event::PointerButton { pos: start, button: egui::PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE }]);
            frame(&mut app, &context, vec![Event::PointerMoved(end)]);
            frame(&mut app, &context, vec![Event::PointerButton { pos: end, button: egui::PointerButton::Primary, pressed: false, modifiers: Modifiers::NONE }]);
            let placement = app.controller.document().entities[&line_id].placement;
            assert!((placement.0[2][3] - 10.).abs() < 1e-6);
        } else {
            click(&mut app, &context, end);
            app.controller.submit("0,0,20").unwrap();
            let start = app.controller.document().entities.values().find_map(|entity| match &entity.geometry {
                Geometry::Curve(Curve::Line { start, .. }) => Some(*start), _ => None,
            }).expect("created line");
            assert!((start[2] - 10.).abs() < 1e-6);
        }
    }
}

#[cfg(feature = "occt-brep")]
#[test]
fn line_surface_snap_refines_depth_before_choosing_close_surfaces() {
    let context = egui::Context::default();
    let limits = om9_core::budget::Limits::default();
    let coarse = om9_kernel::brep::sphere_brep(3., 0.5, &limits).unwrap();
    let fine = om9_kernel::brep::sphere_brep(3., 0.001, &limits).unwrap();
    let mut front = Entity::new("front coarse", Geometry::Brep(coarse), None);
    front.placement = om9_core::geometry::Transform::translation([0., 0., 0.01]);
    let front_id = front.id;
    let back = Entity::new("back fine", Geometry::Brep(fine), None);
    let back_id = back.id;
    let mut document = Document::with_template_tolerance("close overlapping surfaces");
    document.entities.insert(front.id, front);
    document.entities.insert(back.id, back);
    let mut app = MatrixApp::new(document).unwrap();
    frame(&mut app, &context, vec![]);
    frame(&mut app, &context, vec![]);
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.submit("Line").unwrap();
    app.set_snap_override(Some(SnapKind::Surface));
    let point = screen_of(&app, [1.278, 0.822, 0.01 + (9_f64 - 1.278_f64.powi(2) - 0.822_f64.powi(2)).sqrt()]);
    let camera = &app.controller.workspace.views[&ViewId::LookingDown].camera;
    let candidates = om9_scene::osnap::candidates(app.controller.scene().unwrap(), camera, viewport(&app),
        [point.x as f64, point.y as f64], Default::default(), None).unwrap();
    let first = candidates.iter().find(|candidate| candidate.kind == SnapKind::Surface).unwrap();
    assert_eq!(first.root, Some(back_id), "fixture must reverse the display-depth ordering");
    frame(&mut app, &context, vec![Event::PointerMoved(point)]);
    let snap = app.snap_candidate().expect("refined surface snap").clone();
    assert_eq!(snap.root, Some(front_id), "exact front surface must win over its coarser display mesh");
    click(&mut app, &context, point);
    app.controller.submit("20,20,20").unwrap();
    let start = app.controller.document().entities.values().find_map(|entity| match &entity.geometry {
        Geometry::Curve(Curve::Line { start, .. }) => Some(*start), _ => None,
    }).expect("created line");
    assert!(start.iter().zip(snap.world).all(|(a,b)| (a-b).abs() < 1e-7));
}

#[cfg(feature = "occt-brep")]
#[test]
fn snappy_gumball_refines_depth_for_close_surfaces() {
    let context = egui::Context::default();
    let limits = om9_core::budget::Limits::default();
    let mut front = Entity::new("front coarse", Geometry::Brep(om9_kernel::brep::sphere_brep(3., 0.5, &limits).unwrap()), None);
    front.placement = om9_core::geometry::Transform::translation([10., 10., 0.01]);
    let mut back = Entity::new("back fine", Geometry::Brep(om9_kernel::brep::sphere_brep(3., 0.001, &limits).unwrap()), None);
    back.placement = om9_core::geometry::Transform::translation([10., 10., 0.]);
    let line = Entity::new("moving line", Geometry::Curve(Curve::Line {
        start: [-1., 0., 0.], end: [1., 0., 0.], domain: [0., 2.],
    }), None);
    let line_id = line.id;
    let mut document = Document::with_template_tolerance("gumball close surfaces");
    for entity in [front, back, line] { document.entities.insert(entity.id, entity); }
    let mut app = MatrixApp::new(document).unwrap();
    for _ in 0..3 { frame(&mut app, &context, vec![]); }
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.select_objects(&[line_id]).unwrap();
    app.controller.submit("Gumball Snappy").unwrap();
    app.set_snap_override(Some(SnapKind::Surface));
    frame(&mut app, &context, vec![]);
    let start = screen_of(&app, [0., 0., 0.]);
    let end = screen_of(&app, [11.278, 10.822, 0.01 + (9_f64 - 1.278_f64.powi(2) - 0.822_f64.powi(2)).sqrt()]);
    frame(&mut app, &context, vec![Event::PointerMoved(start), Event::PointerButton { pos: start, button: egui::PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE }]);
    frame(&mut app, &context, vec![Event::PointerMoved(end)]);
    frame(&mut app, &context, vec![Event::PointerButton { pos: end, button: egui::PointerButton::Primary, pressed: false, modifiers: Modifiers::NONE }]);
    let placement = app.controller.document().entities[&line_id].placement;
    let delta = [placement.0[0][3] - 10., placement.0[1][3] - 10., placement.0[2][3] - 0.01];
    let radius = delta.iter().map(|value| value * value).sum::<f64>().sqrt();
    assert!((radius - 3.).abs() < 1e-6, "Gumball must snap to the front surface: {radius}");
}

#[cfg(feature = "occt-brep")]
#[test]
fn line_surface_snap_chooses_the_front_of_overlapping_objects() {
    let context = egui::Context::default();
    let limits = om9_core::budget::Limits::default();
    let sphere = om9_kernel::brep::sphere_brep(3., 0.1, &limits).unwrap();
    let first = Entity::new("first", Geometry::Brep(sphere.clone()), None);
    let second = Entity::new("second", Geometry::Brep(sphere), None);
    let (mut back, mut front) = if first.id < second.id { (first, second) } else { (second, first) };
    back.placement = om9_core::geometry::Transform::translation([0., 0., -5.]);
    front.placement = om9_core::geometry::Transform::translation([0., 0., 5.]);
    let front_id = front.id;
    let mut document = Document::with_template_tolerance("overlap surface snap");
    document.entities.insert(back.id, back);
    document.entities.insert(front.id, front);
    let mut app = MatrixApp::new(document).unwrap();
    frame(&mut app, &context, vec![]);
    frame(&mut app, &context, vec![]);
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.submit("Line").unwrap();
    app.set_snap_override(Some(SnapKind::Surface));
    let point = screen_of(&app, [1., 0.5, 5. + (9_f64 - 1. - 0.25).sqrt()]);
    frame(&mut app, &context, vec![Event::PointerMoved(point)]);
    let snap = app.snap_candidate().expect("front sphere snap").clone();
    assert_eq!(snap.root, Some(front_id));
    assert!(snap.world[2] > 5.);
    click(&mut app, &context, point);
    app.controller.submit("20,20,20").unwrap();
    let start = app.controller.document().entities.values().find_map(|entity| match &entity.geometry {
        Geometry::Curve(Curve::Line { start, .. }) => Some(*start), _ => None,
    }).expect("created line");
    assert!(start.iter().zip(snap.world).all(|(a,b)| (a-b).abs() < 1e-7));
}

#[cfg(feature = "occt-brep")]
#[test]
fn snappy_gumball_refuses_a_surface_behind_shaded_mesh_or_hatch() {
    for is_hatch in [false, true] {
    let context = egui::Context::default();
    let limits = om9_core::budget::Limits::default();
    let mut sphere = Entity::new("sphere", Geometry::Brep(om9_kernel::brep::sphere_brep(3., 0.1, &limits).unwrap()), None);
    sphere.placement = om9_core::geometry::Transform::translation([10., 10., 0.]);
    let display = om9_core::geometry::Mesh {
        vertices: vec![[5., 5., 10.], [15., 5., 10.], [15., 15., 10.], [5., 15., 10.]],
        faces: vec![vec![0, 1, 2, 3]], normals: vec![], colors: vec![], corner_uvs: vec![],
    };
    let geometry = if is_hatch {
        Geometry::Hatch(om9_core::geometry::Hatch {
            loops: vec![display.vertices.clone()], display, pattern: None, color: [220, 220, 220, 255],
        })
    } else { Geometry::Mesh(display) };
    let mesh = Entity::new("occluder", geometry, None);
    let line = Entity::new("moving line", Geometry::Curve(Curve::Line {
        start: [-1., 0., 0.], end: [1., 0., 0.], domain: [0., 2.],
    }), None);
    let line_id = line.id;
    let mut document = Document::with_template_tolerance("gumball mesh occluder");
    for entity in [sphere, mesh, line] { document.entities.insert(entity.id, entity); }
    let mut app = MatrixApp::new(document).unwrap();
    for _ in 0..3 { frame(&mut app, &context, vec![]); }
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.workspace.views.get_mut(&ViewId::LookingDown).unwrap().mode = om9_scene::workspace::DisplayMode::Shaded;
    app.controller.select_objects(&[line_id]).unwrap();
    app.controller.submit("Gumball Snappy").unwrap();
    app.set_snap_override(Some(SnapKind::Surface));
    frame(&mut app, &context, vec![]);
    let start = screen_of(&app, [0., 0., 0.]);
    let end = screen_of(&app, [11., 10.5, (9_f64 - 1. - 0.25).sqrt()]);
    frame(&mut app, &context, vec![Event::PointerMoved(start), Event::PointerButton { pos: start, button: egui::PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE }]);
    frame(&mut app, &context, vec![Event::PointerMoved(end)]);
    frame(&mut app, &context, vec![Event::PointerButton { pos: end, button: egui::PointerButton::Primary, pressed: false, modifiers: Modifiers::NONE }]);
    assert_eq!(app.controller.document().entities[&line_id].placement, om9_core::geometry::Transform::identity());
    assert_eq!(app.controller.coordinator().undo_len(), 0);
    assert!(app.status_line().contains("occluded"), "{}", app.status_line());
    }
}

#[cfg(feature = "occt-brep")]
#[test]
fn shaded_mesh_in_front_of_brep_refuses_surface_snap_behind_it() {
    use om9_scene::workspace::DisplayMode;
    for (z, mode, hidden, selectable, blocked) in [
        (10., DisplayMode::Shaded, false, true, true),
        (10., DisplayMode::Shaded, false, false, true),
        (-10., DisplayMode::Shaded, false, true, false),
        (10., DisplayMode::Wireframe, false, true, false),
        (10., DisplayMode::Shaded, true, true, false),
        (99.995, DisplayMode::Shaded, false, true, false),
    ] {
    let context = egui::Context::default();
    let limits = om9_core::budget::Limits::default();
    let sphere = Entity::new("sphere", Geometry::Brep(om9_kernel::brep::sphere_brep(3., 0.1, &limits).unwrap()), None);
    let mut mesh = Entity::new("occluder", Geometry::Mesh(om9_core::geometry::Mesh {
        vertices: vec![[-5., -5., z], [5., -5., z], [5., 5., z], [-5., 5., z]],
        faces: vec![vec![0, 1, 2, 3]], normals: vec![], colors: vec![], corner_uvs: vec![],
    }), None);
    mesh.attributes.hidden = hidden;
    mesh.attributes.selectable = selectable;
    let mut document = Document::with_template_tolerance("shaded mesh occluder");
    document.entities.insert(sphere.id, sphere);
    document.entities.insert(mesh.id, mesh);
    let mut app = MatrixApp::new(document).unwrap();
    for _ in 0..3 { frame(&mut app, &context, vec![]); }
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.workspace.views.get_mut(&ViewId::LookingDown).unwrap().mode = mode;
    app.controller.submit("Line").unwrap();
    app.set_snap_override(Some(SnapKind::Surface));
    let point = screen_of(&app, [1., 0.5, (9_f64 - 1. - 0.25).sqrt()]);
    click(&mut app, &context, point);
    if blocked {
        assert!(app.snap_candidate().is_none(), "must not snap through a visible shaded mesh");
        assert!(app.status_line().contains("Surface snap unavailable"), "{}", app.status_line());
    } else {
        assert!(app.snap_candidate().is_some(), "unobstructed surface must remain available: z={z}, mode={mode:?}, hidden={hidden}");
    }
    assert_eq!(app.controller.document().entities.len(), 2);
    assert_eq!(app.controller.coordinator().undo_len(), 0);
    }
}

#[cfg(feature = "occt-brep")]
#[test]
fn unselectable_brep_remains_a_surface_reference_when_visible() {
    use om9_scene::workspace::DisplayMode;
    for (z, mode, hidden, front_reference) in [
        (10., DisplayMode::Shaded, false, true),
        (-10., DisplayMode::Shaded, false, false),
        (10., DisplayMode::Wireframe, false, true),
        (10., DisplayMode::Shaded, true, false),
    ] {
        let context = egui::Context::default();
        let limits = om9_core::budget::Limits::default();
        let sphere = Entity::new("target", Geometry::Brep(om9_kernel::brep::sphere_brep(3., 0.1, &limits).unwrap()), None);
        let mut occluder = Entity::new("unselectable", Geometry::Brep(om9_kernel::brep::sphere_brep(3., 0.1, &limits).unwrap()), None);
        occluder.placement = om9_core::geometry::Transform::translation([0., 0., z]);
        occluder.attributes.selectable = false;
        occluder.attributes.hidden = hidden;
        let mut document = Document::with_template_tolerance("unselectable brep occluder");
        document.entities.insert(sphere.id, sphere);
        document.entities.insert(occluder.id, occluder);
        let mut app = MatrixApp::new(document).unwrap();
        for _ in 0..3 { frame(&mut app, &context, vec![]); }
        app.controller.workspace.active = ViewId::LookingDown;
        app.controller.workspace.views.get_mut(&ViewId::LookingDown).unwrap().mode = mode;
        app.controller.submit("Line").unwrap();
        app.set_snap_override(Some(SnapKind::Surface));
        let point = screen_of(&app, [1., 0.5, (9_f64 - 1. - 0.25).sqrt()]);
        click(&mut app, &context, point);
        let candidate = app.snap_candidate().expect("a visible sphere must be a surface reference");
        assert_eq!(candidate.world[2] > 5., front_reference, "z={z}, mode={mode:?}, hidden={hidden}");
        assert_eq!(app.controller.coordinator().undo_len(), 0);
    }
}

#[cfg(feature = "occt-brep")]
#[test]
fn shaded_hatch_blocks_brep_surface_snap() {
    use om9_scene::workspace::DisplayMode;
    for (z, mode, hidden, blocked) in [
        (10., DisplayMode::Shaded, false, true),
        (-10., DisplayMode::Shaded, false, false),
        (10., DisplayMode::Wireframe, false, false),
        (10., DisplayMode::Shaded, true, false),
    ] {
        let context = egui::Context::default();
        let limits = om9_core::budget::Limits::default();
        let sphere = Entity::new("target", Geometry::Brep(om9_kernel::brep::sphere_brep(3., 0.1, &limits).unwrap()), None);
        let vertices = vec![[-5., -5., z], [5., -5., z], [5., 5., z], [-5., 5., z]];
        let hatch = om9_core::geometry::Hatch {
            display: om9_core::geometry::Mesh {
                vertices: vertices.clone(), faces: vec![vec![0, 1, 2, 3]],
                normals: vec![], colors: vec![], corner_uvs: vec![],
            },
            loops: vec![vertices], pattern: None, color: [220, 220, 220, 255],
        };
        let mut occluder = Entity::new("hatch occluder", Geometry::Hatch(hatch), None);
        occluder.attributes.hidden = hidden;
        let mut document = Document::with_template_tolerance("hatch occluder");
        document.entities.insert(sphere.id, sphere);
        document.entities.insert(occluder.id, occluder);
        let mut app = MatrixApp::new(document).unwrap();
        for _ in 0..3 { frame(&mut app, &context, vec![]); }
        app.controller.workspace.active = ViewId::LookingDown;
        app.controller.workspace.views.get_mut(&ViewId::LookingDown).unwrap().mode = mode;
        app.controller.submit("Line").unwrap();
        app.set_snap_override(Some(SnapKind::Surface));
        let point = screen_of(&app, [1., 0.5, (9_f64 - 1. - 0.25).sqrt()]);
        click(&mut app, &context, point);
        if blocked {
            assert!(app.snap_candidate().is_none(), "must not snap through a visible shaded hatch");
            assert!(app.status_line().contains("occluded"), "{}", app.status_line());
        } else {
            assert!(app.snap_candidate().is_some(), "unobstructed sphere must remain available");
        }
        assert_eq!(app.controller.coordinator().undo_len(), 0);
    }
}

#[cfg(feature = "occt-brep")]
#[test]
fn nurbs_surface_snap_selects_the_front_canonical_surface() {
    use om9_scene::workspace::DisplayMode;
    for (z, mode, hidden, front) in [
        (10., DisplayMode::Shaded, false, true),
        (-10., DisplayMode::Shaded, false, false),
        (10., DisplayMode::Wireframe, false, true),
        (10., DisplayMode::Shaded, true, false),
    ] {
        let context = egui::Context::default();
        let limits = om9_core::budget::Limits::default();
        let sphere = Entity::new("sphere", Geometry::Brep(om9_kernel::brep::sphere_brep(3., 0.1, &limits).unwrap()), None);
        let surface = om9_core::surface_kernel::plane([-5., -5., z], [1., 0., 0.], [0., 1., 0.], 10., 10., &limits).unwrap();
        let mut surface = Entity::new("surface occluder", Geometry::Surface(surface), None);
        surface.attributes.hidden = hidden;
        let mut document = Document::with_template_tolerance("surface occluder");
        document.entities.insert(sphere.id, sphere);
        document.entities.insert(surface.id, surface);
        let mut app = MatrixApp::new(document).unwrap();
        for _ in 0..3 { frame(&mut app, &context, vec![]); }
        app.controller.workspace.active = ViewId::LookingDown;
        app.controller.workspace.views.get_mut(&ViewId::LookingDown).unwrap().mode = mode;
        app.controller.submit("Line").unwrap();
        app.set_snap_override(Some(SnapKind::Surface));
        let point = screen_of(&app, [1., 0.5, (9_f64 - 1. - 0.25).sqrt()]);
        click(&mut app, &context, point);
        let snap = app.snap_candidate().expect("visible front surface must be available");
        if front { assert!((snap.world[2] - 10.).abs() < 1e-7, "{:?}", snap.world); }
        else { assert!((snap.world.iter().map(|v| v * v).sum::<f64>() - 9.).abs() < 1e-6, "{:?}", snap.world); }
        assert_eq!(app.controller.coordinator().undo_len(), 0);
    }
}

#[cfg(feature = "occt-brep")]
#[test]
fn nurbs_surface_snap_refines_curved_source_not_display_triangles() {
    let context = egui::Context::default();
    let surface = om9_core::geometry::NurbsSurface {
        degree: [2, 1], size: [3, 2],
        poles: vec![[-5., -5., 0.], [-5., 5., 0.], [0., -5., 4.], [0., 5., 4.], [5., -5., 0.], [5., 5., 0.]],
        weights: vec![1.; 6], knots: [vec![0., 0., 0., 1., 1., 1.], vec![0., 0., 1., 1.]],
        domain: [[0., 1.], [0., 1.]], periodic: [false, false], reversed: false,
    };
    let mut document = Document::with_template_tolerance("curved NURBS snap");
    let entity = Entity::new("curved surface", Geometry::Surface(surface), None);
    document.entities.insert(entity.id, entity);
    let mut app = MatrixApp::new(document).unwrap();
    for _ in 0..3 { frame(&mut app, &context, vec![]); }
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.workspace.views.get_mut(&ViewId::LookingDown).unwrap().mode = om9_scene::workspace::DisplayMode::Shaded;
    app.controller.submit("Line").unwrap();
    app.set_snap_override(Some(SnapKind::Surface));
    let point = screen_of(&app, [1., 0.5, 1.92]);
    click(&mut app, &context, point);
    let hit = app.snap_candidate().expect("curved NURBS surface snap").world;
    let analytic_z = 2. * (1. - hit[0] * hit[0] / 25.);
    assert!((hit[2] - analytic_z).abs() < 1e-7, "{hit:?} expected z={analytic_z}");
}

#[cfg(feature = "occt-brep")]
#[test]
fn overlapping_unbound_surface_is_refused_instead_of_guessing_depth() {
    let context = egui::Context::default();
    let limits = om9_core::budget::Limits::default();
    let sphere = om9_kernel::brep::sphere_brep(3., 0.1, &limits).unwrap();
    let native = om9_kernel::brep::to_3dm(&sphere, 0.001, &limits).unwrap();
    let mut front = Entity::new("bound", Geometry::Brep(sphere), None);
    front.placement = om9_core::geometry::Transform::translation([0., 0., 5.]);
    let mut back = Entity::new("unbound", Geometry::NativeBrep(native), None);
    back.placement = om9_core::geometry::Transform::translation([0., 0., -5.]);
    let mut document = Document::with_template_tolerance("unbound overlapping surface");
    document.entities.insert(front.id, front);
    document.entities.insert(back.id, back);
    let mut app = MatrixApp::new(document).unwrap();
    for _ in 0..3 { frame(&mut app, &context, vec![]); }
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.submit("Line").unwrap();
    app.set_snap_override(Some(SnapKind::Surface));
    let point = screen_of(&app, [1., 0.5, 5. + (9_f64 - 1. - 0.25).sqrt()]);
    click(&mut app, &context, point);
    assert!(app.snap_candidate().is_none());
    assert!(app.status_line().contains("Surface snap unavailable"), "{}", app.status_line());
    assert_eq!(app.controller.document().entities.len(), 2);
    assert_eq!(app.controller.coordinator().undo_len(), 0);
}

#[cfg(feature = "occt-brep")]
#[test]
fn trimmed_cylinder_snap_starts_line_through_front_hole() {
    use om9_core::{budget::Limits,rhino_semantics::{set_trimmed_surface,TrimmedSurface}};
    use om9_scene::camera::Camera;
    let limits=Limits::default();let context=egui::Context::default();
    let surface=om9_core::surface_kernel::cylinder([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,5.,&limits).unwrap();
    let trim=TrimmedSurface {underlying:surface.clone(),outer:vec![[0.,0.],[1.,0.],[1.,1.],[0.,1.]],
        inners:vec![vec![[0.01,0.4],[0.08,0.4],[0.08,0.6],[0.01,0.6]]],reversed:false};
    let mut entity=Entity::new("trim cylinder",Geometry::Surface(surface),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();
    let mut doc=Document::with_template_tolerance("trimmed snap line");doc.entities.insert(entity.id,entity);
    let path=std::env::temp_dir().join(format!("om9-trim-snap-line-{}.om9",uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path,&doc,&limits,&om9_core::transaction::Cancellation::default()).unwrap();
    let mut app=MatrixApp::from_file(&path).unwrap();std::fs::remove_file(path).unwrap();
    for _ in 0..3 {frame(&mut app,&context,vec![]);}
    app.controller.workspace.active=ViewId::LookingDown;
    app.controller.workspace.views.get_mut(&ViewId::LookingDown).unwrap().camera=
        Camera::orthographic([10.,0.4,2.5],[0.,0.4,2.5],[0.,0.,1.],8.).unwrap();
    app.controller.workspace.views.get_mut(&ViewId::LookingDown).unwrap().mode=om9_scene::workspace::DisplayMode::Shaded;
    frame(&mut app,&context,vec![]);let before=app.controller.document().clone();
    app.controller.submit("Line").unwrap();app.set_snap_override(Some(SnapKind::Surface));
    let point=screen_of(&app,[0.,0.4,2.5]);frame(&mut app,&context,vec![Event::PointerMoved(point)]);
    let snap=app.snap_candidate().unwrap_or_else(|| {
        let camera=&app.controller.workspace.views[&ViewId::LookingDown].camera;
        let hits=om9_scene::osnap::candidates(app.controller.scene().unwrap(),camera,viewport(&app),[point.x as f64,point.y as f64],Default::default(),None).unwrap();
        panic!("trimmed rear snap; status {}, pointer {point:?}, candidates {hits:?}",app.status_line())
    }).world;
    assert!(snap[0] < -1.9,"expected rear surface: {snap:?}");
    let miss=screen_of(&app,[0.,3.5,2.5]);frame(&mut app,&context,vec![Event::PointerMoved(miss)]);
    assert!(app.snap_candidate().is_none());click(&mut app,&context,miss);
    assert_eq!(app.controller.document(),&before,"a miss in the parallel view must not commit a stale point");
    frame(&mut app,&context,vec![Event::PointerMoved(point)]);
    click(&mut app,&context,point);app.controller.submit("-3,2,2.5").unwrap();
    let start=app.controller.document().entities.values().find_map(|e|match &e.geometry {
        Geometry::Curve(Curve::Line {start,..})=>Some(*start),_=>None,
    }).expect("created line");
    assert!(start.iter().zip(snap).all(|(a,b)|(a-b).abs()<1e-7));
    let after=app.controller.document().clone();app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
    app.controller.redo().unwrap();assert_eq!(app.controller.document(),&after);
}

#[test]
fn endpoint_snap_works_when_view_ray_is_parallel_to_input_plane() {
    use om9_scene::camera::Camera;
    let context=egui::Context::default();let mut doc=Document::new("parallel endpoint snap");
    let entity=Entity::new("source",Geometry::Curve(Curve::Line {start:[0.,0.4,2.5],end:[0.,1.4,2.5],domain:[0.,1.]}),None);
    doc.entities.insert(entity.id,entity);let mut app=MatrixApp::new(doc).unwrap();
    for _ in 0..3 {frame(&mut app,&context,vec![]);}
    app.controller.workspace.active=ViewId::LookingDown;
    app.controller.workspace.views.get_mut(&ViewId::LookingDown).unwrap().camera=
        Camera::orthographic([10.,0.4,2.5],[0.,0.4,2.5],[0.,0.,1.],8.).unwrap();
    frame(&mut app,&context,vec![]);app.controller.submit("Line").unwrap();app.set_snap_override(Some(SnapKind::End));
    let point=screen_of(&app,[0.,0.4,2.5]);frame(&mut app,&context,vec![Event::PointerMoved(point)]);
    let snap=app.snap_candidate().expect("endpoint snap");assert_eq!(snap.kind,SnapKind::End);
    assert!(snap.world.iter().zip([0.,0.4,2.5]).all(|(a,b)|(a-b).abs()<1e-7));
}

#[cfg(feature = "occt-brep")]
#[test]
fn surface_snap_accepts_certified_full_cylinder_seam() {
    use om9_core::{budget::Limits,rhino_semantics::{set_trimmed_surface,TrimmedSurface}};
    use om9_scene::{camera::Camera,osnap};
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::cylinder([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,5.,&limits).unwrap();
    let trim=TrimmedSurface {underlying:surface.clone(),outer:vec![[0.,0.2],[1.,0.2],[1.,0.8],[0.,0.8]],inners:vec![],reversed:false};
    for placement in [om9_core::geometry::Transform::identity(),om9_core::geometry::Transform::translation([5.,-3.,7.])] {
        let mut entity=Entity::new("full seam",Geometry::Surface(surface.clone()),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();entity.placement=placement;let id=entity.id;
        let mut doc=Document::with_template_tolerance("seam snap");doc.entities.insert(id,entity);
        let path=std::env::temp_dir().join(format!("om9-seam-snap-{}.om9",uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&path,&doc,&limits,&om9_core::transaction::Cancellation::default()).unwrap();
        let app=MatrixApp::from_file(&path).unwrap();std::fs::remove_file(path).unwrap();
        let camera=Camera::orthographic(placement.point([10.,0.,2.5]).unwrap(),placement.point([0.,0.,2.5]).unwrap(),[0.,0.,1.],8.).unwrap();
        let viewport=Viewport {origin:[0.,0.],size:[400.,400.]};
        let candidates=osnap::candidates(app.controller.scene().unwrap(),&camera,viewport,[200.,200.],Default::default(),None).unwrap();
        let candidate=candidates.iter().find(|c|c.kind==SnapKind::Surface && c.root==Some(id)).unwrap();
        let hit=app.controller.refine_surface_snap(candidate,&camera,viewport).unwrap();let expected=placement.point([2.,0.,2.5]).unwrap();
        assert!(hit.world.iter().zip(expected).all(|(a,b)|(a-b).abs()<1e-7));
    }
}

#[cfg(feature = "occt-brep")]
#[test]
fn surface_snap_skips_front_hole_and_finds_rear_cylinder() {
    use om9_core::{budget::Limits,rhino_semantics::{set_trimmed_surface,TrimmedSurface}};
    use om9_scene::{camera::Camera,osnap};
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::cylinder([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,5.,&limits).unwrap();
    let trim=TrimmedSurface {underlying:surface.clone(),outer:vec![[0.,0.],[1.,0.],[1.,1.],[0.,1.]],
        inners:vec![vec![[0.01,0.4],[0.08,0.4],[0.08,0.6],[0.01,0.6]]],reversed:false};
    let affine=om9_core::geometry::Transform([[2.,0.5,0.,4.],[0.25,1.,0.,-3.],[0.,0.,1.,7.],[0.,0.,0.,1.]]);
    for placement in [om9_core::geometry::Transform::identity(),affine] {
        let mut entity=Entity::new("trim cylinder",Geometry::Surface(surface.clone()),None);
        set_trimmed_surface(&mut entity,&trim,&limits).unwrap();entity.placement=placement;let id=entity.id;
        let mut doc=Document::with_template_tolerance("trim snap multi-hit");doc.entities.insert(id,entity);
        let path=std::env::temp_dir().join(format!("om9-trim-snap-{}.om9",uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&path,&doc,&limits,&om9_core::transaction::Cancellation::default()).unwrap();
        let mut app=MatrixApp::from_file(&path).unwrap();std::fs::remove_file(path).unwrap();
        app.controller.submit("Point").unwrap();app.controller.submit("100,100,100").unwrap();app.controller.undo().unwrap();
        assert_eq!(app.controller.coordinator().redo_len(),1);
        let camera=Camera::orthographic(placement.point([10.,0.4,2.5]).unwrap(),placement.point([0.,0.4,2.5]).unwrap(),[0.,0.,1.],12.).unwrap();
        let viewport=Viewport {origin:[0.,0.],size:[400.,400.]};
        let candidates=osnap::candidates(app.controller.scene().unwrap(),&camera,viewport,[200.,200.],Default::default(),None).unwrap();
        let candidate=candidates.iter().find(|c|c.kind==SnapKind::Surface && c.root==Some(id)).unwrap();
        let expected=placement.point([-((4_f64-0.16).sqrt()),0.4,2.5]).unwrap();
        let check=|point:[f64;3]|assert!(point.iter().zip(expected).all(|(a,b)|(a-b).abs()<1e-6),"expected {expected:?}, got {point:?}");
        check(app.controller.refine_surface_snap(candidate,&camera,viewport).unwrap().world);
        let before=app.controller.document().clone();
        let pressure=app.controller.coordinator().reserve_remaining_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::Kernel,None,app.controller.scene_cache_usage().unwrap()).unwrap();
        assert!(app.controller.refine_surface_snap(candidate,&camera,viewport).is_err());
        assert_eq!(app.controller.document(),&before);
        assert_eq!(app.controller.coordinator().redo_len(),1);
        drop(pressure);
        check(app.controller.refine_surface_snap(candidate,&camera,viewport).unwrap().world);
        assert_eq!(app.controller.coordinator().redo_len(),1);
    }
}

#[cfg(feature = "occt-brep")]
#[test]
fn fallback_trim_surface_snap_refuses_unverified_source_region() {
    use om9_core::{budget::Limits,rhino_semantics::{set_trimmed_surface,TrimmedSurface}};
    use om9_scene::{camera::Camera,osnap};
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::plane([-5.,-5.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    let trim=TrimmedSurface {underlying:surface.clone(),outer:vec![[0.,0.],[1.,0.],[1.,1.],[0.,1.]],
        inners:vec![vec![[0.1,0.1],[0.9,0.1],[0.9,0.9],[0.1,0.9]],vec![[0.3,0.3],[0.7,0.3],[0.7,0.7],[0.3,0.7]]],reversed:false};
    let mut entity=Entity::new("fallback",Geometry::Surface(surface),None);
    set_trimmed_surface(&mut entity,&trim,&limits).unwrap();let id=entity.id;
    let mut doc=Document::with_template_tolerance("fallback snap");doc.entities.insert(id,entity);
    let mut app=MatrixApp::new(doc).unwrap();
    let camera=Camera::orthographic([0.,0.,10.],[0.,0.,0.],[0.,1.,0.],10.).unwrap();
    let viewport=Viewport {origin:[0.,0.],size:[400.,400.]};
    let candidates=osnap::candidates(app.controller.scene().unwrap(),&camera,viewport,[200.,200.],Default::default(),None).unwrap();
    assert!(!candidates.iter().any(|c|c.kind==SnapKind::Surface && c.root==Some(id)),"must not offer an unverified fallback surface");
    let context=egui::Context::default();for _ in 0..3 {frame(&mut app,&context,vec![]);}
    app.controller.workspace.active=ViewId::LookingDown;app.controller.submit("Line").unwrap();
    app.set_snap_override(Some(SnapKind::Surface));let point=screen_of(&app,[0.,0.,0.]);click(&mut app,&context,point);
    assert!(app.snap_candidate().is_none());assert_eq!(app.controller.document().entities.len(),1);
    assert_eq!(app.controller.coordinator().undo_len(),0);
    let surface=om9_core::surface_kernel::plane([-5.,-5.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    let valid=TrimmedSurface {underlying:surface.clone(),outer:vec![[0.1,0.1],[0.9,0.1],[0.9,0.9],[0.1,0.9]],inners:vec![],reversed:false};
    let mut entity=Entity::new("valid",Geometry::Surface(surface),None);set_trimmed_surface(&mut entity,&valid,&limits).unwrap();let id=entity.id;
    let mut doc=Document::with_template_tolerance("valid snap");doc.entities.insert(id,entity);let app=MatrixApp::new(doc).unwrap();
    let candidates=osnap::candidates(app.controller.scene().unwrap(),&camera,viewport,[200.,200.],Default::default(),None).unwrap();
    let candidate=candidates.iter().find(|c|c.kind==SnapKind::Surface && c.root==Some(id)).unwrap();
    assert!(app.controller.refine_surface_snap(candidate,&camera,viewport).is_ok());
}

#[cfg(feature = "occt-brep")]
#[test]
fn surface_snap_considers_valid_surface_behind_unverified_trim() {
    use om9_core::{budget::Limits,rhino_semantics::{set_trimmed_surface,TrimmedSurface}};
    use om9_scene::workspace::DisplayMode;
    for (mode,x,expected) in [(DisplayMode::Wireframe,1.5,true),(DisplayMode::Shaded,1.5,true),
        (DisplayMode::Shaded,1.2001,false),(DisplayMode::Shaded,0.9,false)] {
        let limits=Limits::default();let surface=om9_core::surface_kernel::plane([-2.,-2.,0.],[1.,0.,0.],[0.,1.,0.],4.,4.,&limits).unwrap();
        let trim=TrimmedSurface {underlying:surface.clone(),outer:vec![[0.2,0.2],[0.8,0.2],[0.8,0.8],[0.2,0.8]],
            inners:vec![vec![[0.3,0.3],[0.7,0.3],[0.7,0.7],[0.3,0.7]],vec![[0.4,0.4],[0.6,0.4],[0.6,0.6],[0.4,0.6]]],reversed:false};
        let mut front=Entity::new("fallback",Geometry::Surface(surface.clone()),None);
        set_trimmed_surface(&mut front,&trim,&limits).unwrap();
        let mut back=Entity::new("back",Geometry::Surface(surface),None);
        back.placement=om9_core::geometry::Transform::translation([0.,0.,-1.]);let back_id=back.id;
        let mut doc=Document::with_template_tolerance("fallback snap behind");doc.entities.insert(front.id,front);doc.entities.insert(back.id,back);
        let mut app=MatrixApp::new(doc).unwrap();app.controller.submit("Point").unwrap();app.controller.submit("100,100,100").unwrap();app.controller.undo().unwrap();
        let before=app.controller.document().clone();let context=egui::Context::default();
        for _ in 0..3 {frame(&mut app,&context,vec![]);}
        app.controller.workspace.active=ViewId::LookingDown;
        app.controller.workspace.views.get_mut(&ViewId::LookingDown).unwrap().mode=mode;
        app.controller.submit("Line").unwrap();app.set_snap_override(Some(SnapKind::Surface));
        let point=screen_of(&app,[x,0.,-1.]);click(&mut app,&context,point);
        assert_eq!(app.snap_candidate().is_some(),expected,"mode={mode:?}, x={x}: {}",app.status_line());
        if expected {assert_eq!(app.snap_candidate().unwrap().root,Some(back_id));}
        assert_eq!(app.controller.document(),&before);assert_eq!(app.controller.coordinator().redo_len(),1);
    }
}

#[cfg(feature = "occt-brep")]
#[test]
fn surface_snap_reaches_back_surface_through_verified_fallback_hole_after_reopen() {
    use om9_core::{budget::Limits,rhino_semantics::{set_trimmed_surface,TrimmedSurface},transaction::Cancellation};
    use om9_scene::workspace::DisplayMode;
    for (mode,x,expected,concave) in [(DisplayMode::Wireframe,0.,true,false),(DisplayMode::Shaded,0.,true,false),
        (DisplayMode::Shaded,0.8,false,false),(DisplayMode::Wireframe,0.,true,true),
        (DisplayMode::Shaded,0.,true,true),(DisplayMode::Shaded,0.8,false,true)] {
        let limits=Limits::default();let surface=om9_core::surface_kernel::plane([-2.,-2.,0.],[1.,0.,0.],[0.,1.,0.],4.,4.,&limits).unwrap();
        let outer=(0..1025).map(|i|{let a=std::f64::consts::TAU*(i as f64)/1025.;[0.5+0.4*a.cos(),0.5+0.4*a.sin()]}).collect();
        let hole=if concave {vec![[0.4,0.4],[0.6,0.4],[0.6,0.6],[0.5,0.53],[0.4,0.6]]}
            else {vec![[0.4,0.4],[0.6,0.4],[0.6,0.6],[0.4,0.6]]};
        let trim=TrimmedSurface {underlying:surface.clone(),outer,inners:vec![hole],reversed:false};
        let mut front=Entity::new("fallback with hole",Geometry::Surface(surface.clone()),None);set_trimmed_surface(&mut front,&trim,&limits).unwrap();
        let mut back=Entity::new("back",Geometry::Surface(surface),None);back.placement=om9_core::geometry::Transform::translation([0.,0.,-1.]);let back_id=back.id;
        let mut doc=Document::with_template_tolerance("fallback hole snap");doc.entities.insert(front.id,front);doc.entities.insert(back.id,back);
        let path=std::env::temp_dir().join(format!("om9-hole-snap-{}.om9",uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&path,&doc,&limits,&Cancellation::default()).unwrap();let mut app=MatrixApp::from_file(&path).unwrap();std::fs::remove_file(path).unwrap();
        assert!(app.controller.scene().unwrap().objects.iter().any(|o|o.buffers.is_trim_display_fallback()));
        app.controller.submit("Point").unwrap();app.controller.submit("100,100,100").unwrap();app.controller.undo().unwrap();let before=app.controller.document().clone();
        let context=egui::Context::default();for _ in 0..3 {frame(&mut app,&context,vec![]);}
        app.controller.workspace.active=ViewId::LookingDown;app.controller.workspace.views.get_mut(&ViewId::LookingDown).unwrap().mode=mode;
        app.controller.submit("Line").unwrap();app.set_snap_override(Some(SnapKind::Surface));let point=screen_of(&app,[x,0.,-1.]);click(&mut app,&context,point);
        assert_eq!(app.snap_candidate().is_some(),expected,"mode={mode:?}, x={x}: {}",app.status_line());
        if expected {assert_eq!(app.snap_candidate().unwrap().root,Some(back_id));}
        assert_eq!(app.controller.document(),&before);assert_eq!(app.controller.coordinator().redo_len(),1);
    }
}

#[cfg(feature = "occt-brep")]
#[test]
fn surface_snap_hits_reopened_many_edge_trim_material_but_not_hole() {
    use om9_core::{budget::Limits,evaluation::{evaluate_curve,Parameter},rhino_semantics::{set_trimmed_surface,TrimmedSurface},transaction::Cancellation};
    let limits=Limits::default();
    for count in [708,1020] {for (world,expected) in [([8.,5.3,0.],true),([5.,5.3,0.],false)] {
        let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
        let outer=(0..count).map(|i|{let a=std::f64::consts::TAU*i as f64/count as f64;[0.5+0.4*a.cos(),0.5+0.4*a.sin()]}).collect();
        let trim=TrimmedSurface {underlying:surface.clone(),outer,inners:vec![vec![[0.4,0.4],[0.6,0.4],[0.6,0.6],[0.4,0.6]]],reversed:false};
        let mut entity=Entity::new("many edge trim",Geometry::Surface(surface),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();let id=entity.id;
        let mut doc=Document::with_template_tolerance("many edge surface snap");doc.entities.insert(id,entity);
        let path=std::env::temp_dir().join(format!("om9-many-edge-snap-{}.om9",uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&path,&doc,&limits,&Cancellation::default()).unwrap();
        let mut app=MatrixApp::from_file(&path).unwrap();std::fs::remove_file(path).unwrap();
        assert!(!app.controller.scene().unwrap().objects[0].buffers.is_trim_display_fallback());
        app.controller.submit("Point").unwrap();app.controller.submit("100,100,100").unwrap();app.controller.undo().unwrap();let before=app.controller.document().clone();
        let context=egui::Context::default();for _ in 0..3 {frame(&mut app,&context,vec![]);}
        app.controller.workspace.active=ViewId::LookingDown;app.controller.submit("Line").unwrap();app.set_snap_override(Some(SnapKind::Surface));
        let point=screen_of(&app,world);click(&mut app,&context,point);
        assert_eq!(app.snap_candidate().and_then(|c|c.root)==Some(id),expected,"count={count}, world={world:?}: {}",app.status_line());
        assert_eq!(app.controller.document(),&before);assert_eq!(app.controller.coordinator().redo_len(),1);
        if expected {let snap=app.snap_candidate().unwrap().world;app.controller.submit("20,20,20").unwrap();
            let start=app.controller.document().entities.values().find_map(|e|match &e.geometry {
                Geometry::Curve(curve@Curve::Line {..})=>Some(evaluate_curve(curve,Parameter::Fraction(0.),0,e.placement,&limits).unwrap()[0]),_=>None,
            }).unwrap();assert!(start.iter().zip(snap).all(|(a,b)|(a-b).abs()<1e-7));
            let after=app.controller.document().clone();app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);app.controller.redo().unwrap();assert_eq!(app.controller.document(),&after);
        }
    }}
}

#[cfg(feature = "occt-brep")]
#[test]
fn surface_snap_hits_reopened_many_edge_cylinder_trim() {
    use om9_core::{budget::Limits,rhino_semantics::{set_trimmed_surface,TrimmedSurface},surface_kernel,transaction::Cancellation};
    use om9_scene::workspace::DisplayMode;
    let limits=Limits::default();
    let surface=surface_kernel::cylinder([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,5.,&limits).unwrap();
    let outer=(0..1020).map(|i|{let a=std::f64::consts::TAU*i as f64/1020.;[0.5+0.3*a.cos(),0.5+0.3*a.sin()]}).collect();
    let trim=TrimmedSurface {underlying:surface.clone(),outer,inners:vec![vec![[0.4,0.4],[0.6,0.4],[0.6,0.6],[0.4,0.6]]],reversed:false};
    for (mode,u,expected) in [(DisplayMode::Wireframe,0.7,true),(DisplayMode::Shaded,0.7,true),
        (DisplayMode::Wireframe,0.5,false),(DisplayMode::Shaded,0.5,false)] {
        let mut entity=Entity::new("many edge cylinder",Geometry::Surface(surface.clone()),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();let id=entity.id;
        let mut doc=Document::with_template_tolerance("many edge cylinder snap");doc.entities.insert(id,entity);
        let path=std::env::temp_dir().join(format!("om9-cylinder-snap-{}.om9",uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&path,&doc,&limits,&Cancellation::default()).unwrap();
        let mut app=MatrixApp::from_file(&path).unwrap();std::fs::remove_file(path).unwrap();
        assert!(!app.controller.scene().unwrap().objects[0].buffers.is_trim_display_fallback());
        app.controller.submit("Point").unwrap();app.controller.submit("100,100,100").unwrap();app.controller.undo().unwrap();let before=app.controller.document().clone();
        let context=egui::Context::default();for _ in 0..3 {frame(&mut app,&context,vec![]);}
        app.controller.workspace.active=ViewId::LookingDown;app.controller.workspace.views.get_mut(&ViewId::LookingDown).unwrap().mode=mode;
        let point=surface_kernel::evaluate(&surface,u,0.53,&limits).unwrap();
        let eye=[point[0]*5.,point[1]*5.,point[2]];
        app.controller.workspace.views.get_mut(&ViewId::LookingDown).unwrap().camera=om9_scene::camera::Camera::orthographic(eye,point,[0.,0.,1.],10.).unwrap();
        app.controller.submit("Line").unwrap();app.set_snap_override(Some(SnapKind::Surface));
        let screen=screen_of(&app,point);click(&mut app,&context,screen);
        assert_eq!(app.snap_candidate().and_then(|s|s.root)==Some(id),expected,"{mode:?}, u={u}");
        if expected {let snap=app.snap_candidate().unwrap();
            assert!((snap.world[0].hypot(snap.world[1])-2.).abs()<1e-7);assert!((snap.world[2]-2.65).abs()<1e-4);}
        assert_eq!(app.controller.document(),&before);assert_eq!(app.controller.coordinator().redo_len(),1);
        if expected {app.controller.submit("20,20,20").unwrap();let after=app.controller.document().clone();
            app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);app.controller.redo().unwrap();assert_eq!(app.controller.document(),&after);}
    }
}

#[cfg(feature = "occt-brep")]
#[test]
fn surface_snap_reaches_back_through_curved_fallback_after_reopen() {
    use om9_core::{budget::Limits,geometry::Transform,rhino_semantics::{set_trimmed_surface,TrimmedSurface},transaction::Cancellation};
    use om9_scene::workspace::DisplayMode;
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::cylinder([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,5.,&limits).unwrap();
    let placement=Transform([[2.,0.4,0.,4.],[0.,1.,0.,-3.],[0.,0.,1.5,7.],[0.,0.,0.,1.]]);
    for (mode,front_material,hole,z,expected,count) in [
        (DisplayMode::Wireframe,false,false,2.5,true,1025),(DisplayMode::Shaded,false,false,2.5,true,1025),
        (DisplayMode::Shaded,true,false,2.5,false,1025),(DisplayMode::Wireframe,false,true,2.5,true,1025),
        (DisplayMode::Shaded,false,true,2.5,true,1025),(DisplayMode::Shaded,false,true,2.73,false,1025),
        (DisplayMode::Wireframe,false,true,2.5,true,708),(DisplayMode::Shaded,false,true,2.5,true,708)] {
        let center=if hole {0.2}else if front_material {0.05}else{0.5};let radius=if front_material {0.04}else{0.2};
        let outer=(0..count).map(|i| {let a=std::f64::consts::TAU*(i as f64)/(count as f64);[center+radius*a.cos(),0.5+radius*a.sin()]}).collect();
        let inners=if hole {vec![vec![[0.015,0.45],[0.07,0.45],[0.07,0.55],[0.015,0.55]]]} else {vec![]};
        let trim=TrimmedSurface {underlying:surface.clone(),outer,inners,reversed:false};
        let mut front=Entity::new("curved fallback",Geometry::Surface(surface.clone()),None);
        set_trimmed_surface(&mut front,&trim,&limits).unwrap();front.placement=placement;let front_id=front.id;
        let plane=om9_core::surface_kernel::plane([0.,-2.,0.],[0.,1.,0.],[0.,0.,1.],4.,5.,&limits).unwrap();
        let mut back=Entity::new("back",Geometry::Surface(plane),None);back.placement=placement;let back_id=back.id;
        let mut doc=Document::with_template_tolerance("curved fallback snap");doc.entities.insert(front.id,front);doc.entities.insert(back.id,back);
        let path=std::env::temp_dir().join(format!("om9-curved-snap-{}.om9",uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&path,&doc,&limits,&Cancellation::default()).unwrap();
        let mut app=MatrixApp::from_file(&path).unwrap();std::fs::remove_file(path).unwrap();
        assert_eq!(app.controller.scene().unwrap().objects.iter().find(|o|o.root==front_id).unwrap().buffers.is_trim_display_fallback(),count==1025);
        app.controller.submit("Point").unwrap();app.controller.submit("100,100,100").unwrap();app.controller.undo().unwrap();let before=app.controller.document().clone();
        let context=egui::Context::default();for _ in 0..3 {frame(&mut app,&context,vec![]);}
        app.controller.workspace.active=ViewId::LookingDown;app.controller.workspace.views.get_mut(&ViewId::LookingDown).unwrap().mode=mode;
        let target=placement.point([0.,0.,z]).unwrap();
        app.controller.workspace.views.get_mut(&ViewId::LookingDown).unwrap().camera=
            om9_scene::camera::Camera::orthographic(placement.point([5.,1.,z]).unwrap(),target,[0.,0.,1.],10.).unwrap();
        app.controller.submit("Line").unwrap();app.set_snap_override(Some(SnapKind::Surface));
        let point=screen_of(&app,target);click(&mut app,&context,point);
        if !expected {assert_ne!(app.snap_candidate().and_then(|c|c.root),Some(back_id));}
        else {assert_eq!(app.snap_candidate().and_then(|c|c.root),Some(back_id),"mode={mode:?}, hole={hole}: {}",app.status_line());}
        assert_eq!(app.controller.document(),&before);assert_eq!(app.controller.coordinator().redo_len(),1);
        if expected {
            let snapped=app.snap_candidate().unwrap().world;
            assert!(snapped.iter().zip(target).all(|(a,b)|(a-b).abs()<1e-4));
            app.controller.submit("20,20,20").unwrap();
            let line=app.controller.document().entities.values().find_map(|e|match &e.geometry {
                Geometry::Curve(Curve::Line {start,..})=>Some(*start),_=>None,
            }).expect("line created from snap");
            assert!(line.iter().zip(snapped).all(|(a,b)|(a-b).abs()<1e-7));
            app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
            app.controller.redo().unwrap();
            assert!(app.controller.document().entities.values().any(|e|matches!(&e.geometry,Geometry::Curve(Curve::Line {..}))));
        }
    }
}

#[cfg(feature = "occt-brep")]
#[test]
fn unsupported_native_surface_snap_does_not_fall_back_to_construction_plane() {
    let context = egui::Context::default();
    let limits = om9_core::budget::Limits::default();
    let sphere = om9_kernel::brep::sphere_brep(5., 0.5, &limits).unwrap();
    let native = om9_kernel::brep::to_3dm(&sphere, 0.001, &limits).unwrap();
    let mut document = Document::with_template_tolerance("unsupported surface snap");
    let entity = Entity::new("native sphere", Geometry::NativeBrep(native), None);
    document.entities.insert(entity.id, entity);
    let mut app = MatrixApp::new(document).unwrap();
    frame(&mut app, &context, vec![]);
    frame(&mut app, &context, vec![]);
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.submit("Line").unwrap();
    app.set_snap_override(Some(SnapKind::Surface));
    let point = screen_of(&app, [2.13, 1.37, (25.0_f64 - 2.13_f64.powi(2) - 1.37_f64.powi(2)).sqrt()]);
    click(&mut app, &context, point);
    assert!(app.snap_candidate().is_none());
    assert!(app.status_line().contains("Surface snap unavailable"), "{}", app.status_line());
    assert!(app.controller.is_running());
    assert_eq!(app.controller.document().entities.len(), 1, "unsupported snap must not commit a point or geometry");
}

#[test]
fn explicit_surface_snap_miss_does_not_accept_construction_plane_point() {
    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::with_template_tolerance("missing surface")).unwrap();
    for _ in 0..3 { frame(&mut app, &context, vec![]); }
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.submit("Line").unwrap();
    app.set_snap_override(Some(SnapKind::Surface));
    let point = screen_of(&app, [2., 3., 0.]);
    click(&mut app, &context, point);
    assert!(app.status_line().contains("no surface candidate"), "{}", app.status_line());
    assert!(app.controller.is_running());
    assert!(app.controller.document().entities.is_empty());
    app.controller.submit("10,10,10").unwrap();
    assert!(app.controller.document().entities.is_empty(), "first typed point must not complete a line");
}

#[cfg(feature = "occt-brep")]
#[test]
fn snappy_gumball_moves_to_the_exact_brep_surface() {
    let context = egui::Context::default();
    let limits = om9_core::budget::Limits::default();
    let sphere = om9_kernel::brep::sphere_brep(5., 0.5, &limits).unwrap();
    let mut document = Document::with_template_tolerance("snappy surface precision");
    let line = Entity::new("moving line", Geometry::Curve(Curve::Line {
        start: [-1., 0., 0.], end: [1., 0., 0.], domain: [0., 2.],
    }), None);
    let line_id = line.id;
    document.entities.insert(line_id, line);
    let mut target = Entity::new("target sphere", Geometry::Brep(sphere), None);
    target.placement = om9_core::geometry::Transform::translation([10., 10., 0.]);
    document.entities.insert(target.id, target);
    let mut app = MatrixApp::new(document).unwrap();
    for _ in 0..4 { frame(&mut app, &context, vec![]); }
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.select_objects(&[line_id]).unwrap();
    app.controller.submit("Gumball Snappy").unwrap();
    app.set_snap_override(Some(SnapKind::Surface));
    frame(&mut app, &context, vec![]);
    let start = screen_of(&app, [0., 0., 0.]);
    let end = screen_of(&app, [12.13, 11.37, (25.0_f64 - 2.13_f64.powi(2) - 1.37_f64.powi(2)).sqrt()]);
    frame(&mut app, &context, vec![Event::PointerMoved(start), Event::PointerButton { pos: start, button: egui::PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE }]);
    frame(&mut app, &context, vec![Event::PointerMoved(end)]);
    frame(&mut app, &context, vec![Event::PointerButton { pos: end, button: egui::PointerButton::Primary, pressed: false, modifiers: Modifiers::NONE }]);
    let placement = app.controller.document().entities[&line_id].placement;
    let center = [placement.0[0][3] - 10., placement.0[1][3] - 10., placement.0[2][3]];
    let radius = center.iter().map(|value| value * value).sum::<f64>().sqrt();
    assert!((radius - 5.).abs() < 1e-6, "Gumball snap must reach the source surface: {radius}");
}

#[cfg(feature = "occt-brep")]
#[test]
fn snappy_gumball_moves_to_the_nurbs_source_surface() {
    let context = egui::Context::default();
    let limits = om9_core::budget::Limits::default();
    let surface = om9_core::surface_kernel::plane([5., 5., 4.], [1., 0., 0.], [0., 1., 0.], 10., 10., &limits).unwrap();
    let mut document = Document::with_template_tolerance("snappy NURBS surface");
    let line = Entity::new("moving line", Geometry::Curve(Curve::Line {
        start: [-1., 0., 0.], end: [1., 0., 0.], domain: [0., 2.],
    }), None);
    let line_id = line.id;
    document.entities.insert(line_id, line);
    let target = Entity::new("target surface", Geometry::Surface(surface), None);
    document.entities.insert(target.id, target);
    let mut app = MatrixApp::new(document).unwrap();
    for _ in 0..4 { frame(&mut app, &context, vec![]); }
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.select_objects(&[line_id]).unwrap();
    app.controller.submit("Gumball Snappy").unwrap();
    app.set_snap_override(Some(SnapKind::Surface));
    frame(&mut app, &context, vec![]);
    let start = screen_of(&app, [0., 0., 0.]);
    let end = screen_of(&app, [10., 10., 4.]);
    frame(&mut app, &context, vec![Event::PointerMoved(start), Event::PointerButton { pos: start, button: egui::PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE }]);
    frame(&mut app, &context, vec![Event::PointerMoved(end)]);
    frame(&mut app, &context, vec![Event::PointerButton { pos: end, button: egui::PointerButton::Primary, pressed: false, modifiers: Modifiers::NONE }]);
    let placement = app.controller.document().entities[&line_id].placement;
    assert!((placement.0[2][3] - 4.).abs() < 1e-7, "{placement:?}");
    assert_eq!(app.controller.coordinator().undo_len(), 1);
}

#[cfg(feature = "occt-brep")]
#[test]
fn snappy_gumball_surface_override_refuses_release_without_surface() {
    let context = egui::Context::default();
    let mut document = Document::with_template_tolerance("missing surface snap");
    let line = Entity::new("moving line", Geometry::Curve(Curve::Line {
        start: [-1., 0., 0.], end: [1., 0., 0.], domain: [0., 2.],
    }), None);
    let line_id = line.id;
    document.entities.insert(line_id, line);
    let mut app = MatrixApp::new(document).unwrap();
    for _ in 0..4 { frame(&mut app, &context, vec![]); }
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.select_objects(&[line_id]).unwrap();
    app.controller.submit("Gumball Snappy").unwrap();
    app.set_snap_override(Some(SnapKind::Surface));
    frame(&mut app, &context, vec![]);
    let start = screen_of(&app, [0., 0., 0.]);
    let end = screen_of(&app, [8., 5., 0.]);
    frame(&mut app, &context, vec![Event::PointerMoved(start), Event::PointerButton { pos: start, button: egui::PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE }]);
    frame(&mut app, &context, vec![Event::PointerMoved(end)]);
    frame(&mut app, &context, vec![Event::PointerButton { pos: end, button: egui::PointerButton::Primary, pressed: false, modifiers: Modifiers::NONE }]);
    assert_eq!(app.controller.document().entities[&line_id].placement, om9_core::geometry::Transform::identity());
    assert_eq!(app.controller.coordinator().undo_len(), 0);
    assert!(app.status_line().contains("surface snap candidate"), "{}", app.status_line());
}

#[test]
fn an_end_marker_is_consumed_by_the_point_prompt_and_smarttrack_keeps_two_anchors() {
    let context = egui::Context::default();
    let mut document = Document::new("osnap ui");
    let line = Entity::new(
        "line",
        Geometry::Curve(Curve::Line {
            start: [0., 0., 0.],
            end: [10., 0., 0.],
            domain: [0., 10.],
        }),
        None,
    );
    let line_id = line.id;
    document.entities.insert(line_id, line);
    let mut app = MatrixApp::new(document).unwrap();
    frame(&mut app, &context, vec![]);
    frame(&mut app, &context, vec![]);
    app.controller.workspace.active = ViewId::LookingDown;

    app.controller.submit("Line").unwrap();
    let first = screen_of(&app, [0.2, 0., 0.]);
    click(&mut app, &context, first);
    assert_eq!(
        app.snap_candidate().map(|candidate| candidate.kind),
        Some(SnapKind::End),
        "the pointer is close enough to the line end to show END"
    );
    assert_eq!(
        app.snap_candidate().map(|candidate| candidate.tooltip()),
        Some("End point".to_owned())
    );
    assert!(
        app.smarttrack_guides().len() == 2,
        "two CPlane guides per anchor"
    );
    let empty = screen_of(&app, [5., 8., 0.]);
    frame(&mut app, &context, vec![Event::PointerMoved(empty)]);
    assert!(
        app.snap_candidate().is_none(),
        "no candidate, no stale marker"
    );
    let second = screen_of(&app, [9.8, 0., 0.]);
    click(&mut app, &context, second);
    let Geometry::Curve(Curve::Line { start, end, .. }) = &app
        .controller
        .document()
        .entities
        .values()
        .next()
        .unwrap()
        .geometry
    else {
        panic!("line entity");
    };
    assert_eq!(*start, [0., 0., 0.]);
    assert_eq!(*end, [10., 0., 0.]);
    assert_eq!(app.controller.document().entities.len(), 2);
}

#[test]
fn escape_clears_the_smarttrack_guides_and_pending_snap() {
    let context = egui::Context::default();
    let mut document = Document::new("osnap escape");
    let line = Entity::new(
        "line",
        Geometry::Curve(Curve::Line {
            start: [0., 0., 0.],
            end: [10., 0., 0.],
            domain: [0., 10.],
        }),
        None,
    );
    let id = line.id;
    document.entities.insert(id, line);
    let mut app = MatrixApp::new(document).unwrap();
    frame(&mut app, &context, vec![]);
    frame(&mut app, &context, vec![]);
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.submit("Line").unwrap();
    let first = screen_of(&app, [0.2, 0., 0.]);
    click(&mut app, &context, first);
    assert_eq!(app.smarttrack_guides().len(), 2);
    frame(&mut app, &context, vec![key(egui::Key::Escape)]);
    assert!(app.smarttrack_guides().is_empty());
    assert!(app.snap_candidate().is_none());
    assert!(!app.controller.is_running());
}

#[test]
fn disabling_end_removes_the_marker_from_the_next_frame() {
    let context = egui::Context::default();
    let mut document = Document::new("osnap disabled");
    let line = Entity::new(
        "line",
        Geometry::Curve(Curve::Line {
            start: [0., 0., 0.],
            end: [10., 0., 0.],
            domain: [0., 10.],
        }),
        None,
    );
    let id = line.id;
    document.entities.insert(id, line);
    let mut app = MatrixApp::new(document).unwrap();
    frame(&mut app, &context, vec![]);
    frame(&mut app, &context, vec![]);
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.submit("Point").unwrap();
    let at = screen_of(&app, [9.8, 0., 0.]);
    frame(&mut app, &context, vec![Event::PointerMoved(at)]);
    assert_eq!(app.snap_candidate().map(|c| c.kind), Some(SnapKind::End));
    app.set_snap_enabled(SnapKind::End, false);
    let empty = screen_of(&app, [5., 8., 0.]);
    frame(&mut app, &context, vec![Event::PointerMoved(empty)]);
    assert!(app.snap_candidate().is_none(), "disabled END has no marker");
    app.set_snap_enabled(SnapKind::End, true);
    frame(&mut app, &context, vec![Event::PointerMoved(at)]);
    assert_eq!(app.snap_candidate().map(|c| c.kind), Some(SnapKind::End));
}
