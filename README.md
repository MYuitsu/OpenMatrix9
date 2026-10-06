# OpenMatrix9

An experimental jewelry CAD workbench for FreeCAD. Rust owns command catalog,
state and behavior; C++/Qt provides native integration. Unsupported commands
remain visible and disabled.

This project was developed with Matrix9 as a functional reference. Compatibility
identifiers and familiar color roles remain. This public snapshot excludes
commercial artwork, extracted resource archives, manuals, decompiled source and
private design samples. It does not claim clean-room development, affiliation
with the original vendors, or legal clearance of the implementation.

All **510 icon bindings** use independently authored SVGs. The exporter accepts
authored catalogs only and cannot restore legacy bitmaps.

## Tiến độ lệnh / Command progress

Cập nhật: **2026-10-06**. Bảng này mô tả phạm vi đang có trong source public.
**510 icon bindings không tương đương 510 lệnh đã hoàn thành.** Lệnh chưa có
implementation vẫn hiển thị nhưng bị vô hiệu hóa. Có tên trong command completion
hoặc có phím tắt không chứng minh lệnh đã chạy được.

| Trạng thái | Ý nghĩa |
|---|---|
| ✅ Đã triển khai trong phạm vi ghi rõ | Có đường thực thi native/Rust và kiểm tra tương ứng; không cam kết toàn bộ tùy chọn của Matrix9/Rhino. |
| 🟡 Một phần | Có chức năng nền tảng nhưng còn giới hạn hoặc chưa tích hợp phần phát triển mới. |
| ⬜ Chưa hoàn thành | Chưa có đường thực thi được xác nhận; icon/catalog chưa được tính là implementation. |

### Các lệnh đã có đường thực thi

| Nhóm | Lệnh / chức năng | Phạm vi và kiểm tra |
|---|---|---|
| ✅ File / project | New, Open, Save, Save As, Undo, Redo | Dùng lệnh native FreeCAD; [workspace lifecycle](tests/core_workspace_lifecycle_smoke.FCMacro). |
| ✅ Selection | Select All, Select None, Delete | Chọn/xóa đối tượng của project hiện hành; Undo qua host; [workspace](tests/workspace_smoke.FCMacro). |
| ✅ Curve | `Line`, `Polyline` | Chuột và command console, tọa độ/option trong phạm vi session; [curve](tests/curve_smoke.FCMacro), [polyline console](tests/polyline_console_smoke.FCMacro). Các lệnh curve khác chưa được suy ra là hoàn thành. |
| ✅ View | Top, Front, Right, Left, Rear, Bottom, Isometric | View native FreeCAD; [workspace](tests/workspace_smoke.FCMacro). |
| ✅ Viewport | `RestoreViewports`, `SynchronizeViews`, `CenterViewport`, `ShowGrid`, `ViewportTabs` | Workspace bốn view, camera, grid và tabs; [views](tests/core_views_smoke.FCMacro), [tabs](tests/core_view_tabs_smoke.FCMacro). |
| ✅ Zoom | `Zoom_Window`, `Zoom_Dynamic`, `Zoom_Extents`, `Zoom_Selected`, Fit All | Zoom theo viewport/bounds/selection; [view controls](tests/core_view_controls_smoke.FCMacro), [zoom bounds](tests/core_zoom_bounds_smoke.FCMacro). |
| ✅ View utilities | `Crosshairs`, `ViewCaptureToFile`, `PictureFrame` | Cursor overlay, capture ảnh và đặt ảnh tham chiếu; [capture](tests/core_capture_smoke.FCMacro), [picture frame](tests/core_picture_frame_smoke.FCMacro). |
| ✅ Measure | `Distance`, `Angle` | Đo khoảng cách hai điểm và góc giữa hai hướng; [distance](tests/core_distance_smoke.FCMacro), [angle](tests/core_angle_smoke.FCMacro). |
| ✅ Snap | `Osnap E`, `Osnap M`, `Osnap P`, master snap, `Ortho` | End/Mid/Point và trạng thái Ortho trong công cụ được hỗ trợ; [snap tools](tests/core_snap_point_tools_smoke.FCMacro), [keyboard](tests/core_keyboard_smoke.FCMacro). Chưa đủ mọi loại O-Snap của Rhino. |
| ✅ Notes | Notes, Project Notes | Ghi chú lưu cùng project; [notes](tests/core_notes_smoke.FCMacro). |
| ✅ Interface | `CommandHistory`, `Properties`, `F6`, command console/completion | History, inspector native, context menu và tra tên lệnh; [keyboard](tests/core_keyboard_smoke.FCMacro), [completion](tests/command_completion_smoke.FCMacro). Chỉ thực thi lệnh có host implementation. |
| 🟡 3DM | `Import3dm`, `Export3dm`, File > Open/Import/Export `.3dm` | Geometry exchange và source preservation nền tảng; xem bảng openNURBS bên dưới và [3DM host checks](tests/three_dm_smoke.FCMacro). Export geometry-only không giữ toàn bộ dữ liệu nguồn. |

### Các nhóm chưa hoàn thành

| Nhóm | Phần còn thiếu |
|---|---|
| ⬜ Curve mở rộng | Các lệnh ngoài Line/Polyline: circle, ellipse, arc, interpolate, rebuild, blend, offset, trim/split và công cụ edit curve đầy đủ. |
| ⬜ Surface / Solid | Tạo và sửa hình học theo catalog: loft, sweep, network, fillet, Boolean và các biến thể; có khả năng tương ứng trong FreeCAD không có nghĩa OM9 đã tích hợp. |
| ⬜ Transform / Edit | Duplicate, Move, Rotate, Mirror, Array, Join, Explode, Split, Trim theo luồng command OM9 đầy đủ. Khả năng thao tác đối tượng của FreeCAD không được tính là hoàn thành các lệnh này. |
| ⬜ Builders | Ring Rail và các builder trang sức/parametric history. |
| ⬜ Gems / Cutters | Gem Loader, placement/prong/channel/pavé và tạo cutters theo catalog. |
| ⬜ Render / Materials | Workflow render, material/texture và môi trường đầy đủ. |
| ⬜ SubD / Clayoo / Emboss | Công cụ tạo/sửa SubD và emboss theo catalog. |
| 🟡 Settings / Utilities | Có thiết lập giao diện, keyboard/snap/view nền tảng; chưa hoàn thành toàn bộ catalog. `PointsOn`, `Group`, `UnGroup`, `gvCenterObjects` có tên/phím tắt nhưng chưa được xác nhận hoàn thành. |

Không công bố phần trăm hoàn thành tổng thể: số icon, tên lệnh và số test không
phải cùng một đơn vị đo. Danh mục giao diện: [MainMenu.ini](Resources/menu/MainMenu.ini);
đường dispatch: [Command.cpp](Gui/Command.cpp) và [Rust catalog](rust/src/ffi.rs).

## Tiến độ tương thích openNURBS / Rhino 3DM

**Chưa full openNURBS và chưa tương đương Rhino SDK.** Thư viện được tải từ
upstream public tại commit cố định trong [OpenNURBS.cmake](cmake/OpenNURBS.cmake).
Đọc được, giữ nguyên dữ liệu nguồn và chỉnh sửa được là các mức hỗ trợ khác nhau.

| Dữ liệu / chức năng | Source public hiện tại | Phần còn thiếu |
|---|---|---|
| ✅ Points và curves | Point, line/polyline, arc/circle và rational NURBS qua native CAD conversion. | Không suy ra mọi lớp curve hoặc mọi trường hợp suy biến đều đã tương thích. |
| ✅ Surfaces / BRep | Conversion surface/BRep được hỗ trợ, trim loops, holes, shells, solids và extrusion. | Kiểm chứng toàn bộ biến thể topology/dung sai và lớp native còn lại. |
| ✅ Mesh | Triangle/quad mesh; geometry export có cơ chế giữ quad nguồn khi mesh không đổi. | Full mesh attributes và mọi dữ liệu đi kèm chưa được cam kết. |
| 🟡 Block / `ON_InstanceRef` | Geometry-only import mở rộng embedded blocks thành CAD/mesh, áp dụng transform lồng nhau, scale không đều và reflection. | Source public chưa có structural block preservation writer; geometry-only export không giữ graph definition/reference. |
| ✅ Metadata hình học cơ bản | Name, nested layer path, color, visibility, lock và chuyển đơn vị trong phạm vi geometry exchange. | Full layer/material/document dependency remapping. |
| 🟡 Preserve source data | Snapshot nguồn bất biến, hash, namespace/UUID và file đính kèm trong FCStd; giữ record chưa chỉnh sửa được. | Giữ trong snapshot không đồng nghĩa render/edit/export lại đầy đủ. [Preservation checks](tests/three_dm_preservation_smoke.FCMacro). |
| 🟡 Annotation / lights / native classes khác | Có thể giữ dữ liệu trong snapshot và record retained; geometry-only không hỗ trợ mọi lớp. | Editable/display conversion cho dimensions, leaders, hatches, point clouds, clipping objects và lớp chưa có converter. |
| ⬜ Materials / textures / render settings | Snapshot có thể giữ nguồn; geometry-only export không bảo toàn đầy đủ. | Dependency closure, tài nguyên và appearance roundtrip. |
| ⬜ History / plugin userdata | Chưa có reconstruction của Rhino history hoặc Matrix builders. | Unknown reference-bearing/plugin payload cần chính sách bảo toàn và test; không chạy plugin Rhino. |
| 🟡 Rhino 5 export | Writer geometry nhắm archive version5; có native/host test. | Chưa kiểm thử trực tiếp bằng ứng dụng Rhino5; không tuyên bố hỗ trợ đầy đủ các phiên bản mới hơn. |

### Bản phát triển đang chờ tích hợp vào public

Các phần sau đã có kết quả test ở workspace phát triển riêng nhưng **chưa có
trong source public này**, nên clone repo hiện tại chưa nhận các chức năng đó:

| Phần phát triển | Kết quả đã kiểm tra | Còn trước khi hoàn thành |
|---|---|---|
| Selected preservation writer | Giữ native UUID và dependency closure; selected block giữ definition/member/reference thay vì flatten. | Review, tích hợp và nối standard/menu/CMD export. |
| Geometry/metadata overlays | Thay BRep/mesh được hỗ trợ; name Unicode, color, visibility, lock. | Layer/dependency/member edits và CAD mới chưa có source. |
| Copy/delete và FCStd | UUID mới cho copy, geometry/placement độc lập, không hồi sinh đối tượng bị bỏ khỏi selection. | Tài liệu cũ thiếu verified baseline và merge từ archive khác nhau. |
| Structural / affine blocks | Shared/nested definitions, rigid instances, preview scale/reflection; export rigid delta ghép với ma trận native. | Mixed mesh/shear host acceptance và chỉnh member definition đầy đủ. |

Kết quả phát triển đã ghi nhận: **8 bộ native**, **23 kiểm tra structural block**,
**14 signature**, **12 preserved export**, **19 preservation regression** pass.
Đây là bằng chứng từ workspace phát triển, không phải kết quả tái chạy trên
public snapshot. Hai mẫu nhẫn riêng đã dùng để test geometry; preservation export
của chúng còn bị chặn bởi plugin payload chưa hiểu. Mẫu riêng không được đưa vào repo.

Public snapshot có bằng chứng audit giao diện **3093/3093 checks** và **68 Rust
tests** trong [progress ledger](docs/openmatrix9-progress.json) và
[publication checks](docs/public-source-checks.json); audit icon không chứng minh
hoàn thành toàn bộ chức năng CAD. Hướng kiểm tra và yêu cầu fixture:
[build and validation](docs/build.md).

See [build and checks](docs/build.md), [publication audit](docs/public-source-review.md)
and [third-party notices](THIRD_PARTY_NOTICES.md).

The project follows its existing **LGPL-2.1-or-later** declaration; the license
text is in [LICENSE](LICENSE). Separate dependency notices remain applicable.
