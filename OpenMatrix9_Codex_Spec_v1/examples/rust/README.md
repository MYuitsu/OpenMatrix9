# Thư viện mẫu Rust cho 607 spec

`om9_spec_examples` là thư viện ví dụ độc lập, không liên kết vào workbench.
Mỗi module trong `src/features.rs` tương ứng exact feature ID và cùng code block
được trình bày trong feature spec. Request validator tạo plan bất biến; không
tạo geometry, không gọi API FreeCAD và không thay đổi capability của command.

Chạy từ thư mục OpenMatrix9:

```text
cargo test --manifest-path OpenMatrix9_Codex_Spec_v1/examples/rust/Cargo.toml
```

Thêm thư viện vào một project thử nghiệm:

```toml
[dependencies]
om9_spec_examples = { path = "PATH/TO/OpenMatrix9_Codex_Spec_v1/examples/rust" }
```

Thay đường dẫn cho đúng máy. Không coi `path` này là dependency đã được gắn vào
production Rust crate. Đọc [CODE_GUIDE.md](../../CODE_GUIDE.md) để hiểu ranh giới
validator/solver, đơn vị, enum choices, revision và host transaction.
