#!/usr/bin/env python3
"""Author visibility toolbar icons from the local Rhino command inventory."""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'Resources/icons/toolbar-visibility'
BULB = '<path d="M9 12a7 7 0 1 1 14 0q0 4-4 7v4h-6v-4q-4-3-4-7z" fill="{fill}" stroke="#eee" stroke-width="1.6"/><path d="M13 26h6" stroke="#eee" stroke-width="2"/>'
LOCK = '<rect x="9" y="14" width="14" height="13" rx="2" fill="#e3b83b" stroke="#eee" stroke-width="1.5"/><path d="{shackle}" fill="none" stroke="#eee" stroke-width="2.5"/>'
HIDE = BULB.format(fill='#777777')
SHOW = BULB.format(fill='#e3b83b')
LOCKED = LOCK.format(shackle='M12 14V9a4 4 0 0 1 8 0v5')
UNLOCK = LOCK.format(shackle='M12 14V9a4 4 0 0 1 8 0')
ARROW = '<path d="M3 24h7m-2-3 3 3-3 3" fill="none" stroke="#d34642" stroke-width="2"/>'
SELECT = '<path d="M23 4h6v6h-6z" fill="#d34642" stroke="#eee" stroke-width="1"/>'
INVERT = '<path d="M3 6h6v5H3zM23 23h6v5h-6z" fill="#d34642" stroke="#eee" stroke-width="1"/>'
SWAP = '<path d="M2 9h6l-2-2m22 16h-6l2 2" fill="none" stroke="#d34642" stroke-width="2"/>'
DOTS = '<circle cx="6" cy="8" r="2" fill="#eee"/><circle cx="26" cy="8" r="2" fill="#eee"/><circle cx="6" cy="24" r="2" fill="#eee"/><circle cx="26" cy="24" r="2" fill="#eee"/>'
CLIP = '<path d="M4 23 14 7l14 8-10 12z" fill="#ccc" stroke="#eee" stroke-width="1.5"/><path d="M8 8h15M8 12h10" stroke="#d34642" stroke-width="2"/>'
LETTERS = {
    'A': '<path d="M1 30 6 19l5 11M3 26h6" fill="none" stroke="#eee" stroke-width="2"/>',
    'B': '<path d="M1 19v11h6q5-1 0-5 5-3 0-6z" fill="none" stroke="#eee" stroke-width="2"/>',
    'C': '<path d="M10 20q-9-3-9 5t9 4" fill="none" stroke="#eee" stroke-width="2"/>',
}
MARKS = {
    'Hide': HIDE, 'Show': SHOW, 'ShowSelected': SHOW+SELECT, 'InvertHide': HIDE+INVERT,
    'HideSwap': HIDE+SWAP, 'Lock': LOCKED, 'Unlock': UNLOCK, 'UnlockSelected': UNLOCK+SELECT,
    'InvertLock': LOCKED+INVERT, 'LockSwap': LOCKED+SWAP, 'InvertHidePoints': HIDE+DOTS+INVERT,
    'HidePoints': HIDE+DOTS, 'ClippingPlane': CLIP, 'DisableClipping': CLIP+'<path d="M5 27 27 5" stroke="#d34642" stroke-width="3"/>',
    'HideA': HIDE+LETTERS['A'], 'HideB': HIDE+LETTERS['B'], 'HideC': HIDE+LETTERS['C'],
    'HideCustom': HIDE+ARROW,
}

def main():
    OUT.mkdir(parents=True, exist_ok=True)
    mp = ROOT / 'Resources/menu/source-manifest.json'
    manifest = json.loads(mp.read_text(encoding='utf-8'))
    for name, mark in MARKS.items():
        key = 'ToolbarVisibility' + name
        path = OUT / (key + '.svg')
        path.write_text('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32">' + mark + '</svg>\n', encoding='utf-8', newline='\n')
        manifest['authored_symbols'][key] = {'symbol': key, 'label': name, 'style': 'toolbar-visibility-v1', 'semantic_source': 'locally installed Matrix toolbar command labels and macros', 'palette': ['#eeeeee', '#777777', '#e3b83b', '#d34642'], 'mapping': 'authored-command-symbol', 'sha256': hashlib.sha256(path.read_bytes()).hexdigest()}
    mp.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')

if __name__ == '__main__':
    main()
