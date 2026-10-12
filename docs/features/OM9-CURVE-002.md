# OM9-CURVE-002 — Line

Status: PARTIAL. Original public command: `Line`.

Reference: `ref/matrix9/OpenMatrix9_Codex_Spec_v1/specs/02-curve/om9-curve-002-line.md`. Use the existing spec without PDF rereading.

## Spec-derived behavior

Draw one line segment by choosing its start and end. The spec lists BothSides, Normal/IgnoreTrims, Angled, Vertical, FourPoint and Bisector option cues; it does not normalize every option's complete parameters or defaults.

## Implemented contract and host decisions

- Two-point Line and BothSides are supported through one Rust session for menu, viewport and CMD. Public command is `Line`; icon key remains `CurveLineSingleLine`.
- BothSides before the first point toggles symmetry about that first point. With midpoint `(0,0,0)` and endpoint `(3,4,0)`, output endpoints are `(-3,-4,0)` and `(3,4,0)`, length 10 mm. This is an explicitly documented implementation choice for the initially unnormalized option.
- A normal two-point Line from `(0,0,0)` to `(3,4,0)` produces one native wire edge of length 5 mm. Commit occurs after the second valid point. Empty Enter before two points and coincident/nonfinite input cannot create geometry.
- Coordinate syntax, units, per-view C-plane/DPI picking, finite-input limits, error handling, successful-only history and document/workbench cancellation follow the Polyline host contract.
- Geometry is created with native Part/OpenCascade bindings using typed Python C API arguments; CMD text is never evaluated as Python or shell code. One native transaction supplies Undo/Redo and persistence.

## Remaining behavior

Normal/IgnoreTrims, Angled, Vertical, FourPoint and Bisector remain unsupported. Other advanced Line behavior is also not claimed. No temporary rubber-band preview or Matrix snapping/C-plane parity is implemented. The command remains PARTIAL even when the basic mouse and CMD cases pass.

## Evidence

The Rust semantic fixture verifies endpoints and BothSides symmetry. The native Curve fixture verifies menu/viewport and CMD geometry, independent ray projection, Undo/Redo, save/reload, errors and lifecycle. Consult the Curve validation report for exact executed cases.
