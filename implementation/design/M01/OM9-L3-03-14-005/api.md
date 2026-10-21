# OM9-L3-03-14-005 — Matrix Object Info Overlay (typed API)

Owner module `OM9-L2-03-14` (UI command adapter, stage E02, gate **M01-UI**).
Implementation owner: `crates/om9-core/src/object_info.rs`. Behaviour reference:
Matrix8 book 1, pages 32-34 (shipped with the milestone under `reference/`).

Oracle — positive: *Select box shows type, bounding box dimensions, geometry
validity sourced from model.* Negative: *Missing BRep must show unavailable
measurement, not zero volume claim.*

## 1. Typed API

```rust
pub enum Measurement { Size([f64; 3]), Unavailable(&'static str) }
pub struct ObjectInfo {
    pub kind: &'static str,          // "Point", "Line", "Circle", "Curve", "Surface",
                                     // "Mesh", "Point cloud", "B-rep", "Native B-rep", "Instance"
    pub dimensions: Measurement,
    pub valid: Option<bool>,         // None when the model cannot judge it here
    pub detail: String,              // length / radius / counts
    pub color: Option<[u8; 4]>,      // the layer colour the label is drawn in
}
Document::object_info(id: ObjectId, limits: &Limits) -> Result<ObjectInfo>
```

## 2. Rules

| Geometry | kind | dimensions | valid |
| --- | --- | --- | --- |
| Point | `Point` | `Size([0,0,0])` (a point really is a point) | `Some(true)` |
| `Curve::Line` | `Line` | the bounding size of its two ends; detail `length …` | `Some(validate)` |
| `Curve::Circle` | `Circle` | `Size([2r, 2r, 0])`; detail `radius …` | `Some(validate)` |
| Mesh | `Mesh` | **`Unavailable`** — no mesh bounds are computed yet | `Some(validate)` |
| Native B-rep without a payload | `Native B-rep` | **`Unavailable`** ("the native kernel payload is not loaded") | `Some(validate)` |
| B-rep | `B-rep` | **`Unavailable`** ("B-rep bounds come from the kernel and are not loaded") | `Some(validate)` |
| Surface / Cloud / Instance | as named | `Unavailable` with the reason | `Some(validate)` |
| an id that is not in the document | — | — | `Err(Missing("object info"))` |

**The rule the plan asks for**: a measurement that is not available is reported
as `Unavailable(reason)`, never as a zero that would read as a real size; and a
validity that cannot be judged is `None`, never an invented `true`.

## 3. The in-viewport label (delivered in round 8)

`MatrixApp::object_info_label` paints the line beside a **single** selected object:
the object's centre is projected with `view.camera.project`, and the text is
`om9_core::object_info::label_text(&info)` — `type · x × y × z · valid`, or
`type · unavailable (reason) · invalid`, drawn in the object's layer colour.
The audit receipt carries the same line as `object_info { kind, label }`, so a real
window can be checked without a screenshot.

Unit test: `om9-core` `object_info::label_tests::the_label_reads_type_size_and_validity`
(known size, unavailable size, and an unjudged validity).

## 3. Ownership

This item delivers the **data and its rules** (the model, the fixtures, the
tests). The manual's *"quick pop up label beside a selected object in the
viewports … in the layer colour of the object"* — the painted overlay — is the
next slice: it needs the world→screen projection of the selection centre and the
label layout. Until it is painted, the owner's manual check confirms the fields
and wording through the model and the receipt, not yet on screen.
