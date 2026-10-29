#!/usr/bin/env python3
"""Generate independent SubD toolbar icons from command meanings."""
from pathlib import Path
import hashlib
import json

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'Resources/icons/toolbar-subd'
MARKS = {
    'EditMode': '<circle cx="16" cy="16" r="11" fill="#d3d3d3" stroke="#eee" stroke-width="2"/><path d="M7 18q8-13 18-3l-5 6z" fill="#c63b40"/>',
    'Primitives': '<path d="M5 12l9-6 9 5-9 6z" fill="#e47679" stroke="#eee" stroke-width="1.5"/><path d="M5 12v10l9 5V17zm9 5v10l9-5V11z" fill="#bd3439" stroke="#eee" stroke-width="1.5"/>',
    'Creation': '<path d="M5 24q2-15 10-17 11 0 12 17z" fill="#c43b40" stroke="#eee" stroke-width="2"/><path d="M8 20h16M11 15h10" stroke="#eee" stroke-width="1.4"/>',
    'Edition': '<path d="M5 22q3-16 13-15l8 5-3 12z" fill="#c43b40" stroke="#eee" stroke-width="2"/><path d="m8 23 15-15 3 3-15 15z" fill="#eee"/>',
    'Selection': '<path d="M6 7h20v18H6z" fill="none" stroke="#eee" stroke-width="2" stroke-dasharray="3 2"/><circle cx="16" cy="16" r="6" fill="#c43b40"/>',
    'Hotkeys': '<rect x="4" y="9" width="24" height="15" rx="2" fill="#ddd" stroke="#555" stroke-width="2"/><path d="M7 13h3m2 0h3m2 0h3m2 0h3M8 17h3m2 0h3m2 0h5M10 21h12" stroke="#bc3439" stroke-width="2"/>',
    'Settings': '<circle cx="16" cy="16" r="10" fill="#c43b40" stroke="#eee" stroke-width="2"/><circle cx="16" cy="16" r="4" fill="#eee"/><path d="M16 2v5m0 18v5M2 16h5m18 0h5" stroke="#eee" stroke-width="3"/>',
    'Help': '<circle cx="16" cy="16" r="12" fill="#ddd" stroke="#555" stroke-width="2"/><path d="M11 11q0-6 5-6t5 5q0 3-5 5v3m0 4v2" fill="none" stroke="#c43b40" stroke-width="3"/>',
    'Feedback': '<path d="M4 5h24v16H15l-7 6v-6H4z" fill="#ddd" stroke="#555" stroke-width="2"/><circle cx="10" cy="13" r="2" fill="#c43b40"/><circle cx="16" cy="13" r="2" fill="#c43b40"/><circle cx="22" cy="13" r="2" fill="#c43b40"/>',
    'Elements': '<circle cx="9" cy="10" r="4" fill="#c43b40"/><rect x="17" y="6" width="8" height="8" fill="#ddd" stroke="#555"/><path d="M6 26l5-9 5 9zm12-5 4-4 4 4-4 4z" fill="#c43b40"/>',
}

def main():
    OUT.mkdir(parents=True, exist_ok=True)
    mp = ROOT / 'Resources/menu/source-manifest.json'
    manifest = json.loads(mp.read_text(encoding='utf-8'))
    for name, mark in MARKS.items():
        key = 'ToolbarSubD' + name
        path = OUT / (key + '.svg')
        path.write_text('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32">' + mark + '</svg>\n', encoding='utf-8', newline='\n')
        manifest['authored_symbols'][key] = {'symbol': key, 'label': name, 'style': 'toolbar-subd-v1', 'semantic_source': 'locally installed Clayoo toolbar command labels', 'palette': ['#c43b40', '#eeeeee', '#555555'], 'mapping': 'authored-command-symbol', 'sha256': hashlib.sha256(path.read_bytes()).hexdigest()}
    mp.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')

if __name__ == '__main__':
    main()
