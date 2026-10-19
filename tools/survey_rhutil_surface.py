"""What does RhUtil (RMA.Rhino) actually offer, and does it touch jewellery?

9482 lines, the largest type in Rhino_DotNet. This lists its public method-name
families so the gem-command work can be judged against something concrete.
Names only.
"""
import json
import re
from collections import Counter
from pathlib import Path

ROOT = Path("H:/OpenMatrix9")
RUTIL = ROOT / "ref/rhino5/dnspy/Rhino_DotNet/Rhino_DotNet/RMA/Rhino/RhUtil.cs"
ONUTIL = ROOT / "ref/rhino5/dnspy/Rhino_DotNet/Rhino_DotNet/RMA/OpenNURBS/OnUtil.cs"
OUT = ROOT / "build" / "ref-rhutil-surface.json"

PUBLIC_METHOD = re.compile(r"^\s*public\s+(?:static\s+)?[A-Za-z0-9_<>,.\[\]?\s]+\s+"
                           r"([A-Za-z_][A-Za-z0-9_]*)\s*\(", re.M)
JEWELLERY = ["gem", "stone", "prong", "bezel", "ring", "cutter", "pave", "halo",
             "setting", "carat", "facet", "girdle", "milgrain", "channel", "rail"]
GEOM = ["Brep", "Nurbs", "Mesh", "Curve", "Surface", "Xform", "Transform", "Intersect",
        "Trim", "Offset", "Solid", "Volume", "Area", "Mass", "Closest", "Boolean",
        "Fillet", "Revolve", "Sweep", "Loft", "Extrude", "Cap", "Join", "Split"]


def families(names):
    out = Counter()
    for n in names:
        for key in GEOM:
            if key.lower() in n.lower():
                out[key] += 1
                break
    return out


def main():
    payload = {"schema": "om9.ref_rhutil_surface.v1"}
    for label, path in (("RhUtil", RUTIL), ("OnUtil", ONUTIL)):
        if not path.is_file():
            payload[label] = {"missing": True}
            continue
        text = path.read_text(encoding="utf-8", errors="replace")
        names = PUBLIC_METHOD.findall(text)
        low = text.lower()
        payload[label] = {
            "file": path.name,
            "lines": text.count("\n"),
            "public_methods": len(names),
            "distinct": len(set(names)),
            "jewellery_hits": {k: low.count(k) for k in JEWELLERY if low.count(k)},
            "geometry_families": dict(families(names).most_common()),
            "sample": sorted(set(names))[:30],
        }
    OUT.write_text(json.dumps(payload, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")
    for label in ("RhUtil", "OnUtil"):
        d = payload[label]
        print(f"=== {label}  ({d.get('lines')} lines, {d.get('public_methods')} public methods, "
              f"{d.get('distinct')} distinct)")
        print("   jewellery keyword hits:", d.get("jewellery_hits") or "(none)")
        print("   geometry families:", d.get("geometry_families"))
        print("   sample:", ", ".join(d.get("sample", [])[:18]))
        print()


if __name__ == "__main__":
    main()
