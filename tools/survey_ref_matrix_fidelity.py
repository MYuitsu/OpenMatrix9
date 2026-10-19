"""Fidelity check: do the domain areas of the Matrix decompile carry real bodies?

A decompile is only usable if the interesting files contain actual method bodies
rather than decompiler placeholders. This measures, per area, the share of .cs
files with a placeholder body and the share with a real return/statement body.
Read-only; no source text is copied.
"""
import json
import re
from collections import Counter
from pathlib import Path

ROOT = Path("H:/OpenMatrix9")
MATRIX = ROOT / "ref/matrix/decompiled-ui/Matrix"
OUT = ROOT / "build" / "ref-matrix-fidelity.json"

PLACEHOLDER = re.compile(r"NotImplementedException|NotSupportedException|"
                         r"throw new Exception\(\"The method or operation is not implemented")
BODY_SIGNAL = re.compile(r"\breturn\b|\bif\s*\(|\bfor\s*\(|\bwhile\s*\(|\bswitch\s*\(")

AREAS = ["MatrixBuilder", "CS", "MatrixData", "PIM", "UserData", "Interfaces",
         "Utilities", "Commands", "UserInterface", "SmartTargets", "KeyCheck",
         "VRay", "ZipIt", "Settings"]


def measure(base):
    rel = base.relative_to(MATRIX).as_posix()
    cs = [p for p in base.rglob("*.cs")]
    placeholder = 0
    real = 0
    lines = 0
    for f in cs:
        text = f.read_text(encoding="utf-8", errors="replace")
        lines += text.count("\n")
        n_ph = len(PLACEHOLDER.findall(text))
        n_body = len(BODY_SIGNAL.findall(text))
        if n_ph and n_ph > n_body:
            placeholder += 1
        elif n_body:
            real += 1
    return {"area": rel, "cs": len(cs), "lines": lines,
            "placeholder_dominant": placeholder, "with_real_body": real}


def main():
    rows = [measure(MATRIX / area) for area in AREAS if (MATRIX / area).is_dir()]
    payload = {"schema": "om9.ref_matrix_fidelity.v1", "areas": rows}
    OUT.write_text(json.dumps(payload, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")
    print(f"{'area':24s} {'cs':>5s} {'lines':>8s} {'real body':>10s} {'stub-dominant':>14s}")
    for r in rows:
        print(f"{r['area']:24s} {r['cs']:5d} {r['lines']:8d} {r['with_real_body']:10d} "
              f"{r['placeholder_dominant']:14d}")


if __name__ == "__main__":
    main()
