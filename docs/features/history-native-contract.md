# Native History contract — 2026-10-09

Applies to OM9-HISTORY-001, OM9-TOOLS-017 and OM9-INFO-015..018.
This contract describes the new Join/Surface global-policy slice; its supported slice is validated in the SDK and cold restore.
Spec v1 engineering contracts define requirements. Defaults, supported geometry
and transaction boundaries below describe current OM9 decisions.

## Commands and policy

Stable menu/F6 aliases share the native controller with CMD History,
MatrixHistoryRecord, MatrixHistoryUpdate, MatrixClearObjectHistory and
JoinHistory (alias gvJoinHistory). Rust owns command identity, policy, graph
validation and descendant scheduling. C++ hosts document links, Qt controls,
native geometry recompute and transactions.

One native OpenMatrix9Gui::HistorySettings object stores per-document policy.
Record and Update initially Yes; Lock initially No; BrokenHistoryWarning
initially Yes. The History panel exposes all four. Each policy change commits
its own Undo transaction immediately. Closing the panel, Esc or Cancel closes
the session; it does not revert already applied policy changes.

The RCORE-09 policy revision (2026-10-09) separates Record and Update, superseding
the earlier OM9 baseline that suspended updates when either flag was off.
Original source descriptions and historical validation counts describe their
recorded baseline; they do not establish evidence for this newer policy.
Update=No suspends existing
History regeneration without clearing parent links. Record=No controls new
relationships and does not suspend existing ones. A new Join or supported Surface History created while Record=No is a geometry
snapshot without links; turning Record on later does not attach it retroactively.
Re-enabling Update schedules affected recorded descendants. Lock guards
independent edits to recorded child geometry, placement and construction parameters; parent
edits remain permitted. BrokenHistoryWarning controls detach feedback.

## Join History and graph ownership

Join History accepts at least two distinct whole Edge/Wire objects in the active
document that form one connected curve. Surface, Mesh, mixed and disconnected
inputs reject. Native curve Join supplies endpoint midpoint behavior and its
explicit tolerance checks. The panel displays Tolerance in mm, initially 1e-7.
Parents are retained. The result is a native OpenMatrix9Gui::HistoryJoin Part
feature at document root with world-space geometry, Parents links, Tolerance,
Recorded, HistoryDirty and OM9FeatureId metadata. HistoryDirty persists suspended
or failed output state rather than presenting its retained geometry as current.
Hidden PlacementSources tracks native
ancestor frames; group changes refresh links and touch the child.

With Update enabled an existing recorded child follows native parent links through recompute.
A child can become a parent of another Join History. Rust rejects cyclic or
invalid graphs and orders affected descendants; native links, not transient
pointer IDs, persist in FCStd. Independent Shape/Placement edits to an unlocked
child detach its incoming links while downstream links to that object remain.
Disconnected/null/error parents clear active child and descendant geometry while
retaining records for recovery; direct parent deletion detaches and retains the
last snapshot. Disabled global updates preserve suspended geometry.
Clear Object History detaches incoming records on selected HistoryJoin or
SurfaceHistory objects, including locked children;
it does not clear the entire graph or delete geometry.

Ordinary Join/Trim/Explode/Boolean remain snapshot operations. Before replacing
recorded children or parents, their Edit transaction checks Lock and detaches
affected Join/Surface History records with optional warning. Undo can restore geometry
and links. SourceNames strings do not establish dependencies. This integration
covers native Join and supported Surface History graphs; it does not promise History for
every existing builder or external feature type.

## Lifecycle and limits

Join History selection supports whole-object preselection, CMD object names,
viewport selection, Enter and selection Undo. Its nonmodal options panel creates
no child until OK. Input snapshots are revalidated before commit. One transaction
creates settings if needed and the child, then recomputes; failure aborts it.
Cancel/document switch/close discard the uncommitted session. Policy toggles and
Clear use separate transactions; document Undo/Redo is the persistent undo path.

Current scope covers Join-derived curve chains, existing Sweep1/Sweep2/Loft
SurfaceHistory records and per-document policy.
The separate [durable Builder foundation](builder-history-native-contract.md)
now extends this policy to native versioned template records and outputs, including
explicit deleted-output reconstruction, grouped frames and cold FCStd restoration.
Original arbitrary Builder solvers, full Matrix color conventions and general
interoperability remain unsupported. [Native Cage](cage-command-native-contract.md)
shares Lock/detach/Clear integration while its active capture dependency updates
independently of global Record/Update.
An existing, separately validated native SurfaceHistory graph supports Sweep1,
Sweep2 and Loft; see [its contract](surface-advanced-options.md) and
[validation](../validation/2026-10-09-surface-constraints-history.md).
Its per-object UpdateHistory remains an additional gate: global resume does
not override a locally suspended surface. Global Record controls new records;
global Update/Record, Lock, Clear and warning integrate with both native graphs.
No full Matrix/Rhino History compatibility is claimed.

## Verification

Fixtures: rust/tests/history_graph.rs and tests/history_smoke.FCMacro.
Supported SDK report: build/history_smoke-1/0a28ceafabe247a8ad0389272225c714/results.json
—83/83, including two generations, global Join/Surface policy, ordinary Edit
detach/Undo, grouped frame translation/rotation/reparent/Undo, invalid/null
full-chain propagation/recovery and FCStd persistence.
Latest cold restore: build/history_smoke-1/2e27955c3b79451a8ee7c47bcb1a8bff/results.json
—4/4. Status: validated_supported_slice; broad features remain
partially_implemented. See the
[validation record](../validation/2026-10-09-edit-history-special-types.md)
and [live ledger](../openmatrix9-progress.json).
