# 3DM import contract (G00-04)

## 1. Biên giới

`om9-exchange` đọc `.3dm` bằng openNURBS pin `eb92af3ba1806b0a34a99aba0d3bda83e3d46083`, chuyển sang
`om9-core::geometry` (`NurbsCurve`, `NurbsSurface`, `Mesh`, `Cloud`, `Curve`), và **không** giữ object
openNURBS nào sau khi import xong.

## 2. Round-trip đã đo

Fixture version 5 (`archive_version 50`, 9 528 B) chứa Point, Line, RationalArc (degree 2, rational,
3 CV, domain [2,4], midpoint chính xác), Mesh (một quad), Cloud (2 điểm + màu). Đọc lại cho JSON **giống
hệt** — tên, trọng số hữu tỉ, domain, topology mesh, màu cloud đều giữ.

## 3. Partial success

Mỗi object hỏng/không hỗ trợ được ghi vào báo cáo import: `{object_index, name, reason}`; import vẫn
thành công cho phần còn lại; exit code khác 0 chỉ khi **không** đọc được archive.

## 4. Phạm vi MVP

Trong: point, curve (line/arc/NURBS hữu tỉ), mesh, cloud, layer/name/attribute cơ bản, đơn vị mm/inch.
Ngoài MVP: block/nested instance transform, trim đầy đủ sang OCCT, material/texture, preview.
