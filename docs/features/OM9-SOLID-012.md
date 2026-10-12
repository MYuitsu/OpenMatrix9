# OM9-SOLID-012 — Box

<!-- rhino-core-extended-input:start -->
## Extended numeric input — validated slice

Extended typed point input through existing native geometry; no all-option or all-P0 completion. Legacy Solid mm/cm/in callback remains; actual accepted relative base and Undo are repaired.

[Runtime/source evidence](../validation/2026-10-09-rhino-core-extended-input.md). Native geometry is reused; live constraints, locale and other missing contracts remain open.

<!-- rhino-core-extended-input:end -->

Native Corners/Diagonal/3Point/Vertical/Center and oriented Cube Box; full dimensions, Enter defaults, signed extents, frozen CPlane, CMD/mouse, right-click shortcut, scene preview and persistence. Associative History/layer/Styles integration remains unverified.

[Contract, options and limits](../../OpenMatrix9_Codex_Spec_v1/specs/04-solid/om9-solid-012-box.md).

[Native validation](../validation/2026-10-09-solid-box-sphere-options.md).

Rust owns ordered state and construction math; C++/Qt owns native references, Part/OCCT geometry, scene preview and one transaction. Snapshot results preserve input geometry and persist through native Undo/Redo and FCStd reload.

## Earlier source evidence

Matrix 8 Book 1, printed pp.222–223 / PDF pp.232–233, was extracted and visually checked in the 2026-10-06 core implementation. The current normalized spec engineering contract is preserved; host solver choices and remaining compatibility limits are separated in its implementation notes. Original reference inputs remain unchanged.
