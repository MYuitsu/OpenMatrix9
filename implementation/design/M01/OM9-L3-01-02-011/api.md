# OM9-L3-01-02-011 — Command Availability Preconditions (typed API)

Owner module `OM9-L2-01-02` (backend framework, stage E01, gate **AUTO_ONLY**).
Implementation owner: `standalone/crates/om9-core/src/capability.rs`. Reuse
decision: **ADAPT** the "the operation says why it cannot continue" habit of
`gui.rs`; **ADD** declared preconditions and a context-checked gate (see
`source_crosswalk.json`).

Oracle — positive: *Unavailable command disabled when wrong selection/document
state.* Negative: *Hidden implementation missing must not appear clickable or
mutate doc.*

## 1. Typed API

```rust
/// What a command needs from the session before it can be offered at all.
pub struct Preconditions { pub needs_document: bool, pub needs_selection: bool, pub needs_active_view: bool }

/// The session facts availability is decided against.
pub struct ContextState { pub has_document: bool, pub selection: usize, pub has_active_view: bool }
impl ContextState { pub fn empty_document() -> Self; }   // startup: a document, no selection, one view

impl CapabilityRegistry {
    /// Availability from the recorded receipts alone (no session context claimed).
    pub fn availability(&self, id: &str) -> Availability;
    /// Receipts **and** the session state.
    pub fn availability_with(&self, id: &str, context: &ContextState) -> Availability;
    /// The gate: evidence, preconditions, input kind, options, phase.
    pub fn admit_with(&self, id: &str, kind: GeometryKind, options: &[&str], phase: Phase,
                      context: &ContextState) -> Result<&Operation>;
    /// The same gate with no session context claimed.
    pub fn admit(&self, id: &str, kind: GeometryKind, options: &[&str], phase: Phase) -> Result<&Operation>;
}
```

## 2. Rules

| Situation | Result |
| --- | --- |
| no operation with that id | disabled: `Operation has no standalone implementation` — never hidden, never clickable |
| a precondition the session does not meet | disabled with the reason: `No document is open`, `No active view`, `Select an object first` |
| several preconditions fail | the first in that fixed order wins, so two callers read the same reason |
| a command with no preconditions | enabled in any session (subject to its receipts) |
| receipts missing | disabled with the receipt reason (`Rust semantic receipt is missing`, …), whatever the session looks like |
| `admit_with` on a ready session and a declared option typed in another case | admitted: the gate reads options with `Operation::option`, so it agrees with the descriptor |
| `admit_with` with an undeclared option / an undeclared input kind / an undeclared phase | refused (`unsupported operation options` / `… representation` / `… phase`) |
| `admit` (no context) on a command that needs a document | refused: no context is claimed, so the precondition cannot be honoured |
| any refusal | the gate only borrows the descriptor: no document change, no transaction, no undo entry |

## 3. Ownership

* The **descriptor** declares what the command needs; the **session** hands the
  facts in as `ContextState`; the **registry** decides and explains.
* Availability is advice for the interface (offer it disabled with the reason);
  `admit_with` is the enforcement for the run itself. Both use the same rule, so
  a command the interface offers as disabled cannot be run behind its back.
* The registry never hides a registered command and never mutates a document.
