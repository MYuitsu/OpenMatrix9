# Owned File lifecycle implementation plan

> **For agentic workers:** Use superpowers:executing-plans inline, with TDD and one final independent review. The ongoing user goal authorizes implementation; no staging/commit/push or new worktree.

**Goal:** Implement the complete standalone New/Open/Save/Save As lifecycle required by RCORE-00 and OM9-FILE-002/003/005/006, preserving Matrix9 menu/CMD/toolbar behavior and the full compatibility requirements.

**Architecture:** Safe Rust owns file identity, dirty state, delayed intent, session guards, Save/Discard/Cancel, candidate document/scene preparation and publication. The existing `.om9` archive supplies bounded restoration and atomic output. Native OS picker selection is an adapter that returns an owned path and captured token; it never replaces document state itself.

**Tech Stack:** Existing Rust core/app/archive/scene, egui0.36.2; a selected Windows picker adapter after its license/dependency/lifetime review. No FreeCAD/Qym desktop or whole OCCT.

**Spec:** `OpenMatrix9_Codex_Spec_v2/specs/00-rhino-core/00-qymcad-foundation-migration.md`, `MATRIX9_SCREENSHOT_DETAILS_2026-10-14.md`, `../01-core/om9-file-002-new.md`, `om9-file-003-open.md`, `om9-file-005-save.md`, `om9-file-006-save-as.md`.

## Global constraints

- Preserve v1, historical receipts and all unaccepted M01–M12/feature/capability gates.
- New/Open dirty replacement requires explicit Save/Discard/Cancel. Save failure or Cancel prevents replacement; validate candidate before publish.
- Save includes the complete supported document, including hidden/off/locked state, Layers, History/results/resources and retained records. Full view/material/annotation persistence remains a required continuation wherever the current schema lacks it.
- Save first asks path; subsequent Save uses active path; Save As always asks and adopts a new path only after filesystem acceptance.
- `.om9` and `.3dm` are separate codecs. No extension deception or enabled unsupported branch. `.3dm` UI/codec interoperability remains in the complete objective.
- Save/output is not a geometry Undo action. New/Open creates fresh runtime authority and clears old document command/selection/Undo callbacks only after candidate acceptance.
- All fixture paths, generated files and build artifacts stay under H. Tests remove only their explicitly owned files/directories.

## Review focus

1. Undo→new branch can reuse a document revision but contain different data: dirty must not compare revision alone.
2. New/Open cancel/error preserves an active Line draft, old document/scene/current/selection and executable Redo.
3. A picker or overwrite response arriving after document mutation/reopen cannot save or replace a newer session.
4. Old world/Undo/scene/dialog/input owners coexist with archive decode/restore and candidate scene: reserve aggregate peak before allocations.
5. Save As failure or an unsupported suffix cannot adopt the destination, mark clean or damage an existing target.

## Task1: Controller-owned lifecycle and I/O

**Files:** Create `standalone/crates/om9-app/src/file_lifecycle.rs`, `src/session/files.rs`, `tests/file_lifecycle.rs`; modify `src/session.rs`, `src/session/layers.rs`, `src/session/layer_dialogs.rs`, `src/lib.rs`; add held constructor in `standalone/crates/om9-core/src/transaction.rs` with a targeted test.

**Interfaces:** `FileKind::{New,Open,Save,SaveAs}`, `FilePhase::{Unsaved,OpenPath,SavePath,Overwrite}`, opaque `FileToken`, `FileOutcome::{Pending,Saved,Replaced}`. Controller exposes `begin_file_operation(kind)`, readonly active operation/token/phase/path, `decide_unsaved(token,decision)`, `accept_file_path(token,path)`, `confirm_file_overwrite(token,bool)`, `cancel_file_operation(token)`, `active_file_path()`, `is_dirty()` and restoration diagnostics. Later UI routes exact feature IDs through these methods.

- [x] Write RED tests for initial/edited/saved/Undo/Redo/branched dirty state, first/subsequent Save, Save As cancel/failure/overwrite, full Hidden/Layer persistence, New/Open dirty decisions, missing/corrupt/unsupported/stale/foreign paths, same-file new authority and tight old+new peak refusal.
- [x] Run `rtk proxy cargo test -p om9-app --test file_lifecycle -- --nocapture`; witness missing behavior, then implement safe owned state and guarded synchronous I/O using existing archive contracts.
- [x] Dirty fingerprint streams serialization into SHA256 without whole JSON allocation; cache by coordinator epoch, compare saved content fingerprint. This avoids retaining an extra unaccounted saved Document and handles revision reuse.
- [x] Stage candidate document/scene under old retained usage and full configured limits. Add `TransactionCoordinator::new_with_held(world,limits,held)` rather than silently shrinking the future controller's configured limits.
- [x] Run targeted tests and existing Layers/viewport/controller/cold/allocation regressions. Preserve all preacceptance state; successful intermediate Save in a replacement flow is reported as actual accepted I/O even if subsequent candidate preparation fails.

## Task2: Matrix9 entry routes and native picker

**Files:** `src/ui.rs`, `src/ui/chrome.rs`, new `src/ui/files.rs`, dispatcher/transcript/main launch and native-picker adapter as actually selected; actual-widget tests.

- [x] Route menu/CMD/F6/Standard New/Open/Save icons through the same owned operation. Keep exact IDs and unavailable codecs visible with reasons.
- [x] Render owned Save/Discard/Cancel and overwrite prompts; gate keyboard/pointer/IME so old gestures cannot affirm a new token. Reuse bounded area state; clear per-token editor state.
- [x] Use a real OS file chooser and owned callback. Cancel leaves path/document/draft intact. State the adapter's actual unsafe/native/dependency boundary and supported platforms without claiming third-party memory safety.
- [ ] On accepted replacement clear old transcript draft/preview/modal/popup/selection and refresh window title/path; failed/canceled operations retain them. Native `--ui path` must retain active path and expose restoration diagnostics.
- [ ] Verify actual widget events, native window menu/CMD/icon routes and native chooser lifecycle where automation permits. Do not substitute direct controller calls for real UI evidence.

## Task3: Complete persistence/interoperability continuation

- [ ] Add versioned supported view/CPlane/camera/material/annotation metadata persistence and explicit migration; preserve unknown/unavailable backend diagnostics.
- [ ] Wire selected `.3dm` codec through owned import/export/file lifecycle only after exact type×field capability evidence; retain loss/provenance/session guards.
- [ ] Cover close-window dirty flow, resource failures, fresh-process reopen, old references/late jobs, snapshot/retained source and complete original File acceptance conditions.

## Task4: Verification and synchronized evidence

- [ ] Run format, warning-strict Clippy, workspace/release/native UI/cold process checks with source/binary hashes including compiled SVG inputs, appropriate SDK jobs only for changed native adapters.
- [x] Independent review; fix Important/Critical findings with RED→GREEN tests.
- [ ] Update v2 specs/catalog/lookup/YAML/status/ledger/manifest, README en/vn, live progress and backend measurement roadmap to measured scope; public-source audit and immutable v1 checks.

Ruling: execute in the authorized H checkout and retain uncommitted work—repository instructions prohibit implicit staging/commits. The current schema and selected codec are not complete File feature acceptance; this plan keeps their missing requirements explicit instead of redefining success.

## Owned File lifecycle service — 2026-10-14

[Controller/I/O evidence](../../validation/2026-10-14-owned-file-service.md): 458 workspace tests/80 focused subset;17 File service checks and two new bounded cold OM9 children. New/Open/Save/SaveAs ownership, dirty fingerprints, per-response tokens and atomic archive I/O are implemented as Task1 prerequisites. File menu/CMD/native chooser remain disabled/unaccepted; .3dm and full view/material persistence remain required. Existing native Layers/viewport regressions are separate from File UI evidence. Full feature/M01–M12 gates and catalog statuses remain open.

## Owned File UI and native cancellation — 2026-10-14

[Current scoped evidence](../../validation/2026-10-14-owned-file-ui.md): 475 workspace tests/96 focused subset;16 File widgets;8 native File checks and3 real Windows picker cancellations, distinct from scripted path delivery. Native `.om9` menu/CMD/F6/Standard routes and owned prompts are connected.10+13+6 Layers/viewport regressions and7 cold children are separate. OS successful selection/save/open, native backend diagnostics/single overwrite UX, UI/native peak ownership, native startup, close/async/full persistence/.3dm remain required. All full feature/RCORE/M9IMG gates and catalog statuses remain open.

## Selected Win32 File adapter and replay guards — 2026-10-14

[Current scoped evidence](../../validation/2026-10-14-owned-file-win32.md): 483 workspace/104 focused subsets; selected authored Win32 adapter exposes HRESULT errors and disables OS overwrite prompt for the existing owned confirmation.4 audit+3 native+1 UI-error checks;8 native File checks/3 real Windows cancels;10+13+6 regressions/7 cold children. Readiness guards prevent missing-modal events from editing CMD; original missed-click cause remains unknown. OS selection/save/open/overwrite/native startup, full budgets/close/async/persistence/.3dm and all full feature/RCORE/Matrix9 gates remain required.

## Owned window close — 2026-10-14

[Scoped close evidence](../../validation/2026-10-14-owned-file-close.md): 494 workspace/115 focused subset. Owned Save/Discard/Cancel controls window closing in normal UI and minimized logic; stale consent/save failure/chooser Cancel cannot approve shutdown. Native replay11 checks includes2 root WM_CLOSE requests and4 real picker cancels; original File8 and Layers10+13/viewport6 are separate overlapping regressions;7 cold children exit. Successful OS selection/save/open/overwrite/startup, native Save-success/Discard-button shutdown, complete teardown/budgets/async/persistence/.3dm and full gates remain required.

## Native File selection and startup — 2026-10-14

[Scoped native File evidence](../../validation/2026-10-14-owned-file-selection.md): 498 workspace/119 focused subset;3 real Windows Show/GetResult selections, alternate SaveAs/owned Replace adopts path only after acceptance with original/seed SHA256 preservation. Native actual --ui path clean startup3checks and cold report match. Success5/close11/File8/Layers10+13/viewport6 are separate overlapping replays;7 Job-owned cold children plus2 bounded CLI inspections. Missing-module native startup, native success/Discard shutdown, complete persistence/.3dm/async/budgets/disposal and all full gates remain open.
