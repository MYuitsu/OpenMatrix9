"""Per-subdirectory usability census of every decompiled-ui project.

Answers "does this subfolder contain usable Matrix code?" at the granularity of each
folder, for all six projects plus the native-inspection tree.

For every folder it measures:
  cs          number of .cs files
  lines       total lines
  real        files whose bodies look like real code (compute signals dominate)
  stub        files where decompiler placeholders dominate
  thin        files that are mostly properties/fields (little logic)
  dllimport   native calls (means the logic is elsewhere)
  vendor      hits for vendor/licence/telemetry/DB vocabulary
Names and counts only; no source bodies are copied.
"""
import json
import re
from collections import Counter
from pathlib import Path

ROOT = Path("H:/OpenMatrix9")
UI = ROOT / "ref/matrix/decompiled-ui"
OUT = ROOT / "build" / "ref-ui-subdir-census.json"

PROJECTS = [
    ("Matrix", "Matrix.rhp managed plugin"),
    ("MatrixWPFControls", "WPF controls and windows"),
    ("MatrixControls", "WinForms author-drawn controls"),
    ("MatrixCommon", "shared helpers"),
    ("PIMData", "jewellery data records and gem map"),
    ("MatrixTranslation", "i18n"),
    ("native-inspection", "extracted PE resources (no .cs)"),
    ("tools", "inspection scripts (no .cs)"),
]

PLACEHOLDER = re.compile(r"NotImplementedException|NotSupportedException")
COMPUTE = re.compile(r"\breturn\b|\bif\s*\(|\bfor\s*\(|\bwhile\s*\(|\bswitch\s*\(|\bnew\s+[A-Z]")
DLLIMPORT = re.compile(r"\[DllImport|UnsafeNativeMethods")
# a file that is mostly auto-properties carries little logic
PROPERTY = re.compile(r"\{\s*get;\s*(?:private\s+)?set;\s*\}")
VENDOR = re.compile(r"(?i)stuller|counter ?sketch|\bsku\b|pricing|licen[cs]e|sentinel|"
                    r"telemetry|optout|netfabb|update|account|credential|password|"
                    r"connectionstring|sql[a-z]*connection|entityconnection|\bvray\b|"
                    r"tspline|clayoo")
JEWELLERY = re.compile(r"(?i)\bgem|\bstone|prong|bezel|\bring\b|\brail\b|pave|halo|azure|"
                       r"milgrain|channel|carat|facet|girdle|cabochon|pearl|\bmetal\b")


def measure_folder(folder: Path, project_root: Path):
    cs = sorted(folder.glob("*.cs"))
    if not cs:
        return None
    stats = {"cs": 0, "lines": 0, "real": 0, "stub": 0, "thin": 0,
             "declared_stub": 0, "dllimport": 0, "vendor": 0, "jewellery": 0}
    for f in cs:
        text = f.read_text(encoding="utf-8", errors="replace")
        stats["cs"] += 1
        stats["lines"] += text.count("\n")
        ph = len(PLACEHOLDER.findall(text))
        compute = len(COMPUTE.findall(text))
        props = len(PROPERTY.findall(text))
        if ph:
            # any decompiler placeholder at all: reported separately so a folder with
            # a few stubbed members is not described as if it had none
            stats["declared_stub"] += 1
        if ph and ph >= compute:
            stats["stub"] += 1
        elif compute == 0 and props > 0:
            stats["thin"] += 1
        else:
            stats["real"] += 1
        if DLLIMPORT.search(text):
            stats["dllimport"] += 1
        if VENDOR.search(text):
            stats["vendor"] += 1
        if JEWELLERY.search(text):
            stats["jewellery"] += 1
    rel = folder.relative_to(project_root).as_posix()
    stats["folder"] = rel if rel != "." else "(root)"
    return stats


def main():
    payload = {"schema": "om9.ref_ui_subdir_census.v1", "projects": {}}
    for name, label in PROJECTS:
        base = UI / name
        if not base.is_dir():
            payload["projects"][name] = {"label": label, "missing": True}
            continue
        rows = []
        for folder in [base] + sorted(p for p in base.rglob("*") if p.is_dir()):
            row = measure_folder(folder, base)
            if row:
                rows.append(row)
        # keep the most substantial folders
        rows.sort(key=lambda r: -r["lines"])
        payload["projects"][name] = {"label": label, "folders": rows}

    OUT.write_text(json.dumps(payload, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")

    for name, data in payload["projects"].items():
        print("=" * 78)
        if data.get("missing"):
            print(f"{name}: MISSING")
            continue
        rows = data["folders"]
        tot = {k: sum(r[k] for r in rows) for k in
               ("cs", "lines", "real", "stub", "thin", "declared_stub",
                "dllimport", "vendor", "jewellery")}
        print(f"{name} — {data['label']}   ({len(rows)} folders)")
        print(f"   TOTAL cs={tot['cs']} lines={tot['lines']} real={tot['real']} "
              f"stub_dominant={tot['stub']} has_placeholder={tot['declared_stub']} "
              f"thin={tot['thin']} dllimport={tot['dllimport']} "
              f"vendor={tot['vendor']} jewellery={tot['jewellery']}")
        for r in rows[:14]:
            if r["lines"] < 40:
                continue
            print(f"   {r['lines']:7d}L cs={r['cs']:3d} real={r['real']:3d} "
                  f"ph={r['declared_stub']:3d} thin={r['thin']:3d} vendor={r['vendor']:3d} "
                  f"jew={r['jewellery']:3d}  {r['folder']}")
        print()


if __name__ == "__main__":
    main()
