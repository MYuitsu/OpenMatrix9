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
    let index = (0..body.display.edges.len()).find(|&i| {
        let Ok((a,b)) = om9_kernel::editing::straight_edge_endpoints(body, i, app.controller.coordinator().limits()) else { return false; };
        [a,b].iter().all(|p| p[1].abs() < 1e-7 && (p[2] - 8.).abs() < 1e-7)
    }).unwrap();
    let key = body.display.edges[index];
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
            for [ia, ib, ic, _] in &brep.display.triangles {
                let a = brep.display.points[*ia as usize]; let b = brep.display.points[*ib as usize]; let c = brep.display.points[*ic as usize];
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
    let mut doc = Document::new("Pipe chain points");
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
    click(&mut app, &ctx, [10.0, 3.0, 0.0]);
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
        original.display.faces.iter().position(|face| *face == key).unwrap()
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
        face.display
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
    let edge = original.display.edges.iter().position(|e| *e == key).unwrap();
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
    let edge = original.display.edges.iter().position(|e| *e == key).unwrap();
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
    let edges: Vec<_> = (0..shape.display.edges.len()).filter_map(|index| {
        let (a, b) = om9_kernel::editing::oriented_edge_endpoints(&shape, index, &limits).ok()?;
        (a[1].abs() < 1e-6 && b[1].abs() < 1e-6 && (a[2]-10.0).abs() < 1e-6 &&
         (b[2]-10.0).abs() < 1e-6 && ((a[0]-b[0]).abs()-5.0).abs() < 1e-6).then_some(shape.display.edges[index])
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
                .display
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
