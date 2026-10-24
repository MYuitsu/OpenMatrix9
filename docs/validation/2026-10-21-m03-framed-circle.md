# Framed circle creation through SDK v1 — 2026-10-21

A plugin can now place a full circle on an arbitrary orthonormal frame, not only the world
XY plane. The slice belongs to `OM9-L2-02-04` (`OM9-L3-02-04-002`).

## Changed

| Layer | Change |
|---|---|
| Public SDK | `Host::add_circle_on_frame(label, center, x, y, radius)` and `ObjectAddCircleFrame`; capability entry. |
| Host | Builds `Curve::Circle { center, x, y, radius, domain: [0, tau], angle_domain: None }`, validates it with the curve record, and stages it through the shared add-output path (current layer, budgets, one transaction/Undo). |
| Example plugin | `CurveShapeDemo` also creates a circle on the XZ plane and reports its info. |
| Tests | Host test for the XZ circle (start point along `x`, quarter point along `y`, closed/planar) and the zero-radius / non-unit-frame refusals; native DLL test asserts the framed circle and the 3→7 object count. |

## Semantics fixed by this slice

- The native parameter is the angle in radians with domain `[0, tau]`, so `evaluate_curve`
  and `curve_semantics` behave exactly like the existing world-XY circle.
- The frame must be orthonormal and the radius positive; a malformed frame is a `curve`
  refusal and is never silently re-orthonormalized.
- The circle publishes as one object in the command's single transaction; one Undo removes it.

## Measured batch

Windows x64, rustc 1.98.1, `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`,
`om9-app --no-default-features` (standalone Controller/document/scene/archive path; no
OCCT adapter and no FreeCAD/Rhino host).

| Batch | Result | What it establishes |
|---|---|---|
| `cargo test -p om9-sdk -p om9-plugin-host -j 2` | 58 passed, 8 suites | Framed circle creation and refusals. |
| `cargo test -p om9-core -j 2` | 403 passed, 47 suites | The reused curve record is unchanged. |
| `cargo test -p om9-app --no-default-features --lib session::plugins -j 2` | 5 passed, 135 filtered | The Controller publication/Undo boundary still holds. |
| `cargo test -p om9-app --no-default-features --test plugin_native -j 2 -- --ignored` | 17 passed | A real `.om9p` DLL: the XZ circle appears alongside the ellipse/rectangle/polygon and the document grows 3→7 in one Undo unit. |
| `python tools/validate_plugin_sdk.py --target-dir L:/om9-target --output build/standalone-validation/ui-artifacts/20261021-plugin-sdk-frame-circle` | PASS, 10 commands | Package/inspect, `CreateProfile`, cold `--inspect` hash equality, cancel/unknown-command preserving the `.om9` bytes, SDK ZIP built here and its example compiled offline outside the checkout. |
| `cargo check -p om9-app -p om9-plugin-host -j 2` (default features, incl. `occt-brep`) | exit 0 | The product-feature build still compiles. |
| `python tools/public_source_audit.py` | 8819 source files, 511 icon bindings, 510 SVG, 0 errors | No private/commercial source or binary was introduced. |
| `rustfmt --check` on the edited Rust files; `git diff --check` | PASS | Formatting and tracked whitespace are clean. |

The [JSON receipt](2026-10-21-m03-framed-circle.json) records commands, exit codes, source
hashes and artifact fingerprints; `observed.json` remains under the ignored
`build/standalone-validation/ui-artifacts/20261021-plugin-sdk-frame-circle/` run directory.

## Not proved here

- Ellipse on an arbitrary frame (only the `add_ellipse` frame), polygon arcs, spiral/helix and
  curve editing remain at their owners.
- macOS host and plugin execution are unverified; interactive UI acceptance was not
  performed. No manual or M09 gate was promoted.

## Next exact reference and action

Next document: [`SDK-INTEGRATION-PLAN.md`](../../OM9_Milestones/SDK-INTEGRATION-PLAN.md).
Next action: expose the recorded change feed at M01 `01-03` L3-011, or return to M02 for
subobject/pick query services.
