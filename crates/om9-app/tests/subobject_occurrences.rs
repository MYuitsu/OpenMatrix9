// SPDX-License-Identifier: LGPL-2.1-or-later
//! Exact occurrence paths distinguish repeated shared mesh topology and drive all four overlays.
use eframe::egui::{self, Event, Modifiers, Pos2, Rect, Vec2};
use om9_app::{
    session::{Controller, SelectedSubobject},
    ui::MatrixApp,
};
use om9_core::{
    budget::Limits,
    geometry::{Geometry, Mesh, Transform},
    identity::ObjectId,
    model::{Definition, Document, Entity},
};
use om9_scene::{
    camera::{Camera, Viewport},
    scene::{PickIntent, PickKind, Subelement},
    workspace::ViewId,
};

struct Fixture {
    document: Document,
    a: ObjectId,
    b: ObjectId,
    left: ObjectId,
    right: ObjectId,
    leaf: ObjectId,
}
fn fixture(hidden: bool) -> Fixture {
    let mut document = Document::new("nested shared topology");
    let mut leaf = Entity::new(
        "mesh member",
        Geometry::Mesh(Mesh {
            vertices: vec![
                [-1., -1., -1.],
                [1., -1., -1.],
                [1., 1., -1.],
                [-1., 1., -1.],
                [-1., -1., 1.],
                [1., -1., 1.],
                [1., 1., 1.],
                [-1., 1., 1.],
            ],
            faces: vec![
                vec![0, 3, 2, 1],
                vec![4, 5, 6, 7],
                vec![0, 1, 5, 4],
                vec![1, 2, 6, 5],
                vec![2, 3, 7, 6],
                vec![3, 0, 4, 7],
            ],
            normals: vec![],
            colors: vec![],

            corner_uvs: Vec::new(),
        }),
        None,
    );
    leaf.attributes.hidden = hidden;
    let leaf_id = leaf.id;
    let inner = Definition::new("inner", vec![leaf]);
    let inner_id = inner.id;
    document.definitions.insert(inner_id, inner);
    let mut left = Entity::new("left", Geometry::Instance(inner_id), None);
    left.placement = Transform::translation([0., -3., 0.]);
    let mut right = Entity::new("right", Geometry::Instance(inner_id), None);
    right.placement = Transform::translation([0., 3., 0.]);
    let (left_id, right_id) = (left.id, right.id);
    let outer = Definition::new("outer", vec![left, right]);
    let outer_id = outer.id;
    document.definitions.insert(outer_id, outer);
    let mut a = Entity::new("occurrence A", Geometry::Instance(outer_id), None);
    a.placement = Transform::translation([-8., 0., 0.]);
    let mut b = Entity::new("mirrored occurrence B", Geometry::Instance(outer_id), None);
    b.placement = Transform([
        [-1., 0., 0., 8.],
        [0., 1., 0., 0.],
        [0., 0., 1., 0.],
        [0., 0., 0., 1.],
    ]);
    let (aid, bid) = (a.id, b.id);
    document.entities.insert(aid, a);
    document.entities.insert(bid, b);
    Fixture {
        document,
        a: aid,
        b: bid,
        left: left_id,
        right: right_id,
        leaf: leaf_id,
    }
}
fn reference(c: &Controller, root: ObjectId, path: &[ObjectId]) -> SelectedSubobject {
    let viewport = Viewport {
        origin: [0., 0.],
        size: [800., 600.],
    };
    let camera = Camera::orthographic([0., 0., 100.], [0.; 3], [0., 1., 0.], 40.).unwrap();
    let world = c
        .document()
        .member_transform(root, path)
        .unwrap()
        .point([0.3, 0.1, 1.])
        .unwrap();
    let screen = camera.project(world, viewport).unwrap().unwrap().screen;
    let hit = c
        .scene()
        .unwrap()
        .pick_kind(
            screen,
            &camera,
            viewport,
            1.,
            PickIntent::Select,
            PickKind::Face,
        )
        .unwrap()
        .unwrap();
    assert_eq!(hit.root, root);
    assert_eq!(hit.path, path);
    assert_eq!(hit.subset, Subelement::MeshFace(1));
    SelectedSubobject::from_hit(&hit)
}
#[test]
fn shared_leaf_faces_remain_distinct_across_mirrored_root_occurrences() {
    let f = fixture(false);
    let mut c = Controller::new(f.document, Limits::default()).unwrap();
    let path = [f.left, f.leaf];
    let a = reference(&c, f.a, &path);
    let b = reference(&c, f.b, &path);
    c.select_subobject_ref(a.clone()).unwrap();
    c.select_subobject_ref(b.clone()).unwrap();
    assert_eq!(c.subselection(), [a.clone(), b.clone()]);
    c.toggle_subobject_ref(a).unwrap();
    assert_eq!(c.subselection(), [b]);
    assert_eq!(c.coordinator().undo_len(), 0);
}
#[test]
fn two_paths_inside_one_root_keep_the_same_leaf_face_as_two_selections() {
    let f = fixture(false);
    let mut c = Controller::new(f.document, Limits::default()).unwrap();
    let a = reference(&c, f.a, &[f.left, f.leaf]);
    let b = reference(&c, f.a, &[f.right, f.leaf]);
    c.select_subobject_ref(a.clone()).unwrap();
    c.select_subobject_ref(b.clone()).unwrap();
    c.select_subobject_ref(a.clone()).unwrap();
    assert_eq!(c.subselection().len(), 2);
    assert_eq!(a.root, b.root);
    assert_eq!(a.subset, b.subset);
    assert_ne!(a.path, b.path);
    c.toggle_subobject_ref(b).unwrap();
    assert_eq!(c.subselection(), [a]);
    assert!(
        c.select_subobject(f.a, Subelement::MeshFace(1)).is_err(),
        "missing path cannot guess a member"
    );
}
#[test]
fn invalid_path_source_and_old_publication_do_not_rematch() {
    let f = fixture(false);
    let mut c = Controller::new(f.document, Limits::default()).unwrap();
    let a = reference(&c, f.a, &[f.left, f.leaf]);
    let mut bad = a.clone();
    bad.path[0] = ObjectId::new();
    assert!(c.select_subobject_ref(bad).is_err());
    let mut bad = a.clone();
    bad.source = c.document().stamp(f.b).unwrap();
    assert!(c.select_subobject_ref(bad).is_err());
    c.select_subobject_ref(a.clone()).unwrap();
    c.select_objects(&[f.a]).unwrap();
    c.numeric_move([2., 0., 0.]).unwrap();
    assert!(c.subselection().is_empty());
    assert!(c.select_subobject_ref(a.clone()).is_err());
    c.undo().unwrap();
    assert!(c.select_subobject_ref(a).is_err());
    assert!(c.subselection().is_empty());
}
#[test]
fn hidden_nested_member_is_not_a_selectable_subobject() {
    let f = fixture(true);
    let mut c = Controller::new(f.document, Limits::default()).unwrap();
    let o = c
        .scene()
        .unwrap()
        .objects
        .iter()
        .find(|o| o.root == f.a && o.path == [f.left, f.leaf])
        .unwrap();
    let key = SelectedSubobject {
        root: o.root,
        path: o.path.clone(),
        subset: Subelement::MeshFace(1),
        source: o.source,
        world_epoch: c.coordinator().epoch(),
    };
    assert!(c.select_subobject_ref(key).is_err());
    assert!(c.subselection().is_empty());
}
fn frame(
    app: &mut MatrixApp,
    ctx: &egui::Context,
    events: Vec<Event>,
) -> Vec<egui::epaint::ClippedShape> {
    let mut output = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 720.))),
            events,
            ..Default::default()
        },
        |ui| app.show(ui),
    );
    output.textures_delta.clear();
    output.shapes
}
#[test]
fn ctrl_shift_click_keeps_the_full_path_and_outlines_only_that_member_in_four_views() {
    let f = fixture(false);
    let mut app = MatrixApp::new(f.document).unwrap();
    let ctx = egui::Context::default();
    for _ in 0..4 {
        frame(&mut app, &ctx, vec![]);
    }
    let ids = [f.left, f.leaf];
    let world = app
        .controller
        .document()
        .member_transform(f.a, &ids)
        .unwrap()
        .point([0.3, 0.1, 1.])
        .unwrap();
    let rect = app.view_rects[&ViewId::LookingDown];
    let vp = Viewport {
        origin: [rect.min.x as f64, rect.min.y as f64],
        size: [rect.width() as f64, rect.height() as f64],
    };
    let p = app.controller.workspace.views[&ViewId::LookingDown]
        .camera
        .project(world, vp)
        .unwrap()
        .unwrap()
        .screen;
    let pos = egui::pos2(p[0] as f32, p[1] as f32);
    let modifiers = Modifiers::CTRL | Modifiers::SHIFT;
    frame(
        &mut app,
        &ctx,
        vec![Event::ModifiersChanged(modifiers), Event::PointerMoved(pos)],
    );
    for pressed in [true, false] {
        frame(
            &mut app,
            &ctx,
            vec![
                Event::ModifiersChanged(modifiers),
                Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers,
                },
            ],
        );
    }
    let key = app.sub_object().unwrap();
    assert_eq!(key.root, f.a);
    assert_eq!(key.path, ids);
    assert_eq!(key.subset, Subelement::MeshFace(1));
    let shapes = frame(
        &mut app,
        &ctx,
        vec![Event::ModifiersChanged(Modifiers::NONE)],
    );
    let transform = app
        .controller
        .document()
        .member_transform(f.a, &ids)
        .unwrap();
    for view in [
        ViewId::LookingDown,
        ViewId::Perspective,
        ViewId::ThroughFinger,
        ViewId::SideView,
    ] {
        let rect = app.view_rects[&view];
        let viewport = Viewport {
            origin: [rect.min.x as f64, rect.min.y as f64],
            size: [rect.width() as f64, rect.height() as f64],
        };
        let expected: Vec<_> = [
            [-1., -1., 1.],
            [1., -1., 1.],
            [1., 1., 1.],
            [-1., 1., 1.],
            [-1., -1., 1.],
        ]
        .into_iter()
        .map(|p| {
            let p = app.controller.workspace.views[&view]
                .camera
                .project(transform.point(p).unwrap(), viewport)
                .unwrap()
                .unwrap()
                .screen;
            egui::pos2(p[0] as f32, p[1] as f32)
        })
        .collect();
        let matching = shapes
            .iter()
            .filter(|shape| shape.clip_rect == rect)
            .filter(|shape| match &shape.shape {
                egui::Shape::Path(path) => path.points == expected && path.stroke.width == 2.,
                _ => false,
            })
            .count();
        assert_eq!(
            matching, 1,
            "{view:?}: exact selected occurrence polygon is drawn once"
        );
    }
    assert_eq!(app.controller.coordinator().undo_len(), 0);
}
