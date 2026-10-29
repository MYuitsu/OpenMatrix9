# OM9-L2-04-19 — 07-be-jewelry-metal-weight-calculations

**Trạng thái:** đã triển khai và có L4 tự động cho cả 5 L3. Cả 5 L3 có
`manual_test_gate: AUTO_ONLY`, nên dashboard chấp nhận record
`AUTOMATED_EVIDENCE_ONLY` và hiển thị **✅5/5** — **không có chữ ký người**.

## Phạm vi đã làm

| L3 | Năng lực | L4 |
| --- | --- | --- |
| 001 | Thể tích từ closed B-Rep, quy đổi đơn vị dự án về mm | 7/7 |
| 002 | Registry density hợp kim, provenance, test-only | 6/6 |
| 003 | Khối lượng gram + pennyweight từ volume × density | 6/6 |
| 004 | Breakdown nhiều vật liệu, chống đếm trùng body | 7/7 |
| 005 | Từ chối open shell/NaN/zero, giữ report cũ | 7/7 |

## Code chính

- `crates/om9-core/src/metal_weight.rs` — `MetalMaterial`, `DensityUnit`,
  `MetalMaterialRegistry`, `SolidMeasure`, `closed_solid_volume_mm3`,
  `volume_mm3_from_project`, `MetalEstimate`, `MassPart`/`MassBreakdown`.
- `crates/om9-app/src/session/metal.rs` — feature `OM9-TOOLS-006`, command line
  `MetalWeights`/`Metal Weights`, `material=<id>`, report gram/dwt và per-body.
- `crates/om9-app/src/session.rs` — `metal_registry`, `last_metal_report`; route icon và
  command line vào cùng `calculate_metal_weights`.
- `crates/om9-app/src/session/measure.rs` — thêm `MeasureUnit::Gram`/`Pennyweight`.
- `crates/om9-core/tests/metal_weight.rs` — 11 test core.
- `crates/om9-app/tests/metal_weights.rs` — 6 test app (B-Rep thật, open shell thật).

## Điểm kỹ thuật quan trọng

- **Đơn vị:** density lưu duy nhất `g/cm^3`; `kg/m^3` phải khai báo qua `DensityUnit` và
  được chia 1000. Giá trị `15500` nhập như `g/cm^3` bị từ chối, nên không thể đọc nhầm
  `kg/m^3` thành `g/cm^3`.
- **Volume:** B-Rep đo bằng kernel theo mm; volume dự án được nhân hệ số khối
  (`factor^3`), không dùng chuyển đổi độ dài.
- **Mass:** `grams = volume_mm3 / 1000 * density_g_cm3`; `1 dwt = 1.55517384 g`.
- **Multi-material:** mỗi body một dòng; `instances` là cách duy nhất để khai báo số bản;
  trùng body id bị từ chối. Assignment của app phải phủ đúng selection.
- **Lỗi:** open shell → `Invalid("metal weight requires a closed solid")`; volume
  NaN/0/âm → `Invalid("metal volume")`; material thiếu → `Missing("metal material density")`.
  Report chỉ được gán sau khi mọi body thành công, nên lỗi giữ report cũ và không đổi
  document/Undo.
- **Read-only:** command không tạo transaction; test so sánh document và `undo_len()`.

## Kiểm chứng

- `cargo test -p om9-core --test metal_weight -j 2` — **11/11 pass**.
- `cargo test -p om9-app --test metal_weights -j 2` — **6/6 pass** (B-Rep box 10×5×2 =
  100 mm³, open shell bị từ chối, hai body 100/200 mm³ = 1.0/1.6/2.6 g).
- `python tools/record_m04_metal_weight_l3_l4_evidence.py` — 0 failures; 5 gói L4 + 5
  AUTO_ONLY review.
- `python tools/om9_milestone_dashboard.py` — L2-04-19 ✅5 🟡0 ⬜0; toàn milestone
  M04 tăng 5 L3 done.
- `cargo test -p om9-core -j 2` — toàn bộ core pass (bao gồm 11 test metal weight).
- `cargo check --workspace --all-targets -j 2` — sạch.
- `python tools/public_source_audit.py` — `errors: []`.

## Ghi chú cuối milestone (chốt 2026-10-27)

- **Tái dùng:** dùng `om9_kernel::creation::measure` và `transform_brep` sẵn có; không
  viết lại phép đo B-Rep. Chỉ thêm tầng unit/density/mass còn thiếu.
- **Cổng AUTO_ONLY:** L3 plan ghi `manual_test_gate: AUTO_ONLY`; record
  `OM9-L3-04-19-*.ui-review.json` do generator tạo có
  `reviewer.status: AUTOMATED_EVIDENCE_ONLY`, không phải chữ ký người. File
  `requires_human_review: true` trong handoff vẫn được ghi nhận là chưa có người review.
- **Chưa làm (không tuyên bố):** builder panel Metal Weights với các toggle GV/User/
  Use Object Materials và nút Calculate như manual; hiện command tính ngay từ selection.
  Density built-in là giá trị nominal kèm provenance và ±1%, không phải số liệu lô nhà
  cung cấp.
- **Bước tiếp theo:** nếu muốn đóng `requires_human_review`, chạy app, thử icon
  `OM9-TOOLS-006` với box kín, open shell và hai body khác vật liệu, rồi ký record người
  riêng. Trước khi chốt toàn workspace, xử lý các test `file_widgets` đang fail độc lập.

## Không tuyên bố

- Không tuyên bố đã có human review hoặc đã đóng gate M04-UI cho phần panel.
- Không tuyên bố mọi vật liệu thực tế có density chính xác; registry là reference.
- Không sửa status trong cây `OM9_Milestones`.
