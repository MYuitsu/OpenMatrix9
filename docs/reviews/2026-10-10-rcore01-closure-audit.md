# RCORE-01 completion audit

Objective: complete RCORE-01, retaining its original P0 scope. This audit reads
the current chapter, native-input-tree plan, `CoreGeometryContract.cpp`,
`CoreObjects.cpp`, native input gate and import binder. A bounded increment is
not chapter completion. The current chapter is **in progress**.

## Concrete work from this audit

`commit3dm` previously opened a transaction and bound live prepared CAD without
the input-tree gate. The retained RED report at
`H:/FreeCAD-src/build/import-commit-gate-red2-20261010/results.json` shows a
curve pole of `1e151` accepted: four extra document objects, Undo 1 → 2 and
Redo 1 → 0. This is a real mutation-boundary gap, separate from raw decoding.

The import adapter now gates every prepared CAD row and preserved affine CAD
preview before native topology evaluation, transaction opening or assignment.
It reuses Rust numeric/traversal validation and OCCT topology validation.
Worker entry rejects before document lookup. Exact built-in Python containers
ensure the native storage inspected by the gate matches the Python binder's
iteration; lengths use intrinsic storage, never overridden Python callbacks.

The runtime fixture records every persisted property member's bytes for the
document, all objects and their view providers, plus types, property statuses,
object states and Undo/Redo. ZIP timestamps are excluded; member payloads are
compared. It avoids computed `App::Part.Shape`/`App::Link.Shape` getters. A
metadata edit with unchanged property names proves the oracle detects values.
Rejected requests also preserve detached BRep bytes and an executable Redo.
See [the resulting evidence](../validation/2026-10-10-rhino-core-import-commit-gate.md).

## Numbered adapter requirements

| Requirement in chapter | Current evidence | Remaining requirement; closure is unproven |
|---|---|---|
| 1. Rust tags, owned snapshots, validation and versioned operation/type/options registry | `geometry_contract.rs`, `core_objects.rs`, basis/input-tree modules and focused Rust tests | Native command integration still covers selected inspection/exchange/consumer paths; inventory and gate every supported solver/options combination. |
| 2. Small native bridge resolving identity, subelements, placement and copying geometry/topology | `CoreObjects.cpp`, `CoreGeometryContract.cpp`, instance and ownership fixtures; common face/loop/edge-use graph, Rust-owned NURBS/Bezier basis and ten analytic curve/surface kinds including UV conics and indirect frames | Direct Part/nested direct Part links now have checked world composition and bounded native identity/appearance attributes. Ordered group/member/subelement definitions and nested link identities now have bounded owned-graph evidence. Mesh/cloud raw-local points/facets/bounds and direct/group/link world frames now have owned-snapshot evidence. Ordered native arrays now retain owned indices/identities/member paths and world frames through the existing Part/group/mesh/cloud readers. Remaining reference contexts, ownership/cycle fixtures, native channels/archive float64/subelements and arbitrary attributes/userdata remain open, along with global3D edge/pcurve correspondence. Owned3D edge/vertex/tolerance rows now join the UV topology, retaining standalone and loose native geometry; this does not certify global parameter correspondence. See world/attributes follow-up; derived chains have their separate owned-metadata evidence. |
| 3. Rust capability decision before enabling an operation; precise unsupported reason | Shared Rust native Edit policy now gates existing Join/Explode/Trim/Boolean declaration, concrete input and current solve options; real host regressions retain existing solvers. | Creation/Surface/Rebuild/cage and other enabled consumers still require integration; do not infer all families from these Edit checks. |
| 4. Tested OCCT BRep/openNURBS adapter and exception mapping | Kernel/trim/parameter/polycurve/instance fixtures, current focused native CTests, new import gate | Raw decode/allocation and remaining analytic/cage/custom native owners are outside the current gate. Mapping to the chapter's typed errors is not uniform. |
| 5. Geometry/topology validation, conversion report, Rust commit/reject contract | Prepared CAD/mesh/current-cloud have full GeometrySnapshot field structure, privately owned geometry/frozen attributes and Rust result admission. CAD master restores with matching SDK; mesh archive correspondence is checked at float32 precision. | Upstream exactness/deviation/parameter maps, remaining channels/attributes and other enabled/native consumers remain open. |
| 6. GUI-thread transactional commit of geometry/meta/links/History; cache generation | Core registry/delayed Rebuild and prepared CAD/mesh/current-cloud GUI-thread preflight/admission use private geometry/frozen metadata; predictable rejection precedes transaction and preserves executable Redo. | Other workers/publication paths, full source lock/ownership, remaining non-CAD adapters and arbitrary post-transaction failure Redo restoration remain open. |
| 7. Native exception boundary; constrained Python bootstrap/API use | Existing adapter ownership comments, Rust FFI layout checks and native imports | Audit every entrypoint. An instrumented set of copy/solver calls does not establish a global native allocation/lifetime certificate. |

## Capability matrix

| Capability | What is evidenced | What must still be verified for the full requirement |
|---|---|---|
| C01 BRep identity/placement | Existing source stamps and selected operation consumers | Direct Part/nested direct Part link frames now have focused proof; remaining instance references, uniform consumers and persistent subelement identity remain open. |
| C02 rational/periodic NURBS curve | Circle query, periodic persistence, supported edits and basis gates | Remaining conversion/edit families, attribute retention and parameter policies. |
| C03 NURBS support/face trims/pcurves | Hole/seam converter preflight, common owned face/loop/orientation/edge-use, Rust-owned NURBS basis with rational weights/periodic axes/full domains, native analytic metadata and explicit Bezier curve/surface/UV control nets | Offset/extrusion/revolution with typed basis chains and inner one/two-axis trims now have focused metadata evidence; native UReverse/VReverse and broader trim/parameter-map families remain uninspected. Read-only support copying does not certify conversion parameter maps. |
| C04 face/shell/solid/compound | Native classification, owned incidence/vertex-link/face-orientation graph and bounded OCCT validity/cavity/solid inspection reports; mirrored frames retain native/world orientation separately. | Integrate these scoped reports into uniform operation/commit consumers. Native algorithm validity is distinct from global embedding certification; representation provenance alone does not establish validity. |
| C05 point cloud/mesh distinct from CAD | Existing independent representations plus native owned snapshots and prepared result/admission/cold restore; float64 current-cloud channels, native mesh segment budgets and archive correspondence at float32 precision are tested. | Remaining native channels/subelements/attributes and other enabled consumers need evidence; no mesh cache is promoted to CAD master. |
| C06 shared instances | Existing App::Link isolation/transform fixture | Nested direct Part definitions now have checked frame/attribute descriptors. Ordered group/member/subelement definitions now have owned-graph evidence; mesh/cloud definitions now retain owned raw-local geometry/frame evidence; arrays now have bounded ordered occurrence/frame evidence; remaining reference contexts, ownership/cycle fixtures, edit-reference and attributes across families remain open; block authoring remains linked to RCORE-06. |
| C07 lightweight extrusion | Existing retained-source and BRep working representations | Explicit conversion/preservation reporting and structural exchange regression for the declared scope. |
| C08 CAD vs tessellation | Accepted sphere display fixture and CAD presentation path | Common generation/invalidation/export routing across display consumers, without promoting mesh cache to CAD. |
| C09 polycurve partition semantics | Flat line/arc/NURBS/polyline partition/reversal/persistence fixture | Nested/reference/unsupported policies, attributes, locus/parameter-map claims beyond the recorded scope. |
| C10 operation × type × options registry | Rust versioned inspection/working exchange plus existing native Join/Explode/Trim/Boolean declaration/input/solve integration, including special leaf preflight and actual command regressions. | Other enabled operations, full Edit result/conversion reporting and remaining original obligations are open. |
| C11 native lifetime | Delayed Rebuild and detached subobject fixtures | Other workers, cancellation/close/replacement paths and outstanding native owners. |
| C12 annotation structural preservation | Linked to RCORE-10/11 and existing source-retention policy | A typed preservation/unsupported contract for the P0 exchange boundary; no outline flattening may be called structural preservation. Annotation authoring is outside the selected P0 scope. |

## Acceptance fixtures

The milestone records are evidence at their own source/runtime fingerprints.
They do not certify arbitrary families or all current sources merely because
their counts remain listed in the chapter.

| Fixture | Existing evidence / current change | Open scope |
|---|---|---|
| T01 rational circle R10 | Kernel and curve-parameter evidence | Keep the declared sampled vs global bound and exact conversion/report scope. |
| T02 square20/holeR3 | Numbered acceptance in kernel-contract record; common snapshot retains outer/hole loops and two boundary components; authored 3DM import has copied NURBS support, and native analytic plane/conic pcurve metadata is tested separately | Broader trim families and complete 3D/UV correspondence remain separate. |
| T03 open periodic cylinder | Numbered acceptance plus common snapshot retains seam occurrences, distinct pcurves/UV sides and two boundary components; owned NURBS support is checked for imported cylinder/authored compact-periodic surface, with native analytic cylinder support and indirect frames tested separately | Uniform solid/manifold policy and all native parameter-reversal cases remain separate. |
| T04 sphere display independence | Numbered acceptance in kernel-contract record | Cross-consumer cache/representation routing remains separate. |
| T05 reverse/persist polycurve | Polycurve-partition record | Recorded flat segment families; nested/reference and broader report integration remain open. |
| T06 shared mirrored/nonuniform instances | Instance-isolation record | Recorded authored two-root definition; arbitrary nested composition remains open. |
| T07 malformed basis before allocation/commit | Owned Hatch, native owner trees, mesh/cloud gates and new prepared-CAD commit rejection | Raw decoder/input construction, uncovered analytic/cage/worker paths, native allocation instrumentation, metadata/type and aggregate batch budgets are not closed. Fake count overflow is tested on native initialized fixtures, not invented as a Python host input API. |
| T08 stale preview/source edit/close | Real delayed Rebuild delivery record | Other native workers/publication paths and broader lifetime evidence remain open. |
| T09 periodic/seam/hole FCStd | Warm/cold periodic-persistence record | Declared sampled oracle and family scope; broader structural/attribute preservation is separate. |
| T10 analytic→NURBS parameter dependency | Host Circle/Line parameter-dependency record | Other analytic/restricted/UV families and unproved parameter maps reject or remain unsupported. |
| T11 clone/subobject mutation | Native subobject-ownership record | Recorded Line/quadratic Wire/untrimmed tensor Face; broader topology families and owner lifetimes remain open. |

## Remaining schema and invariants

`GeometryRef` identity/generation, units and frame provenance already have
selected real consumers. The common topology increment now supplies owned
faces/loops/orientations and distinct pcurve domain/UV endpoint observations,
with boundary connectivity and repeated-face occurrence metadata. Its 383 host
checks and 585 full Rust tests are recorded in
[topology snapshot evidence](../validation/2026-10-10-rhino-core-topology-snapshot.md).
The owned-support follow-up adds full NURBS face/pcurve and standalone 3D curve
basis, retaining exact numeric poles/weights/knots, compact periodic axes and
full domains separately from trim restrictions, orientation and native location.
Its 54 new +383 regression host checks and 594 full Rust tests are recorded in
[owned-support evidence](../validation/2026-10-10-rhino-core-owned-support.md).
The analytic follow-up now copies native Line/Circle/Ellipse/Hyperbola/Parabola
and Plane/Cylinder/Cone/Sphere/Torus support into a Rust-owned value model,
including UV sense separately from the embedded frame determinant. Authored
conics cover both UV senses and reflected supporting surfaces cover indirect
native frames. Supporting-geometry metadata does not close analytic input-tree
allocation gates. Native UReverse/VReverse reparameterization remains untested.
The final selected plugin passed 174 analytic +437 regression host checks and
598 full Rust tests; the source/runtime fingerprints and process outcomes are
recorded in [analytic support evidence](../validation/2026-10-10-rhino-core-analytic-support.md).
Bezier support now reuses the existing native checked adapters and copies its
original control nets/rational weights into private Rust-owned wrappers, with
explicit Bezier tags and natural domains separate from native restrictions.
Independent Bernstein/native tests exercise rational and polynomial 3D curves/
surfaces, rectangular U-major control nets, trim/location/orientation and
authored UV support. The accepted plugin passed76 Bezier +611 regression host
checks and603 Rust tests, recorded in
[Bezier support evidence](../validation/2026-10-10-rhino-core-bezier-support.md).
These copies do not certify conversion parameter maps.
Offset/extrusion/revolution now have the focused owned-chain evidence below;
other unrecognized support kinds remain explicit unknowns. These snapshots still
do not supply all required
`GeometrySnapshot.geometry_data/topology_data/source_attributes`,
`FaceData.loops/pcurves/orientation`, conversion report fields and
`GeometryResult.topology_map/diagnostics`. Null observations are honest unknowns,
not completed schema requirements.

Full chapter closure also requires proving rational basis/domain/direction/seam
retention; ordered polycurve joints; exact-once world placement; valid shell and
cavity policies; display-cache independence; preview isolation; stale/locked/
deleted-source checks; errors/cancel without source deletion or empty Undo; and
the error categories `UnsupportedRepresentation`, `InvalidBasis`, `InvalidTrim`,
`NonFiniteGeometry`, `StaleGeometry`, `UnsupportedTransform`, `KernelFailure`,
`ConversionLoss` with operation/input identity. Existing generic Python error
strings and a narrow ABI test cannot stand in for uniform typed diagnostics.

Next implementation: complete other supporting geometry types, full edge/pcurve
correspondence, attributes and world composition of the common snapshot, plus full topology/solid and
Rust validation/report schema for supported BRep faces/shells/solids, reusing
the accepted hole/seam/open-cylinder/cavity converters and fixtures. Continue
raw allocation, uncovered numeric consumers and uniform mutation/worker gates
as required work. Do not repeat already accepted Line/Polyline constructors.

## Derived supporting geometry follow-up — 2026-10-10

Offset 3D/UV curves, offset surfaces, extrusion and revolution now retain typed
Rust-owned basis chains, including analytic/NURBS/Bezier leaves and inner trim
restrictions. One-axis surface trim can keep the other axis unbounded. The final
plugin passes91 new +687 regression =778 host checks and610 Rust tests.
[Evidence and limits](../validation/2026-10-10-rhino-core-derived-support.md).
This changes supporting metadata evidence only; regularity, global parameter
correspondence, world/attributes, solid certification, reports and all remaining
original gates still require completion. Earlier numerical records retain their
own historical fingerprints. Full chapter remains in progress.

## World frame/source attribute follow-up — 2026-10-10

Checked Rust world composition now applies native placement exactly once for
direct Part/compound subfaces and nested direct Part App::Link definitions,
including mirror/nonuniform scale and all four nested LinkTransform combinations.
Native identity/appearance attributes are owned separately for instance/definition;
uninspected fields remain explicit.78 new +778 regression =856 host checks and
620 full Rust tests pass on the matching plugin, with process exit0.
[Evidence and limits](../validation/2026-10-10-rhino-core-world-attributes.md).
Group/member/subelement/array and mesh/cloud instance definitions, arbitrary
attributes/userdata and all earlier full-chapter gaps remain open. Neither the
scoped review nor these tests close the original RCORE-01.

## Instance/group definition follow-up — 2026-10-10

Ordered placed/plain group definitions, empty groups, nested one-step link
identities, actual member paths and single/multiple Part subelements now have
Rust-owned bounded occurrence graphs and independent native world samples.
48 new +863 regression =911 host checks and624 full Rust tests passed on the
matching plugin, all actual process exits0.
[Evidence and limits](../validation/2026-10-10-rhino-core-instance-definitions.md).
Arrays/mesh/cloud, link-container references, overlapping selection contexts,
full attributes/topology/report/gate requirements remain open. Original chapter
completion is unproven; this evidence does not close the complete P0 contract.

## Native discrete snapshot follow-up — 2026-10-10

Direct/group/link native mesh/cloud leaves now retain Rust-owned raw-local
points, triangle indices, local bounds and exact-once native kernel transforms.
Counts/retained data gate before staging/slices; values gate before cloning;
serialized rejection is atomic but follows bounded staging/cloning. Exact native
property ownership checks cover the unregistered PointKernel case. Mesh SDK
imports stay deferred in an independent cold-module fixture.
99 new +6 cold +911 regression =1016 host checks and628 full Rust tests pass;
all11 matching selected processes exit0.
[Evidence and limits](../validation/2026-10-10-rhino-core-discrete-snapshots.md).
Native channels/flags/segments, archive float64 attributes and subelement mapping
remain uninspected. Arrays/reference contexts and all original topology/report/
capability/typed-error/allocation/worker/commit/lifetime requirements remain open.
Full RCORE-01/C05/C06 completion is still unproven.

## Native3D edge/vertex follow-up — 2026-10-10

Rust-owned3D edges/vertices/tolerances/locations now join the common UV topology,
including standalone Part roots and loose mixed topology. Native SameParameter/
SameRange flags are observations; null certificate and separate restrictions
preserve uncertainty. Direct/parent/link raw-local frames apply placement once.
Current-SDK RED reproduced incorrect absence status, not the reviewer's possible
frame loss; both null-degenerate root kinds now reuse owned edge metadata.
921 new +1016 regression =1937 host checks;632 full Rust tests;all12 matching
selected process exits0. [Evidence and limits](../validation/2026-10-10-rhino-core-edge-geometry.md).
Global parameter correspondence, reversals, manifold/solid/cavity/report/attribute/
array/reference and every original allocation/worker/commit/lifecycle requirement
remain open. Full RCORE-01 completion is still unproven.

## Native instance-array follow-up — 2026-10-10

Rust-owned ordered array metadata retains real indices, optional physical element
identities, definition stamps and checked native world frames. Virtual member paths
are preserved; cumulative minimum graph count/depth rejects before native path
staging.609 array+10 boundary+1937 regression=2556 host checks;635 full Rust tests;
all14 matching host exits0. [Evidence and limits](../validation/2026-10-10-rhino-core-instance-arrays.md).
Remaining reference contexts, ownership/cycle fixtures, attributes/discrete
channels, global topology/certificates/reports and every original allocation/
worker/commit/lifecycle requirement remain open. Full completion stays unproven.

## Native reference-context follow-up — 2026-10-10

Bounded group/link/array paths now retain explicit names/labels, collapsed alias
fallback, native point identity/world frames and configured occurrence-only
Face1/Face3 selection; repeated prefixes and suppressed object/topology
traversal reject.540 new+2556 regression=3096 host checks;641 full Rust tests;
15 matching host exits0. [Evidence and limits](../validation/2026-10-10-rhino-core-reference-contexts.md).
Configured topology/overlapping selection, native ownership/cycles, arbitrary
attributes/discrete channels and all original certificates/reports/allocation/
worker/commit/lifecycle obligations remain open. Whole completion is unproven.

## Native topology-validation follow-up — 2026-10-10

Rust now owns vertex-link and face-orientation observations. Scoped native CAD
inspection reports combine bounded OCCT validity/cavity/orientation with these
facts; native Solid roots require all prerequisites and world orientation is
derived from native once.287 new+3096 regression=3383 host checks;655 Rust;
17 matching exits0. [Evidence and limits](../validation/2026-10-10-rhino-core-topology-validation.md).
Original uniform operation/conversion/commit consumers and every other chapter
obligation remain open; native algorithm validity is not a global embedding
certificate. Existing historical audit statements must be read with this newer
scoped evidence, not as absence of the new inspection report.

## Prepared CAD commit-result follow-up — 2026-10-10

Prepared CAD host/affine-preview rows now have private native copies, recursively
owned plain metadata, Rust field/admission policy and typed diagnostics before
transaction. Native-generated immutable validity reports persist in the same
object/archive transaction. Metadata errors preserve executable Redo; row identity
is reset before recursive validation.154 new+3434 regression=3588 host checks;
660 Rust;22 matching host exits0. [Evidence and limits](../validation/2026-10-10-rhino-core-prepared-cad-results.md).
This advances adapter items3–6 without completing full output GeometrySnapshot,
upstream conversion/parameter/attribute reports, non-CAD admission, every enabled
consumer or arbitrary post-transaction failure Redo restoration. Logical limits
are not a process-wide allocation certificate. Every original remaining item
above stays open except the specifically evidenced prepared CAD/metadata scope.

## Prepared CAD full result-field follow-up — 2026-10-10

Prepared native CAD now has every named GeometrySnapshot field, complete report
field structure, independently restorable no-mesh OCCT V3 master and existing
owned topology/support observations. Exact binding attributes and full-width
unit revision are captured before transaction.107 full-snapshot warm/cold+95
metadata+3499 regression=3701 host checks;665 Rust;24 actual exits0.
[Evidence and limits](../validation/2026-10-10-rhino-core-prepared-result-snapshots.md).
This supersedes the earlier missing-field observation for prepared CAD only;
upstream certificates/maps/remaining userdata, other operation families/non-CAD
and every other original requirement remain open.
Minor deferred: native writer budget errors map to KernelFailure. Full chapter
closure is not inferred from schema structure or this accepted native master.

## Prepared discrete result follow-up — 2026-10-10

Prepared mesh/current float64 clouds and affine previews now have private owned
geometry, complete snapshot/result fields and Rust schema/channel/index admission
before transaction. Native segment budgets precede clone; archive/native mesh
oriented triangles match at declared float32 precision with location once.
Real observer mutation and malformed inputs preserve source/executable Redo;
reports and geometry restore independently in a second process.81 warm+8 cold
+3701 regression=3790 host checks;675 Rust;26 actual exits0.
[Evidence and limits](../validation/2026-10-10-rhino-core-prepared-discrete-results.md).
Earlier non-CAD missing-result/admission observations are superseded for these
prepared rows only. Upstream certificates/maps, remaining channels/attributes,
other consumers/workers and every original remaining item stay open. Native
exception categories remain non-uniform; logical budgets are not process-wide
allocation certificates. Whole chapter completion remains unproven.

## Native Edit registry follow-up — 2026-10-10

Existing native Edit consumers now use shared Rust kind/type/options/backend
policy before declaration, input copy and solve, including Trim option callbacks.
Native Part/Mesh gates are reused; block/cage leaves and ShapeString now validate
before extraction/copy. Real RED special Explode mutated source/Undo; GREEN
retains sources, executable Redo and valid scalar block world placement.
679 Rust;4456 host checks;36 matching actual exits0.
[Evidence and limits](../validation/2026-10-10-rhino-core-native-edit-capabilities.md).
Per-owner validation does not close aggregate budgets, full Edit GeometryResult/
conversion/maps, every annotation leaf, raw allocation or other consumers.
All original requirements remain authoritative; full chapter is in progress.
