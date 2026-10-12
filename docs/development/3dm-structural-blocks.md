# Structural 3dm blocks in the development module

The isolated development runtime (`H:/FreeCAD-src/build/3dm-preservation-sdk`) exposes native block creation through `ThreeDm`. This keeps concurrent public module builds from overwriting the tested binary/scripts. The public primary checkout has not yet integrated this package. Standard File Export, menu/CMD Export3dm and named/native export now share preservation routing for supported source/block selections. Use the explicit `export_preserved` API when deliberately requesting preservation of a selection without mode inference.

```python
import FreeCAD as App, Part, ThreeDm
doc = App.ActiveDocument
member = doc.addObject('Part::Feature', 'NewBlockMember')
member.Shape = Part.makeBox(2, 3, 4)
definition = ThreeDm.create_definition(doc, [member], 'New jewelry block')
instance = ThreeDm.create_instance(
    definition,
    App.Placement(App.Vector(10, 20, 30), App.Rotation()),
    'Placed jewelry block',
)
doc.recompute()
ThreeDm.export_preserved('H:/exports/jewelry.3dm', [instance])
```

The destination folder must already exist. The writer emits Rhino5 version5/50, rereads the native graph/payloads and replaces the destination only after validation.

`create_definition` accepts new Part/Mesh objects, instances created by `create_instance`, and verified source-backed geometry or instance objects. It moves those members into a hidden reusable App::Part while retaining their current world placement relative to the initially unplaced definition. Creation validates supported inputs before its single Undo transaction. Source UUIDs/records and the included archive stay immutable provenance; promoted members receive separate persistent creation IDs for their native definition-member records.

Verified unchanged source geometry can retain its original native payload with a member transform; edited supported CAD/mesh stages current local geometry. Retained geometry without FreeCAD `Placement` uses an explicit editable `OM9BlockMemberPlacement`. TextDot payload/attributes, names and transforms have acceptance evidence. This does not establish support for arbitrary retained classes or plugin data: native compatibility/reference/version checks still gate those records. Missing legacy baselines require explicit migration. One new definition cannot silently combine different archive namespaces.

Verified shared source-member proxies may also be passed to `create_definition`. The proxy moves into the new definition with its current world placement; its canonical source object stays in place and keeps its source identity. Each promoted proxy receives a separate persistent member UUID. Export combines the current proxy placement/scale/LinkTransform with the current canonical payload once, applying canonical metadata and explicit proxy overrides. Unchanged geometry copies directly from native data; a two-dimensional rational NURBS curve fixture retains its dimension, knots, weights and native payload checksum, including after FCStd. Edited supported CAD/mesh and current instance targets use independent overlays. A missing proxy metadata baseline, invalid canonical CAD/mesh, foreign namespace, chained proxy target or singular/nonfinite transform rejects before creation/copy mutation or destination replacement.

Selected original top-level retained geometry now uses the native transform overlay, including after promotion to a new definition. Supported existing native definition members and physical shared proxies selected alone now export with an independent stable top-level UUID, current payload and physical world Placement. Exporting its placed block applies current retained member deltas in definition-local coordinates. CAD source selections and their promoted member representations can coexist in one request without applying the world transform twice.

`create_instance` creates an App::Link with `LinkTransform=True`. It can target a new definition or a verified imported definition in the active document. Placement, LinkTransform and ScaleVector determine its current native instance matrix; source geometry is not flattened. Nested new instances can be grouped into another new definition. Imported target dependencies follow current supported host edits and stay backed by the included immutable archive.

New definition/instance/member UUIDs persist through FCStd and repeated export. A copied new host receives a deterministic separate UUID scoped to its original creation UUID and current host name, without changing provenance during export. Rename of a copied host's internal FreeCAD Name and manually added members need further explicit identity fixtures; Label changes do not rename the internal Name.

`ThreeDm.copy_definition(definition, name=None, copy_targets=True)` explicitly copies a supported new/imported definition family in one Undo transaction. It allocates fresh persistent definition/member/reference identities and copies each shared nested target once. `copy_targets=False` copies member references while keeping their current target definitions shared. It retains local placements, supported current geometry edits, native source provenance and the verified source archive. Imported static definition copies retain native description, URL, URL tag and safe user text via `source_definition_uuid` in the native new-definition protocol. Copies of copied families retain that original native provenance. Linked definitions and unresolved/opaque identity-bearing data require their resource/reference implementation before safe export.

Family copying supports verified source proxies in new and imported definitions. The copied proxies keep separate persistent member UUIDs and reuse one copied canonical payload for each original canonical object across the copied family. Editing this copied canonical object updates its copied shared uses while leaving the original geometry unchanged. Deep copying retargets canonical instance references to copied target definitions, while retaining each proxy's placement and scale. Mixed CAD/mesh, nested references, FCStd and Undo/Redo have acceptance evidence. This does not establish arbitrary retained/plugin payload, App::Link array/subelement or legacy proxy migration coverage.

Native copied definition metadata and copied geometry/attributes share an overflow-safe 512 MiB serialized-copy-data quota per source namespace. Large requested definition copies are checked before cloning; later checks include current copied member lists and metadata. This quota does not describe total process RSS or a global multi-source memory limit.

## Explicit legacy archive migration

`ThreeDm.migrate_archive(owner, rebuild_previews=False, origins=None, targets=None, fork_namespace=False, upgrades=None)` restores supported legacy owner/source/proxy baselines from the owner's verified included archive, in one Undo transaction. `owner` is the `RhinoSourceArchive` object in the active editable document. Finish any open user transaction first. The function verifies the archive hash/schema/units/namespace, native IDs, existing native record fields and property types. A hidden temporary reference document supplies original converted geometry, native instance matrices, metadata and definition/proxy baselines. It closes on success or error, and the original active document is restored. The public import still uses its native project/task guard and native commit transaction; only read-only preparation is shared with the migration reference.

Missing owner schema/hash and an owner namespace with unambiguous existing source links can be restored. Source ownership, record, metadata, placement, host identity, native class/capability and signature fields are restored without replacing current canonical CAD/mesh or relinking current instance targets. Edited geometry remains changed relative to the reconstructed native baseline. Current metadata/placements/membership stay edits. Unchanged NURBS data retains the original native payload rather than taking a host conversion round trip. Definition and proxy baselines come from the original native member graph, not the edited current graph.

The default verifies an affine cache against independently reconstructed native display payloads before initializing its display baseline. It then refreshes verified caches from current canonical members. A changed/unverified cache requires the explicit `rebuild_previews=True` option. That option replaces generated display data with known native reference data, then rebuilds from the verified current graph; canonical geometry and the root's rigid placement delta stay intact. Failure rolls back restored properties, generated preview objects and cache replacement. Rebuild is an explicit discard of derived preview edits, not of CAD/mesh edits. Original cache objects/properties return through Undo; Redo and FCStd restoration retain the new baselines.

For duplicated legacy source/definition UUIDs, pass `origins={native_uuid: original_host.Name}`. Missing or contradictory origin mappings refuse before mutation. Matching original host witnesses allow later unambiguous migrations without another mapping. Copies receive fresh persistent definition/member creation IDs while keeping original native provenance; physical App::Link targets and current geometry remain intact. Verified shared proxy copies keep their canonical payload and reconstruct original membership metadata. Existing creation IDs survive repeated migration. Foreign/conflicting ownership, mismatched native records, unsupported legacy representations and missing membership provenance still refuse. Affine serialized targets are not inferred to point at copied definitions. Pass `targets={affine_reference.Name: intended_definition.Name}` to explicitly assign original, migrated-copy or existing new targets in the same namespace. Physical App::Link references keep their object pointers and refuse this option. A missing/wrong/foreign target, ambiguous/colliding new UUID or failed mapped preview rejects with full rollback. Native matrices, source records, geometry edits and placements remain intact. A changed/unverified cache still requires `rebuild_previews=True`. A supported recursive copy that also duplicates its archive owner can explicitly use `fork_namespace=True`. The fork retains native provenance and current owned geometry/links while persisting a separate namespace, origin scope and host witness. Repeating it on the same host is idempotent; copying that fork gets another scope. Different owners sharing one namespace refuse export. Foreign physical targets/source members and failed preview rebuilding refuse with rollback. Actual recursive shared point, mixed CAD/mesh shear/reflection and rational NURBS families have additional evidence, including restoring missing native proxy metadata baselines and preserving shared canonical payloads. Arbitrary retained/plugin proxy graphs, missing membership provenance and arbitrary historical FCStd representations remain pending; the supported explicit host-upgrade slice is described below.

FreeCAD's recursive `document.copyObject(new_definition, True)` also has evidence for wholly new nested CAD/mesh families. Use `copy_definition` for imported families: ordinary copied imported objects retain source identity fields and cannot be silently assumed to have a separate native definition identity. Mixed affine display caches are copied as independent owned children and checked against the verified original; subsequent manual cache edits remain protected. Manually inserting members and renaming a copied host's internal Name still require broader identity fixtures.

Select placed instances for structural export. Definition containers are not implicit whole-project selections. Missing targets, foreign archive namespaces, singular/projective matrices, cycles, unsupported members, duplicate identities and unsupported fields fail. Native block depth64 is supported;65 is rejected. Empty embedded definitions remain real native blocks.

All FreeCAD geometry and staged new instance matrices use millimeters. Imported geometry normalizes to millimeters once before the native new-instance matrix is applied. Definition placement is composed only when LinkTransform is enabled.

Instances created by `create_instance` may also be added to an imported definition using `imported_definition.addObject(instance)` followed by document recompute. Select its placed source root for export. The new reference remains a native definition member; copied new references get separate stable IDs. Its target may be a new definition or a verified definition in the same archive namespace. Imported affine previews traverse these new definitions and follow subsequent member, placement, scale and LinkTransform edits. A new instance can retarget between supported new/imported definitions; an existing bound archive namespace stays verified even when its current target has no snapshot ownership. Foreign namespace targets, cyclic links, edited previews and selecting one new reference as both a top-level root and a member reject.

Existing imported rigid references may also target an explicit new definition using `source_instance.setLink(new_definition)`. Imported affine references expose `OM9DefinitionTargetUUID`; for an original newly created definition, set this to `new_definition.OM9NewDefinitionUUID`, then recompute. Original source UUID, snapshot and native instance matrix remain provenance. Export and display follow the current target graph, including nested new/imported targets. A definition with archive ownership must match the reference's namespace; an unowned new definition can be staged separately for two source namespaces. Native source UUIDs resolve inside their verified owning namespace; a new target identity cannot collide with that namespace's records. The unsourced original and its reimported native counterpart can coexist in the document without becoming duplicate source mappings.

Promoted source instance members follow current new/imported targets and native matrices. Copied promoted CAD/mesh/instance members receive separate stable creation-scoped output UUIDs. Their dependencies remain verified current shared dependencies; preserving original geometry via `follow_canonical=false` is not a whole-graph snapshot guarantee. Empty new definitions keep their verified owning namespace even after their source members are removed.

Current limits: arbitrary retained-class creation/display/edit coverage, linked definition copying, broader copied ownership migration and arbitrary legacy representation upgrades, different-source document merge policy, full historical/retained graph support and final public integration remain pending. Ordinary recursively copied imported definitions require explicit identity/provenance migration; use `copy_definition` for a separate native family. Manually inserted new geometry without creation identity requires wider identity/observer fixtures. Native protocol coverage is wider than those host routes; this distinction is recorded in the validation reports. Actual Rhino5 application acceptance remains untested.

Evidence: [new block creation and graph validation](../validation/2026-10-06-3dm-new-structural-blocks.md).

Further evidence: [new instances in imported definitions](../validation/2026-10-06-3dm-new-member-instances.md).

Further evidence: [existing source references targeting new definitions](../validation/2026-10-06-3dm-source-new-targets.md).

Further evidence: [source geometry and instance members in new definitions](../validation/2026-10-06-3dm-source-members.md).

Further evidence: [copying new and imported native definition families](../validation/2026-10-06-3dm-definition-copies.md).

Further evidence: [combined copied native data budget](../validation/2026-10-06-3dm-copy-budget.md).

Further evidence: [source proxy creation and family copying](../validation/2026-10-06-3dm-source-proxy-creation.md).

Further evidence: [explicit legacy archive migration](../validation/2026-10-07-3dm-legacy-migration.md).

Further evidence: [explicit copied-origin migration](../validation/2026-10-07-3dm-copy-origin-migration.md).

Further evidence: [explicit affine target migration](../validation/2026-10-07-3dm-affine-target-migration.md).

Further evidence: [recursive archive namespace migration](../validation/2026-10-07-3dm-recursive-namespace-migration.md).

Ordinary source proxy/member copies now derive stable output UUIDs from namespace, native source UUID and member host internal Name. Separate proxies of an independent canonical copy keep separate identities. Export does not mutate source hosts. This stability is verified across repeated export and FCStd; host internal Name changes and source/copy representation changes remain outside that guarantee.

Further evidence: [recursive shared CAD/mesh and native NURBS migration](../validation/2026-10-07-3dm-recursive-shared-migration.md).

## Shared export entry points

Default `ThreeDm.export_file(path, objects)` preserves supported source/block selections and uses geometry export for ordinary unsourced CAD/mesh. Registered File Export, named/native export and menu/CMD use this route. Plain layer groups deduplicate selected roots; archive/definition containers do not implicitly select all source records. Unsupported identities, dependencies and representations keep precise writer guards.

Explicit `geometry_only=True` is available on export_file/export_selection/export_named/export. The menu/CMD save dialog provides an unchecked Geometry only option that discloses omissions. Source/new placed blocks flatten from a staged verified current native graph, preserving mesh type and affine coordinates. Retained records, stale/manual display caches and unsupported graphs refuse atomically. Standard File Export defaults to preservation; use the OM9 dialog or API for explicit omission.

Evidence: [shared export routes](../validation/2026-10-07-3dm-shared-export-routes.md).

## Explicit legacy host representation upgrades

`ThreeDm.migrate_archive(owner, rebuild_previews=True, upgrades={legacy.Name: {'type': 'App::Link', 'placement_mode': 'delta'}})` replaces a verified legacy flattened holder with its native host while keeping its exact internal Name, source UUID/signatures/matrix and supported incoming graph. Use `placement_mode='instance'` only when the old Placement already represents the complete physical instance matrix. A whole affine Placement must differ from native A by a proper rigid delta; scale/shear cannot be guessed or discarded. Optional physical `target=definition.Name` explicitly chooses a verified current original/copied/new definition. Repeated upgraded-host policies are idempotent; conflicting targets refuse.

Geometry policies use `{'type': 'Part::Feature'}` or `{'type': 'Mesh::Feature'}` and require valid current payload of that kind. Current geometry remains current; native source bytes restore original provenance only. No empty-payload restore or CAD tessellation is inferred. Native mixed App::Part and CAD affine previews rebuild from current canonical members under explicit `rebuild_previews=True`.

Supported dynamic values, property groups/documentation/editor flags, label/color/visibility, ordered groups, whole App::Link and custom Link/LinkList references survive. Expressions, array/subelement/external links, scripted providers and unknown payload types require dedicated adapters and refuse. One Undo/Redo and any late failure include physical host replacement, incoming rewiring, previews/Origin helpers and baseline fields. Python callers must reacquire changed objects with `document.getObject(name)` after replacement or Undo/Redo; old wrappers refer to removed hosts.

Evidence and exact limitations: [legacy host upgrades](../validation/2026-10-07-3dm-host-upgrades.md). These synthesized layouts do not establish arbitrary historical FCStd support.

## Retained native placement

Imported retained non-instance objects without a standard FreeCAD Placement expose `OM9BlockMemberPlacement` as an editable App::PropertyPlacement. Set it to an explicit translation/rotation delta; selected top-level export composes current parent world Placement, while block-member export uses the local delta and leaves the root native matrix separate. Source geometry remains authoritative and source signatures are not reset by movement. Payload/property tampering refuses rather than exporting the old snapshot over an edit. Invalid retained placement types or an ambiguous standard Placement refuse. Only TextDot/PointCloud native behavior has the focused evidence in this package; generic transform dispatch does not enable unknown classes or references.

PointCloud native transforms use inverse-transpose normal directions while retaining original magnitudes and colors. Copies and mm normalization use the same native helper. Member output transforms follow the copy's own local matrix, and native graph bounds refresh. Native semantic inventory now includes TextDot point/text and PointCloud point/normal/color counts plus point bounds; full serialized payload digests still protect fields beyond those summary facts.

Evidence and format-specific limits: [retained transforms](../validation/2026-10-07-3dm-retained-transforms.md). Display/text/per-point editing, arbitrary retained selected-member graphs, nonempty secondary-text downgrade and complete retained compatibility remain pending.

## Selecting a native definition member alone

Use the ordinary whole-object export entrypoint for a verified native member or its shared physical proxy. The writer creates an independent top-level native copy with a stable scoped UUID; the document's original member, source signature and definition membership do not change. Canonical definition placement and proxy/current native deltas compose in the selected host's physical world frame. Selecting the placed block instead preserves its native reference graph; selecting both keeps both representations. This API does not infer which of several block-instance paths should supply a shared member's coordinates.

Native copy protocol role defaults to definition-member; a selected independent copy explicitly uses top-level and must be reachable as a root. Changed supported CAD/mesh stages current local payload; unchanged native/rational geometry and retained payload use the original native record. New explicit members have a separate stable selected-root UUID. Nested source references use their current target closure, including current new definitions. Out-of-plane2D native curves promote dimensions before affine transformation, retaining exact rational weights and knots. Arrays/subelements and unsafe opaque references still refuse.

Evidence: [selected members](../validation/2026-10-07-3dm-selected-members.md). Actual menu/CMD/standard export, supported class interactions, mm/cm, source immutability and Undo/Redo/FCStd are verified; complete class/field/display/edit/resource coverage remains pending.


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
