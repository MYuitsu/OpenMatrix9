# CAD Wireframe implementation plan

**Goal:** Render Rhino-style CAD boundaries and trimmed surface isocurves in OM9, without tessellation diagonals on BReps.
**Architecture:** A view-local Coin state selects Wireframe/Shaded; transient provider wrappers suppress CAD faces only in Wireframe. Native CAD edge/vertex nodes retain picking. Surface isocurves are display-only, trimmed in UV against face boundaries; source archives and CAD shapes are unchanged. Meshes retain polygon wire display.
**Spec:** OM9-VIEWPORT-001; user screenshots and request dated 2026-10-08; existing workspace contract.
**Tech:** C++/Coin/OCCT, Python import attributes, Qt/FreeCAD runtime.

Constraints: Development checkout only; pinned openNURBS unchanged; serial builds/tests; separate preview runtime; no commit, push or main integration; do not terminate user applications.

Review focus: mixed viewport state, native picking, links/affine previews, trim holes/seams, provider rebuild/deletion and workbench exit.

- [x] Add real FreeCAD regression that fails on triangulation diagonals, checks face/edge picking, mixed views and immutable geometry/source.
- [x] Implement Gui/CadPresentation.{h,cpp}; adapt CoreWorkspace; add matching SDK Part import library for native shapes. Cache transient isocurves by shape identity and density; invalidate on edits; remove adapters on workbench exit.
- [x] Bind source line colors and source isocurve density in Python import; retain density inside FCStd. Report bounded rendering limitations explicitly.
- [x] Build a separate runtime, run new test plus viewport lifecycle/menu regressions serially, inspect user ring screenshots.
- [x] Record source/binary hashes and evidence in validation, progress, viewport spec and README; open an interactive ring preview for user.

Skill changes require separate business-rule consent after verified behavior. This implementation is already authorized by the user.
