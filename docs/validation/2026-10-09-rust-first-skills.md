# Rust ưu tiên số 1 — kiểm chứng skill ngày2026-10-09

Người dùng yêu cầu cập nhật ưu tiên Rust, và xác nhận chuyển logic Phase2 hiện tại. Đã cập nhật7 skill OM9 trong17 bản hiện hữu tại source thử riêng, Mod/OpenMatrix9 và thư mục skill của Codex. Ba bản quy tắc chung [rust-first.md](../../skills/openmatrix9-workflow/references/rust-first.md) có cùng SHA-256. Không cài thêm những skill không có bản cài trước đó.

Safe Rust sở hữu portable logic, validation, state, dữ liệu, index/cache và worker. C++ chỉ giữ API native FreeCAD/Qt/OCCT/openNURBS; Python giữ bootstrap/API/test/tool khi có ràng buộc cụ thể. Quy tắc ghi rõ ownership, bounds/lifetime của FFI, snapshot bất biến cho worker, invalidation và giới hạn chứng nhận an toàn bộ nhớ của thư viện native.

Baseline tìm thấy5 ràng buộc chưa được ghi rõ trước sửa: chuyển CV portable, dữ liệu worker độc lập, unsafe/FFI và phạm vi an toàn, Python UI fallback, lý do ngoại lệ/migration. Forward kiểm tra5 tình huống lựa chọn dưới áp lực và7/7 router skill trực tiếp đều đạt. Kiểm tra frontmatter/link cho17 bản và3 reference đồng nhất đạt bằng helper verify-rust-first-skills.ps1. [Bằng chứng có hash](2026-10-09-rust-first-skills.json).

Đây là kiểm chứng policy. Code chuyển đổi phải qua các test Rust/native/FreeCAD/Rhino mới. Thiết kế và kế hoạch Phase2 đã được người dùng duyệt; migration đang thực hiện tại source om9-dev, runtime om9-phase2-rust-sdk riêng. Phase2 đã nghiệm thu trước đó giữ nguyên làm baseline.
