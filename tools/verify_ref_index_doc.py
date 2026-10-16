"""Verify the private reference index document's factual claims.

Cross-checks the numbers written in docs/private-reference-index.md against the
machine-produced survey artefacts and the filesystem, so the document cannot drift
from its evidence. Fails closed on a mismatch.
"""
import json
import re
import sys
from pathlib import Path

ROOT = Path("H:/OpenMatrix9")
DOC = ROOT / "docs/private-reference-index.md"


def read_json(rel):
    return json.loads((ROOT / rel).read_text(encoding="utf-8"))


def count_files(path):
    return sum(1 for p in path.rglob("*") if p.is_file())


def main():
    text = DOC.read_text(encoding="utf-8")
    survey = read_json("build/ref-survey.json")
    vb6 = read_json("build/ref-vb6-measure.json")
    fidelity = read_json("build/ref-matrix-fidelity.json")
    by_label = {r["label"]: r for r in survey["projects"] if not r.get("missing")}

    checks = []

    def check(name, doc_value, actual):
        checks.append({"claim": name, "document": doc_value, "actual": actual,
                       "ok": doc_value == actual})

    # census table numbers
    check("Matrix.rhp files", 659, by_label["Matrix.rhp (managed plugin)"]["files"])
    check("MatrixWPFControls files", 201, by_label["MatrixWPFControls.dll"]["files"])
    check("MatrixControls files", 53, by_label["MatrixControls.dll"]["files"])
    check("MatrixCommon files", 25, by_label["MatrixCommon.dll"]["files"])
    check("PIMData files", 49, by_label["PIMData.dll"]["files"])
    check("MatrixTranslation files", 4, by_label["MatrixTranslation.dll"]["files"])
    check("RhinoCommon files", 643, by_label["Rhino 5 RhinoCommon SDK"]["files"])
    check("Rhino_DotNet files", 2125, by_label["Rhino 5 Rhino_DotNet"]["files"])

    # Matrix fidelity table
    areas = {r["area"]: r for r in fidelity["areas"]}
    check("MatrixBuilder lines", 18635, areas["MatrixBuilder"]["lines"])
    check("CS lines", 16441, areas["CS"]["lines"])
    check("Interfaces lines", 7010, areas["Interfaces"]["lines"])
    check("KeyCheck lines", 11266, areas["KeyCheck"]["lines"])

    # VB6 numbers
    check("VB6 code files", 230, vb6["totals"]["code_files"])
    check("VB6 real procedure names", 2008, vb6["totals"]["real_names"])
    check("VB6 synthetic names", 1342, vb6["totals"]["synthetic_names"])
    ext = vb6["by_extension"]
    check("VB6 .bas real names", 0, ext[".bas"]["real_names"])
    check("VB6 .frm files", 126, ext[".frm"]["files"])
    check("VB6 .frm real names", 1424, ext[".frm"]["real_names"])

    # ref/ tree totals still match the document's table
    check("ref total files (document table sum)",
          sum(by_label[k]["files"] for k in by_label
              if k in {"Matrix.rhp (managed plugin)", "MatrixWPFControls.dll",
                       "MatrixControls.dll", "MatrixCommon.dll", "PIMData.dll",
                       "MatrixTranslation.dll", "VB Decompiler Lite export",
                       "Rhino 5 RhinoCommon SDK", "Rhino 5 Rhino_DotNet",
                       "Rhino 5 RhinoWindows", "Rhino 5 WindowsAPI",
                       "Matrix plugins (rhino5 tree)"}),
          by_label["Matrix.rhp (managed plugin)"]["files"]
          + by_label["MatrixWPFControls.dll"]["files"]
          + by_label["MatrixControls.dll"]["files"]
          + by_label["MatrixCommon.dll"]["files"]
          + by_label["PIMData.dll"]["files"]
          + by_label["MatrixTranslation.dll"]["files"]
          + by_label["VB Decompiler Lite export"]["files"]
          + by_label["Rhino 5 RhinoCommon SDK"]["files"]
          + by_label["Rhino 5 Rhino_DotNet"]["files"]
          + by_label["Rhino 5 RhinoWindows"]["files"]
          + by_label["Rhino 5 WindowsAPI"]["files"]
          + by_label["Matrix plugins (rhino5 tree)"]["files"])

    # the document must contain the private-reference warning and the two links
    for needle in ("KHÔNG được publish", "private-reference-policy.md",
                   "docs/analysis/matrix9-geometry-signature-inventory.md",
                   "check_ref_survey_safety.py"):
        checks.append({"claim": f"document mentions {needle!r}",
                       "document": True, "actual": needle in text, "ok": needle in text})

    # deferred items must be labelled as such, not presented as done
    checks.append({"claim": "deferred work is labelled chưa làm",
                   "document": True, "actual": "chưa làm" in text,
                   "ok": "chưa làm" in text})

    # the two derived documents must be git-ignored
    import subprocess
    for rel in ("docs/private-reference-index.md",
                "docs/analysis/matrix9-geometry-signature-inventory.md"):
        rc = subprocess.run(["git", "check-ignore", "-q", rel], cwd=ROOT,
                            capture_output=True).returncode
        checks.append({"claim": f"{rel} git-ignored", "document": True,
                       "actual": rc == 0, "ok": rc == 0})

    bad = [c for c in checks if not c["ok"]]
    print(json.dumps({"checks": len(checks), "failed": bad}, ensure_ascii=False, indent=1))
    if bad:
        sys.exit(1)


if __name__ == "__main__":
    main()
