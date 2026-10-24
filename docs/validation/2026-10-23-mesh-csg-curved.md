# Mesh CSG — curved 16,384-triangle fixture repaired, 2026-10-23

Target: standalone safe Rust `om9-core`, Windows x64. Owner:
`OM9-L2-10-02` / `OM9-L3-10-02-002`, with the existing M03 utility consumer.
This is a numerical bug fix for the authored fixture from the
[scaling checkpoint](2026-10-23-mesh-csg-scaling.md). No dependency, native code,
UI command, skill business rule or publication is added.

## Cause and correction

The earlier error text `CSG feature below tolerance` was a false rejection for
this fixture. It was not evidence that the actual material feature was below
the configured distance tolerance. Instrumentation isolated source triangle
3192 and nearby triangle 11267; their normalized plane separation was
`3.2639580801855934e-8`, while normalized tolerance was only
`2.581481256071081e-9` (world tolerance `1e-8`).

The former rule used `abs(plane_distance) / 8` as a normal-probe displacement,
then rejected displacement below `2 * eps`. For this witness the displacement
was `4.079947600231992e-9`, falsely below that minimum. It conflated a safety
factor with the geometric tolerance and considered infinite supporting planes.

The corrected rule computes the actual normal-line intersection with each
candidate plane, `t = -distance / dot(candidate_normal, source_normal)`, and
checks whether the hit lies in the finite triangle. Probes remain at most
`abs(t) / 4` from the source center, capped by the same initial `8 * eps`.
Consequently both probes stay on their starting side of every limiting wall;
remote plane extensions do not create artificial barriers. A focused unit
test retains the exact witness coordinates and verifies those side signs.

The absolute geometry tolerance stays **`1e-8`**, with the same normalization,
minimum probe displacement (`2 * eps`), position welding, triangle intersection,
degenerate-area and output embedding checks. Default input/work/output budgets
stay **25,000 / 20,000,000 / 50,000**. No face is deleted to evade the error,
no small fragment is skipped, and no hole-fill fallback is invoked. Internal
material fragments are still excluded only through the existing solid
classification, as required for a boolean boundary.

A separate unit regression still refuses a finite wall with inadequate probe
clearance; the guard was not removed. Another verifies that an infinite plane
belonging to a remote finite triangle does not constrain an empty probe segment.

## Original fixture and geometry evidence

The source generator is unchanged: two unit-radius octahedra, each subdivided
five times and projected to the sphere after every subdivision, with centers
`(0,0,0)` and `(0.47,0.13,0.09)`. Each has 8,192 triangles, total **16,384**.
Assertions operate on the polygonal source solids, not analytic sphere volume.

At the default options, the fixture now passes:

- Union, intersection and A-minus-B: closed oriented edge incidence, kernel
  vertex-fan validation, unchanged geometric embedding validation, and independent
  signed-tetrahedron volume conservation.
- Single-input `repair_self_intersections` over the joined crossing shells:
  closed output with **15,724 triangles**, same volume as the separate-operand
  union, and **11,176,806 work units**, below the unchanged 20M cap.
- Per-corner affine UV charts with distinct face offsets: all generated corners
  interpolate correctly and each output triangle retains its chart/seam.
- Both operands and the joined repair source remain equal to their pre-operation
  Rust Mesh snapshots; no partial source mutation.

| Measurement | Observed |
| --- | ---: |
| Union volume | 5.70713209062495 |
| Intersection volume | 2.65811845179003 |
| A-minus-B volume | 1.52450681941747 |
| Self-overlap repair volume | 5.70713209062495 |

Release union time for this fixture: **290.389 ms**. The complete four-operation
regression (union/intersection/difference plus repair with UV checks) takes
1,159.752 ms in this single local sample, including its independent checks and
fixture-side UV preparation. These timings are not a latency guarantee.

Volume identities use the previous `1e-7` absolute assertion bound; primary UV
checks retain `1e-8`. The independent checks do not replace the kernel's closed
topology/intersection checks. The previous negative fixture becomes a normal
positive geometry regression; the explicit release curved benchmark includes
this scale again, with its original parameters and tolerance.

## Execution and remaining limits

Build at repository root with `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`.

| Check | Result |
| --- | --- |
| Full core regression | 514 passed, 0 failed, 2 explicit benchmarks ignored |
| Release mesh CSG suite with ignored benchmarks included | 17 passed, 0 failed |
| App utility regression | 5 passed, 0 failed |
| Core Clippy, library and tests, warnings denied | Exit 0 |
| Rustfmt on the kernel, spatial index and geometry tests | Exit 0 |

The combined core/app Clippy invocation found an unrelated `needless_return` in
`crates/om9-app/src/session/solid.rs:596`; that file was not edited for this fix.
Core Clippy is executed separately with warnings denied. The combined failure
is retained in `clippy.log`, not represented as a passing check.

The [receipt](../../implementation/evidence/AFTER/mesh-csg-curved-2026-10-23/receipt.json)
records full core/app regression, all release mesh CSG tests including the two
explicit benchmarks, Clippy, rustfmt, source fingerprints, source audit and
artifact pruning. The diagnostic witness and raw positive geometry measurements
are retained beside the receipt.

This closes the recorded 16K fixture failure, not arbitrary mesh robustness.
The kernel still requires closed oriented triangle input and uses `f64`;
genuinely insufficient clearance, inconsistent topology, ambiguous classifications
and exhausted budgets remain explicit errors. Adaptive exact predicates,
production-model coverage, open-mesh repair, native UI/SDK commands and macOS
execution are outside this fix. Manual gates remain unsigned.

This restores the already specified repair semantics, so it does not introduce
a new reusable business rule or approve the earlier pending skill proposal.
