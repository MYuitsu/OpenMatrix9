# Danh mục owner và tham số đối chiếu từ mã

Ngày chụp nguồn: 2026-10-26. Đây là khai báo trong mã, không phải chứng nhận runtime. Đọc `SPEC.vi.md` trước khi sửa. Các điều kiện stage được giữ nguyên trong `source-inventory.json`; phải đọc lại parser của owner trước mỗi gói việc.

- 37 provider đăng ký trong adapter + Line + Polyline = **39 nhóm**.
- Controller có **57 field `_pending`**; **19 field** ngoài nhóm option đăng ký cần audit riêng. Số field không phải số lệnh hay số tính năng.

## Các nhóm đã có adapter

| Owner | File / hàm | Khai báo option trong các stage | ID lấy từ module |
|---|---|---|---|
| `solid` | [solid.rs](../../../../crates/om9-app/src/session/solid.rs) / `solid_option_tokens` | `Point`, `Undo`, `Cancel`, `Center`, `2Point`, `3Point`, `4Point`, `Vertical`, `AroundCurve`, `FitPoints`, `Tangent`, `Solution=1`, `Solution=2`, `Radius=`, `Diameter=`, `Diagonal`, `Cube`, `Length=`, `Width=`, `Height=` | Tra dispatcher; không suy ID từ tên owner |
| `circle` | [circle.rs](../../../../crates/om9-app/src/session/circle.rs) / `circle_option_tokens` | `Center`, `2Point`, `3Point`, `Vertical`, `FitPoints`, `AroundCurve`, `Tangent`, `Deformable`, `Approx`, `Degree=`, `PointCount=`, `Radius=`, `Point=`, `Solution=`, `OnCurve=`, `Size=Diameter`, `Size=Circumference`, `Size=Area`, `Diameter=`, `Circumference=`, `Area=`, `Undo`, `Cancel` | `OM9-CURVE-005` |
| `rect` | [rect.rs](../../../../crates/om9-app/src/session/rect.rs) / `rect_option_tokens` | `Square`, `Length=`, `Cancel`, `Center`, `3Point`, `Vertical`, `Undo` | `OM9-CURVE-004` |
| `interp` | [interp.rs](../../../../crates/om9-app/src/session/interp.rs) / `interp_option_tokens` | `Degree=`, `Close`, `Undo`, `Cancel` | `OM9-CURVE-003` |
| `ellipse` | [ellipse.rs](../../../../crates/om9-app/src/session/ellipse.rs) / `ellipse_option_tokens` | `Center`, `Corner`, `Diameter`, `Vertical`, `FromFoci`, `AroundCurve`, `OnCurve=`, `MarkFoci`, `Deformable`, `Approx`, `Degree=`, `PointCount=`, `RadiusX=`, `RadiusY=`, `Undo`, `Cancel` | `OM9-CURVE-006` |
| `arc` | [arc.rs](../../../../crates/om9-app/src/session/arc.rs) / `arc_option_tokens` | `3Point`, `Center`, `StartPoint`, `Radius=`, `Diameter=`, `Undo`, `Cancel` | `OM9-CURVE-007`, `OM9-CURVE-008` |
| `rebuild` | [rebuild.rs](../../../../crates/om9-app/src/session/rebuild.rs) / `rebuild_option_tokens` | `Degree=`, `Points=`, `Cancel` | `OM9-CURVE-009` |
| `fit` | [fit.rs](../../../../crates/om9-app/src/session/fit.rs) / `fit_option_tokens` | `Tolerance=`, `Cancel` | `OM9-CURVE-010` |
| `offset_curve` | [offset.rs](../../../../crates/om9-app/src/session/offset.rs) / `offset_curve_option_tokens` | `Distance=`, `Cancel` | `OM9-CURVE-018` |
| `project` | [project.rs](../../../../crates/om9-app/src/session/project.rs) / `project_option_tokens` | `Direction=`, `Cancel` | `OM9-CURVE-016` |
| `blend` | [blend.rs](../../../../crates/om9-app/src/session/blend.rs) / `blend_option_tokens` | `Scale1=`, `Scale2=`, `Cancel` | `OM9-CURVE-011`, `OM9-CURVE-012`, `OM9-CURVE-052` |
| `isocurve` | [isocurve.rs](../../../../crates/om9-app/src/session/isocurve.rs) / `isocurve_option_tokens` | `U=`, `V=`, `Cancel` | `OM9-CURVE-019` |
| `pull` | [pull.rs](../../../../crates/om9-app/src/session/pull.rs) / `pull_option_tokens` | `Tolerance=`, `Cancel` | `OM9-CURVE-015` |
| `intersect` | [intersect.rs](../../../../crates/om9-app/src/session/intersect.rs) / `intersect_option_tokens` | `Tolerance=`, `Cancel` | `OM9-CURVE-017` |
| `crv2view` | [crv2view.rs](../../../../crates/om9-app/src/session/crv2view.rs) / `crv2view_option_tokens` | `Cancel` | `OM9-CURVE-013` |
| `through_points` | [through_points.rs](../../../../crates/om9-app/src/session/through_points.rs) / `through_points_option_tokens` | `Degree=`, `Closed`, `Open`, `Cancel` | `OM9-CURVE-026` |
| `polygon` | [polygon.rs](../../../../crates/om9-app/src/session/polygon.rs) / `polygon_option_tokens` | `Sides=`, `Circumscribed`, `Inscribed`, `Undo`, `Cancel` | `OM9-CURVE-028` |
| `helix` | [helix.rs](../../../../crates/om9-app/src/session/helix.rs) / `helix_option_tokens` | `Radius=`, `Height=`, `Turns=`, `Axis=`, `Undo`, `Cancel` | `OM9-CURVE-030` |
| `spiral` | [spiral.rs](../../../../crates/om9-app/src/session/spiral.rs) / `spiral_option_tokens` | `StartRadius=`, `EndRadius=`, `Turns=`, `Height=`, `Axis=`, `Undo`, `Cancel` | `OM9-CURVE-029` |
| `fillet_corners` | [fillet_corners.rs](../../../../crates/om9-app/src/session/fillet_corners.rs) / `fillet_corners_option_tokens` | `Radius=`, `Cancel` | `OM9-CURVE-023` |
| `divide` | [divide.rs](../../../../crates/om9-app/src/session/divide.rs) / `divide_option_tokens` | `Count=`, `Cancel` | `OM9-CURVE-036` |
| `to_lines` | [to_lines.rs](../../../../crates/om9-app/src/session/to_lines.rs) / `to_lines_option_tokens` | `Count=`, `Cancel` | `OM9-CURVE-050` |
| `crv_seam` | [crv_seam.rs](../../../../crates/om9-app/src/session/crv_seam.rs) / `crv_seam_option_tokens` | `Seam=`, `Cancel` | `OM9-CURVE-031` |
| `control_curve` | [control_curve.rs](../../../../crates/om9-app/src/session/control_curve.rs) / `control_curve_option_tokens` | `Degree=`, `Undo`, `Cancel` | `OM9-CURVE-042` |
| `continue_curve` | [continue_curve.rs](../../../../crates/om9-app/src/session/continue_curve.rs) / `continue_curve_option_tokens` | `Degree=`, `Undo`, `Cancel` | `OM9-CURVE-041` |
| `offset_on_surface` | [offset_on_surface.rs](../../../../crates/om9-app/src/session/offset_on_surface.rs) / `offset_on_surface_option_tokens` | `Distance=`, `Cancel` | `OM9-CURVE-020` |
| `offset_normal` | [offset_normal.rs](../../../../crates/om9-app/src/session/offset_normal.rs) / `offset_normal_option_tokens` | `Distance=`, `Cancel` | `OM9-CURVE-057` |
| `corner` | [corner_ops.rs](../../../../crates/om9-app/src/session/corner_ops.rs) / `corner_option_tokens` | `Radius=`, `Cancel`, `Distance=` | `OM9-CURVE-021`, `OM9-CURVE-022` |
| `cross_section` | [cross_section.rs](../../../../crates/om9-app/src/session/cross_section.rs) / `cross_section_option_tokens` | `Count=`, `Spacing=`, `Cancel` | `OM9-CURVE-043` |
| `curve_soft_edit` | [curve_soft_edit.rs](../../../../crates/om9-app/src/session/curve_soft_edit.rs) / `curve_soft_edit_option_tokens` | `Index=`, `Point=`, `Cancel` | `OM9-CURVE-055` |
| `angle_isocurve` | [angle_isocurve.rs](../../../../crates/om9-app/src/session/angle_isocurve.rs) / `angle_isocurve_option_tokens` | `Angle=`, `Direction=`, `Cancel` | `OM9-CURVE-044` |
| `extrude_surface` | [extrude_surface.rs](../../../../crates/om9-app/src/session/extrude_surface.rs) / `extrude_surface_option_tokens` | `Height=`, `Cancel` | `OM9-SURFACE-002` |
| `heightfield` | [heightfield.rs](../../../../crates/om9-app/src/session/heightfield.rs) / `heightfield_option_tokens` | `File=`, `Height=`, `Size=`, `Cancel` | `OM9-SURFACE-031` |
| `extend_curve` | [extend_curve.rs](../../../../crates/om9-app/src/session/extend_curve.rs) / `extend_curve_option_tokens` | `Length=`, `End`, `Start`, `Cancel` | `OM9-CURVE-027` |
| `tween_curves` | [tween_curves.rs](../../../../crates/om9-app/src/session/tween_curves.rs) / `tween_curves_option_tokens` | `Count=`, `Cancel` | `OM9-CURVE-038` |
| `continue_interp` | [continue_interp.rs](../../../../crates/om9-app/src/session/continue_interp.rs) / `continue_interp_option_tokens` | `Degree=`, `Undo`, `Cancel` | `OM9-CURVE-035` |
| `arc_blend` | [arc_blend.rs](../../../../crates/om9-app/src/session/arc_blend.rs) / `arc_blend_option_tokens` | `Cancel` | `OM9-CURVE-056` |
| `line` | [session.rs](../../../../crates/om9-app/src/session.rs) / `CommandSession + command_options adapter` | `BothSides`, `Length` | `OM9-CURVE-002` |
| `polyline` | [polyline.rs](../../../../crates/om9-app/src/session/polyline.rs) / `PolylineOptionInterface` | `PersistentClose`, `Mode`, `Helpers`, `Length`, `Close`, `Direction`, `Center`, `Undo` | `OM9-CURVE-001` |

## Các pending owner phải audit bổ sung

Các field dưới đây chưa có trong registry 39 nhóm. Không được tự gắn option của Line/Polyline vào chúng. Phân loại `OPTIONS`, `POSITIONAL_INPUT`, `IMMEDIATE_ACTION`, hoặc `UNSUPPORTED` theo parser và milestone thật.

| Field | Kiểu hiện tại | Trạng thái gói đặc tả |
|---|---|---|
| `top11_pending` | `self::top11::Top11Pending` | AUDIT_REQUIRED |
| `surface_pending` | `SurfacePending` | AUDIT_REQUIRED |
| `curve_pair_pending` | `CurvePairPending` | AUDIT_REQUIRED |
| `surface_pair_pending` | `SurfacePairPending` | AUDIT_REQUIRED |
| `seam_pending` | `SeamPending` | AUDIT_REQUIRED |
| `soft_edit_pending` | `SoftEditPending` | AUDIT_REQUIRED |
| `tween_pending` | `TweenPending` | AUDIT_REQUIRED |
| `extend_pending` | `ExtendPending` | AUDIT_REQUIRED |
| `split_pending` | `SplitPending` | AUDIT_REQUIRED |
| `tangent_pending` | `TangentPending` | AUDIT_REQUIRED |
| `extrude_all_pending` | `ExtrudeAllPending` | AUDIT_REQUIRED |
| `variable_offset_pending` | `VarOffsetPending` | AUDIT_REQUIRED |
| `shrink_pending` | `ShrinkPending` | AUDIT_REQUIRED |
| `variable_blend_pending` | `VarBlendPending` | AUDIT_REQUIRED |
| `chamfer_pending` | `ChamferPending` | AUDIT_REQUIRED |
| `fillet_pending` | `FilletPending` | AUDIT_REQUIRED |
| `variable_chamfer_pending` | `VarChamferPending` | AUDIT_REQUIRED |
| `history_options_pending` | `HistoryOptionsPending` | AUDIT_REQUIRED |
| `slash_pending` | `SlashPending` | AUDIT_REQUIRED |

## Tính toàn vẹn

`source-inventory.json` chứa SHA256 của các file đọc, token payload gốc, vị trí hàm và nguyên văn khai báo availability. Khi nguồn đổi, kiểm lại inventory; không tự ghi `validated` cho implementation.
