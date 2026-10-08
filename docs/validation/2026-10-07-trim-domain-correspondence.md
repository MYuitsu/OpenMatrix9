# OM9-FILE-012 — native curved trim domain correspondence

The native resolver validates the endpoints of each referenced trim portion by
evaluating its C2 proxy on the actual owning surface and comparing against its
edge subdomain endpoints. Trim m_bRev3d determines endpoint pairing. Evaluation
uses source units, absolute model and edge tolerances, plus64epsilon coordinate
roundoff. Wrong in-bounds trim intervals now produce structured invalid-reference
issues and fail import before host mutation. Exact source schema/model stays
deferred and immutable; the detached linked graph retains its source lifetime.

## Fresh verified scope

-32 real native quadratic NURBS face cases: planar/nonplanar, independently
  reversed C3/proxy pairs, topological edge reversal/trim m_bRev3d, segment reversal
  and correct/wrong trim intervals.16 valid,16 invalid. Four distinct domains:
  C3=0..1, edge=11..23, trim/C2=101..149, evaluation=31..47. Referenced edge=14..20,
  matching trim=113..137; incorrect full trim=101..149 is within bounds but rejected.
-17 native samples per valid path compare against the independent equation
  (2u,2u(1-u),4u(1-u)) for the curved face, z=0 for planar. Edge parameters and
  native direction helpers agree1e-12. Lifetime persists after inventory release.
-5 tolerance cases use a0.001 trim parameter shift: model tolerance0.01 admits,
  model1e-8 rejects, edge0.001 admits, negative/NaN source tolerances reject.
  Source and effective tolerance, endpoint UV/3D/parameters/deviation are exposed.
-176 new FreeCAD checks:16 invalid imports atomic,16 positive source/native
  records, retained/no false Shape, source deletion/FCStd/reconstructed analysis,
  pending selected export atomicity and immutable fixtures.
- Complete fresh native30/30 and GUI1901/1901 across57 suites, every process0.
  Both supplied rings pass. Native fixture generation completed before GUI.
  Runtime9 scripts2 binaries2 separate readers stable; original modern and280
  SDK2013 files/pristine library/generated repairs guarded. App/Rust unchanged,
  their earlier results remain historical. No SDK repair was added here.

## Regression evidence

Native RED: a native-valid reference with a wrong in-bounds trim portion reported
linked; the test failed before production validation. Host RED against the older
module failed on the first wrong-trim diagnosis. Both reports are checkpointed.

## Remaining requirements and limits

Endpoint agreement is a necessary check, not proof of the interior trim/edge
correspondence. Diagnostics explicitly say endpoints_verified and interior_mapping
unverified. Arbitrary nonlinear parameter maps, self-intersections/periodic seams,
singular trims without3D edges, full CurveOnSurface approximation/UV agreement,
current host display/edit/remap/selected export and independent target/Rhino5
application acceptance still need complete adapters and proofs.

An attempted independent reversed C2/proxy pair exposed a pinned Brep IsValid
assumption: its loop check evaluates raw m_C2 at trim.Domain endpoints rather than
the trim proxy (opennurbs_brep.cpp around3618). These cases were invalid fixtures
under the current SDK oracle, not passing acceptance cases; no source or SDK guard
was bypassed. C2 proxy reversal/validity requires a separate audit. Valid topology
tests here reverse the native edge and update connected m_bRev3d via native APIs.
This limitation is still open in the comprehensive128/16/6 coverage.

Tolerance evidence applies to declared source model/edge tolerances and roundoff;
it does not certify the entire Brep or provide a geometric error bound over its
interior. Shared cloning remains SDK-SizeOf-accounted rather than a heap/RSS bound.
Existing source snapshots, unavailable-class diagnostics and selected export gate
stay active. No full-class completion or actual Rhino5 interoperability claim.
All128 classes/16 component/6 document categories, plugin/history/resource/version
semantics and final review/integration remain in_progress.

## GUI evidence

|Suite|Checks|Artifact|
|---|---|---|
|trim_domains|176/176|build\three_dm_trim_domains_smoke-1\d27f0093965441ada65749a2fe2582b2|
|reference_graph_integrity|38/38|build\three_dm_reference_graph_integrity_smoke-1\47a3344cf3454c5ebeffbf0c35366134|
|native_reference_resolution|88/88|build\three_dm_native_reference_resolution_smoke-1\c46df9807618413fae04c88ec9de4d22|
|curve_on_surface_schema|120/120|build\three_dm_curve_on_surface_schema_smoke-1\454087b0d6ec4a6b97378a878035b5b4|
|curve_on_surface|32/32|build\three_dm_curve_on_surface_smoke-1\b0171ce0c16b4c6c9a34abcbcb12077f|
|hatch_loop_rows|103/103|build\three_dm_hatch_loop_rows_smoke-1\08bc263876df48a598870edccca266fe|
|hatch_loop_ui|104/104|build\three_dm_hatch_loop_ui_smoke-1\ecc425ddebb0485ea4cbfadbcc65e816|
|hatch_legacy_loops|84/84|build\three_dm_hatch_legacy_loops_smoke-1\9d4cf28fb2a646fcbee41f479ff06693|
|hatch_loop_baseline|4/4|build\three_dm_hatch_loop_baseline_smoke-1\463e65550c2a4950bc62efa80b73acad|
|included_file_copy|8/8|build\three_dm_included_file_copy_smoke-1\7d028211a7ee4c39bb858ae3ed9a842d|
|hatch_loop_edit|132/132|build\three_dm_hatch_loop_edit_smoke-1\9a77c459dfc647e2aed69b3b6ecc09b4|
|hatch_shared_loop_edit|12/12|build\three_dm_hatch_shared_loop_edit_smoke-1\5c84db0f863442c6a21ecb6bd7e7b627|
|hatch_loops|54/54|build\three_dm_hatch_loops_smoke-1\a4381e781f12465c83aa70288f0402d1|
|hatch_legacy|24/24|build\three_dm_hatch_legacy_smoke-1\468a0481d82443b78f0335c75d4220e5|
|hatch_current|48/48|build\three_dm_hatch_current_smoke-1\7423b06222f44300be3e9e54bf33dde1|
|hatch_shared_current|16/16|build\three_dm_hatch_shared_current_smoke-1\dc3d7284355e470ba7b89cd3189639e5|
|hatch_affine|24/24|build\three_dm_hatch_affine_smoke-1\22ac8b910e014dd7b919e00069f162bc|
|hatch|16/16|build\three_dm_hatch_smoke-1\09da94cd9ac346e3b11c1f8d0ec06a96|
|cloud_geometry|50/50|build\three_dm_cloud_geometry_smoke-1\095c7c18ba9949f79b3b868a84cbe832|
|point_cloud|43/43|build\three_dm_point_cloud_smoke-1\20c553a482474ba5a483e4d8021847b4|
|text_dot|34/34|build\three_dm_text_dot_smoke-1\dfdb428e851142d2aded5a3bcff83684|
|selected_member|33/33|build\three_dm_selected_member_smoke-1\42ba5c3f48614e4f9c5cde3737e5847d|
|retained_transform|25/25|build\three_dm_retained_transform_smoke-1\7628647b59004902a0b3655d16f98a5c|
|host_upgrade|39/39|build\three_dm_host_upgrade_smoke-1\1118ecbebe7a4cd68e02ffafdcc28f86|
|shared_export_routes|30/30|build\three_dm_shared_export_routes_smoke-1\ddd92627aea743128b0dcd6a790589ed|
|recursive_shared_migration|24/24|build\three_dm_recursive_shared_migration_smoke-1\ea0781f780f94463bd40c0134b98e92b|
|recursive_namespace_migration|25/25|build\three_dm_recursive_namespace_migration_smoke-1\92d3d132ff93416fbae1841db9ad10db|
|affine_target_migration|32/32|build\three_dm_affine_target_migration_smoke-1\3d8cf7bad5d048c2b2a992dd19f514b2|
|copy_origin_migration|22/22|build\three_dm_copy_origin_migration_smoke-1\3b2d456d64fa4d888ef72afb5677cae2|
|legacy_migration|35/35|build\three_dm_legacy_migration_smoke-1\3c5c1c227c564fa2a13accc461da8b73|
|proxy_retarget|6/6|build\three_dm_proxy_retarget_smoke-1\c8b8ff5c774141f9bacd0b146b5dd8af|
|source_proxy_creation|34/34|build\three_dm_source_proxy_creation_smoke-1\08bc0f1926eb4ab4b2f9e713ec302205|
|definition_copy|27/27|build\three_dm_definition_copy_smoke-1\05d19855efc34ce9a20ff524a0699161|
|source_members|23/23|build\three_dm_source_members_smoke-1\3f85b4fafde14903b208b0464f87d86e|
|source_member_geometry|8/8|build\three_dm_source_member_geometry_smoke-1\5f21ac1a7c3843a4bdb15b311056539a|
|source_new_target|27/27|build\three_dm_source_new_target_smoke-1\63f4150cd0fe4fa5b4b4bd4102a6a725|
|new_blocks|29/29|build\three_dm_new_blocks_smoke-1\8ee6ac20d0b84670b425fac959d2564a|
|new_member_instance|20/20|build\three_dm_new_member_instance_smoke-1\039d0cdf1b8f429a9c0e4e4d73b846e2|
|reference_edit|18/18|build\three_dm_reference_edit_smoke-1\8f6a1b0dc51347879bd633529e01f60c|
|independent_member|18/18|build\three_dm_independent_member_smoke-1\a8e234ad7b6b44bd9a00e39191567d9b|
|shared_proxy|10/10|build\three_dm_shared_proxy_smoke-1\bf30893a04c2404f8009be4c78d36974|
|shared_proxy_geometry|8/8|build\three_dm_shared_proxy_geometry_smoke-1\0952374381ef45088d7bdd7d74e2ca2a|
|retained_member|6/6|build\three_dm_retained_member_smoke-1\bb89c10c27c94fedb71ef79bb2aa70af|
|definition_placement|12/12|build\three_dm_definition_placement_smoke-1\9f42671e525445dba3b444e9febcd849|
|new_geometry|20/20|build\three_dm_new_geometry_smoke-1\b44562ec2e2b46b9b255d329a89b75a8|
|preserved_export|13/13|build\three_dm_preserved_export_smoke-1\200692e937ca4c479184e836d93543ef|
|member_overlay|12/12|build\three_dm_member_overlay_smoke-1\91e595307e5d494cbfebf8c65c8cba5e|
|block_structure|23/23|build\three_dm_block_structure_smoke-1\1865236979c8419a9e59056fb48487ab|
|affine_refresh|10/10|build\three_dm_affine_refresh_smoke-1\6c2692d10e784924917e9bff8467bb65|
|affine_refresh_reopen|2/2|build\three_dm_affine_refresh_reopen_smoke-1\36b03775748540f78b1cd5c47b643d93|
|source_signature|14/14|build\three_dm_source_signature_smoke-1\c838a576b65b4afc9fc378e92a5fc10e|
|preservation|19/19|build\three_dm_preservation_smoke-1\403ceb98ef24490993f0147ab97f453d|
|definition_overlay|8/8|build\three_dm_definition_overlay_smoke-1\648512090c6246b085867333a461666c|
|layer_overlay|7/7|build\three_dm_layer_overlay_smoke-1\8044632432ef4bf0b27627ef98169004|
|mixed_shear|11/11|build\three_dm_mixed_shear_smoke-1\0aa3a0d3786d4e5183f1934d50d92c32|
|merge_bridge|8/8|build\three_dm_merge_bridge_smoke-1\44286e4a56f049938374a591dbb26591|
|geometry|33/33|build\three_dm_smoke-1\61c551fd6ea14630a943cafb91bc1cc2|

SHA-verified source/evidence checkpoint: `H:\FreeCAD-src\build\checkpoints\trim-domain-correspondence-20261007-114057.zip`.
