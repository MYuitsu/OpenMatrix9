# OM9-L3-01-02-010 — Command Event And Observer API (typed API)

Owner module `OM9-L2-01-02` (backend framework, stage E01, gate **AUTO_ONLY**).
Implementation owner: `standalone/crates/om9-app/src/command_events.rs` with the
session that emits. Reuse decision: **ADAPT** the "the run reports what happened"
habit of `gui.rs`; **ADD** typed events, the total order and the contained
observer seam (see `source_crosswalk.json`).

Oracle — positive: *Commands emit started/progress/warn/finished typed events
with monotonic ordering.* Negative: *Observer panic/exception handled without
corrupting dispatch transaction.*

## 1. Typed API

```rust
pub enum EventKind { Started, Progress, Warn, Finished }
pub enum EventOutcome { Committed, Cancelled, Refused }

pub struct CommandEvent {
    pub sequence: u64,            // 1, 2, 3 … over the whole session
    pub kind: EventKind,
    pub command: String,          // the feature id of the run
    pub detail: String,
    pub outcome: Option<EventOutcome>,   // only on Finished
}

pub struct EventLog { … }
impl EventLog {
    pub const MAX_EVENTS: usize = 256;
    pub fn emit(&mut self, kind, command, detail, outcome) -> CommandEvent;
    pub fn events(&self) -> &VecDeque<CommandEvent>;
    pub fn last(&self) -> Option<&CommandEvent>;
    pub fn sequence_is_monotonic(&self) -> bool;
    pub fn run_len(&self, command: &str) -> usize;   // back to the latest Started
}

pub trait Observer { fn observe(&mut self, event: &CommandEvent); }
pub fn notify(observer: &mut dyn Observer, event: &CommandEvent) -> bool;   // false = panicked, contained
pub fn notify_all(observers: &mut [&mut dyn Observer], event: &CommandEvent) -> usize;

// session
pub struct Controller { pub events: EventLog, … }
impl Controller { pub fn warn(&mut self, detail: &str); }
```

## 2. Rules

| Situation | Events emitted |
| --- | --- |
| `dispatch LINE` | `Started(line)` |
| first point of a two-point command | `Progress(line, "first point")` |
| an option or a point refused inside the run (`Controller::warn`) | `Warn(line, detail)` — the run continues |
| the command commits | `Finished(line, Committed)` |
| Escape cancels the run | `Finished(line, Cancelled)` |
| the coordinator does not publish | `Finished(line, Refused)` |
| a second run | its `Started` carries the next sequence number: the order is total across runs |
| a command refused before it started (zero length, `Busy`) | **no event** — nothing ran, so nothing is invented |
| `warn` with no run | nothing |
| the log reaches 256 events | the oldest is dropped, the sequence keeps rising |
| an observer panics | `notify` contains it and returns `false`; the other observers still hear the event, the log is untouched, and the run (and its transaction) carries on |

## 3. Ownership

* The **log** owns the order: the sequence is assigned where the event is created,
  so two callers can never produce the same number.
* The **session** owns what is worth reporting; the **observer** is told, never
  asked: a listener cannot change a run.
* A panicking listener is a listener problem, not a document problem: the seam
  contains it (`catch_unwind`) and reports the survivor count instead of
  propagating the panic into the dispatch.
