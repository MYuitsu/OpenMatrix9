# OM9-L3-03-21-012 — Dimension Options — menu binding

- Owner: `OM9-L2-03-21`
- Command id: `OM9-MEASURE-016`
- Command line: `DimOptions`
- Annotation kind: `Kind::any editable annotation`
- Selection rule: one annotation, or nothing to use the session defaults
- Body: `crates/om9-app/src/session/annotation.rs`
- Model owner: `crates/om9-core/src/annotation.rs` (`Annotation`, `Kind`, `layout`, slot `annotation::SLOT`)

## What the row does

opens the option set of the selected annotation (or the session defaults when nothing is selected), so text size, arrow and precision preferences are one documented form rather than hidden state.

## Input contract

| step | prompt | input |
| --- | --- | --- |
| 1 | `Edit annotation options; Enter to apply: ` | typed input |

Points are resolved through the CPlane captured when the command started, and the record stores
that plane's axes so a later projection cannot reinterpret the annotation.

## What it refuses

- more than one selected object -> Error::Invalid("select one annotation"); a locked object -> Error::Permission("annotation edit")
- A locked object is refused by the permission check before any option is shown.

## Limits of this row

- with no selection the row edits the session defaults, which the Style row records as observable state
- the form is the same one the creation rows use

## Milestone ownership

Implemented in `OM9-L2-03-21` itself: the typed owner is `om9_core::annotation`, and the row adds
no second geometry kernel. The manual UI gate stays open (see the evidence file).

## Verification

- `cargo test -p om9-app --test annotation` → 8 passed
- `cargo test -p om9-app --test annotation_ui` → 3 passed
- `cargo test -p om9-app --test annotation edits_options_and_recenter_preserve_identity_and_are_atomic` (this row's oracle)

The golden case: an annotation edited through the option form keeps its geometry and commits once.
