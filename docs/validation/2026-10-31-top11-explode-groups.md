# Top11 Explode: preserve group membership — 2026-10-31

Status: partial automated evidence; not complete Top11 or M03-UI acceptance. The normalized Top11 Explode reference says the exploded objects remain grouped.

Explode now maps each deleted source to its generated objects and replaces that source in every containing group, preserving group identity, name and unaffected members. The membership update, deletion and inserts occur in one transaction. A GroupReplace transaction mutation checks document and group revisions, member existence/uniqueness, and conflicting group writes before applying the edit; Undo/Redo restores the entire document state.

The app fixture explodes a two-segment polycurve belonging to two groups, one also containing an unaffected object and the other containing only the source. It checks CMD and toolbar, group identities and membership, .om9 roundtrip and Undo/Redo. Transaction fixtures reject stale group revisions, repeated writes and missing members without changing the world. Mesh and BRep groups share the mapping but have not been separately verified in this batch; nested or overlapping group selection interactions, broader group policies and aggregate resource limits remain open.

Windows x64, CARGO_TARGET_DIR=L:\om9-target, -j 2: 37 document graph/change tests, 48 core curve_ops/top11 tests, 168 app Top11/options/BRep/snapshot tests and 97 no-default Top11/options tests passed. The no-default build reports existing unrelated warnings. Physical desktop and signed M03-UI acceptance remain open; the desktop launch attempt reported access denied in the preceding mesh evidence. No native or openNURBS measurement was performed.

Source SHA-256: transaction.rs `cf7d288b2e37b9418c4eb2c731c27f3f15abd9ea0c738547a4e3d4badc786dfb`; document_graph.rs `7dc627f1b38898a36140d59ced327e76ad1351e2fc377cdd68d1e2aaadc102d4`; app session/top11.rs `7699c23fba7d8ed70a516f98c14afbf7f4e26eef5c6323e3083341ce70c09963`; app tests/top11.rs `a7f89a1d5ac5ac2704bc90a3b6054730ed388cdcca673b9a7b0b2b11183f696b`.

Update: [mixed Explode evidence](2026-10-31-top11-explode-mixed.md) covers one grouped curve/mesh/BRep selection.
