# Quyết định mô hình tái sử dụng QymCAD

Ngày: 2026-10-18 · Người ký: chủ dự án · L4: `OM9-M00-L4-01-03-03`

## Quyết định

**Composition** — QymCAD là *tài liệu tham chiếu*, không phải mã được link hay ship:

1. Không thêm QymCAD như dependency (không crate path, không git dependency).
2. Mọi thứ học từ QymCAD được viết lại thành code Rust của OM9, với test riêng của OM9.
3. Không copy file nguồn QymCAD vào repo; chỉ trích dẫn đường dẫn + SHA làm tham chiếu.
4. License QymCAD là AGPL-3.0-or-later; vì không link/copy nên nghĩa vụ AGPL không lan sang OM9.

## Vì sao không fork, không clean-room

- **Fork**: phải nuôi một bản QymCAD (AGPL) và merge upstream — không cần thiết vì OM9 có kiến trúc riêng
  (crate hoá theo năng lực, Rust-first).
- **Clean-room**: tốn kém và không cần, vì QymCAD công khai và chỉ dùng làm *tham chiếu hành vi*, không
  phải để che giấu nguồn.
- **Composition**: đúng với policy hiện hành (`AGENTS.md`: QymCAD chỉ là code samples/routines) và
  không tạo nghĩa vụ pháp lý mới.

## Hệ quả

- `QymCAD/` trong workspace là read-only reference; không build trong CI của OM9.
- Khi cần hành vi của QymCAD, ghi lại *hành vi* trong design note + test của OM9, không copy code.
