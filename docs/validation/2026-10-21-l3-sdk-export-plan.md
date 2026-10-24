# L3 SDK export plan index

Planning only: the milestone L3 records decide which surface belongs in SDK v1.
This index is generated from those records; edit the L3 JSON, not this file.

| status | count | meaning |
|---|---|---|
| `exported_in_sdk_v1` | 30 | Delivered, with a measured receipt. |
| `planned` | 36 | Needed by plugins, not delivered yet. |
| `not_in_sdk` | 10 | Host-internal codec/state/fixtures. |

Total annotated L3: **76** across 8 owners.

## L2-01-02 - Be Command Dispatch Undo Redo

| L3 | title | status | SDK surface / note |
|---|---|---|---|
| `OM9-L3-01-02-001` | Command IDs And Alias Registry | `planned` | đề xuất: `commands()`, `command_descriptor(id)` — Registry do host sở hữu; SDK đọc danh sách. |
| `OM9-L3-01-02-002` | Typed Command Descriptor | `planned` | đề xuất: `command_descriptor(id)` — Kiểu tham số do host khai báo; SDK đọc descriptor. |
| `OM9-L3-01-02-003` | Command Context And Active Document | `not_in_sdk` | Context/active document là trạng thái host. |
| `OM9-L3-01-02-004` | Interactive Command State Machine | `not_in_sdk` | State machine tương tác là host UI. |
| `OM9-L3-01-02-005` | Command Option Parser | `planned` | đề xuất: `parse_command_arguments(id, tokens)` — Parser ở host để khớp CLI; SDK gọi và nhận lỗi typed. |
| `OM9-L3-01-02-006` | Atomic Command Transactions | `planned` | đề xuất: `transaction_info()` — Ranh giới transaction do host giữ; SDK chỉ mô tả. |
| `OM9-L3-01-02-007` | Undo Redo Journal | `planned` | đề xuất: `undo_journal(limit)` — Journal ở host; SDK đọc các bước gần nhất. |
| `OM9-L3-01-02-008` | Command Cancel Rollback | `not_in_sdk` | Cancel rollback là chính sách host. |
| `OM9-L3-01-02-009` | Command Repeat And Dispatch | `planned` | đề xuất: `run_command(id, arguments)` — Dispatch ở host; SDK gọi command đã đăng ký. |
| `OM9-L3-01-02-010` | Command Event And Observer API | `exported_in_sdk_v1` | `Host::progress`, `Host::warning`, `MessageLevel`, `PluginMessage` |
| `OM9-L3-01-02-011` | Command Availability Preconditions | `planned` | đề xuất: `command_available(id)` — Precondition do host đánh giá; SDK đọc boolean. |

## L2-01-03 - Be Document Scene Entities

| L3 | title | status | SDK surface / note |
|---|---|---|---|
| `OM9-L3-01-03-001` | Document Entity IDs | `planned` | đề xuất: `object_id_of(id)`, `is_object_id(text)` — Cấp phát id ở host; SDK chỉ parse/đọc dạng chuỗi. |
| `OM9-L3-01-03-002` | Typed Scene Entity Records | `exported_in_sdk_v1` | `Host::object_attributes`, `Host::object_info`, `Host::object_placement`, `Host::query_objects`, `Host::rename_object`, `Host::set_object_attributes`, `Host::set_object_placement` |
| `OM9-L3-01-03-003` | Geometry Handle And Attributes | `exported_in_sdk_v1` | `Host::add_mesh`, `Host::object_attributes`, `Host::query_objects`, `Host::rename_object`, `Host::set_object_attributes` |
| `OM9-L3-01-03-004` | Layer Domain Records | `exported_in_sdk_v1` | `Host::add_object`, `Host::assign_objects_to_layer`, `Host::clear_current_layer`, `Host::layer_create`, `Host::layer_info`, `Host::layer_order`, `Host::layer_update`, `Host::layers`, `Host::set_current_layer` |
| `OM9-L3-01-03-005` | Entity To Layer Membership | `exported_in_sdk_v1` | `Host::add_object`, `Host::assign_objects_to_layer`, `Host::layer_create`, `Host::layer_update`, `Host::layers`, `Host::set_current_layer` |
| `OM9-L3-01-03-006` | Layer Selection Visibility Rules | `exported_in_sdk_v1` | `Host::is_object_pickable` |
| `OM9-L3-01-03-007` | Group Membership Model | `exported_in_sdk_v1` | `Host::create_group`, `Host::delete_group`, `Host::groups` |
| `OM9-L3-01-03-008` | Block Definition And Instance Graph | `exported_in_sdk_v1` | `Host::block_definitions` |
| `OM9-L3-01-03-009` | Jewelry Semantic Extension Record | `exported_in_sdk_v1` | `Host::clear_object_extension`, `Host::object_extensions`, `Host::set_object_extension` |
| `OM9-L3-01-03-010` | Selection State Document Contract | `planned` | đề xuất: `selection_revision()` — Đồng bộ selection do host giữ; SDK đọc revision. |
| `OM9-L3-01-03-011` | Document Change Notification | `exported_in_sdk_v1` | `Host::last_change` |
| `OM9-L3-01-03-012` | Document Snapshot And Restore | `planned` | đề xuất: `document_snapshot()`, `document_restore(snapshot)` — Snapshot format ở host; SDK nhận/trả handle mờ, không tự serialize. |
| `OM9-L3-01-03-013` | Import Source Provenance Metadata | `exported_in_sdk_v1` | `Host::object_provenance` |

## L2-01-07 - Febe Picking Selection Generic Handles

| L3 | title | status | SDK surface / note |
|---|---|---|---|
| `OM9-L3-01-07-001` | Screen Ray And Hit Test Query | `planned` | đề xuất: `pick_point(x_mm, y_mm)`, `pick_at_ray(origin, direction)` — Hit test/ray là host; SDK chỉ trả object/subobject id + khoảng cách. |
| `OM9-L3-01-07-002` | Object Selection Set Core | `exported_in_sdk_v1` | `Host::add_to_selection`, `Host::remove_from_selection`, `Host::selected_objects`, `Host::set_selection`, `Host::toggle_selection` |
| `OM9-L3-01-07-003` | Subobject Face Edge Vertex Picking | `planned` | đề xuất: `pick_subobject(x_mm, y_mm, kind)` — Topology OCCT ở host; SDK trả (object_id, subobject_kind, index). |
| `OM9-L3-01-07-004` | Selection Window And Crossing | `planned` | đề xuất: `select_window(min, max, crossing)` — Window UI ở host; SDK nhận vùng và trả id đã chọn. |
| `OM9-L3-01-07-005` | Object Selection Disambiguation | `planned` | đề xuất: `pick_nearest(x_mm, y_mm)` — Heuristic ưu tiên ở host; SDK trả một kết quả đã phân xử. |
| `OM9-L3-01-07-006` | Selection Highlighting And Hover | `planned` | đề xuất: `set_highlight(ids)`, `clear_highlight()` — Vẽ highlight ở host; SDK chỉ đặt/đọc tập id. |
| `OM9-L3-01-07-007` | Generic Translation Rotation Scale Handles | `planned` | đề xuất: `gumball_drag_start(axis)`, `gumball_drag_commit()` — Handle/vẽ ở host; SDK chỉ commit kéo thành mutation. |
| `OM9-L3-01-07-008` | Drag Transaction And Commit Cancel | `planned` | đề xuất: `begin_drag(ids)`, `end_drag(cancel)` — Transaction kéo do host giữ; SDK đóng/mở ranh giới. |

## L2-01-08 - Be Units Tolerances Basic Parameters

| L3 | title | status | SDK surface / note |
|---|---|---|---|
| `OM9-L3-01-08-001` | Millimeter World Unit Model | `exported_in_sdk_v1` | `Host::document_units`, `Host::parse_angle`, `Host::parse_coordinates`, `Host::parse_length` |
| `OM9-L3-01-08-002` | Angle Conversion And Degrees | `exported_in_sdk_v1` | `Host::document_units`, `Host::parse_angle`, `Host::parse_coordinates`, `Host::parse_length` |
| `OM9-L3-01-08-003` | Document Absolute Relative Tolerances | `exported_in_sdk_v1` | `Host::document_units`, `Host::parse_angle`, `Host::parse_coordinates`, `Host::parse_length`, `Host::set_document_tolerance` |
| `OM9-L3-01-08-004` | Typed Numeric Parameter Values | `exported_in_sdk_v1` | `Host::apply_parameters`, `Host::validate_parameter` |
| `OM9-L3-01-08-005` | Builder Parameter Range Validation | `exported_in_sdk_v1` | `Host::apply_parameters`, `Host::validate_parameter` |
| `OM9-L3-01-08-006` | Unit Safe Parameter Serialization | `exported_in_sdk_v1` | `Host::apply_parameters`, `Host::validate_parameter` |
| `OM9-L3-01-08-007` | Coordinate Numeric Input Primitives | `exported_in_sdk_v1` | `Host::document_units`, `Host::parse_angle`, `Host::parse_coordinates`, `Host::parse_length` |

## L2-01-12 - Be Native Project Codec Stl Export

| L3 | title | status | SDK surface / note |
|---|---|---|---|
| `OM9-L3-01-12-001` | Native Project Container Format | `not_in_sdk` | Container là host codec, plugin không ghi trực tiếp. |
| `OM9-L3-01-12-002` | Scene And Layer State Encoding | `not_in_sdk` | Encoding scene/layer thuộc host. |
| `OM9-L3-01-12-003` | B-Rep Mesh And Instance Encoding | `not_in_sdk` | Encoding brep/mesh/instance thuộc host. |
| `OM9-L3-01-12-004` | Builder History Stub Persistence | `not_in_sdk` | Builder history stub thuộc host. |
| `OM9-L3-01-12-005` | Atomic Save And Write Failure Recovery | `exported_in_sdk_v1` | `Host::project_state`, `Host::save_project`, `Host::save_project_as` |
| `OM9-L3-01-12-006` | Project Load Validation And Compatibility | `not_in_sdk` | Load validation thuộc host. |
| `OM9-L3-01-12-007` | Document Reload And Restoration | `not_in_sdk` | Reload/restore là flow host (open/replace). |
| `OM9-L3-01-12-008` | STL Tessellation Export Codec | `planned` | đề xuất: `export_stl(path, options)` — Tessellation OCCT ở host; SDK chỉ tham số + đường dẫn host cấp. |
| `OM9-L3-01-12-009` | Export Mesh Units And Winding | `planned` | đề xuất: `export_options { unit, winding }` — Quy đổi đơn vị/winding thực thi ở host; SDK khai báo lựa chọn. |
| `OM9-L3-01-12-010` | Project Roundtrip Regression Fixtures | `not_in_sdk` | Fixture roundtrip là kiểm thử nội bộ. |

## L2-02-01 - Be Rhino Nurbs Object Semantics

| L3 | title | status | SDK surface / note |
|---|---|---|---|
| `OM9-L3-02-01-001` | Point And Polyline Semantic Types | `planned` | đề xuất: `geometry_semantics(id)` — Semantic point/polyline đã có một phần; mở rộng ở host. |
| `OM9-L3-02-01-002` | NURBS Curve Control Points Knots Weights | `exported_in_sdk_v1` | `Host::curve_definition`, `Host::curve_info`, `Host::curve_semantics`, `Host::evaluate_curve` |
| `OM9-L3-02-01-003` | Open Closed And Planar Curve Rules | `exported_in_sdk_v1` | `Host::curve_definition`, `Host::curve_semantics` |
| `OM9-L3-02-01-004` | Surface UV Domain And Seam Rules | `planned` | đề xuất: `surface_domain(id)` — UV/seam do kernel host; SDK đọc domain. |
| `OM9-L3-02-01-005` | Trimmed Vs Underlying Surface Model | `planned` | đề xuất: `trimmed_surface(id)` — Trim ở host; SDK đọc loại đối tượng. |
| `OM9-L3-02-01-006` | Surface Edge And Isocurve Semantics | `planned` | đề xuất: `surface_edges(id)`, `surface_isocurves(id)` — Topology ở host; SDK đọc danh sách. |
| `OM9-L3-02-01-007` | Polysurface And Closed Solid Semantics | `planned` | đề xuất: `polysurface_kind(id)` — Kiểm tra kín ở host; SDK đọc kết luận. |
| `OM9-L3-02-01-008` | Extrusion Vs Full B-Rep Representations | `planned` | đề xuất: `representation_kind(id)` — Phân loại ở host; SDK đọc loại. |
| `OM9-L3-02-01-009` | Mesh Vs NURBS Object Type Rules | `planned` | đề xuất: `mesh_vs_nurbs_rule(id)` — Rule ở host; SDK đọc kết quả. |
| `OM9-L3-02-01-010` | Instance And Block Semantic Type | `planned` | đề xuất: `instance_semantics(id)` — Instance graph ở host; SDK đọc loại/member. |

## L2-02-04 - 04

| L3 | title | status | SDK surface / note |
|---|---|---|---|
| `OM9-L3-02-04-001` | Line Polyline Point Creation | `planned` | đề xuất: `add_point/line/polyline (đã có)` — Tạo hình học ở host; SDK đã có API tạo, cần entry L3 riêng. |
| `OM9-L3-02-04-002` | Circle Arc Ellipse Polygon Creation | `exported_in_sdk_v1` | `Host::add_arc`, `Host::add_circle_on_frame`, `Host::add_ellipse`, `Host::add_interpolated_curve`, `Host::add_polygon`, `Host::add_rectangle`, `KnotMode` |
| `OM9-L3-02-04-003` | Interpolated And Controlpoint NURBS Curve | `exported_in_sdk_v1` | `Host::add_arc`, `Host::add_interpolated_curve`, `Host::add_nurbs_curve`, `KnotMode` |
| `OM9-L3-02-04-004` | Curve Edit Points And Grips | `planned` | đề xuất: `move_edit_point(id, index, point)` — Grip/edit state ở host; SDK đổi điểm. |
| `OM9-L3-02-04-005` | Join Explode Close And Split Curves | `exported_in_sdk_v1` | `Host::curve_segments`, `Host::evaluate_curve_segment` |
| `OM9-L3-02-04-006` | Offset Fillet Chamfer Curves | `planned` | đề xuất: `offset_curve(id, distance)`, `fillet_curves(a, b, radius)` — Thuật toán kernel ở host; SDK tham số + kết quả. |
| `OM9-L3-02-04-007` | Blend Arcblend And Match Curve Continuity | `planned` | đề xuất: `blend_curves(a, b, continuity)` — Continuity ở host; SDK chọn mức. |
| `OM9-L3-02-04-008` | Rebuild Refit And Knot Operations | `planned` | đề xuất: `rebuild_curve(id, options)` — Rebuild ở host; SDK tham số. |
| `OM9-L3-02-04-009` | Project Pullback And Curve Intersections | `planned` | đề xuất: `project_curve(id, target)` — Chiếu ở host; SDK nhận curve mới. |
| `OM9-L3-02-04-010` | Duplicate Edge Border Isocurve | `planned` | đề xuất: `duplicate_edge_curve(id, edge_index)` — Topology ở host; SDK chọn cạnh/viền. |
| `OM9-L3-02-04-011` | Curve Seams Directions And Divisions | `planned` | đề xuất: `curve_seams(id)`, `curve_divisions(id, count)` — Seam/division ở host; SDK đọc/chia. |
| `OM9-L3-02-04-012` | Sketch On Surface Mesh And Two Views | `planned` | đề xuất: `sketch_on_surface(curve, surface)` — Sketch engine ở host; SDK gọi và nhận kết quả. |

## L2-03-20 - 12-febe-matrix

| L3 | title | status | SDK surface / note |
|---|---|---|---|
| `OM9-L3-03-20-001` | Distance And Length Measurement | `exported_in_sdk_v1` | `Host::measure_angle`, `Host::measure_curve_length`, `Host::measure_distance`, `Host::measure_mesh_area` |
| `OM9-L3-03-20-002` | Angle And Radius Measurement | `exported_in_sdk_v1` | `Host::measure_angle`, `Host::measure_curve_radius`, `Host::measure_distance` |
| `OM9-L3-03-20-003` | Bounding Size And Center Measurement | `exported_in_sdk_v1` | `Host::object_bounds` |
| `OM9-L3-03-20-004` | Jewelry Object Count And Type Summary | `exported_in_sdk_v1` | `Host::object_summary` |
| `OM9-L3-03-20-005` | Measurement Label And Command Feedback | `exported_in_sdk_v1` | `Host::report_measurement` |
