# OM9-L3-03-21-005 — Aligned Dimension — menu binding

- Owner: `OM9-L2-03-21`
- Command id: `OM9-MEASURE-009`
- Command line: `DimAligned`
- Annotation kind: `Kind::Aligned`
- Selection rule: nothing selected
- Body: `crates/om9-app/src/session/annotation.rs`
- Model owner: `crates/om9-core/src/annotation.rs` (`Annotation`, `Kind`, `layout`, slot `annotation::SLOT`)

## What the row does

three typed points place an aligned dimension: the value is the true distance between the witness points, measured along their own direction.

## Input contract

| step | prompt | input |
| --- | --- | --- |
| 1 | `First witness/leader tip: ` | typed input |
| 2 | `Second witness/leader elbow: ` | typed input |
| 3 | `Annotation text position: ` | typed input |

Points are resolved through the CPlane captured when the command started, and the record stores
that plane's axes so a later projection cannot reinterpret the annotation.

## What it refuses

- the same invalid-witness refusal as the other linear rows
- A locked object is refused by the permission check before any option is shown.

## Limits of this row

- aligned is the only linear row that reports the full 3D distance when the pair leaves the CPlane
- the five linear rows share one oracle and one commit path, which is what the five-family test pins

## Milestone ownership

Implemented in `OM9-L2-03-21` itself: the typed owner is `om9_core::annotation`, and the row adds
no second geometry kernel. The manual UI gate stays open (see the evidence file).

## Verification

- `cargo test -p om9-app --test annotation` → 8 passed
- `cargo test -p om9-app --test annotation_ui` → 3 passed
- `cargo test -p om9-app --test annotation five_dimension_families_have_independent_value_oracles_and_one_undo` (this row's oracle)

The golden case: witness points (0,0) and (3,4) -> the label reads 5.
