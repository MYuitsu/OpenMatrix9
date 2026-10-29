# OM9-L3-04-01-022 — Transform · Array Along Curve — menu binding

- Owner: `OM9-L2-04-01`
- Command id: `OM9-TRANSFORM-021`
- Authored script: `! _ArrayCrv`
- Menu: Transform ▸ Array Along Curve (`MainMenu.ini [Menu10]`, icon from `menu_catalog::TRANSFORM`)
- Body: `crates/om9-app/src/session/transform.rs`

## What the row does

The stations are measured along the path: `curve_ops::arc_length` gives the total
length and each copy is placed at an equal share of it, found by bisection on
`curve_ops::length_between`. The source stays at the start of the curve and the copies are
committed as `Mutation::Insert`, so the whole array is one Undo entry.

## Selection rule

The path curve comes from the selection: exactly one selected curve is the path and every
other selected object is transformed along it. The row is refused before the first prompt
when that split is not possible.

## Input contract

| step | prompt | input |
| --- | --- | --- |
| 1 | `Number of items: ` | number |

Every point is resolved through the CPlane captured at start; every number is validated at
its own prompt by `check_number`.

## What it refuses

- No selection → `Error::Missing("transform selection")`, before any prompt.
- No curve in the selection → `Error::Missing("path curve selection")`.
- More than one curve in the selection → `Error::Invalid("one path curve")`.
- Only curves in the selection → `Error::Missing("select the objects and the path curve together")`.
- `Error::Invalid("array count")` for a count below two, a fractional count, or more than `MAX_CURVE_ITEMS` (512).
- `Error::Invalid("array path length")` for a path with zero length.

## Limits of this row

- Equal arc-length spacing across the whole path: there is no spacing-in-millimetres form yet and no `Roadlike` orientation option.
- Copies keep the source orientation: they are translated to their station, not rotated to the path tangent.
- Copies are new objects; there is no history-linked instance form.
- The path curve comes from the selection, because this shell has no mid-command object pick.
- One run is capped at 512 items: every station costs a bisection on the owner length service.

## Verification

- `cargo test -p om9-app --test transform_menu m03_04_01_022_array_along_curve_spaces_copies_by_arc_length`
- `cargo test -p om9-app --lib -j 2` (unit tests for the station helpers and scalar rules)

The golden case: box at the origin on a path from (0,0,0) to (10,0,0), count 3 → three boxes at the measured stations 0, 5 and 10 mm, one Undo entry.
