// SPDX-License-Identifier: LGPL-2.1-or-later
//! One nonblocking window clock. Input wakes the UI independently of heavy scene draws.
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PreviewQuality {
    #[default]
    Q0,
    Q1,
    Q2,
    Q3,
}
impl PreviewQuality {
    pub fn scale(self) -> f32 {
        match self {
            Self::Q0 => 1.,
            Self::Q1 => 0.85,
            Self::Q2 => 0.7,
            Self::Q3 => 0.5,
        }
    }
    pub fn samples(self) -> u32 {
        match self {
            Self::Q0 | Self::Q1 => 4,
            _ => 1,
        }
    }
    pub fn pixels(self) -> f64 {
        match self {
            Self::Q0 => 0.5,
            Self::Q1 => 1.,
            Self::Q2 => 2.,
            Self::Q3 => 4.,
        }
    }
    pub fn index(self) -> u8 {
        match self {
            Self::Q0 => 0,
            Self::Q1 => 1,
            Self::Q2 => 2,
            Self::Q3 => 3,
        }
    }
    fn worse(self) -> Self {
        match self {
            Self::Q0 => Self::Q1,
            Self::Q1 => Self::Q2,
            _ => Self::Q3,
        }
    }
    fn better(self) -> Self {
        match self {
            Self::Q3 => Self::Q2,
            Self::Q2 => Self::Q1,
            _ => Self::Q0,
        }
    }
}
pub struct WindowFrameScheduler {
    hz: u32,
    fixed: bool,
    pub quality: PreviewQuality,
    next: Option<Instant>,
    last: Option<Instant>,
    pub interval: Option<Duration>,
    over: u32,
    under: u32,
    history: VecDeque<(Instant, f64, bool)>,
    switched: Instant,
    recovery: Option<Instant>,
}
impl WindowFrameScheduler {
    pub fn from_env() -> Self {
        let hz = std::env::var("OM9_FRAME_HZ")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .filter(|v| v.is_finite());
        Self::new(hz, Instant::now())
    }
    pub fn new(fixed: Option<f64>, now: Instant) -> Self {
        let fixed = fixed.filter(|v| *v == 30. || *v == 60.);
        Self {
            hz: fixed.map_or(60, |v| v as u32),
            fixed: fixed.is_some(),
            quality: PreviewQuality::Q0,
            next: None,
            last: None,
            interval: None,
            over: 0,
            under: 0,
            history: VecDeque::new(),
            switched: now,
            recovery: None,
        }
    }
    pub fn hz(&self) -> u32 {
        self.hz
    }
    pub fn target(&self) -> Duration {
        Duration::from_secs_f64(1. / self.hz as f64)
    }
    pub fn due(&self, now: Instant) -> bool {
        // Winit/vsync may deliver the frame a fraction early. Refusing that opportunity would
        // present the cached image and postpone a ready camera by an entire refresh interval.
        // This is CPU dispatch lead, never future camera sampling; the absolute cadence stays fixed.
        self.next.is_none_or(|n| {
            now >= n || n.saturating_duration_since(now) <= Duration::from_millis(1)
        })
    }
    /// Called once per rendered window frame, never from a pane or a layout retry.
    pub fn begin(&mut self, now: Instant) {
        self.interval = self.last.map(|last| now.saturating_duration_since(last));
        self.last = Some(now);
        let period = self.target();
        let mut next = self.next.unwrap_or(now) + period;
        if next <= now {
            next = now + period;
        }
        self.next = Some(next);
    }
    pub fn idle(&mut self) {
        self.last = None;
        self.next = None;
        self.interval = None;
        self.recovery = None;
        self.history.clear();
        self.over = 0;
        self.under = 0;
    }
    pub fn reset_quality_feedback(&mut self) {
        self.recovery = None;
        self.history.clear();
        self.over = 0;
        self.under = 0;
    }
    pub fn wait(&self, now: Instant) -> Duration {
        self.next.map_or(Duration::ZERO, |n| {
            n.saturating_duration_since(now)
                .saturating_sub(Duration::from_millis(1))
        })
    }
    pub fn observe(&mut self, now: Instant, work: Duration) {
        let late = self
            .interval
            .is_some_and(|dt| dt > self.target() + Duration::from_millis(2));
        self.observe_load(now, work, late);
    }
    pub fn observe_load(&mut self, now: Instant, work: Duration, missed: bool) {
        let ms = work.as_secs_f64() * 1000.;
        let budget = self.target().as_secs_f64() * 1000.;
        let late = missed || ms > budget;
        self.over = if late { self.over + 1 } else { 0 };
        self.under = if ms <= budget * 0.75 {
            self.under + 1
        } else {
            0
        };
        if self.over >= 3 {
            self.quality = self.quality.worse();
            self.over = 0;
            self.under = 0;
        }
        if self.under >= 60 {
            self.quality = self.quality.better();
            self.under = 0;
        }
        self.history.push_back((now, ms, late));
        while self
            .history
            .front()
            .is_some_and(|(t, _, _)| now.saturating_duration_since(*t) > Duration::from_secs(1))
        {
            self.history.pop_front();
        }
        if self.fixed {
            return;
        }
        let can_switch = now.saturating_duration_since(self.switched) >= Duration::from_secs(5);
        let full_window = self.history.front().is_some_and(|(t, _, _)| {
            now.saturating_duration_since(*t) >= Duration::from_millis(900)
        });
        if self.hz == 60
            && self.quality == PreviewQuality::Q3
            && full_window
            && self.history.iter().filter(|r| r.2).count() * 5 > self.history.len()
            && can_switch
        {
            self.hz = 30;
            self.switched = now;
            self.recovery = None;
        }
        if self.hz == 30 {
            let mut work: Vec<_> = self.history.iter().map(|r| r.1).collect();
            work.sort_by(f64::total_cmp);
            let fast =
                !work.is_empty() && work[((work.len() - 1) as f64 * 0.95).ceil() as usize] <= 12.;
            if !fast {
                self.recovery = None;
            } else {
                let start = *self.recovery.get_or_insert(now);
                if now.saturating_duration_since(start) >= Duration::from_secs(3) && can_switch {
                    self.hz = 60;
                    self.switched = now;
                    self.recovery = None;
                }
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn deadlines_do_not_sleep_or_inherit_idle() {
        let now = Instant::now();
        let mut s = WindowFrameScheduler::new(Some(30.), now);
        s.begin(now);
        assert!(!s.due(now));
        assert!(s.due(now + Duration::from_secs(1)));
        s.begin(now + Duration::from_secs(1));
        assert!(s.wait(now + Duration::from_secs(1)) <= s.target());
        s.idle();
        s.begin(now + Duration::from_secs(20));
        assert!(s.interval.is_none());
    }
    #[test]
    fn profiles_degrade_and_recover_with_hysteresis() {
        let now = Instant::now();
        let mut s = WindowFrameScheduler::new(Some(60.), now);
        for i in 0..3 {
            s.observe(now + Duration::from_millis(i), Duration::from_millis(20));
        }
        assert_eq!(s.quality, PreviewQuality::Q1);
        for i in 0..59 {
            s.observe(
                now + Duration::from_millis(10 + i),
                Duration::from_millis(2),
            );
        }
        assert_eq!(s.quality, PreviewQuality::Q1);
        s.observe(now + Duration::from_millis(70), Duration::from_millis(2));
        assert_eq!(s.quality, PreviewQuality::Q0);
    }
    #[test]
    fn invalid_hz_never_panics() {
        let s = WindowFrameScheduler::new(Some(f64::NAN), Instant::now());
        assert_eq!(s.hz(), 60);
    }
    #[test]
    fn auto_fallback_and_recovery_require_sustained_windows() {
        let now = Instant::now();
        let mut s = WindowFrameScheduler::new(None, now);
        for i in 0..=360 {
            s.observe_load(
                now + Duration::from_millis(i * 17),
                Duration::from_millis(25),
                true,
            );
        }
        assert_eq!(s.hz(), 30);
        for i in 1..=180 {
            s.observe_load(
                now + Duration::from_millis(6120 + i * 34),
                Duration::from_millis(5),
                false,
            );
        }
        assert_eq!(s.hz(), 60);
        s.idle();
        assert!(s.recovery.is_none());
        assert!(s.history.is_empty());
    }
}
