// SPDX-License-Identifier: LGPL-2.1-or-later
//! `OM9-L3-03-09-*`: F6 context classification, custom-menu persistence and provider gates.
use om9_app::{f6::ContextKind, ui::MatrixApp};
use om9_core::model::Document;

#[test]
fn f6_uses_the_selected_class_and_empty_general_context() {
    let mut app = MatrixApp::new(Document::new("f6 context")).unwrap();
    assert_eq!(app.f6_context().kind, ContextKind::General);
    assert!(app.f6_context().enabled_count() >= 1);

    app.controller.submit("Line").unwrap();
    app.controller.submit("0,0,0").unwrap();
    app.controller.submit("10,0,0").unwrap();
    let line = *app.controller.document().entities.keys().next().unwrap();
    app.controller.select_objects(&[line]).unwrap();
    assert_eq!(app.f6_context().kind, ContextKind::Curve);
    assert_eq!(app.f6_context().enabled_count(), 0);
    assert!(
        app.f6_context()
            .actions
            .iter()
            .all(|action| action.availability.reason().is_some())
    );
}

#[test]
fn a_custom_menu_persists_in_the_project_with_stable_order_and_stale_disable() {
    let mut app = MatrixApp::new(Document::new("f6 custom")).unwrap();
    app.f6_custom.command_ids = vec![
        om9_app::session::POINT.to_owned(),
        "DeletedCommand".to_owned(),
        om9_app::session::LINE.to_owned(),
    ];
    app.stage_project_state().unwrap();
    app.f6_custom.command_ids.clear();
    app.reload_project_state().unwrap();
    assert_eq!(
        app.f6_custom.command_ids,
        vec![
            om9_app::session::POINT.to_owned(),
            "DeletedCommand".to_owned(),
            om9_app::session::LINE.to_owned(),
        ]
    );
    let context = app.f6_custom.context();
    assert_eq!(context.kind, ContextKind::Custom);
    assert!(context.actions[0].availability.enabled());
    assert!(context.actions[1].availability.reason().is_some());
    assert!(context.actions[2].availability.enabled());
}
