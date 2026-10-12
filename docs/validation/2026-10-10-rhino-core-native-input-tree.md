# Native NURBS input-tree preflight — partial RCORE-01.T07

Existing openNURBS/OCCT converters and the shared Rust basis validator are reused.
The new synchronous adapter examines the original reachable NURBS basis and
bounded Mesh/PointCloud arrays before the covered root consumers copy geometry,
call GetNurbForm, inspect Domain/CRC or construct converted output. Line/Polyline
commands, menus and solvers are reused.
The all-P0 objective remains in progress; T07 is not certified complete globally.

## Implemented boundary

Safe Rust owns a fixed value gate: depth64,16384 visited owner occurrences and
64MiB checked aggregate logical input bytes. Repeated owners in separate branches
are allowed and charged again; active-path cycles fail. The native basis policy
remains degree25,1million poles/axis knots and coordinate magnitude1e150. These
are explicit OM9 technical processing limits, not recovered Rhino defaults.

C++ borrows live initialized immutable SDK owners, checks storage envelopes and
extracts basis values for the existing Rust validator. It owns only temporary
native extraction vectors and local traversal state. Rust receives copied opaque
numeric identities and one exclusive caller-owned initialized544-byte state for
each synchronous call; no host pointers/geometry/allocator ownership are retained.
CountSummary is a copied16-byte record, with zero reserved/bytes on rejection.
Rust has no allocations in the traversal/count policy. Release retains panic=abort;
native exceptions unwind locally and discard failed state without crossing Rust.
Null/alignment/address guards do not establish pointer lifetime or authenticity.

Registered unknown openNURBS derived owner classes fail before conversion.
Registered supported classes are NURBS, Line, Arc, Polyline, PolyCurve,
CurveProxy/BRepEdge/BRepTrim/PolyEdgeSegment, PolyEdgeCurve, CurveOnSurface,
NURBS/Plane/ClippingPlane/SurfaceProxy/BRepFace, Rev/SumSurface and Extrusion.
This is not a certificate for arbitrary custom native virtual overrides.

## Actual consumer and relationship audit

| Current consumer/owner | Original relationship read | Boundary covered or still open |
|---|---|---|
| curve3d/curve2d/importCurve/importCurveObserved | CurveProxy::ProxyCurve; PolyCurve::SegmentCurves and SegmentParameters; CurveOnSurface::m_c2/m_c3/m_s; direct basis/Polyline fields | Whole reachable gate before IsValid/GetNurbForm. Plain import sequences curve conversion before Domain argument evaluation; observed import gates before Domain/CRC. |
| importSurface | SurfaceProxy::ProxySurface; Rev::m_curve; SumSurface::m_curve[2]; Extrusion::m_profile/m_profile_count | Gate every reachable original basis before profile evaluation/conversion or GetNurbForm. Borrow original profile; do not call allocating Profile3d. |
| importBrep/resolvedBrepOrientation | m_S/m_C3/m_C2 plus native face/edge/trim proxy owners, topology/adjacency envelopes and edge-to-trim references | Entire NURBS owner forest and edge trim references before ON_Brep copy. Orientation checks the independent copy with IsValid before SolidOrientation; source caches/ownership remain separate. |
| curveON/exportCurve/exportSurface | OCCT Trimmed/Offset BasisCurve; RectangularTrimmed/Offset BasisSurface; Revolution/LinearExtrusion BasisCurve | Original wrapped BSpline/Bezier basis before trim/copy/transform/GeomConvert. Bezier and Polyline extraction use the same Rust basis policy. |
| exportBrep | TopoDS child TShapes; BRep_TEdge::Curves representations, 3D support, pcurves including second seam pcurve, surface and regularity surfaces | Native shape forest before BRepCheck/NurbsConvert/output. Native handles remain native-owned. Polygon/cache numeric storage is outside this basis gate. |
| ThreeDmArchive::readArchive | block member traversal; Duplicate/transform before low-level converter; surface BrepForm; signed-placement IsSolid/SolidOrientation | Supported source curves, BReps and surfaces now pass the tree gate before these member staging/query steps. Raw archive parse and native-model decode remain open. |
| Mesh and PointCloud adapters | Mesh arrays, face and N-gon indices, mesh validity, cloud points/optional fields and value copies | Direct array counts/capacities, logical bytes, face/N-gon indices and supported numeric fields are gated before `ON_Mesh::IsValid`, Explode `Duplicate`, or PointCloud copy. Current adapters stay in use; nested decoder allocation remains open. |
| ThreeDmExplode | source.Duplicate; Extrusion::BrepForm; cage `IsValid`, knot/CV reads and `IsoSurface` | Supported curves, BReps, surfaces and meshes validate before duplicate; unsupported PointCloud Explode rejects before clone; extrusion enters the surface gate before `BrepForm`. NURBS cages check count/product, knots, CV strides/storage and finite values before SDK validity/evaluation. The gate begins after `ONX_Model::Read`. |
| ThreeDmCurveOnSurface current-field staging | Duplicate/child staging and archive reconstruction | Root staging/decoder allocation remains open; low-level covered converters use the gate only when reached. |
| ThreeDmCage/CageGeometry | current cage extraction and native deformation adapters | Other cage-edit/deformation paths are not covered by the `Explode` cage gate; member/worker/global allocation clauses remain open. |
| Preserved opaque archive records | retained without solver interpretation | Retention does not prove edit/solver support, native decode preflight or all userdata allocation. |

ON array Count/Capacity and pointer span/alignment precede bounded child reads.
NURBS capacity zero remains the SDK externally-managed allocation convention,
requiring the caller's live readable allocation. No source field is repaired,
clamped, normalized or written. Converter-owned working results may still use the
existing convention changes and validate them again.

## Executed evidence

- 572 full Rust tests pass. The16 new traversal/ABI/payload tests exercise sticky
  failure, forged state, exact depth/node/byte bounds, overflow, cycles, layout,
  malformed pointers and count summaries. Clippy exits0 with existing warnings.
- Tools:50 run,49 pass,1 skip.
- The 2026-10-10 final native fixture:64 checks pass on actual converter entry hooks. It observes
  original hidden basis rejection, unused BRep C2/C3, safe restored owner cycles,
  BRep vertex adjacency and out-of-range edge trim references rejected before
  copy, invalid face-loop topology rejected before orientation query, 64/65-deep
  proxies,16384 children, shared800000-CV aggregate rejection, registered unknown
  derived owner, domain ordering and rational/periodic/reversed-subdomain/trimmed
  BRep positives. Hooks compile out in normal production libraries. Full output:
  `H:/FreeCAD-src/build/rcore-native/native-input-tree-topology-index-green-20261010.json`.
- Archive/member follow-up: the instrumented RED run recorded66 checks/2 failures;
  an invalid transformed block curve reached `Duplicate`, and an invalid surface
  reached `BrepForm`. The final `ThreeDmNativeInputTreeTests` result is67/67,
  including those cases plus a tagged signed-placement BRep rejected before its
  `IsSolid`/`SolidOrientation` query. Assertions require the native basis error,
  so an earlier archive-read failure cannot satisfy them. Full output:
  `H:/FreeCAD-src/build/rcore-native/native-input-tree-archive-signed-green-20261010.json`.
- Six affected archive/block CTests pass after rebuilding against the changed
  archive path: CurveOnSurfaceArchive, NativeInputTree, NativeTransform,
  Preservation, Archive and Blocks. The native-transform suite used a fresh
  per-run TEMP directory because its fixtures use fixed filenames.
- The preceding checkpoint had seven RCORE-01 CTests and the geometry regression
  pass after relinking. On the topology-index follow-up revision, the affected
  `ThreeDmNativeInputTreeTests` target was rebuilt and passed 64/64, then the
  plugin was rebuilt and the fresh shared-host macro passed 21/21.
- Shared-host rerun after the topology guard:21/21 authored native input-tree checks pass in a fresh
  FreeCAD process using the selected plugin. Native source staged BRep bytes,
  destination sentinel, document object/property inventory, geometry, placement,
  state and actual Undo/Redo counts remain unchanged for curve/surface
  NaN/infinity/coordinate-limit errors. Valid rational input still writes V5.
  The 205 parameter-dependency and13 Hatch host controls are separate earlier
  evidence and are not counted as part of this rerun. Private fake grid counts are
  exercised in native fixtures, not claimed as a public host count-input API.
  This run uses `H:/FreeCAD-src/build/native-input-tree-shared-host-20261010-index-repair`.
- After the archive/member gate, the plugin-only build against the shared SDK
  succeeded and a fresh selected-plugin host process again passed21/21 checks
  with exit code0. Plugin SHA-256 is
  `08e9e7ca60f7c557346d4d585534a58403e8278666376b1936c8b3b95c0e3283`; results
  are in `H:/FreeCAD-src/build/native-input-tree-archive-stage-final-20261010`.
- Explode/cage follow-up: the fixture validates original supported block members
  before `Duplicate`, and gates cages before `IsValid`, knot/CV reads or
  `IsoSurface`. RED recorded one failure at the finite-knot-overflow case; the
  repaired fixture passes73/73, including oversized cage count/stride, non-finite
  CVs and finite extreme knots whose subtraction overflows. Six focused
  archive/block CTests and the positive `OM9ThreeDmExplodeTest` pass. The
  plugin-only build against the shared SDK succeeded and a fresh selected-plugin
  host passes21/21 (exit0), plugin SHA-256
  `ab8c29cc1fed70e9f29cf9754725a65cbfc42f77a0eccf496632ca25765d570e`; results
  are in `H:/FreeCAD-src/build/native-input-tree-explode-final2-20261010`.
  [Exact evidence and independent review](2026-10-10-rhino-core-native-explode-cage.json).
- Mesh/PointCloud follow-up: RED showed an invalid mesh face reaching the SDK
  validity path and a malformed PointCloud capacity being traversed. The green
  native fixture now passes92/92; malformed block mesh fails before
  `Duplicate`/`IsValid`, PointCloud array capacity fails before value reads, and
  valid archive mesh/cloud adapters still pass. The six focused archive/block
  CTests and positive Explode integration pass. A plugin-only rebuild against
  the shared SDK and fresh selected-plugin host pass21/21; exact build/source
  hashes and independent review are in
  [the mesh/cloud evidence](2026-10-10-rhino-core-native-mesh-cloud.md).

See [exact results, source/module/process fingerprints and retained artifacts](2026-10-10-rhino-core-native-input-tree-evidence.json).
The focused 2026-10-10 rerun, current source hashes and review rulings are in
[the follow-up evidence record](2026-10-10-rhino-core-native-input-tree-followup.json).

## RED and excluded attempts

Native baseline:23 checks/12 failures; expanded safe owner-cycle baseline27/16.
Domain-order regression29/1; registered unknown-owner regression43/1. Each failed
behavior precedes its production repair. Initial fixture compile failed because
integer surface bounds selected ambiguous OCCT overloads; explicit doubles fixed
the fixture. Two proxy-cycle setup attempts recursed inside SDK cache teardown
before reaching the gate; exact fixture-owned processes were verified/stopped,
and they are excluded. The accepted cycle uses public CurveOnSurface/SumSurface
relationships, restoring the injected owner pointer before destruction.

The independent review found an unsafe orientation-query ordering: pinned
OpenNURBS 5 `SolidOrientation()` reaches `IsManifold()`, whose second edge trim
reference was not range-checked. The added fixture first recorded 63 checks/2
failures; the preflight now rejects edge trim references before copy, and the
orientation path rejects an invalid copied face-loop graph before querying
`SolidOrientation()`. The repaired fixture passes 64/64.

Initial Rust policy stub was observed1pass/7fail; that first raw filename was later
overwritten by its8-pass run and is not represented as retained RED evidence.
The later forged-state RED and separate four-case FFI/two-case payload RED logs
are retained. The first12-pass host macro was superseded by the final21-pass
curve+surface macro. Source and proof file hashes identify the actual accepted run.

## Remaining requirements

This is a bounded converter preflight increment, not all-P0 completion or a global
no-native-allocation certificate. Input setup/native archive decode and bounded
C++ extraction vectors can allocate before complete Rust value validation.
Instrumentation measures the named copy/GetNurbForm/converted-construction entry
points; it does not intercept every SDK/cache/container/process/worker allocator.
Raw archive decoding and allocations inside `ONX_Model::Read`, analytic/mesh/point
input policies, other cage-edit paths, arbitrary custom native owners, aggregate
worker memory and broader concurrent lifetimes still require evidence.
These remain required by the parent all-P0 plan.

## Follow-up review rulings

Ruling: retain early rejection of `DeleteEdge`/`DeleteTrim` tombstones — the
pinned SDK's `IsCorrupt` rejects the cleared back-pointers before reaching its
later deleted-index checks, and the prior copy-plus-validation path rejects the
same state — cost if wrong: a different SDK version that permits non-compacted
BReps remains unsupported.

Ruling: validate every edge-to-trim reference before authorizing a BRep copy,
then validate the independent orientation copy before calling `SolidOrientation`
— the pinned OpenNURBS 5 manifold path reads its paired trim without a range
check — cost if wrong: malformed topology that the SDK intended to recover will
be refused earlier.

Ruling: validate the original supported archive member before `Duplicate`,
surface `BrepForm`, or signed-placement orientation queries — the regression
fixtures observed those operations before the converter-only gate — cost if
wrong: valid but unlisted native owner classes may be rejected before a working
converter can inspect them. The gate still begins after `ONX_Model::Read`.

Ruling: check NURBS cage counts, storage/strides, knot differences/domain width
and CV/weight values before `IsValid` or `IsoSurface` — finite extreme knot
endpoints overflow subtraction in the pinned SDK evaluation path — cost if
wrong: a supported cage with invalid numeric input is refused; this does not
cover cage-edit APIs or allocations during archive decoding.

The existing `Line` and `Polyline` commands and their `Part.makePolygon` geometry
construction are reused from `CurveController.cpp`; no duplicate wire constructor
or solver was added. Raw input decoding, global allocation, analytic families,
other cage-edit and worker requirements stay open.
