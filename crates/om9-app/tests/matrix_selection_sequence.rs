// SPDX-License-Identifier: LGPL-2.1-or-later
//! `OM9-L3-03-03-001`: Matrix plain/Shift/Ctrl click mapping and locked refusal.
use eframe::egui::{self, Event, Modifiers, PointerButton, Pos2, Rect, Vec2};
use om9_app::ui::MatrixApp;
use om9_core::{
    geometry::Geometry,
    model::{Document, Entity},
};
use om9_scene::camera::Viewport;
use om9_scene::workspace::ViewId;

fn screen_rect() -> Rect {
    Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 900.))
}

fn frame(app: &mut MatrixApp, context: &egui::Context, events: Vec<Event>) {
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

fn viewport(app: &MatrixApp) -> Viewport {
    let rect = app.view_rects[&ViewId::LookingDown];
    Viewport {
        origin: [rect.min.x as f64, rect.min.y as f64],
        size: [rect.width() as f64, rect.height() as f64],
    }
}

fn screen_of(app: &MatrixApp, point: [f64; 3]) -> Pos2 {
    let projected = app.controller.workspace.views[&ViewId::LookingDown]
        .camera
        .project(point, viewport(app))
        .unwrap()
        .unwrap();
    Pos2::new(projected.screen[0] as f32, projected.screen[1] as f32)
}

fn click(app: &mut MatrixApp, context: &egui::Context, at: Pos2, modifiers: Modifiers) {
    frame(app, context, vec![Event::PointerMoved(at)]);
    frame(
        app,
        context,
        vec![
            Event::ModifiersChanged(modifiers),
            Event::PointerMoved(at),
            Event::PointerButton {
                pos: at,
                button: PointerButton::Primary,
                pressed: true,
                modifiers,
            },
        ],
    );
    frame(
        app,
        context,
        vec![Event::PointerButton {
            pos: at,
            button: PointerButton::Primary,
            pressed: false,
            modifiers,
        }],
    );
}

#[test]
fn plain_click_replaces_shift_adds_and_ctrl_toggles() {
    let context = egui::Context::default();
    let mut document = Document::new("selection mapping");
    let a = Entity::new("a", Geometry::Point([0., 0., 0.]), None);
    let a_id = a.id;
    document.entities.insert(a_id, a);
    let b = Entity::new("b", Geometry::Point([10., 0., 0.]), None);
    let b_id = b.id;
    document.entities.insert(b_id, b);
    let mut locked = Entity::new("locked", Geometry::Point([20., 0., 0.]), None);
    locked.attributes.selectable = false;
    locked.attributes.locked = true;
    let locked_id = locked.id;
    document.entities.insert(locked_id, locked);
    let mut app = MatrixApp::new(document).unwrap();
    frame(&mut app, &context, vec![]);
    frame(&mut app, &context, vec![]);
    app.controller.workspace.active = ViewId::LookingDown;

    let a_pos = screen_of(&app, [0., 0., 0.]);
    let b_pos = screen_of(&app, [10., 0., 0.]);
    let locked_pos = screen_of(&app, [20., 0., 0.]);

    click(&mut app, &context, a_pos, Modifiers::NONE);
    assert_eq!(app.controller.selected(), &[a_id].into_iter().collect());
    click(&mut app, &context, b_pos, Modifiers::NONE);
    assert_eq!(app.controller.selected(), &[b_id].into_iter().collect());

    click(
        &mut app,
        &context,
        a_pos,
        Modifiers {
            shift: true,
            ..Modifiers::NONE
        },
    );
    assert_eq!(
        app.controller.selected(),
        &[a_id, b_id].into_iter().collect()
    );
    click(
        &mut app,
        &context,
        b_pos,
        Modifiers {
            ctrl: true,
            ..Modifiers::NONE
        },
    );
    assert_eq!(app.controller.selected(), &[a_id].into_iter().collect());
    click(
        &mut app,
        &context,
        b_pos,
        Modifiers {
            ctrl: true,
            ..Modifiers::NONE
        },
    );
    assert_eq!(
        app.controller.selected(),
        &[a_id, b_id].into_iter().collect()
    );

    click(&mut app, &context, locked_pos, Modifiers::NONE);
    assert!(
        !app.controller.selected().contains(&locked_id),
        "a locked object never enters the selection"
    );
}

#[test]
fn failed_object_pick_keeps_selected_grips() {
    use om9_app::session::GripPick;
    use om9_core::{budget::memory::MemoryClass, top11::Top11Action};
    let mut app = MatrixApp::new(Document::with_template_tolerance("grip pick pressure")).unwrap();
    app.controller.submit("Polyline").unwrap();
    for point in ["0,0,0", "10,0,0", "10,10,0", ""] {
        app.controller.submit(point).unwrap();
    }
    let id = *app.controller.document().entities.keys().next().unwrap();
    app.controller.submit("Point").unwrap();
    app.controller.submit("100,100,100").unwrap();
    app.controller.undo().unwrap();
    app.controller.select_objects(&[id]).unwrap();
    app.controller.top11_invoke(Top11Action::ControlPoints).unwrap();
    let grip = app.controller.top11_grip_handles().unwrap()[0];
    app.controller.select_grips(&[grip], GripPick::Replace);
    let context = egui::Context::default();
    frame(&mut app, &context, vec![]);
    frame(&mut app, &context, vec![]);
    assert!(app.controller.grips_selected(), "grip selection lost before click");
    app.controller.workspace.active = ViewId::LookingDown;
    let at = screen_of(&app, [5., 0., 0.]);
    assert!(app.view_rects[&ViewId::LookingDown].contains(at));
    let before = app.controller.document().clone();
    let pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
        MemoryClass::Kernel, None, app.controller.scene_cache_usage().unwrap(),
    ).unwrap();
    let camera = &app.controller.workspace.views[&ViewId::LookingDown].camera;
    assert!(app.controller.scene().unwrap().pick_all(
        [at.x as f64, at.y as f64], camera, viewport(&app), 4.,
        om9_scene::scene::PickIntent::Select,
    ).is_err());
    click(&mut app, &context, at, Modifiers::NONE);
    assert!(app.status_line().contains("budget exceeded"), "{}", app.status_line());
    assert!(app.controller.grips_selected(), "failed pick cleared selected grips");
    assert_eq!(app.controller.document(), &before);
    assert_eq!(app.controller.coordinator().redo_len(), 1);
    drop(pressure);
    click(&mut app, &context, at, Modifiers::NONE);
    assert!(!app.controller.grips_selected(), "successful object pick should clear grips");
}

#[test]
fn failed_pick_preserves_selection_and_redo_until_retry() {
    use om9_core::budget::memory::MemoryClass;
    let mut doc=Document::with_template_tolerance("pick pressure");
    let first=Entity::new("first",Geometry::Point([0.,0.,0.]),None);let first_id=first.id;
    let second=Entity::new("second",Geometry::Point([10.,0.,0.]),None);let second_id=second.id;
    doc.entities.insert(first_id,first);doc.entities.insert(second_id,second);
    let mut app=MatrixApp::new(doc).unwrap();let context=egui::Context::default();
    app.controller.submit("Point").unwrap();app.controller.submit("100,100,100").unwrap();app.controller.undo().unwrap();
    app.controller.select_objects(&[first_id]).unwrap();let before=app.controller.document().clone();
    frame(&mut app,&context,vec![]);frame(&mut app,&context,vec![]);app.controller.workspace.active=ViewId::LookingDown;
    let at=screen_of(&app,[10.,0.,0.]);
    let pressure=app.controller.coordinator().reserve_remaining_external_memory_with_held(MemoryClass::Kernel,None,app.controller.scene_cache_usage().unwrap()).unwrap();
    let camera=&app.controller.workspace.views[&ViewId::LookingDown].camera;
    assert!(app.controller.scene().unwrap().pick_all([at.x as f64,at.y as f64],camera,viewport(&app),4.,om9_scene::scene::PickIntent::Select).is_err(),"fixture must refuse the picking cache");
    click(&mut app,&context,at,Modifiers::NONE);
    assert_eq!(app.controller.selected(),&[first_id].into_iter().collect());
    assert!(app.status_line().contains("budget exceeded"),"{}",app.status_line());
    assert_eq!(app.controller.document(),&before);assert_eq!(app.controller.coordinator().redo_len(),1);
    drop(pressure);click(&mut app,&context,at,Modifiers::NONE);
    assert_eq!(app.controller.selected(),&[second_id].into_iter().collect());
}

#[test]
fn the_overlap_popup_names_the_type_and_layer_swatch() {
    let context = egui::Context::default();
    let mut document = Document::new("popup swatch");
    for (label, z, colour) in [
        ("near", 0., [10, 20, 30, 255]),
        ("far", -5., [40, 50, 60, 255]),
    ] {
        let mut entity = Entity::new(label, Geometry::Point([0., 0., z]), None);
        entity.attributes.color = Some(colour);
        let id = entity.id;
        document.entities.insert(id, entity);
    }
    let mut app = MatrixApp::new(document).unwrap();
    frame(&mut app, &context, vec![]);
    frame(&mut app, &context, vec![]);
    app.controller.workspace.active = ViewId::LookingDown;
    let at = screen_of(&app, [0., 0., 0.]);
    click(&mut app, &context, at, Modifiers::NONE);
    let labels = app.ambiguous_labels();
    assert_eq!(labels.len(), 2, "{labels:?}");
    assert!(
        labels.iter().all(|label| label.contains("[Point]")),
        "{labels:?}"
    );
    let swatches = app.ambiguous_swatches();
    assert_eq!(swatches.len(), 2);
    assert!(swatches.iter().all(Option::is_some), "{swatches:?}");
}

#[test]
#[cfg(feature="occt-brep")]
fn source_verified_fallback_appears_in_overlap_popup_without_reopening() {
    use om9_core::{budget::Limits,rhino_semantics::{set_trimmed_surface,TrimmedSurface}};
    for row in [0,1] {
        let limits=Limits::default();
        let surface=om9_core::surface_kernel::plane([-2.,-2.,0.],[1.,0.,0.],[0.,1.,0.],4.,4.,&limits).unwrap();
        let outer=(0..1025).map(|i|{let a=std::f64::consts::TAU*(i as f64)/1025.;[0.5+0.3*a.cos(),0.5+0.3*a.sin()]}).collect();
        let trim=TrimmedSurface {underlying:surface.clone(),outer,inners:vec![],reversed:false};
        let mut entity=Entity::new("trim",Geometry::Surface(surface),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();let trim_id=entity.id;
        let point=Entity::new("near",Geometry::Point([0.,0.,1.]),None);let point_id=point.id;
        let mut doc=Document::with_template_tolerance("source fallback overlap");doc.entities.insert(trim_id,entity);doc.entities.insert(point_id,point);
        let context=egui::Context::default();let mut app=MatrixApp::new(doc).unwrap();
        app.controller.submit("Point").unwrap();app.controller.submit("100,100,100").unwrap();app.controller.undo().unwrap();let before=app.controller.document().clone();
        frame(&mut app,&context,vec![]);frame(&mut app,&context,vec![]);app.controller.workspace.active=ViewId::LookingDown;
        let at=screen_of(&app,[0.,0.,0.]);click(&mut app,&context,at,Modifiers::NONE);
        assert_eq!(app.ambiguous_candidates(),&[point_id,trim_id]);
        for _ in 0..3 {frame(&mut app,&context,vec![]);}
        let chosen=app.ambiguous_candidates()[row];
        let popup=context.memory(|m|m.area_rect(egui::Id::new("om9-pick-popup"))).unwrap();
        click(&mut app,&context,popup.min+egui::vec2(55.,35.+19.*row as f32),Modifiers::NONE);
        for _ in 0..3 {frame(&mut app,&context,vec![]);}
        assert!(app.ambiguous_candidates().is_empty());assert!(app.controller.selected().contains(&chosen));
        assert_eq!(app.controller.document(),&before);assert_eq!(app.controller.coordinator().redo_len(),1);
    }
}

#[test]
#[cfg(feature="occt-brep")]
fn reopened_concave_fallback_overlap_selects_chosen_object_and_ignores_hole() {
    use om9_core::{budget::Limits,rhino_semantics::{set_trimmed_surface,TrimmedSurface},transaction::Cancellation};
    for row in 0..2 {
        let limits=Limits::default();
        let surface=om9_core::surface_kernel::plane([-2.,-2.,0.],[1.,0.,0.],[0.,1.,0.],4.,4.,&limits).unwrap();
        let mut outer=(0..1025).map(|i|{let a=std::f64::consts::TAU*(i as f64)/1025.;[0.5+0.3*a.cos(),0.5+0.3*a.sin()]}).collect::<Vec<_>>();outer[0]=[0.62,0.5];
        let trim=TrimmedSurface {underlying:surface.clone(),outer,inners:vec![vec![[0.4,0.4],[0.6,0.4],[0.6,0.6],[0.5,0.53],[0.4,0.6]]],reversed:false};
        let mut entity=Entity::new("trim",Geometry::Surface(surface),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();let trim_id=entity.id;
        let point=Entity::new("near",Geometry::Point([0.,0.88,1.]),None);let point_id=point.id;
        let hole_point=Entity::new("hole",Geometry::Point([0.,-0.2,1.]),None);let hole_id=hole_point.id;
        let mut doc=Document::with_template_tolerance("reopened concave overlap");for e in [entity,point,hole_point] {doc.entities.insert(e.id,e);}
        let path=std::env::temp_dir().join(format!("om9-concave-overlap-{}.om9",uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&path,&doc,&limits,&Cancellation::default()).unwrap();
        let reopened=om9_app::session::Controller::from_file(&path,limits).unwrap();std::fs::remove_file(path).unwrap();
        let context=egui::Context::default();let mut app=MatrixApp::new(reopened.document().clone()).unwrap();
        app.controller.submit("Point").unwrap();app.controller.submit("100,100,100").unwrap();app.controller.undo().unwrap();let before=app.controller.document().clone();
        frame(&mut app,&context,vec![]);frame(&mut app,&context,vec![]);app.controller.workspace.active=ViewId::LookingDown;
        let at=screen_of(&app,[0.,0.88,0.]);click(&mut app,&context,at,Modifiers::NONE);
        assert_eq!(app.ambiguous_candidates(),&[point_id,trim_id]);
        for _ in 0..3 {frame(&mut app,&context,vec![]);}
        let chosen=app.ambiguous_candidates()[row];let popup=context.memory(|m|m.area_rect(egui::Id::new("om9-pick-popup"))).unwrap();
        click(&mut app,&context,popup.min+egui::vec2(55.,35.+19.*row as f32),Modifiers::NONE);
        for _ in 0..3 {frame(&mut app,&context,vec![]);}
        assert!(app.ambiguous_candidates().is_empty());assert!(app.controller.selected().contains(&chosen));
        let hole=screen_of(&app,[0.,-0.2,0.]);click(&mut app,&context,hole,Modifiers::NONE);
        assert!(app.ambiguous_candidates().is_empty());assert_eq!(app.controller.selected(),&[hole_id].into_iter().collect());
        assert_eq!(app.controller.document(),&before);assert_eq!(app.controller.coordinator().redo_len(),1);
    }
}

#[test]
#[cfg(feature="occt-brep")]
fn fallback_source_memory_refusal_keeps_selection_and_does_not_block_distant_pick() {
    use om9_core::{budget::{Limits,memory::MemoryClass},rhino_semantics::{set_trimmed_surface,TrimmedSurface}};
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::plane([-2.,-2.,0.],[1.,0.,0.],[0.,1.,0.],4.,4.,&limits).unwrap();
    let outer=(0..1025).map(|i|{let a=std::f64::consts::TAU*(i as f64)/1025.;[0.5+0.3*a.cos(),0.5+0.3*a.sin()]}).collect();
    let trim=TrimmedSurface {underlying:surface.clone(),outer,inners:vec![],reversed:false};
    let mut entity=Entity::new("trim",Geometry::Surface(surface),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();
    let near=Entity::new("behind",Geometry::Point([0.,0.,-1.]),None);let near_id=near.id;
    let far=Entity::new("far",Geometry::Point([10.,0.,0.]),None);let far_id=far.id;
    let mut doc=Document::with_template_tolerance("fallback memory");for e in [entity,near,far] {doc.entities.insert(e.id,e);}
    let context=egui::Context::default();let mut app=MatrixApp::new(doc).unwrap();
    app.controller.submit("Point").unwrap();app.controller.submit("100,100,100").unwrap();app.controller.undo().unwrap();
    app.controller.select_objects(&[far_id]).unwrap();let before=app.controller.document().clone();
    frame(&mut app,&context,vec![]);frame(&mut app,&context,vec![]);app.controller.workspace.active=ViewId::LookingDown;
    let centre=screen_of(&app,[0.,0.,0.]);let far_pos=screen_of(&app,[10.,0.,0.]);
    let camera=&app.controller.workspace.views[&ViewId::LookingDown].camera;
    app.controller.scene().unwrap().pick_all([centre.x as f64,centre.y as f64],camera,viewport(&app),4.,om9_scene::scene::PickIntent::Select).unwrap();
    let pressure=app.controller.coordinator().reserve_remaining_external_memory_with_held(MemoryClass::Kernel,None,app.controller.scene_cache_usage().unwrap()).unwrap();
    click(&mut app,&context,centre,Modifiers::NONE);
    assert_eq!(app.controller.selected(),&[far_id].into_iter().collect());assert!(!app.controller.selected().contains(&near_id));
    assert_eq!(app.controller.document(),&before);assert_eq!(app.controller.coordinator().redo_len(),1);
    click(&mut app,&context,far_pos,Modifiers::NONE);assert_eq!(app.controller.selected(),&[far_id].into_iter().collect());
    drop(pressure);
}

#[test]
#[cfg(feature="occt-brep")]
fn reopened_curved_fallback_rear_material_is_selected_by_viewport_click() {
    use om9_core::{budget::Limits,rhino_semantics::{set_trimmed_surface,TrimmedSurface},transaction::Cancellation};
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::cylinder([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,5.,&limits).unwrap();
    let outer=(0..1025).map(|i| {let a=std::f64::consts::TAU*(i as f64)/1025.;[0.5+0.2*a.cos(),0.5+0.2*a.sin()]}).collect();
    let trim=TrimmedSurface {underlying:surface.clone(),outer,inners:vec![],reversed:false};
    let mut entity=Entity::new("curved fallback",Geometry::Surface(surface),None);
    set_trimmed_surface(&mut entity,&trim,&limits).unwrap();let id=entity.id;
    let mut doc=Document::with_template_tolerance("curved fallback selection");doc.entities.insert(id,entity);
    let path=std::env::temp_dir().join(format!("om9-curved-click-{}.om9",uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path,&doc,&limits,&Cancellation::default()).unwrap();
    let reopened=om9_archive::Archive::open(&path,&limits).unwrap();std::fs::remove_file(&path).unwrap();
    let context=egui::Context::default();let mut app=MatrixApp::new(reopened).unwrap();
    app.controller.submit("Point").unwrap();app.controller.submit("100,100,100").unwrap();app.controller.undo().unwrap();
    let before=app.controller.document().clone();
    frame(&mut app,&context,vec![]);frame(&mut app,&context,vec![]);
    app.controller.workspace.active=ViewId::LookingDown;
    app.controller.workspace.views.get_mut(&ViewId::LookingDown).unwrap().camera=
        om9_scene::camera::Camera::orthographic([5.,1.,2.5],[0.,0.,2.5],[0.,0.,1.],10.).unwrap();
    let at=screen_of(&app,[0.,0.,2.5]);click(&mut app,&context,at,Modifiers::NONE);
    assert!(app.controller.selected().contains(&id));
    assert_eq!(app.controller.document(),&before);assert_eq!(app.controller.coordinator().redo_len(),1);

    // Control: the same click and camera hit the same untrimmed support.
    let mut control_doc=Document::with_template_tolerance("curved support control");
    let control=Entity::new("support",Geometry::Surface(trim.underlying.clone()),None);let control_id=control.id;
    control_doc.entities.insert(control_id,control);
    let context=egui::Context::default();let mut control_app=MatrixApp::new(control_doc).unwrap();
    frame(&mut control_app,&context,vec![]);frame(&mut control_app,&context,vec![]);
    control_app.controller.workspace.active=ViewId::LookingDown;
    control_app.controller.workspace.views.get_mut(&ViewId::LookingDown).unwrap().camera=
        om9_scene::camera::Camera::orthographic([5.,1.,2.5],[0.,0.,2.5],[0.,0.,1.],10.).unwrap();
    let at=screen_of(&control_app,[0.,0.,2.5]);click(&mut control_app,&context,at,Modifiers::NONE);
    assert!(control_app.controller.selected().contains(&control_id));
}

#[test]
#[cfg(feature="occt-brep")]
fn reopened_affine_curved_fallback_selects_rear_material_by_viewport_click() {
    use om9_core::{budget::Limits,geometry::Transform,rhino_semantics::{set_trimmed_surface,TrimmedSurface},transaction::Cancellation};
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::cylinder([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,5.,&limits).unwrap();
    let outer=(0..1025).map(|i| {let a=std::f64::consts::TAU*(i as f64)/1025.;[0.5+0.2*a.cos(),0.5+0.2*a.sin()]}).collect();
    let trim=TrimmedSurface {underlying:surface.clone(),outer,inners:vec![],reversed:false};
    for placement in [Transform([[2.,0.4,0.,4.],[0.,1.,0.,-3.],[0.,0.,1.5,7.],[0.,0.,0.,1.]]),
        Transform([[-1.5,0.2,0.,4.],[0.,1.,0.,-3.],[0.,0.,1.,7.],[0.,0.,0.,1.]])] {
        let mut entity=Entity::new("affine curved fallback",Geometry::Surface(surface.clone()),None);
        set_trimmed_surface(&mut entity,&trim,&limits).unwrap();entity.placement=placement;let id=entity.id;
        let mut doc=Document::with_template_tolerance("affine curved selection");doc.entities.insert(id,entity);
        let path=std::env::temp_dir().join(format!("om9-affine-curved-click-{}.om9",uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&path,&doc,&limits,&Cancellation::default()).unwrap();
        let reopened=om9_archive::Archive::open(&path,&limits).unwrap();std::fs::remove_file(&path).unwrap();
        let context=egui::Context::default();let mut app=MatrixApp::new(reopened).unwrap();
        app.controller.submit("Point").unwrap();app.controller.submit("100,100,100").unwrap();app.controller.undo().unwrap();
        let before=app.controller.document().clone();frame(&mut app,&context,vec![]);frame(&mut app,&context,vec![]);
        app.controller.workspace.active=ViewId::LookingDown;
        let target=placement.point([0.,0.,2.5]).unwrap();let eye=placement.point([5.,1.,2.5]).unwrap();
        app.controller.workspace.views.get_mut(&ViewId::LookingDown).unwrap().camera=
            om9_scene::camera::Camera::orthographic(eye,target,[0.,0.,1.],10.).unwrap();
        let at=screen_of(&app,target);click(&mut app,&context,at,Modifiers::NONE);
        assert!(app.controller.selected().contains(&id));
        assert_eq!(app.controller.document(),&before);assert_eq!(app.controller.coordinator().redo_len(),1);
    }
}

#[test]
#[cfg(feature="occt-brep")]
fn reopened_curved_fallback_overlap_chooser_selects_each_object() {
    use om9_core::{budget::Limits,geometry::Transform,rhino_semantics::{set_trimmed_surface,TrimmedSurface},transaction::Cancellation};
    let limits=Limits::default();
    let surface=om9_core::surface_kernel::cylinder([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,5.,&limits).unwrap();
    let outer=(0..1025).map(|i| {let a=std::f64::consts::TAU*(i as f64)/1025.;[0.5+0.2*a.cos(),0.5+0.2*a.sin()]}).collect();
    let trim=TrimmedSurface {underlying:surface.clone(),outer,inners:vec![],reversed:false};
    let placement=Transform([[2.,0.4,0.,4.],[0.,1.,0.,-3.],[0.,0.,1.5,7.],[0.,0.,0.,1.]]);
    for row in 0..2 {
        let mut entity=Entity::new("trim",Geometry::Surface(surface.clone()),None);
        set_trimmed_surface(&mut entity,&trim,&limits).unwrap();entity.placement=placement;let trim_id=entity.id;
        let target=placement.point([0.,0.,2.5]).unwrap();let point=Entity::new("point",Geometry::Point(target),None);let point_id=point.id;
        let mut doc=Document::with_template_tolerance("curved overlap");doc.entities.insert(trim_id,entity);doc.entities.insert(point_id,point);
        let path=std::env::temp_dir().join(format!("om9-curved-overlap-{}.om9",uuid::Uuid::new_v4()));
        om9_archive::Archive::save(&path,&doc,&limits,&Cancellation::default()).unwrap();
        let reopened=om9_archive::Archive::open(&path,&limits).unwrap();std::fs::remove_file(&path).unwrap();
        let context=egui::Context::default();let mut app=MatrixApp::new(reopened).unwrap();
        app.controller.submit("Point").unwrap();app.controller.submit("100,100,100").unwrap();app.controller.undo().unwrap();let before=app.controller.document().clone();
        frame(&mut app,&context,vec![]);frame(&mut app,&context,vec![]);app.controller.workspace.active=ViewId::LookingDown;
        app.controller.workspace.views.get_mut(&ViewId::LookingDown).unwrap().camera=
            om9_scene::camera::Camera::orthographic(placement.point([5.,1.,2.5]).unwrap(),target,[0.,0.,1.],10.).unwrap();
        let at=screen_of(&app,target);click(&mut app,&context,at,Modifiers::NONE);
        assert_eq!(app.ambiguous_candidates(),&[point_id,trim_id]);
        for _ in 0..3 {frame(&mut app,&context,vec![]);}
        let chosen=app.ambiguous_candidates()[row];let popup=context.memory(|m|m.area_rect(egui::Id::new("om9-pick-popup"))).unwrap();
        click(&mut app,&context,popup.min+egui::vec2(55.,35.+19.*row as f32),Modifiers::NONE);
        for _ in 0..3 {frame(&mut app,&context,vec![]);}
        assert!(app.ambiguous_candidates().is_empty());assert!(app.controller.selected().contains(&chosen));
        assert_eq!(app.controller.document(),&before);assert_eq!(app.controller.coordinator().redo_len(),1);
    }
}
