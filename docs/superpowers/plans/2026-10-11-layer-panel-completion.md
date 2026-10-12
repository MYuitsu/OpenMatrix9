# Layer panel implementation plan

> **For agentic workers:** Use superpowers:executing-plans to implement this plan task by task in the current session.

**Goal:** Complete the documented Layers panel interactions while retaining document identity, transactions and independent visibility.

**Architecture:** Rust owns slot/action/visibility/name/capability policy. CoreLayers adapts FreeCAD groups, properties, selection and transactions. MatrixSidebar binds the Matrix gestures and keeps modal operations tied to the originating document.

**Tech Stack:** Safe Rust, scalar C ABI, C++ FreeCAD/Qt6.

**Spec:** OpenMatrix9_Codex_Spec_v1/specs/01-core/om9-layer-001-layer-management.md; user authorized continuation after reviewing its audit on 2026-10-11.

## Global constraints

- Preserve exact OM9 ID, private references and unrelated changes; no commercial assets.
- Build only OpenMatrix9Gui via build/build-om9-plugin.ps1 and the shared SDK. Select an isolated plugin output and profile; restore the prior cache output selection.
- No automatic new business-rule skill update without its existing consent procedure.
- User clarified the original interface has no footer combo/C A L V C (2026-10-11): omit them entirely.

## Review focus

- Locked/hidden objects cannot be picked or edited through the OM9 dispatcher; unlock restores prior Selectable.
- Layer Off/On preserves explicitly hidden members, including after save/reload.
- Assign rejects structural/foreign-document objects atomically and retains current layer.
- Modal rename/color/selective Show abort after document switch/close; cancellation is inert.
- Undo/Redo restores membership, names, colors, visibility and lock consistently.

## Tasks

1. Rust policy: extend layers.rs and layers tests with Lights as separate slot33, effective visibility/selectability, group action and bounded Advanced name validation; run failing then passing tests.
2. Native layer actions: extend CoreLayers.h/.cpp with assignSelected, selectMembers, hideSelected/reverse, showHidden/subset, setName, group visibility and selection protection. Store explicit member visibility/Selectable separately from effective state, and use existing transaction abort boundaries.
3. Panel: replace passive layer names with current-layer buttons; In arrow assigns; right-click swatch selects; Advanced Shift+right-click rename/color; enable Lights, Hide/Show and group controls; add an actual Advanced collapse button. Qt dialogs must revalidate document name/UID and selected identities before committing.
4. Common integration: protect locked selection at native dispatch, mouse and select-all; route supported new root Curve/Solid/Surface outputs through the layer adapter where their transaction contracts permit it. Keep semantic/import/History constraints explicit rather than silently relocating storage/source objects.
5. Tests: native layer-panel smoke (assignment/current separation, lock, group/object hiding, name/color gestures, selective Show cancel/commit, persistence and multiple documents); update Circle layer fixture for the corrected gestures. Build plugin only, run both matching-host fixtures and appropriate regressions, inspect actual screenshot and audit public source.
6. Synchronize spec, feature record, README, FEATURES JSON/YAML, IMPLEMENTATION_STATUS, manifest and live ledger to actual supported evidence. Omit the footer per the user's correction; unverified capability paths remain explicit.

Execution proceeds within the user's instruction to implement the reviewed missing parts. No commit/push is requested.
