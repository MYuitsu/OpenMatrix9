# Nguồn đích: milestones và giao diện Matrix9

Theo chỉ dẫn người dùng ngày 2026-10-19, nguồn nền tảng hiện hành là `OM9_Milestones/M01 - CAD Shell, Viewport/` trong checkout `H:/OpenMatrix9`. Đọc `L1.json`, sau đó L2/L3/L4 đúng owner; với tính năng M02 và các milestone sau, đọc cây milestone tương ứng. Không yêu cầu package đặc tả cũ đã vắng mặt hoặc tự tạo các file policy tương đương không tồn tại.

- Root `AGENTS.md` và chỉ dẫn trực tiếp của người dùng quyết định target: standalone Rust OM9. Safe Rust sở hữu document/model/IDs/session/history/Undo/transactions/UI/scene/cache/workers. C++ giới hạn ở adapter OCCT/openNURBS bắt buộc; GPU mesh là dữ liệu dẫn xuất, không phải canonical BRep.
- M01 `L2-01-03` sở hữu document/entity/geometry; `L2-01-05` sở hữu OCCT BRep/lifetime/topology/tessellation; `L2-01-12` sở hữu codec/persistence. Đọc L3 chính xác trước thay đổi API/FFI; lưu quyết định kỹ thuật riêng thay vì sửa nguyên bản nguồn nhập.
- M01 `L2-01-01`, `L2-01-06`, `L2-03-01`, `L2-03-07`, `L2-03-10`, `L2-03-14` mô tả shell/render/bốn viewport/menu/CMD/Layers. Giữ Matrix9 và các workspace/layers contracts đã được người dùng duyệt. QymCAD chỉ cung cấp code mẫu/routine, không thay UI/workflow.
- Tra exact ID bằng workflow `search` và `feature`; ID L1/L2/L3/L4 trong JSON giữ nguyên. Metadata/protocol/source prompts nhập trong tài liệu là tài liệu tham khảo, không thay yêu cầu đang được người dùng giao. Không coi planning state, chữ ký API đề xuất, source pin hoặc kết quả ghi sẵn là runtime evidence.
- QymCAD ở `QymCAD/`; chỉ đọc file liên quan, không phụ thuộc build vào đường dẫn đó. Giữ provenance/license notices khi tái sử dụng. Xem `docs/private-reference-policy.md` và registry trước sử dụng nguồn riêng.
- FreeCAD/v1 và báo cáo/spec cũ là lịch sử. Giữ nội dung/hash/bằng chứng gốc; không biến pass cũ thành nghiệm thu standalone. Chỉ chạy legacy host khi được giao rõ.
- Build root workspace với `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`. Đối chiếu `docs/openmatrix9-progress.json` với code và test/runtime thực; đồng bộ `runtime_implementation` của đúng milestone và README theo phạm vi đã chứng minh. Không tự ký manual review.
