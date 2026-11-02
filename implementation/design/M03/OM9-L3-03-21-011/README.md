# OM9-L3-03-21-011 — Edit Text — menu binding

- Owner: `OM9-L2-03-21`
- Command id: `OM9-MEASURE-015`
- Command line: `gvEditText`
- Annotation kind: `Kind::Text and Leader only`
- Selection rule: exactly one text block or leader
- Body: `crates/om9-app/src/session/annotation.rs`
- Model owner: `crates/om9-core/src/annotation.rs` (`Annotation`, `Kind`, `layout`, slot `annotation::SLOT`)

## What the row does

changes the text of a Text Block or Leader and nothing else: the geometry, the points and the object identity stay.

## Input contract

| step | prompt | input |
| --- | --- | --- |
| 1 | `Edit annotation options; Enter to apply: ` | typed input |

Points are resolved through the CPlane captured when the command started, and the record stores
that plane's axes so a later projection cannot reinterpret the annotation.

## What it refuses

- a dimension in the selection -> Error::Invalid("select a Text Block or Leader"); no selection -> Error::Invalid("select one annotation")
- A locked object is refused by the permission check before any option is shown.

## Limits of this row

- the row refuses to pretend a dimension is a text block, which is the failure mode this split exists to prevent
- empty text is refused

## Milestone ownership

Implemented in `OM9-L2-03-21` itself: the typed owner is `om9_core::annotation`, and the row adds
no second geometry kernel. The manual UI gate stays open (see the evidence file).

## Verification

- `cargo test -p om9-app --test annotation` → 8 passed
- `cargo test -p om9-app --test annotation_ui` → 3 passed
- `cargo test -p om9-app --test annotation text_and_leader_edit_text_without_flattening_semantics` (this row's oracle)

The golden case: leader text changed to "Gem B" -> same tip/elbow strokes and the same object id, one Undo entry.
