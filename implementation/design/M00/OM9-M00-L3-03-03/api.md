# Hợp đồng picking và gesture (gate M00-L3-03-03)

## 1. Picking

| Tầng | Phép thử | Ngưỡng | Đo |
| --- | --- | --- | --- |
| object | tia vs bán kính bao | bán kính | trúng tâm 0,0024 ≤ 5,0024; góc trượt |
| face | Möller–Trumbore, `t` dương nhỏ nhất | không | tam giác 4861 tại `t=9,147` (kỳ vọng 9,0) |
| edge/vertex | khoảng cách tia–đoạn/điểm | `2·tan(fov/2)·depth/height · pixels` | trúng ở 0,000/1e-6 với 0,0207; lệch 20 px trượt |

Đầu vào: `screen` (pixel **logic**), `camera`, `viewport`, `tolerance_pixels`, `PickIntent`.
Đầu ra: `PickHit` (entity + thành phần). Thứ tự bắt buộc: object → face → edge/vertex (rẻ trước).

## 2. Gesture

- Preview: `AffineGesture::update` **không** ghi document.
- Cancel: `AffineGesture::cancel` trả scene nguyên trạng, **không** đụng undo/redo.
- Commit: `prepare` → `TransactionCoordinator::commit` = **một** bước undo.
- Gumball: `Session::begin/drag` cộng dồn; `move_mutations` = `Mutation::Placement`; `copy_mutations` =
  `Mutation::Insert` với `ObjectId` mới.

## 3. Bất biến

1. Không có mutation nào được tạo ra khi chỉ preview hoặc khi cancel.
2. Một lần commit = một bước undo; redo phải tái lập đúng placement.
3. Picking là hàm thuần của (screen, camera, viewport, tolerance, intent).
4. DPI: toạ độ logic cho layout/picking, pixel vật lý cho render target; ngưỡng pixel không nhân scale.
