# OM9-L3-03-21-008 — Leader — menu binding

- Owner: `OM9-L2-03-21`
- Command id: `OM9-MEASURE-012`
- Command line: `Leader`
- Annotation kind: `Kind::Leader`
- Selection rule: nothing selected
- Body: `crates/om9-app/src/session/annotation.rs`
- Model owner: `crates/om9-core/src/annotation.rs` (`Annotation`, `Kind`, `layout`, slot `annotation::SLOT`)

## What the row does

three typed points draw a two-segment leader (tip, elbow, text position) and then the row asks for the text, which is stored as the annotation's own text.

## Input contract

| step | prompt | input |
| --- | --- | --- |
| 1 | `First witness/leader tip: ` | typed input |
| 2 | `Second witness/leader elbow: ` | typed input |
| 3 | `Annotation text position: ` | typed input |
| 4 | `Annotation text: ` | typed input |

Points are resolved through the CPlane captured when the command started, and the record stores
that plane's axes so a later projection cannot reinterpret the annotation.

## What it refuses

- an empty text is refused instead of storing an invisible label
- A locked object is refused by the permission check before any option is shown.

## Limits of this row

- the leader keeps its elbow, so it is not flattened into a single line
- Edit Text edits the text without rewriting the geometry

## Milestone ownership

Implemented in `OM9-L2-03-21` itself: the typed owner is `om9_core::annotation`, and the row adds
no second geometry kernel. The manual UI gate stays open (see the evidence file).

## Verification

- `cargo test -p om9-app --test annotation` → 8 passed
- `cargo test -p om9-app --test annotation_ui` → 3 passed
- `cargo test -p om9-app --test annotation text_and_leader_edit_text_without_flattening_semantics` (this row's oracle)

The golden case: tip (1,1), elbow (5,5), text at (6,6) and the text "Gem A" -> one annotation whose two segments are independent strokes.
