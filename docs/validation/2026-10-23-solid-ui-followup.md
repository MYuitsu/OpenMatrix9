# Solid geometry and UI follow-up — 2026-10-23

Standalone Windows x64 Rust OM9, OCCT 8.0.1. This follows the
[exact ellipsoid batch](2026-10-23-exact-ellipsoid.md), preserving its historical
receipt and source hashes. New logs, authored fixtures, screenshots, runtime hashes
and [receipt](../../implementation/evidence/M03/solid-ui-followup-2026-10-23/receipt.json)
are stored under `implementation/evidence/M03/solid-ui-followup-2026-10-23/`.

## Reproduced defects and repairs

Four new regression tests failed before the repairs (`red.log`):

| Before | After |
|---|---|
| Text menu returned usage instead of accepting content | Text prompt, then Height; spaces and digits remain content; invalid height can be retried; Esc makes no edit |
| Boolean Two Objects menu returned usage | Operation prompt accepts U/Union, D/Difference, I/Intersection, case insensitive; invalid choice retains the prompt |
| ExtrudeCrv sampled a circle into 128 straight edges; r=3, h=5 measured 141.314004679227 mm³ | Analytic circles and rational NURBS retain their original basis; measured 141.371669411541 mm³ = 45π |
| ExtrudeAll published each result separately | All results are prepared before one transaction; one Undo/Redo handles the whole operation; a later invalid profile leaves document/history/old Redo unchanged |

Negative or fractional topology indices are also rejected before Extract Surface,
Fillet Edge or Shell can silently target index zero. Existing names and feature IDs
are preserved. Milestone row numbers differ from runtime IDs: Sphere is L3-013 /
`OM9-SOLID-014`, Ellipsoid is L3-014 / `OM9-SOLID-013`; Boolean Union is
L3-001 / `OM9-SOLID-003`. The checklist records both explicitly.

Safe Rust owns validation, curve traversal, borrowed input packing, pending prompts,
budget accounting, staging and atomic publication. The new C++ exception is scoped to
OCCT edge/wire/planar-face/prism construction and the existing owned snapshot ABI.
It catches native exceptions and retains no Rust pointers. This does not certify
OCCT or unsafe FFI as memory safe.

## Exact geometry and numerical measurement

The authored rational biquadratic 9×5 ellipsoid net remains the construction path;
there is no GTransform/converter/fitting fallback. Ellipse extrusion exposed another
integration issue: adaptive Gauss returned 94.999955159970 mm³ for a rational ellipse
with expected 94.247779607694 mm³, despite correct side intersections. Canonical
volume measurement now uses span-aware `BRepGProp::VolumePropertiesGK` with Eps=1e-12,
and rejects an integration failure. Surface area retains adaptive integration.

The six ellipsoid axis fixtures now have maximum observed relative volume error
`2.220e-15`; semi axes (2,3,4) measure **100.530964914874 mm³**, analytic
100.530964914873 mm³. The acceptance threshold stays `1e-10` relative. The same
498 diameters / 996 intersections, serialized-source reload, one solid/shell and zero
naked edges pass. Exact means rational representation, subject to f64 and OCCT
tolerances; volume agreement alone is not a surface proof.

Exact extrusion tests cover an ellipse, its reversed curve, and a reversed rectangle,
each with heights -5 and +5 (six positive fixtures). Ellipse side intersections use
independent analytic fractions. Open, nonplanar, disconnected and tangential cases,
zero/NaN direction and uncertified periodic continuity are rejected. General periodic
NURBS continuity is an existing core limitation, not certified by this batch.

## Validation

All Cargo commands use `CARGO_TARGET_DIR=L:/om9-target`, `-j 2` and the pinned
OCCT 8.0.1 DLL directory on PATH. `kernel.log`, `app.log`, `build.log`, `clippy.log`,
`review-fixtures.log`, `native-files.log` and `clippy-final.log` record the invocations.

- Full kernel: 71 passed, 0 failed, one pre-existing ignored lifetime probe.
- App: 176 library + 7 Solid + 8 new workflow tests = 191 passed, 0 failed.
  Native saved-file tests are ignored in ordinary runs and invoked explicitly.
- All 21 Solid bindings have measured Controller menu-dispatch coverage. This is
  distinct from clicking all 21 menu items in a real window.
- Native app build and Clippy with warnings denied pass.
- Thirty-three authored `.om9` review files were exported by the passing workflows.
- Scoped Rust 2024 formatting, whitespace checks, JSON and review-link checks pass.
  Public source audit reports 12,179 source files, 511 icon bindings, 510 SVGs and
  zero errors. Artifact pruning ran: zero removable old runs; cited evidence is kept.
- One explicit saved-file test cold-opened three files produced through real Windows
  menu/input/Save As actions: Text H/height 5 (one closed solid, volume 2.912109375),
  Boolean Two Objects/U (one closed solid, 1499.999999999999), ExtrudeAll/height 5
  (two original curves plus two closed solids, total 282.743338823081 mm³).
  Every measured solid has zero naked edges. Text also matches the authored H fixture.

The native executable SHA256 is
`74e292a854014d0432a6a4a38dddd7e40e032c98e29a129615227c05011d8195`.
Text and Boolean operation prompts were observed; ExtrudeAll was observed through
one Undo removing both solids and one Redo restoring both. All three files were saved
under new names, and the clean windows were closed. Input review fixtures were kept.
The prior native Ellipsoid observation used the earlier binary and retains its own
fingerprints. Latest exact ellipsoid geometry is covered by the current kernel tests.

Text curved glyph outlines still use the existing sampled font contours; this batch
certifies H and the prompt, not exact Bézier glyph geometry. The Pipe fixture is a
straight line; the existing sampled curved-spine path is not newly certified.

## Human review and proposed reusable rules

[M03-UI checklist and fixtures](../../implementation/evidence/M03/solid-ui-followup-2026-10-23/M03-UI-CHECKLIST.md)
provide all 21 scenarios. Human acceptance remains unsigned, `human_approval=null`
and `manual_gate_promoted=false`; agent observations cannot supply that signature.
No release or full-L2 completion is claimed.

Root AGENTS.md (business rules, 2026-10-05) requires consent before recording changed
business behavior in reusable skills. Proposed addition to the feature-port skill:

> Solid > Text opens content, then Height. Preserve spaces and digits as content;
> invalid height remains at Height, and Esc cancels without a document edit.
> Solid > Boolean Two Objects opens Operation and accepts U/Union, D/Difference,
> I/Intersection case insensitively. An invalid option retains the prompt. Valid menu
> and CMD input use the same dispatcher and one transaction.

These prompt rules are proposed only; no repository or installed skill has been changed.

Approval update, 2026-10-23: the user subsequently answered “Đồng ý ghi vào skill”.
The approved rules are now recorded in the source and installed feature-port skill,
under [solid-prompt-contract](../../skills/openmatrix9-feature-port/references/solid-prompt-contract.md).
The proposal and receipt above describe the earlier state. The user separately
answered “Chưa duyệt” for M03-UI, so that gate remains pending.
