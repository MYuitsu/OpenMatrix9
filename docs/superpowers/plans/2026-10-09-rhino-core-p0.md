# Rhino core P0 implementation plan

> **For agentic workers:** Execute the existing written RCORE contracts in bounded, independently tested increments. Use test-driven-development and verification-before-completion. Parallel investigators own disjoint source files; the primary worker integrates common native interfaces.

**Goal:** Implement and verify the missing P0 foundations in `specs/00-rhino-core`, preserving current supported commands and uncommitted work.

**Architecture:** Safe Rust owns typed units, coordinate parsing, capability and lifecycle decisions. Existing FreeCAD/Qt/OCCT adapters own document access, widgets, transactions and persistence. Native callbacks receive owned snapshots and reject stale state before mutation.

**Tech Stack:** Rust 2024, C++/Qt6/FreeCAD/OCCT, Python only for host acceptance macros and evidence tooling.

**Spec:** `OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/README.md`, chapters 01–05, P0 portions of 06–07, 09–10. P1/P2 tools remain outside this request.

## Global constraints

- Keep exact feature IDs, supported slices, source-derived contracts and license notices.
- Do not replace existing numerical thresholds with new document defaults.
- Preview must not mutate the document. A failed/cancelled operation must not create geometry or empty Undo records.
- Native geometry, render caches, retained archive payloads and instance definitions are different representations.
- Preserve all existing working-tree changes. Current uncommitted native features are the implementation baseline; an empty HEAD worktree would omit them.
- No publication, push, commercial reference copying or unrelated changes.
- Full native and Rhino compatibility may only be claimed with corresponding runtime evidence. A tested foundation does not complete every capability in its chapter.

## Review focus

- NaN/Inf, dimensional mismatch and conversion overflow must fail before native mutation.
- Translated/rotated CPlanes and relative/world input must apply transforms once.
- Cancelled/failed commands must not consume one-shot snap or overwrite the last repeatable success.
- Context/source changes between preview and commit must reject stale work.
- Saved metadata, history dependencies and edited geometry must survive reload without resurrecting the original archive state.

## Task 1: Units and document context (primary worker)

Files: new `rust/src/units.rs`, `rust/tests/units.rs`, `Gui/CoreUnits.{h,cpp}`; registration in `lib.rs`, `Gui/CMakeLists.txt`, `Gui/AppOpenMatrix9Gui.cpp`; narrow `CurveController` integration.

- [x] Test dimension exponents, mm/inch, angle independence, invalid/custom units, overflow and both unit-change modes.
- [x] Implement versioned context validation and explicit length parsing in Rust. Native metadata serialization must validate before opening a transaction.
- [x] Expose native context get/set and ensure input session snapshots cannot commit after context changes.
- [ ] Test native transaction rollback, Undo/Redo and FCStd cold restore. Unknown dimensional scaling remains rejected until an adapter proves coverage.
  - Checkpoint: Pre-transaction rejection, Undo/Redo and cold restore pass. Failure injection after transaction opening remains untested.

## Task 2: Shared coordinates (coordinates worker)

Files: new `rust/src/coordinates.rs`, coordinate tests; existing `rust/src/curve.rs`.

- [x] Reproduce missing `w`, `wr` and `@` forms using RCORE-03.T01–03 exact vectors.
- [x] Implement typed shared parsing, finite/right-handed frames, relative-base requirements and state-preserving errors.
- [ ] Verify all existing shared point-input families; preserve the documented family-specific latch policy and identify unsupported CPlane operations explicitly.
  - Checkpoint: Line/Polyline, Rectangle, Box/Sphere and the Circle menu/latch slices are tested; complete all-family acceptance remains open.

## Task 3: Command and picking lifecycle (command_snap worker)

Files: `rust/src/core_snaps.rs`, `state.rs`, `ffi.rs`, relevant tests; `Gui/CoreSnaps.{h,cpp}`; coordinated `CurveController` hooks.

- [x] Reproduce one-shot consumption and repeatability gaps.
- [x] Separate persistent, one-shot and suspended snap state; consume one-shot only after accepted input.
- [x] Separate last repeatable successful command from icon history; recheck native availability on repeat.
- [x] Verify cancel/cleanup, invalid click, successful pick and excluded commands; inspect native selection permissions.

## Task 4: History and persistence (history_io worker)

Files: history/retained archive Rust/native modules and focused acceptance macros, selected after inspecting existing contracts.

- [x] Audit actual RCORE-09/10 behavior against existing native evidence; reproduce missing safeguards before changing code.
- [ ] Implement concrete cycle/revision/schema/current-state safeguards with host integration.
  - Checkpoint: Cycle, retained-schema and stale-state guards are implemented; common durable revisions and current-state export remain open.
- [ ] Verify errors are atomic and dependencies, metadata and geometry survive Undo/Redo and reload.
  - Checkpoint: The listed History/Builder/Circle/retained fixtures pass; common all-family revision and persistence contracts remain open.
- [x] Preserve explicit unsupported Rhino/plugin/clipboard combinations; record exact remaining evidence needs.

## Task 5: Representation and NURBS foundation (primary worker)

- [x] Audit supported type/backend gates and basis validation for existing modeling adapters.
- [ ] Add shared Rust validation only where a native consumer is integrated; test malformed basis, weights, knots and checked grid sizes.
  - Checkpoint: Generated-spline publication validation is integrated. Rational Basis checks have unit evidence only; a common checked-grid-size gate is not implemented.
- [x] Add native fixtures for rational curve, trimmed hole, open periodic face and display/CAD separation.

## Task 6: Integration and evidence

- [x] Run targeted Rust tests then full suite, changed-file formatting and lint; establish baseline failures separately.
- [x] Build matching SDK native module and run new/affected GUI macros and required cold reload checks.
- [x] Review integrated changes for ownership, ABI, lifecycle and regressions.
- [x] Synchronize relevant RCORE specs, implementation status, README, manifest and progress ledger with exact passed checks and remaining gaps. Never mark a whole chapter complete from a narrow slice.

## Execution record

- User confirmed **only P0** on 2026-10-09. Existing specs are the design authority and implementation is already authorized.
- Initial full Rust baseline: **256 passed across 41 suites**, exit 0.
- Ruling: work in the current nested checkout to preserve the substantial uncommitted baseline and configured SDK; do not reset or copy only HEAD to a worktree.
- The obsolete route points to `ref/matrix9`; actual spec package is `OpenMatrix9_Codex_Spec_v1` and is used throughout.

- Completed checkboxes cover this bounded implementation plan only; entire P0 remains open as enumerated in the validation report. Native test/build details are authoritative there.
