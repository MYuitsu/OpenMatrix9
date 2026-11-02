#!/usr/bin/env python3
"""Generate independent, authored symbols for the six Clayoo Emboss buttons."""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "Resources/icons/toolbar-emboss"
MARKS = {
    "Emboss": '<path d="M3 19 16 11 29 19 16 27Z" fill="#b63439" stroke="#eee" stroke-width="1.5"/><path d="M9 16 16 7l7 9-7 5z" fill="#e26467" stroke="#fff" stroke-width="1.3"/>',
    "Resources": '<path d="M3 9h11l3 3h12v13H3z" fill="#b63439" stroke="#eee" stroke-width="2"/><path d="M8 16h4v4H8zm7 0h4v4h-4zm7 0h4v4h-4z" fill="#eee"/>',
    "TextureCreator": '<path d="M4 7h18v18H4z" fill="#b63439" stroke="#eee" stroke-width="2"/><path d="M7 10h4v4H7zm7 0h4v4h-4zM7 17h4v4H7zm7 0h4v4h-4z" fill="#eee"/><path d="m20 21 7-8 3 3-7 8-4 1z" fill="#ef8081" stroke="#eee" stroke-width="1"/>',
    "RasterToVector": '<path d="M3 8h4v4H3zm5 0h4v4H8zm-5 5h4v4H3zm5 5h4v4H8z" fill="#cf3439"/><path d="M13 16h6m-3-3 3 3-3 3" fill="none" stroke="#eee" stroke-width="2"/><path d="M20 23q1-15 9-13" fill="none" stroke="#cf3439" stroke-width="3"/><circle cx="20" cy="23" r="2" fill="#eee"/><circle cx="29" cy="10" r="2" fill="#eee"/>',
    "TextOnCurve": '<path d="M3 24q12-15 26-5" fill="none" stroke="#cf3439" stroke-width="3"/><path d="M8 17 13 5l5 12m-8-4h6M21 7h7m-3.5 0v9" fill="none" stroke="#eee" stroke-width="2.2" stroke-linecap="round"/>',
    "Decimator": '<path d="M3 23 8 7l8 4 7-5 6 17z" fill="#b63439" stroke="#eee" stroke-width="1.5"/><path d="M3 23 16 11l13 12M8 7l15-1-7 5v12" fill="none" stroke="#eee" stroke-width="1.2"/>',
}


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    manifest_path = ROOT / "Resources/menu/source-manifest.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    for name, mark in MARKS.items():
        key = f"ToolbarEmboss{name}"
        path = OUT / f"{key}.svg"
        path.write_text('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32">' + mark + '</svg>\n', encoding="utf-8", newline="\n")
        manifest["authored_symbols"][key] = {
            "symbol": key, "label": name, "style": "toolbar-emboss-v1",
            "semantic_source": "locally installed Clayoo toolbar command labels",
            "palette": ["#b63439", "#cf3439", "#eeeeee"],
            "mapping": "authored-command-symbol", "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        }
    manifest_path.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
