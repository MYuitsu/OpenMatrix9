// SPDX-License-Identifier: LGPL-2.1-or-later
//! Local authenticated stdio-to-window bridge. The UI thread owns the Controller.
use super::{Server, message, tools};
use crate::{
    file_lifecycle::{FileKind, FileOutcome},
    session::Controller,
};
use om9_core::model::Document;
use serde_json::{Value, json};
use std::{
    fs::{self, OpenOptions},
    io::{self, BufRead, BufReader, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::Duration,
};

const MAX_LINE: u64 = 1024 * 1024;
const IO_TIMEOUT: Duration = Duration::from_secs(5);
struct Request {
    value: Value,
    reply: mpsc::Sender<Value>,
}

fn directory() -> io::Result<PathBuf> {
    let base = std::env::var_os("LOCALAPPDATA")
        .ok_or_else(|| io::Error::other("LOCALAPPDATA is not set"))?;
    Ok(PathBuf::from(base).join("OpenMatrix9").join("mcp-live"))
}

fn line<R: Read>(reader: R) -> io::Result<String> {
    let mut text = String::new();
    let size = BufReader::new(reader)
        .take(MAX_LINE + 1)
        .read_line(&mut text)?;
    if size == 0 || size as u64 > MAX_LINE || !text.ends_with('\n') {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid or oversized request",
        ));
    }
    Ok(text)
}

pub struct LiveHost {
    receiver: mpsc::Receiver<Request>,
    server: Server,
    stop: Arc<AtomicBool>,
    registration: PathBuf,
}

impl LiveHost {
    /// Start only for a real interactive window, never for headless Controller tests.
    pub fn start(ctx: eframe::egui::Context) -> io::Result<Self> {
        let dir = directory()?;
        fs::create_dir_all(&dir)?;
        let listener = TcpListener::bind("127.0.0.1:0")?;
        listener.set_nonblocking(true)?;
        let token = uuid::Uuid::new_v4().to_string();
        let registration = dir.join(format!(
            "{}-{}.json",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&registration)?;
        writeln!(
            file,
            "{}",
            json!({"address": listener.local_addr()?.to_string(), "token": token})
        )?;
        let (sender, receiver) = mpsc::sync_channel::<Request>(1);
        let stop = Arc::new(AtomicBool::new(false));
        let stopping = stop.clone();
        std::thread::spawn(move || {
            while !stopping.load(Ordering::Acquire) {
                match listener.accept() {
                    Ok((mut stream, peer)) if peer.ip().is_loopback() => {
                        let _ = stream.set_read_timeout(Some(IO_TIMEOUT));
                        let _ = stream.set_write_timeout(Some(IO_TIMEOUT));
                        let request = line(&stream)
                            .ok()
                            .and_then(|s| serde_json::from_str::<Value>(&s).ok());
                        let Some(request) = request else { continue };
                        if request.get("token").and_then(Value::as_str) != Some(token.as_str()) {
                            continue;
                        }
                        if request.get("probe").and_then(Value::as_bool) == Some(true) {
                            let _ = writeln!(stream, "{}", json!({"ready":true}));
                            continue;
                        }
                        let Some(value) = request.get("request").cloned() else {
                            continue;
                        };
                        let (reply, response) = mpsc::channel();
                        if sender.send(Request { value, reply }).is_err() {
                            break;
                        }
                        ctx.request_repaint();
                        loop {
                            match response.recv_timeout(Duration::from_millis(200)) {
                                Ok(value) => {
                                    let _ = writeln!(stream, "{value}");
                                    break;
                                }
                                Err(mpsc::RecvTimeoutError::Timeout)
                                    if !stopping.load(Ordering::Acquire) => {}
                                Err(_) => break,
                            }
                        }
                    }
                    Ok(_) => {}
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(25))
                    }
                    Err(_) => break,
                }
            }
        });
        let mut server = Server::new();
        server.controller = Some(
            Controller::new(
                Document::new("MCP bridge placeholder"),
                crate::document_limits(),
            )
            .map_err(|e| io::Error::other(e.to_string()))?,
        );
        Ok(Self {
            receiver,
            server,
            stop,
            registration,
        })
    }

    /// Called on the UI thread before drawing. Returns whether a command was applied.
    pub fn service(&mut self, controller: &mut Controller) -> bool {
        let mut changed = false;
        while let Ok(request) = self.receiver.try_recv() {
            let tool = request
                .value
                .pointer("/params/name")
                .and_then(Value::as_str)
                .unwrap_or("");
            changed |= matches!(
                tool,
                "om9_command"
                    | "om9_dispatch"
                    | "om9_point"
                    | "om9_select"
                    | "om9_undo"
                    | "om9_redo"
                    | "om9_save"
            );
            let response = self.server.handle_live(controller, &request.value);
            let _ = request.reply.send(response);
        }
        changed
    }
}
impl Drop for LiveHost {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        let _ = fs::remove_file(&self.registration);
    }
}

impl Server {
    fn handle_live(&mut self, controller: &mut Controller, request: &Value) -> Value {
        let id = request.get("id").cloned().unwrap_or(Value::Null);
        let tool = request
            .pointer("/params/name")
            .and_then(Value::as_str)
            .unwrap_or("");
        let file_command = (tool == "om9_command"
            && request
                .pointer("/params/arguments/text")
                .and_then(Value::as_str)
                .and_then(FileKind::from_command)
                .is_some())
            || (tool == "om9_dispatch"
                && request
                    .pointer("/params/arguments/feature")
                    .and_then(Value::as_str)
                    .and_then(FileKind::from_feature)
                    .is_some());
        if request.get("method").and_then(Value::as_str) == Some("tools/call")
            && (matches!(tool, "om9_new" | "om9_open" | "om9_ui") || file_command)
        {
            return json!({"jsonrpc":"2.0", "id":id, "result":{"content":[{"type":"text", "text":"Use OM9's window to create or open a document; use om9_save to save the live document."}], "isError":true}});
        }
        std::mem::swap(
            controller,
            self.controller.as_mut().expect("live placeholder"),
        );
        self.path = self
            .controller
            .as_ref()
            .and_then(|c| c.active_file_path().map(Path::to_path_buf));
        let response = if request.get("method").and_then(Value::as_str) == Some("tools/call")
            && tool == "om9_save"
        {
            let args = request
                .pointer("/params/arguments")
                .cloned()
                .unwrap_or_else(|| json!({}));
            let result = self.live_save(&args);
            let (text, error) = match result {
                Ok(v) => (v.to_string(), false),
                Err(e) => (e, true),
            };
            json!({"jsonrpc":"2.0", "id":id, "result":{"content":[{"type":"text", "text":text}], "isError":error}})
        } else {
            self.handle(request).unwrap_or(Value::Null)
        };
        std::mem::swap(
            controller,
            self.controller.as_mut().expect("live placeholder"),
        );
        response
    }

    fn live_save(&mut self, args: &Value) -> Result<Value, String> {
        let requested = args.get("path").and_then(Value::as_str).map(PathBuf::from);
        let controller = self.controller.as_mut().expect("live controller");
        if controller.file_operation().is_some() {
            return Err("finish the file operation already open in OM9 first".into());
        }
        let current = controller.active_file_path().map(Path::to_path_buf);
        let same = requested.is_none() || requested.as_ref() == current.as_ref();
        if same && current.is_none() {
            return Err("supply a new .om9 path for this unsaved document".into());
        }
        if !same && requested.as_ref().is_some_and(|p| p.exists()) {
            return Err(
                "destination already exists; use Save As in the OM9 window to confirm overwrite"
                    .into(),
            );
        }
        let result: Result<(), String> = (|| {
            let outcome = controller
                .begin_file_operation(if same {
                    FileKind::Save
                } else {
                    FileKind::SaveAs
                })
                .map_err(message)?;
            if outcome == FileOutcome::Saved {
                return Ok(());
            }
            let token = controller
                .file_operation()
                .ok_or("save operation disappeared")?
                .token();
            let path = requested.as_ref().ok_or("missing save path")?;
            let outcome = controller.accept_file_path(token, path).map_err(message)?;
            if outcome != FileOutcome::Saved {
                return Err("save needs confirmation in OM9".into());
            }
            Ok(())
        })();
        if result.is_err() {
            if let Some(op) = controller.file_operation() {
                let _ = controller.cancel_file_operation(op.token());
            }
        }
        result?;
        self.path = self
            .controller
            .as_ref()
            .and_then(|c| c.active_file_path().map(Path::to_path_buf));
        Ok(
            json!({"saved": self.path.as_ref().map(|p| p.display().to_string()), "state":self.snapshot(0)}),
        )
    }
}

fn exchange(address: SocketAddr, token: &str, request: &Value, probe: bool) -> io::Result<Value> {
    if !address.ip().is_loopback() {
        return Err(io::Error::other("invalid bridge address"));
    }
    let mut stream = TcpStream::connect_timeout(&address, IO_TIMEOUT)?;
    stream.set_read_timeout(Some(if probe {
        IO_TIMEOUT
    } else {
        Duration::from_secs(600)
    }))?;
    stream.set_write_timeout(Some(IO_TIMEOUT))?;
    writeln!(
        stream,
        "{}",
        json!({"token":token,"probe":probe,"request":request})
    )?;
    let mut response = String::new();
    BufReader::new(stream)
        .take(32 * 1024 * 1024)
        .read_line(&mut response)?;
    serde_json::from_str(&response).map_err(io::Error::other)
}

fn windows() -> io::Result<Vec<(SocketAddr, String)>> {
    let mut result = Vec::new();
    let dir = directory()?;
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(result),
        Err(e) => return Err(e),
    };
    for entry in entries.flatten().take(256) {
        if entry
            .metadata()
            .map_or(true, |m| !m.is_file() || m.len() > 4096)
        {
            continue;
        }
        let Ok(text) = fs::read_to_string(entry.path()) else {
            continue;
        };
        if text.len() > 4096 {
            continue;
        }
        let Ok(value) = serde_json::from_str::<Value>(&text) else {
            continue;
        };
        let Some(address) = value
            .get("address")
            .and_then(Value::as_str)
            .and_then(|s| s.parse::<SocketAddr>().ok())
        else {
            continue;
        };
        let Some(token) = value.get("token").and_then(Value::as_str) else {
            continue;
        };
        if exchange(address, token, &Value::Null, true).is_ok_and(|v| v["ready"] == true) {
            result.push((address, token.to_owned()));
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(id: u32, tool: &str, arguments: Value) -> Value {
        json!({"jsonrpc":"2.0", "id":id, "method":"tools/call", "params":{"name":tool,"arguments":arguments}})
    }
    fn content(response: &Value) -> Value {
        assert_ne!(response["result"]["isError"], true, "{response}");
        serde_json::from_str(response["result"]["content"][0]["text"].as_str().unwrap()).unwrap()
    }
    #[test]
    fn live_memory_observes_the_window_without_changing_pending_command() {
        let mut server = Server::new();
        server.controller = Some(Controller::new(Document::new("live placeholder"), crate::document_limits()).unwrap());
        let mut window = Controller::new(Document::new("live memory"), crate::document_limits()).unwrap();
        window.submit("Point").unwrap();
        let before = window.coordinator().snapshot();
        let prompt = window.prompt_with_options();
        let memory = content(&server.handle_live(&mut window,
            &request(1, "om9_memory", json!({"additional_pending_bytes":u64::MAX}))));
        assert_eq!(memory["request"]["fits_known_managed_inventory"], false);
        assert_eq!(memory["schema_version"], 1);
        assert_eq!(window.coordinator().snapshot(), before);
        assert_eq!(window.prompt_with_options(), prompt);
        assert!(window.is_running());
    }

    #[test]
    fn live_controller_uses_window_document_and_file_lifecycle() {
        let mut server = Server::new();
        server.controller =
            Some(Controller::new(Document::new("placeholder"), crate::document_limits()).unwrap());
        let mut window =
            Controller::new(Document::new("live test"), crate::document_limits()).unwrap();
        let state = content(&server.handle_live(&mut window, &request(1, "om9_state", json!({}))));
        assert_eq!(state["document"]["name"], "live test");
        let command = content(&server.handle_live(
            &mut window,
            &request(2, "om9_command", json!({"text":"Box 10 10 10"})),
        ));
        assert_eq!(command["result"]["ok"], true);
        assert_eq!(window.document().entities.len(), 1);
        let forbidden = server.handle_live(&mut window, &request(3, "om9_new", json!({})));
        assert_eq!(forbidden["result"]["isError"], true);
        assert_eq!(window.document().entities.len(), 1);
        content(&server.handle_live(&mut window, &request(4, "om9_undo", json!({}))));
        assert!(window.document().entities.is_empty());
        content(&server.handle_live(&mut window, &request(5, "om9_redo", json!({}))));
        assert_eq!(window.document().entities.len(), 1);
        let path = std::env::temp_dir().join(format!("om9-live-{}.om9", uuid::Uuid::new_v4()));
        let saved = content(
            &server.handle_live(&mut window, &request(6, "om9_save", json!({"path":path}))),
        );
        assert!(saved["saved"].is_string());
        assert!(!window.is_dirty().unwrap());
        assert_eq!(window.active_file_path(), Some(path.as_path()));
        window.begin_file_operation(FileKind::SaveAs).unwrap();
        let token = window.file_operation().unwrap().token();
        let busy = server.handle_live(&mut window, &request(7, "om9_save", json!({})));
        assert_eq!(busy["result"]["isError"], true);
        assert_eq!(window.file_operation().unwrap().token(), token);
        window.cancel_file_operation(token).unwrap();
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn authenticated_transport_reaches_ui_thread_and_unregisters() {
        let mut host = LiveHost::start(eframe::egui::Context::default()).unwrap();
        let registration = host.registration.clone();
        let data: Value =
            serde_json::from_str(&fs::read_to_string(&registration).unwrap()).unwrap();
        let address = data["address"].as_str().unwrap().parse().unwrap();
        let token = data["token"].as_str().unwrap().to_owned();
        assert!(exchange(address, "wrong token", &Value::Null, true).is_err());
        assert_eq!(
            exchange(address, &token, &Value::Null, true).unwrap()["ready"],
            true
        );
        let worker = std::thread::spawn(move || {
            exchange(
                address,
                &token,
                &request(9, "om9_command", json!({"text":"Box 2 3 4"})),
                false,
            )
            .unwrap()
        });
        let mut window =
            Controller::new(Document::new("transport"), crate::document_limits()).unwrap();
        for _ in 0..300 {
            if host.service(&mut window) {
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        let result = content(&worker.join().unwrap());
        assert_eq!(result["result"]["ok"], true);
        assert_eq!(window.document().entities.len(), 1);
        drop(host);
        assert!(!registration.exists());
    }
}

/// Codex launches this proxy over stdio; it never creates or replaces a GUI document.
pub fn serve_stdio() -> om9_core::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();
    for line in stdin.lock().lines() {
        let line = line.map_err(|e| om9_core::Error::Io(e.to_string()))?;
        if line.len() as u64 > MAX_LINE {
            return Err(om9_core::Error::Invalid("MCP request too large"));
        }
        let Ok(request) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        let Some(id) = request.get("id").cloned() else {
            continue;
        };
        let method = request.get("method").and_then(Value::as_str).unwrap_or("");
        let response = if method == "initialize" {
            json!({"jsonrpc":"2.0", "id":id, "result":{"protocolVersion":request.pointer("/params/protocolVersion").and_then(Value::as_str).unwrap_or(super::PROTOCOL_VERSION), "capabilities":{"tools":{}}, "serverInfo":{"name":"om9-live","version":env!("CARGO_PKG_VERSION")}, "instructions":"Controls the one currently open OM9 window. Use the window for New and Open. Changes use its normal Undo history."}})
        } else if method == "tools/list" {
            let mut list = tools::schema();
            list.as_array_mut()
                .expect("tool array")
                .retain(|t| !matches!(t["name"].as_str(), Some("om9_new" | "om9_open" | "om9_ui")));
            json!({"jsonrpc":"2.0", "id":id, "result":{"tools":list}})
        } else if method == "ping" {
            json!({"jsonrpc":"2.0", "id":id, "result":{}})
        } else {
            match windows() {
                Ok(found) if found.len() == 1 => exchange(found[0].0, &found[0].1, &request, false)
                    .unwrap_or_else(|e| json!({"jsonrpc":"2.0", "id":id, "error":{"code":-32000,"message":format!("OM9 connection failed: {e}")}})),
                Ok(found) => json!({"jsonrpc":"2.0", "id":id, "error":{"code":-32000,"message":if found.is_empty() {"No live OM9 window is connected. Start the new OM9 build."} else {"Multiple OM9 windows are connected. Leave only the intended window open."}}}),
                Err(e) => json!({"jsonrpc":"2.0", "id":id, "error":{"code":-32000,"message":e.to_string()}}),
            }
        };
        writeln!(stdout, "{response}").map_err(|e| om9_core::Error::Io(e.to_string()))?;
        stdout
            .flush()
            .map_err(|e| om9_core::Error::Io(e.to_string()))?;
    }
    Ok(())
}
