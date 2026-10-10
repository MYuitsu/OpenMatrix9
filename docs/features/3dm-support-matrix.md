# Rhino 5 exchange support matrix

Current consolidated audit: [2026-10-07 full openNURBS gaps](../validation/2026-10-07-full-opennurbs-gap-audit.md). The128-name raw catalog includes5 commented registration rows; normalized static registration discovery is123. Historical tables/slices below are not complete current coverage. Full support remains unproven.

Current development scope and evidence: [CurveOnSurface coupled native transforms](../validation/2026-10-07-curve-on-surface-coupled-transforms.md),1479 GUI checks/native26. Current compiled SDKs include the declared Read-only repair; original sources remain unchanged. Dated sections below record earlier implementation stages; supported selections share preservation routing. Full compatibility and public integration remain pending.

This is the OpenMatrix9 geometry-exchange scope, not a claim to reproduce every Rhino feature. User fixtures are immutable. Preservation import retains readable source-backed geometry with its concrete class and capability; retained data is not falsely reported as CAD editable. Geometry-only import rejects unsupported conversion before document mutation. Export is staged and reconverted before replacing the destination.

| Data | Current behavior | Evidence / remaining work |
|---|---|---|
| Points, lines, polylines, arcs, circles, rational NURBS | Native CAD conversion | Native geometry/archive suites |
| Trimmed surfaces, holes, shells, solids, extrusions | Native CAD conversion, source 3D edges and trim loops | Native suites and actual ring fixtures; tight construction precision protects small trims |
| Triangle and quad meshes | Native mesh conversion; unchanged original quads restored on export | Fixed ring and host suite |
| Embedded block instances | Geometry-only recursively expands CAD/mesh/native PointCloud leaves with accumulated affine transforms | Native nested-block test; actual unmeshed fixture runtime validation recorded separately |
| Nested block transforms, nonuniform scale, reflection | Applied to cloned openNURBS geometry before conversion | Native nested-block test; affine bottom-row roundoff normalized only within 1e-12 |
| Definition geometry | Excluded from top-level import; instantiated only through references | Native deduplication test |
| Missing/external-only definitions, cyclic blocks, singular transforms | Explicit error; no partial import | Missing/cyclic native tests; code guards depth64 and output1million |
| Names, nested layers, colors, visibility, locks, units | Retained as geometry metadata; inherited block color/visibility/lock applied | Native and host suites |
| Original block definition/reference structure on export | Not preserved; expanded geometry exports as ordinary CAD/mesh | Approved flattening approach; structural block roundtrip is future work |
| Annotation text, dimensions, leaders, text dots | Unsupported model geometry is rejected | TextDot rejection test; conversions require separate implementation |
| PointCloud | Current double points/normals/RGBA/plane in geometry-only and preservation modes; native affine preview | Named tested slices above; nonempty intensity explicitly rejected for the independent2013 target-reader baseline |
| Hatches, clipping objects and other native geometry classes outside listed paths | Unsupported geometry conversion is rejected with class name; source-retention has separate scope | No editable support claim; add type-specific tests |
| Proprietary plugin objects / custom geometry | Rejected when no supported native geometry representation exists | Plugin-specific conversion/evidence needed |
| Materials, textures, lights, render settings, document metadata, arbitrary user strings | Not preserved by this geometry-exchange contract (lock metadata is an explicit exception) | Full appearance/document roundtrip is future work |
| Rhino parametric history and Matrix builders | Not preserved | No compatible reconstruction implemented |
| Actual Rhino 5 application interoperability | Not tested | Native archive version 5/50 is validated; check in Rhino when available |

Geometry acceptance uses topology validity after BRep serialization and FCStd reopen, object count, metadata, optimal geometric bounds, and adaptive OCC area/volume integration. FreeCAD's plain `Shape.Volume` uses non-adaptive integration and can vary with NURBS representation; it is not the precision oracle for complex imported shapes. Existing geometric tolerances in the runtime tests are unchanged.

## Preservation foundation (2026-10-06)

The preceding table describes geometry-only exchange. Preserve source data mode now stores the entire original archive in FCStd with verified hashes and scoped identities. Unsupported editable geometry, block references and lights receive explicit source-retained records. Source component tables, resources, settings and userdata remain in the immutable snapshot; this does not establish editable/rendering support. Registered Open/Import defaults to preserve mode. Legacy export rejects preserved content unless editable objects are explicitly exported as geometry only with an omission notice.

Coverage discovery and unverified classes: `docs/3dm-coverage.json`. Evidence: `docs/validation/2026-10-06-3dm-preservation-foundation.md`. Merged archive export and structural blocks remain the next package.

## Selected preservation writer in the development checkout

The explicit `ThreeDm.export_preserved` API now retains structural definitions/references, applies source geometry/instance/metadata overlays, exports independently edited copies, supports canonical BRep/mesh/nested-instance member edits and creates/reuses Unicode layer hierarchies. Native instance UUID remapping is explicit when imported namespaces collide. Native bounds and definition-object mode remain verified after member edits. FreeCAD persistence and atomic rejection tests are recorded in `docs/validation/2026-10-06-3dm-member-layer-overlays.md`.

Definition membership/name edits, ordered source-member additions/removals and empty embedded definitions now preserve native structure. Affine previews regenerate automatically after canonical member changes, including in a fresh process opening FCStd. Evidence: `docs/validation/2026-10-06-3dm-definition-preview-overlays.md`.

New CAD/mesh and new local members of existing definitions now export alongside source objects. Definition placement and LinkTransform/ScaleVector compose through native instance overlays, with local canonical signatures and separate preview integrity checks. Evidence: `docs/validation/2026-10-06-3dm-new-geometry.md` and `docs/validation/2026-10-06-3dm-definition-placement.md`.

Transformed shared-member proxies now receive fresh native UUIDs while unchanged shared branches retain canonical identities. Copies follow current canonical BRep/mesh/nested-instance edits, support reflected/nonuniform scaling and metadata overrides, and allow copy-only selection closure. Rust limits serialized copy payloads to512MiB before native allocation and rechecks after overlays. Affine mesh previews now bake canonical Placement exactly once. Retained TextDot members without host Placement import, preserve native payload/UUID and survive FCStd. TextDot display/editability is still unverified. Evidence: `docs/validation/2026-10-06-3dm-shared-member-proxies.md`.

Independent source-backed BRep/mesh/nested-instance members now stage their own current payloads, with separate output UUIDs even when provenance is shared. They survive canonical host deletion, support proxies of copies and follow FCStd/Undo/Redo. Source geometry user text and complete object attributes are retained where their reference semantics are understood. BRep previews now bake wrapper Location into the transform before resetting child Placement; world bounds agree with native output. Evidence: `docs/validation/2026-10-06-3dm-independent-member-copies.md`. This explicit CAD/mesh staging path does not claim untouched original native topology retention for independently staged copies.

This development package is not integrated into the public source checkout; standard export/menu/CMD remains guarded. New standalone definitions/reference edits, legacy owner/baseline/proxy migration, distinct-source document merging, arbitrary plugin/resources/history and actual Rhino5 application acceptance remain pending. Full openNURBS support is not complete.


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
