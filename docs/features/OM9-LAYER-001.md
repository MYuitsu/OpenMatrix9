# OM9-LAYER-001 — Layer Management

Panel Layers đã triển khai và nghiệm thu trên plugin dùng shared SDK, ngày 2026-10-11.
Trạng thái **partially_implemented / validated_supported_slice**.

- Tách chọn current khỏi arrow Assign; right swatch chọn members được phép.
- Lock chặn OM9 click/Region Select/edit/Delete; unlock giữ Selectable trước đó.
- Layer/group Off/On, object Hidden và Hide/Show trái/phải giữ trạng thái riêng.
- Advanced mặc định thu, Shift+right tên/màu, kiểm stale modal và FCStd.
- Lights slot33; footer combo/C A L V C bỏ theo xác nhận của người dùng.
- Supported root outputs nhận current; Explode giữ ownership, Rebuild defaults và real dependents được kiểm tra.

Rust **11/11**; native **panel 71/71, outputs 21/21, permission 5/5, modal 5/5, circle 51/51, mouse 51/51**; build OpenMatrix9Gui và public-source audit đạt.
Native evidence dùng FreeCAD SDK tương thích, chưa triển khai vào launcher chính FreeCAD1.1.4 khác ABI.
Art/material, general import hierarchy và mọi semantic output family chưa nghiệm thu.

[Spec](../../OpenMatrix9_Codex_Spec_v1/specs/01-core/om9-layer-001-layer-management.md) ·
[Report và giới hạn](../validation/2026-10-11-layer-panel-implementation.md) ·
[Audit trước triển khai](../validation/2026-10-11-layer-management-audit.md).
