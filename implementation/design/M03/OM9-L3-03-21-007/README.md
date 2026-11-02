# OM9-L3-03-21-007 — Radial Dimension — menu binding

- Owner: `OM9-L2-03-21`
- Command id: `OM9-MEASURE-011`
- Command line: `DimRadius`
- Annotation kind: `Kind::Radius`
- Selection rule: exactly one circle or arc curve
- Body: `crates/om9-app/src/session/annotation.rs`
- Model owner: `crates/om9-core/src/annotation.rs` (`Annotation`, `Kind`, `layout`, slot `annotation::SLOT`)

## What the row does

the radial twin of the diameter row: the same seeded centre and rim point, and the value is the radius itself.

## Input contract

| step | prompt | input |
| --- | --- | --- |
| 1 | `Annotation text position: ` | typed input |

Points are resolved through the CPlane captured when the command started, and the record stores
that plane's axes so a later projection cannot reinterpret the annotation.

## What it refuses

- the same three refusals as the diameter row
- A locked object is refused by the permission check before any option is shown.

## Limits of this row

- diameter and radius share one seeding rule, so the two rows can never disagree about which circle they measured
- the row keeps the source stamp

## Milestone ownership

Implemented in `OM9-L2-03-21` itself: the typed owner is `om9_core::annotation`, and the row adds
no second geometry kernel. The manual UI gate stays open (see the evidence file).

## Verification

- `cargo test -p om9-app --test annotation` → 8 passed
- `cargo test -p om9-app --test annotation_ui` → 3 passed
- `cargo test -p om9-app --test annotation diameter_and_radius_use_the_placed_circle_and_keep_source` (this row's oracle)

The golden case: circle of radius 5 -> the label reads 5.
