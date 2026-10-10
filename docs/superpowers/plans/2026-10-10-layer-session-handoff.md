# OM9 ↔ Matrix9 Layer Session Handoff Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans for inline execution after design/plan review. Steps use checkbox (`- [ ]`) syntax for tracking. Không tự spawn subagent.

**Goal:** Chuyển phiên qua 3DM/Copy/Paste giữ hình học hiện tại, toàn bộ bộ màu, cấu trúc layer và khóa/mở/ẩn/hiện để tiếp tục thiết kế trang sức.

**Architecture:** Safe Rust quyết định state/merge/guard; FreeCAD/Qt/openNURBS và RhinoCommon chỉ commit native state. Bổ sung bảng layer độc lập, metadata clipboard, UI hoạt động và lệnh chuyển riêng ở Matrix9.

**Tech Stack:** Rust hiện hành; C++ FreeCAD/Qt/openNURBS; C# RhinoCommon tương thích Rhino5; Python/PowerShell cho kiểm thử.

**Spec:** ../specs/2026-10-10-layer-session-handoff-design.md

**Status:** Proposed; user requested planning, chưa triển khai sản phẩm. Spec và plan phải được duyệt trước execution.

## Global Constraints

- Source baseline: H:/FreeCAD-src/build/om9-phase3-dev; runtime baseline: H:/FreeCAD-src/build/om9-phase3-sdk. Khi execution tạo/reuse candidate riêng theo using-git-worktrees; root S dưới đây là candidate mới, không sửa accepted baseline.
- Tất cả shell command dùng rtk. Không commit unrelated changes, không tự tích hợp main/runtime chính.
- Safe Rust priority 1; không đưa policy mới vào Python/C++/C#. ABI có version, owned handles và generation; lỗi không xuyên ABI.
- Hai chiều 3DM/clipboard, source-wins theo full path; toàn bộ bảng layer kể cả trống đi kèm cả Selected và Session. Session thu thập object khóa/ẩn trực tiếp.
- Một transaction Undo cho nhận geometry + cập nhật existing layers + active layer; failure/cancel phục hồi before-state. Không silent skip unsupported geometry.
- Giữ worker60%, typed clipboard Rhino5 native, current geometry, Phase1–3 tolerance và phân biệt CAD/mesh/cloud.
- Không real-time/history/render, không full layer editor hoặc object UUID merge. Chưa sửa skill business rules ở giai đoạn lập kế hoạch.

## Review Focus

1. Mở khóa cha vô tình mở object/con khóa riêng: pin persistent/local flags, Task1/2/5.
2. Copy Selected giữ hình nhưng mất layer trống/bộ màu: pin full layer table + clipboard binding, Task2/4/5.
3. Source-wins làm đổi object có sẵn nhưng Undo chỉ xóa object mới: pin before-state/Redo, Task3/4/5.
4. CMD/tree/editor pending vượt lock: pin guard cùng generation ngay trước commit, Task3/5.
5. Rhino current-layer auto-normal làm mở khóa nguồn hoặc payload clipboard cũ được ghép nhầm: pin fallback + identity/version checks, Task1/4/5.

## Task 1 — Rust layer state và merge contract

**Files under S:** create rust/src/layer_state.rs, layer_exchange.rs, layer_ffi.rs; modify rust/src/lib.rs, rust/src/modeling_exchange.rs, rust/src/phase2_snap.rs; create rust/tests/layer_state.rs, layer_exchange.rs, layer_ffi.rs; extend Gui/RustBridge.h.

**Interfaces:** LayerSnapshotV1 {layers, objects, active_layer, generation}; LayerRow {source_id, parent_id, name, path_components, rgb, locked, visible, persistent_locked, persistent_visible}; ObjectLayerRow {id, layer_id, locked, visible, color_source, rgb}. Use Option<bool> for persistent unset/set state. IDs are owned values, not host pointers.

Functions: validate_layers(&LayerSnapshotV1) -> Result<(), LayerError>; effective_state(&LayerSnapshotV1, ObjectId) -> Result<EffectiveState, LayerError>; plan_receive(&LayerSnapshotV1, &LayerSnapshotV1, TransferScope) -> Result<LayerApplyPlan, LayerError>; can_mutate(&LayerSnapshotV1, &[ObjectId], LayerOperation) -> Result<(), LayerError>. LayerApplyPlan owns mappings, before/after rows, active-layer choice and expected destination generation.

- [ ] Write/run failing tests: own vs inherited lock, parent persistent restoration, hidden snap rejection/visible locked snap allowed, full-path matching with duplicate leaf names/case ambiguity, empty palette, ByObject preservation, cycle/missing parent rejection, stale generation and active fallback. Fallback uses source valid active, destination valid active, sorted valid layer, then unique OM9 Transfer Work.
- [ ] Implement typed model, membership indexes and source-wins plan without geometry input. FFI exports versioned create/free/validate/plan/apply-result handles; checked counts and explicit error codes; no retaining native pointers.
- [ ] Run rtk cargo test --manifest-path <S>/rust/Cargo.toml; rtk cargo fmt --manifest-path <S>/rust/Cargo.toml --check; rtk cargo clippy --manifest-path <S>/rust/Cargo.toml --all-targets -- -D warnings. All existing + new Rust tests pass.

## Task 2 — Native 3DM và clipboard mang đủ layer table

**Files under S:** modify Gui/ThreeDmArchive.h/.cpp, Gui/ThreeDmStaging.h/.cpp, Gui/ThreeDmModeling.cpp, Gui/ThreeDmPython.cpp, Gui/ThreeDmMerge.cpp, Gui/CoreThreeDm.cpp, ThreeDm.py; create Gui/LayerExchangeAdapter.h/.cpp; add native test tests/layer_exchange_native.cpp and register in existing build test configuration.

**Interfaces:** ExchangeModel adds layerRows and activeLayerId; ExchangeItem retains own object state/color source separately from computed effective state. Stage schema version2 has explicit layer table. Adapter converts native facts into Task1 values; geometry converter remains baseline owner. Clipboard publication includes native Rhino geometry format and OM9.LayerSession.v1 metadata bound to the geometry payload digest/byte length and version.

- [ ] Write/run failing native tests for empty/nested layers, RGB, persistent flags, ByLayer/ByObject, source/destination UUID remapping and full table with selected subset. Assert re-read flags separately; no conversion of inherited layer locks into object locks.
- [ ] Implement native layer/object field read/write and retained-export overlay. Upgrade existing flat import compatibility; use legacy true OM9Locked as object lock unless reliable stored provenance proves separation. Persist canonical layer/object records in FCStd through Task3.
- [ ] Implement Selected and Session collection. Session includes hidden/locked model objects; unsupported items fail preflight with inventory. Serialize geometry once; do not add rendering meshes to express colors. Empty geometry with valid layer table is an allowed palette-only transfer through OM9 commands.
- [ ] Test valid extended clipboard, native-only clipboard, unknown version, truncated metadata, digest mismatch, publish failure and max-size checks. Extended metadata mismatch rejects the transfer; native-only explicitly reports reduced palette evidence. Clipboard publish failure keeps previous clipboard.
- [ ] Native tests pass and existing 3DM/modeling/clipboard policy tests pass. Verify preserve mode still uses current edited geometry and preserves untouched native records according to its existing contract.

## Task 3 — OM9 layer panel, persistence, mutation guard và Undo

**Files under S:** create Gui/LayerDocumentAdapter.h/.cpp, Gui/LayerController.h/.cpp; modify Gui/MatrixSidebar.cpp/.h, Gui/CurveGeometry.cpp, Gui/ModelingCurveEditor.cpp, Gui/Phase3Inputs.cpp, Gui/CoreThreeDm.cpp, Gui/SnapObjectInfo.cpp, Gui/SnapQueryIndex.cpp, Gui/CoreSnapGeometry.cpp and command registration/build lists. Modify Gui/tests/MatrixSidebarTest.cpp; create tests/modeling_layer_panel_smoke.FCMacro, modeling_layer_guards_smoke.FCMacro, modeling_layer_lifecycle_smoke.FCMacro.

**Interfaces:** LayerController::setCurrent(path), assignSelection(path), setLocked(path,bool), setVisible(path,bool), receive(snapshot,scope). Each consumes Task1 plans and commits on GUI thread. LayerDocumentAdapter persists canonical layer table, object own flags/color source and active layer; compatibility OM9Locked/Visibility/Selectable are projections.

- [ ] Write/run failing host tests: enabled panel controls; selected-set assignment; new geometry takes active layer/color; ByObject unaffected; imported custom/nested layers accessible; palette source colors replace static swatches without merging equal RGB layers.
- [ ] Implement live per-document sidebar backed by Rust state. Maintain existing32 names/order as default preset only; additional dropdown/list exposes all imported layers. New UI gestures are explicitly OM9 choices, not claimed Matrix9 recovered behavior.
- [ ] Centralize preflight + commit lock checks for supported editing, transforms, Delete, reassignment and property/tree entrypoints. Batch contains locked object → no partial edit. Pending editor becomes stale after layer lock. Use native edit/delete hooks where those supported UI routes bypass OM9 command dispatch; pin real runtime behavior before claiming coverage.
- [ ] Persist/reload FCStd; implement atomic receive with existing-layer updates and active fallback in Undo/Redo. Restore selection/model state on Cancel/failure without rolling back unrelated transactions.
- [ ] Host tests pass: menu and CMD invoke same controller; locked visible objects snap but cannot mutate; hidden do not snap; legacy migration conservative; Undo/Redo/reopen keep all local/effective states. Layer metadata toggles leave geometry signatures unchanged.

## Task 4 — Matrix9/Rhino5 commands cho bàn giao phiên

**Files under S:** create bridge/rhino5/OM9LayerTransfer/OM9LayerTransfer.csproj, Plugin.cs, TransferCommands.cs, RhinoLayerAdapter.cs, NativeLayerApi.cs; create rust/src/layer_bridge_ffi.rs and rust/tests/layer_bridge_ffi.rs; update packaging/build files; create tests/rhino5_layer_exchange.py and tests/run_rhino5_layer_exchange.ps1.

**Interfaces:** RhinoLayerAdapter extracts RhinoCommon layer/object metadata into Task1 snapshot and applies LayerApplyPlan using source-to-destination IDs. NativeLayerApi invokes the same Rust policy via a separate cdylib wrapper; no duplicated merge logic in C#. Build target framework/bitness from installed Rhino5 SDK and actual Matrix host, with explicit loader diagnostics; do not guess architecture from install folder.

- [ ] Before product implementation, test a minimal runtime API probe: installed Matrix loaded, host architecture, persistent flags, current layer switching, command-level Undo/Redo and failure rollback ownership. Record exact API/version evidence. Do not use a live user document as fixture.
- [ ] Implement OM9Copy3dm, OM9CopySession, OM9Paste3dm, OM9Import3dm, OM9ExportSelected3dm, OM9ExportSession3dm after checking name collision. Publish/import actual native geometry through baseline native bridge; complete metadata/colors/empty layers through the extended contract. Do not invoke native selection-dependent Copy for Session.
- [ ] Receiver stages/prevalidates before mutation; maps full paths; changes current layer to a valid fallback before locking/hiding old current layer; applies own object flags and persistent child flags separately. Empty palette transfers work without geometry.
- [ ] Make one command-owned Undo encompass additions and existing-layer changes. Use saved before-images for failure rollback; Rhino BeginUndoRecord returning0 is not success. Test failure after layer update and after geometry add, then Undo/Redo with preexisting content. No blind _Undo affecting earlier user work.
- [ ] Verify separate toolbar/buttons and original native shortcuts still operate. Demonstrate C# adapter only invokes native APIs, Rust owns policy, no product Python implementation.

## Task 5 — Kiểm chứng chuyển phiên trang sức, hiệu năng và acceptance

**Files under S:** create tests/rhino5_verify_layer_session.py, tests/run_layer_session_verify.ps1, tests/modeling_layer_session_reread.FCMacro, tests/modeling_layer_performance_smoke.FCMacro; create docs/features/OM9-LAYER-001.md and docs/validation/layer-session/{requirements.json,summary.json,acceptance.md}; update source progress/README/test-roadmap after verified implementation only.

- [ ] Build fixtures: all32 default slots with non-default source colors, empty layers, same RGB different names, nested duplicate leaves, parent/object/con own locks, hidden layers, ByObject exceptions, existing destination collisions, edited ring/BRep/curve and heavy mesh/cloud. Include invalid clipboard/version/unsupported inventory cases.
- [ ] Run real OM9 ↔ Matrix9 both directions, file and clipboard, Selected and Session. Matrix9 must be loaded; clean Rhino5-only result is interim. In destination unlock a layer, edit geometry/draw more, lock/hide another, transfer back and continue. Assert object counts/identity mapping, current geometry oracle, full layer palette, local/persistent/effective flags and active layer.
- [ ] SaveAs/Export Selected in Rhino5, reread exactly those outputs in OM9; save/reopen FCStd. Test one Undo/Redo restores preexisting destination color/lock/visibility and removes/restores imported objects together. Test Cancel/failure restores all state.
- [ ] Add per-stage milliseconds to JSONL and summary: metadata extraction, geometry serialization/conversion, layer merge, clipboard publish/read, commit, redraw and total; object/layer/byte counts, direction, scope, cold/warm. Run alternating baseline/candidate A/B with same fixtures and sample counts; report distributions, not only fastest run.
- [ ] On 10k mixed objects with million-point mesh/cloud, assert layer toggles/merge metadata do not call point-array extraction, BRep serialization or remeshing and do not alter geometry. Warm snap cache obeys existing accepted budgets; locks/visibility invalidate only eligibility state. No invented fixed latency threshold for GUI redraw of 10k objects.
- [ ] Run full candidate Rust/native checks and accepted Phase1–3 verification scripts. Pin source/runtime/plugin hashes, actual Matrix host, run IDs and exact artifacts; distinguish prepared/ran/passed fixtures and remaining scope. Update progress only to the proved slice; no full openNURBS claim.

## Handoff and completion

Before execution, review linked spec/plan and preserve user's final clarification: all layer colors and lock/unlock state travel with handoff, no realtime. Default snap/active fallback and whole-session collection are proposed design choices documented for review. Inline execution matches existing user preference; no implementation starts merely because this plan exists.

Completion requires Task1–5 application evidence on final candidate, with Matrix9 loaded and repeatable two-way session handoff. Source changes, Rust test counts or Rhino-only PASS do not substitute for that. Candidate stays separate until integration is authorized.
