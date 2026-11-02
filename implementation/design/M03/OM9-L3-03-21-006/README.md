# OM9-L3-03-21-006 — Diameter Dimension — menu binding

- Owner: `OM9-L2-03-21`
- Command id: `OM9-MEASURE-010`
- Command line: `DimDiameter`
- Annotation kind: `Kind::Diameter`
- Selection rule: exactly one circle or arc curve
- Body: `crates/om9-app/src/session/annotation.rs`
- Model owner: `crates/om9-core/src/annotation.rs` (`Annotation`, `Kind`, `layout`, slot `annotation::SLOT`)

## What the row does

one selected circle or arc is the measured object: the record seeds the centre and one rim point from the curve itself (centre plus the radius along the curve's X axis), and the row only asks for the text position; the value is twice the radius.

## Input contract

| step | prompt | input |
| --- | --- | --- |
| 1 | `Annotation text position: ` | typed input |

Points are resolved through the CPlane captured when the command started, and the record stores
that plane's axes so a later projection cannot reinterpret the annotation.

## What it refuses

- no selection, more than one object, or a selected curve that is not a circle/arc -> Error::Invalid ("select one circle or arc", "radial dimension needs a circle/arc")
- A locked object is refused by the permission check before any option is shown.

## Limits of this row

- the geometry is read, not copied: the annotation stores the centre, a rim point and the frame, so a later Edit Dimension works on the same record
- the source stamp is kept so the row records where the shape came from

## Milestone ownership

Implemented in `OM9-L2-03-21` itself: the typed owner is `om9_core::annotation`, and the row adds
no second geometry kernel. The manual UI gate stays open (see the evidence file).

## Verification

- `cargo test -p om9-app --test annotation` → 8 passed
- `cargo test -p om9-app --test annotation_ui` → 3 passed
- `cargo test -p om9-app --test annotation diameter_and_radius_use_the_placed_circle_and_keep_source` (this row's oracle)

The golden case: circle of radius 5 at (2,2) -> the label reads 10, and editing the circle's radius later does not silently re-write the stored annotation.
