# Full native Hatch loop editing and pattern rendering (pending)

Continue the approved full openNURBS specification inline. Current scalar fields,
boundary display and independent SDK2013 semantic cases are evidence for named
slices; they do not complete Hatch or its curve/userdata/pattern requirements.

1. Inventory each real native Hatch loop curve subtype and nested curve/userdata
   representation before designing its editable schema. Keep native class/type,
   rational weights, knots/domains, loop role and applicable child metadata.
   Never silently convert all source loops into one easier representation.
2. Add bounded persistent current-loop fields/files with source/host schema and
   baseline checks. Stage edits on a native clone, validate closed2D geometry and
   Hatch loop topology, then apply current plane/base/pattern fields before own
   placements exactly once. Preserve unchanged native fields and safe child data;
   unsafe dependencies must produce an explicit preflight report.
3. RED tests cover rational CV/weight/knot edits, native domains, outer/inner
   loops, required subtype-specific edits and topology, invalid/nonfinite fields,
   shared/canonical/independent copies, mm/cm and source/destination invariants.
   Loop support stays incomplete until the full inventoried subtype requirements
   have evidence; a NURBS-only adapter cannot establish universal completion.
4. Persist FreeCAD controls and reconstruct derived boundary display from the
   current loop payload. Verify Undo/Redo, copied/proxy graphs, physical parents,
   FCStd after external deletion and actual native export/reread. Derived OCC/Coin
   preview is never export authority or an implicit tessellation of the source.
5. Add editable native pattern resources/line content with declared shared versus
   independent identity/reference semantics. Preserve descriptions, signed dashes,
   offsets/base/directions and metadata; refresh every dependent hatch/closure.
   Establish an actual Rhino5 oracle for nonzero line bases/Plus before claiming
   full pattern appearance. Implement clipping against current outer/inner loops,
   full solid/line fills and display without silently dropping unsupported cases.
6. Extend the independent SDK2013 reader beyond its explicitly NURBS fixtures as
   required. Native semantic gates and actual Rhino5 rendering remain separate.
   Account for inactive/other gradient/userdata cases and version incompatibility;
   no automatic gradient loss or approximation is authorized by this plan.
7. Build/install, run applicable native/FreeCAD regressions and immutable user ring
   fixtures, synchronize scoped coverage/spec/README/ledger and checkpoint. Keep
   full class/category scope, resource/document/history/version work and final
   comprehensive review open until each original acceptance gate is verified.

## Execution contract after native curve probes

The first implementation prerequisite is now `hatch_loop_native`: recursive
typed source inventory for the five independent serializable curve classes.
Metadata and source curve identities are immutable provenance. This field is not
an editable overlay and must not be treated as completed loop editing.

For the staged editor, keep the existing five scalar fields compatible. Add an
explicit versioned loop payload with a verified source/baseline and bounded
included-file persistence. Stage complete current loop trees on exact native
clones before scalar fields and own placement. The request must distinguish
structure/type edits, numeric geometry edits, role/topology changes and preserved
child metadata. Native validation, exact reread and atomic replacement remain
mandatory. Cover NURBS CV/weight/knot/order/domain, Arc plane/radius/angular versus
curve domains, Line endpoints, Polyline points/parameters and nested PolyCurve
independent child/segment parameters. Add/remove and topology acceptance are
separate required tests, not an implicit guarantee from IsClosed/IsValid.

CurveOnSurface is applicable: the SDK probe constructs a valid closed dimension2
Hatch loop over a dimension2 NURBS surface. Its child surface and both curve
representations require their own explicit schema/reference policy. PolyEdge
records require object/component/domain resolution; plain PolyCurve casting or
DuplicateCurve conversion cannot establish lossless reference editing. Standalone
CurveProxy Write/Read returning false is an archive limitation to account for,
not permission to convert every source reference curve to NURBS.

Construct owning PolyCurve trees in place or through proven deep-copy operations.
Pinned SDK rvalue ownership was observed to leave dangling child pointers in the
first fixture; avoid it in the upcoming editor and retain the integration audit.
Current child plugin/UUID gates are fail-closed prerequisites. They do not satisfy
the remaining requirement to resolve/remap supported references and userdata.


## Hatch native loop foundation — 2026-10-07

- [x] Typed native inventory for NURBS/Arc/Line/Polyline/nested PolyCurve, exact
  class/domain/CV/weight/knot/segment parameters and child metadata identities.
- [x] Eight native mm/cm subtype roundtrips; FreeCAD54 new checks for current
  scalar fields/boundaries/copies/blocks/FCStd and child opaque-data rejection.
- [x] Reviewed nine curve registrations; proved CurveProxy serialization limit,
  unresolved PolyEdge reference decoding and valid2D CurveOnSurface Hatch loop.
- [x] Recursive child-data/reference gates and inactive gradient userdata check.
- [ ] Typed loop/topology editing, reference/surface adapters, safe plugin data,
  native pattern editing/full rendering, broader independent legacy5 and actual
  Rhino5 acceptance. Full openNURBS remains in_progress.

Current evidence: `docs/validation/2026-10-07-3dm-hatch-loops.md` — isolated
FreeCAD999/999 across45 suites; native18/18. No class-wide completion.


## Hatch typed loop editing progress — 2026-10-07

- [x] Staged native five-class payload, rational CV/weight/knot and independent
  domain edits, Arc radius, Polyline/Line points and recursive PolyCurve edits.
- [x] Named outer/inner addition/removal, disjoint roots/islands, crossing,
  retraced/self-intersecting/invalid field rejection with atomic native staging.
- [x] Bounded FileIncluded host persistence, baseline/schema/hash gates, native
  preflight before edit, Undo/Redo, independent object/family/shared proxy current
  loops and exact FCStd export after external source deletion in mm/cm.
- [x] Required FreeCAD included-file ownership fix; standalone copy/edit/delete/
  Undo/Redo/FCStd regressions plus App556 passing,2 skipped,7 disabled,0 failures.
- [ ] Typed ordinary editor UI, CurveOnSurface/PolyEdge/reference adapters,
  supported plugin data/remapping, native pattern editing/full fill rendering,
  broader independent legacy5 and actual Rhino5 acceptance remain pending.

Current evidence: `docs/validation/2026-10-07-3dm-hatch-loop-edit.md` — isolated
FreeCAD1155/1155 across49 suites; native19/19. Five-class script-adapter evidence
does not establish complete loop or class-wide support. Full goal in_progress.


## Hatch independent typed legacy5 loops — 2026-10-07

- [x] Unchanged SDK20130711 source280 files verified against original ZIP.
- [x] Native32 named legacy archives: prior20 Hatch/pattern/graph cases now also
  compare typed trees;12 new mm/cm original/normalized/edited/zero-base/canonical
  and independent copied-block paths preserve five native child classes exactly.
- [x] FreeCAD84 new checks for exact legacy decode/reencode of edited loops,
  child domains/metadata, zero base, block with inner Arc and copied families.
- [ ] Typed ordinary UI, CurveOnSurface/PolyEdge/reference/plugin adapters,
  broader subtype/precision/version cases, native pattern content/full rendering
  and actual Rhino5 application acceptance remain pending. No class-wide finish.

Current evidence: `docs/validation/2026-10-07-3dm-hatch-legacy-loops.md` — isolated
FreeCAD1239/1239 across50 suites; native20/20. Full128/16/6 scope in_progress.

## Ordinary editor execution contract — 2026-10-07

Continue step4 inline under the approved full exchange design. C++/Qt owns a
modal native-field tree, while Python only binds document persistence and undo.
Double-click and the Hatch context menu route to the same editor. Detached loop
values keep exact source indices/native types and immutable metadata. Numeric
fields use17-digit display, but unchanged fields retain their original JSON
numbers rather than being parsed back from the widget. Array indices and field
names are identities, not editable labels. Boundary role choices and explicit
circle addition/removal retain original surviving provenance; new circles have
null source indices. Native topology and child-data gates run before acceptance.

Commit revalidates the host and current baseline, then writes one document
transaction. Cancel, invalid fields, native rejection or a stale open editor
must not change the included file. Invalid acceptance stays in the same dialog
with its native error. Millimeter local coordinates and radian angles are
explicit. The initial numeric tree/add-circle controls do not establish the
remaining arbitrary row/segment creation or native type/reference/surface UI.
The table is bounded to65536 nodes and depth64, with an explicit refusal rather
than dropped values; larger bounded API payloads still need a virtualized editor.

RED/GREEN host evidence must exercise the real Qt dialog, native export/reread,
mm/cm, unchanged precision, cancellation, geometry rejection, Undo/Redo and
source immutability. Full128/16/6 requirements and actual Rhino5 acceptance remain
open; no public integration or final package review is authorized by this slice.


## Ordinary Hatch loop editor — in progress, 2026-10-07

Detached C++/Qt fields and Python commit/undo binding are implemented in the
isolated development source. The new UI regression has crash/unfinished-run
evidence and is not accepted. Existing API132/132 passes on the changed module,
process0. Prior GUI1239/native20 evidence is historical for the legacy-loop
checkpoint, not a full regression of this UI source. See
`docs/validation/2026-10-07-3dm-hatch-loop-ui-in-progress.md` for failing artifacts,
counterprobes and the next diagnostic action. Full128/16/6 scope remains active.


## Ordinary native Hatch numeric controls — verified scope, 2026-10-07

- [x] C++/Qt tree for current five-class numeric fields, boundary roles and
  explicit circle insertion/removal; detached data/native preflight before one
  document transaction, cancel/error/stale-host guards and Undo/Redo.
- [x] FreeCAD104 new UI assertions: mm/cm exact native reread, child strings,
  copied fields and FCStd/editor/export after external source deletion.
- [x] Failed-driver traceback isolated deleted native tree-item wrappers;
  QModelIndex/model actions retain real Qt/native checks. Nonfinite parser
  floating environment remains unchanged in the named probe. Full process0.
- [ ] Arbitrary CV/knot/point/segment insertion, native class/rationality changes,
  surface/reference/plugin adapters, native pattern/full fill rendering and
  actual Rhino5 acceptance remain pending. This is not all-UI/class completion.

Current evidence: `docs/validation/2026-10-07-3dm-hatch-loop-ui.md` — isolated
GUI1344/1344 across51 suites; fresh native20/20. Full128/16/6 scope in_progress.

## Structural row execution — 2026-10-07

Continue approved step4 inline: explicit Duplicate selected row / Remove selected
row for native NURBS CV/knots, Polyline points/parameters and PolyCurve segments/
parameters, recursively. Duplication copies detached raw values and original
source indices; it is not automatic knot refinement or curve conversion. Users
can edit the copied fields and paired arrays before confirmation. Fixed Line
endpoints, Arc plane/domain and source/type metadata cannot change cardinality.
Never remove the last raw row; native cardinality/domain/closure/topology checks
remain the commit authority. Precheck editor size/depth before replacing the
detached tree, so a refused structural operation cannot truncate its values.

RED `three_dm_hatch_loop_rows_smoke-1/f1fdfdfabc71481e80a47d50c9ba131f`
fails at the absent row action. GREEN must exercise actual paired Polyline point/
parameter insertion, NURBS CV/knot insertion/removal and PolyCurve child segment
splitting with preserved child source/domain/user strings, exact native export,
Undo/Redo, cancellation/invalid cardinality and FCStd after external deletion in
mm/cm. Generic new class/rationality, reference/surface/plugin controls remain
pending; this raw structural editor is an OpenMatrix9 implementation choice.

## CurveOnSurface archive prerequisite — 2026-10-07

Continue the full surface/reference adapter package from the proved optional
child Read failure, before curve/surface schema editing. Native RED is
`OM9-FILE-012.CurveOnSurfaceArchive`: exact child payload recovery fails against
the original pinned library. The compatibility format already stores all three
children; silently omitting the approximation or changing the class is invalid.

Ruling: replace only the defective Read function in an audited build-generated
copy of the SDK translation unit — the ClassId factory and nested model readers
must invoke the correction, and public archive APIs provide no factory override.
Verify normalized upstream source SHA before applying the patch, keep original
SDK source unchanged and retain pristine diagnostic libraries/readers. Cost:
compiled modern SDK now contains a declared application-owned repair, so earlier
"unmodified reader" binary evidence is historical, not a claim for the new module.
Do not patch the existing independent2013 reader; a separate repaired variant
may verify this named field path without replacing the original oracle.

Decode every child into staged ownership, verify actual curve/surface type and
dimensions, allow only presence flags0/1, then commit once. Any wrong type,
truncated payload or invalid resolved geometry must leave existing children intact.
Known serialized PolyEdgeCurve/PolyEdgeSegment children with dimension0 retain
identity and raw payload for later owning-model reference resolution; this is
archive recovery, not a declaration of semantic validity or export support.
RED/GREEN must include2D/3D, absent/present approximation, archive5/6/7, native
ClassId factory and full model write/read/reencode, exact child classes/fields/
tags and evaluated points. Source archives remain immutable. Full Hatch schemas,
OCC display/editing, transforms/references/plugins and Rhino5 app acceptance stay
open after this prerequisite; no all-class completion or public integration.


## Native Hatch raw structural rows — verified scope, 2026-10-07

- [x] Explicit detached duplicate/remove for existing NURBS CV/knots, Polyline
  points/parameters and PolyCurve segments/parameters; preserve source identity
  and exact child metadata, fixed-cardinality refusal and prechecked UI bounds.
- [x]103 new real UI checks: mm/cm paired edits, atomic native rejection,
  Undo/Redo, cancel, FCStd/source deletion;12 independent SDK2013 archive paths
  directly compare exact child trees and native decode/reencode.
- [ ] New native class/rationality/complete child creation controls, reference/
  surface/plugin adapters, broader rational/precision/version/performance cases,
  native pattern/full fill rendering and actual Rhino5 app acceptance remain.

Current evidence: `docs/validation/2026-10-07-3dm-hatch-loop-rows.md` — GUI1447/1447
across52 suites, native20/20. No full-class completion; full128/16/6 in_progress.
Earlier dated limitations are superseded only within this named scope.


## CurveOnSurface archive recovery — verified prerequisite, 2026-10-07

- [x] Generated, SHA-guarded replacement of Read only; original modern/SDK2013
  source unchanged, pristine modern library/original legacy readers retained.
  Compiled current libraries explicitly contain the application-owned repair.
- [x] Exact parameter/optional approximation/surface recovery in12 native2D/3D
  version5/6/7 cases;117 malformed/truncated refusals preserve existing children.
  Two unresolved PolyEdge child records retain reference IDs, pending resolution.
  Four separate repaired SDK2013 field/evaluation/reencode checks pass.
- [x] Correct retained capability;32 FreeCAD checks include FCStd after external
  source deletion and atomic refusal of unverified selected export.
- [ ] Native child/surface/reference/plugin schema, composed CAD display/edit,
  coupled surface/approximation transforms and verified export remain pending.
  Archive recovery does not establish semantic validity of deferred references.

Latest evidence: `docs/validation/2026-10-07-curve-on-surface-archive-recovery.md`:
GUI1479/1479 across53 suites and native21/21. Full128/16/6 remains in_progress;
actual Rhino5 application acceptance, complete Hatch/pattern/render and all
remaining class/component/document/version requirements remain required.
Earlier dated SDK-reader limitations are superseded only in this named scope.

## CurveOnSurface coupled transforms — inline continuation, 2026-10-07

Archive recovery checkpoint269 files is verified. Continue the full adapter:
first reproduce stale m_c3 through the real transformNativeGeometry dispatcher,
then stage native children and update the surface and optional approximation
together. Keep UV parameter fields unchanged when native surface parameterization
is unchanged; preserve native class/domain/userdata and out-of-plane dimensions.
Refusal must leave the original children and metadata intact.

Audit every concrete owned surface representation: NURBS, Plane, Rev, Sum and
Extrusion, including nested child curves, shape-preserving affine limits and any
parameter remapping required by the native representation. Reference/proxy
surfaces/curves require owning-model linkage and dependency remapping before use.
No implicit type conversion may hide a native transform limitation. This work
does not remove the export gate until the full child/schema/reference pipeline
and independent target-version writer are verified. Full128/16/6 remains required.

- [x] RED/GREEN coupled transform for rational/nonrational NURBS2D/3D and optional native
  approximation, native affine/out-of-plane placement and atomic failure.
- [ ] Remaining native surface representations and child metadata/reference
  audit; explicit parameter mapping and native class-transform limits.
- [ ] Native schemas, composed display/current edits, owning-model references,
  plugin/version guards, selected/native-block export and independent Rhino5
  verification; then continue all full Hatch and all128/16/6 requirements.


## CurveOnSurface coupled native transforms — verified scope, 2026-10-07

- [x]159 native2D/3D rational/nonrational degree1/3 NURBS and named Plane/Rev/
  Sum/Extrusion transform paths; move model surface and optional Arc/NURBS
  approximation together, keep original UV child and native class/domain/raw
  weights/knots/metadata within tested parameter-preserving operations.
- [x]32 atomic refusals: unsafe userdata, owning aliases, depth/metadata budgets,
  unresolved PolyEdge and unsupported native parameter/type-transform operations.
  Native metadata transforms exactly once; UV metadata stays unchanged.
- [ ] Full CurveOnSurface schema/current host edits/display/export, all surface/
  reference/plugin adapters, parameter rebasing and independent target/application
  verification remain required. Export gate stays. No full-class completion.

Current evidence: `docs/validation/2026-10-07-curve-on-surface-coupled-transforms.md`:
native26/26; GUI1479/1479 across53 process0 suites. Full128/16/6 remains in_progress.


## CurveOnSurface native schema — inline continuation, 2026-10-07

The native schema test compiled and failed on the missing versioned field tree.
Expose stable source-unit fields for owned curve/surface children and metadata,
with exact class dispatch and bounded traversal. Preserve raw homogeneous CVs,
knots/domains/planes, native surface construction fields and optional absence.
Keep runtime caches, pointer identities and copy counters outside the schema.
Verify direct fields against written/read model inventory for all five concrete
surface kinds. This read-only schema does not remove the pending export gate,
establish editable/display capability or complete the full128/16/6 goal.

- [x] Native child/surface field tree and direct/model equality, ten cases.
- [ ] Deferred native references and dependency closure; current host lifecycle.
- [ ] Full editing/display/export and all remaining approved requirements.


## CurveOnSurface source native schema — verified scope, 2026-10-07

- [x] Versioned source-unit native tree:10 direct/model field paths across
  NURBS/Plane/Rev/Sum/Extrusion and optional approximation;7 explicit invalid,
  depth/cycle/numeric/metadata refusals. Stable child metadata included.
- [x]8 deferred PolyEdge parameter/approximation reference paths: exact UUID,
  component index, domains and both reversed states; source dependency presence
  and missing-target diagnostics. SHA-guarded modern SDK Read flag repair.
- [x]120 FreeCAD schema/source-record/FCStd/source-deletion and atomic refusal
  checks. Native27/27; GUI1599/1599 across54 process0 suites.
- [ ] Owning-model reference resolution, complete native/current edit/display/
  selected export, parameter/type mappings, plugin fields and independent target/
  actual Rhino5 acceptance. Full128/16/6 remains in_progress. No full-class claim.

Evidence: `docs/validation/2026-10-07-curve-on-surface-native-schema.md`.


## Native reference resolution — inline continuation, 2026-10-07

Schema checkpoint336 files verifies. New real source-model fixtures compile and
fail because detached linked reference geometry is absent. Resolve native PolyEdge
UUID/component/domain/direction against the owning model into a separate lifetime-
owned graph, leaving source schema/model deferred and untouched. Support native
curve ownership and Brep edge/trim links with explicit validation, graph cycles,
missing/wrong targets, parameter bounds and shape/class preservation. Evaluate
native linked geometry and prove source-model/host persistence and target-version
roundtrip independently before enabling any selected writer path. Existing gate
stays during this work. Full128/16/6 and all approved requirements remain required.


## Native reference graph — verified source-model scope, 2026-10-07

- [x] Detached lifetime-owned graph resolves4 native CurveOnSurface PolyEdge
  parameter/approximation paths and4 real Brep box edge/trim paths, both directions.
  Native points, EdgeParameter, source schema and version5 record roundtrip exact.
  Two lifetime tests retain owner geometry after releasing inventory handles.
- [x]8 wrong/missing/cyclic/component/domain/dimension references diagnosed before
  host mutation;88 host persistence/refusal checks. Native28/28 and GUI1687/1687
  across55 process0 suites; source model stays deferred and unmodified.
- [ ] Full reference graph/types/UV/trim mapping/remap/shared-target budgets,
  current host edit/display/export, plugin/history data, independent target and
  actual Rhino5 acceptance remain required. Full128/16/6 remains in_progress.

Evidence: `docs/validation/2026-10-07-native-reference-resolution.md`.


## Native graph sharing and parametric integrity — inline continuation

Native reference checkpoint378 files verifies. A real three-root fixture fails
because each root clones the same owner. Share validated owner clones across roots
while every returned graph retains its full target lifetime closure. Add aggregate
SDK-accounted clone/owner/node budgets before allocation, with deterministic
explicit limit results; do not claim a global RSS bound. Validate Brep proxy/edge
subdomain consistency before returning linked status. Preserve immutable source,
isolated failure, stable source schema and existing selected export gate. Expand
real shared/nested/failure/ownership and parametric tests, then native/host regressions.
Full128/16/6 plus all current edit/display/remap/target/application requirements stay
active; this does not replace full reference or class compatibility acceptance.


## Shared native graph integrity — verified scope, 2026-10-07

- [x] Three roots share one detached native target, exact per-root closures retain
  lifetime; nested root reuses its owning CurveOnSurface and transitive closure.
- [x] Six aggregate SDK-accounted byte/clone/node/depth quota cases; limits checked
  before the corresponding work. Failed attempts remain charged across roots.
  SizeOf accounting is not a global heap/RSS bound.
- [x] Four straight box edge cases verify affine edge/proxy parameter intervals,
  both directions. Inconsistent in-bounds fields refuse before host mutation.
  New host38/38; complete native29/29 and GUI1725/1725 across56 process0 suites.
- [ ] General trim/UV/seam/singular/reference classes, edits/remap/export,
  independent target and actual Rhino5 application acceptance remain required.
  Full128/16/6 remains in_progress; class-wide completion is not established.

Evidence: `docs/validation/2026-10-07-reference-graph-integrity.md`.


## Trim/edge domain correspondence — inline continuation

Verified prior shared-graph checkpoint413 files. A real quadratic NURBS face
fixture has a native-valid reference with an in-bounds full trim interval that
does not correspond to the strict edge interval; current resolver incorrectly
links it. Validate trim subdomain endpoint correspondence via exact native C2
and owning surface evaluation, trim m_bRev3d and edge subdomains, using source
model/edge tolerances and roundoff. Expose endpoint proof separately from still
unverified interior parameter mapping. Preserve exact source, class and lifetime;
test curved planar/nonplanar surfaces and independent edge/C2/segment reversals.
Full128/16/6, nonlinear interior/seam/singular/type/reference mapping, editable
host and selected export plus independent/application acceptance stay required.


## Native curved trim domain correspondence — verified scope, 2026-10-07

- [x]32 planar/nonplanar quadratic face domain cases:16 valid and16 wrong trim
  intervals; C3 proxy, topological trim and segment reversal, four independent
  domains, exact analytic geometry and lifetime.5 model/edge tolerance cases.
- [x] Native endpoint correspondence via C2 and owning surface, source-unit
  model/edge tolerance and roundoff; explicit interior_mapping=unverified.
 176 host persistence/refusal checks; native30/30 and GUI1901/1901 across57 suites.
- [ ] General nonlinear interior trim/UV maps, C2 proxy reversal/native Brep
  validity audit, seams/singular topology, current host edit/display/export,
  independent/application acceptance and full128/16/6 remain required.

Evidence: `docs/validation/2026-10-07-trim-domain-correspondence.md`.


## Independent SDK2013 native reference interoperability — continuation

Verified trim checkpoint496 files. New separate SDK2013 reader links no modern
OM9 decoder and keeps existing pristine/Hatch/CurveOnSurface readers unchanged.
First reversed PolyEdge fixture fails exact native field comparison: true source
flag becomes false. Preserve that negative binary/library/report and original280
SDK source hashes. Add SHA-guarded generated SDK2013 Read correction using public
domain/Reverse APIs (old header lacks a flag setter), explicitly declare both Read
corrections. Decode native classes/fields and reencode36 named archives; compare
full modern native schema/metadata/reference analysis, then host source-deletion
FCStd lifecycle on independently reencoded archives. This is declared repaired
library evidence, not pristine reader or actual Rhino5 application acceptance.
Full128/16/6 and current native editing/display/remap/export/all remaining packages
remain active. The source/selected writer gate stays while its adapters are pending.
