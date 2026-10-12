# OM9-SOLID-001 — BooleanDifference

<!-- rhino-core-edit-result-20261011:start -->
Rust admission và native CAD Edit GeometryResult đã triển khai trên solver sẵn có.
Runtime14/15 đã đạt các hồi quy CAD/lệnh/guard và cold reload; callback abort còn thất bại 2/5 kiểm tra Redo.
Runtime15: Dimension NaN/Infinity 10/10, callback font/đồ họa và lỗi đúng Explode/source 12/12; dùng lại kiểm finite Rust.
Full Rust 808 đạt; bằng chứng native giữ riêng theo plugin, không suy thành nghiệm thu toàn chương.
**RCORE-01 đang triển khai**, không đóng consumer/lifecycle hay yêu cầu gốc.
[Bằng chứng, lỗi đang mở và bước tiếp](../validation/2026-10-11-rhino-core-edit-result.md).
<!-- rhino-core-edit-result-20261011:end -->

Source engineering contract: `specs/04-solid/om9-solid-001-boolean-difference.md`. Matrix 8 Book 1 PDF pp.223
re-read through continuation to the next command heading on 2026-10-09.
Original PDF remains an ignored reference, excluded from runtime resources.

Implemented slice: Ordered target/cutter sets for solid BRep, surface-area BRep or closed Mesh via polyhedral OCCT; target replacement and cutter DeleteInput, cavities and persistence. Mixed categories and invalid/empty results reject.

Invocation: stable menu/F6 `OM9_SolidDifference` and CMD `BooleanDifference`. The
[shared native Edit contract](edit-native-contract.md) distinguishes manual
facts from host options/defaults, geometry types, tolerance, lifecycle and limits.

Validation: options `build/edit_options_smoke-1/19b793add82d48de9e92ffcbf2c43343/results.json` — 57/57;
core `build/edit_commands_smoke-1/4ebfad5e7e844c999ca94dc0a4493dea/results.json` — 115/115. Fixtures use exact feature IDs.
Both FreeCAD processes exited 0. Matching SDK build passed; Rust 140 tests
and Python 23 tests passed. Current regression reports and audit limitations
are in [implementation ledger](../openmatrix9-progress.json).

Status: `partially_implemented`, `validated_supported_slice`. Source edits do
not recompute snapshot results. Live Matrix History, annotation/block/cage
adapters, document-tolerance compatibility and exhaustive topology remain open.
