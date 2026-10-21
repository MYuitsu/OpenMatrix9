# OM9-L3-03-14-003 — Layer Visible And Locked Controls (typed API)

Owner module `OM9-L2-03-14` (UI command adapter, stage E02, gate **M01-UI**).
Implementation owners: `crates/om9-app/src/session/layers.rs` (the rules),
`crates/om9-app/src/ui/layers.rs` (the controls), `crates/om9-core/src/model.rs`
(`display_attributes`, `permissions`).

Oracle — positive: *Toggle visible/locked on Gems affects viewport and picking
immediately.* Negative: *Locked object edit attempt must have no mutation and
informative status.*

## 1. Typed API

```rust
Controller::set_layer_property(slot: PresetSlot, property: LayerProperty) -> Result<()>
enum LayerProperty { Visible(bool), Locked(bool) }
Document::display_attributes(object) -> DisplayAttributes { visible, selectable, editable, layer, … }
Controller::select_layer(slot) -> Result<()>        // picks the layer's selectable objects
```

The palette paints the same two flags the controller writes: the lock icon is
white when locked and grey when not, and the visibility control draws a "/"
across the swatch when the layer is hidden.

## 2. Rules

| Toggle | Effect |
| --- | --- |
| Visible(false) on Gems | every object on Gems reports `visible = false` and `selectable = false` at once; the scene entry flips in the same call, so the viewport and picking follow immediately |
| select_layer on a hidden layer | selects nothing (no object is pick-capable) |
| Visible(true) | the objects come back visible, selectable and pick-capable |
| Locked(true) on Gems | the objects stay **visible** but `editable = false`: they can be inspected, not changed |
| moving an object **onto** a locked layer | refused before anything is applied (`Permission`) |
| moving an object **off** a locked layer | refused before anything is applied (`Permission("object is on a locked layer")`) |
| the refusal | no entity changes, no journal entry, and the status line shows the reason |
| inspection of a locked/hidden layer | still complete: `display_attributes` keeps the name, the layer and the flags |

## 3. Ownership

* The **controller** owns the toggles and the two lock rules (target and source);
  the **document** owns the effective flags; the **scene** follows the document.
* The palette only paints state and routes clicks — it never keeps a copy of the
  visible/locked flags.
* A lock never hides information: it only refuses edits, with a reason.
