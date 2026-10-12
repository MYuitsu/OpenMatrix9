# OM9-SURFACE-003 — Sweep 2

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

Menu ID: `OM9_SurfaceSweepSweep2Rails`. CMD alias: `Sweep2`.
Verified supported slice: 2026-10-09; full feature remains partially implemented.

## Reference and implementation contract

Matrix 8 Book 1 PDF pp.184–186 (printed pp.174–176) were read through the
Sweep 2 continuation, including the Maintain Height illustration.
Shares [Sweep 1](OM9-SURFACE-001.md)'s ordered selection, Chain Edges,
Reverse/seams, Automatic/Natural, section Rebuild, Preview and lifecycle contract,
with two distinct rails before profiles. Links retain every rail component and
section, including parent world placement.

The matching SDK's ContactOnBorder path fails on minimal fixtures and is not
used. These strategies and bounds are explicit OM9 decisions:

- One profile must intersect both rail starts; rails share closure state.
  Sample 65 equivalent arc-length stations on each rail; Rust computes frames
  using rail separation and primary rail tangent. Width scales with separation.
  Maintain Height ON preserves the section normal coordinate; OFF scales it
  with width. Initial host choice is OFF. Tangent coordinate stays unchanged.
  Native loft joins these transported copies, degree at most three. Closed
  rails omit the duplicate final station and use a closed loft.
  Dense curved-section PipeShell produced invalid geometry on this SDK;
  lofting the same sections passed contact and crown-height checks.
- Multiple profiles use the native auxiliary-spine Contact mode with original
  profiles. Maintain Height is disabled for this case and rejected by the
  adapter if requested. Unsupported correspondence fails explicitly.
- Closed Sweep requires closed rails and at least two sections. A two-profile
  closed annulus and persistence are verified.
- Check 65 samples on each rail/section against the resulting surface, with
  tolerance `max(1e-4 mm, curveLength * 1e-7)`. Crossing/coincident frames and
  unmatched contact fail. Sampling is not certified whole-curve contact.
- Output is an uncapped native surface/shell. Settings include
  `maintainHeight`, fitting, closure, directions/seams and Preview.
  Originals and source links remain intact.

## Advanced options

Independent rail G1/G2 constraints use unambiguous support-face edges and matching
open nonrational sections. Native geometry checks original contact, normals and
curvature. Refit supplies a finite cross-section tolerance with native hull bounds.
Add Slash controls actual section correspondence for one profile on open rails,
using paired viewport picks or `AddSlash=a,b`. History ON makes results
associative; CMD `gvSweep2History` forces OM9-SURFACE-004. See the precise
[bounds, defaults and lifecycle](surface-advanced-options.md).

## Remaining options

General multiple-profile Maintain Height, Preserve First/Last Shape constraints,
Simple Sweep, Point endpoints and dragged seams remain unsupported. Multiple-
profile/closed-rail Slash and fitting/closed/height/Slash combinations with G1/G2
remain unsupported. Full compatibility remains unverified.

## Validation

An arch between rails growing from width 5 to 9 mm retains height 2 mm with
Maintain Height ON and becomes 3.6 mm OFF. Both rails keep contact.
Existing one/multiple-profile strip tests, closed annulus area, manual Preview,
Undo/Redo and FCStd options/source links/placements pass.
See [the validation record](../validation/2026-10-09-surface-options.md).
Advanced continuation: [constraints, Refit, Slash and History validation](../validation/2026-10-09-surface-constraints-history.md).
