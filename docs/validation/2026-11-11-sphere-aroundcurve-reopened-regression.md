# Open regression: Sphere AroundCurve source edge after v4 reopen

## Mixed displayed edge after reopen — 2026-11-12

An authored Box has one edge's display segment added under a different edge's
display ID, then is saved and reopened. Before the correction, selecting the
foreign segment was accepted because only the first segment was checked. The
shared AroundCurve source resolver now verifies that all displayed segments
under the selected ID resolve uniquely to the same source edge before it
extracts the analytic curve. Both the viewport-facing selection handler and
typed `Path=box.EdgeN` reject the fixture, keeping the drawing, pending first
selection stage and existing Redo. The existing valid straight and circular
source fixtures remain successful with their center and Undo/Redo checks.

Using `CARGO_TARGET_DIR=L:/om9-target` and `-j 2`, the Sphere-filtered app
library batch passed 53 tests with one ignored, including four AroundCurve
regressions; five simulated viewport AroundCurve tests passed. The debug
executable built and the public source audit reported no errors. This is not
desktop evidence, and other source geometries, memory pressure, native peak
and latency have not been exhaustively tested. The additional source read and
all-segment verification are synchronous and may repeat during the command;
very fine edges can take substantially longer, as measured separately for
[Length](2026-11-12-length-mixed-display-segments.md#bounded-segment-timing).

The sections below describe the earlier reordered-edge work rather than the
current status of the mixed-display fixture.

The controller test `sphere_aroundcurve_reopened_edge_tracks_selected_source` constructs a Box with consistently permuted saved topology/display edge indices, saves and reopens it, selects the displayed edge twice to set the path and center, and creates a radius-one Sphere. The fixture failed: the selected midpoint was `[0, 2.5, 6]`, while the created Sphere center was `[0, 0, 6]`. The test is explicitly ignored in the normal batch until the underlying bug is corrected; running it with `--ignored` reproduces the failure.

`solid_sphere_pick_brep_edge` resolves the selected stored UUID to an index and `sphere_curve_source` interprets that index in a newly read native map. The path is reconstructed again when a world point or `OnCurve=` chooses the center, so fixing only initial selection is insufficient. The typed `Path=Object.EdgeN` route also calls the same index-based helper and needs a separate contract check. Source stamp validation alone does not establish edge identity across the two maps.

Next: use source-verified selected geometry through every center-selection route, retain and account its lifetime, and validate source/placement before commit. Reject ambiguous or stale matches and preserve pending inputs/document/Redo on refusal. Cover straight and circular trimmed edges, typed and viewport routes where applicable, saved/reopened source, placement and Undo/Redo. Do not infer equivalence of native and persisted indices or approximate a CAD curve by its viewport mesh. No fix or desktop validation is claimed here.

## Scoped correction, 2026-11-11

The previously ignored Box fixture now passes for viewport edge and center picks, viewport pick followed by `OnCurve=0.5`, and typed `Path=box.Edge1` followed by `OnCurve=0.5`; all three verify the center and Undo/Redo. Every center operation rechecks the captured source stamp and resolves the selected display segment against exactly one source edge. The native selection and extraction use the same edge map. The resulting source curve and its Rust-side reservation stay together while calculating center; native OCCT peak remains unbounded by that reservation. An intentionally displaced display edge is refused while keeping the document, Redo and pending selection stage.

An initial NURBS-only conversion changed the angular position for `OnCurve=0.25` on a circular edge by about 0.077 mm in a radius-2 fixture. The adapter now returns the source circle's exact frame and angular domain from the same selected native map, preserving the previous circular parameterization; the reopened circle fixture compares its result with the independent pre-existing circular-edge oracle. Only this full-circle case is covered; other trimmed and imported curves, non-rigid placement, memory-pressure at each stage and desktop remain unverified. An unverifiable/ambiguous display is refused and no source-view rebuild is performed.

After correcting an unrelated stale assertion that expected newly created Box to have no topology proof, the `sphere_` library filter passed 28 tests. Five `viewport_sphere_aroundcurve` tests passed. The shared-adapter `wireframe` suite passed 36 with three diagnostics ignored; `dup_border` passed 9. A no-default-features `cargo check` passed with existing warnings. `--exact` was once mistakenly invoked with an incomplete test name and ran zero tests; it was not counted as evidence.
