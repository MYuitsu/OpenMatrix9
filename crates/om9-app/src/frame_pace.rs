//! Frame pacing for camera motion.
//!
//! The event-driven UI normally waits for input; while a gesture or its settle is in flight the renderer
//! must produce frames at a fixed rate. `FramePacer` schedules the next repaint from the end of the
//! current frame, so a 30 Hz target is stable even when the UI work takes most of the interval.

use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug)]
pub struct FrameTiming {
    pub dt: Duration,
    pub jitter_ms: f64,
    pub late: bool,
}

pub struct FramePacer {
    target: Duration,
    last: Option<Instant>,
    next: Instant,
}

impl FramePacer {
    /// `hz` is clamped to the supported 30..60 range; the environment default is 30 for the stable
    /// first stage.
    pub fn new(hz: f64) -> Self {
        let hz = hz.clamp(30.0, 60.0);
        let target = Duration::from_secs_f64(1.0 / hz);
        let now = Instant::now();
        Self { target, last: None, next: now }
    }

    pub fn from_env() -> Self {
        let hz = std::env::var("OM9_FRAME_HZ")
            .ok()
            .and_then(|value| value.trim().parse::<f64>().ok())
            .unwrap_or(30.0);
        Self::new(hz)
    }

    pub fn target(&self) -> Duration {
        self.target
    }

    pub fn begin(&mut self, now: Instant) -> FrameTiming {
        // An input event can wake the event loop before the deadline; hold the frame until the next
        // slot so the interval is stable instead of bursting.
        if let Some(wait) = self.next.checked_duration_since(now) {
            std::thread::sleep(wait);
        }
        let now = Instant::now();
        let dt = self.last.map(|last| now.saturating_duration_since(last)).unwrap_or(self.target);
        self.last = Some(now);
        self.next = now + self.target;
        let jitter_ms = dt.as_secs_f64() * 1000.0 - self.target.as_secs_f64() * 1000.0;
        FrameTiming { dt, jitter_ms, late: dt > self.target + Duration::from_millis(2) }
    }

    /// Sleeps out the remainder of the current interval; never adds a second full interval.
    pub fn finish(&mut self, ctx: &eframe::egui::Context, now: Instant) {
        let wait = self.next.saturating_duration_since(now);
        ctx.request_repaint_after(wait);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_target_is_limited_to_30_60_hz() {
        assert_eq!(FramePacer::new(10.0).target(), Duration::from_secs_f64(1.0 / 30.0));
        assert_eq!(FramePacer::new(120.0).target(), Duration::from_secs_f64(1.0 / 60.0));
    }

    #[test]
    fn jitter_is_measured_against_the_target() {
        let mut pacer = FramePacer::new(30.0);
        let start = Instant::now();
        let first = pacer.begin(start);
        assert_eq!(first.dt, pacer.target());
        std::thread::sleep(Duration::from_millis(40));
        let second = pacer.begin(Instant::now());
        assert!(second.jitter_ms > 0.0, "{}", second.jitter_ms);
    }
}