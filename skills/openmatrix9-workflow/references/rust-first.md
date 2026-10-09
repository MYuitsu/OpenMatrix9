# Rust là ưu tiên số 1 trong OpenMatrix9

Quy tắc do người dùng yêu cầu: ưu tiên Rust để tăng an toàn bộ nhớ, sau đó mới dùng ngôn ngữ khác. Áp dụng khi chọn kiến trúc, viết mới và chuyển code OM9; không thay mục tiêu chức năng, tên lệnh hoặc phạm vi nghiệm thu.

1. **Safe Rust trước:** Rust sở hữu logic nghiệp vụ, command/editor state, dữ liệu độc lập, validation số/CV/weight/knot, thuật toán hình học có thể tách khỏi kernel, spatial index/cache, budget và worker orchestration. Code cũ bằng C++/Python không phải lý do để tiếp tục đặt logic mới ở đó. Đánh giá phương án Rust trước; tiện, quen hoặc nhanh viết hơn không đủ để bỏ Rust.
2. **C++ sau Rust, chỉ cho ràng buộc native:** giữ lớp cầu nối tối thiểu với FreeCAD document/transactions, Qt widgets/views, OCCT và openNURBS. Kernel/thư viện native không bị viết lại chỉ để đổi ngôn ngữ. C++ đọc metadata/topology và dựng/commit native shape; chuyển quyết định và dữ liệu độc lập sang Rust. UI model/state/validation thuộc Rust, widget adapter có thể vẫn là Qt/C++.
3. **Python và ngôn ngữ khác sau đó:** chỉ dùng cho bootstrap/API bắt buộc, cầu nối thật sự cần thiết, test ứng dụng và công cụ hỗ trợ khi Rust chưa phù hợp. Không thêm business logic hoặc vòng lặp hình học nặng vào Python vì tiện. Macro Rhino/FreeCAD và generator/audit hiện hữu được giữ để kiểm chứng, không tự động viết lại mọi tool hay SVG thành Rust.

## Ranh giới bộ nhớ và đa luồng

- Ưu tiên dữ liệu Rust sở hữu bằng Vec/String/typed structs, borrowed slices có thời hạn rõ và checked arithmetic trước allocation/indexing. Cô lập unsafe vào FFI module nhỏ; ghi điều kiện pointer/length/alignment/lifetime, lỗi, allocation/free cùng phía và concurrency. Không cho panic hoặc exception đi xuyên ABI; xác định và kiểm thử chính sách panic/error của build.
- Không cho Rust giữ raw pointer tới Qt widget/FreeCAD document/OCCT object đã hết lifetime. Dùng snapshot độc lập và identity/generation để loại stale requests. Việc đọc/mutate document và commit transaction chạy theo quy tắc GUI/native host; worker chỉ nhận dữ liệu độc lập bất biến hoặc bản copy kernel được chứng minh an toàn, không nhận mutable document.
- Safe Rust không tự chứng nhận unsafe, FFI, native dependencies hoặc toàn ứng dụng memory-safe. Kiểm ownership, bounds, cancellation và lifecycle ở cả hai phía; Rust unit tests không thay thế native/host/Rhino regression.

## Chuyển code và ngoại lệ

Chuyển từng phần với behavioral baseline, test Rust và replay native/ứng dụng trên source/runtime mới có hash. Giữ Undo/Redo/Cancel/current geometry/worker60% và ngưỡng CAD đã duyệt. Không dùng số file Rust làm tỷ lệ completion hay gọi cập nhật skill là đã chuyển code.

Khi cần dùng ngôn ngữ khác, ghi trong feature/design record: phần code, API/ràng buộc cụ thể khiến Rust chưa phù hợp, owner dữ liệu ở mỗi phía, lớp adapter tối thiểu, hướng chuyển Rust nếu khả thi và điều kiện bỏ fallback. Ngoại lệ thường trực do native API phải được phân biệt với fallback tạm thời. Không cần xin lại phép cho một bridge nằm trong công việc đã được người dùng cho phép.

Ví dụ: sửa curve CV — Rust validate basis và quản lý request sở hữu; Qt nhận trường số; adapter OCCT tạo và kiểm native curve; FreeCAD commit một transaction trên GUI thread. Không thêm validation nghiệp vụ tương tự trong Python hoặc C++ rồi gọi toàn pipeline memory-safe.
