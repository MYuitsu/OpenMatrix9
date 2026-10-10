---
name: openmatrix9-ui
description: Use when rebuilding the Matrix9 startup layout, MAIN MENU, sidebar, icons, F6 context menu, Command region, construction grid or viewport colors inside the Rust and FreeCAD OpenMatrix9 workbench.
---

# Matrix9 interface in FreeCAD

Rust là ưu tiên số 1 khi triển khai hoặc chuyển code OM9. Đọc [quy tắc Rust và ranh giới native](../openmatrix9-workflow/references/rust-first.md) trước khi chọn ngôn ngữ/FFI/worker. Safe Rust sở hữu logic, validation và dữ liệu độc lập; C++ chỉ là bridge cho API native bắt buộc, Python cho bootstrap/test/tool khi cần. Ghi rõ lý do và owner của mọi ngoại lệ; không coi native FFI là memory-safe hoặc skill update là migration hoàn tất.

For the user-approved finite grid, pinned Command/completion and viewport title/menu contract, read `openmatrix9-workspace-contract` (repository `skills/openmatrix9-workspace-contract/SKILL.md`). Its packaged references supplement the earlier theme baseline below.

Keep Rust responsible for catalog, state and behavior; C++/Qt hosts widgets and native FreeCAD views. Python registers the workbench.

1. Use openmatrix9-workflow `status`, then `route ui-menu` or `route ui-viewports`. Read the existing design and relevant task before changing code. Existing user authorization to continue remains valid; an old document's approval wording does not require a new approval.
2. Resolve the exact UI feature ID. Read `Resources/menu/MainMenu.ini`, the configuration included by Rust, for menu order and counts. Use [ui-evidence.md](references/ui-evidence.md) for the component being built; inspect one relevant form/resource at a time.
3. Implement a working slice: Rust catalog/state → C ABI → Qt host → verified FreeCAD command. Unsupported commands remain visible but disabled. Preserve access to FreeCAD menus and document editing.
4. Distribute extracted, traceable assets with the module. The installed workbench must run without `ref` or this machine's absolute paths.
5. Use openmatrix9-build-validation for native build, runtime comparison and lifecycle checks. Update the live progress ledger with evidence and the next document.

Menu structure does not require reading the entire Ghidra export. If a specific event's behavior is unclear after metadata/spec reading, use openmatrix9-native-mapping for that event alone.

Compare against the user's selected screenshot. For Command, construction grids and viewport colors, read [command-grid-theme.md](references/command-grid-theme.md) and its packaged reference image: Command belongs above the native views as one selectable text document containing history, live prompt and input; Polyline shows a picked-point chain before Enter commits it. The Command region is Matrix green, viewport canvases are black, and major grid cells contain minor subdivisions. These user choices supersede the earlier gradient preview.

If a selected reference is unavailable, report that visual gap while continuing from verified sources. MAIN MENU/ICON HISTORY and native four-view workspace are implemented slices with remaining functional gaps; other controls enable only proven commands. Reconcile the live ledger rather than assuming all features complete from matching appearance.

## Cập nhật nghiệp vụ sau khi code

Sau khi hoàn tất code và kiểm tra, nếu có đổi nghiệp vụ, hỏi người dùng có cập nhật nghiệp vụ đó vào skill không. Chỉ ghi quy tắc mới khi có đồng ý; nếu người dùng đã yêu cầu cập nhật skill cho chính thay đổi này thì không hỏi lại. Áp dụng [quy tắc xác nhận cập nhật nghiệp vụ](../openmatrix9-workflow/references/business-rule-updates.md) để phân biệt nghiệp vụ mới với sửa kỹ thuật, chuẩn bị đề xuất cụ thể và đồng bộ skill sau khi được đồng ý.
