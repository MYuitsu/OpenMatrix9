# OM9-SOLID-014 — Sphere

<!-- common-command-mnemonics-20261012:start -->
2026-10-12: quy tắc gạch chân một ký tự áp dụng qua engine Rust chung cho toàn
bộ schema Command Options hiện có: Line/Polyline, InterpCrv, Rectangle, Circle,
Ellipse, Rebuild, Box/Sphere, Sweep1/History, Sweep2/History và Loft. Tùy chọn con
khai báo alias trong phase cha; tên đầy đủ vẫn click được. Alias trùng hoặc
disabled không được quảng cáo bằng gạch chân. Lệnh mới đăng ký schema để dùng
cùng resolver, renderer và binding CMD–Builder. Trạng thái toàn feature còn partial.
[Bằng chứng native](../validation/2026-10-12-common-command-mnemonics.md).
<!-- common-command-mnemonics-20261012:end -->

<!-- rhino-core-extended-input:start -->
## Extended numeric input — validated slice

Extended typed point input through existing native geometry; no all-option or all-P0 completion. Legacy Solid mm/cm/in callback remains; actual accepted relative base and Undo are repaired.

[Runtime/source evidence](../validation/2026-10-09-rhino-core-extended-input.md). Native geometry is reused; live constraints, locale and other missing contracts remain open.

<!-- rhino-core-extended-input:end -->

Native Center/Diameter, 2Point, 3Point/Radius, 4Point/Radius, Vertical, FitPoints, AroundCurve and planar Tangent Sphere; exact curve references, explicit ambiguous branch selection, native mouse/CMD, preview and persistence. General nonplanar tangent solving and associative History remain unsupported.

[Contract, options and limits](../../OpenMatrix9_Codex_Spec_v1/specs/04-solid/om9-solid-014-sphere.md).

[Native validation](../validation/2026-10-09-solid-box-sphere-options.md).

Rust owns ordered state and construction math; C++/Qt owns native references, Part/OCCT geometry, scene preview and one transaction. Snapshot results preserve input geometry and persist through native Undo/Redo and FCStd reload.

## Earlier source evidence

Matrix 8 Book 1, printed pp.222–223 / PDF pp.232–233, was extracted and visually checked in the 2026-10-06 core implementation. The current normalized spec engineering contract is preserved; host solver choices and remaining compatibility limits are separated in its implementation notes. Original reference inputs remain unchanged.
