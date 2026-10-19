# Wireframe pass (OM9-L3-01-06-004)

`MeshData::edge_lines()` trả **cạnh duy nhất** của mesh dưới dạng line list; `RenderStyle.wireframe = true`
vẽ chúng bằng pipeline `LineList` (depth compare `LessEqual`, không ghi depth) đè lên mặt đã tô.

Test: `the_wireframe_pass_draws_edges` — frame có wireframe nhiều pixel hơn frame chỉ tô, chênh > 500 px.
