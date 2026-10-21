// SPDX-License-Identifier: LGPL-2.1-or-later
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
            app: MatrixApp::new(Document::new("navigation lifecycle")).unwrap(),
            ctx: Default::default(),
            time: 0.,
        };
        h.frame(vec![]);
        h
    }
    fn frame(&mut self, events: Vec<Event>) {
        self.time += 1. / 30.;
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 800.))),
            events,
            time: Some(self.time),
            ..Default::default()
        };
        let mut output = self.ctx.run_ui(input, |ui| self.app.show(ui));
        output.textures_delta.clear();
    }
    fn button(&mut self, pos: Pos2, pressed: bool) {
        self.frame(vec![
            Event::PointerMoved(pos),
            Event::PointerButton {
                pos,
                button: PointerButton::Secondary,
                pressed,
                modifiers: Modifiers::NONE,
            },
        ]);
    }
}
#[test]
fn capture_crosses_other_panes_and_release_has_no_inertia() {
    let mut h = Harness::new();
    h.app.controller.workspace.active = ViewId::LookingDown;
    let start = h.app.view_rects[&ViewId::Perspective].center();
    let untouched = h.app.controller.workspace.views[&ViewId::LookingDown]
        .camera
        .clone();
    h.button(start, true);
    h.frame(vec![Event::PointerMoved(start + Vec2::new(50., 10.))]);
    let first = h.app.controller.workspace.views[&ViewId::Perspective]
        .camera
        .revision;
    let outside = h.app.view_rects[&ViewId::LookingDown].center();
    h.frame(vec![Event::PointerMoved(outside)]);
    assert!(
        h.app.controller.workspace.views[&ViewId::Perspective]
            .camera
            .revision
            > first
    );
    assert_eq!(
        h.app.controller.workspace.views[&ViewId::LookingDown].camera,
        untouched
    );
    h.button(outside, false);
    let stopped = h.app.controller.workspace.views[&ViewId::Perspective]
        .camera
        .clone();
    for _ in 0..10 {
        h.frame(vec![]);
    }
    assert_eq!(
        h.app.controller.workspace.views[&ViewId::Perspective].camera,
        stopped
    );
}
#[test]
fn focus_loss_releases_owner_and_does_not_move_on_return() {
    let mut h = Harness::new();
    let start = h.app.view_rects[&ViewId::Perspective].center();
    h.button(start, true);
    h.frame(vec![Event::PointerMoved(start + Vec2::new(20., 10.))]);
    h.frame(vec![Event::WindowFocused(false)]);
    let stopped = h.app.controller.workspace.views[&ViewId::Perspective]
        .camera
        .clone();
    h.frame(vec![
        Event::WindowFocused(true),
        Event::PointerMoved(start + Vec2::new(80., 30.)),
    ]);
    assert_eq!(
        h.app.controller.workspace.views[&ViewId::Perspective].camera,
        stopped
    );
}
#[test]
fn zoom_settles_with_no_input_and_preserves_screen_anchor() {
    let mut h = Harness::new();
    let id = ViewId::Perspective;
    let rect = h.app.view_rects[&id];
    let cursor = rect.center() + Vec2::new(70., 20.);
    h.frame(vec![Event::PointerMoved(cursor)]);
    let before = h.app.controller.workspace.views[&id].camera.clone();
    let vp = om9_scene::camera::Viewport {
        origin: [rect.min.x as f64, rect.min.y as f64],
        size: [rect.width() as f64, rect.height() as f64],
    };
    let rig = om9_app::camera_rig::CameraRig::from_camera(&before).unwrap();
    let f = rig.orientation.rotate([0., 0., -1.]);
    let ray = before.ray([cursor.x as f64, cursor.y as f64], vp).unwrap();
    let dot = |a: [f64; 3], b: [f64; 3]| (0..3).map(|i| a[i] * b[i]).sum::<f64>();
    let t =
        dot(std::array::from_fn(|i| before.target[i] - ray.origin[i]), f) / dot(ray.direction, f);
    let anchor = std::array::from_fn(|i| ray.origin[i] + t * ray.direction[i]);
    h.frame(vec![Event::MouseWheel {
        unit: egui::MouseWheelUnit::Point,
        delta: Vec2::new(0., 252.),
        modifiers: Modifiers::NONE,
        phase: egui::TouchPhase::Move,
    }]);
    for _ in 0..8 {
        h.frame(vec![]);
        let c = &h.app.controller.workspace.views[&id].camera;
        let point = c.project(anchor, vp).unwrap().unwrap();
        assert!((point.screen[0] - cursor.x as f64).abs() < 0.1);
        assert!((point.screen[1] - cursor.y as f64).abs() < 0.1);
    }
    let stopped = h.app.controller.workspace.views[&id].camera.clone();
    h.frame(vec![]);
    assert_eq!(h.app.controller.workspace.views[&id].camera, stopped);
}
#[test]
fn repeated_egui_layout_pass_does_not_apply_drag_twice() {
    let mut once = Harness::new();
    let mut retry = Harness::new();
    let id = ViewId::Perspective;
    let start = once.app.view_rects[&id].center();
    once.button(start, true);
    retry.button(start, true);
    let event = Event::PointerMoved(start + Vec2::new(24., 7.));
    once.frame(vec![event.clone()]);
    retry.time += 1. / 30.;
    let input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280., 800.))),
        events: vec![event],
        time: Some(retry.time),
        ..Default::default()
    };
    let mut passes = 0;
    let mut output = retry.ctx.run_ui(input, |ui| {
        passes += 1;
        retry.app.show(ui);
        if passes == 1 {
            ui.ctx().request_discard("navigation lifecycle retry");
        }
    });
    output.textures_delta.clear();
    assert!(passes >= 2);
    assert_eq!(
        once.app.controller.workspace.views[&id].camera,
        retry.app.controller.workspace.views[&id].camera
    );
}
#[test]
fn escape_stops_capture_and_later_pointer_motion_cannot_resume_it() {
    let mut h = Harness::new();
    let id = ViewId::Perspective;
    let start = h.app.view_rects[&id].center();
    h.button(start, true);
    h.frame(vec![Event::PointerMoved(start + Vec2::new(10., 3.))]);
    h.frame(vec![Event::Key {
        key: egui::Key::Escape,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    }]);
    let stopped = h.app.controller.workspace.views[&id].camera.clone();
    h.frame(vec![Event::PointerMoved(start + Vec2::new(90., 15.))]);
    assert_eq!(stopped, h.app.controller.workspace.views[&id].camera);
}

#[test]
fn drag_interrupts_zoom_at_the_displayed_pose_and_cancels_unseen_tail() {
    let mut h = Harness::new();
    let id = ViewId::Perspective;
    let start = h.app.view_rects[&id].center();
    h.frame(vec![
        Event::PointerMoved(start),
        Event::MouseWheel {
            unit: egui::MouseWheelUnit::Point,
            delta: Vec2::new(0., 1024.),
            modifiers: Modifiers::NONE,
            phase: egui::TouchPhase::Move,
        },
    ]);
    h.frame(vec![]);
    let displayed = h.app.controller.workspace.views[&id].camera.clone();
    h.button(start, true);
    assert_eq!(displayed, h.app.controller.workspace.views[&id].camera);
    h.frame(vec![Event::PointerMoved(start + Vec2::new(20., 4.))]);
    h.button(start + Vec2::new(20., 4.), false);
    let stopped = h.app.controller.workspace.views[&id].camera.clone();
    for _ in 0..12 {
        h.frame(vec![]);
    }
    assert_eq!(stopped, h.app.controller.workspace.views[&id].camera);
}
#[test]
fn disappearing_owner_and_dpi_layout_change_do_not_leave_capture_stuck() {
    let mut h = Harness::new();
    let id = ViewId::Perspective;
    let start = h.app.view_rects[&id].center();
    h.button(start, true);
    h.frame(vec![Event::PointerMoved(start + Vec2::new(10., 2.))]);
    let stopped = h.app.controller.workspace.views[&id].camera.clone();
    h.app
        .controller
        .workspace
        .toggle_maximize(ViewId::LookingDown)
        .unwrap();
    h.ctx.set_pixels_per_point(1.5);
    h.frame(vec![]);
    h.app
        .controller
        .workspace
        .toggle_maximize(ViewId::LookingDown)
        .unwrap();
    h.frame(vec![Event::PointerMoved(start + Vec2::new(60., 12.))]);
    assert_eq!(stopped, h.app.controller.workspace.views[&id].camera);
}
#[test]
fn high_dpi_panes_remain_inside_the_actual_canvas() {
    let mut h = Harness::new();
    h.ctx.set_zoom_factor(1.5);
    for _ in 0..3 {
        h.time += 1. / 30.;
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1066., 666.))),
            time: Some(h.time),
            ..Default::default()
        };
        let mut output = h.ctx.run_ui(input, |ui| h.app.show(ui));
        output.textures_delta.clear();
        let screen = h.ctx.content_rect();
        assert_eq!(h.app.view_rects.len(), 4);
        for rect in h.app.view_rects.values() {
            assert!(
                rect.width() <= screen.width() && rect.height() <= screen.height(),
                "{rect:?} extends beyond {screen:?}"
            );
        }
    }
}
