// SPDX-License-Identifier: LGPL-2.1-or-later
#![cfg(feature = "occt-brep")]

use eframe::egui::{self, Event, Modifiers, PointerButton, Pos2, Rect, Vec2};
use om9_app::{session::InvocationOrigin, ui::MatrixApp};
use om9_core::{
    budget::Limits,
    geometry::{Curve, Geometry, Transform},
    model::{Document, Entity},
};
use om9_scene::{camera::Viewport, workspace::ViewId};

fn frame(app: &mut MatrixApp, context: &egui::Context, events: Vec<Event>) {
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280.0, 900.0))),
            events,
            ..Default::default()
        },
        |ui| app.show(ui),
    );
    output.textures_delta.clear();
}

#[test]
fn reopened_reordered_length_routes_through_measure_icon_and_cmd() {
    let limits = Limits::default();
    let shape = om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Box { x_mm: 2., y_mm: 3., z_mm: 4. }, 0.1, &limits).unwrap();
    let mut display = shape.display().clone();
    let length = |index: usize| {
        let segment = display.segments.iter().find(|segment| segment[2] as usize == index).unwrap();
        om9_core::measure::distance_mm(display.points[segment[0] as usize], display.points[segment[1] as usize]).unwrap()
    };
    let first = (0..display.edges.len()).find(|&i| (length(i) - 4.).abs() < 1e-8).unwrap();
    let other = (0..display.edges.len()).find(|&i| {
        let segment = display.segments.iter().find(|segment| segment[2] as usize == i).unwrap();
        let a = display.points[segment[0] as usize]; let b = display.points[segment[1] as usize];
        (length(i) - 3.).abs() < 1e-8 && (0..3).all(|axis| ((a[axis]+b[axis])*0.5 - [2.,1.5,4.][axis]).abs() < 1e-8)
    }).unwrap();
    display.edges.swap(first, other);
    for entry in &mut display.face_edges { if entry[1] as usize == first { entry[1] = other as u32; } else if entry[1] as usize == other { entry[1] = first as u32; } }
    for entry in &mut display.segments { if entry[2] as usize == first { entry[2] = other as u32; } else if entry[2] as usize == other { entry[2] = first as u32; } }
    let edge = display.edges[first];
    let shape = om9_core::brep::Brep::from_native(shape.source().to_owned(), display, &limits).unwrap();
    let mut document = Document::new("Length entry routes");
    let entity = Entity::new("box", Geometry::Brep(shape), document.current_layer);
    let id = entity.id;
    document.entities.insert(id, entity);
    let path = std::env::temp_dir().join(format!("om9-length-ui-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, &document, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    for menu in [true, false] {
        let mut app = MatrixApp::from_file(&path).unwrap();
        let before = app.controller.document().clone();
        let ctx = egui::Context::default();
        frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
        app.set_sub_object_kind(om9_scene::scene::PickKind::Edge);
        click_in_view(&mut app, &ctx, ViewId::Perspective, [2.,1.5,4.], Modifiers { ctrl: true, shift: true, ..Modifiers::NONE });
        assert!(app.controller.subselection().iter().any(|selected| selected.root == id && selected.subset == om9_scene::scene::Subelement::BrepEdge(edge)), "viewport did not select expected edge");
        if menu {
            let at = app.top_menu_rect(3).unwrap().center();
            for pressed in [true, false] { frame(&mut app, &ctx, vec![Event::PointerMoved(at), Event::PointerButton {
                pos: at, button: PointerButton::Primary, pressed, modifiers: Modifiers::NONE }]); }
            let index = om9_app::menu_catalog::MEASURE.iter().position(|name| *name == "AnalyzeLength").unwrap();
            frame(&mut app, &ctx, vec![]);
            let at = app.rail_icon_rects[2 + index].center();
            for pressed in [true, false] { frame(&mut app, &ctx, vec![Event::PointerMoved(at), Event::PointerButton {
                pos: at, button: PointerButton::Primary, pressed, modifiers: Modifiers::NONE }]); }
        } else {
            frame(&mut app, &ctx, vec![Event::Text("Length".into())]);
            frame(&mut app, &ctx, vec![Event::Key { key: egui::Key::Enter, physical_key: None,
                pressed: true, repeat: false, modifiers: Modifiers::NONE }]);
        }
        assert!(app.transcript.history().contains("Length = 3 mm"), "menu={menu} {}", app.transcript.history());
        assert_eq!(app.controller.document(), &before);
    }
    std::fs::remove_file(path).unwrap();
}

#[test]
fn viewport_selected_box_face_extracts_isocurve_through_cmd() {
    use om9_core::evaluation::{evaluate_curve,Parameter};
    let limits=Limits::default();let shape=om9_kernel::brep::box_brep([2.,3.,4.],0.1,&limits).unwrap();
    let entity=Entity::new("box",Geometry::Brep(shape),None);let id=entity.id;
    let mut document=Document::with_template_tolerance("isocurve face UI");document.entities.insert(id,entity);
    let mut app=MatrixApp::new(document).unwrap();let before=app.controller.document().clone();let ctx=egui::Context::default();
    frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);app.set_sub_object_kind(om9_scene::scene::PickKind::Face);
    click_in_view(&mut app,&ctx,ViewId::Perspective,[1.,1.5,4.],Modifiers {ctrl:true,shift:true,..Modifiers::NONE});
    assert!(app.controller.subselection().iter().any(|s|s.root==id && matches!(s.subset,om9_scene::scene::Subelement::BrepFace(_))));
    frame(&mut app,&ctx,vec![Event::Text("ExtractIsocurve".into())]);
    frame(&mut app,&ctx,vec![Event::Key {key:egui::Key::Enter,physical_key:None,pressed:true,repeat:false,modifiers:Modifiers::NONE}]);
    assert!(!app.controller.isocurve_option_tokens().is_empty(),"{}",app.transcript.history());
    frame(&mut app,&ctx,vec![Event::Key {key:egui::Key::Enter,physical_key:None,pressed:true,repeat:false,modifiers:Modifiers::NONE}]);
    let output=app.controller.document().entities.values().find(|e|e.label=="ExtractIsocurve").expect("isocurve output");
    let Geometry::Curve(curve)=&output.geometry else {panic!("curve")};
    for t in [0.,0.5,1.] {let p=evaluate_curve(curve,Parameter::Fraction(t),0,output.placement,&limits).unwrap()[0];assert!((p[2]-4.).abs()<1e-7,"{p:?}");}
    let after=app.controller.document().clone();app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
    app.controller.redo().unwrap();assert_eq!(app.controller.document(),&after);
}

#[test]
fn viewport_selected_circular_hole_face_extracts_isocurve_after_reopen() {
    use om9_kernel::{creation::{self,Primitive,Profile},editing::{self,BooleanOp,BooleanOutcome}};
    let limits=Limits::default();
    let profile=Profile::new(vec![[-5.,-5.,0.],[5.,-5.,0.],[5.,5.,0.],[-5.,5.,0.]]).unwrap();
    let block=creation::extrude(&profile,[0.,0.,10.],true,0.2,&limits).unwrap();
    let cutter=creation::primitive(Primitive::Cylinder {radius_mm:2.,height_mm:10.},0.2,&limits).unwrap();
    let BooleanOutcome::Body(body)=editing::boolean(&block,&cutter,BooleanOp::Difference,0.2,&limits).unwrap() else {panic!("cut")};
    let entity=Entity::new("drilled block",Geometry::Brep(body),None);let id=entity.id;
    let mut doc=Document::with_template_tolerance("drilled block");doc.entities.insert(id,entity);
    let path=std::env::temp_dir().join(format!("om9-circular-hole-{}.om9",uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path,&doc,&limits,&om9_core::transaction::Cancellation::default()).unwrap();
    for menu in [false,true] {
        let mut app=MatrixApp::from_file(&path).unwrap();
        app.controller.workspace.views.get_mut(&ViewId::LookingDown).unwrap().camera=
            om9_scene::camera::Camera::orthographic([3.,1.,30.],[3.,1.,10.],[0.,1.,0.],14.).unwrap();
        let before=app.controller.document().clone();let ctx=egui::Context::default();
        frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);app.set_sub_object_kind(om9_scene::scene::PickKind::Face);
        click_in_view(&mut app,&ctx,ViewId::LookingDown,[3.,1.,10.],Modifiers {ctrl:true,shift:true,..Modifiers::NONE});
        assert!(app.controller.subselection().iter().any(|s|s.root==id));
        if menu {app.controller.dispatch("OM9-CURVE-019",InvocationOrigin::Menu).unwrap();}
        else {app.controller.submit("ExtractIsocurve").unwrap();}
        app.controller.submit("").unwrap();
        assert_eq!(app.controller.document().entities.values().filter(|e|e.label=="ExtractIsocurve").count(),2);
        let after=app.controller.document().clone();app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
        app.controller.redo().unwrap();assert_eq!(app.controller.document(),&after);
    }
    std::fs::remove_file(path).unwrap();
}

#[test]
fn viewport_selected_native_box_face_extracts_isocurve_through_cmd() {
    use om9_core::evaluation::{evaluate_curve,Parameter};
    let limits=Limits::default();let shape=om9_kernel::brep::box_brep([2.,3.,4.],0.1,&limits).unwrap();
    let native=om9_kernel::brep::to_3dm(&shape,1e-6,&limits).unwrap();
    let entity=Entity::new("native box",Geometry::NativeBrep(native),None);let id=entity.id;
    let mut document=Document::with_template_tolerance("native isocurve UI");document.entities.insert(id,entity);
    let path=std::env::temp_dir().join(format!("om9-native-isocurve-ui-{}.om9",uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path,&document,&limits,&om9_core::transaction::Cancellation::default()).unwrap();
    for menu in [false,true] {
    let mut app=MatrixApp::from_file(&path).unwrap();
    let before=app.controller.document().clone();let ctx=egui::Context::default();
    frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);app.set_sub_object_kind(om9_scene::scene::PickKind::Face);
    click_in_view(&mut app,&ctx,ViewId::Perspective,[1.,1.5,4.],Modifiers {ctrl:true,shift:true,..Modifiers::NONE});
    assert!(app.controller.subselection().iter().any(|s|s.root==id && matches!(s.subset,om9_scene::scene::Subelement::MeshFace(_))));
    if menu {
        let at=app.main_category_rect(0).unwrap().center();
        for pressed in [true,false] {frame(&mut app,&ctx,vec![Event::PointerMoved(at),Event::PointerButton {pos:at,button:PointerButton::Primary,pressed,modifiers:Modifiers::NONE}]);}
        let index=om9_app::menu_catalog::CURVE.iter().position(|name|*name=="CurveCurveFromObjectsExtractIsocurve").unwrap();
        ctx.memory_mut(|memory|memory.data.insert_temp(egui::Id::new(("main-menu-page","Curve")),index/20));
        frame(&mut app,&ctx,vec![]);let at=app.rail_icon_rects[2+index%20].center();
        for pressed in [true,false] {frame(&mut app,&ctx,vec![Event::PointerMoved(at),Event::PointerButton {pos:at,button:PointerButton::Primary,pressed,modifiers:Modifiers::NONE}]);}
    } else {
        frame(&mut app,&ctx,vec![Event::Text("ExtractIsocurve".into())]);
        frame(&mut app,&ctx,vec![Event::Key {key:egui::Key::Enter,physical_key:None,pressed:true,repeat:false,modifiers:Modifiers::NONE}]);
    }
    assert!(!app.controller.isocurve_option_tokens().is_empty(),"{}",app.transcript.history());
    if menu {
        let at=app.command_input_rect().unwrap().center();
        for pressed in [true,false] {frame(&mut app,&ctx,vec![Event::PointerMoved(at),Event::PointerButton {pos:at,button:PointerButton::Primary,pressed,modifiers:Modifiers::NONE}]);}
    }
    frame(&mut app,&ctx,vec![Event::Key {key:egui::Key::Enter,physical_key:None,pressed:true,repeat:false,modifiers:Modifiers::NONE}]);
    let output=app.controller.document().entities.values().find(|e|e.label=="ExtractIsocurve").unwrap_or_else(||panic!("menu={menu} {}",app.transcript.history()));
    let Geometry::Curve(curve)=&output.geometry else {panic!("curve")};
    for t in [0.,0.5,1.] {let p=evaluate_curve(curve,Parameter::Fraction(t),0,output.placement,&limits).unwrap()[0];assert!((p[2]-4.).abs()<1e-7,"{p:?}");}
    let after=app.controller.document().clone();app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
    app.controller.redo().unwrap();assert_eq!(app.controller.document(),&after);
    }
    std::fs::remove_file(path).unwrap();
}

#[test]
fn selected_large_face_isocurve_worker_polls_in_egui() {
    let limits=Limits::default();let shape=om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Sphere {radius_mm:3.},0.001,&limits).unwrap();
    assert!(shape.display().triangles.len()>20_000);let face=shape.topology().faces[0];
    let entity=Entity::new("sphere",Geometry::Brep(shape),None);let id=entity.id;
    let mut doc=Document::with_template_tolerance("isocurve face worker UI");doc.entities.insert(id,entity);
    let mut app=MatrixApp::new(doc).unwrap();let before=app.controller.document().clone();let ctx=egui::Context::default();
    frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);app.controller.select_objects(&[id]).unwrap();
    app.controller.select_subobject(id,om9_scene::scene::Subelement::BrepFace(face)).unwrap();
    frame(&mut app,&ctx,vec![Event::Text("ExtractIsocurve".into())]);
    let start=std::time::Instant::now();frame(&mut app,&ctx,vec![Event::Key {key:egui::Key::Enter,physical_key:None,
        pressed:true,repeat:false,modifiers:Modifiers::NONE}]);
    assert!(start.elapsed()<std::time::Duration::from_secs(1));assert!(app.controller.isocurve_face_verification_pending());
    let deadline=std::time::Instant::now()+std::time::Duration::from_secs(30);
    while app.controller.face_verification_pending() {frame(&mut app,&ctx,vec![]);assert!(std::time::Instant::now()<deadline);
        std::thread::sleep(std::time::Duration::from_millis(5));}
    assert!(!app.controller.isocurve_option_tokens().is_empty(),"{}",app.transcript.history());
    frame(&mut app,&ctx,vec![Event::Key {key:egui::Key::Enter,physical_key:None,pressed:true,repeat:false,modifiers:Modifiers::NONE}]);
    assert_eq!(app.controller.document().entities.values().filter(|e|e.label=="ExtractIsocurve").count(),1);
    let after=app.controller.document().clone();app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
    app.controller.redo().unwrap();assert_eq!(app.controller.document(),&after);
}

#[test]
fn large_selected_face_extract_polls_worker_in_egui() {
    let limits = Limits::default();
    let shape = om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Sphere { radius_mm: 3. }, 0.001, &limits).unwrap();
    assert!(shape.display().triangles.len() > 20_000);
    let mut document = Document::with_template_tolerance("async extract UI");
    let entity = Entity::new("sphere", Geometry::Brep(shape), None);
    let id = entity.id; document.entities.insert(id, entity);
    let mut app = MatrixApp::new(document).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
    app.controller.select_objects(&[id]).unwrap();
    app.controller.submit("ExtractSrf").unwrap(); app.controller.submit("Copy=Yes").unwrap();
    click_in_view(&mut app, &ctx, ViewId::Perspective, [0., 0., 3.], Modifiers::NONE);
    assert_eq!(app.controller.subselection().len(), 1, "{}", app.transcript.history());
    let before = app.controller.document().clone();
    app.controller.submit("").unwrap();
    assert!(app.controller.face_verification_pending());
    let start = std::time::Instant::now();
    frame(&mut app, &ctx, vec![]);
    assert!(start.elapsed() < std::time::Duration::from_secs(1), "polling a busy worker must not wait for it");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while app.controller.face_verification_pending() {
        frame(&mut app, &ctx, vec![]);
        assert!(std::time::Instant::now() < deadline, "worker did not finish");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    let output = app.controller.document().entities.values().find(|entity| entity.id != id).expect("extracted face");
    let Geometry::Brep(face) = &output.geometry else { panic!("BRep face") };
    let area = om9_kernel::creation::measure(face, &limits).unwrap().area_mm2;
    assert!((area - 36. * std::f64::consts::PI).abs() < 1e-6);
    app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
    app.controller.redo().unwrap(); assert_eq!(app.controller.document().entities.len(), 2);
}

#[test]
fn large_numeric_face_extract_polls_worker_in_egui() {
    let limits = Limits::default();
    let body = om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Sphere { radius_mm: 3. }, 0.01, &limits).unwrap();
    let mut document = Document::with_template_tolerance("async numeric UI");
    let entity = Entity::new("sphere", Geometry::Brep(body), None);
    let id = entity.id; document.entities.insert(id, entity);
    let mut app = MatrixApp::new(document).unwrap();
    app.controller.select_objects(&[id]).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
    let enter_text = |app: &mut MatrixApp, text: &str| {
        frame(app, &ctx, vec![Event::Text(text.into())]);
        frame(app, &ctx, vec![Event::Key { key: egui::Key::Enter, physical_key: None,
            pressed: true, repeat: false, modifiers: Modifiers::NONE }]);
        frame(app, &ctx, vec![Event::Key { key: egui::Key::Enter, physical_key: None,
            pressed: false, repeat: false, modifiers: Modifiers::NONE }]);
    };
    enter_text(&mut app, "ExtractSrf");
    enter_text(&mut app, "Copy=Yes");
    let before = app.controller.document().clone();
    enter_text(&mut app, "0");
    assert!(app.controller.face_verification_pending());
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while app.controller.face_verification_pending() {
        frame(&mut app, &ctx, vec![]);
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    let output = app.controller.document().entities.values().find(|e| e.id != id).expect("extracted face");
    let Geometry::Brep(face) = &output.geometry else { panic!() };
    assert!((om9_kernel::creation::measure(face, &limits).unwrap().area_mm2 - 36. * std::f64::consts::PI).abs() < 1e-6);
    app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
    app.controller.redo().unwrap(); assert_eq!(app.controller.document().entities.len(), 2);
}

#[test]
fn large_inline_shell_polls_worker_in_egui() {
    let limits = Limits::default();
    let body = om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Cylinder { radius_mm: 2., height_mm: 5. }, 0.0001, &limits).unwrap();
    assert!(body.display().triangles.len() > 512);
    let cap = (0..body.topology().faces.len()).find(|&i| {
        let face = om9_kernel::editing::extract_face(&body, i, 0.1, &limits).unwrap();
        (om9_kernel::creation::measure(&face, &limits).unwrap().area_mm2 - 4. * std::f64::consts::PI).abs() < 1e-6
    }).unwrap();
    let mut document = Document::with_template_tolerance("async shell UI");
    let entity = Entity::new("cylinder", Geometry::Brep(body), None);
    let id = entity.id; document.entities.insert(id, entity);
    let mut app = MatrixApp::new(document).unwrap();
    app.controller.select_objects(&[id]).unwrap();
    let before = app.controller.document().clone();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![Event::Text(format!("Shell {cap} 0.5"))]);
    frame(&mut app, &ctx, vec![Event::Key { key: egui::Key::Enter, physical_key: None,
        pressed: true, repeat: false, modifiers: Modifiers::NONE }]);
    assert!(app.controller.face_verification_pending());
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while app.controller.face_verification_pending() {
        frame(&mut app, &ctx, vec![]);
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    let Geometry::Brep(shell) = &app.controller.document().entities[&id].geometry else { panic!() };
    assert!((om9_kernel::creation::measure(shell, &limits).unwrap().volume_mm3 - 9.875 * std::f64::consts::PI).abs() < 1e-6);
    app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
    app.controller.redo().unwrap();
}

#[test]
fn large_inline_face_extract_polls_worker_in_egui() {
    let limits = Limits::default();
    let body = om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Sphere { radius_mm: 3. }, 0.01, &limits).unwrap();
    let mut document = Document::with_template_tolerance("async inline UI");
    let entity = Entity::new("sphere", Geometry::Brep(body), None);
    let id = entity.id; document.entities.insert(id, entity);
    let mut app = MatrixApp::new(document).unwrap();
    app.controller.select_objects(&[id]).unwrap();
    let before = app.controller.document().clone();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![Event::Text("ExtractSrf 0 Copy=Yes".into())]);
    frame(&mut app, &ctx, vec![Event::Key { key: egui::Key::Enter, physical_key: None,
        pressed: true, repeat: false, modifiers: Modifiers::NONE }]);
    assert!(app.controller.face_verification_pending());
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while app.controller.face_verification_pending() {
        frame(&mut app, &ctx, vec![]);
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    let output = app.controller.document().entities.values().find(|e| e.id != id).expect("extracted face");
    let Geometry::Brep(face) = &output.geometry else { panic!() };
    assert!((om9_kernel::creation::measure(face, &limits).unwrap().area_mm2 - 36. * std::f64::consts::PI).abs() < 1e-6);
    app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
    app.controller.redo().unwrap(); assert_eq!(app.controller.document().entities.len(), 2);
}

fn click(app: &mut MatrixApp, context: &egui::Context, world: [f64; 3]) {
    click_with_modifiers(app, context, world, Modifiers::NONE);
}

fn click_with_modifiers(
    app: &mut MatrixApp,
    context: &egui::Context,
    world: [f64; 3],
    modifiers: Modifiers,
) {
    click_in_view(app, context, ViewId::LookingDown, world, modifiers);
}

fn click_in_view(app: &mut MatrixApp, context: &egui::Context, view_id: ViewId, world: [f64; 3], modifiers: Modifiers) {
    let rect = app.view_rects[&view_id];
    let viewport = Viewport {
        origin: [rect.min.x as f64, rect.min.y as f64],
        size: [rect.width() as f64, rect.height() as f64],
    };
    let p = app.controller.workspace.views[&view_id]
        .camera
        .project(world, viewport)
        .unwrap()
        .unwrap()
        .screen;
    let pos = Pos2::new(p[0] as f32, p[1] as f32);
    frame(
        app,
        context,
        vec![Event::ModifiersChanged(modifiers), Event::PointerMoved(pos)],
    );
    for pressed in [true, false] {
        frame(
            app,
            context,
            vec![Event::PointerButton {
                pos,
                button: PointerButton::Primary,
                pressed,
                modifiers,
            }],
        );
    }
}

#[test]
fn viewport_dupedge_selects_edges_after_invocation() {
    let mut app = MatrixApp::new(Document::new("viewport DupEdge")).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
    app.controller.submit("Box 4 6 8").unwrap();
    app.controller.dispatch("OM9-CURVE-040", InvocationOrigin::Menu).unwrap();
    click_in_view(&mut app, &ctx, ViewId::Perspective, [2., 0., 8.], Modifiers::NONE);
    assert_eq!(app.controller.subselection().len(), 1, "{}", app.controller.prompt());
    click_in_view(&mut app, &ctx, ViewId::Perspective, [4., 3., 8.], Modifiers::NONE);
    assert_eq!(app.controller.subselection().len(), 2, "two distinct edges should be selected");
    let rect = app.view_rects[&ViewId::Perspective];
    let vp = Viewport { origin: [rect.min.x as f64, rect.min.y as f64], size: [rect.width() as f64, rect.height() as f64] };
    let screen = app.controller.workspace.views[&ViewId::Perspective].camera.project([2., 0., 8.], vp).unwrap().unwrap().screen;
    let pos = Pos2::new(screen[0] as f32, screen[1] as f32);
    let moved = pos + Vec2::new(20., 10.);
    frame(&mut app, &ctx, vec![Event::PointerButton { pos, button: PointerButton::Secondary, pressed: true, modifiers: Modifiers::NONE }]);
    frame(&mut app, &ctx, vec![Event::PointerMoved(moved)]);
    frame(&mut app, &ctx, vec![Event::PointerButton { pos: moved, button: PointerButton::Secondary, pressed: false, modifiers: Modifiers::NONE }]);
    assert!(app.controller.dupedge_needs_pick(), "right-drag must not finish the command");
    assert_eq!(app.controller.subselection().len(), 2);
    for pressed in [true, false] {
        frame(&mut app, &ctx, vec![Event::PointerButton { pos: moved, button: PointerButton::Secondary, pressed, modifiers: Modifiers::NONE }]);
    }
    assert_eq!(app.controller.document().entities.len(), 3);
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document().entities.len(), 1);
    app.controller.redo().unwrap();
    assert_eq!(app.controller.document().entities.len(), 3);
}

#[test]
fn viewport_dupedge_reopened_reordered_source_matches_the_visible_edge() {
    let limits = Limits::default();
    let forward = om9_kernel::brep::read_brep(include_str!("fixtures/dupedge-order-a.brep"), 0.01, &limits).unwrap();
    let reverse = om9_kernel::brep::read_brep(include_str!("fixtures/dupedge-order-b.brep"), 0.01, &limits).unwrap();
    let index_of_top_edge = |shape: &om9_core::brep::Brep| {
        (0..shape.topology().edges.len()).find(|&index| {
            let Ok((a, b)) = om9_kernel::editing::oriented_edge_endpoints(shape, index, &limits) else { return false; };
            let matches = |p: [f64; 3], q: [f64; 3]| (0..3).all(|i| (p[i] - q[i]).abs() < 1e-8);
            (matches(a, [0., 0., 6.]) && matches(b, [4., 0., 6.]))
                || (matches(b, [0., 0., 6.]) && matches(a, [4., 0., 6.]))
        }).expect("top source edge")
    };
    let old_index = index_of_top_edge(&forward);
    let new_index = index_of_top_edge(&reverse);
    assert_ne!(old_index, new_index, "fixture must actually change native edge order");
    let mut old_display = forward.display().clone();
    old_display.source_sha256 = [0; 32];
    let hybrid = om9_core::brep::Brep::from_native(reverse.source().to_owned(), old_display, &limits).unwrap();
    let mut document = Document::new("reordered source and prior display");
    let entity = Entity::new("box faces", Geometry::Brep(hybrid), document.current_layer);
    document.entities.insert(entity.id, entity);
    let path = std::env::temp_dir().join(format!("om9-dupedge-reordered-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, &document, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    let mut app = MatrixApp::from_file(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
    app.controller.submit("DupEdge").unwrap();
    click_in_view(&mut app, &ctx, ViewId::Perspective, [2., 0., 6.], Modifiers::NONE);
    assert_eq!(app.controller.subselection().len(), 1, "{}", app.transcript.history());
    click_in_view(&mut app, &ctx, ViewId::Perspective, [4., 2.5, 6.], Modifiers::NONE);
    assert_eq!(app.controller.subselection().len(), 2, "{}", app.transcript.history());
    app.controller.submit("").unwrap();
    let outputs: Vec<_> = app.controller.document().entities.values().filter(|entity| entity.label == "DupEdge").collect();
    assert_eq!(outputs.len(), 2);
    let ids: Vec<_> = outputs.iter().map(|entity| entity.id).collect();
    for expected in [[[0., 0., 6.], [4., 0., 6.]], [[4., 0., 6.], [4., 5., 6.]]] {
        assert!(outputs.iter().any(|entity| {
            let Geometry::Curve(curve) = &entity.geometry else { return false; };
            let endpoints: Vec<_> = [0., 1.].into_iter().map(|fraction| om9_core::evaluation::evaluate_curve(curve,
                om9_core::evaluation::Parameter::Fraction(fraction), 0, Transform::identity(), &limits).unwrap()[0]).collect();
            expected.iter().all(|point| endpoints.iter().any(|p| (0..3).all(|i| (p[i] - point[i]).abs() < 1e-8)))
        }));
    }
    app.controller.undo().unwrap(); assert!(ids.iter().all(|id| !app.controller.document().entities.contains_key(id)));
    app.controller.redo().unwrap(); assert!(ids.iter().all(|id| app.controller.document().entities.contains_key(id)));
}

#[test]
fn viewport_dupedge_reopened_affine_box_picks_visible_edges() {
    let limits = Limits::default();
    let shape = om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Box { x_mm: 4., y_mm: 5., z_mm: 6. }, 0.01, &limits).unwrap();
    let placement = Transform([[2.,0.5,0.,0.],[0.,1.,0.,0.],[0.,0.,1.,0.],[0.,0.,0.,1.]]);
    let mut document = Document::new("reopened affine viewport DupEdge");
    let mut entity = Entity::new("box",Geometry::Brep(shape),document.current_layer); entity.placement=placement;
    document.entities.insert(entity.id,entity);
    let path=std::env::temp_dir().join(format!("om9-dupedge-affine-ui-{}.om9",uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path,&document,&limits,&om9_core::transaction::Cancellation::default()).unwrap();
    let mut app=MatrixApp::from_file(&path).unwrap(); std::fs::remove_file(path).unwrap();
    let ctx=egui::Context::default(); frame(&mut app,&ctx,vec![]); frame(&mut app,&ctx,vec![]);
    app.controller.submit("DupEdge").unwrap();
    for local in [[2.,0.,6.],[4.,2.5,6.]] {
        click_in_view(&mut app,&ctx,ViewId::Perspective,placement.point(local).unwrap(),Modifiers::NONE);
    }
    assert_eq!(app.controller.subselection().len(),2,"{}",app.transcript.history());
    app.controller.submit("").unwrap();
    let outputs: Vec<_>=app.controller.document().entities.values().filter(|e|e.label=="DupEdge").collect();
    assert_eq!(outputs.len(),2);
    for local in [[[0.,0.,6.],[4.,0.,6.]],[[4.,0.,6.],[4.,5.,6.]]] {
        let expected=local.map(|p|placement.point(p).unwrap());
        assert!(outputs.iter().any(|entity| {
            let Geometry::Curve(curve)=&entity.geometry else {return false};
            let endpoints: Vec<_>=[0.,1.].into_iter().map(|t|om9_core::evaluation::evaluate_curve(curve,
                om9_core::evaluation::Parameter::Fraction(t),0,entity.placement,&limits).unwrap()[0]).collect();
            expected.iter().all(|p|endpoints.iter().any(|q|(0..3).all(|i|(p[i]-q[i]).abs()<1e-8)))
        }));
    }
    let ids: Vec<_>=outputs.iter().map(|e|e.id).collect(); app.controller.undo().unwrap();
    assert!(ids.iter().all(|id|!app.controller.document().entities.contains_key(id)));
    app.controller.redo().unwrap(); assert!(ids.iter().all(|id|app.controller.document().entities.contains_key(id)));
}

#[test]
fn viewport_dupedge_reopened_box_via_icon_and_typed_cmd() {
    let limits = Limits::default();
    let shape = om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Box { x_mm: 4., y_mm: 5., z_mm: 6. }, 0.01, &limits).unwrap();
    let mut document = Document::new("DupEdge entry routes");
    let entity = Entity::new("box", Geometry::Brep(shape), document.current_layer);
    document.entities.insert(entity.id, entity);
    let path = std::env::temp_dir().join(format!("om9-dupedge-routes-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, &document, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    for menu in [true, false] {
        let mut app = MatrixApp::from_file(&path).unwrap();
        let ctx = egui::Context::default();
        frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
        if menu {
            let at = app.main_category_rect(0).unwrap().center();
            for pressed in [true, false] {
                frame(&mut app, &ctx, vec![Event::PointerMoved(at), Event::PointerButton { pos: at,
                    button: PointerButton::Primary, pressed, modifiers: Modifiers::NONE }]);
            }
            let index = om9_app::menu_catalog::CURVE.iter().position(|name| *name == "CurveCurveFromObjectsDuplicateEdge").unwrap();
            ctx.memory_mut(|memory| memory.data.insert_temp(egui::Id::new(("main-menu-page", "Curve")), index / 20));
            frame(&mut app, &ctx, vec![]);
            let at = app.rail_icon_rects[2 + index % 20].center();
            for pressed in [true, false] {
                frame(&mut app, &ctx, vec![Event::PointerMoved(at), Event::PointerButton { pos: at,
                    button: PointerButton::Primary, pressed, modifiers: Modifiers::NONE }]);
            }
        } else {
            frame(&mut app, &ctx, vec![Event::Text("DupEdge".into())]);
            frame(&mut app, &ctx, vec![Event::Key { key: egui::Key::Enter, physical_key: None,
                pressed: true, repeat: false, modifiers: Modifiers::NONE }]);
        }
        assert_eq!(app.controller.active_feature(), Some("OM9-CURVE-040"), "menu={menu} {}", app.transcript.history());
        click_in_view(&mut app, &ctx, ViewId::Perspective, [2., 0., 6.], Modifiers::NONE);
        assert_eq!(app.controller.subselection().len(), 1, "menu={menu} {}", app.transcript.history());
        let rect = app.view_rects[&ViewId::Perspective];
        let at = rect.center();
        for pressed in [true, false] {
            frame(&mut app, &ctx, vec![Event::PointerMoved(at), Event::PointerButton { pos: at,
                button: PointerButton::Secondary, pressed, modifiers: Modifiers::NONE }]);
        }
        let output = app.controller.document().entities.values().find(|e| e.label == "DupEdge").unwrap();
        let id = output.id;
        let Geometry::Curve(curve) = &output.geometry else { panic!() };
        let endpoints: Vec<_> = [0., 1.].into_iter().map(|fraction| om9_core::evaluation::evaluate_curve(curve,
            om9_core::evaluation::Parameter::Fraction(fraction), 0, Transform::identity(), &limits).unwrap()[0]).collect();
        for expected in [[0., 0., 6.], [4., 0., 6.]] {
            assert!(endpoints.iter().any(|p| (0..3).all(|i| (p[i] - expected[i]).abs() < 1e-8)));
        }
        app.controller.undo().unwrap(); assert!(!app.controller.document().entities.contains_key(&id));
        app.controller.redo().unwrap(); assert!(app.controller.document().entities.contains_key(&id));
    }
    std::fs::remove_file(path).unwrap();
}

#[test]
fn viewport_extract_wireframe_reopened_box_via_icon_and_typed_cmd() {
    let limits=Limits::default();
    let shape=om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Box {x_mm:4.,y_mm:5.,z_mm:6.},0.01,&limits).unwrap();
    let mut doc=Document::new("ExtractWireframe routes");
    let entity=Entity::new("box",Geometry::Brep(shape),None); let id=entity.id; doc.entities.insert(id,entity);
    let path=std::env::temp_dir().join(format!("om9-wireframe-ui-{}.om9",uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path,&doc,&limits,&om9_core::transaction::Cancellation::default()).unwrap();
    for menu in [true,false] {
        let mut app=MatrixApp::from_file(&path).unwrap(); let ctx=egui::Context::default();
        frame(&mut app,&ctx,vec![]); frame(&mut app,&ctx,vec![]);
        click_in_view(&mut app,&ctx,ViewId::Perspective,[2.,0.,6.],Modifiers::NONE);
        assert!(app.selected_objects().contains(&id),"viewport failed to select box");
        let before=app.controller.document().clone();
        if menu {
            let at=app.main_category_rect(0).unwrap().center();
            for pressed in [true,false] {frame(&mut app,&ctx,vec![Event::PointerMoved(at),Event::PointerButton {
                pos:at,button:PointerButton::Primary,pressed,modifiers:Modifiers::NONE}]);}
            let index=om9_app::menu_catalog::CURVE.iter().position(|name|*name=="CurveCurveFromObjectsExtractWireframe").unwrap();
            ctx.memory_mut(|memory|memory.data.insert_temp(egui::Id::new(("main-menu-page","Curve")),index/20));
            frame(&mut app,&ctx,vec![]); let at=app.rail_icon_rects[2+index%20].center();
            for pressed in [true,false] {frame(&mut app,&ctx,vec![Event::PointerMoved(at),Event::PointerButton {
                pos:at,button:PointerButton::Primary,pressed,modifiers:Modifiers::NONE}]);}
        } else {
            frame(&mut app,&ctx,vec![Event::Text("ExtractWireframe".into())]);
            frame(&mut app,&ctx,vec![Event::Key {key:egui::Key::Enter,physical_key:None,pressed:true,repeat:false,modifiers:Modifiers::NONE}]);
        }
        let deadline=std::time::Instant::now()+std::time::Duration::from_secs(30);
        while app.controller.brep_wireframe_worker_pending() {
            assert!(std::time::Instant::now()<deadline,"ExtractWireframe worker timed out");
            frame(&mut app,&ctx,vec![]);std::thread::sleep(std::time::Duration::from_millis(5));
        }
        let curves:Vec<_>=app.controller.document().entities.values().filter(|e|e.label=="ExtractWire").collect();
        assert_eq!(curves.len(),12,"menu={menu}: {}",app.transcript.history());
        let mut edges=std::collections::BTreeSet::new();
        for entity in curves {
            let Geometry::Curve(curve)=&entity.geometry else {panic!("curve")};
            let eval=|t|om9_core::evaluation::evaluate_curve(curve,om9_core::evaluation::Parameter::Fraction(t),0,entity.placement,&limits).unwrap()[0];
            let (a,b)=(eval(0.),eval(1.));
            for p in [a,b] {assert!(p.iter().zip([4.,5.,6.]).all(|(v,d)|v.abs()<1e-8||(*v-d).abs()<1e-8));}
            let corner=|p:[f64;3]|[p[0]>2.,p[1]>2.5,p[2]>3.]; let (a,b)=(corner(a),corner(b));
            assert_eq!((0..3).filter(|i|a[*i]!=b[*i]).count(),1);
            assert!(edges.insert(if a<b {(a,b)} else {(b,a)}));
        }
        assert_eq!(edges.len(),12); let after=app.controller.document().clone();
        app.controller.undo().unwrap(); assert_eq!(app.controller.document(),&before);
        app.controller.redo().unwrap(); assert_eq!(app.controller.document(),&after);
    }
    std::fs::remove_file(path).unwrap();
}

#[test]
fn viewport_dupedge_reopened_box_selects_exact_edge_and_undoes() {
    let mut original = om9_app::session::Controller::new(Document::new("reopened Box"), Limits::default()).unwrap();
    original.submit("Box 4 6 8").unwrap();
    let path = std::env::temp_dir().join(format!("om9-dupedge-ui-reopen-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, original.document(), &Limits::default(),
        &om9_core::transaction::Cancellation::default()).unwrap();
    for (pick, endpoints) in [
        ([2., 0., 8.], [[0., 0., 8.], [4., 0., 8.]]),
        ([4., 3., 8.], [[4., 0., 8.], [4., 6., 8.]]),
    ] {
        let mut app = MatrixApp::from_file(&path).unwrap();
        let ctx = egui::Context::default();
        frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
        app.controller.submit("DupEdge").unwrap();
        click_in_view(&mut app, &ctx, ViewId::Perspective, pick, Modifiers::NONE);
        assert_eq!(app.controller.subselection().len(), 1, "{}", app.transcript.history());
        app.controller.submit("").unwrap();
        let output = app.controller.document().entities.values().find(|entity| entity.label == "DupEdge").unwrap();
        let output_id = output.id;
        let Geometry::Curve(curve) = &output.geometry else { panic!("expected curve") };
        let actual: Vec<_> = [0., 1.].into_iter().map(|fraction| om9_core::evaluation::evaluate_curve(curve,
            om9_core::evaluation::Parameter::Fraction(fraction), 0, Transform::identity(), &Limits::default()).unwrap()[0]).collect();
        for expected in endpoints {
            assert!(actual.iter().any(|point| (0..3).all(|axis| (point[axis] - expected[axis]).abs() < 1e-8)));
        }
        app.controller.undo().unwrap();
        assert!(!app.controller.document().entities.contains_key(&output_id));
        app.controller.redo().unwrap();
        assert!(app.controller.document().entities.contains_key(&output_id));
        app.controller.undo().unwrap();
        app.controller.submit("Point").unwrap();
        app.controller.submit("10,10,10").unwrap();
        assert_eq!(app.controller.coordinator().redo_len(), 0);
        assert!(!app.controller.document().entities.contains_key(&output_id));
        assert!(app.controller.document().entities.values().any(|e| matches!(e.geometry, Geometry::Brep(_))));
    }
    std::fs::remove_file(path).unwrap();
}

#[test]
#[ignore = "requires external openNURBS V5 example and admitted native exchange"]
fn viewport_dupedge_external_v5_twisted_brep_after_reopen() {
    use om9_core::transaction::{Cancellation,TransactionCoordinator,World};
    use om9_exchange::{BrepMode,ImportMode,ImportOptions};use sha2::{Digest,Sha256};
    let limits=Limits::default();let executable=std::path::PathBuf::from(std::env::var_os("OM9_EXCHANGE_NATIVE").unwrap());
    let fixture=std::path::PathBuf::from(std::env::var_os("OM9_EXTERNAL_3DM_FIXTURE").unwrap());
    let hash:[u8;32]=Sha256::digest(std::fs::read(&executable).unwrap()).into();
    let host=om9_app::three_dm_host::ThreeDmHost::admit(&executable,hash).unwrap();
    let doc=Document::with_template_tolerance("external native BRep");let doc_id=doc.id;
    let mut coordinator=TransactionCoordinator::new(World::new(vec![doc],&limits).unwrap(),limits).unwrap();
    host.import_with_options(&mut coordinator,doc_id,&fixture,&ImportOptions {mode:ImportMode::Geometry,brep:BrepMode::RetainOpaque,..Default::default()},&Cancellation::default()).unwrap();
    let imported=&coordinator.world().documents[&doc_id];
    let id=imported.entities.values().find(|e|matches!(e.geometry,Geometry::NativeBrep(_))).expect("imported native BRep").id;
    let path=std::env::temp_dir().join(format!("om9-external-native-{}.om9",uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path,imported,&limits,&Cancellation::default()).unwrap();
    let mut app=MatrixApp::from_file(&path).unwrap();std::fs::remove_file(path).unwrap();
    app.controller.workspace.views.get_mut(&ViewId::Perspective).unwrap().camera=
        om9_scene::camera::Camera::orthographic([24.,-24.,20.],[5.,3.,5.],[0.,0.,1.],30.).unwrap();
    let ctx=egui::Context::default();frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);
    app.controller.submit("Point").unwrap();app.controller.submit("30,30,30").unwrap();app.controller.undo().unwrap();
    app.controller.select_objects(&[id]).unwrap();let before=app.controller.document().clone();app.controller.submit("DupEdge").unwrap();
    let deadline=std::time::Instant::now()+std::time::Duration::from_secs(20);
    while app.controller.dupedge_source_view_pending() {assert!(std::time::Instant::now()<deadline);frame(&mut app,&ctx,vec![]);std::thread::yield_now();}
    click_in_view(&mut app,&ctx,ViewId::Perspective,[5.,0.,0.],Modifiers::NONE);
    click_in_view(&mut app,&ctx,ViewId::Perspective,[10.,0.,6.],Modifiers::NONE);
    app.controller.submit("").unwrap_or_else(|e|panic!("{e:?}; {}",app.transcript.history()));
    let curves:Vec<_>=app.controller.document().entities.values().filter(|e|e.label=="DupEdge").collect();
    assert_eq!(curves.len(),2,"output {curves:?}; transcript {}",app.transcript.history());
    let mut found=[false;2];
    for entity in curves {
        let Geometry::Curve(curve)=&entity.geometry else {panic!("curve")};
        let points=[0.,0.2,0.5,0.8,1.].map(|t|om9_core::evaluation::evaluate_curve(curve,om9_core::evaluation::Parameter::Fraction(t),0,entity.placement,&limits).unwrap()[0]);
        for (p,t) in points.iter().zip([0.,0.2,0.5,0.8,1.]) {
            assert!((0..3).all(|i|(p[i]-(points[0][i]*(1.-t)+points[4][i]*t)).abs()<1e-6),"{points:?}");
        }
        let ab=points.iter().all(|p|p[1].abs()<1e-6&&p[2].abs()<1e-6&&p[0]>=-1e-6&&p[0]<=10.+1e-6);
        let bf=points.iter().all(|p|(p[0]-10.).abs()<1e-6&&p[1].abs()<1e-6&&p[2]>=-1e-6&&p[2]<=12.+1e-6);
        let endpoints=|a:[f64;3],b:[f64;3]| [a,b].iter().all(|expected|[points[0],points[4]].iter().any(|p|(0..3).all(|i|(p[i]-expected[i]).abs()<1e-6)));
        if ab && endpoints([0.,0.,0.],[10.,0.,0.]) {found[0]=true;}
        if bf && endpoints([10.,0.,0.],[10.,0.,12.]) {found[1]=true;}
        assert!(ab||bf,"unexpected edge: {points:?}");
    }
    assert_eq!(found,[true,true]);
    let after=app.controller.document().clone();app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);app.controller.redo().unwrap();assert_eq!(app.controller.document(),&after);
}

#[test]
#[ignore = "requires external openNURBS V5 example and admitted native exchange"]
fn viewport_dupedge_external_v5_ambiguous_edge_keeps_document_and_redo() {
    use om9_core::transaction::{Cancellation,TransactionCoordinator,World};
    use om9_exchange::{BrepMode,ImportMode,ImportOptions};
    use sha2::{Digest,Sha256};
    let limits=Limits::default();
    let executable=std::path::PathBuf::from(std::env::var_os("OM9_EXCHANGE_NATIVE").unwrap());
    let fixture=std::path::PathBuf::from(std::env::var_os("OM9_EXTERNAL_3DM_FIXTURE").unwrap());
    let hash:[u8;32]=Sha256::digest(std::fs::read(&executable).unwrap()).into();
    let host=om9_app::three_dm_host::ThreeDmHost::admit(&executable,hash).unwrap();
    let doc=Document::with_template_tolerance("external ambiguous edge"); let doc_id=doc.id;
    let mut coordinator=TransactionCoordinator::new(World::new(vec![doc],&limits).unwrap(),limits).unwrap();
    host.import_with_options(&mut coordinator,doc_id,&fixture,&ImportOptions {mode:ImportMode::Geometry,brep:BrepMode::RetainOpaque,..Default::default()},&Cancellation::default()).unwrap();
    let imported=&coordinator.world().documents[&doc_id];
    let id=imported.entities.values().find(|e|matches!(e.geometry,Geometry::NativeBrep(_))).unwrap().id;
    let path=std::env::temp_dir().join(format!("om9-external-ambiguous-{}.om9",uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path,imported,&limits,&Cancellation::default()).unwrap();
    let mut app=MatrixApp::from_file(&path).unwrap(); std::fs::remove_file(path).unwrap();
    app.controller.workspace.views.get_mut(&ViewId::Perspective).unwrap().camera=
        om9_scene::camera::Camera::orthographic([5.,-20.,20.],[5.,0.,0.],[0.,0.,1.],30.).unwrap();
    let ctx=egui::Context::default(); frame(&mut app,&ctx,vec![]); frame(&mut app,&ctx,vec![]);
    app.controller.submit("Point").unwrap(); app.controller.submit("30,30,30").unwrap(); app.controller.undo().unwrap();
    app.controller.select_objects(&[id]).unwrap(); let before=app.controller.document().clone();
    let redo=app.controller.coordinator().redo_len(); assert!(redo>0);
    app.controller.submit("DupEdge").unwrap();
    let deadline=std::time::Instant::now()+std::time::Duration::from_secs(20);
    while app.controller.dupedge_source_view_pending() {assert!(std::time::Instant::now()<deadline);frame(&mut app,&ctx,vec![]);std::thread::yield_now();}
    click_in_view(&mut app,&ctx,ViewId::Perspective,[5.,0.,0.],Modifiers::NONE);
    let rect=app.view_rects[&ViewId::Perspective];
    let vp=Viewport {origin:[rect.min.x as f64,rect.min.y as f64],size:[rect.width() as f64,rect.height() as f64]};
    let camera=&app.controller.workspace.views[&ViewId::Perspective].camera;
    let screen=camera.project([10.,0.,6.],vp).unwrap().unwrap().screen;
    let hits=app.controller.scene().unwrap().pick_edge_candidates(screen,camera,vp,6.,om9_scene::scene::PickIntent::Select).unwrap();
    assert_eq!(hits.len(),2,"expected two projected edges: {hits:?}");
    for expected in [[10.,0.,6.],[10.,48./7.,-6./7.]] {
        assert!(hits.iter().any(|hit|(0..3).all(|i|(hit.world_point[i]-expected[i]).abs()<1e-6)),"{hits:?}");
    }
    assert!(hits.iter().all(|hit|hit.screen_distance<1e-6));
    assert!((hits[0].depth-hits[1].depth).abs()>9.);
    click_in_view(&mut app,&ctx,ViewId::Perspective,[10.,0.,6.],Modifiers::NONE);
    assert!(app.transcript.history().contains("ambiguous edges"),"{}",app.transcript.history());
    assert_eq!(app.controller.document(),&before);
    assert_eq!(app.controller.coordinator().redo_len(),redo);
    app.controller.workspace.views.get_mut(&ViewId::Perspective).unwrap().camera=
        om9_scene::camera::Camera::orthographic([24.,-24.,20.],[5.,3.,5.],[0.,0.,1.],30.).unwrap();
    frame(&mut app,&ctx,vec![]);
    click_in_view(&mut app,&ctx,ViewId::Perspective,[10.,0.,6.],Modifiers::NONE);
    app.controller.submit("").unwrap_or_else(|e|panic!("{e:?}; {}",app.transcript.history()));
    let curves:Vec<_>=app.controller.document().entities.values().filter(|e|e.label=="DupEdge").collect();
    assert_eq!(curves.len(),2,"first selection should survive ambiguity and camera change");
    for expected in [[[0.,0.,0.],[10.,0.,0.]],[[10.,0.,0.],[10.,0.,12.]]] {
        assert!(curves.iter().any(|e| {let Geometry::Curve(curve)=&e.geometry else {return false};
            let ends=[0.,1.].map(|t|om9_core::evaluation::evaluate_curve(curve,om9_core::evaluation::Parameter::Fraction(t),0,e.placement,&limits).unwrap()[0]);
            expected.iter().all(|target|ends.iter().any(|p|(0..3).all(|i|(p[i]-target[i]).abs()<1e-6)))
        }),"missing expected edge {expected:?}");
    }
    let after=app.controller.document().clone();
    app.controller.undo().unwrap(); assert_eq!(app.controller.document(),&before);
    app.controller.redo().unwrap(); assert_eq!(app.controller.document(),&after);
}

#[test]
#[ignore = "requires admitted native exchange and authored openNURBS 3DM fixture"]
fn viewport_dupedge_opaque_imported_3dm_trimmed_arc_after_reopen() {
    use om9_core::transaction::{Cancellation,TransactionCoordinator,World};
    use om9_exchange::{BrepMode,ImportMode,ImportOptions};use sha2::{Digest,Sha256};
    let limits=Limits::default();let executable=std::path::PathBuf::from(std::env::var_os("OM9_EXCHANGE_NATIVE").unwrap());
    let fixture=std::path::PathBuf::from(std::env::var_os("OM9_DUPEDGE_3DM_ARC").unwrap());
    let hash:[u8;32]=Sha256::digest(std::fs::read(&executable).unwrap()).into();
    let host=om9_app::three_dm_host::ThreeDmHost::admit(&executable,hash).unwrap();
    let doc=Document::with_template_tolerance("native imported arc");let doc_id=doc.id;
    let mut coordinator=TransactionCoordinator::new(World::new(vec![doc],&limits).unwrap(),limits).unwrap();
    host.import_with_options(&mut coordinator,doc_id,&fixture,&ImportOptions {mode:ImportMode::Geometry,brep:BrepMode::RetainOpaque,..Default::default()},&Cancellation::default()).unwrap();
    let imported=&coordinator.world().documents[&doc_id];
    let id=imported.entities.values().find(|e|matches!(e.geometry,Geometry::NativeBrep(_))).expect("native imported face").id;
    let path=std::env::temp_dir().join(format!("om9-native-imported-arc-{}.om9",uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path,imported,&limits,&Cancellation::default()).unwrap();
    let mut app=MatrixApp::from_file(&path).unwrap();std::fs::remove_file(path).unwrap();
    app.controller.workspace.views.get_mut(&ViewId::Perspective).unwrap().camera=
        om9_scene::camera::Camera::orthographic([8.,-10.,14.],[1.,1.,2.5],[0.,0.,1.],8.).unwrap();
    let ctx=egui::Context::default();frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);
    app.controller.submit("Point").unwrap();app.controller.submit("20,20,20").unwrap();app.controller.undo().unwrap();
    app.controller.select_objects(&[id]).unwrap();let before=app.controller.document().clone();app.controller.submit("DupEdge").unwrap();
    let deadline=std::time::Instant::now()+std::time::Duration::from_secs(20);
    while app.controller.dupedge_source_view_pending() {assert!(std::time::Instant::now()<deadline);frame(&mut app,&ctx,vec![]);std::thread::yield_now();}
    let stamp=app.controller.document().stamp(id).unwrap();let display=app.controller.scene().unwrap().root_brep_pick_display(&stamp).unwrap();
    let point=display.segments.iter().filter_map(|s| {let a=display.points[s[0] as usize];let b=display.points[s[1] as usize];
        if (a[2]-5.).abs()>1e-7 || (b[2]-5.).abs()>1e-7 {return None;}
        let p=std::array::from_fn::<_,3,_>(|i|(a[i]+b[i])*0.5);Some(((p[0]-1.847759065).powi(2)+(p[1]-0.765366865).powi(2),p))
    }).min_by(|a,b|a.0.total_cmp(&b.0)).unwrap().1;
    click_in_view(&mut app,&ctx,ViewId::Perspective,point,Modifiers::NONE);
    app.controller.submit("").unwrap_or_else(|e|panic!("{e:?}; {}",app.transcript.history()));
    let entity=app.controller.document().entities.values().find(|e|e.label=="DupEdge").expect("arc");
    let Geometry::Curve(curve)=&entity.geometry else {panic!("curve")};
    let points=[0.,0.2,0.5,0.8,1.].map(|t|om9_core::evaluation::evaluate_curve(curve,om9_core::evaluation::Parameter::Fraction(t),0,entity.placement,&limits).unwrap()[0]);
    for p in points {assert!((p[0].hypot(p[1])-2.).abs()<1e-6&&(p[2]-5.).abs()<1e-6&&p[0]>=-1e-6&&p[1]>=-1e-6,"{p:?}");}
    for expected in [[2.,0.,5.],[0.,2.,5.]] {assert!([points[0],points[4]].iter().any(|p|(0..3).all(|i|(p[i]-expected[i]).abs()<1e-6)),"{points:?}");}
    for (point,expected) in [points[0],points[4]].iter().zip([[0.,2.,5.],[2.,0.,5.]]) {
        assert!((0..3).all(|i|(point[i]-expected[i]).abs()<1e-6),"source edge orientation: {points:?}");
    }
    let after=app.controller.document().clone();app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);app.controller.redo().unwrap();assert_eq!(app.controller.document(),&after);
}

#[test]
#[ignore = "requires the admitted native 3DM exchange and authored trimmed arc fixture"]
fn viewport_dupedge_reopened_imported_3dm_trimmed_arc_keeps_geometry() {
    let limits = Limits::default();
    let source = std::path::PathBuf::from(std::env::var_os("OM9_DUPEDGE_3DM_ARC").expect("OM9_DUPEDGE_3DM_ARC"));
    let mut imported = om9_app::session::Controller::new(Document::with_template_tolerance("arc import"), limits).unwrap();
    imported.import_3dm(&source).unwrap();
    assert!(imported.document().entities.values().any(|e| matches!(e.geometry, Geometry::Brep(_))), "expected editable BRep");
    let path = std::env::temp_dir().join(format!("om9-dupedge-3dm-arc-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, imported.document(), &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    let mut app = MatrixApp::from_file(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    app.controller.workspace.views.get_mut(&ViewId::Perspective).unwrap().camera =
        om9_scene::camera::Camera::orthographic([8., -10., 14.], [1., 1., 2.5], [0., 0., 1.], 8.).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
    app.controller.submit("DupEdge").unwrap();
    let shape = app.controller.document().entities.values().find_map(|e| match &e.geometry { Geometry::Brep(b) => Some(b), _ => None }).unwrap();
    let display = shape.display();
    let visible = display.segments.iter().filter_map(|s| {
        let a = display.points[s[0] as usize]; let b = display.points[s[1] as usize];
        if (a[2] - 5.).abs() > 1e-8 || (b[2] - 5.).abs() > 1e-8 { return None; }
        let p = std::array::from_fn::<_, 3, _>(|i| (a[i] + b[i]) * 0.5);
        Some(((p[0] - 1.847759065).powi(2) + (p[1] - 0.765366865).powi(2), p))
    }).min_by(|a,b| a.0.total_cmp(&b.0)).unwrap().1;
    click_in_view(&mut app, &ctx, ViewId::Perspective, visible, Modifiers::NONE);
    assert_eq!(app.controller.subselection().len(), 1, "{}", app.transcript.history());
    app.controller.submit("").unwrap();
    let output = app.controller.document().entities.values().find(|entity| entity.label == "DupEdge").unwrap();
    let id = output.id;
    let Geometry::Curve(curve) = &output.geometry else { panic!() };
    let mut endpoints = Vec::new();
    for fraction in [0., 0.2, 0.5, 0.8, 1.] {
        let p = om9_core::evaluation::evaluate_curve(curve, om9_core::evaluation::Parameter::Fraction(fraction),
            0, Transform::identity(), &limits).unwrap()[0];
        assert!((p[0].hypot(p[1]) - 2.).abs() < 1e-7 && (p[2] - 5.).abs() < 1e-7 && p[0] >= -1e-8 && p[1] >= -1e-8, "{p:?}");
        if fraction == 0. || fraction == 1. { endpoints.push(p); }
    }
    for expected in [[2., 0., 5.], [0., 2., 5.]] {
        assert!(endpoints.iter().any(|p| (0..3).all(|i| (p[i] - expected[i]).abs() < 1e-7)));
    }
    // ON_BrepEdge start/end reported by the independent fixture generator.
    for (actual, expected) in endpoints.iter().zip([[0., 2., 5.], [2., 0., 5.]]) {
        assert!((0..3).all(|i| (actual[i] - expected[i]).abs() < 1e-7), "edge orientation {endpoints:?}");
    }
    app.controller.undo().unwrap(); assert!(!app.controller.document().entities.contains_key(&id));
    app.controller.redo().unwrap(); assert!(app.controller.document().entities.contains_key(&id));
}

#[test]
#[ignore = "requires the admitted native 3DM exchange and authored cylinder fixture"]
fn viewport_dupedge_reopened_imported_3dm_cylinder_uses_source_and_undo() {
    let limits = Limits::default();
    let source = std::path::PathBuf::from(std::env::var_os("OM9_DUPEDGE_3DM_CYLINDER").expect("OM9_DUPEDGE_3DM_CYLINDER"));
    let mut imported = om9_app::session::Controller::new(Document::with_template_tolerance("cylinder import"), limits).unwrap();
    imported.import_3dm(&source).unwrap();
    assert!(imported.document().entities.values().any(|e| matches!(e.geometry, Geometry::Brep(_))), "expected editable BRep");
    let path = std::env::temp_dir().join(format!("om9-dupedge-3dm-cylinder-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, imported.document(), &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    let mut app = MatrixApp::from_file(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    app.controller.submit("Point").unwrap(); app.controller.submit("10,10,10").unwrap();
    app.controller.undo().unwrap();
    assert_eq!(app.controller.coordinator().redo_len(), 1);
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
    let before = app.controller.coordinator().snapshot();
    app.controller.submit("DupEdge").unwrap();
    click_in_view(&mut app, &ctx, ViewId::Perspective, [0., -2., 5.], Modifiers::NONE);
    assert_eq!(app.controller.subselection().len(), 1, "{}", app.transcript.history());
    app.controller.cancel();
    assert_eq!(app.controller.coordinator().snapshot(), before);
    assert_eq!(app.controller.coordinator().redo_len(), 1);
    assert!(!app.controller.is_running());
    app.controller.submit("DupEdge").unwrap();
    click_in_view(&mut app, &ctx, ViewId::Perspective, [0., -2., 5.], Modifiers::NONE);
    assert_eq!(app.controller.subselection().len(), 1, "{}", app.transcript.history());
    app.controller.submit("").unwrap();
    let output = app.controller.document().entities.values().find(|entity| entity.label == "DupEdge").unwrap();
    let id = output.id;
    let Geometry::Curve(curve) = &output.geometry else { panic!() };
    for fraction in [0., 0.2, 0.5, 0.8, 1.] {
        let p = om9_core::evaluation::evaluate_curve(curve, om9_core::evaluation::Parameter::Fraction(fraction),
            0, Transform::identity(), &limits).unwrap()[0];
        assert!((p[0].hypot(p[1]) - 2.).abs() < 1e-7 && (p[2] - 5.).abs() < 1e-7, "{p:?}");
    }
    app.controller.undo().unwrap(); assert!(!app.controller.document().entities.contains_key(&id));
    app.controller.redo().unwrap(); assert!(app.controller.document().entities.contains_key(&id));
}

#[test]
#[ignore = "requires an admitted native 3DM exchange executable and authored 3DM fixture"]
fn viewport_dupedge_reopened_imported_3dm_box_uses_source_and_undo() {
    let limits = Limits::default();
    // Authored inventory fixture: the root ON_Brep is the unit Box at the origin.
    let path_3dm = std::path::PathBuf::from(std::env::var_os("OM9_DUPEDGE_3DM_FIXTURE").expect("OM9_DUPEDGE_3DM_FIXTURE"));
    let mut imported = om9_app::session::Controller::new(Document::with_template_tolerance("import target"), limits).unwrap();
    imported.import_3dm(&path_3dm).unwrap();
    let entity = imported.document().entities.values().find(|e| matches!(e.geometry, Geometry::Brep(_)))
        .expect("the imported box must be an editable OCCT BRep");
    let mut isolated = Document::with_template_tolerance("isolated imported 3DM box");
    let isolated_entity = Entity::new("imported box", entity.geometry.clone(), isolated.current_layer);
    isolated.entities.insert(isolated_entity.id, isolated_entity);
    let path = std::env::temp_dir().join(format!("om9-dupedge-import-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, &isolated, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    let mut app = MatrixApp::from_file(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    app.controller.workspace.views.get_mut(&ViewId::Perspective).unwrap().camera =
        om9_scene::camera::Camera::orthographic([4., -5., 6.], [0.5, 0.5, 0.5], [0., 0., 1.], 2.5).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
    app.controller.submit("DupEdge").unwrap();
    click_in_view(&mut app, &ctx, ViewId::Perspective, [0.5, 0., 1.], Modifiers::NONE);
    assert_eq!(app.controller.subselection().len(), 1, "{}", app.transcript.history());
    app.controller.submit("").unwrap();
    let output = app.controller.document().entities.values().find(|entity| entity.label == "DupEdge").unwrap();
    let id = output.id;
    let Geometry::Curve(curve) = &output.geometry else { panic!() };
    let endpoints: Vec<_> = [0., 1.].into_iter().map(|fraction| om9_core::evaluation::evaluate_curve(curve,
        om9_core::evaluation::Parameter::Fraction(fraction), 0, Transform::identity(), &limits).unwrap()[0]).collect();
    for expected in [[0., 0., 1.], [1., 0., 1.]] {
        assert!(endpoints.iter().any(|p| (0..3).all(|axis| (p[axis] - expected[axis]).abs() < 1e-7)));
    }
    app.controller.undo().unwrap(); assert!(!app.controller.document().entities.contains_key(&id));
    app.controller.redo().unwrap(); assert!(app.controller.document().entities.contains_key(&id));
}

#[test]
fn viewport_dupedge_reopened_on_brep_cylinder_edge_uses_source() {
    let limits = Limits::default();
    let original = om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Cylinder { radius_mm: 2., height_mm: 5. }, 0.01, &limits).unwrap();
    let native = om9_kernel::brep::to_3dm(&original, 0.001, &limits).unwrap();
    let imported = om9_kernel::brep::from_3dm(&native.payload, 0.001, 0.01, &limits).unwrap();
    let mut document = Document::new("ON_Brep cylinder converted to OCCT");
    let entity = Entity::new("converted cylinder", Geometry::Brep(imported), document.current_layer);
    document.entities.insert(entity.id, entity);
    let path = std::env::temp_dir().join(format!("om9-dupedge-onbrep-cylinder-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, &document, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    let mut app = MatrixApp::from_file(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
    app.controller.submit("DupEdge").unwrap();
    click_in_view(&mut app, &ctx, ViewId::Perspective, [0., -2., 5.], Modifiers::NONE);
    assert_eq!(app.controller.subselection().len(), 1, "{}", app.transcript.history());
    app.controller.submit("").unwrap();
    let output = app.controller.document().entities.values().find(|entity| entity.label == "DupEdge").unwrap();
    let id = output.id;
    let Geometry::Curve(curve) = &output.geometry else { panic!() };
    for fraction in [0., 0.2, 0.5, 0.8, 1.] {
        let p = om9_core::evaluation::evaluate_curve(curve, om9_core::evaluation::Parameter::Fraction(fraction),
            0, Transform::identity(), &limits).unwrap()[0];
        assert!((p[0].hypot(p[1]) - 2.).abs() < 1e-7 && (p[2] - 5.).abs() < 1e-7, "{p:?}");
    }
    app.controller.undo().unwrap(); assert!(!app.controller.document().entities.contains_key(&id));
    app.controller.redo().unwrap(); assert!(app.controller.document().entities.contains_key(&id));
}

#[test]
fn viewport_dupedge_reopened_on_brep_payload_uses_source_and_undo() {
    let limits = Limits::default();
    let original = om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Box { x_mm: 4., y_mm: 5., z_mm: 6. }, 0.01, &limits).unwrap();
    let native = om9_kernel::brep::to_3dm(&original, 0.001, &limits).unwrap();
    let imported = om9_kernel::brep::from_3dm(&native.payload, 0.001, 0.01, &limits).unwrap();
    let mut document = Document::new("ON_Brep payload converted to OCCT");
    let entity = Entity::new("converted box", Geometry::Brep(imported), document.current_layer);
    document.entities.insert(entity.id, entity);
    let path = std::env::temp_dir().join(format!("om9-dupedge-onbrep-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, &document, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    let mut app = MatrixApp::from_file(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
    app.controller.submit("DupEdge").unwrap();
    click_in_view(&mut app, &ctx, ViewId::Perspective, [2., 0., 6.], Modifiers::NONE);
    assert_eq!(app.controller.subselection().len(), 1, "{}", app.transcript.history());
    app.controller.submit("").unwrap();
    let output = app.controller.document().entities.values().find(|entity| entity.label == "DupEdge").unwrap();
    let id = output.id;
    let Geometry::Curve(curve) = &output.geometry else { panic!() };
    let endpoints: Vec<_> = [0., 1.].into_iter().map(|fraction| om9_core::evaluation::evaluate_curve(curve,
        om9_core::evaluation::Parameter::Fraction(fraction), 0, Transform::identity(), &limits).unwrap()[0]).collect();
    for expected in [[0., 0., 6.], [4., 0., 6.]] {
        assert!(endpoints.iter().any(|p| (0..3).all(|axis| (p[axis] - expected[axis]).abs() < 1e-7)));
    }
    app.controller.undo().unwrap(); assert!(!app.controller.document().entities.contains_key(&id));
    app.controller.redo().unwrap(); assert!(app.controller.document().entities.contains_key(&id));
}

#[test]
fn viewport_dupedge_reopened_box_large_world_placement() {
    let limits = Limits::default();
    let origin = 1e9;
    let shape = om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Box { x_mm: 4., y_mm: 5., z_mm: 6. }, 0.01, &limits).unwrap();
    let mut document = Document::new("far placement");
    let mut entity = Entity::new("box", Geometry::Brep(shape), document.current_layer);
    entity.placement = Transform::translation([origin; 3]);
    document.entities.insert(entity.id, entity);
    let path = std::env::temp_dir().join(format!("om9-dupedge-far-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, &document, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    let mut app = MatrixApp::from_file(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    app.controller.workspace.views.get_mut(&ViewId::Perspective).unwrap().camera =
        om9_scene::camera::Camera::orthographic([origin + 8., origin - 10., origin + 14.],
            [origin + 2., origin + 2.5, origin + 3.], [0., 0., 1.], 12.).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
    app.controller.submit("DupEdge").unwrap();
    click_in_view(&mut app, &ctx, ViewId::Perspective, [origin + 2., origin, origin + 6.], Modifiers::NONE);
    assert_eq!(app.controller.subselection().len(), 1, "{}", app.transcript.history());
    app.controller.submit("").unwrap();
    let output = app.controller.document().entities.values().find(|entity| entity.label == "DupEdge").unwrap();
    let Geometry::Curve(curve) = &output.geometry else { panic!() };
    let middle = om9_core::evaluation::evaluate_curve(curve, om9_core::evaluation::Parameter::Fraction(0.5),
        0, Transform::identity(), &limits).unwrap()[0];
    assert!((middle[0] - origin - 2.).abs() < 0.001 && (middle[1] - origin).abs() < 0.001 && (middle[2] - origin - 6.).abs() < 0.001);
}

#[test]
fn viewport_dupedge_shaded_refuses_edge_hidden_by_another_solid() {
    use om9_scene::workspace::DisplayMode;
    let limits = Limits::default();
    let back = om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Box { x_mm: 4., y_mm: 5., z_mm: 6. }, 0.01, &limits).unwrap();
    let front = om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Box { x_mm: 6., y_mm: 4., z_mm: 2. }, 0.01, &limits).unwrap();
    let mut document = Document::with_template_tolerance("occluded edge");
    let back = Entity::new("back", Geometry::Brep(back), document.current_layer);
    let back_id = back.id;
    let mut front = Entity::new("front", Geometry::Brep(front), document.current_layer);
    front.placement = Transform::translation([-1., -2., 8.]);
    document.entities.insert(back.id, back);
    document.entities.insert(front.id, front);
    let path = std::env::temp_dir().join(format!("om9-dupedge-occlusion-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, &document, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    let mut app = MatrixApp::from_file(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    let view = app.controller.workspace.views.get_mut(&ViewId::LookingDown).unwrap();
    view.mode = DisplayMode::Shaded;
    view.camera = om9_scene::camera::Camera::orthographic([2., -10., 50.], [2., 0., 5.], [0., 1., 0.], 10.).unwrap();
    app.controller.submit("Point").unwrap(); app.controller.submit("20,20,20").unwrap();
    app.controller.undo().unwrap();
    assert_eq!(app.controller.coordinator().redo_len(), 1);
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
    let rect = app.view_rects[&ViewId::LookingDown];
    let viewport = Viewport { origin: [rect.min.x as f64, rect.min.y as f64], size: [rect.width() as f64, rect.height() as f64] };
    let camera = &app.controller.workspace.views[&ViewId::LookingDown].camera;
    let cursor = camera.project([2., 0., 6.], viewport).unwrap().unwrap().screen;
    let candidates = app.controller.scene().unwrap().pick_edge_candidates(cursor, camera, viewport, 6., om9_scene::scene::PickIntent::Select).unwrap();
    assert_eq!(candidates.len(), 1, "fixture needs exactly one edge candidate: {candidates:?}");
    assert_eq!(candidates[0].root, back_id);
    let before = app.controller.coordinator().snapshot();
    app.controller.submit("DupEdge").unwrap();
    click_in_view(&mut app, &ctx, ViewId::LookingDown, [2., 0., 6.], Modifiers::NONE);
    assert!(app.controller.subselection().is_empty(), "a shaded solid must occlude the edge");
    assert!(app.controller.dupedge_needs_pick());
    assert_eq!(app.controller.coordinator().snapshot(), before);
    assert_eq!(app.controller.coordinator().redo_len(), 1);
    assert!(app.transcript.history().contains("hidden by shaded geometry"), "{}", app.transcript.history());
    app.controller.cancel();
    app.set_sub_object_kind(om9_scene::scene::PickKind::Edge);
    let modifiers = Modifiers { ctrl: true, shift: true, ..Modifiers::NONE };
    click_in_view(&mut app, &ctx, ViewId::LookingDown, [2., 0., 6.], modifiers);
    assert!(app.controller.subselection().is_empty(), "preselection must also reject the hidden edge");
    assert_eq!(app.controller.coordinator().snapshot(), before);
    assert_eq!(app.controller.coordinator().redo_len(), 1);
    app.controller.workspace.views.get_mut(&ViewId::LookingDown).unwrap().mode = DisplayMode::Wireframe;
    app.controller.submit("DupEdge").unwrap();
    click_in_view(&mut app, &ctx, ViewId::LookingDown, [2., 0., 6.], Modifiers::NONE);
    assert_eq!(app.controller.subselection().len(), 1, "wireframe exposes the edge");
    app.controller.submit("").unwrap();
    assert_eq!(app.controller.document().entities.values().filter(|entity| entity.label == "DupEdge").count(), 1);
}

#[test]
fn viewport_dupedge_shaded_accepts_visible_box_edge() {
    use om9_scene::workspace::DisplayMode;
    let limits = Limits::default();
    let shape = om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Box { x_mm: 4., y_mm: 5., z_mm: 6. }, 0.01, &limits).unwrap();
    let mut document = Document::with_template_tolerance("visible shaded edge");
    let entity = Entity::new("box", Geometry::Brep(shape), document.current_layer);
    document.entities.insert(entity.id, entity);
    let mut app = MatrixApp::new(document).unwrap();
    let view = app.controller.workspace.views.get_mut(&ViewId::LookingDown).unwrap();
    view.mode = DisplayMode::Shaded;
    view.camera = om9_scene::camera::Camera::orthographic([2., -10., 50.], [2., 0., 5.], [0., 1., 0.], 10.).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
    app.controller.submit("DupEdge").unwrap();
    click_in_view(&mut app, &ctx, ViewId::LookingDown, [2., 0., 6.], Modifiers::NONE);
    assert_eq!(app.controller.subselection().len(), 1, "{}", app.transcript.history());
    app.controller.submit("").unwrap();
    assert_eq!(app.controller.document().entities.values().filter(|entity| entity.label == "DupEdge").count(), 1);
}

#[test]
fn viewport_dupedge_shaded_refuses_self_occluded_box_edge() {
    use om9_scene::workspace::DisplayMode;
    let limits = Limits::default();
    let shape = om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Box { x_mm: 4., y_mm: 5., z_mm: 6. }, 0.01, &limits).unwrap();
    let mut document = Document::with_template_tolerance("self occluded edge");
    let entity = Entity::new("box", Geometry::Brep(shape), document.current_layer);
    document.entities.insert(entity.id, entity);
    let mut app = MatrixApp::new(document).unwrap();
    let view = app.controller.workspace.views.get_mut(&ViewId::LookingDown).unwrap();
    view.mode = DisplayMode::Shaded;
    view.camera = om9_scene::camera::Camera::orthographic([2., -10., 50.], [2., 0., 5.], [0., 1., 0.], 10.).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
    let rect = app.view_rects[&ViewId::LookingDown];
    let viewport = Viewport { origin: [rect.min.x as f64, rect.min.y as f64], size: [rect.width() as f64, rect.height() as f64] };
    let camera = &app.controller.workspace.views[&ViewId::LookingDown].camera;
    let cursor = camera.project([2., 5., 0.], viewport).unwrap().unwrap().screen;
    let candidates = app.controller.scene().unwrap().pick_edge_candidates(cursor, camera, viewport, 6., om9_scene::scene::PickIntent::Select).unwrap();
    assert_eq!(candidates.len(), 1, "fixture needs one edge candidate: {candidates:?}");
    app.controller.submit("DupEdge").unwrap();
    click_in_view(&mut app, &ctx, ViewId::LookingDown, [2., 5., 0.], Modifiers::NONE);
    assert!(app.controller.subselection().is_empty());
    assert!(app.transcript.history().contains("hidden by shaded geometry"), "{}", app.transcript.history());
}

#[test]
fn viewport_dupedge_reopened_rotated_box_uses_current_layer_and_undo() {
    use om9_app::layer_command::LayerChoice;
    use om9_core::layers::PresetSlot;
    let limits = Limits::default();
    let shape = om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Box { x_mm: 4., y_mm: 6., z_mm: 8. }, 0.01, &limits).unwrap();
    let mut document = Document::new("rotated reopened box");
    let mut entity = Entity::new("box", Geometry::Brep(shape), document.current_layer);
    entity.placement.0[0] = [0., -1., 0., 10.];
    entity.placement.0[1] = [1., 0., 0., -4.];
    entity.placement.0[2][3] = 3.;
    document.entities.insert(entity.id, entity);
    let path = std::env::temp_dir().join(format!("om9-dupedge-placed-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, &document, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    let mut app = MatrixApp::from_file(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    app.controller.set_current_layer(LayerChoice::Slot(PresetSlot::new(5).unwrap())).unwrap();
    let layer = app.controller.document().current_layer;
    assert!(layer.is_some());
    app.controller.workspace.views.get_mut(&ViewId::Perspective).unwrap().camera =
        om9_scene::camera::Camera::orthographic([18., -14., 20.], [7., -2., 7.], [0., 0., 1.], 14.).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
    app.controller.dispatch("OM9-CURVE-040", InvocationOrigin::Menu).unwrap();
    click_in_view(&mut app, &ctx, ViewId::Perspective, [10., -2., 11.], Modifiers::NONE);
    assert_eq!(app.controller.subselection().len(), 1, "{}", app.transcript.history());
    app.controller.submit("").unwrap();
    let output = app.controller.document().entities.values().find(|entity| entity.label == "DupEdge").unwrap();
    assert_eq!(output.attributes.layer, layer);
    let id = output.id;
    let Geometry::Curve(curve) = &output.geometry else { panic!() };
    let actual: Vec<_> = [0., 1.].into_iter().map(|fraction| om9_core::evaluation::evaluate_curve(curve,
        om9_core::evaluation::Parameter::Fraction(fraction), 0, Transform::identity(), &limits).unwrap()[0]).collect();
    for expected in [[10., -4., 11.], [10., 0., 11.]] {
        assert!(actual.iter().any(|point| (0..3).all(|axis| (point[axis] - expected[axis]).abs() < 1e-8)));
    }
    app.controller.undo().unwrap(); assert!(!app.controller.document().entities.contains_key(&id));
    app.controller.redo().unwrap();
    assert_eq!(app.controller.document().entities[&id].attributes.layer, layer);
}

#[test]
fn viewport_dupedge_reopened_box_commits_two_edges_in_one_undo() {
    let mut original = om9_app::session::Controller::new(Document::new("reopened Box"), Limits::default()).unwrap();
    original.submit("Box 4 6 8").unwrap();
    let path = std::env::temp_dir().join(format!("om9-dupedge-ui-many-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, original.document(), &Limits::default(),
        &om9_core::transaction::Cancellation::default()).unwrap();
    let mut app = MatrixApp::from_file(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
    app.controller.submit("DupEdge").unwrap();
    click_in_view(&mut app, &ctx, ViewId::Perspective, [2., 0., 8.], Modifiers::NONE);
    click_in_view(&mut app, &ctx, ViewId::Perspective, [4., 3., 8.], Modifiers::NONE);
    assert_eq!(app.controller.subselection().len(), 2, "{}", app.transcript.history());
    click_in_view(&mut app, &ctx, ViewId::Perspective, [2., 0., 8.], Modifiers::NONE);
    assert_eq!(app.controller.subselection().len(), 1);
    click_in_view(&mut app, &ctx, ViewId::Perspective, [2., 0., 8.], Modifiers::NONE);
    assert_eq!(app.controller.subselection().len(), 2);
    app.controller.submit("").unwrap();
    let curves: Vec<_> = app.controller.document().entities.values().filter(|entity| entity.label == "DupEdge").collect();
    assert_eq!(curves.len(), 2);
    let ids: Vec<_> = curves.iter().map(|entity| entity.id).collect();
    for endpoints in [[[0., 0., 8.], [4., 0., 8.]], [[4., 0., 8.], [4., 6., 8.]]] {
        assert!(curves.iter().any(|entity| {
            let Geometry::Curve(curve) = &entity.geometry else { return false };
            let actual: Vec<_> = [0., 1.].into_iter().map(|fraction| om9_core::evaluation::evaluate_curve(curve,
                om9_core::evaluation::Parameter::Fraction(fraction), 0, Transform::identity(), &Limits::default()).unwrap()[0]).collect();
            endpoints.iter().all(|expected| actual.iter().any(|point| (0..3).all(|axis| (point[axis] - expected[axis]).abs() < 1e-8)))
        }));
    }
    app.controller.undo().unwrap();
    assert!(ids.iter().all(|id| !app.controller.document().entities.contains_key(id)));
    app.controller.redo().unwrap();
    assert!(ids.iter().all(|id| app.controller.document().entities.contains_key(id)));
}

#[test]
fn reopened_native_box_dupedge_two_edges_are_one_undo() {
    let limits=Limits::default();let shape=om9_kernel::brep::box_brep([4.,6.,8.],0.1,&limits).unwrap();
    let native=om9_kernel::brep::to_3dm(&shape,1e-6,&limits).unwrap();
    let mut doc=Document::with_template_tolerance("native two edges");
    let entity=Entity::new("box",Geometry::NativeBrep(native),doc.current_layer);let id=entity.id;doc.entities.insert(id,entity);
    let path=std::env::temp_dir().join(format!("om9-native-two-edges-{}.om9",uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path,&doc,&limits,&om9_core::transaction::Cancellation::default()).unwrap();
    let mut app=MatrixApp::from_file(&path).unwrap();std::fs::remove_file(path).unwrap();
    let ctx=egui::Context::default();frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);
    app.controller.submit("Point").unwrap();app.controller.submit("20,20,20").unwrap();app.controller.undo().unwrap();
    app.controller.select_objects(&[id]).unwrap();let before=app.controller.document().clone();app.controller.submit("DupEdge").unwrap();
    let deadline=std::time::Instant::now()+std::time::Duration::from_secs(20);
    while app.controller.dupedge_source_view_pending() {assert!(std::time::Instant::now()<deadline);frame(&mut app,&ctx,vec![]);std::thread::yield_now();}
    for point in [[2.,0.,8.],[4.,3.,8.],[2.,0.,8.],[2.,0.,8.]] {click_in_view(&mut app,&ctx,ViewId::Perspective,point,Modifiers::NONE);}
    app.controller.submit("").unwrap_or_else(|e|panic!("{e:?}; {}",app.transcript.history()));
    let curves:Vec<_>=app.controller.document().entities.values().filter(|e|e.label=="DupEdge").collect();assert_eq!(curves.len(),2);
    for expected in [[[0.,0.,8.],[4.,0.,8.]],[[4.,0.,8.],[4.,6.,8.]]] {
        assert!(curves.iter().any(|e| {let Geometry::Curve(curve)=&e.geometry else {return false};
            let ends=[0.,1.].map(|t|om9_core::evaluation::evaluate_curve(curve,om9_core::evaluation::Parameter::Fraction(t),0,e.placement,&limits).unwrap()[0]);
            expected.iter().all(|target|ends.iter().any(|p|(0..3).all(|i|(p[i]-target[i]).abs()<1e-6)))
        }),"missing expected edge {expected:?}");
    }
    let after=app.controller.document().clone();app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);app.controller.redo().unwrap();assert_eq!(app.controller.document(),&after);
}

#[test]
fn reopened_native_box_dupedge_selects_source_and_undoes() {
    let limits=Limits::default();
    let shape=om9_kernel::brep::box_brep([4.,5.,6.],0.1,&limits).unwrap();
    let native=om9_kernel::brep::to_3dm(&shape,1e-6,&limits).unwrap();
    let mut doc=Document::with_template_tolerance("native DupEdge");
    let mut entity=Entity::new("box",Geometry::NativeBrep(native),doc.current_layer);
    entity.placement=Transform([[2.,0.5,0.,10.],[0.,1.,0.,-3.],[0.,0.,1.,2.],[0.,0.,0.,1.]]);
    let id=entity.id;doc.entities.insert(id,entity);
    let path=std::env::temp_dir().join(format!("om9-native-dupedge-{}.om9",uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path,&doc,&limits,&om9_core::transaction::Cancellation::default()).unwrap();
    for preselected in [false,true] {
    let mut app=MatrixApp::from_file(&path).unwrap();
    app.controller.workspace.views.get_mut(&ViewId::LookingDown).unwrap().camera=
        om9_scene::camera::Camera::orthographic([14.,-13.,52.],[14.,-3.,7.],[0.,1.,0.],14.).unwrap();
    let ctx=egui::Context::default();frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);
    app.controller.submit("Point").unwrap();app.controller.submit("50,50,50").unwrap();app.controller.undo().unwrap();
    if preselected {app.controller.select_objects(&[id]).unwrap();} let before=app.controller.document().clone();
    if preselected {app.controller.dispatch("OM9-CURVE-040",om9_app::session::InvocationOrigin::Menu).unwrap();} else {app.controller.submit("DupEdge").unwrap();}
    if !preselected {click_in_view(&mut app,&ctx,ViewId::LookingDown,[14.,-3.,8.],Modifiers::NONE);}
    assert!(app.controller.dupedge_source_view_pending(),"{}",app.transcript.history());
    let deadline=std::time::Instant::now()+std::time::Duration::from_secs(20);
    while app.controller.dupedge_source_view_pending() {assert!(std::time::Instant::now()<deadline);frame(&mut app,&ctx,vec![]);std::thread::yield_now();}
    click_in_view(&mut app,&ctx,ViewId::LookingDown,[14.,-3.,8.],Modifiers::NONE);
    app.controller.submit("").unwrap_or_else(|e|panic!("{e:?}; {}",app.transcript.history()));
    let entity=app.controller.document().entities.values().find(|e|e.label=="DupEdge").expect("curve");
    let Geometry::Curve(curve)=&entity.geometry else {panic!("curve")};
    let ends=[0.,1.].map(|t|om9_core::evaluation::evaluate_curve(curve,om9_core::evaluation::Parameter::Fraction(t),0,entity.placement,&limits).unwrap()[0]);
    for expected in [[10.,-3.,8.],[18.,-3.,8.]] {assert!(ends.iter().any(|p|(0..3).all(|i|(p[i]-expected[i]).abs()<1e-6)),"{ends:?}");}
    let after=app.controller.document().clone();app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);app.controller.redo().unwrap();assert_eq!(app.controller.document(),&after);
    }
    std::fs::remove_file(path).unwrap();
}

#[test]
fn reopened_native_box_dupedge_initial_pick_starts_source_view() {
    let limits=Limits::default();
    let shape=om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Box {x_mm:4.,y_mm:5.,z_mm:6.},0.01,&limits).unwrap();
    let native=om9_kernel::brep::to_3dm(&shape,1e-6,&limits).unwrap();
    for geometry in [Geometry::Brep(shape),Geometry::NativeBrep(native)] {
        let is_native=matches!(geometry,Geometry::NativeBrep(_));
        let mut doc=Document::with_template_tolerance("DupEdge native comparison");
        let entity=Entity::new("box",geometry,doc.current_layer);doc.entities.insert(entity.id,entity);
        let path=std::env::temp_dir().join(format!("om9-dupedge-native-{}.om9",uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&path,&doc,&limits,&om9_core::transaction::Cancellation::default()).unwrap();
        let mut app=MatrixApp::from_file(&path).unwrap();std::fs::remove_file(path).unwrap();
        let view=app.controller.workspace.views.get_mut(&ViewId::LookingDown).unwrap();
        view.camera=om9_scene::camera::Camera::orthographic([2.,-10.,50.],[2.,0.,5.],[0.,1.,0.],10.).unwrap();
        let ctx=egui::Context::default();frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);
        app.controller.submit("Point").unwrap();app.controller.submit("50,50,50").unwrap();app.controller.undo().unwrap();
        let before=app.controller.document().clone();app.controller.submit("DupEdge").unwrap();
        click_in_view(&mut app,&ctx,ViewId::LookingDown,[2.,0.,6.],Modifiers::NONE);
        if is_native {
            assert!(app.controller.subselection().is_empty(),"first click prepares the source view");
            assert!(app.controller.dupedge_needs_pick());
            assert!(app.controller.dupedge_source_view_pending());
            assert_eq!(app.controller.document(),&before);assert_eq!(app.controller.coordinator().redo_len(),1);
        } else {
            assert_eq!(app.controller.subselection().len(),1,"{}",app.transcript.history());
            app.controller.submit("").unwrap();
            assert_eq!(app.controller.document().entities.values().filter(|e|e.label=="DupEdge").count(),1);
        }
    }
}

#[test]
fn viewport_dupedge_reopened_sphere_selects_seam() {
    let limits = Limits::default();
    let shape = om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Sphere { radius_mm: 5. }, 0.01, &limits).unwrap();
    let mut document = Document::new("reopened sphere");
    let entity = Entity::new("Sphere", Geometry::Brep(shape), document.current_layer);
    document.entities.insert(entity.id, entity);
    let path = std::env::temp_dir().join(format!("om9-dupedge-sphere-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, &document, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    let mut app = MatrixApp::from_file(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
    app.controller.submit("DupEdge").unwrap();
    click_in_view(&mut app, &ctx, ViewId::Perspective, [5., 0., 0.], Modifiers::NONE);
    assert_eq!(app.controller.subselection().len(), 1, "{}", app.transcript.history());
    app.controller.submit("").unwrap();
    let output = app.controller.document().entities.values().find(|entity| entity.label == "DupEdge").unwrap();
    let id = output.id;
    let Geometry::Curve(curve) = &output.geometry else { panic!("expected curve") };
    for fraction in [0., 0.25, 0.5, 0.75, 1.] {
        let p = om9_core::evaluation::evaluate_curve(curve, om9_core::evaluation::Parameter::Fraction(fraction),
            0, Transform::identity(), &limits).unwrap()[0];
        assert!((p.iter().map(|value| value * value).sum::<f64>().sqrt() - 5.).abs() < 1e-7);
    }
    app.controller.undo().unwrap(); assert!(!app.controller.document().entities.contains_key(&id));
    app.controller.redo().unwrap(); assert!(app.controller.document().entities.contains_key(&id));
}

#[test]
fn viewport_dupedge_reopened_trimmed_arc_keeps_source_geometry() {
    let limits = Limits::default();
    for profile_deflection in [0.5, 0.01] {
    let arc = om9_core::curve_build::arc_through([2., 0., 0.], [2f64.sqrt(), 2f64.sqrt(), 0.], [0., 2., 0.]).unwrap();
    let profile = Curve::Polycurve { segments: vec![arc,
        Curve::Line { start: [0., 2., 0.], end: [2., 0., 0.], domain: [0., 1.] }], reversed: false };
    let shape = om9_kernel::extrusion::shell(&profile, [0., 0., 5.], profile_deflection, &limits).unwrap();
    let mut document = Document::new("reopened arc shell");
    if profile_deflection == 0.5 {
        document.tolerance = Some(om9_core::history::Tolerance::try_new(1e-7, 0., 0.).unwrap());
    }
    let entity = Entity::new("shell", Geometry::Brep(shape), document.current_layer);
    document.entities.insert(entity.id, entity);
    let path = std::env::temp_dir().join(format!("om9-dupedge-arc-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, &document, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    let mut app = MatrixApp::from_file(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    app.controller.workspace.views.get_mut(&ViewId::Perspective).unwrap().camera =
        om9_scene::camera::Camera::orthographic([8., -10., 14.], [1., 1., 2.5], [0., 0., 1.], 8.).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
    app.controller.submit("DupEdge").unwrap();
    let Geometry::Brep(shape) = &app.controller.document().entities.values().next().unwrap().geometry else { panic!() };
    let display = shape.display();
    let visible_point = display.segments.iter().filter_map(|segment| {
        let a = display.points[segment[0] as usize];
        let b = display.points[segment[1] as usize];
        if (a[2] - 5.).abs() > 1e-8 || (b[2] - 5.).abs() > 1e-8 { return None; }
        let midpoint = std::array::from_fn::<_, 3, _>(|axis| (a[axis] + b[axis]) * 0.5);
        let distance = (midpoint[0] - 1.847759065).powi(2) + (midpoint[1] - 0.765366865).powi(2);
        Some((distance, midpoint))
    }).min_by(|a, b| a.0.total_cmp(&b.0)).unwrap().1;
    click_in_view(&mut app, &ctx, ViewId::Perspective, visible_point, Modifiers::NONE);
    assert_eq!(app.controller.subselection().len(), 1, "{}", app.transcript.history());
    app.controller.submit("").unwrap();
    if profile_deflection==0.5 {assert!(app.controller.dupedge_source_view_pending(),"coarse view must be refreshed");}
    if app.controller.dupedge_source_view_pending() {
        let deadline=std::time::Instant::now()+std::time::Duration::from_secs(20);
        while app.controller.dupedge_source_view_pending() {assert!(std::time::Instant::now()<deadline);frame(&mut app,&ctx,vec![]);std::thread::yield_now();}
        let root=app.controller.document().entities.values().find(|e|matches!(e.geometry,Geometry::Brep(_))).unwrap().id;
        let stamp=app.controller.document().stamp(root).unwrap();let display=app.controller.scene().unwrap().root_brep_pick_display(&stamp).unwrap();
        let point=display.segments.iter().filter_map(|s| {let a=display.points[s[0] as usize];let b=display.points[s[1] as usize];
            if (a[2]-5.).abs()>1e-8||(b[2]-5.).abs()>1e-8{return None;}
            let p=std::array::from_fn::<_,3,_>(|i|(a[i]+b[i])*0.5);Some(((p[0]-1.847759065).powi(2)+(p[1]-0.765366865).powi(2),p))
        }).min_by(|a,b|a.0.total_cmp(&b.0)).unwrap().1;
        click_in_view(&mut app,&ctx,ViewId::Perspective,point,Modifiers::NONE);
        app.controller.submit("").unwrap_or_else(|e|panic!("{e:?}; {}",app.transcript.history()));
    }
    let output = app.controller.document().entities.values().find(|entity| entity.label == "DupEdge").unwrap();
    let id = output.id;
    let Geometry::Curve(curve) = &output.geometry else { panic!("expected curve") };
    let mut endpoints = Vec::new();
    for fraction in [0., 0.2, 0.5, 0.8, 1.] {
        let p = om9_core::evaluation::evaluate_curve(curve, om9_core::evaluation::Parameter::Fraction(fraction),
            0, Transform::identity(), &limits).unwrap()[0];
        assert!((p[0].hypot(p[1]) - 2.).abs() < 1e-7 && (p[2] - 5.).abs() < 1e-7 && p[0] >= -1e-8 && p[1] >= -1e-8, "{p:?}");
        if fraction == 0. || fraction == 1. { endpoints.push(p); }
    }
    for expected in [[2., 0., 5.], [0., 2., 5.]] {
        assert!(endpoints.iter().any(|point| (0..3).all(|axis| (point[axis] - expected[axis]).abs() < 1e-7)));
    }
    app.controller.undo().unwrap(); assert!(!app.controller.document().entities.contains_key(&id));
    app.controller.redo().unwrap(); assert!(app.controller.document().entities.contains_key(&id));
    let output_path = std::env::temp_dir().join(format!("om9-dupedge-arc-output-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&output_path, app.controller.document(), &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    let reopened = MatrixApp::from_file(&output_path).unwrap();
    std::fs::remove_file(output_path).unwrap();
    let Geometry::Curve(curve) = &reopened.controller.document().entities[&id].geometry else { panic!("expected saved curve") };
    let p = om9_core::evaluation::evaluate_curve(curve, om9_core::evaluation::Parameter::Fraction(0.5),
        0, Transform::identity(), &limits).unwrap()[0];
    assert!((p[0].hypot(p[1]) - 2.).abs() < 1e-7 && (p[2] - 5.).abs() < 1e-7);
    }
}

#[test]
fn viewport_dupedge_reopened_misleading_display_chord_refuses() {
    let limits = Limits::default();
    let shape = om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Cylinder { radius_mm: 2., height_mm: 5. }, 0.01, &limits).unwrap();
    let mut display = shape.display().clone();
    let edge = display.segments.iter().find(|s| {
        let a = display.points[s[0] as usize]; let b = display.points[s[1] as usize];
        (a[2] - 5.).abs() < 1e-8 && (b[2] - 5.).abs() < 1e-8
    }).unwrap()[2];
    let first = u32::try_from(display.points.len()).unwrap();
    display.points.push([2., 0., 5.]); display.points.push([0., 2., 5.]);
    display.segments.push([first, first + 1, edge]);
    let altered = om9_core::brep::Brep::from_native(shape.source().to_owned(), display, &limits).unwrap();
    let mut document = Document::new("misleading cylinder chord");
    let entity = Entity::new("cylinder", Geometry::Brep(altered), document.current_layer);
    document.entities.insert(entity.id, entity);
    let path = std::env::temp_dir().join(format!("om9-dupedge-fake-chord-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, &document, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    let mut app = MatrixApp::from_file(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    app.controller.submit("Point").unwrap(); app.controller.submit("10,10,10").unwrap(); app.controller.undo().unwrap();
    app.controller.workspace.views.get_mut(&ViewId::Perspective).unwrap().camera =
        om9_scene::camera::Camera::orthographic([8., -10., 14.], [1., 1., 2.5], [0., 0., 1.], 40.).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
    app.controller.submit("DupEdge").unwrap();
    click_in_view(&mut app, &ctx, ViewId::Perspective, [1., 1., 5.], Modifiers::NONE);
    assert_eq!(app.controller.subselection().len(), 1, "{}", app.transcript.history());
    let before = app.controller.coordinator().snapshot();
    let at = app.view_rects[&ViewId::Perspective].center();
    for pressed in [true, false] {
        frame(&mut app, &ctx, vec![Event::PointerMoved(at), Event::PointerButton { pos: at,
            button: PointerButton::Secondary, pressed, modifiers: Modifiers::NONE }]);
    }
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    while app.controller.dupedge_source_view_pending() {
        assert!(std::time::Instant::now() < deadline);
        frame(&mut app, &ctx, vec![]);
        std::thread::yield_now();
    }
    assert!(app.transcript.history().contains("source view ready; select the edge again"),
        "{}", app.transcript.history());
    assert_eq!(app.controller.coordinator().snapshot(), before);
    assert_eq!(app.controller.coordinator().redo_len(), 1);
    assert!(app.controller.dupedge_needs_pick());
}

#[test]
fn viewport_dupedge_coarse_affine_curve_rebuilds_then_reselects() {
    let limits=Limits::default();
    let source=om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Cylinder{radius_mm:2.,height_mm:5.},0.01,&limits).unwrap();
    let shape=om9_kernel::brep::read_brep(source.source(),0.5,&limits).unwrap();
    let placement=Transform([[2.,0.5,0.,10.],[0.,1.,0.,-2.],[0.,0.,1.,1.],[0.,0.,0.,1.]]);
    let s=shape.display().segments.iter().find(|s| {
        let a=shape.display().points[s[0] as usize]; let b=shape.display().points[s[1] as usize];
        (a[2]-5.).abs()<1e-8 && (b[2]-5.).abs()<1e-8
    }).unwrap();
    let a=shape.display().points[s[0] as usize]; let b=shape.display().points[s[1] as usize];
    let midpoint=placement.point(std::array::from_fn(|i|(a[i]+b[i])*0.5)).unwrap();
    let mut document=Document::new("coarse affine recovery");
    let mut entity=Entity::new("cylinder",Geometry::Brep(shape),document.current_layer); entity.placement=placement; let id=entity.id;
    document.entities.insert(id,entity);
    let path=std::env::temp_dir().join(format!("om9-dupedge-affine-recovery-{}.om9",uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path,&document,&limits,&om9_core::transaction::Cancellation::default()).unwrap();
    let mut app=MatrixApp::from_file(&path).unwrap(); std::fs::remove_file(path).unwrap();
    app.controller.submit("Point").unwrap(); app.controller.submit("20,20,20").unwrap(); app.controller.undo().unwrap();
    app.controller.workspace.views.get_mut(&ViewId::Perspective).unwrap().camera=
        om9_scene::camera::Camera::orthographic([20.,-15.,20.],[10.,-2.,3.],[0.,0.,1.],12.).unwrap();
    let ctx=egui::Context::default(); frame(&mut app,&ctx,vec![]); frame(&mut app,&ctx,vec![]);
    app.controller.submit("DupEdge").unwrap(); click_in_view(&mut app,&ctx,ViewId::Perspective,midpoint,Modifiers::NONE);
    assert_eq!(app.controller.subselection().len(),1,"{}",app.transcript.history());
    let before=app.controller.coordinator().snapshot(); app.controller.submit("").unwrap();
    assert!(app.controller.dupedge_source_view_pending());
    let deadline=std::time::Instant::now()+std::time::Duration::from_secs(20);
    while app.controller.dupedge_source_view_pending() {
        assert!(std::time::Instant::now()<deadline); frame(&mut app,&ctx,vec![]); std::thread::yield_now();
    }
    assert!(app.transcript.history().contains("source view ready; select the edge again"),"{}",app.transcript.history());
    assert_eq!(app.controller.coordinator().snapshot(),before); assert_eq!(app.controller.coordinator().redo_len(),1);
    let stamp=app.controller.document().stamp(id).unwrap();
    let view=app.controller.scene().unwrap().root_brep_pick_display(&stamp).unwrap();
    let s=view.segments.iter().filter(|s| {
        let p=view.points[s[0] as usize]; let q=view.points[s[1] as usize];
        (p[2]-5.).abs()<1e-8 && (q[2]-5.).abs()<1e-8
    }).min_by(|x,y| {
        let distance=|s: &&[u32;3]| {let p=view.points[s[0] as usize]; let q=view.points[s[1] as usize];
            let w=placement.point(std::array::from_fn(|i|(p[i]+q[i])*0.5)).unwrap();
            (0..3).map(|i|(w[i]-midpoint[i]).powi(2)).sum::<f64>()};
        distance(x).total_cmp(&distance(y))
    }).unwrap();
    let p=view.points[s[0] as usize]; let q=view.points[s[1] as usize];
    let retry=placement.point(std::array::from_fn(|i|(p[i]+q[i])*0.5)).unwrap();
    click_in_view(&mut app,&ctx,ViewId::Perspective,retry,Modifiers::NONE);
    app.controller.submit("").unwrap();
    let output=app.controller.document().entities.values().find(|e|e.label=="DupEdge").expect("duplicated curve");
    assert_eq!(output.placement,placement); let Geometry::Curve(curve)=&output.geometry else {panic!("curve")};
    for t in [0.,0.2,0.5,0.8,1.] {
        let point=om9_core::evaluation::evaluate_curve(curve,om9_core::evaluation::Parameter::Fraction(t),0,output.placement,&limits).unwrap()[0];
        let y=point[1]+2.; let x=(point[0]-10.-0.5*y)/2.; assert!((x*x+y*y-4.).abs()<1e-7 && (point[2]-6.).abs()<1e-8);
    }
    let after=app.controller.document().entities.clone(); app.controller.undo().unwrap();
    assert_eq!(app.controller.document().entities.len(),1); app.controller.redo().unwrap(); assert_eq!(app.controller.document().entities,after);
}

#[test]
fn viewport_dupedge_reopened_modified_arc_display_refuses_and_preserves_redo() {
    let limits = Limits::default();
    for deflection in [0.5, 0.01] {
    for shift in [0.01, 0.0000005] {
    let arc = om9_core::curve_build::arc_through([2., 0., 0.], [2f64.sqrt(), 2f64.sqrt(), 0.], [0., 2., 0.]).unwrap();
    let profile = Curve::Polycurve { segments: vec![arc,
        Curve::Line { start: [0., 2., 0.], end: [2., 0., 0.], domain: [0., 1.] }], reversed: false };
    let shape = om9_kernel::extrusion::shell(&profile, [0., 0., 5.], deflection, &limits).unwrap();
    let mut display = shape.display().clone();
    let segment = *display.segments.iter().filter(|segment| {
        let a = display.points[segment[0] as usize];
        let b = display.points[segment[1] as usize];
        (a[2] - 5.).abs() < 1e-8 && (b[2] - 5.).abs() < 1e-8
    }).min_by(|a, b| {
        let distance = |s: &&[u32; 3]| {
            let p = display.points[s[0] as usize]; let q = display.points[s[1] as usize];
            ((p[0] + q[0]) * 0.5 - 1.847759065).powi(2) + ((p[1] + q[1]) * 0.5 - 0.765366865).powi(2)
        };
        distance(a).total_cmp(&distance(b))
    }).unwrap();
    display.points[segment[0] as usize][2] += shift;
    display.points[segment[1] as usize][2] += shift;
    let a = display.points[segment[0] as usize]; let b = display.points[segment[1] as usize];
    let visible = std::array::from_fn(|i| (a[i] + b[i]) * 0.5);
    if deflection == 0.01 {
        assert!(om9_kernel::editing::source_edge_curve_at_point_admitted(&shape, visible, 0.05, &limits, |_, _| Ok(())).is_ok(),
            "the point-only resolver sees one source edge despite the altered segment");
    }
    let altered = om9_core::brep::Brep::from_native(shape.source().to_owned(), display, &limits).unwrap();
    let mut document = Document::new("modified arc display");
    document.tolerance = Some(om9_core::history::Tolerance::try_new(
        if shift < 1e-6 { 1e-7 } else { 0.001 }, 0., 0.).unwrap());
    let entity = Entity::new("shell", Geometry::Brep(altered), document.current_layer);
    document.entities.insert(entity.id, entity);
    let path = std::env::temp_dir().join(format!("om9-dupedge-modified-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, &document, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    let mut app = MatrixApp::from_file(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    app.controller.submit("Box 1 1 1").unwrap();
    app.controller.undo().unwrap();
    assert_eq!(app.controller.coordinator().redo_len(), 1);
    app.controller.workspace.views.get_mut(&ViewId::Perspective).unwrap().camera =
        om9_scene::camera::Camera::orthographic([8., -10., 14.], [1., 1., 2.5], [0., 0., 1.], 8.).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
    app.controller.submit("DupEdge").unwrap();
    click_in_view(&mut app, &ctx, ViewId::Perspective, visible, Modifiers::NONE);
    assert_eq!(app.controller.subselection().len(), 1, "{}", app.transcript.history());
    let before = app.controller.coordinator().snapshot();
    let result = app.controller.submit("");
    if result.is_ok() { assert!(app.controller.dupedge_source_view_pending()); }
    assert_eq!(app.controller.coordinator().snapshot(), before);
    assert_eq!(app.controller.coordinator().redo_len(), 1);
    assert!(app.controller.dupedge_needs_pick());
    }
    }
}

#[test]
fn reopened_native_cylinder_dupedge_selects_circle_and_undoes() {
    let limits=Limits::default();
    let shape=om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Cylinder {radius_mm:2.,height_mm:5.},0.01,&limits).unwrap();
    let native=om9_kernel::brep::to_3dm(&shape,1e-6,&limits).unwrap();
    let mut doc=Document::with_template_tolerance("native cylinder DupEdge");
    let entity=Entity::new("cylinder",Geometry::NativeBrep(native),doc.current_layer);let id=entity.id;doc.entities.insert(id,entity);
    let path=std::env::temp_dir().join(format!("om9-native-dupedge-circle-{}.om9",uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path,&doc,&limits,&om9_core::transaction::Cancellation::default()).unwrap();
    let mut app=MatrixApp::from_file(&path).unwrap();std::fs::remove_file(path).unwrap();
    let ctx=egui::Context::default();frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);
    app.controller.submit("Point").unwrap();app.controller.submit("50,50,50").unwrap();app.controller.undo().unwrap();
    app.controller.select_objects(&[id]).unwrap();let before=app.controller.document().clone();
    app.controller.submit("DupEdge").unwrap();
    let deadline=std::time::Instant::now()+std::time::Duration::from_secs(20);
    while app.controller.dupedge_source_view_pending() {assert!(std::time::Instant::now()<deadline);frame(&mut app,&ctx,vec![]);std::thread::yield_now();}
    click_in_view(&mut app,&ctx,ViewId::Perspective,[0.,-2.,5.],Modifiers::NONE);
    app.controller.submit("").unwrap_or_else(|e|panic!("{e:?}; {}",app.transcript.history()));
    let entity=app.controller.document().entities.values().find(|e|e.label=="DupEdge").expect("curve");
    let Geometry::Curve(curve)=&entity.geometry else {panic!("curve")};
    for t in [0.,0.2,0.4,0.7,1.] {
        let p=om9_core::evaluation::evaluate_curve(curve,om9_core::evaluation::Parameter::Fraction(t),0,entity.placement,&limits).unwrap()[0];
        assert!((p[0].hypot(p[1])-2.).abs()<1e-6&&(p[2]-5.).abs()<1e-6,"{p:?}");
    }
    let after=app.controller.document().clone();app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);app.controller.redo().unwrap();assert_eq!(app.controller.document(),&after);
}

#[test]
fn viewport_dupedge_reopened_cylinder_selects_closed_source_edge() {
    let limits = Limits::default();
    for profile_deflection in [0.5, 0.01] {
    let shape = om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Cylinder { radius_mm: 2., height_mm: 5. }, profile_deflection, &limits,
    ).unwrap();
    let mut document = Document::new("reopened cylinder");
    let entity = Entity::new("Cylinder", Geometry::Brep(shape), document.current_layer);
    document.entities.insert(entity.id, entity);
    let path = std::env::temp_dir().join(format!("om9-dupedge-circle-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, &document, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    let mut app = MatrixApp::from_file(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
    app.controller.submit("DupEdge").unwrap();
    click_in_view(&mut app, &ctx, ViewId::Perspective, [0., -2., 5.], Modifiers::NONE);
    assert_eq!(app.controller.subselection().len(), 1, "{}", app.transcript.history());
    app.controller.submit("").unwrap();
    let output = app.controller.document().entities.values().find(|entity| entity.label == "DupEdge").unwrap();
    let id = output.id;
    let Geometry::Curve(curve) = &output.geometry else { panic!("expected curve") };
    for fraction in [0., 0.2, 0.4, 0.7, 1.] {
        let p = om9_core::evaluation::evaluate_curve(curve, om9_core::evaluation::Parameter::Fraction(fraction),
            0, Transform::identity(), &limits).unwrap()[0];
        assert!((p[0].hypot(p[1]) - 2.).abs() < 1e-7 && (p[2] - 5.).abs() < 1e-7, "{p:?}");
    }
    app.controller.undo().unwrap(); assert!(!app.controller.document().entities.contains_key(&id));
    app.controller.redo().unwrap(); assert!(app.controller.document().entities.contains_key(&id));
    }
}

#[test]
fn viewport_dupedge_opens_preexisting_v4_box_fixture() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../implementation/evidence/M03/solid-ui-followup-2026-10-23/review-fixtures/OM9-SOLID-012-after.om9");
    let mut app = MatrixApp::from_file(&path).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
    app.controller.submit("DupEdge").unwrap();
    click_in_view(&mut app, &ctx, ViewId::Perspective, [1., 0., 4.], Modifiers::NONE);
    assert_eq!(app.controller.subselection().len(), 1, "{}", app.transcript.history());
    app.controller.submit("").unwrap();
    let output = app.controller.document().entities.values().find(|entity| entity.label == "DupEdge").unwrap();
    let output_id = output.id;
    let Geometry::Curve(curve) = &output.geometry else { panic!("expected curve") };
    let actual: Vec<_> = [0., 1.].into_iter().map(|fraction| om9_core::evaluation::evaluate_curve(curve,
        om9_core::evaluation::Parameter::Fraction(fraction), 0, Transform::identity(), &Limits::default()).unwrap()[0]).collect();
    for expected in [[0., 0., 4.], [2., 0., 4.]] {
        assert!(actual.iter().any(|point| (0..3).all(|axis| (point[axis] - expected[axis]).abs() < 1e-8)));
    }
    app.controller.undo().unwrap(); assert!(!app.controller.document().entities.contains_key(&output_id));
    app.controller.redo().unwrap(); assert!(app.controller.document().entities.contains_key(&output_id));
}

#[test]
fn viewport_dupedge_reopened_box_accepts_two_preselected_edges() {
    let mut original = om9_app::session::Controller::new(Document::new("reopened Box"), Limits::default()).unwrap();
    original.submit("Box 4 6 8").unwrap();
    let path = std::env::temp_dir().join(format!("om9-dupedge-preselect-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, original.document(), &Limits::default(),
        &om9_core::transaction::Cancellation::default()).unwrap();
    let mut app = MatrixApp::from_file(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
    app.set_sub_object_kind(om9_scene::scene::PickKind::Edge);
    let modifiers = Modifiers { ctrl: true, shift: true, ..Modifiers::NONE };
    click_in_view(&mut app, &ctx, ViewId::Perspective, [2., 0., 8.], modifiers);
    click_in_view(&mut app, &ctx, ViewId::Perspective, [4., 3., 8.], modifiers);
    assert_eq!(app.controller.subselection().len(), 2);
    app.controller.dispatch("OM9-CURVE-040", InvocationOrigin::Menu).unwrap();
    let curves: Vec<_> = app.controller.document().entities.values().filter(|entity| entity.label == "DupEdge").collect();
    assert_eq!(curves.len(), 2);
    let ids: Vec<_> = curves.iter().map(|entity| entity.id).collect();
    for endpoints in [[[0., 0., 8.], [4., 0., 8.]], [[4., 0., 8.], [4., 6., 8.]]] {
        assert!(curves.iter().any(|entity| {
            let Geometry::Curve(curve) = &entity.geometry else { return false };
            let actual: Vec<_> = [0., 1.].into_iter().map(|fraction| om9_core::evaluation::evaluate_curve(curve,
                om9_core::evaluation::Parameter::Fraction(fraction), 0, Transform::identity(), &Limits::default()).unwrap()[0]).collect();
            endpoints.iter().all(|expected| actual.iter().any(|point| (0..3).all(|axis| (point[axis] - expected[axis]).abs() < 1e-8)))
        }));
    }
    app.controller.undo().unwrap();
    assert!(ids.iter().all(|id| !app.controller.document().entities.contains_key(id)));
    app.controller.redo().unwrap();
    assert!(ids.iter().all(|id| app.controller.document().entities.contains_key(id)));
}

#[test]
fn viewport_dupedge_reopened_coincident_edges_refuse_without_losing_redo() {
    let limits = Limits::default();
    let shape = om9_kernel::brep::read_brep(include_str!("fixtures/dupedge-coincident.brep"), 0.01, &limits).unwrap();
    let mut document = Document::new("coincident edges");
    let entity = Entity::new("coincident boxes", Geometry::Brep(shape), document.current_layer);
    document.entities.insert(entity.id, entity);
    let path = std::env::temp_dir().join(format!("om9-dupedge-coincident-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, &document, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    let mut app = MatrixApp::from_file(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    app.controller.submit("Box 1 1 1").unwrap();
    app.controller.undo().unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
    let before = app.controller.coordinator().snapshot();
    app.controller.submit("DupEdge").unwrap();
    click_in_view(&mut app, &ctx, ViewId::Perspective, [2., 0., 6.], Modifiers::NONE);
    assert!(app.controller.dupedge_needs_pick());
    assert!(app.controller.subselection().is_empty());
    assert_eq!(app.controller.coordinator().snapshot(), before);
    assert_eq!(app.controller.coordinator().redo_len(), 1);
}

#[test]
fn viewport_dupedge_rejects_corner_with_multiple_edges() {
    let mut app = MatrixApp::new(Document::new("ambiguous DupEdge corner")).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
    app.controller.submit("Box 4 6 8").unwrap();
    let before = app.controller.coordinator().snapshot();
    app.controller.dispatch("OM9-CURVE-040", InvocationOrigin::Menu).unwrap();
    click_in_view(&mut app, &ctx, ViewId::Perspective, [4., 0., 8.], Modifiers::NONE);
    assert!(app.controller.dupedge_needs_pick());
    assert!(app.controller.subselection().is_empty());
    assert_eq!(app.controller.coordinator().snapshot(), before);
}

#[test]
fn viewport_dupedge_reports_candidate_budget_error_and_keeps_session() {
    let mut app = MatrixApp::new(Document::new("bounded DupEdge")).unwrap();
    let body = om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Box { x_mm: 4., y_mm: 6., z_mm: 8. },
        0.01, &Limits::default(),
    ).unwrap();
    let mut document = Document::new("bounded DupEdge");
    let entity = Entity::new("box", Geometry::Brep(body), document.current_layer);
    document.entities.insert(entity.id, entity);
    let limits = Limits { max_outputs: 1, ..Limits::default() };
    app.controller = om9_app::session::Controller::new(document, limits).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
    app.controller.dispatch("OM9-CURVE-040", InvocationOrigin::Menu).unwrap();
    let before = app.controller.coordinator().snapshot();
    click_in_view(&mut app, &ctx, ViewId::Perspective, [4., 0., 8.], Modifiers::NONE);
    assert!(app.transcript.history().contains("edge pick candidates"), "{}", app.transcript.history());
    assert!(app.controller.dupedge_needs_pick());
    assert!(app.controller.subselection().is_empty());
    assert_eq!(app.controller.coordinator().snapshot(), before);
}

#[test]
fn viewport_dupedge_accepts_preselected_edges() {
    let mut app = MatrixApp::new(Document::new("preselected DupEdge")).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
    app.controller.submit("Box 4 6 8").unwrap();
    app.set_sub_object_kind(om9_scene::scene::PickKind::Edge);
    let modifiers = Modifiers { ctrl: true, shift: true, ..Modifiers::NONE };
    click_in_view(&mut app, &ctx, ViewId::Perspective, [2., 0., 8.], modifiers);
    click_in_view(&mut app, &ctx, ViewId::Perspective, [4., 3., 8.], modifiers);
    assert_eq!(app.controller.subselection().len(), 2);
    app.controller.dispatch("OM9-CURVE-040", InvocationOrigin::Menu).unwrap();
    assert_eq!(app.controller.document().entities.len(), 3);
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document().entities.len(), 1);
}

#[test]
fn viewport_ellipsoid_center_axes_and_side_height() {
    let mut app = MatrixApp::new(Document::new("viewport ellipsoid")).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![]);
    let before = app.controller.document().clone();
    app.controller.dispatch("OM9-SOLID-013", InvocationOrigin::Menu).unwrap();
    click(&mut app, &ctx, [0., 0., 0.]);
    click(&mut app, &ctx, [4., 0., 0.]);
    click(&mut app, &ctx, [0., 3., 0.]);
    assert!(app.controller.prompt().starts_with("End of third"));
    click_in_view(&mut app, &ctx, ViewId::SideView, [0., 0., 5.], Modifiers::NONE);
    let entity = app.controller.document().entities.values().next().unwrap();
    let Geometry::Brep(body) = &entity.geometry else { panic!("ellipsoid BRep") };
    let volume = om9_kernel::creation::measure(body, app.controller.coordinator().limits()).unwrap().volume_mm3;
    assert!((volume - 80.0 * std::f64::consts::PI).abs() < 8.0, "{volume}");
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &before);
}

#[test]
fn viewport_ellipsoid_diameter_axes_and_side_height() {
    let mut app = MatrixApp::new(Document::new("viewport ellipsoid diameter")).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![]);
    let before = app.controller.document().clone();
    app.controller.dispatch("EllipsoidDiameter", InvocationOrigin::Menu).unwrap();
    click(&mut app, &ctx, [-4., 0., 0.]);
    click(&mut app, &ctx, [4., 0., 0.]);
    click(&mut app, &ctx, [0., 3., 0.]);
    click_in_view(&mut app, &ctx, ViewId::SideView, [0., 0., 5.], Modifiers::NONE);
    let entity = app.controller.document().entities.values().next().unwrap();
    let Geometry::Brep(body) = &entity.geometry else { panic!("ellipsoid BRep") };
    let volume = om9_kernel::creation::measure(body, app.controller.coordinator().limits()).unwrap().volume_mm3;
    assert!((volume - 80.0 * std::f64::consts::PI).abs() < 8.0, "{volume}");
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &before);
}

#[test]
fn viewport_ellipsoid_from_foci_and_markers() {
    let mut app = MatrixApp::new(Document::new("viewport ellipsoid foci")).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![]);
    let before = app.controller.document().clone();
    app.controller.dispatch("EllipsoidFromFoci", InvocationOrigin::Menu).unwrap();
    app.controller.submit("MarkFoci=Yes").unwrap();
    click(&mut app, &ctx, [-3., 0., 0.]);
    click(&mut app, &ctx, [3., 0., 0.]);
    click(&mut app, &ctx, [0., 4., 0.]);
    assert_eq!(app.controller.prompt(), "End of third axis");
    click_in_view(&mut app, &ctx, ViewId::SideView, [0., 0., 2.], Modifiers::NONE);
    assert_eq!(app.controller.document().entities.len(), 3);
    let entity = app.controller.document().entities.values().find(|e| matches!(e.geometry, Geometry::Brep(_))).unwrap();
    let Geometry::Brep(body) = &entity.geometry else { panic!("ellipsoid BRep") };
    let volume = om9_kernel::creation::measure(body, app.controller.coordinator().limits()).unwrap().volume_mm3;
    assert!((volume - 160.0 * std::f64::consts::PI / 3.0).abs() < 8.0, "{volume}");
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &before);
}

#[test]
fn viewport_ellipsoid_corner_and_side_height() {
    let mut app = MatrixApp::new(Document::new("viewport ellipsoid corner")).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![]);
    let before = app.controller.document().clone();
    app.controller.submit("Ellipsoid Corner").unwrap();
    click(&mut app, &ctx, [-4., -3., 0.]);
    click(&mut app, &ctx, [4., 3., 0.]);
    assert_eq!(app.controller.prompt(), "End of third axis");
    assert_eq!(app.controller.solid_preview([0., 0., 5.]).len(), 192);
    assert_eq!(app.controller.document(), &before);
    click_in_view(&mut app, &ctx, ViewId::SideView, [0., 0., 5.], Modifiers::NONE);
    let entity = app.controller.document().entities.values().next().unwrap();
    let Geometry::Brep(body) = &entity.geometry else { panic!("ellipsoid BRep") };
    let volume = om9_kernel::creation::measure(body, app.controller.coordinator().limits()).unwrap().volume_mm3;
    assert!((volume - 80.0 * std::f64::consts::PI).abs() < 8.0, "{volume}");
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &before);
}

#[test]
fn viewport_ellipsoid_around_curve_selects_source_and_center() {
    let rail = Entity::new("rail", Geometry::Curve(Curve::Line { start: [-5.,0.,0.], end: [5.,0.,0.], domain: [0.,1.] }), None);
    let id = rail.id; let mut doc = Document::new("viewport ellipsoid around"); doc.entities.insert(id, rail);
    let mut app = MatrixApp::new(doc).unwrap(); let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
    let before = app.controller.document().clone();
    app.controller.submit("Ellipsoid AroundCurve").unwrap();
    click(&mut app, &ctx, [0.,0.,0.]); assert_eq!(app.controller.prompt(), "Pick center on curve or OnCurve=");
    click(&mut app, &ctx, [0.,0.,0.]); assert_eq!(app.controller.prompt(), "End of first axis");
    app.controller.submit("2").unwrap(); app.controller.submit("3").unwrap(); app.controller.submit("4").unwrap();
    let entity = app.controller.document().entities.values().find(|e| e.id != id).unwrap();
    let a = entity.placement.point([0.,0.,0.]).unwrap(); let b = entity.placement.point([0.,0.,4.]).unwrap();
    assert!((b[0]-a[0]-4.).abs() < 1e-7);
    let Geometry::Brep(body) = &entity.geometry else { panic!("ellipsoid BRep") };
    let volume = om9_kernel::creation::measure(body, app.controller.coordinator().limits()).unwrap().volume_mm3;
    assert!((volume - 32. * std::f64::consts::PI).abs() < 1e-7);
    app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
}

#[test]
fn viewport_torus_picks_ring_and_tube_in_side_view() {
    let mut app=MatrixApp::new(Document::new("viewport torus")).unwrap();
    let ctx=egui::Context::default(); frame(&mut app,&ctx,vec![]); frame(&mut app,&ctx,vec![]);
    let before=app.controller.document().clone(); app.controller.submit("Torus").unwrap();
    click(&mut app,&ctx,[0.,0.,0.]); click(&mut app,&ctx,[5.,0.,0.]);
    assert_eq!(app.controller.prompt(),"Tube radius or point");
    assert_eq!(app.controller.solid_preview([5.,0.,2.]).len(),256);
    assert_eq!(app.controller.document(),&before);
    click_in_view(&mut app,&ctx,ViewId::SideView,[5.,0.,2.],Modifiers::NONE);
    let entity=app.controller.document().entities.values().next().unwrap();
    let Geometry::Brep(body)=&entity.geometry else {panic!("torus BRep")};
    let volume=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap().volume_mm3;
    assert!((volume-40.*std::f64::consts::PI.powi(2)).abs()<15.,"{volume}");
    app.controller.undo().unwrap(); assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_torus_three_points_and_side_view_tube() {
    let mut app=MatrixApp::new(Document::new("viewport torus 3point")).unwrap();
    let ctx=egui::Context::default(); frame(&mut app,&ctx,vec![]); frame(&mut app,&ctx,vec![]);
    let before=app.controller.document().clone(); app.controller.submit("Torus 3Point").unwrap();
    click(&mut app,&ctx,[5.,0.,0.]); click(&mut app,&ctx,[-5.,0.,0.]); click(&mut app,&ctx,[0.,5.,0.]);
    assert_eq!(app.controller.prompt(),"Tube radius or point");
    click_in_view(&mut app,&ctx,ViewId::SideView,[0.,5.,2.],Modifiers::NONE);
    let entity=app.controller.document().entities.values().next().unwrap(); let Geometry::Brep(body)=&entity.geometry else {panic!("torus BRep")};
    let volume=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap().volume_mm3;
    assert!((volume-40.*std::f64::consts::PI.powi(2)).abs()<15.,"{volume}");
    app.controller.undo().unwrap(); assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_torus_three_point_radius_selects_side() {
    let mut app=MatrixApp::new(Document::new("viewport torus 3point radius")).unwrap();
    let ctx=egui::Context::default(); frame(&mut app,&ctx,vec![]); frame(&mut app,&ctx,vec![]);
    let before=app.controller.document().clone(); app.controller.submit("Torus 3Point").unwrap();
    click(&mut app,&ctx,[-2.,0.,0.]); click(&mut app,&ctx,[2.,0.,0.]);
    app.controller.submit("Radius=3").unwrap(); click(&mut app,&ctx,[0.,2.,0.]);
    assert_eq!(app.controller.prompt(),"Tube radius or point"); app.controller.submit("1").unwrap();
    let entity=app.controller.document().entities.values().next().unwrap();
    let center=entity.placement.point([0.,0.,0.]).unwrap(); assert!((center[1]-5.0_f64.sqrt()).abs()<0.2);
    app.controller.undo().unwrap(); assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_torus_around_curve_selects_source_and_center() {
    let rail=Entity::new("rail",Geometry::Curve(Curve::Line{start:[-5.,0.,0.],end:[5.,0.,0.],domain:[0.,1.]}),None);let id=rail.id;
    let mut doc=Document::new("viewport torus around");doc.entities.insert(id,rail);let mut app=MatrixApp::new(doc).unwrap();let ctx=egui::Context::default();
    frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);let before=app.controller.document().clone();
    app.controller.submit("Torus AroundCurve").unwrap();click(&mut app,&ctx,[0.,0.,0.]);assert_eq!(app.controller.prompt(),"Pick center on curve or OnCurve=");
    click(&mut app,&ctx,[0.,0.,0.]);assert_eq!(app.controller.prompt(),"Ring radius or point");
    app.controller.submit("5").unwrap();app.controller.submit("1").unwrap();
    let entity=app.controller.document().entities.values().find(|e|e.id!=id).unwrap();let a=entity.placement.point([0.,0.,0.]).unwrap();let b=entity.placement.point([0.,0.,1.]).unwrap();
    assert!((b[0]-a[0]-1.).abs()<1e-7);app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_torus_vertical_picks_ring_and_tube() {
    let mut app=MatrixApp::new(Document::new("viewport vertical torus")).unwrap();let ctx=egui::Context::default();
    frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);let before=app.controller.document().clone();
    app.controller.submit("Torus Vertical").unwrap();click(&mut app,&ctx,[0.,0.,0.]);click(&mut app,&ctx,[5.,0.,0.]);
    assert_eq!(app.controller.prompt(),"Tube radius or point");click(&mut app,&ctx,[6.,0.,0.]);
    let entity=app.controller.document().entities.values().next().unwrap();let a=entity.placement.point([0.,0.,0.]).unwrap();let b=entity.placement.point([0.,0.,1.]).unwrap();
    assert!((b[1]-a[1]+1.).abs()<0.05);app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_cone_center_radius_and_apex() {
    let mut app=MatrixApp::new(Document::new("viewport cone")).unwrap();let ctx=egui::Context::default();
    frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);let before=app.controller.document().clone();app.controller.submit("Cone").unwrap();
    click(&mut app,&ctx,[0.,0.,0.]);click(&mut app,&ctx,[3.,0.,0.]);assert_eq!(app.controller.solid_preview([0.,0.,5.]).len(),68);
    click_in_view(&mut app,&ctx,ViewId::SideView,[0.,0.,5.],Modifiers::NONE);let e=app.controller.document().entities.values().next().unwrap();
    let Geometry::Brep(body)=&e.geometry else{panic!()};let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();assert!((m.volume_mm3-15.*std::f64::consts::PI).abs()<5.,"{m:?}");
    app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_pyramid_star_two_radii_and_apex() {
    let mut app=MatrixApp::new(Document::new("viewport pyramid star")).unwrap();let ctx=egui::Context::default();
    frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);let before=app.controller.document().clone();app.controller.submit("Pyramid Star").unwrap();app.controller.submit("NumSides=5").unwrap();
    click(&mut app,&ctx,[0.,0.,0.]);click(&mut app,&ctx,[5.,0.,0.]);click(&mut app,&ctx,[2.,0.,0.]);
    click_in_view(&mut app,&ctx,ViewId::SideView,[0.,0.,6.],Modifiers::NONE);
    let e=app.controller.document().entities.values().next().unwrap();let Geometry::Brep(body)=&e.geometry else{panic!()};
    let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();let expected=5.*5.*2.*(std::f64::consts::PI/5.).sin()*2.;assert!((m.volume_mm3-expected).abs()<8.,"{m:?}");
    app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_pyramid_edge_flip_and_apex() {
    let mut app=MatrixApp::new(Document::new("viewport pyramid edge")).unwrap();let ctx=egui::Context::default();
    frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);let before=app.controller.document().clone();app.controller.submit("Pyramid Edge").unwrap();
    click(&mut app,&ctx,[0.,0.,0.]);app.controller.submit("Flip").unwrap();click(&mut app,&ctx,[4.,0.,0.]);
    click_in_view(&mut app,&ctx,ViewId::SideView,[2.,-2.,6.],Modifiers::NONE);
    let e=app.controller.document().entities.values().next().unwrap();let Geometry::Brep(body)=&e.geometry else{panic!()};
    let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();assert!((m.volume_mm3-32.).abs()<4.,"{m:?}");
    let center=e.placement.point([0.,0.,0.]).unwrap();assert!((center[0]-2.).abs()<0.1 && (center[1]+2.).abs()<0.1);app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_pyramid_circumscribed_midpoint_and_apex() {
    let mut app=MatrixApp::new(Document::new("viewport pyramid circumscribed")).unwrap();let ctx=egui::Context::default();
    frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);let before=app.controller.document().clone();app.controller.submit("Pyramid").unwrap();app.controller.submit("Circumscribed").unwrap();
    click(&mut app,&ctx,[0.,0.,0.]);click(&mut app,&ctx,[3.,0.,0.]);click_in_view(&mut app,&ctx,ViewId::SideView,[0.,0.,5.],Modifiers::NONE);
    let e=app.controller.document().entities.values().next().unwrap();let Geometry::Brep(body)=&e.geometry else{panic!()};
    let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();assert!((m.volume_mm3-60.).abs()<5.,"{m:?}");
    app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_sphere_aroundcurve_resolves_ambiguous_source_edges() {
    let mut app = MatrixApp::new(Document::new("overlapping source edges")).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
    app.controller.submit("Box 4 6 8").unwrap();
    let first = *app.controller.document().entities.keys().next().unwrap();
    app.controller.submit("Box 4 6 8").unwrap();
    let second = *app.controller.document().entities.keys().find(|&&id| id != first).unwrap();
    app.controller.clear_selection();
    app.controller.submit("Sphere AroundCurve").unwrap();
    click_in_view(&mut app, &ctx, ViewId::Perspective, [2.,0.,8.], Modifiers::NONE);
    let index = app.ambiguous_candidates().iter().position(|id| *id == second).expect("second box candidate");
    assert!(app.ambiguous_labels()[index].contains("Edge"));
    app.choose_ambiguous(index).unwrap();
    assert_eq!(app.controller.solid_sphere_brep_edge_stage(), 2);
    click_in_view(&mut app, &ctx, ViewId::Perspective, [3.,0.,8.], Modifiers::NONE);
    app.controller.submit("3").unwrap();
    let sphere = app.controller.document().entities.values().find(|e| e.id != first && e.id != second).unwrap();
    let center = sphere.placement.point([0.,0.,0.]).unwrap();
    assert!((center[0]-3.).abs() < 1. && center[1].abs() < 0.1 && (center[2]-8.).abs() < 0.1, "{center:?}");
    app.controller.undo().unwrap();
    app.controller.submit("Sphere AroundCurve").unwrap();
    click_in_view(&mut app, &ctx, ViewId::Perspective, [2.,0.,8.], Modifiers::NONE);
    assert!(app.ambiguous_candidates().len() >= 2);
    app.controller.cancel();
    app.controller.submit("Sphere AroundCurve").unwrap();
    assert!(app.choose_ambiguous(0).is_err());
    assert_eq!(app.controller.solid_sphere_brep_edge_stage(), 1);
}

#[test]
fn viewport_sphere_aroundcurve_uses_selected_edge_when_hits_overlap() {
    use om9_app::session::SelectedSubobject;
    use om9_scene::scene::{PickIntent, PickKind, Subelement};
    let mut app = MatrixApp::new(Document::new("overlapping box edges")).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
    app.controller.submit("Box 4 6 8").unwrap();
    let id = *app.controller.document().entities.keys().next().unwrap();
    app.controller.submit("Box 4 6 8").unwrap();
    let other_id = *app.controller.document().entities.keys().find(|&&x| x != id).unwrap();
    let Geometry::Brep(body) = &app.controller.document().entities[&id].geometry else { panic!("box") };
    let index = (0..body.display().edges.len()).find(|&i| {
        let Ok((a,b)) = om9_kernel::editing::straight_edge_endpoints(body, i, app.controller.coordinator().limits()) else { return false; };
        [a,b].iter().all(|p| p[1].abs() < 1e-7 && (p[2] - 8.).abs() < 1e-7)
    }).unwrap();
    let key = body.display().edges[index];
    app.controller.clear_selection();
    app.controller.submit("Sphere AroundCurve").unwrap();
    let scene = app.controller.scene().unwrap();
    let object = scene.objects.iter().find(|o| o.root == id && o.path.is_empty()).unwrap();
    let reference = SelectedSubobject { root: id, path: vec![], subset: Subelement::BrepEdge(key), source: object.source, world_epoch: scene.epoch };
    app.controller.solid_sphere_pick_brep_edge(reference, [2.,0.,8.]).unwrap();
    let rect = app.view_rects[&ViewId::LookingDown];
    let vp = Viewport { origin: [rect.min.x as f64, rect.min.y as f64], size: [rect.width() as f64, rect.height() as f64] };
    let camera = &app.controller.workspace.views[&ViewId::LookingDown].camera;
    let screen = camera.project([2.,0.,8.], vp).unwrap().unwrap().screen;
    let hits = app.controller.scene().unwrap().pick_kind_all(screen, camera, vp, 6., PickIntent::Select, PickKind::Edge).unwrap();
    assert!(hits.len() >= 2, "fixture needs overlapping projected edges: {}", hits.len());
    click(&mut app, &ctx, [2.,0.,8.]);
    assert!(app.controller.prompt().starts_with("Radius"), "{}", app.controller.prompt());
    app.controller.submit("3").unwrap();
    let sphere = app.controller.document().entities.values().find(|e| e.id != id && e.id != other_id).unwrap();
    let center = sphere.placement.point([0.,0.,0.]).unwrap();
    assert!((center[0]-2.).abs() < 0.2 && center[1].abs() < 1e-7 && (center[2]-8.).abs() < 1e-7, "{center:?}");
}

#[test]
fn viewport_sphere_aroundcurve_selects_brep_edge_and_center() {
    let mut app = MatrixApp::new(Document::new("viewport sphere BRep edge")).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
    app.controller.submit("Box 4 6 8").unwrap();
    let id = *app.controller.document().entities.keys().next().unwrap();
    app.controller.clear_selection();
    app.controller.submit("Sphere AroundCurve").unwrap();
    click_in_view(&mut app, &ctx, ViewId::Perspective, [2., 0., 8.], Modifiers::NONE);
    assert_eq!(app.controller.solid_sphere_brep_edge_stage(), 2, "{}", app.controller.prompt());
    click_in_view(&mut app, &ctx, ViewId::Perspective, [3., 0., 8.], Modifiers::NONE);
    assert!(app.controller.prompt().starts_with("Radius"), "{}", app.controller.prompt());
    app.controller.submit("3").unwrap();
    let sphere = app.controller.document().entities.values().find(|e| e.id != id).unwrap();
    let center = sphere.placement.point([0., 0., 0.]).unwrap();
    assert!((center[0] - 3.).abs() < 1.0 && center[1].abs() < 0.1 && (center[2] - 8.).abs() < 0.1, "{center:?}");
}

#[test]
fn viewport_sphere_aroundcurve_uses_elevated_curve_world_hit() {
    let mut doc = Document::new("elevated curve center");
    let rail = Entity::new("rail", Geometry::Curve(om9_core::curve_build::polyline(&[[-10., 0., 5.], [10., 0., 5.]]).unwrap()), None);
    let id = rail.id; doc.entities.insert(id, rail);
    let mut app = MatrixApp::new(doc).unwrap(); let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
    app.controller.submit("Sphere AroundCurve").unwrap();
    click_in_view(&mut app, &ctx, ViewId::Perspective, [2.,0.,5.], Modifiers::NONE);
    assert_eq!(app.controller.solid_sphere_brep_edge_stage(), 3);
    click_in_view(&mut app, &ctx, ViewId::Perspective, [4.,0.,5.], Modifiers::NONE);
    assert!(app.controller.prompt().starts_with("Radius"), "{}", app.controller.prompt());
    app.controller.submit("3").unwrap();
    let sphere = app.controller.document().entities.values().find(|e| e.id != id).unwrap();
    let center = sphere.placement.point([0.,0.,0.]).unwrap();
    assert!((center[0]-4.).abs() < 0.2 && center[1].abs() < 1e-8 && (center[2]-5.).abs() < 1e-8, "{center:?}");
}

#[test]
fn viewport_slab_development_picks_side_near_second_segment() {
    let mut doc=Document::new("Slab L point picks");
    let rail=Entity::new("rail",Geometry::Curve(om9_core::curve_build::polyline(&[[0.,0.,0.],[4.,0.,0.],[4.,3.,0.]]).unwrap()),None);
    let id=rail.id;doc.entities.insert(id,rail);
    let mut app=MatrixApp::new(doc).unwrap();let ctx=egui::Context::default();
    frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);
    app.controller.select_objects(&[id]).unwrap();app.controller.begin_slab_development().unwrap();
    click(&mut app,&ctx,[6.,2.,0.]);
    assert!(app.controller.prompt().starts_with("Slab height"),"{}",app.controller.prompt());
    click_in_view(&mut app,&ctx,ViewId::SideView,[0.,0.,3.],Modifiers::NONE);
    let result=app.controller.document().entities.values().find(|e|e.id!=id).expect("Slab output");
    let Geometry::Brep(body)=&result.geometry else{panic!("BRep")};
    let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();
    assert!((m.volume_mm3-54.).abs()<5.,"{m:?}");
}

#[test]
fn viewport_slab_development_numeric_distance_then_side() {
    let mut doc=Document::new("Slab distance and side");
    let rail=Entity::new("rail",Geometry::Curve(om9_core::curve_build::polyline(&[[0.,0.,0.],[4.,0.,0.]]).unwrap()),None);
    let id=rail.id;doc.entities.insert(id,rail);
    let mut app=MatrixApp::new(doc).unwrap();let ctx=egui::Context::default();
    frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);
    app.controller.select_objects(&[id]).unwrap();app.controller.begin_slab_development().unwrap();
    app.controller.submit("Distance=2").unwrap();click(&mut app,&ctx,[0.,-6.,0.]);
    assert!(app.controller.prompt().starts_with("Slab height"));
    click_in_view(&mut app,&ctx,ViewId::SideView,[0.,0.,3.],Modifiers::NONE);
    let result=app.controller.document().entities.values().find(|e|e.id!=id).expect("Slab output");
    let Geometry::Brep(body)=&result.geometry else{panic!("BRep")};
    let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();
    assert!((m.volume_mm3-24.).abs()<2.,"{m:?}");
}

#[test]
fn viewport_slab_development_selects_periodic_double_seam_nurbs() {
    use om9_core::geometry::NurbsCurve;
    let limits=om9_core::budget::Limits::default();
    for reversed in [false,true] {
        let n=NurbsCurve {degree:3,poles:vec![[4.,0.,0.],[0.,4.,0.],[-4.,0.,0.],[0.,-4.,0.],[4.,0.,0.],[0.,4.,0.],[-4.,0.,0.]],
            weights:vec![1.,0.7,1.3,0.9,1.,0.7,1.3],knots:vec![0.,1.,2.,2.,4.,5.,6.,6.,8.,9.,10.],
            domain:[2.,6.],periodic:true,reversed,seam:None};
        let curve=Curve::Nurbs(n);let at=om9_core::curve_ops::point_at(&curve,0.875,&limits).unwrap();
        let mut original=Vec::new();let mut shifted=Vec::new();let d=if reversed {-0.25} else {0.25};
        for i in 0..4096 {let f=(i as f64+0.5)/4096.;let p=om9_core::curve_ops::point_at(&curve,f,&limits).unwrap();
            let t=om9_core::curve_ops::tangent_at(&curve,f,&limits).unwrap();original.push(p);shifted.push([p[0]+d*t[1],p[1]-d*t[0],0.]);}
        let area=|ring:&Vec<[f64;3]>|0.5*(0..ring.len()).map(|i| {let a=ring[i];let b=ring[(i+1)%ring.len()];a[0]*b[1]-b[0]*a[1]}).sum::<f64>().abs();
        let expected=2.*(area(&shifted)-area(&original));
        let mut doc=Document::new("Slab double seam viewport");let source=Entity::new("rail",Geometry::Curve(curve),None);
        let id=source.id;doc.entities.insert(id,source);let mut app=MatrixApp::new(doc).unwrap();let ctx=egui::Context::default();
        frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);let before=app.controller.document().clone();app.controller.begin_slab_development().unwrap();
        click(&mut app,&ctx,at);assert!(!app.controller.solid_slab_needs_curve());app.controller.submit("Distance=0.25").unwrap();
        click(&mut app,&ctx,[5.,0.,0.]);assert!(app.controller.prompt().starts_with("Slab height"));
        click_in_view(&mut app,&ctx,ViewId::SideView,[0.,0.,2.],Modifiers::NONE);
        let result=app.controller.document().entities.values().find(|e|e.id!=id).unwrap();let Geometry::Brep(body)=&result.geometry else {panic!("BRep")};
        let m=om9_kernel::creation::measure(body,&limits).unwrap();assert!(m.closed && m.solids==1);
        assert!((m.volume_mm3-expected).abs()<3.,"{reversed} {m:?} expected {expected}");
        app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
    }
}

#[test]
fn viewport_slab_development_selects_certified_periodic_nurbs() {
    use om9_core::geometry::NurbsCurve;
    let limits=om9_core::budget::Limits::default();
    let source=NurbsCurve {degree:2,poles:vec![[4.,0.,0.],[0.,4.,0.],[-4.,0.,0.],[0.,-4.,0.],[4.,0.,0.],[0.,4.,0.]],
        weights:vec![1.;6],knots:(0..=8).map(|v|v as f64).collect(),domain:[2.,6.],periodic:true,reversed:false,seam:None};
    let curve=Curve::Nurbs(source);let at=om9_core::curve_ops::point_at(&curve,0.875,&limits).unwrap();
    let step=1./4096.;let perimeter=(0..4096).map(|i| {let t=om9_core::curve_ops::derivative_at(&curve,(i as f64+0.5)*step,&limits).unwrap();t[0].hypot(t[1])*step}).sum::<f64>();
    let mut doc=Document::new("Slab periodic viewport");let source=Entity::new("rail",Geometry::Curve(curve),None);
    let id=source.id;doc.entities.insert(id,source);let mut app=MatrixApp::new(doc).unwrap();let ctx=egui::Context::default();
    frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);let before=app.controller.document().clone();app.controller.begin_slab_development().unwrap();
    click(&mut app,&ctx,at);assert!(!app.controller.solid_slab_needs_curve());app.controller.submit("Distance=0.25").unwrap();
    click(&mut app,&ctx,[5.,0.,0.]);assert!(app.controller.prompt().starts_with("Slab height"));
    click_in_view(&mut app,&ctx,ViewId::SideView,[0.,0.,2.],Modifiers::NONE);
    let output=app.controller.document().entities.values().find(|e|e.id!=id).unwrap();let Geometry::Brep(body)=&output.geometry else {panic!("BRep")};
    let m=om9_kernel::creation::measure(body,&limits).unwrap();let expected=2.*(0.25*perimeter+std::f64::consts::PI*0.25_f64.powi(2));
    assert!((m.volume_mm3-expected).abs()<3.,"{m:?}");app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_slab_development_selects_shifted_closed_nurbs_seam() {
    let limits=om9_core::budget::Limits::default();
    let step=std::f64::consts::TAU/10000.;let perimeter=(0..10000).map(|i| {let a=(i as f64+0.5)*step;(25.*a.sin().powi(2)+9.*a.cos().powi(2)).sqrt()*step}).sum::<f64>();
    for reversed in [false,true] {
        let mut ellipse=om9_core::curve_build::ellipse([0.;3],[1.,0.,0.],[0.,1.,0.],5.,3.,&limits).unwrap();
        ellipse.seam=Some(0.17);ellipse.reversed=reversed;
        let mut doc=Document::new("Slab shifted-seam viewport");let rail=Entity::new("ellipse",Geometry::Curve(Curve::Nurbs(ellipse)),None);
        let id=rail.id;doc.entities.insert(id,rail);let mut app=MatrixApp::new(doc).unwrap();let ctx=egui::Context::default();
        frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);let before=app.controller.document().clone();
        app.controller.begin_slab_development().unwrap();click(&mut app,&ctx,[5.,0.,0.]);assert!(!app.controller.solid_slab_needs_curve());
        app.controller.submit("Distance=0.5").unwrap();click(&mut app,&ctx,[7.,0.,0.]);assert!(app.controller.prompt().starts_with("Slab height"));
        click_in_view(&mut app,&ctx,ViewId::SideView,[0.,0.,2.],Modifiers::NONE);
        let result=app.controller.document().entities.values().find(|e|e.id!=id).expect("Slab output");
        let Geometry::Brep(body)=&result.geometry else {panic!("BRep")};let m=om9_kernel::creation::measure(body,&limits).unwrap();
        assert!((m.volume_mm3-2.*(0.5*perimeter+0.25*std::f64::consts::PI)).abs()<3.,"{m:?}");
        app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
    }
}

#[test]
fn viewport_slab_development_selects_closed_nurbs() {
    let limits=om9_core::budget::Limits::default();
    let ellipse=om9_core::curve_build::ellipse([0.;3],[1.,0.,0.],[0.,1.,0.],5.,3.,&limits).unwrap();
    let mut doc=Document::new("Slab closed NURBS viewport");let rail=Entity::new("ellipse",Geometry::Curve(Curve::Nurbs(ellipse)),None);
    let id=rail.id;doc.entities.insert(id,rail);let mut app=MatrixApp::new(doc).unwrap();let ctx=egui::Context::default();
    frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);let before=app.controller.document().clone();
    app.controller.begin_slab_development().unwrap();click(&mut app,&ctx,[5.,0.,0.]);assert!(!app.controller.solid_slab_needs_curve());
    app.controller.submit("Distance=0.5").unwrap();click(&mut app,&ctx,[7.,0.,0.]);assert!(app.controller.prompt().starts_with("Slab height"));
    click_in_view(&mut app,&ctx,ViewId::SideView,[0.,0.,2.],Modifiers::NONE);
    let result=app.controller.document().entities.values().find(|e|e.id!=id).expect("Slab output");
    let Geometry::Brep(body)=&result.geometry else {panic!("BRep")};let m=om9_kernel::creation::measure(body,&limits).unwrap();
    let step=std::f64::consts::TAU/10000.;let perimeter=(0..10000).map(|i| {let a=(i as f64+0.5)*step;(25.*a.sin().powi(2)+9.*a.cos().powi(2)).sqrt()*step}).sum::<f64>();
    let expected=2.*(0.5*perimeter+0.25*std::f64::consts::PI);
    assert!((m.volume_mm3-expected).abs()<3.,"{m:?}");app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_slab_development_selects_closed_polyline() {
    use om9_core::geometry::NurbsCurve;
    for nurbs in [false,true] {
        let points=vec![[0.,0.,0.],[4.,0.,0.],[4.,3.,0.],[0.,3.,0.],[0.,0.,0.]];
        let curve=if nurbs {Curve::Nurbs(NurbsCurve {degree:1,poles:points.clone(),weights:vec![1.,2.,1.,3.,1.],
            knots:vec![0.,0.,0.25,0.5,0.75,1.,1.],domain:[0.,1.],periodic:false,reversed:false,seam:None})}
            else {om9_core::curve_build::polyline(&points).unwrap()};
        let mut doc=Document::new("Slab closed viewport");let rail=Entity::new("rail",Geometry::Curve(curve),None);
        let id=rail.id;doc.entities.insert(id,rail);let mut app=MatrixApp::new(doc).unwrap();let ctx=egui::Context::default();
        frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);let before=app.controller.document().clone();
        app.controller.begin_slab_development().unwrap();click(&mut app,&ctx,[2.,0.,0.]);
        assert!(!app.controller.solid_slab_needs_curve());click(&mut app,&ctx,[2.,-1.,0.]);
        assert!(app.controller.prompt().starts_with("Slab height"));
        click_in_view(&mut app,&ctx,ViewId::SideView,[0.,0.,2.],Modifiers::NONE);
        let result=app.controller.document().entities.values().find(|e|e.id!=id).expect("Slab output");
        let Geometry::Brep(body)=&result.geometry else {panic!("BRep")};
        let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();
        assert!((m.volume_mm3-36.).abs()<5.,"{m:?}");app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
    }
}

#[test]
fn viewport_slab_development_selects_degree_one_nurbs() {
    use om9_core::geometry::NurbsCurve;
    for reversed in [false,true] {
        let mut doc=Document::new("Slab degree-one NURBS picks");
        let rail=Entity::new("rail",Geometry::Curve(Curve::Nurbs(NurbsCurve {
            degree:1,poles:vec![[0.,0.,0.],[4.,0.,0.],[4.,3.,0.]],weights:vec![1.,2.,1.],
            knots:vec![0.,0.,0.5,1.,1.],domain:[0.,1.],periodic:false,reversed,seam:None})),None);
        let id=rail.id;doc.entities.insert(id,rail);let mut app=MatrixApp::new(doc).unwrap();let ctx=egui::Context::default();
        frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);
        let before=app.controller.document().clone();app.controller.begin_slab_development().unwrap();
        click(&mut app,&ctx,[2.,0.,0.]);assert!(!app.controller.solid_slab_needs_curve());
        click(&mut app,&ctx,[6.,2.,0.]);assert!(app.controller.prompt().starts_with("Slab height"));
        click_in_view(&mut app,&ctx,ViewId::SideView,[0.,0.,3.],Modifiers::NONE);
        let result=app.controller.document().entities.values().find(|e|e.id!=id).expect("Slab output");
        let Geometry::Brep(body)=&result.geometry else{panic!("BRep")};
        let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();
        assert!((m.volume_mm3-54.).abs()<5.,"{m:?}");
        app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
    }
}

#[test]
fn viewport_slab_development_selects_nurbs_and_creates_sector() {
    use om9_core::geometry::NurbsCurve;
    let mut doc=Document::new("Slab NURBS picks");
    let rail=Entity::new("arc",Geometry::Curve(Curve::Nurbs(NurbsCurve {degree:2,poles:vec![[5.,0.,0.],[5.,5.,0.],[0.,5.,0.]],weights:vec![1.,std::f64::consts::FRAC_1_SQRT_2,1.],knots:vec![0.,0.,0.,1.,1.,1.],domain:[0.,1.],periodic:false,reversed:false,seam:None})),None);
    let id=rail.id;doc.entities.insert(id,rail);let mut app=MatrixApp::new(doc).unwrap();let ctx=egui::Context::default();
    frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);app.controller.begin_slab_development().unwrap();let r=2_f64.sqrt();
    click(&mut app,&ctx,[5./r,5./r,0.]);assert!(!app.controller.solid_slab_needs_curve());
    click(&mut app,&ctx,[7./r,7./r,0.]);assert!(app.controller.prompt().starts_with("Slab height"));
    click_in_view(&mut app,&ctx,ViewId::SideView,[0.,0.,3.],Modifiers::NONE);
    let result=app.controller.document().entities.values().find(|e|e.id!=id).expect("Slab output");
    let Geometry::Brep(body)=&result.geometry else{panic!("BRep")};
    let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();
    assert!((m.volume_mm3-18.*std::f64::consts::PI).abs()<5.,"{m:?}");
}

#[test]
fn viewport_slab_development_selects_reversed_nurbs() {
    use om9_core::geometry::NurbsCurve;
    let mut doc=Document::new("Slab NURBS picks");
    let rail=Entity::new("arc",Geometry::Curve(Curve::Nurbs(NurbsCurve {degree:2,poles:vec![[5.,0.,0.],[5.,5.,0.],[0.,5.,0.]],weights:vec![1.,std::f64::consts::FRAC_1_SQRT_2,1.],knots:vec![0.,0.,0.,1.,1.,1.],domain:[0.,1.],periodic:false,reversed:true,seam:None})),None);
    let id=rail.id;doc.entities.insert(id,rail);let mut app=MatrixApp::new(doc).unwrap();let ctx=egui::Context::default();
    frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);app.controller.begin_slab_development().unwrap();let r=2_f64.sqrt();
    click(&mut app,&ctx,[5./r,5./r,0.]);assert!(!app.controller.solid_slab_needs_curve());
    click(&mut app,&ctx,[3./r,3./r,0.]);assert!(app.controller.prompt().starts_with("Slab height"));
    click_in_view(&mut app,&ctx,ViewId::SideView,[0.,0.,3.],Modifiers::NONE);
    let result=app.controller.document().entities.values().find(|e|e.id!=id).expect("Slab output");
    let Geometry::Brep(body)=&result.geometry else{panic!("BRep")};
    let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();
    assert!((m.volume_mm3-12.*std::f64::consts::PI).abs()<5.,"{m:?}");
}

#[test]
fn viewport_slab_development_selects_arc_and_creates_sector() {
    let mut doc=Document::new("Slab arc picks");
    let rail=Entity::new("arc",Geometry::Curve(Curve::Circle {center:[0.,0.,0.],x:[1.,0.,0.],y:[0.,1.,0.],radius:5.,domain:[0.,std::f64::consts::FRAC_PI_2],angle_domain:None}),None);
    let id=rail.id;doc.entities.insert(id,rail);let mut app=MatrixApp::new(doc).unwrap();let ctx=egui::Context::default();
    frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);
    app.controller.begin_slab_development().unwrap();let r=2_f64.sqrt();
    click(&mut app,&ctx,[5./r,5./r,0.]);assert!(!app.controller.solid_slab_needs_curve());
    click(&mut app,&ctx,[7./r,7./r,0.]);assert!(app.controller.prompt().starts_with("Slab height"));
    click_in_view(&mut app,&ctx,ViewId::SideView,[0.,0.,3.],Modifiers::NONE);
    let result=app.controller.document().entities.values().find(|e|e.id!=id).expect("Slab output");
    let Geometry::Brep(body)=&result.geometry else{panic!("BRep")};
    let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();
    assert!((m.volume_mm3-18.*std::f64::consts::PI).abs()<5.,"{m:?}");
}

#[test]
fn viewport_slab_development_selects_circle_and_creates_annulus() {
    let mut doc=Document::new("Slab circle picks");
    let rail=Entity::new("circle",Geometry::Curve(Curve::Circle {center:[0.,0.,0.],x:[1.,0.,0.],y:[0.,1.,0.],radius:5.,domain:[0.,std::f64::consts::TAU],angle_domain:None}),None);
    let id=rail.id;doc.entities.insert(id,rail);let mut app=MatrixApp::new(doc).unwrap();let ctx=egui::Context::default();
    frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);
    app.controller.begin_slab_development().unwrap();click(&mut app,&ctx,[5.,0.,0.]);
    assert!(!app.controller.solid_slab_needs_curve());
    click(&mut app,&ctx,[7.,0.,0.]);assert!(app.controller.prompt().starts_with("Slab height"));
    click_in_view(&mut app,&ctx,ViewId::SideView,[0.,0.,3.],Modifiers::NONE);
    let result=app.controller.document().entities.values().find(|e|e.id!=id).expect("Slab output");
    let Geometry::Brep(body)=&result.geometry else{panic!("BRep")};
    let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();
    assert!((m.volume_mm3-72.*std::f64::consts::PI).abs()<15.,"{m:?}");
}

#[test]
fn viewport_slab_development_selects_curve_after_start() {
    let mut doc=Document::new("Slab source pick");
    let rail=Entity::new("rail",Geometry::Curve(om9_core::curve_build::polyline(&[[0.,0.,0.],[4.,0.,0.]]).unwrap()),None);
    let id=rail.id;doc.entities.insert(id,rail);
    let mut app=MatrixApp::new(doc).unwrap();let ctx=egui::Context::default();
    frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);
    let before=app.controller.document().clone();app.controller.begin_slab_development().unwrap();
    assert!(app.controller.solid_slab_needs_curve());
    click(&mut app,&ctx,[2.,0.,0.]);
    assert!(!app.controller.solid_slab_needs_curve());assert_eq!(app.controller.document(),&before);
    click(&mut app,&ctx,[0.,-2.,0.]);
    click_in_view(&mut app,&ctx,ViewId::SideView,[0.,0.,3.],Modifiers::NONE);
    let result=app.controller.document().entities.values().find(|e|e.id!=id).expect("Slab output");
    let Geometry::Brep(body)=&result.geometry else{panic!("BRep")};
    let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();
    assert!((m.volume_mm3-24.).abs()<2.,"{m:?}");
    app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_slab_development_ambiguous_curve_choice_is_session_scoped() {
    let mut doc=Document::new("Slab overlapping curves");
    let curve=om9_core::curve_build::polyline(&[[0.,0.,0.],[4.,0.,0.]]).unwrap();
    let a=Entity::new("first",Geometry::Curve(curve.clone()),None);let first=a.id;doc.entities.insert(first,a);
    let b=Entity::new("second",Geometry::Curve(curve),None);let second=b.id;doc.entities.insert(second,b);
    let mut app=MatrixApp::new(doc).unwrap();let ctx=egui::Context::default();
    frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);
    app.controller.begin_slab_development().unwrap();click(&mut app,&ctx,[2.,0.,0.]);
    let index=app.ambiguous_candidates().iter().position(|id|*id==second).expect("second candidate");
    app.choose_ambiguous(index).unwrap();assert!(!app.controller.solid_slab_needs_curve());
    app.controller.cancel();app.controller.begin_slab_development().unwrap();click(&mut app,&ctx,[2.,0.,0.]);
    assert!(app.ambiguous_candidates().len()>=2);app.controller.cancel();app.controller.begin_slab_development().unwrap();
    assert!(app.choose_ambiguous(0).is_err());assert!(app.controller.solid_slab_needs_curve());
    assert_eq!(app.controller.document().entities.len(),2);assert_eq!(app.controller.coordinator().undo_len(),0);
}

#[test]
fn viewport_slab_development_picks_on_vertical_source_plane() {
    let mut doc=Document::new("Slab vertical source plane");
    let rail=Entity::new("rail",Geometry::Curve(om9_core::curve_build::polyline(&[[0.,0.,0.],[0.,4.,0.],[0.,4.,3.]]).unwrap()),None);
    let id=rail.id;doc.entities.insert(id,rail);let mut app=MatrixApp::new(doc).unwrap();let ctx=egui::Context::default();
    frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);
    app.controller.select_objects(&[id]).unwrap();app.controller.begin_slab_development().unwrap();
    click_in_view(&mut app,&ctx,ViewId::SideView,[0.,6.,2.],Modifiers::NONE);
    assert!(app.controller.prompt().starts_with("Slab height"),"{}",app.controller.prompt());
    click(&mut app,&ctx,[3.,0.,0.]);
    let result=app.controller.document().entities.values().find(|e|e.id!=id).expect("Slab output");
    let Geometry::Brep(body)=&result.geometry else{panic!("BRep")};
    let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();
    assert!((m.volume_mm3-54.).abs()<5.,"{m:?}");
}

#[test]
fn viewport_slab_development_picks_side_and_height() {
    let mut doc=Document::new("Slab point picks");
    let rail=Entity::new("rail",Geometry::Curve(om9_core::curve_build::polyline(&[[0.,0.,0.],[4.,0.,0.]]).unwrap()),None);
    let id=rail.id;doc.entities.insert(id,rail);
    let mut app=MatrixApp::new(doc).unwrap();let ctx=egui::Context::default();
    frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);
    app.controller.select_objects(&[id]).unwrap();app.controller.begin_slab_development().unwrap();
    click(&mut app,&ctx,[0.,-2.,0.]);
    assert!(app.controller.prompt().starts_with("Slab height"),"{}",app.controller.prompt());
    click_in_view(&mut app,&ctx,ViewId::SideView,[0.,0.,3.],Modifiers::NONE);
    let result=app.controller.document().entities.values().find(|e|e.id!=id).expect("Slab output");
    let Geometry::Brep(body)=&result.geometry else{panic!("BRep")};
    let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();
    assert!((m.volume_mm3-24.).abs()<2.,"{m:?}");
}

#[test]
fn viewport_sphere_aroundcurve_selects_curve_and_center() {
    let mut doc = Document::new("viewport sphere around curve");
    let rail = Entity::new("rail", Geometry::Curve(om9_core::curve_build::polyline(&[[-10., 0., 0.], [10., 0., 0.]]).unwrap()), None);
    let id = rail.id;
    doc.entities.insert(id, rail);
    let mut app = MatrixApp::new(doc).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
    app.controller.submit("Sphere AroundCurve").unwrap();
    click(&mut app, &ctx, [2., 0., 0.]);
    assert!(app.controller.prompt().starts_with("Pick center"), "{}", app.controller.prompt());
    click(&mut app, &ctx, [4., 0., 0.]);
    assert!(app.controller.prompt().starts_with("Radius"), "{}", app.controller.prompt());
    app.controller.submit("3").unwrap();
    let sphere = app.controller.document().entities.values().find(|e| e.id != id).unwrap();
    let center = sphere.placement.point([0., 0., 0.]).unwrap();
    assert!((center[0] - 4.).abs() < 0.2 && center[1].abs() < 1e-7, "{center:?}");
}

#[test]
fn viewport_sphere_three_point_radius_selects_direction() {
    let mut app=MatrixApp::new(Document::new("viewport sphere chord")).unwrap();let ctx=egui::Context::default();
    frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);let before=app.controller.document().clone();
    app.controller.submit("Sphere 3Point").unwrap();click(&mut app,&ctx,[0.,0.,0.]);click(&mut app,&ctx,[4.,0.,0.]);
    app.controller.submit("Radius=3").unwrap();assert!(app.controller.document().entities.is_empty());
    click(&mut app,&ctx,[2.,-5.,0.]);let e=app.controller.document().entities.values().next().unwrap();
    let center=e.placement.point([0.,0.,0.]).unwrap();assert!((center[0]-2.).abs()<0.15 && (center[1]+5_f64.sqrt()).abs()<0.15,"{center:?}");
    app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_box_cube_uses_second_view_for_3d_corner() {
  for (view, requested) in [(ViewId::ThroughFinger,[4.,0.,4.]),(ViewId::SideView,[0.,4.,4.])] {
    let mut app=MatrixApp::new(Document::new("viewport cube")).unwrap();let ctx=egui::Context::default();
    frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);let before=app.controller.document().clone();
    app.controller.submit("Box").unwrap();app.controller.submit("Cube").unwrap();
    click(&mut app,&ctx,[0.,0.,0.]);
    click_in_view(&mut app,&ctx,view,requested,Modifiers::NONE);
    let e=app.controller.document().entities.values().next().expect("cube must commit at second corner");
    let Geometry::Brep(body)=&e.geometry else {panic!()};
    let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();
    let side=(32_f64/3.).sqrt();let expected=side.powi(3);assert!((m.volume_mm3-expected).abs()<3.,"{m:?}");
    let target=e.placement.point([side,side,side]).unwrap();assert!((0..3).all(|i|(target[i]-requested[i]).abs()<0.2),"{view:?} {target:?}");
    app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
  }
}

#[test]
fn viewport_box_vertical_picks_width_and_height_on_distinct_axes() {
    let mut app=MatrixApp::new(Document::new("viewport vertical box")).unwrap();let ctx=egui::Context::default();
    frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);let before=app.controller.document().clone();
    app.controller.submit("Box").unwrap();app.controller.submit("Vertical").unwrap();
    click(&mut app,&ctx,[0.,0.,0.]);click(&mut app,&ctx,[4.,0.,0.]);
    click_in_view(&mut app,&ctx,ViewId::ThroughFinger,[0.,0.,3.],Modifiers::NONE);
    assert!(app.controller.prompt().starts_with("Height"),"{}",app.controller.prompt());
    click(&mut app,&ctx,[0.,-5.,0.]);
    let e=app.controller.document().entities.values().next().unwrap();let Geometry::Brep(body)=&e.geometry else{panic!()};
    let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();assert!((m.volume_mm3-60.).abs()<8.,"{m:?}");
    app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_box_negative_height_stays_below_base() {
    let mut app=MatrixApp::new(Document::new("viewport negative box")).unwrap();let ctx=egui::Context::default();
    frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);let before=app.controller.document().clone();
    app.controller.submit("Box").unwrap();click(&mut app,&ctx,[0.,0.,0.]);click(&mut app,&ctx,[4.,3.,0.]);
    click_in_view(&mut app,&ctx,ViewId::SideView,[0.,0.,-5.],Modifiers::NONE);
    let e=app.controller.document().entities.values().next().unwrap();let Geometry::Brep(body)=&e.geometry else{panic!()};
    let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();let origin=e.placement.point([0.,0.,0.]).unwrap();
    assert!(origin[2] < -4.5 && (m.volume_mm3-60.).abs()<8.,"{origin:?} {m:?}");
    app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_pyramid_around_curve_selects_source_and_center() {
    let rail=Entity::new("rail",Geometry::Curve(Curve::Line{start:[0.,0.,0.],end:[10.,0.,0.],domain:[0.,1.]}),None);let id=rail.id;
    let mut doc=Document::new("viewport pyramid around");doc.entities.insert(id,rail);let mut app=MatrixApp::new(doc).unwrap();let ctx=egui::Context::default();
    frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);let before=app.controller.document().clone();
    app.controller.submit("Pyramid AroundCurve").unwrap();click(&mut app,&ctx,[5.,0.,0.]);click(&mut app,&ctx,[5.,0.,0.]);
    app.controller.submit("3").unwrap();app.controller.submit("5").unwrap();
    let e=app.controller.document().entities.values().find(|e|e.id!=id).unwrap();let a=e.placement.point([0.,0.,0.]).unwrap();let b=e.placement.point([0.,0.,5.]).unwrap();
    assert!((a[0]-5.).abs()<0.1 && (b[0]-a[0]-5.).abs()<1e-7);app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_pyramid_open_base() {
    let mut app=MatrixApp::new(Document::new("viewport pyramid open")).unwrap();let ctx=egui::Context::default();
    frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);let before=app.controller.document().clone();app.controller.submit("Pyramid").unwrap();app.controller.submit("Solid=No").unwrap();
    click(&mut app,&ctx,[0.,0.,0.]);click(&mut app,&ctx,[2.,0.,0.]);click_in_view(&mut app,&ctx,ViewId::SideView,[0.,0.,6.],Modifiers::NONE);
    let e=app.controller.document().entities.values().next().unwrap();let Geometry::Brep(body)=&e.geometry else{panic!("pyramid BRep")};
    let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();assert!(!m.closed && m.faces==4 && m.naked_edges==4,"{m:?}");
    app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_pyramid_center_corner_and_side_apex() {
    let mut app=MatrixApp::new(Document::new("viewport pyramid")).unwrap();let ctx=egui::Context::default();
    frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);let before=app.controller.document().clone();app.controller.submit("Pyramid").unwrap();
    click(&mut app,&ctx,[0.,0.,0.]);click(&mut app,&ctx,[2.,0.,0.]);assert_eq!(app.controller.prompt(),"Apex height or point");
    click_in_view(&mut app,&ctx,ViewId::SideView,[0.,0.,6.],Modifiers::NONE);
    let e=app.controller.document().entities.values().next().unwrap();let Geometry::Brep(body)=&e.geometry else{panic!("pyramid BRep")};
    let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();assert!(m.closed && (m.volume_mm3-16.).abs()<2.,"{m:?}");
    app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_tube_area_sets_base_before_height() {
    let mut app=MatrixApp::new(Document::new("viewport tube area")).unwrap();let ctx=egui::Context::default();
    frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);let before=app.controller.document().clone();app.controller.submit("Tube").unwrap();
    click(&mut app,&ctx,[0.,0.,0.]);app.controller.submit(&format!("Area={}",9.*std::f64::consts::PI)).unwrap();click(&mut app,&ctx,[1.,0.,0.]);
    click_in_view(&mut app,&ctx,ViewId::SideView,[0.,0.,5.],Modifiers::NONE);
    let e=app.controller.document().entities.values().next().unwrap();let Geometry::Brep(body)=&e.geometry else{panic!()};
    let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();assert!((m.volume_mm3-40.*std::f64::consts::PI).abs()<6.,"{m:?}");
    app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_tube_wall_thickness_pick() {
    let mut app=MatrixApp::new(Document::new("viewport tube wall")).unwrap();let ctx=egui::Context::default();
    frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);let before=app.controller.document().clone();app.controller.submit("Tube").unwrap();
    click(&mut app,&ctx,[0.,0.,0.]);click(&mut app,&ctx,[3.,0.,0.]);app.controller.submit("WallThickness").unwrap();click(&mut app,&ctx,[1.,0.,0.]);
    assert_eq!(app.controller.prompt(),"Height or point");click_in_view(&mut app,&ctx,ViewId::SideView,[0.,0.,5.],Modifiers::NONE);
    let e=app.controller.document().entities.values().next().unwrap();let Geometry::Brep(body)=&e.geometry else{panic!()};
    let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();assert!((m.volume_mm3-40.*std::f64::consts::PI).abs()<6.,"{m:?}");
    app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_tube_both_sides_height() {
    let mut app=MatrixApp::new(Document::new("viewport tube both sides")).unwrap();let ctx=egui::Context::default();
    frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);let before=app.controller.document().clone();app.controller.submit("Tube").unwrap();app.controller.submit("BothSides=Yes").unwrap();
    click(&mut app,&ctx,[0.,0.,0.]);click(&mut app,&ctx,[3.,0.,0.]);click(&mut app,&ctx,[1.,0.,0.]);
    click_in_view(&mut app,&ctx,ViewId::SideView,[0.,0.,5.],Modifiers::NONE);
    let e=app.controller.document().entities.values().next().unwrap();let Geometry::Brep(body)=&e.geometry else{panic!()};
    let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();assert!((m.volume_mm3-80.*std::f64::consts::PI).abs()<10.,"{m:?}");
    assert!((e.placement.point([0.,0.,0.]).unwrap()[2]+5.).abs()<0.1);app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_tube_tangent_point_selects_curve_and_contact() {
    let rail=Entity::new("rail",Geometry::Curve(Curve::Line{start:[-5.,0.,0.],end:[5.,0.,0.],domain:[0.,1.]}),None);let id=rail.id;
    let mut doc=Document::new("viewport tangent tube");doc.entities.insert(id,rail);let mut app=MatrixApp::new(doc).unwrap();let ctx=egui::Context::default();
    frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);let before=app.controller.document().clone();app.controller.submit("Tube Tangent").unwrap();
    click(&mut app,&ctx,[0.,0.,0.]);click(&mut app,&ctx,[0.,0.,0.]);assert_eq!(app.controller.prompt(),"Choose Point for a non-tangent constraint");
    app.controller.submit("Point").unwrap();click(&mut app,&ctx,[3.,2.,0.]);app.controller.submit("1").unwrap();app.controller.submit("5").unwrap();
    let e=app.controller.document().entities.values().find(|e|e.id!=id).unwrap();let Geometry::Brep(body)=&e.geometry else{panic!("tube BRep")};
    let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();assert!((m.volume_mm3-std::f64::consts::PI*(3.25_f64.powi(2)-1.)*5.).abs()<8.,"{m:?}");
    app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_tube_around_curve_selects_source_and_center() {
    let rail=Entity::new("rail",Geometry::Curve(Curve::Line{start:[0.,0.,0.],end:[10.,0.,0.],domain:[0.,1.]}),None);let id=rail.id;
    let mut doc=Document::new("viewport tube around");doc.entities.insert(id,rail);let mut app=MatrixApp::new(doc).unwrap();let ctx=egui::Context::default();
    frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);let before=app.controller.document().clone();
    app.controller.submit("Tube AroundCurve").unwrap();click(&mut app,&ctx,[5.,0.,0.]);click(&mut app,&ctx,[5.,0.,0.]);
    app.controller.submit("3").unwrap();app.controller.submit("1").unwrap();app.controller.submit("5").unwrap();
    let e=app.controller.document().entities.values().find(|e|e.id!=id).unwrap();let a=e.placement.point([0.,0.,0.]).unwrap();let b=e.placement.point([0.,0.,5.]).unwrap();
    assert!((a[0]-5.).abs()<0.1 && (b[0]-a[0]-5.).abs()<1e-7);app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_tube_fit_points_and_undo() {
    let mut app=MatrixApp::new(Document::new("viewport tube fit")).unwrap();let ctx=egui::Context::default();
    frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);let before=app.controller.document().clone();app.controller.submit("Tube FitPoints").unwrap();
    for p in [[3.,0.,0.],[0.,3.,0.],[-3.,0.,0.],[0.,-3.,0.]] {click(&mut app,&ctx,p);}
    app.controller.submit("").unwrap();assert_eq!(app.controller.prompt(),"Second radius or point");app.controller.submit("1").unwrap();app.controller.submit("5").unwrap();
    let e=app.controller.document().entities.values().next().unwrap();let Geometry::Brep(body)=&e.geometry else{panic!("tube BRep")};
    let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();assert!(m.closed && (m.volume_mm3-40.*std::f64::consts::PI).abs()<6.,"{m:?}");
    app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_tube_three_point_radius_selects_side() {
    let mut app=MatrixApp::new(Document::new("viewport tube radius")).unwrap();let ctx=egui::Context::default();
    frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);let before=app.controller.document().clone();
    app.controller.submit("Tube 3Point").unwrap();click(&mut app,&ctx,[0.,0.,0.]);click(&mut app,&ctx,[4.,0.,0.]);
    app.controller.submit("Radius=3").unwrap();click(&mut app,&ctx,[2.,2.,0.]);app.controller.submit("1").unwrap();app.controller.submit("5").unwrap();
    let e=app.controller.document().entities.values().next().unwrap();let center=e.placement.point([0.,0.,0.]).unwrap();assert!((center[1]-5.0_f64.sqrt()).abs()<0.1);
    app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_tube_center_radius_and_height() {
    let mut app=MatrixApp::new(Document::new("viewport tube")).unwrap();let ctx=egui::Context::default();
    frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);let before=app.controller.document().clone();
    app.controller.submit("Tube").unwrap();click(&mut app,&ctx,[0.,0.,0.]);click(&mut app,&ctx,[3.,0.,0.]);click(&mut app,&ctx,[1.,0.,0.]);
    assert_eq!(app.controller.prompt(),"Height or point");assert_eq!(app.controller.solid_preview([0.,0.,5.]).len(),264);assert_eq!(app.controller.document(),&before);
    click_in_view(&mut app,&ctx,ViewId::SideView,[0.,0.,5.],Modifiers::NONE);
    let e=app.controller.document().entities.values().next().unwrap();let Geometry::Brep(body)=&e.geometry else{panic!("tube BRep")};
    let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();assert!(m.closed && (m.volume_mm3-40.*std::f64::consts::PI).abs()<5.,"{m:?}");
    app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_cylinder_center_radius_and_side_height() {
    let mut app=MatrixApp::new(Document::new("viewport cylinder")).unwrap();let ctx=egui::Context::default();
    frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);let before=app.controller.document().clone();
    app.controller.submit("Cylinder").unwrap();click(&mut app,&ctx,[0.,0.,0.]);click(&mut app,&ctx,[3.,0.,0.]);
    assert_eq!(app.controller.prompt(),"Height or point");
    assert_eq!(app.controller.solid_preview([0.,0.,5.]).len(),132);assert_eq!(app.controller.document(),&before);
    click_in_view(&mut app,&ctx,ViewId::SideView,[0.,0.,5.],Modifiers::NONE);
    let entity=app.controller.document().entities.values().next().unwrap();let Geometry::Brep(body)=&entity.geometry else {panic!("cylinder BRep")};
    let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();assert!((m.volume_mm3-45.*std::f64::consts::PI).abs()<5.,"{m:?}");
    app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_cylinder_three_point_radius_and_height() {
    let mut app=MatrixApp::new(Document::new("viewport cylinder 3point radius")).unwrap();let ctx=egui::Context::default();
    frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);let before=app.controller.document().clone();
    app.controller.submit("Cylinder 3Point").unwrap();click(&mut app,&ctx,[-2.,0.,0.]);click(&mut app,&ctx,[2.,0.,0.]);
    app.controller.submit("Radius=3").unwrap();click(&mut app,&ctx,[0.,2.,0.]);assert_eq!(app.controller.prompt(),"Height or point");
    click_in_view(&mut app,&ctx,ViewId::SideView,[0.,5.0_f64.sqrt(),5.],Modifiers::NONE);
    let entity=app.controller.document().entities.values().next().unwrap();let Geometry::Brep(body)=&entity.geometry else{panic!("cylinder BRep")};
    let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();assert!((m.volume_mm3-45.*std::f64::consts::PI).abs()<6.,"{m:?}");
    app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_cylinder_around_curve_selects_source_and_center() {
    let rail=Entity::new("rail",Geometry::Curve(Curve::Line{start:[-5.,0.,0.],end:[5.,0.,0.],domain:[0.,1.]}),None);let id=rail.id;
    let mut doc=Document::new("viewport cylinder around");doc.entities.insert(id,rail);let mut app=MatrixApp::new(doc).unwrap();let ctx=egui::Context::default();
    frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);let before=app.controller.document().clone();
    app.controller.submit("Cylinder AroundCurve").unwrap();click(&mut app,&ctx,[0.,0.,0.]);assert_eq!(app.controller.prompt(),"Pick center on curve or OnCurve=");
    click(&mut app,&ctx,[0.,0.,0.]);assert_eq!(app.controller.prompt(),"Base radius or point");app.controller.submit("2").unwrap();app.controller.submit("5").unwrap();
    let entity=app.controller.document().entities.values().find(|e|e.id!=id).unwrap();let a=entity.placement.point([0.,0.,0.]).unwrap();let b=entity.placement.point([0.,0.,5.]).unwrap();
    assert!((b[0]-a[0]-5.).abs()<1e-7);app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_cylinder_fit_points_and_undo() {
    let mut app=MatrixApp::new(Document::new("viewport cylinder fit")).unwrap();let ctx=egui::Context::default();
    frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);let before=app.controller.document().clone();app.controller.submit("Cylinder FitPoints").unwrap();
    for p in [[3.,0.,0.],[0.,3.,0.],[-3.,0.,0.],[0.,-3.,0.]] {click(&mut app,&ctx,p);}
    app.controller.submit("").unwrap();assert_eq!(app.controller.prompt(),"Height or point");app.controller.submit("5").unwrap();
    let entity=app.controller.document().entities.values().next().unwrap();let Geometry::Brep(body)=&entity.geometry else{panic!("cylinder BRep")};
    let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();assert!((m.volume_mm3-45.*std::f64::consts::PI).abs()<6.,"{m:?}");
    app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_cylinder_tangent_point_selects_curve_and_contact() {
    let rail=Entity::new("rail",Geometry::Curve(Curve::Line{start:[-5.,0.,0.],end:[5.,0.,0.],domain:[0.,1.]}),None);let id=rail.id;
    let mut doc=Document::new("viewport cylinder tangent");doc.entities.insert(id,rail);let mut app=MatrixApp::new(doc).unwrap();let ctx=egui::Context::default();
    frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);let before=app.controller.document().clone();
    app.controller.submit("Cylinder Tangent").unwrap();click(&mut app,&ctx,[0.,0.,0.]);click(&mut app,&ctx,[0.,0.,0.]);
    assert_eq!(app.controller.prompt(),"Choose Point for a non-tangent constraint");app.controller.submit("Point").unwrap();click(&mut app,&ctx,[3.,2.,0.]);app.controller.submit("5").unwrap();
    let entity=app.controller.document().entities.values().find(|e|e.id!=id).unwrap();let Geometry::Brep(body)=&entity.geometry else{panic!("cylinder BRep")};
    let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();assert!((m.volume_mm3-std::f64::consts::PI*3.25_f64.powi(2)*5.).abs()<8.,"{m:?}");
    app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_truncated_cone_tangent_point_selects_rail_and_contact() {
    let curve=Curve::Line {start:[-5.,0.,0.],end:[5.,0.,0.],domain:[0.,1.]};
    let rail=Entity::new("rail",Geometry::Curve(curve),None); let id=rail.id; let mut doc=Document::new("viewport tangent cone"); doc.entities.insert(id,rail);
    let mut app=MatrixApp::new(doc).unwrap(); let ctx=egui::Context::default(); frame(&mut app,&ctx,vec![]); frame(&mut app,&ctx,vec![]);
    let before=app.controller.document().clone(); app.controller.submit("TCone").unwrap(); app.controller.submit("Tangent").unwrap();
    click(&mut app,&ctx,[0.,0.,0.]); assert_eq!(app.controller.prompt(),"Pick tangent location or OnCurve=");
    click(&mut app,&ctx,[0.,0.,0.]); app.controller.submit("Point").unwrap(); click(&mut app,&ctx,[3.,2.,0.]);
    app.controller.submit("5").unwrap(); app.controller.submit("1").unwrap();
    let entity=app.controller.document().entities.values().find(|e|e.id!=id).unwrap(); let center=entity.placement.point([0.,0.,0.]).unwrap();
    assert!(center[0].abs()<0.15 && (center[1]-3.25).abs()<0.2);
    app.controller.undo().unwrap(); assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_truncated_cone_around_curve_selects_rail_and_center() {
    let curve=Curve::Line {start:[-5.,0.,0.],end:[5.,0.,0.],domain:[0.,1.]};
    let rail=Entity::new("rail",Geometry::Curve(curve),None); let id=rail.id; let mut doc=Document::new("viewport around curve"); doc.entities.insert(id,rail);
    let mut app=MatrixApp::new(doc).unwrap(); let ctx=egui::Context::default(); frame(&mut app,&ctx,vec![]); frame(&mut app,&ctx,vec![]);
    let before=app.controller.document().clone(); app.controller.submit("TCone").unwrap(); app.controller.submit("AroundCurve").unwrap();
    click(&mut app,&ctx,[0.,0.,0.]); assert_eq!(app.controller.prompt(),"Pick center on curve or OnCurve=");
    click(&mut app,&ctx,[0.,0.,0.]); assert_eq!(app.controller.prompt(),"Base radius or point");
    app.controller.submit("3").unwrap(); app.controller.submit("5").unwrap(); app.controller.submit("1").unwrap();
    let entity=app.controller.document().entities.values().find(|e|e.id!=id).unwrap(); let Geometry::Brep(body)=&entity.geometry else {panic!()};
    let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap(); assert!((m.volume_mm3-65.*std::f64::consts::PI/3.).abs()<1e-6);
    let a=entity.placement.point([0.,0.,0.]).unwrap(); let b=entity.placement.point([0.,0.,5.]).unwrap();
    assert!(a[0].abs()<0.1 && a[1].abs()<0.1 && (b[0]-a[0]-5.).abs()<1e-6);
    app.controller.undo().unwrap(); assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_truncated_cone_fit_points_and_undo() {
    let mut app=MatrixApp::new(Document::new("viewport fit cone")).unwrap();
    let ctx=egui::Context::default(); frame(&mut app,&ctx,vec![]); frame(&mut app,&ctx,vec![]);
    let before=app.controller.document().clone(); app.controller.submit("TCone").unwrap(); app.controller.submit("FitPoints").unwrap();
    for point in [[6.,7.,0.],[3.,10.,0.],[0.,7.,0.],[3.,4.,0.]] {click(&mut app,&ctx,point);}
    app.controller.submit("").unwrap(); app.controller.submit("5").unwrap(); app.controller.submit("1").unwrap();
    let Geometry::Brep(body)=&app.controller.document().entities.values().next().unwrap().geometry else {panic!()};
    let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap(); assert!((m.volume_mm3-65.*std::f64::consts::PI/3.).abs()<1.5);
    app.controller.undo().unwrap(); assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_truncated_cone_solid_no_commits_open_face() {
    let mut app=MatrixApp::new(Document::new("viewport cone open")).unwrap();
    let ctx=egui::Context::default(); frame(&mut app,&ctx,vec![]); frame(&mut app,&ctx,vec![]);
    let before=app.controller.document().clone(); app.controller.submit("TCone").unwrap(); app.controller.submit("Solid=No").unwrap();
    click(&mut app,&ctx,[3.,7.,0.]); click(&mut app,&ctx,[6.,7.,0.]); app.controller.submit("5").unwrap(); click(&mut app,&ctx,[4.,7.,0.]);
    let Geometry::Brep(body)=&app.controller.document().entities.values().next().unwrap().geometry else {panic!()};
    let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();
    assert_eq!((m.solids,m.faces,m.naked_edges),(0,1,2)); assert!(!m.closed);
    app.controller.undo().unwrap(); assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_truncated_cone_three_point_radius_selects_side() {
    let mut app=MatrixApp::new(Document::new("viewport 3Point radius cone")).unwrap();
    let ctx=egui::Context::default(); frame(&mut app,&ctx,vec![]); frame(&mut app,&ctx,vec![]);
    let before=app.controller.document().clone(); app.controller.submit("TCone").unwrap(); app.controller.submit("3Point").unwrap();
    click(&mut app,&ctx,[0.,0.,0.]); click(&mut app,&ctx,[4.,0.,0.]);
    app.controller.submit("Radius=3").unwrap(); click(&mut app,&ctx,[2.,2.,0.]);
    assert_eq!(app.controller.prompt(),"Height or point"); app.controller.submit("5").unwrap(); app.controller.submit("1").unwrap();
    let entity=app.controller.document().entities.values().next().unwrap();
    let center=entity.placement.point([0.,0.,0.]).unwrap(); assert!((center[0]-2.).abs()<0.1 && (center[1]-5.0_f64.sqrt()).abs()<0.1);
    app.controller.undo().unwrap(); assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_truncated_cone_three_point_base_and_undo() {
    let mut app=MatrixApp::new(Document::new("viewport 3Point cone")).unwrap();
    let ctx=egui::Context::default(); frame(&mut app,&ctx,vec![]); frame(&mut app,&ctx,vec![]);
    let before=app.controller.document().clone(); app.controller.submit("TCone").unwrap(); app.controller.submit("3Point").unwrap();
    click(&mut app,&ctx,[6.,7.,0.]); click(&mut app,&ctx,[3.,10.,0.]); click(&mut app,&ctx,[0.,7.,0.]);
    assert_eq!(app.controller.prompt(),"Height or point"); app.controller.submit("5").unwrap(); click(&mut app,&ctx,[4.,7.,0.]);
    let entity=app.controller.document().entities.values().next().unwrap(); let Geometry::Brep(body)=&entity.geometry else {panic!()};
    let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();
    assert!((m.volume_mm3-65.*std::f64::consts::PI/3.).abs()<1.5,"{m:?}");
    app.controller.undo().unwrap(); assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_truncated_cone_two_point_and_mouse_height() {
    let mut app=MatrixApp::new(Document::new("viewport 2Point cone")).unwrap();
    let ctx=egui::Context::default(); frame(&mut app,&ctx,vec![]); frame(&mut app,&ctx,vec![]);
    let before=app.controller.document().clone(); app.controller.dispatch("TruncatedCone2Point",InvocationOrigin::Menu).unwrap();
    click(&mut app,&ctx,[0.,7.,0.]); click(&mut app,&ctx,[6.,7.,0.]);
    assert_eq!(app.controller.prompt(),"Height or point");
    click_in_view(&mut app,&ctx,ViewId::SideView,[3.,7.,5.],Modifiers::NONE);
    assert_eq!(app.controller.prompt(),"Top radius or point");
    click(&mut app,&ctx,[4.,7.,0.]);
    let entity=app.controller.document().entities.values().next().unwrap();
    let Geometry::Brep(body)=&entity.geometry else {panic!()};
    let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();
    assert!((m.volume_mm3-65.*std::f64::consts::PI/3.).abs()<1.2,"{m:?}");
    app.controller.undo().unwrap(); assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_truncated_cone_picks_base_and_top_and_undoes() {
    let mut app=MatrixApp::new(Document::new("viewport truncated cone")).unwrap();
    let ctx=egui::Context::default(); frame(&mut app,&ctx,vec![]); frame(&mut app,&ctx,vec![]);
    let before=app.controller.document().clone(); app.controller.dispatch("OM9-SOLID-020",InvocationOrigin::Menu).unwrap();
    click(&mut app,&ctx,[3.,7.,0.]); click(&mut app,&ctx,[6.,7.,0.]);
    app.controller.submit("5").unwrap(); click(&mut app,&ctx,[4.,7.,0.]);
    let entity=app.controller.document().entities.values().next().unwrap();
    let Geometry::Brep(body)=&entity.geometry else {panic!()};
    let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();
    assert!((m.volume_mm3-65.*std::f64::consts::PI/3.).abs()<1.0,"{m:?}");
    app.controller.undo().unwrap(); assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_pipe_bent_chain_picks_endpoint_radius_points() {
    let mut doc = Document::new("Bent Pipe point input");
    let body = om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Box { x_mm: 5.0, y_mm: 5.0, z_mm: 5.0 }, 0.01, &Limits::default()).unwrap();
    let mut entity = Entity::new("box", Geometry::Brep(body), None);
    entity.placement = Transform::translation([0.0, 0.0, -5.0]);
    doc.entities.insert(entity.id, entity);
    let mut app = MatrixApp::new(doc).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
    let before = app.controller.document().clone();
    app.controller.submit("_Pipe _C").unwrap();
    click(&mut app, &ctx, [2.5, 0.0, 0.0]);
    click(&mut app, &ctx, [5.0, 2.5, 0.0]);
    app.controller.submit("").unwrap();
    assert_eq!(app.controller.solid_preview([0.0,-1.0,0.0]).len(),64);
    let preview=app.controller.solid_preview([0.0,-1.0,0.0]);
    for _ in 0..10 {assert_eq!(app.controller.solid_preview([0.0,-1.0,0.0]),preview);}
    assert_eq!(app.controller.document(),&before); assert_eq!(app.controller.coordinator().undo_len(),0);
    click(&mut app, &ctx, [0.0, -1.0, 0.0]);
    click(&mut app, &ctx, [6.0, 5.0, 0.0]);
    app.controller.submit("").unwrap();
    let result = app.controller.document().entities.values().find(|e| !before.entities.contains_key(&e.id)).unwrap();
    let Geometry::Brep(brep) = &result.geometry else { panic!() };
    let m = om9_kernel::creation::measure(brep, app.controller.coordinator().limits()).unwrap();
    assert!((m.volume_mm3 - 10.0 * std::f64::consts::PI).abs() < 0.2, "{m:?}");
    assert_eq!(app.controller.coordinator().undo_len(), 1);
    app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
}

#[test]
fn viewport_pipe_chain_cached_edge_rejects_deleted_source() {
    let mut doc=Document::new("stale pipe chain edge"); let body=om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Box {x_mm:5.,y_mm:5.,z_mm:5.},0.01,&Limits::default()).unwrap();
    let mut entity=Entity::new("box",Geometry::Brep(body),None); entity.placement=Transform::translation([0.,0.,-5.]); let id=entity.id; doc.entities.insert(id,entity);
    let mut app=MatrixApp::new(doc).unwrap(); let ctx=egui::Context::default(); frame(&mut app,&ctx,vec![]); frame(&mut app,&ctx,vec![]);
    app.controller.submit("_Pipe _C").unwrap(); click(&mut app,&ctx,[2.5,0.,0.]); click(&mut app,&ctx,[5.,2.5,0.]); app.controller.submit("").unwrap();
    assert_eq!(app.controller.solid_preview([0.,-1.,0.]).len(),64);
    assert_eq!(app.controller.delete_objects(&[id]).unwrap(),1); let after=app.controller.document().clone(); let undo_count=app.controller.coordinator().undo_len();
    assert!(app.controller.solid_preview([0.,-1.,0.]).is_empty()); assert!(app.controller.accept_world_point([0.,-1.,0.]).is_err());
    assert_eq!(app.controller.document(),&after); assert_eq!(app.controller.coordinator().undo_len(),undo_count);
}

#[test]
fn viewport_pipe_bent_chain_round_caps_thin_and_thick() {
    for thick in [false, true] {
        let mut doc = Document::new("Bent round Pipe");
        let body = om9_kernel::creation::primitive(
            om9_kernel::creation::Primitive::Box { x_mm: 5.0, y_mm: 5.0, z_mm: 5.0 }, 0.01, &Limits::default()).unwrap();
        let entity = Entity::new("box", Geometry::Brep(body), None);
        doc.entities.insert(entity.id, entity);
        let mut app = MatrixApp::new(doc).unwrap();
        let ctx = egui::Context::default();
        frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
        let before = app.controller.document().clone();
        app.controller.submit("_Pipe _C").unwrap();
        click(&mut app, &ctx, [2.5, 0.0, 5.0]);
        click(&mut app, &ctx, [5.0, 2.5, 5.0]);
        app.controller.submit("").unwrap();
        app.controller.submit("Cap=Round").unwrap();
        if thick {
            app.controller.submit("Thick=Yes").unwrap();
            for radius in [0.5, 1.0, 0.5, 1.0] { app.controller.submit(&radius.to_string()).unwrap(); }
        } else { app.controller.submit("1").unwrap(); app.controller.submit("1").unwrap(); }
        app.controller.submit("").unwrap();
        let result = app.controller.document().entities.values().find(|e| !before.entities.contains_key(&e.id)).unwrap();
        let Geometry::Brep(brep) = &result.geometry else { panic!() };
        let m = om9_kernel::creation::measure(brep, app.controller.coordinator().limits()).unwrap();
        let expected = if thick { (7.5 + 7.0 / 6.0) * std::f64::consts::PI }
            else { (10.0 + 4.0 / 3.0) * std::f64::consts::PI };
        assert!(m.closed && m.solids == 1);
        assert!((m.volume_mm3 - expected).abs() < 1e-5, "{} {expected}", m.volume_mm3);
        assert_eq!(app.controller.coordinator().undo_len(), 1);
        app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
    }
}

#[test]
fn viewport_pipe_thick_bent_brep_chain_makes_hollow_walls() {
    for capped in [false, true] {
        let mut doc = Document::new("Bent Thick Pipe");
        let body = om9_kernel::creation::primitive(
            om9_kernel::creation::Primitive::Box { x_mm: 5.0, y_mm: 5.0, z_mm: 5.0 }, 0.01, &Limits::default()).unwrap();
        let entity = Entity::new("box", Geometry::Brep(body), None);
        doc.entities.insert(entity.id, entity);
        let mut app = MatrixApp::new(doc).unwrap();
        let ctx = egui::Context::default();
        frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
        let before = app.controller.document().clone();
        app.controller.submit("_Pipe _C").unwrap();
        click(&mut app, &ctx, [2.5, 0.0, 5.0]);
        click(&mut app, &ctx, [5.0, 2.5, 5.0]);
        app.controller.submit("").unwrap();
        app.controller.submit("Thick=Yes").unwrap();
        if !capped { app.controller.submit("Cap=None").unwrap(); }
        for radius in [0.5, 1.0, 0.5, 1.0] { app.controller.submit(&radius.to_string()).unwrap(); }
        app.controller.submit("").unwrap();
        let result = app.controller.document().entities.values().find(|e| !before.entities.contains_key(&e.id)).unwrap();
        let Geometry::Brep(brep) = &result.geometry else { panic!() };
        let m = om9_kernel::creation::measure(brep, app.controller.coordinator().limits()).unwrap();
        assert_eq!(m.closed, capped);
        if capped { assert!((m.volume_mm3 - 7.5 * std::f64::consts::PI).abs() < 1e-5, "{m:?}"); }
        else {
            assert_eq!(m.solids, 0);
            assert!((m.area_mm2 - 30.0 * std::f64::consts::PI).abs() < 1e-4, "{m:?}");
            let mut signed = [0.0_f64; 2];
            for [ia, ib, ic, _] in &brep.display().triangles {
                let a = brep.display().points[*ia as usize]; let b = brep.display().points[*ib as usize]; let c = brep.display().points[*ic as usize];
                let center = std::array::from_fn::<_, 3, _>(|i| (a[i] + b[i] + c[i]) / 3.0);
                let u = std::array::from_fn::<_, 3, _>(|i| b[i] - a[i]);
                let v = std::array::from_fn::<_, 3, _>(|i| c[i] - a[i]);
                let n = [u[1]*v[2]-u[2]*v[1], u[2]*v[0]-u[0]*v[2], u[0]*v[1]-u[1]*v[0]];
                let p1 = [center[0].clamp(0.0, 5.0), 0.0, 5.0];
                let p2 = [5.0, center[1].clamp(0.0, 5.0), 5.0];
                let r1 = std::array::from_fn::<_, 3, _>(|i| center[i]-p1[i]);
                let r2 = std::array::from_fn::<_, 3, _>(|i| center[i]-p2[i]);
                let d = |r: [f64; 3]| r.iter().map(|x| x*x).sum::<f64>();
                let radial = if d(r1) < d(r2) { r1 } else { r2 };
                let radius = d(radial).sqrt();
                if radius > 0.2 && radius < 1.1 {
                    let which = usize::from(radius > 0.7);
                    signed[which] += n[0]*radial[0] + n[1]*radial[1] + n[2]*radial[2];
                }
            }
            assert!(signed[0] < 0.0 && signed[1] > 0.0, "wall orientation {signed:?}");
        }
        assert_eq!(app.controller.coordinator().undo_len(), 1);
        app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
    }
}

#[test]
fn viewport_pipe_chain_sweeps_brep_edges_at_right_angle() {
    for capped in [false, true] {
        let mut doc = Document::new("Bent Pipe chain");
        let body = om9_kernel::creation::primitive(
            om9_kernel::creation::Primitive::Box { x_mm: 5.0, y_mm: 5.0, z_mm: 5.0 }, 0.01, &Limits::default()).unwrap();
        let entity = Entity::new("box", Geometry::Brep(body), None);
        doc.entities.insert(entity.id, entity);
        let mut app = MatrixApp::new(doc).unwrap();
        let ctx = egui::Context::default();
        frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
        let before = app.controller.document().clone();
        app.controller.submit("_Pipe _C").unwrap();
        click(&mut app, &ctx, [2.5, 0.0, 5.0]);
        click(&mut app, &ctx, [5.0, 2.5, 5.0]);
        assert_eq!(app.controller.subselection().len(), 2);
        app.controller.submit("").unwrap();
        if !capped { app.controller.submit("Cap=None").unwrap(); }
        app.controller.submit("1").unwrap(); app.controller.submit("1").unwrap(); app.controller.submit("").unwrap();
        let result = app.controller.document().entities.values().find(|e| !before.entities.contains_key(&e.id)).unwrap();
        let Geometry::Brep(brep) = &result.geometry else { panic!() };
        let m = om9_kernel::creation::measure(brep, app.controller.coordinator().limits()).unwrap();
        assert_eq!(m.closed, capped);
        if capped { assert!((m.volume_mm3 - 10.0 * std::f64::consts::PI).abs() < 1e-5, "{m:?}"); }
        assert_eq!(app.controller.coordinator().undo_len(), 1);
        app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
    }
}

#[test]
fn viewport_pipe_mixed_chain_accepts_endpoint_and_station_points() {
    // Exact B-rep snap refinement needs the same explicit tolerance as New.
    let mut doc = Document::with_template_tolerance("Pipe chain points");
    let line = Entity::new("line", Geometry::Curve(Curve::Line {
        start: [0.0, 0.0, 0.0], end: [5.0, 0.0, 0.0], domain: [0.0, 1.0],
    }), None);
    doc.entities.insert(line.id, line);
    let body = om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Box { x_mm: 5.0, y_mm: 5.0, z_mm: 5.0 }, 0.01, &Limits::default()).unwrap();
    let mut box_entity = Entity::new("box", Geometry::Brep(body), None);
    box_entity.placement = Transform::translation([5.0, 0.0, -5.0]);
    doc.entities.insert(box_entity.id, box_entity);
    let mut app = MatrixApp::new(doc).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
    let before = app.controller.document().clone();
    app.controller.submit("_Pipe _C").unwrap();
    click(&mut app, &ctx, [2.5, 0.0, 0.0]);
    click(&mut app, &ctx, [7.5, 0.0, 0.0]);
    app.controller.submit("").unwrap();
    app.controller.submit("ShapeBlending=Global").unwrap();
    click(&mut app, &ctx, [0.0, 1.0, 0.0]);
    assert!(app.controller.prompt().starts_with("End radius"), "{}", app.status_line());
    click(&mut app, &ctx, [10.0, 3.0, 0.0]);
    assert!(!app.controller.prompt().starts_with("End radius"), "{}", app.status_line());
    app.controller.submit("5,0,0").unwrap();
    let preview=app.controller.solid_preview([5.,2.,0.]); assert_eq!(preview.len(),64); assert!((preview[0][0][0]-5.).abs()<1e-6 && (preview[0][0][1]-2.).abs()<1e-6);
    app.controller.submit("2").unwrap();
    app.controller.submit("").unwrap();
    let result = app.controller.document().entities.values().find(|e| !before.entities.contains_key(&e.id)).unwrap();
    let Geometry::Brep(brep) = &result.geometry else { panic!() };
    let m = om9_kernel::creation::measure(brep, app.controller.coordinator().limits()).unwrap();
    assert!((m.volume_mm3 - 130.0 * std::f64::consts::PI / 3.0).abs() < 0.2, "{m:?}");
    assert!(app.controller.subselection().is_empty());
    assert_eq!(app.controller.coordinator().undo_len(), 1);
    app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
}

#[test]
fn viewport_pipe_chain_mixes_curve_and_brep_edge_in_selection_order() {
    for edge_first in [false, true] {
        let mut doc = Document::new("mixed Pipe chain");
        let line = Entity::new("line", Geometry::Curve(Curve::Line {
            start: [0.0, 0.0, 5.0], end: [5.0, 0.0, 5.0], domain: [0.0, 1.0],
        }), None);
        doc.entities.insert(line.id, line);
        let body = om9_kernel::creation::primitive(
            om9_kernel::creation::Primitive::Box { x_mm: 5.0, y_mm: 5.0, z_mm: 5.0 }, 0.01, &Limits::default()).unwrap();
        let mut box_entity = Entity::new("box", Geometry::Brep(body), None);
        box_entity.placement = Transform::translation([5.0, 0.0, 0.0]);
        doc.entities.insert(box_entity.id, box_entity);
        let mut app = MatrixApp::new(doc).unwrap();
        let ctx = egui::Context::default();
        frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
        let before = app.controller.document().clone();
        app.controller.submit("_Pipe _C").unwrap();
        let curve_point = [2.5, 0.0, 5.0];
        let edge_point = [7.5, 0.0, 5.0];
        let (first, last) = if edge_first { (edge_point, curve_point) } else { (curve_point, edge_point) };
        click(&mut app, &ctx, first); click(&mut app, &ctx, last);
        assert_eq!(app.controller.selected().len(), 1);
        assert_eq!(app.controller.subselection().len(), 1);
        app.controller.submit("Undo").unwrap();
        if edge_first { assert!(app.controller.selected().is_empty()); }
        else { assert!(app.controller.subselection().is_empty()); }
        click(&mut app, &ctx, last);
        app.controller.submit("").unwrap();
        app.controller.submit("1").unwrap(); app.controller.submit("1").unwrap(); app.controller.submit("").unwrap();
        let result = app.controller.document().entities.values().find(|e| !before.entities.contains_key(&e.id)).unwrap();
        let Geometry::Brep(brep) = &result.geometry else { panic!() };
        let m = om9_kernel::creation::measure(brep, app.controller.coordinator().limits()).unwrap();
        assert!((m.volume_mm3 - 10.0 * std::f64::consts::PI).abs() < 1e-5);
        assert_eq!(app.controller.coordinator().undo_len(), 1);
        app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
    }
}

#[test]
fn viewport_pipe_chain_selects_exact_brep_edges() {
    let limits = Limits::default();
    let mut doc = Document::new("Pipe B-Rep edge chain");
    for x in [0.0, 5.0] {
        let body = om9_kernel::creation::primitive(
            om9_kernel::creation::Primitive::Box { x_mm: 5.0, y_mm: 5.0, z_mm: 5.0 }, 0.01, &limits).unwrap();
        let mut entity = Entity::new("box", Geometry::Brep(body), None);
        entity.placement = Transform::translation([x, 0.0, 0.0]);
        doc.entities.insert(entity.id, entity);
    }
    let mut app = MatrixApp::new(doc).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
    let before = app.controller.document().clone();
    app.controller.submit("_Pipe _C").unwrap();
    click(&mut app, &ctx, [2.5, 0.0, 5.0]);
    assert_eq!(app.controller.subselection().len(), 1);
    app.controller.submit("Cancel").unwrap();
    assert!(app.controller.subselection().is_empty());
    app.controller.submit("_Pipe _C").unwrap();
    click(&mut app, &ctx, [2.5, 0.0, 5.0]);
    click(&mut app, &ctx, [7.5, 0.0, 5.0]);
    assert_eq!(app.controller.subselection().len(), 2);
    app.controller.submit("Undo").unwrap();
    assert_eq!(app.controller.subselection().len(), 1);
    click(&mut app, &ctx, [7.5, 0.0, 5.0]);
    app.controller.submit("").unwrap();
    app.controller.submit("1").unwrap(); app.controller.submit("1").unwrap(); app.controller.submit("").unwrap();
    let result = app.controller.document().entities.values().find(|e| !before.entities.contains_key(&e.id)).unwrap();
    let Geometry::Brep(brep) = &result.geometry else { panic!() };
    let m = om9_kernel::creation::measure(brep, app.controller.coordinator().limits()).unwrap();
    assert!((m.volume_mm3 - 10.0 * std::f64::consts::PI).abs() < 1e-5);
    assert!(app.controller.subselection().is_empty());
    assert_eq!(app.controller.coordinator().undo_len(), 1);
    app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
}

#[test]
fn viewport_pipe_chain_selects_connected_lines_in_order() {
    let mut doc = Document::new("Pipe chain");
    for (a, b) in [(5.0, 0.0), (10.0, 5.0)] {
        let rail = Entity::new("rail", Geometry::Curve(Curve::Line {
            start: [a, 0.0, 0.0], end: [b, 0.0, 0.0], domain: [0.0, 1.0],
        }), None);
        doc.entities.insert(rail.id, rail);
    }
    let mut app = MatrixApp::new(doc).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
    let before = app.controller.document().clone();
    app.controller.submit("_Pipe _C").unwrap();
    click(&mut app, &ctx, [2.5, 0.0, 0.0]);
    click(&mut app, &ctx, [7.5, 0.0, 0.0]);
    app.controller.submit("").unwrap();
    app.controller.submit("1").unwrap(); app.controller.submit("1").unwrap(); app.controller.submit("").unwrap();
    let brep = app.controller.document().entities.values().find_map(|e| match &e.geometry {
        Geometry::Brep(b) => Some(b), _ => None,
    }).unwrap();
    let m = om9_kernel::creation::measure(brep, app.controller.coordinator().limits()).unwrap();
    assert!((m.volume_mm3 - 10.0 * std::f64::consts::PI).abs() < 1e-5);
    assert_eq!(app.controller.coordinator().undo_len(), 1);
    app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
}

#[test]
fn viewport_pipe_round_intermediate_station_on_line() {
    let rail=Entity::new("rail",Geometry::Curve(Curve::Line {start:[0.,0.,0.],end:[10.,0.,0.],domain:[0.,1.]}),None); let id=rail.id;
    let mut doc=Document::new("viewport round station"); doc.entities.insert(id,rail); let mut app=MatrixApp::new(doc).unwrap(); let ctx=egui::Context::default();
    frame(&mut app,&ctx,vec![]); frame(&mut app,&ctx,vec![]); let before=app.controller.document().clone();
    app.controller.submit("Pipe").unwrap(); click(&mut app,&ctx,[5.,0.,0.]); app.controller.submit("Cap=Round").unwrap();
    app.controller.submit("1").unwrap(); app.controller.submit("3").unwrap(); click(&mut app,&ctx,[5.,0.,0.]);
    assert!(app.controller.prompt().contains("Intermediate radius")); app.controller.submit("4").unwrap(); app.controller.submit("").unwrap();
    let entity=app.controller.document().entities.values().find(|e|e.id!=id).unwrap(); let Geometry::Brep(body)=&entity.geometry else {panic!()};
    let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap(); let expected=std::f64::consts::PI*(10.*69./7.+56./3.);
    assert!(m.closed && (m.volume_mm3-expected).abs()<0.03*expected,"{m:?}"); app.controller.undo().unwrap(); assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_pipe_local_round_variable_radii_on_arc() {
  for global in [false,true] {
    let curve=om9_core::curve_build::arc_by_center([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],20.,0.,std::f64::consts::FRAC_PI_2).unwrap();
    let rail=Entity::new("arc",Geometry::Curve(curve),None); let id=rail.id; let mut doc=Document::new("viewport local round arc"); doc.entities.insert(id,rail);
    let mut app=MatrixApp::new(doc).unwrap(); let ctx=egui::Context::default(); frame(&mut app,&ctx,vec![]); frame(&mut app,&ctx,vec![]); let before=app.controller.document().clone();
    app.controller.submit("Pipe").unwrap(); click(&mut app,&ctx,[14.14,14.14,0.]); assert!(!app.controller.solid_pipe_needs_curve());
    app.controller.submit("Cap=Round").unwrap(); if global {app.controller.submit("ShapeBlending=Global").unwrap();} app.controller.submit("1").unwrap(); app.controller.submit("2").unwrap(); app.controller.submit("").unwrap();
    let entity=app.controller.document().entities.values().find(|e|e.id!=id).unwrap(); let Geometry::Brep(body)=&entity.geometry else {panic!()};
    let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap(); let expected=std::f64::consts::PI*(20.*std::f64::consts::FRAC_PI_2*if global {7./3.} else {83./35.}+6.);
    assert!(m.closed && (m.volume_mm3-expected).abs()<0.03*expected,"{m:?}"); app.controller.undo().unwrap(); assert_eq!(app.controller.document(),&before);
  }
}

#[test]
fn viewport_pipe_local_round_variable_radii_on_line() {
    let rail=Entity::new("rail",Geometry::Curve(Curve::Line {start:[0.,0.,0.],end:[10.,0.,0.],domain:[0.,1.]}),None); let id=rail.id;
    let mut doc=Document::new("viewport local round pipe"); doc.entities.insert(id,rail); let mut app=MatrixApp::new(doc).unwrap(); let ctx=egui::Context::default();
    frame(&mut app,&ctx,vec![]); frame(&mut app,&ctx,vec![]); let before=app.controller.document().clone();
    app.controller.submit("Pipe").unwrap(); click(&mut app,&ctx,[5.,0.,0.]); app.controller.submit("Cap=Round").unwrap(); app.controller.submit("1").unwrap(); assert_eq!(app.controller.solid_preview([10.,2.,0.]).len(),240); app.controller.submit("2").unwrap(); app.controller.submit("").unwrap();
    let entity=app.controller.document().entities.values().find(|e|e.id!=id).unwrap(); let Geometry::Brep(body)=&entity.geometry else {panic!()};
    let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap(); let expected=std::f64::consts::PI*(10.*83./35.+6.);
    assert!(m.closed && (m.volume_mm3-expected).abs()<0.02,"{m:?}"); app.controller.undo().unwrap(); assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_pipe_local_intermediate_station_on_line() {
    let rail=Entity::new("rail",Geometry::Curve(Curve::Line {start:[0.,0.,0.],end:[10.,0.,0.],domain:[0.,1.]}),None); let id=rail.id;
    let mut doc=Document::new("viewport local station"); doc.entities.insert(id,rail); let mut app=MatrixApp::new(doc).unwrap(); let ctx=egui::Context::default();
    frame(&mut app,&ctx,vec![]); frame(&mut app,&ctx,vec![]); let before=app.controller.document().clone();
    app.controller.submit("Pipe").unwrap(); click(&mut app,&ctx,[5.,0.,0.]); app.controller.submit("1").unwrap(); app.controller.submit("3").unwrap();
    click(&mut app,&ctx,[5.,0.,0.]); assert!(app.controller.prompt().contains("Intermediate radius")); app.controller.submit("4").unwrap(); app.controller.submit("").unwrap();
    let entity=app.controller.document().entities.values().find(|e|e.id!=id).unwrap(); let Geometry::Brep(body)=&entity.geometry else {panic!()};
    let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap(); let expected=std::f64::consts::PI*10.*69./7.;
    assert!(m.closed && (m.volume_mm3-expected).abs()<0.03*expected,"{m:?}"); app.controller.undo().unwrap(); assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_pipe_local_thick_variable_radii_on_line() {
    let rail=Entity::new("rail",Geometry::Curve(Curve::Line {start:[0.,0.,0.],end:[10.,0.,0.],domain:[0.,1.]}),None); let id=rail.id;
    let mut doc=Document::new("viewport local thick pipe"); doc.entities.insert(id,rail);
    let mut app=MatrixApp::new(doc).unwrap(); let ctx=egui::Context::default(); frame(&mut app,&ctx,vec![]); frame(&mut app,&ctx,vec![]);
    let before=app.controller.document().clone(); app.controller.submit("Pipe").unwrap(); click(&mut app,&ctx,[5.,0.,0.]); app.controller.submit("Thick=Yes").unwrap();
    for v in ["1","2","1.5"] {app.controller.submit(v).unwrap();} assert_eq!(app.controller.solid_preview([10.,3.,0.]).len(),352); app.controller.submit("3").unwrap(); app.controller.submit("").unwrap();
    let entity=app.controller.document().entities.values().find(|e|e.id!=id).unwrap(); let Geometry::Brep(body)=&entity.geometry else {panic!()};
    let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap(); let expected=std::f64::consts::PI*10.*669./140.;
    assert!(m.closed && (m.volume_mm3-expected).abs()<0.03*expected,"{m:?}"); app.controller.undo().unwrap(); assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_pipe_local_variable_radii_on_line() {
    let rail=Entity::new("rail",Geometry::Curve(Curve::Line {start:[0.,0.,0.],end:[10.,0.,0.],domain:[0.,1.]}),None); let id=rail.id;
    let mut doc=Document::new("viewport local pipe"); doc.entities.insert(id,rail);
    let mut app=MatrixApp::new(doc).unwrap(); let ctx=egui::Context::default(); frame(&mut app,&ctx,vec![]); frame(&mut app,&ctx,vec![]);
    let before=app.controller.document().clone(); app.controller.submit("Pipe").unwrap(); click(&mut app,&ctx,[5.,0.,0.]);
    click(&mut app,&ctx,[0.,1.,0.]); click(&mut app,&ctx,[10.,2.,0.]); app.controller.submit("").unwrap();
    let entity=app.controller.document().entities.values().find(|e|e.id!=id).unwrap(); let Geometry::Brep(body)=&entity.geometry else {panic!()};
    let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap(); let expected=10.*std::f64::consts::PI*83./35.;
    assert!(m.closed && (m.volume_mm3-expected).abs()<0.15,"{m:?}"); app.controller.undo().unwrap(); assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_pipe_multiple_global_station_on_mixed_rails() {
  for round in [false,true] {
    let arc=Entity::new("arc",Geometry::Curve(om9_core::curve_build::arc_by_center([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],5.,0.,std::f64::consts::FRAC_PI_2).unwrap()),None);
    let line=Entity::new("line",Geometry::Curve(Curve::Line {start:[0.,-15.,0.],end:[10.,-15.,0.],domain:[0.,1.]}),None);
    let mut doc=Document::new("mixed viewport stations"); doc.entities.insert(arc.id,arc); doc.entities.insert(line.id,line);
    let mut app=MatrixApp::new(doc).unwrap(); let ctx=egui::Context::default(); frame(&mut app,&ctx,vec![]); frame(&mut app,&ctx,vec![]);
    let before=app.controller.document().clone(); app.controller.submit("Pipe").unwrap(); app.controller.submit("Multiple").unwrap();
    click(&mut app,&ctx,[3.54,3.54,0.]); click(&mut app,&ctx,[5.,-15.,0.]); assert_eq!(app.controller.selected_ordered().len(),2); app.controller.submit("").unwrap();
    app.controller.submit("ShapeBlending=Global").unwrap(); if round {app.controller.submit("Cap=Round").unwrap();} app.controller.submit("1").unwrap(); app.controller.submit("3").unwrap();
    click(&mut app,&ctx,[3.54,3.54,0.]); assert!(app.controller.prompt().contains("Intermediate radius")); app.controller.submit("4").unwrap(); app.controller.submit("").unwrap();
    let mut volumes:Vec<_>=app.controller.document().entities.values().filter_map(|e|if let Geometry::Brep(b)=&e.geometry {Some(om9_kernel::creation::measure(b,app.controller.coordinator().limits()).unwrap().volume_mm3)} else {None}).collect();
    volumes.sort_by(f64::total_cmp); assert_eq!(volumes.len(),2); for (v,len) in volumes.iter().zip([5.*std::f64::consts::FRAC_PI_2,10.]) {let expected=std::f64::consts::PI*(len*29./3.+if round {56./3.} else {0.}); assert!((*v-expected).abs()<0.08*expected);}
    app.controller.undo().unwrap(); assert_eq!(app.controller.document(),&before);
  }
}

#[test]
fn viewport_pipe_multiple_global_station_on_straight_rails() {
    let mut doc=Document::new("multiple viewport stations");
    for y in [-5.,5.] {let rail=Entity::new("rail",Geometry::Curve(Curve::Line {start:[0.,y,0.],end:[10.,y,0.],domain:[0.,1.]}),None); doc.entities.insert(rail.id,rail);}
    let mut app=MatrixApp::new(doc).unwrap(); let ctx=egui::Context::default(); frame(&mut app,&ctx,vec![]); frame(&mut app,&ctx,vec![]);
    let before=app.controller.document().clone(); app.controller.submit("Pipe").unwrap(); app.controller.submit("Multiple").unwrap();
    click(&mut app,&ctx,[5.,-5.,0.]); click(&mut app,&ctx,[5.,5.,0.]); app.controller.submit("").unwrap();
    app.controller.submit("ShapeBlending=Global").unwrap(); app.controller.submit("1").unwrap(); app.controller.submit("3").unwrap();
    click(&mut app,&ctx,[5.,-5.,0.]); assert!(app.controller.prompt().contains("Intermediate radius")); app.controller.submit("4").unwrap(); app.controller.submit("").unwrap();
    let volumes:Vec<_>=app.controller.document().entities.values().filter_map(|e|if let Geometry::Brep(b)=&e.geometry {Some(om9_kernel::creation::measure(b,app.controller.coordinator().limits()).unwrap().volume_mm3)} else {None}).collect();
    assert_eq!(volumes.len(),2); for v in volumes {assert!((v-std::f64::consts::PI*10.*29./3.).abs()<0.2);}
    assert_eq!(app.controller.coordinator().undo_len(),1); app.controller.undo().unwrap(); assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_multi_pipe_menu_and_original_macro_share_geometry() {
  for menu in [true,false] {
    let mut doc=Document::new("Multi Pipe routes");
    for y in [-5.,5.] {let rail=Entity::new("rail",Geometry::Curve(Curve::Line{start:[0.,y,0.],end:[10.,y,0.],domain:[0.,1.]}),None);doc.entities.insert(rail.id,rail);}
    let mut app=MatrixApp::new(doc).unwrap();let ctx=egui::Context::default();frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![]);
    let before=app.controller.document().clone();
    if menu {app.controller.dispatch("SolidGVDMultiPipe",InvocationOrigin::Menu).unwrap();} else {app.controller.submit("_Pipe _Multiple").unwrap();}
    assert_eq!(app.controller.prompt(),"Select curves for Pipe (Enter when done)");
    click(&mut app,&ctx,[5.,-5.,0.]);click(&mut app,&ctx,[5.,5.,0.]);app.controller.submit("").unwrap();
    app.controller.submit("1").unwrap();app.controller.submit("1").unwrap();app.controller.submit("").unwrap();
    let volumes:Vec<_>=app.controller.document().entities.values().filter_map(|e|match &e.geometry {Geometry::Brep(b)=>Some(om9_kernel::creation::measure(b,app.controller.coordinator().limits()).unwrap().volume_mm3),_=>None}).collect();
    assert_eq!(volumes.len(),2);for v in volumes {assert!((v-10.*std::f64::consts::PI).abs()<1e-5);}
    assert_eq!(app.controller.coordinator().undo_len(),1);app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
  }
}

#[test]
fn viewport_pipe_multiple_selects_two_rails_and_commits_once() {
  for point_input in [false,true] {
    let mut doc = Document::new("Multiple Pipe");
    for y in [-5.0, 5.0] {
        let rail = Entity::new("rail", Geometry::Curve(Curve::Line {
            start: [0.0, y, 0.0], end: [10.0, y, 0.0], domain: [0.0, 1.0],
        }), None);
        doc.entities.insert(rail.id, rail);
    }
    let mut app = MatrixApp::new(doc).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
    let before = app.controller.document().clone();
    app.controller.dispatch("OM9-SOLID-011", InvocationOrigin::Menu).unwrap();
    app.controller.submit("Multiple").unwrap();
    click(&mut app, &ctx, [5.0, -5.0, 0.0]);
    click(&mut app, &ctx, [5.0, 5.0, 0.0]);
    assert_eq!(app.controller.selected().len(), 2);
    app.controller.submit("Undo").unwrap();
    assert_eq!(app.controller.selected().len(), 1);
    click(&mut app, &ctx, [5.0, 5.0, 0.0]);
    app.controller.submit("").unwrap();
    if point_input {assert_eq!(app.controller.solid_preview([0.,-4.,0.]).len(),64); click(&mut app,&ctx,[0.,-4.,0.]); click(&mut app,&ctx,[10.,-4.,0.]);}
    else {app.controller.submit("1").unwrap(); app.controller.submit("1").unwrap();}
    app.controller.submit("").unwrap();
    let volumes: Vec<_> = app.controller.document().entities.values().filter_map(|e| match &e.geometry {
        Geometry::Brep(b) => Some(om9_kernel::creation::measure(b, app.controller.coordinator().limits()).unwrap().volume_mm3), _ => None,
    }).collect();
    assert_eq!(volumes.len(), 2);
    for volume in volumes { assert!((volume - 10.0 * std::f64::consts::PI).abs() < if point_input {0.2} else {1e-5}); }
    assert_eq!(app.controller.coordinator().undo_len(), 1);
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &before);
  }
}

#[test]
fn viewport_pipe_intermediate_station_sets_radius_and_commits() {
    let mut doc = Document::new("Pipe middle station");
    let rail = Entity::new("rail", Geometry::Curve(Curve::Line {
        start: [0.0, 0.0, 0.0], end: [10.0, 0.0, 0.0], domain: [0.0, 1.0],
    }), None);
    let id = rail.id;
    doc.entities.insert(id, rail);
    let mut app = MatrixApp::new(doc).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![]);
    let before = app.controller.document().clone();
    app.controller.dispatch("OM9-SOLID-011", InvocationOrigin::Menu).unwrap();
    click(&mut app, &ctx, [5.0, 0.0, 0.0]);
    app.controller.submit("ShapeBlending=Global").unwrap();
    app.controller.submit("1").unwrap();
    app.controller.submit("3").unwrap();
    click(&mut app, &ctx, [5.0, 0.0, 0.0]);
    assert!(app.controller.prompt().contains("Intermediate radius"));
    click(&mut app, &ctx, [5.0, 2.0, 0.0]);
    app.controller.submit("").unwrap();
    let result = app.controller.document().entities.values().find(|e| e.id != id).unwrap();
    let Geometry::Brep(brep) = &result.geometry else { panic!() };
    let m = om9_kernel::creation::measure(brep, app.controller.coordinator().limits()).unwrap();
    assert!((m.volume_mm3 - 130.0 * std::f64::consts::PI / 3.0).abs() < 0.1, "{m:?}");
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &before);
}

#[test]
fn viewport_pipe_radius_points_on_both_sides_make_expected_geometry() {
  for diameter in [false, true] {
    for sign in [-1.0, 1.0] {
        let mut doc = Document::new("Pipe radius points");
        let rail = Entity::new("rail", Geometry::Curve(Curve::Line {
            start: [0.0, 0.0, 0.0], end: [10.0, 0.0, 0.0], domain: [0.0, 1.0],
        }), None);
        let id = rail.id;
        doc.entities.insert(id, rail);
        let mut app = MatrixApp::new(doc).unwrap();
        let ctx = egui::Context::default();
        frame(&mut app, &ctx, vec![]);
        frame(&mut app, &ctx, vec![]);
        let before = app.controller.document().clone();
        app.controller.dispatch("OM9-SOLID-011", InvocationOrigin::Menu).unwrap();
        click(&mut app, &ctx, [5.0, 0.0, 0.0]);
        if diameter {app.controller.submit("Diameter").unwrap();}
        assert_eq!(app.controller.solid_preview([0.0,sign*2.0,0.0]).len(),64);
        click(&mut app, &ctx, [0.0, sign * 2.0, 0.0]);
        assert!(app.controller.prompt().contains(if diameter {"End diameter"} else {"End radius"}));
        assert_eq!(app.controller.solid_preview([10.0,sign*2.0,0.0]).len(),176);
        click(&mut app, &ctx, [10.0, sign * 2.0, 0.0]);
        app.controller.submit("").unwrap();
        let result = app.controller.document().entities.values().find(|e| e.id != id).unwrap();
        let Geometry::Brep(brep) = &result.geometry else { panic!() };
        let m = om9_kernel::creation::measure(brep, app.controller.coordinator().limits()).unwrap();
        assert!((m.volume_mm3 - 40.0 * std::f64::consts::PI).abs() < 0.1, "{m:?}");
        app.controller.undo().unwrap();
        assert_eq!(app.controller.document(), &before);
    }
  }
}

#[test]
fn viewport_boss_smooth_nurbs_line_on_cylinder() {
    let cylinder=om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Cylinder {radius_mm:5.,height_mm:10.},0.01,&Limits::default()).unwrap();
    let count=om9_kernel::creation::measure(&cylinder,&Limits::default()).unwrap().faces;
    let face=(0..count).map(|i| om9_kernel::editing::extract_face(&cylinder,i,0.01,&Limits::default()).unwrap())
        .find(|b| (om9_kernel::creation::measure(b,&Limits::default()).unwrap().area_mm2-100.*std::f64::consts::PI).abs()<1e-6).unwrap();
    let target=Entity::new("cylinder",Geometry::Brep(face),None); let id=target.id;
    let arch=Curve::Nurbs(om9_core::geometry::NurbsCurve {degree:2,poles:vec![[-1.,0.,3.],[-1.,0.,4.],[1.,0.,4.],[1.,0.,3.]],weights:vec![1.,0.7,1.2,1.],knots:vec![0.,0.,0.,0.5,1.,1.,1.],domain:[0.,1.],periodic:false,reversed:false,seam:None});
    let line=Curve::Line {start:[1.,0.,3.],end:[-1.,0.,3.],domain:[0.,1.]};
    let rail=Entity::new("profile",Geometry::Curve(Curve::Polycurve {segments:vec![arch,line],reversed:false}),None); let rail_id=rail.id;
    let mut doc=Document::new("viewport polycurve cylinder Boss"); doc.entities.insert(id,target); doc.entities.insert(rail_id,rail);
    let mut app=MatrixApp::new(doc).unwrap(); let ctx=egui::Context::default(); frame(&mut app,&ctx,vec![]); frame(&mut app,&ctx,vec![]);
    let before=app.controller.document().clone(); app.controller.submit("Boss").unwrap();
    click(&mut app,&ctx,[-1.,0.,3.]); app.controller.submit("").unwrap(); click(&mut app,&ctx,[0.,5.,3.]);
    let Geometry::Brep(body)=&app.controller.document().entities[&id].geometry else {panic!()};
    let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap(); assert_eq!(m.faces,4); assert_eq!(m.shells,1);
    app.controller.undo().unwrap(); assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_boss_smooth_rational_nurbs_on_cylinder() {
    let cylinder=om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Cylinder {radius_mm:5.,height_mm:10.},0.01,&Limits::default()).unwrap();
    let count=om9_kernel::creation::measure(&cylinder,&Limits::default()).unwrap().faces;
    let face=(0..count).map(|i| om9_kernel::editing::extract_face(&cylinder,i,0.01,&Limits::default()).unwrap())
        .find(|b| (om9_kernel::creation::measure(b,&Limits::default()).unwrap().area_mm2-100.*std::f64::consts::PI).abs()<1e-6).unwrap();
    let target=Entity::new("cylinder",Geometry::Brep(face),None); let id=target.id;
    let n=om9_core::geometry::NurbsCurve {degree:2,poles:vec![[1.,0.,3.],[1.,0.,4.],[-1.,0.,4.],[-1.,0.,2.],[1.,0.,2.],[1.,0.,3.]],weights:vec![1.,0.7,1.2,0.8,1.3,1.],knots:vec![0.,0.,0.,0.25,0.5,0.75,1.,1.,1.],domain:[0.,1.],periodic:false,reversed:false,seam:None};
    let rail=Entity::new("profile",Geometry::Curve(Curve::Nurbs(n)),None); let rail_id=rail.id;
    let mut doc=Document::new("viewport smooth cylinder Boss"); doc.entities.insert(id,target); doc.entities.insert(rail_id,rail);
    let mut app=MatrixApp::new(doc).unwrap(); let ctx=egui::Context::default(); frame(&mut app,&ctx,vec![]); frame(&mut app,&ctx,vec![]);
    let before=app.controller.document().clone(); app.controller.submit("Boss").unwrap();
    click(&mut app,&ctx,[1.,0.,3.]); app.controller.submit("").unwrap(); click(&mut app,&ctx,[0.,5.,3.]);
    let Geometry::Brep(body)=&app.controller.document().entities[&id].geometry else {panic!()};
    let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap(); assert_eq!(m.faces,5); assert_eq!(m.shells,1);
    app.controller.undo().unwrap(); assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_boss_circle_and_ellipse_on_cylindrical_boundary() {
    for circle in [true,false] {
    let cylinder=om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Cylinder {radius_mm:5.,height_mm:10.},0.01,&Limits::default()).unwrap();
    let count=om9_kernel::creation::measure(&cylinder,&Limits::default()).unwrap().faces;
    let face=(0..count).map(|i| om9_kernel::editing::extract_face(&cylinder,i,0.01,&Limits::default()).unwrap())
        .find(|b| (om9_kernel::creation::measure(b,&Limits::default()).unwrap().area_mm2-100.*std::f64::consts::PI).abs()<1e-6).unwrap();
    let target=Entity::new("cylinder",Geometry::Brep(face),None); let id=target.id;
    let curve=if circle {om9_core::curve_build::circle_on_frame([0.,0.,3.],[1.,0.,0.],[0.,0.,1.],1.).unwrap()}
        else {Curve::Nurbs(om9_core::curve_build::ellipse([0.,0.,3.],[1.,0.,0.],[0.,0.,1.],1.,0.5,&Limits::default()).unwrap())};
    let rail=Entity::new("circle",Geometry::Curve(curve),None); let rail_id=rail.id;
    let mut doc=Document::new("viewport cylinder circle Boss"); doc.entities.insert(id,target); doc.entities.insert(rail_id,rail);
    let mut app=MatrixApp::new(doc).unwrap(); let ctx=egui::Context::default(); frame(&mut app,&ctx,vec![]); frame(&mut app,&ctx,vec![]);
    let before=app.controller.document().clone(); app.controller.submit("Boss").unwrap();
    click(&mut app,&ctx,[1.,0.,3.]); app.controller.submit("").unwrap(); click(&mut app,&ctx,[0.,5.,3.]);
    let Geometry::Brep(body)=&app.controller.document().entities[&id].geometry else {panic!()};
    let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap(); assert_eq!(m.faces,if circle {2} else {5}); assert_eq!(m.shells,1);
    app.controller.undo().unwrap(); assert_eq!(app.controller.document(),&before);
    }
}

#[test]
fn viewport_boss_multiple_polygons_on_cylinder() {
    let cylinder=om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Cylinder {radius_mm:5.,height_mm:10.},0.01,&Limits::default()).unwrap();
    let count=om9_kernel::creation::measure(&cylinder,&Limits::default()).unwrap().faces;
    let face=(0..count).map(|i| om9_kernel::editing::extract_face(&cylinder,i,0.01,&Limits::default()).unwrap())
        .find(|b| (om9_kernel::creation::measure(b,&Limits::default()).unwrap().area_mm2-100.*std::f64::consts::PI).abs()<1e-6).unwrap();
    let target=Entity::new("cylinder",Geometry::Brep(face),None); let id=target.id;
    let mut doc=Document::new("viewport multi cylinder Boss"); doc.entities.insert(id,target);
    for (x0,x1) in [(-3.,-2.),(2.,3.)] {
        let curve=om9_core::curve_build::polygon(&[[x0,0.,2.],[x1,0.,2.],[x1,0.,4.],[x0,0.,4.]]).unwrap();
        let entity=Entity::new("profile",Geometry::Curve(curve),None); doc.entities.insert(entity.id,entity);
    }
    let mut app=MatrixApp::new(doc).unwrap(); let ctx=egui::Context::default(); frame(&mut app,&ctx,vec![]); frame(&mut app,&ctx,vec![]);
    let before=app.controller.document().clone(); app.controller.submit("Boss").unwrap();
    click(&mut app,&ctx,[-2.5,0.,2.]); click(&mut app,&ctx,[2.5,0.,2.]); app.controller.submit("").unwrap(); click(&mut app,&ctx,[0.,5.,3.]);
    let Geometry::Brep(body)=&app.controller.document().entities[&id].geometry else {panic!()};
    let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap(); assert_eq!(m.faces,9); assert_eq!(m.shells,1);
    assert_eq!(app.controller.coordinator().undo_len(),1); app.controller.undo().unwrap(); assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_boss_oblique_circle_on_cylindrical_boundary() {
    let cylinder=om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Cylinder {radius_mm:5.,height_mm:10.},0.01,&Limits::default()).unwrap();
    let count=om9_kernel::creation::measure(&cylinder,&Limits::default()).unwrap().faces;
    let face=(0..count).map(|i| om9_kernel::editing::extract_face(&cylinder,i,0.01,&Limits::default()).unwrap())
        .find(|b| (om9_kernel::creation::measure(b,&Limits::default()).unwrap().area_mm2-100.*std::f64::consts::PI).abs()<1e-6).unwrap();
    let boundary=Entity::new("cylinder face",Geometry::Brep(face),None); let id=boundary.id;
    let h=3.0_f64.sqrt()/2.;
    let curve=om9_core::curve_build::circle_on_frame([0.,0.,5.],[1.,0.,0.],[0.,0.5,h],1.).unwrap();
    let rail=Entity::new("profile",Geometry::Curve(curve),None); let rail_id=rail.id;
    let mut doc=Document::new("viewport oblique circle Boss"); doc.entities.insert(rail_id,rail); doc.entities.insert(id,boundary);
    let mut app=MatrixApp::new(doc).unwrap(); let ctx=egui::Context::default(); frame(&mut app,&ctx,vec![]); frame(&mut app,&ctx,vec![]);
    let before=app.controller.document().clone(); app.controller.submit("Boss").unwrap();
    click(&mut app,&ctx,[1.,0.,5.]); app.controller.submit("").unwrap(); click(&mut app,&ctx,[0.,5.,5.]);
    let Geometry::Brep(body)=&app.controller.document().entities[&id].geometry else {panic!()};
    let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();
    assert_eq!((m.faces,m.shells),(2,1)); app.controller.undo().unwrap(); assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_boss_oblique_polygon_on_cylindrical_boundary() {
    let cylinder=om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Cylinder {radius_mm:5.,height_mm:10.},0.01,&Limits::default()).unwrap();
    let count=om9_kernel::creation::measure(&cylinder,&Limits::default()).unwrap().faces;
    let face=(0..count).map(|i| om9_kernel::editing::extract_face(&cylinder,i,0.01,&Limits::default()).unwrap())
        .find(|b| (om9_kernel::creation::measure(b,&Limits::default()).unwrap().area_mm2-100.*std::f64::consts::PI).abs()<1e-6).unwrap();
    let boundary=Entity::new("cylinder face",Geometry::Brep(face),None); let id=boundary.id;
    let h=3.0_f64.sqrt()/2.;
    let curve=om9_core::curve_build::polygon(&[[-1.,0.,4.],[1.,0.,4.],[1.,0.5,4.+h],[-1.,0.5,4.+h]]).unwrap();
    let rail=Entity::new("profile",Geometry::Curve(curve),None); let rail_id=rail.id;
    let mut doc=Document::new("viewport oblique cylinder Boss"); doc.entities.insert(rail_id,rail); doc.entities.insert(id,boundary);
    let mut app=MatrixApp::new(doc).unwrap(); let ctx=egui::Context::default(); frame(&mut app,&ctx,vec![]); frame(&mut app,&ctx,vec![]);
    let before=app.controller.document().clone(); app.controller.submit("Boss").unwrap();
    click(&mut app,&ctx,[0.,0.,4.]); app.controller.submit("").unwrap(); click(&mut app,&ctx,[0.,5.,4.]);
    let Geometry::Brep(body)=&app.controller.document().entities[&id].geometry else {panic!()};
    let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();
    assert_eq!((m.faces,m.shells),(5,1)); app.controller.undo().unwrap(); assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_boss_polygon_on_cylindrical_boundary() {
    let cylinder=om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Cylinder {radius_mm:5.,height_mm:10.},0.01,&Limits::default()).unwrap();
    let count=om9_kernel::creation::measure(&cylinder,&Limits::default()).unwrap().faces;
    let face=(0..count).map(|i| om9_kernel::editing::extract_face(&cylinder,i,0.01,&Limits::default()).unwrap())
        .find(|b| (om9_kernel::creation::measure(b,&Limits::default()).unwrap().area_mm2-100.*std::f64::consts::PI).abs()<1e-6).unwrap();
    let boundary=Entity::new("cylinder face",Geometry::Brep(face),None); let id=boundary.id;
    let curve=om9_core::curve_build::polygon(&[[-1.,0.,2.],[1.,0.,2.],[1.,0.,4.],[-1.,0.,4.]]).unwrap();
    let rail=Entity::new("profile",Geometry::Curve(curve),None); let rail_id=rail.id;
    let mut doc=Document::new("viewport cylinder Boss"); doc.entities.insert(rail_id,rail); doc.entities.insert(id,boundary);
    let mut app=MatrixApp::new(doc).unwrap(); let ctx=egui::Context::default(); frame(&mut app,&ctx,vec![]); frame(&mut app,&ctx,vec![]);
    let before=app.controller.document().clone(); app.controller.submit("Boss").unwrap();
    click(&mut app,&ctx,[0.,0.,2.]); app.controller.submit("").unwrap(); click(&mut app,&ctx,[0.,5.,3.]);
    let Geometry::Brep(body)=&app.controller.document().entities[&id].geometry else {panic!()};
    let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();
    assert_eq!(m.faces,5); assert_eq!(m.shells,1); app.controller.undo().unwrap(); assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_boss_piecewise_nurbs_near_inclined_boundary() {
    let h=std::f64::consts::FRAC_1_SQRT_2;
    let e=om9_core::curve_build::ellipse([5.,5.,0.],[h,h,0.],[-h,h,0.],2.,1.,&Limits::default()).unwrap();
    let segments=(0..4).map(|i| Curve::Nurbs(om9_core::geometry::NurbsCurve { degree:2,poles:e.poles[2*i..=2*i+2].to_vec(),weights:e.weights[2*i..=2*i+2].to_vec(),knots:vec![0.,0.,0.,1.,1.,1.],domain:[0.,1.],periodic:false,reversed:false,seam:None })).collect();
    let rail=Entity::new("ellipse",Geometry::Curve(Curve::Polycurve {segments,reversed:false}),None); let rail_id=rail.id;
    let p=om9_kernel::creation::Profile::new(vec![[0.,0.,-1.38],[10.,0.,0.62],[10.,10.,2.62],[0.,10.,0.62]]).unwrap();
    let boundary=Entity::new("boundary",Geometry::Brep(om9_kernel::creation::planar(&p,0.01,&Limits::default()).unwrap()),None); let id=boundary.id;
    let mut doc=Document::new("viewport piecewise Boss"); doc.entities.insert(rail_id,rail); doc.entities.insert(id,boundary);
    let mut app=MatrixApp::new(doc).unwrap(); let ctx=egui::Context::default(); frame(&mut app,&ctx,vec![]); frame(&mut app,&ctx,vec![]);
    let before=app.controller.document().clone(); app.controller.submit("Boss").unwrap();
    click(&mut app,&ctx,[5.+2.*h,5.+2.*h,0.]); app.controller.submit("").unwrap(); click(&mut app,&ctx,[9.,1.,0.62]);
    let Geometry::Brep(body)=&app.controller.document().entities[&id].geometry else {panic!()};
    let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap(); assert_eq!(m.faces,5); assert_eq!(m.shells,1);
    app.controller.undo().unwrap(); assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_boss_rotated_nurbs_near_inclined_boundary() {
    let h=std::f64::consts::FRAC_1_SQRT_2;
    let curve=Curve::Nurbs(om9_core::curve_build::ellipse([5.,5.,0.],[h,h,0.],[-h,h,0.],2.,1.,&Limits::default()).unwrap());
    let rail=Entity::new("ellipse",Geometry::Curve(curve),None); let rail_id=rail.id;
    let p=om9_kernel::creation::Profile::new(vec![[0.,0.,-1.38],[10.,0.,0.62],[10.,10.,2.62],[0.,10.,0.62]]).unwrap();
    let face=om9_kernel::creation::planar(&p,0.01,&Limits::default()).unwrap();
    let boundary=Entity::new("boundary",Geometry::Brep(face),None); let id=boundary.id;
    let mut doc=Document::new("viewport near plane Boss"); doc.entities.insert(rail_id,rail); doc.entities.insert(id,boundary);
    let mut app=MatrixApp::new(doc).unwrap(); let ctx=egui::Context::default();
    frame(&mut app,&ctx,vec![]); frame(&mut app,&ctx,vec![]);
    let before=app.controller.document().clone(); app.controller.submit("Boss").unwrap();
    click(&mut app,&ctx,[5.+2.*h,5.+2.*h,0.]); app.controller.submit("").unwrap(); click(&mut app,&ctx,[9.,1.,0.62]);
    let Geometry::Brep(body)=&app.controller.document().entities[&id].geometry else { panic!() };
    let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();
    assert_eq!(m.faces,2); assert_eq!(m.shells,1); app.controller.undo().unwrap(); assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_boss_selects_curved_profiles_with_inclined_boundary() {
    for circle in [false, true] {
        let p = om9_kernel::creation::Profile::new(vec![[0.,0.,5.],[10.,0.,7.],[10.,10.,7.],[0.,10.,5.]]).unwrap();
        let face = om9_kernel::creation::planar(&p, 0.01, &Limits::default()).unwrap();
        let boundary = Entity::new("inclined", Geometry::Brep(face), None); let id = boundary.id;
        let curve = if circle { om9_core::curve_build::circle_on_frame([5.,5.,0.],[1.,0.,0.],[0.,1.,0.],1.).unwrap() }
            else { Curve::Nurbs(om9_core::curve_build::ellipse([5.,5.,0.],[1.,0.,0.],[0.,1.,0.],2.,1.,&Limits::default()).unwrap()) };
        let rail = Entity::new("profile", Geometry::Curve(curve), None); let rail_id = rail.id;
        let mut doc = Document::new("viewport curved inclined Boss"); doc.entities.insert(rail_id, rail); doc.entities.insert(id, boundary);
        let mut app = MatrixApp::new(doc).unwrap(); let ctx = egui::Context::default();
        frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
        let before = app.controller.document().clone(); app.controller.submit("Boss").unwrap();
        click(&mut app, &ctx, if circle { [6.,5.,0.] } else { [7.,5.,0.] });
        app.controller.submit("").unwrap(); click(&mut app, &ctx, [9.,1.,6.8]);
        let Geometry::Brep(body) = &app.controller.document().entities[&id].geometry else { panic!() };
        let m = om9_kernel::creation::measure(body, app.controller.coordinator().limits()).unwrap();
        assert_eq!(m.faces, 2); assert_eq!(m.shells, 1);
        app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
    }
}

#[test]
fn viewport_boss_selects_inclined_planar_boundary() {
    let p = om9_kernel::creation::Profile::new(vec![[0.,0.,5.],[10.,0.,7.],[10.,10.,7.],[0.,10.,5.]]).unwrap();
    let face = om9_kernel::creation::planar(&p, 0.01, &Limits::default()).unwrap();
    let boundary = Entity::new("inclined", Geometry::Brep(face), None); let id = boundary.id;
    let curve = om9_core::curve_build::polygon(&[[1.,2.,0.],[3.,2.,0.],[3.,4.,0.],[1.,4.,0.]]).unwrap();
    let rail = Entity::new("profile", Geometry::Curve(curve), None); let rail_id = rail.id;
    let mut doc = Document::new("viewport inclined Boss"); doc.entities.insert(rail_id, rail); doc.entities.insert(id, boundary);
    let mut app = MatrixApp::new(doc).unwrap(); let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
    let before = app.controller.document().clone(); app.controller.submit("Boss").unwrap();
    click(&mut app, &ctx, [2.,2.,0.]); app.controller.submit("").unwrap(); click(&mut app, &ctx, [9.,1.,6.8]);
    let Geometry::Brep(body) = &app.controller.document().entities[&id].geometry else { panic!() };
    let m = om9_kernel::creation::measure(body, app.controller.coordinator().limits()).unwrap();
    assert_eq!(m.faces, 5); assert!((m.area_mm2-(96.*1.04_f64.sqrt()+43.2)).abs() < 1e-5, "{m:?}");
    app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
}

#[test]
fn viewport_boss_selects_coplanar_two_face_boundary() {
    let a = om9_kernel::creation::Profile::new(vec![[0.,0.,5.],[5.,0.,5.],[5.,10.,5.],[0.,10.,5.]]).unwrap();
    let b = om9_kernel::creation::Profile::new(vec![[5.,0.,5.],[10.,0.,5.],[10.,10.,5.],[5.,10.,5.]]).unwrap();
    let face = |p| om9_kernel::creation::planar(p, 0.01, &Limits::default()).unwrap();
    let shell = om9_kernel::editing::sew(&face(&a), &face(&b), 1e-7, 0.01, &Limits::default()).unwrap();
    let boundary = Entity::new("boundary", Geometry::Brep(shell), None); let id = boundary.id;
    let curve = om9_core::curve_build::polygon(&[[1.,2.,0.],[3.,2.,0.],[3.,4.,0.],[1.,4.,0.]]).unwrap();
    let rail = Entity::new("profile", Geometry::Curve(curve), None); let rail_id = rail.id;
    let mut doc = Document::new("viewport Boss two faces"); doc.entities.insert(rail_id, rail); doc.entities.insert(id, boundary);
    let mut app = MatrixApp::new(doc).unwrap(); let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
    let before = app.controller.document().clone(); app.controller.submit("Boss").unwrap();
    click(&mut app, &ctx, [2.,2.,0.]); app.controller.submit("").unwrap(); click(&mut app, &ctx, [9.,1.,5.]);
    let Geometry::Brep(body) = &app.controller.document().entities[&id].geometry else { panic!() };
    let m = om9_kernel::creation::measure(body, app.controller.coordinator().limits()).unwrap();
    assert_eq!(m.faces, 6); assert!((m.area_mm2-136.).abs() < 1e-6, "{m:?}");
    app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
}

#[test]
fn viewport_boss_selects_mixed_nurbs_line_in_both_directions() {
    for (reversed, reversed_segment) in [(false,false), (false,true), (true,false), (true,true)] {
        let arch = Curve::Nurbs(om9_core::geometry::NurbsCurve {
            degree: 2, poles: vec![[3.,5.,0.],[5.,9.,0.],[7.,5.,0.]], weights: vec![1.;3],
            knots: vec![0.,0.,0.,1.,1.,1.], domain: [0.,1.], periodic: false, reversed: reversed_segment, seam: None,
        });
        let line = Curve::Line { start: if reversed_segment { [3.,5.,0.] } else { [7.,5.,0.] }, end: if reversed_segment { [7.,5.,0.] } else { [3.,5.,0.] }, domain: [0.,1.] };
        let rail = Entity::new("profile", Geometry::Curve(Curve::Polycurve { segments: vec![arch,line], reversed }), None); let rail_id = rail.id;
        let profile = om9_kernel::creation::Profile::new(vec![[0.,0.,5.],[10.,0.,5.],[10.,10.,5.],[0.,10.,5.]]).unwrap();
        let face = om9_kernel::creation::planar(&profile, 0.01, &Limits::default()).unwrap();
        let boundary = Entity::new("boundary", Geometry::Brep(face), None); let id = boundary.id;
        let mut doc = Document::new("Viewport NURBS Boss"); doc.entities.insert(rail_id, rail); doc.entities.insert(id, boundary);
        let mut app = MatrixApp::new(doc).unwrap(); let ctx = egui::Context::default();
        frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
        let before = app.controller.document().clone(); app.controller.submit("Boss").unwrap();
        click(&mut app, &ctx, [5.,7.,0.]); app.controller.submit("").unwrap(); click(&mut app, &ctx, [9.,1.,5.]);
        let Geometry::Brep(body) = &app.controller.document().entities[&id].geometry else { panic!() };
        let m = om9_kernel::creation::measure(body, app.controller.coordinator().limits()).unwrap();
        let expected = 100. - 16./3. + 5.*(4. + 2.*5.0_f64.sqrt() + 2.0_f64.asinh());
        assert_eq!(m.faces, 3); assert!((m.area_mm2 - expected).abs() < 1e-7, "{m:?}");
        app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
    }
}

#[test]
fn viewport_boss_selects_mixed_arc_line_in_both_directions() {
    for reversed in [false, true] {
        let arc = om9_core::curve_build::arc_by_center([5.0,5.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0],2.0,0.0,std::f64::consts::PI).unwrap();
        let line = Curve::Line { start: [3.0,5.0,0.0], end: [7.0,5.0,0.0], domain: [0.0,1.0] };
        let rail = Entity::new("profile", Geometry::Curve(Curve::Polycurve { segments: vec![arc,line], reversed }), None); let rail_id = rail.id;
        let profile = om9_kernel::creation::Profile::new(vec![[0.0,0.0,5.0],[10.0,0.0,5.0],[10.0,10.0,5.0],[0.0,10.0,5.0]]).unwrap();
        let face = om9_kernel::creation::planar(&profile, 0.01, &Limits::default()).unwrap();
        let boundary = Entity::new("boundary", Geometry::Brep(face), None); let id = boundary.id;
        let mut doc = Document::new("Viewport arc Boss"); doc.entities.insert(rail_id, rail); doc.entities.insert(id, boundary);
        let mut app = MatrixApp::new(doc).unwrap(); let ctx = egui::Context::default();
        frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
        let before = app.controller.document().clone(); app.controller.submit("Boss").unwrap();
        click(&mut app, &ctx, [5.0,7.0,0.0]); app.controller.submit("").unwrap(); click(&mut app, &ctx, [9.0,1.0,5.0]);
        let Geometry::Brep(body) = &app.controller.document().entities[&id].geometry else { panic!() };
        let m = om9_kernel::creation::measure(body, app.controller.coordinator().limits()).unwrap();
        assert_eq!(m.faces, 3); assert!((m.area_mm2-(120.0+8.0*std::f64::consts::PI)).abs() < 1e-5, "{m:?}");
        app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
    }
}

#[test]
fn viewport_boss_selects_rational_ellipse() {
    let ellipse = om9_core::curve_build::ellipse([6.0,6.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0],2.0,3.0,&Limits::default()).unwrap();
    let rail = Entity::new("ellipse", Geometry::Curve(Curve::Nurbs(ellipse)), None); let rail_id = rail.id;
    let profile = om9_kernel::creation::Profile::new(vec![[0.0,0.0,5.0],[12.0,0.0,5.0],[12.0,12.0,5.0],[0.0,12.0,5.0]]).unwrap();
    let face = om9_kernel::creation::planar(&profile, 0.01, &Limits::default()).unwrap();
    let boundary = Entity::new("boundary", Geometry::Brep(face), None); let id = boundary.id;
    let mut doc = Document::new("Viewport ellipse Boss"); doc.entities.insert(rail_id, rail); doc.entities.insert(id, boundary);
    let mut app = MatrixApp::new(doc).unwrap(); let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
    let before = app.controller.document().clone(); app.controller.submit("Boss").unwrap();
    click(&mut app, &ctx, [8.0,6.0,0.0]); app.controller.submit("").unwrap(); click(&mut app, &ctx, [1.0,1.0,5.0]);
    let Geometry::Brep(body) = &app.controller.document().entities[&id].geometry else { panic!() };
    let m = om9_kernel::creation::measure(body, app.controller.coordinator().limits()).unwrap();
    assert_eq!(m.faces, 2); assert_eq!(m.shells, 1); assert_eq!(m.naked_edges, 5);
    assert_eq!(app.controller.coordinator().undo_len(), 1); app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
}

#[test]
fn viewport_boss_selects_mixed_circle_and_polygon() {
    let mut doc = Document::new("Viewport mixed Boss");
    let circle = om9_core::curve_build::circle_on_frame([2.0,2.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0],1.0).unwrap();
    let a = Entity::new("circle", Geometry::Curve(circle), None); doc.entities.insert(a.id, a);
    let polygon = om9_core::curve_build::polygon(&[[6.0,6.0,0.0],[8.0,6.0,0.0],[8.0,8.0,0.0],[6.0,8.0,0.0]]).unwrap();
    let b = Entity::new("polygon", Geometry::Curve(polygon), None); doc.entities.insert(b.id, b);
    let profile = om9_kernel::creation::Profile::new(vec![[0.0,0.0,5.0],[10.0,0.0,5.0],[10.0,10.0,5.0],[0.0,10.0,5.0]]).unwrap();
    let face = om9_kernel::creation::planar(&profile, 0.01, &Limits::default()).unwrap();
    let boundary = Entity::new("boundary", Geometry::Brep(face), None); let id = boundary.id; doc.entities.insert(id, boundary);
    let mut app = MatrixApp::new(doc).unwrap(); let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
    let before = app.controller.document().clone(); app.controller.submit("Boss").unwrap();
    click(&mut app, &ctx, [3.0, 2.0, 0.0]); click(&mut app, &ctx, [7.0, 6.0, 0.0]);
    app.controller.submit("").unwrap(); click(&mut app, &ctx, [9.0, 1.0, 5.0]);
    let Geometry::Brep(body) = &app.controller.document().entities[&id].geometry else { panic!() };
    let m = om9_kernel::creation::measure(body, app.controller.coordinator().limits()).unwrap();
    assert_eq!(m.faces, 6); assert!((m.area_mm2-(136.0+9.0*std::f64::consts::PI)).abs() < 1e-6, "{m:?}");
    app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
}

#[test]
fn viewport_boss_selects_multiple_profiles_and_undoes_last_pick() {
    let mut doc = Document::new("Viewport multi Boss");
    for x in [1.0, 6.0] {
        let curve = om9_core::curve_build::polygon(&[[x,1.0,0.0],[x+2.0,1.0,0.0],[x+2.0,3.0,0.0],[x,3.0,0.0]]).unwrap();
        let rail = Entity::new("profile", Geometry::Curve(curve), None); doc.entities.insert(rail.id, rail);
    }
    let profile = om9_kernel::creation::Profile::new(vec![[0.0,0.0,5.0],[10.0,0.0,5.0],[10.0,10.0,5.0],[0.0,10.0,5.0]]).unwrap();
    let face = om9_kernel::creation::planar(&profile, 0.01, &Limits::default()).unwrap();
    let boundary = Entity::new("boundary", Geometry::Brep(face), None); let id = boundary.id; doc.entities.insert(id, boundary);
    let mut app = MatrixApp::new(doc).unwrap(); let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
    let before = app.controller.document().clone(); app.controller.submit("Boss").unwrap();
    click(&mut app, &ctx, [2.0, 1.0, 0.0]); click(&mut app, &ctx, [7.0, 1.0, 0.0]);
    app.controller.submit("Undo").unwrap(); click(&mut app, &ctx, [7.0, 1.0, 0.0]);
    app.controller.submit("").unwrap(); click(&mut app, &ctx, [8.0, 8.0, 5.0]);
    let Geometry::Brep(body) = &app.controller.document().entities[&id].geometry else { panic!() };
    let m = om9_kernel::creation::measure(body, app.controller.coordinator().limits()).unwrap();
    assert_eq!(m.faces, 9); assert!((m.area_mm2-172.0).abs() < 1e-6, "{m:?}");
    assert_eq!(app.controller.coordinator().undo_len(), 1);
    app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
}

#[test]
fn viewport_boss_selects_polygon_and_planar_boundary() {
    let curve = om9_core::curve_build::polygon(&[[2.0,2.0,0.0], [4.0,2.0,0.0], [4.0,4.0,0.0], [2.0,4.0,0.0]]).unwrap();
    let rail = Entity::new("profile", Geometry::Curve(curve), None); let profile_id = rail.id;
    let profile = om9_kernel::creation::Profile::new(vec![[0.0,0.0,5.0],[10.0,0.0,5.0],[10.0,10.0,5.0],[0.0,10.0,5.0]]).unwrap();
    let face = om9_kernel::creation::planar(&profile, 0.01, &Limits::default()).unwrap();
    let boundary = Entity::new("boundary", Geometry::Brep(face), None); let boundary_id = boundary.id;
    let mut doc = Document::new("Viewport Boss"); doc.entities.insert(profile_id, rail); doc.entities.insert(boundary_id, boundary);
    let mut app = MatrixApp::new(doc).unwrap(); let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
    let before = app.controller.document().clone();
    app.controller.dispatch("OM9-SOLID-021", InvocationOrigin::Menu).unwrap();
    click(&mut app, &ctx, [3.0, 2.0, 0.0]);
    app.controller.submit("").unwrap();
    click(&mut app, &ctx, [8.0, 8.0, 5.0]);
    let Geometry::Brep(body) = &app.controller.document().entities[&boundary_id].geometry else { panic!() };
    let m = om9_kernel::creation::measure(body, app.controller.coordinator().limits()).unwrap();
    assert_eq!(m.faces, 5); assert!((m.area_mm2-136.0).abs() < 1e-6, "{m:?}");
    assert_eq!(app.controller.coordinator().undo_len(), 1);
    app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
}

#[test]
fn viewport_pipe_global_station_on_arc() {
    let arc=om9_core::curve_build::arc_by_center([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],5.,0.,std::f64::consts::FRAC_PI_2).unwrap();
    let rail=Entity::new("arc",Geometry::Curve(arc),None); let id=rail.id; let mut doc=Document::new("viewport arc station"); doc.entities.insert(id,rail);
    let mut app=MatrixApp::new(doc).unwrap(); let ctx=egui::Context::default(); frame(&mut app,&ctx,vec![]); frame(&mut app,&ctx,vec![]);
    let before=app.controller.document().clone(); app.controller.submit("Pipe").unwrap(); click(&mut app,&ctx,[3.54,3.54,0.]);
    assert!(!app.controller.solid_pipe_needs_curve()); app.controller.submit("ShapeBlending=Global").unwrap();
    app.controller.submit("1").unwrap(); app.controller.submit("3").unwrap(); click(&mut app,&ctx,[3.54,3.54,0.]);
    assert!(app.controller.prompt().contains("Intermediate radius"),"{}",app.controller.prompt()); app.controller.submit("4").unwrap(); app.controller.submit("").unwrap();
    let entity=app.controller.document().entities.values().find(|e|e.id!=id).unwrap(); let Geometry::Brep(body)=&entity.geometry else {panic!()};
    let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap(); let expected=std::f64::consts::PI*5.*std::f64::consts::FRAC_PI_2*29./3.;
    assert!(m.closed && (m.volume_mm3-expected).abs()<0.08*expected,"{m:?}"); app.controller.undo().unwrap(); assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_pipe_thick_global_on_arc() {
    let arc=om9_core::curve_build::arc_by_center([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],5.,0.,std::f64::consts::FRAC_PI_2).unwrap();
    let rail=Entity::new("arc",Geometry::Curve(arc),None); let id=rail.id; let mut doc=Document::new("viewport thick arc"); doc.entities.insert(id,rail);
    let mut app=MatrixApp::new(doc).unwrap(); let ctx=egui::Context::default(); frame(&mut app,&ctx,vec![]); frame(&mut app,&ctx,vec![]);
    let before=app.controller.document().clone(); app.controller.submit("Pipe").unwrap(); click(&mut app,&ctx,[3.54,3.54,0.]);
    assert!(!app.controller.solid_pipe_needs_curve()); app.controller.submit("Thick=Yes").unwrap(); app.controller.submit("ShapeBlending=Global").unwrap();
    for value in ["0.5","1","0.75","1.5"] {app.controller.submit(value).unwrap();} app.controller.submit("").unwrap();
    let entity=app.controller.document().entities.values().find(|e|e.id!=id).unwrap(); let Geometry::Brep(body)=&entity.geometry else {panic!()};
    let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();
    let expected=std::f64::consts::PI*5.*std::f64::consts::FRAC_PI_2*((1.+1.5+2.25)-(0.25+0.375+0.5625))/3.;
    assert!(m.closed && (m.volume_mm3-expected).abs()<0.03*expected,"{m:?}");
    app.controller.undo().unwrap(); assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_pipe_global_varying_radius_on_arc() {
    let arc=om9_core::curve_build::arc_by_center([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],5.,0.,std::f64::consts::FRAC_PI_2).unwrap();
    let rail=Entity::new("arc",Geometry::Curve(arc),None); let id=rail.id; let mut doc=Document::new("viewport varying arc"); doc.entities.insert(id,rail);
    let mut app=MatrixApp::new(doc).unwrap(); let ctx=egui::Context::default(); frame(&mut app,&ctx,vec![]); frame(&mut app,&ctx,vec![]);
    let before=app.controller.document().clone(); app.controller.submit("Pipe").unwrap(); click(&mut app,&ctx,[3.54,3.54,0.]);
    assert!(!app.controller.solid_pipe_needs_curve(), "{}",app.controller.prompt());
    app.controller.submit("ShapeBlending=Global").unwrap(); click(&mut app,&ctx,[7.,0.,0.]);
    assert!(app.controller.prompt().contains("End radius"), "{}",app.controller.prompt());
    click(&mut app,&ctx,[0.,8.,0.]); assert!(app.controller.prompt().contains("Enter"), "{}",app.controller.prompt());
    app.controller.submit("").unwrap(); let entity=app.controller.document().entities.values().find(|e|e.id!=id).unwrap();
    let Geometry::Brep(body)=&entity.geometry else {panic!()}; let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();
    let expected=std::f64::consts::PI*5.*std::f64::consts::FRAC_PI_2*(4.+6.+9.)/3.;
    assert!(m.closed && (m.volume_mm3-expected).abs()<0.08*expected,"{m:?}");
    app.controller.undo().unwrap(); assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_pipe_circle_section_points_use_endpoint_tangent() {
    let curve=om9_core::curve_build::circle_on_frame([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],5.).unwrap();
    let rail=Entity::new("circle",Geometry::Curve(curve),None); let id=rail.id;
    let mut doc=Document::new("Viewport circular Pipe points"); doc.entities.insert(id,rail);
    let mut app=MatrixApp::new(doc).unwrap(); let ctx=egui::Context::default();
    frame(&mut app,&ctx,vec![]); frame(&mut app,&ctx,vec![]);
    let before=app.controller.document().clone(); app.controller.submit("Pipe").unwrap();
    click(&mut app,&ctx,[5.,0.,0.]); app.controller.submit("Diameter").unwrap();
    click(&mut app,&ctx,[6.,0.,0.]); click(&mut app,&ctx,[6.,0.,0.]);
    app.controller.submit("").unwrap();
    let entity=app.controller.document().entities.values().find(|e|e.id!=id).unwrap();
    let Geometry::Brep(body)=&entity.geometry else {panic!()};
    let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();
    assert!((m.volume_mm3-10.*std::f64::consts::PI.powi(2)).abs()<0.1,"{m:?}");
    app.controller.undo().unwrap(); assert_eq!(app.controller.document(),&before);
}

#[test]
fn viewport_pipe_selects_circle_for_round_closed_rail() {
    for thick in [false, true] {
        let curve = om9_core::curve_build::circle_on_frame([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 5.0).unwrap();
        let rail = Entity::new("circle", Geometry::Curve(curve), None);
        let id = rail.id; let mut doc = Document::new("Viewport circular Pipe"); doc.entities.insert(id, rail);
        let mut app = MatrixApp::new(doc).unwrap(); let ctx = egui::Context::default();
        frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
        let before = app.controller.document().clone(); app.controller.submit("Pipe").unwrap();
        click(&mut app, &ctx, [5.0, 0.0, 0.0]);
        assert!(!app.controller.solid_pipe_needs_curve());
        app.controller.submit("Cap=Round").unwrap();
        if thick { app.controller.submit("Thick=Yes").unwrap(); for r in [0.5, 1.0, 0.5, 1.0] { app.controller.submit(&r.to_string()).unwrap(); } }
        else { app.controller.submit("1").unwrap(); app.controller.submit("1").unwrap(); }
        app.controller.submit("").unwrap();
        let body = app.controller.document().entities.values().find_map(|e| match &e.geometry { Geometry::Brep(b) => Some(b), _ => None }).unwrap();
        let m = om9_kernel::creation::measure(body, app.controller.coordinator().limits()).unwrap();
        let expected = (if thick { 7.5 } else { 10.0 }) * std::f64::consts::PI.powi(2);
        assert!(m.closed && m.solids == 1 && (m.volume_mm3-expected).abs() < 1e-6, "{m:?}");
        app.controller.undo().unwrap(); assert_eq!(app.controller.document(), &before);
    }
}

#[test]
fn viewport_pipe_selects_placed_curve_after_starting_command() {
    for via_menu in [true, false] {
        let mut doc = Document::new("Pipe viewport");
        let mut curve = Entity::new("rail", Geometry::Curve(Curve::Line {
            start: [0.0, 0.0, 0.0], end: [10.0, 0.0, 0.0], domain: [0.0, 1.0],
        }), None);
        curve.placement = Transform::translation([5.0, 0.0, 0.0]);
        let id = curve.id;
        doc.entities.insert(id, curve);
        let mut app = MatrixApp::new(doc).unwrap();
        let ctx = egui::Context::default();
        frame(&mut app, &ctx, vec![]);
        frame(&mut app, &ctx, vec![]);
        let before = app.controller.document().clone();
        if via_menu { app.controller.dispatch("OM9-SOLID-011", InvocationOrigin::Menu).unwrap(); }
        else { app.controller.submit("Pipe").unwrap(); }
        assert!(app.controller.solid_pipe_needs_curve());
        assert!(app.controller.submit("1").is_err());
        click(&mut app, &ctx, [10.0, 0.0, 0.0]);
        assert!(!app.controller.solid_pipe_needs_curve());
        app.controller.submit("1").unwrap();
        assert_eq!(app.controller.coordinator().undo_len(), 0);
        app.controller.submit("1").unwrap();
        app.controller.submit("").unwrap();
        let result = app.controller.document().entities.values().find(|e| e.id != id).unwrap();
        let Geometry::Brep(brep) = &result.geometry else { panic!() };
        let m = om9_kernel::creation::measure(brep, app.controller.coordinator().limits()).unwrap();
        assert!((m.volume_mm3 - 10.0 * std::f64::consts::PI).abs() < 1e-6);
        for (actual, expected) in m.bounds_mm.iter().zip([5.0, -1.0, -1.0, 15.0, 1.0, 1.0]) {
            assert!((actual - expected).abs() < 1e-5, "{:?}", m.bounds_mm);
        }
        assert_eq!(app.controller.coordinator().undo_len(), 1);
        app.controller.undo().unwrap();
        assert_eq!(app.controller.document(), &before);
    }
}

#[test]
fn viewport_pipe_selection_undo_and_cancel_do_not_commit() {
    let mut doc = Document::new("Pipe cancel");
    let curve = Entity::new("rail", Geometry::Curve(Curve::Line {
        start: [0.0, 0.0, 0.0], end: [10.0, 0.0, 0.0], domain: [0.0, 1.0],
    }), None);
    doc.entities.insert(curve.id, curve);
    let body = om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Box { x_mm: 2.0, y_mm: 2.0, z_mm: 2.0 },
        0.01, &Limits::default(),
    ).unwrap();
    let body = Entity::new("box", Geometry::Brep(body), None);
    let body_id = body.id;
    doc.entities.insert(body_id, body);
    let mut app = MatrixApp::new(doc).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![]);
    let before = app.controller.document().clone();
    app.controller.submit("Pipe").unwrap();
    assert!(app.controller.solid_pipe_pick_curve(body_id).is_err());
    click(&mut app, &ctx, [8.0, 0.0, 0.0]);
    assert!(!app.controller.solid_pipe_needs_curve());
    app.controller.submit("Undo").unwrap();
    assert!(app.controller.solid_pipe_needs_curve());
    click(&mut app, &ctx, [8.0, 0.0, 0.0]);
    app.controller.submit("Cancel").unwrap();
    assert_eq!(app.controller.coordinator().undo_len(), 0);
    assert_eq!(app.controller.document(), &before);
}

#[test]
fn viewport_pipe_disambiguates_overlapping_curves_and_rejects_stale_popup() {
    let mut doc = Document::new("overlapping Pipe rails");
    let mut ids = Vec::new();
    for length in [10.0, 20.0] {
        let curve = Entity::new("rail", Geometry::Curve(Curve::Line {
            start: [0.0, 0.0, 0.0], end: [length, 0.0, 0.0], domain: [0.0, 1.0],
        }), None);
        ids.push(curve.id);
        doc.entities.insert(curve.id, curve);
    }
    let mut app = MatrixApp::new(doc).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![]);
    app.controller.submit("Pipe").unwrap();
    click(&mut app, &ctx, [5.0, 0.0, 0.0]);
    assert_eq!(app.ambiguous_candidates().len(), 2);
    let index = app.ambiguous_candidates().iter().position(|id| *id == ids[1]).unwrap();
    app.controller.cancel();
    app.controller.submit("Pipe").unwrap();
    assert!(app.choose_ambiguous(index).is_err());
    assert!(app.controller.solid_pipe_needs_curve());
    click(&mut app, &ctx, [5.0, 0.0, 0.0]);
    let index = app.ambiguous_candidates().iter().position(|id| *id == ids[1]).unwrap();
    app.choose_ambiguous(index).unwrap();
    assert!(app.controller.selected().contains(&ids[1]));
    app.controller.submit("1").unwrap();
    app.controller.submit("1").unwrap();
    app.controller.submit("").unwrap();
    let brep = app.controller.document().entities.values().find_map(|e| match &e.geometry {
        Geometry::Brep(brep) => Some(brep), _ => None,
    }).unwrap();
    let measure = om9_kernel::creation::measure(brep, app.controller.coordinator().limits()).unwrap();
    assert!((measure.volume_mm3 - 20.0 * std::f64::consts::PI).abs() < 1e-6);
}

#[test]
fn viewport_shell_selects_two_faces_and_undoes() {
    let limits = Limits::default();
    let shape = om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Box { x_mm: 10.0, y_mm: 10.0, z_mm: 10.0 }, 0.01, &limits,
    ).unwrap();
    let mut doc = Document::new("Shell viewport");
    let entity = Entity::new("box", Geometry::Brep(shape), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut app = MatrixApp::new(doc).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![]);
    let before = app.controller.document().clone();
    app.controller.dispatch("OM9-SOLID-029", InvocationOrigin::Menu).unwrap();
    click_in_view(&mut app, &ctx, ViewId::LookingDown, [5.0, 5.0, 10.0], Modifiers::NONE);
    click_in_view(&mut app, &ctx, ViewId::SideView, [10.0, 5.0, 5.0], Modifiers::NONE);
    assert_eq!(app.controller.subselection().len(), 2);
    let Geometry::Brep(original) = &before.entities[&id].geometry else { panic!() };
    let indices: Vec<_> = app.controller.subselection().iter().map(|item| {
        let om9_scene::scene::Subelement::BrepFace(key) = item.subset else { panic!() };
        original.display().faces.iter().position(|face| *face == key).unwrap()
    }).collect();
    let expected = om9_kernel::editing::shell_faces(original, &indices, 1.0, 0.01, &limits).unwrap();
    let expected_volume = om9_kernel::creation::measure(&expected, &limits).unwrap().volume_mm3;
    app.controller.submit("Thickness=1").unwrap();
    app.controller.submit("").unwrap();
    let Geometry::Brep(result) = &app.controller.document().entities[&id].geometry else { panic!() };
    let measure = om9_kernel::creation::measure(result, &limits).unwrap();
    assert!(measure.closed && (measure.volume_mm3 - expected_volume).abs() < 1e-5);
    assert_eq!(app.controller.coordinator().undo_len(), 1);
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &before);
}

#[test]
fn viewport_selects_extract_surface_face_then_enter_separates_it() {
    let limits = Limits::default();
    let mut doc = Document::new("Extract viewport face");
    let body = om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Box {
            x_mm: 10.0,
            y_mm: 10.0,
            z_mm: 10.0,
        },
        0.01,
        &limits,
    )
    .unwrap();
    let entity = Entity::new("box", Geometry::Brep(body), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut app = MatrixApp::new(doc).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![]);
    app.controller.workspace.active = ViewId::LookingDown;
    let before = app.controller.document().clone();
    app.controller
        .dispatch("OM9-SOLID-006", InvocationOrigin::Menu)
        .unwrap();
    click(&mut app, &ctx, [5.0, 5.0, 10.0]);
    assert_eq!(app.controller.document(), &before);
    assert_eq!(app.controller.subselection().len(), 1);
    assert_eq!(app.controller.coordinator().undo_len(), 0);
    app.controller.submit("").unwrap();
    assert_eq!(app.controller.document().entities.len(), 2);
    let Geometry::Brep(remaining) = &app.controller.document().entities[&id].geometry else {
        panic!("shell")
    };
    let m =
        om9_kernel::creation::measure(remaining, app.controller.coordinator().limits()).unwrap();
    assert!(!m.closed);
    assert_eq!(m.faces, 5);
    let extracted = app
        .controller
        .document()
        .entities
        .values()
        .find(|e| e.id != id)
        .unwrap();
    let Geometry::Brep(face) = &extracted.geometry else {
        panic!("face")
    };
    assert!(
        face.display()
            .points
            .iter()
            .all(|point| (point[2] - 10.0).abs() < 1e-8)
    );
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &before);
}

#[test]
fn viewport_extract_collects_faces_from_multiple_objects() {
    let limits = Limits::default();
    let body = om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Box {
            x_mm: 10.0,
            y_mm: 10.0,
            z_mm: 10.0,
        },
        0.01,
        &limits,
    )
    .unwrap();
    let mut doc = Document::new("Extract two boxes");
    let mut ids = Vec::new();
    for x in [0.0, 20.0] {
        let mut entity = Entity::new("box", Geometry::Brep(body.clone()), None);
        entity.placement = Transform::translation([x, 0.0, 0.0]);
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut app = MatrixApp::new(doc).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![]);
    let before = app.controller.document().clone();
    app.controller
        .dispatch("OM9-SOLID-006", InvocationOrigin::Menu)
        .unwrap();
    click(&mut app, &ctx, [5.0, 5.0, 10.0]);
    click(&mut app, &ctx, [25.0, 5.0, 10.0]);
    assert_eq!(app.controller.subselection().len(), 2);
    click_with_modifiers(
        &mut app,
        &ctx,
        [5.0, 5.0, 10.0],
        Modifiers {
            ctrl: true,
            ..Modifiers::NONE
        },
    );
    assert_eq!(app.controller.subselection().len(), 1);
    assert_eq!(app.controller.subselection()[0].root, ids[1]);
    click_with_modifiers(&mut app, &ctx, [5.0, 5.0, 10.0], Modifiers::NONE);
    assert_eq!(app.controller.subselection().len(), 2);
    app.controller.submit("Undo").unwrap();
    assert_eq!(app.controller.subselection().len(), 1);
    click(&mut app, &ctx, [5.0, 5.0, 10.0]);
    assert_eq!(app.controller.subselection().len(), 2);
    app.controller.submit("").unwrap();
    assert_eq!(app.controller.document().entities.len(), 4);
    for id in ids {
        let Geometry::Brep(remainder) = &app.controller.document().entities[&id].geometry else {
            panic!()
        };
        assert_eq!(
            om9_kernel::creation::measure(remainder, app.controller.coordinator().limits())
                .unwrap()
                .faces,
            5
        );
    }
    assert_eq!(app.controller.coordinator().undo_len(), 1);
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &before);
}

#[test]
fn viewport_extract_disambiguates_overlapping_sources() {
    let limits = Limits::default();
    let body = om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Box {
            x_mm: 10.0,
            y_mm: 10.0,
            z_mm: 10.0,
        },
        0.01,
        &limits,
    )
    .unwrap();
    let mut doc = Document::new("Overlapping extract faces");
    let mut ids = Vec::new();
    for name in ["First", "Second"] {
        let entity = Entity::new(name, Geometry::Brep(body.clone()), None);
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut app = MatrixApp::new(doc).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![]);
    let before = app.controller.document().clone();
    app.controller
        .dispatch("OM9-SOLID-006", InvocationOrigin::Menu)
        .unwrap();
    click(&mut app, &ctx, [5.0, 5.0, 10.0]);
    assert_eq!(app.ambiguous_candidates().len(), 2);
    assert!(app.controller.subselection().is_empty());
    let labels = app.ambiguous_labels();
    assert_ne!(labels[0], labels[1]);
    assert!(labels.iter().all(|label| label.contains("face")));
    app.controller.cancel();
    app.controller.submit("ExtractSrf").unwrap();
    assert!(app.choose_ambiguous(0).is_err());
    assert!(app.controller.subselection().is_empty());
    click(&mut app, &ctx, [5.0, 5.0, 10.0]);
    assert_eq!(app.ambiguous_candidates().len(), 2);
    let camera = app.controller.workspace.views[&ViewId::LookingDown]
        .camera
        .clone();
    app.controller
        .workspace
        .views
        .get_mut(&ViewId::LookingDown)
        .unwrap()
        .camera
        .eye[0] += 1.0;
    assert!(app.choose_ambiguous(0).is_err());
    app.controller
        .workspace
        .views
        .get_mut(&ViewId::LookingDown)
        .unwrap()
        .camera = camera;
    click(&mut app, &ctx, [5.0, 5.0, 10.0]);
    let choice = app
        .ambiguous_candidates()
        .iter()
        .position(|id| *id == ids[1])
        .unwrap();
    app.choose_ambiguous(choice).unwrap();
    assert_eq!(app.controller.subselection()[0].root, ids[1]);
    click_with_modifiers(
        &mut app,
        &ctx,
        [5.0, 5.0, 10.0],
        Modifiers {
            ctrl: true,
            ..Modifiers::NONE
        },
    );
    let choice = app
        .ambiguous_candidates()
        .iter()
        .position(|id| *id == ids[1])
        .unwrap();
    app.choose_ambiguous(choice).unwrap();
    assert!(app.controller.subselection().is_empty());
    click_with_modifiers(&mut app, &ctx, [5.0, 5.0, 10.0], Modifiers::NONE);
    let choice = app
        .ambiguous_candidates()
        .iter()
        .position(|id| *id == ids[1])
        .unwrap();
    app.choose_ambiguous(choice).unwrap();
    app.controller.submit("").unwrap();
    assert_eq!(
        app.controller.document().entities[&ids[0]],
        before.entities[&ids[0]]
    );
    let Geometry::Brep(remainder) = &app.controller.document().entities[&ids[1]].geometry else {
        panic!()
    };
    assert_eq!(
        om9_kernel::creation::measure(remainder, app.controller.coordinator().limits())
            .unwrap()
            .faces,
        5
    );
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &before);
}

#[test]
fn viewport_fillet_copy_handle_to_new_and_existing_location() {
    let limits = Limits::default();
    let shape = om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Box { x_mm: 10.0, y_mm: 10.0, z_mm: 10.0 }, 0.01, &limits,
    ).unwrap();
    let mut doc = Document::new("Copy fillet handle");
    let entity = Entity::new("box", Geometry::Brep(shape), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut app = MatrixApp::new(doc).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![]);
    let before = app.controller.document().clone();
    app.controller.dispatch("OM9-SOLID-007", InvocationOrigin::Menu).unwrap();
    app.controller.submit("NextRadius=1").unwrap();
    click(&mut app, &ctx, [0.0, 5.0, 10.0]);
    let reference = app.controller.subselection()[0].clone();
    app.controller.submit("").unwrap();
    app.controller.submit("AddHandle").unwrap();
    app.controller.submit("CurrentRadius=1.5").unwrap();
    click(&mut app, &ctx, [0.0, 2.5, 10.0]);
    app.controller.submit("").unwrap();
    let handles = app.controller.solid_fillet_handles();
    assert_eq!(handles.len(), 3);
    app.controller.submit("CopyHandle").unwrap();
    click(&mut app, &ctx, handles[2].1);
    assert!(app.controller.solid_fillet_copying_to_target());
    click(&mut app, &ctx, [0.0, 7.5, 10.0]);
    let handles = app.controller.solid_fillet_handles();
    assert_eq!(handles.len(), 4);
    assert_eq!(handles[2].2, 1.5);
    assert_eq!(handles[3].2, 1.5);
    click(&mut app, &ctx, handles[0].1);
    let handles = app.controller.solid_fillet_handles();
    assert_eq!(handles.len(), 4);
    assert_eq!(handles[0].2, 1.5);
    assert_eq!(handles[1].2, 1.0);
    assert!(app.controller.solid_fillet_copy_to_hit(&[(usize::MAX, 0.0)]).is_err());
    assert!(app.controller.solid_fillet_copying_to_target());
    assert_eq!(app.controller.solid_fillet_handles().len(), 4);
    let Geometry::Brep(original) = &before.entities[&id].geometry else { panic!() };
    let om9_scene::scene::Subelement::BrepEdge(key) = reference.subset else { panic!() };
    let edge = original.display().edges.iter().position(|e| *e == key).unwrap();
    let controls: Vec<_> = handles[2..].iter().map(|h| {
        let (fraction, _) = om9_kernel::editing::fillet_project_edge(original, edge, h.1, &limits).unwrap();
        (edge, fraction, h.2)
    }).collect();
    let expected = om9_kernel::editing::fillet_with_radius_controls(original, &[(edge, 1.5, 1.0)], &controls, 0.01, &limits).unwrap();
    let expected_volume = om9_kernel::creation::measure(&expected, &limits).unwrap().volume_mm3;
    let (preview, _) = app.controller.solid_fillet_preview().unwrap();
    assert!((om9_kernel::creation::measure(preview, &limits).unwrap().volume_mm3 - expected_volume).abs() < 1e-5);
    assert_eq!(app.controller.coordinator().undo_len(), 0);
    app.controller.submit("").unwrap();
    assert!(!app.controller.solid_fillet_copying_to_target());
    app.controller.submit("").unwrap();
    assert_eq!(app.controller.coordinator().undo_len(), 1);
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &before);
}

#[test]
fn viewport_fillet_removes_only_added_handles() {
    use om9_app::command_options::CommandOptionInterface;
    let limits = Limits::default();
    let shape = om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Box { x_mm: 10.0, y_mm: 10.0, z_mm: 10.0 }, 0.01, &limits,
    ).unwrap();
    let mut doc = Document::new("Remove fillet handles");
    let entity = Entity::new("box", Geometry::Brep(shape), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut app = MatrixApp::new(doc).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![]);
    let before = app.controller.document().clone();
    app.controller.dispatch("OM9-SOLID-007", InvocationOrigin::Menu).unwrap();
    app.controller.submit("NextRadius=1").unwrap();
    click(&mut app, &ctx, [0.0, 5.0, 10.0]);
    app.controller.submit("").unwrap();
    assert!(!app.controller.command_options().iter().any(|o| o.id.name() == "RemoveHandle"));
    app.controller.submit("AddHandle").unwrap();
    app.controller.submit("CurrentRadius=1.5").unwrap();
    click(&mut app, &ctx, [0.0, 2.5, 10.0]);
    click(&mut app, &ctx, [0.0, 7.5, 10.0]);
    app.controller.submit("").unwrap();
    assert!(app.controller.command_options().iter().any(|o| o.id.name() == "RemoveHandle"));
    let handles = app.controller.solid_fillet_handles();
    assert_eq!(handles.len(), 4);
    app.controller.submit("RemoveHandle").unwrap();
    assert!(app.controller.solid_fillet_select_handle(handles[0].0).is_err());
    assert_eq!(app.controller.solid_fillet_handles().len(), 4);
    click(&mut app, &ctx, handles[2].1);
    assert_eq!(app.controller.solid_fillet_handles().len(), 3);
    click(&mut app, &ctx, handles[3].1);
    assert_eq!(app.controller.solid_fillet_handles().len(), 2);
    app.controller.submit("").unwrap();
    assert!(!app.controller.command_options().iter().any(|o| o.id.name() == "RemoveHandle"));
    let (preview, _) = app.controller.solid_fillet_preview().unwrap();
    let volume = om9_kernel::creation::measure(preview, &limits).unwrap().volume_mm3;
    let expected = 1000.0 - (1.0 - std::f64::consts::PI / 4.0) * 10.0;
    assert!((volume - expected).abs() < 1e-5);
    assert_eq!(app.controller.document(), &before);
    app.controller.submit("").unwrap();
    assert_eq!(app.controller.coordinator().undo_len(), 1);
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &before);
}

#[test]
fn viewport_fillet_adds_and_edits_interior_handle_on_placed_edge() {
    let limits = Limits::default();
    let shape = om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Box { x_mm: 10.0, y_mm: 10.0, z_mm: 10.0 }, 0.01, &limits,
    ).unwrap();
    let mut doc = Document::new("Add fillet handle");
    let mut entity = Entity::new("box", Geometry::Brep(shape), None);
    entity.placement = Transform::translation([5.0, 0.0, 0.0]);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut app = MatrixApp::new(doc).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![]);
    let before = app.controller.document().clone();
    app.controller.dispatch("OM9-SOLID-007", InvocationOrigin::Menu).unwrap();
    app.controller.submit("NextRadius=1").unwrap();
    click(&mut app, &ctx, [5.0, 5.0, 10.0]);
    let reference = app.controller.subselection()[0].clone();
    app.controller.submit("").unwrap();
    app.controller.submit("AddHandle").unwrap();
    app.controller.submit("CurrentRadius=1.5").unwrap();
    click(&mut app, &ctx, [5.0, 2.5, 10.0]);
    assert_eq!(app.controller.solid_fillet_handles().len(), 3);
    let handle = app.controller.solid_fillet_handles()[2];
    assert!((handle.1[0] - 5.0).abs() < 1e-6 && (handle.1[1] - 2.5).abs() < 0.1);
    assert!((handle.1[2] - 10.0).abs() < 1e-6 && handle.2 == 1.5);
    let Geometry::Brep(original) = &before.entities[&id].geometry else { panic!() };
    let om9_scene::scene::Subelement::BrepEdge(key) = reference.subset else { panic!() };
    let edge = original.display().edges.iter().position(|e| *e == key).unwrap();
    let local = [handle.1[0] - 5.0, handle.1[1], handle.1[2]];
    let (fraction, _) = om9_kernel::editing::fillet_project_edge(original, edge, local, &limits).unwrap();
    let expected = om9_kernel::editing::fillet_with_radius_controls(original, &[(edge, 1.0, 1.0)], &[(edge, fraction, 1.5)], 0.01, &limits).unwrap();
    let expected_volume = om9_kernel::creation::measure(&expected, &limits).unwrap().volume_mm3;
    let (preview, _) = app.controller.solid_fillet_preview().unwrap();
    let actual = om9_kernel::creation::measure(preview, &limits).unwrap().volume_mm3;
    assert!((actual - expected_volume).abs() < 1e-5);
    assert!(app.controller.solid_fillet_add_handle(reference.clone(), handle.1).is_err());
    assert_eq!(app.controller.solid_fillet_handles().len(), 3);
    click(&mut app, &ctx, [5.0, 7.5, 10.0]);
    assert_eq!(app.controller.solid_fillet_handles().len(), 4);
    app.controller.submit("").unwrap();
    app.controller.submit("SetAll=1.1").unwrap();
    assert!(app.controller.solid_fillet_handles().iter().all(|h| h.2 == 1.1));
    click(&mut app, &ctx, handle.1);
    assert_eq!(app.controller.solid_fillet_current_radius(), Some(1.1));
    assert!(app.controller.submit("CurrentRadius=1000").is_err());
    assert_eq!(app.controller.solid_fillet_current_radius(), Some(1.1));
    app.controller.submit("LinkHandles=Yes").unwrap();
    app.controller.submit("CurrentRadius=1.2").unwrap();
    assert!(app.controller.solid_fillet_handles().iter().all(|h| h.2 == 1.2));
    app.controller.submit("LinkHandles=No").unwrap();
    app.controller.submit("CurrentRadius=1.3").unwrap();
    let radii: Vec<_> = app.controller.solid_fillet_handles().iter().map(|h| h.2).collect();
    assert_eq!(radii, vec![1.2, 1.2, 1.3, 1.2]);
    app.controller.submit("").unwrap();
    assert_eq!(app.controller.coordinator().undo_len(), 1);
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &before);
}

#[test]
fn viewport_fillet_selects_shared_tangent_vertex_handle() {
    let limits = Limits::default();
    let profile = om9_kernel::creation::Profile::new(vec![
        [0.0, 0.0, 0.0], [5.0, 0.0, 0.0], [10.0, 0.0, 0.0],
        [10.0, 10.0, 0.0], [0.0, 10.0, 0.0],
    ]).unwrap();
    let shape = om9_kernel::creation::extrude(&profile, [0.0, 0.0, 10.0], true, 0.01, &limits).unwrap();
    let edges: Vec<_> = (0..shape.display().edges.len()).filter_map(|index| {
        let (a, b) = om9_kernel::editing::oriented_edge_endpoints(&shape, index, &limits).ok()?;
        (a[1].abs() < 1e-6 && b[1].abs() < 1e-6 && (a[2]-10.0).abs() < 1e-6 &&
         (b[2]-10.0).abs() < 1e-6 && ((a[0]-b[0]).abs()-5.0).abs() < 1e-6).then_some(shape.display().edges[index])
    }).collect();
    assert_eq!(edges.len(), 2);
    let mut doc = Document::new("Shared fillet handle");
    let entity = Entity::new("prism", Geometry::Brep(shape), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut app = MatrixApp::new(doc).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![]);
    let before = app.controller.document().clone();
    app.controller.dispatch("OM9-SOLID-007", InvocationOrigin::Menu).unwrap();
    app.controller.submit("NextRadius=1").unwrap();
    for edge in edges {
        let object = app.controller.scene().unwrap().objects.iter().find(|o| o.root == id && o.path.is_empty()).unwrap();
        let reference = om9_app::session::SelectedSubobject {
            root: id, path: vec![], subset: om9_scene::scene::Subelement::BrepEdge(edge),
            source: object.source, world_epoch: app.controller.scene().unwrap().epoch,
        };
        app.controller.solid_fillet_pick_edge(reference).unwrap();
    }
    app.controller.submit("").unwrap();
    click(&mut app, &ctx, [5.0, 0.0, 10.0]);
    assert_eq!(app.controller.solid_fillet_current_radius(), Some(1.0));
    app.controller.submit("CurrentRadius=1.5").unwrap();
    let shared: Vec<_> = app.controller.solid_fillet_handles().iter().filter(|h| (h.1[0]-5.0).abs() < 1e-6).map(|h| h.2).collect();
    assert_eq!(shared, vec![1.5, 1.5]);
    app.controller.submit("").unwrap();
    assert_eq!(app.controller.coordinator().undo_len(), 1);
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &before);
}

#[test]
fn viewport_fillet_edge_selects_then_previews_then_commits() {
    use om9_app::command_options::{CommandOptionEffect, CommandOptionInterface};
    let limits = Limits::default();
    let shape = om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Box {
            x_mm: 10.0,
            y_mm: 10.0,
            z_mm: 10.0,
        },
        0.01,
        &limits,
    )
    .unwrap();
    let mut doc = Document::new("Viewport fillet");
    let mut entity = Entity::new("box", Geometry::Brep(shape), None);
    entity.placement = Transform::translation([5.0, 0.0, 0.0]);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut app = MatrixApp::new(doc).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![]);
    let before = app.controller.document().clone();
    app.controller
        .dispatch("OM9-SOLID-007", InvocationOrigin::Menu)
        .unwrap();
    app.controller.submit("NextRadius=1").unwrap();
    click(&mut app, &ctx, [5.0, 5.0, 10.0]);
    assert_eq!(app.controller.subselection().len(), 1);
    app.controller.submit("NextRadius=2").unwrap();
    click(&mut app, &ctx, [15.0, 5.0, 10.0]);
    assert_eq!(app.controller.subselection().len(), 2);
    let Geometry::Brep(original) = &before.entities[&id].geometry else {
        panic!()
    };
    let selected: Vec<_> = app
        .controller
        .subselection()
        .iter()
        .map(|reference| {
            let om9_scene::scene::Subelement::BrepEdge(key) = reference.subset else {
                panic!()
            };
            original
                .display()
                .edges
                .iter()
                .position(|edge| *edge == key)
                .unwrap()
        })
        .collect();
    app.controller.submit("").unwrap();
    let (preview, placement) = app.controller.solid_fillet_preview().unwrap();
    assert_eq!(placement, before.entities[&id].placement);
    let m = om9_kernel::creation::measure(preview, app.controller.coordinator().limits()).unwrap();
    let expected = 1000.0 - (1.0 - std::f64::consts::PI / 4.0) * 50.0;
    assert!(
        m.closed && (m.volume_mm3 - expected).abs() < 1e-5,
        "{}",
        m.volume_mm3
    );
    assert_eq!(app.controller.document(), &before);
    assert_eq!(app.controller.coordinator().undo_len(), 0);
    let handles = app.controller.solid_fillet_handles();
    assert_eq!(handles.len(), 4);
    let first = handles
        .iter()
        .find(|(index, _, _, _)| *index == 0)
        .unwrap()
        .1;
    click(&mut app, &ctx, first);
    assert_eq!(app.controller.solid_fillet_current_radius(), Some(1.0));
    let option = app
        .controller
        .command_options()
        .into_iter()
        .find(|o| o.id.name() == "CurrentRadius")
        .unwrap();
    assert!(option.shortcut.is_some());
    assert_eq!(
        app.controller.activate_command_option(option.id).unwrap(),
        CommandOptionEffect::Draft("CurrentRadius=".into())
    );
    app.controller.submit("CurrentRadius=1.5").unwrap();
    let (edited, _) = app.controller.solid_fillet_preview().unwrap();
    let edited_volume =
        om9_kernel::creation::measure(edited, app.controller.coordinator().limits())
            .unwrap()
            .volume_mm3;
    let expected_brep = om9_kernel::editing::fillet_with_end_radii(
        original,
        &[(selected[0], 1.5, 1.0), (selected[1], 2.0, 2.0)],
        0.01,
        app.controller.coordinator().limits(),
    )
    .unwrap();
    let expected =
        om9_kernel::creation::measure(&expected_brep, app.controller.coordinator().limits())
            .unwrap()
            .volume_mm3;
    assert!((edited_volume - expected).abs() < 1e-5, "{edited_volume}");
    assert!(app.controller.submit("CurrentRadius=1000").is_err());
    assert_eq!(app.controller.solid_fillet_current_radius(), Some(1.5));
    let (preview_after_error, _) = app.controller.solid_fillet_preview().unwrap();
    assert!(
        (om9_kernel::creation::measure(preview_after_error, app.controller.coordinator().limits())
            .unwrap()
            .volume_mm3
            - edited_volume)
            .abs()
            < 1e-7
    );
    let second = app
        .controller
        .solid_fillet_handles()
        .iter()
        .find(|(index, _, _, _)| *index == 1)
        .unwrap()
        .1;
    click(&mut app, &ctx, second);
    assert_eq!(app.controller.solid_fillet_current_radius(), Some(1.0));
    app.controller.submit("CurrentRadius=1.8").unwrap();
    let (both, _) = app.controller.solid_fillet_preview().unwrap();
    let expected_brep = om9_kernel::editing::fillet_with_end_radii(
        original,
        &[(selected[0], 1.5, 1.8), (selected[1], 2.0, 2.0)],
        0.01,
        app.controller.coordinator().limits(),
    )
    .unwrap();
    let actual = om9_kernel::creation::measure(both, app.controller.coordinator().limits())
        .unwrap()
        .volume_mm3;
    let expected =
        om9_kernel::creation::measure(&expected_brep, app.controller.coordinator().limits())
            .unwrap()
            .volume_mm3;
    assert!((actual - expected).abs() < 1e-5);
    let option = app.controller.command_options().into_iter().find(|o| o.id.name() == "SetAll").unwrap();
    assert!(option.shortcut.is_some());
    assert_eq!(app.controller.activate_command_option(option.id).unwrap(), CommandOptionEffect::Draft("SetAll=".into()));
    app.controller.submit("SetAll=1.25").unwrap();
    assert!(app.controller.solid_fillet_handles().iter().all(|handle| handle.2 == 1.25));
    let (all, _) = app.controller.solid_fillet_preview().unwrap();
    let all_volume = om9_kernel::creation::measure(all, app.controller.coordinator().limits()).unwrap().volume_mm3;
    let expected = 1000.0 - (1.0 - std::f64::consts::PI / 4.0) * 10.0 * 2.0 * 1.25_f64.powi(2);
    assert!((all_volume - expected).abs() < 1e-5);
    assert!(app.controller.submit("SetAll=1000").is_err());
    assert!(app.controller.solid_fillet_handles().iter().all(|handle| handle.2 == 1.25));
    let (retained, _) = app.controller.solid_fillet_preview().unwrap();
    let retained_volume = om9_kernel::creation::measure(retained, app.controller.coordinator().limits()).unwrap().volume_mm3;
    assert!((retained_volume - all_volume).abs() < 1e-7);
    assert_eq!(app.controller.document(), &before);
    app.controller.cancel();
    assert!(app.controller.solid_fillet_preview().is_none());
    assert_eq!(app.controller.document(), &before);
    app.controller
        .dispatch("OM9-SOLID-007", InvocationOrigin::Menu)
        .unwrap();
    app.controller.submit("NextRadius=1").unwrap();
    click(&mut app, &ctx, [5.0, 5.0, 10.0]);
    app.controller.submit("").unwrap();
    let handle = app.controller.solid_fillet_handles()[0].1;
    click(&mut app, &ctx, handle);
    app.controller.submit("CurrentRadius=1.5").unwrap();
    app.controller.submit("SelectEdges").unwrap();
    app.controller.submit("NextRadius=2").unwrap();
    click(&mut app, &ctx, [15.0, 5.0, 10.0]);
    assert_eq!(app.controller.subselection().len(), 2);
    app.controller.submit("").unwrap();
    app.controller.submit("").unwrap();
    assert_eq!(app.controller.coordinator().undo_len(), 1);
    let Geometry::Brep(result) = &app.controller.document().entities[&id].geometry else {
        panic!()
    };
    let m = om9_kernel::creation::measure(result, app.controller.coordinator().limits()).unwrap();
    let expected_brep = om9_kernel::editing::fillet_with_end_radii(
        original,
        &[(selected[0], 1.5, 1.0), (selected[1], 2.0, 2.0)],
        0.01,
        app.controller.coordinator().limits(),
    )
    .unwrap();
    let expected =
        om9_kernel::creation::measure(&expected_brep, app.controller.coordinator().limits())
            .unwrap()
            .volume_mm3;
    assert!((m.volume_mm3 - expected).abs() < 1e-5);
    app.controller.undo().unwrap();
    assert_eq!(app.controller.document(), &before);
}

#[test]
fn viewport_extract_selection_undo_and_cancel_do_not_commit() {
    let limits = Limits::default();
    let body = om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Box {
            x_mm: 10.0,
            y_mm: 10.0,
            z_mm: 10.0,
        },
        0.01,
        &limits,
    )
    .unwrap();
    let mut doc = Document::new("Extract pending");
    let entity = Entity::new("box", Geometry::Brep(body), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut app = MatrixApp::new(doc).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![]);
    let before = app.controller.document().clone();
    app.controller
        .dispatch("OM9-SOLID-006", InvocationOrigin::Menu)
        .unwrap();
    click(&mut app, &ctx, [5.0, 5.0, 10.0]);
    app.controller.select_objects(&[]).unwrap();
    assert!(app.controller.submit("").is_err());
    assert_eq!(app.controller.document(), &before);
    app.controller.select_objects(&[id]).unwrap();
    app.controller.submit("Undo").unwrap();
    assert!(app.controller.subselection().is_empty());
    assert!(app.controller.submit("").is_err());
    click(&mut app, &ctx, [5.0, 5.0, 10.0]);
    app.controller.cancel();
    assert!(app.controller.subselection().is_empty());
    assert_eq!(app.controller.document(), &before);
    assert_eq!(app.controller.coordinator().undo_len(), 0);
}

#[test]
fn viewport_clicks_select_difference_targets_and_cutters() {
    let limits = Limits::default();
    let mut doc = Document::new("Difference viewport picks");
    let mut ids = Vec::new();
    for (side, placement) in [(10.0, [0.0, 0.0, 0.0]), (4.0, [8.0, 3.0, 0.0])] {
        let body = om9_kernel::creation::primitive(
            om9_kernel::creation::Primitive::Box {
                x_mm: side,
                y_mm: side,
                z_mm: side,
            },
            0.01,
            &limits,
        )
        .unwrap();
        let mut entity = Entity::new("body", Geometry::Brep(body), None);
        entity.placement = Transform::translation(placement);
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut app = MatrixApp::new(doc).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![]);
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller
        .dispatch("OM9-SOLID-001", InvocationOrigin::Menu)
        .unwrap();
    app.controller.submit("DeleteInput=Yes").unwrap();
    click(&mut app, &ctx, [0.0, 5.0, 10.0]);
    assert!(app.controller.selected().contains(&ids[0]));
    app.controller.submit("").unwrap();
    click(&mut app, &ctx, [12.0, 5.0, 4.0]);
    assert!(app.controller.selected().contains(&ids[1]));
    app.controller.submit("").unwrap();
    assert_eq!(app.controller.document().entities.len(), 1);
    let Geometry::Brep(body) = &app
        .controller
        .document()
        .entities
        .values()
        .next()
        .unwrap()
        .geometry
    else {
        panic!("BRep result")
    };
    let volume = om9_kernel::creation::measure(body, app.controller.coordinator().limits())
        .unwrap()
        .volume_mm3;
    assert!((volume - 968.0).abs() < 1e-6, "{volume}");
}

#[test]
fn overlapping_viewport_candidates_route_into_difference_cutter_selection() {
    let limits = Limits::default();
    let mut doc = Document::new("Overlapping difference picks");
    let mut ids = Vec::new();
    for (side, placement) in [(10.0, [0.0, 0.0, 0.0]), (4.0, [3.0, 3.0, 0.0])] {
        let body = om9_kernel::creation::primitive(
            om9_kernel::creation::Primitive::Box {
                x_mm: side,
                y_mm: side,
                z_mm: side,
            },
            0.01,
            &limits,
        )
        .unwrap();
        let mut entity = Entity::new("body", Geometry::Brep(body), None);
        entity.placement = Transform::translation(placement);
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut app = MatrixApp::new(doc).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![]);
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller.submit("BooleanDifference").unwrap();
    app.controller.submit("DeleteInput=Yes").unwrap();
    click(&mut app, &ctx, [0.0, 5.0, 10.0]);
    assert!(app.controller.selected().contains(&ids[0]));
    app.controller.submit("").unwrap();
    click(&mut app, &ctx, [3.0, 5.0, 4.0]);
    let index = app
        .ambiguous_candidates()
        .iter()
        .position(|id| *id == ids[1])
        .expect("cutter candidate");
    assert!(app.ambiguous_candidates().len() > 1);
    app.controller.cancel();
    app.controller.submit("BooleanDifference").unwrap();
    app.controller.submit("DeleteInput=Yes").unwrap();
    app.controller.solid_difference_pick(ids[0], false).unwrap();
    app.controller.submit("").unwrap();
    assert!(app.choose_ambiguous(index).is_err());
    assert!(app.controller.selected().is_empty());
    click(&mut app, &ctx, [3.0, 5.0, 4.0]);
    let index = app
        .ambiguous_candidates()
        .iter()
        .position(|id| *id == ids[1])
        .expect("current cutter candidate");
    app.choose_ambiguous(index).unwrap();
    assert!(app.controller.selected().contains(&ids[1]));
    app.controller.submit("").unwrap();
    let Geometry::Brep(body) = &app
        .controller
        .document()
        .entities
        .values()
        .next()
        .unwrap()
        .geometry
    else {
        panic!("BRep result")
    };
    let volume = om9_kernel::creation::measure(body, app.controller.coordinator().limits())
        .unwrap()
        .volume_mm3;
    assert!((volume - 936.0).abs() < 1e-6, "{volume}");
}

#[test]
fn viewport_clicks_collect_boolean_union_inputs() {
    let limits = Limits::default();
    let mut doc = Document::new("Union viewport picks");
    let mut ids = Vec::new();
    for x in [0.0, 5.0] {
        let body = om9_kernel::creation::primitive(
            om9_kernel::creation::Primitive::Box {
                x_mm: 10.0,
                y_mm: 10.0,
                z_mm: 10.0,
            },
            0.01,
            &limits,
        )
        .unwrap();
        let mut entity = Entity::new("body", Geometry::Brep(body), None);
        entity.placement = Transform::translation([x, 0.0, 0.0]);
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut app = MatrixApp::new(doc).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![]);
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller
        .dispatch("OM9-SOLID-003", InvocationOrigin::Menu)
        .unwrap();
    click(&mut app, &ctx, [0.0, 5.0, 10.0]);
    click(&mut app, &ctx, [15.0, 5.0, 10.0]);
    assert!(ids.iter().all(|id| app.controller.selected().contains(id)));
    app.controller.submit("").unwrap();
    assert_eq!(app.controller.document().entities.len(), 3);
    let entity = app
        .controller
        .document()
        .entities
        .iter()
        .find(|(id, _)| !ids.contains(id))
        .unwrap()
        .1;
    let Geometry::Brep(body) = &entity.geometry else {
        panic!("BRep result")
    };
    let volume = om9_kernel::creation::measure(body, app.controller.coordinator().limits())
        .unwrap()
        .volume_mm3;
    assert!((volume - 1500.0).abs() < 1e-6, "{volume}");
}

#[test]
fn viewport_clicks_collect_boolean_intersection_sets() {
    let limits = Limits::default();
    let mut doc = Document::new("Intersection viewport picks");
    let mut ids = Vec::new();
    for x in [0.0, 5.0] {
        let body = om9_kernel::creation::primitive(
            om9_kernel::creation::Primitive::Box {
                x_mm: 10.0,
                y_mm: 10.0,
                z_mm: 10.0,
            },
            0.01,
            &limits,
        )
        .unwrap();
        let mut entity = Entity::new("body", Geometry::Brep(body), None);
        entity.placement = Transform::translation([x, 0.0, 0.0]);
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut app = MatrixApp::new(doc).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![]);
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller
        .dispatch("OM9-SOLID-002", InvocationOrigin::Menu)
        .unwrap();
    click(&mut app, &ctx, [0.0, 5.0, 10.0]);
    assert!(app.controller.selected().contains(&ids[0]));
    app.controller.submit("").unwrap();
    click(&mut app, &ctx, [15.0, 5.0, 10.0]);
    assert!(app.controller.selected().contains(&ids[1]));
    app.controller.submit("").unwrap();
    assert_eq!(app.controller.document().entities.len(), 3);
    let entity = app
        .controller
        .document()
        .entities
        .iter()
        .find(|(id, _)| !ids.contains(id))
        .unwrap()
        .1;
    let Geometry::Brep(body) = &entity.geometry else {
        panic!("BRep result")
    };
    let volume = om9_kernel::creation::measure(body, app.controller.coordinator().limits())
        .unwrap()
        .volume_mm3;
    assert!((volume - 500.0).abs() < 1e-6, "{volume}");
}

#[test]
fn boolean_two_common_affine_preview_is_painted_at_world_location() {
    let limits = Limits::default();
    let body = om9_kernel::creation::primitive(om9_kernel::creation::Primitive::Box { x_mm: 2., y_mm: 2., z_mm: 2. }, 0.01, &limits).unwrap();
    let shifted = om9_kernel::transform_ops::transform_brep(&body, &Transform::translation([1., 0., 0.]), 0.01, &limits).unwrap();
    let placement = Transform([[2., 0., 0., 10.], [0., 1., 0., 4.], [0., 0., 1., 0.], [0., 0., 0., 1.]]);
    let mut doc = Document::new("Boolean Two affine preview");
    let mut a = Entity::new("first", Geometry::Brep(body), None);
    let mut b = Entity::new("second", Geometry::Brep(shifted), None);
    a.placement = placement; b.placement = placement;
    let ids = [a.id, b.id]; doc.entities.insert(a.id, a); doc.entities.insert(b.id, b);
    let mut app = MatrixApp::new(doc).unwrap();
    let ctx = egui::Context::default(); frame(&mut app, &ctx, vec![]); frame(&mut app, &ctx, vec![]);
    app.controller.select_objects(&ids).unwrap();
    app.controller.submit("Boolean2Objects").unwrap(); app.controller.submit("").unwrap();
    let (preview, _) = app.controller.solid_boolean_two_preview().unwrap();
    let rect = app.view_rects[&ViewId::LookingDown];
    let viewport = Viewport { origin: [rect.min.x as f64, rect.min.y as f64], size: [rect.width() as f64, rect.height() as f64] };
    let camera = &app.controller.workspace.views[&ViewId::LookingDown].camera;
    let expected: Vec<_> = preview.display().segments.iter().filter_map(|segment| {
        let a = placement.point(preview.display().points[segment[0] as usize]).ok()?;
        let b = placement.point(preview.display().points[segment[1] as usize]).ok()?;
        let a = camera.project(a, viewport).ok()??.screen;
        let b = camera.project(b, viewport).ok()??.screen;
        Some([Pos2::new(a[0] as f32, a[1] as f32), Pos2::new(b[0] as f32, b[1] as f32)])
    }).collect();
    assert!(!expected.is_empty());
    let mut output = ctx.run_ui(egui::RawInput { screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 900.))), ..Default::default() }, |ui| app.show(ui));
    output.textures_delta.clear();
    let painted = output.shapes.iter().any(|shape| match &shape.shape {
        egui::Shape::LineSegment { points, stroke } if stroke.color == egui::Color32::from_rgb(0, 200, 130) => expected.iter().any(|line| points[0].distance(line[0]) < 0.1 && points[1].distance(line[1]) < 0.1),
        _ => false,
    });
    assert!(painted, "preview line should be projected after applying its placement");
}

#[test]
fn viewport_clicks_collect_boolean_two_objects() {
    let limits = Limits::default();
    let mut doc = Document::new("Boolean Two Objects viewport picks");
    let mut ids = Vec::new();
    for x in [0.0, 5.0] {
        let body = om9_kernel::creation::primitive(
            om9_kernel::creation::Primitive::Box {
                x_mm: 10.0,
                y_mm: 10.0,
                z_mm: 10.0,
            },
            0.01,
            &limits,
        )
        .unwrap();
        let mut entity = Entity::new("body", Geometry::Brep(body), None);
        entity.placement = Transform::translation([x, 0.0, 0.0]);
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut app = MatrixApp::new(doc).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![]);
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller
        .dispatch("OM9-SOLID-004", InvocationOrigin::Menu)
        .unwrap();
    click(&mut app, &ctx, [0.0, 5.0, 10.0]);
    click(&mut app, &ctx, [15.0, 5.0, 10.0]);
    assert!(ids.iter().all(|id| app.controller.selected().contains(id)));
    assert_eq!(app.controller.coordinator().undo_len(), 0);
    app.controller.submit("DeleteInput=Yes").unwrap();
    app.controller.submit("D").unwrap();
    assert_eq!(
        app.controller.solid_boolean_two_preview().unwrap().1,
        "Difference"
    );
    click(&mut app, &ctx, [0.0, 5.0, 10.0]);
    assert_eq!(
        app.controller.solid_boolean_two_preview().unwrap().1,
        "ReverseDifference"
    );
    app.controller.submit("D").unwrap();
    app.controller.submit("").unwrap();
    assert_eq!(app.controller.document().entities.len(), 1);
    let Geometry::Brep(body) = &app
        .controller
        .document()
        .entities
        .values()
        .next()
        .unwrap()
        .geometry
    else {
        panic!("BRep result")
    };
    let volume = om9_kernel::creation::measure(body, app.controller.coordinator().limits())
        .unwrap()
        .volume_mm3;
    assert!((volume - 500.0).abs() < 1e-6, "{volume}");
    app.controller.undo().unwrap();
    assert!(
        ids.iter()
            .all(|id| app.controller.document().entities.contains_key(id))
    );
}

#[test]
fn viewport_clicks_collect_boolean_split_target_and_cutter() {
    let limits = Limits::default();
    let mut doc = Document::new("Boolean Split viewport");
    let mut ids = Vec::new();
    for x in [0.0, 5.0] {
        let body = om9_kernel::creation::primitive(
            om9_kernel::creation::Primitive::Box {
                x_mm: 10.0,
                y_mm: 10.0,
                z_mm: 10.0,
            },
            0.01,
            &limits,
        )
        .unwrap();
        let mut entity = Entity::new("body", Geometry::Brep(body), None);
        entity.placement = Transform::translation([x, 0.0, 0.0]);
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let mut app = MatrixApp::new(doc).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![]);
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller
        .dispatch("SolidBooleanSplit", InvocationOrigin::Menu)
        .unwrap();
    click(&mut app, &ctx, [0.0, 5.0, 10.0]);
    assert!(app.controller.selected().contains(&ids[0]));
    app.controller.submit("").unwrap();
    click(&mut app, &ctx, [15.0, 5.0, 10.0]);
    assert!(app.controller.selected().contains(&ids[1]));
    app.controller.submit("").unwrap();
    assert_eq!(app.controller.document().entities.len(), 4);
    assert!(
        ids.iter()
            .all(|id| app.controller.document().entities.contains_key(id))
    );
    let volumes: Vec<_> = app
        .controller
        .document()
        .entities
        .iter()
        .filter(|(id, _)| !ids.contains(id))
        .map(|(_, entity)| match &entity.geometry {
            Geometry::Brep(body) => {
                om9_kernel::creation::measure(body, app.controller.coordinator().limits())
                    .unwrap()
                    .volume_mm3
            }
            _ => panic!("BRep piece"),
        })
        .collect();
    assert_eq!(volumes.len(), 2);
    assert!(volumes.iter().all(|v| (*v - 500.0).abs() < 1e-6));
}

#[test]
fn viewport_click_collects_create_solid_surface_and_rejects_open_set() {
    let limits = Limits::default();
    let profile = om9_kernel::creation::Profile::new(vec![
        [0.0, 0.0, 0.0],
        [10.0, 0.0, 0.0],
        [10.0, 10.0, 0.0],
        [0.0, 10.0, 0.0],
    ])
    .unwrap();
    let face = om9_kernel::creation::planar(&profile, 0.01, &limits).unwrap();
    let entity = Entity::new("surface", Geometry::Brep(face), None);
    let id = entity.id;
    let mut doc = Document::new("Create Solid viewport");
    doc.entities.insert(id, entity);
    let before = doc.clone();
    let mut app = MatrixApp::new(doc).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![]);
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller
        .dispatch("SolidCreateSolid", InvocationOrigin::Menu)
        .unwrap();
    click(&mut app, &ctx, [0.0, 5.0, 0.0]);
    assert!(app.controller.selected().contains(&id));
    assert!(app.controller.submit("").is_err());
    assert_eq!(app.controller.document(), &before);
    assert_eq!(app.controller.coordinator().undo_len(), 0);
    assert!(app.controller.is_running());
}

#[test]
fn viewport_click_selects_open_shell_for_cap_planar() {
    let limits = Limits::default();
    let body = om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Box {
            x_mm: 10.0,
            y_mm: 10.0,
            z_mm: 10.0,
        },
        0.01,
        &limits,
    )
    .unwrap();
    let shell = om9_kernel::editing::remove_face(&body, 0, 0.01, &limits).unwrap();
    let entity = Entity::new("open box", Geometry::Brep(shell), None);
    let id = entity.id;
    let mut doc = Document::new("Cap viewport");
    doc.entities.insert(id, entity);
    let mut app = MatrixApp::new(doc).unwrap();
    let ctx = egui::Context::default();
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, vec![]);
    app.controller.workspace.active = ViewId::LookingDown;
    app.controller
        .dispatch("OM9-SOLID-005", InvocationOrigin::Menu)
        .unwrap();
    click(&mut app, &ctx, [0.0, 5.0, 10.0]);
    assert!(app.controller.selected().contains(&id));
    app.controller.submit("").unwrap();
    let Geometry::Brep(body) = &app.controller.document().entities[&id].geometry else {
        panic!("capped BRep")
    };
    let m = om9_kernel::creation::measure(body, app.controller.coordinator().limits()).unwrap();
    assert!(m.closed && m.solids == 1 && (m.volume_mm3 - 1000.0).abs() < 1e-6);
    assert_eq!(app.controller.coordinator().undo_len(), 1);
}
