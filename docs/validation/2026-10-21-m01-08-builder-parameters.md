# Typed builder parameters through SDK v1 (OM9-L2-01-08 L3-004/005/006) — 2026-10-21

This completes the units owner's SDK-eligible scope. The document owner already implements
the typed parameter rule: `request::Dimension`, `request::Value` (`Number`/`Quantity`/
`Choice`/`Boolean`/`Text`), `request::OptionDomain::check` (bounds, step, count integer,
text size) and `command_schema::CommandSchema::apply` (defaults, required/unknown names,
typed defaults). This slice exports that contract to plugins instead of adding a second
validator.

## Changed

| Layer | Change |
|---|---|
| Public DTOs | `Dimension` (length/angle/ratio/count), `ParameterDomain` (number with canonical-unit bounds + step, choice, boolean, text size), `ParameterSpec` (name, domain, typed default, required) and `ParameterValue` (kind-tagged `length`/`angle`/`ratio`/`count`/`choice`/`boolean`/`text`). |
| Public methods | `Host::validate_parameter` (one typed value against its domain) and `Host::apply_parameters` (defaults + required/unknown + every value, returning the filled typed map); two new `HostRequest` methods and capability entries. |
| Host dispatch | The host converts the SDK DTOs to `request::OptionDomain`/`request::Value` and runs `OptionDomain::check` / `CommandSchema::apply`, so bounds, step, count, choices and text-budget rules are the document owner's own. One call is bounded to 128 declarations and 128 supplied values. |
| Wire contract | The `kind` tag is the unit: `{"kind":"length","millimeters":2.5}`, `{"kind":"angle","degrees":90}`, `{"kind":"count","value":6}`. A bare JSON number is refused (`parameter`) because a quantity needs a unit, and an angle record cannot deserialize as a length. |
| Example plugin | `ParameterDemo` declares a required length with bounds, a count default and a choice default, applies them and returns the typed map. |
| Tests | Host tests for valid/invalid parameters and the schema bounds, an SDK wire-form test, and a native DLL test that also proves the calls stage no mutation. |

## Semantics fixed by this slice

- Bounds are canonical: millimetres for a length, degrees for an angle. The minimum is
  exclusive unless `min_inclusive` is true; `max` is inclusive; `step` is measured from
  `min` (or zero) and a value off the grid is refused (`parameter`).
- A count must be a non-negative integer; a number outside the finite domain, a wrong
  dimension, a missing unit, a value outside a choice list, or text over `max_bytes` is a
  `parameter` refusal.
- `apply_parameters` fills declared defaults, refuses an unknown name, refuses a missing
  required parameter, refuses a schema whose default violates its own domain or declares
  both required and a default, and returns the typed canonical map.
- Validation stages no document mutation: Undo/Redo and the document stay untouched.
- The app does not yet have a builder parameter editor (that wiring belongs to M04/M01-02);
  the rule exported here is the same one command admission uses, not a parallel copy.

## Measured batch

Windows x64, rustc 1.98.1, `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`,
`om9-app --no-default-features` (standalone Controller/document/scene/archive path; no
OCCT adapter and no FreeCAD/Rhino host).

| Batch | Result | What it establishes |
|---|---|---|
| `cargo test -p om9-sdk -p om9-plugin-host -j 2` | 41 passed, 8 suites | Domain checks for length bounds/step, count integer, choice/text, defaults, required/unknown names, the 128 limits and the wire-form unit safety. |
| `cargo test -p om9-core -j 2` | 403 passed, 47 suites | The reused `OptionDomain`/`CommandSchema` rules are unchanged. |
| `cargo test -p om9-app --no-default-features --lib session::plugins -j 2` | 4 passed, 135 filtered | The Controller publication/Undo boundary still holds. |
| `cargo test -p om9-app --no-default-features --test plugin_native -j 2 -- --ignored` | 7 passed | A real `.om9p` DLL: `ParameterDemo` returns `radius` length 3 mm, `sides` count 6, `profile` "round" and stages nothing; a bare number, a wrong dimension, an out-of-range length and a missing required radius are all refused with no document change. |
| `python tools/validate_plugin_sdk.py --target-dir L:/om9-target --output build/standalone-validation/ui-artifacts/20261021-plugin-sdk-parameters` | PASS, 10 commands | Package/inspect, `CreateProfile`, cold `--inspect` hash equality, cancel/unknown-command preserving the `.om9` bytes, SDK ZIP built here and its example compiled offline outside the checkout. |
| `cargo check -p om9-app -p om9-plugin-host -j 2` (default features, incl. `occt-brep`) | exit 0 | The product-feature build still compiles. |
| `python tools/public_source_audit.py` | 8789 source files, 511 icon bindings, 510 SVG, 0 errors | No private/commercial source or binary was introduced. |
| `rustfmt --check` on the edited Rust files; `git diff --check` | PASS | Formatting and tracked whitespace are clean. |

The [JSON receipt](2026-10-21-m01-08-builder-parameters.json) records commands, exit codes,
source hashes and artifact fingerprints; `observed.json` remains under the ignored
`build/standalone-validation/ui-artifacts/20261021-plugin-sdk-parameters/` run directory.

## Not proved here

- The application has no builder parameter editor yet, so this batch proves the shared
  contract and the plugin path, not a UI parity comparison.
- `OM9-L3-01-08-004/005/006` are covered as typed values, range/step/count validation and
  unit-safe serialization; document tolerance editing and CPlane frame resolution remain
  open from the earlier units slice.
- macOS host and plugin execution are unverified; interactive UI acceptance was not
  performed. No manual or M09 gate was promoted.

## Next exact reference and action

Next document: [`SDK-INTEGRATION-PLAN.md`](../../OM9_Milestones/SDK-INTEGRATION-PLAN.md).
Next action: the units owner is complete for SDK-eligible scope; take the next plan-named
owner slice, `OM9-L2-01-12` file lifecycle (host file service with atomic save/open and no
document swap inside a command) or the M01 `03-07`/`03-10` command/UI contribution work.
