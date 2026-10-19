# Gumball numeric edit (OM9-L2-03-03-006)

Ngày: 2026-10-18.

## Lệnh

Gõ trong CMD, áp dụng cho **selection** hiện tại, quanh **pivot gumball**, **một transaction**:

| Lệnh | Nghĩa |
| --- | --- |
| `MoveX 5`, `MoveY -2`, `MoveZ 0.5` | dời theo trục thế giới |
| `Move 1,2,3` | dời theo vector |
| `RotateZ 30`, `RotateX -90`, `RotateY 45` | quay quanh trục, đơn vị độ |
| `Scale 2` | phóng đều quanh pivot |
| `ScaleX 1.5`, `ScaleY 0.9`, `ScaleZ 2` | phóng một trục |

Sai cú pháp, thiếu selection, factor ≤ 0 → lỗi, document không đổi.

## Điều đã phát hiện khi test

`Point`/`Line` lưu toạ độ **trong geometry** với placement identity, nên chỉ đọc `placement` là không đủ:
test đọc **vị trí thế giới** (`placement * geometry`) — đúng thứ viewport vẽ. Ma trận `Transform` là **row-major,
translation ở cột 3**; ma trận scale đều phải theo quy ước đó (lần đầu viết sai hàng → `Invalid("affine matrix")`).

## Test

`crates/om9-app/tests/gumball_numeric.rs` — **4/4**:

- `MoveX 5`: mọi object dời đúng +5 theo X, **một** journal entry, Undo trả lại vị trí cũ.
- `RotateZ 90` quanh pivot (5,0,0): hai đầu về (5,-5,0) và (5,5,0).
- `Scale 2` quanh pivot: hai đầu về (-5,0,0) và (15,0,0).
- Không selection → lỗi; `Move 1,2` và `Scale 0` → lỗi, không đổi gì.
- Parser: 11 trường hợp trong unit test của module.

## Chưa làm

- Numeric edit cho **subobject** (chọn mặt/điểm) — hiện chỉ object.
- Gumball **relocate** (đặt lại gốc) và **alignment** theo trục object — `OM9-L2-03-03-005`, vòng sau.
