"""Answer concretely: does ref/rhino5 give usable code patterns for the gem command set?

Two separate questions, measured separately:

A. Does Rhino_DotNet (RMA.OpenNURBS / RMA.Rhino) carry the openNURBS type surface that
   gem commands would sit on (Brep, NurbsCurve, NurbsSurface, Mesh, Xform, archive)?
B. Is there any *algorithm* in ref/rhino5 at all - i.e. do those files contain real
   method bodies that compute something, or only wrapper/dispatch bodies?

Read-only. Names, counts and structure only; no source bodies are copied.
"""
import json
import re
from collections import Counter
from pathlib import Path

ROOT = Path("H:/OpenMatrix9")
DOTNET = ROOT / "ref/rhino5/dnspy/Rhino_DotNet"
COMMON = ROOT / "ref/rhino5/dnspy/RhinoCommon"
OUT = ROOT / "build" / "ref-rhino5-gem-usability.json"

# types a gem-seat / prong / halo / pave command would actually need
GEM_RELEVANT = [
    "Brep", "NurbsCurve", "NurbsSurface", "Nurbs", "Mesh", "Curve", "Surface",
    "Xform", "Transform", "Point3d", "Vector3d", "Plane", "Intersection", "Intersect",
    "Trim", "Offset", "Solid", "MassProperties", "Area", "Volume", "ClosestPoint",
    "Boolean", "Fillet", "Extrusion", "RevSurface", "Sweep", "Loft", "Tolerance",
    "UnitSystem", "ObjectAttributes", "UserData",
]

CLASS_RE = re.compile(r"^\s*(?:public|internal)\s+(?:sealed\s+|abstract\s+|partial\s+|static\s+)*"
                      r"(?:class|struct|enum|interface)\s+([A-Za-z_][A-Za-z0-9_]*)")
METHOD_RE = re.compile(r"^\s*(?:public|internal|protected)\s+(?:static\s+|virtual\s+|override\s+|"
                       r"sealed\s+|new\s+|abstract\s+)*[A-Za-z0-9_<>,.\[\]?\s]+\s+"
                       r"([A-Za-z_][A-Za-z0-9_]*)\s*\(", re.M)
# a body that computes: arithmetic / control flow / geometry calls, not a stub
COMPUTE_RE = re.compile(r"\breturn\b|\bif\s*\(|\bfor\s*\(|\bwhile\s*\(|\bswitch\s*\(|"
                        r"\bnew\s+[A-Z]|\+=|-=|\*=|/=")
# a body that only forwards to native
FORWARD_RE = re.compile(r"UnsafeNativeMethods|__Internal|DllImport|Marshal\.")


def scan(base):
    rows = []
    for f in sorted(base.rglob("*.cs")):
        text = f.read_text(encoding="utf-8", errors="replace")
        classes = CLASS_RE.findall(text)
        methods = METHOD_RE.findall(text)
        compute = len(COMPUTE_RE.findall(text))
        forward = len(FORWARD_RE.findall(text))
        rows.append({
            "path": f.relative_to(base).as_posix(),
            "bytes": f.stat().st_size,
            "lines": text.count("\n"),
            "classes": classes,
            "methods": len(methods),
            "compute_signals": compute,
            "forward_signals": forward,
        })
    return rows


def main():
    payload = {"schema": "om9.ref_rhino5_gem_usability.v1"}

    dotnet = scan(DOTNET)
    common = scan(COMMON)

    def relevant(rows):
        hits = []
        for r in rows:
            joined = " ".join(r["classes"]) + " " + r["path"]
            matched = [k for k in GEM_RELEVANT if k.lower() in joined.lower()]
            if matched:
                hits.append({"path": r["path"], "classes": r["classes"][:6],
                             "matched": matched, "lines": r["lines"],
                             "compute_signals": r["compute_signals"],
                             "forward_signals": r["forward_signals"]})
        return hits

    payload["rhino_dotnet"] = {
        "files": len(dotnet),
        "lines": sum(r["lines"] for r in dotnet),
        "classes": sum(len(r["classes"]) for r in dotnet),
        "namespaces": dict(Counter(r["path"].split("/")[0] for r in dotnet).most_common()),
        "gem_relevant_files": relevant(dotnet),
    }
    payload["rhinocommon"] = {
        "files": len(common),
        "lines": sum(r["lines"] for r in common),
        "classes": sum(len(r["classes"]) for r in common),
        "gem_relevant_files": relevant(common),
    }
    # body character: compute vs forward, for the whole RhinoCommon tree
    totals = {
        "compute_signals": sum(r["compute_signals"] for r in common),
        "forward_signals": sum(r["forward_signals"] for r in common),
        "files_with_more_compute_than_forward":
            sum(1 for r in common if r["compute_signals"] > r["forward_signals"]),
        "files_with_more_forward_than_compute":
            sum(1 for r in common if r["forward_signals"] > r["compute_signals"]),
    }
    payload["rhinocommon_body_character"] = totals
    payload["rhino_dotnet_body_character"] = {
        "compute_signals": sum(r["compute_signals"] for r in dotnet),
        "forward_signals": sum(r["forward_signals"] for r in dotnet),
        "files_with_more_compute_than_forward":
            sum(1 for r in dotnet if r["compute_signals"] > r["forward_signals"]),
        "files_with_more_forward_than_compute":
            sum(1 for r in dotnet if r["forward_signals"] > r["compute_signals"]),
    }
    OUT.write_text(json.dumps(payload, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")

    for name in ("rhino_dotnet", "rhinocommon"):
        d = payload[name]
        print(f"=== {name}: {d['files']} files, {d['lines']} lines, {d['classes']} classes")
        if "namespaces" in d:
            print("   namespaces:", d["namespaces"])
        print(f"   gem-relevant files: {len(d['gem_relevant_files'])}")
        for h in d["gem_relevant_files"][:14]:
            print(f"     {h['lines']:6d}L compute={h['compute_signals']:5d} "
                  f"forward={h['forward_signals']:5d}  {h['path']}")
    print()
    print("=== body character (compute vs forward-to-native) ===")
    for k in ("rhinocommon_body_character", "rhino_dotnet_body_character"):
        print(f"  {k}: {payload[k]}")


if __name__ == "__main__":
    main()
