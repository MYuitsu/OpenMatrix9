# OM9-L3-01-01-004 — Docking Layout Default Restore (typed API)

Owner module `OM9-L2-01-01`. Implementation owner:
`standalone/crates/om9-app/src/dock_layout.rs` (model) and
`standalone/crates/om9-app/src/ui.rs` (commands). Reuse decision: **ADAPT**
`qymcad-shell`'s slot/container idea, **ADD** the versioned default and its
restore (see `source_crosswalk.json`).

Oracle — positive: *Reset custom layout recreates Matrix-oriented baseline
positions.* Negative: *Unknown layout version does not crash, resets to
default.*

## 1. Typed API

```rust
// dock_layout.rs — the model owns the baseline; the UI never builds one.
pub const SCHEMA_VERSION: u32 = 2;

impl DockLayout {
    /// The documented Matrix baseline: the left column in the reference order,
    /// the right column hidden, the documented panel sizes, nothing floating.
    pub fn matrix_default() -> Self;

    /// Return to the baseline in place. Panel placement, visibility, roll-up,
    /// floating rectangles and the stored homes are all baseline again.
    pub fn reset_to_default(&mut self);

    /// Repair whatever the document asked for that the model cannot honour
    /// (unknown keys, orders that are not dense, a float without a rectangle).
    pub fn normalize(&mut self) -> Vec<String>;

    pub fn to_json(&self) -> Result<String>;
    pub fn from_json(bytes: &[u8], limits: &Limits) -> Result<(Self, Vec<String>)>;
    /// Parses with the built-in limits: the entry point tests and tools use.
    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, Vec<String>)>;
    /// Total load: a missing file is a clean default; unusable bytes reset.
    pub fn load(path: &Path, limits: &Limits) -> (Self, Vec<String>);
}

// ui.rs — the two documented commands.
impl MatrixApp {
    /// View ▸ Matrix Menus ▸ Reset Menus: baseline in this session, the kept
    /// configuration is not overwritten.
    pub fn reset_menus(&mut self, ctx: Option<&egui::Context>);
    /// View ▸ Matrix Menus ▸ Reload Menus: the configuration the program opens
    /// with, falling back to the factory default.
    pub fn reload_menus(&mut self);
}
```

## 2. Result and errors

| Input | Result | Diagnostic (exact text) |
| --- | --- | --- |
| bytes with `schema_version == 2` | the parsed layout, repaired | `normalize` lines, if any |
| `schema_version` other than 2 (1, 99, missing ⇒ 0) | **the baseline** + diagnostics | `layout version {n} is not supported; restored the Matrix default` |
| bytes that are not JSON | `Err(Error::Invalid("layout document"))` from `from_json`; `load` folds it into the baseline | `layout ignored ({error}); restored the Matrix default` |
| missing file | the baseline, no diagnostic | — |
| bytes over the limit | `Err(Error::Budget("layout file bytes"))` | — |
| `reset_to_default` | baseline, no error | — |

`reset_to_default` never fails and never panics: it is the total fallback every
other path can delegate to. Nothing in this item touches the document, the undo
journal or the scene: only the interface placement.

## 3. Ownership

* The **model** owns the baseline (`matrix_default`), the version gate and the
  repair (`normalize`). There is exactly one baseline in the code base.
* The **UI** owns the commands and the messages; it calls `reset_to_default`
  and never assembles a layout by hand.
* A layout that cannot be honoured is never partially applied: `from_json`
  either returns the parsed layout (repaired) or the baseline.

## 4. Documented baseline (fixture constants)

* Left column order: `icon-history, main-menu, display, snaps, info-settings,
  layers, projects, builder-slot`.
* Right column: hidden (`right-dock.visible = false`), width 300.
* Panel sizes: left rail 300, right dock 300, command line 90, toolbar 67.
* Every menu visible and expanded; nothing floating; no stored homes.
