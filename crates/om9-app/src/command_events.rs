// SPDX-License-Identifier: LGPL-2.1-or-later
//! Typed command events and the observer seam (OM9-L3-01-02-010).
//!
//! One run of a command reports what happened through one ordered log:
//! `Started`, any number of `Progress`/`Warn`, then exactly one `Finished`.
//! Observers are told about each event through a seam that CONTAINS a panicking
//! observer, so a broken listener can never corrupt the dispatch or the
//! transaction it is watching. Pure: no document, no session, no egui.
use std::collections::VecDeque;
use std::panic::{AssertUnwindSafe, catch_unwind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    Started,
    Progress,
    Warn,
    Finished,
}

impl EventKind {
    pub fn label(self) -> &'static str {
        match self {
            EventKind::Started => "started",
            EventKind::Progress => "progress",
            EventKind::Warn => "warn",
            EventKind::Finished => "finished",
        }
    }
}

/// How a run ended. Only the `Finished` event carries one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventOutcome {
    Committed,
    Cancelled,
    Refused,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandEvent {
    /// 1, 2, 3 … in the order the events happened, over the whole session.
    pub sequence: u64,
    pub kind: EventKind,
    /// The feature id of the command the event belongs to.
    pub command: String,
    pub detail: String,
    pub outcome: Option<EventOutcome>,
}

/// The ordered event log. Bounded, so a chatty command cannot grow it forever.
#[derive(Debug, Clone)]
pub struct EventLog {
    next: u64,
    events: VecDeque<CommandEvent>,
}

impl Default for EventLog {
    fn default() -> Self {
        Self::new()
    }
}

impl EventLog {
    pub const MAX_EVENTS: usize = 256;

    pub fn new() -> Self {
        Self {
            next: 1,
            events: VecDeque::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn events(&self) -> &VecDeque<CommandEvent> {
        &self.events
    }

    /// The last event, which is what a status line would show.
    pub fn last(&self) -> Option<&CommandEvent> {
        self.events.back()
    }

    /// Emit one event and return it. The sequence is assigned here, so two
    /// callers can never produce the same number and the order is total.
    pub fn emit(
        &mut self,
        kind: EventKind,
        command: &str,
        detail: &str,
        outcome: Option<EventOutcome>,
    ) -> CommandEvent {
        let event = CommandEvent {
            sequence: self.next,
            kind,
            command: command.to_owned(),
            detail: detail.to_owned(),
            outcome,
        };
        self.next = self.next.saturating_add(1);
        if self.events.len() == Self::MAX_EVENTS {
            self.events.pop_front();
        }
        self.events.push_back(event.clone());
        event
    }

    /// True when the stored events are strictly increasing.
    pub fn sequence_is_monotonic(&self) -> bool {
        self.events
            .iter()
            .map(|event| event.sequence)
            .try_fold(0, |previous, sequence| {
                (sequence > previous).then_some(sequence)
            })
            .is_some()
    }

    /// The number of events a run produced for one command since its `Started`.
    pub fn run_len(&self, command: &str) -> usize {
        let mut count = 0;
        for event in self.events.iter().rev() {
            if event.command != command {
                continue;
            }
            count += 1;
            if event.kind == EventKind::Started {
                break;
            }
        }
        count
    }
}

/// A listener. Kept as a trait object so the shell can attach one without the
/// core knowing what it is (log, window, test double …).
pub trait Observer {
    fn observe(&mut self, event: &CommandEvent);
}

/// Tell one observer about one event. Returns false when the observer panicked:
/// the panic is contained here and the caller carries on, so a broken listener
/// cannot abort a dispatch or leave a transaction half-applied.
pub fn notify(observer: &mut dyn Observer, event: &CommandEvent) -> bool {
    catch_unwind(AssertUnwindSafe(|| observer.observe(event))).is_ok()
}

/// Tell every observer, counting the ones that survived a panic.
pub fn notify_all(observers: &mut [&mut dyn Observer], event: &CommandEvent) -> usize {
    let mut survivors = 0;
    for observer in observers.iter_mut() {
        if notify(*observer, event) {
            survivors += 1;
        }
    }
    survivors
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Recorder {
        seen: Vec<u64>,
    }
    impl Observer for Recorder {
        fn observe(&mut self, event: &CommandEvent) {
            self.seen.push(event.sequence);
        }
    }

    struct Exploding;
    impl Observer for Exploding {
        fn observe(&mut self, _event: &CommandEvent) {
            panic!("observer exploded");
        }
    }

    #[test]
    fn a_run_reports_started_progress_warn_and_finished_in_order() {
        let mut log = EventLog::new();
        log.emit(EventKind::Started, "line", "start", None);
        log.emit(EventKind::Progress, "line", "first point", None);
        log.emit(EventKind::Warn, "line", "zero length refused", None);
        log.emit(
            EventKind::Finished,
            "line",
            "committed",
            Some(EventOutcome::Committed),
        );
        let kinds: Vec<EventKind> = log.events().iter().map(|event| event.kind).collect();
        assert_eq!(
            kinds,
            vec![
                EventKind::Started,
                EventKind::Progress,
                EventKind::Warn,
                EventKind::Finished
            ]
        );
        assert!(log.sequence_is_monotonic());
        assert_eq!(log.events()[0].sequence, 1);
        assert_eq!(log.run_len("line"), 4);
        assert_eq!(log.last().unwrap().outcome, Some(EventOutcome::Committed));
    }

    #[test]
    fn the_log_is_bounded_and_the_sequence_still_rises() {
        let mut log = EventLog::new();
        for index in 0..EventLog::MAX_EVENTS + 10 {
            log.emit(EventKind::Progress, "line", &format!("step {index}"), None);
        }
        assert_eq!(log.len(), EventLog::MAX_EVENTS);
        assert!(log.sequence_is_monotonic());
        assert_eq!(
            log.events().back().unwrap().sequence,
            (EventLog::MAX_EVENTS + 10) as u64
        );
    }

    #[test]
    fn a_panicking_observer_is_contained_and_the_others_still_hear_it() {
        let mut log = EventLog::new();
        let event = log.emit(EventKind::Started, "line", "start", None);
        let mut good = Recorder { seen: Vec::new() };
        let mut bad = Exploding;
        let survivors = notify_all(&mut [&mut bad, &mut good], &event);
        assert_eq!(survivors, 1, "the panic was contained, not propagated");
        assert_eq!(good.seen, vec![event.sequence]);
        // The log is untouched by the failure.
        assert_eq!(log.len(), 1);
        assert!(log.sequence_is_monotonic());
    }
}
