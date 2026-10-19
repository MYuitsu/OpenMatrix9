# OCCT upgrade gate — bài test quyết định "còn cần patch hay không"

Ngày: 2026-10-17 · Trạng thái: **đã chạy, đã phân biệt được hai bản**

## 1. Vì sao có file này

Patch `TKG3d` (sửa D2/D3 của offset curve) chỉ là **backstop tạm thời**. Mỗi lần nâng OCCT, ta phải trả lời được câu hỏi:
*"bản mới đã sửa lỗi chưa, có bỏ được patch không?"* — bằng **một bài test**, không bằng phỏng đoán.

Bài test: [`native/occt_conformance.cpp`](../../../native/occt_conformance.cpp) · công cụ: [`implementation/tools/occt_upgrade_gate.py`](../../tools/occt_upgrade_gate.py).

## 2. Bài test kiểm gì (oracle độc lập, không dùng đạo hàm của OCCT)

| Check | Oracle | Vì sao chọn ca này |
| --- | --- | --- |
| `line_offset_d2_zero`, `line_offset_d3_zero` | giải tích: offset của đường thẳng là đường thẳng → `D2 = D3 = 0` | bắt lỗi thô |
| `circle_offset_d2_exact`, `circle_offset_d3_exact` | giải tích: đường tròn bán kính `r + d` → `D2 = -(r+d)(cos,sin)`, `D3 = (r+d)(sin,-cos)` | **ca này PASS cả trên bản lỗi** — chứng minh vì sao test đơn giản không phát hiện được (`R` hằng ⇒ `Dr = D2r = 0`) |
| `ellipse_offset_d2_vs_fdiff` | sai phân 5 điểm của `EvalD0` + Richardson | ellipse có `R` biến thiên ⇒ lỗi lộ ra |
| `ellipse_offset_d3_vs_fdiff` | như trên (đạo hàm bậc 3) | |
| `spline_offset_d2/d3_vs_fdiff` | B-spline degree 3, knot không đều (`0, 0.15, 0.55, 0.85, 1`) | đúng chế độ NURBS thực tế |
| `box_volume_400`, `cut_volume_analytic` | giải tích `10×10×4` và `400 − π·2²·4` | kiểm hình học solid |
| `box_valid`, `cut_valid`, `cut_faces_7` | `BRepCheck_Analyzer`, đếm face | |
| `mesh_triangles_positive`, `mesh_indices_consistent` | `BRepMesh_IncrementalMesh` + kiểm chỉ số tam giác | |

**Điểm cốt lõi:** oracle cho D2/D3 là **sai phân của điểm** (`EvalD0`) — điểm vẫn đúng trên bản lỗi, nên nếu `D2/D3` sai thì hiệu số lộ ra ngay. Richardson khử số hạng `O(h²)` để phân biệt **lỗi kernel** với **sai số rời rạc**.

## 3. Kết quả đo (2026-10-17, OCCT 8.0.1)

| Check | Stock 8.0.1 (nguyên bản) | Patched TKG3d |
| --- | --- | --- |
| `line_offset_d2/d3_zero` | PASS (0) | PASS (0) |
| `circle_offset_d2/d3_exact` | PASS (0) | PASS (0) |
| `ellipse_offset_d2_vs_fdiff` | **FAIL 4.166e-01** | PASS 4.5e-09 |
| `ellipse_offset_d3_vs_fdiff` | **FAIL 9.996e-01** | PASS 3.4e-08 |
| `spline_offset_d2_vs_fdiff` | **FAIL 7.180e-01** | PASS 9.6e-09 |
| `spline_offset_d3_vs_fdiff` | **FAIL 1.001e+00** | PASS 1.6e-05 |
| solid + mesh (7 check) | PASS | PASS |
| **verdict** | `PATCH_REQUIRED` (exit 2) | `PATCH_NOT_NEEDED` (exit 0) |

Sai số tương đối **41 % (D2) và ~100 % (D3)** trên bản nguyên bản — không phải nhiễu số, mà là công thức sai.

Bằng chứng: `implementation/evidence/M00/OM9-L2-02/OCCT-CONFORMANCE-8.0.1-{stock,patched}.json`.

## 4. Quy trình khi nâng OCCT (bắt buộc)

```powershell
# 1. nạp SDK mới theo pin (verify SHA-256 trước)
python implementation/tools/fetch_occt.py --fetch

# 2. chạy gate: build probe + so stock với bản vá (control)
python implementation/tools/occt_upgrade_gate.py --sdk build/standalone-deps/occt-<ver>-sdk `
    --patched-dll <TKG3d đã vá> --write
```

| Kết quả | Hành động |
| --- | --- |
| `PATCH_REQUIRED` (exit 2) | Giữ patch (hoặc chuyển offset derivative sang `om9-core`) |
| `PATCH_NOT_NEEDED` (exit 0) | **Bỏ patch**, xoá khỏi `pin.json`, cập nhật notices |
| Control fail (exit 3) | Patch không còn tác dụng → điều tra, không tin gate |

## 5. Về hiệu năng (đính chính)

Patch **không làm giảm hiệu năng**: cùng thuật toán, chỉ sửa hệ số (bớt một số hạng, thêm một phép nhân `1/R²`) — chênh lệch dưới mức đo được. Cái patch gây tốn là **bảo trì** (mỗi bản OCCT phải vá lại) và **nghĩa vụ công bố source của phần đã sửa** (LGPL). Vì vậy gate này tồn tại: bỏ patch đúng lúc **upstream sửa xong**, không phải vì tốc độ.

## 6. Trạng thái upstream (kiểm 2026-10-17)

- **D2**: master đã sửa (`Add(Ndir·(3Dr²/R⁴ − D2r/R²))`) → phần D2 của patch là backport.
- **D3**: master vẫn còn công thức cũ → khi master phát hành bản sửa D3, chạy gate là biết ngay.
