"""Merge an execution-hardened milestone package into the current milestone tree.

Conflict-safe: for every L1/L2/L3 file the package also ships, the merge keeps
the local record and adopts only the incoming L4-planning members. Any other
difference is reported as a conflict and that file is left untouched. Every L4
record the package ships is added; files the package does not ship are kept.

Writes a manifest so every change can be reviewed or reverted.
"""
import argparse
import json
import re
from pathlib import Path

ROOT = Path("H:/OpenMatrix9")
MILESTONES = ROOT / "OM9_Milestones"

# Per-milestone package definitions: field names differ between packages
# (M04 uses m04_execution_handoff/planned_l4_count_m04, M03 uses
# m03_luna_execution_contract/m03_l4_planned_count).
PACKAGES = {
    "m04": {
        "milestone_dir": "M04 - Builder, History and Ring",
        "incoming": "build/m04-incoming",
        "manifest": "build/m04-merge-manifest.json",
        "l1": ["l4_count", "execution_readiness", "planning_state"],
        "l2": ["l4_status", "planned_l4_count_m04", "planning_state", "execution_state",
               "readiness_state"],
        "l3": ["children_l4", "l4_status", "planning_state", "m04_execution_handoff",
               "task_ids", "readiness_state", "planning_handoff"],
        "l3_spec": ["status", "l4_status", "execution_status"],
        "l2_definition": ["l3_l4_status"],
    },
    "m03": {
        "milestone_dir": "M03 - Curve, Surface, Solid and 3DM Exchange",
        "incoming": "build/m03-incoming",
        "manifest": "build/m03-merge-manifest.json",
        "l1": ["l4_count", "planning_state", "note", "m03_l4_scope", "m03_execution_hardening"],
        # planned_l4_count_m08 / l4_count_planned_m08: the E13 L4 that
        # tools/merge_l4_overlay.py --package m08 places under this module for the L3
        # the M03 package defers to M08; the deferred module is no longer spec-only.
        "l2": ["m03_l4_planned_count", "execution_state", "readiness_state", "planning_state",
               "l4_status", "planned_l4_count_m08"],
        "l3": ["children_l4", "l4_status", "planning_state", "task_ids",
               "m03_luna_execution_contract", "readiness_state", "acceptance_category",
               "planning_handoff"],
        "l3_spec": ["status", "l4_status", "execution_status", "l4_count_planned_m03"],
        "l2_definition": ["l3_l4_status", "l3_deferred_count", "l3_m03_owner_count",
                          "l4_count_planned_m03", "l4_count_planned_m08"],
    },
}


def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def write(path, data):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def find_milestone_dir(name):
    for path in MILESTONES.iterdir():
        if path.is_dir() and path.name.startswith(name.split(" ")[0] + " "):
            return path
    raise SystemExit(f"milestone dir not found: {name}")


def merge_record(kind, spec, current, incoming):
    """Return (merged, adopted_fields, conflicts) for a planning record."""
    merged = json.loads(json.dumps(current))  # deep copy
    adopted, conflicts = [], []

    for field in spec[kind]:
        cur_has, inc_has = field in merged, field in incoming
        if inc_has and (not cur_has or merged[field] != incoming[field]):
            merged[field] = incoming[field]
            adopted.append(field)
        elif cur_has and not inc_has:
            conflicts.append(f"{field}: present locally, absent in package")

    nested = {"l3": ("specification", spec["l3_spec"]),
              "l2": ("definition", spec["l2_definition"])}
    if kind in nested:
        container, fields = nested[kind]
        if container in incoming:
            target = merged.setdefault(container, {})
            for field in fields:
                if field in incoming[container]:
                    if target.get(field) != incoming[container][field]:
                        target[field] = incoming[container][field]
                        adopted.append(f"{container}.{field}")

    # Everything else must be identical; only the order of planning members may differ.
    def comparable(data):
        out = json.loads(json.dumps(data))
        for field in spec[kind]:
            out.pop(field, None)
        for container, fields in nested.values():
            node = out.get(container)
            if isinstance(node, dict):
                for field in fields:
                    node.pop(field, None)
        return out

    left, right = comparable(merged), comparable(incoming)
    if left != right:
        for key in sorted(set(left) | set(right)):
            if left.get(key) != right.get(key):
                if isinstance(left.get(key), dict) and isinstance(right.get(key), dict):
                    for sub in sorted(set(left[key]) | set(right[key])):
                        if left[key].get(sub) != right[key].get(sub):
                            conflicts.append(f"{key}.{sub}")
                else:
                    conflicts.append(key)
    return merged, adopted, conflicts


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--package", required=True, choices=sorted(PACKAGES))
    parser.add_argument("--apply", action="store_true")
    args = parser.parse_args()

    spec = PACKAGES[args.package]
    currentRoot = find_milestone_dir(spec["milestone_dir"])
    incomingRoot = ROOT / spec["incoming"] / "OpenMatrix9" / spec["milestone_dir"]
    manifestPath = ROOT / spec["manifest"]
    if not incomingRoot.is_dir():
        raise SystemExit(f"incoming package not extracted: {incomingRoot}")

    manifest = {
        "schema": f"om9.{args.package}.merge_manifest.v1",
        "package": args.package,
        "strategy": "local L1/L2/L3 win; adopt incoming L4-planning members only; add every incoming L4 record",
        "l4_added": [],
        "planning_updated": [],
        "unchanged": [],
        "conflicts": [],
    }

    for path in sorted(incomingRoot.rglob("*.json")):
        rel = path.relative_to(incomingRoot).as_posix()
        target = currentRoot / rel
        name = path.name
        if name == "l4.json":
            if target.exists():
                if target.read_bytes() != path.read_bytes():
                    manifest["conflicts"].append(
                        f"{rel}: existing local L4 differs from package L4")
                else:
                    manifest["unchanged"].append(rel)
                continue
            manifest["l4_added"].append(rel)
            if args.apply:
                write(target, read(path))
            continue

        kind = {"L1.json": "l1", "L2.json": "l2", "l3.json": "l3"}.get(name)
        if kind is None:
            manifest["conflicts"].append(f"{rel}: unrecognised file name")
            continue
        if not target.exists():
            manifest["conflicts"].append(f"{rel}: package file has no local counterpart")
            continue

        merged, adopted, conflicts = merge_record(kind, spec, read(target), read(path))
        if conflicts:
            manifest["conflicts"].append({"file": rel, "fields": conflicts})
            continue
        if not adopted:
            manifest["unchanged"].append(rel)
            continue
        manifest["planning_updated"].append({"file": rel, "adopted": adopted})
        if args.apply:
            write(target, merged)

    shipped = {p.relative_to(incomingRoot).as_posix() for p in incomingRoot.rglob("*") if p.is_file()}
    manifest["local_only_files_kept"] = sum(
        1 for p in currentRoot.rglob("*")
        if p.is_file() and p.relative_to(currentRoot).as_posix() not in shipped)

    manifestPath.parent.mkdir(parents=True, exist_ok=True)
    manifestPath.write_text(json.dumps(manifest, ensure_ascii=False, indent=1) + "\n",
                            encoding="utf-8")
    print(json.dumps({
        "package": args.package,
        "apply": args.apply,
        "l4_added": len(manifest["l4_added"]),
        "planning_updated": len(manifest["planning_updated"]),
        "unchanged": len(manifest["unchanged"]),
        "conflicts": len(manifest["conflicts"]),
        "local_only_files_kept": manifest["local_only_files_kept"],
    }, indent=1))
    for entry in manifest["conflicts"][:10]:
        print("CONFLICT:", entry)


if __name__ == "__main__":
    main()
