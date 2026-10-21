# OM9-L3-03-14-006 — Superselect By Jewelry Object Type (typed API)

Owner module `OM9-L2-03-14` (UI command adapter, stage E02, gate **M01-UI**).
Implementation owner: `crates/om9-app/src/superselect.rs`. Behaviour reference:
Matrix8 book 1, pages 32-34 ("Super Select — This tool determines what types of
objects you …", shipped with the milestone under `reference/`).

Oracle — positive: *Select Gems filter matches jewelry-semantic GemRef, bad/open
filter uses typed kernel verdict.* Negative: *Do not classify ordinary mesh as
valid gemstone by color/name alone.*

## 1. Typed API

```rust
pub enum SelectFilter { Gems, BadGeometry, OpenGeometry, Duplicate }
pub enum SkipReason { NoGemExtension, GeometryValid, NoOpenVerdict(&'static str), UniqueLabel }
pub struct SuperselectOutcome { pub matched: Vec<ObjectId>, pub skipped: Vec<(ObjectId, SkipReason)> }

pub fn classify(document, entity, filter, limits) -> Option<SkipReason>
impl Controller {
    pub fn superselect(&self, filter: SelectFilter) -> SuperselectOutcome;   // read-only
    pub fn apply_superselect(&mut self, filter: SelectFilter) -> usize;      // sets the selection
}
```

## 2. Rules

| Filter | Matches | Skips with |
| --- | --- | --- |
| `Gems` | an entity carrying the `om9.gem` extension (`GEM_EXTENSION_SLOT`) with version ≥ 1 and a payload that is not blank | `NoGemExtension` |
| `BadGeometry` | an entity whose geometry fails the model's own `validate(limits)` | `GeometryValid` |
| `OpenGeometry` | nothing yet: **no typed open/closed verdict exists**, so nothing is claimed | `NoOpenVerdict("…")` on every entity |
| `Duplicate` | a label (case-insensitive) that an earlier object of the same geometry kind already uses | `UniqueLabel` on the first |

The **negative oracle is structural**: a mesh whose label is literally `Gem`, or
that was drawn on the Gems layer colour, carries no `om9.gem` extension and is
therefore **skipped**, never classified as a gemstone. The model also refuses a
payload-free extension at creation (`Extension::new`), so the filter can never
see one.

`apply_superselect` replaces the selection and returns the count; it reads the
document and never writes it (no transaction, no journal entry).

## 3. Ownership

* The **core model** owns the semantic slot (`om9.gem`) and the geometry verdict
  (`validate`): SuperSelect only reads them.
* The **app** owns the filter vocabulary and the selection it produces.
* A missing verdict is reported, not invented — the same rule as Object Info's
  unavailable measurement.
