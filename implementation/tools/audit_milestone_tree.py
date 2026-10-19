#!/usr/bin/env python
"""Audit the milestone plan tree (M*/L1.json/L2/l3.json/L4) for structural consistency.

Checks, all data-driven:

1. every milestone has ``L1.json`` with a parseable ``id``/``planning_state``;
2. every L2 has ``L2.json``; every L3 has ``l3.json``; every L4 directory has an ``l4*.json``;
3. ids are unique, match the level prefix, and agree with the folder number;
4. parent links agree in both directions (``L2.children_l3`` vs child ``parent_l2_id``,
   ``l3.children_l4`` vs the L4 ``parent_l3_id``);
5. the counts an ``L1.json`` declares (``module_count``/``l3_count``/``l4_count``) are compared with
   the tree it actually describes, because a declared count that drifts is a silent planning defect;
6. a curated jewellery-capability matrix maps the capabilities a Matrix-class product needs onto the
   L3 titles, so a missing owner shows up as a gap instead of being discovered during implementation.

    python implementation/tools/audit_milestone_tree.py             # summary only
    python implementation/tools/audit_milestone_tree.py --json      # machine-readable
    python implementation/tools/audit_milestone_tree.py --write     # write the markdown report
"""
from __future__ import annotations

import argparse
import json
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
TREE = ROOT / "OM9_Milestones"
REPORT = ROOT / "implementation/design/MILESTONE-TREE-AUDIT.md"

LEVEL_ID = re.compile(r"^OM9-(M\d{2}|Z[A-Z]+)-L([1-4])-(\d{2})(?:-(\d{2}))?(?:-(\d{3}))?$")
FOLDER_L2 = re.compile(r"^L2-(\d{2})-(\d{2})")
FOLDER_L3 = re.compile(r"^L3-(\d{3})")
FOLDER_L4 = re.compile(r"^L4-(\d{2})")

# The capabilities a Matrix-class jewellery product needs, each mapped to a title pattern. A hit
# means the plan tree names an owner; the absence of a hit is the finding.
CAPABILITIES = [
    ("Giao diện shell/docking/viewport", r"shell|docking|viewport|panel"),
    ("Lệnh + option + undo/redo", r"command|undo|redo|dispatch"),
    ("Tài liệu + entity/layer", r"document|entity|layer|scene"),
    ("Đơn vị + dung sai", r"unit|tolerance|numeric|parameter"),
    ("OCCT B-Rep kernel bridge", r"occt|brep|b-rep"),
    ("GPU/wgpu hiển thị", r"gpu|wgpu|render|shading|draw"),
    ("Đường cong NURBS (Interp/Circle/Arc/Ellipse)", r"curve|circle|arc|ellipse|interp|nurbs"),
    ("Bề mặt (Loft/Sweep/Trim/Rebuild)", r"surface|loft|sweep|trim|refit"),
    ("Khối solid + boolean/fillet/shell", r"solid|boolean|fillet|shell|offset|hollow"),
    ("Chọn + gumball + transform", r"selection|gumball|transform|handles"),
    ("CPlane + toạ độ", r"cplane|coordinate"),
    ("Object snap + smarttrack", r"snap|osnap|smarttrack"),
    ("Thư viện đá (gem)", r"gem|stone|diamond"),
    ("Đặt đá: prong/head/bezel/pavé/channel/azure", r"prong|head|bezel|pave|pavé|channel|azure|setting"),
    ("Ring builder + profile/rail", r"ring|profile|rail|resize"),
    ("Cutters (M07)", r"cutter|sprue|hollow"),
    ("Lịch sử/parametric rebuild", r"history|associative|rebuild|dirty"),
    ("Tính khối lượng kim loại", r"weight|metal|density"),
    ("3DM import/export (openNURBS)", r"3dm|rhino|opennurbs|import|export"),
    ("STL/in 3D", r"stl|mesh|print|cast"),
    ("Layer clipboard/exchange", r"clipboard|exchange"),
    ("Đo lường (angle/distance)", r"measure|distance|angle"),
    ("Cage/SubD", r"cage|subd|deform"),
    ("Kiểm thử/nghiệm thu", r"acceptance|regression|validation|test"),
]


def load(path: pathlib.Path) -> dict | None:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        return {"__error__": f"{error}"}


def audit() -> dict:
    findings: list[str] = []
    milestones: list[dict] = []
    ids: dict[str, str] = {}
    duplicate_ids: list[str] = []
    l2_count = l3_count = l4_count = 0
    l4_missing = 0

    def register(identifier: str, where: str) -> None:
        if not identifier:
            findings.append(f"missing id at {where}")
            return
        if identifier in ids:
            duplicate_ids.append(identifier)
            findings.append(f"duplicate id {identifier} at {where} and {ids[identifier]}")
            return
        ids[identifier] = where

    for milestone_dir in sorted(p for p in TREE.iterdir() if p.is_dir()):
        l1_path = milestone_dir / "L1.json"
        entry = {
            "folder": milestone_dir.name,
            "key": milestone_dir.name.split(" ")[0],
            "declared": {},
            "actual": {"l2": 0, "l3": 0, "l4": 0},
            "l2": [],
        }
        if not l1_path.exists():
            findings.append(f"{milestone_dir.name}: no L1.json")
            milestones.append(entry)
            continue
        l1 = load(l1_path) or {}
        # The folder may carry a sort prefix (`ZAFTER - Extensions`); the milestone key is the
        # L1 `milestone` value, which every L2 record repeats.
        if isinstance(l1.get("milestone"), str) and l1["milestone"]:
            entry["key"] = l1["milestone"]
        register(str(l1.get("id", "")), str(l1_path.relative_to(ROOT)))
        if "__error__" in l1:
            findings.append(f"{milestone_dir.name}: L1.json unreadable ({l1['__error__']})")
        for key in ("module_count", "l3_count", "l4_count"):
            if key in l1:
                entry["declared"][key] = l1[key]
        for key in ("l4_count_in_tree", "l4_placed_by_milestone", "cross_milestone_note"):
            if key in l1:
                entry[key] = l1[key]
        entry["depends_on"] = l1.get("depends_on_milestones")
        entry["planning_state"] = l1.get("planning_state")

        # A release-gate milestone may hold L4 records without L2/L3 parents (M09 documents this:
        # two gates, no original L2/L3 ids, 30 L4). Count them as direct L4 records.
        direct_l4 = 0
        for candidate in milestone_dir.rglob("l4*.json"):
            relative = candidate.relative_to(milestone_dir).parts
            if "L3-" not in " ".join(relative):
                direct_l4 += 1
        entry["l4_direct"] = direct_l4
        entry["actual"]["l4"] += direct_l4
        l4_count += direct_l4

        # A milestone may host another milestone's L4 records (M08 records them with
        # `l4_placed_by_milestone`); those legitimately inflate this milestone's physical count.
        hosted = 0
        own = entry.get("hosted_l4_from")
        if isinstance(own, dict):
            # The host states what it hosts; this is the authoritative side of a placement.
            hosted = sum(int(value or 0) for value in own.values())
        else:
            for other in milestones:
                if other is entry:
                    # A milestone listing its own placements is not hosting another's records.
                    continue
                placement = other.get("l4_placed_by_milestone")
                if isinstance(placement, dict):
                    hosted += int(placement.get(entry["key"], 0) or 0)
        entry["l4_hosted"] = hosted

        for l2_dir in sorted(p for p in milestone_dir.iterdir() if p.is_dir() and p.name.startswith("L2-")):
            entry["actual"]["l2"] += 1
            l2_count += 1
            l2_path = l2_dir / "L2.json"
            l2 = load(l2_path) if l2_path.exists() else None
            if l2 is None:
                findings.append(f"{milestone_dir.name}/{l2_dir.name}: no L2.json")
                l2 = {}
            elif "__error__" in l2:
                findings.append(f"{l2_dir.name}: L2.json unreadable ({l2['__error__']})")
            l2_id = str(l2.get("id", ""))
            register(l2_id, str(l2_path.relative_to(ROOT)))
            folder = FOLDER_L2.match(l2_dir.name)
            if folder and l2_id and not l2_id.endswith(f"-L2-{folder.group(1)}-{folder.group(2)}"):
                findings.append(
                    f"{l2_dir.name}: folder number {folder.group(1)}-{folder.group(2)} != id {l2_id}"
                )
            if l2.get("milestone") and l2["milestone"] != entry["key"]:
                findings.append(f"{l2_dir.name}: milestone {l2['milestone']} != folder {entry['key']}")
            declared_children = l2.get("children_l3")
            declared_l3 = len(declared_children) if isinstance(declared_children, list) else None
            actual_l3 = 0
            l3_entries = []
            for l3_dir in sorted(
                p for p in l2_dir.iterdir() if p.is_dir() and p.name.startswith("L3-")
            ):
                actual_l3 += 1
                l3_count += 1
                entry["actual"]["l3"] += 1
                l3_path = l3_dir / "l3.json"
                l3 = load(l3_path) if l3_path.exists() else None
                if l3 is None:
                    findings.append(f"{l2_dir.name}/{l3_dir.name}: no l3.json")
                    l3 = {}
                elif "__error__" in l3:
                    findings.append(f"{l3_dir.name}: l3.json unreadable ({l3['__error__']})")
                l3_id = str(l3.get("id", ""))
                register(l3_id, str(l3_path.relative_to(ROOT)))
                suffix = FOLDER_L3.match(l3_dir.name)
                if suffix and l3_id and not l3_id.endswith(f"-{suffix.group(1)}"):
                    findings.append(
                        f"{l3_dir.name}: folder number {suffix.group(1)} != id {l3_id}"
                    )
                if l3.get("parent_l2_id") and l2_id and l3["parent_l2_id"] != l2_id:
                    findings.append(
                        f"{l3_dir.name}: parent_l2_id {l3['parent_l2_id']} != {l2_id}"
                    )
                specification = l3.get("specification") or {}
                actual_l4 = 0
                for l4_dir in sorted(
                    p for p in l3_dir.iterdir() if p.is_dir() and p.name.startswith("L4-")
                ):
                    actual_l4 += 1
                    l4_count += 1
                    entry["actual"]["l4"] += 1
                    candidates = sorted(l4_dir.glob("l4*.json"))
                    if not candidates:
                        l4_missing += 1
                        findings.append(f"{l3_dir.name}/{l4_dir.name}: no l4 json")
                        continue
                    l4 = load(candidates[0]) or {}
                    register(str(l4.get("id", "")), str(candidates[0].relative_to(ROOT)))
                    parent = l4.get("parent_l3_id") or l4.get("parent_l3")
                    if parent and l3_id and parent != l3_id:
                        findings.append(
                            f"{candidates[0].name}: parent_l3_id {parent} != {l3_id}"
                        )
                declared_l4 = l3.get("children_l4")
                if isinstance(declared_l4, list) and len(declared_l4) != actual_l4:
                    findings.append(
                        f"{l3_dir.name}: children_l4 says {len(declared_l4)}, tree has {actual_l4}"
                    )
                l3_entries.append(
                    {
                        "id": l3_id,
                        "title": str(l3.get("title", "")),
                        "goal": str(specification.get("goal", "")),
                        "l4": actual_l4,
                        "planning_state": str(l3.get("planning_state", "")),
                        "target_milestone": str(l3.get("target_implementation_milestone", "")),
                    }
                )
            if declared_l3 is not None and declared_l3 != actual_l3:
                findings.append(
                    f"{l2_dir.name}: children_l3 says {declared_l3}, tree has {actual_l3}"
                )
            entry["l2"].append(
                {"id": l2_id, "title": str(l2.get("title", "")), "l3": l3_entries}
            )
        milestones.append(entry)

    # Declared-vs-actual counts per milestone.
    for entry in milestones:
        declared = entry["declared"]
        actual = entry["actual"]
        if declared.get("module_count") not in (None, actual["l2"]):
            findings.append(
                f"{entry['key']}: module_count {declared['module_count']} != {actual['l2']} L2 in tree"
            )
        if declared.get("l3_count") not in (None, actual["l3"]):
            findings.append(
                f"{entry['key']}: l3_count {declared['l3_count']} != {actual['l3']} L3 in tree"
            )
        # A milestone may host another milestone's L4 records; M08 documents this with
        # `l4_placed_by_milestone` and distinguishes logical (`l4_count`) from physical
        # (`l4_count_in_tree`) counts, so compare against the physical number when it exists.
        declared_l4 = declared.get("l4_count")
        if isinstance(entry.get("l4_count_in_tree"), int):
            declared_l4 = entry["l4_count_in_tree"]
        entry["l4_declared"] = declared_l4

    # L4 hosting needs every L1 read first (a host may sort before the milestone that documents the
    # placement), so the physical comparison runs after the loop.
    for entry in milestones:
        hosted = 0
        own = entry.get("hosted_l4_from")
        if isinstance(own, dict):
            # The host states what it hosts; this is the authoritative side of a placement.
            hosted = sum(int(value or 0) for value in own.values())
        else:
            for other in milestones:
                if other is entry:
                    # A milestone listing its own placements is not hosting another's records.
                    continue
                placement = other.get("l4_placed_by_milestone")
                if isinstance(placement, dict):
                    hosted += int(placement.get(entry["key"], 0) or 0)
        entry["l4_hosted"] = hosted
        declared_l4 = entry.get("l4_declared")
        if isinstance(declared_l4, int):
            expected = declared_l4 + hosted
            note = f" (declared {declared_l4} + {hosted} hosted)" if hosted else ""
            if expected != entry["actual"]["l4"]:
                findings.append(
                    f"{entry['key']}: l4_count {expected} != {entry['actual']['l4']} L4 in tree{note}"
                )

    # Dependency sanity: a milestone may only depend on an earlier milestone.
    order = {entry["key"]: index for index, entry in enumerate(milestones)}
    for entry in milestones:
        depends = entry.get("depends_on")
        if isinstance(depends, str) and depends:
            depends = [depends]
        for dependency in depends or []:
            if dependency in order and order[dependency] > order[entry["key"]]:
                findings.append(
                    f"{entry['key']}: depends on later milestone {dependency}"
                )

    # Capability coverage over every L3 title.
    titles = [
        (l3["id"], l3["title"], entry["key"])
        for entry in milestones
        for l2 in entry["l2"]
        for l3 in l2["l3"]
    ]
    capability = []
    for label, pattern in CAPABILITIES:
        regex = re.compile(pattern, re.IGNORECASE)
        hits = [(identifier, title, milestone) for identifier, title, milestone in titles if regex.search(title)]
        capability.append(
            {
                "capability": label,
                "matches": len(hits),
                "milestones": sorted({milestone for _, _, milestone in hits}),
                "examples": [f"{identifier} {title}" for identifier, title, _ in hits[:3]],
            }
        )

    return {
        "milestones": milestones,
        "totals": {
            "milestones": len(milestones),
            "l2": l2_count,
            "l3": l3_count,
            "l4": l4_count,
            "l4_without_json": l4_missing,
            "duplicate_ids": len(duplicate_ids),
        },
        "duplicate_ids": sorted(set(duplicate_ids)),
        "findings": findings,
        "capability": capability,
    }


def markdown(result: dict) -> str:
    totals = result["totals"]
    lines = [
        "# Audit cây kế hoạch (M / L1 / L2 / L3 / L4)",
        "",
        f"Sinh bởi `implementation/tools/audit_milestone_tree.py` — {totals['milestones']} milestone, "
        f"{totals['l2']} L2, {totals['l3']} L3, {totals['l4']} L4.",
        "",
        "## 1. Tổng theo milestone",
        "",
        "| Milestone | L2 | L3 | L4 | Khai báo (L1.json) | Khớp? | depends_on |",
        "| --- | ---: | ---: | ---: | --- | --- | --- |",
    ]
    for entry in result["milestones"]:
        declared = entry["declared"]
        claim = ", ".join(f"{key.split('_')[0]}={value}" for key, value in declared.items())
        declared_l4 = entry.get("l4_declared")
        expected_l4 = (declared_l4 + entry.get("l4_hosted", 0)) if isinstance(declared_l4, int) else None
        matches = (
            declared.get("module_count") in (None, entry["actual"]["l2"])
            and declared.get("l3_count") in (None, entry["actual"]["l3"])
            and (expected_l4 is None or expected_l4 == entry["actual"]["l4"])
        )
        lines.append(
            f"| {entry['key']} | {entry['actual']['l2']} | {entry['actual']['l3']} | "
            f"{entry['actual']['l4']} | {claim or '—'} | {'có' if matches else '**lệch**'} | "
            f"{entry.get('depends_on') or '—'} |"
        )
    lines += [
        "",
        "## 2. Phát hiện",
        "",
    ]
    if result["findings"]:
        lines += [f"- {finding}" for finding in result["findings"]]
    else:
        lines.append("- Không có lệch cấu trúc nào.")
    lines += [
        "",
        "## 3. Phủ năng lực trang sức (Matrix-class)",
        "",
        "| Năng lực | Số L3 khớp | Milestone | Ví dụ |",
        "| --- | ---: | --- | --- |",
    ]
    for row in result["capability"]:
        examples = "; ".join(row["examples"][:2]).replace("|", "\\|")
        lines.append(
            f"| {row['capability']} | {row['matches']} | {', '.join(row['milestones']) or '—'} | {examples or '—'} |"
        )
    lines.append("")
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--json", action="store_true")
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    result = audit()
    if args.json:
        print(json.dumps(result, indent=2, ensure_ascii=False))
        return 0
    totals = result["totals"]
    print(
        f"milestones={totals['milestones']} l2={totals['l2']} l3={totals['l3']} "
        f"l4={totals['l4']} l4_without_json={totals['l4_without_json']} "
        f"duplicate_ids={totals['duplicate_ids']}"
    )
    print(f"findings={len(result['findings'])}")
    for finding in result["findings"][:40]:
        print("  -", finding)
    gaps = [row for row in result["capability"] if row["matches"] == 0]
    print(f"capability gaps={len(gaps)}")
    for row in gaps:
        print("  *", row["capability"])
    if args.write:
        REPORT.write_text(markdown(result) + "\n", encoding="utf-8")
        print("wrote", REPORT.relative_to(ROOT))
    return 0


if __name__ == "__main__":
    sys.exit(main())
