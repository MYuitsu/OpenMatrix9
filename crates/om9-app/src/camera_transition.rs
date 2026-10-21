//! Fixed-duration camera transitions for wheel zoom and view changes.
//!
//! Unlike a first-order spring, a transition has a known start, end and duration. Sampling by wall
//! clock makes a late frame land at the correct pose and guarantees the final frame equals the target.

use crate::camera_rig::CameraRig;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug)]
pub struct Ease {
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
}

impl Ease {
    pub const STANDARD: Self = Self { x1: 0.25, y1: 0.1, x2: 0.25, y2: 1.0 };

    fn bezier(a: f64, b: f64, t: f64) -> f64 {
        let u = 1.0 - t;
        3.0 * u * u * t * a + 3.0 * u * t * t * b + t * t * t
    }

    /// Solves x(t) = progress, then returns y(t). Bisection is deterministic and ample for a 30/60 Hz
    /// sample; it avoids a Newton iteration with a pathological derivative at the endpoints.
    pub fn sample(self, progress: f64) -> f64 {
        let progress = progress.clamp(0.0, 1.0);
        if progress <= 0.0 {
            return 0.0;
        }
        if progress >= 1.0 {
            return 1.0;
        }
        let mut lo = 0.0;
        let mut hi = 1.0;
        for _ in 0..24 {
            let mid = 0.5 * (lo + hi);
            if Self::bezier(self.x1, self.x2, mid) < progress {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        Self::bezier(self.y1, self.y2, 0.5 * (lo + hi)).clamp(0.0, 1.0)
    }
}

pub struct CameraTransition {
    from: CameraRig,
    to: CameraRig,
    started: Instant,
    duration: Duration,
    curve: Ease,
}

impl CameraTransition {
    pub fn new(from: CameraRig, to: CameraRig, now: Instant, duration: Duration) -> Self {
        Self {
            from,
            to,
            started: now,
            duration: duration.max(Duration::from_millis(1)),
            curve: Ease::STANDARD,
        }
    }

    pub fn target(&self) -> CameraRig {
        self.to
    }

    pub fn from(&self) -> CameraRig {
        self.from
    }

    pub fn progress(&self, now: Instant) -> f64 {
        let elapsed = now.saturating_duration_since(self.started).as_secs_f64();
        (elapsed / self.duration.as_secs_f64()).clamp(0.0, 1.0)
    }

    pub fn sample(&self, now: Instant) -> (CameraRig, bool) {
        let progress = self.progress(now);
        if progress >= 1.0 {
            return (self.to, true);
        }
        let eased = self.curve.sample(progress);
        let orientation = self.from.orientation.slerp(self.to.orientation, eased);
        let mut pivot = [0.0; 3];
        for axis in 0..3 {
            pivot[axis] = self.from.pivot[axis]
                + (self.to.pivot[axis] - self.from.pivot[axis]) * eased;
        }
        let distance = log_lerp(self.from.distance, self.to.distance, eased);
        let ortho_span = match (self.from.ortho_span, self.to.ortho_span) {
            (Some(from), Some(to)) => Some(log_lerp(from, to, eased)),
            _ => self.to.ortho_span,
        };
        (
            CameraRig { pivot, orientation, distance, ortho_span },
            false,
        )
    }

    /// Starts a new transition from the currently displayed pose, keeping at least `min_remaining`.
    pub fn retarget(&mut self, to: CameraRig, now: Instant, min_remaining: Duration) {
        let (current, _) = self.sample(now);
        let elapsed = now.saturating_duration_since(self.started);
        let remaining = self.duration.saturating_sub(elapsed).max(min_remaining);
        self.from = current;
        self.to = to;
        self.started = now;
        self.duration = remaining;
    }
}

fn log_lerp(from: f64, to: f64, t: f64) -> f64 {
    if from <= 0.0 || to <= 0.0 {
        return from + (to - from) * t;
    }
    (from.ln() + (to.ln() - from.ln()) * t).exp()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::camera_rig::CameraRig;
    use om9_scene::camera::Camera;

    fn camera() -> Camera {
        Camera::perspective([0.0, -10.0, 2.0], [0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 0.6).unwrap()
    }

    fn rig(camera: &Camera) -> CameraRig {
        CameraRig::from_camera(camera).unwrap()
    }

    #[test]
    fn endpoints_are_exact() {
        let camera = camera();
        let from = rig(&camera);
        let mut to = from;
        to.zoom(0.5);
        to.pan([1.0, 2.0, 0.0]);
        let now = Instant::now();
        let transition = CameraTransition::new(from, to, now, Duration::from_millis(140));
        let (at_start, finished) = transition.sample(now);
        assert!(!finished);
        assert!((at_start.distance - from.distance).abs() < 1e-12);
        let (at_end, finished) = transition.sample(now + Duration::from_millis(140));
        assert!(finished);
        assert!((at_end.distance - to.distance).abs() < 1e-12);
        assert_eq!(at_end.pivot, to.pivot);
    }

    #[test]
    fn ease_is_monotonic_and_bounded() {
        let mut previous = 0.0;
        for step in 0..=100 {
            let value = Ease::STANDARD.sample(step as f64 / 100.0);
            assert!((0.0..=1.0).contains(&value));
            assert!(value >= previous - 1e-12);
            previous = value;
        }
    }

    #[test]
    fn zoom_is_log_interpolated() {
        let camera = camera();
        let from = rig(&camera);
        let mut to = from;
        to.zoom(0.25);
        let now = Instant::now();
        let transition = CameraTransition::new(from, to, now, Duration::from_millis(140));
        let (mid, _) = transition.sample(now + Duration::from_millis(70));
        assert!(mid.distance < from.distance);
        assert!(mid.distance > to.distance);
    }

    #[test]
    fn retarget_continues_from_the_displayed_pose() {
        let camera = camera();
        let from = rig(&camera);
        let mut first = from;
        first.pan([1.0, 0.0, 0.0]);
        let now = Instant::now();
        let mut transition =
            CameraTransition::new(from, first, now, Duration::from_millis(140));
        let sample_time = now + Duration::from_millis(70);
        let (displayed, _) = transition.sample(sample_time);
        let mut next = displayed;
        next.pan([2.0, 0.0, 0.0]);
        transition.retarget(next, sample_time, Duration::from_millis(80));
        let (after, _) = transition.sample(sample_time);
        assert!((after.pivot[0] - displayed.pivot[0]).abs() < 1e-12);
    }
}
