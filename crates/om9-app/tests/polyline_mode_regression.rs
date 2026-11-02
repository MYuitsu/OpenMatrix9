// SPDX-License-Identifier: LGPL-2.1-or-later
use eframe::egui::{self, Event, Key, Modifiers, Pos2, Rect, Vec2};
use om9_app::{
    session::{Controller, InvocationOrigin},
    ui::MatrixApp,
};
use om9_core::{
    budget::Limits,
    geometry::{Curve, Geometry},
    model::Document,
};

fn frame(app: &mut MatrixApp, ctx: &egui::Context, events: Vec<Event>) {
    let mut raw = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1600., 900.))),
        events,
        ..Default::default()
    };
    eframe::App::raw_input_hook(app, ctx, &mut raw);
    let mut output = ctx.run_ui(raw, |ui| app.show(ui));
    output.textures_delta.clear();
}
fn key(key: Key, text: &str) -> Vec<Event> {
    vec![
        Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        },
        Event::Text(text.into()),
    ]
}
fn app() -> (MatrixApp, egui::Context) {
    let mut app = MatrixApp::new(Document::new("Mode regression")).unwrap();
    let ctx = egui::Context::default();
    app.controller
        .dispatch("OM9-CURVE-001", InvocationOrigin::Menu)
        .unwrap();
    app.transcript.set_prompt(&app.controller.prompt());
    for _ in 0..3 {
        frame(&mut app, &ctx, vec![]);
    }
    (app, ctx)
}
fn click(app: &mut MatrixApp, ctx: &egui::Context, name: &str) {
    let pos = app
        .command_option_token_rect(name)
        .expect("inline option")
        .center();
    for event in [
        Event::PointerMoved(pos),
        Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Modifiers::NONE,
        },
        Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        },
    ] {
        frame(app, ctx, vec![event]);
    }
    frame(app, ctx, vec![]);
}
#[test]
fn m_toggles_twice_and_updates_live_prompt_without_a_draft() {
    let (mut app, ctx) = app();
    app.controller.accept_world_point([0., 0., 0.]).unwrap();
    app.controller.accept_world_point([10., 0., 0.]).unwrap();
    for (text, expected) in [("M", "Mode=Arc"), ("m", "Mode=Line")] {
        frame(&mut app, &ctx, key(Key::M, text));
        assert!(
            app.controller.prompt().contains(expected),
            "expected {expected}, got {} (draft {:?})",
            app.controller.prompt(),
            app.transcript.draft()
        );
        assert!(
            app.transcript
                .text()
                .lines()
                .last()
                .unwrap()
                .contains(expected)
        );
        assert_eq!(app.transcript.draft(), "");
        frame(
            &mut app,
            &ctx,
            vec![Event::Key {
                key: Key::M,
                physical_key: None,
                pressed: false,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
        );
    }
    assert!(app.controller.document().entities.is_empty());
    assert_eq!(app.controller.coordinator().undo_len(), 0);
}
#[test]
fn clicking_mode_and_flags_uses_the_same_toggle_as_the_keyboard() {
    let (mut app, ctx) = app();
    click(&mut app, &ctx, "Mode");
    assert!(app.controller.prompt().contains("Mode=Arc"));
    click(&mut app, &ctx, "Mode");
    assert!(app.controller.prompt().contains("Mode=Line"));
    click(&mut app, &ctx, "PersistentClose");
    assert!(app.controller.polyline_options().unwrap().0);
    frame(&mut app, &ctx, key(Key::P, "P"));
    assert!(!app.controller.polyline_options().unwrap().0);
    click(&mut app, &ctx, "Helpers");
    frame(&mut app, &ctx, key(Key::H, "h"));
    assert!(!app.controller.polyline_options().unwrap().1);
}
#[test]
fn shortcuts_preserve_nonempty_drafts_paste_and_ime() {
    let (mut app, ctx) = app();
    app.transcript.set_draft("Mode=");
    app.focus_command_editor();
    frame(&mut app, &ctx, key(Key::M, "m"));
    assert!(app.controller.prompt().contains("Mode=Line"));
    assert!(app.transcript.draft().starts_with("Mode="));
    app.transcript.set_draft("");
    app.focus_command_editor();
    frame(&mut app, &ctx, vec![Event::Paste("Mode=Arc".into())]);
    assert!(app.controller.prompt().contains("Mode=Line"));
    assert_eq!(app.transcript.draft(), "Mode=Arc");
    app.transcript.set_draft("");
    frame(
        &mut app,
        &ctx,
        vec![
            Event::Ime(egui::ImeEvent::Commit("m".into())),
            key(Key::M, "m").remove(0),
        ],
    );
    assert!(app.controller.prompt().contains("Mode=Line"));
}
fn segments(c: &Controller) -> Vec<Curve> {
    let Geometry::Curve(Curve::Polycurve { segments, .. }) =
        &c.document().entities.values().next().unwrap().geometry
    else {
        panic!("polycurve");
    };
    segments.clone()
}
#[test]
fn menu_and_cmd_build_the_same_line_arc_line_with_one_undo() {
    let mut results = Vec::new();
    for menu in [false, true] {
        let mut c = Controller::new(Document::new("arc"), Limits::default()).unwrap();
        if menu {
            c.dispatch("OM9-CURVE-001", InvocationOrigin::Menu).unwrap();
        } else {
            c.submit("Polyline").unwrap();
        }
        for p in [[0., 0., 0.], [10., 0., 0.]] {
            c.accept_world_point(p).unwrap();
        }
        c.submit("m").unwrap();
        assert!(c.prompt().contains("Mode=Arc"));
        assert!(c.polyline_preview([15., 5., 0.]).len() > 2);
        c.accept_world_point([15., 5., 0.]).unwrap();
        c.submit("Mode").unwrap();
        c.accept_world_point([20., 5., 0.]).unwrap();
        c.submit("").unwrap();
        let s = segments(&c);
        assert_eq!(s.len(), 3);
        let Curve::Circle { center, radius, .. } = s[1] else {
            panic!("an exact analytic arc, not a line");
        };
        assert!((radius - 5.).abs() < 1e-10);
        assert!((center[0] - 10.).abs() < 1e-10 && (center[1] - 5.).abs() < 1e-10);
        let jet =
            om9_core::curve_ops::fraction_derivatives(&s[1], 0., 1, &Limits::default()).unwrap();
        assert!(jet[1][0] > 0. && jet[1][1].abs() < 1e-10);
        assert_eq!(c.coordinator().undo_len(), 1);
        let serialized = serde_json::to_string(c.document()).unwrap();
        let restored: Document = serde_json::from_str(&serialized).unwrap();
        assert_eq!(
            restored.entities.values().next().unwrap().geometry,
            c.document().entities.values().next().unwrap().geometry
        );
        c.undo().unwrap();
        assert!(c.document().entities.is_empty());
        c.redo().unwrap();
        assert_eq!(segments(&c), s);
        results.push(s);
    }
    assert_eq!(results[0], results[1]);
}
#[test]
fn typed_options_and_coordinates_construct_arc_direction_and_center() {
    let mut c = Controller::new(Document::new("options"), Limits::default()).unwrap();
    for text in [
        "Polyline", "m", "0,0,0", "10,0,0", "5,5,0", "c", "0,5,0", "0,10,0", "",
    ] {
        c.submit(text).unwrap();
    }
    let s = segments(&c);
    assert_eq!(s.len(), 2);
    for curve in s {
        assert!(matches!(curve, Curve::Circle { .. }));
    }
}
#[test]
fn invalid_arc_retains_points_and_cancel_keeps_the_document_empty() {
    let mut c = Controller::new(Document::new("negative"), Limits::default()).unwrap();
    for text in ["Polyline", "0,0,0", "10,0,0", "m"] {
        c.submit(text).unwrap();
    }
    assert!(c.submit("20,0,0").is_err());
    assert!(c.prompt().contains("Mode=Arc"));
    c.submit("15,5,0").unwrap();
    c.submit("Undo").unwrap();
    c.submit("15,5,0").unwrap();
    c.submit("Cancel").unwrap();
    assert!(c.document().entities.is_empty());
    assert_eq!(c.coordinator().undo_len(), 0);
}

#[test]
fn holding_m_toggles_once_and_keeps_the_draft_empty() {
    let (mut app, ctx) = app();
    frame(&mut app, &ctx, key(Key::M, "m"));
    assert!(app.controller.prompt().contains("Mode=Arc"));
    let mut repeated = key(Key::M, "m");
    if let Event::Key { repeat, .. } = &mut repeated[0] {
        *repeat = true;
    }
    frame(&mut app, &ctx, repeated);
    assert!(app.controller.prompt().contains("Mode=Arc"));
    assert_eq!(app.transcript.draft(), "");
}

#[test]
fn viewport_typing_m_uses_mode_instead_of_mid_snap() {
    let (mut app, ctx) = app();
    ctx.memory_mut(|memory| memory.surrender_focus(egui::Id::new("matrix-command-transcript")));
    frame(&mut app, &ctx, vec![]);
    frame(&mut app, &ctx, key(Key::M, "m"));
    assert!(app.controller.prompt().contains("Mode=Arc"));
    assert_eq!(app.transcript.draft(), "");
}

#[test]
fn switching_back_after_an_arc_keeps_close_distinct_from_persistent_close() {
    let (mut app, ctx) = app();
    for text in ["0,0,0", "10,0,0", "Mode=Arc", "15,5,0"] {
        app.controller.submit(text).unwrap();
    }
    app.transcript.set_prompt(&app.controller.prompt());
    frame(&mut app, &ctx, key(Key::M, "m"));
    assert!(app.controller.prompt().contains("Mode=Line"));
    let close = app.command_option_token_rect("Close").unwrap();
    let persistent = app.command_option_token_rect("PersistentClose").unwrap();
    assert!(close.left() > persistent.right());
    click(&mut app, &ctx, "Close");
    assert!(!app.controller.is_running());
    let s = segments(&app.controller);
    assert_eq!(s.len(), 3);
    assert!(matches!(s[1], Curve::Circle { .. }));
    assert_eq!(app.controller.coordinator().undo_len(), 1);
}
