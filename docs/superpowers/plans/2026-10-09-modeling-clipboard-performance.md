# Modeling clipboard and performance implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:executing-plans, inline execution with one final fresh reviewer.

**Goal:** Finish native Rhino 5 ↔ OM9 geometry Copy/Paste and measured optimization using at most 60% available CPU threads.

**Architecture:** Transfer validated V5 archives through the Windows Rhino clipboard format. Keep openNURBS reading, graph expansion and SDK conversion serial; detach independent OCCT face assemblies for a bounded native pool, then join stable results and stage. Prepare jobs away from the GUI, bind objects and publish clipboard on the GUI thread.

**Tech Stack:** Rust policy, C++23/OCCT/openNURBS/Qt6, FreeCAD Python, Win32 clipboard.

**Spec:** ../specs/2026-10-09-modeling-phase1-clipboard-performance-design.md

## Global Constraints

- Both directions, current selected geometry only; no history or render.
- Target workers=max(1,floor(0.60*N)); affinity-aware N, unknown→1; effective bounded by tasks and RAM. No fixed four-thread cap.
- 512 MiB payload maximum; validate before clipboard publication or document transaction.
- Preserve accepted Phase 1 runtime/evidence. Candidate runtime: H:/FreeCAD-src/build/om9-clipboard-sdk.
- No source commits, pushes or integration requested. Preserve a source snapshot instead.
- Native workers never touch documents, Python objects, GUI or clipboard; all openNURBS calls remain serial.
- One transaction per Paste; Copy does not mutate document. Preserve text shortcuts.

## Review Focus

- Clipboard changes/busy/rejected publication must preserve old data and document.
- Async completion after document close/switch must not write into another project.
- Partial native worker failure/cancellation must join and clean staging, never partially bind.
- Block/reflection/unit transforms and stable member order must match serial geometry.
- Huge Mesh/Cloud availability checks must avoid point iteration and memory amplification.

### Task 1: Policy and safe parallel native preparation

Files: rust/src/modeling_exchange.rs; rust/tests/modeling_exchange_policy.rs; Gui/ThreeDmBrep.cpp; Gui/ThreeDmGeometry.h; Gui/ThreeDmArchive.{h,cpp}; Gui/ThreeDmModeling.cpp; Gui/ThreeDmPython.cpp; Gui/ThreeDmThreadPool.{h,cpp}; cmake/OpenNURBS.cmake.

Interfaces: Rust worker_target(available:u32)->u32 and bounded_workers(available,tasks,ram_slots)->u32, exported C ABI. Native deferred BRep assembly contains only independently owned OCCT faces/flags/tolerances. prepareModeling3dm(path,staging,scale=0) returns stable JSON plus worker/timing diagnostics; releases GIL during native work.

- [x] RED: Rust tests 0/1/2/4/8/12/16/24/32/u32::MAX, task/RAM limits; native tests serial/thread geometry equivalence, exception cleanup and actual concurrent tasks.
- [x] Run tests: expected failure for missing worker policy/pool interfaces.
- [x] Implement dynamic policy and OCCT-only assembly pool; retain serial SDK path, immutable source hash checks, validations and decorated errors.
- [x] Run Rust full suite and native targeted regression suites: expected PASS.

### Task 2: Native clipboard and application commands

Files: ThreeDmClipboard.py; InitGui.py; Gui/CoreThreeDm.cpp; Gui/Command.cpp; Gui/Workbench.cpp; rust/src/core_3dm.rs; CMakeLists.txt; tools/tests/test_modeling_clipboard.py; tests/modeling_clipboard_smoke.FCMacro.

Interfaces: copy_selection(), paste_selection(), available(operation), install_shortcuts()/remove_shortcuts(); menu, Command and Ctrl+C/V call these same functions. Short GUI clipboard snapshot; async native preparation and one main-thread commit. Busy job state/progress/cancel with explicit target-document guard.

- [x] RED: decoder rejects empty/text/truncated/oversize archives, no old clipboard overwrite on unsupported Copy; actual application menu/Command/shortcut and text-focus checks fail before feature.
- [x] Implement Win32 HGLOBAL adapter for registered Rhino 5 format; header/version check, own staging; background preparation and main-thread commit/publish, cleanup/cancel.
- [x] Run unit and actual FreeCAD gates: expected PASS including repeated Paste UUID/Undo/Redo, source close and FCStd reopen.

### Task 3: Mesh bulk construction and measurements

Files: ThreeDm.py; tests/modeling_clipboard_profile.FCMacro; tests/modeling_clipboard_smoke.FCMacro.

Interfaces: _prepare_import accepts optional preconverted model to reuse async native result. Mesh batch construction preserves triangle/quad triangulation and canonical signature.

- [x] RED: geometry orientation/topology test and fixture measurement expose facet construction overhead.
- [x] Implement bulk mesh construction; compare same-fixture baseline and candidate, warm-up plus ≥5 serial/30%/60% runs; record stage medians/max, CPU/RAM, N/target/effective.
- [x] Run Python project suite and actual application profiles/regressions: expected PASS and repeatable hotspot improvement; do not claim unmeasured speedup.

### Task 4: Actual Rhino gate, final review and evidence

Files: tests/rhino5_clipboard_manual.py; docs/validation/modeling-phase-1-clipboard/*; README.md; docs/openmatrix9-progress.json; docs/validation/opennurbs-test-roadmap.{json,md}; Spec v1 OM9-FILE-012 supplement.

- [x] Capture actual owned Rhino _CopyToClipboard format/header, run native _Paste for OM9 payload; synthetic types/units/placement/subset and real ring, SaveAs/reimport fidelity.
- [x] Bind reports to source/runtime hashes; old Phase 1 gate remains scoped and intact.
- [x] Dispatch one fresh final reviewer; fix Important/Critical findings with regression RED→GREEN.
- [x] Update evidence and progress. If owned Rhino cannot run, deliver the prepared manual script and explicitly leave actual Rhino acceptance pending.

Plan self-review: interfaces consistent; deferred pool never receives SDK geometry; clipboard adapter shares existing modeling writer/binder; app gate is separate. Latest user instruction to finish authorizes execution of the agreed design without another confirmation.
