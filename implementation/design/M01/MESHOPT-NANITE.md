# meshoptimizer và Nanite — so sánh, phần đã làm, phần còn lại

Ngày: 2026-10-18. Bối cảnh: Matrix9 yêu cầu VGA lớn, nên hướng đúng là **cluster LOD + GPU culling**, không
phải cố giảm draw call bằng mẹo CPU.

## 1. meshoptimizer (Arseny Kapoulkine) — gồm những gì

| Thành phần | Việc nó làm | Trạng thái trong OM9 |
| --- | --- | --- |
| `meshopt_optimizeVertexCache` | sắp lại index để tái dùng vertex trong cache 32 entry (Forsyth 2001) | **đã làm** — `om9_render::meshlet::optimize_vertex_cache` |
| `meshopt_optimizeVertexFetch` | sắp lại vertex theo thứ tự dùng | chưa (dễ, làm sau) |
| `meshopt_optimizeOverdraw` | sắp triangle theo depth để giảm overdraw | chưa |
| `meshopt_buildMeshlets` | chia cluster ≤ 64 vertex / ≤ 124 triangle + bounding sphere | **đã làm** — `build_meshlets` |
| `meshopt_simplify` | QEM + khoá biên + tuỳ chọn attribute | QEM của OM9 **đã có** phần lõi; thiếu attribute và khoá biên mềm |
| `meshopt_simplifySloppy` | giảm rất mạnh, không giữ topology | chưa |
| meshlet culling (`meshopt` không làm) | cần compute shader + indirect draw | **chưa** — đây là phần Nanite |

## 2. Nanite (Epic, UE5 2021) — kiến trúc

1. **Cluster**: mỗi cluster ≤ 128 triangle / ≤ 64 vertex; cluster có bounding sphere.
2. **LOD hierarchy**: cluster được nhóm thành nhóm có LOD riêng; mỗi nhóm có 2–3 mức.
3. **GPU culling**: compute shader kiểm tra frustum + occlusion + **screen-space error** cho từng cluster, ghi
   danh sách cluster sống vào buffer.
4. **Indirect draw**: một `multi_draw_indexed_indirect` vẽ tất cả cluster sống; không có draw call CPU.
5. **Virtual geometry**: dữ liệu mesh nằm ở đĩa/streaming, nạp theo cluster.

Yêu cầu GPU: compute shader đầy đủ, indirect draw, bộ nhớ VRAM lớn (vài GB cho cảnh 100M+ triangle). Điều này
khớp với việc Matrix9 yêu cầu VGA lớn.

## 3. Phần đã làm và đo được

| Việc | Kết quả đo |
| --- | --- |
| Vertex-cache optimization trên nhẫn (1536 tri) | **cache miss ratio 1,021 → 0,708 (−31 %)** |
| Meshlet splitting trên nhẫn | **38 cluster**, mỗi cluster ≤ 64 vertex, giữ đúng tổng diện tích |
| Test | 4 test: mọi triangle vào đúng 1 cluster; index hợp lệ; diện tích cluster = diện tích mesh; cache ratio không tăng |

Số 1,021 nghĩa là trước khi tối ưu, trung bình mỗi tam giác nạp hơn 3 vertex mới; sau tối ưu còn 0,708 —
đúng mức meshoptimizer báo cho lưới đều.

## 4. Phần còn lại (theo thứ tự tác động)

| # | Việc | Vì sao |
| --- | --- | --- |
| 1 | **Giải mâu thuẫn đo scene 24k mesh (42 ms vs 8,4 ms)** | trước khi thêm bất kỳ thuật toán nào, phải chắc phép đo đúng |
| 2 | **GPU cluster culling** (compute + indirect) | biến 4 draw thành 1 indirect, cull theo cluster + screen-space error |
| 3 | **LOD hierarchy theo cluster** | Nanite thật: mỗi nhóm cluster có LOD riêng, không phải LOD toàn mesh |
| 4 | **OptimizeOverdraw** | giảm fill cho cảnh chồng nhau |
| 5 | **Streaming cluster** | file 2 GB không cần nạp hết vào VRAM |

## 5. Ghi chú trung thực

- Phần "meshoptimizer" ở đây là **tự hiện thực lại ý tưởng** (cache ordering, meshlet split), không phải
  vendoring thư viện C++ của Kapoulkine. Nếu muốn đúng hành vi công nghiệp, có thể vendor `meshoptimizer`
  (MIT) làm native crate — đó là việc riêng.
- Nanite chưa được hiện thực; mới có **đơn vị dữ liệu** cho nó (meshlet + bounding sphere).
- Con số 42 ms của QEM ở scene 24k vẫn chưa giải thích; tôi không dùng nó để kết luận thuật toán nào thắng.
