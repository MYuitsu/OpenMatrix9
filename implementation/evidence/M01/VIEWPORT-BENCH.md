# Viewport batch benchmark — đo thật

Ngày: 2026-10-18. Máy: RTX 3060 (Vulkan), MSVC 14.44, build **release**.
Lệnh: `cargo test --release -p om9-render --test scene_bench -- --nocapture`
Target: 960×540, depth `Depth32Float`, mỗi object = cube 8 đỉnh/12 tam giác, một arena chung.

| Object | Hiển thị | Bị cull | Draw call | ms/khung (CPU+submit, có poll) | FPS tương đương |
| --- | --- | --- | --- | --- | --- |
| 200 | 200 | 0 | 200 | 2,37 * | 421 |
| 500 | 500 | 0 | 500 | 0,39 | 2551 |
| 1000 | 972 | 28 | 972 | 0,60 | 1673 |
| 2000 | 1673 | 327 | 1673 | **1,02** | **981** |

\* ca 200 gồm cả lần tạo pipeline/arena đầu tiên; các ca sau dùng lại đường đã ấm nên thấp hơn.
Con số là **cpu+submit** (đo tới khi `device.poll(Wait)` trả về), tức gồm cả thời gian GPU của khung.

## Kết luận

- 2000 object: **~1 ms/khung** ở 960×540, dư ngân sách 16,6 ms rất nhiều.
- Frustum culling cắt **327/2000** object khi camera ở [40,-40,40] nhìn gốc.
- Mỗi object một draw call vẫn ổn; multi-draw chỉ cần khi vượt ~5000.
- Ảnh stress thật trong app: `implementation/evidence/M01/VIEWPORT-BENCH-2000.png` (2000 box, release).

## Heavy mesh (sphere 5008 tam giác/object, deflection 0.002)

| Object | Tam giác trong arena | Hiển thị | Bị cull | ms/khung |
| --- | --- | --- | --- | --- |
| 50 | 250 400 | 33 | 17 | 0,19 |
| 100 | 500 800 | 58 | 42 | 0,22 |
| 200 | **1 001 600** | 71 | 129 | 0,31 |
| 400 | **2 003 200** | 86 | 314 | 0,32 |

Lệnh: `$env:OM9_BENCH_DEFLECTION='0.002'; cargo test --release -p om9-render --test scene_bench bench_heavy -- --nocapture`.
Lưu ý: culling cắt phần lớn object nên số tam giác **được vẽ** nhỏ hơn nhiều; đo thông lượng thô cần cờ tắt culling.

## Partial vertex update (sculpt)

`SceneBatch::update_vertices(object, first_vertex, &positions)` ghi đúng dải đỉnh vào arena — nền cho
nét cọ sculpt, không upload lại cả mesh.

## Điều chưa đo

- Cảnh 4 viewport cùng lúc (ước lượng tuyến tính: 4 × 1,02 ≈ 4,1 ms — vẫn dưới 16,6).
- Mesh nặng (sphere 5008 tam giác thay vì 12) — cần đo riêng vì băng thông vertex tăng ~400×.
- Overdraw với vật liệu trong suốt (chưa có).
