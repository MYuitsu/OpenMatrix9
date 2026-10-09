# OM9-INFO-015 — Rhino History

Source engineering contract: specs/01-core/om9-info-015-rhino-history.md in Spec v1.

Implemented slice: History panel exposes Record, Update, Lock and BrokenHistoryWarning; each change applies immediately in its own Undo transaction.
RCORE-09 policy revision (2026-10-09): Record controls new supported links;
Update controls existing recorded descendants independently of Record.
The [native History contract](history-native-contract.md) documents command
routes, defaults, ownership, transactions, ordinary Edit detach behavior and limits.

Status: partially_implemented; validated_supported_slice (2026-10-09).
Fixtures: rust/tests/history_graph.rs and tests/history_smoke.FCMacro.
Native SDK History83/83 and latest cold restore4/4 pass;
see the [validation record](../validation/2026-10-09-edit-history-special-types.md).
No full Matrix History compatibility is claimed.
Current evidence and remaining work are in the [live ledger](../openmatrix9-progress.json).
