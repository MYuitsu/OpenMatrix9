# Owned layer transaction primitives — 2026-10-14

This implements Task1 of [the complete Layers plan](../superpowers/plans/2026-10-14-owned-layers.md),
within RCORE-00 and OM9-LAYER-001. Whole Layers, M9IMG-04 and M01–M12 remain open.

`LayerPlan` captures coordinator/caller context and document owner, epoch,
session generation, revision and unit-context revision. Create/current, Update,
SetCurrent and plain-root Assign use the owned coordinator. New layer UUIDs
participate in the retired identity ledger; source ownership and all batch
permissions are validated before private document clones. Metadata read/write
overlap rejects Lock+Assign and changed-layer output batches instead of using
the old permission. Caller savepoint rollback retains retired UUID authority
and leaves the prior global Redo executable.

Current does not assign existing objects. Assign retains current, geometry and
placement, records by-layer color and rejects locked/foreign/structural/retained
sources. Lock/visibility leave explicit object visible/selectable preferences
untouched. A current no-op at a full History budget creates no publication or
Undo entry and preserves Redo. Create/update/Assign use checked revisions and
admitted storage; Cancel before preparation and after private preparation keeps
the immediate world/stack unchanged.

The scene resolves assigned color from the layer and updates after a later
swatch change. An independent actual OM9 child reopens a saved .om9 and verifies
UUID/order/current/name/color/lock/visibility, object preferences and geometry.
It runs under the existing NativeProgram job with a 10-second deadline and
64-KiB stdout cap; exited=true/active_processes=0. Its measured private commit
peak is 1,925,120 bytes for this single authored metadata fixture.

Measured receipt: [source-bound batch](2026-10-14-owned-layer-primitives.json).
375 workspace tests pass; 23 SDK tests remain ignored here. The 15 focused
checks (12 transactions, one scene, one retained refusal, one cold process) are
workspace subsets, not 15 additional distinct tests. Format and all-target
warning-strict Clippy pass. Independent review reproduced/fixed the full-History
no-op issue and verified the corrected primitive. Assign uses one admitted
source set and one retained-row scan, polls Cancel, and uses a bounded borrowed
layer permission walk; no quantitative large-scene latency claim is made.

Pending: durable preset-slot/independent Hidden schema and explicit migration;
complete current-output binding during command sessions; actual layer panel/CMD,
multi-selection, right-click actions, group toggles, Advanced dialogs and Lights;
complete UI/native metadata/renderer/performance acceptance. Older SDK/native UI
receipts retain their earlier source checkpoint and are not refreshed by this
primitive batch. No feature/capability or full acceptance gate is promoted.
