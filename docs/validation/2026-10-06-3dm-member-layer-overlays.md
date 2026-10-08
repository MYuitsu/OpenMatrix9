# Definition members and layer overlays

This slice extends the explicit `ThreeDm.export_preserved` API in the private development checkout. It does not establish full openNURBS support or integration into the public source checkout.

## Implemented behavior

- Request `dependency_overlays` contains canonical definition-member edits. The native writer verifies that each member is reachable from the selected source roots, belongs to that namespace and retains its UUID and definition-member role. Duplicate, unrelated and malformed member overlays reject before destination replacement. Dependency overlays do not become additional selection roots.
- Editable member BRep/mesh replacements use definition-local millimeters. Nested instance member placement overlays preserve the native definition graph. A member shared by repeated instances is replaced once. Imported namespaces receive independent edits, including when their original archive UUIDs collide.
- Geometry edits recompute definition and instance bounding boxes. Member lock overlays retain `ON::idef_object` mode, using the existing `OpenMatrix9.Locked` user string rather than changing the member into top-level geometry. Source snapshots remain immutable.
- The pinned openNURBS instance classes do not implement their own `UpdateReferencedComponents` override. The merge writer explicitly remaps both instance-definition member UUID lists and instance-reference definition UUIDs. A regression with different edited points in two namespaces exposed the earlier incorrect shared-definition result; after the fix both distinct world coordinates are verified in native and FreeCAD tests.
- Layer metadata overlays reuse existing hierarchy entries or create bounded native layer paths separated by `::`. Native attributes and dependency edges point to the final layer; parent dependencies are included in the selected closure. Unicode paths, shared hierarchy reuse and invalid-path atomic rejection are checked.
- Retained FreeCAD objects now bind imported layer, color, lock and visibility attributes before capturing their baseline. These objects previously bypassed the editable geometry inserter and lost their visible metadata properties.

## Evidence

Native `ThreeDmMergeTests`: RED ignored member edit; GREEN local coordinates, UUID/role retention, refreshed bounds, definition-mode lock retention, independent namespace edits, Unicode layer hierarchy and atomic invalid/unreachable/duplicate rejection. Full native CTest8/8 passed, exit0,33.54s, including both immutable user-ring geometry fixtures. SDK module and scripts build exit0.

Fresh FreeCAD runtime results:

| Suite | Result | Artifact |
|---|---|---|
| Member overlays |12/12, process0|`build/three_dm_member_overlay_smoke-1/9bac2e27d1494d168f53a1254dfb6f2d/results.json`|
| Layer overlays |7/7, process0|`build/three_dm_layer_overlay_smoke-1/f58884e41c08467982d858355bca47fe/results.json`|
| Structural blocks |23/23, process0|`build/three_dm_block_structure_smoke-1/14e2b8d8d5c940029dbbf52dea68c92f/results.json`|
| Mixed shear preview |9/9, process0|`build/three_dm_mixed_shear_smoke-1/7958b72a33d7495e8d6db6a5b211c046/results.json`|
| Source signatures |14/14, process0|`build/three_dm_source_signature_smoke-1/5a8c1a62c6c54c138f8483ee6505654f/results.json`|
| Preservation lifecycle |19/19, process0|`build/three_dm_preservation_smoke-1/953258a3062c458ab57079be967a618e/results.json`|
| Preserved export |13/13, process0|`build/three_dm_preserved_export_smoke-1/fa03d27934b446f1a6acf871fe2729d1/results.json`|

## Remaining scope

Definition membership changes/deletions, definition-container placement/name overlays, unsourced new geometry and distinct-archive document merging are still pending. Derived affine display previews currently remain at the imported geometry after canonical member edits; these tests verify the updated native export, not automatic regeneration of those previews. Standard export/menu/CMD routing is still guarded. Arbitrary opaque plugin payloads, appearance/resources/history semantics, all-class Rhino5 compatibility, actual Rhino5 application acceptance, final review and public-source integration remain incomplete. The ring regression results prove geometry handling, not full preservation export of their opaque plugin tables.

Subsequent implementation supersedes the membership/name and preview-refresh gaps above: `2026-10-06-3dm-definition-preview-overlays.md` records ordered membership edits, empty blocks and automatic affine preview regeneration. Definition placement, unsourced new geometry and the broader compatibility/integration scope remain pending.
