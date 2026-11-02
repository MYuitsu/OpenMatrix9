// SPDX-License-Identifier: LGPL-2.1-or-later
//! Matrix History UI state and associative-update policy (`OM9-L2-03-13`).
//!
//! The graph itself lives in `om9_core::history`; this module owns the user-facing
//! switches (record / update), the broken-link report, clearing one object's history and
//! reopening a builder from a recorded recipe. Every mutation goes through the same
//! transaction journal as the rest of the app.
use om9_core::{
    Error, Result,
    builder::{BuilderKind, BuilderSession},
    changes::ChangeCause,
    history::{EvaluationState, Recipe},
    history_ops,
    identity::{HistoryId, ObjectId},
    request::Value,
    transaction::{Mutation, TransactionRequest},
};

use super::Controller;

/// One row of the history report the panel paints.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryEntry {
    pub recipe: HistoryId,
    pub command: String,
    pub state: EvaluationState,
    pub outputs: Vec<ObjectId>,
    /// The row has a missing/changed binding or an unresolved/dirty/failed state.
    pub problem: bool,
    /// Human-readable node/source detail and the remediation hint the panel paints.
    pub detail: String,
}

impl Controller {
    pub fn history_record_enabled(&self) -> bool {
        self.history_record
    }

    pub fn history_update_enabled(&self) -> bool {
        self.history_update
    }

    pub fn history_rebuilding(&self) -> bool {
        self.history_rebuilding
    }

    /// Turn Matrix History recording on or off for recipes created from now on.
    /// A toggle during an edit transaction is refused: the transaction was prepared with
    /// the old policy and must not be recorded under a different one.
    pub fn set_history_record(&mut self, enabled: bool) -> Result<String> {
        if self.is_running() {
            return Err(Error::Invalid("history record during edit transaction"));
        }
        self.history_record = enabled;
        let message = format!("History recording {}", if enabled { "on" } else { "off" });
        self.history_notice = Some(message.clone());
        Ok(message)
    }

    pub fn toggle_history_record(&mut self) -> Result<String> {
        self.set_history_record(!self.history_record)
    }

    /// Turn associative recompute on or off. Re-enabling validates the graph first: a
    /// broken or cyclic dependency keeps the switch off instead of silently rebuilding
    /// against missing inputs.
    pub fn set_history_update(&mut self, enabled: bool) -> Result<String> {
        if self.history_rebuilding {
            return Err(Error::Busy);
        }
        if enabled && !self.history_update {
            let document = self.document();
            document
                .history
                .execution_order()
                .map_err(|_| Error::Invalid("history dependency cycle"))?;
            // A Dirty recipe with a changed source is exactly what re-enabling update is
            // meant to rebuild; only a missing binding, an Unresolved/Failed node or a
            // cycle keeps the switch off.
            let blocking = history_ops::broken_links(document, &document.history)
                .iter()
                .any(|row| {
                    !row.missing_inputs.is_empty()
                        || !row.missing_outputs.is_empty()
                        || matches!(
                            row.state,
                            EvaluationState::Unresolved(_) | EvaluationState::Failed(_)
                        )
                });
            if blocking {
                return Err(Error::Invalid("history dependency broken"));
            }
        }
        self.history_update = enabled;
        let message = format!("History update {}", if enabled { "on" } else { "off" });
        self.history_notice = Some(message.clone());
        Ok(message)
    }

    pub fn toggle_history_update(&mut self) -> Result<String> {
        self.set_history_update(!self.history_update)
    }

    /// Every recorded feature the panel lists, with the broken-link flag the report adds.
    pub fn history_entries(&self) -> Vec<HistoryEntry> {
        let rows = history_ops::broken_links(self.document(), &self.document().history);
        self.document()
            .history
            .recipes
            .values()
            .filter(|recipe| recipe.state != EvaluationState::Detached)
            .map(|recipe| {
                let row = rows.iter().find(|row| row.recipe == recipe.id);
                HistoryEntry {
                    recipe: recipe.id,
                    command: recipe.command.clone(),
                    state: recipe.state.clone(),
                    outputs: recipe.outputs.clone(),
                    problem: row.is_some(),
                    detail: row.map_or_else(String::new, describe_problem),
                }
            })
            .collect()
    }

    /// The broken rows alone, for the command-line report.
    pub fn history_broken_links(&self) -> Vec<history_ops::BrokenLink> {
        history_ops::broken_links(self.document(), &self.document().history)
    }

    pub fn history_state_counts(&self) -> std::collections::BTreeMap<&'static str, usize> {
        history_ops::state_counts(&self.document().history)
    }

    /// Detach the recipe that owns `object`; the geometry stays and every other
    /// association in the graph keeps working.
    pub fn clear_object_history(&mut self, object: ObjectId) -> Result<String> {
        if self.is_running() {
            return Err(Error::Busy);
        }
        let base = self.document().clone();
        if !base.entities.contains_key(&object) {
            return Err(Error::Missing("history object"));
        }
        let id = base
            .history
            .owner(object)
            .ok_or(Error::Missing("history recipe"))?;
        if base
            .history
            .recipes
            .get(&id)
            .is_some_and(|recipe| recipe.policy.lock)
        {
            return Err(Error::Invalid("history recipe locked"));
        }
        let mut draft = base.clone();
        history_ops::detach_object(&mut draft.history, object)?;
        let recipe = draft
            .history
            .recipes
            .get(&id)
            .cloned()
            .ok_or(Error::Missing("history recipe"))?;
        let prepared = self.coordinator.prepare(TransactionRequest::new(
            "Clear History",
            vec![Mutation::HistoryUpdate {
                document: self.document,
                expected_revision: base.revision,
                recipes: vec![Box::new(recipe)],
                published: vec![],
            }],
        ))?;
        let before = self.document().clone();
        let receipt = self.coordinator.commit(prepared)?;
        if receipt.published {
            self.refresh_scene()?;
            self.record_history_change(&before, ChangeCause::Edit);
            self.sync_published(ChangeCause::Edit);
        }
        let message = format!("History cleared for object {object}");
        self.history_notice = Some(message.clone());
        Ok(message)
    }

    pub fn clear_selected_history(&mut self) -> Result<String> {
        let object = self
            .selection
            .iter()
            .next()
            .copied()
            .ok_or(Error::Missing("history object"))?;
        self.clear_object_history(object)
    }

    /// Reopen the builder that produced `object`, seeded from the recorded parameters.
    pub fn begin_builder_from_history(&mut self, object: ObjectId) -> Result<String> {
        if self.is_running() {
            return Err(Error::Busy);
        }
        let document = self.document().clone();
        let id = document
            .history
            .owner(object)
            .ok_or(Error::Missing("history recipe"))?;
        let recipe = document
            .history
            .recipes
            .get(&id)
            .cloned()
            .ok_or(Error::Missing("history recipe"))?;
        if recipe.environment.schema_version != 1 || recipe.environment.command_version == 0 {
            return Err(Error::Invalid("builder schema version"));
        }
        let kind = builder_kind_for_command(&recipe.command)
            .ok_or(Error::Unsupported("history builder"))?;
        let inputs: Vec<ObjectId> = recipe
            .inputs
            .iter()
            .map(|binding| binding.source.object)
            .collect();
        let mut session = BuilderSession::new(kind, &document, &inputs)?;
        session.adopt_history(object, id);
        for (name, value) in &recipe.parameters {
            if session.field(name).is_some() {
                session.set_field(name, value.clone())?;
            }
        }
        let registry = self.builder_registry()?;
        session.refresh(&document, &registry)?;
        self.builder = Some(session);
        let message = format!("Editing {} from history", kind.title());
        self.history_notice = Some(message.clone());
        Ok(message)
    }

    pub fn begin_builder_from_selected_history(&mut self) -> Result<String> {
        let object = self
            .selection
            .iter()
            .next()
            .copied()
            .ok_or(Error::Missing("history object"))?;
        self.begin_builder_from_history(object)
    }

    /// The editor panel's "edit recorded parameter" path. It marks the recipe Dirty and,
    /// when associative update is enabled, rebuilds it and its downstream recipes in the
    /// same call; with update off the row stays Dirty until the user re-enables it.
    pub fn apply_history_parameter(
        &mut self,
        id: HistoryId,
        key: &str,
        value: Value,
    ) -> Result<()> {
        self.set_history_parameter(id, key, value)?;
        if self.history_update {
            self.recompute_history(id)?;
        } else {
            self.history_notice = Some("History update off: recipe marked dirty".into());
        }
        Ok(())
    }

    /// Command-line and menu aliases for the history switches and report.
    pub fn submit_history_text(&mut self, text: &str) -> Option<Result<()>> {
        let lower = text.trim().to_ascii_lowercase();
        match lower.as_str() {
            "historyrecord" | "gvhistoryrecord" | "rhinohistory" | "historyon" => {
                Some(self.toggle_history_record().map(|_| ()))
            }
            "historyupdate" | "gvhistoryupdate" | "historyupdateon" => {
                Some(self.toggle_history_update().map(|_| ()))
            }
            "history" | "historyreport" | "commandhistory" | "historypanel" => {
                self.history_window_requested = true;
                self.history_notice = Some("History report".into());
                Some(Ok(()))
            }
            "clearhistory" | "gvclearhistory" => Some(self.clear_selected_history().map(|_| ())),
            "edithistory" | "historyedit" | "editbuilder" | "editbyhistory" => {
                Some(self.begin_builder_from_selected_history().map(|_| ()))
            }
            _ => None,
        }
    }

    /// Menu/feature IDs for the same history switches.
    pub fn dispatch_history_feature(&mut self, feature: &str) -> Option<Result<()>> {
        match feature {
            "HistoryRecord" | "RhinoHistoryON" | "GVHistoryRecordON" | "HistoryOn" => {
                Some(self.toggle_history_record().map(|_| ()))
            }
            "HistoryUpdate" | "GVHistoryUpdateON" => Some(self.toggle_history_update().map(|_| ())),
            "HistoryReport" | "CommandHistory" | "HistoryPanel" => {
                self.history_window_requested = true;
                Some(Ok(()))
            }
            "ClearHistory" | "InfoSettingsGVClearHistory" => {
                Some(self.clear_selected_history().map(|_| ()))
            }
            "EditHistory" | "HistoryEdit" | "EditBuilderByHistory" => {
                Some(self.begin_builder_from_selected_history().map(|_| ()))
            }
            _ => None,
        }
    }

    pub fn take_history_notice(&mut self) -> Option<String> {
        self.history_notice.take()
    }

    pub fn take_history_window_request(&mut self) -> bool {
        std::mem::take(&mut self.history_window_requested)
    }

    /// Recipes still waiting for their output insert; close/open must not race them.
    pub fn history_has_pending(&self) -> bool {
        history_ops::has_pending(&self.document().history)
    }
}

fn describe_problem(row: &history_ops::BrokenLink) -> String {
    let mut parts = Vec::new();
    for (role, source) in row.missing_inputs.iter().zip(&row.missing_sources) {
        parts.push(format!("missing input {role} (source {source})"));
    }
    for (role, source) in row.changed_inputs.iter().zip(&row.changed_sources) {
        parts.push(format!("changed input {role} (source {source})"));
    }
    for output in &row.missing_outputs {
        parts.push(format!("missing output {output}"));
    }
    if parts.is_empty() {
        match &row.state {
            EvaluationState::Dirty(reason) => format!("stale: {reason}"),
            EvaluationState::Unresolved(reason) => format!("unresolved: {reason}"),
            EvaluationState::Failed(reason) => format!("failed: {reason}"),
            other => format!("{other:?}"),
        }
    } else {
        parts.join("; ")
    }
}

fn builder_kind_for_command(command: &str) -> Option<BuilderKind> {
    match command {
        "RingRail" | "RingBuilder" => Some(BuilderKind::RingRail),
        "RailProfile" => Some(BuilderKind::RailProfile),
        "RailSweep" => Some(BuilderKind::RailSweep),
        "OutsideRail" => Some(BuilderKind::OutsideRail),
        "RingResizer" => Some(BuilderKind::RingResizer),
        _ => None,
    }
}

/// The recorded recipe behind one object, for the panel's edit row.
pub fn recipe_for_object(document: &om9_core::model::Document, object: ObjectId) -> Option<Recipe> {
    let id = document.history.owner(object)?;
    document.history.recipes.get(&id).cloned()
}
