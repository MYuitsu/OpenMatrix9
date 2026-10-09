# Exact RCORE-01 fixture oracle review — 2026-10-09

Independent review requested under `superpowers:requesting-code-review`.
Baseline: `fcdb0ac`. This review covers the native candidate and FreeCAD macro
for RCORE-01.T01/T02/T03 and the optional exact-input T04 volume oracle, not the
later shared Rust contract or all P0 work.
Only this report was edited; no production source, build or GUI process was run
by the reviewer. Fixture source, baseline diagnostics and final matching
production-gate evidence were checked on 2026-10-10.

## Assessment

The final optional argument reads the exact GUI-produced imported sphere BRep
and checks native adaptive volume. Its source and both matching 64-check input
replays were reviewed. Base 59/142/79-check runs retain their original scope;
the optional path has separate exact-input evidence below.

No blocking numerical, topology or ownership-oracle defect remains in the scoped
fixture source. The tighter native adaptive-area replay and final matching warm,
separate-process cold and native production-gate replays passed. The declared
absolute length/area bound is `1e-7` in mm/mm²; the repair
tightens integration precision without enlarging that bound.

The fixtures are independently authored using public native geometry APIs and
reuse the existing ThreeDm converters. Native C++ is necessary to inspect OCCT
edge-use identities, pcurves and adaptive mass properties; Python is restricted
to FreeCAD application fixtures, transactions and cold persistence. These are
test/native API exceptions to Rust-first, not new production geometry logic.

## Oracle assessment

- T01 constructs an analytic radius-10 circle, imports its rational NURBS form,
  records poles/weights/knots/domain and requires positive nonuniform weights.
  A separately authored rational quadratic quarter-circle checks the middle
  weight `sqrt(1/2)`. Radial/plane samples and analytic circumference are distinct
  checks. The candidate explicitly leaves a certified global deviation and
  analytic-to-NURBS parameter mapping unproven; sampled maxima cannot close T10.
- T02 authors a 20×20 square with a reversed radius-3 hole. The trim classifier
  must exclude the hole center and include `(5,0,0)`, while GUI checks additionally
  require one outer/one inner loop and center-to-face distance 3. Native adaptive
  area is compared with `400−9π`; default `Shape.Area` is recorded only as an
  observation. This distinction matters: the warm imported default area was
  `371.95006120129017`, whereas the analytic value is `371.72566611769184`.
- T03 starts with the uncapped analytic radius-4/height-7 lateral face whose
  source U direction is periodic. Converted surface closed/periodic facts are
  recorded separately from shape type. No solid is inferred from periodicity.
  Unique edge identities and ordered edge-use occurrences are counted
  separately, requiring a seam identity used twice. Connected free boundaries
  use shared vertex identity rather than approximate coordinates; the native
  candidate additionally requires degree-two endpoint incidence, radius/circle
  length, constant end planes and exactly two components at z=0 and z=7.
- Source fingerprints and existing identity/revision guards cover transfer
  immutability. A single import transaction, Undo removal and Redo restoration
  cover these outputs. They do not exercise mutable shallow clones, subobjects,
  sibling instances or asynchronous worker lifetime required by full T08/T11.
  A separate final native case exports an explicitly U-periodic NURBS surface
  and compares its source and shared sibling basis before/after export. That
  narrow converter ownership case passes without a production copy repair; it
  does not close the other T11 mutation/lifecycle scenarios.
- Cold persistence compares these three exact saved objects, native module and
  FCStd hashes, type, basis, loop/use topology, placement and CAD world bounds.
  Each cold snapshot repeats the geometric oracles. It does not include an
  explicitly periodic NURBS curve, so full RCORE-01.T09 remains open.

## Primary local API cross-checks

`TopoShapeWirePyImp.cpp::getOrderedEdges` walks `BRepTools_WireExplorer` rather
than a unique-edge map. `TopoShapeEdgePyImp.cpp::isSeam` calls native seam
classification. `TopoShapeFacePyImp.cpp::curveOnSurface` returns the native
occurrence pcurve/domain, and `isPartOfDomain` calls the UV trim classifier.
The native fixture also uses `BRep_Tool::IsClosed(edge,face)`, whose OCCT header
defines seam closure as two pcurves on the same face. Generic shape closure is
not treated as a sufficient solid test.

`TopoShapePyImp.cpp::optimalBoundingBox(False,False)` passes both flags to
`BRepBndLib::AddOptimal`. This excludes display triangulation and additional
shape-tolerance enlargement. OCCT still uses `Precision::Confusion` for its
numerical calculations, so the result is not an exact symbolic extremum or an
absence of all numerical enlargement. The warm circle bounds reflect this:
XY bounds are approximately ±10.0000001 and Z bounds ±1e-7.

`BRepGProp.hxx::SurfaceProperties(shape,props,Eps)` defines `Eps` as relative
adaptive integration precision. The fixture correctly checks measured absolute
analytic residual and successive-run difference separately. A failing 1e-10 /
1e-14 convergence comparison is evidence to tighten the integration pair, not
to enlarge the approved absolute bound. Recording returned error estimates is
useful diagnostics but does not replace the independent analytic oracle.

## Execution evidence inspected

Final matching production-gate evidence supersedes earlier warm/native baseline
runs. Root reports SDK build `450c52` exit 0, final native rerun `9122ba` exit 0
with 59/59 checks and three passing regression CTests. The reviewer read the
persisted native JSON and CTest log; Geometry, TrimInteriorMapping (152 checks)
and MultiShellSolid (32 checks) passed. Final native basis run `f9a819` also
reports exit 0/PASS; its selected gate review is separate.

| Final report | Result | SHA-256 |
|---|---|---|
| `build/merge_replay-1/1bf94f2737bf48abb8b9040071ba4103/results.json` | Warm, 142/142, `ok=true` | `b87c7cc0be205aadc0b7475c2600315d5fec30835c3ace86a0fdd4838dc187b7` |
| `build/merge_replay-1/975f60237386484981f2ce57e5e6d1b1/results.json` | Separate-process cold, 79/79, `ok=true` | `965eeedd73df32e6a0aa7fa0ebc5c7dccef6a7c3159663b0c6ca6787504d7e0f` |
| `H:/FreeCAD-src/build/p0-kernel-native-baseline.json` | Final native, 59/59, `ok=true` | `f14f1b625aec416aace94ff52bd6d43efc4b4de8862280d3baf2169b5468a9c9` |
| `H:/FreeCAD-src/build/p0-kernel-sphere-wireframe.json` | Exact imported sphere, 64/64, `ok=true` | `a5dd54fbfb4b13ab8866e39d8d20ea153bbd646533378de406aaba434dceb0ea` |
| `H:/FreeCAD-src/build/p0-kernel-sphere-shaded.json` | Exact imported sphere, 64/64, `ok=true` | `644d4bcd3cab8eb5b9adeec0d2d82b571e0096f865b49b85fd7e0fdee65e62df` |

Both final application reports identify native module SHA-256
`b80631e87580fb0c33efa63fb65151edbab1ac2b22d1a75bbcb05c2c35df660c`,
matching the module currently on disk. Cold replay verifies the same module and
saved FCStd hash `50779628e072e6c87828ebb7ad0343d8f166b0a5b02c16f4a8711b140bacb5b9`.
It repeats radius/trim/free-boundary checks and compares type, basis, loop/use
topology, placement and geometry-only world bounds without changing thresholds.
Root associates these replays with the current macro hash recorded below.

Root reports native build `d39fb2` exit 0 and exact sphere oracle executions
`252f0a`, both exit 0. The final companion host report
`build/merge_replay-1/1efa76209858406d966eefef052da9b4/results.json` passes 142
checks and persists each imported BRep. Both 3529-byte files have SHA-256
`726132b9ce7ca59bc89619fbbd434fff433b2d5a8f44df6afee1223175aedf91`, independently
recomputed from disk and matching both native oracle input hashes.

The optional oracle requires a readable input no larger than 4 MiB and a native
valid solid, then compares signed adaptive volume against `36π`. This is a
guard for stable saved test inputs, not a general hostile-file preallocation
certificate. Eps `1e-14`/`1e-16` masses are `113.09733553824248` and
`113.09733553824245 mm³`; analytic residual is `9.009895052258798e-9 mm³` and pair
difference `2.842170943040401e-14 mm³`, both below the unchanged `1e-5 mm³` bound.
Default nonadaptive volume `113.04787279841815` remains diagnostic. Kernel
relative-error estimates do not replace these independent residual checks.

Historical observations retained for the numerical/diagnostic audit:

- Warm FreeCAD report: `build/merge_replay-1/351520da23ca41879c1be164f69e255d/results.json`:
  `ok=true`, 142 checks, all passed. Native module SHA-256:
  `71a762ab77445d52ef113d3a578c65a7c79c177d2e31cdcd50901b2739078e45`.
- Archived native RED: `H:/FreeCAD-src/build/p0-kernel-area-convergence-red.json`:
  only the T03 integration convergence comparison failed. The companion proof
  records process exit 1 and source/executable/result hashes. This failed run is
  diagnostic evidence and must not be presented as an accepted final execution.
- Earlier native baseline: `ok=true`, 56 checks, all passed. Root observed build session `48134` and run
  tool `8fe6ad` exit 0, plus Geometry/TrimInteriorMapping/MultiShellSolid tests
  passing. That working JSON path was subsequently superseded by the final
  59-check run above. Both observed runs use the tighter integration pair
  `Eps=1e-14` and `1e-16` with the unchanged absolute `1e-7 mm²` bound:

  | Fixture | Analytic area | Eps=1e-14 | Eps=1e-16 | Pair difference | Absolute analytic residual |
  |---|---:|---:|---:|---:|---:|
  | T02 square/hole | 371.72566611769184 | 371.725666117692 | 371.7256661176919 | 1.1368683772161603e-13 | 5.684341886080802e-14 |
  | T03 open cylinder | 175.92918860102841 | 175.92918861504387 | 175.92918861504376 | 1.1368683772161603e-13 | 1.4015341776030255e-8 |

  The T03 coarse `Eps=1e-10` result was `175.9291913036002`, despite its returned
  relative-error estimate near `2.8e-16`. This directly demonstrates why a kernel
  estimate is diagnostic and cannot replace the independent analytic check.
- Archived cold RED:
  `build/merge_replay-1/bee8d5098ad2433b95d7d8604d2c033a/results.json`:
  `ok=false`, 53 checks reached, one failure comparing cylinder bounds. The
  earlier warm bounds used display triangulation (e.g. X minimum
  `-3.999890350216483`), while cold display bounds differed (X minimum
  `-3.99930513853103`). The repaired macro queries native geometry with
  triangulation disabled. These failed display-bound comparisons are excluded
  from final acceptance; neither the CAD tolerance nor cold comparison bound
  was loosened to hide them.

## Reviewed fixture/source hashes

Hashes identify the final scoped source and matching production-gate artifacts.

| File/artifact | SHA-256 |
|---|---|
| `tests/native/rhino_core_kernel_contract.cpp` | `cb2bc59731c95140fb1348089a81c9839464ac009d0a5fbc5a75ddef03235b6e` |
| `tests/native/CMakeLists.txt` | `ef6ff6264dffcb6f928c2821e9adb0fe60a7780532151f7a89e45e61db6ae550` |
| `tests/rhino_core_kernel_contract_smoke.FCMacro` | `5032a961fb9870a916e46faec6d7caf95c63d0a349d2a7ac341c4a22ee34e4d9` |
| Final native executable `H:/FreeCAD-src/build/rcore-native/ThreeDmKernelContractTests.exe` | `e2198a66c91449a0bd9222595f2243f01250fe0b72cab23c3bcf05d5a8a13c51` |
| Final native JSON | `f14f1b625aec416aace94ff52bd6d43efc4b4de8862280d3baf2169b5468a9c9` |
| Archived cold RED JSON | `a3e0be59d197d4ae7fba15a72953a39db572a319a0b3455f603bc777c4a802e1` |

The 142-check warm run used macro SHA-256
`c45cae39c5e65d035ef5b7737619764d3d837a34e2637a2171f596c234c8f932`.
The final 142/79-check replays supersede that earlier warm run and cover the
current macro hash and module. No old module run is presented as proof of later
production gate integration.

## Resolved findings and numbered acceptance scope

1. Resolved minor wording: the macro comment now says that the optimal bounds
   exclude display triangulation and **additional shape-tolerance enlargement**,
   while OCCT numerical tolerance remains.
2. Resolved native area convergence: the tighter pair passes both analytic and
   pair-difference checks. Final cold replay also passes with geometry-only bounds.
3. Final source/runtime hashes and matching production-gate results are recorded.

RCORE-01.T02's literal assertions are covered: the 20×20 face retains its radius-3
trim hole, the center is outside, and independent native adaptive area differs
from `400−9π` by `5.684341886080802e-14 mm²` within the unchanged `1e-7` bound.
The GUI/cold `Shape.Area` value remains diagnostic; this is not a claim that its
default mass calculation meets that bound.

RCORE-01.T03's literal assertions are covered: the independent uncapped source
is periodic U, the converted object is an open face without solids, and exactly
two connected circular free boundaries are identified. Seam identity/use counts,
closure, radius and z end planes provide additional evidence.

RCORE-01.T01 remains sampled rather than globally certified. The final separate
cold test is meaningful partial T09 evidence, but its circle is rational/closed,
not an explicitly periodic NURBS curve. T07, T08, T10 and full T11 remain open.
T04 is reviewed in `2026-10-10-geometry-companion-review.md`.

No Rhino runtime, full cross-backend parity, all-consumer invalid-allocation
rejection or global memory-safety claim is supported by these candidates.
