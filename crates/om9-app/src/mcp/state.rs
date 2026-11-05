// SPDX-License-Identifier: LGPL-2.1-or-later
//! MCP typed state projection: objects, selection, layers, command and events.
use super::Server;
use om9_core::geometry::Geometry;
use serde_json::{Value, json};

impl Server {
    pub(super) fn memory_diagnostics(&self, args: &Value) -> Result<Value, String> {
        if !args.is_object() || args.as_object().is_some_and(|fields|
            fields.keys().any(|key| key != "additional_pending_bytes")) {
            return Err("expected an object with optional additional_pending_bytes".into());
        }
        let requested = args.get("additional_pending_bytes").map(|value|
            value.as_u64().ok_or_else(|| "additional_pending_bytes must be a nonnegative u64 integer".to_string())).transpose()?;
        let controller = self.controller.as_ref().ok_or(super::NO_DOCUMENT)?;
        let report = controller.coordinator().memory_diagnostics(controller.scene_cache_usage(), requested);
        serde_json::to_value(report).map_err(super::message)
    }

    pub(super) fn snapshot(&self, limit: usize) -> Value {
        let Some(controller) = self.controller.as_ref() else {
            return json!({"document": null, "hint": "call om9_new or om9_open"});
        };
        let document = controller.document();
        let selection: Vec<Value> = controller
            .selected_ordered()
            .iter()
            .map(|id| {
                json!({
                    "id": id.to_string(),
                    "label": document.entities.get(id).map(|entity| entity.label.as_str()).unwrap_or(""),
                })
            })
            .collect();
        let layers: Vec<Value> = document
            .layers
            .values()
            .map(|layer| {
                json!({
                    "id": layer.id.to_string(),
                    "name": layer.name,
                    "visible": layer.visible,
                    "selectable": layer.selectable,
                    "locked": layer.locked,
                    "color": layer.color,
                })
            })
            .collect();
        let objects: Vec<Value> = document
            .entities
            .values()
            .take(limit)
            .map(|entity| {
                json!({
                    "id": entity.id.to_string(),
                    "label": entity.label,
                    "kind": entity.geometry.kind().tag(),
                    "layer": entity
                        .attributes
                        .layer
                        .and_then(|id| document.layers.get(&id))
                        .map(|layer| layer.name.clone()),
                    "visible": entity.attributes.visible,
                    "hidden": entity.attributes.hidden,
                    "selectable": entity.attributes.selectable,
                    "locked": entity.attributes.locked,
                    "placement": entity.placement,
                    "geometry": geometry_view(&entity.geometry),
                })
            })
            .collect();
        let command = controller.active_command_input_snapshot();
        json!({
            "document": {
                "id": document.id.to_string(),
                "name": document.name,
                "revision": document.revision,
                "objects": document.entities.len(),
                "layers": document.layers.len(),
                "current_layer": document
                    .current_layer
                    .and_then(|id| document.layers.get(&id))
                    .map(|layer| layer.name.clone()),
                "path": self.path.as_ref().map(|path| path.display().to_string()),
            },
            "command": {
                "running": controller.is_running(),
                "feature": controller.running_feature(),
                "repeat": controller.repeat_slot(),
                "phase": format!("{:?}", controller.phase()),
                "prompt": controller.prompt_with_options(),
                "options": command
                    .options
                    .iter()
                    .map(|option| option.label.clone())
                    .collect::<Vec<_>>(),
            },
            "selection": selection,
            "undo": controller.coordinator().undo_len(),
            "benchmark_no_undo": controller.coordinator().benchmark_no_undo(),
            "redo": controller.coordinator().redo_len(),
            "layers": layers,
            "objects_total": document.entities.len(),
            "objects_truncated": document.entities.len() > limit,
            "objects": objects,
            "last_change": self.last_change(),
        })
    }

    pub(super) fn last_change(&self) -> Option<Value> {
        let controller = self.controller.as_ref()?;
        let batch = controller.last_document_change()?;
        Some(json!({
            "cause": format!("{:?}", batch.cause),
            "from_revision": batch.from_revision,
            "to_revision": batch.to_revision,
            "changes": batch
                .changes
                .iter()
                .map(|change| format!("{change:?}"))
                .collect::<Vec<_>>(),
        }))
    }

    pub(super) fn drain_events(&mut self) -> Vec<Value> {
        let Some(controller) = self.controller.as_ref() else {
            return Vec::new();
        };
        let log = controller.events();
        let events: Vec<Value> = log
            .events()
            .iter()
            .filter(|event| event.sequence > self.last_sequence)
            .map(|event| {
                json!({
                    "seq": event.sequence,
                    "kind": event.kind.label(),
                    "command": event.command,
                    "detail": event.detail,
                    "outcome": event.outcome.map(|outcome| format!("{outcome:?}").to_lowercase()),
                })
            })
            .collect();
        if let Some(last) = log.events().back() {
            self.last_sequence = last.sequence;
        }
        events
    }
}

fn geometry_view(geometry: &Geometry) -> Value {
    match geometry {
        Geometry::Mesh(mesh) => json!({
            "kind": "Mesh",
            "vertices": mesh.vertices.len(),
            "faces": mesh.faces.len(),
        }),
        Geometry::Cloud(cloud) => json!({"kind": "Cloud", "points": cloud.points.len()}),
        Geometry::Brep(_) => json!({"kind": "Brep"}),
        Geometry::NativeBrep(brep) => json!({
            "kind": "NativeBrep",
            "display_vertices": brep.display.vertices.len(),
            "payload_bytes": brep.payload.len(),
        }),
        other => {
            let kind = other.kind().tag();
            let value = serde_json::to_value(other).unwrap_or(Value::Null);
            let bytes = value.to_string().len();
            if bytes > 4096 {
                json!({"kind": kind, "omitted_bytes": bytes})
            } else {
                json!({"kind": kind, "value": value})
            }
        }
    }
}

pub(super) fn limit(args: &Value) -> usize {
    args.get("limit")
        .and_then(Value::as_u64)
        .unwrap_or(100)
        .clamp(1, 5000) as usize
}
