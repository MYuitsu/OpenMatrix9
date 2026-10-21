# OM9-L3-03-14-001 — Matrix Layer Category Palette (typed API)

Owner module `OM9-L2-03-14` (UI command adapter, stage E02, gate **M01-UI**:
manual test required). Implementation owner: `crates/om9-core/src/layers/category.rs`
(model) with `crates/om9-app/src/ui/layers.rs` (palette) and
`src/layer_command.rs` (the CMD path). Behaviour reference: Matrix8 book 1,
pages 54-57, "The Layers Menu" (shipped with the milestone under `reference/`).

Oracle — positive: *Palette has Metals/Gems/User/Heads/Finger/Cutting/Creation
semantic categories.* Negative: *Duplicate display label must not cause ID
collision.*

## 1. Typed API

```rust
// om9-core/src/layers/category.rs
pub enum PaletteGroup { Metals, Gems, User, Builders }   // the manual's four groups
pub enum LayerCategory { Metals, Gems, User, Heads, Finger, Cutting, Creation }

impl PaletteGroup {
    pub const ALL: [Self; 4];
    pub fn label(self) -> &'static str;
    pub fn categories(self) -> &'static [LayerCategory];
}
impl LayerCategory {
    pub const ALL: [Self; 7];                 // palette order
    pub fn label(self) -> &'static str;
    pub fn group(self) -> PaletteGroup;
    pub fn slots(self) -> &'static [u8];      // the rows this category shows
}
impl PresetSlot {
    pub fn category(self) -> Option<LayerCategory>;   // None only for 33 (Lights)
    pub fn palette_group(self) -> Option<PaletteGroup>;
    pub fn category_label(self) -> &'static str;      // "Lights" when uncategorised
}

// om9-app
impl MatrixApp {
    pub fn palette_categories(&self) -> [&'static str; 7];
    pub fn palette_slot_label(&self, number: u8) -> Option<&'static str>;
}
// and the audit receipt carries palette.categories + palette.slots
```

## 2. Rules

| Slot | Category | Group | Row label |
| --- | --- | --- | --- |
| 1-4 | Metals | Metals | `Metals` |
| 5-8 | Gems | Gems | `Gems` |
| 9-12 | User | User | `User` |
| 13 | Heads | Builders | `Heads` |
| 14 | Finger | Builders | `Finger` |
| 15 | Cutting | Builders | `Cutting` |
| 16 | Creation | Builders | `Creation` |
| 17-32 | User (extra colours) | User | `User` |
| 33 | **none** | none | `Lights` |

- The palette shows the **seven** categories the plan names, and the model also
  keeps the **four** groups the manual names (Metals, Gems, User, Builders), with
  Builders carrying Heads/Finger/Cutting/Creation.
- A category is **not** a fixed four-slot range: Metals/Gems/User hold four
  colours each, the four builder categories hold one slot each. The palette's
  visible 4-row blocks are a layout detail (`layer_command::group`), not a
  category.
- The row label is the category; the row's layer name stays the layer's own name
  (a custom slot may be renamed), so the two never fight.
- **Identity is the slot number** (`PresetSlot`, serialised as `u8`). A duplicate
  display label cannot collide with a preset id; a custom slot may not take a
  reserved label (`preset::valid_label`), and the CMD path refuses an ambiguous
  quoted label instead of guessing.
- Numbers outside `1..=33` are not slots (`Error::Invalid`).

## 3. Ownership

* The **core** owns the categories and the slot mapping: the palette, the CMD
  path and the receipt all read the same model.
* The **palette** owns only what a row looks like; the label is painted in the
  row's free left area, so it cannot cover the name/swatch/lock/visibility
  controls on the right.
* The **manual gate** (`M01-UI`) confirms the look: seven names visible, colours
  matching the manual, group toggles still working.
