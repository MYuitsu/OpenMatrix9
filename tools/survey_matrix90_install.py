"""Survey the live Matrix9 install at L:\\Matrix90.

Purpose: decide which files are worth decoding for OM9, and which must never be
touched. Records only paths, sizes, kinds and small structural facts - no content
from licence, credential or vendor files is read.
"""
import json
import re
from pathlib import Path

ROOT = Path("L:/Matrix90")
OUT = Path("H:/OpenMatrix9/build/matrix90-install-survey.json")

# never read the bytes of these: licence/activation, credentials, or the running binaries
BLOCKED = ("SentinelKeyW.DLL", "SentinelSetupW.dll", "MatrixKey.dll",
           "MatrixStandAloneKeyProgrammer.exe", "MatrixStandAloneProgrammerInstaller.EXE",
           "Matrix90.exe", "Matrix90_backup.exe", "Matrix90.exe.gzf",
           "support.exe", "MatrixUpdater.exe", "MatrixStartup.exe",
           "MatrixLanguageSwitcher.exe", "unins000.exe", "unins001.exe",
           "unins000.dat", "unins001.dat", "ICACLS.exe", "icacls.exe")

INTERESTING = {
    "Matrix.rui": "Rhino UI file: toolbars, menus, command ids",
    "ContextMenu.xml": "context-menu definition (VB6 export could not recover this)",
    "Matrix.tb": "toolbar definition",
    "StartUp.ini": "startup configuration",
    "MatrixStartup.exe.config": "runtime configuration",
    "MatrixUpdater.exe.config": "update configuration",
    "Matrix90.log": "runtime log",
    "UpdateLog.txt": "update history - version provenance",
}

DIR_NOTES = {
    "Defaults": "shipped default libraries (gems, cutters, profiles)",
    "Library": "user/shipped libraries",
    "Config": "configuration files",
    "dpro": "profile data",
    "ImageTrace": "image-trace assets",
    "MeshRepair": "mesh repair data",
    "Plugins": "plugin assemblies (some already decompiled in ref/)",
    "Revo": "revolve/toolpath data",
    "UserInterface": "UI assets",
    "utils": "helper utilities",
    "Vray": "V-Ray render assets",
}


def main():
    if not ROOT.is_dir():
        print("L:/Matrix90 not found")
        raise SystemExit(1)

    payload = {"schema": "om9.matrix90_install_survey.v1", "folders": {}, "files": {}}

    for entry in sorted(ROOT.iterdir()):
        if entry.is_dir():
            files = [p for p in entry.rglob("*") if p.is_file()]
            by_ext = {}
            for p in files:
                by_ext[p.suffix.lower()] = by_ext.get(p.suffix.lower(), 0) + 1
            payload["folders"][entry.name] = {
                "note": DIR_NOTES.get(entry.name, ""),
                "files": len(files),
                "bytes": sum(p.stat().st_size for p in files),
                "by_extension": dict(sorted(by_ext.items(), key=lambda kv: -kv[1])[:10]),
            }
        else:
            size = entry.stat().st_size
            payload["files"][entry.name] = {
                "bytes": size,
                "blocked": entry.name in BLOCKED,
                "why_interesting": INTERESTING.get(entry.name, ""),
            }

    # Plugins detail: which assemblies exist here vs what ref/ already has
    plugins = ROOT / "Plugins"
    if plugins.is_dir():
        payload["plugins"] = [
            {"name": p.name, "bytes": p.stat().st_size}
            for p in sorted(plugins.iterdir()) if p.is_file()
        ]

    OUT.write_text(json.dumps(payload, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")

    print("=== folders ===")
    for name, d in payload["folders"].items():
        print(f"  {name:16s} {d['files']:5d} files {d['bytes']/1024/1024:9.1f} MB  {d['note']}")
    print()
    print("=== root files worth decoding (not blocked) ===")
    for name, d in payload["files"].items():
        if d["blocked"]:
            continue
        flag = "INTERESTING" if d["why_interesting"] else ""
        if flag or name.lower().endswith((".xml", ".ini", ".config", ".log", ".txt", ".rui", ".tb", ".json")):
            print(f"  {name:34s} {d['bytes']:10,d} B  {flag} {d['why_interesting']}")
    print()
    print("=== blocked (must not be read) ===")
    print("  " + ", ".join(n for n, d in payload["files"].items() if d["blocked"]))
    print()
    if "plugins" in payload:
        print(f"=== Plugins: {len(payload['plugins'])} files ===")
        for p in sorted(payload["plugins"], key=lambda x: -x["bytes"])[:18]:
            print(f"  {p['bytes']:10,d} B  {p['name']}")


if __name__ == "__main__":
    main()
