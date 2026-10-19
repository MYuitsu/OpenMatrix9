# Selection highlight (OM9-L3-01-06-007)

`RenderStyle::highlighted()` đổi tint sang xanh lá (0.2, 0.9, 0.35); cùng một mesh, cùng camera, cùng
pipeline — chỉ uniform tint đổi, nên highlight không cần pass riêng.

Test: `the_highlight_tints_the_selected_object` — frame thường có 0 pixel "xanh trội"; frame highlight
có > 5 000 pixel xanh.
