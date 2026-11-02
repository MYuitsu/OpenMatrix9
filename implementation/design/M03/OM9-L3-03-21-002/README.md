# OM9-L3-03-21-002 — Vertical Dimension — menu binding

- Owner: `OM9-L2-03-21`
- Command id: `OM9-MEASURE-006`
- Command line: `DimVertical`
- Annotation kind: `Kind::Vertical`
- Selection rule: nothing selected
- Body: `crates/om9-app/src/session/annotation.rs`
- Model owner: `crates/om9-core/src/annotation.rs` (`Annotation`, `Kind`, `layout`, slot `annotation::SLOT`)

## What the row does

three typed points place a vertical dimension; the value is the |dy| of the witness span, so a diagonal pair still reports its vertical projection.

## Input contract

| step | prompt | input |
| --- | --- | --- |
| 1 | `First witness/leader tip: ` | typed input |
| 2 | `Second witness/leader elbow: ` | typed input |
| 3 | `Annotation text position: ` | typed input |

Points are resolved through the CPlane captured when the command started, and the record stores
that plane's axes so a later projection cannot reinterpret the annotation.

## What it refuses

- a vertical dimension shares the horizontal refusal: no third point, no annotation
- A locked object is refused by the permission check before any option is shown.

## Limits of this row

- horizontal and vertical differ only in the measured axis, so one witness pair can be dimensioned twice with two different values
- the same CPlane rule applies

## Milestone ownership

Implemented in `OM9-L2-03-21` itself: the typed owner is `om9_core::annotation`, and the row adds
no second geometry kernel. The manual UI gate stays open (see the evidence file).

## Verification

- `cargo test -p om9-app --test annotation` → 8 passed
- `cargo test -p om9-app --test annotation_ui` → 3 passed
- `cargo test -p om9-app --test annotation five_dimension_families_have_independent_value_oracles_and_one_undo` (this row's oracle)

The golden case: witness points 10 mm apart in X and 3 mm apart in Y -> the label reads 3.
