# OM9-CURVE-003 — Interp Curve

Command `InterpCrv`, icon `CurveFreeFormInterpolatePoints`, Curve menu.
Implemented slice: native point interpolation, parameterization, closure and
command lifecycle. This is not a claim of complete Matrix/Rhino compatibility.

## Evidence

Requested catalog: `OpenMatrix9_Codex_Spec_v1/specs/02-curve/om9-curve-003-interp-curve.md`.
The former `ref/matrix9/` package is now outside the public source layout.
Catalog evidence describes point picking, Enter, automatic close and Alt.
Matrix 8 Book 1 printed pp.125–126 / PDF pp.135–136 are still
`TODO_EVIDENCE`: the registered PDF no longer exists on this machine. The next
feature, Rectangle, starts at PDF p.137; continuation range is inferred from
that boundary, not a verified manual read. Original references are preserved.

Supplementary primary documentation, read 2026-10-06:
[Rhino 5 InterpCrv](https://docs.mcneel.com/rhino/5/help/en-us/commands/interpcrv.htm).
It supports the meanings of Uniform, Chord, SqrtChrd, smooth periodic Close,
nonperiodic Sharp, Undo and Alt suspension. This is supplemental evidence,
not recovery of Matrix defaults.

## OpenMatrix9 decisions

- Inputs: ordered world points, entered through the shared command frame or
  viewport picker. CMD coordinates/relative coordinates use the active CPlane;
  mouse coordinates use the existing native snaps and Ortho/Shift rules.
  Selection is not required; unrelated selected objects remain untouched.
- Defaults: degree 3, Uniform knots, PersistentClose=No; millimetres. Accepted
  degrees are 1, 3, 5, 7, 9, 11. With too few points, open degree is reduced to
  `min(requested, points-1)`; periodic degree is additionally reduced to odd.
  This bounded support is a host decision, not a verified Matrix degree range.
- Limits: at most 256 picked points, finite coordinates within 1e9 mm,
  consecutive points at least 1e-7 mm apart. Failed/ill-conditioned fits give an
  editable error, never nonfinite geometry.
- Close: at least three picked points; creates a periodic B-spline. AutoClose
  uses a 10 logical-pixel radius around the first point and works without Osnap.
  Alt suspends it. Sharp repeats the start point in an open knot-vector fit to
  create a closed, nonperiodic curve. Enter commits the current open fit unless
  PersistentClose is enabled. PersistentClose previews a closed curve from three
  distinct points; the two-point Rhino variant is not implemented.
- Degree/Knots support both `Option=value` and pending value prompts; Undo removes
  the last picked point. Esc/Cancel removes temporary geometry. StartTangent and
  EndTangent are explicitly unsupported and return an error.
- Output: one valid native `Part::Feature` with one non-rational B-spline edge,
  constructed by Part/OpenCascade from Rust's poles/knots/multiplicities.
  Output uses document-root ownership and the established Curve line color.
- Preview is unpickable scene geometry in the document's 3D views. It uses the
  same fit and explicit close state as commit, including Alt and CMD repeated
  endpoints. It does not add a document object or transaction.
- Commit is one document transaction. Undo/Redo and FCStd serialization use
  native FreeCAD geometry. Menu, CMD and repeat share the same session. Success
  history is recorded only after geometry commit. Changing/replacing/closing the
  document or leaving the workbench cancels the session.
- Builder/Styles/associative History are not used: this slice outputs standalone
  geometry. Unrelated surface, solid, trim, volume and selection-window foundation
  requirements do not apply to this point-drawing command.

## Validation and continuation

`rust/tests/spline_geometry.rs`, `rust/tests/curve_session.rs` and
`rust/tests/curve_command_permissions.rs` cover interpolation, closed seam,
invalid values, option flow, preview/commit consistency and write permissions.
`tests/curve_spline_smoke.FCMacro` tests actual menu/CMD/mouse events, native
interpolation within 1e-7 mm, AutoClose/Alt, cancellation and persistence.
Existing `tests/curve_smoke.FCMacro` protects Line/Polyline behavior.

Remaining compatibility: Matrix manual/default verification, tangent workflows,
two-point PersistentClose, specialist foundation workflows beyond the existing
host picker. Next document: Matrix 8 Book 1 PDF pp.135–136 when available.
