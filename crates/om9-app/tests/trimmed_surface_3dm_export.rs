// SPDX-License-Identifier: LGPL-2.1-or-later
#![cfg(all(windows,feature="occt-brep"))]
use om9_core::{budget::Limits,geometry::{Geometry,Transform},model::{Document,Entity},
    rhino_semantics::{set_trimmed_surface,TrimmedSurface},transaction::{Cancellation,TransactionCoordinator,World}};
use sha2::{Digest,Sha256};

#[test]
#[ignore = "requires native exchange executable beside the built om9 binary"]
fn actual_executable_roundtrips_reopened_trimmed_plane_with_hole() {
    let limits = Limits::default();
    let surface = om9_core::surface_kernel::plane([0.,0.,0.], [1.,0.,0.], [0.,1.,0.], 10., 10., &limits).unwrap();
    let outer=(0..1020).map(|i|{let a=std::f64::consts::TAU*i as f64/1020.;[0.5+0.3*a.cos(),0.5+0.3*a.sin()]}).collect();
    let trim = TrimmedSurface { underlying: surface.clone(),
        outer,
        inners: vec![vec![[0.4,0.4],[0.6,0.4],[0.6,0.6],[0.4,0.6]]], reversed: false };
    let mut entity = Entity::new("trim", Geometry::Surface(surface), None);
    set_trimmed_surface(&mut entity, &trim, &limits).unwrap();
    entity.placement = Transform::translation([3.,-4.,7.]);
    let mut doc = Document::with_template_tolerance("executable trim roundtrip");
    doc.entities.insert(entity.id, entity);
    let folder = std::env::temp_dir().join(format!("om9-trim-cli-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&folder).unwrap();
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup { fn drop(&mut self) { let _ = std::fs::remove_dir_all(&self.0); } }
    let _cleanup = Cleanup(folder.clone());
    let input = folder.join("input.om9");
    let exchange = folder.join("output.3dm");
    let output = folder.join("output.om9");
    om9_archive::Archive::save(&input, &doc, &limits, &Cancellation::default()).unwrap();
    let exe = env!("CARGO_BIN_EXE_om9");
    for (mode, from, to) in [("--export-3dm", &input, &exchange), ("--import-3dm", &exchange, &output)] {
        let result = std::process::Command::new(exe).arg(mode).arg(from).arg(to)
            .env_remove("OM9_EXCHANGE_NATIVE").env_remove("OM9_EXCHANGE_NATIVE_SHA256")
            .output().unwrap();
        assert!(result.status.success(), "{mode}: {}", String::from_utf8_lossy(&result.stderr));
    }
    assert_eq!(om9_archive::Archive::open(&input, &limits).unwrap(), doc);
    let reopened = om9_archive::Archive::open(&output, &limits).unwrap();
    assert_eq!(reopened.entities.len(), 1);
    let restored = reopened.entities.values().next().unwrap();
    assert!(matches!(restored.geometry, Geometry::Brep(_)));
    let ray = |p| om9_kernel::measurement::try_surface_ray_hit(&restored.geometry, restored.placement,
        p, [0.,0.,-1.], p, 1e-8, &limits).unwrap();
    assert!(ray([6.,-1.,20.]).is_some());
    assert!(ray([8.,1.,20.]).is_none());
    assert!(ray([12.,5.,20.]).is_none());
}

#[test]
#[ignore = "requires built openNURBS exchange executable"]
fn trimmed_plane_exports_3dm_and_reimports_with_hole() {
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    let outer=(0..1020).map(|i|{let a=std::f64::consts::TAU*i as f64/1020.;[0.5+0.3*a.cos(),0.5+0.3*a.sin()]}).collect();
    let trim=TrimmedSurface {underlying:surface.clone(),outer,
        inners:vec![vec![[0.4,0.4],[0.6,0.4],[0.6,0.6],[0.4,0.6]]],reversed:false};
    let mut entity=Entity::new("trim",Geometry::Surface(surface),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();
    entity.placement=Transform::translation([3.,-4.,7.]);let mut doc=Document::with_template_tolerance("trim 3dm");doc.entities.insert(entity.id,entity);
    let mut controller=om9_app::session::Controller::new(doc,limits).unwrap();
    controller.submit("Point").unwrap();controller.submit("100,100,100").unwrap();controller.undo().unwrap();
    let before=controller.document().clone();assert_eq!(controller.coordinator().redo_len(),1);
    let native=std::path::PathBuf::from(std::env::var_os("OM9_EXCHANGE_NATIVE").unwrap());
    let digest:[u8;32]=Sha256::digest(std::fs::read(&native).unwrap()).into();
    let host=if std::env::var_os("OM9_TEST_NATIVE_LAYOUT").is_some() {
        om9_app::three_dm_host::ThreeDmHost::from_directory(native.parent().unwrap()).unwrap()
    } else {om9_app::three_dm_host::ThreeDmHost::admit(&native,digest).unwrap()};
    assert_eq!(host.fingerprint(),digest);
    let path=std::env::temp_dir().join(format!("om9-trim-3dm-{}.3dm",uuid::Uuid::new_v4()));
    struct Remove(std::path::PathBuf);impl Drop for Remove {fn drop(&mut self){let _=std::fs::remove_file(&self.0);}}
    let _cleanup=Remove(path.clone());std::fs::write(&path,b"previous destination").unwrap();
    let pressure=controller.coordinator().reserve_remaining_external_memory_with_held(
        om9_core::budget::memory::MemoryClass::Kernel,None,controller.scene_cache_usage().unwrap()).unwrap();
    assert!(host.export(controller.coordinator(),controller.document().id,&path,&Cancellation::default()).is_err());
    assert_eq!(std::fs::read(&path).unwrap(),b"previous destination");assert_eq!(controller.document(),&before);
    assert_eq!(controller.coordinator().redo_len(),1);drop(pressure);
    let mut pressure=controller.coordinator().reserve_remaining_external_memory_with_held(
        om9_core::budget::memory::MemoryClass::Kernel,None,controller.scene_cache_usage().unwrap()).unwrap();
    let remaining=pressure.bytes().checked_sub(32*1024*1024).unwrap();pressure.shrink_to(remaining).unwrap();
    assert!(host.export(controller.coordinator(),controller.document().id,&path,&Cancellation::default()).is_err());
    assert_eq!(std::fs::read(&path).unwrap(),b"previous destination");assert_eq!(controller.document(),&before);
    assert_eq!(controller.coordinator().redo_len(),1);drop(pressure);
    host.export(controller.coordinator(),controller.document().id,&path,&Cancellation::default()).unwrap();
    assert_eq!(controller.document(),&before);assert_eq!(controller.coordinator().redo_len(),1);
    let imported=Document::with_template_tolerance("3dm import");let id=imported.id;
    let mut coordinator=TransactionCoordinator::new(World::new(vec![imported],&limits).unwrap(),limits).unwrap();
    host.import(&mut coordinator,id,&path,&Cancellation::default()).unwrap();
    let doc=&coordinator.world().documents[&id];assert_eq!(doc.entities.len(),1);let restored=doc.entities.values().next().unwrap();
    assert!(matches!(restored.geometry,Geometry::Brep(_)),"expected editable BRep");
    let ray=|p|om9_kernel::measurement::try_surface_ray_hit(&restored.geometry,restored.placement,p,[0.,0.,-1.],p,1e-8,&limits).unwrap();
    assert!(ray([6.,-1.,20.]).is_some());assert!(ray([8.,1.,20.]).is_none());assert!(ray([12.,5.,20.]).is_none());
    // Exercise downstream CAD after the 3DM exchange rather than relying on
    // the imported object's display topology or edge count as geometry proof.
    use om9_core::evaluation::{evaluate_curve,Parameter};
    let source_id=restored.id;
    let project=std::env::temp_dir().join(format!("om9-imported-trim-{}.om9",uuid::Uuid::new_v4()));let _project_cleanup=Remove(project.clone());
    om9_archive::Archive::save(&project,doc,&limits,&Cancellation::default()).unwrap();
    let mut c=om9_app::session::Controller::from_file(&project,limits).unwrap();
    c.submit("Point").unwrap();c.submit("100,100,100").unwrap();c.undo().unwrap();c.select_objects(&[source_id]).unwrap();
    let before=c.document().clone();
    let expected:Vec<_>=std::iter::once(&trim.outer).chain(&trim.inners).flat_map(|ring|(0..ring.len()).map(move |i|{
        [ring[i],ring[(i+1)%ring.len()]].map(|uv|[3.+10.*uv[0],-4.+10.*uv[1],7.])
    })).collect();
    for (command,label) in [("ExtractWireframe","ExtractWire"),("DupBorder","DupBorder")] {
    c.select_objects(&[source_id]).unwrap();
    let pressure=c.coordinator().reserve_remaining_external_memory_with_held(
        om9_core::budget::memory::MemoryClass::Kernel,None,c.scene_cache_usage().unwrap()).unwrap();
    assert!(c.submit(command).is_err());assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),1);drop(pressure);
    c.submit(command).unwrap();
    let outputs:Vec<_>=c.document().entities.values().filter(|e|e.label==label).collect();assert_eq!(outputs.len(),1024);
    let mut seen=vec![false;expected.len()];
    for output in outputs {let Geometry::Curve(curve)=&output.geometry else {panic!("expected edge curve")};
        let ends=[0.,1.].map(|t|evaluate_curve(curve,Parameter::Fraction(t),0,output.placement,&limits).unwrap()[0]);
        let near=|a:[f64;3],b:[f64;3]|(0..3).all(|i|(a[i]-b[i]).abs()<1e-6);
        let index=expected.iter().position(|pair|(near(ends[0],pair[0])&&near(ends[1],pair[1]))
            ||(near(ends[0],pair[1])&&near(ends[1],pair[0]))).expect("unexpected imported boundary");
        assert!(!seen[index]);seen[index]=true;
    }
    assert!(seen.into_iter().all(|v|v));let after=c.document().clone();c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);c.undo().unwrap();
    }
}

#[test]
#[ignore = "requires built openNURBS exchange executable"]
fn trimmed_cylinder_exports_3dm_and_reimports_with_hole() {
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::cylinder([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,5.,&limits).unwrap();
    let mut outer=Vec::new();for side in 0..4 {for i in 0..255 {let t=i as f64/255.;outer.push(match side {
        0=>[0.1*(1.-t)+0.3*t,0.2],1=>[0.3,0.2*(1.-t)+0.8*t],
        2=>[0.3*(1.-t)+0.1*t,0.8],_=>[0.1,0.8*(1.-t)+0.2*t],
    });}}
    let trim=TrimmedSurface {underlying:surface.clone(),outer,
        inners:vec![vec![[0.15,0.4],[0.2,0.4],[0.2,0.6],[0.15,0.6]]],reversed:false};
    let mut entity=Entity::new("trim",Geometry::Surface(surface.clone()),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();let selected=entity.id;
    let mut doc=Document::with_template_tolerance("trim cylinder 3dm");doc.entities.insert(selected,entity);
    let other=Entity::new("not selected",Geometry::Point([30.,0.,0.]),None);doc.entities.insert(other.id,other);
    let controller=om9_app::session::Controller::new(doc,limits).unwrap();let before=controller.document().clone();
    let native=std::path::PathBuf::from(std::env::var_os("OM9_EXCHANGE_NATIVE").unwrap());
    let digest:[u8;32]=Sha256::digest(std::fs::read(&native).unwrap()).into();let host=om9_app::three_dm_host::ThreeDmHost::admit(&native,digest).unwrap();
    let path=std::env::temp_dir().join(format!("om9-trim-cylinder-{}.3dm",uuid::Uuid::new_v4()));
    struct Remove(std::path::PathBuf);impl Drop for Remove {fn drop(&mut self){let _=std::fs::remove_file(&self.0);}}
    let _cleanup=Remove(path.clone());
    host.export_selected(controller.coordinator(),controller.document().id,&[selected],&path,&Cancellation::default()).unwrap();
    assert_eq!(controller.document(),&before);
    let imported=Document::with_template_tolerance("3dm import");let id=imported.id;
    let mut coordinator=TransactionCoordinator::new(World::new(vec![imported],&limits).unwrap(),limits).unwrap();
    host.import(&mut coordinator,id,&path,&Cancellation::default()).unwrap();
    let doc=&coordinator.world().documents[&id];assert_eq!(doc.entities.len(),1);let restored=doc.entities.values().next().unwrap();
    assert!(matches!(restored.geometry,Geometry::Brep(_)));
    let ray=|u| {let p=om9_core::surface_kernel::evaluate(&surface,u,0.5,&limits).unwrap();let n=[p[0]/2.,p[1]/2.,0.];
        let origin=[p[0]-0.5*n[0],p[1]-0.5*n[1],p[2]];
        om9_kernel::measurement::try_surface_ray_hit(&restored.geometry,restored.placement,origin,n,origin,1e-8,&limits).unwrap()};
    assert!(ray(0.12).is_some());assert!(ray(0.175).is_none());assert!(ray(0.35).is_none());
    use om9_core::evaluation::{evaluate_curve,Parameter};
    let source_id=restored.id;let project=std::env::temp_dir().join(format!("om9-cylinder-3dm-reopen-{}.om9",uuid::Uuid::new_v4()));
    let _project_cleanup=Remove(project.clone());om9_archive::Archive::save(&project,doc,&limits,&Cancellation::default()).unwrap();
    let mut c=om9_app::session::Controller::from_file(&project,limits).unwrap();
    c.select_objects(&[source_id]).unwrap();c.submit("Move").unwrap();
    c.accept_world_point([0.,0.,0.]).unwrap();c.accept_world_point([7.,-3.,2.]).unwrap();
    c.submit("Point").unwrap();c.submit("100,100,100").unwrap();c.undo().unwrap();c.select_objects(&[source_id]).unwrap();let before=c.document().clone();
    let placed=|uv:[f64;2]| {let p=om9_core::surface_kernel::evaluate(&surface,uv[0],uv[1],&limits).unwrap();[p[0]+7.,p[1]-3.,p[2]+2.]};
    for (option,uvs,menu) in [("U=0.5",[[0.1,0.5],[0.15,0.5],[0.2,0.5],[0.3,0.5]],false),
        ("V=0.175",[[0.175,0.2],[0.175,0.4],[0.175,0.6],[0.175,0.8]],true)] {
        c.select_objects(&[source_id]).unwrap();
        if !menu {
            let pressure=c.coordinator().reserve_remaining_external_memory_with_held(
                om9_core::budget::memory::MemoryClass::Kernel,None,c.scene_cache_usage().unwrap()).unwrap();
            assert!(c.submit("ExtractIsocurve").is_err());assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),1);
            drop(pressure);
        }
        if menu {c.dispatch("OM9-CURVE-019",om9_app::session::InvocationOrigin::Menu).unwrap();} else {c.submit("ExtractIsocurve").unwrap();}
        c.submit(option).unwrap();
        if !menu {
            let pressure=c.coordinator().reserve_remaining_external_memory_with_held(
                om9_core::budget::memory::MemoryClass::Kernel,None,c.scene_cache_usage().unwrap()).unwrap();
            assert!(c.submit("").is_err());assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),1);
            assert!(!c.isocurve_option_tokens().is_empty());drop(pressure);
        }
        c.submit("").unwrap();
        let iso:Vec<_>=c.document().entities.values().filter(|e|e.label=="ExtractIsocurve").collect();assert_eq!(iso.len(),2);
        let expected_iso=uvs.map(placed);let mut found=vec![false;4];
        for item in iso {let Geometry::Curve(curve)=&item.geometry else {panic!("isocurve")};
            let mut indices=[0;2];
            for (slot,t) in [0.,1.].into_iter().enumerate() {let p=evaluate_curve(curve,Parameter::Fraction(t),0,item.placement,&limits).unwrap()[0];
                let n=expected_iso.iter().position(|q|(0..3).all(|i|(p[i]-q[i]).abs()<1e-6)).expect("isocurve endpoint");
                assert!(!found[n]);found[n]=true;indices[slot]=n;}
            assert!((indices[0]/2)==(indices[1]/2));
            let uv=[(uvs[indices[0]][0]+uvs[indices[1]][0])*0.5,(uvs[indices[0]][1]+uvs[indices[1]][1])*0.5];
            let expected_mid=placed(uv);let mid=evaluate_curve(curve,Parameter::Fraction(0.5),0,item.placement,&limits).unwrap()[0];
            assert!((0..3).all(|i|(mid[i]-expected_mid[i]).abs()<1e-6),"midpoint {mid:?} expected {expected_mid:?}");}
        assert!(found.into_iter().all(|v|v));let iso_after=c.document().clone();
        c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&iso_after);c.undo().unwrap();
    }
    let mut expected=Vec::new();
    for ring in std::iter::once(&trim.outer).chain(&trim.inners) {for i in 0..ring.len() {
        expected.push([ring[i],ring[(i+1)%ring.len()]].map(placed));
    }}
    for (command,label) in [("ExtractWireframe","ExtractWire"),("DupBorder","DupBorder")] {
        c.select_objects(&[source_id]).unwrap();
        let pressure=c.coordinator().reserve_remaining_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::Kernel,None,c.scene_cache_usage().unwrap()).unwrap();
        assert!(c.submit(command).is_err());assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),1);
        drop(pressure);c.submit(command).unwrap();
        let outputs:Vec<_>=c.document().entities.values().filter(|e|e.label==label).collect();assert_eq!(outputs.len(),1024);
        let mut seen=vec![false;expected.len()];
        for output in outputs {let Geometry::Curve(curve)=&output.geometry else {panic!("edge curve")};
            let p=[0.,0.5,1.].map(|t|evaluate_curve(curve,Parameter::Fraction(t),0,output.placement,&limits).unwrap()[0]);
            let near=|a:[f64;3],b:[f64;3]|(0..3).all(|i|(a[i]-b[i]).abs()<1e-6);
            let index=expected.iter().position(|pair|(near(p[0],pair[0])&&near(p[2],pair[1]))
                ||(near(p[0],pair[1])&&near(p[2],pair[0]))).expect("unexpected cylindrical boundary");
            assert!(!seen[index]);seen[index]=true;
            assert!(((p[1][0]-7.).hypot(p[1][1]+3.)-2.).abs()<1e-6);
            assert!((p[1][2]-(p[0][2]+p[2][2])*0.5).abs()<1e-6);
        }
        assert!(seen.into_iter().all(|v|v));let after=c.document().clone();
        c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);c.undo().unwrap();
    }
}

#[test]
#[ignore = "requires built openNURBS exchange executable"]
fn full_seam_cylinder_exports_3dm_and_reimports_with_hole() {
    let limits=Limits::default();let surface=om9_core::surface_kernel::cylinder([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,5.,&limits).unwrap();
    let trim=TrimmedSurface {underlying:surface.clone(),outer:vec![[0.,0.2],[1.,0.2],[1.,0.8],[0.,0.8]],
        inners:vec![vec![[0.3,0.4],[0.4,0.4],[0.4,0.6],[0.3,0.6]]],reversed:false};
    let mut entity=Entity::new("trim",Geometry::Surface(surface.clone()),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();
    let mut doc=Document::with_template_tolerance("full seam 3dm");doc.entities.insert(entity.id,entity);
    let controller=om9_app::session::Controller::new(doc,limits).unwrap();let before=controller.document().clone();
    let native=std::path::PathBuf::from(std::env::var_os("OM9_EXCHANGE_NATIVE").unwrap());
    let digest:[u8;32]=Sha256::digest(std::fs::read(&native).unwrap()).into();let host=om9_app::three_dm_host::ThreeDmHost::admit(&native,digest).unwrap();
    let path=std::env::temp_dir().join(format!("om9-trim-seam-{}.3dm",uuid::Uuid::new_v4()));
    struct Remove(std::path::PathBuf);impl Drop for Remove {fn drop(&mut self){let _=std::fs::remove_file(&self.0);}}
    let _cleanup=Remove(path.clone());host.export(controller.coordinator(),controller.document().id,&path,&Cancellation::default()).unwrap();
    assert_eq!(controller.document(),&before);
    let imported=Document::with_template_tolerance("3dm seam import");let id=imported.id;
    let mut coordinator=TransactionCoordinator::new(World::new(vec![imported],&limits).unwrap(),limits).unwrap();
    host.import(&mut coordinator,id,&path,&Cancellation::default()).unwrap();
    let doc=&coordinator.world().documents[&id];assert_eq!(doc.entities.len(),1);let restored=doc.entities.values().next().unwrap();
    assert!(matches!(restored.geometry,Geometry::Brep(_)));
    let ray=|u,v| {let p=om9_core::surface_kernel::evaluate(&surface,u,v,&limits).unwrap();let n=[p[0]/2.,p[1]/2.,0.];
        let origin=[p[0]-0.5*n[0],p[1]-0.5*n[1],p[2]];
        om9_kernel::measurement::try_surface_ray_hit(&restored.geometry,restored.placement,origin,n,origin,1e-8,&limits).unwrap()};
    assert!(ray(0.2,0.5).is_some());assert!(ray(0.8,0.5).is_some());
    assert!(ray(0.35,0.5).is_none());assert!(ray(0.2,0.1).is_none());
}

#[test]
#[ignore = "requires built openNURBS exchange executable"]
fn trimmed_surface_block_exports_selected_and_reimports_with_hole() {
    use om9_core::model::Definition;
    let limits=Limits::default();let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    let trim=TrimmedSurface {underlying:surface.clone(),outer:vec![[0.2,0.2],[0.8,0.2],[0.8,0.8],[0.2,0.8]],
        inners:vec![vec![[0.4,0.4],[0.6,0.4],[0.6,0.6],[0.4,0.6]]],reversed:false};
    let mut member=Entity::new("trim",Geometry::Surface(surface),None);set_trimmed_surface(&mut member,&trim,&limits).unwrap();
    let definition=Definition::new("trim block",vec![member]);let mut root=Entity::new("instance",Geometry::Instance(definition.id),None);
    root.placement=Transform::translation([3.,-4.,7.]);let selected=root.id;
    let mut doc=Document::with_template_tolerance("block 3dm");doc.definitions.insert(definition.id,definition);doc.entities.insert(root.id,root);
    let controller=om9_app::session::Controller::new(doc,limits).unwrap();let before=controller.document().clone();
    let native=std::path::PathBuf::from(std::env::var_os("OM9_EXCHANGE_NATIVE").unwrap());
    let digest:[u8;32]=Sha256::digest(std::fs::read(&native).unwrap()).into();let host=om9_app::three_dm_host::ThreeDmHost::admit(&native,digest).unwrap();
    let path=std::env::temp_dir().join(format!("om9-trim-block-{}.3dm",uuid::Uuid::new_v4()));
    struct Remove(std::path::PathBuf);impl Drop for Remove {fn drop(&mut self){let _=std::fs::remove_file(&self.0);}}
    let _cleanup=Remove(path.clone());
    host.export_selected(controller.coordinator(),controller.document().id,&[selected],&path,&Cancellation::default()).unwrap();
    assert_eq!(controller.document(),&before);
    let imported=Document::with_template_tolerance("3dm block import");let id=imported.id;
    let mut coordinator=TransactionCoordinator::new(World::new(vec![imported],&limits).unwrap(),limits).unwrap();
    host.import(&mut coordinator,id,&path,&Cancellation::default()).unwrap();
    let doc=&coordinator.world().documents[&id];assert_eq!(doc.entities.len(),1);let root=doc.entities.values().next().unwrap();
    let Geometry::Instance(definition)=root.geometry else {panic!("expected instance")};
    let members=&doc.definitions[&definition].members;assert_eq!(members.len(),1);let restored=&members[0];
    assert!(matches!(restored.geometry,Geometry::Brep(_)));
    let placement=root.placement.compose(restored.placement).unwrap();
    let ray=|p|om9_kernel::measurement::try_surface_ray_hit(&restored.geometry,placement,p,[0.,0.,-1.],p,1e-8,&limits).unwrap();
    assert!(ray([6.,-1.,20.]).is_some());assert!(ray([8.,1.,20.]).is_none());assert!(ray([12.,5.,20.]).is_none());
    let root_id = root.id;
    let mut controller = om9_app::session::Controller::new(doc.as_ref().clone(), limits).unwrap();
    controller.select_objects(&[root_id]).unwrap(); let before = controller.document().clone();
    controller.submit("ExtractWireframe").unwrap();
    let curves: Vec<_> = controller.document().entities.values().filter(|e| e.label == "ExtractWire").collect();
    assert_eq!(curves.len(), 8);
    let loops = [vec![[5.,-2.,7.],[11.,-2.,7.],[11.,4.,7.],[5.,4.,7.]],
        vec![[7.,0.,7.],[9.,0.,7.],[9.,2.,7.],[7.,2.,7.]]];
    let expected: Vec<_> = loops.iter().flat_map(|ring| (0..ring.len()).map(move |i| [ring[i],ring[(i+1)%ring.len()]])).collect();
    let mut seen = [false;8];
    for e in curves {
        let Geometry::Curve(curve) = &e.geometry else { panic!("curve") };
        let points = [0.,1.].map(|t| om9_core::evaluation::evaluate_curve(curve,
            om9_core::evaluation::Parameter::Fraction(t),0,e.placement,&limits).unwrap()[0]);
        let near = |a:[f64;3],b:[f64;3]| (0..3).all(|i| (a[i]-b[i]).abs()<1e-6);
        let index = expected.iter().position(|pair| near(points[0],pair[0]) && near(points[1],pair[1])
            || near(points[0],pair[1]) && near(points[1],pair[0])).expect("unexpected edge");
        assert!(!seen[index]); seen[index] = true;
    }
    assert!(seen.into_iter().all(|value| value));
    let after = controller.document().clone(); controller.undo().unwrap(); assert_eq!(controller.document(),&before);
    controller.redo().unwrap(); assert_eq!(controller.document(),&after);
    let opaque = Document::with_template_tolerance("opaque 3dm block"); let opaque_id = opaque.id;
    let mut coordinator = TransactionCoordinator::new(World::new(vec![opaque],&limits).unwrap(),limits).unwrap();
    host.import_with_options(&mut coordinator,opaque_id,&path,&om9_exchange::ImportOptions {
        mode:om9_exchange::ImportMode::Geometry,brep:om9_exchange::BrepMode::RetainOpaque,..Default::default()
    },&Cancellation::default()).unwrap();
    let opaque = coordinator.world().documents[&opaque_id].as_ref().clone();
    let root = opaque.entities.values().find(|e| matches!(e.geometry,Geometry::Instance(_))).unwrap();
    let Geometry::Instance(definition) = root.geometry else { unreachable!() }; let root_id = root.id;
    assert!(matches!(opaque.definitions[&definition].members[0].geometry, Geometry::NativeBrep(_)));
    let mut controller = om9_app::session::Controller::new(opaque,limits).unwrap();
    controller.select_objects(&[root_id]).unwrap(); let before = controller.document().clone();
    controller.submit("ExtractWireframe").unwrap();
    let curves: Vec<_> = controller.document().entities.values().filter(|e| e.label == "ExtractWire").collect();
    assert_eq!(curves.len(),8); let mut seen = [false;8];
    for e in curves {
        let Geometry::Curve(curve) = &e.geometry else { panic!("curve") };
        let points = [0.,1.].map(|t| om9_core::evaluation::evaluate_curve(curve,
            om9_core::evaluation::Parameter::Fraction(t),0,e.placement,&limits).unwrap()[0]);
        let near = |a:[f64;3],b:[f64;3]| (0..3).all(|i| (a[i]-b[i]).abs()<1e-6);
        let index = expected.iter().position(|pair| near(points[0],pair[0]) && near(points[1],pair[1])
            || near(points[0],pair[1]) && near(points[1],pair[0])).expect("unexpected opaque edge");
        assert!(!seen[index]); seen[index] = true;
    }
    assert!(seen.into_iter().all(|value| value));
    let after = controller.document().clone(); controller.undo().unwrap(); assert_eq!(controller.document(),&before);
    controller.redo().unwrap(); assert_eq!(controller.document(),&after);
}

#[test]
#[ignore = "requires built openNURBS exchange executable"]
fn nested_3dm_block_extractwireframe_preserves_instances_and_undo() {
    use om9_core::{evaluation::{evaluate_curve,Parameter},model::Definition};
    use om9_exchange::{BrepMode,ImportMode,ImportOptions};
    let limits = Limits::default();
    let shape = om9_kernel::brep::box_brep([2.,3.,4.],0.1,&limits).unwrap();
    let mut leaf = Entity::new("box",Geometry::Brep(shape),None);
    leaf.placement = Transform::translation([1.,2.,0.]);
    let inner = Definition::new("inner box",vec![leaf]); let inner_id = inner.id;
    let mut nested = Entity::new("nested",Geometry::Instance(inner_id),None);
    let affine = Transform([[2.,0.5,0.,0.],[0.,1.,0.,3.],[0.,0.,1.,0.],[0.,0.,0.,1.]]);
    nested.placement = affine;
    let outer = Definition::new("outer block",vec![nested]); let outer_id = outer.id;
    let mut doc = Document::with_template_tolerance("authored nested 3dm");
    doc.definitions.insert(inner_id,inner); doc.definitions.insert(outer_id,outer);
    for x in [10.,40.] {
        let mut root = Entity::new("instance",Geometry::Instance(outer_id),None);
        root.placement = Transform::translation([x,0.,0.]); doc.entities.insert(root.id,root);
    }
    let controller = om9_app::session::Controller::new(doc,limits).unwrap();
    let native = std::path::PathBuf::from(std::env::var_os("OM9_EXCHANGE_NATIVE").unwrap());
    let digest:[u8;32] = Sha256::digest(std::fs::read(&native).unwrap()).into();
    let host = om9_app::three_dm_host::ThreeDmHost::admit(&native,digest).unwrap();
    let path = std::env::temp_dir().join(format!("om9-nested-wire-{}.3dm",uuid::Uuid::new_v4()));
    struct Remove(std::path::PathBuf); impl Drop for Remove { fn drop(&mut self) { let _=std::fs::remove_file(&self.0); } }
    let _cleanup = Remove(path.clone());
    host.export(controller.coordinator(),controller.document().id,&path,&Cancellation::default()).unwrap();
    let mut expected = Vec::new();
    for x in [10.,40.] {
        let world = Transform::translation([x,0.,0.]).compose(affine).unwrap().compose(Transform::translation([1.,2.,0.])).unwrap();
        let corners:Vec<_> = (0..8).map(|i|world.point([if i&1==0 {0.} else {2.},if i&2==0 {0.} else {3.},if i&4==0 {0.} else {4.}]).unwrap()).collect();
        for a in 0..8 { for b in a+1..8 { if ((a^b) as u32).count_ones()==1 { expected.push([corners[a],corners[b]]); } } }
    }
    assert_eq!(expected.len(),24);
    for brep in [BrepMode::EditableStrict,BrepMode::RetainOpaque] {
        let doc = Document::with_template_tolerance("import nested"); let id = doc.id;
        let mut coordinator = TransactionCoordinator::new(World::new(vec![doc],&limits).unwrap(),limits).unwrap();
        host.import_with_options(&mut coordinator,id,&path,&ImportOptions {mode:ImportMode::Geometry,brep,..Default::default()},&Cancellation::default()).unwrap();
        let imported = coordinator.world().documents[&id].as_ref().clone();
        let roots:Vec<_> = imported.entities.values().filter(|e|matches!(e.geometry,Geometry::Instance(_))).map(|e|e.id).collect();
        assert_eq!(roots.len(),2); assert_eq!(imported.definitions.len(),2);
        let leaves:Vec<_> = imported.definitions.values().flat_map(|d|d.members.iter())
            .filter(|e| !matches!(e.geometry,Geometry::Instance(_))).collect();
        assert_eq!(leaves.len(),1);
        assert!(if brep==BrepMode::RetainOpaque { matches!(leaves[0].geometry,Geometry::NativeBrep(_)) }
            else { matches!(leaves[0].geometry,Geometry::Brep(_)) });
        let archive = std::env::temp_dir().join(format!("om9-nested-wire-{}.om9",uuid::Uuid::new_v4()));
        let _archive_cleanup = Remove(archive.clone());
        om9_archive::Archive::save(&archive,&imported,&limits,&Cancellation::default()).unwrap();
        let mut c = om9_app::session::Controller::from_file(&archive,limits).unwrap(); c.select_objects(&roots).unwrap();
        let before = c.document().clone(); c.submit("ExtractWireframe").unwrap();
        let outputs:Vec<_> = c.document().entities.values().filter(|e|e.label=="ExtractWire").collect();
        assert_eq!(outputs.len(),24); let mut seen = vec![false;24];
        for e in outputs {
            let Geometry::Curve(curve) = &e.geometry else {panic!("curve")};
            let points = [0.,1.].map(|t|evaluate_curve(curve,Parameter::Fraction(t),0,e.placement,&limits).unwrap()[0]);
            let near=|a:[f64;3],b:[f64;3]|(0..3).all(|i|(a[i]-b[i]).abs()<1e-6);
            let index = expected.iter().position(|pair|near(points[0],pair[0])&&near(points[1],pair[1]) || near(points[0],pair[1])&&near(points[1],pair[0])).expect("unexpected edge");
            assert!(!seen[index]);seen[index]=true;
        }
        assert!(seen.into_iter().all(|value|value));let after=c.document().clone();
        om9_archive::Archive::save(&archive,&after,&limits,&Cancellation::default()).unwrap();
        assert_eq!(om9_app::session::Controller::from_file(&archive,limits).unwrap().document().entities,after.entities);
        c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }
}

#[test]
#[ignore = "requires built openNURBS exchange executable"]
fn late_unsupported_trim_keeps_destination_document_and_redo() {
    let limits=Limits::default();let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    let mut doc=Document::with_template_tolerance("late trim failure");
    for (index,reversed) in [(1,false),(2,true)] {
        let trim=TrimmedSurface {underlying:surface.clone(),outer:vec![[0.2,0.2],[0.8,0.2],[0.8,0.8],[0.2,0.8]],inners:vec![],reversed};
        let mut entity=Entity::new("trim",Geometry::Surface(surface.clone()),None);
        set_trimmed_surface(&mut entity,&trim,&limits).unwrap();entity.id=om9_core::identity::ObjectId(uuid::Uuid::from_u128(index));doc.entities.insert(entity.id,entity);
    }
    let mut controller=om9_app::session::Controller::new(doc,limits).unwrap();
    controller.submit("Point").unwrap();controller.submit("100,100,100").unwrap();controller.undo().unwrap();
    let before=controller.document().clone();assert_eq!(controller.coordinator().redo_len(),1);
    let native=std::path::PathBuf::from(std::env::var_os("OM9_EXCHANGE_NATIVE").unwrap());
    let digest:[u8;32]=Sha256::digest(std::fs::read(&native).unwrap()).into();let host=om9_app::three_dm_host::ThreeDmHost::admit(&native,digest).unwrap();
    let path=std::env::temp_dir().join(format!("om9-trim-fail-{}.3dm",uuid::Uuid::new_v4()));
    struct Remove(std::path::PathBuf);impl Drop for Remove {fn drop(&mut self){let _=std::fs::remove_file(&self.0);}}
    let _cleanup=Remove(path.clone());std::fs::write(&path,b"previous destination").unwrap();
    let canceled=Cancellation::default();canceled.cancel();
    assert!(matches!(host.export(controller.coordinator(),controller.document().id,&path,&canceled),Err(om9_core::Error::Canceled)));
    assert_eq!(std::fs::read(&path).unwrap(),b"previous destination");
    assert!(host.export(controller.coordinator(),controller.document().id,&path,&Cancellation::default()).is_err());
    assert_eq!(std::fs::read(&path).unwrap(),b"previous destination");assert_eq!(controller.document(),&before);
    assert_eq!(controller.coordinator().redo_len(),1);
}

#[test]
#[ignore = "requires built openNURBS exchange executable"]
fn mixed_trim_and_editable_brep_export_and_reimport() {
    let limits=Limits::default();let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    let trim=TrimmedSurface {underlying:surface.clone(),outer:vec![[0.2,0.2],[0.8,0.2],[0.8,0.8],[0.2,0.8]],inners:vec![vec![[0.4,0.4],[0.6,0.4],[0.6,0.6],[0.4,0.6]]],reversed:false};
    let mut entity=Entity::new("trim",Geometry::Surface(surface),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();
    let box_shape=om9_kernel::brep::box_brep([2.,2.,2.],0.1,&limits).unwrap();
    let mut block=Entity::new("solid",Geometry::Brep(box_shape),None);block.placement=Transform::translation([20.,0.,0.]);let mut doc=Document::with_template_tolerance("mixed export");
    doc.entities.insert(entity.id,entity);doc.entities.insert(block.id,block);
    let mut controller=om9_app::session::Controller::new(doc,limits).unwrap();controller.submit("Point").unwrap();controller.submit("100,100,100").unwrap();controller.undo().unwrap();
    let before=controller.document().clone();assert_eq!(controller.coordinator().redo_len(),1);
    let original=before.entities.values().find(|e|e.label=="solid").unwrap();
    let Geometry::Brep(original_brep)=&original.geometry else {panic!("expected solid source")};
    let expected_bounds=om9_kernel::brep::world_bounds(original_brep,&original.placement,&limits).unwrap();
    let native=std::path::PathBuf::from(std::env::var_os("OM9_EXCHANGE_NATIVE").unwrap());
    let digest:[u8;32]=Sha256::digest(std::fs::read(&native).unwrap()).into();let host=om9_app::three_dm_host::ThreeDmHost::admit(&native,digest).unwrap();
    let path=std::env::temp_dir().join(format!("om9-trim-mixed-{}.3dm",uuid::Uuid::new_v4()));
    struct Remove(std::path::PathBuf);impl Drop for Remove {fn drop(&mut self){let _=std::fs::remove_file(&self.0);}}
    let _cleanup=Remove(path.clone());std::fs::write(&path,b"previous destination").unwrap();
    host.export(controller.coordinator(),controller.document().id,&path,&Cancellation::default()).unwrap();
    assert_eq!(controller.document(),&before);assert_eq!(controller.coordinator().redo_len(),1);
    let imported=Document::with_template_tolerance("3dm mixed import");let id=imported.id;
    let mut coordinator=TransactionCoordinator::new(World::new(vec![imported],&limits).unwrap(),limits).unwrap();
    host.import(&mut coordinator,id,&path,&Cancellation::default()).unwrap();
    let doc=&coordinator.world().documents[&id];assert_eq!(doc.entities.len(),2);
    let trimmed=doc.entities.values().find(|e|e.label=="trim").unwrap();let solid=doc.entities.values().find(|e|e.label=="solid").unwrap();
    let ray=|entity:&Entity,p|om9_kernel::measurement::try_surface_ray_hit(&entity.geometry,entity.placement,p,[0.,0.,-1.],p,1e-8,&limits).unwrap();
    assert!(ray(trimmed,[3.,3.,10.]).is_some());assert!(ray(trimmed,[5.,5.,10.]).is_none());
    let Geometry::Brep(solid_brep)=&solid.geometry else {panic!("expected editable solid")};
    assert!((om9_kernel::brep::source_volume(solid_brep,&limits).unwrap()-8.).abs()<1e-6);
    let actual_bounds=om9_kernel::brep::world_bounds(solid_brep,&solid.placement,&limits).unwrap();
    for (actual,expected) in actual_bounds.iter().zip(expected_bounds) {assert!((actual-expected).abs()<1e-5);}
}

#[test]
#[ignore = "requires built openNURBS exchange executable"]
fn multiple_trimmed_surfaces_and_solid_export_and_reimport() {
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    let trim=TrimmedSurface {underlying:surface.clone(),outer:vec![[0.2,0.2],[0.8,0.2],[0.8,0.8],[0.2,0.8]],
        inners:vec![vec![[0.4,0.4],[0.6,0.4],[0.6,0.6],[0.4,0.6]]],reversed:false};
    let mut doc=Document::with_template_tolerance("multiple trim 3dm");
    for (id,label,offset) in [(1,"trim one",0.),(2,"trim two",20.)] {
        let mut entity=Entity::new(label,Geometry::Surface(surface.clone()),None);
        set_trimmed_surface(&mut entity,&trim,&limits).unwrap();
        entity.placement=Transform::translation([offset,0.,0.]);
        entity.id=om9_core::identity::ObjectId(uuid::Uuid::from_u128(id));doc.entities.insert(entity.id,entity);
    }
    let mut solid=Entity::new("solid",Geometry::Brep(om9_kernel::brep::box_brep([2.,2.,2.],0.1,&limits).unwrap()),None);
    solid.id=om9_core::identity::ObjectId(uuid::Uuid::from_u128(3));solid.placement=Transform::translation([40.,0.,0.]);
    doc.entities.insert(solid.id,solid);
    let mut controller=om9_app::session::Controller::new(doc,limits).unwrap();
    controller.submit("Point").unwrap();controller.submit("100,100,100").unwrap();controller.undo().unwrap();
    let before=controller.document().clone();assert_eq!(controller.coordinator().redo_len(),1);
    let original=before.entities.values().find(|e|e.label=="solid").unwrap();
    let Geometry::Brep(source)=&original.geometry else {panic!("expected solid source")};
    let expected_bounds=om9_kernel::brep::world_bounds(source,&original.placement,&limits).unwrap();
    let native=std::path::PathBuf::from(std::env::var_os("OM9_EXCHANGE_NATIVE").unwrap());
    let digest:[u8;32]=Sha256::digest(std::fs::read(&native).unwrap()).into();let host=om9_app::three_dm_host::ThreeDmHost::admit(&native,digest).unwrap();
    let path=std::env::temp_dir().join(format!("om9-trim-multiple-{}.3dm",uuid::Uuid::new_v4()));
    struct Remove(std::path::PathBuf);impl Drop for Remove {fn drop(&mut self){let _=std::fs::remove_file(&self.0);}}
    let _cleanup=Remove(path.clone());host.export(controller.coordinator(),controller.document().id,&path,&Cancellation::default()).unwrap();
    assert_eq!(controller.document(),&before);assert_eq!(controller.coordinator().redo_len(),1);
    let imported=Document::with_template_tolerance("3dm multiple import");let id=imported.id;
    let mut coordinator=TransactionCoordinator::new(World::new(vec![imported],&limits).unwrap(),limits).unwrap();
    host.import(&mut coordinator,id,&path,&Cancellation::default()).unwrap();
    let doc=&coordinator.world().documents[&id];assert_eq!(doc.entities.len(),3);
    for (label,offset) in [("trim one",0.),("trim two",20.)] {
        let entity=doc.entities.values().find(|e|e.label==label).unwrap();
        let ray=|p|om9_kernel::measurement::try_surface_ray_hit(&entity.geometry,entity.placement,p,[0.,0.,-1.],p,1e-8,&limits).unwrap();
        assert!(ray([offset+3.,3.,10.]).is_some());assert!(ray([offset+5.,5.,10.]).is_none());
        assert!(ray([offset+9.,9.,10.]).is_none());
    }
    let solid=doc.entities.values().find(|e|e.label=="solid").unwrap();
    let Geometry::Brep(source)=&solid.geometry else {panic!("expected solid")};
    assert!((om9_kernel::brep::source_volume(source,&limits).unwrap()-8.).abs()<1e-6);
    let bounds=om9_kernel::brep::world_bounds(source,&solid.placement,&limits).unwrap();
    for (actual,expected) in bounds.iter().zip(expected_bounds) {assert!((actual-expected).abs()<1e-5);}
}

#[test]
#[ignore = "requires built openNURBS exchange executable"]
fn larger_trim_source_exports_and_reimports_with_hole() {
    let limits=Limits::default();let n=28usize;
    let mut poles=Vec::new();for u in 0..n {for v in 0..n {poles.push([10.*u as f64/(n-1) as f64,10.*v as f64/(n-1) as f64,0.]);}}
    let mut knots=vec![0.];knots.extend((0..n).map(|i|i as f64/(n-1) as f64));knots.push(1.);
    let surface=om9_core::geometry::NurbsSurface {degree:[1,1],size:[n,n],poles,weights:vec![1.;n*n],knots:[knots.clone(),knots],domain:[[0.,1.],[0.,1.]],periodic:[false,false],reversed:false};
    let trim=TrimmedSurface {underlying:surface.clone(),outer:vec![[0.2,0.2],[0.8,0.2],[0.8,0.8],[0.2,0.8]],
        inners:vec![vec![[0.4,0.4],[0.6,0.4],[0.6,0.6],[0.4,0.6]]],reversed:false};
    let mut entity=Entity::new("large trim",Geometry::Surface(surface),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();
    assert!(entity.extensions[om9_core::model::TRIM_SURFACE_EXTENSION_SLOT].payload.len()>16*1024);
    let mut doc=Document::with_template_tolerance("large trim 3dm");doc.entities.insert(entity.id,entity);
    let mut controller=om9_app::session::Controller::new(doc,limits).unwrap();
    controller.submit("Point").unwrap();controller.submit("100,100,100").unwrap();controller.undo().unwrap();
    let before=controller.document().clone();assert_eq!(controller.coordinator().redo_len(),1);
    let native=std::path::PathBuf::from(std::env::var_os("OM9_EXCHANGE_NATIVE").unwrap());
    let digest:[u8;32]=Sha256::digest(std::fs::read(&native).unwrap()).into();let host=om9_app::three_dm_host::ThreeDmHost::admit(&native,digest).unwrap();
    let path=std::env::temp_dir().join(format!("om9-trim-large-{}.3dm",uuid::Uuid::new_v4()));
    struct Remove(std::path::PathBuf);impl Drop for Remove {fn drop(&mut self){let _=std::fs::remove_file(&self.0);}}
    let _cleanup=Remove(path.clone());host.export(controller.coordinator(),controller.document().id,&path,&Cancellation::default()).unwrap();
    assert_eq!(controller.document(),&before);assert_eq!(controller.coordinator().redo_len(),1);
    let imported=Document::with_template_tolerance("3dm large import");let id=imported.id;
    let mut coordinator=TransactionCoordinator::new(World::new(vec![imported],&limits).unwrap(),limits).unwrap();
    host.import(&mut coordinator,id,&path,&Cancellation::default()).unwrap();
    let doc=&coordinator.world().documents[&id];assert_eq!(doc.entities.len(),1);let entity=doc.entities.values().next().unwrap();
    let ray=|p|om9_kernel::measurement::try_surface_ray_hit(&entity.geometry,entity.placement,p,[0.,0.,-1.],p,1e-8,&limits).unwrap();
    assert!(ray([3.,3.,10.]).is_some());assert!(ray([5.,5.,10.]).is_none());assert!(ray([9.,9.,10.]).is_none());
}

