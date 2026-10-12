# Phase 3 Rust BRep Modeling Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan inline, task-by-task. Steps use checkbox (`- [x]`) syntax for tracking.

**Goal:** Complete P3.1–P3.3 and APP-GATE-P3: imported native CAD → cutter/Boolean → new Loft/Sweep detail → self-contained FCStd → current selected V5 → actual Rhino5 → OM9 reread.

**Architecture:** Selectively integrate existing Rust surface/edit sessions and native adapters into an isolated extension of the accepted optimized Phase1/2 source. Rust owns classification, validation and request lifecycle; native adapters capture current world-space geometry, call the kernel and transact on the GUI thread.

**Tech Stack:** Safe Rust, existing C++23/Qt/FreeCAD/OCCT/openNURBS bridge; PowerShell and Rhino/FreeCAD host test glue.

**Spec:** [Approved written design](../specs/2026-10-09-phase3-rust-brep-modeling-design.md).

**Authorization:** User response `okie triển khai khôi` approves the presented written design and explicitly requests implementation. Execute inline as requested; this plan makes the existing P3.1–P3.3 execution concrete, without adding a new feature scope or another authorization prompt.

## Global Constraints

- Isolated source `H:/FreeCAD-src/build/om9-phase3-dev`; runtime `H:/FreeCAD-src/build/om9-phase3-sdk`; separate module/native build directories. Accepted Phase1/2 source and binaries stay immutable except progress documentation.
- Rhino5 V5 target, pinned openNURBS `eb92af3ba1806b0a34a99aba0d3bda83e3d46083`.
- Ring bounds 0.001mm; area/volume `max(0.001, 0.0001 * abs(reference))`; independent analytical fixtures specify their own tolerances.
- Current/new geometry only; no silent original snapshot restoration; mesh is not editable CAD; no mesh/cloud point enumeration for command availability.
- History/render excluded; Undo/Redo/Cancel/stale/read-only/task guards and source-deleted FCStd included.
- Worker budget `max(1, floor(0.60 * logical CPU))`, limited by task/RAM. No document/widget pointers in workers.
- No commit/push/primary integration/Git repair. Retain snapshots, hashes, exact diffs, failed reports and the execution ledger.

## Review Focus

- Trim on holed/complementary/seam/pole faces keeps the intended region and topology; Task3 covers it.
- Open shells never enter closed-solid Boolean, invalid/empty result never deletes inputs; Tasks1/3 cover it.
- Nested transformed inputs apply parent/world placement once; Task3 covers it.
- Geometry/placement/deletion/document change while preview dialog is open invalidates request before commit; Tasks1/3 cover it.
- Curve Join remains the accepted Phase2 path, face Join uses the Phase3 adapter, mixed selection rejects before mutation; Tasks1/3 and final Phase1/2 replay cover it.

### Task 1: Rust operation policy and owned requests

**Files:** Create `rust/src/phase3_modeling.rs`, `rust/tests/phase3_modeling.rs`; modify `rust/src/lib.rs`, `Gui/RustBridge.h`; create small Phase3 FFI adapter if needed.

**Interfaces:** `capability(Operation, &[InputFacts]) -> Result<(), Reason>` consumes independent native topology facts; `Request` owns document/generation/input identities and signatures; `validate_current(...) -> Result<(), Reason>` rejects stale inputs and invalid lifecycle without a kernel call. C ABI accepts bounded, caller-owned snapshots for the duration of each call; no panic crosses it.

- [x] Write tests for valid solid Difference, open-shell rejection, mixed/invalid/preservation-protected input rejection, face/curve Join routing, numeric options, exact stale/Cancel transitions and zero heavy geometry data requirements.
- [x] Run `rtk cargo test --manifest-path rust/Cargo.toml --test phase3_modeling`; observe named missing behavior RED.
- [x] Implement safe Rust policy/request ownership and bounded native ABI adapters; keep kernel and GUI operations native.
- [x] Repeat the same tests GREEN and existing Rust tests; record evidence and native exceptions in the ledger.

### Task 2: P3.1 converter fidelity on the new baseline

**Files:** Create `tests/native/three_dm_modeling_brep.cpp`, `tests/modeling_brep_smoke.FCMacro`; modify `tests/native/CMakeLists.txt`; only change `Gui/ThreeDmBrep.cpp`, `ThreeDmSolidShells.cpp`, `ThreeDmTrimMapping.cpp` on a reproduced error.

**Interfaces:** Existing `importBrep`/`exportBrep` with Task1 native topology facts. Native target `ThreeDmModelingBrepTests`, CTest `OM9-MODELING.Brep`.

- [x] Add independent hole/cylinder-seam/sphere-pole/torus/cavity/open-shell assertions, analytic mass/bounds/topology, edited cylinder common UV basis and pcurve checks.
- [x] Build/run the new target in `om9-phase3-native`; a reused passing converter is recorded as baseline, never rewritten to fabricate RED.
- [x] Correct only reproduced conversion errors, then run named tests and existing seam/pole/domain/multishell regressions GREEN.
- [x] Run the actual FreeCAD BRep macro on the matching Phase3 runtime and bind source/module/fixture hashes.

### Task 3: P3.2 Surface/Edit integration and current input lifecycle

**Files:** Integrate selected `Gui/Surface*`, `Gui/Edit*`, `rust/src/surface.rs`, `rust/src/surface_refit.rs`, `rust/src/edit.rs` and corresponding tests from main; merge `Gui/Command.cpp`, `Workbench.cpp`, `NativeCommands.h`, `RustBridge.h`, CMake lists, `rust/src/lib.rs` and CMD routing. Create `tests/modeling_surface_edit_smoke.FCMacro`.

**Interfaces:** Reuse `buildSurface/commitSurface`, `editInput/verifyEditInputs/buildEdit/commitEdit`; supplement surface input snapshots with Task1 owned signatures. Menu/CMD/mouse share the same controller. Native facts are checked through Task1 before kernel operations and again before transaction commit.

- [x] Establish imported rational Loft/Sweep, face Trim/Join/Explode, closed-solid Difference, nested placement and stale/failure/read-only/task/Cancel host regressions; observe missing adapter or promised behavioral assertion RED.
- [x] Merge dependency closures selectively, preserve accepted curve Join/editor/snap/clipboard; disable history hooks not required for the workflow rather than introducing unrelated history behavior.
- [x] Move portable decisions/state/options into Rust; retain Qt/native shape/GIL bridges only. Add stale surface signature validation and closed-solid policy before every relevant mutation.
- [x] Run Rust surface/edit tests, native build/module activation, actual menu/CMD/mouse and lifecycle tests GREEN; add regressions for any additionally activated Boolean operation.

### Task 4: P3.3 ring workflow and actual Rhino gate

**Files:** Create `tests/modeling_ring_workflow_smoke.FCMacro`, `tests/native/three_dm_modeling_workflow_oracle.cpp`, `tests/rhino5_verify_phase3.py` and owned Rhino launcher/reimport glue; modify phase runner and manifest/evidence helpers for Phase3.

**Interfaces:** Consume Tasks1–3 commands, accepted CAD snap and current selected export. Manifest `phase3-build.json` binds source/runtime; application evidence binds fixture/output hashes, owned PID/exit and script immutability.

- [x] Add independently redistributable ring/torus cutter/cut/detail oracle and failing actual-command workflow; private user ring is supplemental only.
- [x] Exercise snap → cutter → Difference → new Loft/Sweep detail, Undo/Redo each step, delete source, FCStd close/reopen in a fresh process, edited+new selected V5 export.
- [x] Run actual isolated Rhino5 Open/SaveAs/Export Selected, then OM9 reread those exact outputs; keep the caller user's open document untouched. Log actual command durations separately from session/oracle durations.
- [x] Run the complete prepared Phase3 application matrix on one final matching binary; preserve failed attempts and all requirements without replacing geometry checks with object counts.

### Task 5: regression, review and completion audit

**Files:** `docs/validation/modeling-phase-3/{requirements,summary}.json`, final review, progress/roadmap/README, phase application acceptance.

- [x] Run fresh relevant Phase1/2 native/Rust/host/clipboard regressions on the final Phase3 binary; formatting/Clippy/release and ABI/native build checks pass.
- [x] Dispatch one independent whole-change reviewer with the approved spec, this plan, exact diff and ledger; fix Critical/Important findings with RED→GREEN and fresh suite evidence.
- [x] Audit all 18 requirement groups against actual artifacts; uncertain or partial evidence remains incomplete. All named original P3.1–P3.3 deliverables and APP-GATE-P3 must be satisfied.
- [x] Synchronize acceptance, progress, roadmap JSON/Markdown and README; report actual scope, prepared batches, seven broader packages and unknown total future batches. Provide the user Rhino entry command. Mark the goal complete only after the full audit passes.

Final evidence: ../../validation/modeling-phase-3/summary.json. All18 groups verified within approved modeling scope, isolated runtime retained.
