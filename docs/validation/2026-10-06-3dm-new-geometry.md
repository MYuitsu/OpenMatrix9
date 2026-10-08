# New geometry and new block members

Implemented and tested in the private development checkout. Full openNURBS, public-source integration and actual Rhino5 application acceptance remain incomplete. These paths use the explicit `export_preserved` API; standard/menu/CMD routing remains guarded.

## Behavior

- New Part CAD and Mesh objects export alongside selected source-backed geometry. Native UUIDs are allocated for the request without adding source properties to the FreeCAD document. World placement includes the object and its parent exactly once.
- A selection containing only new CAD/mesh creates a millimeter native document and uses the same Rhino5 writer, native reread verification and atomic destination replacement.
- New CAD/mesh inside an existing block definition exports as definition-local geometry. Ordered definition membership references its allocated UUID; repeated placements share one native member. Removed source members stay omitted.
- Affine previews render current unsourced canonical members and automatically refresh after subsequent member edits. Export continues to use canonical geometry and native instance matrices.
- New member geometry/membership survives FCStd save/reopen and geometry edit Undo/Redo. Exports from two imports of the same source retain independent scoped identities alongside new geometry.
- Invalid geometry, UUID/host collisions, invalid namespace/role/fields, malformed request tables, broken source ownership and unreachable new members reject before replacing the destination. Request size remains bounded32MiB; staged CAD is bounded512MiB. Empty or unsupported new containers are not implicit whole-project selections.

## Evidence

Native RED: unsupported new geometry. Subsequent schema RED: a malformed `sources` table was accepted as an empty new document. Both now reject correctly. Native merge target builds/runs exit0; all8 native CTest suites pass, exit0,30.01s, including both immutable ring geometry fixtures. SDK module/scripts build exit0.

Host RED: `preservation_request` refused unsourced selected CAD/mesh. Final fresh FreeCAD results:

| Suite | Result | Artifact |
|---|---|---|
| New geometry/member acceptance |20/20, process0|`build/three_dm_new_geometry_smoke-1/c5b9d65ef7c84ac587f516de84ce8d3b/results.json`|
| Existing preservation export |13/13, process0|`build/three_dm_preserved_export_smoke-1/463d199bebca40c2b3b954444f7b27e6/results.json`|
| Existing member overlays |12/12, process0|`build/three_dm_member_overlay_smoke-1/931b7396119e4e02b190809ecc9ab0bb/results.json`|
| Existing definition overlays |8/8, process0|`build/three_dm_definition_overlay_smoke-1/6b26a28d4ad84e849a515dc9faf3eae1/results.json`|
| Existing affine refresh |10/10, process0|`build/three_dm_affine_refresh_smoke-1/60f8f3c3edb94b8ba61292ba7e8be08f/results.json`|
| Existing lifecycle/rollback |19/19, process0|`build/three_dm_preservation_smoke-1/bc52673772c74ef1af294829dfbd2748/results.json`|

CAD box volume24 and world bounds(105,206,307); mesh world origin(110,220,330). The same new block member appears at bounds X17/107 and Z39/309 while retaining volume24. After reopening, geometry changes produce volume60 in both instances; Undo/Redo produces60/120. Source bytes remain unchanged and new document objects acquire no archive provenance during export.

## Remaining scope

New standalone definition creation, definition-container placement, transformed shared-member proxies and older owner-link migration remain pending. Distinct source document metadata still needs an explicit merge policy. General appearance/resources/history, unknown plugin dependencies, every pinned class/target compatibility, final review and public-source integration remain incomplete. The ring geometry regression does not prove preservation export of their opaque plugin tables. The comprehensive goal remains active.
