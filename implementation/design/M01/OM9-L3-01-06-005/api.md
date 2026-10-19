# Shaded pass (OM9-L3-01-06-005)

Không upload normal: fragment shader tính pháp tuyến từ `dpdx/dpdy` của vị trí world, chiếu sáng Lambert
với hướng sáng cố định, nhân với `RenderStyle.tint`.

Test: `shading_varies_across_a_curved_face` — độ lệch chuẩn kênh đỏ trên silhouette > 5 (mặt cong được tô
biến thiên, không phẳng một màu).
