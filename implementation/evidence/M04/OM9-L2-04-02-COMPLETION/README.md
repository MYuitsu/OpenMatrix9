# OM9-L2-04-02 — 05-febe-ring-rail-profile-and-resize-tools

**Trạng thái:** đã hoàn tất phần code + L4 tự động cho cả 9 L3; **cổng M04-UI vẫn mở**
(không có `ui-review.json` có chữ ký). Dashboard giữ L2 ở 🟡 9/9, không tự nâng ✅.

## Phạm vi đã làm

| L3 | Năng lực | Bằng chứng tự động |
| --- | --- | --- |
| 001 | Ring size US + đường kính mm | `OM9-L3-04-02-001` (6 L4) |
| 002 | Outside ring rail | `OM9-L3-04-02-002` (6 L4) |
| 003 | Profile placer trên rail | `OM9-L3-04-02-003` (6 L4) |
| 004 | Custom/advanced rail kín, phẳng, không tự cắt | `OM9-L3-04-02-004` (7 L4) |
| 005 | Four rail profile + Sweep4 | `OM9-L3-04-02-005` (7 L4) |
| 006 | Merge profile + end cap | `OM9-L3-04-02-006` (7 L4) |
| 007 | Ring resizer | `OM9-L3-04-02-007` (7 L4) |
| 008 | Sweep associative sau khi đổi size/profile | `OM9-L3-04-02-008` (7 L4) |
| 009 | Handle rail/profile, drag + Apply một lần | `OM9-L3-04-02-009` (7 L4) |

Tổng: 42 receipt tự động (4–5 receipt/gói, phần còn lại của 6/7 L4 là fixture
`positive.json`/`negative.json` và crosswalk); mỗi gói có `source_crosswalk.json`,
`api.md`, `positive.json`, `negative.json`, `README.md` và receipt
`OM9-L4-…-03/04/05/06/07`.

## Code chính

- `crates/om9-core/src/ring_size.rs` — bảng US size 1..15 đóng băng kèm provenance,
  `RingSizeStandard::{Us, from_id, diameter_mm, nearest_size, table}`.
- `crates/om9-core/src/producers.rs` — `RingRailProducer` đọc `standard` + `us_size`;
  `RailProfileProducer` đọc `position` (0..1) và `roll_deg`; `RailSweepProducer` kiểm
  `closed_rail` trước khi sweep.
- `crates/om9-core/src/curve_ops.rs` — `validate_ring_rail` (kín/phẳng/không tự cắt/không
  đoạn dài 0), `merge_profile_sections` (đồng phẳng, cùng winding, nối liền), fallback
  tangent tại seam NURBS.
- `crates/om9-core/src/builder.rs` — field `standard`, `us_size`, `position`, `roll_deg`,
  `closed_rail`; handle thứ ba cho profile; `BuilderHandle::scale`.
- `crates/om9-app/src/session/ring_tools.rs` — dispatcher cho `OM9-TOOLS-002/003/004/032/033/034/035`;
  `004` chọn four-rail khi ≥5 curve, ngược lại custom rail.
- `crates/om9-app/src/session/surface_edit.rs` — `OM9-TOOLS-032` là alias của
  `OM9-TOOLS-004`, cùng command/label.
- `crates/om9-app/src/ui/chrome.rs` — field numeric gửi đúng `Dimension`; drag handle nhân
  `handle.scale`.

## Kiểm thử đã chạy

- `cargo test -p om9-core --test ring_tools -j 2` — 6/6 pass.
- `cargo test -p om9-app --test ring_tools -j 2` — 10/10 pass.
- `cargo test -p om9-core --test builder_session -j 2` — 8/8 pass.
- `cargo test -p om9-app --test builder_controls --test builder_panel --test builder_styles --test builder_slot -j 2`
  — pass.
- `cargo test -p om9-core -j 2` — toàn bộ core pass (bao gồm `builder_controls`,
  `producers`, `ring_tools`, `builder_session`).
- `cargo check --workspace --all-targets -j 2` — sạch.
- `python tools/public_source_audit.py` — `errors: []`.
- `cargo test -p om9-app -j 2` — phần lớn pass; **7 test `file_widgets` fail vì
  `Stale("file operation document/context")`**, không liên quan L2-04-02 (không có thay
  đổi file/undo trong phạm vi này). Cần xử lý riêng trước khi chốt toàn workspace.
- `python tools/record_m04_ring_tools_l3_l4_evidence.py` — 0 failures, ghi 7 gói mới
  (001/003/004/005/006/008/009).

## Ghi chú cuối milestone (chốt 2026-10-27)

- **Tái dùng code trước** thay vì viết lại: builder session, producer registry, bốn rail
  metadata (`OM9-TOOLS-004`), resizer và outside rail đã có được nối vào dispatcher chung;
  chỉ thêm phần còn thiếu (US size, position/roll, validate custom rail, merge/endcap,
  alias 032).
- **Không sửa status trong cây plan** `OM9_Milestones`; dashboard đọc bằng chứng.
- **Giới hạn có chủ đích:** end cap curve tạo planar surface; cap B-Rep hở dùng
  `om9_kernel::editing::cap_planar` dưới feature `occt-brep`, ngoài feature trả
  `Unsupported`. Sweep trong app dùng registry OCCT nên output là B-Rep; test core dùng
  backend analytic để kiểm surface.
- **Cổng người vẫn mở:** cần chạy app, thao tác 9 L3 (đặc biệt tab Builders, icon
  `OM9-TOOLS-002/003/004/032/033/034/035`, drag handle, Apply/Cancel), rồi ký
  `ui-review.json` nếu đạt. Không tự ký.
- **Bước tiếp theo:** review thủ công M04-UI; sau đó chạy lại
  `python tools/om9_milestone_dashboard.py` và cập nhật ledger. Trước khi chốt toàn
  workspace, chạy `cargo test` đầy đủ và `python tools/public_source_audit.py`.

## Không tuyên bố

- Không tuyên bố đã được người review UI, chưa có ảnh chụp/gesture frame.
- Không tuyên bố B-Rep cap cho mọi cấu hình; chỉ đường cong kín và shell hở theo feature.
- Không tuyên bố các L3 khác trong M04 đã xong.
