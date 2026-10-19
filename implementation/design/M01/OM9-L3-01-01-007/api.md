# OM9-L3-01-01-007 — Global Status And Error Surface (typed API)

Owner module `OM9-L2-01-01`. Implementation owner:
`standalone/crates/om9-app/src/status_surface.rs` (the surface) and
`src/ui/chrome.rs` (the always-visible line). Reuse decision: **ADAPT**
QymCAD's "the operation tells the status what happened" threading and its
`Bottom` slot kind; **ADD** severity, acknowledgement and the bounded tail (see
`source_crosswalk.json`).

Oracle — positive: *Invalid command / OCCT diagnostic visible in status, not
lost in console.* Negative: *Error popup cannot swallow escape/cancel state or
dismiss silently.*

## 1. Typed API

```rust
// status_surface.rs — the model. No egui, no files, no command state.
pub enum StatusKind { Ready, Info, Warning, Error }
impl StatusKind { pub fn label(self) -> &'static str; pub fn is_failure(self) -> bool; }

pub struct StatusSurface { kind, message, revision, tail }
impl StatusSurface {
    pub const MAX_MESSAGE_BYTES: usize = 512;
    pub const MAX_TAIL: usize = 8;
    pub fn kind(&self) -> StatusKind;
    pub fn message(&self) -> &str;
    pub fn revision(&self) -> u64;                 // changes only when the line changes
    pub fn tail(&self) -> Vec<&str>;               // recent messages, oldest first
    pub fn report(&mut self, kind: StatusKind, message: &str) -> bool;
    pub fn info(&mut self, message: &str) -> bool;
    pub fn warning(&mut self, message: &str) -> bool;
    pub fn error(&mut self, message: &str) -> bool;
    /// Returns the line to Ready unless a failure is still unacknowledged.
    pub fn complete_success(&mut self) -> bool;
    /// The only way a failure leaves the line.
    pub fn acknowledge(&mut self) -> bool;
    pub fn line(&self) -> String;                  // "Error: Unknown command: Foo"
}

// chrome.rs — the surface on screen.
// The line is always visible; Dismiss appears only while the kind is a failure;
// hovering the line lists the recent messages (the tail).
```

## 2. Rules

| Situation | Result |
| --- | --- |
| a command fails | `kind = Error`, the message on the line, and the same text in the command history |
| a command succeeds afterwards | the failure stays: `complete_success` refuses while `kind.is_failure()` |
| a panel or file message arrives | it never replaces a failure (it goes to the history; see OM9-L3-01-01-003 N30) |
| a nonfatal diagnostic (warning) | `kind = Warning`, visible, and the previous *failure* is not silently swallowed |
| the user presses Dismiss | `kind = Ready`; only the failure that was on screen is acknowledged |
| the same message is reported twice | no change, `revision` does not move |
| the message is longer than 512 bytes | cut on a character boundary; newlines and tabs fold to spaces |
| the message is empty or blank | the line reads `Ready` |
| a second failure replaces the first | the first stays in the tail (and the history), the Dismiss that follows clears the line |
| Esc while a failure is on screen | the failure is not a dialog: Esc still cancels the running command (or the layer dialog, or the drag) |

`Esc` ownership deliberately does not live here: the surface never consumes a
key, never opens a modal and never answers for another owner.

## 3. Ownership

* The **surface** owns the current line, its severity, its revision and a bounded
  tail. It is pure: no IO, no document, no egui.
* The **command, file and layer owners** decide what to report and keep their own
  Esc/cancel state.
* The audit receipt carries `status_kind`, `status_message`, `status_tail` and
  `diagnostics`, so a failure can be checked after the fact, not only on screen.
