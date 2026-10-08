# Existing native block reference target edits

Verified in the development checkout and installed SDK module/scripts. This supersedes earlier guards against changing an existing reference to another verified definition in the same archive namespace. Full openNURBS remains in progress; the public primary checkout remains clean and has not been integrated or pushed.

## Supported behavior

- Existing rigid App::Link references use their actual LinkedObject definition; affine references expose OM9DefinitionTargetUUID while retaining the immutable native matrix and editable rigid delta. A target must be a unique App::Part definition from the same included archive/namespace. Foreign, missing, nil, ambiguous and non-definition targets reject.
- Host signatures and dependency traversal follow the current target rather than the original native target. Canonical nested references, independent copied references and proxies following canonical copies use the same target resolution. Native instance geometry remains ON_InstanceRef; original selected instance UUIDs remain stable.
- Native instance_definition_uuid overlays update the graph before definition/member-overlay reachability and Rust closure. This retains the new definition graph and omits an obsolete target when no selected dependency needs it. Dependent copies follow their edited canonical source; independent overlays retain their own reference target/matrix. Same-hash multiple namespaces remap the resulting native graphs separately.
- Rust instance edit policy, bounded graph closure and native matrix/target validation remain authoritative. Cycles, singular transforms, unsupported actions, malformed targets, missing dependency information and incompatible userdata fail before destination replacement. Source snapshots are immutable.
- Affine derived preview traversal follows the current target and composes its placement once. CAD/mesh target changes, reflected/nonuniform link scaling, nested targets, FCStd, later edits and Undo/Redo are covered by live/native geometry checks.

## RED to GREEN and persistence

The native test originally rejected the new target definition overlay as unreachable because selection still followed the source reference graph. Reference overlays now run before reachability. The host test originally rejected retargeted links, then exposed forwarded App::Link attributes being counted as extra definition containers. Target resolution now identifies actual App::Part containers.

The mixed CAD/mesh affine test exposed a FCStd false-positive preview integrity failure: OCC restore renormalized a direction by roughly 1e-16 and changed two final decimal edge-parameter digits. The derived BRep display fingerprint now canonicalizes numeric tokens to12 significant digits. Original host/native geometry and its source signature are not quantized. Verified legacy exact preview fingerprints remain accepted; real preview geometry edits still reject atomically after reopen. This is a display integrity fingerprint, not an exact native payload comparison. Quantization boundaries can still warrant further persistence fixtures; tiny changes below this display fingerprint resolution are not claimed to be detected.

The test compares closed-solid volume24 rather than the volume of a compound containing an open mesh face, whose OCC aggregate volume is not a solid-volume measure. Preview and native XMin111 agree; the solid is unchanged.

## Validation

- SDK OpenMatrix9Gui link and subsequent script refresh builds exit0, complete before GUI suites. Installed ThreeDmArchiveState.py SHA256 matches the tested source.
- Rust74 tests across18 suites, exit0; cargo fmt --check exit0.
- Native CTest8/8 exit0 in29.62s, including both immutable real-ring geometry fixtures. The merge suite additionally checks independent copied-reference target/matrix world point(-183,628,1251), canonical-dependent copy world point(17,28,51), two separately remapped target graphs, invalid target UUID/class, cycles, singular matrices and atomic unchanged destination/source. Geometry-only ring success does not prove full preservation of opaque resources/classes in those files.
- FreeCAD200 checks across16 suites; every final process exit0. Reference suite18/18 includes source UUID retention, obsolete closure exclusion, live/native geometry, reflected scale/definition placement, legacy preview baseline, mixed-target affine preview, canonical/copy target independence, copied world point(26,38,50), FCStd, Undo/Redo, foreign/missing target refusal and edited-preview atomic refusal. The affine fresh-process reopen suite follows the current fixture generation.

|Suite|Checks|Artifact|
|---|---|---|
|reference_edit|18/18|`build/three_dm_reference_edit_smoke-1/47869b01a45a42c89c1a565cc03a8b4e/results.json`|
|independent_member|18/18|`build/three_dm_independent_member_smoke-1/5861de4a5ae54043957a8f04b081f00a/results.json`|
|shared_proxy|10/10|`build/three_dm_shared_proxy_smoke-1/611051f5ca384faab3201680b5880439/results.json`|
|shared_proxy_geometry|8/8|`build/three_dm_shared_proxy_geometry_smoke-1/953fcceb8602428394b0cb967f622081/results.json`|
|retained_member|6/6|`build/three_dm_retained_member_smoke-1/30c4bc1382484c85b0d5851d91871e33/results.json`|
|definition_placement|12/12|`build/three_dm_definition_placement_smoke-1/93fa5786e5644004abebb08cc5d7c497/results.json`|
|new_geometry|20/20|`build/three_dm_new_geometry_smoke-1/efbddbb951514276a96542738ffde1db/results.json`|
|preserved_export|13/13|`build/three_dm_preserved_export_smoke-1/e9aa0378c38043078734c3358134cc7e/results.json`|
|member_overlay|12/12|`build/three_dm_member_overlay_smoke-1/c15341e0d4ba42ca88528cbbc7a8f918/results.json`|
|block_structure|23/23|`build/three_dm_block_structure_smoke-1/e0404d015eb7493180f4e6a9f9e656e0/results.json`|
|affine_refresh|10/10|`build/three_dm_affine_refresh_smoke-1/8276e78b3cad4ed4a987c571bd08558d/results.json`|
|affine_refresh_reopen|2/2|`build/three_dm_affine_refresh_reopen_smoke-1/4e453b9db8ac4602a4758562377215c1/results.json`|
|source_signature|14/14|`build/three_dm_source_signature_smoke-1/939301278d814e809a8053bba1101544/results.json`|
|preservation|19/19|`build/three_dm_preservation_smoke-1/f36565a1c2cb460f833f98805d936e2c/results.json`|
|definition_overlay|8/8|`build/three_dm_definition_overlay_smoke-1/e95e24cc4883495fa35385f3f2e8f4b2/results.json`|
|layer_overlay|7/7|`build/three_dm_layer_overlay_smoke-1/88f2af788c5c4c0d85b8f876558d6cad/results.json`|

## Remaining scope

New standalone definitions/user block creation and explicit legacy owner/source/proxy migration remain pending. Accepting an already verified legacy preview fingerprint does not constitute a general provenance migration. Distinct-source document metadata merge policy, standard/menu/CMD preservation routing, complete-package review and primary public integration remain pending.

Independent editable copies still stage current host CAD/mesh rather than claim untouched native topology retention. Appearance, annotations, resources/settings/history, safe arbitrary plugin references, complete class/version coverage and actual Rhino5 application acceptance remain incomplete. No full-support claim or broad coverage-row promotion is made from these fixture-specific results.

## Source fingerprints

- `Gui/ThreeDmMerge.cpp` SHA256 `ecec8424b2b71383474f774f4388bc1d95c698cbfe69a64be1efd67063455f25`
- `ThreeDmArchiveState.py` SHA256 `364562c2d3309fb80f7a3e19f2823d504b84a7964e4e9b658991d97a01050f6e`
- `tests/native/three_dm_merge.cpp` SHA256 `65ec3d1f7292ab9b3a2b3e7db58797b874b59d76143ddf8b0e5560f0f80709a7`
- `tests/native/three_dm_blocks.cpp` SHA256 `b9efae0558bb6a0f9c3d55039dd9772669b9ae28b72508a98dfdd627c7c10f3c`
- `tests/three_dm_reference_edit_smoke.FCMacro` SHA256 `5bb740b050de61f92b9feb6b9cd7c08b61291a42e4be5235621d5d79cdbe0e57`
