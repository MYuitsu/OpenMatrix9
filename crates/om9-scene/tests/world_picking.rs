// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    budget::Limits,
    geometry::{Curve, Geometry, Mesh, Transform},
    model::{Definition, Document, Entity},
    transaction::*,
};
use om9_scene::{camera::*, scene::*};
fn fixture() -> (
    TransactionCoordinator,
    om9_core::identity::DocumentId,
    om9_core::identity::ObjectId,
) {
    let mut d = Document::new("scene");
    let e = Entity::new(
        "curve",
        Geometry::Curve(Curve::Line {
            start: [0., 0., 0.],
            end: [10., 0., 0.],
            domain: [0., 10.],
        }),
        d.current_layer,
    );
    let id = e.id;
    d.entities.insert(id, e);
    let did = d.id;
    (
        TransactionCoordinator::new(
            World::new(vec![d], &Limits::default()).unwrap(),
            Limits::default(),
        )
        .unwrap(),
        did,
        id,
    )
}
fn camera() -> Camera {
    Camera::orthographic([0., 0., 100.], [0., 0., 0.], [0., 1., 0.], 40.).unwrap()
}
fn viewport() -> Viewport {
    Viewport {
        origin: [0., 0.],
        size: [400., 400.],
    }
}
#[test]
fn mapped_circle_sampling_uses_angular_span_and_keeps_native_parameters() {
    let mut doc = Document::new("mapped circle display");
    let entity = Entity::new(
        "mapped circle",
        Geometry::Curve(Curve::Circle {
            center: [0.; 3],
            x: [1., 0., 0.],
            y: [0., 1., 0.],
            radius: 10.,
            domain: [11., 1_000_011.],
            angle_domain: Some([1., 1. + std::f64::consts::TAU]),
        }),
        None,
    );
    let id = entity.id;
    doc.entities.insert(id, entity);
    let scene = Scene::capture(std::sync::Arc::new(doc), 1, &Limits::default()).unwrap();
    let object = scene.objects.iter().find(|o| o.root == id).unwrap();
    let RenderBuffers::Curve { points, parameters } = object.buffers.as_ref() else {
        panic!("curve buffers")
    };
    assert!(
        points.len() < 1000,
        "native parameter span is not angular display length"
    );
    assert_eq!(parameters.first(), Some(&11.));
    assert_eq!(parameters.last(), Some(&1_000_011.));
    assert!((points[0][0] - 10. * 1f64.cos()).abs() < 1e-10);
    assert!((points[0][1] - 10. * 1f64.sin()).abs() < 1e-10);
    assert!((points.last().unwrap()[0] - points[0][0]).abs() < 1e-10);
}
#[test]
fn mapped_circle_extreme_domains_preserve_native_samples_or_disclose_unrepresentable_bindings() {
    for domain in [[0., 1e308], [1., 1. + f64::EPSILON]] {
        let mut doc = Document::new("native parameter precision");
        let curve = Curve::Circle {
            center: [0.; 3],
            x: [1., 0., 0.],
            y: [0., 1., 0.],
            radius: 10.,
            domain,
            angle_domain: Some([1., 1. + std::f64::consts::TAU]),
        };
        doc.validate(&Limits::default()).unwrap();
        let entity = Entity::new("mapped circle", Geometry::Curve(curve.clone()), None);
        let id = entity.id;
        doc.entities.insert(id, entity);
        let scene = Scene::capture(std::sync::Arc::new(doc), 1, &Limits::default()).unwrap();
        if domain[0] == 1. {
            assert!(
                scene.objects.is_empty(),
                "collapsed circle cannot expose false curve bindings"
            );
            assert!(
                scene
                    .diagnostics
                    .iter()
                    .any(|s| s.contains("native parameter precision"))
            );
        } else {
            assert!(scene.diagnostics.is_empty());
            let object = scene.objects.iter().find(|o| o.root == id).unwrap();
            let RenderBuffers::Curve { points, parameters } = object.buffers.as_ref() else {
                panic!("curve buffers")
            };
            assert!(parameters.windows(2).all(|p| p[0] < p[1]));
            assert_eq!(parameters.last(), Some(&1e308));
            for (point, parameter) in points.iter().zip(parameters) {
                let exact = om9_core::evaluation::evaluate_curve(
                    &curve,
                    om9_core::evaluation::Parameter::Native(*parameter),
                    0,
                    Transform::identity(),
                    &Limits::default(),
                )
                .unwrap()[0];
                assert_eq!(*point, exact);
            }
            let exact = om9_core::evaluation::evaluate_curve(
                &curve,
                om9_core::evaluation::Parameter::Native(9e307),
                0,
                Transform::identity(),
                &Limits::default(),
            )
            .unwrap()[0];
            let screen = camera().project(exact, viewport()).unwrap().unwrap().screen;
            let hit = scene
                .pick(screen, &camera(), viewport(), 6., PickIntent::Select)
                .unwrap()
                .unwrap();
            let Subelement::CurveParameter(parameter) = hit.subset else {
                panic!("native circle parameter")
            };
            assert!((parameter / 1e308 - 0.9).abs() < 1e-7);
        }
    }
}
#[test]
fn rendered_curve_picking_resolves_source_parameter_and_never_creates_mesh_master() {
    let (c, d, id) = fixture();
    let scene = Scene::capture(
        c.world().documents[&d].clone(),
        c.epoch(),
        &Limits::default(),
    )
    .unwrap();
    let hit = scene
        .pick([250., 200.], &camera(), viewport(), 6., PickIntent::Select)
        .unwrap()
        .unwrap();
    assert_eq!(hit.root, id);
    assert_eq!(hit.world_point, [5., 0., 0.]);
    assert_eq!(hit.subset, Subelement::CurveParameter(5.));
    assert_eq!(
        c.world().documents[&d].entities[&id].geometry.kind(),
        om9_core::geometry::GeometryKind::Curve
    );
    assert!(
        scene
            .pick([200., 100.], &camera(), viewport(), 6., PickIntent::Select)
            .unwrap()
            .is_none(),
        "grid lines are not scene objects"
    );
}
#[test]
fn nested_mirrored_instance_pick_keeps_root_member_path_and_single_world_transform() {
    let mut d = Document::new("instances");
    let member = Entity::new("point", Geometry::Point([2., 3., 0.]), d.current_layer);
    let member_id = member.id;
    let def = Definition::new("shared", vec![member]);
    let def_id = def.id;
    d.definitions.insert(def_id, def);
    let mut root = Entity::new("instance", Geometry::Instance(def_id), d.current_layer);
    root.placement = Transform::translation([10., 0., 0.])
        .compose(Transform::scale([-2., 3., 1.]).unwrap())
        .unwrap();
    let id = root.id;
    d.entities.insert(id, root);
    let did = d.id;
    let c = TransactionCoordinator::new(
        World::new(vec![d], &Limits::default()).unwrap(),
        Limits::default(),
    )
    .unwrap();
    let scene = Scene::capture(
        c.world().documents[&did].clone(),
        c.epoch(),
        &Limits::default(),
    )
    .unwrap();
    let hit = scene
        .pick([260., 110.], &camera(), viewport(), 6., PickIntent::Select)
        .unwrap()
        .unwrap();
    assert_eq!(hit.root, id);
    assert_eq!(hit.path, vec![member_id]);
    assert_eq!(hit.world_point, [6., 9., 0.]);
}
#[test]
fn hidden_locked_and_unselectable_stay_distinct_for_reference_picking() {
    let (c, d, id) = fixture();
    let mut doc = c.world().documents[&d].as_ref().clone();
    doc.entities.get_mut(&id).unwrap().attributes.locked = true;
    let scene = Scene::capture(
        std::sync::Arc::new(doc.clone()),
        c.epoch(),
        &Limits::default(),
    )
    .unwrap();
    assert!(
        scene
            .pick([250., 200.], &camera(), viewport(), 6., PickIntent::Select)
            .unwrap()
            .is_none()
    );
    assert!(
        scene
            .pick(
                [250., 200.],
                &camera(),
                viewport(),
                6.,
                PickIntent::Reference { snap_locked: true }
            )
            .unwrap()
            .is_some()
    );
    assert!(
        scene
            .pick(
                [250., 200.],
                &camera(),
                viewport(),
                6.,
                PickIntent::Reference { snap_locked: false }
            )
            .unwrap()
            .is_none()
    );
    doc.entities.get_mut(&id).unwrap().attributes.locked = false;
    doc.entities.get_mut(&id).unwrap().attributes.selectable = false;
    let scene = Scene::capture(
        std::sync::Arc::new(doc.clone()),
        c.epoch(),
        &Limits::default(),
    )
    .unwrap();
    assert!(
        scene
            .pick([250., 200.], &camera(), viewport(), 6., PickIntent::Select)
            .unwrap()
            .is_none()
    );
    assert!(
        scene
            .pick(
                [250., 200.],
                &camera(),
                viewport(),
                6.,
                PickIntent::Reference { snap_locked: true }
            )
            .unwrap()
            .is_some()
    );
    doc.entities.get_mut(&id).unwrap().attributes.visible = false;
    let scene = Scene::capture(std::sync::Arc::new(doc), c.epoch(), &Limits::default()).unwrap();
    assert!(
        scene
            .pick(
                [250., 200.],
                &camera(),
                viewport(),
                6.,
                PickIntent::Reference { snap_locked: true }
            )
            .unwrap()
            .is_none()
    );
}
#[test]
fn stale_camera_or_source_invalidates_held_pick_before_dispatch() {
    let (mut c, d, id) = fixture();
    let mut camera = camera();
    let scene = Scene::capture(
        c.world().documents[&d].clone(),
        c.epoch(),
        &Limits::default(),
    )
    .unwrap();
    let hit = scene
        .pick([250., 200.], &camera, viewport(), 6., PickIntent::Select)
        .unwrap()
        .unwrap();
    camera.pan([10., 0.]).unwrap();
    assert!(hit.revalidate(&c, &camera).is_err());
    let source = c.world().documents[&d].stamp(id).unwrap();
    let p = c
        .prepare(TransactionRequest::new(
            "move",
            vec![Mutation::Placement {
                source,
                placement: Transform::translation([10., 0., 0.]),
            }],
        ))
        .unwrap();
    c.commit(p).unwrap();
    assert!(hit.revalidate(&c, &camera).is_err());
}
#[test]
fn mesh_triangle_pick_maps_to_original_polygon_face() {
    let mut d = Document::new("mesh");
    let e = Entity::new(
        "quad",
        Geometry::Mesh(Mesh {
            vertices: vec![[0., 0., 0.], [10., 0., 0.], [10., 10., 0.], [0., 10., 0.]],
            faces: vec![vec![0, 1, 2, 3]],
            normals: vec![],
            colors: vec![],
        }),
        d.current_layer,
    );
    let id = e.id;
    d.entities.insert(id, e);
    let scene = Scene::capture(std::sync::Arc::new(d), 1, &Limits::default()).unwrap();
    let hit = scene
        .pick([225., 125.], &camera(), viewport(), 6., PickIntent::Select)
        .unwrap()
        .unwrap();
    assert_eq!(hit.root, id);
    assert_eq!(hit.subset, Subelement::MeshFace(0));
    assert_eq!(hit.world_point, [2.5, 7.5, 0.]);
}
