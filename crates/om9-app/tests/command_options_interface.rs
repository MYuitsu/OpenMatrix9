// SPDX-License-Identifier: LGPL-2.1-or-later
use eframe::egui::{self, Event, Key, Modifiers, Pos2, Rect, Vec2};
use om9_app::{
    command_options::{CommandOptionEffect, CommandOptionInterface},
    session::Controller,
    ui::MatrixApp,
};
use om9_core::{
    budget::Limits,
    geometry::{Curve, Geometry},
    model::{Document, Entity},
};

#[derive(Clone, Copy, Debug)]
enum Route {
    Keyboard,
    Pointer,
    Typed,
}
fn frame(app: &mut MatrixApp, ctx: &egui::Context, events: Vec<Event>) {
    let mut raw = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 720.))),
        events,
        ..Default::default()
    };
    eframe::App::raw_input_hook(app, ctx, &mut raw);
    let mut output = ctx.run_ui(raw, |ui| app.show(ui));
    output.textures_delta.clear();
}
fn key(key: Key, pressed: bool) -> Event {
    Event::Key {
        key,
        physical_key: None,
        pressed,
        repeat: false,
        modifiers: Modifiers::NONE,
    }
}
fn enter(app: &mut MatrixApp, ctx: &egui::Context) {
    frame(app, ctx, vec![key(Key::Enter, true)]);
    frame(app, ctx, vec![key(Key::Enter, false)]);
}
fn setup(command: &str) -> (MatrixApp, egui::Context) {
    let mut app = MatrixApp::new(Document::new("shared options")).unwrap();
    app.controller.submit(command).unwrap();
    app.transcript.set_prompt(&app.controller.prompt());
    let ctx = egui::Context::default();
    app.focus_command_editor();
    for _ in 0..4 {
        frame(&mut app, &ctx, vec![]);
    }
    (app, ctx)
}
fn activate(app: &mut MatrixApp, ctx: &egui::Context, name: &str, route: Route) {
    let option = app
        .controller
        .command_options()
        .into_iter()
        .find(|option| option.id.name() == name)
        .expect(name);
    match route {
        Route::Keyboard => {
            let shortcut = option.shortcut.unwrap();
            let keycode = Key::from_name(&shortcut.to_string()).unwrap();
            frame(
                app,
                ctx,
                vec![
                    key(keycode, true),
                    Event::Text(shortcut.to_ascii_lowercase().to_string()),
                ],
            );
            frame(app, ctx, vec![key(keycode, false)]);
        }
        Route::Pointer => {
            let at = app.command_option_token_rect(name).expect("token").center();
            assert!(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 720.)).contains(at));
            for event in [
                Event::PointerMoved(at),
                Event::PointerButton {
                    pos: at,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: Modifiers::NONE,
                },
                Event::PointerButton {
                    pos: at,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: Modifiers::NONE,
                },
            ] {
                frame(app, ctx, vec![event]);
            }
            frame(app, ctx, vec![]);
        }
        Route::Typed => {
            frame(app, ctx, vec![Event::Paste(name.to_ascii_lowercase())]);
            enter(app, ctx);
        }
    }
}
fn label(c: &Controller, name: &str) -> String {
    c.command_options()
        .into_iter()
        .find(|o| o.id.name() == name)
        .expect(name)
        .label
}

#[test]
fn boss_cancel_option_uses_shared_routes() {
    for route in [Route::Pointer, Route::Typed] {
        let (mut app, ctx) = setup("Boss");
        activate(&mut app, &ctx, "Cancel", route);
        assert!(!app.controller.is_running());
        assert_eq!(app.controller.coordinator().undo_len(), 0);
    }
}

#[test]
fn boss_undo_option_uses_shared_routes() {
    for route in [Route::Keyboard, Route::Pointer, Route::Typed] {
        let (mut app, ctx) = setup("Boss");
        activate(&mut app, &ctx, "Undo", route);
        assert!(app.controller.is_running());
        assert_eq!(app.controller.coordinator().undo_len(), 0);
    }
}

#[test]
fn pipe_chain_edges_option_uses_shared_routes() {
    for route in [Route::Keyboard, Route::Pointer, Route::Typed] {
        let (mut app, ctx) = setup("Pipe");
        activate(&mut app, &ctx, "ChainEdges", route);
        assert!(app.controller.prompt().contains("connected curve segments"));
        assert_eq!(app.controller.coordinator().undo_len(), 0);
    }
}

#[test]
fn pipe_multiple_option_uses_shared_routes() {
    for route in [Route::Keyboard, Route::Pointer, Route::Typed] {
        let (mut app, ctx) = setup("Pipe");
        activate(&mut app, &ctx, "Multiple", route);
        assert!(app.controller.prompt().contains("Select curves"));
        assert_eq!(app.controller.coordinator().undo_len(), 0);
    }
}

#[test]
fn pipe_cap_choice_cycles_none_round_flat_on_shared_routes() {
    for route in [Route::Keyboard, Route::Pointer, Route::Typed] {
        let (mut app, ctx) = setup("Pipe");
        assert_eq!(label(&app.controller, "Cap"), "Cap=Flat");
        for expected in ["Cap=None", "Cap=Round", "Cap=Flat"] {
            activate(&mut app, &ctx, "Cap", route);
            assert_eq!(label(&app.controller, "Cap"), expected);
        }
        assert_eq!(app.controller.coordinator().undo_len(), 0);
    }
}

#[test]
fn pipe_thick_option_uses_shared_routes() {
    for route in [Route::Keyboard, Route::Pointer, Route::Typed] {
        let (mut app, ctx) = setup("Pipe");
        assert_eq!(label(&app.controller, "Thick"), "Thick=No");
        activate(&mut app, &ctx, "Thick", route);
        assert_eq!(label(&app.controller, "Thick"), "Thick=Yes");
        activate(&mut app, &ctx, "Thick", route);
        assert_eq!(label(&app.controller, "Thick"), "Thick=No");
        assert_eq!(app.controller.coordinator().undo_len(), 0);
    }
}

#[test]
fn pipe_global_option_uses_shared_routes_without_losing_entered_radius() {
    for route in [Route::Keyboard, Route::Pointer, Route::Typed] {
        let mut doc = Document::new("Pipe Global option");
        let rail = Entity::new("rail", Geometry::Curve(Curve::Line {
            start: [0.0, 0.0, 0.0], end: [0.0, 0.0, 10.0], domain: [0.0, 1.0],
        }), None);
        let id = rail.id;
        doc.entities.insert(id, rail);
        let mut app = MatrixApp::new(doc).unwrap();
        app.controller.select_objects(&[id]).unwrap();
        app.controller.submit("Pipe").unwrap();
        app.transcript.set_prompt(&app.controller.prompt());
        let ctx = egui::Context::default();
        app.focus_command_editor();
        for _ in 0..4 { frame(&mut app, &ctx, vec![]); }
        assert_eq!(label(&app.controller, "ShapeBlending=Global"), "ShapeBlending=Global (No)");
        activate(&mut app, &ctx, "ShapeBlending=Global", route);
        assert_eq!(label(&app.controller, "ShapeBlending=Global"), "ShapeBlending=Global (Yes)");
        activate(&mut app, &ctx, "ShapeBlending=Local", route);
        assert_eq!(label(&app.controller, "ShapeBlending=Global"), "ShapeBlending=Global (No)");
        activate(&mut app, &ctx, "ShapeBlending=Global", route);
        app.controller.submit("1").unwrap();
        assert!(app.controller.prompt().contains("End radius"));
        activate(&mut app, &ctx, "ShapeBlending=Global", route);
        assert!(app.controller.prompt().contains("End radius"));
        app.controller.submit("2").unwrap();
        app.controller.submit("").unwrap();
        let result = app.controller.document().entities.values().find(|e| e.id != id).unwrap();
        let Geometry::Brep(brep) = &result.geometry else { panic!() };
        let m = om9_kernel::creation::measure(brep, app.controller.coordinator().limits()).unwrap();
        assert!((m.volume_mm3 - 70.0 * std::f64::consts::PI / 3.0).abs() < 1e-6);
    }
}

#[test]
fn truncated_cone_tangent_and_point_use_shared_option_routes() {
    for route in [Route::Keyboard,Route::Pointer,Route::Typed] {
        let rail=Entity::new("rail",Geometry::Curve(Curve::Line {start:[-5.,0.,0.],end:[5.,0.,0.],domain:[0.,1.]}),None); let id=rail.id;
        let mut doc=Document::new("cone tangent options"); doc.entities.insert(id,rail); let mut app=MatrixApp::new(doc).unwrap(); app.controller.select_objects(&[id]).unwrap();
        app.controller.submit("TCone").unwrap(); app.transcript.set_prompt(&app.controller.prompt()); app.focus_command_editor();
        let ctx=egui::Context::default(); for _ in 0..4 {frame(&mut app,&ctx,vec![]);}
        activate(&mut app,&ctx,"Tangent",route); app.controller.submit("OnCurve=0.5").unwrap();
        app.transcript.set_prompt(&app.controller.prompt()); app.focus_command_editor(); for _ in 0..2 {frame(&mut app,&ctx,vec![]);}
        activate(&mut app,&ctx,"Point",route); assert_eq!(app.controller.prompt(),"Pick point on base circle");
    }
}

#[test]
fn truncated_cone_around_curve_uses_shared_option_routes() {
    for route in [Route::Keyboard,Route::Pointer,Route::Typed] {
        let (mut app,ctx)=setup("TCone"); activate(&mut app,&ctx,"AroundCurve",route);
        assert_eq!(app.controller.prompt(),"Select curve for AroundCurve"); assert_eq!(app.controller.coordinator().undo_len(),0);
    }
}

#[test]
fn truncated_cone_fit_points_uses_shared_option_routes() {
    for route in [Route::Keyboard,Route::Pointer,Route::Typed] {
        let (mut app,ctx)=setup("TCone"); activate(&mut app,&ctx,"FitPoints",route);
        assert_eq!(app.controller.prompt(),"Add fit points or Enter to fit"); assert_eq!(app.controller.coordinator().undo_len(),0);
    }
}

#[test]
fn truncated_cone_solid_uses_shared_option_routes() {
    for route in [Route::Keyboard,Route::Pointer,Route::Typed] {
        let (mut app,ctx)=setup("TCone"); assert_eq!(label(&app.controller,"Solid"),"Solid=Yes");
        activate(&mut app,&ctx,"Solid",route); assert_eq!(label(&app.controller,"Solid"),"Solid=No");
        app.controller.accept_world_point([3.,7.,0.]).unwrap(); app.controller.submit("3").unwrap();
        app.controller.submit("5").unwrap(); app.controller.submit("1").unwrap();
        let Geometry::Brep(body)=&app.controller.document().entities.values().next().unwrap().geometry else {panic!()};
        let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();
        assert_eq!((m.solids,m.faces,m.naked_edges),(0,1,2)); assert!(!m.closed);
    }
}

#[test]
fn ellipsoid_mark_foci_uses_shared_option_routes() {
    for route in [Route::Keyboard, Route::Pointer, Route::Typed] {
        let (mut app, ctx) = setup("Ellipsoid FromFoci");
        assert_eq!(label(&app.controller, "MarkFoci"), "MarkFoci=No");
        activate(&mut app, &ctx, "MarkFoci", route);
        assert_eq!(label(&app.controller, "MarkFoci"), "MarkFoci=Yes");
        assert_eq!(app.controller.coordinator().undo_len(), 0);
    }
}

#[test]
fn ellipsoid_corner_uses_shared_option_routes() {
    for route in [Route::Keyboard, Route::Pointer, Route::Typed] {
        let (mut app, ctx) = setup("Ellipsoid");
        activate(&mut app, &ctx, "Corner", route);
        assert_eq!(app.controller.prompt(), "First corner of ellipse box");
        assert_eq!(app.controller.coordinator().undo_len(), 0);
    }
}

#[test]
fn ellipsoid_around_curve_uses_shared_option_routes() {
    for route in [Route::Keyboard, Route::Pointer, Route::Typed] {
        let (mut app, ctx) = setup("Ellipsoid");
        activate(&mut app, &ctx, "AroundCurve", route);
        assert_eq!(app.controller.prompt(), "Select curve for AroundCurve");
        assert_eq!(app.controller.coordinator().undo_len(), 0);
    }
}

#[test]
fn torus_two_point_uses_shared_option_routes() {
    for route in [Route::Keyboard, Route::Pointer, Route::Typed] {
        let (mut app, ctx) = setup("Torus");
        activate(&mut app, &ctx, "2Point", route);
        assert_eq!(app.controller.prompt(), "First diameter endpoint");
        assert_eq!(app.controller.coordinator().undo_len(), 0);
    }
}

#[test]
fn torus_three_point_uses_shared_option_routes() {
    for route in [Route::Keyboard, Route::Pointer, Route::Typed] {
        let (mut app, ctx) = setup("Torus"); activate(&mut app, &ctx, "3Point", route);
        assert_eq!(app.controller.prompt(), "First point on ring");
        assert_eq!(app.controller.coordinator().undo_len(), 0);
    }
}

#[test]
fn torus_around_curve_uses_shared_option_routes() {
    for route in [Route::Keyboard,Route::Pointer,Route::Typed] {
        let (mut app,ctx)=setup("Torus");activate(&mut app,&ctx,"AroundCurve",route);
        assert_eq!(app.controller.prompt(),"Select curve for AroundCurve");assert_eq!(app.controller.coordinator().undo_len(),0);
    }
}

#[test]
fn torus_vertical_uses_shared_option_routes() {
    for route in [Route::Keyboard,Route::Pointer,Route::Typed] {
        let (mut app,ctx)=setup("Torus");activate(&mut app,&ctx,"Vertical",route);
        app.controller.accept_world_point([0.,0.,0.]).unwrap();app.controller.accept_world_point([5.,0.,0.]).unwrap();app.controller.submit("1").unwrap();
        let entity=app.controller.document().entities.values().next().unwrap();let a=entity.placement.point([0.,0.,0.]).unwrap();let b=entity.placement.point([0.,0.,1.]).unwrap();
        assert!((b[1]-a[1]+1.).abs()<1e-8);
    }
}

#[test]
fn cylinder_solid_uses_shared_option_routes() {
    for route in [Route::Keyboard,Route::Pointer,Route::Typed] {
        let (mut app,ctx)=setup("Cylinder");assert_eq!(label(&app.controller,"Solid"),"Solid=Yes");activate(&mut app,&ctx,"Solid",route);
        assert_eq!(label(&app.controller,"Solid"),"Solid=No");
        app.controller.accept_world_point([0.,0.,0.]).unwrap();app.controller.submit("2").unwrap();app.controller.submit("3").unwrap();
        let Geometry::Brep(body)=&app.controller.document().entities.values().next().unwrap().geometry else {panic!()};
        let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();assert_eq!((m.solids,m.faces,m.naked_edges),(0,1,2));
    }
}

#[test]
fn cylinder_three_point_uses_shared_option_routes() {
    for route in [Route::Keyboard,Route::Pointer,Route::Typed] {
        let (mut app,ctx)=setup("Cylinder");activate(&mut app,&ctx,"3Point",route);
        assert_eq!(app.controller.prompt(),"First point on base circle");assert_eq!(app.controller.coordinator().undo_len(),0);
    }
}

#[test]
fn cylinder_around_curve_uses_shared_option_routes() {
    for route in [Route::Keyboard,Route::Pointer,Route::Typed] {
        let (mut app,ctx)=setup("Cylinder");activate(&mut app,&ctx,"AroundCurve",route);
        assert_eq!(app.controller.prompt(),"Select curve for AroundCurve");assert_eq!(app.controller.coordinator().undo_len(),0);
    }
}

#[test]
fn cylinder_fit_points_uses_shared_option_routes() {
    for route in [Route::Keyboard,Route::Pointer,Route::Typed] {
        let (mut app,ctx)=setup("Cylinder");activate(&mut app,&ctx,"FitPoints",route);
        assert_eq!(app.controller.prompt(),"Add fit points or Enter to fit");assert_eq!(app.controller.coordinator().undo_len(),0);
    }
}

#[test]
fn cylinder_tangent_uses_shared_option_routes() {
    for route in [Route::Keyboard,Route::Pointer,Route::Typed] {
        let (mut app,ctx)=setup("Cylinder");activate(&mut app,&ctx,"Tangent",route);
        assert_eq!(app.controller.prompt(),"Select tangent curve");assert_eq!(app.controller.coordinator().undo_len(),0);
    }
}

#[test]
fn cylinder_returns_to_center_from_other_base_modes() {
    for mode in ["2Point", "3Point", "FitPoints", "AroundCurve", "Tangent"] {
        for route in [Route::Keyboard, Route::Pointer, Route::Typed] {
            let (mut app, ctx) = setup("Cylinder");
            app.controller.submit(mode).unwrap();
            frame(&mut app, &ctx, vec![]);
            activate(&mut app, &ctx, "Center", route);
            assert_eq!(app.controller.prompt(), "Center of cylinder (or enter radius and height)");
            assert_eq!(app.controller.coordinator().undo_len(), 0);
            app.controller.accept_world_point([0., 0., 0.]).unwrap();
            app.controller.submit("2").unwrap();
            app.controller.submit("3").unwrap();
            let Geometry::Brep(body) = &app.controller.document().entities.values().next().unwrap().geometry else { panic!() };
            let measure = om9_kernel::creation::measure(body, app.controller.coordinator().limits()).unwrap();
            assert!(measure.closed && (measure.volume_mm3 - 12. * std::f64::consts::PI).abs() < 1e-6);
        }
    }
}

#[test]
fn truncated_cone_base_area_uses_shared_option_routes() {
    for route in [Route::Keyboard,Route::Pointer,Route::Typed] {
        let (mut app,ctx)=setup("TCone"); app.controller.accept_world_point([3.,7.,0.]).unwrap();
        app.transcript.set_prompt(&app.controller.prompt()); app.focus_command_editor(); for _ in 0..2 {frame(&mut app,&ctx,vec![]);}
        assert_eq!(label(&app.controller,"Area"),"Area=No"); activate(&mut app,&ctx,"Area",route);
        assert_eq!(label(&app.controller,"Area"),"Area=Yes","route {route:?}"); assert_eq!(app.controller.prompt(),"Base area");
        app.controller.submit(&(9.*std::f64::consts::PI).to_string()).unwrap();
        app.controller.submit("5").unwrap(); app.controller.submit("1").unwrap();
        let entity=app.controller.document().entities.values().next().unwrap(); let Geometry::Brep(body)=&entity.geometry else {panic!()};
        let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();
        assert!((m.volume_mm3-65.*std::f64::consts::PI/3.).abs()<1e-6);
    }
}

#[test]
fn cone_area_and_solid_use_shared_option_routes() {
    for route in [Route::Keyboard,Route::Pointer,Route::Typed] {
        let (mut app,ctx)=setup("Cone");activate(&mut app,&ctx,"Solid",route);assert_eq!(label(&app.controller,"Solid"),"Solid=No");
        app.controller.accept_world_point([0.,0.,0.]).unwrap();app.transcript.set_prompt(&app.controller.prompt());for _ in 0..2{frame(&mut app,&ctx,vec![]);}
        if matches!(route,Route::Typed){app.focus_command_editor();frame(&mut app,&ctx,vec![]);}activate(&mut app,&ctx,"Area",route);assert_eq!(app.controller.prompt(),"Base area");
        app.controller.submit(&(4.*std::f64::consts::PI).to_string()).unwrap();app.controller.submit("5").unwrap();
        let Geometry::Brep(body)=&app.controller.document().entities.values().next().unwrap().geometry else{panic!()};
        let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();assert!(!m.closed && m.faces==1 && m.naked_edges==1,"{m:?}");
    }
}

#[test]
fn cone_three_point_uses_shared_option_routes() {
    for route in [Route::Keyboard,Route::Pointer,Route::Typed] {
        let (mut app,ctx)=setup("Cone");activate(&mut app,&ctx,"3Point",route);assert_eq!(app.controller.prompt(),"First point on base circle");assert_eq!(app.controller.coordinator().undo_len(),0);
    }
}

#[test]
fn pyramid_around_curve_uses_shared_option_routes() {
    for route in [Route::Keyboard, Route::Pointer, Route::Typed] {
        let (mut app, ctx) = setup("Pyramid");
        activate(&mut app, &ctx, "AroundCurve", route);
        assert_eq!(app.controller.prompt(), "Select curve for AroundCurve");
        assert!(app.controller.solid_pipe_needs_curve());
        assert_eq!(app.controller.coordinator().undo_len(), 0);
    }
}

#[test]
fn pyramid_solid_uses_shared_option_routes() {
    for route in [Route::Keyboard, Route::Pointer, Route::Typed] {
        let (mut app, ctx) = setup("Pyramid");
        assert_eq!(label(&app.controller, "Solid"), "Solid=Yes");
        activate(&mut app, &ctx, "Solid", route);
        assert_eq!(label(&app.controller, "Solid"), "Solid=No");
        assert_eq!(app.controller.coordinator().undo_len(), 0);
        app.controller.accept_world_point([0., 0., 0.]).unwrap();
        app.controller.submit("3").unwrap();
        app.controller.submit("5").unwrap();
        let Geometry::Brep(body) = &app.controller.document().entities.values().next().unwrap().geometry else { panic!() };
        let m = om9_kernel::creation::measure(body, app.controller.coordinator().limits()).unwrap();
        assert!(!m.closed && m.naked_edges == 4, "{route:?}: {m:?}");
    }
}

#[test]
fn pyramid_star_automatic_uses_shared_option_routes() {
    for route in [Route::Keyboard,Route::Pointer,Route::Typed] {
        let (mut app,ctx)=setup("Pyramid Star");app.controller.accept_world_point([0.,0.,0.]).unwrap();app.controller.submit("3").unwrap();
        app.transcript.set_prompt(&app.controller.prompt());for _ in 0..2{frame(&mut app,&ctx,vec![]);}if matches!(route,Route::Typed){app.focus_command_editor();frame(&mut app,&ctx,vec![]);}
        activate(&mut app,&ctx,"Automatic",route);assert_eq!(app.controller.prompt(),"Apex height or point","{route:?}");app.controller.submit("6").unwrap();
        let Geometry::Brep(body)=&app.controller.document().entities.values().next().unwrap().geometry else{panic!()};
        let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();assert!((m.volume_mm3-4.*9.*(std::f64::consts::PI/4.).sin()*2.).abs()<1e-6);
    }
}

#[test]
fn pyramid_star_uses_shared_option_routes() {
    for route in [Route::Keyboard,Route::Pointer,Route::Typed] {
        let (mut app,ctx)=setup("Pyramid");activate(&mut app,&ctx,"Star",route);
        app.controller.accept_world_point([0.,0.,0.]).unwrap();app.controller.submit("5").unwrap();
        assert_eq!(app.controller.prompt(),"Second star radius or point (Automatic)","{route:?}");app.controller.submit("2").unwrap();app.controller.submit("6").unwrap();
        let Geometry::Brep(body)=&app.controller.document().entities.values().next().unwrap().geometry else{panic!()};
        let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();assert!((m.volume_mm3-4.*5.*2.*(std::f64::consts::PI/4.).sin()*2.).abs()<1e-6);
    }
}

#[test]
fn pyramid_edge_flip_uses_shared_option_routes() {
    for route in [Route::Keyboard,Route::Pointer,Route::Typed] {
        let (mut app,ctx)=setup("Pyramid Edge");app.controller.accept_world_point([0.,0.,0.]).unwrap();app.transcript.set_prompt(&app.controller.prompt());
        for _ in 0..2{frame(&mut app,&ctx,vec![]);}if matches!(route,Route::Typed){app.focus_command_editor();frame(&mut app,&ctx,vec![]);}
        activate(&mut app,&ctx,"Flip",route);app.controller.accept_world_point([4.,0.,0.]).unwrap();app.controller.submit("6").unwrap();
        let e=app.controller.document().entities.values().next().unwrap();let center=e.placement.point([0.,0.,0.]).unwrap();assert!((center[1]+2.).abs()<1e-8,"{route:?}");
    }
}

#[test]
fn pyramid_edge_uses_shared_option_routes() {
    for route in [Route::Keyboard,Route::Pointer,Route::Typed] {
        let (mut app,ctx)=setup("Pyramid");activate(&mut app,&ctx,"Edge",route);assert_eq!(app.controller.prompt(),"First point of base edge","{route:?}");
        app.controller.accept_world_point([0.,0.,0.]).unwrap();app.controller.accept_world_point([4.,0.,0.]).unwrap();app.controller.submit("6").unwrap();
        let Geometry::Brep(body)=&app.controller.document().entities.values().next().unwrap().geometry else{panic!()};
        let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();assert!((m.volume_mm3-32.).abs()<1e-6);
    }
}

#[test]
fn pyramid_circumscribed_uses_shared_option_routes() {
    for route in [Route::Keyboard,Route::Pointer,Route::Typed] {
        let (mut app,ctx)=setup("Pyramid");activate(&mut app,&ctx,"Circumscribed",route);
        app.controller.accept_world_point([0.,0.,0.]).unwrap();assert_eq!(app.controller.prompt(),"Base edge midpoint or radius","{route:?}");
        app.controller.submit("3").unwrap();app.controller.submit("5").unwrap();let Geometry::Brep(body)=&app.controller.document().entities.values().next().unwrap().geometry else{panic!()};
        let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();assert!((m.volume_mm3-60.).abs()<1e-6);
    }
}

#[test]
fn pyramid_num_sides_uses_shared_option_routes() {
    for route in [Route::Keyboard,Route::Pointer,Route::Typed] {
        let (mut app,ctx)=setup("Pyramid");
        match route { Route::Typed => { app.focus_command_editor();frame(&mut app,&ctx,vec![Event::Paste("NumSides=5".into())]);enter(&mut app,&ctx); },
            _ => { activate(&mut app,&ctx,"NumSides",route);frame(&mut app,&ctx,vec![Event::Text("5".into())]);enter(&mut app,&ctx); } }
        assert_eq!(app.controller.prompt(),"Center of pyramid base","{route:?}");app.controller.accept_world_point([0.,0.,0.]).unwrap();
        app.controller.submit("2").unwrap();app.controller.submit("6").unwrap();let Geometry::Brep(body)=&app.controller.document().entities.values().next().unwrap().geometry else{panic!()};
        let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();
        let expected=5.*2.*(std::f64::consts::TAU/5.).sin()*2.;assert!((m.volume_mm3-expected).abs()<1e-6);
    }
}

#[test]
fn tube_area_uses_shared_option_routes() {
    for route in [Route::Keyboard,Route::Pointer,Route::Typed] {
        let (mut app,ctx)=setup("Tube");app.controller.accept_world_point([0.,0.,0.]).unwrap();
        app.transcript.set_prompt(&app.controller.prompt());for _ in 0..2 {frame(&mut app,&ctx,vec![]);}
        if matches!(route,Route::Typed){app.focus_command_editor();frame(&mut app,&ctx,vec![]);}
        activate(&mut app,&ctx,"Area",route);assert_eq!(app.controller.prompt(),"Base area","{route:?}");
        app.controller.submit(&(9.*std::f64::consts::PI).to_string()).unwrap();app.controller.submit("1").unwrap();app.controller.submit("5").unwrap();
        let Geometry::Brep(body)=&app.controller.document().entities.values().next().unwrap().geometry else{panic!()};
        let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();assert!((m.volume_mm3-40.*std::f64::consts::PI).abs()<1e-6);
    }
}

#[test]
fn sphere_tangent_radius_centimeters_uses_shared_option_routes() {
    for route in [Route::Keyboard,Route::Pointer,Route::Typed] {
        let mut doc=Document::new("tangent cm options");doc.page_unit=om9_core::geometry::Unit::Centimeter;
        let curve=om9_core::curve_build::polyline(&[[-20.,0.,0.],[20.,0.,0.]]).unwrap();let rail=Entity::new("rail",Geometry::Curve(curve),None);let id=rail.id;doc.entities.insert(id,rail);
        let mut app=MatrixApp::new(doc).unwrap();let ctx=egui::Context::default();app.controller.select_objects(&[id]).unwrap();
        app.controller.submit("Sphere").unwrap();app.controller.submit("Tangent").unwrap();app.controller.accept_world_point([0.,0.,0.]).unwrap();
        app.transcript.set_prompt(&app.controller.prompt());app.focus_command_editor();for _ in 0..4 {frame(&mut app,&ctx,vec![]);}
        match route {
            Route::Typed => {frame(&mut app,&ctx,vec![Event::Paste("radius=0.5".into())]);enter(&mut app,&ctx);}
            _ => {activate(&mut app,&ctx,"Radius",route);frame(&mut app,&ctx,vec![Event::Text("0.5".into())]);enter(&mut app,&ctx);}
        }
        let e=app.controller.document().entities.values().find(|e|e.id!=id).expect("sphere");let Geometry::Brep(body)=&e.geometry else {panic!()};
        let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();
        assert!((m.volume_mm3-4.*std::f64::consts::PI*125./3.).abs()<1e-6,"{route:?}: {m:?}");
    }
}

#[test]
fn sphere_three_point_radius_routes_wait_for_direction() {
    for route in [Route::Keyboard,Route::Pointer,Route::Typed] {
        let (mut app,ctx)=setup("Sphere 3Point");app.controller.accept_world_point([0.,0.,0.]).unwrap();app.controller.accept_world_point([4.,0.,0.]).unwrap();
        app.transcript.set_prompt(&app.controller.prompt());for _ in 0..3 {frame(&mut app,&ctx,vec![]);}
        match route {Route::Typed=>{app.focus_command_editor();frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![Event::Paste("Radius=3".into())]);enter(&mut app,&ctx);}
        _=>{activate(&mut app,&ctx,"Radius",route);frame(&mut app,&ctx,vec![Event::Text("3".into())]);enter(&mut app,&ctx);}}
        assert_eq!(app.controller.prompt(),"Choose direction away from chord","{route:?}");assert!(app.controller.document().entities.is_empty());
        app.controller.accept_world_point([2.,5.,0.]).unwrap();let Geometry::Brep(body)=&app.controller.document().entities.values().next().unwrap().geometry else {panic!()};
        let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();assert!((m.volume_mm3-36.*std::f64::consts::PI).abs()<1e-6);
    }
}

#[test]
fn sphere_radius_non_mm_uses_shared_option_routes() {
    for route in [Route::Keyboard,Route::Pointer,Route::Typed] {
        let mut doc=Document::new("inch sphere options");doc.page_unit=om9_core::geometry::Unit::Inch;
        let mut app=MatrixApp::new(doc).unwrap();let ctx=egui::Context::default();app.controller.submit("Sphere").unwrap();
        app.controller.accept_world_point([0.,0.,0.]).unwrap();app.transcript.set_prompt(&app.controller.prompt());app.focus_command_editor();
        for _ in 0..4 {frame(&mut app,&ctx,vec![]);}
        match route {
            Route::Typed => {frame(&mut app,&ctx,vec![Event::Paste("Radius=2".into())]);enter(&mut app,&ctx);}
            _ => {activate(&mut app,&ctx,"Radius",route);frame(&mut app,&ctx,vec![Event::Text("2".into())]);enter(&mut app,&ctx);}
        }
        let Geometry::Brep(body)=&app.controller.document().entities.values().next().expect("sphere").geometry else {panic!()};
        let measured=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();
        let expected=4.*std::f64::consts::PI*50.8_f64.powi(3)/3.;assert!((measured.volume_mm3-expected).abs()/expected<1e-8,"{route:?}: {measured:?}");
    }
}

#[test]
fn box_width_non_mm_uses_shared_option_routes() {
    for route in [Route::Keyboard,Route::Pointer,Route::Typed] {
        let mut doc=Document::new("inch box options");doc.page_unit=om9_core::geometry::Unit::Inch;
        let mut app=MatrixApp::new(doc).unwrap();let ctx=egui::Context::default();
        app.controller.submit("Box").unwrap();app.controller.submit("3Point").unwrap();
        app.controller.accept_world_point([0.,0.,0.]).unwrap();app.controller.accept_world_point([25.4,0.,0.]).unwrap();
        app.transcript.set_prompt(&app.controller.prompt());app.focus_command_editor();for _ in 0..4 {frame(&mut app,&ctx,vec![]);}
        match route {
            Route::Typed => {frame(&mut app,&ctx,vec![Event::Paste("Width=2".into())]);enter(&mut app,&ctx);}
            _ => {activate(&mut app,&ctx,"Width",route);frame(&mut app,&ctx,vec![Event::Text("2".into())]);enter(&mut app,&ctx);}
        }
        assert!(app.controller.prompt().starts_with("Height"),"{route:?}: {}",app.controller.prompt());
        app.controller.submit("3").unwrap();let Geometry::Brep(body)=&app.controller.document().entities.values().next().unwrap().geometry else {panic!()};
        let measured=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();
        assert!((measured.volume_mm3-6.*25.4_f64.powi(3)).abs()<1e-5,"{route:?}: {measured:?}");
    }
}

#[test]
fn tube_wall_thickness_uses_shared_option_routes() {
    for route in [Route::Keyboard,Route::Pointer,Route::Typed] {
        let (mut app,ctx)=setup("Tube");app.controller.accept_world_point([0.,0.,0.]).unwrap();app.controller.submit("3").unwrap();
        app.transcript.set_prompt(&app.controller.prompt());for _ in 0..2{frame(&mut app,&ctx,vec![]);}
        match route {Route::Typed=>{app.focus_command_editor();frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![Event::Paste("WallThickness=2".into())]);enter(&mut app,&ctx);},
        _=>{activate(&mut app,&ctx,"WallThickness",route);frame(&mut app,&ctx,vec![Event::Text("2".into())]);enter(&mut app,&ctx);}}
        assert_eq!(app.controller.prompt(),"Height or point","{route:?}");app.controller.submit("5").unwrap();
        let Geometry::Brep(body)=&app.controller.document().entities.values().next().unwrap().geometry else{panic!()};
        let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();assert!((m.volume_mm3-40.*std::f64::consts::PI).abs()<1e-6);
    }
}

#[test]
fn tube_both_sides_uses_shared_option_routes() {
    for route in [Route::Keyboard,Route::Pointer,Route::Typed] {
        let (mut app,ctx)=setup("Tube");assert_eq!(label(&app.controller,"BothSides"),"BothSides=No");activate(&mut app,&ctx,"BothSides",route);
        assert_eq!(label(&app.controller,"BothSides"),"BothSides=Yes");assert_eq!(app.controller.coordinator().undo_len(),0);
        app.controller.accept_world_point([0.,0.,0.]).unwrap();app.controller.submit("3").unwrap();app.controller.submit("1").unwrap();app.controller.submit("5").unwrap();
        let Geometry::Brep(body)=&app.controller.document().entities.values().next().unwrap().geometry else{panic!()};
        let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();assert!((m.volume_mm3-80.*std::f64::consts::PI).abs()<1e-6);
    }
}

#[test]
fn tube_tangent_uses_shared_option_routes() {
    for route in [Route::Keyboard,Route::Pointer,Route::Typed] {
        let (mut app,ctx)=setup("Tube");activate(&mut app,&ctx,"Tangent",route);
        assert_eq!(app.controller.prompt(),"Select tangent curve");assert_eq!(app.controller.coordinator().undo_len(),0);
    }
}

#[test]
fn tube_around_curve_uses_shared_option_routes() {
    for route in [Route::Keyboard,Route::Pointer,Route::Typed] {
        let (mut app,ctx)=setup("Tube");activate(&mut app,&ctx,"AroundCurve",route);
        assert_eq!(app.controller.prompt(),"Select curve for AroundCurve");assert_eq!(app.controller.coordinator().undo_len(),0);
    }
}

#[test]
fn tube_fit_points_uses_shared_option_routes() {
    for route in [Route::Keyboard,Route::Pointer,Route::Typed] {
        let (mut app,ctx)=setup("Tube");activate(&mut app,&ctx,"FitPoints",route);
        assert_eq!(app.controller.prompt(),"Add fit points or Enter to fit");assert_eq!(app.controller.coordinator().undo_len(),0);
    }
}

#[test]
fn tube_three_point_radius_uses_shared_option_routes() {
    for route in [Route::Keyboard,Route::Pointer,Route::Typed] {
        let (mut app,ctx)=setup("Tube 3Point");
        app.controller.accept_world_point([0.,0.,0.]).unwrap();app.controller.accept_world_point([4.,0.,0.]).unwrap();
        app.transcript.set_prompt(&app.controller.prompt());for _ in 0..2 {frame(&mut app,&ctx,vec![]);}
        match route {
            Route::Typed => {app.focus_command_editor();frame(&mut app,&ctx,vec![]);frame(&mut app,&ctx,vec![Event::Paste("Radius=3".into())]);enter(&mut app,&ctx);},
            _ => {activate(&mut app,&ctx,"Radius",route);frame(&mut app,&ctx,vec![Event::Text("3".into())]);enter(&mut app,&ctx);},
        }
        assert_eq!(app.controller.prompt(),"Choose base circle side", "{route:?}");
    }
}

#[test]
fn truncated_cone_three_point_radius_uses_shared_option_routes() {
    for route in [Route::Keyboard,Route::Pointer,Route::Typed] {
        let (mut app,ctx)=setup("TCone"); app.controller.submit("3Point").unwrap();
        app.controller.accept_world_point([0.,0.,0.]).unwrap(); app.controller.accept_world_point([4.,0.,0.]).unwrap();
        app.transcript.set_prompt(&app.controller.prompt()); for _ in 0..2 {frame(&mut app,&ctx,vec![]);}
        match route {
            Route::Typed => {app.focus_command_editor(); frame(&mut app,&ctx,vec![]); frame(&mut app,&ctx,vec![Event::Paste("Radius=3".into())]); assert_eq!(app.transcript.draft(),"Radius=3"); enter(&mut app,&ctx);},
            _ => {activate(&mut app,&ctx,"Radius",route); frame(&mut app,&ctx,vec![Event::Text("3".into())]); enter(&mut app,&ctx);},
        }
        assert_eq!(app.controller.prompt(),"Choose base circle side", "route {route:?} draft {}",app.transcript.draft());
    }
}

#[test]
fn truncated_cone_three_point_uses_shared_option_routes() {
    for route in [Route::Keyboard, Route::Pointer, Route::Typed] {
        let (mut app,ctx)=setup("TCone");
        activate(&mut app,&ctx,"3Point",route);
        assert_eq!(app.controller.prompt(),"First point on base circle");
        assert_eq!(app.controller.coordinator().undo_len(),0);
    }
}

#[test]
fn truncated_cone_two_point_uses_shared_option_routes() {
    for route in [Route::Keyboard, Route::Pointer, Route::Typed] {
        let (mut app,ctx)=setup("TCone");
        activate(&mut app,&ctx,"2Point",route);
        assert_eq!(app.controller.prompt(),"First diameter endpoint");
        assert_eq!(app.controller.coordinator().undo_len(),0);
    }
}

#[test]
fn truncated_cone_diameter_uses_shared_keyboard_pointer_and_typed_routes() {
    for route in [Route::Keyboard, Route::Pointer, Route::Typed] {
        let mut app=MatrixApp::new(Document::new("Truncated Cone options")).unwrap();
        app.controller.dispatch("OM9-SOLID-020",om9_app::session::InvocationOrigin::Menu).unwrap();
        app.controller.accept_world_point([3.,7.,0.]).unwrap();
        app.transcript.set_prompt(&app.controller.prompt());
        let ctx=egui::Context::default(); app.focus_command_editor();
        for _ in 0..4 {frame(&mut app,&ctx,vec![]);}
        assert_eq!(label(&app.controller,"Diameter"),"Diameter=No");
        activate(&mut app,&ctx,"Diameter",route);
        assert_eq!(label(&app.controller,"Diameter"),"Diameter=Yes");
        app.controller.submit("6").unwrap(); app.controller.submit("5").unwrap(); app.controller.submit("2").unwrap();
        let entity=app.controller.document().entities.values().next().unwrap();
        let Geometry::Brep(body)=&entity.geometry else {panic!()};
        let m=om9_kernel::creation::measure(body,app.controller.coordinator().limits()).unwrap();
        assert!((m.volume_mm3-65.*std::f64::consts::PI/3.).abs()<1e-6);
    }
}

#[test]
fn pipe_diameter_and_cap_use_shared_keyboard_pointer_and_typed_routes() {
    for route in [Route::Keyboard, Route::Pointer, Route::Typed] {
        let mut doc = Document::new("Pipe options");
        let rail = Entity::new("rail", Geometry::Curve(Curve::Line {
            start: [0.0, 0.0, 0.0], end: [0.0, 0.0, 10.0], domain: [0.0, 1.0],
        }), None);
        let id = rail.id;
        doc.entities.insert(id, rail);
        let mut app = MatrixApp::new(doc).unwrap();
        app.controller.select_objects(&[id]).unwrap();
        app.controller.submit("Pipe").unwrap();
        app.transcript.set_prompt(&app.controller.prompt());
        let ctx = egui::Context::default();
        app.focus_command_editor();
        for _ in 0..4 { frame(&mut app, &ctx, vec![]); }
        assert_eq!(label(&app.controller, "Diameter"), "Diameter=No");
        assert_eq!(label(&app.controller, "Cap"), "Cap=Flat");
        activate(&mut app, &ctx, "Diameter", route);
        activate(&mut app, &ctx, "Cap", route);
        assert_eq!(label(&app.controller, "Diameter"), "Diameter=Yes");
        assert_eq!(label(&app.controller, "Cap"), "Cap=None");
        assert_eq!(app.controller.coordinator().undo_len(), 0);
        app.controller.submit("2").unwrap();
        app.controller.submit("2").unwrap();
        app.controller.submit("").unwrap();
        let result = app.controller.document().entities.values().find(|e| e.id != id).unwrap();
        let Geometry::Brep(brep) = &result.geometry else { panic!() };
        let m = om9_kernel::creation::measure(brep, app.controller.coordinator().limits()).unwrap();
        assert!(!m.closed);
        assert!((m.size_mm()[0] - 2.0).abs() < 1e-5);
        app.controller.undo().unwrap();
        assert_eq!(app.controller.document().entities.len(), 1);
    }
}

#[test]
fn boolean_difference_delete_input_uses_shared_option_routes() {
    for route in [Route::Keyboard, Route::Pointer, Route::Typed] {
        let (mut app, ctx) = setup("BooleanDifference");
        assert_eq!(label(&app.controller, "DeleteInput"), "DeleteInput=No");
        activate(&mut app, &ctx, "DeleteInput", route);
        assert_eq!(label(&app.controller, "DeleteInput"), "DeleteInput=Yes");
        activate(&mut app, &ctx, "DeleteInput", route);
        assert_eq!(label(&app.controller, "DeleteInput"), "DeleteInput=No");
        assert!(app.controller.document().entities.is_empty());
        assert_eq!(app.controller.coordinator().undo_len(), 0);
    }
}

#[test]
fn extract_surface_options_use_shared_routes() {
    for route in [Route::Keyboard, Route::Pointer, Route::Typed] {
        let (mut app, ctx) = setup("ExtractSrf");
        assert_eq!(label(&app.controller, "Copy"), "Copy=No");
        assert_eq!(label(&app.controller, "OutputLayer"), "OutputLayer=Current");
        activate(&mut app, &ctx, "Copy", route);
        activate(&mut app, &ctx, "OutputLayer", route);
        assert_eq!(label(&app.controller, "Copy"), "Copy=Yes");
        assert_eq!(label(&app.controller, "OutputLayer"), "OutputLayer=Input");
        assert_eq!(app.controller.coordinator().undo_len(), 0);
    }
}

#[test]
fn shell_thickness_uses_shared_numeric_option_routes() {
    for route in [Route::Keyboard, Route::Pointer, Route::Typed] {
        let (mut app, ctx) = setup("Shell");
        if matches!(route, Route::Typed) {
            frame(&mut app, &ctx, vec![Event::Paste("Thickness=1.25".into())]);
        } else {
            activate(&mut app, &ctx, "Thickness", route);
            assert_eq!(app.transcript.draft(), "Thickness=");
            frame(&mut app, &ctx, vec![Event::Text("1.25".into())]);
        }
        enter(&mut app, &ctx);
        assert_eq!(label(&app.controller, "Thickness"), "Thickness=1.25");
        assert_eq!(app.controller.coordinator().undo_len(), 0);
    }
}

#[test]
#[cfg(feature = "occt-brep")]
fn fillet_add_handle_option_uses_shared_routes() {
    for route in [Route::Keyboard, Route::Pointer, Route::Typed] {
        let limits = Limits::default();
        let shape = om9_kernel::creation::primitive(
            om9_kernel::creation::Primitive::Box { x_mm: 10.0, y_mm: 10.0, z_mm: 10.0 }, 0.01, &limits,
        ).unwrap();
        let edge = shape.display.edges[0];
        let mut doc = Document::new("AddHandle option");
        let entity = om9_core::model::Entity::new("box", Geometry::Brep(shape), None);
        let id = entity.id;
        doc.entities.insert(id, entity);
        let mut app = MatrixApp::new(doc).unwrap();
        app.controller.submit("FilletEdge").unwrap();
        app.controller.submit("NextRadius=1").unwrap();
        let object = app.controller.scene().unwrap().objects.iter().find(|o| o.root == id && o.path.is_empty()).unwrap();
        let reference = om9_app::session::SelectedSubobject { root: id, path: vec![],
            subset: om9_scene::scene::Subelement::BrepEdge(edge), source: object.source,
            world_epoch: app.controller.scene().unwrap().epoch };
        app.controller.solid_fillet_pick_edge(reference.clone()).unwrap();
        app.controller.submit("").unwrap();
        let ctx = egui::Context::default();
        app.transcript.set_prompt(&app.controller.prompt());
        app.focus_command_editor();
        for _ in 0..4 { frame(&mut app, &ctx, vec![]); }
        activate(&mut app, &ctx, "AddHandle", route);
        assert!(app.controller.solid_fillet_adding_handle(), "{route:?}");
        assert_eq!(label(&app.controller, "CurrentRadius"), "CurrentRadius=1");
        assert_eq!(app.controller.coordinator().undo_len(), 0);
        let handles = app.controller.solid_fillet_handles();
        let middle = std::array::from_fn(|i| (handles[0].1[i] + handles[1].1[i]) * 0.5);
        app.controller.solid_fillet_add_handle(reference, middle).unwrap();
        app.controller.submit("").unwrap();
        assert!(!app.controller.solid_fillet_adding_handle());
        assert!(app.controller.solid_fillet_preview().is_some());
        app.transcript.set_prompt(&app.controller.prompt());
        frame(&mut app, &ctx, vec![]);
        activate(&mut app, &ctx, "RemoveHandle", route);
        assert!(app.controller.prompt().contains("remove"), "{route:?}");
        let added = app.controller.solid_fillet_handles()[2].0;
        app.controller.solid_fillet_select_handle(added).unwrap();
        assert_eq!(app.controller.solid_fillet_handles().len(), 2);
        app.controller.submit("").unwrap();
        assert!(!app.controller.command_options().iter().any(|o| o.id.name() == "RemoveHandle"));
        app.transcript.set_prompt(&app.controller.prompt());
        frame(&mut app, &ctx, vec![]);
        activate(&mut app, &ctx, "CopyHandle", route);
        assert!(app.controller.prompt().contains("handle to copy"), "{route:?}");
        app.controller.solid_fillet_select_handle(0).unwrap();
        assert!(app.controller.solid_fillet_copying_to_target());
        app.controller.submit("").unwrap();
        assert!(!app.controller.solid_fillet_copying_to_target());
    }
}

#[test]
fn fillet_show_radius_uses_shared_option_routes() {
    for route in [Route::Keyboard, Route::Pointer, Route::Typed] {
        let (mut app, ctx) = setup("FilletEdge");
        assert_eq!(label(&app.controller, "ShowRadius"), "ShowRadius=Yes");
        activate(&mut app, &ctx, "ShowRadius", route);
        assert_eq!(label(&app.controller, "ShowRadius"), "ShowRadius=No");
        assert!(!app.controller.solid_fillet_show_radius());
        activate(&mut app, &ctx, "ShowRadius", route);
        assert!(app.controller.solid_fillet_show_radius());
        assert_eq!(app.controller.coordinator().undo_len(), 0);
    }
}

#[test]
#[cfg(feature = "occt-brep")]
fn fillet_set_all_keyboard_pointer_and_typed_assignment_update_preview() {
    for route in [Route::Keyboard, Route::Pointer, Route::Typed] {
        let limits = Limits::default();
        let shape = om9_kernel::creation::primitive(
            om9_kernel::creation::Primitive::Box { x_mm: 10.0, y_mm: 10.0, z_mm: 10.0 },
            0.01, &limits,
        ).unwrap();
        let edges = [shape.display.edges[0], shape.display.edges[2]];
        let mut doc = Document::new("SetAll routes");
        let entity = om9_core::model::Entity::new("box", Geometry::Brep(shape), None);
        let id = entity.id;
        doc.entities.insert(id, entity);
        let mut app = MatrixApp::new(doc).unwrap();
        let before = app.controller.document().clone();
        app.controller.submit("FilletEdge").unwrap();
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
        let ctx = egui::Context::default();
        app.transcript.set_prompt(&app.controller.prompt());
        app.focus_command_editor();
        for _ in 0..4 { frame(&mut app, &ctx, vec![]); }
        match route {
            Route::Typed => frame(&mut app, &ctx, vec![Event::Paste("SetAll=1.5".into())]),
            _ => {
                activate(&mut app, &ctx, "SetAll", route);
                assert_eq!(app.transcript.draft(), "SetAll=");
                frame(&mut app, &ctx, vec![Event::Text("1.5".into())]);
            }
        }
        enter(&mut app, &ctx);
        assert!(app.controller.solid_fillet_handles().iter().all(|item| item.2 == 1.5));
        let (preview, _) = app.controller.solid_fillet_preview().unwrap();
        let volume = om9_kernel::creation::measure(preview, app.controller.coordinator().limits()).unwrap().volume_mm3;
        let expected = 1000.0 - (1.0 - std::f64::consts::PI / 4.0) * 45.0;
        assert!((volume - expected).abs() < 1e-5, "{route:?}: {volume}");
        assert_eq!(app.controller.document(), &before);
        assert_eq!(app.controller.coordinator().undo_len(), 0);
        app.controller.solid_fillet_select_handle(0).unwrap();
        app.transcript.set_prompt(&app.controller.prompt());
        frame(&mut app, &ctx, vec![]);
        assert_eq!(label(&app.controller, "LinkHandles"), "LinkHandles=No", "initial {route:?}");
        activate(&mut app, &ctx, "LinkHandles", route);
        assert_eq!(label(&app.controller, "LinkHandles"), "LinkHandles=Yes");
        app.controller.submit("CurrentRadius=2").unwrap();
        assert!(app.controller.solid_fillet_handles().iter().all(|item| item.2 == 2.0));
        let (linked, _) = app.controller.solid_fillet_preview().unwrap();
        let linked_volume = om9_kernel::creation::measure(linked, app.controller.coordinator().limits()).unwrap().volume_mm3;
        let expected = 1000.0 - (1.0 - std::f64::consts::PI / 4.0) * 80.0;
        assert!((linked_volume - expected).abs() < 1e-5);
        assert!(app.controller.submit("CurrentRadius=1000").is_err());
        assert!(app.controller.solid_fillet_handles().iter().all(|item| item.2 == 2.0));
        let (retained, _) = app.controller.solid_fillet_preview().unwrap();
        assert!((om9_kernel::creation::measure(retained, app.controller.coordinator().limits()).unwrap().volume_mm3 - linked_volume).abs() < 1e-7);
        frame(&mut app, &ctx, vec![]);
        activate(&mut app, &ctx, "LinkHandles", route);
        assert_eq!(label(&app.controller, "LinkHandles"), "LinkHandles=No", "second {route:?}");
        app.controller.submit("CurrentRadius=1.5").unwrap();
        let radii: Vec<_> = app.controller.solid_fillet_handles().iter().map(|item| item.2).collect();
        assert_eq!(radii, vec![1.5, 2.0, 2.0, 2.0]);
        assert_eq!(app.controller.coordinator().undo_len(), 0);
        enter(&mut app, &ctx);
        assert_eq!(app.controller.coordinator().undo_len(), 1);
        app.controller.undo().unwrap();
        assert_eq!(app.controller.document(), &before);
    }
}

#[test]
#[cfg(feature = "occt-brep")]
fn fillet_select_edges_routes_preserve_edits_and_allow_more_edges() {
    for route in [Route::Keyboard, Route::Pointer, Route::Typed] {
        let limits = Limits::default();
        let shape = om9_kernel::creation::primitive(
            om9_kernel::creation::Primitive::Box { x_mm: 10.0, y_mm: 10.0, z_mm: 10.0 },
            0.01, &limits,
        ).unwrap();
        let edges = [shape.display.edges[0], shape.display.edges[2]];
        let mut doc = Document::new("SelectEdges routes");
        let entity = om9_core::model::Entity::new("box", Geometry::Brep(shape), None);
        let id = entity.id;
        doc.entities.insert(id, entity);
        let mut app = MatrixApp::new(doc).unwrap();
        let before = app.controller.document().clone();
        app.controller.submit("FilletEdge").unwrap();
        app.controller.submit("NextRadius=1").unwrap();
        let make_reference = |app: &MatrixApp, edge| {
            let object = app.controller.scene().unwrap().objects.iter().find(|o| o.root == id && o.path.is_empty()).unwrap();
            om9_app::session::SelectedSubobject {
                root: id, path: vec![], subset: om9_scene::scene::Subelement::BrepEdge(edge),
                source: object.source, world_epoch: app.controller.scene().unwrap().epoch,
            }
        };
        let reference = make_reference(&app, edges[0]);
        app.controller.solid_fillet_pick_edge(reference).unwrap();
        app.controller.submit("").unwrap();
        app.controller.solid_fillet_select_handle(0).unwrap();
        app.controller.submit("CurrentRadius=1.5").unwrap();
        let ctx = egui::Context::default();
        app.transcript.set_prompt(&app.controller.prompt());
        app.focus_command_editor();
        for _ in 0..4 { frame(&mut app, &ctx, vec![]); }
        activate(&mut app, &ctx, "SelectEdges", route);
        assert!(app.controller.solid_fillet_needs_edge());
        assert!(app.controller.solid_fillet_preview().is_none());
        assert_eq!(app.controller.subselection().len(), 1);
        app.controller.submit("NextRadius=2").unwrap();
        let reference = make_reference(&app, edges[1]);
        app.controller.solid_fillet_pick_edge(reference).unwrap();
        app.controller.submit("").unwrap();
        let radii: Vec<_> = app.controller.solid_fillet_handles().iter().map(|h| h.2).collect();
        assert_eq!(radii, vec![1.5, 1.0, 2.0, 2.0]);
        let Geometry::Brep(original) = &before.entities[&id].geometry else { panic!() };
        let expected = om9_kernel::editing::fillet_with_end_radii(original, &[(0, 1.5, 1.0), (2, 2.0, 2.0)], 0.01, &limits).unwrap();
        let expected = om9_kernel::creation::measure(&expected, &limits).unwrap().volume_mm3;
        let (preview, _) = app.controller.solid_fillet_preview().unwrap();
        let actual = om9_kernel::creation::measure(preview, &limits).unwrap().volume_mm3;
        assert!((actual - expected).abs() < 1e-5);
        assert_eq!(app.controller.document(), &before);
        assert_eq!(app.controller.coordinator().undo_len(), 0);
        app.controller.submit("").unwrap();
        assert_eq!(app.controller.coordinator().undo_len(), 1);
        app.controller.undo().unwrap();
        assert_eq!(app.controller.document(), &before);
    }
}

#[test]
fn fillet_next_radius_option_accepts_keyboard_and_pointer_draft() {
    for route in [Route::Keyboard, Route::Pointer] {
        let (mut app, ctx) = setup("FilletEdge");
        activate(&mut app, &ctx, "NextRadius", route);
        assert_eq!(app.transcript.draft(), "NextRadius=");
        frame(&mut app, &ctx, vec![Event::Text("1".into())]);
        enter(&mut app, &ctx);
        assert_eq!(label(&app.controller, "NextRadius"), "NextRadius=1");
        assert_eq!(app.controller.coordinator().undo_len(), 0);
    }
}

#[test]
fn boolean_union_delete_input_uses_shared_option_routes() {
    for route in [Route::Keyboard, Route::Pointer, Route::Typed] {
        let (mut app, ctx) = setup("BooleanUnion");
        assert_eq!(label(&app.controller, "DeleteInput"), "DeleteInput=No");
        activate(&mut app, &ctx, "DeleteInput", route);
        assert_eq!(label(&app.controller, "DeleteInput"), "DeleteInput=Yes");
        activate(&mut app, &ctx, "DeleteInput", route);
        assert_eq!(label(&app.controller, "DeleteInput"), "DeleteInput=No");
        assert!(app.controller.document().entities.is_empty());
        assert_eq!(app.controller.coordinator().undo_len(), 0);
    }
}

#[test]
fn boolean_intersection_delete_input_uses_shared_option_routes() {
    for route in [Route::Keyboard, Route::Pointer, Route::Typed] {
        let (mut app, ctx) = setup("BooleanIntersection");
        assert_eq!(label(&app.controller, "DeleteInput"), "DeleteInput=No");
        activate(&mut app, &ctx, "DeleteInput", route);
        assert_eq!(label(&app.controller, "DeleteInput"), "DeleteInput=Yes");
        activate(&mut app, &ctx, "DeleteInput", route);
        assert_eq!(label(&app.controller, "DeleteInput"), "DeleteInput=No");
        assert!(app.controller.document().entities.is_empty());
        assert_eq!(app.controller.coordinator().undo_len(), 0);
    }
}

#[test]
fn boolean_split_delete_input_uses_shared_option_routes() {
    for route in [Route::Keyboard, Route::Pointer, Route::Typed] {
        let (mut app, ctx) = setup("BooleanSplit");
        assert_eq!(label(&app.controller, "DeleteInput"), "DeleteInput=No");
        activate(&mut app, &ctx, "DeleteInput", route);
        assert_eq!(label(&app.controller, "DeleteInput"), "DeleteInput=Yes");
        activate(&mut app, &ctx, "DeleteInput", route);
        assert_eq!(label(&app.controller, "DeleteInput"), "DeleteInput=No");
        assert!(app.controller.document().entities.is_empty());
        assert_eq!(app.controller.coordinator().undo_len(), 0);
    }
}

#[test]
fn create_solid_delete_input_uses_shared_option_routes() {
    for route in [Route::Keyboard, Route::Pointer, Route::Typed] {
        let (mut app, ctx) = setup("CreateSolid");
        assert_eq!(label(&app.controller, "DeleteInput"), "DeleteInput=No");
        activate(&mut app, &ctx, "DeleteInput", route);
        assert_eq!(label(&app.controller, "DeleteInput"), "DeleteInput=Yes");
        activate(&mut app, &ctx, "DeleteInput", route);
        assert_eq!(label(&app.controller, "DeleteInput"), "DeleteInput=No");
        assert!(app.controller.document().entities.is_empty());
        assert_eq!(app.controller.coordinator().undo_len(), 0);
    }
}

#[test]
fn boolean_two_delete_input_uses_shared_option_routes() {
    for route in [Route::Keyboard, Route::Pointer, Route::Typed] {
        let (mut app, ctx) = setup("Boolean2Objects");
        assert_eq!(label(&app.controller, "DeleteInput"), "DeleteInput=No");
        activate(&mut app, &ctx, "DeleteInput", route);
        assert_eq!(label(&app.controller, "DeleteInput"), "DeleteInput=Yes");
        activate(&mut app, &ctx, "DeleteInput", route);
        assert_eq!(label(&app.controller, "DeleteInput"), "DeleteInput=No");
        assert!(app.controller.document().entities.is_empty());
        assert_eq!(app.controller.coordinator().undo_len(), 0);
    }
}

#[test]
fn boolean_two_additional_operations_use_shared_option_routes() {
    for (name, expected) in [
        ("ReverseDifference", 1000.0),
        ("InversionIntersection", 1500.0),
    ] {
        for route in [Route::Keyboard, Route::Pointer, Route::Typed] {
            let limits = Limits::default();
            let mut doc = Document::new("Boolean2 options");
            let mut ids = Vec::new();
            for (width, x) in [(10.0, 0.0), (15.0, 5.0)] {
                let body = om9_kernel::creation::primitive(
                    om9_kernel::creation::Primitive::Box {
                        x_mm: width,
                        y_mm: 10.0,
                        z_mm: 10.0,
                    },
                    0.01,
                    &limits,
                )
                .unwrap();
                let mut entity = om9_core::model::Entity::new("box", Geometry::Brep(body), None);
                entity.placement = om9_core::geometry::Transform::translation([x, 0.0, 0.0]);
                ids.push(entity.id);
                doc.entities.insert(entity.id, entity);
            }
            let mut app = MatrixApp::new(doc).unwrap();
            app.controller.select_objects(&ids).unwrap();
            app.controller.submit("Boolean2Objects").unwrap();
            app.controller.submit("DeleteInput=Yes").unwrap();
            app.transcript.set_prompt(&app.controller.prompt());
            let ctx = egui::Context::default();
            app.focus_command_editor();
            for _ in 0..4 {
                frame(&mut app, &ctx, vec![]);
            }
            activate(&mut app, &ctx, name, route);
            assert_eq!(app.controller.document().entities.len(), 2);
            assert_eq!(app.controller.coordinator().undo_len(), 0);
            app.controller.submit("").unwrap();
            assert_eq!(
                app.controller.document().entities.len(),
                1,
                "{name}: {route:?}"
            );
            let Geometry::Brep(body) = &app
                .controller
                .document()
                .entities
                .values()
                .next()
                .unwrap()
                .geometry
            else {
                panic!("BRep result");
            };
            let volume = om9_kernel::creation::measure(body, app.controller.coordinator().limits())
                .unwrap()
                .volume_mm3;
            assert!(
                (volume - expected).abs() < 1e-6,
                "{name}: {route:?}: {volume}"
            );
            assert_eq!(app.controller.coordinator().undo_len(), 1);
        }
    }
}

#[test]
fn commands_share_keyboard_pointer_and_typed_activation() {
    let cases = [
        ("Line", "BothSides", None, "BothSides=Yes"),
        ("Rectangle", "Square", None, "Square=Yes"),
        ("Circle", "Deformable", None, "Deformable=Yes"),
        ("Ellipse", "Deformable", None, "Deformable=Yes"),
        ("Arc", "Center", None, "Center=Yes"),
        ("Polygon", "Circumscribed", None, "Circumscribed=Yes"),
        ("Helix", "Turns", Some("2"), "Turns=2"),
        ("Spiral", "Height", Some("5"), "Height=5"),
        ("Curve", "Degree", Some("2"), "Degree=2"),
        ("InterpCrv", "Degree", Some("5"), "Degree=5"),
        ("Box", "Center", None, "Center=Yes"),
        ("Sphere", "3Point", None, "3Point=Yes"),
    ];
    for route in [Route::Keyboard, Route::Pointer, Route::Typed] {
        for (command, name, value, expected) in cases {
            let (mut app, ctx) = setup(command);
            activate(&mut app, &ctx, name, route);
            if let Some(value) = value {
                assert_eq!(
                    app.transcript.draft(),
                    format!("{name}="),
                    "{command} {route:?}"
                );
                assert_eq!(app.controller.coordinator().undo_len(), 0);
                frame(&mut app, &ctx, vec![Event::Text(value.into())]);
                enter(&mut app, &ctx);
            }
            assert_eq!(
                label(&app.controller, name),
                expected,
                "{command} {route:?}"
            );
            assert_eq!(app.transcript.draft(), "");
            assert!(app.controller.document().entities.is_empty());
            assert_eq!(app.controller.coordinator().undo_len(), 0);
        }
    }
}

#[test]
fn numeric_options_keep_invalid_draft_and_commit_actual_geometry_once() {
    for route in [Route::Keyboard, Route::Pointer, Route::Typed] {
        let (mut app, ctx) = setup("Circle");
        app.controller.accept_world_point([0.; 3]).unwrap();
        app.transcript.set_prompt(&app.controller.prompt());
        for _ in 0..3 {
            frame(&mut app, &ctx, vec![]);
        }
        activate(&mut app, &ctx, "Radius", route);
        assert_eq!(app.transcript.draft(), "Radius=");
        frame(&mut app, &ctx, vec![Event::Text("-5".into())]);
        enter(&mut app, &ctx);
        assert_eq!(app.transcript.draft(), "Radius=-5");
        assert!(app.controller.document().entities.is_empty());
        app.transcript.set_draft("radius=5");
        enter(&mut app, &ctx);
        assert_eq!(app.controller.document().entities.len(), 1);
        let geometry = &app
            .controller
            .document()
            .entities
            .values()
            .next()
            .unwrap()
            .geometry;
        assert!(
            matches!(geometry, Geometry::Curve(Curve::Circle { radius, center, .. }) if *radius == 5. && *center == [0.;3])
        );
        assert_eq!(app.controller.coordinator().undo_len(), 1);
        app.controller.submit("Undo").unwrap();
        assert!(app.controller.document().entities.is_empty());
    }
}

#[test]
fn collisions_are_unique_numeric_input_and_paste_are_preserved() {
    let (mut app, ctx) = setup("Circle");
    let options = app.controller.command_options();
    let mut used = std::collections::BTreeSet::new();
    for option in &options {
        if let Some(shortcut) = option.shortcut {
            assert!(shortcut.is_ascii_alphabetic());
            assert!(used.insert(shortcut));
            assert_eq!(
                app.controller.resolve_command_option(&shortcut.to_string()),
                Some(option.id)
            );
        }
    }
    assert!(
        options
            .iter()
            .find(|o| o.id.name() == "Cancel")
            .unwrap()
            .shortcut
            .is_none()
    );
    frame(
        &mut app,
        &ctx,
        vec![key(Key::Num2, true), Event::Text("2".into())],
    );
    assert_eq!(app.transcript.draft(), "2");
    app.transcript.set_draft("");
    frame(&mut app, &ctx, vec![Event::Paste("Deformable".into())]);
    assert_eq!(label(&app.controller, "Deformable"), "Deformable=No");
    assert_eq!(app.transcript.draft(), "Deformable");
}

#[test]
fn fixed_choices_refresh_the_selected_value_on_every_input_route() {
    for route in [Route::Keyboard, Route::Pointer, Route::Typed] {
        let (mut app, ctx) = setup("Circle");
        app.controller.accept_world_point([0.; 3]).unwrap();
        app.transcript.set_prompt(&app.controller.prompt());
        for _ in 0..3 {
            frame(&mut app, &ctx, vec![]);
        }
        activate(&mut app, &ctx, "Size=Area", route);
        assert_eq!(label(&app.controller, "Size=Area"), "Size=Area (Yes)");
        assert_eq!(
            label(&app.controller, "Size=Diameter"),
            "Size=Diameter (No)"
        );
        activate(&mut app, &ctx, "Size=Diameter", route);
        assert_eq!(
            label(&app.controller, "Size=Diameter"),
            "Size=Diameter (Yes)"
        );
        assert_eq!(label(&app.controller, "Size=Area"), "Size=Area (No)");
        assert!(app.controller.document().entities.is_empty());
        assert_eq!(app.controller.coordinator().undo_len(), 0);
    }
}

#[test]
fn stale_options_are_refused_and_value_activation_does_not_mutate_owner() {
    let mut c = Controller::new(Document::new("stale"), Limits::default()).unwrap();
    c.submit("Helix").unwrap();
    let turns = c.resolve_command_option("Turns").unwrap();
    let before = c.prompt().to_owned();
    assert_eq!(
        c.activate_command_option(turns).unwrap(),
        CommandOptionEffect::Draft("Turns=".into())
    );
    assert_eq!(c.prompt(), before);
    c.cancel();
    c.submit("Helix").unwrap();
    assert!(
        c.activate_command_option(turns).is_err(),
        "same-owner new run must reject the old ID"
    );
    c.cancel();
    c.submit("Circle").unwrap();
    assert!(c.activate_command_option(turns).is_err());
    assert!(c.document().entities.is_empty());
    assert_eq!(c.coordinator().undo_len(), 0);
}

#[test]
fn line_repeat_slot_never_leaks_options_into_polyline() {
    use om9_app::session::{InvocationOrigin, LINE};
    for origin in [InvocationOrigin::Menu, InvocationOrigin::Cmd] {
        let (mut app, ctx) = setup("Line");
        app.controller.accept_world_point([0.; 3]).unwrap();
        app.controller.accept_world_point([10., 0., 0.]).unwrap();
        assert_eq!(app.controller.repeat_slot(), Some(LINE));
        match origin {
            InvocationOrigin::Menu => app.controller.dispatch("OM9-CURVE-001", origin).unwrap(),
            InvocationOrigin::Cmd => app.controller.submit("Polyline").unwrap(),
            _ => unreachable!(),
        }
        app.transcript.set_prompt(&app.controller.prompt());
        for _ in 0..3 {
            frame(&mut app, &ctx, vec![]);
        }
        assert!(app.controller.prompt().starts_with("Start of polyline"));
        for name in ["BothSides", "Length"] {
            assert!(
                app.controller.resolve_command_option(name).is_none(),
                "{origin:?}: {name}"
            );
            assert!(
                app.command_option_token_rect(name).is_none(),
                "{origin:?}: {name}"
            );
        }
        for name in ["PersistentClose", "Mode", "Helpers"] {
            assert!(
                app.command_option_token_rect(name).is_some(),
                "{origin:?}: {name}"
            );
        }
        app.controller.accept_world_point([0.; 3]).unwrap();
        app.transcript.set_prompt(&app.controller.prompt());
        for _ in 0..3 {
            frame(&mut app, &ctx, vec![]);
        }
        assert!(app.command_option_token_rect("BothSides").is_none());
        assert!(app.command_option_token_rect("Length").is_some());
        assert_eq!(app.controller.document().entities.len(), 1);
        assert_eq!(app.controller.coordinator().undo_len(), 1);
        app.controller.cancel();
    }
}

#[test]
fn line_length_snapshot_uses_the_input_unit_without_rescaling_geometry() {
    let mut document = Document::new("inch options");
    document.page_unit = om9_core::geometry::Unit::Inch;
    let mut c = Controller::new(document, Limits::default()).unwrap();
    c.submit("Line").unwrap();
    c.submit("length=2.5").unwrap();
    assert_eq!(label(&c, "Length"), "Length=2.5");
    c.accept_world_point([0.; 3]).unwrap();
    c.accept_world_point([100., 0., 0.]).unwrap();
    assert!(
        matches!(&c.document().entities.values().next().unwrap().geometry, Geometry::Curve(Curve::Line { end, .. }) if (end[0]-63.5).abs() < 1e-9)
    );
    assert_eq!(c.coordinator().undo_len(), 1);
}

#[test]
fn accelerators_do_not_repeat_or_interrupt_ime_drafts_and_other_editors() {
    let (mut app, ctx) = setup("Rectangle");
    frame(
        &mut app,
        &ctx,
        vec![key(Key::S, true), Event::Text("s".into())],
    );
    assert_eq!(label(&app.controller, "Square"), "Square=Yes");
    let mut repeat = key(Key::S, true);
    if let Event::Key { repeat, .. } = &mut repeat {
        *repeat = true;
    }
    frame(&mut app, &ctx, vec![repeat, Event::Text("s".into())]);
    assert_eq!(label(&app.controller, "Square"), "Square=Yes");
    assert_eq!(app.transcript.draft(), "");
    frame(&mut app, &ctx, vec![key(Key::S, false)]);
    app.transcript.set_draft("Length=12");
    frame(
        &mut app,
        &ctx,
        vec![key(Key::S, true), Event::Text("s".into())],
    );
    assert_eq!(label(&app.controller, "Square"), "Square=Yes");
    assert!(app.transcript.draft().contains("Length=12"));
    app.transcript.set_draft("");
    frame(
        &mut app,
        &ctx,
        vec![
            Event::Ime(egui::ImeEvent::Preedit {
                text: "s".into(),
                active_range_chars: None,
            }),
            key(Key::S, true),
        ],
    );
    assert_eq!(label(&app.controller, "Square"), "Square=Yes");
    frame(
        &mut app,
        &ctx,
        vec![Event::Ime(egui::ImeEvent::Commit("s".into()))],
    );
    app.transcript.set_draft("");
    let other = egui::Id::new("another-editor");
    ctx.memory_mut(|m| m.request_focus(other));
    frame(
        &mut app,
        &ctx,
        vec![key(Key::S, true), Event::Text("s".into())],
    );
    assert_eq!(label(&app.controller, "Square"), "Square=Yes");
}

#[test]
fn line_prompt_carries_its_options_inline() {
    let (mut app, ctx) = setup("Line");
    app.transcript.set_prompt(&app.controller.prompt());
    frame(&mut app, &ctx, vec![]);
    let prompt = app.controller.prompt();
    assert!(prompt.contains("BothSides=No"), "{prompt}");
    assert!(prompt.contains("Length="), "{prompt}");
    assert!(app.command_option_token_rect("BothSides").is_some());
    assert!(app.command_option_token_rect("Length").is_some());
}
