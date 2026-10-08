# PointCloud geometry-only, affine preview and independent legacy5 evidence

## Current verified scope

Geometry-only import now represents ON_PointCloud as independent double FeaturePython fields with a derived colored Coin PointSet. Native ExchangeItem has a typed ON_PointCloud alternative alongside CAD and mesh. Direct current cloud export and mixed CAD/mesh/cloud selections stage complete SHA256 field files; they never read float display coordinates. Physical parent and native member placement compose once. Source-backed cloud/member/block selections first build the verified current native graph, then flatten it under the explicit geometry-only policy. Geometry-only writes new attributes and intentionally omit source definitions/tables/history/userdata. Preservation export retains those supported native source fields as before.

Nested affine/reflected/nonuniform instances expand cloned native clouds, apply inverse-transpose normals while preserving original length and zero, transform active/valid dormant plane, then normalize units to mm. Optional normal/RGBA/value arrays and ordered/plane state remain exact native fields. Typed cloud leaves allow mixed affine previews; canonical current-field changes schedule automatic refresh. Preview fingerprints include the complete double cloud fields and native local placement, and their integrity is required before export. Current native source fields are export authority; edited derived preview fields or direct selection of a derived child are rejected. Preview and current graph survive FCStd independently of original external paths.

Complete cloud files are bounded to512MiB each and in aggregate before host binding/native geometry writing. Native parsing validates schema, exact field keys/types/counts/hash/class, finite numeric data and affine nonsingular transforms. The writer validates3 RGB integer bytes when supplied and uses its default RGB when absent. This removes an unchecked empty-array access found by a malformed-manifest test. All cloud target checks precede destination writes; host export still stages beside the destination and replaces atomically after reread. The native geometry-only writer compares complete native PointCloud fields after reread, not only object count.

## Independent Rhino5-era reader evidence

Primary SDK: [McNeel openNURBS20130711](https://files.mcneel.com/opennurbs/5.0/2013-07-11/opennurbs_20130711.zip),1807257bytes, SHA256 `5cb9ff879c94c63145526a188f9ddc8c522f35cd82d8726ad49ed64a63d6e05a`. Every extracted source file was compared byte-for-byte with the archive. Separate `tests/native/rhino5_reader` builds unchanged old sources and their bundled zlib; no modern ON_* symbols/classes are linked into that executable. Build settings only: MSVC `/UWIN32` for old x64 headers and `/Od` for opennurbs_lookup.cpp because the modern optimizer hit an internal compiler error.

The executable verifies ON::Version201307115, has a compile-time assertion that native ON_PointCloud has no m_V, and reads/writes modern Rhino5-header fixtures. Modern native inspection of that independent reader's output proves exact points, nonzero/zero normals, RGBA including alpha, plane, ordered/reserved flags, UUID, units, native and attribute user text survive for both mm and cm. Nonempty intensity is lost. Legacy PointCloud uses chunk1.1; the pinned modern writer uses chunk1.2. A modern writer rereading its own Rhino5-header file cannot establish older-reader compatibility.

The shared preservation/geometry-only gate now states this precise baseline: nonempty current intensity is rejected before output replacement. Import/edit/FCStd retain intensity; explicit current clearing permits export and leaves the included source immutable. This is a documented incompatibility with the2013 Rhino5-era reader, not a claim that every later Rhino5 service release behaves identically.

Installed Rhino5.14 and RhinoCommon5.14.00522.08390 were detected, but actual application acceptance is unverified. The first owned launcher encountered the configured RunAsAdmin UAC before receiving a Rhino PID. A current-user RunAsInvoker attempt obtained PID45112, then exited3 without fixture results. Its terminal result is recorded separately; no license/elevation/application acceptance inference is made. Existing user Rhino PID62736 was not touched, and permanent compatibility settings were not changed. The optional application harness remains available when a usable runtime can execute it.

## Verification

- RED: old non-preserving import refused ON_PointCloud inside the mixed nested affine fixture. Initial GUI binding then exposed FeaturePython lacking ShapeColor/Selectable; only supported view properties are assigned.
- A strengthened malformed-native-manifest test stalled before its first hash rejection. Per-check progress located the empty RGB array access in the binding. Owned stalled test PIDs were verified and stopped; the final corrected test completed successfully. These timed-out/terminated runs are not passing evidence.
- Native analytical checks independently verify transformed point `(88.75308642197531,266,432)`, nonprincipal normal `(-2,1,0)`, plane origin `(80,320,540)`, mm/cm scaling, mixed CAD/mesh/cloud types, exact full cloud reread and userdata omission. Independent SDK201307115 interop is a separate executable/test.
- Fresh isolated FreeCAD817/817 across39 suites, each final owned process exit0 with stable source/runtime/binary fingerprints. The new50-check suite covers both units, current fields, mixed export, physical placement, intensity retention/rejection, FCStd, automatic affine refresh, full preview/native equality, root delta, copied definitions under reflection/nonuniform scale, preview tamper/direct-child rejection and malformed hash/matrix/type/color atomic guards.
- Native13/13,31.87s, exit0, includes both immutable user ring fixtures. Fixture generation completed before the final GUI run. Rust source/tests are unchanged at previous75/75/fmt fingerprints; no fresh Rust test claim.

## Remaining scope

Full openNURBS remains in_progress. Generic recursive FreeCAD copy can change derived child labels and therefore its preview fingerprint; that raw copy is rejected until an explicit verified rebuild/migration handles it. OM9 copy_definition with current native cloud fields and a placed reflected/nonuniform copied graph is verified. Arbitrary legacy cloud/preview upgrades and mixed source-backed plus unsourced cloud combinations are not established by this package.

Large clouds near the512MiB limits still need streaming/memory/performance work; current JSON and Coin arrays allocate complete data. Local hidden points are FreeCAD display state and are not serialized in3DM. FeaturePython has no standard Selectable property; lock metadata survives exchange, while PointCloud viewport locking behavior needs separate work. Geometry-only omits source-only reserved flag provenance/userdata by its explicit contract. Exact Rhino appearance and actual Rhino5 application acceptance remain unverified.

Remaining annotations/hatches/styles, clipping/layout/other geometry, appearance/resources/components/document/history/reference/version and legacy categories require the full128-class/16-component/6-document-category audit. Final comprehensive review and public-source integration remain pending. Development source is build/om9-dev; the verified runtime is build/3dm-preservation-sdk. An initial build command also refreshed generated module output in build/relWithDebInfo before switching back to the isolated target; that normal-build output is not the validated integration artifact. No public source checkout was modified by this package.

## Reproduction

`rtk proxy H:/FreeCAD-src/.pixi/envs/default/python.exe H:/FreeCAD-src/build/fetch-opennurbs5-sdk.py`

`rtk proxy cmd /c H:/FreeCAD-src/build/3dm-rhino5-reader.cmd`

`rtk proxy cmd /c H:/FreeCAD-src/build/3dm-legacy-cloud-tests.cmd`

`rtk proxy cmd /c H:/FreeCAD-src/build/3dm-isolated-module.cmd`

`rtk proxy cmd /c H:/FreeCAD-src/build/3dm-preservation-regressions.cmd`

`rtk proxy H:/FreeCAD-src/.pixi/envs/default/python.exe H:/FreeCAD-src/build/run-cloud-geometry-regressions.py`

|Suite|Checks|Result artifact|
|---|---|---|
|cloud_geometry|50/50|`build/three_dm_cloud_geometry_smoke-1/3b3d7c12030b45069f526898084a3ee3/results.json`|
|point_cloud|43/43|`build/three_dm_point_cloud_smoke-1/cebc0168dc9d44998b8b0430f37a9eb4/results.json`|
|text_dot|34/34|`build/three_dm_text_dot_smoke-1/7136ea1e2bd84a039a86b376ed39fdff/results.json`|
|selected_member|33/33|`build/three_dm_selected_member_smoke-1/92426d4a445648da9d7d878aa00730b5/results.json`|
|retained_transform|25/25|`build/three_dm_retained_transform_smoke-1/19888d88280a476c89e8c0c86abb73bb/results.json`|
|host_upgrade|39/39|`build/three_dm_host_upgrade_smoke-1/74fd8a70208646bdb84e22d97b2109b1/results.json`|
|shared_export_routes|29/29|`build/three_dm_shared_export_routes_smoke-1/0561363669494591afc866f6fa93b739/results.json`|
|recursive_shared_migration|24/24|`build/three_dm_recursive_shared_migration_smoke-1/b17671d2349348c5a92fc79ef755c276/results.json`|
|recursive_namespace_migration|25/25|`build/three_dm_recursive_namespace_migration_smoke-1/97fe3de7aeaf4e84938fb4846f20f46e/results.json`|
|affine_target_migration|32/32|`build/three_dm_affine_target_migration_smoke-1/0cd16afafe7947728acfbd3847d8a90e/results.json`|
|copy_origin_migration|22/22|`build/three_dm_copy_origin_migration_smoke-1/11801776ed154b29be867ad84359fa40/results.json`|
|legacy_migration|35/35|`build/three_dm_legacy_migration_smoke-1/566dca180c424d63af65fd0b29c474ae/results.json`|
|proxy_retarget|6/6|`build/three_dm_proxy_retarget_smoke-1/a0c3b037ff994b529a9e3ea360eb652f/results.json`|
|source_proxy_creation|34/34|`build/three_dm_source_proxy_creation_smoke-1/65478d38d15a43a284d4f9531a09f81b/results.json`|
|definition_copy|27/27|`build/three_dm_definition_copy_smoke-1/72febeb0469d44d6a38bdd46c3603719/results.json`|
|source_members|23/23|`build/three_dm_source_members_smoke-1/0e833453173d401b8f3e72be62a6b477/results.json`|
|source_member_geometry|8/8|`build/three_dm_source_member_geometry_smoke-1/510ed9352fe845a2b8b0cdfbb5f2ab6f/results.json`|
|source_new_target|27/27|`build/three_dm_source_new_target_smoke-1/05b39f0843ca42e580a5433bb49581de/results.json`|
|new_blocks|29/29|`build/three_dm_new_blocks_smoke-1/84bf1cda6a09481aaac812dbb2f5cb52/results.json`|
|new_member_instance|20/20|`build/three_dm_new_member_instance_smoke-1/0b8ca5ea7b9f4ac0919e4b6fad35f248/results.json`|
|reference_edit|18/18|`build/three_dm_reference_edit_smoke-1/586dfe5ed6ca4154a3c97c900abbace0/results.json`|
|independent_member|18/18|`build/three_dm_independent_member_smoke-1/49e3ed200c86438ea685a42d1283b0d8/results.json`|
|shared_proxy|10/10|`build/three_dm_shared_proxy_smoke-1/5ed1575bd43841f8bfe8ae5ee5868c8c/results.json`|
|shared_proxy_geometry|8/8|`build/three_dm_shared_proxy_geometry_smoke-1/567827734d8e49e7b6a562811468a91f/results.json`|
|retained_member|6/6|`build/three_dm_retained_member_smoke-1/40f8a8b270fc487fa01f045ce1bf8bf3/results.json`|
|definition_placement|12/12|`build/three_dm_definition_placement_smoke-1/335a58902bfb459bb1ead9d29c2c0350/results.json`|
|new_geometry|20/20|`build/three_dm_new_geometry_smoke-1/94d22192bae546bcbc53a0bf3f952a59/results.json`|
|preserved_export|13/13|`build/three_dm_preserved_export_smoke-1/d595d37bbd2742a3821a6a7ff62b3adf/results.json`|
|member_overlay|12/12|`build/three_dm_member_overlay_smoke-1/ec4bd3e7341f413dbfbd917409f97254/results.json`|
|block_structure|23/23|`build/three_dm_block_structure_smoke-1/e1daf2058be344078ca565f862f0f79d/results.json`|
|affine_refresh|10/10|`build/three_dm_affine_refresh_smoke-1/6d184c6e1a65411fa7902a1dae831c83/results.json`|
|affine_refresh_reopen|2/2|`build/three_dm_affine_refresh_reopen_smoke-1/4fb62e87bc864e6c98cde9b7a5611fe4/results.json`|
|source_signature|14/14|`build/three_dm_source_signature_smoke-1/7869114f0cbc45549c4002ce615b5441/results.json`|
|preservation|19/19|`build/three_dm_preservation_smoke-1/511067ff12084a3a8376952b8679b72f/results.json`|
|definition_overlay|8/8|`build/three_dm_definition_overlay_smoke-1/79de5b4f819d4009b3ec0e90eeac9f16/results.json`|
|layer_overlay|7/7|`build/three_dm_layer_overlay_smoke-1/ae0225f3de0343e3a1d1c69f71b1b22b/results.json`|
|mixed_shear|11/11|`build/three_dm_mixed_shear_smoke-1/a5afc33fa0634c2ab3fa326d34a6dba0/results.json`|
|merge_bridge|8/8|`build/three_dm_merge_bridge_smoke-1/cdadc15a1e984c4ca82d8e2f2124a4e9/results.json`|
|geometry|33/33|`build/three_dm_smoke-1/a52e3d47478a4bb8bafa6f44d6f2461b/results.json`|

Checkpoint: `H:\FreeCAD-src\build\checkpoints\cloud-geometry-legacy5-20261007-045921.zip`; all saved files are independently SHA256 verified.

## Current fingerprints

- `ThreeDm.py` SHA256 `449e11f46847c0573b947324925967a0d808c02aca10b75dcc754ee67eba6bba`
- `ThreeDmArchiveState.py` SHA256 `19d15ba70ae081c305273dd699204dbcbe07c966883ac44101c6cda63e9915e6`
- `ThreeDmMigration.py` SHA256 `146cae509edb79f790fe5b8f71a3a7aadde02853c440ce205d50fccec54f2622`
- `ThreeDmPointCloud.py` SHA256 `daa37635ab67b2d2ff38fdea15e9e553d27054b776a8f9b71de7b7a38b48cce6`
- `ThreeDmTextDot.py` SHA256 `d345d8c5f4e58a7270e622f88fed28ce94f9856137d0c5b50c2b16575a544537`
- `ThreeDmNativeFields.py` SHA256 `b7000a7d672e2f99296b8480e1edc5eadea9984b8226dfde6b1204aea1f0b21d`
- `ThreeDmUpgrade.py` SHA256 `5fcf9e048f914175136dd489709e82e22adafc512d9369a6773aa701da3d25a6`
- `InitGui.py` SHA256 `2d920887a898a94f90eef120bdc14f989a96567eb8094e66f6b6cd022389f157`
- `Gui/ThreeDmArchive.h` SHA256 `3a5674de39aefcff78bb508d1b6cc444cd6c06b4202f9d2323e3790d98a5509b`
- `Gui/ThreeDmArchive.cpp` SHA256 `a05a08e86cd861650e77cde6f4df73f87f580867444db5bb2488a6909bca003e`
- `Gui/ThreeDmPointCloud.h` SHA256 `c6c9be6101f2ae2955c36b92f9de0d2c7aebd66d0797ea1eae0da02b7b81e3c1`
- `Gui/ThreeDmPointCloud.cpp` SHA256 `ab67dc2807b49d79a45a38f2aead125bdfdc28a474ab70912ed5c091ab4cbab2`
- `Gui/ThreeDmPython.cpp` SHA256 `d690640a46e74e948c38d381a3c6a317f2504d574e0d386a941489d9e718b11d`
- `Gui/ThreeDmMerge.cpp` SHA256 `23e6206ed85eafb596b43423c2d6e9adeba21fe19a03e51361f961ddb12b4314`
- `tests/native/CMakeLists.txt` SHA256 `74f57b54c0436185e9ad5ffecc5aa722999943f820b6a5a07d76321a8563a13c`
- `tests/native/three_dm_cloud_geometry.cpp` SHA256 `3e9bae3f44e0504eac3a7ef4b7956273f5a3bcf625ba46ef59aa05d487d9827e`
- `tests/native/three_dm_legacy_cloud.cpp` SHA256 `437b2d85508593f552eb085a2dd90f6782848b52d1ec38797fa4f35852434299`
- `tests/native/rhino5_reader/CMakeLists.txt` SHA256 `7d62d334248eb3cad0f3a748f01b2f4549fd8ab6fc3e3e49ece9fed00a2d68f3`
- `tests/native/rhino5_reader/point_cloud_reader.cpp` SHA256 `793e1c48a1a2ab64b111acaf4087020c2c7c99580f4c14cdac5fe42385311f3c`
- `tests/rhino5_point_cloud_interop.py` SHA256 `f08ca1f312fcf2de8b0e3f0072412dfb8ada5ed5cfb0a8519a55e0b807101665`
- `tests/three_dm_cloud_geometry_smoke.FCMacro` SHA256 `94d8c96dbfb7316f6ffb3c1dbb90d6674b00b45bea5cfad25d731b005f0364df`
- `docs/3dm-coverage.json` SHA256 `1eb09145ed55c46efaf85985eb858e3b89dce861b914f16ba99ba461ee08bc99`
- `README.md` SHA256 `7c77c6f022ecfddd843e03de3c51c62d21afd7d8b6ed40e301b1cce028aec834`
- `ref/matrix9/OpenMatrix9_Codex_Spec_v1/MANIFEST.json` SHA256 `37c484de601a7da540efae2f755bcf3299db1ed46ff2fb6a67305376864368c6`
- `docs/features/OM9-FILE-012.md` SHA256 `3cb09b8eee46967f5184db2e9f7d3250b813707255aa0d322be0129fd4388b55`
- `docs/features/3dm-support-matrix.md` SHA256 `50665f94749688c74d0ffa16cb4ba97f54c55f7ab4fe951b9184b387ec3b9738`
- `docs/development/3dm-structural-blocks.md` SHA256 `22fb504b605b3a81201c5e89fcea888be7d140f88e9380883913a5e651b0e4aa`
- `ref/matrix9/OpenMatrix9_Codex_Spec_v1/specs/01-core/om9-file-012-rhino5-3dm-exchange.md` SHA256 `7aac5c88882bfa089655f3162811278bcf5a11703baf5944bbc67e78df6aae56`
- `docs/superpowers/specs/2026-10-06-full-opennurbs-design.md` SHA256 `08d2f2cbb271a444659ae8612c0a618cc842563ca74ebaed0729afee944bb994`
- `docs/superpowers/plans/2026-10-06-3dm-merge-blocks.md` SHA256 `254574c4cbc64df4acf09c0515a41ce4faf2dea876fae8ff2f6878d46a8c20d1`
- `docs/superpowers/plans/2026-10-07-3dm-point-cloud.md` SHA256 `f08999970de7d8136b4b3a291d5fdb67bff269bcabe3389f35572b5ed00e713a`
- `docs/superpowers/plans/2026-10-07-3dm-point-cloud-completion.md` SHA256 `7c9e6fb8993477cfc3ec1e1fcf72218759646a1131cf89f47121f2b161c1f4cf`
- Isolated `OpenMatrix9Gui.pyd` SHA256 `69d7d444e5f49ec5eb41e1d02d9ddd6003d5794c8275a55e707b896b9432c59a`
- Independent `Rhino5PointCloudReader.exe` SHA256 `f58eacca3fdc2528d289fced99ffa1b419c1e3d89b9cac360764392c72c0567e`
