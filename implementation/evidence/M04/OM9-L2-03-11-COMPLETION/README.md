# M04 L2-03-11 builder UI completion

This directory records the second builder slice for `OM9-L2-03-11` (02-febe-matrix-builder-ui-
and-live-handles). It completes the capabilities the first slice
(`../OM9-L2-03-11/`) left open and re-uses that code instead of adding a second builder.

## What changed on top of the first slice

- `BuilderField::step/quantize/slider_ratio/from_slider_ratio` and
  `BuilderSession::set_slider`: a slider moves in whole field steps, so 2.0 + 0.46 is stored
  as exactly 2.5 mm and the panel, the model and the preview read the same number.
- `BuilderSession::revision` and `set_field_at`: a panel event captured before a viewport
  handle drag is refused instead of overwriting the newer value.
- `BuilderSession::editable` plus `Controller::builder_adjust_field`/`builder_set_slider`
  guards: a locked input makes the builder read-only, paints no handle and disables every edit
  control, so a read-only session cannot be edited by accident.
- The BUILDERS panel paints one tab per mode (each keeps its own parameters), a slider next
  to each numeric field, a read-only warning line and the Profile Library window (with a stable
  `.id`, so it keeps its place when the title changes).
- The reserved M01 slot is now the live panel: `crates/om9-app/tests/builder_slot.rs` records
  that handover and still proves the dock reservation itself.
- `RingBuilder` is accepted as the L3-009 fixture's command-line name for the ring rail mode.
- New automated suites: `crates/om9-core/tests/builder_controls.rs` and
  `crates/om9-app/tests/builder_controls.rs`.

## L3 traceability

| L3 | Capability | State |
|----|------------|-------|
| `OM9-L3-03-11-001` | Builder Typed Input Boxes | automated_verified_manual_gate_open |
| `OM9-L3-03-11-002` | Builder Mode Tabs And Context | automated_verified_manual_gate_open |
| `OM9-L3-03-11-003` | Builder Parameter Controls And Sliders | automated_verified_manual_gate_open |
| `OM9-L3-03-11-004` | Builder Live Preview Lifecycle | automated_verified_manual_gate_open |
| `OM9-L3-03-11-005` | Builder Viewport Control Handles | automated_verified_pointer_frame_open |
| `OM9-L3-03-11-006` | Builder Handle And Panel Bidirectional Sync | automated_verified_manual_gate_open |
| `OM9-L3-03-11-007` | Builder Profile Selector Window | automated_verified_manual_gate_open |
| `OM9-L3-03-11-008` | Builder Accept Cancel Reset And Undo | automated_verified_manual_gate_open |
| `OM9-L3-03-11-009` | Builder F6 And Commandline Input Binding | automated_verified_manual_gate_open |

## Verification

See `observed.json` and the logs here. Focused commands:

- `cargo test -p om9-core --test builder_session --test builder_controls --test history_mutation`
- `cargo test -p om9-app --test builder_panel --test builder_controls --test builder_slot --test top11`
- `cargo test -p om9-kernel --features 3dm-brep --test associative_history` (real OCCT sweep)
- `cargo check --workspace --all-targets`

## Not claimed

- The M04-UI manual gate: a human has not yet reviewed the panel/handle gesture on screen.
- A time-based preview throttle: previews run synchronously on each accepted edit.
- macOS runtime for any of this: the evidence is Windows x64 only.

## Ghi chú cuối milestone (chốt 2026-10-27)

Phần này gom mọi ghi chú rải rác của `OM9-L2-03-11` vào một chỗ, để phiên sau không phải
suy lại từ log. Đọc hết mục này trước khi mở lại milestone.

### 1. Trạng thái thật và cách đọc dashboard

- Cả 9 L3 đã có code + test tự động; mỗi L3 có đủ artefact L4 (6/6, hoặc 7/7 với
  `-004`, `-005`, `-006`, `-008`, `-009` vì có thêm task adversarial `-07`).
- Dashboard hiển thị `**OM9-L2-03-11 · ✅0 🟡9 ⬜0**` — **đây là trạng thái đúng**, không
  phải làm thiếu. `🟡` nghĩa là có bằng chứng nhưng **chưa có chữ ký người review**; cả 9
  L3 đều `requires_manual_test: true`, gate `M04-UI`.
- Không được tự sửa status trong `OM9_Milestones/**` và không được tự tạo
  `ui-review.json` với `SIGNED`. Dashboard chỉ tin evidence.
- Bằng chứng per-L3: `implementation/evidence/M04/OM9-L3-03-11-00X/`
  (`source_crosswalk.json`, `api.md`, `positive.json`, `negative.json`,
  `OM9-L4-…-03/04/05/06` + `-07`).
- Tool tái lập toàn bộ gói L4: `tools/record_m04_builder_l3_l4_evidence.py`
  (chạy lại sẽ ghi đè receipt bằng lần chạy mới, kèm SHA-256).
- Ledger: `docs/openmatrix9-progress.json`, key `m04-builder-ui-l4-evidence-2026-10-27`
  (11 feature: 9 L3 builder UI + 2 L3 rail). Key cũ `m04-builder-ui-history-journal-2026-10-27`
  vẫn giữ cho slice đầu.

### 2. Cách đóng gate M04-UI (việc của con người)

1. Chạy app trên Windows, mở từng mode: Ring Rail, Rail Profile, Rail Sweep, Outside Rail,
   Ring Resizer.
2. Thao tác: đổi tab (giữ tham số riêng), kéo slider, kéo handle trong viewport, chọn
   preset ở `Library…`, gõ lệnh + `Width=`/`Diameter=`/`Target=`, Enter/Apply, Escape/Cancel,
   Reset rồi Undo/Redo, F6 trên một rail đã chọn.
3. Chụp ảnh panel + handle (đặc biệt `-005` cần pointer-frame: kéo handle trên layer khóa
   phải không di chuyển).
4. Nếu đạt: tạo `ui-review.json` trong từng thư mục L3 với
   `{"reviewer": {"status": "SIGNED", "role": "..."}, "manual_test_gate": "M04-UI"}`,
   rồi chạy `python tools/om9_milestone_dashboard.py` để 🟡 → ✅.
5. Nếu có lỗi: ghi lỗi vào `ui-review.json` (status khác `SIGNED`) và mở lại L3 tương ứng;
   không sửa receipt L4 cũ.

### 3. Phạm vi L3-001 (typed input boxes) — chỗ dễ hiểu nhầm

- `om9-core` **không có kiểu Gem**; builder family này chỉ khai báo slot `Curve`.
  Fixture generic của L3-001 có “GemRef placeholder”, nhưng nó **ngoài phạm vi** L2 này.
- Test đang chứng minh: slot `rail`/`profile`/`ring` chỉ nhận `GeometryKind::Curve`;
  Surface đưa vào slot Curve bị `Invalid("builder input kind")`; entity id được giữ nguyên.
- Đừng thêm Gem giả chỉ để khớp fixture; nếu tương lai có Gem geometry thật thì mở L3 mới.

### 4. Blocker workspace check (không thuộc L2-03-11)

- Tại thời điểm chốt, `cargo check --workspace --all-targets` **đỏ** ở
  `crates/om9-core/tests/w8_w9_w10_kernels.rs:333` (E0061): `capture_parameters` trong
  `crates/om9-core/src/cage.rs` đã đổi thành 3 tham số `(cage, points, limits)`, test cũ
  còn gọi 1 tham số. Đây là thay đổi đồng thời của agent khác (cage/inversion), không phải
  builder.
- Kỳ vọng `Unsupported("cage parameter inversion")` có thể đã lỗi thời vì `invert` giờ tính
  được cho box cage — người sửa `cage.rs` phải cập nhật test theo hành vi mới.
- Khi test đó xanh, chạy lại `cargo check --workspace --all-targets` rồi mới coi là
  workspace sạch. Các suite builder vẫn xanh độc lập.

### 5. Việc còn lại của nhóm L2-04-02 (không nằm trong L2-03-11)

- `OM9-L3-04-02-001` custom rail, `-004` four-rail profile, `-005` merge/endcap, `-006`
  bảng size vẫn `⬜`; đừng gộp vào milestone này.
- Trong L2-03-11, mọi mode đã có: tabs, slider, preview, handle, sync, selector,
  accept/cancel/reset/undo, F6 + command line.

### 6. Giới hạn kỹ thuật đã biết (đừng “sửa” mà không có L3 mới)

- `RingResizer` là **non-historical**: tạo object độc lập, ring gốc không đổi; không có
  mode cập nhật recipe cũ.
- `OutsideRail` chỉ nhận rail tròn kín; offset domain `0.1..50`, step `0.1` (min từng là
  `0.001` gây `Invalid("numeric option step")`).
- Preset library là static; `remove_preset` chỉ session-local và làm rơi preview, nên Apply
  sau đó trả `Missing("profile preset")`.
- Tab switch lấy `roles.len()` id đầu tiên từ selection, nên tab Ring Rail bỏ qua selection
  còn thừa; switch lỗi thì khôi phục tab cũ.
- Preview chạy đồng bộ mỗi edit, chưa có throttle theo thời gian (đã ghi “Not claimed”).

### 7. Môi trường build/kiểm chứng

- `CARGO_TARGET_DIR=L:\om9-target`, `-j 2` (pagefile 8 GiB; parallelism mặc định chết
  `os error 1455`).
- Test OCCT cần `--features 3dm-brep` và `PATH=L:\om9-target\debug;$env:PATH`.
- Lệnh kiểm chứng nhanh:
  - `cargo test -p om9-core --test builder_session --test builder_controls --test history_mutation`
  - `cargo test -p om9-app --test builder_panel --test builder_controls --test f6_context --test top11`
  - `python tools/public_source_audit.py` → `errors: []`
  - `python tools/om9_milestone_dashboard.py`

### 8. Tóm tắt một dòng

Code đã xong và được dùng lại (không viết lại); bằng chứng L4 đã đủ cho 9/9 L3; dashboard
đã cập nhật; **chỉ còn gate M04-UI do người ký** và blocker `cage.rs` bên ngoài milestone.

