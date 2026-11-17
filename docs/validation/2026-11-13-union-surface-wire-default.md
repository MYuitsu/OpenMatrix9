# Certified Union and default surface wires — 2026-11-13

## Certified Union

The user authorized trying the existing strict axis-aligned spline-box certificate for Union. Checked product Union and Boolean Two Objects Union may now use a temporary analytic operand only when that certificate succeeds. Other Boolean modes, including intermediate unions, keep their earlier path; original source operands remain the basis for validation.

`boolean_union_certified_baked_box_is_correct_and_undoable` exercised a saved/reopened radius-3 sphere and transformed 4 × 8 × 8 box through both product Union and Boolean Two Objects Union at two display deflections. Each result was one closed solid with no naked edges and the analytic volume `256 + 18π` mm³; preview did not change the document or Redo, and Undo/Redo restored the document. Eleven additional Boolean Two Objects regression tests passed. This is one geometry fixture and is not a proof of every Boolean result or every possible spline box.

## Untrimmed surface wires

For a standalone NURBS surface without trim metadata, scene and ExtractWireframe now share default-density parameters: distinct interior knots per axis, or a midpoint when that axis has no interior knots. The display samples those curves; ExtractWireframe creates exact NURBS curves with the source placement in the same transaction as boundaries. The previously supported trimmed paths are unchanged. The default follows the meaning of density 1 in the local openNURBS header, not a claim that a user-set or imported 3DM density is active in OM9; no active density setting is implemented here.

The reopened plane fixture checks four boundaries and two midpoint wires in display and exact output, plus Undo/Redo. `surface_knot_wires_match_display_and_exact_extraction_after_reopen` checks a placed cylinder: three interior knot wires and one midpoint wire, sampled against the source surface and exact extracted curves, through CMD and menu dispatch, saving/reopening and Undo/Redo. Surface, periodic, cone, trim, mixed-selection, pressure and related command regressions: 89 passed, 8 ignored. Scene metadata/admission and surface-picking regressions: 32 passed. Managed pressure is simulated, not physical exhaustion. Desktop visual appearance, arbitrary surfaces, non-default density, native-memory peak and third-party 3DM density are not established by these tests.

The debug app built in `L:/om9-target-surface` with two jobs; the no-default-features check passed. Public source audit reported no errors. Existing compiler warnings remain. No new build directory was created. The user approved recording the two tested scopes in the feature guidance; the repository and installed copies were updated identically.
