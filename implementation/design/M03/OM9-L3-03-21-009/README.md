# OM9-L3-03-21-009 — Text Block — menu binding

- Owner: `OM9-L2-03-21`
- Command id: `OM9-MEASURE-013`
- Command line: `TextBlock`
- Annotation kind: `Kind::Text`
- Selection rule: nothing selected
- Body: `crates/om9-app/src/session/annotation.rs`
- Model owner: `crates/om9-core/src/annotation.rs` (`Annotation`, `Kind`, `layout`, slot `annotation::SLOT`)

## What the row does

one typed point places a text block and the next answer is its text; nothing is measured, so no witness points are stored.

## Input contract

| step | prompt | input |
| --- | --- | --- |
| 1 | `Annotation text position: ` | typed input |
| 2 | `Annotation text: ` | typed input |

Points are resolved through the CPlane captured when the command started, and the record stores
that plane's axes so a later projection cannot reinterpret the annotation.

## What it refuses

- an empty text is refused
- A locked object is refused by the permission check before any option is shown.

## Limits of this row

- a text block is model-space state like a dimension: it lives in the document, not in the viewport overlay
- the row is also reachable as DimText

## Milestone ownership

Implemented in `OM9-L2-03-21` itself: the typed owner is `om9_core::annotation`, and the row adds
no second geometry kernel. The manual UI gate stays open (see the evidence file).

## Verification

- `cargo test -p om9-app --test annotation` → 8 passed
- `cargo test -p om9-app --test annotation_ui` → 3 passed
- `cargo test -p om9-app --test annotation text_and_leader_edit_text_without_flattening_semantics` (this row's oracle)

The golden case: text at (4,4) reading "OM9" -> one annotation with no witness points.
