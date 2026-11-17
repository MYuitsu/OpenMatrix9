// SPDX-License-Identifier: LGPL-2.1-or-later
//! Rubber-band selection in the viewport: dragging a rectangle over several objects selects exactly the
//! ones it touches, through the real `MatrixApp::show` input path.
use eframe::egui::{self, Event, Modifiers, Pos2, Rect, Vec2};
use om9_app::ui::MatrixApp;
use om9_core::{
    geometry::Geometry,
    model::{Document, Entity},
};
use om9_scene::camera::Viewport;
use om9_scene::workspace::ViewId;

fn screen_rect() -> Rect {
    Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 720.))
}

fn document_with_points() -> Document {
    let mut document = Document::new("band");
    for (index, x) in [-8.0, 8.0].into_iter().enumerate() {
        let entity = Entity::new(
            if index == 0 { "left" } else { "right" },
            Geometry::Point([x, 0.0, 0.0]),
            None,
        );
        let id = entity.id;
        document.entities.insert(id, entity);
    }
    document
}

fn frames(app: &mut MatrixApp, context: &egui::Context, frames: usize) {
    for _ in 0..frames {
        let mut output = context.run_ui(
            egui::RawInput {
                screen_rect: Some(screen_rect()),
                ..Default::default()
            },
            |ui| app.show(ui),
        );
        output.textures_delta.clear();
    }
}

fn events(app: &mut MatrixApp, context: &egui::Context, events: Vec<Event>) {
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(screen_rect()),
            events,
            ..Default::default()
        },
        |ui| app.show(ui),
    );
    output.textures_delta.clear();
}

fn viewport_of(app: &MatrixApp, view: ViewId) -> Viewport {
    let rect = app.view_rects[&view];
    Viewport {
        origin: [rect.min.x as f64, rect.min.y as f64],
        size: [rect.width() as f64, rect.height() as f64],
    }
}

fn screen_of(app: &MatrixApp, view: ViewId, point: [f64; 3]) -> Pos2 {
    let viewport = viewport_of(app, view);
    let camera = &app.controller.workspace.views[&view].camera;
    let projected = camera
        .project(point, viewport)
        .expect("projection")
        .expect("in front");
    Pos2::new(projected.screen[0] as f32, projected.screen[1] as f32)
}

/// Presses at `start`, moves to `end` in steps, releases: the pointer path of a rubber band.
fn drag(app: &mut MatrixApp, context: &egui::Context, start: Pos2, end: Pos2) {
    events(app, context, vec![Event::ModifiersChanged(Modifiers::NONE)]);
    events(app, context, vec![Event::PointerMoved(start)]);
    events(
        app,
        context,
        vec![Event::PointerButton {
            pos: start,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Modifiers::NONE,
        }],
    );
    for step in 1..=4 {
        let t = step as f32 / 4.0;
        events(
            app,
            context,
            vec![Event::PointerMoved(start + (end - start) * t)],
        );
    }
    events(
        app,
        context,
        vec![Event::PointerButton {
            pos: end,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        }],
    );
    frames(app, context, 2);
}

#[test]
fn a_band_over_two_points_selects_both() {
    let context = egui::Context::default();
    let mut app = MatrixApp::new(document_with_points()).expect("app");
    frames(&mut app, &context, 4);
    let ids: Vec<_> = app.controller.document().entities.keys().copied().collect();
    assert_eq!(ids.len(), 2);
    let left = screen_of(&app, ViewId::Perspective, [-8.0, 0.0, 0.0]);
    let right = screen_of(&app, ViewId::Perspective, [8.0, 0.0, 0.0]);
    eprintln!(
        "left {left:?} right {right:?} rect {:?}",
        app.view_rects[&ViewId::Perspective]
    );
    let start = Pos2::new(left.x.min(right.x) - 25.0, left.y.min(right.y) - 25.0);
    let end = Pos2::new(left.x.max(right.x) + 25.0, left.y.max(right.y) + 25.0);
    drag(&mut app, &context, start, end);
    let selected = app.controller.selection_state().objects;
    assert_eq!(
        selected.len(),
        2,
        "both points are inside the band: {selected:?}"
    );
    for id in &ids {
        assert!(selected.contains(id));
    }
}

#[test]
fn a_band_around_one_point_selects_only_that_one() {
    let context = egui::Context::default();
    let mut app = MatrixApp::new(document_with_points()).expect("app");
    frames(&mut app, &context, 4);
    let ids: Vec<_> = app.controller.document().entities.keys().copied().collect();
    let left = screen_of(&app, ViewId::Perspective, [-8.0, 0.0, 0.0]);
    let start = Pos2::new(left.x - 20.0, left.y - 20.0);
    let end = Pos2::new(left.x + 20.0, left.y + 20.0);
    drag(&mut app, &context, start, end);
    let selected = app.controller.selection_state().objects;
    assert_eq!(
        selected.len(),
        1,
        "only the left point is inside: {selected:?}"
    );
    assert!(!ids.is_empty());
}

#[test]
fn unsupported_trim_can_be_selected_with_an_enclosing_window() {
    use om9_core::{budget::Limits,rhino_semantics::{set_trimmed_surface,TrimmedSurface}};
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::plane([-2.,-2.,0.],[1.,0.,0.],[0.,1.,0.],4.,4.,&limits).unwrap();
    let trim=TrimmedSurface {underlying:surface.clone(),outer:vec![[0.2,0.2],[0.8,0.2],[0.8,0.8],[0.2,0.8]],
        inners:vec![vec![[0.3,0.3],[0.7,0.3],[0.7,0.7],[0.3,0.7]],vec![[0.4,0.4],[0.6,0.4],[0.6,0.6],[0.4,0.6]]],reversed:false};
    let mut entity=Entity::new("unverified trim",Geometry::Surface(surface),None);
    set_trimmed_surface(&mut entity,&trim,&limits).unwrap();let id=entity.id;
    let mut document=Document::new("trim fallback window");document.entities.insert(id,entity);
    let context=egui::Context::default();let mut app=MatrixApp::new(document).unwrap();frames(&mut app,&context,4);
    let before=app.controller.document().clone();
    let a=screen_of(&app,ViewId::LookingDown,[-2.,-2.,0.]);let b=screen_of(&app,ViewId::LookingDown,[2.,2.,0.]);
    let start=Pos2::new(a.x.min(b.x)-10.,a.y.min(b.y)-10.);
    let end=Pos2::new(a.x.max(b.x)+10.,a.y.max(b.y)+10.);
    drag(&mut app,&context,start,end);
    assert!(app.controller.selection_state().objects.contains(&id));
    assert_eq!(app.controller.document(),&before);
}

#[test]
fn reopened_trim_with_nearby_collinear_vertices_is_click_selectable() {
    use om9_core::{budget::Limits,rhino_semantics::{set_trimmed_surface,TrimmedSurface},transaction::Cancellation};
    let limits=Limits::default();let y=0.5_f64;let next=f64::from_bits(y.to_bits()+1);
    let surface=om9_core::surface_kernel::plane([-2.,-2.,0.],[1.,0.,0.],[0.,1.,0.],4.,4.,&limits).unwrap();
    let trim=TrimmedSurface {underlying:surface.clone(),outer:vec![[0.2,0.2],[0.8,0.2],[0.8,0.8],[0.2,0.8],[0.2,next],[0.2,y]],inners:vec![],reversed:false};
    let mut entity=Entity::new("valid trim",Geometry::Surface(surface),None);
    set_trimmed_surface(&mut entity,&trim,&limits).unwrap();let id=entity.id;
    let mut document=Document::new("reopened trim");document.entities.insert(id,entity);
    let path=std::env::temp_dir().join(format!("om9-collinear-trim-{}.om9",uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path,&document,&limits,&Cancellation::default()).unwrap();
    let reopened=om9_app::session::Controller::from_file(&path,limits).unwrap();std::fs::remove_file(path).unwrap();
    let context=egui::Context::default();let mut app=MatrixApp::new(reopened.document().clone()).unwrap();
    app.controller.submit("Point").unwrap();app.controller.submit("100,100,100").unwrap();app.controller.undo().unwrap();
    let before=app.controller.document().clone();frames(&mut app,&context,4);
    let pos=screen_of(&app,ViewId::LookingDown,[0.,0.,0.]);
    events(&mut app,&context,vec![Event::PointerMoved(pos)]);
    for pressed in [true,false] {events(&mut app,&context,vec![Event::PointerButton {pos,button:egui::PointerButton::Primary,pressed,modifiers:Modifiers::NONE}]);}
    frames(&mut app,&context,2);
    assert!(app.controller.selection_state().objects.contains(&id));
    assert_eq!(app.controller.document(),&before);assert_eq!(app.controller.coordinator().redo_len(),1);
}

#[test]
fn reopened_trim_with_nearby_nonlinear_vertices_is_click_selectable() {
    use om9_core::{budget::Limits,rhino_semantics::{set_trimmed_surface,TrimmedSurface},transaction::Cancellation};
    let limits=Limits::default();let y=0.5_f64;let next=f64::from_bits(y.to_bits()+1);
    let surface=om9_core::surface_kernel::plane([-2.,-2.,0.],[1.,0.,0.],[0.,1.,0.],4.,4.,&limits).unwrap();
    let trim=TrimmedSurface {underlying:surface.clone(),outer:vec![[0.2,0.2],[0.8,0.2],[0.8,y],[0.7,next],[0.8,0.8],[0.2,0.8]],inners:vec![],reversed:false};
    let mut entity=Entity::new("notched trim",Geometry::Surface(surface),None);
    set_trimmed_surface(&mut entity,&trim,&limits).unwrap();let id=entity.id;
    let mut document=Document::new("reopened notched trim");document.entities.insert(id,entity);
    let path=std::env::temp_dir().join(format!("om9-notched-trim-{}.om9",uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path,&document,&limits,&Cancellation::default()).unwrap();
    let reopened=om9_app::session::Controller::from_file(&path,limits).unwrap();std::fs::remove_file(path).unwrap();
    let context=egui::Context::default();let mut app=MatrixApp::new(reopened.document().clone()).unwrap();
    app.controller.submit("Point").unwrap();app.controller.submit("100,100,100").unwrap();app.controller.undo().unwrap();
    let before=app.controller.document().clone();frames(&mut app,&context,4);
    let pos=screen_of(&app,ViewId::LookingDown,[0.,-0.4,0.]);
    events(&mut app,&context,vec![Event::PointerMoved(pos)]);
    for pressed in [true,false] {events(&mut app,&context,vec![Event::PointerButton {pos,button:egui::PointerButton::Primary,pressed,modifiers:Modifiers::NONE}]);}
    frames(&mut app,&context,2);
    assert!(app.controller.selection_state().objects.contains(&id));
    assert_eq!(app.controller.document(),&before);assert_eq!(app.controller.coordinator().redo_len(),1);
}

#[test]
fn reopened_501_edge_trim_is_click_selectable() {
    use om9_core::{budget::Limits,rhino_semantics::{set_trimmed_surface,TrimmedSurface},transaction::Cancellation};
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::plane([-2.,-2.,0.],[1.,0.,0.],[0.,1.,0.],4.,4.,&limits).unwrap();
    let outer=(0..501).map(|i|{let a=std::f64::consts::TAU*(i as f64)/501.;[0.5+0.3*a.cos(),0.5+0.3*a.sin()]}).collect();
    let trim=TrimmedSurface {underlying:surface.clone(),outer,inners:vec![],reversed:false};
    let mut entity=Entity::new("501 edges",Geometry::Surface(surface),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();let id=entity.id;
    let mut doc=Document::new("reopened 501 edge trim");doc.entities.insert(id,entity);
    let path=std::env::temp_dir().join(format!("om9-501-trim-{}.om9",uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path,&doc,&limits,&Cancellation::default()).unwrap();
    let reopened=om9_app::session::Controller::from_file(&path,limits).unwrap();std::fs::remove_file(path).unwrap();
    let context=egui::Context::default();let mut app=MatrixApp::new(reopened.document().clone()).unwrap();
    app.controller.submit("Point").unwrap();app.controller.submit("100,100,100").unwrap();app.controller.undo().unwrap();
    let before=app.controller.document().clone();frames(&mut app,&context,4);let pos=screen_of(&app,ViewId::LookingDown,[0.,0.,0.]);
    events(&mut app,&context,vec![Event::PointerMoved(pos)]);
    for pressed in [true,false] {events(&mut app,&context,vec![Event::PointerButton {pos,button:egui::PointerButton::Primary,pressed,modifiers:Modifiers::NONE}]);}
    frames(&mut app,&context,2);
    assert!(app.controller.selection_state().objects.contains(&id));assert_eq!(app.controller.document(),&before);
    assert_eq!(app.controller.coordinator().redo_len(),1);
}

#[test]
#[cfg(feature="occt-brep")]
fn reopened_712_edge_trim_displays_and_selects_material_not_hole() {
    use om9_core::{budget::Limits,rhino_semantics::{set_trimmed_surface,TrimmedSurface},transaction::Cancellation};
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::plane([-2.,-2.,0.],[1.,0.,0.],[0.,1.,0.],4.,4.,&limits).unwrap();
    let outer=(0..708).map(|i|{let a=std::f64::consts::TAU*(i as f64)/708.;[0.5+0.45*a.cos(),0.5+0.45*a.sin()]}).collect();
    let trim=TrimmedSurface {underlying:surface.clone(),outer,inners:vec![vec![[0.3,0.3],[0.7,0.3],[0.7,0.7],[0.3,0.7]]],reversed:false};
    let mut entity=Entity::new("712 edge trim",Geometry::Surface(surface),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();let id=entity.id;
    let mut doc=Document::with_template_tolerance("reopened trimmed display");doc.entities.insert(id,entity);
    let path=std::env::temp_dir().join(format!("om9-712-trim-{}.om9",uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path,&doc,&limits,&Cancellation::default()).unwrap();
    let reopened=om9_app::session::Controller::from_file(&path,limits).unwrap();std::fs::remove_file(path).unwrap();
    assert!(!reopened.scene().unwrap().objects[0].buffers.is_trim_display_fallback());
    let context=egui::Context::default();let mut app=MatrixApp::new(reopened.document().clone()).unwrap();
    app.controller.submit("Point").unwrap();app.controller.submit("100,100,100").unwrap();app.controller.undo().unwrap();
    let before=app.controller.document().clone();frames(&mut app,&context,4);
    app.controller.workspace.views.get_mut(&ViewId::LookingDown).unwrap().camera=
        om9_scene::camera::Camera::orthographic([0.,0.,10.],[0.,0.,0.],[0.,1.,0.],5.).unwrap();
    for (world,selected) in [([1.3,0.,0.],true),([0.,0.,0.],false),([1.95,0.,0.],false)] {
        app.controller.clear_selection();
        let pos=screen_of(&app,ViewId::LookingDown,world);events(&mut app,&context,vec![Event::PointerMoved(pos)]);
        if !selected {
            let rect=app.view_rects[&ViewId::LookingDown];
            let viewport=om9_scene::camera::Viewport {origin:[rect.min.x as f64,rect.min.y as f64],size:[rect.width() as f64,rect.height() as f64]};
            let camera=&app.controller.workspace.views[&ViewId::LookingDown].camera;
            let hits=app.controller.scene().unwrap().pick_all([pos.x as f64,pos.y as f64],camera,viewport,6.,om9_scene::scene::PickIntent::Select).unwrap();
            assert!(hits.is_empty(),"raw scene pick at {world:?}: {hits:?}");
        }
        for pressed in [true,false] {events(&mut app,&context,vec![Event::PointerButton {pos,button:egui::PointerButton::Primary,pressed,modifiers:Modifiers::NONE}]);}
        frames(&mut app,&context,2);assert_eq!(app.controller.selection_state().objects.contains(&id),selected,"world={world:?}, status={}",app.status_line());
    }
    assert_eq!(app.controller.document(),&before);assert_eq!(app.controller.coordinator().redo_len(),1);
}

#[test]
#[cfg(feature="occt-brep")]
fn reopened_convex_1025_edge_fallback_is_click_selectable_from_source() {
    use om9_core::{budget::Limits,rhino_semantics::{set_trimmed_surface,TrimmedSurface},transaction::Cancellation};
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::plane([-2.,-2.,0.],[1.,0.,0.],[0.,1.,0.],4.,4.,&limits).unwrap();
    let outer=(0..1025).map(|i|{let a=std::f64::consts::TAU*(i as f64)/1025.;[0.5+0.3*a.cos(),0.5+0.3*a.sin()]}).collect();
    let trim=TrimmedSurface {underlying:surface.clone(),outer,inners:vec![],reversed:false};
    let mut entity=Entity::new("1025 edges",Geometry::Surface(surface),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();let id=entity.id;
    let mut doc=Document::with_template_tolerance("reopened fallback source");doc.entities.insert(id,entity);
    let path=std::env::temp_dir().join(format!("om9-1025-trim-{}.om9",uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path,&doc,&limits,&Cancellation::default()).unwrap();
    let reopened=om9_app::session::Controller::from_file(&path,limits).unwrap();std::fs::remove_file(path).unwrap();
    assert!(reopened.scene().unwrap().objects[0].buffers.is_trim_display_fallback());
    let context=egui::Context::default();let mut app=MatrixApp::new(reopened.document().clone()).unwrap();
    app.controller.submit("Point").unwrap();app.controller.submit("100,100,100").unwrap();app.controller.undo().unwrap();
    let before=app.controller.document().clone();frames(&mut app,&context,4);let pos=screen_of(&app,ViewId::LookingDown,[0.,0.,0.]);
    events(&mut app,&context,vec![Event::PointerMoved(pos)]);
    for pressed in [true,false] {events(&mut app,&context,vec![Event::PointerButton {pos,button:egui::PointerButton::Primary,pressed,modifiers:Modifiers::NONE}]);}
    frames(&mut app,&context,2);assert!(app.controller.selection_state().objects.contains(&id));
    let outside=screen_of(&app,ViewId::LookingDown,[1.8,0.,0.]);events(&mut app,&context,vec![Event::PointerMoved(outside)]);
    for pressed in [true,false] {events(&mut app,&context,vec![Event::PointerButton {pos:outside,button:egui::PointerButton::Primary,pressed,modifiers:Modifiers::NONE}]);}
    frames(&mut app,&context,2);assert!(app.controller.selection_state().objects.is_empty());
    assert_eq!(app.controller.document(),&before);assert_eq!(app.controller.coordinator().redo_len(),1);
}

#[test]
#[cfg(feature="occt-brep")]
fn reopened_large_fallback_with_hole_selects_material_not_hole() {
    use om9_core::{budget::Limits,rhino_semantics::{set_trimmed_surface,TrimmedSurface},transaction::Cancellation};
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::plane([-2.,-2.,0.],[1.,0.,0.],[0.,1.,0.],4.,4.,&limits).unwrap();
    let outer=(0..1025).map(|i|{let a=std::f64::consts::TAU*(i as f64)/1025.;[0.5+0.3*a.cos(),0.5+0.3*a.sin()]}).collect();
    let trim=TrimmedSurface {underlying:surface.clone(),outer,inners:vec![vec![[0.4,0.4],[0.6,0.4],[0.6,0.6],[0.4,0.6]]],reversed:false};
    let mut entity=Entity::new("1025 edges with hole",Geometry::Surface(surface),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();let id=entity.id;
    let mut doc=Document::with_template_tolerance("reopened fallback hole");doc.entities.insert(id,entity);
    let path=std::env::temp_dir().join(format!("om9-1025-hole-{}.om9",uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path,&doc,&limits,&Cancellation::default()).unwrap();
    let reopened=om9_app::session::Controller::from_file(&path,limits).unwrap();std::fs::remove_file(path).unwrap();
    assert!(reopened.scene().unwrap().objects[0].buffers.is_trim_display_fallback());
    let context=egui::Context::default();let mut app=MatrixApp::new(reopened.document().clone()).unwrap();
    app.controller.submit("Point").unwrap();app.controller.submit("100,100,100").unwrap();app.controller.undo().unwrap();
    let before=app.controller.document().clone();frames(&mut app,&context,4);
    let material=screen_of(&app,ViewId::LookingDown,[0.8,0.,0.]);events(&mut app,&context,vec![Event::PointerMoved(material)]);
    for pressed in [true,false] {events(&mut app,&context,vec![Event::PointerButton {pos:material,button:egui::PointerButton::Primary,pressed,modifiers:Modifiers::NONE}]);}
    frames(&mut app,&context,2);assert!(app.controller.selection_state().objects.contains(&id));
    let hole=screen_of(&app,ViewId::LookingDown,[0.,0.,0.]);events(&mut app,&context,vec![Event::PointerMoved(hole)]);
    for pressed in [true,false] {events(&mut app,&context,vec![Event::PointerButton {pos:hole,button:egui::PointerButton::Primary,pressed,modifiers:Modifiers::NONE}]);}
    frames(&mut app,&context,2);assert!(app.controller.selection_state().objects.is_empty());
    assert_eq!(app.controller.document(),&before);assert_eq!(app.controller.coordinator().redo_len(),1);
}

#[test]
#[cfg(feature="occt-brep")]
fn reopened_concave_fallback_selects_material_not_hole_or_outside() {
    use om9_core::{budget::Limits,rhino_semantics::{set_trimmed_surface,TrimmedSurface},transaction::Cancellation};
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::plane([-2.,-2.,0.],[1.,0.,0.],[0.,1.,0.],4.,4.,&limits).unwrap();
    let mut outer=(0..1025).map(|i|{let a=std::f64::consts::TAU*(i as f64)/1025.;[0.5+0.3*a.cos(),0.5+0.3*a.sin()]}).collect::<Vec<_>>();
    outer[0]=[0.62,0.5];
    let trim=TrimmedSurface {underlying:surface.clone(),outer,inners:vec![vec![[0.4,0.4],[0.6,0.4],[0.6,0.6],[0.5,0.53],[0.4,0.6]]],reversed:false};
    let mut entity=Entity::new("concave trim with hole",Geometry::Surface(surface),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();let id=entity.id;
    let mut doc=Document::with_template_tolerance("reopened concave fallback");doc.entities.insert(id,entity);
    let path=std::env::temp_dir().join(format!("om9-concave-trim-{}.om9",uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path,&doc,&limits,&Cancellation::default()).unwrap();
    let reopened=om9_app::session::Controller::from_file(&path,limits).unwrap();std::fs::remove_file(path).unwrap();
    assert!(reopened.scene().unwrap().objects[0].buffers.is_trim_display_fallback());
    let context=egui::Context::default();let mut app=MatrixApp::new(reopened.document().clone()).unwrap();
    app.controller.submit("Point").unwrap();app.controller.submit("100,100,100").unwrap();app.controller.undo().unwrap();
    let before=app.controller.document().clone();frames(&mut app,&context,4);
    for (world,selected) in [([0.,0.88,0.],true),([0.,-0.2,0.],false),([0.8,0.,0.],false)] {
        let pos=screen_of(&app,ViewId::LookingDown,world);events(&mut app,&context,vec![Event::PointerMoved(pos)]);
        for pressed in [true,false] {events(&mut app,&context,vec![Event::PointerButton {pos,button:egui::PointerButton::Primary,pressed,modifiers:Modifiers::NONE}]);}
        frames(&mut app,&context,2);assert_eq!(app.controller.selection_state().objects.contains(&id),selected);
    }
    assert_eq!(app.controller.document(),&before);assert_eq!(app.controller.coordinator().redo_len(),1);
}
