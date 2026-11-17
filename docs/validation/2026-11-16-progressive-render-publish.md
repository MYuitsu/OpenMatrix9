# Progressive viewport publication, exact spheres and retained Undo

Status: renderer fix and small native checks passed. The 512/768 matrix is
incomplete: RDP disconnected during the first large per-sphere run at sphere 409.
No Boolean is included in this task's native tests.

The old trace created 506 spheres while the published Perspective frame remained
1535 with zero mesh objects. Every new scene canceled unfinished preparation and
rejected pending images. The renderer now drains one immutable presentation cohort
across active viewports when changes only append unchanged-source objects. Its
original publication key is retained; an intermediate image is never marked as
the current source. Source stamps, display-buffer identity, placement, visibility,
color and leaf revisions must match. Edits, Undo, remesh, document replacement,
selection/preview changes and source masks retain strict invalidation. Geometry,
CAD tolerance, document transactions and the Undo policy were not changed.

The GPU bridge suite passed 30 tests, including continuous sphere appends, completed
frame publication, source/camera/preview invalidation, resource accounting and
Undo/Redo. A separate exact-sphere Light/High source test passed. Three old GPU test
fixtures were updated to create canonical BReps through the real Controller.
The release builds use `L:/om9-target`, `-j 2`; no additional target was created.

The first native burst replay made images progress but grouped 2–3 new spheres.
The user rejected that visual pacing. The current **test replay** therefore waits
for all four current, full-resolution completed images after each commit before
sending the next Sphere command. It records every `sphere_viewport_presented`
acknowledgement and commit-to-present delay. The renderer still coalesces arbitrary
bursts; this test pacing is not a claim that every intermediate state of an
unlimited command stream will be displayed.

Evidence root: `build/standalone-validation/ui-artifacts/progressive-publish-20261116/`.
The accepted small replay and interrupted large attempt are under `per-sphere/`.
The per-sphere executable SHA256 is
`6c3c3e274d17dff68667eb82ec1bd22912e251829f91aed87ddb451d41027054`.

| Native fixture | Result | Orbit published FPS | Selected published FPS | Max commit-to-present | Peak RSS | Undo |
|---|---|---:|---:|---:|---:|---:|
| 16 Light | passed, every 1…16 acknowledged | 30.092 | 30.083 | 475.177 ms | 300.594 MiB | 16 |
| 16 High | passed, every 1…16 acknowledged | 30.087 | 30.079 | 457.821 ms | 311.746 MiB | 16 |
| 512 Light | interrupted by RDP disconnect, 409 acknowledged | unmeasured | unmeasured | incomplete | incomplete | 384 sampled at last resource checkpoint |
| 512 High | not run | — | — | — | — | — |
| 768 Light | not run | — | — | — | — | — |
| 768 High | not run | — | — | — | — | — |

Both small profiles passed exact sphere volume, closed-solid/naked-edge checks,
world placements, identical canonical source fingerprints and Undo/Redo. Screenshots
of both native windows were inspected while creation was in progress. The GPU
publication FPS is not measured RDP network/video delivery; small fixed-window
counts can be slightly over the 30 Hz target at interval boundaries.

![Light during per-sphere creation](../../build/standalone-validation/ui-artifacts/progressive-publish-20261116/per-sphere/native-16-light/01-light-shaded.png)

## Increasing creation cost

The interrupted 512 Light trace separates the time inside the ordinary Sphere
command from the wait after it returns:

| Sphere range | Median command | Median commit-to-present |
|---|---:|---:|
| 1–32 | 27.94 ms | 471.01 ms |
| 97–128 | 148.18 ms | 351.89 ms |
| 225–256 | 789.20 ms | 89.63 ms |
| 353–384 | 2363.39 ms | 164.94 ms |

At 384 spheres, 384 Undo steps and 768 full Undo memory inventories were logged.
The growth is primarily inside the command, while publication waits do not grow
similarly. This is not a measured per-function CPU profile, so it does not prove
that Undo inventories are the only cost.

Static tracing identifies whole-history work in
`crates/om9-core/src/transaction/undo_memory.rs`: the inventory walks saved/current
worlds and each Undo/Redo entry's before/after documents, then measures retained
document storage. Distinct growing document snapshots can require visiting roughly
1+2+…+N entity references per complete pass. `trim_undo_memory_with_reservations`
refreshes when capture ownership generation changes. It is called after publication
and again when native BRep insertion evidence is attached through
`Coordinator::retain_insert_evidence`. Other retained-world validation also runs
during commit. Repeated whole-history inventories/validation are the next profiling
target; keeping Undo does not inherently require rewalking all old data per sphere.
No Undo accounting, ownership guard, memory limit or retention rule was weakened.

Next action: with RDP Active, resume the remaining native stages using
`rtk proxy python build/standalone-validation/ui-artifacts/progressive-publish-20261116/per-sphere/run-staged.py --large`.
Retries preserve the interrupted trace in a separate directory. Large FPS/freeze/RAM
results must not be inferred from the small tests or from the interrupted attempt.
