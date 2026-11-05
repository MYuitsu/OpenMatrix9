// SPDX-License-Identifier: LGPL-2.1-or-later
//! Tools › Add-ons… window, Install from Disk, drag-and-drop install and add-on shortcuts.
use super::MatrixApp;
use crate::addons::FeatureName;
use crate::session::InvocationOrigin;
use eframe::egui::{self, Color32, RichText};
use om9_addons::host::{AddonStatus, InstallOptions, RepoKind};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Default)]
pub(crate) struct AddonsUi {
    pub open: bool,
    pub search: String,
    pub install_request: bool,
    /// (ok, text) shown at the top of the window.
    pub notice: Option<(bool, String)>,
    /// The add-on whose enable is waiting for the user to accept its permissions.
    pub pending_grant: Option<String>,
}

enum Action {
    Enable {
        id: String,
        grant: bool,
    },
    Disable(String),
    Uninstall(String),
    /// Start one of the add-on's commands (same path as its menu icon).
    Run(String),
    InstallFromDisk,
    /// An add-on-wide setting changed (its language …).
    Set {
        id: String,
        key: String,
        value: String,
    },
}

/// An add-on-wide choice shown on the add-on's row: (key, label, [(value, shown)], current).
type WideChoice = (String, String, Vec<(String, String)>, String);

struct RowView {
    id: String,
    name: String,
    version: String,
    tagline: Option<String>,
    repo: RepoKind,
    status: AddonStatus,
    commands: Vec<(String, String, Option<String>)>,
    permissions: Vec<(String, String)>,
    wide: Vec<WideChoice>,
}

impl MatrixApp {
    pub(crate) fn refresh_addons(&mut self) {
        let names: Vec<FeatureName<'_>> = self
            .features
            .iter()
            .map(|f| FeatureName {
                id: &f.id,
                command: f.command.as_deref(),
            })
            .collect();
        self.controller.refresh_addon_names(&names);
    }

    /// Open the user's add-on host (interactive app only; audits and tests never touch
    /// the user's configuration).
    pub fn attach_addons(&mut self) {
        match crate::addons::open_host() {
            Ok((host, log)) => {
                for line in log {
                    self.transcript.append_history(&format!("Add-ons: {line}"));
                }
                let names: Vec<FeatureName<'_>> = self
                    .features
                    .iter()
                    .map(|f| FeatureName {
                        id: &f.id,
                        command: f.command.as_deref(),
                    })
                    .collect();
                self.controller.attach_addons(host, &names);
            }
            Err(error) => self
                .transcript
                .append_history(&format!("Add-ons unavailable: {error}")),
        }
    }

    /// Install a package chosen by the user (dialog or drop) and enable it.
    pub(crate) fn install_addon_path(&mut self, path: &Path) {
        let Some(host) = self.controller.addon_host().cloned() else {
            self.addons_ui.notice = Some((false, "Add-ons are unavailable in this session".into()));
            return;
        };
        let result = host.lock().map(|mut h| {
            h.install_zip(
                path,
                &InstallOptions {
                    enable: true,
                    ..Default::default()
                },
            )
        });
        let shown = crate::transcript::redact_paths(&path.display().to_string());
        match result {
            Ok(Ok(report)) => {
                let mut text = format!(
                    "Installed add-on {} {}{}",
                    report.id,
                    report.version,
                    if report.enabled { " (enabled)" } else { "" }
                );
                for w in &report.warnings {
                    text.push_str(&format!("\n  {w}"));
                    if w.contains("cấp quyền") {
                        self.addons_ui.pending_grant = Some(report.id.clone());
                    }
                }
                self.transcript.append_history(&text);
                self.status.info(&format!("Add-on {} installed", report.id));
                self.addons_ui.notice = Some((true, text));
                self.addons_ui.open = true;
            }
            Ok(Err(errors)) => {
                let text = format!("Add-on {shown} was not installed: {}", errors.join("; "));
                self.transcript.append_history(&text);
                self.status.error(&text);
                self.addons_ui.notice = Some((false, text));
                self.addons_ui.open = true;
            }
            Err(_) => {
                self.addons_ui.notice = Some((false, "Add-on host is unavailable".into()));
            }
        }
        self.refresh_addons();
    }

    /// Per-frame add-on work: dropped `.zip` files, shortcuts and the window.
    pub(crate) fn addons_frame(&mut self, ctx: &egui::Context) {
        if self.controller.addon_host().is_none() {
            return;
        }
        let dropped: Vec<PathBuf> = ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .map(|f| f.path().to_path_buf())
                .filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("zip")))
                .collect()
        });
        for path in dropped {
            self.install_addon_path(&path);
        }
        if ctx.input(|i| !i.raw.hovered_files.is_empty()) {
            egui::Area::new(egui::Id::new("addon-drop-hint"))
                .order(egui::Order::Foreground)
                .anchor(egui::Align2::CENTER_CENTER, [0., 0.])
                .show(ctx, |ui| {
                    egui::Frame::popup(ui.style()).show(ui, |ui| {
                        ui.label(RichText::new("Drop an OM9 add-on .zip to install it").size(18.));
                    });
                });
        }
        if !self.controller.is_running() {
            let shortcuts = self
                .controller
                .addon_host()
                .and_then(|h| h.lock().ok().map(|h| h.shortcuts()))
                .unwrap_or_default();
            for (keys, code) in shortcuts {
                if let Some((mods, key)) = crate::addons::egui_shortcut(&keys)
                    && ctx.input_mut(|i| i.consume_key(mods, key))
                {
                    self.invoke(&code, InvocationOrigin::Menu);
                }
            }
        }
        if self.addons_ui.open {
            let mut open = true;
            let mut actions = Vec::new();
            egui::Window::new("Add-ons")
                .id(egui::Id::new("om9-addons"))
                .open(&mut open)
                .default_width(560.)
                .default_height(520.)
                .vscroll(true)
                .show(ctx, |ui| self.addons_window_body(ui, &mut actions));
            self.addons_ui.open = open;
            self.apply_addon_actions(actions);
        }
    }

    fn addon_rows(&self) -> Vec<RowView> {
        let Some(host) = self.controller.addon_host() else {
            return Vec::new();
        };
        let Ok(h) = host.lock() else {
            return Vec::new();
        };
        let shortcuts: BTreeMap<String, String> =
            h.shortcuts().into_iter().map(|(k, c)| (c, k)).collect();
        h.search(&self.addons_ui.search)
            .into_iter()
            .map(|r| {
                let labels: BTreeMap<String, String> = h
                    .bound_commands()
                    .into_iter()
                    .filter(|(_, owner, _)| owner == &r.id)
                    .map(|(code, _, label)| {
                        let shown = self.controller.tr().command(&code, &label);
                        (code, shown)
                    })
                    .collect();
                let wide = h
                    .settings(&r.id)
                    .unwrap_or_default()
                    .into_iter()
                    .filter(|(spec, _)| spec.addon_wide)
                    .filter_map(|(spec, value)| match (&spec.kind, value) {
                        (
                            om9_addons::settings::Kind::Enum { choices },
                            om9_addons::settings::SettingValue::Str(current),
                        ) => Some((
                            spec.key.clone(),
                            spec.label.clone(),
                            choices
                                .iter()
                                .enumerate()
                                .map(|(i, c)| {
                                    (c.clone(), spec.choice_labels.get(i).unwrap_or(c).clone())
                                })
                                .collect(),
                            current,
                        )),
                        _ => None,
                    })
                    .collect();
                RowView {
                    commands: r
                        .commands
                        .iter()
                        .map(|c| {
                            (
                                c.clone(),
                                labels.get(c).cloned().unwrap_or_else(|| c.clone()),
                                shortcuts.get(c).cloned(),
                            )
                        })
                        .collect(),
                    permissions: r.permissions.clone().into_iter().collect(),
                    wide,
                    id: r.id,
                    name: r.name,
                    version: r.version.unwrap_or_else(|| "?".into()),
                    tagline: r.tagline,
                    repo: r.repo,
                    status: r.status,
                }
            })
            .collect()
    }

    fn addons_window_body(&mut self, ui: &mut egui::Ui, actions: &mut Vec<Action>) {
        ui.horizontal(|ui| {
            ui.label("Search:");
            ui.add(egui::TextEdit::singleline(&mut self.addons_ui.search).desired_width(220.));
            if ui
                .button("Install from Disk…")
                .on_hover_text(
                    "Choose an OM9 add-on .zip; you can also drag the .zip onto the window",
                )
                .clicked()
            {
                actions.push(Action::InstallFromDisk);
            }
        });
        if let Some((ok, text)) = &self.addons_ui.notice {
            ui.colored_label(
                if *ok {
                    Color32::from_rgb(40, 120, 60)
                } else {
                    Color32::from_rgb(170, 40, 40)
                },
                text,
            );
        }
        ui.separator();
        let rows = self.addon_rows();
        if rows.is_empty() {
            ui.label(
                "No add-ons installed. Use Install from Disk… or drop a .zip onto the window.",
            );
        }
        for row in rows {
            let enabled = row.status == AddonStatus::Enabled;
            ui.horizontal(|ui| {
                let mut on = enabled;
                let can_toggle = !matches!(
                    row.status,
                    AddonStatus::Invalid(_) | AddonStatus::Incompatible(_)
                );
                let response = ui.add_enabled(
                    can_toggle,
                    egui::Checkbox::new(&mut on, RichText::new(&row.name).strong()),
                );
                if response.changed() {
                    actions.push(if on {
                        Action::Enable {
                            id: row.id.clone(),
                            grant: false,
                        }
                    } else {
                        Action::Disable(row.id.clone())
                    });
                }
                ui.label(&row.version);
                ui.weak(row.repo.as_str());
            });
            let (color, text) = match &row.status {
                AddonStatus::Enabled => (Color32::from_rgb(40, 120, 60), "Enabled".to_owned()),
                AddonStatus::Disabled => (Color32::GRAY, "Disabled".to_owned()),
                AddonStatus::Invalid(e) => (
                    Color32::from_rgb(170, 40, 40),
                    format!("Invalid: {}", e.join("; ")),
                ),
                AddonStatus::Incompatible(e) => {
                    (Color32::from_rgb(170, 90, 0), format!("Incompatible: {e}"))
                }
                AddonStatus::Failed(e) => {
                    (Color32::from_rgb(170, 40, 40), format!("Not enabled: {e}"))
                }
            };
            ui.colored_label(color, text);
            if let Some(t) = &row.tagline {
                ui.label(t);
            }
            // Add-on-wide choices (Ngôn ngữ / Language …): one row each.
            for (key, label, choices, current) in &row.wide {
                ui.horizontal(|ui| {
                    ui.label(label);
                    let shown = choices
                        .iter()
                        .find(|(v, _)| v == current)
                        .map_or(current.as_str(), |(_, s)| s.as_str());
                    egui::ComboBox::from_id_salt(("addon-wide", &row.id, key))
                        .selected_text(shown)
                        .show_ui(ui, |ui| {
                            for (value, text) in choices {
                                if ui.selectable_label(value == current, text).clicked()
                                    && value != current
                                {
                                    actions.push(Action::Set {
                                        id: row.id.clone(),
                                        key: key.clone(),
                                        value: value.clone(),
                                    });
                                }
                            }
                        });
                });
            }
            if !row.permissions.is_empty() && !enabled {
                let list = row
                    .permissions
                    .iter()
                    .map(|(k, v)| format!("{k} ({v})"))
                    .collect::<Vec<_>>()
                    .join(", ");
                ui.label(format!("Requests: {list}"));
                if ui.button("Enable and grant permissions").clicked() {
                    actions.push(Action::Enable {
                        id: row.id.clone(),
                        grant: true,
                    });
                }
            }
            // Uninstall stays in reach: a package such as OM9 Matrix lists hundreds of commands.
            if row.repo == RepoKind::User && ui.button("Uninstall").clicked() {
                actions.push(Action::Uninstall(row.id.clone()));
            }
            egui::CollapsingHeader::new(format!("Commands ({})", row.commands.len()))
                .id_salt(("addon-details", &row.id))
                .default_open(
                    self.addons_ui.pending_grant.as_deref() == Some(&row.id)
                        || (enabled && row.commands.len() <= 20),
                )
                .show(ui, |ui| {
                    for (code, label, keys) in &row.commands {
                        ui.horizontal(|ui| {
                            let run = ui
                                .add_enabled(
                                    enabled,
                                    egui::Button::new(RichText::new(label).strong()),
                                )
                                .on_hover_text("Run the command (same as its menu icon)");
                            if run.clicked() {
                                actions.push(Action::Run(code.clone()));
                            }
                            ui.weak(code);
                            if let Some(k) = keys {
                                ui.label(format!("[{k}]"));
                            }
                        });
                    }
                    ui.weak(
                        "Parameters are set while the command runs: on the command line \
                         (Size=1.5 …) and in the BUILDERS panel below Projects. The values you \
                         accept become the defaults for next time.",
                    );
                });
            ui.separator();
        }
    }

    fn apply_addon_actions(&mut self, actions: Vec<Action>) {
        if actions.is_empty() {
            return;
        }
        let Some(host) = self.controller.addon_host().cloned() else {
            return;
        };
        for action in actions {
            let Ok(mut h) = host.lock() else {
                return;
            };
            match action {
                Action::InstallFromDisk => {
                    drop(h);
                    self.addons_ui.install_request = true;
                }
                Action::Enable { id, grant } => match h.enable(&id, grant) {
                    Ok(warnings) => {
                        self.addons_ui.pending_grant = None;
                        let mut text = format!("Enabled add-on {id}");
                        for w in warnings {
                            text.push_str(&format!("\n  {w}"));
                        }
                        self.transcript.append_history(&text);
                        self.addons_ui.notice = Some((true, text));
                    }
                    Err(e) => {
                        if e.contains("cấp quyền") {
                            self.addons_ui.pending_grant = Some(id.clone());
                        }
                        self.addons_ui.notice = Some((false, format!("{id}: {e}")));
                    }
                },
                Action::Disable(id) => {
                    let r = h.disable(&id);
                    self.addons_ui.notice = Some(match r {
                        Ok(()) => (true, format!("Disabled add-on {id}")),
                        Err(e) => (false, format!("{id}: {e}")),
                    });
                }
                Action::Uninstall(id) => {
                    let r = h.uninstall(&id);
                    self.addons_ui.notice = Some(match r {
                        Ok(()) => (true, format!("Uninstalled add-on {id}")),
                        Err(e) => (false, format!("{id}: {e}")),
                    });
                }
                Action::Set { id, key, value } => {
                    self.addons_ui.notice = Some(match h.set_setting(&id, &key, &value) {
                        Ok(_) => (true, format!("{id}: {key} = {value}")),
                        Err(e) => (false, format!("{id}: {e}")),
                    });
                }
                Action::Run(code) => {
                    drop(h);
                    self.addons_ui.open = false;
                    self.refresh_addons();
                    self.invoke(&code, InvocationOrigin::Menu);
                    return;
                }
            }
        }
        self.refresh_addons();
    }

    /// Windows: native picker for Install from Disk; other platforms use drag-and-drop.
    pub(crate) fn service_addon_picker(&mut self, ctx: &egui::Context, frame: &eframe::Frame) {
        if !std::mem::take(&mut self.addons_ui.install_request) {
            return;
        }
        match crate::native_picker::pick_addon_zip(frame) {
            Ok(Some(path)) => self.install_addon_path(&path),
            Ok(None) => {}
            Err(error) => {
                self.addons_ui.notice = Some((
                    false,
                    format!("{error}. Drag the add-on .zip onto the window instead."),
                ));
            }
        }
        ctx.request_repaint();
    }
}
