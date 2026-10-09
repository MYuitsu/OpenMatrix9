# OM9-INFO-018 — Matrix History Update

Source engineering contract: specs/01-core/om9-info-018-matrix-history-update.md in Spec v1.

Implemented slice, RCORE-09 policy revision (2026-10-09): Update suspends/resumes
existing recorded descendants without clearing links, independently of Record.
The [native History contract](history-native-contract.md) documents command
routes, defaults, ownership, transactions, ordinary Edit detach behavior and limits.

Status: partially_implemented; validated_supported_slice (2026-10-09).
Fixtures: rust/tests/history_graph.rs and tests/history_smoke.FCMacro.
Native SDK History83/83 and latest cold restore4/4 pass;
see the [validation record](../validation/2026-10-09-edit-history-special-types.md).
No full Matrix History compatibility is claimed.
Current evidence and remaining work are in the [live ledger](../openmatrix9-progress.json).
