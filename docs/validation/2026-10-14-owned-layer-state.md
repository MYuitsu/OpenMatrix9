# Owned layer state/schema policy — 2026-10-14

Measured [source-bound receipt](2026-10-14-owned-layer-state.json) extends Task1
with a scoped part of Task2 in [the full Layers plan](../superpowers/plans/2026-10-14-owned-layers.md).
Full OM9-LAYER-001, M9IMG-04 and RCORE M01–M12 remain open.

Safe Rust now owns durable preset slots 1–33 independently of UUID/display label,
using the existing local OM9 names/colors with explicit LGPL provenance. Basic
and Lights presets retain their fixed palette/name; only Advanced17–32 admit
custom labels/colors. Slots are unique, immutable across Update and range checked.
Names enforce trim/control/128-byte/reserved-name rules; streamed Unicode lower
case comparisons reject collisions with other layers, including two proposed
renames in one batch before any private clone. Generic imported duplicate labels
remain valid unless they collide with an Advanced custom label. No old layer
is assigned a slot merely from its name or table order.

`Attributes.hidden` is independent of explicit visible/selectable, layer Off and
lock. Transactional Hidden changes preserve those preferences and use the same
session/source generation, attribute revision, Cancel/Undo/Redo and private
publication barriers. Scene tests cover root and definition-member masks for
display, picking, edit and reference; Show restores only OM9 Hidden.

`.om9` schema4 saves the slot and independent Hidden. Opening schema1/2/3 requires
the corresponding explicit migration. Legacy preflight rejects new fields even
false/null in root/member attributes and layers, while preserving Cloud.hidden
and unrelated geometry/recipe fields. Cancel, budget and restoration counter
overflow leave the original file unchanged. Actual `om9 --migrate-v3` followed
by a fresh owned process reopens schema4 without inferred preset identity.

Native schema1 geometry wire remains closed: borrowed serialization omits preset
and Hidden fields, projects object visibility as explicit visible && !Hidden,
and never edits the OM9 document. Actual SDK write/fresh inspection/publication
of a hidden root Point verifies that projection. Export receipts expose counts
for dropped independent Hidden distinctions/preset slots. Diagnostics admit
the live JSON and all response strings/vector storage before allocation.
Retained native projection uses the same visibility rule; its wider cold suite
is historical and is not newly accepted by this batch.

Measured: **394 workspace tests**, **24 ignored SDK tests**;
**34 focused checks are included in the workspace total**. Format and
warning-strict all-target Clippy passed. Four actual owned OM9 children exited
with no active descendants, under a 10-second/64-KiB stdout cap; measured private
commit peaks 1,912,832–1,949,696 bytes for these small authored fixtures.
The separate [native export receipt](2026-10-14-owned-layer-state-exchange.json)
runs **8 SDK integration tests / 18 jobs**, outside
the workspace portable count; every job was reaped. Independent review ran
30 focused checks and reproduced the two corrected admission defects.

Pending: pure CMD/group action routing, current output rebinding at actual
command acceptance, complete Layers widgets and gestures, selection and delayed
rename/color/selective-Show dialogs, Advanced/Lights UI, native hidden definition
members/locked-hidden combinations, whole family/Rhino/performance/global-memory
acceptance. Older native/UI and broad foundation receipts retain their prior
source checkpoints; no combined current-source SDK total is claimed here.
No full feature/capability status or acceptance checklist is promoted.
