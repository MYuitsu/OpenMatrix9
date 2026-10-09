# OM9-SOLID-014 — Sphere

Native Center/Diameter, 2Point, 3Point/Radius, 4Point/Radius, Vertical, FitPoints, AroundCurve and planar Tangent Sphere; exact curve references, explicit ambiguous branch selection, native mouse/CMD, preview and persistence. General nonplanar tangent solving and associative History remain unsupported.

[Contract, options and limits](../../OpenMatrix9_Codex_Spec_v1/specs/04-solid/om9-solid-014-sphere.md).

[Native validation](../validation/2026-10-09-solid-box-sphere-options.md).

Rust owns ordered state and construction math; C++/Qt owns native references, Part/OCCT geometry, scene preview and one transaction. Snapshot results preserve input geometry and persist through native Undo/Redo and FCStd reload.

## Earlier source evidence

Matrix 8 Book 1, printed pp.222–223 / PDF pp.232–233, was extracted and visually checked in the 2026-10-06 core implementation. The current normalized spec engineering contract is preserved; host solver choices and remaining compatibility limits are separated in its implementation notes. Original reference inputs remain unchanged.
