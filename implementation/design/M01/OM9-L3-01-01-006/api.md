# OM9-L3-01-01-006 — Active Document Window State (typed API)

Owner module `OM9-L2-01-01`. Implementation owner:
`standalone/crates/om9-app/src/session/files.rs` (the lifecycle),
`src/ui/files.rs` (the title and the modal), `src/startup.rs` (the base title).
Reuse decision: **ADAPT** QymCAD's title rule; **ADD** the unsaved-work decision
path QymCAD does not have (see `source_crosswalk.json`).

Oracle — positive: *Switch active document changes title and unsaved indicator
without stale UI refs.* Negative: *Close document with unsaved changes requires
an explicit decision.*

## 1. Typed API

```rust
// session/files.rs — one operation at a time, addressed by a token.
pub fn active_file_path(&self) -> Option<&Path>;
pub fn is_dirty(&self) -> Result<bool>;          // the document differs from the file
pub fn file_operation(&self) -> Option<&FileOperation>;   // token + phase + intent
pub fn begin_file_operation(&mut self, kind: FileKind) -> Result<FileOutcome>;
pub fn begin_window_close(&mut self) -> Result<FileOutcome>;
pub fn decide_unsaved(&mut self, token: FileToken, decision: UnsavedDecision) -> Result<FileOutcome>;
pub fn accept_file_path(&mut self, token: FileToken, path: &Path) -> Result<FileOutcome>;
pub fn confirm_file_overwrite(&mut self, token: FileToken, yes: bool) -> Result<FileOutcome>;
pub fn cancel_file_operation(&mut self, token: FileToken) -> Result<()>;

// ui/files.rs — the title, and the modal that owns the decision.
pub(super) fn update_file_title(&mut self, ctx: &egui::Context);
pub(super) fn file_ui(&mut self, ctx: &egui::Context);
pub(super) fn service_window_close(&mut self, ctx: &egui::Context) -> bool;

// The observable title.
window_title = "OpenMatrix9 — " + (file name, else the document name) + (" *" if dirty)
```

`FileKind = { New, Open, Save, SaveAs }`, `FilePhase =
{ Unsaved, OpenPath, SavePath, Overwrite, CloseReady }`, `UnsavedDecision =
{ Save, Discard }`, `FileOutcome = { Pending, Saved, Replaced, SavedPending,
SavedAndReplaced, CloseReady, SavedAndCloseReady }`.

## 2. Title rule

| State | Title |
| --- | --- |
| launch, no change | `OpenMatrix9 — Untitled` (never empty) |
| document differs from its file | `OpenMatrix9 — Untitled *` |
| saved to `part.om9` | `OpenMatrix9 — part.om9` (the marker is gone) |
| changed again after the save | `OpenMatrix9 — part.om9 *` |
| another file opened | `OpenMatrix9 — other.om9` (and no marker: a freshly opened document is clean) |
| a very long name | the name is cut at 480 bytes, on a character boundary |
| only menus moved, collapsed, docked or floated | unchanged: panel placement is interface state, not document state |

The viewport title is sent only when the string changed. That rule is QymCAD's
(`gui.rs:1598`), and it is kept: a title command every frame makes some window
managers flicker.

## 3. Close decision (the negative oracle)

`begin_window_close()` with a clean document → `FileOutcome::CloseReady`, no
prompt. With unsaved changes → the operation enters `FilePhase::Unsaved` and
**nothing closes and nothing is written** until `decide_unsaved` answers:

| Answer | Result |
| --- | --- |
| `Save` with a known path | the file is written, `FileOutcome::SavedAndCloseReady` |
| `Save` without a path | `FilePhase::SavePath`: the user picks the file first |
| `Discard` | `FileOutcome::CloseReady`; the file on disk is untouched |
| `cancel_file_operation(token)` | no close, the document stays dirty, the file is untouched |
| a token from a cancelled or finished operation | `Err`: a stale prompt can never close or save anything |

`New` and `Open` with unsaved changes enter the same `Unsaved` phase, so the
same three answers apply.

## 4. Ownership

* The **document** owns the entity and undo state; the lifecycle owns the
  question "does the document differ from its file".
* The **UI** owns the title string and the modal; it never writes a file itself.
* Panel placement, the command draft and the layout store are outside this item:
  moving a menu must not make the document dirty.
