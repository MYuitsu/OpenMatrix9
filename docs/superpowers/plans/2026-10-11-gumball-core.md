# Core Gumball Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Triển khai và nghiệm thu ba spec lõi OM9-INFO-020/021/022 trong FreeCAD: Gumball, Alignment và Relocate.

**Architecture:** Safe Rust quản lý session, frame, input/modifiers và affine plan. Native adapters dùng FreeCAD/Coin draggers, dựng preview ngoài document và commit một transaction. Các entrypoint menu/CMD/chuột/phím chia sẻ controller; tận dụng object, coordinate, units, layer và preservation contracts hiện có.

**Tech Stack:** Rust 2024; C++/Qt/Coin/OCCT/openNURBS của shared FreeCAD SDK; Python chỉ cho fixture/bootstrap. Không thêm dependency để giải quyết state/input vốn có thể viết bằng Rust standard library và các module hiện hữu.

**Spec:** [Thiết kế đã được người dùng duyệt ngày 2026-10-11](../specs/2026-10-11-gumball-core-design.md).

## Global Constraints

- Phạm vi gồm OM9-INFO-020, OM9-INFO-021 và OM9-INFO-022. Smart Targets/Clayoo/SubD và lệnh Move/Rotate/Scale độc lập nằm ngoài phạm vi đã chốt.
- SDK duy nhất: `H:/FreeCAD-src/build/relWithDebInfo`; chỉ build `OpenMatrix9Gui` qua `build/build-om9-plugin.ps1`.
- Worker mặc định 60% logical CPUs; không build root FreeCAD hoặc clone runtime đầy đủ.
- Plugin runtime riêng, shared host qua `build/run-om9-plugin.ps1`, test profile riêng mỗi lần chạy.
- Rust sở hữu logic/state/validation; C++ chỉ giữ native objects/widgets, observations và geometry/transaction adapter. Không giữ raw host pointer trong Rust.
- X đỏ, Y xanh, Z vàng. On/Off/Toggle; default OM9 Off, Alignment CPlane, Copy Off mỗi gesture.
- Preview không thêm document object/property/Undo entry. Commit nguyên tử; lỗi/Cancel giữ model trước gesture.
- Giữ exact feature ID và alias, thêm `GumballRelocate` như alias của `RelocateGumball`; không đăng ký Python command trùng.
- Input length dùng `units::parse_length(raw, mm_per_unit)`, angle dùng `units::parse_angle(raw)`; factor là finite dimensionless scalar. Không dùng bounds của measurement Angle để giới hạn mọi geometry.
- Object alignment không xác định được phải giữ frame cũ và báo unresolved. Capability thiếu phải có lý do, không fallback mesh hoặc ghi Shape vào feature parametric rồi bị recompute ghi đè.
- Không nâng completion/parity do icon, validator, compilation hay kiểm tra source.

## Review Focus

1. Parent+child và App::Link cùng definition: mỗi occurrence biến đổi đúng một lần, shared source không bị sửa. Pin trong Task 2.
2. Ctrl có từ mouse-down khác Ctrl thêm sau drag-start; Alt auto-repeat không đổi Copy nhiều lần. Pin trong Tasks 1 và 4.
3. Numeric nhập từ viewport/CMD/editor phải giữ mouse grab và IME/text focus đúng; CoreMouse/CurveController không nuốt event. Pin trong Tasks 3 và 4.
4. Source thay thế/xóa, lock hoặc units đổi giữa preview/commit phải reject nguyên tử và dọn callback. Pin trong Tasks 2 và 5.
5. Nonuniform/negative scale và copied retained 3DM giữ representation, orientation, metadata/preservation qua Undo/Redo/cold reload. Pin trong Tasks 2 và 5.

---

## Baseline và workspace

Repo plugin là `H:/FreeCAD-src/Mod/OpenMatrix9`, độc lập với repo FreeCAD cha.
Checkout hiện có nhiều thay đổi chưa commit của các family khác. Commit thiết kế
là `8c085c8`; không chọn commit đó làm baseline của product nếu bỏ sót working tree.

Cache được đọc ngày 2026-10-11: `build/openmatrix9-history-cage` trỏ đúng checkout
plugin và shared SDK, output hiện là `build/surface-history-admission-runtime1-20261011`.
Phải đọc lại cache khi bắt đầu vì các chat khác có thể đang đổi output này.

- [ ] Trước Task 1, kiểm tra Git dir/common dir/superproject, trạng thái staged,
  source hashes và cache. Lưu manifest baseline ở report directory, gồm changed
  tracked files và untracked public source cần thiết; không commit/ghi đè thay đổi
  khác, không copy ignored private inputs/build/runtime.
- [ ] Chạy một Rust baseline trên đúng working source; ghi exit/report riêng.
  Failure đã có trước được phân biệt với regression, không tự gộp vào Gumball.
- [ ] Execution method/workspace được người dùng chọn ở handoff cuối plan. Nếu
  chọn làm tại checkout hiện tại, giữ scoped patches và kiểm file mới thay đổi
  trước edit. Nếu chọn isolation, dùng worktree tools theo skill và bảo toàn
  working-source baseline; không khởi đầu từ default branch cũ thiếu code mới.

Các command bên dưới giả định thực thi từ `H:/FreeCAD-src/Mod/OpenMatrix9` và
dùng RTK. Adapter build chỉ dùng cache đã xác nhận đúng checkout.

## Task 1: Rust session, transform math, input và ABI

**Files:** Create `rust/src/gumball.rs`, `rust/src/gumball_ffi.rs`,
`rust/tests/gumball_session.rs`, `rust/tests/gumball_ffi.rs`; modify
`rust/src/lib.rs`, `rust/src/ffi.rs`, `Gui/RustBridge.h`.

**Interfaces:**

- `gumball::command(icon: &str) -> Option<&'static str>` ánh xạ ba icon sang alias;
  `gumball::kind(name: &str) -> Option<CommandKind>` nhận alias và prefix `_`/OM9.
- `GumballSession::new() -> Self`; `apply(&mut self, event: Event) -> Result<Effect, GumballError>`;
  `snapshot(&self) -> Snapshot`; `plan(&self) -> Result<TransformPlan, GumballError>`.
- Types công khai của module: `Alignment { CPlane, Object, World }`,
  `CommandKind { Gumball, Alignment, Relocate }`,
  `Axis { X, Y, Z }`, `Plane { XY, YZ, ZX }`,
  `Handle { MoveAxis(Axis), MovePlane(Plane), MoveFree, Rotate(Axis), Scale(Axis) }`.
- `Event`: Enable(bool), Align(Alignment), Begin(BeginContext), Motion(PointerRay),
  Modifier(ModifierEvent), Numeric(String), Accept, Undo, Cancel,
  BeginRelocate, RelocatePoint([f64;3]), CommitSucceeded.
- `BeginContext` chứa frame origin/X/Y/Z, source stamps, view/document/unit
  generation, mm_per_unit, handle, start ray và modifiers. `PointerRay` có
  world origin/direction từ native camera; Rust giải projection/constraint.
  `ModifierEvent` ghi key, pressed, auto_repeat. Không dùng float render translation
  làm input geometry; source identity được copy sang Rust.
- `TransformPlan`: row-major `[f64;16]`, operation Move/Rotate/Scale/Extrude,
  Copy flag, source identities, session generation và accepted input provenance.
- `Snapshot`: enabled, phase, alignment, frame, active handle, Copy, constraint,
  generation, preview plan/error. Hover không tạo checkpoint; input accepted có
  checkpoint. Dùng giới hạn `coordinate_input::MAX_INPUTS = 64` cho session Undo.
- FFI session handle `u64`; `om9_gumball_create() -> u64`,
  `om9_gumball_begin(u64, const Om9GumballBegin*, const Om9GumballSource*, size_t) -> u32`,
  `om9_gumball_event(u64, const Om9GumballEvent*) -> u32`,
  `om9_gumball_input(u64, const uint8_t*, size_t) -> u32`,
  `om9_gumball_snapshot(u64, Om9GumballSnapshot*) -> u32`,
  `om9_gumball_destroy(u64) -> u32`. Records `repr(C)` chỉ chứa fixed-width
  scalars/arrays; borrowed pointers chỉ sống trong call. Status 0 thành công,
  1 invalid handle/ABI/input, 2 recoverable state/geometry error. Snapshot có
  phase/flags/frame/matrix/constraint/generation; write toàn record chỉ khi hợp lệ.
- Native source list admission kế thừa `native_input_tree` depth/nodes/bytes
  budgets; numeric text kế thừa 1024-byte input cap. Không thêm JSON parser/dependency.
- ABI records chốt các dữ liệu cần thiết: `Om9GumballSource` chứa document,
  object, generation, revision, epoch, parent owner và world matrix `[f64;16]`;
  `Om9GumballBegin` chứa document/view/unit generations, mm_per_unit, frame
  `[f64;12]`, start ray origin/direction `[f64;3]`, handle và modifier mask.
  `Om9GumballEvent` chứa event kind, ray origin/direction, world point, key,
  pressed và auto_repeat. Bool dùng `u32`, ID dùng `u64`; modifier mask kế thừa
  Shift=1, Ctrl=2, Alt=4, Meta=8 của CoreMouse. Header khai báo cùng enums/layout;
  trường không áp dụng cho event không được đọc như số geometry.

- [ ] **Step 1 — RED:** Viết semantic tests theo exact IDs: X Move 5 từ frame
  World dịch `[5,0,0]`; frame O=(10,20,30), X=(0,1,0) dịch thành `[0,5,0]`;
  pivot (3,4,0), point (5,4,0) quay 90° thành (3,6,0); Scale X factor 2 giữ Y/Z
  và pivot; Shift factor 2 scale đủ ba trục; zero/NaN/Infinity reject giữ snapshot.
  Pin Alt two taps, auto-repeat, Ctrl-at-start relocate vs Ctrl-after-start
  extrusion, numeric length/angle/factor dimensions, hover/Undo/Cancel và no-op.

  Helper `world_move_x() -> GumballSession` trong test enable session, Begin
  MoveAxis(X), frame World, source stamp hợp lệ và mm_per_unit=1. Helper
  `point(matrix: [f64;16], p: [f64;3]) -> [f64;3]` dùng phép nhân độc lập trong
  fixture, không gọi transform math của production.

  ```rust
  #[test]
  fn world_axis_numeric_move_is_exact() {
      let mut session = world_move_x();
      session.apply(Event::Numeric("5 mm".into())).unwrap();
      assert_eq!(point(session.plan().unwrap().matrix, [0.0; 3]), [5.0, 0.0, 0.0]);
  }
  #[test]
  fn both_relocate_aliases_share_one_command() {
      assert_eq!(kind("RelocateGumball"), Some(CommandKind::Relocate));
      assert_eq!(kind("GumballRelocate"), Some(CommandKind::Relocate));
  }
  ```
- [ ] **Step 2 — Verify RED:** `rtk cargo test --manifest-path rust/Cargo.toml --test gumball_session --test gumball_ffi`.
  Xác nhận failure do API/behavior chưa có; không coi lỗi môi trường là RED.
- [ ] **Step 3 — Implement:** Viết pure Rust session/math, reuse coordinate/frame,
  units và source/ancestor observations. Kiểm frame affine/rigid, singularity và
  checked finite math bằng contracts hiện có. Define FFI record layout/asserts ở
  cả Rust/C++; invalid call không mutate session/output và không unwind qua ABI.
  Map command aliases/captions; metadata này chưa bật executor native.
- [ ] **Step 4 — GREEN:** Chạy lại hai suites; thêm pointer-null, bad enum/count,
  short/invalid UTF-8, stale session handle, session independence và destroy cleanup.
- [ ] **Step 5 — Commit:** Commit chỉ files của task đã kiểm chứng; tách additions
  trong files đang dirty bằng scoped hunks, không stage cả working tree.

## Task 2: Native geometry, selection ownership và Object frame

**Files:** Create `Gui/GumballGeometry.h/.cpp`, `Gui/GumballFrame.h/.cpp`,
`tests/gumball_geometry_smoke.FCMacro`; modify `Gui/CMakeLists.txt`,
`Gui/AppOpenMatrix9Gui.cpp` chỉ để đăng ký read-only fixture observations.

**Interfaces:**

- Native owned structs `GumballInputs` và `GumballStagedBatch` chứa source
  snapshots/private geometry/native output, không truyền owner handles sang Rust.
- `captureGumballInputs(App::Document&, const std::vector<Gui::SelectionObject>&,
  const CoordinateContext&) -> GumballInputs`;
  `verifyGumballInputs(App::Document&, const GumballInputs&) -> void`.
- `stageGumballBatch(const GumballInputs&, const Om9GumballSnapshot&) -> GumballStagedBatch`;
  chỉ tạo output riêng, chưa publish document. Publisher được triển khai và
  kiểm chứng qua pointer gestures ở Task 4.
- `captureGumballFrame(const GumballInputs&, Gui::View3DInventor*) -> GumballFrameObservations`.
  Observations gồm origin, tangent/direction, normal và capability/unresolved
  reason; Rust sở hữu normalized frame. CPlane/World không cần topology inference.
- Reuse `captureObject/verifyObject`, `captureCoordinateContext/verifyCoordinateContext`,
  `sourceAttributes`, prepared geometry/validation, `CoreLayers::objectEditable`,
  `CoreLayers::inheritLayer`. App::Link sửa occurrence, không FollowLink rồi sửa definition.
- Read-only Python observation `gumballGeometryFacts(document_name: str,
  object_names: list[str], handle: str, value: str, alignment: str) -> dict`:
  capture inputs, chạy Rust Begin/Numeric và stage private output, trả facts rồi
  giải phóng stage. Không nhận raw matrix, không publish document. Facts gồm
  selected_roots, source_signatures_before/after, staged bounds/type/orientation,
  frame/capability/error; không dùng hook này làm chứng cứ UI hoặc persistence.

- [ ] **Step 1 — RED:** Native tests độc lập đo box ở nested App::Part, parent+child,
  hai links chung definition, mesh/points và plain/retained 3DM objects. Kiểm
  stage không mutation; Copy có kế hoạch identity riêng; Scale X
  circle thành ellipse, reflected solid orientation hợp lệ; hole face extrusion
  giữ hole; invalid/nonplanar extrusion reject toàn batch.
  Object frame fixtures gồm curve tangent/normal, planar face, mesh face/edge/vertex;
  light/camera observations kiểm theo capability host thật, unavailable có lý do.

  ```python
  def test_staged_scale_preserves_document():
      facts = OpenMatrix9Gui.gumballGeometryFacts(
          doc.Name, [circle.Name], "ScaleX", "2", "World")
      assert facts["source_signatures_after"] == facts["source_signatures_before"]
      assert facts["staged"][0]["curve_type"] == "Ellipse"
      assert facts["staged"][0]["bounds_x"] == 4.0 * radius
  ```

  Fixture tạo circle ở World origin với radius=5 mm trước observation. Native
  facts đo private output bằng OCCT/Mesh/Points API, không lấy đáp án từ matrix.
- [ ] **Step 2 — Verify RED:** Build plugin-only matching source và chạy fixture
  bằng shared-host runner. Failure phải là chưa có adapter/behavior, ghi report và exit.
- [ ] **Step 3 — Implement:** Capture immutable sources, dùng Rust để deduplicate/
  normalize selected roots từ ancestor observations. Stage BRep/Mesh/Points/
  instance transform và planar extrusion bằng native adapters đúng representation.
  Giữ group/layer/material/metadata và retained archive state. Parametric family
  chỉ sửa property/placement được owner hỗ trợ; từ chối thay Shape không bền vững.
  Verify source/lock/units trước và sau staging; sai một source/output phải hủy
  toàn bộ private batch. Publication/transaction chưa thuộc task này.
- [ ] **Step 4 — GREEN:** Chạy geometry suite; thêm source deleted/replaced, layer
  locked, unit change, bad copy/extrusion output và second-object failure. Đo
  source signatures/object count/Undo trước-sau để pin no mutation. Kiểm retained
  archive observations; actual export/commit/Undo/Redo được pin ở Tasks 4–5.
- [ ] **Step 5 — Commit:** Scoped native geometry/frame và fixture evidence.

## Task 3: Controller, menu/CMD, preference và lifecycle

**Files:** Create `Gui/GumballController.h/.cpp`, `tests/gumball_commands_smoke.FCMacro`;
modify `Gui/Command.cpp`, `Gui/Workbench.cpp`, `Gui/CurveController.cpp`,
`Gui/CoreKeyboard.cpp`, `Gui/CoreMouse.cpp`, `Gui/CMakeLists.txt`,
`Gui/AppOpenMatrix9Gui.cpp`, `rust/src/ffi.rs`.

**Interfaces:** `GumballController::instance()`, static `handles(std::size_t)`
and `matches(std::size_t,const QString&)`; instance `available(std::size_t) const`,
`execute(std::size_t)`, `activate()`, `deactivate()`, `active() const`,
`submit(const QString&) -> bool`, `cancel()`, `processEvent(QObject*,QEvent*) -> bool`.
`active()` nghĩa đang gesture/editor/relocate, không phải chỉ enabled Ready.
`submit` chỉ consume exact command hoặc input của session đang active.

**Produces:** Preference On/Off/alignment; one session owner; prompt/error qua
CurveController; read-only native Python observations `gumballState()` và
`gumballHandles()` để fixture tìm handle screen position, không API sửa geometry.

- [ ] **Step 1 — RED:** Native menu/CMD fixtures cho On/Off/Toggle, CPlane/Object/
  World và hai Relocate aliases. Kiểm default/restore preference, selection
  change, empty/locked/mixed selection và command availability. Pin scene clear
  khi Line/Surface/Edit bắt đầu, deactivate/document close/view destroyed; enable
  preference vẫn giữ, controller không giành mouse callback của session khác.

  ```python
  def test_gumball_toggle_creates_no_geometry_or_undo():
      count, undo = len(doc.Objects), doc.UndoCount
      Gui.runCommand(gumball_command_id)
      assert OpenMatrix9Gui.gumballState()["enabled"] is True
      Gui.runCommand(gumball_command_id)
      assert OpenMatrix9Gui.gumballState()["enabled"] is False
      assert (len(doc.Objects), doc.UndoCount) == (count, undo)
  ```

  Fixture bắt đầu ở profile mới default Off; command ID lấy từ Rust icon mapping
  đã kiểm chứng Task 1, không hardcode caption hoặc command name suy đoán.
- [ ] **Step 2 — Verify RED:** Chạy `gumball_commands_smoke.FCMacro`; fixture phải
  fail tại command/session requirement, không pass từ icon/metadata.
- [ ] **Step 3 — Implement:** Nối router/caption/permissions và Workbench lifecycle.
  Menu toggle/alignment/relocate là view state; geometry gesture kiểm AlterDoc
  và source locks riêng. Không ghi successful geometry history tại drag-start.
  Exclude toggle/alignment/relocate khỏi repeat candidate; successful geometry
  commit mới ghi execution. Suspend/hide Ready overlay khi family khác hoạt động,
  cancel gesture trước transition; tránh recursion của `CurveController::cancel`.
  Route numeric/relocate input trước parser của command khác khi Gumball owns input.
- [ ] **Step 4 — GREEN:** Test CMD keyboard từ viewport/console, Enter theo phase,
  Esc editor trước rồi gesture, no empty Undo và workbench/document switch cleanup.
  Re-run command repeat/Line slice khi routing/filter thay đổi.
- [ ] **Step 5 — Commit:** Scoped controller/router và command fixture.

## Task 4: Handles, pointer/numeric gestures, Alignment và Relocate UI

**Files:** Create `Gui/GumballOverlay.h/.cpp`, `tests/gumball_core_smoke.FCMacro`;
modify controller, `Gui/GumballGeometry.h/.cpp`, `Gui/CoreMouse.cpp`, `Gui/CurveController.cpp`, `Gui/CoreKeyboard.cpp`,
`Gui/CMakeLists.txt` và `rust/tests/gumball_session.rs` theo interface đã định.

**Interfaces:** `GumballOverlay::attach(Gui::View3DInventor*)`, `detach()`,
`render(const Om9GumballSnapshot&)`, `hit(const QPoint&) -> std::optional<HandlePick>`,
`numericEditor(HandlePick,bool copy)`, `clearPreview()`, `handlePositions() const`.
`HandlePick` chứa exact handle enum và native world ray observations.
Overlay sở hữu Coin nodes/QPointer editor; controller sở hữu lifecycle và Rust session.
Task này thêm `publishGumballBatch(App::Document&, const GumballInputs&,
const Om9GumballSnapshot&, GumballStagedBatch&&) -> void`: verify lại source,
ownership, lock và units; publish toàn batch trong một transaction, abort đầy đủ
nếu bất kỳ source/output thất bại. Consume capture/stage của Task 2.

- [ ] **Step 1 — RED:** Fixture gửi actual Qt pointer press/move/release tới từng
  handle. Đo Move axis/plane/origin, Rotate pivot, Scale1D/Shift3D, click numeric
  exact distance/angle/factor, Alt-copy numeric/drag, Ctrl extrusion và Ctrl-start
  relocate. Native handle coordinates chỉ dùng để nhắm pointer, không thay event.
  Pin typed Enter khi còn giữ chuột chỉ đặt distance/bước góc; release mới commit.

  ```python
  def test_numeric_during_drag_waits_for_release():
      original = source.Shape.BoundBox.XMin
      pointer_press_move("MoveX")
      type_viewport("5 mm")
      press_enter()
      assert source.Shape.BoundBox.XMin == original
      pointer_release()
      assert source.Shape.BoundBox.XMin == original + 5.0
      doc.undo()
      assert source.Shape.BoundBox.XMin == original
  ```

  Helpers gửi Qt events tới viewport ở handle position từ native observations;
  fixture dùng box World, mm units. Assert hit handle trước nhập; không gọi API
  geometry mutation. Thêm batch second-source failure và Copy IDs ở cùng suite.
- [ ] **Step 2 — Verify RED:** Chạy core fixture trên shared host, giữ failure
  report/source/plugin hash và screenshot. Pointer chưa đến handle là harness
  failure cần sửa, không bỏ hit-test assertion.
- [ ] **Step 3 — Implement:** Ghép exported native translation/planar/rotation
  draggers, square scale và origin handles trong scene overlay; set màu OM9 và
  auto-size theo DPI/camera. Handle identity lấy từ picked scene path.
  World rays dùng camera API; math/constraints ở Rust. Native float fields chỉ
  phục vụ render. `processEvent` được gọi trước CoreMouse/CurveController khi
  hit handle hoặc Gumball active; click geometry/foreign editor/navigation đi tiếp.
  Capture mouse đúng owner và release một lần; Alt auto-repeat ignored; key timing
  phân biệt Ctrl relocation/extrusion. Numeric editor đơn vị đúng, IME/text focus
  không bị global shortcuts nuốt. Relocate CMD/picks nhận origin/X/tùy chọn Y,
  Enter hoàn tất theo CPlane khi frame đủ, invalid giữ phase và frame trước.
  Nối publisher vào release/accept geometry; verify toàn source batch ngay trước
  commit, cập nhật copied identities và retained archive theo owner contract.
- [ ] **Step 4 — GREEN:** Chạy core fixture DPI 1 và 1.5; pin no float rounding
  của typed values, pan/zoom/orbit không commit, selection không mất, Copy cursor,
  Cancel khôi phục frame/geometry và multiple-selection pivot reset. Numeric
  Ctrl-click chỉ đổi widget; no document count/Undo change cho Alignment/Relocate.
- [ ] **Step 5 — Commit:** Scoped overlay/events và genuine pointer evidence.

## Task 5: Persistence, regression, review và cập nhật completion

**Files:** Create `tests/gumball_core_cold_smoke.FCMacro`,
`docs/features/OM9-INFO-020.md`, `docs/features/OM9-INFO-021.md`,
`docs/features/OM9-INFO-022.md`, `docs/validation/2026-10-11-gumball-core.md`;
modify exact three feature specs, FEATURES JSON/YAML, IMPLEMENTATION_STATUS,
01-core README, MANIFEST, root README và progress ledger. Fix product defects
chỉ trong task owner đã xác định; mọi fix cần regression RED/GREEN.

**Interfaces:** FCStd path/report từ warm fixture sang cold process; immutable
expected geometry/ownership/preservation facts, không tham chiếu live host objects.
Feature records phân biệt evidence, OM9 defaults, unsupported representations
và next action; không dùng một aggregate pass để tick mọi requirement.

- [ ] **Step 1 — RED:** Warm fixture commit Move/Rotate/Scale/Copy/Extrude rồi
  Undo/Redo và save. Cold process reopen đo geometry/frame-independent bounds,
  actual curve/surface representation, signed orientation, group/layer/materials,
  independent copy IDs, instance source và retained 3DM export/reimport facts.
  Đổi lock/units, replace/delete source sau preview rồi release phải atomic reject.
  Document/view/workbench close không để callback hoặc preview object còn sống.

  ```python
  def test_cold_copy_keeps_independent_identity_and_source():
      restored = FreeCAD.openDocument(saved_path)
      original = restored.getObject(expected["source_name"])
      copied = restored.getObject(expected["copy_name"])
      assert geometry_facts(original) == expected["source_geometry"]
      assert object_identity(original) != object_identity(copied)
      assert abs(copied.Shape.BoundBox.XMin - expected["copy_xmin"]) < 1e-9
      assert layer_id(copied) == expected["layer_id"]
      assert retained_export_facts(restored) == expected["archive_facts"]
  ```

  Warm fixture lưu expected geometry/facts JSON bên cạnh FCStd. `geometry_facts`
  đọc native topology counts, curve/surface types, bounds, area/volume và sampled
  coordinates; chuẩn hóa floats tới 9 chữ số thập phân cho fixture mm này.
  Không so sánh native TShape pointer/identity giữa hai process. Các helpers
  đọc object/layer identity và export preservation bằng API native hiện có;
  cold process không nhập module state hoặc live object từ warm process.
- [ ] **Step 2 — Verify RED:** Chạy warm/cold fixtures trước sửa persistence issue;
  xác định failure thuộc code hay harness; giữ actual process exits.
- [ ] **Step 3 — Implement:** Sửa đúng persistence/publication/lifecycle defect
  từ RED, không mở thêm family. Bổ sung feature records và traceability từng
  acceptance item -> fixture/evidence; giữ item chưa chứng minh chưa hoàn tất.
- [ ] **Step 4 — Verify:** Chạy Rust suites/full Rust một lần sau integration,
  native command/geometry/core/cold; regression theo files bị đổi: Line/Polyline,
  repeat, mouse navigation, layers/History, retained 3DM. Chạy public source audit
  và whitespace checks. Independent whole-change review theo method đã chọn;
  xử lý actionable findings, chỉ re-run suites liên quan tới fix.
- [ ] **Step 5 — Sync/commit:** Đồng bộ spec/checklists/catalog/status/manifest/
  README/ledger với đúng evidence. Không ghi complete nếu required behavior còn
  missing. Commit scoped changes, không commit user models hoặc source khác.

## Verification commands và output cần quan sát

```powershell
rtk cargo test --manifest-path rust/Cargo.toml --test gumball_session --test gumball_ffi
rtk proxy powershell -NoProfile -ExecutionPolicy Bypass -File H:/FreeCAD-src/build/build-om9-plugin.ps1 -CheckOnly
rtk proxy powershell -NoProfile -ExecutionPolicy Bypass -File H:/FreeCAD-src/build/build-om9-plugin.ps1
rtk proxy python tools/public_source_audit.py
rtk git -c core.whitespace=cr-at-eol diff --check
```

Run native macro từ root FreeCAD bằng `build/run-om9-plugin.ps1` với
`-PluginRuntime` bằng chính runtime đọc từ cache đã xác minh,
`-Macro` bằng absolute path tới fixture và `-OutputDirectory` riêng dưới
`H:/FreeCAD-src/build/shared-host-tests/`. Các chạy command/geometry/core/cold
đều phải có `results.json.ok=true`, không failed checks và actual exit code 0.
Report chỉ xuất hiện không đồng nghĩa process pass. Runtime directory cache có
thể đổi; ghi actual paths/hashes vào report trước dùng, không tự copy SDK.

## Coverage self-review và handoff

Thiết kế sections mapping: display/selection -> Tasks 2–4; state/math/numeric/
Copy/Extrude -> Tasks 1/2/4; Alignment/Relocate -> Tasks 1–4; geometry/History/
staging -> Task 2, publication/atomic commit -> Task 4; lifecycle -> Tasks 3–5; save/reload -> Task 5;
build/source audit/spec synchronization -> Task 5. Năm Review Focus đều có
test step được chỉ định. Các interface tên/type dùng cùng tên giữa tasks.

Đề xuất **Native**: tôi triển khai trong session này trên baseline đã kiểm tra,
giữ scoped patches tại checkout hiện tại, rồi một reviewer kiểm toàn bộ thay đổi.
Các tasks phụ thuộc chặt vào cùng session/ABI và event routing, nên cách này
tránh trao đổi interface qua nhiều implementers. Chưa bắt đầu product code;
người dùng cần duyệt plan và chọn method trước execution theo writing-plans.
