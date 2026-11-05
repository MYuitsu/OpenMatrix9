# Single Line

Vẽ một đoạn thẳng, kèm 9 kiểu dựng như Matrix: hai phía, vuông góc mặt, theo góc, thẳng đứng, 4 điểm, phân giác, vuông góc / tiếp tuyến đường cong, nối dài.

- **Mã lệnh:** `CurveLineSingleLine`
- **Vị trí:** menu Curve, ô số 2
- **Icon:** `Resources/icons/curve-minimal/CurveLineSingleLine.svg`
- **Gọi bằng dòng lệnh:** `CurveLineSingleLine`, `Line`
- **Cách chạy:** từng bước trên dòng lệnh; Enter để tạo, Esc để huỷ

## Cách dùng

1. Bấm điểm đầu (hoặc chọn một kiểu dựng trên dòng lệnh).
2. Bấm điểm cuối, hoặc gõ chiều dài rồi bấm hướng.

## Tuỳ chọn (lệnh phụ)

- BothSides: điểm đầu là trung điểm.
- Normal: chọn mặt / khối / lưới, bấm điểm trên mặt; đường theo pháp tuyến (IgnoreTrims).
- Angled: đường gốc 2 điểm, gõ góc xoay, bấm điểm cuối.
- Vertical: đường vuông góc mặt phẳng làm việc.
- FourPoint: 2 điểm cho hướng, 2 điểm cho đầu và cuối.
- Bisector: đỉnh góc, cạnh đầu, cạnh cuối, rồi điểm cuối.
- Perpendicular: vuông góc từ đường cong; Point: từ một điểm tới đường cong; 2Curves: vuông góc chung hai đường cong.
- Tangent: tiếp tuyến tại điểm trên đường cong; Point: từ một điểm tiếp xúc đường cong; 2Curves: tiếp tuyến chung hai đường cong.
- Extension: nối dài đầu gần nhất của đường cong đã chọn.
