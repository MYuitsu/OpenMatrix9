// SPDX-License-Identifier: LGPL-2.1-or-later
use eframe::egui::{self, Event, Key, Modifiers, Pos2, Rect, Vec2};
use om9_app::{
    polyline_options::{
        PolylineOptionId as Id, PolylineOptionInterface, PolylineOptionValue as Value,
    },
    session::Controller,
    ui::MatrixApp,
};
use om9_core::{
    budget::Limits,
    geometry::{Curve, Geometry},
    model::Document,
};

#[derive(Clone, Copy, Debug)]
enum Route {
    Keyboard,
    Pointer,
    Typed,
}

fn frame(app: &mut MatrixApp, ctx: &egui::Context, events: Vec<Event>) {
    let mut raw = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1800., 1000.))),
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
fn setup(option: Id) -> (MatrixApp, egui::Context) {
    let mut app = MatrixApp::new(Document::new("Polyline interface")).unwrap();
    app.controller.submit("Polyline").unwrap();
    let count = match option {
        Id::Close => 3,
        Id::Undo => 2,
        Id::Length | Id::Direction | Id::Center => 1,
        _ => 0,
    };
    let points = if option == Id::Center {
        [[5., 0., 0.], [10., 0., 0.], [10., 10., 0.]]
    } else {
        [[0., 0., 0.], [10., 0., 0.], [10., 10., 0.]]
    };
    for point in points.into_iter().take(count) {
        app.controller.accept_world_point(point).unwrap();
    }
    if matches!(option, Id::Direction | Id::Center) {
        app.controller.submit("Mode=Arc").unwrap();
    }
    app.transcript.set_prompt(&app.controller.prompt());
    let ctx = egui::Context::default();
    for _ in 0..3 {
        frame(&mut app, &ctx, vec![]);
    }
    (app, ctx)
}
fn activate(app: &mut MatrixApp, ctx: &egui::Context, id: Id, route: Route) {
    match route {
        Route::Keyboard => {
            let pressed = match id.shortcut() {
                'P' => Key::P,
                'M' => Key::M,
                'H' => Key::H,
                'L' => Key::L,
                'C' => Key::C,
                'D' => Key::D,
                'U' => Key::U,
                _ => unreachable!(),
            };
            frame(
                app,
                ctx,
                vec![
                    key(pressed, true),
                    Event::Text(id.shortcut().to_ascii_lowercase().to_string()),
                ],
            );
            frame(app, ctx, vec![key(pressed, false)]);
        }
        Route::Pointer => {
            let at = app
                .command_option_token_rect(id.name())
                .expect("visible descriptor click target")
                .center();
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
            app.focus_command_editor();
            frame(app, ctx, vec![Event::Paste(id.name().to_ascii_lowercase())]);
            enter(app, ctx);
        }
    }
    assert_eq!(app.transcript.draft(), "", "{id:?}, {route:?}");
    assert_eq!(
        app.transcript
            .text()
            .lines()
            .last()
            .unwrap()
            .trim_end_matches([':', ' ']),
        app.controller.prompt().trim_end_matches([':', ' ']),
        "live prompt matches the owner"
    );
}
fn value(c: &Controller, id: Id) -> Value {
    c.polyline_command_options()
        .into_iter()
        .find(|option| option.id == id)
        .unwrap()
        .value
}
fn curve(c: &Controller) -> &Curve {
    let Geometry::Curve(curve) = &c
        .document()
        .entities
        .values()
        .next()
        .expect("committed entity")
        .geometry
    else {
        panic!("curve");
    };
    curve
}

#[test]
fn every_polyline_parameter_has_the_same_keyboard_pointer_and_typed_behavior() {
    for route in [Route::Keyboard, Route::Pointer, Route::Typed] {
        for id in [
            Id::PersistentClose,
            Id::Mode,
            Id::Helpers,
            Id::Length,
            Id::Close,
            Id::Direction,
            Id::Center,
            Id::Undo,
        ] {
            let (mut app, ctx) = setup(id);
            activate(&mut app, &ctx, id, route);
            match id {
                Id::PersistentClose | Id::Helpers => {
                    assert_eq!(value(&app.controller, id), Value::Flag(true))
                }
                Id::Mode => assert_eq!(value(&app.controller, id), Value::Choice("Arc")),
                Id::Length => {
                    assert_eq!(app.controller.prompt(), "Length");
                    frame(&mut app, &ctx, vec![Event::Text("5".into())]);
                    enter(&mut app, &ctx);
                    assert_eq!(value(&app.controller, id), Value::Number(Some(5.)));
                    assert!(
                        app.transcript
                            .text()
                            .lines()
                            .last()
                            .unwrap()
                            .contains("Length=5")
                    );
                    app.controller.accept_world_point([30., 40., 0.]).unwrap();
                    app.controller.submit("").unwrap();
                    let Curve::Polycurve { segments, .. } = curve(&app.controller) else {
                        panic!("polycurve");
                    };
                    assert_eq!(
                        segments[0],
                        Curve::Line {
                            start: [0.; 3],
                            end: [3., 4., 0.],
                            domain: [0., 1.]
                        }
                    );
                }
                Id::Direction => {
                    assert!(app.controller.prompt().starts_with("Direction"));
                    app.controller.accept_world_point([1., 0., 0.]).unwrap();
                    assert_eq!(value(&app.controller, id), Value::Point(Some([1., 0., 0.])));
                    assert!(app.controller.prompt().contains("Direction=1,0,0"));
                    app.controller.accept_world_point([5., 5., 0.]).unwrap();
                    app.controller.submit("").unwrap();
                    let Curve::Polycurve { segments, .. } = curve(&app.controller) else {
                        panic!("polycurve");
                    };
                    assert!(
                        matches!(segments[0], Curve::Circle { radius, .. } if (radius-5.).abs()<1e-10)
                    );
                }
                Id::Center => {
                    assert!(app.controller.prompt().starts_with("Center"));
                    app.controller.accept_world_point([0.; 3]).unwrap();
                    assert_eq!(value(&app.controller, id), Value::Point(Some([0.; 3])));
                    assert!(app.controller.prompt().contains("Center=0,0,0"));
                    app.controller.accept_world_point([0., 5., 0.]).unwrap();
                    app.controller.submit("").unwrap();
                    let Curve::Polycurve { segments, .. } = curve(&app.controller) else {
                        panic!("polycurve");
                    };
                    assert!(
                        matches!(segments[0], Curve::Circle { radius, .. } if (radius-5.).abs()<1e-10)
                    );
                }
                Id::Close => {
                    assert!(!app.controller.is_running());
                    let Curve::Polycurve { segments, .. } = curve(&app.controller) else {
                        panic!("polycurve");
                    };
                    assert_eq!(segments.len(), 3);
                    assert_eq!(app.controller.coordinator().undo_len(), 1);
                }
                Id::Undo => {
                    assert_eq!(app.controller.polyline_preview([20., 0., 0.]).len(), 1);
                    assert!(app.controller.document().entities.is_empty());
                    assert_eq!(app.controller.coordinator().undo_len(), 0);
                }
            }
        }
    }
}

#[test]
fn descriptors_expose_current_values_and_unambiguous_stage_shortcuts() {
    let mut c = Controller::new(Document::new("metadata"), Limits::default()).unwrap();
    c.submit("Polyline").unwrap();
    for text in ["p", "h", "m", "m", "0,0,0", "10,0,0", "10,10,0"] {
        c.submit(text).unwrap();
    }
    assert_eq!(c.resolve_polyline_option("c"), Some(Id::Close));
    for id in [
        Id::PersistentClose,
        Id::Mode,
        Id::Helpers,
        Id::Length,
        Id::Close,
        Id::Undo,
    ] {
        assert_eq!(c.resolve_polyline_option(id.name()), Some(id));
        assert_eq!(
            c.resolve_polyline_option(&id.shortcut().to_ascii_lowercase().to_string()),
            Some(id)
        );
    }
    c.activate_polyline_option(Id::Mode).unwrap();
    assert_eq!(c.resolve_polyline_option("c"), Some(Id::Center));
    assert!(c.resolve_polyline_option("Length").is_none());
    let before = c.prompt().to_owned();
    assert!(c.activate_polyline_option(Id::Length).is_err());
    assert_eq!(c.prompt(), before);
    assert!(c.document().entities.is_empty());
}

#[test]
fn invalid_value_retains_the_parameter_stage_and_has_no_document_effect() {
    let (mut app, ctx) = setup(Id::Length);
    activate(&mut app, &ctx, Id::Length, Route::Pointer);
    frame(&mut app, &ctx, vec![Event::Text("-5".into())]);
    enter(&mut app, &ctx);
    assert_eq!(app.controller.prompt(), "Length");
    assert_eq!(app.transcript.draft(), "-5");
    assert!(app.controller.polyline_command_options().is_empty());
    assert!(app.controller.document().entities.is_empty());
    assert_eq!(app.controller.coordinator().undo_len(), 0);
}
