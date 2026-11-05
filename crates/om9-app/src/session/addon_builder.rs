// SPDX-License-Identifier: LGPL-2.1-or-later
//! The builder of an add-on command (`builder = true` in its manifest).
//!
//! Starting such a command does not create anything. It opens a builder:
//!
//! - the command line shows the command and its options as `Size=1.5  Spacing=0.2 …`;
//!   typing `Size=2` (or clicking a token) changes a value;
//! - the BUILDERS panel below the Project Manager shows the command thumbnail and the same
//!   values as sliders and fields;
//! - the viewports show a live preview of the result for the selected curves. The preview
//!   is drawn over the scene and is never a document object;
//! - Enter (or Accept) creates the result in one transaction and remembers the values as the
//!   add-on's defaults for next time; Esc (or Cancel) closes the builder and changes nothing.
//!
//! The values live in the builder until it is accepted, so changing them never touches the
//! user's preferences file and there is no separate settings dialog to open.
use super::Controller;
use om9_addons::settings::{Kind, SettingSpec, SettingValue};
use om9_core::{Error, Result};
use std::collections::BTreeMap;

/// The preview the viewports draw while a builder is open, in document units.
#[derive(Debug, Clone, Default)]
pub struct BuilderPreview {
    /// Feature edges of every previewed object (sharp or open edges only).
    pub edges: Vec<[[f64; 3]; 2]>,
    /// Number of objects Enter would create.
    pub count: usize,
    /// What the add-on reported for these values (count, real gap, …).
    pub messages: Vec<String>,
    /// Why there is no preview (no curve selected, invalid value, add-on refusal).
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AddonBuilder {
    /// Catalog code (`BuilderGemOnCrv`).
    pub code: String,
    /// Display label (`Gem On Crv`).
    pub label: String,
    /// The Matrix command name typed in CMD (`gvGemOnCrv`), when the catalog has one.
    pub typed: Option<String>,
    pub owner: String,
    pub addon_name: String,
    pub specs: Vec<SettingSpec>,
    pub values: BTreeMap<String, SettingValue>,
    /// Bumped on every value change; the preview is recomputed when it or the selection moves.
    pub revision: u64,
    preview_key: Option<(u64, u64, u64)>,
    pub preview: BuilderPreview,
    /// The panel of an interactive command: the session previews and Enter goes to it.
    pub interactive: bool,
}

impl AddonBuilder {
    /// The current value of a setting.
    pub fn value(&self, key: &str) -> Option<&SettingValue> {
        self.values.get(key)
    }
    /// `Size=1.5`, `Layout=Dàn đều` … in declaration order.
    pub fn tokens(&self) -> Vec<(String, String)> {
        self.specs
            .iter()
            .map(|s| {
                let v = self.values.get(&s.key).unwrap_or(&s.default);
                (
                    s.key.clone(),
                    format!("{}={}", s.option, display_value(s, v)),
                )
            })
            .collect()
    }
    /// The setting an option name refers to: its `option`, key or label, case-insensitive,
    /// or a unique prefix of an option name.
    pub fn find(&self, name: &str) -> Option<&SettingSpec> {
        let n = name.trim().to_lowercase();
        if n.is_empty() {
            return None;
        }
        if let Some(s) = self
            .specs
            .iter()
            .find(|s| s.option.to_lowercase() == n || s.key == n || s.label.to_lowercase() == n)
        {
            return Some(s);
        }
        let mut hits = self
            .specs
            .iter()
            .filter(|s| s.option.to_lowercase().starts_with(&n));
        match (hits.next(), hits.next()) {
            (Some(s), None) => Some(s),
            _ => None,
        }
    }
}

/// A value as the user reads it: unit for lengths, label for enum choices.
pub fn display_value(spec: &SettingSpec, v: &SettingValue) -> String {
    match (&spec.kind, v) {
        (Kind::Enum { choices }, SettingValue::Str(s)) => choices
            .iter()
            .position(|c| c == s)
            .and_then(|i| spec.choice_labels.get(i).cloned())
            .unwrap_or_else(|| s.clone()),
        (Kind::Bool, SettingValue::Bool(b)) => if *b { "Yes" } else { "No" }.into(),
        (Kind::Float { .. }, SettingValue::Float(f)) => {
            let t = format!("{f:.3}");
            t.trim_end_matches('0').trim_end_matches('.').to_owned()
        }
        _ => v.display(),
    }
}

/// What one line of input meant while a builder is open.
enum BuilderInput<'a> {
    Accept,
    Set(&'a str, &'a str),
    /// A bare option name: toggles a flag, cycles a choice. A word that is not an option
    /// closes the builder and runs as a command.
    Step(&'a str),
}

fn classify(text: &str) -> BuilderInput<'_> {
    let t = text.trim();
    if t.is_empty() {
        return BuilderInput::Accept;
    }
    if let Some((name, value)) = t.split_once('=') {
        return BuilderInput::Set(name.trim(), value.trim());
    }
    BuilderInput::Step(t)
}

/// Feature edges of a triangle mesh: open edges and edges whose faces meet at more than
/// ~20°, so the preview reads as an outline rather than a wireframe of every triangle.
pub fn feature_edges(vertices: &[[f64; 3]], triangles: &[[u32; 3]]) -> Vec<[usize; 2]> {
    fn normal(v: &[[f64; 3]], t: &[u32; 3]) -> [f64; 3] {
        let (a, b, c) = (v[t[0] as usize], v[t[1] as usize], v[t[2] as usize]);
        let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let w = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let n = [
            u[1] * w[2] - u[2] * w[1],
            u[2] * w[0] - u[0] * w[2],
            u[0] * w[1] - u[1] * w[0],
        ];
        let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        if l > 0.0 {
            [n[0] / l, n[1] / l, n[2] / l]
        } else {
            [0.0; 3]
        }
    }
    let cos_limit = 20f64.to_radians().cos();
    let mut faces: BTreeMap<(usize, usize), Vec<[f64; 3]>> = BTreeMap::new();
    for t in triangles {
        if t.iter().any(|&i| i as usize >= vertices.len()) {
            continue;
        }
        let n = normal(vertices, t);
        for k in 0..3 {
            let (a, b) = (t[k] as usize, t[(k + 1) % 3] as usize);
            faces.entry((a.min(b), a.max(b))).or_default().push(n);
        }
    }
    faces
        .into_iter()
        .filter(|(_, ns)| {
            ns.len() != 2 || {
                let (p, q) = (ns[0], ns[1]);
                p[0] * q[0] + p[1] * q[1] + p[2] * q[2] < cos_limit
            }
        })
        .map(|((a, b), _)| [a, b])
        .collect()
}

impl Controller {
    pub fn addon_builder(&self) -> Option<&AddonBuilder> {
        self.addon_builder.as_ref()
    }

    /// Open the builder of `code` with the add-on's remembered values.
    pub(super) fn start_addon_builder(&mut self, code: &str) -> Result<()> {
        let host = self
            .addons
            .clone()
            .ok_or(Error::Unsupported("add-on host"))?;
        let info = host
            .lock()
            .map_err(|_| Error::Io("add-on host poisoned".into()))?
            .command_info(code)
            .ok_or(Error::Unsupported("add-on command"))?;
        if self.addon_session.as_ref().is_none_or(|s| s.code != code) {
            self.cancel();
        } else {
            self.cancel_addon_builder();
        }
        self.addon_builder = Some(AddonBuilder {
            code: code.to_owned(),
            label: info.command.label.clone(),
            typed: self.addon_typed.get(code).cloned(),
            owner: info.owner,
            addon_name: info.addon_name,
            specs: info.settings.iter().map(|(s, _)| s.clone()).collect(),
            values: info.settings.into_iter().map(|(s, v)| (s.key, v)).collect(),
            revision: 0,
            preview_key: None,
            preview: BuilderPreview::default(),
            interactive: false,
        });
        self.events.emit(
            crate::command_events::EventKind::Started,
            code,
            "builder",
            None,
        );
        self.refresh_builder_preview();
        Ok(())
    }

    /// Change one builder value from typed text (`2`, `0,25 mm`, `fixed_gap`, a choice label).
    pub fn set_builder_text(&mut self, key: &str, raw: &str) -> Result<()> {
        let b = self
            .addon_builder
            .as_mut()
            .ok_or(Error::Missing("builder"))?;
        let spec = b
            .specs
            .iter()
            .find(|s| s.key == key)
            .ok_or(Error::Missing("builder option"))?;
        // A choice may be typed as its label ("Giữ khe") or by its value ("fixed_gap").
        let raw = match &spec.kind {
            Kind::Enum { choices } => spec
                .choice_labels
                .iter()
                .position(|l| l.eq_ignore_ascii_case(raw.trim()))
                .map(|i| choices[i].clone())
                .or_else(|| {
                    choices
                        .iter()
                        .find(|c| c.eq_ignore_ascii_case(raw.trim()))
                        .cloned()
                })
                .unwrap_or_else(|| raw.to_owned()),
            _ => raw.to_owned(),
        };
        let value = spec
            .parse_user(&raw)
            .map_err(|e| Error::Io(format!("{}: {e}", spec.label)))?;
        b.values.insert(key.to_owned(), value);
        b.revision += 1;
        Ok(())
    }

    /// Change one builder value from a widget (slider, checkbox, choice).
    pub fn set_builder_value(&mut self, key: &str, value: SettingValue) -> Result<()> {
        let b = self
            .addon_builder
            .as_mut()
            .ok_or(Error::Missing("builder"))?;
        let spec = b
            .specs
            .iter()
            .find(|s| s.key == key)
            .ok_or(Error::Missing("builder option"))?;
        let value = spec
            .validate(&value)
            .map_err(|e| Error::Io(format!("{}: {e}", spec.label)))?;
        if b.values.get(key) != Some(&value) {
            b.values.insert(key.to_owned(), value);
            b.revision += 1;
        }
        Ok(())
    }

    /// Toggle a flag or move a choice to the next one (a click on its command-line token).
    pub fn step_builder_value(&mut self, key: &str) -> Result<()> {
        let b = self
            .addon_builder
            .as_ref()
            .ok_or(Error::Missing("builder"))?;
        let spec = b
            .specs
            .iter()
            .find(|s| s.key == key)
            .ok_or(Error::Missing("builder option"))?;
        let next = match (&spec.kind, b.values.get(key)) {
            (Kind::Bool, Some(SettingValue::Bool(v))) => SettingValue::Bool(!v),
            (Kind::Enum { choices }, Some(SettingValue::Str(v))) => {
                let i = choices.iter().position(|c| c == v).unwrap_or(0);
                SettingValue::Str(choices[(i + 1) % choices.len()].clone())
            }
            _ => return Err(Error::Invalid("option needs a value")),
        };
        self.set_builder_value(key, next)
    }

    /// Back to the values the add-on ships with.
    pub fn reset_builder_defaults(&mut self) {
        if let Some(b) = self.addon_builder.as_mut() {
            b.values = b
                .specs
                .iter()
                .map(|s| (s.key.clone(), s.default.clone()))
                .collect();
            b.revision += 1;
        }
    }

    /// Recompute the preview when the values, the selection or the document changed.
    /// Cheap when nothing changed; the UI calls it every frame while a builder is open.
    pub fn refresh_builder_preview(&mut self) {
        // An interactive builder shows what its session last answered. The session can answer
        // after the builder opened (a command started on a preselected stone previews at
        // once), so this is read every frame, not only when the key below changes.
        if let (Some(b), Some(s)) = (self.addon_builder.as_mut(), self.addon_session.as_ref())
            && b.interactive
        {
            b.preview.count = s.reply.preview.len();
            b.preview.messages = s.reply.messages.clone();
            b.preview.error = s.reply.error.clone();
        }
        let Some(b) = self.addon_builder.as_ref() else {
            return;
        };
        let key = (
            b.revision,
            self.selection_revision,
            self.document().context_revision,
        );
        if b.preview_key == Some(key) {
            return;
        }
        if b.interactive {
            // The session draws the preview; tell it the panel values changed.
            let changed = b.preview_key.is_some_and(|old| old.0 != key.0);
            if let Some(b) = self.addon_builder.as_mut() {
                b.preview_key = Some(key);
            }
            if changed {
                let _ = self.session_step(om9_addon_abi::session::Input::Selection);
            }
            if let (Some(b), Some(s)) = (self.addon_builder.as_mut(), self.addon_session.as_ref()) {
                b.preview.count = s.reply.preview.len();
                b.preview.messages = s.reply.messages.clone();
                b.preview.error = s.reply.error.clone();
            }
            return;
        }
        let (code, values) = (b.code.clone(), b.values.clone());
        let to_mm = self.document().page_unit.factor();
        let preview = match self.addon_invocation(&code, &values) {
            Err(e) => BuilderPreview {
                error: Some(e.to_string()),
                ..Default::default()
            },
            Ok(out) => {
                let mut edges = Vec::new();
                for m in &out.meshes {
                    for [a, c] in feature_edges(&m.vertices, &m.triangles) {
                        let (p, q) = (m.vertices[a], m.vertices[c]);
                        edges.push([
                            [p[0] / to_mm, p[1] / to_mm, p[2] / to_mm],
                            [q[0] / to_mm, q[1] / to_mm, q[2] / to_mm],
                        ]);
                    }
                }
                BuilderPreview {
                    edges,
                    count: out.meshes.len(),
                    messages: out.messages,
                    error: None,
                }
            }
        };
        if let Some(b) = self.addon_builder.as_mut() {
            b.preview = preview;
            b.preview_key = Some(key);
        }
    }

    /// Enter / Accept: create the result with the builder's values, remember them, close.
    pub fn accept_addon_builder(&mut self) -> Result<()> {
        let b = self
            .addon_builder
            .clone()
            .ok_or(Error::Missing("builder"))?;
        if b.interactive {
            self.session_step(om9_addon_abi::session::Input::Enter)?;
            if self.addon_session.is_none() {
                self.store_builder_values(&b);
                self.addon_builder = None;
            }
            return Ok(());
        }
        let created = self.run_addon_with(&b.code, &b.values, &b.label)?;
        if created == 0 {
            // Nothing to place (no curve, curve too short): the builder stays open so the
            // user can pick a curve or change a value; the reason is already reported.
            return Err(Error::Invalid("nothing to create"));
        }
        self.store_builder_values(&b);
        self.addon_builder = None;
        Ok(())
    }

    pub(super) fn store_builder_values(&mut self, b: &AddonBuilder) {
        if let Some(host) = self.addons.clone()
            && let Ok(mut h) = host.lock()
            && let Err(e) = h.store_settings(&b.owner, &b.values)
        {
            self.addon_messages
                .push(format!("Không lưu được thông số mặc định: {e}"));
        }
    }

    /// Esc / Cancel: close the builder; nothing is created or stored.
    pub fn cancel_addon_builder(&mut self) -> bool {
        match self.addon_builder.take() {
            Some(b) => {
                self.events.emit(
                    crate::command_events::EventKind::Finished,
                    &b.code,
                    "cancelled",
                    Some(crate::command_events::EventOutcome::Cancelled),
                );
                true
            }
            None => false,
        }
    }

    /// A line typed while a builder is open. `Ok(true)` = consumed by the builder;
    /// `Ok(false)` = not an option, the builder closed and the caller runs the text.
    pub(super) fn submit_builder_text(&mut self, text: &str) -> Result<bool> {
        let Some(b) = self.addon_builder.as_ref() else {
            return Ok(false);
        };
        match classify(text) {
            BuilderInput::Accept => self.accept_addon_builder().map(|_| true),
            BuilderInput::Set(name, value) => {
                let key = b
                    .find(name)
                    .map(|s| s.key.clone())
                    .ok_or_else(|| Error::Io(format!("{}: không có tuỳ chọn {name:?}", b.label)))?;
                self.set_builder_text(&key, value).map(|_| true)
            }
            BuilderInput::Step(name) => match b.find(name).map(|s| (s.key.clone(), s.kind.clone()))
            {
                Some((key, Kind::Bool | Kind::Enum { .. })) => {
                    self.step_builder_value(&key).map(|_| true)
                }
                Some(_) => Err(Error::Io(format!(
                    "{}: gõ {}=giá trị",
                    b.label,
                    b.find(name).map(|s| s.option.as_str()).unwrap_or(name)
                ))),
                None => {
                    self.cancel_addon_builder();
                    Ok(false)
                }
            },
        }
    }

    /// The command-line prompt while a builder is open.
    pub(super) fn addon_builder_prompt(&self) -> Option<String> {
        let b = self.addon_builder.as_ref()?;
        let curves = self
            .selection
            .iter()
            .filter_map(|id| self.document().entities.get(id))
            .filter(|e| matches!(e.geometry, om9_core::geometry::Geometry::Curve(_)))
            .count();
        Some(if curves == 0 {
            format!("{}: chọn đường cong (Esc huỷ): ", b.label)
        } else {
            format!(
                "{}: chỉnh thông số, Enter = đặt {} viên (Esc huỷ): ",
                b.label, b.preview.count
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_lines_are_classified() {
        assert!(matches!(classify("  "), BuilderInput::Accept));
        assert!(matches!(classify("Size=2"), BuilderInput::Set("Size", "2")));
        assert!(matches!(
            classify("size = 0,5 mm"),
            BuilderInput::Set("size", "0,5 mm")
        ));
        assert!(matches!(classify("Layout"), BuilderInput::Step("Layout")));
    }

    #[test]
    fn a_closed_box_outline_has_its_twelve_edges() {
        let v = [
            [0., 0., 0.],
            [1., 0., 0.],
            [1., 1., 0.],
            [0., 1., 0.],
            [0., 0., 1.],
            [1., 0., 1.],
            [1., 1., 1.],
            [0., 1., 1.],
        ];
        let t = [
            [0, 2, 1],
            [0, 3, 2],
            [4, 5, 6],
            [4, 6, 7],
            [0, 1, 5],
            [0, 5, 4],
            [1, 2, 6],
            [1, 6, 5],
            [2, 3, 7],
            [2, 7, 6],
            [3, 0, 4],
            [3, 4, 7],
        ];
        // The face diagonals are flat (0°) and drop out; the 12 cube edges stay.
        assert_eq!(feature_edges(&v, &t).len(), 12);
    }
}
