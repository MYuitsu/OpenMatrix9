# OM9-L3-03-21-004 — Rotated Dimension — menu binding

- Owner: `OM9-L2-03-21`
- Command id: `OM9-MEASURE-008`
- Command line: `DimRotated`
- Annotation kind: `Kind::Rotated`
- Selection rule: nothing selected
- Body: `crates/om9-app/src/session/annotation.rs`
- Model owner: `crates/om9-core/src/annotation.rs` (`Annotation`, `Kind`, `layout`, slot `annotation::SLOT`)

## What the row does

three typed points place a rotated dimension: the measured direction is the witness direction itself, and the text rotation follows it.

## Input contract

| step | prompt | input |
| --- | --- | --- |
| 1 | `First witness/leader tip: ` | typed input |
| 2 | `Second witness/leader elbow: ` | typed input |
| 3 | `Annotation text position: ` | typed input |

Points are resolved through the CPlane captured when the command started, and the record stores
that plane's axes so a later projection cannot reinterpret the annotation.

## What it refuses

- an invalid witness is refused by name and the pending command keeps its question
- A locked object is refused by the permission check before any option is shown.

## Limits of this row

- the rotation is stored in degrees on the record, so Edit Dimension can change it later
- the dimension is planar by construction

## Milestone ownership

Implemented in `OM9-L2-03-21` itself: the typed owner is `om9_core::annotation`, and the row adds
no second geometry kernel. The manual UI gate stays open (see the evidence file).

## Verification

- `cargo test -p om9-app --test annotation` → 8 passed
- `cargo test -p om9-app --test annotation_ui` → 3 passed
- `cargo test -p om9-app --test annotation five_dimension_families_have_independent_value_oracles_and_one_undo` (this row's oracle)

The golden case: two witness points on a 45 degree diagonal -> the label reports the diagonal length and the text turns with it.
