# Camera navigation optimization — Blender / FreeCAD / KiCad / VTK lessons

Status: **PARTIALLY IMPLEMENTED (2026-10-21).** P1–P3 code and unit tests landed; manual 30 Hz
acceptance on the 125-stone scene is still pending a real pointer log. See “Implementation progress”
at the end.

Owner: OM9 viewport / M01. Scope: 125–27,000 occurrences, 30 Hz first, 60 Hz later.

## 1. Goal

Make orbit, pan and zoom feel continuous at the display cadence, with deep zoom into a prong or
stone, without changing document, picking, export or accuracy.

- 30 Hz: frame interval 33.3 ms, p95 ≤ 35 ms, no frame < 25 ms or > 40 ms while a gesture is active.
- Deep zoom: perspective radius at least 0.2 mm, no near-plane clipping.
- No teleport when input arrives in bursts (RDP, smooth-scroll devices).
- Same result on the 5×5×5 stone scene and the 729/27k scenes.

Not a target: >60 Hz, GPU instancing, picking performance.

## 2. What the logs established

- `stone125-30.jsonl` (125 whole earrings with `Diamond_Oval`):
  - pan: 41/41 camera frames at **33.3 ms**;
  - zoom: 54 events, 11 slow frames, but `dt_ms` equals the gap since the previous wheel event
    (511 ↔ 511.4, 612 ↔ 611.8, 821 ↔ 820.9, 1442 ↔ 1442.0);
  - `prepare_ms` ≈ 0.005, `draw_ms` ≈ 0.3 in those frames.
- After the shared close-mode fix, deep zoom audit:
  - radius 54.6 → 1.1 → 0.896 mm;
  - frame **4.8–5.1 ms**, prepare 0.02 ms, draw 0.24 ms;
  - one-time close rebuild 219.7 ms.
- Conclusion: the current limit is **input/animation semantics**, not raw GPU cost at these scenes.
  Wheel bursts are separated by idle gaps, and the spring finishes too quickly to bridge them.

## 3. Reference implementations

### Blender — `view3d_navigate_smoothview.cc`

- Smooth view runs on a **100 Hz timer** (`TIMER1`, 1.0/100.0), independent of display refresh.
- `factor = timer_duration / time_allowed`.
- **Smoothstep**: `3f² − 2f³`.
- Orientation: quaternion **slerp** (`interp_qt_qtqt`); offset: lerp; distance/lens: lerp.
- Duration is scaled by the rotation angle, so small rotations do not lag.
- Every tick tags the region for redraw; `RV3D_NAVIGATING` blocks competing edits.
- Mouse orbit/pan is direct manipulation; smooth-view is for transitions (views/camera).

Source: https://github.com/blender/blender/blob/2375ab40/source/blender/editors/space_view3d/view3d_navigate_smoothview.cc

### FreeCAD — `View3DInventorViewer.cpp`, `NavigationAnimation.cpp`

- `renderScene()` calls `scheduleRedraw()` while `isAnimating()`, so animation drives frames.
- `NavigationAnimation` uses a fixed-duration Qt animation.
- `FixedTimeAnimation` computes an axis/angle and linear velocity from the camera to the target,
  applies a delta rotation around the **rotation center** every tick, and snaps the exact target on
  stop (no drift).

Sources:
- https://github.com/FreeCAD/FreeCAD/blob/b7e8c00e6ea1a72029a8a69adcbdcd5bb212961c/src/Gui/View3DInventorViewer.cpp
- https://github.com/FreeCAD/FreeCAD/blob/a740632616cccc713281718fcee17501a1367202/src/Gui/NavigationAnimation.cpp

### KiCad 3D Viewer — `eda_3d_canvas.cpp`

Closest to OM9:

- `m_camera_is_moving` + `m_strtime_camera_movement`.
- Every paint: `curtime_delta_s = elapsed * speed`, `camera.Interpolate(delta)`, stop after 1.0 s,
  otherwise `Request_refresh()`.
- View changes use **Bezier** interpolation:
  `SetInterpolateMode(BEZIER); SetT0_and_T1_current_T(); Zoom_T1(1.26f);`
- Animation speed is a user multiplier (1–5 → 0.25×, 0.5×, 1×, 2×, 4×).
- Zoom toward the cursor: `move_pivot_based_on_cur_mouse_position()` sets the look-at target from a
  ray hit, then animates.
- `Request_refresh()` posts a redraw or starts a 10 ms one-shot timer; a paint guard prevents
  overlapping frames.

Source: https://gitlab.com/kicad/code/kicad/-/raw/master/3d-viewer/3d_canvas/eda_3d_canvas.cpp
Blog: https://www.kicad.org/blog/2020/09/Development-Highlight-3D-Viewer-Improvements/

### VTK / ParaView / SALOME

- VTK interactor styles use timers for spin/rotate.
- `vtkRenderWindow::SetDesiredUpdateRate` and `vtkLODActor` render a cheaper representation while
  interacting, then full quality when still.
- Lesson: interactive quality can be lower than still quality; this is not an accuracy change to
  the document.

Sources:
- https://vtk.org/doc/release/7.1/html/vtkInteractorStyleTrackballCamera_8h_source.html
- https://davis.lbl.gov/Manuals/VTK-4.5/classvtkLODActor.html

## 4. Design decisions for OM9

### 4.1 Two navigation modes

| Mode | Behaviour |
| --- | --- |
| Drag orbit / pan | Direct 1:1, no spring; one update per paced frame |
| Wheel zoom | Accumulated target + fixed-duration transition (120–160 ms) |
| View change / pivot | Fixed-duration Bezier transition (180–250 ms) |

Direct drag is what Blender/KiCad do. The current exponential spring adds lag and makes bursty
input look like a jump followed by a pause.

### 4.2 Camera transition, not per-frame spring

New `camera_transition.rs` (or rewrite `camera_motion.rs`):

```rust
pub struct CameraTransition {
    from: CameraPose,
    to: CameraPose,
    started: Instant,
    duration: Duration,
    curve: Ease, // Bezier(0.25, 0.1, 0.25, 1) or Smoothstep
}

impl CameraTransition {
    pub fn sample(&self, now: Instant) -> (Camera, bool); // camera, finished
}
```

- `t = (now - started).as_secs_f64() / duration`.
- `ease(t)` = cubic Bezier or smoothstep.
- Orientation: slerp; target/pivot: lerp; distance/ortho span: log-lerp.
- `t >= 1`: snap exactly to `to`, return finished.
- No spring velocity state; no asymptotic tail.

`CameraPose` remains the app-level representation (azimuth/elevation/radius/ortho span) initially.
A quaternion rig is a separate phase (see §4.7).

### 4.3 Speed multiplier

- `OM9_NAV_SPEED` or a View preference, range 0.25–4.0, default 1.0.
- Applied to `duration` (speed > 1 → shorter), matching KiCad.
- User can make navigation snappier without changing code.

### 4.4 Wheel accumulator + transition

Keep the current `zoom_pending` accumulator, but feed it into a transition:

- Wheel event adds points to `zoom_pending`.
- A paced frame consumes up to 2 steps, updates the **target** distance, starts/retargets a
  transition.
- If more pending remains, the next frame continues; the transition duration is per burst, not per
  point.
- Persist residual across frames; do not drop input.

### 4.5 Zoom toward the cursor

- Ray from pointer (`Camera::ray`).
- Intersect the plane through the current pivot, normal = view direction (or the scene hit point).
- Move the pivot toward that point while changing distance, so the point under the cursor stays
  under the cursor.
- KiCad implements the same idea in `move_pivot_based_on_cur_mouse_position`.
- Orthographic: adjust pivot and span.

### 4.6 Frame loop

- Keep `FramePacer` at 30 Hz (later 60 Hz).
- While a transition or drag is active: `request_repaint_after(next)` every frame.
- While idle: event-driven, no redraw.
- `CameraTransition` uses real elapsed time, so a late frame still samples the correct pose.
- No 100 Hz camera timer initially; elapsed-time sampling at 30 Hz is mathematically equivalent for
  a 120–200 ms transition. Add a 100 Hz timer only if measurement shows >1 frame of error.

### 4.7 Quaternion rig (phase 2)

Blender/FreeCAD use quaternions. OM9's turntable azimuth/elevation is adequate for a Z-up CAD
viewport, but a quaternion rig enables:

- trackball orbit,
- arbitrary view orientation,
- slerp without pole/gimbal issues.

Plan: after the transition path is stable, add `CameraRig { pivot, orientation: Quat, distance }`
and convert to/from the serialized `Camera` at the boundary. Do not change `.om9` schema.

### 4.8 Interactive quality

VTK-style, only if needed after the transition/direct-drag work:

- while moving: MSAA 4× → 1×/2×, render scale 0.75;
- idle: restore 4×/1.0;
- active view only during motion; other panes use cached textures.
- This must not alter accuracy when still.

## 5. Changes by file

- `crates/om9-app/src/camera_transition.rs` (new): `CameraPose`, `CameraTransition`, `Ease`,
  slerp/lerp/log-lerp, tests.
- `crates/om9-app/src/camera_motion.rs`: replaced or reduced to wrappers; remove exponential
  smoothing.
- `crates/om9-app/src/ui.rs`:
  - `viewport_camera_input`: direct orbit/pan; wheel → accumulator → transition;
  - `request_repaint` only while a gesture/transition is active;
  - log `dt_ms`, `transition_ms`, `t`, `pivot`, `speed`;
  - hover picking paused while moving.
- `crates/om9-app/src/frame_pace.rs`: unchanged target 30/60; ensure one `begin` per frame for the
  active view.
- `crates/om9-app/src/gpu_viewport.rs`: only if interactive quality is enabled.
- `crates/om9-scene/src/camera.rs`: optional `zoom_toward(pivot, factor)` helper; no schema change.
- Do not change `Camera` serde fields or `.om9`.

## 6. Edge cases

- Input burst after idle: transition samples elapsed time; no giant jump.
- Wheel direction reversal: residual adds algebraically; target clamps; no overshoot.
- New gesture during transition: retarget from the **current sampled pose**, not from the old `to`.
- Pointer leaves canvas: keep drag capture; do not stop pacing until release.
- Deep zoom: `fit_clip` near/far; focus cull at close range (already present); shared close mode.
- Audit: bypass transitions and pacing; deterministic camera audit unchanged.
- 4 viewports: only active view transitions; others cached.
- RDP 30 Hz cap: document; do not chase 60 there.

## 7. Tests and acceptance

Unit tests:
- `t=0` returns `from`, `t=1` returns `to` exactly.
- Ease is monotonic; Bezier endpoints are 0/1.
- Slerp takes the shortest arc; no quaternion sign flip.
- Log distance interpolation is monotonic for zoom in/out.
- Retarget mid-transition uses the sampled pose, not the old target.
- Speed multiplier 0.5× doubles duration; 2× halves it.
- Wheel accumulator total factor preserved across frames.
- Direct drag applies exactly the pointer delta (within epsilon) over one frame.

Manual/log:
- 125 stone and 729 scenes: during a continuous drag/scroll, `dt_ms` 33.3 ± 2 ms, no gap-derived
  outliers; `prepare/draw` under budget.
- Zoom 100 → 0.5 mm: several 30 Hz frames, no teleport, no near clip.
- Hover pick does not run during motion.
- `om9-app --lib` all pass; `world_picking` 7/7.

## 8. Phases

1. **P1 — Transition skeleton + tests**: `CameraPose`, Bezier/smoothstep, elapsed sampling, snap;
   no UI wiring.
2. **P2 — Input rewrite**: direct drag; wheel accumulator → transition; speed env; repaint policy.
3. **P3 — Zoom-to-cursor**: pivot ray/plane, perspective + ortho; tests.
4. **P4 — Measure 125/729**: one audit per scene; stop if 30 Hz is even.
5. **P5 — Optional quaternion rig**: only if trackball/arbitrary orientation is required.
6. **P6 — Interactive quality**: only if measurement shows GPU/fill is the limiter.

## 9. Out of scope

- GPU instancing / LOD rewrite.
- Picking performance.
- Changing document schema or export.
- 60 Hz guarantee; 30 Hz is the acceptance target.
- Copying GPL/LGPL code. Only algorithms/architecture are reused:
  - Blender GPL-2.0-or-later,
  - FreeCAD LGPL-2.1-or-later,
  - KiCad GPL-3.0-or-later,
  - VTK BSD-3-Clause.

## 10. Risks

- Direct drag may expose RDP input gaps; if so, add a **bounded** input smoothing (max 1 frame,
  alpha ≤ 0.5) rather than a global spring.
- Retargeting a transition every wheel event can extend it indefinitely; cap total duration and
  consume pending in ≤2 steps/frame.
- Zoom-to-cursor changes pivot; document/selection must not change.
- Deep zoom close rebuild is one-time; do not rebuild on every radius change (shared close mode
  already enforces this).

## Implementation progress (2026-10-21)

Landed:

- `crates/om9-app/src/camera_rig.rs`: quaternion rig, slerp, turntable orbit, pan, log zoom,
  zoom-to-cursor. Tests: roundtrip, shortest-arc slerp, distance-preserving orbit, zoom clamp,
  cursor screen stability.
- `crates/om9-app/src/camera_transition.rs`: elapsed-time cubic-Bezier transition, slerp/log-lerp,
  retarget, endpoint exactness. Tests: endpoints, monotone ease, log zoom, retarget.
- `crates/om9-app/src/ui.rs`: direct drag (no spring), wheel accumulator → transition, zoom-to-cursor,
  `OM9_NAV_SPEED`, transition map, `set_moving` keyed on the transition.
- `camera_motion` remains compiled for its old tests but is no longer used by the viewport input.

Verification so far:

- `om9-app --lib`: 135 passed, 0 failed.
- `frame_pace`: 2 passed.
- `camera_rig` + `camera_transition` + old `camera_motion`: 15 passed.
- App built release and launched on `bong-125-whole-stone.om9` (PID 56456).

Not yet verified:

- A real manual orbit/zoom log with the new rig/transition path (synthetic input injection did not
  deliver camera events in this session).
- 30 Hz p95 and deep-zoom visual acceptance on the new path.
- Removal of the now-unused `camera_motion` module.

### UI-path evidence (2026-10-21)

- `crates/om9-app/tests/interaction_log.rs`: real right-drag through egui now records **orbit** and
  middle-drag records **pan** on the direct-rig path — 1/1 pass.
- `crates/om9-app/tests/navigation_input.rs`: a real wheel burst over the Perspective viewport records
  camera revision growth, a smaller orbit radius, and a moved pivot (zoom-to-cursor) — 1/1 pass.
- `om9-app --lib`: 135/135.
- Still pending: a real 30 Hz manual log on the 125-stone scene; synthetic OS input did not deliver
  camera events in this session.

### Perspective log: hold gap (2026-10-21)

`nav-rig-125.jsonl` (583 camera events): 93 pan frames all at 33.3 ms; orbit median 33.6 ms but p95
477.6 ms. The slow orbit frames match the gap since the previous pointer event (478↔477.6,
560↔559.8, 755↔754.5, 446↔445.7), and prepare/draw stay at 0.01/0.4 ms. Holding the button still
stopped the paced loop, so the next move inherited the pause. The input path now schedules the pacer
while a drag button is held even when the pointer delta is zero. Fresh log: `nav-hold-125.jsonl`.

### Pointer-down gap (2026-10-21)

`nav-hold-125.jsonl`: every slow frame (orbit 46-754 ms, zoom 71-2125 ms) equals the gap since the
previous input event; prepare/draw remain 0.01/0.4 ms. The loop still stopped when the pointer left the
canvas or the motion settled while the button was down. The pacer now runs whenever any pointer button
is down on the active view and resumes after re-entry. Fresh log: `nav-pointer-125.jsonl`.

## Forensics / per-view clock plan

The latest 30 Hz log analysis and the per-view `NavState`, gesture-capture and transition-retarget
plan are in [VIEWPORT-NAVIGATION-FORENSICS.md](VIEWPORT-NAVIGATION-FORENSICS.md).
