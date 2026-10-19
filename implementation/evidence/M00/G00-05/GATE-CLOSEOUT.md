# G00-05 — Associative history closeout

| L4 | Trạng thái | Căn cứ |
| --- | --- | --- |
| L4-05-01-01 | EVIDENCED | `QymCAD/crates/qymcad-kernel/src/naming.rs` — 4 quy tắc đặt tên (mảnh của face bị cắt, seam, không có hàng xóm thì theo vị trí, cạnh theo cặp face) + `NameMint` theo document ⇒ tất định |
| L4-05-01-02 | EVIDENCED | `QymCAD/crates/qymcad-core/src/refs.rs` — `Ref { … }`, `RefError`; `drivers.rs` — `DriverRef` |
| L4-05-01-03 | EVIDENCED | bảng mapping ở api.md §2 |
| L4-05-01-04 | OPEN | baseline test tách/gộp topology chưa có |
| L4-05-02-01 | EVIDENCED | `history_ops` + `history_replay` chạy: OK (test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s; test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s) |
| L4-05-02-02 | OPEN | conflict UI và rollback lệnh cần UI thật (M04) |
| L4-05-02-03 | OPEN | persistence feature id + cache chưa test |
| L4-05-02-04 | EVIDENCED | quyết định hỗ trợ history MVP + fallback ở api.md §3 |

