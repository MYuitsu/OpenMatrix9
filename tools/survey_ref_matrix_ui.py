"""Census of the Matrix tree's UI/command surface.

The UI-control assemblies were shown to contain no command sidebar, command line,
layers palette or viewport tab XAML, so those must be located in the Matrix tree.
This measures where: per folder, the class names and the command-like member names,
plus a census of classes deriving from Rhino command/UI base types.

Read-only. Names and counts only, no source bodies.
"""
import json
import re
from collections import Counter
from pathlib import Path

ROOT = Path("H:/OpenMatrix9")
MATRIX = ROOT / "ref/matrix/decompiled-ui/Matrix"
OUT = ROOT / "build" / "ref-matrix-ui-census.json"

CLASS_RE = re.compile(r"^\s*(?:public|internal)\s+(?:sealed\s+|abstract\s+|static\s+|partial\s+)*"
                      r"(?:class|struct|enum|interface)\s+([A-Za-z_][A-Za-z0-9_]*)"
                      r"(?:\s*:\s*([A-Za-z0-9_.,<> ]+))?", re.M)
PUBLIC_MEMBER_RE = re.compile(r"^\s*public\s+(?:static\s+|virtual\s+|override\s+|sealed\s+)*"
                              r"(?:[A-Za-z0-9_<>,.\[\]?]+\s+)+([A-Za-z_][A-Za-z0-9_]*)\s*[({=;]", re.M)

# Rhino SDK base types that mark a real command or a UI surface
COMMAND_BASES = ("Rhino.Commands.Command", "Command", "Rhino.UI.Panel", "Rhino.UI.Dialog",
                 "Rhino.Display.DisplayConduit", "Rhino.Input.Custom.GetPoint",
                 "Rhino.Input.Custom.GetObject", "Rhino.Input.RhinoGet", "Form", "UserControl",
                 "Window", "Rhino.PlugIns.PlugIn")


def census(path):
    text = path.read_text(encoding="utf-8", errors="replace")
    classes = CLASS_RE.findall(text)
    members = PUBLIC_MEMBER_RE.findall(text)
    bases = {b.strip().split("<")[0].split(".")[-1] for _, b in classes if b}
    return classes, members, bases


def main():
    per_area = {}
    class_index = {}
    base_counter = Counter()
    for f in sorted(MATRIX.rglob("*.cs")):
        rel = f.relative_to(MATRIX).as_posix()
        area = rel.split("/", 1)[0] if "/" in rel else "(root)"
        classes, members, bases = census(f)
        rec = per_area.setdefault(area, {"files": 0, "classes": 0, "members": 0,
                                         "command_like": []})
        rec["files"] += 1
        rec["classes"] += len(classes)
        rec["members"] += len(members)
        for name, base in classes:
            clean = (base or "").strip().split("<")[0]
            short = clean.split(".")[-1] if clean else ""
            if short:
                base_counter[short] += 1
            class_index.setdefault(name, []).append(rel)
            if any(k in clean for k in ("Rhino.Commands.Command", "Rhino.Input.Custom",
                                        "Rhino.Display.DisplayConduit", "Rhino.UI")):
                rec["command_like"].append({"class": name, "base": clean, "file": rel})

    # also count areas that look like UI
    ui_areas = {k: v for k, v in per_area.items() if "UserInterface" in k or "Commands" in k}
    payload = {
        "schema": "om9.ref_matrix_ui_census.v1",
        "per_area": per_area,
        "rhino_base_types": dict(base_counter.most_common(30)),
        "class_count": len(class_index),
    }
    OUT.write_text(json.dumps(payload, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")

    print("=== areas with Rhino command/UI base types ===")
    for area, rec in sorted(per_area.items(), key=lambda kv: -len(kv[1]["command_like"])):
        if rec["command_like"]:
            print(f"  {area:34s} classes={rec['classes']:4d} command_like={len(rec['command_like']):4d}")
    print()
    print("=== Rhino/UI base types seen ===")
    for base, n in base_counter.most_common(18):
        print(f"  {n:4d}  {base}")
    print()
    print(f"distinct class names: {len(class_index)}")


if __name__ == "__main__":
    main()
