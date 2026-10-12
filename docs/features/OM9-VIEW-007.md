# OM9-VIEW-007 — Zoom Selected

Source spec: `ref/matrix9/OpenMatrix9_Codex_Spec_v1/specs/01-core/om9-view-007-zoom-selected.md`. Matrix8 Book1, printed92 / PDF102 was verified locally; the section ends on that same page before Zoom Window. PDF103 was checked to establish continuation boundaries. OM9-VIEW-006 Zoom Extents is also wholly described on PDF102.

## Source-derived behavior

Zoom Selected fits the chosen objects or points into the active viewport. Zoom Extents fits the entire model and recenters that viewport. Both describe view changes, without new geometry or input dimensions. The manual supplies no FreeCAD bounding-box margin or animation defaults.

## OpenMatrix9 decisions

Map `ViewZoomZoomSelected` to `Std_ViewFitSelection`, accessible through the original View-menu icon, Workspace menu and Fit sel button. OM9-VIEW-006's `ViewZoomZoomExtents`/`FitAll` use `Std_ViewFitAll`. The active3D view and its native host bounding boxes determine framing. Selection comes from the active document; no selection or an edit session disables selected fit. Arbitrary unsupported geometry is not converted. There are no numeric parameters or geometry outputs; units remain the existing document's units.

Commands run immediately, with native camera animation and no preview/commit/cancel builder lifecycle. They do not add geometry undo transactions or dependency history. OpenMatrix9 records a successful, available host view dispatch; it does not assert camera persistence inside FCStd or geometrical modeling behavior. Host dialogs are not introduced.

Validation uses two solid fixtures separated100mm, an orthographic camera and selected One. Fit sel must reduce camera height to less than half of Fit all while preserving selection and shapes. Real menu/button availability, cameras and native transactions are tested separately. The automated fixture proves whole-object solid selection; subelement/point/link/group bounding-box fidelity remains host-dependent and unverified.

See `docs/validation/2026-10-04-workspace.md` for final evidence. The next independent view slice is OM9-VIEWPORT-001; four simultaneous views and Matrix-specific C-planes remain deferred.
