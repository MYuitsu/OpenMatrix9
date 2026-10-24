// SPDX-License-Identifier: LGPL-2.1-or-later
//! `OM9-L3-03-02-007`: double-clicking the viewport title toggles maximize/restore.
use eframe::egui::{self, Event, Modifiers, PointerButton, Pos2, Rect, Vec2};
use om9_app::ui::MatrixApp;
use om9_core::model::Document;
use om9_scene::workspace::ViewId;

struct Harness {
    app: MatrixApp,
    ctx: egui::Context,
    time: f64,
}
impl Harness {
    fn new() -> Self {
        let mut h = Self {
            app: MatrixApp::new(Document::new("maximize title")).unwrap(),
            ctx: Default::default(),
            time: 0.,
        };
        h.frame(vec![]);
        h.frame(vec![]);
        h
    }
    fn frame(&mut self, events: Vec<Event>) {
        self.time += 1. / 60.;
        let mut output = self.ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 800.))),
                events,
                time: Some(self.time),
                ..Default::default()
            },
            |ui| self.app.show(ui),
        );
        output.textures_delta.clear();
    }
    fn double_click(&mut self, at: Pos2) {
        for _ in 0..2 {
            self.frame(vec![
                Event::PointerMoved(at),
                Event::PointerButton {
                    pos: at,
                    button: PointerButton::Primary,
                    pressed: true,
                    modifiers: Modifiers::NONE,
                },
            ]);
            self.frame(vec![Event::PointerButton {
                pos: at,
                button: PointerButton::Primary,
                pressed: false,
                modifiers: Modifiers::NONE,
            }]);
        }
    }
}

#[test]
fn double_clicking_a_title_maximizes_and_keeps_every_camera() {
    let mut h = Harness::new();
    let target = ViewId::LookingDown;
    let cameras = h.app.controller.workspace.views.clone();
    let title = h.app.view_rects[&target].min + Vec2::new(20., 9.);
    h.double_click(title);
    assert_eq!(h.app.controller.workspace.maximized, Some(target));
    assert_eq!(h.app.controller.workspace.views, cameras);
}

#[test]
fn double_clicking_the_canvas_does_not_toggle_the_layout() {
    let mut h = Harness::new();
    let canvas = h.app.view_rects[&ViewId::LookingDown].center();
    h.double_click(canvas);
    assert_eq!(h.app.controller.workspace.maximized, None);
}
