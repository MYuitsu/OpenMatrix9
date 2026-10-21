// SPDX-License-Identifier: LGPL-2.1-or-later
//! Native-window diagnostic input replay. Goes through the same raw-event router as mouse input.
use eframe::egui::{self, Event, Modifiers, PointerButton, Pos2, Vec2};
use std::time::{Duration, Instant};
pub struct NavigationAudit {
    step: u32,
    next: Option<Instant>,
    total: u32,
    pub finished: bool,
    pub settling: bool,
    capture: bool,
}
impl NavigationAudit {
    pub fn from_env() -> Option<Self> {
        let total = std::env::var("OM9_NAVIGATION_AUDIT")
            .ok()?
            .parse::<u32>()
            .ok()?
            .clamp(30, 3600);
        Some(Self {
            step: 0,
            next: None,
            total,
            finished: false,
            settling: false,
            capture: false,
        })
    }
    pub fn input(
        &mut self,
        ctx: &egui::Context,
        raw: &mut egui::RawInput,
        center: Option<Pos2>,
        ready: bool,
        hz: u32,
    ) {
        self.input_at(ctx, raw, center, ready, hz, Instant::now());
    }
    fn input_at(
        &mut self,
        ctx: &egui::Context,
        raw: &mut egui::RawInput,
        center: Option<Pos2>,
        ready: bool,
        hz: u32,
        now: Instant,
    ) {
        if self.finished {
            return;
        }
        if self.capture {
            if let Some(image) = raw.events.iter().find_map(|e| {
                if let Event::Screenshot { image, .. } = e {
                    Some(image)
                } else {
                    None
                }
            }) {
                let pixels: Vec<u8> = image.pixels.iter().flat_map(|c| c.to_array()).collect();
                if let Some(path) = std::env::var_os("OM9_NAVIGATION_AUDIT_SCREENSHOT") {
                    let _ = image::save_buffer(
                        path,
                        &pixels,
                        image.size[0] as u32,
                        image.size[1] as u32,
                        image::ColorType::Rgba8,
                    );
                }
                self.finished = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            ctx.request_repaint_after(Duration::from_millis(16));
            return;
        }
        raw.focused = true;
        let period = Duration::from_secs_f64(1. / hz as f64);
        ctx.request_repaint_after(period);
        let Some(center) = center else {
            return;
        };
        if self.step == 0 && !ready {
            return;
        }
        if self.settling {
            if ready {
                if std::env::var_os("OM9_NAVIGATION_AUDIT_SCREENSHOT").is_some() {
                    self.capture = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
                } else {
                    self.finished = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
            return;
        }
        let deadline = self.next.unwrap_or(now);
        // Native callbacks may arrive fractionally before a vsync. Keep an absolute cadence,
        // with the same dispatch lead as the window scheduler, instead of restarting a relative
        // timer and silently reducing a nominal 60 Hz input replay to 30 Hz.
        if now + Duration::from_millis(1) < deadline {
            return;
        }
        let next = deadline + period;
        self.next = Some(if next > now { next } else { now + period });
        let segment = self.total / 3;
        let button = |pos, button, pressed| Event::PointerButton {
            pos,
            button,
            pressed,
            modifiers: Modifiers::NONE,
        };
        match self.step {
            0 => {
                raw.events.push(Event::PointerMoved(center));
                raw.events
                    .push(button(center, PointerButton::Secondary, true));
            }
            s if s < segment => {
                let t = s as f32 * 0.13;
                raw.events.push(Event::PointerMoved(
                    center + Vec2::new(t.sin() * 65., t.cos() * 40.),
                ));
            }
            s if s == segment => raw
                .events
                .push(button(center, PointerButton::Secondary, false)),
            s if s == segment + 1 => {
                raw.events.push(Event::PointerMoved(center));
                raw.events.push(button(center, PointerButton::Middle, true));
            }
            s if s < segment * 2 => {
                let t = (s - segment) as f32 * 0.13;
                raw.events.push(Event::PointerMoved(
                    center + Vec2::new(t.sin() * 30., t.cos() * 20.),
                ));
            }
            s if s == segment * 2 => {
                raw.events
                    .push(button(center, PointerButton::Middle, false));
                raw.events
                    .push(Event::PointerMoved(center + Vec2::new(40., 20.)));
            }
            s if s < self.total && s % 4 == 0 => raw.events.push(Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: Vec2::new(0., 40.),
                modifiers: Modifiers::NONE,
                phase: egui::TouchPhase::Move,
            }),
            _ => {}
        }
        self.step += 1;
        if self.step >= self.total {
            self.settling = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_replay_preserves_cadence_on_fractionally_early_callbacks() {
        for hz in [30, 60] {
            let mut audit = NavigationAudit {
                step: 0,
                next: None,
                total: 360,
                finished: false,
                settling: false,
                capture: false,
            };
            let ctx = egui::Context::default();
            let origin = Instant::now();
            for slot in 0..120 {
                let now = origin + Duration::from_secs_f64(slot as f64 / hz as f64);
                let now = if slot == 0 {
                    now
                } else {
                    now - Duration::from_micros(300)
                };
                let mut raw = egui::RawInput::default();
                audit.input_at(&ctx, &mut raw, Some(egui::pos2(200., 200.)), true, hz, now);
                assert!(
                    !raw.events.is_empty(),
                    "{hz} Hz slot {slot} lost its raw input"
                );
                assert_eq!(audit.step, slot + 1);
                let mut duplicate = egui::RawInput::default();
                audit.input_at(
                    &ctx,
                    &mut duplicate,
                    Some(egui::pos2(200., 200.)),
                    true,
                    hz,
                    now,
                );
                assert!(
                    duplicate.events.is_empty(),
                    "one event batch per replay slot"
                );
            }
            let mut late = egui::RawInput::default();
            audit.input_at(
                &ctx,
                &mut late,
                Some(egui::pos2(200., 200.)),
                true,
                hz,
                origin + Duration::from_secs(20),
            );
            assert_eq!(
                audit.step, 121,
                "late replay must skip slots instead of catching up"
            );
        }
    }
}
