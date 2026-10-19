> **Current target: specs v2 (2026-10-13).** [OM9-TOOLS-017](../../OpenMatrix9_Codex_Spec_v2/specs/09-tools/om9-tools-017-join-history.md)
> Use standalone OM9 and Matrix9 UI contract for new work. The record below is legacy FreeCAD/source evidence; its v1 links, native passes and host limitations do not certify the new runtime.

# OM9-TOOLS-017 — Join History

Source engineering contract: specs/09-tools/om9-tools-017-join-history.md in Spec v1.

Implemented slice: Connected separate native curves create a retained-parent HistoryJoin child; surfaces and Mesh reject.
The [native History contract](history-native-contract.md) documents command
routes, defaults, ownership, transactions, ordinary Edit detach behavior and limits.

Status: partially_implemented; validated_supported_slice (2026-10-09).
Fixtures: rust/tests/history_graph.rs and tests/history_smoke.FCMacro.
Native SDK History83/83 and latest cold restore4/4 pass;
see the [validation record](../validation/2026-10-09-edit-history-special-types.md).
No full Matrix History compatibility is claimed.
Current evidence and remaining work are in the [live ledger](../openmatrix9-progress.json).
