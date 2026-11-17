# Match Surface

Sửa mặt để cạnh của nó khớp cạnh mặt đích như Rhino 5 MatchSrf (Position / Tangency / Curvature), giữ đầu kia.

- **Mã lệnh:** `SurfaceSurfaceEditToolsMatch`
- **Vị trí:** menu Surface, ô số 28
- **Icon:** `Resources/icons/surface-minimal/SurfaceSurfaceEditToolsMatch.svg`
- **Gọi bằng dòng lệnh:** `SurfaceSurfaceEditToolsMatch`, `MatchSrf`
- **Cách chạy:** từng bước trên dòng lệnh; Enter để tạo, Esc để huỷ

## Cách dùng

1. Bấm gần cạnh mặt cần sửa.
2. Bấm gần cạnh mặt đích.
3. Chọn Continuity khi xem trước, Enter.

## Tuỳ chọn (lệnh phụ)

- Continuity: Position / Tangency / Curvature.

| Dòng lệnh | Ý nghĩa | Mặc định | Khoảng |
|---|---|---|---|
| `Average=` | Hai mặt cùng đổi, gặp ở giữa (MatchSrf) | No |  |
| `Continuity=` | Liên tục | tangency | position, tangency, curvature |
