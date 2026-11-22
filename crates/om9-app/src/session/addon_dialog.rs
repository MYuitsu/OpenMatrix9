// SPDX-License-Identifier: LGPL-2.1-or-later
//! The options dialog of an interactive add-on command (Rhino 5 / Matrix 9 "Sweep 2 Rails
//! Options", "Surface From Curve Network", "Patch Surface Options" …).
//!
//! It shows up when the command has something to make, either because it waits only for
//! Enter with options to change (`want = nothing`) or because it previews a result and has
//! settings of its own. It shows the same values as the command line, as checkboxes, radio
//! buttons and fields:
//!
//! - a change is the same input as typing `Name=value` on the command line;
//! - a setting of the command (`SweepTolerance`, `EdgeTolerance` …) is remembered as the
//!   add-on's value and the preview is recomputed;
//! - OK is Enter (the right mouse button too) and Cancel is Esc.
//!
//! A command with a builder panel (`builder = true`) keeps its panel and gets no dialog.
use super::Controller;
use om9_addon_abi::session::{Input, OptionKind, Want};
use om9_addons::settings::{Kind, SettingValue};
use om9_core::{Error, Result};

/// What one row of the dialog edits.
#[derive(Debug, Clone, PartialEq)]
pub enum DialogKind {
    /// A checkbox (`Yes`/`No`).
    Toggle,
    /// Radio buttons or a list: `(value as sent, value as shown)`.
    Choice(Vec<(String, String)>),
    /// A number field.
    Number,
    /// A button (`Point`, `SetAll` …).
    Action,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DialogRow {
    /// The option name as the add-on knows it (`MaintainHeight`).
    pub name: String,
    /// The name in the interface language.
    pub label: String,
    /// The current value as sent (`Yes`, `Rebuild`, `0.01`); empty for an action.
    pub value: String,
    pub kind: DialogKind,
    /// A setting of the command (`sw_tolerance`): changed in the add-on's settings, not in
    /// the running command.
    pub setting: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AddonDialog {
    /// Catalog code of the command; the UI keeps the window per command.
    pub code: String,
    pub title: String,
    pub prompt: String,
    pub rows: Vec<DialogRow>,
    /// Enter would make the result (the command waits for nothing else).
    pub ready: bool,
    /// Words beside an option (`PointCount` → `(2)`, Rhino's current value).
    pub notes: std::collections::BTreeMap<String, String>,
    /// Lines under the options (`Maximum deviation: 0.012`).
    pub footer: Vec<String>,
    /// Framed groups, in order: `(title, option names)` (Rhino's "Rail curve options" …).
    pub groups: Vec<(String, Vec<String>)>,
    /// An option enabled only while another has one of some values:
    /// `PreserveFirstShape` → (`Simplify`, [`Rebuild`, `Refit`]).
    pub when: std::collections::BTreeMap<String, (String, Vec<String>)>,
}

impl AddonDialog {
    /// Whether the row named `name` can be changed now (see [`AddonDialog::when`]).
    pub fn enabled(&self, name: &str) -> bool {
        let Some((other, values)) = self.when.get(name) else { return true };
        self.rows
            .iter()
            .find(|r| &r.name == other)
            .is_none_or(|r| values.iter().any(|v| v.eq_ignore_ascii_case(&r.value)))
    }
}

/// `MaintainHeight` → `Maintain height`, `SweepTolerance` → `Sweep tolerance`.
pub fn humanize(name: &str) -> String {
    let mut out = String::new();
    for (i, ch) in name.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            out.push(' ');
            out.extend(ch.to_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}

/// An option's name in the interface language, or in plain English words.
fn shown(tr: &om9_addons::i18n::Tr, name: &str) -> String {
    let t = tr.option(name);
    if t != name {
        return t;
    }
    // Rhino's own words in its dialogs, where the option name is shorter.
    match name {
        "CurrentLayer" => "Create new object on current layer".into(),
        "PreserveTangents" => "Preserve end tangent directions".into(),
        "SelectMasterCurve" => "Select Master Curve".into(),
        "AddSlash" => "Add Slash".into(),
        "RemoveSlashes" => "Remove slashes".into(),
        "Simplify" => "Simplify".into(),
        _ => humanize(name),
    }
}

fn setting_text(kind: &Kind, value: &SettingValue) -> String {
    match (kind, value) {
        (Kind::Bool, SettingValue::Bool(b)) => if *b { "Yes" } else { "No" }.into(),
        (_, SettingValue::Float(f)) => om9_addon_abi::session::fmt(*f),
        (_, SettingValue::Int(i)) => i.to_string(),
        (_, SettingValue::Str(s)) => s.clone(),
        (_, v) => v.display(),
    }
}

impl Controller {
    /// The options dialog of the running add-on command, when it has one now.
    pub fn addon_dialog(&self) -> Option<AddonDialog> {
        let session = self.addon_session.as_ref()?;
        if matches!(session.code.as_str(), super::from_lines::CODE | super::extrude_lines::CODE | super::extrude_lines::tapered::CODE | super::extrude_point::CODE) {return None;}
        if self
            .addon_builder
            .as_ref()
            .is_some_and(|b| b.code == session.code)
        {
            return None;
        }
        let reply = &session.reply;
        let confirm = matches!(reply.want, Want::Nothing);
        let previewing = !reply.preview.is_empty() || !session.lines.is_empty();
        let tr = &self.tr;
        let mut rows: Vec<DialogRow> = Vec::new();
        for o in &reply.options {
            if o.name.eq_ignore_ascii_case("Undo") {
                continue;
            }
            let value = o.value.clone().unwrap_or_default();
            let kind = match &o.kind {
                OptionKind::Toggle => DialogKind::Toggle,
                OptionKind::Choice { choices } => DialogKind::Choice(
                    choices.iter().map(|c| (c.clone(), tr.value(c))).collect(),
                ),
                OptionKind::Number => DialogKind::Number,
                _ => DialogKind::Action,
            };
            rows.push(DialogRow {
                name: o.name.clone(),
                label: shown(tr, &o.name),
                value,
                kind,
                setting: None,
            });
        }
        let has_options = !rows.is_empty();
        // The command's own settings, unless the command line already has them.
        let info = self
            .addons
            .as_ref()
            .and_then(|h| h.lock().ok()?.command_info(&session.code));
        let mut settings = 0;
        if let Some(info) = &info {
            for (spec, value) in &info.settings {
                if rows.iter().any(|r| r.name.eq_ignore_ascii_case(&spec.option)) {
                    continue;
                }
                // A command that shows its own options decides which flags and choices apply
                // (Closed only for closed rails …); only its numeric settings are added.
                if has_options && matches!(spec.kind, Kind::Bool | Kind::Enum { .. }) {
                    continue;
                }
                let kind = match &spec.kind {
                    Kind::Bool => DialogKind::Toggle,
                    Kind::Enum { choices } => DialogKind::Choice(
                        choices
                            .iter()
                            .enumerate()
                            .map(|(i, c)| {
                                let shown = spec.choice_labels.get(i).cloned().unwrap_or_else(|| c.clone());
                                (c.clone(), tr.value(&shown))
                            })
                            .collect(),
                    ),
                    Kind::Int { .. } | Kind::Float { .. } => DialogKind::Number,
                    _ => continue,
                };
                let label = if tr.lang() == om9_addons::i18n::Lang::Vi {
                    spec.label.clone()
                } else {
                    humanize(&spec.option)
                };
                rows.push(DialogRow {
                    name: spec.option.clone(),
                    label,
                    value: setting_text(&spec.kind, value),
                    kind,
                    setting: Some(spec.key.clone()),
                });
                settings += 1;
            }
        }
        let show = (confirm && has_options) || (previewing && settings > 0) || (confirm && settings > 0);
        if !show {
            return None;
        }
        let name = tr.command(&session.code, &session.label);
        let title = if session.code == super::split_curves::CODE { session.label.clone() } else { match tr.lang() {
            om9_addons::i18n::Lang::Vi => format!("Tuỳ chọn {name}"),
            om9_addons::i18n::Lang::En => format!("{name} Options"),
        }};
        let mut notes = std::collections::BTreeMap::new();
        let mut footer = Vec::new();
        let mut groups = Vec::new();
        let mut when = std::collections::BTreeMap::new();
        let list = |t: &str| -> Vec<String> { t.split(',').map(|x| x.trim().to_owned()).filter(|x| !x.is_empty()).collect() };
        for (name, text) in &reply.notes {
            match name.as_str() {
                "" => footer.push(tr.text(text)),
                // "Title|A,B,C": a framed group of options.
                "@group" => {
                    if let Some((title, names)) = text.split_once('|') {
                        groups.push((tr.text(title), list(names)));
                    }
                }
                // "Name|Other=v1,v2": Name only while Other is v1 or v2.
                "@when" => {
                    if let Some((target, rule)) = text.split_once('|')
                        && let Some((other, values)) = rule.split_once('=')
                    {
                        when.insert(target.trim().to_owned(), (other.trim().to_owned(), list(values)));
                    }
                }
                _ => {
                    notes.insert(name.clone(), text.clone());
                }
            }
        }
        Some(AddonDialog {
            code: session.code.clone(),
            title,
            prompt: tr.text(&reply.prompt),
            rows,
            ready: confirm || previewing,
            notes,
            footer,
            groups,
            when,
        })
    }

    /// One change made in the dialog. `value` is empty for an action.
    pub fn set_addon_dialog_value(&mut self, name: &str, value: &str) -> Result<()> {
        let dialog = self.addon_dialog().ok_or(Error::Missing("options dialog"))?;
        let row = dialog
            .rows
            .iter()
            .find(|r| r.name == name)
            .ok_or(Error::Missing("dialog option"))?
            .clone();
        if row.kind == DialogKind::Number && value.trim().is_empty() {
            return Err(Error::Invalid("enter a number"));
        }
        let owner = self
            .addon_session
            .as_ref()
            .map(|s| s.owner.clone())
            .ok_or(Error::Missing("add-on session"))?;
        match &row.setting {
            Some(key) => {
                self.remember_setting(&owner, key, value)?;
                // The preview follows the new value.
                self.session_step(Input::Selection)
            }
            None => {
                let line = if value.is_empty() {
                    name.to_owned()
                } else {
                    format!("{name}={value}")
                };
                if !self.submit_session_text(&line)? {
                    return Err(Error::Missing("add-on session"));
                }
                // Remembered for the next run when the add-on has the same setting
                // (Rhino keeps the dialog's last values).
                if !value.is_empty() {
                    let key = self.addons.as_ref().and_then(|h| {
                        let code = self.addon_session.as_ref()?.code.clone();
                        h.lock()
                            .ok()?
                            .command_info(&code)?
                            .settings
                            .into_iter()
                            .find(|(s, _)| s.option.eq_ignore_ascii_case(name))
                            .map(|(s, _)| s)
                    });
                    if let Some(spec) = key {
                        let raw = match &spec.kind {
                            Kind::Enum { choices } => choices
                                .iter()
                                .zip(spec.choice_labels.iter().chain(std::iter::repeat(&String::new())))
                                .find(|(c, l)| c.eq_ignore_ascii_case(value) || l.eq_ignore_ascii_case(value))
                                .map(|(c, _)| c.clone()),
                            _ => Some(value.to_owned()),
                        };
                        if let Some(raw) = raw {
                            let _ = self.remember_setting(&owner, &spec.key, &raw);
                        }
                    }
                }
                Ok(())
            }
        }
    }

    fn remember_setting(&mut self, owner: &str, key: &str, raw: &str) -> Result<()> {
        let host = self.addons.clone().ok_or(Error::Unsupported("add-on host"))?;
        let raw = match raw {
            "Yes" => "true",
            "No" => "false",
            other => other,
        };
        host.lock()
            .map_err(|_| Error::Io("add-on host poisoned".into()))?
            .set_setting(owner, key, raw)
            .map(|_| ())
            .map_err(Error::Io)
    }
}
