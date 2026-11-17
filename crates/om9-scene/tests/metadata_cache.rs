// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    budget::Limits,
    geometry::{Geometry, Transform},
    model::{Definition, Document, Entity, Layer},
};
use om9_scene::scene::Scene;
use std::sync::Arc;

#[test]
fn trimmed_surface_scene_respects_rectangular_boundary() {
    use om9_core::rhino_semantics::{set_trimmed_surface, TrimmedSurface};
    use om9_scene::scene::RenderBuffers;

    let limits = Limits::default();
    let surface = om9_core::surface_kernel::plane(
        [0., 0., 0.], [1., 0., 0.], [0., 1., 0.], 10., 10., &limits,
    ).unwrap();
    let trim = TrimmedSurface {
        underlying: surface.clone(),
        outer: vec![[0.2, 0.2], [0.8, 0.2], [0.8, 0.8], [0.2, 0.8]],
        inners: vec![],
        reversed: false,
    };
    let mut entity = Entity::new("trimmed", Geometry::Surface(surface), None);
    set_trimmed_surface(&mut entity, &trim, &limits).unwrap();
    let mut document = Document::new("trim display");
    document.entities.insert(entity.id, entity);
    let scene = Scene::capture(Arc::new(document), 1, &limits).unwrap();
    let RenderBuffers::Mesh { vertices, .. } = scene.objects[0].buffers.as_ref() else {
        panic!("expected surface display mesh");
    };
    assert!(vertices.iter().all(|p| (2. - 1e-9..=8. + 1e-9).contains(&p[0])
        && (2. - 1e-9..=8. + 1e-9).contains(&p[1])), "display extends beyond trimmed surface");
}

#[test]
fn trim_metadata_change_refuses_reusing_surface_buffers() {
    use om9_core::rhino_semantics::{set_trimmed_surface, TrimmedSurface};
    let limits = Limits::default();
    let surface = om9_core::surface_kernel::plane(
        [0., 0., 0.], [1., 0., 0.], [0., 1., 0.], 10., 10., &limits,
    ).unwrap();
    let mut entity = Entity::new("surface", Geometry::Surface(surface.clone()), None);
    let id = entity.id;
    let mut document = Document::new("trim change");
    document.entities.insert(id, entity.clone());
    let original = Scene::capture(Arc::new(document.clone()), 1, &limits).unwrap();
    let trim = TrimmedSurface {
        underlying: surface,
        outer: vec![[0.2, 0.2], [0.8, 0.2], [0.8, 0.8], [0.2, 0.8]],
        inners: vec![], reversed: false,
    };
    set_trimmed_surface(&mut entity, &trim, &limits).unwrap();
    document.entities.insert(id, entity);
    assert!(original.update_owned(Arc::new(document.clone()), 2, None, &limits).is_err());
    let trimmed = Scene::capture(Arc::new(document.clone()), 2, &limits).unwrap();
    document.entities.get_mut(&id).unwrap().extensions.remove(om9_core::model::TRIM_SURFACE_EXTENSION_SLOT);
    assert!(trimmed.update_owned(Arc::new(document), 3, None, &limits).is_err());
}

#[test]
fn trimmed_surface_incremental_addition_matches_full_capture_and_admission() {
    use om9_core::rhino_semantics::{set_trimmed_surface,TrimmedSurface};
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    let trim=TrimmedSurface {underlying:surface.clone(),outer:vec![[0.1,0.1],[0.9,0.1],[0.9,0.9],[0.1,0.9]],
        inners:vec![vec![[0.4,0.4],[0.6,0.4],[0.6,0.6],[0.4,0.6]]],reversed:false};
    let mut entity=Entity::new("trim",Geometry::Surface(surface),None);
    set_trimmed_surface(&mut entity,&trim,&limits).unwrap();let id=entity.id;
    let mut document=Document::new("trim update");
    let initial=Scene::capture(Arc::new(document.clone()),1,&limits).unwrap();
    let bound=initial.update_usage(Some(&entity)).unwrap();
    document.entities.insert(id,entity);
    let peak=Scene::candidate_scene_peak(&document,&limits).unwrap();
    let full=Scene::capture(Arc::new(document.clone()),2,&limits).unwrap();
    let updated=initial.update_owned(Arc::new(document.clone()),2,Some(id),&limits).unwrap();
    assert!(peak>=full.cache_usage().unwrap().bytes);
    assert!(bound.bytes>=updated.cache_usage().unwrap().bytes-initial.cache_usage().unwrap().bytes);
    assert_eq!(full.objects[0].buffers.as_ref(),updated.objects[0].buffers.as_ref());
    assert!(!full.objects[0].buffers.is_trim_display_fallback());
    assert!(!updated.objects[0].buffers.is_trim_display_fallback());
}

#[test]
fn larger_trim_surface_incremental_update_matches_full_capture() {
    use om9_core::rhino_semantics::{set_trimmed_surface,TrimmedSurface};
    let limits=Limits::default();let n=28usize;
    let mut poles=Vec::new();for u in 0..n {for v in 0..n {poles.push([10.*u as f64/(n-1) as f64,10.*v as f64/(n-1) as f64,0.]);}}
    let mut knots=vec![0.];knots.extend((0..n).map(|i|i as f64/(n-1) as f64));knots.push(1.);
    let surface=om9_core::geometry::NurbsSurface {degree:[1,1],size:[n,n],poles,weights:vec![1.;n*n],knots:[knots.clone(),knots],domain:[[0.,1.],[0.,1.]],periodic:[false,false],reversed:false};
    let trim=TrimmedSurface {underlying:surface.clone(),outer:vec![[0.2,0.2],[0.8,0.2],[0.8,0.8],[0.2,0.8]],inners:vec![vec![[0.4,0.4],[0.6,0.4],[0.6,0.6],[0.4,0.6]]],reversed:false};
    let mut entity=Entity::new("large trim",Geometry::Surface(surface),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();let id=entity.id;
    let mut doc=Document::new("large trim incremental");let initial=Scene::capture(Arc::new(doc.clone()),1,&limits).unwrap();
    let bound=initial.update_usage(Some(&entity)).unwrap();doc.entities.insert(id,entity);
    let full=Scene::capture(Arc::new(doc.clone()),2,&limits).unwrap();
    let updated=initial.update_owned(Arc::new(doc),2,Some(id),&limits).unwrap();
    assert!(bound.bytes>=updated.cache_usage().unwrap().bytes-initial.cache_usage().unwrap().bytes);
    assert_eq!(full.objects[0].buffers.as_ref(),updated.objects[0].buffers.as_ref());
    assert!(!updated.objects[0].buffers.is_trim_display_fallback());
}

#[test]
fn many_edge_trim_scene_preflight_covers_capture_and_incremental_update() {
    use om9_core::rhino_semantics::{set_trimmed_surface,TrimmedSurface};
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    let outer=(0..129).map(|i| {let angle=std::f64::consts::TAU*i as f64/129.;
        [0.5+0.25*angle.cos(),0.5+0.25*angle.sin()]}).collect();
    let trim=TrimmedSurface {underlying:surface.clone(),outer,inners:vec![],reversed:false};
    let mut entity=Entity::new("many edges",Geometry::Surface(surface),None);
    set_trimmed_surface(&mut entity,&trim,&limits).unwrap();let id=entity.id;
    let mut document=Document::new("many-edge trim admission");
    let initial=Scene::capture(Arc::new(document.clone()),1,&limits).unwrap();
    let bound=initial.update_usage(Some(&entity)).unwrap();
    document.entities.insert(id,entity);
    let peak=Scene::candidate_scene_peak(&document,&limits).unwrap();
    let full=Scene::capture(Arc::new(document.clone()),2,&limits).unwrap();
    let updated=initial.update_owned(Arc::new(document.clone()),2,Some(id),&limits).unwrap();
    assert!(!full.objects[0].buffers.is_trim_display_fallback());
    assert_eq!(full.objects[0].buffers.as_ref(),updated.objects[0].buffers.as_ref());
    assert!(peak>=full.cache_usage().unwrap().bytes);
    assert!(bound.bytes>=updated.cache_usage().unwrap().bytes-initial.cache_usage().unwrap().bytes);
    let tight=Limits {max_bytes:4*1024*1024,..limits};
    let tight_peak=Scene::candidate_scene_peak(&document,&tight).unwrap();
    let admitted=Scene::capture(Arc::new(document.clone()),3,&tight).expect("simple many-edge trim should fit its actual scene budget");
    assert!(!admitted.objects[0].buffers.is_trim_display_fallback());
    assert!(tight_peak>=admitted.cache_usage().unwrap().bytes);
    let insufficient=Limits {max_bytes:64*1024,..limits};
    assert!(Scene::capture(Arc::new(document),4,&insufficient).is_err());
}

#[test]
fn unsupported_trim_display_is_reported_in_both_scene_paths() {
    use om9_core::rhino_semantics::{set_trimmed_surface,TrimmedSurface};
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    let trim=TrimmedSurface {underlying:surface.clone(),outer:vec![[0.,0.],[1.,0.],[1.,1.],[0.,1.]],
        inners:vec![vec![[0.1,0.1],[0.9,0.1],[0.9,0.9],[0.1,0.9]],vec![[0.3,0.3],[0.7,0.3],[0.7,0.7],[0.3,0.7]]],reversed:false};
    let mut entity=Entity::new("nested",Geometry::Surface(surface),None);
    set_trimmed_surface(&mut entity,&trim,&limits).unwrap();let id=entity.id;
    let mut document=Document::new("unsupported trim");
    let initial=Scene::capture(Arc::new(document.clone()),1,&limits).unwrap();
    document.entities.insert(id,entity);
    let peak=Scene::candidate_scene_peak(&document,&limits).unwrap();
    let full=Scene::capture(Arc::new(document.clone()),2,&limits).unwrap();
    let updated=initial.update_owned(Arc::new(document),2,Some(id),&limits).unwrap();
    assert_eq!(full.objects.len(),1);assert_eq!(updated.objects.len(),1);
    assert!(full.objects[0].buffers.is_trim_display_fallback());
    assert!(updated.objects[0].buffers.is_trim_display_fallback());
    assert!(full.diagnostics.iter().any(|s|s.contains("trim display unsupported")));
    assert!(updated.diagnostics.iter().any(|s|s.contains("trim display unsupported")));
    assert!(peak>=full.cache_usage().unwrap().bytes);
}

#[test]
fn valid_trim_with_adjacent_float_levels_displays_and_selects_source_region() {
    use om9_core::rhino_semantics::{classify_trim_uv,set_trimmed_surface,validate_trim_loop_topology,TrimmedSurface};
    use om9_scene::{camera::{Camera,Viewport},scene::PickIntent};
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    let y=0.5_f64;let next=f64::from_bits(y.to_bits()+1);
    let trim=TrimmedSurface {underlying:surface.clone(),outer:vec![[0.2,0.2],[0.8,0.2],[0.8,0.8],
        [0.2,0.8],[0.2,next],[0.2,y]],inners:vec![],reversed:false};
    trim.validate(&limits).unwrap();validate_trim_loop_topology(&trim).unwrap();
    assert_eq!(classify_trim_uv(&trim,[0.5,0.6]).unwrap(),Some(true));
    assert_eq!(classify_trim_uv(&trim,[0.9,0.6]).unwrap(),Some(false));
    let mut entity=Entity::new("valid fallback",Geometry::Surface(surface),None);
    set_trimmed_surface(&mut entity,&trim,&limits).unwrap();let id=entity.id;
    let mut doc=Document::new("valid trim fallback");doc.entities.insert(id,entity);
    let scene=Scene::capture(Arc::new(doc.clone()),1,&limits).unwrap();
    assert!(!scene.objects[0].buffers.is_trim_display_fallback());
    assert_eq!(scene.document.entities[&id],doc.entities[&id]);
    let camera=Camera::orthographic([5.,6.,20.],[5.,6.,0.],[0.,1.,0.],10.).unwrap();
    let viewport=Viewport {origin:[0.,0.],size:[400.,400.]};
    assert_eq!(scene.pick_all([200.,200.],&camera,viewport,4.,PickIntent::Select).unwrap()[0].root,id);
    let outside=camera.project([9.,6.,0.],viewport).unwrap().unwrap().screen;
    assert!(scene.pick_all(outside,&camera,viewport,4.,PickIntent::Select).unwrap().is_empty());
    let bound=Scene::candidate_scene_peak(&doc,&limits).unwrap();
    assert!(bound>=scene.cache_usage().unwrap().bytes);
    let mut blank=doc.clone();blank.entities.remove(&id);
    let initial=Scene::capture(Arc::new(blank),0,&limits).unwrap();
    let updated=initial.update_owned(Arc::new(doc.clone()),1,Some(id),&limits).unwrap();
    assert!(!updated.objects[0].buffers.is_trim_display_fallback());
    assert_eq!(updated.objects[0].buffers.as_ref(),scene.objects[0].buffers.as_ref());
}

#[test]
fn valid_trim_with_adjacent_nonlinear_levels_preserves_notch_and_picks() {
    use om9_core::rhino_semantics::{classify_trim_uv,set_trimmed_surface,validate_trim_loop_topology,TrimmedSurface};
    use om9_scene::{camera::{Camera,Viewport},scene::PickIntent};
    let limits=Limits::default();let y=0.5_f64;let next=f64::from_bits(y.to_bits()+1);
    let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    let trim=TrimmedSurface {underlying:surface.clone(),outer:vec![[0.2,0.2],[0.8,0.2],[0.8,y],[0.7,next],[0.8,0.8],[0.2,0.8]],inners:vec![],reversed:false};
    trim.validate(&limits).unwrap();validate_trim_loop_topology(&trim).unwrap();
    assert_eq!(classify_trim_uv(&trim,[0.5,0.4]).unwrap(),Some(true));
    assert_eq!(classify_trim_uv(&trim,[0.9,0.4]).unwrap(),Some(false));
    let mut entity=Entity::new("valid notched trim",Geometry::Surface(surface),None);
    set_trimmed_surface(&mut entity,&trim,&limits).unwrap();let id=entity.id;
    let mut doc=Document::new("valid notched fallback");doc.entities.insert(id,entity);
    let scene=Scene::capture(Arc::new(doc.clone()),1,&limits).unwrap();assert!(!scene.objects[0].buffers.is_trim_display_fallback());
    assert_eq!(scene.document.entities[&id],doc.entities[&id]);
    let om9_scene::scene::RenderBuffers::Mesh {vertices,..}=scene.objects[0].buffers.as_ref() else {panic!("mesh")};
    assert!(vertices.iter().any(|p|(p[0]-7.).abs()<1e-12 && (p[1]-next*10.).abs()<1e-12));
    assert!(vertices.iter().any(|p|(p[0]-8.).abs()<1e-12 && (p[1]-5.).abs()<1e-12));
    let camera=Camera::orthographic([5.,4.,20.],[5.,4.,0.],[0.,1.,0.],10.).unwrap();
    let viewport=Viewport {origin:[0.,0.],size:[400.,400.]};
    assert_eq!(scene.pick_all([200.,200.],&camera,viewport,4.,PickIntent::Select).unwrap()[0].root,id);
    let outside=camera.project([9.,4.,0.],viewport).unwrap().unwrap().screen;
    assert!(scene.pick_all(outside,&camera,viewport,4.,PickIntent::Select).unwrap().is_empty());
    let bound=Scene::candidate_scene_peak(&doc,&limits).unwrap();assert!(bound>=scene.cache_usage().unwrap().bytes);
    let mut blank=doc.clone();blank.entities.remove(&id);
    let initial=Scene::capture(Arc::new(blank),0,&limits).unwrap();
    let updated=initial.update_owned(Arc::new(doc),1,Some(id),&limits).unwrap();
    assert!(!updated.objects[0].buffers.is_trim_display_fallback());
    assert_eq!(updated.objects[0].buffers.as_ref(),scene.objects[0].buffers.as_ref());
}

#[test]
fn valid_501_edge_trim_is_selectable_without_fallback() {
    use om9_core::rhino_semantics::{set_trimmed_surface,validate_trim_loop_topology,TrimmedSurface};
    use om9_scene::{camera::{Camera,Viewport},scene::PickIntent};
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    let outer=(0..501).map(|i| {let a=std::f64::consts::TAU*(i as f64)/501.;[0.5+0.3*a.cos(),0.5+0.3*a.sin()]}).collect();
    let trim=TrimmedSurface {underlying:surface.clone(),outer,inners:vec![],reversed:false};
    trim.validate(&limits).unwrap();validate_trim_loop_topology(&trim).unwrap();
    let mut entity=Entity::new("501 edge trim",Geometry::Surface(surface),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();let id=entity.id;
    let mut doc=Document::new("501 edge trim");doc.entities.insert(id,entity);
    let scene=Scene::capture(Arc::new(doc.clone()),1,&limits).unwrap();assert!(!scene.objects[0].buffers.is_trim_display_fallback());
    assert_eq!(scene.document.entities[&id],doc.entities[&id]);
    let camera=Camera::orthographic([5.,5.,20.],[5.,5.,0.],[0.,1.,0.],10.).unwrap();let viewport=Viewport {origin:[0.,0.],size:[400.,400.]};
    assert_eq!(scene.pick_all([200.,200.],&camera,viewport,4.,PickIntent::Select).unwrap()[0].root,id);
    let outside=camera.project([9.,5.,0.],viewport).unwrap().unwrap().screen;
    assert!(scene.pick_all(outside,&camera,viewport,4.,PickIntent::Select).unwrap().is_empty());
    let peak=Scene::candidate_scene_peak(&doc,&limits).unwrap();assert!(peak>=scene.cache_usage().unwrap().bytes);
    let mut blank=doc.clone();blank.entities.remove(&id);let initial=Scene::capture(Arc::new(blank),0,&limits).unwrap();
    let updated=initial.update_owned(Arc::new(doc),1,Some(id),&limits).unwrap();
    assert_eq!(scene.objects[0].buffers.as_ref(),updated.objects[0].buffers.as_ref());
}

#[test]
fn large_trim_topology_limit_keeps_crossing_and_capacity_checks() {
    use om9_core::rhino_semantics::{validate_trim_loop_topology,TrimmedSurface,TRIM_TOPOLOGY_MAX_EDGES};
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    let polygon=|count:usize| (0..count).map(|i|{let a=std::f64::consts::TAU*(i as f64)/(count as f64);[0.5+0.3*a.cos(),0.5+0.3*a.sin()]}).collect::<Vec<_>>();
    let mut trim=TrimmedSurface {underlying:surface,outer:polygon(TRIM_TOPOLOGY_MAX_EDGES),inners:vec![],reversed:false};
    assert_eq!(TRIM_TOPOLOGY_MAX_EDGES,707);validate_trim_loop_topology(&trim).unwrap();
    trim.outer.swap(50,350);assert!(validate_trim_loop_topology(&trim).is_err());
    trim.outer=polygon(TRIM_TOPOLOGY_MAX_EDGES+1);
    assert!(matches!(validate_trim_loop_topology(&trim),Err(om9_core::Error::Unsupported("trim topology exceeds verification limit"))));
}

#[test]
fn max_supported_707_edge_trim_matches_incremental_scene() {
    use om9_core::rhino_semantics::{set_trimmed_surface,TrimmedSurface,TRIM_TOPOLOGY_MAX_EDGES};
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    let outer=(0..TRIM_TOPOLOGY_MAX_EDGES).map(|i|{let a=std::f64::consts::TAU*(i as f64)/(TRIM_TOPOLOGY_MAX_EDGES as f64);[0.5+0.3*a.cos(),0.5+0.3*a.sin()]}).collect();
    let trim=TrimmedSurface {underlying:surface.clone(),outer,inners:vec![],reversed:false};
    let mut entity=Entity::new("707 edges",Geometry::Surface(surface),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();let id=entity.id;
    let mut doc=Document::new("707 edges");doc.entities.insert(id,entity);
    let full=Scene::capture(Arc::new(doc.clone()),1,&limits).unwrap();assert!(!full.objects[0].buffers.is_trim_display_fallback());
    let mut blank=doc.clone();blank.entities.remove(&id);let initial=Scene::capture(Arc::new(blank),0,&limits).unwrap();
    let peak=initial.update_usage(doc.entities.get(&id)).unwrap();
    let updated=initial.update_owned(Arc::new(doc),1,Some(id),&limits).unwrap();
    assert!(peak.bytes>=updated.cache_usage().unwrap().bytes-initial.cache_usage().unwrap().bytes);
    assert_eq!(full.objects[0].buffers.as_ref(),updated.objects[0].buffers.as_ref());
}

#[test]
fn large_trim_with_hole_keeps_hole_and_incremental_admission() {
    use om9_core::rhino_semantics::{set_trimmed_surface,TrimmedSurface};
    use om9_scene::{camera::{Camera,Viewport},scene::PickIntent};
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    let outer=(0..501).map(|i|{let a=std::f64::consts::TAU*(i as f64)/501.;[0.5+0.4*a.cos(),0.5+0.4*a.sin()]}).collect();
    let trim=TrimmedSurface {underlying:surface.clone(),outer,inners:vec![vec![[0.4,0.4],[0.6,0.4],[0.6,0.6],[0.4,0.6]]],reversed:false};
    let mut entity=Entity::new("large trim hole",Geometry::Surface(surface),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();let id=entity.id;
    let mut doc=Document::new("large hole");doc.entities.insert(id,entity);
    let scene=Scene::capture(Arc::new(doc.clone()),1,&limits).unwrap();assert!(!scene.objects[0].buffers.is_trim_display_fallback());
    let camera=Camera::orthographic([5.,5.,20.],[5.,5.,0.],[0.,1.,0.],10.).unwrap();let viewport=Viewport {origin:[0.,0.],size:[400.,400.]};
    assert!(scene.pick_all([200.,200.],&camera,viewport,1.,PickIntent::Select).unwrap().is_empty());
    let inside=camera.project([7.,5.,0.],viewport).unwrap().unwrap().screen;
    assert_eq!(scene.pick_all(inside,&camera,viewport,1.,PickIntent::Select).unwrap()[0].root,id);
    let mut blank=doc.clone();blank.entities.remove(&id);let initial=Scene::capture(Arc::new(blank),0,&limits).unwrap();
    let updated=initial.update_owned(Arc::new(doc),1,Some(id),&limits).unwrap();
    assert_eq!(scene.objects[0].buffers.as_ref(),updated.objects[0].buffers.as_ref());
}

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
    assert!(old.objects[0].buffers.ptr_eq(&new.objects[0].buffers));
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
        (Geometry::Surface(surface), 168, 242, 66),
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
    let limits = Limits::default();
    let mut document = Document::new("shared old buffer");
    let first = Entity::new("first", Geometry::Point([0., 0., 0.]), None);
    document.entities.insert(first.id, first);
    let old = Scene::capture(Arc::new(document.clone()), 1, &limits).unwrap();
    let shared_bytes = old.shared_buffer_bytes();
    let added = Entity::new("second", Geometry::Point([1., 2., 3.]), None);
    let id = added.id;
    document.entities.insert(id, added);
    let (new, retained_bytes) = old.update_owned_with_retained_bytes(Arc::new(document), 2, Some(id), &limits).unwrap();
    assert!(new.objects.iter().any(|object| object.buffers.ptr_eq(&old.objects[0].buffers)));
    assert_eq!(retained_bytes + shared_bytes, new.cache_usage().unwrap().bytes);
}
