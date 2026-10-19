# Hợp đồng số học — gate M00-L3-02-03

Pin: OCCT **8.0.1** (`native/occt/pin.json`). Harness: `native/occt-rust-proof`.

## 1. Đại lượng và oracle

| Nhóm | Oracle | Dung sai |
| --- | --- | --- |
| Box 10×10×4 | `V = 400 mm³`, `A = 360 mm²` | 1e-9 |
| Sphere r=5 | `V = 4/3π·125`, `A = 4π·25` | 1e-7 |
| Cylinder r=2, h=4 | `V = π·4·4`, `A = 24π` | 1e-9 / 1e-7 |
| Fillet r=1 trên cạnh dài 4 | `V = 400 − (1 − π/4)·4` | 1e-6 |
| Offset vào 1 | `V = 8·8·2` | 1e-6 |
| Tessellation | node phải nằm trên mặt cầu: `| ‖p‖ − r | ≤ deflection` | theo deflection |

## 2. Hợp đồng suy biến

| Ca | Kỳ vọng |
| --- | --- |
| Cut với box chỉ **tiếp xúc** | vẫn valid, volume không đổi (400) |
| Offset **xuyên thành** | `valid = false` (không tạo solid) |
| Box ở **giới hạn kernel** (1e-12) | lỗi có kiểu: `Standard_DomainError` |

## 3. Tessellation: hai thuộc tính tách rời

- **closed**: không có cạnh biên (`boundary_edges == 0`).
- **manifold**: không có cạnh dùng > 2 tam giác (`non_manifold_edges == 0`).

Đo được: `boundary_edges = 0`, `non_manifold_edges = 2` (một ở mỗi cực cầu) ở cả deflection 0,01 và 0,001.
Đây là **thuộc tính của mesher OCCT**, được ghi lại chứ không "làm tròn" thành watertight.

## 4. Bài test

`native/occt-rust-proof/tests/numeric.rs`: 4 test (primitive, fillet/offset, degenerate, mesh closed + refine).
