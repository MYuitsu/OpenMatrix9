# Shared selected viewport lines, 2026-11-04

Selected mesh/curve lines, their original-position ghost and selected shape guides now join the window's shared scene-line owner. Per-view grid and point markers remain independent. The same scene/selection/visibility/sample signature controls invalidation; the selected and guide batches receive the current Gumball preview transform, while the ghost remains at the original placement. Their GPU handles participate in existing deduplicated live, worker and in-flight tracking. Geometry sources, modeling precision and STL/3DM export are unchanged.

## Controlled GPU evidence

A synthetic 64×64 grid mesh with 4,225 vertices and 8,192 triangles was selected and rendered in four offscreen viewports using the same GPU, document and camera. A reconstructed previous per-viewport path held **11,028,416 logical overlay bytes**; the shared path held **2,770,880 bytes**, including 2,752,512 bytes for the shared selected/ghost buffers. This is a reduction of 8,257,536 bytes (about 74.9%) for this controlled fixture, not a physical VRAM measurement or guarantee for all scenes. Its complete Perspective images matched pixel-for-pixel.

The mixed mesh/curve/point Perspective comparison also matched every RGBA pixel between the previous and shared paths for Wireframe/Shaded, mesh/line/point selection, hide/Undo/Redo, orbit/pan/zoom and a visible selected-mesh translation preview including its original-position ghost. This is one fixture and one preview transform, not proof for arbitrary B-reps, transparency, rotations, scaling or every intermediate desktop gesture. A four-viewport test separately verifies selected and ghost buffers share the same owner and are counted once.

Default-feature E: debug checks with `-j 2`: 24 serial GPU/egui Gumball tests, 28 serial GPU viewport tests and the subsequent selected-mesh allocation fixture passed without adapter skips. No process/physical VRAM peak, desktop pointer/display latency, complete GPU budget enforcement or over-budget batching has been measured or completed.

## Release replay after the change

The current release built in 6m16s with `-j 2`, executable SHA-256 `441bc7e97e83bca27bcb39c336d1867f5c190af45c92a5e8e19f9a58d136c908` and navigation source fingerprint `90691fc52906aa76a6058dbae425656528626adab56fc8a901e4fca05a7d98ec`. Four hidden synthetic replays used RTX 3060/Vulkan, four views, simulated DPI 1, 30 Hz, 90 requested frames and two repetitions per fixture. They select no objects and therefore exercise the ordinary-scene path, not the selected-mesh saving measured above. All four exited cleanly, fixtures remained unchanged, and diagnostics report zero errors and dropped records. The earlier matching release comparison is in [unselected evidence](2026-11-04-shared-viewport-world-lines.md).

| Objects and repetition | App frame interval p50 / p95 / p99, ms | Sampled maximum logical overlay bytes |
| --- | --- | --- |
| 125, 1 | 33.316 / 34.314 / 44.138 | 65,174,624 |
| 125, 2 | 33.326 / 49.980 / 66.263 | 65,174,624 |
| 729, 1 | 33.318 / 40.285 / 50.103 | 372,741,744 |
| 729, 2 | 33.310 / 33.855 / 49.771 | 372,741,744 |

The earlier unselected release runs had 125-object p95 34.548/36.952 ms and 729-object p95 33.885/33.715 ms. The larger p95 in one current 125 run and one current 729 run are observations requiring investigation if reproducible; these short, non-interleaved runs do not establish a causal regression or prove equal performance. The overlay maxima match the earlier runs. App frames are not presentation times, GPU completion is not screen presentation, and simulated input/DPI do not establish desktop feel. Raw summary and logs: `build/standalone-validation/ui-artifacts/gpu-shared-selected-release-20261104/summary.json`; earlier comparison: `build/standalone-validation/ui-artifacts/gpu-shared-world-release-20261104/summary.json`.
