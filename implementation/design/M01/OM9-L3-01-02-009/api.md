# OM9-L3-01-02-009 — Command Repeat And Dispatch (typed API)

Owner module `OM9-L2-01-02` (backend framework, stage E01, gate **AUTO_ONLY**).
Implementation owner: `standalone/crates/om9-app/src/session.rs`. Reuse decision:
**ADAPT** the single doorway of `gui.rs`; **ADD** the repeat slot with an explicit
repeatability rule (see `source_crosswalk.json`).

Oracle — positive: *Repeat last repeatable action invokes same registered command
ID with new context.* Negative: *Nonrepeatable/file destructive commands cannot
be silently repeated.*

## 1. Typed API

```rust
impl Controller {
    /// The command an empty Enter would run again, if it may be repeated.
    pub fn repeatable_last(&self) -> Option<&str>;
    /// Only the interactive geometry commands are repeatable today.
    pub fn is_repeatable(feature: &str) -> bool;
    /// The raw slot, for the audit: empty, or a repeatable feature.
    pub fn repeat_slot(&self) -> Option<&str>;
    /// The last completed commands, bounded to 9.
    pub fn recent(&self) -> &VecDeque<String>;
}
```

## 2. Rules

| Situation | Result |
| --- | --- |
| LINE completes, then an empty Enter | the same registered command starts again in a **new session**, against the context of now (its own snapshot of document/view/revision) |
| POINT completes, then an empty Enter | POINT starts again and the next click commits a second point |
| `recent` | bounded to 9 entries, oldest dropped; it names the completed geometry commands |
| `is_repeatable("Open"/"Save"/"SaveAs"/"New"/"Close")` | `false` — a file or destructive command is never repeated, and never occupies the slot |
| `is_repeatable("Line"/"POINT"/"line")` | `true` (case-insensitive) |
| a file command runs after a geometry command | the slot still holds the geometry command: the empty Enter repeats that, never the file command |
| an empty Enter with nothing to repeat | `Missing("previous repeatable command")` |
| an empty Enter while a command is running | refused (`Invalid("point required")`): the running command owns the Enter |
| a session whose context is gone | not running, so nothing is repeated from it |

## 3. Ownership

* The **session** owns the repeat slot and the recent list; the **transcript**
  owns the typed history of the CMD frame.
* Repeatability is a property of the command, not of the history: a feature that
  is not repeatable can never be reached by an empty Enter, whatever the history
  contains.
* Repeating starts a **new** run: it takes a fresh context snapshot, so the rules
  of OM9-L3-01-02-003 apply to it like to any other run.
