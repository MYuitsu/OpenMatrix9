"""Generate tools/matrix_manual_sheets.py: authored manual-sheet anchors.

Reads every authored reference sheet on disk (any milestone, any L3) plus the M03
crosswalk table, so the registry covers each sheet exactly once and the crosswalk
has one place to resolve a sheet id. The output is committed as authored support
data; re-running must be a no-op unless the milestone sheets change.

Usage: python tools/gen_manual_sheets.py
"""
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MILESTONES = ROOT / "OM9_Milestones"
TABLE = ROOT / "build" / "m03-crosswalk-table.json"
OUT = ROOT / "tools" / "matrix_manual_sheets.py"

MANUAL_LABELS = {
    "m8book1": ("Matrix 8 Instruction Manual Book 1", "book1"),
    "m8book2": ("Matrix 8 Instruction Manual Book 2", "book2"),
    "m9addendum": ("Matrix 9.0 Addendum Manual", "addendum"),
    "rhino5": ("Rhino 5 for Windows User's Guide", "rhino5"),
}


def parse_sheet(stem):
    """Return (manual, pages, section) for a sheet stem like m8book1__p138-140__circle."""
    parts = stem.split("__")
    if len(parts) < 3:
        return None
    prefix, pages, section = parts[0], parts[1], parts[2]
    manual = next((label for key, (label, _) in MANUAL_LABELS.items()
                   if prefix.startswith(key)), prefix)
    page_range = pages.lstrip("p").replace("-", "-")
    return manual, page_range, section.replace("-", " ")


def main():
    # Every authored sheet on disk, regardless of milestone or L3 tag.
    sheets = {}
    for path in sorted(MILESTONES.rglob("reference/text/*.md")):
        sheets.setdefault(path.stem, path.relative_to(ROOT).as_posix())
    for row in json.loads(TABLE.read_text(encoding="utf-8")):
        for sheet in row.get("sheets") or ([row["sheet"]] if row.get("sheet") else []):
            sheets.setdefault(sheet, None)

    entries = {}
    for sheet, locator in sheets.items():
        parsed = parse_sheet(sheet)
        if parsed is None:
            continue
        manual, pages, section = parsed
        anchor = "mx-sheet-" + re.sub(r"[^a-z0-9]+", "-", sheet.lower()).strip("-")
        entries.setdefault(anchor, {
            "manual": manual,
            "pages": pages,
            "sheet": sheet,
            "section": section.title(),
        })
    body = ["\"\"\"Authored Matrix manual-sheet anchors (generated).",
            "",
            "Each entry points at one reference sheet shipped in OM9_Milestones/**/reference/text.",
            "Regenerate with tools/gen_manual_sheets.py; the crosswalk validator fails closed if an",
            "anchor here no longer resolves to a sheet.",
            "\"\"\"",
            "",
            "MANUAL_SHEETS = {"]
    for anchor in sorted(entries):
        item = entries[anchor]
        body.append(f"    {anchor!r}: dict(")
        body.append(f"        manual={item['manual']!r}, pages={item['pages']!r},")
        body.append(f"        sheet={item['sheet']!r}, section={item['section']!r},")
        body.append("        topic='authored Matrix manual section'),")
    body.append("}")
    body.append("")
    OUT.write_text("\n".join(body), encoding="utf-8")
    print(json.dumps({"anchors": len(entries)}))


if __name__ == "__main__":
    main()
