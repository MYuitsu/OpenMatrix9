# Curve definition and closed/planar semantics through SDK v1 — 2026-10-21

This extends the curve read surface at the NURBS semantics owner (`OM9-L2-02-01`): a plugin
can now read NURBS control data (poles, weights, knots, seam), an arc angle domain, and
whether a curve is closed and planar under an explicit or the document's own tolerance. Both
services reuse the owner's records and `rhino_semantics::classify_curve`; no second
classification rule is added.

## Changed

| Layer | Change |
|---|---|
| Public SDK | `CurveDefinition` (poles/weights/knots/degree/domain/periodic/reversed/seam, arc angle domain, polycurve segment count) and `CurveSemantics` (closed, planar, endpoints, tolerance); `Host::curve_definition` and `Host::curve_semantics`; two new requests and capability entries. |
| Host | Control data is bounded (8192 poles / 16384 knots) and a larger curve is a `budget` refusal. Semantics uses an explicit `tolerance_mm` or `Document::effective_tolerance`; a document that states no tolerance is a `tolerance` refusal and a curve whose sampled length is at or below the tolerance is a `curve` refusal (degenerate), never reclassified. |
| Example plugin | `CurveDemo` gained `{"definition":true}` and `{"semantics":true}`. |
| Tests | Host test for NURBS control data, arc angle domain, closed circle/polyline, explicit tolerance and every refusal; the native DLL test asserts line definition (no control data), circle definition (no arc domain), line open/planar and polyline closed at the template tolerance. |

## Semantics fixed by this slice

- Definitions are object-local and bounded; a truncated definition is never returned.
- Closedness is `distance(first, last) <= tolerance` on the owner's 33-sample curve; planarity
  uses the owner's Newell-normal rule. The tolerance used is reported back.
- A missing document tolerance refuses the implicit measurement; the plugin must pass an
  explicit tolerance instead of receiving an invented default.
- A curve shorter than the tolerance is refused as degenerate rather than reclassified as a
  point or a closed curve.
- Reading a definition or semantics stages no mutation.

## Measured batch

Windows x64, rustc 1.98.1, `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`,
`om9-app --no-default-features` (standalone Controller/document/scene/archive path; no
OCCT adapter and no FreeCAD/Rhino host).

| Batch | Result | What it establishes |
|---|---|---|
| `cargo test -p om9-sdk -p om9-plugin-host -j 2` | 48 passed, 8 suites | Definitions, semantics, bounded control data and the refusal codes. |
| `cargo test -p om9-core -j 2` | 403 passed, 47 suites | The reused classifier is unchanged. |
| `cargo test -p om9-app --no-default-features --lib session::plugins -j 2` | 5 passed, 135 filtered | The Controller publication/Undo boundary still holds. |
| `cargo test -p om9-app --no-default-features --test plugin_native -j 2 -- --ignored` | 10 passed | A real `.om9p` DLL: line definition has no poles/knots, circle definition has no arc domain, line is open and planar at 0.001 mm, and the square polyline is closed. |
| `python tools/validate_plugin_sdk.py --target-dir L:/om9-target --output build/standalone-validation/ui-artifacts/20261021-plugin-sdk-curve-semantics` | PASS, 10 commands | Package/inspect, `CreateProfile`, cold `--inspect` hash equality, cancel/unknown-command preserving the `.om9` bytes, SDK ZIP built here and its example compiled offline outside the checkout. |
| `cargo check -p om9-app -p om9-plugin-host -j 2` (default features, incl. `occt-brep`) | exit 0 | The product-feature build still compiles. |
| `python tools/public_source_audit.py` | 8799 source files, 511 icon bindings, 510 SVG, 0 errors | No private/commercial source or binary was introduced. |
| `rustfmt --check` on the edited Rust files; `git diff --check` | PASS | Formatting and tracked whitespace are clean. |

The [JSON receipt](2026-10-21-m01-02-01-curve-definition.json) records commands, exit codes,
source hashes and artifact fingerprints; `observed.json` remains under the ignored
`build/standalone-validation/ui-artifacts/20261021-plugin-sdk-curve-semantics/` run
directory.

## Not proved here

- NURBS weights/knots are readable but not writable; curve construction, editing, offset,
  fillet/blend, rebuild and intersections remain at their owners.
- Polycurve segment parameters are still not exposed.
- macOS host and plugin execution are unverified; interactive UI acceptance was not
  performed. No manual or M09 gate was promoted.

## Next exact reference and action

Next document: [`SDK-INTEGRATION-PLAN.md`](../../OM9_Milestones/SDK-INTEGRATION-PLAN.md).
Next action: expose curve construction (`add_arc`/`add_nurbs`) at the same owner, or take
`OM9-L2-01-12` file lifecycle.
