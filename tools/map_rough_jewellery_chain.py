"""Map the rough-jewellery build chain onto the Matrix files that serve it.

Excludes rendering entirely (V-Ray, render editor, animation). For each stage of the
bench-jeweller workflow it names the Matrix file(s) that carry the behaviour and the
OM9 milestone L3 records that own the work, so the algorithm extraction has a target
list rather than a guess.
"""
import json
import re
from pathlib import Path

ROOT = Path("H:/OpenMatrix9")
UI = ROOT / "ref/matrix/decompiled-ui"
OUT = ROOT / "build" / "rough-jewellery-chain.json"

# Stage -> (Matrix file globs, OM9 L3 prefixes, what the stage produces)
STAGES = [
    ("1. Vành và profile (ring rail / shank)", [
        ("Matrix", "MatrixBuilder/FourRailProfile/*.cs"),
        ("Matrix", "MatrixBuilder/*.cs"),
        ("Matrix", "CS/CSUtils/CSUtilSweep.cs"),
        ("Matrix", "CS/CSUtils/CSUtilRailRevolve.cs"),
        ("Matrix", "CS/CSUtils/CSUtilGeometry.cs"),
     ], ["OM9-L3-04-02"], "rail vành, profile, sweep 1/2 rail, rail revolve"),

    ("2. Thân và mặt (solid body)", [
        ("Matrix", "MatrixBuilder/CylinderBuilder/*.cs"),
        ("Matrix", "MatrixBuilder/UnderGallery/*.cs"),
        ("Matrix", "MatrixBuilder/Rope/*.cs"),
        ("Matrix", "MatrixBuilder/Panelling/*.cs"),
     ], ["OM9-L3-04-03"], "thân, dưới gallery, rope, panelling"),

    ("3. Đặt đá theo curve", [
        ("Matrix", "MatrixBuilder/GemOnCurveBasic/*.cs"),
        ("Matrix", "MatrixBuilder/GemOnCurveAdvanced/*.cs"),
        ("Matrix", "MatrixBuilder/GemOnCurveMulti/*.cs"),
        ("Matrix", "MatrixBuilder/GemCountOnCurve/*.cs"),
        ("Matrix", "MatrixBuilder/GemListOnCurve/*.cs"),
        ("Matrix", "CS/CSGemOffsetCurves/*.cs"),
     ], ["OM9-L3-04-10"], "phân bố đá dọc curve, offset curve theo gem"),

    ("4. Đặt đá theo surface / flow", [
        ("Matrix", "Commands/SmartFlow/*.cs"),
        ("Matrix", "SmartTargets/*.cs"),
        ("Matrix", "SmartTargets/MSR/*.cs"),
     ], ["OM9-L3-04-10"], "flow theo surface, splop, smart target"),

    ("5. Pave", [
        ("Matrix", "CS/CSBlocks/CSProngPlacerGC.cs"),
        ("Matrix", "CS/CSBlocks/CSSortGems.cs"),
        ("Matrix", "CS/CSBlocks/CSArray.cs"),
        ("Matrix", "CS/CSUtils/CSUtilGeometry.cs"),
     ], ["OM9-L3-04-12"], "rải đá pave, vùng, mật độ, azure"),

    ("6. Halo / cluster / channel", [
        ("Matrix", "MatrixBuilder/CSHalo/*.cs"),
        ("Matrix", "CS/CSBlocks/CSCluster.cs"),
        ("Matrix", "CS/CSBlocks/CSClusterMetalChannel.cs"),
        ("Matrix", "CS/CSBlocks/CSBezel.cs"),
     ], ["OM9-L3-04-11"], "halo, cluster, channel, bezel"),

    ("7. Prong / head / setting", [
        ("Matrix", "PIM/GemBlock.cs"),
        ("Matrix", "PIM/GemDefinition.cs"),
        ("Matrix", "PIM/GemInstance.cs"),
        ("Matrix", "Utilities/GemDefinition.cs"),
        ("Matrix", "UserInterface/LayerRender/*.cs"),
     ], ["OM9-L3-04-14"], "chấu, head, basket, bezel setting"),

    ("8. Cutter (trừ kim loại)", [
        ("Matrix", "MatrixBuilder/*Cutter*/*.cs"),
        ("Matrix", "MatrixBuilder/CylinderBuilder/*.cs"),
        ("Matrix", "CS/CSUtils/CSUtilGeometry.cs"),
     ], ["OM9-L3-04-17", "OM9-L3-04-18"], "cutter, boolean trừ, bright cut"),
]

EXCLUDE = re.compile(r"(?i)vray|render|animation|tspline|clayoo|magick|leadtools")


def main():
    payload = {"schema": "om9.rough_jewellery_chain.v1", "stages": []}
    for name, globs, l3_prefixes, produces in STAGES:
        present, lines_total = [], 0
        for project, pattern in globs:
            base = UI / project
            for f in sorted(base.glob(pattern)):
                if EXCLUDE.search(f.as_posix()):
                    continue
                n = f.read_text(encoding="utf-8", errors="replace").count("\n")
                lines_total += n
                present.append({"project": project,
                                "file": f.relative_to(base).as_posix(), "lines": n})
        payload["stages"].append({
            "stage": name, "produces": produces, "l3_prefixes": l3_prefixes,
            "files": present, "files_found": len(present), "lines": lines_total,
        })

    OUT.write_text(json.dumps(payload, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")

    for s in payload["stages"]:
        print(f"{s['stage']}   -> {s['files_found']} files, {s['lines']} lines")
        print(f"    {s['produces']}")
        print(f"    OM9 L3: {', '.join(s['l3_prefixes'])}")
        for f in s["files"]:
            print(f"      {f['lines']:5d}L  {f['project']}/{f['file']}")
        print()


if __name__ == "__main__":
    main()
