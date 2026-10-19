# Rust trước và ownership OM9 v2

Đọc [đích milestone](milestones-target.md). Safe Rust sở hữu nghiệp vụ, document/model/IDs/generations, command/session/options, History/Undo/journal, UI state/scene/index/cache và workers. Giữ nguyên hợp đồng Matrix9 UI/feature; ưu tiên Rust không cho phép thay semantics hoặc scope nghiệm thu.

- Giữ/thử port Rust logic hiện có nếu host-free; native property/transaction/FCStd/Qt-Coin adapter của FreeCAD phải chuyển sang own OM9. Không mặc định dùng FreeCAD/Project/Shell của QymCAD.
- C++ chỉ bridge OCCT/openNURBS hoặc API native thực sự cần. Rust own math/data/UI/state khi phù hợp; operation adapter không sở hữu document, quyết định publish hay session. Ghi constraint, owner, ABI/error/lifetime/concurrency và closure cần thiết cho mỗi ngoại lệ.
- Python giữ tooling/audit/fixture khi cần; không workbench registration hoặc vòng lặp hình học production. Old FCMacro/Rhino tests là nguồn replay intent, không tự phải viết lại tất cả tool thành Rust.
- Owned Vec/String/typed structs, checked allocation/indexing, snapshot độc lập và identity/generation/version. Unsafe/FFI nhỏ; không unwinding qua ABI, allocation/free cùng phía. Safe Rust caller không tự chứng nhận native dependencies memory-safe.
- Worker chỉ dùng dữ liệu bất biến/owned hoặc handle kernel đã chứng minh concurrency; không mutable UI/document pointers. Publish có source/context/permission guards, global atomic rollback và old Redo protection. Worker60% là budget, không cam kết phần trăm CPU/RAM.
- Affine Gumball drag dùng display cache/matrix, cheap guards trên pointer path; exact geometry validation/commit ở publication. Geometry-changing previews có budget riêng. Đo runtime mới, không tuyên bố nhanh từ kiến trúc.

Ví dụ CV edit: Rust validate basis/state/owned inputs, native operation adapter chỉ tạo/kiểm curve khi cần, OM9 journal publish một transaction; UI giữ Matrix9 interaction. Chuyển từng capability và replay geometry/Undo/Cancel/.om9/3DM/UI trên app mới có fingerprint, không dùng số file Rust làm completion.
