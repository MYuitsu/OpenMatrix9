# Quyết định license sản phẩm OpenMatrix9

Ngày: 2026-10-18 · Người ký: chủ dự án · L4: `OM9-M00-L4-06-01-04`

## Quyết định

Sản phẩm phát hành theo **LGPL-2.1-or-later**, khớp với `license.workspace` đang khai trong `Cargo.toml`
của cả sáu crate thành viên.

## Căn cứ

- Toàn bộ dependency Rust trong cây là MIT/Apache/BSD/Zlib; **không có GPL/AGPL** (audit 345 package).
- OCCT là LGPL-2.1 + exception, link động — tương thích và đã có nghĩa vụ công bố phần sửa `TKG3d`.
- openNURBS có license riêng cho phép dùng, giữ notice.
- QymCAD AGPL không lan vì không link/copy (xem `REUSE-MODEL-DECISION.md`).

## Nghĩa vụ khi phát hành

1. Kèm `THIRD-PARTY-NOTICES-M00.md` (OCCT modified component + openNURBS + QymCAD reference-only).
2. Công bố source `TKG3d` đã sửa: upstream archive digest + patch + build script.
3. Cho phép thay thế DLL (ship động).
