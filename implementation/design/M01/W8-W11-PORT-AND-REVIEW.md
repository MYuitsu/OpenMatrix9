# W8–W11 — port tiếp và phân loại lại bucket review

Ngày: 2026-10-17 · Trạng thái: **W8/W9/W10 kernel xong (đợt 1); W11 đã phân loại xong, 11 module portable còn lại chưa port**

Bốn wave cuối theo [kế hoạch chuyển `rust/`](RUST-LEGACY-MIGRATION.md). Nguyên tắc giữ nguyên:
**port code trước, phần nối (command/UI/native) làm sau**, và mỗi L3 chỉ được đánh dấu nghiệm thu khi
phủ đủ oracle của nó.

## W8 — exchange/3dm (5 module, 43 KB)

| Legacy | Hành vi giữ lại | Target |
| --- | --- | --- |
| `phase3_modeling.rs` (portable theo chính ghi chú module: "no kernel or host pointers") | chính sách năng lực cho thao tác modeling: `Kind`, `Operation`, `Reason` + **nguyên văn** thông báo người dùng, `InputFacts`, `validate_inputs`/`capability`, `validate_result_count`, `KERNEL_TOLERANCE`, `join_uses_surface_adapter`, `surface_options` (style/section/point count/tolerance/input count) | `modeling_capability` |

`retained_archive`/`core_3dm_archive`/`modeling_exchange`/`core_3dm` vẫn `PARTIAL` — `om9-exchange` đã
sở hữu đường retained 3DM; phần archive-writing rộng hơn chưa port.

## W9 — cage/subd (2 module, 51 KB)

| Legacy | Hành vi giữ lại | Target |
| --- | --- | --- |
| `cage.rs` (phần portable) | record cage sở hữu + luật admission (knot clamped, multiplicity ≤ degree+1, weight > 0, lưới điểm khớp count) + **đánh giá rational tensor** (U fastest) qua `curve_build::basis` + `Region` (Global/Local) | `cage::{Cage, Region}` |

**Không port**: `om9_cage_*` C entry points, capture/binding lifecycle, và **nghịch đảo tham số**
(`capture_parameters` trả `Unsupported("cage parameter inversion")` — cần solver native, không xấp xỉ).
`native_cage_capability.rs` là tầng native.

## W10 — core ui/scene (13 module, 48 KB)

| Legacy | Hành vi giữ lại | Target |
| --- | --- | --- |
| `core_angle.rs` | góc giữa hai đường định hướng, 4 điểm, `atan2(|a×b|, a·b)` (ổn định gần 0° và 180°), trả **độ** | `measure::angle_degrees` |
| `core_distance.rs` | khoảng cách hai điểm + đơn vị hiển thị (mm/cm/m/in/ft) | `measure::{distance, distance_mm}` |

Khác biệt có chủ ý: legacy trả `Option`/`NaN`; target trả `Error` có nhãn (`measurement direction`,
`measurement coordinate`, `measurement angle`). Biên `1e9` của legacy giữ nguyên
(`MAX_MEASURE_COORDINATE`). `core_snaps`/`core_views`/`core_mouse`/`core_keyboard`/`viewport_title`… vẫn
`PARTIAL`: hành vi nằm trong `om9-app` UI, không tách module 1:1.

## W11 — bucket review (33 module, 246 KB): đã đọc và phân loại

Công cụ tái lập: [`implementation/tools/classify_w11.py`](../tools/classify_w11.py). Luật đọc (ghi trong
chính script):

1. `*_ffi.rs` hoặc module chỉ có `extern "C"`/`no_mangle` với ≤ 2 API Rust → **LEGACY_HOST** (vỏ C ABI).
2. Chạm `TopoDS`/`BRep`/`TopExp`/`opencascade` → **BLOCKED** (chờ `OM9-L2-01-05` + quyết định M00/OCCT).
3. Target đã sở hữu hành vi → **COVERED**.
4. Không có public item → **STUB**.
5. Còn lại → **PORT** (logic portable, target chưa có).

Kết quả: **BLOCKED 5 · LEGACY_HOST 14 · PORT 9 · COVERED 2 · STUB 3 = 33** (khớp inventory: PORT 71 / PARTIAL 29 / COVERED 3 / BLOCKED 6 / LEGACY_HOST 31 / TEST 8 / STUB 3).

| Nhóm | Module |
| --- | --- |
| BLOCKED (OCCT) | `solid.rs` 60,4 · `geometry_contract.rs` 19,9 · `topology_snapshot.rs` 8,9 · `edge_geometry.rs` 4,2 · `shape_validation.rs` 3,1 |
| COVERED | `solid/geometry.rs` → `om9-core/geometry.rs`; `lib.rs` → `om9-archive/lib.rs` |
| STUB | `toolbar.rs`, `workbench.rs`, `gem.rs` |
| PORT (11) | `phase2_snap.rs` 14,2 · `hatch_loop_input.rs` 8,0 · `source_attributes.rs` 7,5 · `phase2_session.rs` 4,3 · `native_input_tree.rs` 3,9 · `geometry_frame.rs` 2,9 · `worker_budget.rs` 2,3 · `state.rs` 2,0 · `topology_snapshot.rs` 8,9 · `edge_geometry.rs` 4,2 · `ui/mod.rs` 0,6 |
| LEGACY_HOST (14) | `phase2_ffi.rs`, `ffi.rs`, `phase2_snap_ffi.rs`, `geometry_contract_ffi.rs`, `edit.rs`, `native_*_capability.rs` (`solid`/`edit`/`rebuild`), `*_ffi.rs` còn lại |

Ba lưu ý khi đọc bảng:

- `topology_snapshot.rs`/`edge_geometry.rs`/`shape_validation.rs` **không** chạm OCCT trực tiếp nhưng dữ
  liệu là topology solid → công cụ có danh sách ngoại lệ riêng (`SOLID_TOPOLOGY`) xếp chúng `BLOCKED`,
  để lần sau không "port" nhầm một snapshot rỗng nghĩa.
- `phase2_snap.rs` (14,2 KB) là **snap index** — ứng viên W11 giá trị nhất (M02 osnap), nhưng cần chốt
  nguồn dữ liệu snap trước khi port.
- 9 module PORT còn lại **chưa port**: W11 là wave đọc/phân loại, không phải wave code.

## Test (12)

`crates/om9-core/tests/w8_w9_w10_kernels.rs`:

- **measure**: góc 90°/45°/0°/180° đúng (≤1e-9); từ chối đường dài 0, toạ độ > 1e9, NaN; distance đổi đơn
  vị (25,4 mm = 1 in = 2,54 cm = 0,0254 m) và `distance_mm`;
- **modeling capability**: boolean cần solid kín; Loft/Sweep chỉ nhận curve; Join không trộn curve với
  surface; Sweep2 cần 3 input khi selection hoàn tất; TwoObjects đúng 2; Explode cần ≥ 2 component;
  selection chưa xong chỉ từ chối input xấu/protected; `surface_options` theo style/section/point
  count/tolerance/count; `refusal` trả đúng nguyên văn; wire round-trip;
- **cage**: cage đơn vị là ánh xạ identity tại 5 bộ tham số; cage scale+dịch map đúng khối tham số; weight
  đều 1 và 1000 cho cùng kết quả (chuẩn hoá); từ chối lưới điểm sai, weight ≤ 0, điểm NaN, knot không
  clamped, tham số ngoài domain; `Region` validate; nghịch đảo trả `Unsupported`.

## Còn lại

- **W8**: archive writing rộng hơn retained path (thuộc `om9-exchange`).
- **W9**: capture/binding lifecycle + nghịch đảo tham số (native solver).
- **W10**: 10 module `PARTIAL` (hành vi đã nằm trong `om9-app` UI); `core_view_tabs` (4,1 KB) chưa port.
- **W11**: 9 module PORT ở trên — port dần khi có nguồn dữ liệu (snap, topology) và chủ sở hữu L3.
- Không L3 nào được đánh dấu nghiệm thu trong đợt này: W8–W11 vẫn là **kernel port**, đóng L3 cần
  tầng command/UI/native tương ứng (xem lý do trong W7 note).
