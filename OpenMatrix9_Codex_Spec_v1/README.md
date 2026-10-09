# OpenMatrix9 — hợp đồng và hướng dẫn triển khai

607 feature spec, 16 nhóm nghiệp vụ và 33 module; thêm nhóm nền tảng
[`specs/00-rhino-core`](specs/00-rhino-core/README.md). Mỗi feature spec có hợp đồng thiết kế bằng tiếng
Việt, vai trò đầu vào, tham số, thứ tự xử lý, code Rust riêng và ca nghiệm thu.
Các đoạn hướng dẫn được viết lại theo thiết kế OpenMatrix9; không chứa listing
decompile, assembly hoặc đoạn văn sao chép từ manual. Nhóm `00` dẫn các file
source/API đã đối chiếu để kiểm tra kết luận về khả năng FreeCAD.

Đọc [INDEX.md](INDEX.md), chọn exact feature ID rồi mở đúng spec. Tra cứu nhanh
qua [FEATURE_LOOKUP.json](FEATURE_LOOKUP.json); chỉ truy vấn mục cần làm thay vì
nạp cả catalog. [FEATURES.json](FEATURES.json) và bản YAML chứa metadata đầy đủ. Chỉ đọc
[CODE_GUIDE.md](CODE_GUIDE.md) và phần liên quan của
[ENGINEERING_CONTRACTS.md](ENGINEERING_CONTRACTS.md) khi cần ranh giới host/solver.

[Nhóm 00 - nền tảng Rhino và khả năng FreeCAD](specs/00-rhino-core/README.md)
gồm 12 đặc tả chi tiết. Mỗi chương ghi hành vi Rhino cần có, FreeCAD đã có ở
mức UI/API nào, source/symbol làm bằng chứng, phần OM9 cần tích hợp, lifecycle
và ca nghiệm thu. “FreeCAD có API” không đồng nghĩa “OM9 đã hỗ trợ”.
Nhóm nền tảng không tăng số 607 feature hoặc nâng trạng thái native.
[Chi tiết giao diện từ code decompile](specs/00-rhino-core/UI_DETAILS.md)
bổ sung F6, panel/pick, Shade/material, View Manager và Layout. Hành vi được
viết trực tiếp trong 12 chương và tám feature UI; phần decompile không kèm
bảng tham chiếu code. [Phạm vi dữ liệu](specs/00-rhino-core/DECOMPILE_AUDIT.md)
ghi các dependency còn thiếu và nhánh chưa được runtime kiểm chứng.
[Báo cáo đối chiếu ban đầu](../docs/reviews/2026-10-09-rhino-core-gap-audit.md)
giữ nguồn và các phát hiện; file RHINO_CORE_REQUIREMENTS cũ là trang chuyển hướng.

Code mẫu dùng [thư viện Rust độc lập](examples/rust/README.md), đã biên dịch và
qua kiểm thử validator trên toàn bộ 607 contract. Mẫu tạo operation plan, còn
adapter và geometry solver phải triển khai theo từng feature. Trạng thái native
được giữ riêng trong [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md).
Biên dịch mẫu không làm một feature trở thành lệnh hoạt động trong FreeCAD.

Các giá trị hoặc tương tác chưa xác định được ghi rõ; không tự suy ra default,
tolerance, topology, lịch sử phụ thuộc hoặc tính tương thích đầy đủ. Required và
cardinality trong mẫu là chính sách minh họa, cần điều chỉnh theo supported slice.

[GUIDANCE_VALIDATION.md](GUIDANCE_VALIDATION.md) ghi phạm vi kiểm tra của lần cập
nhật. File đầu vào và thông báo giấy phép được giữ nguyên. Bỏ dẫn nguồn và viết
lại hướng dẫn không tự xác nhận quyền công bố toàn bộ cây thư mục.
