// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    budget::Limits,
    geometry::{Geometry, Transform},
    model::{Definition, Document, Entity, Layer},
};
use om9_scene::scene::Scene;
use std::sync::Arc;
#[test]
fn metadata_refresh_reuses_buffers_and_updates_instance_masks_color_and_stamps() {
    let limits = Limits::default();
    let mut d = Document::new("metadata buffers");
    let layer = Layer::new("generic", [1, 2, 3, 255]);
    let lid = layer.id;
    d.layers.insert(lid, layer);
    d.layer_order.push(lid);
    let mut member = Entity::new("member", Geometry::Point([1., 2., 3.]), Some(lid));
    member.attributes.color = None;
    member.placement = Transform::translation([2., 0., 0.]);
    let definition = Definition::new("definition", vec![member]);
    let mut root = Entity::new("root", Geometry::Instance(definition.id), None);
    root.placement = Transform::translation([3., 0., 0.]);
    d.definitions.insert(definition.id, definition);
    let eid = root.id;
    d.entities.insert(eid, root);
    let old = Scene::capture(Arc::new(d.clone()), 1, &limits).unwrap();
    d.layers.get_mut(&lid).unwrap().locked = true;
    d.layers.get_mut(&lid).unwrap().color = [8, 9, 10, 255];
    d.entities.get_mut(&eid).unwrap().attributes_revision += 1;
    let new = old.update_owned(Arc::new(d), 2, None, &limits).unwrap();
    assert!(Arc::ptr_eq(
        &old.objects[0].buffers,
        &new.objects[0].buffers
    ));
    assert_eq!(new.objects[0].transform, old.objects[0].transform);
    assert_eq!(new.objects[0].color, [8, 9, 10, 255]);
    assert!(new.objects[0].visible);
    assert!(!new.objects[0].selectable);
    assert_eq!(new.objects[0].source.attributes_revision, 2);
}
#[test]
fn cache_refresh_refuses_changed_geometry_owner_and_unreserved_metadata_clone() {
    let limits = Limits::default();
    let mut d = Document::new("cache proof");
    let root = Entity::new("root", Geometry::Point([0.; 3]), None);
    let id = root.id;
    d.entities.insert(id, root);
    let scene = Scene::capture(Arc::new(d.clone()), 1, &limits).unwrap();
    let tiny = Limits {
        max_bytes: scene.update_usage(None).unwrap().bytes - 1,
        ..limits
    };
    assert!(
        scene
            .update_owned(Arc::new(d.clone()), 2, None, &tiny)
            .is_err()
    );
    d.entities.get_mut(&id).unwrap().geometry = Geometry::Point([1.; 3]);
    assert!(scene.update_owned(Arc::new(d), 2, None, &limits).is_err());
    let mut foreign = scene.document.as_ref().clone();
    foreign.renew_session(&limits).unwrap();
    assert!(
        scene
            .update_owned(Arc::new(foreign), 2, None, &limits)
            .is_err()
    );
}

#[test]
fn mesh_and_surface_update_estimates_match_published_buffers() {
    use om9_core::geometry::Mesh;
    use om9_scene::scene::RenderBuffers;
    let limits = Limits::default();
    let document = Document::new("bounded additions");
    let scene = Scene::capture(Arc::new(document.clone()), 1, &limits).unwrap();
    let surface = om9_core::surface_kernel::plane([0., 0., 0.], [1., 0., 0.], [0., 1., 0.], 2., 2., &limits).unwrap();
    let mesh = Mesh { vertices: vec![[0.,0.,0.], [1.,0.,0.], [1.,1.,0.], [0.,1.,0.]],
        faces: vec![vec![0,1,2,3], vec![0,2,3]], normals: vec![], colors: vec![], corner_uvs: vec![] };
    for (geometry, expected_vertices, expected_triangles, expected_edges) in [
        (Geometry::Mesh(mesh), 4, 3, 7),
        (Geometry::Surface(surface), 144, 242, 44),
    ] {
        let entity = Entity::new("added", geometry, None);
        let id = entity.id;
        let estimated = scene.update_usage(Some(&entity)).unwrap();
        assert_eq!(estimated.points, expected_vertices as u64);
        let mut updated = document.clone();
        updated.entities.insert(id, entity);
        let too_small = Limits { max_bytes: estimated.bytes - 1, ..limits };
        assert!(scene.update_owned(Arc::new(updated.clone()), 2, Some(id), &too_small).is_err());
        assert!(scene.objects.is_empty(), "refusal must leave the original scene unchanged");
        let result = scene.update_owned(Arc::new(updated), 2, Some(id), &limits).unwrap();
        let RenderBuffers::Mesh { vertices, triangles, edges } = result.objects[0].buffers.as_ref() else { panic!("mesh buffers") };
        assert_eq!((vertices.len(), triangles.len(), edges.len()), (expected_vertices, expected_triangles, expected_edges));
    }
}

#[test]
fn incremental_retained_bytes_exclude_only_reused_buffers() {
    use om9_scene::scene::RenderBuffers;
    let limits = Limits::default();
    let mut document = Document::new("shared old buffer");
    let first = Entity::new("first", Geometry::Point([0., 0., 0.]), None);
    document.entities.insert(first.id, first);
    let old = Scene::capture(Arc::new(document.clone()), 1, &limits).unwrap();
    let RenderBuffers::Points { points, native_indices } = old.objects[0].buffers.as_ref() else { panic!("point buffers") };
    let shared_bytes = std::mem::size_of::<RenderBuffers>() + 16
        + points.capacity() * std::mem::size_of::<[f64; 3]>()
        + native_indices.capacity() * std::mem::size_of::<u32>();
    let added = Entity::new("second", Geometry::Point([1., 2., 3.]), None);
    let id = added.id;
    document.entities.insert(id, added);
    let (new, retained_bytes) = old.update_owned_with_retained_bytes(Arc::new(document), 2, Some(id), &limits).unwrap();
    assert!(new.objects.iter().any(|object| Arc::ptr_eq(&object.buffers, &old.objects[0].buffers)));
    assert_eq!(retained_bytes + shared_bytes as u64, new.cache_usage().unwrap().bytes);
}
