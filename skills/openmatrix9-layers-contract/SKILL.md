---
name: openmatrix9-layers-contract
description: Use when implementing, reviewing or resuming the OpenMatrix9 Layers panel, current-layer output, layer assignment, lock and selection, Hide/Show or Advanced layer customization. Includes regressions where layer toggles lose object visibility or Selectable permission.
---

# OpenMatrix9 Layers business contract

Đọc [đích specs v2 và hợp đồng UI Matrix9](../openmatrix9-workflow/references/spec-v2-target.md) trước lựa chọn kiến trúc, spec hoặc runtime. Package đích `OpenMatrix9_Codex_Spec_v2`; QymCAD chỉ cho code mẫu/routine, giao diện giữ Matrix9 như mô tả hiện tại.

Áp dụng cho **OM9-LAYER-001**. Người dùng đồng ý lưu thành skill mới ngày 2026-10-11: “có lưu trong skill mới”. Các quy tắc dưới đây là hợp đồng OM9 đã kiểm chứng; không chứng nhận toàn bộ Matrix/Rhino.

## Thao tác panel

| Control | Hành vi |
|---|---|
| Tên hoặc swatch, chuột trái | Chọn current layer cho output mới; không chuyển object đã có. |
| In arrow, chuột trái | Gán selection sang layer đích, đổi màu và membership; giữ current. Kiểm cả batch trước khi ghi. |
| Swatch, chuột phải | Thay selection bằng members visible, selectable, không Hidden/off/locked. |
| Lock | Giữ display, chặn OM9 pick/Region Select/edit/Delete và output trên layer khóa; unlock phục hồi quyền Selectable trước đó. |
| Visibility layer/nhóm | Bật/tắt display, giữ explicit visibility và object Hidden. Nhóm gồm 4 slot: tất cả hiện thì tắt, mixed thì hiện. |
| Hide / Show, chuột trái | Hide selection; Show objects do OM9 Hide. Không bật layer Off hoặc object explicit invisible khác. |
| Hide / Show, chuột phải | Hide phần visible ngoài selection; Show mở danh sách hidden để hiện phần được chọn. Cancel không ghi. |
| Advanced | Mặc định thu; mở/thu không xóa state. Chỉ slot17..32 có Shift + chuột phải tên để rename hoặc swatch để đổi màu. |
| Lights | Slot33 riêng ngoài 32 model slots; có current/Assign/selection/lock/visibility, không tự tạo illumination. |
| Footer combo / C A L V C | Bỏ toàn bộ theo xác nhận của người dùng; không dựng lại từ screenshot. |

Khối cơ bản: trái Metal01–04/Gem01–04; phải User01–04/Heads/Finger/Cutting/Creation. Advanced: trái User17–20/25–28; phải User21–24/29–32. Current có highlight.

CMD nhận `Layer=1..33`, tên chuẩn/tên tùy biến, `Layer=Lights`, `Layer=None` hoặc 0. New document có current=None; group tạo lazy, visible/unlocked, giữ palette OM9 hiện hữu. Rename chỉ đổi Label, giữ stable OM9 identity/slot; tên trim, không rỗng/control characters, tối đa 128 UTF-8 bytes, không trùng tên layer khác hoặc tên chuẩn dành riêng.

## State, ownership và lifetime

- Giữ layer visibility, explicit object visibility, `OM9ObjectHidden` và explicit Selectable riêng trong .om9. Effective OM9 Selectable chỉ mask lock; visibility và OM9 guards ngăn hidden picks. Không ghi false do visibility trở lại thành lựa chọn Selectable của người dùng; Tree Show phải giữ quyền chọn.
- Assign từ chối source/target khóa, foreign document và structural/placed/import ownership không an toàn, không chuyển một phần batch. Không tự di chuyển storage/rig/cage.
- Supported root Curve/Solid/Surface/Edit/Rebuild outputs lấy current ngay trước transaction commit. Current=None giữ style hiện hữu; Circle/Ellipse giữ xanh legacy. Layer Off nhận output invisible. Explode giữ source ownership/placement và layer metadata. Rebuild DeleteInput chỉ bỏ qua validated plain root OM9 layer owner; real dependents vẫn chặn.
- Layer actions có transaction/abort; output dùng transaction command hiện hữu. Kiểm Undo/Redo và .om9 cho membership, name/color, Hidden, lock và current.
- Rename/color/selective Show giữ snapshot có document session và object generation. Sau modal, revalidate; đóng/mở lại cùng file vẫn là session mới dù Name/UID/ID giống nhau. Không dùng retained raw pointer hoặc chỉ so persistent UID để cho phép stale write.

## Ranh giới triển khai và nghiệm thu

Safe Rust sở hữu slot/default/parser/action/name/visibility/permission policy; OM9 document/permissions/journal quản lý properties, groups, selection, transactions và labels; C++ chỉ bridge geometry native cần thiết. Python dùng test/tool. Đối chiếu quy tắc Rust của dự án trước khi thêm FFI.

Tìm checkout OM9 hiện hành, đọc spec `OpenMatrix9_Codex_Spec_v2/specs/01-core/om9-layer-001-layer-management.md` và report `docs/validation/2026-10-11-layer-panel-implementation.md`. Mốc đã kiểm chứng: Rust11/11, native204/204; đây là evidence lịch sử, không thay kiểm tra source/runtime mới. Native fixtures: `layer_panel_smoke`, `layer_outputs_smoke`, `layer_visibility_permission_smoke`, `layer_modal_lifecycle_smoke`, `circle_layers_smoke`, `core_mouse_smoke` trong `tests/*.FCMacro`.

Target v2 dùng executable OM9 và selective backend theo build-validation; chưa có target thì ghi chưa triển khai. Plugin build/run scripts và .FCMacro chỉ cho bảo trì/đối chiếu legacy FreeCAD, không đóng gate v2. Đồng bộ spec/catalog/README/live ledger theo evidence thực tế.

MatrixArt/material, general imported-layer hierarchy, mọi semantic builder family và session-time output-layer binding còn ngoài slice này. Không dùng lock của OM9 để tuyên bố mọi native/backend operation đã có permission service; snap/reference là policy riêng. Legacy visibility đã bị code cũ ghi đè không đủ dữ liệu để tự khôi phục ý định. Unsupported commands giữ disabled.

Khi có thay đổi nghiệp vụ mới sau này, hoàn tất code/kiểm tra rồi theo quy tắc xác nhận cập nhật skill của dự án. Đồng ý lưu skill lần này đã đủ cho hợp đồng trên; không hỏi lại cho cùng nội dung.
