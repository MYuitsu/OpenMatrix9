# Viewport navigation implementation

Approved scope: user plan 2026-10-21; standalone Rust OM9. Source investigation:
[forensic review](VIEWPORT-NAVIGATION-FORENSICS-REVIEW-2026-10-21.md).

## Contract

Direct orbit/pan, no release inertia. Raw wheel input updates a log-scale target; critically damped
response omega=90/s times finite speed (0.25..4), exact target by 180ms/speed. Same scale controls
distance/span and anchor compensation. One nonblocking scheduler/window, per-view state and one
pointer owner. Input is consumed once per egui frame; interruption starts from displayed camera.

Preview profiles Q0=100%/4x/current display policy; Q1=85%/4x/1px; Q2=70%/1x/2px;
Q3=50%/1x/4px. Selected meshes stay LOD0. Three over-budget frames reduce quality; sixty frames
with 25% headroom increase it. At Q3, >20% deadline misses in one second falls from 60 to 30Hz;
return after three seconds with work p95<=12ms, at least five seconds between frequency changes.
Explicit OM9_FRAME_HZ=30/60 fixes the benchmark cadence; default is auto.

Release or 80ms held-still starts refinement; wheel starts it after settling. Final image: native
physical resolution, 4x MSAA, LOD0, full display edges, no 3mm focus omission. Only publish a complete
image matching document/scene/camera/selection/size/generation. New input invalidates old jobs.
Slow completion (>250ms) is visible and never mislabeled complete. Canonical geometry/picking/Undo/
archive data is unchanged by preview quality. Final accuracy is source display tessellation accuracy.

## Checkpoints

1. Telemetry: session/build fingerprints, input/frame IDs, actual dt, current render counters, bounded
   buffered diagnostics and dropped-record counts.
2. Scheduler and ownership: layout -> route -> sample -> paint; no UI sleep; focus/release/cancel,
   pane boundaries, active!=gesture owner and multipass tests.
3. Camera: raw units, wheel conservation/reversal/clamp, intermediate anchor reprojection, direct
   interruption, poles/deep zoom, camera/overlay/pick consistency.
4. Rendering: adaptive targets, shared retained geometry/occurrences, encoder API, incremental owned
   preparation, generation-safe full-quality publication. Upload budget 8MiB, CPU service budget 2ms.
5. Validation: three warm repeats on 125/729/27000, 1V/4V, DPI 100/125/150; local and RDP separate;
   PresentMon for presentation, optional GPU timestamps. Targets: 60Hz p95<=18.5/p99<=25ms; 30Hz
   p95<=35/p99<=40ms; receive-to-submit p95<=20/37ms; warm 125/729 final-image p95<=250ms.

Use CARGO_TARGET_DIR=L:/om9-target and -j 2. Preserve all pre-existing checkout changes. No manual
gate is signed by automated checks. Evidence and remaining work are recorded separately from this
approved contract. Skills/business rules are not edited without the requested consent.

## Trạng thái thi công

Runtime đã được triển khai theo checkpoint; xem [implementation receipt](../../evidence/M01/2026-10-21-navigation-implementation.md) để phân biệt code, test và gate chưa đạt. Mọi ngưỡng trong kế hoạch vẫn là mục tiêu nghiệm thu, không phải kết quả mặc định. Forensic review được giữ nguyên làm hồ sơ lịch sử.

### Acceptance thresholds

| Metric | Target |
| --- | --- |
| Local presented 60 Hz | p95 <=18.5 ms; p99 <=25 ms |
| Local presented 30 Hz | p95 <=35 ms; p99 <=40 ms |
| App receive to submit | p95 <=20 ms / <=37 ms |
| Wheel last event to exact target | <=180 ms/speed plus one displayed frame |
| Warm 125/729 final image | p95 <=250 ms; full source LOD0/4x |
| Heavy final image | complete, input remains responsive, report actual completion time |
| Idle after completion | no periodic geometry draw |

Validation matrix: 125/729/27000 root objects, 1V/4V, DPI 100/125/150, three warm repetitions, Console/local and RDP separately. Missing measurement stays unproved. Queue 1/2 A/B uses OM9_FRAME_LATENCY; optional GPU timestamps and PresentMon are separate evidence. Worker snapshots and upload limits do not by themselves prove bounded GPU draw latency.
