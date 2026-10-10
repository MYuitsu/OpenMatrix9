# OM9-FILE-012 — Rhino 5 exchange implementation

Current development scope and evidence: [CurveOnSurface coupled native transforms](../validation/2026-10-07-curve-on-surface-coupled-transforms.md),1479 GUI checks/native26. Current compiled SDKs include the declared Read-only repair; original sources remain unchanged. Dated sections below record earlier implementation stages; supported selections share preservation routing. Full compatibility and public integration remain pending.

User-authorized extension of Core Import and Export Selected. Native FreeCAD command delegation and FCStd Save semantics remain unchanged. Standard Open/Import/Export registers the 3DM host adapter.

Rust owns command identity, captions, completion and command permissions. C++ owns native dialogs, post-dialog document/task guards, the Undo transaction, openNURBS archive validation and OpenCASCADE geometry conversion. `ThreeDm.py` binds converted native geometry to FreeCAD object properties/groups and stages selected FreeCAD geometry for the native writer. It uses no Python Rhino dependency or Python geometry converter.

openNURBS revision: `eb92af3ba1806b0a34a99aba0d3bda83e3d46083`; license packaged in `Resources/licenses/openNURBS.txt`. Standalone SDK mode supports the existing FreeCAD build; a fresh build fetches the pinned revision through CMake.

Conversion tests cover rational curves, circles, independent openNURBS box/line fixtures, a trimmed face with a hole, closed box/cylinder/sphere volume, mesh counts, inch scale, unitless scale, nested layer states, unsupported annotations and malformed archives. Host tests exercise world placement, Undo/Redo, FCStd persistence, native menu/CMD dialogs, cancellation, file preservation and translated/nested meshes.

Restrictions and acceptance contract are recorded in the Core extension spec. Do not infer that all 130 original Core requirements are complete from this slice.

## Real user fixture and final validation

`tests/fixtures/3dm/Oval-twist-ring-fixed.3dm` is the immutable user-supplied regression fixture. Native and FreeCAD round trips retain all 101 objects; the real-file host suite passes 345 checks. The host behavior suite passes 32 checks, including registered standard entry points, task guards and suffix-aware overwrite confirmation. Original 3D edges are retained during BRep import; source/kernel tolerance is retained in `OM9Tolerance` and archive output. Staged output is converted and validated before replacement. See the validation record for limits and exact reproduction commands.

Selecting imported layer groups and their contained objects exports the geometry once; plain layer groups are flattened, while overlapping Body/Tip selections are rejected.

## Embedded block support

User-authorized CAD expansion, native nested transforms and definition deduplication are implemented and verified. Final current evidence: `docs/validation/2026-10-06-3dm-blocks.md`; complete support/limitation inventory: `docs/features/3dm-support-matrix.md`. Rhino block structure is flattened, not round-tripped structurally.


## Development preservation progress — 2026-10-07

The separately tested `ThreeDm.export_preserved` route preserves native block structure and supported current source/member/definition edits. The standard/menu/CMD exporter retains its existing guarded conversion route. Explicit legacy migration now accepts verified origin mappings for copied source/definition identities; independent copies, original definition metadata, physical references and unchanged rational proxy payloads have native reread, Undo/Redo and FCStd evidence. Full openNURBS remains in progress; affine copied-target mapping, recursive copied ownership/legacy upgrades, linked resources, full reference/class/version coverage and actual Rhino5 application acceptance remain pending.

- [x] Supported copied-origin migration, independent edited point geometry and retained native definition metadata.
- [x] Original/copy Undo/Redo, stable IDs after FCStd/repeated migration, malformed mapping/property rejection before mutation.
- [x] Copied shared proxy retains exact native two-dimensional rational NURBS payload.
- [ ] Full openNURBS compatibility and shared standard/menu/CMD preservation workflow.

Evidence: `docs/validation/2026-10-07-3dm-copy-origin-migration.md` — isolated FreeCAD483/483 across29 suites, native8/8. Public integration is pending.


## Affine target migration progress — 2026-10-07

`ThreeDm.migrate_archive(..., targets={affine.Name: definition.Name})` explicitly resolves current affine targets during legacy/copy provenance migration. Supported original/new/migrated-copy definitions retain native structure and source matrices. Both mixed App::Part and Part::Feature affine previews are verified against native CAD/mesh output. Wrong/foreign/ambiguous/colliding targets reject; cyclic rebuilding rolls back fields, new IDs, placements, geometry and original display objects. Physical App::Link pointers remain unchanged. Full openNURBS stays in progress.

- [x] Explicit affine mapping to migrated copied, original imported and existing new definitions.
- [x] Native current geometry/matrices and target graph closure; Undo/Redo, FCStd and stable repeated migration.
- [x] Foreign/invalid identity/type rejection and cyclic preview rebuild rollback.
- [ ] Recursive copied archive-owner/legacy representation upgrades and full openNURBS class/reference/version coverage.

Evidence: `docs/validation/2026-10-07-3dm-affine-target-migration.md` — fresh isolated FreeCAD515/515 across30 suites, native8/8. Shared menu/CMD routing, final review/public integration and actual Rhino5 acceptance remain pending.


## Recursive archive namespace migration progress — 2026-10-07

`ThreeDm.migrate_archive(..., fork_namespace=True)` explicitly assigns supported recursively copied archive owners and their owned graph an independent persistent scope. Native provenance, edited geometry, physical target sharing, Undo/Redo and FCStd remain intact. Distinct owners sharing one namespace refuse preservation export. Native collision UUID aliases are stable for the same ordered source set.

- [x] Actual recursive whole-group copy, nested source ownership, persistent fork witnesses and idempotent migration.
- [x] Independent analytic geometry/native block export; Undo/Redo, FCStd, delete-original-owner and copy-of-fork behavior.
- [x] Foreign source/target/property/witness rejection and failed preview rollback.
- [ ] Arbitrary recursive shared-proxy/mixed graphs, historical representation upgrades and full class/reference/resource/version coverage.

Evidence: `docs/validation/2026-10-07-3dm-recursive-namespace-migration.md` — fresh isolated FreeCAD540/540 across31 suites; native8/8. Full openNURBS, shared menu/CMD routing, final integration and actual Rhino5 acceptance remain in progress.


## Recursive shared CAD/mesh and NURBS progress — 2026-10-07

Supported recursive namespace forks now have actual shared point, mixed CAD/mesh shear/reflection and exact rational NURBS evidence. One copied canonical payload remains shared by multiple definition proxies. Ordinary source proxy/member output IDs are deterministic per namespace/native UUID/internal host Name; separate independent proxy branches do not alias. Current edits and native provenance remain independent.

- [x] Actual recursive shared point and mixed CAD/mesh graph copies; restored native proxy metadata baselines and independent analytic geometry.
- [x] Undo/Redo, FCStd, repeated source-proxy/direct-member export identities and immutable archive bytes.
- [x] Native rational NURBS CRC retention across namespace forks and shared definitions.
- [ ] Arbitrary retained/plugin proxy families, historical representation upgrades and complete class/reference/resource/version coverage.

Evidence: `docs/validation/2026-10-07-3dm-recursive-shared-migration.md` — fresh isolated FreeCAD564/564 across32 suites; native8/8. Full openNURBS, shared export routing, final integration and actual Rhino5 acceptance remain in progress.


## Shared export route progress — 2026-10-07

Supported source/native block selections now use the preservation writer through default File Export, named/native export, menu and actual CMD execution. Default retains native selected graph/data. The OM9 save dialog and API offer explicit geometry-only omission; verified native expansion keeps affine CAD/mesh coordinates and mesh types. No whole-archive selection is inferred from an owner container.

- [x] Standard/named/native/menu/CMD preservation routing with real selected native geometry/graph evidence.
- [x] Explicit geometry-only disclosure/default, native block flattening and mixed mesh coordinates.
- [x] Task/project/subelement/manual-preview guards, atomic refusal, cancel/overwrite lifecycle and FCStd without external input.
- [ ] Historical representation upgrades, standalone member/retained transforms and full class/reference/resource/version coverage.

Evidence: `docs/validation/2026-10-07-3dm-shared-export-routes.md` — isolated FreeCAD593/593 across33 suites, native8/8. Full openNURBS, final public integration and actual Rhino5 acceptance remain in progress.


## Explicit legacy host upgrade progress — 2026-10-07

Explicit migration upgrades now replace supported older CAD/mesh/flattened instance holders with the native representation verified from their included archive. Exact host Names, native UUIDs, immutable source matrices and original provenance survive; current canonical geometry and supported incoming references remain current. Instance placement interpretation is explicit; preview replacement requires explicit rebuilding. Missing current geometry, unsupported dependencies/scripts/custom payloads and ambiguous affine placement refuse. One transaction includes replacement/rewiring, Undo/Redo, FCStd and late-failure rollback.

- [x] Supported physical, CAD/mesh and mixed affine host upgrades with independent native geometry/placement evidence.
- [x] Incoming links/groups/custom references, current color/property documentation, idempotence, Undo/Redo/FCStd and injected rollback.
- [ ] Actual historical user documents, arbitrary legacy/script/plugin/array/subelement adapters and full openNURBS coverage.

Evidence: `docs/validation/2026-10-07-3dm-host-upgrades.md` — fresh isolated FreeCAD632/632 across34 suites, native8/8. Full openNURBS, final public integration and actual Rhino5 acceptance remain in progress.


## Native retained placement progress — 2026-10-07

TextDot and PointCloud retained selections now export an explicit native placement delta. Canonical block members use definition-local deltas; selected roots compose world parents once. Native copies preserve coordinates, inverse-transpose normal direction/magnitude, colors and supported userdata; member/reference bounds refresh. Source signatures/bytes stay immutable, altered opaque payloads and invalid/unreachable transforms refuse atomically. This supplies native placement support, not text/per-point editing or display.

- [x] Scoped selected-root/canonical-member native transformations, unit normalization, copied PointCloud normals/colors, UUID/closure, Undo/Redo and FCStd.
- [ ] Definition-member-only selection, complete retained class/field/display/edit/resource/reference/version coverage and actual Rhino5 application acceptance.

Rhino5 source TextDot text fields survive. Nonempty secondary text in a Rhino6 source remains protected by whole-version rejection; it is not advertised as Rhino5 compatible. PointCloud runtime hidden flags/intensities and newer fields are outside this verified slice.

Evidence: `docs/validation/2026-10-07-3dm-retained-transforms.md` — fresh isolated FreeCAD657/657 across35 suites, native9/9, Rust75/75 and fmt exit0. Full openNURBS and final public integration remain in progress.


## Selected native definition-member progress — 2026-10-07

Supported canonical CAD/mesh/native retained members and physical shared proxies now export separately as independent native top-level records. Stable selected UUIDs keep original membership/provenance intact. Current local payload plus physical world/proxy placement is applied once; unrelated ancestors are omitted. Mixed block/member selection retains both canonical and selected identities, and nested references retain current required targets. New explicit members have separate stable selected identities. Out-of-plane native2D curves promote to3D without refitting or changing rational weights/knots; geometry-only block reading uses the same correction.

- [x] Canonical/current point/CAD/mesh, TextDot/PointCloud mm/cm, rational/reflected shared proxies, nested/current new targets, distinct namespaces, mixed selection, Undo/Redo/FCStd and actual native/menu/CMD/standard export geometry.
- [ ] Arbitrary retained/plugin graphs, link arrays/subelements, complete geometry/field/display/edit/resource/component/document/history/version coverage and actual Rhino5 application acceptance.

Evidence: `docs/validation/2026-10-07-3dm-selected-members.md` — fresh isolated FreeCAD690/690 across36 suites; native9/9,29.36s. Unchanged Rust source/tests retain prior75/75/fmt evidence. Full openNURBS and final public integration remain in progress.


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

## Current full-exchange execution checkpoint — 2026-10-07

Earlier counts and `interior_mapping=unverified` statements above describe their
original historical runs. The current property catalog and package report are
authoritative for the expanded, bounded scopes below; no class-wide completion
or completion percentage follows from the counts.

- [x] Discovery:131 source registrations,128 actual runtime classes, with
  commented/abstract/helper/obsolete applicability and evidence separated.
- [x] Thirty no-C3 surface/UV profiles through actual Rhino5 API and GUI;
  native decoded fields and current FreeCAD lifecycle verified. Coherent C3
  reader omission remains an explicit target incompatibility guard.
- [x] Nonlinear/reversed native trim correspondence with independently checked
  interior samples and closed-disk seam endpoints; no global certification.
- [x] Four analytical curved two-shell fixtures through actual Rhino5 GUI,
  precise eight-file Rhino measurement and native/FreeCAD reimport. Keep the
  original default-volume comparison1/4 separate; analytical threshold1e-6mm³
  unchanged. Corrected GUI measurement adapter has not been rerun in Rhino.
- [x] Native-only member/copy/proxy placement, Undo/Redo, tamper refusal and
  portable FCStd. Independent V5 fixture verifies reflected SDK80 staging and
  explicit Geometry only V5 output as two valid CAD cavities with separate UUIDs,
  exact bounds and784mm³ each. Inward preservation still refuses atomically.
- [x] Six owning nested UV controls: exact source fields and native placement
  survive Undo/Redo and FCStd after external source deletion.
- [ ] Actual Rhino5 retention of these six nested controls; V5 guard remains.
- [ ] Full reference remap/current editing/global mapping, general shell
  applicability, remaining geometry, annotation/resources/document/history,
  final review, primary source integration and installed acceptance.

Fresh native42/42, isolated FreeCAD26 reports/2082 checks including both original
ring fixtures894 checks, Python coverage6 plus measurement/evidence5 tests.
Ring bounds remain0.001mm. Source hashes agree with captured proof and installed
Python policy. Primary source has not been integrated or pushed.

Evidence: `docs/3dm-capabilities.json`, `docs/3dm-capability-slices.json`,
`docs/validation/2026-10-07-opennurbs-packages-1-3.md`,
`docs/validation/opennurbs-current-proofs-20261007/summary.json`,
`docs/validation/rhino5-curved-shells-20261007/combined-acceptance.json`.

## Checkpoint 2026-10-08: restricted nested UV

- [x] Six nested controls pass actual Rhino5 API and exact decoded FreeCAD
  reimport (25 checks), including native dependencies and source-deleted FCStd.
- [x] Restricted no-C3 nested Line/bilinear 2D UV profile: selected V5 unchanged/
  moved native writer (143 checks), actual host lifecycle (60 checks), stable
  top-level-copy UUID across export/reopen and atomic geometry/layer collisions.
- [x] Actual GUI Open/SaveAs of twelve OM9 writer outputs; full decoded native
  reimport73 checks with source-deleted FCStd and opaque plugin safety guard.
- [ ] General nested UV types, mapping/remap and remaining approved packages.

This supersedes the older six-control target-pending entry only for the profile
above. Fresh final native 42/42, serial FreeCAD 27 reports/2,131 checks and Python
11 tests pass. Both rings retain 894 checks at 0.001 mm bounds; no tolerance is
relaxed. Primary source is not integrated or pushed; full exchange remains open.

Evidence: `docs/validation/2026-10-08-opennurbs-nested-uv-profile.md`,
`docs/validation/opennurbs-current-proofs-20261007/summary.json`,
`docs/validation/rhino5-nested-profiles-20261007/`,
`docs/validation/rhino5-nested-writer-20261008/` (GUI result pending).

- [x] Independent expanded nested90 fixtures and FreeCAD721 retention checks.
- [x] Actual Rhino5 API90 and decoded reimport361; public V5/lifecycle900 for
  bounded single-span positive-weight maps. Other properties remain guarded.
- [x] Actual GUI Open/SaveAs of180 OM9 writer outputs and decoded native/FreeCAD reimport1081 checks.

Evidence: `docs/validation/2026-10-08-opennurbs-expanded-nested-uv.md`.

Current fresh native42/42, rebuilt installed module and runtime30 reports/3465
checks. Evidence: `docs/validation/2026-10-08-opennurbs-expanded-v5-admission.md`.

Actual expanded GUI180/180 and exact decoded reimport1081 now pass. Current native42 suites and31 runtime reports/4546 checks retain unchanged ring thresholds. Standalone PolyEdge top-level resolution/dependency adapter and remap remain open;12 independent native controls prepared for actual Rhino5 document acceptance. See `docs/validation/2026-10-08-opennurbs-expanded-gui-polyedge.md`. No full-support or primary integration claim.

Measurement reporting queue: `docs/validation/opennurbs-test-roadmap.md` and JSON track named batches, fixture results, pending prepared count and unclosed packages after each attempt. Current standalone diagnostic retry remains pending; total future batch count unknown. Reporting does not advance compatibility status.

Standalone PolyEdge scoped native ownership/typed fields/retention verified825 native and121 host checks. Actual Rhino12/12 roots invalid;0 passed, class/reversal loss decoded. Atomic common V5 refusal retained. General edit/copy/remap and remaining package2 requirements stay open. Evidence: `docs/validation/2026-10-08-opennurbs-standalone-reference-support.md`.

Final standalone checkpoint: native43/43, installed runtime32 reports/4667 checks, rings894 unchanged thresholds; relevant Python18 pass. Next mixed ON_PolyCurve/reference-child6 target controls prepared; parent adapter/target unverified. Full remains incomplete.

Mixed ordinary PolyCurve/reference-child typed adapter and bounded native owner graph, preserved public reader and atomic V5 guard verified363 native/64 host checks. Actual Rhino6/6 invalid, child class/metadata loss6 and reversal loss3;3 disconnected saved parents rejected before host mutation. Final native44/44;33 runtime reports/4731 checks; both rings894 unchanged thresholds. Whole tools41/42 known UI route reference gap.0 prepared pending batches,7 packages open; full incomplete. Evidence: `docs/validation/2026-10-08-opennurbs-mixed-reference-support.md`.

UI menu reference follow-up2026-10-08: actual configuration `Resources/menu/MainMenu.ini` was already present; obsolete `ref/MainMenu.ini` routing corrected. Primary/dev decoded menu configuration matches18 groups/11 quick icons. Whole tools42/42 now passes; earlier41/42 logs preserved. No runtime menu/3DM/SDK change or Rhino rerun required. [Evidence](../validation/2026-10-08-ui-menu-reference-route.md).

Copy/remap checkpoint2026-10-08: namespace owner UUID bug and silent snapshot restoration fixed. Native36/36 cases (18 originals),2611 checks; FreeCAD18/18 lifecycle cases,612 checks. Final45/45 native;34 reports/5343 checks; tools42/42. Public V5 incompatibility remains; native owner geometry edits refuse safely. Packages2–8 open,total future batches unknown. Evidence: [docs/validation/2026-10-08-native-reference-remap.md](../../docs/validation/2026-10-08-native-reference-remap.md).

Seam preparation checkpoint2026-10-08:8/8 native(229 checks),8/8 FreeCAD(32 checks), actual Rhino5 pending. Current regression46/46 suites,35 reports/5375 checks; same verified production binary.1 prepared pending batch,7 packages(2–8) open,total future count unknown. Source/fixture bindings verified; no primary integration or push. [Evidence](../../docs/validation/2026-10-08-seam-profiles-preparation.md).

Seam actual checkpoint2026-10-08:Rhino5 GUI8/8 passed,no bad objects. Exact saved native decode200 and actual FreeCAD reimport49 checks passed. Final47/47 native;36 reports/5424 checks.0 prepared batches,7 packages2–8 open,total future count unknown. Scoped no-C3 seam/pole/periodic exchange verified; singular reference and general CAD editing remain open. SDK unchanged,no primary integration/push. [Evidence](../../docs/validation/2026-10-08-seam-profiles-complete.md).

UV-reference checkpoint2026-10-08:native8/8(895),actual FreeCAD8/8(272) passed. Current graph serialization/lifetime and child userdata copy-count retention corrected; source retained. Final48/48 native,37 reports/5696 checks,tools42/42.1 prepared8-file target batch awaiting actualRhino5;7 packages open,total future batch count unknown. PublicV5 still guarded; no full/primary integration/push claim. [Evidence](../../docs/validation/2026-10-08-uv-reference-remap.md).

Actual UV-reference8:Rhino read8,0passed/8invalid roots,8valid owners,8SaveAs command failures,0saved outputs;no saved reimport. Native507 source-retention/atomicV5-refusal checks passed. Scope declared incompatible;895 native remap and272 host checks retained. Final49/49 native,37 runtime reports/5696 checks,tools42/42.0 prepared batches,7 packages open,total future count unknown. Next seam/singular native applicability. No full/integration/push claim. [Evidence](../../docs/validation/2026-10-08-uv-reference-rhino5.md).

BRep seam/pole checkpoint2026-10-08:2/2 native(211),2/2 FreeCAD(65) passed including copy/edit/UndoRedo/source-deletedFCStd. Cylinder negative-proxy cap synchronization fixed with bounded complete-basis checks. Final50/50 native,38 reports/5761 checks.1 prepared4-file Rhino5 batch(2sources+2edited exports),7 packages open,total future batch count unknown;actual Rhino pending,full/integration/push incomplete. [Evidence](../../docs/validation/2026-10-08-seam-trim-preparation.md).

BRep seam/pole+edit actual checkpoint2026-10-08:RhinoGUI4/4 passed,0 bad objects;native decoded topology/commonUVbasis1800 +actual FreeCAD reimport45 pass. Rhino knot insertion verified by complete common basis,maxCVdifference4.44e-16;rawC2 bytes not identical. Final51native/39reports/5806checks,tools42/42.0prepared batches,7packages open,total future count unknown. Full/integration/push incomplete. [Evidence](../../docs/validation/2026-10-08-seam-trim-complete.md).


User-facing 3DM menu checkpoint2026-10-08: direct Import/Export actions and builtin-icon 3DM toolbar; same IDs/handlers, Rhino5 source-preserving defaults. Actual FreeCAD 18 checks and Rust 75 tests pass on isolated preview binary. No full geometry regression or primary integration claim. [Evidence](../validation/2026-10-08-3dm-exchange-menu.md).


Native Qt activation crash fixed2026-10-08: GIL guard covers Python import/call/error/cleanup in both exchange commands. User oval31 source records →52 native-menu host geometries valid,8 native-action checks and18 UI regression checks pass. Prior Python-only command invocation missed GIL requirement. Actual Rhino new-ring export not tested. [Evidence](../validation/2026-10-08-native-menu-gil-fix.md).


Oval display diagnosis2026-10-08: native basis samples match OCCT within1.12e-14mm; OM9 polygon wireframe exposes tessellation diagonals. Same shape hashes yield smooth Shaded/native CAD wireframe. Per-view CAD wireframe adapter remains open; not a full geometry/Rhino roundtrip claim. [Evidence](../validation/2026-10-08-oval-display-diagnostic.md).

Later CAD Wireframe checkpoint2026-10-08 supersedes the display limitation
above: native edges +trimmed isocurves, source RGB/density, independent modes,
external/nested reflected links.8 actual FreeCAD reports/105 checks passed on
the isolated final binary; user ring source/shape unchanged. Pixel parity and
all affine-member density properties remain unverified; full exchange is not
claimed. [Evidence](../validation/2026-10-08-cad-wireframe.md).

Working modeling Phase1 actual application acceptance2026-10-09:10 completed Rhino5 cases; real ring369 checks/29 roots+1 member;4 actual FreeCAD saved-output reports230 checks. Edit/add/current Selected V5 and source-independent FCStd verified. Phase2–5/full exchange/primary integration remain open. Evidence: ../validation/modeling-phase-1/application-evidence.json.
