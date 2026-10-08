# Selected native definition members

Standard selected export now accepts supported canonical native definition members and their physical shared proxies, including independent source-backed copies. Export creates an independent native top-level record with a stable UUID derived from the import namespace, original native UUID and physical internal host Name. Original member IDs, definition membership, provenance signatures and included source bytes stay intact; export adds no document objects or source properties. Renaming labels does not redefine the identity. Selecting a member alongside its placed block retains both the original canonical member and the independent selected root. Unrelated ancestor definitions and placements are omitted. Selecting a nested reference retains only its current required target closure, including a verified new definition target.

This is an OpenMatrix9 selection policy for whole tree objects. Coordinates use the selected physical host's document/world frame, including its definition container Placement and its own native delta. A placed block's matrix is not guessed for a member shared by several instances. Sub-element/path selection and link arrays require separate representation/export support. Existing subelement guards remain; selected link arrays explicitly refuse.

Unchanged CAD/native curves preserve the original native payload and apply the physical parent/proxy matrix. Changed supported CAD/mesh uses the actual current local payload followed by that matrix. Retained TextDot/PointCloud uses its explicit native delta, supported metadata and original payload; arbitrary opaque payload edits refuse. Native nested references use verified current native instance overlays and target dependencies. A reflected/scaled shared proxy applies its own matrix; LinkTransform=False cancels the canonical Placement exactly once. Current copy overlays remain independent of simultaneously exported canonical edits. Explicitly identified new CAD/mesh members can also be selected alone or with their block; their selected-root UUID is separate from their membership UUID and remains stable without a retained import namespace.

The existing bounded member-copy protocol gains an optional `role` of top-level or definition-member (default unchanged). Native allocation and refresh set the appropriate object mode and record role. A selected root has a separate host binding from its canonical dependency overlay. All copies must be reachable from the selected graph; malformed roles/matrices, colliding UUIDs, unsupported userdata/self-UUID references and missing/cyclic dependencies refuse before destination replacement. Rust's existing copy/overlay and closure/budget policy remains authoritative. Final native class/reference/version/resource guards, semantic manifest equality and complete serialized geometry/attribute digests remain required.

## Native 2D curve transforms

Tests found that native ON_Curve::Transform on a2D curve can discard transformed Z. The shared native helper explicitly calls ChangeDimension(3) only when the affine matrix maps the XY input plane outside XY. It requires successful3D promotion; plane-preserving maps retain the original2D dimension. Rational degree/order, knot vector, homogeneous weights and mathematically transformed control points survive. The source curve remains2D. The geometry-only block reader now uses this helper too: an independent native regression failed for its out-of-plane point before the fix, then passed. No additional curve fitting or tessellation is introduced.

## Acceptance evidence

- RED: direct canonical member selection failed at the old top-level-only guard. Mixed block/member selection exposed a host-binding collision. The copied-role default initially exposed QJsonObject inserting an absent field on nonconst operator[]; value() now preserves the default. App::Link lacks getGlobalPlacement on this runtime, so physical parent composition uses the existing verified source_placement adapter.
- GREEN:33/33 focused GUI assertions cover canonical/retained/current CAD and mesh, shared rational and reflected CAD proxies, stable identities without mutation, exact closure, mixed member/block selection, Undo/Redo/FCStd, mm/cm PointCloud fields/bounds, two namespaces, repeated export, new members and current new reference targets. Actual native named, menu, CMD and Std_Export dialogs write the same selected-member UUID and geometry.
- Selected edited point(2,3,4) with member translation(5,6,7) and definition translation(100,200,300) exports(107,209,311). Retained TextDot(4,5,6) under T(10,20,30)*Rz90 and that parent exports(105,224,336); the original block member stays local(5,24,36). Selected edited box volume120 and mesh vertices(5,6,7),(7,6,7),(5,9,7) are independently checked. A reflected CAD proxy exports volume2880. A selected nested reference with a new target exports box volume24 at X100.
- Native reread directly checks PointCloud points, non-axis normal directions/original magnitudes, per-point colors, user strings, top-level object mode, no ancestors, mixed original/member IDs and atomic invalid role/unreachable-copy rejection. Native rational2D/3D oracles compare exact knots/weights and analytic transformed control points; a geometry-only block oracle checks its parametric point including Z9.
- The GUI curve oracle integrates derivative magnitude separately over knot spans using adaptive Simpson integration, with a finite accumulated error estimate below1e-8mm and length agreement within2e-8mm. Cached Shape.Length is recorded diagnostically rather than used as a precision oracle: its two values differed by0.0011756mm even when both exact curves agreed. Bounds agree within1e-6mm. Integration evidence from the fresh full run is recorded below.
- Fresh isolated FreeCAD690/690 across36 suites, each final owned process0 and stable source/runtime/binary fingerprints throughout. Fresh native9/9,29.36s, final exit0, including both immutable user ring fixtures. Rust source/tests are unchanged at their prior verified75/75/fmt hashes; no new Rust-test claim is inferred from C++/GUI results.

## Remaining scope

These fixtures establish the named class/selection interactions, not class-wide display/edit/field/resource compatibility. Arbitrary retained/plugin graphs, undocumented userdata identity remapping, link arrays/subelements, large graph performance, remaining legacy adapters, linked/external resources and complete appearance/document/history/reference/version semantics remain pending. Existing source-version rejection protects incompatible newer content; actual Rhino5 application behavior remains untested. Final whole-package review, primary/public integration and the full openNURBS goal remain in progress. Source is build/om9-dev; runtime is build/3dm-preservation-sdk.

## Reproduction

`rtk proxy cmd /c H:/FreeCAD-src/build/3dm-isolated-module.cmd`

`rtk proxy cmd /c H:/FreeCAD-src/build/3dm-preservation-regressions.cmd`

`rtk proxy H:/FreeCAD-src/.pixi/envs/default/python.exe H:/FreeCAD-src/build/run-selected-member-regressions.py`

Finish native fixture generation before GUI regressions. Reports require final owned process exit0, not merely completed assertions or an observation timeout.

|Suite|Checks|Result artifact|
|---|---|---|
|selected_member|33/33|`build/three_dm_selected_member_smoke-1/a32acfc32156412d8a72591db7870de4/results.json`|
|retained_transform|25/25|`build/three_dm_retained_transform_smoke-1/f3fe4fd452784d89b660d00f548705b6/results.json`|
|host_upgrade|39/39|`build/three_dm_host_upgrade_smoke-1/1d0b600123cd439688ec6e059a2bcc14/results.json`|
|shared_export_routes|29/29|`build/three_dm_shared_export_routes_smoke-1/54a09c0d7da843638b0eb83793990c36/results.json`|
|recursive_shared_migration|24/24|`build/three_dm_recursive_shared_migration_smoke-1/3e18a63a9467434ea059eb925239815f/results.json`|
|recursive_namespace_migration|25/25|`build/three_dm_recursive_namespace_migration_smoke-1/e33ab56b5bbd43ad8734b3bf1a6938f8/results.json`|
|affine_target_migration|32/32|`build/three_dm_affine_target_migration_smoke-1/ba4c259a4d7044d48d5addc2f86f34af/results.json`|
|copy_origin_migration|22/22|`build/three_dm_copy_origin_migration_smoke-1/5519bbeb045d4f76a50ead1a6e3f9520/results.json`|
|legacy_migration|35/35|`build/three_dm_legacy_migration_smoke-1/56c840f2d61a4d51bd5ee2005e592939/results.json`|
|proxy_retarget|6/6|`build/three_dm_proxy_retarget_smoke-1/01684d92e7d64956a36b819d6a98da31/results.json`|
|source_proxy_creation|34/34|`build/three_dm_source_proxy_creation_smoke-1/5288a87fd1c54d46b834b1278f58fe96/results.json`|
|definition_copy|27/27|`build/three_dm_definition_copy_smoke-1/019a44a9982b4c76a6d31357e3ba41c6/results.json`|
|source_members|23/23|`build/three_dm_source_members_smoke-1/1bf796c501094ad4be9cc53afe306ae1/results.json`|
|source_member_geometry|8/8|`build/three_dm_source_member_geometry_smoke-1/e803309073e14ad1aa6243babe9b2ff1/results.json`|
|source_new_target|27/27|`build/three_dm_source_new_target_smoke-1/b6deab7b59284324bb2970ee8021ea68/results.json`|
|new_blocks|29/29|`build/three_dm_new_blocks_smoke-1/c596408da95d43d99b334480b55bc631/results.json`|
|new_member_instance|20/20|`build/three_dm_new_member_instance_smoke-1/35ee8a3b2e224b7888aa7bfe61ce1d8a/results.json`|
|reference_edit|18/18|`build/three_dm_reference_edit_smoke-1/b53c016aa6b24fc29c5c125475c3f27d/results.json`|
|independent_member|18/18|`build/three_dm_independent_member_smoke-1/8bac92276ef84361bfcb18ed821b0179/results.json`|
|shared_proxy|10/10|`build/three_dm_shared_proxy_smoke-1/d63f86338d834160aa0620925a0bc826/results.json`|
|shared_proxy_geometry|8/8|`build/three_dm_shared_proxy_geometry_smoke-1/a8ac04393f5d492ca1be297d5526307e/results.json`|
|retained_member|6/6|`build/three_dm_retained_member_smoke-1/f3165595dbd64d3aad93fcb82e3ff0f5/results.json`|
|definition_placement|12/12|`build/three_dm_definition_placement_smoke-1/863ac7deb68043c9b6736cbf41a4fc28/results.json`|
|new_geometry|20/20|`build/three_dm_new_geometry_smoke-1/22dfb400844d42d183b4d22f039db2d6/results.json`|
|preserved_export|13/13|`build/three_dm_preserved_export_smoke-1/8574dd57279f4045a33b7cd191293ba3/results.json`|
|member_overlay|12/12|`build/three_dm_member_overlay_smoke-1/5a5de1caaed8487a891a9d43d52b1217/results.json`|
|block_structure|23/23|`build/three_dm_block_structure_smoke-1/0e00febdebe9403ea33ccc9d62a28790/results.json`|
|affine_refresh|10/10|`build/three_dm_affine_refresh_smoke-1/2e02033604f64a1b805b4e7af6c001d7/results.json`|
|affine_refresh_reopen|2/2|`build/three_dm_affine_refresh_reopen_smoke-1/1b6739a3f85744d7a6da1983c240e829/results.json`|
|source_signature|14/14|`build/three_dm_source_signature_smoke-1/aa73fb3a598e414093648296b63fa189/results.json`|
|preservation|19/19|`build/three_dm_preservation_smoke-1/924b61d3c3c4419cb948d1d14fe39f25/results.json`|
|definition_overlay|8/8|`build/three_dm_definition_overlay_smoke-1/de079a0213664fee9c6c267c3c086633/results.json`|
|layer_overlay|7/7|`build/three_dm_layer_overlay_smoke-1/fc345dca16164cb5bd28aea324aa81d4/results.json`|
|mixed_shear|11/11|`build/three_dm_mixed_shear_smoke-1/4f570ee4ccd241a9a5a4ed6717abb256/results.json`|
|merge_bridge|8/8|`build/three_dm_merge_bridge_smoke-1/da17d185422348968a3959668e64080f/results.json`|
|geometry|33/33|`build/three_dm_smoke-1/38b7a070bb7046bd88c7719bc78e6df4/results.json`|

Curve integration evidence:

```json
{
  "actual_length": 21.16952623043094,
  "expected_length": 21.168350597084963,
  "actual_bounds": [
    -2.0000000000000036,
    9.68610023441978,
    -4.000000000000009,
    7.999999999999998,
    8.999999999999998,
    8.999999999999998
  ],
  "expected_bounds": [
    -2.0000000000000018,
    9.686100234419776,
    -4.0000000000000036,
    8.0,
    9.0,
    9.0
  ],
  "integrated_actual": 21.168350597083574,
  "integrated_expected": 21.168350597083567,
  "integration_error": 6.790648082617238e-10
}
```

## Fingerprints

- `ThreeDm.py` SHA256 `65b837a9ff628284b74a370f4fe7c8bf1248289412fa38d0d533ff58973b186d`
- `ThreeDmArchiveState.py` SHA256 `c52e6af41643d5a8b122210817aee24e9ba0f23116a0cefe54bceb95af04729c`
- `ThreeDmMigration.py` SHA256 `86f573e4090b1d80d76627dff849fb742e7f1cebed5ba59346056853f695a9f5`
- `ThreeDmUpgrade.py` SHA256 `5fcf9e048f914175136dd489709e82e22adafc512d9369a6773aa701da3d25a6`
- `InitGui.py` SHA256 `2d920887a898a94f90eef120bdc14f989a96567eb8094e66f6b6cd022389f157`
- `Gui/ThreeDmArchive.cpp` SHA256 `9c21edbda9be29bb98697170d560e2f07257060c56617e04cf39cd9b4e1a9f73`
- `Gui/ThreeDmTransforms.cpp` SHA256 `48186208868279b99fb6aa40309db616b831db335eaf8a8e225d6c0edbfe3f3b`
- `Gui/ThreeDmMerge.cpp` SHA256 `fa354c7062473580afba59f44aa4c84dddb5b922110ccadf7ce402718ecab991`
- `tests/native/three_dm_native_transform.cpp` SHA256 `8e7635d1133dd0129c06b79771be5de08e31145e61462b18af49e86ff628a123`
- `tests/three_dm_selected_member_smoke.FCMacro` SHA256 `1aeb0d0658b338bb137b09e307000d2e65baa1a7abf9c006c8e69409c3cee119`
- `docs/3dm-coverage.json` SHA256 `60cec1c6e6e009fa3ae6888eae6c0836bf41f8856a05340a8789d8be14fa932a`
- `docs/development/3dm-structural-blocks.md` SHA256 `7c26ca96f51b47d66fc8b37673f0fed48182447453749ed7d46864236c71a573`
- `docs/features/OM9-FILE-012.md` SHA256 `0ce703a80a8e5d2efee928c5695f24acb8cd34fef8c407fcabbf90abc2ec3dca`
- `docs/features/3dm-support-matrix.md` SHA256 `a6ea16eb58a1ecfd39261694547fe4c3f8ce478544a82f54bf7dfde0046b1e5a`
- `ref/matrix9/OpenMatrix9_Codex_Spec_v1/specs/01-core/om9-file-012-rhino5-3dm-exchange.md` SHA256 `32256bf2cae3c6e72224f09ae03451d7936444f8659a58d1d888d9faa4075e5c`
- `ref/matrix9/OpenMatrix9_Codex_Spec_v1/MANIFEST.json` SHA256 `b8f93e4043c75478ae805f97f3a75e22815703cc98bd5560e00109f049211d2e`
- `docs/superpowers/specs/2026-10-06-full-opennurbs-design.md` SHA256 `f436497b36ec76b04302c1a72199790130ed85271f9f1b333aa712de27105527`
- `docs/superpowers/plans/2026-10-06-3dm-merge-blocks.md` SHA256 `e296623ca7b794a2b4ad8a35123f38be097b85b8f2348dee2803a99ccf52f5d7`
- `README.md` SHA256 `1686b724002d07f83ba68f9ecc98ad2cb778b3055697084e6f126ab4dee49b55`
- Isolated `OpenMatrix9Gui.pyd` SHA256 `7486c1dce968326e8e63ef9985f190601a7febf38096a8988fcf22f26d157354`
