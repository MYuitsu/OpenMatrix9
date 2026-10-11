# OM9-INFO-021 — Gumball Alignment

Ngày 2026-10-11; `partially_implemented` / `validated_supported_slice`.
Command `GumballAlignment`, `OM9_GumballAlignment`; legacy
`OM9_InfoSettingsGumballAlignment`. [Spec](../../OpenMatrix9_Codex_Spec_v1/specs/01-core/om9-info-021-gumball-alignment.md).

Nguồn: Matrix 8 Book 1 PDF 45–47, trang in 35–37; continuation hết trước
History PDF 48. CPlane/Object/World đổi hệ trục widget, không xoay geometry.
Menu native và typed CMD cùng Rust policy; default OM9 CPlane đã được duyệt.

CPlane theo native viewport frame; World theo XYZ. Object dùng native Placement
của một whole owner hoặc curve tangent / surface tangent-normal native.
Wire/Shell/Compound đúng một child được unwrap, bounded 64 levels. Multiple
owners không có frame duy nhất báo unresolved và giữ alignment trước.
Mesh/Points whole-owner dùng Placement; Mesh component frame, light/camera
và mọi frame không có native adapter vẫn unsupported, không ngầm chọn World.
Selected BRep Vertex hoặc component không có tangent/surface frame cũng báo
`UnsupportedFrame`; giữ alignment và frame trước, không dùng owner axes ngầm.

Frame là view state; không tạo object/property/Undo. Preference alignment lưu
profile, shared with menu/CMD. Bộ frame/pivot theo incarnation/revision của
một owner lưu trong session cache bounded 256; không serialize cache vào FCStd.
Chọn lại owner còn nguyên giữ relocated pivot; thay source/explicit Alignment
xóa giá trị cũ. Multi-selection không dùng single-owner cache.

Native checks: object 30° / rotated one-edge Wire, rotated CPlane, menu states,
World, unsupported Mesh component, unchanged Shape/count/Undo và reselect.
Final-review regression kiểm Vertex1 giữ rotated CPlane và không tạo Undo.
[Evidence](../validation/2026-10-11-gumball-core.md).
Tiếp theo: Mesh face/edge/vertex, light/camera frame adapters và source fixtures.
