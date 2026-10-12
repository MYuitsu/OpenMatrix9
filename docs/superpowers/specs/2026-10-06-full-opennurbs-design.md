# OM9-FILE-012 — comprehensive openNURBS exchange

Status: written specification approved for continuation on 2026-10-06 by the user’s instruction to continue after the written-spec review handoff. The preservation writer/structural-block plan proceeds inline under the existing selected execution method. Current partial evidence includes source-preserved/native structural exchange and explicit origin/affine-target migration and supported recursive namespace forks/shared CAD/mesh/native NURBS graphs shared standard/menu/CMD preservation routing and supported explicit legacy host upgrades and scoped native TextDot/PointCloud placements and independent selected members/proxies/2D-to3D transforms: see `docs/validation/2026-10-07-3dm-hatch-affine.md`. Comprehensive class/category coverage, later semantic packages and final integration/acceptance remain in progress.

## User objective and completion meaning

The user primarily works with 3DM files and requests complete openNURBS support in OpenMatrix9/FreeCAD. The design keeps Rhino 5 as the export target and adds both editable native representations and preservation of data that FreeCAD cannot edit. This is an OpenMatrix9 extension, not reconstructed Matrix behavior.

“Complete” means a versioned coverage inventory accounts for every geometry and model-component category exposed by the pinned openNURBS revision. Each row must have implemented and tested handling, or a documented compatibility rejection when the target format cannot represent it. No successful exchange may silently discard input data within its advertised scope. An opaque preserved record is not an editable FreeCAD feature and must never be reported as such.

Scope is comprehensive 3DM exchange supported by openNURBS, not reimplementation of the entire Rhino modeling SDK. Proprietary plugin execution and recomputation of Rhino/Matrix history are separate systems. Lossless editing of arbitrary undocumented plugin payloads cannot be guaranteed; preserve supported raw records where dependencies remain valid, otherwise reject the affected export with a precise explanation.

## Baseline and source evidence

- Source: `Gui/ThreeDmArchive.{h,cpp}`, `Gui/ThreeDmPython.cpp`, `ThreeDm.py`, Rust command policy and existing native dialogs.
- Current exchange model stores only CAD/mesh plus a small metadata subset. Embedded blocks are flattened; other model geometry fails import. Many non-geometry tables are not exported.
- Pinned openNURBS: `eb92af3ba1806b0a34a99aba0d3bda83e3d46083`. `opennurbs_model_component.h` enumerates Image, TextureMapping, Material, LinePattern, Layer, Group, TextStyle, DimStyle, RenderLight, HatchPattern, InstanceDefinition, ModelGeometry, HistoryRecord, RenderContent, EmbeddedFile and SectionStyle. Unset/Mixed/count and obsolete entries are not real content categories.
- Settings, properties, views, object attributes and userdata also require inventory; enumerating component types alone is insufficient.
- Current validation: unmeshed ring135 objects/547 checks; fixed ring101 objects/347 checks; host33 checks; five native suites. See `docs/validation/2026-10-06-3dm-blocks.md`.
- Official context: https://developer.rhino3d.com/guides/opennurbs/what-is-opennurbs/ and https://developer.rhino3d.com/guides/opennurbs/migration-guide/ . Local pinned headers govern implementation; documentation for newer revisions must not establish unsupported capabilities.

## Architecture

### Native archive model and inventory

C++ retains an immutable source archive and a manifest of object/component UUIDs, references, source units/version and preservation status. Inventory all model geometry, component tables, properties/settings/views, userdata and embedded/external resource references before document mutation. Classify each record as native editable, native display with retained source, opaque preserved, or incompatible/unreadable. Present a concise import report; unsupported editable behavior must not silently become success by meshing.

The manifest is schema-versioned. Stable source UUIDs identify records independently of FreeCAD labels and list positions. Maintain document-local mappings, content signatures, source snapshots and dependency edges. Collision handling is explicit: two imported archives may share UUIDs; namespace imports internally, and remap output UUIDs only when every affected reference can be updated safely. Unknown dependencies make unsafe merges fail rather than corrupt payloads.

### FreeCAD document representation and persistence

Use native Part/Mesh/Points/annotation representations where their behavior matches the input. Represent block definitions with reusable definition containers and placed links/instances, including nesting. Nonuniform/reflected transforms must retain native geometry and the original instance transform even when FreeCAD Placement alone cannot express it. Keep the current expansion approach available for explicit geometry-only exchange; preservation mode must retain definition/reference structure.

For unsupported editable types, create a clearly identified document object containing a persistent native record and optional supported preview. Preview geometry is derived display data, never the authoritative archive payload. Do not fabricate empty CAD as a substitute for the source object. Objects with no preview still appear in the tree/report and retain selectable export identity.

Store source archive/resource bytes inside FCStd through included-file/document persistence facilities, rather than an absolute path to the user's original file. Persist manifest schema, IDs, block relationships, component/resource mappings and source signatures. Saving/reopening must work after the original3DM and temporary staging directories are removed. Documents imported in the previous geometry-only implementation remain usable; do not invent missing source records for them.

Rust owns exchange modes, capability/status policy, selection/dependency decisions and operation/report state. C++/Qt owns archive serialization, geometry conversion, dialogs and native transactions. Python remains the FreeCAD object-binding adapter; no Python Rhino geometry converter.

### Merge/export behavior

Reconstruct output from retained source records plus the current FreeCAD state. Unchanged source-backed objects preserve native geometry and attributes; changed supported objects replace the corresponding geometry using their stable mapping. Explicit deletion removes the object from output. Duplication creates a new identity while retaining applicable shared dependencies. Renaming, layer/color edits and placement changes update the corresponding fields without discarding unrelated data.

Preserved objects that cannot be edited must reject unsupported edits; an invalid source signature/dependency must produce an actionable conflict report. Transforming unsupported geometry is allowed only where the native class supports it and reference updates are understood. No stale source payload may silently override a supported edit.

`Export3dm` and the standard selected-export entrypoint continue to export the selected whole objects. Compute the transitive closure of required block definitions, member geometry, styles, layers, groups, materials, mapping and resources. Do not export unrelated top-level source objects merely because they are present in the retained archive. Cycles, missing dependencies or opaque references whose safe closure cannot be established must fail selected export. Exporting an entire project is a separate explicit mode; it does not change selection semantics or silently become the default.

Rhino5 output remains staged beside the destination, checked for version5/50, reread and semantically validated before atomic replacement. Any incompatible newer-only record produces a pre-export report and failure. A separately requested conversion can later supply a documented approximation; never silently down-convert or wrap newer content and claim Rhino5 native compatibility. Preserve the immutable source snapshot even when export is rejected.

### Compatibility reporting

Each category records read/preserve/display/edit/write/Rhino5 capability separately. The report identifies the source class, UUID/name, dependent components, target-format issue and applicable supported action. Retention of raw source bytes alone does not prove the data can be safely written into a merged Rhino5 archive. Successful output must verify semantic records, dependency resolution and data retention, not just file existence or object count.

## Coverage work packages and acceptance

| Package | Required handling | Required acceptance |
|---|---|---|
| Inventory/preservation foundation | All known model/component categories, schema, stable identity, immutable archive/resource persistence, native-only placeholders | Mixed-type inventory; data survives FCStd reopen with source removed; malformed/incompatible records fail without document changes |
| Structural blocks | Definition/member graph, repeated/nested instances, identity, nonuniform/reflected transforms, embedded linked definitions | Re-export real definition/reference structure, distinct placements, no duplicate originals; rename/delete/edit/FCStd tests; missing/cyclic/external-only definitions handled explicitly |
| Remaining geometry | Point clouds with available colors/normals, hatches and patterns, text dots, text, dimensions/leaders and style references, clipping/page/detail objects and every remaining geometry class in pinned inventory | Type-specific native fixtures; placement/units/bounds and style/content preservation; unsupported edit behavior explicit; no implicit CAD tessellation |
| Appearance/resources | Materials, textures, images/embedded files, texture mapping, object/layer material references, line patterns and render lights/content | Dependency closure; texture bytes/checksums and transforms; external-resource resolution explicit; no network fetch on import; reopen without original paths |
| Document/components | Full layer metadata, groups, text/dimension styles, views/cplanes/settings/properties and other applicable tables | Stable references and semantic field equality; selected vs whole-project exports distinguished; unit-normalization accounted for |
| History/userdata/plugins | History records, user dictionaries/strings and userdata that native archive supports retaining; opaque plugin geometry when native class/raw records are safely readable and writable | Retention and reference validity; unchanged records preserved; edits invalidate stale history appropriately; no claim of plugin/history execution; unsafe unknown dependencies fail |
| Format coverage | Per-class Rhino5 compatibility gates and coverage completeness | Every real pinned category accounted for; incompatible newer content fails before destination replacement; unknown future types are not silently ignored |

Rhino5 does not necessarily support every category available in the newer pinned library. The compatibility inventory must establish this using actual target-version writes and rereads. API availability must not be confused with target-format support.

## Input, units and lifecycle

Keep active-document/edit/task guards, conversion/preflight before mutation, one Undo transaction, cancellation without document/target changes and successful-only command history. Unitless/custom archives require explicit scale. Geometry/annotation/layout/resource transforms must declare how scale is applied; preserve original units in provenance and export millimeter geometry consistently. Do not accidentally apply unit scale twice to blocks or mapping transforms.

Maintain file-size/output-count/depth guards. Add bounded archive/resource/manifest sizes and clear errors for corrupt references, cyclic graphs and missing external resources. Resolve linked files/textures using an explicit user-provided or source-relative local path; source paths are data, not instructions, and do not authorize network access or arbitrary file copying.

## Validation and completion gates

1. Machine-readable inventory records every category/class discovered from the pinned implementation; class inheritance and table aliases must not create duplicate completion rows. Each row has source evidence, compatibility and test status.
2. Create independent native fixtures for each supported geometry/component category and interactions between categories. Test preservation after native archive reread and host FCStd save/close/reopen with the original source unavailable.
3. Retain both immutable user ring fixtures and existing menu/CMD, Undo/Redo, standard import/export, failure preservation and placement regression suites.
4. Geometry comparisons use optimal analytic bounds and adaptive area/volume integration; preserve the existing1e-3mm bounds threshold and max(1e-3,1e-4relative) area/volume thresholds for these rings. Add category-specific units/tolerances where relevant. Capture integration error estimates and ensure they are finite and below the tolerance supporting each claim.
5. Block validation compares definition/member UUID graph and accumulated world geometry. Opaque/table validation compares semantic fields, references, payload hashes where serialization supports stable bytes, and decoded equivalence otherwise. A whole-file byte hash is not an archive semantic oracle.
6. Exercise mixed-selection exports, source-backed object edits/duplication/deletion, multiple imported archives, ID collisions, unit changes, missing resources and incompatible target formats. Confirm selected output does not include unrelated top-level objects.
7. Build/install and execute the final module from the primary checkout. A coverage row is complete only after native plus applicable FreeCAD runtime tests pass. Reports distinguish preservation, display and editing support.
8. Run actual Rhino5 interoperability checks when that application is available. Until then report native version/semantic checks as verified and Rhino5 application behavior as untested.

## Rollout and limits

Implement foundation first, then structural blocks, remaining geometry, appearance/resources, document data and opaque/history handling, with regression gates after each package. The detailed implementation plan must choose concrete schemas/APIs and fixtures before coding each package. Existing source/worktree changes are uncommitted and must be preserved; this design does not authorize pushing or publishing.

Do not mark full coverage complete while inventory rows lack evidence. If a pinned class cannot be safely retained or written through openNURBS, record an explicit incompatibility rather than claiming universal preservation. Do not change reusable skill business rules for this expanded scope until implementation is verified and the user authorizes that specific skill update.


## Native TextDot fields progress — 2026-10-07

- [x] Persistent independent double anchor, Unicode primary text, font/height/display flags; derived native screen-text preview; canonical/current/copied members, physical proxies, namespaces, mm/cm, Undo/Redo/FCStd and atomic malformed/incompatible rejection.
- [x] Newer TextDot secondary text is retained on import and through FCStd; incompatible Rhino5 export still refuses before destination replacement.
- [ ] Exact Rhino font/background/hover appearance, TextDot in affine retained previews, point-cloud per-point editing/display, remaining geometry/resources/components/document/history/version semantics and actual Rhino5 application acceptance.

Evidence: `docs/validation/2026-10-07-3dm-text-dot.md` — fresh isolated FreeCAD724/724 across37 suites; native10/10,30.85s. Rust unchanged at prior75/75/fmt evidence. Native fields and source bytes remain independent of the derived Coin float preview. Full openNURBS and public integration remain in progress.


## Native PointCloud fields progress — 2026-10-07

- [x] Persistent independent double points/normals, exact RGBA bytes, intensity/plane/ordered state; derived colored PointSet; current canonical/copied/shared members, namespaces, mm/cm, Undo/Redo/FCStd and atomic malformed/provenance/singular rejection.
- [x] Bounded hashed field files, current-array inventory SHA256, source and reserved flags preservation, identity caches and valid dormant-plane normalization. Local point hiding/size persist only as display semantics.
- [ ] Verify Rhino5 intensity reader support and resolve the interim selected-closure export gate. Large-data memory/performance, affine retained previews, geometry-only cloud adapters and arbitrary legacy upgrades remain pending.
- [ ] Remaining annotation/geometry, resources/components/document/history/reference/version, actual Rhino5 application acceptance, final comprehensive review and public integration.

Evidence: `docs/validation/2026-10-07-3dm-point-cloud.md` — isolated FreeCAD767/767 across38 suites; native11/11,30.50s. Rust unchanged at prior75/75/fmt evidence. Native double fields stay independent of the Coin float preview. Full openNURBS remains in progress.


## PointCloud geometry and legacy5 reader progress — 2026-10-07

- [x] Independent unchanged McNeel SDK20130711 reader proves exact native cloud fields in mm/cm and intensity loss; precise shared rejection before writes replaces the interim unknown-reader gate. Actual Rhino5.14 application acceptance remains unverified.
- [x] Geometry-only current native double PointCloud alongside CAD/mesh; native nested affine/unit normalization, selected cloud/member/block routes, explicit userdata omission, exact cloud reread and atomic malformed-field/transform/RGB rejection.
- [x] Derived typed mixed affine cloud previews, automatic canonical refresh, native/full-field fingerprints, root delta, FCStd and verified copied-definition graph; edited/unverified preview copies remain guarded.
- [ ] Large-data streaming/performance, PointCloud viewport lock semantics, arbitrary cloud/preview legacy upgrades and actual Rhino5 application acceptance.
- [ ] Remaining geometry/annotations/styles, resources/components/document/history/reference/version audit, comprehensive review and public integration.

Evidence: `docs/validation/2026-10-07-3dm-cloud-geometry-legacy5.md` — isolated FreeCAD817/817 across39 suites; native13/13,31.87s; unchanged Rust75 evidence. Earlier dated pending PointCloud geometry/affine/reader notes are superseded within this tested scope. Full openNURBS remains in_progress.


## Hatch native references progress — 2026-10-07

- [x] Custom line HatchPattern UUID dependency and full named pattern fields; rational native outer/inner loops, plane/basepoint/rotation/scale/bounds, selected closure and exact native payload validation.
- [x] Native mm/cm pattern-spacing normalization, retained placement/parent, Undo/Redo, independent copy and FCStd after external source deletion; NaN/Infinity dash rejection.
- [ ] Hatch display/current-field/loop editing, general affine/reflected patterns, broader block/member/proxy, built-in/solid/gradient, corrupt-source/reference and child-loop userdata/legacy cases.
- [ ] Remaining full geometry/annotation/style/resource/component/document/history/version audit, actual Rhino5 acceptance, comprehensive review and public integration.

Current evidence: `docs/validation/2026-10-07-3dm-hatch-native.md` — isolated FreeCAD833/833 across40 suites, native14/14. Rust unchanged at prior75/75/fmt evidence. Native retention is not editable/rendering support; full openNURBS remains in_progress.


## Hatch native affine progress — 2026-10-07

- [x] Explicit native plane/UV loop rebasing for named determinant-one scale,
  reflection, shear/anisotropic and tilted-plane cases in mm/cm; isolated complete
  transformed line patterns and preserved siblings, stable generated UUIDs.
- [x] Native canonical/baseline copied-member closure and FreeCAD shared selected,
  promoted/copied block proxies, current placement/scale, Undo/Redo, FCStd and
  atomic singular errors. Published line-frame mathematical/serialized oracle.
- [ ] Actual Rhino5 pattern rendering/roundtrip oracle, class-wide built-in/solid,
  gradients, editable/display Hatch fields, child-loop userdata/refs, incomplete
  document native-reader repair and remaining full class/category requirements.

Current evidence: `docs/validation/2026-10-07-3dm-hatch-affine.md` — isolated
FreeCAD857/857 across41 suites; native15/15,36.07s. Earlier dated affine guards are
superseded only in this tested scope. Full openNURBS remains in_progress.


## Hatch current fields and boundary progress — 2026-10-07

- [x] Persistent native origin/axes/base/rotation/tiny positive scale and exact
  pattern reference selection; staged mm/cm current-field/copy/block/proxy routes.
- [x] Derived native rational boundary BRep/Coin outline, Undo/Redo, embedded
  FCStd/source deletion, schema/baseline/choice/invalid-field atomic rejection.
- [x] Gradient is explicitly retained without the ordinary adapter and this
  unverified Rhino5 export rejects before replacement; full gradient support open.
- [ ] Native loop/pattern-line editing, full pattern rendering, actual Rhino5
  renderer/roundtrip and remaining full class/category semantics and acceptance.

Current evidence: `docs/validation/2026-10-07-3dm-hatch-current.md` — isolated
FreeCAD921/921 across43 suites; native16/16. Earlier dated pending current-field/
preview notes superseded within this scope only; full openNURBS in_progress.


## Hatch legacy5 semantic progress — 2026-10-07

- [x] Independent unchanged SDK201307115 native20 mm/cm exact Hatch/loop/pattern/
  block decode-reencode cases, including current tiny scale and named builtins.
- [x] Typed legacy movable-base representation and public attribute ownership
  repair; separate raw-loss and repaired evidence retained without SDK edits.
- [x] Enabled gradient type/colors/repeat are lost at the modern version5 boundary;
  target-version incompatibility and atomic production rejection are established.
- [ ] Actual Rhino5 rendering, native loop/pattern-line editing, full pattern
  rendering, broader legacy/userdata/repair and remaining full class/category work.

Current evidence: `docs/validation/2026-10-07-3dm-hatch-legacy5.md` — isolated
FreeCAD945/945 across44 suites; native17/17. Full openNURBS remains in_progress.


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
