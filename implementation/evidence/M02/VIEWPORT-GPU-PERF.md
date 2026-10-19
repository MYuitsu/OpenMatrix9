# Hiệu năng viewport GPU — chọn object không còn dựng lại cảnh

Ngày: 2026-10-18.

## Vấn đề

Đường GPU dựng lại toàn bộ `SceneBatch` mỗi khi **selection revision** đổi, vì hiệu ứng highlight nằm
trong cùng batch. Với cảnh lớn, mỗi lần click chọn là một lần upload lại toàn bộ hình học → khựng.

## Sửa

- `SceneBatch::render_over_into`: pass với `LoadOp::Load` thay vì clear, để vẽ **lên trên** cảnh đã có.
- `GpuScene` tách làm hai batch:
  - `batch` — toàn cảnh, dựng lại chỉ khi **scene epoch** hoặc **CPlane revision** đổi.
  - `highlight` — chỉ các object đang chọn, dựng lại khi **selection revision** đổi; vẽ sau bằng
    `render_over_into` với màu xanh alpha 0.55 (blend).
- `selection_frame` cache kết quả theo `(selection revision, camera revision, kích thước viewport)` trong
  `MatrixApp`, nên không chiếu lại toàn bộ đỉnh của vùng chọn mỗi frame.

## Đo

`measure_selection_rebuild_cost` (release, chạy riêng):

| Việc | Thời gian |
| --- | --- |
| Dựng lại cảnh 8 000 cube (hành vi cũ mỗi lần chọn) | **54,61 ms** |
| Dựng lại highlight cho 1 object (hành vi mới) | **0,014 ms** |

Chênh **~3 900 lần**; click chọn trong cảnh 8 000 object không còn khựng.

## Test

- `selection_overlay`: 6/6 (khung chọn, gumball hiện khi chọn, clear, Ctrl toggle, window/crossing, Esc huỷ).
- `gumball_viewport` 3/3, `gumball_interaction` 5/5.
- clippy sạch cho `om9-render` + `om9-app` (cả `--features gpu-sphere`).

## Còn mở

- Các L3 còn thiếu của `OM9-L2-01-07` (-003 edge/vertex, -005 popup, -006 hover) và `OM9-L2-03-03`
  (-003 popup, -005 align/relocate, -006 numeric edit).
