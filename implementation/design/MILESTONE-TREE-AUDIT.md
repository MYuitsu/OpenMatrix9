# Audit cây kế hoạch (M / L1 / L2 / L3 / L4)

Sinh bởi `implementation/tools/audit_milestone_tree.py` — 11 milestone, 102 L2, 791 L3, 2611 L4.

## 1. Tổng theo milestone

| Milestone | L2 | L3 | L4 | Khai báo (L1.json) | Khớp? | depends_on |
| --- | ---: | ---: | ---: | --- | --- | --- |
| M00 | 7 | 19 | 73 | module=7, l3=19, l4=73 | có | — |
| M01 | 13 | 117 | 516 | module=13, l3=117, l4=516 | có | ['M00'] |
| M02 | 9 | 59 | 306 | module=9, l3=59, l4=306 | có | ['M00', 'M01'] |
| M03 | 14 | 243 | 898 | module=14, l3=243, l4=854 | có | ['M01'] |
| M04 | 7 | 51 | 340 | module=7, l3=51, l4=340 | có | ['M01'] |
| M05 | 5 | 43 | 126 | module=5, l3=43, l4=126 | có | ['M03'] |
| M06 | 2 | 19 | 76 | module=2, l3=19, l4=76 | có | ['M03', 'M05'] |
| M07 | 3 | 19 | 69 | module=3, l3=19, l4=65 | có | ['M03'] |
| M08 | 8 | 51 | 177 | module=8, l3=51, l4=221 | có | ['M03', 'M05', 'M06'] |
| M09 | 0 | 0 | 30 | module=0, l3=0, l4=30 | có | ['M00', 'M01', 'M02', 'M03', 'M04', 'M05', 'M06', 'M07', 'M08'] |
| AFTER | 34 | 170 | 0 | module=34, l3=170, l4=0 | có | ['M09'] |

## 2. Phát hiện

- Không có lệch cấu trúc nào.

## 3. Phủ năng lực trang sức (Matrix-class)

| Năng lực | Số L3 khớp | Milestone | Ví dụ |
| --- | ---: | --- | --- |
| Giao diện shell/docking/viewport | 30 | AFTER, M01, M02, M03, M04, M05, M08 | OM9-L3-01-01-002 Workbench Docking Regions; OM9-L3-01-01-003 Reorder Collapse And Float Panels |
| Lệnh + option + undo/redo | 31 | AFTER, M01, M02, M03, M04, M08 | OM9-L3-01-02-001 Command IDs And Alias Registry; OM9-L3-01-02-002 Typed Command Descriptor |
| Tài liệu + entity/layer | 28 | AFTER, M01, M03, M05 | OM9-L3-01-01-006 Active Document Window State; OM9-L3-01-02-003 Command Context And Active Document |
| Đơn vị + dung sai | 19 | AFTER, M00, M01, M02, M03, M04, M05, M08 | OM9-M00-L3-02-03 Geometry numeric invariants; OM9-M00-L3-04-02 NURBS / trimmed BRep / units proof |
| OCCT B-Rep kernel bridge | 19 | AFTER, M00, M01, M03, M04, M05 | OM9-M00-L3-04-02 NURBS / trimmed BRep / units proof; OM9-M00-L3-07-02 Linux optional & packaged OCCT |
| GPU/wgpu hiển thị | 16 | AFTER, M00, M01, M02 | OM9-M00-L3-03-02 GPU renderer stability and performance; OM9-L3-01-06-001 GPU Device And Render Loop |
| Đường cong NURBS (Interp/Circle/Arc/Ellipse) | 136 | AFTER, M00, M01, M02, M03, M05, M06, M08 | OM9-M00-L3-01-03 REUSE ADAPT ADD architecture decision; OM9-M00-L3-03-01 Source UI architecture and constraints |
| Bề mặt (Loft/Sweep/Trim/Rebuild) | 102 | AFTER, M00, M01, M02, M03, M04, M05, M06, M08 | OM9-M00-L3-04-02 NURBS / trimmed BRep / units proof; OM9-L3-01-01-007 Global Status And Error Surface |
| Khối solid + boolean/fillet/shell | 49 | AFTER, M01, M03, M07 | OM9-L3-01-06-005 Shaded Face And Solid Pass; OM9-L3-02-01-007 Polysurface And Closed Solid Semantics |
| Chọn + gumball + transform | 74 | AFTER, M00, M01, M02, M03, M04, M08 | OM9-M00-L3-03-03 Picking + transactional selection; OM9-L3-01-03-006 Layer Selection Visibility Rules |
| CPlane + toạ độ | 7 | M01, M02 | OM9-L3-01-08-007 Coordinate Numeric Input Primitives; OM9-L3-03-10-004 Numeric And Coordinate Feedback |
| Object snap + smarttrack | 14 | M01, M02, M04 | OM9-L3-01-03-012 Document Snapshot And Restore; OM9-L3-03-15-007 Master Jobbag Document Snapshot |
| Thư viện đá (gem) | 46 | AFTER, M02, M05, M06, M07 | OM9-L3-03-05-002 Gem View Material Color Cues; OM9-L3-03-09-004 F6 Gem And Report Modes |
| Đặt đá: prong/head/bezel/pavé/channel/azure | 36 | AFTER, M02, M05, M06, M07 | OM9-L3-03-05-006 Display Settings Preview Presets; OM9-L3-04-11-003 Baguette Channel Layout |
| Ring builder + profile/rail | 46 | AFTER, M01, M03, M04, M05, M06, M07, M08 | OM9-L3-03-01-004 F4 World Origin And Ring Center Display; OM9-L3-01-09-003 Revolve Profile And Rail Revolve |
| Cutters (M07) | 14 | AFTER, M02, M07 | OM9-L3-03-05-003 Curve And Cutter Visibility Presets; OM9-L3-04-16-001 Gem Cutter Azure For Single Stone |
| Lịch sử/parametric rebuild | 30 | AFTER, M01, M03, M04, M05, M06, M07, M08 | OM9-L3-01-12-004 Builder History Stub Persistence; OM9-L3-03-07-005 Recent Command Icon History |
| Tính khối lượng kim loại | 10 | AFTER, M01, M04, M05, M06, M07 | OM9-L3-02-01-002 NURBS Curve Control Points Knots Weights; OM9-L3-04-19-001 Metal Volume From Closed B-Rep |
| 3DM import/export (openNURBS) | 32 | AFTER, M00, M01, M02, M03, M08 | OM9-M00-L3-04-01 3DM capability gap and dependency candidate; OM9-M00-L3-04-03 Import security and transparent limitations |
| STL/in 3D | 28 | AFTER, M01, M03, M05 | OM9-L3-01-12-003 B-Rep Mesh And Instance Encoding; OM9-L3-01-12-008 STL Tessellation Export Codec |
| Layer clipboard/exchange | 1 | AFTER | OM9-L3-06-02-004 Exchange Format Compatibility Report |
| Đo lường (angle/distance) | 10 | AFTER, M01, M02, M03 | OM9-L3-01-08-002 Angle Conversion And Degrees; OM9-L3-03-06-004 Known Distance Image Calibration |
| Cage/SubD | 15 | AFTER, M03, M08 | OM9-L3-04-01-020 Transform · Cage Edit — menu binding; OM9-L3-04-01-021 Transform · ReleaseFromCage — menu binding |
| Kiểm thử/nghiệm thu | 9 | AFTER, M01, M02, M04 | OM9-L3-01-05-002 B-Rep Shape Type And Validation; OM9-L3-01-06-008 Viewport Invalidation And Redraw |

