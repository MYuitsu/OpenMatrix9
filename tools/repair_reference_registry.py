"""Repair the machine-local private reference registry.

Every `recovered_inputs` path and `reference_guides` path in
docs/openmatrix9-reference-locations.json pointed at
H:/OpenMatrix9-private-references/2026-10-09/code/... which no longer exists (the
user removed it as duplicative and unused). The live sources are now the
workspace-local ref/ trees.

Run with --apply to write. --check only validates that every path either exists or
is explicitly recorded as removed.
"""
import argparse
import json
from pathlib import Path

ROOT = Path("H:/OpenMatrix9")
REGISTRY = ROOT / "docs" / "openmatrix9-reference-locations.json"

LIVE = {
    "live_root": "H:/OpenMatrix9/ref",
    "code/decompiled-ui": "H:/OpenMatrix9/ref/matrix/decompiled-ui",
    "code/decompiled-ui/Matrix": "H:/OpenMatrix9/ref/matrix/decompiled-ui/Matrix",
    "code/decompiled-ui/MatrixCommon": "H:/OpenMatrix9/ref/matrix/decompiled-ui/MatrixCommon",
    "code/decompiled-ui/MatrixControls": "H:/OpenMatrix9/ref/matrix/decompiled-ui/MatrixControls",
    "code/decompiled-ui/MatrixWPFControls": "H:/OpenMatrix9/ref/matrix/decompiled-ui/MatrixWPFControls",
    "code/decompiled-ui/MatrixTranslation": "H:/OpenMatrix9/ref/matrix/decompiled-ui/MatrixTranslation",
    "code/decompiled-ui/PIMData": "H:/OpenMatrix9/ref/matrix/decompiled-ui/PIMData",
    "code/decompiled-ui/native-inspection": "H:/OpenMatrix9/ref/matrix/decompiled-ui/native-inspection",
    "code/decompiled-ui/tools": "H:/OpenMatrix9/ref/matrix/decompiled-ui/tools",
    "code/Matrix": "H:/OpenMatrix9/ref/matrix/decompiled-ui/Matrix",
    "code/Matrix90": None,  # removed by the owner: duplicate, unused
    "code/vb6-lite": "H:/OpenMatrix9/ref/matrix/vb6-lite",
    "code/ghidra/Matrix90.exe.c": "H:/OpenMatrix9/ref/matrix/ghidra/Matrix90.exe.c",
    "code/vb6-lite/source-manifest.json": "H:/OpenMatrix9/ref/matrix/vb6-lite/source-manifest.json",
    "code/vb6-lite/inspection-report.json": "H:/OpenMatrix9/ref/matrix/vb6-lite/inspection-report.json",
    "rhino5/dnspy/RhinoCommon": "H:/OpenMatrix9/ref/rhino5/dnspy/RhinoCommon",
    "rhino5/dnspy/Rhino_DotNet": "H:/OpenMatrix9/ref/rhino5/dnspy/Rhino_DotNet",
    "rhino5/ghidra": "H:/OpenMatrix9/ref/rhino5/ghidra",
}
REMOVED_NOTE = ("Removed by the owner on 2026-10-16: duplicate of "
                "ref/matrix/decompiled-ui (Matrix) / unused commercial menu bitmaps (Matrix90).")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--apply", action="store_true")
    args = parser.parse_args()

    data = json.loads(REGISTRY.read_text(encoding="utf-8"))
    report = {"live": {}, "live_items": [], "dropped": [], "unresolved": []}

    # manuals: correct the filename drift (SOURCES.json names a PDF that is rhino5.pdf on disk)
    manuals = data.setdefault("manuals", {})
    guide = ROOT / "ref/rhino5/rhino5.pdf"
    if guide.is_file():
        manuals["windows_pdf_user_s_guide.pdf"] = str(guide).replace("\\", "/")
    for name, path in manuals.items():
        entry = f"manual {name} -> {path}"
        if Path(path).exists():
            report["live_items"].append(entry)
        else:
            report["unresolved"].append(entry)

    # recovered inputs
    recovered = data.setdefault("recovered_inputs", {})
    recovered.pop("code", None)  # the archive code root no longer exists
    for key, target in LIVE.items():
        if target is None:
            recovered.pop(key, None)
            report["dropped"].append(key)
            continue
        if Path(target).exists():
            recovered[key] = target
            report["live"][key] = target
        else:
            report["unresolved"].append(f"{key} -> {target} (live target missing)")

    # public documentation pointers must resolve inside this checkout
    public = data.setdefault("public_documentation", {})
    for key, rel in (("rhino5_core_index", "ref/rhino5/CORE_REFERENCE_INDEX.json"),
                     ("rhino5_guide_sources", "ref/rhino5/SOURCES.json"),
                     ("rhino5_reference_readme", "ref/rhino5/README.md"),
                     ("rhino5_guide_pdf", "ref/rhino5/rhino5.pdf")):
        path = ROOT / rel
        if path.is_file():
            public[key] = str(path).replace("\\", "/")
            report["live_items"].append(f"public {key} -> {rel}")
        else:
            report["unresolved"].append(f"public {key} -> {rel} (missing)")

    # reference images that lived only under the old FreeCAD checkout: record them as
    # unavailable rather than leaving dead absolute paths
    others = data.setdefault("other_private_references", {})
    lost = sorted(k for k, v in others.items()
                  if isinstance(v, str) and not Path(v).exists())
    for key in lost:
        others.pop(key)
        report["dropped"].append(f"other_private_references.{key} (dead path)")
    if lost:
        data.setdefault("removed_inputs", {})["reference_images"] = (
            "Absolute paths under the old FreeCAD checkout no longer exist. The images are "
            "private commercial screenshots; recover from the private archive on demand.")

    # reference guides
    guides = data.setdefault("reference_guides", {})
    guides.pop("Matrix90_Codex_Guide_MD", None)
    guides["matrix9_decompiled_ui"] = LIVE["code/decompiled-ui"]
    guides["rhino5_reference_tree"] = "H:/OpenMatrix9/ref/rhino5"
    report["dropped"].append("Matrix90_Codex_Guide_MD (path no longer exists)")

    data["removed_inputs"] = {
        "code/Matrix90": REMOVED_NOTE,
        "Matrix90_Codex_Guide_MD": "Removed with the 2026-10-09 archive layout; not referenced by any milestone.",
    }
    data["registry_repaired_at"] = "2026-10-16"
    data["registry_repair_note"] = (
        "Every path above was verified to exist at repair time. The previous "
        "OpenMatrix9-private-references/2026-10-09/code tree no longer exists; the live "
        "sources are the workspace-local ref/ trees. See docs/private-reference-index.md."
    )

    if args.apply:
        REGISTRY.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"apply": args.apply, "live": len(report["live"]),
                      "dropped": report["dropped"],
                      "unresolved": report["unresolved"]}, ensure_ascii=False, indent=1))


if __name__ == "__main__":
    main()
