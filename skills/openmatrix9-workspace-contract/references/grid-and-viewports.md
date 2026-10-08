# Grid and native viewports

## Finite construction grid

The grid has **8 by 8 major cells**, each divided into **5 by 5 minor cells**, giving **40 by 40 minor cells**. Current OM9 spacing is 1 mm minor and 5 mm major; the construction-plane extent is -20 to +20 mm on each axis. This is a bounded world grid, not an infinite screen overlay. Pan/zoom changes its screen position and scale; rotation follows the native camera.

Keep the grid unpickable, outside document geometry and excluded from model fit bounds. Viewport canvases are black (#000000); major lines are stronger than minor lines. Command/active UI uses Matrix green (#82B48C), with black text. New Curve edges use green (#008255). Native four-view rendering and geometry stay in FreeCAD.

Reference: (Ảnh tham chiếu riêng không phân phối trong source public.).

## Viewport titles and layout

Each native view has a small top-left label and a separate small dropdown arrow. Keep the original names: **Looking Down**, **Perspective**, **Side View**, **Through Finger**. Active label uses Matrix green; inactive labels use gray.

Clicking a label activates its own view. Double-clicking the label shows only that view; double-clicking again restores four views (4V). Dropdown **Maximize / Restore 4V** calls the same action. Tabs switch the active single view while maximized.

Preserve camera pose/zoom, geometry, selection, document Undo and an active point command while toggling. Native redraw may update automatic near/far clipping; this does not authorize resetting the camera. Restore native title chrome and display settings on workbench exit. Rebuild one title/menu per replaced view; respect active-document and native task restrictions.

Reference: (Ảnh tham chiếu riêng không phân phối trong source public.).

## Display modes and implementation limits

Keep the original display-mode names from the screenshot and mark the current mode. **Wireframe** and **Shaded** have basic native adapters for the selected viewport; preserve the neighbor's mode. Reconcile local menu marks after native global Wireframe/Shaded changes. Other Matrix rendering modes and submenu operations remain visibly disabled until implemented and verified.

User-approved CAD Wireframe contract (2026-10-08): for BReps, show native CAD boundaries and trimmed surface isocurves; suppress render-mesh triangle diagonals. Separate meshes retain polygon wire lines. Shaded fills native surfaces. Keep each viewport's mode independent, including neighboring Shaded views. Preserve source line RGB and source `m_wire_density` in `ViewObject.OM9IsoCurveDensity` through geometry/preservation import and FCStd reopen. On a black background, display very dark edges with contrasting gray without changing stored LineColor.

Isocurves are display guides, unpickable and outside document geometry; native CAD edge/vertex picking remains available. Trim guides against pcurves/holes, skip interior guides on analytical planes, and invalidate caches on shape/density/color changes. Adapt sources behind external/nested/reflected Links; on workbench exit restore canonical provider modes across documents before removing shared scene nodes, without forwarding stale Link modes into sources. Rendering must not change shapes, archive bytes/UUID graphs, source files or Undo.

Density semantics: -1 means boundaries only; 0 means knot wires; 1 adds an interior wire when there are no internal knots; N>=2 adds N-1 wires per knot span. Current display caps density at32 with a warning while retaining the raw source property. Display chord deflection0.005mm is not a geometry tolerance. Preview limits are4096 parameter lines per face,65536 points per guide and2 million points per object; report preview failure and keep CAD boundaries. Newly inserted providers normalize on the next workspace update/activation (currently a200ms timer).

Verified limits: affine compound previews do not retain distinct per-member density, and mixed affine preview children still default to1. ViewObject density edits currently affect display only; geometry-only export does not write these edits into native attributes. No new Rhino5 isocurve/pixel oracle exists. CAD display acceptance does not establish full openNURBS or equivalence to every Matrix/Rhino preset. Repository evidence: `docs/validation/2026-10-08-cad-wireframe.md` and its `cad-wireframe-20261008/summary.json`, resolved against the active OM9 checkout; compact packaged checkpoint: [cad-wireframe-proof.json](../assets/cad-wireframe-proof.json).

Verify original names, active marks, double-click and dropdown toggles, independent view modes, exit/reactivation, document/view replacement and pending Polyline preservation. Check actual native geometry and both menu/mouse and CMD entry paths where available; matching screenshots alone is not acceptance.

For CAD display changes, use an independent planar BRep to detect unwanted triangle diagonals and verify face/edge picking; include a trimmed curved face with a hole, knot/density variants, a separate mesh, inactive external/nested reflected Links, edit/cache and FCStd reopen. Compare source/shape hashes, Undo and neighboring view images; run native Qt Import/Export actions to cover the actual menu callback. Keep the supplied ring fixtures as regression. Report binary-bound evidence and unresolved properties rather than a completion percentage.
