# Circle remaining branches — 2026-10-09

Feature `OM9-CURVE-005` remains `partially_implemented` / `validated_supported_slice`.
[Current scope](../features/OM9-CURVE-005.md). Source: Matrix8 Book1 PDF138–140,
printed128–130. Original engineering contract and Rust example remain unchanged.

## Native evidence

Matching SDK FreeCAD.exe, MSVC2022/Qt6/OpenCascade8 native build succeeds.
Each report is `ok=true`, all checks pass and runner process exits0; selected
native module path is asserted. New coverage176 checks; existing/shared regressions665.
Final native module SHA256 `97d84edbbe29c108b69fa07affc08cd0d108ffdae2cefdbd2e1a4ac566330908`. The final targeted suites were rerun after
the private History spline adapter fix; earlier regression reports cover unchanged
Circle/Curve and shared History/Surface/Solid behavior.

| Suite | Checks | Report relative to module root |
|---|---:|---|
| circle_nonplanar_smoke | 30 | `build/circle_nonplanar_smoke-1/0232008ec2214cf98453ea19d93eee0b/results.json` |
| circle_history_more_smoke | 89 | `build/circle_history_more_smoke-1/66ef2407bc6a467ca03f62ffd1b3ff22/results.json` |
| circle_layers_smoke | 52 | `build/circle_layers_smoke-1/04d2073d994748429eafa5eae05e0917/results.json` |
| circle_history_more_restore | 5 | `build/circle_history_more_restore-1/c60d95d4e984413ba7375d0111ff30b6/results.json` |
| circle_spatial_history_smoke | 51 | `build/circle_spatial_history_smoke-1/1c701b4b14d54f48a2a15b1b0210d236/results.json` |
| circle_history_restore | 3 | `build/circle_history_restore-1/f97c234cc8654b15868fb37d4efab98a/results.json` |
| circle_smoke | 42 | `build/circle_smoke-1/0ec5c7dfad1540919ca7ca27115d5565/results.json` |
| circle_advanced_smoke | 64 | `build/circle_advanced_smoke-1/f2bc37bf287f416a96879647d7b62e94/results.json` |
| rectangle_smoke | 37 | `build/rectangle_smoke-1/8779fbea46ec4ef3a7ae7e44bcba1ab8/results.json` |
| curve_smoke | 27 | `build/curve_smoke-1/de25c7ffe10a479ea03103231560c6db/results.json` |
| curve_spline_smoke | 41 | `build/curve_spline_smoke-1/5167e2b11fb54e988becc41ae86ecfd0/results.json` |
| history_smoke | 83 | `build/history_smoke-1/18c157ce039a4e1aaa1b9194ef04a102/results.json` |
| surface_history_smoke | 65 | `build/surface_history_smoke-1/eda69e6472fd47b785d949053b666c0f/results.json` |
| solid_options_smoke | 252 | `build/solid_options_smoke-1/880b9acc7f0e40a29754fdce449585fb/results.json` |

## Covered behavior

Noncoplanar cubic NURBS with exact known contact circle; moving contacts, two-curve
Radius, FromFirstPoint, free Point, Vertical rejection/recovery, transformed sources,
native angular/radial verification, near-planar projection regression, Deformable,
spatial History source/placement updates, Undo/Redo and FCStd.

All Circle recipes: Center numeric/picked, 2Point, 3Point/free/Radius, Vertical,
Orientation/live Direction, AroundCurve, FitPoints Selection and Tangent, including
Deformable. Live explicit vertices, direction and picked Radius endpoints update;
replaced sources are pruned. Whole Fit sources allow bounded topology growth.
Record/Update/Lock, constant-only records, detach/source deletion under Lock,
Undo/Redo, hot restore and cold-process restore without activating the workbench.

Layer selection via32 sidebar slots and typed names/index/None, persistent plain
root groups, colors/visibility/lock, lock rejection before any creation transaction,
normal/History/Deformable ownership, one-step creation Undo, color/visibility Undo,
per-document isolation and FCStd. Imported metadata/color is verified unchanged
before save; reload comparison uses FreeCAD's native float32/8-bit color packing.

## Ownership, review and verification

Safe Rust owns numeric/session/replay/solver and layer policies. GUI-thread C++
adapts exact native curve evaluation, document links/transactions and view providers.
Synchronous callbacks retain no pointers and catch native/Python exceptions before
returning through FFI. FFI and OCCT are not certified memory-safe.

Review corrected stale Direction source replacement, exact3D postcheck on the planar
solver path, and replay Vertical/FromFirstPoint acceptance parity. Native tests exposed
periodic History construction passing an integer where FreeCAD requires a PyBool;
the adapter now passes Py_True/Py_False and all Deformable replay checks pass.

Complete Rust run:237 tests pass in39 nonempty suites, including56 Circle/layer tests. Python tools:23 pass. Clippy exits0
with style/legacy warnings; selected Rust formatting and Git whitespace checks pass.
An intermediate concurrent Cage command test failed; the final complete Rust run
passes. See `build/circle-complete-rust-tests.log` and
`build/circle-complete-targeted-tests.log`. Source audit still flags290 existing
private/reference artifacts and validates510 icon bindings; no publication performed.

## Limits

Spatial search is bounded, multistart/local and not exhaustive/certified. Failure to
find a verified solution does not establish mathematical infeasibility. Stored
VertexN/EdgeN and normalized fractions are not automatic topology correspondence.
Individual surface edit-CV selection and full Rhino layer inheritance/import mapping
remain unaccepted. Typed/mouse point values without explicit native source refs are
constants. Layer support here applies to Circle outputs; it is not global CAD feature
acceptance. The complete original Matrix compatibility contract remains unaccepted.
