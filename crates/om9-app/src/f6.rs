// SPDX-License-Identifier: LGPL-2.1-or-later
//! F6 context actions. The adapter classifies the current selection and resolves labels and
//! availability from the command registry; it never invents a command or a second selection
//! backend. Builder prefill is a typed request handed to the owning provider.
use om9_core::{
    Error, Result,
    geometry::Point3,
    identity::ObjectId,
    model::{Document, Entity, GEM_EXTENSION_SLOT},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Providers {
    pub m03_curve_surface: bool,
    pub m04_builder: bool,
    pub m05_gem: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextKind {
    General,
    Solid,
    Curve,
    Surface,
    Gem,
    Report,
    Builder,
    Custom,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Availability {
    Enabled,
    Disabled(String),
    RegisteredDisabled(String),
}

impl Availability {
    pub fn enabled(&self) -> bool {
        matches!(self, Self::Enabled)
    }

    pub fn reason(&self) -> Option<&str> {
        match self {
            Self::Enabled => None,
            Self::Disabled(reason) | Self::RegisteredDisabled(reason) => Some(reason),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Action {
    pub id: String,
    pub label: String,
    pub availability: Availability,
}

impl Action {
    fn enabled(id: &str, label: &str) -> Self {
        Self {
            id: id.to_owned(),
            label: label.to_owned(),
            availability: Availability::Enabled,
        }
    }

    fn disabled(id: &str, label: &str, reason: &str) -> Self {
        Self {
            id: id.to_owned(),
            label: label.to_owned(),
            availability: Availability::Disabled(reason.to_owned()),
        }
    }

    fn registered_disabled(id: &str, label: &str, reason: &str) -> Self {
        Self {
            id: id.to_owned(),
            label: label.to_owned(),
            availability: Availability::RegisteredDisabled(reason.to_owned()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Context {
    pub kind: ContextKind,
    pub title: String,
    pub actions: Vec<Action>,
}

impl Context {
    pub fn enabled_count(&self) -> usize {
        self.actions
            .iter()
            .filter(|action| action.availability.enabled())
            .count()
    }
}

/// Classify by the complete selected set. Mixed or unknown semantics deliberately fall back
/// to General; a gem extension on one member never forces a mixed set into Gem mode.
pub fn classify(document: &Document, selection: &BTreeSet<ObjectId>) -> ContextKind {
    if selection.is_empty() {
        return ContextKind::General;
    }
    let entities: Vec<&Entity> = selection
        .iter()
        .filter_map(|id| document.entities.get(id))
        .collect();
    if entities.len() != selection.len() || entities.is_empty() {
        return ContextKind::General;
    }
    if entities
        .iter()
        .all(|entity| entity.extension(GEM_EXTENSION_SLOT).is_some())
    {
        return ContextKind::Gem;
    }
    if entities
        .iter()
        .all(|entity| matches!(entity.geometry, om9_core::geometry::Geometry::Curve(_)))
    {
        return ContextKind::Curve;
    }
    if entities.iter().all(|entity| {
        matches!(
            entity.geometry,
            om9_core::geometry::Geometry::Surface(_)
                | om9_core::geometry::Geometry::Brep(_)
                | om9_core::geometry::Geometry::NativeBrep(_)
        )
    }) {
        return ContextKind::Surface;
    }
    if entities.iter().all(|entity| {
        matches!(
            entity.geometry,
            om9_core::geometry::Geometry::Mesh(_)
                | om9_core::geometry::Geometry::Brep(_)
                | om9_core::geometry::Geometry::NativeBrep(_)
        )
    }) {
        return ContextKind::Solid;
    }
    ContextKind::General
}

pub fn context(
    document: &Document,
    selection: &BTreeSet<ObjectId>,
    providers: Providers,
) -> Context {
    let kind = classify(document, selection);
    context_for_kind(kind, providers)
}

fn context_for_kind(kind: ContextKind, providers: Providers) -> Context {
    let mut actions = Vec::new();
    match kind {
        ContextKind::General => {
            actions.push(Action::enabled(crate::session::LINE, "Line"));
            actions.push(Action::enabled(crate::session::POINT, "Point"));
            actions.push(Action::enabled(
                crate::session::ZOOM_EXTENTS,
                "Zoom Extents",
            ));
            actions.push(Action::enabled(
                crate::session::ZOOM_SELECTED,
                "Zoom Selected",
            ));
            actions.push(Action::enabled(crate::session::VIEW_HOME, "Home"));
            actions.push(Action::enabled(crate::session::VIEW_CENTER, "Center"));
        }
        ContextKind::Curve => {
            actions.push(if providers.m03_curve_surface {
                Action::enabled("EditCurve", "Edit Curve")
            } else {
                Action::registered_disabled(
                    "EditCurve",
                    "Edit Curve",
                    "M03 curve edit provider is not available",
                )
            });
            actions.push(Action::disabled(
                "OffsetCurve",
                "Offset Curve",
                "M03 curve offset provider is not available",
            ));
        }
        ContextKind::Surface | ContextKind::Solid => {
            actions.push(if providers.m03_curve_surface {
                Action::enabled("EditSurface", "Edit Surface")
            } else {
                Action::registered_disabled(
                    "EditSurface",
                    "Edit Surface",
                    "M03 surface/solid edit provider is not available",
                )
            });
            actions.push(Action::disabled(
                "Boolean",
                "Boolean",
                "M03 solid boolean provider is not available",
            ));
        }
        ContextKind::Gem => {
            actions.push(if providers.m05_gem {
                Action::enabled("GemProperties", "Gem Properties")
            } else {
                Action::registered_disabled(
                    "GemProperties",
                    "Gem Properties",
                    "M05 gem provider is not available",
                )
            });
            actions.push(if providers.m05_gem {
                Action::enabled("GemReport", "Gem Report")
            } else {
                Action::registered_disabled(
                    "GemReport",
                    "Gem Report",
                    "M05 report provider is not available",
                )
            });
        }
        ContextKind::Report => {
            actions.push(Action::registered_disabled(
                "GemReport",
                "Gem Report",
                "M05 report provider is not available",
            ));
        }
        ContextKind::Builder => {
            actions.push(Action::registered_disabled(
                "BuilderPrefill",
                "Builder",
                "M04 builder provider is not available",
            ));
        }
        ContextKind::Custom => {}
    }
    Context {
        kind,
        title: match kind {
            ContextKind::General => "General",
            ContextKind::Solid => "Solid",
            ContextKind::Curve => "Curve",
            ContextKind::Surface => "Surface",
            ContextKind::Gem => "Gem",
            ContextKind::Report => "Report",
            ContextKind::Builder => "Builder",
            ContextKind::Custom => "Custom",
        }
        .to_owned(),
        actions,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomMenu {
    pub version: u32,
    /// Command IDs only. The command engine stays the single owner of implementations.
    pub command_ids: Vec<String>,
}

impl Default for CustomMenu {
    fn default() -> Self {
        Self {
            version: 1,
            command_ids: Vec::new(),
        }
    }
}

impl CustomMenu {
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|_| Error::Invalid("F6 custom menu json"))
    }

    pub fn from_json(text: &str) -> Result<Self> {
        let mut menu: Self =
            serde_json::from_str(text).map_err(|_| Error::Invalid("F6 custom menu schema"))?;
        if menu.version != 1 {
            return Err(Error::Invalid("F6 custom menu version"));
        }
        let mut seen = BTreeSet::new();
        menu.command_ids.retain(|id| seen.insert(id.clone()));
        Ok(menu)
    }

    pub fn context(&self) -> Context {
        let actions = self
            .command_ids
            .iter()
            .map(|id| {
                if is_known_command(id) {
                    Action::enabled(id, command_label(id))
                } else {
                    Action::disabled(id, id, "command registry id is unavailable")
                }
            })
            .collect();
        Context {
            kind: ContextKind::Custom,
            title: "Custom".to_owned(),
            actions,
        }
    }
}

fn is_known_command(id: &str) -> bool {
    matches!(
        id,
        crate::session::LINE
            | crate::session::POINT
            | crate::session::ZOOM_EXTENTS
            | crate::session::ZOOM_SELECTED
            | crate::session::VIEW_HOME
            | crate::session::VIEW_CENTER
    )
}

fn command_label(id: &str) -> &str {
    match id {
        crate::session::LINE => "Line",
        crate::session::POINT => "Point",
        crate::session::ZOOM_EXTENTS => "Zoom Extents",
        crate::session::ZOOM_SELECTED => "Zoom Selected",
        crate::session::VIEW_HOME => "Home",
        crate::session::VIEW_CENTER => "Center",
        _ => id,
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BuilderPrefill {
    pub version: u32,
    pub selection: Vec<ObjectId>,
    pub origin_hint: Point3,
    pub plane_revision: u64,
}

/// Emit the typed M04 prefill request. There is no fake builder backend: without the provider
/// this returns a typed refusal and the F6 action stays disabled.
pub fn builder_prefill(
    selection: &BTreeSet<ObjectId>,
    origin_hint: Point3,
    plane_revision: u64,
    providers: Providers,
) -> Result<BuilderPrefill> {
    if !providers.m04_builder {
        return Err(Error::Unsupported("M04 builder provider"));
    }
    if selection.is_empty() {
        return Err(Error::Missing("builder selection"));
    }
    if !origin_hint.iter().all(|value| value.is_finite()) {
        return Err(Error::Invalid("builder origin"));
    }
    Ok(BuilderPrefill {
        version: 1,
        selection: selection.iter().copied().collect(),
        origin_hint,
        plane_revision,
    })
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct F6WindowPreference {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub pinned: bool,
}

impl F6WindowPreference {
    pub fn default_at(anchor: eframe::egui::Pos2) -> Self {
        Self {
            x: anchor.x,
            y: anchor.y,
            width: 260.,
            height: 320.,
            pinned: false,
        }
    }

    /// Clamp a logical-point preference to the active screen. This runs whenever the popup opens,
    /// so a stale DPI/screen pixel position cannot render outside the monitor.
    pub fn clamp(self, screen: eframe::egui::Rect) -> Self {
        let width = self.width.clamp(180., screen.width().max(180.));
        let height = self.height.clamp(120., screen.height().max(120.));
        let x = self
            .x
            .clamp(screen.min.x, (screen.max.x - width).max(screen.min.x));
        let y = self
            .y
            .clamp(screen.min.y, (screen.max.y - height).max(screen.min.y));
        Self {
            x,
            y,
            width,
            height,
            ..self
        }
    }

    pub fn rect(self) -> eframe::egui::Rect {
        eframe::egui::Rect::from_min_size(
            eframe::egui::pos2(self.x, self.y),
            eframe::egui::vec2(self.width, self.height),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use om9_core::{
        geometry::{Curve, Geometry, Mesh},
        model::Extension,
    };

    fn document_with(geometries: &[Geometry]) -> (Document, BTreeSet<ObjectId>) {
        let mut document = Document::new("f6");
        let mut ids = BTreeSet::new();
        for geometry in geometries {
            let entity = Entity::new("item", geometry.clone(), None);
            ids.insert(entity.id);
            document.entities.insert(entity.id, entity);
        }
        (document, ids)
    }

    fn curve() -> Geometry {
        Geometry::Curve(Curve::Line {
            start: [0.; 3],
            end: [1., 0., 0.],
            domain: [0., 1.],
        })
    }

    fn mesh() -> Geometry {
        Geometry::Mesh(Mesh {
            vertices: vec![[0.; 3], [1., 0., 0.], [0., 1., 0.]],
            faces: vec![vec![0, 1, 2]],
            normals: Vec::new(),
            colors: Vec::new(),

            corner_uvs: Vec::new(),
        })
    }

    #[test]
    fn empty_and_curve_and_solid_selections_get_distinct_contexts() {
        let (document, empty) = document_with(&[]);
        assert_eq!(classify(&document, &empty), ContextKind::General);
        let (document, curves) = document_with(&[curve()]);
        assert_eq!(classify(&document, &curves), ContextKind::Curve);
        let (document, solids) = document_with(&[mesh()]);
        assert_eq!(classify(&document, &solids), ContextKind::Solid);
        let missing = [ObjectId::new()].into_iter().collect();
        assert_eq!(classify(&document, &missing), ContextKind::General);
    }

    #[test]
    fn mixed_or_unknown_semantics_fall_back_to_general() {
        let (mut document, mut ids) = document_with(&[curve(), mesh()]);
        assert_eq!(classify(&document, &ids), ContextKind::General);
        // A gem extension on one member must not force the mixed set into Gem mode.
        let gem = Entity::new("gem", Geometry::Point([0.; 3]), None);
        let gem_id = gem.id;
        let mut gem = gem;
        gem.extensions.insert(
            GEM_EXTENSION_SLOT.to_owned(),
            Extension::new(1, r#"{"shape":"round"}"#).unwrap(),
        );
        document.entities.insert(gem_id, gem);
        ids.insert(gem_id);
        assert_eq!(classify(&document, &ids), ContextKind::General);
    }

    #[test]
    fn gem_context_registers_but_disables_without_m05() {
        let mut gem = Entity::new("gem", Geometry::Point([0.; 3]), None);
        gem.extensions.insert(
            GEM_EXTENSION_SLOT.to_owned(),
            Extension::new(1, r#"{"shape":"round"}"#).unwrap(),
        );
        let mut document = Document::new("gem");
        let id = gem.id;
        document.entities.insert(id, gem);
        let default_context = context(&document, &[id].into_iter().collect(), Providers::default());
        assert_eq!(default_context.kind, ContextKind::Gem);
        assert_eq!(default_context.enabled_count(), 0);
        assert!(
            default_context
                .actions
                .iter()
                .all(|action| matches!(action.availability, Availability::RegisteredDisabled(_)))
        );
        let provider = context(
            &document,
            &[id].into_iter().collect(),
            Providers {
                m05_gem: true,
                ..Providers::default()
            },
        );
        assert!(provider.enabled_count() >= 1);
    }

    #[test]
    fn general_actions_use_the_registry_and_are_not_duplicated() {
        let (document, empty) = document_with(&[]);
        let default_context = context(&document, &empty, Providers::default());
        assert_eq!(default_context.actions[0].id, crate::session::LINE);
        assert!(
            default_context
                .actions
                .iter()
                .any(|action| action.id == crate::session::POINT)
        );
        let mut ids: Vec<&str> = default_context
            .actions
            .iter()
            .map(|action| action.id.as_str())
            .collect();
        let count = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), count);
    }

    #[test]
    fn a_custom_menu_keeps_order_and_disables_a_deleted_command() {
        let menu = CustomMenu {
            version: 1,
            command_ids: vec![
                crate::session::POINT.to_owned(),
                "DeletedCommand".to_owned(),
                crate::session::LINE.to_owned(),
                crate::session::POINT.to_owned(),
            ],
        };
        let loaded = CustomMenu::from_json(&menu.to_json().unwrap()).unwrap();
        assert_eq!(
            loaded.command_ids,
            vec![
                crate::session::POINT.to_owned(),
                "DeletedCommand".to_owned(),
                crate::session::LINE.to_owned(),
            ]
        );
        let context = loaded.context();
        assert!(context.actions[0].availability.enabled());
        assert!(
            context.actions[1]
                .availability
                .reason()
                .unwrap()
                .contains("unavailable")
        );
        assert!(context.actions[2].availability.enabled());
    }

    #[test]
    fn builder_prefill_is_typed_and_requires_its_provider() {
        let _ = document_with(&[]);
        let ids: BTreeSet<ObjectId> = [ObjectId::new()].into_iter().collect();
        assert!(builder_prefill(&ids, [1., 2., 3.], 1, Providers::default()).is_err());
        let prefill = builder_prefill(
            &ids,
            [1., 2., 3.],
            7,
            Providers {
                m04_builder: true,
                ..Providers::default()
            },
        )
        .unwrap();
        assert_eq!(prefill.version, 1);
        assert_eq!(prefill.origin_hint, [1., 2., 3.]);
        assert_eq!(prefill.selection, ids.iter().copied().collect::<Vec<_>>());
        assert_eq!(prefill.plane_revision, 7);
    }

    #[test]
    fn a_floating_window_is_clamped_inside_the_screen() {
        let screen = eframe::egui::Rect::from_min_size(
            eframe::egui::pos2(0., 0.),
            eframe::egui::vec2(1280., 720.),
        );
        let off = F6WindowPreference {
            x: 5000.,
            y: -100.,
            width: 900.,
            height: 900.,
            pinned: true,
        }
        .clamp(screen);
        assert!(off.x >= 0. && off.y >= 0.);
        assert!(off.x + off.width <= 1280. + 0.01);
        assert!(off.y + off.height <= 720. + 0.01);
        assert!(off.pinned);
        assert!(screen.contains(off.rect().min));
    }
}
