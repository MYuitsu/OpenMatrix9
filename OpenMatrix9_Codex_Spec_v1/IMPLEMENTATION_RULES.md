# Quy tắc triển khai và đồng bộ

Hợp đồng thiết kế nêu hành vi mục tiêu. Implementation notes ghi phạm vi native
đã có bằng chứng. Code mẫu minh họa request validation/plan; các vai trò, kiểu
và số lượng của mẫu không chứng minh tương thích geometry đầy đủ.

Trước khi triển khai, chốt input types, units, ranges, defaults, output topology,
preview/commit/cancel, dependency/History, failure, Undo/Redo và save/reload.
Chi tiết chưa xác định cần quyết định OpenMatrix9 hoặc để chưa hỗ trợ; không
tự biến một ví dụ screenshot hay tên option thành giá trị mặc định.

Đối chiếu [specs/00-rhino-core](specs/00-rhino-core/README.md) và ghi các
RCORE áp dụng trước khi triển khai feature phụ thuộc dịch vụ CAD nền. Nếu
hành vi nguồn khác supported slice đang có, ghi discrepancy và fixture cần
kiểm tra; không sửa mô tả hoặc đánh dấu tương thích khi chưa có evidence.

Phân biệt FreeCAD có UI/API, có primitive một phần, chưa thấy đường tương
đương trong phạm vi rà và chưa kiểm chứng. Đọc source/symbol được dẫn và
baseline hash trước khi tái sử dụng; kiểm tra module có trong bản build thực
tế. Thay đổi source cần rà lại kết luận khả năng, không chỉ cập nhật hash.

Kiểm thử geometry/state theo exact ID và tolerance đã công bố. Lỗi native phải
abort transaction; input/revision thay đổi cần validate lại. Command không có
adapter hoặc option còn thiếu phải disabled/unsupported có lý do cụ thể.

Sau khi có evidence, cập nhật spec, FEATURES.json, FEATURES.yaml,
IMPLEMENTATION_STATUS.md và ledger triển khai. `partially_implemented` cùng
`validated_supported_slice` xác nhận đúng supported slice đã kiểm chứng;
không xác nhận mọi options. Không thay evidence native bằng kết quả validator.
Checklist chung chỉ hoàn tất khi có bằng chứng hoặc lý do không áp dụng.

MANIFEST.json ghi hash của tài liệu/hướng dẫn và mã mẫu được tạo. Không đưa
build artifacts hay đầu vào riêng vào manifest của gói hướng dẫn. Giữ nguyên
file đầu vào/thông báo giấy phép; không công bố hoặc push khi chưa được yêu cầu.
