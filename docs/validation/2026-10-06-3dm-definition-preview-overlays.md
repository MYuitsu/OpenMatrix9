# Definition membership and automatic affine preview refresh

Implemented and tested in the private development checkout. Full openNURBS and public-source integration remain incomplete. Only the explicit preservation writer API uses these export overlays.

## Behavior and defects resolved

- `definition_overlays` changes a reachable native definition's name and ordered member UUID list. Members must be source-owned definition geometry in the same namespace. Native graph closure follows the edited membership, so removed members are not resurrected; existing source members can be added from another definition. Duplicate members, unreachable overlays, name conflicts, unsupported fields and cycles reject before destination replacement. Definitions and instances retain their source UUIDs.
- Empty embedded definitions preserve native instances and reimport as structural links/containers. External-only definitions remain an explicit unsupported dependency. The pinned `ON_InstanceDefinition::Internal_InstanceGeometryIdIndex` returns false (integer0) for an empty list; its `IsInstanceGeometryId` wrapper consequently reports true for arbitrary UUIDs. Archive traversal now checks the actual member UUID list, preventing empty definitions from hiding all top-level placements.
- Canonical member geometry/membership changes mark the archive family dirty. A document observer regenerates affine previews after recompute, composing original native matrices with current nested member placements. Root placement remains intact. BRep/mesh previews use current host canonical geometry; these derived shapes are never the native export source. A mesh added to an existing BRep compound preview is represented by faceted display geometry only.
- Preview content integrity is checked before regeneration. Manually modified preview geometry is not overwritten, and preservation export remains an atomic rejection for it. Failures leave a precise `OM9PreviewStatus`. Empty affine instances retain a stable App::Part container, allowing later source membership additions to acquire display geometry.
- Workbench initialization installs the observer, so opening FCStd in a fresh FreeCAD process does not require another import to activate refresh.
- Archive ownership uses `App::PropertyLinkHidden`, separating provenance from FreeCAD's local geometry dependency traversal. A normal PropertyLink caused `App::Part.addObject/removeObject` to pull archive storage, siblings and instances into/out of definitions. Hidden ownership fixes actual add/remove behavior while retaining owner identity, copied objects, included snapshots and FCStd persistence. Previously saved documents with normal owner links do not automatically migrate their property types in this slice.
- Native import rollback saves/restores the original Python exception around `abortTransaction`. Observer callbacks previously cleared that exception, turning an intentional binding failure into a SystemError. The regression now checks the exact injected RuntimeError and document rollback.

## Evidence

Native CTest8/8, exit0,29.58s after the definition/empty-block changes; includes both immutable ring geometry fixtures. SDK module/scripts build exit0 after the rollback bridge fix. Native fixtures cover rename, member removal/addition/order, cycle and duplicate atomic rejection, empty definitions, bounding boxes and immutable source snapshots.

| FreeCAD suite | Result | Artifact |
|---|---|---|
| Definition overlays |8/8, process0|`build/three_dm_definition_overlay_smoke-1/29541539a42d4cf9b57974b70e6f857a/results.json`|
| Automatic preview |10/10, process0|`build/three_dm_affine_refresh_smoke-1/50a00675434f4c21b935357a50861fef/results.json`|
| Fresh-process FCStd reopen |2/2, process0|`build/three_dm_affine_refresh_reopen_smoke-1/6ef7eef25b5347ab8f8edf2f96a59d50/results.json`|
| Member overlays |12/12, process0|`build/three_dm_member_overlay_smoke-1/b45bfe4a784d427ab899754e5e0014e7/results.json`|
| Structural blocks |23/23, process0|`build/three_dm_block_structure_smoke-1/43b6e4f669374e1caa5909523f0007d3/results.json`|
| Layer overlays |7/7, process0|`build/three_dm_layer_overlay_smoke-1/aaea9e515cb640b88204a748f990d3f9/results.json`|
| Source signature |14/14, process0|`build/three_dm_source_signature_smoke-1/1b1d82d0aa5745568037be7157dffd86/results.json`|
| Preserved export |13/13, process0|`build/three_dm_preserved_export_smoke-1/c1bedc15c7fd41e283f852ca532d1b31/results.json`|
| Lifecycle and rollback |19/19, process0|`build/three_dm_preservation_smoke-1/13af8200d1214231a167cfb09302d22c/results.json`|

The new semantic tests failed before their implementation: ignored definition overlays, stale preview volume, missing empty affine container, and lost rollback exception. A test saving FCStd directly in the shared Windows Temp root encountered an unrelated inaccessible socket; the generated reopen fixture now uses its own directory under the private build tree.

## Remaining scope

Definition-container placement, transformed shared-member proxies, unsourced geometry/new members, distinct-archive document merging and older owner-link migration remain pending. Preview appearance beyond existing display metadata and classes without canonical display geometry still require their own coverage. Standard export/menu/CMD remains guarded. Arbitrary plugin data, appearance/resources/history, per-class Rhino5 compatibility, actual Rhino5 application acceptance, final review and public-source integration remain incomplete. Passing the ring geometry fixtures does not prove preservation export of their opaque plugin tables.
