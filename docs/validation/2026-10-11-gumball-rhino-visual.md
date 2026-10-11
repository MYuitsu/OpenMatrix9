# Gumball theo giao diện Rhino 5 — 2026-10-11

User duyệt chỉnh phần nhìn của Gumball lõi: X đỏ, Y xanh lá, Z xanh dương;
arrows/arcs thanh; outline Scale squares ở phía đối diện arrows; menu ball
cạnh tâm; numeric editor gọn. Đây là revision của thiết kế Z vàng trước đó.
Ba spec `OM9-INFO-020/021/022` vẫn `partially_implemented` /
`validated_supported_slice`; không nâng thành full Matrix/Rhino parity.

Đã đối chiếu [Rhino 5 Gumball Help](https://docs.mcneel.com/rhino/5/help/en-us/commands/gumball.htm)
và hai hình minh họa chính thức. Không sao chép artwork vào source public.

## Implementation và ranh giới native

`GumballOverlay` dựng Coin scene primitives riêng thay hình mặc định của
FreeCAD dragger: cones, line arcs, camera-facing squares, dashed guides không
pickable và menu disk. Scene paths giữ 13 Rust operation IDs; menu ball dùng
native ID 13, được chặn trước Rust gesture begin. Overlay vẫn bounds-excluded,
preview unpickable; không thêm document objects. Coin camera rotation và float
chỉ dùng để vẽ. Rust giữ operation/session/frame/ray math và numeric validation.

Ngoại lệ C++ cần thiết: Coin scene graph/picking và Qt editor/menu thuộc native
viewer. Relocate/Reset/CPlane/Object/World gọi handlers hiện hữu; Reset recapture
tâm selection theo alignment hiện hành. Menu nhận release của viewport press
không để cờ suppressRelease nuốt lần click transform kế tiếp. Không bổ sung
business/geometry solver trong C++ cho visual revision này.

## Verification

Chỉ build `OpenMatrix9Gui` với shared SDK `H:/FreeCAD-src/build/relWithDebInfo`,
cache `H:/FreeCAD-src/build/openmatrix9-history-cage`, 14 workers. Không build
root FreeCAD hay copy host runtime. Build cuối exit 0.

```powershell
rtk proxy powershell -NoProfile -ExecutionPolicy Bypass -File H:/FreeCAD-src/build/build-om9-plugin.ps1 -RuntimeDirectory H:/FreeCAD-src/build/gumball-rhino-runtime-20261011
rtk proxy powershell -NoProfile -ExecutionPolicy Bypass -File H:/FreeCAD-src/build/run-om9-plugin.ps1 -PluginRuntime H:/FreeCAD-src/build/gumball-rhino-runtime-20261011 -Macro tests/gumball_visual_smoke.FCMacro -OutputDirectory H:/FreeCAD-src/build/shared-host-tests/gumball-rhino-visual-20261011/recheck
rtk proxy powershell -NoProfile -ExecutionPolicy Bypass -File tests/run_gumball_smoke.ps1 -PluginRuntime H:/FreeCAD-src/build/gumball-rhino-runtime-20261011 -Scope Core -OutputDirectory H:/FreeCAD-src/build/shared-host-tests/gumball-rhino-visual-20261011/recheck-core
rtk cargo test --locked --release --manifest-path rust/Cargo.toml --target-dir H:/FreeCAD-src/build/openmatrix9-history-cage/Gui/rust-target
rtk proxy python tools/public_source_audit.py
```

Đặt `QT_SCALE_FACTOR=1.5` trước visual runner để tái kiểm DPI 150%.
Evidence root: `H:/FreeCAD-src/build/shared-host-tests/gumball-rhino-visual-20261011`.

| Evidence | Kết quả |
|---|---|
| `red/results.json` trên accepted core binary cũ | 8 lỗi / 12 checks: màu, Scale placement, numeric size, thiếu menu ball |
| `menu-red2/results.json` trước release fix | Chỉ lỗi next handle click sau popup-owned release; Relocate/Reset đạt |
| `final-pointer-dpi1` | 23/23, actual host exit 0 |
| `final-pointer-dpi150` | 23/23, actual host exit 0 |
| `final-core/suite.json` | 210/210, 9 processes, mọi actual exit 0 |
| `rust-test.log` | 826 tests, 119 suites gồm doc-tests, zero failures, exit 0 |

Hai visual runs kiểm actual rendered RGB pixels, native scene-path targets,
opposite-side Scale positions, compact editor bounds, numeric Move/Scale/Rotate,
mouse drag Scale ra xa tâm nở đúng X/pivot trong một Undo, popup pointer input,
alignment/Relocate/Reset và geometry/Undo unchanged cho view operations.
Đã xem `final-pointer-dpi1/gumball-render.png` trực tiếp.
Core runs kiểm commands, ownership, lifecycle, callbacks, cancel và mọi native
operation/modifier đã hỗ trợ tại DPI 1 và 1.5. Tổng cuối 256 native checks /
11 processes. Không rerun persistence/regression ngoài Core cho visual-only
revision; [evidence lõi trước đó](2026-10-11-gumball-core.md) có phạm vi riêng.

Mọi final native run dùng cùng plugin SHA256:
`A058D753A01AFC5369F3CD1B7552C5DE8E78373E7DAB271149F63C224BFAB8DF`.
Shared host SHA256:
`98EEF899354A8AA84CFA2934E6E1A3C8558C7E6FFF3B89C347C06731F06018A8`.
Native/Rust runs dùng dirty-source baseline được giữ nguyên; commit visual
chỉ chứa thay đổi Gumball, không gom các CAD family edits khác.

## Demo và giới hạn

Cửa sổ `Rhino-style Gumball Demo` đã mở trên shared host với đúng final plugin,
isolated profile tại `build/shared-host-tests/gumball-rhino-live-demo-20261011`.
`demo-ready.json` ghi module path, enabled state, selection và attached overlay.
Windows Computer Use trả `window id ... no longer belongs ...` khi activate/state
capture, vẫn lỗi sau một lần refresh/retry. Không tuyên bố đã drag trực tiếp qua
Windows automation; native Qt pointer fixtures nêu trên đã chạy thành công.

Owner/capability/snap/persistence/Redo limits giữ như
[feature record](../features/OM9-INFO-020.md). Menu chỉ cung cấp operations được
hỗ trợ; chưa có toàn bộ menu/preferences hay visual parity trong mọi camera.
Revision này chưa phải bản phát hành GitHub hoặc binary chứng nhận cho stock
FreeCAD 1.1.4; runtime đang chạy là shared development host.
