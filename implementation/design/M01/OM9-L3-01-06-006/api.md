# Grid and clipping (OM9-L3-01-06-006)

`grid_lines()` dựng lưới 21x21 trên mặt phẳng z=0; `RenderStyle.grid = true` vẽ bằng pipeline line.
Clipping dùng `Camera.near/far` trong ma trận chiếu.

Test: `the_ground_grid_is_drawn_and_clipped_by_the_far_plane` — lưới vẽ được vào frame rỗng (> 200 px);
sphere (khoảng cách 9..19) với `far = 5` bị cắt hoàn toàn (< 200 px).
