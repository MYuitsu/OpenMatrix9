> **Current target: specs v2 (2026-10-13).** [OM9-TRANSFORM-041](../../OpenMatrix9_Codex_Spec_v2/specs/05-transform/om9-transform-041-create-cage.md)
> Use standalone OM9 and Matrix9 UI contract for new work. The record below is legacy FreeCAD/source evidence; its v1 links, native passes and host limitations do not certify the new runtime.

# OM9-TRANSFORM-041 — Create Cage

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

Source: Spec v1 `specs/05-transform/om9-transform-041-create-cage.md`.

Implemented slice: whole native geometry defines a World BoundingBox with
positive extents; explicit OM9 2/2/2 counts and 1/1/1 degrees create one
independent editable native CageControl. Creating a cage does not capture its
bounding-box sources. Other coordinate/base modes remain unsupported.

The [native Cage command contract](cage-command-native-contract.md) records
stable routes, input roles, defaults, ownership and limits. Rust tests pass;
the matching SDK command fixture passes 27/27 checks.
Status: partially_implemented; validated_supported_slice.
