# OM9-L3-01-02-003 — Command Context And Active Document (typed API)

Owner module `OM9-L2-01-02` (backend framework, stage E01, gate **AUTO_ONLY**).
Implementation owner: `standalone/crates/om9-app/src/session.rs`. Reuse decision:
**ADAPT** the single doorway of `gui.rs`; **ADD** the document/view snapshot and
the cancellation on context loss (see `source_crosswalk.json`).

Oracle — positive: *Context includes active document, viewport and typed
selection consistently.* Negative: *Close active doc during command must
cancel/refuse without mutation.*

## 1. Typed API

```rust
pub struct CommandContext {
    pub document: DocumentId,
    pub view: om9_scene::workspace::ViewId,
    pub selection: Vec<ObjectId>,   // typed, in id order
    pub revision: u64,              // document.context_revision
    pub epoch: u64,                 // publication epoch
}

impl Controller {
    /// The session facts a command would start against right now.
    pub fn context(&self) -> CommandContext;
    /// Stale when the document, the view, the revision or the epoch moved.
    pub fn validate_context(&self, context: &CommandContext) -> Result<()>;
    /// Idle | AwaitPoint | AwaitEnd  (OM9-L3-01-02-004).
    pub fn phase(&self) -> CmdPhase;
    /// Drop a session whose context is gone; true when one was dropped.
    pub fn cancel_if_stale(&mut self) -> bool;
}
```

A `CommandSession` stores the snapshot it started with, so every later step is
checked against the world the command was invoked in.

## 2. Rules

| Situation | Result |
| --- | --- |
| an untouched session | `context()` is deterministic; two calls are equal |
| the active view is switched while a command asks for a point | the command is **not running** (`is_running()` false, `phase()` Idle, `prompt()` back to `Command: `), `validate_context` is `Stale`, and a click is refused with `Missing("active command")` |
| a snapshot is validated after the view changed and then restored | valid again: no revision moved, so nothing else was disturbed |
| the document revision or the publication epoch moved (a document was closed, replaced or edited outside the command) | `Stale("command document/view context")` |
| the selection changes during a point command | the command continues: a point command takes points, not objects, and the selection is recorded for the commands that need it |
| a layer change made by the command itself | the session's snapshot follows that one transaction (`session/layers.rs`), so the command is not killed by its own change |
| losing the context | **no mutation**: the refusals happen before any entity is prepared or committed |

## 3. Ownership

* The **session** owns the snapshot of the run; the **document** and the
  **workspace** own the truth it is compared against.
* A stale session is not silently re-pointed at the new document or view: it is
  cancelled, and the caller can report that the command was dropped.
* The snapshot is read-only data: no geometry, no transactions.

> **Correction (2026-10-16, same day).** The first version of this rule made ANY change of the active view lose the command context. The pre-existing `tests/chrome_actions.rs` (`view_tab_restores_cmd_focus_and_preserves_numeric_draft`, `view_menu_restores_cmd_focus_…`) proved that wrong: switching a view TAB must keep a running command and its numeric draft. The rule is now: a command loses its context when its **document identity** changes, when the **view it started in is gone**, or when the revision/epoch moves. A view-tab switch keeps the run, which still uses the view and plane it captured. The fixtures (K1/K2 of 003, N5 of 004, U4 of 006, S3 of 009) and their tests were corrected to that trigger, and `chrome_actions` is green again (5/5).
