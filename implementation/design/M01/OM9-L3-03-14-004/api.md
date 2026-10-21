# OM9-L3-03-14-004 — Select All Objects By Layer (typed API)

Owner module `OM9-L2-03-14` (UI command adapter, stage E02, gate **M01-UI**).
Implementation owner: `crates/om9-app/src/session/layers.rs` (the gesture) with
`crates/om9-app/src/ui/layers.rs` (the palette row/group) and the core's
`permissions_for_entity` (the selectable policy).

Oracle — positive: *Select all objects in active layer returns exactly its
editable objects.* Negative: *Hidden/locked objects must respect selectable
policy without silent override.*

## 1. Typed API

```rust
Controller::select_layer(slot: PresetSlot) -> Result<()>            // the gesture
Controller::selectable_layer_member_count(slot) -> Result<usize>    // what it will select
Document::permissions_for_entity(entity, snap_locked) -> Permissions { visible, selectable, editable, … }
```

## 2. Rules

| Situation | Result |
| --- | --- |
| a layer with two visible, unlocked objects | both are selected; a second gesture **replaces** the selection (never appends) |
| an object on another layer | not selected |
| one object of the layer hidden (`set_objects_hidden`) | the gesture selects the other one only |
| a **locked** layer | selects nothing, and the previous selection is **cleared** instead of silently kept |
| a **hidden** layer | selects nothing |
| a slot with no layer | selects nothing and is not an error |
| any gesture | writes no transaction and no journal entry; the selection is session state |
| over the object budget | refused with `Budget("layer selection count")` before anything is retained |

The policy lives in the core (`permissions_for_entity` walks the layer chain:
`selectable &= visible && !object.locked && !layer.locked`), so the palette, the
group gesture and the CMD path all obey one rule.

## 3. Ownership

* The **core** decides what is selectable; the **controller** only collects what
  the rule allows and replaces the set.
* The palette's row gesture and its group toggle go through the same function, so
  a group gesture cannot select what a row gesture would refuse.
