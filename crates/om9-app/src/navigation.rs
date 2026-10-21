// SPDX-License-Identifier: LGPL-2.1-or-later
//! Ephemeral navigation state. Workspace Camera remains the persisted pose.
use crate::camera_rig::CameraRig;
use om9_scene::{
    camera::{Camera, Projection, Viewport},
    workspace::ViewId,
};
use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavigationPhase {
    Idle,
    Dragging,
    Settling,
    Refining,
}
impl NavigationPhase {
    pub fn name(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Dragging => "dragging",
            Self::Settling => "settling",
            Self::Refining => "refining",
        }
    }
}

/// Pure scalar analytic response; no numerical integration or frame-rate-dependent wheel drain.
#[derive(Clone, Debug)]
pub struct ZoomResponse {
    base: CameraRig,
    anchor: [f64; 3],
    x: f64,
    velocity: f64,
    target: f64,
    sampled: Instant,
    last_input: Instant,
    speed: f64,
    last_direction: f64,
}
impl ZoomResponse {
    pub fn new(base: CameraRig, anchor: [f64; 3], now: Instant, speed: f64) -> Self {
        Self {
            base,
            anchor,
            x: 0.,
            velocity: 0.,
            target: 0.,
            sampled: now,
            last_input: now,
            speed: finite_speed(speed),
            last_direction: 0.,
        }
    }
    pub fn add_points(&mut self, points: f64, now: Instant) {
        if !points.is_finite() {
            return;
        }
        self.advance(now);
        if self.last_direction != 0. && self.last_direction != points.signum() {
            self.velocity = 0.;
        }
        self.last_direction = points.signum();
        let initial = self.base.ortho_span.unwrap_or(self.base.distance);
        self.target = (self.target + points / 40. * 0.9_f64.ln())
            .clamp((1e-6 / initial).ln(), (1e15 / initial).ln());
        let remaining = self.target - self.x;
        // A reversal must not continue travelling away from the user's new target.
        self.velocity = if remaining >= 0. {
            self.velocity.clamp(0., 90. * self.speed * remaining)
        } else {
            self.velocity.clamp(90. * self.speed * remaining, 0.)
        };
        self.last_input = now;
    }
    fn advance(&mut self, now: Instant) {
        let dt = now.saturating_duration_since(self.sampled).as_secs_f64();
        let omega = 90. * self.speed;
        let error = self.x - self.target;
        let b = self.velocity + omega * error;
        let decay = (-omega * dt).exp();
        self.x = self.target + (error + b * dt) * decay;
        self.velocity = (self.velocity - omega * b * dt) * decay;
        self.sampled = now;
    }
    pub fn sample(&mut self, now: Instant) -> (CameraRig, bool) {
        self.advance(now);
        let done =
            now.saturating_duration_since(self.last_input).as_secs_f64() >= 0.18 / self.speed;
        if done {
            self.x = self.target;
            self.velocity = 0.;
        }
        let scale = self.x.exp();
        let mut pose = self.base;
        pose.zoom_toward(self.anchor, scale);
        (pose, done)
    }
    pub fn target_scale(&self) -> f64 {
        self.target.exp()
    }
}
pub fn finite_speed(value: f64) -> f64 {
    if value.is_finite() {
        value.clamp(0.25, 4.)
    } else {
        1.
    }
}

pub struct ViewNavigationState {
    pub phase: NavigationPhase,
    pub generation: u64,
    pub last_input: Instant,
    zoom: Option<ZoomResponse>,
    expected_camera: Camera,
}
#[derive(Clone, Copy, Debug)]
pub struct GestureSession {
    pub view: ViewId,
    pub button: eframe::egui::PointerButton,
    pub last: eframe::egui::Pos2,
}
#[derive(Default)]
pub struct NavigationController {
    pub views: BTreeMap<ViewId, ViewNavigationState>,
    pub gesture: Option<GestureSession>,
    pub input_seq: u64,
}
impl NavigationController {
    fn state(&mut self, id: ViewId, camera: &Camera, now: Instant) -> &mut ViewNavigationState {
        let state = self.views.entry(id).or_insert_with(|| ViewNavigationState {
            phase: NavigationPhase::Idle,
            generation: 0,
            last_input: now,
            zoom: None,
            expected_camera: camera.clone(),
        });
        if state.expected_camera != *camera {
            state.zoom = None;
            state.phase = NavigationPhase::Refining;
            state.generation = state.generation.wrapping_add(1);
            state.expected_camera = camera.clone();
        }
        state
    }
    pub fn sync(&mut self, id: ViewId, camera: &Camera, now: Instant) {
        self.state(id, camera, now);
    }
    pub fn cancel(&mut self) {
        self.gesture = None;
        for s in self.views.values_mut() {
            s.zoom = None;
            if s.phase != NavigationPhase::Idle {
                s.phase = NavigationPhase::Refining;
                s.generation = s.generation.wrapping_add(1);
            }
        }
    }
    pub fn begin_drag(
        &mut self,
        id: ViewId,
        button: eframe::egui::PointerButton,
        pos: eframe::egui::Pos2,
        camera: &Camera,
        now: Instant,
    ) {
        if self.gesture.is_some() {
            return;
        }
        let state = self.state(id, camera, now);
        state.zoom = None;
        state.phase = NavigationPhase::Dragging;
        state.last_input = now;
        state.generation = state.generation.wrapping_add(1);
        self.gesture = Some(GestureSession {
            view: id,
            button,
            last: pos,
        });
    }
    pub fn end_drag(&mut self) {
        if let Some(g) = self.gesture.take() {
            if let Some(s) = self.views.get_mut(&g.view) {
                s.phase = NavigationPhase::Refining;
            }
        }
    }
    pub fn drag(
        &mut self,
        camera: &Camera,
        viewport: Viewport,
        delta: [f64; 2],
        pan: bool,
        now: Instant,
    ) -> Option<Camera> {
        if delta == [0., 0.] || delta.iter().any(|x| !x.is_finite()) {
            return None;
        }
        let id = self.gesture?.view;
        let s = self.state(id, camera, now);
        let mut rig = CameraRig::from_camera(camera)?;
        if pan {
            let span = match camera.projection {
                Projection::Orthographic { vertical_span_mm } => vertical_span_mm,
                Projection::Perspective { vertical_fov_rad } => {
                    2. * rig.distance * (vertical_fov_rad * 0.5).tan()
                }
            };
            let per_pixel = span / viewport.size[1].max(1.);
            let right = rig.orientation.rotate([1., 0., 0.]);
            let up = rig.orientation.rotate([0., 1., 0.]);
            rig.pan(std::array::from_fn(|a| {
                per_pixel * (-delta[0] * right[a] + delta[1] * up[a])
            }));
        } else {
            rig.orbit_turntable(delta[0] * 0.01, -delta[1] * 0.01);
        }
        let out = rig.to_camera(camera)?;
        s.expected_camera = out.clone();
        s.phase = NavigationPhase::Dragging;
        s.last_input = now;
        s.generation = s.generation.wrapping_add(1);
        Some(out)
    }
    pub fn wheel(
        &mut self,
        id: ViewId,
        camera: &Camera,
        viewport: Viewport,
        cursor: [f64; 2],
        points: f64,
        now: Instant,
        speed: f64,
    ) {
        self.wheel_displayed(id, camera, camera, viewport, cursor, points, now, speed);
    }
    pub fn wheel_displayed(
        &mut self,
        id: ViewId,
        camera: &Camera,
        displayed: &Camera,
        viewport: Viewport,
        cursor: [f64; 2],
        points: f64,
        now: Instant,
        speed: f64,
    ) {
        if !points.is_finite() || points == 0. || self.gesture.is_some() {
            return;
        }
        let s = self.state(id, camera, now);
        let Some(base) = CameraRig::from_camera(displayed) else {
            return;
        };
        let forward = base.orientation.rotate([0., 0., -1.]);
        let Some(anchor) = displayed.ray(cursor, viewport).ok().and_then(|r| {
            let den = dot(r.direction, forward);
            if den.abs() < 1e-12 {
                return None;
            }
            let t = dot(
                std::array::from_fn(|a| base.pivot[a] - r.origin[a]),
                forward,
            ) / den;
            (t > 0.).then(|| std::array::from_fn(|a| r.origin[a] + r.direction[a] * t))
        }) else {
            return;
        };
        // Same screen anchor under a moving zoom maps to the same world point. A changed anchor
        // starts from the displayed camera, cancelling only the unseen remainder of the old segment.
        let replace = s.zoom.as_ref().is_none_or(|z| {
            (0..3).any(|a| (z.anchor[a] - anchor[a]).abs() > 1e-8 * base.distance.max(1.))
        });
        if replace {
            s.zoom = Some(ZoomResponse::new(base, anchor, now, speed));
        }
        if let Some(z) = s.zoom.as_mut() {
            z.add_points(points, now);
        }
        s.last_input = now;
        s.phase = NavigationPhase::Settling;
        s.generation = s.generation.wrapping_add(1);
    }
    pub fn tick(&mut self, id: ViewId, camera: &Camera, now: Instant) -> Option<Camera> {
        let s = self.state(id, camera, now);
        if s.phase == NavigationPhase::Dragging
            && now.saturating_duration_since(s.last_input) >= Duration::from_millis(80)
        {
            s.phase = NavigationPhase::Refining;
        }
        let z = s.zoom.as_mut()?;
        let (pose, done) = z.sample(now);
        let changed = (0..3).any(|a| (pose.pivot[a] - camera.target[a]).abs() > 1e-14)
            || (CameraRig::from_camera(camera)?
                .ortho_span
                .unwrap_or(CameraRig::from_camera(camera)?.distance)
                - pose.ortho_span.unwrap_or(pose.distance))
            .abs()
                > 1e-14;
        if done {
            s.zoom = None;
            s.phase = NavigationPhase::Refining;
        }
        if !changed {
            return None;
        }
        let out = pose.to_camera(camera)?;
        s.expected_camera = out.clone();
        Some(out)
    }
    pub fn moving(&self, id: ViewId) -> bool {
        self.views.get(&id).is_some_and(|s| {
            matches!(
                s.phase,
                NavigationPhase::Dragging | NavigationPhase::Settling
            )
        })
    }
    pub fn generation(&self, id: ViewId) -> u64 {
        self.views.get(&id).map_or(0, |s| s.generation)
    }
    pub fn completed(&mut self, id: ViewId, generation: u64) {
        if let Some(s) = self.views.get_mut(&id) {
            if s.generation == generation && s.phase == NavigationPhase::Refining {
                s.phase = NavigationPhase::Idle;
            }
        }
    }
    pub fn needs_tick(&self) -> bool {
        self.views.values().any(|s| {
            matches!(
                s.phase,
                NavigationPhase::Dragging | NavigationPhase::Settling | NavigationPhase::Refining
            )
        })
    }
}
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    (0..3).map(|i| a[i] * b[i]).sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn camera() -> Camera {
        Camera::perspective([0., -100., 20.], [0.; 3], [0., 0., 1.], 0.6).unwrap()
    }
    #[test]
    fn wheel_conserves_burst_and_finishes_exactly() {
        let now = Instant::now();
        let c = camera();
        let base = CameraRig::from_camera(&c).unwrap();
        let mut z = ZoomResponse::new(base, base.pivot, now, 1.);
        z.add_points(1024., now);
        z.add_points(-252., now + Duration::from_millis(10));
        assert!((z.target_scale() - 0.9_f64.powf((1024. - 252.) / 40.)).abs() < 1e-12);
        let (p, done) = z.sample(now + Duration::from_millis(190));
        assert!(done);
        assert!((p.distance - base.distance * z.target_scale()).abs() < 1e-12);
    }
    #[test]
    fn every_zoom_sample_preserves_anchor() {
        let now = Instant::now();
        let c = camera();
        let base = CameraRig::from_camera(&c).unwrap();
        let right = base.orientation.rotate([1., 0., 0.]);
        let q = std::array::from_fn(|a| base.pivot[a] + right[a] * 12.);
        let vp = Viewport {
            origin: [0.; 2],
            size: [800., 600.],
        };
        let before = c.project(q, vp).unwrap().unwrap().screen;
        let mut z = ZoomResponse::new(base, q, now, 1.);
        z.add_points(252., now);
        for ms in 0..=180 {
            let (p, _) = z.sample(now + Duration::from_millis(ms));
            let screen = p
                .to_camera(&c)
                .unwrap()
                .project(q, vp)
                .unwrap()
                .unwrap()
                .screen;
            assert!((screen[0] - before[0]).abs() < 1e-7);
            assert!((screen[1] - before[1]).abs() < 1e-7);
        }
    }
    #[test]
    fn reversal_moves_towards_new_target_without_overshoot() {
        let now = Instant::now();
        let base = CameraRig::from_camera(&camera()).unwrap();
        let mut z = ZoomResponse::new(base, base.pivot, now, 1.);
        z.add_points(252., now);
        z.sample(now + Duration::from_millis(30));
        z.add_points(-504., now + Duration::from_millis(30));
        let mut previous = z.x;
        for ms in 31..=210 {
            z.sample(now + Duration::from_millis(ms));
            assert!(z.x >= previous - 1e-12);
            assert!(z.x <= z.target + 1e-12);
            previous = z.x;
        }
    }
    #[test]
    fn fractional_wheel_partition_and_finite_speed_are_stable() {
        let now = Instant::now();
        let base = CameraRig::from_camera(&camera()).unwrap();
        let mut whole = ZoomResponse::new(base, base.pivot, now, f64::NAN);
        let mut parts = ZoomResponse::new(base, base.pivot, now, 1.);
        whole.add_points(12.75, now);
        for _ in 0..51 {
            parts.add_points(0.25, now);
        }
        assert!((whole.target_scale() - parts.target_scale()).abs() < 1e-13);
        for ms in [1, 7, 19, 63, 180] {
            let t = now + Duration::from_millis(ms);
            let (a, _) = whole.sample(t);
            let (b, _) = parts.sample(t);
            assert!((a.distance - b.distance).abs() < 1e-11);
        }
        assert_eq!(finite_speed(f64::INFINITY), 1.);
    }
    #[test]
    fn held_still_refines_but_preserves_capture_and_resume_is_direct() {
        let now = Instant::now();
        let c = camera();
        let id = ViewId::Perspective;
        let mut nav = NavigationController::default();
        nav.begin_drag(
            id,
            eframe::egui::PointerButton::Secondary,
            eframe::egui::Pos2::ZERO,
            &c,
            now,
        );
        assert!(nav.tick(id, &c, now + Duration::from_millis(80)).is_none());
        assert!(!nav.moving(id));
        assert!(nav.gesture.is_some());
        let generation = nav.generation(id);
        nav.completed(id, generation.wrapping_sub(1));
        assert_eq!(nav.views[&id].phase, NavigationPhase::Refining);
        nav.completed(id, generation);
        assert_eq!(nav.views[&id].phase, NavigationPhase::Idle);
        let vp = Viewport {
            origin: [0.; 2],
            size: [800., 600.],
        };
        assert!(
            nav.drag(&c, vp, [1., 2.], false, now + Duration::from_secs(1))
                .is_some()
        );
        assert!(nav.moving(id));
    }
    #[test]
    fn settle_a_and_drag_b_have_independent_owners_and_external_camera_cancels_zoom() {
        let now = Instant::now();
        let c = camera();
        let vp = Viewport {
            origin: [0.; 2],
            size: [800., 600.],
        };
        let mut nav = NavigationController::default();
        nav.wheel(ViewId::Perspective, &c, vp, [400., 300.], 252., now, 1.);
        nav.begin_drag(
            ViewId::LookingDown,
            eframe::egui::PointerButton::Middle,
            eframe::egui::Pos2::ZERO,
            &c,
            now,
        );
        assert!(
            nav.tick(ViewId::Perspective, &c, now + Duration::from_millis(30))
                .is_some()
        );
        assert_eq!(nav.gesture.unwrap().view, ViewId::LookingDown);
        let mut home = c.clone();
        home.revision += 100;
        nav.sync(ViewId::Perspective, &home, now);
        assert!(
            nav.tick(ViewId::Perspective, &home, now + Duration::from_millis(60))
                .is_none()
        );
    }
    #[test]
    fn orthographic_clamp_and_deep_zoom_preserve_anchor() {
        let now = Instant::now();
        let mut c = camera();
        c.projection = Projection::Orthographic {
            vertical_span_mm: 100.,
        };
        let base = CameraRig::from_camera(&c).unwrap();
        let anchor = [8., 0., 0.];
        let vp = Viewport {
            origin: [0.; 2],
            size: [800., 600.],
        };
        let before = c.project(anchor, vp).unwrap().unwrap().screen;
        let mut z = ZoomResponse::new(base, anchor, now, 1.);
        z.add_points(40. * (0.2_f64 / 100.).ln() / 0.9_f64.ln(), now);
        for ms in 1..=180 {
            let (pose, _) = z.sample(now + Duration::from_millis(ms));
            let p = pose
                .to_camera(&c)
                .unwrap()
                .project(anchor, vp)
                .unwrap()
                .unwrap()
                .screen;
            assert!((p[0] - before[0]).abs() < 1e-6);
            assert!((p[1] - before[1]).abs() < 1e-6);
        }
        let (pose, done) = z.sample(now + Duration::from_millis(180));
        assert!(done);
        assert!((pose.ortho_span.unwrap() - 0.2).abs() < 1e-10);
        let mut clamped = pose;
        clamped.zoom_toward(anchor, 1e-99);
        let first = clamped;
        clamped.zoom_toward(anchor, 1e-99);
        assert_eq!(clamped.pivot, first.pivot);
    }
}
