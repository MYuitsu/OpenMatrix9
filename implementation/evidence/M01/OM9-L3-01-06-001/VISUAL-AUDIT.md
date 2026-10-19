# Xác minh thị giác — ứng dụng thật, 2026-10-18

Ảnh `visual-audit.png` là screenshot do chính app chụp trong chế độ `--ui-audit` (1280×800), với
`OM9_GPU_SPHERE=1` và build `--features gpu-sphere`.

- Adapter trong receipt: **NVIDIA GeForce RTX 3060**, Vulkan.
- Viewport **Perspective**: sphere OCCT (tessellation 0.05) hiện rõ, có sáng tối, cùng lưới 21×21 nền.
- Ba viewport còn lại là **orthographic** (`LookingDown`, `ThroughFinger`, `SideView`) nên `scene_camera`
  trả `None` — đúng thiết kế: pass GPU chỉ chạy cho camera phối cảnh.
- Đường 2D của document (Line) và lưới CPlane vẫn vẽ đè lên ảnh GPU.

Đây là bằng chứng cho: khung hình GPU có thật (001), một renderer dùng chung (002), sphere đọc tròn
(005), lưới và clipping theo `near/far` (006).
