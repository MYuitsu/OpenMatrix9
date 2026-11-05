# Top11: prepare scene before transaction publication — 2026-10-31

Status: partial automated evidence; not complete Top11 or M03-UI acceptance.

Top11 previously committed document edits before building the new scene. If scene admission failed, it returned an error after changing the document and discarded the old scene. Top11 now prepares its transaction and candidate scene while retaining the current scene, reserves the candidate cache, and publishes only after successful preparation. The candidate document is already part of the transaction reservation; its object count is not counted a second time during scene validation. Post-publication result selection continues to skip hidden/unselectable geometry.

A 4096-leaf nested block fixture with a 3 MB budget reproduces the former Duplicate failure: the operation reported an error after the document changed. With the change, it is refused with original document, scene and Undo depth intact. An existing Copy budget fixture confirms its first copy and retry behavior still work. The tested failure is budget-based; exhaustive allocation failures, concurrent state changes, renderer/GPU publication and Undo/Redo scene failures are not established here. Preparing a full candidate scene may use more time/memory and refuse larger jobs than an incremental update.

Windows x64, CARGO_TARGET_DIR=L:\om9-target, -j 2: 85 core document/curve_ops/top11 tests, 179 app Top11/options/BRep/snapshot tests, 107 no-default Top11/options tests, and five synthetic Top11 UI tests passed. No-default reports existing unrelated warnings. Physical desktop and signed M03-UI acceptance remain open; a prior desktop launch returned access denied. No native implementation was changed and no openNURBS measurement was performed.

Source SHA-256: app session/top11.rs `b63e3baa6b92d595dd64d28f64f6d90e19826294c030f14336a3752389be39ad`; app tests/top11.rs `c7c3f526bd5d1d5692485c1006e33b6ec72e5d9a8dccb03aceddcbc78943e656`.
