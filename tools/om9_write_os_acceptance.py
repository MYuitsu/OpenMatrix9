import hashlib
import json
import os
import platform

out = os.path.abspath(r"build\gumball-os-audit")
with open(os.path.join(out, "ready.json"), encoding="utf-8") as handle:
    ready = json.load(handle)
with open(os.path.join(out, "result.json"), encoding="utf-8") as handle:
    result = json.load(handle)
with open(os.path.join(out, "done.json"), encoding="utf-8") as handle:
    done = json.load(handle)
with open(os.path.join(out, "input-diagnostic.json"), encoding="utf-8") as handle:
    diagnostic = json.load(handle)
with open(os.path.join(out, "state.json"), encoding="utf-8") as handle:
    state = json.load(handle)
with open(os.path.join(out, "gumball-os-audit.png"), "rb") as handle:
    screenshot = hashlib.sha256(handle.read()).hexdigest()
with open(r"L:\om9-target\debug\om9.exe", "rb") as handle:
    exe = hashlib.sha256(handle.read()).hexdigest()

record = {
    "schema": "om9-gumball-os-gpu-acceptance/v1",
    "date": "2026-11-01",
    "command": r"powershell -NoProfile -ExecutionPolicy Bypass -File tools\gumball-os-audit.ps1",
    "environment": {
        "platform": platform.platform(),
        "window": "real eframe/wgpu window (1600x1000 client)",
        "input": "Win32 SetCursorPos + mouse_event after AttachThreadInput foreground handoff",
        "binary_sha256": exe,
    },
    "fixture": {
        "line": [[0.0, 0.0, 0.0], [10.0, 0.0, 0.0]],
        "snap_point": [5.0, 5.0, 0.0],
        "gumball": "Snappy, free Move handle",
    },
    "receipts": {
        "ready": ready,
        "input": diagnostic,
        "result": result,
        "state": state,
        "done": done,
    },
    "screenshot": {
        "path": "build/gumball-os-audit/gumball-os-audit.png",
        "sha256": screenshot,
        "note": "App-side egui ViewportCommand::Screenshot, real GPU framebuffer; kept in build/ because docs raster assets require review.",
    },
    "checks": {
        "drag_started": bool(result.get("drag_started")),
        "line_moved": bool(result.get("moved")),
        "landed_on_snap_point": result.get("after") == [[0.0, 5.0, 0.0], [10.0, 5.0, 0.0]],
        "screenshot_saved": os.path.exists(os.path.join(out, "gumball-os-audit.png")),
        "process_completed": done.get("status") == "ok",
        "app_focused": bool(state.get("focused")),
        "gumball_session_committed": bool(state.get("dragging")) and not state.get("session"),
    },
}
record["all_passed"] = all(record["checks"].values())
path = r"docs/validation/2026-11-01-gumball-os-gpu.json"
with open(path, "w", encoding="utf-8", newline="\n") as handle:
    json.dump(record, handle, indent=2)
    handle.write("\n")
print(path, record["all_passed"], record["checks"])
