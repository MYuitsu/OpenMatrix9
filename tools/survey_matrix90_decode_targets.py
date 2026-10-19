"""Cross-check the live install against what ref/ already has decompiled.

Answers: which plugin assemblies have never been exported, and what kind of file are
the high-value non-assembly candidates (a .rui is a zip, .menu/.ini/.xml are text)?
File headers only; no vendor or licence content is read.
"""
import json
import zipfile
from pathlib import Path

INSTALL = Path("L:/Matrix90")
UI = INSTALL / "Plugins"
DECOMPILED = Path("H:/OpenMatrix9/ref/matrix/decompiled-ui")
OUT = Path("H:/OpenMatrix9/build/matrix90-decode-targets.json")

# assemblies ref/ already exported
ALREADY = {"Matrix.rhp", "MatrixControls.dll", "MatrixWPFControls.dll", "MatrixCommon.dll",
           "PIMData.dll", "MatrixTranslation.dll"}

# never open: licence/activation, credentials, the running executables
BLOCKED = {"SentinelKeyW.DLL", "SentinelSetupW.dll", "MatrixKey.dll",
           "MatrixStandAloneKeyProgrammer.exe", "MatrixStandAloneProgrammerInstaller.EXE",
           "Matrix90.exe", "Matrix90_backup.exe", "Matrix90.exe.gzf", "support.exe",
           "MatrixUpdater.exe", "MatrixStartup.exe", "MatrixLanguageSwitcher.exe",
           "unins000.exe", "unins001.exe", "icacls.exe"}

# third-party dependencies: not Matrix code, do not spend effort
THIRD_PARTY_PREFIX = ("Magick", "AWSSDK", "Leadtools", "log4net", "EntityFramework",
                      "Newtonsoft", "ICSharpCode", "GalaSoft", "Microsoft.", "System.",
                      "SmartAssembly", "SimplePaletteQuantizer", "SharpBITS", "PT_SDK",
                      "PanelingToolsDotNet", "OctTree", "EventHelperLibrary", "ruiSDK",
                      "VRayForRhino", "Stuller.Web", "SharpCompress")


def kind_of(path: Path):
    """Cheap structural classification from the first bytes."""
    try:
        with path.open("rb") as fh:
            head = fh.read(16)
    except OSError:
        return "unreadable"
    if head[:2] == b"MZ":
        return "PE executable/dll"
    if head[:2] == b"PK":
        return "zip container"
    if head[:2] == b"\x1f\x8b":
        return "gzip"
    if head[:2] == b"3D":
        return "openNURBS 3dm document"
    if head[:4] == b"\xd0\xcf\x11\xe0":
        return "OLE compound"
    if head[:3] == b"\xef\xbb\xbf" or head[:5] == b"<?xml":
        return "text (UTF-8 BOM/xml)"
    if head[:4] == b"ws6\x00":
        return "Rhino workspace (ws6)"
    if head[:1] == b"[":
        return "text (ini sections)"
    if head and all(32 <= b < 127 or b in (9, 10, 13) for b in head[:8]):
        return "text"
    return "binary"


def main():
    payload = {"schema": "om9.matrix90_decode_targets.v1",
               "undecompiled_assemblies": [], "reference_files": [], "notes": {}}

    for p in sorted(UI.iterdir()):
        if not p.is_file() or p.name in BLOCKED:
            continue
        if p.suffix.lower() not in (".dll", ".rhp", ".exe"):
            continue
        if p.name in ALREADY:
            continue
        if p.name.startswith(THIRD_PARTY_PREFIX):
            continue
        payload["undecompiled_assemblies"].append({
            "name": p.name, "bytes": p.stat().st_size, "kind": kind_of(p),
            "is_managed": kind_of(p) == "PE executable/dll",
        })

    # high-value non-assembly candidates across the install
    candidates = [
        ("Matrix.rui", "Rhino UI definition in text form: toolbars, menus, command ids"),
        ("ContextMenu.xml", "context menu definition the VB6 export could not recover"),
        ("Matrix.tb", "Rhino ws6 workspace/toolbar state"),
        ("Config/MainMenu.ini", "main menu definition - candidate for the 7-section sidebar"),
        ("Config/HeadStyles.ini", "head style names"),
        ("Config/HeadGallery.cpt", "head gallery"),
        ("Config/headprong.cpt", "head/prong defaults"),
        ("Config/GemCrv.xml", "gem outline curves (same data as PIMData gemCrv resource)"),
        ("Config/gem.col", "gem colour table"),
        ("Config/glist.txt", "gem list"),
        ("Config/WaxTypes.txt", "wax types"),
        ("Config/pattern.dat", "pattern data"),
        ("UserInterface/Language/English.ini",
         "translation ids -> English strings (resolves ActionTranslationId)"),
        ("UserInterface/SliderSettings.ini", "slider defaults"),
        ("UserInterface/F6/mode.menu", "F6 mode list"),
        ("UserInterface/F6/curveLayout.menu", "F6 curve mode menu"),
        ("UserInterface/F6/gemLayout.menu", "F6 gem mode menu"),
        ("UserInterface/F6/objectActions.menu", "F6 object-actions menu"),
        ("UserInterface/F6/surfaceModeling.menu", "F6 surface mode menu"),
        ("UserInterface/F6/tsplines.menu", "F6 tsplines mode menu (vendor)"),
        ("StartUp.ini", "startup configuration"),
        ("Matrix90.log", "runtime log"),
        ("UpdateLog.txt", "version/update provenance"),
        ("Config/Default.3dm", "shipped default document: layers, units, tolerance"),
        ("Config/DefaultStart.3dm", "shipped startup document"),
    ]
    for rel, why in candidates:
        p = INSTALL / rel
        if p.is_file():
            payload["reference_files"].append({
                "path": rel, "bytes": p.stat().st_size, "kind": kind_of(p), "why": why,
            })
        else:
            payload["reference_files"].append({"path": rel, "missing": True, "why": why})

    # what is inside Matrix.rui (names only)
    rui = INSTALL / "Matrix.rui"
    if rui.is_file() and zipfile.is_zipfile(rui):
        with zipfile.ZipFile(rui) as z:
            names = z.namelist()
        payload["notes"]["matrix_rui"] = {
            "entries": len(names),
            "extensions": {},
            "sample": names[:12],
        }
        ext = {}
        for n in names:
            ext[Path(n).suffix.lower()] = ext.get(Path(n).suffix.lower(), 0) + 1
        payload["notes"]["matrix_rui"]["extensions"] = dict(
            sorted(ext.items(), key=lambda kv: -kv[1]))

    OUT.write_text(json.dumps(payload, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")

    print("=== plugin assemblies NOT yet decompiled (Matrix-owned) ===")
    for row in payload["undecompiled_assemblies"]:
        print(f"  {row['bytes']:10,d} B  {row['kind']:20s} {row['name']}")
    print()
    print("=== reference files worth decoding ===")
    for row in payload["reference_files"]:
        if row.get("missing"):
            print(f"  MISSING            {row['path']}")
        else:
            print(f"  {row['bytes']:10,d} B  {row['kind']:16s} {row['path']}  <- {row['why']}")
    print()
    if "matrix_rui" in payload["notes"]:
        n = payload["notes"]["matrix_rui"]
        print(f"=== Matrix.rui: {n['entries']} entries, extensions {n['extensions']}")
        print("   sample:", ", ".join(n["sample"]))


if __name__ == "__main__":
    main()
