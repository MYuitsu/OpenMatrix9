#!/usr/bin/env python3
"""Crosswalk the normalized recovered Solid menu against standalone bindings.

An exact icon-name match is a source-level association, not runtime acceptance.
"""

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
inventory = json.loads((ROOT / "docs/analysis/matrix-command-layers.json").read_text(encoding="utf-8"))
solid = next(menu for menu in inventory["menus"] if menu["name"] == "Solid")
bindings = (ROOT / "crates/om9-app/src/solid_menu.rs").read_text(encoding="utf-8")
rows = re.findall(
    r'row\(\s*"(OM9-L3-03-18-\d+)",\s*"[^"]*",\s*"[^"]*",\s*"([^"]+)"',
    bindings,
)
owners = {icon: owner for owner, icon in rows}
if len(rows) != 21 or len(owners) != 21:
    raise SystemExit("Solid binding inventory changed; review extractor")

palette_text = (ROOT / "crates/om9-app/src/menu_catalog.rs").read_text(encoding="utf-8")
palette_block = palette_text.split("pub const SOLID: &[&str] = &[", 1)[1].split("];", 1)[0]
palette = set(re.findall(r'"([^"]+)"', palette_block))

print("# Recovered Solid menu crosswalk (source inventory only)\n")
print("Exact icon matches identify existing L3 catalog rows; they do not establish behavior, runtime, or alternate-mode coverage. Other entries require individual review. Palette presence is an exact icon-name match.\n")
print("| Group | Entry | Icon ID | Existing L3 | Palette |")
print("| --- | --- | --- | --- | --- |")
count = 0
matched = 0
for group in solid["commands"]:
    for option in group["options"]:
        icon = option["command_id"]
        owner = owners.get(icon)
        matched += bool(owner)
        count += 1
        cells = [group["name"], option["label"], icon, owner or "Review", "Yes" if icon in palette else "No"]
        print("| " + " | ".join(str(cell).replace("|", "\\|") for cell in cells) + " |")
print(f"\n{count} recovered entries; {matched} exact L3 icon matches; {len(palette)} standalone palette icons. Counts describe source records only. Solid Extrude All is in the palette/L3 catalog but not this recovered menu inventory.")
