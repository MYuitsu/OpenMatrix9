# RCORE-01 — Snapshot admission and native history disposal

FreeCAD replay kiểm tổng history/property records và dung lượng snapshot/tên trước khi sao chép; dùng lại bảng transaction descriptions với borrowed view đồng bộ. Hủy Undo/Redo dùng FlagToggler, chặn reentrant mutation và pop record trước destructor; nhánh visible Redo của failed preserving group cũng được bảo vệ.
**App273/plugin275/kernel240:3372/3372 kiểm tra hành vi qua34 host;33/34 host sạch log.** Một host close-object có startup Unhandled std::exception trước fixture, nên runtime gate chưa đạt toàn bộ. Snapshot38, normal-close finalizer16 và empty-group Redo5 đạt; các lượt diagnostic278/280 không cộng vào tổng. Rust toàn bộ lần gần nhất1037/0, không chạy lại cho thay đổi native này.
**RCORE-01 vẫn đang triển khai:** startup deadlock, peak/opaque allocation, lifecycle và phần còn thiếu của hợp đồng gốc; các checklist lịch sử cần đối chiếu theo evidence, không tự coi mọi fixture đã đạt là chưa làm.

Constructor preflight runs before allocating Impl, transaction/name copies or history vectors. Integer subtraction bounds aggregate counts and logical byte charges; native owner histories use contiguous pointer snapshots. Existing transaction metadata queries avoid copied descriptions; borrowed names are not retained across callbacks or workers. This is logical storage accounting, not a certified allocator/peak limit.

Normal-close RED267 records a real sole-reference Python finalizer recursively clearing Undo, two finalizer entries and heap-corruption exit -1073740940. Existing FlagToggler and mutation controls now protect native history destruction; retired entries leave visible containers before delete. Actual close cases clear/remove/value/close-object pass4 checks each, without disabling their callback.

Independent review found visible Redo disposal outside the initial guard. Native RED272 reproduces a real detached object destructor increasing the transaction lock after a failed empty preserving group. The corrected visible/held guard gives GREEN5/5, with a usable follow-up transaction.

All34 fresh native hosts exit0 and loaded App/plugin/kernel hashes match;33 hosts pass the clean-log gate, while close-object274 has an unhandled startup resource-deadlock before fixture execution. Diagnostic278 reproduces the startup error; tracing280 passes cleanly and does not establish a fix. One earlier cold271 test was incorrectly dispatched before warm files existed; its FileNotFoundError remains preserved and excluded. Cold276 follows warm completion and passes62. Startup duplicate workbench registration warning remains unchanged.

Remaining:

- Startup resource-deadlock remains open: close-object274 and its278 reproduction report GUIApplication::notify Unhandled std::exception before the finalizer fixture starts, although all behavior checks and shutdown exit0. Diagnostic trace280 has a clean startup; it is not proof that the intermittent error is fixed. Preserve and collect a stack from an actually failing startup before any fix.
- The constructor now bounds aggregate owner-history/property records and logical snapshot/name storage before copies. Simultaneous journal/recovery/merge/inverse peak storage, opaque custom values/closures, allocator metadata and all native status/cache identities remain outside this proved bound.
- History destruction reentrancy is verified for regular native Undo/Redo cleanup and visible/held preserving-group disposal. Other derived-object/member destruction, global/source-participant callbacks and worker cancellation/close boundaries still need targeted lifecycle evidence; a single history fix does not certify every FreeCAD destructor.
- Continue the original representation/conversion/ownership/result-routing requirements with existing accepted fixtures. Historical unchecked rows are not automatically new failures: reconcile C01-C12/T01-T11 against their accepted slices, then fill missing families, maps, provenance, channels/attributes, enabled consumers and structural preservation.

Next: Capture the remaining startup deadlock at its native origin; reconcile original RCORE01 accepted slices, then reproduce remaining object/member lifecycle and aggregate preparation budgets using existing FreeCAD and Rust mechanisms.

[Machine evidence](2026-10-12-rhino-core-snapshot-history-disposal.json).
