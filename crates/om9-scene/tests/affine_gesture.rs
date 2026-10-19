// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    budget::Limits,
    geometry::{Geometry, Transform},
    model::{Document, Entity},
    transaction::*,
};
use om9_scene::{affine::*, camera::*, scene::*};
fn fixture() -> (
    TransactionCoordinator,
    om9_core::identity::DocumentId,
    om9_core::identity::ObjectId,
) {
    let mut d = Document::new("gesture");
    let e = Entity::new("point", Geometry::Point([1., 2., 3.]), d.current_layer);
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
#[test]
fn warm_affine_updates_reuse_buffers_and_cancel_keeps_geometry_and_undo() {
    let (c, d, id) = fixture();
    let mut scene = Scene::capture(
        c.world().documents[&d].clone(),
        c.epoch(),
        &Limits::default(),
    )
    .unwrap();
    let buffers = scene.objects[0].buffers.clone();
    let before = c.snapshot();
    let mut gesture = AffineGesture::begin(&c, d, &[id]).unwrap();
    for x in 0..100 {
        gesture
            .update(&c, &mut scene, Transform::translation([x as f64, 0., 0.]))
            .unwrap();
        assert!(std::sync::Arc::ptr_eq(&buffers, &scene.objects[0].buffers));
    }
    assert_eq!(c.snapshot(), before);
    gesture.cancel(&mut scene).unwrap();
    assert_eq!(
        scene.objects[0].display_transform().unwrap(),
        Transform::identity()
    );
    assert_eq!(c.snapshot(), before);
}
#[test]
fn commit_updates_exact_placement_in_one_undo_and_redo() {
    let (mut c, d, id) = fixture();
    let mut scene = Scene::capture(
        c.world().documents[&d].clone(),
        c.epoch(),
        &Limits::default(),
    )
    .unwrap();
    let mut gesture = AffineGesture::begin(&c, d, &[id]).unwrap();
    gesture
        .update(&c, &mut scene, Transform::translation([10., 0., 0.]))
        .unwrap();
    let p = gesture.prepare(&c).unwrap();
    c.commit(p).unwrap();
    assert_eq!(c.undo_len(), 1);
    assert_eq!(
        c.world().documents[&d].entities[&id]
            .placement
            .point([1., 2., 3.])
            .unwrap(),
        [11., 2., 3.]
    );
    c.undo().unwrap();
    assert_eq!(
        c.world().documents[&d].entities[&id].placement,
        Transform::identity()
    );
    c.redo().unwrap();
    assert_eq!(
        c.world().documents[&d].entities[&id]
            .placement
            .point([1., 2., 3.])
            .unwrap(),
        [11., 2., 3.]
    );
}
#[test]
fn preview_and_pick_proxy_use_the_same_gesture_matrix() {
    let (c, d, id) = fixture();
    let mut scene = Scene::capture(
        c.world().documents[&d].clone(),
        c.epoch(),
        &Limits::default(),
    )
    .unwrap();
    let mut gesture = AffineGesture::begin(&c, d, &[id]).unwrap();
    gesture
        .update(&c, &mut scene, Transform::translation([10., 0., 0.]))
        .unwrap();
    let camera = Camera::orthographic([0., 0., 100.], [0., 0., 0.], [0., 1., 0.], 40.).unwrap();
    let hit = scene
        .pick(
            [310., 180.],
            &camera,
            Viewport {
                origin: [0., 0.],
                size: [400., 400.],
            },
            6.,
            PickIntent::Select,
        )
        .unwrap()
        .unwrap();
    assert_eq!(hit.root, id);
    assert_eq!(hit.world_point, [11., 2., 3.]);
}
#[test]
fn stale_gesture_is_rejected_without_touching_public_redo() {
    let (mut c, d, id) = fixture();
    let mut scene = Scene::capture(
        c.world().documents[&d].clone(),
        c.epoch(),
        &Limits::default(),
    )
    .unwrap();
    let mut gesture = AffineGesture::begin(&c, d, &[id]).unwrap();
    gesture
        .update(&c, &mut scene, Transform::translation([10., 0., 0.]))
        .unwrap();
    let source = c.world().documents[&d].stamp(id).unwrap();
    let p = c
        .prepare(TransactionRequest::new(
            "other",
            vec![Mutation::Placement {
                source,
                placement: Transform::translation([20., 0., 0.]),
            }],
        ))
        .unwrap();
    c.commit(p).unwrap();
    c.undo().unwrap();
    let before = c.snapshot();
    assert!(gesture.prepare(&c).is_err());
    assert_eq!(c.snapshot(), before);
    c.redo().unwrap();
}

#[test]
fn unchanged_gesture_accepts_no_document_write_or_undo_and_keeps_redo() {
    let (mut c, d, id) = fixture();
    let source = c.world().documents[&d].stamp(id).unwrap();
    let p = c
        .prepare(TransactionRequest::new(
            "seed",
            vec![Mutation::Placement {
                source,
                placement: Transform::translation([10., 0., 0.]),
            }],
        ))
        .unwrap();
    c.commit(p).unwrap();
    c.undo().unwrap();
    let mut scene = Scene::capture(
        c.world().documents[&d].clone(),
        c.epoch(),
        &Limits::default(),
    )
    .unwrap();
    let mut gesture = AffineGesture::begin(&c, d, &[id]).unwrap();
    gesture
        .update(&c, &mut scene, Transform::identity())
        .unwrap();
    let before = c.snapshot();
    let p = gesture.prepare(&c).unwrap();
    let receipt = c.commit(p).unwrap();
    assert!(!receipt.published);
    assert_eq!(c.snapshot(), before);
    c.redo().unwrap();
}
