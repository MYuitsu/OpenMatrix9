// SPDX-License-Identifier: LGPL-2.1-or-later
//! Shared existing-rail confirmation for the direct and add-on adapters.
use super::Controller;
use om9_core::{Error, Result, geometry::Geometry, history::EvaluationState,
    identity::{DocumentId, ObjectId, SessionId, SourceStamp},
    resource::{Resource, ResourceKind}, transaction::Mutation};

const CURRENT: &str = "om9.ring-rail.current.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplaceChoice { Yes, No, Cancel }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Frontend { Direct(ObjectId, u64), Addon(bool, u64) }

#[derive(Debug, Clone)]
pub(super) struct Confirmation {
    authority: SessionId, document: DocumentId, epoch: u64, revision: u64, run: u64,
    source: SourceStamp, frontend: Frontend, choice: Option<ReplaceChoice>,
    label: String, unavailable: Option<&'static str>,
}

impl Controller {
    fn ring_rail_frontend(&self) -> Option<Frontend> {
        if let Some(session) = &self.addon_session
            && matches!(session.owner.as_str(), "om9_matrix" | "om9_tools" | "om9_top11")
            && matches!(session.code.as_str(), "BuilderRingRail" | "TopIconRingRail") {
                return self.addon_builder.as_ref().map(|builder|
                    Frontend::Addon(session.code == "TopIconRingRail", builder.revision));
            }
        self.builder.as_ref().filter(|session| session.kind() == om9_core::builder::BuilderKind::RingRail)
            .map(|session| Frontend::Direct(session.output(), session.revision()))
    }

    /// The durable current-rail pointer is part of the same transaction as its source.
    /// Old drawings without a pointer use an unambiguous recorded rail or explicit pick.
    pub fn current_ring_rail(&self) -> Result<Option<ObjectId>> {
        let mut resources = self.document().resources.values().filter(|resource| resource.name == CURRENT);
        if let Some(resource) = resources.next() {
            if resources.next().is_some() || resource.kind != ResourceKind::OpaqueBinary { return Err(Error::Invalid("current Ring Rail resource")); }
            let bytes = resource.payload()?;
            if bytes.len() > 128 { return Err(Error::Budget("current Ring Rail resource")); }
            let object: ObjectId = serde_json::from_slice(bytes).map_err(|_| Error::Invalid("current Ring Rail id"))?;
            if let Some(entity) = self.document().entities.get(&object) {
                if !matches!(entity.geometry, Geometry::Curve(_)) { return Err(Error::Invalid("current Ring Rail is not a curve")); }
                return Ok(Some(object));
            }
        }
        let mut candidates = std::collections::BTreeSet::new();
        for recipe in self.document().history.recipes.values() {
            if recipe.command.eq_ignore_ascii_case("RingRail") && recipe.state != EvaluationState::Detached {
                for object in &recipe.outputs {
                    if self.document().entities.get(object).is_some_and(|entity| matches!(entity.geometry, Geometry::Curve(_))) { candidates.insert(*object); }
                }
            }
        }
        if candidates.len() <= 1 { return Ok(candidates.first().copied()); }
        let selected: Vec<_> = self.selected_ordered().into_iter().filter(|object| candidates.contains(object)).collect();
        if selected.len() == 1 { return Ok(Some(selected[0])); }
        Err(Error::Invalid("select the current Ring Rail when several recorded rails exist"))
    }

    fn ring_replace_unavailable(&self, object: ObjectId) -> Option<&'static str> {
        let entity = self.document().entities.get(&object)?;
        if !self.document().permissions_for_entity(entity, false).is_ok_and(|permissions| permissions.editable) {
            return Some("The current Ring Rail is locked or read-only");
        }
        if self.document().history.owner(object).and_then(|id| self.document().history.recipes.get(&id)).is_some_and(|recipe| recipe.policy.lock) {
            return Some("The current Ring Rail History is locked");
        }
        None
    }

    pub(super) fn ring_rail_has_dependents(&self, object: ObjectId) -> bool {
        self.history_update && self.document().history.recipes.values().any(|recipe|
            recipe.state != EvaluationState::Detached && recipe.policy.update
                && recipe.inputs.iter().any(|input| input.source.object == object))
    }

    fn validate_ring_confirmation(&self) -> Result<()> {
        let state = self.ring_rail_replace.as_ref().ok_or(Error::Missing("Ring Rail confirmation"))?;
        if state.authority != self.coordinator.authority() || state.document != self.document().id
            || state.epoch != self.coordinator.epoch() || state.revision != self.document().revision
            || state.run != self.command_run || Some(state.frontend) != self.ring_rail_frontend()
            || self.document().resolve(&state.source).is_err() { return Err(Error::Stale("Ring Rail confirmation; request it again")); }
        Ok(())
    }

    pub fn ring_rail_confirmation_pending(&self) -> bool {
        self.ring_rail_replace.as_ref().is_some_and(|state| state.choice.is_none()) && self.ring_rail_frontend().is_some()
    }
    pub fn ring_rail_confirmation_label(&self) -> Option<&str> {
        self.ring_rail_confirmation_pending().then(|| self.ring_rail_replace.as_ref().map(|state| state.label.as_str())).flatten()
    }
    pub fn ring_rail_replace_unavailable(&self) -> Option<&'static str> {
        self.ring_rail_replace.as_ref().and_then(|state| state.unavailable)
    }
    pub(super) fn ring_rail_replacing(&self) -> bool {
        self.ring_rail_replace.as_ref().is_some_and(|state| state.choice == Some(ReplaceChoice::Yes))
    }

    /// True means the caller must retain its preview and wait for a choice.
    pub(super) fn request_ring_rail_confirmation(&mut self) -> Result<bool> {
        let Some(frontend) = self.ring_rail_frontend() else { return Ok(false); };
        if self.ring_rail_replace.is_some() {
            if let Err(error) = self.validate_ring_confirmation() { self.ring_rail_replace = None; return Err(error); }
            return Ok(self.ring_rail_confirmation_pending());
        }
        if matches!(frontend, Frontend::Direct(output, _) if self.document().entities.contains_key(&output)) { return Ok(false); }
        let Some(object) = self.current_ring_rail()? else { return Ok(false); };
        let source = self.document().stamp(object)?;
        let label = self.document().entities[&object].label.chars().take(80).collect();
        self.command_run = self.command_run.wrapping_add(1);
        self.ring_rail_replace = Some(Confirmation { authority: self.coordinator.authority(), document: self.document().id,
            epoch: self.coordinator.epoch(), revision: self.document().revision, run: self.command_run, source, frontend,
            choice: None, label, unavailable: self.ring_replace_unavailable(object) });
        Ok(true)
    }

    pub fn decide_ring_rail_replace(&mut self, choice: ReplaceChoice) -> Result<()> {
        if choice == ReplaceChoice::Cancel {
            let run = self.ring_rail_replace.as_ref().ok_or(Error::Missing("Ring Rail confirmation"))?.run;
            if run != self.command_run || self.ring_rail_frontend().is_none() { self.ring_rail_replace = None; return Err(Error::Stale("Ring Rail command")); }
            self.ring_rail_replace = None; self.cancel(); return Ok(());
        }
        if let Err(error) = self.validate_ring_confirmation() { self.ring_rail_replace = None; return Err(error); }
        let object = self.ring_rail_replace.as_ref().ok_or(Error::Missing("Ring Rail confirmation"))?.source.object;
        if choice == ReplaceChoice::Yes && let Some(reason) = self.ring_replace_unavailable(object) { return Err(Error::Unsupported(reason)); }
        self.ring_rail_replace.as_mut().expect("validated confirmation").choice = Some(choice);
        match self.ring_rail_frontend() {
            Some(Frontend::Addon(_, _)) => self.session_step(om9_addon_abi::session::Input::Enter),
            Some(Frontend::Direct(_, _)) => self.builder_apply(),
            None => Err(Error::Stale("Ring Rail command")),
        }
    }

    pub(super) fn submit_ring_rail_confirmation(&mut self, text: &str) -> Option<Result<()>> {
        if !self.ring_rail_confirmation_pending() { return None; }
        let choice = match text.trim().to_ascii_lowercase().as_str() {
            "yes" | "y" => ReplaceChoice::Yes, "no" | "n" => ReplaceChoice::No,
            "cancel" | "esc" => ReplaceChoice::Cancel,
            _ => return Some(Err(Error::Invalid("choose Yes, No or Cancel"))),
        };
        Some(self.decide_ring_rail_replace(choice))
    }

    /// Bind replacement to the existing output/recipe only after all source guards.
    pub(super) fn prepare_ring_rail_replacement(&mut self) -> Result<()> {
        if !self.ring_rail_replacing() { return Ok(()); }
        self.validate_ring_confirmation()?;
        let object = self.ring_rail_replace.as_ref().expect("confirmation").source.object;
        if let Some(reason) = self.ring_replace_unavailable(object) { return Err(Error::Unsupported(reason)); }
        let owner = self.document().history.owner(object);
        let session = self.builder.as_mut().ok_or(Error::Missing("Ring Rail builder"))?;
        if session.output() != object {
            session.adopt_history(object, owner.unwrap_or(session.recipe()));
            self.builder_refresh()?;
            let frontend = self.ring_rail_frontend().ok_or(Error::Stale("Ring Rail command"))?;
            self.ring_rail_replace.as_mut().expect("confirmation").frontend = frontend;
        }
        Ok(())
    }

    pub(super) fn ring_rail_current_mutations(&self, object: ObjectId) -> Result<Vec<Mutation>> {
        let bytes = serde_json::to_vec(&object).map_err(|_| Error::Invalid("current Ring Rail id"))?;
        let existing: Vec<_> = self.document().resources.values().filter(|resource| resource.name == CURRENT).collect();
        if existing.len() == 1 && existing[0].kind == ResourceKind::OpaqueBinary && existing[0].payload()? == bytes { return Ok(Vec::new()); }
        let resource = Resource::from_bytes(CURRENT, ResourceKind::OpaqueBinary, &bytes, self.coordinator.limits())?;
        let mut mutations: Vec<_> = existing.into_iter().map(|old| Mutation::ResourceDelete { document: self.document,
            expected_revision: self.document().revision, resource: old.id }).collect();
        mutations.push(Mutation::ResourceInsert { document: self.document, expected_revision: self.document().revision, resource });
        Ok(mutations)
    }
}
