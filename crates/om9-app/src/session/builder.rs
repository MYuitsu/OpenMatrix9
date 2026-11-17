// SPDX-License-Identifier: LGPL-2.1-or-later
//! Builder panel and recompute journal for the M04 ring chain (`OM9-L2-03-11`,
//! `OM9-L2-04-02`).
//!
//! The panel owns no geometry of its own: [`BuilderSession`] keeps the typed fields and the
//! preview, the producer registry computes it, and Apply inserts the entity, its typed
//! recipe and its published result in one transaction. A parameter edit through the panel or
//! through a viewport handle goes to the same field, so the two can never disagree.
use om9_core::{
    associative::RecomputeOutcome,
    builder::{BuilderHandle, BuilderKind, BuilderSession},
    changes::ChangeCause,
    geometry::Point3,
    history::Recipe,
    identity::{HistoryId, ObjectId},
    producers::ProducerRegistry,
    request::Value,
    result::PublishedResult,
    transaction::{Mutation, TransactionRequest},
};

use super::{Controller, Error, Result};

/// The panel's command-line grammar for one field, for the shared option formatter.
pub const BUILDER_FIELDS: &[(&str, &str)] = &[
    ("Diameter", "diameter_mm"),
    ("CenterX", "center_x_mm"),
    ("CenterY", "center_y_mm"),
    ("CenterZ", "center_z_mm"),
    ("Center X", "center_x_mm"),
    ("Center Y", "center_y_mm"),
    ("Center Z", "center_z_mm"),
    ("Width", "width_mm"),
    ("Height", "height_mm"),
    ("Offset", "offset_mm"),
    ("Target", "target_diameter_mm"),
    ("Thickness", "thickness_mm"),
    // The jewelry ring family (`OM9-L2-04-03`) fields, so the command line can address the
    // same parameters the panel paints.
    ("Wall", "thickness_mm"),
    ("Stones", "stone_count"),
    ("Stone", "stone_diameter_mm"),
    ("Raised", "raised_height_mm"),
    ("Ridge", "raised_width_mm"),
    ("Top Width", "top_width_mm"),
    ("Top Length", "top_length_mm"),
    ("Shoulder", "shoulder_mm"),
];

/// The command names that open one builder mode. Matrix spells them as `gvRingRail` and as
/// the menu label; both reach the same session. `RingBuilder` is the name the L3-009 fixture
/// uses for the command-line route, so it opens the same ring rail session.
pub fn builder_command(text: &str) -> Option<BuilderKind> {
    match text.trim().to_ascii_lowercase().as_str() {
        "ringrail" | "ring rail" | "gvringrail" | "ringbuilder" | "ring builder" | "builderringrail" | "topiconringrail" | "ringrailtop" => {
            Some(BuilderKind::RingRail)
        }
        "railprofile" | "rail profile" | "gvrailprofile" | "profileplacer" => {
            Some(BuilderKind::RailProfile)
        }
        "railsweep" | "rail sweep" | "gvrailsweep" => Some(BuilderKind::RailSweep),
        "outsiderail" | "outside rail" | "gvoutsiderail" => Some(BuilderKind::OutsideRail),
        "ringresizer" | "ring resizer" | "gvringresizer" | "resizer" => {
            Some(BuilderKind::RingResizer)
        }
        "eternityband" | "eternity band" | "eternity" => Some(BuilderKind::EternityBand),
        "signetring" | "signet ring" | "signet" => Some(BuilderKind::SignetRing),
        "raisedband" | "raised band" | "raised" => Some(BuilderKind::RaisedBand),
        "awardring" | "award ring" | "award" => Some(BuilderKind::AwardRing),
        _ => None,
    }
}

/// One field as the panel paints it: a number with its domain, a boolean, or a choice.
#[derive(Debug, Clone, PartialEq)]
pub struct BuilderFieldView {
    pub name: &'static str,
    pub label: &'static str,
    pub value: f64,
    pub min: f64,
    pub max: f64,
    /// The numeric dimension the panel must send back (length, angle, ratio or count).
    pub dimension: om9_core::request::Dimension,
    /// Quantisation step of a numeric field, so the slider moves in whole steps.
    pub step: Option<f64>,
    /// Slider position in `0.0..=1.0`, or `None` for a non-numeric field.
    pub ratio: Option<f64>,
    pub boolean: Option<bool>,
    /// Selected choice and its labels, for the profile preset combo box.
    pub choice: Option<(String, Vec<String>)>,
}

impl Controller {
    pub fn builder_choice_label(&self, name: &str, value: &str) -> String {
        if name == "standard" && self.builder.as_ref().is_some_and(|session| session.kind() == BuilderKind::RingRail) {
            om9_core::ring_size::rail_system_label(value).into()
        } else { value.into() }
    }

    /// The panel's copy of the open builder fields. The panel never holds a second model.
    pub fn builder_field_views(&self) -> Vec<BuilderFieldView> {
        let Some(session) = self.builder.as_ref() else {
            return Vec::new();
        };
        session
            .fields()
            .iter()
            .map(|field| {
                let boolean = match field.value {
                    om9_core::request::Value::Boolean(value) => Some(value),
                    _ => None,
                };
                let choice = match (&field.value, &field.domain) {
                    (
                        om9_core::request::Value::Choice(selected),
                        om9_core::request::OptionDomain::Choice(choices),
                    ) => Some((selected.clone(), choices.clone())),
                    _ => None,
                };
                let (dimension, min, max) = match &field.domain {
                    om9_core::request::OptionDomain::Number {
                        min,
                        max,
                        dimension,
                        ..
                    } => (
                        *dimension,
                        min.unwrap_or(f64::NEG_INFINITY),
                        max.unwrap_or(f64::INFINITY),
                    ),
                    _ => (om9_core::request::Dimension::Length, 0.0, 0.0),
                };
                BuilderFieldView {
                    name: field.name,
                    label: field.label,
                    value: field.numeric().unwrap_or(0.0),
                    min,
                    max,
                    dimension,
                    step: field.step(),
                    ratio: field.slider_ratio(),
                    boolean,
                    choice,
                }
            })
            .collect()
    }
    pub fn builder_summary(&self) -> Option<String> {
        self.builder
            .as_ref()
            .and_then(|session| session.preview_for_document(self.document()))
            .map(|preview| preview.summary.clone())
    }
    pub fn builder_is_open(&self) -> bool {
        self.builder.is_some()
    }
}

impl Controller {
    /// The producer table for this build: OCCT when the native feature is compiled in, the
    /// analytic reference chain otherwise.
    pub(crate) fn builder_registry(&self) -> Result<ProducerRegistry> {
        #[cfg(feature = "occt-brep")]
        {
            om9_kernel::associative::history_registry()
        }
        #[cfg(not(feature = "occt-brep"))]
        {
            let mut registry = ProducerRegistry::new();
            om9_core::builder::register_reference_producers(&mut registry)?;
            Ok(registry)
        }
    }

    /// Start one builder from the current selection. The selected curves fill the command's
    /// roles in order; a wrong count or kind is refused before a panel appears.
    pub fn begin_builder(&mut self, kind: BuilderKind) -> Result<()> {
        if self.ring_rail_history_pending() { return Err(Error::Busy); }
        if self.is_running() {
            return Err(Error::Busy);
        }
        #[cfg(feature = "occt-brep")]
        self.cancel_ring_preview_candidate();
        self.builder_saved.clear();
        let selection = self.builder_selection(kind);
        self.start_builder(kind, &selection)
    }

    /// The selection entries one mode consumes, in command order. A ring rail has no input,
    /// so a selection left over from another tab is not an argument error there.
    fn builder_selection(&self, kind: BuilderKind) -> Vec<ObjectId> {
        self.selected_ordered()
            .into_iter()
            .take(kind.input_roles().len())
            .collect()
    }

    fn start_builder(&mut self, kind: BuilderKind, selection: &[ObjectId]) -> Result<()> {
        let document = &self.coordinator.world().documents[&self.document];
        let mut session = BuilderSession::new(kind, document, selection)?;
        if kind == BuilderKind::RingRail { session.set_rail_shape(self.ring_rail_shape.clone(), self.coordinator.limits())?; }
        #[cfg(feature = "occt-brep")]
        let background = self.background_ring_preview_enabled && kind.ring_family().is_some();
        #[cfg(not(feature = "occt-brep"))]
        let background = false;
        if background {
            #[cfg(feature = "occt-brep")]
            {
                self.builder = Some(session);
                if let Err(error) = self.queue_background_ring_preview() { self.builder = None; return Err(error); }
            }
        } else {
            let registry = self.builder_registry()?;
            session.refresh(document, &registry)?;
            self.builder = Some(session);
        }
        self.events.emit(
            crate::command_events::EventKind::Started,
            kind.command(),
            "builder",
            None,
        );
        Ok(())
    }

    /// Switch the builder tab. Each mode keeps its own fields: leaving a tab stores the
    /// session, returning restores it and refreshes the preview against the current document.
    /// A switch that cannot start the target restores the previous tab, so a failed switch
    /// never leaves the panel empty.
    pub fn switch_builder(&mut self, kind: BuilderKind) -> Result<()> {
        if self.ring_rail_history_pending() { return Err(Error::Busy); }
        if self.builder.as_ref().is_some_and(|session| session.kind() != kind) { self.ring_rail_replace = None; }
        if self
            .builder
            .as_ref()
            .is_some_and(|session| session.kind() == kind)
        {
            return Ok(());
        }
        #[cfg(feature = "occt-brep")]
        self.cancel_ring_preview_candidate();
        let previous = self.builder.take();
        let previous_kind = previous.as_ref().map(BuilderSession::kind);
        if let Some(previous) = previous {
            self.builder_saved.insert(previous.kind(), previous);
        }
        let result = if let Some(mut session) = self.builder_saved.remove(&kind) {
            #[cfg(feature = "occt-brep")]
            let background = self.background_ring_preview_enabled && kind.ring_family().is_some();
            #[cfg(not(feature = "occt-brep"))]
            let background = false;
            if background {
                #[cfg(feature = "occt-brep")]
                {
                    self.builder = Some(session);
                    match self.queue_background_ring_preview() {
                        Ok(()) => Ok(()),
                        Err(error) => {
                            if let Some(session) = self.builder.take() { self.builder_saved.insert(kind, session); }
                            Err(error)
                        }
                    }
                }
                #[cfg(not(feature = "occt-brep"))]
                unreachable!()
            } else {
            let registry = self.builder_registry()?;
            let document = &self.coordinator.world().documents[&self.document];
            match session.refresh(document, &registry) {
                Ok(()) => {
                    self.builder = Some(session);
                    self.events.emit(
                        crate::command_events::EventKind::Progress,
                        kind.command(),
                        "builder tab",
                        None,
                    );
                    Ok(())
                }
                Err(error) => {
                    self.builder_saved.insert(kind, session);
                    Err(error)
                }
            }
            }
        } else {
            let selection = self.builder_selection(kind);
            self.start_builder(kind, &selection)
        };
        if result.is_err()
            && let Some(previous_kind) = previous_kind
        {
            self.builder = self.builder_saved.remove(&previous_kind);
        }
        result
    }

    /// The tab row: every mode and whether it is the active one.
    pub fn builder_tabs(&self) -> Vec<(BuilderKind, bool)> {
        let active = self.builder.as_ref().map(BuilderSession::kind);
        BuilderKind::ALL
            .into_iter()
            .map(|kind| (kind, active == Some(kind)))
            .collect()
    }

    /// The parked tabs. Each keeps its own typed parameters while another tab is active, so
    /// returning to it restores exactly the session the user left.
    pub fn builder_saved(&self) -> &std::collections::BTreeMap<BuilderKind, BuilderSession> {
        &self.builder_saved
    }

    pub fn builder(&self) -> Option<&BuilderSession> {
        self.builder.as_ref()
    }

    /// The profile presets the selector window lists for the open mode.
    pub fn builder_presets(&self) -> Vec<om9_core::producers::ProfilePreset> {
        self.builder
            .as_ref()
            .map(BuilderSession::presets)
            .unwrap_or_default()
    }
    pub fn builder_selected_preset(&self) -> Option<om9_core::producers::ProfilePreset> {
        self.builder
            .as_ref()
            .and_then(BuilderSession::selected_preset)
    }
    /// Choose a profile from the selector window; only the preview changes.
    pub fn builder_set_preset(&mut self, name: &str) -> Result<()> {
        {
            let session = self
                .builder
                .as_mut()
                .ok_or(Error::Missing("builder session"))?;
            session.set_preset(name)?;
        }
        self.builder_refresh()
    }
    /// Remove a preset from this session's library. Removing the selected shape drops the
    /// preview, so Apply cannot use a shape the library no longer lists.
    pub fn builder_remove_preset(&mut self, name: &str) -> Result<()> {
        {
            let session = self
                .builder
                .as_mut()
                .ok_or(Error::Missing("builder session"))?;
            session.remove_preset(name)?;
        }
        if self
            .builder
            .as_ref()
            .is_some_and(|session| session.preview().is_none())
        {
            return Err(Error::Missing("profile preset"));
        }
        self.builder_refresh()
    }

    pub fn builder_handles(&self) -> Vec<BuilderHandle> {
        if self.builder.is_none() {
            let handles = self.gem_pave_handles();
            if !handles.is_empty() {
                return handles;
            }
        }
        self.builder
            .as_ref()
            .and_then(|session| session.preview_for_document(self.document()))
            .map(|preview| preview.handles.clone())
            .unwrap_or_default()
    }

    /// The status line the panel and the command line share.
    pub fn builder_prompt(&self) -> Option<String> {
        let session = self.builder.as_ref()?;
        let summary = session
            .preview_for_document(self.document())
            .map(|preview| preview.summary.clone())
            .unwrap_or_else(|| "no preview".into());
        Some(format!("{}: {summary}", session.title()))
    }

    /// Apply one typed value from the panel or the command line.
    pub fn builder_set_field(&mut self, name: &str, value: Value) -> Result<()> {
        self.builder_require_editable()?;
        let changed = {
            let session = self
                .builder
                .as_mut()
                .ok_or(Error::Missing("builder session"))?;
            let before = session.revision();
            session.set_field(name, value)?;
            session.revision() != before
        };
        if changed { self.builder_refresh() } else { Ok(()) }
    }

    /// Drag a viewport handle or move a slider by `delta` millimetres. A locked input is
    /// refused before the field moves: the document would reject the edit, so the builder
    /// must not pretend it happened (`OM9-L3-03-11-005`).
    pub fn builder_adjust_field(&mut self, name: &str, delta: f64) -> Result<()> {
        if self.builder.is_none() && self.gem_pave.is_some() {
            return self.gem_pave_adjust_handle(name, delta);
        }
        self.builder_require_editable()?;
        let changed = {
            let session = self
                .builder
                .as_mut()
                .ok_or(Error::Missing("builder session"))?;
            let before = session.revision();
            session.adjust_field(name, delta)?;
            session.revision() != before
        };
        if changed { self.builder_refresh() } else { Ok(()) }
    }

    /// Set a slider position (`0.0..=1.0`); the field's own range and step decide the
    /// value, so the display, the model and the preview agree (`OM9-L3-03-11-003`).
    pub fn builder_set_slider(&mut self, name: &str, ratio: f64) -> Result<()> {
        self.builder_require_editable()?;
        let changed = {
            let session = self
                .builder
                .as_mut()
                .ok_or(Error::Missing("builder session"))?;
            if let Some(field) = session.field(name)
                && let (Some(current), Some(next)) = (field.numeric(), field.from_slider_ratio(ratio))
                && (current - next).abs() <= 1.0e-10_f64.max(current.abs() * 1.0e-12)
            {
                return Ok(());
            }
            let before = session.revision();
            session.set_slider(name, ratio)?;
            session.revision() != before
        };
        if changed { self.builder_refresh() } else { Ok(()) }
    }

    /// Apply a panel edit that was captured at `revision`. A queued event older than the
    /// latest handle drag is refused instead of overwriting it (`OM9-L3-03-11-006`).
    pub fn builder_set_field_at(&mut self, name: &str, value: Value, revision: u64) -> Result<()> {
        self.builder_require_editable()?;
        let changed = {
            let session = self
                .builder
                .as_mut()
                .ok_or(Error::Missing("builder session"))?;
            let before = session.revision();
            session.set_field_at(name, value, revision)?;
            session.revision() != before
        };
        if changed { self.builder_refresh() } else { Ok(()) }
    }

    /// The edit counter the panel captured when it painted its widgets.
    pub fn builder_revision(&self) -> Option<u64> {
        self.builder.as_ref().map(BuilderSession::revision)
    }

    /// Whether the open builder may edit anything: a locked layer or object makes the
    /// session read-only.
    pub fn builder_editable(&self) -> bool {
        if self.ring_rail_history_pending() { return false; }
        let document = &self.coordinator.world().documents[&self.document];
        self.builder
            .as_ref()
            .is_some_and(|session| session.editable(document))
    }

    fn builder_require_editable(&self) -> Result<()> {
        if self.builder.as_ref().is_some() && !self.builder_editable() {
            return Err(Error::Invalid("builder input locked"));
        }
        Ok(())
    }

    /// Recompute the preview after an edit. The document is read, never written.
    pub fn builder_refresh(&mut self) -> Result<()> {
        #[cfg(feature = "occt-brep")]
        if self.background_ring_preview_enabled && self.builder.as_ref().is_some_and(|session| session.kind().ring_family().is_some()) {
            return self.queue_background_ring_preview();
        }
        #[cfg(feature = "occt-brep")]
        self.cancel_ring_preview_candidate();
        let registry = self.builder_registry()?;
        let document = &self.coordinator.world().documents[&self.document];
        let session = self
            .builder
            .as_mut()
            .ok_or(Error::Missing("builder session"))?;
        session.refresh(document, &registry)?;
        self.events.emit(
            crate::command_events::EventKind::Progress,
            session.kind().command(),
            &session
                .preview()
                .map(|preview| preview.summary.clone())
                .unwrap_or_default(),
            None,
        );
        Ok(())
    }

    /// Back to the values the builder opened with.
    pub fn builder_reset(&mut self) -> Result<()> {
        if self.ring_rail_history_pending() { return Err(Error::Busy); }
        let selection: Vec<ObjectId> = self
            .builder
            .as_ref()
            .ok_or(Error::Missing("builder session"))?
            .inputs()
            .iter()
            .map(|(_, object)| *object)
            .collect();
        {
            let document = &self.coordinator.world().documents[&self.document];
            self.builder
                .as_mut()
                .ok_or(Error::Missing("builder session"))?
                .reset(document, &selection)?;
        }
        if let Some(builder) = self.builder.as_ref().filter(|builder| builder.kind() == BuilderKind::RingRail) {
            self.ring_rail_shape = builder.rail_shape().cloned();
        }
        self.builder_refresh()
    }

    /// Escape: the preview is dropped and the document was never touched.
    pub fn builder_cancel(&mut self) -> bool {
        if self.ring_rail_history_pending() { self.cancel_join_history_worker(); return true; }
        if self.builder.as_ref().is_some_and(|session| session.kind() == BuilderKind::RingRail) { self.ring_rail_replace = None; }
        #[cfg(feature = "occt-brep")]
        self.cancel_ring_preview_candidate();
        let Some(session) = self.builder.take() else {
            return false;
        };
        self.builder_saved.clear();
        self.applied_style = None;
        self.events.emit(
            crate::command_events::EventKind::Finished,
            session.kind().command(),
            "builder canceled",
            None,
        );
        true
    }

    /// Commit the previewed feature in one transaction: insert, typed recipe and published
    /// result together, so one Undo removes the whole feature.
    pub fn builder_apply(&mut self) -> Result<()> {
        if self.ring_rail_history_pending() { return Err(Error::Busy); }
        if self.builder.as_ref().is_some_and(|session| session.kind() == BuilderKind::RingRail) {
            if self.request_ring_rail_confirmation()? { return Ok(()); }
            self.prepare_ring_rail_replacement()?;
        }
        self.builder_require_editable()?;
        #[cfg(feature = "occt-brep")]
        if self.background_ring_preview_enabled && self.background_ring_preview_pending() {
            self.ring_preview_apply_queued = true;
            return Ok(());
        }
        #[cfg(feature = "occt-brep")]
        self.cancel_ring_preview_candidate();
        #[cfg(feature = "occt-brep")]
        if self.builder.as_ref().and_then(BuilderSession::preview).is_some_and(|preview|
            matches!(&preview.geometry, om9_core::geometry::Geometry::Brep(brep) if brep.has_allocation_owner())) {
            return self.builder_apply_leased();
        }
        let session = self
            .builder
            .as_ref()
            .ok_or(Error::Missing("builder session"))?;
        let kind = session.kind();
        let document = &self.coordinator.world().documents[&self.document];
        let mut plan = session.plan(document)?;
        // Reopening a recorded feature adopts its identity, so Apply updates that object:
        // the published result must name the revision the Geometry mutation will produce.
        let editing = self.document().entities.contains_key(&plan.entity.id);
        let replacing = kind == BuilderKind::RingRail && self.ring_rail_replacing();
        let reset_placement = editing && replacing && self.document().entities[&plan.entity.id].placement != om9_core::geometry::Transform::identity();
        if editing {
            let original = &self.document().entities[&plan.entity.id];
            let revision = original.geometry_revision.checked_add(u64::from(original.geometry != plan.entity.geometry) + u64::from(reset_placement))
                .ok_or(Error::Overflow("builder geometry revision"))?;
            plan.entity.geometry_revision = revision;
            plan.published.geometry_revision = revision;
        }
        // The Matrix History switches are stamped on the new recipe: recording off means
        // the geometry is committed without a history record; update off means a later
        // source change does not rebuild it until the user re-enables update.
        plan.recipe.policy.record = self.history_record;
        plan.recipe.policy.update = self.history_update;
        let record = self.history_record;
        if editing && !reset_placement && self.document().entities[&plan.entity.id].geometry == plan.entity.geometry
            && self.document().history.recipes.get(&plan.recipe.id) == Some(&plan.recipe)
            && self.document().results.get(&plan.entity.id) == Some(&plan.published) {
                self.builder = None;
                self.ring_rail_replace = None;
                self.builder_saved.clear();
                self.applied_style = None;
                if !self.addon_session.as_ref().is_some_and(|session| matches!(session.code.as_str(), "BuilderRingRail" | "TopIconRingRail")) {
                    self.events.emit(crate::command_events::EventKind::Finished, kind.command(), "unchanged", None);
                }
                return Ok(());
            }
        let old_scene = self.scene.as_ref().ok_or(Error::Missing("command scene"))?;
        if old_scene.epoch != self.coordinator.epoch() {
            return Err(Error::Stale("command scene"));
        }
        let mut held = self.scene_cache_usage()?;
        let preview_candidate = match &plan.entity.geometry {
            om9_core::geometry::Geometry::Brep(source) => {
                use om9_core::brep::BrepSourceAccess;
                session.preview().and_then(|preview| preview.brep_candidates.iter()
                    .find(|candidate| candidate.to_canonical().identity() == source.identity()))
            }
            _ => None,
        };
        let mut scene_points = old_scene.cache_usage()?.points;
        held.add(old_scene.update_held_usage_counted_with_candidate(Some(&plan.entity), preview_candidate, &mut scene_points)?,
            self.coordinator.limits())?;
        let expected_revision = self
            .coordinator
            .world()
            .documents
            .get(&self.document)
            .ok_or(Error::Missing("document"))?
            .revision;
        let mut mutations = if editing && self.document().entities[&plan.entity.id].geometry != plan.entity.geometry {
            vec![Mutation::Geometry {
                source: self.document().stamp(plan.entity.id)?,
                geometry: plan.entity.geometry.clone(),
            }]
        } else if editing { Vec::new() } else {
            vec![Mutation::Insert {
                document: self.document,
                entity: plan.entity.clone(),
            }]
        };
        if reset_placement { mutations.push(Mutation::Placement { source: self.document().stamp(plan.entity.id)?, placement: om9_core::geometry::Transform::identity() }); }
        let existing_recipe = self.document().history.recipes.contains_key(&plan.recipe.id);
        if record || (editing && existing_recipe) {
            if existing_recipe {
                mutations.push(Mutation::HistoryUpdate {
                    document: self.document,
                    expected_revision,
                    recipes: vec![Box::new(plan.recipe.clone())],
                    published: vec![(plan.entity.id, plan.published.clone())],
                });
            } else {
                mutations.push(Mutation::History {
                    document: self.document,
                    expected_revision,
                    recipe: Box::new(plan.recipe.clone()),
                    published: vec![(plan.entity.id, plan.published.clone())],
                });
            }
        }
        if kind == BuilderKind::RingRail { mutations.extend(self.ring_rail_current_mutations(plan.entity.id)?); }
        if kind == BuilderKind::RingRail && editing && self.ring_rail_has_dependents(plan.entity.id)
            && mutations.iter().any(|mutation| matches!(mutation, Mutation::Geometry { .. } | Mutation::Placement { .. })) {
            self.request_ring_rail_history_worker(TransactionRequest::new(kind.title(), mutations))?;
            self.events.emit(crate::command_events::EventKind::Progress, kind.command(), "updating dependent features", None);
            return Ok(());
        }
        let mut prepared = self
            .coordinator
            .prepare_with_held_usage(TransactionRequest::new(kind.title(), mutations), held)?;
        let mut candidates = self.builder.as_mut().ok_or(Error::Missing("builder session"))?.take_preview_brep_candidates()?;
        let stage = self.stage_builder_preview_representations(&mut prepared, &mut candidates, held);
        self.builder.as_mut().ok_or(Error::Missing("builder session"))?.restore_preview_brep_candidates(candidates)?;
        if let Err(error) = stage {
            drop(prepared);
            self.reset_builder_preview_store_allowance()?;
            return Err(error);
        }
        let before = match self.coordinator.capture_retained_document(self.document) {
            Ok(before) => before,
            Err(error) => { drop(prepared); self.reset_builder_preview_store_allowance()?; return Err(error); }
        };
        let receipt = match self.commit_with_candidate_scene(prepared, held) {
            Ok(receipt) => receipt,
            Err(error) => { self.reset_builder_preview_store_allowance()?; return Err(error); }
        };
        self.builder = None;
        self.ring_rail_replace = None;
        self.builder_saved.clear();
        self.applied_style = None;
        if receipt.published || !record {
            self.record_history_change(&before, ChangeCause::Edit);
            if record || editing {
                self.sync_published(ChangeCause::Edit);
            }
            if !(kind == BuilderKind::RingRail && self.addon_session.as_ref().is_some_and(|session|
                matches!(session.code.as_str(), "BuilderRingRail" | "TopIconRingRail"))) {
                self.events.emit(crate::command_events::EventKind::Finished, kind.command(), if replacing { "Ring Rail replaced" } else { "applied" }, None);
            }
        }
        Ok(())
    }

    pub(super) fn stage_builder_preview_representations(&self, prepared: &mut om9_core::transaction::PreparedTransaction,
        candidates: &mut [om9_core::brep::Brep], held: om9_core::budget::Usage) -> Result<()> {
        Self::stage_builder_representations(&self.coordinator, prepared, candidates, held)
    }

    pub(super) fn stage_builder_representations(coordinator: &om9_core::transaction::TransactionCoordinator,
        prepared: &mut om9_core::transaction::PreparedTransaction,
        candidates: &mut [om9_core::brep::Brep], held: om9_core::budget::Usage) -> Result<()> {
        use om9_core::{brep::{BrepRepresentation, BrepRepresentationProfile, BrepSourceAccess, BrepSourceIdentity},
            budget::{Usage, memory::MemoryClass}};
        if candidates.is_empty() { return Ok(()); }
        coordinator.begin_pending_brep_representations(prepared, candidates.len(), held)?;
        for candidate in candidates {
            let source = candidate.to_canonical();
            if !prepared.candidate_world().documents.values().any(|document| document.entities.values()
                .any(|entity| matches!(&entity.geometry, om9_core::geometry::Geometry::Brep(brep) if brep.identity() == source.identity()))) {
                return Err(Error::Stale("builder BRep candidate"));
            }
            let display = candidate.display_allocation_bytes()?;
            let owner_bytes = (std::mem::size_of::<std::sync::Mutex<om9_core::transaction::ExternalMemoryReservation>>()
                + 2 * std::mem::size_of::<usize>()) as u64;
            let store_bytes = Usage::btree::<(BrepSourceIdentity, BrepRepresentationProfile), BrepRepresentation>(1)?.bytes
                .checked_add(Usage::btree::<BrepSourceIdentity, BrepRepresentationProfile>(1)?.bytes)
                .and_then(|n| n.checked_add(BrepRepresentation::source_receipt_bytes()))
                .ok_or(Error::Overflow("builder BRep cache metadata"))?;
            let total = display.checked_add(owner_bytes).and_then(|n| n.checked_add(store_bytes))
                .ok_or(Error::Overflow("builder BRep display"))?.max(1);
            let owner = match candidate.display_owner_reservation() {
                Some(owner) => owner,
                None => {
                    let physical = om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
                    let lease = coordinator.reserve_external_memory_with_allocated(MemoryClass::Kernel,
                        total, display, physical, held)?;
                    let owner = std::sync::Arc::new(std::sync::Mutex::new(lease));
                    owner.lock().unwrap_or_else(|e| e.into_inner()).set_allocated_bytes(display + owner_bytes)?;
                    candidate.bind_display_allocation_owner(owner.clone(), &coordinator.external_memory_account())?;
                    owner
                }
            };
            {
                let mut lease = owner.lock().unwrap_or_else(|e| e.into_inner());
                if !lease.belongs_to(&coordinator.external_memory_account()) || lease.bytes() < total {
                    return Err(Error::Invalid("builder BRep display owner"));
                }
                lease.set_allocated_bytes(display + owner_bytes)?;
            }
            let (source, representation) = candidate.clone().into_canonical_and_representation();
            prepared.insert_pending_brep_representation(&source, representation, coordinator.limits(), coordinator, held)?;
            owner.lock().unwrap_or_else(|e| e.into_inner()).set_allocated_bytes(total)?;
        }
        Ok(())
    }

    fn reset_builder_preview_store_allowance(&self) -> Result<()> {
        let Some(preview) = self.builder.as_ref().and_then(BuilderSession::preview) else { return Ok(()); };
        let overhead = (std::mem::size_of::<std::sync::Mutex<om9_core::transaction::ExternalMemoryReservation>>()
            + 2 * std::mem::size_of::<usize>()) as u64;
        for candidate in &preview.brep_candidates {
            if let Some(owner) = candidate.display_owner_reservation() {
                let bytes = candidate.display_allocation_bytes()?.checked_add(overhead)
                    .ok_or(Error::Overflow("builder BRep display owner"))?;
                owner.lock().unwrap_or_else(|e| e.into_inner()).set_allocated_bytes(bytes)?;
            }
        }
        Ok(())
    }

    #[cfg(feature = "occt-brep")]
    fn builder_apply_leased(&mut self) -> Result<()> {
        let session = self.builder.as_ref().ok_or(Error::Missing("builder session"))?;
        let preview = session.preview().ok_or(Error::Missing("builder preview"))?;
        let kind = session.kind();
        let editing = self.document().entities.contains_key(&session.output());
        let old_scene = self.scene.as_ref().ok_or(Error::Missing("command scene"))?;
        if old_scene.epoch != self.coordinator.epoch() { return Err(Error::Stale("command scene")); }
        let estimate = om9_core::model::Entity::new("builder preview", preview.geometry.clone(), self.document().current_layer);
        let mut held = self.scene_cache_usage()?;
        held.add(old_scene.update_held_usage(Some(&estimate))?, self.coordinator.limits())?;
        drop(estimate);
        let prepared = self.coordinator.prepare_leased_builder(self.document, session,
            self.history_record, self.history_update, held)?;
        let before = self.coordinator.capture_retained_document(self.document)?;
        let receipt = self.commit_with_candidate_scene(prepared, held)?;
        self.builder = None;
        self.builder_saved.clear();
        self.applied_style = None;
        if receipt.published || !self.history_record {
            self.record_history_change(&before, ChangeCause::Edit);
            if self.history_record || editing { self.sync_published(ChangeCause::Edit); }
            self.events.emit(crate::command_events::EventKind::Finished, kind.command(), "applied", None);
        }
        Ok(())
    }

    /// Command-line / Enter handling for an open builder. `None` means no builder is open.
    pub fn builder_text(&mut self, text: &str) -> Option<Result<()>> {
        self.builder.as_ref()?;
        let trimmed = text.trim();
        let lower = trimmed.to_ascii_lowercase();
        if trimmed.is_empty() || lower == "apply" || lower == "enter" {
            return Some(self.builder_apply());
        }
        if lower == "cancel" || lower == "esc" {
            self.builder_cancel();
            return Some(Ok(()));
        }
        if lower == "reset" {
            return Some(self.builder_reset());
        }
        if let Some(rest) = lower.strip_prefix("solid=") {
            let value = match rest {
                "yes" | "true" | "on" => true,
                "no" | "false" | "off" => false,
                _ => return Some(Err(Error::Invalid("builder solid"))),
            };
            return Some(self.builder_set_field("solid", Value::Boolean(value)));
        }
        if let Some(rest) = lower.strip_prefix("preset=") {
            return Some(self.builder_set_preset(rest.trim()));
        }
        if let Some(rest) = lower.strip_prefix("standard=").or_else(|| lower.strip_prefix("sizesystem=")) {
            let id = match rest.trim() {
                "us" => "US".to_string(),
                "vnm" => "VNM".to_string(),
                "" | "explicit" | "diameter" => "explicit".to_string(),
                "circumference" => "iso".to_string(),
                other => other.to_string(),
            };
            return Some(self.builder_set_field("standard", Value::Choice(id)));
        }
        if let Some(rest) = lower.strip_prefix("size=").or_else(|| lower.strip_prefix("ringsize=")) {
            let size: f64 = match rest.trim().parse() {
                Ok(size) => size,
                Err(_) => return Some(Err(Error::Invalid("US ring size"))),
            };
            return Some(Value::ratio(size).and_then(|value| self.builder_set_field("us_size", value)));
        }
        if let Some(rest) = lower.strip_prefix("roll=") {
            let angle = match rest.trim().parse::<f64>() {
                Ok(angle) => angle,
                Err(_) => return Some(Err(Error::Invalid("profile roll"))),
            };
            let value = match om9_core::units::AngleDeg::try_new(angle) {
                Ok(value) => value,
                Err(error) => return Some(Err(error.into())),
            };
            return Some(self.builder_set_field("roll_deg", Value::angle(value)));
        }
        if let Some(rest) = lower.strip_prefix("position=") {
            let ratio = match rest.trim().parse::<f64>() {
                Ok(ratio) => ratio,
                Err(_) => return Some(Err(Error::Invalid("profile position"))),
            };
            let value = match Value::ratio(ratio) {
                Ok(value) => value,
                Err(error) => return Some(Err(error)),
            };
            return Some(self.builder_set_field("position", value));
        }
        if let Some(rest) = lower.strip_prefix("closedrail=") {
            let value = match rest {
                "yes" | "true" | "on" => true,
                "no" | "false" | "off" => false,
                _ => return Some(Err(Error::Invalid("builder closed rail"))),
            };
            return Some(self.builder_set_field("closed_rail", Value::Boolean(value)));
        }
        // A style name keeps its original case; the lowercased prefix and the original text
        // have the same byte length for these ASCII commands.
        if let Some(rest) = lower.strip_prefix("stylesave=") {
            let name = &trimmed[trimmed.len() - rest.len()..];
            return Some(self.builder_save_style(name.trim()).map(|_| ()));
        }
        if let Some(rest) = lower.strip_prefix("style=") {
            let name = &trimmed[trimmed.len() - rest.len()..];
            return Some(self.builder_apply_style(name.trim()).map(|_| ()));
        }
        if lower == "stylematch" || lower == "match attrs" {
            return Some(self.builder_match_attributes().map(|_| ()));
        }
        for (token, field) in BUILDER_FIELDS {
            let prefix = format!("{}=", token.to_ascii_lowercase());
            if let Some(rest) = lower.strip_prefix(&prefix) {
                // The lowercased prefix and the original token have the same byte length
                // for these ASCII field names.
                let value = &trimmed[trimmed.len() - rest.len()..];
                return Some(match self.builder_length(value) {
                    Ok(length) => self.builder_set_field(field, length),
                    Err(error) => Err(error),
                });
            }
        }
        // A bare number edits the first numeric field, like a prompt default.
        if let Ok(length) = self.builder_length(trimmed)
            && let Some(field) = self
                .builder
                .as_ref()
                .and_then(|session| {
                    session
                        .fields()
                        .iter()
                        .find(|field| field.numeric().is_some())
                })
                .map(|field| field.name)
        {
            return Some(self.builder_set_field(field, length));
        }
        Some(Err(Error::Invalid("builder field")))
    }

    fn builder_length(&self, text: &str) -> Result<Value> {
        Ok(Value::length(om9_core::units::LengthMm::parse(
            text,
            self.document().page_unit,
        )?))
    }

    /// The clickable option tokens of the open builder.
    pub fn builder_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        let Some(session) = self.builder.as_ref() else {
            return Vec::new();
        };
        let mut tokens: Vec<(&'static str, &'static str)> = session
            .fields()
            .iter()
            .map(|field| (field.label, "field"))
            .collect();
        if session.field("solid").is_some() {
            tokens.push(("Solid", "toggle"));
        }
        if session.field("preset").is_some() {
            tokens.push(("Profile", "profile"));
        }
        tokens.push(("Apply", "apply"));
        tokens.push(("Reset", "reset"));
        tokens.push(("Cancel", "cancel"));
        tokens
    }

    /// Edit one recorded parameter through the transaction journal (the panel's "edit
    /// existing feature" path). The recipe becomes Dirty; the next recompute rebuilds it.
    pub fn set_history_parameter(&mut self, id: HistoryId, key: &str, value: Value) -> Result<()> {
        let base = self.document();
        let revision = base.revision;
        let mut recipe = base
            .history
            .recipes
            .get(&id)
            .cloned()
            .ok_or(Error::Missing("history recipe"))?;
        recipe.parameters.insert(key.into(), value);
        recipe.policy.update = self.history_update;
        recipe.state = om9_core::history::EvaluationState::Dirty("parameter edit".into());
        let held = self.scene_cache_usage()?;
        let prepared = self.coordinator.prepare_with_held_usage(TransactionRequest::new(
            "History Parameter",
            vec![Mutation::HistoryUpdate {
                document: self.document,
                expected_revision: revision,
                recipes: vec![Box::new(recipe)],
                published: vec![],
            }],
        ), held)?;
        self.commit_with_candidate_scene(prepared, held)?;
        self.sync_published(ChangeCause::Edit);
        Ok(())
    }

    /// Recompute one recorded feature through the registry and journal the result as one
    /// transaction. A rebuild that is already running refuses a second start (`-006`), and
    /// the document lifecycle sees the flag while it owns the world.
    pub fn recompute_history(&mut self, id: HistoryId) -> Result<RecomputeOutcome> {
        if self.history_rebuilding {
            return Err(Error::Busy);
        }
        self.history_rebuilding = true;
        self.events.emit(
            crate::command_events::EventKind::Started,
            "History",
            &format!("updating {id}"),
            None,
        );
        let mut rebuilt = 0usize;
        let mut result = self.recompute_history_inner(id, false);
        if result.is_ok() {
            rebuilt += 1;
        }
        // One source edit rebuilds its consumers too, in execution order, so a chain
        // A -> B -> C is clean again after a single Apply (`-002`).
        if self.history_update && result.is_ok() {
            let limit = self
                .document()
                .history
                .recipes
                .len()
                .saturating_mul(2)
                .max(1);
            for _ in 0..limit {
                let next = {
                    let graph = &self.document().history;
                    match om9_core::history_ops::plan_recompute(graph) {
                        Ok(plan) => plan.into_iter().find(|candidate| *candidate != id),
                        Err(error) => {
                            result = Err(error);
                            break;
                        }
                    }
                };
                let Some(next) = next else { break };
                match self.recompute_history_inner(next, true) {
                    Ok(RecomputeOutcome::Failed { .. }) => {
                        // The failing node is the answer; it keeps its previous result.
                        result = self
                            .document()
                            .history
                            .recipes
                            .get(&next)
                            .map(|recipe| RecomputeOutcome::Failed {
                                recipe: next,
                                reason: match &recipe.state {
                                    om9_core::history::EvaluationState::Failed(reason) => {
                                        reason.clone()
                                    }
                                    _ => "rebuild failed".into(),
                                },
                                previous_retained: true,
                            })
                            .ok_or(Error::Missing("history recipe"));
                        break;
                    }
                    Ok(RecomputeOutcome::Cached { .. }) => break,
                    Ok(_) => rebuilt += 1,
                    Err(error) => {
                        result = Err(error);
                        break;
                    }
                }
            }
        }
        self.history_rebuilding = false;
        self.history_notice = Some(match &result {
            Ok(RecomputeOutcome::Failed { recipe, reason, .. }) => {
                format!("History update failed at {recipe}: {reason}")
            }
            Ok(_) => format!("History update complete: {rebuilt} feature(s)"),
            Err(error) => format!("History update failed: {error}"),
        });
        match &result {
            Ok(RecomputeOutcome::Failed { recipe, reason, .. }) => {
                self.events.emit(
                    crate::command_events::EventKind::Warn,
                    "History",
                    &format!("failed at {recipe}: {reason}"),
                    Some(crate::command_events::EventOutcome::Refused),
                );
            }
            Ok(_) => {
                self.events.emit(
                    crate::command_events::EventKind::Finished,
                    "History",
                    &format!("complete: {rebuilt} feature(s)"),
                    Some(crate::command_events::EventOutcome::Committed),
                );
            }
            Err(error) => {
                self.events.emit(
                    crate::command_events::EventKind::Warn,
                    "History",
                    &format!("failed: {error}"),
                    Some(crate::command_events::EventOutcome::Refused),
                );
            }
        }
        result
    }

    fn recompute_history_inner(
        &mut self,
        id: HistoryId,
        refresh_inputs: bool,
    ) -> Result<RecomputeOutcome> {
        let base = self.document().clone();
        let mut draft = base.clone();
        if refresh_inputs {
            om9_core::history_ops::refresh_inputs(&base, &mut draft.history, id)?;
        }
        let registry = self.builder_registry()?;
        let mut candidates = Vec::new();
        let mut staged_cache = self.history_cache.clone();
        let outcome = om9_core::producers::recompute_registered_with_candidates(
            &mut draft,
            &mut staged_cache,
            id,
            &registry,
            &mut candidates,
        )?;
        if matches!(
            outcome,
            RecomputeOutcome::Failed { .. } | RecomputeOutcome::Cached { .. }
        ) {
            return Ok(outcome);
        }
        let mut mutations = Vec::new();
        for (object, entity) in &draft.entities {
            let Some(before) = base.entities.get(object) else {
                continue;
            };
            if entity.geometry != before.geometry {
                mutations.push(Mutation::Geometry {
                    source: base.stamp(*object)?,
                    geometry: entity.geometry.clone(),
                });
            }
        }
        let mut recipes: Vec<Box<Recipe>> = Vec::new();
        for (recipe_id, recipe) in &draft.history.recipes {
            if base.history.recipes.get(recipe_id) != Some(recipe) {
                recipes.push(Box::new(recipe.clone()));
            }
        }
        let mut published: Vec<(ObjectId, PublishedResult)> = Vec::new();
        for (object, result) in &draft.results {
            if base.results.get(object) != Some(result) {
                published.push((*object, result.clone()));
            }
        }
        if mutations.is_empty() && recipes.is_empty() && published.is_empty() {
            return Ok(outcome);
        }
        let old_scene = self.scene.as_ref().ok_or(Error::Missing("command scene"))?;
        let mut held = self.scene_cache_usage()?;
        let mut scene_points = old_scene.cache_usage()?.points;
        for mutation in &mutations {
            if let Mutation::Geometry { source, .. } = mutation
                && let Some(entity) = draft.entities.get(&source.object)
            {
                use om9_core::brep::BrepSourceAccess;
                let candidate = match &entity.geometry {
                    om9_core::geometry::Geometry::Brep(brep) => candidates.iter()
                        .find(|candidate| candidate.to_canonical().identity() == brep.identity()),
                    _ => None,
                };
                held.add(
                    old_scene.update_held_usage_counted_with_candidate(Some(entity), candidate, &mut scene_points)?,
                    self.coordinator.limits(),
                )?;
            }
        }
        mutations.push(Mutation::HistoryUpdate {
            document: self.document,
            expected_revision: base.revision,
            recipes,
            published,
        });
        // Candidates share their canonical allocations with the private draft
        // and published-result snapshots. Keep those allocations charged while
        // transaction staging registers the new document owner.
        use om9_core::{budget::{Usage, memory::MemoryClass}, transaction::ExternalMemoryReservation};
        let physical = om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes));
        let mut guards = Vec::new();
        let _guard_storage = if candidates.is_empty() { None } else {
            let guard_bytes = Usage::array::<ExternalMemoryReservation>(candidates.len())?.bytes.max(1);
            let mut storage = self.coordinator.reserve_external_memory_with_held(
                MemoryClass::CpuCache, guard_bytes, physical, held)?;
            guards.try_reserve_exact(candidates.len()).map_err(|_| Error::Budget("history BRep guards"))?;
            let actual = Usage::array::<ExternalMemoryReservation>(guards.capacity())?.bytes.max(1);
            if actual > storage.bytes() {
                self.coordinator.grow_external_memory_with_held(&mut storage, actual - guard_bytes, physical, held)?;
            }
            storage.set_allocated_bytes(actual)?;
            Some(storage)
        };
        for candidate in &candidates {
            let bytes = candidate.storage_usage(self.coordinator.limits())?.bytes;
            let mut guard = self.coordinator.reserve_external_memory_with_held(MemoryClass::Kernel, bytes, physical, held)?;
            guard.set_allocated_bytes(bytes)?;
            guards.push(guard);
        }
        let mut prepared = self.coordinator.prepare_with_held_usage(
            TransactionRequest::new("Recompute History", mutations),
            held,
        )?;
        if !candidates.is_empty() {
            use om9_core::brep::{BrepRepresentation, BrepRepresentationProfile, BrepSourceAccess, BrepSourceIdentity};
            self.coordinator.begin_pending_brep_representations(&mut prepared, candidates.len(), held)?;
            for (mut candidate, mut source_guard) in candidates.into_iter().zip(guards) {
                let source = candidate.to_canonical();
                let belongs = prepared.candidate_world().documents.values().any(|document|
                    document.entities.values().chain(document.definitions.values().flat_map(|d| d.members.iter()))
                        .any(|entity| matches!(&entity.geometry, om9_core::geometry::Geometry::Brep(brep) if brep.identity() == source.identity())));
                if !belongs { return Err(Error::Stale("history BRep candidate")); }
                let display = candidate.display_allocation_bytes()?;
                let owner = (std::mem::size_of::<std::sync::Mutex<om9_core::transaction::ExternalMemoryReservation>>()
                    + 2 * std::mem::size_of::<usize>()) as u64;
                let store = Usage::btree::<(BrepSourceIdentity, BrepRepresentationProfile), BrepRepresentation>(1)?.bytes
                    .checked_add(Usage::btree::<BrepSourceIdentity, BrepRepresentationProfile>(1)?.bytes)
                    .and_then(|n| n.checked_add(BrepRepresentation::source_receipt_bytes()))
                    .ok_or(Error::Overflow("history BRep cache metadata"))?;
                let bytes = display.checked_add(owner).and_then(|n| n.checked_add(store))
                    .ok_or(Error::Overflow("history BRep display"))?.max(1);
                let overhead = owner.checked_add(store).ok_or(Error::Overflow("history BRep display"))?;
                self.coordinator.grow_external_memory_with_held(&mut source_guard, overhead, physical, held)?;
                let mut lease = source_guard.split_off(bytes)?;
                lease.set_allocated_bytes(display)?;
                let lease = std::sync::Arc::new(std::sync::Mutex::new(lease));
                lease.lock().unwrap_or_else(|e| e.into_inner()).set_allocated_bytes(display + owner)?;
                candidate.bind_display_allocation_owner(lease.clone(), &self.coordinator.external_memory_account())?;
                let (source, representation) = candidate.into_canonical_and_representation();
                prepared.insert_pending_brep_representation(&source, representation, self.coordinator.limits(), &self.coordinator, held)?;
                lease.lock().unwrap_or_else(|e| e.into_inner()).set_allocated_bytes(bytes)?;
            }
        }
        let before = self.coordinator.capture_retained_document(self.document)?;
        let receipt = self.commit_with_candidate_scene(prepared, held)?;
        if receipt.published {
            self.history_cache = staged_cache;
            self.record_history_change(&before, ChangeCause::Edit);
            self.sync_published(ChangeCause::Edit);
        }
        Ok(outcome)
    }

    /// World positions of the builder handles, for the viewport pass.
    pub fn builder_handle_positions(&self) -> Vec<(Point3, Point3, &'static str)> {
        self.builder_handles()
            .into_iter()
            .map(|handle| (handle.position, handle.axis, handle.field))
            .collect()
    }

    /// One row of the builder style browser (`OM9-L3-03-12-002`).
    ///
    /// The core library lends the rows; the adapter copies what the panel needs so the UI can
    /// borrow `self` mutably while painting.
    pub fn builder_styles(&self) -> Vec<BuilderStyleRow> {
        let Some(session) = self.builder.as_ref() else {
            return Vec::new();
        };
        let applied = self.applied_style.as_deref().map(str::to_ascii_lowercase);
        self.style_library
            .styles_for(session.kind())
            .into_iter()
            .map(|row| BuilderStyleRow {
                name: row.style.name.clone(),
                enabled: row.enabled,
                icon: row.thumbnail.icon,
                fallback: row.thumbnail.fallback,
                builtin: row.builtin,
                applied: applied.as_deref() == Some(row.style.name.to_ascii_lowercase().as_str()),
            })
            .collect()
    }

    /// Which style the open session last applied, if any.
    pub fn builder_applied_style(&self) -> Option<&str> {
        self.applied_style.as_deref()
    }

    /// The user-style file, when `OM9_STYLE_LIBRARY` names one.
    pub fn builder_style_path(&self) -> Option<&std::path::Path> {
        self.style_path.as_deref()
    }

    /// Point the user-style file at `path` (or back to session-only storage).
    ///
    /// The app calls this when the styles location is configured; tests use it instead of a
    /// process-wide environment variable, so two controllers can persist to different files.
    pub fn builder_set_style_path(&mut self, path: Option<std::path::PathBuf>) {
        self.style_path = path;
    }

    /// Load the user styles from the configured file. A refused file reports itself instead of
    /// silently replacing the user list.
    pub fn builder_load_styles(&mut self) -> Result<usize> {
        let path = self
            .style_path
            .clone()
            .ok_or(Error::Missing("style library path"))?;
        match self.style_library.load_from(&path) {
            Ok(count) => {
                self.style_error = None;
                Ok(count)
            }
            Err(error) => {
                self.style_error = Some(format!("{error}"));
                Err(error)
            }
        }
    }

    /// The last style-library problem, shown once by the panel.
    pub fn builder_style_error(&self) -> Option<&str> {
        self.style_error.as_deref()
    }

    /// Load a style into the open session as a preview (`OM9-L3-03-12-003`).
    ///
    /// Nothing is committed: the parameters land in the session, the preview is recomputed,
    /// and the document is untouched until the builder's own Apply.
    pub fn builder_apply_style(&mut self, name: &str) -> Result<String> {
        let style = self
            .style_library
            .find(name)
            .cloned()
            .ok_or(Error::Missing("builder style"))?;
        self.builder_require_editable()?;
        let session = self
            .builder
            .as_mut()
            .ok_or(Error::Missing("builder session"))?;
        let report = style.apply(session)?;
        self.applied_style = Some(report.style.clone());
        self.builder_refresh()?;
        Ok(report.summary())
    }

    /// Snapshot the open session as a user style (`OM9-L3-03-12-004`).
    ///
    /// Defaults in the library are never rewritten: the new style goes to the user list, and
    /// the file is written only when `OM9_STYLE_LIBRARY` names one.
    pub fn builder_save_style(&mut self, name: &str) -> Result<String> {
        let session = self
            .builder
            .as_ref()
            .ok_or(Error::Missing("builder session"))?;
        let style = om9_core::builder_style::BuilderStyle::snapshot(session, name.trim());
        self.style_library.save_user(style)?;
        if let Some(path) = self.style_path.clone() {
            self.style_library.save_to(&path)?;
        }
        self.applied_style = Some(name.trim().to_string());
        Ok(format!("Saved style {}", name.trim()))
    }

    /// Copy the declared attributes of the selected entity into the open session
    /// (`OM9-L3-03-12-006`).
    ///
    /// Only the attributes the style vocabulary declares are copied, and no entity is
    /// rewritten: the source keeps its identity and the builder keeps its own output id.
    pub fn builder_match_attributes(&mut self) -> Result<String> {
        let source = self
            .selection
            .iter()
            .next()
            .copied()
            .ok_or(Error::Missing("style source"))?;
        let entity = self
            .document()
            .entities
            .get(&source)
            .ok_or(Error::Missing("style source"))?
            .clone();
        self.builder_require_editable()?;
        let session = self
            .builder
            .as_mut()
            .ok_or(Error::Missing("builder session"))?;
        let mut style = om9_core::builder_style::BuilderStyle::snapshot(session, "match");
        let copied = style.match_attributes(&entity)?;
        let mut applied = Vec::new();
        let mut skipped = Vec::new();
        for name in &copied {
            let Some(field) = om9_core::builder_style::builder_field_for(name) else {
                skipped.push(*name);
                continue;
            };
            let value = style
                .parameters
                .get(*name)
                .ok_or(Error::Invalid("style attribute"))?;
            if session.field(field).is_none() {
                skipped.push(*name);
                continue;
            }
            session.set_field(field, value.to_request())?;
            applied.push(*name);
        }
        if applied.is_empty() {
            return Err(Error::Invalid("style attribute"));
        }
        self.builder_refresh()?;
        let skipped = if skipped.is_empty() {
            String::new()
        } else {
            format!(" (no field for {})", skipped.join(", "))
        };
        Ok(format!("Matched {}{skipped}", applied.join(", ")))
    }
}

/// A builder style browser row, owned by the app so the panel can paint it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuilderStyleRow {
    pub name: String,
    /// Enabled for the open mode. An incompatible style is listed but disabled.
    pub enabled: bool,
    /// The icon key, or the fallback icon when the resource is missing/corrupt.
    pub icon: &'static str,
    pub fallback: bool,
    pub builtin: bool,
    /// The style this session last applied, for the selected state in the list.
    pub applied: bool,
}
