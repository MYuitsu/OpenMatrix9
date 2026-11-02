// SPDX-License-Identifier: LGPL-2.1-or-later
//! Headless Model Context Protocol server (`om9 --mcp`).
//!
//! One JSON-RPC 2.0 message per line on stdin, one reply per request on stdout, so an agent client
//! can run the same `Controller` the Command Line runs and read typed document state instead of a
//! screenshot: objects with geometry, selection, layers, the active prompt and options, undo depth,
//! the ordered command events and the last change batch.
//!
//! `OM9_MCP_LOG=<path>` appends one JSON line per tool call, so a test run keeps a durable action
//! log next to the interactive `OM9_INTERACTION_LOG`. Only protocol replies go to stdout.
use crate::session::{Controller, DispatchOutcome, InvocationOrigin};
use om9_archive::Archive;
use om9_core::{
    budget::Limits, geometry::Geometry, identity::ObjectId, model::Document,
    transaction::Cancellation,
};
use serde_json::{Value, json};
use std::{
    fs::{File, OpenOptions},
    io::{self, BufRead, Write},
    path::{Path, PathBuf},
    time::Instant,
};

/// Protocol revision echoed when the client does not name one.
pub const PROTOCOL_VERSION: &str = "2024-11-05";

const NO_DOCUMENT: &str = "no document loaded: call om9_new or om9_open first";

pub struct Server {
    controller: Option<Controller>,
    limits: Limits,
    path: Option<PathBuf>,
    last_sequence: u64,
    log: Option<File>,
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}

impl Server {
    pub fn new() -> Self {
        Self {
            controller: None,
            limits: crate::document_limits(),
            path: None,
            last_sequence: 0,
            log: std::env::var_os("OM9_MCP_LOG")
                .and_then(|path| OpenOptions::new().create(true).append(true).open(path).ok()),
        }
    }

    /// One JSON-RPC request in, at most one response out. Notifications get no reply.
    pub fn handle(&mut self, request: &Value) -> Option<Value> {
        let id = request.get("id").cloned()?;
        let method = request.get("method").and_then(Value::as_str).unwrap_or("");
        let result = match method {
            "initialize" => Ok(initialize(request)),
            "ping" => Ok(json!({})),
            "tools/list" => Ok(json!({"tools": tools()})),
            "tools/call" => Ok(self.call_tool(request.get("params").unwrap_or(&Value::Null))),
            "shutdown" => Ok(Value::Null),
            other => Err(format!("unknown method `{other}`")),
        };
        Some(match result {
            Ok(result) => json!({"jsonrpc": "2.0", "id": id, "result": result}),
            Err(message) => {
                json!({"jsonrpc": "2.0", "id": id, "error": {"code": -32601, "message": message}})
            }
        })
    }

    fn call_tool(&mut self, params: &Value) -> Value {
        let name = params.get("name").and_then(Value::as_str).unwrap_or("");
        let arguments = params
            .get("arguments")
            .cloned()
            .unwrap_or_else(|| json!({}));
        let started = Instant::now();
        let result = self.tool(name, &arguments);
        let (text, is_error) = match &result {
            Ok(value) => (pretty(value), false),
            Err(error) => (error.clone(), true),
        };
        self.log_action(name, &arguments, result.is_ok(), started);
        json!({
            "content": [{"type": "text", "text": text}],
            "isError": is_error,
        })
    }

    fn tool(&mut self, name: &str, args: &Value) -> Result<Value, String> {
        match name {
            "om9_new" => {
                let name = args
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or("Untitled");
                let document = Document::with_template_tolerance(name);
                let mut controller = Controller::new(document, self.limits).map_err(message)?;
                controller.record_changes();
                self.controller = Some(controller);
                self.path = None;
                self.last_sequence = 0;
                Ok(self.snapshot(100))
            }
            "om9_open" => {
                let path = path_arg(args, "path")?;
                let mut controller = Controller::from_file(&path, self.limits).map_err(message)?;
                controller.record_changes();
                self.controller = Some(controller);
                self.path = Some(path);
                self.last_sequence = 0;
                Ok(self.snapshot(100))
            }
            "om9_save" => self.save(args),
            "om9_command" => {
                let text = args.get("text").and_then(Value::as_str).unwrap_or("");
                self.command(text)
            }
            "om9_dispatch" => self.dispatch(args),
            "om9_point" => self.point(args),
            "om9_state" => Ok(self.snapshot(limit(args))),
            "om9_select" => self.select(args),
            "om9_undo" => self.undo_redo(true),
            "om9_redo" => self.undo_redo(false),
            "om9_report" => self.report(args),
            "om9_ui" => self.ui_replay(args),
            other => Err(format!("unknown tool `{other}`")),
        }
    }

    /// Replays pointer/key events through the real `MatrixApp` widget tree and returns receipts:
    /// this is the hover/pick/orbit/pan/zoom/rubber-band/menu/focus path the headless geometry
    /// tools cannot see.
    fn ui_replay(&mut self, args: &Value) -> Result<Value, String> {
        let document = self
            .controller
            .as_ref()
            .ok_or(NO_DOCUMENT)?
            .document()
            .clone();
        let size = match args.get("size").and_then(Value::as_array) {
            Some(pair) if pair.len() == 2 => [
                pair[0].as_f64().unwrap_or(1280.) as f32,
                pair[1].as_f64().unwrap_or(720.) as f32,
            ],
            _ => [1280., 720.],
        };
        let dpi = args.get("dpi").and_then(Value::as_f64).unwrap_or(1.0) as f32;
        let steps = args
            .get("steps")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        crate::ui_replay::replay(document, size, dpi, &steps).map_err(message)
    }

    fn controller(&mut self) -> Result<&mut Controller, String> {
        self.controller.as_mut().ok_or_else(|| NO_DOCUMENT.into())
    }

    fn command(&mut self, text: &str) -> Result<Value, String> {
        let outcome = {
            let controller = self.controller()?;
            controller.submit_effect(text)
        };
        let events = self.drain_events();
        let result = match outcome {
            Ok(DispatchOutcome::Command) => json!({"ok": true}),
            Ok(DispatchOutcome::File(_)) => json!({
                "ok": true,
                "file": "file commands need a dialog; use om9_open / om9_save / om9_report"
            }),
            Err(error) => json!({"ok": false, "error": error.to_string()}),
        };
        Ok(json!({"result": result, "events": events, "state": self.snapshot(50)}))
    }

    fn dispatch(&mut self, args: &Value) -> Result<Value, String> {
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

    fn point(&mut self, args: &Value) -> Result<Value, String> {
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

    fn select(&mut self, args: &Value) -> Result<Value, String> {
        if args.get("clear").and_then(Value::as_bool).unwrap_or(false) {
            self.controller()?.clear_selection();
            return Ok(self.snapshot(50));
        }
        let ids = self.selection_ids(args)?;
        self.controller()?.select_objects(&ids).map_err(message)?;
        Ok(self.snapshot(50))
    }

    fn selection_ids(&self, args: &Value) -> Result<Vec<ObjectId>, String> {
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

    fn undo_redo(&mut self, undo: bool) -> Result<Value, String> {
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

    fn save(&mut self, args: &Value) -> Result<Value, String> {
        let path = match args.get("path").and_then(Value::as_str) {
            Some(path) => PathBuf::from(path),
            None => self
                .path
                .clone()
                .ok_or("missing `path`: no document path is open yet")?,
        };
        {
            let controller = self.controller.as_ref().ok_or(NO_DOCUMENT)?;
            Archive::save(
                &path,
                controller.document(),
                &self.limits,
                &Cancellation::default(),
            )
            .map_err(message)?;
        }
        self.path = Some(path.clone());
        let bytes = std::fs::metadata(&path).map(|meta| meta.len()).unwrap_or(0);
        Ok(json!({
            "saved": path.display().to_string(),
            "bytes": bytes,
            "state": self.snapshot(0),
        }))
    }

    fn report(&mut self, args: &Value) -> Result<Value, String> {
        let path = match args.get("path").and_then(Value::as_str) {
            Some(path) => PathBuf::from(path),
            None => std::env::temp_dir().join(format!("om9-report-{}.json", std::process::id())),
        };
        let controller = self.controller.as_ref().ok_or(NO_DOCUMENT)?;
        let mut file = File::create(&path).map_err(message)?;
        crate::report::write(
            &mut file,
            controller.document(),
            controller.file_diagnostics(),
            &self.limits,
        )
        .map_err(message)?;
        file.flush().map_err(message)?;
        let bytes = file.metadata().map(|meta| meta.len()).unwrap_or(0);
        Ok(json!({"path": path.display().to_string(), "bytes": bytes}))
    }

    fn snapshot(&self, limit: usize) -> Value {
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
            "redo": controller.coordinator().redo_len(),
            "layers": layers,
            "objects_total": document.entities.len(),
            "objects_truncated": document.entities.len() > limit,
            "objects": objects,
            "last_change": self.last_change(),
        })
    }

    fn last_change(&self) -> Option<Value> {
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

    fn drain_events(&mut self) -> Vec<Value> {
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

    fn log_action(&mut self, tool: &str, args: &Value, ok: bool, started: Instant) {
        let Some(log) = self.log.as_mut() else {
            return;
        };
        let record = json!({
            "event": "mcp_tool",
            "tool": tool,
            "arguments": args,
            "ok": ok,
            "elapsed_ms": started.elapsed().as_millis() as u64,
            "objects": self.controller.as_ref().map(|controller| controller.document().entities.len()),
            "revision": self.controller.as_ref().map(|controller| controller.document().revision),
        });
        if let Ok(mut line) = serde_json::to_vec(&record) {
            line.push(b'\n');
            let _ = log.write_all(&line);
            let _ = log.flush();
        }
    }
}

/// Run the server until stdin ends. Protocol replies are the only stdout output.
pub fn serve<R: BufRead, W: Write>(mut reader: R, mut writer: W) -> io::Result<()> {
    let mut server = Server::new();
    let mut line = String::new();
    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
            return Ok(());
        }
        if line.trim().is_empty() {
            continue;
        }
        let request: Value = match serde_json::from_str(&line) {
            Ok(request) => request,
            Err(error) => {
                writeln!(
                    writer,
                    "{}",
                    json!({"jsonrpc": "2.0", "id": null, "error": {"code": -32700, "message": error.to_string()}})
                )?;
                writer.flush()?;
                continue;
            }
        };
        if let Some(response) = server.handle(&request) {
            writeln!(writer, "{}", serde_json::to_string(&response)?)?;
            writer.flush()?;
        }
    }
}

/// `om9 --mcp`: stdin/stdout are the transport, so a startup error is a protocol-level failure.
pub fn serve_stdio() -> om9_core::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    serve(stdin.lock(), stdout.lock()).map_err(|error| om9_core::Error::Io(error.to_string()))
}

fn initialize(request: &Value) -> Value {
    let version = request
        .pointer("/params/protocolVersion")
        .and_then(Value::as_str)
        .unwrap_or(PROTOCOL_VERSION);
    json!({
        "protocolVersion": version,
        "capabilities": {"tools": {}},
        "serverInfo": {"name": "om9", "version": env!("CARGO_PKG_VERSION")},
        "instructions": "Headless OM9 test server over the same Controller as the Command Line. Call om9_new or om9_open, then send one command line at a time with om9_command (\"Circle\", \"0,0,0\", \"5\"; empty text is Enter). Use om9_point for the viewport-click path and om9_dispatch for the menu path. Read om9_state for typed geometry, selection, layers, prompt, options, undo depth and command events; om9_undo/om9_redo/om9_save/om9_report cover the rest. No screenshots are needed."
    })
}

fn tools() -> Value {
    json!([
        {
            "name": "om9_new",
            "description": "Start an empty OM9 document, replacing the current session; returns typed state.",
            "inputSchema": {"type": "object", "properties": {"name": {"type": "string"}}}
        },
        {
            "name": "om9_open",
            "description": "Open an .om9 project through the same file path the application uses; returns typed state.",
            "inputSchema": {
                "type": "object",
                "properties": {"path": {"type": "string"}},
                "required": ["path"]
            }
        },
        {
            "name": "om9_save",
            "description": "Save the current document as an .om9 archive.",
            "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}}}
        },
        {
            "name": "om9_command",
            "description": "Type one line into the OM9 command line: a command (Circle), an option (Diameter=10), a coordinate (0,0,0), a value (5) or empty text for Enter. Same implementation as the CMD box.",
            "inputSchema": {"type": "object", "properties": {"text": {"type": "string"}}}
        },
        {
            "name": "om9_dispatch",
            "description": "Start a command through its menu/icon entry by feature id (for example OM9-CURVE-005), to test the menu route.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "feature": {"type": "string"},
                    "origin": {"type": "string", "enum": ["menu", "cmd", "f6", "builder"]}
                },
                "required": ["feature"]
            }
        },
        {
            "name": "om9_point",
            "description": "Click at a world point for the running command (the viewport-click path).",
            "inputSchema": {
                "type": "object",
                "properties": {"point": {"type": "array", "items": {"type": "number"}, "minItems": 3, "maxItems": 3}},
                "required": ["point"]
            }
        },
        {
            "name": "om9_state",
            "description": "Read typed state: objects with geometry, selection, layers, running prompt and options, undo/redo depth, last change and command events.",
            "inputSchema": {
                "type": "object",
                "properties": {"limit": {"type": "integer", "minimum": 1, "maximum": 5000}}
            }
        },
        {
            "name": "om9_select",
            "description": "Set the selection by object ids, by exact labels, every selectable object, or clear it.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "ids": {"type": "array", "items": {"type": "string"}},
                    "labels": {"type": "array", "items": {"type": "string"}},
                    "all": {"type": "boolean"},
                    "clear": {"type": "boolean"}
                }
            }
        },
        {
            "name": "om9_undo",
            "description": "Undo one document transaction.",
            "inputSchema": {"type": "object", "properties": {}}
        },
        {
            "name": "om9_redo",
            "description": "Redo one undone transaction.",
            "inputSchema": {"type": "object", "properties": {}}
        },
        {
            "name": "om9_report",
            "description": "Write the full persistent JSON projection (every object, geometry, history recipe and document hash) to a file for exact verification.",
            "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}}}
        },
        {
            "name": "om9_ui",
            "description": "Replay mouse/keyboard input through the real MatrixApp widget tree (hover, pick, orbit, pan, zoom, rubber-band, menu hover, focus) and return one typed receipt per step: view/panel rects, camera revisions, selection, prompt, focus and command text. Use `steps`: [{\"wait\":1},{\"move\":[x,y]},{\"click\":[x,y],\"button\":\"left\"},{\"drag\":{\"from\":[x,y],\"to\":[x,y],\"steps\":8,\"button\":\"right\"}},{\"wheel\":120,\"at\":[x,y]},{\"key\":\"Escape\"},{\"text\":\"Line\"}].",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "size": {"type": "array", "items": {"type": "number"}, "minItems": 2, "maxItems": 2},
                    "dpi": {"type": "number"},
                    "steps": {"type": "array", "items": {"type": "object"}}
                }
            }
        }
    ])
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

fn pretty(value: &Value) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
}

fn message(error: impl std::fmt::Display) -> String {
    error.to_string()
}

fn required_str<'a>(args: &'a Value, key: &str) -> Result<&'a str, String> {
    args.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("missing non-empty string argument `{key}`"))
}

fn path_arg(args: &Value, key: &str) -> Result<PathBuf, String> {
    required_str(args, key)
        .map(Path::new)
        .map(Path::to_path_buf)
}

fn limit(args: &Value) -> usize {
    args.get("limit")
        .and_then(Value::as_u64)
        .unwrap_or(100)
        .clamp(1, 5000) as usize
}

fn point3(value: &Value) -> Option<[f64; 3]> {
    let array = value.as_array()?;
    if array.len() != 3 {
        return None;
    }
    let point = [array[0].as_f64()?, array[1].as_f64()?, array[2].as_f64()?];
    point.iter().all(|axis| axis.is_finite()).then_some(point)
}

fn parse_id(text: &str) -> Result<ObjectId, String> {
    uuid::Uuid::parse_str(text.trim())
        .map(ObjectId)
        .map_err(|_| format!("invalid object id `{text}`"))
}
