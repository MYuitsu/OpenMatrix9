# OM9-SURFACE-001 — Sweep 1

Menu ID: `OM9_SurfaceSweepSweep1Rail`. CMD alias: `Sweep1`.

## Source-derived contract

The supplied `OpenMatrix9_Codex_Spec_v1/specs/03-surface/om9-surface-001-sweep-1.md`
defines one rail followed by one or more cross-section curves in selection order,
Enter, seam alignment for closed profiles, and a Sweep 1 options dialog.
Its citation is Matrix 8 Book 1, printed p.170 / PDF p.180.
The requested `ref/matrix9/` prefix is absent in this checkout; the supplied
package is located directly under `Mod/OpenMatrix9/OpenMatrix9_Codex_Spec_v1`.

`TODO_EVIDENCE`: the original PDF is absent at the registered local location.
Continuation pages, Chain Edges and additional Matrix option semantics have not
been reverified. Implementation decisions below are not recovered Matrix defaults.

## OpenMatrix9 implementation decisions

- Rust owns the command identity, ordered input session and option validation.
  C++/Qt hosts selection, the dialog and FreeCAD's typed Part/OpenCascade APIs.
- Inputs: a Part edge or connected wire, optionally an explicitly selected
  `EdgeN`/`WireN`; then one or more profiles, all open or all closed. Curves
  must be valid and longer than 1e-7 mm. Duplicate references are rejected.
  The host limit is 256 inputs. Whole faces/solids are rejected; select edges.
- The default host trihedron is CorrectedFrenet. The dialog also exposes Frenet.
  These are host choices. Per-input Reverse is available. Closed single-edge
  profiles can move the seam by a normalized curve parameter fraction [0,1).
  Multi-edge seam rotation and mouse-drag seam markers remain unsupported.
- Preview uses an unpickable scene-graph node in every existing 3D view;
  it creates no document object or undo entry. OK rebuilds and validates the
  exact shape before opening a single document transaction. Esc/Cancel removes
  the preview. Document close/switch and workbench deactivation cancel safely.
- Output: an uncapped `Part::Feature` surface/shell, never an automatic solid.
  Original curves remain. `OM9FeatureId`, `OM9Command`, `SourceCurves` and
  `SurfaceOptions` persist in FCStd. Geometry is a snapshot; source links are
  provenance, not a recompute feature or Sweep 1 History (OM9-SURFACE-002).
- Kernel failures disable OK, report the error and leave no partial output.
  Undo/Redo covers the output and metadata in one transaction.

## Validation

`rust/tests/surface_session.rs` verifies ordered inputs, duplicates, minimum
counts and cancellation. `tests/surface_commands_smoke.FCMacro` checks native
cylinder and closed-ring torus surface areas, validity, topology, preservation
of inputs, preview isolation, Undo/Redo and FCStd save/reload.
Current test results and artifact locations are in `docs/openmatrix9-progress.json`.

Surface menu, CMD alias and the F6 Surface submenu use the same handler.
This is a native implementation of the documented command slice; full Matrix
and Rhino compatibility, exact F6 context parity and every Matrix option are unverified.
