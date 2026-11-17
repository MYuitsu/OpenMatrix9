#![cfg(feature = "occt-brep")]
use om9_app::ui::MatrixApp;
use om9_core::model::Document;
use om9_app::display::{DisplayPreferences, ViewportDetail};
use om9_core::geometry::Geometry;

#[test]
fn handled_navigation_keeps_keyboard_ime_focus_and_primary_clicks() {
    use eframe::egui::{Event,ImeEvent,Key,Modifiers,PointerButton,Pos2};
    let kept=vec![Event::Text("Sphere".into()),Event::Paste("5,0,0".into()),
        Event::Ime(ImeEvent::Commit("cầu".into())),Event::WindowFocused(false),
        Event::Key{key:Key::Enter,physical_key:None,pressed:true,repeat:false,modifiers:Modifiers::NONE},
        Event::PointerButton{pos:Pos2::ZERO,button:PointerButton::Primary,pressed:true,modifiers:Modifiers::NONE}];
    let mut events=kept.clone();
    events.insert(0,Event::PointerMoved(Pos2::new(10.,20.)));
    events.insert(3,Event::PointerButton{pos:Pos2::ZERO,button:PointerButton::Secondary,pressed:false,modifiers:Modifiers::NONE});
    om9_app::display::consume_handled_navigation_events(&mut events);
    assert_eq!(events,kept);
}

#[test]
fn light_motion_quality_restores_full_idle_and_respects_pressure() {
    use om9_app::frame_scheduler::PreviewQuality::{Q0,Q1,Q2,Q3};
    assert_eq!(ViewportDetail::Light.motion_quality(true,Q0),Q2);
    assert_eq!(ViewportDetail::Light.motion_quality(true,Q1),Q2);
    assert_eq!(ViewportDetail::Light.motion_quality(true,Q3),Q3);
    assert_eq!(ViewportDetail::Light.motion_quality(false,Q0),Q0);
    assert_eq!(ViewportDetail::High.motion_quality(true,Q0),Q0);
    let motion=ViewportDetail::Light.motion_quality(true,Q0);
    assert_eq!(motion.samples(),1);assert_eq!(motion.scale(),0.7);
    assert_eq!(ViewportDetail::Light.motion_quality(false,Q0).samples(),4);
}

fn drain(app: &mut MatrixApp) {
 let until=std::time::Instant::now()+std::time::Duration::from_secs(30);
 let original=app.controller.scene().unwrap().publication_id();
 while app.controller.boolean_union_worker_pending() || app.controller.viewport_detail_pending() {
  assert!(std::time::Instant::now()<until,"worker timeout");
  if let Some(result)=app.controller.poll_boolean_union_worker(){result.unwrap();}
  if let Some(result)=app.controller.poll_viewport_detail(){result.unwrap();}
  if app.controller.viewport_detail_pending(){assert_eq!(app.controller.scene().unwrap().publication_id(),original,"partial profile publication");}
  std::thread::sleep(std::time::Duration::from_millis(2));
 }
}
fn command(app:&mut MatrixApp,text:&str){app.controller.submit(text).unwrap();}
fn triangles(app:&MatrixApp)->usize {
    app.controller.document().entities.keys().map(|id|app.controller.scene().unwrap()
        .root_brep_pick_display(&app.controller.document().stamp(*id).unwrap()).unwrap().triangles.len()).sum()
}
#[test]
fn viewport_detail_sphere_boolean_profiles_preserve_source_and_geometry() {
    let mut counts=Vec::new();let mut sources=Vec::new();
    for (detail,menu) in [(ViewportDetail::High,false),(ViewportDetail::Light,true)] {
        let mut app=MatrixApp::new(Document::with_template_tolerance("viewport profile")).unwrap();
        app.controller.request_viewport_detail(detail).unwrap();

        for center in ["0,0,0","5,0,0"] {
            if menu {app.controller.dispatch("OM9-SOLID-014",om9_app::session::InvocationOrigin::Menu).unwrap();}else{command(&mut app,"Sphere");}
            command(&mut app,center);command(&mut app,"5");
        }
        assert_eq!(app.controller.document().entities.len(),2);
        let ids=app.controller.document().entities.keys().copied().collect::<Vec<_>>();
        let body=app.controller.document().entities[&ids[0]].geometry.clone();
        let Geometry::Brep(body)=body else {panic!("sphere")};sources.push(body.source().to_owned());
        let input_triangles=triangles(&app);
        let before=app.controller.document().clone();
        app.controller.select_objects(&ids).unwrap();
        if menu{app.controller.dispatch("OM9-SOLID-004",om9_app::session::InvocationOrigin::Menu).unwrap();}else{command(&mut app,"Boolean2Objects");}
        command(&mut app,"DeleteInput=Yes");command(&mut app,"Union");drain(&mut app);command(&mut app,"");
        assert_eq!(app.controller.document().entities.len(),1);
        let result=app.controller.document().clone();
        let body=&app.controller.document().entities.values().next().unwrap().geometry;
        let Geometry::Brep(body)=body else {panic!("union")};
        let measured=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();
        let expected=1125.*std::f64::consts::PI/4.;
        assert!((measured.volume_mm3-expected).abs()/expected<1e-6,"{measured:?}");
        assert!(measured.closed);assert_eq!(measured.solids,1);assert_eq!(measured.naked_edges,0);
        for (a,b) in measured.bounds_mm.into_iter().zip([-5.,-5.,-5.,10.,5.,5.]){assert!((a-b).abs()<1e-5);}
        let output_triangles=triangles(&app);counts.push((input_triangles,output_triangles));
        app.controller.undo().unwrap();assert_eq!(app.controller.document(),&before);
        app.controller.redo().unwrap();assert_eq!(app.controller.document(),&result);
        let publication=app.controller.scene().unwrap().publication_id();
        let other=if detail==ViewportDetail::High {ViewportDetail::Light}else{ViewportDetail::High};
        app.controller.request_viewport_detail(other).unwrap();
        assert_eq!(app.controller.scene().unwrap().publication_id(),publication);
        drain(&mut app);
        assert_eq!(app.controller.viewport_detail(),other);assert_eq!(app.controller.document(),&result);
        assert_ne!(triangles(&app),output_triangles);
    }
    assert_eq!(sources[0],sources[1]);
    assert!(counts[1].0<counts[0].0,"input {counts:?}");assert!(counts[1].1<counts[0].1,"output {counts:?}");
    eprintln!("viewport profiles high/light triangles: {counts:?}");
}
#[test]
fn viewport_detail_batch_cancel_stale_and_old_preferences() {
    let mut app=MatrixApp::new(Document::with_template_tolerance("detail cancellation")).unwrap();
    for x in [0.,5.] { app.controller.solid_sphere_points([x,0.,0.],5.).unwrap(); }
    let before=app.controller.document().clone();let publication=app.controller.scene().unwrap().publication_id();
    app.controller.request_viewport_detail(ViewportDetail::High).unwrap();
    assert!(app.controller.poll_viewport_detail().is_none());
    app.controller.cancel_viewport_detail();assert_eq!(app.controller.scene().unwrap().publication_id(),publication);
    assert_eq!(app.controller.document(),&before);
    let until=std::time::Instant::now()+std::time::Duration::from_secs(5);
    loop {
        assert!(std::time::Instant::now()<until,"cancelled native worker did not release its proof");
        app.controller.request_viewport_detail(ViewportDetail::High).unwrap();
        assert!(app.controller.submit("Sphere").is_err());
        match app.controller.poll_viewport_detail() {
            Some(Err(om9_core::Error::Busy)) => std::thread::sleep(std::time::Duration::from_millis(2)),
            Some(result) => {result.unwrap();break;},
            None => {drain(&mut app);break;},
        }
    }
    assert_eq!(app.controller.document(),&before);assert_eq!(app.controller.viewport_detail(),ViewportDetail::High);
    let old=serde_json::to_value(DisplayPreferences::default()).unwrap();let mut old=old.as_object().unwrap().clone();old.remove("viewport_detail");
    let (loaded,warning)=DisplayPreferences::from_json(&serde_json::Value::Object(old).to_string());
    assert!(warning.is_none());assert_eq!(loaded.viewport_detail,ViewportDetail::Light);
}

#[test]
fn viewport_detail_stale_scene_cannot_publish() {
    let mut app=MatrixApp::new(Document::with_template_tolerance("stale detail")).unwrap();
    app.controller.solid_sphere_points([0.,0.,0.],5.).unwrap();
    app.controller.request_viewport_detail(ViewportDetail::High).unwrap();
    assert!(app.controller.poll_viewport_detail().is_none());
    // Exercise a model mutation from another owner while the worker is alive.
    app.controller.solid_sphere_points([5.,0.,0.],5.).unwrap();
    let current=app.controller.document().clone();let publication=app.controller.scene().unwrap().publication_id();
    assert!(app.controller.poll_viewport_detail().unwrap().is_err());
    assert_eq!(app.controller.document(),&current);assert_eq!(app.controller.scene().unwrap().publication_id(),publication);
    assert_eq!(app.controller.viewport_detail(),ViewportDetail::Light);
}

#[test]
fn viewport_detail_refines_surface_snap_and_refuses_pressure() {
    use om9_scene::camera::{Camera,Viewport};
    let mut app=MatrixApp::new(Document::with_template_tolerance("profile snap")).unwrap();
    app.controller.solid_sphere_points([0.,0.,0.],5.).unwrap();
    let camera=Camera::orthographic([0.,0.,20.],[0.,0.,0.],[0.,1.,0.],12.).unwrap();
    let viewport=Viewport{origin:[0.,0.],size:[600.,600.]};
    let expected=[1.,1.,23f64.sqrt()];
    let cursor=camera.project(expected,viewport).unwrap().unwrap().screen;
    for detail in [ViewportDetail::Light,ViewportDetail::High] {
        app.controller.request_viewport_detail(detail).unwrap();drain(&mut app);
        let candidates=om9_scene::osnap::candidates(app.controller.scene().unwrap(),&camera,viewport,cursor,Default::default(),None).unwrap();
        let hit=candidates.iter().find(|c|c.kind==om9_scene::osnap::SnapKind::Surface).expect("source-linked candidate");
        let refined=app.controller.refine_surface_snap(hit,&camera,viewport).unwrap();
        let error=refined.world.into_iter().zip(expected).map(|(a,b)|(a-b).powi(2)).sum::<f64>().sqrt();
        assert!(error<1e-6,"source refinement: {error}");
    }
    let mut controller=om9_app::session::Controller::new(Document::with_template_tolerance("pressure"),
        om9_core::budget::Limits{max_bytes:8*1024*1024,..Default::default()}).unwrap();
    controller.solid_sphere_points([0.,0.,0.],5.).unwrap();
    let before=controller.document().clone();let publication=controller.scene().unwrap().publication_id();
    controller.request_viewport_detail(ViewportDetail::High).unwrap();
    assert!(controller.poll_viewport_detail().unwrap().is_err());
    assert_eq!(controller.document(),&before);assert_eq!(controller.scene().unwrap().publication_id(),publication);
    assert_eq!(controller.viewport_detail(),ViewportDetail::Light);
}
