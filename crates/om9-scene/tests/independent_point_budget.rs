// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{budget::Limits, geometry::{Geometry, Mesh}, model::{Document, Entity}, Error};
use om9_scene::scene::Scene;
use std::sync::Arc;

fn triangle() -> Document {
    let mut document = Document::new("independent scene points");
    let mesh = Mesh { vertices: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
        faces: vec![vec![0, 1, 2]], normals: vec![], colors: vec![], corner_uvs: vec![] };
    let entity = Entity::new("triangle", Geometry::Mesh(mesh), None);
    document.entities.insert(entity.id, entity);
    document
}

#[test]
fn batched_scene_updates_check_accumulated_display_points_before_publication() {
    let limits = Limits { max_points: 5, ..Limits::default() };
    let scene = Scene::capture(Arc::new(triangle()), 1, &limits).unwrap();
    let point = Entity::new("extra", Geometry::Point([2., 2., 2.]), None);
    let mut total = scene.cache_usage().unwrap().points;
    assert!(matches!(scene.update_held_usage_counted(Some(&point), &mut 0), Err(Error::Invalid("scene point baseline"))));
    let first = scene.update_held_usage_counted(Some(&point), &mut total).unwrap();
    let second = scene.update_held_usage_counted(Some(&point), &mut total).unwrap();
    assert_eq!((total, first.points, second.points), (5, 0, 0));
    assert!(matches!(scene.update_held_usage_counted(Some(&point), &mut total), Err(Error::Budget("points"))));
    assert_eq!(total, 5);
}

#[test]
fn source_and_display_have_independent_point_limits_but_share_byte_budget() {
    let document = triangle();
    let limits = Limits { max_points: 3, ..Limits::default() };
    let source = document.validate(&limits).unwrap();
    assert_eq!(source.points, 3);
    let scene = Scene::capture(Arc::new(document.clone()), 1, &limits).unwrap();
    assert_eq!(scene.cache_usage().unwrap().points, 3);
    let cache = scene.cache_usage().unwrap();
    let cache_held = scene.cache_held_usage().unwrap();
    assert_eq!(cache_held.points, 0);
    assert_eq!(cache_held.bytes, cache.bytes);
    let point = Entity::new("extra", Geometry::Point([2., 2., 2.]), None);
    assert!(matches!(scene.update_held_usage(Some(&point)), Err(Error::Budget("points"))),
        "the independent scene point cap must still reject an oversized display");
    let held = scene.update_held_usage(None).unwrap();
    assert_eq!(held.points, 0);
    assert!(held.bytes > 0, "scene preparation bytes must remain charged");
    assert!(matches!(Scene::capture(Arc::new(document.clone()), 1, &Limits { max_points: 2, ..limits }), Err(Error::Budget("points"))));
    let tight = Limits { max_bytes: source.bytes + 8, ..limits };
    assert!(matches!(Scene::capture(Arc::new(document), 1, &tight), Err(Error::Budget("bytes"))));
}
