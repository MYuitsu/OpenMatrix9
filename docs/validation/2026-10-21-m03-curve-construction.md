# Arc and interpolated-curve construction through SDK v1 — 2026-10-21

A plugin can now create an arc through three points and an interpolated NURBS curve through
points, using the geometry owner's builders and the same output/transaction path as the
existing line/circle/polyline services. The slice belongs to the M03 curve creation owner
(`OM9-L2-02-04`, L3-002 and L3-003).

## Changed

| Layer | Change |
|---|---|
| Public SDK | `KnotMode` (chord length / uniform), `Host::add_arc(label, start, through, end)` and `Host::add_interpolated_curve(label, points, degree, mode)`; two new requests and capability entries. |
| Host | Both build with `curve_build::arc_through` / `curve_build::interpolate` and stage through one shared `add_output` path (current layer, permissions, budgets, one `Entity`). The previous `ObjectAdd` body was reduced to that shared path. |
| Example plugin | `CurveBuildDemo` creates an arc and a cubic interpolated spline in one command and reads their info/definition back. |
| Tests | Host test for the built arc (increasing angle domain, midpoint) and spline (degree/poles, endpoint interpolation), one transaction/Undo, layer placement, and the collinear-arc / even-degree / over-budget-label refusals; native DLL test asserts the same through the packaged plugin. |

## Semantics fixed by this slice

- `add_arc` returns a circle with `angle_domain` from `start` to `end` through the middle
  point and a `[0,1]` native domain; collinear or repeated points are a `curve` refusal.
- `add_interpolated_curve` follows the document rule: odd degree 1..=11 and smaller than the
  point count; an even or out-of-range degree is a `curve` refusal. `KnotMode` selects
  chord-length or uniform parameters, and the resulting curve passes through the points.
- Both stage one object on the document's current layer, obey the layer permission, label
  budget, object count and geometry budget, and publish with the command's single Undo unit.
- A refused build publishes nothing; the document and existing Redo are unchanged.

## Measured batch

Windows x64, rustc 1.98.1, `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`,
`om9-app --no-default-features` (standalone Controller/document/scene/archive path; no
OCCT adapter and no FreeCAD/Rhino host).

| Batch | Result | What it establishes |
|---|---|---|
| `cargo test -p om9-sdk -p om9-plugin-host -j 2` | 49 passed, 8 suites | Arc/spline construction, the shared output path, one transaction and every refusal. |
| `cargo test -p om9-core -j 2` | 403 passed, 47 suites | The reused builders are unchanged. |
| `cargo test -p om9-app --no-default-features --lib session::plugins -j 2` | 5 passed, 135 filtered | The Controller publication/Undo boundary still holds. |
| `cargo test -p om9-app --no-default-features --test plugin_native -j 2 -- --ignored` | 11 passed | A real `.om9p` DLL: `CurveBuildDemo` creates an arc (midpoint at `[0,5,0]`) and a cubic spline (4 poles, degree 3) in one Undo unit; the document grows from 3 to 5 objects and reverses with Undo/Redo. |
| `python tools/validate_plugin_sdk.py --target-dir L:/om9-target --output build/standalone-validation/ui-artifacts/20261021-plugin-sdk-curve-build` | PASS, 10 commands | Package/inspect, `CreateProfile`, cold `--inspect` hash equality, cancel/unknown-command preserving the `.om9` bytes, SDK ZIP built here and its example compiled offline outside the checkout. |
| `cargo check -p om9-app -p om9-plugin-host -j 2` (default features, incl. `occt-brep`) | exit 0 | The product-feature build still compiles. |
| `python tools/public_source_audit.py` | 8801 source files, 511 icon bindings, 510 SVG, 0 errors | No private/commercial source or binary was introduced. |
| `rustfmt --check` on the edited Rust files; `git diff --check` | PASS | Formatting and tracked whitespace are clean. |

The [JSON receipt](2026-10-21-m03-curve-construction.json) records commands, exit codes,
source hashes and artifact fingerprints; `observed.json` remains under the ignored
`build/standalone-validation/ui-artifacts/20261021-plugin-sdk-curve-build/` run directory.

## Not proved here

- Only arc-through-three-points and interpolated NURBS are exposed. Control-point NURBS
  construction, ellipse/polygon helpers, curve editing, offset, fillet/blend, rebuild and
  intersections remain at their owners.
- Polycurve segment parameters and surface/BRep semantics are still not exposed.
- macOS host and plugin execution are unverified; interactive UI acceptance was not
  performed. No manual or M09 gate was promoted.

## Next exact reference and action

Next document: [`SDK-INTEGRATION-PLAN.md`](../../OM9_Milestones/SDK-INTEGRATION-PLAN.md).
Next action: add control-point NURBS construction or take `OM9-L2-01-12` file lifecycle.
