// SPDX-License-Identifier: LGPL-2.1-or-later
//! Explicit native plugin loading and the shared CLI/CMD transaction boundary.

use super::Controller;
use om9_core::{
    Error, Result, changes::ChangeCause, identity::ObjectId, transaction::TransactionRequest,
};
use om9_plugin_host::{LoadedPlugin, ProjectState};
use om9_scene::scene::Scene;
use om9_sdk::{MessageLevel, PluginDescriptor};
use serde_json::Value;
use std::{path::Path, rc::Rc};

fn project_change(batch: &om9_core::changes::ChangeBatch) -> om9_plugin_host::ProjectChange {
    use om9_core::changes::Change;
    let changes = batch
        .changes
        .iter()
        .map(|change| {
            let (kind, id) = match change {
                Change::CurrentLayerChanged(id) => {
                    ("current_layer_changed", id.map(|id| id.to_string()))
                }
                Change::LayerOrderChanged => ("layer_order_changed", None),
                Change::LayerCreated(id) => ("layer_created", Some(id.to_string())),
                Change::LayerModified(id) => ("layer_modified", Some(id.to_string())),
                Change::LayerRemoved(id) => ("layer_removed", Some(id.to_string())),
                Change::EntityAdded(id) => ("entity_added", Some(id.to_string())),
                Change::EntityRemoved(id) => ("entity_removed", Some(id.to_string())),
                Change::EntityModified(id) => ("entity_modified", Some(id.to_string())),
                Change::ExtensionChanged(id) => ("extension_changed", Some(id.to_string())),
                Change::GroupCreated(id) => ("group_created", Some(id.to_string())),
                Change::GroupModified(id) => ("group_modified", Some(id.to_string())),
                Change::GroupRemoved(id) => ("group_removed", Some(id.to_string())),
                Change::FeatureAdded(id) => ("feature_added", Some(id.to_string())),
                Change::FeatureModified(id) => ("feature_modified", Some(id.to_string())),
                Change::FeatureRemoved(id) => ("feature_removed", Some(id.to_string())),
            };
            om9_plugin_host::ChangeEntry {
                kind: kind.into(),
                id,
            }
        })
        .collect();
    om9_plugin_host::ProjectChange {
        from_revision: batch.from_revision,
        to_revision: batch.to_revision,
        cause: match batch.cause {
            om9_core::changes::ChangeCause::Edit => "edit",
            om9_core::changes::ChangeCause::Undo => "undo",
            om9_core::changes::ChangeCause::Redo => "redo",
            om9_core::changes::ChangeCause::Replace => "replace",
            om9_core::changes::ChangeCause::Restore => "restore",
        }
        .into(),
        changes,
    }
}
fn plugin_error(error: impl std::fmt::Display) -> Error {
    Error::Io(format!("plugin: {error}"))
}

fn token(text: &str) -> Option<(&str, &str)> {
    let text = text.trim_start();
    if text.is_empty() {
        return None;
    }
    let end = text.find(char::is_whitespace).unwrap_or(text.len());
    Some((&text[..end], text[end..].trim_start()))
}

impl Controller {
    /// Load a package only on an explicit request. Its library remains live for
    /// this controller; duplicate plugin IDs are refused.
    pub fn load_plugin(&mut self, path: &Path) -> Result<PluginDescriptor> {
        if self.is_running() {
            return Err(Error::Busy);
        }
        let plugin = LoadedPlugin::open(path).map_err(plugin_error)?;
        let descriptor = plugin.descriptor().clone();
        if self
            .plugins
            .iter()
            .any(|loaded| loaded.descriptor().id == descriptor.id)
        {
            return Err(Error::Invalid("duplicate plugin ID"));
        }
        self.plugins
            .try_reserve(1)
            .map_err(|_| Error::Budget("loaded plugins"))?;
        self.plugins.push(Rc::new(plugin));
        Ok(descriptor)
    }

    pub fn plugin_descriptors(&self) -> Vec<PluginDescriptor> {
        self.plugins
            .iter()
            .map(|plugin| plugin.descriptor().clone())
            .collect()
    }

    /// The same service used by live CMD invocation and the headless CLI.
    pub fn run_loaded_plugin(
        &mut self,
        id: &str,
        command: &str,
        arguments: Value,
    ) -> Result<Value> {
        let plugin = self
            .plugins
            .iter()
            .find(|plugin| plugin.descriptor().id == id)
            .cloned()
            .ok_or(Error::Missing("loaded plugin"))?;
        self.run_plugin(&plugin, command, arguments)
    }

    /// Plugins stage owned mutations. A successful command publishes a fully
    /// prepared scene and document together, with one ordinary OM9 Undo entry.
    pub fn run_plugin(
        &mut self,
        plugin: &LoadedPlugin,
        command: &str,
        arguments: Value,
    ) -> Result<Value> {
        if self.is_running() {
            return Err(Error::Busy);
        }
        let event_command = format!("{}:{command}", plugin.descriptor().id);
        self.events.emit(
            crate::command_events::EventKind::Started,
            &event_command,
            "plugin started",
            None,
        );
        let result = (|| {
            let selection = self.selected().iter().copied().collect::<Vec<_>>();
            let last_change = self.plugin_change.as_ref().map(project_change);
            let project = ProjectState {
                path: self.file.path.clone(),
                dirty: self.is_dirty()?,
            };
            let run = plugin
                .prepare(
                    &self.coordinator,
                    self.document,
                    &selection,
                    project,
                    last_change,
                    command,
                    arguments,
                )
                .map_err(plugin_error)?;
            // The plugin reports progress/warnings through the same bounded event log
            // the Command line and its observers read (OM9-L3-01-02-010).
            for message in &run.messages {
                self.events.emit(
                    match message.level {
                        MessageLevel::Progress => crate::command_events::EventKind::Progress,
                        MessageLevel::Warning => crate::command_events::EventKind::Warn,
                    },
                    &event_command,
                    &message.text,
                    None,
                );
            }
            let staged_selection = run.selection.clone();
            let before = self.document().clone();
            if let Some(request) = run.request {
                self.publish_plugin_request(request, staged_selection.as_deref())?;
            } else if let Some(ids) = staged_selection.as_deref() {
                self.apply_plugin_selection_now(ids)?;
            }
            if run.save_as
                && let Err(error) =
                    self.begin_file_operation(crate::file_lifecycle::FileKind::SaveAs)
            {
                // Save As is a host flow: the commit already happened, the path dialog is
                // the user's next step, and a busy file operation can only warn.
                self.events.emit(
                    crate::command_events::EventKind::Warn,
                    &event_command,
                    &format!("project save-as failed: {error}"),
                    None,
                );
            }
            if run.save
                && let Err(error) = self.begin_file_operation(crate::file_lifecycle::FileKind::Save)
            {
                // The command edits are committed; a failed save must not turn the
                // command into a failure. The document stays dirty and the user sees why.
                self.events.emit(
                    crate::command_events::EventKind::Warn,
                    &event_command,
                    &format!("project save failed: {error}"),
                    None,
                );
            }
            // Record the committed difference of this plugin command for the next one.
            if let Some(batch) = om9_core::changes::ChangeBatch::between(
                &before,
                self.document(),
                om9_core::changes::ChangeCause::Edit,
            )
            .ok()
            .flatten()
            {
                self.plugin_change = Some(batch);
            }
            Ok(run.result)
        })();
        self.events.emit(
            crate::command_events::EventKind::Finished,
            &event_command,
            if result.is_ok() {
                "plugin finished"
            } else {
                "plugin refused"
            },
            Some(if result.is_ok() {
                crate::command_events::EventOutcome::Committed
            } else {
                crate::command_events::EventOutcome::Refused
            }),
        );
        result
    }

    /// Validate one staged plugin selection against a scene with the same rule
    /// `select_objects` uses: a canonical, selectable root with pick geometry.
    fn validate_plugin_selection(
        scene: &Scene,
        ids: &[ObjectId],
        limits: &om9_core::budget::Limits,
    ) -> Result<std::collections::BTreeSet<ObjectId>> {
        if ids.len() as u64 > limits.max_outputs {
            return Err(Error::Budget("plugin selection"));
        }
        let mut next = std::collections::BTreeSet::new();
        for id in ids {
            if id.is_nil()
                || !scene.objects.iter().any(|object| {
                    object.root == *id && object.selectable && object.has_pick_geometry()
                })
            {
                return Err(Error::Permission("plugin selected root"));
            }
            next.insert(*id);
        }
        Ok(next)
    }

    fn apply_plugin_selection(&mut self, selection: std::collections::BTreeSet<ObjectId>) {
        let order: Vec<_> = selection.iter().copied().collect();
        if selection != self.selection || order != self.selection_order {
            self.selection_revision = self.selection_revision.saturating_add(1);
        }
        self.selection = selection;
        self.selection_order = order;
        self.clear_subselection();
    }

    /// Apply a selection from a command that changed no document state.
    fn apply_plugin_selection_now(&mut self, ids: &[ObjectId]) -> Result<()> {
        let next = {
            let scene = self
                .scene
                .as_ref()
                .ok_or(Error::Missing("plugin selection scene"))?;
            if scene.epoch != self.coordinator.epoch() {
                return Err(Error::Stale("plugin selection scene"));
            }
            Self::validate_plugin_selection(scene, ids, self.coordinator.limits())?
        };
        self.apply_plugin_selection(next);
        Ok(())
    }

    /// Prepare the candidate, validate the staged selection against the scene that will
    /// be published, then commit. A bad selection refuses the command before publication.
    fn publish_plugin_request(
        &mut self,
        request: TransactionRequest,
        selection: Option<&[ObjectId]>,
    ) -> Result<()> {
        let old_scene = self.scene.as_ref().ok_or(Error::Missing("plugin scene"))?;
        if old_scene.epoch != self.coordinator.epoch() {
            return Err(Error::Stale("plugin scene"));
        }
        let old_usage = self.scene_cache_usage()?;
        let mut prepared = self
            .coordinator
            .prepare_with_held_usage(request, old_usage)?;
        let candidate = prepared
            .candidate_world()
            .documents
            .get(&self.document)
            .ok_or(Error::Missing("plugin candidate document"))?
            .clone();
        if std::sync::Arc::ptr_eq(
            &candidate,
            &self.coordinator.world().documents[&self.document],
        ) {
            let staged = match selection {
                Some(ids) => Some(Self::validate_plugin_selection(
                    old_scene,
                    ids,
                    self.coordinator.limits(),
                )?),
                None => None,
            };
            self.coordinator.commit(prepared)?;
            if let Some(selection) = staged {
                self.apply_plugin_selection(selection);
            }
            return Ok(());
        }
        let epoch = self
            .coordinator
            .epoch()
            .checked_add(1)
            .ok_or(Error::Overflow("plugin scene epoch"))?;
        // Include the candidate, private Undo and preparation maps before any
        // scene allocation, then reserve the actual new cache before publish.
        let scene_limits = self
            .coordinator
            .prepared_remaining_limits(&prepared, old_usage)?;
        let scene = Scene::capture(candidate, epoch, &scene_limits)?;
        let staged = match selection {
            Some(ids) => Some(Self::validate_plugin_selection(
                &scene,
                ids,
                self.coordinator.limits(),
            )?),
            None => None,
        };
        let mut held = old_usage;
        held.add(scene.cache_usage()?, self.coordinator.limits())?;
        self.coordinator
            .reserve_prepared_held_usage(&mut prepared, held)?;
        let receipt = self.coordinator.commit(prepared)?;
        if receipt.published {
            self.scene = Some(scene);
            self.prune_selection();
            self.clear_subselection();
            self.clear_gumball_preview();
            self.sync_published(ChangeCause::Edit);
        }
        if let Some(selection) = staged {
            self.apply_plugin_selection(selection);
        }
        Ok(())
    }

    pub(super) fn submit_plugin_text(&mut self, text: &str) -> Option<Result<()>> {
        let (name, rest) = token(text)?;
        if name.eq_ignore_ascii_case("PluginLoad") {
            return Some((|| {
                let path: String = serde_json::from_str(rest).map_err(plugin_error)?;
                let descriptor = self.load_plugin(Path::new(&path))?;
                self.events.emit(
                    crate::command_events::EventKind::Finished,
                    "PluginLoad",
                    &format!("loaded {}", descriptor.id),
                    None,
                );
                Ok(())
            })());
        }
        if !name.eq_ignore_ascii_case("PluginRun") {
            return None;
        }
        Some((|| {
            let (id, rest) = token(rest).ok_or(Error::Invalid("plugin ID required"))?;
            let (command, json) = token(rest).ok_or(Error::Invalid("plugin command required"))?;
            let arguments: Value = serde_json::from_str(json).map_err(plugin_error)?;
            if !arguments.is_object() {
                return Err(Error::Invalid("plugin arguments must be a JSON object"));
            }
            self.run_loaded_plugin(id, command, arguments).map(|_| ())
        })())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use om9_core::{
        budget::Limits,
        geometry::{Curve, Geometry},
        identity::ObjectId,
        model::{Document, Entity},
        transaction::Mutation,
    };

    #[test]
    fn plugin_budget_refusal_preserves_scene_and_document() {
        let limits = Limits {
            max_points: 1,
            ..Limits::default()
        };
        let mut controller =
            Controller::new(Document::with_template_tolerance("budget"), limits).unwrap();
        let before = controller.document().clone();
        let epoch = controller.scene().unwrap().epoch;
        let mutations = (0..2)
            .map(|n| Mutation::Insert {
                document: controller.document,
                entity: Entity::new(
                    "point",
                    Geometry::Point([n as f64, 0.0, 0.0]),
                    controller.document().current_layer,
                ),
            })
            .collect();
        assert!(matches!(
            controller
                .publish_plugin_request(TransactionRequest::new("over budget", mutations), None),
            Err(Error::Budget(_))
        ));
        assert_eq!(controller.document(), &before);
        assert_eq!(controller.scene().unwrap().epoch, epoch);
        assert!(controller.scene().unwrap().objects.is_empty());
        assert_eq!(controller.coordinator().undo_len(), 0);
        assert_eq!(controller.coordinator().redo_len(), 0);
    }

    #[test]
    fn staged_plugin_batch_publishes_scene_in_one_undo_unit() {
        let mut controller = Controller::new(
            Document::with_template_tolerance("plugins"),
            Limits::default(),
        )
        .unwrap();
        let line = Entity::new(
            "line",
            Geometry::Curve(Curve::Line {
                start: [0.0; 3],
                end: [10.0, 0.0, 0.0],
                domain: [0.0, 10.0],
            }),
            controller.document().current_layer,
        );
        let polyline = Entity::new(
            "polyline",
            Geometry::Curve(
                om9_core::curve_build::polyline(&[[0.0; 3], [0.0, 5.0, 0.0], [5.0, 5.0, 0.0]])
                    .unwrap(),
            ),
            controller.document().current_layer,
        );
        let ids = [line.id, polyline.id];
        controller
            .publish_plugin_request(
                TransactionRequest::new(
                    "plugin batch",
                    vec![
                        Mutation::Insert {
                            document: controller.document,
                            entity: line,
                        },
                        Mutation::Insert {
                            document: controller.document,
                            entity: polyline,
                        },
                    ],
                ),
                None,
            )
            .unwrap();
        assert_eq!(controller.document().entities.len(), 2);
        assert_eq!(controller.scene().unwrap().objects.len(), 2);
        assert_eq!(
            controller.scene().unwrap().epoch,
            controller.coordinator().epoch()
        );
        assert_eq!(controller.coordinator().undo_len(), 1);
        controller.undo().unwrap();
        assert!(controller.document().entities.is_empty());
        assert!(controller.scene().unwrap().objects.is_empty());
        controller.redo().unwrap();
        assert!(
            ids.iter()
                .all(|id| controller.document().entities.contains_key(id))
        );
        assert_eq!(controller.scene().unwrap().objects.len(), 2);
    }

    #[test]
    fn refused_plugin_batch_keeps_scene_document_and_redo() {
        let mut controller = Controller::new(
            Document::with_template_tolerance("plugins"),
            Limits::default(),
        )
        .unwrap();
        let point = Entity::new(
            "point",
            Geometry::Point([0.0; 3]),
            controller.document().current_layer,
        );
        controller
            .publish_plugin_request(
                TransactionRequest::new(
                    "seed",
                    vec![Mutation::Insert {
                        document: controller.document,
                        entity: point,
                    }],
                ),
                None,
            )
            .unwrap();
        controller.undo().unwrap();
        let before = controller.document().clone();
        let epoch = controller.scene().unwrap().epoch;
        let valid = Entity::new(
            "valid",
            Geometry::Point([1.0; 3]),
            controller.document().current_layer,
        );
        let invalid = Entity::new(
            "invalid",
            Geometry::Point([f64::NAN, 0.0, 0.0]),
            controller.document().current_layer,
        );
        assert!(
            controller
                .publish_plugin_request(
                    TransactionRequest::new(
                        "invalid batch",
                        vec![
                            Mutation::Insert {
                                document: controller.document,
                                entity: valid
                            },
                            Mutation::Insert {
                                document: controller.document,
                                entity: invalid
                            },
                        ]
                    ),
                    None
                )
                .is_err()
        );
        assert_eq!(controller.document(), &before);
        assert_eq!(controller.scene().unwrap().epoch, epoch);
        assert!(controller.scene().unwrap().objects.is_empty());
        assert_eq!(controller.coordinator().redo_len(), 1);
        controller.redo().unwrap();
        assert_eq!(controller.document().entities.len(), 1);
    }

    #[test]
    fn plugin_selection_is_validated_before_publication_and_applied_after() {
        let mut controller = Controller::new(
            Document::with_template_tolerance("selection"),
            Limits::default(),
        )
        .unwrap();
        let first = Entity::new(
            "first",
            Geometry::Point([0.0; 3]),
            controller.document().current_layer,
        );
        let second = Entity::new(
            "second",
            Geometry::Point([1.0; 3]),
            controller.document().current_layer,
        );
        let ids = [first.id, second.id];
        let request = TransactionRequest::new(
            "insert pair",
            vec![
                Mutation::Insert {
                    document: controller.document,
                    entity: first,
                },
                Mutation::Insert {
                    document: controller.document,
                    entity: second,
                },
            ],
        );
        controller
            .publish_plugin_request(request, Some(&ids))
            .unwrap();
        assert_eq!(
            controller.selected(),
            &ids.iter()
                .copied()
                .collect::<std::collections::BTreeSet<_>>()
        );
        assert_eq!(controller.coordinator().undo_len(), 1);

        // A selection that names an unknown root refuses the whole command.
        let before = controller.document().clone();
        let epoch = controller.scene().unwrap().epoch;
        let stranger = Entity::new(
            "stranger",
            Geometry::Point([2.0; 3]),
            controller.document().current_layer,
        );
        let stranger_id = stranger.id;
        let request = TransactionRequest::new(
            "unknown selection",
            vec![Mutation::Insert {
                document: controller.document,
                entity: stranger,
            }],
        );
        let bogus = [ObjectId(uuid::Uuid::new_v4())];
        assert!(
            controller
                .publish_plugin_request(request, Some(&bogus))
                .is_err()
        );
        assert_eq!(controller.document(), &before);
        assert_eq!(controller.scene().unwrap().epoch, epoch);
        assert_eq!(controller.coordinator().undo_len(), 1);
        assert!(!controller.document().entities.contains_key(&stranger_id));
        assert_eq!(
            controller.selected(),
            &ids.iter()
                .copied()
                .collect::<std::collections::BTreeSet<_>>()
        );
    }
    #[test]
    fn cmd_plugin_errors_preserve_document_and_existing_command() {
        let mut controller = Controller::new(
            Document::with_template_tolerance("plugins"),
            Limits::default(),
        )
        .unwrap();
        let before = controller.document().clone();
        for command in [
            "PluginRun",
            "PluginRun missing command []",
            "PluginLoad not-json",
            "PluginRun missing command {}",
        ] {
            assert!(controller.submit(command).is_err());
            assert_eq!(controller.document(), &before);
            assert_eq!(controller.coordinator().undo_len(), 0);
        }
        controller.submit("Line").unwrap();
        assert!(controller.submit("PluginLoad \"missing.om9p\"").is_err());
        assert!(controller.is_running());
        assert_eq!(controller.document(), &before);
    }
}
