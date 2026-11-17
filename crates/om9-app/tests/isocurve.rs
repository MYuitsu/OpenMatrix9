#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};

fn reopen_document_file(doc:&Document,limits:&Limits)->Document {
    let path=std::env::temp_dir().join(format!("om9-isocurve-reopen-{}.om9",uuid::Uuid::new_v4()));
    struct Cleanup(std::path::PathBuf);impl Drop for Cleanup {fn drop(&mut self){let _=std::fs::remove_file(&self.0);}}
    let _cleanup=Cleanup(path.clone());
    om9_archive::Archive::save(&path,doc,limits,&om9_core::transaction::Cancellation::default()).unwrap();
    om9_archive::Archive::open(&path,limits).unwrap()
}

fn finish_face_verification(controller: &mut Controller) -> om9_core::Result<()> {
    let deadline=std::time::Instant::now()+std::time::Duration::from_secs(30);
    while controller.face_verification_pending() {
        if let Some(result)=controller.poll_face_verification() { return result; }
        assert!(std::time::Instant::now()<deadline);
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    Ok(())
}

#[test]
fn captured_isocurve_worker_releases_document_and_clips_hole() {
    use om9_core::{budget::memory::MemoryClass, evaluation::{evaluate_curve,Parameter}, geometry::Transform,
        rhino_semantics::TrimmedSurface, transaction::Cancellation};
    use std::sync::{Arc,mpsc};
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    let trim=TrimmedSurface {underlying:surface.clone(),outer:vec![[0.,0.],[1.,0.],[1.,1.],[0.,1.]],
        inners:vec![vec![[0.4,0.4],[0.6,0.4],[0.6,0.6],[0.4,0.6]]],reversed:false};
    let source=om9_kernel::measurement::trimmed_surface_to_face_source_admitted(&trim,&limits,|_,_|Ok(())).unwrap();
    let shape=om9_kernel::brep::read_brep(&source,0.1,&limits).unwrap();
    let display=shape.display_liveness();
    let entity=Entity::new("trimmed face",Geometry::Brep(shape),None);let id=entity.id;
    let mut document=Document::with_template_tolerance("captured isocurve");document.entities.insert(id,entity);
    let controller=Controller::new(document,limits).unwrap();
    let stamp=controller.document().stamp(id).unwrap();
    let source=controller.coordinator().capture_brep_source(&stamp).unwrap();
    let bounds=om9_kernel::editing::face_isocurve_bounds_from_source(source.source(),None,&limits).unwrap();
    let value=(bounds[1][0]+bounds[1][1])*0.5;
    let cancel=Cancellation::default();let mut validation_calls=0;
    source.validate_canonical_admitted(&limits,|_,_|{validation_calls+=1;Ok(())}).unwrap();
    let mut calls=0;
    assert!(matches!(om9_kernel::editing::face_isocurve_from_captured_source_cancellable(&source,true,value,None,&limits,&cancel,|_,_|{
        calls+=1;if calls>validation_calls {Err(om9_core::Error::Budget("isocurve test"))} else {Ok(())}
    }),Err(om9_core::Error::Budget("isocurve test"))));
    assert!(calls>validation_calls);
    let canceled=Cancellation::default();canceled.cancel();
    assert!(matches!(om9_kernel::editing::face_isocurve_from_captured_source_cancellable(&source,true,value,None,&limits,&canceled,|_,_|Ok(())),Err(om9_core::Error::Canceled)));
    let mid_cancel=Cancellation::default();let mut calls=0;
    assert!(matches!(om9_kernel::editing::face_isocurve_from_captured_source_cancellable(&source,true,value,None,&limits,&mid_cancel,|_,_|{
        calls+=1;if calls>validation_calls {mid_cancel.cancel();Err(om9_core::Error::Canceled)} else {Ok(())}
    }),Err(om9_core::Error::Canceled)));
    let weak=Arc::downgrade(&controller.scene().unwrap().document);
    let mut reservation=controller.coordinator().reserve_external_memory(MemoryClass::Kernel,8*1024*1024,None).unwrap();
    let peer=controller.coordinator().memory_admission_peer().unwrap();
    let (send,receive)=mpsc::channel();
    let worker=std::thread::spawn(move || {
        receive.recv().unwrap();
        let cancel=Cancellation::default();
        let curves=om9_kernel::editing::face_isocurve_from_captured_source_cancellable(&source,true,value,None,&limits,&cancel,|live,pending| {
            if live.checked_add(pending).is_none_or(|needed|needed>reservation.bytes()) {return Err(om9_core::Error::Budget("test"));}
            reservation.set_allocated_bytes(live)?;reservation.recheck_pending()
        }).unwrap();
        drop(source);
        (curves,reservation)
    });
    drop(controller);
    assert!(weak.upgrade().is_none());assert!(!display.is_alive());
    send.send(()).unwrap();
    let (curves,reservation)=worker.join().unwrap();
    assert_eq!(curves.len(),2);
    for curve in &curves {for fraction in [0.,0.5,1.] {
        let point=evaluate_curve(curve,Parameter::Fraction(fraction),0,Transform::identity(),&limits).unwrap()[0];
        assert!(!(point[0]>4.+1e-6 && point[0]<6.-1e-6 && point[1]>4.+1e-6 && point[1]<6.-1e-6),"curve crosses hole: {point:?}");
    }}
    assert!(peer.external_reserved_bytes(MemoryClass::Kernel)>0);
    drop(curves);drop(reservation);
    assert_eq!(peer.external_reserved_bytes(MemoryClass::Kernel),0);
}

#[test]
fn default_brep_face_wires_use_source_and_clip_holes() {
    use om9_core::{evaluation::{evaluate_curve,Parameter},geometry::Transform,rhino_semantics::TrimmedSurface};
    use om9_kernel::editing::default_face_isocurves_from_source_admitted;
    let limits=Limits::default();
    let box_shape=om9_kernel::brep::box_brep([4.,6.,8.],0.1,&limits).unwrap();
    let wires=default_face_isocurves_from_source_admitted(box_shape.source(),&limits,|_,_|Ok(())).unwrap();
    assert_eq!(wires.len(),12);
    for profile in [0.5,0.01] {
        let remeshed=om9_kernel::brep::read_brep(box_shape.source(),profile,&limits).unwrap();
        let other=default_face_isocurves_from_source_admitted(remeshed.source(),&limits,|_,_|Ok(())).unwrap();
        assert_eq!(other,wires,"display quality must not alter source wires");
    }
    {
        use om9_scene::{camera::{Camera,Viewport},scene::{PickIntent,PickKind,RenderBuffers,Scene}};
        use std::sync::Arc;
        let mut document=Document::new("source wire scene");
        let entity=Entity::new("box",Geometry::Brep(box_shape.clone()),None);let id=entity.id;
        document.entities.insert(id,entity);
        let scene=Scene::capture(Arc::new(document),1,&limits).unwrap();
        let stamp=scene.document.stamp(id).unwrap();
        let lines:Vec<Vec<_>>=wires.iter().map(|curve| (0..=8).map(|i|
            evaluate_curve(curve,Parameter::Fraction(i as f64/8.),0,Transform::identity(),&limits).unwrap()[0]).collect()).collect();
        let (prepared,_)=Scene::prepare_brep_buffers(&box_shape,&limits,|_,_|Ok(())).unwrap();
        let (prepared,_)=prepared.with_interior_wires(&lines,&limits,|_,_|Ok(())).unwrap();
        let RenderBuffers::Mesh {edges,..}=prepared.as_ref() else {panic!("mesh")};
        assert_eq!(edges.len(),box_shape.display().segments.len()+12*8);
        let scene=scene.try_replace_root_brep_display(&stamp,&box_shape,prepared,[0,0],Arc::new(()),Arc::new(())).unwrap();
        let camera=Camera::orthographic([2.,3.,30.],[2.,3.,0.],[0.,1.,0.],10.).unwrap();
        let viewport=Viewport {origin:[0.,0.],size:[800.,800.]};
        let center=camera.project([2.,3.,8.],viewport).unwrap().unwrap().screen;
        assert!(scene.pick_kind(center,&camera,viewport,2.,PickIntent::Select,PickKind::Edge).unwrap().is_none());
        assert!(scene.pick_edge_candidates(center,&camera,viewport,2.,PickIntent::Select).unwrap().is_empty());
    }
    for curve in &wires {
        let p=evaluate_curve(curve,Parameter::Fraction(0.5),0,Transform::identity(),&limits).unwrap()[0];
        assert!((p[0]-2.).abs()<1e-7 && (p[1]-3.).abs()<1e-7
            || (p[0]-2.).abs()<1e-7 && (p[2]-4.).abs()<1e-7
            || (p[1]-3.).abs()<1e-7 && (p[2]-4.).abs()<1e-7, "{p:?}");
    }
    let cylinder=om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Cylinder {radius_mm:2.,height_mm:5.},0.1,&limits).unwrap();
    let cylinder_wires=default_face_isocurves_from_source_admitted(cylinder.source(),&limits,|_,_|Ok(())).unwrap();
    assert!(!cylinder_wires.is_empty());
    for curve in &cylinder_wires {
        for fraction in [0.,0.5,1.] {
            let p=evaluate_curve(curve,Parameter::Fraction(fraction),0,Transform::identity(),&limits).unwrap()[0];
            let radius=(p[0]*p[0]+p[1]*p[1]).sqrt();
            assert!(p[2]>=-1e-7 && p[2]<=5.+1e-7 && radius<=2.+1e-7);
            assert!((radius-2.).abs()<1e-6 || p[2].abs()<1e-6 || (p[2]-5.).abs()<1e-6, "{p:?}");
        }
    }
    let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    let trim=TrimmedSurface {underlying:surface,outer:vec![[0.1,0.1],[0.9,0.1],[0.9,0.9],[0.1,0.9]],
        inners:vec![vec![[0.4,0.4],[0.6,0.4],[0.6,0.6],[0.4,0.6]]],reversed:false};
    let source=om9_kernel::measurement::trimmed_surface_to_face_source_admitted(&trim,&limits,|_,_|Ok(())).unwrap();
    let wires=default_face_isocurves_from_source_admitted(&source,&limits,|_,_|Ok(())).unwrap();
    assert_eq!(wires.len(),4);
    for curve in &wires {
        for fraction in [0.,0.5,1.] {
            let p=evaluate_curve(curve,Parameter::Fraction(fraction),0,Transform::identity(),&limits).unwrap()[0];
            assert!((p[2]).abs()<1e-7 && p[0]>=1.-1e-7 && p[0]<=9.+1e-7 && p[1]>=1.-1e-7 && p[1]<=9.+1e-7);
            assert!(!(p[0]>4.+1e-7 && p[0]<6.-1e-7 && p[1]>4.+1e-7 && p[1]<6.-1e-7), "wire crosses hole: {p:?}");
        }
    }
    assert!(default_face_isocurves_from_source_admitted(&source,&limits,|_,pending|
        if pending>0 {Err(om9_core::Error::Budget("test"))} else {Ok(())}).is_err());
    let mut checkpoints=0;
    let cancelled=default_face_isocurves_from_source_admitted(box_shape.source(),&limits,|_,_| {
        checkpoints+=1;
        if checkpoints==6 {Err(om9_core::Error::Canceled)} else {Ok(())}
    });
    assert!(cancelled.is_err());
    assert_eq!(checkpoints,6,"native traversal must stop at a refused checkpoint");
    let limited=Limits {max_outputs:3,..limits};
    assert!(default_face_isocurves_from_source_admitted(&source,&limited,|_,_|Ok(())).is_err());
    assert!(default_face_isocurves_from_source_admitted("invalid",&limits,|_,_|Ok(())).is_err());
}

#[test]
fn default_brep_wires_use_interior_surface_knots() {
    use om9_core::{evaluation::{evaluate_curve,Parameter},geometry::{NurbsSurface,Transform},rhino_semantics::TrimmedSurface};
    let limits=Limits::default();
    let surface=NurbsSurface {degree:[1,1],size:[3,2],
        poles:vec![[0.,0.,0.],[0.,10.,0.],[2.5,0.,0.],[2.5,10.,0.],[10.,0.,0.],[10.,10.,0.]],
        weights:vec![1.;6],knots:[vec![0.,0.,0.25,1.,1.],vec![0.,0.,1.,1.]],
        domain:[[0.,1.],[0.,1.]],periodic:[false,false],reversed:false};
    let trim=TrimmedSurface {underlying:surface,outer:vec![[0.1,0.1],[0.9,0.1],[0.9,0.9],[0.1,0.9]],inners:vec![],reversed:false};
    let source=om9_kernel::measurement::trimmed_surface_to_face_source_admitted(&trim,&limits,|_,_|Ok(())).unwrap();
    let wires=om9_kernel::editing::default_face_isocurves_from_source_admitted(&source,&limits,|_,_|Ok(())).unwrap();
    assert_eq!(wires.len(),2);
    let mut vertical=false; let mut horizontal=false;
    for curve in wires {
        let a=evaluate_curve(&curve,Parameter::Fraction(0.),0,Transform::identity(),&limits).unwrap()[0];
        let b=evaluate_curve(&curve,Parameter::Fraction(1.),0,Transform::identity(),&limits).unwrap()[0];
        vertical |= (a[0]-2.5).abs()<1e-7 && (b[0]-2.5).abs()<1e-7;
        horizontal |= (a[1]-5.).abs()<1e-7 && (b[1]-5.).abs()<1e-7;
    }
    assert!(vertical && horizontal,"default wires must use the interior knot and midpoint");
}

#[test]
fn single_face_brep_isocurve_clips_hole_and_rejects_solid() {
    use om9_core::{evaluation::{evaluate_curve,Parameter},geometry::Transform,rhino_semantics::TrimmedSurface};
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    let trim=TrimmedSurface {underlying:surface,outer:vec![[0.1,0.1],[0.9,0.1],[0.9,0.9],[0.1,0.9]],
        inners:vec![vec![[0.4,0.4],[0.6,0.4],[0.6,0.6],[0.4,0.6]]],reversed:false};
    let source=om9_kernel::measurement::trimmed_surface_to_face_source_admitted(&trim,&limits,|_,_|Ok(())).unwrap();
    let brep=om9_kernel::brep::read_brep(&source,0.1,&limits).unwrap();
    let mut face=Entity::new("face",Geometry::Brep(brep),None);face.placement=Transform::translation([3.,-4.,7.]);let id=face.id;
    let box_entity=Entity::new("box",Geometry::Brep(om9_kernel::brep::box_brep([2.,2.,2.],0.1,&limits).unwrap()),None);let box_id=box_entity.id;
    let mut doc=Document::new("single face isocurve");doc.entities.insert(id,face);doc.entities.insert(box_id,box_entity);
    let mut c=Controller::new(reopen_document_file(&doc,&limits),limits).unwrap();
    c.submit("Point").unwrap();c.submit("100,100,100").unwrap();c.undo().unwrap();
    let before=c.document().clone();c.select_objects(&[box_id]).unwrap();assert!(c.submit("ExtractIsocurve").is_err());
    assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),1);
    c.select_objects(&[id]).unwrap();c.submit("ExtractIsocurve").unwrap();c.submit("U=0.5").unwrap();
    c.submit("").unwrap();let curves:Vec<_>=c.document().entities.values().filter(|e|e.label=="ExtractIsocurve").collect();assert_eq!(curves.len(),2);
    let expected=[[4.,1.,7.],[7.,1.,7.],[9.,1.,7.],[12.,1.,7.]];let mut seen=[false;4];
    for e in curves {let Geometry::Curve(curve)=&e.geometry else {panic!("curve")};
        for fraction in [0.,1.] {let p=evaluate_curve(curve,Parameter::Fraction(fraction),0,e.placement,&limits).unwrap()[0];
            let i=expected.iter().position(|q|(0..3).all(|axis|(p[axis]-q[axis]).abs()<1e-7)).expect("unexpected endpoint");
            assert!(!seen[i]);seen[i]=true;}}
    assert!(seen.into_iter().all(|v|v));let after=c.document().clone();
    c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    c.select_objects(&[id]).unwrap();c.submit("ExtractIsocurve").unwrap();c.submit("U=0.5").unwrap();
    c.delete_objects(&[id]).unwrap();let after_delete=c.document().clone();let redo_before=c.coordinator().redo_len();
    assert!(c.submit("").is_err());assert_eq!(c.document(),&after_delete);assert_eq!(c.coordinator().redo_len(),redo_before);
    assert!(!c.isocurve_option_tokens().is_empty());
}

#[test]
fn native_brep_isocurve_preserves_trim_and_history_after_reopen() {
    use om9_core::{evaluation::{evaluate_curve,Parameter},geometry::Transform,rhino_semantics::TrimmedSurface};
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    let trim=TrimmedSurface {underlying:surface,outer:vec![[0.1,0.1],[0.9,0.1],[0.9,0.9],[0.1,0.9]],
        inners:vec![vec![[0.4,0.4],[0.6,0.4],[0.6,0.6],[0.4,0.6]]],reversed:false};
    let source=om9_kernel::measurement::trimmed_surface_to_face_source_admitted(&trim,&limits,|_,_|Ok(())).unwrap();
    let brep=om9_kernel::brep::read_brep(&source,0.1,&limits).unwrap();
    let native=om9_kernel::brep::to_3dm(&brep,1e-6,&limits).unwrap();
    let mut malformed=native.clone();malformed.payload=vec![1,2,3,4].into();
    let invalid=Entity::new("bad native",Geometry::NativeBrep(malformed),None);let invalid_id=invalid.id;
    let solid=om9_kernel::brep::box_brep([2.,2.,2.],0.1,&limits).unwrap();
    let solid=Entity::new("native solid",Geometry::NativeBrep(om9_kernel::brep::to_3dm(&solid,1e-6,&limits).unwrap()),None);let solid_id=solid.id;
    let mut entity=Entity::new("native face",Geometry::NativeBrep(native),None);
    entity.placement=Transform::translation([3.,-4.,7.]);let id=entity.id;
    let mut doc=Document::with_template_tolerance("native isocurve");doc.entities.insert(id,entity);doc.entities.insert(invalid_id,invalid);doc.entities.insert(solid_id,solid);
    let mut c=Controller::new(reopen_document_file(&doc,&limits),limits).unwrap();
    c.submit("Point").unwrap();c.submit("100,100,100").unwrap();c.undo().unwrap();c.select_objects(&[id]).unwrap();
    let before=c.document().clone();
    for rejected in [invalid_id,solid_id] {c.select_objects(&[rejected]).unwrap();assert!(c.submit("ExtractIsocurve").is_err());
        assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),1);}
    for (option,menu,expected) in [("U=0.5",false,[[4.,1.,7.],[7.,1.,7.],[9.,1.,7.],[12.,1.,7.]]),
        ("V=0.5",true,[[8.,-3.,7.],[8.,0.,7.],[8.,2.,7.],[8.,5.,7.]])] {
        c.select_objects(&[id]).unwrap();
        if !menu {let pressure=c.coordinator().reserve_remaining_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::Kernel,None,c.scene_cache_usage().unwrap()).unwrap();
            assert!(matches!(c.submit("ExtractIsocurve"),Err(om9_core::Error::Budget(_))));assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),1);drop(pressure);}
        if menu {c.dispatch("OM9-CURVE-019",om9_app::session::InvocationOrigin::Menu).unwrap();} else {c.submit("ExtractIsocurve").unwrap();}
        c.submit(option).unwrap();
        if !menu {let pressure=c.coordinator().reserve_remaining_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::Kernel,None,c.scene_cache_usage().unwrap()).unwrap();
            assert!(matches!(c.submit(""),Err(om9_core::Error::Budget(_))));assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),1);
            assert!(!c.isocurve_option_tokens().is_empty());drop(pressure);}
        if !menu {let mut pressure=c.coordinator().reserve_remaining_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::Kernel,None,c.scene_cache_usage().unwrap()).unwrap();
            let remaining=pressure.bytes().checked_sub(128).unwrap();pressure.shrink_to(remaining).unwrap();
            assert!(matches!(c.submit(""),Err(om9_core::Error::Budget(_))));assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),1);
            assert!(!c.isocurve_option_tokens().is_empty());drop(pressure);}
        c.submit("").unwrap();
        let curves:Vec<_>=c.document().entities.values().filter(|e|e.label=="ExtractIsocurve").collect();assert_eq!(curves.len(),2);
        let mut seen=[false;4];
        for e in curves {let Geometry::Curve(curve)=&e.geometry else {panic!("curve")};
            for t in [0.,1.] {let p=evaluate_curve(curve,Parameter::Fraction(t),0,e.placement,&limits).unwrap()[0];
                let i=expected.iter().position(|q|(0..3).all(|axis|(p[axis]-q[axis]).abs()<1e-6)).expect("unexpected endpoint");
                assert!(!seen[i]);seen[i]=true;}}
        assert!(seen.into_iter().all(|v|v));let after=c.document().clone();
        c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);c.undo().unwrap();
    }
    c.select_objects(&[id]).unwrap();c.submit("ExtractIsocurve").unwrap();c.delete_objects(&[id]).unwrap();
    let deleted=c.document().clone();let redo=c.coordinator().redo_len();assert!(c.submit("").is_err());
    assert_eq!(c.document(),&deleted);assert_eq!(c.coordinator().redo_len(),redo);assert!(!c.isocurve_option_tokens().is_empty());
}

#[test]
fn native_multiface_cannot_select_original_brep_face_after_reopen() {
    use om9_scene::scene::Subelement;
    let limits=Limits::default();
    let brep=om9_kernel::brep::box_brep([4.,6.,8.],0.1,&limits).unwrap();
    let original_face=brep.topology().faces[0];
    let native=om9_kernel::brep::to_3dm(&brep,1e-6,&limits).unwrap();
    let entity=Entity::new("native box",Geometry::NativeBrep(native),None);let id=entity.id;
    let mut doc=Document::with_template_tolerance("native multiface isocurve");doc.entities.insert(id,entity);
    let mut c=Controller::new(reopen_document_file(&doc,&limits),limits).unwrap();
    c.submit("Point").unwrap();c.submit("100,100,100").unwrap();c.undo().unwrap();
    let before=c.document().clone();let redo=c.coordinator().redo_len();
    assert!(c.select_subobject(id,Subelement::BrepFace(original_face)).is_err());
    c.select_objects(&[id]).unwrap();
    assert!(c.submit("ExtractIsocurve").is_err());
    assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),redo);
}

#[test]
#[ignore = "requires the external openNURBS V5 example and native exchange"]
fn external_v5_native_face_isocurve_after_reopen() {
    use om9_core::{evaluation::{evaluate_curve,Parameter},transaction::{Cancellation,TransactionCoordinator,World}};
    use om9_exchange::{BrepMode,ImportMode,ImportOptions};use om9_scene::{camera::{Camera,Viewport},scene::{PickIntent,PickKind}};
    use sha2::{Digest,Sha256};
    let limits=Limits::default();let executable=std::path::PathBuf::from(std::env::var_os("OM9_EXCHANGE_NATIVE").unwrap());
    let fixture=std::path::PathBuf::from(std::env::var_os("OM9_EXTERNAL_3DM_FIXTURE").unwrap());
    let hash:[u8;32]=Sha256::digest(std::fs::read(&executable).unwrap()).into();
    let host=om9_app::three_dm_host::ThreeDmHost::admit(&executable,hash).unwrap();
    let doc=Document::with_template_tolerance("external isocurve");let doc_id=doc.id;
    let mut coordinator=TransactionCoordinator::new(World::new(vec![doc],&limits).unwrap(),limits).unwrap();
    host.import_with_options(&mut coordinator,doc_id,&fixture,&ImportOptions {mode:ImportMode::Geometry,brep:BrepMode::RetainOpaque,..Default::default()},&Cancellation::default()).unwrap();
    let imported=&coordinator.world().documents[&doc_id];let id=imported.entities.values().find(|e|matches!(e.geometry,Geometry::NativeBrep(_))).unwrap().id;
    let mut c=Controller::new(reopen_document_file(imported,&limits),limits).unwrap();
    c.submit("Point").unwrap();c.submit("40,40,40").unwrap();c.undo().unwrap();c.select_objects(&[id]).unwrap();let before=c.document().clone();
    let viewport=Viewport {origin:[0.,0.],size:[400.,400.]};
    let camera=Camera::orthographic([30.,3.,6.],[10.,3.,6.],[0.,0.,1.],20.).unwrap();
    let hit=c.scene().unwrap().pick_kind([200.,200.],&camera,viewport,5.,PickIntent::Select,PickKind::Face).unwrap().unwrap();
    c.toggle_native_isocurve_face_hit(&hit,&camera,viewport,[200.,200.],5.).unwrap();
    c.submit("ExtractIsocurve").unwrap();c.submit("").unwrap();
    let output=c.document().entities.values().find(|e|e.label=="ExtractIsocurve").expect("curve");
    let Geometry::Curve(curve)=&output.geometry else {panic!("curve")};
    let points=[0.,0.25,0.5,0.75,1.].map(|t|evaluate_curve(curve,Parameter::Fraction(t),0,output.placement,&limits).unwrap()[0]);
    let boundary=[[0.,0.],[8.,-1.],[7.,13.],[0.,12.]];
    for p in points {
        assert!((p[0]-10.).abs()<1e-6,"{p:?}");
        for i in 0..4 {let a=boundary[i];let b=boundary[(i+1)%4];let cross=(b[0]-a[0])*(p[2]-a[1])-(b[1]-a[1])*(p[1]-a[0]);
            assert!(cross>=-1e-5,"outside source face: {p:?}");}
    }
    for p in [points[0],points[4]] {
        assert!((0..4).any(|i| {let a=boundary[i];let b=boundary[(i+1)%4];let dx=b[0]-a[0];let dy=b[1]-a[1];
            let u=((p[1]-a[0])*dx+(p[2]-a[1])*dy)/(dx*dx+dy*dy);let u=u.clamp(0.,1.);
            (p[1]-a[0]-u*dx).hypot(p[2]-a[1]-u*dy)<1e-6}),"endpoint not on source boundary: {p:?}");
    }
    assert!((0..3).map(|i|(points[0][i]-points[4][i]).powi(2)).sum::<f64>().sqrt()>1.);
    let after=c.document().clone();c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
}

#[test]
fn native_multiface_viewport_pick_extracts_correct_face_and_history() {
    use om9_core::{evaluation::{evaluate_curve,Parameter},geometry::Transform};
    use om9_scene::{camera::{Camera,Viewport},scene::{PickIntent,PickKind}};
    let limits=Limits::default();
    let shape=om9_kernel::brep::box_brep([4.,6.,8.],0.1,&limits).unwrap();
    let native=om9_kernel::brep::to_3dm(&shape,1e-6,&limits).unwrap();
    let mut entity=Entity::new("native box",Geometry::NativeBrep(native),None);
    entity.placement=Transform::translation([7.,-3.,2.]);let id=entity.id;
    let mut doc=Document::with_template_tolerance("native face");doc.entities.insert(id,entity);
    let mut c=Controller::new(reopen_document_file(&doc,&limits),limits).unwrap();
    c.submit("Point").unwrap();c.submit("100,100,100").unwrap();c.undo().unwrap();let before=c.document().clone();
    let viewport=Viewport {origin:[0.,0.],size:[400.,400.]};
    for (index,(eye,target,axis,expected)) in [([9.,0.,30.],[9.,0.,6.],2,10.),([30.,0.,6.],[9.,0.,6.],0,11.)].into_iter().enumerate() {
        c.select_objects(&[id]).unwrap();
        let camera=Camera::orthographic(eye,target,[0.,1.,0.],20.).unwrap();
        let hit=c.scene().unwrap().pick_kind([200.,200.],&camera,viewport,5.,PickIntent::Select,PickKind::Face).unwrap().unwrap();
        assert!(c.toggle_native_isocurve_face_hit(&hit,&camera,viewport,[300.,200.],5.).is_err());
        assert!(c.subselection().is_empty());assert_eq!(c.coordinator().redo_len(),1);
        c.toggle_native_isocurve_face_hit(&hit,&camera,viewport,[200.,200.],5.).unwrap();
        if index==0 {
            let pressure=c.coordinator().reserve_remaining_external_memory_with_held(om9_core::budget::memory::MemoryClass::Kernel,None,c.scene_cache_usage().unwrap()).unwrap();
            assert!(matches!(c.submit("ExtractIsocurve"),Err(om9_core::Error::Budget(_))));
            assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),1);drop(pressure);
            c.submit("ExtractIsocurve").unwrap();
        } else {c.dispatch("OM9-CURVE-019",om9_app::session::InvocationOrigin::Menu).unwrap();}
        if index==0 {
            let pressure=c.coordinator().reserve_remaining_external_memory_with_held(om9_core::budget::memory::MemoryClass::Kernel,None,c.scene_cache_usage().unwrap()).unwrap();
            assert!(matches!(c.submit(""),Err(om9_core::Error::Budget(_))));assert_eq!(c.document(),&before);
            assert_eq!(c.coordinator().redo_len(),1);assert!(!c.isocurve_option_tokens().is_empty());drop(pressure);
        }
        c.request_isocurve_build_worker().unwrap();
        let deadline=std::time::Instant::now()+std::time::Duration::from_secs(30);
        if index==0 {
            let pressure=c.coordinator().reserve_remaining_external_memory_with_held(om9_core::budget::memory::MemoryClass::Kernel,None,c.scene_cache_usage().unwrap()).unwrap();
            let result=loop {if let Some(result)=c.poll_isocurve_build_worker(){break result;}
                assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(2));};
            assert!(matches!(result,Err(om9_core::Error::Budget(_))),"{result:?}");
            assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),1);
            assert!(!c.isocurve_option_tokens().is_empty());drop(pressure);
            c.request_isocurve_build_worker().unwrap();
        }
        loop {if let Some(result)=c.poll_isocurve_build_worker(){result.unwrap();break;}
            assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(2));}
        let output=c.document().entities.values().find(|entity|entity.label=="ExtractIsocurve").unwrap();
        let Geometry::Curve(curve)=&output.geometry else {panic!("curve")};
        for t in [0.,0.5,1.] {let p=evaluate_curve(curve,Parameter::Fraction(t),0,output.placement,&limits).unwrap()[0];
            assert!((p[axis]-expected).abs()<1e-6,"wrong face {p:?}");}
        let after=c.document().clone();c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);c.undo().unwrap();c.clear_subselection();
    }
    c.select_objects(&[id]).unwrap();
    let camera=Camera::orthographic([9.,0.,30.],[9.,0.,6.],[0.,1.,0.],20.).unwrap();
    let hit=c.scene().unwrap().pick_kind([200.,200.],&camera,viewport,5.,PickIntent::Select,PickKind::Face).unwrap().unwrap();
    c.toggle_native_isocurve_face_hit(&hit,&camera,viewport,[200.,200.],5.).unwrap();
    c.submit("ExtractIsocurve").unwrap();c.request_isocurve_build_worker().unwrap();c.clear_subselection();
    assert!(matches!(c.poll_isocurve_build_worker(),Some(Err(om9_core::Error::Stale(_)))));
    assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),1);
    assert!(!c.isocurve_option_tokens().is_empty());
    c.submit("Cancel").unwrap();c.select_objects(&[id]).unwrap();
    let hit=c.scene().unwrap().pick_kind([200.,200.],&camera,viewport,5.,PickIntent::Select,PickKind::Face).unwrap().unwrap();
    c.toggle_native_isocurve_face_hit(&hit,&camera,viewport,[200.,200.],5.).unwrap();c.submit("ExtractIsocurve").unwrap();
    c.delete_objects(&[id]).unwrap();let deleted=c.document().clone();let redo=c.coordinator().redo_len();
    assert!(c.submit("").is_err());assert_eq!(c.document(),&deleted);assert_eq!(c.coordinator().redo_len(),redo);
}

#[test]
fn native_cylinder_viewport_face_extracts_source_curve_at_two_profiles() {
    use om9_core::{evaluation::{evaluate_curve,Parameter},geometry::Transform};
    use om9_scene::{camera::{Camera,Viewport},scene::{PickIntent,PickKind}};
    let limits=Limits::default();let viewport=Viewport {origin:[0.,0.],size:[400.,400.]};
    for (deflection,perspective) in [(0.1,false),(0.02,true)] {
        let shape=om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Cylinder {radius_mm:2.,height_mm:5.},deflection,&limits).unwrap();
        let native=om9_kernel::brep::to_3dm(&shape,1e-6,&limits).unwrap();
        let mut entity=Entity::new("native cylinder",Geometry::NativeBrep(native),None);
        entity.placement=Transform([[1.5,0.25,0.,7.],[0.,1.,0.,-3.],[0.,0.,1.,2.],[0.,0.,0.,1.]]);let placement=entity.placement;let id=entity.id;
        let mut doc=Document::with_template_tolerance("native cylinder face");doc.entities.insert(id,entity);
        let mut c=Controller::new(reopen_document_file(&doc,&limits),limits).unwrap();let before=c.document().clone();c.select_objects(&[id]).unwrap();
        let target=placement.point([0.,0.37,2.5]).unwrap();let eye=[target[0]+20.,target[1],target[2]];
        let camera=if perspective {Camera::perspective(eye,target,[0.,0.,1.],0.7).unwrap()} else {Camera::orthographic(eye,target,[0.,0.,1.],10.).unwrap()};
        let hit=c.scene().unwrap().pick_kind([200.,200.],&camera,viewport,5.,PickIntent::Select,PickKind::Face).unwrap().unwrap();
        c.toggle_native_isocurve_face_hit(&hit,&camera,viewport,[200.,200.],5.).unwrap();
        let start=std::time::Instant::now();c.submit("ExtractIsocurve").unwrap();let start_elapsed=start.elapsed();
        let finish=std::time::Instant::now();c.request_isocurve_build_worker().unwrap();
        let deadline=std::time::Instant::now()+std::time::Duration::from_secs(30);
        loop {if let Some(result)=c.poll_isocurve_build_worker(){result.unwrap();break;}
            assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(2));}
        let finish_elapsed=finish.elapsed();
        eprintln!("native cylinder deflection={deflection} perspective={perspective} start={start_elapsed:?} finish={finish_elapsed:?}");
        let output=c.document().entities.values().find(|entity|entity.label=="ExtractIsocurve").unwrap();
        let Geometry::Curve(curve)=&output.geometry else {panic!("curve")};
        let inverse=om9_core::top11::inverse_affine(placement).unwrap();
        for t in [0.,0.25,0.5,0.75,1.] {let world=evaluate_curve(curve,Parameter::Fraction(t),0,output.placement,&limits).unwrap()[0];let p=inverse.point(world).unwrap();
            assert!((p[0].hypot(p[1])-2.).abs()<1e-6 && p[2]>=-1e-6 && p[2]<=5.+1e-6,"{p:?}");}
        let after=c.document().clone();c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }
}

#[test]
#[ignore = "diagnostic large NativeBrep timing; run explicitly"]
fn native_sphere_selected_face_timing_probe() {
    use om9_core::evaluation::{evaluate_curve,Parameter};
    use om9_scene::{camera::{Camera,Viewport},scene::{PickIntent,PickKind}};
    let limits=Limits::default();
    let shape=om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Sphere {radius_mm:3.},0.001,&limits).unwrap();
    let triangles=shape.display().triangles.len();
    let native=om9_kernel::brep::to_3dm(&shape,1e-6,&limits).unwrap();
    let entity=Entity::new("native sphere",Geometry::NativeBrep(native),None);let id=entity.id;
    let mut doc=Document::with_template_tolerance("native sphere timing");doc.entities.insert(id,entity);
    let mut c=Controller::new(reopen_document_file(&doc,&limits),limits).unwrap();let before=c.document().clone();c.select_objects(&[id]).unwrap();
    let viewport=Viewport {origin:[0.,0.],size:[400.,400.]};let camera=Camera::orthographic([20.,0.4,0.],[0.,0.4,0.],[0.,0.,1.],10.).unwrap();
    let hit=c.scene().unwrap().pick_kind([200.,200.],&camera,viewport,5.,PickIntent::Select,PickKind::Face).unwrap().unwrap();
    let select=std::time::Instant::now();c.toggle_native_isocurve_face_hit(&hit,&camera,viewport,[200.,200.],5.).unwrap();let select_ms=select.elapsed().as_secs_f64()*1000.;
    let start=std::time::Instant::now();c.submit("ExtractIsocurve").unwrap();let start_ms=start.elapsed().as_secs_f64()*1000.;
    let finish=std::time::Instant::now();c.submit("").unwrap();let finish_ms=finish.elapsed().as_secs_f64()*1000.;
    eprintln!("native sphere triangles={triangles} select_ms={select_ms:.2} start_ms={start_ms:.2} finish_ms={finish_ms:.2}");
    let result=c.document().entities.values().find(|e|e.label=="ExtractIsocurve").unwrap();
    let Geometry::Curve(curve)=&result.geometry else {panic!("curve")};
    for t in [0.,0.25,0.5,0.75,1.] {let p=evaluate_curve(curve,Parameter::Fraction(t),0,result.placement,&limits).unwrap()[0];
        assert!((p[0].hypot(p[1]).hypot(p[2])-3.).abs()<1e-6,"{p:?}");}
    let after=c.document().clone();c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
}

#[test]
#[ignore = "diagnostic for synchronous editable B-rep viewport pick"]
fn picked_brep_sphere_source_ray_timing_probe() {
    use om9_scene::{camera::{Camera,Viewport},scene::{PickIntent,PickKind}};
    let limits=Limits::default();
    let shape=om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Sphere {radius_mm:3.},0.001,&limits).unwrap();
    let triangles=shape.display().triangles.len();
    let entity=Entity::new("sphere",Geometry::Brep(shape),None);let id=entity.id;
    let mut doc=Document::with_template_tolerance("picked B-rep timing");doc.entities.insert(id,entity);
    let mut c=Controller::new(reopen_document_file(&doc,&limits),limits).unwrap();let before=c.document().clone();c.select_objects(&[id]).unwrap();
    let viewport=Viewport {origin:[0.,0.],size:[400.,400.]};let camera=Camera::orthographic([20.,0.4,0.],[0.,0.4,0.],[0.,0.,1.],10.).unwrap();
    let hit=c.scene().unwrap().pick_kind([200.,200.],&camera,viewport,5.,PickIntent::Select,PickKind::Face).unwrap().unwrap();
    let start=std::time::Instant::now();c.toggle_native_isocurve_face_hit(&hit,&camera,viewport,[200.,200.],5.).unwrap();let pick=start.elapsed();
    let start=std::time::Instant::now();c.submit("ExtractIsocurve").unwrap();let begin=start.elapsed();
    let start=std::time::Instant::now();c.submit("").unwrap();let finish=start.elapsed();
    eprintln!("editable B-rep triangles={triangles} pick={pick:?} start={begin:?} finish={finish:?}");
    let output=c.document().entities.values().find(|e|e.label=="ExtractIsocurve").unwrap();
    let Geometry::Curve(curve)=&output.geometry else {panic!("curve")};
    for t in [0.,0.25,0.5,0.75,1.] {let p=om9_core::evaluation::evaluate_curve(curve,om9_core::evaluation::Parameter::Fraction(t),0,output.placement,&limits).unwrap()[0];
        assert!((p.iter().map(|x|x*x).sum::<f64>().sqrt()-3.).abs()<1e-6,"{p:?}");}
    let after=c.document().clone();c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
}

#[test]
fn native_viewport_face_pick_through_source_hole_is_refused() {
    use om9_core::{geometry::Mesh,rhino_semantics::TrimmedSurface};
    use om9_scene::{camera::{Camera,Viewport},scene::{PickIntent,PickKind}};
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    let trim=TrimmedSurface {underlying:surface,outer:vec![[0.,0.],[1.,0.],[1.,1.],[0.,1.]],
        inners:vec![vec![[0.49,0.19],[0.51,0.19],[0.51,0.21],[0.49,0.21]]],reversed:false};
    let source=om9_kernel::measurement::trimmed_surface_to_face_source_admitted(&trim,&limits,|_,_|Ok(())).unwrap();
    let shape=om9_kernel::brep::read_brep(&source,0.1,&limits).unwrap();
    let mut native=om9_kernel::brep::to_3dm(&shape,1e-6,&limits).unwrap();
    native.display=std::sync::Arc::new(Mesh {vertices:vec![[1.,1.,0.],[9.,1.,0.],[1.,9.,0.]],
        faces:vec![vec![0,1,2]],normals:vec![],colors:vec![],corner_uvs:vec![]});
    let entity=Entity::new("bad display",Geometry::NativeBrep(native),None);let id=entity.id;
    let mut doc=Document::with_template_tolerance("native hole pick");doc.entities.insert(id,entity);
    let mut c=Controller::new(reopen_document_file(&doc,&limits),limits).unwrap();
    c.submit("Point").unwrap();c.submit("100,100,100").unwrap();c.undo().unwrap();let before=c.document().clone();
    c.select_objects(&[id]).unwrap();let camera=Camera::orthographic([5.,2.,10.],[5.,2.,0.],[0.,1.,0.],10.).unwrap();
    let viewport=Viewport {origin:[0.,0.],size:[400.,400.]};
    let hit=c.scene().unwrap().pick_kind([200.,200.],&camera,viewport,2.,PickIntent::Select,PickKind::Face).unwrap().unwrap();
    c.toggle_native_isocurve_face_hit(&hit,&camera,viewport,[200.,200.],2.).unwrap();
    assert!(c.submit("ExtractIsocurve").is_err());assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),1);
}

#[test]
fn native_multiface_source_ray_inspects_exact_face_without_display_indices() {
    use om9_core::evaluation::{evaluate_curve,Parameter};
    use om9_core::geometry::Transform;
    use om9_kernel::editing::{IsocurveFaceRay,ray_face_isocurve_from_source_admitted};
    let limits=Limits::default();
    let box_shape=om9_kernel::brep::box_brep([4.,6.,8.],0.1,&limits).unwrap();
    let native=om9_kernel::brep::to_3dm(&box_shape,1e-6,&limits).unwrap();
    let entity=Entity::new("native box",Geometry::NativeBrep(native),None);let id=entity.id;
    let mut document=Document::with_template_tolerance("native source ray");document.entities.insert(id,entity);
    let document=reopen_document_file(&document,&limits);
    let Geometry::NativeBrep(native)=&document.entities[&id].geometry else {panic!("native box")};
    let source=om9_kernel::brep::from_3dm_source_typed_admitted(&native.payload,1e-6,&limits,|_,_|Ok(())).unwrap();
    let resolve=|origin,direction,reference,max_distance| om9_kernel::editing::ray_face_isocurve_bounds_from_source(
        &source,origin,direction,reference,1e-6,max_distance,&limits);
    let top=resolve([2.,3.,20.],[0.,0.,-1.],[2.,3.,8.],1e-5).unwrap();
    let side=resolve([20.,3.,4.],[-1.,0.,0.],[4.,3.,4.],1e-5).unwrap();
    let spans=|b:[[f64;2];2]| {let mut spans=[b[0][1]-b[0][0],b[1][1]-b[1][0]];spans.sort_by(f64::total_cmp);spans};
    let a=spans(top);let b=spans(side);
    assert!((a[0]-4.).abs()<1e-6 && (a[1]-6.).abs()<1e-6,"{a:?}");
    assert!((b[0]-6.).abs()<1e-6 && (b[1]-8.).abs()<1e-6,"{b:?}");
    let top_witness=om9_kernel::editing::IsocurveFaceWitness {triangle:[[0.,0.,8.],[4.,0.,8.],[0.,6.,8.]],deflection:0.1,tolerance:1e-6};
    let side_witness=om9_kernel::editing::IsocurveFaceWitness {triangle:[[4.,0.,0.],[4.,6.,0.],[4.,0.,8.]],deflection:0.1,tolerance:1e-6};
    let ray=IsocurveFaceRay {origin:[2.,3.,20.],direction:[0.,0.,-1.],reference:[2.,3.,8.],tolerance:1e-6,max_distance:1e-5,witness:Some(top_witness)};
    assert!(om9_kernel::editing::ray_face_isocurve_hit_with_witness_from_source(&source,ray,&limits).is_ok());
    let wrong=IsocurveFaceRay {witness:Some(side_witness),..ray};
    assert!(om9_kernel::editing::ray_face_isocurve_hit_with_witness_from_source(&source,wrong,&limits).is_err());
    assert!(ray_face_isocurve_from_source_admitted(&source,true,(top[1][0]+top[1][1])*0.5,wrong,&limits,|_,_|Ok(())).is_err());
    let overflow=IsocurveFaceRay {witness:Some(om9_kernel::editing::IsocurveFaceWitness {deflection:f64::MAX,tolerance:f64::MAX,..top_witness}),..ray};
    assert!(om9_kernel::editing::ray_face_isocurve_hit_with_witness_from_source(&source,overflow,&limits).is_err());
    for (origin,direction,reference,bounds,axis,expected) in [
        ([2.,3.,20.],[0.,0.,-1.],[2.,3.,8.],top,2,8.),
        ([20.,3.,4.],[-1.,0.,0.],[4.,3.,4.],side,0,4.)] {
        let ray=IsocurveFaceRay {origin,direction,reference,tolerance:1e-6,max_distance:1e-5,witness:None};
        let value=(bounds[1][0]+bounds[1][1])*0.5;
        let curves=ray_face_isocurve_from_source_admitted(&source,true,value,ray,&limits,|_,_|Ok(())).unwrap();
        assert_eq!(curves.len(),1);
        for fraction in [0.,0.5,1.] {
            let p=evaluate_curve(&curves[0],Parameter::Fraction(fraction),0,Transform::identity(),&limits).unwrap()[0];
            assert!((p[axis]-expected).abs()<1e-6,"wrong source face: {p:?}");
        }
        assert!(ray_face_isocurve_from_source_admitted(&source,true,value,ray,&limits,|_,pending|
            if pending>0 {Err(om9_core::Error::Budget("bytes"))} else {Ok(())}).is_err());
    }
    assert!(resolve([2.,3.,20.],[0.,0.,-1.],[2.,3.,12.],0.1).is_err());
    assert!(resolve([-5.,3.,13.],[1.,0.,-1.],[0.,3.,8.],1e-5).is_err());
    assert!(resolve([2.,3.,20.],[0.,0.,0.],[2.,3.,8.],1e-5).is_err());
    let ambiguous=IsocurveFaceRay {origin:[-5.,3.,13.],direction:[1.,0.,-1.],reference:[0.,3.,8.],tolerance:1e-6,max_distance:1e-5,witness:None};
    assert!(ray_face_isocurve_from_source_admitted(&source,true,(top[1][0]+top[1][1])*0.5,ambiguous,&limits,|_,_|Ok(())).is_err());
    let mut limited=limits;limited.max_nodes=1;
    assert!(om9_kernel::editing::ray_face_isocurve_bounds_from_source(&source,[2.,3.,20.],[0.,0.,-1.],[2.,3.,8.],1e-6,1e-5,&limited).is_err());
    assert!(resolve([f64::NAN,3.,20.],[0.,0.,-1.],[2.,3.,8.],1e-5).is_err());
}

#[test]
fn native_source_ray_respects_trim_hole() {
    use om9_core::rhino_semantics::TrimmedSurface;
    use om9_core::evaluation::{evaluate_curve,Parameter};
    use om9_core::geometry::Transform;
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    let trim=TrimmedSurface {underlying:surface,outer:vec![[0.,0.],[1.,0.],[1.,1.],[0.,1.]],
        inners:vec![vec![[0.4,0.4],[0.6,0.4],[0.6,0.6],[0.4,0.6]]],reversed:false};
    let source=om9_kernel::measurement::trimmed_surface_to_face_source_admitted(&trim,&limits,|_,_|Ok(())).unwrap();
    let brep=om9_kernel::brep::read_brep(&source,0.1,&limits).unwrap();
    let native=om9_kernel::brep::to_3dm(&brep,1e-6,&limits).unwrap();
    let source=om9_kernel::brep::from_3dm_source_typed_admitted(&native.payload,1e-6,&limits,|_,_|Ok(())).unwrap();
    let resolve=|x,y| om9_kernel::editing::ray_face_isocurve_bounds_from_source(&source,[x,y,10.],[0.,0.,-1.],[x,y,0.],1e-6,1e-5,&limits);
    let bounds=resolve(2.,2.).unwrap();assert!(resolve(5.,5.).is_err());assert!(resolve(11.,5.).is_err());
    let ray=om9_kernel::editing::IsocurveFaceRay {origin:[2.,2.,10.],direction:[0.,0.,-1.],reference:[2.,2.,0.],tolerance:1e-6,max_distance:1e-5,witness:None};
    let curves=om9_kernel::editing::ray_face_isocurve_from_source_admitted(&source,true,(bounds[1][0]+bounds[1][1])*0.5,ray,&limits,|_,_|Ok(())).unwrap();
    assert_eq!(curves.len(),2);
    let points:Vec<_>=curves.iter().flat_map(|curve|[0.,1.].map(|f|
        evaluate_curve(curve,Parameter::Fraction(f),0,Transform::identity(),&limits).unwrap()[0])).collect();
    let axis=if (points[0][0]-5.).abs()<1e-6 {0} else {1};
    let mut coordinates:Vec<_>=points.iter().map(|p|{assert!((p[axis]-5.).abs()<1e-6 && p[2].abs()<1e-6);p[1-axis]}).collect();
    coordinates.sort_by(f64::total_cmp);
    for (got,want) in coordinates.into_iter().zip([0.,4.,6.,10.]) {assert!((got-want).abs()<1e-6);}
    let invalid=om9_kernel::editing::IsocurveFaceRay {reference:[5.,5.,0.],origin:[5.,5.,10.],..ray};
    assert!(om9_kernel::editing::ray_face_isocurve_from_source_admitted(&source,true,(bounds[1][0]+bounds[1][1])*0.5,invalid,&limits,|_,_|Ok(())).is_err());
}

#[test]
fn selected_drilled_face_worker_recovers_exact_remesh_and_extracts_isocurve() {
    use om9_kernel::{creation::{self,Primitive,Profile},editing::{self,BooleanOp,BooleanOutcome}};
    use om9_scene::scene::Subelement;
    let limits=Limits::default();let deflection=0.2;
    for extra_holes in [false,true] {
    let profile=Profile::new(vec![[-5.,-5.,0.],[5.,-5.,0.],[5.,5.,0.],[-5.,5.,0.]]).unwrap();
    let block=creation::extrude(&profile,[0.,0.,10.],true,deflection,&limits).unwrap();
    let cutter=creation::primitive(Primitive::Cylinder {radius_mm:2.,height_mm:10.},deflection,&limits).unwrap();
    let BooleanOutcome::Body(mut body)=editing::boolean(&block,&cutter,BooleanOp::Difference,deflection,&limits).unwrap() else {panic!("cut")};
    if extra_holes {
        let extra=creation::primitive(Primitive::Cylinder {radius_mm:0.75,height_mm:10.},deflection,&limits).unwrap();
        for (x,y) in [(-3.,-3.),(-3.,3.),(3.,-3.),(3.,3.)] {
            let placed=om9_kernel::transform_ops::transform_brep(&extra,&om9_core::geometry::Transform::translation([x,y,0.]),deflection,&limits).unwrap();
            let BooleanOutcome::Body(next)=editing::boolean(&body,&placed,BooleanOp::Difference,deflection,&limits).unwrap() else {panic!("extra cut")};
            body=next;
        }
    }
    assert_eq!(body.display().triangles.len()>512,extra_holes,"cover both sides of the previous worker threshold");
    let triangle=*body.display().triangles.iter().find(|tri|tri[..3].iter().all(|i|(body.display().points[*i as usize][2]-10.).abs()<1e-7)).unwrap();
    let witness=std::array::from_fn(|i|body.display().points[triangle[i] as usize]);
    assert!(editing::verify_face_display(&body,body.display(),triangle[3] as usize,witness,1e-6,&limits).is_err(),
        "fixture must require the worker's source-remesh proof");
    let face=body.topology().faces[triangle[3] as usize];
    let entity=Entity::new("drilled block",Geometry::Brep(body),None);let id=entity.id;
    let mut document=Document::with_template_tolerance("selected drilled face");document.entities.insert(id,entity);
    let mut c=Controller::new(reopen_document_file(&document,&limits),limits).unwrap();
    for menu in [false,true] {
    c.select_objects(&[id]).unwrap();c.select_subobject(id,Subelement::BrepFace(face)).unwrap();
    let before=c.document().clone();
    if menu {c.dispatch("OM9-CURVE-019",om9_app::session::InvocationOrigin::Menu).unwrap();}
    else {c.submit("ExtractIsocurve").unwrap();}
    assert!(c.isocurve_face_verification_pending());
    let deadline=std::time::Instant::now()+std::time::Duration::from_secs(30);
    loop {if let Some(result)=c.poll_face_verification(){result.unwrap();break;}
        assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(2));}
    c.submit("").unwrap();
    let curves:Vec<_>=c.document().entities.values().filter(|entity|entity.label=="ExtractIsocurve").collect();
    assert_eq!(curves.len(),2);
    let endpoints:Vec<_>=curves.iter().flat_map(|entity|{
        let Geometry::Curve(curve)=&entity.geometry else {panic!("curve")};
        [0.,1.].map(|t|om9_core::evaluation::evaluate_curve(curve,om9_core::evaluation::Parameter::Fraction(t),0,entity.placement,&limits).unwrap()[0])
    }).collect();
    let fixed=if endpoints.iter().all(|point|point[0].abs()<1e-6){0}else{1};
    let mut values:Vec<_>=endpoints.iter().map(|point|{assert!(point[fixed].abs()<1e-6 && (point[2]-10.).abs()<1e-6);point[1-fixed]}).collect();
    values.sort_by(f64::total_cmp);
    for (value,expected) in values.into_iter().zip([-5.,-2.,2.,5.]) {assert!((value-expected).abs()<1e-6);}
    let after=c.document().clone();c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);c.undo().unwrap();
    c.clear_subselection();
    }
    c.select_objects(&[id]).unwrap();c.select_subobject(id,Subelement::BrepFace(face)).unwrap();
    let before=c.document().clone();let redo=c.coordinator().redo_len();
    c.submit("ExtractIsocurve").unwrap();assert!(c.isocurve_face_verification_pending());
    c.clear_subselection();
    assert!(finish_face_verification(&mut c).is_err());
    assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),redo);
    }
}

#[test]
fn picked_circular_hole_brep_isocurve_uses_the_selected_source_face() {
    use om9_kernel::{creation::{self,Primitive,Profile},editing::{self,BooleanOp,BooleanOutcome}};
    use om9_scene::{camera::{Camera,Viewport},scene::{PickIntent,PickKind}};
    let limits=Limits::default();
    let profile=Profile::new(vec![[-5.,-5.,0.],[5.,-5.,0.],[5.,5.,0.],[-5.,5.,0.]]).unwrap();
    let block=creation::extrude(&profile,[0.,0.,10.],true,0.2,&limits).unwrap();
    let cutter=creation::primitive(Primitive::Cylinder {radius_mm:2.,height_mm:10.},0.2,&limits).unwrap();
    let BooleanOutcome::Body(body)=editing::boolean(&block,&cutter,BooleanOp::Difference,0.2,&limits).unwrap() else {panic!("cut")};
    let entity=Entity::new("drilled block",Geometry::Brep(body),None);let id=entity.id;
    let mut doc=Document::with_template_tolerance("picked circular hole");doc.entities.insert(id,entity);
    let mut c=Controller::new(reopen_document_file(&doc,&limits),limits).unwrap();
    let before=c.document().clone();
    let camera=Camera::orthographic([3.,1.,30.],[3.,1.,10.],[0.,1.,0.],14.).unwrap();
    let viewport=Viewport {origin:[0.,0.],size:[400.,400.]};
    let hit=c.scene().unwrap().pick_kind([200.,200.],&camera,viewport,2.,PickIntent::Select,PickKind::Face).unwrap().unwrap();
    c.toggle_native_isocurve_face_hit(&hit,&camera,viewport,[200.,200.],2.).unwrap();
    c.submit("ExtractIsocurve").unwrap();c.request_isocurve_build_worker().unwrap();
    let deadline=std::time::Instant::now()+std::time::Duration::from_secs(30);
    loop {if let Some(result)=c.poll_isocurve_build_worker(){result.unwrap();break;}
        assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(2));}
    let curves:Vec<_>=c.document().entities.values().filter(|e|e.label=="ExtractIsocurve").collect();
    assert_eq!(curves.len(),2,"the center isocurve must be clipped by the hole");
    let points:Vec<_>=curves.iter().flat_map(|e| {
        let Geometry::Curve(curve)=&e.geometry else {panic!("curve")};
        [0.,1.].map(|t|om9_core::evaluation::evaluate_curve(curve,om9_core::evaluation::Parameter::Fraction(t),0,e.placement,&limits).unwrap()[0])
    }).collect();
    let fixed_axis=if points.iter().all(|p|p[0].abs()<1e-6) {0} else {1};
    let mut values:Vec<_>=points.iter().map(|p| {assert!(p[fixed_axis].abs()<1e-6 && (p[2]-10.).abs()<1e-6,"{p:?}");p[1-fixed_axis]}).collect();
    values.sort_by(f64::total_cmp);
    for (got,expected) in values.into_iter().zip([-5.,-2.,2.,5.]) {assert!((got-expected).abs()<1e-6,"{got} != {expected}");}
    let after=c.document().clone();c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    let mut stale=Controller::new(reopen_document_file(&doc,&limits),limits).unwrap();
    stale.submit("Point").unwrap();stale.submit("100,100,100").unwrap();stale.undo().unwrap();let unchanged=stale.document().clone();
    let hit=stale.scene().unwrap().pick_kind([200.,200.],&camera,viewport,2.,PickIntent::Select,PickKind::Face).unwrap().unwrap();
    stale.toggle_native_isocurve_face_hit(&hit,&camera,viewport,[200.,200.],2.).unwrap();
    stale.submit("ExtractIsocurve").unwrap();stale.request_isocurve_build_worker().unwrap();stale.clear_subselection();
    assert!(matches!(stale.poll_isocurve_build_worker(),Some(Err(om9_core::Error::Stale(_)))));
    assert_eq!(stale.document(),&unchanged);assert_eq!(stale.coordinator().redo_len(),1);
}

#[test]
fn source_witness_samples_do_not_prove_triangle_avoids_trim_hole() {
    use om9_core::rhino_semantics::TrimmedSurface;
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    let trim=TrimmedSurface {underlying:surface,outer:vec![[0.,0.],[1.,0.],[1.,1.],[0.,1.]],
        inners:vec![vec![[0.49,0.19],[0.51,0.19],[0.51,0.21],[0.49,0.21]]],reversed:false};
    let source=om9_kernel::measurement::trimmed_surface_to_face_source_admitted(&trim,&limits,|_,_|Ok(())).unwrap();
    let witness=om9_kernel::editing::IsocurveFaceWitness {triangle:[[1.,1.,0.],[9.,1.,0.],[1.,9.,0.]],deflection:0.,tolerance:1e-6};
    // (5,2) lies inside both this triangle and the hole, while the vertices
    // and centroid are all on material. Sampling alone accepts the witness.
    assert!(om9_kernel::editing::face_isocurve_bounds_from_source(&source,Some(witness),&limits).is_ok());
}

#[test]
fn malformed_brep_display_through_trim_hole_is_refused() {
    use om9_core::rhino_semantics::TrimmedSurface;
    use om9_scene::{camera::{Camera,Viewport},scene::{PickIntent,PickKind,Subelement}};
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    let trim=TrimmedSurface {underlying:surface,outer:vec![[0.,0.],[1.,0.],[1.,1.],[0.,1.]],
        inners:vec![vec![[0.49,0.19],[0.51,0.19],[0.51,0.21],[0.49,0.21]]],reversed:false};
    let source=om9_kernel::measurement::trimmed_surface_to_face_source_admitted(&trim,&limits,|_,_|Ok(())).unwrap();
    let brep=om9_kernel::brep::read_brep(&source,0.1,&limits).unwrap();
    let mut display=brep.display().clone();let first=display.points.len() as u32;
    display.points.extend([[1.,1.,0.],[9.,1.,0.],[1.,9.,0.]]);
    display.triangles=vec![[first,first+1,first+2,0]];
    let face=display.topology.faces[0];
    let entity=Entity::new("invalid display",Geometry::Brep(om9_core::brep::Brep::from_native(source,display,&limits).unwrap()),None);let id=entity.id;
    let mut doc=Document::with_template_tolerance("hole pick");doc.entities.insert(id,entity);
    let mut c=Controller::new(reopen_document_file(&doc,&limits),limits).unwrap();
    c.submit("Point").unwrap();c.submit("100,100,100").unwrap();c.undo().unwrap();let before=c.document().clone();
    let camera=Camera::orthographic([5.,2.,10.],[5.,2.,0.],[0.,1.,0.],10.).unwrap();
    let viewport=Viewport {origin:[0.,0.],size:[400.,400.]};
    let hit=c.scene().unwrap().pick_kind([200.,200.],&camera,viewport,2.,PickIntent::Select,PickKind::Face).unwrap().unwrap();
    assert_eq!(hit.subset,Subelement::BrepFace(face));assert!((hit.world_point[0]-5.).abs()<1e-7 && (hit.world_point[1]-2.).abs()<1e-7);
    c.select_subobject(id,hit.subset).unwrap();
    c.submit("ExtractIsocurve").unwrap();
    assert!(finish_face_verification(&mut c).is_err());
    assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),1);
}

#[test]
fn small_trim_hole_crossed_by_display_triangle_is_refused() {
    use om9_core::rhino_semantics::TrimmedSurface;
    use om9_scene::{camera::{Camera,Viewport},scene::{PickIntent,PickKind,Subelement}};
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    // A 0.004 mm square hole, much larger than the 1e-6 mm vertex tolerance.
    let trim=TrimmedSurface {underlying:surface,outer:vec![[0.,0.],[1.,0.],[1.,1.],[0.,1.]],
        inners:vec![vec![[0.4998,0.1998],[0.5002,0.1998],[0.5002,0.2002],[0.4998,0.2002]]],reversed:false};
    let source=om9_kernel::measurement::trimmed_surface_to_face_source_admitted(&trim,&limits,|_,_|Ok(())).unwrap();
    let brep=om9_kernel::brep::read_brep(&source,0.1,&limits).unwrap();
    let mut display=brep.display().clone();let first=display.points.len() as u32;
    display.points.extend([[1.,1.,0.],[9.,1.,0.],[1.,9.,0.]]);display.triangles=vec![[first,first+1,first+2,0]];
    let face=display.topology.faces[0];
    let entity=Entity::new("invalid small hole display",Geometry::Brep(om9_core::brep::Brep::from_native(source,display,&limits).unwrap()),None);let id=entity.id;
    let mut doc=Document::with_template_tolerance("small hole pick");doc.entities.insert(id,entity);
    let mut c=Controller::new(reopen_document_file(&doc,&limits),limits).unwrap();
    c.submit("Point").unwrap();c.submit("100,100,100").unwrap();c.undo().unwrap();let before=c.document().clone();
    let camera=Camera::orthographic([5.,2.,10.],[5.,2.,0.],[0.,1.,0.],10.).unwrap();
    let viewport=Viewport {origin:[0.,0.],size:[400.,400.]};
    let hit=c.scene().unwrap().pick_kind([200.,200.],&camera,viewport,2.,PickIntent::Select,PickKind::Face).unwrap().unwrap();
    assert_eq!(hit.subset,Subelement::BrepFace(face));
    c.select_subobject(id,hit.subset).unwrap();
    c.submit("ExtractIsocurve").unwrap();
    assert!(finish_face_verification(&mut c).is_err(),"triangle crossing the small hole was accepted");
    assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),1);
    let mut c=Controller::new(reopen_document_file(&doc,&limits),limits).unwrap();
    c.submit("Point").unwrap();c.submit("100,100,100").unwrap();c.undo().unwrap();let before=c.document().clone();
    let hit=c.scene().unwrap().pick_kind([200.,200.],&camera,viewport,2.,PickIntent::Select,PickKind::Face).unwrap().unwrap();
    c.toggle_native_isocurve_face_hit(&hit,&camera,viewport,[200.,200.],2.).unwrap();
    assert!(c.submit("ExtractIsocurve").is_err(),"source ray must refuse a hit in the hole");
    assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),1);
}

#[test]
fn small_trim_hole_with_valid_display_remains_usable() {
    use om9_core::rhino_semantics::TrimmedSurface;
    use om9_scene::scene::Subelement;
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    let trim=TrimmedSurface {underlying:surface,outer:vec![[0.,0.],[1.,0.],[1.,1.],[0.,1.]],
        inners:vec![vec![[0.4998,0.1998],[0.5002,0.1998],[0.5002,0.2002],[0.4998,0.2002]]],reversed:false};
    let source=om9_kernel::measurement::trimmed_surface_to_face_source_admitted(&trim,&limits,|_,_|Ok(())).unwrap();
    let brep=om9_kernel::brep::read_brep(&source,0.01,&limits).unwrap();
    let face=brep.topology().faces[0];
    let entity=Entity::new("valid small hole",Geometry::Brep(brep),None);let id=entity.id;
    let mut doc=Document::with_template_tolerance("valid small hole");doc.entities.insert(id,entity);
    let mut c=Controller::new(reopen_document_file(&doc,&limits),limits).unwrap();let before=c.document().clone();
    c.select_subobject(id,Subelement::BrepFace(face)).unwrap();c.submit("ExtractIsocurve").unwrap();finish_face_verification(&mut c).unwrap();
    c.submit("U=0.5").unwrap();c.submit("").unwrap();
    assert!(c.document().entities.values().any(|e|e.label=="ExtractIsocurve"));
    let after=c.document().clone();c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
}

#[test]
fn rotated_planar_face_rejects_triangle_spanning_hole() {
    use om9_core::rhino_semantics::TrimmedSurface;
    let limits=Limits::default();let k=std::f64::consts::FRAC_1_SQRT_2;
    let surface=om9_core::surface_kernel::plane([0.,0.,0.],[k,k,0.],[0.,0.,1.],10.,10.,&limits).unwrap();
    let trim=TrimmedSurface {underlying:surface,outer:vec![[0.,0.],[1.,0.],[1.,1.],[0.,1.]],
        inners:vec![vec![[0.49,0.19],[0.51,0.19],[0.51,0.21],[0.49,0.21]]],reversed:false};
    let source=om9_kernel::measurement::trimmed_surface_to_face_source_admitted(&trim,&limits,|_,_|Ok(())).unwrap();
    let brep=om9_kernel::brep::read_brep(&source,0.1,&limits).unwrap();
    let mut display=brep.display().clone();let first=display.points.len() as u32;
    let points=[[k,k,1.],[9.*k,9.*k,1.],[k,k,9.]];
    display.points.extend(points);display.triangles=vec![[first,first+1,first+2,0]];
    assert!(om9_kernel::editing::verify_face_display(&brep,&display,0,points,1e-6,&limits).is_err());
}

#[test]
fn concave_outer_does_not_use_rectangle_shortcut() {
    use om9_core::rhino_semantics::TrimmedSurface;
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    let trim=TrimmedSurface {underlying:surface,
        outer:vec![[0.,0.],[1.,0.],[1.,1.],[0.65,1.],[0.65,0.3],[0.55,0.3],[0.55,1.],[0.,1.]],
        inners:vec![vec![[0.05,0.2],[0.1,0.2],[0.1,0.3],[0.05,0.3]]],reversed:false};
    let source=om9_kernel::measurement::trimmed_surface_to_face_source_admitted(&trim,&limits,|_,_|Ok(())).unwrap();
    let brep=om9_kernel::brep::read_brep(&source,0.1,&limits).unwrap();
    let mut display=brep.display().clone();let first=display.points.len() as u32;
    let points=[[1.,1.,0.],[9.,1.,0.],[1.,9.,0.]];
    display.points.extend(points);display.triangles=vec![[first,first+1,first+2,0]];
    // The hole bound only touches the triangle's box, but the triangle also
    // crosses the concave outer notch and must still receive the full check.
    assert!(om9_kernel::editing::verify_face_display(&brep,&display,0,points,1e-6,&limits).is_err());
}

#[test]
fn rectangular_face_rejects_triangle_spanning_two_holes() {
    use om9_core::rhino_semantics::TrimmedSurface;
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    let trim=TrimmedSurface {underlying:surface,outer:vec![[0.,0.],[1.,0.],[1.,1.],[0.,1.]],
        inners:vec![vec![[0.49,0.19],[0.51,0.19],[0.51,0.21],[0.49,0.21]],
                    vec![[0.19,0.49],[0.21,0.49],[0.21,0.51],[0.19,0.51]]],reversed:false};
    let source=om9_kernel::measurement::trimmed_surface_to_face_source_admitted(&trim,&limits,|_,_|Ok(())).unwrap();
    let brep=om9_kernel::brep::read_brep(&source,0.1,&limits).unwrap();
    let mut display=brep.display().clone();let first=display.points.len() as u32;
    let points=[[1.,1.,0.],[9.,1.,0.],[1.,9.,0.]];
    display.points.extend(points);display.triangles=vec![[first,first+1,first+2,0]];
    assert!(om9_kernel::editing::verify_face_display(&brep,&display,0,points,1e-6,&limits).is_err());
}

#[test]
fn selected_planar_brep_with_hole_uses_valid_source_display() {
    use om9_core::rhino_semantics::TrimmedSurface;
    use om9_scene::scene::Subelement;
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    let trim=TrimmedSurface {underlying:surface,outer:vec![[0.,0.],[1.,0.],[1.,1.],[0.,1.]],
        inners:vec![vec![[0.4,0.4],[0.6,0.4],[0.6,0.6],[0.4,0.6]]],reversed:false};
    let source=om9_kernel::measurement::trimmed_surface_to_face_source_admitted(&trim,&limits,|_,_|Ok(())).unwrap();
    let brep=om9_kernel::brep::read_brep(&source,0.1,&limits).unwrap();let face=brep.topology().faces[0];
    let entity=Entity::new("trimmed face",Geometry::Brep(brep),None);let id=entity.id;
    let mut doc=Document::with_template_tolerance("valid hole");doc.entities.insert(id,entity);
    let mut c=Controller::new(reopen_document_file(&doc,&limits),limits).unwrap();let before=c.document().clone();
    c.select_subobject(id,Subelement::BrepFace(face)).unwrap();c.submit("ExtractIsocurve").unwrap();finish_face_verification(&mut c).unwrap();
    c.submit("U=0.5").unwrap();c.submit("").unwrap();
    assert_eq!(c.document().entities.values().filter(|entity|entity.label=="ExtractIsocurve").count(),2);
    let after=c.document().clone();c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
}

#[test]
#[ignore = "diagnostic timing, run explicitly"]
fn planar_face_validation_timing_probe() {
    use om9_core::rhino_semantics::TrimmedSurface;
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    for hole in [false,true] {
    let trim=TrimmedSurface {underlying:surface.clone(),outer:vec![[0.,0.],[1.,0.],[1.,1.],[0.,1.]],
        inners:if hole {vec![vec![[0.4,0.4],[0.6,0.4],[0.6,0.6],[0.4,0.6]]]} else {vec![]},reversed:false};
    let source=om9_kernel::measurement::trimmed_surface_to_face_source_admitted(&trim,&limits,|_,_|Ok(())).unwrap();
    let brep=om9_kernel::brep::read_brep(&source,0.1,&limits).unwrap();
    let mut display=brep.display().clone();display.points.clear();display.triangles.clear();
    let n=10u32;
    for y in 0..=n {for x in 0..=n {display.points.push([x as f64,y as f64,0.]);}}
    for y in 0..n {for x in 0..n {if hole && (4..6).contains(&x) && (4..6).contains(&y) {continue;}
        let i=y*(n+1)+x;display.triangles.push([i,i+1,i+n+1,0]);display.triangles.push([i+1,i+n+2,i+n+1,0]);}}
    let witness=[display.points[0],display.points[1],display.points[(n+1) as usize]];
    let start=std::time::Instant::now();
    om9_kernel::editing::verify_face_display(&brep,&display,0,witness,1e-6,&limits).unwrap();
    eprintln!("planar verification: hole={hole}, {} triangles, {:?}",display.triangles.len(),start.elapsed());
    }
}

#[test]
fn native_brep_cylinder_isocurves_preserve_source_and_placement() {
    use om9_core::{evaluation::{evaluate_curve,Parameter},geometry::Transform,rhino_semantics::TrimmedSurface};
    let limits=Limits::default();let surface=om9_core::surface_kernel::cylinder([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,5.,&limits).unwrap();
    let trim=TrimmedSurface {underlying:surface.clone(),outer:vec![[0.1,0.2],[0.3,0.2],[0.3,0.8],[0.1,0.8]],
        inners:vec![vec![[0.15,0.4],[0.2,0.4],[0.2,0.6],[0.15,0.6]]],reversed:false};
    let source=om9_kernel::measurement::trimmed_surface_to_face_source_admitted(&trim,&limits,|_,_|Ok(())).unwrap();
    let brep=om9_kernel::brep::read_brep(&source,0.1,&limits).unwrap();
    let native=om9_kernel::brep::to_3dm(&brep,1e-6,&limits).unwrap();
    let mut entity=Entity::new("native cylinder",Geometry::NativeBrep(native),None);entity.placement=Transform::translation([7.,-3.,2.]);let id=entity.id;
    let mut doc=Document::with_template_tolerance("native cylinder isocurve");doc.entities.insert(id,entity);
    let mut c=Controller::new(reopen_document_file(&doc,&limits),limits).unwrap();
    c.submit("Point").unwrap();c.submit("100,100,100").unwrap();c.undo().unwrap();let before=c.document().clone();
    let placed=|uv:[f64;2]| {let p=om9_core::surface_kernel::evaluate(&surface,uv[0],uv[1],&limits).unwrap();[p[0]+7.,p[1]-3.,p[2]+2.]};
    for (option,uvs) in [("U=0.5",[[0.1,0.5],[0.15,0.5],[0.2,0.5],[0.3,0.5]]),
                         ("V=0.175",[[0.175,0.2],[0.175,0.4],[0.175,0.6],[0.175,0.8]])] {
        c.select_objects(&[id]).unwrap();c.submit("ExtractIsocurve").unwrap();c.submit(option).unwrap();c.submit("").unwrap();
        let curves:Vec<_>=c.document().entities.values().filter(|e|e.label=="ExtractIsocurve").collect();assert_eq!(curves.len(),2);
        let expected=uvs.map(placed);let mut seen=[false;4];
        for e in curves {let Geometry::Curve(curve)=&e.geometry else {panic!("curve")};let mut indices=[0;2];
            for (slot,t) in [0.,1.].into_iter().enumerate() {let p=evaluate_curve(curve,Parameter::Fraction(t),0,e.placement,&limits).unwrap()[0];
                let i=expected.iter().position(|q|(0..3).all(|axis|(p[axis]-q[axis]).abs()<1e-6)).expect("unexpected endpoint");
                assert!(!seen[i]);seen[i]=true;indices[slot]=i;}
            assert_eq!(indices[0]/2,indices[1]/2);
            let uv=[(uvs[indices[0]][0]+uvs[indices[1]][0])*0.5,(uvs[indices[0]][1]+uvs[indices[1]][1])*0.5];
            let expected_mid=placed(uv);let mid=evaluate_curve(curve,Parameter::Fraction(0.5),0,e.placement,&limits).unwrap()[0];
            assert!((0..3).all(|i|(mid[i]-expected_mid[i]).abs()<1e-6));
        }
        assert!(seen.into_iter().all(|v|v));let after=c.document().clone();
        c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);c.undo().unwrap();
    }
}

#[test]
fn selected_box_face_isocurve_follows_selected_source_face() {
    use om9_core::evaluation::{evaluate_curve,Parameter};
    use om9_scene::scene::Subelement;
    let limits=Limits::default();let brep=om9_kernel::brep::box_brep([4.,6.,8.],0.1,&limits).unwrap();
    let areas:Vec<_>=(0..brep.topology().faces.len()).map(|i|{
        let face=om9_kernel::editing::extract_face(&brep,i,0.1,&limits).unwrap();
        om9_kernel::creation::measure(&face,&limits).unwrap().area_mm2
    }).collect();
    let other=(1..areas.len()).find(|i|(areas[*i]-areas[0]).abs()>1.).unwrap();
    let mut display=brep.display().clone();display.faces.swap(0,other);
    for item in &mut display.face_edges {if item[0]==0 {item[0]=other as u32;} else if item[0] as usize==other {item[0]=0;}}
    for item in &mut display.triangles {if item[3]==0 {item[3]=other as u32;} else if item[3] as usize==other {item[3]=0;}}
    let brep=om9_core::brep::Brep::from_native(brep.source().to_owned(),display,&limits).unwrap();
    let keys=brep.topology().faces.clone();let display=brep.display().clone();
    let entity=Entity::new("box",Geometry::Brep(brep),None);let id=entity.id;
    let mut doc=Document::with_template_tolerance("box isocurve");doc.entities.insert(id,entity);
    let mut c=Controller::new(reopen_document_file(&doc,&limits),limits).unwrap();
    c.submit("Point").unwrap();c.submit("100,100,100").unwrap();c.undo().unwrap();let before=c.document().clone();
    for face in [0usize,1] {
        if face==0 {c.select_objects(&[id]).unwrap();} else {c.select_objects(&[]).unwrap();}
        c.select_subobject(id,Subelement::BrepFace(keys[face])).unwrap();
        if face==0 {let pressure=c.coordinator().reserve_remaining_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::Kernel,None,c.scene_cache_usage().unwrap()).unwrap();
            assert!(matches!(c.submit("ExtractIsocurve"),Err(om9_core::Error::Budget(_))));
            assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),1);drop(pressure);}
        let tri=display.triangles.iter().find(|t|t[3] as usize==face).unwrap();
        let p=[display.points[tri[0] as usize],display.points[tri[1] as usize],display.points[tri[2] as usize]];
        let axis=(0..3).find(|&a|(p[0][a]-p[1][a]).abs()<1e-8 && (p[0][a]-p[2][a]).abs()<1e-8).unwrap();
        if face==0 {c.submit("ExtractIsocurve").unwrap();} else {c.dispatch("OM9-CURVE-019",om9_app::session::InvocationOrigin::Menu).unwrap();}
        finish_face_verification(&mut c).unwrap();
        if face==0 {let pressure=c.coordinator().reserve_remaining_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::Kernel,None,c.scene_cache_usage().unwrap()).unwrap();
            assert!(matches!(c.submit(""),Err(om9_core::Error::Budget(_))));assert_eq!(c.document(),&before);
            assert_eq!(c.coordinator().redo_len(),1);assert!(!c.isocurve_option_tokens().is_empty());drop(pressure);}
        c.submit("").unwrap();
        let outputs:Vec<_>=c.document().entities.values().filter(|e|e.label=="ExtractIsocurve").collect();assert_eq!(outputs.len(),1);
        let Geometry::Curve(curve)=&outputs[0].geometry else {panic!("curve")};
        let points=[0.,0.5,1.].map(|t|evaluate_curve(curve,Parameter::Fraction(t),0,outputs[0].placement,&limits).unwrap()[0]);
        for point in points {
            assert!((point[axis]-p[0][axis]).abs()<1e-7,"wrong face: {point:?} vs {p:?}");}
        let varying=(0..3).find(|&a|(points[0][a]-points[2][a]).abs()>1e-7).expect("degenerate output");
        let mut low=f64::INFINITY;let mut high=f64::NEG_INFINITY;
        for triangle in display.triangles.iter().filter(|t|t[3] as usize==face) {for index in &triangle[..3] {
            let value=display.points[*index as usize][varying];low=low.min(value);high=high.max(value);
        }}
        let dimensions=[4.,6.,8.];assert!(low.abs()<1e-7 && (high-dimensions[varying]).abs()<1e-7);
        assert!(p[0][axis].abs()<1e-7 || (p[0][axis]-dimensions[axis]).abs()<1e-7);
        assert!((points[0][varying].min(points[2][varying])-low).abs()<1e-7);
        assert!((points[0][varying].max(points[2][varying])-high).abs()<1e-7);
        let after=c.document().clone();c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);c.undo().unwrap();
        c.clear_subselection();
    }
    c.select_objects(&[id]).unwrap();c.select_subobject(id,Subelement::BrepFace(keys[0])).unwrap();
    c.submit("ExtractIsocurve").unwrap();finish_face_verification(&mut c).unwrap();c.clear_subselection();c.select_subobject(id,Subelement::BrepFace(keys[1])).unwrap();
    assert!(c.submit("").is_err());assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),1);
    assert!(!c.isocurve_option_tokens().is_empty());c.submit("Cancel").unwrap();
}

#[test]
fn selected_brep_isocurve_refuses_mixed_face_display() {
    use om9_scene::scene::Subelement;
    let limits=Limits::default();let shape=om9_kernel::brep::box_brep([4.,6.,8.],0.1,&limits).unwrap();
    let mut display=shape.display().clone();let source_index=display.triangles.iter().position(|t|t[3]==0).unwrap();
    let target=display.triangles.iter().find(|t|t[3]!=0).unwrap()[3];let key=display.faces[target as usize];
    let mut misplaced=display.triangles.remove(source_index);misplaced[3]=target;display.triangles.push(misplaced);
    let shape=om9_core::brep::Brep::from_native(shape.source().to_owned(),display,&limits).unwrap();
    let entity=Entity::new("box",Geometry::Brep(shape),None);let id=entity.id;
    let mut doc=Document::with_template_tolerance("mixed face isocurve");doc.entities.insert(id,entity);
    let mut c=Controller::new(reopen_document_file(&doc,&limits),limits).unwrap();
    c.submit("Point").unwrap();c.submit("100,100,100").unwrap();c.undo().unwrap();c.select_objects(&[id]).unwrap();
    c.select_subobject(id,Subelement::BrepFace(key)).unwrap();let before=c.document().clone();
    c.submit("ExtractIsocurve").unwrap();assert!(finish_face_verification(&mut c).is_err());assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),1);
}

#[test]
fn selected_reordered_cylinder_face_isocurves_keep_affine_placement() {
    use om9_core::{evaluation::{evaluate_curve,Parameter},geometry::Transform};
    use om9_scene::scene::Subelement;
    let limits=Limits::default();let body=om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Cylinder {radius_mm:2.,height_mm:5.},0.1,&limits).unwrap();
    let side=(0..body.topology().faces.len()).find(|&i|{
        let face=om9_kernel::editing::extract_face(&body,i,0.1,&limits).unwrap();
        (om9_kernel::creation::measure(&face,&limits).unwrap().area_mm2-20.*std::f64::consts::PI).abs()<1e-6
    }).unwrap();
    let other=(0..body.topology().faces.len()).find(|i|*i!=side).unwrap();let mut display=body.display().clone();
    display.faces.swap(side,other);
    for item in &mut display.face_edges {if item[0] as usize==side {item[0]=other as u32;} else if item[0] as usize==other {item[0]=side as u32;}}
    for item in &mut display.triangles {if item[3] as usize==side {item[3]=other as u32;} else if item[3] as usize==other {item[3]=side as u32;}}
    let shape=om9_core::brep::Brep::from_native(body.source().to_owned(),display,&limits).unwrap();let face_key=shape.topology().faces[other];
    let placement=Transform([[2.,0.3,0.,3.],[0.,1.5,0.,-4.],[0.,0.,1.2,7.],[0.,0.,0.,1.]]);
    let inverse=om9_core::top11::inverse_affine(placement).unwrap();
    let mut entity=Entity::new("cylinder",Geometry::Brep(shape),None);entity.placement=placement;let id=entity.id;
    let mut doc=Document::with_template_tolerance("selected cylinder isocurve");doc.entities.insert(id,entity);
    let mut c=Controller::new(reopen_document_file(&doc,&limits),limits).unwrap();
    c.submit("Point").unwrap();c.submit("100,100,100").unwrap();c.undo().unwrap();let before=c.document().clone();
    for (option,circle,menu) in [("U=2.5",true,false),("V=0.5",false,true)] {
        c.select_objects(&[id]).unwrap();c.select_subobject(id,Subelement::BrepFace(face_key)).unwrap();
        if menu {c.dispatch("OM9-CURVE-019",om9_app::session::InvocationOrigin::Menu).unwrap();} else {c.submit("ExtractIsocurve").unwrap();}
        finish_face_verification(&mut c).unwrap();
        c.submit(option).unwrap();c.submit("").unwrap();
        let curves:Vec<_>=c.document().entities.values().filter(|e|e.label=="ExtractIsocurve").collect();assert_eq!(curves.len(),1);
        let Geometry::Curve(curve)=&curves[0].geometry else {panic!("curve")};let mut samples=Vec::new();
        for t in [0.,0.25,0.5,0.75,1.] {let world=evaluate_curve(curve,Parameter::Fraction(t),0,curves[0].placement,&limits).unwrap()[0];
            let p=inverse.point(world).unwrap();assert!((p[0].hypot(p[1])-2.).abs()<1e-6,"{p:?}");
            if circle {assert!((p[2]-2.5).abs()<1e-6,"{p:?}");}
            else {assert!((p[0]-2.*0.5_f64.cos()).abs()<1e-6 && (p[1]-2.*0.5_f64.sin()).abs()<1e-6,"{p:?}");}
            samples.push(p);
        }
        if circle {assert!(samples[0][0]*samples[2][0]+samples[0][1]*samples[2][1]< -3.9);}
        else {let ends=[samples[0][2],samples[4][2]];assert!(ends[0].min(ends[1]).abs()<1e-6 && (ends[0].max(ends[1])-5.).abs()<1e-6);}
        let after=c.document().clone();c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);c.undo().unwrap();c.clear_subselection();
    }
}

#[test]
fn selected_sphere_face_isocurve_matches_equator_and_reports_timing() {
    use om9_core::evaluation::{evaluate_curve,Parameter};use om9_scene::scene::Subelement;
    let limits=Limits::default();let shape=om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Sphere {radius_mm:3.},0.001,&limits).unwrap();
    let triangles=shape.display().triangles.len();let key=shape.topology().faces[0];
    let entity=Entity::new("sphere",Geometry::Brep(shape),None);let id=entity.id;
    let mut doc=Document::with_template_tolerance("sphere isocurve timing");doc.entities.insert(id,entity);
    let mut c=Controller::new(reopen_document_file(&doc,&limits),limits).unwrap();c.select_objects(&[id]).unwrap();
    c.select_subobject(id,Subelement::BrepFace(key)).unwrap();let before=c.document().clone();
    let start=std::time::Instant::now();c.submit("ExtractIsocurve").unwrap();let start_ms=start.elapsed().as_secs_f64()*1000.;
    assert!(c.isocurve_face_verification_pending());assert!(matches!(c.submit("U=0.5"),Err(om9_core::Error::Busy)));
    let deadline=std::time::Instant::now()+std::time::Duration::from_secs(30);
    let mut max_poll_ms=0_f64;
    loop {let poll=std::time::Instant::now();let result=c.poll_face_verification();max_poll_ms=max_poll_ms.max(poll.elapsed().as_secs_f64()*1000.);
        if let Some(result)=result {result.unwrap();break;}
        assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(2));}
    let commit=std::time::Instant::now();c.submit("").unwrap();let commit_ms=commit.elapsed().as_secs_f64()*1000.;
    eprintln!("sphere triangles={triangles} dispatch_ms={start_ms:.2} max_poll_ms={max_poll_ms:.2} commit_ms={commit_ms:.2}");
    let curves:Vec<_>=c.document().entities.values().filter(|e|e.label=="ExtractIsocurve").collect();assert_eq!(curves.len(),1);
    let Geometry::Curve(curve)=&curves[0].geometry else {panic!("curve")};
    for t in [0.,0.25,0.5,0.75,1.] {let p=evaluate_curve(curve,Parameter::Fraction(t),0,curves[0].placement,&limits).unwrap()[0];
        assert!(p[2].abs()<1e-6 && (p[0].hypot(p[1])-3.).abs()<1e-6,"{p:?}");}
    let after=c.document().clone();c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
}

#[test]
fn selected_face_isocurve_worker_handles_cancel_stale_and_pressure() {
    use om9_core::budget::memory::MemoryClass;use om9_scene::scene::Subelement;
    let limits=Limits::default();let shape=om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Sphere {radius_mm:3.},0.01,&limits).unwrap();
    assert!(shape.display().triangles.len()>512);let key=shape.topology().faces[0];
    let entity=Entity::new("sphere",Geometry::Brep(shape),None);let id=entity.id;
    let mut doc=Document::with_template_tolerance("isocurve worker");doc.entities.insert(id,entity);
    let doc=reopen_document_file(&doc,&limits);
    for mode in ["cancel","stale","pressure","subselection only"] {
        let mut c=Controller::new(doc.clone(),limits).unwrap();c.submit("Point").unwrap();c.submit("100,100,100").unwrap();c.undo().unwrap();
        if mode!="subselection only" {c.select_objects(&[id]).unwrap();}
        c.select_subobject(id,Subelement::BrepFace(key)).unwrap();let before=c.document().clone();
        c.submit("ExtractIsocurve").unwrap();assert!(c.isocurve_face_verification_pending());assert!(c.is_running());
        if mode=="cancel" {c.submit("Cancel").unwrap();assert!(!c.face_verification_pending());assert!(!c.is_running());}
        else {
            if mode=="stale" {c.clear_subselection();c.select_subobject(id,Subelement::BrepFace(key)).unwrap();}
            let pressure=if mode=="pressure" || mode=="stale" {Some(c.coordinator().reserve_remaining_external_memory_with_held(
                MemoryClass::Kernel,None,c.scene_cache_usage().unwrap()).unwrap())} else {None};
            let deadline=std::time::Instant::now()+std::time::Duration::from_secs(30);
            let result=loop {if let Some(result)=c.poll_face_verification() {break result;}
                assert!(std::time::Instant::now()<deadline);std::thread::sleep(std::time::Duration::from_millis(2));};
            if mode=="stale" {assert!(matches!(result,Err(om9_core::Error::Stale(_))),"{result:?}");}
            else if mode=="pressure" {
                match result {
                    Err(om9_core::Error::Budget(_)) => (),
                    Ok(()) => { assert!(!c.isocurve_option_tokens().is_empty()); c.cancel(); },
                    other => panic!("unexpected pressure result: {other:?}"),
                }
            }
            else {result.unwrap();assert!(!c.isocurve_option_tokens().is_empty());c.submit("").unwrap();
                c.undo().unwrap();assert_eq!(c.document(),&before);}
            drop(pressure);
        }
        assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),1);
    }
}

#[test]
fn brep_isocurve_clips_concave_face_and_two_holes() {
    use om9_core::{evaluation::{evaluate_curve,Parameter},rhino_semantics::TrimmedSurface};
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    let trim=TrimmedSurface {underlying:surface,
        outer:vec![[0.1,0.1],[0.9,0.1],[0.9,0.9],[0.7,0.9],[0.7,0.35],[0.6,0.35],[0.6,0.9],[0.1,0.9]],
        inners:vec![vec![[0.2,0.4],[0.3,0.4],[0.3,0.6],[0.2,0.6]],
                    vec![[0.4,0.4],[0.5,0.4],[0.5,0.6],[0.4,0.6]]],reversed:false};
    let source=om9_kernel::measurement::trimmed_surface_to_face_source_admitted(&trim,&limits,|_,_|Ok(())).unwrap();
    let brep=om9_kernel::brep::read_brep(&source,0.1,&limits).unwrap();
    let mut face=Entity::new("face",Geometry::Brep(brep),None);
    face.placement=om9_core::geometry::Transform::translation([3.,-4.,7.]);let id=face.id;
    let mut doc=Document::new("concave face with holes");doc.entities.insert(id,face);
    let mut c=Controller::new(reopen_document_file(&doc,&limits),limits).unwrap();
    c.submit("Point").unwrap();c.submit("100,100,100").unwrap();c.undo().unwrap();c.select_objects(&[id]).unwrap();
    let before=c.document().clone();c.submit("ExtractIsocurve").unwrap();c.submit("U=-0.01").unwrap();assert!(c.submit("").is_err());
    assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),1);assert!(!c.isocurve_option_tokens().is_empty());
    c.submit("U=0.5").unwrap();c.submit("").unwrap();
    let curves:Vec<_>=c.document().entities.values().filter(|e|e.label=="ExtractIsocurve").collect();assert_eq!(curves.len(),4);
    let expected=[[4.,5.],[6.,7.],[8.,9.],[10.,12.]];let mut found=[false;4];
    for e in curves {let Geometry::Curve(curve)=&e.geometry else {panic!("curve")};
        let ends=[0.,1.].map(|t|evaluate_curve(curve,Parameter::Fraction(t),0,e.placement,&limits).unwrap()[0]);
        for p in ends {assert!((p[1]-1.).abs()<1e-7 && (p[2]-7.).abs()<1e-7);}
        let pair=[ends[0][0].min(ends[1][0]),ends[0][0].max(ends[1][0])];
        let index=expected.iter().position(|q|(pair[0]-q[0]).abs()<1e-7 && (pair[1]-q[1]).abs()<1e-7).expect("unexpected clipped span");
        assert!(!found[index]);found[index]=true;
    }
    assert!(found.into_iter().all(|v|v));let after=c.document().clone();
    c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    c.undo().unwrap();c.select_objects(&[id]).unwrap();c.submit("ExtractIsocurve").unwrap();c.submit("U=0.4").unwrap();c.submit("").unwrap();
    let mut spans=Vec::new();
    for e in c.document().entities.values().filter(|e|e.label=="ExtractIsocurve") {
        let Geometry::Curve(curve)=&e.geometry else {panic!("curve")};
        let ends=[0.,1.].map(|t|evaluate_curve(curve,Parameter::Fraction(t),0,e.placement,&limits).unwrap()[0]);
        for p in ends {assert!((p[1]-0.).abs()<1e-7 && (p[2]-7.).abs()<1e-7);}
        spans.push([ends[0][0].min(ends[1][0]),ends[0][0].max(ends[1][0])]);
    }
    spans.sort_by(|a,b|a[0].total_cmp(&b[0]));let mut merged:Vec<[f64;2]>=Vec::new();
    for span in spans {if let Some(last)=merged.last_mut() {if span[0] <= last[1]+1e-7 {
        assert!(span[0]>=last[1]-1e-7,"overlapping isocurve outputs: {last:?} {span:?}");last[1]=last[1].max(span[1]);continue;
    }}merged.push(span);}
    assert_eq!(merged.len(),2,"{merged:?}");
    for (actual,expected) in merged.iter().zip([[4.,9.],[10.,12.]]) {
        assert!((actual[0]-expected[0]).abs()<1e-7 && (actual[1]-expected[1]).abs()<1e-7,"{merged:?}");
    }
    let after_boundary=c.document().clone();c.undo().unwrap();assert_eq!(c.document(),&before);
    c.redo().unwrap();assert_eq!(c.document(),&after_boundary);
    c.undo().unwrap();c.select_objects(&[id]).unwrap();c.submit("ExtractIsocurve").unwrap();c.submit("U=0.35").unwrap();c.submit("").unwrap();
    let mut spans=Vec::new();
    for e in c.document().entities.values().filter(|e|e.label=="ExtractIsocurve") {
        let Geometry::Curve(curve)=&e.geometry else {panic!("curve")};
        let ends=[0.,1.].map(|t|evaluate_curve(curve,Parameter::Fraction(t),0,e.placement,&limits).unwrap()[0]);
        for p in ends {assert!((p[1]+0.5).abs()<1e-7 && (p[2]-7.).abs()<1e-7);}
        spans.push([ends[0][0].min(ends[1][0]),ends[0][0].max(ends[1][0])]);
    }
    spans.sort_by(|a,b|a[0].total_cmp(&b[0]));assert!(!spans.is_empty());
    let mut right=4.;for span in spans {assert!((span[0]-right).abs()<1e-7,"gap or overlap at {right}: {span:?}");right=span[1];}
    assert!((right-12.).abs()<1e-7);
    let after_vertex=c.document().clone();c.undo().unwrap();assert_eq!(c.document(),&before);
    c.redo().unwrap();assert_eq!(c.document(),&after_vertex);
    c.undo().unwrap();c.select_objects(&[id]).unwrap();c.submit("ExtractIsocurve").unwrap();c.submit("U=0.1").unwrap();c.submit("").unwrap();
    let curves:Vec<_>=c.document().entities.values().filter(|e|e.label=="ExtractIsocurve").collect();assert_eq!(curves.len(),1);
    let Geometry::Curve(curve)=&curves[0].geometry else {panic!("curve")};
    let ends=[0.,1.].map(|t|evaluate_curve(curve,Parameter::Fraction(t),0,curves[0].placement,&limits).unwrap()[0]);
    let xs=[ends[0][0].min(ends[1][0]),ends[0][0].max(ends[1][0])];
    assert!((xs[0]-4.).abs()<1e-7 && (xs[1]-12.).abs()<1e-7);
    for p in ends {assert!((p[1]+3.).abs()<1e-7 && (p[2]-7.).abs()<1e-7);}
    let after_outer=c.document().clone();c.undo().unwrap();assert_eq!(c.document(),&before);
    c.redo().unwrap();assert_eq!(c.document(),&after_outer);
}

#[test]
fn larger_trim_source_isocurve_clips_hole_after_reopen() {
    use om9_core::{evaluation::{evaluate_curve,Parameter},rhino_semantics::{set_trimmed_surface,TrimmedSurface}};
    let limits=Limits::default();let n=28usize;
    let mut poles=Vec::new();for u in 0..n {for v in 0..n {poles.push([10.*u as f64/(n-1) as f64,10.*v as f64/(n-1) as f64,0.]);}}
    let mut knots=vec![0.];knots.extend((0..n).map(|i|i as f64/(n-1) as f64));knots.push(1.);
    let surface=om9_core::geometry::NurbsSurface {degree:[1,1],size:[n,n],poles,weights:vec![1.;n*n],knots:[knots.clone(),knots],domain:[[0.,1.],[0.,1.]],periodic:[false,false],reversed:false};
    let trim=TrimmedSurface {underlying:surface.clone(),outer:vec![[0.2,0.2],[0.8,0.2],[0.8,0.8],[0.2,0.8]],inners:vec![vec![[0.4,0.4],[0.6,0.4],[0.6,0.6],[0.4,0.6]]],reversed:false};
    let mut entity=Entity::new("large trim",Geometry::Surface(surface),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();let id=entity.id;
    let mut doc=Document::new("large trim isocurve");doc.entities.insert(id,entity);
    let reopened=reopen_document_file(&doc,&limits);
    let mut c=Controller::new(reopened,limits).unwrap();
    c.submit("Point").unwrap();c.submit("100,100,100").unwrap();c.undo().unwrap();c.select_objects(&[id]).unwrap();let before=c.document().clone();
    let class=om9_core::budget::memory::MemoryClass::Kernel;
    let baseline=c.coordinator().external_reserved_bytes(class);
    let pressure=c.coordinator().reserve_remaining_external_memory_with_held(class,None,c.scene_cache_usage().unwrap()).unwrap();
    assert!(c.submit("ExtractIsocurve").is_err());assert!(c.isocurve_option_tokens().is_empty());
    assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),1);drop(pressure);
    c.submit("ExtractIsocurve").unwrap();assert!(c.coordinator().external_reserved_bytes(class)>baseline);
    c.submit("Cancel").unwrap();assert_eq!(c.coordinator().external_reserved_bytes(class),baseline);
    c.submit("ExtractIsocurve").unwrap();c.submit("U=0.5").unwrap();c.submit("").unwrap();
    assert_eq!(c.coordinator().external_reserved_bytes(class),baseline);
    let outputs:Vec<_>=c.document().entities.values().filter(|e|e.id!=id).collect();assert_eq!(outputs.len(),2);
    let mut spans=outputs.iter().map(|e| {let Geometry::Curve(curve)=&e.geometry else {panic!("curve")};
        let p=[0.,1.].map(|t|evaluate_curve(curve,Parameter::Fraction(t),0,e.placement,&limits).unwrap()[0]);
        assert!((p[0][1]-5.).abs()<1e-7&&(p[1][1]-5.).abs()<1e-7);[p[0][0].min(p[1][0]),p[0][0].max(p[1][0])] }).collect::<Vec<_>>();
    spans.sort_by(|a,b|a[0].total_cmp(&b[0]));for (a,b) in spans.iter().zip([[2.,4.],[6.,8.]]) {assert!((a[0]-b[0]).abs()<1e-7&&(a[1]-b[1]).abs()<1e-7);}
    let after=c.document().clone();c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
}

#[test]
fn many_edge_trim_isocurve_clips_hole_after_reopen() {
    use om9_core::{budget::memory::MemoryClass,evaluation::{evaluate_curve,Parameter},geometry::Transform,
        rhino_semantics::{set_trimmed_surface,TrimmedSurface}};
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    let outer:Vec<_>=(0..1020).map(|i|{let a=std::f64::consts::TAU*(i as f64)/1020.;[0.5+0.4*a.cos(),0.5+0.4*a.sin()]}).collect();
    let trim=TrimmedSurface {underlying:surface.clone(),outer:outer.clone(),inners:vec![vec![[0.4,0.4],[0.6,0.4],[0.6,0.6],[0.4,0.6]]],reversed:false};
    let mut entity=Entity::new("many edge trim",Geometry::Surface(surface),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();
    let placement=Transform([[2.,0.5,0.,3.],[0.,1.,0.,-2.],[0.,0.,1.,7.],[0.,0.,0.,1.]]);entity.placement=placement;let id=entity.id;
    let mut doc=Document::new("many edge isocurve");doc.entities.insert(id,entity);
    let reopened=reopen_document_file(&doc,&limits);
    let y=0.53;let mut outer_hits=Vec::new();
    for i in 0..outer.len() {let a=outer[i];let b=outer[(i+1)%outer.len()];
        if (a[1]<y && b[1]>y)||(b[1]<y && a[1]>y) {outer_hits.push(a[0]+(y-a[1])/(b[1]-a[1])*(b[0]-a[0]));}}
    outer_hits.sort_by(f64::total_cmp);assert_eq!(outer_hits.len(),2);
    let expected=[[outer_hits[0],0.4],[0.6,outer_hits[1]]].map(|span|span.map(|x|placement.point([10.*x,5.3,0.]).unwrap()));
    for menu in [false,true] {
        let mut c=Controller::new(reopened.clone(),limits).unwrap();c.submit("Point").unwrap();c.submit("100,100,100").unwrap();c.undo().unwrap();
        c.select_objects(&[id]).unwrap();let before=c.document().clone();
        if menu {c.dispatch("OM9-CURVE-019",om9_app::session::InvocationOrigin::Menu).unwrap();}else{c.submit("ExtractIsocurve").unwrap();}
        c.submit("U=0.53").unwrap();let pressure=c.coordinator().reserve_remaining_external_memory_with_held(MemoryClass::Kernel,None,c.scene_cache_usage().unwrap()).unwrap();
        assert!(c.submit("").is_err());assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),1);drop(pressure);
        c.submit("").unwrap();let outputs:Vec<_>=c.document().entities.values().filter(|e|e.label=="ExtractIsocurve").collect();assert_eq!(outputs.len(),2);
        let mut seen=[false;2];
        for e in outputs {let Geometry::Curve(curve)=&e.geometry else{panic!("curve")};
            let points=[0.,1.].map(|t|evaluate_curve(curve,Parameter::Fraction(t),0,e.placement,&limits).unwrap()[0]);
            let near=|a:[f64;3],b:[f64;3]|(0..3).all(|i|(a[i]-b[i]).abs()<1e-7);
            let index=expected.iter().position(|pair|near(points[0],pair[0])&&near(points[1],pair[1]) || near(points[0],pair[1])&&near(points[1],pair[0])).unwrap();
            assert!(!seen[index]);seen[index]=true;
        }
        assert!(seen.into_iter().all(|value|value));let after=c.document().clone();c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }
}

#[test]
fn isocurve_refuses_1025_edge_trim_and_preserves_pending_and_redo() {
    use om9_core::rhino_semantics::{set_trimmed_surface,TrimmedSurface};
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    let outer=(0..1025).map(|i|{let a=std::f64::consts::TAU*i as f64/1025.;[0.5+0.4*a.cos(),0.5+0.4*a.sin()]}).collect();
    let trim=TrimmedSurface {underlying:surface.clone(),outer,inners:vec![],reversed:false};
    let mut entity=Entity::new("oversized trim",Geometry::Surface(surface),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();let id=entity.id;
    let mut doc=Document::with_template_tolerance("isocurve trim limit");doc.entities.insert(id,entity);
    let reopened=reopen_document_file(&doc,&limits);
    let mut c=Controller::new(reopened,limits).unwrap();c.submit("Point").unwrap();c.submit("100,100,100").unwrap();c.undo().unwrap();
    c.select_objects(&[id]).unwrap();let before=c.document().clone();
    c.submit("ExtractIsocurve").unwrap();c.submit("U=0.53").unwrap();
    assert!(matches!(c.submit(""),Err(om9_core::Error::Unsupported(_))));
    assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),1);assert!(!c.isocurve_option_tokens().is_empty());
    c.submit("Cancel").unwrap();assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),1);
}

#[test]
fn many_edge_cylinder_isocurve_clips_hole_after_reopen() {
    use om9_core::{evaluation::{evaluate_curve,Parameter},geometry::Transform,rhino_semantics::{set_trimmed_surface,TrimmedSurface},surface_kernel};
    let limits=Limits::default();
    let surface=surface_kernel::cylinder([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,5.,&limits).unwrap();
    let mut outer=Vec::new();
    for side in 0..4 {for i in 0..255 {let t=i as f64/255.;outer.push(match side {
        0=>[0.1*(1.-t)+0.8*t,0.2],1=>[0.8,0.2*(1.-t)+0.8*t],
        2=>[0.8*(1.-t)+0.1*t,0.8],_=>[0.1,0.8*(1.-t)+0.2*t],
    });}}
    let trim=TrimmedSurface {underlying:surface.clone(),outer,inners:vec![vec![[0.3,0.4],[0.6,0.4],[0.6,0.6],[0.3,0.6]]],reversed:false};
    let mut entity=Entity::new("many edge cylinder",Geometry::Surface(surface.clone()),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();
    let placement=Transform([[2.,0.5,0.,10.],[0.25,1.,0.,20.],[0.,0.,1.,30.],[0.,0.,0.,1.]]);entity.placement=placement;let id=entity.id;
    let mut doc=Document::with_template_tolerance("many edge cylinder isocurve");doc.entities.insert(id,entity);
    let reopened=reopen_document_file(&doc,&limits);
    let mut c=Controller::new(reopened,limits).unwrap();c.submit("Point").unwrap();c.submit("100,100,100").unwrap();c.undo().unwrap();
    c.select_objects(&[id]).unwrap();let before=c.document().clone();c.submit("ExtractIsocurve").unwrap();c.submit("U=0.53").unwrap();c.submit("").unwrap();
    let outputs:Vec<_>=c.document().entities.values().filter(|e|e.label=="ExtractIsocurve").collect();assert_eq!(outputs.len(),2);
    let intervals=[[0.1,0.3],[0.6,0.8]];let mut seen=[false;2];
    for output in outputs {let Geometry::Curve(curve)=&output.geometry else {panic!("curve")};
        let actual=[0.,0.3,1.].map(|t|evaluate_curve(curve,Parameter::Fraction(t),0,output.placement,&limits).unwrap()[0]);
        let i=(0..2).find(|&i|[0.,0.3,1.].iter().enumerate().all(|(j,&t)| {
            let u=intervals[i][0]+t*(intervals[i][1]-intervals[i][0]);
            let expected=placement.point(surface_kernel::evaluate(&surface,u,0.53,&limits).unwrap()).unwrap();
            (0..3).all(|k|(expected[k]-actual[j][k]).abs()<1e-7)
        })).expect("clipped source curve");assert!(!seen[i]);seen[i]=true;
    }
    assert_eq!(seen,[true;2]);let after=c.document().clone();c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
}

#[test]
fn certified_full_seam_cylinder_isocurves_after_reopen() {
    use om9_core::{evaluation::{evaluate_curve,Parameter},geometry::Transform,rhino_semantics::{set_trimmed_surface,TrimmedSurface}};
    let limits=Limits::default();let surface=om9_core::surface_kernel::cylinder([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,5.,&limits).unwrap();
    let trim=TrimmedSurface {underlying:surface.clone(),outer:vec![[0.,0.2],[1.,0.2],[1.,0.8],[0.,0.8]],
        inners:vec![vec![[0.3,0.4],[0.6,0.4],[0.6,0.6],[0.3,0.6]]],reversed:false};
    let mut entity=Entity::new("full cylinder",Geometry::Surface(surface.clone()),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();entity.placement=Transform::translation([10.,20.,30.]);let id=entity.id;
    let mut doc=Document::new("full seam isocurve");doc.entities.insert(id,entity);
    let path=std::env::temp_dir().join(format!("om9-full-seam-iso-{}.om9",uuid::Uuid::new_v4()));om9_archive::Archive::save(&path,&doc,&limits,&om9_core::transaction::Cancellation::default()).unwrap();
    for (option,expected) in [("U=0.3",1),("U=0.5",2),("V=0.5",2)] {for menu in [false,true] {
        let mut c=Controller::from_file(&path,limits).unwrap();c.submit("Point").unwrap();c.submit("1,2,3").unwrap();c.undo().unwrap();c.select_objects(&[id]).unwrap();let before=c.document().clone();
        if menu {c.dispatch("OM9-CURVE-019",om9_app::session::InvocationOrigin::Menu).unwrap();}else{c.submit("ExtractIsocurve").unwrap();}c.submit(option).unwrap();
        let pressure=c.coordinator().reserve_remaining_external_memory_with_held(om9_core::budget::memory::MemoryClass::Kernel,None,c.scene_cache_usage().unwrap()).unwrap();
        assert!(c.submit("").is_err());assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),1);drop(pressure);c.submit("").unwrap();
        let outputs:Vec<_>=c.document().entities.values().filter(|e|e.label=="ExtractIsocurve").collect();assert_eq!(outputs.len(),expected);
        let mut circle=None;let intervals: &[[f64;2]]=match option {"U=0.3"=>&[[0.,1.]],"U=0.5"=>&[[0.,0.3],[0.6,1.]],_=>&[[0.2,0.4],[0.6,0.8]]};let mut seen=[false;2];
        for e in outputs {let Geometry::Curve(curve)=&e.geometry else{panic!("curve")};let p=[0.,0.23,0.61,1.].map(|t|evaluate_curve(curve,Parameter::Fraction(t),0,e.placement,&limits).unwrap()[0]);
            for q in p {assert!(((q[0]-10.).hypot(q[1]-20.)-2.).abs()<1e-8);if option=="U=0.3" {assert!((q[2]-31.5).abs()<1e-8);}if option=="V=0.5" {assert!((q[0]-8.).abs()<1e-8&&(q[1]-20.).abs()<1e-8);}}
            let index=(0..intervals.len()).find(|i|[0.,0.23,0.61,1.].into_iter().enumerate().all(|(j,t)| {let v=intervals[*i][0]+t*(intervals[*i][1]-intervals[*i][0]);
                let uv=if option=="V=0.5" {[0.5,v]}else{[v,if option=="U=0.3" {0.3}else{0.5}]};
                let q=e.placement.point(om9_core::surface_kernel::evaluate(&surface,uv[0],uv[1],&limits).unwrap()).unwrap();(0..3).all(|k|(q[k]-p[j][k]).abs()<1e-8)})).unwrap();
            assert!(!seen[index]);seen[index]=true;
            if option=="U=0.3" {circle=Some(e.id);}
        }
        assert!(seen[..intervals.len()].iter().all(|v|*v));
        let after=c.document().clone();c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
        if let Some(circle)=circle {c.select_objects(&[circle]).unwrap();assert!((c.measure_selected_length().unwrap().value-4.*std::f64::consts::PI).abs()<1e-5);}
    }}
    std::fs::remove_file(path).unwrap();
}

#[test]
fn curved_isocurve_refuses_uncertain_world_scale_without_losing_retry() {
    use om9_core::{geometry::Transform,rhino_semantics::{set_trimmed_surface,TrimmedSurface}};
    let limits=Limits::default();let surface=om9_core::surface_kernel::cylinder([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,5.,&limits).unwrap();
    let trim=TrimmedSurface {underlying:surface.clone(),outer:vec![[0.1,0.2],[0.8,0.2],[0.8,0.8],[0.1,0.8]],inners:vec![],reversed:false};
    let mut entity=Entity::new("large placement",Geometry::Surface(surface),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();entity.placement=Transform::translation([1e12,0.,0.]);let id=entity.id;
    let mut doc=Document::new("large curved isocurve");doc.entities.insert(id,entity);let mut c=Controller::new(doc,limits).unwrap();
    c.submit("Point").unwrap();c.submit("1,2,3").unwrap();c.undo().unwrap();c.select_objects(&[id]).unwrap();let before=c.document().clone();
    c.submit("ExtractIsocurve").unwrap();c.submit("U=0.5").unwrap();
    assert!(matches!(c.submit(""),Err(om9_core::Error::Unsupported("isocurve placement is numerically uncertain"))));
    assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),1);assert!(!c.isocurve_option_tokens().is_empty());
}

#[test]
fn curved_isocurve_clips_diagonal_hole_and_refuses_full_closed_seam() {
    use om9_core::{evaluation::{evaluate_curve,Parameter},geometry::Transform,rhino_semantics::{set_trimmed_surface,TrimmedSurface}};
    let limits=Limits::default();let surface=om9_core::surface_kernel::cylinder([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,5.,&limits).unwrap();
    for seam in [false,true] {
        let outer=if seam {vec![[0.,0.2],[1.,0.2],[0.9,0.8],[0.,0.8]]} else {vec![[0.2,0.2],[0.8,0.2],[0.8,0.8],[0.2,0.8]]};
        let trim=TrimmedSurface {underlying:surface.clone(),outer,inners:vec![vec![[0.4,0.4],[0.6,0.4],[0.5,0.6]]],reversed:false};
        let mut entity=Entity::new("curved trim",Geometry::Surface(surface.clone()),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();entity.placement=Transform::translation([10.,20.,30.]);let id=entity.id;
        let mut doc=Document::new("curved isocurve");doc.entities.insert(id,entity);let mut c=Controller::new(doc,limits).unwrap();
        c.submit("Point").unwrap();c.submit("1,2,3").unwrap();c.undo().unwrap();c.select_objects(&[id]).unwrap();let before=c.document().clone();
        c.submit("ExtractIsocurve").unwrap();c.submit("U=0.5").unwrap();
        if seam {assert!(matches!(c.submit(""),Err(om9_core::Error::Unsupported("trim spans a closed surface seam"))));assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),1);assert!(!c.isocurve_option_tokens().is_empty());continue;}
        c.submit("").unwrap();let outputs:Vec<_>=c.document().entities.values().filter(|e|e.label=="ExtractIsocurve").collect();assert_eq!(outputs.len(),2);
        let intervals=[[0.2,0.45],[0.55,0.8]];let mut seen=[false;2];
        for output in outputs {let Geometry::Curve(curve)=&output.geometry else {panic!("curve")};
            let pts=[0.,0.3,1.].map(|t|evaluate_curve(curve,Parameter::Fraction(t),0,output.placement,&limits).unwrap()[0]);
            let i=(0..2).find(|i|[0.,0.3,1.].into_iter().enumerate().all(|(j,t)| {let u=intervals[*i][0]+t*(intervals[*i][1]-intervals[*i][0]);
                let p=output.placement.point(om9_core::surface_kernel::evaluate(&surface,u,0.5,&limits).unwrap()).unwrap();(0..3).all(|k|(p[k]-pts[j][k]).abs()<1e-8)})).unwrap();
            assert!(!seen[i]);seen[i]=true;
        }
        assert_eq!(seen,[true;2]);let after=c.document().clone();c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }
}

#[test]
fn trimmed_cylinder_isocurves_clip_to_outer_and_hole_after_reopen() {
    use om9_core::{evaluation::{evaluate_curve,Parameter},geometry::Transform,rhino_semantics::{set_trimmed_surface,TrimmedSurface}};
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::cylinder([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,5.,&limits).unwrap();
    let trim=TrimmedSurface {underlying:surface.clone(),outer:vec![[0.1,0.2],[0.8,0.2],[0.8,0.8],[0.1,0.8]],
        inners:vec![vec![[0.3,0.4],[0.6,0.4],[0.6,0.6],[0.3,0.6]]],reversed:false};
    for placement in [Transform::translation([10.,20.,30.]),Transform([[2.,0.5,0.,10.],[0.25,1.,0.,20.],[0.,0.,1.,30.],[0.,0.,0.,1.]])] {
        let mut entity=Entity::new("trim cylinder",Geometry::Surface(surface.clone()),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();entity.placement=placement;let id=entity.id;
        let mut doc=Document::new("trim cylinder isocurve");doc.entities.insert(id,entity);
        let path=std::env::temp_dir().join(format!("om9-trim-cylinder-iso-{}.om9",uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&path,&doc,&limits,&om9_core::transaction::Cancellation::default()).unwrap();
        for menu in [false,true] {for axis in ['U','V'] {
            let mut c=Controller::from_file(&path,limits).unwrap();c.submit("Point").unwrap();c.submit("1,2,3").unwrap();c.undo().unwrap();c.select_objects(&[id]).unwrap();let before=c.document().clone();
            if menu {c.dispatch("OM9-CURVE-019",om9_app::session::InvocationOrigin::Menu).unwrap();}else{c.submit("ExtractIsocurve").unwrap();}
            c.submit(&format!("{axis}=0.5")).unwrap();
            let pressure=c.coordinator().reserve_remaining_external_memory_with_held(om9_core::budget::memory::MemoryClass::Kernel,None,c.scene_cache_usage().unwrap()).unwrap();
            assert!(c.submit("").is_err());assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),1);assert!(!c.isocurve_option_tokens().is_empty());drop(pressure);c.submit("").unwrap();
            let outputs:Vec<_>=c.document().entities.values().filter(|e|e.label=="ExtractIsocurve").collect();assert_eq!(outputs.len(),2);
            let intervals=if axis=='U' {[[0.1,0.3],[0.6,0.8]]}else{[[0.2,0.4],[0.6,0.8]]};let mut seen=[false;2];let mut output_ids=[None;2];
            for output in outputs {
                let Geometry::Curve(curve)=&output.geometry else {panic!("curve")};
                let actual=[0.,0.23,0.61,1.].map(|t|evaluate_curve(curve,Parameter::Fraction(t),0,output.placement,&limits).unwrap()[0]);
                for p in actual {let (dx,dy)=(p[0]-10.,p[1]-20.);let (x,y)=if placement.0[0][0]==1. {(dx,dy)}else{let x=(dx-0.5*dy)/1.875;(x,dy-0.25*x)};
                    assert!((x.hypot(y)-2.).abs()<1e-8);if axis=='U' {assert!((p[2]-32.5).abs()<1e-8);}}
                let i=(0..2).find(|i|[0.,0.23,0.61,1.].into_iter().enumerate().all(|(j,t)| {
                    let v=intervals[*i][0]+t*(intervals[*i][1]-intervals[*i][0]);let uv=if axis=='U' {[v,0.5]}else{[0.5,v]};
                    let expected=placement.point(om9_core::surface_kernel::evaluate(&surface,uv[0],uv[1],&limits).unwrap()).unwrap();
                    (0..3).all(|k|(actual[j][k]-expected[k]).abs()<1e-8)
                })).expect("clipped UV isocurve");assert!(!seen[i]);seen[i]=true;output_ids[i]=Some(output.id);
            }
            assert_eq!(seen,[true;2]);let after=c.document().clone();c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
            let output_path=std::env::temp_dir().join(format!("om9-trim-cylinder-iso-output-{}.om9",uuid::Uuid::new_v4()));
            om9_archive::Archive::save(&output_path,&after,&limits,&om9_core::transaction::Cancellation::default()).unwrap();
            let mut reopened=Controller::from_file(&output_path,limits).unwrap();std::fs::remove_file(output_path).unwrap();assert_eq!(reopened.document().entities,after.entities);
            if axis=='U' && placement.0[0][0]==1. {for i in 0..2 {
                let a=om9_core::surface_kernel::evaluate(&surface,intervals[i][0],0.5,&limits).unwrap();let b=om9_core::surface_kernel::evaluate(&surface,intervals[i][1],0.5,&limits).unwrap();
                let angle=(b[1].atan2(b[0])-a[1].atan2(a[0])).rem_euclid(std::f64::consts::TAU);
                reopened.select_objects(&[output_ids[i].unwrap()]).unwrap();let measured=reopened.measure_selected_length().unwrap().value;
                assert!((measured-2.*angle).abs()<1e-5);
            }}
        }}
        std::fs::remove_file(path).unwrap();
    }
}

#[test]
fn surface_curve_commands_refuse_ambiguous_multiple_selection() {
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    let a=Entity::new("first surface",Geometry::Surface(surface.clone()),None);
    let mut b=Entity::new("second surface",Geometry::Surface(surface),None);
    b.placement=om9_core::geometry::Transform::translation([20.,0.,0.]);
    let point=Entity::new("point",Geometry::Point([50.,0.,0.]),None);
    let (a_id,b_id,p_id)=(a.id,b.id,point.id);
    let mut doc=Document::new("multiple selected");
    doc.entities.insert(a_id,a); doc.entities.insert(b_id,b); doc.entities.insert(p_id,point);
    let path=std::env::temp_dir().join(format!("om9-multiple-surface-{}.om9",uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path,&doc,&limits,&om9_core::transaction::Cancellation::default()).unwrap();
    for (command,feature) in [("DupBorder","OM9-CURVE-039"),("ExtractIsocurve","OM9-CURVE-019")] {
        let selections: &[[_;2]] = if command=="DupBorder" {&[[a_id,p_id]]} else {&[[a_id,b_id],[a_id,p_id]]};
        for &selection in selections { for menu in [false,true] {
            let mut c=Controller::from_file(&path,limits).unwrap();
            c.submit("Point").unwrap(); c.submit("1,2,3").unwrap(); c.undo().unwrap(); c.select_objects(&selection).unwrap();
            let before=c.document().clone();
            let result=if menu {c.dispatch(feature,om9_app::session::InvocationOrigin::Menu)} else {c.submit(command)};
            assert!(result.is_err(),"{command} must not choose an arbitrary selected object");
            assert_eq!(c.document(),&before); assert_eq!(c.coordinator().redo_len(),1);
            if command=="ExtractIsocurve" {assert!(c.isocurve_option_tokens().is_empty());}
            c.select_objects(&[a_id]).unwrap();
            if menu {c.dispatch(feature,om9_app::session::InvocationOrigin::Menu).unwrap();} else {c.submit(command).unwrap();}
            if command=="ExtractIsocurve" {c.submit("").unwrap();}
            let label=if command=="DupBorder" {"DupBorder"} else {"ExtractIsocurve"};
            assert_eq!(c.document().entities.values().filter(|e| e.label==label).count(),if command=="DupBorder" {4} else {1});
            let after=c.document().clone(); c.undo().unwrap(); assert_eq!(c.document(),&before); c.redo().unwrap(); assert_eq!(c.document(),&after);
        }}
    }
    std::fs::remove_file(path).unwrap();
}

#[test]
fn trimmed_plane_isocurve_splits_at_hole_after_reopen() {
    use om9_core::{evaluation::{evaluate_curve, Parameter}, geometry::{Curve, Transform}, rhino_semantics::{set_trimmed_surface, TrimmedSurface}};
    let limits = Limits::default();
    let surface = om9_core::surface_kernel::plane([0.,0.,0.], [1.,0.,0.], [0.,1.,0.], 10., 10., &limits).unwrap();
    let trim = TrimmedSurface { underlying: surface.clone(), outer: vec![[0.2,0.2],[0.8,0.2],[0.8,0.8],[0.2,0.8]],
        inners: vec![vec![[0.4,0.4],[0.6,0.4],[0.6,0.6],[0.4,0.6]]], reversed: false };
    let mut entity = Entity::new("trimmed", Geometry::Surface(surface), None);
    set_trimmed_surface(&mut entity, &trim, &limits).unwrap();
    entity.placement = Transform::translation([20.,30.,4.]); let id = entity.id;
    let mut doc = Document::new("trimmed isocurve"); doc.entities.insert(id, entity);
    let path = std::env::temp_dir().join(format!("om9-isocurve-trim-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, &doc, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    for menu in [false, true] { for axis in ["U", "V"] {
        let mut c = Controller::from_file(&path, limits).unwrap(); c.select_objects(&[id]).unwrap();
        let before = c.document().clone();
        if menu { c.dispatch("OM9-CURVE-019", om9_app::session::InvocationOrigin::Menu).unwrap(); }
        else { c.submit("ExtractIsocurve").unwrap(); }
        c.submit(&format!("{axis}=0.5")).unwrap(); c.submit("").unwrap();
        let outputs: Vec<_> = c.document().entities.values().filter(|e| e.label == "ExtractIsocurve").collect();
        assert_eq!(outputs.len(), 2);
        let mut ranges = Vec::new();
        for output in outputs {
            let Geometry::Curve(curve @ Curve::Line { .. }) = &output.geometry else { panic!("trimmed isocurve segment") };
            let points = [0., 1.].map(|t| evaluate_curve(curve, Parameter::Fraction(t), 0, output.placement, &limits).unwrap()[0]);
            let free = if axis == "U" { 0 } else { 1 };
            let fixed = 1 - free;
            assert!(points.iter().all(|p| (p[fixed] - if axis == "U" { 35. } else { 25. }).abs() < 1e-9 && (p[2]-4.).abs() < 1e-9));
            let base = if axis == "U" { 20. } else { 30. };
            ranges.push([(points[0][free]-base).min(points[1][free]-base), (points[0][free]-base).max(points[1][free]-base)]);
        }
        ranges.sort_by(|a,b| a[0].total_cmp(&b[0]));
        for (actual, expected) in ranges.iter().zip([[2.,4.],[6.,8.]]) { for i in 0..2 { assert!((actual[i]-expected[i]).abs() < 1e-9); } }
        let after = c.document().clone();
        let output_path = std::env::temp_dir().join(format!("om9-isocurve-trim-output-{}.om9", uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&output_path, &after, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
        let reopened = Controller::from_file(&output_path, limits).unwrap(); std::fs::remove_file(output_path).unwrap();
        assert_eq!(reopened.document().entities, after.entities);
        c.undo().unwrap(); assert_eq!(c.document(), &before); c.redo().unwrap(); assert_eq!(c.document(), &after);
    }}
    std::fs::remove_file(path).unwrap();
}

#[test]
fn trimmed_plane_isocurve_crosses_hole_vertices_once() {
    use om9_core::{evaluation::{evaluate_curve, Parameter}, geometry::{Curve, Transform}, rhino_semantics::{set_trimmed_surface, TrimmedSurface}};
    let limits = Limits::default();
    for reversed in [false, true] {
        let surface = om9_core::surface_kernel::plane([0.,0.,0.], [1.,0.,0.], [0.,1.,0.], 10., 10., &limits).unwrap();
        let mut hole = vec![[0.5,0.3],[0.7,0.5],[0.5,0.7],[0.3,0.5]];
        if reversed { hole.reverse(); }
        let trim = TrimmedSurface { underlying: surface.clone(), outer: vec![[0.2,0.2],[0.8,0.2],[0.8,0.8],[0.2,0.8]],
            inners: vec![hole], reversed: false };
        let mut entity = Entity::new("trimmed", Geometry::Surface(surface), None);
        set_trimmed_surface(&mut entity, &trim, &limits).unwrap(); let id = entity.id;
        let mut doc = Document::new("vertex crossing"); doc.entities.insert(id, entity);
        for axis in ["U", "V"] {
        let mut c = Controller::new(doc.clone(), limits).unwrap(); c.select_objects(&[id]).unwrap();
        let before = c.document().clone(); c.submit("ExtractIsocurve").unwrap(); c.submit(&format!("{axis}=0.5")).unwrap(); c.submit("").unwrap();
        let mut ranges = Vec::new();
        for output in c.document().entities.values().filter(|e| e.label == "ExtractIsocurve") {
            let Geometry::Curve(curve @ Curve::Line { .. }) = &output.geometry else { panic!("isocurve line") };
            let a = evaluate_curve(curve, Parameter::Fraction(0.), 0, Transform::identity(), &limits).unwrap()[0];
            let b = evaluate_curve(curve, Parameter::Fraction(1.), 0, Transform::identity(), &limits).unwrap()[0];
            let free = if axis == "U" { 0 } else { 1 }; let fixed = 1 - free;
            assert!((a[fixed]-5.).abs() < 1e-9 && (b[fixed]-5.).abs() < 1e-9);
            ranges.push([a[free].min(b[free]), a[free].max(b[free])]);
        }
        ranges.sort_by(|a,b| a[0].total_cmp(&b[0]));
        assert_eq!(ranges.len(), 2);
        for (actual, expected) in ranges.iter().zip([[2.,3.],[7.,8.]]) { for i in 0..2 { assert!((actual[i]-expected[i]).abs() < 1e-9); } }
        c.undo().unwrap(); assert_eq!(c.document(), &before); c.redo().unwrap();
        }
    }
}

#[test]
fn trimmed_plane_isocurve_tangent_or_near_vertex_keeps_session() {
    use om9_core::rhino_semantics::{set_trimmed_surface, TrimmedSurface};
    let limits = Limits::default();
    let surface = om9_core::surface_kernel::plane([0.,0.,0.], [1.,0.,0.], [0.,1.,0.], 10., 10., &limits).unwrap();
    let trim = TrimmedSurface { underlying: surface.clone(), outer: vec![[0.2,0.2],[0.8,0.2],[0.8,0.8],[0.2,0.8]],
        inners: vec![vec![[0.5,0.3],[0.7,0.5],[0.5,0.7],[0.3,0.5]]], reversed: false };
    let mut entity = Entity::new("trimmed", Geometry::Surface(surface), None);
    set_trimmed_surface(&mut entity, &trim, &limits).unwrap(); let id = entity.id;
    let mut doc = Document::new("tangent vertex"); doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, limits).unwrap(); c.submit("Point").unwrap(); c.submit("1,2,3").unwrap(); c.undo().unwrap();
    c.select_objects(&[id]).unwrap(); let before = c.document().clone(); c.submit("ExtractIsocurve").unwrap();
    for value in ["U=0.3", "U=0.5000000000005"] {
        c.submit(value).unwrap(); assert!(c.submit("").is_err());
        assert_eq!(c.document(), &before); assert_eq!(c.coordinator().redo_len(), 1);
        assert!(!c.isocurve_option_tokens().is_empty());
    }
    c.submit("U=0.5").unwrap(); c.submit("").unwrap();
    assert_eq!(c.document().entities.values().filter(|e| e.label == "ExtractIsocurve").count(), 2);
}

#[test]
fn trimmed_plane_isocurve_keeps_affine_placement() {
    use om9_core::{evaluation::{evaluate_curve, Parameter}, geometry::{Curve, Transform}, rhino_semantics::{set_trimmed_surface, TrimmedSurface}};
    let limits = Limits::default();
    let surface = om9_core::surface_kernel::plane([0.,0.,0.], [1.,0.,0.], [0.,1.,0.], 10., 10., &limits).unwrap();
    let trim = TrimmedSurface { underlying: surface.clone(), outer: vec![[0.2,0.2],[0.8,0.2],[0.8,0.8],[0.2,0.8]],
        inners: vec![], reversed: false };
    let mut entity = Entity::new("trimmed", Geometry::Surface(surface), None);
    set_trimmed_surface(&mut entity, &trim, &limits).unwrap();
    let placement = Transform([[2.,0.5,0.,20.],[0.25,1.,0.,30.],[0.,0.,1.,4.],[0.,0.,0.,1.]]);
    entity.placement = placement; let id = entity.id;
    let mut doc = Document::new("affine trimmed isocurve"); doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, limits).unwrap(); c.select_objects(&[id]).unwrap(); let before = c.document().clone();
    c.submit("ExtractIsocurve").unwrap(); c.submit("U=0.5").unwrap(); c.submit("").unwrap();
    let output = c.document().entities.values().find(|e| e.label == "ExtractIsocurve").unwrap();
    let Geometry::Curve(curve @ Curve::Line { .. }) = &output.geometry else { panic!("trimmed isocurve segment") };
    let ends = [0.,1.].map(|t| evaluate_curve(curve, Parameter::Fraction(t), 0, output.placement, &limits).unwrap()[0]);
    let expected = [[2.,5.,0.],[8.,5.,0.]].map(|p| placement.point(p).unwrap());
    let near = |a: [f64;3], b: [f64;3]| (0..3).all(|i| (a[i]-b[i]).abs() < 1e-9);
    assert!((near(ends[0], expected[0]) && near(ends[1], expected[1])) || (near(ends[0], expected[1]) && near(ends[1], expected[0])));
    c.undo().unwrap(); assert_eq!(c.document(), &before); c.redo().unwrap();
}

#[test]
fn trimmed_isocurve_refuses_uncertain_large_parameter_offset() {
    use om9_core::rhino_semantics::{set_trimmed_surface, TrimmedSurface};
    let limits = Limits::default();
    let mut surface = om9_core::surface_kernel::plane([0.,0.,0.], [1.,0.,0.], [0.,1.,0.], 10., 10., &limits).unwrap();
    surface.knots[0] = vec![1e8,1e8,1e8+10.,1e8+10.]; surface.domain[0] = [1e8,1e8+10.];
    let trim = TrimmedSurface { underlying: surface.clone(), outer: vec![[1e8+2.,0.2],[1e8+8.,0.2],[1e8+8.,0.8],[1e8+2.,0.8]],
        inners: vec![], reversed: false };
    let mut entity = Entity::new("large parameter offset", Geometry::Surface(surface), None);
    set_trimmed_surface(&mut entity, &trim, &limits).unwrap(); let id = entity.id;
    let mut doc = Document::new("isocurve parameter precision"); doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, limits).unwrap(); c.submit("Point").unwrap(); c.submit("1,2,3").unwrap(); c.undo().unwrap();
    c.select_objects(&[id]).unwrap(); let before = c.document().clone(); c.submit("ExtractIsocurve").unwrap(); c.submit("U=0.5").unwrap();
    assert!(c.submit("").is_err()); assert_eq!(c.document(), &before); assert_eq!(c.coordinator().redo_len(), 1);
    assert!(!c.isocurve_option_tokens().is_empty());
}

#[test]
fn trimmed_plane_isocurve_refuses_large_cancellation_before_publication() {
    use om9_core::{geometry::Transform, rhino_semantics::{set_trimmed_surface, TrimmedSurface}};
    let limits = Limits::default();
    let surface = om9_core::surface_kernel::plane([1e8,0.,0.], [1.,0.,0.], [0.,1.,0.], 10., 10., &limits).unwrap();
    let trim = TrimmedSurface { underlying: surface.clone(), outer: vec![[0.2,0.2],[0.8,0.2],[0.8,0.8],[0.2,0.8]],
        inners: vec![], reversed: false };
    let mut entity = Entity::new("large local coordinates", Geometry::Surface(surface), None);
    set_trimmed_surface(&mut entity, &trim, &limits).unwrap();
    entity.placement = Transform::translation([-1e8,0.,0.]); let id = entity.id;
    let mut doc = Document::new("isocurve cancellation"); doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, limits).unwrap(); c.select_objects(&[id]).unwrap();
    let before = c.document().clone(); c.submit("ExtractIsocurve").unwrap(); c.submit("U=0.5").unwrap();
    assert!(c.submit("").is_err()); assert_eq!(c.document(), &before);
    assert!(!c.isocurve_option_tokens().is_empty());
}

#[test]
fn trimmed_plane_isocurve_refuses_deleted_source_after_retry_failure() {
    use om9_core::rhino_semantics::{set_trimmed_surface, TrimmedSurface};
    let limits = Limits::default();
    let surface = om9_core::surface_kernel::plane([0.,0.,0.], [1.,0.,0.], [0.,1.,0.], 10., 10., &limits).unwrap();
    let trim = TrimmedSurface { underlying: surface.clone(), outer: vec![[0.2,0.2],[0.8,0.2],[0.8,0.8],[0.2,0.8]],
        inners: vec![], reversed: false };
    let mut entity = Entity::new("trimmed", Geometry::Surface(surface), None);
    set_trimmed_surface(&mut entity, &trim, &limits).unwrap(); let id = entity.id;
    let mut doc = Document::new("stale trimmed isocurve"); doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, limits).unwrap(); c.select_objects(&[id]).unwrap();
    c.submit("ExtractIsocurve").unwrap(); c.submit("U=0.2").unwrap(); assert!(c.submit("").is_err());
    c.delete_objects(&[id]).unwrap(); let before = c.coordinator().snapshot();
    c.submit("U=0.5").unwrap(); assert!(c.submit("").is_err());
    assert_eq!(c.coordinator().snapshot(), before);
    assert!(!c.isocurve_option_tokens().is_empty());
}

#[test]
fn trimmed_plane_nearly_parallel_crossing_is_refused_without_losing_redo() {
    use om9_core::rhino_semantics::{set_trimmed_surface, TrimmedSurface};
    let limits = Limits::default();
    let surface = om9_core::surface_kernel::plane([0.,0.,0.], [1.,0.,0.], [0.,1.,0.], 10., 10., &limits).unwrap();
    let trim = TrimmedSurface { underlying: surface.clone(), outer: vec![[0.2,0.49999999],[0.8,0.50000001],[0.8,0.8],[0.2,0.8]],
        inners: vec![], reversed: false };
    let mut entity = Entity::new("shallow crossing", Geometry::Surface(surface), None);
    set_trimmed_surface(&mut entity, &trim, &limits).unwrap(); let id = entity.id;
    let mut doc = Document::new("isocurve shallow crossing"); doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, limits).unwrap();
    c.submit("Point").unwrap(); c.submit("1,2,3").unwrap(); c.undo().unwrap(); c.select_objects(&[id]).unwrap();
    let before = c.document().clone(); c.submit("ExtractIsocurve").unwrap(); c.submit("U=0.5").unwrap();
    assert!(c.submit("").is_err());
    assert_eq!(c.document(), &before); assert_eq!(c.coordinator().redo_len(), 1);
    assert!(!c.isocurve_option_tokens().is_empty());
    c.submit("U=0.6").unwrap(); c.submit("").unwrap();
    assert_eq!(c.document().entities.values().filter(|e| e.label == "ExtractIsocurve").count(), 1);
    c.undo().unwrap(); assert_eq!(c.document(), &before);
}

#[test]
fn trimmed_plane_isocurve_refusal_keeps_session_and_redo_for_retry() {
    use om9_core::{budget::memory::MemoryClass, rhino_semantics::{set_trimmed_surface, TrimmedSurface}};
    let limits = Limits::default();
    let surface = om9_core::surface_kernel::plane([0.,0.,0.], [1.,0.,0.], [0.,1.,0.], 10., 10., &limits).unwrap();
    let trim = TrimmedSurface { underlying: surface.clone(), outer: vec![[0.2,0.2],[0.8,0.2],[0.8,0.8],[0.2,0.8]],
        inners: vec![vec![[0.4,0.4],[0.6,0.4],[0.6,0.6],[0.4,0.6]]], reversed: false };
    let mut entity = Entity::new("trimmed", Geometry::Surface(surface), None);
    set_trimmed_surface(&mut entity, &trim, &limits).unwrap(); let id = entity.id;
    let mut doc = Document::new("trimmed isocurve retry"); doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, limits).unwrap();
    c.submit("Point").unwrap(); c.submit("1,2,3").unwrap(); c.undo().unwrap(); c.select_objects(&[id]).unwrap();
    let before = c.document().clone(); c.submit("ExtractIsocurve").unwrap();
    for option in ["U=0.4", "U=0.1", "U=2"] {
        c.submit(option).unwrap(); assert!(c.submit("").is_err(), "{option}");
        assert_eq!(c.document(), &before); assert_eq!(c.coordinator().redo_len(), 1);
        assert!(!c.isocurve_option_tokens().is_empty());
    }
    c.submit("U=0.5").unwrap();
    let pressure = c.coordinator().reserve_remaining_external_memory_with_held(MemoryClass::Kernel, None, c.scene_cache_usage().unwrap()).unwrap();
    assert!(c.submit("").is_err()); assert_eq!(c.document(), &before); assert_eq!(c.coordinator().redo_len(), 1);
    assert!(!c.isocurve_option_tokens().is_empty()); drop(pressure);
    c.submit("").unwrap();
    assert_eq!(c.document().entities.values().filter(|e| e.label == "ExtractIsocurve").count(), 2);
    c.undo().unwrap(); assert_eq!(c.document(), &before); c.redo().unwrap();
}

fn controller_with_plane() -> (Controller, om9_core::identity::ObjectId) {
    let mut doc = Document::new("isocurve");
    let surface = om9_core::surface_kernel::plane(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        10.0,
        10.0,
        &Limits::default(),
    )
    .unwrap();
    let entity = Entity::new("plane", Geometry::Surface(surface), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    (Controller::new(doc, Limits::default()).unwrap(), id)
}

#[test]
fn extract_isocurve_creates_a_curve_from_a_surface() {
    let (mut c, id) = controller_with_plane();
    c.select_objects(&[id]).unwrap();
    c.submit("ExtractIsocurve").unwrap();
    c.submit("V=0.5").unwrap();
    c.submit("").unwrap();
    assert_eq!(c.document().entities.len(), 2);
    let curve = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "ExtractIsocurve")
        .unwrap();
    assert!(matches!(
        curve.geometry,
        Geometry::Curve(om9_core::geometry::Curve::Nurbs(_))
    ));
}

#[test]
fn isocurve_accepts_lowercase_options_and_preserves_history() {
    for (lower, upper) in [("u=0.25", "U=0.25"), ("v=0.75", "V=0.75")] {
        let (mut c, id) = controller_with_plane();
        c.select_objects(&[id]).unwrap();
        let before = c.document().clone();
        c.submit("ExtractIsocurve").unwrap();
        c.submit(lower).unwrap();
        c.submit("").unwrap();
        let lower_curve = c.document().entities.values().find(|e| e.label == "ExtractIsocurve").unwrap().geometry.clone();
        let after = c.document().clone();
        c.undo().unwrap();
        assert_eq!(c.document(), &before);
        c.redo().unwrap();
        assert_eq!(c.document(), &after);

        let (mut reference, id) = controller_with_plane();
        reference.select_objects(&[id]).unwrap();
        reference.submit("ExtractIsocurve").unwrap();
        reference.submit(upper).unwrap();
        reference.submit("").unwrap();
        let upper_curve = &reference.document().entities.values().find(|e| e.label == "ExtractIsocurve").unwrap().geometry;
        assert_eq!(&lower_curve, upper_curve);
    }
}

#[test]
fn isocurve_default_uses_middle_of_trimmed_parameter_domain_after_reopen() {
    use om9_core::{evaluation::{evaluate_curve, Parameter}, geometry::Transform};
    let limits = Limits::default();
    let mut surface = om9_core::surface_kernel::plane([0.,0.,0.], [1.,0.,0.], [0.,1.,0.], 10., 10., &limits).unwrap();
    surface.domain[1] = [0.2, 0.4];
    let entity = Entity::new("trimmed plane", Geometry::Surface(surface), None);
    let id = entity.id;
    let mut doc = Document::new("trimmed isocurve");
    doc.entities.insert(id, entity);
    let path = std::env::temp_dir().join(format!("om9-isocurve-domain-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, &doc, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    let mut c = Controller::from_file(&path, limits).unwrap();
    std::fs::remove_file(path).unwrap();
    c.select_objects(&[id]).unwrap();
    let before = c.document().clone();
    c.submit("ExtractIsocurve").unwrap();
    c.submit("").unwrap();
    let result = c.document().entities.values().find(|e| e.label == "ExtractIsocurve").unwrap();
    let Geometry::Curve(curve) = &result.geometry else { panic!("curve") };
    for t in [0., 0.5, 1.] {
        let p = evaluate_curve(curve, Parameter::Fraction(t), 0, Transform::identity(), &limits).unwrap()[0];
        assert!((p[0] - t * 10.).abs() < 1e-8 && (p[1] - 3.).abs() < 1e-8, "{p:?}");
    }
    let after = c.document().clone();
    c.undo().unwrap(); assert_eq!(c.document(), &before);
    c.redo().unwrap(); assert_eq!(c.document(), &after);
}


#[test]
fn isocurve_exposes_u_v_tokens() {
    let mut doc = Document::new("iso token");
    let surface = om9_core::surface_kernel::plane(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        10.0,
        10.0,
        &Limits::default(),
    )
    .unwrap();
    let entity = Entity::new("plane", Geometry::Surface(surface), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("ExtractIsocurve").unwrap();
    let tokens = c.isocurve_option_tokens();
    for expected in ["U=", "V="] {
        assert!(tokens.iter().any(|(_, command)| *command == expected));
    }
}

#[test]
fn reopened_placed_surface_isocurve_preserves_world_position_and_history() {
    use om9_core::{evaluation::{evaluate_curve, Parameter}, geometry::Transform};
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    for placement in [Transform::translation([10.,20.,30.]),
        Transform([[2.,0.5,0.,10.],[0.,1.,0.,20.],[0.,0.,1.,30.],[0.,0.,0.,1.]])] {
    let mut entity=Entity::new("plane",Geometry::Surface(surface.clone()),None);
    entity.placement=placement; let id=entity.id;
    let mut doc=Document::new("placed isocurve"); doc.entities.insert(id,entity);
    let path=std::env::temp_dir().join(format!("om9-isocurve-placement-{}.om9",uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path,&doc,&limits,&om9_core::transaction::Cancellation::default()).unwrap();
    let mut c=Controller::from_file(&path,limits).unwrap(); std::fs::remove_file(path).unwrap();
    c.select_objects(&[id]).unwrap(); let before=c.document().entities.clone();
    c.submit("ExtractIsocurve").unwrap(); c.submit("V=0.5").unwrap(); c.submit("").unwrap();
    let out=c.document().entities.values().find(|e|e.label=="ExtractIsocurve").unwrap();
    let Geometry::Curve(curve)=&out.geometry else {panic!("expected curve")};
    for t in [0.,0.25,0.5,0.75,1.] {
        let actual=evaluate_curve(curve,Parameter::Fraction(t),0,out.placement,&limits).unwrap()[0];
        let expected=placement.point(om9_core::surface_kernel::evaluate(&surface,0.5,t,&limits).unwrap()).unwrap();
        assert!((0..3).all(|i|(actual[i]-expected[i]).abs()<1e-8),"{t}: {actual:?} vs {expected:?}");
    }
    let after=c.document().entities.clone(); c.undo().unwrap(); assert_eq!(c.document().entities,before);
    c.redo().unwrap(); assert_eq!(c.document().entities,after);
    }
}

#[test]
fn reopened_cylinder_isocurves_keep_geometry_and_affine_placement() {
    use om9_core::{evaluation::{evaluate_curve, Parameter}, geometry::Transform};
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::cylinder([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,5.,&limits).unwrap();
    let placement=Transform([[2.,0.5,0.,10.],[0.,1.,0.,20.],[0.,0.,1.,30.],[0.,0.,0.,1.]]);
    for (option,circle) in [("U=0.4",true),("V=0.25",false)] {
        let mut entity=Entity::new("cylinder",Geometry::Surface(surface.clone()),None);
        entity.placement=placement; let id=entity.id;
        let mut doc=Document::new("cylinder isocurve"); doc.entities.insert(id,entity);
        let path=std::env::temp_dir().join(format!("om9-cylinder-isocurve-{}.om9",uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&path,&doc,&limits,&om9_core::transaction::Cancellation::default()).unwrap();
        let mut c=Controller::from_file(&path,limits).unwrap(); std::fs::remove_file(path).unwrap();
        c.select_objects(&[id]).unwrap(); let before=c.document().entities.clone();
        c.submit("ExtractIsocurve").unwrap(); c.submit(option).unwrap(); c.submit("").unwrap();
        let out=c.document().entities.values().find(|e|e.label=="ExtractIsocurve").unwrap();
        let Geometry::Curve(curve)=&out.geometry else {panic!("expected curve")};
        for t in [0.,0.125,0.25,0.5,0.75,1.] {
            let local=evaluate_curve(curve,Parameter::Fraction(t),0,Transform::identity(),&limits).unwrap()[0];
            if circle {assert!((local[0].hypot(local[1])-2.).abs()<1e-8 && (local[2]-2.).abs()<1e-8,"{local:?}");}
            else {assert!(local[0].abs()<1e-8 && (local[1]-2.).abs()<1e-8 && (local[2]-5.*t).abs()<1e-8,"{local:?}");}
            let actual=evaluate_curve(curve,Parameter::Fraction(t),0,out.placement,&limits).unwrap()[0];
            let expected=placement.point(local).unwrap();
            assert!((0..3).all(|i|(actual[i]-expected[i]).abs()<1e-8));
        }
        let after=c.document().entities.clone(); c.undo().unwrap(); assert_eq!(c.document().entities,before);
        c.redo().unwrap(); assert_eq!(c.document().entities,after);
    }
}

#[test]
fn isocurve_invalid_parameter_keeps_pending_for_correction_and_redo() {
    let (mut c,id)=controller_with_plane();
    c.submit("Point").unwrap(); c.submit("1,2,3").unwrap(); c.undo().unwrap();
    c.select_objects(&[id]).unwrap(); let before=c.coordinator().snapshot();
    c.submit("ExtractIsocurve").unwrap(); c.submit("V=2").unwrap();
    assert!(c.submit("").is_err());
    assert_eq!(c.coordinator().snapshot(),before); assert_eq!(c.coordinator().redo_len(),1);
    assert!(!c.isocurve_option_tokens().is_empty(),"failed parameter must remain correctable");
    c.submit("V=0.5").unwrap(); c.submit("").unwrap();
    assert_eq!(c.coordinator().redo_len(),0);
    let after=c.document().entities.clone(); c.undo().unwrap();
    assert_eq!(c.document().entities.len(),1); c.redo().unwrap(); assert_eq!(c.document().entities,after);
}

#[test]
fn isocurve_pressure_keeps_pending_and_allows_retry() {
    use om9_core::budget::memory::MemoryClass;
    let (mut c,id)=controller_with_plane();
    c.submit("Point").unwrap(); c.submit("1,2,3").unwrap(); c.undo().unwrap();
    c.select_objects(&[id]).unwrap(); let before=c.coordinator().snapshot();
    c.submit("ExtractIsocurve").unwrap(); c.submit("V=0.5").unwrap();
    let pressure=c.coordinator().reserve_remaining_external_memory_with_held(
        MemoryClass::Kernel,None,c.scene_cache_usage().unwrap()).unwrap();
    assert!(c.submit("").is_err());
    assert_eq!(c.coordinator().snapshot(),before); assert_eq!(c.coordinator().redo_len(),1);
    assert!(!c.isocurve_option_tokens().is_empty(),"failed commit must remain retryable");
    drop(pressure); c.submit("").unwrap();
    let after=c.document().entities.clone(); c.undo().unwrap();
    assert_eq!(c.document().entities.len(),1); c.redo().unwrap(); assert_eq!(c.document().entities,after);
}

#[test]
fn isocurve_failed_input_can_be_cancelled_without_losing_redo() {
    let (mut c,id)=controller_with_plane();
    c.submit("Point").unwrap(); c.submit("1,2,3").unwrap(); c.undo().unwrap();
    c.select_objects(&[id]).unwrap(); let before=c.coordinator().snapshot();
    c.submit("ExtractIsocurve").unwrap(); c.submit("V=2").unwrap();
    assert!(c.submit("").is_err()); c.submit("Cancel").unwrap();
    assert_eq!(c.coordinator().snapshot(),before); assert_eq!(c.coordinator().redo_len(),1);
    assert!(c.isocurve_option_tokens().is_empty());
}

#[test]
fn isocurve_retry_refuses_deleted_source_without_publishing() {
    let (mut c,id)=controller_with_plane(); c.select_objects(&[id]).unwrap();
    c.submit("ExtractIsocurve").unwrap(); c.submit("V=2").unwrap(); assert!(c.submit("").is_err());
    c.delete_objects(&[id]).unwrap(); let before=c.coordinator().snapshot();
    c.submit("V=0.5").unwrap(); let error=c.submit("").unwrap_err();
    assert!(error.to_string().contains("ExtractIsocurve source"),"{error}");
    assert_eq!(c.coordinator().snapshot(),before); assert!(!c.isocurve_option_tokens().is_empty());
    c.submit("Cancel").unwrap();
}

#[test]
fn isocurve_retry_refuses_source_moved_after_failure() {
    use om9_core::gumball::{Drag, Handle, Session};
    let (mut c,id)=controller_with_plane(); c.select_objects(&[id]).unwrap();
    c.submit("ExtractIsocurve").unwrap(); c.submit("V=2").unwrap(); assert!(c.submit("").is_err());
    let mut gesture=Session::begin(c.gumball_pivot().unwrap(),c.gumball_targets(),false).unwrap();
    gesture.drag(Handle::MoveFree,Drag::WorldDelta([5.,0.,0.])).unwrap();
    c.commit_gumball(&gesture,"Move").unwrap(); let before=c.coordinator().snapshot();
    c.submit("V=0.5").unwrap(); let error=c.submit("").unwrap_err();
    assert!(error.to_string().contains("ExtractIsocurve source"),"{error}");
    assert_eq!(c.coordinator().snapshot(),before); assert!(!c.isocurve_option_tokens().is_empty());
    c.submit("Cancel").unwrap();
}
