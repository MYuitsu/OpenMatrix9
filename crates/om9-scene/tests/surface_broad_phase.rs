// SPDX-License-Identifier: LGPL-2.1-or-later
use std::sync::Arc;
use om9_core::{
    budget::Limits,
    geometry::{Geometry, Transform},
    model::{Definition, Document, Entity},
};
use om9_scene::{camera::{Camera, Viewport}, osnap::{self, SnapKind, SnapSettings}, scene::Scene};

#[test]
fn visible_small_surface_triangles_remain_snap_candidates() {
    let limits = Limits::default();
    let surface = om9_core::surface_kernel::plane([0., 0., 0.], [1., 0., 0.], [0., 1., 0.], 2., 2., &limits).unwrap();
    for scale in [1e-4_f64, 1e-7] {
        let mut document = Document::new("small displayed surface");
        let mut entity = Entity::new("plane", Geometry::Surface(surface.clone()), None);
        entity.placement = Transform::scale([scale, scale, 1.]).unwrap();
        let id = entity.id;
        document.entities.insert(id, entity);
        let scene = Scene::capture(Arc::new(document), 0, &limits).unwrap();
        let viewport = Viewport { origin: [0., 0.], size: [600., 600.] };
        for perspective in [false, true] {
            let camera = if perspective {
                let mut camera = Camera::perspective([scale, scale, 5. * scale], [scale, scale, 0.], [0., 1., 0.], 0.9).unwrap();
                camera.near_mm = scale * 0.01;
                camera
            } else {
                Camera::orthographic([scale, scale, 5.], [scale, scale, 0.], [0., 1., 0.], scale * 4.).unwrap()
            };
            let point = [scale * 0.7, scale * 1.2, 0.];
            let screen = camera.project(point, viewport).unwrap().unwrap().screen;
            let ray = camera.visible_ray(screen, viewport).unwrap();
            assert!(scene.spatial_index().unwrap().ray_candidates(ray.origin, ray.direction).contains(&0));
            let hits = osnap::candidates(&scene, &camera, viewport, screen, SnapSettings::default(), None).unwrap();
            let hit = hits.iter().find(|candidate| candidate.kind == SnapKind::Surface && candidate.root == Some(id))
                .unwrap_or_else(|| panic!("missing surface candidate at scale {scale}, perspective={perspective}"));
            assert!(hit.world.iter().zip(point).all(|(a, b)| (a - b).abs() < scale * 1e-6));
        }
    }
}

#[test]
fn visible_small_mesh_blocks_surface_behind_it() {
    let limits = Limits::default();
    let scale = 1e-7;
    let mut document = Document::new("small displayed occluder");
    let mut entity = Entity::new("mesh", Geometry::Mesh(om9_core::geometry::Mesh {
        vertices: vec![[0., 0., 0.], [2., 0., 0.], [0., 2., 0.]],
        faces: vec![vec![0, 1, 2]], normals: vec![], colors: vec![], corner_uvs: vec![],
    }), None);
    entity.placement = Transform::scale([scale, scale, 1.]).unwrap();
    document.entities.insert(entity.id, entity);
    let scene = Scene::capture(Arc::new(document), 0, &limits).unwrap();
    let viewport = Viewport { origin: [0., 0.], size: [600., 600.] };
    let camera = Camera::orthographic([scale, scale, 5.], [scale, scale, 0.], [0., 1., 0.], scale * 4.).unwrap();
    let cursor = camera.project([0.4 * scale, 0.4 * scale, 0.], viewport).unwrap().unwrap().screen;
    assert!(osnap::visible_display_occludes(&scene, &camera, viewport, cursor, [0.4 * scale, 0.4 * scale, -1.], 0., &[], None).unwrap());
    assert!(!osnap::visible_display_occludes(&scene, &camera, viewport, cursor, [0.4 * scale, 0.4 * scale, 1.], 0., &[], None).unwrap());
}

#[test]
fn native_brep_display_blocks_surface_behind_even_if_not_a_snap_reference() {
    use om9_core::geometry::{Mesh, NativeBrep};
    let limits = Limits::default();
    let native = NativeBrep {
        display: Mesh {
            vertices: vec![[-2., -2., 0.], [2., -2., 0.], [2., 2., 0.], [-2., 2., 0.]],
            faces: vec![vec![0, 1, 2, 3]], normals: vec![], colors: vec![], corner_uvs: vec![],
        },
        payload: vec![1].into(), crc: 0,
    };
    let mut document = Document::new("native brep display occlusion");
    let entity = Entity::new("occluder", Geometry::NativeBrep(native), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut scene = Scene::capture(Arc::new(document), 0, &limits).unwrap();
    let object = scene.objects.iter_mut().find(|object| object.root == id).unwrap();
    object.selectable = false;
    object.reference_snappable = false;
    let viewport = Viewport { origin: [0., 0.], size: [600., 600.] };
    let camera = Camera::orthographic([0., 0., 10.], [0., 0., 0.], [0., 1., 0.], 10.).unwrap();
    let cursor = camera.project([0.25, 0.25, 0.], viewport).unwrap().unwrap().screen;
    assert!(osnap::visible_display_occludes(&scene, &camera, viewport, cursor, [0.25, 0.25, -1.], 0., &[], None).unwrap());
    assert!(!osnap::visible_display_occludes(&scene, &camera, viewport, cursor, [0.25, 0.25, 1.], 0., &[], None).unwrap());
    assert!(!osnap::visible_display_occludes(&scene, &camera, viewport, cursor, [0.25, 0.25, -1.], 0., &[id], None).unwrap());
    scene.objects[0].visible = false;
    assert!(!osnap::visible_display_occludes(&scene, &camera, viewport, cursor, [0.25, 0.25, -1.], 0., &[], None).unwrap());
}

#[test]
fn indexed_display_occlusion_matches_separated_analytic_quads_and_preview() {
    use om9_core::geometry::Mesh;
    let limits = Limits::default();
    let mut document = Document::new("occlusion broad phase");
    let mut objects = Vec::new();
    for i in 0..80 {
        let x = (i % 10) as f64 * 5. - 25.;
        let y = (i / 10) as f64 * 5. - 20.;
        let mut entity = Entity::new("quad", Geometry::Mesh(Mesh {
            vertices: vec![[0., 0., 0.], [2., 0., 0.], [2., 2., 0.], [0., 2., 0.]],
            faces: vec![vec![0, 1, 2, 3]], normals: vec![], colors: vec![], corner_uvs: vec![],
        }), None);
        entity.placement = Transform::translation([x, y, 0.]);
        objects.push((entity.id, x, y));
        document.entities.insert(entity.id, entity);
    }
    let mut scene = Scene::capture(Arc::new(document), 0, &limits).unwrap();
    let viewport = Viewport { origin: [0., 0.], size: [800., 800.] };
    for preview in [false, true] {
        if preview {
            scene.objects.iter_mut().find(|object| object.root == objects[0].0).unwrap().preview = Transform::translation([0., 1., 0.]);
            scene.invalidate_picking_bounds();
        }
        for perspective in [false, true] {
            let camera = if perspective { Camera::perspective([0., 0., 80.], [0., 0., 0.], [0., 1., 0.], 0.9).unwrap() }
                else { Camera::orthographic([0., 0., 80.], [0., 0., 0.], [0., 1., 0.], 65.).unwrap() };
            for (i, (id, x, y)) in objects.iter().enumerate() {
                let point = [x + 1., y + 1. + if preview && i == 0 { 1. } else { 0. }, 0.];
                let cursor = camera.project(point, viewport).unwrap().unwrap().screen;
                let ray = camera.ray(cursor, viewport).unwrap();
                let behind = [point[0] + ray.direction[0], point[1] + ray.direction[1], point[2] + ray.direction[2]];
                let front = [point[0] - ray.direction[0], point[1] - ray.direction[1], point[2] - ray.direction[2]];
                assert!(osnap::visible_display_occludes(&scene, &camera, viewport, cursor, behind, 0., &[], None).unwrap(), "missing {i}, perspective={perspective}, preview={preview}");
                assert!(!osnap::visible_display_occludes(&scene, &camera, viewport, cursor, front, 0., &[], None).unwrap());
                assert!(!osnap::visible_display_occludes(&scene, &camera, viewport, cursor, behind, 0., &[*id], None).unwrap());
            }
        }
    }
}

#[test]
fn indexed_surface_candidates_match_analytic_planes_across_instances_and_preview() {
    let limits = Limits::default();
    let surface = om9_core::surface_kernel::plane([0., 0., 0.], [1., 0., 0.], [0., 1., 0.], 2., 2., &limits).unwrap();
    let definition = Definition::new("shared surface", vec![Entity::new("plane", Geometry::Surface(surface), None)]);
    let definition_id = definition.id;
    let mut document = Document::new("surface index oracle");
    document.definitions.insert(definition_id, definition);
    let mut rectangles = Vec::new();
    for i in 0..60 {
        let x = (i % 10) as f64 * 5. - 25.;
        let y = (i / 10) as f64 * 5. - 15.;
        let z = (i % 3) as f64;
        let sx = if i % 2 == 0 { 1.5 } else { -1.5 };
        let mut instance = Entity::new("instance", Geometry::Instance(definition_id), None);
        instance.placement = Transform::translation([x, y, z]).compose(Transform::scale([sx, 0.75, 1.]).unwrap()).unwrap();
        let id = instance.id;
        document.entities.insert(id, instance);
        rectangles.push((id, x, y, z, sx));
    }
    let mut scene = Scene::capture(Arc::new(document), 0, &limits).unwrap();
    let viewport = Viewport { origin: [10., 20.], size: [800., 600.] };
    for preview in [false, true] {
        if preview {
            let object = scene.objects.iter_mut().find(|object| object.root == rectangles[0].0).unwrap();
            object.preview = Transform::translation([0., 20., 0.]);
            scene.invalidate_picking_bounds();
        }
        for perspective in [false, true] {
            let camera = if perspective {
                Camera::perspective([0., 0., 80.], [0., 0., 0.], [0., 1., 0.], 0.9).unwrap()
            } else { Camera::orthographic([0., 0., 80.], [0., 0., 0.], [0., 1., 0.], 60.).unwrap() };
            for i in 0..180 {
                let screen = if i < rectangles.len() {
                    let (_, x, y, z, sx) = rectangles[i];
                    let offset = if preview && i == 0 { 20. } else { 0. };
                    camera.project([x + sx, y + 0.75 + offset, z], viewport).unwrap().unwrap().screen
                } else {
                    [viewport.origin[0] + 41.371 + ((i * 83) % 715) as f64,
                        viewport.origin[1] + 39.219 + ((i * 67) % 520) as f64]
                };
                let ray = camera.visible_ray(screen, viewport).unwrap();
                let mut expected = Vec::new();
                for (index, (id, x, y, z, sx)) in rectangles.iter().enumerate() {
                    let t = (*z - ray.origin[2]) / ray.direction[2];
                    let px = ray.origin[0] + t * ray.direction[0];
                    let py = ray.origin[1] + t * ray.direction[1];
                    let y = y + if preview && index == 0 { 20. } else { 0. };
                    if t >= 0. && px > x.min(x + 2. * sx) && px < x.max(x + 2. * sx)
                        && py > y && py < y + 1.5 {
                        expected.push(*id);
                    }
                }
                let mut actual: Vec<_> = osnap::candidates(&scene, &camera, viewport, screen, SnapSettings::default(), None).unwrap()
                    .into_iter().filter(|candidate| candidate.kind == SnapKind::Surface).map(|candidate| candidate.root.unwrap()).collect();
                expected.sort(); actual.sort();
                if actual != expected {
                    for id in &expected {
                        if !actual.contains(id) {
                            let index = scene.objects.iter().position(|object| object.root == *id).unwrap();
                            let indexed = scene.spatial_index().unwrap().ray_candidates(ray.origin, ray.direction).contains(&index);
                            eprintln!("missing object={:?} rectangle={:?} indexed={indexed} ray={ray:?}", id, rectangles.iter().find(|row| row.0 == *id));
                        }
                    }
                }
                assert_eq!(actual, expected, "preview={preview}, perspective={perspective}, screen={screen:?}");
            }
        }
    }
}
