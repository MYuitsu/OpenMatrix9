# Curve information and evaluation through SDK v1 — 2026-10-21

A plugin can now read a curve's identity and evaluate it with derivatives, using the
geometry owner's existing evaluator. The slice belongs to the NURBS curve semantics owner
(`OM9-L2-02-01`, `OM9-L3-02-01-002`) and reuses `evaluation::evaluate_curve` instead of
adding a second evaluator.

## Changed

| Layer | Change |
|---|---|
| Public SDK | `CurveInfo` (kind, native domain, NURBS degree/pole count/periodic/reversed, polycurve segment count), `CurveSample` (parameter, local point, D1..D3) and `Host::curve_info` / `Host::evaluate_curve`; two new `HostRequest` methods and capability entries. |
| Host | `curve_info` classifies line/circle/NURBS/polycurve from the entity geometry; `evaluate_curve` runs `om9_core::evaluation::evaluate_curve` at native parameters with `Transform::identity()` (object-local mm), at most 1024 samples and order ≤ 3, and refuses a polycurve because it needs an explicit segment parameter. |
| Example plugin | `CurveDemo` reads the identity and samples start/middle/end with D1, or a single point with `{"sample":true}`. |
| Tests | Host test for line/circle/NURBS identity and evaluation plus every refusal (point object, out-of-domain parameter, order 4, polycurve, sample budget); a native DLL test asserting the line midpoint/derivative, the circle midpoint/derivative and the polycurve refusal with no document change. |

## Semantics fixed by this slice

- Coordinates are object-local millimetres: the object placement is not applied, so a plugin
  composes it exactly once (the same split as `object_geometry` and `ObjectInfo.placement`).
- Parameters are native domain values, not 0..1 fractions; `curve_info().domain` is the
  authority and an out-of-domain parameter is a `curve` refusal.
- Derivative order is D0-D3; a higher order is `unsupported`. At most 1024 samples are
  evaluated per call.
- `polycurve` reports its segment count and no single domain; evaluation is refused rather
  than inventing a global parameter.
- Reading and evaluating stage no mutation: the document, scene and Undo stay untouched.

## Measured batch

Windows x64, rustc 1.98.1, `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`,
`om9-app --no-default-features` (standalone Controller/document/scene/archive path; no
OCCT adapter and no FreeCAD/Rhino host).

| Batch | Result | What it establishes |
|---|---|---|
| `cargo test -p om9-sdk -p om9-plugin-host -j 2` | 47 passed, 8 suites | Curve identity/evaluation and the refusal codes, on top of every existing host contract. |
| `cargo test -p om9-core -j 2` | 403 passed, 47 suites | The reused evaluator is unchanged. |
| `cargo test -p om9-app --no-default-features --lib session::plugins -j 2` | 5 passed, 135 filtered | The Controller publication/Undo boundary still holds. |
| `cargo test -p om9-app --no-default-features --test plugin_native -j 2 -- --ignored` | 10 passed | A real `.om9p` DLL: line domain `[0,1]`, midpoint `[5,0,0]` with D1 `[10,0,0]`; circle midpoint `[-5,0,0]` with D1 `-5y`; polycurve refused; no document change and one Undo unit from `CreateProfile`. |
| `python tools/validate_plugin_sdk.py --target-dir L:/om9-target --output build/standalone-validation/ui-artifacts/20261021-plugin-sdk-curve` | PASS, 10 commands | Package/inspect, `CreateProfile`, cold `--inspect` hash equality, cancel/unknown-command preserving the `.om9` bytes, SDK ZIP built here and its example compiled offline outside the checkout. |
| `cargo check -p om9-app -p om9-plugin-host -j 2` (default features, incl. `occt-brep`) | exit 0 | The product-feature build still compiles. |
| `python tools/public_source_audit.py` | 8797 source files, 511 icon bindings, 510 SVG, 0 errors | No private/commercial source or binary was introduced. |
| `rustfmt --check` on the edited Rust files; `git diff --check` | PASS | Formatting and tracked whitespace are clean. |

The [JSON receipt](2026-10-21-m03-curve-evaluation.json) records commands, exit codes, source
hashes and artifact fingerprints; `observed.json` remains under the ignored
`build/standalone-validation/ui-artifacts/20261021-plugin-sdk-curve/` run directory.

## Not proved here

- Only line, circle/arc (angle domain) and NURBS evaluation is exposed. Curve construction,
  editing, offset, fillet/blend, rebuild, intersections and polycurve segment parameters
  remain at their owners.
- NURBS rationality is not reported as a separate flag; weights participate in evaluation
  but a plugin cannot yet read or write them.
- macOS host and plugin execution are unverified; interactive UI acceptance was not
  performed. No manual or M09 gate was promoted.

## Next exact reference and action

Next document: [`SDK-INTEGRATION-PLAN.md`](../../OM9_Milestones/SDK-INTEGRATION-PLAN.md).
Next action: expose the remaining curve read services (weights/knots/seam/closure, split
parameters) at the same owner, or take `OM9-L2-01-12` file lifecycle.
