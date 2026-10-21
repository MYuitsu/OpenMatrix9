# Viewport navigation forensics — per-view clock, gesture capture, retargeted transitions

Status: **analysis + implementation plan**, 2026-10-21. This file is not runtime evidence.
Owner: OM9 viewport / M01. Related:
[CAMERA-NAVIGATION-OPTIMIZATION.md](CAMERA-NAVIGATION-OPTIMIZATION.md).

## 1. Log evidence

Source: `build/stress-27000/nav-pointer-125.jsonl` (125 stone scene, 30 Hz pacer).

| Metric | Value |
| --- | --- |
| camera events | 326 (orbit 201, zoom 113, pan 12) |
| dt all: median / p95 / max | 33.5 / 724.2 / 2001.3 ms |
| event gap: median / p95 / max | 34 / 749 / 3536 ms |
| gaps >100 ms | 59 of 325 |
| continuous dt (gap ≤100 ms): median / p95 / max | 33.5 / **33.7** / **81.7** ms |
| orbit dt: median / p95 / max | 33.5 / 72.2 / 1631 ms |
| pan dt | 33.3 ms for 12/12 |
| orbit radius | constant 74.8 mm (rotation correct) |
| zoom radius | 100 → 9.0 mm (deep zoom works) |
| prepare/draw on slow frames | 0.01 / 0.3–0.45 ms |

Two separate facts:

1. **Long frames are input gaps, not render stalls.** In most outliers `dt_ms` equals the gap since the
   previous pointer/wheel event (1810.3↔1810.3, 1861↔1861, 1485↔1484.5). During the gap the app is
   idle waiting for input; it is not rendering that slowly.
2. **A few scheduling outliers remain inside continuous motion:** 40.2, 54.7, 64.8, 72.2, 78.3,
   81.7 ms. Prepare/draw stay below 1 ms, so these are frame-loop/pacer problems, not GPU.

## 2. Root causes, ranked

1. **One global `FramePacer` shared by four viewports.**
   `viewport_camera_input` called `begin` only for `workspace.active`. A right-drag on Perspective does
   not set `active` (only title/primary click does), so the gesture pane could run with a pacer owned by
   another pane, or with none. This matches pan being perfect while orbit occasionally stalled.
   The code now has a gesture-owner condition, but it has not been rebuilt/verified.
2. **Pointer capture is too narrow.**
   `response.dragged_by(...)` depends on the canvas response. When the pointer leaves the canvas or
   hovers another pane, the drag can stop; the next move inherits the gap.
3. **Wheel transitions are recreated, not retargeted.**
   Each wheel frame builds a new `CameraTransition` from the current pose to the old target with a full
   140 ms duration. This resets easing and makes zoom look stepped even with steady input.
4. **The pacer sleeps in the UI thread.**
   `FramePacer::begin` sleeps until the next deadline. It guarantees 30 Hz but blocks input handling; a
   late event is processed late. Acceptable for the 30 Hz stage only if ownership and capture are fixed;
   candidate for replacement after measurement.
5. **Logging only records gesture frames.**
   Settle frames and non-gesture paced frames are not logged, so the log cannot show whether a
   transition was smooth between input events.
6. **Decompiled Matrix UI has no camera loop to copy.**
   `ref/matrix/decompiled-ui` contains UI/display conduits; camera manipulation is native Rhino and is
   not present as a full loop. KiCad/Blender/FreeCAD remain the navigation references.

## 3. Chosen design

- **Per-view `NavState`**, including its own `FramePacer`; one clock per viewport, one gesture owner.
- **Gesture capture** independent of the canvas response.
- **Direct drag 1:1** for orbit/pan (already implemented; keep).
- **Bezier transition with retarget** for wheel zoom and view changes; never recreate per frame.
- **Zoom-to-cursor** (already implemented; keep).
- **30 Hz first**, 60 Hz only after continuous-frame p95 ≤35 ms on 125/729.
- **Log every paced frame**, not only input frames.

Rejected for now:

- Global 120 Hz timer: elapsed sampling at 30/60 is sufficient; a second clock adds jitter.
- Predictive reprojection/timewarp: accuracy/overshoot risk with no measured benefit.
- Spring/inertia: direct drag is more responsive; KiCad/Blender use direct drag.

## 4. Data model

Replace the global `MatrixApp.frame_pacer`, `camera_transition`, and `zoom_pending` with one map:

```rust
struct NavState {
    rig: CameraRig,
    transition: Option<CameraTransition>,
    pacer: FramePacer,
    wheel_pending: f64,
    drag: Option<DragCapture>,
    last_input: Instant,
    last_pointer: Option<egui::Pos2>,
}

struct DragCapture {
    button: egui::PointerButton,
    view: ViewId,
}
```

- `MatrixApp.nav: BTreeMap<ViewId, NavState>`.
- `CameraRig` and `CameraTransition` stay as implemented.
- `FramePacer` moves into each `NavState`; no shared `last/next`.
- Delete global `frame_pacer`, `camera_transition`, `zoom_pending` fields.

## 5. Input pipeline

### 5.1 Active view and capture

- On any pointer press inside a viewport (`Primary`, `Secondary`, `Middle`), set
  `workspace.active = id`.
- For orbit/pan buttons, create `DragCapture { button, view }` if none.
- While capture exists:
  - use the global pointer position/delta even outside the canvas;
  - apply orbit/pan directly to `rig` each paced frame;
  - keep the pacer running while the button is down, even with zero delta.
- On release anywhere, clear capture and let the transition settle.

### 5.2 Direct drag

- Orbit: `rig.orbit_turntable(dx*0.01, -dy*0.01)`.
- Pan: camera right/up at the current view, `world_per_pixel`.
- Write `Camera` immediately; no transition, no spring.

### 5.3 Wheel

- Add wheel points to `wheel_pending`.
- Consume at most 2 steps per paced frame.
- Compute `target` from the existing transition target or current rig.
- Apply `zoom_toward(cursor_point, factor)`.
- **If a transition exists, call `retarget(target, now, 80ms)`; if not, create one with 140 ms.**
- Do not recreate the transition every frame.
- Keep the pacer alive while pending is non-zero; after it drains, let the transition settle.

### 5.4 Transition sampling

- Sample at `now`; remove the current `now + 16 ms` hack.
- `t = elapsed/duration`, Bezier, slerp, log distance.
- Final sample snaps to target.
- When a new gesture starts, sample the current pose and drop the transition.

## 6. Frame loop

- Each `NavState` owns a `FramePacer` (30 Hz target; `OM9_FRAME_HZ`).
- `begin` is called only by the state that owns a drag, transition, or wheel pending; never by all four
  panes.
- `finish` is called exactly once by that state.
- While a mouse button is down but delta is zero, schedule the next frame anyway.
- While idle, no pacing.
- Keep the pacer's sleep for the 30 Hz stage. If post-fix continuous outliers persist, run the
  `OM9_PACE_SLEEP=0` experiment: deadline scheduling only, measure input age and jitter, then choose.
- Log every paced frame with `dt_ms`, `gap_ms`, `mode`, `owner`, `transition_t`, `input_age_ms`.

## 7. Instrumentation

Add to the camera/frame record:

- `owner_view` and `active_view`;
- `gap_ms` since the previous input event;
- `input_age_ms`;
- `transition_t` and `transition_duration_ms`;
- `wheel_pending`;
- `pacer_skew_ms` (actual dt − target).

A separate `nav_frame` event (env `OM9_NAV_FRAME_LOG=1`) writes every paced frame, including settle
frames. This closes the current blind spot.

## 8. Tests and acceptance

Unit / UI:

- Existing `interaction_log`: right-drag orbit, middle-drag pan — keep passing.
- Existing `navigation_input`: wheel burst, radius reduction, cursor pivot — keep passing.
- New: set `workspace.active = LookingDown`, drag Perspective; assert Perspective camera revision and
  orbit log still change (pacer-owner fix).
- New: pointer leaves the Perspective canvas while held, then returns; assert drag remains captured and
  camera continues.
- New: two wheel events 100 ms apart; assert transition is retargeted (progress monotonic) and final
  distance matches accumulated steps, not a restart.
- New: continuous drag log has no `dt` equal to a gap >40 ms.

Manual / logs:

- 125 stone + 729, `OM9_FRAME_HZ=30`:
  - continuous frames p95 ≤35 ms, max ≤40 ms;
  - no `dt ≈ gap` while a drag capture is active;
  - 0 unexplained outliers >40 ms.
- Deep zoom 100→0.2 mm: no near clip, no teleport, several intermediate frames.
- `om9-app --lib` all pass; `world_picking` 7/7; accuracy unchanged.
- Evidence: `build/stress-27000/nav-owner-125.jsonl`, `nav-owner-729.jsonl`.

## 9. Phases

1. **P1 — Reproduce/measure**: build the already-present gesture-owner change; add per-frame logging;
   run the active-view mismatch test.
2. **P2 — Per-view NavState + pacer**: move pacer/transition/pending; delete the global pacer field.
3. **P3 — Capture**: pointer press/release ownership; drag outside canvas; active view on press.
4. **P4 — Retarget transitions**: wheel fixes; remove per-frame recreate and +16 ms sample.
5. **P5 — Validate 30 Hz**: 125/729; if outliers persist, test sleep-free pacing.
6. **P6 — 60 Hz / quality**: only if 30 Hz stable and local 60 is available.

## 10. Out of scope / assumptions

- No document/schema/picking/export changes.
- No GPU instancing/LOD rewrite.
- 60 Hz is not promised on RDP; 30 Hz is the acceptance target.
- Keep quaternion rig, direct drag, zoom-to-cursor.
- `camera_motion` is obsolete after `NavState` lands; remove it and its tests, or keep only as a
  deprecated wrapper for one phase.
- Matrix decompiled code is UI/display only; camera references are KiCad/Blender/FreeCAD behavior.
- No code copied from GPL/LGPL projects; algorithms only.
