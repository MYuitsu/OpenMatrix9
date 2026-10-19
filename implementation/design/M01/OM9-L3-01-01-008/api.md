# OM9-L3-01-01-008 — Builder Panel Docking Slot (typed API)

Owner module `OM9-L2-01-01`; the builders themselves belong to M04. Reuse
decision: **ADAPT** `qymcad-shell`'s keyed place; **ADD** the reservation and a
placeholder that needs no builder model (see `source_crosswalk.json`).

Oracle — positive: *Reserve Builder docking slot below Projects without creating
a second Builder model.* Negative: *Absent Builder implementation shows a
noninteractive placeholder.*

## 1. Typed API

```rust
// dock_layout.rs — the reservation is a property of the layout model.
pub enum SectionId { …, Projects, BuilderSlot }
impl SectionId {
    /// The documented order has the reserved slot directly below Projects.
    pub const ALL: [SectionId; 8];
    /// True for the reserved slot: no placement operation is permitted.
    pub fn is_reserved(self) -> bool;
    pub fn is_movable(self) -> bool;      // panel_ops.rs: !is_reserved()
}

// panel_ops.rs — the single door refuses it, whatever the caller.
impl DockLayout {
    pub fn apply_panel_op(&mut self, op: PanelOp) -> Result<PanelReport, PanelOpError>;
    // BuilderSlot ⇒ Err(PanelOpError::NotPermitted { section })
}

// ui/chrome.rs — the placeholder. Every widget is disabled or hover-only; the
// bar carries no close box and starts no drag.
fn builder_slot_ui(&mut self, ui: &mut egui::Ui);
```

## 2. Rules

| Situation | Result |
| --- | --- |
| the documented baseline | the reserved slot is in the left column, index 7, directly below Projects, visible |
| any placement operation (move, place, dock, dock home, float, set rect, collapse) | `NotPermitted`, and the layout is unchanged byte for byte |
| a mouse click on its bar | the documented refusal message; no roll-up; no drag starts |
| its bar | no `+`/`−` close box (the slot cannot be rolled up) |
| its body with no builder implementation | a placeholder: disabled buttons, hover-only gem box, disabled slider; it names the reserved area and M04 |
| a builder model | **none**: the slot owns no document, no entities, no undo entries and no second model object |
| a layout document that moves, hides or floats the reserved slot | `normalize` puts it back in the documented left slot with a diagnostic, because the reservation is not user state |

## 3. Ownership

* The **layout model** owns the reservation: the slot's position is documented,
  not persisted user state, so a stored file cannot break it.
* **M04** will own the builders that open in the slot; M01 only reserves the area
  and shows the placeholder.
* Nothing here touches the document: the placeholder is drawn from the layout,
  never from a builder object.
