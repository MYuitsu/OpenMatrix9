// SPDX-License-Identifier: LGPL-2.1-or-later
//! Shared command parameter contract. Owners provide snapshots; the UI never
//! keeps a second parameter value or chooses a command-specific parser.
use om9_core::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CommandOptionId {
    pub(crate) owner: &'static str,
    pub(crate) name: &'static str,
    /// Controller lifecycle generation that produced this snapshot.
    pub(crate) run: u64,
}
impl CommandOptionId {
    pub fn name(self) -> &'static str {
        self.name
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandOptionKind {
    Toggle,
    Choice,
    Number,
    Point,
    Text,
    Action,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommandValue {
    Flag(bool),
    Choice(&'static str),
    Number(Option<f64>),
    Integer(Option<usize>),
    Point(Option<om9_core::geometry::Point3>),
    Text(Option<String>),
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandOption {
    pub id: CommandOptionId,
    pub kind: CommandOptionKind,
    pub label: String,
    /// Typed current value from the owner; label is presentation only.
    pub value: CommandValue,
    /// A unique, advertised accelerator in the current snapshot. Cancel uses
    /// Escape, and numeric keys remain available for coordinate entry.
    pub shortcut: Option<char>,
    pub(crate) activation: Activation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Activation {
    Submit(&'static str),
    Draft(&'static str),
    Polyline(crate::polyline_options::PolylineOptionId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandOptionEffect {
    Applied,
    /// Start entering a value without sending an incomplete assignment to an
    /// owner. The normal submit path validates the completed value.
    Draft(String),
}

pub trait CommandOptionInterface {
    fn command_options(&self) -> Vec<CommandOption>;
    fn activate_command_option(&mut self, id: CommandOptionId) -> Result<CommandOptionEffect>;

    fn resolve_command_option(&self, text: &str) -> Option<CommandOptionId> {
        let text = text.trim();
        if text.is_empty() {
            return None;
        }
        let options = self.command_options();
        // Full names win over accelerators. Never reinterpret a value as a
        // different option just because two labels share their first letter.
        options
            .iter()
            .find(|option| {
                option.id.name.eq_ignore_ascii_case(text)
                    || matches!(option.activation, Activation::Draft(prefix) if prefix.eq_ignore_ascii_case(text))
            })
            .or_else(|| {
                options.iter().find(|option| {
                    option
                        .shortcut
                        .is_some_and(|key| text.eq_ignore_ascii_case(&key.to_string()))
                })
            })
            .map(|option| option.id)
    }
}

pub(crate) fn assign_shortcuts(options: &mut [CommandOption]) {
    let mut used = std::collections::BTreeSet::new();
    // Reserve initial letters first; collisions then use an unused letter in
    // their own label. A later collision cannot steal another option's initial.
    for option in options.iter_mut() {
        if option.id.name == "Cancel" {
            continue;
        }
        let first = option
            .id
            .name
            .chars()
            .next()
            .filter(char::is_ascii_alphabetic);
        if let Some(key) = first.map(|c| c.to_ascii_uppercase())
            && used.insert(key)
        {
            option.shortcut = Some(key);
        }
    }
    for option in options
        .iter_mut()
        .filter(|o| o.shortcut.is_none() && o.id.name != "Cancel")
    {
        option.shortcut = option
            .id
            .name
            .chars()
            .filter(char::is_ascii_alphabetic)
            .map(|c| c.to_ascii_uppercase())
            .find(|key| used.insert(*key));
    }
}
