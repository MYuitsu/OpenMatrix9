"""Per-file usability classification for every decompiled-ui project.

Folder-level tiers hide mixed folders (a ViewModel folder can hold both a portable
project-manager view model and a vendor pricing view model). This classifies each
.cs file, which is the honest granularity for "is this file usable?".

Tier rules, in order:
  D  vendor/licence/telemetry/DB/third-party-SDK vocabulary, or a decompiler stub
  C  designer/settings/generated/resource file, or almost no logic
  A  carries jewellery/geometry domain signals and real bodies
  B  real bodies, generic plumbing or UI mechanics
"""
import json
import re
from collections import Counter
from pathlib import Path

ROOT = Path("H:/OpenMatrix9")
UI = ROOT / "ref/matrix/decompiled-ui"
OUT = ROOT / "build" / "ref-ui-file-tiers.json"

PLACEHOLDER = re.compile(r"NotImplementedException|NotSupportedException")
COMPUTE = re.compile(r"\breturn\b|\bif\s*\(|\bfor\s*\(|\bwhile\s*\(|\bswitch\s*\(|\bnew\s+[A-Z]")
PROPERTY = re.compile(r"\{\s*get;\s*(?:private\s+)?set;\s*\}")
VENDOR = re.compile(r"(?i)stuller|counter ?sketch|\bsku\b|pricing|licen[cs]e|sentinel|"
                    r"telemetry|optout|netfabb|\bupdater?\b|account|credential|password|"
                    r"connectionstring|sql[a-z]*connection|entityconnection|\bvray\b|tspline|"
                    r"clayoo|\bbermark\b|lead ?tools|magick")
JEWELLERY = re.compile(r"(?i)\bgem|\bstone|prong|bezel|\bring\b|\brail\b|pave|halo|azure|"
                       r"milgrain|channel|carat|facet|girdle|cabochon|pearl|\bmetal\b|"
                       r"shank|signet|findings")
GEOMETRY = re.compile(r"(?i)\bbrep|nurbs|\bmesh\b|surface|curve|loft|sweep|revolve|offset|"
                      r"intersect|boolean|fillet|tolerance|xform|transform")

# Files judged by READING them, which override the keyword heuristic. The heuristic
# over-promotes files whose identifiers merely contain "channel" (a WCF channel), or
# vendor SDK wrappers that also mention geometry.
PINNED_D = {
    # third-party render SDK
    "VRay/VRayVisMat.cs", "VRay/VRayObject.cs", "Interfaces/VRayInterface.cs",
    "VRay/VRayMaterial.cs", "VRay/VRayUtils.cs",
    # T-Splines third-party SDK
    "Interfaces/TSplinesInterface.cs", "Interfaces/TSplinesObject.cs", "TSplineStore.cs",
    "MatrixBuilder/TSplineSignet/TSplineSignetBuilder.cs",
    "MatrixBuilder/TSplineBezel/TSplineBezelBuilder.cs",
    "MatrixBuilder/TSplineTubeFromCurves/TSplineTubeFromCurvesBuilder.cs",
    "MatrixBuilder/TSplineUnderBezel/TSplineUnderBezelBuilder.cs",
    # licence / activation
    "KeyCheck/SentinelKeys.cs", "KeyCheck/SentinelKey.cs", "KeyCheck/SentinelKeysLicense.cs",
    # vendor accounts, catalogues, pricing
    "UserInterface/StullerSubmit/StullerSubmit.cs",
    "UserInterface/StullerSubmit/StullerSubmitHelper.cs",
    "StullerPartsHelper.cs",
    "ViewModel/PricingReportViewModel.cs", "Model/StoneGrouping.cs", "Model/MetalGrouping.cs",
    "Model/StockPartGrouping.cs", "Data/DAL.cs", "Helpers/PricingSystemHelper.cs",
    # WCF update service (the "Channel" in these names is a WCF channel, not geometry)
    "MatrixServiceReference/Version9.cs", "MatrixServiceReference/IUpdateService9.cs",
    "MatrixServiceReference/IUpdateService9Channel.cs",
    "MatrixServiceReference/UpdateService9Client.cs",
    "Helpers/UpdateHelper.cs", "Helpers/DataProtectionHelper.cs", "Helpers/RegistryHelper.cs",
    "Helpers/EventHelper.cs", "Helpers/OperatingSystemHelper.cs", "Helpers/ConnectivityHelper.cs",
    "Helpers/ProcessInfo.cs", "Helpers/SplashEventHelper.cs", "Helpers/ActiveWindowHelper.cs",
    "Helpers/ProgressBarEventArgs.cs",
    # telemetry / remote access
    "Commands/GVOptOut.cs", "Commands/gvRemoteAssistance.cs",
    "Commands/ResetStullerAccountLoginCommand.cs",
    # bundled compression with no upstream notice in-tree
    "ZipIt/ZipIt.cs", "ZipIt/Deflater.cs", "ZipIt/Inflater.cs",
    # Stuller PIM database layer (SQL Server, and one file embeds a plaintext
    # connection string with credentials - never read that value into a deliverable)
    "Gemvision/Stuller/PIMData/PIMDataAccess.cs",
    "Gemvision/Stuller/PIMData/PIMDataLoadFromDB.cs",
    "Gemvision/Stuller/PIMData/PIMDataSaveToDB.cs",
    "Gemvision/Stuller/PIMData/PIMDataXML.cs",
}


# Generic helpers that the keyword heuristic would over-promote (a single mention of
# "transform" or "surface" is not domain code). Kept as naming references.
PINNED_B = {
    "Configuration.cs", "MatrixOptions.cs",
    "Helpers/EnumHelper.cs", "Helpers/LogHelper.cs", "Helpers/ApplicationHelper.cs",
    "Helpers/RhinoInstallationHelper.cs", "Helpers/BitsHelper.cs", "Helpers/ProgressBarEventArgs.cs",
    "TranslationStore.cs", "T.cs",
    "Utilities/Converters/MathFormulaConverter.cs", "ViewModel/MessageBoxViewModel.cs",
    "Utilities/Helpers/ImageHelper.cs",
}


def classify(path: Path, project: str):
    text = path.read_text(encoding="utf-8", errors="replace")
    name = path.name
    rel = path.relative_to(UI / project).as_posix()
    if rel in PINNED_D:
        return "D", "pinned: read and judged do-not-port"
    if rel in PINNED_B:
        return "B", "pinned: generic helper, naming reference only"
    ph = len(PLACEHOLDER.findall(text))
    compute = len(COMPUTE.findall(text))
    props = len(PROPERTY.findall(text))
    vendor = len(VENDOR.findall(text))
    jew = len(JEWELLERY.findall(text))
    geom = len(GEOMETRY.findall(text))
    generated = ".Designer.cs" in name or name.endswith(".g.cs") or name in (
        "AssemblyInfo.cs", "Resources.Designer.cs", "Settings.Designer.cs")

    if ph and ph >= compute:
        return "D", "decompiler stub"
    if generated:
        return "C", "generated/designer"
    if vendor >= 3 and vendor >= jew + geom:
        return "D", "vendor/licence/telemetry/SDK dominant"
    if compute == 0 and props > 0:
        return "C", "properties/fields only"
    # Domain tier needs a real domain footprint: a solid jewellery vocabulary alongside
    # geometry, or a heavy jewellery vocabulary in code that computes. One incidental
    # keyword ("transform" in a helper) is explicitly not enough.
    if (jew >= 8 and geom >= 3 and compute >= 5) or (jew >= 15 and compute >= 10):
        return "A", ("jewellery+geometry signals" if geom >= 3 else "jewellery signal")
    if compute >= 3:
        return "B", "real bodies, no domain signal"
    return "C", "little logic"


def main():
    payload = {"schema": "om9.ref_ui_file_tiers.v1", "projects": {}}
    totals = Counter()
    for project in sorted(p.name for p in UI.iterdir() if p.is_dir()):
        base = UI / project
        rows = []
        for f in sorted(base.rglob("*.cs")):
            tier, why = classify(f, project)
            rows.append({"file": f.relative_to(base).as_posix(), "tier": tier, "why": why,
                         "lines": f.read_text(encoding="utf-8", errors="replace").count("\n")})
            totals[tier] += 1
        payload["projects"][project] = rows

    (ROOT / "build" / "ref-ui-file-tiers.json").write_text(
        json.dumps(payload, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")

    print(f"files classified: {sum(totals.values())}  {dict(totals)}")
    print()
    for project, rows in payload["projects"].items():
        c = Counter(r["tier"] for r in rows)
        a_lines = sum(r["lines"] for r in rows if r["tier"] == "A")
        print(f"=== {project}   files={len(rows)}  A={c['A']} B={c['B']} C={c['C']} D={c['D']}  "
              f"(A lines={a_lines})")
        for tier in ("A", "D"):
            items = [r for r in rows if r["tier"] == tier]
            items.sort(key=lambda r: -r["lines"])
            for r in items[:8]:
                print(f"     {tier} {r['lines']:6d}L {r['why']:34s} {r['file']}")
        print()


if __name__ == "__main__":
    main()
