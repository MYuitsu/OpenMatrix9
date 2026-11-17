#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};
use om9_kernel::creation;

// Check all default interior wires once, then let boundary-specific tests continue
// to exercise their original geometric assertions.
fn without_interior_wires<'a>(outputs: Vec<&'a Entity>, surface: &om9_core::geometry::NurbsSurface, limits: &Limits) -> Vec<&'a Entity> {
    use om9_core::{geometry::Curve, surface_kernel::{self, SurfaceIso}};
    let mut remaining = outputs;
    for axis in 0..2 {
        for parameter in surface_kernel::default_wire_parameters(surface, axis).unwrap() {
            let iso = if axis == 0 { SurfaceIso::V } else { SurfaceIso::U };
            let expected = surface_kernel::isocurve(surface, iso, parameter, limits).unwrap();
            let index = remaining.iter().position(|entity| matches!(&entity.geometry, Geometry::Curve(Curve::Nurbs(curve)) if *curve == expected))
                .expect("missing exact interior wire");
            remaining.remove(index);
        }
    }
    remaining
}

#[test]
fn extract_wireframe_rejects_trimmed_surfaces_without_partial_output() {
    use om9_core::rhino_semantics::{set_trimmed_surface, TrimmedSurface};
    let limits = Limits::default();
    let surface = om9_core::surface_kernel::cylinder([0.,0.,0.], [1.,0.,0.], [0.,1.,0.], 2., 5., &limits).unwrap();
    let trim = TrimmedSurface { underlying: surface.clone(), outer: vec![[0.2,0.2],[0.8,0.2],[0.8,0.8],[0.2,0.8]],
        inners: vec![vec![[0.4,0.4],[0.6,0.4],[0.5,0.6]]], reversed: false };
    let mut trimmed = Entity::new("trimmed", Geometry::Surface(surface), None);
    set_trimmed_surface(&mut trimmed, &trim, &limits).unwrap();
    let box_entity = Entity::new("box", Geometry::Brep(creation::primitive(
        creation::Primitive::Box { x_mm: 4., y_mm: 5., z_mm: 6. }, 0.1, &limits).unwrap()), None);
    let ids = [trimmed.id, box_entity.id];
    let mut document = Document::new("trimmed surface wireframe");
    for entity in [trimmed, box_entity] { document.entities.insert(entity.id, entity); }
    let seed = Controller::new(document, limits).unwrap();
    for multiple in [false, true] {
        for menu in [false, true] {
            let mut c = reopen_controller(&seed, limits);
            c.submit("Point").unwrap(); c.submit("1,2,3").unwrap(); c.undo().unwrap();
            c.select_objects(if multiple { &ids[..] } else { &ids[..1] }).unwrap();
            let before = c.document().clone();
            let result = if menu { c.dispatch("OM9-CURVE-054", om9_app::session::InvocationOrigin::Menu) }
                else { c.submit("ExtractWireframe") };
            assert!(result.is_err(), "trimmed source must not yield untrimmed boundaries: multiple={multiple}, menu={menu}");
            assert_eq!(c.document(), &before);
            assert_eq!(c.coordinator().redo_len(), 1);
        }
    }
}

#[test]
fn extract_wireframe_trimmed_plane_preserves_loops_and_undo() {
    use om9_core::evaluation::{evaluate_curve, Parameter};
    use om9_core::geometry::{Curve, Transform};
    use om9_core::rhino_semantics::{set_trimmed_surface, TrimmedSurface};
    let limits = Limits::default();
    let surface = om9_core::surface_kernel::plane([0.,0.,0.], [1.,0.,0.], [0.,1.,0.], 10., 10., &limits).unwrap();
    let outer = vec![[0.2,0.2],[0.8,0.2],[0.8,0.8],[0.2,0.8]];
    let inner = vec![[0.4,0.4],[0.6,0.4],[0.5,0.6]];
    let trim = TrimmedSurface { underlying: surface.clone(), outer: outer.clone(), inners: vec![inner.clone()], reversed: false };
    let mut entity = Entity::new("trimmed", Geometry::Surface(surface), None);
    set_trimmed_surface(&mut entity, &trim, &limits).unwrap();
    entity.placement = Transform::translation([20., 30., 4.]);
    let id = entity.id;
    let box_entity = Entity::new("box", Geometry::Brep(creation::primitive(
        creation::Primitive::Box { x_mm: 4., y_mm: 5., z_mm: 6. }, 0.1, &limits).unwrap()), None);
    let box_id = box_entity.id;
    let mut doc = Document::new("trimmed plane wireframe");
    doc.entities.insert(id, entity); doc.entities.insert(box_id, box_entity);
    let seed = Controller::new(doc, limits).unwrap();
    let selection = [id, box_id];
    let expected: Vec<_> = [outer, inner].iter().flat_map(|loop_| (0..loop_.len()).map(move |i| {
        [loop_[i], loop_[(i + 1) % loop_.len()]].map(|p| [20. + 10. * p[0], 30. + 10. * p[1], 4.])
    })).collect();
    for multiple in [false, true] {
        for menu in [false, true] {
            let mut c = reopen_controller(&seed, limits);
            c.submit("Point").unwrap(); c.submit("1,2,3").unwrap(); c.undo().unwrap();
            c.select_objects(if multiple { &selection[..] } else { &selection[..1] }).unwrap();
            let before = c.document().clone();
            if !multiple && !menu {
                let pressure = c.coordinator().reserve_remaining_external_memory_with_held(
                    om9_core::budget::memory::MemoryClass::Kernel, None, c.scene_cache_usage().unwrap()).unwrap();
                assert!(c.submit("ExtractWireframe").is_err());
                assert_eq!(c.document(), &before);
                assert_eq!(c.coordinator().redo_len(), 1);
                drop(pressure);
            }
            if menu { c.dispatch("OM9-CURVE-054", om9_app::session::InvocationOrigin::Menu).unwrap(); }
            else { c.submit("ExtractWireframe").unwrap(); }
            let outputs: Vec<_> = c.document().entities.values().filter(|e| e.label == "ExtractWire" && e.placement == Transform::translation([20.,30.,4.])).collect();
            assert_eq!(outputs.len(), 7);
            let mut seen = [false; 7];
            for entity in outputs {
                let Geometry::Curve(curve @ Curve::Line { .. }) = &entity.geometry else { panic!("trim edge must be a line") };
                let ends = [0.,1.].map(|t| evaluate_curve(curve, Parameter::Fraction(t), 0, entity.placement, &limits).unwrap()[0]);
                let near = |a: [f64; 3], b: [f64; 3]| (0..3).all(|i| (a[i]-b[i]).abs() < 1e-9);
                let index = expected.iter().position(|pair| (near(ends[0],pair[0]) && near(ends[1],pair[1])) || (near(ends[0],pair[1]) && near(ends[1],pair[0]))).unwrap();
                assert!(!seen[index]); seen[index] = true;
            }
            assert!(seen.into_iter().all(|v| v));
            let after = c.document().clone();
            assert_eq!(reopen_controller(&c, limits).document().entities, after.entities);
            c.undo().unwrap(); assert_eq!(c.document(), &before);
            c.redo().unwrap(); assert_eq!(c.document(), &after);
        }
    }
}

#[test]
fn trimmed_plane_commands_refuse_intersecting_or_nested_loops() {
    use om9_core::rhino_semantics::{set_trimmed_surface, TrimmedSurface};
    let limits = Limits::default();
    let surface = om9_core::surface_kernel::plane([0.,0.,0.], [1.,0.,0.], [0.,1.,0.], 10., 10., &limits).unwrap();
    let outer = vec![[0.1,0.1],[0.9,0.1],[0.9,0.9],[0.1,0.9]];
    let cases = [
        (outer.clone(), vec![vec![[0.2,0.2],[0.6,0.2],[0.6,0.6],[0.2,0.6]], vec![[0.4,0.4],[0.8,0.4],[0.8,0.8],[0.4,0.8]]]),
        (outer, vec![vec![[0.2,0.2],[0.8,0.2],[0.8,0.8],[0.2,0.8]], vec![[0.3,0.3],[0.4,0.3],[0.4,0.4],[0.3,0.4]]]),
        (vec![[0.1,0.1],[0.9,0.9],[0.1,0.9],[0.8,0.1]], vec![]),
        (vec![[0.1,0.1],[0.8,0.1],[0.5,0.1],[0.8,0.9],[0.1,0.9]], vec![]),
    ];
    for (outer, inners) in cases {
        let trim = TrimmedSurface { underlying: surface.clone(), outer, inners, reversed: false };
        let mut entity = Entity::new("ambiguous trim", Geometry::Surface(surface.clone()), None);
        set_trimmed_surface(&mut entity, &trim, &limits).unwrap(); let id = entity.id;
        let mut doc = Document::new("ambiguous trim"); doc.entities.insert(id, entity);
        let seed = Controller::new(doc, limits).unwrap();
        for command in ["ExtractWireframe", "DupBorder", "ExtractIsocurve"] {
            let mut c = reopen_controller(&seed, limits);
            c.submit("Point").unwrap(); c.submit("1,2,3").unwrap(); c.undo().unwrap(); c.select_objects(&[id]).unwrap();
            let before = c.document().clone();
            let started = c.submit(command);
            let result = if command == "ExtractIsocurve" && started.is_ok() { c.submit("") } else { started };
            assert!(result.is_err(), "{command} must refuse ambiguous loops");
            assert_eq!(c.document(), &before); assert_eq!(c.coordinator().redo_len(), 1);
        }
    }
}

#[test]
fn trimmed_plane_commands_accept_subdivided_straight_boundaries() {
    use om9_core::{evaluation::{evaluate_curve, Parameter}, geometry::{Curve, Transform}, rhino_semantics::{set_trimmed_surface, TrimmedSurface}};
    let limits = Limits::default();
    let surface = om9_core::surface_kernel::plane([0.,0.,0.], [1.,0.,0.], [0.,1.,0.], 10., 10., &limits).unwrap();
    let outer = vec![[0.1,0.1],[0.3,0.1],[0.9,0.1],[0.9,0.9],[0.1,0.9]];
    let inner = vec![[0.4,0.4],[0.5,0.4],[0.6,0.4],[0.6,0.6],[0.4,0.6]];
    let trim = TrimmedSurface { underlying: surface.clone(), outer: outer.clone(), inners: vec![inner.clone()], reversed: false };
    let mut entity = Entity::new("subdivided trim", Geometry::Surface(surface), None);
    set_trimmed_surface(&mut entity, &trim, &limits).unwrap();
    entity.placement = Transform::translation([20.,30.,4.]); let id = entity.id;
    let mut doc = Document::new("subdivided trim"); doc.entities.insert(id, entity);
    let seed = Controller::new(doc, limits).unwrap();
    let world = |p: [f64;2]| [20. + 10.*p[0], 30. + 10.*p[1], 4.];
    let boundary: Vec<_> = [outer, inner].iter().flat_map(|polygon| (0..polygon.len()).map(move |i| [world(polygon[i]), world(polygon[(i+1)%polygon.len()])])).collect();
    for (command, feature, label) in [("ExtractWireframe","OM9-CURVE-054","ExtractWire"),
        ("DupBorder","OM9-CURVE-039","DupBorder"), ("ExtractIsocurve","OM9-CURVE-019","ExtractIsocurve")] {
        for menu in [false, true] {
            for axis in if command == "ExtractIsocurve" { &['U','V'][..] } else { &['U'][..] } {
                let mut c = reopen_controller(&seed, limits); c.select_objects(&[id]).unwrap(); let before = c.document().clone();
                if menu { c.dispatch(feature, om9_app::session::InvocationOrigin::Menu).unwrap(); } else { c.submit(command).unwrap(); }
                if command == "ExtractIsocurve" { c.submit(if *axis == 'U' { "U=0.5" } else { "V=0.5" }).unwrap(); c.submit("").unwrap(); }
                let expected = if command != "ExtractIsocurve" { boundary.clone() } else if *axis == 'U' {
                    vec![[world([0.1,0.5]),world([0.4,0.5])],[world([0.6,0.5]),world([0.9,0.5])]]
                } else { vec![[world([0.5,0.1]),world([0.5,0.4])],[world([0.5,0.6]),world([0.5,0.9])]] };
                let outputs: Vec<_> = c.document().entities.values().filter(|e| e.label == label).collect();
                assert_eq!(outputs.len(), expected.len(), "{command}");
                let mut seen = vec![false; expected.len()];
                for output in outputs {
                    let Geometry::Curve(curve @ Curve::Line { .. }) = &output.geometry else { panic!("trim line") };
                    let p = [0.,1.].map(|t| evaluate_curve(curve, Parameter::Fraction(t), 0, output.placement, &limits).unwrap()[0]);
                    let near = |a: [f64;3], b: [f64;3]| (0..3).all(|i| (a[i]-b[i]).abs() < 1e-8);
                    let i = expected.iter().position(|q| (near(p[0],q[0]) && near(p[1],q[1])) || (near(p[0],q[1]) && near(p[1],q[0]))).unwrap();
                    assert!(!seen[i]); seen[i] = true;
                }
                assert!(seen.into_iter().all(|v| v)); let completed = c.document().clone();
                c.undo().unwrap(); assert_eq!(c.document(), &before); c.redo().unwrap(); assert_eq!(c.document(), &completed);
            }
        }
    }
}

#[test]
fn trimmed_bilinear_surface_diagonal_boundaries_preserve_source_geometry() {
    use om9_core::{evaluation::{evaluate_curve,Parameter},geometry::Transform,rhino_semantics::{set_trimmed_surface,TrimmedSurface}};
    let limits=Limits::default();
    for rational in [false,true] {
    let mut surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    surface.poles[3][2]=4.;
    if rational {surface.weights=vec![1.,2.,3.,4.];}
    let outer=vec![[0.1,0.1],[0.8,0.2],[0.7,0.9],[0.2,0.8]];
    let hole=vec![[0.35,0.35],[0.55,0.38],[0.45,0.6]];
    let trim=TrimmedSurface {underlying:surface.clone(),outer:outer.clone(),inners:vec![hole.clone()],reversed:false};
    let placement=Transform([[2.,0.5,0.,10.],[0.25,1.,0.,20.],[0.,0.,1.,30.],[0.,0.,0.,1.]]);
    let mut entity=Entity::new("warped trim",Geometry::Surface(surface),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();entity.placement=placement;let id=entity.id;
    let plane=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    let mut other=Entity::new("plane",Geometry::Surface(plane),None);other.placement=Transform::translation([40.,0.,0.]);let other_id=other.id;
    let mut doc=Document::new("bilinear trim");doc.entities.insert(id,entity);doc.entities.insert(other_id,other);let seed=Controller::new(doc,limits).unwrap();
    for (command,feature,label) in [("ExtractWireframe","OM9-CURVE-054","ExtractWire"),("DupBorder","OM9-CURVE-039","DupBorder")] {for menu in [false,true] {for multiple in [false,true] {
        let mut c=reopen_controller(&seed,limits);c.submit("Point").unwrap();c.submit("1,2,3").unwrap();c.undo().unwrap();let selection=[id,other_id];c.select_objects(if multiple {&selection[..]}else{&selection[..1]}).unwrap();let before=c.document().clone();
        let pressure=c.coordinator().reserve_remaining_external_memory_with_held(om9_core::budget::memory::MemoryClass::Kernel,None,c.scene_cache_usage().unwrap()).unwrap();
        assert!(c.submit(command).is_err());assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),1);drop(pressure);
        if menu {c.dispatch(feature,om9_app::session::InvocationOrigin::Menu).unwrap();}else{c.submit(command).unwrap();}
        let outputs:Vec<_>=c.document().entities.values().filter(|e|e.label==label).collect();assert_eq!(outputs.len(),if multiple {if command=="ExtractWireframe" {13}else{11}}else{7});
        let outputs:Vec<_>=outputs.into_iter().filter(|e|e.placement==placement).collect();assert_eq!(outputs.len(),7);let mut seen=[false;7];
        for output in outputs {let Geometry::Curve(curve)=&output.geometry else{panic!("curve")};
            let actual=[0.,0.2,0.5,0.8,1.].map(|t|evaluate_curve(curve,Parameter::Fraction(t),0,output.placement,&limits).unwrap()[0]);
            let index=(0..7).find(|i|[0.,0.2,0.5,0.8,1.].into_iter().enumerate().all(|(j,t)| {
                let polygon=if *i<4 {&outer}else{&hole};let k=if *i<4 {*i}else{*i-4};
                let a=polygon[k];let b=polygon[(k+1)%polygon.len()];let u=a[0]+t*(b[0]-a[0]);let v=a[1]+t*(b[1]-a[1]);
                let w=if rational {[1.,2.,3.,4.]}else{[1.;4]};
                let terms=[(1.-u)*(1.-v)*w[0],(1.-u)*v*w[1],u*(1.-v)*w[2],u*v*w[3]];let sum: f64=terms.iter().sum();
                let p=placement.point([10.*(terms[2]+terms[3])/sum,10.*(terms[1]+terms[3])/sum,4.*terms[3]/sum]).unwrap();(0..3).all(|k|(p[k]-actual[j][k]).abs()<1e-8)
            })).expect("analytic bilinear boundary");assert!(!seen[index]);seen[index]=true;
        }
        assert_eq!(seen,[true;7]);let after=c.document().clone();c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
        assert_eq!(reopen_controller(&c,limits).document().entities,after.entities);
    }}}
    }
}

#[test]
fn bilinear_trim_large_world_scale_refuses_without_partial_output() {
    use om9_core::{geometry::Transform,rhino_semantics::{set_trimmed_surface,TrimmedSurface}};
    let limits=Limits::default();let mut surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();surface.poles[3][2]=4.;
    let trim=TrimmedSurface {underlying:surface.clone(),outer:vec![[0.1,0.1],[0.8,0.2],[0.7,0.9],[0.2,0.8]],inners:vec![],reversed:false};
    let mut entity=Entity::new("large warped trim",Geometry::Surface(surface),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();entity.placement=Transform::translation([1e12,0.,0.]);let id=entity.id;
    let mut doc=Document::new("large warped trim");doc.entities.insert(id,entity);let seed=Controller::new(doc,limits).unwrap();
    for command in ["ExtractWireframe","DupBorder"] {let mut c=reopen_controller(&seed,limits);c.submit("Point").unwrap();c.submit("1,2,3").unwrap();c.undo().unwrap();c.select_objects(&[id]).unwrap();let before=c.document().clone();
        assert!(matches!(c.submit(command),Err(om9_core::Error::Unsupported("bilinear trim precision is uncertain"))));assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),1);}
}

#[test]
fn trimmed_cylinder_unverified_seam_is_refused_atomically() {
    use om9_core::rhino_semantics::{set_trimmed_surface,TrimmedSurface};
    let limits=Limits::default();let surface=om9_core::surface_kernel::cylinder([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,5.,&limits).unwrap();
    let trim=TrimmedSurface {underlying:surface.clone(),outer:vec![[0.,0.2],[1.,0.2],[0.9,0.8],[0.,0.8]],inners:vec![],reversed:false};
    let mut entity=Entity::new("full seam",Geometry::Surface(surface),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();let id=entity.id;
    let mut doc=Document::new("full seam trim");doc.entities.insert(id,entity);let seed=Controller::new(doc,limits).unwrap();
    for command in ["ExtractWireframe","DupBorder"] {
        let mut c=reopen_controller(&seed,limits);c.submit("Point").unwrap();c.submit("1,2,3").unwrap();c.undo().unwrap();c.select_objects(&[id]).unwrap();
        let before=c.document().clone();assert!(matches!(c.submit(command),Err(om9_core::Error::Unsupported("trim spans a closed surface seam"))));
        assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),1);
    }
}

#[test]
fn certified_full_seam_cylinder_produces_physical_borders() {
    use om9_core::{evaluation::{evaluate_curve,Parameter},geometry::Transform,rhino_semantics::{set_trimmed_surface,TrimmedSurface}};
    let limits=Limits::default();let surface=om9_core::surface_kernel::cylinder([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,5.,&limits).unwrap();
    let trim=TrimmedSurface {underlying:surface.clone(),outer:vec![[0.,0.2],[1.,0.2],[1.,0.8],[0.,0.8]],
        inners:vec![vec![[0.3,0.4],[0.6,0.4],[0.6,0.6],[0.3,0.6]]],reversed:false};
    let mut entity=Entity::new("full cylinder",Geometry::Surface(surface),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();entity.placement=Transform::translation([10.,20.,30.]);let id=entity.id;
    let mut doc=Document::new("full seam trim");doc.entities.insert(id,entity);let seed=Controller::new(doc,limits).unwrap();
    for (command,feature,label,expected) in [("ExtractWireframe","OM9-CURVE-054","ExtractWire",7),("DupBorder","OM9-CURVE-039","DupBorder",6)] {
        for menu in [false,true] {
            let mut c=reopen_controller(&seed,limits);c.submit("Point").unwrap();c.submit("1,2,3").unwrap();c.undo().unwrap();c.select_objects(&[id]).unwrap();let before=c.document().clone();
            let pressure=c.coordinator().reserve_remaining_external_memory_with_held(om9_core::budget::memory::MemoryClass::Kernel,None,c.scene_cache_usage().unwrap()).unwrap();
            assert!(c.submit(command).is_err());assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),1);drop(pressure);
            if menu {c.dispatch(feature,om9_app::session::InvocationOrigin::Menu).unwrap();}else{c.submit(command).unwrap();}
            let outputs:Vec<_>=c.document().entities.values().filter(|e|e.label==label).collect();assert_eq!(outputs.len(),expected);
            let mut seam=0;let mut circles=[false;2];
            for e in outputs {let Geometry::Curve(curve)=&e.geometry else{panic!("curve")};
                let p=[0.,0.23,0.61,1.].map(|t|evaluate_curve(curve,Parameter::Fraction(t),0,e.placement,&limits).unwrap()[0]);
                for q in p {assert!(((q[0]-10.).hypot(q[1]-20.)-2.).abs()<1e-8);assert!(q[2]>=31.-1e-8&&q[2]<=34.+1e-8);}
                if p.iter().all(|q|(q[0]-12.).abs()<1e-8&&(q[1]-20.).abs()<1e-8) {seam+=1;}
                for (i,z) in [31.,34.].into_iter().enumerate() {if p.iter().all(|q|(q[2]-z).abs()<1e-8) {assert!(!circles[i]);circles[i]=true;}}
            }
            assert_eq!(seam,if command=="DupBorder" {0}else{1});assert_eq!(circles,[true,true]);
            let after=c.document().clone();c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
            assert_eq!(reopen_controller(&c,limits).document().entities,after.entities);
        }
    }
    for (command,label,expected) in [("DupBorder","DupBorder",6),("ExtractWireframe","ExtractWire",7)] {for maximum in [expected-1,expected] {
        let restricted=Limits {max_outputs:maximum,..limits};let mut c=reopen_controller(&seed,restricted);
        c.submit("Point").unwrap();c.submit("1,2,3").unwrap();c.undo().unwrap();c.select_objects(&[id]).unwrap();let before=c.document().clone();
        if maximum<expected {assert!(c.submit(command).is_err());assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),1);}
        else {c.submit(command).unwrap();assert_eq!(c.document().entities.values().filter(|e|e.label==label).count() as u64,expected);}
    }}
}

#[test]
fn trimmed_cylinder_axis_boundaries_are_exact_after_reopen() {
    use om9_core::{evaluation::{evaluate_curve,Parameter},geometry::Transform,rhino_semantics::{set_trimmed_surface,TrimmedSurface}};
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::cylinder([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,5.,&limits).unwrap();
    let outer=vec![[0.1,0.2],[0.8,0.2],[0.8,0.8],[0.1,0.8]];
    let hole=vec![[0.3,0.4],[0.6,0.4],[0.6,0.6],[0.3,0.6]];
    let trim=TrimmedSurface {underlying:surface.clone(),outer:outer.clone(),inners:vec![hole.clone()],reversed:false};
    for placement in [Transform::translation([10.,20.,30.]),Transform([[2.,0.5,0.,10.],[0.25,1.,0.,20.],[0.,0.,1.,30.],[0.,0.,0.,1.]])] {
    let mut entity=Entity::new("trim cylinder",Geometry::Surface(surface.clone()),None);
    set_trimmed_surface(&mut entity,&trim,&limits).unwrap();entity.placement=placement;let id=entity.id;
    let plane=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    let mut other=Entity::new("plane",Geometry::Surface(plane),None);other.placement=Transform::translation([40.,0.,0.]);let other_id=other.id;
    let mut doc=Document::new("trim cylinder");doc.entities.insert(id,entity);doc.entities.insert(other_id,other);let seed=Controller::new(doc,limits).unwrap();
    for (command,feature,label) in [("ExtractWireframe","OM9-CURVE-054","ExtractWire"),("DupBorder","OM9-CURVE-039","DupBorder")] {
        for menu in [false,true] {for multiple in [false,true] {
            let mut c=reopen_controller(&seed,limits);c.submit("Point").unwrap();c.submit("1,2,3").unwrap();c.undo().unwrap();
            let selection=[id,other_id];c.select_objects(if multiple {&selection[..]} else {&selection[..1]}).unwrap();let before=c.document().clone();
            let pressure=c.coordinator().reserve_remaining_external_memory_with_held(om9_core::budget::memory::MemoryClass::Kernel,None,c.scene_cache_usage().unwrap()).unwrap();
            assert!(c.submit(command).is_err());assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),1);drop(pressure);
            if menu {c.dispatch(feature,om9_app::session::InvocationOrigin::Menu).unwrap();}else{c.submit(command).unwrap();}
            let outputs:Vec<_>=c.document().entities.values().filter(|e|e.label==label).collect();assert_eq!(outputs.len(),if multiple {if command=="ExtractWireframe" {14}else{12}}else{8});
            let outputs:Vec<_>=outputs.into_iter().filter(|e|e.placement==placement).collect();assert_eq!(outputs.len(),8);
            let mut seen=[false;8];let mut arc_id=None;
            for output in outputs {
                let Geometry::Curve(curve)=&output.geometry else {panic!("boundary")};
                let actual=[0.,0.23,0.61,1.].map(|t|evaluate_curve(curve,Parameter::Fraction(t),0,output.placement,&limits).unwrap()[0]);
                for p in actual {let (dx,dy)=(p[0]-10.,p[1]-20.);let (x,y)=if placement.0[0][0]==1. {(dx,dy)}else{let x=(dx-0.5*dy)/1.875;(x,dy-0.25*x)};
                    assert!((x.hypot(y)-2.).abs()<1e-8);assert!(p[2]>=31.-1e-8&&p[2]<=34.+1e-8);}
                let index=(0..8).find(|i| {
                    let polygon=if *i<4 {&outer}else{&hole};let j=*i%4;
                    let a=polygon[j];let b=polygon[(j+1)%4];
                    [0.,0.23,0.61,1.].into_iter().enumerate().all(|(j,t)| {
                        let uv=[a[0]+t*(b[0]-a[0]),a[1]+t*(b[1]-a[1])];
                        let expected=output.placement.point(om9_core::surface_kernel::evaluate(&surface,uv[0],uv[1],&limits).unwrap()).unwrap();
                        (0..3).all(|k|(actual[j][k]-expected[k]).abs()<1e-8)
                    })
                }).expect("source UV edge including orientation");
                assert!(!seen[index]);seen[index]=true;if index==0 {arc_id=Some(output.id);}
            }
            assert_eq!(seen,[true;8]);let after=c.document().clone();c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
            let mut reopened=reopen_controller(&c,limits);assert_eq!(reopened.document().entities,after.entities);
            if placement.0[0][0]==1. {
                let a=om9_core::surface_kernel::evaluate(&surface,outer[0][0],outer[0][1],&limits).unwrap();
                let b=om9_core::surface_kernel::evaluate(&surface,outer[1][0],outer[1][1],&limits).unwrap();
                let angle=(b[1].atan2(b[0])-a[1].atan2(a[0])).rem_euclid(std::f64::consts::TAU);
                reopened.select_objects(&[arc_id.unwrap()]).unwrap();let measured=reopened.measure_selected_length().unwrap().value;
                assert!((measured-2.*angle).abs()<1e-5,"{measured} vs {}",2.*angle);
            }
        }}
    }
    }
}

#[test]
fn trimmed_plane_with_large_parameter_offset_preserves_source_boundaries() {
    use om9_core::{evaluation::{evaluate_curve, Parameter}, geometry::{Curve, Transform}, rhino_semantics::{set_trimmed_surface, TrimmedSurface}};
    let limits = Limits::default(); let offset = 1e8;
    let mut surface = om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    surface.knots = [vec![offset,offset,offset+4.,offset+4.],vec![offset,offset,offset+4.,offset+4.]];
    surface.domain = [[offset,offset+4.],[offset,offset+4.]];
    let outer = vec![[1.,1.],[3.,1.],[3.,3.],[1.,3.]];
    let inner = vec![[1.5,1.5],[2.,1.5],[2.,2.],[1.5,2.]];
    let shift = |loop_: &Vec<[f64;2]>| loop_.iter().map(|p| [p[0]+offset,p[1]+offset]).collect();
    let trim = TrimmedSurface {underlying:surface.clone(),outer:shift(&outer),inners:vec![shift(&inner)],reversed:false};
    let mut entity = Entity::new("offset trim",Geometry::Surface(surface),None);
    set_trimmed_surface(&mut entity,&trim,&limits).unwrap(); entity.placement=Transform::translation([20.,30.,4.]); let id=entity.id;
    let mut doc=Document::new("offset trim"); doc.entities.insert(id,entity); let seed=Controller::new(doc,limits).unwrap();
    let world=|p:[f64;2]| [20.+2.5*p[0],30.+2.5*p[1],4.];
    let expected:Vec<_>=[outer,inner].iter().flat_map(|poly| (0..poly.len()).map(move |i| [world(poly[i]),world(poly[(i+1)%poly.len()])])).collect();
    for (command,feature,label) in [("ExtractWireframe","OM9-CURVE-054","ExtractWire"),("DupBorder","OM9-CURVE-039","DupBorder")] {
        for menu in [false,true] {
            let mut c=reopen_controller(&seed,limits); c.select_objects(&[id]).unwrap(); let before=c.document().clone();
            if menu {c.dispatch(feature,om9_app::session::InvocationOrigin::Menu).unwrap()} else {c.submit(command).unwrap()}
            let outputs:Vec<_>=c.document().entities.values().filter(|e|e.label==label).collect(); assert_eq!(outputs.len(),8);
            let mut seen=[false;8];
            for output in outputs {
                let Geometry::Curve(curve @ Curve::Line {..})=&output.geometry else {panic!("trim boundary")};
                let points=[0.,1.].map(|t|evaluate_curve(curve,Parameter::Fraction(t),0,output.placement,&limits).unwrap()[0]);
                let near=|a:[f64;3],b:[f64;3]|(0..3).all(|i|(a[i]-b[i]).abs()<1e-8);
                let i=expected.iter().position(|p|(near(points[0],p[0])&&near(points[1],p[1]))||(near(points[0],p[1])&&near(points[1],p[0]))).unwrap();
                assert!(!seen[i]); seen[i]=true;
            }
            assert!(seen.into_iter().all(|v|v)); let after=c.document().clone(); c.undo().unwrap(); assert_eq!(c.document(),&before); c.redo().unwrap(); assert_eq!(c.document(),&after);
        }
    }
}

#[test]
fn trimmed_affine_plane_with_nonunit_restricted_domains_uses_source_parameters() {
    use om9_core::{evaluation::{evaluate_curve, Parameter}, geometry::{Curve, Transform}, rhino_semantics::{set_trimmed_surface, TrimmedSurface}};
    let limits = Limits::default();
    let mut surface = om9_core::surface_kernel::plane([0.,0.,0.], [1.,0.,0.], [0.,1.,0.], 10., 10., &limits).unwrap();
    surface.knots = [vec![10.,10.,20.,20.], vec![2.,2.,6.,6.]];
    surface.domain = [[12.,18.],[3.,5.]];
    let outer = vec![[13.,3.2],[17.,3.2],[17.,4.8],[13.,4.8]];
    let inner = vec![[14.,3.6],[16.,3.6],[16.,4.4],[14.,4.4]];
    let trim = TrimmedSurface { underlying: surface.clone(), outer: outer.clone(), inners: vec![inner.clone()], reversed: false };
    let mut entity = Entity::new("trimmed nonunit", Geometry::Surface(surface), None);
    set_trimmed_surface(&mut entity, &trim, &limits).unwrap(); entity.placement = Transform::translation([20.,30.,4.]); let id = entity.id;
    let mut doc = Document::new("trimmed nonunit domain"); doc.entities.insert(id, entity);
    let seed = Controller::new(doc, limits).unwrap();
    let world = |p: [f64;2]| [20. + (p[0]-10.), 30. + (p[1]-2.)*2.5, 4.];
    let boundary: Vec<_> = [outer, inner].iter().flat_map(|loop_| (0..loop_.len()).map(move |i| [world(loop_[i]), world(loop_[(i+1)%loop_.len()])])).collect();
    for (command, feature, label) in [("ExtractWireframe","OM9-CURVE-054","ExtractWire"),
        ("DupBorder","OM9-CURVE-039","DupBorder"), ("ExtractIsocurve","OM9-CURVE-019","ExtractIsocurve")] {
        for menu in [false, true] {
            for axis in if command == "ExtractIsocurve" { &['U', 'V'][..] } else { &['U'][..] } {
            let mut c = reopen_controller(&seed, limits); c.select_objects(&[id]).unwrap(); let before = c.document().clone();
            if menu { c.dispatch(feature, om9_app::session::InvocationOrigin::Menu).unwrap(); }
            else { c.submit(command).unwrap(); }
            if command == "ExtractIsocurve" { c.submit(if *axis == 'U' { "U=4" } else { "V=15" }).unwrap(); c.submit("").unwrap(); }
            let expected = if command != "ExtractIsocurve" { boundary.clone() }
                else if *axis == 'U' { vec![[[23.,35.,4.],[24.,35.,4.]], [[26.,35.,4.],[27.,35.,4.]]] }
                else { vec![[[25.,33.,4.],[25.,34.,4.]], [[25.,36.,4.],[25.,37.,4.]]] };
            let outputs: Vec<_> = c.document().entities.values().filter(|e| e.label == label).collect();
            assert_eq!(outputs.len(), expected.len(), "{command}");
            let mut seen = vec![false; expected.len()];
            for output in outputs {
                let Geometry::Curve(curve @ Curve::Line { .. }) = &output.geometry else { panic!("trim edge") };
                let p = [0.,1.].map(|t| evaluate_curve(curve, Parameter::Fraction(t), 0, output.placement, &limits).unwrap()[0]);
                let near = |a: [f64;3], b: [f64;3]| (0..3).all(|i| (a[i]-b[i]).abs() < 1e-8);
                let i = expected.iter().position(|q| (near(p[0],q[0]) && near(p[1],q[1])) || (near(p[0],q[1]) && near(p[1],q[0]))).unwrap();
                assert!(!seen[i]); seen[i] = true;
            }
            assert!(seen.into_iter().all(|v| v)); c.undo().unwrap(); assert_eq!(c.document(), &before); c.redo().unwrap();
            }
        }
    }
}

#[test]
fn extract_wireframe_refuses_trim_above_topology_limit_without_reclaiming_undo() {
    use om9_core::rhino_semantics::{set_trimmed_surface, TrimmedSurface};
    let limits = Limits { max_name_bytes: 64 * 1024, ..Limits::default() };
    let surface = om9_core::surface_kernel::plane([0.,0.,0.], [1.,0.,0.], [0.,1.,0.], 10., 10., &limits).unwrap();
    let outer = (0..1025).map(|i| {
        let angle = std::f64::consts::TAU * f64::from(i) / 1025.;
        [0.5 + 0.35 * angle.cos(), 0.5 + 0.35 * angle.sin()]
    }).collect();
    let trim = TrimmedSurface { underlying: surface.clone(), outer, inners: vec![], reversed: false };
    let mut entity = Entity::new("large trim", Geometry::Surface(surface), None);
    set_trimmed_surface(&mut entity, &trim, &limits).unwrap();
    assert!(entity.extensions[om9_core::model::TRIM_SURFACE_EXTENSION_SLOT].payload.len() > 16 * 1024);
    let id = entity.id;
    let mut doc = Document::new("large trim metadata"); doc.entities.insert(id, entity);
    let mut c = reopen_controller(&Controller::new(doc, limits).unwrap(), limits);
    c.submit("Point").unwrap(); c.submit("1,2,3").unwrap(); c.undo().unwrap(); c.select_objects(&[id]).unwrap();
    let before = c.document().clone();
    assert!(matches!(c.submit("ExtractWireframe"), Err(om9_core::Error::Unsupported(_))));
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().redo_len(), 1);
}

#[test]
fn extract_wireframe_large_surface_trim_survives_reopen_and_undo() {
    use om9_core::{evaluation::{evaluate_curve,Parameter},rhino_semantics::{set_trimmed_surface,TrimmedSurface}};
    let limits=Limits::default();let n=28usize;
    let mut poles=Vec::new();for u in 0..n {for v in 0..n {poles.push([10.*u as f64/(n-1) as f64,10.*v as f64/(n-1) as f64,0.]);}}
    let mut knots=vec![0.];knots.extend((0..n).map(|i|i as f64/(n-1) as f64));knots.push(1.);
    let surface=om9_core::geometry::NurbsSurface {degree:[1,1],size:[n,n],poles,weights:vec![1.;n*n],knots:[knots.clone(),knots],domain:[[0.,1.],[0.,1.]],periodic:[false,false],reversed:false};
    let outer=vec![[0.2,0.2],[0.8,0.2],[0.8,0.8],[0.2,0.8]];
    let hole=vec![[0.4,0.4],[0.6,0.4],[0.6,0.6],[0.4,0.6]];
    let trim=TrimmedSurface {underlying:surface.clone(),outer:outer.clone(),inners:vec![hole.clone()],reversed:false};
    let mut entity=Entity::new("large trimmed",Geometry::Surface(surface),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();
    assert!(entity.extensions[om9_core::model::TRIM_SURFACE_EXTENSION_SLOT].payload.len()>16*1024);
    let id=entity.id;let mut doc=Document::new("large trimmed source");doc.entities.insert(id,entity);
    let mut c=reopen_controller(&Controller::new(doc,limits).unwrap(),limits);
    c.submit("Point").unwrap();c.submit("100,100,100").unwrap();c.undo().unwrap();
    c.select_objects(&[id]).unwrap();let before=c.document().clone();
    let pressure=c.coordinator().reserve_remaining_external_memory_with_held(om9_core::budget::memory::MemoryClass::Kernel,None,c.scene_cache_usage().unwrap()).unwrap();
    assert!(c.submit("ExtractWireframe").is_err());assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),1);drop(pressure);
    c.submit("ExtractWireframe").unwrap();
    let outputs:Vec<_>=c.document().entities.values().filter(|e|e.id!=id).collect();assert_eq!(outputs.len(),8);
    let expected:Vec<_>=[outer,hole].into_iter().flat_map(|p| (0..p.len()).map(|i| {
        let a=p[i];let b=p[(i+1)%p.len()];[[a[0]*10.,a[1]*10.,0.],[b[0]*10.,b[1]*10.,0.]]
    }).collect::<Vec<_>>()).collect();
    let mut seen=vec![false;8];
    for output in outputs {
        let Geometry::Curve(curve)=&output.geometry else {panic!("expected curve")};
        let points=[0.,1.].map(|t|evaluate_curve(curve,Parameter::Fraction(t),0,output.placement,&limits).unwrap()[0]);
        let near=|a:[f64;3],b:[f64;3]|(0..3).all(|i|(a[i]-b[i]).abs()<1e-7);
        let index=expected.iter().position(|e|(near(points[0],e[0])&&near(points[1],e[1]))||(near(points[0],e[1])&&near(points[1],e[0]))).unwrap();
        assert!(!seen[index]);seen[index]=true;
    }
    assert!(seen.into_iter().all(|x|x));c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document().entities.len(),9);
}

#[test]
fn large_trim_boundary_commands_are_atomic_after_reopen() {
    use om9_core::{budget::memory::MemoryClass,evaluation::{evaluate_curve,Parameter},geometry::Transform,
        rhino_semantics::{set_trimmed_surface,TrimmedSurface}};
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
    let outer:Vec<_>=(0..501).map(|i|{let a=std::f64::consts::TAU*(i as f64)/501.;[0.5+0.4*a.cos(),0.5+0.4*a.sin()]}).collect();
    let hole=vec![[0.4,0.4],[0.6,0.4],[0.6,0.6],[0.4,0.6]];
    let trim=TrimmedSurface {underlying:surface.clone(),outer:outer.clone(),inners:vec![hole.clone()],reversed:false};
    let mut entity=Entity::new("large trim",Geometry::Surface(surface),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();
    let placement=Transform([[2.,0.5,0.,3.],[0.,1.,0.,-2.],[0.,0.,1.,7.],[0.,0.,0.,1.]]);entity.placement=placement;let id=entity.id;
    let mut doc=Document::new("large boundary");doc.entities.insert(id,entity);
    let seed=Controller::new(doc,limits).unwrap();
    let expected:Vec<_>=[outer,hole].iter().flat_map(|ring|(0..ring.len()).map(|i| {
        [ring[i],ring[(i+1)%ring.len()]].map(|p|placement.point([10.*p[0],10.*p[1],0.]).unwrap())
    })).collect();
    for (command,label,menu_id) in [("ExtractWireframe","ExtractWire","OM9-CURVE-054"),("DupBorder","DupBorder","OM9-CURVE-039")] {
        for menu in [false,true] {
            let mut c=reopen_controller(&seed,limits);
            c.submit("Point").unwrap();c.submit("100,100,100").unwrap();c.undo().unwrap();c.select_objects(&[id]).unwrap();
            let before=c.document().clone();
            if !menu {
                let pressure=c.coordinator().reserve_remaining_external_memory_with_held(MemoryClass::Kernel,None,c.scene_cache_usage().unwrap()).unwrap();
                assert!(c.submit(command).is_err());assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),1);drop(pressure);
            }
            if menu {c.dispatch(menu_id,om9_app::session::InvocationOrigin::Menu).unwrap();}else{c.submit(command).unwrap();}
            let outputs:Vec<_>=c.document().entities.values().filter(|e|e.label==label).collect();assert_eq!(outputs.len(),505,"{command}, menu={menu}");
            let mut seen=vec![false;expected.len()];
            for e in outputs {let Geometry::Curve(curve)=&e.geometry else {panic!("curve")};
                let points=[0.,1.].map(|t|evaluate_curve(curve,Parameter::Fraction(t),0,e.placement,&limits).unwrap()[0]);
                let near=|a:[f64;3],b:[f64;3]|(0..3).all(|i|(a[i]-b[i]).abs()<1e-7);
                let index=expected.iter().position(|pair|(near(points[0],pair[0])&&near(points[1],pair[1]))||(near(points[0],pair[1])&&near(points[1],pair[0]))).expect("unexpected edge");
                assert!(!seen[index]);seen[index]=true;
            }
            assert!(seen.into_iter().all(|value|value));let after=c.document().clone();
            assert_eq!(reopen_controller(&c,limits).document().entities,after.entities);
            c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
        }
    }
}

#[test]
fn large_trim_boundary_commands_respect_topology_and_output_limits() {
    use om9_core::{budget::memory::MemoryClass,evaluation::{evaluate_curve,Parameter},rhino_semantics::{set_trimmed_surface,TrimmedSurface}};
    let limits=Limits::default();
    for count in [708,1024,1025] {
        let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap();
        let outer=(0..count).map(|i|{let a=std::f64::consts::TAU*(i as f64)/(count as f64);[0.5+0.3*a.cos(),0.5+0.3*a.sin()]}).collect();
        let trim=TrimmedSurface {underlying:surface.clone(),outer,inners:vec![],reversed:false};
        let mut entity=Entity::new("trim",Geometry::Surface(surface),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();let id=entity.id;
        let mut doc=Document::new("boundary limits");doc.entities.insert(id,entity);
        for command in ["ExtractWireframe","DupBorder"] {
            let active_limits=limits;
            let mut c=reopen_controller(&Controller::new(doc.clone(),active_limits).unwrap(),active_limits);
            c.submit("Point").unwrap();c.submit("100,100,100").unwrap();c.undo().unwrap();c.select_objects(&[id]).unwrap();
            let before=c.document().clone();
            if count==1025 {
                assert!(matches!(c.submit(command),Err(om9_core::Error::Unsupported(_))));assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),1);
            } else {
                if count==1024 {
                    let pressure=c.coordinator().reserve_remaining_external_memory_with_held(MemoryClass::Kernel,None,c.scene_cache_usage().unwrap()).unwrap();
                    assert!(c.submit(command).is_err());assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),1);drop(pressure);
                }
                if command=="DupBorder" {c.dispatch("OM9-CURVE-039",om9_app::session::InvocationOrigin::Menu).unwrap();}
                else {c.submit(command).unwrap();}
                let after=c.document().clone();assert_eq!(after.entities.len(),count+1);
                let expected:Vec<_>=(0..count).map(|i|[trim.outer[i],trim.outer[(i+1)%count]]
                    .map(|p|[10.*p[0],10.*p[1],0.])).collect();
                let mut seen=vec![false;count];
                for output in after.entities.values().filter(|e|e.id!=id) {
                    let Geometry::Curve(curve)=&output.geometry else {panic!("expected curve")};
                    let ends=[0.,1.].map(|t|evaluate_curve(curve,Parameter::Fraction(t),0,output.placement,&limits).unwrap()[0]);
                    let near=|a:[f64;3],b:[f64;3]|(0..3).all(|i|(a[i]-b[i]).abs()<1e-7);
                    let index=expected.iter().position(|pair|(near(ends[0],pair[0])&&near(ends[1],pair[1]))
                        ||(near(ends[0],pair[1])&&near(ends[1],pair[0]))).expect("unexpected output edge");
                    assert!(!seen[index]);seen[index]=true;
                }
                assert!(seen.into_iter().all(|v|v));
                c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
                let tight=Limits {max_outputs:(count-1) as u64,..limits};let mut c=Controller::new(doc.clone(),tight).unwrap();
                c.submit("Point").unwrap();c.submit("100,100,100").unwrap();c.undo().unwrap();c.select_objects(&[id]).unwrap();let before=c.coordinator().snapshot();
                assert!(c.submit(command).is_err());assert_eq!(c.coordinator().snapshot(),before);assert_eq!(c.coordinator().redo_len(),1);
            }
        }
    }
}

#[test]
fn large_curved_trim_boundary_commands_preserve_source_and_history() {
    use om9_core::{budget::memory::MemoryClass,evaluation::{evaluate_curve,Parameter},geometry::Transform,
        rhino_semantics::{set_trimmed_surface,TrimmedSurface},surface_kernel};
    let limits=Limits::default();
    let surface=surface_kernel::cylinder([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,5.,&limits).unwrap();
    let mut outer=Vec::new();
    for (side,count) in [177,177,177,177].into_iter().enumerate() {
        for i in 0..count {let t=(i as f64)/(count as f64);outer.push(match side {
            0=>[0.1*(1.-t)+0.3*t,0.2],1=>[0.3,0.2*(1.-t)+0.8*t],
            2=>[0.3*(1.-t)+0.1*t,0.8],_=>[0.1,0.8*(1.-t)+0.2*t],
        });}
    }
    assert_eq!(outer.len(),708);
    let trim=TrimmedSurface {underlying:surface.clone(),outer:outer.clone(),inners:vec![],reversed:false};
    let mut entity=Entity::new("large cylindrical trim",Geometry::Surface(surface.clone()),None);
    set_trimmed_surface(&mut entity,&trim,&limits).unwrap();
    let placement=Transform([[1.5,0.2,0.,3.],[0.,1.,0.,-2.],[0.,0.,2.,1.],[0.,0.,0.,1.]]);entity.placement=placement;let id=entity.id;
    let mut doc=Document::new("large curved boundary");doc.entities.insert(id,entity);let seed=Controller::new(doc,limits).unwrap();
    let expected:Vec<_>=(0..outer.len()).map(|i|[outer[i],outer[(i+1)%outer.len()]].map(|uv|
        placement.point(surface_kernel::evaluate(&surface,uv[0],uv[1],&limits).unwrap()).unwrap())).collect();
    for (command,label) in [("ExtractWireframe","ExtractWire"),("DupBorder","DupBorder")] {
        let mut c=reopen_controller(&seed,limits);c.submit("Point").unwrap();c.submit("100,100,100").unwrap();c.undo().unwrap();
        c.select_objects(&[id]).unwrap();let before=c.document().clone();
        let pressure=c.coordinator().reserve_remaining_external_memory_with_held(MemoryClass::Kernel,None,c.scene_cache_usage().unwrap()).unwrap();
        assert!(c.submit(command).is_err());assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),1);drop(pressure);
        c.submit(command).unwrap();let outputs:Vec<_>=c.document().entities.values().filter(|e|e.label==label).collect();assert_eq!(outputs.len(),708);
        let mut seen=vec![false;expected.len()];
        for e in outputs {let Geometry::Curve(curve)=&e.geometry else {panic!("curve")};
            let points=[0.,1.].map(|t|evaluate_curve(curve,Parameter::Fraction(t),0,e.placement,&limits).unwrap()[0]);
            let near=|a:[f64;3],b:[f64;3]|(0..3).all(|i|(a[i]-b[i]).abs()<1e-7);
            let index=expected.iter().position(|pair|near(points[0],pair[0])&&near(points[1],pair[1]) || near(points[0],pair[1])&&near(points[1],pair[0])).expect("unexpected curved edge");
            assert!(!seen[index]);seen[index]=true;
            let a=outer[index];let b=outer[(index+1)%outer.len()];let uv=[(a[0]+b[0])*0.5,(a[1]+b[1])*0.5];
            let expected_mid=placement.point(surface_kernel::evaluate(&surface,uv[0],uv[1],&limits).unwrap()).unwrap();
            let mid=evaluate_curve(curve,Parameter::Fraction(0.5),0,e.placement,&limits).unwrap()[0];
            assert!(near(mid,expected_mid));let y=mid[1]+2.;let x=(mid[0]-3.-0.2*y)/1.5;
            assert!((x.hypot(y)-2.).abs()<1e-7);
        }
        assert!(seen.into_iter().all(|value|value));let after=c.document().clone();
        assert_eq!(reopen_controller(&c,limits).document().entities,after.entities);
        c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }
}

#[test]
fn trimmed_rational_diagonal_with_extreme_weights_is_refused() {
    use om9_core::rhino_semantics::{set_trimmed_surface, TrimmedSurface};
    let limits = Limits::default();
    for warp in [true, false] {
        let mut surface = om9_core::surface_kernel::plane([0.,0.,0.], [1.,0.,0.], [0.,1.,0.], 10., 10., &limits).unwrap();
        if warp { surface.poles[3][2] = 1e-9; }
        surface.weights[3] = 1e-20;
        let trim = TrimmedSurface { underlying: surface.clone(), outer: vec![[0.2,0.2],[0.8,0.2],[0.75,0.8],[0.2,0.8]],
            inners: vec![], reversed: false };
        let mut entity = Entity::new("nonaffine trim", Geometry::Surface(surface), None);
        set_trimmed_surface(&mut entity, &trim, &limits).unwrap();
        let id = entity.id;
        let mut doc = Document::new("nonaffine trimmed surface"); doc.entities.insert(id, entity);
        let seed = Controller::new(doc, limits).unwrap();
        for command in ["ExtractWireframe", "DupBorder"] {
            let mut c = reopen_controller(&seed, limits);
            c.submit("Point").unwrap(); c.submit("1,2,3").unwrap(); c.undo().unwrap(); c.select_objects(&[id]).unwrap();
            let before = c.document().clone();
            assert!(c.submit(command).is_err());
            assert_eq!(c.document(), &before);
            assert_eq!(c.coordinator().redo_len(), 1);
        }
    }
}

#[test]
fn dupborder_refuses_diagonal_curved_trim_boundaries() {
    use om9_core::rhino_semantics::{set_trimmed_surface, TrimmedSurface};
    let limits = Limits::default();
    let surface = om9_core::surface_kernel::cylinder([0.,0.,0.], [1.,0.,0.], [0.,1.,0.], 2., 5., &limits).unwrap();
    let trim = TrimmedSurface { underlying: surface.clone(), outer: vec![[0.2,0.2],[0.8,0.2],[0.8,0.8],[0.2,0.8]],
        inners: vec![vec![[0.4,0.4],[0.6,0.4],[0.5,0.6]]], reversed: false };
    let mut entity = Entity::new("trimmed", Geometry::Surface(surface), None);
    set_trimmed_surface(&mut entity, &trim, &limits).unwrap();
    let id = entity.id;
    let mut doc = Document::new("trimmed surface curves"); doc.entities.insert(id, entity);
    let seed = Controller::new(doc, limits).unwrap();
    for command in ["DupBorder"] {
        let mut c = reopen_controller(&seed, limits);
        c.submit("Point").unwrap(); c.submit("1,2,3").unwrap(); c.undo().unwrap();
        c.select_objects(&[id]).unwrap();
        let before = c.document().clone();
        assert!(c.submit(command).is_err(), "{command} must not use the untrimmed domain");
        assert_eq!(c.document(), &before);
        assert_eq!(c.coordinator().redo_len(), 1);
    }
}

#[test]
fn dupborder_trimmed_plane_preserves_loops_and_undo() {
    use om9_core::evaluation::{evaluate_curve, Parameter};
    use om9_core::geometry::{Curve, Transform};
    use om9_core::rhino_semantics::{set_trimmed_surface, TrimmedSurface};
    let limits = Limits::default();
    let surface = om9_core::surface_kernel::plane([0.,0.,0.], [1.,0.,0.], [0.,1.,0.], 10., 10., &limits).unwrap();
    let outer = vec![[0.2,0.2],[0.8,0.2],[0.8,0.8],[0.2,0.8]];
    let inner = vec![[0.4,0.4],[0.6,0.4],[0.5,0.6]];
    let trim = TrimmedSurface { underlying: surface.clone(), outer: outer.clone(), inners: vec![inner.clone()], reversed: false };
    let mut entity = Entity::new("trimmed", Geometry::Surface(surface), None);
    set_trimmed_surface(&mut entity, &trim, &limits).unwrap();
    entity.placement = Transform::translation([20.,30.,4.]); let id = entity.id;
    let mut doc = Document::new("trimmed plane border"); doc.entities.insert(id, entity);
    let seed = Controller::new(doc, limits).unwrap();
    let expected: Vec<_> = [outer, inner].iter().flat_map(|loop_| (0..loop_.len()).map(move |i| {
        [loop_[i], loop_[(i+1)%loop_.len()]].map(|p| [20.+10.*p[0], 30.+10.*p[1], 4.])
    })).collect();
    for menu in [false, true] {
        let mut c = reopen_controller(&seed, limits);
        c.submit("Point").unwrap(); c.submit("1,2,3").unwrap(); c.undo().unwrap(); c.select_objects(&[id]).unwrap();
        let before = c.document().clone();
        if !menu {
            let pressure = c.coordinator().reserve_remaining_external_memory_with_held(
                om9_core::budget::memory::MemoryClass::Kernel, None, c.scene_cache_usage().unwrap()).unwrap();
            assert!(c.submit("DupBorder").is_err()); assert_eq!(c.document(), &before); assert_eq!(c.coordinator().redo_len(), 1);
            drop(pressure);
        }
        if menu { c.dispatch("OM9-CURVE-039", om9_app::session::InvocationOrigin::Menu).unwrap(); }
        else { c.submit("DupBorder").unwrap(); }
        let outputs: Vec<_> = c.document().entities.values().filter(|e| e.label == "DupBorder").collect();
        assert_eq!(outputs.len(), 7); let mut seen = [false;7];
        for entity in outputs {
            let Geometry::Curve(curve @ Curve::Line { .. }) = &entity.geometry else { panic!("trim edge") };
            let ends = [0.,1.].map(|t| evaluate_curve(curve, Parameter::Fraction(t), 0, entity.placement, &limits).unwrap()[0]);
            let near = |a: [f64;3], b: [f64;3]| (0..3).all(|i| (a[i]-b[i]).abs() < 1e-9);
            let index = expected.iter().position(|pair| (near(ends[0],pair[0]) && near(ends[1],pair[1])) || (near(ends[0],pair[1]) && near(ends[1],pair[0]))).unwrap();
            assert!(!seen[index]); seen[index] = true;
        }
        assert!(seen.into_iter().all(|v| v));
        let after = c.document().clone(); assert_eq!(reopen_controller(&c, limits).document().entities, after.entities);
        c.undo().unwrap(); assert_eq!(c.document(), &before); c.redo().unwrap(); assert_eq!(c.document(), &after);
    }
}

#[test]
fn extract_wireframe_mixed_reopened_sources_are_one_transaction() {
    use om9_core::evaluation::{evaluate_curve, Parameter};
    use om9_core::geometry::{Curve, Mesh, Transform};
    let limits = Limits::default();
    let mut mesh = Entity::new("mesh", Geometry::Mesh(Mesh {
        vertices: vec![[0.,0.,0.], [2.,0.,0.], [0.,2.,0.]], faces: vec![vec![0,1,2]],
        normals: vec![], colors: vec![], corner_uvs: vec![],
    }), None);
    mesh.placement = Transform::translation([20., 0., 0.]);
    let mut surface = Entity::new("surface", Geometry::Surface(om9_core::surface_kernel::plane(
        [0.,0.,0.], [1.,0.,0.], [0.,1.,0.], 4., 3., &limits).unwrap()), None);
    surface.placement = Transform::translation([40., 0., 0.]);
    let brep = Entity::new("box", Geometry::Brep(creation::primitive(
        creation::Primitive::Box { x_mm: 4., y_mm: 5., z_mm: 6. }, 0.1, &limits).unwrap()), None);
    let ids = [mesh.id, surface.id, brep.id];
    let mut doc = Document::new("mixed sources");
    for entity in [mesh, surface, brep] { doc.entities.insert(entity.id, entity); }
    let seed = Controller::new(doc, limits).unwrap();
    for menu in [false, true] {
    let mut c = reopen_controller(&seed, limits);
    c.select_objects(&ids).unwrap();
    let before = c.document().clone();
    if menu { c.dispatch("OM9-CURVE-054", om9_app::session::InvocationOrigin::Menu).unwrap(); }
    else { c.submit("ExtractWireframe").unwrap(); }
    let outputs: Vec<_> = c.document().entities.values().filter(|e| e.label == "ExtractWire").collect();
    assert_eq!(outputs.len(), 21);
    assert_eq!(outputs.iter().filter(|e| matches!(e.geometry, Geometry::Curve(Curve::Line { .. })) && e.placement == Transform::translation([20.,0.,0.])).count(), 3);
    assert_eq!(outputs.iter().filter(|e| e.placement == Transform::translation([40.,0.,0.])).count(), 6);
    let mesh_midpoints = [[21.,0.,0.], [21.,1.,0.], [20.,1.,0.]];
    let surface_midpoints = [[42.,0.,0.], [42.,3.,0.], [40.,1.5,0.], [44.,1.5,0.], [42.,1.5,0.], [42.,1.5,0.]];
    let mut mesh_seen = [false; 3];
    let mut surface_seen = [false; 6];
    let mut box_seen: Vec<[f64; 3]> = Vec::new();
    for entity in outputs {
        let Geometry::Curve(curve) = &entity.geometry else { panic!("curve") };
        let p = evaluate_curve(curve, Parameter::Fraction(0.5), 0, entity.placement, &limits).unwrap()[0];
        let (expected, seen) = if entity.placement == Transform::translation([20.,0.,0.]) { (&mesh_midpoints[..], &mut mesh_seen[..]) }
            else if entity.placement == Transform::translation([40.,0.,0.]) { (&surface_midpoints[..], &mut surface_seen[..]) }
            else {
                let x = (p[0] - 2.).abs() < 1e-7 && (p[1].abs() < 1e-7 || (p[1] - 5.).abs() < 1e-7) && (p[2].abs() < 1e-7 || (p[2] - 6.).abs() < 1e-7);
                let y = (p[1] - 2.5).abs() < 1e-7 && (p[0].abs() < 1e-7 || (p[0] - 4.).abs() < 1e-7) && (p[2].abs() < 1e-7 || (p[2] - 6.).abs() < 1e-7);
                let z = (p[2] - 3.).abs() < 1e-7 && (p[0].abs() < 1e-7 || (p[0] - 4.).abs() < 1e-7) && (p[1].abs() < 1e-7 || (p[1] - 5.).abs() < 1e-7);
                assert!(x || y || z, "unexpected box midpoint {p:?}");
                assert!(!box_seen.iter().any(|q| (0..3).all(|i| (p[i] - q[i]).abs() < 1e-7)), "duplicate box edge {p:?}");
                box_seen.push(p);
                continue;
            };
        let index = expected.iter().enumerate().position(|(i,q)| !seen[i] && (0..3).all(|i| (p[i] - q[i]).abs() < 1e-7)).expect("unexpected midpoint");
        assert!(!seen[index], "duplicate edge midpoint {p:?}");
        seen[index] = true;
    }
    let after = c.document().clone();
    assert_eq!(reopen_controller(&c, limits).document().entities, after.entities);
    c.undo().unwrap(); assert_eq!(c.document(), &before);
    c.redo().unwrap(); assert_eq!(c.document(), &after);
    }
}

#[test]
fn extract_wireframe_mixed_affine_sources_keep_world_edges_and_measurement() {
    use om9_core::{evaluation::{evaluate_curve, Parameter}, geometry::{Mesh, Transform}};
    let limits = Limits::default();
    let box_transform = Transform([[2.,0.25,0.,3.],[0.,1.,0.,-2.],[0.,0.,0.5,1.],[0.,0.,0.,1.]]);
    let mesh_transform = Transform([[-2.,0.5,0.,20.],[0.,1.,0.,4.],[0.,0.,1.,2.],[0.,0.,0.,1.]]);
    let surface_transform = Transform([[1.,0.5,0.,40.],[0.,2.,0.,-3.],[0.,0.,1.,7.],[0.,0.,0.,1.]]);
    let mut box_entity = Entity::new("box", Geometry::Brep(creation::primitive(
        creation::Primitive::Box { x_mm:4.,y_mm:5.,z_mm:6. },0.1,&limits).unwrap()), None);
    box_entity.placement = box_transform;
    let mut mesh = Entity::new("mesh", Geometry::Mesh(Mesh { vertices:vec![[0.,0.,0.],[2.,0.,0.],[0.,2.,0.]],
        faces:vec![vec![0,1,2]], normals:vec![], colors:vec![], corner_uvs:vec![] }), None);
    mesh.placement = mesh_transform;
    let mut surface = Entity::new("surface", Geometry::Surface(om9_core::surface_kernel::plane(
        [0.,0.,0.],[1.,0.,0.],[0.,1.,0.],4.,3.,&limits).unwrap()), None);
    surface.placement = surface_transform;
    let ids = [box_entity.id, mesh.id, surface.id];
    let mut doc = Document::new("affine mixed wire");
    for entity in [box_entity,mesh,surface] { doc.entities.insert(entity.id,entity); }
    let mut c = reopen_controller(&Controller::new(doc,limits).unwrap(),limits);
    c.select_objects(&ids).unwrap(); let before = c.document().clone();
    c.submit("ExtractWireframe").unwrap();
    let mut expected:[Vec<[[f64;3];2]>;3]=Default::default();
    let corner = |mask:usize| [if mask&1==0 {0.} else {4.}, if mask&2==0 {0.} else {5.}, if mask&4==0 {0.} else {6.}];
    for axis in [1_usize,2,4] { for mask in 0..8 { if mask&axis==0 {
        expected[0].push([box_transform.point(corner(mask)).unwrap(),box_transform.point(corner(mask|axis)).unwrap()]);
    }}}
    let mesh_points=[[0.,0.,0.],[2.,0.,0.],[0.,2.,0.]];
    for (a,b) in [(0,1),(1,2),(2,0)] { expected[1].push([mesh_transform.point(mesh_points[a]).unwrap(),mesh_transform.point(mesh_points[b]).unwrap()]); }
    let surface_points=[[0.,0.,0.],[4.,0.,0.],[4.,3.,0.],[0.,3.,0.]];
    for (a,b) in [(0,1),(1,2),(2,3),(3,0)] { expected[2].push([surface_transform.point(surface_points[a]).unwrap(),surface_transform.point(surface_points[b]).unwrap()]); }
    for [a,b] in [[[2.,0.,0.],[2.,3.,0.]], [[0.,1.5,0.],[4.,1.5,0.]]] {
        expected[2].push([surface_transform.point(a).unwrap(),surface_transform.point(b).unwrap()]);
    }
    let mut seen:[Vec<bool>;3]=[vec![false;12],vec![false;3],vec![false;6]];
    let mut chosen:[Option<(om9_core::identity::ObjectId,f64)>;3]=[None,None,None];
    for entity in c.document().entities.values().filter(|e| e.label=="ExtractWire") {
        let group=if entity.placement==box_transform {0} else if entity.placement==mesh_transform {1}
            else if entity.placement==surface_transform {2} else {panic!("unexpected placement")};
        let Geometry::Curve(curve)=&entity.geometry else {panic!("curve")};
        let ends=[0.,1.].map(|t|evaluate_curve(curve,Parameter::Fraction(t),0,entity.placement,&limits).unwrap()[0]);
        let close=|a:[f64;3],b:[f64;3]| (0..3).all(|i|(a[i]-b[i]).abs()<1e-7);
        let index=expected[group].iter().position(|e| (close(ends[0],e[0])&&close(ends[1],e[1])) || (close(ends[0],e[1])&&close(ends[1],e[0]))).expect("unexpected world edge");
        assert!(!seen[group][index]); seen[group][index]=true;
        let length=(0..3).map(|i|(ends[0][i]-ends[1][i]).powi(2)).sum::<f64>().sqrt();
        chosen[group].get_or_insert((entity.id,length));
    }
    assert!(seen.iter().all(|group|group.iter().all(|value|*value)));
    let after = c.document().clone(); c.undo().unwrap(); assert_eq!(c.document(),&before);
    c.redo().unwrap(); assert_eq!(c.document(),&after);
    let mut saved = reopen_controller(&c,limits);
    for (id,length) in chosen.into_iter().map(Option::unwrap) {
        saved.select_objects(&[id]).unwrap();
        assert!((saved.measure_selected_length().unwrap().value-length).abs()<1e-6);
    }
}

#[test]
fn extract_wireframe_mixed_sources_refuse_late_invalid_edge_and_pressure_atomically() {
    use om9_core::{budget::memory::MemoryClass, geometry::Mesh};
    let limits = Limits::default();
    for invalid in [true, false] {
        let mut brep = Entity::new("box", Geometry::Brep(creation::primitive(
            creation::Primitive::Box { x_mm: 4., y_mm: 5., z_mm: 6. }, 0.1, &limits).unwrap()), None);
        let mut mesh = Entity::new("mesh", Geometry::Mesh(Mesh {
            vertices: vec![[0.,0.,0.], if invalid { [0.,0.,0.] } else { [2.,0.,0.] }, [0.,2.,0.]],
            faces: vec![vec![0,1,2]], normals: vec![], colors: vec![], corner_uvs: vec![],
        }), None);
        // BTree selection order must process the valid B-rep before the mesh.
        if mesh.id < brep.id { std::mem::swap(&mut mesh.id, &mut brep.id); }
        let ids = [brep.id, mesh.id];
        let mut doc = Document::new("mixed failure");
        for entity in [brep, mesh] { doc.entities.insert(entity.id, entity); }
        let mut c = reopen_controller(&Controller::new(doc, limits).unwrap(), limits);
        c.submit("Point").unwrap(); c.submit("7,8,9").unwrap(); c.undo().unwrap();
        c.select_objects(&ids).unwrap();
        let before = c.coordinator().snapshot();
        if invalid {
            assert!(c.submit("ExtractWireframe").is_err());
        } else {
            let pressure = c.coordinator().reserve_remaining_external_memory_with_held(
                MemoryClass::Kernel, None, c.scene_cache_usage().unwrap()).unwrap();
            assert!(c.submit("ExtractWireframe").is_err());
            assert_eq!(c.coordinator().snapshot(), before); assert_eq!(c.coordinator().redo_len(), 1);
            drop(pressure);
            c.submit("ExtractWireframe").unwrap();
            assert_eq!(c.document().entities.values().filter(|e| e.label == "ExtractWire").count(), 15);
            c.undo().unwrap();
            assert_eq!(c.document().entities, before.world.documents.values().next().unwrap().entities);
            continue;
        }
        assert_eq!(c.coordinator().snapshot(), before); assert_eq!(c.coordinator().redo_len(), 1);
    }
}

#[test]
fn extract_wireframe_reopened_mesh_uses_face_edges_and_one_transaction() {
    use om9_core::{budget::memory::MemoryClass, geometry::{Curve, Mesh, Transform}};
    let limits = Limits::default();
    let points = vec![[0.,0.,0.], [2.,0.,0.], [2.,2.,0.], [0.,2.,0.], [4.,1.,0.]];
    let mesh = Mesh { vertices: points.clone(), faces: vec![vec![0,1,2,3], vec![1,4,2]], normals: vec![], colors: vec![], corner_uvs: vec![] };
    let mut entity = Entity::new("mesh", Geometry::Mesh(mesh), None);
    let placement = Transform::translation([10., 20., 30.]);
    entity.placement = placement;
    let id = entity.id;
    let mut doc = Document::new("mesh wire"); doc.entities.insert(id, entity);
    let seed = Controller::new(doc, limits).unwrap();
    let mut c = reopen_controller(&seed, limits);
    c.submit("Point").unwrap(); c.submit("7,8,9").unwrap(); c.undo().unwrap();
    c.select_objects(&[id]).unwrap();
    let before = c.coordinator().snapshot();
    let before_document = c.document().clone();
    let pressure = c.coordinator().reserve_remaining_external_memory_with_held(MemoryClass::Kernel, None, c.scene_cache_usage().unwrap()).unwrap();
    assert!(c.submit("ExtractWireframe").is_err());
    assert_eq!(c.coordinator().snapshot(), before); assert_eq!(c.coordinator().redo_len(), 1);
    drop(pressure);
    c.submit("ExtractWireframe").unwrap();
    let curves: Vec<_> = c.document().entities.values().filter(|e| e.label == "ExtractWire").collect();
    assert_eq!(curves.len(), 6);
    let mut edges = std::collections::BTreeSet::new();
    for entity in curves {
        assert_eq!(entity.placement, placement);
        let Geometry::Curve(Curve::Line { start, end, .. }) = &entity.geometry else { panic!("line") };
        let a = points.iter().position(|p| p == start).unwrap();
        let b = points.iter().position(|p| p == end).unwrap();
        assert!(edges.insert((a.min(b), a.max(b))));
    }
    assert_eq!(edges, [(0,1),(1,2),(2,3),(0,3),(1,4),(2,4)].into_iter().collect());
    let after = c.document().clone();
    let reopened = reopen_controller(&c, limits); assert_eq!(reopened.document().entities, after.entities);
    c.undo().unwrap(); assert_eq!(c.document(), &before_document);
    c.redo().unwrap(); assert_eq!(c.document(), &after);
}

#[test]
fn extract_wireframe_mesh_zero_length_edge_refuses_without_losing_redo() {
    use om9_core::geometry::Mesh;
    let limits = Limits::default();
    let mesh = Mesh { vertices: vec![[0.,0.,0.], [0.,0.,0.], [2.,0.,0.], [0.,2.,0.]],
        faces: vec![vec![0,1,2,3]], normals: vec![], colors: vec![], corner_uvs: vec![] };
    let entity = Entity::new("degenerate mesh", Geometry::Mesh(mesh), None); let id = entity.id;
    let mut doc = Document::new("invalid edge"); doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, limits).unwrap();
    c.submit("Point").unwrap(); c.submit("7,8,9").unwrap(); c.undo().unwrap();
    c.select_objects(&[id]).unwrap(); let before = c.coordinator().snapshot();
    assert!(c.submit("ExtractWireframe").is_err());
    assert_eq!(c.coordinator().snapshot(), before); assert_eq!(c.coordinator().redo_len(), 1);
}

#[test]
fn extract_wireframe_reopened_surface_display_boundaries_are_exact_and_undoable() {
    use om9_core::evaluation::{evaluate_curve, Parameter};
    use om9_core::geometry::Transform;
    use om9_scene::scene::RenderBuffers;
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],4.,3.,&limits).unwrap();
    let entity=Entity::new("surface",Geometry::Surface(surface),None); let id=entity.id;
    let mut doc=Document::new("surface wireframe"); doc.entities.insert(id,entity);
    let original=Controller::new(doc,limits).unwrap(); let mut c=reopen_controller(&original,limits);
    c.submit("Point").unwrap(); c.submit("1,2,3").unwrap(); c.undo().unwrap();
    c.select_objects(&[id]).unwrap();
    let object=c.scene().unwrap().objects.iter().find(|o|o.root==id).unwrap();
    let RenderBuffers::Mesh {vertices,edges,..}=object.buffers.as_ref() else {panic!("surface display")};
    let mut sides=[false;6]; assert!(!edges.is_empty());
    for [a,b] in edges {
        let p=vertices[*a as usize]; let q=vertices[*b as usize];
        let side=if p[0].abs()<1e-9 && q[0].abs()<1e-9 {0}
            else if (p[0]-4.).abs()<1e-9 && (q[0]-4.).abs()<1e-9 {1}
            else if p[1].abs()<1e-9 && q[1].abs()<1e-9 {2}
            else if (p[1]-3.).abs()<1e-9 && (q[1]-3.).abs()<1e-9 {3}
            else if (p[0]-2.).abs()<1e-9 && (q[0]-2.).abs()<1e-9 {4}
            else if (p[1]-1.5).abs()<1e-9 && (q[1]-1.5).abs()<1e-9 {5}
            else {panic!("unexpected interior display edge {p:?} {q:?}")};
        sides[side]=true;
    }
    assert!(sides.into_iter().all(|s|s));
    let before=c.document().clone(); c.submit("ExtractWireframe").unwrap();
    let curves: Vec<_>=c.document().entities.values().filter(|e|e.label=="ExtractWire").collect();
    assert_eq!(curves.len(),6);
    let mut found=[false;6];
    for entity in curves {
        let Geometry::Curve(curve)=&entity.geometry else {panic!("curve")};
        let points: Vec<_>=[0.,0.25,0.5,0.75,1.].into_iter().map(|t|
            evaluate_curve(curve,Parameter::Fraction(t),0,Transform::identity(),&limits).unwrap()[0]).collect();
        let side=if points.iter().all(|p|p[0].abs()<1e-9) {0}
            else if points.iter().all(|p|(p[0]-4.).abs()<1e-9) {1}
            else if points.iter().all(|p|p[1].abs()<1e-9) {2}
            else if points.iter().all(|p|(p[1]-3.).abs()<1e-9) {3}
            else if points.iter().all(|p|(p[0]-2.).abs()<1e-9) {4}
            else if points.iter().all(|p|(p[1]-1.5).abs()<1e-9) {5}
            else {panic!("not a source boundary: {points:?}")};
        assert!(!found[side]); found[side]=true;
    }
    assert!(found.into_iter().all(|v|v)); assert_eq!(c.coordinator().redo_len(),0);
    let after=c.document().clone(); let reopened=reopen_controller(&c,limits);
    assert_eq!(reopened.document().entities, after.entities);
    c.undo().unwrap(); assert_eq!(c.document(),&before);
    c.redo().unwrap(); assert_eq!(c.document(),&after);
}

#[test]
fn surface_knot_wires_match_display_and_exact_extraction_after_reopen() {
    use om9_core::{evaluation::{evaluate_curve, Parameter}, geometry::{Curve, Transform}, surface_kernel::{self, SurfaceIso}};
    use om9_scene::scene::RenderBuffers;
    let limits = Limits::default();
    let surface = surface_kernel::cylinder([0.,0.,0.], [1.,0.,0.], [0.,1.,0.], 2., 5., &limits).unwrap();
    let mut entity = Entity::new("cylinder", Geometry::Surface(surface.clone()), None);
    entity.placement = Transform::translation([10.,20.,30.]);
    let placement = entity.placement; let id = entity.id;
    let mut doc = Document::new("surface knot wires"); doc.entities.insert(id, entity);
    let seed = Controller::new(doc, limits).unwrap();
    for menu in [false, true] {
        let mut c = reopen_controller(&seed, limits);
        c.select_objects(&[id]).unwrap(); let before = c.document().clone();
        let obj = c.scene().unwrap().objects.iter().find(|o| o.root == id).unwrap();
        let RenderBuffers::Mesh { vertices, edges, .. } = obj.buffers.as_ref() else { panic!("surface display") };
        assert_eq!(edges.len(), 88);
        let displayed = vertices[144..].to_vec();
        if menu { c.dispatch("OM9-CURVE-054", om9_app::session::InvocationOrigin::Menu).unwrap(); }
        else { c.submit("ExtractWireframe").unwrap(); }
        let outputs: Vec<_> = c.document().entities.values().filter(|e| e.label == "ExtractWire").collect();
        assert_eq!(outputs.len(), 7);
        for (wire, (axis, fixed)) in [(0,0.25), (0,0.5), (0,0.75), (1,0.5)].into_iter().enumerate() {
            let iso = if axis == 0 { SurfaceIso::V } else { SurfaceIso::U };
            let exact = surface_kernel::isocurve(&surface, iso, fixed, &limits).unwrap();
            let entity = outputs.iter().find(|e| matches!(&e.geometry, Geometry::Curve(Curve::Nurbs(n)) if *n == exact)).expect("interior curve");
            assert_eq!(entity.placement, placement);
            let Geometry::Curve(curve) = &entity.geometry else { unreachable!() };
            for i in 0..12 {
                let t = i as f64 / 11.;
                let (u,v) = if axis == 0 { (fixed,t) } else { (t,fixed) };
                let source = surface_kernel::evaluate(&surface, u, v, &limits).unwrap();
                let curve_point = evaluate_curve(curve, Parameter::Fraction(t), 0, Transform::identity(), &limits).unwrap()[0];
                for k in 0..3 {
                    assert!((source[k] - displayed[wire * 12 + i][k]).abs() < 1e-9);
                    assert!((source[k] - curve_point[k]).abs() < 1e-9);
                }
            }
        }
        let after = c.document().clone();
        assert_eq!(reopen_controller(&c, limits).document().entities, after.entities);
        c.undo().unwrap(); assert_eq!(c.document(), &before);
        c.redo().unwrap(); assert_eq!(c.document(), &after);
    }
}

#[test]
fn extract_wireframe_open_curved_surface_keeps_placement_and_undo() {
    use om9_core::{evaluation::{evaluate_curve, Parameter}, geometry::Transform};
    let limits = Limits::default();
    let mut surface = om9_core::surface_kernel::cylinder([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,5.,&limits).unwrap();
    surface.domain[0] = [0., 0.5];
    let placement = Transform::translation([10., -2., 3.]);
    let mut entity = Entity::new("half cylinder", Geometry::Surface(surface), None);
    entity.placement = placement; let id = entity.id;
    let mut doc = Document::new("open curved surface"); doc.entities.insert(id, entity);
    let seed = Controller::new(doc, limits).unwrap(); let mut c = reopen_controller(&seed, limits);
    c.select_objects(&[id]).unwrap(); let before = c.document().clone();
    c.submit("ExtractWireframe").unwrap();
    let curves: Vec<_> = c.document().entities.values().filter(|e|e.label=="ExtractWire").collect();
    let Geometry::Surface(surface) = &c.document().entities[&id].geometry else { panic!("surface") };
    let curves = without_interior_wires(curves, surface, &limits);
    assert_eq!(curves.len(), 4);
    let corners = [[12., -2., 3.], [8., -2., 3.], [12., -2., 8.], [8., -2., 8.]];
    let mut edges = std::collections::BTreeSet::new();
    for entity in curves {
        assert_eq!(entity.placement, placement);
        let Geometry::Curve(curve) = &entity.geometry else { panic!("curve") };
        let point = |t| evaluate_curve(curve, Parameter::Fraction(t), 0, entity.placement, &limits).unwrap()[0];
        let corner = |p: [f64; 3]| corners.iter().position(|c| (0..3).all(|i| (p[i]-c[i]).abs()<1e-7)).expect("boundary endpoint must be a surface corner");
        let (a, b) = (corner(point(0.)), corner(point(1.)));
        let edge = (a.min(b), a.max(b));
        assert!([(0, 1), (2, 3), (0, 2), (1, 3)].contains(&edge), "unexpected edge {edge:?}");
        assert!(edges.insert(edge), "duplicate edge {edge:?}");
        for t in [0., 0.25, 0.5, 0.75, 1.] {
            let p = point(t);
            assert!(((p[0]-10.).hypot(p[1]+2.)-2.).abs()<1e-7 && p[2]>=3.-1e-8 && p[2]<=8.+1e-8, "{p:?}");
            if edge.0 / 2 == edge.1 / 2 {
                assert!((p[2] - corners[edge.0][2]).abs() < 1e-7 && p[1] >= -2. - 1e-7, "arc point {p:?}");
            } else {
                assert!((p[0] - corners[edge.0][0]).abs() < 1e-7 && (p[1] + 2.).abs() < 1e-7, "line point {p:?}");
            }
        }
    }
    assert_eq!(edges.len(), 4);
    let after = c.document().clone(); c.undo().unwrap(); assert_eq!(c.document(), &before);
    c.redo().unwrap(); assert_eq!(c.document(), &after);
}

#[test]
fn extract_wireframe_closed_cylinder_keeps_one_displayed_seam() {
    use om9_core::{evaluation::{evaluate_curve, Parameter}, geometry::Transform};
    let limits = Limits::default();
    for menu in [false, true] {
        let surface = om9_core::surface_kernel::cylinder([0.,0.,0.], [1.,0.,0.], [0.,1.,0.], 2., 5., &limits).unwrap();
        let mut entity = Entity::new("closed cylinder", Geometry::Surface(surface), None);
        entity.placement = Transform::translation([10., -2., 3.]);
        let id = entity.id;
        let mut document = Document::new("closed cylinder wire"); document.entities.insert(id, entity);
        let mut c = reopen_controller(&Controller::new(document, limits).unwrap(), limits);
        c.select_objects(&[id]).unwrap(); let before = c.document().clone();
        if menu { c.dispatch("OM9-CURVE-054", om9_app::session::InvocationOrigin::Menu).unwrap(); }
        else { c.submit("ExtractWireframe").unwrap(); }
        let curves: Vec<_> = c.document().entities.values().filter(|e| e.label == "ExtractWire").collect();
        let Geometry::Surface(surface) = &c.document().entities[&id].geometry else { panic!("surface") };
        let curves = without_interior_wires(curves, surface, &limits);
        assert_eq!(curves.len(), 3);
        let mut circular = [false; 2]; let mut seam = 0;
        for entity in curves {
            let Geometry::Curve(curve) = &entity.geometry else { panic!("curve") };
            let points: Vec<_> = [0., 0.25, 0.5, 0.75, 1.].into_iter().map(|t|
                evaluate_curve(curve, Parameter::Fraction(t), 0, entity.placement, &limits).unwrap()[0]).collect();
            assert!(points.iter().all(|p| ((p[0]-10.).hypot(p[1]+2.)-2.).abs() < 1e-7));
            if points.iter().all(|p| (p[2]-3.).abs() < 1e-7) { assert!(!circular[0]); circular[0] = true; }
            else if points.iter().all(|p| (p[2]-8.).abs() < 1e-7) { assert!(!circular[1]); circular[1] = true; }
            else {
                assert!(points.iter().all(|p| (p[0]-points[0][0]).abs() < 1e-7 && (p[1]-points[0][1]).abs() < 1e-7));
                assert!((points[0][2]-3.).abs() < 1e-7 && (points[4][2]-8.).abs() < 1e-7);
                seam += 1;
            }
        }
        assert_eq!(circular, [true, true]); assert_eq!(seam, 1);
        let after = c.document().clone(); c.undo().unwrap(); assert_eq!(c.document(), &before);
        c.redo().unwrap(); assert_eq!(c.document(), &after);
    }
}

#[test]
fn extract_wireframe_certified_periodic_surface_keeps_one_seam() {
    use om9_core::{evaluation::{evaluate_curve, Parameter}, geometry::{NurbsSurface, Transform}};
    let limits = Limits::default();
    for certified in [true, false] {
        let rows = [[0.,0.], [2.,0.], [2.,2.], [0.,2.], [if certified {0.} else {0.1},0.], [2.,0.]];
        let surface = NurbsSurface { degree:[2,1], size:[6,2],
            poles: rows.iter().flat_map(|p| [[p[0],p[1],0.], [p[0],p[1],5.]]).collect(),
            weights: vec![1.;12], knots:[vec![-2.,-1.,0.,1.,2.,3.,4.,5.,6.], vec![0.,0.,1.,1.]],
            domain:[[0.,4.],[0.,1.]], periodic:[true,false], reversed:false };
        let mut entity = Entity::new("periodic", Geometry::Surface(surface.clone()), None);
        entity.placement = Transform::translation([10.,20.,30.]); let id = entity.id;
        let mut doc = Document::new("periodic wire"); doc.entities.insert(id, entity);
        let mut c = reopen_controller(&Controller::new(doc, limits).unwrap(), limits);
        c.submit("Point").unwrap(); c.submit("1,2,3").unwrap(); c.undo().unwrap();
        c.select_objects(&[id]).unwrap(); let snapshot = c.coordinator().snapshot(); let before = c.document().clone();
        if !certified {
            assert!(c.submit("ExtractWireframe").is_err());
            assert_eq!(c.coordinator().snapshot(), snapshot); assert_eq!(c.coordinator().redo_len(), 1);
            continue;
        }
        c.submit("ExtractWireframe").unwrap();
        let curves: Vec<_> = c.document().entities.values().filter(|e| e.label == "ExtractWire").collect();
        let curves = without_interior_wires(curves, &surface, &limits);
        assert_eq!(curves.len(), 3);
        let mut seen = [false; 3];
        for entity in curves {
            let Geometry::Curve(curve) = &entity.geometry else { panic!("curve") };
            let point = |t| evaluate_curve(curve, Parameter::Fraction(t), 0, entity.placement, &limits).unwrap()[0];
            let first = point(0.); let last = point(1.);
            let index = if (first[2]-30.).abs() < 1e-8 && (last[2]-30.).abs() < 1e-8 { 0 }
                else if (first[2]-35.).abs() < 1e-8 && (last[2]-35.).abs() < 1e-8 { 1 } else { 2 };
            assert!(!seen[index]); seen[index] = true;
            for t in [0.,0.13,0.37,0.61,0.91,1.] {
                let (u, v) = match index { 0 => (4.*t, 0.), 1 => (4.*t, 1.), _ => (0., t) };
                let expected = entity.placement.point(om9_core::surface_kernel::evaluate(&surface, u, v, &limits).unwrap()).unwrap();
                let actual = point(t);
                assert!((0..3).all(|i| (actual[i]-expected[i]).abs() < 1e-8));
            }
        }
        assert_eq!(seen, [true; 3]);
        let after = c.document().clone(); c.undo().unwrap(); assert_eq!(c.document(), &before);
        c.redo().unwrap(); assert_eq!(c.document(), &after);
    }
}

#[test]
fn extract_wireframe_conical_surface_omits_collapsed_apex_curve() {
    use om9_core::{evaluation::{evaluate_curve, Parameter}, geometry::Transform};
    let limits = Limits::default();
    let mut surface = om9_core::surface_kernel::cylinder([0.,0.,0.], [1.,0.,0.], [0.,1.,0.], 2., 5., &limits).unwrap();
    for u in 0..surface.size[0] { surface.poles[u * surface.size[1] + 1] = [0.,0.,5.]; }
    let mut entity = Entity::new("conical surface", Geometry::Surface(surface.clone()), None);
    entity.placement = Transform::translation([10.,20.,30.]); let id = entity.id;
    let mut doc = Document::new("conical wire"); doc.entities.insert(id, entity);
    let seed = Controller::new(doc, limits).unwrap();
    for menu in [false, true] {
    let mut c = reopen_controller(&seed, limits);
    c.select_objects(&[id]).unwrap(); let before = c.document().clone();
    if menu { c.dispatch("OM9-CURVE-054", om9_app::session::InvocationOrigin::Menu).unwrap(); }
    else { c.submit("ExtractWireframe").unwrap(); }
    let curves: Vec<_> = c.document().entities.values().filter(|e| e.label == "ExtractWire").collect();
    let curves = without_interior_wires(curves, &surface, &limits);
    assert_eq!(curves.len(), 2, "the apex is a point, not a curve");
    let mut seen = [false; 2];
    for entity in curves {
        let Geometry::Curve(curve) = &entity.geometry else { panic!("curve") };
        let points: Vec<_> = [0.,0.25,0.5,0.75,1.].into_iter().map(|t|
            evaluate_curve(curve, Parameter::Fraction(t), 0, entity.placement, &limits).unwrap()[0]).collect();
        assert!(points.iter().any(|p| (0..3).any(|i| (p[i]-points[0][i]).abs() > 1e-7)), "collapsed curve");
        let side = if points.iter().all(|p| (p[2]-30.).abs() < 1e-8) { 0 } else { 1 };
        assert!(!seen[side]); seen[side] = true;
        for (t, actual) in [0.,0.25,0.5,0.75,1.].into_iter().zip(points) {
            let (u,v) = if side == 0 { (t,0.) } else { (0.,t) };
            let expected = entity.placement.point(om9_core::surface_kernel::evaluate(&surface,u,v,&limits).unwrap()).unwrap();
            assert!((0..3).all(|i| (actual[i]-expected[i]).abs() < 1e-8));
        }
    }
    assert_eq!(seen, [true; 2]);
    let after = c.document().clone(); c.undo().unwrap(); assert_eq!(c.document(), &before);
    c.redo().unwrap(); assert_eq!(c.document(), &after);
    }
}

#[test]
fn extract_wireframe_mixed_conical_surfaces_keep_nonzero_boundary() {
    let limits = Limits::default();
    let original = om9_core::surface_kernel::cylinder([0.,0.,0.], [1.,0.,0.], [0.,1.,0.], 2., 5., &limits).unwrap();
    let mut collapsed = original.clone(); let mut small = original;
    for u in 0..collapsed.size[0] {
        collapsed.poles[u * collapsed.size[1] + 1] = [0.,0.,5.];
        let p = &mut small.poles[u * small.size[1] + 1]; p[0] *= 1e-5; p[1] *= 1e-5;
    }
    let a = Entity::new("cone", Geometry::Surface(collapsed), None);
    let mut b = Entity::new("small top", Geometry::Surface(small), None);
    b.placement = om9_core::geometry::Transform::translation([10.,0.,0.]);
    let (a_id,b_id) = (a.id,b.id);
    let mut doc = Document::new("mixed cones"); doc.entities.insert(a_id,a); doc.entities.insert(b_id,b);
    let seed = Controller::new(doc, limits).unwrap();
    for menu in [false,true] {
        let mut c = reopen_controller(&seed,limits);
        c.submit("Point").unwrap(); c.submit("1,2,3").unwrap(); c.undo().unwrap(); c.select_objects(&[a_id,b_id]).unwrap();
        let before = c.document().clone();
        let pressure = c.coordinator().reserve_remaining_external_memory_with_held(
            om9_core::budget::memory::MemoryClass::Kernel,None,c.scene_cache_usage().unwrap()).unwrap();
        assert!(c.submit("ExtractWireframe").is_err()); assert_eq!(c.document(),&before); assert_eq!(c.coordinator().redo_len(),1);
        drop(pressure);
        if menu { c.dispatch("OM9-CURVE-054",om9_app::session::InvocationOrigin::Menu).unwrap(); }
        else { c.submit("ExtractWireframe").unwrap(); }
        let outputs: Vec<_> = c.document().entities.values().filter(|e| e.label == "ExtractWire").collect();
        assert_eq!(outputs.len(),13);
        assert_eq!(outputs.iter().filter(|e| e.placement.0[0][3] == 10.).count(),7);
        let after = c.document().clone(); c.undo().unwrap(); assert_eq!(c.document(),&before);
        c.redo().unwrap(); assert_eq!(c.document(),&after);
    }
}

#[test]
fn extract_wireframe_doubly_periodic_seams_survive_reopen_and_extrusion() {
    use om9_core::{evaluation::{evaluate_curve, Parameter}, geometry::{NurbsSurface, Transform}};
    let limits = Limits::default();
    let circle = [[1.,0.], [0.,1.], [-1.,0.], [0.,-1.], [1.,0.], [0.,1.]];
    let weights = [1.,0.7,1.2,0.9,1.,0.7];
    let knots = vec![-2.,-1.,0.,1.,2.,3.,4.,5.,6.];
    let mut poles = Vec::new(); let mut ws = Vec::new();
    for (i, [x,y]) in circle.iter().copied().enumerate() {
        for (j, [radial,z]) in circle.iter().copied().enumerate() {
            poles.push([(3.+0.5*radial)*x, (3.+0.5*radial)*y, 0.5*z]); ws.push(weights[i]*weights[j]);
        }
    }
    let surface = NurbsSurface { degree:[2,2], size:[6,6], poles, weights:ws,
        knots:[knots.clone(),knots], domain:[[0.,4.],[0.,4.]], periodic:[true,true], reversed:false };
    surface.certify_repeated_periodic_basis(&limits).unwrap();
    let mut entity = Entity::new("torus", Geometry::Surface(surface.clone()), None);
    entity.placement = Transform::translation([10.,20.,30.]); let id = entity.id;
    let mut doc = Document::new("periodic torus wires"); doc.entities.insert(id, entity);
    let mut c = reopen_controller(&Controller::new(doc, limits).unwrap(), limits);
    c.select_objects(&[id]).unwrap(); let before = c.document().clone();
    c.submit("ExtractWireframe").unwrap();
    let outputs: Vec<_> = c.document().entities.values().filter(|e| e.label == "ExtractWire").collect();
    let outputs = without_interior_wires(outputs, &surface, &limits);
    assert_eq!(outputs.len(), 2);
    let mut seen = [false; 2]; let mut major_id = None;
    for entity in outputs {
        let Geometry::Curve(curve) = &entity.geometry else { panic!("curve") };
        let point = |t| evaluate_curve(curve, Parameter::Fraction(t), 0, entity.placement, &limits).unwrap()[0];
        let expected = entity.placement.point(om9_core::surface_kernel::evaluate(&surface, 4.*0.37, 0., &limits).unwrap()).unwrap();
        let p = point(0.37);
        let axis = if (0..3).all(|i| (p[i] - expected[i]).abs() < 1e-8) { 0 } else { 1 };
        assert!(!seen[axis]); seen[axis] = true;
        if axis == 0 { major_id = Some(entity.id); }
        for t in [0.,0.13,0.37,0.61,0.91,1.] {
            let (u,v) = if axis == 0 { (4.*t,0.) } else { (0.,4.*t) };
            let expected = entity.placement.point(om9_core::surface_kernel::evaluate(&surface, u, v, &limits).unwrap()).unwrap();
            let actual = point(t);
            assert!((0..3).all(|i| (actual[i]-expected[i]).abs()<1e-8));
        }
    }
    assert_eq!(seen, [true;2]);
    let after = c.document().clone(); c.undo().unwrap(); assert_eq!(c.document(), &before);
    c.redo().unwrap(); assert_eq!(c.document(), &after);
    let mut c = reopen_controller(&c, limits);
    let mut area2 = 0.; let mut previous = om9_core::surface_kernel::evaluate(&surface,0.,0.,&limits).unwrap();
    for i in 1..=4096 {
        let p = om9_core::surface_kernel::evaluate(&surface,4.*i as f64/4096.,0.,&limits).unwrap();
        area2 += previous[0]*p[1] - p[0]*previous[1]; previous = p;
    }
    let expected_volume = area2.abs()*0.5*3.;
    c.select_objects(&[major_id.unwrap()]).unwrap(); let before = c.document().entities.clone();
    c.submit("ExtrudeCrv 3").unwrap();
    let (body, placement) = c.document().entities.values().find_map(|e| match &e.geometry {
        Geometry::Brep(body) if !before.contains_key(&e.id) => Some((body,e.placement)), _ => None,
    }).expect("extrusion must create solid");
    let measure = om9_kernel::creation::measure(body,&limits).unwrap();
    assert_eq!(measure.solids, 1); assert!(measure.closed);
    assert!((measure.volume_mm3-expected_volume).abs() < expected_volume*1e-5, "{measure:?} vs {expected_volume}");
    let bounds = om9_kernel::brep::world_bounds(body,&placement,&limits).unwrap();
    let bottom = 30. + om9_core::surface_kernel::evaluate(&surface,0.,0.,&limits).unwrap()[2];
    assert!((bounds[2]-bottom).abs()<1e-5 && (bounds[5]-bottom-3.).abs()<1e-5, "{bounds:?}");
    let after = c.document().entities.clone(); c.undo().unwrap(); assert_eq!(c.document().entities,before);
    c.redo().unwrap(); assert_eq!(c.document().entities,after);
}

#[test]
fn extract_wireframe_closed_surface_and_mesh_are_atomic() {
    use om9_core::geometry::Mesh;
    let limits = Limits::default();
    let surface = Entity::new("closed cylinder", Geometry::Surface(om9_core::surface_kernel::cylinder(
        [0.,0.,0.], [1.,0.,0.], [0.,1.,0.], 2., 5., &limits).unwrap()), None);
    let mesh = Entity::new("triangle", Geometry::Mesh(Mesh { vertices: vec![[10.,0.,0.],[12.,0.,0.],[10.,2.,0.]],
        faces: vec![vec![0,1,2]], normals: vec![], colors: vec![], corner_uvs: vec![] }), None);
    let ids = [surface.id, mesh.id]; let mut doc = Document::new("closed mixed");
    for entity in [surface, mesh] { doc.entities.insert(entity.id, entity); }
    let mut c = reopen_controller(&Controller::new(doc, limits).unwrap(), limits);
    c.select_objects(&ids).unwrap(); let before = c.document().clone();
    c.submit("ExtractWireframe").unwrap();
    assert_eq!(c.document().entities.values().filter(|e| e.label == "ExtractWire").count(), 10);
    let after = c.document().clone(); c.undo().unwrap(); assert_eq!(c.document(), &before);
    c.redo().unwrap(); assert_eq!(c.document(), &after);
}

#[test]
fn extract_wireframe_open_and_closed_surface_reject_pressure_without_losing_redo() {
    use om9_core::budget::memory::MemoryClass;
    let limits=Limits::default();
    for closed in [false,true] {
        let surface=if closed { om9_core::surface_kernel::cylinder([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,5.,&limits).unwrap() }
            else { om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],4.,3.,&limits).unwrap() };
        let entity=Entity::new("surface",Geometry::Surface(surface),None); let id=entity.id;
        let mut doc=Document::new("surface refusal"); doc.entities.insert(id,entity);
        let seed=Controller::new(doc,limits).unwrap(); let mut c=reopen_controller(&seed,limits);
        c.submit("Point").unwrap(); c.submit("1,2,3").unwrap(); c.undo().unwrap();
        c.select_objects(&[id]).unwrap(); let before=c.coordinator().snapshot();
        let pressure=c.coordinator().reserve_remaining_external_memory_with_held(
            MemoryClass::Kernel,None,c.scene_cache_usage().unwrap()).unwrap();
        assert!(c.submit("ExtractWireframe").is_err());
        assert_eq!(c.coordinator().snapshot(),before); assert_eq!(c.coordinator().redo_len(),1);
        drop(pressure);
        c.submit("ExtractWireframe").unwrap(); assert_eq!(c.document().entities.len(), if closed {8} else {7});
    }
}

#[test]
fn extract_wireframe_reopened_box_uses_source_and_one_transaction() {
    use om9_core::{evaluation::{evaluate_curve, Parameter}, geometry::Transform};
    let limits = Limits::default();
    let mut original = Controller::new(Document::new("reopened wireframe"), limits).unwrap();
    original.submit("Box 10 20 30").unwrap();
    let id = *original.document().entities.keys().next().unwrap();
    let path = std::env::temp_dir().join(format!("om9-extract-reopen-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, original.document(), &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    let mut c = Controller::from_file(&path, limits).unwrap();
    std::fs::remove_file(path).unwrap();
    c.set_current_layer(om9_app::layer_command::LayerChoice::Slot(om9_core::layers::PresetSlot::new(5).unwrap())).unwrap();
    let layer = c.document().current_layer;
    c.select_objects(&[id]).unwrap();
    c.submit("ExtractWireframe").unwrap();
    let curves: Vec<_> = c.document().entities.values().filter(|e| e.label == "ExtractWire").collect();
    assert_eq!(curves.len(), 12);
    assert!(curves.iter().all(|entity| entity.attributes.layer == layer));
    let mut unique = std::collections::BTreeSet::new();
    for entity in curves {
        let Geometry::Curve(curve) = &entity.geometry else { panic!() };
        let eval = |t| evaluate_curve(curve, Parameter::Fraction(t), 0, Transform::identity(), &limits).unwrap()[0];
        let (a, b) = (eval(0.), eval(1.));
        for p in [a, b] {
            assert!(p.iter().zip([10., 20., 30.]).all(|(v, d)| v.abs() < 1e-9 || (*v - d).abs() < 1e-9));
        }
        assert_eq!((0..3).filter(|i| (a[*i] - b[*i]).abs() > 1e-9).count(), 1);
        let corner = |p: [f64; 3]| [p[0] > 5., p[1] > 10., p[2] > 15.];
        let (x, y) = (corner(a), corner(b));
        assert!(unique.insert(if x < y { (x, y) } else { (y, x) }), "duplicate Box edge");
    }
    assert_eq!(unique.len(), 12);
    let saved = reopen_controller(&c, limits);
    assert_eq!(saved.document().entities.len(), 13);
    for (id, entity) in &c.document().entities {
        let restored = &saved.document().entities[id];
        assert_eq!(restored.geometry, entity.geometry);
        assert_eq!(restored.placement, entity.placement);
        assert_eq!(restored.attributes.layer, entity.attributes.layer);
    }
    c.undo().unwrap(); assert_eq!(c.document().entities.len(), 1);
    c.redo().unwrap(); assert_eq!(c.document().entities.len(), 13);
}

#[test]
fn extract_wireframe_product_box_uses_source_edges_and_one_transaction() {
    use om9_core::{evaluation::{evaluate_curve, Parameter}, geometry::Transform, layers::PresetSlot};
    use om9_app::layer_command::LayerChoice;
    let mut c = Controller::new(Document::new("wireframe"), Limits::default()).unwrap();
    c.submit("Box 10 20 30").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    c.set_current_layer(LayerChoice::Slot(PresetSlot::new(5).unwrap())).unwrap();
    let layer = c.document().current_layer;
    assert!(layer.is_some());
    c.select_objects(&[id]).unwrap();
    let before = c.coordinator().undo_len();
    c.submit("ExtractWireframe").unwrap();
    let curves: Vec<_> = c.document().entities.values().filter(|e| e.label == "ExtractWire").collect();
    assert_eq!(curves.len(), 12);
    assert!(curves.iter().all(|e| matches!(e.geometry, Geometry::Curve(om9_core::geometry::Curve::Nurbs(_)))));
    assert!(curves.iter().all(|e| e.attributes.layer == layer));
    let dimensions = [10.,20.,30.];
    for entity in &curves {
        let Geometry::Curve(curve) = &entity.geometry else { unreachable!() };
        let eval = |t| evaluate_curve(curve,Parameter::Fraction(t),0,Transform::identity(),&Limits::default()).unwrap()[0];
        let (a,b) = (eval(0.),eval(1.));
        assert!(a.iter().zip(dimensions).all(|(v,d)| v.abs()<1e-9 || (*v-d).abs()<1e-9));
        assert!(b.iter().zip(dimensions).all(|(v,d)| v.abs()<1e-9 || (*v-d).abs()<1e-9));
        assert_eq!((0..3).filter(|i| (a[*i]-b[*i]).abs()>1e-9).count(),1);
        let mid = eval(0.5);
        assert!((0..3).all(|i| (mid[i]-(a[i]+b[i])*0.5).abs()<1e-9));
    }
    let path = std::env::temp_dir().join(format!("om9-extractwire-{}.om9",uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path,c.document(),&Limits::default(),&om9_core::transaction::Cancellation::default()).unwrap();
    let restored = Controller::from_file(&path,Limits::default()).unwrap();
    for entity in &curves {
        let restored_entity = &restored.document().entities[&entity.id];
        assert_eq!(restored_entity.geometry,entity.geometry);
        assert_eq!(restored_entity.placement,entity.placement);
        assert_eq!(restored_entity.attributes.layer,entity.attributes.layer);
    }
    std::fs::remove_file(path).unwrap();
    assert_eq!(c.coordinator().undo_len(), before + 1);
    c.undo().unwrap(); assert_eq!(c.document().entities.len(), 1);
    c.redo().unwrap(); assert_eq!(c.document().entities.len(), 13);
}

#[test]
fn extract_wireframe_box_edges_match_analytic_corners_in_two_profiles() {
    use om9_core::{evaluation::{evaluate_curve, Parameter}, geometry::Transform};
    let limits = Limits::default();
    let seed = creation::primitive(creation::Primitive::Box { x_mm: 4., y_mm: 5., z_mm: 6. },0.1,&limits).unwrap();
    let mut results = Vec::new();
    for deflection in [0.5,0.01] {
        let (mut c,id) = attested_controller(seed.source(),deflection,Transform::identity(),limits);
        c.select_objects(&[id]).unwrap(); c.submit("ExtractWireframe").unwrap();
        let mut edges = Vec::new();
        for entity in c.document().entities.values().filter(|e| e.label=="ExtractWire") {
            let Geometry::Curve(curve) = &entity.geometry else { panic!() };
            let eval = |t| evaluate_curve(curve,Parameter::Fraction(t),0,Transform::identity(),&limits).unwrap()[0];
            let (a,b)=(eval(0.),eval(1.));
            for point in [a,b] {
                assert!(point.iter().zip([4.,5.,6.]).all(|(v,d)| v.abs()<1e-9 || (*v-d).abs()<1e-9));
            }
            assert_eq!((0..3).filter(|i| (a[*i]-b[*i]).abs()>1e-9).count(),1);
            let mid=eval(0.5);
            assert!((0..3).all(|i| (mid[i]-(a[i]+b[i])*0.5).abs()<1e-9));
            edges.push(curve.clone());
        }
        assert_eq!(edges.len(),12);
        edges.sort_by(|a,b| {
            let point = |c| evaluate_curve(c,Parameter::Fraction(0.25),0,Transform::identity(),&limits).unwrap()[0];
            let x=point(a); let y=point(b);
            x[0].total_cmp(&y[0]).then(x[1].total_cmp(&y[1])).then(x[2].total_cmp(&y[2]))
        });
        results.push(edges);
    }
    assert_eq!(results[0],results[1]);
}

#[test]
fn extract_wireframe_closed_curves_are_exact_for_both_display_profiles() {
    use om9_core::{evaluation::{evaluate_curve, Parameter}, geometry::{Curve, Transform}};
    let limits = Limits::default();
    let circle = om9_core::curve_build::circle_on_frame([0.; 3], [1.,0.,0.], [0.,1.,0.], 2.).unwrap();
    let seed = om9_kernel::extrusion::shell(&circle, [0.,0.,5.], 0.1, &limits).unwrap();
    let mut outputs = Vec::new();
    for (deflection, menu) in [(0.5, false), (0.01, true)] {
        let mut placement = Transform::identity();
        placement.0[0][3] = 10.; placement.0[1][3] = -4.; placement.0[2][3] = 3.;
        let (c, id) = attested_controller(seed.source(), deflection, placement, limits);
        let mut c = reopen_controller(&c, limits);
        let Geometry::Brep(body) = &c.document().entities[&id].geometry else { panic!() };
        let count = body.topology().edges.len();
        assert_eq!(count,3, "one seam and two closed boundary edges");
        c.select_objects(&[id]).unwrap();
        let before = c.coordinator().undo_len();
        if menu { c.dispatch("OM9-CURVE-054", om9_app::session::InvocationOrigin::Menu).unwrap(); }
        else { c.submit("ExtractWireframe").unwrap(); }
        let mut closed = Vec::new();
        let mut seams = 0;
        let curves: Vec<_> = c.document().entities.values().filter(|e| e.label == "ExtractWire").collect();
        assert_eq!(curves.len(), count);
        for entity in curves {
            let Geometry::Curve(curve @ Curve::Nurbs(_)) = &entity.geometry else { panic!("source NURBS required") };
            let points: Vec<_> = [0.,0.13,0.37,0.61,0.91,1.].iter().map(|t|
                evaluate_curve(curve,Parameter::Fraction(*t),0,Transform::identity(),&limits).unwrap()[0]).collect();
            if points.iter().all(|p| ((p[0]-10.).hypot(p[1]+4.)-2.).abs()<1e-8 && (p[2]-points[0][2]).abs()<1e-8) {
                closed.push(curve.clone());
            } else if points.iter().all(|p| (p[0]-points[0][0]).abs()<1e-8 && (p[1]-points[0][1]).abs()<1e-8)
                && (points[0][2]-points[5][2]).abs()>4.9 {
                seams += 1;
            }
        }
        assert_eq!(closed.len(), 2);
        assert_eq!(seams, 1, "shared seam emitted exactly once");
        closed.sort_by(|a,b| {
            let z = |curve| evaluate_curve(curve,Parameter::Fraction(0.),0,Transform::identity(),&limits).unwrap()[0][2];
            z(a).total_cmp(&z(b))
        });
        assert_eq!(c.coordinator().undo_len(), before+1);
        c.undo().unwrap(); assert_eq!(c.document().entities.len(),1);
        c.redo().unwrap(); assert_eq!(c.document().entities.len(),count+1);
        outputs.push(closed);
    }
    assert_eq!(outputs[0],outputs[1]);
}

#[test]
fn extract_wireframe_reopened_sphere_skips_poles_and_keeps_source_seam() {
    use om9_core::{evaluation::{evaluate_curve, Parameter}, geometry::Transform};
    let limits = Limits::default();
    let sphere = creation::primitive(creation::Primitive::Sphere { radius_mm: 5. }, 0.1, &limits).unwrap();
    for (profile, menu) in [(0.5, false), (0.01, true)] {
        let (c, id) = attested_controller(sphere.source(),profile,Transform::translation([10., 0., 0.]),limits);
        let mut c = reopen_controller(&c, limits); c.select_objects(&[id]).unwrap();
        let before = c.document().entities.clone();
        if menu { c.dispatch("OM9-CURVE-054", om9_app::session::InvocationOrigin::Menu).unwrap(); }
        else { c.submit("ExtractWireframe").unwrap(); }
        let curves: Vec<_> = c.document().entities.values().filter(|e| e.label == "ExtractWire").collect();
        assert_eq!(curves.len(), 1, "one nondegenerate seam, not the two pole edges");
        let Geometry::Curve(curve) = &curves[0].geometry else { panic!("curve") };
        let points: Vec<_> = [0., 0.1, 0.25, 0.5, 0.75, 0.9, 1.].into_iter().map(|t|
            evaluate_curve(curve, Parameter::Fraction(t), 0, Transform::identity(), &limits).unwrap()[0]).collect();
        assert!(points.iter().all(|p| ((p[0]-10.).powi(2)+p[1].powi(2)+p[2].powi(2)-25.).abs()<1e-7));
        assert!((points[0][2].abs()-5.).abs()<1e-8 && (points[6][2].abs()-5.).abs()<1e-8);
        assert!(points[0][2] * points[6][2] < 0.);
        let after = c.document().entities.clone(); c.undo().unwrap(); assert_eq!(c.document().entities,before);
        c.redo().unwrap(); assert_eq!(c.document().entities,after);
    }
}

#[test]
fn source_sphere_edge_output_limit_counts_real_curves() {
    let limits = Limits::default();
    let sphere = creation::primitive(creation::Primitive::Sphere { radius_mm: 5. }, 0.1, &limits).unwrap();
    let one_output = Limits { max_outputs: 1, ..limits };
    let curves = om9_kernel::editing::all_source_edges_nurbs_admitted(&sphere, &one_output, |_, _| Ok(())).unwrap();
    assert_eq!(curves.len(), 1);
}

#[test]
fn extract_wireframe_scaled_sphere_single_curve_keeps_placement() {
    use om9_core::{evaluation::{evaluate_curve, Parameter}, geometry::Transform};
    let limits = Limits::default();
    let sphere = creation::primitive(creation::Primitive::Sphere { radius_mm: 5. }, 0.1, &limits).unwrap();
    let placement = Transform([[2.,0.,0.,10.],[0.,1.,0.,0.],[0.,0.,0.5,0.],[0.,0.,0.,1.]]);
    let (c,id) = attested_controller(sphere.source(),0.01,placement,limits);
    let mut c = reopen_controller(&c,limits); c.select_objects(&[id]).unwrap();
    let before = c.document().entities.clone(); c.submit("ExtractWireframe").unwrap();
    let curves: Vec<_> = c.document().entities.values().filter(|e| e.label=="ExtractWire").collect();
    assert_eq!(curves.len(),1); assert_eq!(curves[0].placement,placement);
    let Geometry::Curve(curve) = &curves[0].geometry else { panic!("curve") };
    for t in [0.,0.1,0.25,0.5,0.75,0.9,1.] {
        let p = evaluate_curve(curve,Parameter::Fraction(t),0,curves[0].placement,&limits).unwrap()[0];
        assert!((((p[0]-10.)/2.).powi(2)+p[1].powi(2)+(p[2]/0.5).powi(2)-25.).abs()<1e-7);
    }
    let after = c.document().entities.clone(); c.undo().unwrap(); assert_eq!(c.document().entities,before);
    c.redo().unwrap(); assert_eq!(c.document().entities,after);
}

#[test]
fn extract_wireframe_trimmed_arcs_keep_geometry_for_two_profiles() {
    use om9_core::{evaluation::{evaluate_curve, Parameter}, geometry::{Curve, Transform}};
    let limits = Limits::default();
    let arc = om9_core::curve_build::arc_through([2.,0.,0.], [2f64.sqrt(),2f64.sqrt(),0.], [0.,2.,0.]).unwrap();
    let profile = Curve::Polycurve { segments: vec![arc,
        Curve::Line { start: [0.,2.,0.], end: [2.,0.,0.], domain: [0.,1.] }], reversed: false };
    let seed = om9_kernel::extrusion::shell(&profile, [0.,0.,5.], 0.1, &limits).unwrap();
    let mut results = Vec::new();
    for deflection in [0.5,0.01] {
        let (c,id) = attested_controller(seed.source(),deflection,Transform::identity(),limits);
        let mut c = reopen_controller(&c, limits);
        let Geometry::Brep(shape) = &c.document().entities[&id].geometry else { panic!() };
        let expected_count = shape.topology().edges.len();
        let mut endpoints = Vec::new();
        for index in 0..expected_count {
            let Ok(curve) = om9_kernel::editing::edge_nurbs(shape,index,&limits) else { continue };
            let mid = evaluate_curve(&curve,Parameter::Fraction(0.5),0,Transform::identity(),&limits).unwrap()[0];
            let pair = om9_kernel::editing::oriented_edge_endpoints(shape,index,&limits).unwrap();
            if (mid[0].hypot(mid[1])-2.).abs()<1e-8 && (pair.0[2]-pair.1[2]).abs()<1e-8 {
                let Curve::Nurbs(nurbs) = curve else { panic!("edge adapter returns NURBS") };
                endpoints.push((pair,nurbs.domain));
            }
        }
        assert_eq!(endpoints.len(),2);
        c.select_objects(&[id]).unwrap();
        c.submit("ExtractWireframe").unwrap();
        let curves: Vec<_> = c.document().entities.values().filter_map(|entity|
            match &entity.geometry { Geometry::Curve(curve) if entity.label == "ExtractWire" => Some(curve), _ => None }).collect();
        assert_eq!(curves.len(),expected_count);
        let mut arcs = Vec::new();
        for curve in curves {
            let points: Vec<_> = [0.,0.17,0.38,0.73,1.].iter().map(|t|
                evaluate_curve(curve,Parameter::Fraction(*t),0,Transform::identity(),&limits).unwrap()[0]).collect();
            if points.iter().all(|p| (p[0].hypot(p[1])-2.).abs()<1e-8 && (p[2]-points[0][2]).abs()<1e-8) {
                assert!(points.iter().all(|p| p[0]>=-1e-8 && p[1]>=-1e-8));
                assert!((points[0][0]-points[4][0]).abs()>1.9 && (points[0][1]-points[4][1]).abs()>1.9);
                let Curve::Nurbs(nurbs) = curve else { panic!("edge output returns NURBS") };
                assert!(endpoints.iter().any(|((a,b),domain)| (0..3).all(|i|
                    (points[0][i]-a[i]).abs()<1e-8 && (points[4][i]-b[i]).abs()<1e-8)
                    && nurbs.domain == *domain && domain[1] > domain[0]),
                    "trimmed arc must retain oriented endpoints and converted NURBS domain");
                arcs.push(curve.clone());
            }
        }
        assert_eq!(arcs.len(),2);
        arcs.sort_by(|a,b| {
            let z = |curve| evaluate_curve(curve,Parameter::Fraction(0.),0,Transform::identity(),&limits).unwrap()[0][2];
            z(a).total_cmp(&z(b))
        });
        results.push(arcs);
    }
    assert_eq!(results[0],results[1]);
}

#[test]
#[ignore = "manual independent OCCT domain comparison fixture"]
fn extract_wireframe_write_domain_fixture() {
    use om9_core::{evaluation::{evaluate_curve, Parameter}, geometry::{Curve, Transform}};
    let limits = Limits::default();
    let arc = om9_core::curve_build::arc_through([2.,0.,0.], [2f64.sqrt(),2f64.sqrt(),0.], [0.,2.,0.]).unwrap();
    let profile = Curve::Polycurve { segments: vec![arc,
        Curve::Line { start: [0.,2.,0.], end: [2.,0.,0.], domain: [0.,1.] }], reversed: false };
    let seed = om9_kernel::extrusion::shell(&profile,[0.,0.,5.],0.1,&limits).unwrap();
    std::fs::write("L:/om9-extractwire-domain-source.brep", seed.source()).unwrap();
    let (mut c,id) = attested_controller(seed.source(),0.1,Transform::identity(),limits);
    c.select_objects(&[id]).unwrap(); c.submit("ExtractWireframe").unwrap();
    for entity in c.document().entities.values().filter(|e| e.label=="ExtractWire") {
        let Geometry::Curve(curve @ Curve::Nurbs(n)) = &entity.geometry else { panic!() };
        let p = evaluate_curve(curve,Parameter::Fraction(0.5),0,Transform::identity(),&limits).unwrap()[0];
        if (p[0].hypot(p[1])-2.).abs()<1e-8 && p[0]>0.1 && p[1]>0.1 {
            println!("arc_z={:.9} converted_domain=[{:.17},{:.17}]",p[2],n.domain[0],n.domain[1]);
        }
    }
}

#[test]
fn extract_wireframe_commit_budget_preserves_redo_and_document() {
    let limits = Limits { max_objects: 10, ..Limits::default() };
    let seed = creation::primitive(creation::Primitive::Box { x_mm: 4., y_mm: 5., z_mm: 6. },0.1,&limits).unwrap();
    let (c,id) = attested_controller(seed.source(),0.1,om9_core::geometry::Transform::identity(),limits);
    let mut c = reopen_controller(&c, limits);
    c.submit("Point").unwrap(); c.submit("1,2,3").unwrap(); c.undo().unwrap();
    c.select_objects(&[id]).unwrap();
    let before = c.coordinator().snapshot();
    assert!(c.submit("ExtractWireframe").is_err());
    assert_eq!(c.coordinator().snapshot(),before);
    assert_eq!(c.coordinator().redo_len(),1);
}

#[test]
fn extract_wireframe_reopened_pressure_preserves_document_and_redo() {
    use om9_core::budget::memory::MemoryClass;
    let limits = Limits { max_bytes: 64 * 1024 * 1024, ..Limits::default() };
    let mut seed = Controller::new(Document::new("wireframe pressure reopen"), limits).unwrap();
    seed.submit("Box 10 20 30").unwrap();
    let id = *seed.document().entities.keys().next().unwrap();
    let mut c = reopen_controller(&seed, limits);
    c.submit("Point").unwrap(); c.submit("1,2,3").unwrap(); c.undo().unwrap();
    c.select_objects(&[id]).unwrap();
    let before = c.coordinator().snapshot();
    let pressure = c.coordinator().reserve_remaining_external_memory_with_held(
        MemoryClass::Kernel, None, c.scene_cache_usage().unwrap()).unwrap();
    assert!(c.submit("ExtractWireframe").is_err());
    assert_eq!(c.coordinator().snapshot(), before);
    assert_eq!(c.coordinator().redo_len(), 1);
    drop(pressure);
    c.submit("ExtractWireframe").unwrap();
    assert_eq!(c.document().entities.len(), 13);
}

#[test]
fn extract_wireframe_reopened_affine_box_keeps_source_placement_and_edges() {
    use om9_core::{evaluation::{evaluate_curve, Parameter}, geometry::Transform};
    let limits = Limits::default();
    let seed = creation::primitive(creation::Primitive::Box { x_mm: 4., y_mm: 5., z_mm: 6. }, 0.1, &limits).unwrap();
    for placement in [
        Transform([[2., 0., 0., 3.], [0., 1., 0., -2.], [0., 0., 0.5, 1.], [0., 0., 0., 1.]]),
        Transform([[1., 0.5, 0., 3.], [0., 1., 0., -2.], [0., 0., 1., 1.], [0., 0., 0., 1.]]),
        Transform([[-1., 0., 0., 3.], [0., 2., 0., -2.], [0., 0., 1., 1.], [0., 0., 0., 1.]]),
    ] {
      for (profile, menu) in [(0.5, true), (0.01, false)] {
        let (c, id) = attested_controller(seed.source(), profile, placement, limits);
        let mut c = reopen_controller(&c, limits); c.select_objects(&[id]).unwrap();
        let before = c.document().entities.clone();
        if menu { c.dispatch("OM9-CURVE-054", om9_app::session::InvocationOrigin::Menu).unwrap(); }
        else { c.submit("ExtractWireframe").unwrap(); }
        let curves: Vec<_> = c.document().entities.values().filter(|e| e.label == "ExtractWire").collect();
        assert_eq!(curves.len(), 12);
        let mut unique = std::collections::BTreeSet::new();
        for entity in curves {
            assert_eq!(entity.placement, placement);
            let Geometry::Curve(curve) = &entity.geometry else { panic!("curve") };
            let local = |t| evaluate_curve(curve, Parameter::Fraction(t), 0, Transform::identity(), &limits).unwrap()[0];
            let a = local(0.); let b = local(1.);
            for point in [a, b] { assert!(point.iter().zip([4.,5.,6.]).all(|(v,d)| v.abs()<1e-8 || (*v-d).abs()<1e-8)); }
            assert_eq!((0..3).filter(|i| (a[*i]-b[*i]).abs()>1e-8).count(), 1);
            let midpoint = evaluate_curve(curve, Parameter::Fraction(0.5), 0, entity.placement, &limits).unwrap()[0];
            let expected = placement.point(std::array::from_fn(|i| (a[i]+b[i])*0.5)).unwrap();
            assert!((0..3).all(|i| (midpoint[i]-expected[i]).abs()<1e-8));
            let corner = |p: [f64; 3]| [p[0] > 2., p[1] > 2.5, p[2] > 3.];
            let (x,y) = (corner(a),corner(b)); assert!(unique.insert(if x<y {(x,y)} else {(y,x)}));
        }
        assert_eq!(unique.len(), 12);
        let after = c.document().entities.clone(); let mut saved = reopen_controller(&c,limits);
        for (id, entity) in &after { assert_eq!(saved.document().entities[id].geometry,entity.geometry); assert_eq!(saved.document().entities[id].placement,entity.placement); }
        let edge = saved.document().entities.values().find(|e| e.label == "ExtractWire").unwrap();
        let edge_id = edge.id;
        let Geometry::Curve(edge_curve) = &edge.geometry else { panic!("curve") };
        let p = evaluate_curve(edge_curve,Parameter::Fraction(0.),0,edge.placement,&limits).unwrap()[0];
        let q = evaluate_curve(edge_curve,Parameter::Fraction(1.),0,edge.placement,&limits).unwrap()[0];
        let length = (0..3).map(|i| (p[i]-q[i]).powi(2)).sum::<f64>().sqrt();
        saved.select_objects(&[edge_id]).unwrap();
        assert!((saved.measure_selected_length().unwrap().value-length).abs()<1e-6);
        c.undo().unwrap(); assert_eq!(c.document().entities,before);
        c.redo().unwrap(); assert_eq!(c.document().entities,after);
      }
    }
}

#[test]
fn native_brep_source_edge_resolves_unique_box_segment_without_display() {
    use om9_core::evaluation::{evaluate_curve,Parameter};
    let limits=Limits::default();
    let shape=om9_kernel::brep::box_brep([4.,5.,6.],0.1,&limits).unwrap();
    let native=om9_kernel::brep::to_3dm(&shape,1e-6,&limits).unwrap();
    let source=om9_kernel::brep::from_3dm_source_typed_admitted(&native.payload,1e-6,&limits,|_,_|Ok(())).unwrap();
    let resolve=|point,segment|om9_kernel::editing::source_edge_from_text_segment_admitted(
        &source,point,0.01,segment,1e-6,&limits,|_,_|Ok(()));
    let (curve,point)=resolve([2.,0.,6.],[[0.,0.,6.],[4.,0.,6.]]).unwrap();
    assert!((point[0]-2.).abs()<1e-6 && point[1].abs()<1e-6 && (point[2]-6.).abs()<1e-6);
    let endpoints=[0.,1.].map(|t|evaluate_curve(&curve,Parameter::Fraction(t),0,om9_core::geometry::Transform::identity(),&limits).unwrap()[0]);
    for expected in [[0.,0.,6.],[4.,0.,6.]] {
        assert!(endpoints.iter().any(|p|(0..3).all(|i|(p[i]-expected[i]).abs()<1e-6)),"{endpoints:?}");
    }
    assert!(resolve([2.,0.,6.],[[0.,0.,6.],[4.,5.,6.]]).is_err());
    assert!(resolve([2.,0.,6.],[[0.,0.,6.],[0.,0.,6.]]).is_err());
    let mut called=false;
    let rejected=om9_kernel::editing::source_edge_from_text_segment_admitted(&source,[2.,0.,6.],0.01,
        [[0.,0.,6.],[4.,0.,6.]],1e-6,&limits,|_,_|{called=true;Err(om9_core::Error::Budget("managed memory"))});
    assert!(called && rejected.is_err());
}

#[test]
fn extract_wireframe_reopened_native_brep_uses_source_and_one_undo() {
    use om9_core::{evaluation::{evaluate_curve,Parameter},geometry::Transform};
    let limits=Limits::default();let shape=om9_kernel::brep::box_brep([4.,5.,6.],0.1,&limits).unwrap();
    let mut native=om9_kernel::brep::to_3dm(&shape,0.001,&limits).unwrap();
    for p in &mut std::sync::Arc::make_mut(&mut native.display).vertices {p[0]+=1000.;}
    for (placement,menu) in [(Transform::translation([7.,-3.,2.]),false),
        (Transform([[1.,0.5,0.,3.],[0.,1.,0.,-2.],[0.,0.,2.,1.],[0.,0.,0.,1.]]),true)] {
        let mut entity=Entity::new("native",Geometry::NativeBrep(native.clone()),None);entity.placement=placement;let id=entity.id;
        let mut doc=Document::with_template_tolerance("native edge source");doc.entities.insert(id,entity);
        let mut c=reopen_controller(&Controller::new(doc,limits).unwrap(),limits);
        c.submit("Point").unwrap();c.submit("100,100,100").unwrap();c.undo().unwrap();c.select_objects(&[id]).unwrap();
        let before=c.document().clone();let pressure=c.coordinator().reserve_remaining_external_memory_with_held(om9_core::budget::memory::MemoryClass::Kernel,None,c.scene_cache_usage().unwrap()).unwrap();
        assert!(c.submit("ExtractWireframe").is_err());assert_eq!(c.document(),&before);assert_eq!(c.coordinator().redo_len(),1);drop(pressure);
        if menu {c.dispatch("OM9-CURVE-054",om9_app::session::InvocationOrigin::Menu).unwrap();} else {c.submit("ExtractWireframe").unwrap();}
        let outputs:Vec<_>=c.document().entities.values().filter(|e|e.label=="ExtractWire").collect();assert_eq!(outputs.len(),12);
        let corners:Vec<_>=(0..8).map(|i|placement.point([if i&1==0 {0.}else{4.},if i&2==0 {0.}else{5.},if i&4==0 {0.}else{6.}]).unwrap()).collect();
        let expected:Vec<_>=(0..8).flat_map(|a|(a+1..8).filter(move |b|((a^b) as u32).count_ones()==1).map(move |b|(a,b))).collect();
        let mut seen=std::collections::BTreeSet::new();
        for entity in outputs {let Geometry::Curve(curve)=&entity.geometry else {panic!("curve")};
            let points=[0.,1.].map(|t|evaluate_curve(curve,Parameter::Fraction(t),0,entity.placement,&limits).unwrap()[0]);
            let near=|a:[f64;3],b:[f64;3]|(0..3).all(|i|(a[i]-b[i]).abs()<1e-6);
            let edge=expected.iter().find(|&&(a,b)|(near(points[0],corners[a])&&near(points[1],corners[b]))||(near(points[0],corners[b])&&near(points[1],corners[a]))).unwrap();
            assert!(seen.insert(*edge));
        }
        assert_eq!(seen.len(),12);let after=c.document().clone();c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }
}

#[test]
fn extract_wireframe_native_brep_failure_after_other_source_keeps_redo() {
    let limits=Limits::default();let shape=om9_kernel::brep::box_brep([4.,5.,6.],0.1,&limits).unwrap();
    let mut native=om9_kernel::brep::to_3dm(&shape,0.001,&limits).unwrap();native.payload=vec![1,2,3,4].into();
    let mut good=Entity::new("first",Geometry::Brep(shape),None);good.id=om9_core::identity::ObjectId(uuid::Uuid::from_u128(1));
    let mut bad=Entity::new("invalid native",Geometry::NativeBrep(native),None);bad.id=om9_core::identity::ObjectId(uuid::Uuid::from_u128(2));
    let ids=[good.id,bad.id];let mut doc=Document::with_template_tolerance("bad native");doc.entities.insert(good.id,good);doc.entities.insert(bad.id,bad);
    let mut c=reopen_controller(&Controller::new(doc,limits).unwrap(),limits);
    c.submit("Point").unwrap();c.submit("100,100,100").unwrap();c.undo().unwrap();c.select_objects(&ids).unwrap();let before=c.coordinator().snapshot();
    assert!(c.submit("ExtractWireframe").is_err());assert_eq!(c.coordinator().snapshot(),before);assert_eq!(c.coordinator().redo_len(),1);
}

#[test]
fn extract_wireframe_native_brep_and_mesh_share_one_transaction() {
    use om9_core::{evaluation::{evaluate_curve,Parameter},geometry::{Mesh,Transform}};
    let limits=Limits::default();let shape=om9_kernel::brep::box_brep([4.,5.,6.],0.1,&limits).unwrap();
    let native=om9_kernel::brep::to_3dm(&shape,0.001,&limits).unwrap();let entity=Entity::new("native",Geometry::NativeBrep(native),None);
    let mut mesh=Entity::new("mesh",Geometry::Mesh(Mesh {vertices:vec![[0.,0.,0.],[2.,0.,0.],[0.,2.,0.]],faces:vec![vec![0,1,2]],normals:vec![],colors:vec![],corner_uvs:vec![]}),None);
    mesh.placement=Transform::translation([20.,0.,0.]);let ids=[entity.id,mesh.id];
    let mut doc=Document::with_template_tolerance("native and mesh");for e in [entity,mesh] {doc.entities.insert(e.id,e);}
    let mut c=reopen_controller(&Controller::new(doc,limits).unwrap(),limits);c.select_objects(&ids).unwrap();let before=c.document().clone();
    c.submit("ExtractWireframe").unwrap();let outputs:Vec<_>=c.document().entities.values().filter(|e|e.label=="ExtractWire").collect();assert_eq!(outputs.len(),15);
    let mut native_count=0;let mut mesh_count=0;
    for e in outputs {let Geometry::Curve(curve)=&e.geometry else {panic!("curve")};
        let p=evaluate_curve(curve,Parameter::Fraction(0.5),0,e.placement,&limits).unwrap()[0];
        if (20.-1e-7..=22.+1e-7).contains(&p[0]) {mesh_count+=1;} else {assert!((0.-1e-7..=4.+1e-7).contains(&p[0]));native_count+=1;}
    }
    assert_eq!((native_count,mesh_count),(12,3));let after=c.document().clone();c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
}

#[test]
#[ignore = "requires native exchange and authored openNURBS 3DM fixture"]
fn extract_wireframe_opaque_imported_3dm_quarter_cylinder() {
    use om9_core::{evaluation::{evaluate_curve,Parameter},transaction::{Cancellation,TransactionCoordinator,World}};
    use om9_exchange::{BrepMode,ImportMode,ImportOptions};
    use sha2::{Digest,Sha256};
    let limits=Limits::default();let native_path=std::path::PathBuf::from(std::env::var_os("OM9_EXCHANGE_NATIVE").unwrap());
    let fixture=std::path::PathBuf::from(std::env::var_os("OM9_DUPEDGE_3DM_ARC").unwrap());
    let digest:[u8;32]=Sha256::digest(std::fs::read(&native_path).unwrap()).into();let host=om9_app::three_dm_host::ThreeDmHost::admit(&native_path,digest).unwrap();
    let document=Document::with_template_tolerance("imported native arc");let document_id=document.id;
    let mut coordinator=TransactionCoordinator::new(World::new(vec![document],&limits).unwrap(),limits).unwrap();
    let options=ImportOptions {mode:ImportMode::Geometry,brep:BrepMode::RetainOpaque,..Default::default()};
    host.import_with_options(&mut coordinator,document_id,&fixture,&options,&Cancellation::default()).unwrap();
    let imported=coordinator.world().documents[&document_id].as_ref().clone();
    let id=imported.entities.values().find(|e|matches!(e.geometry,Geometry::NativeBrep(_))).unwrap().id;
    let mut c=reopen_controller(&Controller::new(imported,limits).unwrap(),limits);
    c.submit("Point").unwrap();c.submit("100,100,100").unwrap();c.undo().unwrap();c.select_objects(&[id]).unwrap();let before=c.document().clone();
    c.submit("ExtractWireframe").unwrap();let outputs:Vec<_>=c.document().entities.values().filter(|e|e.label=="ExtractWire").collect();assert_eq!(outputs.len(),4);
    let mut arcs=0;let mut lines=0;
    for entity in outputs {let Geometry::Curve(curve)=&entity.geometry else {panic!("curve")};
        let points=[0.,0.25,0.5,0.75,1.].map(|t|evaluate_curve(curve,Parameter::Fraction(t),0,entity.placement,&limits).unwrap()[0]);
        if points.iter().all(|p|p[2].abs()<1e-6 || (p[2]-5.).abs()<1e-6) && (points[0][2]-points[4][2]).abs()<1e-6 {
            arcs+=1;for p in points {assert!((p[0]*p[0]+p[1]*p[1]-4.).abs()<1e-5);assert!(p[0]>=-1e-6&&p[1]>=-1e-6);}
        } else {
            lines+=1;for p in points {assert!((p[0]-2.).abs()<1e-6&&p[1].abs()<1e-6 || p[0].abs()<1e-6&&(p[1]-2.).abs()<1e-6);assert!((-1e-6..=5.+1e-6).contains(&p[2]));}
        }
    }
    assert_eq!((arcs,lines),(2,2));let after=c.document().clone();c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
}

#[test]
#[ignore = "requires native exchange and authored openNURBS 3DM fixture"]
fn extract_wireframe_opaque_imported_3dm_capped_cylinder() {
    use om9_core::{evaluation::{evaluate_curve,Parameter},transaction::{Cancellation,TransactionCoordinator,World}};
    use om9_exchange::{BrepMode,ImportMode,ImportOptions};use sha2::{Digest,Sha256};
    let limits=Limits::default();let native_path=std::path::PathBuf::from(std::env::var_os("OM9_EXCHANGE_NATIVE").unwrap());
    let fixture=std::path::PathBuf::from(std::env::var_os("OM9_DUPEDGE_3DM_CYLINDER").unwrap());
    let digest:[u8;32]=Sha256::digest(std::fs::read(&native_path).unwrap()).into();let host=om9_app::three_dm_host::ThreeDmHost::admit(&native_path,digest).unwrap();
    let document=Document::with_template_tolerance("imported native cylinder");let document_id=document.id;
    let mut coordinator=TransactionCoordinator::new(World::new(vec![document],&limits).unwrap(),limits).unwrap();
    host.import_with_options(&mut coordinator,document_id,&fixture,&ImportOptions {mode:ImportMode::Geometry,brep:BrepMode::RetainOpaque,..Default::default()},&Cancellation::default()).unwrap();
    let imported=coordinator.world().documents[&document_id].as_ref().clone();let id=imported.entities.values().find(|e|matches!(e.geometry,Geometry::NativeBrep(_))).unwrap().id;
    let mut c=reopen_controller(&Controller::new(imported,limits).unwrap(),limits);c.select_objects(&[id]).unwrap();let before=c.document().clone();
    c.submit("ExtractWireframe").unwrap();let outputs:Vec<_>=c.document().entities.values().filter(|e|e.label=="ExtractWire").collect();assert_eq!(outputs.len(),3);
    let mut circles=0;let mut seams=0;
    for entity in outputs {let Geometry::Curve(curve)=&entity.geometry else {panic!("curve")};
        let points=[0.,0.25,0.5,0.75,1.].map(|t|evaluate_curve(curve,Parameter::Fraction(t),0,entity.placement,&limits).unwrap()[0]);
        if (points[0][2]-points[4][2]).abs()<1e-6 {circles+=1;for p in points {assert!((p[0]*p[0]+p[1]*p[1]-4.).abs()<1e-5);assert!(p[2].abs()<1e-6||(p[2]-5.).abs()<1e-6);}}
        else {seams+=1;for p in points {assert!((p[0]*p[0]+p[1]*p[1]-4.).abs()<1e-5);assert!((-1e-6..=5.+1e-6).contains(&p[2]));}}
    }
    assert_eq!((circles,seams),(2,1));let after=c.document().clone();c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
}

#[test]
#[ignore = "requires native exchange and authored openNURBS 3DM fixture"]
fn extract_wireframe_nested_imported_3dm_occurrences() {
    use om9_core::{evaluation::{evaluate_curve,Parameter}, geometry::Transform, model::Definition,
        transaction::{Cancellation,TransactionCoordinator,World}};
    use om9_exchange::{BrepMode,ImportMode,ImportOptions};
    use sha2::{Digest,Sha256};
    let limits = Limits::default();
    let native_path = std::path::PathBuf::from(std::env::var_os("OM9_EXCHANGE_NATIVE").unwrap());
    let fixture = std::path::PathBuf::from(std::env::var_os("OM9_DUPEDGE_3DM_ARC").unwrap());
    let digest: [u8;32] = Sha256::digest(std::fs::read(&native_path).unwrap()).into();
    let host = om9_app::three_dm_host::ThreeDmHost::admit(&native_path,digest).unwrap();
    let doc = Document::with_template_tolerance("imported block"); let doc_id = doc.id;
    let mut coordinator = TransactionCoordinator::new(World::new(vec![doc], &limits).unwrap(), limits).unwrap();
    host.import_with_options(&mut coordinator, doc_id, &fixture,
        &ImportOptions { mode: ImportMode::Geometry, brep: BrepMode::RetainOpaque, ..Default::default() },
        &Cancellation::default()).unwrap();
    let imported = coordinator.world().documents[&doc_id].as_ref();
    let geometry = imported.entities.values().find(|e| matches!(e.geometry, Geometry::NativeBrep(_))).unwrap().geometry.clone();
    // Construct a new document: an imported root's retained-file binding cannot
    // be moved into a definition without also remapping its ownership metadata.
    let mut document = Document::with_template_tolerance("imported block");
    let mut leaf = Entity::new("imported leaf", geometry, None);
    leaf.placement = Transform::translation([1.,2.,0.]);
    let inner = Definition::new("imported quarter cylinder", vec![leaf]); let inner_id = inner.id;
    let mut nested = Entity::new("nested", Geometry::Instance(inner_id), None);
    let affine = Transform([[2.,0.5,0.,0.],[0.,1.,0.,3.],[0.,0.,1.,0.],[0.,0.,0.,1.]]);
    nested.placement = affine;
    let outer = Definition::new("outer", vec![nested]); let outer_id = outer.id;
    document.definitions.insert(inner_id, inner); document.definitions.insert(outer_id, outer);
    let mut roots = Vec::new();
    for x in [10.,40.] {
        let mut root = Entity::new("instance", Geometry::Instance(outer_id), None);
        root.placement = Transform::translation([x,0.,0.]); roots.push(root.id);
        document.entities.insert(root.id, root);
    }
    let placements = [10.,40.].map(|x| Transform::translation([x,0.,0.]).compose(affine).unwrap()
        .compose(Transform::translation([1.,2.,0.])).unwrap());
    let mut c = reopen_controller(&Controller::new(document, limits).unwrap(), limits);
    c.select_objects(&roots).unwrap(); let before = c.document().clone();
    c.submit("ExtractWireframe").unwrap();
    let outputs: Vec<_> = c.document().entities.values().filter(|e| e.label == "ExtractWire").collect();
    assert_eq!(outputs.len(),8);
    let mut counts = [[0,0];2];
    for e in outputs {
        let group = placements.iter().position(|p| *p == e.placement).expect("composed placement");
        let Geometry::Curve(curve) = &e.geometry else { panic!("curve") };
        let points = [0.,0.25,0.5,0.75,1.].map(|t| evaluate_curve(curve,Parameter::Fraction(t),0,Transform::identity(),&limits).unwrap()[0]);
        let arc = (points[0][2] - points[4][2]).abs() < 1e-6;
        counts[group][if arc {0} else {1}] += 1;
        for p in points {
            if arc { assert!((p[0]*p[0]+p[1]*p[1]-4.).abs()<1e-5 && p[0]>=-1e-6 && p[1]>=-1e-6);
                assert!(p[2].abs()<1e-6 || (p[2]-5.).abs()<1e-6); }
            else { assert!((p[0]-2.).abs()<1e-6 && p[1].abs()<1e-6 || p[0].abs()<1e-6 && (p[1]-2.).abs()<1e-6);
                assert!((-1e-6..=5.+1e-6).contains(&p[2])); }
        }
    }
    assert_eq!(counts, [[2,2],[2,2]]);
    let after = c.document().clone(); assert_eq!(reopen_controller(&c, limits).document().entities, after.entities);
    c.undo().unwrap(); assert_eq!(c.document(), &before); c.redo().unwrap(); assert_eq!(c.document(), &after);
}

#[test]
#[ignore = "requires native exchange and authored openNURBS 3DM fixture"]
fn extract_wireframe_nested_curved_3dm_block_roundtrip() {
    use om9_core::{evaluation::{evaluate_curve,Parameter},geometry::Transform,model::Definition,
        transaction::{Cancellation,TransactionCoordinator,World}};
    use om9_exchange::{BrepMode,ImportMode,ImportOptions};
    use sha2::{Digest,Sha256};
    let limits=Limits::default();
    let native_path=std::path::PathBuf::from(std::env::var_os("OM9_EXCHANGE_NATIVE").unwrap());
    let fixture=std::path::PathBuf::from(std::env::var_os("OM9_DUPEDGE_3DM_ARC").unwrap());
    let digest:[u8;32]=Sha256::digest(std::fs::read(&native_path).unwrap()).into();
    let host=om9_app::three_dm_host::ThreeDmHost::admit(&native_path,digest).unwrap();
    let seed=Document::with_template_tolerance("seed curved 3dm"); let seed_id=seed.id;
    let mut coordinator=TransactionCoordinator::new(World::new(vec![seed],&limits).unwrap(),limits).unwrap();
    host.import_with_options(&mut coordinator,seed_id,&fixture,&ImportOptions {mode:ImportMode::Geometry,brep:BrepMode::RetainOpaque,..Default::default()},&Cancellation::default()).unwrap();
    let geometry=coordinator.world().documents[&seed_id].entities.values().find(|e|matches!(e.geometry,Geometry::NativeBrep(_))).unwrap().geometry.clone();
    let mut leaf=Entity::new("quarter cylinder",geometry,None);leaf.placement=Transform::translation([1.,2.,0.]);
    let inner=Definition::new("inner curved",vec![leaf]);let inner_id=inner.id;
    let mut nested=Entity::new("nested",Geometry::Instance(inner_id),None);
    nested.placement=Transform([[2.,0.5,0.,0.],[0.,1.,0.,3.],[0.,0.,1.,0.],[0.,0.,0.,1.]]);
    let outer=Definition::new("outer curved",vec![nested]);let outer_id=outer.id;
    let mut doc=Document::with_template_tolerance("curved block 3dm");
    doc.definitions.insert(inner_id,inner);doc.definitions.insert(outer_id,outer);
    for x in [10.,40.] {let mut root=Entity::new("root",Geometry::Instance(outer_id),None);
        root.placement=Transform::translation([x,0.,0.]);doc.entities.insert(root.id,root);}
    let c=Controller::new(doc,limits).unwrap();
    let path=std::env::temp_dir().join(format!("om9-nested-curved-{}.3dm",uuid::Uuid::new_v4()));
    struct Remove(std::path::PathBuf);impl Drop for Remove {fn drop(&mut self){let _=std::fs::remove_file(&self.0);}}
    let _cleanup=Remove(path.clone());
    host.export(c.coordinator(),c.document().id,&path,&Cancellation::default()).unwrap();
    for brep in [BrepMode::EditableStrict,BrepMode::RetainOpaque] {
        let doc=Document::with_template_tolerance("import curved block");let id=doc.id;
        let mut coordinator=TransactionCoordinator::new(World::new(vec![doc],&limits).unwrap(),limits).unwrap();
        host.import_with_options(&mut coordinator,id,&path,&ImportOptions {mode:ImportMode::Geometry,brep,..Default::default()},&Cancellation::default()).unwrap();
        let imported=coordinator.world().documents[&id].as_ref().clone();
        let roots:Vec<_>=imported.entities.values().filter(|e|matches!(e.geometry,Geometry::Instance(_))).map(|e|e.id).collect();
        assert_eq!(roots.len(),2);assert_eq!(imported.definitions.len(),2);
        let leaves:Vec<_>=imported.definitions.values().flat_map(|d|d.members.iter()).filter(|e|!matches!(e.geometry,Geometry::Instance(_))).collect();
        assert_eq!(leaves.len(),1);
        assert!(if brep==BrepMode::RetainOpaque {matches!(leaves[0].geometry,Geometry::NativeBrep(_))}else{matches!(leaves[0].geometry,Geometry::Brep(_))});
        let seed=Controller::new(imported,limits).unwrap();let mut c=reopen_controller(&seed,limits);c.select_objects(&roots).unwrap();
        let before=c.document().clone();c.submit("ExtractWireframe").unwrap();
        let outputs:Vec<_>=c.document().entities.values().filter(|e|e.label=="ExtractWire").collect();assert_eq!(outputs.len(),8);
        let mut seen=[[false;4];2];
        for e in outputs {let Geometry::Curve(curve)=&e.geometry else {panic!("curve")};
            let world=[0.,0.25,0.5,0.75,1.].map(|t|evaluate_curve(curve,Parameter::Fraction(t),0,e.placement,&limits).unwrap()[0]);
            let group=if world[0][0]<30. {0} else {1};let offset=if group==0 {10.}else{40.};
            let points=world.map(|p| {let y=p[1]-5.;[(p[0]-offset-0.5*(y+2.))/2.-1.,y,p[2]]});
            let arc=(points[0][2]-points[4][2]).abs()<1e-6;
            let edge=if arc {if points[0][2].abs()<1e-6 {0}else{1}}
                else if (points[0][0]-2.).abs()<1e-6 {2}else{3};
            assert!(!seen[group][edge],"duplicate source edge");seen[group][edge]=true;
            if arc { let endpoint=|p:[f64;3],x:f64,y:f64|(p[0]-x).abs()<1e-6&&(p[1]-y).abs()<1e-6;
                assert!(endpoint(points[0],2.,0.)&&endpoint(points[4],0.,2.) || endpoint(points[0],0.,2.)&&endpoint(points[4],2.,0.)); }
            else { assert!(points[0][2].abs()<1e-6&&(points[4][2]-5.).abs()<1e-6 || (points[0][2]-5.).abs()<1e-6&&points[4][2].abs()<1e-6); }
            for p in points {if arc {assert!((p[0]*p[0]+p[1]*p[1]-4.).abs()<1e-5 && p[0]>=-1e-6 && p[1]>=-1e-6);assert!(p[2].abs()<1e-6 || (p[2]-5.).abs()<1e-6);}
                else {assert!((p[0]-2.).abs()<1e-6 && p[1].abs()<1e-6 || p[0].abs()<1e-6 && (p[1]-2.).abs()<1e-6);assert!((-1e-6..=5.+1e-6).contains(&p[2]));}}
        }
        assert!(seen.into_iter().flatten().all(|value|value));let after=c.document().clone();
        assert_eq!(reopen_controller(&c,limits).document().entities,after.entities);
        c.undo().unwrap();assert_eq!(c.document(),&before);c.redo().unwrap();assert_eq!(c.document(),&after);
    }
}

#[test]
fn source_all_edges_refuses_after_preparing_some_curves() {
    let limits = Limits::default();
    let shape = creation::primitive(creation::Primitive::Box { x_mm: 4., y_mm: 5., z_mm: 6. }, 0.1, &limits).unwrap();
    let mut calls = 0;
    let result = om9_kernel::editing::all_source_edges_nurbs_admitted(&shape, &limits, |_, pending| {
        if pending > 0 { calls += 1; }
        if calls == 5 { Err(om9_core::Error::Budget("simulated edge output pressure")) } else { Ok(()) }
    });
    assert!(result.is_err());
    assert_eq!(calls, 5);
}

#[test]
fn extract_wireframe_reopened_compound_keeps_box_and_sphere_edges() {
    use om9_kernel::editing::{boolean, BooleanOp, BooleanOutcome};
    use om9_core::geometry::Transform;
    let limits = Limits::default();
    let box_shape = creation::primitive(creation::Primitive::Box { x_mm: 4., y_mm: 5., z_mm: 6. }, 0.1, &limits).unwrap();
    let sphere = creation::primitive(creation::Primitive::Sphere { radius_mm: 2. }, 0.1, &limits).unwrap();
    let sphere = om9_kernel::transform_ops::transform_brep(&sphere, &Transform::translation([20., 0., 0.]), 0.1, &limits).unwrap();
    let BooleanOutcome::Body(shape) = boolean(&box_shape, &sphere, BooleanOp::Union, 0.1, &limits).unwrap() else { panic!("compound expected") };
    let mut document = Document::new("late edge refusal");
    let entity = Entity::new("box and sphere", Geometry::Brep(shape), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let seed = Controller::new(document, limits).unwrap();
    let mut c = reopen_controller(&seed, limits);
    let Geometry::Brep(shape) = &c.document().entities[&id].geometry else { panic!() };
    let mut admissions = 0;
    let result = om9_kernel::editing::all_source_edges_nurbs_admitted(shape, &limits, |_, pending| {
        if pending > 0 { admissions += 1; }
        Ok(())
    });
    assert_eq!(result.unwrap().len(), 13);
    assert!(admissions >= 3);
    c.select_objects(&[id]).unwrap(); let before = c.document().entities.clone();
    c.submit("ExtractWireframe").unwrap();
    assert_eq!(c.document().entities.values().filter(|e| e.label == "ExtractWire").count(),13);
    let after = c.document().entities.clone(); c.undo().unwrap(); assert_eq!(c.document().entities,before);
    c.redo().unwrap(); assert_eq!(c.document().entities,after);
}

#[test]
fn extract_wireframe_reopened_multiple_boxes_is_one_transaction() {
    use om9_core::{evaluation::{evaluate_curve, Parameter}, geometry::Transform};
    let limits = Limits::default();
    let body = creation::primitive(creation::Primitive::Box { x_mm: 4., y_mm: 5., z_mm: 6. },0.1,&limits).unwrap();
    for menu in [false, true] {
        let first = Entity::new("first",Geometry::Brep(body.clone()),None);
        let mut second = Entity::new("second",Geometry::Brep(body.clone()),None);
        second.placement = Transform::translation([20., 0., 0.]);
        let ids = [first.id,second.id];
        let mut document = Document::new("multiple reopened boxes");
        document.entities.insert(first.id,first); document.entities.insert(second.id,second);
        let seed = Controller::new(document,limits).unwrap();
        let mut c = reopen_controller(&seed,limits);
        c.set_current_layer(om9_app::layer_command::LayerChoice::Slot(om9_core::layers::PresetSlot::new(5).unwrap())).unwrap();
        let layer = c.document().current_layer;
        c.select_objects(&ids).unwrap(); let before = c.document().entities.clone();
        let undo_before = c.coordinator().undo_len();
        if menu { c.dispatch("OM9-CURVE-054", om9_app::session::InvocationOrigin::Menu).unwrap(); }
        else { c.submit("ExtractWireframe").unwrap(); }
        let curves: Vec<_> = c.document().entities.values().filter(|e| e.label == "ExtractWire").collect();
        assert_eq!(curves.len(),24);
        let mut counts = [0, 0];
        for entity in curves {
            assert_eq!(entity.attributes.layer, layer);
            let Geometry::Curve(curve) = &entity.geometry else { panic!("curve") };
            let p = evaluate_curve(curve, Parameter::Fraction(0.5), 0, Transform::identity(), &limits).unwrap()[0];
            let group = if p[0] <= 4. { 0 } else { 1 };
            assert!(if group == 0 { p[0] >= 0. } else { (20. ..=24.).contains(&p[0]) });
            counts[group] += 1;
        }
        assert_eq!(counts,[12,12]);
        assert_eq!(c.coordinator().undo_len(),undo_before + 1);
        let after = c.document().entities.clone(); c.undo().unwrap(); assert_eq!(c.document().entities,before);
        c.redo().unwrap(); assert_eq!(c.document().entities,after);
    }
}

#[test]
fn extract_wireframe_multiple_reopened_box_and_sphere_are_atomic() {
    let limits = Limits::default();
    let mut first = Entity::new("box", Geometry::Brep(creation::primitive(
        creation::Primitive::Box { x_mm: 4., y_mm: 5., z_mm: 6. }, 0.1, &limits).unwrap()), None);
    let mut second = Entity::new("sphere", Geometry::Brep(creation::primitive(
        creation::Primitive::Sphere { radius_mm: 3. }, 0.1, &limits).unwrap()), None);
    if first.id > second.id { std::mem::swap(&mut first.id, &mut second.id); }
    let ids = [first.id, second.id];
    let mut document = Document::new("multi source sphere");
    document.entities.insert(first.id, first); document.entities.insert(second.id, second);
    let seed = Controller::new(document, limits).unwrap();
    let mut c = reopen_controller(&seed, limits);
    let Geometry::Brep(box_source) = &c.document().entities[&ids[0]].geometry else { panic!("box B-rep") };
    assert_eq!(om9_kernel::editing::all_source_edges_nurbs_admitted(box_source, &limits, |_, _| Ok(())).unwrap().len(), 12);
    assert!(ids[0] < ids[1], "BTreeSet visits the valid box first");
    c.select_objects(&ids).unwrap(); let before = c.document().entities.clone();
    c.submit("ExtractWireframe").unwrap();
    assert_eq!(c.document().entities.values().filter(|e| e.label == "ExtractWire").count(),13);
    let after = c.document().entities.clone(); c.undo().unwrap(); assert_eq!(c.document().entities,before);
    c.redo().unwrap(); assert_eq!(c.document().entities,after);
}

#[test]
fn extract_wireframe_multiple_reopened_box_and_cylinder_shell_preserves_edges() {
    use om9_core::{evaluation::{evaluate_curve, Parameter}, geometry::Transform};
    let limits = Limits::default();
    let box_body = creation::primitive(creation::Primitive::Box { x_mm: 4., y_mm: 5., z_mm: 6. }, 0.1, &limits).unwrap();
    let circle = om9_core::curve_build::circle_on_frame([0.; 3], [1.,0.,0.], [0.,1.,0.], 2.).unwrap();
    let shell = om9_kernel::extrusion::shell(&circle, [0.,0.,5.], 0.1, &limits).unwrap();
    for (profile, menu) in [(0.5, false), (0.01, true)] {
        let a = Entity::new("box", Geometry::Brep(om9_kernel::brep::read_brep(box_body.source(), profile, &limits).unwrap()), None);
        let mut b = Entity::new("cylinder", Geometry::Brep(om9_kernel::brep::read_brep(shell.source(), profile, &limits).unwrap()), None);
        b.placement = Transform::translation([20., 0., 0.]);
        let ids = [a.id, b.id]; let mut doc = Document::new("mixed wireframe roots");
        doc.entities.insert(a.id, a); doc.entities.insert(b.id, b);
        let seed = Controller::new(doc, limits).unwrap(); let mut c = reopen_controller(&seed, limits);
        c.select_objects(&ids).unwrap(); let before = c.document().entities.clone();
        if menu { c.dispatch("OM9-CURVE-054", om9_app::session::InvocationOrigin::Menu).unwrap(); }
        else { c.submit("ExtractWireframe").unwrap(); }
        let curves: Vec<_> = c.document().entities.values().filter(|e| e.label == "ExtractWire").collect();
        assert_eq!(curves.len(), 15);
        let mut closed = 0; let mut seam = 0; let mut box_edges = 0;
        for entity in curves {
            let Geometry::Curve(curve) = &entity.geometry else { panic!("curve") };
            let points: Vec<_> = [0., 0.13, 0.37, 0.61, 0.91, 1.].iter().map(|t|
                evaluate_curve(curve, Parameter::Fraction(*t), 0, Transform::identity(), &limits).unwrap()[0]).collect();
            if points.iter().all(|p| p[0] <= 4.) { box_edges += 1; }
            else if points.iter().all(|p| ((p[0]-20.).hypot(p[1])-2.).abs()<1e-8 && (p[2]-points[0][2]).abs()<1e-8) { closed += 1; }
            else if points.iter().all(|p| (p[0]-points[0][0]).abs()<1e-8 && (p[1]-points[0][1]).abs()<1e-8)
                && (points[0][2]-points[5][2]).abs()>4.9 { seam += 1; }
        }
        assert_eq!((box_edges, closed, seam), (12, 2, 1));
        let after = c.document().entities.clone();
        let saved = reopen_controller(&c, limits);
        for (id, entity) in &after {
            assert_eq!(saved.document().entities[id].geometry, entity.geometry);
            assert_eq!(saved.document().entities[id].placement, entity.placement);
        }
        c.undo().unwrap(); assert_eq!(c.document().entities, before);
        c.redo().unwrap(); assert_eq!(c.document().entities, after);
    }
}

#[test]
fn extract_wireframe_multiple_affine_cylinder_and_box_match_world_geometry() {
    use om9_core::{evaluation::{evaluate_curve, Parameter}, geometry::Transform};
    let limits = Limits::default();
    let block = creation::primitive(creation::Primitive::Box { x_mm: 4., y_mm: 5., z_mm: 6. },0.1,&limits).unwrap();
    let circle = om9_core::curve_build::circle_on_frame([0.;3], [1.,0.,0.], [0.,1.,0.], 2.).unwrap();
    let shell = om9_kernel::extrusion::shell(&circle, [0.,0.,5.], 0.1, &limits).unwrap();
    let placement = Transform([[2.,0.5,0.,20.],[0.,1.,0.,0.],[0.,0.,1.,0.],[0.,0.,0.,1.]]);
    for (profile, menu) in [(0.5, true), (0.01, false)] {
        let a = Entity::new("box",Geometry::Brep(om9_kernel::brep::read_brep(block.source(),profile,&limits).unwrap()),None);
        let mut b = Entity::new("shell",Geometry::Brep(om9_kernel::brep::read_brep(shell.source(),profile,&limits).unwrap()),None);
        b.placement = placement;
        let ids = [a.id,b.id]; let mut doc = Document::new("affine mixed wireframe");
        doc.entities.insert(a.id,a); doc.entities.insert(b.id,b);
        let seed = Controller::new(doc,limits).unwrap(); let mut c = reopen_controller(&seed,limits);
        c.select_objects(&ids).unwrap(); let before = c.document().entities.clone();
        if menu { c.dispatch("OM9-CURVE-054", om9_app::session::InvocationOrigin::Menu).unwrap(); }
        else { c.submit("ExtractWireframe").unwrap(); }
        let mut box_count=0; let mut circular=0; let mut seam=0;
        for entity in c.document().entities.values().filter(|e| e.label=="ExtractWire") {
            let Geometry::Curve(curve) = &entity.geometry else { panic!("curve") };
            if entity.placement == Transform::identity() { box_count+=1; continue; }
            assert_eq!(entity.placement,placement);
            let points: Vec<_> = [0.,0.13,0.37,0.61,0.91,1.].iter().map(|t|
                evaluate_curve(curve,Parameter::Fraction(*t),0,entity.placement,&limits).unwrap()[0]).collect();
            assert!(points.iter().all(|p| (((p[0]-20.-0.5*p[1])/2.).powi(2)+p[1].powi(2)-4.).abs()<1e-8));
            if points.iter().all(|p| (p[2]-points[0][2]).abs()<1e-8) { circular+=1; }
            else if (points[0][2]-points[5][2]).abs()>4.9 { seam+=1; }
        }
        assert_eq!((box_count,circular,seam),(12,2,1));
        let after = c.document().entities.clone(); let saved = reopen_controller(&c,limits);
        for (id, entity) in &after { assert_eq!(saved.document().entities[id].geometry,entity.geometry); assert_eq!(saved.document().entities[id].placement,entity.placement); }
        c.undo().unwrap(); assert_eq!(c.document().entities,before);
        c.redo().unwrap(); assert_eq!(c.document().entities,after);
    }
}

#[test]
fn extract_wireframe_reopened_affine_circle_extrudes_in_world_direction() {
    use om9_core::{evaluation::{evaluate_curve, Parameter}, geometry::Transform};
    let limits = Limits::default();
    let circle = om9_core::curve_build::circle_on_frame([0.;3], [1.,0.,0.], [0.,1.,0.], 2.).unwrap();
    let shell = om9_kernel::extrusion::shell(&circle,[0.,0.,5.],0.1,&limits).unwrap();
    let placement = Transform([[2.,0.5,0.25,20.],[0.,1.,0.,-3.],[0.,0.,2.,4.],[0.,0.,0.,1.]]);
    for (profile, menu, height) in [(0.5, true, 5.), (0.01, false, -5.)] {
        let (seed,id) = attested_controller(shell.source(),profile,placement,limits);
        let mut c = reopen_controller(&seed,limits); c.select_objects(&[id]).unwrap();
        c.submit("ExtractWireframe").unwrap();
        let mut reopened = reopen_controller(&c,limits);
        let curve_id = reopened.document().entities.values().find_map(|entity| {
            if entity.label != "ExtractWire" { return None; }
            let Geometry::Curve(curve) = &entity.geometry else { return None; };
            let p = evaluate_curve(curve,Parameter::Fraction(0.),0,entity.placement,&limits).ok()?[0];
            let q = evaluate_curve(curve,Parameter::Fraction(0.5),0,entity.placement,&limits).ok()?[0];
            ((p[2]-4.).abs()<1e-8 && (q[2]-4.).abs()<1e-8 && (p[0]-q[0]).abs()>1.).then_some(entity.id)
        }).expect("lower extracted closed curve");
        reopened.select_objects(&[curve_id]).unwrap(); let before = reopened.document().entities.clone();
        if menu { reopened.dispatch("OM9-SOLID-009",om9_app::session::InvocationOrigin::Menu).unwrap(); reopened.submit(&height.to_string()).unwrap(); }
        else { reopened.submit(&format!("ExtrudeCrv {height}")).unwrap(); }
        let output = reopened.document().entities.iter().find(|(id,_)| !before.contains_key(id)).expect("extruded solid").1;
        let Geometry::Brep(body) = &output.geometry else { panic!("B-rep") };
        assert_eq!(output.placement,placement);
        let measure = creation::measure(body,&limits).unwrap();
        assert!(measure.closed && measure.solids == 1);
        assert!((measure.volume_mm3 * 4. - 40.*std::f64::consts::PI).abs() < 1e-5,
            "world volume {}", measure.volume_mm3 * 4.);
        let bounds = om9_kernel::brep::world_bounds(body,&output.placement,&limits).unwrap();
        let (zmin,zmax) = if height > 0. { (4.,9.) } else { (-1.,4.) };
        assert!((bounds[2]-zmin).abs()<1e-6 && (bounds[5]-zmax).abs()<1e-6, "{bounds:?}");
        let after = reopened.document().entities.clone(); reopened.undo().unwrap(); assert_eq!(reopened.document().entities,before);
        reopened.redo().unwrap(); assert_eq!(reopened.document().entities,after);
    }
}

#[test]
fn extract_wireframe_multiple_roots_budget_refusal_preserves_redo() {
    let limits = Limits { max_objects: 20, ..Limits::default() };
    let body = creation::primitive(creation::Primitive::Box { x_mm: 4., y_mm: 5., z_mm: 6. },0.1,&limits).unwrap();
    let first = Entity::new("first", Geometry::Brep(body.clone()), None);
    let mut second = Entity::new("second", Geometry::Brep(body), None);
    second.placement = om9_core::geometry::Transform::scale([2., 1., 1.]).unwrap();
    let ids = [first.id, second.id]; let mut document = Document::new("multi output limit");
    document.entities.insert(first.id, first); document.entities.insert(second.id, second);
    let seed = Controller::new(document, limits).unwrap(); let mut c = reopen_controller(&seed, limits);
    c.submit("Point").unwrap(); c.submit("1,2,3").unwrap(); c.undo().unwrap();
    c.select_objects(&ids).unwrap(); let before = c.coordinator().snapshot();
    assert!(c.submit("ExtractWireframe").is_err());
    assert_eq!(c.coordinator().snapshot(), before); assert_eq!(c.coordinator().redo_len(), 1);
}

#[test]
fn extract_wireframe_multiple_roots_memory_refusal_can_retry() {
    use om9_core::budget::memory::MemoryClass;
    let limits = Limits { max_bytes: 64 * 1024 * 1024, ..Limits::default() };
    let body = creation::primitive(creation::Primitive::Box { x_mm: 4., y_mm: 5., z_mm: 6. },0.1,&limits).unwrap();
    let first = Entity::new("first", Geometry::Brep(body.clone()), None);
    let mut second = Entity::new("second", Geometry::Brep(body), None);
    second.placement = om9_core::geometry::Transform::scale([2., 1., 1.]).unwrap();
    let ids = [first.id, second.id]; let mut document = Document::new("multi memory limit");
    document.entities.insert(first.id, first); document.entities.insert(second.id, second);
    let seed = Controller::new(document, limits).unwrap(); let mut c = reopen_controller(&seed, limits);
    c.submit("Point").unwrap(); c.submit("1,2,3").unwrap(); c.undo().unwrap();
    c.select_objects(&ids).unwrap(); let before = c.coordinator().snapshot();
    let pressure = c.coordinator().reserve_remaining_external_memory_with_held(
        MemoryClass::Kernel, None, c.scene_cache_usage().unwrap()).unwrap();
    assert!(c.submit("ExtractWireframe").is_err());
    assert_eq!(c.coordinator().snapshot(), before); assert_eq!(c.coordinator().redo_len(), 1);
    drop(pressure); c.submit("ExtractWireframe").unwrap();
    assert_eq!(c.document().entities.values().filter(|e| e.label == "ExtractWire").count(), 24);
}

fn reopen_controller(c: &Controller, limits: Limits) -> Controller {
    let path = std::env::temp_dir().join(format!("om9-wireframe-reopen-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, c.document(), &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    let reopened = Controller::from_file(&path, limits).unwrap();
    std::fs::remove_file(path).unwrap();
    reopened
}

fn attested_controller(source: &str, deflection: f64, placement: om9_core::geometry::Transform,
    limits: Limits) -> (Controller, om9_core::identity::ObjectId) {
    let captured = om9_kernel::brep::capture_source_topology(source, deflection, 0.3, &limits, |_, _| Ok(())).unwrap();
    let (body, proof) = captured.into_parts();
    let mut entity = Entity::new("source", Geometry::Brep(body), None);
    entity.placement = placement;
    let id = entity.id;
    let mut document = Document::new("DupEdge exact");
    document.entities.insert(id, entity);
    let mut c = Controller::new(document, limits).unwrap();
    let bytes = proof.storage_bytes().unwrap();
    let mut lease = c.coordinator().reserve_external_memory_with_held(
        om9_core::budget::memory::MemoryClass::CpuCache, bytes,
        om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes)),
        c.scene_cache_usage().unwrap()).unwrap();
    lease.set_allocated_bytes(bytes).unwrap();
    c.adopt_topology_proof(c.document().stamp(id).unwrap(), proof, lease).unwrap();
    (c, id)
}

fn select_edges(c: &mut Controller, id: om9_core::identity::ObjectId, indices: &[usize]) {
    let Geometry::Brep(shape) = &c.document().entities[&id].geometry else { panic!() };
    let keys: Vec<_> = indices.iter().map(|i| shape.topology().edges[*i]).collect();
    for key in keys { c.select_subobject(id, om9_scene::scene::Subelement::BrepEdge(key)).unwrap(); }
}

#[test]
fn dupedge_source_circle_menu_cmd_profiles_placement_and_one_undo() {
    use om9_core::{evaluation::{evaluate_curve, Parameter}, geometry::{Curve, Transform}};
    let limits = Limits::default();
    let circle = om9_core::curve_build::circle_on_frame([0.; 3], [1.,0.,0.], [0.,1.,0.], 2.).unwrap();
    let seed = om9_kernel::extrusion::shell(&circle, [0.,0.,5.], 0.1, &limits).unwrap();
    let mut outputs = Vec::new();
    for (profile, menu) in [(0.5, false), (0.01, true)] {
        let mut placement = Transform::identity();
        placement.0[0] = [0., -1., 0., 10.];
        placement.0[1] = [1., 0., 0., -4.];
        placement.0[2][3] = 3.;
        let (mut c, id) = attested_controller(seed.source(), profile, placement, limits);
        let Geometry::Brep(shape) = &c.document().entities[&id].geometry else { panic!() };
        let indices: Vec<_> = (0..shape.topology().edges.len()).filter(|i|
            om9_kernel::editing::oriented_edge_endpoints_or_closed(shape, *i, &limits).unwrap().is_none()).collect();
        assert_eq!(indices.len(), 2);
        select_edges(&mut c, id, &indices);
        let before = c.coordinator().undo_len();
        if menu { c.dispatch("OM9-CURVE-040", om9_app::session::InvocationOrigin::Menu).unwrap(); }
        else { c.submit("DupEdge").unwrap(); }
        assert_eq!(c.coordinator().undo_len(), before + 1);
        let mut curves: Vec<_> = c.document().entities.values().filter_map(|e|
            match &e.geometry { Geometry::Curve(curve @ Curve::Nurbs(_)) if e.label == "DupEdge" => Some(curve.clone()), _ => None }).collect();
        assert_eq!(curves.len(), 2);
        let eval = |curve: &Curve, t| evaluate_curve(curve, Parameter::Fraction(t), 0, Transform::identity(), &limits).unwrap()[0];
        curves.sort_by(|a,b| eval(a,0.)[2].total_cmp(&eval(b,0.)[2]));
        for (curve,z) in curves.iter().zip([3.,8.]) {
            for t in [0.,0.13,0.37,0.61,0.91,1.] {
                let p = eval(curve,t);
                assert!(((p[0]-10.).hypot(p[1]+4.)-2.).abs() < 1e-8, "{p:?}");
                assert!((p[2]-z).abs() < 1e-8);
            }
        }
        c.undo().unwrap(); assert_eq!(c.document().entities.len(), 1);
        c.redo().unwrap(); assert_eq!(c.document().entities.len(), 3);
        outputs.push(curves);
    }
    assert_eq!(outputs[0], outputs[1]);
}

#[test]
fn dupedge_commit_refusal_preserves_all_outputs_and_redo() {
    let limits = Limits { max_objects: 3, ..Limits::default() };
    let seed = creation::primitive(creation::Primitive::Box { x_mm: 4., y_mm: 5., z_mm: 6. }, 0.1, &limits).unwrap();
    let (mut c, id) = attested_controller(seed.source(), 0.1, om9_core::geometry::Transform::identity(), limits);
    c.submit("Point").unwrap(); c.submit("1,2,3").unwrap(); c.undo().unwrap();
    select_edges(&mut c, id, &[0,1,2]);
    let before = c.coordinator().snapshot();
    assert!(c.submit("DupEdge").is_err());
    assert_eq!(c.coordinator().snapshot(), before);
    assert_eq!(c.coordinator().redo_len(), 1);
    assert_eq!(c.subselection().len(), 3);
}

#[test]
fn dupedge_rejects_unattested_legacy_edges_without_writing() {
    let limits = Limits::default();
    let body = creation::primitive(creation::Primitive::Box { x_mm: 4., y_mm: 5., z_mm: 6. }, 0.1, &limits).unwrap();
    let entity = Entity::new("legacy", Geometry::Brep(body), None); let id = entity.id;
    let mut doc = Document::new("legacy"); doc.entities.insert(id,entity);
    let mut c = Controller::new(doc, limits).unwrap(); select_edges(&mut c,id,&[0]);
    let before = c.coordinator().snapshot();
    assert!(matches!(c.submit("DupEdge"), Err(om9_core::Error::Unsupported(_))));
    assert_eq!(c.coordinator().snapshot(), before);
}

#[test]
fn dupedge_reopened_product_box_refuses_without_proof_or_mutation() {
    let limits = Limits::default();
    let mut original = Controller::new(Document::new("reopened Box"), limits).unwrap();
    original.submit("Box 4 5 6").unwrap();
    let id = *original.document().entities.keys().next().unwrap();
    let path = std::env::temp_dir().join(format!("om9-dupedge-reopen-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, original.document(), &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    let mut reopened = Controller::from_file(&path, limits).unwrap();
    std::fs::remove_file(path).unwrap();
    select_edges(&mut reopened, id, &[0]);
    let before = reopened.coordinator().snapshot();
    let undo = reopened.coordinator().undo_len();
    let redo = reopened.coordinator().redo_len();
    assert!(matches!(reopened.submit("DupEdge"), Err(om9_core::Error::Unsupported(_))));
    assert_eq!(reopened.coordinator().snapshot(), before);
    assert_eq!(reopened.coordinator().undo_len(), undo);
    assert_eq!(reopened.coordinator().redo_len(), redo);
}

#[test]
fn reopened_box_source_projection_distinguishes_two_edge_midpoints() {
    let limits = Limits::default();
    let mut c = Controller::new(Document::new("projection after open"), limits).unwrap();
    c.submit("Box 4 5 6").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    let path = std::env::temp_dir().join(format!("om9-dupedge-projection-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, c.document(), &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    let reopened = Controller::from_file(&path, limits).unwrap();
    std::fs::remove_file(path).unwrap();
    let Geometry::Brep(shape) = &reopened.document().entities[&id].geometry else { panic!() };
    for point in [[2., 0., 0.], [0., 2.5, 0.]] {
        let mut distances = Vec::new();
        for index in 0..shape.topology().edges.len() {
            let (_, projected) = om9_kernel::editing::fillet_project_edge(shape, index, point, &limits).unwrap();
            let distance = (0..3).map(|i| (projected[i] - point[i]).powi(2)).sum::<f64>().sqrt();
            distances.push(distance);
        }
        distances.sort_by(f64::total_cmp);
        assert!(distances[0] < 1e-9);
        assert!(distances[1] > 1.0, "the source edge must be distinguishable");
        let (curve, projected) = om9_kernel::editing::source_edge_curve_at_point_admitted(
            shape, point, 0.001, &limits, |_, _| Ok(()),
        ).unwrap();
        assert!((0..3).all(|axis| (projected[axis] - point[axis]).abs() < 1e-8));
        let middle = om9_core::evaluation::evaluate_curve(&curve,
            om9_core::evaluation::Parameter::Fraction(0.5), 0,
            om9_core::geometry::Transform::identity(), &limits).unwrap()[0];
        assert!((0..3).all(|axis| (middle[axis] - point[axis]).abs() < 1e-8));
    }
    let corner = [0., 0., 0.];
    let mut matching_edges = 0;
    for index in 0..shape.topology().edges.len() {
        let (_, projected) = om9_kernel::editing::fillet_project_edge(shape, index, corner, &limits).unwrap();
        let distance = (0..3).map(|axis| (projected[axis] - corner[axis]).powi(2)).sum::<f64>().sqrt();
        if distance < 1e-9 { matching_edges += 1; }
    }
    assert_eq!(matching_edges, 3, "a Box corner belongs to three source edges");
    let segment = [[0., 0., 0.], [4., 0., 0.]];
    let (resolved, _) = om9_kernel::editing::source_edge_curve_at_segment_admitted(
        shape, [0.1, 0.1, 0.], 0.15, segment, 1e-6, &limits, |_, _| Ok(()),
    ).unwrap();
    let middle = om9_core::evaluation::evaluate_curve(&resolved,
        om9_core::evaluation::Parameter::Fraction(0.5), 0,
        om9_core::geometry::Transform::identity(), &limits).unwrap()[0];
    assert!((middle[0] - 2.).abs() < 1e-8 && middle[1].abs() < 1e-8 && middle[2].abs() < 1e-8);
    assert!(om9_kernel::editing::source_edge_curve_at_segment_admitted(
        shape, [0.1, 0.1, 0.], 0.15, [[0., 0., 0.], [4., 0.01, 0.]], 1e-6, &limits, |_, _| Ok(()),
    ).is_err(), "altered segment must not establish a source edge");
    assert!(om9_kernel::editing::source_edge_curve_at_segment_admitted(
        shape, [0.1, 0.1, 0.], 0.15, [[0., 0., 0.], [0., 0., 0.]], 1e-6, &limits, |_, _| Ok(()),
    ).is_err(), "a point alone is not a segment");
    assert!(om9_kernel::editing::source_edge_curve_at_point_admitted(
        shape, corner, 0.001, &limits, |_, _| Ok(()),
    ).is_err());
    assert!(om9_kernel::editing::source_edge_curve_at_point_admitted(
        shape, [20., 20., 20.], 0.001, &limits, |_, _| Ok(()),
    ).is_err());
    assert!(om9_kernel::editing::source_edge_curve_at_point_admitted(
        shape, [2., 0., 0.], 0.001, &limits, |_, _| Err(om9_core::Error::Budget("refused")),
    ).is_err());
}

#[test]
#[ignore = "isolated DupEdge source-adapter process memory diagnostic"]
#[cfg(windows)]
fn dupedge_source_adapter_process_memory_probe() {
    let limits = Limits::default();
    let shape = creation::primitive(creation::Primitive::Box { x_mm: 4., y_mm: 5., z_mm: 6. }, 0.01, &limits).unwrap();
    let before = om9_native_owner::process_memory().expect("process counters");
    let start = std::time::Instant::now();
    let iterations = 128;
    for _ in 0..iterations {
        let (curve, point) = om9_kernel::editing::source_edge_curve_at_segment_admitted(
            &shape, [2., 0., 0.], 0.01, [[0., 0., 0.], [4., 0., 0.]], 1e-6, &limits, |_, _| Ok(()),
        ).unwrap();
        assert!((point[0] - 2.).abs() < 1e-8 && point[1].abs() < 1e-8 && point[2].abs() < 1e-8);
        let midpoint = om9_core::evaluation::evaluate_curve(&curve, om9_core::evaluation::Parameter::Fraction(0.5),
            0, om9_core::geometry::Transform::identity(), &limits).unwrap()[0];
        assert!((midpoint[0] - 2.).abs() < 1e-8 && midpoint[1].abs() < 1e-8 && midpoint[2].abs() < 1e-8);
    }
    let elapsed = start.elapsed();
    let after = om9_native_owner::process_memory().expect("process counters");
    println!("DUPEDGE_PROCESS_MEMORY iterations={iterations} elapsed_ms={:.3} before_private={} before_peak_pagefile={} before_working={} before_peak_working={} after_private={} after_peak_pagefile={} after_working={} after_peak_working={}",
        elapsed.as_secs_f64() * 1000., before.private_bytes, before.peak_pagefile_bytes, before.working_set_bytes,
        before.peak_working_set_bytes, after.private_bytes, after.peak_pagefile_bytes, after.working_set_bytes, after.peak_working_set_bytes);
}

#[test]
#[ignore = "isolated ExtractWireframe process memory diagnostic"]
#[cfg(windows)]
fn extract_wireframe_reopened_multiple_cylinders_process_memory_probe() {
    use om9_core::geometry::Transform;
    let limits=Limits::default();
    let body=creation::primitive(creation::Primitive::Cylinder {radius_mm:2.,height_mm:5.},0.01,&limits).unwrap();
    let mut doc=Document::new("wireframe memory sample"); let mut ids=Vec::new();
    for i in 0..8 {
        let mut entity=Entity::new("cylinder",Geometry::Brep(body.clone()),None);
        entity.placement=Transform::translation([10.*i as f64,0.,0.]); ids.push(entity.id); doc.entities.insert(entity.id,entity);
    }
    let seed=Controller::new(doc,limits).unwrap(); let mut c=reopen_controller(&seed,limits); drop(seed);
    c.select_objects(&ids).unwrap(); let before_doc=c.document().entities.clone();
    let before=om9_native_owner::process_memory().expect("process counters"); let started=std::time::Instant::now();
    c.submit("ExtractWireframe").unwrap(); let elapsed=started.elapsed();
    let after=om9_native_owner::process_memory().expect("process counters");
    assert_eq!(c.document().entities.values().filter(|e|e.label=="ExtractWire").count(),24);
    let after_doc=c.document().entities.clone(); c.undo().unwrap(); assert_eq!(c.document().entities,before_doc);
    c.redo().unwrap(); assert_eq!(c.document().entities,after_doc);
    println!("EXTRACTWIRE_PROCESS_MEMORY roots=8 curves=24 deflection_mm=0.01 elapsed_ms={:.3} before_private={} before_peak_pagefile={} before_working={} before_peak_working={} after_private={} after_peak_pagefile={} after_working={} after_peak_working={}",
        elapsed.as_secs_f64()*1000.,before.private_bytes,before.peak_pagefile_bytes,before.working_set_bytes,before.peak_working_set_bytes,
        after.private_bytes,after.peak_pagefile_bytes,after.working_set_bytes,after.peak_working_set_bytes);
}

#[test]
fn full_circle_chord_verification_respects_interior_deviation() {
    let limits = Limits::default();
    let shape = creation::primitive(creation::Primitive::Cylinder { radius_mm: 2., height_mm: 5. }, 0.01, &limits).unwrap();
    let point = |angle: f64| [2. * angle.cos(), 2. * angle.sin(), 5.];
    let check = |a: f64, b: f64, tolerance: f64| {
        om9_kernel::editing::source_edge_curve_at_segment_admitted(
            &shape, point((a + b) / 2.), tolerance, [point(a), point(b)], 1e-7, &limits, |_, _| Ok(()))
    };
    assert!(check(0.4, 0.45, 0.001).is_ok());
    assert!(check(0.4, 0.45, 0.0001).is_err(), "endpoints alone must not certify an oversized sagitta");
    assert!(check(0.4, 0.4 + std::f64::consts::PI, 0.01).is_err(), "a diameter is not a circle edge");
    let sagitta = 2. * (1. - 0.025_f64.cos());
    for step in 0..12 {
        let a = 0.2 + f64::from(step) * 0.2;
        assert!(check(a, a + 0.05, sagitta * 1.05).is_ok());
        assert!(check(a, a + 0.05, sagitta * 0.95).is_err());
    }
}

#[test]
fn trimmed_arc_source_witness_rejects_points_on_missing_arc() {
    use om9_core::geometry::Curve;
    let limits = Limits::default();
    let arc = om9_core::curve_build::arc_through([2., 0., 0.], [2f64.sqrt(), 2f64.sqrt(), 0.], [0., 2., 0.]).unwrap();
    let profile = Curve::Polycurve { segments: vec![arc,
        Curve::Line { start: [0., 2., 0.], end: [2., 0., 0.], domain: [0., 1.] }], reversed: false };
    let shape = om9_kernel::extrusion::shell(&profile, [0., 0., 5.], 0.01, &limits).unwrap();
    assert!((0..shape.topology().edges.len()).any(|i| om9_kernel::editing::circular_edge(&shape, i, &limits).is_ok()));
    let point = |angle: f64| [2. * angle.cos(), 2. * angle.sin(), 5.];
    let check = |a: f64, b: f64| om9_kernel::editing::source_edge_curve_at_segment_admitted(
        &shape, point((a + b) / 2.), 0.001, [point(a), point(b)], 1e-7, &limits, |_, _| Ok(()));
    assert!(check(0.4, 0.45).is_ok());
    assert!(check(-0.4, -0.35).is_err(), "the missing portion of a trimmed circle is not the source edge");
}

#[test]
fn circular_edge_source_witness_handles_large_coordinates() {
    use om9_core::geometry::Transform;
    let limits = Limits::default();
    let base = creation::primitive(creation::Primitive::Cylinder { radius_mm: 2., height_mm: 5. }, 0.01, &limits).unwrap();
    for origin in [1e6, 1e9] {
        let shape = om9_kernel::transform_ops::transform_brep(
            &base, &Transform::translation([origin, origin, origin]), 0.01, &limits).unwrap();
        let display = shape.display();
        let (segment, index) = display.segments.iter().filter_map(|s| {
            let a = display.points[s[0] as usize];
            let b = display.points[s[1] as usize];
            ((a[2] - (origin + 5.)).abs() < 1e-5 && (b[2] - (origin + 5.)).abs() < 1e-5
                && a[0] < origin - 0.5 && b[0] < origin - 0.5).then_some(([a, b], s[2] as usize))
        }).next().expect("cylinder ring segment");
        assert!(om9_kernel::editing::circular_edge(&shape, index, &limits).is_ok(), "source must remain analytic");
        let point = std::array::from_fn(|i| (segment[0][i] + segment[1][i]) * 0.5);
        let resolve = |segment| om9_kernel::editing::source_edge_curve_at_segment_admitted(
            &shape, point, 0.010001, segment, 1e-6, &limits, |_, _| Ok(()));
        assert!(resolve(segment).is_ok(), "origin {origin}");
        let mut bad = segment;
        bad[1][2] += 0.01;
        assert!(resolve(bad).is_err(), "shifted endpoint accepted at origin {origin}");
    }
}

#[test]
fn source_edge_segment_verification_budget_refuses_without_guessing() {
    let limits = Limits::default();
    let shape = creation::primitive(creation::Primitive::Cylinder { radius_mm: 2., height_mm: 5. }, 0.01, &limits).unwrap();
    let angle: f64 = 0.0001;
    let segment = [[2., 0., 5.], [2. * angle.cos(), 2. * angle.sin(), 5.]];
    let point = [2. * (angle / 2.).cos(), 2. * (angle / 2.).sin(), 5.];
    let constrained = Limits { max_nodes: 64, ..limits };
    let result = om9_kernel::editing::source_edge_curve_at_segment_admitted(
        &shape, point, 1e-8, segment, 1e-9, &constrained, |_, _| Ok(()),
    );
    assert!(format!("{result:?}").contains("segment verification budget"), "unexpected result: {result:?}");
}

#[test]
#[ignore = "isolated representative curved-edge adapter timing"]
fn dupedge_curved_segment_timing_probe() {
    let limits = Limits::default();
    let shape = creation::primitive(creation::Primitive::Cylinder { radius_mm: 2., height_mm: 5. }, 0.01, &limits).unwrap();
    let display = shape.display();
    let segment = display.segments.iter().find_map(|s| {
        let a = display.points[s[0] as usize]; let b = display.points[s[1] as usize];
        if (a[2] - 5.).abs() < 1e-8 && (b[2] - 5.).abs() < 1e-8
            && a[0] > 0.5 && b[0] > 0.5 && a[1] > 0.5 && b[1] > 0.5 { Some([a, b]) } else { None }
    }).expect("top circular display segment");
    let point = std::array::from_fn(|i| (segment[0][i] + segment[1][i]) * 0.5);
    let start = std::time::Instant::now();
    for _ in 0..32 {
        let (curve, _) = om9_kernel::editing::source_edge_curve_at_segment_admitted(
            &shape, point, 0.010001, segment, 1e-6, &limits, |_, _| Ok(()),
        ).unwrap();
        let sample = om9_core::evaluation::evaluate_curve(&curve, om9_core::evaluation::Parameter::Fraction(0.5),
            0, om9_core::geometry::Transform::identity(), &limits).unwrap()[0];
        assert!((sample[0].hypot(sample[1]) - 2.).abs() < 1e-7 && (sample[2] - 5.).abs() < 1e-7);
    }
    println!("DUPEDGE_CURVE_TIMING iterations=32 elapsed_ms={:.3}", start.elapsed().as_secs_f64() * 1000.);
}

#[test]
fn source_edge_segment_endpoints_must_respect_full_segment_tolerance() {
    let limits = Limits::default();
    let shape = creation::primitive(creation::Primitive::Box { x_mm: 4., y_mm: 5., z_mm: 6. }, 0.01, &limits).unwrap();
    let result = om9_kernel::editing::source_edge_curve_at_segment_admitted(
        &shape, [2., 0., 0.], 0.0001, [[0., 0.001, 0.], [4., 0.001, 0.]], 0.002, &limits, |_, _| Ok(()),
    );
    assert!(result.is_err(), "endpoint tolerance must not permit a segment outside the source-distance bound");
}

#[test]
fn source_edge_segment_refuses_curve_that_departs_between_quarter_samples() {
    use om9_core::geometry::{Curve, NurbsCurve};
    let limits = Limits::default();
    // y(t) = 200*t*(t-1/4)*(t-1/2)*(t-3/4)*(t-1): all old samples lie on y=0,
    // but y(1/8) = 2625/4096 mm, well beyond the 0.01 mm display bound.
    let y = [0., 3.75, -8.125, 8.125, -3.75, 0.];
    let curve = Curve::Nurbs(NurbsCurve { degree: 5,
        poles: y.into_iter().enumerate().map(|(i, value)| [i as f64 * 0.8, value, 0.]).collect(),
        weights: vec![1.; 6], knots: vec![0., 0., 0., 0., 0., 0., 1., 1., 1., 1., 1., 1.],
        domain: [0., 1.], periodic: false, reversed: false, seam: None });
    let profile = Curve::Polycurve { segments: vec![curve,
        Curve::Line { start: [4., 0., 0.], end: [4., -5., 0.], domain: [0., 1.] },
        Curve::Line { start: [4., -5., 0.], end: [0., -5., 0.], domain: [0., 1.] },
        Curve::Line { start: [0., -5., 0.], end: [0., 0., 0.], domain: [0., 1.] },
    ], reversed: false };
    let shape = om9_kernel::extrusion::shell(&profile, [0., 0., 5.], 0.01, &limits).unwrap();
    for i in 0..=32 {
    let t = i as f64 / 32.;
    let target = [4. * t, 200. * t * (t - 0.25) * (t - 0.5) * (t - 0.75) * (t - 1.), 5.];
    let nearest = shape.display().segments.iter().map(|segment| {
        let a = shape.display().points[segment[0] as usize];
        let b = shape.display().points[segment[1] as usize];
        let d = std::array::from_fn::<_, 3, _>(|i| b[i] - a[i]);
        let len2 = d.iter().map(|v| v * v).sum::<f64>();
        let t = if len2 > 0. { (0..3).map(|i| (target[i] - a[i]) * d[i]).sum::<f64>() / len2 } else { 0. }.clamp(0., 1.);
        (0..3).map(|i| (target[i] - a[i] - t * d[i]).powi(2)).sum::<f64>().sqrt()
    }).fold(f64::INFINITY, f64::min);
    assert!(nearest <= 0.01, "generated display misses source sample by {nearest} mm");
    }
    let result = om9_kernel::editing::source_edge_curve_at_segment_admitted(
        &shape, [2., 0., 5.], 0.010001, [[0., 0., 5.], [4., 0., 5.]], 1e-6, &limits, |_, _| Ok(()),
    );
    assert!(result.is_err(), "a curve that departs between the previous three samples must not match a straight segment");
}

#[test]
fn source_edge_segment_at_large_native_coordinates() {
    use om9_core::geometry::Transform;
    let limits = Limits::default();
    let base = creation::primitive(creation::Primitive::Box { x_mm: 4., y_mm: 5., z_mm: 6. }, 0.01, &limits).unwrap();
    for origin in [1e9, 1e12] {
        let shape = om9_kernel::transform_ops::transform_brep(&base, &Transform::translation([origin, origin, origin]), 0.01, &limits).unwrap();
        let display = shape.display();
        let segment = display.segments.iter().find_map(|s| {
            let a = display.points[s[0] as usize]; let b = display.points[s[1] as usize];
            if (a[1] - origin).abs() < 0.001 && (b[1] - origin).abs() < 0.001
                && (a[2] - origin).abs() < 0.001 && (b[2] - origin).abs() < 0.001
                && (a[0] - b[0]).abs() > 1. { Some([a, b]) } else { None }
        }).expect("far x edge");
        let point = std::array::from_fn(|i| (segment[0][i] + segment[1][i]) * 0.5);
        let (curve, _) = om9_kernel::editing::source_edge_curve_at_segment_admitted(
            &shape, point, 0.01, segment, 1e-6, &limits, |_, _| Ok(())).unwrap();
        let middle = om9_core::evaluation::evaluate_curve(&curve,
            om9_core::evaluation::Parameter::Fraction(0.5), 0, Transform::identity(), &limits).unwrap()[0];
        assert!((middle[0] - (origin + 2.)).abs() < 0.001 && (middle[1] - origin).abs() < 0.001 && (middle[2] - origin).abs() < 0.001);
    }
}

#[test]
fn source_arc_segment_at_large_native_coordinates() {
    use om9_core::geometry::{Curve, Transform};
    let limits = Limits::default();
    let arc = om9_core::curve_build::arc_through([2., 0., 0.], [2f64.sqrt(), 2f64.sqrt(), 0.], [0., 2., 0.]).unwrap();
    let profile = Curve::Polycurve { segments: vec![arc,
        Curve::Line { start: [0., 2., 0.], end: [2., 0., 0.], domain: [0., 1.] }], reversed: false };
    let base = om9_kernel::extrusion::shell(&profile, [0., 0., 5.], 0.1, &limits).unwrap();
    {
        let origin = 1e6;
        let shape = om9_kernel::transform_ops::transform_brep(&base, &Transform::translation([origin, origin, origin]), 0.1, &limits)
            .unwrap_or_else(|error| panic!("origin {origin}: {error}"));
        let display = shape.display();
        let segment = display.segments.iter().filter_map(|s| {
            let a = display.points[s[0] as usize]; let b = display.points[s[1] as usize];
            if (a[2] - origin - 5.).abs() > 0.001 || (b[2] - origin - 5.).abs() > 0.001 { return None; }
            let midpoint = std::array::from_fn::<_, 3, _>(|i| (a[i] + b[i]) * 0.5);
            let d = (midpoint[0] - origin - 1.847759065).powi(2) + (midpoint[1] - origin - 0.765366865).powi(2);
            Some((d, [a, b], midpoint))
        }).min_by(|a, b| a.0.total_cmp(&b.0)).unwrap();
        let (curve, _) = om9_kernel::editing::source_edge_curve_at_segment_admitted(
            &shape, segment.2, 0.2, segment.1, 1e-6, &limits, |_, _| Ok(())).unwrap();
        let middle = om9_core::evaluation::evaluate_curve(&curve,
            om9_core::evaluation::Parameter::Fraction(0.5), 0, Transform::identity(), &limits).unwrap()[0];
        assert!(((middle[0] - origin).hypot(middle[1] - origin) - 2.).abs() < 0.001);
    }
}

#[test]
fn source_edge_segment_threshold_on_small_and_large_boxes() {
    let limits = Limits::default();
    for size in [0.001, 1., 10_000.] {
        let shape = creation::primitive(creation::Primitive::Box { x_mm: size, y_mm: size, z_mm: size },
            size * 0.01, &limits).unwrap();
        let resolve = |offset| om9_kernel::editing::source_edge_curve_at_segment_admitted(
            &shape, [size * 0.5, 0., 0.], size * 0.001,
            [[0., offset, 0.], [size, offset, 0.]], 1e-6, &limits, |_, _| Ok(()));
        assert!(resolve(0.5e-6).is_ok(), "inside endpoint threshold for size {size}");
        assert!(resolve(2e-6).is_err(), "outside endpoint threshold for size {size}");
    }
}

#[test]
fn source_edge_segment_refuses_two_coincident_native_edges() {
    let limits = Limits::default();
    let shape = om9_kernel::brep::read_brep(include_str!("fixtures/dupedge-coincident.brep"), 0.01, &limits).unwrap();
    let error = om9_kernel::editing::source_edge_curve_at_segment_admitted(
        &shape, [2., 0., 0.], 0.01, [[0., 0., 0.], [4., 0., 0.]], 1e-6, &limits, |_, _| Ok(()),
    ).unwrap_err();
    assert!(error.to_string().contains("ambiguous edges"), "{error}");
}

#[test]
fn source_edge_segment_refuses_edges_inside_matching_tolerance() {
    let limits = Limits::default();
    let shape = om9_kernel::brep::read_brep(include_str!("fixtures/dupedge-near.brep"), 0.01, &limits).unwrap();
    let error = om9_kernel::editing::source_edge_curve_at_segment_admitted(
        &shape, [2., 0.00000075, 0.], 0.01,
        [[0., 0.00000075, 0.], [4., 0.00000075, 0.]], 1e-6, &limits, |_, _| Ok(()),
    ).unwrap_err();
    assert!(error.to_string().contains("ambiguous edges"), "{error}");
}

#[test]
fn source_edge_pick_handles_sphere_seam_and_refuses_degenerate_pole() {
    let limits = Limits::default();
    let sphere = creation::primitive(creation::Primitive::Sphere { radius_mm: 5. }, 0.01, &limits).unwrap();
    let (curve, point) = om9_kernel::editing::source_edge_curve_at_point_admitted(
        &sphere, [5., 0., 0.], 0.01, &limits, |_, _| Ok(()),
    ).unwrap();
    assert!((point[0] - 5.).abs() < 1e-8 && point[1].abs() < 1e-8 && point[2].abs() < 1e-8);
    for fraction in [0., 0.25, 0.5, 0.75, 1.] {
        let p = om9_core::evaluation::evaluate_curve(&curve, om9_core::evaluation::Parameter::Fraction(fraction),
            0, om9_core::geometry::Transform::identity(), &limits).unwrap()[0];
        assert!((p.iter().map(|value| value * value).sum::<f64>().sqrt() - 5.).abs() < 1e-7);
    }
    assert!(om9_kernel::editing::source_edge_curve_at_point_admitted(
        &sphere, [0., 0., 5.], 0.01, &limits, |_, _| Ok(()),
    ).is_err(), "a pick at a degenerate pole must be refused");
}

#[test]
fn dupedge_product_box_selected_line() {
    use om9_core::{evaluation::{evaluate_curve, Parameter}, geometry::Transform};
    let limits = Limits::default();
    let mut c = Controller::new(Document::new("product Box"), limits).unwrap();
    c.submit("Box 4 5 6").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    let Geometry::Brep(shape) = &c.document().entities[&id].geometry else { panic!() };
    let (a,b) = om9_kernel::editing::oriented_edge_endpoints(shape, 0, &limits).unwrap();
    select_edges(&mut c,id,&[0]);
    c.submit("DupEdge").unwrap();
    let result = c.document().entities.values().find(|e| e.label == "DupEdge").unwrap();
    let Geometry::Curve(curve) = &result.geometry else { panic!() };
    for t in [0.,0.25,0.5,1.] {
        let p = evaluate_curve(curve,Parameter::Fraction(t),0,Transform::identity(),&limits).unwrap()[0];
        assert!((0..3).all(|i| (p[i]-(a[i]+(b[i]-a[i])*t)).abs()<1e-9));
    }
    assert_eq!(c.document().entities.len(),2, "only selected edge");
}

#[test]
fn dupedge_box_edge_matches_analytic_line_for_two_displays() {
    use om9_core::{evaluation::{evaluate_curve, Parameter}, geometry::Transform};
    let limits = Limits::default();
    let seed = creation::primitive(creation::Primitive::Box { x_mm: 4., y_mm: 5., z_mm: 6. }, 0.1, &limits).unwrap();
    let mut results = Vec::new();
    for deflection in [0.5, 0.01] {
        let mut placement = Transform::identity();
        placement.0[0] = [0., -1., 0., 10.];
        placement.0[1] = [1., 0., 0., -4.];
        placement.0[2][3] = 3.;
        let (mut c, id) = attested_controller(seed.source(), deflection, placement, limits);
        let Geometry::Brep(shape) = &c.document().entities[&id].geometry else { panic!() };
        let (index, reversed) = (0..shape.topology().edges.len()).find_map(|index| {
            let (a, b) = om9_kernel::editing::oriented_edge_endpoints(shape, index, &limits).ok()?;
            let near = |p: [f64; 3], q: [f64; 3]| (0..3).all(|i| (p[i]-q[i]).abs() < 1e-9);
            if near(a,[0.,0.,0.]) && near(b,[4.,0.,0.]) { Some((index,false)) }
            else if near(b,[0.,0.,0.]) && near(a,[4.,0.,0.]) { Some((index,true)) }
            else { None }
        }).unwrap();
        select_edges(&mut c, id, &[index]);
        c.submit("DupEdge").unwrap();
        let Geometry::Curve(curve) = &c.document().entities.values().find(|e| e.label == "DupEdge").unwrap().geometry else { panic!() };
        for t in [0., 0.2, 0.5, 0.8, 1.] {
            let point = evaluate_curve(curve, Parameter::Fraction(t), 0, Transform::identity(), &limits).unwrap()[0];
            let expected = [10., -4. + 4. * if reversed { 1. - t } else { t }, 3.];
            assert!((0..3).all(|i| (point[i]-expected[i]).abs() < 1e-9), "{point:?} != {expected:?}");
        }
        results.push(curve.clone());
    }
    assert_eq!(results[0], results[1]);
}

#[test]
fn dupedge_invoke_then_pick_enter_and_cancel() {
    let mut c = Controller::new(Document::new("interactive DupEdge"), Limits::default()).unwrap();
    c.submit("Box 4 5 6").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    select_edges(&mut c, id, &[0, 1]);
    let refs = c.subselection().to_vec();
    c.clear_subselection();
    let before = c.coordinator().snapshot();
    c.dispatch("OM9-CURVE-040", om9_app::session::InvocationOrigin::Menu).unwrap();
    assert!(c.dupedge_needs_pick());
    assert!(c.submit("").is_err());
    assert!(c.dupedge_needs_pick());
    for edge in &refs { c.dupedge_pick(edge.clone()).unwrap(); }
    c.cancel();
    assert!(!c.is_running());
    assert_eq!(c.coordinator().snapshot(), before);
    c.submit("DupEdge").unwrap();
    for edge in refs { c.dupedge_pick(edge).unwrap(); }
    c.submit("").unwrap();
    assert!(!c.is_running());
    assert_eq!(c.document().entities.len(), 3);
    c.undo().unwrap();
    assert_eq!(c.document().entities.len(), 1);
}

#[test]
fn dupedge_outputs_use_current_layer() {
    use om9_app::layer_command::LayerChoice;
    use om9_core::layers::PresetSlot;
    let mut c = Controller::new(Document::new("DupEdge layer"), Limits::default()).unwrap();
    c.submit("Box 4 5 6").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    c.set_current_layer(LayerChoice::Slot(PresetSlot::new(5).unwrap())).unwrap();
    let target = c.document().current_layer;
    assert!(target.is_some());
    select_edges(&mut c, id, &[0, 1]);
    c.submit("DupEdge").unwrap();
    let outputs: Vec<_> = c.document().entities.values().filter(|e| e.label == "DupEdge").collect();
    assert_eq!(outputs.len(), 2);
    assert!(outputs.iter().all(|entity| entity.attributes.layer == target));
}

#[test]
fn dupedge_pending_commit_refusal_keeps_selection_for_retry() {
    let limits = Limits { max_objects: 3, ..Limits::default() };
    let seed = creation::primitive(creation::Primitive::Box { x_mm: 4., y_mm: 5., z_mm: 6. }, 0.1, &limits).unwrap();
    let (mut c, id) = attested_controller(seed.source(), 0.1, om9_core::geometry::Transform::identity(), limits);
    select_edges(&mut c, id, &[0, 1, 2]);
    let refs = c.subselection().to_vec();
    c.clear_subselection();
    c.submit("DupEdge").unwrap();
    for edge in &refs { c.dupedge_pick(edge.clone()).unwrap(); }
    let before = c.coordinator().snapshot();
    assert!(c.submit("").is_err());
    assert_eq!(c.coordinator().snapshot(), before);
    assert!(c.dupedge_needs_pick());
    assert_eq!(c.subselection().len(), 3);
    c.dupedge_pick(refs[2].clone()).unwrap();
    c.dupedge_pick(refs[1].clone()).unwrap();
    c.submit("").unwrap();
    assert_eq!(c.document().entities.len(), 2);
    c.undo().unwrap();
    assert_eq!(c.document().entities.len(), 1);
}

#[test]
fn dupedge_pending_rejects_other_root_without_losing_first_edge() {
    let mut c = Controller::new(Document::new("DupEdge roots"), Limits::default()).unwrap();
    c.submit("Box 4 5 6").unwrap();
    let first = *c.document().entities.keys().next().unwrap();
    c.submit("Box 6 7 8").unwrap();
    let second = *c.document().entities.keys().find(|&&id| id != first).unwrap();
    select_edges(&mut c, first, &[0]);
    select_edges(&mut c, second, &[0]);
    let refs = c.subselection().to_vec();
    c.clear_subselection();
    let before = c.coordinator().snapshot();
    c.submit("DupEdge").unwrap();
    c.dupedge_pick(refs[0].clone()).unwrap();
    assert!(c.dupedge_pick(refs[1].clone()).is_err());
    assert_eq!(c.subselection(), &refs[..1]);
    assert_eq!(c.coordinator().snapshot(), before);
    c.submit("").unwrap();
    assert_eq!(c.document().entities.len(), 3);
}

#[test]
fn dupedge_attested_nonrigid_placement_keeps_curve_and_undo() {
    let limits = Limits::default();
    let seed = creation::primitive(creation::Primitive::Box { x_mm: 4., y_mm: 5., z_mm: 6. }, 0.1, &limits).unwrap();
    let mut placement = om9_core::geometry::Transform::identity();
    placement.0[0][0] = 2.;
    let (mut c, id) = attested_controller(seed.source(), 0.1, placement, limits);
    select_edges(&mut c, id, &[0]);
    let before = c.document().entities.clone();
    c.submit("DupEdge").unwrap();
    let entity = c.document().entities.values().find(|e| e.label == "DupEdge").unwrap();
    assert_eq!(entity.placement,placement);
    let Geometry::Curve(_) = &entity.geometry else { panic!("curve") };
    let after = c.document().entities.clone(); c.undo().unwrap(); assert_eq!(c.document().entities,before);
    c.redo().unwrap(); assert_eq!(c.document().entities,after);
}

#[test]
fn dupedge_trimmed_arc_keeps_domain_geometry_and_orientation() {
    use om9_core::{evaluation::{evaluate_curve, Parameter}, geometry::{Curve, Transform}};
    let limits = Limits::default();
    let arc = om9_core::curve_build::arc_through([2.,0.,0.], [2f64.sqrt(),2f64.sqrt(),0.], [0.,2.,0.]).unwrap();
    let profile = Curve::Polycurve { segments: vec![arc,
        Curve::Line { start: [0.,2.,0.], end: [2.,0.,0.], domain: [0.,1.] }], reversed: false };
    let seed = om9_kernel::extrusion::shell(&profile,[0.,0.,5.],0.1,&limits).unwrap();
    let mut outputs = Vec::new();
    for profile in [0.5,0.01] {
        let (mut c,id) = attested_controller(seed.source(),profile,Transform::identity(),limits);
        let Geometry::Brep(shape) = &c.document().entities[&id].geometry else { panic!() };
        let (index,(a,b)) = (0..shape.topology().edges.len()).find_map(|i| {
            let pair = om9_kernel::editing::oriented_edge_endpoints(shape,i,&limits).ok()?;
            let curve = om9_kernel::editing::edge_nurbs(shape,i,&limits).ok()?;
            let mid = evaluate_curve(&curve,Parameter::Fraction(0.5),0,Transform::identity(),&limits).ok()?[0];
            (pair.0[2].abs()<1e-9 && pair.1[2].abs()<1e-9 && (mid[0].hypot(mid[1])-2.).abs()<1e-8).then_some((i,pair))
        }).unwrap();
        select_edges(&mut c,id,&[index]); c.submit("DupEdge").unwrap();
        let Geometry::Curve(curve @ Curve::Nurbs(_)) = &c.document().entities.values().find(|e| e.label=="DupEdge").unwrap().geometry else { panic!() };
        let eval = |t| evaluate_curve(curve,Parameter::Fraction(t),0,Transform::identity(),&limits).unwrap()[0];
        for (actual, expected) in [(eval(0.),a),(eval(1.),b)] {
            assert!((0..3).all(|i| (actual[i]-expected[i]).abs()<1e-8));
        }
        for t in [0.,0.17,0.38,0.73,1.] {
            let p=eval(t);
            assert!((p[0].hypot(p[1])-2.).abs()<1e-8 && p[2].abs()<1e-8);
            assert!(p[0]>=-1e-8 && p[1]>=-1e-8, "must stay on quarter arc: {p:?}");
        }
        outputs.push(curve.clone());
    }
    assert_eq!(outputs[0],outputs[1]);
}

#[test]
fn dupedge_later_degenerate_edge_failure_keeps_document_and_redo() {
    let limits=Limits::default();
    let seed=creation::primitive(creation::Primitive::Sphere { radius_mm: 5. },0.1,&limits).unwrap();
    let (mut c,id)=attested_controller(seed.source(),0.1,om9_core::geometry::Transform::identity(),limits);
    c.submit("Point").unwrap(); c.submit("1,2,3").unwrap(); c.undo().unwrap();
    let Geometry::Brep(shape)=&c.document().entities[&id].geometry else { panic!() };
    let good=(0..shape.topology().edges.len()).find(|i| om9_kernel::editing::edge_nurbs(shape,*i,&limits).is_ok()).unwrap();
    let bad=(0..shape.topology().edges.len()).find(|i| om9_kernel::editing::edge_nurbs(shape,*i,&limits).is_err()).unwrap();
    select_edges(&mut c,id,&[good,bad]);
    let before=c.coordinator().snapshot();
    assert!(c.submit("DupEdge").is_err());
    assert_eq!(c.coordinator().snapshot(),before);
    assert_eq!(c.coordinator().redo_len(),1);
}

#[test]
fn wireframe_capture_refusal_preserves_document_and_redo() {
    use om9_core::budget::memory::MemoryClass;
    let limits = Limits { max_bytes: 64 * 1024 * 1024, ..Limits::default() };
    let brep = creation::primitive(creation::Primitive::Box { x_mm: 10., y_mm: 20., z_mm: 30. },
        creation::DEFAULT_DEFLECTION_MM, &limits).unwrap();
    let mut document = Document::new("wireframe pressure");
    let entity = Entity::new("box", Geometry::Brep(brep), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut controller = Controller::new(document, limits).unwrap();
    controller.submit("Point").unwrap();
    controller.submit("1,2,3").unwrap();
    controller.undo().unwrap();
    controller.select_objects(&[id]).unwrap();
    select_edges(&mut controller, id, &[0]);
    let before = controller.coordinator().snapshot();
    let redo = controller.coordinator().redo_len();
    assert!(redo > 0);
    let pressure = controller.coordinator().reserve_remaining_external_memory_with_held(
        MemoryClass::Kernel, None, controller.scene_cache_usage().unwrap()).unwrap();
    assert!(controller.coordinator().capture_tracked_document(controller.document().id).is_err());
    assert!(controller.dispatch_wireframe("OM9-CURVE-040").is_err());
    assert_eq!(controller.coordinator().snapshot(), before);
    assert_eq!(controller.coordinator().redo_len(), redo);
    drop(pressure);
}

#[test]
fn extract_wireframe_nested_blocks_reopened_instances_and_undo() {
    use om9_core::{evaluation::{evaluate_curve, Parameter}, geometry::{Mesh, Transform}, model::Definition};
    let limits = Limits::default();
    let mut mesh = Entity::new("mesh", Geometry::Mesh(Mesh {
        vertices: vec![[0.,0.,0.],[2.,0.,0.],[0.,2.,0.]], faces: vec![vec![0,1,2]],
        normals: vec![], colors: vec![], corner_uvs: vec![],
    }), None);
    mesh.placement = Transform::translation([1.,2.,0.]);
    let mut box_member = Entity::new("box", Geometry::Brep(creation::primitive(
        creation::Primitive::Box { x_mm: 1., y_mm: 1., z_mm: 1. }, 0.1, &limits).unwrap()), None);
    box_member.placement = Transform::translation([5.,0.,0.]);
    let inner = Definition::new("inner", vec![mesh, box_member]); let inner_id = inner.id;
    let mut nested = Entity::new("nested", Geometry::Instance(inner_id), None);
    nested.placement = Transform([[2.,0.5,0.,0.],[0.,1.,0.,3.],[0.,0.,1.,0.],[0.,0.,0.,1.]]);
    let outer = Definition::new("outer", vec![nested]); let outer_id = outer.id;
    let mut first = Entity::new("first", Geometry::Instance(outer_id), None);
    first.placement = Transform::translation([10.,0.,0.]);
    let mut second = Entity::new("second", Geometry::Instance(outer_id), None);
    second.placement = Transform::translation([40.,0.,0.]);
    let ids = [first.id, second.id];
    let mut doc = Document::new("nested wire");
    doc.definitions.insert(inner_id, inner); doc.definitions.insert(outer_id, outer);
    doc.entities.insert(first.id, first); doc.entities.insert(second.id, second);
    let seed = Controller::new(doc, limits).unwrap();
    for menu in [false, true] {
        let mut c = reopen_controller(&seed, limits);
        c.select_objects(&ids).unwrap(); let before = c.document().clone();
        if menu { c.dispatch("OM9-CURVE-054", om9_app::session::InvocationOrigin::Menu).unwrap(); }
        else { c.submit("ExtractWireframe").unwrap(); }
        let outputs: Vec<_> = c.document().entities.values().filter(|e| e.label == "ExtractWire").collect();
        assert_eq!(outputs.len(), 30);
        let mut expected = Vec::new();
        for offset in [10., 40.] {
            let transform = Transform::translation([offset,0.,0.]).compose(
                Transform([[2.,0.5,0.,0.],[0.,1.,0.,3.],[0.,0.,1.,0.],[0.,0.,0.,1.]])).unwrap();
            for point in [[2.,2.,0.],[2.,3.,0.],[1.,3.,0.]] {
                expected.push(transform.point(point).unwrap());
            }
            for axis in 0..3 {
                for a in [0.,1.] { for b in [0.,1.] {
                    let mut point = match axis { 0 => [0.5,a,b], 1 => [a,0.5,b], _ => [a,b,0.5] };
                    point[0] += 5.;
                    expected.push(transform.point(point).unwrap());
                }}
            }
        }
        let mut seen = vec![false; expected.len()];
        for e in outputs {
            let Geometry::Curve(curve) = &e.geometry else { panic!("curve") };
            let p = evaluate_curve(curve, Parameter::Fraction(0.5), 0, e.placement, &limits).unwrap()[0];
            let index = expected.iter().position(|q| (0..3).all(|i| (p[i]-q[i]).abs() < 1e-7)).expect("unexpected midpoint");
            assert!(!seen[index], "duplicate edge midpoint {p:?}");
            seen[index] = true;
        }
        assert!(seen.into_iter().all(|value| value));
        let after = c.document().clone();
        assert_eq!(reopen_controller(&c, limits).document().entities, after.entities);
        c.undo().unwrap(); assert_eq!(c.document(), &before);
        c.redo().unwrap(); assert_eq!(c.document(), &after);
    }
}

#[test]
fn extract_wireframe_nested_unsupported_and_memory_preserve_redo() {
    use om9_core::{budget::memory::MemoryClass, geometry::{Curve, Mesh}, model::Definition};
    let limits = Limits::default();
    for case in ["memory", "unsupported", "hidden", "unselectable"] {
        let mesh = Entity::new("mesh", Geometry::Mesh(Mesh {
            vertices: vec![[0.,0.,0.],[2.,0.,0.],[0.,2.,0.]], faces: vec![vec![0,1,2]],
            normals: vec![], colors: vec![], corner_uvs: vec![],
        }), None);
        let mut members = vec![mesh];
        if case == "unsupported" { members.push(Entity::new("line", Geometry::Curve(Curve::Line {
            start: [0.,0.,0.], end: [1.,0.,0.], domain: [0.,1.],
        }), None)); }
        if case == "hidden" || case == "unselectable" {
            let mut hidden = members[0].clone(); hidden.id = om9_core::identity::ObjectId::new();
            if case == "hidden" { hidden.attributes.hidden = true; }
            else { hidden.attributes.selectable = false; }
            members.push(hidden);
        }
        let definition = Definition::new("block", members); let definition_id = definition.id;
        let root = Entity::new("root", Geometry::Instance(definition_id), None); let root_id = root.id;
        let mut doc = Document::new("block failure"); doc.definitions.insert(definition_id, definition); doc.entities.insert(root_id, root);
        let mut c = reopen_controller(&Controller::new(doc, limits).unwrap(), limits);
        c.submit("Point").unwrap(); c.submit("100,100,100").unwrap(); c.undo().unwrap();
        c.select_objects(&[root_id]).unwrap(); let before = c.coordinator().snapshot();
        let pressure = if case != "memory" { None } else { Some(c.coordinator().reserve_remaining_external_memory_with_held(
            MemoryClass::Kernel, None, c.scene_cache_usage().unwrap()).unwrap()) };
        assert!(c.submit("ExtractWireframe").is_err());
        assert_eq!(c.coordinator().snapshot(), before); assert_eq!(c.coordinator().redo_len(), 1);
        drop(pressure);
        if case == "memory" { c.submit("ExtractWireframe").unwrap(); }
    }
}

#[test]
fn extract_wireframe_nested_trim_uses_composed_placement() {
    use om9_core::{evaluation::{evaluate_curve, Parameter}, geometry::Transform, model::Definition,
        rhino_semantics::{set_trimmed_surface, TrimmedSurface}};
    let limits = Limits::default();
    let surface = om9_core::surface_kernel::plane([0.,0.,0.], [1.,0.,0.], [0.,1.,0.], 10., 10., &limits).unwrap();
    let outer = vec![[0.2,0.2],[0.8,0.2],[0.8,0.8],[0.2,0.8]];
    let trim = TrimmedSurface { underlying: surface.clone(), outer: outer.clone(), inners: vec![], reversed: false };
    let mut leaf = Entity::new("trim", Geometry::Surface(surface), None);
    set_trimmed_surface(&mut leaf, &trim, &limits).unwrap();
    leaf.placement = Transform::translation([2.,3.,0.]);
    let inner = Definition::new("inner", vec![leaf]); let inner_id = inner.id;
    let mut nested = Entity::new("nested", Geometry::Instance(inner_id), None);
    nested.placement = Transform([[2.,0.5,0.,0.],[0.,1.,0.,4.],[0.,0.,1.,0.],[0.,0.,0.,1.]]);
    let outer_def = Definition::new("outer", vec![nested]); let outer_id = outer_def.id;
    let mut root = Entity::new("root", Geometry::Instance(outer_id), None);
    root.placement = Transform::translation([10.,0.,0.]); let root_id = root.id;
    let mut doc = Document::new("nested trimmed wire");
    doc.definitions.insert(inner_id, inner); doc.definitions.insert(outer_id, outer_def); doc.entities.insert(root_id, root);
    let mut c = reopen_controller(&Controller::new(doc, limits).unwrap(), limits);
    c.select_objects(&[root_id]).unwrap(); let before = c.document().clone();
    c.submit("ExtractWireframe").unwrap();
    let outputs: Vec<_> = c.document().entities.values().filter(|e| e.label == "ExtractWire").collect();
    assert_eq!(outputs.len(), 4);
    let expected: Vec<_> = (0..4).map(|i| {
        let a = outer[i]; let b = outer[(i+1)%4];
        let x = (a[0]+b[0])*5. + 2.; let y = (a[1]+b[1])*5. + 3.;
        [10. + 2.*x + 0.5*y, 4. + y, 0.]
    }).collect();
    let mut seen = [false; 4];
    for e in outputs {
        let Geometry::Curve(curve) = &e.geometry else { panic!("curve") };
        let p = evaluate_curve(curve, Parameter::Fraction(0.5), 0, e.placement, &limits).unwrap()[0];
        let index = expected.iter().position(|q| (0..3).all(|i| (p[i]-q[i]).abs() < 1e-7)).unwrap();
        assert!(!seen[index]); seen[index] = true;
    }
    assert!(seen.into_iter().all(|value| value));
    let after = c.document().clone(); c.undo().unwrap(); assert_eq!(c.document(), &before);
    c.redo().unwrap(); assert_eq!(c.document(), &after);
}

#[test]
fn extract_wireframe_nested_invalid_native_after_valid_leaf_keeps_redo() {
    use om9_core::{geometry::Mesh, model::Definition};
    let limits = Limits::default();
    let shape = om9_kernel::brep::box_brep([4.,5.,6.], 0.1, &limits).unwrap();
    let mut native = om9_kernel::brep::to_3dm(&shape, 0.001, &limits).unwrap();
    native.payload = vec![1,2,3,4].into();
    let mesh = Entity::new("valid first", Geometry::Mesh(Mesh {
        vertices: vec![[0.,0.,0.],[2.,0.,0.],[0.,2.,0.]], faces: vec![vec![0,1,2]],
        normals: vec![], colors: vec![], corner_uvs: vec![],
    }), None);
    let invalid = Entity::new("bad native second", Geometry::NativeBrep(native), None);
    let def = Definition::new("mixed block", vec![mesh,invalid]); let def_id = def.id;
    let root = Entity::new("root", Geometry::Instance(def_id), None); let root_id = root.id;
    let mut doc = Document::with_template_tolerance("invalid native block");
    doc.definitions.insert(def_id, def); doc.entities.insert(root_id, root);
    let mut c = reopen_controller(&Controller::new(doc, limits).unwrap(), limits);
    c.submit("Point").unwrap(); c.submit("100,100,100").unwrap(); c.undo().unwrap();
    c.select_objects(&[root_id]).unwrap(); let before = c.coordinator().snapshot();
    assert!(c.submit("ExtractWireframe").is_err());
    assert_eq!(c.coordinator().snapshot(), before); assert_eq!(c.coordinator().redo_len(),1);
}
