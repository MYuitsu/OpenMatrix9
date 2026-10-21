# 500k Object Level and Interaction Log (2026-10-20)

Scope: standalone Rust OM9 viewport performance work requested by the user on 2026-10-20: "thêm mức
500 000 object ... để tạo ra 1000 cái bông tai, ghi log lại khu xoay zoom move để có log phục vụ cho
tối ưu".

## Budget level

- `crates/om9-app/src/lib.rs::document_limits()` keeps the product limits by default and raises them
  only for an explicit stress run. The named level is `OM9_STRESS_LEVEL=500k` (500,000 objects and
  500,000,000 points); individual variables still win:
  - `OM9_MAX_OBJECTS` (default 100,000, clamp 1..2,000,000) — the 500,000 level is
    `OM9_MAX_OBJECTS=500000`.
  - `OM9_MAX_POINTS` (default 2,000,000, clamp 1..500,000,000).
  - `OM9_MAX_BYTES` (default 256 MiB, clamp 1 MiB..64 GiB).
- `MatrixApp::from_file` and `MatrixApp::new` use `document_limits()`, so the override reaches document
  restore, transaction admission and scene capture. The default behaviour is unchanged when no
  variable is set.

## 1000 earring copies

`crates/om9-app/tests/stress_instances.rs` (ignored by default) now has `OM9_STRESS_WHOLE=1`: the whole
imported earring (42 objects) becomes one block definition and N `Geometry::Instance` entities carry
different placements (lattice position, spin, tilt, scale, jitter).

Generated project: `build/stress-27000/bong-1000-whole.om9`, 7,717,406 bytes, 1,000 instances,
42 members each = 42,000 scene objects, 72,492 triangles per occurrence (72.5 million triangles).

Run:

```bat
set OM9_STRESS_LEVEL=500k
set OM9_INTERACTION_LOG=build\stress-27000\interaction.jsonl
L:\om9-target\release\om9.exe --ui "build\stress-27000\bong-1000-whole.om9"
```

Measured on the RTX 3060 / 1600x1000 window:

| Phase | Result |
| --- | --- |
| Scene build after open | about 55 s with the window "Not Responding" |
| Steady state, 4 viewports | 138 fps / 7.3 ms (92 fps earlier, 226 fps transient) |
| Private memory | ~9.3 GB |
| Effective limits in the session log | objects 500,000, points 500,000,000 |

VRAM versus RAM, measured while that scene was open:

- Windows `Get-Counter '\GPU Process Memory(*)\Dedicated Usage'` for the app PID:
  **8,456,536,064 bytes ≈ 8.06 GB dedicated VRAM**. `nvidia-smi` showed 9,668 MiB used of 12,288 MiB
  with the other desktop applications, 2,446 MiB free.
- The "~9.3 GB" figure is **Windows private commit (CPU), not VRAM**: `PrivateMemorySize64` 9,289 MB
  while the resident working set was 1,188 MB.
- The current viewport path builds one full `GpuScene` per viewport, so the 1,000-earring geometry is
  uploaded four times (one per visible view). That is why the dedicated VRAM is about four times the
  single-copy vertex/index arithmetic.

## 8000 earring copies (ceiling found)

`build/stress-27000/bong-8000-whole.om9`: 8,000 instances x 42 members = 336,000 scene objects,
579,936,000 triangles. The document loads (session_start records 8,000 entities), but the first frame
that uploads the document scene ends in:

```
wgpu error: Out of Memory
```

With or without `OM9_MAX_OBJECTS=500000`, because the failure is the GPU vertex/index budget of the
per-object scene path, not document admission. The 1,000-earring scene already reserves 8.06 GB of
dedicated VRAM, and the path builds one `GpuScene` per viewport, so eight times the earrings would ask
for roughly 64 GB of VRAM rather than the 11.4 GB single-copy arithmetic. Removing that ceiling needs
GPU instancing (one mesh per definition member, one instance-transform buffer) plus one shared scene
across the four viewports, or a display LOD; a larger document budget cannot fix it. This is the
optimization target the log is meant to measure.

## Interaction optimizations (2026-10-20)

`OM9_CAMERA_AUDIT=<frames>` orbits, then zooms, then pans the Perspective view one step per frame and
logs each frame, so interaction cost is measured without OS input. 120 frames per run:

| Configuration | Orbit median | Zoom median | Pan median | Notes |
| --- | --- | --- | --- | --- |
| Before, LOD 1200 | 16.7 ms | **707.2 ms** | 16.7 ms | zoom rebuilt the overlay every frame |
| Full detail, after the zoom fix | 19.3 ms | 19.2 ms | 19.1 ms | 52 fps steady |
| LOD 1200, after the zoom fix | **16.7 ms** | **16.6 ms** | **16.7 ms** | 60 fps, p95 17.2 ms |

Two changes produced this:

1. **Zoom no longer rebuilds the line overlay.** The scene fast path compared the world-space point
   size, which changes on every zoom step, and then rebuilt every grid/curve/point line — 13,000 curve
   segments for 1,000 earrings, about 700 ms per frame. The overlay is now rebuilt only when the scene
   actually has `Points` objects and their size moved by more than 5%; curves and the grid are
   independent of the zoom. Selection changes still rebuild the overlay as before.
2. **Display LOD (`OM9_VIEWPORT_LOD=<triangles>`).** Each unique source mesh is decimated once with the
   QEM simplifier and every occurrence reuses it; the document and the BRep source stay untouched. At
   `OM9_VIEWPORT_LOD=1200` the 1,000-earring scene drops from 8.06 GB to **4.55 GB dedicated VRAM** and
   from 9.3 GB to **5.2 GB private commit**, and all three gestures hold 60 fps.

The first frame after a camera change can still include the initial scene build (4.4 s at LOD, 6.7 s
full detail); that is startup, not steady interaction. Loading the 1,000-earring scene still takes about
55 s because the scene is built on the UI thread; GPU instancing and an off-thread build are the next
optimizations.

Run the smooth configuration with:

```bat
set OM9_STRESS_LEVEL=500k
set OM9_VIEWPORT_LOD=1200
set OM9_INTERACTION_LOG=build\stress-27000\interaction.jsonl
L:\om9-target\release\om9.exe --ui "build\stress-27000\bong-1000-whole.om9"
```

`OM9_CAMERA_AUDIT=120` added to that command reproduces the table and closes the window when done.

## Interaction log

`crates/om9-app/src/interaction_log.rs`: `OM9_INTERACTION_LOG=<path>` appends one JSON object per line.
Nothing is written when the variable is unset; a write error never interrupts a gesture.

- `event:"session_start"`: document, entity count, the three effective limits.
- `event:"camera"` for orbit / pan / zoom: `view`, `action`, `dx`, `dy`, `wheel`, `frame_ms`,
  `revision`, `eye`, `target`, `scene_objects`, `visible`, `culled`, `draw_calls`.
- `event:"gumball"` for move / rotate / scale: `view`, `action`, `frame_ms`, `selected`.

`tests/interaction_log.rs` drives a right-drag and a middle-drag through egui over the Perspective
viewport and asserts the `orbit` and `pan` records, their frame time, revision and camera. Result:
1 passed. The log path is verified from the real widget reducer, so a native drag writes the same
schema.

Real OS mouse automation from the agent session was not reliable: the app window competed with the
foreground Telegram window, and `SetForegroundWindow`/`mouse_event` did not deliver a drag to the
canvas even after the window was raised. The widget-path test above is the automated proof; the native
drag remains a manual step for the user.

## Evidence

- `build/stress-27000/bong.om9` (imported 3DM), `bong-1000-whole.om9`, `bong-8000-whole.om9`.
- `build/stress-27000/bong-1000-whole-view.png` (1000 earrings, 92 fps).
- `build/stress-27000/whole.stderr.log` (8000 earrings: wgpu Out of Memory).
- `build/stress-27000/interaction-1000-whole.jsonl` (session_start only: native drag was not
  delivered), `tests/interaction_log.rs` (orbit/pan records proven).
