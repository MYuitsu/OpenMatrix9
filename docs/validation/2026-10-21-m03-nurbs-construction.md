# Control-point NURBS construction through SDK v1 — 2026-10-21

This completes the construction scope of `OM9-L3-02-04-003` for the plugin boundary: a
plugin can publish a control-point NURBS record, not only an interpolated one. The record is
validated by the geometry owner's `NurbsCurve::validate` and staged through the same
single-transaction output path as every other `add_*` service.

## Changed

| Layer | Change |
|---|---|
| Public SDK | `Host::add_nurbs_curve(label, degree, poles, weights, knots)`; one new request and capability entry. |
| Host | Bounds the record (8192 poles / 16384 knots), requires one weight per pole, `poles > degree`, enough knots, derives the clamped domain `[knots[degree], knots[poles.len()]]`, then validates with `NurbsCurve::validate`. `periodic`, `reversed` and `seam` construction is not exposed. |
| Example plugin | `CurveBuildDemo` also publishes a cubic control-point NURBS and returns its definition. |
| Tests | Host test for a valid cubic (degree/poles/domain, endpoint interpolation, one Undo) and the weight-count, zero-degree, short-knot and negative-weight refusals; native DLL test asserts the third curve and the 3→6 object / Undo / Redo sequence. |

## Semantics fixed by this slice

- The record rule is the document's: one weight per pole, `poles + degree + 1` non-decreasing
  knots, finite positive weights, and a clamped domain derived from the knot vector. A
  malformed record is a `curve` refusal and publishes nothing.
- Construction is static: `periodic`, `reversed` and `seam` are not exposed, so a plugin
  cannot create a periodic seam record through this call.
- The curve publishes as one object in the command's single transaction; one Undo removes it.
- Control data is bounded, so a huge record is a `budget` refusal before allocation.

## Measured batch

Windows x64, rustc 1.98.1, `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`,
`om9-app --no-default-features` (standalone Controller/document/scene/archive path; no
OCCT adapter and no FreeCAD/Rhino host).

| Batch | Result | What it establishes |
|---|---|---|
| `cargo test -p om9-sdk -p om9-plugin-host -j 2` | 50 passed, 8 suites | Record validation, one transaction and every refusal. |
| `cargo test -p om9-core -j 2` | 403 passed, 47 suites | The reused `NurbsCurve::validate` is unchanged. |
| `cargo test -p om9-app --no-default-features --lib session::plugins -j 2` | 5 passed, 135 filtered | The Controller publication/Undo boundary still holds. |
| `cargo test -p om9-app --no-default-features --test plugin_native -j 2 -- --ignored` | 11 passed | A real `.om9p` DLL: `CurveBuildDemo` adds a cubic control-point NURBS (4 poles) after the arc and spline; the document grows from 3 to 6 objects in one Undo unit and reverses with Undo/Redo. |
| `python tools/validate_plugin_sdk.py --target-dir L:/om9-target --output build/standalone-validation/ui-artifacts/20261021-plugin-sdk-nurbs-build` | PASS, 10 commands | Package/inspect, `CreateProfile`, cold `--inspect` hash equality, cancel/unknown-command preserving the `.om9` bytes, SDK ZIP built here and its example compiled offline outside the checkout. |
| `cargo check -p om9-app -p om9-plugin-host -j 2` (default features, incl. `occt-brep`) | exit 0 | The product-feature build still compiles. |
| `python tools/public_source_audit.py` | 8803 source files, 511 icon bindings, 510 SVG, 0 errors | No private/commercial source or binary was introduced. |
| `rustfmt --check` on the edited Rust files; `git diff --check` | PASS | Formatting and tracked whitespace are clean. |

The [JSON receipt](2026-10-21-m03-nurbs-construction.json) records commands, exit codes,
source hashes and artifact fingerprints; `observed.json` remains under the ignored
`build/standalone-validation/ui-artifacts/20261021-plugin-sdk-nurbs-build/` run directory.

## Not proved here

- Periodic/reversed/seam NURBS construction, point/knot editing, ellipse/polygon helpers,
  offset/fillet/blend/rebuild and intersections remain at their owners.
- Polycurve segment parameters and surface/BRep semantics are still not exposed.
- macOS host and plugin execution are unverified; interactive UI acceptance was not
  performed. No manual or M09 gate was promoted.

## Next exact reference and action

Next document: [`SDK-INTEGRATION-PLAN.md`](../../OM9_Milestones/SDK-INTEGRATION-PLAN.md).
Next action: take `OM9-L2-01-12` file lifecycle (host file service with atomic save/open and
no document swap inside a command), or continue curve editing at the curve owner.
