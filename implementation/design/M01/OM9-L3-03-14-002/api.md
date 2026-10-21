# OM9-L3-03-14-002 — Current Layer And Assignment UI (typed API)

Owner module `OM9-L2-03-14` (UI command adapter, stage E02, gate **M01-UI**).
Implementation owner: `crates/om9-app/src/session/layers.rs` (the controller
path) with `crates/om9-app/src/ui/layers.rs` (the palette) and
`crates/om9-app/src/layer_command.rs` (the CMD path).

Oracle — positive: *Select current layer Metals, assign object A, panel and model
both reflect same LayerId.* Negative: *Assign onto locked/unavailable layer must
refuse explicitly.*

## 1. Typed API

```rust
// crates/om9-app/src/session/layers.rs (Controller)
pub fn set_current_layer(&mut self, choice: LayerChoice) -> Result<()>;
pub fn assign_layer(&mut self, slot: PresetSlot, sources: Vec<SourceStamp>) -> Result<()>;
pub fn set_layer_property(&mut self, slot: PresetSlot, property: LayerProperty) -> Result<()>;
pub fn layer_id(&self, slot: PresetSlot) -> Option<LayerId>;
pub fn select_layer(&mut self, slot: PresetSlot) -> Result<()>;

// crates/om9-app/src/layer_command.rs
pub enum LayerChoice { None, Slot(PresetSlot), Existing(LayerId) }
pub fn parse(text: &str, document: &Document) -> Result<Option<LayerChoice>>;
```

## 2. Rules

| Action | Result |
| --- | --- |
| `set_current_layer(Slot(1))` when Metals has no layer yet | the preset layer is created **and** made current in one call; the group it belongs to is created with it (no loose preset) |
| `set_current_layer(Slot(1))` twice | the second call only sets the current layer: the layer already exists |
| `set_current_layer(None)` | the document's `current_layer` becomes `None` |
| `assign_layer(slot, sources)` with a selection | one transaction moves the objects to that layer; the **current** layer does not move with them |
| `assign_layer(..., vec![])` (nothing selected) | succeeds and adds no journal entry — it is a no-op, not an error |
| `assign_layer` onto a **locked** layer | refused **before** anything is applied: `Error::Permission("layer is locked")`, the objects stay where they were and the journal does not move |
| the panel's current row | reads `document.current_layer` through `layer_id(slot)`, so the painted row and the model are the same LayerId |
| the CMD path | `Layer 5` / `Layer Gems` / `Layer "Gems"` resolve to the same slot; an ambiguous quoted label is refused instead of guessed |

## 3. Ownership

* The **controller** owns the actions (create + current in one `LayerAction` batch);
  the **core** owns the transaction and the validation that an action changes
  something.
* The **palette** only paints and routes: its current-layer highlight is
  `layer_id(slot) == document.current_layer`, not a second copy of the state.
* The lock is a property of the layer, not of the row: `set_layer_property` may
  lock/unlock at any time, and the assignment rule reads it at the moment of the
  assignment.

## 4. Open question for the owner (not changed by this item)

The current layer code also refuses **making a locked layer current**. This item
does not touch that rule and its oracle does not mention it; Matrix9 may allow
choosing a locked layer as current (the lock protects its contents, not the
choice). The fixtures therefore do not assert either way — please confirm which
behaviour you want.
