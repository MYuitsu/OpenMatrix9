#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};

#[test]
fn reopened_native_shell_border_uses_source_and_preserves_history() {
    use om9_core::{evaluation::{evaluate_curve,Parameter}, geometry::Transform};
    let limits=Limits::default();
    let circle=om9_core::curve_build::circle_on_frame([0.;3],[1.,0.,0.],[0.,1.,0.],2.).unwrap();
    let shell=om9_kernel::extrusion::shell(&circle,[0.,0.,5.],0.1,&limits).unwrap();
    let native=om9_kernel::brep::to_3dm(&shell,1e-6,&limits).unwrap();
    let mut entity=Entity::new("native shell",Geometry::NativeBrep(native),None);
    entity.placement=Transform::translation([30.,0.,0.]); let id=entity.id;
    let mut doc=Document::new("native shell");doc.entities.insert(id,entity);
    let path=std::env::temp_dir().join(format!("om9-native-border-{}.om9",uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path,&doc,&limits,&om9_core::transaction::Cancellation::default()).unwrap();
    for menu in [false,true] {
        let mut c=Controller::from_file(&path,limits).unwrap();
        c.submit("Point").unwrap();c.submit("1,2,3").unwrap();c.undo().unwrap();
        c.select_objects(&[id]).unwrap();let before=c.document().clone();
        let pressure=c.coordinator().reserve_remaining_external_memory_with_held(om9_core::budget::memory::MemoryClass::Kernel,None,c.scene_cache_usage().unwrap()).unwrap();
        assert!(c.submit("DupBorder").is_err());assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),1);drop(pressure);
        if menu {c.dispatch("OM9-CURVE-039",om9_app::session::InvocationOrigin::Menu).unwrap();} else {c.submit("DupBorder").unwrap();}
        let outputs:Vec<_>=c.document().entities.values().filter(|e|e.label=="DupBorder").collect();
        assert_eq!(outputs.len(),2);
        let mut heights=Vec::new();
        for e in outputs {
            let Geometry::Curve(curve)=&e.geometry else {panic!("curve expected")};
            for t in [0.,0.17,0.41,0.73] {
                let p=evaluate_curve(curve,Parameter::Fraction(t),0,e.placement,&limits).unwrap()[0];
                assert!(((p[0]-30.).hypot(p[1])-2.).abs()<1e-5,"{p:?}");
                assert!(p[2].abs()<1e-5||(p[2]-5.).abs()<1e-5,"{p:?}");
                heights.push(p[2]);
            }
        }
        assert!(heights.iter().any(|z|z.abs()<1e-5));assert!(heights.iter().any(|z|(*z-5.).abs()<1e-5));
        let after=c.document().clone();c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }
    std::fs::remove_file(path).unwrap();
}

#[test]
fn native_border_mixed_selection_and_invalid_source_are_atomic() {
    let limits=Limits::default();
    let circle=om9_core::curve_build::circle_on_frame([0.;3],[1.,0.,0.],[0.,1.,0.],2.).unwrap();
    let shell=om9_kernel::extrusion::shell(&circle,[0.,0.,5.],0.1,&limits).unwrap();
    let native=om9_kernel::brep::to_3dm(&shell,1e-6,&limits).unwrap();
    let mut bad=native.clone();bad.payload=vec![1,2,3,4].into();
    let plane=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    let a=Entity::new("plane",Geometry::Surface(plane),None);let b=Entity::new("native",Geometry::NativeBrep(native),None);
    let invalid=Entity::new("bad",Geometry::NativeBrep(bad),None);let (a_id,b_id,bad_id)=(a.id,b.id,invalid.id);
    let mut doc=Document::new("mixed");doc.entities.insert(a_id,a);doc.entities.insert(b_id,b);doc.entities.insert(bad_id,invalid);
    let mut c=Controller::new(doc,limits).unwrap();c.submit("Point").unwrap();c.submit("1,2,3").unwrap();c.undo().unwrap();
    c.select_objects(&[a_id,bad_id]).unwrap();let before=c.document().clone();
    assert!(c.submit("DupBorder").is_err());assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),1);
    c.select_objects(&[a_id,b_id]).unwrap();c.submit("DupBorder").unwrap();
    assert_eq!(c.document().entities.values().filter(|e|e.label=="DupBorder").count(),6);
    let after=c.document().clone();c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
}

#[test]
fn dupborder_brep_admission_pressure_reports_memory_for_retry() {
    use om9_core::budget::memory::MemoryClass;
    let limits=Limits {max_bytes:64*1024*1024,..Limits::default()};
    let circle=om9_core::curve_build::circle_on_frame([0.;3],[1.,0.,0.],[0.,1.,0.],2.).unwrap();
    let shell=om9_kernel::extrusion::shell(&circle,[0.,0.,5.],0.1,&limits).unwrap();
    let entity=Entity::new("shell",Geometry::Brep(shell),None);let id=entity.id;let mut doc=Document::new("brep pressure");doc.entities.insert(id,entity);
    let mut c=Controller::new(doc,limits).unwrap();c.submit("Point").unwrap();c.submit("1,2,3").unwrap();c.undo().unwrap();c.select_objects(&[id]).unwrap();
    let before=c.document().clone();let mut pressure=c.coordinator().reserve_remaining_external_memory_with_held(MemoryClass::Kernel,None,c.scene_cache_usage().unwrap()).unwrap();
    pressure.shrink_to(pressure.bytes()-1024).unwrap();let error=c.submit("DupBorder").unwrap_err();
    assert!(matches!(error,om9_core::Error::Budget("managed memory"|"managed RAM"|"available physical RAM"|"available system commit"|"external memory headroom"|"bytes")),"{error:?}");
    assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),1);drop(pressure);
    c.submit("DupBorder").unwrap();let after=c.document().clone();c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
}

#[test]
fn dupborder_multiple_brep_and_surface_boundaries_after_reopen() {
    use om9_core::{evaluation::{evaluate_curve,Parameter}, geometry::{Curve,Transform}};
    let limits=Limits::default();
    let plane=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    let circle=om9_core::curve_build::circle_on_frame([0.;3],[1.,0.,0.],[0.,1.,0.],2.).unwrap();
    for profile in [0.1,0.01] {
    let shell=om9_kernel::extrusion::shell(&circle,[0.,0.,5.],profile,&limits).unwrap();
    let a=Entity::new("plane",Geometry::Surface(plane.clone()),None);
    let mut b=Entity::new("shell",Geometry::Brep(shell.clone()),None); b.placement=Transform::translation([30.,0.,0.]);
    let mut d=Entity::new("shell two",Geometry::Brep(shell),None);
    d.placement=Transform([[2.,0.5,0.,50.],[0.,1.,0.,0.],[0.,0.,1.,0.],[0.,0.,0.,1.]]);
    let (a_id,b_id,d_id)=(a.id,b.id,d.id);let mut doc=Document::new("mixed borders");doc.entities.insert(a_id,a);doc.entities.insert(b_id,b);doc.entities.insert(d_id,d);
    let path=std::env::temp_dir().join(format!("om9-mixed-border-{}.om9",uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path,&doc,&limits,&om9_core::transaction::Cancellation::default()).unwrap();
    for ids in [[a_id,b_id],[b_id,d_id]] {for menu in [false,true] {
        let mut c=Controller::from_file(&path,limits).unwrap();c.submit("Point").unwrap();c.submit("1,2,3").unwrap();c.undo().unwrap();c.select_objects(&ids).unwrap();let before=c.document().clone();
        let pressure=c.coordinator().reserve_remaining_external_memory_with_held(om9_core::budget::memory::MemoryClass::Kernel,None,c.scene_cache_usage().unwrap()).unwrap();
        assert!(c.submit("DupBorder").is_err());assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),1);drop(pressure);
        if menu {c.dispatch("OM9-CURVE-039",om9_app::session::InvocationOrigin::Menu).unwrap();} else {c.submit("DupBorder").unwrap();}
        let outputs:Vec<_>=c.document().entities.values().filter(|e|e.label=="DupBorder").collect();
        assert_eq!(outputs.len(),if ids[0]==a_id {6}else{4});let mut counts=[0usize;3];let mut heights=[[false;2];2];let mut base_circle=None;
        for e in outputs {
            let Geometry::Curve(curve)=&e.geometry else {panic!("border")};
            let points=[0.,0.17,0.41,0.73,1.].map(|t|evaluate_curve(curve,Parameter::Fraction(t),0,e.placement,&limits).unwrap()[0]);
            let center=e.placement.0[0][3];
            if center==0. {
                counts[0]+=1;let [a,b]=[points[0],points[4]];
                for p in [a,b] {assert!(p[0].abs()<1e-8||(p[0]-10.).abs()<1e-8);assert!(p[1].abs()<1e-8||(p[1]-10.).abs()<1e-8);}
                assert_ne!((a[0]-b[0]).abs()<1e-8,(a[1]-b[1]).abs()<1e-8);
                assert!(matches!(curve,Curve::Nurbs(_)));
            } else {
                let i=if center==30. {1} else {assert_eq!(center,50.);2};counts[i]+=1;
                let z=if points[0][2].abs()<1e-7 {0} else {assert!((points[0][2]-5.).abs()<1e-7);1};
                assert!(!heights[i-1][z]);heights[i-1][z]=true;
                if i==1 && z==0 {base_circle=Some(e.id);}
                for p in points {let x=if i==1 {p[0]-center}else{(p[0]-center-0.5*p[1])/2.};
                    assert!((x.hypot(p[1])-2.).abs()<1e-7);assert!((p[2]-5.*z as f64).abs()<1e-7);}
            }
        }
        assert_eq!(counts,if ids[0]==a_id {[4,2,0]}else{[0,2,2]});
        let after=c.document().clone();c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
        let output_path=std::env::temp_dir().join(format!("om9-mixed-border-output-{}.om9",uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&output_path,&after,&limits,&om9_core::transaction::Cancellation::default()).unwrap();
        let mut reopened=Controller::from_file(&output_path,limits).unwrap();std::fs::remove_file(output_path).unwrap();
        assert_eq!(reopened.document().entities,after.entities);reopened.select_objects(&[base_circle.unwrap()]).unwrap();
        let length=reopened.measure_selected_length().unwrap().value;
        assert!((length-4.*std::f64::consts::PI).abs()<1e-5,"{length}");
        let before_extrude=reopened.document().clone();reopened.submit("ExtrudeCrv 5").unwrap();
        let (body,placement)=reopened.document().entities.values().find_map(|e| match &e.geometry {
            Geometry::Brep(body) if !before_extrude.entities.contains_key(&e.id)=>Some((body,e.placement)),_=>None,
        }).expect("extrusion result");
        let measure=om9_kernel::creation::measure(body,&limits).unwrap();assert_eq!(measure.solids,1);assert!(measure.closed);
        assert!((measure.volume_mm3-20.*std::f64::consts::PI).abs()<1e-5);
        let bounds=om9_kernel::brep::world_bounds(body,&placement,&limits).unwrap();
        for (actual,expected) in bounds.iter().zip([28.,-2.,0.,32.,2.,5.]) {assert!((actual-expected).abs()<1e-5,"{bounds:?}");}
        let after_extrude=reopened.document().clone();reopened.undo().unwrap();assert_eq!(reopened.document(),&before_extrude);
        reopened.redo().unwrap();assert_eq!(reopened.document(),&after_extrude);
    }}
    let limited=Limits {max_outputs:5,..limits};let mut c=Controller::from_file(&path,limited).unwrap();
    c.submit("Point").unwrap();c.submit("1,2,3").unwrap();c.undo().unwrap();c.select_objects(&[a_id,b_id]).unwrap();let before=c.document().clone();
    assert!(c.submit("DupBorder").is_err());assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),1);
    std::fs::remove_file(path).unwrap();
    }
}

#[test]
fn dupborder_mixed_closed_brep_refuses_without_partial_output() {
    let limits=Limits::default();let plane=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    let shape=om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Box {x_mm:4.,y_mm:5.,z_mm:6.},0.1,&limits).unwrap();
    let mut a=Entity::new("plane",Geometry::Surface(plane),None);let mut b=Entity::new("closed box",Geometry::Brep(shape),None);
    if a.id>b.id {std::mem::swap(&mut a.id,&mut b.id)}let (a_id,b_id)=(a.id,b.id);
    let mut doc=Document::new("mixed closed");doc.entities.insert(a_id,a);doc.entities.insert(b_id,b);
    let path=std::env::temp_dir().join(format!("om9-closed-border-{}.om9",uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path,&doc,&limits,&om9_core::transaction::Cancellation::default()).unwrap();
    let mut c=Controller::from_file(&path,limits).unwrap();std::fs::remove_file(path).unwrap();
    c.submit("Point").unwrap();c.submit("1,2,3").unwrap();c.undo().unwrap();c.select_objects(&[a_id,b_id]).unwrap();let before=c.document().clone();
    assert!(c.submit("DupBorder").is_err());assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),1);
}

#[test]
fn dupborder_multiple_surfaces_are_atomic_after_reopen() {
    use om9_core::{evaluation::{evaluate_curve,Parameter}, geometry::{Curve,Transform}, rhino_semantics::{set_trimmed_surface,TrimmedSurface}};
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    let mut a=Entity::new("trimmed",Geometry::Surface(surface.clone()),None);
    let trim=TrimmedSurface {underlying:surface.clone(),outer:vec![[0.2,0.2],[0.8,0.2],[0.8,0.8],[0.2,0.8]],inners:vec![],reversed:false};
    set_trimmed_surface(&mut a,&trim,&limits).unwrap(); a.placement=Transform::translation([10.,0.,0.]);
    let mut b=Entity::new("plane",Geometry::Surface(surface),None); b.placement=Transform::translation([30.,0.,0.]);
    let point=Entity::new("point",Geometry::Point([60.,0.,0.]),None); let (a_id,b_id,p_id)=(a.id,b.id,point.id);
    let mut doc=Document::new("two borders"); doc.entities.insert(a_id,a);doc.entities.insert(b_id,b);doc.entities.insert(p_id,point);
    let path=std::env::temp_dir().join(format!("om9-multiborder-{}.om9",uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path,&doc,&limits,&om9_core::transaction::Cancellation::default()).unwrap();
    for menu in [false,true] {
        let mut c=Controller::from_file(&path,limits).unwrap();
        c.submit("Point").unwrap();c.submit("1,2,3").unwrap();c.undo().unwrap();c.select_objects(&[a_id,b_id]).unwrap();
        let before=c.document().clone();
        let pressure=c.coordinator().reserve_remaining_external_memory_with_held(om9_core::budget::memory::MemoryClass::Kernel,None,c.scene_cache_usage().unwrap()).unwrap();
        assert!(c.submit("DupBorder").is_err());assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),1);drop(pressure);
        if menu {c.dispatch("OM9-CURVE-039",om9_app::session::InvocationOrigin::Menu).unwrap();} else {c.submit("DupBorder").unwrap();}
        let outputs:Vec<_>=c.document().entities.values().filter(|e|e.label=="DupBorder").collect();assert_eq!(outputs.len(),8);
        let mut edges=std::collections::BTreeSet::new();
        for e in outputs {
            let Geometry::Curve(curve)=&e.geometry else {panic!("curve")};
            let ends=[0.,1.].map(|t|evaluate_curve(curve,Parameter::Fraction(t),0,e.placement,&limits).unwrap()[0]);
            let (x0,x1,y0,y1)=if e.placement.0[0][3]==10. {(12.,18.,2.,8.)} else {(30.,40.,0.,10.)};
            for p in ends {assert!((p[0]-x0).abs()<1e-8||(p[0]-x1).abs()<1e-8);assert!((p[1]-y0).abs()<1e-8||(p[1]-y1).abs()<1e-8);}
            assert_ne!((ends[0][0]-ends[1][0]).abs()<1e-8,(ends[0][1]-ends[1][1]).abs()<1e-8);
            let xy=|p:[f64;3]|[(p[0]*100.).round() as i32,(p[1]*100.).round() as i32];let (x,y)=(xy(ends[0]),xy(ends[1]));
            assert!(edges.insert(if x<y {(x,y)}else{(y,x)}));
            assert!(matches!(curve,Curve::Line {..}|Curve::Nurbs(_)));
        }
        let after=c.document().clone();c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
        let output_path=std::env::temp_dir().join(format!("om9-multisurface-border-output-{}.om9",uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&output_path,&after,&limits,&om9_core::transaction::Cancellation::default()).unwrap();
        let mut reopened=Controller::from_file(&output_path,limits).unwrap();std::fs::remove_file(output_path).unwrap();
        assert_eq!(reopened.document().entities,after.entities);
        for (offset,length) in [(10.,6.),(30.,10.)] {
            let id=reopened.document().entities.values().find(|e|e.label=="DupBorder" && e.placement.0[0][3]==offset).unwrap().id;
            reopened.select_objects(&[id]).unwrap();assert!((reopened.measure_selected_length().unwrap().value-length).abs()<1e-6);
        }
    }
    let limited=Limits {max_outputs:7,..limits};
    let mut c=Controller::from_file(&path,limited).unwrap(); c.submit("Point").unwrap();c.submit("1,2,3").unwrap();c.undo().unwrap();c.select_objects(&[a_id,b_id]).unwrap();
    let before=c.document().clone();assert!(c.submit("DupBorder").is_err());assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),1);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn dupborder_multiple_surfaces_rejects_late_unsupported_trim_atomically() {
    use om9_core::rhino_semantics::{set_trimmed_surface,TrimmedSurface};
    let limits=Limits::default();
    let plane=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    let cylinder=om9_core::surface_kernel::cylinder([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,5.,&limits).unwrap();
    let mut a=Entity::new("plane",Geometry::Surface(plane),None);
    let mut b=Entity::new("trimmed cylinder",Geometry::Surface(cylinder.clone()),None);
    let trim=TrimmedSurface {underlying:cylinder,outer:vec![[0.2,0.2],[0.8,0.2],[0.75,0.8],[0.2,0.8]],inners:vec![],reversed:false};
    set_trimmed_surface(&mut b,&trim,&limits).unwrap();
    if a.id>b.id {std::mem::swap(&mut a.id,&mut b.id)}
    let (a_id,b_id)=(a.id,b.id);let mut doc=Document::new("late unsupported");doc.entities.insert(a_id,a);doc.entities.insert(b_id,b);
    let mut c=Controller::new(doc,limits).unwrap();c.submit("Point").unwrap();c.submit("1,2,3").unwrap();c.undo().unwrap();c.select_objects(&[a_id,b_id]).unwrap();
    let before=c.document().clone();assert!(c.submit("DupBorder").is_err());assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),1);
}

#[test]
fn conical_surface_border_omits_only_the_exact_apex() {
    use om9_core::{evaluation::{evaluate_curve, Parameter}, geometry::Transform};
    let limits = Limits::default();
    for collapsed in [true, false] {
        let mut surface = om9_core::surface_kernel::cylinder([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,5.,&limits).unwrap();
        for u in 0..surface.size[0] {
            let p = &mut surface.poles[u * surface.size[1] + 1];
            if collapsed { *p = [0.,0.,5.]; } else { p[0] *= 1e-5; p[1] *= 1e-5; }
        }
        let mut entity = Entity::new("cone",Geometry::Surface(surface.clone()),None);
        entity.placement = Transform::translation([10.,20.,30.]); let id = entity.id;
        let mut doc = Document::new("cone border"); doc.entities.insert(id,entity);
        let path = std::env::temp_dir().join(format!("om9-cone-border-{}.om9",uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&path,&doc,&limits,&om9_core::transaction::Cancellation::default()).unwrap();
        for menu in [false,true] {
            let mut c = Controller::from_file(&path,limits).unwrap();
            c.select_objects(&[id]).unwrap(); let before = c.document().clone();
            if menu { c.dispatch("OM9-CURVE-039",om9_app::session::InvocationOrigin::Menu).unwrap(); }
            else { c.submit("DupBorder").unwrap(); }
            let outputs: Vec<_> = c.document().entities.values().filter(|e| e.label == "DupBorder").collect();
            assert_eq!(outputs.len(),if collapsed {1} else {2});
            let mut seen = [false;2];
            for output in outputs {
                let Geometry::Curve(curve) = &output.geometry else { panic!("border curve") };
                let point = |t| evaluate_curve(curve,Parameter::Fraction(t),0,output.placement,&limits).unwrap()[0];
                let side = if (point(0.)[2]-30.).abs() < 1e-8 {0} else {1};
                assert!(!seen[side]); seen[side] = true;
                for t in [0.,0.13,0.37,0.61,0.91,1.] {
                    let expected = output.placement.point(om9_core::surface_kernel::evaluate(&surface,t,side as f64,&limits).unwrap()).unwrap();
                    let actual = point(t); assert!((0..3).all(|i| (actual[i]-expected[i]).abs() < 1e-8));
                }
            }
            assert_eq!(seen,[true,!collapsed]); let after = c.document().clone();
            c.undo().unwrap(); assert_eq!(c.document(),&before); c.redo().unwrap(); assert_eq!(c.document(),&after);
        }
        std::fs::remove_file(path).unwrap();
    }
}

#[test]
fn conical_surface_border_output_limit_preserves_redo() {
    let limits = Limits {max_outputs: 1,..Limits::default()};
    let mut surface = om9_core::surface_kernel::cylinder([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,5.,&limits).unwrap();
    for u in 0..surface.size[0] { let p=&mut surface.poles[u*surface.size[1]+1]; p[0]*=1e-5; p[1]*=1e-5; }
    let entity = Entity::new("nonzero cone",Geometry::Surface(surface),None); let id = entity.id;
    let mut doc = Document::new("limit"); doc.entities.insert(id,entity);
    let mut c = Controller::new(doc,limits).unwrap();
    c.submit("Point").unwrap(); c.submit("1,2,3").unwrap(); c.undo().unwrap(); c.select_objects(&[id]).unwrap();
    let before = c.document().clone();
    assert!(c.submit("DupBorder").is_err()); assert_eq!(c.document(),&before); assert_eq!(c.coordinator().redo_len(),1);
}

#[test]
fn reopened_multiface_shell_duplicates_only_open_boundaries() {
    use om9_core::{evaluation::{evaluate_curve, Parameter}, geometry::{Curve, Transform}};
    let limits = Limits::default();
    let corners = [[0.,0.,0.], [4.,0.,0.], [4.,3.,0.], [0.,3.,0.], [0.,0.,0.]];
    let profile = Curve::Polycurve { segments: corners.windows(2).map(|p|
        Curve::Line { start: p[0], end: p[1], domain: [0.,1.] }).collect(), reversed: false };
    let shape = om9_kernel::extrusion::shell(&profile, [0.,0.,5.], 0.1, &limits).unwrap();
    assert_eq!(shape.topology().faces.len(), 4);
    let mut document = Document::new("open four-face shell");
    let entity = Entity::new("shell", Geometry::Brep(shape), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let path = std::env::temp_dir().join(format!("om9-border-shell-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, &document, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    let mut c = Controller::from_file(&path, limits).unwrap();
    std::fs::remove_file(path).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("DupBorder").unwrap();
    let curves: Vec<_> = c.document().entities.values().filter_map(|entity| match &entity.geometry {
        Geometry::Curve(curve) if entity.label == "DupBorder" => Some(curve), _ => None,
    }).collect();
    assert_eq!(curves.len(), 8);
    let mut edges = std::collections::BTreeSet::new();
    for curve in curves {
        let eval = |t| evaluate_curve(curve, Parameter::Fraction(t), 0, Transform::identity(), &limits).unwrap()[0];
        let (a,b) = (eval(0.), eval(1.));
        assert!((a[2]-b[2]).abs() < 1e-9 && (a[2].abs() < 1e-9 || (a[2]-5.).abs() < 1e-9));
        for p in [a,b] {
            assert!((p[0].abs() < 1e-9 || (p[0]-4.).abs() < 1e-9)
                && (p[1].abs() < 1e-9 || (p[1]-3.).abs() < 1e-9));
        }
        let corner = |p: [f64; 3]| [(p[0] / 4.).round() as i32, (p[1] / 3.).round() as i32, (p[2] / 5.).round() as i32];
        let (x,y) = (corner(a),corner(b));
        assert_eq!((0..2).filter(|i| x[*i] != y[*i]).count(), 1);
        assert!(edges.insert(if x < y { (x,y) } else { (y,x) }));
    }
    c.undo().unwrap(); assert_eq!(c.document().entities.len(), 1);
    c.redo().unwrap(); assert_eq!(c.document().entities.len(), 9);
}

#[test]
fn closed_solid_has_no_border_and_preserves_redo() {
    let limits = Limits::default();
    let mut c = Controller::new(Document::new("closed box"), limits).unwrap();
    c.submit("Box 4 3 5").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    c.submit("Point").unwrap(); c.submit("1,2,3").unwrap(); c.undo().unwrap();
    c.select_objects(&[id]).unwrap();
    let before = c.coordinator().snapshot();
    assert!(c.submit("DupBorder").is_err());
    assert_eq!(c.coordinator().snapshot(), before);
    assert_eq!(c.coordinator().redo_len(), 1);
}

#[test]
fn source_boundary_output_limit_excludes_non_boundary_seam() {
    let limits = Limits::default();
    let circle = om9_core::curve_build::circle_on_frame([0.; 3], [1.,0.,0.], [0.,1.,0.], 2.).unwrap();
    let shape = om9_kernel::extrusion::shell(&circle, [0.,0.,5.], 0.1, &limits).unwrap();
    assert_eq!(shape.topology().edges.len(), 3);
    let output_limits = Limits { max_outputs: 2, ..limits };
    let curves = om9_kernel::editing::source_boundary_edges_nurbs_admitted(&shape, &output_limits, |_, _| Ok(())).unwrap();
    assert_eq!(curves.len(), 2);
    let too_few = Limits { max_outputs: 1, ..limits };
    assert!(om9_kernel::editing::source_boundary_edges_nurbs_admitted(&shape, &too_few, |_, _| Ok(())).is_err());
}

#[test]
fn dup_border_certified_periodic_surface_keeps_only_open_boundary_curves() {
    use om9_core::{evaluation::{evaluate_curve, Parameter}, geometry::{NurbsSurface, Transform}};
    let limits=Limits::default(); let rows=[[0.,0.],[2.,0.],[2.,2.],[0.,2.],[0.,0.],[2.,0.]];
    let weights=[1.,0.7,1.2,0.9,1.,0.7];
    let surface=NurbsSurface {degree:[2,1],size:[6,2],
        poles:rows.iter().flat_map(|p|[[p[0],p[1],0.],[p[0],p[1],5.]]).collect(),
        weights:weights.iter().flat_map(|w|[*w,*w]).collect(),
        knots:[vec![-2.,-1.,0.,1.,2.,3.,4.,5.,6.],vec![0.,0.,1.,1.]],
        domain:[[0.,4.],[0.,1.]],periodic:[true,false],reversed:false};
    surface.certify_repeated_periodic_basis(&limits).unwrap();
    let mut entity=Entity::new("periodic surface",Geometry::Surface(surface.clone()),None);
    entity.placement=Transform::translation([10.,20.,30.]); let id=entity.id;
    let mut doc=Document::new("periodic surface border"); doc.entities.insert(id,entity);
    let path=std::env::temp_dir().join(format!("om9-periodic-border-{}.om9",uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path,&doc,&limits,&om9_core::transaction::Cancellation::default()).unwrap();
    let mut c=Controller::from_file(&path,limits).unwrap(); std::fs::remove_file(path).unwrap();
    c.select_objects(&[id]).unwrap(); let before=c.document().entities.clone(); c.submit("DupBorder").unwrap();
    let curves:Vec<_>=c.document().entities.values().filter(|e|e.label=="DupBorder").collect(); assert_eq!(curves.len(),2);
    for entity in curves {
        let Geometry::Curve(curve)=&entity.geometry else {panic!("curve")};
        assert!(om9_core::rhino_semantics::classify_curve(curve,1e-8,&limits).unwrap().closed,
            "certified periodic boundary must remain geometrically closed");
        let p=evaluate_curve(curve,Parameter::Fraction(0.),0,entity.placement,&limits).unwrap()[0];
        let v=if (p[2]-30.).abs()<1e-8 {0.} else {assert!((p[2]-35.).abs()<1e-8); 1.};
        for t in [0.,0.13,0.37,0.61,0.91,1.] {
            let local=om9_core::surface_kernel::evaluate(&surface,4.*t,v,&limits).unwrap();
            let expected=entity.placement.point(local).unwrap();
            let actual=evaluate_curve(curve,Parameter::Fraction(t),0,entity.placement,&limits).unwrap()[0];
            assert!((0..3).all(|i|(actual[i]-expected[i]).abs()<1e-8),"{t}: {actual:?} vs {expected:?}");
        }
    }
    let after=c.document().entities.clone(); c.undo().unwrap(); assert_eq!(c.document().entities,before);
    c.redo().unwrap(); assert_eq!(c.document().entities,after);

    // A boundary must remain usable as a closed extrusion profile after another save/open.
    let id=*c.document().entities.values().find(|e| {
        if let Geometry::Curve(curve)=&e.geometry {
            e.label=="DupBorder" && (evaluate_curve(curve,Parameter::Fraction(0.),0,e.placement,&limits).unwrap()[0][2]-30.).abs()<1e-8
        } else {false}
    }).map(|e|&e.id).unwrap();
    let path=std::env::temp_dir().join(format!("om9-periodic-border-profile-{}.om9",uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path,c.document(),&limits,&om9_core::transaction::Cancellation::default()).unwrap();
    let mut c=Controller::from_file(&path,limits).unwrap(); std::fs::remove_file(path).unwrap();
    let mut area2=0.; let mut previous=om9_core::surface_kernel::evaluate(&surface,0.,0.,&limits).unwrap();
    for i in 1..=4096 {
        let p=om9_core::surface_kernel::evaluate(&surface,4.*i as f64/4096.,0.,&limits).unwrap();
        area2+=previous[0]*p[1]-p[0]*previous[1]; previous=p;
    }
    let expected_volume=area2.abs()*0.5*5.;
    c.select_objects(&[id]).unwrap(); let before=c.document().entities.clone();
    c.submit("ExtrudeCrv 5").unwrap();
    let (body,placement)=c.document().entities.values().find_map(|e| match &e.geometry {
        Geometry::Brep(body) if !before.contains_key(&e.id)=>Some((body,e.placement)), _=>None,
    }).expect("extrusion must create a solid");
    let measure=om9_kernel::creation::measure(body,&limits).unwrap();
    assert_eq!(measure.solids,1); assert!(measure.closed);
    assert!((measure.volume_mm3-expected_volume).abs()<expected_volume*1e-5,"{measure:?} vs {expected_volume}");
    let bounds=om9_kernel::brep::world_bounds(body,&placement,&limits).unwrap();
    assert!((bounds[2]-30.).abs()<1e-5 && (bounds[5]-35.).abs()<1e-5,"{bounds:?}");
    assert!(bounds[0]>9.9 && bounds[3]<12.1 && bounds[1]>19.9 && bounds[4]<22.1,"{bounds:?}");
    let after=c.document().entities.clone(); c.undo().unwrap(); assert_eq!(c.document().entities,before);
    c.redo().unwrap(); assert_eq!(c.document().entities,after);
}

#[test]
fn dup_border_uncertified_periodic_surface_refuses_and_preserves_redo() {
    use om9_core::geometry::NurbsSurface;
    let limits=Limits::default(); let rows=[[0.,0.],[2.,0.],[2.,2.],[0.,2.],[0.1,0.],[2.,0.]];
    let surface=NurbsSurface {degree:[2,1],size:[6,2],
        poles:rows.iter().flat_map(|p|[[p[0],p[1],0.],[p[0],p[1],5.]]).collect(), weights:vec![1.;12],
        knots:[vec![-2.,-1.,0.,1.,2.,3.,4.,5.,6.],vec![0.,0.,1.,1.]],
        domain:[[0.,4.],[0.,1.]],periodic:[true,false],reversed:false};
    surface.validate(&limits).unwrap(); assert!(surface.certify_repeated_periodic_basis(&limits).is_err());
    let entity=Entity::new("uncertified",Geometry::Surface(surface),None); let id=entity.id;
    let mut doc=Document::new("uncertified border"); doc.entities.insert(id,entity);
    let mut c=Controller::new(doc,limits).unwrap(); c.submit("Point").unwrap(); c.submit("20,20,20").unwrap(); c.undo().unwrap();
    c.select_objects(&[id]).unwrap(); let before=c.coordinator().snapshot(); assert!(c.submit("DupBorder").is_err());
    assert_eq!(c.coordinator().snapshot(),before); assert_eq!(c.coordinator().redo_len(),1);
}

#[test]
fn dup_border_certified_closed_torus_refuses_empty_boundary_without_losing_redo() {
    use om9_core::geometry::NurbsSurface;
    let limits=Limits::default(); let circle=[[1.,0.],[0.,1.],[-1.,0.],[0.,-1.],[1.,0.],[0.,1.]];
    let weights=[1.,0.7,1.2,0.9,1.,0.7]; let knots=vec![-2.,-1.,0.,1.,2.,3.,4.,5.,6.];
    let mut poles=Vec::new(); let mut ws=Vec::new();
    for (i,[x,y]) in circle.iter().copied().enumerate() {for (j,[radial,z]) in circle.iter().copied().enumerate() {
        poles.push([(3.+0.5*radial)*x,(3.+0.5*radial)*y,0.5*z]); ws.push(weights[i]*weights[j]);
    }}
    let surface=NurbsSurface {degree:[2,2],size:[6,6],poles,weights:ws,knots:[knots.clone(),knots],
        domain:[[0.,4.],[0.,4.]],periodic:[true,true],reversed:false};
    surface.certify_repeated_periodic_basis(&limits).unwrap();
    let entity=Entity::new("torus",Geometry::Surface(surface),None); let id=entity.id;
    let mut doc=Document::new("closed periodic surface"); doc.entities.insert(id,entity);
    let mut c=Controller::new(doc,limits).unwrap(); c.submit("Point").unwrap(); c.submit("20,20,20").unwrap(); c.undo().unwrap();
    c.select_objects(&[id]).unwrap(); let before=c.coordinator().snapshot();
    assert!(c.submit("DupBorder").unwrap_err().to_string().contains("closed surface has no boundary"));
    assert_eq!(c.coordinator().snapshot(),before); assert_eq!(c.coordinator().redo_len(),1);
}

#[test]
fn dup_border_partial_cylinder_keeps_open_ends() {
    let limits=Limits::default();
    let mut surface=om9_core::surface_kernel::cylinder([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,5.,&limits).unwrap();
    surface.domain[0]=[0.,0.5];
    let entity=Entity::new("half cylinder",Geometry::Surface(surface),None); let id=entity.id;
    let mut document=Document::new("partial cylinder border"); document.entities.insert(id,entity);
    let mut c=Controller::new(document,limits).unwrap(); c.select_objects(&[id]).unwrap();
    c.submit("DupBorder").unwrap();
    assert_eq!(c.document().entities.values().filter(|e|e.label=="DupBorder").count(),4,"partial domain has four true boundaries");
}

#[test]
fn dup_border_almost_closed_surface_keeps_distinct_edges() {
    let limits=Limits::default();
    let mut surface=om9_core::surface_kernel::cylinder([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,5.,&limits).unwrap();
    surface.poles[16][0]+=1e-9; surface.poles[17][0]+=1e-9;
    let entity=Entity::new("nearly closed",Geometry::Surface(surface),None); let id=entity.id;
    let mut doc=Document::new("distinct near seam"); doc.entities.insert(id,entity);
    let mut c=Controller::new(doc,limits).unwrap(); c.select_objects(&[id]).unwrap(); c.submit("DupBorder").unwrap();
    assert_eq!(c.document().entities.values().filter(|e|e.label=="DupBorder").count(),4,
        "merely close control rows are not proof of a closed seam");
}

#[test]
fn dup_border_reopened_closed_cylinder_surface_excludes_parameter_seam() {
    use om9_core::{evaluation::{evaluate_curve, Parameter}, geometry::Transform};
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::cylinder([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,5.,&limits).unwrap();
    let placement=Transform([[2.,0.5,0.,10.],[0.,1.,0.,-2.],[0.,0.,1.,3.],[0.,0.,0.,1.]]);
    let mut entity=Entity::new("closed cylinder surface",Geometry::Surface(surface),None); entity.placement=placement;
    let id=entity.id; let mut document=Document::new("closed cylinder border"); document.entities.insert(id,entity);
    let path=std::env::temp_dir().join(format!("om9-closed-surface-border-{}.om9",uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path,&document,&limits,&om9_core::transaction::Cancellation::default()).unwrap();
    let mut c=Controller::from_file(&path,limits).unwrap(); std::fs::remove_file(path).unwrap();
    c.select_objects(&[id]).unwrap(); let before=c.document().entities.clone(); c.submit("DupBorder").unwrap();
    let curves:Vec<_>=c.document().entities.values().filter(|e|e.label=="DupBorder").collect();
    assert_eq!(curves.len(),2,"cylinder boundary consists of two circles, not two seam copies");
    for entity in curves {
        let Geometry::Curve(curve)=&entity.geometry else {panic!("curve")};
        let mut z: Option<f64>=None;
        for t in [0.,0.13,0.37,0.61,0.91,1.] {
            let p=evaluate_curve(curve,Parameter::Fraction(t),0,entity.placement,&limits).unwrap()[0];
            let y=p[1]+2.; let x=(p[0]-10.-0.5*y)/2.; assert!((x*x+y*y-4.).abs()<1e-8);
            assert!((p[2]-3.).abs()<1e-8 || (p[2]-8.).abs()<1e-8);
            if let Some(expected)=z {assert!((p[2]-expected).abs()<1e-8)} else {z=Some(p[2])}
        }
    }
    let after=c.document().entities.clone(); c.undo().unwrap(); assert_eq!(c.document().entities,before);
    c.redo().unwrap(); assert_eq!(c.document().entities,after);
}

#[test]
fn dup_border_reopened_placed_surface_matches_world_boundary() {
    use om9_core::{evaluation::{evaluate_curve, Parameter}, geometry::Transform};
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],4.,3.,&limits).unwrap();
    for (placement,menu) in [
        (Transform::translation([10.,20.,30.]),false),
        (Transform([[2.,0.5,0.,10.],[0.,1.,0.,20.],[0.,0.,1.,30.],[0.,0.,0.,1.]]),true),
    ] {
        let mut entity=Entity::new("placed surface",Geometry::Surface(surface.clone()),None); entity.placement=placement;
        let id=entity.id; let mut doc=Document::new("placed surface border"); doc.entities.insert(id,entity);
        let path=std::env::temp_dir().join(format!("om9-surface-border-{}.om9",uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&path,&doc,&limits,&om9_core::transaction::Cancellation::default()).unwrap();
        let mut c=Controller::from_file(&path,limits).unwrap(); std::fs::remove_file(&path).unwrap();
        c.select_objects(&[id]).unwrap(); let before=c.document().entities.clone();
        if menu {c.dispatch("OM9-CURVE-039",om9_app::session::InvocationOrigin::Menu).unwrap();} else {c.submit("DupBorder").unwrap();}
        let outputs:Vec<_>=c.document().entities.values().filter(|e|e.label=="DupBorder").collect(); assert_eq!(outputs.len(),4);
        let corners=[[0.,0.,0.],[4.,0.,0.],[4.,3.,0.],[0.,3.,0.]];
        for i in 0..4 {
            let expected=[placement.point(corners[i]).unwrap(),placement.point(corners[(i+1)%4]).unwrap()];
            assert!(outputs.iter().any(|entity| {
                let Geometry::Curve(curve)=&entity.geometry else {return false};
                let endpoints:Vec<_>=[0.,1.].into_iter().map(|t|evaluate_curve(curve,Parameter::Fraction(t),0,entity.placement,&limits).unwrap()[0]).collect();
                expected.iter().all(|p|endpoints.iter().any(|q|(0..3).all(|k|(p[k]-q[k]).abs()<1e-8)))
            }),"missing boundary {expected:?}");
        }
        let after=c.document().entities.clone();
        om9_archive::Archive::save(&path,c.document(),&limits,&om9_core::transaction::Cancellation::default()).unwrap();
        let reopened=Controller::from_file(&path,limits).unwrap(); std::fs::remove_file(&path).unwrap();
        for (id,entity) in &after {assert_eq!(reopened.document().entities[id].geometry,entity.geometry); assert_eq!(reopened.document().entities[id].placement,entity.placement);}
        c.undo().unwrap(); assert_eq!(c.document().entities,before);
        c.redo().unwrap(); assert_eq!(c.document().entities,after);
    }
}

#[test]
fn dup_border_placed_surface_output_limit_keeps_redo() {
    let limits=Limits {max_objects:4,..Limits::default()};
    let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],4.,3.,&limits).unwrap();
    let mut entity=Entity::new("placed surface",Geometry::Surface(surface),None);
    entity.placement=om9_core::geometry::Transform::translation([10.,20.,30.]); let id=entity.id;
    let mut doc=Document::new("surface border limit"); doc.entities.insert(id,entity);
    let mut c=Controller::new(doc,limits).unwrap(); c.submit("Point").unwrap(); c.submit("1,2,3").unwrap(); c.undo().unwrap();
    c.select_objects(&[id]).unwrap(); let before=c.coordinator().snapshot();
    assert!(c.submit("DupBorder").is_err()); assert_eq!(c.coordinator().snapshot(),before); assert_eq!(c.coordinator().redo_len(),1);
}

#[test]
fn dup_border_creates_four_boundary_curves() {
    let mut doc = Document::new("dup border");
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
    c.submit("DupBorder").unwrap();
    assert_eq!(
        c.document()
            .entities
            .values()
            .filter(|entity| entity.label == "DupBorder")
            .count(),
        4
    );
    c.undo().unwrap();
    assert_eq!(c.document().entities.values().filter(|entity| entity.label == "DupBorder").count(), 0);
}

#[test]
fn brep_dup_border_uses_exact_edges_independent_of_display_and_keeps_placement() {
    use om9_core::{evaluation::{evaluate_curve, Parameter}, geometry::{Curve, Transform}};
    let limits = Limits::default();
    let circle = om9_core::curve_build::circle_on_frame([0., 0., 0.], [1., 0., 0.], [0., 1., 0.], 2.).unwrap();
    let seed = om9_kernel::extrusion::shell(&circle, [0., 0., 5.], 0.1, &limits).unwrap();
    let mut outputs = Vec::new();
    for deflection in [0.5, 0.01] {
        let body = om9_kernel::brep::read_brep(seed.source(), deflection, &limits).unwrap();
        let mut entity = Entity::new("cylinder shell", Geometry::Brep(body), None);
        let mut placement = Transform::identity();
        placement.0[0][3] = 10.; placement.0[1][3] = -4.; placement.0[2][3] = 3.;
        entity.placement = placement;
        let id = entity.id;
        let mut document = Document::new("border source");
        document.entities.insert(id, entity);
        let mut controller = Controller::new(document, limits).unwrap();
        controller.select_objects(&[id]).unwrap();
        controller.submit("DupBorder").unwrap();
        let mut curves: Vec<_> = controller.document().entities.values().filter_map(|entity| {
            if entity.label != "DupBorder" { return None; }
            match &entity.geometry { Geometry::Curve(curve @ Curve::Nurbs(_)) => Some(curve.clone()),
                other => panic!("expected exact NURBS, got {other:?}") }
        }).collect();
        assert_eq!(curves.len(), 2);
        curves.sort_by(|a, b| {
            let point = |curve| evaluate_curve(curve, Parameter::Fraction(0.), 0, Transform::identity(), &limits).unwrap()[0];
            point(a)[2].total_cmp(&point(b)[2])
        });
        for (curve, z) in curves.iter().zip([3., 8.]) {
            for fraction in [0., 0.13, 0.37, 0.61, 0.91, 1.] {
                let p = evaluate_curve(curve, Parameter::Fraction(fraction), 0, Transform::identity(), &limits).unwrap()[0];
                assert!(((p[0] - 10.).hypot(p[1] + 4.) - 2.).abs() < 1e-8, "{p:?}");
                assert!((p[2] - z).abs() < 1e-8, "{p:?}");
            }
        }
        controller.undo().unwrap();
        assert_eq!(controller.document().entities.values().filter(|entity| entity.label == "DupBorder").count(), 0);
        controller.redo().unwrap();
        assert_eq!(controller.document().entities.values().filter(|entity| entity.label == "DupBorder").count(), 2);
        outputs.push(curves);
    }
    assert_eq!(outputs[0], outputs[1]);
}

#[test]
fn reopened_brep_dup_border_does_not_trust_persisted_edge_order() {
    use om9_core::{evaluation::{evaluate_curve, Parameter}, geometry::Transform};
    let limits = Limits::default();
    let circle = om9_core::curve_build::circle_on_frame([0.; 3], [1.,0.,0.], [0.,1.,0.], 2.).unwrap();
    let shape = om9_kernel::extrusion::shell(&circle, [0.,0.,5.], 0.1, &limits).unwrap();
    let mut display = shape.display().clone();
    let counts: Vec<_> = (0..display.edges.len()).map(|i| display.face_edges.iter().filter(|entry| entry[1] as usize == i).count()).collect();
    let boundary = counts.iter().position(|count| *count == 1).unwrap();
    let seam = counts.iter().position(|count| *count != 1).unwrap();
    display.edges.swap(boundary, seam);
    for entry in &mut display.face_edges {
        if entry[1] as usize == boundary { entry[1] = seam as u32; }
        else if entry[1] as usize == seam { entry[1] = boundary as u32; }
    }
    for segment in &mut display.segments {
        if segment[2] as usize == boundary { segment[2] = seam as u32; }
        else if segment[2] as usize == seam { segment[2] = boundary as u32; }
    }
    let shape = om9_core::brep::Brep::from_native(shape.source().to_owned(), display, &limits).unwrap();
    let mut document = Document::new("border reordered topology");
    let entity = Entity::new("cylinder shell", Geometry::Brep(shape), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let path = std::env::temp_dir().join(format!("om9-border-order-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, &document, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    let mut c = Controller::from_file(&path, limits).unwrap();
    std::fs::remove_file(path).unwrap();
    let original = c.document().entities[&id].clone();
    c.select_objects(&[id]).unwrap();
    let undo = c.coordinator().undo_len();
    c.submit("DupBorder").unwrap();
    let curves: Vec<_> = c.document().entities.values().filter_map(|entity| match &entity.geometry {
        Geometry::Curve(curve) if entity.label == "DupBorder" => Some(curve), _ => None,
    }).collect();
    assert_eq!(curves.len(), 2);
    for curve in curves {
        for t in [0., 0.25, 0.5, 0.75, 1.] {
            let p = evaluate_curve(curve, Parameter::Fraction(t), 0, Transform::identity(), &limits).unwrap()[0];
            assert!((p[0].hypot(p[1]) - 2.).abs() < 1e-8 && (p[2].abs() < 1e-8 || (p[2]-5.).abs() < 1e-8), "wrong boundary: {p:?}");
        }
    }
    assert_eq!(c.coordinator().undo_len(), undo + 1);
    c.undo().unwrap();
    assert_eq!(c.document().entities.len(), 1);
    assert_eq!(c.document().entities[&id], original);
    c.redo().unwrap();
    assert_eq!(c.document().entities.values().filter(|entity| entity.label == "DupBorder").count(), 2);
}

#[test]
fn brep_dup_border_pressure_keeps_document_and_redo_then_retries() {
    use om9_core::budget::memory::MemoryClass;
    let limits = Limits { max_bytes: 64 * 1024 * 1024, ..Limits::default() };
    let circle = om9_core::curve_build::circle_on_frame([0.; 3], [1.,0.,0.], [0.,1.,0.], 2.).unwrap();
    let shape = om9_kernel::extrusion::shell(&circle, [0.,0.,5.], 0.1, &limits).unwrap();
    let mut document = Document::new("border pressure");
    let entity = Entity::new("shell", Geometry::Brep(shape), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut c = Controller::new(document, limits).unwrap();
    c.submit("Point").unwrap(); c.submit("1,2,3").unwrap(); c.undo().unwrap();
    c.select_objects(&[id]).unwrap();
    let before = c.coordinator().snapshot();
    let pressure = c.coordinator().reserve_remaining_external_memory_with_held(
        MemoryClass::Kernel, None, c.scene_cache_usage().unwrap()).unwrap();
    assert!(c.submit("DupBorder").is_err());
    assert_eq!(c.coordinator().snapshot(), before);
    assert_eq!(c.coordinator().redo_len(), 1);
    drop(pressure);
    c.submit("DupBorder").unwrap();
    assert_eq!(c.document().entities.len(), 3);
}

#[test]
fn native_boundary_extraction_preserves_rational_nurbs_curve() {
    use om9_core::{evaluation::{evaluate_curve, Parameter}, geometry::{Curve, NurbsCurve, Transform}};
    let limits = Limits::default();
    let original = Curve::Nurbs(NurbsCurve { degree: 2,
        poles: vec![[0., 0., 0.], [1., 2., 0.], [2., 0., 0.]], weights: vec![1., 0.7, 1.],
        knots: vec![0., 0., 0., 1., 1., 1.], domain: [0., 1.], periodic: false, reversed: false, seam: None });
    let profile = Curve::Polycurve { segments: vec![original.clone(),
        Curve::Line { start: [2., 0., 0.], end: [0., 0., 0.], domain: [0., 1.] }], reversed: false };
    let shell = om9_kernel::extrusion::shell(&profile, [0., 0., 5.], 0.3, &limits).unwrap();
    let source_limits = Limits { max_points: 20, ..limits };
    assert!(shell.display().points.len() > 20);
    assert!(shell.validate(&source_limits).is_err());
    assert!(!om9_core::curve_surface_ops::boundary_edges(&shell, &source_limits).unwrap().is_empty());
    let point = |curve: &Curve, t| evaluate_curve(curve, Parameter::Fraction(t), 0, Transform::identity(), &limits).unwrap()[0];
    let curves: Vec<_> = (0..shell.topology().edges.len()).map(|index|
        om9_kernel::editing::edge_nurbs(&shell, index, &source_limits).unwrap()).collect();
    let indices: Vec<_> = (0..shell.topology().edges.len()).rev().collect();
    let batch = om9_kernel::editing::edges_nurbs(&shell, &indices, &limits).unwrap();
    assert_eq!(batch, curves.iter().rev().cloned().collect::<Vec<_>>());
    assert!(om9_kernel::editing::edges_nurbs(&shell, &[0, shell.topology().edges.len()], &limits).is_err());
    let matching = curves.iter().find(|curve| {
        let a = point(curve, 0.); let b = point(curve, 1.);
        a[2].abs() < 1e-9 && b[2].abs() < 1e-9 && (a[0] - b[0]).abs() > 1.9
            && point(curve, 0.5)[1] > 0.1
    }).expect("original curved boundary");
    let reversed = point(matching, 0.)[0] > point(matching, 1.)[0];
    for t in [0., 0.17, 0.38, 0.73, 1.] {
        let a = point(&original, t); let b = point(matching, if reversed { 1. - t } else { t });
        assert!((0..3).all(|axis| (a[axis] - b[axis]).abs() < 1e-9), "{a:?} {b:?}");
    }
}

#[test]
fn failed_brep_dup_border_leaves_document_and_history_unchanged() {
    let source_limits = Limits::default();
    let circle = om9_core::curve_build::circle_on_frame([0., 0., 0.], [1., 0., 0.], [0., 1., 0.], 2.).unwrap();
    let shell = om9_kernel::extrusion::shell(&circle, [0., 0., 5.], 0.1, &source_limits).unwrap();
    let mut document = Document::new("atomic border");
    let entity = Entity::new("shell", Geometry::Brep(shell), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let limits = Limits { max_objects: 3, ..source_limits };
    let mut controller = Controller::new(document, limits).unwrap();
    controller.submit("Point").unwrap();
    controller.submit("20,20,20").unwrap();
    controller.undo().unwrap();
    controller.select_objects(&[id]).unwrap();
    let before = controller.document().clone();
    let undo = controller.coordinator().undo_len();
    let redo = controller.coordinator().redo_len();
    assert_eq!(redo, 1);
    assert!(matches!(controller.submit("DupBorder"), Err(om9_core::Error::Budget(_))));
    assert_eq!(controller.document(), &before);
    assert_eq!(controller.coordinator().undo_len(), undo);
    assert_eq!(controller.coordinator().redo_len(), redo);
}
