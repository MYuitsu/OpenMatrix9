// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{budget::Limits,geometry::Geometry,model::{Document,Entity},rhino_semantics::{set_trimmed_surface,TrimmedSurface}};

#[test]
fn reopened_many_edge_trim_fits_bounded_scene_and_picks_source_region() {
    use om9_scene::{camera::{Camera,Viewport},scene::PickIntent};
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::plane([-2.,-2.,0.],[1.,0.,0.],[0.,1.,0.],4.,4.,&limits).unwrap();
    let outer=(0..129).map(|i| {let a=std::f64::consts::TAU*i as f64/129.;[0.5+0.25*a.cos(),0.5+0.25*a.sin()]}).collect();
    let trim=TrimmedSurface {underlying:surface.clone(),outer,inners:vec![],reversed:false};
    let mut entity=Entity::new("trim",Geometry::Surface(surface),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();let id=entity.id;
    let mut document=Document::with_template_tolerance("trim budget");document.entities.insert(id,entity);
    let path=std::env::temp_dir().join(format!("om9-trim-scene-budget-{}.om9",uuid::Uuid::new_v4()));
    struct Remove(std::path::PathBuf);impl Drop for Remove {fn drop(&mut self) {let _=std::fs::remove_file(&self.0);}}
    let _cleanup=Remove(path.clone());
    om9_archive::Archive::save(&path,&document,&limits,&om9_core::transaction::Cancellation::default()).unwrap();
    let bounded=Limits {max_bytes:4*1024*1024,..limits};
    let mut controller=om9_app::session::Controller::from_file(&path,bounded).unwrap();
    controller.submit("Point").unwrap();controller.submit("100,100,100").unwrap();controller.undo().unwrap();
    assert_eq!(controller.coordinator().redo_len(),1);
    let scene=controller.scene().unwrap();assert!(!scene.objects[0].buffers.is_trim_display_fallback());
    let camera=Camera::orthographic([0.,0.,10.],[0.,0.,0.],[0.,1.,0.],4.).unwrap();
    let viewport=Viewport {origin:[0.,0.],size:[400.,400.]};
    let project=|p|camera.project(p,viewport).unwrap().unwrap().screen;
    assert_eq!(scene.pick(project([0.,0.,0.]),&camera,viewport,2.,PickIntent::Select).unwrap().unwrap().root,id);
    assert!(scene.pick(project([1.5,0.,0.]),&camera,viewport,2.,PickIntent::Select).unwrap().is_none());
    assert_eq!(controller.coordinator().redo_len(),1);
}

#[test]
fn paste_many_edge_trim_with_bounded_scene_is_one_undoable_transaction() {
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::plane([-2.,-2.,0.],[1.,0.,0.],[0.,1.,0.],4.,4.,&limits).unwrap();
    let outer=(0..129).map(|i| {let a=std::f64::consts::TAU*i as f64/129.;[0.5+0.25*a.cos(),0.5+0.25*a.sin()]}).collect();
    let trim=TrimmedSurface {underlying:surface.clone(),outer,inners:vec![],reversed:false};
    let mut entity=Entity::new("trim",Geometry::Surface(surface),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();let id=entity.id;
    let mut document=Document::with_template_tolerance("trim copy");document.entities.insert(id,entity);
    let mut source=om9_app::session::Controller::new(document,limits).unwrap();source.select_objects(&[id]).unwrap();
    let payload=source.copy_selected_om9().unwrap();
    let bounded=Limits {max_bytes:4*1024*1024,..limits};
    let mut target=om9_app::session::Controller::new(Document::with_template_tolerance("trim paste"),bounded).unwrap();
    target.submit("Point").unwrap();target.submit("100,100,100").unwrap();target.undo().unwrap();
    let before=target.document().clone();assert_eq!(target.coordinator().redo_len(),1);
    let pressure=target.coordinator().reserve_remaining_external_memory_with_held(
        om9_core::budget::memory::MemoryClass::Kernel,None,target.scene_cache_usage().unwrap()).unwrap();
    assert!(target.paste_om9(&payload).is_err());assert_eq!(target.document(),&before);assert_eq!(target.coordinator().redo_len(),1);
    drop(pressure);
    let created=target.paste_om9(&payload).unwrap();assert_eq!(created.len(),1);
    assert!(!target.scene().unwrap().objects[0].buffers.is_trim_display_fallback());
    assert_eq!(target.coordinator().undo_len(),1);
    target.undo().unwrap();assert!(target.document().entities.is_empty());
    target.redo().unwrap();assert!(target.document().entities.contains_key(&created[0]));
}
