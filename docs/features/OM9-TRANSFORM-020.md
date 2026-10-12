# OM9-TRANSFORM-020 — Cage Edit

<!-- rhino-core-cage-borrowed-admission-20261011:start -->
Basis Cage và saved binding được Rust kiểm trên dữ liệu mượn trước copy/input
buffer; native selection và recompute có lỗi typed kèm operation/object. Giữ
solver, imported basis, Undo/Redo và Release liên kết hỏng. 1322 checks/24
actual host exits0 trên cùng plugin; full Rust734 đạt. Results/maps, History,
native allocation khác và yêu cầu gốc còn mở; **RCORE-01 đang triển khai**.
[Bằng chứng và giới hạn](../validation/2026-10-11-rhino-core-cage-borrowed-admission.md).
<!-- rhino-core-cage-borrowed-admission-20261011:end -->

<!-- rhino-core-native-cage-policy:start -->
Cage hiện có dùng policy Rust chung cho declaration/capture/commit và request
độc lập cho API/recompute; Part/Mesh được kiểm trước copy/Topology extraction.
Giữ Create bằng bounds, Restore archive và Release binding hiện có. 1268 checks/
23 actual host exits0 trên cùng plugin; full Rust730 đạt. Rational-control/
binding allocation, results/maps, các consumer khác và yêu cầu gốc còn mở;
**RCORE-01 đang triển khai**. [Bằng chứng và giới hạn](../validation/2026-10-11-rhino-core-native-cage-policy.md).
<!-- rhino-core-native-cage-policy:end -->

Source: Spec v1 `specs/05-transform/om9-transform-020-cage-edit.md`.

Implemented slice: select whole captives, then one existing native 3D cage;
Global/Local and finite nonnegative falloff create persistent native bindings.
The binding updates independently of global History Record/Update. An explicit
Restore choice routes retained 3DM records through the verified restore service.
Line/surface controls and Accurate/Fast/refit modes remain unsupported.

The [native Cage command contract](cage-command-native-contract.md) records
stable routes, input roles, ownership, stale checks and limits. Rust tests pass;
the matching SDK command fixture passes 27/27 checks.
Status: partially_implemented; validated_supported_slice.
