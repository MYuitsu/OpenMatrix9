#!/usr/bin/env python3
"""Author the Clayoo Sculpt toolbar symbols without copying reference pixels."""
from pathlib import Path
import hashlib
import json

OUT = Path(__file__).resolve().parents[1] / "Resources/icons/toolbar-sculpt"
COMMON = '<circle cx="16" cy="16" r="11" fill="#eeeeee" stroke="#3a3a3a" stroke-width="2"/>'
# Simple red marks retain the red/gray family of the supplied toolbar while
# distinguishing the operation at menu size. These are new vector shapes.
MARKS = {
    "Sculpt": '<path d="M7 14c4-9 13-8 18-3-3 1-4 3-5 6-3-1-6-2-8 3-2-2-3-4-5-6z" fill="#cf3439"/>',
    "Brush": '<path d="M8 20q5-9 12-10l3 3q-2 7-11 10z" fill="#cf3439"/><path d="M9 23l-3 3" stroke="#555" stroke-width="2"/>',
    "Crease": '<path d="M8 10q5 13 16 12M8 7q6 12 16 12" fill="none" stroke="#cf3439" stroke-width="3"/>',
    "Flatten": '<path d="M7 18h18v5H7z" fill="#cf3439"/><path d="M10 12h12" stroke="#cf3439" stroke-width="3"/>',
    "Inflate": '<path d="M7 20q9-17 18 0" fill="#cf3439"/><path d="M16 6v7m-3-4 3-3 3 3" fill="none" stroke="#444" stroke-width="1.8"/>',
    "Pinch": '<path d="M8 10l6 6-6 6M24 10l-6 6 6 6" fill="none" stroke="#cf3439" stroke-width="3"/>',
    "Smooth": '<path d="M7 19q4-5 9 0t9 0" fill="none" stroke="#cf3439" stroke-width="3"/><path d="M10 11h12" stroke="#aaa" stroke-width="2"/>',
    "Twist": '<path d="M8 16a8 8 0 1 1 5 7" fill="none" stroke="#cf3439" stroke-width="3"/><path d="M9 9l-2 6 6-1" fill="#cf3439"/>',
    "Drag": '<path d="M8 20q4-8 9-9l5 5q-2 6-10 7z" fill="#cf3439"/><path d="M18 7h7v7m0-7-7 7" fill="none" stroke="#444" stroke-width="2"/>',
}


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    manifest_path = OUT.parents[1] / "menu/source-manifest.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    for name, mark in MARKS.items():
        svg = (
            '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32">'
            + COMMON + mark + '</svg>\n'
        )
        path = OUT / f"ToolbarSculpt{name}.svg"
        path.write_text(svg, encoding="utf-8", newline="\n")
        key = f"ToolbarSculpt{name}"
        manifest["authored_symbols"][key] = {
            "symbol": key,
            "label": f"Sculpt {name}",
            "style": "toolbar-sculpt-v1",
            "semantic_source": "locally installed Clayoo toolbar command labels",
            "palette": ["#cf3439", "#eeeeee", "#3a3a3a"],
            "mapping": "authored-command-symbol",
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        }
    manifest_path.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
