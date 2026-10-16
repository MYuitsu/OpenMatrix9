# Owned Layers panel/CMD/scene checkpoint — 2026-10-14

[Measured source-bound receipt](2026-10-14-owned-layers-ui.json) covers a part of
Task3 in [the complete Layers plan](../superpowers/plans/2026-10-14-owned-layers.md).
Full OM9-LAYER-001, M9IMG-04 and RCORE M01–M12 remain open.

Safe Rust owns current layer, assignment, lazy groups, selection, Hidden and
permissions. Basic rows keep Metal/Gem at left and User01–04/Heads/Finger/Cutting/
Creation at right. Advanced is collapsed by default, expands in the screenshot
order, and keeps stable preset IDs independently of labels (slot21 is User25).
Lights is slot33. Name/swatch left-click chooses current; arrow left-click
assigns selection without changing current; swatch right-click replaces eligible
visible/selectable/drawable membership. Locks, visibility and group bars preserve
explicit object visibility/selectability/Hidden. Hide sets only Hidden, reverse
Hide uses visible drawable roots outside the selected set, and Show restores
only Hidden. Controls use authored vector shapes so native missing-font glyphs
cannot replace triangles/locks/visibility/carets with tofu boxes.

`Layer=1` through `Layer=33`, canonical names, Lights and None/0 route through
the same owned controller. Unquoted numeric/None/0 tokens retain their reserved
meaning; quoted names first resolve a unique existing case-insensitive label.
Thus an already-valid Advanced custom label such as `"1"` or `"None"` is
addressable without weakening stable slot syntax or banning old metadata.
The parser is bounded to256 bytes and rejects control characters/unbalanced
quotes. Pending foundation Point/Line sessions retain start point, CPlane,
BothSides, camera and input draft across a successful owned layer action;
current is read again at actual output acceptance. Off accepts invisible output,
lock refuses output without losing the draft. Assign creates a missing group
and applies membership atomically in either request order with one Undo.

Controller selection is admitted before replacing its owned tree. Membership
uses two borrowed passes grouped by root, not one whole-scene scan per root;
all-hidden Cloud buffers and hidden definition leaves do not create selectable
phantoms. Selection-source collection checks max_outputs and held world/cache/
selection storage before allocation. Scene metadata refresh proves unchanged
identity/context/geometry/placement, reuses immutable display buffers and
refreshes inherited masks/colors/stamps. Old and prospective cache metadata,
world and Undo states remain held in admission before publication.

Scene startup and Undo/Redo refresh now preflight expanded occurrences, cache
buffers, paths, diagnostics and scratch before allocation. Every Instance
occurrence is charged before descent, including empty shared definitions:
100 roots over seven binary levels have25,500 visits even with zero drawable
leaves and are refused at max_nodes2000 while the502-node catalog validates.
Scene allocations are reserved before filling so moving Vec reallocation does
not add an unadmitted old/new coexistence peak. These fixture bounds are not
global GPU/cross-job memory or full performance certification.
The isolated test-only allocator forces alloc/copy/free on realloc and measures
the whole test executable heap: nested 4,096-leaf peaks 2,190,638–
2,190,638 bytes under 2,500,000; Cloud 65,537-point peaks
3,417,751–3,417,751 bytes under 5,000,000. Independent review
reproduced the moving-realloc RED 2,872,138 before this fix and GREEN 2,186,682
after it, plus initial 1.9-MB refusal, 3-MB publication refusal, empty-DAG budget
and hidden Cloud selection. Its 18 checks are separate review probes and are
not added to the portable workspace total.

The native replay uses the real egui controls and actual CMD through this
application's raw-input hook; it never injects desktop input into other apps.
Thirteen Layers checks verify pending Line/current/output, swatch selection,
lazy Assign, Hide/Show, group Off/On, Lock/Unlock, Advanced/collapse and actual
CMD Undo/Redo restoring the same preset UUIDs. Removing a duplicate running
command row from Main Menu restores the approved CMD-only prompt location and
keeps layer hit regions stable when a command accepts. A widget regression
checks that stability. Six native viewport-menu checks were rerun for popup,
numeric draft, Maximize/Restore, Escape and outside dismissal.
The S6 popup uses a light background with dark enabled labels, fixed content
width and work-area height; it scrolls in short windows. Native capture waits
for the opening animation to settle. When the tall menu overlaps its anchor,
the replay renders one closed frame before the next gesture so egui's prior
Area hit regions have expired; the production action/ownership gates are
unchanged. The receipt requires the popup still open when capturing. Actual
native PNG inspection confirms the complete S6 list through Viewport Properties.
These checks do not establish immediate reopening with no closed paint frame.

Measured: **414 workspace tests / 24 ignored SDK tests**;
**37 focused checks are a workspace subset**. Format, warning-strict
all-target Clippy and release build passed. Two own-app 1600×1000 native windows
produced source/binary/screenshot-bound receipts, observed window handles and
loaded modules, with no descendants or forbidden legacy host dependencies.
Four bounded cold OM9 children from the workspace suite exited cleanly and
verify owned metadata/schema migration, not interactive save/reopen acceptance.
Older native SDK/schema-policy receipts retain their historical source checkpoints;
there is no fresh combined SDK count in this UI batch.

Pending: Advanced rename/color and selective-Show delayed dialogs (currently
visible-unavailable), their Cancel/stale/owner lifecycle; current-output binding
for other command families; complete keyboard/DPI/IME behavior, interactive
cold-save workflow, structural/native family combinations, full large-scene
latency/global memory and all complete Layers/M9IMG/RCORE acceptance gates.
Synchronous scene capture is traversal-bounded but has no cancellation token;
general async cancellation remains pending.
No full feature/capability/catalog implementation status is promoted.
