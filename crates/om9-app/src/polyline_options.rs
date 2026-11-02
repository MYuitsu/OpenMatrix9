// SPDX-License-Identifier: LGPL-2.1-or-later
//! The shared Polyline parameter interface. It contains no UI or geometry logic.
use om9_core::{Result, geometry::Point3};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PolylineOptionId {
    PersistentClose,
    Mode,
    Helpers,
    Length,
    Close,
    Direction,
    Center,
    Undo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolylineOptionKind {
    Toggle,
    Choice,
    Number,
    Point,
    Action,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PolylineOptionValue {
    Flag(bool),
    Choice(&'static str),
    Number(Option<f64>),
    Point(Option<Point3>),
    None,
}

impl PolylineOptionId {
    pub fn name(self) -> &'static str {
        match self {
            Self::PersistentClose => "PersistentClose",
            Self::Mode => "Mode",
            Self::Helpers => "Helpers",
            Self::Length => "Length",
            Self::Close => "Close",
            Self::Direction => "Direction",
            Self::Center => "Center",
            Self::Undo => "Undo",
        }
    }

    pub fn shortcut(self) -> char {
        self.name().chars().next().expect("nonempty option name")
    }

    pub fn kind(self) -> PolylineOptionKind {
        match self {
            Self::PersistentClose | Self::Helpers => PolylineOptionKind::Toggle,
            Self::Mode => PolylineOptionKind::Choice,
            Self::Length => PolylineOptionKind::Number,
            Self::Direction | Self::Center => PolylineOptionKind::Point,
            Self::Close | Self::Undo => PolylineOptionKind::Action,
        }
    }
}

/// A current, owner-derived parameter value. Absence from the snapshot means
/// the parameter is unavailable in this input stage (e.g. Length in Arc mode).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PolylineCommandOption {
    pub id: PolylineOptionId,
    pub value: PolylineOptionValue,
}

impl PolylineCommandOption {
    pub fn label(self) -> String {
        let name = self.id.name();
        match self.value {
            PolylineOptionValue::Flag(value) => {
                format!("{name}={}", if value { "Yes" } else { "No" })
            }
            PolylineOptionValue::Choice(value) => format!("{name}={value}"),
            PolylineOptionValue::Number(Some(value)) => format!("{name}={value}"),
            PolylineOptionValue::Point(Some([x, y, z])) => format!("{name}={x},{y},{z}"),
            _ => name.into(),
        }
    }

    pub fn matches(self, text: &str) -> bool {
        let text = text.trim();
        text.eq_ignore_ascii_case(self.id.name())
            || text.eq_ignore_ascii_case(&self.id.shortcut().to_string())
    }
}

pub trait PolylineOptionInterface {
    /// One source for prompt labels, current values, shortcuts and click targets.
    fn polyline_command_options(&self) -> Vec<PolylineCommandOption>;

    /// Activates a currently available option through the normal typed parser.
    /// Value/point options enter their input stage; toggles/choices update now.
    fn activate_polyline_option(&mut self, option: PolylineOptionId) -> Result<()>;

    fn resolve_polyline_option(&self, text: &str) -> Option<PolylineOptionId> {
        self.polyline_command_options()
            .into_iter()
            .find(|option| option.matches(text))
            .map(|option| option.id)
    }
}
