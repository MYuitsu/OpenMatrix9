# OM9-VIEW-006 — Zoom Extents

Source: `specs/01-core/om9-view-006-zoom-extents.md`, Matrix8 Book1, printed92/PDF102. The full section and boundary before Zoom Selected were verified in the local manual.

The source describes fitting the model's complete extents and recentering the active viewport. OpenMatrix9 maps `ViewZoomZoomExtents` and Workspace `FitAll` to FreeCAD's `Std_ViewFitAll`. The active3D view supplies bounding boxes, margins and camera animation; these are host choices, not recovered Matrix defaults. No numeric inputs, new geometry, builder preview, geometry undo or dependency history are introduced. Native view/task permissions govern availability. A successful, available host view dispatch enters bounded session icon history; geometry and selection are preserved.

The real test fixture uses two solids separated100mm. Fit all establishes an orthographic framing height, and OM9-VIEW-007 selected fit must reduce that height while preserving objects and selection. Selection/group/subelement/link framing is delegated to the host; only whole-object solid fixture framing is measured here. FCStd geometry persistence is verified separately; camera persistence is not claimed.

Final evidence is in `docs/validation/2026-10-04-workspace.md`. The next independent UI feature is four simultaneous viewports; this adapter affects only the current active native view.
