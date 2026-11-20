// SPDX-License-Identifier: LGPL-2.1-or-later
//! Refine B-rep display candidates only after checking their captured context.
use super::Controller;
use om9_core::{Error, Result, geometry::{Point3, Transform}};
use om9_scene::{
    camera::{Camera, Viewport},
    osnap::{SnapCandidate, SnapKind},
};
use std::sync::{Arc, Weak, atomic::{AtomicBool, Ordering}, mpsc};

#[cfg(test)]
mod receipt_tests {
    use super::*;
    use om9_core::{geometry::{Geometry, Mesh, NativeBrep, Transform}, model::{Document, Entity}};
    use om9_scene::{camera::Projection, osnap::SurfaceCandidateSource};

    #[test]
    #[cfg(feature="occt-brep")]
    fn fallback_curved_trim_selects_valid_rear_source_hit() {
        use om9_core::{budget::Limits, rhino_semantics::{classify_trim_uv,set_trimmed_surface,TrimmedSurface}};
        let limits=Limits::default();
        let surface=om9_core::surface_kernel::cylinder([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,5.,&limits).unwrap();
        let outer=(0..1025).map(|i| {let a=std::f64::consts::TAU*(i as f64)/1025.;
            [0.5+0.2*a.cos(),0.5+0.2*a.sin()]}).collect();
        let trim=TrimmedSurface {underlying:surface.clone(),outer,inners:vec![],reversed:false};
        let mut entity=Entity::new("curved fallback",Geometry::Surface(surface),None);
        set_trimmed_surface(&mut entity,&trim,&limits).unwrap();
        let mut doc=Document::with_template_tolerance("curved fallback");doc.entities.insert(entity.id,entity);
        let path=std::env::temp_dir().join(format!("om9-curved-fallback-{}.om9",uuid::Uuid::new_v4()));
        struct Cleanup(std::path::PathBuf);impl Drop for Cleanup {fn drop(&mut self){let _=std::fs::remove_file(&self.0);}}
        let _cleanup=Cleanup(path.clone());
        om9_archive::Archive::save(&path,&doc,&limits,&om9_core::transaction::Cancellation::default()).unwrap();
        let reopened=om9_archive::Archive::open(&path,&limits).unwrap();
        let c=Controller::new(reopened,limits).unwrap();
        let object=&c.scene().unwrap().objects[0];assert!(object.buffers.is_trim_display_fallback());
        let viewport=Viewport {origin:[0.,0.],size:[400.,400.]};
        let camera=Camera::orthographic([5.,1.,2.5],[0.,0.,2.5],[0.,0.,1.],10.).unwrap();
        let ray=camera.visible_ray([200.,200.],viewport).unwrap();
        let entity=c.scene().unwrap().entity_for(c.document(),object).unwrap();
        let hits=om9_kernel::measurement::surface_ray_hits_uv(&entity.geometry,object.display_transform().unwrap(),
            ray.origin,ray.direction,0.001,&limits).unwrap();
        let mut hits=hits;hits.sort_by(|a,b|a.distance.total_cmp(&b.distance));
        assert_eq!(hits.len(),2);
        assert_eq!(classify_trim_uv(&trim,hits[0].uv).unwrap(),Some(false));
        assert_eq!(classify_trim_uv(&trim,hits[1].uv).unwrap(),Some(true));
        assert!((hits[1].point[0]+10./26_f64.sqrt()).abs()<1e-6);
        assert!(c.fallback_trim_source_pick(object,&camera,viewport,[200.,200.]).unwrap().is_some());
        // A ray through the periodic seam produces two UV records at the
        // same front point, plus the distinct rear intersection.
        let seam=Camera::orthographic([5.,0.,2.5],[0.,0.,2.5],[0.,0.,1.],10.).unwrap()
            .visible_ray([200.,200.],viewport).unwrap();
        let mut seam_hits=om9_kernel::measurement::surface_ray_hits_uv(&entity.geometry,object.display_transform().unwrap(),
            seam.origin,seam.direction,0.001,&limits).unwrap();
        seam_hits.sort_by(|a,b|a.distance.total_cmp(&b.distance));
        assert_eq!(seam_hits.len(),3);
        assert!((seam_hits[0].distance-seam_hits[1].distance).abs()<1e-8);
        assert!(seam_hits[2].distance-seam_hits[1].distance>3.9);
        let seam_camera=Camera::orthographic([5.,0.,2.5],[0.,0.,2.5],[0.,0.,1.],10.).unwrap();
        assert!(c.fallback_trim_source_pick(object,&seam_camera,viewport,[200.,200.]).unwrap().is_some());
        let tangent=Camera::orthographic([2.,5.,2.5],[2.,0.,2.5],[0.,0.,1.],10.).unwrap();
        assert!(c.fallback_trim_source_pick(object,&tangent,viewport,[200.,200.]).unwrap().is_none());
        for z in [3.499,4.5] {
            let outside=Camera::orthographic([5.,1.,z],[0.,0.,z],[0.,0.,1.],10.).unwrap();
            assert!(c.fallback_trim_source_pick(object,&outside,viewport,[200.,200.]).unwrap().is_none());
        }
        let mut with_hole=trim.clone();with_hole.inners.push(vec![[0.45,0.45],[0.55,0.45],[0.55,0.55],[0.45,0.55]]);
        let mut holed=Entity::new("holed",Geometry::Surface(with_hole.underlying.clone()),None);
        set_trimmed_surface(&mut holed,&with_hole,&limits).unwrap();let mut d=Document::with_template_tolerance("hole");d.entities.insert(holed.id,holed);
        let h=Controller::new(d,limits).unwrap();let obj=&h.scene().unwrap().objects[0];
        assert!(h.fallback_trim_source_pick(obj,&seam_camera,viewport,[200.,200.]).unwrap().is_none());
        let mut altered=trim.underlying.clone();altered.poles[3][0]+=1e-9;
        assert!(certified_fallback_cylinder(&altered,&limits).is_none());
        let mut altered_trim=trim.clone();altered_trim.underlying=altered.clone();
        let mut altered_entity=Entity::new("uncertified",Geometry::Surface(altered),None);
        set_trimmed_surface(&mut altered_entity,&altered_trim,&limits).unwrap();
        let mut altered_doc=Document::with_template_tolerance("uncertified curved fallback");
        altered_doc.entities.insert(altered_entity.id,altered_entity);
        let uncertified=Controller::new(altered_doc,limits).unwrap();let uncertified_obj=&uncertified.scene().unwrap().objects[0];
        assert!(uncertified_obj.buffers.is_trim_display_fallback());
        let pressure=uncertified.coordinator().reserve_remaining_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::Kernel,None,uncertified.scene_cache_usage().unwrap()).unwrap();
        assert!(uncertified.fallback_trim_source_pick(uncertified_obj,&camera,viewport,[200.,200.]).unwrap().is_none());
        drop(pressure);
        let pressure=c.coordinator().reserve_remaining_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::Kernel,None,c.scene_cache_usage().unwrap()).unwrap();
        assert!(matches!(c.fallback_trim_source_pick(object,&camera,viewport,[200.,200.]),Err(Error::Budget(_))));
        drop(pressure);
        assert!(c.fallback_trim_source_pick(object,&camera,viewport,[200.,200.]).unwrap().is_some());
    }

    #[test]
    #[cfg(feature="occt-brep")]
    fn fallback_curved_trim_handles_affine_placement_and_refuses_ill_conditioning() {
        use om9_core::{budget::Limits,rhino_semantics::{set_trimmed_surface,TrimmedSurface},transaction::Cancellation};
        let limits=Limits::default();
        let surface=om9_core::surface_kernel::cylinder([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,5.,&limits).unwrap();
        let outer=(0..1025).map(|i| {let a=std::f64::consts::TAU*(i as f64)/1025.;
            [0.5+0.2*a.cos(),0.5+0.2*a.sin()]}).collect();
        let trim=TrimmedSurface {underlying:surface.clone(),outer,inners:vec![],reversed:false};
        let placements=[
            Transform([[2.,0.4,0.,4.],[0.,1.,0.,-3.],[0.,0.,1.5,7.],[0.,0.,0.,1.]]),
            Transform([[-1.5,0.2,0.,4.],[0.,1.,0.,-3.],[0.,0.,1.,7.],[0.,0.,0.,1.]]),
        ];
        for placement in placements {
            let mut entity=Entity::new("affine cylinder",Geometry::Surface(surface.clone()),None);
            set_trimmed_surface(&mut entity,&trim,&limits).unwrap();entity.placement=placement;
            let mut doc=Document::with_template_tolerance("affine curved fallback");doc.entities.insert(entity.id,entity);
            let path=std::env::temp_dir().join(format!("om9-affine-curved-{}.om9",uuid::Uuid::new_v4()));
            struct Cleanup(std::path::PathBuf);impl Drop for Cleanup {fn drop(&mut self){let _=std::fs::remove_file(&self.0);}}
            let _cleanup=Cleanup(path.clone());om9_archive::Archive::save(&path,&doc,&limits,&Cancellation::default()).unwrap();
            let c=Controller::new(om9_archive::Archive::open(&path,&limits).unwrap(),limits).unwrap();
            let object=&c.scene().unwrap().objects[0];assert!(object.buffers.is_trim_display_fallback());
            let viewport=Viewport {origin:[0.,0.],size:[400.,400.]};
            for (z,want) in [(2.5,true),(3.45,false),(3.499,false),(4.5,false)] {
                let eye=placement.point([5.,1.,z]).unwrap();let target=placement.point([0.,0.,z]).unwrap();
                let camera=Camera::orthographic(eye,target,[0.,0.,1.],10.).unwrap();
                assert_eq!(c.fallback_trim_source_pick(object,&camera,viewport,[200.,200.]).unwrap().is_some(),want);
            }
        }
        let bad=Transform::scale([1.,1.,1e-9]).unwrap();
        assert!(fallback_inverse_bound(bad).is_none());
        assert!(fallback_inverse_bound(Transform::identity()).is_some());
        let diag=Transform::scale([2.,0.5,4.]).unwrap();
        let expected=(0.5_f64.powi(2)+2_f64.powi(2)+0.25_f64.powi(2)).sqrt();
        assert!((fallback_inverse_bound(diag).unwrap()-expected).abs()<1e-12);
    }

    #[test]
    #[cfg(feature="occt-brep")]
    fn fallback_curved_occlusion_clears_only_excluded_front_hits() {
        use om9_core::{budget::{Limits,memory::MemoryClass},rhino_semantics::{set_trimmed_surface,TrimmedSurface}};
        let limits=Limits::default();
        let surface=om9_core::surface_kernel::cylinder([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,5.,&limits).unwrap();
        let outer=(0..1025).map(|i| {let a=std::f64::consts::TAU*(i as f64)/1025.;[0.5+0.2*a.cos(),0.5+0.2*a.sin()]}).collect();
        let trim=TrimmedSurface {underlying:surface.clone(),outer,inners:vec![],reversed:false};
        let placement=Transform([[2.,0.4,0.,4.],[0.,1.,0.,-3.],[0.,0.,1.5,7.],[0.,0.,0.,1.]]);
        let mut entity=Entity::new("trim cylinder",Geometry::Surface(surface),None);
        set_trimmed_surface(&mut entity,&trim,&limits).unwrap();entity.placement=placement;
        let mut doc=Document::with_template_tolerance("curved fallback occlusion");doc.entities.insert(entity.id,entity);
        let c=Controller::new(doc,limits).unwrap();let object=&c.scene().unwrap().objects[0];
        assert!(object.buffers.is_trim_display_fallback());
        let viewport=Viewport {origin:[0.,0.],size:[400.,400.]};
        let target=placement.point([0.,0.,2.5]).unwrap();let eye=placement.point([5.,1.,2.5]).unwrap();
        let camera=Camera::orthographic(eye,target,[0.,0.,1.],10.).unwrap();
        let clear=||c.fallback_surface_proven_clear(object,&camera,viewport,[200.,200.],target,0.001);
        assert!(clear());
        let snapshot=c.capture_surface_occlusion().unwrap().unwrap();
        assert_eq!(snapshot.fallback.as_ref().unwrap().sources.len(),1);
        let expected=om9_scene::osnap::visible_display_occludes_with_fallback_proof(c.scene().unwrap(),&camera,viewport,
            [200.,200.],target,0.001,&[],None,|object|c.fallback_surface_proven_clear(object,&camera,viewport,[200.,200.],target,0.001)).unwrap();
        let worker_camera=camera.clone();
        let actual=std::thread::spawn(move || snapshot.scene.visible_display_occludes_with_fallback_proof(&worker_camera,viewport,
            [200.,200.],target,0.001,&[],None,|object| snapshot.fallback.as_ref().unwrap().sources.iter().find(|source|
                source.root==object.root && source.leaf==object.leaf && source.path==object.path).is_some_and(|source|
                    fallback_surface_proven_clear_source(&source.geometry,&source.trim,object.display_transform().unwrap(),
                        &worker_camera,viewport,[200.,200.],target,0.001,&snapshot.limits))).unwrap()).join().unwrap();
        assert_eq!(actual,expected);
        let mut with_target=c.document().clone();
        let mut sphere=Entity::new("behind fallback",Geometry::Brep(om9_kernel::brep::sphere_brep(0.2,0.02,&limits).unwrap()),None);
        sphere.placement=Transform::translation(target);
        let sphere_id=sphere.id;
        with_target.entities.insert(sphere_id,sphere);
        let worker_controller=Controller::new(with_target,limits).unwrap();
        let candidates:Vec<_>=om9_scene::osnap::candidates(worker_controller.scene().unwrap(),&camera,viewport,[200.,200.],
            Default::default(),None).unwrap().into_iter().filter(|candidate|
                candidate.kind==SnapKind::Surface && candidate.root==Some(sphere_id)).collect();
        assert!(!candidates.is_empty());
        let budget=worker_controller.coordinator().reserve_external_memory(MemoryClass::Kernel,8*1024*1024,None).unwrap();
        let mut scheduler=SurfaceSnapScheduler::default();
        scheduler.submit_with_occlusion(&worker_controller,candidates,camera.clone(),viewport,[200.,200.],budget).unwrap();
        let deadline=std::time::Instant::now()+std::time::Duration::from_secs(10);
        let receipt=loop { if let Some(result)=scheduler.poll(&worker_controller) {break result.unwrap()}
            assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(5)); };
        assert!(receipt.resolve(&worker_controller,&camera,viewport,[200.,200.]).unwrap().is_some());
        assert_eq!(receipt.display_occluded(),Some(false),"the worker must prove the fallback is clear");
        let mut detached_controller=Controller::new(c.document().clone(),limits).unwrap();
        let old_document=Arc::downgrade(&detached_controller.scene().unwrap().document);
        let detached=detached_controller.capture_surface_occlusion().unwrap().unwrap();
        detached_controller.begin_file_operation(crate::file_lifecycle::FileKind::New).unwrap();
        assert!(old_document.upgrade().is_none());
        let source=&detached.fallback.as_ref().unwrap().sources[0];
        assert!(fallback_surface_proven_clear_source(&source.geometry,&source.trim,placement,&camera,
            viewport,[200.,200.],target,0.001,&detached.limits));
        let behind=placement.point([-3.,-0.6,2.5]).unwrap();
        assert!(!fallback_surface_proven_clear_source(&source.geometry,&source.trim,placement,&camera,
            viewport,[200.,200.],behind,0.001,&detached.limits));
        let retained=detached_controller.coordinator().external_reserved_bytes(MemoryClass::Kernel);
        drop(detached);
        assert!(detached_controller.coordinator().external_reserved_bytes(MemoryClass::Kernel)<retained);
        assert!(!c.fallback_surface_proven_clear(object,&camera,viewport,[200.,200.],behind,0.001));
        let off_ray=placement.point([0.,0.1,2.5]).unwrap();
        assert!(!c.fallback_surface_proven_clear(object,&camera,viewport,[200.,200.],off_ray,0.001));
        let pressure=c.coordinator().reserve_remaining_external_memory_with_held(MemoryClass::Kernel,None,c.scene_cache_usage().unwrap()).unwrap();
        assert!(!clear());
        assert!(c.capture_surface_occlusion().is_err(), "fallback copies must not escape their admission");
        drop(pressure);assert!(clear());
        assert!(c.capture_surface_occlusion().unwrap().is_some());
        let hole_outer=(0..1025).map(|i| {let a=std::f64::consts::TAU*(i as f64)/1025.;[0.2+0.2*a.cos(),0.5+0.2*a.sin()]}).collect();
        let holed=TrimmedSurface {underlying:trim.underlying.clone(),outer:hole_outer,
            inners:vec![vec![[0.015,0.45],[0.07,0.45],[0.07,0.55],[0.015,0.55]]],reversed:false};
        let mut entity=Entity::new("hole",Geometry::Surface(holed.underlying.clone()),None);
        set_trimmed_surface(&mut entity,&holed,&limits).unwrap();entity.placement=placement;
        let mut doc=Document::with_template_tolerance("curved hole occlusion");doc.entities.insert(entity.id,entity);
        let h=Controller::new(doc,limits).unwrap();let obj=&h.scene().unwrap().objects[0];
        assert!(h.fallback_surface_proven_clear(obj,&camera,viewport,[200.,200.],target,0.001));
    }

    #[test]
    #[cfg(feature="occt-brep")]
    fn fallback_source_pick_verifies_large_convex_trim_and_refuses_uncertainty() {
        use om9_core::{budget::{Limits,memory::MemoryClass},rhino_semantics::{set_trimmed_surface,TrimmedSurface}};
        let limits=Limits::default();
        let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
        let polygon=|count:usize,step:usize| (0..count).map(|i|{let a=std::f64::consts::TAU*((i*step%count) as f64)/(count as f64);[0.5+0.3*a.cos(),0.5+0.3*a.sin()]}).collect::<Vec<_>>();
        assert!(certified_strict_convex_pick(&polygon(1025,1),[0.5,0.5],0.001));
        assert!(!certified_strict_convex_pick(&polygon(1025,5),[0.5,0.5],0.001));
        assert!(certified_strict_convex_pick(&polygon(1536,1),[0.5,0.5],0.001));
        assert!(!certified_strict_convex_pick(&polygon(1537,1),[0.5,0.5],0.001));
        for placement in [Transform::identity(),Transform([[2.,0.5,0.,4.],[0.25,1.,0.,-3.],[0.,0.,1.,7.],[0.,0.,0.,1.]])] {
            let trim=TrimmedSurface {underlying:surface.clone(),outer:polygon(1025,1),inners:vec![],reversed:false};
            let mut entity=Entity::new("large fallback",Geometry::Surface(surface.clone()),None);
            set_trimmed_surface(&mut entity,&trim,&limits).unwrap();entity.placement=placement;
            let mut document=Document::with_template_tolerance("source fallback pick");document.entities.insert(entity.id,entity);
            let c=Controller::new(document,limits).unwrap();let object=&c.scene().unwrap().objects[0];assert!(object.buffers.is_trim_display_fallback());
            let viewport=Viewport {origin:[0.,0.],size:[400.,400.]};
            let try_at=|uv:[f64;2]| {let target=placement.point([uv[0]*10.,uv[1]*10.,0.]).unwrap();
                let camera=Camera::orthographic([target[0],target[1],target[2]+10.],target,[0.,1.,0.],10.).unwrap();
                c.fallback_trim_source_pick(object,&camera,viewport,[200.,200.])};
            assert!(try_at([0.5,0.5]).unwrap().is_some());
            assert!(try_at([0.9,0.5]).unwrap().is_none());
            assert!(try_at([0.8,0.5]).unwrap().is_none());
            assert!(try_at([0.8-1e-6,0.5]).unwrap().is_none());
            let before=c.coordinator().snapshot();let pressure=c.coordinator().reserve_remaining_external_memory_with_held(
                MemoryClass::Kernel,None,c.scene_cache_usage().unwrap()).unwrap();
            assert!(matches!(try_at([0.5,0.5]),Err(Error::Budget(_))));assert_eq!(c.coordinator().snapshot(),before);
            drop(pressure);assert!(try_at([0.5,0.5]).unwrap().is_some());
        }
        let trim=TrimmedSurface {underlying:surface.clone(),outer:polygon(1025,1),inners:vec![],reversed:false};
        let mut entity=Entity::new("legacy no tolerance",Geometry::Surface(surface),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();
        let mut doc=Document::new("legacy no tolerance");doc.entities.insert(entity.id,entity);let c=Controller::new(doc,limits).unwrap();
        let camera=Camera::orthographic([5.,5.,10.],[5.,5.,0.],[0.,1.,0.],10.).unwrap();let viewport=Viewport {origin:[0.,0.],size:[400.,400.]};
        assert!(matches!(c.fallback_trim_source_pick(&c.scene().unwrap().objects[0],&camera,viewport,[200.,200.]),Err(Error::Missing("document tolerance"))));
        let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
        let trim=TrimmedSurface {underlying:surface.clone(),outer:polygon(1025,1),inners:vec![vec![[0.4,0.4],[0.6,0.4],[0.6,0.6],[0.4,0.6]]],reversed:false};
        let mut entity=Entity::new("large hole",Geometry::Surface(surface),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();
        let mut doc=Document::with_template_tolerance("unsupported hole");doc.entities.insert(entity.id,entity);let c=Controller::new(doc,limits).unwrap();
        assert!(c.fallback_trim_source_pick(&c.scene().unwrap().objects[0],&camera,viewport,[200.,200.]).unwrap().is_none());
        let target=[7.,5.,0.];let material_camera=Camera::orthographic([7.,5.,10.],target,[0.,1.,0.],10.).unwrap();
        assert!(c.fallback_trim_source_pick(&c.scene().unwrap().objects[0],&material_camera,viewport,[200.,200.]).unwrap().is_some());
        let before=c.coordinator().snapshot();let pressure=c.coordinator().reserve_remaining_external_memory_with_held(
            MemoryClass::Kernel,None,c.scene_cache_usage().unwrap()).unwrap();
        assert!(matches!(c.fallback_trim_source_pick(&c.scene().unwrap().objects[0],&material_camera,viewport,[200.,200.]),Err(Error::Budget(_))));
        assert_eq!(c.coordinator().snapshot(),before);drop(pressure);
        assert!(c.fallback_trim_source_pick(&c.scene().unwrap().objects[0],&material_camera,viewport,[200.,200.]).unwrap().is_some());
    }

    #[test]
    #[cfg(feature="occt-brep")]
    fn fallback_source_pick_concave_trim_and_hole() {
        use om9_core::{budget::Limits,rhino_semantics::{set_trimmed_surface,TrimmedSurface}};
        let limits=Limits::default();
        let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
        let mut outer=(0..1025).map(|i|{let a=std::f64::consts::TAU*(i as f64)/1025.;[0.5+0.3*a.cos(),0.5+0.3*a.sin()]}).collect::<Vec<_>>();
        outer[0]=[0.62,0.5];
        let mut trim=TrimmedSurface {underlying:surface.clone(),outer,inners:vec![vec![[0.4,0.4],[0.6,0.4],[0.6,0.6],[0.5,0.53],[0.4,0.6]]],reversed:false};
        assert!(!certified_convex_trim_pick(&trim,[0.5,0.72],0.001));
        assert!(certified_general_trim_pick(&trim,[0.5,0.72],0.001));
        assert!(!certified_general_trim_pick(&trim,[0.5,0.45],0.001));
        assert!(!certified_general_trim_pick(&trim,[0.7,0.5],0.001));
        assert!(!certified_general_trim_pick(&trim,[0.61999,0.5],0.001));
        let mut entity=Entity::new("concave trim",Geometry::Surface(surface),None);
        set_trimmed_surface(&mut entity,&trim,&limits).unwrap();
        let mut doc=Document::with_template_tolerance("concave source pick");doc.entities.insert(entity.id,entity);
        let c=Controller::new(doc,limits).unwrap();let object=&c.scene().unwrap().objects[0];assert!(object.buffers.is_trim_display_fallback());
        let viewport=Viewport {origin:[0.,0.],size:[400.,400.]};
        let pick=|uv:[f64;2]|{let target=[uv[0]*10.,uv[1]*10.,0.];
            let camera=Camera::orthographic([target[0],target[1],10.],target,[0.,1.,0.],10.).unwrap();
            c.fallback_trim_source_pick(object,&camera,viewport,[200.,200.]).unwrap().is_some()};
        assert!(pick([0.5,0.72]));assert!(!pick([0.5,0.45]));assert!(!pick([0.7,0.5]));
        trim.inners.push(vec![[0.45,0.42],[0.55,0.42],[0.55,0.48],[0.45,0.48]]);
        assert!(!certified_general_trim_pick(&trim,[0.5,0.72],0.001));
    }

    #[test]
    #[cfg(feature="occt-brep")]
    fn fallback_hole_certificate_refuses_intersections_nesting_and_near_boundaries() {
        use om9_core::{budget::Limits,rhino_semantics::TrimmedSurface};
        let limits=Limits::default();let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
        let outer=(0..1025).map(|i|{let a=std::f64::consts::TAU*(i as f64)/1025.;[0.5+0.4*a.cos(),0.5+0.4*a.sin()]}).collect();
        let first=vec![[0.3,0.4],[0.4,0.4],[0.4,0.6],[0.3,0.6]];
        let second=vec![[0.6,0.4],[0.7,0.4],[0.7,0.6],[0.6,0.6]];
        let mut trim=TrimmedSurface {underlying:surface,outer,inners:vec![first,second],reversed:false};
        assert!(certified_convex_trim_pick(&trim,[0.5,0.5],0.001));
        assert!(!certified_convex_trim_pick(&trim,[0.35,0.5],0.001));
        assert!(!certified_convex_trim_pick(&trim,[0.40001,0.5],0.001));
        trim.inners[1]=vec![[0.35,0.45],[0.45,0.45],[0.45,0.55],[0.35,0.55]];
        assert!(!certified_convex_trim_pick(&trim,[0.5,0.5],0.001));
        trim.inners[1]=vec![[0.32,0.45],[0.38,0.45],[0.38,0.55],[0.32,0.55]];
        assert!(!certified_convex_trim_pick(&trim,[0.5,0.5],0.001));
        trim.inners[1]=vec![[0.39999,0.4],[0.5,0.4],[0.5,0.6],[0.39999,0.6]];
        assert!(!certified_convex_trim_pick(&trim,[0.55,0.5],0.001));
        trim.inners[1]=vec![[0.40001,0.4],[0.5,0.4],[0.5,0.6],[0.40001,0.6]];
        assert!(!certified_general_trim_pick(&trim,[0.55,0.5],0.001));
        // Their axis-aligned bounds overlap, but a diagonal separating axis
        // proves these two convex holes are distinct.
        trim.inners=vec![vec![[0.34,0.42],[0.42,0.50],[0.50,0.42],[0.42,0.34]],
            vec![[0.47,0.55],[0.55,0.63],[0.63,0.55],[0.55,0.47]]];
        assert!(certified_convex_trim_pick(&trim,[0.5,0.72],0.001));
        trim.inners[1].reverse();assert!(certified_convex_trim_pick(&trim,[0.5,0.72],0.001));
        trim.inners[1]=vec![[0.6,0.4],[0.7,0.4],[0.65,0.5],[0.7,0.6],[0.6,0.6]];
        assert!(!certified_convex_trim_pick(&trim,[0.5,0.72],0.001));
        trim.inners[1]=vec![[0.75,0.7],[0.95,0.7],[0.95,0.8],[0.75,0.8]];
        assert!(!certified_convex_trim_pick(&trim,[0.5,0.72],0.001));
    }

    #[test]
    #[cfg(feature="occt-brep")]
    fn fallback_source_pick_resolves_nested_occurrence_and_permissions() {
        use om9_core::{budget::Limits,model::Definition,rhino_semantics::{set_trimmed_surface,TrimmedSurface}};
        let limits=Limits::default();let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
        let outer=(0..1025).map(|i|{let a=std::f64::consts::TAU*(i as f64)/1025.;[0.5+0.3*a.cos(),0.5+0.3*a.sin()]}).collect();
        let trim=TrimmedSurface {underlying:surface.clone(),outer,inners:vec![],reversed:false};
        for selectable in [true,false] {
            let mut leaf=Entity::new("leaf",Geometry::Surface(surface.clone()),None);set_trimmed_surface(&mut leaf,&trim,&limits).unwrap();leaf.attributes.selectable=selectable;
            leaf.placement=Transform::translation([1.,2.,0.]);let inner=Definition::new("inner",vec![leaf]);let inner_id=inner.id;
            let mut nested=Entity::new("nested",Geometry::Instance(inner_id),None);nested.placement=Transform::translation([4.,0.,0.]);
            let outer=Definition::new("outer",vec![nested]);let outer_id=outer.id;
            let mut root=Entity::new("root",Geometry::Instance(outer_id),None);root.placement=Transform::translation([10.,0.,0.]);let root_id=root.id;
            let mut doc=Document::with_template_tolerance("nested source pick");doc.definitions.insert(inner_id,inner);doc.definitions.insert(outer_id,outer);doc.entities.insert(root_id,root);
            let c=Controller::new(doc,limits).unwrap();let object=&c.scene().unwrap().objects[0];assert_eq!(object.root,root_id);assert_eq!(object.path.len(),2);
            let target=[20.,7.,0.];let camera=Camera::orthographic([20.,7.,10.],target,[0.,1.,0.],10.).unwrap();let viewport=Viewport {origin:[0.,0.],size:[400.,400.]};
            assert_eq!(c.fallback_trim_source_pick(object,&camera,viewport,[200.,200.]).unwrap().is_some(),selectable);
        }
    }

    #[test]
    #[cfg(feature="occt-brep")]
    fn optimized_convex_screen_never_accepts_rejected_reference_cases() {
        fn original(poly:&[[f64;2]],point:[f64;2],guard:f64)->bool {
            let mut orientation=0_i8;
            for i in 0..poly.len() {let a=poly[i];let b=poly[(i+1)%poly.len()];let length=(b[0]-a[0]).hypot(b[1]-a[1]);
                if !length.is_finite() || length==0. {return false;}
                for j in 0..=poly.len() {if j<poly.len() && (j==i || j==(i+1)%poly.len()) {continue;}
                    let p=if j==poly.len(){point}else{poly[j]};let x=(b[0]-a[0])*(p[1]-a[1]);let y=(b[1]-a[1])*(p[0]-a[0]);let cross=x-y;
                    let scale=a.into_iter().chain(b).chain(p).map(f64::abs).fold(1_f64,f64::max);
                    let delta=(b[0]-a[0]).abs()+(b[1]-a[1]).abs()+(p[0]-a[0]).abs()+(p[1]-a[1]).abs();
                    let bound=128.*f64::EPSILON*(x.abs()+y.abs()+scale*delta)+if j==poly.len(){guard*length}else{0.};
                    if !cross.is_finite() || !bound.is_finite() || cross.abs()<=bound {return false;}
                    let sign=if cross>0. {1}else{-1};if orientation==0 {orientation=sign;}else if orientation!=sign{return false;}
                }
            }true
        }
        for offset in [0.,1e6] {for count in [8,63,1025] {
            let mut polygon:Vec<_>=(0..count).map(|i|{let a=std::f64::consts::TAU*(i as f64)/(count as f64);[offset+3.*a.cos(),offset+2.*a.sin()]}).collect();
            for reverse in [false,true] {if reverse {polygon.reverse();}
                for point in [[offset,offset],[offset+3.,offset],[offset+2.9,offset],[offset+4.,offset]] {
                    for guard in [0.,0.001] {let fast=certified_strict_convex_pick(&polygon,point,guard);
                        assert!(!fast || original(&polygon,point,guard),"optimized screen accepted a reference refusal");}
                }
            }
        }}
    }

    #[test]
    #[ignore = "timing diagnostic for synchronous fallback source selection"]
    #[cfg(feature="occt-brep")]
    fn fallback_source_pick_timing_probe() {
        use om9_core::{budget::Limits,rhino_semantics::{set_trimmed_surface,TrimmedSurface}};
        let limits=Limits::default();
        for (count,has_hole,concave) in [(1025,false,false),(1536,false,false),(1025,true,false),(1025,true,true),(1536,false,true)] {
            let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
            let mut outer=(0..count).map(|i|{let a=std::f64::consts::TAU*(i as f64)/(count as f64);[0.5+0.3*a.cos(),0.5+0.3*a.sin()]}).collect::<Vec<_>>();
            if concave {outer[0]=[0.62,0.5];}
            let inners=if has_hole {vec![vec![[0.4,0.4],[0.6,0.4],[0.6,0.6],[0.4,0.6]]]}else{vec![]};
            let trim=TrimmedSurface {underlying:surface.clone(),outer,inners,reversed:false};
            let mut entity=Entity::new("timing trim",Geometry::Surface(surface),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();
            let mut doc=Document::with_template_tolerance("fallback timing");doc.entities.insert(entity.id,entity);
            let c=Controller::new(doc,limits).unwrap();let object=&c.scene().unwrap().objects[0];
            let (x,y)=if has_hole && concave {(5.,7.2)}else if has_hole {(7.,5.)}else{(5.,5.)};
            let camera=Camera::orthographic([x,y,10.],[x,y,0.],[0.,1.,0.],10.).unwrap();let viewport=Viewport {origin:[0.,0.],size:[400.,400.]};
            assert!(c.fallback_trim_source_pick(object,&camera,viewport,[200.,200.]).unwrap().is_some());
            let mut samples=Vec::new();for _ in 0..5 {let start=std::time::Instant::now();
                assert!(c.fallback_trim_source_pick(object,&camera,viewport,[200.,200.]).unwrap().is_some());samples.push(start.elapsed().as_micros());}
            eprintln!("fallback-source-pick outer_vertices={count} hole={has_hole} concave={concave} microseconds={samples:?}");
        }
    }

    #[test]
    #[cfg(feature = "occt-brep")]
    fn fallback_occlusion_proof_is_conservative_near_outer_bounds_and_pressure() {
        use om9_core::{budget::{Limits,memory::MemoryClass},rhino_semantics::{set_trimmed_surface,TrimmedSurface}};
        let limits=Limits::default();
        let surface=om9_core::surface_kernel::plane([-2.,-2.,0.],[1.,0.,0.],[0.,1.,0.],4.,4.,&limits).unwrap();
        let trim=TrimmedSurface {underlying:surface.clone(),outer:vec![[0.2,0.2],[0.8,0.2],[0.8,0.8],[0.2,0.8]],
            inners:vec![vec![[0.3,0.3],[0.7,0.3],[0.7,0.7],[0.3,0.7]],vec![[0.4,0.4],[0.6,0.4],[0.6,0.6],[0.4,0.6]]],reversed:false};
        for placement in [Transform::identity(),Transform([[2.,0.5,0.,4.],[0.25,1.,0.,-3.],[0.,0.,1.,7.],[0.,0.,0.,1.]])] {
            let mut entity=Entity::new("fallback",Geometry::Surface(surface.clone()),None);
            set_trimmed_surface(&mut entity,&trim,&limits).unwrap();entity.placement=placement;
            let mut document=Document::with_template_tolerance("fallback proof");document.entities.insert(entity.id,entity);
            let controller=Controller::new(document,limits).unwrap();let scene=controller.scene().unwrap();let object=&scene.objects[0];
            assert!(object.buffers.is_trim_display_fallback());
            let viewport=Viewport {origin:[0.,0.],size:[400.,400.]};
            let test=|x:f64| {let point=placement.point([x,0.,0.]).unwrap();let camera=Camera::orthographic([point[0],point[1],point[2]+10.],point,[0.,1.,0.],10.).unwrap();
                let target=[point[0],point[1],point[2]-1.];
                controller.fallback_surface_proven_clear(object,&camera,viewport,[200.,200.],target,0.001)};
            assert!(test(1.5));assert!(!test(1.2001));assert!(!test(0.9));assert!(!test(0.));
            let point=placement.point([1.5,0.,0.]).unwrap();
            let grazing=Camera::orthographic([point[0]-10_000.,point[1],point[2]+10.],point,[0.,1.,0.],10.).unwrap();
            let target=[point[0]+1.,point[1],point[2]-0.001];
            assert!(!controller.fallback_surface_proven_clear(object,&grazing,viewport,[200.,200.],target,0.001),"grazing ray needs a larger uncertainty guard");
            let pressure=controller.coordinator().reserve_remaining_external_memory_with_held(MemoryClass::Kernel,None,controller.scene_cache_usage().unwrap()).unwrap();
            assert!(!test(1.5));drop(pressure);assert!(test(1.5));
        }
    }

    #[test]
    #[cfg(feature = "occt-brep")]
    fn nonconvex_hole_point_requires_secure_interior() {
        let hole=[[0.4,0.4],[0.6,0.4],[0.6,0.6],[0.5,0.53],[0.4,0.6]];
        assert!(trim_point_inside_with_margin(&hole,[0.5,0.5],0.001));
        assert!(!trim_point_inside_with_margin(&hole,[0.5,0.52999],0.001));
        assert!(!trim_point_inside_with_margin(&hole,[0.5,0.55],0.001));
        assert!(!trim_point_inside_with_margin(&hole,[f64::NAN,0.5],0.001));
        assert!(!trim_point_inside_with_margin(&hole,[0.5,0.5],f64::NAN));
    }

    #[test]
    #[cfg(feature = "occt-brep")]
    fn fallback_hole_clear_proof_rejects_near_boundary_and_memory_pressure() {
        use om9_core::{budget::{Limits,memory::MemoryClass},rhino_semantics::{set_trimmed_surface,TrimmedSurface}};
        let limits=Limits::default();let surface=om9_core::surface_kernel::plane([-2.,-2.,0.],[1.,0.,0.],[0.,1.,0.],4.,4.,&limits).unwrap();
        let outer=(0..1025).map(|i|{let a=std::f64::consts::TAU*(i as f64)/1025.;[0.5+0.4*a.cos(),0.5+0.4*a.sin()]}).collect();
        let trim=TrimmedSurface {underlying:surface.clone(),outer,inners:vec![vec![[0.4,0.4],[0.6,0.4],[0.6,0.6],[0.4,0.6]]],reversed:false};
        for placement in [Transform::identity(),Transform([[2.,0.5,0.,4.],[0.25,1.,0.,-3.],[0.,0.,1.,7.],[0.,0.,0.,1.]])] {
            let mut entity=Entity::new("fallback hole",Geometry::Surface(surface.clone()),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();entity.placement=placement;
            let mut doc=Document::with_template_tolerance("hole proof");doc.entities.insert(entity.id,entity);let c=Controller::new(doc,limits).unwrap();let object=&c.scene().unwrap().objects[0];
            assert!(object.buffers.is_trim_display_fallback());let viewport=Viewport {origin:[0.,0.],size:[400.,400.]};
            let check=|x:f64|{let p=placement.point([x,0.,0.]).unwrap();let camera=Camera::orthographic([p[0],p[1],p[2]+10.],p,[0.,1.,0.],10.).unwrap();
                c.fallback_surface_proven_clear(object,&camera,viewport,[200.,200.],[p[0],p[1],p[2]-1.],0.001)};
            assert!(check(0.));assert!(!check(0.3999));assert!(!check(0.8));
            let pressure=c.coordinator().reserve_remaining_external_memory_with_held(MemoryClass::Kernel,None,c.scene_cache_usage().unwrap()).unwrap();
            assert!(!check(0.));drop(pressure);assert!(check(0.));
        }
    }

    #[test]
    #[cfg(feature = "occt-brep")]
    fn certified_seam_check_rejects_real_boundaries_and_nearby_holes() {
        use om9_core::{budget::Limits,rhino_semantics::TrimmedSurface};
        let limits=Limits::default();
        let surface=om9_core::surface_kernel::cylinder([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,5.,&limits).unwrap();
        let mut trim=TrimmedSurface {underlying:surface.clone(),outer:vec![[0.,0.2],[1.,0.2],[1.,0.8],[0.,0.8]],inners:vec![],reversed:false};
        assert!(certified_trim_seam_hit(&surface,&trim,[0.,0.5],&limits).unwrap());
        assert!(!certified_trim_seam_hit(&surface,&trim,[0.,0.2],&limits).unwrap());
        assert!(!certified_trim_seam_hit(&surface,&trim,[1e-16,0.5],&limits).unwrap());
        trim.inners.push(vec![[1e-16,0.4],[0.1,0.4],[0.1,0.6],[1e-16,0.6]]);
        assert!(!certified_trim_seam_hit(&surface,&trim,[0.,0.5],&limits).unwrap());
    }

    #[test]
    #[cfg(feature = "occt-brep")]
    fn mixed_surface_and_brep_sources_run_without_document_or_display() {
        use om9_core::budget::{Limits, memory::MemoryClass};
        let limits=Limits::default();
        let surface=om9_core::surface_kernel::plane([0.,0.,-2.],[1.,0.,0.],[0.,1.,0.],4.,4.,&limits).unwrap();
        let brep=om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Box { x_mm:4., y_mm:4., z_mm:4. },0.1,&limits).unwrap();
        let display=brep.display_liveness();
        let mut document=Document::with_template_tolerance("mixed surface sources");
        let plane=Entity::new("surface",Geometry::Surface(surface),None);let plane_id=plane.id;document.entities.insert(plane.id,plane);
        let body=Entity::new("body",Geometry::Brep(brep),None);let body_id=body.id;document.entities.insert(body.id,body);
        let mut controller=Controller::new(document,limits).unwrap();
        let camera=Camera::orthographic([2.,2.,10.],[2.,2.,0.],[0.,1.,0.],10.).unwrap();
        let viewport=Viewport {origin:[0.,0.],size:[400.,400.]};let cursor=[200.,200.];
        let all=om9_scene::osnap::candidates(controller.scene().unwrap(),&camera,viewport,cursor,Default::default(),None).unwrap();
        let find=|id| all.iter().find(|candidate|candidate.kind==SnapKind::Surface && candidate.root==Some(id)).unwrap().clone();
        let body_candidate=find(body_id);
        let candidates=vec![find(plane_id),body_candidate.clone(),body_candidate];
        assert!(matches!(admitted_mixed_source_budget(controller.document(),&candidates,u64::MAX,u64::MAX),
            Err(Error::Overflow("surface batch admission"))));
        let exact=SurfaceSnapJob::source_only_minimum(&controller,&candidates,&camera,viewport,cursor).unwrap().unwrap();
        let exact_reservation=controller.coordinator().reserve_external_memory(MemoryClass::Kernel,exact,None).unwrap();
        let mut exact_job=SurfaceSnapJob::start(&controller,candidates.clone(),camera.clone(),viewport,cursor,exact_reservation).unwrap();
        assert!(exact_job.source_is_owned_surface);
        let deadline=std::time::Instant::now()+std::time::Duration::from_secs(10);
        let exact_receipt=loop {if let Some(result)=exact_job.try_result(){break result.unwrap();}assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(5));};
        assert_eq!(exact_receipt.resolve(&controller,&camera,viewport,cursor).unwrap().unwrap().root,Some(body_id));
        assert!(matches!(exact_receipt.resolve(&controller,&camera,viewport,[cursor[0]+1.,cursor[1]]),Err(Error::Stale(_))));
        assert_eq!(exact_receipt.accept(&controller,&camera,viewport,cursor).unwrap().unwrap().root,Some(body_id));
        drop(exact_job);
        let reservation=controller.coordinator().reserve_external_memory(MemoryClass::Kernel,8*1024*1024,None).unwrap();
        let mut job=SurfaceSnapJob::start(&controller,candidates.clone(),camera.clone(),viewport,cursor,reservation).unwrap();
        assert!(job.source_is_owned_surface);
        let deadline=std::time::Instant::now()+std::time::Duration::from_secs(10);
        let receipt=loop {if let Some(result)=job.try_result(){break result.unwrap();}assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(5));};
        let hit=receipt.accept(&controller,&camera,viewport,cursor).unwrap().unwrap();
        assert_eq!(hit.root,Some(body_id));assert!((hit.world[2]-4.).abs()<1e-8);
        let probe=controller.coordinator().reserve_external_memory(MemoryClass::Kernel,8*1024*1024,None).unwrap();
        let minimum=SurfaceSnapJob::validate_request(&controller,&candidates,&camera,viewport,cursor,&probe).unwrap();
        drop(probe);
        let low=controller.coordinator().reserve_external_memory(MemoryClass::Kernel,minimum,None).unwrap();
        let mut fallback=SurfaceSnapJob::start(&controller,candidates.clone(),camera.clone(),viewport,cursor,low).unwrap();
        assert!(!fallback.source_is_owned_surface);
        let deadline=std::time::Instant::now()+std::time::Duration::from_secs(10);
        let receipt=loop {if let Some(result)=fallback.try_result(){break result.unwrap();}assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(5));};
        assert_eq!(receipt.accept(&controller,&camera,viewport,cursor).unwrap().unwrap().root,Some(body_id));
        let reservation=controller.coordinator().reserve_external_memory(MemoryClass::Kernel,8*1024*1024,None).unwrap();
        let weak=Arc::downgrade(&controller.scene().unwrap().document);
        let (release,gate)=std::sync::mpsc::channel();
        let permit=controller.coordinator().external_memory_account().try_acquire_surface_snap().unwrap();
        let mut stale=SurfaceSnapJob::start_with_permit(&controller,candidates,camera.clone(),viewport,cursor,reservation,permit,Some(gate)).unwrap();
        assert!(stale.source_is_owned_surface);
        controller.begin_file_operation(crate::file_lifecycle::FileKind::New).unwrap();
        assert!(weak.upgrade().is_none());assert!(!display.is_alive());
        release.send(()).unwrap();
        let deadline=std::time::Instant::now()+std::time::Duration::from_secs(10);
        let receipt=loop {if let Some(result)=stale.try_result(){break result.unwrap();}assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(5));};
        assert!(matches!(receipt.accept(&controller,&camera,viewport,cursor),Err(Error::Stale(_))));
    }

    #[test]
    #[cfg(feature = "occt-brep")]
    fn multiple_surface_sources_run_without_retaining_document() {
        use om9_core::budget::{Limits, memory::MemoryClass};
        let limits=Limits::default();
        let mut document=Document::with_template_tolerance("two surfaces");
        let mut ids=Vec::new();
        for z in [0.,3.] {
            let surface=om9_core::surface_kernel::plane([-2.,-2.,z],[1.,0.,0.],[0.,1.,0.],4.,4.,&limits).unwrap();
            let entity=Entity::new("surface",Geometry::Surface(surface),None);
            ids.push(entity.id); document.entities.insert(entity.id,entity);
        }
        let mut controller=Controller::new(document,limits).unwrap();
        let camera=Camera::orthographic([0.,0.,10.],[0.,0.,0.],[0.,1.,0.],10.).unwrap();
        let viewport=Viewport {origin:[0.,0.],size:[400.,400.]}; let cursor=[200.,200.];
        let candidates: Vec<_> = om9_scene::osnap::candidates(controller.scene().unwrap(),&camera,viewport,cursor,Default::default(),None).unwrap()
            .into_iter().filter(|candidate| candidate.kind==SnapKind::Surface && candidate.root.is_some_and(|id| ids.contains(&id))).collect();
        assert_eq!(candidates.len(),2);
        let reserve=|| controller.coordinator().reserve_external_memory(MemoryClass::Kernel,8*1024*1024,None).unwrap();
        let mut job=SurfaceSnapJob::start(&controller,candidates.clone(),camera.clone(),viewport,cursor,reserve()).unwrap();
        assert!(job.source_is_owned_surface);
        let deadline=std::time::Instant::now()+std::time::Duration::from_secs(10);
        let receipt=loop {if let Some(result)=job.try_result(){break result.unwrap();}assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(5));};
        let hit=receipt.accept(&controller,&camera,viewport,cursor).unwrap().unwrap();
        assert_eq!(hit.root,Some(ids[1]));
        assert!((hit.world[2]-3.).abs()<1e-8);
        let probe=reserve();
        let minimum=SurfaceSnapJob::validate_request(&controller,&candidates,&camera,viewport,cursor,&probe).unwrap();
        drop(probe);
        let fallback_reservation=controller.coordinator().reserve_external_memory(MemoryClass::Kernel,minimum,None).unwrap();
        let mut fallback=SurfaceSnapJob::start(&controller,candidates.clone(),camera.clone(),viewport,cursor,fallback_reservation).unwrap();
        assert!(!fallback.source_is_owned_surface);
        let deadline=std::time::Instant::now()+std::time::Duration::from_secs(10);
        let receipt=loop {if let Some(result)=fallback.try_result(){break result.unwrap();}assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(5));};
        assert_eq!(receipt.accept(&controller,&camera,viewport,cursor).unwrap().unwrap().root,Some(ids[1]));
        let reservation=reserve();
        let weak=Arc::downgrade(&controller.scene().unwrap().document);
        let (release,gate)=std::sync::mpsc::channel();
        let permit=controller.coordinator().external_memory_account().try_acquire_surface_snap().unwrap();
        let mut stale=SurfaceSnapJob::start_with_permit(&controller,candidates,camera.clone(),viewport,cursor,reservation,permit,Some(gate)).unwrap();
        assert!(stale.source_is_owned_surface);
        controller.begin_file_operation(crate::file_lifecycle::FileKind::New).unwrap();
        assert!(weak.upgrade().is_none());
        release.send(()).unwrap();
        let deadline=std::time::Instant::now()+std::time::Duration::from_secs(10);
        let receipt=loop {if let Some(result)=stale.try_result(){break result.unwrap();}assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(5));};
        assert!(matches!(receipt.accept(&controller,&camera,viewport,cursor),Err(Error::Stale(_))));
    }

    #[test]
    #[cfg(feature = "occt-brep")]
    fn trimmed_cylinder_worker_admits_scratch_and_finds_rear_hit() {
        use om9_core::{budget::{Limits,memory::MemoryClass},rhino_semantics::{set_trimmed_surface,TrimmedSurface}};
        let limits=Limits::default();
        let surface=om9_core::surface_kernel::cylinder([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,5.,&limits).unwrap();
        let trim=TrimmedSurface {underlying:surface.clone(),outer:vec![[0.,0.],[1.,0.],[1.,1.],[0.,1.]],
            inners:vec![vec![[0.01,0.4],[0.08,0.4],[0.08,0.6],[0.01,0.6]]],reversed:false};
        let mut entity=Entity::new("trim",Geometry::Surface(surface),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();
        let mut doc=Document::with_template_tolerance("trim worker");doc.entities.insert(entity.id,entity);
        let mut controller=Controller::new(doc,limits).unwrap();
        let camera=Camera::orthographic([10.,0.4,2.5],[0.,0.4,2.5],[0.,0.,1.],8.).unwrap();
        let viewport=Viewport {origin:[0.,0.],size:[400.,400.]};let cursor=[200.,200.];
        let candidate=om9_scene::osnap::candidates(controller.scene().unwrap(),&camera,viewport,cursor,Default::default(),None).unwrap()
            .into_iter().find(|c|c.kind==SnapKind::Surface).unwrap();
        let small=controller.coordinator().reserve_external_memory(MemoryClass::Kernel,4096,None).unwrap();
        assert!(matches!(SurfaceSnapJob::start(&controller,vec![candidate.clone()],camera.clone(),viewport,cursor,small),Err(Error::Budget(_))));
        let before=controller.coordinator().external_reserved_bytes(MemoryClass::Kernel);
        let enough=controller.coordinator().reserve_external_memory(MemoryClass::Kernel,8*1024*1024,None).unwrap();
        let mut job=SurfaceSnapJob::start(&controller,vec![candidate.clone()],camera.clone(),viewport,cursor,enough).unwrap();
        assert!(job.source_is_owned_surface);
        assert!(controller.coordinator().external_pending_bytes(MemoryClass::Kernel) < 8*1024*1024);
        let deadline=std::time::Instant::now()+std::time::Duration::from_secs(10);
        let receipt=loop {if let Some(result)=job.try_result(){break result.unwrap();} assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(5));};
        let retained=receipt._reservation.lock().unwrap().bytes();
        assert_eq!(retained,receipt.retained_bytes().unwrap());
        assert!(retained<8*1024*1024);
        assert!(receipt.accept(&controller,&camera,viewport,cursor).unwrap().unwrap().world[0] < -1.9);
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel),before+retained);
        drop(job);
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel),before);
        let baseline=controller.coordinator().external_reserved_bytes(MemoryClass::Kernel);
        let cancel_budget=controller.coordinator().reserve_external_memory(MemoryClass::Kernel,8*1024*1024,None).unwrap();
        let (release,gate)=std::sync::mpsc::channel();
        let permit=controller.coordinator().external_memory_account().try_acquire_surface_snap().unwrap();
        let mut canceled=SurfaceSnapJob::start_with_permit(&controller,vec![candidate.clone()],camera.clone(),viewport,cursor,cancel_budget,permit,Some(gate)).unwrap();
        canceled.cancel();release.send(()).unwrap();
        let deadline=std::time::Instant::now()+std::time::Duration::from_secs(10);
        loop {if let Some(result)=canceled.try_result(){assert!(matches!(result,Err(Error::Canceled)));break;}assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(5));}
        drop(canceled);
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel),baseline);
        let mut spent=controller.coordinator().reserve_external_memory(MemoryClass::Kernel,8*1024*1024,None).unwrap();
        spent.mark_allocated(8*1024*1024).unwrap();
        assert!(matches!(SurfaceSnapJob::start(&controller,vec![candidate.clone()],camera.clone(),viewport,cursor,spent),Err(Error::Budget(_))));
        let enough=controller.coordinator().reserve_external_memory(MemoryClass::Kernel,8*1024*1024,None).unwrap();
        let old_document=Arc::downgrade(&controller.scene().unwrap().document);
        let (release, gate)=std::sync::mpsc::channel();
        let permit=controller.coordinator().external_memory_account().try_acquire_surface_snap().unwrap();
        let mut stale=SurfaceSnapJob::start_with_permit(&controller,vec![candidate],camera.clone(),viewport,cursor,enough,permit,Some(gate)).unwrap();
        assert!(stale.source_is_owned_surface);
        controller.begin_file_operation(crate::file_lifecycle::FileKind::New).unwrap();
        assert!(old_document.upgrade().is_none(), "surface worker must not retain the old document");
        release.send(()).unwrap();
        let deadline=std::time::Instant::now()+std::time::Duration::from_secs(10);
        let receipt=loop {if let Some(result)=stale.try_result(){break result.unwrap();} assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(5));};
        assert!(matches!(receipt.accept(&controller,&camera,viewport,cursor),Err(Error::Stale(_))));
    }

    #[test]
    #[cfg(feature = "occt-brep")]
    fn surface_snap_reopened_front_face_does_not_snap_to_back() {
        use om9_core::budget::Limits;
        let limits = Limits::default();
        let body = om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Box {
            x_mm: 4., y_mm: 5., z_mm: 6.
        }, 0.1, &limits).unwrap();
        let mut display = body.display().clone();
        let face_at_z = |z: f64| -> usize {
            (0..display.faces.len()).find(|&face| {
                let tris: Vec<_> = display.triangles.iter().filter(|tri| tri[3] as usize == face).collect();
                !tris.is_empty() && tris.iter().all(|tri| tri[..3].iter().all(|&i| (display.points[i as usize][2]-z).abs() < 1e-8))
            }).unwrap()
        };
        let front = face_at_z(6.); let back = face_at_z(0.);
        display.faces.swap(front, back);
        for entry in &mut display.face_edges {
            if entry[0] as usize == front { entry[0] = back as u32; } else if entry[0] as usize == back { entry[0] = front as u32; }
        }
        for entry in &mut display.triangles {
            if entry[3] as usize == front { entry[3] = back as u32; } else if entry[3] as usize == back { entry[3] = front as u32; }
        }
        let body = om9_core::brep::Brep::from_native(body.source().to_owned(), display, &limits).unwrap();
        let mut document = Document::with_template_tolerance("surface snap reordered faces");
        let entity = Entity::new("box", Geometry::Brep(body), None);
        document.entities.insert(entity.id, entity);
        let path = std::env::temp_dir().join(format!("om9-snap-order-{}.om9", uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&path, &document, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
        let controller = Controller::from_file(&path, limits).unwrap(); std::fs::remove_file(path).unwrap();
        let camera = Camera::orthographic([2., 2.5, 20.], [2., 2.5, 0.], [0., 1., 0.], 8.).unwrap();
        let viewport = Viewport { origin: [0., 0.], size: [800., 800.] };
        let cursor = camera.project([2., 2.5, 6.], viewport).unwrap().unwrap().screen;
        let candidate = om9_scene::osnap::candidates(controller.scene().unwrap(), &camera, viewport, cursor, Default::default(), None).unwrap()
            .into_iter().find(|candidate| candidate.kind == SnapKind::Surface && (candidate.world[2]-6.).abs() < 1e-8).unwrap();
        let refined = controller.refine_surface_snap(&candidate, &camera, viewport).unwrap();
        assert!((refined.world[2]-6.).abs() < 1e-8, "snapped to {:?}, expected front face", refined.world);
        let group = controller.refine_surface_snap_group(&[&candidate, &candidate], &camera, viewport).unwrap();
        assert!(group[..2].iter().all(|value| value.as_ref().is_some_and(|snap| (snap.world[2]-6.).abs() < 1e-8)));
        let mut altered = candidate.clone();
        altered.surface_source.as_mut().unwrap().face_witness.as_mut().unwrap().0[0][2] = 0.;
        assert!(controller.refine_surface_snap(&altered, &camera, viewport).is_err());
        let reservation = controller.coordinator().reserve_external_memory(om9_core::budget::memory::MemoryClass::Kernel, 8 * 1024 * 1024, None).unwrap();
        let mut job = SurfaceSnapJob::start(&controller, vec![candidate], camera.clone(), viewport, cursor, reservation).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let receipt = loop {
            if let Some(result) = job.try_result() { break result.unwrap(); }
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(5));
        };
        let snap = receipt.accept(&controller, &camera, viewport, cursor).unwrap().unwrap();
        assert!((snap.world[2]-6.).abs() < 1e-8);
    }

    #[test]
    #[cfg(feature = "occt-brep")]
    fn admitted_occlusion_snapshot_releases_old_document_and_budget() {
        use om9_core::{budget::{Limits, memory::MemoryClass}, geometry::Mesh};
        let limits = Limits::default();
        let mut document = Document::with_template_tolerance("worker occlusion lifetime");
        let entity = Entity::new("triangle", Geometry::Mesh(Mesh {
            vertices: vec![[-3.,-3.,0.], [3.,-3.,0.], [0.,3.,0.]], faces: vec![vec![0,1,2]],
            normals: vec![], colors: vec![], corner_uvs: vec![],
        }), None);
        document.entities.insert(entity.id, entity);
        let mut controller = Controller::new(document, limits).unwrap();
        let weak = Arc::downgrade(&controller.scene().unwrap().document);
        let snapshot = controller.capture_surface_occlusion().unwrap().unwrap();
        controller.begin_file_operation(crate::file_lifecycle::FileKind::New).unwrap();
        assert!(weak.upgrade().is_none());
        let before_drop = controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache);
        let camera = Camera::orthographic([0.,0.,10.], [0.,0.,0.], [0.,1.,0.], 8.).unwrap();
        let viewport = Viewport { origin: [0.,0.], size: [800.,800.] };
        assert!(snapshot.scene.visible_display_occludes_with_fallback_proof(&camera, viewport,
            [400.,400.], [0.,0.,-1.], 0.001, &[], None, |_| false).unwrap());
        drop(snapshot);
        assert!(controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache) < before_drop);
    }

    #[test]
    #[cfg(feature = "occt-brep")]
    fn scheduled_surface_snap_checks_detached_display_occlusion() {
        use om9_core::{budget::Limits, geometry::Mesh};
        let limits = Limits::default();
        let body = om9_kernel::brep::sphere_brep(2., 0.1, &limits).unwrap();
        let mut document = Document::with_template_tolerance("worker occlusion");
        let entity = Entity::new("sphere", Geometry::Brep(body), None);
        let sphere = entity.id;
        document.entities.insert(sphere, entity);
        let blocker = Entity::new("blocker", Geometry::Mesh(Mesh {
            vertices: vec![[-3.,-3.,3.], [3.,-3.,3.], [0.,3.,3.]], faces: vec![vec![0,1,2]],
            normals: vec![], colors: vec![], corner_uvs: vec![],
        }), None);
        document.entities.insert(blocker.id, blocker);
        let controller = Controller::new(document, limits).unwrap();
        let camera = Camera::orthographic([0.,0.,10.], [0.,0.,0.], [0.,1.,0.], 8.).unwrap();
        let viewport = Viewport { origin: [0.,0.], size: [800.,800.] };
        let cursor = [400.,400.];
        let candidates: Vec<_> = om9_scene::osnap::candidates(controller.scene().unwrap(), &camera, viewport, cursor,
            Default::default(), None).unwrap().into_iter().filter(|candidate|
                candidate.kind == SnapKind::Surface && candidate.root == Some(sphere)).collect();
        assert!(!candidates.is_empty());
        let reservation = controller.coordinator().reserve_external_memory(om9_core::budget::memory::MemoryClass::Kernel,
            8*1024*1024, None).unwrap();
        let mut scheduler = SurfaceSnapScheduler::default();
        scheduler.submit_with_occlusion(&controller, candidates, camera.clone(), viewport, cursor, reservation).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let receipt = loop {
            if let Some(result) = scheduler.poll(&controller) { break result.unwrap(); }
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(5));
        };
        assert!(receipt.resolve(&controller, &camera, viewport, cursor).unwrap().is_some());
        assert_eq!(receipt.display_occluded(), Some(true));
        let changed = Camera::orthographic([1.,0.,10.], [0.,0.,0.], [0.,1.,0.], 8.).unwrap();
        assert!(receipt.resolve(&controller, &changed, viewport, cursor).is_err());
        drop(receipt);
        let candidates: Vec<_> = om9_scene::osnap::candidates(controller.scene().unwrap(), &camera, viewport, cursor,
            Default::default(), None).unwrap().into_iter().filter(|candidate|
                candidate.kind == SnapKind::Surface && candidate.root == Some(sphere)).collect();
        let before = controller.coordinator().snapshot();
        let budget = controller.coordinator().reserve_external_memory(om9_core::budget::memory::MemoryClass::Kernel,
            8*1024*1024, None).unwrap();
        let pressure = controller.coordinator().reserve_remaining_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::CpuCache, None, controller.scene_cache_usage().unwrap()).unwrap();
        assert!(scheduler.submit_with_occlusion(&controller, candidates.clone(), camera.clone(), viewport, cursor, budget).is_err());
        assert!(!scheduler.is_busy());
        assert_eq!(controller.coordinator().snapshot(), before);
        drop(pressure);
        let budget = controller.coordinator().reserve_external_memory(om9_core::budget::memory::MemoryClass::Kernel,
            8*1024*1024, None).unwrap();
        scheduler.submit_with_occlusion(&controller, candidates, camera.clone(), viewport, cursor, budget).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let retried = loop {
            if let Some(result) = scheduler.poll(&controller) { break result.unwrap(); }
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(5));
        };
        assert_eq!(retried.display_occluded(), Some(true));
    }

    #[test]
    #[cfg(feature = "occt-brep")]
    fn surface_candidate_rejects_new_scene_with_same_document_and_epoch() {
        use om9_core::budget::Limits;
        let limits = Limits::default();
        let body = om9_kernel::brep::sphere_brep(2., 0.1, &limits).unwrap();
        let mut document = Document::with_template_tolerance("surface publication");
        let entity = Entity::new("body", Geometry::Brep(body), None);
        document.entities.insert(entity.id, entity);
        let mut controller = Controller::new(document, limits).unwrap();
        let camera = Camera::orthographic([0., 0., 10.], [0., 0., 0.], [0., 1., 0.], 8.).unwrap();
        let viewport = Viewport { origin: [0., 0.], size: [800., 800.] };
        let cursor = camera.project([0., 0., 0.], viewport).unwrap().unwrap().screen;
        let candidate = om9_scene::osnap::candidates(controller.scene().unwrap(), &camera, viewport, cursor, Default::default(), None).unwrap()
            .into_iter().find(|candidate| candidate.kind == SnapKind::Surface).unwrap();
        assert!(controller.validate_surface_candidate(&candidate, &camera, viewport).is_ok());
        let occupied = controller.coordinator().external_memory_account().try_acquire_surface_snap().unwrap();
        let budget = controller.coordinator().reserve_external_memory(om9_core::budget::memory::MemoryClass::Kernel, 1024*1024, None).unwrap();
        let mut scheduler = SurfaceSnapScheduler::default();
        let candidates = vec![candidate.clone()];
        scheduler.submit(&controller, candidates.clone(), camera.clone(), viewport, cursor, budget).unwrap();
        assert!(scheduler.matches_request(&controller, &candidates, &camera, viewport, cursor));
        let scene = controller.scene().unwrap();
        let replacement = om9_scene::scene::Scene::capture(Arc::clone(&scene.document), scene.epoch, &limits).unwrap();
        controller.scene = Some(replacement);
        assert!(!scheduler.matches_request(&controller, &candidates, &camera, viewport, cursor));
        scheduler.cancel(); drop(occupied);
        assert!(matches!(controller.validate_surface_candidate(&candidate, &camera, viewport), Err(Error::Stale("surface snap scene"))));
    }

    #[test]
    fn native_surface_candidate_refuses_changed_payload_or_checksum() {
        let body = NativeBrep {
            display: Arc::new(Mesh { vertices: vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]], faces: vec![vec![0, 1, 2]], normals: vec![], colors: vec![], corner_uvs: vec![] }),
            payload: vec![1, 2, 3].into(), crc: 1,
        };
        let mut document = Document::new("native surface source");
        let entity = Entity::new("body", Geometry::NativeBrep(body.into_source_and_representation().0), None);
        let id = entity.id;
        let receipt = om9_scene::scene::brep_display_fingerprint(&entity);
        document.entities.insert(id, entity);
        let source = SurfaceCandidateSource {
            path: vec![], leaf: id, leaf_generation: document.entities[&id].generation,
            leaf_geometry_revision: document.entities[&id].geometry_revision,
            brep_display_fingerprint: receipt, source: document.stamp(id).unwrap(), scene_epoch: 1, scene_publication: uuid::Uuid::new_v4(),
            placement: Transform::identity(),
            camera: Camera { eye: [0., 0., 10.], target: [0., 0., 0.], up: [0., 1., 0.],
                projection: Projection::Orthographic { vertical_span_mm: 10. }, near_mm: 0.1, far_mm: 100., revision: 1 },
            viewport: Viewport { origin: [0., 0.], size: [100., 100.] }, cursor: [50., 50.], face: None, face_witness: None, face_witness_triangle: None, native_face_count: None,
        };
        let candidate = SnapCandidate { kind: SnapKind::Surface, root: Some(id), world: [0., 0., 0.],
            screen: [50., 50.], distance_px: 0., normal: None, parameter: None, tracking_tangent: None, source: None, second_source: None, surface_source: Some(source), apparent: false };
        assert!(captured_entity(&document, &candidate).is_ok());
        let mut changed_payload = document.clone();
        let Geometry::NativeBrep(brep) = &mut changed_payload.entities.get_mut(&id).unwrap().geometry else { panic!("native BRep") };
        brep.payload = vec![4, 5, 6].into();
        assert!(matches!(captured_entity(&changed_payload, &candidate), Err(Error::Stale("surface snap BRep display source"))));
        let Geometry::NativeBrep(brep) = &mut document.entities.get_mut(&id).unwrap().geometry else { panic!("native BRep") };
        brep.crc = 2;
        assert!(matches!(captured_entity(&document, &candidate), Err(Error::Stale("surface snap BRep display source"))));
    }
}

/// Captured results are not usable until checked against the current view and document.
/// The caller must also validate the published viewport receipt and applicable
/// occlusion before using the point; this worker does not perform those checks.
pub struct SurfaceSnapReceipt {
    document: Weak<om9_core::model::Document>,
    original: Arc<Vec<SnapCandidate>>,
    refined: Vec<SnapCandidate>,
    camera: Camera,
    viewport: Viewport,
    cursor: [f64; 2],
    selection_revision: u64,
    preview_revision: u64,
    display_occluded: Option<bool>,
    cancellation: Arc<AtomicBool>,
    // Last: receipt data and weak headers drop before the shared charge.
    _reservation: Arc<std::sync::Mutex<om9_core::transaction::ExternalMemoryReservation>>,
}

fn surface_receipt_fixed_bytes() -> Result<u64> {
    let bytes = std::mem::size_of::<SurfaceSnapReceipt>()
        .checked_add(std::mem::size_of::<SurfaceSnapJob>())
        .and_then(|n| n.checked_add(std::mem::size_of::<Vec<SnapCandidate>>()))
        .and_then(|n| n.checked_add(std::mem::size_of::<om9_core::model::Document>()))
        .and_then(|n| n.checked_add(std::mem::size_of::<AtomicBool>()))
        .and_then(|n| n.checked_add(std::mem::size_of::<std::sync::Mutex<om9_core::transaction::ExternalMemoryReservation>>()))
        .and_then(|n| n.checked_add(8 * std::mem::size_of::<usize>()))
        .ok_or(Error::Overflow("surface snap receipt"))?;
    u64::try_from(bytes).map_err(|_| Error::Overflow("surface snap receipt"))
}

fn candidate_weak_owner_bytes(candidate: &SnapCandidate) -> u64 {
    if candidate.surface_source.as_ref().is_some_and(|source| matches!(source.brep_display_fingerprint,
        Some(om9_scene::scene::BrepDisplayReceipt::Native { .. }))) {
        (std::mem::size_of::<Vec<u8>>() + std::mem::size_of::<om9_core::geometry::Mesh>()
            + 4 * std::mem::size_of::<usize>()) as u64
    } else { 0 }
}

fn candidate_path_entries(candidate: &SnapCandidate, include_clone: bool) -> Result<usize> {
    let paths = [
        candidate.surface_source.as_ref().map(|source| &source.path),
        candidate.source.as_ref().map(|source| &source.path),
        candidate.second_source.as_ref().map(|source| &source.path),
    ];
    paths.into_iter().flatten().try_fold(0usize, |total, path| {
        let entries = if include_clone { path.capacity().checked_add(path.len()) } else { Some(path.capacity()) }
            .ok_or(Error::Overflow("surface snap candidate paths"))?;
        total.checked_add(entries).ok_or(Error::Overflow("surface snap candidate paths"))
    })
}

impl SurfaceSnapReceipt {
    /// Available only when the worker captured a complete display occlusion snapshot.
    pub(crate) fn display_occluded(&self) -> Option<bool> { self.display_occluded }
    /// Compare a cached result to the entire current request before reuse.
    pub fn matches_request(&self, controller: &Controller, candidates: &[SnapCandidate],
        camera: &Camera, viewport: Viewport, cursor: [f64; 2]) -> bool {
        let Some(scene) = controller.scene() else { return false };
        !self.cancellation.load(Ordering::Acquire)
            && self.document.ptr_eq(&Arc::downgrade(&scene.document))
            && self.original.as_slice() == candidates && self.camera == *camera
            && self.viewport == viewport && self.cursor == cursor
            && self.selection_revision == controller.selection_state().revision
            && self.preview_revision == controller.preview_revision()
            && candidates.iter().all(|candidate| candidate.surface_source.as_ref()
                .is_some_and(|source| source.scene_publication == scene.publication_id()))
    }
    fn retained_bytes(&self) -> Result<u64> {
        let slots = self.original.capacity().checked_add(self.refined.capacity())
            .and_then(|n| n.checked_mul(std::mem::size_of::<SnapCandidate>()))
            .ok_or(Error::Overflow("surface snap receipt"))?;
        let mut bytes = surface_receipt_fixed_bytes()?.checked_add(u64::try_from(slots)
            .map_err(|_| Error::Overflow("surface snap receipt"))?).ok_or(Error::Overflow("surface snap receipt"))?;
        for candidate in self.original.iter().chain(&self.refined) {
            let path = candidate_path_entries(candidate, false)?
                .checked_mul(std::mem::size_of::<om9_core::identity::ObjectId>()).ok_or(Error::Overflow("surface snap receipt"))?;
            bytes = bytes.checked_add(u64::try_from(path).map_err(|_| Error::Overflow("surface snap receipt"))?)
                .and_then(|n| n.checked_add(candidate_weak_owner_bytes(candidate)))
                .ok_or(Error::Overflow("surface snap receipt"))?;
        }
        Ok(bytes.max(1))
    }
    pub fn accept(self, controller: &Controller, camera: &Camera, viewport: Viewport, cursor: [f64; 2]) -> Result<Option<SnapCandidate>> {
        self.resolve(controller, camera, viewport, cursor)
    }

    /// Revalidate an accounted receipt before each use, including cached previews.
    pub fn resolve(&self, controller: &Controller, camera: &Camera, viewport: Viewport, cursor: [f64; 2]) -> Result<Option<SnapCandidate>> {
        if self.cancellation.load(Ordering::Acquire) { return Err(Error::Canceled); }
        let scene = controller.scene().ok_or(Error::Missing("surface snap scene"))?;
        if !self.document.ptr_eq(&Arc::downgrade(&scene.document)) {
            return Err(Error::Stale("surface snap document snapshot"));
        }
        if *camera != self.camera || viewport != self.viewport || cursor != self.cursor
            || controller.selection_state().revision != self.selection_revision
            || controller.preview_revision() != self.preview_revision {
            return Err(Error::Stale("surface snap view or cursor"));
        }
        for candidate in self.original.iter() {
            controller.validate_surface_candidate(candidate, camera, viewport)?;
        }
        Ok(om9_scene::osnap::choose(&self.refined))
    }
}

/// A single worker holding a registered document snapshot and caller reservation.
/// The caller supplies the full candidate set and a conservative native-work
/// reservation; candidate enumeration and occluders are outside this worker.
/// This foundation is not yet connected to UI scheduling or gesture release.
pub struct SurfaceSnapJob {
    document: Weak<om9_core::model::Document>,
    cancel: Arc<AtomicBool>,
    result: Option<mpsc::Receiver<Result<SurfaceSnapReceipt>>>,
    delivered: bool,
    candidates: Arc<Vec<SnapCandidate>>,
    camera: Camera,
    viewport: Viewport,
    cursor: [f64; 2],
    selection_revision: u64,
    preview_revision: u64,
    #[cfg(test)]
    source_is_owned_surface: bool,
    // A caller can retain the job and cancel a delivered receipt. Keep its
    // shared candidates, weak identity and cancellation owner charged too.
    _reservation: Arc<std::sync::Mutex<om9_core::transaction::ExternalMemoryReservation>>,
}

enum SurfaceWorkerSource {
    Document(om9_core::transaction::CapturedDocument),
    #[cfg(feature = "occt-brep")]
    Brep { source: om9_core::transaction::CapturedBrepSource, tolerance: Result<f64> },
    #[cfg(feature = "occt-brep")]
    Native { source: om9_core::transaction::CapturedNativeBrepSource, tolerance: Result<f64> },
    #[cfg(feature = "occt-brep")]
    Surface { geometry: om9_core::geometry::Geometry, trim: Option<om9_core::model::Extension>, tolerance: Result<f64> },
    #[cfg(feature = "occt-brep")]
    Sources { sources: Vec<OwnedSnapSource>, tolerance: Result<f64> },
}

// Source allocations must drop before the reservation on spawn failure too.
struct SurfaceWorkerInput {
    source: Option<SurfaceWorkerSource>,
    occlusion: Option<SurfaceOcclusionInput>,
    reservation: Option<Arc<std::sync::Mutex<om9_core::transaction::ExternalMemoryReservation>>>,
}
// A Drop type is captured as a whole, preserving field drop order if spawning fails.
impl Drop for SurfaceWorkerInput { fn drop(&mut self) {} }

/// Own the immutable display buffers and their admission independently of Document.
pub(super) struct SurfaceOcclusionInput {
    scene: om9_scene::scene::OcclusionSnapshot,
    tolerance: f64,
    #[cfg(feature = "occt-brep")]
    fallback: Option<FallbackOcclusionSources>,
    #[cfg(feature = "occt-brep")]
    limits: om9_core::budget::Limits,
    _buffers: Arc<super::SceneBufferLease>,
    _reservation: om9_core::transaction::ExternalMemoryReservation,
}

impl SurfaceOcclusionInput {
    pub(super) fn edge_occluded(&self, camera: &Camera, viewport: Viewport,
        candidate: &SnapCandidate) -> Result<bool> {
        self.scene.visible_display_occludes_before(camera, viewport, candidate.screen, candidate.world, self.tolerance)
    }
}

#[cfg(feature = "occt-brep")]
struct FallbackOcclusionSource {
    root: om9_core::identity::ObjectId,
    path: Vec<om9_core::identity::ObjectId>,
    leaf: om9_core::identity::ObjectId,
    geometry: om9_core::geometry::Geometry,
    trim: om9_core::model::Extension,
}

#[cfg(feature = "occt-brep")]
struct FallbackOcclusionSources {
    sources: Vec<FallbackOcclusionSource>,
    // Drop source allocations before releasing their reservation, including on errors.
    reservation: om9_core::transaction::ExternalMemoryReservation,
}

#[cfg(feature = "occt-brep")]
fn fallback_surface_proven_clear_source(geometry: &om9_core::geometry::Geometry,
    extension: &om9_core::model::Extension, transform: om9_core::geometry::Transform,
    camera: &Camera, viewport: Viewport, cursor: [f64; 2], target: [f64; 3],
    tolerance: f64, limits: &om9_core::budget::Limits) -> bool {
    let check = || -> Result<bool> {
        let om9_core::geometry::Geometry::Surface(surface) = geometry else { return Ok(false) };
        let is_plane = super::wireframe::is_affine_trim_plane(surface);
        if !is_plane && certified_fallback_cylinder(surface, limits).is_none() { return Ok(false); }
        if extension.payload.len() > om9_core::model::trim_surface_payload_limit(limits) { return Ok(false); }
        let trim=om9_core::rhino_semantics::TrimmedSurface::from_extension(extension,limits)?;
        if trim.underlying!=*surface {return Ok(false);}
        if !is_plane {
            let ray=camera.ray(cursor,viewport)?;
            return certified_cylinder_front_clear(surface,geometry,&trim,transform,&ray,target,tolerance,limits);
        }
        let p0=transform.point(surface.poles[0])?;let pu=transform.point(surface.poles[2])?;let pv=transform.point(surface.poles[1])?;
        let u=std::array::from_fn::<_,3,_>(|i|pu[i]-p0[i]);let v=std::array::from_fn::<_,3,_>(|i|pv[i]-p0[i]);
        let length=|p:[f64;3]|p.into_iter().map(|x|x*x).sum::<f64>().sqrt();
        let (lu,lv)=(length(u),length(v));let normal=[u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]];
        let sin_angle=length(normal)/(lu*lv);
        if !lu.is_finite() || !lv.is_finite() || lu<=1e-6 || lv<=1e-6 || !sin_angle.is_finite() || sin_angle<=1e-6 {return Ok(false);}
        let ray=camera.ray(cursor,viewport)?;
        let incidence=normal.iter().zip(ray.direction).map(|(a,b)|a*b).sum::<f64>().abs()/(length(normal)*length(ray.direction));
        if !incidence.is_finite() || incidence<=1e-6 {return Ok(false);}
        let hits=om9_kernel::measurement::surface_ray_hits_uv(geometry,transform,ray.origin,ray.direction,tolerance,limits)
            .map_err(|e|Error::Io(format!("trim occlusion source: {e}")))?;
        if hits.len()!=1 {return Ok(false);}
        let hit=hits[0];let target_distance=length(std::array::from_fn(|i|target[i]-ray.origin[i]));
        if !target_distance.is_finite() || hit.distance>target_distance+tolerance {return Ok(false);}
        let expected=transform.point(om9_core::surface_kernel::evaluate(surface,hit.uv[0],hit.uv[1],limits)?)?;
        if length(std::array::from_fn(|i|expected[i]-hit.point[i]))>tolerance.min(1e-6) {return Ok(false);}
        let scale=p0.into_iter().chain(pu).chain(pv).chain(ray.origin).chain(target).chain(hit.point).map(f64::abs).fold(1_f64,f64::max);
        let guard=(4.*tolerance+1024.*f64::EPSILON*scale)/(lu.min(lv)*sin_angle*incidence);
        if !guard.is_finite() {return Ok(false);}
        if (0..2).any(|axis| {
            let lo=trim.outer.iter().map(|p|p[axis]).fold(f64::INFINITY,f64::min);
            let hi=trim.outer.iter().map(|p|p[axis]).fold(f64::NEG_INFINITY,f64::max);
            let span=surface.knots[axis][surface.size[axis]]-surface.knots[axis][surface.degree[axis]];
            let margin=guard*span.abs();
            hit.uv[axis]<lo-margin || hit.uv[axis]>hi+margin
        }) {return Ok(true);}
        let uv_guard=guard*(0..2).map(|axis|(surface.knots[axis][surface.size[axis]]
            -surface.knots[axis][surface.degree[axis]]).abs()).fold(0_f64,f64::max);
        if !uv_guard.is_finite() || !trim.inners.iter().any(|hole|trim_point_inside_with_margin(hole,hit.uv,uv_guard))
            || om9_core::rhino_semantics::validate_trim_loop_topology_for_source_pick(&trim).is_err()
            || trim.inners.iter().flatten().any(|&point|
                om9_core::rhino_semantics::classify_trim_loop_uv(&trim.outer,point).ok()!=Some(Some(true))) {
            return Ok(false);
        }
        Ok(true)
    };
    check().unwrap_or(false)
}

impl Controller {
    /// Capture display and any fallback source copies under separate admission.
    pub(super) fn capture_surface_occlusion(&self) -> Result<Option<SurfaceOcclusionInput>> {
        use om9_core::budget::memory::MemoryClass;
        let scene = self.scene().ok_or(Error::Missing("surface occlusion scene"))?;
        #[cfg(not(feature = "occt-brep"))]
        if scene.objects.iter().any(|object| object.visible && object.buffers.is_trim_display_fallback()) { return Ok(None); }
        let physical = om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes))
            .ok_or(Error::Unsupported("physical RAM availability is unavailable"))?;
        #[cfg(feature = "occt-brep")]
        let fallback = {
            let mut count = 0usize;
            let mut copy_bytes = 0u64;
            let mut scratch_bytes = 0u64;
            for object in scene.objects.iter().filter(|object| object.visible && object.buffers.is_trim_display_fallback()) {
                let entity = scene.entity_for(&scene.document, object)?;
                let Some(extension) = entity.extensions.get(om9_core::model::TRIM_SURFACE_EXTENSION_SLOT) else { continue };
                if !matches!(entity.geometry, om9_core::geometry::Geometry::Surface(_))
                    || extension.payload.len() > om9_core::model::trim_surface_payload_limit(self.coordinator().limits()) { continue; }
                count = count.checked_add(1).ok_or(Error::Overflow("occlusion fallback sources"))?;
                let path = object.path.len().checked_mul(std::mem::size_of::<om9_core::identity::ObjectId>())
                    .and_then(|n| n.checked_mul(2)).ok_or(Error::Overflow("occlusion fallback sources"))?;
                copy_bytes = copy_bytes.checked_add(surface_copy_budget(entity)?)
                    .and_then(|n| n.checked_add(path as u64)).ok_or(Error::Overflow("occlusion fallback sources"))?;
                let scratch = extension.payload.len().checked_mul(64).and_then(|n| n.checked_add(8192))
                    .ok_or(Error::Overflow("occlusion fallback scratch"))?;
                scratch_bytes = scratch_bytes.max(u64::try_from(scratch).map_err(|_| Error::Overflow("occlusion fallback scratch"))?);
            }
            if count == 0 { None } else {
                let slots = count.checked_mul(std::mem::size_of::<FallbackOcclusionSource>()).and_then(|n| n.checked_mul(2))
                    .ok_or(Error::Overflow("occlusion fallback sources"))?;
                let bytes = copy_bytes.checked_add(slots as u64).and_then(|n| n.checked_add(scratch_bytes))
                    .ok_or(Error::Overflow("occlusion fallback sources"))?;
                let reservation = self.coordinator().reserve_external_memory_with_held(MemoryClass::Kernel, bytes,
                    Some(physical), self.scene_cache_usage()?)?;
                let mut captured = FallbackOcclusionSources { sources: Vec::new(), reservation };
                captured.reservation.recheck_pending()?;
                captured.sources.try_reserve_exact(count).map_err(|_| Error::Budget("occlusion fallback sources"))?;
                captured.reservation.mark_allocated(u64::try_from(captured.sources.capacity().checked_mul(std::mem::size_of::<FallbackOcclusionSource>())
                    .ok_or(Error::Overflow("occlusion fallback sources"))?).map_err(|_| Error::Overflow("occlusion fallback sources"))?)?;
                for object in scene.objects.iter().filter(|object| object.visible && object.buffers.is_trim_display_fallback()) {
                    let entity = scene.entity_for(&scene.document, object)?;
                    let Some(extension) = entity.extensions.get(om9_core::model::TRIM_SURFACE_EXTENSION_SLOT) else { continue };
                    if !matches!(entity.geometry, om9_core::geometry::Geometry::Surface(_))
                        || extension.payload.len() > om9_core::model::trim_surface_payload_limit(self.coordinator().limits()) { continue; }
                    captured.reservation.recheck_pending()?;
                    let (geometry, trim, actual) = copy_surface_source(entity)?;
                    let mut path = Vec::new();
                    path.try_reserve_exact(object.path.len()).map_err(|_| Error::Budget("occlusion fallback path"))?;
                    path.extend_from_slice(&object.path);
                    let path_bytes = path.capacity().checked_mul(std::mem::size_of::<om9_core::identity::ObjectId>())
                        .ok_or(Error::Overflow("occlusion fallback path"))?;
                    captured.reservation.mark_allocated(actual.checked_add(path_bytes as u64).ok_or(Error::Overflow("occlusion fallback sources"))?)?;
                    captured.sources.push(FallbackOcclusionSource { root: object.root, path, leaf: object.leaf,
                        geometry, trim: trim.ok_or(Error::Missing("occlusion fallback trim"))? });
                }
                if captured.reservation.pending_bytes() < scratch_bytes { return Err(Error::Budget("occlusion fallback scratch")); }
                Some(captured)
            }
        };
        let buffers = self.share_scene_buffer_owner_with_sample(scene, Some(physical))?;
        let overhead = std::mem::size_of::<SurfaceOcclusionInput>() as u64
            + om9_scene::scene::PickSnapshot::additional_metadata_bytes();
        let weak_receipts = scene.pick_snapshot_weak_receipt_bytes()?;
        let scratch = scene.render_snapshot_pick_scratch_estimate()?.bytes;
        let bytes = scene.render_snapshot_metadata_estimate()?.bytes.checked_add(overhead)
            .and_then(|n| n.checked_add(weak_receipts))
            .and_then(|n| n.checked_add(scratch))
            .ok_or(Error::Overflow("surface occlusion admission"))?;
        let mut reservation = self.coordinator().reserve_external_memory_with_held(
            MemoryClass::CpuCache, bytes, Some(physical), self.scene_cache_usage()?)?;
        reservation.recheck_pending()?;
        let snapshot = scene.try_render_snapshot()?;
        let weak_receipts = snapshot.pick_snapshot_weak_receipt_bytes()?;
        let scratch = snapshot.render_snapshot_pick_scratch_estimate()?.bytes;
        let required = snapshot.metadata_storage_usage()?.bytes.checked_add(overhead)
            .and_then(|n| n.checked_add(weak_receipts))
            .and_then(|n| n.checked_add(scratch))
            .ok_or(Error::Overflow("surface occlusion admission"))?;
        if required > reservation.bytes() {
            let additional = required - reservation.bytes();
            self.coordinator().grow_external_memory_with_held(&mut reservation, additional,
                om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes)), self.scene_cache_usage()?)?;
        }
        reservation.shrink_to(required)?;
        snapshot.transfer_memory_allowance(&mut reservation, self.coordinator(), self.scene_cache_usage()?)?;
        let scene = snapshot.into_occlusion_snapshot()?;
        reservation.mark_allocated(overhead.checked_add(weak_receipts).ok_or(Error::Overflow("surface occlusion admission"))?)?;
        Ok(Some(SurfaceOcclusionInput { scene, tolerance: self.document().effective_tolerance()?.absolute_mm,
            #[cfg(feature = "occt-brep")] fallback,
            #[cfg(feature = "occt-brep")] limits: *self.coordinator().limits(),
            _buffers: buffers,
            _reservation: reservation }))
    }
}

#[cfg(feature = "occt-brep")]
enum OwnedSnapSource {
    Surface { geometry: om9_core::geometry::Geometry, trim: Option<om9_core::model::Extension> },
    Brep(om9_core::transaction::CapturedBrepSource),
    Native(om9_core::transaction::CapturedNativeBrepSource),
}

#[cfg(feature = "occt-brep")]
fn mixed_source_copy_budget(document: &om9_core::model::Document, candidates: &[SnapCandidate]) -> Result<Option<u64>> {
    let slots = candidates.len().checked_mul(std::mem::size_of::<OwnedSnapSource>())
        .and_then(|bytes| bytes.checked_mul(2)).ok_or(Error::Overflow("surface batch sources"))?;
    let mut total = u64::try_from(slots).map_err(|_| Error::Overflow("surface batch sources"))?;
    for candidate in candidates {
        let entity = captured_entity(document, candidate)?;
        match &entity.geometry {
            om9_core::geometry::Geometry::Surface(_) => {
                total = total.checked_add(surface_copy_budget(entity)?).ok_or(Error::Overflow("surface batch sources"))?;
            }
            om9_core::geometry::Geometry::Brep(_) | om9_core::geometry::Geometry::NativeBrep(_) => {}
            _ => return Ok(None),
        }
    }
    Ok(Some(total))
}

#[cfg(feature = "occt-brep")]
fn admitted_mixed_source_budget(document: &om9_core::model::Document, candidates: &[SnapCandidate],
    minimum: u64, pending: u64) -> Result<Option<u64>> {
    let Some(bytes) = mixed_source_copy_budget(document, candidates)? else { return Ok(None) };
    let needed = minimum.checked_add(bytes).ok_or(Error::Overflow("surface batch admission"))?;
    Ok((needed <= pending).then_some(bytes))
}

#[cfg(feature = "occt-brep")]
fn single_surface_source<'a>(document: &'a om9_core::model::Document, candidates: &[SnapCandidate])
    -> Result<Option<&'a om9_core::model::Entity>> {
    let Some(first) = candidates.first().and_then(|candidate| candidate.surface_source.as_ref()) else { return Ok(None) };
    if candidates.iter().any(|candidate| candidate.surface_source.as_ref().is_none_or(|source|
        candidate.root != Some(first.source.object) || source.source != first.source || source.path != first.path
            || source.leaf != first.leaf || source.leaf_generation != first.leaf_generation
            || source.leaf_geometry_revision != first.leaf_geometry_revision || source.placement != first.placement)) {
        return Ok(None);
    }
    let entity = captured_entity(document, &candidates[0])?;
    Ok(matches!(entity.geometry, om9_core::geometry::Geometry::Surface(_)).then_some(entity))
}

#[cfg(feature = "occt-brep")]
fn surface_copy_budget(entity: &om9_core::model::Entity) -> Result<u64> {
    use om9_core::geometry::{Geometry, Point3};
    let Geometry::Surface(surface) = &entity.geometry else { return Err(Error::Invalid("surface source")) };
    let knots = surface.knots[0].len().checked_add(surface.knots[1].len()).ok_or(Error::Overflow("surface source"))?;
    let doubles = knots.checked_add(surface.weights.len()).ok_or(Error::Overflow("surface source"))?;
    let bytes = surface.poles.len().checked_mul(std::mem::size_of::<Point3>())
        .and_then(|n| doubles.checked_mul(std::mem::size_of::<f64>()).and_then(|m| n.checked_add(m)))
        .and_then(|n| n.checked_add(entity.extensions.get(om9_core::model::TRIM_SURFACE_EXTENSION_SLOT).map_or(0, |e| e.payload.len())))
        .and_then(|n| n.checked_mul(2))
        .and_then(|n| n.checked_add(std::mem::size_of::<SurfaceWorkerSource>() + 2 * std::mem::size_of::<usize>()))
        .ok_or(Error::Overflow("surface source"))?;
    u64::try_from(bytes).map_err(|_| Error::Overflow("surface source"))
}

#[cfg(feature = "occt-brep")]
fn copy_surface_source(entity: &om9_core::model::Entity) -> Result<(om9_core::geometry::Geometry, Option<om9_core::model::Extension>, u64)> {
    use om9_core::geometry::{Geometry, NurbsSurface};
    fn copy<T: Copy>(src: &[T]) -> Result<Vec<T>> {
        let mut result = Vec::new();
        result.try_reserve_exact(src.len()).map_err(|_| Error::Budget("surface source allocation"))?;
        result.extend_from_slice(src);
        Ok(result)
    }
    let Geometry::Surface(src) = &entity.geometry else { return Err(Error::Invalid("surface source")) };
    let surface = NurbsSurface { degree: src.degree, size: src.size, poles: copy(&src.poles)?,
        weights: copy(&src.weights)?, knots: [copy(&src.knots[0])?, copy(&src.knots[1])?],
        domain: src.domain, periodic: src.periodic, reversed: src.reversed };
    let trim = entity.extensions.get(om9_core::model::TRIM_SURFACE_EXTENSION_SLOT).map(|extension| {
        let mut payload = String::new();
        payload.try_reserve_exact(extension.payload.len()).map_err(|_| Error::Budget("surface trim allocation"))?;
        payload.push_str(&extension.payload);
        Ok::<_, Error>(om9_core::model::Extension { version: extension.version, payload })
    }).transpose()?;
    let actual = surface.poles.capacity().checked_mul(std::mem::size_of::<om9_core::geometry::Point3>())
        .and_then(|n| surface.weights.capacity().checked_add(surface.knots[0].capacity())
            .and_then(|v| v.checked_add(surface.knots[1].capacity()))
            .and_then(|v| v.checked_mul(std::mem::size_of::<f64>())).and_then(|v| n.checked_add(v)))
        .and_then(|n| n.checked_add(trim.as_ref().map_or(0, |e| e.payload.capacity())))
        .ok_or(Error::Overflow("surface source"))?;
    if u64::try_from(actual).map_err(|_| Error::Overflow("surface source"))? > surface_copy_budget(entity)? {
        return Err(Error::Budget("surface source capacity"));
    }
    Ok((Geometry::Surface(surface), trim, actual as u64))
}

impl SurfaceSnapJob {
    fn validate_request(
        controller: &Controller,
        candidates: &Vec<SnapCandidate>,
        camera: &Camera,
        viewport: Viewport,
        cursor: [f64; 2],
        reservation: &om9_core::transaction::ExternalMemoryReservation,
    ) -> Result<u64> {
        use om9_core::budget::memory::MemoryClass;
        if reservation.class() != MemoryClass::Kernel
            || !reservation.belongs_to(&controller.coordinator().external_memory_account()) {
            return Err(Error::Invalid("surface snap worker admission"));
        }
        let minimum = Self::request_minimum(controller, candidates, camera, viewport, cursor)?;
        if reservation.pending_bytes() < minimum {
            return Err(Error::Budget("surface snap worker candidates"));
        }
        Ok(minimum)
    }

    /// Required Rust-side storage for this request, before optional native work.
    /// This is an admission floor, not a bound on OCCT's internal allocations.
    pub fn request_minimum(
        controller: &Controller, candidates: &Vec<SnapCandidate>, camera: &Camera,
        viewport: Viewport, cursor: [f64; 2],
    ) -> Result<u64> {
        if candidates.is_empty() || candidates.len() as u64 > controller.coordinator().limits().max_outputs {
            return Err(Error::Invalid("surface snap worker admission"));
        }
        // Retain the actual input allocation and make a result candidate for
        // each input. Each cloned source owns a copy of its instance path.
        // This is a Rust storage floor, not an upper bound on allocator or OCCT work.
        let overflow = || Error::Overflow("surface snap worker candidates");
        let slots = candidates.capacity().checked_add(candidates.len()).ok_or_else(overflow)?;
        let mut minimum = slots.checked_mul(std::mem::size_of::<SnapCandidate>()).ok_or_else(overflow)?;
        for candidate in candidates {
            let paths = candidate_path_entries(candidate, true)?;
            minimum = minimum.checked_add(paths.checked_mul(std::mem::size_of::<om9_core::identity::ObjectId>()).ok_or_else(overflow)?).ok_or_else(overflow)?;
        }
        let mut minimum = u64::try_from(minimum).map_err(|_| overflow())?
            .checked_add(surface_receipt_fixed_bytes()?).ok_or_else(overflow)?;
        for candidate in candidates {
            minimum = minimum.checked_add(candidate_weak_owner_bytes(candidate).checked_mul(2).ok_or_else(overflow)?)
                .ok_or_else(overflow)?;
        }
        let mut trim_scratch=0u64;
        for candidate in candidates {
            trim_scratch=trim_scratch.max(trim_snap_scratch(controller.document(),candidate)?);
        }
        minimum=minimum.checked_add(trim_scratch).ok_or_else(overflow)?;
        #[cfg(feature = "occt-brep")]
        if let Some(entity) = single_surface_source(controller.document(), candidates)? {
            minimum = minimum.checked_add(surface_copy_budget(entity)?).ok_or_else(overflow)?;
        }
        for candidate in candidates {
            controller.validate_surface_candidate(candidate, camera, viewport)?;
            if candidate.surface_source.as_ref().is_none_or(|source| source.cursor != cursor) {
                return Err(Error::Stale("surface snap cursor"));
            }
        }
        Ok(minimum)
    }

    /// Rust storage floor for retaining only the requested sources. `None`
    /// means this batch needs the current tracked-document fallback. The
    /// native kernel's internal allocations are not bounded by this value.
    #[cfg(feature = "occt-brep")]
    pub fn source_only_minimum(
        controller: &Controller, candidates: &Vec<SnapCandidate>, camera: &Camera,
        viewport: Viewport, cursor: [f64; 2],
    ) -> Result<Option<u64>> {
        let minimum = Self::request_minimum(controller, candidates, camera, viewport, cursor)?;
        if single_surface_source(controller.document(), candidates)?.is_some() {
            return Ok(Some(minimum));
        }
        let first = &candidates[0];
        if candidates.iter().all(|candidate| same_batch_source(first, candidate))
            && matches!(captured_entity(controller.document(), first)?.geometry, om9_core::geometry::Geometry::Brep(_) | om9_core::geometry::Geometry::NativeBrep(_)) {
            return Ok(Some(minimum));
        }
        mixed_source_copy_budget(controller.document(), candidates)?
            .map(|extra| minimum.checked_add(extra).ok_or(Error::Overflow("surface batch admission")))
            .transpose()
    }

    pub fn start(
        controller: &Controller,
        candidates: Vec<SnapCandidate>,
        camera: Camera,
        viewport: Viewport,
        cursor: [f64; 2],
        reservation: om9_core::transaction::ExternalMemoryReservation,
    ) -> Result<Self> {
        Self::validate_request(controller, &candidates, &camera, viewport, cursor, &reservation)?;
        let permit = controller.coordinator().external_memory_account().try_acquire_surface_snap()?;
        Self::start_with_permit(controller, candidates, camera, viewport, cursor, reservation, permit,
            #[cfg(test)] None)
    }

    fn start_with_permit(
        controller: &Controller,
        candidates: Vec<SnapCandidate>,
        camera: Camera,
        viewport: Viewport,
        cursor: [f64; 2],
        reservation: om9_core::transaction::ExternalMemoryReservation,
        permit: om9_core::transaction::ExternalSurfaceSnapPermit,
        #[cfg(test)] gate: Option<mpsc::Receiver<()>>,
    ) -> Result<Self> {
        Self::start_with_permit_and_occlusion(controller, candidates, camera, viewport, cursor,
            reservation, permit, None, #[cfg(test)] gate)
    }

    fn start_with_permit_and_occlusion(
        controller: &Controller,
        candidates: Vec<SnapCandidate>,
        camera: Camera,
        viewport: Viewport,
        cursor: [f64; 2],
        reservation: om9_core::transaction::ExternalMemoryReservation,
        permit: om9_core::transaction::ExternalSurfaceSnapPermit,
        occlusion: Option<SurfaceOcclusionInput>,
        #[cfg(test)] gate: Option<mpsc::Receiver<()>>,
    ) -> Result<Self> {
        #[cfg(feature = "occt-brep")]
        let mut reservation = reservation;
        let minimum = Self::validate_request(controller, &candidates, &camera, viewport, cursor, &reservation)?;
        #[cfg(not(feature = "occt-brep"))]
        let _ = minimum;
        let source_document = Arc::downgrade(&controller.scene().ok_or(Error::Missing("surface snap scene"))?.document);
        let worker_source_document = source_document.clone();
        let current_document = controller.coordinator().world().documents.get(&controller.document().id)
            .ok_or(Error::Missing("surface snap document"))?;
        if !source_document.ptr_eq(&Arc::downgrade(current_document)) {
            return Err(Error::Stale("surface snap scene document"));
        }
        #[cfg(feature = "occt-brep")]
        let source = {
            let scene = controller.scene().ok_or(Error::Missing("surface snap scene"))?;
            let first = &candidates[0];
            let eligible = candidates.iter().all(|candidate| same_batch_source(first, candidate));
            let occurrence = first.surface_source.as_ref().and_then(|source| scene.objects.iter().find(|object| {
                object.root == source.source.object && object.path == source.path && object.leaf == source.leaf
                    && object.source == source.source && object.brep_display_fingerprint == source.brep_display_fingerprint
            }));
            if let Some(object) = occurrence.filter(|object| eligible && matches!(object.brep_display_fingerprint, Some(om9_scene::scene::BrepDisplayReceipt::Occt(_)))) {
                let lease = scene.capture_brep_source_for(controller.coordinator(), object)?;
                SurfaceWorkerSource::Brep { source: lease, tolerance: controller.document().effective_tolerance().map(|value| value.absolute_mm) }
            } else if let Some(object) = occurrence.filter(|object| eligible && matches!(object.brep_display_fingerprint, Some(om9_scene::scene::BrepDisplayReceipt::Native { .. }))) {
                let lease = scene.capture_native_brep_source_for(controller.coordinator(), object)?;
                SurfaceWorkerSource::Native { source: lease, tolerance: controller.document().effective_tolerance().map(|value| value.absolute_mm) }
            } else if let Some(entity) = single_surface_source(controller.document(), &candidates)? {
                reservation.recheck_pending()?;
                let (geometry, trim, allocated) = copy_surface_source(entity)?;
                reservation.mark_allocated(allocated)?;
                SurfaceWorkerSource::Surface { geometry, trim,
                    tolerance: controller.document().effective_tolerance().map(|value| value.absolute_mm) }
            } else if let Some(copy_budget) = admitted_mixed_source_budget(controller.document(), &candidates,
                minimum, reservation.pending_bytes())? {
                reservation.recheck_pending()?;
                let mut sources = Vec::new();
                sources.try_reserve_exact(candidates.len()).map_err(|_| Error::Budget("surface batch allocation"))?;
                let slots = sources.capacity().checked_mul(std::mem::size_of::<OwnedSnapSource>())
                    .ok_or(Error::Overflow("surface batch allocation"))?;
                let slots = u64::try_from(slots).map_err(|_| Error::Overflow("surface batch allocation"))?;
                let slot_limit = candidates.len().checked_mul(std::mem::size_of::<OwnedSnapSource>())
                    .and_then(|bytes| bytes.checked_mul(2)).ok_or(Error::Overflow("surface batch allocation"))?;
                if slots > u64::try_from(slot_limit).map_err(|_| Error::Overflow("surface batch allocation"))?
                    || slots > copy_budget { return Err(Error::Budget("surface batch allocation")); }
                reservation.mark_allocated(slots)?;
                for candidate in &candidates {
                    reservation.recheck_pending()?;
                    let entity = captured_entity(controller.document(), candidate)?;
                    match &entity.geometry {
                        om9_core::geometry::Geometry::Surface(_) => {
                            let (geometry, trim, bytes) = copy_surface_source(entity)?;
                            reservation.mark_allocated(bytes)?;
                            sources.push(OwnedSnapSource::Surface { geometry, trim });
                        }
                        om9_core::geometry::Geometry::Brep(_) => {
                            let reference = candidate.surface_source.as_ref().ok_or(Error::Missing("surface snap source"))?;
                            let object = scene.objects.iter().find(|object| object.root == reference.source.object
                                && object.path == reference.path && object.leaf == reference.leaf
                                && object.source == reference.source && object.brep_display_fingerprint == reference.brep_display_fingerprint)
                                .ok_or(Error::Stale("surface snap scene occurrence"))?;
                            sources.push(OwnedSnapSource::Brep(scene.capture_brep_source_for(controller.coordinator(), object)?));
                        }
                        om9_core::geometry::Geometry::NativeBrep(_) => {
                            let reference = candidate.surface_source.as_ref().ok_or(Error::Missing("surface snap source"))?;
                            let object = scene.objects.iter().find(|object| object.root == reference.source.object
                                && object.path == reference.path && object.leaf == reference.leaf
                                && object.source == reference.source && object.brep_display_fingerprint == reference.brep_display_fingerprint)
                                .ok_or(Error::Stale("surface snap scene occurrence"))?;
                            sources.push(OwnedSnapSource::Native(scene.capture_native_brep_source_for(controller.coordinator(), object)?));
                        }
                        _ => return Err(Error::Stale("surface snap batch source")),
                    }
                }
                SurfaceWorkerSource::Sources { sources,
                    tolerance: controller.document().effective_tolerance().map(|value| value.absolute_mm) }
            } else {
                SurfaceWorkerSource::Document(controller.coordinator().capture_tracked_document(controller.document().id)?)
            }
        };
        #[cfg(not(feature = "occt-brep"))]
        let source = SurfaceWorkerSource::Document(controller.coordinator().capture_tracked_document(controller.document().id)?);
        #[cfg(all(test, feature = "occt-brep"))]
        let source_is_owned_surface = matches!(&source, SurfaceWorkerSource::Surface { .. } | SurfaceWorkerSource::Sources { .. });
        #[cfg(all(test, not(feature = "occt-brep")))]
        let source_is_owned_surface = false;
        let limits = *controller.coordinator().limits();
        let selection_revision = controller.selection_state().revision;
        let preview_revision = controller.preview_revision();
        let candidates = Arc::new(candidates);
        let worker_candidates = Arc::clone(&candidates);
        let worker_camera = camera.clone();
        let cancel = Arc::new(AtomicBool::new(false));
        let worker_cancel = Arc::clone(&cancel);
        let (sender, receiver) = mpsc::sync_channel(1);
        let reservation = Arc::new(std::sync::Mutex::new(reservation));
        let input = SurfaceWorkerInput { source: Some(source), occlusion, reservation: Some(Arc::clone(&reservation)) };
        std::thread::Builder::new().name("om9-surface-snap".into()).spawn(move || {
            let mut input = input;
            let source = input.source.take().expect("surface worker source");
            let occlusion = input.occlusion.take();
            let reservation = input.reservation.take().expect("surface worker reservation");
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                #[cfg(test)]
                if let Some(gate) = gate { gate.recv().map_err(|_| Error::Canceled)?; }
                #[cfg(feature = "occt-brep")]
                let worker_baseline = {
                    let owner = reservation.lock().unwrap_or_else(|error| error.into_inner());
                    minimum.checked_add(owner.bytes().saturating_sub(owner.pending_bytes()))
                        .ok_or(Error::Overflow("surface worker baseline"))?
                };
                let mut refined = Vec::new();
                refined.try_reserve_exact(worker_candidates.len()).map_err(|_| Error::Budget("surface snap results"))?;
                let mut index = 0;
                while index < worker_candidates.len() {
                    if worker_cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
                    let mut end = index + 1;
                    while end < worker_candidates.len() && end - index < 8
                        && same_batch_source(&worker_candidates[index], &worker_candidates[end]) {
                        end += 1;
                    }
                    match &source {
                        #[cfg(feature = "occt-brep")]
                        SurfaceWorkerSource::Brep { source, tolerance } => {
                            let mut group = [&worker_candidates[index]; 8];
                            for (slot, candidate) in group.iter_mut().zip(&worker_candidates[index..end]) { *slot = candidate; }
                            let tolerance = tolerance.as_ref().map_err(Clone::clone)?;
                            refined.extend(refine_leased_brep_group(source, &group[..end-index], &worker_camera, viewport, *tolerance, &limits)?.into_iter().flatten());
                        }
                        #[cfg(feature = "occt-brep")]
                        SurfaceWorkerSource::Native { source, tolerance } => {
                            let tolerance = *tolerance.as_ref().map_err(Clone::clone)?;
                            refine_leased_native_group(source, &worker_candidates[index..end], &worker_camera, viewport,
                                tolerance, &limits, &reservation, worker_baseline, &worker_cancel, &mut refined)?;
                        }
                        #[cfg(feature = "occt-brep")]
                        SurfaceWorkerSource::Surface { geometry, trim, tolerance } => {
                            let tolerance = *tolerance.as_ref().map_err(Clone::clone)?;
                            for candidate in &worker_candidates[index..end] {
                                if let Some(result) = try_refine_surface_geometry(geometry, trim.as_ref(), tolerance,
                                    candidate, &worker_camera, viewport, &limits)? { refined.push(result); }
                            }
                        }
                        #[cfg(feature = "occt-brep")]
                        SurfaceWorkerSource::Sources { sources, tolerance } => {
                            let tolerance = *tolerance.as_ref().map_err(Clone::clone)?;
                            match sources.get(index).ok_or(Error::Invalid("surface batch source"))? {
                                OwnedSnapSource::Surface { .. } => for (offset, candidate) in worker_candidates[index..end].iter().enumerate() {
                                    let Some(OwnedSnapSource::Surface { geometry, trim }) = sources.get(index + offset) else {
                                        return Err(Error::Invalid("surface batch source type"));
                                    };
                                    if let Some(result) = try_refine_surface_geometry(geometry, trim.as_ref(), tolerance,
                                        candidate, &worker_camera, viewport, &limits)? { refined.push(result); }
                                },
                                OwnedSnapSource::Brep(source) => {
                                    let mut group = [&worker_candidates[index]; 8];
                                    for (slot, candidate) in group.iter_mut().zip(&worker_candidates[index..end]) { *slot = candidate; }
                                    refined.extend(refine_leased_brep_group(source, &group[..end-index], &worker_camera,
                                        viewport, tolerance, &limits)?.into_iter().flatten());
                                }
                                OwnedSnapSource::Native(source) => {
                                    refine_leased_native_group(source, &worker_candidates[index..end], &worker_camera, viewport,
                                        tolerance, &limits, &reservation, worker_baseline, &worker_cancel, &mut refined)?;
                                }
                            }
                        }
                        SurfaceWorkerSource::Document(document) => {
                            if end - index > 1 {
                                let mut group = [&worker_candidates[index]; 8];
                                for (slot, candidate) in group.iter_mut().zip(&worker_candidates[index..end]) { *slot = candidate; }
                                refined.extend(refine_captured_group(document, &group[..end-index], &worker_camera, viewport, &limits)?.into_iter().flatten());
                            } else if let Some(candidate) = try_refine_captured_surface(document, &worker_candidates[index], &worker_camera, viewport, &limits)? {
                                refined.push(candidate);
                            }
                        }
                    }
                    index = end;
                }
                if worker_cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
                let display_occluded = if let Some(snapshot) = &occlusion {
                    if let Some(candidate) = om9_scene::osnap::choose_ref(refined.iter()) {
                        Some(snapshot.scene.visible_display_occludes_with_fallback_proof(&worker_camera, viewport,
                            cursor, candidate.world, snapshot.tolerance, &[], candidate.surface_source.as_ref(), |object| {
                                #[cfg(feature = "occt-brep")]
                                {
                                    let Some(fallback) = &snapshot.fallback else { return false };
                                    if fallback.reservation.recheck_pending().is_err() { return false; }
                                    fallback.sources.iter().find(|source| source.root == object.root
                                        && source.leaf == object.leaf && source.path == object.path).is_some_and(|source|
                                        object.display_transform().ok().is_some_and(|transform|
                                            fallback_surface_proven_clear_source(&source.geometry, &source.trim, transform,
                                                &worker_camera, viewport, cursor, candidate.world, snapshot.tolerance, &snapshot.limits)))
                                }
                                #[cfg(not(feature = "occt-brep"))]
                                { let _ = object; false }
                            })?)
                    } else { Some(false) }
                } else { None };
                if worker_cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
                Ok((refined, display_occluded))
            })).unwrap_or_else(|_| Err(Error::Io("surface snap worker panicked".into())));
            // Source copies/leases are no longer needed by the result. Keep
            // the receipt charged only for its own retained allocations.
            drop(source);
            drop(occlusion);
            let result = result.and_then(|(refined, display_occluded)| {
                let receipt = SurfaceSnapReceipt { document: worker_source_document, original: worker_candidates,
                    refined, camera: worker_camera, viewport, cursor, selection_revision, preview_revision,
                    display_occluded, _reservation: reservation, cancellation: Arc::clone(&worker_cancel) };
                let retained = receipt.retained_bytes()?;
                {
                    let mut owner = receipt._reservation.lock().unwrap_or_else(|e| e.into_inner());
                    if retained > owner.bytes() { return Err(Error::Budget("surface snap receipt capacity")); }
                    owner.set_allocated_bytes(retained)?;
                    owner.shrink_to(retained)?;
                }
                Ok(receipt)
            });
            // All native calls have finished. Release before notifying the
            // scheduler, so its next job can acquire the slot immediately.
            drop(permit);
            let _ = sender.send(result);
        }).map_err(|error| Error::Io(format!("surface snap worker start: {error}")))?;
        Ok(Self { document: source_document, cancel, result: Some(receiver), delivered: false, candidates, camera, viewport, cursor, selection_revision, preview_revision,
            #[cfg(test)] source_is_owned_surface, _reservation: reservation })
    }

    pub fn cancel(&self) { self.cancel.store(true, Ordering::Release); }
    pub fn try_result(&mut self) -> Option<Result<SurfaceSnapReceipt>> {
        let receiver = self.result.as_ref()?;
        match receiver.try_recv() {
            Ok(result) => { self.result = None; self.delivered = true; Some(result) }
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => {
                self.result = None;
                Some(Err(Error::Io("surface snap worker exited without a result".into())))
            }
        }
    }
}

impl Drop for SurfaceSnapJob {
    fn drop(&mut self) { if !self.delivered { self.cancel(); } }
}

struct PendingSurfaceSnap {
    document: Weak<om9_core::model::Document>,
    candidates: Vec<SnapCandidate>,
    camera: Camera,
    viewport: Viewport,
    cursor: [f64; 2],
    selection_revision: u64,
    preview_revision: u64,
    occlusion: Option<SurfaceOcclusionInput>,
    reservation: om9_core::transaction::ExternalMemoryReservation,
}

/// Bounded latest-input scheduling; UI integration and memory sizing remain separate.
#[derive(Default)]
pub struct SurfaceSnapScheduler {
    active: Option<SurfaceSnapJob>,
    obsolete: bool,
    pending: Option<PendingSurfaceSnap>,
}

impl SurfaceSnapScheduler {
    /// True only for the current, non-obsolete request. Callers can avoid
    /// reserving another worker budget when polling unchanged pointer input.
    pub fn matches_request(&self, controller: &Controller, candidates: &[SnapCandidate],
        camera: &Camera, viewport: Viewport, cursor: [f64; 2]) -> bool {
        let Some(scene) = controller.scene() else { return false };
        if candidates.iter().any(|candidate| candidate.surface_source.as_ref()
            .is_none_or(|source| source.scene_publication != scene.publication_id())) { return false; }
        let document = Arc::downgrade(&scene.document);
        let selection_revision = controller.selection_state().revision;
        let preview_revision = controller.preview_revision();
        (!self.obsolete && self.pending.is_none() && self.active.as_ref().is_some_and(|active| {
            active.document.ptr_eq(&document) && active.candidates.as_slice() == candidates && active.camera == *camera
                && active.viewport == viewport && active.cursor == cursor
                && active.selection_revision == selection_revision && active.preview_revision == preview_revision
        })) || self.pending.as_ref().is_some_and(|pending| {
            pending.document.ptr_eq(&document) && pending.candidates.as_slice() == candidates && pending.camera == *camera
                && pending.viewport == viewport && pending.cursor == cursor
                && pending.selection_revision == selection_revision && pending.preview_revision == preview_revision
        })
    }

    pub fn submit(
        &mut self,
        controller: &Controller,
        candidates: Vec<SnapCandidate>,
        camera: Camera,
        viewport: Viewport,
        cursor: [f64; 2],
        reservation: om9_core::transaction::ExternalMemoryReservation,
    ) -> Result<()> {
        self.submit_inner(controller, candidates, camera, viewport, cursor, reservation, false)
    }

    pub(crate) fn submit_with_occlusion(
        &mut self, controller: &Controller, candidates: Vec<SnapCandidate>, camera: Camera,
        viewport: Viewport, cursor: [f64; 2], reservation: om9_core::transaction::ExternalMemoryReservation,
    ) -> Result<()> {
        self.submit_inner(controller, candidates, camera, viewport, cursor, reservation, true)
    }

    fn submit_inner(
        &mut self, controller: &Controller, candidates: Vec<SnapCandidate>, camera: Camera,
        viewport: Viewport, cursor: [f64; 2], reservation: om9_core::transaction::ExternalMemoryReservation,
        capture_occlusion: bool,
    ) -> Result<()> {
        SurfaceSnapJob::validate_request(controller, &candidates, &camera, viewport, cursor, &reservation)?;
        if self.matches_request(controller, &candidates, &camera, viewport, cursor) { return Ok(()); }
        let document = Arc::downgrade(&controller.scene().ok_or(Error::Missing("surface snap scene"))?.document);
        let selection_revision = controller.selection_state().revision;
        let preview_revision = controller.preview_revision();
        // UI callers require an admitted worker snapshot. Falling back to an
        // inline source proof would run native geometry on the event thread.
        let occlusion = if capture_occlusion {
            Some(controller.capture_surface_occlusion()?.ok_or(Error::Unsupported("surface occlusion worker"))?)
        } else { None };
        let request = PendingSurfaceSnap {
            document, candidates, camera, viewport, cursor,
            selection_revision, preview_revision, occlusion, reservation,
        };
        if let Some(active) = &self.active {
            active.cancel();
            self.obsolete = true;
            self.pending = Some(request);
        } else {
            self.pending = Some(request);
            self.obsolete = false;
            self.try_launch_pending(controller)?;
        }
        Ok(())
    }

    /// Cancel queued work immediately. An in-flight native call may still finish;
    /// poll until it has exited before starting another request.
    pub fn cancel(&mut self) {
        self.pending = None;
        if let Some(active) = &self.active {
            active.cancel();
            self.obsolete = true;
        }
    }

    pub fn is_busy(&self) -> bool { self.active.is_some() || self.pending.is_some() }

    fn try_launch_pending(&mut self, controller: &Controller) -> Result<()> {
        let Some(request) = self.pending.as_ref() else { return Ok(()) };
        if controller.scene().is_none_or(|scene| !request.document.ptr_eq(&Arc::downgrade(&scene.document)))
            || request.selection_revision != controller.selection_state().revision
            || request.preview_revision != controller.preview_revision() {
            self.pending = None;
            return Err(Error::Stale("pending surface snap context"));
        }
        let permit = match controller.coordinator().external_memory_account().try_acquire_surface_snap() {
            Ok(permit) => permit,
            Err(Error::Invalid("surface snap already active")) => return Ok(()),
            Err(error) => return Err(error),
        };
        let request = self.pending.take().expect("pending request checked above");
        self.active = Some(SurfaceSnapJob::start_with_permit_and_occlusion(controller, request.candidates, request.camera,
            request.viewport, request.cursor, request.reservation, permit, request.occlusion, #[cfg(test)] None)?);
        Ok(())
    }

    /// Nonblocking. The caller must continue polling while busy and separately
    /// validate the published frame and occlusion before using a receipt.
    pub fn poll(&mut self, controller: &Controller) -> Option<Result<SurfaceSnapReceipt>> {
        let mut completed = None;
        if let Some(result) = self.active.as_mut().and_then(SurfaceSnapJob::try_result) {
            self.active = None;
            if !self.obsolete { completed = Some(result); }
            self.obsolete = false;
        }
        if self.active.is_none() && let Err(error) = self.try_launch_pending(controller) {
            return Some(Err(error));
        }
        completed
    }
}

fn same_batch_source(a: &SnapCandidate, b: &SnapCandidate) -> bool {
    let (Some(a_source), Some(b_source)) = (&a.surface_source, &b.surface_source) else { return false };
    a.kind == SnapKind::Surface && b.kind == SnapKind::Surface && a.root == b.root
        && a_source.face.is_some() && b_source.face.is_some()
        && a_source.source == b_source.source && a_source.path == b_source.path
        && a_source.leaf == b_source.leaf && a_source.scene_epoch == b_source.scene_epoch
        && a_source.scene_publication == b_source.scene_publication
        && a_source.leaf_generation == b_source.leaf_generation
        && a_source.leaf_geometry_revision == b_source.leaf_geometry_revision
        && a_source.brep_display_fingerprint == b_source.brep_display_fingerprint
        && a_source.placement == b_source.placement && a_source.camera == b_source.camera
        && a_source.viewport == b_source.viewport && a_source.cursor == b_source.cursor
}

fn trim_snap_scratch(document: &om9_core::model::Document, candidate: &SnapCandidate) -> Result<u64> {
    let entity=captured_entity(document,candidate)?;
    if !matches!(&entity.geometry,om9_core::geometry::Geometry::Surface(_)) {return Ok(0);}
    let Some(extension)=entity.extensions.get(om9_core::model::TRIM_SURFACE_EXTENSION_SLOT) else {return Ok(0)};
    let bytes=extension.payload.len().checked_mul(64).and_then(|n|n.checked_add(8192))
        .ok_or(Error::Overflow("surface snap trim scratch"))?;
    u64::try_from(bytes).map_err(|_|Error::Overflow("surface snap trim scratch"))
}

fn captured_entity<'a>(document: &'a om9_core::model::Document, candidate: &SnapCandidate) -> Result<&'a om9_core::model::Entity> {
    let source = candidate.surface_source.as_ref().ok_or(Error::Missing("surface snap source"))?;
    captured_surface_entity(document, candidate.root.ok_or(Error::Missing("surface snap root"))?, source)
}

fn captured_surface_entity<'a>(document: &'a om9_core::model::Document, root: om9_core::identity::ObjectId,
    source: &om9_scene::osnap::SurfaceCandidateSource) -> Result<&'a om9_core::model::Entity> {
    if root != source.source.object { return Err(Error::Stale("surface snap root")); }
    document.resolve(&source.source)?;
    let mut entity = document.entities.get(&root)
        .ok_or(Error::Missing("surface snap root"))?;
    for id in &source.path {
        let om9_core::geometry::Geometry::Instance(definition) = entity.geometry else { return Err(Error::Invalid("surface snap instance path")) };
        entity = document.definitions.get(&definition).ok_or(Error::Missing("surface snap definition"))?
            .members.iter().find(|member| member.id == *id).ok_or(Error::Missing("surface snap member"))?;
    }
    if entity.id != source.leaf { return Err(Error::Stale("surface snap leaf")); }
    if entity.generation != source.leaf_generation || entity.geometry_revision != source.leaf_geometry_revision {
        return Err(Error::Stale("surface snap leaf revision"));
    }
    let fingerprint = om9_scene::scene::brep_display_fingerprint(entity);
    if fingerprint != source.brep_display_fingerprint {
        return Err(Error::Stale("surface snap BRep display source"));
    }
    Ok(entity)
}

fn typed_trim_boundary_witness(surface: &om9_core::geometry::NurbsSurface,
    trim: &om9_core::rhino_semantics::TrimmedSurface, placement: Transform, point: Point3,
    tolerance: f64, limits: &om9_core::budget::Limits, canceled: &dyn Fn() -> bool) -> Result<Option<Point3>> {
    let cost = surface.size[0].checked_mul(surface.size[1]).ok_or(Error::Overflow("surface boundary work"))?;
    let work = std::cell::Cell::new(0usize);
    for loop_ in std::iter::once(&trim.outer).chain(&trim.inners) {
        for index in 0..loop_.len() {
            if canceled() { return Err(Error::Canceled); }
            let a = loop_[index]; let b = loop_[(index + 1) % loop_.len()];
            let evaluate = |fraction: f64| -> Result<(f64, Point3)> {
                let next = work.get().checked_add(cost).ok_or(Error::Overflow("surface boundary work"))?;
                if next > 20_000_000 { return Err(Error::Budget("surface boundary work")); }
                work.set(next);
                let uv = std::array::from_fn::<_, 2, _>(|axis| a[axis] * (1. - fraction) + b[axis] * fraction);
                let witness = placement.point(om9_core::surface_kernel::evaluate(surface, uv[0], uv[1], limits)?)?;
                let distance = (0..3).map(|axis| (point[axis] - witness[axis]).powi(2)).sum::<f64>().sqrt();
                if !distance.is_finite() { return Err(Error::Invalid("surface boundary distance")); }
                Ok((distance, witness))
            };
            let mut distances = [0.; 65];
            for sample in 0..=64 {
                if sample % 16 == 0 && canceled() { return Err(Error::Canceled); }
                let (distance, witness) = evaluate(sample as f64 / 64.)?;
                if distance <= tolerance { return Ok(Some(witness)); }
                distances[sample] = distance;
            }
            for sample in 0usize..=64 {
                if sample > 0 && distances[sample] > distances[sample - 1]
                    || sample < 64 && distances[sample] > distances[sample + 1] { continue; }
                let mut low = sample.saturating_sub(1) as f64 / 64.;
                let mut high = (sample + 1).min(64) as f64 / 64.;
                for _ in 0..40 {
                    if canceled() { return Err(Error::Canceled); }
                    let left = low + (high - low) * 0.3819660112501051;
                    let right = high - (high - low) * 0.3819660112501051;
                    let (left_distance, left_witness) = evaluate(left)?;
                    let (right_distance, right_witness) = evaluate(right)?;
                    if left_distance <= tolerance { return Ok(Some(left_witness)); }
                    if right_distance <= tolerance { return Ok(Some(right_witness)); }
                    if left_distance < right_distance { high = right; } else { low = left; }
                }
            }
        }
    }
    Ok(None)
}

/// Find a source-evaluated witness for a typed point on a standalone surface.
/// A missing witness is inconclusive: the numerical projection can miss a valid
/// point or find a different branch. The caller must run this off the UI thread
/// and revalidate the source, occurrence and prompt before accepting its result.
pub(crate) fn typed_surface_witness(document: &om9_core::model::Document,
    root: om9_core::identity::ObjectId, source: &om9_scene::osnap::SurfaceCandidateSource,
    point: om9_core::geometry::Point3, limits: &om9_core::budget::Limits,
    canceled: &dyn Fn() -> bool) -> Result<Option<om9_core::geometry::Point3>> {
    if canceled() { return Err(Error::Canceled); }
    let tolerance = document.effective_tolerance()?.absolute_mm;
    if !tolerance.is_finite() || tolerance <= 0. { return Err(Error::Invalid("surface point tolerance")); }
    let entity = captured_surface_entity(document, root, source)?;
    let om9_core::geometry::Geometry::Surface(surface) = &entity.geometry else {
        return Err(Error::Unsupported("typed point on BRep surface"));
    };
    let projection = om9_core::curve_surface_ops::closest_point_on_placed_surface_cancellable(
        surface, source.placement, point, limits, canceled)?;
    if let Some(extension) = entity.extensions.get(om9_core::model::TRIM_SURFACE_EXTENSION_SLOT) {
        if extension.payload.len() > om9_core::model::trim_surface_payload_limit(limits) {
            return Err(Error::Unsupported("surface point trim metadata exceeds supported size"));
        }
        let trim = om9_core::rhino_semantics::TrimmedSurface::from_extension(extension, limits)?;
        if trim.underlying != *surface { return Err(Error::Invalid("surface point trim source mismatch")); }
        om9_core::rhino_semantics::validate_trim_loop_topology_for_source_pick(&trim)?;
        if canceled() { return Err(Error::Canceled); }
        if projection.distance.is_finite() && projection.distance <= tolerance
            && om9_core::rhino_semantics::classify_trim_uv(&trim, projection.parameters)? == Some(true) {
            return Ok(Some(projection.point));
        }
        return typed_trim_boundary_witness(surface, &trim, source.placement, point, tolerance, limits, canceled);
    }
    Ok((projection.distance.is_finite() && projection.distance <= tolerance).then_some(projection.point))
}

/// Owns a single typed-coordinate witness search and its tracked source lease.
enum TypedPointSource {
    Surface(om9_core::transaction::CapturedDocument),
    #[cfg(feature = "occt-brep")]
    Brep(om9_core::transaction::CapturedBrepSource),
    #[cfg(feature = "occt-brep")]
    Native(om9_core::transaction::CapturedNativeBrepSource),
}

pub(crate) struct TypedSurfaceJob {
    cancel: Arc<AtomicBool>,
    receiver: std::sync::mpsc::Receiver<Result<Option<om9_core::geometry::Point3>>>,
}

impl TypedSurfaceJob {
    pub(crate) fn start(controller: &Controller, root: om9_core::identity::ObjectId,
        source: &om9_scene::osnap::SurfaceCandidateSource, point: om9_core::geometry::Point3) -> Result<Self> {
        use om9_core::budget::memory::MemoryClass;
        om9_core::geometry::finite(point)?;
        let scene = controller.scene().ok_or(Error::Missing("surface point scene"))?;
        let object = scene.objects.iter().find(|object| object.root == root && object.path == source.path
            && object.leaf == source.leaf && object.source == source.source
            && object.leaf_generation == source.leaf_generation
            && object.leaf_geometry_revision == source.leaf_geometry_revision
            && object.brep_display_fingerprint == source.brep_display_fingerprint
            && object.display_transform().is_ok_and(|placement| placement == source.placement)
            && object.visible && (object.selectable || object.reference_snappable))
            .ok_or(Error::Stale("surface point target"))?;
        let entity = scene.entity_for(&scene.document, object)?;
        let limits = *controller.coordinator().limits();
        let tolerance = controller.document().effective_tolerance()?.absolute_mm;
        let (input, bytes) = match &entity.geometry {
            om9_core::geometry::Geometry::Surface(surface) => {
                let trim_bytes = entity.extensions.get(om9_core::model::TRIM_SURFACE_EXTENSION_SLOT)
                    .map_or(0, |extension| extension.payload.len());
                let bytes = surface.validate(&limits)?.bytes.checked_mul(4)
                    .and_then(|n| n.checked_add((trim_bytes as u64).checked_mul(64)?))
                    .and_then(|n| n.checked_add(1024 * 1024)).ok_or(Error::Overflow("surface point worker"))?;
                (TypedPointSource::Surface(controller.coordinator().capture_tracked_document(controller.document().id)?), bytes)
            }
            #[cfg(feature = "occt-brep")]
            om9_core::geometry::Geometry::Brep(_) => {
                let lease = scene.capture_brep_source_for(controller.coordinator(), object)?;
                let bytes = lease.canonical_storage_bytes()?.checked_mul(4)
                    .and_then(|n| n.checked_add(4 * 1024 * 1024)).ok_or(Error::Overflow("surface point worker"))?;
                (TypedPointSource::Brep(lease), bytes)
            }
            #[cfg(feature = "occt-brep")]
            om9_core::geometry::Geometry::NativeBrep(_) => {
                let lease = scene.capture_native_brep_source_for(controller.coordinator(), object)?;
                let bytes = u64::try_from(lease.payload().len()).map_err(|_| Error::Overflow("surface point native payload"))?
                    .checked_mul(64).and_then(|n| n.checked_add(16 * 1024 * 1024))
                    .ok_or(Error::Overflow("surface point worker"))?;
                (TypedPointSource::Native(lease), bytes)
            }
            _ => return Err(Error::Unsupported("typed point on this surface source")),
        };
        let mut reservation = controller.coordinator().reserve_external_memory_with_held(MemoryClass::Kernel, bytes,
            om9_native_owner::physical_memory().map(|sample| (sample.total_bytes, sample.available_bytes)),
            controller.scene_cache_usage()?)?;
        reservation.recheck_pending()?;
        let account = controller.coordinator().external_memory_account();
        let source = source.clone();
        let cancel = Arc::new(AtomicBool::new(false));
        let worker_cancel = Arc::clone(&cancel);
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        std::thread::Builder::new().name("om9-typed-surface".into()).spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let _permit = loop {
                    if worker_cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
                    match account.try_acquire_surface_snap() {
                        Ok(permit) => break permit,
                        Err(Error::Invalid("surface snap already active")) => std::thread::sleep(std::time::Duration::from_millis(10)),
                        Err(error) => return Err(error),
                    }
                };
                reservation.recheck_pending()?;
                match &input {
                    TypedPointSource::Surface(document) => typed_surface_witness(document, root, &source, point, &limits,
                        &|| worker_cancel.load(Ordering::Acquire)),
                    #[cfg(feature = "occt-brep")]
                    TypedPointSource::Brep(brep) => {
                        let faces = &brep.topology().faces;
                        if faces.is_empty() || faces.len() as u64 > limits.max_nodes { return Err(Error::Budget("surface point faces")); }
                        if source.face.is_some_and(|face| !faces.contains(&face)) { return Err(Error::Stale("surface point face")); }
                        for (index, face) in faces.iter().enumerate() {
                            if worker_cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
                            if source.face.is_some_and(|selected| selected != *face) { continue; }
                            reservation.recheck_pending()?;
                            let witness = om9_kernel::measurement::point_on_source_face(brep.source(), index,
                                source.placement, point, tolerance, limits.max_nodes)
                                .map_err(|error| Error::Io(format!("surface point face: {error}")))?;
                            if witness.is_some() { return Ok(witness); }
                        }
                        Ok(None)
                    }
                    #[cfg(feature = "occt-brep")]
                    TypedPointSource::Native(native) => {
                        let mut admit = |live: u64, pending: u64| -> Result<()> {
                            if worker_cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
                            let total = live.checked_add(pending).ok_or(Error::Overflow("surface point native admission"))?;
                            if total > reservation.bytes() { return Err(Error::Budget("surface point native admission")); }
                            reservation.set_allocated_bytes(live)?;
                            reservation.recheck_pending()
                        };
                        let (converted, faces) = om9_kernel::brep::native_source_and_face_count_admitted(
                            native.payload(), tolerance, &limits, &mut admit)?;
                        if faces == 0 || faces as u64 > limits.max_nodes { return Err(Error::Budget("surface point faces")); }
                        if source.native_face_count != Some(faces) || source.face.is_some() {
                            return Err(Error::Stale("surface point native face count"));
                        }
                        for index in 0..faces {
                            if worker_cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
                            reservation.recheck_pending()?;
                            let witness = om9_kernel::measurement::point_on_source_face(&converted, index,
                                source.placement, point, tolerance, limits.max_nodes)
                                .map_err(|error| Error::Io(format!("surface point native face: {error}")))?;
                            if witness.is_some() { return Ok(witness); }
                        }
                        Ok(None)
                    }
                }
            })).unwrap_or_else(|_| Err(Error::Io("surface point worker panicked".into())));
            drop(input);
            drop(reservation);
            let _ = sender.send(result);
        }).map_err(|error| Error::Io(format!("surface point worker: {error}")))?;
        Ok(Self { cancel, receiver })
    }

    pub(crate) fn try_result(&self) -> Option<Result<Option<om9_core::geometry::Point3>>> {
        match self.receiver.try_recv() {
            Ok(result) => Some(result),
            Err(std::sync::mpsc::TryRecvError::Empty) => None,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => Some(Err(Error::Io("surface point worker disconnected".into()))),
        }
    }
}

impl Drop for TypedSurfaceJob {
    fn drop(&mut self) { self.cancel.store(true, Ordering::Release); }
}

#[cfg(feature = "occt-brep")]
fn refine_native_surface(text: &str, face_count: usize, candidate: &SnapCandidate, camera: &Camera,
    viewport: Viewport, tolerance: f64, limits: &om9_core::budget::Limits) -> Result<Option<SnapCandidate>> {
    let source = candidate.surface_source.as_ref().ok_or(Error::Missing("native surface source"))?;
    source.face_witness.ok_or(Error::Missing("native surface witness"))?;
    let inverse = om9_core::top11::inverse_affine(source.placement)?;
    let ray = camera.visible_ray(source.cursor, viewport)?;
    let origin = inverse.point(ray.origin)?;
    let next = inverse.point(std::array::from_fn(|index| ray.origin[index] + ray.direction[index]))?;
    let direction = std::array::from_fn(|index| next[index] - origin[index]);
    let matrix = source.placement.0;
    let bound = (0..3).flat_map(|row| (0..3).map(move |column| matrix[row][column] * matrix[row][column])).sum::<f64>().sqrt();
    if !bound.is_finite() || bound <= 0. { return Err(Error::Invalid("native surface placement")); }
    let local_tolerance = tolerance / bound.max(1.);
    // Native display meshes have no certified face mapping or deflection. Use
    // them only to nominate the object; find its nearest exact source hit from
    // the near plane, and let the viewport projection enforce clipping.
    let ray = om9_kernel::editing::IsocurveFaceRay { origin, direction, reference: origin,
        tolerance: local_tolerance, max_distance: f64::MAX, witness: None };
    let Some((_, local)) = om9_kernel::editing::try_ray_face_isocurve_hit_with_witness_from_source(text, ray, limits)
        .map_err(|error| Error::Io(format!("native surface snap: {error}")))? else { return Ok(None) };
    let world = source.placement.point(local)?;
    let Some(projected) = camera.project(world, viewport)? else { return Ok(None) };
    let mut refined = candidate.clone();
    refined.world = world;
    refined.screen = projected.screen;
    refined.distance_px = (projected.screen[0] - source.cursor[0]).hypot(projected.screen[1] - source.cursor[1]);
    refined.normal = None;
    refined.surface_source.as_mut().expect("native surface source").native_face_count = Some(face_count);
    Ok(Some(refined))
}

#[cfg(feature = "occt-brep")]
fn refine_leased_native_group(source: &om9_core::transaction::CapturedNativeBrepSource,
    candidates: &[SnapCandidate], camera: &Camera, viewport: Viewport, tolerance: f64,
    limits: &om9_core::budget::Limits, reservation: &std::sync::Mutex<om9_core::transaction::ExternalMemoryReservation>,
    baseline: u64, cancel: &AtomicBool, refined: &mut Vec<SnapCandidate>) -> Result<()> {
    let mut admit = |live: u64, pending: u64| -> Result<()> {
        if cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
        let allocated = baseline.checked_add(live).ok_or(Error::Overflow("native surface snap admission"))?;
        let required = allocated.checked_add(pending).ok_or(Error::Overflow("native surface snap admission"))?;
        let mut owner = reservation.lock().unwrap_or_else(|error| error.into_inner());
        if required > owner.bytes() { return Err(Error::Budget("native surface snap admission")); }
        owner.set_allocated_bytes(allocated)?;
        owner.recheck_pending()
    };
    let (text, face_count) = om9_kernel::brep::native_source_and_face_count_admitted(source.payload(), tolerance, limits, &mut admit)?;
    for candidate in candidates {
        if cancel.load(Ordering::Acquire) { return Err(Error::Canceled); }
        if let Some(result) = refine_native_surface(&text, face_count, candidate, camera, viewport, tolerance, limits)? {
            refined.push(result);
        }
    }
    drop(text);
    admit(0, 0)
}

#[cfg(feature = "occt-brep")]
fn refine_leased_brep_group(
    source: &om9_core::transaction::CapturedBrepSource, candidates: &[&SnapCandidate],
    camera: &Camera, viewport: Viewport, tolerance: f64, limits: &om9_core::budget::Limits,
) -> Result<[Option<SnapCandidate>; 8]> {
    if candidates.is_empty() || candidates.len() > 8 { return Err(Error::Invalid("surface snap group")); }
    let first = candidates[0];
    let context = first.surface_source.as_ref().ok_or(Error::Missing("surface snap source"))?;
    let mut faces = [uuid::Uuid::nil(); 8];
    let mut witnesses = [([[0.; 3]; 3], 0.); 8];
    for (index, candidate) in candidates.iter().enumerate() {
        if !same_batch_source(first, candidate) { return Err(Error::Invalid("surface snap group")); }
        faces[index] = candidate.surface_source.as_ref().and_then(|value| value.face)
            .ok_or(Error::Missing("surface snap face"))?;
        witnesses[index] = candidate.surface_source.as_ref().and_then(|value| value.face_witness)
            .ok_or(Error::Missing("surface snap face witness"))?;
    }
    let ray = camera.visible_ray(context.cursor, viewport)?;
    let query = |selected: &[uuid::Uuid], selected_witnesses: &[om9_kernel::measurement::FaceWitness]| {
        om9_kernel::measurement::try_verified_captured_brep_face_ray_hits_fixed(source, selected, selected_witnesses, context.placement,
            ray.origin, ray.direction, tolerance, limits.max_nodes)
            .map_err(|error| Error::Io(format!("surface snap refinement: {error}")))
    };
    let mut result: [Option<SnapCandidate>; 8] = std::array::from_fn(|_| None);
    let project = |candidate: &SnapCandidate, point| -> Result<Option<SnapCandidate>> {
        let Some(projected) = camera.project(point, viewport)? else { return Ok(None) };
        let mut result = candidate.clone();
        result.world = point;
        result.screen = projected.screen;
        result.distance_px = (projected.screen[0] - context.cursor[0]).hypot(projected.screen[1] - context.cursor[1]);
        result.normal = None;
        Ok(Some(result))
    };
    if let Some(points) = query(&faces[..candidates.len()], &witnesses[..candidates.len()])? {
        for (index, (candidate, &point)) in candidates.iter().zip(points.as_slice()).enumerate() {
            result[index] = project(candidate, point)?;
        }
    } else if candidates.len() > 1 {
        for (index, candidate) in candidates.iter().enumerate() {
            if let Some(point) = query(&faces[index..=index], &witnesses[index..=index])? {
                result[index] = project(candidate, point.as_slice()[0])?;
            }
        }
    }
    Ok(result)
}

fn refine_captured_group(
    document: &om9_core::model::Document, candidates: &[&SnapCandidate], camera: &Camera,
    viewport: Viewport, limits: &om9_core::budget::Limits,
) -> Result<[Option<SnapCandidate>; 8]> {
    if candidates.is_empty() || candidates.len() > 8 { return Err(Error::Invalid("surface snap group")); }
    let mut refined: [Option<SnapCandidate>; 8] = std::array::from_fn(|_| None);
    #[cfg(feature = "occt-brep")]
    {
        let first = candidates.first().ok_or(Error::Invalid("empty surface snap group"))?;
        let source = first.surface_source.as_ref().ok_or(Error::Missing("surface snap source"))?;
        let entity = captured_entity(document, first)?;
        if let om9_core::geometry::Geometry::Brep(brep) = &entity.geometry {
            let mut faces = [uuid::Uuid::nil(); 8];
            let mut witnesses = [([[0.; 3]; 3], 0.); 8];
            for (index, candidate) in candidates.iter().enumerate() {
                if index >= faces.len() || !same_batch_source(first, candidate) { return Err(Error::Invalid("surface snap group")); }
                faces[index] = candidate.surface_source.as_ref().and_then(|s| s.face).ok_or(Error::Missing("surface snap face"))?;
                witnesses[index] = candidate.surface_source.as_ref().and_then(|s| s.face_witness).ok_or(Error::Missing("surface snap face witness"))?;
            }
            let ray = camera.visible_ray(source.cursor, viewport)?;
            let tolerance = document.effective_tolerance()?.absolute_mm;
            let points = om9_kernel::measurement::try_verified_brep_face_ray_hits_fixed(brep, &faces[..candidates.len()], &witnesses[..candidates.len()], source.placement,
                ray.origin, ray.direction, tolerance, limits).map_err(|error| Error::Io(format!("surface snap refinement: {error}")))?;
            if let Some(points) = points {
                for (index, (candidate, &point)) in candidates.iter().zip(points.as_slice()).enumerate() {
                    let Some(projected) = camera.project(point, viewport)? else { continue };
                    let mut result = (*candidate).clone();
                    result.world = point;
                    result.screen = projected.screen;
                    result.distance_px = (projected.screen[0] - source.cursor[0]).hypot(projected.screen[1] - source.cursor[1]);
                    result.normal = None;
                    refined[index] = Some(result);
                }
                return Ok(refined);
            }
            // A display triangle can hit where its exact face does not. Retry
            // individually only for this geometric miss, preserving each slot.
            for (index, candidate) in candidates.iter().enumerate() {
                refined[index] = try_refine_captured_surface(document, candidate, camera, viewport, limits)?;
            }
            return Ok(refined);
        }
    }
    for (index, candidate) in candidates.iter().enumerate() {
        refined[index] = try_refine_captured_surface(document, candidate, camera, viewport, limits)?;
    }
    Ok(refined)
}

fn refine_captured_surface(
    document: &om9_core::model::Document,
    candidate: &SnapCandidate,
    camera: &Camera,
    viewport: Viewport,
    limits: &om9_core::budget::Limits,
) -> Result<SnapCandidate> {
    try_refine_captured_surface(document, candidate, camera, viewport, limits)?
        .ok_or(Error::Missing("visible exact surface snap intersection"))
}

#[cfg(feature = "occt-brep")]
fn certified_trim_seam_hit(surface: &om9_core::geometry::NurbsSurface,
    trim: &om9_core::rhino_semantics::TrimmedSurface, uv: [f64;2], limits: &om9_core::budget::Limits) -> Result<bool> {
    let axis=match super::wireframe::validate_trim_closed_seams(surface,trim,limits) {
        Ok(Some(axis))=>axis,
        Ok(None) | Err(Error::Unsupported(_))=>return Ok(false),
        Err(error)=>return Err(error),
    };
    let ends=surface.domain[axis];
    if uv[axis]!=ends[0] && uv[axis]!=ends[1] {return Ok(false);}
    let other=1-axis;
    let low=trim.outer.iter().map(|p|p[other]).fold(f64::INFINITY,f64::min);
    let high=trim.outer.iter().map(|p|p[other]).fold(f64::NEG_INFINITY,f64::max);
    let scale=ends.into_iter().chain([low,high]).map(f64::abs).fold(1_f64,f64::max);
    let guard=128.*f64::EPSILON*scale;
    if !guard.is_finite() || uv[other]<=low+guard || uv[other]>=high-guard {return Ok(false);}
    Ok(trim.inners.iter().flatten().all(|p|p[axis]>ends[0]+guard && p[axis]<ends[1]-guard))
}

#[cfg(feature = "occt-brep")]
fn try_refine_surface_geometry(
    geometry: &om9_core::geometry::Geometry, extension: Option<&om9_core::model::Extension>, tolerance: f64,
    candidate: &SnapCandidate, camera: &Camera, viewport: Viewport, limits: &om9_core::budget::Limits,
) -> Result<Option<SnapCandidate>> {
    let om9_core::geometry::Geometry::Surface(surface) = geometry else { return Err(Error::Invalid("surface source")) };
    let source = candidate.surface_source.as_ref().ok_or(Error::Missing("surface snap source"))?;
    let ray = camera.visible_ray(source.cursor, viewport)?;
    let point = if let Some(extension) = extension {
        if extension.payload.len()>om9_core::model::trim_surface_payload_limit(limits) {return Err(Error::Unsupported("surface snap trim metadata exceeds supported size"));}
        let trim=om9_core::rhino_semantics::TrimmedSurface::from_extension(extension,limits)?;
        if trim.underlying!=*surface {return Err(Error::Invalid("surface snap trim source mismatch"));}
        om9_core::rhino_semantics::validate_trim_loop_topology_for_display(&trim)?;
        let mut hits=om9_kernel::measurement::surface_ray_hits_uv(geometry,source.placement,ray.origin,ray.direction,tolerance,limits)
            .map_err(|error| Error::Io(format!("surface snap UV intersection: {error}")))?;
        hits.sort_by(|a,b|a.distance.total_cmp(&b.distance));
        let verify_tolerance=tolerance.min(1e-6);
        for pair in hits.windows(2) {
            let apart=(0..3).map(|i|(pair[0].point[i]-pair[1].point[i]).powi(2)).sum::<f64>().sqrt();
            if (pair[1].distance-pair[0].distance).abs()<=verify_tolerance && apart>verify_tolerance {
                return Err(Error::Unsupported("ambiguous surface snap intersections"));
            }
        }
        let mut selected=None;
        for hit in hits {
            let local=om9_core::surface_kernel::evaluate(surface,hit.uv[0],hit.uv[1],limits)?;
            let expected=source.placement.point(local)?;
            let separation=(0..3).map(|i|(expected[i]-hit.point[i]).powi(2)).sum::<f64>().sqrt();
            if !separation.is_finite() || separation>verify_tolerance {return Err(Error::Unsupported("surface snap UV source mismatch"));}
            match om9_core::rhino_semantics::classify_trim_uv(&trim,hit.uv)? {
                Some(true)=>{selected=Some(hit.point);break;},
                Some(false)=>{},
                None if certified_trim_seam_hit(surface,&trim,hit.uv,limits)?=>{selected=Some(hit.point);break;},
                None=>return Err(Error::Unsupported("surface snap near ambiguous trim boundary")),
            }
        }
        selected
    } else {
        om9_kernel::measurement::try_surface_ray_hit(geometry, source.placement, ray.origin, ray.direction, ray.origin, tolerance, limits)
            .map_err(|error| Error::Io(format!("surface snap refinement: {error}")))?
    };
    let Some(point) = point else { return Ok(None) };
    let Some(projected) = camera.project(point, viewport)? else { return Ok(None) };
    let mut result = candidate.clone();
    result.world = point;
    result.screen = projected.screen;
    result.distance_px = (projected.screen[0] - source.cursor[0]).hypot(projected.screen[1] - source.cursor[1]);
    result.normal = None;
    Ok(Some(result))
}

fn try_refine_captured_surface(
    document: &om9_core::model::Document,
    candidate: &SnapCandidate,
    camera: &Camera,
    viewport: Viewport,
    limits: &om9_core::budget::Limits,
) -> Result<Option<SnapCandidate>> {
    let source = candidate.surface_source.as_ref().ok_or(Error::Missing("surface snap source"))?;
    let entity = captured_entity(document, candidate)?;
    #[cfg(feature = "occt-brep")]
    {
        let ray = camera.visible_ray(source.cursor, viewport)?;
        let tolerance = document.effective_tolerance()?.absolute_mm;
        if matches!(entity.geometry, om9_core::geometry::Geometry::Surface(_)) {
            return try_refine_surface_geometry(&entity.geometry, entity.extensions.get(om9_core::model::TRIM_SURFACE_EXTENSION_SLOT),
                tolerance, candidate, camera, viewport, limits);
        }
        let point = match &entity.geometry {
            om9_core::geometry::Geometry::Brep(brep) => om9_kernel::measurement::try_verified_brep_face_ray_hit(
                brep, source.face.ok_or(Error::Missing("surface snap face"))?, source.placement,
                ray.origin, ray.direction, candidate.world, tolerance,
                source.face_witness.ok_or(Error::Missing("surface snap face witness"))?, limits),
            _ => return Err(Error::Unsupported("exact surface snap for this geometry")),
        }.map_err(|error| Error::Io(format!("surface snap refinement: {error}")))?;
        let Some(point) = point else { return Ok(None) };
        let Some(projected) = camera.project(point, viewport)? else { return Ok(None) };
        let mut result = candidate.clone();
        result.world = point;
        result.screen = projected.screen;
        result.distance_px = (projected.screen[0] - source.cursor[0]).hypot(projected.screen[1] - source.cursor[1]);
        result.normal = None;
        Ok(Some(result))
    }
    #[cfg(not(feature = "occt-brep"))]
    { let _ = (camera, viewport, limits, source, entity); Err(Error::Unsupported("surface snap requires OCCT")) }
}

#[cfg(feature="occt-brep")]
fn certified_strict_convex_pick(polygon:&[[f64;2]],point:[f64;2],guard:f64)->bool {
    if polygon.len()<3 || polygon.len()>om9_core::rhino_semantics::TRIM_SOURCE_PICK_MAX_EDGES || !guard.is_finite() || guard<0. || !point.iter().all(|v|v.is_finite()) {return false;}
    // One upper bound for every coordinate keeps the numerical screen at
    // least as conservative while avoiding a six-value scan for every pair.
    let scale=polygon.iter().flatten().copied().chain(point).map(f64::abs).fold(1_f64,f64::max);
    if !scale.is_finite() {return false;}
    let mut orientation=0_i8;
    for i in 0..polygon.len() {
        let a=polygon[i];let b=polygon[(i+1)%polygon.len()];
        let dx=b[0]-a[0];let dy=b[1]-a[1];let edge_delta=dx.abs()+dy.abs();
        let length=dx.hypot(dy);if !length.is_finite() || length==0. {return false;}
        for j in 0..=polygon.len() {
            if j<polygon.len() && (j==i || j==(i+1)%polygon.len()) {continue;}
            let p=if j==polygon.len(){point}else{polygon[j]};
            let px=p[0]-a[0];let py=p[1]-a[1];let x=dx*py;let y=dy*px;let cross=x-y;
            let delta=edge_delta+px.abs()+py.abs();
            let bound=128.*f64::EPSILON*(x.abs()+y.abs()+scale*delta) + if j==polygon.len(){guard*length}else{0.};
            if !cross.is_finite() || !bound.is_finite() || cross.abs()<=bound {return false;}
            let sign=if cross>0. {1}else{-1};if orientation==0 {orientation=sign;}else if orientation!=sign {return false;}
        }
    }
    true
}

#[cfg(feature="occt-brep")]
fn strict_trim_side(a:[f64;2],b:[f64;2],p:[f64;2],scale:f64,guard:f64)->Option<i8> {
    let dx=b[0]-a[0];let dy=b[1]-a[1];let px=p[0]-a[0];let py=p[1]-a[1];
    let length=dx.hypot(dy);let x=dx*py;let y=dy*px;let cross=x-y;
    let bound=128.*f64::EPSILON*(x.abs()+y.abs()+scale*(dx.abs()+dy.abs()+px.abs()+py.abs()))+guard*length;
    if !length.is_finite() || length==0. || !cross.is_finite() || !bound.is_finite() || cross.abs()<=bound {None}
    else {Some(if cross>0. {1}else{-1})}
}

#[cfg(feature="occt-brep")]
fn certified_convex_trim_pick(trim:&om9_core::rhino_semantics::TrimmedSurface,point:[f64;2],guard:f64)->bool {
    let Some(edges)=trim.inners.iter().try_fold(trim.outer.len(),|n,hole|n.checked_add(hole.len())) else{return false};
    if edges>om9_core::rhino_semantics::TRIM_SOURCE_PICK_MAX_EDGES || !certified_strict_convex_pick(&trim.outer,point,guard) {return false;}
    let scale=std::iter::once(&trim.outer).chain(&trim.inners).flatten().flatten().copied()
        .chain(point).map(f64::abs).fold(1_f64,f64::max);
    if !scale.is_finite(){return false;}
    let side=|poly:&[[f64;2]],i:usize,p:[f64;2],margin:f64|strict_trim_side(poly[i],poly[(i+1)%poly.len()],p,scale,margin);
    let Some(outer_orientation)=side(&trim.outer,0,point,guard) else{return false};
    for (index,hole) in trim.inners.iter().enumerate() {
        if hole.len()<3{return false;}
        let n=hole.len() as f64;let center=hole.iter().fold([0.,0.],|s,p|[s[0]+p[0]/n,s[1]+p[1]/n]);
        if !certified_strict_convex_pick(hole,center,0.) {return false;}
        let Some(hole_orientation)=side(hole,0,center,0.) else{return false};
        // Every hole vertex must be securely inside the outer convex polygon.
        if hole.iter().any(|&p|(0..trim.outer.len()).any(|i|side(&trim.outer,i,p,guard)!=Some(outer_orientation))) {return false;}
        // At least one hole half-plane must exclude the picked point by the
        // guard, so a click near or inside the hole is never accepted.
        if !(0..hole.len()).any(|i|side(hole,i,point,guard)==Some(-hole_orientation)) {return false;}
        for other in &trim.inners[..index] {
            if other.len()<3{return false;}
            let Some(other_orientation)=side(other,0,other[2],0.) else{return false};
            let separated=|a:&[[f64;2]],orientation:i8,b:&[[f64;2]]| {
                (0..a.len()).any(|i|b.iter().all(|&p|side(a,i,p,guard)==Some(-orientation)))
            };
            if !separated(hole,hole_orientation,other) && !separated(other,other_orientation,hole) {return false;}
        }
    }
    true
}

#[cfg(feature="occt-brep")]
fn trim_point_segment_distance(p:[f64;2],a:[f64;2],b:[f64;2])->f64 {
    let dx=b[0]-a[0];let dy=b[1]-a[1];let length=dx.hypot(dy);
    if !length.is_finite() || length==0. {return f64::NAN;}
    let unit=[dx/length,dy/length];
    let t=((p[0]-a[0])*unit[0]+(p[1]-a[1])*unit[1]).clamp(0.,length);
    (p[0]-(a[0]+t*unit[0])).hypot(p[1]-(a[1]+t*unit[1]))
}

#[cfg(feature="occt-brep")]
fn trim_point_inside_with_margin(polygon:&[[f64;2]],point:[f64;2],guard:f64)->bool {
    if !guard.is_finite() || guard<0. || polygon.len()<3 || polygon.len()>om9_core::rhino_semantics::TRIM_SOURCE_PICK_MAX_EDGES
        || om9_core::rhino_semantics::classify_trim_loop_uv(polygon,point).ok()!=Some(Some(true)) {return false;}
    let scale=polygon.iter().flatten().copied().chain(point).map(f64::abs).fold(1_f64,f64::max);
    let margin=guard+2048.*f64::EPSILON*scale;
    if !margin.is_finite() {return false;}
    (0..polygon.len()).all(|i| {let distance=trim_point_segment_distance(point,polygon[i],polygon[(i+1)%polygon.len()]);
        distance.is_finite() && distance>margin})
}

#[cfg(feature="occt-brep")]
fn certified_general_trim_pick(trim:&om9_core::rhino_semantics::TrimmedSurface,point:[f64;2],guard:f64)->bool {
    use om9_core::rhino_semantics::{classify_trim_loop_uv,classify_trim_uv,validate_trim_loop_topology_for_source_pick};
    if !guard.is_finite() || guard<0. || classify_trim_uv(trim,point).ok()!=Some(Some(true))
        || validate_trim_loop_topology_for_source_pick(trim).is_err() {return false;}
    if trim.inners.iter().flatten().any(|&p|classify_trim_loop_uv(&trim.outer,p).ok()!=Some(Some(true))) {return false;}
    let loops=std::iter::once(&trim.outer).chain(&trim.inners);
    let scale=loops.clone().flatten().flatten().copied().chain(point).map(f64::abs).fold(1_f64,f64::max);
    let margin=guard+2048.*f64::EPSILON*scale;
    if !margin.is_finite() {return false;}
    let distance=trim_point_segment_distance;
    for (li,polygon) in loops.clone().enumerate() {
        for i in 0..polygon.len() {
            let a=polygon[i];let b=polygon[(i+1)%polygon.len()];
            let d=distance(point,a,b);if !d.is_finite() || d<=margin {return false;}
            for other in loops.clone().skip(li+1) {
                for j in 0..other.len() {
                    let c=other[j];let d=other[(j+1)%other.len()];
                    if (0..2).any(|axis|a[axis].max(b[axis])+margin<c[axis].min(d[axis])
                        || c[axis].max(d[axis])+margin<a[axis].min(b[axis])) {continue;}
                    if [distance(a,c,d),distance(b,c,d),distance(c,a,b),distance(d,a,b)]
                        .into_iter().any(|v|!v.is_finite() || v<=margin) {return false;}
                }
            }
        }
    }
    true
}

// Restrict this first curved fallback to cylinders that can be reproduced
// exactly by the native Rust constructor. A merely similar spline is not a
// certificate, even when a few evaluated points happen to lie on a cylinder.
#[cfg(feature="occt-brep")]
fn certified_fallback_cylinder(surface:&om9_core::geometry::NurbsSurface,limits:&om9_core::budget::Limits)->Option<(f64,f64)> {
    if surface.poles.len()!=18 {return None;}
    let centre=std::array::from_fn::<_,3,_>(|i| (surface.poles[0][i]+surface.poles[8][i])*0.5);
    let xvec=std::array::from_fn::<_,3,_>(|i|surface.poles[0][i]-centre[i]);
    let yvec=std::array::from_fn::<_,3,_>(|i|surface.poles[4][i]-centre[i]);
    let zvec=std::array::from_fn::<_,3,_>(|i|surface.poles[1][i]-surface.poles[0][i]);
    let length=|v:[f64;3]|v.into_iter().map(|x|x*x).sum::<f64>().sqrt();
    let radius=length(xvec);let height=length(zvec);
    if !radius.is_finite() || !height.is_finite() || radius<=1e-6 || height<=1e-6 {return None;}
    let x=xvec.map(|v|v/radius);let y=yvec.map(|v|v/radius);
    let rebuilt=om9_core::surface_kernel::cylinder(centre,x,y,radius,height,limits).ok()?;
    if rebuilt!=*surface {return None;} Some((radius,height))
}

#[cfg(feature="occt-brep")]
fn fallback_inverse_bound(transform:om9_core::geometry::Transform)->Option<f64> {
    let a=transform.0;
    let cross=|x:[f64;3],y:[f64;3]|[x[1]*y[2]-x[2]*y[1],x[2]*y[0]-x[0]*y[2],x[0]*y[1]-x[1]*y[0]];
    let rows=[[a[0][0],a[0][1],a[0][2]],[a[1][0],a[1][1],a[1][2]],[a[2][0],a[2][1],a[2][2]]];
    let cof=[cross(rows[1],rows[2]),cross(rows[2],rows[0]),cross(rows[0],rows[1])];
    let det=(0..3).map(|i|rows[0][i]*cof[0][i]).sum::<f64>();
    let norm=rows.iter().flatten().map(|v|v*v).sum::<f64>().sqrt();
    let bound=cof.iter().flatten().map(|v|v*v).sum::<f64>().sqrt()/det.abs();
    if !norm.is_finite() || !bound.is_finite() || det.abs()<=1e-12*norm.powi(3) || bound>1e6 {None} else {Some(bound)}
}

#[cfg(feature="occt-brep")]
fn pick_certified_fallback_cylinder(surface:&om9_core::geometry::NurbsSurface,
    trim:&om9_core::rhino_semantics::TrimmedSurface, transform:om9_core::geometry::Transform,
    ray:&om9_scene::camera::Ray, mut hits:Vec<om9_kernel::measurement::SurfaceRayUvHit>,
    camera:&Camera, viewport:Viewport, tolerance:f64, limits:&om9_core::budget::Limits) -> Result<Option<f64>> {
    use om9_core::rhino_semantics::classify_trim_uv;
    let Some((radius,height))=certified_fallback_cylinder(surface,limits) else{return Ok(None)};
    // Frobenius norm of the inverse bounds world-to-local error amplification.
    // Refuse near-singular matrices before using a computed inverse bound.
    let a=transform.0;
    let Some(inverse_bound)=fallback_inverse_bound(transform) else{return Ok(None)};
    hits.sort_by(|a,b|a.distance.total_cmp(&b.distance));
    let len=|v:[f64;3]|v.into_iter().map(|x|x*x).sum::<f64>().sqrt();
    let ray_len=len(ray.direction);if !ray_len.is_finite() || ray_len==0. {return Ok(None);}
    for pair in hits.windows(2) {
        let apart=len(std::array::from_fn(|i|pair[0].point[i]-pair[1].point[i]));
        if (pair[1].distance-pair[0].distance).abs()<=tolerance && apart>tolerance {return Ok(None);}
    }
    for hit in hits {
        let derivatives=om9_core::surface_kernel::derivatives(surface,hit.uv[0],hit.uv[1],limits)?;
        let expected=transform.point(derivatives.point)?;
        if len(std::array::from_fn(|i|expected[i]-hit.point[i]))>tolerance.min(1e-6) {return Ok(None);}
        let vector=|p:[f64;3]|std::array::from_fn::<_,3,_>(|i|(0..3).map(|j|a[i][j]*p[j]).sum::<f64>());
        let u=vector(derivatives.du);let v=vector(derivatives.dv);
        let normal=[u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]];
        let incidence=normal.iter().zip(ray.direction).map(|(a,b)|a*b).sum::<f64>().abs()/(len(normal)*ray_len);
        if !incidence.is_finite() || incidence<0.05 {return Ok(None);}
        let scale=ray.origin.into_iter().chain(hit.point).map(f64::abs).fold(1_f64,f64::max);
        let guard=(16.*tolerance+4096.*f64::EPSILON*scale)*inverse_bound/(radius.min(height)*incidence);
        if !guard.is_finite() || guard>0.05 {return Ok(None);}
        match classify_trim_uv(trim,hit.uv)? {
            Some(false) => {
                let safe=std::iter::once(&trim.outer).chain(&trim.inners).all(|p|(0..p.len()).all(|i| {
                    let d=trim_point_segment_distance(hit.uv,p[i],p[(i+1)%p.len()]);d.is_finite() && d>guard
                }));
                if !safe {return Ok(None);}
            }
            Some(true) => {
                if !(certified_convex_trim_pick(trim,hit.uv,guard) || certified_general_trim_pick(trim,hit.uv,guard)) {return Ok(None);}
                return Ok(camera.project(hit.point,viewport)?.map(|p|p.depth));
            }
            None=>return Ok(None),
        }
    }
    Ok(None)
}

#[cfg(feature="occt-brep")]
fn certified_cylinder_front_clear(surface:&om9_core::geometry::NurbsSurface,
    geometry:&om9_core::geometry::Geometry,
    trim:&om9_core::rhino_semantics::TrimmedSurface, transform:om9_core::geometry::Transform,
    ray:&om9_scene::camera::Ray, target:[f64;3], tolerance:f64, limits:&om9_core::budget::Limits) -> Result<bool> {
    let Some((radius,height))=certified_fallback_cylinder(surface,limits) else{return Ok(false)};
    let Some(inverse_bound)=fallback_inverse_bound(transform) else{return Ok(false)};
    let len=|v:[f64;3]|v.into_iter().map(|x|x*x).sum::<f64>().sqrt();
    let ray_len=len(ray.direction);let to_target=std::array::from_fn::<_,3,_>(|i|target[i]-ray.origin[i]);
    if !ray_len.is_finite() || ray_len==0. || !tolerance.is_finite() || tolerance<=0. || trim.outer.len()<3 {return Ok(false);}
    let target_distance=(0..3).map(|i|to_target[i]*ray.direction[i]/ray_len).sum::<f64>();
    let off_ray=len(std::array::from_fn(|i|to_target[i]-target_distance*ray.direction[i]/ray_len));
    if !target_distance.is_finite() || target_distance<0. || !off_ray.is_finite() || off_ray>tolerance {return Ok(false);}
    let hits=om9_kernel::measurement::surface_ray_hits_uv(geometry,transform,
        ray.origin,ray.direction,tolerance,limits).map_err(|e|Error::Io(format!("curved trim occlusion: {e}")))?;
    let mut checked=0;
    for hit in hits.into_iter().filter(|h|h.distance<=target_distance+tolerance) {
        checked+=1;
        let derivatives=om9_core::surface_kernel::derivatives(surface,hit.uv[0],hit.uv[1],limits)?;
        let expected=transform.point(derivatives.point)?;
        if len(std::array::from_fn(|i|expected[i]-hit.point[i]))>tolerance.min(1e-6) {return Ok(false);}
        let vector=|p:[f64;3]|std::array::from_fn::<_,3,_>(|i|(0..3).map(|j|transform.0[i][j]*p[j]).sum::<f64>());
        let u=vector(derivatives.du);let v=vector(derivatives.dv);
        let normal=[u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]];
        let incidence=normal.iter().zip(ray.direction).map(|(a,b)|a*b).sum::<f64>().abs()/(len(normal)*ray_len);
        if !incidence.is_finite() || incidence<0.05 {return Ok(false);}
        let scale=ray.origin.into_iter().chain(target).chain(hit.point).map(f64::abs).fold(1_f64,f64::max);
        let guard=(16.*tolerance+4096.*f64::EPSILON*scale)*inverse_bound/(radius.min(height)*incidence);
        if !guard.is_finite() || guard>0.05 {return Ok(false);}
        let outside=(0..2).any(|axis| {
            let lo=trim.outer.iter().map(|p|p[axis]).fold(f64::INFINITY,f64::min);
            let hi=trim.outer.iter().map(|p|p[axis]).fold(f64::NEG_INFINITY,f64::max);
            hit.uv[axis]<lo-guard || hit.uv[axis]>hi+guard
        });
        if outside {continue;}
        if !trim.inners.iter().any(|hole|trim_point_inside_with_margin(hole,hit.uv,guard))
            || om9_core::rhino_semantics::validate_trim_loop_topology_for_source_pick(trim).is_err()
            || trim.inners.iter().flatten().any(|&p|
                om9_core::rhino_semantics::classify_trim_loop_uv(&trim.outer,p).ok()!=Some(Some(true))) {return Ok(false);}
    }
    Ok(checked>0)
}

impl Controller {
    /// Point selection on an untrimmed display fallback is allowed only when
    /// its bounded trim and source intersections can be verified.
    /// All uncertainty is returned to the caller as a refusal.
    pub(crate) fn fallback_trim_source_pick(&self, object:&om9_scene::scene::SceneObject,
        camera:&Camera, viewport:Viewport, cursor:[f64;2]) -> Result<Option<f64>> {
        #[cfg(feature="occt-brep")]
        {
            use om9_core::{budget::memory::MemoryClass,geometry::Geometry,rhino_semantics::{classify_trim_uv,TrimmedSurface}};
            if !object.visible || !object.selectable || !object.buffers.is_trim_display_fallback() {return Ok(None);}
            let scene=self.scene().ok_or(Error::Missing("trim pick scene"))?;
            if scene.epoch!=self.coordinator().epoch() {return Err(Error::Stale("trim pick scene"));}
            self.document().resolve(&object.source)?;
            let entity=scene.entity_for(&scene.document, object)?;
            let Geometry::Surface(surface)=&entity.geometry else{return Ok(None)};
            let is_plane=super::wireframe::is_affine_trim_plane(surface);
            // Unsupported curved surfaces cannot produce a source pick. Check
            // before admission or OCCT so they do not block unrelated picks
            // under memory pressure or pay for an unnecessary ray query.
            if !is_plane && certified_fallback_cylinder(surface,self.coordinator().limits()).is_none() {return Ok(None);}
            let ray=camera.visible_ray(cursor,viewport)?;
            let (centre,radius)=object.sphere_bounds()?;
            let direction2=ray.direction.iter().map(|x|x*x).sum::<f64>();
            let toward=std::array::from_fn::<_,3,_>(|i|centre[i]-ray.origin[i]);
            let along=(toward.iter().zip(ray.direction).map(|(a,b)|a*b).sum::<f64>()/direction2).max(0.);
            let separation=(0..3).map(|i|(toward[i]-along*ray.direction[i]).powi(2)).sum::<f64>().sqrt();
            let scale=centre.into_iter().chain(ray.origin).map(f64::abs).fold(1_f64,f64::max);
            let padding=1e-6+1024.*f64::EPSILON*scale;
            if direction2.is_finite() && direction2>0. && separation.is_finite() && radius.is_finite() && padding.is_finite()
                && separation>radius+padding {return Ok(None);}
            let ext=entity.extensions.get(om9_core::model::TRIM_SURFACE_EXTENSION_SLOT).ok_or(Error::Missing("trim pick metadata"))?;
            let limits=self.coordinator().limits();
            if ext.payload.len()>om9_core::model::trim_surface_payload_limit(limits) {return Ok(None);}
            let scratch=ext.payload.len().checked_mul(64).and_then(|n|n.checked_add(8192)).ok_or(Error::Overflow("trim pick scratch"))?;
            let _reservation=self.coordinator().reserve_external_memory_with_held(MemoryClass::Kernel,
                u64::try_from(scratch).map_err(|_|Error::Overflow("trim pick scratch"))?,
                om9_native_owner::physical_memory().map(|s|(s.total_bytes,s.available_bytes)),self.scene_cache_usage()?)?;
            let trim=TrimmedSurface::from_extension(ext,limits)?;
            if trim.underlying!=*surface || trim.outer.len()<3 || trim.outer.len()>om9_core::rhino_semantics::TRIM_SOURCE_PICK_MAX_EDGES {return Ok(None);}
            let transform=object.display_transform()?;
            let tolerance=self.document().effective_tolerance()?.absolute_mm;
            let hits=om9_kernel::measurement::surface_ray_hits_uv(&entity.geometry,transform,ray.origin,ray.direction,tolerance,limits)
                .map_err(|e|Error::Io(format!("trim pick source: {e}")))?;
            if !is_plane {return pick_certified_fallback_cylinder(surface,&trim,transform,&ray,hits,camera,viewport,tolerance,limits);}
            if hits.len()!=1 {return Ok(None);}let hit=hits[0];
            if classify_trim_uv(&trim,hit.uv)?!=Some(true) {return Ok(None);}
            let expected=transform.point(om9_core::surface_kernel::evaluate(surface,hit.uv[0],hit.uv[1],limits)?)?;
            let length=|v:[f64;3]|v.into_iter().map(|x|x*x).sum::<f64>().sqrt();
            if length(std::array::from_fn(|i|expected[i]-hit.point[i]))>tolerance.min(1e-6) {return Ok(None);}
            let p0=transform.point(surface.poles[0])?;let pu=transform.point(surface.poles[2])?;let pv=transform.point(surface.poles[1])?;
            let u=std::array::from_fn::<_,3,_>(|i|pu[i]-p0[i]);let v=std::array::from_fn::<_,3,_>(|i|pv[i]-p0[i]);
            let (lu,lv)=(length(u),length(v));let normal=[u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]];
            let sin_angle=length(normal)/(lu*lv);
            let incidence=normal.iter().zip(ray.direction).map(|(a,b)|a*b).sum::<f64>().abs()/(length(normal)*length(ray.direction));
            if !lu.is_finite() || !lv.is_finite() || lu<=1e-6 || lv<=1e-6 || !sin_angle.is_finite() || sin_angle<=1e-6 || !incidence.is_finite() || incidence<=1e-6 {return Ok(None);}
            let scale=p0.into_iter().chain(pu).chain(pv).chain(ray.origin).chain(hit.point).map(f64::abs).fold(1_f64,f64::max);
            let span=(0..2).map(|axis|(surface.knots[axis][surface.size[axis]]-surface.knots[axis][surface.degree[axis]]).abs()).fold(0_f64,f64::max);
            let guard=(4.*tolerance+1024.*f64::EPSILON*scale)*span/(lu.min(lv)*sin_angle*incidence);
            if !guard.is_finite() || !(certified_convex_trim_pick(&trim,hit.uv,guard)
                || certified_general_trim_pick(&trim,hit.uv,guard)) {return Ok(None);}
            let Some(projected)=camera.project(hit.point,viewport)? else{return Ok(None)};
            Ok(Some(projected.depth))
        }
        #[cfg(not(feature="occt-brep"))]
        {let _=(object,camera,viewport,cursor);Ok(None)}
    }

    /// A conservative exception to fallback occlusion: prove a ray meets an
    /// verified source outside the outer bounds or inside a verified hole.
    /// Every error or numerically uncertain case remains occluding.
    pub(crate) fn fallback_surface_proven_clear(&self, object: &om9_scene::scene::SceneObject,
        camera: &Camera, viewport: Viewport, cursor: [f64;2], target: [f64;3], tolerance: f64) -> bool {
        #[cfg(feature = "occt-brep")]
        {
            let check=|| -> Result<bool> {
                let scene=self.scene().ok_or(Error::Missing("trim occlusion scene"))?;
                let entity=scene.entity_for(&scene.document, object)?;
                let om9_core::geometry::Geometry::Surface(surface)=&entity.geometry else {return Ok(false)};
                if !object.buffers.is_trim_display_fallback() {return Ok(false);}
                let is_plane=super::wireframe::is_affine_trim_plane(surface);
                if !is_plane && certified_fallback_cylinder(surface,self.coordinator().limits()).is_none() {return Ok(false);}
                let extension=entity.extensions.get(om9_core::model::TRIM_SURFACE_EXTENSION_SLOT).ok_or(Error::Missing("trim occlusion metadata"))?;
                if extension.payload.len()>om9_core::model::trim_surface_payload_limit(self.coordinator().limits()) {return Ok(false);}
                let scratch=extension.payload.len().checked_mul(64).and_then(|n|n.checked_add(8192)).ok_or(Error::Overflow("trim occlusion scratch"))?;
                let _reservation=self.coordinator().reserve_external_memory_with_held(
                    om9_core::budget::memory::MemoryClass::Kernel,u64::try_from(scratch).map_err(|_|Error::Overflow("trim occlusion scratch"))?,
                    om9_native_owner::physical_memory().map(|s|(s.total_bytes,s.available_bytes)),self.scene_cache_usage()?)?;
                Ok(fallback_surface_proven_clear_source(&entity.geometry, extension, object.display_transform()?, camera, viewport, cursor, target, tolerance, self.coordinator().limits()))
            };
            check().unwrap_or(false)
        }
        #[cfg(not(feature = "occt-brep"))]
        {let _=(object,camera,viewport,cursor,target,tolerance);false}
    }

    fn reserve_trim_snap_scratch(&self, candidate: &SnapCandidate)
        -> Result<Option<om9_core::transaction::ExternalMemoryReservation>> {
        let bytes=trim_snap_scratch(self.document(),candidate)?;
        if bytes==0 {return Ok(None);}
        Ok(Some(self.coordinator().reserve_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::Kernel,bytes,
            om9_native_owner::physical_memory().map(|s|(s.total_bytes,s.available_bytes)),self.scene_cache_usage()?)?))
    }
    pub(crate) fn can_group_surface_candidates(a: &SnapCandidate, b: &SnapCandidate) -> bool {
        same_batch_source(a, b)
    }

    pub(crate) fn refine_surface_snap_group(
        &self, candidates: &[&SnapCandidate], camera: &Camera, viewport: Viewport,
    ) -> Result<[Option<SnapCandidate>; 8]> {
        if candidates.is_empty() || candidates.len() > 8
            || candidates.iter().any(|candidate| !same_batch_source(&candidates[0], candidate)) {
            return Err(Error::Invalid("surface snap group"));
        }
        for candidate in candidates { self.validate_surface_candidate(candidate, camera, viewport)?; }
        refine_captured_group(self.document(), candidates, camera, viewport, self.coordinator().limits())
    }

    pub(crate) fn validate_surface_candidate<'a>(
        &'a self,
        candidate: &SnapCandidate,
        camera: &Camera,
        viewport: Viewport,
    ) -> Result<&'a om9_scene::scene::SceneObject> {
        if candidate.kind != SnapKind::Surface {
            return Err(Error::Invalid("surface snap candidate required"));
        }
        let source = candidate
            .surface_source
            .as_ref()
            .ok_or(Error::Unsupported("surface snap has no source binding"))?;
        if &source.camera != camera
            || source.viewport != viewport
            || source.scene_epoch != self.coordinator().epoch()
        {
            return Err(Error::Stale("surface snap camera or scene"));
        }
        let root = candidate.root.ok_or(Error::Invalid("surface snap root"))?;
        if source.source.object != root || self.document().resolve(&source.source).is_err() {
            return Err(Error::Stale("surface snap source"));
        }
        let scene = self.scene().ok_or(Error::Missing("surface snap scene"))?;
        if scene.epoch != source.scene_epoch || scene.publication_id() != source.scene_publication {
            return Err(Error::Stale("surface snap scene"));
        }
        let object = scene
            .objects
            .iter()
            .find(|object| {
                object.root == root
                    && object.path == source.path
                    && object.leaf == source.leaf
                    && object.leaf_generation == source.leaf_generation
                    && object.leaf_geometry_revision == source.leaf_geometry_revision
                    && object.brep_display_fingerprint == source.brep_display_fingerprint
                    && object.source == source.source
            })
            .ok_or(Error::Stale("surface snap instance"))?;
        let placement = object.display_transform()?;
        if placement != source.placement {
            return Err(Error::Stale("surface snap placement"));
        }
        let entity = scene.entity_for(&scene.document, object)?;
        if matches!(&entity.geometry, om9_core::geometry::Geometry::Surface(_))
            && object.buffers.is_trim_display_fallback() {
            return Err(Error::Unsupported("surface snap requires verified trim display"));
        }
        if matches!(&entity.geometry, om9_core::geometry::Geometry::Brep(_)) {
            let face = source.face.ok_or(Error::Missing("surface snap face"))?;
            let (witness, deflection) = source.face_witness.ok_or(Error::Missing("surface snap face witness"))?;
            let display = object.pick_brep_display(entity)?.ok_or(Error::Stale("surface snap display"))?;
            let index = display.topology.faces.iter().position(|id| *id == face).ok_or(Error::Stale("surface snap face"))?;
            let triangle = source.face_witness_triangle.and_then(|i| display.triangles.get(i)).ok_or(Error::Stale("surface snap face witness"))?;
            if deflection != display.deflection_mm || !({
                triangle[3] as usize == index && (0..3).all(|i| display.points.get(triangle[i] as usize) == Some(&witness[i]))
            }) { return Err(Error::Stale("surface snap face witness")); }
        } else if matches!(&entity.geometry, om9_core::geometry::Geometry::NativeBrep(_)) {
            let (witness, deflection) = source.face_witness.ok_or(Error::Missing("native surface snap witness"))?;
            let om9_scene::scene::RenderBuffers::Mesh { vertices, triangles, .. } = object.buffers.as_ref() else {
                return Err(Error::Stale("native surface snap display"));
            };
            let (indices, _) = source.face_witness_triangle.and_then(|index| triangles.get(index))
                .ok_or(Error::Stale("native surface snap witness"))?;
            if deflection != 0. || !(0..3).all(|index| vertices.get(indices[index] as usize) == Some(&witness[index])) {
                return Err(Error::Stale("native surface snap witness"));
            }
        } else if !matches!(&entity.geometry, om9_core::geometry::Geometry::Surface(_)) && source.face.is_none() {
            return Err(Error::Unsupported("surface snap face binding"));
        }
        Ok(object)
    }

    /// Resolve a surface candidate against its canonical geometry and source binding.
    pub fn refine_surface_snap(
        &self,
        candidate: &SnapCandidate,
        camera: &Camera,
        viewport: Viewport,
    ) -> Result<SnapCandidate> {
        self.validate_surface_candidate(candidate, camera, viewport)?;
        let _reservation=self.reserve_trim_snap_scratch(candidate)?;
        refine_captured_surface(self.document(), candidate, camera, viewport, self.coordinator().limits())
    }
    pub(crate) fn try_refine_surface_snap(
        &self, candidate: &SnapCandidate, camera: &Camera, viewport: Viewport,
    ) -> Result<Option<SnapCandidate>> {
        self.validate_surface_candidate(candidate, camera, viewport)?;
        let _reservation=self.reserve_trim_snap_scratch(candidate)?;
        try_refine_captured_surface(self.document(), candidate, camera, viewport, self.coordinator().limits())
    }
}
