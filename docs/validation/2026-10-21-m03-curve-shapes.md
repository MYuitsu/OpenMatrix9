# Ellipse and rectangle construction through SDK v1 — 2026-10-21

This closes the remaining creation kinds of `OM9-L3-02-04-002` for the plugin boundary: the
owner's ellipse and rectangle builders are now reachable, so a plugin can publish a rational
ellipse and a closed rectangle in addition to circles, arcs and NURBS curves.

## Changed

| Layer | Change |
|---|---|
| Public SDK | `Host::add_ellipse(label, origin, x, y, radius_x, radius_y)` and `Host::add_rectangle(label, origin, x, y, width, height)`; two requests and capability entries. |
| Host | Calls `curve_build::ellipse` (degree-2 rational NURBS) and `curve_build::rectangle` (closed four-segment polycurve). Both require a unit, perpendicular frame `(x, y)` and positive sizes; malformed input is a `curve` refusal. Staging uses the shared `add_output` path. |
| Example plugin | `CurveShapeDemo` creates an ellipse and a rectangle, then reads info, a quarter sample and the rectangle's closed/planar semantics. |
| Tests | Host test for the ellipse quarter point, closed/planar ellipse and rectangle, one Undo, and the non-unit-frame / zero-radius / zero-size refusals; native DLL test for the same through the packaged plugin. |

## Semantics fixed by this slice

- The frame is caller-supplied and must be orthonormal: `|x| = |y| = 1` and `x . y = 0`.
  Otherwise the frame is a `curve` refusal rather than silently re-orthonormalized.
- Ellipses are exact rational-quadratic NURBS (the document has no analytic ellipse kind),
  with radii along `x` and `y`; a non-positive radius is a `curve` refusal.
- Rectangles are closed four-segment polycurves starting at `origin`, running `width` along
  `x` and `height` along `y`; the first and last point coincide, so `curve_semantics` reports
  them closed.
- Both publish as one object in the command's single transaction, obey the current-layer
  permission and budgets, and one Undo removes them.

## Measured batch

Windows x64, rustc 1.98.1, `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`,
`om9-app --no-default-features` (standalone Controller/document/scene/archive path; no
OCCT adapter and no FreeCAD/Rhino host).

| Batch | Result | What it establishes |
|---|---|---|
| `cargo test -p om9-sdk -p om9-plugin-host -j 2` | 52 passed, 8 suites | Ellipse/rectangle construction and every refusal. |
| `cargo test -p om9-core -j 2` | 403 passed, 47 suites | The reused builders are unchanged. |
| `cargo test -p om9-app --no-default-features --lib session::plugins -j 2` | 5 passed, 135 filtered | The Controller publication/Undo boundary still holds. |
| `cargo test -p om9-app --no-default-features --test plugin_native -j 2 -- --ignored` | 13 passed | A real `.om9p` DLL: `CurveShapeDemo` creates the ellipse (quarter point `[0,2,0]`) and rectangle (4 segments, closed), the document grows from 3 to 5 objects in one Undo unit, and Undo/Redo reverses it. |
| `python tools/validate_plugin_sdk.py --target-dir L:/om9-target --output build/standalone-validation/ui-artifacts/20261021-plugin-sdk-curve-shapes` | PASS, 10 commands | Package/inspect, `CreateProfile`, cold `--inspect` hash equality, cancel/unknown-command preserving the `.om9` bytes, SDK ZIP built here and its example compiled offline outside the checkout. |
| `cargo check -p om9-app -p om9-plugin-host -j 2` (default features, incl. `occt-brep`) | exit 0 | The product-feature build still compiles. |
| `python tools/public_source_audit.py` | 8807 source files, 511 icon bindings, 510 SVG, 0 errors | No private/commercial source or binary was introduced. |
| `rustfmt --check` on the edited Rust files; `git diff --check` | PASS | Formatting and tracked whitespace are clean. |

The [JSON receipt](2026-10-21-m03-curve-shapes.json) records commands, exit codes, source
hashes and artifact fingerprints; `observed.json` remains under the ignored
`build/standalone-validation/ui-artifacts/20261021-plugin-sdk-curve-shapes/` run directory.

## Not proved here

- Polygon-by-points, spiral/helix, control-point editing, offset/fillet/blend/rebuild and
  intersections remain at their owners.
- Periodic/seam NURBS construction and polycurve segment parameters are still not exposed.
- macOS host and plugin execution are unverified; interactive UI acceptance was not
  performed. No manual or M09 gate was promoted.

## Next exact reference and action

Next document: [`SDK-INTEGRATION-PLAN.md`](../../OM9_Milestones/SDK-INTEGRATION-PLAN.md).
Next action: take `OM9-L2-01-12` file lifecycle (project state plus atomic save semantics), or
add polygon-by-points at the same owner.
