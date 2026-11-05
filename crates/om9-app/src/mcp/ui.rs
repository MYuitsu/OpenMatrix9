// SPDX-License-Identifier: LGPL-2.1-or-later
//! MCP `om9_ui`: replay pointer/key events through the real widget tree.
use super::{NO_DOCUMENT, Server, message};
use serde_json::Value;

impl Server {
    /// Replays pointer/key events through the real `MatrixApp` widget tree and returns receipts:
    /// this is the hover/pick/orbit/pan/zoom/rubber-band/menu/focus path the headless geometry
    /// tools cannot see.
    pub(super) fn ui_replay(&mut self, args: &Value) -> Result<Value, String> {
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
}
