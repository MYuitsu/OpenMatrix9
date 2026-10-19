# OM9-L3-01-01-005 — Docking Layout Persistence (typed API)

Owner module `OM9-L2-01-01`. Implementation owner:
`standalone/crates/om9-app/src/dock_layout.rs` (the store) and
`standalone/crates/om9-app/src/ui.rs` (the session commands). Reuse decision:
**ADAPT** `qymcad-shell`'s stable keys and container sizes plus `gui.rs`'s
store idea; **ADD** the bounded layout file and its repair diagnostics (see
`source_crosswalk.json`).

Oracle — positive: *Persist dock positions and reopen same named panel IDs.*
Negative: *Removed/unrecognized panel ID ignored with warning not crash.*

## 1. Typed API

```rust
// dock_layout.rs — the store. One file, one schema, no panics.
pub const SCHEMA_VERSION: u32 = 2;

impl DockLayout {
    pub fn to_json(&self) -> Result<String>;
    pub fn from_json(bytes: &[u8], limits: &Limits) -> Result<(Self, Vec<String>)>;
    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, Vec<String>)>;   // built-in limits
    pub fn load(path: &Path, limits: &Limits) -> (Self, Vec<String>); // total: never fails
    pub fn save(&self, path: &Path) -> Result<()>;
    pub fn normalize(&mut self) -> Vec<String>;                       // repair + warnings
}

// ui.rs — the session commands.
impl MatrixApp {
    pub fn set_dock_layout_path(&mut self, path: Option<PathBuf>);
    pub fn dock_layout_path(&self) -> Option<&Path>;
    /// Writes only when the layout changed; a failed write is a warning.
    pub fn save_dock_layout(&mut self) -> Result<()>;
    /// Loads the kept configuration; a missing file is a clean default.
    pub fn load_dock_layout(&mut self) -> Vec<String>;
    pub fn menus_have_unsaved_changes(&self) -> bool;
}
```

The document format is schema 2: `panels` keyed by `PanelId::key`
(`left-rail`, `right-dock`, `command-line`, `command-search`, `toolbar`) and
`sections` keyed by `SectionId::key` (the eight documented menu keys). Each
entry carries its own state (`visible`, `collapsed`, `size`/`height`, `dock`,
`order`, `float`, `home`). **Names are the contract**: a menu keeps its key, so
reopening a file restores the same named panel.

## 2. Results and errors

| Situation | Result | Diagnostic (substring on record) |
| --- | --- | --- |
| every name known, every value usable | the layout, no diagnostics | — |
| unknown panel key | removed | `layout panel '{key}' is not recognized; removed` |
| unknown menu key | removed | `menu section '{key}' is not recognized; removed` |
| a known name missing | entry created from its documented default | — |
| panel size not finite or outside its bounds | clamped to the documented bounds | `panel '{key}' size … is not usable; using …` |
| menu height not finite or outside its bounds | clamped | `{title} height … is not usable; using …` |
| floating rectangle not finite | dropped (the menu is docked, not lost) | `{title} floating rectangle is not finite; the stored position was dropped` |
| floating rectangle below the minimum | raised to the minimum | `{title} floating rectangle … is smaller than the minimum; raised to …` |
| a menu floats without a rectangle | placed at the default position | `{title} was floating without a rectangle; placed at the default position` |
| menu orders not dense | renumbered | `menu orders were not dense; the documented order was restored` |
| a stored home outside its column | repaired to the documented slot | `{title} home … is outside the … menu column; the documented position is used` |
| file missing | the documented baseline, no diagnostic | — |
| file unreadable / empty / oversized | the baseline | `layout ignored (…); restored the Matrix default` |
| `schema_version` other than 2 | the baseline | `layout version {n} is not supported; restored the Matrix default` |
| `save` to an unusable path | `Err` to the caller, status warning in the UI | `layout not saved ({error})` |

Loading never panics and never half-applies: `load` returns either the parsed
(and repaired) layout or the baseline. Saving is the only writer, and only the
documented command (Set as Default) calls it.

## 3. Ownership

* The **model** owns the file format, the version gate, the repair and the
  diagnostics. Keys are stable identifiers, never display titles.
* The **UI** owns when a file is read or written and how a diagnostic is shown
  (status line + command history).
* Nothing in this item touches the document, the undo journal or the scene.
