"""Run the full reference-anchor verification sweep and summarise pass/fail.

Every check exits non-zero on drift, so this converts the suite into one report.
"""
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path("H:/OpenMatrix9")
PY = sys.executable

CHECKS = [
    ("crosswalk", [PY, "tools/validate_matrix_ref_crosswalk.py"]),
    ("rhino5 anchors", [PY, "tools/validate_rhino5_anchors.py"]),
    ("supplements applied", [PY, "tools/validate_matrix_ref_supplements_applied.py"]),
    ("implementation samples", [PY, "tools/validate_implementation_samples.py"]),
    ("runbook strip regression", [PY, "tools/test_runbook_section_strip.py"]),
    ("survey safety", [PY, "tools/check_ref_survey_safety.py"]),
    ("index doc facts", [PY, "tools/verify_ref_index_doc.py"]),
    ("merge m03", [PY, "tools/verify_merge.py", "m03"]),
    ("merge m04", [PY, "tools/verify_merge.py", "m04"]),
    ("overlay m05", [PY, "tools/verify_l4_overlay_merge.py", "m05"]),
    ("overlay m06", [PY, "tools/verify_l4_overlay_merge.py", "m06"]),
    ("overlay m07", [PY, "tools/verify_l4_overlay_merge.py", "m07"]),
]

rows = []
for name, cmd in CHECKS:
    proc = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True)
    detail = ""
    out = proc.stdout or ""
    start = out.find("{")
    if start >= 0:
        try:
            data = json.loads(out[start:])
            for key in ("errors", "problems", "hard_errors", "failed"):
                if key in data:
                    detail = f"{key}={len(data[key])}"
                    break
            for key in ("checked", "per_milestone"):
                if not detail and key in data:
                    detail = f"{key}={data[key]}"
        except json.JSONDecodeError:
            pass
    rows.append({"check": name, "exit": proc.returncode, "detail": detail,
                 "ok": proc.returncode == 0})

failed = [r for r in rows if not r["ok"]]
for r in rows:
    print(f"  {'PASS' if r['ok'] else 'FAIL'}  {r['check']:28s} exit={r['exit']}  {r['detail']}")
print()
print(json.dumps({"checks": len(rows), "failed": failed}, ensure_ascii=False, indent=1))
if failed:
    sys.exit(1)
print("ALL CHECKS PASS")
