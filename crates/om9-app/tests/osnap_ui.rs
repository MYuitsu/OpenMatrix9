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

// Keep the legacy fixture's exact source/display pairs when loading the app.
#[cfg(feature = "occt-brep")]
#[derive(Default)]
struct BrepFixtures(Vec<(String, serde_json::Value)>);

#[cfg(feature = "occt-brep")]
impl BrepFixtures {
    fn brep(&mut self, name: &str, value: om9_core::brep::Brep) -> Entity {
        let legacy = serde_json::json!({"Brep": &value});
        let (source, _) = value.into_canonical_and_representation();
        let entity = Entity::new(name, Geometry::Brep(source), None);
        self.0.push((entity.id.to_string(), legacy));
        entity
    }

    fn native(&mut self, name: &str, value: om9_core::geometry::NativeBrep) -> Entity {
        let legacy = serde_json::json!({"NativeBrep": &value});
        let (source, _) = value.into_source_and_representation();
        let entity = Entity::new(name, Geometry::NativeBrep(source), None);
        self.0.push((entity.id.to_string(), legacy));
        entity
    }

    fn open(&mut self, document: Document) -> MatrixApp {
        self.try_open(document).unwrap()
    }

    fn try_open(&mut self, document: Document) -> om9_core::Result<MatrixApp> {
        let path = self.write(document);
        let app = MatrixApp::from_file(&path);
        std::fs::remove_file(path).unwrap();
        app
    }

    fn write(&mut self, document: Document) -> std::path::PathBuf {
        use sha2::{Digest, Sha256};
        let template = include_bytes!("../../../implementation/evidence/M03/solid-ui-followup-2026-10-23/review-fixtures/Boolean2-D-after.om9");
        let start = u64::from_le_bytes(template[40..48].try_into().unwrap()) as usize;
        let mut metadata = serde_json::to_value(document).unwrap();
        for (id, entity) in metadata["entities"].as_object_mut().unwrap() {
            if let Some((_, legacy)) = self.0.iter().find(|(source_id, _)| source_id == id) {
                entity["geometry"] = legacy.clone();
            }
        }
        self.0.clear();
        let metadata = serde_json::to_vec(&metadata).unwrap();
        let mut fixture = template[..start].to_vec();
        fixture[48..56].copy_from_slice(&(metadata.len() as u64).to_le_bytes());
        fixture[56..88].copy_from_slice(&Sha256::digest(&metadata));
        fixture.extend_from_slice(&metadata);
        let path = std::env::temp_dir().join(format!("om9-osnap-fixture-{}.om9", uuid::Uuid::new_v4()));
        std::fs::write(&path, fixture).unwrap();
        path
    }
}

fn screen_rect() -> Rect {
    Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 900.))
}

fn frame(app: &mut MatrixApp, context: &egui::Context, events: Vec<Event>) {
    frame_at_time(app, context, events, None);
}

fn frame_at_time(
    app: &mut MatrixApp,
    context: &egui::Context,
    events: Vec<Event>,
    time: Option<f64>,
) -> egui::FullOutput {
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(screen_rect()),
            events,
            time,
            ..Default::default()
        },
        |ui| app.show(ui),
    );
    output.textures_delta.clear();
    output
}

fn dwell_smarttrack(app: &mut MatrixApp, context: &egui::Context) {
    let time = context.input(|input| input.time + 1.);
    frame_at_time(app, context, vec![], Some(time));
}

fn wait_snap(app: &mut MatrixApp, context: &egui::Context, kind: SnapKind) -> om9_scene::osnap::SnapCandidate {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
    loop {
        if let Some(candidate) = app.snap_candidate().filter(|candidate| candidate.kind == kind) {
            return candidate.clone();
        }
        assert!(std::time::Instant::now() < deadline, "waiting for {kind:?}: {}", app.status_line());
        std::thread::sleep(std::time::Duration::from_millis(5));
        frame(app, context, vec![]);
    }
}

fn wait_status(app: &mut MatrixApp, context: &egui::Context, expected: &str) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
    while !app.status_line().contains(expected) {
        assert!(std::time::Instant::now() < deadline, "waiting for {expected}: {}", app.status_line());
        std::thread::sleep(std::time::Duration::from_millis(5));
        frame(app, context, vec![]);
    }
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

#[test]
fn default_apparent_int_keeps_distinct_nested_occurrences_through_line_commit() {
    use om9_core::{budget::Limits, curve_ops::point_at, geometry::Transform, model::Definition};
    use om9_scene::{camera::Camera, workspace::DisplayMode};

    let mut document = Document::with_template_tolerance("shared block Line intersection");
    let curve = Curve::Line {
        start: [-4., 0., 0.],
        end: [4., 0., 0.],
        domain: [0., 1.],
    };
    let member = Entity::new("shared source", Geometry::Curve(curve.clone()), None);
    let leaf = member.id;
    let definition = Definition::new("shared line", vec![member]);
    let horizontal = Entity::new("horizontal", Geometry::Instance(definition.id), None);
    let mut vertical = Entity::new("vertical", Geometry::Instance(definition.id), None);
    vertical.placement = Transform([
        [0., -1., 0., 0.],
        [1., 0., 0., 0.],
        [0., 0., 1., 6.],
        [0., 0., 0., 1.],
    ]);
    let far_path = vec![horizontal.id, leaf];
    let near_path = vec![vertical.id, leaf];
    let block = Definition::new("crossing occurrences", vec![horizontal, vertical]);
    let mut instance = Entity::new("scaled block", Geometry::Instance(block.id), None);
    instance.placement = Transform::translation([6., 2., 2.])
        .compose(Transform::scale([2., 3., 1.]).unwrap())
        .unwrap();
    let root = instance.id;
    document.definitions.insert(definition.id, definition);
    document.definitions.insert(block.id, block);
    document.entities.insert(root, instance);

    let mut app = MatrixApp::new(document).unwrap();
    let context = egui::Context::default();
    app.controller.workspace.active = ViewId::LookingDown;
    frame(&mut app, &context, vec![]);
    frame(&mut app, &context, vec![]);
    assert!(!app.snap_enabled(SnapKind::Intersection));
    app.set_snap_enabled(SnapKind::End, false);
    app.set_snap_enabled(SnapKind::Mid, false);
    app.set_snap_enabled(SnapKind::Intersection, true);

    let before = app.controller.document().clone();
    let expected_source = [6., 2., 8.];
    for mode in [DisplayMode::Wireframe, DisplayMode::Shaded] {
        for project in [false, true] {
            let view = app.controller.workspace.views.get_mut(&ViewId::LookingDown).unwrap();
            view.mode = mode;
            view.camera = Camera::orthographic([6., 2., 30.], [6., 2., 0.], [0., 1., 0.], 60.).unwrap();
            app.controller.set_project(project);
            app.controller.submit("Line").unwrap();
            let cursor = screen_of(&app, expected_source);
            frame(&mut app, &context, vec![Event::PointerMoved(cursor)]);
            let hit = wait_snap(&mut app, &context, SnapKind::Intersection);
            assert!(hit.apparent, "{hit:?}");
            assert_eq!(hit.marker_label(), "Apparent Int");
            assert_eq!(hit.world, expected_source);
            let scene = app.controller.scene().unwrap();
            for (source, path) in [
                (hit.source.as_ref().unwrap(), &near_path),
                (hit.second_source.as_ref().unwrap(), &far_path),
            ] {
                assert_eq!(source.root, root);
                assert_eq!(source.leaf, leaf);
                assert_eq!(source.path, *path);
                source.validate(scene).unwrap();
            }
            let occurrence = scene.objects.iter()
                .find(|object| object.root == root && object.path == near_path)
                .unwrap();
            let parameter = hit.parameter.unwrap();
            assert_eq!(parameter, 0.5);
            let local = point_at(&curve, parameter, &Limits::default()).unwrap();
            assert_eq!(
                occurrence.display_transform().unwrap().point(local).unwrap(),
                hit.world,
            );
            assert_eq!(app.controller.document(), &before);

            click(&mut app, &context, cursor);
            assert!(app.snap_enabled(SnapKind::Intersection));
            app.controller.submit("15,4,0").unwrap();
            let expected_start = [6., 2., if project { 0. } else { 8. }];
            let after = app.controller.document();
            assert_eq!(after.entities.len(), before.entities.len() + 1);
            assert!(after.entities.values().any(|entity| {
                matches!(&entity.geometry, Geometry::Curve(Curve::Line { start, end, .. })
                    if *start == expected_start && *end == [15., 4., 0.])
            }));
            assert_eq!(app.controller.coordinator().undo_len(), 1);
            app.controller.undo().unwrap();
            assert_eq!(app.controller.document(), &before);
        }
    }
}

#[cfg(feature = "occt-brep")]
#[test]
fn surface_snap_ui_commits_an_exact_sphere_point_instead_of_the_display_triangle() {
    let mut fixtures = BrepFixtures::default();
    let context = egui::Context::default();
    let limits = om9_core::budget::Limits::default();
    let sphere = om9_kernel::brep::sphere_brep(5., 0.5, &limits).unwrap();
    let mut document = Document::with_template_tolerance("surface snap precision");
    let entity = fixtures.brep("sphere", sphere);
    document.entities.insert(entity.id, entity);
    let mut app = fixtures.open(document);
    frame(&mut app, &context, vec![]);
    frame(&mut app, &context, vec![]);
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.submit("Line").unwrap();
    app.set_snap_override(Some(SnapKind::Surface));
    let point = screen_of(&app, [2.13, 1.37, (25.0_f64 - 2.13_f64.powi(2) - 1.37_f64.powi(2)).sqrt()]);
    frame(&mut app, &context, vec![Event::PointerMoved(point)]);
    let snap = wait_snap(&mut app, &context, SnapKind::Surface);
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
    let mut fixtures = BrepFixtures::default();
    for gumball in [false, true] {
        let context = egui::Context::default();
        let limits = om9_core::budget::Limits::default();
        let mut box_entity = fixtures.brep("box", om9_kernel::brep::box_brep([10., 10., 10.], 0.1, &limits).unwrap());
        box_entity.placement = om9_core::geometry::Transform::translation([10., 10., 0.]);
        let box_id = box_entity.id;
        let mut document = Document::with_template_tolerance("grouped surface UI");
        document.entities.insert(box_id, box_entity);
        let line = Entity::new("moving line", Geometry::Curve(Curve::Line {
            start: [-1., 0., 0.], end: [1., 0., 0.], domain: [0., 2.],
        }), None);
        let line_id = line.id;
        if gumball { document.entities.insert(line_id, line); }
        let mut app = fixtures.open(document);
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
    let mut fixtures = BrepFixtures::default();
    let context = egui::Context::default();
    let limits = om9_core::budget::Limits::default();
    let coarse = om9_kernel::brep::sphere_brep(3., 0.5, &limits).unwrap();
    let fine = om9_kernel::brep::sphere_brep(3., 0.001, &limits).unwrap();
    let mut front = fixtures.brep("front coarse", coarse);
    front.placement = om9_core::geometry::Transform::translation([0., 0., 0.01]);
    let front_id = front.id;
    let back = fixtures.brep("back fine", fine);
    let back_id = back.id;
    let mut document = Document::with_template_tolerance("close overlapping surfaces");
    document.entities.insert(front.id, front);
    document.entities.insert(back.id, back);
    let mut app = fixtures.open(document);
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
    let snap = wait_snap(&mut app, &context, SnapKind::Surface);
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
    let mut fixtures = BrepFixtures::default();
    let context = egui::Context::default();
    let limits = om9_core::budget::Limits::default();
    let mut front = fixtures.brep("front coarse", om9_kernel::brep::sphere_brep(3., 0.5, &limits).unwrap());
    front.placement = om9_core::geometry::Transform::translation([10., 10., 0.01]);
    let mut back = fixtures.brep("back fine", om9_kernel::brep::sphere_brep(3., 0.001, &limits).unwrap());
    back.placement = om9_core::geometry::Transform::translation([10., 10., 0.]);
    let line = Entity::new("moving line", Geometry::Curve(Curve::Line {
        start: [-1., 0., 0.], end: [1., 0., 0.], domain: [0., 2.],
    }), None);
    let line_id = line.id;
    let mut document = Document::with_template_tolerance("gumball close surfaces");
    for entity in [front, back, line] { document.entities.insert(entity.id, entity); }
    let mut app = fixtures.open(document);
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
    let mut fixtures = BrepFixtures::default();
    let context = egui::Context::default();
    let limits = om9_core::budget::Limits::default();
    let sphere = om9_kernel::brep::sphere_brep(3., 0.1, &limits).unwrap();
    let first = fixtures.brep("first", sphere.clone());
    let second = fixtures.brep("second", sphere);
    let (mut back, mut front) = if first.id < second.id { (first, second) } else { (second, first) };
    back.placement = om9_core::geometry::Transform::translation([0., 0., -5.]);
    front.placement = om9_core::geometry::Transform::translation([0., 0., 5.]);
    let front_id = front.id;
    let mut document = Document::with_template_tolerance("overlap surface snap");
    document.entities.insert(back.id, back);
    document.entities.insert(front.id, front);
    let mut app = fixtures.open(document);
    frame(&mut app, &context, vec![]);
    frame(&mut app, &context, vec![]);
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.submit("Line").unwrap();
    app.set_snap_override(Some(SnapKind::Surface));
    let point = screen_of(&app, [1., 0.5, 5. + (9_f64 - 1. - 0.25).sqrt()]);
    frame(&mut app, &context, vec![Event::PointerMoved(point)]);
    let snap = wait_snap(&mut app, &context, SnapKind::Surface);
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
    let mut fixtures = BrepFixtures::default();
    for is_hatch in [false, true] {
    let context = egui::Context::default();
    let limits = om9_core::budget::Limits::default();
    let mut sphere = fixtures.brep("sphere", om9_kernel::brep::sphere_brep(3., 0.1, &limits).unwrap());
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
    let mut app = fixtures.open(document);
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
    let mut fixtures = BrepFixtures::default();
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
    let sphere = fixtures.brep("sphere", om9_kernel::brep::sphere_brep(3., 0.1, &limits).unwrap());
    let mut mesh = Entity::new("occluder", Geometry::Mesh(om9_core::geometry::Mesh {
        vertices: vec![[-5., -5., z], [5., -5., z], [5., 5., z], [-5., 5., z]],
        faces: vec![vec![0, 1, 2, 3]], normals: vec![], colors: vec![], corner_uvs: vec![],
    }), None);
    mesh.attributes.hidden = hidden;
    mesh.attributes.selectable = selectable;
    let mut document = Document::with_template_tolerance("shaded mesh occluder");
    document.entities.insert(sphere.id, sphere);
    document.entities.insert(mesh.id, mesh);
    let mut app = fixtures.open(document);
    for _ in 0..3 { frame(&mut app, &context, vec![]); }
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.workspace.views.get_mut(&ViewId::LookingDown).unwrap().mode = mode;
    app.controller.submit("Line").unwrap();
    app.set_snap_override(Some(SnapKind::Surface));
    let point = screen_of(&app, [1., 0.5, (9_f64 - 1. - 0.25).sqrt()]);
    click(&mut app, &context, point);
    if blocked {
        wait_status(&mut app, &context, "Surface snap unavailable");
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
fn shaded_mesh_hides_the_smarttrack_curve_reference() {
    use om9_scene::workspace::DisplayMode;

    for mode in [DisplayMode::Wireframe, DisplayMode::Shaded] {
        let context = egui::Context::default();
        let mut document = Document::with_template_tolerance("SmartTrack visibility");
        let anchor = Entity::new(
            "anchor",
            Geometry::Curve(Curve::Line {
                start: [-12., 0., 0.],
                end: [-16., 3., 0.],
                domain: [0., 5.],
            }),
            None,
        );
        let source = Entity::new(
            "source",
            Geometry::Curve(Curve::Line {
                start: [3., -5., 0.],
                end: [3., 5., 0.],
                domain: [0., 10.],
            }),
            None,
        );
        let source_id = source.id;
        let mesh = Entity::new(
            "occluder",
            Geometry::Mesh(om9_core::geometry::Mesh {
                vertices: vec![[0., -3., 10.], [6., -3., 10.], [6., 3., 10.], [0., 3., 10.]],
                faces: vec![vec![0, 1, 2, 3]],
                normals: vec![],
                colors: vec![],
                corner_uvs: vec![],
            }),
            None,
        );
        for entity in [anchor, source, mesh] {
            document.entities.insert(entity.id, entity);
        }
        let mut app = MatrixApp::new(document).unwrap();
        for _ in 0..3 {
            frame(&mut app, &context, vec![]);
        }
        app.controller.workspace.active = ViewId::LookingDown;
        app.controller.workspace.views.get_mut(&ViewId::LookingDown).unwrap().mode = mode;
        app.set_snap_enabled(SnapKind::Mid, false);
        app.controller.submit("Line").unwrap();

        let start = screen_of(&app, [-12., 0., 0.]);
        frame(&mut app, &context, vec![Event::PointerMoved(start)]);
        assert_eq!(
            wait_snap(&mut app, &context, SnapKind::End).world,
            [-12., 0., 0.],
        );
        click(&mut app, &context, start);
        assert_eq!(app.smarttrack_guides().len(), 4);

        let target = screen_of(&app, [3.2, 0., 0.]);
        frame(&mut app, &context, vec![Event::PointerMoved(target)]);
        let candidate = wait_snap(&mut app, &context, SnapKind::SmartTrack);
        let expected_source = (mode == DisplayMode::Wireframe).then_some(source_id);
        assert_eq!(candidate.root, expected_source, "{mode:?}");
        assert_eq!(candidate.source.as_ref().map(|source| source.root), expected_source, "{mode:?}");
    }
}

#[cfg(feature = "occt-brep")]
#[test]
fn shaded_mesh_hides_line_intersection_from_native_viewport() {
    use om9_scene::workspace::DisplayMode;

    for (mode, apparent, cover_z, visible) in [
        (DisplayMode::Wireframe, false, 5., true),
        (DisplayMode::Shaded, false, 5., false),
        (DisplayMode::Wireframe, true, 10., true),
        (DisplayMode::Shaded, true, 5., true),
        (DisplayMode::Shaded, true, 10., false),
    ] {
        let context = egui::Context::default();
        let mut document = Document::with_template_tolerance("intersection visibility");
        let witness = [0., 0., if apparent { 8. } else { 0. }];
        for (start, end) in [
            ([-5., 0., 0.], [5., 0., 0.]),
            ([0., -5., witness[2]], [0., 5., witness[2]]),
        ] {
            let line = Entity::new("line", Geometry::Curve(Curve::Line { start, end, domain: [0., 1.] }), None);
            document.entities.insert(line.id, line);
        }
        let mesh = Entity::new("occluder", Geometry::Mesh(om9_core::geometry::Mesh {
            vertices: vec![[-3., -3., cover_z], [3., -3., cover_z], [3., 3., cover_z], [-3., 3., cover_z]],
            faces: vec![vec![0, 1, 2, 3]],
            normals: vec![], colors: vec![], corner_uvs: vec![],
        }), None);
        document.entities.insert(mesh.id, mesh);
        let mut app = MatrixApp::new(document).unwrap();
        let before = app.controller.document().clone();
        for _ in 0..3 {
            frame(&mut app, &context, vec![]);
        }
        app.controller.workspace.active = ViewId::LookingDown;
        app.controller.workspace.views.get_mut(&ViewId::LookingDown).unwrap().mode = mode;
        if apparent {
            app.focus_command_editor();
            frame(&mut app, &context, vec![]);
            frame(&mut app, &context, vec![Event::Text("ApparentIntersections On".into())]);
            frame(&mut app, &context, vec![key(egui::Key::Enter)]);
            assert!(app.status_line().contains("ApparentIntersections on"));
            app.controller.set_project(true);
        }
        app.controller.submit("Line").unwrap();
        app.set_snap_override(Some(SnapKind::Intersection));
        let cursor = screen_of(&app, witness);
        if visible {
            frame(&mut app, &context, vec![Event::PointerMoved(cursor)]);
            let snap = wait_snap(&mut app, &context, SnapKind::Intersection);
            assert_eq!(snap.world, witness, "{mode:?} cover at Z={cover_z}");
            assert_eq!(snap.apparent, apparent);
            assert_ne!(snap.source.as_ref().unwrap().root, snap.second_source.as_ref().unwrap().root);
        }
        click(&mut app, &context, cursor);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while app.status_line().contains("Waiting for edge snaps") {
            assert!(std::time::Instant::now() < deadline, "{}", app.status_line());
            std::thread::sleep(std::time::Duration::from_millis(5));
            frame(&mut app, &context, vec![]);
        }
        if !visible {
            assert!(app.controller.prompt().starts_with("Start of line"));
            assert_eq!(app.snap_override(), Some(SnapKind::Intersection));
            assert!(app.snap_candidate().is_none());
        } else {
            assert_eq!(app.controller.point_constraint_base().unwrap().xyz(), [0.; 3]);
            assert_eq!(app.snap_override(), None);
        }
        assert_eq!(app.controller.document(), &before);
        assert_eq!(app.controller.coordinator().undo_len(), 0);
        if visible {
            app.controller.submit("12,1,0").unwrap();
            let after = app.controller.document();
            assert_eq!(after.entities.len(), before.entities.len() + 1);
            assert!(after.entities.values().any(|entity| {
                matches!(&entity.geometry, Geometry::Curve(Curve::Line { start, end, .. })
                    if *start == [0.; 3] && *end == [12., 1., 0.])
            }));
            assert_eq!(app.controller.coordinator().undo_len(), 1);
            app.controller.undo().unwrap();
            assert_eq!(app.controller.document(), &before);
        }
    }
}

#[cfg(feature = "occt-brep")]
#[test]
fn unselectable_brep_remains_a_surface_reference_when_visible() {
    let mut fixtures = BrepFixtures::default();
    use om9_scene::workspace::DisplayMode;
    for (z, mode, hidden, front_reference) in [
        (10., DisplayMode::Shaded, false, true),
        (-10., DisplayMode::Shaded, false, false),
        (10., DisplayMode::Wireframe, false, true),
        (10., DisplayMode::Shaded, true, false),
    ] {
        let context = egui::Context::default();
        let limits = om9_core::budget::Limits::default();
        let sphere = fixtures.brep("target", om9_kernel::brep::sphere_brep(3., 0.1, &limits).unwrap());
        let mut occluder = fixtures.brep("unselectable", om9_kernel::brep::sphere_brep(3., 0.1, &limits).unwrap());
        occluder.placement = om9_core::geometry::Transform::translation([0., 0., z]);
        occluder.attributes.selectable = false;
        occluder.attributes.hidden = hidden;
        let mut document = Document::with_template_tolerance("unselectable brep occluder");
        document.entities.insert(sphere.id, sphere);
        document.entities.insert(occluder.id, occluder);
        let mut app = fixtures.open(document);
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
    let mut fixtures = BrepFixtures::default();
    use om9_scene::workspace::DisplayMode;
    for (z, mode, hidden, blocked) in [
        (10., DisplayMode::Shaded, false, true),
        (-10., DisplayMode::Shaded, false, false),
        (10., DisplayMode::Wireframe, false, false),
        (10., DisplayMode::Shaded, true, false),
    ] {
        let context = egui::Context::default();
        let limits = om9_core::budget::Limits::default();
        let sphere = fixtures.brep("target", om9_kernel::brep::sphere_brep(3., 0.1, &limits).unwrap());
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
        let mut app = fixtures.open(document);
        for _ in 0..3 { frame(&mut app, &context, vec![]); }
        app.controller.workspace.active = ViewId::LookingDown;
        app.controller.workspace.views.get_mut(&ViewId::LookingDown).unwrap().mode = mode;
        app.controller.submit("Line").unwrap();
        app.set_snap_override(Some(SnapKind::Surface));
        let point = screen_of(&app, [1., 0.5, (9_f64 - 1. - 0.25).sqrt()]);
        click(&mut app, &context, point);
        if blocked {
            wait_status(&mut app, &context, "occluded");
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
    let mut fixtures = BrepFixtures::default();
    use om9_scene::workspace::DisplayMode;
    for (z, mode, hidden, front) in [
        (10., DisplayMode::Shaded, false, true),
        (-10., DisplayMode::Shaded, false, false),
        (10., DisplayMode::Wireframe, false, true),
        (10., DisplayMode::Shaded, true, false),
    ] {
        let context = egui::Context::default();
        let limits = om9_core::budget::Limits::default();
        let sphere = fixtures.brep("sphere", om9_kernel::brep::sphere_brep(3., 0.1, &limits).unwrap());
        let surface = om9_core::surface_kernel::plane([-5., -5., z], [1., 0., 0.], [0., 1., 0.], 10., 10., &limits).unwrap();
        let mut surface = Entity::new("surface occluder", Geometry::Surface(surface), None);
        surface.attributes.hidden = hidden;
        let mut document = Document::with_template_tolerance("surface occluder");
        document.entities.insert(sphere.id, sphere);
        document.entities.insert(surface.id, surface);
        let mut app = fixtures.open(document);
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
fn reopened_native_and_occt_overlap_resolves_front_source() {
    for native_front in [false, true] {
    let mut fixtures = BrepFixtures::default();
    let context = egui::Context::default();
    let limits = om9_core::budget::Limits::default();
    let sphere = om9_kernel::brep::sphere_brep(3., 0.1, &limits).unwrap();
    let native = om9_kernel::brep::to_3dm(&sphere, 0.001, &limits).unwrap();
    let mut front = fixtures.brep("front OCCT", sphere);
    front.placement = om9_core::geometry::Transform::translation([0., 0., if native_front { -5. } else { 5. }]);
    let front_id = front.id;
    let mut back = fixtures.native("rear native", native);
    back.placement = om9_core::geometry::Transform::translation([0., 0., if native_front { 5. } else { -5. }]);
    let back_id = back.id;
    let mut document = Document::with_template_tolerance("mixed source overlap");
    document.entities.insert(front_id, front);
    document.entities.insert(back_id, back);
    let mut app = fixtures.open(document);
    let scene = app.controller.scene().unwrap();
    let native_object = scene.objects.iter().find(|object| object.root == back_id).unwrap();
    let native_source = scene.capture_native_brep_source_for(app.controller.coordinator(), native_object).unwrap();
    let (_, face_count) = om9_kernel::brep::native_source_and_face_count_admitted(
        native_source.payload(), 0.001, &limits, &mut |_, _| Ok(())).unwrap();
    assert!(face_count > 0, "the reopened native payload must have source faces");
    for _ in 0..3 { frame(&mut app, &context, vec![]); }
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.submit("Line").unwrap();
    app.set_snap_override(Some(SnapKind::Surface));
    let point = screen_of(&app, [1., 0.5, 5. + (9_f64 - 1. - 0.25).sqrt()]);
    let camera = &app.controller.workspace.views[&ViewId::LookingDown].camera;
    let candidates = om9_scene::osnap::candidates(app.controller.scene().unwrap(), camera, viewport(&app),
        [point.x as f64, point.y as f64], Default::default(), None).unwrap();
    assert!(candidates.iter().any(|c| c.kind == SnapKind::Surface && c.root == Some(front_id)));
    assert!(candidates.iter().any(|c| c.kind == SnapKind::Surface && c.root == Some(back_id)));
    frame(&mut app, &context, vec![Event::PointerMoved(point)]);
    let snap = wait_snap(&mut app, &context, SnapKind::Surface);
    assert_eq!(snap.root, Some(if native_front { back_id } else { front_id }));
    let radius = (snap.world[0].powi(2) + snap.world[1].powi(2) + (snap.world[2] - 5.).powi(2)).sqrt();
    assert!((radius - 3.).abs() < 1e-6);
    click(&mut app, &context, point);
    app.controller.submit("20,20,20").unwrap();
    let line = app.controller.document().entities.values().find_map(|entity| match &entity.geometry {
        Geometry::Curve(Curve::Line { start, .. }) => Some(*start), _ => None,
    }).unwrap();
    assert!((0..3).all(|i| (line[i] - snap.world[i]).abs() < 1e-6));
    }
}

#[cfg(feature = "occt-brep")]
#[test]
fn reopening_unsupported_native_with_occt_refuses_the_document() {
    let mut fixtures = BrepFixtures::default();
    let limits = om9_core::budget::Limits::default();
    let sphere = om9_kernel::brep::sphere_brep(3., 0.1, &limits).unwrap();
    let mut native = om9_kernel::brep::to_3dm(&sphere, 0.001, &limits).unwrap();
    // The display is retained, but these opaque bytes cannot be decoded as BRep.
    native.payload = b"unsupported native BRep fixture".to_vec().into();
    native.crc = 1;
    let mut front = fixtures.brep("bound", sphere);
    front.placement = om9_core::geometry::Transform::translation([0., 0., 5.]);
    let mut back = fixtures.native("unbound", native);
    back.placement = om9_core::geometry::Transform::translation([0., 0., -5.]);
    let mut document = Document::with_template_tolerance("unbound overlapping surface");
    document.entities.insert(front.id, front);
    document.entities.insert(back.id, back);
    assert!(fixtures.try_open(document).is_err(), "unreadable native source must refuse the file");
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
    let snap=wait_snap(&mut app,&context,SnapKind::Surface).world;
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
    let context=egui::Context::default();let mut doc=Document::with_template_tolerance("parallel endpoint snap");
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
        app.controller.submit("Line").unwrap();app.set_snap_override(Some(SnapKind::Surface));let point=screen_of(&app,[x,0.,-1.]);
        if expected {frame(&mut app,&context,vec![Event::PointerMoved(point)]);wait_snap(&mut app,&context,SnapKind::Surface);}
        click(&mut app,&context,point);
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
        let point=screen_of(&app,target);
        if expected {frame(&mut app,&context,vec![Event::PointerMoved(point)]);wait_snap(&mut app,&context,SnapKind::Surface);}
        click(&mut app,&context,point);
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
fn reopened_native_sphere_surface_snap_uses_its_source() {
    let mut fixtures = BrepFixtures::default();
    let context = egui::Context::default();
    let limits = om9_core::budget::Limits::default();
    let sphere = om9_kernel::brep::sphere_brep(5., 0.5, &limits).unwrap();
    let native = om9_kernel::brep::to_3dm(&sphere, 0.001, &limits).unwrap();
    let entity = fixtures.native("native sphere", native);
    let id = entity.id;
    let mut document = Document::with_template_tolerance("native sphere snap");
    document.entities.insert(id, entity);
    let mut app = fixtures.open(document);
    let scene = app.controller.scene().unwrap();
    let object = scene.objects.iter().find(|object| object.root == id).unwrap();
    let native_source = scene.capture_native_brep_source_for(app.controller.coordinator(), object).unwrap();
    let (_, faces) = om9_kernel::brep::native_source_and_face_count_admitted(
        native_source.payload(), 0.001, &limits, &mut |_, _| Ok(())).unwrap();
    assert!(faces > 0);
    for _ in 0..3 { frame(&mut app, &context, vec![]); }
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.submit("Line").unwrap();
    app.set_snap_override(Some(SnapKind::Surface));
    let point = screen_of(&app, [2.13, 1.37, (25.0_f64 - 2.13_f64.powi(2) - 1.37_f64.powi(2)).sqrt()]);
    let camera = &app.controller.workspace.views[&ViewId::LookingDown].camera;
    let candidates = om9_scene::osnap::candidates(app.controller.scene().unwrap(), camera, viewport(&app),
        [point.x as f64, point.y as f64], Default::default(), None).unwrap();
    assert!(candidates.iter().any(|candidate| candidate.kind == SnapKind::Surface && candidate.root == Some(id)),
        "native display must produce a surface witness");
    frame(&mut app, &context, vec![Event::PointerMoved(point)]);
    let snap = wait_snap(&mut app, &context, SnapKind::Surface);
    assert_eq!(snap.root, Some(id));
    let radius = snap.world.iter().map(|v| v * v).sum::<f64>().sqrt();
    assert!((radius - 5.).abs() < 1e-6, "native source radius {radius}");
    click(&mut app, &context, point);
    app.controller.submit("20,20,20").unwrap();
    let line = app.controller.document().entities.values().find_map(|entity| match &entity.geometry {
        Geometry::Curve(Curve::Line { start, .. }) => Some(*start), _ => None,
    }).unwrap();
    assert!((0..3).all(|i| (line[i] - snap.world[i]).abs() < 1e-6));
}

#[cfg(all(windows, feature = "occt-brep"))]
#[test]
#[ignore = "requires the packaged native BRep worker and fingerprint"]
fn native_v7_surface_snap_survives_save_as_and_reopen() {
    use om9_app::file_lifecycle::{FileKind, FileOutcome};
    use om9_core::{budget::Limits, geometry::Transform};
    use om9_scene::workspace::DisplayMode;
    use std::{fs, io::Read, time::{Duration, Instant}};

    let limits = Limits::default();
    let seed = om9_kernel::brep::sphere_brep(5., 0.1, &limits).unwrap();
    let native = om9_kernel::brep::to_3dm(&seed, 0.001, &limits).unwrap();
    let mut sphere = Entity::new("native sphere", Geometry::NativeBrep(native.canonical_source()), None);
    let placement = Transform([
        [0., -2., 0., 4.],
        [0.5, 0., 0., -2.],
        [0., 0., 1.5, 1.],
        [0., 0., 0., 1.],
    ]);
    sphere.placement = placement;
    let id = sphere.id;
    let mut document = Document::with_template_tolerance("v7 native sphere snap");
    document.entities.insert(id, sphere);
    let owner = MatrixApp::new(Document::with_template_tolerance("archive fixture owner")).unwrap();
    let archive = om9_archive::canonical_brep::encode_mixed_document_candidate(
        &document,
        &[],
        &owner.controller.coordinator().external_memory_account(),
        &limits,
        |_, _| Ok(()),
    ).unwrap();
    assert_eq!(&archive[8..12], &7_u32.to_le_bytes());
    let nonce = uuid::Uuid::new_v4();
    let input = std::env::temp_dir().join(format!("om9-native-v7-snap-{nonce}.om9"));
    let output = std::env::temp_dir().join(format!("om9-native-v7-snap-{nonce}-saved.om9"));
    fs::write(&input, archive).unwrap();
    let mut app = MatrixApp::from_file(&input).unwrap();
    fs::remove_file(input).unwrap();
    assert!(app.controller.file_diagnostics().is_empty());
    assert_eq!(app.controller.document().entities, document.entities);
    let context = egui::Context::default();
    app.controller.workspace.active = ViewId::LookingDown;
    for _ in 0..3 { frame(&mut app, &context, vec![]); }
    let expected = placement.point([1., 1., 23_f64.sqrt()]).unwrap();
    let before = app.controller.document().clone();
    let end = [20., 20., 20.];
    for mode in [DisplayMode::Wireframe, DisplayMode::Shaded] {
        app.controller.workspace.views.get_mut(&ViewId::LookingDown).unwrap().mode = mode;
        app.controller.submit("Line").unwrap();
        app.set_snap_override(Some(SnapKind::Surface));
        let cursor = screen_of(&app, expected);
        frame(&mut app, &context, vec![Event::PointerMoved(cursor)]);
        let snap = wait_snap(&mut app, &context, SnapKind::Surface);
        assert_eq!(snap.root, Some(id));
        assert!(snap.world.into_iter().zip(expected).all(|(a, b)| (a - b).abs() < 1e-5));
        click(&mut app, &context, cursor);
        app.controller.submit("20,20,20").unwrap();
        let after = app.controller.document().clone();
        assert_eq!(after.entities.len(), 2);
        assert!(after.entities.values().any(|entity| {
            matches!(&entity.geometry, Geometry::Curve(Curve::Line { start, end: actual_end, .. })
                if *start == snap.world && *actual_end == end)
        }));
        assert_eq!(app.controller.coordinator().undo_len(), 1);
        app.controller.undo().unwrap();
        assert_eq!(app.controller.document(), &before);
        if mode == DisplayMode::Shaded {
            app.controller.redo().unwrap();
            assert_eq!(app.controller.document(), &after);
        }
    }

    assert_eq!(app.controller.begin_file_operation(FileKind::SaveAs).unwrap(), FileOutcome::Pending);
    let token = app.controller.file_operation().unwrap().token();
    assert_eq!(app.controller.accept_file_path(token, &output).unwrap(), FileOutcome::Pending);
    assert!(app.controller.file_save_worker_pending());
    // The v7 worker restores the source to verify both the encoding and the staged file.
    let save_started = Instant::now();
    let deadline = save_started + Duration::from_secs(150);
    loop {
        if let Some(result) = app.controller.poll_file_save_worker() {
            assert_eq!(result.unwrap(), FileOutcome::Saved);
            break;
        }
        assert!(Instant::now() < deadline, "native v7 Save As timed out");
        std::thread::sleep(Duration::from_millis(5));
    }
    let mut header = [0; 12];
    fs::File::open(&output).unwrap().read_exact(&mut header).unwrap();
    assert_eq!(&header[8..12], &7_u32.to_le_bytes());
    eprintln!("native v7 Save As completed in {:.1}s", save_started.elapsed().as_secs_f64());
    let reopened = MatrixApp::from_file(&output).unwrap();
    fs::remove_file(output).unwrap();
    assert!(reopened.controller.file_diagnostics().is_empty());
    assert_eq!(reopened.controller.document().entities, app.controller.document().entities);
}

#[cfg(feature = "occt-brep")]
#[test]
fn reopened_native_surface_snap_handles_rotated_nonuniform_placement() {
    let mut fixtures = BrepFixtures::default();
    let context = egui::Context::default();
    let limits = om9_core::budget::Limits::default();
    let sphere = om9_kernel::brep::sphere_brep(5., 0.5, &limits).unwrap();
    let native = om9_kernel::brep::to_3dm(&sphere, 0.001, &limits).unwrap();
    let mut entity = fixtures.native("transformed native sphere", native);
    let placement = om9_core::geometry::Transform([[0., -2., 0., 4.], [0.5, 0., 0., -2.], [0., 0., 1.5, 1.], [0., 0., 0., 1.]]);
    entity.placement = placement;
    let id = entity.id;
    let mut document = Document::with_template_tolerance("native transformed snap");
    document.entities.insert(id, entity);
    let mut app = fixtures.open(document);
    for _ in 0..3 { frame(&mut app, &context, vec![]); }
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.submit("Line").unwrap();
    app.set_snap_override(Some(SnapKind::Surface));
    let expected = placement.point([1., 1., 23_f64.sqrt()]).unwrap();
    let point = screen_of(&app, expected);
    frame(&mut app, &context, vec![Event::PointerMoved(point)]);
    let snap = wait_snap(&mut app, &context, SnapKind::Surface);
    assert_eq!(snap.root, Some(id));
    assert!((0..3).all(|i| (snap.world[i] - expected[i]).abs() < 1e-5),
        "expected {expected:?}, got {:?}", snap.world);
    let before = app.controller.document().clone();
    click(&mut app, &context, point);
    app.controller.submit("20,20,20").unwrap();
    let start = app.controller.document().entities.values().find_map(|entity| match &entity.geometry {
        Geometry::Curve(Curve::Line { start, .. }) => Some(*start), _ => None,
    }).unwrap();
    assert!((0..3).all(|i| (start[i] - snap.world[i]).abs() < 1e-6));
    let after = app.controller.document().clone();
    app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
    app.controller.redo().unwrap(); assert_eq!(app.controller.document(), &after);
}

#[cfg(feature = "occt-brep")]
#[test]
fn directional_snaps_on_cylinder_rim_commit_the_source_witness() {
    use om9_scene::{camera::Camera, workspace::DisplayMode};

    let context = egui::Context::default();
    let mut app = MatrixApp::new(Document::with_template_tolerance("directional rim snaps")).unwrap();
    app.controller.submit("Cylinder 2 10").unwrap();
    let before = app.controller.document().clone();
    let tolerance = before.effective_tolerance().unwrap().absolute_mm;
    let cylinder = before.entities.values().find(|entity| matches!(entity.geometry, Geometry::Brep(_))).unwrap().id;
    let undo = app.controller.coordinator().undo_len();
    let view_id = ViewId::LookingDown;
    app.controller.workspace.active = view_id;
    let view = app.controller.workspace.views.get_mut(&view_id).unwrap();
    view.camera = Camera::orthographic([14., -16., 30.], [0., 0., 5.], [0., 1., 0.], 16.).unwrap();
    for _ in 0..3 { frame(&mut app, &context, vec![]); }

    for mode in [DisplayMode::Wireframe, DisplayMode::Shaded] {
        app.controller.workspace.views.get_mut(&view_id).unwrap().mode = mode;
        for (kind, base, expected) in [
            (SnapKind::Tangent, [4., 0., 10.], [1., -3_f64.sqrt(), 10.]),
            (SnapKind::Perpendicular, [4., -3., 10.], [1.6, -1.2, 10.]),
        ] {
            app.controller.submit("Line").unwrap();
            let [x, y, z] = base;
            app.controller.submit(&format!("{x},{y},{z}")).unwrap();
            app.set_snap_override(Some(kind));
            let cursor = screen_of(&app, expected);
            frame(&mut app, &context, vec![Event::PointerMoved(cursor)]);
            let hit = wait_snap(&mut app, &context, kind);
            let position_error = hit.world.into_iter().zip(expected)
                .map(|(actual, wanted)| (actual - wanted).powi(2)).sum::<f64>().sqrt();
            assert!(position_error <= tolerance,
                "{mode:?} {kind:?}: expected {expected:?}, got {hit:?}");
            let [x, y, z] = hit.world;
            let radius = x.hypot(y);
            assert!((radius - 2.).abs() <= tolerance && (z - 10.).abs() <= tolerance);
            let offset = [base[0] - x, base[1] - y];
            let directional_error = if kind == SnapKind::Tangent {
                (offset[0] * x + offset[1] * y).abs() / radius
            } else {
                (offset[0] * y - offset[1] * x).abs() / radius
            };
            assert!(directional_error <= tolerance, "{mode:?} {kind:?}: residual {directional_error} mm");
            let source = hit.source.as_ref().unwrap();
            assert_eq!(source.root, cylinder);
            source.validate(app.controller.scene().unwrap()).unwrap();

            click(&mut app, &context, cursor);
            assert_eq!(app.snap_override(), None);
            let after = app.controller.document().clone();
            assert_eq!(after.entities.len(), before.entities.len() + 1);
            assert!(after.entities.values().any(|entity| {
                matches!(&entity.geometry, Geometry::Curve(Curve::Line { start, end, .. })
                    if *start == base && *end == hit.world)
            }));
            assert_eq!(app.controller.coordinator().undo_len(), undo + 1);
            app.controller.undo().unwrap();
            assert_eq!(app.controller.document(), &before);
        }
    }
}

#[cfg(feature = "occt-brep")]
#[test]
fn reopened_native_cylinder_edge_quad_matches_line_in_both_display_modes() {
    use om9_core::{budget::Limits, geometry::Transform};
    use om9_kernel::creation::{self, Primitive};
    use om9_scene::{camera::Camera, workspace::DisplayMode};

    let mut fixtures = BrepFixtures::default();
    let context = egui::Context::default();
    let limits = Limits::default();
    let cylinder = creation::primitive(
        Primitive::Cylinder {
            radius_mm: 2.,
            height_mm: 6.,
        },
        0.2,
        &limits,
    )
    .unwrap();
    let native = om9_kernel::brep::to_3dm(&cylinder, 0.001, &limits).unwrap();
    let mut entity = fixtures.native("placed native cylinder", native);
    entity.placement = Transform([
        [0., -2., 0., 4.],
        [0.5, 0., 0., -2.],
        [0., 0., 1.5, 1.],
        [0., 0., 0., 1.],
    ]);
    let source = entity.id;
    let mut document = Document::with_template_tolerance("native cylinder edge Quad");
    document.entities.insert(source, entity);
    let mut app = fixtures.open(document);
    let tolerance = app.controller.document().effective_tolerance().unwrap().absolute_mm;
    // The CPlane's positive X extremum comes from local (0, -2, 6).
    let expected = [8., -2., 10.];
    let before = app.controller.document().clone();
    let undo_before = app.controller.coordinator().undo_len();
    let view = ViewId::LookingDown;
    app.controller.workspace.active = view;
    app.controller.workspace.views.get_mut(&view).unwrap().camera =
        Camera::orthographic([24., -18., 35.], [4., -2., 5.5], [0., 1., 0.], 16.).unwrap();

    for mode in [DisplayMode::Wireframe, DisplayMode::Shaded] {
        app.controller.workspace.views.get_mut(&view).unwrap().mode = mode;
        frame(&mut app, &context, vec![Event::PointerGone]);
        app.controller.submit("Line").unwrap();
        app.set_snap_override(Some(SnapKind::Quadrant));
        let at = screen_of(&app, expected);
        frame(&mut app, &context, vec![Event::PointerMoved(at)]);
        let snap = wait_snap(&mut app, &context, SnapKind::Quadrant);
        assert_eq!(snap.source.as_ref().map(|hit| hit.root), Some(source));
        assert!(
            snap.world.iter().zip(expected).all(|(a, b)| (a - b).abs() <= tolerance),
            "{mode:?}: expected {expected:?}, marker was {:?}",
            snap.world,
        );

        click(&mut app, &context, at);
        assert_eq!(app.snap_override(), None);
        app.controller.submit("20,20,20").unwrap();
        let after = app.controller.document().clone();
        let start = after
            .entities
            .values()
            .find_map(|entity| match &entity.geometry {
                Geometry::Curve(Curve::Line { start, .. }) => Some(*start),
                _ => None,
            })
            .expect("Line committed from Quad");
        assert_eq!(start, snap.world);
        assert_eq!(app.controller.coordinator().undo_len(), undo_before + 1);
        app.controller.undo().unwrap();
        assert_eq!(app.controller.document(), &before);
        app.controller.redo().unwrap();
        assert_eq!(app.controller.document(), &after);
        app.controller.undo().unwrap();
    }
}

#[cfg(feature = "occt-brep")]
#[test]
fn reopened_native_sphere_snap_respects_near_clipping() {
    let mut fixtures = BrepFixtures::default();
    let limits = om9_core::budget::Limits::default();
    let sphere = om9_kernel::brep::sphere_brep(5., 0.5, &limits).unwrap();
    let native = om9_kernel::brep::to_3dm(&sphere, 0.001, &limits).unwrap();
    let entity = fixtures.native("native sphere", native);
    let id = entity.id;
    let mut document = Document::with_template_tolerance("native near clip");
    document.entities.insert(id, entity);
    let mut app = fixtures.open(document);
    let context = egui::Context::default();
    for _ in 0..3 { frame(&mut app, &context, vec![]); }
    app.controller.workspace.active = ViewId::LookingDown;
    let mut camera = om9_scene::camera::Camera::orthographic([0., 0., 20.], [0.; 3], [0., 1., 0.], 20.).unwrap();
    camera.near_mm = 20.;
    app.controller.workspace.views.get_mut(&ViewId::LookingDown).unwrap().camera = camera;
    app.controller.submit("Line").unwrap();
    app.set_snap_override(Some(SnapKind::Surface));
    let point = screen_of(&app, [1., 0., -24_f64.sqrt()]);
    frame(&mut app, &context, vec![Event::PointerMoved(point)]);
    let snap = wait_snap(&mut app, &context, SnapKind::Surface);
    assert_eq!(snap.root, Some(id));
    assert!((snap.world[2] + 24_f64.sqrt()).abs() < 1e-6, "clipped front must not be selected: {:?}", snap.world);
}

#[cfg(feature = "occt-brep")]
#[test]
fn reopening_unsupported_native_refuses_the_document() {
    let mut fixtures = BrepFixtures::default();
    let limits = om9_core::budget::Limits::default();
    let sphere = om9_kernel::brep::sphere_brep(5., 0.5, &limits).unwrap();
    let mut native = om9_kernel::brep::to_3dm(&sphere, 0.001, &limits).unwrap();
    native.payload = b"unsupported native BRep fixture".to_vec().into();
    native.crc = 1;
    let mut document = Document::with_template_tolerance("unsupported surface snap");
    let entity = fixtures.native("native sphere", native);
    document.entities.insert(entity.id, entity);
    assert!(fixtures.try_open(document).is_err(), "unreadable native source must refuse the file");
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
    let mut fixtures = BrepFixtures::default();
    let context = egui::Context::default();
    let limits = om9_core::budget::Limits::default();
    let sphere = om9_kernel::brep::sphere_brep(5., 0.5, &limits).unwrap();
    let mut document = Document::with_template_tolerance("snappy surface precision");
    let line = Entity::new("moving line", Geometry::Curve(Curve::Line {
        start: [-1., 0., 0.], end: [1., 0., 0.], domain: [0., 2.],
    }), None);
    let line_id = line.id;
    document.entities.insert(line_id, line);
    let mut target = fixtures.brep("target sphere", sphere);
    target.placement = om9_core::geometry::Transform::translation([10., 10., 0.]);
    document.entities.insert(target.id, target);
    let mut app = fixtures.open(document);
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
    let mut document = Document::with_template_tolerance("osnap ui");
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
    frame(&mut app, &context, vec![Event::PointerMoved(first)]);
    let marker = wait_snap(&mut app, &context, SnapKind::End);
    assert_eq!(
        marker.kind,
        SnapKind::End,
        "the pointer is close enough to the line end to show END"
    );
    assert_eq!(
        marker.tooltip(),
        "End point"
    );
    click(&mut app, &context, first);
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
    frame(&mut app, &context, vec![Event::PointerMoved(second)]);
    let marker = wait_snap(&mut app, &context, SnapKind::End);
    assert_eq!(marker.world, [10., 0., 0.]);
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
    let mut document = Document::with_template_tolerance("osnap escape");
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
fn smarttrack_disabled_does_not_keep_accepted_snap_anchors() {
    let context = egui::Context::default();
    let mut document = Document::with_template_tolerance("disabled SmartTrack");
    let line = Entity::new("reference", Geometry::Curve(Curve::Line {
        start: [0., 0., 0.], end: [10., 0., 0.], domain: [0., 10.],
    }), None);
    document.entities.insert(line.id, line);
    let mut app = MatrixApp::new(document).unwrap();
    app.set_smarttrack_enabled(false);
    frame(&mut app, &context, vec![]);
    frame(&mut app, &context, vec![]);
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.submit("Line").unwrap();
    let end = screen_of(&app, [0., 0., 0.]);
    frame(&mut app, &context, vec![Event::PointerMoved(end)]);
    assert_eq!(wait_snap(&mut app, &context, SnapKind::End).world, [0.; 3]);
    click(&mut app, &context, end);
    assert!(app.controller.prompt().starts_with("End of line"));
    assert!(app.smarttrack_guides().is_empty());
    frame(&mut app, &context, vec![Event::PointerGone]);
    app.set_smarttrack_enabled(true);
    frame(&mut app, &context, vec![]);
    assert!(app.smarttrack_guides().is_empty(), "disabled interval left a guide");
    frame(&mut app, &context, vec![Event::PointerMoved(end)]);
    wait_snap(&mut app, &context, SnapKind::End);
    dwell_smarttrack(&mut app, &context);
    assert_eq!(app.smarttrack_guides().len(), 2);
}

#[test]
fn smarttrack_pauses_over_a_source_before_capturing_it() {
    let context = egui::Context::default();
    let mut document = Document::with_template_tolerance("SmartTrack hover delay");
    let source = Entity::new("reference", Geometry::Curve(Curve::Line {
        start: [0., 0., 0.], end: [10., 0., 0.], domain: [0., 10.],
    }), None);
    document.entities.insert(source.id, source);
    let second = Entity::new("second reference", Geometry::Point([8., 8., 0.]), None);
    document.entities.insert(second.id, second);
    let mut app = MatrixApp::new(document).unwrap();
    app.set_snap_enabled(SnapKind::Point, true);
    frame_at_time(&mut app, &context, vec![], Some(0.));
    frame_at_time(&mut app, &context, vec![], Some(0.02));
    app.controller.submit("Line").unwrap();

    let reference = screen_of(&app, [0., 0., 0.]);
    frame_at_time(&mut app, &context, vec![Event::PointerMoved(reference)], Some(0.1));
    assert_eq!(app.snap_candidate().unwrap().kind, SnapKind::End);
    assert!(app.smarttrack_guides().is_empty(), "passing over End must not acquire it");
    frame_at_time(&mut app, &context, vec![Event::PointerGone], Some(0.2));
    frame_at_time(&mut app, &context, vec![Event::PointerMoved(reference)], Some(0.7));
    assert!(app.smarttrack_guides().is_empty(), "returning starts a new hover");
    frame_at_time(&mut app, &context, vec![], Some(1.3));
    assert_eq!(app.smarttrack_guides().len(), 2);
    let second = screen_of(&app, [8., 8., 0.]);
    frame_at_time(&mut app, &context, vec![Event::PointerMoved(second)], Some(1.4));
    assert_eq!(wait_snap(&mut app, &context, SnapKind::Point).world, [8., 8., 0.]);
    dwell_smarttrack(&mut app, &context);
    assert_eq!(app.smarttrack_guides().len(), 4);
    assert_eq!(app.controller.document().entities.len(), 2);
    assert_eq!(app.controller.coordinator().undo_len(), 0);

    let guide_pick = screen_of(&app, [0.12, 8.1, 0.]);
    frame(&mut app, &context, vec![Event::PointerMoved(guide_pick)]);
    let marker = wait_snap(&mut app, &context, SnapKind::SmartTrack);
    assert!(marker.world[0].abs() < 1e-6 && (marker.world[1] - 8.).abs() < 1e-4);
    click(&mut app, &context, guide_pick);
    assert_eq!(app.controller.relative_base().unwrap().xyz(), marker.world);
    app.controller.submit("4,8,0").unwrap();
    assert_eq!(app.controller.document().entities.len(), 3);
    assert_eq!(app.controller.coordinator().undo_len(), 1);
}

#[test]
fn perpendicular_refuses_a_segment_extension_and_accepts_a_true_foot() {
    let context = egui::Context::default();
    let mut document = Document::with_template_tolerance("Perp on bounded source lines");
    let segment = Entity::new("short horizontal", Geometry::Curve(Curve::Line {
        start: [0., 0., 0.], end: [10., 0., 0.], domain: [0., 10.],
    }), None);
    let reachable = Entity::new("elevated horizontal", Geometry::Curve(Curve::Line {
        start: [0., -8., 3.], end: [20., -8., 3.], domain: [0., 20.],
    }), None);
    let source = reachable.id;
    document.entities.insert(segment.id, segment);
    document.entities.insert(source, reachable);
    let mut app = MatrixApp::new(document).unwrap();
    frame(&mut app, &context, vec![]);
    frame(&mut app, &context, vec![]);
    let before = app.controller.document().clone();
    app.controller.submit("Line").unwrap();
    app.controller.submit("16,5,0").unwrap();
    app.set_snap_override(Some(SnapKind::Perpendicular));

    let endpoint = screen_of(&app, [10., 0., 0.]);
    frame(&mut app, &context, vec![Event::PointerMoved(endpoint)]);
    assert!(app.snap_candidate().is_none(), "a clamped endpoint is not the perpendicular foot");
    click(&mut app, &context, endpoint);
    assert_eq!(app.controller.document(), &before);
    assert_eq!(app.controller.coordinator().undo_len(), 0);
    assert_eq!(app.snap_override(), Some(SnapKind::Perpendicular));

    let expected = [16., -8., 3.];
    let at = screen_of(&app, expected);
    frame(&mut app, &context, vec![Event::PointerMoved(at)]);
    let marker = wait_snap(&mut app, &context, SnapKind::Perpendicular);
    assert_eq!(marker.world, expected);
    assert_eq!(marker.source.as_ref().map(|hit| hit.root), Some(source));
    click(&mut app, &context, at);
    assert_eq!(app.snap_override(), None);
    let committed = app.controller.document().clone();
    assert!(committed.entities.values().any(|entity| matches!(&entity.geometry,
        Geometry::Curve(Curve::Line { start, end, .. }) if *start == [16., 5., 0.] && *end == expected)));
    assert_eq!(app.controller.coordinator().undo_len(), 1);
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &before);
    app.controller.redo().unwrap();
    assert_eq!(app.controller.document(), &committed);
}

#[test]
#[cfg(feature = "occt-brep")]
fn brep_end_tracking_keeps_a_shared_tangent_and_refuses_a_corner_direction() {
    use om9_core::budget::Limits;
    use om9_kernel::creation::{self, Profile};

    let context = egui::Context::default();
    let mut fixtures = BrepFixtures::default();
    let corner = [0., 0., 12.];
    let smooth_join = [4., 2., 12.];
    let profile = Profile::new(vec![corner, smooth_join, [8., 4., 12.], [8., -4., 12.], [0., -4., 12.]]).unwrap();
    let face = creation::planar(&profile, 0.2, &Limits::default()).unwrap();
    let entity = fixtures.brep("planar face with a split edge", face);
    let id = entity.id;
    let mut document = Document::with_template_tolerance("SmartTrack at two BRep vertices");
    document.entities.insert(id, entity);
    let mut app = fixtures.open(document);
    frame(&mut app, &context, vec![]);
    frame(&mut app, &context, vec![]);
    let before = app.controller.document().clone();
    app.controller.submit("Line").unwrap();

    let at = screen_of(&app, corner);
    frame(&mut app, &context, vec![Event::PointerMoved(at)]);
    let end = wait_snap(&mut app, &context, SnapKind::End);
    assert_eq!(end.world, corner);
    assert_eq!(end.source.as_ref().map(|source| source.root), Some(id));
    assert!(end.tracking_tangent.is_none(), "a sharp corner must not choose one edge's tangent");
    dwell_smarttrack(&mut app, &context);
    assert_eq!(app.smarttrack_guides().len(), 2);

    let at = screen_of(&app, smooth_join);
    frame(&mut app, &context, vec![Event::PointerMoved(at)]);
    let end = wait_snap(&mut app, &context, SnapKind::End);
    let tolerance = app.controller.document().effective_tolerance().unwrap().absolute_mm;
    assert!(end.world.iter().zip(smooth_join).all(|(a, b)| (a - b).abs() <= tolerance));
    assert_eq!(end.source.as_ref().map(|source| source.root), Some(id));
    let tangent = end.tracking_tangent.expect("incident edges share a line at the smooth join");
    assert!((tangent[0] - 2. * tangent[1]).abs() < 1e-9 && tangent[2].abs() < 1e-9);
    dwell_smarttrack(&mut app, &context);
    assert_eq!(app.smarttrack_guides().len(), 6);

    let expected = [14., 7., 12.];
    let at = screen_of(&app, expected);
    frame(&mut app, &context, vec![Event::PointerMoved(at)]);
    let marker = wait_snap(&mut app, &context, SnapKind::SmartTrack);
    assert!(marker.world.iter().zip(expected).all(|(a, b)| (a - b).abs() < 1e-4));
    click(&mut app, &context, at);
    app.controller.submit("14,10,12").unwrap();
    assert!(app.controller.document().entities.values().any(|entity| matches!(&entity.geometry,
        Geometry::Curve(Curve::Line { start, .. }) if *start == marker.world)));
    assert_eq!(app.controller.coordinator().undo_len(), 1);
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &before);
}

#[test]
fn smarttrack_arc_end_supplies_placed_tangent_and_perpendicular_guides() {
    use om9_core::geometry::Transform;
    use std::f64::consts::{FRAC_1_SQRT_2, FRAC_PI_2, FRAC_PI_4};

    let context = egui::Context::default();
    let mut document = Document::with_template_tolerance("SmartTrack directions from an arc");
    let mut source = Entity::new("placed arc", Geometry::Curve(Curve::Circle {
        center: [0., 0., 12.], x: [1., 0., 0.], y: [0., 1., 0.], radius: 4.,
        domain: [0., 1.], angle_domain: Some([FRAC_PI_4, FRAC_PI_2]),
    }), None);
    source.placement = Transform([
        [1.5, 0., 0., 0.], [0., 0.5, 0., 0.], [0., 0., 1., 0.], [0., 0., 0., 1.],
    ]);
    let id = source.id;
    document.entities.insert(id, source);
    let mut app = MatrixApp::new(document).unwrap();
    frame(&mut app, &context, vec![]);
    frame(&mut app, &context, vec![]);
    let before = app.controller.document().clone();
    app.controller.submit("Line").unwrap();

    let anchor = [6. * FRAC_1_SQRT_2, 2. * FRAC_1_SQRT_2, 12.];
    let at = screen_of(&app, anchor);
    frame(&mut app, &context, vec![Event::PointerMoved(at)]);
    let marker = wait_snap(&mut app, &context, SnapKind::End);
    assert_eq!(marker.source.as_ref().map(|source| source.root), Some(id));
    dwell_smarttrack(&mut app, &context);
    assert_eq!(app.smarttrack_guides().len(), 4, "the arc adds two source directions");

    // The placed ellipse has derivative proportional to (-3, 1, 0) at its start.
    let tangent = [anchor[0] + 6., anchor[1] - 2., anchor[2]];
    let perpendicular = [anchor[0] + 2., anchor[1] + 6., anchor[2]];
    let mut selected = Vec::new();
    for expected in [tangent, perpendicular] {
        let at = screen_of(&app, expected);
        frame(&mut app, &context, vec![Event::PointerMoved(at)]);
        let hit = wait_snap(&mut app, &context, SnapKind::SmartTrack);
        assert!(hit.world.iter().zip(expected).all(|(a, b)| (a - b).abs() < 1e-4),
            "expected point on the placed direction {expected:?}, got {:?}", hit.world);
        assert!(hit.source.is_none());
        selected.push(hit.world);
        click(&mut app, &context, at);
    }
    let committed = app.controller.document().clone();
    assert!(committed.entities.values().any(|entity| matches!(&entity.geometry,
        Geometry::Curve(Curve::Line { start, end, .. }) if *start == selected[0] && *end == selected[1])));
    assert_eq!(app.controller.coordinator().undo_len(), 1);
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &before);
    app.controller.redo().unwrap();
    assert_eq!(app.controller.document(), &committed);
}

#[test]
fn perpendicular_one_shot_can_pick_a_smarttrack_guide() {
    let context = egui::Context::default();
    let mut document = Document::with_template_tolerance("Perp on a SmartTrack guide");
    let source = Entity::new("guide reference", Geometry::Point([0., 0., 12.]), None);
    document.entities.insert(source.id, source);
    let mut app = MatrixApp::new(document).unwrap();
    app.set_snap_enabled(SnapKind::Point, true);
    frame(&mut app, &context, vec![]);
    frame(&mut app, &context, vec![]);
    let before = app.controller.document().clone();
    app.controller.submit("Line").unwrap();

    let at = screen_of(&app, [0., 0., 12.]);
    frame(&mut app, &context, vec![Event::PointerMoved(at)]);
    assert_eq!(wait_snap(&mut app, &context, SnapKind::Point).world, [0., 0., 12.]);
    dwell_smarttrack(&mut app, &context);
    assert_eq!(app.smarttrack_guides().len(), 2);

    app.controller.submit("9,4,0").unwrap();
    app.set_object_snaps_enabled(false);
    app.set_snap_override(Some(SnapKind::Perpendicular));
    let on_guide_but_not_perp = screen_of(&app, [15., 0., 12.]);
    click(&mut app, &context, on_guide_but_not_perp);
    assert!(app.snap_candidate().is_none());
    assert_eq!(app.controller.document(), &before);
    assert_eq!(app.snap_override(), Some(SnapKind::Perpendicular));

    let foot = [9., 0., 12.];
    let at = screen_of(&app, foot);
    frame(&mut app, &context, vec![Event::PointerMoved(at)]);
    let marker = app.snap_candidate().expect("perpendicular to the SmartTrack guide");
    assert_eq!(marker.kind, SnapKind::Perpendicular);
    assert_eq!(marker.world, foot);
    assert!(marker.source.is_none());
    click(&mut app, &context, at);
    assert_eq!(app.snap_override(), None);
    let committed = app.controller.document().clone();
    assert!(committed.entities.values().any(|entity| matches!(&entity.geometry,
        Geometry::Curve(Curve::Line { start, end, .. }) if *start == [9., 4., 0.] && *end == foot)));
    assert_eq!(app.controller.coordinator().undo_len(), 1);
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &before);
    app.controller.redo().unwrap();
    assert_eq!(app.controller.document(), &committed);
}

#[test]
fn smarttrack_keeps_the_visible_guide_at_source_height() {
    let context = egui::Context::default();
    let mut document = Document::with_template_tolerance("elevated SmartTrack guide");
    let source = Entity::new("guide anchor", Geometry::Point([0., 0., 12.]), None);
    document.entities.insert(source.id, source);
    let mut app = MatrixApp::new(document).unwrap();
    app.set_snap_enabled(SnapKind::Point, true);
    frame(&mut app, &context, vec![]);
    frame(&mut app, &context, vec![]);
    app.controller.submit("Line").unwrap();

    let anchor = screen_of(&app, [0., 0., 12.]);
    frame(&mut app, &context, vec![Event::PointerMoved(anchor)]);
    assert_eq!(wait_snap(&mut app, &context, SnapKind::Point).world, [0., 0., 12.]);
    dwell_smarttrack(&mut app, &context);
    assert_eq!(app.smarttrack_guides().len(), 2);

    let guide_point = screen_of(&app, [12., 0., 12.]);
    let output = frame_at_time(&mut app, &context, vec![Event::PointerMoved(guide_point)], None);
    let viewport_width = app.view_rects[&ViewId::LookingDown].width();
    assert!(output.shapes.iter().any(|clip| clip.clip_rect.contains(guide_point)
        && matches!(&clip.shape, egui::Shape::LineSegment { points: [start, end], stroke }
            if stroke.color == egui::Color32::from_gray(200)
                && (start.y - guide_point.y).abs() < 0.01
                && (end.y - guide_point.y).abs() < 0.01
                && start.x.min(end.x) < guide_point.x
                && start.x.max(end.x) > guide_point.x
                && (start.x - end.x).abs() > viewport_width / 2.)),
        "the elevated SmartTrack guide should render as one solid gray line");
    let marker = app.snap_candidate().expect("visible elevated SmartTrack guide").clone();
    assert_eq!(marker.kind, SnapKind::SmartTrack);
    assert!(marker.world.iter().zip([12., 0., 12.]).all(|(a, b)| (a - b).abs() < 1e-4),
        "unexpected guide point {:?}", marker.world);
    assert!(marker.source.is_none());

    click(&mut app, &context, guide_point);
    app.controller.submit("4,8,0").unwrap();
    assert!(app.controller.document().entities.values().any(|entity| {
        matches!(&entity.geometry, Geometry::Curve(Curve::Line { start, .. }) if *start == marker.world)
    }));
    assert_eq!(app.controller.coordinator().undo_len(), 1);
}

#[test]
fn smarttrack_guides_expire_before_the_pointer_returns_to_the_viewport() {
    let context = egui::Context::default();
    let mut document = Document::with_template_tolerance("SmartTrack scope");
    let line = Entity::new("reference", Geometry::Curve(Curve::Line {
        start: [0., 0., 0.], end: [10., 0., 0.], domain: [0., 10.],
    }), None);
    let id = line.id;
    document.entities.insert(id, line);
    let mut app = MatrixApp::new(document).unwrap();
    frame(&mut app, &context, vec![]);
    frame(&mut app, &context, vec![]);
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.submit("Line").unwrap();
    let end = screen_of(&app, [0., 0., 0.]);
    frame(&mut app, &context, vec![Event::PointerMoved(end)]);
    wait_snap(&mut app, &context, SnapKind::End);
    dwell_smarttrack(&mut app, &context);
    assert_eq!(app.smarttrack_guides().len(), 2);
    frame(&mut app, &context, vec![Event::PointerGone]);
    let publication = app.controller.scene().unwrap().publication_id();
    app.controller.cancel();
    // A new command can start before the UI renders an idle frame. The
    // Polyline prompt stays live through the later edit, isolating scene expiry.
    app.controller.submit("Polyline").unwrap();
    assert_eq!(app.controller.scene().unwrap().publication_id(), publication);
    frame(&mut app, &context, vec![]);
    assert!(app.smarttrack_guides().is_empty(), "previous Line left a guide");

    frame(&mut app, &context, vec![Event::PointerMoved(end)]);
    wait_snap(&mut app, &context, SnapKind::End);
    dwell_smarttrack(&mut app, &context);
    assert_eq!(app.smarttrack_guides().len(), 2);
    frame(&mut app, &context, vec![Event::PointerGone]);
    app.controller.select_objects(&[id]).unwrap();
    app.controller.numeric_move([0., 20., 0.]).unwrap();
    assert!(app.controller.is_running());
    assert_ne!(app.controller.scene().unwrap().publication_id(), publication);
    frame(&mut app, &context, vec![]);
    assert!(app.smarttrack_guides().is_empty(), "old scene left a guide");
}

#[test]
fn disabling_end_removes_the_marker_from_the_next_frame() {
    let context = egui::Context::default();
    let mut document = Document::with_template_tolerance("osnap disabled");
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
    assert_eq!(wait_snap(&mut app, &context, SnapKind::End).kind, SnapKind::End);
    app.set_snap_enabled(SnapKind::End, false);
    let empty = screen_of(&app, [5., 8., 0.]);
    frame(&mut app, &context, vec![Event::PointerMoved(empty)]);
    assert!(app.snap_candidate().is_none(), "disabled END has no marker");
    app.set_snap_enabled(SnapKind::End, true);
    frame(&mut app, &context, vec![Event::PointerMoved(at)]);
    assert_eq!(wait_snap(&mut app, &context, SnapKind::End).kind, SnapKind::End);
}

#[cfg(feature = "occt-brep")]
#[test]
fn extrude_lines_to_boundary_resolves_the_picked_3dm_face() {
    let limits = Default::default();
    let body = om9_kernel::brep::box_brep([8., 8., 2.], 0.05, &limits).unwrap();
    let native = om9_kernel::brep::to_3dm(&body, 0.001, &limits).unwrap();
    let mut fixtures = BrepFixtures::default();
    let mut boundary = fixtures.native("Rhino box", native);
    boundary.placement = om9_core::geometry::Transform::translation([-4., -4., 4.]);
    let mut doc = Document::with_template_tolerance("3DM extrusion stop");
    let source = Entity::new(
        "curve",
        Geometry::Curve(Curve::Line {
            start: [-1., 0., 0.],
            end: [1., 0., 0.],
            domain: [0., 1.],
        }),
        doc.current_layer,
    );
    let id = source.id;
    doc.entities.insert(source.id, source);
    doc.entities.insert(boundary.id, boundary);
    let path = fixtures.write(doc);
    let mut app = MatrixApp::new(Document::with_template_tolerance("Untitled")).unwrap();
    app.controller.begin_file_operation(om9_app::file_lifecycle::FileKind::Open).unwrap();
    let token = app.controller.file_operation().unwrap().token();
    let opened = app.controller.accept_file_path(token, &path);
    std::fs::remove_file(path).unwrap();
    opened.unwrap();
    let before = app.controller.document().clone();
    app.controller.activate_view("Looking Down").unwrap();
    app.controller
        .workspace
        .views
        .get_mut(&ViewId::LookingDown)
        .unwrap()
        .mode = om9_scene::workspace::DisplayMode::Shaded;
    app.controller.select_objects(&[id]).unwrap();
    app.controller.submit("tsExtrudeLines").unwrap();
    app.controller.submit("ToBoundary").unwrap();
    let ctx = egui::Context::default();
    for _ in 0..3 {
        frame(&mut app, &ctx, vec![]);
    }
    let pick = screen_of(&app, [2., 2., 6.]);
    click(&mut app, &ctx, pick);
    for _ in 0..4 {
        frame(&mut app, &ctx, vec![]);
    }
    assert!(
        app.controller.addon_session().is_none(),
        "{} {:?}",
        app.status_line(),
        app.controller.addon_session().map(|s| &s.reply.messages)
    );
    let mut ends = Vec::new();
    for id in app.controller.selected_ordered() {
        if let Geometry::Curve(Curve::Line { start, end, .. }) =
            &app.controller.document().entities[&id].geometry
            && (start[0] - end[0]).abs() < 1e-7
        {
            ends.push(end[2]);
        }
    }
    assert_eq!(ends.len(), 2);
    assert!(ends.iter().all(|&z| (z - 6.).abs() < 1e-6), "{ends:?}");
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &before);
}
