"""Classify every decompiled-ui folder into a usability tier.

Tiers (deliberately blunt, because "usable" must not be inflated):

  A_PORT_REFERENCE  real domain code whose behaviour OM9 must reproduce; the algorithm
                    can be re-derived from it, but the code is commercial so it is
                    described, never copied.
  B_NAME_REFERENCE  real code, but generic plumbing or UI mechanics; useful for naming
                    and structure, cheaper to rewrite than to adapt.
  C_ASSET_OR_GLUE   resources, designers, settings, generated files, or thin wrappers.
  D_DO_NOT_PORT     vendor integration, licence/activation, telemetry, DB, third-party
                    SDK, or components with no recoverable logic.

Driven by the measured census plus the vendor/jewellery signal, and overridable by an
explicit table for the folders already judged by reading.
"""
import json
from pathlib import Path

ROOT = Path("H:/OpenMatrix9")
CENSUS = json.loads((ROOT / "build" / "ref-ui-subdir-census.json").read_text(encoding="utf-8"))

# Judgements made by reading, not by keyword. Anything matching these prefixes is
# pinned to the tier given.
PINNED = [
    # third-party SDK, licence/activation, telemetry, DB, vendor catalogues, bundled
    # compression, and the decompiler-artifact-only folders
    ("D_DO_NOT_PORT", ["KeyCheck", "VRay", "Interfaces/VRayInterface", "UserInterface/TSplines",
                       "UserInterface/StullerSubmit", "Commands/Test",
                       "UserInterface/LibraryBrowser/StullerFindings",
                       "UserInterface/LibraryBrowser/Bermark", "ZipIt",
                       "MatrixBuilder/TSpline", "MatrixBuilder/TSplineBezel",
                       "MatrixBuilder/TSplineSignet", "MatrixBuilder/TSplineTubeFromCurves",
                       "MatrixBuilder/TSplineUnderBezel", "Utilities/Helpers/StullerPartsHelper",
                       "Utilities/Helpers/PricingSystemHelper", "Data", "UserInterface/EditText",
                       "Utilities/Converters"]),
    ("A_PORT_REFERENCE", ["CS/CSUtils", "CS/CSGemOffsetCurves", "CS/CSData", "CS/CSBlocks",
                          "CS/CSValue", "MatrixBuilder", "PIM",
                          "Utilities/GemDefinition", "Utilities/GemInstance", "Utilities/GemBlock",
                          "Settings", "UserInterface/MetalWeight", "Model", "ViewModel"]),
    ("B_NAME_REFERENCE", ["Utilities", "MatrixData", "Interfaces", "UserInterface/Display",
                          "UserInterface/Controls", "UserInterface/LayoutTool",
                          "UserInterface/LibraryBrowser", "Commands", "UserData", "SmartTargets",
                          "Helpers", "MatrixServiceReference", "Configuration",
                          "MatrixOptions", "Utilities/Behaviors", "Utilities/AttachedProperties",
                          "Utilities/Extensions", "View"]),
    ("C_ASSET_OR_GLUE", ["Properties", "rhino"]),
]


def tier_for(project, folder, row):
    for tier, prefixes in PINNED:
        for prefix in prefixes:
            if folder == prefix or folder.startswith(prefix + "/") or folder.startswith(prefix):
                return tier, f"pinned:{prefix}"
    if project in ("native-inspection", "tools"):
        return "C_ASSET_OR_GLUE", "no managed source"
    if row is None:
        return "C_ASSET_OR_GLUE", "no .cs"
    if row["cs"] and row["declared_stub"] == row["cs"]:
        return "C_ASSET_OR_GLUE", "every file a decompiler placeholder"
    if row["vendor"] and row["vendor"] >= max(1, row["cs"] // 2):
        return "D_DO_NOT_PORT", "vendor vocabulary dominant"
    if row["jewellery"] >= 3 and row["lines"] >= 800:
        return "A_PORT_REFERENCE", "jewellery domain signals + substantial"
    if row["thin"] * 2 >= row["cs"]:
        return "C_ASSET_OR_GLUE", "mostly properties/fields"
    if row["real"] >= 5 and row["lines"] >= 600:
        return "B_NAME_REFERENCE", "real code, no domain signal"
    return "C_ASSET_OR_GLUE", "small or thin"


def main():
    out = {"schema": "om9.ref_ui_tiers.v1", "projects": {}}
    for project, data in CENSUS["projects"].items():
        if data.get("missing"):
            continue
        rows = data.get("folders", [])
        by_folder = {r["folder"]: r for r in rows}
        # include folders with no .cs so nothing is silently missing
        base = ROOT / "ref/matrix/decompiled-ui" / project
        if base.is_dir():
            for d in sorted(p for p in base.rglob("*") if p.is_dir()):
                rel = d.relative_to(base).as_posix()
                by_folder.setdefault(rel, None)
        tiered = {}
        for folder, row in sorted(by_folder.items()):
            tier, why = tier_for(project, folder, row)
            tiered[folder] = {"tier": tier, "why": why,
                              "lines": (row or {}).get("lines", 0),
                              "cs": (row or {}).get("cs", 0),
                              "declared_stub": (row or {}).get("declared_stub", 0),
                              "vendor": (row or {}).get("vendor", 0),
                              "jewellery": (row or {}).get("jewellery", 0)}
        out["projects"][project] = tiered
    (ROOT / "build" / "ref-ui-tiers.json").write_text(
        json.dumps(out, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")

    for project, tiered in out["projects"].items():
        counts = {}
        for v in tiered.values():
            counts[v["tier"]] = counts.get(v["tier"], 0) + 1
        print(f"=== {project}  ({len(tiered)} folders)  {counts}")
        for tier in ("A_PORT_REFERENCE", "B_NAME_REFERENCE", "D_DO_NOT_PORT"):
            items = [(f, v) for f, v in tiered.items() if v["tier"] == tier]
            items.sort(key=lambda kv: -kv[1]["lines"])
            for folder, v in items:
                print(f"   {tier[0]} {v['lines']:7d}L cs={v['cs']:3d} ph={v['declared_stub']:3d} "
                      f"jew={v['jewellery']:3d} vend={v['vendor']:3d}  {folder}")
        print()


if __name__ == "__main__":
    main()
