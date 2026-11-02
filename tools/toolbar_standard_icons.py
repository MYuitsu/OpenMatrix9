#!/usr/bin/env python3
"""Generate original Standard toolbar symbols from verified command names."""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'Resources/icons/toolbar-standard'
MARKS = {
    'Cut': '<circle cx="9" cy="9" r="4" fill="none" stroke="#d34642" stroke-width="2.5"/><circle cx="9" cy="23" r="4" fill="none" stroke="#d34642" stroke-width="2.5"/><path d="M12 11l14 12M12 21 26 8" fill="none" stroke="#eee" stroke-width="2.5"/>',
    'Copy': '<path d="M5 5h13l4 4v15H5z" fill="#f2f2f2" stroke="#444" stroke-width="2"/><path d="M11 10h13l4 4v13H11z" fill="#fafafa" stroke="#444" stroke-width="2"/>',
    'Paste': '<path d="M6 6h20v22H6z" fill="#e3b83b" stroke="#444" stroke-width="2"/><path d="M10 11h12v13H10z" fill="#f5f5f5"/><rect x="12" y="3" width="8" height="5" rx="1" fill="#eee" stroke="#444" stroke-width="1.5"/>',
    'Pan': '<path d="M10 16V7q0-3 3-3t3 3v8-5q0-3 3-3t3 3v6l1-2q1-2 3-1t1 4l-3 9H11L6 19q-2-3 0-4t4 1z" fill="#eee" stroke="#444" stroke-width="2"/>',
    'RotateView': '<path d="M6 16a10 10 0 1 1 7 10" fill="none" stroke="#eee" stroke-width="3"/><path d="M5 8 4 17l9-3" fill="#d7af43"/><path d="M12 16l4-5 5 5-5 5z" fill="#eee" stroke="#555"/>',
    'UndoView': '<path d="M6 14c2-7 12-10 19-4 3 3 3 8 1 11" fill="none" stroke="#eee" stroke-width="3"/><path d="M4 7v9h9z" fill="#d7af43"/><path d="M8 23h12" stroke="#aaa" stroke-width="2"/>',
    'FourViews': '<rect x="4" y="4" width="24" height="24" fill="#eee" stroke="#444" stroke-width="2"/><path d="M16 4v24M4 16h24" stroke="#444" stroke-width="2"/><path d="M5 5h10v10H5z" fill="#e3b83b"/>',
    'CPlaneOrigin': '<path d="M4 22 13 12l15 6-9 10z" fill="#ccc" stroke="#555" stroke-width="1.5"/><path d="M12 21h12m-12 0v-9" stroke="#d34642" stroke-width="2"/><circle cx="12" cy="21" r="2.5" fill="#e3b83b"/>',
    'Help': '<circle cx="16" cy="16" r="12" fill="#5a89cf" stroke="#eee" stroke-width="2"/><path d="M11 11q0-6 5-6t5 5q0 3-5 5v3m0 4v2" fill="none" stroke="#fff" stroke-width="3"/>',
    'DocumentProperties': '<path d="M5 4h17l5 5v19H5z" fill="#eee" stroke="#444" stroke-width="2"/><path d="M9 12h12M9 16h7" stroke="#555" stroke-width="2"/><circle cx="20" cy="21" r="4" fill="#e3b83b" stroke="#444"/>',
    'RightView': '<path d="M4 20h24v5H4zM7 18l3-7h12l4 7" fill="#d34642" stroke="#eee" stroke-width="1.5"/><circle cx="10" cy="25" r="2" fill="#444"/><circle cx="23" cy="25" r="2" fill="#444"/>',
    'ObjectSnaps': '<path d="M7 7v10a9 9 0 0 0 18 0V7h-6v10a3 3 0 0 1-6 0V7z" fill="#d34642" stroke="#eee" stroke-width="2"/><path d="M7 8h6m6 0h6" stroke="#eee" stroke-width="3"/>',
    'SelectAll': '<path d="M5 5h22v22H5z" fill="none" stroke="#eee" stroke-width="2" stroke-dasharray="4 2"/><path d="M10 9v14l4-4 3 7 3-1-3-7h6z" fill="#e3b83b" stroke="#444" stroke-width="1"/>',
    'Hide': '<path d="M8 12a8 8 0 1 1 16 0q0 5-5 8v5h-6v-5q-5-3-5-8z" fill="#e3b83b" stroke="#eee" stroke-width="2"/><path d="M5 27 27 5" stroke="#d34642" stroke-width="3"/>',
    'Lock': '<rect x="7" y="14" width="18" height="14" rx="2" fill="#e3b83b" stroke="#444" stroke-width="2"/><path d="M11 14V9a5 5 0 0 1 10 0v5" fill="none" stroke="#eee" stroke-width="3"/><circle cx="16" cy="20" r="2" fill="#444"/>',
    'Shade': '<circle cx="16" cy="16" r="11" fill="#a9a9a9" stroke="#eee" stroke-width="2"/><path d="M16 5a11 11 0 0 1 0 22z" fill="#555"/>',
    'Render': '<circle cx="16" cy="16" r="11" fill="#426ab8" stroke="#eee" stroke-width="2"/><path d="M9 12q3-5 8-4" fill="none" stroke="#fff" stroke-width="2.5"/>',
    'Spotlight': '<path d="M8 5 20 10 14 19 4 11z" fill="#ddd" stroke="#444" stroke-width="2"/><path d="M17 18 28 13 26 28 13 23z" fill="#e3b83b" fill-opacity="0.8"/>',
    'LinearDimension': '<path d="M5 8v17m22-17v17M5 21h22m-22 0 4-3m-4 3 4 3m18-3-4-3m4 3-4 3" fill="none" stroke="#eee" stroke-width="2"/><path d="M12 12h8" stroke="#e3b83b" stroke-width="3"/>',
}

def main():
    OUT.mkdir(parents=True, exist_ok=True)
    mp = ROOT / 'Resources/menu/source-manifest.json'
    manifest = json.loads(mp.read_text(encoding='utf-8'))
    for name, mark in MARKS.items():
        key = 'ToolbarStandard' + name
        path = OUT / (key + '.svg')
        path.write_text('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32">' + mark + '</svg>\n', encoding='utf-8', newline='\n')
        manifest['authored_symbols'][key] = {'symbol': key, 'label': name, 'style': 'toolbar-standard-v1', 'semantic_source': 'locally installed Matrix toolbar command labels', 'palette': ['#eeeeee', '#444444', '#d34642', '#e3b83b', '#5a89cf'], 'mapping': 'authored-command-symbol', 'sha256': hashlib.sha256(path.read_bytes()).hexdigest()}
    mp.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')

if __name__ == '__main__':
    main()
