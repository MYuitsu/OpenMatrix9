// SPDX-License-Identifier: LGPL-2.1-or-later
#![cfg(feature = "occt-brep")]
use om9_app::session::{Controller, InvocationOrigin};
use om9_core::{
    budget::Limits,
    geometry::{Curve, Geometry, Transform},
    model::{Document, Entity},
};
use om9_kernel::creation;

fn controller() -> Controller {
    Controller::new(Document::new("Solid review"), Limits::default()).unwrap()
}
fn volume(c: &Controller) -> f64 {
    c.document()
        .entities
        .values()
        .filter_map(|e| match &e.geometry {
            Geometry::Brep(b) => Some(
                creation::measure(b, c.coordinator().limits())
                    .unwrap()
                    .volume_mm3,
            ),
            _ => None,
        })
        .sum()
}
// Optional authored fixtures for human review; ordinary tests write no files.
fn review_fixture(c: &Controller, name: &str) {
    let Some(directory) = std::env::var_os("OM9_SOLID_REVIEW_DIRECTORY") else {
        return;
    };
    let path = std::path::PathBuf::from(directory).join(format!("{name}.om9"));
    assert!(!path.exists(), "refuse to overwrite {}", path.display());
    om9_archive::Archive::save(
        &path,
        c.document(),
        c.coordinator().limits(),
        &om9_core::transaction::Cancellation::default(),
    )
    .unwrap();
}
fn circles(count: usize) -> Controller {
    let mut doc = Document::new("Exact extrusion");
    for n in 0..count {
        let curve = om9_core::curve_build::circle_on_frame(
            [n as f64 * 20.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            3.0,
        )
        .unwrap();
        let mut entity = Entity::new("profile", Geometry::Curve(curve), None);
        entity.id = om9_core::identity::ObjectId(uuid::Uuid::from_u128(n as u128 + 1));
        doc.entities.insert(entity.id, entity);
    }
    let ids = doc.entities.keys().copied().collect::<Vec<_>>();
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&ids).unwrap();
    c
}

#[test]
fn text_menu_has_content_height_retry_and_cancel() {
    let mut c = controller();
    c.dispatch("OM9-SOLID-008", InvocationOrigin::Menu).unwrap();
    assert_eq!(c.prompt(), "Text");
    c.submit("H 2026").unwrap();
    assert_eq!(c.prompt(), "Height");
    assert!(c.submit("NaN").is_err());
    assert_eq!(c.prompt(), "Height");
    c.cancel();
    assert!(c.document().entities.is_empty());
    assert_eq!(c.coordinator().undo_len(), 0);
    c.dispatch("OM9-SOLID-008", InvocationOrigin::Menu).unwrap();
    c.submit("H").unwrap();
    c.submit("5").unwrap();
    review_fixture(&c, "Text-after");
    assert_eq!(c.document().entities.len(), 1);
    assert_eq!(c.coordinator().undo_len(), 1);
    assert!(volume(&c) > 0.0);
    c.undo().unwrap();
    assert!(c.document().entities.is_empty());
    c.redo().unwrap();
    assert!(volume(&c) > 0.0);
}

#[test]
fn boolean_two_objects_menu_has_operation_prompt() {
    for (op, expected) in [("U", 1500.0), ("D", 500.0), ("I", 500.0)] {
        let mut c = controller();
        c.submit("Box 10 10 10").unwrap();
        c.submit("Box 10 10 10").unwrap();
        let ids = c.document().entities.keys().copied().collect::<Vec<_>>();
        c.select_objects(&[ids[1]]).unwrap();
        c.submit("Move").unwrap();
        c.accept_world_point([0.0; 3]).unwrap();
        c.accept_world_point([5.0, 0.0, 0.0]).unwrap();
        c.select_objects(&ids).unwrap();
        let before = c.coordinator().undo_len();
        review_fixture(&c, &format!("Boolean2-{op}-before"));
        c.dispatch("OM9-SOLID-004", InvocationOrigin::Menu).unwrap();
        assert!(c.prompt().contains("Union"));
        assert!(c.submit("invalid").is_err());
        assert_eq!(c.document().entities.len(), 2);
        c.submit("DeleteInput=Yes").unwrap();
        c.submit(op).unwrap();
        assert_eq!(c.document().entities.len(), 2);
        c.submit("").unwrap();
        review_fixture(&c, &format!("Boolean2-{op}-after"));
        assert_eq!(c.document().entities.len(), 1);
        assert_eq!(c.coordinator().undo_len(), before + 1);
        assert!((volume(&c) - expected).abs() < 1e-7);
        c.undo().unwrap();
        assert_eq!(c.document().entities.len(), 2);
    }
}

#[test]
fn create_solid_rejects_missing_selection_without_mutation() {
    let mut c = controller();
    c.submit("Box 10 10 10").unwrap();
    let before = c.document().clone();
    let undo = c.coordinator().undo_len();
    c.clear_selection();
    c.submit("CreateSolid").unwrap();
    assert!(c.submit("").is_err());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), undo);
    c.cancel();
}

#[test]
fn immediate_boolean_menu_actions_have_measured_results() {
    for (feature, expected) in [
        ("OM9-SOLID-001", 500.0),
        ("OM9-SOLID-002", 500.0),
        ("OM9-SOLID-003", 1500.0),
    ] {
        let mut c = controller();
        c.submit("Box 10 10 10").unwrap();
        c.submit("Box 10 10 10").unwrap();
        let ids = c.document().entities.keys().copied().collect::<Vec<_>>();
        c.select_objects(&[ids[1]]).unwrap();
        c.submit("Move").unwrap();
        c.accept_world_point([0.0; 3]).unwrap();
        c.accept_world_point([5.0, 0.0, 0.0]).unwrap();
        c.select_objects(&ids).unwrap();
        if matches!(feature, "OM9-SOLID-001" | "OM9-SOLID-002") {
            c.select_objects(&[ids[0]]).unwrap();
        }
        let undo = c.coordinator().undo_len();
        review_fixture(&c, &format!("{feature}-before"));
        c.dispatch(feature, InvocationOrigin::Menu).unwrap();
        if feature == "OM9-SOLID-003" {
            c.submit("DeleteInput=Yes").unwrap();
            c.submit("").unwrap();
        }
        if matches!(feature, "OM9-SOLID-001" | "OM9-SOLID-002") {
            c.submit("DeleteInput=Yes").unwrap();
            c.submit("").unwrap();
            c.solid_boolean_pick(ids[1], false).unwrap();
            c.submit("").unwrap();
        }
        review_fixture(&c, &format!("{feature}-after"));
        assert!((volume(&c) - expected).abs() < 1e-7);
        assert_eq!(c.document().entities.len(), 1);
        assert_eq!(c.coordinator().undo_len(), undo + 1);
        c.undo().unwrap();
        assert_eq!(c.document().entities.len(), 2);
        println!("menu_measured {feature} volume={expected}");
    }
}

#[test]
fn circle_extrusion_preserves_exact_geometry() {
    let mut c = circles(1);
    review_fixture(&c, "ExtrudeCrv-before");
    c.dispatch("OM9-SOLID-009", InvocationOrigin::Menu).unwrap();
    c.submit("5").unwrap();
    review_fixture(&c, "ExtrudeCrv-after");
    let expected = std::f64::consts::PI * 9.0 * 5.0;
    assert!(
        (volume(&c) / expected - 1.0).abs() < 1e-10,
        "{} vs {expected}",
        volume(&c)
    );
}

#[test]
fn extrude_all_is_one_atomic_transaction() {
    let mut c = circles(2);
    review_fixture(&c, "ExtrudeAll-before");
    c.dispatch("OM9-SOLID-010", InvocationOrigin::Menu).unwrap();
    c.submit("5").unwrap();
    review_fixture(&c, "ExtrudeAll-after");
    assert_eq!(c.coordinator().undo_len(), 1);
    assert_eq!(c.document().entities.len(), 4);
    c.undo().unwrap();
    assert_eq!(c.document().entities.len(), 2);
    c.redo().unwrap();
    assert_eq!(c.document().entities.len(), 4);
    // Sorting IDs makes a valid first profile and invalid second profile deterministic.
    let mut doc = Document::new("No partial extrusion");
    let valid =
        om9_core::curve_build::rectangle([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 2.0, 3.0)
            .unwrap();
    for (index, curve) in [
        valid,
        Curve::Line {
            start: [0.0; 3],
            end: [0.0, 0.0, 2.0],
            domain: [0.0, 1.0],
        },
    ]
    .into_iter()
    .enumerate()
    {
        let mut entity = Entity::new("profile", Geometry::Curve(curve), None);
        entity.id = om9_core::identity::ObjectId(uuid::Uuid::from_u128(index as u128 + 1));
        doc.entities.insert(entity.id, entity);
    }
    let ids = doc.entities.keys().copied().collect::<Vec<_>>();
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.submit("Box 1 1 1").unwrap();
    c.undo().unwrap();
    assert_eq!(c.coordinator().redo_len(), 1);
    c.select_objects(&ids).unwrap();
    let before = c.document().clone();
    assert!(c.submit("ExtrudeAll 5").is_err());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), 0);
    assert_eq!(c.coordinator().redo_len(), 1);
}

#[test]
fn invalid_topology_indices_do_not_edit_edge_or_face_zero() {
    let mut c = controller();
    c.submit("Box 10 10 10").unwrap();
    let id = *c.document().entities.keys().next().unwrap();
    c.select_objects(&[id]).unwrap();
    let before = c.document().clone();
    for command in [
        "ExtractSrf -1",
        "ExtractSrf 0.5",
        "FilletEdge -1 1",
        "FilletEdge 0.5 1",
        "Shell -1 1",
        "Shell 0.5 1",
    ] {
        assert!(c.submit(command).is_err(), "{command}");
        assert_eq!(c.document(), &before);
    }
    assert_eq!(c.coordinator().undo_len(), 1);
}

#[test]
fn truncated_cone_staged_coordinates_height_and_top_radius_follow_documented_order() {
    for height in [5.,-5.] {
        let mut c=controller(); let before=c.document().clone();
        c.submit("_TCone").unwrap();
        c.submit("3,7,2").unwrap();
        c.submit("3").unwrap();
        c.submit(&height.to_string()).unwrap();
        assert_eq!(c.coordinator().undo_len(),0);
        c.submit("1").unwrap();
        let entity=c.document().entities.values().next().unwrap();
        let Geometry::Brep(body)=&entity.geometry else {panic!()};
        let m=creation::measure(body,c.coordinator().limits()).unwrap();
        assert!((m.volume_mm3-65.*std::f64::consts::PI/3.).abs()<1e-6);
        let p=entity.placement.point([0.,0.,0.]).unwrap();
        let q=entity.placement.point([0.,0.,5.]).unwrap();
        assert!((p[0]-3.).abs()<1e-8 && (p[1]-7.).abs()<1e-8 && (p[2]-2.).abs()<1e-8);
        assert!((q[2]-(2.+height)).abs()<1e-8);
        c.undo().unwrap(); assert_eq!(c.document(),&before);
    }
}

#[test]
fn truncated_cone_curve_source_deletion_invalidates_preview_and_commit() {
    use om9_core::geometry::Curve;
    for mode in ["AroundCurve", "Tangent"] {
        let rail=Entity::new("rail",Geometry::Curve(Curve::Line {start:[0.,0.,0.],end:[10.,0.,0.],domain:[0.,1.]}),None);
        let id=rail.id; let mut doc=Document::new("stale cone source"); doc.entities.insert(id,rail);
        let mut c=Controller::new(doc,Limits::default()).unwrap(); c.select_objects(&[id]).unwrap();
        c.submit("TCone").unwrap(); c.submit(mode).unwrap(); c.submit("OnCurve=0.5").unwrap();
        if mode=="Tangent" {c.submit("Point").unwrap(); c.accept_world_point([8.,2.,0.]).unwrap();}
        else {c.submit("3").unwrap();}
        assert!(!c.solid_preview([5.,0.,5.]).is_empty());
        assert_eq!(c.delete_objects(&[id]).unwrap(),1);
        let after_delete=c.document().clone(); let undo_count=c.coordinator().undo_len();
        assert!(c.solid_preview([5.,0.,5.]).is_empty());
        c.submit("5").unwrap(); assert!(c.submit("1").is_err());
        assert_eq!(c.document(),&after_delete); assert_eq!(c.coordinator().undo_len(),undo_count);
    }
}

#[test]
fn truncated_cone_direct_dimensions_do_not_bypass_selected_base_mode() {
    for mode in ["2Point", "3Point", "FitPoints", "AroundCurve", "Tangent"] {
        let mut c=controller(); let before=c.document().clone();
        c.submit("TCone").unwrap(); c.submit(mode).unwrap();
        assert!(c.submit("2 5 1").is_err(), "{mode}");
        assert_eq!(c.document(),&before, "{mode}");
        assert_eq!(c.coordinator().undo_len(),0, "{mode}");
    }
    let mut c=controller(); c.submit("TCone").unwrap(); c.submit("2 5 1").unwrap();
    assert_eq!(c.document().entities.len(),1);
}

#[test]
fn truncated_cone_tangent_point_circle_and_out_of_plane_retry() {
    use om9_core::geometry::Curve;
    let circle=om9_core::curve_build::circle_on_frame([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],5.).unwrap();
    let mut rail=Entity::new("circle",Geometry::Curve(circle),None); rail.placement=Transform::translation([10.,20.,4.]); let id=rail.id;
    let mut doc=Document::new("tangent circle"); doc.entities.insert(id,rail); let mut c=Controller::new(doc,Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap(); c.submit("TCone").unwrap(); c.submit("Tangent").unwrap(); c.submit("OnCurve=0").unwrap(); c.submit("Point").unwrap();
    c.accept_world_point([18.,23.,4.]).unwrap(); c.submit("5").unwrap(); c.submit("1").unwrap();
    let entity=c.document().entities.values().find(|e|e.id!=id).unwrap(); let center=entity.placement.point([0.,0.,0.]).unwrap();
    assert!((center[0]-18.).abs()<1e-8 && (center[1]-20.).abs()<1e-8 && (center[2]-4.).abs()<1e-8);
    let line=Curve::Line {start:[0.,0.,0.],end:[0.,0.,10.],domain:[0.,1.]}; let rail=Entity::new("vertical",Geometry::Curve(line),None); let id=rail.id;
    let mut doc=Document::new("out of plane tangent"); doc.entities.insert(id,rail); let mut c=Controller::new(doc,Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap(); let before=c.document().clone(); c.submit("TCone").unwrap(); c.submit("Tangent").unwrap();
    assert!(c.submit("OnCurve=0.5").is_err()); assert_eq!(c.document(),&before); assert_eq!(c.coordinator().undo_len(),0);
}

#[test]
fn truncated_cone_tangent_point_constructs_both_sides_and_preserves_curve() {
    use om9_core::geometry::Curve;
    for side in [1.,-1.] {
        let rail=Entity::new("rail",Geometry::Curve(Curve::Line {start:[0.,0.,0.],end:[10.,0.,0.],domain:[0.,1.]}),None); let id=rail.id;
        let mut doc=Document::new("tangent point cone"); doc.entities.insert(id,rail); let mut c=Controller::new(doc,Limits::default()).unwrap();
        c.select_objects(&[id]).unwrap(); let before=c.document().clone(); c.submit("TCone").unwrap(); c.submit("Tangent").unwrap();
        assert!(c.submit("2 5 1").is_err()); assert_eq!(c.document(),&before);
        c.submit("OnCurve=0.5").unwrap();
        assert!(c.submit("2").is_err()); assert_eq!(c.document(),&before);
        c.submit("Point").unwrap(); assert!(c.submit("2").is_err()); assert_eq!(c.document(),&before);
        assert!(c.accept_world_point([8.,0.,0.]).is_err()); let point=[8.,2.*side,0.];
        let wire=c.solid_preview(point); assert_eq!(wire.len(),64);
        c.accept_world_point(point).unwrap(); c.submit("Undo").unwrap(); c.accept_world_point(point).unwrap();
        c.submit("5").unwrap(); c.submit("1").unwrap();
        let entity=c.document().entities.values().find(|e|e.id!=id).unwrap(); let Geometry::Brep(body)=&entity.geometry else {panic!()};
        let m=creation::measure(body,c.coordinator().limits()).unwrap(); let radius=3.25_f64;
        let expected=std::f64::consts::PI*5.*(radius*radius+radius+1.)/3.; assert!((m.volume_mm3-expected).abs()<1e-6);
        let center=entity.placement.point([0.,0.,0.]).unwrap(); assert!((center[0]-5.).abs()<1e-9 && (center[1]-side*radius).abs()<1e-9);
        c.undo().unwrap(); assert_eq!(c.document(),&before);
    }
}

#[test]
fn truncated_cone_around_curve_picks_center_and_retries_unsupported_placement() {
    use om9_core::geometry::Curve;
    for scaled in [false,true] {
        let curve=Curve::Line {start:[0.,0.,0.],end:[10.,0.,0.],domain:[0.,1.]};
        let mut rail=Entity::new("rail",Geometry::Curve(curve),None);
        rail.placement=if scaled {Transform::translation([10.,20.,4.]).compose(Transform::scale([2.,1.,1.]).unwrap()).unwrap()} else {Transform::translation([10.,20.,4.])};
        let id=rail.id; let mut doc=Document::new("around curve pick"); doc.entities.insert(id,rail); let mut c=Controller::new(doc,Limits::default()).unwrap();
        let before=c.document().clone(); c.dispatch("OM9-SOLID-020",InvocationOrigin::Menu).unwrap(); c.submit("AroundCurve").unwrap();
        assert!(c.solid_pipe_needs_curve()); c.solid_pipe_pick_curve(id).unwrap();
        assert!(c.submit("OnCurve=2").is_err());
        if scaled {assert!(c.accept_world_point([20.,21.,4.]).is_err()); c.submit("OnCurve=0.5").unwrap();}
        else {c.accept_world_point([15.,21.,4.]).unwrap();}
        let preview=c.solid_preview([if scaled {20.} else {15.},23.,4.]); assert_eq!(preview.len(),64);
        c.submit("3").unwrap(); c.submit("Undo").unwrap(); assert_eq!(c.prompt(),"Base radius or point"); c.submit("3").unwrap(); c.submit("5").unwrap(); c.submit("1").unwrap();
        let entity=c.document().entities.values().find(|e|e.id!=id).unwrap(); let center=entity.placement.point([0.,0.,0.]).unwrap();
        assert!((center[0]-if scaled {20.} else {15.}).abs()<1e-8 && (center[1]-20.).abs()<1e-8 && (center[2]-4.).abs()<1e-8);
        c.undo().unwrap(); assert_eq!(c.document(),&before);
    }
}

#[test]
fn truncated_cone_around_curve_uses_placed_tangent_and_preserves_source() {
    use om9_core::geometry::Curve;
    for circle in [false,true] {
        let curve=if circle {om9_core::curve_build::circle_on_frame([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],5.).unwrap()}
            else {Curve::Line {start:[0.,0.,0.],end:[10.,0.,0.],domain:[0.,1.]}};
        let mut rail=Entity::new("rail",Geometry::Curve(curve),None); rail.placement=Transform::translation([10.,20.,4.]); let id=rail.id;
        let mut doc=Document::new("around curve cone"); doc.entities.insert(id,rail); let mut c=Controller::new(doc,Limits::default()).unwrap();
        c.select_objects(&[id]).unwrap(); let before=c.document().clone(); c.submit("TCone").unwrap(); c.submit("AroundCurve").unwrap();
        c.submit(if circle {"OnCurve=0.25"} else {"OnCurve=0.5"}).unwrap(); c.submit("3").unwrap(); c.submit(if circle {"-5"} else {"5"}).unwrap(); c.submit("1").unwrap();
        let entity=c.document().entities.values().find(|e|e.id!=id).unwrap(); let Geometry::Brep(body)=&entity.geometry else {panic!()};
        let m=creation::measure(body,c.coordinator().limits()).unwrap(); assert!((m.volume_mm3-65.*std::f64::consts::PI/3.).abs()<1e-6);
        let a=entity.placement.point([0.,0.,0.]).unwrap(); let b=entity.placement.point([0.,0.,5.]).unwrap();
        let expected=if circle {[10.,25.,4.]} else {[15.,20.,4.]};
        for i in 0..3 {assert!((a[i]-expected[i]).abs()<1e-7);}
        assert!((b[0]-(expected[0]+5.)).abs()<1e-6 && (b[1]-expected[1]).abs()<1e-6 && (b[2]-expected[2]).abs()<1e-6);
        c.undo().unwrap(); assert_eq!(c.document(),&before);
    }
}

#[test]
fn truncated_cone_fit_points_uses_all_samples_in_algebraic_fit() {
    let mut c=controller(); c.submit("TCone").unwrap(); c.submit("FitPoints").unwrap();
    for p in [[2.,0.,0.],[-2.,0.,0.],[0.,4.,0.],[0.,-4.,0.]] {c.accept_world_point(p).unwrap();}
    c.submit("").unwrap(); c.submit("5").unwrap(); c.submit("1").unwrap();
    let Geometry::Brep(body)=&c.document().entities.values().next().unwrap().geometry else {panic!()};
    let m=creation::measure(body,c.coordinator().limits()).unwrap(); let r=10.0_f64.sqrt();
    let expected=std::f64::consts::PI*5.*(r*r+r+1.)/3.; assert!((m.volume_mm3-expected).abs()<1e-6,"{m:?}");
}

#[test]
fn truncated_cone_fit_points_uses_placed_curve_and_surface_control_points() {
    use om9_core::geometry::{Curve,NurbsCurve,NurbsSurface};
    let a=3.0_f64/std::f64::consts::SQRT_2;
    let curve=Curve::Nurbs(NurbsCurve {degree:1,poles:vec![[3.,0.,0.],[0.,3.,0.],[-3.,0.,0.],[0.,-3.,0.]],weights:vec![1.;4],knots:vec![0.,0.,1.,2.,3.,3.],domain:[0.,3.],periodic:false,reversed:false,seam:None});
    let surface=NurbsSurface {degree:[1,1],size:[2,2],poles:vec![[-a,-a,0.],[-a,a,0.],[a,-a,0.],[a,a,0.]],weights:vec![1.;4],knots:[vec![0.,0.,1.,1.],vec![0.,0.,1.,1.]],domain:[[0.,1.],[0.,1.]],periodic:[false,false],reversed:false};
    for geometry in [Geometry::Curve(curve),Geometry::Surface(surface)] {
        let mut entity=Entity::new("controls",geometry,None); entity.placement=Transform::translation([10.,20.,4.]); let id=entity.id;
        let mut doc=Document::new("fit controls"); doc.entities.insert(id,entity); let mut c=Controller::new(doc,Limits::default()).unwrap();
        c.select_objects(&[id]).unwrap(); c.submit("TCone").unwrap(); c.submit("FitPoints").unwrap(); c.submit("").unwrap(); c.submit("5").unwrap(); c.submit("1").unwrap();
        let entity=c.document().entities.values().find(|e|e.id!=id).unwrap(); let Geometry::Brep(body)=&entity.geometry else {panic!()};
        let m=creation::measure(body,c.coordinator().limits()).unwrap(); assert!((m.volume_mm3-65.*std::f64::consts::PI/3.).abs()<1e-6,"{m:?}");
        let center=entity.placement.point([0.,0.,0.]).unwrap(); assert!((center[0]-10.).abs()<1e-9 && (center[1]-20.).abs()<1e-9 && (center[2]-4.).abs()<1e-9);
    }
}

#[test]
fn truncated_cone_fit_points_limit_refuses_without_commit() {
    let limits=Limits {max_points:3,..Limits::default()}; let mut c=Controller::new(Document::new("bounded fit"),limits).unwrap(); let before=c.document().clone();
    c.submit("TCone").unwrap(); c.submit("FitPoints").unwrap();
    for p in [[3.,0.,0.],[0.,3.,0.],[-3.,0.,0.]] {c.accept_world_point(p).unwrap();}
    assert!(c.accept_world_point([0.,-3.,0.]).is_err()); assert!(c.submit("").is_err());
    assert_eq!(c.document(),&before); assert_eq!(c.coordinator().undo_len(),0);
}

#[test]
fn truncated_cone_fit_points_direct_preview_and_collinear_retry() {
    let mut c=controller(); let before=c.document().clone(); c.submit("TCone").unwrap(); c.submit("FitPoints").unwrap();
    c.submit("6,7,2").unwrap(); c.submit("3,7,2").unwrap(); c.submit("0,7,2").unwrap();
    assert!(c.submit("").is_err()); assert_eq!(c.prompt(),"Add fit points or Enter to fit");
    c.submit("Undo").unwrap(); c.submit("3,10,2").unwrap();
    let wire=c.solid_preview([0.,7.,2.]); assert_eq!(wire.len(),64);
    c.accept_world_point([0.,7.,2.]).unwrap(); c.submit("").unwrap(); c.submit("Undo").unwrap();
    assert_eq!(c.prompt(),"Add fit points or Enter to fit"); c.submit("").unwrap(); c.submit("5").unwrap(); c.submit("1").unwrap();
    let entity=c.document().entities.values().next().unwrap(); let Geometry::Brep(body)=&entity.geometry else {panic!()};
    let m=creation::measure(body,c.coordinator().limits()).unwrap();
    assert!(m.closed && m.volume_mm3>0.); c.undo().unwrap(); assert_eq!(c.document(),&before);
}

#[test]
fn truncated_cone_fit_points_uses_placed_mesh_vertices() {
    let mesh=om9_core::geometry::Mesh {vertices:vec![[3.,0.,0.],[0.,3.,0.],[-3.,0.,0.],[0.,-3.,0.]],faces:vec![vec![0,1,2],vec![0,2,3]],normals:vec![],colors:vec![],corner_uvs:vec![]};
    let mut entity=Entity::new("mesh",Geometry::Mesh(mesh),None); entity.placement=Transform::translation([10.,20.,4.]); let id=entity.id;
    let mut doc=Document::new("fit mesh"); doc.entities.insert(id,entity); let mut c=Controller::new(doc,Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap(); c.submit("TCone").unwrap(); c.submit("FitPoints").unwrap(); c.submit("").unwrap(); c.submit("5").unwrap(); c.submit("1").unwrap();
    let entity=c.document().entities.values().find(|e|e.id!=id).unwrap(); let Geometry::Brep(body)=&entity.geometry else {panic!()};
    let m=creation::measure(body,c.coordinator().limits()).unwrap(); assert!((m.volume_mm3-65.*std::f64::consts::PI/3.).abs()<1e-6);
    let center=entity.placement.point([0.,0.,0.]).unwrap(); assert!((center[0]-10.).abs()<1e-9 && (center[1]-20.).abs()<1e-9 && (center[2]-4.).abs()<1e-9);
}

#[test]
fn truncated_cone_fit_points_handles_placed_point_selection_and_undo() {
    let mut doc=Document::new("fit cone"); let mut ids=Vec::new();
    for p in [[3.,0.,0.],[0.,3.,0.],[-3.,0.,0.],[0.,-3.,0.]] {
        let mut e=Entity::new("fit point",Geometry::Point(p),None); e.placement=Transform::translation([10.,20.,4.]); ids.push(e.id); doc.entities.insert(e.id,e);
    }
    let mut c=Controller::new(doc,Limits::default()).unwrap(); c.select_objects(&ids).unwrap(); let before=c.document().clone();
    c.submit("TCone").unwrap(); c.submit("FitPoints").unwrap(); c.submit("").unwrap();
    assert_eq!(c.prompt(),"Height or point"); c.submit("5").unwrap(); c.submit("1").unwrap();
    assert_eq!(c.document().entities.len(),5);
    let entity=c.document().entities.values().find(|e|!ids.contains(&e.id)).unwrap(); let Geometry::Brep(body)=&entity.geometry else {panic!()};
    let m=creation::measure(body,c.coordinator().limits()).unwrap(); assert!((m.volume_mm3-65.*std::f64::consts::PI/3.).abs()<1e-6);
    let center=entity.placement.point([0.,0.,0.]).unwrap(); assert!((center[0]-10.).abs()<1e-9 && (center[1]-20.).abs()<1e-9 && (center[2]-4.).abs()<1e-9);
    c.undo().unwrap(); assert_eq!(c.document(),&before);
}

#[test]
fn truncated_cone_open_solid_option_survives_radius_selection_and_undo() {
    let mut c=controller(); c.submit("TCone").unwrap(); c.submit("Solid=No").unwrap(); c.submit("3Point").unwrap();
    c.submit("0,0,0").unwrap(); c.submit("4,0,0").unwrap(); c.submit("Radius=3").unwrap();
    c.submit("Undo").unwrap(); c.submit("3").unwrap(); c.accept_world_point([2.,1.,0.]).unwrap();
    c.submit("5").unwrap(); c.submit("1").unwrap();
    let Geometry::Brep(body)=&c.document().entities.values().next().unwrap().geometry else {panic!()};
    let m=creation::measure(body,c.coordinator().limits()).unwrap(); assert!(!m.closed && m.naked_edges==2);
}

#[test]
fn truncated_cone_solid_option_controls_open_or_closed_brep() {
    for (closed,bottom,top,height) in [(false,3.0_f64,1.,5.0_f64),(false,1.,3.,-5.),(false,2.,2.,5.),(true,3.,1.,5.),(true,2.,2.,5.)] {
        let mut c=controller(); let before=c.document().clone(); c.submit("TCone").unwrap();
        c.submit(if closed {"Solid=Yes"} else {"Solid=No"}).unwrap(); c.submit("3,7,2").unwrap();
        c.submit(&bottom.to_string()).unwrap(); c.submit(&height.to_string()).unwrap(); c.submit(&top.to_string()).unwrap();
        let entity=c.document().entities.values().next().unwrap(); let Geometry::Brep(body)=&entity.geometry else {panic!()};
        let m=creation::measure(body,c.coordinator().limits()).unwrap();
        if closed {let volume=std::f64::consts::PI*height.abs()*(bottom*bottom+bottom*top+top*top)/3.;
            assert!(m.closed); assert_eq!(m.solids,1); assert!((m.volume_mm3-volume).abs()<1e-6);}
        else {let area=std::f64::consts::PI*(bottom+top)*((bottom-top).powi(2)+height.powi(2)).sqrt();
            assert!(!m.closed); assert_eq!((m.solids,m.faces,m.naked_edges),(0,1,2)); assert!((m.area_mm2-area).abs()<1e-6,"{m:?} expected {area}");}
        assert_eq!(c.coordinator().undo_len(),1); c.undo().unwrap(); assert_eq!(c.document(),&before);
    }
    let mut direct=controller(); direct.submit("TCone 2 2 5").unwrap();
    let Geometry::Brep(body)=&direct.document().entities.values().next().unwrap().geometry else {panic!()};
    let m=creation::measure(body,direct.coordinator().limits()).unwrap(); assert!(m.closed && (m.volume_mm3-20.*std::f64::consts::PI).abs()<1e-6);
    let mut retry=controller(); let before=retry.document().clone(); retry.submit("TCone").unwrap();
    assert!(retry.submit("Solid=maybe").is_err()); retry.submit("Solid=No").unwrap(); retry.cancel();
    assert_eq!(retry.document(),&before); assert_eq!(retry.coordinator().undo_len(),0);
}

#[test]
fn truncated_cone_base_area_assignment_and_size_switching() {
    let mut c=controller(); c.submit("TCone").unwrap(); c.submit("3,7,0").unwrap();
    assert!(c.submit("Area=0").is_err()); assert_eq!(c.prompt(),"Base area");
    c.submit("Diameter").unwrap(); assert_eq!(c.prompt(),"Base radius or point");
    c.submit("Area").unwrap(); c.submit("Radius").unwrap(); c.submit("3").unwrap();
    c.submit("5").unwrap(); c.submit("1").unwrap();
    let entity=c.document().entities.values().next().unwrap(); let Geometry::Brep(body)=&entity.geometry else {panic!()};
    let m=creation::measure(body,c.coordinator().limits()).unwrap(); assert!((m.volume_mm3-65.*std::f64::consts::PI/3.).abs()<1e-6);
    let mut direct=controller(); direct.submit("TCone").unwrap(); direct.submit("3,7,0").unwrap();
    direct.submit(&format!("Area={}",9.*std::f64::consts::PI)).unwrap(); direct.submit("5").unwrap(); direct.submit("1").unwrap();
    let Geometry::Brep(body)=&direct.document().entities.values().next().unwrap().geometry else {panic!()};
    let m=creation::measure(body,direct.coordinator().limits()).unwrap(); assert!((m.volume_mm3-65.*std::f64::consts::PI/3.).abs()<1e-6);
}

#[test]
fn truncated_cone_base_area_uses_square_document_units_and_top_radius() {
    let mut doc=Document::new("cone area units"); doc.page_unit=om9_core::geometry::Unit::Centimeter;
    let mut c=Controller::new(doc,Limits::default()).unwrap(); let before=c.document().clone();
    c.submit("TCone").unwrap(); c.accept_world_point([3.,7.,2.]).unwrap();
    c.submit("Diameter").unwrap(); c.submit("Area").unwrap(); assert_eq!(c.prompt(),"Base area");
    assert!(c.submit("-1").is_err()); assert_eq!(c.prompt(),"Base area");
    c.submit(&(9.*std::f64::consts::PI).to_string()).unwrap();
    c.submit("0.5").unwrap(); c.submit("0.1").unwrap();
    let entity=c.document().entities.values().next().unwrap(); let Geometry::Brep(body)=&entity.geometry else {panic!()};
    let m=creation::measure(body,c.coordinator().limits()).unwrap(); let expected=5.*std::f64::consts::PI*(900.+30.+1.)/3.;
    assert!((m.volume_mm3-expected).abs()<1e-5,"{m:?}"); c.undo().unwrap(); assert_eq!(c.document(),&before);
}

#[test]
fn truncated_cone_three_point_radius_pick_uses_captured_front_plane() {
    let mut c=controller(); c.set_cplane_preset(om9_scene::camera::CPlanePreset::Front).unwrap();
    let plane=c.input_plane(); c.submit("TCone").unwrap(); c.submit("3Point").unwrap();
    c.submit("0,0,2").unwrap(); c.submit("4,0,2").unwrap(); c.submit("Radius").unwrap();
    c.accept_world_point(plane.point([0.,3.,8.]).unwrap()).unwrap();
    assert!(c.accept_world_point(plane.point([2.,0.,2.]).unwrap()).is_err());
    c.accept_world_point(plane.point([2.,1.,2.]).unwrap()).unwrap();
    c.submit("Undo").unwrap(); c.submit("Undo").unwrap(); assert_eq!(c.prompt(),"Base circle radius");
    c.accept_world_point(plane.point([0.,3.,2.]).unwrap()).unwrap();
    c.accept_world_point(plane.point([2.,1.,2.]).unwrap()).unwrap(); c.submit("5").unwrap(); c.submit("1").unwrap();
    let entity=c.document().entities.values().next().unwrap(); let actual=entity.placement.point([0.,0.,0.]).unwrap();
    let expected=plane.point([2.,5.0_f64.sqrt(),2.]).unwrap();
    for i in 0..3 {assert!((actual[i]-expected[i]).abs()<1e-9);}
}

#[test]
fn truncated_cone_three_point_radius_selects_both_centers_and_retries() {
    for (side,expected_y) in [(1.,5.0_f64.sqrt()),(-1.,-5.0_f64.sqrt())] {
        let mut c=controller(); let before=c.document().clone(); c.submit("TCone").unwrap(); c.submit("3Point").unwrap();
        c.submit("0,0,0").unwrap(); c.submit("4,0,0").unwrap(); c.submit("Radius").unwrap();
        assert!(c.submit("1").is_err()); assert_eq!(c.prompt(),"Base circle radius");
        c.submit("Radius=3").unwrap(); assert_eq!(c.prompt(),"Choose base circle side");
        let preview=c.solid_preview([2.,side,0.]); assert_eq!(preview.len(),64);
        assert!((preview[0][0][0]-5.).abs()<1e-9 && (preview[0][0][1]-expected_y).abs()<1e-9);
        c.accept_world_point([2.,side,0.]).unwrap(); c.submit("Undo").unwrap(); assert_eq!(c.prompt(),"Choose base circle side");
        c.accept_world_point([2.,side,0.]).unwrap(); c.submit("5").unwrap(); c.submit("1").unwrap();
        let entity=c.document().entities.values().next().unwrap(); let Geometry::Brep(body)=&entity.geometry else {panic!()};
        let m=creation::measure(body,c.coordinator().limits()).unwrap(); assert!((m.volume_mm3-65.*std::f64::consts::PI/3.).abs()<1e-6);
        let center=entity.placement.point([0.,0.,0.]).unwrap(); assert!((center[0]-2.).abs()<1e-9 && (center[1]-expected_y).abs()<1e-9);
        c.undo().unwrap(); assert_eq!(c.document(),&before);
    }
}

#[test]
fn truncated_cone_three_point_front_frame_projects_preview_and_undoes_second_point() {
    let mut c=controller(); c.set_cplane_preset(om9_scene::camera::CPlanePreset::Front).unwrap();
    let plane=c.input_plane(); let a=plane.point([6.,7.,2.]).unwrap(); let b=plane.point([3.,10.,2.]).unwrap();
    let off=plane.point([3.,10.,9.]).unwrap(); let third=plane.point([0.,7.,2.]).unwrap();
    c.submit("TCone").unwrap(); c.submit("3Point").unwrap(); c.accept_world_point(a).unwrap();
    let wire=c.solid_preview(off); assert_eq!(wire.len(),1);
    for i in 0..3 {assert!((wire[0][1][i]-b[i]).abs()<1e-9);}
    c.accept_world_point(off).unwrap(); c.submit("Undo").unwrap(); assert_eq!(c.prompt(),"Second point on base circle");
    c.accept_world_point(off).unwrap(); c.accept_world_point(third).unwrap(); c.submit("5").unwrap(); c.submit("1").unwrap();
    let entity=c.document().entities.values().next().unwrap(); let center=entity.placement.point([0.,0.,0.]).unwrap();
    let expected=plane.point([3.,7.,2.]).unwrap(); for i in 0..3 {assert!((center[i]-expected[i]).abs()<1e-9);}
}

#[test]
fn truncated_cone_three_point_constructs_circumcircle_and_retries_collinear() {
    for swapped in [false,true] {
        let mut c=controller(); let before=c.document().clone(); c.submit("_TCone").unwrap(); c.submit("3Point").unwrap();
        c.submit("6,7,2").unwrap();
        if swapped {c.submit("3,4,2").unwrap();} else {c.submit("3,10,2").unwrap();}
        assert!(c.submit(if swapped {"0,1,2"} else {"0,13,2"}).is_err());
        assert_eq!(c.prompt(),"Third point on base circle");
        let third=[0.,7.,2.];
        let preview=c.solid_preview(third); assert_eq!(preview.len(),64);
        assert!((preview[0][0][0]-6.).abs()<1e-9 && (preview[0][0][1]-7.).abs()<1e-9);
        c.accept_world_point(third).unwrap(); c.submit("Undo").unwrap();
        assert_eq!(c.prompt(),"Third point on base circle"); c.accept_world_point(third).unwrap();
        c.submit("-5").unwrap(); c.submit("1").unwrap();
        let entity=c.document().entities.values().next().unwrap(); let Geometry::Brep(body)=&entity.geometry else {panic!()};
        let m=creation::measure(body,c.coordinator().limits()).unwrap();
        assert!((m.volume_mm3-65.*std::f64::consts::PI/3.).abs()<1e-6);
        let a=entity.placement.point([0.,0.,0.]).unwrap(); let b=entity.placement.point([0.,0.,5.]).unwrap();
        assert!((a[0]-3.).abs()<1e-9 && (a[1]-7.).abs()<1e-9 && (a[2]-2.).abs()<1e-9);
        assert!((b[2]+3.).abs()<1e-9); c.undo().unwrap(); assert_eq!(c.document(),&before);
    }
}

#[test]
fn truncated_cone_preview_matches_signed_committed_frustum() {
    for height in [5.,-5.] {
        let mut c=controller(); c.submit("TCone").unwrap(); c.submit("3,7,2").unwrap();
        let base=c.solid_preview([6.,7.,2.]); assert_eq!(base.len(),64);
        c.accept_world_point([6.,7.,2.]).unwrap();
        let h=c.solid_preview([3.,7.,2.+height]); assert_eq!(h.len(),65);
        assert!((h[64][1][2]-(2.+height)).abs()<1e-9);
        c.accept_world_point([3.,7.,2.+height]).unwrap();
        let preview=c.solid_preview([4.,7.,2.+height]); assert_eq!(preview.len(),132);
        assert!((preview[0][0][0]-6.).abs()<1e-9 && (preview[0][0][2]-2.).abs()<1e-9);
        assert!((preview[64][0][0]-4.).abs()<1e-9 && (preview[64][0][2]-(2.+height)).abs()<1e-9);
        c.accept_world_point([4.,7.,2.+height]).unwrap();
        let entity=c.document().entities.values().next().unwrap();
        let bottom=entity.placement.point([3.,0.,0.]).unwrap();
        let top=entity.placement.point([1.,0.,5.]).unwrap();
        for i in 0..3 {assert!((bottom[i]-preview[0][0][i]).abs()<1e-9); assert!((top[i]-preview[64][0][i]).abs()<1e-9);}
    }
}

#[test]
fn truncated_cone_two_point_uses_diameter_endpoints_and_undoes_each_stage() {
    for mode in ["TruncatedCone2Point", "_TCone"] {
        let mut c=controller(); let before=c.document().clone();
        if mode=="_TCone" {c.submit(mode).unwrap(); c.submit("2Point").unwrap();}
        else {c.dispatch(mode,InvocationOrigin::Menu).unwrap();}
        assert_eq!(c.prompt(),"First diameter endpoint");
        c.submit("0,7,2").unwrap(); assert!(c.submit("0,7,2").is_err());
        c.submit("6,7,2").unwrap(); assert_eq!(c.prompt(),"Height or point");
        c.submit("Undo").unwrap(); assert_eq!(c.prompt(),"Other diameter endpoint");
        c.submit("6,7,2").unwrap(); c.submit("-5").unwrap(); c.submit("1").unwrap();
        let entity=c.document().entities.values().next().unwrap();
        let Geometry::Brep(body)=&entity.geometry else {panic!()};
        let m=creation::measure(body,c.coordinator().limits()).unwrap();
        assert!((m.volume_mm3-65.*std::f64::consts::PI/3.).abs()<1e-6);
        assert_eq!(entity.placement.point([0.,0.,0.]).unwrap(),[3.,7.,2.]);
        assert_eq!(entity.placement.point([0.,0.,5.]).unwrap(),[3.,7.,-3.]);
        c.undo().unwrap(); assert_eq!(c.document(),&before);
    }
}

#[test]
fn truncated_cone_captures_front_construction_plane_for_signed_height() {
    let mut c=controller(); c.set_cplane_preset(om9_scene::camera::CPlanePreset::Front).unwrap();
    let plane=c.input_plane(); let origin=plane.point([3.,7.,2.]).unwrap(); let normal=plane.normal();
    c.dispatch("OM9-SOLID-020",InvocationOrigin::Menu).unwrap(); c.submit("3,7,2").unwrap();
    c.submit("3").unwrap(); c.submit("-5").unwrap(); c.submit("1").unwrap();
    let entity=c.document().entities.values().next().unwrap();
    let a=entity.placement.point([0.,0.,0.]).unwrap(); let b=entity.placement.point([0.,0.,5.]).unwrap();
    for i in 0..3 { assert!((a[i]-origin[i]).abs()<1e-8); assert!((b[i]-(origin[i]-5.*normal[i])).abs()<1e-8); }
}

#[test]
fn truncated_cone_points_diameter_retry_cancel_and_undo_are_atomic() {
    let mut c=controller(); let before=c.document().clone();
    c.dispatch("OM9-SOLID-020",InvocationOrigin::Menu).unwrap();
    c.accept_world_point([3.,7.,2.]).unwrap();
    assert!(c.submit("0").is_err());
    c.submit("Diameter").unwrap(); c.submit("6").unwrap();
    assert!(c.submit("0").is_err()); c.accept_world_point([3.,7.,-3.]).unwrap();
    c.submit("Undo").unwrap(); c.submit("-5").unwrap();
    assert!(c.submit("-2").is_err()); c.submit("2").unwrap();
    let entity=c.document().entities.values().next().unwrap();
    let Geometry::Brep(body)=&entity.geometry else {panic!()};
    let m=creation::measure(body,c.coordinator().limits()).unwrap();
    assert!((m.volume_mm3-65.*std::f64::consts::PI/3.).abs()<1e-6);
    assert_eq!(c.coordinator().undo_len(),1); c.undo().unwrap(); assert_eq!(c.document(),&before);
    c.submit("TCone").unwrap(); c.accept_world_point([3.,7.,2.]).unwrap(); c.cancel();
    assert_eq!(c.document(),&before); assert_eq!(c.coordinator().undo_len(),0);
}

#[test]
fn remaining_primitive_menu_prompts_produce_analytic_volumes() {
    let pi = std::f64::consts::PI;
    for (feature, input, expected) in [
        ("OM9-SOLID-012", "2 3 4", 24.0),
        ("OM9-SOLID-013", "2 3 4", 32.0 * pi),
        ("OM9-SOLID-014", "3", 36.0 * pi),
        ("OM9-SOLID-015", "5 1", 10.0 * pi * pi),
        ("OM9-SOLID-016", "3 5", 45.0 * pi),
        ("OM9-SOLID-017", "3 2 5", 25.0 * pi),
        ("OM9-SOLID-018", "4 3 5", 30.0),
        ("OM9-SOLID-019", "3 5", 15.0 * pi),
        ("OM9-SOLID-020", "3 1 5", 65.0 * pi / 3.0),
    ] {
        let mut c = controller();
        c.dispatch(feature, InvocationOrigin::Menu).unwrap();
        c.cancel();
        assert_eq!(c.coordinator().undo_len(), 0);
        c.dispatch(feature, InvocationOrigin::Menu).unwrap();
        match feature {
            "OM9-SOLID-012" => {
                c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
                c.accept_world_point([2.0, 3.0, 0.0]).unwrap();
                c.submit("4").unwrap();
            }
            "OM9-SOLID-014" => {
                c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
                c.accept_world_point([3.0, 0.0, 0.0]).unwrap();
            }
            _ => c.submit(input).unwrap(),
        }
        review_fixture(&c, &format!("{feature}-after"));
        assert_eq!(c.coordinator().undo_len(), 1);
        assert!(
            (volume(&c) / expected - 1.0).abs() < 1e-9,
            "{feature}: {} expected {expected}",
            volume(&c)
        );
        c.undo().unwrap();
        assert!(c.document().entities.is_empty());
        println!("menu_measured {feature} volume={expected:.12}");
    }
}

#[test]
fn boss_smooth_nurbs_line_on_cylinder_commit_and_undo() {
    for (outer,inner) in [(false,false),(false,true),(true,false),(true,true)] {
        let cylinder=creation::primitive(creation::Primitive::Cylinder {radius_mm:5.,height_mm:10.},0.01,&Limits::default()).unwrap();
        let count=creation::measure(&cylinder,&Limits::default()).unwrap().faces;
        let face=(0..count).map(|i| om9_kernel::editing::extract_face(&cylinder,i,0.01,&Limits::default()).unwrap())
            .find(|b| (creation::measure(b,&Limits::default()).unwrap().area_mm2-100.*std::f64::consts::PI).abs()<1e-6).unwrap();
        let mut target=Entity::new("cylinder",Geometry::Brep(face),None); target.placement=Transform::translation([10.,0.,0.]); let id=target.id;
        let arch=Curve::Nurbs(om9_core::geometry::NurbsCurve {degree:2,poles:vec![[-1.,0.,3.],[-1.,0.,4.],[1.,0.,4.],[1.,0.,3.]],weights:vec![1.,0.7,1.2,1.],knots:vec![0.,0.,0.,0.5,1.,1.,1.],domain:[0.,1.],periodic:false,reversed:inner,seam:None});
        let line=Curve::Line {start:if inner {[-1.,0.,3.]} else {[1.,0.,3.]},end:if inner {[1.,0.,3.]} else {[-1.,0.,3.]},domain:[0.,1.]};
        let mut rail=Entity::new("profile",Geometry::Curve(Curve::Polycurve {segments:vec![arch,line],reversed:outer}),None); rail.placement=Transform::translation([10.,0.,0.]); let rail_id=rail.id;
        let mut doc=Document::new("polycurve cylinder Boss"); doc.entities.insert(id,target); doc.entities.insert(rail_id,rail);
        let mut c=Controller::new(doc,Limits::default()).unwrap(); let before=c.document().clone();
        if outer {c.dispatch("OM9-SOLID-021",InvocationOrigin::Menu).unwrap();} else {c.submit("Boss").unwrap();}
        c.solid_pipe_pick_curve(rail_id).unwrap(); c.submit("").unwrap(); c.solid_boolean_pick(id,false).unwrap();
        let Geometry::Brep(body)=&c.document().entities[&id].geometry else {panic!()};
        let m=creation::measure(body,c.coordinator().limits()).unwrap(); assert_eq!(m.faces,4); assert_eq!(m.shells,1);
        c.undo().unwrap(); assert_eq!(c.document(),&before);
    }
}

#[test]
fn boss_smooth_rational_nurbs_on_cylinder_commit_and_undo() {
    for reversed in [false,true] {
        let cylinder=creation::primitive(creation::Primitive::Cylinder {radius_mm:5.,height_mm:10.},0.01,&Limits::default()).unwrap();
        let count=creation::measure(&cylinder,&Limits::default()).unwrap().faces;
        let face=(0..count).map(|i| om9_kernel::editing::extract_face(&cylinder,i,0.01,&Limits::default()).unwrap())
            .find(|b| (creation::measure(b,&Limits::default()).unwrap().area_mm2-100.*std::f64::consts::PI).abs()<1e-6).unwrap();
        let mut target=Entity::new("cylinder",Geometry::Brep(face),None); target.placement=Transform::translation([10.,0.,0.]); let id=target.id;
        let n=om9_core::geometry::NurbsCurve {degree:2,poles:vec![[1.,0.,3.],[1.,0.,4.],[-1.,0.,4.],[-1.,0.,2.],[1.,0.,2.],[1.,0.,3.]],weights:vec![1.,0.7,1.2,0.8,1.3,1.],knots:vec![0.,0.,0.,0.25,0.5,0.75,1.,1.,1.],domain:[0.,1.],periodic:false,reversed,seam:None};
        let mut rail=Entity::new("profile",Geometry::Curve(Curve::Nurbs(n)),None); rail.placement=Transform::translation([10.,0.,0.]); let rail_id=rail.id;
        let mut doc=Document::new("smooth cylinder Boss"); doc.entities.insert(id,target); doc.entities.insert(rail_id,rail);
        let mut c=Controller::new(doc,Limits::default()).unwrap(); let before=c.document().clone();
        if reversed {c.dispatch("OM9-SOLID-021",InvocationOrigin::Menu).unwrap();} else {c.submit("Boss").unwrap();}
        c.solid_pipe_pick_curve(rail_id).unwrap(); c.submit("").unwrap(); c.solid_boolean_pick(id,false).unwrap();
        let Geometry::Brep(body)=&c.document().entities[&id].geometry else {panic!()};
        let m=creation::measure(body,c.coordinator().limits()).unwrap(); assert_eq!(m.faces,5); assert_eq!(m.shells,1);
        c.undo().unwrap(); assert_eq!(c.document(),&before);
    }
}

#[test]
fn boss_cylinder_circle_and_polygon_commit_together() {
    let cylinder=creation::primitive(creation::Primitive::Cylinder {radius_mm:5.,height_mm:10.},0.01,&Limits::default()).unwrap();
    let count=creation::measure(&cylinder,&Limits::default()).unwrap().faces;
    let face=(0..count).map(|i| om9_kernel::editing::extract_face(&cylinder,i,0.01,&Limits::default()).unwrap())
        .find(|b| (creation::measure(b,&Limits::default()).unwrap().area_mm2-100.*std::f64::consts::PI).abs()<1e-6).unwrap();
    let target=Entity::new("cylinder",Geometry::Brep(face),None); let id=target.id;
    let circle=Entity::new("circle",Geometry::Curve(om9_core::curve_build::circle_on_frame([0.,0.,2.],[1.,0.,0.],[0.,0.,1.],0.5).unwrap()),None); let cid=circle.id;
    let polygon=Entity::new("polygon",Geometry::Curve(om9_core::curve_build::polygon(&[[-1.,0.,5.],[1.,0.,5.],[1.,0.,6.],[-1.,0.,6.]]).unwrap()),None); let pid=polygon.id;
    let mut doc=Document::new("mixed cylinder Boss"); doc.entities.insert(id,target); doc.entities.insert(cid,circle); doc.entities.insert(pid,polygon);
    let mut c=Controller::new(doc,Limits::default()).unwrap(); let before=c.document().clone(); c.submit("Boss").unwrap();
    c.solid_pipe_pick_curve(cid).unwrap(); c.solid_pipe_pick_curve(pid).unwrap(); c.submit("").unwrap(); c.solid_boolean_pick(id,false).unwrap();
    let Geometry::Brep(body)=&c.document().entities[&id].geometry else {panic!()};
    let m=creation::measure(body,c.coordinator().limits()).unwrap(); assert_eq!(m.faces,6); assert_eq!(m.shells,1); assert_eq!(c.coordinator().undo_len(),1);
    c.undo().unwrap(); assert_eq!(c.document(),&before);
}

#[test]
fn boss_circle_and_ellipse_on_cylinder_commit_and_undo() {
    for circle in [true,false] {
        let cylinder=creation::primitive(creation::Primitive::Cylinder {radius_mm:5.,height_mm:10.},0.01,&Limits::default()).unwrap();
        let count=creation::measure(&cylinder,&Limits::default()).unwrap().faces;
        let face=(0..count).map(|i| om9_kernel::editing::extract_face(&cylinder,i,0.01,&Limits::default()).unwrap())
            .find(|b| (creation::measure(b,&Limits::default()).unwrap().area_mm2-100.*std::f64::consts::PI).abs()<1e-6).unwrap();
        let mut target=Entity::new("cylinder",Geometry::Brep(face),None); target.placement=Transform::translation([10.,0.,0.]); let id=target.id;
        let curve=if circle {om9_core::curve_build::circle_on_frame([0.,0.,3.],[1.,0.,0.],[0.,0.,1.],1.).unwrap()}
            else {Curve::Nurbs(om9_core::curve_build::ellipse([0.,0.,3.],[1.,0.,0.],[0.,0.,1.],1.,0.5,&Limits::default()).unwrap())};
        let mut rail=Entity::new("profile",Geometry::Curve(curve),None); rail.placement=Transform::translation([10.,0.,0.]); let rail_id=rail.id;
        let mut doc=Document::new("curved cylinder Boss"); doc.entities.insert(id,target); doc.entities.insert(rail_id,rail);
        let mut c=Controller::new(doc,Limits::default()).unwrap(); let before=c.document().clone();
        if circle { c.submit("Boss").unwrap(); } else { c.dispatch("OM9-SOLID-021",InvocationOrigin::Menu).unwrap(); }
        c.solid_pipe_pick_curve(rail_id).unwrap(); c.submit("").unwrap(); c.solid_boolean_pick(id,false).unwrap();
        let Geometry::Brep(body)=&c.document().entities[&id].geometry else {panic!()};
        let m=creation::measure(body,c.coordinator().limits()).unwrap(); assert_eq!(m.faces,if circle {2} else {5}); assert_eq!(m.shells,1);
        c.undo().unwrap(); assert_eq!(c.document(),&before);
    }
}

#[test]
fn boss_multiple_polygons_on_cylinder_is_atomic() {
    for overlap in [false,true] {
        let cylinder=creation::primitive(creation::Primitive::Cylinder {radius_mm:5.,height_mm:10.},0.01,&Limits::default()).unwrap();
        let count=creation::measure(&cylinder,&Limits::default()).unwrap().faces;
        let face=(0..count).map(|i| om9_kernel::editing::extract_face(&cylinder,i,0.01,&Limits::default()).unwrap())
            .find(|b| (creation::measure(b,&Limits::default()).unwrap().area_mm2-100.*std::f64::consts::PI).abs()<1e-6).unwrap();
        let target=Entity::new("cylinder",Geometry::Brep(face),None); let id=target.id;
        let a=om9_core::curve_build::polygon(&[[-1.,0.,2.],[1.,0.,2.],[1.,0.,3.],[-1.,0.,3.]]).unwrap();
        let z=if overlap {2.5} else {6.};
        let b=om9_core::curve_build::polygon(&[[-1.,0.,z],[1.,0.,z],[1.,0.,z+1.],[-1.,0.,z+1.]]).unwrap();
        let one=Entity::new("first",Geometry::Curve(a),None); let aid=one.id;
        let two=Entity::new("second",Geometry::Curve(b),None); let bid=two.id;
        let mut doc=Document::new("multi cylinder Boss"); doc.entities.insert(id,target); doc.entities.insert(aid,one); doc.entities.insert(bid,two);
        let mut c=Controller::new(doc,Limits::default()).unwrap(); let before=c.document().clone();
        c.submit("Boss").unwrap(); c.solid_pipe_pick_curve(aid).unwrap(); c.solid_pipe_pick_curve(bid).unwrap(); c.submit("").unwrap();
        if overlap {
            assert!(c.solid_boolean_pick(id,false).is_err()); assert_eq!(c.document(),&before); assert_eq!(c.coordinator().undo_len(),0);
        } else {
            c.solid_boolean_pick(id,false).unwrap();
            let Geometry::Brep(body)=&c.document().entities[&id].geometry else {panic!()};
            let m=creation::measure(body,c.coordinator().limits()).unwrap(); assert_eq!(m.faces,9); assert_eq!(m.shells,1);
            assert_eq!(c.coordinator().undo_len(),1); c.undo().unwrap(); assert_eq!(c.document(),&before);
        }
    }
}

#[test]
fn boss_oblique_curved_profiles_on_cylinder_commit_and_undo() {
    for (circle,reversed) in [(true,false),(true,true),(false,false),(false,true)] {
        let cylinder=creation::primitive(creation::Primitive::Cylinder {radius_mm:5.,height_mm:10.},0.01,&Limits::default()).unwrap();
        let count=creation::measure(&cylinder,&Limits::default()).unwrap().faces;
        let face=(0..count).map(|i| om9_kernel::editing::extract_face(&cylinder,i,0.01,&Limits::default()).unwrap())
            .find(|b| (creation::measure(b,&Limits::default()).unwrap().area_mm2-100.*std::f64::consts::PI).abs()<1e-6).unwrap();
        let mut boundary=Entity::new("cylinder face",Geometry::Brep(face),None); boundary.placement=Transform::translation([10.,0.,0.]); let id=boundary.id;
        let h=3.0_f64.sqrt()/2.; let v=if reversed {[0.,-0.5,-h]} else {[0.,0.5,h]};
        let curve=if circle {om9_core::curve_build::circle_on_frame([0.,0.,5.],[1.,0.,0.],v,1.).unwrap()}
            else {Curve::Nurbs(om9_core::curve_build::ellipse([0.,0.,5.],[1.,0.,0.],v,1.,0.5,&Limits::default()).unwrap())};
        let mut rail=Entity::new("profile",Geometry::Curve(curve),None); rail.placement=Transform::translation([10.,0.,0.]); let rail_id=rail.id;
        let mut doc=Document::new("oblique curved Boss"); doc.entities.insert(rail_id,rail); doc.entities.insert(id,boundary);
        let mut c=Controller::new(doc,Limits::default()).unwrap(); let before=c.document().clone();
        if reversed {c.dispatch("OM9-SOLID-021",InvocationOrigin::Menu).unwrap();} else {c.submit("Boss").unwrap();}
        c.solid_pipe_pick_curve(rail_id).unwrap(); c.submit("").unwrap(); c.solid_boolean_pick(id,false).unwrap();
        let Geometry::Brep(body)=&c.document().entities[&id].geometry else {panic!()};
        let m=creation::measure(body,c.coordinator().limits()).unwrap(); assert_eq!((m.faces,m.shells),(if circle {2} else {5},1));
        c.undo().unwrap(); assert_eq!(c.document(),&before);
    }
}

#[test]
fn boss_oblique_polygon_on_cylinder_commit_and_undo() {
    for reversed in [false,true] {
        let cylinder=creation::primitive(creation::Primitive::Cylinder {radius_mm:5.,height_mm:10.},0.01,&Limits::default()).unwrap();
        let count=creation::measure(&cylinder,&Limits::default()).unwrap().faces;
        let face=(0..count).map(|i| om9_kernel::editing::extract_face(&cylinder,i,0.01,&Limits::default()).unwrap())
            .find(|b| (creation::measure(b,&Limits::default()).unwrap().area_mm2-100.*std::f64::consts::PI).abs()<1e-6).unwrap();
        let mut boundary=Entity::new("cylinder face",Geometry::Brep(face),None); boundary.placement=Transform::translation([10.,0.,0.]); let id=boundary.id;
        let h=3.0_f64.sqrt()/2.; let mut points=vec![[-1.,0.,4.],[1.,0.,4.],[1.,0.5,4.+h],[-1.,0.5,4.+h]];
        if reversed {points.reverse();}
        let mut rail=Entity::new("profile",Geometry::Curve(om9_core::curve_build::polygon(&points).unwrap()),None); rail.placement=Transform::translation([10.,0.,0.]); let rail_id=rail.id;
        let mut doc=Document::new("oblique cylinder Boss"); doc.entities.insert(rail_id,rail); doc.entities.insert(id,boundary);
        let mut c=Controller::new(doc,Limits::default()).unwrap(); let before=c.document().clone();
        if reversed {c.dispatch("OM9-SOLID-021",InvocationOrigin::Menu).unwrap();} else {c.submit("Boss").unwrap();}
        c.solid_pipe_pick_curve(rail_id).unwrap(); c.submit("").unwrap(); c.solid_boolean_pick(id,false).unwrap();
        let Geometry::Brep(body)=&c.document().entities[&id].geometry else {panic!()};
        let m=creation::measure(body,c.coordinator().limits()).unwrap(); assert_eq!((m.faces,m.shells),(5,1));
        c.undo().unwrap(); assert_eq!(c.document(),&before);
    }
}

#[test]
fn boss_polygon_on_cylindrical_face_with_placement_and_undo() {
    for reversed in [false,true] {
    let cylinder=creation::primitive(creation::Primitive::Cylinder {radius_mm:5.,height_mm:10.},0.01,&Limits::default()).unwrap();
    let count=creation::measure(&cylinder,&Limits::default()).unwrap().faces;
    let face=(0..count).map(|i| om9_kernel::editing::extract_face(&cylinder,i,0.01,&Limits::default()).unwrap())
        .find(|b| (creation::measure(b,&Limits::default()).unwrap().area_mm2-100.*std::f64::consts::PI).abs()<1e-6).unwrap();
    let mut boundary=Entity::new("cylinder face",Geometry::Brep(face),None); boundary.placement=Transform::translation([10.,0.,0.]); let id=boundary.id;
    let points=if reversed { [[-1.,0.,4.],[1.,0.,4.],[1.,0.,2.],[-1.,0.,2.]] } else { [[-1.,0.,2.],[1.,0.,2.],[1.,0.,4.],[-1.,0.,4.]] };
    let curve=om9_core::curve_build::polygon(&points).unwrap();
    let mut rail=Entity::new("profile",Geometry::Curve(curve),None); rail.placement=Transform::translation([10.,0.,0.]); let rail_id=rail.id;
    let mut doc=Document::new("cylinder Boss"); doc.entities.insert(rail_id,rail); doc.entities.insert(id,boundary);
    let mut c=Controller::new(doc,Limits::default()).unwrap(); let before=c.document().clone();
    if reversed { c.dispatch("OM9-SOLID-021",InvocationOrigin::Menu).unwrap(); } else { c.submit("Boss").unwrap(); }
    c.solid_pipe_pick_curve(rail_id).unwrap(); c.submit("").unwrap(); c.solid_boolean_pick(id,false).unwrap();
    let Geometry::Brep(body)=&c.document().entities[&id].geometry else { panic!() };
    let m=creation::measure(body,c.coordinator().limits()).unwrap();
    let expected=100.*std::f64::consts::PI+6.*24.0_f64.sqrt()+30.*0.2_f64.asin();
    assert_eq!(m.faces,5); assert_eq!(m.shells,1); assert!((m.area_mm2-expected).abs()<1e-5,"{m:?}");
    c.undo().unwrap(); assert_eq!(c.document(),&before);
    }
}

#[test]
fn boss_piecewise_nurbs_near_inclined_boundary_with_undo() {
    let h=std::f64::consts::FRAC_1_SQRT_2;
    let e=om9_core::curve_build::ellipse([5.,5.,0.],[h,h,0.],[-h,h,0.],2.,1.,&Limits::default()).unwrap();
    for reversed in [false,true] {
        let segments=(0..4).map(|i| Curve::Nurbs(om9_core::geometry::NurbsCurve { degree:2,poles:e.poles[2*i..=2*i+2].to_vec(),weights:e.weights[2*i..=2*i+2].to_vec(),knots:vec![0.,0.,0.,1.,1.,1.],domain:[0.,1.],periodic:false,reversed:false,seam:None })).collect();
        let mut rail=Entity::new("ellipse",Geometry::Curve(Curve::Polycurve { segments,reversed }),None); rail.placement=Transform::translation([10.,0.,0.]); let rail_id=rail.id;
        let p=creation::Profile::new(vec![[0.,0.,-1.38],[10.,0.,0.62],[10.,10.,2.62],[0.,10.,0.62]]).unwrap();
        let mut face=Entity::new("inclined",Geometry::Brep(creation::planar(&p,0.01,&Limits::default()).unwrap()),None); face.placement=Transform::translation([10.,0.,0.]); let id=face.id;
        let mut doc=Document::new("piecewise near plane Boss"); doc.entities.insert(rail_id,rail); doc.entities.insert(id,face);
        let mut c=Controller::new(doc,Limits::default()).unwrap(); let before=c.document().clone();
        if reversed { c.dispatch("OM9-SOLID-021",InvocationOrigin::Menu).unwrap(); } else { c.submit("Boss").unwrap(); }
        c.solid_pipe_pick_curve(rail_id).unwrap(); c.submit("").unwrap(); c.solid_boolean_pick(id,false).unwrap();
        let Geometry::Brep(body)=&c.document().entities[&id].geometry else { panic!() };
        let m=creation::measure(body,c.coordinator().limits()).unwrap(); assert_eq!(m.faces,5); assert_eq!(m.shells,1);
        c.undo().unwrap(); assert_eq!(c.document(),&before);
    }
}

#[test]
fn boss_rotated_nurbs_near_inclined_boundary_with_undo() {
    let h = std::f64::consts::FRAC_1_SQRT_2;
    let curve = Curve::Nurbs(om9_core::curve_build::ellipse([5.,5.,0.],[h,h,0.],[-h,h,0.],2.,1.,&Limits::default()).unwrap());
    let mut rail = Entity::new("ellipse", Geometry::Curve(curve), None); rail.placement = Transform::translation([10.,0.,0.]); let rail_id=rail.id;
    let p = creation::Profile::new(vec![[0.,0.,-1.38],[10.,0.,0.62],[10.,10.,2.62],[0.,10.,0.62]]).unwrap();
    let mut face = Entity::new("inclined", Geometry::Brep(creation::planar(&p, 0.01, &Limits::default()).unwrap()), None);
    face.placement = Transform::translation([10.,0.,0.]); let id=face.id;
    let mut doc = Document::new("near plane Boss"); doc.entities.insert(rail_id,rail); doc.entities.insert(id,face);
    let mut c = Controller::new(doc, Limits::default()).unwrap(); let before=c.document().clone();
    c.submit("Boss").unwrap(); c.solid_pipe_pick_curve(rail_id).unwrap(); c.submit("").unwrap(); c.solid_boolean_pick(id,false).unwrap();
    let Geometry::Brep(body)=&c.document().entities[&id].geometry else { panic!() };
    let m=creation::measure(body,c.coordinator().limits()).unwrap();
    assert_eq!(m.faces,2); assert_eq!(m.shells,1);
    c.undo().unwrap(); assert_eq!(c.document(),&before);
}

#[test]
fn boss_curved_profiles_on_inclined_boundary_with_undo() {
    for circle in [false, true] {
        let profile = creation::Profile::new(vec![[0.,0.,5.],[10.,0.,7.],[10.,10.,7.],[0.,10.,5.]]).unwrap();
        let mut target = Entity::new("inclined", Geometry::Brep(creation::planar(&profile, 0.01, &Limits::default()).unwrap()), None);
        target.placement = Transform::translation([10.,0.,0.]); let id = target.id;
        let curve = if circle {
            om9_core::curve_build::circle_on_frame([5.,5.,0.],[1.,0.,0.],[0.,1.,0.],1.).unwrap()
        } else {
            Curve::Nurbs(om9_core::curve_build::ellipse([5.,5.,0.],[1.,0.,0.],[0.,1.,0.],2.,1.,&Limits::default()).unwrap())
        };
        let mut rail = Entity::new("profile", Geometry::Curve(curve), None); rail.placement = Transform::translation([10.,0.,0.]); let rail_id = rail.id;
        let mut doc = Document::new("curved Boss inclined"); doc.entities.insert(rail_id, rail); doc.entities.insert(id, target);
        let mut c = Controller::new(doc, Limits::default()).unwrap(); let before = c.document().clone();
        if circle { c.submit("Boss").unwrap(); } else { c.dispatch("OM9-SOLID-021", InvocationOrigin::Menu).unwrap(); }
        c.solid_pipe_pick_curve(rail_id).unwrap(); c.submit("").unwrap(); c.solid_boolean_pick(id, false).unwrap();
        let Geometry::Brep(body) = &c.document().entities[&id].geometry else { panic!() };
        let m = creation::measure(body, c.coordinator().limits()).unwrap();
        let dt = 2.*std::f64::consts::PI / 16384.;
        let perimeter: f64 = if circle { 2.*std::f64::consts::PI } else { (0..16384).map(|i| { let t=(i as f64+0.5)*dt; ((2.*t.sin()).powi(2)+t.cos().powi(2)).sqrt()*dt }).sum() };
        let area = if circle { std::f64::consts::PI } else { 2.*std::f64::consts::PI };
        let expected = (100.-area)*1.04_f64.sqrt()+6.*perimeter;
        assert_eq!(m.faces, 2); assert_eq!(m.shells, 1); assert!((m.area_mm2-expected).abs()<1e-5, "{m:?}, expected {expected}");
        c.undo().unwrap(); assert_eq!(c.document(), &before);
    }
}

#[test]
fn boss_polygon_inclined_boundary_with_placement_and_undo() {
    for sign in [-1., 1.] {
        let profile = creation::Profile::new(vec![[0.,0.,5.*sign],[10.,0.,7.*sign],[10.,10.,7.*sign],[0.,10.,5.*sign]]).unwrap();
        let mut target = Entity::new("inclined", Geometry::Brep(creation::planar(&profile, 0.01, &Limits::default()).unwrap()), None);
        target.placement = Transform::translation([10.,0.,0.]); let target_id = target.id;
        let curve = om9_core::curve_build::polygon(&[[1.,2.,0.],[3.,2.,0.],[3.,4.,0.],[1.,4.,0.]]).unwrap();
        let mut rail = Entity::new("profile", Geometry::Curve(curve), None); rail.placement = Transform::translation([10.,0.,0.]); let rail_id = rail.id;
        let mut doc = Document::new("inclined Boss"); doc.entities.insert(rail_id, rail); doc.entities.insert(target_id, target);
        let mut c = Controller::new(doc, Limits::default()).unwrap(); let before = c.document().clone();
        if sign > 0. { c.submit("Boss").unwrap(); } else { c.dispatch("OM9-SOLID-021", InvocationOrigin::Menu).unwrap(); }
        c.solid_pipe_pick_curve(rail_id).unwrap(); c.submit("").unwrap(); c.solid_boolean_pick(target_id, false).unwrap();
        let Geometry::Brep(body) = &c.document().entities[&target_id].geometry else { panic!() };
        let m = creation::measure(body, c.coordinator().limits()).unwrap();
        assert_eq!(m.faces, 5); assert_eq!(m.shells, 1); assert!((m.area_mm2-(96.*1.04_f64.sqrt()+43.2)).abs()<1e-5, "{m:?}");
        c.undo().unwrap(); assert_eq!(c.document(), &before);
    }
}

#[test]
fn boss_coplanar_two_face_boundary_commits_and_undoes() {
    let a = creation::Profile::new(vec![[0.,0.,0.],[5.,0.,0.],[5.,10.,0.],[0.,10.,0.]]).unwrap();
    let b = creation::Profile::new(vec![[5.,0.,0.],[10.,0.,0.],[10.,10.,0.],[5.,10.,0.]]).unwrap();
    let face = |p| creation::planar(p, 0.01, &Limits::default()).unwrap();
    let boundary = om9_kernel::editing::sew(&face(&a), &face(&b), 1e-7, 0.01, &Limits::default()).unwrap();
    let mut target = Entity::new("two faces", Geometry::Brep(boundary), None); target.placement = Transform::translation([10.,0.,5.]); let target_id = target.id;
    let curve = om9_core::curve_build::polygon(&[[1.,2.,0.],[3.,2.,0.],[3.,4.,0.],[1.,4.,0.]]).unwrap();
    let mut rail = Entity::new("profile", Geometry::Curve(curve), None); rail.placement = Transform::translation([10.,0.,0.]); let rail_id = rail.id;
    let mut doc = Document::new("Boss coplanar boundary"); doc.entities.insert(rail_id, rail); doc.entities.insert(target_id, target);
    let mut c = Controller::new(doc, Limits::default()).unwrap(); let before = c.document().clone();
    c.submit("Boss").unwrap(); c.solid_pipe_pick_curve(rail_id).unwrap(); c.submit("").unwrap(); c.solid_boolean_pick(target_id, false).unwrap();
    let Geometry::Brep(body) = &c.document().entities[&target_id].geometry else { panic!() };
    let m = creation::measure(body, c.coordinator().limits()).unwrap();
    assert_eq!(m.faces, 6); assert_eq!(m.shells, 1); assert!((m.area_mm2-136.).abs()<1e-6, "{m:?}");
    c.undo().unwrap(); assert_eq!(c.document(), &before);
}

#[test]
fn boss_mixed_nurbs_line_profile_and_reversal() {
    for (reversed, reversed_segment) in [(false,false), (false,true), (true,false), (true,true)] {
        let arch = Curve::Nurbs(om9_core::geometry::NurbsCurve {
            degree: 2, poles: vec![[3., 5., 0.], [5., 9., 0.], [7., 5., 0.]],
            weights: vec![1.; 3], knots: vec![0., 0., 0., 1., 1., 1.],
            domain: [0., 1.], periodic: false, reversed: reversed_segment, seam: None,
        });
        let line = Curve::Line { start: if reversed_segment { [3.,5.,0.] } else { [7.,5.,0.] }, end: if reversed_segment { [7.,5.,0.] } else { [3.,5.,0.] }, domain: [0., 1.] };
        let mut rail = Entity::new("NURBS profile", Geometry::Curve(Curve::Polycurve { segments: vec![arch, line], reversed }), None);
        rail.placement = Transform::translation([10., 0., 0.]); let rail_id = rail.id;
        let profile = creation::Profile::new(vec![[0.,0.,0.],[10.,0.,0.],[10.,10.,0.],[0.,10.,0.]]).unwrap();
        let mut face = Entity::new("boundary", Geometry::Brep(creation::planar(&profile, 0.01, &Limits::default()).unwrap()), None);
        face.placement = Transform::translation([10.,0.,5.]); let id = face.id;
        let mut doc = Document::new("Boss NURBS polycurve"); doc.entities.insert(rail_id, rail); doc.entities.insert(id, face);
        let mut c = Controller::new(doc, Limits::default()).unwrap(); let before = c.document().clone();
        if reversed { c.dispatch("OM9-SOLID-021", InvocationOrigin::Menu).unwrap(); } else { c.submit("Boss").unwrap(); }
        c.solid_pipe_pick_curve(rail_id).unwrap(); c.submit("").unwrap(); c.solid_boolean_pick(id, false).unwrap();
        let Geometry::Brep(body) = &c.document().entities[&id].geometry else { panic!() };
        let m = creation::measure(body, c.coordinator().limits()).unwrap();
        let perimeter = 4. + 2. * 5.0_f64.sqrt() + 2.0_f64.asinh();
        assert_eq!(m.faces, 3); assert!((m.area_mm2 - (100. - 16./3. + 5.*perimeter)).abs() < 1e-7, "{m:?}");
        c.undo().unwrap(); assert_eq!(c.document(), &before);
    }
}

#[test]
fn boss_mixed_arc_line_placed_profile_and_reversal() {
    for reversed in [false, true] {
        let arc = om9_core::curve_build::arc_by_center([5.0,5.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0],2.0,0.0,std::f64::consts::PI).unwrap();
        let line = Curve::Line { start: [3.0,5.0,0.0], end: [7.0,5.0,0.0], domain: [0.0,1.0] };
        let mut rail = Entity::new("arc profile", Geometry::Curve(Curve::Polycurve { segments: vec![arc,line], reversed }), None);
        rail.placement = Transform::translation([10.0,0.0,0.0]); let rail_id = rail.id;
        let profile = creation::Profile::new(vec![[0.0,0.0,0.0],[10.0,0.0,0.0],[10.0,10.0,0.0],[0.0,10.0,0.0]]).unwrap();
        let mut face = Entity::new("boundary", Geometry::Brep(creation::planar(&profile, 0.01, &Limits::default()).unwrap()), None);
        face.placement = Transform::translation([10.0,0.0,5.0]); let id = face.id;
        let mut doc = Document::new("Boss mixed arc"); doc.entities.insert(rail_id, rail); doc.entities.insert(id, face);
        let mut c = Controller::new(doc, Limits::default()).unwrap(); let before = c.document().clone();
        if reversed { c.dispatch("OM9-SOLID-021", InvocationOrigin::Menu).unwrap(); } else { c.submit("Boss").unwrap(); }
        c.solid_pipe_pick_curve(rail_id).unwrap_or_else(|e| panic!("select profile reversed={reversed}: {e:?}"));
        c.submit("").expect("confirm profile");
        c.solid_boolean_pick(id, false).expect("select boundary");
        let Geometry::Brep(body) = &c.document().entities[&id].geometry else { panic!() };
        let m = creation::measure(body, c.coordinator().limits()).unwrap();
        assert_eq!(m.faces, 3); assert!((m.area_mm2-(120.0+8.0*std::f64::consts::PI)).abs()<1e-5, "{m:?}");
        c.undo().unwrap(); assert_eq!(c.document(), &before);
    }
}

#[test]
fn boss_rational_ellipse_on_placed_boundary_preserves_geometry_and_undo() {
    let nurbs = om9_core::curve_build::ellipse([6.0,6.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0],2.0,3.0,&Limits::default()).unwrap();
    let mut rail = Entity::new("ellipse", Geometry::Curve(Curve::Nurbs(nurbs)), None);
    rail.placement = Transform::translation([10.0,0.0,0.0]); let rail_id = rail.id;
    let profile = creation::Profile::new(vec![[0.0,0.0,0.0],[12.0,0.0,0.0],[12.0,12.0,0.0],[0.0,12.0,0.0]]).unwrap();
    let mut boundary = Entity::new("boundary", Geometry::Brep(creation::planar(&profile, 0.01, &Limits::default()).unwrap()), None);
    boundary.placement = Transform::translation([10.0,0.0,5.0]); let id = boundary.id;
    let mut doc = Document::new("Boss ellipse"); doc.entities.insert(rail_id, rail); doc.entities.insert(id, boundary);
    let mut c = Controller::new(doc, Limits::default()).unwrap(); let before = c.document().clone();
    c.submit("Boss").unwrap(); c.solid_pipe_pick_curve(rail_id).unwrap(); c.submit("").unwrap(); c.solid_boolean_pick(id, false).unwrap();
    let Geometry::Brep(body) = &c.document().entities[&id].geometry else { panic!() };
    let m = creation::measure(body, c.coordinator().limits()).unwrap();
    let count = 4096;
    let step = std::f64::consts::TAU / count as f64;
    let length: f64 = (0..count).map(|i| { let t = (i as f64+0.5)*step; ((2.0*t.sin()).powi(2)+(3.0*t.cos()).powi(2)).sqrt()*step }).sum();
    let expected = 144.0 - 6.0*std::f64::consts::PI + 5.0*length;
    let curve = match &c.document().entities[&rail_id].geometry { Geometry::Curve(c) => c, _ => panic!() };
    let wall = om9_kernel::extrusion::shell(curve, [0.0,0.0,5.0], 0.01, c.coordinator().limits()).unwrap();
    let wall_area = creation::measure(&wall, c.coordinator().limits()).unwrap().area_mm2;
    assert_eq!(m.faces, 2);
    assert!((m.area_mm2-(144.0-6.0*std::f64::consts::PI+wall_area)).abs() < 1e-5, "{m:?}");
    assert!((m.area_mm2-expected).abs() < 1e-5, "{m:?} expected {expected}");
    c.undo().unwrap(); assert_eq!(c.document(), &before);
}

#[test]
fn boss_mixed_circle_and_polygon_preserves_exact_area_with_placement() {
    let mut doc = Document::new("Boss mixed sections");
    let curve = om9_core::curve_build::circle_on_frame([2.0,2.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0],1.0).unwrap();
    let mut circle = Entity::new("circle", Geometry::Curve(curve), None); circle.placement = Transform::translation([10.0,0.0,0.0]);
    let a = circle.id; doc.entities.insert(a, circle);
    let curve = om9_core::curve_build::polygon(&[[6.0,6.0,0.0],[8.0,6.0,0.0],[8.0,8.0,0.0],[6.0,8.0,0.0]]).unwrap();
    let mut poly = Entity::new("polygon", Geometry::Curve(curve), None); poly.placement = Transform::translation([10.0,0.0,0.0]);
    let b = poly.id; doc.entities.insert(b, poly);
    let face = creation::planar(&creation::Profile::new(vec![[0.0,0.0,0.0],[10.0,0.0,0.0],[10.0,10.0,0.0],[0.0,10.0,0.0]]).unwrap(), 0.01, &Limits::default()).unwrap();
    let mut boundary = Entity::new("face", Geometry::Brep(face), None); boundary.placement = Transform::translation([10.0,0.0,5.0]);
    let id = boundary.id; doc.entities.insert(id, boundary);
    let mut c = Controller::new(doc, Limits::default()).unwrap(); let before = c.document().clone();
    c.submit("Boss").unwrap(); c.solid_pipe_pick_curve(a).unwrap(); c.solid_pipe_pick_curve(b).unwrap(); c.submit("").unwrap();
    c.solid_boolean_pick(id, false).unwrap();
    let Geometry::Brep(body) = &c.document().entities[&id].geometry else { panic!() };
    let m = creation::measure(body, c.coordinator().limits()).unwrap();
    assert_eq!(m.faces, 6); assert!((m.area_mm2-(136.0+9.0*std::f64::consts::PI)).abs()<1e-6, "{m:?}");
    c.undo().unwrap(); assert_eq!(c.document(), &before);
}

#[test]
fn boss_multiple_profiles_commit_once_and_overlap_fails_atomically() {
    for overlap in [false, true] {
        let mut doc = Document::new("Boss multiple"); let mut ids = Vec::new();
        for x in if overlap { [1.0, 2.0] } else { [1.0, 6.0] } {
            let curve = om9_core::curve_build::polygon(&[[x,1.0,0.0],[x+2.0,1.0,0.0],[x+2.0,3.0,0.0],[x,3.0,0.0]]).unwrap();
            let entity = Entity::new("profile", Geometry::Curve(curve), None); ids.push(entity.id); doc.entities.insert(entity.id, entity);
        }
        let face = creation::planar(&creation::Profile::new(vec![[0.0,0.0,5.0],[10.0,0.0,5.0],[10.0,10.0,5.0],[0.0,10.0,5.0]]).unwrap(), 0.01, &Limits::default()).unwrap();
        let boundary = Entity::new("boundary", Geometry::Brep(face), None); let boundary_id = boundary.id; doc.entities.insert(boundary_id, boundary);
        let mut c = Controller::new(doc, Limits::default()).unwrap(); let before = c.document().clone();
        c.submit("Boss").unwrap();
        for id in &ids { c.solid_pipe_pick_curve(*id).unwrap(); }
        c.submit("Undo").unwrap(); c.solid_pipe_pick_curve(ids[1]).unwrap(); c.submit("").unwrap();
        let result = c.solid_boolean_pick(boundary_id, false);
        if overlap { assert!(result.is_err()); assert_eq!(c.document(), &before); assert_eq!(c.coordinator().undo_len(), 0); }
        else {
            result.unwrap();
            let Geometry::Brep(body) = &c.document().entities[&boundary_id].geometry else { panic!() };
            let m = creation::measure(body, c.coordinator().limits()).unwrap();
            assert_eq!(m.faces, 9); assert!((m.area_mm2-172.0).abs()<1e-6, "{m:?}");
            assert_eq!(c.coordinator().undo_len(), 1); c.undo().unwrap(); assert_eq!(c.document(), &before);
        }
    }
}

#[test]
fn boss_selection_cancel_and_invalid_boundary_are_atomic() {
    let curve = om9_core::curve_build::polygon(&[[0.0,0.0,0.0],[2.0,0.0,0.0],[2.0,2.0,0.0],[0.0,2.0,0.0]]).unwrap();
    let rail = Entity::new("profile", Geometry::Curve(curve), None); let id = rail.id;
    let mut arc = om9_core::curve_build::circle_on_frame([0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0],1.0).unwrap();
    if let Curve::Circle { domain, .. } = &mut arc { *domain = [0.0, std::f64::consts::PI]; }
    let circle = Entity::new("arc", Geometry::Curve(arc), None);
    let circle_id = circle.id;
    let solid = Entity::new("solid", Geometry::Brep(creation::primitive(creation::Primitive::Box { x_mm: 10.0, y_mm: 10.0, z_mm: 10.0 }, 0.01, &Limits::default()).unwrap()), None);
    let solid_id = solid.id; let mut doc = Document::new("Boss invalid");
    doc.entities.insert(id, rail); doc.entities.insert(circle_id, circle); doc.entities.insert(solid_id, solid);
    let mut c = Controller::new(doc, Limits::default()).unwrap(); let before = c.document().clone();
    c.submit("Boss").unwrap(); c.solid_pipe_pick_curve(circle_id).unwrap();
    assert!(c.submit("").is_err()); assert_eq!(c.document(), &before);
    c.submit("Undo").unwrap();
    c.solid_pipe_pick_curve(id).unwrap(); c.submit("").unwrap();
    assert!(c.solid_boolean_pick(solid_id, false).is_err());
    assert_eq!(c.document(), &before); assert_eq!(c.coordinator().undo_len(), 0);
    c.submit("Undo").unwrap(); c.submit("Cancel").unwrap();
    assert_eq!(c.document(), &before); assert_eq!(c.coordinator().undo_len(), 0);
}

#[test]
fn boss_polygon_and_placed_planar_boundary_commit_and_undo() {
    for via_menu in [false, true] {
        let profile = om9_core::curve_build::polygon(&[[4.0,4.0,0.0], [6.0,4.0,0.0], [6.0,6.0,0.0], [4.0,6.0,0.0]]).unwrap();
        let mut rail = Entity::new("profile", Geometry::Curve(profile), None);
        rail.placement = Transform::translation([10.0, 0.0, 0.0]); let profile_id = rail.id;
        let polygon = creation::Profile::new(vec![[0.0,0.0,0.0], [10.0,0.0,0.0], [10.0,10.0,0.0], [0.0,10.0,0.0]]).unwrap();
        let face = creation::planar(&polygon, 0.01, &Limits::default()).unwrap();
        let mut boundary = Entity::new("boundary", Geometry::Brep(face), None);
        boundary.placement = Transform::translation([10.0, 0.0, 5.0]); let boundary_id = boundary.id;
        let mut doc = Document::new("Boss"); doc.entities.insert(profile_id, rail); doc.entities.insert(boundary_id, boundary);
        let mut c = Controller::new(doc, Limits::default()).unwrap(); let before = c.document().clone();
        if via_menu { c.dispatch("OM9-SOLID-021", InvocationOrigin::Menu).unwrap(); } else { c.submit("Boss").unwrap(); }
        c.solid_pipe_pick_curve(profile_id).unwrap(); c.submit("").unwrap();
        assert_eq!(c.coordinator().undo_len(), 0);
        c.solid_boolean_pick(boundary_id, false).unwrap();
        let Geometry::Brep(body) = &c.document().entities[&boundary_id].geometry else { panic!() };
        let m = creation::measure(body, c.coordinator().limits()).unwrap();
        assert_eq!(m.faces, 5); assert_eq!(m.shells, 1); assert!((m.area_mm2 - 136.0).abs() < 1e-6, "{m:?}");
        assert_eq!(c.coordinator().undo_len(), 1);
        c.undo().unwrap(); assert_eq!(c.document(), &before);
    }
}

#[test]
fn pipe_full_circle_respects_reflected_scaled_placement_for_all_caps() {
    for cap in ["None", "Flat", "Round"] {
        for thick in [false, true] {
            let curve = om9_core::curve_build::circle_on_frame([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 5.0).unwrap();
            let mut rail = Entity::new("circle", Geometry::Curve(curve), None);
            rail.placement = Transform([[-2.0, 0.0, 0.0, 10.0], [0.0, 2.0, 0.0, 20.0], [0.0, 0.0, 2.0, 30.0], [0.0, 0.0, 0.0, 1.0]]);
            let id = rail.id; let mut doc = Document::new("Scaled circular Pipe"); doc.entities.insert(id, rail);
            let mut c = Controller::new(doc, Limits::default()).unwrap();
            let before = c.document().clone(); c.select_objects(&[id]).unwrap(); c.submit("Pipe").unwrap();
            c.submit(&format!("Cap={cap}")).unwrap();
            if thick { c.submit("Thick=Yes").unwrap(); for r in [0.5, 1.0, 0.5, 1.0] { c.submit(&r.to_string()).unwrap(); } }
            else { c.submit("1").unwrap(); c.submit("1").unwrap(); }
            c.submit("").unwrap();
            let body = c.document().entities.values().find_map(|e| match &e.geometry { Geometry::Brep(b) => Some(b), _ => None }).unwrap();
            let m = creation::measure(body, c.coordinator().limits()).unwrap();
            let expected_area = (if thick { 60.0 } else { 40.0 }) * std::f64::consts::PI.powi(2);
            assert!((m.area_mm2 - expected_area).abs() < 1e-5, "{cap} thick={thick}: {m:?}");
            if cap == "None" { assert_eq!(m.solids, 0); assert_eq!(m.naked_edges, 0); }
            else {
                let expected_volume = (if thick { 15.0 } else { 20.0 }) * std::f64::consts::PI.powi(2);
                assert!(m.closed && m.solids == 1 && (m.volume_mm3 - expected_volume).abs() < 1e-5, "{cap} thick={thick}: {m:?}");
            }
            for (actual, expected) in m.bounds_mm.iter().zip([-1.0, 9.0, 29.0, 21.0, 31.0, 31.0]) {
                assert!((actual-expected).abs() < 1e-5, "{cap} thick={thick}: {m:?}");
            }
            c.undo().unwrap(); assert_eq!(c.document(), &before);
        }
    }
}

#[test]
fn pipe_full_circle_round_has_no_end_caps() {
    for thick in [false, true] {
        let curve = om9_core::curve_build::circle_on_frame([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 5.0).unwrap();
        let rail = Entity::new("circle", Geometry::Curve(curve), None);
        let id = rail.id; let mut doc = Document::new("Closed Round Pipe"); doc.entities.insert(id, rail);
        let mut c = Controller::new(doc, Limits::default()).unwrap();
        let before = c.document().clone(); c.select_objects(&[id]).unwrap(); c.submit("Pipe").unwrap();
        c.submit("Cap=Round").unwrap();
        if thick { c.submit("Thick=Yes").unwrap(); for r in [0.5, 1.0, 0.5, 1.0] { c.submit(&r.to_string()).unwrap(); } }
        else { c.submit("1").unwrap(); c.submit("1").unwrap(); }
        c.submit("").unwrap();
        let body = c.document().entities.values().find_map(|e| match &e.geometry { Geometry::Brep(b) => Some(b), _ => None }).unwrap();
        let m = creation::measure(body, c.coordinator().limits()).unwrap();
        let expected = (if thick { 7.5 } else { 10.0 }) * std::f64::consts::PI.powi(2);
        assert!(m.closed && m.solids == 1 && (m.volume_mm3 - expected).abs() < 1e-6, "{m:?}");
        c.undo().unwrap(); assert_eq!(c.document(), &before);
    }
}

#[test]
fn pipe_full_circle_no_cap_builds_exact_surface_walls() {
    for thick in [false, true] {
        let curve = om9_core::curve_build::circle_on_frame([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 5.0).unwrap();
        let rail = Entity::new("circle", Geometry::Curve(curve), None);
        let id = rail.id;
        let mut doc = Document::new("Uncapped circular Pipe"); doc.entities.insert(id, rail);
        let mut c = Controller::new(doc, Limits::default()).unwrap();
        let before = c.document().clone(); c.select_objects(&[id]).unwrap(); c.submit("Pipe").unwrap();
        c.submit("Cap=None").unwrap();
        if thick { c.submit("Thick=Yes").unwrap(); for v in [0.5, 1.0, 0.5, 1.0] { c.submit(&v.to_string()).unwrap(); } }
        else { c.submit("1").unwrap(); c.submit("1").unwrap(); }
        c.submit("").unwrap();
        let body = c.document().entities.values().find_map(|e| match &e.geometry { Geometry::Brep(b) => Some(b), _ => None }).unwrap();
        let m = creation::measure(body, c.coordinator().limits()).unwrap();
        let expected = (if thick { 30.0 } else { 20.0 }) * std::f64::consts::PI.powi(2);
        assert_eq!(m.solids, 0, "{m:?}");
        assert_eq!(m.naked_edges, 0, "{m:?}");
        assert!((m.area_mm2 - expected).abs() < 1e-6, "{m:?}");
        if thick {
            let mut signed = [0.0_f64; 2];
            for [ia, ib, ic, _] in &body.display.triangles {
                let a = body.display.points[*ia as usize]; let b = body.display.points[*ib as usize]; let c = body.display.points[*ic as usize];
                let p = std::array::from_fn::<_, 3, _>(|i| (a[i] + b[i] + c[i]) / 3.0);
                let u = std::array::from_fn::<_, 3, _>(|i| b[i] - a[i]); let v = std::array::from_fn::<_, 3, _>(|i| c[i] - a[i]);
                let n = [u[1]*v[2]-u[2]*v[1], u[2]*v[0]-u[0]*v[2], u[0]*v[1]-u[1]*v[0]];
                let r = (p[0]*p[0] + p[1]*p[1]).sqrt();
                let q = [p[0] - 5.0*p[0]/r, p[1] - 5.0*p[1]/r, p[2]];
                let wall = usize::from((q[0]*q[0] + q[1]*q[1] + q[2]*q[2]).sqrt() > 0.75);
                signed[wall] += n[0]*q[0] + n[1]*q[1] + n[2]*q[2];
            }
            assert!(signed[0] * signed[1] < 0.0, "wall orientations {signed:?}");
        }
        c.undo().unwrap(); assert_eq!(c.document(), &before);
    }
}

#[test]
fn pipe_full_circle_thick_builds_exact_hollow_torus() {
    for via_menu in [false, true] {
        let curve = om9_core::curve_build::circle_on_frame([1.0, 2.0, 3.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0], 5.0).unwrap();
        let mut rail = Entity::new("circle", Geometry::Curve(curve), None);
        rail.placement = Transform::translation([10.0, 20.0, 30.0]);
        let id = rail.id;
        let mut doc = Document::new("Thick circular Pipe"); doc.entities.insert(id, rail);
        let mut c = Controller::new(doc, Limits::default()).unwrap();
        let before = c.document().clone();
        c.select_objects(&[id]).unwrap();
        if via_menu { c.dispatch("OM9-SOLID-011", InvocationOrigin::Menu).unwrap(); } else { c.submit("Pipe").unwrap(); }
        c.submit("Thick=Yes").unwrap();
        for radius in [0.5, 1.0, 1.0, 0.5] { c.submit(&radius.to_string()).unwrap(); }
        c.submit("").unwrap();
        let body = c.document().entities.values().find_map(|e| match &e.geometry { Geometry::Brep(b) => Some(b), _ => None }).unwrap();
        let m = creation::measure(body, c.coordinator().limits()).unwrap();
        assert!(m.closed && m.solids == 1 && (m.volume_mm3 - 7.5 * std::f64::consts::PI.powi(2)).abs() < 1e-6, "{m:?}");
        assert!((m.area_mm2 - 30.0 * std::f64::consts::PI.powi(2)).abs() < 1e-6, "{m:?}");
        assert_eq!(c.coordinator().undo_len(), 1);
        c.undo().unwrap(); assert_eq!(c.document(), &before);
    }
}

#[test]
fn pipe_full_circle_builds_exact_torus_with_placed_frame() {
    for via_menu in [false, true] {
    let curve = om9_core::curve_build::circle_on_frame([1.0, 2.0, 3.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0], 5.0).unwrap();
    let mut rail = Entity::new("circle", Geometry::Curve(curve), None);
    rail.placement = Transform::translation([10.0, 20.0, 30.0]);
    let id = rail.id;
    let mut doc = Document::new("Circular Pipe"); doc.entities.insert(id, rail);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    let before = c.document().clone();
    c.select_objects(&[id]).unwrap();
    if via_menu { c.dispatch("OM9-SOLID-011", InvocationOrigin::Menu).unwrap(); }
    else { c.submit("Pipe").unwrap(); }
    c.submit("1").unwrap(); c.submit("1").unwrap(); c.submit("").unwrap();
    let body = c.document().entities.values().find_map(|e| match &e.geometry { Geometry::Brep(b) => Some(b), _ => None }).unwrap();
    let m = creation::measure(body, c.coordinator().limits()).unwrap();
    let expected = 10.0 * std::f64::consts::PI.powi(2);
    assert!(m.closed && m.solids == 1 && (m.volume_mm3 - expected).abs() < 1e-6, "{m:?}");
    assert!((m.area_mm2 - 20.0 * std::f64::consts::PI.powi(2)).abs() < 1e-6, "{m:?}");
    for (actual, expected) in m.bounds_mm.iter().zip([5.0, 21.0, 27.0, 17.0, 23.0, 39.0]) {
        assert!((actual - expected).abs() < 1e-5, "{m:?}");
    }
    c.undo().unwrap(); assert_eq!(c.document(), &before);
    }
}

#[test]
fn pipe_thick_allows_reversed_endpoint_radius_order_within_tolerance() {
    for bent in [false, true] {
        let mut doc = Document::new("Pipe radius order");
        let mut ids = Vec::new();
        let points = if bent { vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [10.0, 10.0, 0.0]] }
            else { vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0]] };
        for segment in points.windows(2) {
            let e = Entity::new("rail", Geometry::Curve(Curve::Line { start: segment[0], end: segment[1], domain: [0.0, 1.0] }), None);
            ids.push(e.id); doc.entities.insert(e.id, e);
        }
        let mut c = Controller::new(doc, Limits::default()).unwrap();
        let before = c.document().clone();
        c.submit("PipeChain").unwrap();
        for id in ids { c.solid_pipe_pick_curve(id).unwrap(); }
        c.submit("").unwrap(); c.submit("Thick=Yes").unwrap();
        for value in [0.5, 1.0, 1.000004, 0.500004] { c.submit(&value.to_string()).unwrap(); }
        c.submit("").unwrap();
        let body = c.document().entities.values().find_map(|e| match &e.geometry { Geometry::Brep(b) => Some(b), _ => None }).unwrap();
        let m = creation::measure(body, c.coordinator().limits()).unwrap();
        let expected = (if bent { 20.0 } else { 10.0 }) * 0.75 * std::f64::consts::PI;
        assert!(m.closed && (m.volume_mm3 - expected).abs() < 1e-5, "{m:?}");
        c.undo().unwrap(); assert_eq!(c.document(), &before);
    }
}

#[test]
fn pipe_bent_round_short_obtuse_end_rejects_without_commit() {
    let mut doc = Document::new("Short obtuse Pipe end");
    let mut ids = Vec::new();
    for (a, b) in [([0.0, 0.0, 0.0], [1.2, 0.0, 0.0]), ([1.2, 0.0, 0.0], [-3.8, 8.660254037844386, 0.0])] {
        let e = Entity::new("rail", Geometry::Curve(Curve::Line { start: a, end: b, domain: [0.0, 1.0] }), None);
        ids.push(e.id); doc.entities.insert(e.id, e);
    }
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    let before = c.document().clone();
    c.submit("PipeChain").unwrap();
    for id in ids { c.solid_pipe_pick_curve(id).unwrap(); }
    c.submit("").unwrap(); c.submit("Cap=Round").unwrap();
    c.submit("1").unwrap(); c.submit("1").unwrap();
    let error = c.submit("").expect_err("short end must not create a malformed cap");
    println!("short obtuse end: {error:?}");
    assert_eq!(c.document(), &before); assert_eq!(c.coordinator().undo_len(), 0);
}

#[test]
fn pipe_bent_chain_non_right_angles_and_spatial_rail_have_measured_round_caps() {
    let paths: &[&[[f64; 3]]] = &[
        &[[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [17.071067811865476, 7.071067811865475, 0.0]],
        &[[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [5.0, 8.660254037844386, 0.0]],
        &[[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [10.0, 10.0, 0.0], [10.0, 10.0, 10.0]],
    ];
    for path in paths {
        for thick in [false, true] {
            let mut doc = Document::new("Bent Pipe geometry");
            let mut ids = Vec::new();
            for segment in path.windows(2) {
                let e = Entity::new("rail", Geometry::Curve(Curve::Line { start: segment[0], end: segment[1], domain: [0.0, 1.0] }), None);
                ids.push(e.id); doc.entities.insert(e.id, e);
            }
            let mut c = Controller::new(doc, Limits::default()).unwrap();
            let before = c.document().clone();
            c.submit("PipeChain").unwrap();
            for id in ids { c.solid_pipe_pick_curve(id).unwrap(); }
            c.submit("").unwrap(); c.submit("Cap=Round").unwrap();
            if thick {
                c.submit("Thick=Yes").unwrap();
                for value in [0.5, 1.0, 0.5, 1.0] { c.submit(&value.to_string()).unwrap(); }
            } else { c.submit("1").unwrap(); c.submit("1").unwrap(); }
            c.submit("").unwrap();
            let body = c.document().entities.values().find_map(|e| match &e.geometry { Geometry::Brep(b) => Some(b), _ => None }).unwrap();
            let m = creation::measure(body, c.coordinator().limits()).unwrap();
            let length = 10.0 * (path.len() - 1) as f64;
            let expected = if thick { (0.75 * length + 7.0 / 6.0) * std::f64::consts::PI }
                else { (length + 4.0 / 3.0) * std::f64::consts::PI };
            assert!(m.closed && m.solids == 1 && (m.volume_mm3 - expected).abs() < 1e-5, "{path:?} thick={thick}: {m:?}, expected={expected}");
            c.undo().unwrap(); assert_eq!(c.document(), &before);
        }
    }
}

#[test]
fn pipe_straight_chain_global_preserves_small_radius_difference() {
    let mut doc = Document::new("Straight Pipe small taper");
    let rail = Entity::new("rail", Geometry::Curve(Curve::Line { start: [0.0, 0.0, 0.0], end: [10.0, 0.0, 0.0], domain: [0.0, 1.0] }), None);
    let id = rail.id;
    doc.entities.insert(id, rail);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.submit("PipeChain").unwrap();
    c.solid_pipe_pick_curve(id).unwrap();
    c.submit("").unwrap();
    c.submit("ShapeBlending=Global").unwrap();
    c.submit("1").unwrap(); c.submit("1.0005").unwrap(); c.submit("").unwrap();
    let body = c.document().entities.values().find_map(|e| match &e.geometry { Geometry::Brep(b) => Some(b), _ => None }).unwrap();
    let m = creation::measure(body, c.coordinator().limits()).unwrap();
    let expected = std::f64::consts::PI * 10.0 * (1.0 + 1.0005 + 1.0005_f64.powi(2)) / 3.0;
    assert!((m.volume_mm3 - expected).abs() < 1e-7, "{m:?}");
}

#[test]
fn pipe_bent_chain_endpoint_points_use_local_segment_tangents() {
    for thick in [false, true] {
        let mut doc = Document::new("Bent Pipe endpoint points");
        let mut ids = Vec::new();
        for (a, b) in [([0.0, 0.0, 0.0], [10.0, 0.0, 0.0]), ([10.0, 0.0, 0.0], [10.0, 10.0, 0.0])] {
            let entity = Entity::new("rail", Geometry::Curve(Curve::Line { start: a, end: b, domain: [0.0, 1.0] }), None);
            ids.push(entity.id); doc.entities.insert(entity.id, entity);
        }
        let mut c = Controller::new(doc, Limits::default()).unwrap();
        let before = c.document().clone();
        c.submit("PipeChain").unwrap();
        for id in ids { c.solid_pipe_pick_curve(id).unwrap(); }
        c.submit("").unwrap();
        if thick { c.submit("Thick=Yes").unwrap(); }
        assert!(c.submit("2,0,0").is_err());
        let start_preview=c.solid_preview([2.,1.,0.]); assert_eq!(start_preview.len(),64); assert!((start_preview[0][0][0]).abs()<1e-9 && (start_preview[0][0][1]-1.).abs()<1e-9);
        if thick {
            c.submit("2,0.5,0").unwrap(); c.submit("2,1,0").unwrap();
            let end_preview=c.solid_preview([11.,8.,0.]); assert_eq!(end_preview.len(),64); assert!((end_preview[0][0][0]-11.).abs()<1e-9 && (end_preview[0][0][1]-10.).abs()<1e-9);
            c.submit("10.5,8,0").unwrap(); c.submit("11,8,0").unwrap();
        } else {
            c.submit("2,1,0").unwrap(); let end_preview=c.solid_preview([11.,8.,0.]); assert_eq!(end_preview.len(),64); assert!((end_preview[0][0][0]-11.).abs()<1e-9 && (end_preview[0][0][1]-10.).abs()<1e-9); c.submit("11,8,0").unwrap();
        }
        c.submit("").unwrap();
        let body = c.document().entities.values().find_map(|e| match &e.geometry { Geometry::Brep(b) => Some(b), _ => None }).unwrap();
        let m = creation::measure(body, c.coordinator().limits()).unwrap();
        let expected = if thick { 15.0 * std::f64::consts::PI } else { 20.0 * std::f64::consts::PI };
        assert!((m.volume_mm3 - expected).abs() < 1e-5, "{m:?}");
        c.undo().unwrap(); assert_eq!(c.document(), &before);
    }
}

#[test]
fn pipe_bent_round_rejects_short_endpoint_segment_without_mutation() {
    let mut doc = Document::new("Bent Pipe short end");
    let mut ids = Vec::new();
    for (a, b) in [([0.0, 0.0, 0.0], [0.5, 0.0, 0.0]), ([0.5, 0.0, 0.0], [0.5, 5.0, 0.0])] {
        let entity = Entity::new("rail", Geometry::Curve(Curve::Line { start: a, end: b, domain: [0.0, 1.0] }), None);
        ids.push(entity.id); doc.entities.insert(entity.id, entity);
    }
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    let before = c.document().clone();
    c.submit("PipeChain").unwrap();
    for id in ids { c.solid_pipe_pick_curve(id).unwrap(); }
    c.submit("").unwrap(); c.submit("Cap=Round").unwrap();
    c.submit("1").unwrap(); c.submit("1").unwrap();
    assert!(c.submit("").is_err());
    assert_eq!(c.document(), &before); assert_eq!(c.coordinator().undo_len(), 0);
    c.submit("Cancel").unwrap();
}

#[test]
fn pipe_bent_thick_rejects_varying_radii_and_allows_retry() {
    let mut doc = Document::new("Bent Thick Pipe retry");
    let mut ids = Vec::new();
    for (a, b) in [([0.0, 0.0, 0.0], [10.0, 0.0, 0.0]), ([10.0, 0.0, 0.0], [10.0, 10.0, 0.0])] {
        let entity = Entity::new("rail", Geometry::Curve(Curve::Line { start: a, end: b, domain: [0.0, 1.0] }), None);
        ids.push(entity.id); doc.entities.insert(entity.id, entity);
    }
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    let before = c.document().clone();
    c.submit("PipeChain").unwrap();
    for id in ids { c.solid_pipe_pick_curve(id).unwrap(); }
    c.submit("").unwrap(); c.submit("Thick=Yes").unwrap(); c.submit("ShapeBlending=Global").unwrap();
    for value in [1.0, 2.0, 1.0, 3.0] { c.submit(&value.to_string()).unwrap(); }
    assert!(c.submit("").is_err());
    assert_eq!(c.document(), &before); assert_eq!(c.coordinator().undo_len(), 0);
    c.submit("Undo").unwrap(); c.submit("2").unwrap(); c.submit("").unwrap();
    let body = c.document().entities.values().find_map(|e| match &e.geometry { Geometry::Brep(b) => Some(b), _ => None }).unwrap();
    let m = creation::measure(body, c.coordinator().limits()).unwrap();
    assert!((m.volume_mm3 - 60.0 * std::f64::consts::PI).abs() < 1e-5, "{m:?}");
    c.undo().unwrap(); assert_eq!(c.document(), &before);
}

#[test]
fn pipe_bent_chain_rejects_unimplemented_variants_without_mutation() {
    let mut doc = Document::new("Bent Pipe unsupported variants");
    let mut ids = Vec::new();
    for (a, b) in [([0.0, 0.0, 0.0], [10.0, 0.0, 0.0]), ([10.0, 0.0, 0.0], [10.0, 10.0, 0.0])] {
        let entity = Entity::new("rail", Geometry::Curve(Curve::Line { start: a, end: b, domain: [0.0, 1.0] }), None);
        ids.push(entity.id); doc.entities.insert(entity.id, entity);
    }
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    let before = c.document().clone();
    c.submit("PipeChain").unwrap();
    for id in ids { c.solid_pipe_pick_curve(id).unwrap(); }
    c.submit("").unwrap(); c.submit("ShapeBlending=Global").unwrap();
    c.submit("1").unwrap(); c.submit("2").unwrap();
    assert!(c.submit("").is_err());
    assert_eq!(c.document(), &before); assert_eq!(c.coordinator().undo_len(), 0);
    c.submit("Undo").unwrap(); c.submit("1").unwrap();
    assert!(c.submit("Cap=Invalid").is_err());
    assert_eq!(c.document(), &before); assert_eq!(c.coordinator().undo_len(), 0);
    c.submit("Cap=Flat").unwrap(); c.submit("").unwrap();
    let result = c.document().entities.values().find_map(|e| match &e.geometry { Geometry::Brep(b) => Some(b), _ => None }).unwrap();
    let m = creation::measure(result, c.coordinator().limits()).unwrap();
    assert!((m.volume_mm3 - 20.0 * std::f64::consts::PI).abs() < 1e-5);
    c.undo().unwrap(); assert_eq!(c.document(), &before);
}

#[test]
fn pipe_chain_rejects_disconnected_and_reversing_without_commit() {
    let mut doc = Document::new("invalid Pipe chain");
    let mut ids = Vec::new();
    for (a, b) in [([0.0, 0.0, 0.0], [5.0, 0.0, 0.0]),
                   ([8.0, 0.0, 0.0], [10.0, 0.0, 0.0]),
                   ([5.0, 0.0, 0.0], [0.0, 0.0, 0.0])] {
        let entity = Entity::new("rail", Geometry::Curve(Curve::Line { start: a, end: b, domain: [0.0, 1.0] }), None);
        ids.push(entity.id); doc.entities.insert(entity.id, entity);
    }
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    let before = c.document().clone();
    c.submit("PipeChain").unwrap();
    c.solid_pipe_pick_curve(ids[0]).unwrap(); c.solid_pipe_pick_curve(ids[1]).unwrap();
    assert!(c.submit("").is_err());
    c.submit("Undo").unwrap(); c.solid_pipe_pick_curve(ids[2]).unwrap();
    assert!(c.submit("").is_err());
    assert_eq!(c.coordinator().undo_len(), 0);
    assert_eq!(c.document(), &before);
    c.submit("Cancel").unwrap();
}

#[test]
fn pipe_multiple_global_stations_apply_to_straight_rails_atomically() {
    for mixed in [false,true] {
        let mut doc=Document::new("multiple station pipes"); let mut ids=Vec::new();
        let first=Entity::new("first",Geometry::Curve(Curve::Line {start:[0.,0.,0.],end:[10.,0.,0.],domain:[0.,1.]}),None); ids.push(first.id); doc.entities.insert(first.id,first);
        let curve=if mixed {om9_core::curve_build::circle_on_frame([20.,0.,0.],[1.,0.,0.],[0.,1.,0.],5.).unwrap()}
            else {Curve::Line {start:[0.,20.,0.],end:[20.,20.,0.],domain:[0.,1.]}};
        let second=Entity::new("second",Geometry::Curve(curve),None); ids.push(second.id); doc.entities.insert(second.id,second);
        let mut c=Controller::new(doc,Limits::default()).unwrap(); let before=c.document().clone(); c.submit("Pipe").unwrap(); c.submit("Multiple").unwrap();
        for id in &ids {c.solid_pipe_pick_curve(*id).unwrap();} c.submit("").unwrap(); c.submit("ShapeBlending=Global").unwrap(); if mixed {c.submit("Cap=Round").unwrap();} c.submit("1").unwrap(); c.submit("3").unwrap();
        c.accept_world_point([5.,0.,0.]).unwrap(); let preview=c.solid_preview([5.,4.,0.]); assert_eq!(preview.len(),64); assert!((preview[0][0][1]-4.).abs()<1e-9); c.submit("4").unwrap();
        if mixed {assert!(c.submit("").is_err()); assert_eq!(c.document(),&before); assert_eq!(c.coordinator().undo_len(),0); c.submit("Cancel").unwrap();}
        else {
            c.submit("").unwrap(); let mut volumes:Vec<_>=c.document().entities.values().filter_map(|e|if let Geometry::Brep(b)=&e.geometry {Some(om9_kernel::creation::measure(b,c.coordinator().limits()).unwrap().volume_mm3)} else {None}).collect();
            volumes.sort_by(f64::total_cmp); assert_eq!(volumes.len(),2); for (v,length) in volumes.iter().zip([10.,20.]) {assert!((*v-std::f64::consts::PI*length*29./3.).abs()<1e-5,"{volumes:?}");}
            assert_eq!(c.coordinator().undo_len(),1); c.undo().unwrap(); assert_eq!(c.document(),&before);
        }
    }
}

#[test]
fn pipe_multiple_stations_map_normalized_parameter_on_nurbs_and_line() {
    let curve=Curve::Nurbs(om9_core::geometry::NurbsCurve {degree:2,poles:vec![[0.,0.,0.],[5.,0.,0.],[10.,10.,0.]],weights:vec![1.;3],knots:vec![0.,0.,0.,1.,1.,1.],domain:[0.,1.],periodic:false,reversed:false,seam:None});
    let first=Entity::new("nurbs",Geometry::Curve(curve),None); let first_id=first.id;
    let second=Entity::new("line",Geometry::Curve(Curve::Line {start:[0.,-20.,0.],end:[10.,-20.,0.],domain:[0.,1.]}),None); let second_id=second.id;
    let mut doc=Document::new("multiple nonuniform station mapping"); doc.entities.insert(first_id,first); doc.entities.insert(second_id,second);
    let mut c=Controller::new(doc,Limits::default()).unwrap(); let before=c.document().clone(); c.submit("Pipe").unwrap(); c.submit("Multiple").unwrap();
    c.solid_pipe_pick_curve(first_id).unwrap(); c.solid_pipe_pick_curve(second_id).unwrap(); c.submit("").unwrap(); c.submit("ShapeBlending=Global").unwrap();
    c.submit("0.3").unwrap(); c.submit("0.7").unwrap(); c.accept_world_point([7.,4.9,0.]).unwrap(); c.submit("1.2").unwrap(); c.accept_world_point([2.5,0.625,0.]).unwrap(); c.submit("0.8").unwrap(); c.submit("").unwrap();
    let mut volumes:Vec<_>=c.document().entities.values().filter_map(|e|if let Geometry::Brep(b)=&e.geometry {Some(om9_kernel::creation::measure(b,c.coordinator().limits()).unwrap().volume_mm3)} else {None}).collect(); volumes.sort_by(f64::total_cmp); assert_eq!(volumes.len(),2);
    let length=|t:f64| 5.*t*(1.+4.*t*t).sqrt()+2.5*(2.*t).asinh();
    let intervals=[(0.,0.25,0.3,0.8),(0.25,0.7,0.8,1.2),(0.7,1.,1.2,0.7)];
    let expected_line=std::f64::consts::PI*10.*intervals.iter().map(|&(a,b,r,s)|(b-a)*(r*r+r*s+s*s)/3.).sum::<f64>();
    let expected_curve=std::f64::consts::PI*intervals.iter().map(|&(a,b,r,s)|(length(b)-length(a))*(r*r+r*s+s*s)/3.).sum::<f64>();
    for (v,expected) in volumes.iter().zip([expected_line,expected_curve]) {assert!((*v-expected).abs()<0.003*expected,"{volumes:?}, expected {expected}");}
    c.undo().unwrap(); assert_eq!(c.document(),&before);
}

#[test]
fn pipe_multiple_global_stations_apply_to_mixed_open_curves() {
    for curve_first in [false,true] {
      for cap in ["None","Flat","Round"] {
        let arc=Entity::new("arc",Geometry::Curve(om9_core::curve_build::arc_by_center([20.,0.,0.],[1.,0.,0.],[0.,1.,0.],5.,0.,std::f64::consts::FRAC_PI_2).unwrap()),None); let aid=arc.id;
        let line=Entity::new("line",Geometry::Curve(Curve::Line {start:[0.,0.,0.],end:[10.,0.,0.],domain:[0.,1.]}),None); let lid=line.id;
        let mut doc=Document::new("mixed station rails"); doc.entities.insert(aid,arc); doc.entities.insert(lid,line);
        let mut c=Controller::new(doc,Limits::default()).unwrap(); let before=c.document().clone(); c.submit("Pipe").unwrap(); c.submit("Multiple").unwrap();
        let ids=if curve_first {[aid,lid]} else {[lid,aid]}; for id in ids {c.solid_pipe_pick_curve(id).unwrap();} c.submit("").unwrap();
        c.submit("ShapeBlending=Global").unwrap(); c.submit(&format!("Cap={cap}")).unwrap(); c.submit("1").unwrap(); c.submit("3").unwrap();
        let q=5./2_f64.sqrt(); c.accept_world_point(if curve_first {[20.+q,q,0.]} else {[5.,0.,0.]}).unwrap(); c.submit("4").unwrap(); c.submit("").unwrap();
        let mut volumes:Vec<_>=c.document().entities.values().filter_map(|e|if let Geometry::Brep(b)=&e.geometry {Some(om9_kernel::creation::measure(b,c.coordinator().limits()).unwrap().volume_mm3)} else {None}).collect();
        volumes.sort_by(f64::total_cmp); assert_eq!(volumes.len(),2);
        if cap!="None" {for (v,length) in volumes.iter().zip([5.*std::f64::consts::FRAC_PI_2,10.]) {let expected=std::f64::consts::PI*(length*29./3.+if cap=="Round" {56./3.} else {0.}); assert!((*v-expected).abs()<0.03*expected,"{volumes:?}");}}
        else {for entity in c.document().entities.values() {if let Geometry::Brep(body)=&entity.geometry {let m=om9_kernel::creation::measure(body,c.coordinator().limits()).unwrap(); assert!(!m.closed && m.solids==0 && m.naked_edges>0,"{m:?}");}}}
        assert_eq!(c.coordinator().undo_len(),1); c.undo().unwrap(); assert_eq!(c.document(),&before);
      }
    }
}

#[test]
fn pipe_multiple_uses_curved_first_rail_for_section_points() {
    let circle=om9_core::curve_build::circle_on_frame([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],5.).unwrap();
    let first=Entity::new("circle",Geometry::Curve(circle),None); let first_id=first.id;
    let second=Entity::new("line",Geometry::Curve(Curve::Line {start:[20.,0.,0.],end:[30.,0.,0.],domain:[0.,1.]}),None); let second_id=second.id;
    let mut doc=Document::new("multiple curve first"); doc.entities.insert(first_id,first); doc.entities.insert(second_id,second);
    let mut c=Controller::new(doc,Limits::default()).unwrap(); let before=c.document().clone();
    c.submit("Pipe").unwrap(); c.submit("Multiple").unwrap(); c.solid_pipe_pick_curve(first_id).unwrap(); c.solid_pipe_pick_curve(second_id).unwrap(); c.submit("").unwrap();
    c.accept_world_point([6.,0.,0.]).unwrap(); c.accept_world_point([6.,0.,0.]).unwrap(); c.submit("").unwrap();
    let mut volumes:Vec<_>=c.document().entities.values().filter_map(|e|if let Geometry::Brep(body)=&e.geometry {Some(om9_kernel::creation::measure(body,c.coordinator().limits()).unwrap().volume_mm3)} else {None}).collect();
    volumes.sort_by(f64::total_cmp); assert_eq!(volumes.len(),2);
    assert!((volumes[0]-10.*std::f64::consts::PI).abs()<1e-6 && (volumes[1]-10.*std::f64::consts::PI.powi(2)).abs()<1e-6,"{volumes:?}");
    c.undo().unwrap(); assert_eq!(c.document(),&before);
}

#[test]
fn pipe_multiple_section_points_use_first_selected_rail() {
    for diameter in [false,true] {
        let mut doc=Document::new("multiple point radii"); let mut ids=Vec::new();
        for y in [20.,0.] {let rail=Entity::new("line",Geometry::Curve(Curve::Line {start:[0.,y,0.],end:[10.,y,0.],domain:[0.,1.]}),None); ids.push(rail.id); doc.entities.insert(rail.id,rail);}
        let mut c=Controller::new(doc,Limits::default()).unwrap(); let before=c.document().clone();
        c.submit("Pipe").unwrap(); c.submit("Multiple").unwrap(); c.solid_pipe_pick_curve(ids[0]).unwrap(); c.solid_pipe_pick_curve(ids[1]).unwrap(); c.submit("").unwrap();
        if diameter {c.submit("Diameter").unwrap();}
        let preview=c.solid_preview([0.,22.,0.]); assert_eq!(preview.len(),64); assert!((preview[0][0][1]-22.).abs()<1e-9);
        c.accept_world_point([0.,22.,0.]).unwrap(); c.accept_world_point([10.,22.,0.]).unwrap(); c.submit("").unwrap();
        let outputs:Vec<_>=c.document().entities.values().filter(|e|!ids.contains(&e.id)).collect(); assert_eq!(outputs.len(),2);
        for entity in outputs {let Geometry::Brep(body)=&entity.geometry else {panic!()}; let m=om9_kernel::creation::measure(body,c.coordinator().limits()).unwrap();
            assert!((m.volume_mm3-40.*std::f64::consts::PI).abs()<1e-6,"{m:?}");}
        assert_eq!(c.coordinator().undo_len(),1); c.undo().unwrap(); assert_eq!(c.document(),&before);
    }
}

#[test]
fn pipe_multiple_global_thick_round_variants_share_geometry() {
    for variant in [0, 1, 2] {
        let mut doc = Document::new("Multiple Pipe variants");
        for y in [0.0, 20.0] {
            let rail = Entity::new("line", Geometry::Curve(Curve::Line {
                start: [0.0, y, 0.0], end: [10.0, y, 0.0], domain: [0.0, 1.0],
            }), None);
            doc.entities.insert(rail.id, rail);
        }
        let ids: Vec<_> = doc.entities.keys().copied().collect();
        let mut c = Controller::new(doc, Limits::default()).unwrap();
        let before = c.document().clone();
        c.submit("Pipe").unwrap(); c.submit("Multiple").unwrap();
        for id in ids { c.solid_pipe_pick_curve(id).unwrap(); }
        c.submit("").unwrap(); c.submit("ShapeBlending=Global").unwrap();
        if variant == 1 { c.submit("Thick=Yes").unwrap(); }
        if variant == 2 { c.submit("Cap=Round").unwrap(); }
        if variant == 1 { for value in [1.0, 2.0, 2.0, 3.0] { c.submit(&value.to_string()).unwrap(); } }
        else { c.submit("1").unwrap(); c.submit("2").unwrap(); }
        c.submit("").unwrap();
        let expected = match variant {
            0 => 70.0 * std::f64::consts::PI / 3.0,
            1 => 120.0 * std::f64::consts::PI / 3.0,
            _ => (70.0 + 18.0) * std::f64::consts::PI / 3.0,
        };
        let results: Vec<_> = c.document().entities.values().filter_map(|e| match &e.geometry {
            Geometry::Brep(b) => Some(creation::measure(b, c.coordinator().limits()).unwrap().volume_mm3), _ => None,
        }).collect();
        assert_eq!(results.len(), 2);
        for volume in results { assert!((volume - expected).abs() < 1e-5, "{volume} {expected}"); }
        assert_eq!(c.coordinator().undo_len(), 1);
        c.undo().unwrap(); assert_eq!(c.document(), &before);
    }
}

#[test]
fn pipe_multiple_round_respects_output_budget_atomically() {
    let mut doc=Document::new("multiple pipe output budget"); let mut ids=Vec::new();
    for y in [0.,20.] {let rail=Entity::new("rail",Geometry::Curve(Curve::Line {start:[0.,y,0.],end:[10.,y,0.],domain:[0.,1.]}),None); ids.push(rail.id); doc.entities.insert(rail.id,rail);}
    let mut limits=Limits::default(); limits.max_outputs=1;
    let mut c=Controller::new(doc,limits).unwrap(); let before=c.document().clone(); c.submit("Pipe").unwrap(); c.submit("Multiple").unwrap();
    for id in ids {c.solid_pipe_pick_curve(id).unwrap();} c.submit("").unwrap(); c.submit("Cap=Round").unwrap(); c.submit("1").unwrap(); c.submit("1").unwrap();
    assert!(c.submit("").is_err()); assert_eq!(c.document(),&before); assert_eq!(c.coordinator().undo_len(),0); c.submit("Cancel").unwrap();
}

#[test]
fn pipe_multiple_failure_is_atomic_and_retryable() {
    let mut doc = Document::new("Multiple Pipe failure");
    let line = Entity::new("line", Geometry::Curve(Curve::Line {
        start: [0.0, 0.0, 0.0], end: [10.0, 0.0, 0.0], domain: [0.0, 1.0],
    }), None);
    let line_id = line.id;
    doc.entities.insert(line_id, line);
    let circle = Entity::new("circle", Geometry::Curve(om9_core::curve_build::circle_on_frame(
        [20.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 3.0).unwrap()), None);
    let circle_id = circle.id;
    doc.entities.insert(circle_id, circle);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    let before = c.document().clone();
    c.submit("Pipe").unwrap(); c.submit("Multiple").unwrap();
    c.solid_pipe_pick_curve(line_id).unwrap(); c.solid_pipe_pick_curve(circle_id).unwrap();
    c.submit("").unwrap(); c.submit("ShapeBlending=Global").unwrap();
    c.submit("1").unwrap(); c.submit("2").unwrap();
    assert!(c.submit("").is_err());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), 0);
    c.submit("Cancel").unwrap();
}

#[test]
fn pipe_cmd_sorts_multiple_intermediate_stations_and_undoes_last() {
    let mut doc = Document::new("Pipe multiple stations");
    let rail = Entity::new("rail", Geometry::Curve(Curve::Line {
        start: [0.0, 0.0, 0.0], end: [10.0, 0.0, 0.0], domain: [0.0, 1.0],
    }), None);
    let id = rail.id;
    doc.entities.insert(id, rail);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("Pipe").unwrap();
    c.submit("ShapeBlending=Global").unwrap();
    c.submit("1").unwrap(); c.submit("3").unwrap();
    c.submit("7.5,0,0").unwrap(); c.submit("2").unwrap();
    c.submit("2.5,0,0").unwrap(); c.submit("4").unwrap();
    c.submit("Undo").unwrap();
    c.submit("2.5,0,0").unwrap(); c.submit("4").unwrap();
    c.submit("").unwrap();
    let result = c.document().entities.values().find(|e| e.id != id).unwrap();
    let Geometry::Brep(brep) = &result.geometry else { panic!() };
    let m = om9_kernel::creation::measure(brep, c.coordinator().limits()).unwrap();
    assert!((m.volume_mm3 - 240.0 * std::f64::consts::PI / 3.0).abs() < 1e-5, "{m:?}");
}

#[test]
fn pipe_cmd_intermediate_station_retries_and_undoes_selection() {
    let mut doc = Document::new("Pipe stations");
    let rail = Entity::new("rail", Geometry::Curve(Curve::Line {
        start: [0.0, 0.0, 0.0], end: [10.0, 0.0, 0.0], domain: [0.0, 1.0],
    }), None);
    let id = rail.id;
    doc.entities.insert(id, rail);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    let before = c.document().clone();
    c.select_objects(&[id]).unwrap();
    c.submit("Pipe").unwrap();
    c.submit("ShapeBlending=Global").unwrap();
    c.submit("1").unwrap();
    c.submit("3").unwrap();
    assert!(c.submit("0,0,0").is_err());
    c.submit("5,0,0").unwrap();
    assert!(c.submit("0").is_err());
    c.submit("Undo").unwrap();
    c.submit("5,0,0").unwrap();
    c.submit("2").unwrap();
    assert!(c.submit("5,0,0").is_err());
    assert_eq!(c.coordinator().undo_len(), 0);
    c.submit("").unwrap();
    let result = c.document().entities.values().find(|e| e.id != id).unwrap();
    let Geometry::Brep(brep) = &result.geometry else { panic!() };
    let m = om9_kernel::creation::measure(brep, c.coordinator().limits()).unwrap();
    assert!((m.volume_mm3 - 130.0 * std::f64::consts::PI / 3.0).abs() < 1e-5, "{m:?}");
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
}

#[test]
fn pipe_global_multiple_nurbs_stations_follow_curve_length() {
    let curve=Curve::Nurbs(om9_core::geometry::NurbsCurve {degree:2,poles:vec![[0.,0.,0.],[5.,0.,0.],[10.,10.,0.]],weights:vec![1.;3],knots:vec![0.,0.,0.,1.,1.,1.],domain:[0.,1.],periodic:false,reversed:false,seam:None});
    let rail=Entity::new("curve",Geometry::Curve(curve),None); let id=rail.id; let mut doc=Document::new("nonuniform length stations"); doc.entities.insert(id,rail);
    let mut c=Controller::new(doc,Limits::default()).unwrap(); let before=c.document().clone(); c.select_objects(&[id]).unwrap();
    c.submit("Pipe").unwrap(); c.submit("ShapeBlending=Global").unwrap(); c.submit("0.3").unwrap(); c.submit("0.7").unwrap();
    c.accept_world_point([7.,4.9,0.]).unwrap(); c.submit("1.2").unwrap(); c.accept_world_point([2.5,0.625,0.]).unwrap(); c.submit("0.8").unwrap(); c.submit("").unwrap();
    let entity=c.document().entities.values().find(|e|e.id!=id).unwrap(); let Geometry::Brep(body)=&entity.geometry else {panic!()};
    let m=om9_kernel::creation::measure(body,c.coordinator().limits()).unwrap();
    let length=|t:f64| 5.*t*(1.+4.*t*t).sqrt()+2.5*(2.*t).asinh();
    let intervals=[(0.,0.25,0.3,0.8),(0.25,0.7,0.8,1.2),(0.7,1.,1.2,0.7)];
    let expected=std::f64::consts::PI*intervals.iter().map(|&(a,b,r,s)|(length(b)-length(a))*(r*r+r*s+s*s)/3.).sum::<f64>();
    let parameter_volume=std::f64::consts::PI*(0..10000).map(|i|{let t=(i as f64+0.5)/10000.;
        let &(a,b,r,s)=intervals.iter().find(|&&(a,b,_,_)|t>=a && t<=b).unwrap();
        let radius=r+(s-r)*(t-a)/(b-a); radius*radius*10.*(1.+4.*t*t).sqrt()/10000.
    }).sum::<f64>();
    assert!((parameter_volume-expected).abs()>0.005*expected);
    assert!(m.closed && (m.volume_mm3-expected).abs()<0.003*expected,"{m:?}, expected {expected}");
    c.undo().unwrap(); assert_eq!(c.document(),&before);
}

#[test]
fn pipe_curve_station_rejects_closed_or_nonuniform_placement_atomically() {
    for nonuniform in [false,true] {
        let curve=if nonuniform {om9_core::curve_build::arc_by_center([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],5.,0.,std::f64::consts::FRAC_PI_2).unwrap()}
            else {om9_core::curve_build::circle_on_frame([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],5.).unwrap()};
        let mut rail=Entity::new("curve",Geometry::Curve(curve),None);
        if nonuniform {rail.placement=Transform::scale([2.,1.,1.]).unwrap();}
        let id=rail.id; let mut doc=Document::new("unsupported stations"); doc.entities.insert(id,rail);
        let mut c=Controller::new(doc,Limits::default()).unwrap(); let before=c.document().clone(); c.select_objects(&[id]).unwrap();
        c.submit("Pipe").unwrap(); c.submit("ShapeBlending=Global").unwrap(); c.submit("1").unwrap(); c.submit("2").unwrap();
        if nonuniform {assert!(c.accept_world_point([7.071,3.535,0.]).is_err());}
        else {c.accept_world_point([0.,5.,0.]).unwrap(); c.submit("1.5").unwrap(); assert!(c.submit("").is_err());}
        assert_eq!(c.document(),&before); assert_eq!(c.coordinator().undo_len(),0); c.submit("Cancel").unwrap();
    }
}

#[test]
fn pipe_global_multiple_curve_stations_sorted_and_undoable() {
    for capped in [false,true] {
        let arc=om9_core::curve_build::arc_by_center([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],5.,0.,std::f64::consts::FRAC_PI_2).unwrap();
        let mut rail=Entity::new("arc",Geometry::Curve(arc),None); rail.placement=Transform::translation([10.,20.,3.]);
        let id=rail.id; let mut doc=Document::new("multiple arc stations"); doc.entities.insert(id,rail);
        let mut c=Controller::new(doc,Limits::default()).unwrap(); let before=c.document().clone(); c.select_objects(&[id]).unwrap();
        c.submit("Pipe").unwrap(); c.submit("ShapeBlending=Global").unwrap(); if !capped {c.submit("Cap=None").unwrap();}
        c.submit("1").unwrap(); c.submit("3").unwrap();
        let point=|f:f64| {let t=f*std::f64::consts::FRAC_PI_2; [10.+5.*t.cos(),20.+5.*t.sin(),3.]};
        c.accept_world_point(point(0.75)).unwrap(); c.submit("2").unwrap();
        c.accept_world_point(point(0.25)).unwrap(); c.submit("4").unwrap(); c.submit("Undo").unwrap();
        c.accept_world_point(point(0.25)).unwrap(); c.submit("4").unwrap(); c.submit("").unwrap();
        let entity=c.document().entities.values().find(|e|e.id!=id).unwrap(); let Geometry::Brep(body)=&entity.geometry else {panic!()};
        let m=om9_kernel::creation::measure(body,c.coordinator().limits()).unwrap();
        let expected=std::f64::consts::PI*5.*std::f64::consts::FRAC_PI_2*8.;
        if capped {assert!(m.closed && (m.volume_mm3-expected).abs()<0.03*expected,"{m:?}, expected {expected}");}
        else {assert!(!m.closed && m.solids==0 && m.naked_edges>0,"{m:?}");}
        c.undo().unwrap(); assert_eq!(c.document(),&before);
    }
}

#[test]
fn pipe_global_intermediate_station_on_quadratic_nurbs() {
    let curve=Curve::Nurbs(om9_core::geometry::NurbsCurve {degree:2,poles:vec![[0.,0.,0.],[5.,0.,0.],[5.,5.,0.]],weights:vec![1.;3],knots:vec![0.,0.,0.,1.,1.,1.],domain:[0.,1.],periodic:false,reversed:false,seam:None});
    let rail=Entity::new("curve",Geometry::Curve(curve),None); let id=rail.id; let mut doc=Document::new("NURBS station"); doc.entities.insert(id,rail);
    let mut c=Controller::new(doc,Limits::default()).unwrap(); let before=c.document().clone(); c.select_objects(&[id]).unwrap();
    c.submit("Pipe").unwrap(); c.submit("ShapeBlending=Global").unwrap(); c.submit("0.5").unwrap(); c.submit("1.5").unwrap();
    c.accept_world_point([3.75,1.25,0.]).unwrap(); c.submit("2").unwrap(); c.submit("").unwrap();
    let entity=c.document().entities.values().find(|e|e.id!=id).unwrap(); let Geometry::Brep(body)=&entity.geometry else {panic!()};
    let m=om9_kernel::creation::measure(body,c.coordinator().limits()).unwrap(); let length=5.*(1.+(1.+2_f64.sqrt()).ln()/2_f64.sqrt());
    let expected=std::f64::consts::PI*length*29./12.;
    assert!(m.closed && (m.volume_mm3-expected).abs()<0.03*expected,"{m:?}, expected {expected}"); c.undo().unwrap(); assert_eq!(c.document(),&before);
}

#[test]
fn pipe_global_intermediate_station_on_open_arc() {
    let arc=om9_core::curve_build::arc_by_center([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],5.,0.,std::f64::consts::FRAC_PI_2).unwrap();
    let rail=Entity::new("arc",Geometry::Curve(arc),None); let id=rail.id; let mut doc=Document::new("arc stations"); doc.entities.insert(id,rail);
    let mut c=Controller::new(doc,Limits::default()).unwrap(); let before=c.document().clone(); c.select_objects(&[id]).unwrap();
    c.submit("Pipe").unwrap(); c.submit("ShapeBlending=Global").unwrap(); c.submit("1").unwrap(); c.submit("3").unwrap();
    let q=5./2_f64.sqrt(); c.accept_world_point([q,q,0.]).unwrap(); let preview=c.solid_preview([q,q,4.]); assert_eq!(preview.len(),64); assert!((preview[0][0][2]-4.).abs()<1e-8); assert_eq!(c.document(),&before); c.submit("4").unwrap();
    c.submit("Undo").unwrap(); c.accept_world_point([q,q,0.]).unwrap(); c.accept_world_point([q,q,4.]).unwrap();
    assert!(c.accept_world_point([q,q,0.]).is_err()); c.submit("").unwrap();
    let entity=c.document().entities.values().find(|e|e.id!=id).unwrap(); let Geometry::Brep(body)=&entity.geometry else {panic!()};
    let m=om9_kernel::creation::measure(body,c.coordinator().limits()).unwrap();
    let expected=std::f64::consts::PI*5.*std::f64::consts::FRAC_PI_2*29./3.;
    assert!(m.closed && (m.volume_mm3-expected).abs()<0.03*expected,"{m:?}, expected {expected}");
    c.undo().unwrap(); assert_eq!(c.document(),&before);
}

#[test]
fn pipe_global_on_quadratic_nurbs_with_varying_radii() {
    let curve=Curve::Nurbs(om9_core::geometry::NurbsCurve {degree:2,poles:vec![[0.,0.,0.],[5.,0.,0.],[5.,5.,0.]],weights:vec![1.;3],knots:vec![0.,0.,0.,1.,1.,1.],domain:[0.,1.],periodic:false,reversed:false,seam:None});
    for thick in [false,true] {
      for capped in [false,true] {
        let mut rail=Entity::new("curve",Geometry::Curve(curve.clone()),None); rail.placement=Transform::translation([10.,20.,3.]);
        let id=rail.id; let mut doc=Document::new("varying NURBS Pipe"); doc.entities.insert(id,rail);
        let mut c=Controller::new(doc,Limits::default()).unwrap(); let before=c.document().clone(); c.select_objects(&[id]).unwrap();
        c.submit("Pipe").unwrap(); c.submit("ShapeBlending=Global").unwrap();
        if !capped {c.submit("Cap=None").unwrap();}
        if thick {c.submit("Thick=Yes").unwrap(); for v in ["0.5","1","0.75","1.5"] {c.submit(v).unwrap();}}
        else {c.submit("0.5").unwrap(); c.submit("1.5").unwrap();}
        c.submit("").unwrap(); let entity=c.document().entities.values().find(|e|e.id!=id).unwrap();
        let Geometry::Brep(body)=&entity.geometry else {panic!()}; let m=om9_kernel::creation::measure(body,c.coordinator().limits()).unwrap();
        let length=5.*(1.+(1.+2_f64.sqrt()).ln()/2_f64.sqrt());
        let radii=if thick {(1.+1.5+2.25)-(0.25+0.375+0.5625)} else {0.25+0.75+2.25};
        let expected=std::f64::consts::PI*length*radii/3.;
        if capped {assert!(m.closed && (m.volume_mm3-expected).abs()<0.03*expected,"{m:?}, expected {expected}");}
        else {assert!(!m.closed && m.solids==0 && m.naked_edges>0,"{m:?}");}
        c.undo().unwrap(); assert_eq!(c.document(),&before);
      }
    }
}

#[test]
fn pipe_thick_global_on_open_arc() {
  for capped in [false,true] {
    let arc=om9_core::curve_build::arc_by_center([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],5.,0.,std::f64::consts::FRAC_PI_2).unwrap();
    let mut rail=Entity::new("arc",Geometry::Curve(arc),None); rail.placement=Transform::translation([10.,20.,3.]);
    let id=rail.id; let mut doc=Document::new("thick arc Pipe"); doc.entities.insert(id,rail);
    let mut c=Controller::new(doc,Limits::default()).unwrap(); let before=c.document().clone(); c.select_objects(&[id]).unwrap();
    c.submit("Pipe").unwrap(); c.submit("Thick=Yes").unwrap(); c.submit("ShapeBlending=Global").unwrap();
    if !capped {c.submit("Cap=None").unwrap();}
    for value in ["0.5","1","0.75","1.5"] {c.submit(value).unwrap();}
    c.submit("").unwrap(); let entity=c.document().entities.values().find(|e|e.id!=id).unwrap();
    let Geometry::Brep(body)=&entity.geometry else {panic!()}; let m=om9_kernel::creation::measure(body,c.coordinator().limits()).unwrap();
    let expected=std::f64::consts::PI*5.*std::f64::consts::FRAC_PI_2*((1.+1.5+2.25)-(0.25+0.375+0.5625))/3.;
    if capped {assert!(m.closed && (m.volume_mm3-expected).abs()<0.03*expected,"{m:?}, expected {expected}");}
    else {assert!(!m.closed && m.naked_edges>0 && m.solids==0,"{m:?}");}
    c.undo().unwrap(); assert_eq!(c.document(),&before);
  }
}

#[test]
fn pipe_global_varying_radius_rejects_closed_curve_without_commit() {
    let circle=om9_core::curve_build::circle_on_frame([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],5.).unwrap();
    let rail=Entity::new("circle",Geometry::Curve(circle),None); let id=rail.id; let mut doc=Document::new("closed varying Pipe"); doc.entities.insert(id,rail);
    let mut c=Controller::new(doc,Limits::default()).unwrap(); let before=c.document().clone(); c.select_objects(&[id]).unwrap();
    c.submit("Pipe").unwrap(); c.submit("ShapeBlending=Global").unwrap(); c.submit("1").unwrap(); c.submit("2").unwrap();
    assert!(c.submit("").is_err()); assert_eq!(c.document(),&before); assert_eq!(c.coordinator().undo_len(),0);
    c.submit("Undo").unwrap(); c.submit("1").unwrap(); c.submit("").unwrap(); assert_eq!(c.coordinator().undo_len(),1);
    c.undo().unwrap(); assert_eq!(c.document(),&before);
}

#[test]
fn pipe_global_varying_radius_on_open_arc() {
    for via_menu in [false,true] {
        let arc=om9_core::curve_build::arc_by_center([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],5.,0.,std::f64::consts::FRAC_PI_2).unwrap();
        let mut rail=Entity::new("arc",Geometry::Curve(arc),None); rail.placement=Transform::translation([10.,20.,3.]);
        let id=rail.id; let mut doc=Document::new("varying arc Pipe"); doc.entities.insert(id,rail);
        let mut c=Controller::new(doc,Limits::default()).unwrap(); let before=c.document().clone(); c.select_objects(&[id]).unwrap();
        if via_menu {c.dispatch("OM9-SOLID-011",InvocationOrigin::Menu).unwrap();} else {c.submit("Pipe").unwrap();}
        c.submit("ShapeBlending=Global").unwrap(); c.accept_world_point([15.,24.,3.5]).unwrap(); c.accept_world_point([6.,25.,4.]).unwrap();
        c.submit("").unwrap(); let entity=c.document().entities.values().find(|e|e.id!=id).unwrap();
        let Geometry::Brep(body)=&entity.geometry else {panic!()}; let m=om9_kernel::creation::measure(body,c.coordinator().limits()).unwrap();
        let expected=std::f64::consts::PI*5.*std::f64::consts::FRAC_PI_2*(0.25+0.5+1.)/3.;
        assert!(m.closed && (m.volume_mm3-expected).abs()<0.03*expected,"{m:?}, expected {expected}");
        c.undo().unwrap(); assert_eq!(c.document(),&before);
    }
}

#[test]
fn pipe_arc_and_nurbs_endpoint_points_produce_constant_radius_solids() {
    let arc=om9_core::curve_build::arc_by_center([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],5.,0.,std::f64::consts::FRAC_PI_2).unwrap();
    let nurbs=Curve::Nurbs(om9_core::geometry::NurbsCurve {degree:2,poles:vec![[0.,0.,0.],[5.,0.,0.],[5.,5.,0.]],weights:vec![1.;3],knots:vec![0.,0.,0.,1.,1.,1.],domain:[0.,1.],periodic:false,reversed:false,seam:None});
    for (curve,start,end,length) in [(arc,[5.,4.,0.],[-4.,5.,0.],5.*std::f64::consts::FRAC_PI_2),
        (nurbs,[4.,0.,0.],[5.,9.,0.],5.*(1.+(1.+2_f64.sqrt()).ln()/2_f64.sqrt()))] {
        let mut rail=Entity::new("curve",Geometry::Curve(curve),None); rail.placement=Transform::translation([10.,20.,3.]);
        let id=rail.id; let mut doc=Document::new("curved Pipe points"); doc.entities.insert(id,rail);
        let mut c=Controller::new(doc,Limits::default()).unwrap(); let before=c.document().clone(); c.select_objects(&[id]).unwrap();
        c.submit("Pipe").unwrap();
        let point=|p:[f64;3]| [p[0]+10.,p[1]+20.,p[2]+4.];
        c.accept_world_point(point(start)).unwrap(); c.accept_world_point(point(end)).unwrap(); c.submit("").unwrap();
        let entity=c.document().entities.values().find(|e|e.id!=id).unwrap(); let Geometry::Brep(body)=&entity.geometry else {panic!()};
        let m=om9_kernel::creation::measure(body,c.coordinator().limits()).unwrap();
        let expected=std::f64::consts::PI*length;
        assert!(m.closed && (m.volume_mm3-expected).abs()<0.03*expected,"{m:?}, expected {expected}");
        c.undo().unwrap(); assert_eq!(c.document(),&before);
    }
}

#[test]
fn pipe_circle_endpoint_section_points_use_world_tangent() {
    for diameter in [false,true] {
      for vertical in [false,true] {
        let y=if vertical {[0.,0.,1.]} else {[0.,1.,0.]};
        let circle=om9_core::curve_build::circle_on_frame([0.,0.,0.],[1.,0.,0.],y,10.).unwrap();
        let mut rail=Entity::new("circle",Geometry::Curve(circle),None); rail.placement=Transform::translation([5.,3.,4.]);
        let id=rail.id; let mut doc=Document::new("circle point sections"); doc.entities.insert(id,rail);
        let mut c=Controller::new(doc,Limits::default()).unwrap(); let before=c.document().clone();
        c.select_objects(&[id]).unwrap(); c.submit("Pipe").unwrap(); if diameter {c.submit("Diameter").unwrap();}
        let (tangent,radial)=if vertical {("15,3,6","15,5,4")} else {("15,5,4","15,3,6")};
        let radial_point=if vertical {[15.,5.,4.]} else {[15.,3.,6.]}; let preview=c.solid_preview(radial_point);
        assert_eq!(preview.len(),64); for axis in 0..3 {assert!((preview[0][0][axis]-radial_point[axis]).abs()<1e-8);}
        assert!(c.submit(tangent).is_err());
        c.submit(radial).unwrap(); c.submit(radial).unwrap(); c.submit("").unwrap();
        let entity=c.document().entities.values().find(|e|e.id!=id).unwrap(); let Geometry::Brep(body)=&entity.geometry else {panic!()};
        let m=om9_kernel::creation::measure(body,c.coordinator().limits()).unwrap();
        assert!((m.volume_mm3-80.*std::f64::consts::PI.powi(2)).abs()<1e-5,"{m:?}");
        c.undo().unwrap(); assert_eq!(c.document(),&before);
      }
    }
}

#[test]
fn pipe_diameter_mode_accepts_point_radii_for_thick_and_station() {
    for thick in [false, true] {
        let mut doc=Document::new("Pipe diameter point");
        let rail=Entity::new("rail",Geometry::Curve(Curve::Line {start:[0.,0.,0.],end:[10.,0.,0.],domain:[0.,1.]}),None);
        let id=rail.id; doc.entities.insert(id,rail); let mut c=Controller::new(doc,Limits::default()).unwrap();
        let before=c.document().clone(); c.select_objects(&[id]).unwrap(); c.submit("Pipe").unwrap();
        c.submit("Diameter").unwrap(); c.submit("ShapeBlending=Global").unwrap();
        if thick {
            c.submit("Thick=Yes").unwrap();
            for point in ["0,1,0", "0,2,0", "10,1,0", "10,2,0"] { c.submit(point).unwrap(); }
        } else {
            c.submit("0,1,0").unwrap(); c.submit("10,3,0").unwrap();
            c.submit("5,0,0").unwrap(); c.submit("5,2,0").unwrap();
        }
        c.submit("").unwrap();
        let result=c.document().entities.values().find(|e|e.id!=id).unwrap();
        let Geometry::Brep(body)=&result.geometry else {panic!()};
        let m=om9_kernel::creation::measure(body,c.coordinator().limits()).unwrap();
        let expected=if thick {30.*std::f64::consts::PI} else {130.*std::f64::consts::PI/3.};
        assert!((m.volume_mm3-expected).abs()<1e-5, "{m:?}");
        c.undo().unwrap(); assert_eq!(c.document(),&before);
    }
}

#[test]
fn pipe_section_preview_matches_picked_radii_and_keeps_document_unchanged() {
    for diameter in [false,true] {
        let mut rail=Entity::new("rail",Geometry::Curve(Curve::Line {start:[0.,0.,0.],end:[10.,0.,0.],domain:[0.,1.]}),None); rail.placement=Transform::translation([5.,3.,4.]);
        let id=rail.id; let mut doc=Document::new("pipe section preview"); doc.entities.insert(id,rail); let mut c=Controller::new(doc,Limits::default()).unwrap();
        let before=c.document().clone(); c.select_objects(&[id]).unwrap(); c.submit("Pipe").unwrap(); if diameter {c.submit("Diameter").unwrap();}
        assert!(c.solid_preview([7.,3.,4.]).is_empty());
        let first=c.solid_preview([7.,5.,4.]); assert_eq!(first.len(),64); assert!((first[0][0][0]-5.).abs()<1e-9 && (first[0][0][1]-5.).abs()<1e-9);
        assert_eq!(c.document(),&before); assert_eq!(c.coordinator().undo_len(),0); c.accept_world_point([7.,5.,4.]).unwrap();
        let last=c.solid_preview([13.,1.,4.]); assert_eq!(last.len(),176); assert!((last[0][0][0]-15.).abs()<1e-9 && (last[0][0][1]-1.).abs()<1e-9);
        c.accept_world_point([13.,1.,4.]).unwrap(); c.submit("").unwrap(); let entity=c.document().entities.values().find(|e|e.id!=id).unwrap();
        let Geometry::Brep(body)=&entity.geometry else {panic!()}; let m=om9_kernel::creation::measure(body,c.coordinator().limits()).unwrap();
        assert!((m.volume_mm3-40.*std::f64::consts::PI).abs()<1e-6); c.undo().unwrap(); assert_eq!(c.document(),&before);
    }
}

#[test]
fn pipe_straight_thick_body_preview_tracks_both_walls() {
    for round in [false,true] {for global in [false,true] {
        let rail=Entity::new("rail",Geometry::Curve(Curve::Line {start:[0.,0.,0.],end:[10.,0.,0.],domain:[0.,1.]}),None); let id=rail.id;
        let mut doc=Document::new("thick pipe preview"); doc.entities.insert(id,rail); let mut c=Controller::new(doc,Limits::default()).unwrap(); let before=c.document().clone();
        c.select_objects(&[id]).unwrap(); c.submit("Pipe").unwrap(); c.submit("Thick=Yes").unwrap(); if round {c.submit("Cap=Round").unwrap();} if global {c.submit("ShapeBlending=Global").unwrap();}
        for v in ["2","1","1.5"] {c.submit(v).unwrap();} let preview=c.solid_preview([10.,3.,0.]); let count=if round {240} else {176}; assert_eq!(preview.len(),count*2);
        assert!((preview[0][0][1]-1.5).abs()<1e-9 && (preview[count][0][1]-3.).abs()<1e-9);
        let blend=if global {0.25} else {0.15625}; assert!((preview[64][0][1]-(1.+0.5*blend)).abs()<1e-9 && (preview[count+64][0][1]-(2.+blend)).abs()<1e-9);
        if round {assert!((preview[183][1][0]+1.).abs()<1e-9 && (preview[count+183][1][0]+2.).abs()<1e-9);}
        assert_eq!(c.document(),&before); assert_eq!(c.coordinator().undo_len(),0); c.accept_world_point([10.,3.,0.]).unwrap(); assert_eq!(c.solid_preview([10.,3.,0.]),preview); c.submit("Cancel").unwrap(); assert_eq!(c.document(),&before);
    }}
}

#[test]
fn pipe_straight_round_body_preview_has_endpoint_poles() {
    for global in [false,true] {
        let rail=Entity::new("rail",Geometry::Curve(Curve::Line {start:[0.,0.,0.],end:[10.,0.,0.],domain:[0.,1.]}),None); let id=rail.id;
        let mut doc=Document::new("round pipe preview"); doc.entities.insert(id,rail); let mut c=Controller::new(doc,Limits::default()).unwrap(); let before=c.document().clone();
        c.select_objects(&[id]).unwrap(); c.submit("Pipe").unwrap(); c.submit("Cap=Round").unwrap(); if global {c.submit("ShapeBlending=Global").unwrap();} c.submit("1").unwrap();
        let preview=c.solid_preview([10.,2.,0.]); assert_eq!(preview.len(),240);
        assert!((preview[183][1][0]+1.).abs()<1e-9 && (preview[215][1][0]-12.).abs()<1e-9);
        assert_eq!(c.document(),&before); assert_eq!(c.coordinator().undo_len(),0); c.accept_world_point([10.,2.,0.]).unwrap(); assert_eq!(c.solid_preview([10.,2.,0.]),preview);
        c.submit("").unwrap(); let entity=c.document().entities.values().find(|e|e.id!=id).unwrap(); let Geometry::Brep(body)=&entity.geometry else {panic!()};
        let m=om9_kernel::creation::measure(body,c.coordinator().limits()).unwrap(); let expected=std::f64::consts::PI*(10.*if global {7./3.} else {83./35.}+6.);
        assert!(m.closed && (m.volume_mm3-expected).abs()<0.02,"{m:?}, expected {expected}"); c.undo().unwrap(); assert_eq!(c.document(),&before);
    }
}

#[test]
fn pipe_straight_body_preview_tracks_blending_and_confirmation() {
    for global in [false,true] {
        let rail=Entity::new("rail",Geometry::Curve(Curve::Line {start:[0.,0.,0.],end:[10.,0.,0.],domain:[0.,1.]}),None); let id=rail.id;
        let mut doc=Document::new("pipe body preview"); doc.entities.insert(id,rail); let mut c=Controller::new(doc,Limits::default()).unwrap(); let before=c.document().clone();
        c.select_objects(&[id]).unwrap(); c.submit("Pipe").unwrap(); if global {c.submit("ShapeBlending=Global").unwrap();} c.submit("2").unwrap();
        let preview=c.solid_preview([10.,4.,0.]); assert_eq!(preview.len(),176);
        let expected=2.+2.*if global {0.25} else {0.15625}; assert!((preview[64][0][0]-2.5).abs()<1e-9 && (preview[64][0][1]-expected).abs()<1e-9);
        assert_eq!(c.document(),&before); assert_eq!(c.coordinator().undo_len(),0); c.accept_world_point([10.,4.,0.]).unwrap(); assert_eq!(c.solid_preview([10.,4.,0.]),preview);
        c.submit("Cancel").unwrap(); assert_eq!(c.document(),&before); assert_eq!(c.coordinator().undo_len(),0);
    }
}

#[test]
fn pipe_section_preview_respects_point_budget() {
    let rail=Entity::new("rail",Geometry::Curve(Curve::Line {start:[0.,0.,0.],end:[10.,0.,0.],domain:[0.,1.]}),None); let id=rail.id;
    let mut doc=Document::new("preview budget"); doc.entities.insert(id,rail); let mut limits=Limits::default(); limits.max_points=100;
    let mut c=Controller::new(doc,limits).unwrap(); let before=c.document().clone(); c.select_objects(&[id]).unwrap(); c.submit("Pipe").unwrap();
    assert!(c.solid_preview([0.,2.,0.]).is_empty()); assert_eq!(c.document(),&before); assert_eq!(c.coordinator().undo_len(),0);
}

#[test]
fn pipe_cmd_world_radius_points_project_onto_end_sections() {
  for diameter in [false, true] {
    let mut doc = Document::new("Pipe point input");
    let mut rail = Entity::new("rail", Geometry::Curve(Curve::Line {
        start: [0.0, 0.0, 0.0], end: [10.0, 0.0, 0.0], domain: [0.0, 1.0],
    }), None);
    rail.placement = Transform::translation([5.0, 3.0, 0.0]);
    let id = rail.id;
    doc.entities.insert(id, rail);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    let before = c.document().clone();
    c.select_objects(&[id]).unwrap();
    c.submit("Pipe").unwrap();
    if diameter {c.submit("Diameter").unwrap();}
    assert!(c.submit("5,3,0").is_err());
    assert!(c.prompt().contains(if diameter {"Start diameter"} else {"Start radius"}));
    c.submit("7,5,0").unwrap();
    c.submit("13,1,0").unwrap();
    c.submit("").unwrap();
    let result = c.document().entities.values().find(|e| e.id != id).unwrap();
    let Geometry::Brep(brep) = &result.geometry else { panic!() };
    let m = om9_kernel::creation::measure(brep, c.coordinator().limits()).unwrap();
    assert!((m.volume_mm3 - 40.0 * std::f64::consts::PI).abs() < 1e-6, "{m:?}");
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
  }
}

#[test]
fn pipe_round_intermediate_station_on_line_and_arc() {
    for global in [false,true] {for curved in [false,true] {
        let curve=if curved {om9_core::curve_build::arc_by_center([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],20.,0.,std::f64::consts::FRAC_PI_2).unwrap()}
            else {Curve::Line {start:[0.,0.,0.],end:[10.,0.,0.],domain:[0.,1.]}};
        let rail=Entity::new("rail",Geometry::Curve(curve),None); let id=rail.id; let mut doc=Document::new("round station Pipe"); doc.entities.insert(id,rail);
        let mut c=Controller::new(doc,Limits::default()).unwrap(); let before=c.document().clone(); c.select_objects(&[id]).unwrap(); c.submit("Pipe").unwrap();
        c.submit("Cap=Round").unwrap(); if global {c.submit("ShapeBlending=Global").unwrap();} c.submit("1").unwrap(); c.submit("3").unwrap();
        let q=20./2_f64.sqrt(); c.accept_world_point(if curved {[q,q,0.]} else {[5.,0.,0.]}).unwrap(); c.submit("4").unwrap(); c.submit("").unwrap();
        let entity=c.document().entities.values().find(|e|e.id!=id).unwrap(); let Geometry::Brep(body)=&entity.geometry else {panic!()};
        let m=om9_kernel::creation::measure(body,c.coordinator().limits()).unwrap(); let length=if curved {20.*std::f64::consts::FRAC_PI_2} else {10.};
        let expected=std::f64::consts::PI*(length*if global {29./3.} else {69./7.}+56./3.);
        assert!(m.closed && (m.volume_mm3-expected).abs()<0.03*expected,"{m:?}, expected {expected}"); c.undo().unwrap(); assert_eq!(c.document(),&before);
    }}
}

#[test]
fn pipe_local_round_variable_radii_on_open_arc() {
    for global in [false,true] {for thick in [false,true] {
        let curve=om9_core::curve_build::arc_by_center([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],20.,0.,std::f64::consts::FRAC_PI_2).unwrap();
        let mut rail=Entity::new("arc",Geometry::Curve(curve),None); rail.placement=Transform::translation([7.,9.,3.]); let id=rail.id;
        let mut doc=Document::new("local round curved pipe"); doc.entities.insert(id,rail); let mut c=Controller::new(doc,Limits::default()).unwrap(); let before=c.document().clone(); c.select_objects(&[id]).unwrap();
        c.submit("Pipe").unwrap(); c.submit("Cap=Round").unwrap(); if global {c.submit("ShapeBlending=Global").unwrap();} if thick {c.submit("Thick=Yes").unwrap();}
        if thick {for v in ["1","2","1.5","3"] {c.submit(v).unwrap();}} else {c.submit("1").unwrap(); c.submit("2").unwrap();} c.submit("").unwrap();
        let entity=c.document().entities.values().find(|e|e.id!=id).unwrap(); let Geometry::Brep(body)=&entity.geometry else {panic!()};
        let m=om9_kernel::creation::measure(body,c.coordinator().limits()).unwrap(); let length=20.*std::f64::consts::FRAC_PI_2;
        let factor=if thick {if global {4.75} else {669./140.}} else if global {7./3.} else {83./35.};
        let expected=std::f64::consts::PI*(length*factor+if thick {2.*30.625/3.} else {6.});
        assert!(m.closed && (m.volume_mm3-expected).abs()<0.03*expected,"{m:?}, expected {expected}"); c.undo().unwrap(); assert_eq!(c.document(),&before);
    }}
}

#[test]
fn pipe_local_round_variable_radii_on_straight_rail() {
  for thick in [false,true] {
    let rail=Entity::new("rail",Geometry::Curve(Curve::Line {start:[0.,0.,0.],end:[10.,0.,0.],domain:[0.,1.]}),None); let id=rail.id;
    let mut doc=Document::new("local round pipe"); doc.entities.insert(id,rail); let mut c=Controller::new(doc,Limits::default()).unwrap(); let before=c.document().clone(); c.select_objects(&[id]).unwrap();
    c.submit("Pipe").unwrap(); c.submit("Cap=Round").unwrap(); if thick {c.submit("Thick=Yes").unwrap();}
    if thick {for v in ["1","2","1.5","3"] {c.submit(v).unwrap();}} else {c.submit("1").unwrap(); c.submit("2").unwrap();} c.submit("").unwrap();
    let entity=c.document().entities.values().find(|e|e.id!=id).unwrap(); let Geometry::Brep(body)=&entity.geometry else {panic!()};
    let m=om9_kernel::creation::measure(body,c.coordinator().limits()).unwrap(); let expected=if thick {std::f64::consts::PI*(10.*669./140.+2.*30.625/3.)} else {std::f64::consts::PI*(10.*83./35.+6.)};
    assert!(m.closed && (m.volume_mm3-expected).abs()<0.03,"{m:?}, expected {expected}"); c.undo().unwrap(); assert_eq!(c.document(),&before);
  }
}

#[test]
fn pipe_local_multiple_nurbs_stations_follow_sampled_length() {
  for round in [false,true] {
    let curve=Curve::Nurbs(om9_core::geometry::NurbsCurve {degree:2,poles:vec![[0.,0.,0.],[5.,0.,0.],[10.,10.,0.]],weights:vec![1.;3],knots:vec![0.,0.,0.,1.,1.,1.],domain:[0.,1.],periodic:false,reversed:false,seam:None});
    let rail=Entity::new("curve",Geometry::Curve(curve),None); let id=rail.id; let mut doc=Document::new("local multiple NURBS stations"); doc.entities.insert(id,rail);
    let mut c=Controller::new(doc,Limits::default()).unwrap(); let before=c.document().clone(); c.select_objects(&[id]).unwrap();
    c.submit("Pipe").unwrap(); if round {c.submit("Cap=Round").unwrap();} c.submit("0.3").unwrap(); c.submit("0.7").unwrap();
    c.accept_world_point([7.,4.9,0.]).unwrap(); c.submit("1.2").unwrap(); c.accept_world_point([2.5,0.625,0.]).unwrap(); c.submit("0.8").unwrap();
    c.submit("Undo").unwrap(); c.accept_world_point([2.5,0.625,0.]).unwrap(); c.submit("0.8").unwrap(); c.submit("").unwrap();
    let entity=c.document().entities.values().find(|e|e.id!=id).unwrap(); let Geometry::Brep(body)=&entity.geometry else {panic!()};
    let m=om9_kernel::creation::measure(body,c.coordinator().limits()).unwrap(); let length=|t:f64|5.*t*(1.+4.*t*t).sqrt()+2.5*(2.*t).asinh();
    let intervals=[(0.,0.25,0.3,0.8),(0.25,0.7,0.8,1.2),(0.7,1.,1.2,0.7)];
    let expected=std::f64::consts::PI*(intervals.iter().map(|&(a,b,r,s)|{let d=s-r;(length(b)-length(a))*(r*r+r*d+13.*d*d/35.)}).sum::<f64>()+if round {2.*(0.3_f64.powi(3)+0.7_f64.powi(3))/3.} else {0.});
    assert!(m.closed && (m.volume_mm3-expected).abs()<0.003*expected,"{m:?}, expected {expected}");
    c.undo().unwrap(); assert_eq!(c.document(),&before);
  }
}

#[test]
fn pipe_local_intermediate_station_on_line_and_arc() {
    for curved in [false,true] {for capped in [false,true] {
        let curve=if curved {om9_core::curve_build::arc_by_center([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],5.,0.,std::f64::consts::FRAC_PI_2).unwrap()}
            else {Curve::Line {start:[0.,0.,0.],end:[10.,0.,0.],domain:[0.,1.]}};
        let rail=Entity::new("rail",Geometry::Curve(curve),None); let id=rail.id; let mut doc=Document::new("local pipe station"); doc.entities.insert(id,rail);
        let mut c=Controller::new(doc,Limits::default()).unwrap(); let before=c.document().clone(); c.select_objects(&[id]).unwrap();
        c.submit("Pipe").unwrap(); if !capped {c.submit("Cap=None").unwrap();} c.submit("1").unwrap(); c.submit("3").unwrap();
        let q=5./2_f64.sqrt(); let station=if curved {[q,q,0.]} else {[5.,0.,0.]}; c.accept_world_point(station).unwrap(); c.submit("4").unwrap(); c.submit("").unwrap();
        let entity=c.document().entities.values().find(|e|e.id!=id).unwrap(); let Geometry::Brep(body)=&entity.geometry else {panic!()};
        let m=om9_kernel::creation::measure(body,c.coordinator().limits()).unwrap(); let length=if curved {5.*std::f64::consts::FRAC_PI_2} else {10.};
        let expected=std::f64::consts::PI*length*69./7.;
        if capped {assert!(m.closed && (m.volume_mm3-expected).abs()<0.03*expected,"{m:?}, expected {expected}");}
        else {assert!(!m.closed && m.solids==0 && m.naked_edges>0,"{m:?}");}
        c.undo().unwrap(); assert_eq!(c.document(),&before);
    }}
}

#[test]
fn pipe_local_thick_variable_radii_on_line_and_arc() {
    for curved in [false,true] {for capped in [false,true] {
        let curve=if curved {om9_core::curve_build::arc_by_center([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,0.,std::f64::consts::FRAC_PI_2).unwrap()}
            else {Curve::Line {start:[0.,0.,0.],end:[10.,0.,0.],domain:[0.,1.]}};
        let rail=Entity::new("rail",Geometry::Curve(curve),None); let id=rail.id; let mut doc=Document::new("local thick pipe"); doc.entities.insert(id,rail);
        let mut c=Controller::new(doc,Limits::default()).unwrap(); let before=c.document().clone(); c.select_objects(&[id]).unwrap();
        c.submit("Pipe").unwrap(); c.submit("Thick=Yes").unwrap(); if !capped {c.submit("Cap=None").unwrap();}
        for v in ["1","2","1.5","1.5"] {c.submit(v).unwrap();}
        assert!(c.submit("").is_err()); assert_eq!(c.document(),&before); assert_eq!(c.coordinator().undo_len(),0);
        c.submit("Undo").unwrap(); c.submit("3").unwrap(); c.submit("").unwrap();
        let entity=c.document().entities.values().find(|e|e.id!=id).unwrap(); let Geometry::Brep(body)=&entity.geometry else {panic!()};
        let m=om9_kernel::creation::measure(body,c.coordinator().limits()).unwrap(); let length=if curved {10.*std::f64::consts::FRAC_PI_2} else {10.};
        let expected=std::f64::consts::PI*length*669./140.;
        if capped {assert!(m.closed && (m.volume_mm3-expected).abs()<0.03*expected,"{m:?}, expected {expected}");}
        else {assert!(!m.closed && m.solids==0 && m.naked_edges>0,"{m:?}");}
        c.undo().unwrap(); assert_eq!(c.document(),&before);
    }}
}

#[test]
fn pipe_local_variable_radii_on_open_arc_and_open_cap() {
    for capped in [false,true] {
        let curve=om9_core::curve_build::arc_by_center([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],5.,0.,std::f64::consts::FRAC_PI_2).unwrap();
        let rail=Entity::new("arc",Geometry::Curve(curve),None); let id=rail.id; let mut doc=Document::new("local arc pipe"); doc.entities.insert(id,rail);
        let mut c=Controller::new(doc,Limits::default()).unwrap(); let before=c.document().clone(); c.select_objects(&[id]).unwrap();
        c.submit("Pipe").unwrap(); c.submit("ShapeBlending=Global").unwrap(); c.submit("ShapeBlending=Local").unwrap(); if !capped {c.submit("Cap=None").unwrap();} c.submit("1").unwrap(); c.submit("2").unwrap(); c.submit("").unwrap();
        let entity=c.document().entities.values().find(|e|e.id!=id).unwrap(); let Geometry::Brep(body)=&entity.geometry else {panic!()};
        let m=om9_kernel::creation::measure(body,c.coordinator().limits()).unwrap();
        let expected=std::f64::consts::PI*5.*std::f64::consts::FRAC_PI_2*83./35.;
        if capped {assert!(m.closed && (m.volume_mm3-expected).abs()<0.03*expected,"{m:?}, expected {expected}");}
        else {assert!(!m.closed && m.solids==0 && m.naked_edges>0,"{m:?}");}
        c.undo().unwrap(); assert_eq!(c.document(),&before);
    }
}

#[test]
fn pipe_default_local_requires_both_radii_and_confirmation() {
    for via_menu in [true, false] {
        let mut doc = Document::new("Pipe stages");
        let rail = Entity::new("rail", Geometry::Curve(Curve::Line {
            start: [0.0, 0.0, 0.0], end: [0.0, 0.0, 10.0], domain: [0.0, 1.0],
        }), None);
        let id = rail.id;
        doc.entities.insert(id, rail);
        let mut c = Controller::new(doc, Limits::default()).unwrap();
        let before = c.document().clone();
        c.select_objects(&[id]).unwrap();
        if via_menu { c.dispatch("OM9-SOLID-011", InvocationOrigin::Menu).unwrap(); }
        else { c.submit("Pipe").unwrap(); }
        assert!(c.prompt().contains("Start radius"));
        c.submit("1").unwrap();
        assert!(c.prompt().contains("End radius"));
        assert!(c.submit("").is_err());
        assert_eq!(c.coordinator().undo_len(), 0);
        c.submit("2").unwrap();
        assert_eq!(c.document(), &before);
        c.submit("").unwrap();
        let result = c.document().entities.values().find(|e| e.id != id).unwrap();
        let Geometry::Brep(brep) = &result.geometry else { panic!() };
        let m = om9_kernel::creation::measure(brep, c.coordinator().limits()).unwrap();
        assert!((m.volume_mm3 - 10.0 * std::f64::consts::PI * 83.0 / 35.0).abs() < 0.01);
        c.undo().unwrap();
        assert_eq!(c.document(), &before);
    }
}

#[test]
fn pipe_round_caps_create_analytic_capsule() {
    for via_menu in [true, false] {
        for global in [false, true] {
            let mut doc = Document::new("round Pipe");
            let mut rail = Entity::new("rail", Geometry::Curve(Curve::Line {
                start: [0.0, 0.0, 0.0], end: [0.0, 0.0, 10.0], domain: [0.0, 1.0],
            }), None);
            rail.placement = Transform::translation([5.0, -4.0, 3.0]);
            let id = rail.id;
            doc.entities.insert(id, rail);
            let mut c = Controller::new(doc, Limits::default()).unwrap();
            let before = c.document().clone();
            c.select_objects(&[id]).unwrap();
            if via_menu { c.dispatch("OM9-SOLID-011", InvocationOrigin::Menu).unwrap(); }
            else { c.submit("Pipe").unwrap(); }
            c.submit("Cap=Round").unwrap();
            if global { c.submit("ShapeBlending=Global").unwrap(); }
            c.submit("1").unwrap();
            c.submit("1").unwrap();
            c.submit("").unwrap();
            let result = c.document().entities.values().find(|e| e.id != id).unwrap();
            let Geometry::Brep(brep) = &result.geometry else { panic!() };
            let m = om9_kernel::creation::measure(brep, c.coordinator().limits()).unwrap();
            assert!(m.closed && m.solids == 1);
            assert!((m.volume_mm3 - (10.0 + 4.0 / 3.0) * std::f64::consts::PI).abs() < 1e-6, "{m:?}");
            for (actual, expected) in m.bounds_mm.iter().zip([4.0, -5.0, 2.0, 6.0, -3.0, 14.0]) {
                assert!((actual - expected).abs() < 1e-5, "{:?}", m.bounds_mm);
            }
            assert_eq!(c.coordinator().undo_len(), 1);
            c.undo().unwrap();
            assert_eq!(c.document(), &before);
        }
    }
}

#[test]
fn pipe_round_global_and_thick_have_measured_hemispheres() {
    for via_menu in [true, false] {
        for thick in [false, true] {
            let mut doc = Document::new("tapered round Pipe");
            let mut rail = Entity::new("rail", Geometry::Curve(Curve::Line {
                start: [0.0, 0.0, 0.0], end: [0.0, 0.0, 10.0], domain: [0.0, 1.0],
            }), None);
            rail.placement = Transform::translation([5.0, -4.0, 3.0]);
            let id = rail.id;
            doc.entities.insert(id, rail);
            let mut c = Controller::new(doc, Limits::default()).unwrap();
            let before = c.document().clone();
            c.select_objects(&[id]).unwrap();
            if via_menu { c.dispatch("OM9-SOLID-011", InvocationOrigin::Menu).unwrap(); }
            else { c.submit("Pipe").unwrap(); }
            c.submit("Cap=Round").unwrap();
            c.submit("ShapeBlending=Global").unwrap();
            if thick { c.submit("Thick=Yes").unwrap(); }
            if thick { for r in [1.0, 2.0, 2.0, 3.0] { c.submit(&r.to_string()).unwrap(); } }
            else { c.submit("1").unwrap(); c.submit("2").unwrap(); }
            c.submit("").unwrap();
            let result = c.document().entities.values().find(|e| e.id != id).unwrap();
            let Geometry::Brep(brep) = &result.geometry else { panic!() };
            let m = om9_kernel::creation::measure(brep, c.coordinator().limits()).unwrap();
            let capsule = |a: f64, b: f64| std::f64::consts::PI *
                (10.0 * (a*a+a*b+b*b)/3.0 + 2.0*(a.powi(3)+b.powi(3))/3.0);
            let expected = if thick { capsule(2.0, 3.0) - capsule(1.0, 2.0) } else { capsule(1.0, 2.0) };
            assert!(m.closed && m.solids == 1);
            assert!((m.volume_mm3 - expected).abs() < 1e-5, "{} {expected}", m.volume_mm3);
            let (r0, r1) = if thick { (2.0, 3.0) } else { (1.0, 2.0) };
            assert!((m.bounds_mm[2] - (3.0-r0)).abs() < 1e-5);
            assert!((m.bounds_mm[5] - (13.0+r1)).abs() < 1e-5);
            assert_eq!(c.coordinator().undo_len(), 1);
            c.undo().unwrap();
            assert_eq!(c.document(), &before);
        }
    }
}

#[test]
fn pipe_thick_open_rejects_wall_gap_below_sewing_tolerance() {
    let mut doc = Document::new("thin open Pipe");
    let rail = Entity::new("rail", Geometry::Curve(Curve::Line {
        start: [0.0, 0.0, 0.0], end: [0.0, 0.0, 10.0], domain: [0.0, 1.0],
    }), None);
    let id = rail.id;
    doc.entities.insert(id, rail);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    let before = c.document().clone();
    c.select_objects(&[id]).unwrap();
    c.submit("Pipe").unwrap();
    c.submit("Thick=Yes").unwrap();
    c.submit("Cap=None").unwrap();
    for value in [1.0, 1.00000005, 1.0, 1.00000005] { c.submit(&value.to_string()).unwrap(); }
    assert!(c.submit("").is_err());
    assert_eq!(c.coordinator().undo_len(), 0);
    assert_eq!(c.document(), &before);
    c.submit("Cancel").unwrap();
}

#[test]
fn pipe_thick_open_has_two_uncapped_walls() {
    for via_menu in [true, false] {
        for global in [false, true] {
            let mut doc = Document::new("Open Thick Pipe");
            let entity = Entity::new("rail", Geometry::Curve(Curve::Line {
                start: [0.0, 0.0, 0.0], end: [0.0, 0.0, 10.0], domain: [0.0, 1.0],
            }), None);
            let id = entity.id;
            doc.entities.insert(id, entity);
            let mut c = Controller::new(doc, Limits::default()).unwrap();
            let before = c.document().clone();
            c.select_objects(&[id]).unwrap();
            if via_menu { c.dispatch("OM9-SOLID-011", InvocationOrigin::Menu).unwrap(); }
            else { c.submit("Pipe").unwrap(); }
            c.submit("Thick=Yes").unwrap();
            c.submit("Cap=None").unwrap();
            if global { c.submit("ShapeBlending=Global").unwrap(); }
            for radius in if global { [1.0, 2.0, 2.0, 3.0] } else { [1.0, 2.0, 1.0, 2.0] } {
                c.submit(&radius.to_string()).unwrap();
            }
            c.submit("").unwrap();
            let result = c.document().entities.values().find(|e| e.id != id).unwrap();
            let Geometry::Brep(brep) = &result.geometry else { panic!() };
            let m = om9_kernel::creation::measure(brep, c.coordinator().limits()).unwrap();
            assert!(!m.closed && m.solids == 0);
            assert_eq!(m.faces, 2);
            assert_eq!(m.naked_edges, 4);
            let mut orientation = std::collections::BTreeMap::<u32, f64>::new();
            for [a, b, c, face] in &brep.display.triangles {
                let a = brep.display.points[*a as usize];
                let b = brep.display.points[*b as usize];
                let c = brep.display.points[*c as usize];
                let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
                let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
                let normal = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]];
                let radial = [(a[0] + b[0] + c[0]) / 3.0, (a[1] + b[1] + c[1]) / 3.0];
                *orientation.entry(*face).or_default() += normal[0] * radial[0] + normal[1] * radial[1];
            }
            assert_eq!(orientation.len(), 2);
            let directions: Vec<_> = orientation.values().copied().collect();
            assert!(directions[0] * directions[1] < 0.0, "wall normals must oppose: {orientation:?}");
            let expected = if global { 8.0 * std::f64::consts::PI * 101.0_f64.sqrt() }
                else { 60.0 * std::f64::consts::PI };
            assert!((m.area_mm2 - expected).abs() < 1e-5, "{m:?} {expected}");
            assert_eq!(c.coordinator().undo_len(), 1);
            c.undo().unwrap();
            assert_eq!(c.document(), &before);
        }
    }
}

#[test]
fn pipe_thick_global_creates_hollow_frustum_atomically() {
    for via_menu in [true, false] {
        let mut doc = Document::new("Thick Pipe");
        let mut entity = Entity::new("rail", Geometry::Curve(Curve::Line {
            start: [0.0, 0.0, 0.0], end: [0.0, 0.0, 10.0], domain: [0.0, 1.0],
        }), None);
        entity.placement = Transform::translation([5.0, -4.0, 3.0]);
        let id = entity.id;
        doc.entities.insert(id, entity);
        let mut c = Controller::new(doc, Limits::default()).unwrap();
        let before = c.document().clone();
        c.select_objects(&[id]).unwrap();
        if via_menu { c.dispatch("OM9-SOLID-011", InvocationOrigin::Menu).unwrap(); }
        else { c.submit("Pipe").unwrap(); }
        c.submit("Thick=Yes").unwrap();
        c.submit("ShapeBlending=Global").unwrap();
        c.submit("1").unwrap();
        assert!(c.submit("Thick=No").is_err());
        c.submit("2").unwrap();
        c.submit("2").unwrap();
        c.submit("2").unwrap();
        assert!(c.submit("").is_err());
        assert_eq!(c.coordinator().undo_len(), 0);
        c.submit("Undo").unwrap();
        c.submit("3").unwrap();
        c.submit("").unwrap();
        let result = c.document().entities.values().find(|e| e.id != id).unwrap();
        let Geometry::Brep(brep) = &result.geometry else { panic!() };
        let m = om9_kernel::creation::measure(brep, c.coordinator().limits()).unwrap();
        assert!(m.closed);
        let expected = 10.0 * std::f64::consts::PI * ((4.0 + 6.0 + 9.0) - (1.0 + 2.0 + 4.0)) / 3.0;
        assert!((m.volume_mm3 - expected).abs() < 1e-5, "{} {expected}", m.volume_mm3);
        for (actual, expected) in m.bounds_mm.iter().zip([2.0, -7.0, 3.0, 8.0, -1.0, 13.0]) {
            assert!((actual - expected).abs() < 1e-5, "{:?}", m.bounds_mm);
        }
        assert_eq!(c.coordinator().undo_len(), 1);
        c.undo().unwrap();
        assert_eq!(c.document(), &before);
    }
}

#[test]
fn pipe_diameter_and_radius_options_set_numeric_units() {
    use om9_app::command_options::{CommandOptionInterface, CommandValue};
    for via_menu in [true, false] {
        for global in [false, true] {
            let mut doc = Document::new("Pipe diameter");
            let entity = Entity::new("rail", Geometry::Curve(Curve::Line {
                start: [0.0, 0.0, 0.0], end: [0.0, 0.0, 10.0], domain: [0.0, 1.0],
            }), None);
            let id = entity.id;
            doc.entities.insert(id, entity);
            let mut c = Controller::new(doc, Limits::default()).unwrap();
            let before = c.document().clone();
            c.select_objects(&[id]).unwrap();
            if via_menu { c.dispatch("OM9-SOLID-011", InvocationOrigin::Menu).unwrap(); }
            else { c.submit("Pipe").unwrap(); }
            let option = c.command_options().into_iter().find(|o| o.id.name() == "Diameter").unwrap();
            assert_eq!(option.value, CommandValue::Flag(false));
            assert_eq!(option.label, "Diameter=No");
            assert!(option.shortcut.is_some());
            c.activate_command_option(option.id).unwrap();
            let option = c.command_options().into_iter().find(|o| o.id.name() == "Diameter").unwrap();
            assert_eq!(option.value, CommandValue::Flag(true));
            assert_eq!(option.label, "Diameter=Yes");
            if global { c.submit("ShapeBlending=Global").unwrap(); }
            assert!(c.prompt().contains("diameter") || c.prompt().contains("Diameter"));
            assert!(c.submit("0").is_err());
            c.submit("2").unwrap();
            if global {
                c.submit("Radius").unwrap();
                assert!(c.prompt().contains("radius"));
                c.submit("2").unwrap();
            } else { c.submit("2").unwrap(); }
            c.submit("").unwrap();
            let result = c.document().entities.values().find(|e| e.id != id).unwrap();
            let Geometry::Brep(brep) = &result.geometry else { panic!() };
            let m = om9_kernel::creation::measure(brep, c.coordinator().limits()).unwrap();
            let expected = if global { 70.0 * std::f64::consts::PI / 3.0 } else { 10.0 * std::f64::consts::PI };
            assert!((m.volume_mm3 - expected).abs() < 1e-6);
            c.undo().unwrap();
            assert_eq!(c.document(), &before);
        }
    }
}

#[test]
fn pipe_cap_none_and_flat_control_brep_closure() {
    for via_menu in [true, false] {
        for global in [false, true] {
            for capped in [false, true] {
                let mut doc = Document::new("Pipe caps");
                let entity = Entity::new("rail", Geometry::Curve(Curve::Line {
                    start: [0.0, 0.0, 0.0], end: [0.0, 0.0, 10.0], domain: [0.0, 1.0],
                }), None);
                let id = entity.id;
                doc.entities.insert(id, entity);
                let mut c = Controller::new(doc, Limits::default()).unwrap();
                let before = c.document().clone();
                c.select_objects(&[id]).unwrap();
                if via_menu { c.dispatch("OM9-SOLID-011", InvocationOrigin::Menu).unwrap(); }
                else { c.submit("Pipe").unwrap(); }
                assert!(c.submit("Cap=Invalid").is_err());
                c.submit(if capped { "Cap=Flat" } else { "Cap=None" }).unwrap();
                if global { c.submit("ShapeBlending=Global").unwrap(); }
                c.submit("1").unwrap();
                c.submit(if global { "2" } else { "1" }).unwrap();
                c.submit("").unwrap();
                let result = c.document().entities.values().find(|e| e.id != id).unwrap();
                let Geometry::Brep(brep) = &result.geometry else { panic!() };
                let m = om9_kernel::creation::measure(brep, c.coordinator().limits()).unwrap();
                assert_eq!(m.closed, capped);
                assert!((m.size_mm()[2] - 10.0).abs() < 1e-5);
                if capped {
                    let expected = if global { 70.0 * std::f64::consts::PI / 3.0 } else { 10.0 * std::f64::consts::PI };
                    assert!((m.volume_mm3 - expected).abs() < 1e-6);
                }
                c.undo().unwrap();
                assert_eq!(c.document(), &before);
            }
        }
    }
}

#[test]
fn pipe_global_stages_two_radii_and_builds_measured_frustum() {
    for via_menu in [true, false] {
        let mut doc = Document::new("Global Pipe");
        let mut entity = Entity::new("rail", Geometry::Curve(Curve::Line {
            start: [0.0, 0.0, 0.0], end: [0.0, 0.0, 10.0], domain: [0.0, 1.0],
        }), None);
        entity.placement = Transform::translation([5.0, -4.0, 3.0]);
        let id = entity.id;
        doc.entities.insert(id, entity);
        let mut c = Controller::new(doc, Limits::default()).unwrap();
        let before = c.document().clone();
        c.select_objects(&[id]).unwrap();
        if via_menu { c.dispatch("OM9-SOLID-011", InvocationOrigin::Menu).unwrap(); }
        else { c.submit("Pipe").unwrap(); }
        c.submit("ShapeBlending=Global").unwrap();
        c.submit("1").unwrap();
        assert!(c.submit("0").is_err());
        assert!(c.prompt().contains("End radius"));
        c.submit("2").unwrap();
        assert!(c.submit("3").is_err());
        assert_eq!(c.document(), &before);
        c.submit("").unwrap();
        let result = c.document().entities.values().find(|e| e.id != id).unwrap();
        let Geometry::Brep(brep) = &result.geometry else { panic!() };
        let m = om9_kernel::creation::measure(brep, c.coordinator().limits()).unwrap();
        assert!(m.closed);
        assert!((m.volume_mm3 - 70.0 * std::f64::consts::PI / 3.0).abs() < 1e-6);
        for (actual, expected) in m.bounds_mm.iter().zip([3.0, -6.0, 3.0, 7.0, -2.0, 13.0]) {
            assert!((actual - expected).abs() < 1e-5, "{:?}", m.bounds_mm);
        }
        c.undo().unwrap();
        assert_eq!(c.document(), &before);
    }
}

#[test]
fn pipe_follows_placed_curve_in_menu_and_command_paths() {
    let placements = [
        (Transform::translation([-7.0, 14.0, 5.0]), [-8.0, 13.0, 5.0, -6.0, 15.0, 15.0]),
        (Transform([[0.0, -1.0, 0.0, 20.0], [0.0, 0.0, -1.0, 30.0], [1.0, 0.0, 0.0, 40.0], [0.0, 0.0, 0.0, 1.0]]), [19.0, 20.0, 39.0, 21.0, 30.0, 41.0]),
    ];
    for (placement, expected) in placements {
        for via_menu in [true, false] {
            let mut doc = Document::new("placed pipe");
            let mut entity = Entity::new("rail", Geometry::Curve(Curve::Line {
                start: [0.0; 3], end: [0.0, 0.0, 10.0], domain: [0.0, 1.0],
            }), None);
            entity.placement = placement;
            let id = entity.id;
            doc.entities.insert(id, entity);
            let mut c = Controller::new(doc, Limits::default()).unwrap();
            let before = c.document().clone();
            c.select_objects(&[id]).unwrap();
            if via_menu {
                c.dispatch("OM9-SOLID-011", InvocationOrigin::Menu).unwrap();
                c.submit("1").unwrap();
                c.submit("1").unwrap();
                c.submit("").unwrap();
            } else { c.submit("Pipe 1").unwrap(); }
            let result = c.document().entities.values().find(|e| e.id != id).unwrap();
            let Geometry::Brep(brep) = &result.geometry else { panic!() };
            let measure = om9_kernel::creation::measure(brep, c.coordinator().limits()).unwrap();
            assert!((measure.volume_mm3 - 10.0 * std::f64::consts::PI).abs() < 1e-6);
            for (actual, expected) in measure.bounds_mm.iter().zip(expected) {
                assert!((actual - expected).abs() < 1e-5, "{via_menu}: {:?}", measure.bounds_mm);
            }
            assert_eq!(c.coordinator().undo_len(), 1);
            c.undo().unwrap();
            assert_eq!(c.document(), &before);
        }
    }
}

#[test]
fn pipe_extract_fillet_shell_and_cap_have_measured_menu_results() {
    let limits = Limits::default();
    let mut doc = Document::new("pipe");
    let entity = Entity::new(
        "spine",
        Geometry::Curve(Curve::Line {
            start: [0.0; 3],
            end: [0.0, 0.0, 10.0],
            domain: [0.0, 1.0],
        }),
        None,
    );
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, limits).unwrap();
    c.select_objects(&[id]).unwrap();
    review_fixture(&c, "Pipe-before");
    c.dispatch("OM9-SOLID-011", InvocationOrigin::Menu).unwrap();
    c.submit("1").unwrap();
    c.submit("1").unwrap();
    c.submit("").unwrap();
    review_fixture(&c, "Pipe-after");
    assert!((volume(&c) - 10.0 * std::f64::consts::PI).abs() < 1e-7);
    assert_eq!(c.coordinator().undo_len(), 1);
    c.undo().unwrap();
    assert_eq!(c.document().entities.len(), 1);
    for (feature, input, expected) in [
        ("OM9-SOLID-006", "0", 1000.0),
        ("OM9-SOLID-007", "0 1", 997.8539816339745),
        ("OM9-SOLID-029", "0 1", 424.0),
    ] {
        let mut c = controller();
        c.submit("Box 10 10 10").unwrap();
        let id = *c.document().entities.keys().next().unwrap();
        c.select_objects(&[id]).unwrap();
        c.dispatch(feature, InvocationOrigin::Menu).unwrap();
        c.submit(input).unwrap();
        review_fixture(&c, &format!("{feature}-after"));
        assert_eq!(c.coordinator().undo_len(), 2);
        if feature == "OM9-SOLID-006" {
            let Geometry::Brep(remainder) = &c.document().entities[&id].geometry else {
                panic!()
            };
            let remaining = creation::measure(remainder, &limits).unwrap();
            assert!(!remaining.closed);
            assert_eq!(remaining.faces, 5);
            let extracted = c.document().entities.values().find(|e| e.id != id).unwrap();
            let Geometry::Brep(b) = &extracted.geometry else {
                panic!()
            };
            let m = creation::measure(b, &limits).unwrap();
            assert_eq!(m.faces, 1);
            assert!((m.area_mm2 - 100.0).abs() < 1e-7);
        } else {
            assert!(
                (volume(&c) - expected).abs() < 1e-6,
                "{feature}: {}",
                volume(&c)
            );
        }
        c.undo().unwrap();
        assert!((volume(&c) - 1000.0).abs() < 1e-7);
    }
    let boxed = creation::primitive(
        creation::Primitive::Box {
            x_mm: 10.0,
            y_mm: 10.0,
            z_mm: 10.0,
        },
        0.01,
        &limits,
    )
    .unwrap();
    let open = om9_kernel::editing::remove_face(&boxed, 0, 0.01, &limits).unwrap();
    let mut doc = Document::new("cap");
    let entity = Entity::new("open box", Geometry::Brep(open), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, limits).unwrap();
    c.select_objects(&[id]).unwrap();
    review_fixture(&c, "Cap-before");
    c.dispatch("OM9-SOLID-005", InvocationOrigin::Menu).unwrap();
    c.submit("").unwrap();
    review_fixture(&c, "Cap-after");
    assert!((volume(&c) - 1000.0).abs() < 1e-7);
    assert_eq!(c.coordinator().undo_len(), 1);
    println!("menu_measured Pipe Extract Fillet Shell Cap=PASS");
}

#[test]
#[ignore = "requires files saved by the native UI; set OM9_SOLID_NATIVE_REVIEW_DIRECTORY"]
fn native_ui_saved_solid_workflows() {
    let directory = std::path::PathBuf::from(
        std::env::var_os("OM9_SOLID_NATIVE_REVIEW_DIRECTORY").expect("native review directory"),
    );
    let limits = Limits::default();
    for (name, entities, solids, expected) in [
        ("text-live", 1, 1, None),
        ("boolean2-live", 1, 1, Some(1500.0)),
        ("extrude-all-live", 4, 2, Some(90.0 * std::f64::consts::PI)),
    ] {
        let restored = om9_archive::Archive::open_restored(
            &directory.join(format!("{name}.om9")),
            &limits,
            &Default::default(),
        )
        .unwrap();
        assert_eq!(restored.document.entities.len(), entities);
        let mut count = 0;
        let mut total = 0.0;
        for entity in restored.document.entities.values() {
            if let Geometry::Brep(brep) = &entity.geometry {
                let measured = creation::measure(brep, &limits).unwrap();
                assert_eq!(measured.solids, 1);
                assert!(measured.closed);
                assert_eq!(measured.naked_edges, 0);
                assert!(measured.volume_mm3 > 0.0);
                count += 1;
                total += measured.volume_mm3;
            }
        }
        assert_eq!(count, solids);
        if let Some(expected) = expected {
            assert!((total / expected - 1.0).abs() < 1e-10);
        }
        if name == "text-live" {
            let fixture = om9_archive::Archive::open_restored(
                &directory.join("review-fixtures/Text-after.om9"),
                &limits,
                &Default::default(),
            )
            .unwrap();
            let c = Controller::new(fixture.document, limits).unwrap();
            assert!((total / volume(&c) - 1.0).abs() < 1e-10);
        }
        println!("native_ui_saved {name} entities={entities} solids={count} volume={total:.12}");
    }
}
