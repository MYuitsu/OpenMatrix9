# GPU device and render loop (OM9-L3-01-06-001)

`om9_render::Renderer::new()` tạo instance/adapter/device/queue (headless, chưa có swapchain);
`Renderer::render(mesh, camera, width, height)` dựng colour + depth target, upload mesh, vẽ và đọc pixel về.

- Depth: `Depth32Float`, `depth_compare = Less`, `depth_write_enabled`.
- `Frame::painted_pixels()` và `Frame::pixel(x,y)` để test.
- Không có adapter → `None`, caller báo thiếu GPU thay vì giả vờ có viewport.

Test: `crates/om9-render/tests/render_loop.rs::a_device_exists_and_draws_the_kernel_mesh`,
`the_depth_buffer_keeps_the_nearer_triangle`.
