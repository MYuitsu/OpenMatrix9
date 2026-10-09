# Grid and native viewports

## Finite construction grid

The grid has **8 by 8 major cells**, each divided into **5 by 5 minor cells**, giving **40 by 40 minor cells**. Current OM9 spacing is 1 mm minor and 5 mm major; the construction-plane extent is -20 to +20 mm on each axis. This is a bounded world grid, not an infinite screen overlay. Pan/zoom changes its screen position and scale; rotation follows the native camera.

Keep the grid unpickable, outside document geometry and excluded from model fit bounds. Viewport canvases are black (#000000); major lines are stronger than minor lines. Command/active UI uses Matrix green (#82B48C), with black text. New Curve edges use green (#008255). Native four-view rendering and geometry stay in FreeCAD.

Reference: [user's finite grid screenshot](../assets/matrix-finite-grid.png).

## Viewport titles and layout

Each native view has a small top-left label and a separate small dropdown arrow. Keep the original names: **Looking Down**, **Perspective**, **Side View**, **Through Finger**. Active label uses Matrix green; inactive labels use gray.

Clicking a label activates its own view. Double-clicking the label shows only that view; double-clicking again restores four views (4V). Dropdown **Maximize / Restore 4V** calls the same action. Tabs switch the active single view while maximized.

Preserve camera pose/zoom, geometry, selection, document Undo and an active point command while toggling. Native redraw may update automatic near/far clipping; this does not authorize resetting the camera. Restore native title chrome and display settings on workbench exit. Rebuild one title/menu per replaced view; respect active-document and native task restrictions.

Reference: [user's viewport title and dropdown screenshot](../assets/matrix-viewport-menu.png).

## Display modes and implementation limits

Keep the original display-mode names from the screenshot and mark the current mode. **Wireframe** and **Shaded** have basic native adapters for the selected viewport; preserve the neighbor's mode. Reconcile local menu marks after native global Wireframe/Shaded changes. Other Matrix rendering modes and submenu operations remain visibly disabled until implemented and verified.

Current Wireframe uses native Coin polygon lines with CAD edge topology retained, so tessellation lines can be visible. Shaded fills surfaces and retains CAD edges/points. This does not establish equivalence to every Matrix/Rhino display engine or preset. Newly inserted providers normalize on the next workspace update/activation (currently a 200 ms timer).

Verify original names, active marks, double-click and dropdown toggles, independent view modes, exit/reactivation, document/view replacement and pending Polyline preservation. Check actual native geometry and both menu/mouse and CMD entry paths where available; matching screenshots alone is not acceptance.
