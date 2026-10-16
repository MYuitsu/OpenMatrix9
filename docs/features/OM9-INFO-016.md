> **Current target: specs v2 (2026-10-13).** [OM9-INFO-016](../../OpenMatrix9_Codex_Spec_v2/specs/01-core/om9-info-016-matrix-clear-object-history.md)
> Use standalone OM9 and Matrix9 UI contract for new work. The record below is legacy FreeCAD/source evidence; its v1 links, native passes and host limitations do not certify the new runtime.

# OM9-INFO-016 — Matrix Clear Object History

Source engineering contract: specs/01-core/om9-info-016-matrix-clear-object-history.md in Spec v1.

Implemented slice: Selected HistoryJoin/SurfaceHistory objects lose incoming records while geometry and downstream links remain.
The [native History contract](history-native-contract.md) documents command
routes, defaults, ownership, transactions, ordinary Edit detach behavior and limits.

Status: partially_implemented; validated_supported_slice (2026-10-09).
Fixtures: rust/tests/history_graph.rs and tests/history_smoke.FCMacro.
Native SDK History83/83 and latest cold restore4/4 pass;
see the [validation record](../validation/2026-10-09-edit-history-special-types.md).
No full Matrix History compatibility is claimed.
Current evidence and remaining work are in the [live ledger](../openmatrix9-progress.json).
