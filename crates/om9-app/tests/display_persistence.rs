// SPDX-License-Identifier: LGPL-2.1-or-later
//! `OM9-L3-03-05-006`: display preset saved in the project archive and restored on reload.
use om9_app::{display::DisplayPreferences, ui::MatrixApp};
use om9_core::model::Document;

#[test]
fn a_display_preset_is_embedded_and_restored_with_the_project() {
    let mut app = MatrixApp::new(Document::new("display persistence")).unwrap();
    let preset = DisplayPreferences {
        wireframe: false,
        show_edges: false,
        hide_cutters: true,
        ..DisplayPreferences::default()
    };
    app.display_preferences = preset.clone();
    app.stage_project_state().expect("stage display state");
    assert!(
        app.controller
            .embedded_display_state()
            .expect("read resource")
            .is_some()
    );

    app.display_preferences = DisplayPreferences::default();
    app.reload_project_state().expect("reload project");
    assert_eq!(app.display_preferences, preset);
    assert!(!app.display_preferences.wireframe);
    assert!(!app.display_preferences.show_edges);
    assert!(app.display_preferences.hide_cutters);
}

#[test]
fn a_corrupt_display_preset_falls_back_without_losing_the_project() {
    let mut app = MatrixApp::new(Document::new("bad display")).unwrap();
    app.controller
        .store_display_state(b"{not a preset")
        .expect("store bytes");
    app.reload_project_state().expect("reload project");
    assert_eq!(app.display_preferences, DisplayPreferences::default());
    assert_eq!(app.controller.document().name, "bad display");
}
