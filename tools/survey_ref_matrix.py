"""Per-directory survey of the Matrix.rhp decompile plus the Matrix90 ghidra listing.

Read-only. Produces counts, sizes and a domain-keyword census that the internal
index document quotes. No reference source text is copied.
"""
import json
import re
from collections import Counter
from pathlib import Path

ROOT = Path("H:/OpenMatrix9")
MATRIX = ROOT / "ref/matrix/decompiled-ui/Matrix"
GHIDRA = ROOT / "ref/matrix/ghidra/Matrix90.exe.c"
OUT = ROOT / "build" / "ref-matrix-survey.json"

DOMAIN_KEYWORDS = [
    "gem", "stone", "prong", "bezel", "ring", "cutter", "azure", "pave", "metal",
    "weight", "rail", "profile", "head", "channel", "bead", "halo", "cluster",
    "milgrain", "facet", "girdle", "carat", "density", "alloy",
]
# NURBS / solid geometry vocabulary that would matter for a kernel port
KERNEL_KEYWORDS = [
    "nurbs", "brep", "surface", "curve", "mesh", "trim", "boolean", "loft", "sweep",
    "revolve", "fillet", "offset", "intersect", "tessellat", "knot", "weight_",
    "controlpoint", "isocurve",
]


def survey_dirs(base):
    rows = []
    for d in sorted(p for p in base.rglob("*") if p.is_dir()):
        cs = [f for f in d.glob("*.cs")]
        if not cs:
            continue
        lines = 0
        for f in cs:
            lines += f.read_text(encoding="utf-8", errors="replace").count("\n")
        rows.append({
            "rel": d.relative_to(base).as_posix(),
            "cs": len(cs),
            "bytes": sum(f.stat().st_size for f in cs),
            "lines": lines,
        })
    return sorted(rows, key=lambda r: -r["lines"])


def keyword_census(base):
    counts = Counter()
    per_area = {}
    for f in base.rglob("*.cs"):
        rel = f.relative_to(base).as_posix()
        area = rel.split("/", 1)[0] if "/" in rel else "(root)"
        text = f.read_text(encoding="utf-8", errors="replace").lower()
        for kw in DOMAIN_KEYWORDS + KERNEL_KEYWORDS:
            n = text.count(kw)
            if n:
                counts[kw] += n
                per_area.setdefault(area, Counter())[kw] += n
    return counts, per_area


def survey_ghidra():
    if not GHIDRA.is_file():
        return {"missing": True}
    fn = re.compile(r"^[A-Za-z_][A-Za-z0-9_ \*]*\s+([A-Za-z_][A-Za-z0-9_]{3,})\s*\(", re.M)
    names = set()
    with GHIDRA.open("r", encoding="utf-8", errors="replace") as fh:
        for line in fh:
            m = fn.match(line)
            if m:
                names.add(m.group(1))
    interesting = sorted(n for n in names if any(
        k in n.lower() for k in ("gem", "ring", "cutter", "prong", "bezel", "rail",
                                 "pave", "azure", "metal", "boole", "nurbs", "brep")))
    return {
        "bytes": GHIDRA.stat().st_size,
        "named_functions": len(names),
        "domain_named_functions": len(interesting),
        "sample_domain_functions": interesting[:25],
        "sample_any_functions": sorted(names)[:10],
    }


def main():
    payload = {
        "schema": "om9.ref_matrix_survey.v1",
        "matrix_dirs": survey_dirs(MATRIX),
    }
    counts, per_area = keyword_census(MATRIX)
    payload["keyword_totals"] = dict(counts.most_common())
    payload["keyword_by_area"] = {
        area: dict(c.most_common(12)) for area, c in
        sorted(per_area.items(), key=lambda kv: -sum(kv[1].values()))[:14]
    }
    payload["ghidra_matrix90"] = survey_ghidra()
    OUT.write_text(json.dumps(payload, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")

    print("=== Matrix.rhp: top areas by C# lines ===")
    for row in payload["matrix_dirs"][:18]:
        print(f"  {row['lines']:7d} lines {row['cs']:4d} files  {row['rel']}")
    print()
    print("=== domain keyword totals (whole tree) ===")
    for kw, n in payload["keyword_totals"].items():
        print(f"  {kw:14s} {n:7d}")
    print()
    print("=== Matrix90.exe ghidra listing ===")
    g = payload["ghidra_matrix90"]
    print(f"  bytes={g.get('bytes'):,} named_functions={g.get('named_functions')} "
          f"domain_named={g.get('domain_named_functions')}")
    print("  sample domain names:", ", ".join(g.get("sample_domain_functions") or [])[:400])


if __name__ == "__main__":
    main()
