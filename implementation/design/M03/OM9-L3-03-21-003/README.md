# OM9-L3-03-21-003 — Angle Dimension — menu binding

- Owner: `OM9-L2-03-21`
- Command id: `OM9-MEASURE-007`
- Command line: `DimAngle`
- Annotation kind: `Kind::Angular`
- Selection rule: nothing selected
- Body: `crates/om9-app/src/session/annotation.rs`
- Model owner: `crates/om9-core/src/annotation.rs` (`Annotation`, `Kind`, `layout`, slot `annotation::SLOT`)

## What the row does

four typed points place an angular dimension: the vertex, one point on each ray and the text position, and the value is the angle between the two rays in degrees.

## Input contract

| step | prompt | input |
| --- | --- | --- |
| 1 | `Angle vertex: ` | typed input |
| 2 | `First ray point: ` | typed input |
| 3 | `Second ray point: ` | typed input |
| 4 | `Annotation text position: ` | typed input |

Points are resolved through the CPlane captured when the command started, and the record stores
that plane's axes so a later projection cannot reinterpret the annotation.

## What it refuses

- a zero-length ray is refused as an invalid witness instead of reporting a made-up angle
- A locked object is refused by the permission check before any option is shown.

## Limits of this row

- the angle is unsigned: it reports the smaller rotation between the two rays
- the rays live in the annotation plane, so a 3D pair is measured through the CPlane projection

## Milestone ownership

Implemented in `OM9-L2-03-21` itself: the typed owner is `om9_core::annotation`, and the row adds
no second geometry kernel. The manual UI gate stays open (see the evidence file).

## Verification

- `cargo test -p om9-app --test annotation` → 8 passed
- `cargo test -p om9-app --test annotation_ui` → 3 passed
- `cargo test -p om9-app --test annotation five_dimension_families_have_independent_value_oracles_and_one_undo` (this row's oracle)

The golden case: vertex (0,0), rays along +X and +Y -> the label reads 90.
