# PipeChain: selected source edge after v4 reopen

A product-controller fixture saved and reopened a Box with a valid permutation of its display/topology edge order. Before the fix, `solid_pipe_pick_edge` interpreted the stored edge index as an index in a freshly read OCCT source and cached the endpoints of a different edge. The regression was run red, then passed after the change.

The caller now validates the scene reference and compares a segment from the selected display edge against a unique edge in the current native source, using the existing admitted segment verification adapter. It accepts only the degree-one, two-pole NURBS returned for a straight edge, evaluates its endpoints and applies entity placement. Ambiguous, missing, mismatched or non-straight source edges are refused. Source capture and Rust-side output/scratch use the existing coordinator; native OCCT allocations remain outside its bound.

The reopened permutation fixture checks the cached endpoints against the selected display edge, then completes a Pipe and checks its analytic volume, one Undo and Redo. A second fixture offsets the displayed segment away from its source and verifies refusal with document and Redo unchanged. Two focused controller tests passed; five `viewport_pipe_chain` tests passed, including actual geometry, mixed chains, right-angle chains and stale-source refusal. These do not establish all possible imported edges, numerical configurations, non-rigid placements, native peak or desktop behavior. The selection is synchronous and can refuse when display does not meet the strict source tolerance; no automatic reconstruction of the view was added.

## Mixed displayed edge after reopen — 2026-11-12

A Box fixture assigns one segment from a second edge to the first edge's
display ID, saves and reopens the document, then selects that ID for
PipeChain. Before this correction the controller accepted it because it
checked only the first segment, so a click on the foreign segment could select
the wrong source. After correction the controller verifies all segments with
that ID resolve to the same unique source edge before extracting the existing
straight source curve. It rejects the fixture and keeps the drawing, pending
chain and existing Redo. Source capture and output admission remain in their
existing path. This performs an additional synchronous source read; its native
allocation peak and latency for detailed or complex sources were not measured.

With `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`, the focused three PipeChain
controller tests passed, the encompassing Solid input group passed 100 tests,
and five simulated viewport PipeChain tests passed. The debug executable built
and the public source audit reported no errors. This establishes the tested
Box case and the existing valid fixtures, not every display corruption, source
topology, actual desktop click, memory pressure or general affine placement.
