# OM9-SURFACE-002 — Sweep 1 History

<!-- common-command-options-20261012:start -->
Command Options 2026-10-12: các tham số scalar/toggle/choice dùng model Rust
và binder CMD–Builder chung; native validation trước acceptance/preview,
giữ draft lỗi và các guards/transactions hiện có.
[API chung](../command-options-adapter.md) · [Native evidence](../validation/2026-10-12-command-line-options.md).
<!-- common-command-options-20261012:end -->


<!-- rhino-core-surface-result-20261011:start -->
Loft/Sweep giữ solver hiện có, có GeometryResult EditSurface với recipe,
source snapshots/attributes/revision và CAD master khớp Shape được công bố.
History cập nhật/xóa cả Shape và result; freeze/detach, Undo/Redo và FCStd đạt.
Sửa và kiểm Placement/group cùng tên output duy nhất trước publication.
Matching848/21 actual host exits0; full Rust791 đạt.
Global conversion/maps, các consumer và lifecycle toàn family còn mở;
**RCORE-01 đang triển khai**. [Bằng chứng và giới hạn](../validation/2026-10-11-rhino-core-surface-result.md).
<!-- rhino-core-surface-result-20261011:end -->

<!-- rhino-core-surface-history-admission-20261011:start -->
Surface History dùng Rust kiểm JSON/options/group metadata trước native capture;
giữ solver Loft/Sweep và kiểm rail closure từ nguồn hiện tại. Dữ liệu hỏng giữ
raw/source/links/Redo, xóa output cũ và báo InvalidBasis kèm tên child.
GREEN55; matching698/13 actual host exits0; full Rust767 đạt.
Full results/conversion/maps/ownership mọi family còn mở; **RCORE-01 đang triển khai**.
[Bằng chứng và giới hạn](../validation/2026-10-11-rhino-core-surface-history-admission.md).
<!-- rhino-core-surface-history-admission-20261011:end -->

CMD alias `gvSweepHistory` reuses the native Sweep1 input/options workflow and
forces associative output. Reference: Matrix 8 Book1 PDF183 / printed173.
The manual describes automatic rebuilding after rail/profile edits and Closed
Yes only for a closed rail with at least two profiles.

Native `OpenMatrix9Gui::SurfaceHistory` retains ordered curve subreferences,
options and parent-frame dependencies. Direct child Shape edits detach its
parents while descendants remain linked, according to the shared OM9-HISTORY-001
contract. Undo restores those parents. Invalid/deleted sources clear the result
and report an error. Source geometry is preserved; Preview, one-transaction
commit, Cancel, Undo/Redo and FCStd restore share Sweep1 behavior.

Closed defaults Yes only within the supported closed-rail/two-profile case.
Reverse/Flip is per input; the remaining geometric bounds match
[Sweep1](OM9-SURFACE-001.md). Full Matrix History/global controls and dedicated
History menu/F6 placement remain unverified. This feature remains partial.

[Shared advanced contract](surface-advanced-options.md) and
[native validation](../validation/2026-10-09-surface-constraints-history.md).
