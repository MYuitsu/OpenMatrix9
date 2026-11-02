# Shared Polyline parameter interface — 2026-10-25

Scope: the eight existing Polyline parameters. The implementation stays in standalone safe Rust OM9. Other command families and their option interfaces are unchanged.

## Delivered interface

[`PolylineOptionInterface`](../../../crates/om9-app/src/polyline_options.rs) supplies a current snapshot of `PolylineCommandOption` records and activates a `PolylineOptionId`. Each record exposes its stable name, shortcut, kind and owner-derived value. The Polyline session owns availability and values; the UI does not duplicate parameter state or map letters to actions.

| Parameter | Shortcut | Kind / activation |
|---|---|---|
| PersistentClose | P | Toggle existing Yes/No setting |
| Mode | M | Switch existing Line/Arc mode |
| Helpers | H | Toggle existing Yes/No setting |
| Length | L | Request a positive numeric value after a Line point |
| Close | C | Close the curve in Line mode after three points |
| Direction | D | Request an arc direction point |
| Center | C | Request an arc center point |
| Undo | U | Undo the current point/parameter input |

The same snapshot formats prompt labels, underlines shortcuts, creates click hit targets and resolves full names/initials. Keyboard, pointer and typed-name activation call the same session parser. C is unambiguous because Close and Center are exposed in different stages. Unavailable parameters are rejected without mutation; during Length numeric entry the option snapshot is empty.

Accepted Length and picked Direction/Center values now appear in the prompt, e.g. `Length=5`, `Direction=1,0,0` and `Center=0,0,0`. Numeric/point input remains on the existing typed input path. Nonempty drafts, IME, repeat-key suppression, error feedback, protected history and whole-curve transaction boundaries remain covered by regression tests.

## Verification

- [Commands and results](commands.json): 216 app tests passed, zero failures; Clippy with `-D warnings`, scoped rustfmt and the default-feature debug build passed.
- `polyline_options_interface`: three Rust tests, including **24 parameter/input-route scenarios** (8 parameters x keyboard/pointer/typed name), stage/alias checks and invalid numeric input. These scenarios are not 24 separate Rust test functions.
- Geometry assertions verify the 5 mm constrained Line, Direction and Center arcs of radius 5 mm, Close topology and one document Undo transaction; pending Undo and invalid values create no document changes.
- Existing Polyline, protected Command, IME/paste, Line token and application library tests remain in the 216-test batch.
- [Native debug replay](native-debug.json) records real Windows P/H/Length/Mode/Direction/Center/Undo input and clean Escape cancellation on an isolated own-app profile.
- [Length value](../../../implementation/evidence/M01/polyline-parameter-interface-20261025/length-value.png), [Direction value](../../../implementation/evidence/M01/polyline-parameter-interface-20261025/direction-value.png), [Center value](../../../implementation/evidence/M01/polyline-parameter-interface-20261025/center-value.png).

The default-feature release build passed with `CARGO_TARGET_DIR=L:/om9-target` and `-j 2`. The release window separately accepted typed `Polyline`, a viewport point, a click on Length, numeric entry and immediate `Length=5` feedback; Escape returned to idle Command. [Release replay](native-release.json), [release screenshot](../../../implementation/evidence/M01/polyline-parameter-interface-20261025/release-length-value.png).

Owned replay profiles: `build/standalone-validation/ui-artifacts/polyline-parameter-interface-debug-20261025` and `build/standalone-validation/ui-artifacts/polyline-parameter-interface-release-20261025`. The clean debug window was closed; the release window was left open. Human M01/M03 sign-off remains unsigned.

Final public source audit, `git diff --check` and artifact pruning with `--apply` pass. Own-app screenshots are stored in the existing public evidence scope with provenance/hashes; audit policy is unchanged.

## Scope limits

This consolidates parameter control and presentation. Existing geometry/flag lifecycle semantics are preserved: Helpers tracking and continuously closed PersistentClose preview remain unfinished; the earlier scoped Arc/Center restrictions remain in [the previous geometry report](../2026-10-25-command-polyline-shortcuts/README.md). No SDK ABI, native adapter, reusable skill rule or publication is changed.
