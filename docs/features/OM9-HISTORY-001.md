# OM9-HISTORY-001 — History Workflow

Source engineering contract: specs/01-core/om9-history-001-history-workflow.md in Spec v1.

Existing validated slice: native SurfaceHistory for Sweep1/Sweep2/Loft; see
[Surface History validation](../validation/2026-10-09-surface-constraints-history.md)
and [surface contracts](surface-advanced-options.md).

New branch: document policy and a Rust-validated native Join History dependency graph, integrated with existing SurfaceHistory policy.
RCORE-09 policy revision (2026-10-09): Record controls new supported links;
Update controls existing recorded descendants independently of Record.
The [native History contract](history-native-contract.md) documents command
routes, defaults, ownership, transactions, ordinary Edit detach behavior and limits.

Status: partially_implemented. Surface graph slice validated; new Join/global-policy branch validated_supported_slice (2026-10-09).
Fixtures: rust/tests/history_graph.rs and tests/history_smoke.FCMacro.
Native SDK History83/83 and latest cold restore4/4 pass;
see the [validation record](../validation/2026-10-09-edit-history-special-types.md).
No full Matrix History compatibility is claimed.
Current evidence and remaining work are in the [live ledger](../openmatrix9-progress.json).
