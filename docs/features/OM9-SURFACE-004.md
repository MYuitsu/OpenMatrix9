# OM9-SURFACE-004 — Sweep 2 History

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

CMD alias `gvSweep2History` reuses the native Sweep2 input/options workflow and
forces associative output. Reference: Matrix 8 Book1 PDF186 / printed176.
The manual describes automatic rebuilding after edits and Closed/Flip/
Maintain Height options. Native output preserves two ordered rails, sections,
all component subreferences, options and parent frames.

Closed defaults Yes for closed rails with at least two profiles; open-rail
closure is unsupported. Maintain Height defaults Yes with one profile; the
multiple-profile host strategy cannot implement it. Reverse/Flip is per input.
Other geometric bounds match [Sweep2](OM9-SURFACE-003.md).

The native dependency graph handles source/parent changes, suspend/resume,
invalid/deleted inputs, direct child edit detachment, native Undo and fresh-process
FCStd reconstruction. These lifecycle rules share the bounded
[advanced contract](surface-advanced-options.md). Dedicated History menu/F6
placement, global Record/Update controls and full Matrix parity remain unverified.
This feature remains partial. See [native validation](../validation/2026-10-09-surface-constraints-history.md).
