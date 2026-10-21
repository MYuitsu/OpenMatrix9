# Viewport interaction P1a — chunk classes + batched uploads (2026-10-20)

Plan: `implementation/design/M01/VIEWPORT-INTERACTION-PERFORMANCE.md`. This note records P1a only; P1b
(shared scene), P2 (instancing + per-view LOD), P3 (off-thread build) and P4 (conditional cluster) are
not implemented.

## Change

- `crates/om9-render/src/scene.rs`
  - `chunk_capacity_for`: chunks grow in `2/8/32/128/256 MB` classes, never at the device's maximum
    buffer size. The old path opened a ~2 GB chunk when a 1 MB chunk filled, once per view.
  - `pending_vertices` / `pending_indices`: `add_mesh` and `add_static` append bytes; `flush_uploads`
    merges consecutive appends per chunk into one `write_buffer`, called by `render_into*`.
  - `clear` clears pending uploads.

No geometry, shader, document or picking behavior changed.

## Measured (1000 earrings, 42,000 scene objects, 72.5M triangles, 4 views, RTX 3060)

| State | Open | VRAM | Private |
| --- | --- | --- | --- |
| Before | ~55 s | 8.06 GB | 9.3 GB |
| After P1a | **20 s** | **7.05 GB** | 7.8 GB |
| LOD 1200 (before P1a) | ~55 s | 4.55 GB | 5.2 GB |

Open time was polled until the window responded; VRAM is
`Get-Counter '\GPU Process Memory(*)\Dedicated Usage'` for the app PID.

## Tests

- `cargo test --release -p om9-render --test scene_bench -j 2` (serial direct run): 15 passed, 4 ignored;
  2000 objects 1.07 ms/frame, heavy meshes 0.18 ms/frame.
- The same suite run once hung for >10 minutes with the app's release binary also running; the serial
  direct run completed in 8 s. Treat that as a transient GPU/driver/parallel-run condition and re-run
  serially when it recurs; no code path loops in `flush_uploads`.

## Next

P1b: one shared `GpuScene` for the four viewports. It removes the remaining 7.05 GB (four copies of the
same geometry). Acceptance: same rendered image, VRAM <= 1.5 GB, open <= 10 s, then P2.

## P1b — shared scene across four viewports (same day)

`MatrixApp` now owns one `SharedScene = Arc<Mutex<Option<GpuScene>>>`; every `GpuViewport` renders
through it with its own camera, and `refresh_scene` syncs once per epoch/CPlane/selection change.
Measured on the same 1,000-earring scene:

| State | Open | VRAM | Private | Orbit/zoom/pan median |
| --- | --- | --- | --- | --- |
| P1a (4 copies) | 20 s | 7.05 GB | 7.8 GB | 19.3 / 18.9 / 18.7 ms (full detail) |
| P1b full detail | 10 s | 2.05 GB | 2.39 GB | 19.3 / 18.9 / 18.7 ms |
| P1b + LOD 1200 | ~10 s | **1.05 GB** | **1.36 GB** | **16.7 / 16.6 / 16.7 ms** |

Tests: `om9-app --lib` 116 passed, `interaction_log` 1 passed. P1b therefore meets the 1k VRAM/open
targets; the remaining 60 fps p95 for full detail needs P2 LOD/instancing.

## P2a — automatic display LOD (same day)

Without any environment variable, a document scene above the triangle budget is decimated per unique
mesh by `OM9_VIEWPORT_TRI_BUDGET` (default 12,000,000 displayed triangles), scaled by the number of
root occurrences (not expanded object count). Manual `OM9_VIEWPORT_LOD=<triangles>` still overrides,
`off` disables. Measured 1,000 earrings: open 3 s, VRAM 0.55 GB, private 1.03 GB, orbit/zoom/pan p95
17.4/17.0/17.1 ms; screenshot `build/stress-27000/auto-lod-1k.png`. Tests: app lib 116 passed,
interaction_log 1 passed. Instancing (P2b) is still needed for 8k/500k; automatic LOD alone does not
deduplicate geometry.

## 8,000 earrings without instancing (same day)

Automatic LOD (12M displayed triangles) plus overlay chunking (4M vertices / 48 MB per vertex buffer;
a single 469 MB overlay buffer had exceeded the 256 MB device cap) makes the 336,000-object scene run:
open 10 s, VRAM 1.05 GB, private 1.92 GB, orbit/zoom/pan p95 21.4/16.8/16.8 ms. No instancing was
needed for this gate. 500,000 objects remain untested.

## 500,000-object gate — blocked at fixture generation (2026-10-20)

Generating a real 500,000-entity `.om9` fixture with `stress_instances` did not finish within 10
minutes (killed; no viewport measurement). The failure is in the document/archive writer path for
500k entities, before the renderer is reached, not in the viewport. `document_limits` had to be raised
to `OM9_MAX_OBJECTS=600000` and `OM9_MAX_BYTES=4 GB` just to admit it.

Next step for the 500k gate: measure with an instanced/virtual scene (one definition, N occurrences is
already what the fixture uses, but the document still serializes 500k entities) or profile
`Archive::save`/`Document` validation for 500k entities. Do not claim a 500k viewport result until a
fixture opens and camera-audit numbers exist.

## Accuracy groundwork (2026-10-20)

Implemented and unit-tested: locked QEM (`simplify_with_error_locked`), automatic feature locks
(open boundaries and sharp edges > 30 degrees), per-level `ClusterGroup.errors_world`, feature-locked
display LOD, selected objects forced to LOD 0, cluster coarse-pixel threshold wired to the shader
(`OM9_CLUSTER_PIXELS`), and `OM9_LOD_SHARP_DEGREES` for jewellery-specific sharpness. Tests: QEM 6,
meshlet 7, app LOD 2, all passing. Still missing: multi-level shader selection, instance path, and the
image-diff/zoom-matrix accuracy evidence.

## Accuracy evidence on the real jewellery fixture (2026-10-20)

- `qem::accuracy_tests`: sphere 1024 -> 512 triangles, area error 1.41%, volume 3.12%.
- `qem::prong_accuracy_tests`: 0.4 mm x 2 mm prong, 1024 -> tiles at 50%, bounding box moves < 1e-4 mm.
- `om9-app/tests/accuracy_fixture.rs` on `bong.om9`: area error over 8 BReps worst 0.49% at 50% LOD;
  image diff at 75% LOD worst 0.34%, while 50% LOD reaches 3.07% (too much for close-up).
- Decision: LOD 0-75% near/selected; 50% only for small projected objects.

## CPU/GPU split at 27k (2026-10-20)

Camera audit on 27,000 copies, 120 frames:

| Action | frame_ms | prepare_ms | draw submit ms |
| --- | --- | --- | --- |
| orbit | 16.6 | 0.0 | 0.4 |
| zoom | 16.7 | 0.0 | 0.3 |
| pan | 16.7 | 0.0 | 0.3 |

CPU submit is 0.3-0.4 ms and scene prep is 0 in steady state; the frame is GPU/vsync bound. This
confirms the hybrid split: keep document/picking/instance prep on CPU, move per-instance cull/LOD and
draw to GPU (cluster compute + indirect) for any further gain.

## Picking 27k baseline (2026-10-20)

BVH is integrated into `pick_candidates` (fat ray using tolerance x world-per-pixel) and all 7
`world_picking` tests pass. On `bong-27k.om9`, a pick through the dense cloud averaged **1214.89 ms**
over 20 picks: the BVH rejects off-ray objects, but the ray still crosses thousands of objects and the
exact geometry tests dominate. `om9-app/tests/pick_27k.rs` records this RED baseline. Next: distance
ordered traversal with early out for a single pick, and screen-space AABB candidates for the ambiguity
list.

## Picking 27k iterations (2026-10-20)

| Step | Mean pick |
| --- | --- |
| BVH + fat ray | 1214.89 ms |
| + screen-space sphere reject | 558.44 ms |
| + depth early-out | 486.48 ms |
| + near-to-far candidate sort | 493.17 ms (no gain) |

`world_picking` stays 7/7. The remaining cost is exact triangle testing for the many objects still
inside the tolerance/depth cone; a loop-level optimisation cannot reach 16 ms. Next architecture:
GPU ID-buffer for object/hover picking (one-pixel readback) while CPU keeps exact edge/vertex/snap
picking, or leaf-level triangle AABBs inside the CPU BVH.

## Camera smoothing for manual gestures (2026-10-20)

`MatrixApp` now keeps an ephemeral per-view `CameraMotion` (pose = target, azimuth, elevation, radius,
ortho span). Manual orbit/pan/zoom update the target; each frame eases the current pose towards it
(`1 - exp(-dt/tau)`, default tau 60 ms, `OM9_CAMERA_SMOOTH_MS`, 0 disables). Zoom interpolates in log
distance, azimuth takes the shortest path, and the final frame equals the requested camera. Hover
picking is skipped while the camera is moving, so the 27k pick cannot run every animated frame.

Evidence on `bong-729-whole.om9` with `OM9_INTERACTION_LOG`: a 30-step drag produced 11 camera events
with monotone revisions (2..7) and no single-frame jump to the final target; `tau_ms=60`. The scripted
`OM9_CAMERA_AUDIT` path bypasses smoothing and keeps its deterministic camera numbers.

Tests: `camera_motion` 5/5, `om9-app --lib` 122/122.

## Manual gesture log: zoom burst diagnosis (2026-10-20)

Log `build/stress-27000/smooth-final-729.jsonl` (186 camera events, 729 scene):

- frame median 31.3 ms, but 15 slow frames of 998-2309 ms, all `action=zoom`.
- those events carried `wheel` values 108-648; the old formula `0.9^(wheel/40)` produced a 0.18-0.75
  factor in one frame, i.e. several zoom octaves per event, so smoothing still looked like a teleport.
- wheel bursts at that size are normal for RDP/smooth-scroll devices.

Fix: clamp one frame's wheel to +/-2 steps (`0.9^steps`, steps = wheel/40 clamped) before updating the
target. Re-test `smooth-zoom-729.jsonl`: wheel 2-26, frame 5-44 ms, no 1-2 s spike. Pan/orbit remain
around 31 ms in this RDP session, consistent with the 30 fps presentation cap; camera smoothing cannot
raise the display rate.

## Frame pacing 30 Hz (2026-10-20)

`frame_pace::FramePacer` schedules the next frame from an absolute deadline and sleeps out early
input-driven wakeups, so the interval is stable instead of bursting.

- 729 whole earrings, synthetic right-drag, `OM9_FRAME_HZ=30`: 28 logged frames, dt min/median/p95/max
  **33.3 / 33.6 / 33.8 / 33.9 ms**, 0 frames under 25 ms or over 40 ms.
- `OM9_FRAME_HZ=60` on the same scene: median **31.2 ms** (not 16.7), and `OM9_MSAA=1` does not change
  it (31.1 ms). The 60 Hz floor is therefore not MSAA or triangle budget; it is the per-frame
  present/compositor cost of this session (RDP is 30 Hz-class) and needs the GPU timestamp / local
  console test before further quality changes.
- `frame_ms` in the camera log is still the event gap; use `dt_ms` for real frame interval.

## 30 Hz hold-drag fix (2026-10-20)

Outliers in `pace30-final.jsonl` (dt 36-455 ms) happened when the drag button was held still: the
motion settled and the paced loop stopped, so the next pointer move inherited the idle gap. The loop
now stays paced while the button is still down, and only stops after release.

Log `hold30-729.jsonl` (729 scene, OM9_FRAME_HZ=30, 500 ms hold then 40-step drag): 18 frames, dt
min/median/p95/max **33.3 / 33.7 / 33.8 / 33.8 ms**, 0 frames over 40 ms.

## Residual 30 Hz outliers (2026-10-20)

`hold30-729.jsonl` (user drag, 189 camera events): steady frames are 33.3-33.8 ms, but 27 frames are
36-455 ms. The large ones coincide with the pointer leaving the canvas or pausing between drag bursts;
the app stopped pacing, so the next event inherited the idle gap. Two fixes:

- smoothing dt is capped at 1.25 x the pacing interval, so a pause cannot become one giant camera step;
- pacing stays alive while any mouse button is down, even if the drag is momentarily outside the canvas
  or the motion has settled, so the next pointer move lands on the normal 33 ms cadence.

Fresh log `paced-final.jsonl` is the acceptance run for this.

## Deep zoom (2026-10-20)

`Camera::fit_clip(radius)` lowers the near plane with the orbit radius (radius*0.002, min 1e-5) and
updates far; `CameraPose::to_camera` calls it, so a perspective zoom to a prong is no longer clipped by
the fixed 0.01 near plane.

The wheel clamp that dropped input was removed: the whole burst is applied to the target and the
camera spring spreads it over frames. Synthetic wheel at the perspective pane, 729 scene: radius
100 -> 59.8 -> 45.2 -> 31.6 -> 25.9, then 20.3 -> 10.1 with further wheel; 30 Hz pacing stays at
33.3-33.9 ms. `camera_motion` tests 6/6; `om9-app --lib` 125/125.

## Deep zoom follow-up (2026-10-21)

- Wheel clamp removed: the whole burst updates the target, spring spreads it; synthetic wheel took
  perspective radius 100 -> 59.8 -> 45.2 -> 31.6 -> 25.9 -> 20.3 -> 10.1 -> 3.4 -> 1.25 -> 0.55 mm.
- `fit_clip`: near = radius*0.002 (min 1e-5), far = max(radius*8, 10), so a 0.55 mm orbit is not
  clipped and distant overdraw is reduced.
- Close-zoom switch disables merged static ranges when radius < 35% of scene extent; overlay point-size
  rebuild is skipped while the camera is moving.
- 30 Hz pacer remains 33.3-33.9 ms for steady drag. Deep zoom still shows occasional 0.5-1.5 s frames
  in RDP, before/around the close-range rebuild; this needs GPU timestamps/local console to separate
  overdraw/present from the rebuild.

## Deep zoom audit (2026-10-21)

`OM9_CAMERA_AUDIT=120`, `OM9_AUDIT_ZOOM_FACTOR=0.9`, 729 scene:

- fit zoom frames: 4.7-5.0 ms, prepare 0.1 ms, draw 0.2 ms.
- deep zoom (radius 1.1 -> 0.9 mm): frame 652-693 ms, **prepare 216-230 ms**, draw 38-47 ms.
- Fixed 3 mm focus culling and close-range merge switch lowered draw from 345 ms to 38 ms, but prepare
  remains ~220 ms, so the deep zoom batch is still being rebuilt too often (or its upload dominates).
- 30 Hz pacer itself remains 33.3-33.9 ms in steady drag; deep zoom is not yet 30 Hz.

## Shared close mode fixes deep zoom (2026-10-21)

The close-range toggle was computed per viewport camera, so four panes fought over one shared scene and
rebuilt the batch every frame. `GpuViewport::close_mode` is now computed once from the active view and
passed to all panes.

Audit `OM9_CAMERA_AUDIT=120`, `OM9_AUDIT_ZOOM_FACTOR=0.9`, 729 scene:

- radius 54.6 -> 1.1 -> 0.896 mm: frame **4.8-5.1 ms** (max 219.7 ms for the one-time rebuild),
  prepare **0.02 ms**, draw **0.24 ms**.
- Peak frame 219.7 ms is the single close-mode rebuild, not steady state.
- 30 Hz pacer remains 33.3-33.9 ms; deep zoom now has ample headroom.
