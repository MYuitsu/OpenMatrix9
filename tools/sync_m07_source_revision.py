"""Sync M07 `source_revision_candidate` to the value in its own overlay package.

The 69 M07 records carried `746a17f12789...` while the M07 overlay package that the
milestone is derived from pins `c116f8f224eb...`. Neither value is the workspace git
revision, so the field is a source-package stamp; the milestone record should carry
the same stamp as the package it came from. This sets it and reports every change so
the value is never altered silently.
"""
import json
import re
import sys
from pathlib import Path

ROOT = Path("H:/OpenMatrix9")
INCOMING = ROOT / "build/m07-incoming/OpenMatrix9/planning/M07"
MS = ROOT / "OM9_Milestones/M07 - Cutters"


def tokens(text: str):
    return dict(re.findall(r'"task_id"\s*:\s*"([^"]+)"[^}]*?"source_revision_candidate"\s*:\s*"([^"]+)"',
                           text, re.S))


def overlay_stamps():
    """task_id -> source_revision_candidate, read from the overlay package."""
    out = {}
    for p in INCOMING.rglob("l4.json"):
        try:
            d = json.loads(p.read_text(encoding="utf-8-sig"))
        except (OSError, json.JSONDecodeError):
            continue
        tid = d.get("task_id")
        val = d.get("source_revision_candidate")
        if tid and val:
            out[tid] = val
    return out


def main():
    stamps = overlay_stamps()
    print("overlay records with a stamp:", len(stamps))
    changed, missing, already = [], [], 0
    for p in sorted(MS.rglob("l4.json")):
        text = p.read_text(encoding="utf-8")
        d = json.loads(text)
        tid = d.get("task_id")
        want = stamps.get(tid)
        if not want:
            missing.append(tid)
            continue
        current = d.get("source_revision_candidate")
        if current == want:
            already += 1
            continue
        if current is None:
            # the field is absent in this record: do not invent it
            missing.append(tid)
            continue
        updated = text.replace(f'"source_revision_candidate": "{current}"',
                               f'"source_revision_candidate": "{want}"', 1)
        if updated == text:
            missing.append(tid)
            continue
        json.loads(updated)
        p.write_text(updated, encoding="utf-8")
        changed.append((tid, current, want))

    print("already correct:", already)
    print("changed:", len(changed))
    for tid, old, new in changed:
        print(f"  {tid}: {old[:12]}... -> {new[:12]}...")
    print("left alone (no stamp in the overlay or field absent):", len(missing))
    for tid in missing[:6]:
        print("   ", tid)
    if not changed and missing:
        sys.exit(1)


if __name__ == "__main__":
    main()
