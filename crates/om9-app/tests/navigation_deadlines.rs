// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_app::frame_scheduler::WindowFrameScheduler;
use std::time::{Duration, Instant};
#[test]
fn fractionally_early_vsync_does_not_double_the_camera_interval() {
    let origin = Instant::now();
    let mut scheduler = WindowFrameScheduler::new(Some(30.), origin);
    scheduler.begin(origin);
    for frame in 1..=120 {
        let delivery =
            origin + Duration::from_secs_f64(frame as f64 / 30.) - Duration::from_micros(300);
        assert!(
            scheduler.due(delivery),
            "frame {frame} must use the early delivery"
        );
        scheduler.begin(delivery);
        assert!(
            !scheduler.due(delivery + Duration::from_millis(1)),
            "cannot dispatch a second frame in the same slot"
        );
    }
}

#[test]
fn remote_presentation_uses_the_same_bounded_cad_clock() {
    use eframe::wgpu::PresentMode;
    use om9_app::frame_scheduler::surface_present_mode;
    assert_eq!(surface_present_mode(true, None), PresentMode::AutoNoVsync);
    assert_eq!(surface_present_mode(false, None), PresentMode::AutoVsync);
    assert_eq!(surface_present_mode(true, Some("auto-vsync")), PresentMode::AutoVsync);
    assert_eq!(surface_present_mode(false, Some("auto-no-vsync")), PresentMode::AutoNoVsync);
    let origin = Instant::now();
    let mut scheduler = WindowFrameScheduler::new(Some(30.), origin);
    scheduler.begin(origin);
    // A nonblocking surface can deliver input/repaints faster than 30 Hz.
    // Those deliveries must not turn into unrestricted scene submissions.
    let mut draws = 0;
    for ms in 1..=3000 {
        let now = origin + Duration::from_millis(ms);
        if scheduler.due(now) { scheduler.begin(now); draws += 1; }
    }
    assert!((89..=91).contains(&draws), "{draws} draws in three seconds");
}
