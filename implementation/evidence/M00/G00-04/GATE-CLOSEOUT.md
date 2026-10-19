# G00-04 — 3DM / openNURBS closeout

Sinh bởi `implementation/tools/close_out_m00_gates.py` lúc 2026-10-17.

| L4 | Trạng thái | Căn cứ |
| --- | --- | --- |
| L4-04-01-01 | EVIDENCED | `QymCAD/crates/qymcad-io/src/formats.rs:21-33` — `Format` có 10 biến thể (Step, Iges, Stl, Obj, Ply, Gltf, ThreeMf, Amf, Dxf, Svg), **không có 3DM** |
| L4-04-01-02 | EVIDENCED | openNURBS pin `eb92af3b…` build được; `LICENSE` + notice có; probe chạy |
| L4-04-01-03 | EVIDENCED | so sánh: openNURBS là decoder khả dụng duy nhất (OCCT không có 3DM; QymCAD không có) → chọn openNURBS |
| L4-04-01-04 | EVIDENCED | biên giới import: `om9-exchange` đọc 3DM → `Geometry` của `om9-core` (api.md) |
| L4-04-02-01 | EVIDENCED | fixture version 5 ghi bằng pin: 9528 B, 5 loại object |
| L4-04-02-02 | EVIDENCED | đọc lại **độc lập** cho JSON giống hệt bản ghi (point, line, rational arc, mesh, cloud) |
| L4-04-02-03 | OPEN | chuyển NURBS/trim 3DM → `TopoDS` OCCT chưa làm; cần M03 |
| L4-04-02-04 | OPEN | transform/block inheritance/nested instances chưa test |
| L4-04-02-05 | OPEN | inch→mm, tolerance, UV trim chưa test |
| L4-04-03-01 | EVIDENCED | file 3DM rác bị từ chối (exit 1, thông điệp, không crash) |
| L4-04-03-02 | EVIDENCED | thiết kế báo cáo partial success (api.md §3) |
| L4-04-03-03 | EVIDENCED | phạm vi 3DM cho MVP (api.md §4) |
