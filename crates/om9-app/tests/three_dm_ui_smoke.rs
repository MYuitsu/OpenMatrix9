// SPDX-License-Identifier: LGPL-2.1-or-later
//! Four-view startup smoke for a document produced by the editable 3DM import.
//! Selection remains a manual step; this test proves the imported document reaches
//! the real egui/MatrixApp render path without wiping its entities or views.
#![cfg(all(windows, feature = "occt-brep"))]

use eframe::egui;
use om9_app::ui::MatrixApp;
use om9_core::{budget::Limits, geometry::Geometry, model::Document};

fn frame(app: &mut MatrixApp, ctx: &egui::Context, physical: egui::Vec2, ppp: f32) {
    ctx.set_pixels_per_point(ppp);
    let mut output = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, physical / ppp)),
            ..Default::default()
        },
        |ui| app.show(ui),
    );
    output.textures_delta.clear();
}

#[test]
#[ignore = "requires OM9_THREE_DM_OM9 from an editable Rhino 5 import"]
fn imported_editable_document_reaches_four_views() {
    let path = std::env::var_os("OM9_THREE_DM_OM9").expect("OM9_THREE_DM_OM9");
    let mut app = MatrixApp::from_file(std::path::Path::new(&path)).expect("open imported .om9");
    let ctx = egui::Context::default();
    for _ in 0..4 {
        frame(&mut app, &ctx, egui::Vec2::new(1280., 720.), 1.);
    }
    assert_eq!(app.view_rects.len(), 4);
    let document: &Document = app.controller.document();
    assert_eq!(document.layers.len(), 34);
    assert_eq!(document.entities.len(), 43);
    assert_eq!(
        document
            .entities
            .values()
            .filter(|entity| matches!(entity.geometry, Geometry::Brep(_)))
            .count(),
        29
    );
    assert!(document.validate(&Limits::default()).is_ok());
}
