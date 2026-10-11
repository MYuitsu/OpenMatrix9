# Gumball Four View Completion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox syntax for tracking.

**Goal:** Implement the approved OM9-INFO-020/021/022 revision with one shared Gumball session, four native overlays and Rhino reference handle layout.

**Architecture:** Safe Rust owns settings validation, projection visibility, scale anchors and pointer policy. Qt/Coin observes cameras and draws one overlay per native view. Existing FreeCAD geometry publication retains one guarded transaction per gesture.

**Tech Stack:** Rust, C ABI, C++/Qt6, Coin3D, OCCT, native QTest macros.

**Spec:** [020](../../../OpenMatrix9_Codex_Spec_v1/specs/01-core/om9-info-020-gumball.md), [021](../../../OpenMatrix9_Codex_Spec_v1/specs/01-core/om9-info-021-gumball-alignment.md), [022](../../../OpenMatrix9_Codex_Spec_v1/specs/01-core/om9-info-022-relocate-gumball.md).

## Global Constraints

- Existing in-place checkout choice and the user's instruction to implement the approved specs are preserved; execution is inline.
- Only three core specs; no Smart Targets, Clayoo/SubD, viewport renaming or background changes.
- Profile defaults: Off, CPlane, Smooth, Drag Strength 100%.
- X red, Y green, Z blue. Move/Rotate 80 logical px; Scale 8; Origin 4; Plane 20 at 50 on each axis; Menu 10 at (-30,+30); pick radius 5.
- Rust owns portable math/state. Native code owns host observations, Coin primitives, Qt widgets and native geometry mutation.
- Reuse H:/FreeCAD-src/build/relWithDebInfo and build/build-om9-plugin.ps1; never build the FreeCAD root.
- Preserve foreign dirty work, source references, live models and the existing user demo. Scope every commit to owned files.
- Capability failures precede publication; do not overwrite parametric Shapes or shared Link definitions, drop shear, or claim unsupported host representations work.

## Review Focus

1. First press in an inactive viewport starts the requested handle and publishes once.
2. Closing the gesture view cancels all previews; closing a passive view preserves the session.
3. Outside/zero-extent pivots preserve factor=1 despite the display fallback and DPI changes.
4. Idle CPlane changes update all views; relocated frames survive focus changes; gestures freeze their coordinate context.
5. Parametric/Link scaling survives recompute, Undo/Redo and stale dependency callbacks without modifying a shared definition.

---

### Task 1: Rust presentation and pointer policy

**Files:** Create `rust/src/gumball_view.rs`, `rust/tests/gumball_view.rs`; modify `rust/src/gumball.rs`, `rust/src/gumball_ffi.rs`, `rust/src/lib.rs`, `Gui/GumballRust.h`.

**Interfaces:**
- Consumes existing `Frame`, `GumballSession`, `PointerRay`, `GumballEvent` and source records without changing their ABI.
- Produces `ViewSettings::default()`, `ViewSettings::validate() -> Result<(), GumballError>`, `layout(Frame, &[Point], ViewProjection, ViewSettings) -> Result<ViewLayout, GumballError>`.
- Produces `om9_gumball_settings(id, input, output)`, `om9_gumball_layout(id, projection, points, count, output)`; separate C records retain existing ABI sizes.
- Settings are session-owned and copied through the ABI; layout output has stable IDs, visibility and three world Scale anchors.

- [x] Write tests for World Top `{0,1,3,6,9,10,11,13}`, Front `{0,2,5,6,8,10,12,13}`, Right `{1,2,4,6,7,11,12,13}`; a rotated frame uses projection rather than view names. Box [0,0,0]..[12,8,6], pivot [6,4,3] yields anchors [0,4,3], [6,0,3], [6,4,0]. Defaults assert the exact pixel values above. NaN/negative/zero strength rejected atomically, zero group sizes hide the group. Fallback has at least 12 projected pixels while factor begins at 1.
- [x] RED: `rtk cargo test --locked --release --manifest-path rust/Cargo.toml --test gumball_view` fails for the missing API.
- [x] Implement validated settings, basis extents, projection visibility and display-only fallback. Add pointer strength to unconstrained movement/rotation and deviation from Scale factor 1. Add a snapped-world motion event constrained by the captured operation/frame; numeric input ignores strength.
- [x] GREEN: the command above passes; existing `gumball_session` and `gumball_ffi` tests pass unchanged.
- [x] Included in the single scoped Gumball checkpoint after the final native cohort; individual intermediate commits were consolidated.

### Task 2: Four native views and source ownership

**Files:** Modify `Gui/GumballController.h/.cpp`; create `tests/gumball_four_view_smoke.FCMacro`.

**Interfaces:**
- Consumes Task 1 settings/layout ABI and existing `GumballOverlay` rendering/editor interface.
- Produces a collection of `QPointer<View3DInventor>` plus owned overlays. `view`/`overlay` identify only the current input source; one Rust session and one captured source batch remain.
- Extends `gumballState()` with a `views` list (slot, diagnostics, shared frame); preserves old `overlay` diagnostics and active-view `gumballHandles()`.

- [x] Native RED asserts four scene attachments immediately after Command On, first-click numeric Move in each inactive view, shared frame and one Undo entry after each accepted gesture. On the accepted baseline it fails at four attachments.
- [x] Run RED on the accepted plugin with `rtk proxy powershell -File H:/FreeCAD-src/build/run-om9-plugin.ps1 -PluginRuntime H:/FreeCAD-src/build/gumball-rhino-runtime-20261011 -Macro tests/gumball_four_view_smoke.FCMacro -OutputDirectory H:/FreeCAD-src/build/shared-host-tests/gumball-four-view-red-20261011`.
- [x] Implement `syncViews()`, `overlayFor(View3DInventor*)`, `clearOverlays()` and `clearPreviews()`. Resolve receiving viewport before refresh; activate its MDI window and hit in the same event. Stage preview once, fan out the line observations. Do not recapture World/Object frame for camera-only changes; refresh idle CPlane context and preserve relocated override. Cancel source-view close/navigation and stale context; passive close only prunes its overlay.
- [x] Build only the plugin into `H:/FreeCAD-src/build/gumball-four-view-runtime-20261011`. GREEN repeats the native macro with that runtime, plus lifecycle/publication regression fixtures.
- [x] Included in the single scoped Gumball checkpoint after the final native cohort; individual intermediate commits were consolidated.

### Task 3: Rhino reference primitives and picking

**Files:** Modify `Gui/GumballOverlay.h/.cpp`, `Gui/GumballGeometry.h/.cpp`; modify `tests/gumball_visual_smoke.FCMacro`, new four-view macro.

**Interfaces:**
- Consumes Task 1 `ViewLayout` and settings, Task 2 per-view attachments.
- Produces `gumballExtentPoints(inputs, frame) -> vector<array<double,3>>` native world extent observations. BRep bounds are measured in the current Gumball basis; Mesh/Points are measured from native points.
- `render(snapshot, settings, extentPoints)` builds thin arrows, quarter arcs, outlined Scale/Origin, two-color Plane and white outline menu ring with unpickable dotted guides. Projection and anchor admission come from Rust.

- [x] RED native assertions: Top/Front/Right handle sets, Scale on independently measured geometry bounds, menu offset (-30,+30), profile dimensions at zoom 0.5/1/2 and DPI 1/1.5/2. Hidden handles have no native hit paths; guides do not affect fitAll.
- [x] Run the macro with Task 2 binary: expect a layout/anchor failure.
- [x] Render in world units per logical pixel, keep only enabled nondegenerate handle branches, and use five logical pixel native pick radius. Query diagnostics report projected anchors/dimensions. Replace bounded central target scanning with handle-specific projected sampling so distant Scale grips remain testable.
- [x] Build plugin, rerun four-view and visual macros at the three DPI settings. Inspect actual Coin renders of all four views.
- [x] Included in the single scoped Gumball checkpoint after the final native cohort; individual intermediate commits were consolidated.

### Task 4: Core menu, settings and snapping

**Files:** Modify `Gui/GumballController.h/.cpp`, `rust/src/gumball.rs`, `rust/src/gumball_ffi.rs`, `rust/tests/gumball_session.rs`; extend native macro.

**Interfaces:**
- Consumes session settings ABI, native `CoreSnaps::pick` and camera orientation/position/focalDistance.
- Produces native menu actions Snappy/Smooth, Drag Strength, Settings, Rotate View Around Gumball, Restore Defaults. Settings persist only in the existing Gumball parameter group and validate in Rust before saving.
- Orbit uses native NavigationStyle rotation center at the shared pivot, preserving camera orientation and eye position; source camera only, no geometry transaction.

- [x] RED Rust asserts 50% pointer Move gives half the distance, Scale 2 becomes 1.5, numeric 2 stays 2, snap candidates project to the active axis/plane. Native RED exercises menu checked state, invalid strength, group size zero, Restore Defaults, popup release, source-only orbit and unchanged Undo.
- [x] Run targeted Rust tests and the native macro; verify intended failures.
- [x] Implement compact native dialogs for strength and all documented sizes/colors/line widths/positions. Menu actions use the same session settings. During Snappy pointer input use active Osnap candidates; Smooth uses the raw ray. Reject settings changes during gestures. Keep unsupported snap kinds explicit through existing CoreSnaps capability state.
- [x] GREEN targeted tests and native menu/gesture tests. Alt tap, held numeric, Shift Scale3D and Ctrl timing remain regression requirements.
- [x] Included in the single scoped Gumball checkpoint after the final native cohort; individual intermediate commits were consolidated.

### Task 5: Native owner capabilities

**Files:** Modify `Gui/GumballGeometry.h/.cpp`; create `tests/gumball_owner_scale_smoke.FCMacro`.

**Interfaces:**
- Consumes the existing staged affine matrix/source/dependency snapshots and publication guard.
- Produces native property adapters for representable parametric Box/Sphere scaling and individual Link occurrence ScaleVector/Placement. Each adapter validates representability before opening the transaction; unrepresentable shear or owner kinds produce a specific capability error.
- Object alignment consumes native planar oriented Mesh Segment facet observations; Rust validates the unique component frame. Mesh edge/vertex/light/camera and ambiguous/unsupported observations preserve the previous alignment with an explicit capability error.

- [x] RED accepts Box scaling through native dimensions, uniform Sphere scaling through Radius and Link scaling through occurrence properties; source definition unchanged; recompute and Undo/Redo retain exact independent bounds. Sheared Link and incompatible parametric directions reject with no new Undo or owner mutation. Test Mesh component Object alignment and its degenerate rejection.
- [x] Run native macro on Task 4 binary; expect owner rejection where an adapter is missing.
- [x] Implement guarded property publication with expected-value verification and one recompute. Preserve source identity, shared definitions and dependencies. Do not introduce a new associative History graph or silently replace native owners. Record host types that cannot represent the requested affine transform; leave corresponding full-parity acceptance open rather than falsifying completion.
- [x] GREEN owner macro, publication callback regressions, warm/cold persistence and full native Gumball suite.
- [x] Included in the single scoped Gumball checkpoint after the final native cohort; individual intermediate commits were consolidated.

### Task 6: Evidence, status and final review

**Files:** Update the three exact feature records/specs, README Gumball rows, selected manifest hashes and the live progress work item; create `docs/validation/2026-10-11-gumball-four-view-completion.md`.

**Interfaces:** Consumes exact runtime SHA, suite reports and independently inspected rendered images; produces reviewable acceptance coverage and honest remaining host limits.

- [x] Run all Rust tests, native Gumball core/persistence/regression suite, new four-view/owner/DPI tests and `rtk proxy python tools/public_source_audit.py`. Read actual exits and reports; distinguish test plumbing from parity checks.
- [x] Read back specification coverage against each open acceptance item; mark only proven items. Reconcile metadata without overwriting concurrent Curve/RCORE edits.
- [x] Request one fresh final reviewer under executing-plans, supply scoped changes, specs and exact evidence. Fix Important/Critical findings with RED/GREEN tests; record minor rulings.
- [x] Run scoped diff checks and commit the evidence. Apply the user's version policy only for actually completed tasks/specs; do not increment bb for unresolved full-parity requirements.

## Self-review and execution record

Coverage maps four-view/lifetime behavior to Task 2, visual/projection/extents/DPI to Tasks 1/3, numeric/modifiers/settings/snap/orbit to Tasks 1/4, source/owner/history/persistence to Task 5, and every final claim to Task 6. Existing publication, coordinate and cache contracts are retained. Interface records are additive to the old ABI. The prior explicit in-place execution choice and current instruction to implement the approved specs authorize inline execution; no new approval is required for this implementation plan.

## Final execution outcome

Tasks1–6 are complete within the declared supported native scope. The final
profile cohort passed29processes/781checks, including Mesh Segment cache and
metadata retention and existing Part::Cut dependency recompute/Undo. Rust
passed874checks/125suites; focused Gumball45/5. Evidence and unresolved full
original-host parity gates are recorded in the three exact feature specs and
the [validation report](../../validation/2026-10-11-gumball-four-view-completion.md).
No full-spec bb increment or release claim is made for the open owner/frame gates.
