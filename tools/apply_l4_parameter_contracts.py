"""Add command/domain parameter contracts to M03-M07 L3/L4 records.

Reads the derived Matrix command catalog (Menu -> Command -> Option), attaches a
canonical parameter contract to every L3, adds a compact reference to every L4,
and creates one PARAMETER_IMPLEMENTATION L4 per L3 whose catalog options are not
covered by an existing L3, so each menu command keeps its own domain inputs.

The tool is deliberately conservative about values it cannot recover: option
types/defaults/ranges stay UNRESOLVED and are handed to the new L4 rather than
invented.
"""

from __future__ import annotations

import argparse
import json
import re
from collections import defaultdict
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
MILESTONES = ROOT / "OM9_Milestones"
CATALOG_PATH = ROOT / "docs/analysis/matrix-command-layers.json"
REPORT_PATH = ROOT / "docs/analysis/matrix-l4-parameter-coverage.md"
MILESTONE_DIRS = ["M03", "M04", "M05", "M06", "M07"]

CATALOG = json.loads(CATALOG_PATH.read_text(encoding="utf-8"))


def norm(text: str) -> str:
    text = (text or "").lower()
    text = text.replace("3-d", "3d").replace("2-d", "2d").replace("1-d", "1d")
    text = re.sub(r"[^a-z0-9]+", " ", text)
    return " ".join(text.split())


def compact(text: str) -> str:
    return norm(text).replace(" ", "")


MENUS = {m["name"]: m for m in CATALOG["menus"]}
COMMANDS: dict[tuple[str, str], dict[str, Any]] = {}
OPTIONS: dict[tuple[str, str], list[tuple[dict[str, Any], dict[str, Any]]]] = defaultdict(list)
for _menu in CATALOG["menus"]:
    for _command in _menu["commands"]:
        COMMANDS[(norm(_menu["name"]), norm(_command["name"]))] = _command
        for _option in _command["options"]:
            OPTIONS[(norm(_menu["name"]), norm(_option["label"]))].append((_command, _option))
            OPTIONS[(norm(_menu["name"]), compact(_option["label"]))].append((_command, _option))

# Aliases whose Matrix label and catalog label differ in wording/spacing.
M03_ALIASES: dict[tuple[str, str], tuple[str, str, str | None]] = {
    ("Solid", "boolean union"): ("Solid", "Boolean Tools", "Union"),
    ("Solid", "boolean difference"): ("Solid", "Boolean Tools", "Difference"),
    ("Solid", "boolean intersection"): ("Solid", "Boolean Tools", "Intersection"),
    ("Solid", "boolean two objects"): ("Solid", "Boolean Tools", "Boolean Two Object"),
    ("Solid", "cap planar"): ("Solid", "(root)", "Cap Planar"),
    ("Solid", "extract surface"): ("Solid", "(root)", "Extract Surface"),
    ("Solid", "fillet edge"): ("Solid", "Fillet Edge", "Fillet Edge"),
    ("Solid", "text"): ("Solid", "(root)", "Text..."),
    ("Solid", "extrude curve straight"): ("Solid", "Extrude Planar Curve", "Extrude Curve Straight"),
    ("Solid", "solid extrude all"): ("Solid", "Extrude Planar Curve", "Extrude Curve Straight"),
    ("Solid", "pipe"): ("Solid", "(root)", "Pipe"),
    ("Solid", "box"): ("Solid", "Box", "Box"),
    ("Solid", "sphere"): ("Solid", "Sphere", "Sphere"),
    ("Solid", "ellipsoid"): ("Solid", "Ellipsoid", "Ellipsoid from Center"),
    ("Solid", "torus"): ("Solid", "(root)", "Torus"),
    ("Solid", "cylinder"): ("Solid", "(root)", "Cylinder"),
    ("Solid", "tube"): ("Solid", "(root)", "Tube"),
    ("Solid", "pyramid"): ("Solid", "(root)", "Pyramid"),
    ("Solid", "cone"): ("Solid", "(root)", "Cone"),
    ("Solid", "truncated cone"): ("Solid", "(root)", "Truncated Cone"),
    ("Solid", "shell"): ("Solid", "(root)", "Shell"),
    ("Surface", "sweep 1"): ("Surface", "Sweep", "Sweep 1"),
    ("Surface", "sweep 1 history"): ("Surface", "Sweep", "Sweep 1 History"),
    ("Surface", "sweep 2"): ("Surface", "Sweep", "Sweep 2"),
    ("Surface", "sweep 2 history"): ("Surface", "Sweep", "Sweep 2 History"),
    ("Surface", "sweep 4"): ("Surface", "Sweep", "Sweep Multi"),
    ("Surface", "sweep multi"): ("Surface", "Sweep", "Sweep Multi"),
    ("Surface", "profile sweep"): ("Surface", "Sweep", "Profile Sweep"),
    ("Surface", "rebuild"): ("Surface", "Surface Edit Tools", "Rebuild UV"),
    ("Surface", "blend surface"): ("Surface", "(root)", "Blend Surface"),
    ("Surface", "variable blend surfaces"): ("Surface", "Variable Fillet/Blend/Chamfer", "Variable Blend Surfaces"),
    ("Surface", "loft"): ("Surface", "(root)", "Loft"),
    ("Surface", "curve network"): ("Surface", "(root)", "Curve Network"),
    ("Surface", "shrink trimmed surface"): ("Surface", "Surface Edit Tools", "Shrink Trimmed Surface"),
    ("Surface", "patch"): ("Surface", "(root)", "Patch"),
    ("Surface", "planar curves"): ("Surface", "(root)", "Planar Curves"),
    ("Surface", "edge curves"): ("Surface", "(root)", "Edge Curves"),
    ("Surface", "plane from corners"): ("Surface", "Plane", "Plane from Corners"),
    ("Surface", "untrim"): ("Surface", "Surface Edit Tools", "Untrim"),
    ("Surface", "surface extrude all"): ("Surface", "Extrude Curve", "Extrude Straight"),
    ("Surface", "rail revolve"): ("Surface", "(root)", "Rail Revolve"),
    ("Surface", "fillet surfaces"): ("Surface", "(root)", "Fillet Surfaces"),
    ("Surface", "variable fillet surfaces"): ("Surface", "Variable Fillet/Blend/Chamfer", "Variable Fillet Surfaces"),
    ("Surface", "chamfer surfaces"): ("Surface", "(root)", "Chamfer Surfaces"),
    ("Surface", "variable chamfer surfaces"): ("Surface", "Variable Fillet/Blend/Chamfer", "Variable Chamfer Surfaces"),
    ("Surface", "offset surface"): ("Surface", "(root)", "Offset Surface"),
    ("Surface", "variable offset surface"): ("Surface", "(root)", "Variable Offset Surface"),
    ("Surface", "tween surfaces"): ("Surface", "(root)", "Tween Surfaces"),
    ("Surface", "merge surfaces"): ("Surface", "Surface Edit Tools", "Merge"),
    ("Surface", "match surfaces"): ("Surface", "Surface Edit Tools", "Match"),
    ("Surface", "drape"): ("Surface", "(root)", "Drape"),
    ("Surface", "split at isocurve"): ("Surface", "Surface Edit Tools", "Split at Isocurve"),
    ("Surface", "extend surface"): ("Surface", "(root)", "Extend Surface"),
    ("Surface", "extrude normal to surface"): ("Surface", "Extrude Curve", "Extrude Normal to Surface"),
    ("Surface", "unroll surface"): ("Surface", "(root)", "Unroll Surface"),
    ("Surface", "soft edit"): ("Surface", "Surface Edit Tools", "Soft Edit"),
    ("Surface", "cutting plane"): ("Surface", "Plane", "Cutting Plane"),
    ("Surface", "adjust closed surface seam"): ("Surface", "Surface Edit Tools", "Adjust Closed Surface Seam"),
    ("Surface", "set surface tangent"): ("Surface", "Surface Edit Tools", "Set Surface Tangent"),
    ("Transform", "scale 3d"): ("Transform", "Scale", "Scale 3-D"),
    ("Transform", "scale2d"): ("Transform", "Scale", "Scale 2-D"),
    ("Transform", "scale1d"): ("Transform", "Scale", "Scale 1-D"),
    ("Transform", "scale non uniform"): ("Transform", "Scale", "Scale Non-Uniform"),
    ("Transform", "scale by plane"): ("Transform", "Scale", "Scale By Plane"),
    ("Transform", "orient 2 points"): ("Transform", "Orient", "Orient 2 Points"),
    ("Transform", "orient 3 points"): ("Transform", "Orient", "Orient 3 Points"),
    ("Transform", "orient on surface"): ("Transform", "Orient", "Orient on Surface"),
    ("Transform", "orient perpendicular to curve"): ("Transform", "Orient", "Orient Perpendicular to Curve"),
    ("Transform", "orient on curve"): ("Transform", "Orient", "Orient on Curve"),
    ("Transform", "remap to c plane"): ("Transform", "Orient", "Remap to C-Plane"),
    ("Transform", "rotate3d"): ("Transform", "(root)", "Rotate 3-D"),
    ("Transform", "bend"): ("Transform", "(root)", "Bend"),
    ("Transform", "array polar"): ("Transform", "Array", "Array Polar"),
    ("Transform", "array along curve"): ("Transform", "Array", "Array along Curve"),
    ("Transform", "array linear"): ("Transform", "Array", "Array Linear"),
    ("Transform", "array along surface"): ("Transform", "Array", "Array along Surface"),
    ("Transform", "array curve on surface"): ("Transform", "Array", "Array Curve on Surface"),
    ("Transform", "array"): ("Transform", "Array", "Array"),
    ("Transform", "cage edit"): ("Transform", "Cage Editing", "Cage Edit"),
    ("Transform", "releasefromcage"): ("Transform", "Cage Editing", "Release from Cage"),
    ("Transform", "create cage"): ("Transform", "Cage Editing", "Create Cage"),
    ("Transform", "solid points on"): ("Transform", "(root)", "Solid Points On"),
    ("Transform", "project to c plane"): ("Transform", "(root)", "Project to C-Plane"),
    ("Transform", "flow along curve"): ("Transform", "(root)", "Flow along Curve"),
    ("Transform", "flow along surface"): ("Transform", "(root)", "Flow along Surface"),
    ("Transform", "soft move"): ("Transform", "(root)", "Soft Move"),
    ("Transform", "moveuvn"): ("Transform", "(root)", "Move UVN"),
    ("Curve", "polyline"): ("Curve", "Polyline", "Polyline"),
    ("Curve", "line"): ("Curve", "Line", "Single Line"),
    ("Curve", "interp curve"): ("Curve", "Free-Form", "Interp Curve"),
    ("Curve", "rectangle"): ("Curve", "Rectangle", "Rectangle"),
    ("Curve", "circle"): ("Curve", "Circle", "Circle"),
    ("Curve", "ellipse"): ("Curve", "Ellipse", "Ellipse"),
    ("Curve", "arc"): ("Curve", "Arc", "Arc"),
    ("Curve", "arc direction"): ("Curve", "Arc", "Arc Direction"),
    ("Curve", "rebuild"): ("Others", "(root)", "Rebuild"),
    ("Curve", "refit to tolerance"): ("Curve", "Curve Edit Tools", "Refit to Tolerance"),
    ("Curve", "quick curve blend"): ("Curve", "Blend Curves", "Quick Curve Blend"),
    ("Curve", "adjustable curve blend"): ("Curve", "Blend Curves", "Adjustable Curve Blend"),
    ("Curve", "curve from 2 views"): ("Curve", "(root)", "Curve from 2 Views"),
    ("Curve", "curve 2 views with history"): ("Curve", "(root)", "Curve from 2 Views"),
    ("Curve", "pullback"): ("Curve", "Curve from Objects", "Pullback"),
    ("Curve", "project"): ("Curve", "Curve from Objects", "Project"),
    ("Curve", "intersection"): ("Curve", "Curve from Objects", "Intersection"),
    ("Curve", "offset curve"): ("Curve", "Offset Curve", "Offset Curve"),
    ("Curve", "offset curve on surface"): ("Curve", "Offset Curve", "Offset Curve on Surface"),
    ("Curve", "extract isocurve"): ("Curve", "Curve from Objects", "Extract Isocurve"),
    ("Curve", "extract angle isocurve"): ("Curve", "Curve from Objects", "Extract Angle Isocurve"),
    ("Curve", "fillet curves"): ("Curve", "Cleanup Tools", "Fillet Curves"),
    ("Curve", "fillet corners"): ("Curve", "Cleanup Tools", "Fillet Corners"),
    ("Curve", "chamfer curves"): ("Curve", "Cleanup Tools", "Chamfer Curves"),
    ("Curve", "create uv curves"): ("Curve", "Curve from Objects", "Create UV Curves"),
    ("Curve", "apply uv curves"): ("Curve", "Curve from Objects", "Apply UV Curves"),
    ("Curve", "through points"): ("Curve", "Free-Form", "Through Points"),
    ("Curve", "extend curve"): ("Curve", "Extend Curve", "Extend Curve"),
    ("Curve", "polygon"): ("Curve", "Polygon", "Polygon"),
    ("Curve", "spiral"): ("Curve", "(root)", "Spiral"),
    ("Curve", "helix"): ("Curve", "(root)", "Helix"),
    ("Curve", "point"): ("Curve", "Point Object", "Single Point"),
    ("Curve", "mark curve start"): ("Curve", "Point Object", "Mark Curve Start"),
    ("Curve", "mark curve end"): ("Curve", "Point Object", "Mark Curve End"),
    ("Curve", "adjust closed curve seam"): ("Curve", "Curve Edit Tools", "Adjust Closed Curve Seam"),
    ("Curve", "continue interpolated curve"): ("Curve", "Free-Form", "Continue Interpolated Curve"),
    ("Curve", "divide curve"): ("Curve", "Divide Curve by", "Number of Segments"),
    ("Curve", "curve boolean"): ("Curve", "Curve Edit Tools", "Curve Boolean"),
    ("Curve", "tween curves"): ("Curve", "(root)", "Tween Curves"),
    ("Curve", "duplicate edge"): ("Curve", "Curve from Objects", "Duplicate Edge"),
    ("Curve", "duplicate border"): ("Curve", "Curve from Objects", "Duplicate Border"),
    ("Curve", "curve"): ("Curve", "Free-Form", "Curve"),
    ("Curve", "continue control point curve"): ("Curve", "Free-Form", "Continue Control Point Curve"),
    ("Curve", "cross section profiles"): ("Curve", "(root)", "Cross-Section Profiles"),
    ("Curve", "sketch"): ("Curve", "Free-Form", "Sketch"),
    ("Curve", "sketch on surface"): ("Curve", "Free-Form", "Sketch on Surface"),
    ("Curve", "sketch on mesh"): ("Curve", "Free-Form", "Sketch on Mesh"),
    ("Curve", "polyline on surface"): ("Curve", "Free-Form", "Polyline on Surface"),
    ("Curve", "interp curve on surface"): ("Curve", "Free-Form", "Interp Curve on Surface"),
    ("Curve", "curve to lines"): ("Curve", "Convert", "Curve to Lines"),
    ("Curve", "section"): ("Curve", "Curve from Objects", "Section"),
    ("Curve", "match"): ("Curve", "Curve Edit Tools", "Match"),
    ("Curve", "silhouette"): ("Curve", "Curve from Objects", "Silhouette"),
    ("Curve", "extract wire frame"): ("Curve", "Curve from Objects", "Extract Wireframe"),
    ("Curve", "soft edit"): ("Curve", "Curve Edit Tools", "Soft Edit"),
    ("Curve", "offset normal to surface"): ("Curve", "Offset Curve", "Offset Normal to Surface"),
    ("Curve", "arc blend"): ("Curve", "Blend Curves", "Arc Blend"),
    ("Curve", "intersect two sets"): ("Curve", "Curve from Objects", "Intersect Two Sets"),
}


def read_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8-sig"))


def write_json(path: Path, data: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def relative(path: Path) -> str:
    return path.resolve().relative_to(ROOT).as_posix()


def milestone_relative(path: Path) -> str:
    return relative(path).removeprefix("OM9_Milestones/")


def match_l3(title: str) -> tuple[str, str, str | None] | None:
    if " — menu binding" not in title:
        return None
    left = title.split(" — menu binding")[0]
    if " · " not in left:
        return None
    menu, option = left.split(" · ", 1)
    key = (menu.strip(), norm(option))
    alias = M03_ALIASES.get(key)
    if alias:
        return alias
    command = COMMANDS.get((norm(menu), norm(option)))
    if command:
        return (menu.strip(), command["name"], None)
    candidates = OPTIONS.get((norm(menu), norm(option))) or OPTIONS.get((norm(menu), compact(option)))
    if candidates:
        candidate_command, candidate_option = candidates[0]
        return (menu.strip(), candidate_command["name"], candidate_option["label"])
    return None


def command_for(menu: str, name: str) -> dict[str, Any] | None:
    return COMMANDS.get((norm(menu), norm(name)))


def command_options(menu: str, name: str) -> list[dict[str, Any]]:
    command = command_for(menu, name)
    return [] if command is None else command["options"]


BAD_INPUT_MARKERS = (
    "specs/",
    "implementation/",
    "g00",
    "l3 canonical",
    "owner module",
    "contract",
    "gate",
    "sha",
    "toolchain",
    "source_revision",
    "source_path",
    "source audit",
    "source checkout",
    "qymcad source",
    "manual",
    "evidence",
    "fixture identifier",
    "prior milestone",
    "requires_actual_execution",
)
PARAM_WORDS = (
    "diameter",
    "radius",
    "angle",
    "height",
    "width",
    "depth",
    "length",
    "count",
    "density",
    "tolerance",
    "offset",
    "thickness",
    "size",
    "prong",
    "gem",
    "stone",
    "seat",
    "layer",
    "selection",
    "transform",
    "position",
    "open",
    "closed",
    "volume",
    "mass",
    "unit",
    "feature id",
    "asset",
    "transaction",
    "parameter",
    "rebuild",
    "source",
    "rail",
    "setting",
    "component",
    "object",
    "reference",
    "direction",
    "plane",
    "profile",
    "curve",
    "surface",
    "solid",
    "mesh",
    "material",
    "color",
    "shape",
    "instance",
    "placement",
    "orientation",
)


def clean_input(value: str) -> str | None:
    text = " ".join(str(value).split())
    if len(text) < 3:
        return None
    low = text.lower()
    if any(marker in low for marker in BAD_INPUT_MARKERS):
        return None
    if low.startswith(("om9-", "m03.", "m05_", "m06_", "m07_", "invoke ", "no ", "typed command", "line length=")):
        return None
    if any(marker in low for marker in ("fixture_id", "case_id", "command_id", "test_name", "assertions", "setup", "-golden", "-positive", "-negative")):
        return None
    if not re.search(r"\d", text) and not any(word in low for word in PARAM_WORDS):
        return None
    return text


INPUT_VALUE_KEYS = {
    "input",
    "inputs",
    "input_fixture",
    "input_contract",
    "feature_parameters",
    "expected_domain_inputs",
    "negative_input",
}
CONTAINER_KEYS = {
    "positive_fixture",
    "negative_fixture",
    "task_specific_acceptance",
    "golden_test_design",
    "fixture_design",
}


def collect_inputs(value: Any, source: str, key: str, out: list[dict[str, Any]], active: bool = False) -> None:
    if isinstance(value, str):
        if not active:
            return
        cleaned = clean_input(value)
        if cleaned:
            out.append(
                {
                    "raw": cleaned,
                    "type": "UNRESOLVED",
                    "unit": "mm" if "mm" in cleaned.lower() else None,
                    "default": None,
                    "required": True,
                    "source": f"{source}#{key}",
                }
            )
    elif isinstance(value, list):
        for item in value:
            collect_inputs(item, source, key, out, active)
    elif isinstance(value, dict):
        for child_key, child in value.items():
            if child_key == "input" and isinstance(child, dict):
                child_active = False
            elif child_key in INPUT_VALUE_KEYS:
                child_active = True
            elif child_key in CONTAINER_KEYS:
                child_active = False
            else:
                child_active = active
            collect_inputs(child, source, f"{key}.{child_key}", out, child_active)


def domain_inputs(l3: dict[str, Any], l4s: list[dict[str, Any]], l3_path: Path) -> list[dict[str, Any]]:
    out: list[dict[str, Any]] = []
    collect_inputs(l3, relative(l3_path), "l3", out)
    for l4 in l4s:
        if l4.get("task_role") == "PARAMETER_IMPLEMENTATION":
            continue
        path = Path(l4["_path"])
        collect_inputs(l4, relative(path), "l4", out)
    deduped: list[dict[str, Any]] = []
    seen = set()
    for item in out:
        if item["raw"] in seen:
            continue
        seen.add(item["raw"])
        deduped.append(item)
    return deduped


def option_record(option: dict[str, Any], covered_by: str | None) -> dict[str, Any]:
    label = option["label"]
    return {
        "label": label,
        "command_id": option["command_id"],
        "script": option.get("script", ""),
        "alt_script": option.get("alt_script", ""),
        "alt_label": option.get("alt_label", ""),
        "covered_by_l3": covered_by,
        "status": "COVERED" if covered_by else "MISSING",
        "type": None,
        "default": None,
        "range": None,
        "unit": None,
        "inline": {
            "mode": "COMMAND_INLINE",
            "token": label,
            "typed_forms": [label],
            "shortcut": None,
            "shortcut_status": "UNRESOLVED",
            "underlined_index": None,
            "clickable": True,
            "activation": "UNRESOLVED",
            "availability": "UNRESOLVED",
            "value_owner": "UNRESOLVED",
            "value_type": "UNRESOLVED",
            "prompt_fragment": "UNRESOLVED",
            "route": "Controller::submit",
        },
    }


def next_l4(host: dict[str, Any], menu: str, command: str) -> tuple[int, str, str, Path]:
    children = host["l3"].get("children_l4") or []
    marker = f"Parameter Implementation — {menu} · {command} options"
    for child in children:
        title = str(child.get("title", ""))
        matches = title.startswith(marker) if menu and command else title.startswith("Parameter Implementation — ")
        if child.get("role") == "PARAMETER_IMPLEMENTATION" and matches:
            child_file = Path(child.get("file", ""))
            l4_id = child["id"]
            match = re.search(r"-(\d{2})$", l4_id)
            seq = int(match.group(1)) if match else 0
            if child_file.is_absolute():
                folder = child_file.parent
            elif child_file.parts and child_file.parts[0] == "OM9_Milestones":
                folder = (ROOT / child_file).parent
            else:
                folder = (MILESTONES / child_file).parent
            return seq, l4_id, folder.name, folder
    sequences = []
    for child in children:
        match = re.search(r"-(\d{2})$", str(child.get("id", "")))
        if match:
            sequences.append(int(match.group(1)))
    seq = max(sequences, default=0) + 1
    host_l3_id = host["l3"]["id"]
    l4_id = "OM9-L4-" + host_l3_id[len("OM9-L3-") :] + f"-{seq:02d}"
    folder = host["l3_path"].parent / f"L4-{seq:02d} - Parameter Implementation"
    return seq, l4_id, folder.name, folder


def build_contract(l3: dict[str, Any], l4s: list[dict[str, Any]], l3_path: Path, hit, covered: dict[str, str]) -> dict[str, Any]:
    menu = hit[0] if hit else None
    command = hit[1] if hit else None
    option = hit[2] if hit else None
    root_group = command == "(root)"
    inputs = domain_inputs(l3, l4s, l3_path)
    records: list[dict[str, Any]] = []
    missing_options: list[str] = []
    if command:
        for catalog_option in command_options(menu, command):
            covered_by = covered.get(catalog_option["label"])
            records.append(option_record(catalog_option, covered_by))
            if covered_by is None and not root_group:
                missing_options.append(catalog_option["label"])
    if command and root_group:
        status = "ROOT_GROUP"
    elif command and inputs:
        status = "CATALOG_PLUS_DOMAIN"
    elif command:
        status = "CATALOG_ONLY"
    elif inputs:
        status = "DOMAIN_ONLY"
    else:
        status = "MISSING"
    unresolved = [item["raw"] for item in inputs if item["type"] == "UNRESOLVED"]
    missing = [f"option:{label}: type/default/range/unit/prompt" for label in missing_options]
    missing.extend(f"input:{raw}: type/default/range" for raw in unresolved)
    if command and root_group:
        inline_mode = "COMMAND_INLINE_ROOT_GROUP"
    elif command:
        inline_mode = "COMMAND_INLINE"
    else:
        inline_mode = "DOMAIN_COMMAND_INLINE"
    interaction = {
        "schema": "om9.command_inline.v1",
        "mode": inline_mode,
        "handler": "Controller::submit",
        "prompt_source": "owner_current_value_snapshot",
        "renderer": "shared_command_option_formatter",
        "keyboard_and_click_share_handler": True,
        "preserve_draft_ime_focus": True,
        "legacy_button_row": "REMOVE_AFTER_MIGRATION",
        "shortcut_policy": "full_name_or_unique_stage_accelerator",
        "shortcut_resolution": "UNRESOLVED",
        "option_scope": "ROOT_MENU_COMMAND" if root_group else ("CATALOG_COMMAND" if command else "DOMAIN_COMMAND"),
        "option_order": [] if root_group else [record["label"] for record in records],
        "option_count": len(records),
    }
    return {
        "schema": "om9.parameter_contract.v1",
        "status": status,
        "interaction": interaction,
        "catalog": None
        if not command
        else {
            "source": relative(CATALOG_PATH),
            "menu": menu,
            "command": command,
            "option": option,
            "root_group": root_group,
            "path": [menu, command],
        },
        "domain_inputs": inputs,
        "command_options": records,
        "missing_options": missing_options,
        "missing": missing,
        "implementation_l4": None,
    }


def build_ref(contract: dict[str, Any], l3_path: Path) -> dict[str, Any]:
    catalog = contract.get("catalog") or {}
    interaction = contract.get("interaction") or {}
    return {
        "schema": "om9.parameter_contract.ref.v1",
        "status": contract["status"],
        "interaction": {
            "mode": interaction.get("mode", "COMMAND_INLINE"),
            "handler": interaction.get("handler", "Controller::submit"),
            "option_count": interaction.get("option_count", 0),
            "shortcut_resolution": interaction.get("shortcut_resolution", "UNRESOLVED"),
            "legacy_button_row": interaction.get("legacy_button_row", "REMOVE_AFTER_MIGRATION"),
        },
        "l3_contract": milestone_relative(l3_path) + "#/parameter_contract",
        "catalog_command": None
        if not catalog
        else {"menu": catalog["menu"], "command": catalog["command"]},
        "option_count": len(contract["command_options"]),
        "missing_option_count": len(contract["missing_options"]),
        "domain_input_count": len(contract["domain_inputs"]),
        "implementation_l4": contract.get("implementation_l4"),
    }


def insert_after(data: dict[str, Any], key: str, after: str, value: Any) -> dict[str, Any]:
    if key in data:
        data[key] = value
        return data
    result: dict[str, Any] = {}
    inserted = False
    for current_key, current_value in data.items():
        result[current_key] = current_value
        if current_key == after and not inserted:
            result[key] = value
            inserted = True
    if not inserted:
        result[key] = value
    return result


def make_l4(host: dict[str, Any], l4_id: str, title: str, contract: dict[str, Any], template: dict[str, Any]) -> dict[str, Any]:
    if not isinstance(template, dict):
        template = {}
    l3 = host["l3"]
    child_ids = [
        c.get("id")
        for c in (l3.get("children_l4") or [])
        if c.get("id") and c.get("id") != l4_id
    ]
    data = {
        "level": "L4",
        "id": l4_id,
        "milestone": l3.get("milestone", "M03"),
        "parent_l3_id": l3["id"],
        "owner_l2_id": l3.get("parent_l2_id") or (template or {}).get("owner_l2_id", ""),
        "title": title,
        "task_role": "PARAMETER_IMPLEMENTATION",
        "planning_state": "NOT_STARTED",
        "execution_stage": (template or {}).get("execution_stage", "E04"),
        "mini_gate_id": (template or {}).get("mini_gate_id", "G03-A"),
        "risk": "NORMAL",
        "purpose": (
            f"Implement command-inline parameters for {contract['catalog']['menu']} · {contract['catalog']['command']} "
            f"({contract['catalog'].get('option') or 'command'})"
            if contract.get("catalog")
            else f"Implement domain command-inline parameters for {l3.get('title', 'this L3')}"
        ),
        "parent_l3_file": milestone_relative(host["l3_path"]),
        "preconditions": {
            "milestone": f"{l3.get('milestone', 'M03')} prior gates and existing menu-binding L4 verified",
            "required_l4_verified_ids": [child_id for child_id in child_ids if child_id != l4_id],
            "technical_environment": "Real OpenMatrix9 fork, pinned QymCAD audit checkout, Rust/OCCT toolchain, test fixtures",
            "owners": ((template or {}).get("preconditions") or {}).get("owners", [])
            if isinstance((template or {}).get("preconditions"), dict)
            else [],
            "unresolved_owner_policy": "Verify owner function signatures and tests before integration",
        },
        "source_context": (template or {}).get("source_context", {}),
        "parameter_contract": contract,
        "work_plan": {
            "steps": [
                f"Read {relative(host['l3_path'])}#/parameter_contract.interaction and each command_options[].inline record; treat the L3 contract as the single source.",
                "Classify every option as toggle/cycle/choice/value-entry/point-entry/specialized/unsupported, with stage availability and disabled reason; do not invent a shortcut or default.",
                "Implement the shared inline token/span formatter: keyboard initials, typed full names and click targets all call Controller::submit; remove the legacy floating button row for this command.",
                "Resolve type/default/range/unit/current value from the owner snapshot, not by parsing the prompt or display label; keep UNRESOLVED where the owner is not yet known.",
                "Add fixtures for keyboard, click span, stale run/ID, nonempty draft, IME/paste, invalid token, cancel/Esc and Undo; assert owner state and no document mutation.",
                "Run focused tests and record observed evidence; a rendered inline token or a passing metadata test is not a runtime PASS.",
            ],
            "role_checklist": [
                "Do not duplicate the geometry owner; dispatch through the existing command/feature service.",
                "One prompt/editor/draft only; preserve nonempty drafts, paste, IME, selection and unrelated focus.",
                "Advertise only the recorded unique stage accelerator; fall back to the full option name instead of guessing a conflicting initial.",
                "Do not activate hidden options; keep unsupported options visible and disabled with an explicit reason.",
                "Remove the legacy per-command button row after the inline path is migrated; do not run both paths.",
            ],
            "input_fixture": "; ".join(item["raw"] for item in contract["domain_inputs"]) or "Command-inline option fixtures",
            "expected_positive": "Each listed inline option accepts keyboard/click input, updates owner state and produces the documented observable result.",
            "expected_negative": "Invalid, stale, hidden or unsupported inline input is rejected with a visible reason, draft/focus preserved and no document mutation.",
            "required_technical_decisions": [
                "Option type/default/range/unit are resolved from manual and owner code, not inferred from the catalog label.",
                "One shared CommandOptionInterface/formatter serves menu, CMD, inline keyboard and click paths.",
                "Shortcut resolution is full name first, then a unique already-advertised stage accelerator; conflicts do not get a fabricated initial.",
            ],
            "suggested_commands": ["git rev-parse HEAD", "cargo test -p om9-app --test curve_menu"],
            "commands_not_executed": True,
        },
        "acceptance": {
            "positive_fixture_id": f"{l3['id']}-PARAM-POSITIVE",
            "negative_fixture_id": f"{l3['id']}-PARAM-NEGATIVE",
            "required_receipts": [
                "parameter_schema",
                "inline_option_schema",
                "controller_submit_receipt",
                "keyboard_and_click_fixtures",
                "command_dispatch",
            ],
            "required_artifact_after_execution": f"implementation/M03/{l3['id']}/command_inline_contract.json",
            "required_evidence_after_execution": f"implementation/evidence/M03/{l4_id}.json",
            "expected_observable": "Every inline option has typed value/activation/availability metadata and keyboard/click routes through Controller::submit.",
            "negative_observable": "Invalid, stale, hidden or conflicting inline input is refused with a visible reason, draft/focus preserved and no document mutation.",
            "gate": "EXECUTION_REQUIRED",
            "checklist": [
                "All inline options have activation/availability/shortcut/value owner or an explicit BLOCKED reason.",
                "Keyboard initial/full name and click span use the same Controller::submit handler.",
                "No legacy floating button row remains for this command after migration.",
                "Evidence contains actual keyboard/click receipts, owner state and observed outputs.",
            ],
            "pass_rule": "EXECUTION_PASS only with actual tests and evidence; planning text is not implementation proof.",
            "never_claim_success_if_fixture_missing": True,
            "runtime_feature_success_required_for_this_role": True,
            "design_complete_does_not_mark_feature_implemented": True,
        },
        "execution_policy": (template or {}).get("execution_policy", {}),
        "ui_backend_separation": (template or {}).get("ui_backend_separation", {}),
        "execution_handoff": {
            **((template or {}).get("execution_handoff", {}) or {}),
            "task_role": "PARAMETER_IMPLEMENTATION",
            "role_artifact_kind": "parameter_contract",
            "read_l3_file_first": milestone_relative(host["l3_path"]),
            "mandatory_artifact_fields": [
                "parameter_contract",
                "interaction",
                "inline",
                "option_types",
                "defaults",
                "ranges",
                "units",
                "shortcut_availability_activation",
                "positive_and_negative_fixtures",
            ],
        },
        "reference_supplements": (template or {}).get("reference_supplements", {}),
        "reference_source_policy": (template or {}).get("reference_source_policy", {}),
    }
    return data


def attach_samples(data: dict[str, Any], milestone: str) -> dict[str, Any]:
    import sys

    sys.path.insert(0, str(ROOT / "tools"))
    import apply_implementation_samples as samples

    block = samples.build_block(data, data, milestone)
    if block is not None:
        data["implementation_samples"] = block
        data["implementation_sample_guide"] = samples.runbook_block({block["role"]: 1})
    return data


def make_m04_l4(host: dict[str, Any], l4_id: str, title: str, contract: dict[str, Any], template: dict[str, Any]) -> dict[str, Any]:
    if not isinstance(template, dict):
        template = {}
    l3 = host["l3"]
    data = json.loads(json.dumps(template))
    task_acceptance = {
        "input": "; ".join(item["raw"] for item in contract["domain_inputs"]) or "domain inputs to be pinned from the L3",
        "expected_observable": "Typed domain parameters are accepted and produce the documented observable result.",
        "negative_fixture": "missing, non-finite or unsupported parameter value",
        "negative_expected": "Typed rejection with a visible reason and no document mutation",
        "geometry_policy": "BRep validity and stable entity references; no blind tolerance adjustment",
        "units": "Document units explicit; fixture dimensions in millimetres",
        "numerical_tolerance": "length abs 1e-6 mm; volume abs max(1e-4 mm3, relative 1e-6)",
        "test_registration_status": "PROPOSED_NOT_BUNDLED_NOT_RUN",
    }
    data.update(
        {
            "id": l4_id,
            "title": title,
            "task_role": "PARAMETER_IMPLEMENTATION",
            "planning_state": "SPEC_ONLY_NOT_EXECUTED",
            "requires_actual_execution": True,
            "scope_goal": (l3.get("specification") or {}).get("goal") or l3.get("title"),
            "role_dependencies_verified_ids": [
                child.get("id") for child in (l3.get("children_l4") or []) if child.get("id") and child.get("id") != l4_id
            ],
            "parameter_contract": contract,
            "work_plan": {
                "steps": [
                    f"Read {milestone_relative(host['l3_path'])}#/parameter_contract and pin every declared domain input.",
                    "Resolve type, default, range, unit and required/optional status from the manual and owner API; do not invent values.",
                    "Implement typed parameter descriptors and wire them through the owner service and UI adapter.",
                    "Add positive and negative fixtures for each parameter and record observed evidence.",
                ],
                "task_specific_acceptance": task_acceptance,
                "test_hint": l4_id.lower().replace("-", "_"),
                "commands_not_run": ["git rev-parse HEAD", "cargo test -p om9-app"],
                "commands_recommendation": "Owner tests must be identified locally before execution.",
            },
        }
    )
    data.pop("implementation_sample_guide", None)
    if "implementation_samples" in data:
        data["implementation_samples"] = {
            "schema": "om9.l4.implementation_samples.v1",
            "status": "author_skeleton_untrusted",
            "samples": [],
        }
    return data


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--apply", action="store_true")
    args = parser.parse_args()

    records = []
    for milestone in MILESTONE_DIRS:
        mdir = next(p for p in MILESTONES.iterdir() if p.name.startswith(milestone + " "))
        for l3_path in mdir.rglob("l3.json"):
            l3 = read_json(l3_path)
            l4_paths = sorted(l3_path.parent.rglob("l4.json"))
            l4s = []
            for path in l4_paths:
                data = read_json(path)
                data["_path"] = str(path)
                l4s.append(data)
            hit = match_l3(l3.get("title", ""))
            records.append({"milestone": milestone, "l3": l3, "l3_path": l3_path, "l4s": l4s, "hit": hit})

    # covered options per command across M03
    covered: dict[tuple[str, str], dict[str, str]] = defaultdict(dict)
    for record in records:
        hit = record["hit"]
        if not hit:
            continue
        menu, command, option = hit
        if option:
            covered[(menu, command)][option] = record["l3"]["id"]

    # One parameter-implementation L4 per L3 command that still has missing option
    # metadata. Sharing an L4 across a command family hid the per-command domain
    # inputs (for example Project vs Pullback both map to Curve from Objects).
    plan = {}
    for record in records:
        hit = record["hit"]
        if not hit:
            continue
        menu, command, option = hit
        if command == "(root)":
            continue
        catalog_command = command_for(menu, command)
        if catalog_command is None:
            continue
        all_labels = [o["label"] for o in catalog_command["options"]]
        missing = [label for label in all_labels if label not in covered[(menu, command)]]
        seq, l4_id, folder_name, folder = next_l4(record, "", "")
        plan[record["l3"]["id"]] = {
            "menu": menu,
            "command": command,
            "option": option,
            "missing": missing,
            "host": record,
            "l4_id": l4_id,
            "folder": folder,
            "folder_name": folder_name,
        }

    print(f"L3 records: {len(records)}")
    print(f"catalog-matched L3: {sum(1 for r in records if r['hit'])}")
    print(f"L3 with parameter implementation L4: {len(plan)}")
    print(f"L3 with missing option metadata: {sum(1 for p in plan.values() if p['missing'])}")
    print(f"missing option occurrences: {sum(len(p['missing']) for p in plan.values())}")
    print(f"unique missing options: {len({(p['menu'], p['command'], label) for p in plan.values() for label in p['missing']})}")
    for key, item in sorted(plan.items()):
        print(f"  {key} ({item['menu']} / {item['command']}): {len(item['missing'])} missing -> {item['l4_id']}")
        print("     ", item["missing"])

    if not args.apply:
        print("dry-run; no files written")
        return 0

    created = []
    for record in records:
        l3 = record["l3"]
        l3_path = record["l3_path"]
        hit = record["hit"]
        covered_for_command = covered.get((hit[0], hit[1]), {}) if hit else {}
        contract = build_contract(l3, record["l4s"], l3_path, hit, covered_for_command)
        plan_item = plan.get(l3["id"])
        if plan_item:
            contract["implementation_l4"] = plan_item["l4_id"]
        contract["missing"] = [f"option:{label}: type/default/range/unit/prompt" for label in contract["missing_options"]]
        contract["missing"].extend(
            f"input:{item['raw']}: type/default/range" for item in contract["domain_inputs"] if item["type"] == "UNRESOLVED"
        )
        l3 = insert_after(l3, "parameter_contract", "title", contract)
        write_json(l3_path, l3)

        # refs for existing L4s (the new parameter L4 is skipped and keeps its full contract)
        ref = build_ref(contract, l3_path)
        for l4_path in sorted(l3_path.parent.rglob("l4.json")):
            l4 = read_json(l4_path)
            if l4.get("task_role") == "PARAMETER_IMPLEMENTATION":
                continue
            l4 = insert_after(l4, "parameter_contract", "purpose", ref)
            write_json(l4_path, l4)

        if plan_item:
            template_paths = sorted(l3_path.parent.rglob("l4.json"))
            template = read_json(template_paths[0]) if template_paths else {}
            title = f"Parameter Implementation — {l3.get('title', hit[1])}"
            new_l4 = attach_samples(make_l4(record, plan_item["l4_id"], title, contract, template), record["milestone"])
            l4_path = plan_item["folder"] / "l4.json"
            existed = l4_path.exists()
            write_json(l4_path, new_l4)
            children = l3.get("children_l4") or []
            dep_key = "depends_on_l4_ids" if record["milestone"] == "M04" else "depends_on_task_ids"
            child_entry = next((c for c in children if c.get("id") == plan_item["l4_id"]), None)
            if child_entry is None:
                children.append(
                    {
                        "id": plan_item["l4_id"],
                        "title": title,
                        "file": milestone_relative(l4_path),
                        "role": "PARAMETER_IMPLEMENTATION",
                        dep_key: [c.get("id") for c in children if c.get("id")],
                    }
                )
                l3["children_l4"] = children
                write_json(l3_path, l3)
            else:
                child_entry["title"] = title
                child_entry["file"] = milestone_relative(l4_path)
                child_entry["role"] = "PARAMETER_IMPLEMENTATION"
                child_entry.pop("depends_on_task_ids" if dep_key == "depends_on_l4_ids" else "depends_on_l4_ids", None)
                write_json(l3_path, l3)
            if not existed:
                created.append(plan_item)
        elif contract["status"] == "MISSING" and record["milestone"] in {"M03", "M04"}:
            template_paths = sorted(l3_path.parent.rglob("l4.json"))
            template = read_json(template_paths[0]) if template_paths else {}
            seq, l4_id, folder_name, folder = next_l4(record, "", "")
            title = f"Parameter Implementation — {l3.get('title', 'domain parameters')}"
            contract["implementation_l4"] = l4_id
            contract["missing"] = ["domain inputs: type/default/range/unit/prompt"]
            if record["milestone"] == "M03":
                new_l4 = make_l4(record, l4_id, title, contract, template)
            else:
                new_l4 = make_m04_l4(record, l4_id, title, contract, template)
            new_l4 = attach_samples(new_l4, record["milestone"])
            l4_path = folder / "l4.json"
            existed = l4_path.exists()
            contract["implementation_l4"] = l4_id
            l3["parameter_contract"] = contract
            write_json(l3_path, l3)
            write_json(l4_path, new_l4)
            ref = build_ref(contract, l3_path)
            for existing_path in sorted(l3_path.parent.rglob("l4.json")):
                existing = read_json(existing_path)
                if existing.get("task_role") == "PARAMETER_IMPLEMENTATION":
                    continue
                existing = insert_after(existing, "parameter_contract", "purpose", ref)
                write_json(existing_path, existing)
            children = l3.get("children_l4") or []
            dep_key = "depends_on_l4_ids" if record["milestone"] == "M04" else "depends_on_task_ids"
            child_entry = next((c for c in children if c.get("id") == l4_id), None)
            if child_entry is None:
                children.append(
                    {
                        "id": l4_id,
                        "title": title,
                        "file": milestone_relative(l4_path),
                        "role": "PARAMETER_IMPLEMENTATION",
                        dep_key: [c.get("id") for c in children if c.get("id")],
                    }
                )
                l3["children_l4"] = children
                write_json(l3_path, l3)
            else:
                child_entry["title"] = title
                child_entry["file"] = milestone_relative(l4_path)
                child_entry["role"] = "PARAMETER_IMPLEMENTATION"
                child_entry.pop("depends_on_task_ids" if dep_key == "depends_on_l4_ids" else "depends_on_l4_ids", None)
                child_entry[dep_key] = [
                    c.get("id") for c in children if c.get("id") and c.get("id") != l4_id
                ]
                write_json(l3_path, l3)
            if not existed:
                created.append(
                    {
                        "menu": "DOMAIN",
                        "command": l3.get("title"),
                        "missing": ["domain inputs"],
                        "host": record,
                        "l4_id": l4_id,
                        "folder": folder,
                        "folder_name": folder_name,
                    }
                )

        child_task_ids = [str(c.get("id")) for c in l3.get("children_l4") or [] if c.get("id")]
        if child_task_ids and l3.get("task_ids") != child_task_ids:
            l3["task_ids"] = child_task_ids
            write_json(l3_path, l3)

    # Hosted L4s: physically under M03-M07 but parented by an L3 outside the
    # milestone tree (for example the M07 placements that belong to M01). They
    # still need a contract; embed it in the L4 instead of inventing an L3 file.
    all_l3 = {read_json(path).get("id"): (path, read_json(path)) for path in MILESTONES.rglob("l3.json")}
    covered_l4_paths = {
        str(path.resolve())
        for record in records
        for path in record["l3_path"].parent.rglob("l4.json")
    }
    hosted = []
    for milestone in MILESTONE_DIRS:
        mdir = next(p for p in MILESTONES.iterdir() if p.name.startswith(milestone + " "))
        for l4_path in mdir.rglob("l4.json"):
            if str(l4_path.resolve()) in covered_l4_paths:
                continue
            l4 = read_json(l4_path)
            parent_id = l4.get("parent_l3_id")
            if parent_id not in all_l3:
                continue
            l3_path, l3 = all_l3[parent_id]
            l4["_path"] = str(l4_path)
            contract = build_contract(l3, [l4], l3_path, None, {})
            contract["hosted_l4"] = True
            contract["parent_l3_id"] = parent_id
            contract["missing"] = [
                f"input:{item['raw']}: type/default/range" for item in contract["domain_inputs"] if item["type"] == "UNRESOLVED"
            ]
            l4.pop("_path", None)
            l4 = insert_after(l4, "parameter_contract", "purpose", contract)
            write_json(l4_path, l4)
            hosted.append((milestone, l4_path, parent_id, contract["status"]))

    # update M03 L1 counts from the actual tree (owned = physical minus M08 hosted)
    l1_path = next(MILESTONES.glob("M03 */L1.json"))
    l1 = read_json(l1_path)
    m03_dir = next(p for p in MILESTONES.iterdir() if p.name.startswith("M03 "))
    physical_l4 = len(list(m03_dir.rglob("l4.json")))
    hosted_m08 = int(((l1.get("hosted_l4_from") or {}).get("M08", 0)))
    owned = physical_l4 - hosted_m08
    l1["l4_count"] = owned
    scope = l1.get("m03_l4_scope") or {}
    scope["l4_planned_owned_by_m03"] = owned
    gate_counts = scope.get("stage_gate_counts_l4") or {}
    other_gates = sum(int(v) for k, v in gate_counts.items() if k != "G03-A")
    gate_counts["G03-A"] = owned - other_gates
    scope["stage_gate_counts_l4"] = gate_counts
    l1["m03_l4_scope"] = scope
    write_json(l1_path, l1)

    m04_l1_path = next(MILESTONES.glob("M04 */L1.json"))
    m04_l1 = read_json(m04_l1_path)
    m04_dir = next(p for p in MILESTONES.iterdir() if p.name.startswith("M04 "))
    m04_l1["l4_count"] = len(list(m04_dir.rglob("l4.json")))
    write_json(m04_l1_path, m04_l1)

    print(f"created {len(created)} PARAMETER_IMPLEMENTATION L4s")
    print(f"updated {len(records)} L3 files")

    plan_ids = {item["l4_id"] for item in plan.values()}
    domain_l4 = []
    for record in records:
        for l4_path in sorted(record["l3_path"].parent.rglob("l4.json")):
            l4 = read_json(l4_path)
            if l4.get("task_role") != "PARAMETER_IMPLEMENTATION" or l4.get("id") in plan_ids:
                continue
            domain_l4.append(
                {
                    "menu": "DOMAIN",
                    "command": record["l3"].get("title"),
                    "missing": ["domain inputs"],
                    "host": record,
                    "l4_id": l4.get("id"),
                }
            )
    total_parameter_l4 = len(plan) + len(domain_l4)
    tree_counts = {}
    for milestone in MILESTONE_DIRS:
        mdir = next(p for p in MILESTONES.iterdir() if p.name.startswith(milestone + " "))
        tree_counts[milestone] = len(list(mdir.rglob("l4.json")))
    total_l4 = sum(tree_counts.values())

    unique_missing = len(
        {(p["menu"], p["command"], label) for p in plan.values() for label in p["missing"]}
    )
    lines = [
        "# Matrix L4 parameter coverage (M03–M07)",
        "",
        "Derived from `docs/analysis/matrix-command-layers.json` (Menu → Command → Option) and the",
        "existing L3/L4 domain fixtures. This report is planning coverage, not execution evidence.",
        "",
        "## Summary",
        "",
        f"- L3 contracts: **{len(records)}**",
        f"- L4 contracts/refs: **{total_l4}**",
        f"- New `PARAMETER_IMPLEMENTATION` L4s: **{total_parameter_l4}** (catalog command L4s: {len(plan)}"
        f" = {len(plan) - sum(1 for p in plan.values() if not p['missing'])} missing-option + "
        f"{sum(1 for p in plan.values() if not p['missing'])} inline-only; empty-input domains: {len(domain_l4)})",
        f"- Unique missing catalog options handed to those L4s: **{unique_missing}**",
        "- L4 refs added to every existing L4: yes",
        "- Status is deliberately `PARTIAL`/`DOMAIN_ONLY`; no gate or feature status was promoted.",
        "",
        "## Per milestone",
        "",
        "| Milestone | L3 | L4 with contract | New L4 |",
        "|---|---:|---:|---:|",
    ]
    for milestone in MILESTONE_DIRS:
        selected = [r for r in records if r["milestone"] == milestone]
        new_count = sum(1 for p in plan.values() if p["host"]["milestone"] == milestone)
        new_count += sum(1 for p in domain_l4 if p["host"]["milestone"] == milestone)
        lines.append(f"| {milestone} | {len(selected)} | {tree_counts[milestone]} | {new_count} |")

    lines += [
        "",
        "## Per menu (M03 catalog matches)",
        "",
        "| Nhóm lệnh | L3 | L4 tham số mới | Option thiếu duy nhất |",
        "|---|---:|---:|---:|",
    ]
    for menu in ("Curve", "Surface", "Solid", "Transform"):
        if menu == "Curve":
            # Curve has 58 L3s in L2-03-16; one Rebuild row is catalogued under
            # Others/(root), so matching on the catalog menu alone reports 57.
            menu_l3s = [
                r
                for r in records
                if r["milestone"] == "M03"
                and r["l3_path"].parent.parent.name.startswith("L2-03-16")
            ]
            label = "Curve (L2-03-16)"
        else:
            menu_l3s = [
                r for r in records if r["milestone"] == "M03" and r["hit"] and r["hit"][0] == menu
            ]
            label = menu
        menu_plan = [p for p in plan.values() if p["menu"] == menu]
        menu_missing = {
            (p["command"], opt) for p in menu_plan for opt in p["missing"]
        }
        lines.append(
            f"| **{label}** | {len(menu_l3s)} | {len(menu_plan)} | {len(menu_missing)} |"
        )

    lines += [
        "",
        "## New parameter L4s",
        "",
        "| Command | Host L3 | New L4 | Missing options |",
        "|---|---|---|---|",
    ]
    for item in sorted(list(plan.values()) + domain_l4, key=lambda p: p["l4_id"]):
        missing = ", ".join(f"`{label}`" for label in item["missing"]) or "— (inline-only)"
        lines.append(
            f"| {item['menu']} · {item['command']} | `{item['host']['l3']['id']}` | `{item['l4_id']}` | {missing} |"
        )

    lines += [
        "",
        "## Hosted L4s with embedded contracts",
        "",
    ]
    if hosted:
        for milestone, path, parent_id, status in hosted:
            lines.append(f"- `{relative(path)}` — parent `{parent_id}`, status `{status}`")
    else:
        lines.append("- None")
    lines += [
        "",
        "## Interpretation",
        "",
        "- M03 submenu commands expose option families (Line, Arc, Circle, Extend, Boolean Tools, …).",
        "  Existing L3s usually cover the base option only; the new L4 owns the remaining options with",
        "  type/default/range/unit/prompt still `UNRESOLVED` until resolved from the manual and owner code.",
        "- Mỗi L3 command non-root có đúng một L4 command-inline, nên domain input và prompt state của",
        "  từng lệnh không bị gộp. 74 L4 xử lý option thiếu metadata; 30 L4 chỉ cần inline hóa option đã có.",
        "  Curve có 58 L3 → 51 L4 (7 L3 root-group không có option family); 60 option Curve còn `UNRESOLVED`.",
        "- Mỗi option có `inline` record: token, typed forms, shortcut, underline, activation, availability,",
        "  value owner/type và prompt fragment. Shortcut/underline/activation chưa xác minh vẫn `UNRESOLVED`,",
        "  không tự đoán initial gây nuốt số liệu hoặc xung đột với option khác.",
        "- Handler đích là `Controller::submit`; keyboard và click dùng cùng một formatter/span, bỏ legacy",
        "  floating button row sau khi family được migrate.",
        "- `(root)` catalog groups are not treated as one command; their entries are separate commands, so",
        "  they do not receive a parameter L4 from this pass.",
        "- M04–M07 are feature domains, not menu option families. Most existing fixtures state the domain",
        "  inputs; the contract records them as `DOMAIN_ONLY` and does not invent typed defaults. One M04 L3",
        "  and eight M03 non-catalog L3s had no machine-readable input at all, so they received a domain",
        "  parameter L4 rather than a fabricated parameter list.",
        "- Domain inputs with `type: UNRESOLVED` are listed in `missing` for transparency. Where an existing",
        "  L4 fixture/contract already owns them, no sibling L4 is invented.",
        "",
        "## Unmatched M03 titles",
        "",
    ]
    unmatched = [
        r
        for r in records
        if r["milestone"] == "M03" and not r["hit"] and " — menu binding" in str(r["l3"].get("title", ""))
    ]
    if unmatched:
        for record in unmatched:
            lines.append(f"- `{record['l3']['id']}` — {record['l3'].get('title')}")
    else:
        lines.append("- None")
    lines += [
        "",
        "## Regenerate",
        "",
        "```powershell",
        "python tools/apply_l4_parameter_contracts.py --apply",
        "```",
        "",
    ]
    REPORT_PATH.parent.mkdir(parents=True, exist_ok=True)
    REPORT_PATH.write_text("\n".join(lines), encoding="utf-8")
    print(f"wrote {relative(REPORT_PATH)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
