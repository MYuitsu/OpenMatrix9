// SPDX-License-Identifier: LGPL-2.1-or-later
//! Headless Model Context Protocol server (`om9 --mcp`).
//!
//! One JSON-RPC 2.0 message per line on stdin, one reply per request on stdout, so an agent client
//! can run the same `Controller` the Command Line runs and read typed document state instead of a
//! screenshot: objects with geometry, selection, layers, the active prompt and options, undo depth,
//! the ordered command events and the last change batch.
//!
//! `OM9_MCP_LOG=<path>` appends one JSON line per tool call. Only protocol replies go to stdout.
mod command;
mod document;
mod state;
mod tools;
mod ui;
pub mod live;

use crate::session::Controller;
use om9_core::budget::Limits;
use serde_json::{Value, json};
use std::{
    fs::{File, OpenOptions},
    io::{self, BufRead, Write},
    path::PathBuf,
    time::Instant,
};

/// Protocol revision echoed when the client does not name one.
pub const PROTOCOL_VERSION: &str = "2024-11-05";

pub(super) const NO_DOCUMENT: &str = "no document loaded: call om9_new or om9_open first";

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
            "tools/list" => Ok(json!({"tools": tools::schema()})),
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
            "om9_new" => self.new_document(args),
            "om9_open" => self.open_document(args),
            "om9_save" => self.save(args),
            "om9_command" => {
                let text = args.get("text").and_then(Value::as_str).unwrap_or("");
                self.command(text)
            }
            "om9_dispatch" => self.dispatch(args),
            "om9_point" => self.point(args),
            "om9_state" => Ok(self.snapshot(state::limit(args))),
            "om9_memory" => self.memory_diagnostics(args),
            "om9_select" => self.select(args),
            "om9_undo" => self.undo_redo(true),
            "om9_redo" => self.undo_redo(false),
            "om9_report" => self.report(args),
            "om9_export_stl" => self.export_stl(args),
            "om9_ui" => self.ui_replay(args),
            other => Err(format!("unknown tool `{other}`")),
        }
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
        "instructions": "Headless OM9 test server over the same Controller as the Command Line. Call om9_new or om9_open, then send one command line at a time with om9_command (\"Circle\", \"0,0,0\", \"5\"; empty text is Enter). Use om9_point for the viewport-click path and om9_dispatch for the menu path, om9_ui for real widget input. Read om9_state for typed geometry, selection, layers, prompt, options, undo depth and command events; om9_undo/om9_redo/om9_save/om9_report cover the rest. No screenshots are needed."
    })
}

fn pretty(value: &Value) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
}

pub(super) fn message(error: impl std::fmt::Display) -> String {
    error.to_string()
}

pub(super) fn required_str<'a>(args: &'a Value, key: &str) -> Result<&'a str, String> {
    args.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("missing non-empty string argument `{key}`"))
}

pub(super) fn point3(value: &Value) -> Option<[f64; 3]> {
    let array = value.as_array()?;
    if array.len() != 3 {
        return None;
    }
    let point = [array[0].as_f64()?, array[1].as_f64()?, array[2].as_f64()?];
    point.iter().all(|axis| axis.is_finite()).then_some(point)
}
