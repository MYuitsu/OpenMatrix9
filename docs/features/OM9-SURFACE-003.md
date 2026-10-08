# OM9-SURFACE-003 — Sweep 2

Menu ID: `OM9_SurfaceSweepSweep2Rails`. CMD alias: `Sweep2`.

## Source-derived contract

The supplied `OpenMatrix9_Codex_Spec_v1/specs/03-surface/om9-surface-003-sweep-2.md`
defines two rails followed by cross-sections in selection order, Enter,
closed-profile direction/seam alignment, and a Sweep 2 options dialog.
Citation: Matrix 8 Book 1, printed p.174 / PDF p.184.
`TODO_EVIDENCE`: original PDF and continuation pages are unavailable at the
registered location. The spec mentions Chain Edges without normalized semantics.
It does not establish defaults for this OpenMatrix9 kernel implementation.

## OpenMatrix9 implementation decisions

Shares the input, preview, transaction, error and persistence contract of
[Sweep 1](OM9-SURFACE-001.md), with two distinct rails before the profiles.
The output retains both rails and every section in `SourceCurves`.

The matching OCCT SDK's ContactOnBorder call fails on a minimal straight-rail,
straight-profile fixture (including the one-section case); it is never used.
The host instead uses these explicit strategies:

- One profile: it must intersect the starts of both rails; the rails must have
  matching open/closed state. Sample 65 equivalent arc-length stations on each
  rail. Rust computes frames from the rail separation and primary rail tangent,
  transports the section, and scales both section-plane axes by the ratio of
  rail separation. FreeCAD sweeps these sections using the second rail as the
  auxiliary normal guide. Closed rails omit the duplicate last station.
  This is an OpenMatrix9 design decision, not a Matrix option default.
- Multiple profiles: use the native auxiliary spine Contact mode with the
  original profiles. The kernel may reject rail/profile combinations it cannot
  satisfy. It does not silently fall back to a one-rail sweep or ignore inputs.
- Validate 65 arc-length samples on each original rail and section against the
  resulting surface with tolerance `max(1e-4 mm, curveLength * 1e-7)`.
  Sampling is a bounded numerical check, not proof of exact whole-curve contact.
  Crossing/coincident rail frames are rejected. The committed object is an
  uncapped surface/shell, with no automatic solid or source-edit recomputation.

Per-input Reverse and single-edge closed-profile seam fractions are available.
Rail seam dragging, Maintain Height, Chain Edges and Sweep 2 History
(OM9-SURFACE-004) are not implemented in this slice.

## Validation

Rust tests check rail order, duplicate rails, profile closure and transport
matrices. The native macro verifies a two-profile trapezoidal strip with area
65 mm² and a one-profile strip whose width grows from 5 to 9 mm with area
70 mm². It also checks both rails, topology, Undo/Redo and save/reload.
See `docs/openmatrix9-progress.json` for current execution evidence.

Next evidence: verify the original Sweep 2 continuation pages, then extend
general rail/profile correspondence and adaptive boundary error checks.
