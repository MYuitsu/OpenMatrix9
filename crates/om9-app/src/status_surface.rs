// SPDX-License-Identifier: LGPL-2.1-or-later
//! Global status and nonfatal error surface for the standalone shell.
//!
//! Owner: plan module `OM9-L2-01-01` L3-007. The command transcript keeps the
//! full history; this surface keeps the *current* state so a failure cannot be
//! lost by scrolling or by a later successful command. It never clears the
//! pending command draft and never answers a dialog: Esc/cancel ownership stays
//! with the command, file and layer owners.
use std::fmt::Write as _;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StatusKind {
    Ready,
    Info,
    Warning,
    Error,
}

impl StatusKind {
    pub fn label(self) -> &'static str {
        match self {
            StatusKind::Ready => "Ready",
            StatusKind::Info => "Info",
            StatusKind::Warning => "Warning",
            StatusKind::Error => "Error",
        }
    }

    pub fn is_failure(self) -> bool {
        matches!(self, StatusKind::Error)
    }
}

/// One always-visible line plus a bounded rolling tail of recent messages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusSurface {
    kind: StatusKind,
    message: String,
    revision: u64,
    tail: std::collections::VecDeque<String>,
}

impl Default for StatusSurface {
    fn default() -> Self {
        Self::new()
    }
}

impl StatusSurface {
    pub const MAX_MESSAGE_BYTES: usize = 512;
    pub const MAX_TAIL: usize = 8;

    pub fn new() -> Self {
        Self {
            kind: StatusKind::Ready,
            message: "Ready".into(),
            revision: 0,
            tail: std::collections::VecDeque::new(),
        }
    }

    pub fn kind(&self) -> StatusKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Most recent messages, oldest first, including the current one.
    pub fn tail(&self) -> Vec<&str> {
        self.tail.iter().map(String::as_str).collect()
    }

    /// Replace the current status. Returns true when the visible state changed
    /// so a caller can decide to persist or repaint.
    pub fn report(&mut self, kind: StatusKind, message: &str) -> bool {
        let message = Self::bounded(message);
        if self.kind == kind && self.message == message {
            return false;
        }
        self.kind = kind;
        self.message = message;
        self.revision = self.revision.wrapping_add(1);
        self.push_tail();
        true
    }

    pub fn info(&mut self, message: &str) -> bool {
        self.report(StatusKind::Info, message)
    }

    pub fn warning(&mut self, message: &str) -> bool {
        self.report(StatusKind::Warning, message)
    }

    pub fn error(&mut self, message: &str) -> bool {
        self.report(StatusKind::Error, message)
    }

    /// A successful operation returns the line to Ready **unless** an
    /// unacknowledged failure is still on screen.
    pub fn complete_success(&mut self) -> bool {
        if self.kind.is_failure() {
            return false;
        }
        self.report(StatusKind::Ready, "Ready")
    }

    /// Explicit acknowledgement: only the user (or a new document/command
    /// scope) clears a failure, never a later success by itself.
    pub fn acknowledge(&mut self) -> bool {
        self.report(StatusKind::Ready, "Ready")
    }

    /// One-line rendering for the status bar.
    pub fn line(&self) -> String {
        let mut line = String::with_capacity(self.message.len() + 12);
        let _ = write!(line, "{}: {}", self.kind.label(), self.message);
        line
    }

    fn push_tail(&mut self) {
        if self.tail.len() == Self::MAX_TAIL {
            self.tail.pop_front();
        }
        self.tail.push_back(self.message.clone());
    }

    fn bounded(message: &str) -> String {
        let mut bounded = String::with_capacity(message.len().min(Self::MAX_MESSAGE_BYTES));
        for ch in message.chars() {
            if bounded.len() + ch.len_utf8() > Self::MAX_MESSAGE_BYTES {
                break;
            }
            if ch == '\n' || ch == '\r' || ch == '\t' {
                bounded.push(' ');
            } else {
                bounded.push(ch);
            }
        }
        if bounded.trim().is_empty() {
            "Ready".into()
        } else {
            bounded
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failure_survives_a_later_success_and_clears_only_on_acknowledge() {
        let mut status = StatusSurface::new();
        assert_eq!(status.kind(), StatusKind::Ready);
        assert!(status.error("Unknown command: Foo"));
        assert_eq!(status.message(), "Unknown command: Foo");
        assert!(!status.complete_success());
        assert_eq!(status.kind(), StatusKind::Error);
        assert!(status.acknowledge());
        assert_eq!(status.kind(), StatusKind::Ready);
        assert!(!status.complete_success());
        assert!(status.info("Line accepted"));
        assert!(status.complete_success());
        assert_eq!(status.kind(), StatusKind::Ready);
    }

    #[test]
    fn revision_changes_only_when_the_visible_state_changes() {
        let mut status = StatusSurface::new();
        let before = status.revision();
        assert!(status.warning("Layer locked"));
        assert_eq!(status.revision(), before + 1);
        assert!(!status.warning("Layer locked"));
        assert_eq!(status.revision(), before + 1);
        assert!(status.warning("Layer hidden"));
        assert_eq!(status.revision(), before + 2);
    }

    #[test]
    fn messages_are_bounded_on_char_boundaries_and_newlines_are_folded() {
        let mut status = StatusSurface::new();
        status.error(&"ẩ".repeat(1000));
        assert!(status.message().len() <= StatusSurface::MAX_MESSAGE_BYTES);
        assert!(status.message().chars().all(|ch| ch == 'ẩ'));
        status.info("line one\nline two\r\nline three\ttab");
        assert_eq!(status.message(), "line one line two  line three tab");
        status.info("   ");
        assert_eq!(status.message(), "Ready");
    }

    #[test]
    fn tail_is_bounded_and_ordered_oldest_first() {
        let mut status = StatusSurface::new();
        for index in 0..StatusSurface::MAX_TAIL + 5 {
            status.info(&format!("message {index}"));
        }
        let tail = status.tail();
        assert_eq!(tail.len(), StatusSurface::MAX_TAIL);
        assert_eq!(tail[0], "message 5");
        assert_eq!(
            tail[tail.len() - 1],
            format!("message {}", StatusSurface::MAX_TAIL + 4)
        );
        assert!(status.line().starts_with("Info: "));
    }
}
