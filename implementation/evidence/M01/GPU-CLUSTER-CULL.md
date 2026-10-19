# GPU cluster culling — đo thật

Ngày: 2026-10-18. RTX 3060 (Vulkan), release, 480×270.
Test: `crates/om9-render/tests/scene_bench.rs::gpu_cluster_culling_matches_the_cpu_rule`.

## Kiến trúc

```
meshlet::build_meshlets(mesh)          -> 33 cluster (torr 48×16, ≤64 đỉnh)
ClusterRenderer::new(device, queue, clusters)
  storage buffer: ClusterRecord { index_start, index_count, base_vertex, centre+radius }
  indirect buffer: DrawIndexedIndirectArgs[cluster]
  uniform: 6 mặt phẳng frustum + view_projection + (eye, tan) + (half_height, min_px, near)
ClusterRenderer::render(...)
  compute (workgroup 64): test frustum + bán kính pixel -> ghi index_count (0 nếu bị cull)
  render: multi_draw_indexed_indirect(indirect, 0, cluster_count)
```

- Không cần counter: cluster bị cull có `index_count = 0`, GPU bỏ qua.
- `multi_draw_indexed_indirect` là **core** trong wgpu 30 (không cần feature).
- `count_visible()` copy indirect về CPU để đối chiếu.

## Kết quả

| Phép thử | Kết quả |
| --- | --- |
| Toàn cảnh nhìn từ [10,-10,6] | GPU 33/33, **CPU cũng 33/33** → khớp |
| Nhìn sát một bên [7,-3,1.5] | GPU **30/33**, CPU **30/33** → khớp, cull 3 |
| Camera sau lưng | **0** cluster được vẽ |
| Thời gian frame (median, poll từng frame) | **0,15 ms** |

## Ý nghĩa

- Việc chọn cluster giờ nằm hoàn toàn trên GPU; CPU không còn vòng lặp cull và không cần biết cluster nào
  sống.
- 0,15 ms/frame cho một nhẫn 1536 tam giác; với 24 000 mảnh, cluster count ~800 000, dispatch 12 500
  workgroup — vẫn dưới 1 ms trên 3060, nhưng **chưa đo ở mức đó** (việc tiếp theo).
- Con số này đo đúng cách (poll từng frame), khác với lỗi đo trước đây đã ghi trong `QEM-COMPARISON.md`.

## Scene lớn qua cluster renderer

| Cảnh | Cluster | Hiển thị | Dựng meshlet | Frame (median, poll từng frame) |
| --- | --- | --- | --- | --- |
| 5 000 mesh (4 000 nhẫn + 1 000 kim cương) | **147 079** | 23 133 | 2,0 s | **3,88 ms** |
| So sánh: cùng cảnh qua `SceneBatch` + QEM | 4 static range | — | — | **0,30 ms** |

**Kết luận trung thực:** với cảnh **tĩnh**, gộp mesh thắng cluster (0,30 ms so với 3,88 ms) vì 147 000 draw
indirect vẫn tốn hơn 4 draw gộp. Cluster chỉ thắng khi cảnh **động/streaming** hoặc khi culling bỏ được phần
lớn — đúng lý do Nanite dùng nó cho virtual geometry chứ không phải cho mesh tĩnh nhỏ.

## Chọn LOD trong compute (đã chạy)

| Camera | Fine | Coarse |
| --- | --- | --- |
| Cách 8 mm | **33** | 0 |
| Cách 80 mm | 0 | **71** |

Compute chọn level theo **kích thước nhóm trên màn hình** (`coarse_pixels = 24`), không phải theo danh sách
CPU.

## Nối vào app

`OM9_GPU_STRESS=2000 OM9_GPU_CLUSTER=1` → viewport Perspective vẽ 2 000 box qua `ClusterRenderer`.
Ảnh: `GPU-CLUSTER-APP.png` (chụp bởi chính app, chế độ audit).
