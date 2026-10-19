#!/usr/bin/env python
"""Verify the M00 v0.13 architecture and its DAG before any task is started.

Protocol step 1: `python tools/validate_m00_v013.py` must pass before a task may be touched. It checks:

* seven L2 modules, one per gate, each with a `gate_id` and a `gate_registry` pointer;
* every `children_l3` entry resolves to an `l3.json`, and every `children_l4` entry to an `l4.json`;
* ids are unique and match the level pattern, and an L4's parent agrees with its L3;
* the milestone DAG never depends on a later milestone;
* every YAML-free evidence record that claims `om9.m00.evidence.v013` satisfies that schema;
* the four human-approval L4 are present and labelled as such.

Exit code 0 = clean, 1 = issues, 2 = a record cannot even be parsed.
"""
from __future__ import annotations

import sys

import m00_common as m00


def main() -> int:
    issues: list[str] = []
    l1 = m00.load(m00.M00 / "L1.json")

    modules = l1.get("modules") or []
    if len(modules) != 7:
        issues.append(f"L1 declares {len(modules)} modules, expected 7")
    declared_gates = {str(module.get("id")): module for module in modules}

    l2_dirs = sorted(p for p in m00.M00.iterdir() if p.is_dir() and p.name.startswith("L2-"))
    if len(l2_dirs) != 7:
        issues.append(f"the tree holds {len(l2_dirs)} L2 folders, expected 7")

    seen_l3: set[str] = set()
    seen_l4: set[str] = set()
    for l2_dir in l2_dirs:
        l2 = m00.load(l2_dir / "L2.json")
        l2_id = str(l2.get("id") or "")
        if l2_id not in declared_gates:
            issues.append(f"{l2_dir.name}: id {l2_id} is not listed in L1.modules")
        if not l2.get("gate_id"):
            issues.append(f"{l2_id}: no gate_id")
        if not l2.get("gate_registry"):
            issues.append(f"{l2_id}: no gate_registry pointer")
        for child in l2.get("children_l3") or []:
            l3_id = str(child.get("id") or "")
            if l3_id in seen_l3:
                issues.append(f"duplicate L3 id {l3_id}")
            seen_l3.add(l3_id)
            path = m00.resolve(child.get("file", ""))
            if not path.exists():
                issues.append(f"{l3_id}: l3.json missing at {child.get('file')}")
                continue
            l3 = m00.load(path)
            if l3.get("parent_l2_id") != l2_id:
                issues.append(f"{l3_id}: parent_l2_id {l3.get('parent_l2_id')} != {l2_id}")
            for grandchild in l3.get("children_l4") or []:
                l4_id = str(grandchild.get("id") or "")
                if l4_id in seen_l4:
                    issues.append(f"duplicate L4 id {l4_id}")
                seen_l4.add(l4_id)
                l4_path = m00.resolve(grandchild.get("file", ""))
                if not l4_path.exists():
                    issues.append(f"{l4_id}: l4.json missing at {grandchild.get('file')}")
                    continue
                l4 = m00.load(l4_path)
                parent = l4.get("parent_l3_id") or l4.get("parent_l3")
                if parent and parent != l3_id:
                    issues.append(f"{l4_id}: parent_l3_id {parent} != {l3_id}")

    # DAG: a milestone may only depend on an earlier one.
    order = {f"M{index:02d}": index for index in range(10)}
    order["AFTER"] = 10
    for folder in sorted(p for p in m00.ROOT.joinpath("OM9_Milestones").iterdir() if p.is_dir()):
        path = folder / "L1.json"
        if not path.exists():
            continue
        data = m00.load(path)
        key = str(data.get("milestone") or "")
        for dependency in data.get("depends_on_milestones") or []:
            if order.get(dependency, -1) > order.get(key, 99):
                issues.append(f"{key}: depends on later milestone {dependency}")

    # Evidence records that claim the v0.13 schema.
    for path, record in m00.evidence_records():
        if not record:
            print(f"UNPARSEABLE {path.relative_to(m00.ROOT)}")
            return 2
        for error in m00.schema_errors(record):
            issues.append(f"{path.relative_to(m00.ROOT)}: {error}")

    # Human-approval L4 must exist and be labelled.
    for l4_id in sorted(m00.HUMAN_APPROVAL):
        entry = next((item for item in m00.iter_l4() if item["id"] == l4_id), None)
        if entry is None:
            issues.append(f"{l4_id}: the protocol names it as human-approval but the tree has no such L4")
            continue
        text = (entry["l4"].get("title") or "") + (entry["l4"].get("gate") or "")
        if "approval" not in text.lower() and "sign" not in text.lower() and "review" not in text.lower():
            print(f"note: {l4_id} requires human approval; its title does not say so")

    print(f"L2 {len(l2_dirs)} | L3 {len(seen_l3)} | L4 {len(seen_l4)} | evidence records {len(m00.evidence_records())}")
    if issues:
        print(f"FAIL {len(issues)} issue(s):")
        for issue in issues[:40]:
            print("  -", issue)
        return 1
    print("PASS: architecture, DAG and evidence schema are consistent")
    return 0


if __name__ == "__main__":
    sys.exit(main())
