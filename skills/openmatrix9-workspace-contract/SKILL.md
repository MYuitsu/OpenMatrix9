---
name: openmatrix9-workspace-contract
description: Use when implementing or reviewing OpenMatrix9's Matrix-style Command area, finite construction grid, viewport titles/dropdowns, CAD Wireframe/Shaded display or straight Line/Polyline interaction.
---

# Matrix workspace business contract

Rust là ưu tiên số 1 khi triển khai hoặc chuyển code OM9. Đọc [quy tắc Rust và ranh giới native](../openmatrix9-workflow/references/rust-first.md) trước khi chọn ngôn ngữ/FFI/worker. Safe Rust sở hữu logic, validation và dữ liệu độc lập; C++ chỉ là bridge cho API native bắt buộc, Python cho bootstrap/test/tool khi cần. Ghi rõ lý do và owner của mọi ngoại lệ; không coi native FFI là memory-safe hoặc skill update là migration hoàn tất.

Use this one skill for the workspace business group. These rules were approved by the user on 2026-10-06: "push code lên giúp tôi nha và lưu vào skill mới" after reviewing the grid/Command and viewport proposals. Packaged screenshots are user references, not runtime proof.

Read only the relevant reference:

- [Grid and viewports](references/grid-and-viewports.md): finite cell counts, colors, per-view labels, CAD Wireframe/isocurves, display menus and single-view/4V behavior. Includes the user-approved Wireframe update of 2026-10-08 and verified limits.
- [Command and Curve input](references/command-and-curve.md): transcript, pinned input, resizing, completion and supported Line/Polyline options.

Keep original Matrix command names and share one implementation across menu, mouse and CMD. Use repository `ref/matrix9/OpenMatrix9_Codex_Spec_v1/specs` for feature details; the user explicitly says these specs are sufficient, so do not reread PDFs. Follow `openmatrix9-workflow` for progress and `openmatrix9-build-validation` for native validation. This contract supplements the UI skill; it does not prove full core/Curve completion.

The user deferred choosing the middle-button mapping until checking Matrix directly. Do not change that default from an earlier answer or infer it from the screenshots.

After coding and verification, present any new business behavior and ask whether to save it to the skill. Explicit user authorization for that exact update is sufficient; do not ask again. Record technical evidence independently of business consent. Sync repository and installed skill copies after approved updates.
