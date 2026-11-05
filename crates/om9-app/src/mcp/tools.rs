// SPDX-License-Identifier: LGPL-2.1-or-later
//! MCP tool catalog advertised by `tools/list`.
use serde_json::{Value, json};

pub(super) fn schema() -> Value {
    json!([
        {
            "name": "om9_new",
            "description": "Start an empty OM9 document. benchmark_no_undo is for isolated benchmarks only and disables Undo for the entire new session.",
            "inputSchema": {"type": "object", "properties": {"name": {"type": "string"}, "benchmark_no_undo": {"type": "boolean", "default": false}}}
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
            "name": "om9_memory",
            "description": "Read managed RAM inventory, pending/allocated external classes, cached Undo inventory and independent physical/Windows commit samples. Includes the last final external-reservation refusal (historical operands, source coordinator and age), which is distinct from the fresh samples. Read-only, on demand; excludes unmeasured native/RSS/GPU owners. Optional additional_pending_bytes only assesses a new pending allocation against known observations; it does not reserve/admit an operation or certify accuracy.",
            "inputSchema": {
                "type": "object", "additionalProperties": false,
                "properties": {"additional_pending_bytes": {"type": "integer", "minimum": 0}}
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
            "name": "om9_export_stl",
            "description": "Export the selected meshes and editable B-Rep solids to one binary STL in millimetres (File > Export Selected). The destination must end in .stl.",
            "inputSchema": {
                "type": "object",
                "properties": {"path": {"type": "string"}, "linear_tolerance_mm": {"type": "number", "exclusiveMinimum": 0, "description": "Optional requested linear tolerance in millimetres; defaults to document tolerance."}, "angular_tolerance_rad": {"type": "number", "exclusiveMinimum": 0, "maximum": std::f64::consts::PI, "description": "Optional OCCT angular deflection in radians; defaults to 0.3."}},
                "required": ["path"]
            }
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
