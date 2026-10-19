# 3DM (openNURBS) → OM9 mesh → viewport

Ngày: 2026-10-18. Crate: `crates/om9-3dm`. Link: openNURBS pin `eb92af3b…` (static) + zlib/Shlwapi/Usp10/shell32/advapi32.

## Đường đi

1. `write_fixture(path, n)` — openNURBS ghi archive version 5 với `n` mesh (cube 8 đỉnh / 12 tam giác).
2. `read_meshes(path)` — C ABI đếm trước rồi đọc một lần; mỗi mesh thành `om9_core::geometry::Mesh`
   (`vertices`, `faces` tam giác; quad của openNURBS được tách đôi).
3. `flatten(mesh)` — `[f32;3]` + `u32` cho `SceneBatch`.
4. `SceneBatch::render_into` — một pass, một arena.

## Đo (release, 960×540)

| Việc | Kết quả |
| --- | --- |
| Đọc 500 mesh từ 3DM | **18,6 ms** (~37 µs/mesh) |
| Vẽ 500 mesh (tắt culling, 500 draw) | **0,19 ms/khung** |
| Vẽ 500 mesh (bật culling) | 0,14–0,19 ms/khung tuỳ camera |

## Kiểm chứng hình học

- 4 mesh trong fixture → 4 `Mesh`, mỗi mesh **8 đỉnh / 12 tam giác**, tổng 48 tam giác.
- Cube thứ 4 có `max_x > 3` (dịch 1,6 mỗi cube) → dữ liệu dịch chuyển được giữ đúng.
- File không tồn tại → **lỗi**, không import rỗng.
- Culling: test 500 object trong khung hình → **0 bị cull** (đúng); 1 object sau camera → bị cull.

## Giới hạn

- Đọc cả file hai lượt (đếm rồi fill); file rất lớn nên cache handle nếu cần.
- Chưa đọc UV/material/layer: mới có hình học tam giác.
- Chuẩn hoá về `f32` cho GPU; `om9_core::Mesh` giữ `f64`.

## Ngân sách import — vì sao là những con số đó

Bản đầu tôi đặt `20M đỉnh / 60M corner` chỉ vì tròn số. Hai con số đó **không suy ra từ đâu**, và chúng
vượt xa ngân sách của repo (`om9_core::budget::Limits::default().max_bytes = 256 MiB`, `max_points = 2M`).
Đã sửa thành suy ra từ bộ nhớ:

| Hạng mục | Byte mỗi đơn vị | Ghi chú |
| --- | --- | --- |
| đỉnh | ~60 B | `f64` positions trong bridge (24) + bản `Mesh` (24) + bản `f32` cho GPU (12) |
| corner tam giác | ~8 B | `u32` bridge + `u32` trong `Mesh` |
| tỉ lệ corner/đỉnh | ~6 | mặt kín tam giác: F ≈ 2V, corner = 3F |

⇒ ~**108 B/đỉnh**. `ImportBudget::from_memory(bytes)` chia cho 108:

| Ngân sách | Đỉnh tối đa | Corner tối đa | Dùng khi |
| --- | --- | --- | --- |
| 75% RAM trống (`detect()`, mặc định) | bytes/108 | 6× | thực tế |
| 256 MiB (`document()`, dự phòng) | 2 485 729 | 14 914 374 | khi không đọc được RAM |
| 1 GiB (`sculpt()`, phải xin rõ) | 9 942 917 | 59 657 502 | sculpt mật độ cao |
| 10 800 B (test) | 100 | 600 | kiểm tra biên |

File cũng bị chặn: file lớn hơn `max_bytes` bị từ chối **trước khi đọc**, để không tốn thời gian parse.
Test: `the_budget_is_derived_from_memory_and_rejects_an_oversized_import`,
`a_file_larger_than_the_budget_is_refused_before_reading`.

## Trần theo 75% RAM trống và thông báo cho người dùng

`ImportBudget::detect()` = **75% RAM vật lý còn trống** (sàn 64 MiB); đây là mặc định. Khi vượt:

- `ImportError::Budget(BudgetReport)` mang **cả hai con số**: cần bao nhiêu và được bao nhiêu.
- `Display` in câu người dùng đọc được, ví dụ:
  - `3DM import refused: the archive is 48.0 MiB but the import budget is 32.0 MiB (75% of the free memory at the time of the import)`
  - `3DM import refused: the archive needs 32000 vertices and 96000 triangle corners, but the budget allows 16000 vertices and 96000 corners; close files or raise the budget`
- File bị chặn theo byte **trước khi đọc**; hình học bị chặn ngay sau lần đếm (chưa cấp phát mảng lớn).

Test: `the_default_budget_takes_three_quarters_of_the_free_physical_memory`,
`a_refused_import_names_both_limits_for_the_user` (7 test tổng trong `om9-3dm`).

## Jewelry scene: nhiều nhẫn + nhiều kiêm cương, 4 view

Generator: `write_jewelry_fixture(path, rings, stones, ring_u, ring_v)` — mỗi nhẫn là **torus** (`ring_u` ×
`ring_v`), mỗi kiêm cương là **bipyramid 16 cạnh**; đặt trên lưới 12 mm.

Test: `crates/om9-3dm/tests/jewelry_scene.rs`, tham số theo env `OM9_JEWELRY_RINGS/STONES/U/V`.

- Đo: ghi file, đọc file (import), upload vào arena, rồi **4 camera khác nhau** mỗi frame **xoay 3°** và
  **zoom 0,92/1,08** xen kẽ — mô phỏng rotate/move/zoom thật.
- Kết quả ghi ra `jewelry: ... per-view ms=[...]`.
