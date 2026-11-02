# OM9-L3-03-21-010 — Edit Dimension — menu binding

- Owner: `OM9-L2-03-21`
- Command id: `OM9-MEASURE-014`
- Command line: `DimEdit`
- Annotation kind: `Kind::dimension kinds only`
- Selection rule: exactly one dimension
- Body: `crates/om9-app/src/session/annotation.rs`
- Model owner: `crates/om9-core/src/annotation.rs` (`Annotation`, `Kind`, `layout`, slot `annotation::SLOT`)

## What the row does

re-opens a selected dimension through the same option formatter the creation row used, so a stored value or a stored text can be changed without losing the witness points or the identity of the object.

## Input contract

| step | prompt | input |
| --- | --- | --- |
| 1 | `Edit annotation options; Enter to apply: ` | typed input |

Points are resolved through the CPlane captured when the command started, and the record stores
that plane's axes so a later projection cannot reinterpret the annotation.

## What it refuses

- no selection, or more than one -> Error::Invalid("select one annotation"); a locked object -> Error::Permission("annotation edit"); a Text Block or Leader -> Error::Invalid("select a dimension")
- A locked object is refused by the permission check before any option is shown.

## Limits of this row

- the row edits the record, never a flattened display copy
- an unreadable or non-annotation object in the selection is refused rather than reinterpreted

## Milestone ownership

Implemented in `OM9-L2-03-21` itself: the typed owner is `om9_core::annotation`, and the row adds
no second geometry kernel. The manual UI gate stays open (see the evidence file).

## Verification

- `cargo test -p om9-app --test annotation` → 8 passed
- `cargo test -p om9-app --test annotation_ui` → 3 passed
- `cargo test -p om9-app --test annotation edits_options_and_recenter_preserve_identity_and_are_atomic` (this row's oracle)

The golden case: a horizontal dimension edited to a new text keeps its two witness points, its text position and its object id, and the edit is one Undo entry.
