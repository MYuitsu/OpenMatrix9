"""Mechanical survey of ref/ : per-project file counts, sizes, stub ratio.

Read-only. Writes build/ref-survey.json so the index document can quote numbers
that were actually measured. Only counts, paths and file names are recorded - no
reference source text.
"""
import json
import re
from collections import Counter
from pathlib import Path

ROOT = Path("H:/OpenMatrix9")
REF = ROOT / "ref"
OUT = ROOT / "build" / "ref-survey.json"

STUB_RE = re.compile(r"NotImplementedException|throw new NotSupportedException")
PROJECTS = [
    ("ref/matrix/decompiled-ui/Matrix", "Matrix.rhp (managed plugin)"),
    ("ref/matrix/decompiled-ui/MatrixWPFControls", "MatrixWPFControls.dll"),
    ("ref/matrix/decompiled-ui/MatrixControls", "MatrixControls.dll"),
    ("ref/matrix/decompiled-ui/MatrixCommon", "MatrixCommon.dll"),
    ("ref/matrix/decompiled-ui/PIMData", "PIMData.dll"),
    ("ref/matrix/decompiled-ui/MatrixTranslation", "MatrixTranslation.dll"),
    ("ref/matrix/Matrix", "Matrix (pre-dnSpy copy)"),
    ("ref/matrix/vb6-lite", "VB Decompiler Lite export"),
    ("ref/matrix/Matrix90", "native Matrix90 resources"),
    ("ref/rhino5/dnspy/RhinoCommon", "Rhino 5 RhinoCommon SDK"),
    ("ref/rhino5/dnspy/Rhino_DotNet", "Rhino 5 Rhino_DotNet"),
    ("ref/rhino5/dnspy/RhinoWindows", "Rhino 5 RhinoWindows"),
    ("ref/rhino5/dnspy/Rhino.WindowsAPI", "Rhino 5 WindowsAPI"),
    ("ref/rhino5/dnspy/Matrix_plugins", "Matrix plugins (rhino5 tree)"),
]


def survey(rel, label):
    base = ROOT / rel
    if not base.is_dir():
        return {"path": rel, "label": label, "missing": True}
    files = [p for p in base.rglob("*") if p.is_file()]
    by_ext = Counter(p.suffix.lower() for p in files)
    cs = [p for p in files if p.suffix.lower() == ".cs"]
    stub_files = 0
    cs_lines = 0
    for p in cs:
        try:
            text = p.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        cs_lines += text.count("\n")
        if STUB_RE.search(text):
            stub_files += 1
    # top-level layout
    top = sorted(p.name for p in base.iterdir())[:20]
    return {
        "path": rel,
        "label": label,
        "files": len(files),
        "bytes": sum(p.stat().st_size for p in files),
        "by_extension": dict(by_ext.most_common(12)),
        "cs_files": len(cs),
        "cs_lines": cs_lines,
        "cs_files_with_stub": stub_files,
        "top_level": top,
    }


def main():
    payload = {"schema": "om9.ref_survey.v1", "projects": []}
    for rel, label in PROJECTS:
        payload["projects"].append(survey(rel, label))
    # whole-tree totals
    allfiles = [p for p in REF.rglob("*") if p.is_file()]
    payload["totals"] = {
        "files": len(allfiles),
        "bytes": sum(p.stat().st_size for p in allfiles),
        "by_extension": dict(Counter(p.suffix.lower() for p in allfiles).most_common(15)),
    }
    OUT.write_text(json.dumps(payload, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")
    for row in payload["projects"]:
        if row.get("missing"):
            print(f"{row['label']:34s} MISSING")
            continue
        mb = row["bytes"] / 1024 / 1024
        print(f"{row['label']:34s} files={row['files']:6d} {mb:7.1f}MB cs={row['cs_files']:5d} "
              f"lines={row['cs_lines']:7d} stubbed={row['cs_files_with_stub']:4d}")
    print(f"{'TOTAL ref/':34s} files={payload['totals']['files']:6d} "
          f"{payload['totals']['bytes']/1024/1024:7.1f}MB")


if __name__ == "__main__":
    main()
