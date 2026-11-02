# OM9-L3-03-21-001 — Horizontal Dimension — menu binding

- Owner: `OM9-L2-03-21`
- Command id: `OM9-MEASURE-004`
- Command line: `Dim`
- Annotation kind: `Kind::Horizontal`
- Selection rule: nothing selected
- Body: `crates/om9-app/src/session/annotation.rs`
- Model owner: `crates/om9-core/src/annotation.rs` (`Annotation`, `Kind`, `layout`, slot `annotation::SLOT`)

## What the row does

three typed points place a horizontal dimension: the two witness points fix the measured span and the third fixes the text position, and the value is the |dx| of the span in the annotation's own plane.

## Input contract

| step | prompt | input |
| --- | --- | --- |
| 1 | `First witness/leader tip: ` | typed input |
| 2 | `Second witness/leader elbow: ` | typed input |
| 3 | `Annotation text position: ` | typed input |

Points are resolved through the CPlane captured when the command started, and the record stores
that plane's axes so a later projection cannot reinterpret the annotation.

## What it refuses

- fewer than three points never creates an annotation: the command keeps asking, and cancelling leaves the document revision unchanged
- A locked object is refused by the permission check before any option is shown.

## Limits of this row

- the dimension measures along the CPlane X axis captured at start, not along the dragged direction
- the text position is a point, not an offset: moving it later is the Edit Dimension / Recenter rows

## Milestone ownership

Implemented in `OM9-L2-03-21` itself: the typed owner is `om9_core::annotation`, and the row adds
no second geometry kernel. The manual UI gate stays open (see the evidence file).

## Verification

- `cargo test -p om9-app --test annotation` → 8 passed
- `cargo test -p om9-app --test annotation_ui` → 3 passed
- `cargo test -p om9-app --test annotation five_dimension_families_have_independent_value_oracles_and_one_undo` (this row's oracle)

The golden case: two witness points 10 mm apart in X and 3 mm apart in Y -> the label reads 10, one Undo entry.
