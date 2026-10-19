# OM9-L3-01-02-004 — Interactive Command State Machine (typed API)

Owner module `OM9-L2-01-02` (backend framework, stage E01, gate **AUTO_ONLY**).
Implementation owner: `standalone/crates/om9-app/src/session.rs`. Reuse decision:
**ADAPT** the bar state machine of `gui.rs` behind one doorway; **ADD** the
explicit phase (see `source_crosswalk.json`).

Oracle — positive: *Two-click point command progresses Start->AwaitPoint->
AwaitPoint->Commit, Escape aborts.* Negative: *Extra click/out-of-order Enter
cannot commit partial geometry.*

## 1. Typed API

```rust
pub enum CmdPhase { Idle, AwaitPoint, AwaitEnd }

impl Controller {
    pub fn phase(&self) -> CmdPhase;              // Idle when nothing is running
    pub fn prompt(&self) -> &'static str;         // follows the phase, always
    pub fn is_running(&self) -> bool;
    pub fn accept_world_point(&mut self, point: Point3) -> Result<()>;
    pub fn submit(&mut self, text: &str) -> Result<()>;   // Enter / typed input
    pub fn cancel(&mut self);                     // Escape
}
```

The phase is **derived** from the one session state (feature, `start`), so the
prompt, the refusals and the audit read the same thing.

## 2. Rules

| Step | Phase after | Prompt after |
| --- | --- | --- |
| `dispatch LINE` | `AwaitPoint` | `Start of line (BothSides=No): ` |
| first point | `AwaitEnd` | `End of line (BothSides=No): ` |
| second point | `Idle` | `Command: ` — one entity, one undo entry |
| `dispatch POINT` | `AwaitPoint` | `Location of point: ` |
| first point of POINT | `Idle` | `Command: ` |
| `BothSides=Yes` while a line runs | unchanged | `… (BothSides=Yes): ` |
| Escape at any phase | `Idle` | `Command: ` — no entity, no undo entry, a later click is `Missing("active command")` |
| Enter while a point is awaited | unchanged | unchanged — `Invalid("point required")`, no entity |
| a click after the commit | `Idle` | — `Missing("active command")`, no second entity |
| a second `dispatch` while running | unchanged | — `Busy` |
| a zero-length line | `AwaitEnd` (the command stays open) | — `Invalid("nonzero finite line length")`, no entity |
| a command name typed while running (`line`) | unchanged | — `Invalid("x,y[,z] coordinate")`, no entity |
| the context is lost (another view/document) | `Idle` | `Command: ` — the phase machine and the prompt never disagree |

## 3. Ownership

* The **session** owns the phase of the run; the document is only touched at the
  transition that completes the command, inside one transaction.
* Enter is **not** a commit: while a point is awaited it is refused, so a partial
  command can never be committed by pressing Enter.
* Escape aborts the command and nothing else (the context and cancel rules of
  OM9-L3-01-02-003/008 decide what an abort rolls back).

> **Correction (2026-10-16, same day).** The first version of this rule made ANY change of the active view lose the command context. The pre-existing `tests/chrome_actions.rs` (`view_tab_restores_cmd_focus_and_preserves_numeric_draft`, `view_menu_restores_cmd_focus_…`) proved that wrong: switching a view TAB must keep a running command and its numeric draft. The rule is now: a command loses its context when its **document identity** changes, when the **view it started in is gone**, or when the revision/epoch moves. A view-tab switch keeps the run, which still uses the view and plane it captured. The fixtures (K1/K2 of 003, N5 of 004, U4 of 006, S3 of 009) and their tests were corrected to that trigger, and `chrome_actions` is green again (5/5).
