# Curve segments through SDK v1 — 2026-10-21

A plugin can now list a curve's segments and evaluate one segment by index, so a polycurve
(rectangle, polygon, joined curve) no longer needs an opaque refusal. A native curve is one
segment. Owner: `OM9-L2-02-04` (`OM9-L3-02-04-005`).

## Changed

| Layer | Change |
|---|---|
| Public SDK | `CurveSegment { kind, domain, angle_domain, reversed }`, `Host::curve_segments`, `Host::evaluate_curve_segment(id, index, parameters, order)`; two requests and capability entries. |
| Host | Lists a polycurve's segments or the single native segment; refuses a nested polycurve and an out-of-range index; evaluation reuses the same D0-D3 loop, object-local coordinates and 1024-sample bound as `evaluate_curve`. |
| Example plugin | `CurveShapeDemo` reports the rectangle's 4 segments and its first segment midpoint. |
| Tests | Host test for 4 line segments of a rectangle, segment 0 midpoint/derivative and the out-of-range index refusal; native DLL test asserts 4 segments and the `[5,0,0]` midpoint. |

## Semantics fixed by this slice

- `curve_segments` reports each segment's kind, native domain and arc angle domain; a native
  curve is a one-element list. A nested polycurve is `unsupported` rather than flattened.
- `evaluate_curve_segment` addresses the list; `0` on a native curve evaluates the curve
  itself and a larger index is a `curve` refusal.
- Segment evaluation uses the same object-local millimetres, D0-D3 order, finite-parameter
  admission and 1024-sample bound as `evaluate_curve`.
- Both reads stage no mutation.

## Measured batch

Windows x64, rustc 1.98.1, `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`,
`om9-app --no-default-features` (standalone path; no OCCT adapter and no FreeCAD/Rhino host).

| Batch | Result | What it establishes |
|---|---|---|
| `cargo test -p om9-sdk -p om9-plugin-host -j 2` | 61 passed, 8 suites | Segment listing/evaluation and the index refusal. |
| `cargo test -p om9-core -j 2` | 403 passed, 47 suites | No core change. |
| `cargo test -p om9-app --no-default-features --lib session::plugins -j 2` | 5 passed, 135 filtered | Controller publication/Undo boundary holds. |
| `cargo test -p om9-app --no-default-features --test plugin_native -j 2 -- --ignored` | 19 passed | Real `.om9p`: the rectangle reports 4 segments and segment 0 midpoint `[5,0,0]`. |
| `python tools/validate_plugin_sdk.py --target-dir L:/om9-target --output build/standalone-validation/ui-artifacts/20261021-plugin-sdk-segments` | PASS, 10 commands | Package/inspect, cold hash equality, cancel/unknown preserving bytes, offline SDK build. |
| `cargo check -p om9-app -p om9-plugin-host -j 2` (default features) | exit 0 | Product-feature build compiles. |
| `python tools/public_source_audit.py` | 8825 source files, 511 icon bindings, 510 SVG, 0 errors | No private/commercial source or binary. |
| `rustfmt --check` + `git diff --check` | PASS | Formatting/whitespace clean. |

The [JSON receipt](2026-10-21-m03-curve-segments.json) records hashes and artifacts;
`observed.json` stays under the ignored run directory.

## Not proved here

- Segment editing/explode/split, segment reversal for arc direction, and arbitrary-frame
  ellipse remain at their owners.
- A nested polycurve is refused, not flattened.
- macOS host and plugin execution are unverified; interactive UI acceptance was not
  performed. No manual or M09 gate was promoted.

## Next exact reference and action

Next document: [`SDK-INTEGRATION-PLAN.md`](../../OM9_Milestones/SDK-INTEGRATION-PLAN.md).
Next action: broaden the recorded feed to host commands, or return to M02 for subobject/pick
query services.
