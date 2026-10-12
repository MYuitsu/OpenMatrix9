# Layer panel — triển khai và nghiệm thu, 2026-10-11

**OM9-LAYER-001: partially_implemented / validated_supported_slice.**
[Spec](../../OpenMatrix9_Codex_Spec_v1/specs/01-core/om9-layer-001-layer-management.md).
Panel và output slice đã hoạt động trên shared host tương thích; không nâng toàn feature thành hoàn tất.
Người dùng xác nhận bỏ combo/footer C A L V C; đây là yêu cầu trực tiếp, không suy từ manual.

## Kết quả

Current và Assign riêng; swatch phải; lock/OM9 click/Region/edit/Delete; object Hidden,
layer/group Off/On; Hide/Show trái/phải; Advanced thu/mở, rename/color và Cancel;
Lights33; FCStd/Undo/Redo. Modal document session + ObjectStamp từ chối stale rename/color/Show
kể cả mở lại cùng file, persistent UID và native ID không đổi.
Root Curve/Solid/Surface/Edit/Rebuild nhận current tại commit; Explode giữ source ownership.
Native representative outputs: Line, Box, Sphere, Loft, Circle/Ellipse, Join, Explode, Rebuild.

## Bằng chứng

Build chỉ **OpenMatrix9Gui**, shared SDK `H:/FreeCAD-src/build/relWithDebInfo`,
cache có sẵn `build/openmatrix9-history-cage`; không configure/build FreeCAD root.
Plugin-only output `H:/FreeCAD-src/build/om9-layer-panel-runtime-20261011` và test profiles riêng.
Module SHA256 `2eaaa9829e19cc389dc27385d063d92c67479bfb42fdc64aa89db35a9d162ac9`.
Mỗi native suite kiểm exact selected pyd, `ok=true`, mọi check true, runner exit0.
Runner: `build/run-om9-plugin.ps1 -PluginRuntime build/om9-layer-panel-runtime-20261011 -Macro <fixture> -OutputDirectory <report-directory> -TimeoutSeconds 600`.

| Suite | Checks đạt | Fresh report |
|---|---:|---|
| panel | 71 | `H:\FreeCAD-src\build\shared-host-tests\layer-panel-complete-20261011\results.json` |
| outputs | 21 | `H:\FreeCAD-src\build\shared-host-tests\layer-outputs-complete-20261011\results.json` |
| permission | 5 | `H:\FreeCAD-src\build\shared-host-tests\layer-permission-complete-20261011\results.json` |
| modal | 5 | `H:\FreeCAD-src\build\shared-host-tests\layer-modal-complete-20261011\results.json` |
| circle | 51 | `H:\FreeCAD-src\build\shared-host-tests\circle-layer-complete-20261011\results.json` |
| mouse | 51 | `H:\FreeCAD-src\build\shared-host-tests\layer-mouse-complete-20261011\results.json` |

Tổng native **204/204**. Rust **11/11**, command
`cargo test --manifest-path rust/Cargo.toml --locked --offline --test layer_panel --test layers`.
Isolated layers Rustfmt/Clippy đạt. Public source audit: 2454 source files,
511 bindings, 510 SVG, errors=[].
Ảnh kiểm actual Qt panel: `H:/FreeCAD-src/build/shared-host-tests/layer-panel-complete-20261011/layers-panel.png`.
[JSON fingerprints và scope](2026-10-11-layer-panel-implementation.json).

## Review và sửa lỗi

Review độc lập phát hiện hai Important issues, đã tái hiện RED trên native plugin:
Rebuild mặc định xem layer group như dependent; Tree Show không phục hồi Selectable.
Fix bỏ qua chỉ validated OM9 layer owner; real dependent vẫn chặn; native Selectable chỉ
mask lock, giữ explicit preference riêng khỏi visibility. GREEN fixtures kiểm defaults,
Undo/Redo, dependency rejection và quyền selectable true/false. Review lần sửa không còn Important.
Fixture Edit chọn visible dialog thay vì hidden Join dialog chờ DeferredDelete; giữ diagnostics.

## Giới hạn

- MatrixArt/material initialization, general imported-layer hierarchy, mọi semantic builder
  family và session-time output-layer binding chưa nghiệm thu. Lights không tạo illumination.
- Lock bảo vệ OM9 dispatcher/mouse; không tuyên bố mọi raw FreeCAD host command là permission service.
- Không thể phục hồi ý định visibility của file legacy đã bị adapter cũ ghi đè mà không có dữ liệu riêng.
- Full crate Clippy gặp unrelated WIP lints và sandbox config access; isolated layers clean.
  Legacy standalone Qt test target chưa chạy; native gesture fixtures là bằng chứng UI.
- Bản build dùng SDK dev; launcher chính FreeCAD1.1.4 khác ABI chưa được cập nhật bằng pyd này.
- Ảnh chỉ xác nhận layer panel, không chứng nhận viewport rendering/palette pixel parity.
- Người dùng đã duyệt lưu nghiệp vụ trong skill mới `openmatrix9-layers-contract`; bản nguồn và bản cài đặt đồng bộ. [Skill record](2026-10-11-layers-skill.md).

[Audit trước triển khai](2026-10-11-layer-management-audit.md) giữ bằng chứng lịch sử riêng;
source fingerprints của audit cũ không còn đại diện cho source sau triển khai.
