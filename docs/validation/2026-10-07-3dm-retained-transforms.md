# Native retained geometry transforms

The development preservation route now exports current native TextDot and PointCloud transforms using an explicit `OM9BlockMemberPlacement` on retained host objects. Retained geometry without a CAD/Mesh payload is not manufactured into a fake editable shape. A selected top-level object composes its parent world Placement with this delta once, after native source unit normalization to mm. A canonical definition member uses the delta in definition-local mm; the selected root instance keeps its separate native matrix. Original source bytes, original payload signatures and canonical native UUIDs remain immutable. Payload changes still refuse; this feature changes placement, not arbitrary native text/point editing.

Rust action4 permits known non-instance native transform overlays and rejects incompatible/unknown capabilities, unknown references and instance payloads. The native writer validates a finite invertible affine4x4 matrix, strict overlay fields, reachability and userdata before staged export. Dependency transforms remain members rather than becoming selection roots. Copied output-member deltas follow the copy's local matrix; canonical source edits feed canonical-follow copies first. Native definition/reference bounds refresh after geometry transforms. The existing complete native serialized payload/attribute digests and semantic manifest reread checks remain required before destination replacement.

The pinned `opennurbs_pointcloud.cpp` implementation of `ON_PointCloud::Transform` transforms points/plane and invalidates bounds but leaves `m_N` unchanged. `Gui/ThreeDmTransforms.cpp` corrects normals with inverse transpose and preserves their original lengths (including zero vectors), colors and native user strings. This shared helper also covers source unit normalization and member copies. Tests use a non-axis normal of length2 under unequal X/Y scale, so a simple vector transform or forced unit normalization cannot satisfy the oracle. Normal array count, validity, invertibility and direction failures reject. Generic native Transform availability does not prove class-wide compatibility; existing class/reference/resource/version gates remain in force.

## Acceptance evidence

- RED: selected retained geometry lacked an explicit native Placement, and native action4 was rejected. An added GUI check then proved that moving a retained canonical definition member exported stale native coordinates. A native copied-member check exposed the wrong order between copy matrix and output-member delta.
- GREEN:25/25 focused GUI assertions cover native TextDot point/text and PointCloud bounds/counts; current metadata; canonical UUIDs and original baselines; selected closure; parent/child composition; repeated export; independent duplicate placement; Undo/Redo; included archive persistence after external input removal; definition-local member placement; block matrix separation; FCStd; payload mutation rejection; and cm-to-mm normalization.
- Native reread checks exact point arrays, non-axis inverse-transpose normals with original magnitude, per-point colors, geometry/attribute user strings, TextDot source text fields, mm/cm, native copied member class/closure/world coordinates, updated definition/reference bounds and atomic invalid matrix/field/unreachable-member refusal.
- The rigid GUI transform T(10,20,30)*Rz90 maps TextDot(5,6,7) to(4,25,37). A parent T(100,200,300) produces(104,225,337). The canonical block TextDot(4,5,6) becomes local(5,24,36), while the root instance remains T(10,20,30).
- Fresh isolated native9/9, final exit0,29.61s, including both immutable user ring files. Fresh isolated FreeCAD657/657 across35 GUI suites, each final owned process0, with stable Python/runtime/binary fingerprints throughout. Fresh Rust75/75 and cargo fmt --check exit0. General registered/native/menu/CMD routing is reverified by the shared29-check suite; the retained geometry focus uses the shared Python export route.

## Precise limits

These fixtures validate Rhino5 file reread through the pinned openNURBS library, not actual Rhino5 application acceptance. Native TextDot display and text editing, PointCloud display/per-point editing, all class-specific fields and arbitrary retained/plugin/userdata/resource semantics remain unverified. PointCloud runtime hidden flags are not serialized by openNURBS. Point intensities/newer fields are not covered by this slice.

Rhino5 TextDot serialization in this pinned writer does not retain nonempty secondary text. The test compares output to text fields actually present in the reread authoritative Rhino5 source; a separate Rhino6 fixture retains its secondary text and is rejected by the existing source-version gate before overwriting the destination. No silent Rhino6-to-Rhino5 secondary-text loss is accepted. Native `rhino5_write` class-wide coverage cells remain unverified; only the scoped file-write evidence below is recorded.

Selecting an existing native definition member alone still requires a separate selection/promotion policy. A promoted original top-level retained source selection now has verified direct native transform support. Top-level duplication allocates a distinct output UUID but this slice does not establish stable duplicate IDs across repeated exports. Broad legacy/script/plugin/array/subelement layouts, linked resources, full components/document/history/reference/version coverage, final package review/public integration and actual Rhino5 application acceptance remain pending. The full goal remains in progress. Source is build/om9-dev; isolated runtime is build/3dm-preservation-sdk.

## Reproduction

`rtk proxy cmd /c H:/FreeCAD-src/build/3dm-isolated-module.cmd`

`rtk proxy cmd /c H:/FreeCAD-src/build/3dm-preservation-regressions.cmd`

`rtk proxy H:/FreeCAD-src/.pixi/envs/default/python.exe H:/FreeCAD-src/build/run-retained-transform-regressions.py`

`rtk proxy cargo test --manifest-path H:/FreeCAD-src/build/om9-dev/rust/Cargo.toml`

Finish native fixture generation before GUI regressions. A timeout is not process completion. The initial misrouted macro launch used an absolute Macro argument that the helper prefixed again; its owned process was stopped after inspecting the malformed command line. The corrected basename launch supplies the actual RED/GREEN evidence.

|Suite|Checks|Result artifact|
|---|---|---|
|retained_transform|25/25|`build/three_dm_retained_transform_smoke-1/153705f7f56646328d4ce7d38fde6e58/results.json`|
|host_upgrade|39/39|`build/three_dm_host_upgrade_smoke-1/ab738c9231ca483eb58825b7aedf68ab/results.json`|
|shared_export_routes|29/29|`build/three_dm_shared_export_routes_smoke-1/e257e1a670be42ba995c42c4eaee2c3d/results.json`|
|recursive_shared_migration|24/24|`build/three_dm_recursive_shared_migration_smoke-1/5bc76beb00854e15a28a6554371559bf/results.json`|
|recursive_namespace_migration|25/25|`build/three_dm_recursive_namespace_migration_smoke-1/e777ffd2526140358fd0770640948dec/results.json`|
|affine_target_migration|32/32|`build/three_dm_affine_target_migration_smoke-1/265c4a7a536643a886347a0004dda700/results.json`|
|copy_origin_migration|22/22|`build/three_dm_copy_origin_migration_smoke-1/ed49577579b143d380968b8453c90fc4/results.json`|
|legacy_migration|35/35|`build/three_dm_legacy_migration_smoke-1/71375d753c2d4868b3439b1eca7524a9/results.json`|
|proxy_retarget|6/6|`build/three_dm_proxy_retarget_smoke-1/1389f0e343634cc9a7d6192f56a5d0d0/results.json`|
|source_proxy_creation|34/34|`build/three_dm_source_proxy_creation_smoke-1/3e6d9de8104b4de5965f36145e584b4a/results.json`|
|definition_copy|27/27|`build/three_dm_definition_copy_smoke-1/4940cc7ffc1a446e9b71a32d12d2a993/results.json`|
|source_members|23/23|`build/three_dm_source_members_smoke-1/d4424fb9ef184c4493b8ac909c38beb7/results.json`|
|source_member_geometry|8/8|`build/three_dm_source_member_geometry_smoke-1/6a082244fae54b2a904fc1d9c81d1eb5/results.json`|
|source_new_target|27/27|`build/three_dm_source_new_target_smoke-1/fe8900ed5d6e4935a70e321215657875/results.json`|
|new_blocks|29/29|`build/three_dm_new_blocks_smoke-1/f9498d065ac14597b1d3de430ebe7f94/results.json`|
|new_member_instance|20/20|`build/three_dm_new_member_instance_smoke-1/5d72c274ed1c4778b1340d15698ebfd6/results.json`|
|reference_edit|18/18|`build/three_dm_reference_edit_smoke-1/6cb542c66e0e41bf8adbe2aadaa86238/results.json`|
|independent_member|18/18|`build/three_dm_independent_member_smoke-1/24556760396d4c2cb2249bb806992f1c/results.json`|
|shared_proxy|10/10|`build/three_dm_shared_proxy_smoke-1/72d7f87b318149eea7532c6d3b5f5f81/results.json`|
|shared_proxy_geometry|8/8|`build/three_dm_shared_proxy_geometry_smoke-1/3c544ea6988c453380ac7ff284a327fd/results.json`|
|retained_member|6/6|`build/three_dm_retained_member_smoke-1/9068fe458d884255b63b70bc87fc90cc/results.json`|
|definition_placement|12/12|`build/three_dm_definition_placement_smoke-1/d9d937e6ff27444c99cba888957c2c15/results.json`|
|new_geometry|20/20|`build/three_dm_new_geometry_smoke-1/a82530616e5a42428635adf696b6ef01/results.json`|
|preserved_export|13/13|`build/three_dm_preserved_export_smoke-1/0959667ef2d3478a8bc02559e183290b/results.json`|
|member_overlay|12/12|`build/three_dm_member_overlay_smoke-1/921e93ab2b8747bfb938ba326dd34c67/results.json`|
|block_structure|23/23|`build/three_dm_block_structure_smoke-1/ca42ae3c80664d8fb144e4159f83c93e/results.json`|
|affine_refresh|10/10|`build/three_dm_affine_refresh_smoke-1/c9f3eb8e6cf643e0875352045aebf7c0/results.json`|
|affine_refresh_reopen|2/2|`build/three_dm_affine_refresh_reopen_smoke-1/cc96084df3e84807b6dda5a4c62fe579/results.json`|
|source_signature|14/14|`build/three_dm_source_signature_smoke-1/0ddb79a2ff9744b5885456e97e72b37f/results.json`|
|preservation|19/19|`build/three_dm_preservation_smoke-1/3aee2ddcd99a4a31bffa7ca49032b70f/results.json`|
|definition_overlay|8/8|`build/three_dm_definition_overlay_smoke-1/7b3dcc4362504aa8a81539c397ee7554/results.json`|
|layer_overlay|7/7|`build/three_dm_layer_overlay_smoke-1/09093edf95644f6d8f10a723a043153f/results.json`|
|mixed_shear|11/11|`build/three_dm_mixed_shear_smoke-1/5f8dba11beb248bca3f58d3f0e9b9bcf/results.json`|
|merge_bridge|8/8|`build/three_dm_merge_bridge_smoke-1/75cc4275b8444560be7b50495744ba6a/results.json`|
|geometry|33/33|`build/three_dm_smoke-1/814e91141e0443f99beb2cbf2c3d956d/results.json`|

## Fingerprints

- `ThreeDm.py` SHA256 `65b837a9ff628284b74a370f4fe7c8bf1248289412fa38d0d533ff58973b186d`
- `ThreeDmArchiveState.py` SHA256 `3ead15cf7330dd789deb2072b614db658fd5a068b6d07ddd28f88061bb4d9711`
- `ThreeDmMigration.py` SHA256 `86f573e4090b1d80d76627dff849fb742e7f1cebed5ba59346056853f695a9f5`
- `ThreeDmUpgrade.py` SHA256 `5fcf9e048f914175136dd489709e82e22adafc512d9369a6773aa701da3d25a6`
- `InitGui.py` SHA256 `2d920887a898a94f90eef120bdc14f989a96567eb8094e66f6b6cd022389f157`
- `Gui/ThreeDmArchive.h` SHA256 `9f4b496a61acae9f7df6d85d7b0612f6e30fd421d9f0e910cd58a3b55e84f555`
- `Gui/ThreeDmTransforms.cpp` SHA256 `125ceabec9755a45c451c6372c054f81418719fb2c3572374dbd43d9dc971789`
- `Gui/ThreeDmInventory.h` SHA256 `233eee17de92df63343dc7d0cff79287ce92afc4f5075ab8043da82400daa3e8`
- `Gui/ThreeDmInventory.cpp` SHA256 `6772d09065a0ddc1dc3c4e59a5f762bb0d5afc6c717c01b2e42b28c5de326fcf`
- `Gui/ThreeDmMerge.cpp` SHA256 `acecd1d47e1822c8624a4282878c9c0ad82620ea68184b16e45632d2b61ac267`
- `cmake/OpenNURBS.cmake` SHA256 `5a3750836bd8b66900002d0d7d55a1e0bb51ad65d1ee72544b7ac262f87ffe67`
- `rust/src/core_3dm_archive.rs` SHA256 `1255d809ccdfca6d915267b0fc5bfe5536cd5f4a83069319bc57e04dd8deadfe`
- `rust/tests/core_3dm_archive.rs` SHA256 `6e0496621c45d67c97ba615d848876f2a7384db20e32c184f00cf611cfc03a66`
- `tests/native/CMakeLists.txt` SHA256 `ccbf0bdaace873fe1897cdf2bb40d9dd20b261f1d58ad0e7b89e7785fe7c0f7a`
- `tests/native/three_dm_native_transform.cpp` SHA256 `5e22f788c163e49299b5ddb58de1d31a3b5b426b4a5fbf4beacb91939b473ac4`
- `tests/three_dm_retained_transform_smoke.FCMacro` SHA256 `0d48580749ff7634a2c26e6dd26e6e71789cb0eeadc811694627a4826ba0903b`
- `tests/three_dm_source_members_smoke.FCMacro` SHA256 `dc52848f56cc960fa82bd409c4b66e05481c8d42efab492a2f17304f57d7394a`
- `docs/3dm-coverage.json` SHA256 `18ea555b73b364e972f92c246688080371d8cb422f0874a794b7e5fd406b9be7`
- `docs/development/3dm-structural-blocks.md` SHA256 `b73da91926a6feacea6dec7778e6bf78fd297e4f2bfda20ffcc811a4c37e0bcb`
- `docs/features/OM9-FILE-012.md` SHA256 `2b80437e8e004e7e0b62a20cf9ca19c4799269a528405a08f40cdc4bb77e902c`
- `docs/features/3dm-support-matrix.md` SHA256 `bf742bd8adad3bf2fa99aaccb7c97a8b43756bf06d7ccda8030e30d31390bcaa`
- `ref/matrix9/OpenMatrix9_Codex_Spec_v1/specs/01-core/om9-file-012-rhino5-3dm-exchange.md` SHA256 `72e2cc79de1b3028cabc47025c7b06ed51910b4db93c8107c17d4cb64a2e7ed8`
- `ref/matrix9/OpenMatrix9_Codex_Spec_v1/MANIFEST.json` SHA256 `b3e1347368fdb8b9506c7867545d0494b28e2575ad6a51a6816773cfe7e7a71d`
- `docs/superpowers/specs/2026-10-06-full-opennurbs-design.md` SHA256 `0aef69abed0df917dd6af0782f75e8f19363182ef15168d16def74dbe1fab7ca`
- `docs/superpowers/plans/2026-10-06-3dm-merge-blocks.md` SHA256 `80638c7955366b433b31e6d3491a2419d86a1956d57794c4b0a5c621b1ec475c`
- `README.md` SHA256 `3ca745892f2ff48392f3f35df6bb9892a9b375adc6bba679d8eb53055ab2af9a`
- Isolated `OpenMatrix9Gui.pyd` SHA256 `eab47f4f72bf3bdbbbe2436d500c12463cda2f7e6112a8e77a8ced6f6b0fe3ff`
