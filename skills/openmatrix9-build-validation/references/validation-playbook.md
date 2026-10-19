# Build và runtime theo target v2

Đọc [target](../../openmatrix9-workflow/references/milestones-target.md), M01 L1 và L2/L3 owner liên quan. Resolve checkout và actual manifests/targets; chưa có standalone executable thì ghi not_implemented, không invent build command hoặc dùng FreeCAD module thay nó.

## Standalone

1. Semantic Rust tests phù hợp change. Existing rust/Cargo.toml là nguồn portable/legacy components; examples/rust chỉ request planners. Chọn crate thực được sửa; fmt/Clippy/release theo actual build policy.
2. Native adapters cần compiler/toolchain/architecture/ABI matching, headers/exports/dependency closure/version/license manifest. Qym kernel build.rs full OCAF/XCAF/STEP links không là subset mặc định. Error/exception không qua ABI, handle ownership/lifetime/threads/cancel/budgets rõ.
3. Actual OM9 executable/package không link/load/launch FreeCAD, SDK host hoặc Python workbench. Test standalone document/global atomic publication/old Redo/Undo/cold .om9/3DM và source-stale/permissions/failure/no partial writes theo feature.
4. UI thực giữ Matrix9 như mô tả: menu/labels/options, CMD, panels, F6, Layers/Builder, bốn camera/view và Gumball. Nine visible recent commands theo contract v2; old menu capacity20 chỉ là legacy implementation. Screenshot/pointer/DPI/resize/focus/lifecycle, không mock canvases hoặc inferred parity.
5. Nếu chạm rendering/drag, đo latency/frame/commit riêng trên fixture có hash, cache/matrix affine preview và nguồn bất biến; không nới geometry tolerance để tăng tốc. Geometry-changing previews kiểm riêng.

## Legacy FreeCAD khi nhiệm vụ cụ thể cần

Dùng duy nhất shared SDK `H:/FreeCAD-src/build/relWithDebInfo`, OM9-only target OpenMatrix9Gui qua `build/build-om9-plugin.ps1`, profile/runtime qua `build/run-om9-plugin.ps1`. Không root rebuild hoặc copy runtime cho thay đổi OM9. Root rebuild chỉ cho actual FreeCAD core/headers/ABI/toolchain dependency changes, nêu lý do trước khi làm. Verify actual module path/ABI/profile/process exit; .FCMacro fixtures cũ không đóng standalone gate.

## Record

Lưu source/hash/local changes, target/profile/versions, commands/results/logs, actual screenshots, tested scope, UI differences và phần chưa chạy trong docs/validation. Sync v2/README/ledger theo evidence. Không broaden/repeat checks đã qua nếu không có thay đổi/failure/concern mới. Preserve private models và raw sources; runtime/public payload chỉ authorized assets/code, không publish ngầm.
