#!/usr/bin/env python3
"""Ghép catalog lệnh Matrix 9 cho OM9 từ ba nguồn trong repo OpenMatrix9:

1. Resources/menu/MainMenu.ini          – nhóm menu (File, Curve, Gems, ...) và thứ tự icon
2. crates/om9-app/src/icons.rs          – icon!(mã_menu, tên, feature_id, svg)
3. crates/om9-app/assets/features.json  – feature_id -> tên hiển thị, lệnh gõ (gvGemOnCrv), module

Kết quả: om9_command_catalog.json – mỗi lệnh nằm đúng nhóm của nó, kèm lệnh gõ
nếu biết. Feature không có trên menu được xếp vào nhóm theo `module`.

    build_command_catalog.py <repo_root> -o catalog.json
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from collections import OrderedDict
from pathlib import Path

# module trong features.json -> nhóm menu Matrix 9 (dùng cho feature không có icon menu)
MODULE_TO_GROUP = {
    "Top 11 Buttons": "Quick", "File": "File", "View": "View", "Display": "View",
    "Utilities": "Utilities", "Mesh Tools": "Utilities", "Measure": "Measure",
    "Curve Tools": "Curve", "Surface Tools": "Surface", "Solid Tools": "Solid",
    "Transform Tools": "Transform", "SubD": "Clayoo", "T-Splines": "T-Splines",
    "Emboss": "Emboss", "Matrix Art": "Emboss", "Builders": "Builder", "Styles": "Builder",
    "Tools": "Tools", "Matrix Tools": "Tools", "Placing Gems": "Gems",
    "Setting Menu": "Settings", "Creating Cutters": "Cutters", "Render Menu": "Render",
    "Snaps": "Snap", "Info & Settings": "Info", "History": "Info", "Layers": "Info",
    "Project Manager": "Project", "Project Manager Database": "Project",
}
# Mục hướng dẫn giao diện trong manual, không phải lệnh
GUIDE_MODULES = {"A Tour of the Interface", "The Main Menu", "The F6 Menu",
                 "Modeling in the Viewports", "Builders vs. Commands"}
NON_COMMAND_MENUS = {"Custom", "Reset"}  # nút điều khiển sidebar, không phải lệnh CAD


def parse_menu(path: Path):
    groups: "OrderedDict[str, list[str]]" = OrderedDict()
    cur_name, cur_icons, section = None, [], None
    top: list[str] = []

    def flush():
        if section and section.startswith("Menu") and cur_name:
            groups[cur_name] = list(cur_icons)

    for raw in path.read_text(encoding="utf-8-sig").splitlines():
        line = raw.strip()
        m = re.match(r"^\[(.+)\]$", line)
        if m:
            flush()
            section, cur_name, cur_icons = m.group(1), None, []
            continue
        if "=" not in line:
            continue
        k, v = (s.strip() for s in line.split("=", 1))
        if section == "Top11" and k.startswith("Icon"):
            top.append(v)
        elif section and section.startswith("Menu"):
            if k == "Name":
                cur_name = v
            elif re.match(r"^Icon\d+$", k):
                cur_icons.append(v)
    flush()
    return top, groups


def parse_icons(path: Path) -> dict[str, str]:
    text = path.read_text(encoding="utf-8")
    out = {}
    for m in re.finditer(r'icon!\(\s*"([^"]+)",\s*"([^"]*)",\s*"([^"]*)",', text):
        code, _name, feature = m.groups()
        if feature:
            out[code] = feature
    return out


def humanize(code: str) -> str:
    s = re.sub(r"^(TopIcon|Builder|Others|Clayoo(Edit|Creation|Primitives|Selection)?|"
               r"Curve(Curve(FromObjects|EditTools)|FreeForm|Offset|Blend Curves|Line|Polyline)?|"
               r"Surface(SurfaceEditTools|Sweep|VariableFilletBlendChamfer)?|Solid(SolidEditTools(Holes|Faces))?|"
               r"Transform(Array|Scale|CageEditing)?|Analyze(EdgeTools|Diagnostics)?|View(Zoom|SetCamera)?|"
               r"Mesh(MeshRepairTools|MeshEditTools(Collapse)?)?|Dimension|Render(Effects)?|InfoSettings|"
               r"ToolsObjectSnap|Emboss|File|gv)", "", code) or code
    return re.sub(r"(?<=[a-z0-9])(?=[A-Z])", " ", s).replace("_", " ").strip()


def norm_name(s: str) -> str:
    import unicodedata
    s = unicodedata.normalize("NFKD", s).encode("ascii", "ignore").decode().lower()
    s = re.sub(r"\b(builder|tool|tools|the|on|to|of)\b", " ", s)
    s = s.replace(" 2 ", " two ").replace("crv", "curve").replace("srf", "surface")
    return re.sub(r"[^a-z0-9]", "", s)


def build(root: Path) -> dict:
    top, groups = parse_menu(root / "Resources/menu/MainMenu.ini")
    icon_feature = parse_icons(root / "crates/om9-app/src/icons.rs")
    features = {f["id"]: f for f in json.loads((root / "crates/om9-app/assets/features.json").read_text(encoding="utf-8"))}

    used_features: set[str] = set()
    out_groups = []
    seen_codes: dict[str, str] = {}

    def entry(code: str, group: str) -> dict:
        fid = icon_feature.get(code)
        f = features.get(fid) if fid else None
        if fid:
            used_features.add(fid)
        e = {"code": code, "label": f["name"] if f else humanize(code)}
        if f:
            e["command"] = f.get("command") or None
            e["feature_id"] = fid
            e["module"] = f.get("module")
        if code in seen_codes and seen_codes[code] != group:
            e["also_in"] = seen_codes[code]
        seen_codes.setdefault(code, group)
        return e

    out_groups.append({"group": "Quick", "label": "Quick tools", "source": "Top11",
                       "commands": [entry(c, "Quick") for c in top]})
    for name, icons in groups.items():
        if name in NON_COMMAND_MENUS:
            continue
        out_groups.append({"group": name, "label": name, "source": "MainMenu",
                           "commands": [entry(c, name) for c in icons]})

    # Feature có trong tài liệu nhưng không có icon menu -> gắn vào nhóm theo module
    extra: "OrderedDict[str, list]" = OrderedDict()
    for fid, f in features.items():
        if fid in used_features:
            continue
        if f.get("module") in GUIDE_MODULES:
            continue
        g = MODULE_TO_GROUP.get(f.get("module", ""), "Other")
        extra.setdefault(g, []).append({"code": None, "label": f["name"], "command": f.get("command") or None,
                                        "feature_id": fid, "module": f.get("module"), "menu": False})
    by_name = {g["group"]: g for g in out_groups}
    # Ghép mục menu chưa có feature với feature ngoài menu cùng nhóm, theo tên chuẩn hoá
    matched = 0
    for g, items in extra.items():
        if g not in by_name:
            continue
        pool = {}
        for it in items:
            pool.setdefault(norm_name(it["label"]), []).append(it)
        for e in by_name[g]["commands"]:
            if "feature_id" in e:
                continue
            cand = pool.get(norm_name(e["label"])) or []
            if len(cand) == 1:  # chỉ ghép khi duy nhất, tránh đoán sai
                it = cand[0]
                e.update(command=it["command"], feature_id=it["feature_id"], module=it["module"], matched_by="name")
                items.remove(it)
                matched += 1
    for g, items in extra.items():
        if g in by_name:
            by_name[g].setdefault("unlisted", []).extend(items)
        else:
            out_groups.append({"group": g, "label": g, "source": "features.json", "commands": [], "unlisted": items})

    unique_codes = {c["code"] for g in out_groups for c in g["commands"]}
    return {
        "schema": 1,
        "note": "Tên lệnh/nhóm lấy từ catalog OpenMatrix9; không chứa mã hay tài nguyên thương mại.",
        "totals": {
            "groups": len(out_groups),
            "menu_entries": sum(len(g["commands"]) for g in out_groups),
            "unique_menu_codes": len(unique_codes),
            "with_typed_command": sum(1 for g in out_groups for c in g["commands"] if c.get("command")),
            "unlisted_features": sum(len(g.get("unlisted", [])) for g in out_groups),
            "matched_by_name": matched,
            "features_total": len(features),
        },
        "groups": out_groups,
    }


def main(argv=None):
    ap = argparse.ArgumentParser()
    ap.add_argument("repo", type=Path)
    ap.add_argument("-o", type=Path, default=Path("om9_command_catalog.json"))
    a = ap.parse_args(argv)
    cat = build(a.repo)
    a.o.write_text(json.dumps(cat, ensure_ascii=False, indent=1), encoding="utf-8")
    t = cat["totals"]
    print(f"{t['groups']} nhóm, {t['menu_entries']} mục menu ({t['unique_menu_codes']} mã duy nhất), "
          f"{t['with_typed_command']} có lệnh gõ, {t['unlisted_features']} feature ngoài menu / {t['features_total']}")
    for g in cat["groups"]:
        print(f"  {g['group']:10} {len(g['commands']):3} menu  {len(g.get('unlisted', [])):3} ngoài menu")
    return 0


if __name__ == "__main__":
    sys.exit(main())
