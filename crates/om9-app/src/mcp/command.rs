// SPDX-License-Identifier: LGPL-2.1-or-later
//! MCP command, dispatch, click and selection tools.
use super::{NO_DOCUMENT, Server, message, point3, required_str};
use crate::session::{Controller, DispatchOutcome, InvocationOrigin};
use om9_core::identity::ObjectId;
use serde_json::{Value, json};

impl Server {
    pub(super) fn controller(&mut self) -> Result<&mut Controller, String> {
        self.controller.as_mut().ok_or_else(|| NO_DOCUMENT.into())
    }

    pub(super) fn command(&mut self, text: &str) -> Result<Value, String> {
        let outcome = {
            let controller = self.controller()?;
            controller.submit_effect(text)
        };
        let events = self.drain_events();
        let result = match outcome {
            Ok(DispatchOutcome::Command) => json!({"ok": true}),
            Ok(DispatchOutcome::Top11(result)) => json!({"ok": true, "message": result.message()}),
            Ok(DispatchOutcome::File(_)) => json!({
                "ok": true,
                "file": "file commands need a dialog; use om9_open / om9_save / om9_report"
            }),
            Ok(DispatchOutcome::StlExport) => json!({
                "ok": true,
                "stl": "selection ready; call om9_export_stl with a destination path"
            }),
            Err(error) => json!({"ok": false, "error": error.to_string()}),
        };
        Ok(json!({"result": result, "events": events, "state": self.snapshot(50)}))
    }

    pub(super) fn dispatch(&mut self, args: &Value) -> Result<Value, String> {
        let feature = required_str(args, "feature")?.to_owned();
        let origin = match args.get("origin").and_then(Value::as_str).unwrap_or("menu") {
            "menu" => InvocationOrigin::Menu,
            "cmd" => InvocationOrigin::Cmd,
            "f6" => InvocationOrigin::F6,
            "builder" => InvocationOrigin::Builder,
            other => {
                return Err(format!(
                    "origin must be menu, cmd, f6 or builder (got `{other}`)"
                ));
            }
        };
        let outcome = {
            let controller = self.controller()?;
            controller.dispatch_effect(&feature, origin)
        };
        let events = self.drain_events();
        let result = match outcome {
            Ok(_) => json!({"ok": true}),
            Err(error) => json!({"ok": false, "error": error.to_string()}),
        };
        Ok(json!({"result": result, "events": events, "state": self.snapshot(50)}))
    }

    pub(super) fn point(&mut self, args: &Value) -> Result<Value, String> {
        let point = args
            .get("point")
            .and_then(point3)
            .ok_or("`point` must be three finite numbers [x, y, z]")?;
        let outcome = {
            let controller = self.controller()?;
            controller.accept_world_point(point)
        };
        let events = self.drain_events();
        let result = match outcome {
            Ok(()) => json!({"ok": true}),
            Err(error) => json!({"ok": false, "error": error.to_string()}),
        };
        Ok(json!({"result": result, "events": events, "state": self.snapshot(50)}))
    }

    pub(super) fn select(&mut self, args: &Value) -> Result<Value, String> {
        if args.get("clear").and_then(Value::as_bool).unwrap_or(false) {
            self.controller()?.clear_selection();
            return Ok(self.snapshot(50));
        }
        let ids = self.selection_ids(args)?;
        self.controller()?.select_objects(&ids).map_err(message)?;
        Ok(self.snapshot(50))
    }

    pub(super) fn selection_ids(&self, args: &Value) -> Result<Vec<ObjectId>, String> {
        if let Some(list) = args.get("ids").and_then(Value::as_array) {
            return list
                .iter()
                .map(|value| {
                    value
                        .as_str()
                        .ok_or_else(|| "`ids` entries must be strings".to_owned())
                        .and_then(parse_id)
                })
                .collect();
        }
        if let Some(labels) = args.get("labels").and_then(Value::as_array) {
            let controller = self.controller.as_ref().ok_or(NO_DOCUMENT)?;
            let document = controller.document();
            let mut ids = Vec::new();
            for label in labels {
                let label = label
                    .as_str()
                    .ok_or_else(|| "`labels` entries must be strings".to_owned())?;
                let matches: Vec<ObjectId> = document
                    .entities
                    .values()
                    .filter(|entity| entity.label == label)
                    .map(|entity| entity.id)
                    .collect();
                match matches.as_slice() {
                    [] => return Err(format!("no object labelled `{label}`")),
                    [id] => ids.push(*id),
                    many => {
                        return Err(format!(
                            "label `{label}` is ambiguous; use one of: {}",
                            many.iter()
                                .map(|id| id.to_string())
                                .collect::<Vec<_>>()
                                .join(", ")
                        ));
                    }
                }
            }
            return Ok(ids);
        }
        if args.get("all").and_then(Value::as_bool).unwrap_or(false) {
            let controller = self.controller.as_ref().ok_or(NO_DOCUMENT)?;
            let scene = controller.scene().ok_or("no scene")?;
            let mut ids: Vec<ObjectId> = Vec::new();
            for object in &scene.objects {
                if object.selectable && object.has_pick_geometry() && !ids.contains(&object.root) {
                    ids.push(object.root);
                }
            }
            return Ok(ids);
        }
        Err("give `ids`, `labels`, `all: true` or `clear: true`".into())
    }

    pub(super) fn undo_redo(&mut self, undo: bool) -> Result<Value, String> {
        {
            let controller = self.controller()?;
            if undo {
                controller.undo()
            } else {
                controller.redo()
            }
        }
        .map_err(message)?;
        let events = self.drain_events();
        Ok(json!({"events": events, "state": self.snapshot(50)}))
    }
}

fn parse_id(text: &str) -> Result<ObjectId, String> {
    uuid::Uuid::parse_str(text.trim())
        .map(ObjectId)
        .map_err(|_| format!("invalid object id `{text}`"))
}
