# Definition placement and link scaling

Implemented and verified in the private development checkout. This extends the new-geometry package; it does not complete full openNURBS or publish/integrate the changes in the primary public checkout.

## Behavior

- Source block links export the geometry FreeCAD displays. `LinkTransform=False` overrides the linked definition placement; `True` composes the instance placement, link scale vector and definition placement, in that order. Native instance overlays retain the original definition/member UUID graph.
- Nonuniform and reflected link scaling uses the existing full native4x4 instance overlay. Singular scaling fails before destination replacement. Nested member instances follow the same local matrix composition.
- Affine instance previews include current definition placement and nested link transforms. Canonical local CAD/mesh remains the export source. Definitions that share a canonical member remain valid because the definition placement is represented in each applicable native instance matrix, rather than repeatedly transforming the shared geometry.
- Definition-local member signatures exclude the container's parent placement. Moving the definition no longer falsely replaces local canonical geometry. Native record/UUID/geometry CRC retention is tested.
- Instance signatures track the authoritative native record/matrix, placement, metadata and applicable link scaling/definition transform. Rebuilding equivalent derived preview topology no longer changes the source baseline. Separate preview integrity signatures still reject manually edited preview geometry, including nested affine members at export.
- Source links retargeted to a different definition are rejected explicitly; no stale original reference silently overrides the edited host link. Reference editing remains a separate pending feature.
- Placement and scaling survive FCStd reopening. There is no intrinsic placement field in the native instance definition: the output retains equivalent geometry and block references through the composed instance matrices.

## Evidence

Host RED: definition placement was rejected. Local FreeCAD `src/App/Link.{h,cpp}` and actual `Part.getShape` measurements established `LinkTransform` and scale composition. Further structural regression RED: an unrelated recompute rebuilt equivalent affine preview topology and changed its source signature. Authoritative instance signatures now remain stable while the existing manual-preview rejection tests remain green.

SDK scripts build exit0. Native CTest8/8 exit0,30.01s from the unchanged native writer tested in the new-geometry package, including both immutable ring geometry fixtures. Final FreeCAD suites total140 passing checks, all process0:

| Suite | Result | Artifact |
|---|---|---|
| Definition placement/link scale |12/12|`build/three_dm_definition_placement_smoke-1/4e8e5d2d1e684638be801d8af36d839e/results.json`|
| New geometry/members |20/20|`build/three_dm_new_geometry_smoke-1/c0c18bb5a71c4902aac499d9fe1130b9/results.json`|
| Preservation export |13/13|`build/three_dm_preserved_export_smoke-1/16dd244b35d6440483df5f6c93ea96fd/results.json`|
| Member overlays |12/12|`build/three_dm_member_overlay_smoke-1/fa82c8c5deef4b4eafc0a786080f6629/results.json`|
| Structural blocks/signatures |23/23|`build/three_dm_block_structure_smoke-1/5a24b080620d49b4bb2be3ce3d920650/results.json`|
| Affine refresh |10/10|`build/three_dm_affine_refresh_smoke-1/6644a4f76b8e4e4887ce02ac8ee340ab/results.json`|
| Fresh-process FCStd refresh |2/2|`build/three_dm_affine_refresh_reopen_smoke-1/e7fdc7a458d542af943f7ad7033aac0d/results.json`|
| Source signatures |14/14|`build/three_dm_source_signature_smoke-1/f900f9d02d0c40edb4fa4d5943dd1eb9/results.json`|
| Lifecycle/rollback |19/19|`build/three_dm_preservation_smoke-1/d35e3006a5f041ec98bae1ccfbbdce52/results.json`|
| Definition membership |8/8|`build/three_dm_definition_overlay_smoke-1/feceda7d3870441790bdf2f6e50b66bd/results.json`|
| Layer overlays |7/7|`build/three_dm_layer_overlay_smoke-1/104e1c302c5d4ea2b4a855da5cd9a54c/results.json`|

The fresh-process test runs after the current affine suite has saved its reopen fixture. Tests compare exported world points to live FreeCAD geometry for mixed true/false LinkTransform, nested reflected/nonuniform member scaling and shared definitions; affine BRep vertices/volume and mesh vertices match the regenerated previews. Failed retarget/singular exports retain destination bytes.

## Remaining scope

Transformed shared-member proxies, new standalone definitions, legacy owner-link/baseline migration and explicit definition reference edits remain pending. Distinct-source document metadata merge policy, standard/menu/CMD preservation routing, complete-package review and primary public-source integration remain pending. General appearance/resources/history, unknown plugin dependencies, every pinned class and Rhino5 compatibility still require coverage. Actual Rhino5 application behavior is untested. The comprehensive goal remains active.
