// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_app::frame_scheduler::WindowFrameScheduler;
use std::time::{Duration, Instant};
#[test]
fn fractionally_early_vsync_does_not_double_the_camera_interval() {
    let origin = Instant::now();
    let mut scheduler = WindowFrameScheduler::new(Some(60.), origin);
    scheduler.begin(origin);
    for frame in 1..=120 {
        let delivery =
            origin + Duration::from_secs_f64(frame as f64 / 60.) - Duration::from_micros(300);
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
