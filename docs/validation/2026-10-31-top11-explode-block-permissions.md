# Top11 Explode: block member permissions and selection — 2026-10-31

Status: partial automated evidence; not complete Top11 or M03-UI acceptance.

A selectable block may contain hidden or locked members. Previously the transaction committed, then selecting every output failed, causing the command to report an error after changing the document. Top11 now selects only result roots with selectable scene geometry; if selection itself cannot be allocated, it clears selection and emits a warning after the already committed edit. Explode reports the number of created outputs rather than the number selectable. Member attributes and composed transforms remain intact.

The app fixture compares scene transform, visibility, selectability, editability and color before and after exploding a block with visible, hidden and locked members; it verifies successful command completion, selectable output count and Undo/Redo. Another fixture confirms that insertion onto a locked member layer is refused before commit, without changing the document or Undo stack. The former post-commit selection error was reproduced before the fix. These fixtures do not cover all attributes, non-renderable or nested members, all budget failures or scene-refresh failures after commit.

Windows x64, CARGO_TARGET_DIR=L:\om9-target, -j 2: 85 core document/curve_ops/top11 tests, 178 app Top11/options/BRep/snapshot tests and 106 no-default Top11/options tests passed. No-default reports existing unrelated warnings. Physical desktop and signed M03-UI acceptance remain open; a prior desktop launch attempt returned access denied. No native implementation was changed and no openNURBS measurement was performed.

Source SHA-256: app session/top11.rs `ca9f2855570360f019aab89c9f56c7c52bef926551797a096bc54dd28e13a96b`; app tests/top11.rs `2cc6f74acd8115bbe5026aff48619c334d29c91901d4d547899d32fdd2142235`.

Update: [scene preparation evidence](2026-10-31-top11-scene-atomic.md) covers budget failure before Top11 publication; other scene/renderer failure cases remain open.
