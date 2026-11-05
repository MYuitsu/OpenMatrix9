// SPDX-License-Identifier: LGPL-2.1-or-later
//! Running commands provided by add-ons (Preferences › Add-ons).
//!
//! The add-on receives copies only: the selected curves sampled to polylines in world
//! millimetres, the active CPlane normal and its validated settings. It returns meshes
//! through the host, which are checked again and inserted in **one transaction**, so one
//! Undo removes everything the command created. Gems are tagged with the `om9.gem`
//! extension so Tools › Super Select: Gems finds them.
use super::Controller;
use crate::addons::{FeatureName, SharedHost};
use om9_addons::runtime::{CommandInput, CommandOutput, CurveInput};
use om9_addons::settings::SettingValue;
use om9_core::{
    Error, Result,
    geometry::{Geometry, Mesh},
    model::{Entity, Extension, GEM_EXTENSION_SLOT},
    transaction::{Mutation, TransactionRequest},
};
use std::collections::BTreeMap;

/// Samples per selected curve. 1025 = 2^10 + 1 keeps polycurve seams exact for
/// power-of-two segment counts; the tangent error at this density is far below 0.01°.
pub const CURVE_SAMPLES: usize = 1025;

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

impl Controller {
    /// Attach the add-on host and the feature rows used to resolve typed command names.
    pub fn attach_addons(&mut self, host: SharedHost, features: &[FeatureName<'_>]) {
        self.addons = Some(host);
        self.refresh_addon_names(features);
    }
    /// Carry the add-on host and its name table onto a replacement controller
    /// (File › New / Open / Revert). Messages not yet shown move with them.
    pub(super) fn adopt_addons(&mut self, previous: &mut Self) {
        if self.addons.is_none() {
            self.addons = previous.addons.take();
            self.addon_names = std::mem::take(&mut previous.addon_names);
            self.addon_typed = std::mem::take(&mut previous.addon_typed);
            self.tr = std::mem::take(&mut previous.tr);
        }
        self.addon_messages
            .append(&mut std::mem::take(&mut previous.addon_messages));
    }
    pub fn addon_host(&self) -> Option<&SharedHost> {
        self.addons.as_ref()
    }
    /// The interface language of add-on commands: menus, command names, sub-options and
    /// prompts. English keeps every text as written.
    pub fn tr(&self) -> &om9_addons::i18n::Tr {
        &self.tr
    }
    /// Rebuild the command-name table after an add-on was installed, enabled or disabled.
    pub fn refresh_addon_names(&mut self, features: &[FeatureName<'_>]) {
        let (tr, errors) = match self.addons.as_ref().map(|h| h.lock()) {
            Some(Ok(host)) => host.translator(),
            _ => (om9_addons::i18n::Tr::english(), Vec::new()),
        };
        self.tr = tr;
        self.addon_messages
            .extend(errors.into_iter().map(|e| format!("Add-ons: {e}")));
        self.addon_names = match &self.addons {
            Some(host) => match host.lock() {
                Ok(host) => crate::addons::provided_names(&host, features),
                Err(_) => Default::default(),
            },
            None => Default::default(),
        };
        // code → the Matrix name typed in CMD, echoed when the command starts from a menu.
        self.addon_typed = self
            .addon_names
            .values()
            .filter_map(|code| {
                let feature = crate::icons::feature_for(code)?;
                features
                    .iter()
                    .find(|f| f.id == feature)
                    .and_then(|f| f.command)
                    .map(|cmd| (code.clone(), cmd.trim().to_owned()))
            })
            .collect();
        // A builder whose command is no longer provided (add-on disabled) closes.
        if let Some(b) = &self.addon_builder
            && !self.addon_names.contains_key(&b.code.to_ascii_lowercase())
        {
            self.addon_builder = None;
        }
    }
    /// The Matrix command name an add-on command is typed as (`gvGemOnCrv`).
    pub fn addon_typed_name(&self, code: &str) -> Option<&str> {
        self.addon_typed.get(code).map(String::as_str)
    }
    /// The catalog code an add-on provides for a typed name, feature id or code.
    pub fn addon_code(&self, name: &str) -> Option<String> {
        let key = name.trim().to_ascii_lowercase();
        (!key.is_empty())
            .then(|| self.addon_names.get(&key).cloned())
            .flatten()
    }
    /// The add-on command an OM9 icon names: by the icon's catalog code (MainMenu.ini key),
    /// else by its name as a manifest alias (`Circle`).
    pub fn icon_addon(&self, icon: &str) -> Option<String> {
        crate::icons::command_key_for(icon)
            .and_then(|key| self.addon_code(key))
            .or_else(|| self.addon_code(icon))
    }
    /// What an OM9 icon runs and whether it is enabled. An enabled add-on that provides the
    /// icon's catalog code runs; otherwise OM9's own command (its feature available and its
    /// menu catalog row implemented). `fallback` is a feature the caller found for the icon
    /// by other means.
    pub fn icon_target(&self, icon: &str, fallback: Option<&str>) -> Option<IconTarget> {
        if let Some(code) = self.icon_addon(icon) {
            return Some(IconTarget {
                command: code,
                enabled: true,
                addon: true,
            });
        }
        let curve = crate::curve_menu::by_icon(icon);
        let surface = crate::surface_menu::by_icon(icon);
        let builtin = curve
            .map(|binding| binding.feature)
            .or_else(|| surface.map(|binding| binding.feature))
            .or_else(|| crate::icons::feature_for(icon))
            .or(fallback)?;
        let catalog_enabled = curve.is_none_or(crate::curve_menu::CurveMenuBinding::enabled)
            && surface.is_none_or(crate::surface_menu::SurfaceMenuBinding::enabled);
        Some(IconTarget::builtin(
            builtin,
            Self::availability(builtin).is_ok() && catalog_enabled,
        ))
    }
    /// Menu/icon/F6 gate: a built-in operation or a command an enabled add-on provides.
    pub fn is_available(&self, feature: &str) -> bool {
        Self::availability(feature).is_ok() || self.addon_code(feature).is_some()
    }
    pub fn take_addon_messages(&mut self) -> Vec<String> {
        std::mem::take(&mut self.addon_messages)
    }

    /// Run an add-on command: a builder command opens its builder, any other runs at once
    /// with the add-on's remembered values.
    pub fn run_addon_command(&mut self, code: &str) -> Result<()> {
        if self.is_running() {
            return Err(Error::Busy);
        }
        let host = self
            .addons
            .clone()
            .ok_or(Error::Unsupported("add-on host"))?;
        let info = host
            .lock()
            .map_err(|_| Error::Io("add-on host poisoned".into()))?
            .command_info(code);
        if let Some(i) = info.as_ref().filter(|i| i.command.interactive) {
            let label = i.command.label.clone();
            let panel = i.command.builder;
            self.start_addon_session(code, &label)?;
            // A command with its own menu panel (Gem Cutter …) also opens the builder; its
            // values travel with every step and Enter goes to the session.
            if panel {
                self.start_addon_builder(code)?;
                if let Some(b) = self.addon_builder.as_mut() {
                    b.interactive = true;
                }
            }
            return Ok(());
        }
        if info.as_ref().is_some_and(|i| i.command.builder) {
            return self.start_addon_builder(code);
        }
        // Any other command closes an open builder first: one command at a time.
        self.cancel_addon_builder();
        let label = info
            .map(|i| i.command.label)
            .unwrap_or_else(|| code.to_owned());
        self.run_addon_with(code, &BTreeMap::new(), &label)
            .map(|_| ())
    }

    /// The selected curves sampled in world millimetres, the active CPlane normal, and the
    /// add-on's answer for `values` (overriding its stored settings for this call only).
    pub(super) fn addon_invocation(
        &self,
        code: &str,
        values: &BTreeMap<String, SettingValue>,
    ) -> Result<CommandOutput> {
        let host = self
            .addons
            .clone()
            .ok_or(Error::Unsupported("add-on host"))?;
        let to_mm = self.document().page_unit.factor();
        let limits = *self.coordinator.limits();
        let mut curves = Vec::new();
        for id in &self.selection {
            let Some(entity) = self.document().entities.get(id) else {
                continue;
            };
            let Geometry::Curve(curve) = &entity.geometry else {
                continue;
            };
            let samples = om9_core::surface_kernel::sample_curve(curve, CURVE_SAMPLES, &limits)?;
            let mut points = Vec::with_capacity(samples.len());
            for p in samples {
                let q = entity.placement.point(p)?;
                points.push([q[0] * to_mm, q[1] * to_mm, q[2] * to_mm]);
            }
            let closed = points.len() > 2 && {
                let (a, b) = (points[0], points[points.len() - 1]);
                let span = (a[0] - b[0]).hypot(a[1] - b[1]).hypot(a[2] - b[2]);
                span <= 1e-6
            };
            if closed {
                points.pop();
            }
            curves.push(CurveInput { points, closed });
        }
        let plane = self.workspace.views[&self.workspace.active].cplane;
        let input = CommandInput {
            command: code.to_owned(),
            curves,
            plane_normal: cross(plane.x, plane.y),
            settings: Default::default(),
        };
        host.lock()
            .map_err(|_| Error::Io("add-on host poisoned".into()))?
            .invoke_with(code, input, values)
            .map_err(Error::Io)
    }

    /// Invoke and insert the result in one transaction. Returns the number of objects created.
    pub(super) fn run_addon_with(
        &mut self,
        code: &str,
        values: &BTreeMap<String, SettingValue>,
        label: &str,
    ) -> Result<usize> {
        self.events.emit(
            crate::command_events::EventKind::Started,
            code,
            "add-on command",
            None,
        );
        let output = match self.addon_invocation(code, values) {
            Ok(output) => output,
            Err(error) => {
                self.events.emit(
                    crate::command_events::EventKind::Finished,
                    code,
                    "add-on refused",
                    Some(crate::command_events::EventOutcome::Refused),
                );
                return Err(error);
            }
        };
        let to_mm = self.document().page_unit.factor();
        self.addon_messages.extend(output.messages.iter().cloned());
        if output.meshes.is_empty() {
            self.events.emit(
                crate::command_events::EventKind::Finished,
                code,
                "reported",
                None,
            );
            return Ok(0);
        }
        let layer = self.document().current_layer;
        let mut mutations = Vec::with_capacity(output.meshes.len());
        for m in output.meshes {
            let mesh = Mesh {
                vertices: m
                    .vertices
                    .iter()
                    .map(|v| [v[0] / to_mm, v[1] / to_mm, v[2] / to_mm])
                    .collect(),
                faces: m.triangles.iter().map(|t| t.to_vec()).collect(),
                normals: Vec::new(),
                colors: Vec::new(),
                corner_uvs: Vec::new(),
            };
            let mut entity = Entity::new(&m.name, Geometry::Mesh(mesh), layer);
            if entity.attributes.layer.is_some() {
                entity.attributes.color = None;
            }
            if let Ok(extension) = Extension::new(1, m.meta_json.clone()) {
                entity
                    .extensions
                    .insert(GEM_EXTENSION_SLOT.to_owned(), extension);
            }
            mutations.push(Mutation::Insert {
                document: self.document,
                entity,
            });
        }
        let scene = self.scene.as_ref().ok_or(Error::Missing("command scene"))?;
        if scene.epoch != self.coordinator.epoch() { return Err(Error::Stale("command scene")); }
        let created = mutations.len();
        let held = self.scene_cache_usage()?;
        let undo_label = crate::icons::icon_name(code).unwrap_or(label).to_owned();
        let prepared = self
            .coordinator
            .prepare_with_held_usage(TransactionRequest::new(&undo_label, mutations), held)?;
        let receipt = self.commit_with_candidate_scene(prepared, held)?;
        if receipt.published {
            self.sync_published(om9_core::changes::ChangeCause::Edit);
            self.addon_messages.push(format!(
                "{label}: đã đặt {created} viên (một lần Undo gỡ tất cả)"
            ));
        }
        self.events.emit(
            crate::command_events::EventKind::Finished,
            code,
            &format!("created {created} objects"),
            Some(if receipt.published {
                crate::command_events::EventOutcome::Committed
            } else {
                crate::command_events::EventOutcome::Refused
            }),
        );
        Ok(created)
    }
}

/// The command an icon runs (see [`Controller::icon_target`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IconTarget {
    pub command: String,
    pub enabled: bool,
    /// Provided by an installed add-on rather than built into OM9.
    pub addon: bool,
}
impl IconTarget {
    fn builtin(feature: &str, enabled: bool) -> Self {
        Self {
            command: feature.to_owned(),
            enabled,
            addon: false,
        }
    }
}
