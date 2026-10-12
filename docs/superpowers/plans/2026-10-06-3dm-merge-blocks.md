# 3DM preservation writer and structural blocks Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans inline, preserving the previously selected execution method. Steps use checkbox syntax for tracking.

**Goal:** Export selected source-backed objects as Rhino5 while preserving understood native records, structural blocks and dependencies, applying supported FreeCAD edits and rejecting unsafe omissions/remaps.

**Architecture:** Rust computes dependency closure and validates identity/edit policy. Native openNURBS rebuilds a destination model from source snapshots and explicit overlays, remapping understood references and checking target-version rereads. Python binds definitions/instances and stages host geometry; native CAD remains authoritative. Unknown reference-bearing payloads are gated, never silently carried through an unsafe merge.

**Tech Stack:** Existing dependency-free Rust2024, C++23, pinned openNURBS, OCC8, Qt6 and FreeCAD included-file properties.

**Spec:** `docs/superpowers/specs/2026-10-06-full-opennurbs-design.md`. Foundation evidence: `docs/validation/2026-10-06-3dm-preservation-foundation.md`.

## Global Constraints

- Work in existing `H:/FreeCAD-src/build/om9-dev`; preserve all uncommitted baseline changes. No push/publish. Keep task checkpoints/backup evidence instead of mixing the uncommitted baseline into new commits.
- Pinned openNURBS `eb92af3ba1806b0a34a99aba0d3bda83e3d46083`; target archive version5/50.
- Input512MiB; manifest32MiB; objects1million; block depth64. Prefix shell commands with `rtk`.
- Selected export includes required transitive dependencies, never unrelated top-level objects. Source containers are metadata, not implicit whole-project selections.
- Unitless/custom sources require positive finite explicit scale. Normalize geometry to millimeters once; block matrix conversion is `S*M*inverse(S)` when member geometry is normalized by `S`.
- Preserve active-project/task guards, one import Undo, cancellation, successful-only command history, preflight before mutation and staged destination replacement.
- Keep both user fixtures immutable. Bounds1e-3mm; area/volume max(1e-3,1e-4relative), adaptive finite integration errors.
- Do not remove legacy guard globally. Route to the new writer only after its particular safe path has evidence; unsafe classes/resources/history report precise rejection.
- Actual Rhino5 application remains untested unless available. Remaining appearance, annotation, history and full coverage packages stay unverified.

## Review Focus

- Two imports share UUIDs: output identity remapping must update every understood reference; opaque remaps fail (Tasks1/3).
- Selecting one placed block excludes sibling placements and unrelated source geometry while retaining its full nested definition graph (Tasks1/2).
- Source inches/custom units with nested reflected/nonuniform blocks must not be scaled twice (Tasks2/4).
- Editing, deleting or copying a source-backed object must not resurrect its stale source geometry or collapse duplicate identities (Tasks3/4).
- Rhino5 writer can silently drop newer classes/payloads: target reread must prove semantic retention before replacing destination (Tasks2/3).

## Shared interfaces

Export request schema1: `{schema_version:1,sources:[{namespace,snapshot,archive_sha256,scale_mm}],selected:[{host_id,namespace,source_uuid,action,signature,metadata,brep?,vertices?,faces?,instance_matrix?}]}`. `action` is `unchanged`, `replace`, `instance`, or `duplicate`. Geometry-only unsourced objects use a separate `new_geometry` array. Deletion is represented by absence from selected/current host mappings, never a scan of every source top-level record.

`metadata` includes name/layer path/color/visible/locked and explicit changed fields. The native writer preserves unrelated source attributes when updating those fields. Retained unsupported records cannot masquerade as editable replacements.

Rust graph indices refer to `(namespace,source_uuid)` records, never list positions of FreeCAD objects. `dependency_closure(node_count, offsets:&[usize], edges:&[usize], selected:&[usize])->Result<Vec<usize>,ClosureError>` returns sorted unique reachable indices; malformed CSR, missing references and cycles fail. Limit graph nodes1million and edges16million; block expansion depth64 is checked separately in native block traversal because dependency graphs include extra definition/member/component nodes. C ABI `om9_3dm_dependency_closure(node_count,offsets,edges,edge_count,selected,selected_count,output,capacity)->isize`: nonnegative required/output count; -1 malformed input, -2 cycle, -3 graph limit, -4 capacity. Zero-capacity call queries count; caller bounds input before passing arrays.

Native `writePreservedArchive(const QJsonObject& request,const std::filesystem::path& destination)` lives in `Gui/ThreeDmMerge.h/.cpp`. Python native `writePreserved3dm(request_json,path)->None` uses it. Schema/resource/compatibility errors identify namespace, UUID and class.

Host `source_signature(obj)->str` hashes actual native BRep/mesh or instance matrix and relevant metadata, excluding temporary paths/labels with retention suffixes. Persist baseline `OM9SourceSignature`, `OM9SourceMetadata`, `OM9SourcePlacement`, `OM9SourceRecord` and `OM9ArchiveOwner` link. Existing foundation objects migrate within an explicit transaction; never invent a baseline from an already edited shape. If baseline is absent, stage current supported CAD as replacement; retained ambiguity fails.

## Task1: Rust closure and overlay policy

**Files:** `rust/src/core_3dm_archive.rs`, `rust/tests/core_3dm_archive.rs`, `Gui/RustBridge.h`.

- [ ] Add failing tests for shared dependencies/deduplication, selected root exclusion, malformed offsets/out-of-range indices, cycles, graph limits, duplicate source UUID under different namespaces and all C ABI result/error codes.
- [ ] Run `rtk cargo test --manifest-path rust/Cargo.toml --test core_3dm_archive`; verify the new interfaces fail.
- [ ] Implement CSR closure with bounded iterative DFS and explicit cycle checks. Pure policy rejects unsupported retained replacements and ambiguous duplicate source mappings; valid duplication receives a fresh output identity.
- [ ] Run focused Rust tests, fmt and full suite; verify C++ declarations match exports. Record evidence/checkpoint.

## Task2: Native single-source selection and structural archive write

**Files:** create `Gui/ThreeDmMerge.h/.cpp`, `tests/native/three_dm_merge.cpp`; modify `cmake/OpenNURBS.cmake`, native CMake and inventory dependencies. Native test CMake adds an imported static Rust core target with a Cargo release custom command, manifest `rust/Cargo.toml`, isolated target directory under the native build and platform-specific static library name; link merge tests to it without including FreeCAD Gui targets.

- [ ] Create independent source fixture: two top-level placements sharing a nested definition, BRep/point members, unrelated top-level CAD, TextDot, layer/group/material/style dependencies. Add missing/cyclic,64/65 nested block depth and nonmillimeter fixtures.
- [ ] Write failing tests: selecting one block yields one top-level placement with real definition/reference graph; no flattened duplicate originals; selecting TextDot preserves content/attributes; unrelated objects excluded; destination sentinel survives failures.
- [ ] Implement bounded source hash verification, native graph extraction and Rust closure invocation. Inventory the dependencies consumed by this writer, including block members, parent layers, object/layer materials, groups, dimension styles, hatch patterns and mappings. A dependency family not understood by this writer is a compatibility error, not an empty list.
- [ ] Populate an output ONX_Model from selected records/dependencies using native component-reference maps and `UpdateReferencedComponents` where supported. Normalize understood geometry/instance transforms once; preserve source attributes and native userdata only where safe references can be established. Keep history/unknown plugin dependencies gated.
- [ ] Write version5 to the caller's staged destination; reread and compare UUID/class/definition-member graph, applicable attributes/style/content, supported payload retention and native geometry metrics. Reject newer-only records that disappear/change incompatibly, including extra/missing graph nodes.
- [ ] Run `ThreeDmMergeTests` through the existing native helper, then all native suites. Record matrix/closure/version evidence.

## Task3: Edits, duplication and collision-safe multi-source merge

**Files:** `Gui/ThreeDmMerge.cpp`, `Gui/ThreeDmPython.cpp`, `rust/src/core_3dm_archive.rs`, `tests/native/three_dm_merge.cpp`.

- [ ] Add failing native tests for replaced BRep/mesh, name/color/layer/visibility changes, removed selections, copied objects, mixed new/source-backed CAD, two sources sharing UUIDs, opaque collision refusal and unknown dependency refusal.
- [ ] Implement native Python bridge and request schema validation before writes. Read staged replacements through existing converters; require valid CAD and finite placements. Unchanged records retain native source geometry; changed supported records replace it. No source record overrides an explicit host edit.
- [ ] Allocate output UUIDs by scoped identity and explicit duplicate host identity. Rebuild reference maps for every understood selected dependency; never rely on automatic duplicate-name/UUID conflict resolution. Shared unchanged dependencies remain reusable inside each source namespace. Unknown reference-bearing payloads refuse remaps.
- [ ] Apply metadata overlays without replacing all object attributes. Explicit omissions do not resurrect deleted source records. Reject unsupported retained edits and stale/ambiguous owner links.
- [ ] Run focused merge suite and all Rust/native suites. Snapshot hashes remain unchanged; failed writes preserve destination and staged cleanup. Record exact rejection/retention scope.

## Task4: Host structural blocks, persistence and shared export route

**Files:** `ThreeDmArchiveState.py`, `ThreeDm.py`, `Gui/ThreeDmPython.cpp`, `Gui/CoreThreeDm.cpp`, new `tests/three_dm_merge_smoke.FCMacro`.

- [ ] Write failing host tests for reusable hidden definition containers, repeated/nested placed instances, safe selection closure, rigid edits, native nonuniform/reflected matrix retention, copy/delete/rename, same UUID across two imports and FCStd reopen without source/staging.
- [ ] Extend native preparation additively with `definitions` and `instances`; keep manifest schema1 and support existing foundation documents. Definition members have source identities; instance records keep the exact normalized native4x4 matrix.
- [ ] Bind representable rigid/uniform instances as `App::Link` to reusable definition containers. For general affine transforms use explicit source-backed instance objects with native CAD preview, `OM9InstanceMatrix` and editable rigid placement delta. Preview is display data; export uses `delta*OM9InstanceMatrix`, never the baked preview as the source definition. Reject singular/non-affine matrices and unsupported definition edits.
- [ ] Persist signatures/owner links and source records. Build export requests only from selected live objects and their dependencies. New or copied supported CAD receives explicit new/duplicate identity. Old foundation objects lacking signatures use supported replacement or precise retained ambiguity errors.
- [x] Route standard export, menu and CMD through the same preservation writer for safe selections; leave explicit geometry-only route/disclosure available. Do not select entire archives by default. Preflight before destination mutation and successful-only history remain enforced.
- [ ] Verify one Undo/Redo, binding rollback, task/edit/cancel guards, selected instance graph, supported edits, duplication and FCStd reopen with source deleted.

### Task4 verified migration evidence — 2026-10-07

- [x] Explicit source/definition origin mappings allocate independent copied identities while retaining native provenance and edited geometry.
- [x] Explicit affine target mappings resolve original/new/migrated-copy definitions, retain native matrices and current CAD/mesh, and survive Undo/Redo/FCStd.
- [x] Invalid/foreign/ambiguous identities reject; cyclic mapped preview rebuilding restores fields, placements, payload fingerprints and original preview objects.
- [x] Supported nested whole-group archive-owner copies get independent persistent namespaces, retaining native graph provenance and edits; repeated migration, copy-of-fork, Undo/Redo/FCStd and full rollback verified.
- [x] Actual recursive shared point, mixed CAD/mesh shear/reflection and rational NURBS families retain canonical sharing, independent geometry and stable member identities through repeated export and FCStd.
- [ ] Arbitrary retained/plugin graph layouts, real historical representation upgrades and broader arrays/subelements remain pending.

Evidence: `docs/validation/2026-10-07-3dm-affine-target-migration.md`; fresh isolated FreeCAD515/515 across30 suites and native8/8. These checks establish the listed migration layouts, not completion of the full comprehensive objective. Task5 and public integration remain pending.

Latest Task4 evidence: `docs/validation/2026-10-07-3dm-recursive-namespace-migration.md`; fresh isolated FreeCAD540/540 across31 suites and native8/8.

Latest shared-family evidence: `docs/validation/2026-10-07-3dm-recursive-shared-migration.md`; fresh isolated FreeCAD564/564 across32 suites and native8/8.

Latest shared-route evidence: `docs/validation/2026-10-07-3dm-shared-export-routes.md`; real menu/CMD/native/registered export, explicit omission, guards and FCStd verified. Fresh isolated FreeCAD593/593 across33 suites and native8/8.

Latest host-upgrade evidence: `docs/validation/2026-10-07-3dm-host-upgrades.md`; supported synthesized physical/CAD/mesh/mixed affine holders, current payload/graph, exact host identity, explicit placement interpretation, Undo/Redo/FCStd and late rollback verified. Fresh isolated FreeCAD632/632 across34 suites and native8/8. Arbitrary historical/script/plugin layouts remain pending.

Latest native retained placement evidence: `docs/validation/2026-10-07-3dm-retained-transforms.md`; selected-root/canonical-member transforms, copied PointCloud inverse-transpose normals/colors, source unit normalization, native bounds, Undo/Redo/FCStd and atomic rejection verified. Fresh FreeCAD657/657 across35 suites; native9/9; Rust75/75/fmt. Member-only selection, class-wide compatibility and final integration remain pending.

Latest selected-member evidence: `docs/validation/2026-10-07-3dm-selected-members.md`; independent scoped UUIDs and roles, canonical/retained/edited CAD-mesh, rational/reflected proxies, current nested targets, mm/cm, exact closure, Undo/Redo/FCStd and actual menu/CMD/standard export verified. Native out-of-plane2D curves retain rational basis through explicit3D promotion, including geometry-only blocks. Fresh isolated FreeCAD690/690 across36 suites and native9/9. Complete class/field/resource support and final integration remain pending.

## Task5: Final review, primary build and acceptance

**Files:** validation report, support matrix, coverage/progress JSON; intentional integration of verified owned files only.

- [ ] Run Rust full suite and all native suites including merge. Build isolated SDK, wait for linker, run merge/preservation/host macros and both immutable ring geometry suites.
- [ ] Verify original block graph and world CAD against ring fixtures in preservation mode; output selected subset excludes unrelated source top-level records. Keep135/101 geometry-only objects and existing547/347 acceptance checks.
- [ ] Request one fresh reviewer of the complete package. Fix important findings with failing tests then green full affected suites; ledger deferred minor findings and rulings.
- [ ] Back up primary owned files, integrate verified changes, build/install primary SDK, wait for link, rerun acceptance macros. Record process exit, source hashes, output version, semantic closure and adaptive error estimates.
- [ ] Upgrade only test-backed coverage rows. Describe unsafe opaque/newer/resource cases precisely. Subsequent geometry/appearance/document/history packages remain pending; no full-support claim.

## Plan self-review and handoff

The package implements the writer/structural-block section of the approved comprehensive spec. Appearance, annotation editability, full document semantics and arbitrary history/plugin payload support remain separate packages; their unsupported dependencies must fail this writer safely. Task1 supplies Task2 closure; Task2/3 supply Task4 writer request; Task4 signatures/instances preserve schema and ownership consumed by Task5. Native matrices use the same normalization in preparation and write.

Previous inline execution method is retained. Plan is ready for user review before implementation, as required by writing-plans. No skill business-rule changes are included or authorized by this plan.


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
