# OM9-L3-03-14-007 — Object Property Editor Adapter (typed API)

Owner module `OM9-L2-03-14` (UI command adapter, stage E02, gate **M01-UI**).
Implementation owner: `crates/om9-app/src/session/properties.rs` (a child module of
`session`, because it applies a transaction like every other controller path).

Oracle — positive: *Edit color/name/property through document command once; undo
restores previous value.* Negative: *Editing derived read-only BRep field must be
disabled.*

## 1. Typed API

```rust
pub enum PropertyField { Name, Color, Visible, Hidden, Selectable, Locked, Geometry }
pub struct FieldState { pub field: PropertyField, pub editable: bool, pub reason: &'static str }
pub const FIELDS: [FieldState; 7];

/// Only editable fields exist here: the type is the disable (Geometry has no variant).
pub enum PropertyEdit { Label(String), Color(Option<[u8;4]>), Visible(bool), Hidden(bool), Selectable(bool), Locked(bool) }

impl Controller {
    pub fn edit_object_property(&mut self, sources: Vec<SourceStamp>, edit: PropertyEdit) -> Result<()>;
}
```

## 2. Rules

| Field | Editable | Why |
| --- | --- | --- |
| Color, Visible, Hidden, Selectable, Locked | **yes** | they live in `Attributes` and go through ONE `Mutation::Attributes` transaction per edit |
| Name | **yes** (round 23) | it goes through `Mutation::Label`, one transaction per rename; an empty, padded or over-long label is refused |
| Geometry | no | derived from the geometry itself: read-only |

- **One command**: every edited object is sent as one `TransactionRequest`, so one
  gesture is one journal entry and Undo restores every previous value together.
- A **no-op** edit (the value already matches) writes nothing at all.
- An edit on an object whose layer is **locked** is refused by the core's own
  attribute validation (`Permission`) before anything is applied.
- A read-only field cannot be edited *by construction*: `PropertyEdit` has no
  variant for it, so a caller cannot ask, and the UI table states the reason.

## 3. Ownership

* The **core** owns the mutation (`Mutation::Attributes`) and its validation; the
  controller only builds the request.
* The **editor table** owns what the UI may offer; a field the model cannot edit
  is `editable: false` with a reason, not a hidden rule.
* Undo/redo are the coordinator's journal: this item adds no second history.
