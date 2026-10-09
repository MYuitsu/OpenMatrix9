# Mẫu yêu cầu làm việc

```text
Triển khai supported slice OM9-GEM-001.
Đọc AGENTS.md, spec exact ID và CODE_GUIDE.md.
Chốt options/units/limits; đánh dấu chi tiết chưa xác định.
Rust quản lý state, C++/Qt resolve inputs và tạo geometry native.
Thêm fixture exact ID; kiểm tra preview/cancel, Undo/Redo và save/reload.
Chỉ cập nhật trạng thái dựa trên evidence native.
```

```text
Kiểm tra code mẫu của OM9-SURFACE-003.
Mở spec và module tương ứng trong examples/rust/src/features.rs.
Chạy cargo test của thư viện mẫu. Phân biệt validator đã qua với solver chưa triển khai.
```

```text
Đánh giá hành vi còn thiếu của OM9-CURVE-003.
So sánh hợp đồng với implementation notes, ghi options chưa hỗ trợ,
và chọn fixture native để nghiệm thu đúng phạm vi.
```
