# Native PointCloud current fields and derived colored display

Preservation import now binds ON_PointCloud to a class-specific FreeCAD FeaturePython adapter. Current points and normals use double PropertyVectorList; colors use flattened exact RGBA integer bytes; intensity and the16 plane doubles use FloatList; ordered/has-plane use boolean properties. The original source record and included archive stay immutable. The current fields persist independently through Undo/Redo and FCStd after original source removal. Export never takes coordinates from the Coin float PointSet or a float Points kernel.

Per-point arrays are not embedded in the32MiB manifest/request. Preparation creates bounded complete field JSON files with SHA256; Python checks aggregate prepared bytes before parsing/binding. Current edits stage their own complete files. The writer bounds total field inputs across namespaces and independent copies to512MiB, verifies each file's SHA256, actual native class, complete schema/keys, finite coordinates/normals/values, integer RGBA bytes, optional array counts and active plane validity before assigning an edited native clone. ON_PointCloud::IsValid only checks nonempty points in the pinned library, so the adapter validates these other fields explicitly. Source userdata, attributes and reserved flag bits survive. Point changes invalidate only derived bounds; exact identity transforms preserve native serialized caches. Valid dormant plane frames are normalized/transformed without enabling the has-plane flag.

The inventory records bounded binary little-endian per-array SHA256 plus counts, plane, flags and computed bounds. Canonical definition overlays, independent copied members/new definitions, selected native members, physical shared proxies, top-level duplicates and multiple namespaces all use generic native-field dispatch. TextDot retains its old signature representation. Current fields apply before the member's own affine matrix; output-copy field overlays apply after copy refresh. Normal transforms use inverse transpose and preserve original magnitude, including zero normals. Full native geometry/attribute digests and semantic reread still gate atomic destination replacement.

## Verified evidence

- RED: native exchange refused PointCloud file-field overlays at the old unknown-field guard. GUI source-copy export then exposed the aggregate preflight checking action=duplicate before conversion to duplicate_action=transform; validation now checks the declared payload action. The first oracle helper lacked a staging directory and the display-persistence assertion compared the BoolList wrapper directly with a list; these were test-harness issues, corrected after inspecting exact diagnostics. No production workaround was added for those harness errors.
- GREEN:43 focused FreeCAD checks verify exact double current arrays/optional fields, native RGBA alpha, plane/ordered state, source/userdata/attribute retention, Undo/Redo, parent/native delta, current canonical/member/source/definition copies, stable selected-member identity, reflected nonuniform physical proxies, two namespaces, mm/cm, FCStd after original deletion, local hiding/size persistence, explicit optional-array clearing, missing baseline/source/schema/field-count/singular-proxy rejection and atomic intensity target gating. The native field-file oracle rereads actual output arrays, not only count/bounds/CRC.
- Native direct helpers and mm/cm writer fixtures verify full fields, exact serialized identity caches, dormant-plane normalization, inverse-transpose normals, reserved flags, source/userdata immutability, selected closure, canonical and independent copied geometry, malformed keys/types/counts and file hash/path/class guards.
- Fresh isolated FreeCAD767/767 across38 suites; each final owned process exit0 with stable source/runtime/binary fingerprints. Native11/11,30.50s, final exit0, includes both immutable user ring fixtures. Fixture generation completed before the final GUI run. Rust source/tests are unchanged at the prior verified75/75/fmt fingerprints; this is not a fresh Rust test claim.

## Compatibility and coverage limits

The pinned modern ON_PointCloud writer emits chunk1.2, including m_V, even with the archive header set to Rhino5. The modern library reading its own output is insufficient evidence that the actual Rhino5 reader accepts or retains intensity. The [official RhinoCommon GetPointValues documentation](https://mcneel.github.io/rhinocommon-api-docs/api/RhinoCommon/html/M_Rhino_Geometry_PointCloud_GetPointValues.htm) describes point values including intensity and reports the API since7.5; this does not establish Rhino5 file-reader behavior. Import/edit/FCStd preserve intensities. Selected native closure with nonempty current intensity is refused before destination replacement until compatibility is verified. Unrelated intensity records do not block exports whose closure omits them. Explicitly clearing current values permits export and leaves the included source untouched. This is an interim compatibility gate to resolve under the full goal, not a completed intensity exchange feature or a claim of proven incompatibility.

Coin derives PointSet coordinates and per-point color/transparency from current data. Native alpha0 is opaque and255 transparent. Local hidden flags and point size persist as FreeCAD display properties; openNURBS does not serialize runtime hidden indices in3DM. Hiding changes display only, and export retains all points. Exact Rhino rasterization, normal/intensity visualization, large-data performance/memory and streaming remain unverified. The512MiB serialization cap is a supported safety limit, not proof that very large datasets are efficient. PointCloud children inside affine retained compound previews and geometry-only PointCloud editing/export remain pending. Nonfinite dormant plane data and arbitrary legacy holder upgrades are not covered by this slice.

No class/category is marked fully supported from these fixtures. Remaining annotation/text/dimension/hatch/page/clipping geometry, linked/external resources, appearance/components/document/history/reference/version and legacy adapters remain pending, together with actual Rhino5 acceptance, final whole-package review and public integration. The full openNURBS goal stays active. Authoritative development source is build/om9-dev; the runtime is isolated in build/3dm-preservation-sdk.

Spec01-core extension, README, support matrix, scoped coverage and progress ledger are synchronized. IMPLEMENTATION_STATUS.md is absent in the referenced Spec v1 package; FEATURES.json/yaml do not contain this user-authorized OM9-FILE-012 extension. Their source catalog rows are unchanged. No reusable skill or business rule is modified.

## Reproduction

`rtk proxy cmd /c H:/FreeCAD-src/build/3dm-isolated-module.cmd`

`rtk proxy cmd /c H:/FreeCAD-src/build/3dm-preservation-regressions.cmd`

`rtk proxy H:/FreeCAD-src/.pixi/envs/default/python.exe H:/FreeCAD-src/build/run-point-cloud-regressions.py`

|Suite|Checks|Result artifact|
|---|---|---|
|point_cloud|43/43|`build/three_dm_point_cloud_smoke-1/821fd69e6ac741ed9e42fbb7df365c07/results.json`|
|text_dot|34/34|`build/three_dm_text_dot_smoke-1/8dc78d923a724b0293abba818823118f/results.json`|
|selected_member|33/33|`build/three_dm_selected_member_smoke-1/96f558bd9d504c419803682863f04a13/results.json`|
|retained_transform|25/25|`build/three_dm_retained_transform_smoke-1/1198a17260d74389961d58bfcdb9d689/results.json`|
|host_upgrade|39/39|`build/three_dm_host_upgrade_smoke-1/d099b1e5c0844909a3fd16d47aa5147b/results.json`|
|shared_export_routes|29/29|`build/three_dm_shared_export_routes_smoke-1/5b8dd0d6edc647819a8eb12617aff728/results.json`|
|recursive_shared_migration|24/24|`build/three_dm_recursive_shared_migration_smoke-1/5abd46114086447f8c1f075b3df9b4e2/results.json`|
|recursive_namespace_migration|25/25|`build/three_dm_recursive_namespace_migration_smoke-1/80461e7a45f14b1d8f97d067ce6e61f8/results.json`|
|affine_target_migration|32/32|`build/three_dm_affine_target_migration_smoke-1/1484bab0119c4d3e918aab5b24d78331/results.json`|
|copy_origin_migration|22/22|`build/three_dm_copy_origin_migration_smoke-1/62b14d9c67d7494986a5c693395aed1a/results.json`|
|legacy_migration|35/35|`build/three_dm_legacy_migration_smoke-1/bd537c9687dc4deb8816a9900c2ae343/results.json`|
|proxy_retarget|6/6|`build/three_dm_proxy_retarget_smoke-1/28a90ff8af3e4c00b1fc6a3b1db7355b/results.json`|
|source_proxy_creation|34/34|`build/three_dm_source_proxy_creation_smoke-1/1cba10538d224de6a9e1a563d78050cf/results.json`|
|definition_copy|27/27|`build/three_dm_definition_copy_smoke-1/f3a4abe454524df1b018495419333f41/results.json`|
|source_members|23/23|`build/three_dm_source_members_smoke-1/3669eb2628304e048998508152303a1a/results.json`|
|source_member_geometry|8/8|`build/three_dm_source_member_geometry_smoke-1/79468ef273684efda5b48206bc5a4bb3/results.json`|
|source_new_target|27/27|`build/three_dm_source_new_target_smoke-1/d2c7f27b0b0e4a79a746fdfae2b37bae/results.json`|
|new_blocks|29/29|`build/three_dm_new_blocks_smoke-1/10db9a29e8b344cfbfff7936eaf4bd65/results.json`|
|new_member_instance|20/20|`build/three_dm_new_member_instance_smoke-1/e617d5d133e44b949ded891245159296/results.json`|
|reference_edit|18/18|`build/three_dm_reference_edit_smoke-1/5bc046116d7c4e60b712f493416ac10d/results.json`|
|independent_member|18/18|`build/three_dm_independent_member_smoke-1/e09a6eaeea5846c79ef536b31ce58198/results.json`|
|shared_proxy|10/10|`build/three_dm_shared_proxy_smoke-1/bf338cb324494db68b170bd6760c5136/results.json`|
|shared_proxy_geometry|8/8|`build/three_dm_shared_proxy_geometry_smoke-1/df00de0d1dd044ab968d4ded3a4f4f0e/results.json`|
|retained_member|6/6|`build/three_dm_retained_member_smoke-1/e3a29bc14e124f14b73bb8bfd181bd9a/results.json`|
|definition_placement|12/12|`build/three_dm_definition_placement_smoke-1/e8b5e2d5074c4a88b1026346d11b15b1/results.json`|
|new_geometry|20/20|`build/three_dm_new_geometry_smoke-1/d8541626339f46d7a800ae4d55e00cc4/results.json`|
|preserved_export|13/13|`build/three_dm_preserved_export_smoke-1/c1f77de44e3248f8be503398e05cfc25/results.json`|
|member_overlay|12/12|`build/three_dm_member_overlay_smoke-1/e94d9350e44842b483af364a93df3281/results.json`|
|block_structure|23/23|`build/three_dm_block_structure_smoke-1/cc62ee992b8649ec8a018e883d6f4b4c/results.json`|
|affine_refresh|10/10|`build/three_dm_affine_refresh_smoke-1/9a63547197e94fa5a17bdfb3110c797d/results.json`|
|affine_refresh_reopen|2/2|`build/three_dm_affine_refresh_reopen_smoke-1/a679e33ec2034ab393df638e69052cc1/results.json`|
|source_signature|14/14|`build/three_dm_source_signature_smoke-1/9aee69860a13412aa3783e51b0bfead2/results.json`|
|preservation|19/19|`build/three_dm_preservation_smoke-1/c85144cdff1b4314b85fc3b47ea8547b/results.json`|
|definition_overlay|8/8|`build/three_dm_definition_overlay_smoke-1/edc6fd6853ce4afa894f4fa65444b14a/results.json`|
|layer_overlay|7/7|`build/three_dm_layer_overlay_smoke-1/cffdb92df88a445a9579e2f2a1a675d2/results.json`|
|mixed_shear|11/11|`build/three_dm_mixed_shear_smoke-1/83d8e53b4981442e8944ea87f398fbda/results.json`|
|merge_bridge|8/8|`build/three_dm_merge_bridge_smoke-1/0b9a426261884cd198147aad137060e6/results.json`|
|geometry|33/33|`build/three_dm_smoke-1/9be7ee2cdfdf4107b697627b51c6ad5e/results.json`|

Additional viewport smoke:2/2, final owned process0; the active viewport contains the current Coin PointSet. Rendered `H:\FreeCAD-src\build\om9-dev\build\three_dm_point_cloud_display_smoke-1\e127d249cc6f4065a1ec68c443d1f856\native-point-cloud.png` was inspected: distinct red/green/blue/black current point markers are visible. Exact Rhino appearance remains unverified.

Checkpoint: `H:\FreeCAD-src\build\checkpoints\point-cloud-20261007-042655.zip`; every archived file is independently SHA256 verified.

## Fingerprints

- `ThreeDm.py` SHA256 `581b1dfc7fdccf69498bccbe161bdd792df814ef8818f8c6d3a4f9638bacf649`
- `ThreeDmArchiveState.py` SHA256 `72a41cc7de6d9afd78b92b870b3c58bdb9f222282a5ef04bc90225258007276e`
- `ThreeDmTextDot.py` SHA256 `d345d8c5f4e58a7270e622f88fed28ce94f9856137d0c5b50c2b16575a544537`
- `ThreeDmPointCloud.py` SHA256 `b806459c5691180da010d7daf30f9509ad45ee2a28f46cf96f087d0475f2e601`
- `ThreeDmNativeFields.py` SHA256 `b7000a7d672e2f99296b8480e1edc5eadea9984b8226dfde6b1204aea1f0b21d`
- `ThreeDmMigration.py` SHA256 `86f573e4090b1d80d76627dff849fb742e7f1cebed5ba59346056853f695a9f5`
- `ThreeDmUpgrade.py` SHA256 `5fcf9e048f914175136dd489709e82e22adafc512d9369a6773aa701da3d25a6`
- `InitGui.py` SHA256 `2d920887a898a94f90eef120bdc14f989a96567eb8094e66f6b6cd022389f157`
- `CMakeLists.txt` SHA256 `b8733f745bba62063840887f5ba807871c5016cf151edffd71115c9f8106d5ef`
- `cmake/OpenNURBS.cmake` SHA256 `67694b4f1bc1d9716f82457748769e5fb78cda5e154582fc124499a56e89eb66`
- `Gui/ThreeDmInventory.cpp` SHA256 `c09039bcdffceebef069fead3699207874b9630388e94dcd585c74b6fad83caa`
- `Gui/ThreeDmMerge.cpp` SHA256 `47c022b145b80c6d0e6d74ac9b8fe768ab768f3186ad6f3fe410a6f5bf3f24cf`
- `Gui/ThreeDmPointCloud.h` SHA256 `3c1ef47e29a8b1717c80b5608beb16248423175f55242334949b9f8fec66a3c0`
- `Gui/ThreeDmPointCloud.cpp` SHA256 `b6e3cdfb2218e50a91342d20f351076552299a95b0b75d9311ba6b54aaac805c`
- `Gui/ThreeDmTransforms.cpp` SHA256 `d0d13eae1c2ce629e272d693114ba2682ae9d3d03f39f155850f43f7d15c7079`
- `Gui/ThreeDmPython.cpp` SHA256 `d0cda1a53491123efae144626fa9ffe3bad027952be1581ccf89eda149ad07bd`
- `tests/native/CMakeLists.txt` SHA256 `208c5268ff0269db9f28636b23203d6c45af48857bd29749c2049e3623792449`
- `tests/native/three_dm_point_cloud.cpp` SHA256 `629850840d0504efc4099ec832f1f7e9fb4bf2a87e1e2ae4ae1ed1c8551da647`
- `tests/three_dm_point_cloud_smoke.FCMacro` SHA256 `e64e12ba74b7f133ba82e5359593df52d0653b497af57353ab9b3fdc11225e60`
- `tests/three_dm_point_cloud_display_smoke.FCMacro` SHA256 `5a05b2e6e773cde013cb26438f50e9f5baac6d4fef8c2b3c755745667919072c`
- `docs/3dm-coverage.json` SHA256 `4977541f278b5d26cdf13ade9c4974e03708ce2dcc133cf1f3b2149c0a7d1a4e`
- `docs/development/3dm-structural-blocks.md` SHA256 `1cb0778950b9adcedddeb18ed6110676a21fa66ac052e269842db1590746caa3`
- `docs/features/OM9-FILE-012.md` SHA256 `f5b9dea4dbd1e5ee022945f06566a51b90641a7162cbe4d54861db50ea1c873f`
- `docs/features/3dm-support-matrix.md` SHA256 `110ad5d9664e5526cf01aa2b539db777ff1091348f314e46a50adad508b5bf86`
- `ref/matrix9/OpenMatrix9_Codex_Spec_v1/specs/01-core/om9-file-012-rhino5-3dm-exchange.md` SHA256 `fd57fb8c3633fb982b7636696444c21e426a6399dd31fac4b07df033d671127b`
- `ref/matrix9/OpenMatrix9_Codex_Spec_v1/MANIFEST.json` SHA256 `6d6dfb5561e210a5d1278939eaf46ae0f56f3f2d18a4ffeb0e7b1b9c4dbd8d75`
- `docs/superpowers/specs/2026-10-06-full-opennurbs-design.md` SHA256 `0ce38801b6e7b2c824089c15fac39c5d0df6e1dffa0831e77aa3681a1345c56d`
- `docs/superpowers/plans/2026-10-06-3dm-merge-blocks.md` SHA256 `80f64695019c09586cda38c566ccfe890f3ee47601415340a33a8cbac71d1ffc`
- `docs/superpowers/plans/2026-10-07-3dm-point-cloud.md` SHA256 `e8e73478f7a4803993f2f65b42852d4baf0b95bb2dcee6b2a165e1aabc4340bb`
- `README.md` SHA256 `d25868e1628d9be726330117d49fbb1fdb1fd05bafb5436934900bcc567cba04`
- Isolated `OpenMatrix9Gui.pyd` SHA256 `1a667cca2e119536008799674de0505dae2e9e94939015f8fbcea9707e08f09c`
