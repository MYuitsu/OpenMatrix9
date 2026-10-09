# Quy tắc đọc và triển khai spec OpenMatrix9

1. Tra exact ID trong INDEX/FEATURE_LOOKUP, mở đúng spec; không tải cả 607 spec hay metadata catalog cho một lệnh.
2. Đọc hợp đồng, bảng tham số và supported slice trong implementation notes.
3. Dùng CODE_GUIDE cho luồng Rust/C++/Qt/Python và ENGINEERING_CONTRACTS cho điều kiện chung liên quan.
4. Đọc code mẫu như thư viện request-planner độc lập; solver/adapter thực tế vẫn cần triển khai.

- Giữ nguyên feature ID, command aliases và thông báo giấy phép áp dụng.
- Rust quản lý behavior/state, C++/Qt tích hợp native, Python đăng ký workbench.
- Required/cardinality của ví dụ là quyết định mẫu; không gọi đó là default tương thích.
- Không đoán thông tin ghi chưa xác định. Ghi quyết định OM9 rõ ràng và nghiệm thu riêng.
- Mẫu kiểm tra kiểu/hữu hạn; host kiểm tra enum, miền giá trị, units, geometry và topology.
- Giữ command chưa hỗ trợ hiển thị nhưng disabled; không bật do có icon hoặc sample.
- Preview tách khỏi document; commit nguyên tử, hủy không để geometry/undo entry.
- Kiểm tra world placement, revision, Undo/Redo, save/reload và exact ID.
- Không đưa raw listing, địa chỉ native, bitmap thương mại hoặc đoạn văn chép vào spec.
- Sau kiểm chứng native, đồng bộ supported scope trong spec, JSON/YAML và IMPLEMENTATION_STATUS.
- Không đánh dấu native hoàn tất chỉ vì code mẫu biên dịch. Giữ checklist thiếu evidence chưa hoàn tất.

Theo IMPLEMENTATION_RULES để cập nhật bằng chứng và manifest. Các file gốc nằm
ngoài luồng đọc thường xuyên; bảo toàn chúng và thông báo giấy phép.
