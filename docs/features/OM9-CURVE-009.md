# OM9-CURVE-009 — Rebuild

Command `Rebuild`, icon `OthersCurveRebuild`, Curve menu. Implemented slice:
native curve/wire selection, exact degree/pole-count fitting, preview and batch
transactions. Surface Rebuild is a different, unsupported feature.

## Evidence

Requested catalog: `OpenMatrix9_Codex_Spec_v1/specs/02-curve/om9-curve-009-rebuild.md`.
It describes selection, current PointCount/Degree, Preview and OK.
Matrix 8 Book 1 printed p.134 / PDF p.144 remains `TODO_EVIDENCE`: its registered
local PDF is absent. Refit to Tolerance begins at PDF p.145, so p.144 is the
catalog boundary; this is not a verified manual-page read.

Supplementary primary documentation, read 2026-10-06:
[Rhino 5 Rebuild](https://docs.mcneel.com/rhino/5/help/en-us/commands/rebuild.htm).
It supports specified control-point count/degree, evenly spaced knots,
DeleteInput, refreshed Preview and reported deviation. It does not establish
Matrix defaults or the OpenMatrix9 fitting algorithm.

## OpenMatrix9 decisions

- Select whole native curve objects before invocation, or select after invocation
  and press Enter. Accept single-edge curves and one connected, complete wire
  per object, including polylines; process at most 16 objects in a batch.
  Reject faces/solids, disconnected curves and subobject selections explicitly.
- PointCount: 2–256, strictly greater than Degree. Degree: 1–11, including even
  degrees. Current B-spline structure is shown per input; analytic/segmented
  curves are labelled separately. Initial target count is the first input's
  pole count clamped to 4–256 (4 if unavailable); initial degree is its degree
  clamped to 1–3. DeleteInput defaults to true. These are host decisions.
- Samples: native OCCT equal-arc-length discretization, `max(513,4*PointCount+1)`
  samples per input in world coordinates. Rust fits a non-rational B-spline with
  uniform knots by bounded least squares; open endpoints are fixed exactly.
  Closed inputs use a periodic fit, with the repeated endpoint removed from the
  solve. This can smooth corners; it does not promise tolerance-constrained refit.
- Preview displays temporary unpickable scene geometry. Maximum deviation is
  explicitly labelled **sampled**: a bidirectional maximum of native nearest
  geometric distances at 129 equal-arc-length points per side. It is an estimate,
  not a certified Hausdorff bound. Changing fit settings clears stale preview
  and deviation. OK recomputes using the current values, even without Preview.
- The current-layer option is visibly disabled because an active layer system
  is not present. Output is created at the document root. Native input line,
  point color and line width are copied. Input-group/layer ownership is not
  claimed to be preserved.
- Accumulate the parent placement when sampling a curve inside App::Part, so
  root-level output and preview retain the displayed world position.
- Before fitting/commit, verify each selected object still exists, its native
  shape is unchanged and its global placement still matches. DeleteInput rejects objects with dependents rather than
  severing their links; clear DeleteInput to keep such inputs.
- Build all results before opening one batch transaction. OK creates one
  `Part::Feature` per input; DeleteInput removes the selected sources only after
  outputs exist. Failure aborts the transaction. Undo/Redo restores the entire
  batch; FCStd saves native splines. No associative History/Builder/Styles model
  is created.
- Invalid options/fits keep the options editable and leave the document intact.
  Cancel/Esc, document switching/deletion and workbench deactivation remove
  preview and the dialog. A rejected selection yields an explicit prompt.
- No point picking, CPlane coordinate entry, snap, window-selection, surface
  trimming or solid-volume behavior is added by Rebuild; existing native
  selection handles object collection.

## Validation and continuation

Rust geometry tests check exact requested structure, uniform knots, endpoint
preservation, periodic circle fit and invalid/ill-conditioned input limits.
`tests/curve_spline_smoke.FCMacro` checks menu/CMD, pre/postselection, preview,
cancel, open/closed/multiple inputs, polyline wires, DeleteInput, invalid degree
and count, Undo/Redo, save/reload and document-switch cancellation.

Remaining compatibility: Matrix manual/default verification, layer mapping,
subedge editing, exact global deviation certification and full original
Rebuild's specialist options. Next document: Matrix 8 Book 1 PDF p.144.
