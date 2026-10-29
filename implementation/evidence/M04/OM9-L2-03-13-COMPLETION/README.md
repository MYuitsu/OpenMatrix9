# M04 L2-03-13 — Matrix History associative updates

This directory records the Matrix History slice of `OM9-L2-03-13`
(`04-febe-matrix-history-associative-updates`). It re-uses the history graph, typed
recipes, producer registry and one-transaction journal that already existed; no second
history store or geometry engine was added.

## What changed

- `om9_core::history_ops`: `BrokenLink` + `broken_links` (missing input roles, missing
  output ids, unresolved/failed states), `detach_object` (lock-aware), `refresh_inputs`
  (re-stamp a Dirty consumer against the current source) and `has_pending`.
- `om9_core::builder::BuilderSession::adopt_history`: a builder reopened from history keeps
  the selected object id and recipe id, so Apply updates that feature instead of inserting a
  copy.
- `Controller` history state: `history_record`, `history_update`, `history_rebuilding`,
  `history_notice`, `history_window_requested`.
- New `crates/om9-app/src/session/history.rs`: record/update toggles, `history_entries`,
  `history_broken_links`, `clear_object_history`, `begin_builder_from_history`,
  `apply_history_parameter`, command-line and feature-ID dispatch.
- `builder_apply` stamps the record/update policy on new recipes, omits the History mutation
  when recording is off, and edits the adopted object (Geometry + HistoryUpdate) when the
  builder was reopened from history.
- `recompute_history` now rebuilds the dirty consumers in execution order, re-stamping each
  consumer's inputs first, and reports complete/failed with the node id.
- `begin_file_intent` refuses New/Open/window-close while a rebuild owns the document or a
  recipe is still `Pending`.
- UI: the Info/Settings History icons are click-wired, and the History window lists recorded
  features, their state, the broken rows, and per-row Edit/Clear actions.
- Command line: `HistoryRecord`, `HistoryUpdate`, `HistoryReport`, `ClearHistory`,
  `EditHistory` (case-insensitive); the status line consumes one history notice per command.

## L3 traceability

| L3 | Capability | State |
|----|------------|-------|
| `OM9-L3-03-13-001` | Matrix History Record Toggle | automated_verified_manual_gate_open |
| `OM9-L3-03-13-002` | Matrix History Update Toggle | automated_verified_manual_gate_open |
| `OM9-L3-03-13-003` | Matrix History Broken Link Report | automated_verified_manual_gate_open |
| `OM9-L3-03-13-004` | Matrix Clear Single Object History | automated_verified_manual_gate_open |
| `OM9-L3-03-13-005` | Edit Existing Builder By History | automated_verified_manual_gate_open |
| `OM9-L3-03-13-006` | History Commandline And Preview Status | automated_verified_manual_gate_open |

Per-L3 L4 packages: `../OM9-L3-03-13-00X/` (`source_crosswalk.json`, `api.md`,
`positive.json`, `negative.json`, `OM9-L4-…-03/04/05/06`, plus `-07` for 001–005).

## Verification

Focused commands (run at the revision that produced the receipts):

- `cargo test -p om9-app --test history_updates` → 8 passed
- `cargo test -p om9-core --test history_links` → 6 passed
- `cargo test -p om9-app --test builder_controls --test builder_panel --test builder_styles` → 27 passed
- `cargo check --workspace --all-targets` → clean

Reproduce the L4 packages with `python tools/record_m04_history_l3_l4_evidence.py`; it runs
the tests again and rewrites each receipt with the new SHA-256.

## Not claimed

- The M04-UI manual gate: no human has reviewed the History window, the sidebar icons or
  the command-line rendering.
- A time-based/asynchronous rebuild: `recompute_history` is synchronous, so the UI paints
  only the terminal state; the command-event log records `Started / updating` then
  `Finished / complete` or `Warn / failed at <id>`, and `Pending` recipes are the queued
  state the close guard protects.
- macOS runtime: evidence is Windows x64 only.

## Ghi chú cuối milestone (chốt 2026-10-27)

Phần này gom mọi ghi chú của `OM9-L2-03-13` vào một chỗ. Đọc hết mục này trước khi mở lại
milestone.

### 1. Trạng thái thật và cách đọc dashboard

- Cả 6 L3 đã có code + test tự động; mỗi L3 có đủ artefact L4 (6/6 cho `-001` và `-006`,
  7/7 cho `-002`…`-005`).
- Dashboard hiển thị `**OM9-L2-03-13 · ✅0 🟡6 ⬜0**` — **đây là trạng thái đúng**. Cả 6 L3
  đều `requires_manual_test: true`, gate `M04-UI`, nên chưa thể ✅ nếu không có
  `ui-review.json` với `reviewer.status: SIGNED`.
- Không sửa status trong `OM9_Milestones/**`; không tự tạo chữ ký. Dashboard chỉ tin
  evidence.
- Tool tái lập gói L4: `tools/record_m04_history_l3_l4_evidence.py`.
- Ledger: `docs/openmatrix9-progress.json`, key `m04-history-associative-ui-2026-10-27`.

### 2. Cách đóng gate M04-UI (việc của con người)

1. Chạy app Windows, tạo một ring rail (Record on) và một rail profile từ rail đó.
2. Bấm icon History (Record/Update) ở sidebar hoặc gõ `HistoryRecord`/`HistoryUpdate`:
   kiểm tra trạng thái icon và dòng status.
3. Tắt Update, sửa `Diameter` của ring trong History window (Edit → đổi tham số → Apply):
   rail phải đứng yên, profile cũ giữ hình, dòng recipe chuyển Dirty; bật Update lại và
   kiểm tra một lần rebuild đưa cả hai về Clean.
4. Xóa rail nguồn (hoặc mở lại project có tham chiếu mồ côi): mở `HistoryReport`, phải thấy
   dòng RailProfile với input `rail` thiếu, không có sửa ngầm.
5. Chọn profile, bấm Clear trong History window (hoặc `ClearHistory`): profile còn hình,
   recipe Detached, ring vẫn Clean; Undo phải khôi phục association.
6. Chọn ring đã record, Edit: builder mở đúng `Diameter` đã lưu; Apply đổi diameter phải
   cập nhật **cùng object**, không sinh object thứ hai.
7. Đóng cửa sổ khi đang có recipe Pending/rebuild: phải bị chặn với `Busy`, tài liệu không
   đổi.
8. Nếu đạt: tạo `ui-review.json` trong từng thư mục L3 với
   `{"reviewer": {"status": "SIGNED", "role": "..."}, "manual_test_gate": "M04-UI"}` rồi
   chạy `python tools/om9_milestone_dashboard.py`.
9. Nếu lỗi: ghi lỗi vào `ui-review.json` (status khác `SIGNED`), mở lại L3 tương ứng; không
   sửa receipt L4 cũ.

### 3. Hành vi kỹ thuật cần nhớ

- **Record off** không ghi recipe: transaction chỉ có `Insert`, Undo vẫn xóa được geometry.
  Toggle bị từ chối khi builder đang mở (`Invalid("history record during edit transaction")`).
- **Update off**: sửa tham số chỉ đánh dấu recipe `Dirty("parameter edit")`; geometry giữ
  nguyên. Bật lại Update kiểm tra cycle (`execution_order`) và `broken_links` trước khi đổi
  cờ.
- **Rebuild dây chuyền**: `recompute_history` chạy recipe được sửa rồi lần lượt các recipe
  Dirty theo `plan_recompute`; trước mỗi bước gọi `refresh_inputs` để stamp nguồn khớp
  geometry revision hiện tại (nếu không, chính kết quả upstream bị coi là stale).
- **Edit by history**: builder adopt `ObjectId` + `HistoryId`; Apply phát `Geometry` +
  `HistoryUpdate` (một transaction), không `Insert`. Chỉ 5 command builder được mở lại;
  schema_version khác 1 hoặc command khác là lỗi có kiểu.
- **Clear history**: detach đúng một recipe, giữ entity và published result (mô hình
  transaction không có mutation xóa result); recipe lock bị từ chối.
- **Broken link report**: dẫn xuất từ graph mỗi lần đọc, tách `missing_inputs` (object mất)
  khỏi `changed_inputs` (stamp cũ), kèm `missing_sources`/`changed_sources` là ObjectId nguồn;
  `Dirty/Unresolved/Failed` cũng là warning. History window in recipe id + detail
  (`missing input <role> (source <id>)`, `stale: <reason>`) và nhắc remediation Edit/Clear;
  tham chiếu mồ côi sau reopen vẫn hiện, không có “silent repair”.

### 4. Giới hạn đã biết (đừng “sửa” mà không có L3 mới)

- Rebuild đồng bộ, chưa có async/background; UI chỉ kịp vẽ trạng thái cuối. Event log vẫn ghi
  đủ chuỗi `Started / History / updating <id>` → `Finished…complete` hoặc
  `Warn…failed at <id>: <reason>`, và status line hiển thị complete/failed kèm node id.
- `recompute_history` trả `Ok(RecomputeOutcome::Failed)` khi producer lỗi; UI/status đọc
  outcome, không coi đó là thành công. Recipe giữ kết quả cũ và chuyển `Failed`.
- Edit-by-history chỉ dành cho builder family (RingRail/RailProfile/RailSweep/OutsideRail/
  RingResizer); recipe do command khác tạo ra chỉ xem/clear được.
- Detach không xóa published result; nếu tương lai cần “quên hẳn” thì phải thêm mutation
  riêng, không tái dùng HistoryUpdate.
- History window là egui window thường, chưa có test frame tự động cho hit-test icon/window;
  test hiện tại đi qua đúng `Controller` method mà icon/nút gọi.

### 5. Môi trường build/kiểm chứng

- `CARGO_TARGET_DIR=L:\om9-target`, `-j 2` (pagefile 8 GiB; parallelism mặc định chết
  `os error 1455`).
- Test OCCT cần `--features 3dm-brep` và `PATH=L:\om9-target\debug;$env:PATH`.
- Lệnh nhanh:
  - `cargo test -p om9-app --test history_updates`
  - `cargo test -p om9-core --test history_links`
  - `cargo check --workspace --all-targets`
  - `python tools/public_source_audit.py` → `errors: []`
  - `python tools/om9_milestone_dashboard.py`

### 6. Tóm tắt một dòng

Code đã xong và dùng lại history graph/builder sẵn có; 6/6 L3 có L4 evidence; dashboard đã
cập nhật; **chỉ còn gate M04-UI do người ký**.
