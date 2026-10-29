# OM9-L3-03-21-013 — Recenter Dimension Text — menu binding

- Owner: `OM9-L2-03-21`
- Command id: `OM9-MEASURE-017`
- Command line: `DimRecenter`
- Annotation kind: `Kind::dimension kinds only`
- Selection rule: exactly one dimension
- Body: `crates/om9-app/src/session/annotation.rs`
- Model owner: `crates/om9-core/src/annotation.rs` (`Annotation`, `Kind`, `layout`, slot `annotation::SLOT`)

## What the row does

moves the stored text point back onto the annotation's own layout centre and commits immediately: there is no question to answer because the layout owner already knows the centre.

## Input contract

| step | prompt | input |
| --- | --- | --- |
| — | (no prompt: the layout centre is already known) | — |

Points are resolved through the CPlane captured when the command started, and the record stores
that plane's axes so a later projection cannot reinterpret the annotation.

## What it refuses

- a Text Block or Leader -> Error::Invalid("select a dimension"); no selection -> Error::Invalid("select one annotation"); a locked object -> Error::Permission("annotation edit")
- A locked object is refused by the permission check before any option is shown.

## Limits of this row

- the row reads the centre from annotation::layout, so it cannot drift from the renderer
- it writes only the text point: witness points, value and rotation are untouched

## Milestone ownership

Implemented in `OM9-L2-03-21` itself: the typed owner is `om9_core::annotation`, and the row adds
no second geometry kernel. The manual UI gate stays open (see the evidence file).

## Verification

- `cargo test -p om9-app --test annotation` → 8 passed
- `cargo test -p om9-app --test annotation_ui` → 3 passed
- `cargo test -p om9-app --test annotation edits_options_and_recenter_preserve_identity_and_are_atomic` (this row's oracle)

The golden case: a dimension whose text point was dragged away -> the text point returns to the layout centre, the object id is unchanged and the commit is one Undo entry.
