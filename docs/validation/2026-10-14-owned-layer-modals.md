# Owned Layers modal checkpoint — 2026-10-14

[Source-bound receipt](2026-10-14-owned-layer-modals.json) verifies the
Task3 dialog continuation in [the Layers plan](../superpowers/plans/2026-10-14-owned-layers.md).
This supersedes the earlier unavailable-dialog limitation for the implemented
owned-root slice. Full OM9-LAYER-001, M9IMG-04 and RCORE M01–M12 remain open.

Safe Rust owns a Rename/Color/Show draft and opaque dialog token. The token
binds coordinator authority/context, document identity/session/epoch/generation/
revision/context revision and a monotonic serial. It is checked on every edit
and affirmation. Foreign, consumed, replaced-session and Undo-roundtrip handles
cannot mutate or close a newer owner; cancel releases only the exact active
handle. Opening, editing and cancellation do not materialize lazy layers or
write model/Undo. Pending foundation Line start/preview and numeric CMD draft
survive the dialogs. Actual native replay also verifies all four cameras remain
unchanged. Broader document switching through a native open-file UI is pending.

Shift+right-click Advanced name opens Rename, and Advanced swatch opens Color;
only slots17..32 allow them. Rename trims and reuses the existing128 UTF-8 byte,
nonempty/control-free/unique/reserved-name policy. UI draft storage is bounded
to512 bytes; invalid input stays editable. Color uses bounded three-character
integer RGB fields0..255, with Up/Down stepping one unit. Invalid RGB cannot
silently affirm; existing alpha is retained. Affirmation creates a missing group
and changes its target atomically in one Undo, retaining preset slot/UUID/current
and membership. By-layer scene color updates reuse immutable geometry buffers.
Accepted no-op Rename/Color/empty Show closes without allocating action buffers,
changing Undo or erasing Redo, including a tight-budget fixture.

Right Show owns an unchecked list of captured OM9-Hidden roots. Choosing entries
and OK clears only those roots' Hidden flag; Cancel writes nothing. Explicit
visibility/selectability, layer Off/Lock, unrelated Hidden roots, canvas selection
and current remain independent. Each row routes through captured SourceStamp and
object identity even when labels duplicate. Definition-member Hidden masks stay
independent; complete structural/imported/builder families are not certified.
The initially unchecked choice policy is an OM9 ruling; screenshots alone do not
establish the original dialog's default selection.

Snapshot vectors/string capacity, current dialog and bounded UI input/error
reserve are held in controller scene admission before allocations and publication.
max_outputs rejects oversized hidden lists before copying labels. The isolated
test-only allocator forces moving realloc and measures the whole test heap:
held-modal publication refuses without changing epoch/draft/model at
2,214,396–2,214,396 bytes under2,500,000;1,000 hidden roots
with8,192-byte labels refuse snapshot allocation at9,368,430–9,373,126
bytes under14,000,000. These fixture bounds do not certify global egui/GPU caches.

Actual widget regressions reproduced and fixed three reviewed defects:
7,700-byte/multiline labels formerly produced1,664/13,013px rows while virtualization
assumed18px; clipped single-line rows now stay17px with a clickable label/checkbox
and preserve original text/stamp. Repeated modal serials formerly retained an
Area/ScrollState per open; one reusable area/scroll owner now remains at2/6 total
states after1/10/30 closed cycles, resetting scroll for each new draft. Action
button/choice IDs still include authority+serial: releasing an old OK after cancel/
reopen cannot affirm the new token. Token-specific editor state is removed on close,
while CMD editor state survives. Independent review reproduced each RED/GREEN.

Measured: **439 portable workspace tests / 24 ignored SDK tests**;
**61 focused checks are a workspace subset**. Warning-strict all-target
Clippy, format and release build passed. Eleven controller and19 actual-widget
tests cover stale/foreign/cancel/no-op/invalid/IME-event/budget paths. Four cold
fixture tests spawn five bounded own OM9 children; each exits cleanly. The new
fixture commits actual dialogs, checks Undo/Redo, saves/reopens `.om9` in a fresh
CLI child and rejects an old modal handle after same-file reopening. This is not
an accepted interactive native Save/Open workflow.

Release native window replays pass10 dialog,13 earlier Layers and6 viewport-menu
checks. Receipts bind source hashes, executable, screenshot, observed native
window/modules and no descendants/forbidden legacy hosts. Compiled SVG inputs
are included in the new source freeze. Three screenshots capture Rename, Color
and Show while open; the final closed workspace screenshot is separate. The own-
app raw-input hook exercises real egui widgets/CMD; it never injects desktop
input into another application. Debug PNG encoding measured879–967ms, exceeding
egui's0.8-second click limit when it landed between replay press/release. Replay
now waits for each screenshot save acknowledgment before its next gesture;
production click handling is unchanged. This audit path is not performance proof.

Pending: output-layer binding for command families beyond foundation Point/Line;
full structural/native/member behavior, MatrixArt/material, interactive save/open,
OS IME/DPI/multi-monitor, full keyboard navigation, async scene cancellation,
global cross-job/render-cache budgets and large-scene latency. No new SDK jobs
were run; earlier SDK receipts retain their historical checkpoints. No complete
feature, capability or migration gate is promoted by this checkpoint.
