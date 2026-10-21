// SPDX-License-Identifier: LGPL-2.1-or-later
use std::{collections::VecDeque, ops::Range};
/// One selectable text surface. All indices at the editor boundary are Unicode
/// character indices. Mutations intersect only the editable draft suffix.
#[derive(Debug, Clone)]
pub struct Transcript {
    history: String,
    prompt: String,
    draft: String,
    text: String,
    commands: VecDeque<String>,
    recall: Option<usize>,
    saved_draft: String,
}
impl Default for Transcript {
    fn default() -> Self {
        Self::new()
    }
}
impl Transcript {
    pub const MAX_BYTES: usize = 64 * 1024;
    pub const MAX_DRAFT_BYTES: usize = 4096;
    pub fn new() -> Self {
        Self {
            history: String::new(),
            prompt: "Command: ".into(),
            draft: String::new(),
            text: "Command: ".into(),
            commands: VecDeque::new(),
            recall: None,
            saved_draft: String::new(),
        }
    }
    fn rebuild(&mut self) {
        let available = Self::MAX_BYTES.saturating_sub(self.prompt.len() + self.draft.len());
        while self.history.len() > available {
            if let Some(i) = self.history.find('\n') {
                self.history.drain(..=i);
            } else {
                self.history.clear();
            }
        }
        self.text.clear();
        self.text.push_str(&self.history);
        self.text.push_str(&self.prompt);
        self.text.push_str(&self.draft);
    }
    pub fn text(&self) -> &str {
        &self.text
    }
    pub fn history(&self) -> &str {
        &self.history
    }
    pub fn draft(&self) -> &str {
        &self.draft
    }
    pub fn protected_chars(&self) -> usize {
        self.history.chars().count() + self.prompt.chars().count()
    }
    pub fn set_prompt(&mut self, prompt: &str) {
        self.prompt = prompt.chars().take(512).collect();
        self.rebuild();
    }
    pub fn set_draft(&mut self, draft: &str) {
        self.draft.clear();
        for ch in draft.chars() {
            let ch = if ch == '\r' || ch == '\n' { ' ' } else { ch };
            if self.draft.len() + ch.len_utf8() > Self::MAX_DRAFT_BYTES {
                break;
            }
            self.draft.push(ch);
        }
        self.recall = None;
        self.rebuild();
    }
    pub fn append_history(&mut self, line: &str) {
        let line: String = line.chars().take(Self::MAX_DRAFT_BYTES).collect();
        self.history.push_str(&line);
        self.history.push('\n');
        self.rebuild();
    }
    pub fn insert(&mut self, text: &str, index: usize) -> usize {
        let boundary = self.protected_chars();
        if index < boundary {
            return 0;
        }
        let index = index - boundary;
        if index > self.draft.chars().count() {
            return 0;
        }
        let byte = self
            .draft
            .char_indices()
            .nth(index)
            .map_or(self.draft.len(), |(i, _)| i);
        let mut inserted = String::new();
        for ch in text.chars() {
            let ch = if ch == '\r' || ch == '\n' { ' ' } else { ch };
            if self.draft.len() + inserted.len() + ch.len_utf8() > Self::MAX_DRAFT_BYTES {
                break;
            }
            inserted.push(ch);
        }
        let count = inserted.chars().count();
        self.draft.insert_str(byte, &inserted);
        self.recall = None;
        self.rebuild();
        count
    }
    pub fn delete(&mut self, range: Range<usize>) {
        let boundary = self.protected_chars();
        let len = self.draft.chars().count();
        let start = range.start.saturating_sub(boundary).min(len);
        let end = range.end.saturating_sub(boundary).min(len);
        if start >= end {
            return;
        }
        let start = self
            .draft
            .char_indices()
            .nth(start)
            .map_or(self.draft.len(), |(i, _)| i);
        let end = self
            .draft
            .char_indices()
            .nth(end)
            .map_or(self.draft.len(), |(i, _)| i);
        self.draft.drain(start..end);
        self.recall = None;
        self.rebuild();
    }
    pub fn record_command(&mut self, command: &str) {
        if !command.trim().is_empty() {
            self.commands
                .push_back(command.chars().take(Self::MAX_DRAFT_BYTES).collect());
            while self.commands.len() > 256 {
                self.commands.pop_front();
            }
        }
        self.recall = None;
    }
    pub fn recall_up(&mut self) {
        if self.commands.is_empty() {
            return;
        }
        let index = if let Some(i) = self.recall {
            i.saturating_sub(1)
        } else {
            self.saved_draft = self.draft.clone();
            self.commands.len() - 1
        };
        self.draft = self.commands[index].clone();
        self.recall = Some(index);
        self.rebuild();
    }
    pub fn recall_down(&mut self) {
        if let Some(i) = self.recall {
            if i + 1 < self.commands.len() {
                self.draft = self.commands[i + 1].clone();
                self.recall = Some(i + 1);
            } else {
                self.draft = self.saved_draft.clone();
                self.recall = None;
            }
            self.rebuild();
        }
    }
}

/// Strips directory prefixes from anything that looks like an absolute path, so the command history never
/// stores a user name, a network share or a confidential folder name from a file dialog. Only the file name
/// survives.
pub fn redact_paths(message: &str) -> String {
    let mut out = String::with_capacity(message.len());
    for token in message.split_inclusive(' ') {
        let trimmed = token.trim_end_matches(' ');
        let looks_like_path = (trimmed.len() > 2
            && trimmed.as_bytes()[1] == b':'
            && matches!(trimmed.as_bytes()[2], b'\\' | b'/'))
            || trimmed.starts_with('/')
            || trimmed.contains("\\\\");
        if looks_like_path {
            let name = trimmed
                .rsplit(['\\', '/'])
                .next()
                .filter(|name| !name.is_empty())
                .unwrap_or(trimmed);
            out.push_str(name);
        } else {
            out.push_str(trimmed);
        }
        if token.ends_with(' ') {
            out.push(' ');
        }
    }
    out
}

#[cfg(test)]
mod redaction_tests {
    use super::redact_paths;

    #[test]
    fn a_windows_path_keeps_only_the_file_name() {
        assert_eq!(
            redact_paths("Exported Rhino 3DM: C:\\Users\\me\\secret\\ring.3dm"),
            "Exported Rhino 3DM: ring.3dm"
        );
    }

    #[test]
    fn a_unc_path_keeps_only_the_file_name() {
        assert_eq!(
            redact_paths("Imported Rhino 3DM: \\\\server\\share\\client\\ring.3dm"),
            "Imported Rhino 3DM: ring.3dm"
        );
    }

    #[test]
    fn ordinary_words_are_untouched() {
        assert_eq!(redact_paths("Document saved."), "Document saved.");
    }
}
