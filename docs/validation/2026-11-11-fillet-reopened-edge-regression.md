# Open regression: FilletEdge selected B-rep edge after v4 reopen

## Mixed displayed edge after reopen — 2026-11-12

A reopened Box fixture assigns a second source edge's segment the same
display ID as the first. Before the correction, staged FilletEdge accepted
the selection using only the first segment. The shared Fillet display witness
helper now verifies all segments for the selected ID resolve uniquely to the
same source edge before returning the first witness. The fixture now rejects
both the staged selection and direct numeric route while preserving the
document and existing Redo. The existing source-map checks at endpoint,
projection, contour and commit remain in place.

`cargo test -p om9-app --lib fillet_ -j 2` passed 11 tests; the simulated
viewport Fillet subset passed five. `cargo check -p om9-app
--no-default-features -j 2`, the debug executable build and public source audit
passed. The new all-segment check adds synchronous native source reads and may
run repeatedly during Fillet interaction. Its latency for detailed geometry,
native peak, other topologies, physical desktop behavior and all forms of
memory pressure have not been measured. This is not a general certification of
edge identity or a guarantee that every valid edge will be accepted.

The sections below record the earlier reordered-edge work, not the current
status of the mixed-display fixture.

The explicitly ignored controller regression `fillet_reopened_edge_matches_selected_display_edge` saves and reopens a Box with consistently permuted stored display/topology edge order. Its selected display edge has endpoints `[0, 0, 6]` and `[0, 5, 6]`, but `solid_fillet_edge_endpoints` returned `[0, 0, 0]` and `[0, 0, 6]`. Running this ignored test reproduced the failure. This directly exercises the endpoint helper called by the product edge-pick path; a full incorrect Fillet commit was not performed by this fixture.

The fixture was extended through the product selection, preview and commit: resulting volume is `119.67809724509613 mm³`, versus `119.73174770424677 mm³` from the source-side oracle filleting the selected edge at radius 0.5 mm. Thus this fixture demonstrates an incorrect committed result, not only incorrect endpoint feedback. The oracle is a separate native source read, not a general proof of cross-read map stability; it establishes the expected edge for this controlled Box fixture.

The same stored edge offset is also supplied to native preview construction, radius-control projection and shared-endpoint mapping. Correcting endpoint feedback alone would leave actual geometry vulnerable. Next implementation must bind each selected edge to the source in the same native shape/map used by each relevant operation, or carry an independently verified binding whose lifetime and validity cover that operation. Preserve radius laws, contour and orientation semantics; refuse ambiguous/missing/stale input without altering the document or Redo. Exercise one and multiple edges, controls, preview/commit, Undo/Redo and failure on reopened permuted fixtures. No fix or desktop behavior is claimed.

## Scoped correction, 2026-11-11

The source-edge resolver is shared with DupEdge. Fillet now resolves each selected display segment on the same OCCT shape/map used for endpoints, projection, shared contour endpoint checks and preview/commit. The direct numeric route uses the same source verification. The reopened Box regression now returns the selected edge's endpoints and midpoint projection, commits the oracle volume `119.73174770424677 mm³`, and preserves it through Undo/Redo; the test is enabled. A second reopened permutation covers two edges with different radii and analytic volume, and the tangent-contour variable-radius test now also reopens a permuted fixture. A displaced display segment is refused with document and Redo unchanged.

The `fillet_` library batch passed 9 tests, including closed edges, controls and contour behavior; five `solid_difference_ui` Fillet viewport tests and one numeric `solid` integration test passed. The shared `wireframe` suite passed 36/3 ignored and `dup_border` passed 9. A no-default-features check passed with existing warnings. These are scoped fixtures, not proof for all imported/trimmed edges, degenerate or ambiguous topology, in-source local placements, desktop or native/process peak. Verification and OCCT operation remain synchronous, and native allocations are not bounded by the Rust-managed reservation.
