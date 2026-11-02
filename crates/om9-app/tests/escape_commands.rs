// SPDX-License-Identifier: LGPL-2.1-or-later
use eframe::egui::{self, Event, Key, Modifiers, Rect, Vec2};
use om9_app::session::{
    InvocationOrigin, head_commands::HEAD_BUILDER, settings_commands::PRONG_BUILDER,
};
use om9_app::ui::MatrixApp;
use om9_core::{
    gem::{GemSpec, gem_entity},
    geometry::{Curve, Geometry},
    model::{Document, Entity},
};

fn escape(app: &mut MatrixApp, context: &egui::Context) {
    let input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(
            egui::Pos2::ZERO,
            Vec2::new(1280., 800.),
        )),
        events: vec![Event::Key {
            key: Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
        ..Default::default()
    };
    let _ = context.run_ui(input, |ui| app.show(ui));
    let release = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(
            egui::Pos2::ZERO,
            Vec2::new(1280., 800.),
        )),
        events: vec![Event::Key {
            key: Key::Escape,
            physical_key: None,
            pressed: false,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
        ..Default::default()
    };
    let _ = context.run_ui(release, |ui| app.show(ui));
}

#[test]
fn escape_discards_builder_and_setting_previews_without_document_changes() {
    let mut doc = Document::new("escape");
    let rail = Entity::new(
        "rail",
        Geometry::Curve(Curve::Circle {
            center: [0., 0., 0.],
            x: [1., 0., 0.],
            y: [0., 1., 0.],
            radius: 9.,
            domain: [0., std::f64::consts::TAU],
            angle_domain: None,
        }),
        None,
    );
    let rail_id = rail.id;
    doc.entities.insert(rail_id, rail);
    let gem = gem_entity("gem", &GemSpec::round(6., 3.72).unwrap(), None).unwrap();
    let gem_id = gem.id;
    doc.entities.insert(gem_id, gem);
    let mut app = MatrixApp::new(doc).unwrap();
    let ctx = egui::Context::default();
    app.controller.select_objects(&[rail_id]).unwrap();
    app.controller.submit("RailProfile").unwrap();
    assert!(app.controller.is_running());
    escape(&mut app, &ctx);
    assert!(app.controller.builder().is_none());
    assert!(!app.controller.is_running());
    for feature in [HEAD_BUILDER, PRONG_BUILDER] {
        app.controller.select_objects(&[gem_id]).unwrap();
        app.controller
            .dispatch(feature, InvocationOrigin::Menu)
            .unwrap();
        assert!(app.controller.is_running());
        escape(&mut app, &ctx);
        assert!(!app.controller.is_running(), "{feature}");
        assert!(app.controller.head_pending().is_none());
        assert!(app.controller.settings_pending().is_none());
    }
    assert_eq!(app.controller.document().entities.len(), 2);
    assert_eq!(app.controller.coordinator().undo_len(), 0);
}
