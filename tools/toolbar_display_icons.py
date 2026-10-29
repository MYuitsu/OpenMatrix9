#!/usr/bin/env python3
"""Generate authored Display toolbar icons from Rhino command meanings."""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'Resources/icons/toolbar-display'
MARKS = {
    'Rendered': '<circle cx="16" cy="16" r="11" fill="#426ab8" stroke="#eee" stroke-width="2"/><path d="M9 13q3-5 8-5" fill="none" stroke="#fff" stroke-width="2.5"/>',
    'XRay': '<circle cx="16" cy="16" r="11" fill="#72b3d4" fill-opacity="0.45" stroke="#eee" stroke-width="2"/><path d="M8 10l16 12M8 22 24 10" stroke="#fff" stroke-width="2"/>',
    'Pen': '<circle cx="16" cy="16" r="11" fill="none" stroke="#eee" stroke-width="2"/><path d="m8 24 14-15 3 3-14 15-4 1z" fill="#d7af43" stroke="#444" stroke-width="1"/>',
    'Artistic': '<circle cx="16" cy="16" r="11" fill="#e3b83b" stroke="#eee" stroke-width="2"/><path d="M7 18q5-7 9 0t9-1" fill="none" stroke="#d34642" stroke-width="3"/>',
    'MeshSettings': '<path d="M4 25 10 5l17 18z" fill="#83a7c5" stroke="#eee" stroke-width="2"/><path d="M4 25 20 13M10 5l6 20" fill="none" stroke="#444" stroke-width="1.5"/><circle cx="25" cy="24" r="4" fill="#e3b83b" stroke="#444"/>',
    'FlatShade': '<path d="M4 24 12 5l16 19z" fill="#a0a0a0" stroke="#eee" stroke-width="2"/><path d="M12 5 16 24h12z" fill="#555"/>',
    'ObjectShading': '<circle cx="16" cy="16" r="11" fill="#426ab8" stroke="#eee" stroke-width="2"/><path d="M3 3h7v7H3zm19 19h7v7h-7z" fill="none" stroke="#e3b83b" stroke-width="2"/>',
    'DisplayPanel': '<rect x="3" y="5" width="26" height="21" rx="2" fill="#777" stroke="#eee" stroke-width="2"/><path d="M3 10h26M10 10v16" stroke="#eee" stroke-width="2"/><path d="M14 15h11m-11 4h8" stroke="#e3b83b" stroke-width="2"/>',
}

def main():
    OUT.mkdir(parents=True, exist_ok=True)
    mp = ROOT / 'Resources/menu/source-manifest.json'
    manifest = json.loads(mp.read_text(encoding='utf-8'))
    for name, mark in MARKS.items():
        key = 'ToolbarDisplay' + name
        path = OUT / (key + '.svg')
        path.write_text('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32">' + mark + '</svg>\n', encoding='utf-8', newline='\n')
        manifest['authored_symbols'][key] = {'symbol': key, 'label': name, 'style': 'toolbar-display-v1', 'semantic_source': 'locally installed Matrix toolbar command labels and macros', 'palette': ['#eeeeee', '#426ab8', '#e3b83b', '#d34642'], 'mapping': 'authored-command-symbol', 'sha256': hashlib.sha256(path.read_bytes()).hexdigest()}
    mp.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')

if __name__ == '__main__':
    main()
