# Length rejects mixed display segments — 2026-11-12

In the previous selected-edge Length path, only the first displayed segment
with the selected edge ID was matched against source topology. The regression
uses a cylinder with a segment from its other circular edge appended under the
first ring's display ID. The two source rings have equal length, so comparing
measured lengths would not prove identity. Before the change, measurement
incorrectly succeeded in the in-memory fixture. The new check rejects the
same type of mixed edge after save/open, without
changing the document or the existing Redo; the selection measurement handler
also rejects it.

The OCCT bridge now reads the source once and resolves every segment carrying
the selected display ID against the same native edge map. It requires each
segment to identify the same unique source edge before measuring that edge.
It checks display indices and limits, borrows the Rust arrays for the
synchronous call, and does not create a persistent native handle. The original
single-segment kernel API remains available. The prior user's approved rule
already requires refusal when source identity cannot be verified; this is a
correction to that rule, not an added skill rule.

On Windows x64 using `CARGO_TARGET_DIR=L:/om9-target` and `-j 2`, the two app
measurement suites passed 8 and 11 tests, respectively, including existing
reopened reordered Box and curved Cylinder fixtures, two display profiles,
placement, and the new mixed-segment refusal. The kernel library passed 33
tests with three ignored. The debug executable built and the public source
audit reported no errors. Artifact pruning removed no runs; 169 cited old
runs were retained by the existing tool.

Verification of each segment may take longer for detailed display meshes; no
large-model latency, physical desktop click, native peak memory, exhaustive
trim/import topology or all possible corrupted displays have been measured.
The segment witness and source matching remain numerical checks, not a proof
of global geometric equivalence.

## Bounded segment timing

The ignored `selected_length_segment_verification_timing` probe compares the
same current bridge with a single segment and with every displayed segment of
a radius-2 mm cylinder ring. Windows x64, debug, `L:/om9-target`, `-j 2`;
five paired measurements per deflection, single first each time (milliseconds):

| Deflection mm | Segments | Single segment | All selected segments |
| ---: | ---: | --- | --- |
| 0.1 | 10 | 0.976, 0.919, 0.913, 0.910, 0.912 | 1.515, 1.500, 1.515, 1.514, 1.526 |
| 0.01 | 32 | 1.161, 1.157, 1.183, 1.161, 1.171 | 5.635, 5.666, 5.676, 5.679, 5.619 |
| 0.001 | 100 | 3.335, 3.380, 3.379, 3.385, 3.392 | 80.435, 79.998, 80.089, 80.106, 80.333 |
| 0.00001 | 994 | 26.174, 26.921, 27.251, 27.102, 26.590 | 2479.335, 2525.817, 2524.650, 2538.244, 2525.510 |

All measurements returned the analytic circumference. These call timings
include Rust B-rep validation and native source parsing; they exclude UI and
scene setup. They are not a pre-fix benchmark, a controlled performance
comparison, release or desktop latency, nor representative of arbitrary CAD
models. The 0.01 mm profile used in the previous Sphere B0 experiment is only
a context reference, not a claim that every current viewport uses it. The
0.00001 mm case shows a real risk of noticeable synchronous delay for very
fine edges. Keep the identity checks; any optimization must preserve their
ambiguity, curved-segment and memory safeguards. Current
`measurement_bridge.cpp` SHA-256:
`89bb430009f8baf457f6966df6e255a1601b9b0ebb6d8d68fefe5730f6e8c80e`.
The source audit passed; pruning retained cited old runs and removed none.

## Complete-circle chord bound

The shared OCCT edge resolver now has a bounded shortcut for complete circle
edges when `max_nodes >= 4096`. After the same endpoint/source and uniqueness
checks, it bounds the distance of the whole display segment by its maximum
endpoint deviation plus the sagitta of the chord between projected circle
points, with a floating-point margin. Only a bound strictly below the pick
tolerance avoids subdivision. Partial arcs, near-diameter chords, tight node
budgets, nonfinite or insufficient bounds use the existing subdivision path.
This is a numerical calculation with a margin, not interval arithmetic or a
proof of OCCT projection accuracy. No edge list is sampled or skipped.

The same debug probe, same machine and target, after the change measured
all-segment calls (ms):

| Deflection mm | Segments | Five all-segment samples |
| ---: | ---: | --- |
| 0.1 | 10 | 0.883, 0.869, 0.870, 0.870, 0.886 |
| 0.01 | 32 | 1.125, 1.122, 1.125, 1.133, 1.130 |
| 0.001 | 100 | 2.968, 2.943, 2.937, 3.024, 3.021 |
| 0.00001 | 994 | 27.357, 27.544, 28.083, 27.603, 27.601 |

The earlier and later runs were sequential and not interleaved. These are
kernel call timings on one cylinder, not release or desktop timings or a
general latency guarantee. The shortcut does not change the numerical
identity, endpoint, ambiguity or foreign-segment checks.

The full-circle regression tests 12 angles at tolerances 5% above and below
the analytic sagitta, plus an overlong chord; below-bound and diameter cases
are refused. The existing tight-budget, ambiguous-edge, trimmed-edge and
mixed-display regressions also passed. `wireframe` passed 57 with five ignored,
`measure_extended` passed 11 with one ignored, the Solid input group passed
102, and Fillet and Sphere AroundCurve simulated viewport subsets passed five
each. Debug build and public audit passed. Native peak, actual desktop, other
curve representations and full floating-point error bounds remain open.
`editing_bridge.cpp` SHA-256:
`0c4241379e9c8a67c0abc7cf7cab523de3a7485e6cc1cf6399d820653ac631d8`.

## Trim and large-coordinate checks

Two additional source-resolver regressions passed. A shell made from an
analytic quarter-circle accepts a segment on its retained arc and rejects a
segment on the absent part of the underlying full circle. Separately,
analytic cylinder rings translated to local source coordinates of 1e6 and
1e9 mm accept an unaltered segment and reject a 0.01 mm shifted endpoint.
These are individual numerical fixtures; they do not prove all trimmed arcs,
locations, tolerances or floating-point bounds. The complete `wireframe`
suite passed 59 tests with five ignored, and the public source audit passed.
No product code or desktop session was changed in this follow-up.
