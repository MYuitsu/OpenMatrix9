# OM9-L3-03-14-008 — Layer State Persistence Visual Check (typed API)

Owner module `OM9-L2-03-14` (UI command adapter, stage E02, gate **M01-UI**).
Implementation owner: the archive + the controller's layer reads; the palette
paints `document.layers` and nothing else.

Oracle — positive: *Save/reopen 2 layers incl lock and visibility and compare
palette icons and picked IDs.* Negative: *Palette must not show cached unsaved
state after reopen.*

## 1. Path

```rust
Archive::save(path, &document, &limits, &Cancellation::default())?;
Archive::open(path, &limits)? -> Document
Controller::new(restored, limits)   // the palette then reads this document
Controller::layer_id(slot)          // the row shown for a slot, None when absent
Controller::select_layer(slot)      // the picked ids under the selectable policy
```

## 2. Rules

| Check | Result |
| --- | --- |
| save, reopen | the same preset slots come back (the four-slot group created by touching one slot is saved whole), with `locked` / `visible` and the slot colour preserved |
| entities | the same object ids come back, on the same layers |
| palette over the reopened document | `layer_id` finds both layers, and the selectable policy still holds: a locked Gems and a hidden Metals select nothing |
| opening **another** document | the palette shows that document's layers only; the previous slots are absent (`layer_id` = None) and select nothing |
| a **failed** open (missing file) | the current document and its palette flags are untouched |
| an **unsaved** flag applied in memory after the save | is **not** in the reopened file: the flags come from the file, not from a cache |

## 3. Ownership

* The **archive** owns persistence; the **document** is the only source the
  palette reads, so there is no second cache that could show stale state.
* The controller rebuilds its scene from the reopened document; nothing of the
  previous document survives the open.
