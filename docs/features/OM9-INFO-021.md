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
Mesh/Points whole-owner dùng Placement; Native planar oriented Mesh Segment dùng actual world facet tangent/normal; Mesh edge/vertex, light/camera
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
Planar oriented Mesh Segment frame và Scale Segment retention/Undo/Redo/cache có native13checks.
Nonplanar/degenerate/multiple components reject; Mesh edge/vertex và light/camera frame adapters còn mở.

Revision spec ngày 2026-10-11 yêu cầu một frame dùng chung cho cả bốn
overlays: World/Object giữ qua idle focus/camera; CPlane active recapture
đồng bộ khi idle, frozen trong gesture; relocated frame giữ tới explicit
Reset/Alignment/source invalidation. Default CPlane được giữ, World dùng
so ảnh. Menu/CMD và Scale bounds/hit targets phải cập nhật mọi ô.
Đã triển khai một frame và collection overlay đồng bộ cho mọi ô. Native fixtures
kiểm World qua focus, Ctrl widget-only từng ô, Relocate basis/optional Y, zoom,
maximize/restore, source/passive view close và preview/cancel. Relocated transient
group frame giữ trong selection đó, không nhận cache single-owner khác.
[Evidence mới](../validation/2026-10-11-gumball-four-view-completion.md).
[Đối chiếu nguồn lịch sử](../validation/2026-10-11-gumball-four-view-spec-review.md).
