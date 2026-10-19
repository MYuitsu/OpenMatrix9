# G00-01 — QymCAD closeout

| L4 | Trạng thái | Căn cứ |
| --- | --- | --- |
| L4-01-01-01 | EVIDENCED | `git -C QymCAD rev-parse HEAD` = `746a17f1…` (pin khớp), record v0.13 PASS + APPROVED |
| L4-01-01-02 | OPEN | cần checkout độc lập + provenance manifest (audit cũ BLOCKED_DEPENDENCY) |
| L4-01-01-03 | OPEN | cần đọc quy định agent/đối chiếu bản quyền trên checkout độc lập |
| L4-01-01-04 | OPEN | cần chốt manifest source + provenance |
| L4-01-02-01 | OPEN | cần sơ đồ workspace 19 crate (audit cũ BLOCKED) |
| L4-01-02-02 | OPEN | cần khảo sát core/kernel/io/render/pick |
| L4-01-02-03 | OPEN | cần bảng phụ thuộc + license của từng crate QymCAD |
| L4-01-02-04 | OPEN | cần kết luận tái sử dụng theo crate |
| L4-01-03-01 | OPEN | cần so sánh 3 phương án kèm chi phí |
| L4-01-03-02 | OPEN | cần thiết kế biên giới license |
| L4-01-03-03 | EVIDENCED | `design/M00/G00-01/REUSE-MODEL-DECISION.md` — chọn **composition**, record PASS + APPROVED |
| L4-01-03-04 | OPEN | cần review biên giới license sau khi có 01–03 |
