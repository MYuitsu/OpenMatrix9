---
name: openmatrix9-workspace-contract
description: Use when implementing or reviewing OpenMatrix9 workspace startup defaults, the Matrix-style Command area, finite construction grid, viewport titles/dropdowns, CAD Wireframe/Shaded display or straight Line/Polyline interaction.
---

# Matrix workspace business contract

Đọc [đích specs v2 và hợp đồng UI Matrix9](../openmatrix9-workflow/references/spec-v2-target.md) trước lựa chọn kiến trúc, spec hoặc runtime. Package đích `OpenMatrix9_Codex_Spec_v2`; QymCAD chỉ cho code mẫu/routine, giao diện giữ Matrix9 như mô tả hiện tại.

Rust là ưu tiên số 1 khi triển khai hoặc chuyển code OM9. Đọc [quy tắc Rust và ranh giới native](../openmatrix9-workflow/references/rust-first.md) trước khi chọn ngôn ngữ/FFI/worker. Safe Rust sở hữu logic, validation và dữ liệu độc lập; C++ chỉ là bridge cho API native bắt buộc, Python cho bootstrap/test/tool khi cần. Ghi rõ lý do và owner của mọi ngoại lệ; không coi native FFI là memory-safe hoặc skill update là migration hoàn tất.

Use this one skill for the workspace business group. These rules were approved by the user on 2026-10-06: "push code lên giúp tôi nha và lưu vào skill mới" after reviewing the grid/Command and viewport proposals. Packaged screenshots are user references, not runtime proof.

## Mặc định giao diện khi khởi động OpenMatrix9

Người dùng duyệt ngày 2026-10-11: "ghi vào quy tắt".

- Không hiển thị thanh công cụ `3DM` riêng chứa hàng icon Import/Export thừa khi khởi động OpenMatrix9. Giữ các lệnh Import/Export trong menu chữ và khu vực File; giữ các toolbar chuẩn khác.
- Khi OpenMatrix9 khởi động, ẩn dock `Tasks` nếu không có Task dialog đang chạy. Lệnh cần Task vẫn được mở khung này bình thường; không đóng hoặc ẩn Task đang hoạt động.
- Đây là mặc định riêng của OpenMatrix9, áp dụng trong phạm vi khởi động. Không suy rộng thành quy tắc cho mọi workspace/editing domain hoặc chứng nhận toàn bộ vòng đời dock/Task.

Read only the relevant reference:

- [Grid and viewports](references/grid-and-viewports.md): finite cell counts, colors, per-view labels, CAD Wireframe/isocurves, display menus and single-view/4V behavior. Includes the user-approved Wireframe update of 2026-10-08 and verified limits.
- [Command and Curve input](references/command-and-curve.md): transcript, pinned input, resizing, completion and supported Line/Polyline options.

Keep original Matrix command names and share one implementation across menu, mouse and CMD. Use repository `OpenMatrix9_Codex_Spec_v2/specs` for feature details; the user explicitly says these specs are sufficient, so do not reread PDFs. Follow `openmatrix9-workflow` for progress and `openmatrix9-build-validation` for native validation. This contract supplements the UI skill; it does not prove full core/Curve completion.

The user deferred choosing the middle-button mapping until checking Matrix directly. Do not change that default from an earlier answer or infer it from the screenshots.

After coding and verification, present any new business behavior and ask whether to save it to the skill. Explicit user authorization for that exact update is sufficient; do not ask again. Record technical evidence independently of business consent. Sync repository and installed skill copies after approved updates.
