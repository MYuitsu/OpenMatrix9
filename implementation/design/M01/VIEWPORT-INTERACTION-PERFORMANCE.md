# Viewport interaction performance (2026-10-20)

Plan and measured progress for smooth orbit/zoom/pan with many objects. Full phase plan is recorded in
the session plan and summarized here; this file is linked from `OM9-L2-01-06/L2.json` and
`L3-002/l3.json`.

## Targets (RTX 3060 12 GB, 1600x1000, 4 views, 4x MSAA)

| Scene | Open | VRAM | Orbit/zoom/pan p95 |
| --- | --- | --- | --- |
| 1,000 earrings (42,000 objects, 72.5M triangles) | <= 10 s | <= 1.5 GB | <= 16.7 ms |
| 8,000 earrings (336,000 objects, 580M triangles) | <= 30 s | <= 6 GB | <= 33 ms |
| 500,000 instances/objects | <= 60 s | <= 8 GB | <= 66 ms |

## Order

1. **P1a — chunk sizing + batched uploads** (done 2026-10-20): `SceneBatch` chunks grow in
   `2/8/32/128/256 MB` classes instead of the device maximum; vertex/index writes are merged per chunk
   and flushed once in `render_into`.
2. **P1b — one shared GPU scene for the four viewports** (next): the app still builds one `GpuScene`
   per view, so the 1,000-earring geometry is uploaded four times (7.05 GB VRAM).
3. **P2 — instanced renderer + per-view LOD**: dedup `GeometryKey`, instance storage buffer, QEM LOD
   chain `100/50/25/10/4 %`, hysteresis; point markers screen-space.
4. **P3 — off-thread scene preparation + upload budget**: no UI freeze while opening.
5. **P4 — GPU cluster culling**, only if 500k still misses its gate.

Each phase stops when its gate is met; `OM9_VIEWPORT_RENDERER=auto|batch|instanced|cluster` is the
rollback switch. `OM9_VIEWPORT_LOD=auto|<triangles>` remains the display-LOD override.

## Measured

| State | Open | VRAM | Notes |
| --- | --- | --- | --- |
| Before P1a | ~55 s | 8.06 GB | full detail, one scene per view, chunk at device max |
| After P1a | **20 s** | **7.05 GB** | same scene; chunk classes + one write per chunk |
| With `OM9_VIEWPORT_LOD=1200` (before P1a) | ~55 s | 4.55 GB | 60 fps orbit/zoom/pan |
| With camera audit (LOD) | — | — | orbit 16.7, zoom 16.6, pan 16.7 ms median |

P1a did not touch geometry or shaders; the remaining 7.05 GB is the four per-view copies that P1b
removes.

## Scope update 2026-10-20

The user removed 500,000 from the test scope: it is the theoretical object-budget ceiling, not a
workload to certify. The practical ceiling is **27,000 copies**: open 3 s, VRAM 0.55 GB, private
984 MB, orbit/zoom/pan p95 18.8/17.1/17.0 ms (auto LOD, shared scene, chunked geometry/overlay).
8,000 earrings (336,000 scene objects) also pass: 10 s, 1.05 GB, p95 21.4 ms. P2b instancing and P3
off-thread build are now conditional on a real workload above 27,000, not prerequisites for this goal.

## Accuracy policy for jewelry (2026-10-20, approved)

Display LOD is a rendering detail, never a document change. Accuracy rules:

- Screen-space error budget: gem and prong <= 0.5 px; shank/large body <= 1 px; selected, hovered and
  gumball objects always LOD 0.
- Do not decimate when the projected error exceeds the class budget, even if the global triangle budget
  is exceeded: accuracy is hard, the triangle budget is soft.
- Lock open boundaries, cluster-group boundaries and sharp edges (> 30 degrees) during QEM; preserve
  gem facet planes (normal deviation <= 1 degree) and thin components (< 0.5 mm).
- `OM9_VIEWPORT_LOD=auto` remains a fallback for the legacy `SceneBatch` path only. The target renderer
  chooses LOD per instance from cluster `error_world` and the per-class pixel threshold.
- Picking, selection and export keep the full-resolution CPU scene.

Verification: full-detail versus LOD image diff at 100% zoom (<0.1% pixels), fit view (<1%), zoom matrix
(fit/2x/8x/32x), gem facet count/normal error, prong-tip silhouette <= 0.5 px, no seam, and a scene
translated 1,000,000 mm for f32 precision. Evidence goes to `build/stress-27000/accuracy/`.

## Camera navigation plan (2026-10-21)

The Blender/FreeCAD/KiCad/VTK comparison and the rewritten camera-navigation plan are in
[CAMERA-NAVIGATION-OPTIMIZATION.md](CAMERA-NAVIGATION-OPTIMIZATION.md). It replaces the exponential
spring with fixed-duration transitions (elapsed-time Bezier), direct drag, wheel accumulation, and
zoom-to-cursor, while keeping the 30 Hz pacer and the final 60 Hz ceiling.
