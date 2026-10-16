"""Look directly at the Rhino_DotNet surfaces a gem command would use.

Rhino_DotNet is a C++/CLI-style assembly, so its type declarations do not match a
C# `public class` regex. This enumerates what is actually there: namespace folders,
the RMA.* type names, and for the gem-relevant files whether the bodies compute or
just forward. Names and structure only.
"""
import json
import re
from collections import Counter
from pathlib import Path

ROOT = Path("H:/OpenMatrix9")
DOTNET = ROOT / "ref/rhino5/dnspy/Rhino_DotNet"
OUT = ROOT / "build" / "ref-rhino-dotnet-surface.json"

# C++/CLI + C# declaration forms, plus the file-per-type convention this export uses
TYPE_RE = re.compile(r"^\s*(?:public|internal|private)\s+(?:ref\s+|value\s+)?"
                     r"(?:sealed\s+|abstract\s+|partial\s+|static\s+)*"
                     r"(?:class|struct|enum|interface)\s+([A-Za-z_][A-Za-z0-9_]*)", re.M)
NS_RE = re.compile(r"^\s*namespace\s+([A-Za-z0-9_.]+)", re.M)

FOCUS = {
    "Brep": ["OnBrep", "OnBrepFace", "OnBrepEdge", "OnBrepTrim", "OnBrepLoop", "OnBrepVertex"],
    "Nurbs": ["OnNurbsCurve", "OnNurbsSurface", "OnCurve", "OnSurface", "OnBezier"],
    "Mesh": ["OnMesh"],
    "Math": ["OnXform", "On3dPoint", "On3dVector", "OnPlane", "OnInterval"],
    "Intersect": ["OnIntersect", "OnX", "Intersection"],
    "Utility": ["OnUtil", "RhUtil", "RhinoUtil"],
    "MassProps": ["OnMassProperties", "Volume", "Area"],
    "Archive": ["OnBinaryArchive", "OnTextLog", "On3dm"],
}


def main():
    files = sorted(DOTNET.rglob("*.cs"))
    namespaces = Counter()
    type_names = []
    for f in files:
        text = f.read_text(encoding="utf-8", errors="replace")
        for ns in set(NS_RE.findall(text)):
            namespaces[ns] += 1
        type_names.extend(TYPE_RE.findall(text))

    payload = {
        "schema": "om9.ref_rhino_dotnet_surface.v1",
        "files": len(files),
        "namespaces": dict(namespaces.most_common(20)),
        "declared_types": len(set(type_names)),
        "sample_types": sorted(set(type_names))[:20],
        "focus": {},
    }

    for group, stems in FOCUS.items():
        rows = []
        for stem in stems:
            for f in files:
                if f.stem == stem or f.stem.startswith(stem):
                    text = f.read_text(encoding="utf-8", errors="replace")
                    rows.append({
                        "file": f.name,
                        "lines": text.count("\n"),
                        "declared": TYPE_RE.findall(text)[:4],
                        "ns": sorted(set(NS_RE.findall(text)))[:2],
                    })
                    break
        payload["focus"][group] = rows

    OUT.write_text(json.dumps(payload, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")

    print("namespaces:", payload["namespaces"])
    print("declared types:", payload["declared_types"])
    print("sample:", payload["sample_types"][:12])
    print()
    for group, rows in payload["focus"].items():
        print(f"== {group}")
        for r in rows:
            print(f"   {r['lines']:7d}L ns={r['ns']} decl={r['declared']}  {r['file']}")


if __name__ == "__main__":
    main()
