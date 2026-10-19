# Shared scene graphics resources (OM9-L3-01-06-002)

`MeshData { positions, indices }` là dữ liệu trung lập giữa kernel và GPU: `om9-kernel` sản xuất
(`sphere_triangles`), `om9-render` upload. Một `Renderer` dùng chung cho mọi viewport; mỗi lần vẽ tạo
buffer từ `MeshData` (chưa cache — đó là L3-003).

Test: `a_two_times_dpi_target_keeps_the_framing`, `a_camera_move_changes_what_is_drawn`.
