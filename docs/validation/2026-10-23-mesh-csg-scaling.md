# Mesh CSG scaling follow-up — 2026-10-23

Historical scaling checkpoint. The later [16K curved-fixture fix](2026-10-23-mesh-csg-curved.md)
resolves the specific refusal recorded below, at the same tolerance and budgets.
The original logs and timings in this checkpoint remain unchanged.

Standalone safe Rust `om9-core`, Windows x64. Owner remains AFTER
`OM9-L2-10-02` / `OM9-L3-10-02-002`. This is a performance follow-up to the
[initial kernel](2026-10-23-mesh-csg.md), preserving nonzero winding, tolerance,
UV seams and checked-output semantics. It does not add a UI/SDK command or sign
the manual gate.

## Implementation

The original implementation split each triangle against every triangle's
supporting planes, summed every solid angle for each probe, scanned all vertices
for every reconstructed edge, then tested all output triangle pairs.

The follow-up introduces deterministic median-split BVHs in safe Rust:

- Bound queries identify candidate cutters; actual triangle contact is checked
  before making local cuts. Coincident overlays have one source owner.
- Signed ray crossings classify closed oriented solids using BVH candidates;
  edge/parallel ambiguity retries three directions, then uses the original
  solid-angle classification. The signed sum preserves winding multiplicity,
  inward shells and cavities; it is not even/odd fill or a far-field approximation.
- Vertex bound queries find T junctions, and triangular conforming cells stay
  triangles rather than each becoming a three-triangle centroid fan.
- Output embedding validation queries overlapping bounds, then runs the same
  geometric predicate. Closed edge incidence and vertex-fan checks are retained.

BVH build/traversal and primitive tests consume the work budget. Dense overlapping
bounds/intersections can still approach quadratic work and fail explicitly.
Ray tests, fallback calls and candidate-pair counts are now included in CsgReport.

## Defaults and measurement scope

The input admission limit increases from **2,000 to 25,000 total triangles**.
The work budget increases from **10 million to 20 million units**; live-fragment
and output-triangle limits remain **50,000**. This admits input for an attempt,
not a promise every 25K mesh fits the other geometric/resource constraints.

Measurements use `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`, release optimization,
one test thread, and authored fixtures. Planar timings cover the kernel call plus
the independent edge/volume check (both before and after optimization); curved
timings stop after the union call, before the extra intersection/volume checks.
Fixture construction and compilation are excluded.
This is a local measurement, not production-model certification or a latency SLA.

The baseline raised only its triangle admission cap to 30K so that the old
algorithm could be measured past its 2K default; its 10M work cap remained active.
The intermediate optimized run with that same 10M work cap passes 6,144 triangles
at 2,967,892 work units. Thus the improvement is not solely a larger budget.
The 24,576-triangle result uses the new 20M default budget.

| Input triangles, subdivided boxes | Baseline | Optimized default | Output triangles |
| ---: | ---: | ---: | ---: |
| 24 | 2.117 ms | 0.597 ms | 56 |
| 384 | 55.041 ms | 6.133 ms | 336 |
| 1,536 | Work budget exceeded | 24.482 ms | 1,344 |
| 6,144 | Work budget exceeded | 102.150 ms | 5,376 |
| 24,576 | Work budget exceeded | 426.373 ms | 21,504 |

Curved union samples: 1,024 triangles in 23.321 ms; 4,096 in 79.939 ms.
These are single local samples, not statistical performance bounds.

For both boxes, each refinement preserves the same exact closed solid, volume
8 per operand, offset (1,1,1), union volume 15. Refinements weld shared midpoints.
All accepted outputs independently check oriented edge incidence and signed
tetrahedron volume. Kernel validation also checks vertex fans and geometric
embedding; neither check is bypassed for large meshes.

See `baseline.log`, `optimized-trial.log`, `expanded-budget-trial.log` and the final
serial `optimized.log` in
[`implementation/evidence/AFTER/mesh-csg-scaling-2026-10-23`](../../implementation/evidence/AFTER/mesh-csg-scaling-2026-10-23).
The receipt summarizes the exact samples and source fingerprints.

## Geometry regression and numerical limit

The existing analytic box/tetrahedron, connected self-crossing pentagram, cavity,
inward winding, rotation, UV seam and atomic app-error fixtures are retained.
Additional checks compare BVH queries to exhaustive bound queries and signed
ray classification to full solid-angle sums at 1,458 probe points with crossing
positive/negative oriented shells. Unequal dense coplanar triangulations exercise
union/difference/intersection with per-corner UV seams.

Curved fixtures are intersecting independently tessellated spheres, offset
(0.47,0.13,0.09). Union and intersection at **1,024 and 4,096 input triangles**
pass closed topology and volume conservation against the two source meshes.
The expected volume is that of the polygonal sources, not the analytic sphere.

At **16,384 curved input triangles**, the default-tolerance fixture is still
unsupported: `CSG feature below tolerance`. The initial discovery run failed
and is preserved as `curved-limit-discovery.log`; this is not counted as an
accepted geometry fixture. A separate normal regression now requires that
explicit error and unchanged source meshes. The two successful curved scales
remain the positive benchmark. No smaller tolerance, discarded cell, relaxed
embedding check or hole-fill fallback was used to make this result pass.

`f64`, closed consistently oriented triangle input, colour restrictions, world
coordinate precision and tolerance guards remain in effect. Adaptive exact
predicates, near-tangent/sub-tolerance arrangement handling, open-mesh repair,
larger production fixtures and macOS remain open. Increasing the triangle cap
does not resolve these separate restrictions.

## Verification

The [receipt](../../implementation/evidence/AFTER/mesh-csg-scaling-2026-10-23/receipt.json)
records the final core regression, explicit positive benchmarks, app utility
regression, Clippy, rustfmt, source audit and artifact-pruning results. The two
release benchmarks are intentionally ignored in the ordinary test invocation
and are executed separately with `--ignored --test-threads=1`.

No dependency, native adapter, skill business rule, commit or publication was added.
The prior proposal to record reusable repair rules in a skill remains unapproved.
