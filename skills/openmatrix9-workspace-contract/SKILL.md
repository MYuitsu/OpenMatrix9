---
name: openmatrix9-workspace-contract
description: Use when implementing or reviewing OpenMatrix9 workspace startup defaults, the Matrix-style Command area, finite construction grid, viewport titles/dropdowns, CAD Wireframe/Shaded display or straight Line/Polyline interaction.
---

# Matrix workspace business contract

Đọc [đích milestone và giao diện Matrix9](../openmatrix9-workflow/references/milestones-target.md) trước lựa chọn kiến trúc, spec hoặc runtime. Nguồn nền tảng `OM9_Milestones/M01 - CAD Shell, Viewport`; QymCAD chỉ cho code mẫu/routine, giao diện giữ Matrix9 như mô tả hiện tại.

Rust là ưu tiên số 1 khi triển khai hoặc chuyển code OM9. Đọc [quy tắc Rust và ranh giới native](../openmatrix9-workflow/references/rust-first.md) trước khi chọn ngôn ngữ/FFI/worker. Safe Rust sở hữu logic, validation và dữ liệu độc lập; C++ chỉ là bridge cho API native bắt buộc, Python cho bootstrap/test/tool khi cần. Ghi rõ lý do và owner của mọi ngoại lệ; không coi native FFI là memory-safe hoặc skill update là migration hoàn tất.

Use this one skill for the workspace business group. These rules were approved by the user on 2026-10-06: "push code lên giúp tôi nha và lưu vào skill mới" after reviewing the grid/Command and viewport proposals. Packaged screenshots are user references, not runtime proof.

## Mặc định giao diện khi khởi động OpenMatrix9

Người dùng duyệt ngày 2026-10-11: "ghi vào quy tắt".

- Không hiển thị thanh công cụ `3DM` riêng chứa hàng icon Import/Export thừa khi khởi động OpenMatrix9. Giữ các lệnh Import/Export trong menu chữ và khu vực File; giữ các toolbar chuẩn khác.
- Khi OpenMatrix9 khởi động, ẩn dock `Tasks` nếu không có Task dialog đang chạy. Lệnh cần Task vẫn được mở khung này bình thường; không đóng hoặc ẩn Task đang hoạt động.
- Đây là mặc định riêng của OpenMatrix9, áp dụng trong phạm vi khởi động. Không suy rộng thành quy tắc cho mọi workspace/editing domain hoặc chứng nhận toàn bộ vòng đời dock/Task.

## Vùng workbench docking (OM9-L3-01-01-002)

Người dùng đồng ý ngày 2026-10-16: trả lời "làm xong các việc chưa xong cho xong M01 L2-01-01 L3-002" cho đề xuất nội dung skill trong báo cáo cuối của OM9-L3-01-01-002.

Người dùng đồng ý ngày 2026-10-16: quyết định cửa sổ tối thiểu 1280x720 logic, ngưỡng dock phải giao cho agent chọn (chỉ cập nhật hai dòng số tối thiểu và ngưỡng dock bên dưới).

- Rail trái, bốn viewport và ô nhập lệnh luôn được giữ; dock phải nhường khi cửa sổ hẹp hơn 1200 điểm logic.
- Mức tối thiểu được hỗ trợ là 1280x720 điểm logic, áp ở cấp cửa sổ thật; trên màn hình nhỏ hơn mức đó tính theo điểm logic (ví dụ 1366x768 ở DPI 125% hoặc 150%) mức tối thiểu bị giới hạn theo vùng dùng được của màn hình. Cửa sổ nhỏ hơn mức sàn kế hoạch (800x400) hiện lỗi `Layout:` trên thanh trạng thái và giữ kế hoạch tốt gần nhất, không kẹp âm thầm.
- Dòng lệnh không thể bị mất bằng cách ẩn: vẫn được giữ chỗ, kèm thông báo `CommandInputKept`.
- Khi đang có lỗi vùng, đồng bộ kích thước panel và kẹp menu nổi không ghi lại layout.
- Khi dock phải nhường hoặc rail trái bị thu hẹp, ghi thông báo vào lịch sử CMD, và vào thanh trạng thái nếu đang không có lỗi.
- Đây là quy tắc cho kế hoạch vùng workbench, không phải nghiệm thu docking đầy đủ, persistence hay cổng M01-UI.

Read only the relevant reference:

- [Grid and viewports](references/grid-and-viewports.md): finite cell counts, colors, per-view labels, CAD Wireframe/isocurves, display menus and single-view/4V behavior. Includes the user-approved Wireframe update of 2026-10-08 and verified limits.
- [Command and Curve input](references/command-and-curve.md): transcript, pinned input, resizing, completion and supported Line/Polyline options.

Keep original Matrix command names and share one implementation across menu, mouse and CMD. Use repository `OM9_Milestones` for feature details; the user explicitly says these specs are sufficient, so do not reread PDFs. Follow `openmatrix9-workflow` for progress and `openmatrix9-build-validation` for native validation. This contract supplements the UI skill; it does not prove full core/Curve completion.

The user deferred choosing the middle-button mapping until checking Matrix directly. Do not change that default from an earlier answer or infer it from the screenshots.

After coding and verification, present any new business behavior and ask whether to save it to the skill. Explicit user authorization for that exact update is sufficient; do not ask again. Record technical evidence independently of business consent. Sync repository and installed skill copies after approved updates.
