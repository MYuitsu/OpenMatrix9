# Top11 Explode: block entry route — 2026-10-31

Status: partial automated evidence; not complete Top11 or M03-UI acceptance. The normalized Top11 reference distinguishes the button (spares block instances) from typing Explode in the command line (explodes an instance). Both routes use the same dispatcher with an explicit route option retained through select-after-invocation.

CMD Explode replaces an instance by fresh copies of its direct definition members, composing the root and member placements; other instances and the shared definition stay intact. Nested instances remain instances after this one-level operation. Group membership is updated with other Explode replacements in the same transaction. The toolbar skips instances in a mixed supported selection and refuses an instance-only selection. A block output-count limit is checked before cloning members.

Fixtures cover preselection and select-after-invocation, shared definition, composed translations, group membership, .om9 roundtrip, Undo/Redo, mixed block/polycurve CMD and toolbar behavior, output-limit atomic refusal, and one nested instance's world transform. They do not establish all inherited display/permission attributes, nested depth/resource cases, external 3DM block fidelity or physical UI behavior.

Windows x64, CARGO_TARGET_DIR=L:\om9-target, -j 2: 85 core document/curve_ops/top11 tests, 175 app Top11/options/BRep/snapshot tests and 103 no-default Top11/options tests passed. The additional nested-instance fixture passed separately under both configurations after those suites. No-default reports existing unrelated warnings. Physical desktop and signed M03-UI acceptance remain open; a prior desktop launch attempt returned access denied. No native implementation was changed and no openNURBS measurement was performed.

Source SHA-256: app session/top11.rs `7d6fe8f296b06549f6a7ae87d0e4485f371e791ecd4e191aba6fc6d1df21ddd5`; app tests/top11.rs `f701b66e498efafaba44f78a977d924c0c358fe4c8da78d768a7a95b0eca5600`.

Update: [member-permission evidence](2026-10-31-top11-explode-block-permissions.md) covers hidden and locked block members and output selection.
