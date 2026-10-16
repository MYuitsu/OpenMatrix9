"""Extract the method inventory of the rough-jewellery core files.

Names and signatures only - no bodies. This is the input for writing OM9 algorithm
descriptions, and it doubles as evidence that an anchor points at a real operation.
"""
import json
import re
from pathlib import Path

ROOT = Path("H:/OpenMatrix9")
CORE = ROOT / "ref/matrix/decompiled-ui/Matrix"
OUT = ROOT / "build" / "rough-core-methods.json"

TARGETS = [
    "MatrixBuilder/MatrixBuilderBase.cs",
    "MatrixBuilder/MatrixBuilderCore.cs",
    "MatrixBuilder/MatrixGetPoint.cs",
    "MatrixBuilder/FourRailProfile/FourRailProfileBuilder.cs",
    "CS/CSUtils/CSUtilGeometry.cs",
    "CS/CSUtils/CSUtilSweep.cs",
    "CS/CSUtils/CSUtilRailRevolve.cs",
    "CS/CSUtils/CSUtilCurves.cs",
    "CS/CSGemOffsetCurves/GemOffsetCurve.cs",
    "CS/CSBlocks/CSCluster.cs",
    "CS/CSBlocks/CSProngPlacerGC.cs",
    "CS/CSBlocks/CSBezel.cs",
    "CS/CSBlocks/CSArray.cs",
    "MatrixBuilder/GemOnCurveMulti/GemOnCurveMultiBuilder.cs",
    "MatrixBuilder/GemOnCurveAdvanced/GemOnCurveAdvancedBuilder.cs",
    "MatrixBuilder/GemCountOnCurve/GemCountOnCurveBuilder.cs",
    "MatrixBuilder/GemListOnCurve/GemListOnCurveBuilder.cs",
    "MatrixBuilder/CSHalo/CSHaloBuilder.cs",
    "PIM/GemBlock.cs",
    "PIM/GemDefinition.cs",
]

DECL = re.compile(r"^\s*(?:public|internal|protected)\s+(?:static\s+|virtual\s+|override\s+|"
                  r"sealed\s+|new\s+|abstract\s+|async\s+)*"
                  r"([A-Za-z0-9_<>,.\[\]?]+(?:\s*<[^>]*>)?)\s+"
                  r"([A-Za-z_][A-Za-z0-9_]*)\s*\(([^)]*)\)", re.M)
CLASS = re.compile(r"^\s*(?:public|internal)\s+(?:sealed\s+|abstract\s+|partial\s+|static\s+)*"
                   r"(?:class|struct|enum)\s+([A-Za-z_][A-Za-z0-9_]*)"
                   r"(?:\s*:\s*([A-Za-z0-9_.,<> ]+))?", re.M)

# operation families that matter for the rough build chain
FAMILIES = {
    "builder-lifecycle": ("Update", "AddToDocument", "GetValues", "PostDraw", "Conduit",
                          "OnDynamicDraw", "Pick", "Commit", "Cancel", "Reset", "Init"),
    "sweep-loft": ("Sweep", "Loft", "Pipe", "Ruled", "ThruSections"),
    "rail-revolve": ("RailRevolve", "Revolve", "Rail"),
    "prong-dome-bezel": ("Prong", "Dome", "Bezel", "Basket", "Bead", "Head"),
    "gem-distribution": ("Gem", "Stone", "Spacing", "Count", "Distribute", "Place", "Layout",
                         "Array", "Sort", "Collision", "Overlap"),
    "curve-ops": ("Offset", "Trim", "Split", "Join", "Blend", "Project", "Divide", "Seam"),
    "brep-ops": ("Brep", "Face", "Trim", "Naked", "Cap", "Solid", "Boolean", "Join",
                 "Intersect", "Clean"),
    "transform": ("Xform", "Transform", "Orient", "Rotate", "Scale", "Mirror", "Translate"),
    "persistence": ("Read", "Write", "Serialize", "UserData", "Save", "Load"),
}


def main():
    payload = {"schema": "om9.rough_core_methods.v1", "files": {}}
    missing = []
    for rel in TARGETS:
        path = CORE / rel
        if not path.is_file():
            missing.append(rel)
            continue
        text = path.read_text(encoding="utf-8", errors="replace")
        classes = CLASS.findall(text)
        methods = [(ret.strip(), name, args.strip()) for ret, name, args in DECL.findall(text)]
        names = [m[1] for m in methods]
        grouped = {}
        for fam, keys in FAMILIES.items():
            hit = sorted({n for n in names if any(k.lower() in n.lower() for k in keys)})
            if hit:
                grouped[fam] = hit
        payload["files"][rel] = {
            "lines": text.count("\n"),
            "classes": [{"name": n, "base": (b or "").strip()} for n, b in classes],
            "methods": len(methods),
            "families": grouped,
        }
    payload["missing"] = missing
    OUT.write_text(json.dumps(payload, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")

    if missing:
        print("MISSING:", missing)
    for rel, d in payload["files"].items():
        bases = ", ".join(f"{c['name']}" + (f":{c['base']}" if c["base"] else "")
                          for c in d["classes"][:3])
        print(f"=== {rel}   ({d['lines']}L, {d['methods']} methods)")
        print(f"    types: {bases}")
        for fam, names in d["families"].items():
            shown = ", ".join(names[:10])
            more = f" (+{len(names)-10})" if len(names) > 10 else ""
            print(f"      {fam:20s} {shown}{more}")
        print()


if __name__ == "__main__":
    main()
