# Curve Commands Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox syntax for tracking.

**Goal:** Implement Matrix9 Curve commands with one shared handler for menu/mouse and CMD, beginning with Polyline and Line and continuing through the 58 feature IDs.

**Architecture:** Rust owns command lookup, parsing, options, session state and validation. Native C++/Qt routes viewport events and CMD submission to that session; a FreeCAD Part adapter applies validated geometry in document transactions.

**Tech Stack:** Rust static library, C++23, Qt6, existing FreeCAD SDK and Part/OpenCascade.

**Spec:** `docs/superpowers/specs/2026-10-04-curve-command-design.md` (approved).

## Global Constraints

- Use the existing `ref/matrix9/OpenMatrix9_Codex_Spec_v1/specs/02-curve` specifications. Do not reread PDFs or require PDF verification.
- Keep original Matrix9 public command names and defer handler renaming/refactoring until Curve is stable.
- Mouse and CMD call one implementation. Test geometry, lifecycle, Undo/Redo and persistence, not merely availability.
- Preserve existing workspace commands, menu ordering, icon keys and unrelated user changes.
- Record unspecified defaults and host behavior as OpenMatrix9 decisions; do not claim they were recovered from Matrix.
- Native execution in the current chat is the proposed execution method; no subagent dispatch is necessary for implementation.

## Review Focus

- Document closure or switching during input must cancel preview and cannot commit into another document.
- Invalid coordinates, nonfinite numbers and coincident points must not create corrupt geometry or lose the current session.
- High DPI and nondefault viewport orientation must map mouse coordinates correctly to the active construction plane.
- Workbench switches must not duplicate the CMD dock or leave callbacks attached.
- A menu click starts a session; successful-only history must wait for commit and ignore cancel/error.

## Task 1: Shared Rust Curve session

**Files:** create `rust/src/curve.rs`, `rust/tests/curve_session.rs`; modify `rust/src/lib.rs`, `rust/src/ffi.rs`, `Gui/RustBridge.h`; create `docs/features/OM9-CURVE-001.md` and `OM9-CURVE-002.md`.

**Interfaces:** `CurveSession::start(name: &str) -> Result<(), CurveError>`, `input(text: &str) -> Result<CurveEffect, CurveError>`, `point(point: [f64; 3], modifiers: u32) -> Result<CurveEffect, CurveError>`, `cancel()`. `CurveEffect` distinguishes waiting, preview, commit and cancel; commit contains typed line/arc segments, never executable code. FFI exposes matching fixed-layout input/effect structures, caller-owned buffers and bounded error text, with no Rust panic crossing the ABI.

- [ ] Read the two local specs and document their selection, options, commit/cancel, units, defaults and tolerance contracts. Use millimetres internally with host conversion; document chosen values where the specs are silent. List unsupported options explicitly rather than silently ignoring them.
- [ ] Write failing semantic tests for original command lookup, two-point Line, Polyline Enter/Close, invalid/coincident/nonfinite inputs, Esc cancellation and session restart. A Line from `(0,0,0)` to `(3,4,0)` must produce one segment of length 5; a closed Polyline through `(0,0,0)`, `(10,0,0)`, `(10,10,0)` must return to its initial vertex.
- [ ] Run `cargo test --manifest-path rust/Cargo.toml --test curve_session`; confirm the missing session behavior fails before implementing it.
- [ ] Implement the interfaces above and their ABI. Keep original command text separate from icon identity and existing workspace IDs. Reject unsupported options with a visible explanation while retaining the session.
- [ ] Run the full Rust suite, fmt and Clippy; require all tests and lint to pass.

## Task 2: Native CMD, viewport input and geometry commit

**Files:** create `Gui/CurveController.h`, `Gui/CurveController.cpp`, `Gui/CurveGeometry.h`, `Gui/CurveGeometry.cpp`; modify `Gui/Command.cpp`, `Gui/Workbench.cpp`, `Gui/CMakeLists.txt`, `cmake/StandaloneSDK.cmake`; create `tests/curve_smoke.FCMacro`.

**Interfaces:** `CurveController::start(std::size_t command) -> bool`, `submit(const QString&)`, `cancel()`, `activate()`, `deactivate()`. The controller consumes Task 1's effects. `CurveGeometry::commit(App::Document&, const CurveEffect&)` creates native Part geometry under one transaction or aborts on failure. The controller records history only after successful commit.

- [ ] Write the native smoke fixture before implementation. Through Qt input, click Curve's Line button and pick two viewport points; submit `Line` and the equivalent coordinates through the CMD widget. Assert one edge and equivalent endpoints/length in both resulting objects. Check no document, Esc, Undo/Redo and no history insertion at invocation.
- [ ] Run the fixture through `tests/run_menu_smoke.ps1 -Macro curve_smoke.FCMacro`; confirm failure because Curve/CMD is not implemented, preserving its report.
- [ ] Add a single `OM9CommandFrame` dock with `OM9CommandInput` and prompt/output widgets. Enter sends input; Esc cancels. Route both existing Curve menu icon indices and typed original names to Task 1's session. Never evaluate entered text as Python or shell.
- [ ] Add viewport callbacks through `View3DInventorViewer::addEventCallback` and remove them on cancellation/deactivation/destruction. Convert cursor picks using the active camera and construction plane. Preview remains transient and has no undo entry.
- [ ] Add the SDK's Part import library and necessary OpenCascade/Coin headers/libraries after checking matching paths. Build native line/arc wire shapes; validate before assignment to a Part feature, abort transaction on error, recompute and commit once.
- [ ] Extend native tests for document switch/closure, three workbench switches, nondefault camera plane, invalid inputs and host task restrictions. Confirm no duplicate docks, orphan previews or commits into the wrong document.
- [ ] Build `OpenMatrix9Gui` in `D:/FreeCAD-src/build/openmatrix9`, run the Curve fixture, and compare typed/mouse geometry. Inspect actual screenshot artifacts and label Qt event simulation separately from manual desktop interaction.

## Task 3: Complete Polyline and Line options

**Files:** extend Task 1/2 files and tests; update their feature contracts and validation report.

**Interfaces:** extend the typed session inputs/effects with the selection and geometric constraints required by each local spec. Changes must stay synchronized between Rust definitions, C ABI and C++ callers.

- [ ] Add failing tests for each option in `OM9-CURVE-001` and `OM9-CURVE-002`, using the local specs as the input checklist: Polyline Line/Arc switching, Close/PersistentClose, Length, Direction/Center and Helpers; Line BothSides, Normal/IgnoreTrims, Angled, Vertical, FourPoint and Bisector. If adding other previously observed options, distinguish them from this spec checklist in the feature contract.
- [ ] Implement one option at a time through the shared session and native adapter. Test each enabled option through CMD and viewport input where applicable; options that remain unsupported retain PARTIAL status.
- [ ] Verify BothSides around `(0,0,0)` with picked endpoint `(3,4,0)` produces endpoints `(-3,-4,0)` and `(3,4,0)`, length 10. Verify vertical lines align with construction-plane normal, normal lines with face normal, and angle/bisector constraints by vector measurements.
- [ ] Verify mixed straight/arc Polyline topology, endpoint continuity, closed state, cancellation and single-step Undo/Redo. Save/reload FCStd and compare geometry, object ownership and source-object preservation.
- [ ] Run full Rust checks and native Curve regression at 100%, 150% and 200%. Run existing `workspace_smoke.FCMacro` and `menu_smoke.FCMacro`. Write `docs/validation/2026-10-04-curve-commands.md` and update the progress ledger with exact results and remaining gaps.

## Task 4: Continue remaining Curve commands in order

**Files:** create exact-ID feature contracts/tests for `OM9-CURVE-003` through `OM9-CURVE-058`; extend the session and geometry adapter only as each feature requires.

**Interfaces:** keep Task 1's resolver/effect pipeline and Task 2's controller/transaction boundary. Additional native geometry operations are typed effects, not separate CMD implementations.

- [ ] For each next feature, read only its local spec, normalize its contract and add exact geometry/option/lifecycle tests before code. Use the catalog's existing names; resolve entries with missing names or command-plus-option sequences through local recovered references without reading PDFs or inventing public commands.
- [ ] Observe the failing tests, implement the feature and confirm semantic plus both native input paths pass. Repeat through ID 058; do not infer completion from a menu icon or catalog checkbox.
- [ ] Run the current Curve regression after each group and existing workspace/menu regression after integration changes. Record PASS/PARTIAL/FAIL per feature and per input path, with unexecuted cases explicitly untested.
- [ ] Refactor internal handlers only once Curve behavior is stable, preserving public names and rerunning the same acceptance tests.

## Plan review

This plan preserves all 58 commands as the intended scope. Tasks 1–3 establish the first usable and verifiable group; Task 4 is repeated per exact spec and does not claim that later features are already implemented or fully specified here. Product code has not changed. Review this plan before execution, as required by writing-plans.
