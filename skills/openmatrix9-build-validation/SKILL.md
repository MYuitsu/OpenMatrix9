---
name: openmatrix9-build-validation
description: Use when compiling, linking, loading or verifying the Rust and C++/Qt OpenMatrix9 workbench, checking its FreeCAD startup layout, or diagnosing ABI and build-environment failures.
---

# Prove the native workbench works

Rust là ưu tiên số 1 khi triển khai hoặc chuyển code OM9. Đọc [quy tắc Rust và ranh giới native](../openmatrix9-workflow/references/rust-first.md) trước khi chọn ngôn ngữ/FFI/worker. Safe Rust sở hữu logic, validation và dữ liệu độc lập; C++ chỉ là bridge cho API native bắt buộc, Python cho bootstrap/test/tool khi cần. Ghi rõ lý do và owner của mọi ngoại lệ; không coi native FFI là memory-safe hoặc skill update là migration hoàn tất.

Resolve the OpenMatrix9 and FreeCAD source roots. Run openmatrix9-workflow `status` and `route build-validation`; inspect the actual Cargo/CMake/ABI and parent registration before choosing build commands. A FreeCAD executable or plan checkbox does not prove OpenMatrix9 was built.

Use [validation-playbook.md](references/validation-playbook.md) for environment detection, commands and runtime cases. Recheck source/build/prefix paths on this machine. If an existing cache points to the old E: checkout, choose a fresh appropriate D: build rather than deleting or blindly reusing it. The route's machine-specific paths are candidates to verify, not portable constants.

Verify progressively:

1. Rust semantic tests, formatting, Clippy and release build.
2. Complete C ABI agreement, C++ compilation/linking, target presence and packaged resources.
3. Actual module import/workbench activation in FreeCAD.
4. Selected UI or feature behavior, document interaction and lifecycle against source evidence.

Record the exact command/environment/result and runtime artifacts in `docs/validation/<date>-<slice>.md`. Capture actual screenshots with available authorized UI tools; generated mockups are not runtime evidence. If a check is unavailable, identify it as untested and continue independent checks.

Update the live progress ledger only to the level supported by evidence. Rust passing alone cannot validate the native workbench; successful loading alone cannot validate feature geometry. Read only the relevant feature/UI specs for expected behavior. Use openmatrix9-native-mapping only when a specific original behavior must be resolved, not for normal compiler errors.

After checks, follow [progress synchronization](../openmatrix9-workflow/references/progress.md) before reporting completion: check relevant Spec v1 acceptance items and align implementation status, the project README and the ledger with the verified slice and remaining untested behavior.

Finish within existing authorization. Keep unrelated user changes and original reference inputs intact; stage only the selected source/docs/artifacts when committing.

## Cập nhật nghiệp vụ sau khi code

Sau khi hoàn tất code và kiểm tra, nếu có đổi nghiệp vụ, hỏi người dùng có cập nhật nghiệp vụ đó vào skill không. Chỉ ghi quy tắc mới khi có đồng ý; nếu người dùng đã yêu cầu cập nhật skill cho chính thay đổi này thì không hỏi lại. Áp dụng [quy tắc xác nhận cập nhật nghiệp vụ](../openmatrix9-workflow/references/business-rule-updates.md) để phân biệt nghiệp vụ mới với sửa kỹ thuật, chuẩn bị đề xuất cụ thể và đồng bộ skill sau khi được đồng ý.
