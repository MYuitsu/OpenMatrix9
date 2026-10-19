"""Verify a merged milestone package: merged tree == package + anchors/planning.

Usage: python tools/verify_merge.py <package>   (m03 | m04)
"""
import json
import sys
from pathlib import Path

ROOT = Path("H:/OpenMatrix9")
sys.path.insert(0, str(ROOT / "tools"))
import apply_matrix_ref_supplements as mod  # noqa: E402
import merge_milestone_package as mg  # noqa: E402

KIND = {"L1.json": "l1", "L2.json": "l2", "l3.json": "l3"}
# Authored enrichment applied on top of the shipped package tree. These members are
# not part of the incoming body, so they are excluded when proving merge fidelity.
ENRICHMENT = {"implementation_samples", "implementation_sample_guide"}
# The M08 package (tools/merge_l4_overlay.py --package m08) executes the L3 this
# package defers to E13. Those L3 legitimately gain children_l4 and planning state
# from that package, so the merge proof checks the contribution instead of
# demanding the shipped "deferred, not planned here" body.
M08_DEFERRAL = "DEFERRED_TO_M08"
M08_L3 = ROOT / "OM9_Milestones" / "M08 - Jewelry Modeling"
M08_MANIFEST = ROOT / "OM9_Milestones" / "M08 - Jewelry Modeling" / \
    "reference" / "m08-l4" / "M08_L4_EXECUTION_MANIFEST.json"


def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def strip_planning(spec, kind, data):
    out = json.loads(json.dumps(data))
    for field in spec[kind]:
        out.pop(field, None)
    for container, fields in {"l3": ("specification", spec["l3_spec"]),
                              "l2": ("definition", spec["l2_definition"])}.values():
        node = out.get(container)
        if isinstance(node, dict):
            for field in fields:
                node.pop(field, None)
    return out


def deferred_l3_roots(milestone_dir):
    """L3 ids whose shipped body hands execution to M08."""
    roots = {}
    for path in milestone_dir.rglob("l3.json"):
        data = read(path)
        if str(data.get("l4_status") or "").startswith(M08_DEFERRAL):
            roots[data["id"]] = data
    return roots


def m08_contributions():
    """parent L3 id -> the L4 ids the M08 package merges under it."""
    if not M08_MANIFEST.is_file():
        return {}
    per_l3 = {}
    for item in read(M08_MANIFEST)["items"]:
        per_l3.setdefault(item["parent_l3_id"], []).append(item["task_id"])
    return {l3_id: sorted(ids) for l3_id, ids in per_l3.items()}


def m08_module_counts():
    """owner module id -> how many L4 the M08 package plans for it."""
    if not M08_MANIFEST.is_file():
        return {}
    per_module = {}
    for item in read(M08_MANIFEST)["items"]:
        per_module[item["owner_module_id"]] = per_module.get(item["owner_module_id"], 0) + 1
    return per_module


def main():
    spec = mg.PACKAGES[sys.argv[1]]
    cur = mg.find_milestone_dir(spec["milestone_dir"])
    inc = ROOT / spec["incoming"] / "OpenMatrix9" / spec["milestone_dir"]
    if not inc.is_dir():
        print(json.dumps({"package": sys.argv[1], "incoming": str(inc),
                          "errors": [f"package tree not found: {inc}"]}, indent=1))
        raise SystemExit(2)
    package_files = [p for p in inc.rglob("*.json") if p.is_file()]
    if not package_files:
        print(json.dumps({"package": sys.argv[1], "incoming": str(inc),
                          "errors": ["package tree has no JSON records"]}, indent=1))
        raise SystemExit(2)
    errors = []
    checked = {}
    deferred = deferred_l3_roots(inc)
    contributions = m08_contributions()
    deferred_seen = {}
    deferred_l2 = []
    deferred_modules = {read(p)["parent_l2_id"] for p in inc.rglob("l3.json")
                        if read(p).get("id") in deferred}
    for path in sorted(package_files):
        rel = path.relative_to(inc).as_posix()
        target = cur / rel
        if not target.is_file():
            errors.append(f"missing locally: {rel}")
            continue
        kind = KIND.get(path.name, "l4")
        checked[kind] = checked.get(kind, 0) + 1
        curdata, incdata = read(target), read(path)
        # A deferred L3 now carries the M08 package's children instead of the
        # shipped "not planned here" body; prove that contribution separately.
        is_deferred = path.name == "l3.json" and incdata.get("id") in deferred
        if path.name == "L2.json":
            for l3dir in sorted(p for p in target.parent.iterdir() if p.is_dir()):
                record = l3dir / "l3.json"
                if record.is_file() and read(record).get("id") in deferred:
                    deferred_l2.append((rel, incdata, curdata))
                    break
        if is_deferred:
            expected = sorted(contributions.get(incdata["id"], []))
            actual = sorted((child.get("id") for child in
                             (curdata.get("children_l4") or [])), key=str)
            deferred_seen[incdata["id"]] = actual
            if actual != expected:
                errors.append(f"{rel}: M08 children differ from the M08 package: "
                              f"{actual} != {expected}")
            extra = [k for k in set(strip_planning(spec, kind, curdata))
                     - set(strip_planning(spec, kind, incdata))]
            if extra:
                errors.append(f"{rel}: non-planning content added: {extra}")
        if kind == "l4":
            stripped = {k: v for k, v in curdata.items()
                        if k not in (mod.REF_KEY, mod.POLICY_FIELD)
                        and k not in ENRICHMENT}
            if stripped != incdata:
                diff = [k for k in set(stripped) | set(incdata)
                        if stripped.get(k) != incdata.get(k)]
                errors.append(f"{rel}: L4 body differs from package: {diff}")
            supplements = curdata.get(mod.REF_KEY, {})
            if supplements.get("status") != "requirements_only_not_runtime_evidence":
                errors.append(f"{rel}: anchor status not requirements-only")
            if curdata.get(mod.POLICY_FIELD, {}).get("status_effect") != "NONE_REQUIREMENTS_ONLY":
                errors.append(f"{rel}: policy status_effect wrong")
        elif not is_deferred:
            if strip_planning(spec, kind, curdata) != strip_planning(spec, kind, incdata):
                errors.append(f"{rel}: non-planning content differs from package")
            # The module and the L3 it defers to are re-planned by the M08 package,
            # so their planning fields are allowed to move on (proved below).
            replanned = (kind == "l2" and incdata.get("id") in deferred_modules)
            for field in spec[kind]:
                if field in incdata and curdata.get(field) != incdata[field]:
                    if replanned and field == "planning_state":
                        continue
                    errors.append(f"{rel}: planning field not adopted: {field}")
    # The L2 that owns deferred L3 must record that its L4 are now planned, and the
    # deferred L3 must have moved out of the shipped "not planned here" status.
    for rel, incdata, curdata in deferred_l2:
        if curdata.get("planning_state") != "SPECIFIED_L4_PLANNED_NOT_IMPLEMENTED":
            errors.append(f"{rel}: owner of deferred L3 reports "
                          f"{curdata.get('planning_state')!r}")
        if curdata.get("definition", {}).get("l3_l4_status") != "L3_DEFINED_L4_PLANNED_NOT_IMPLEMENTED":
            errors.append(f"{rel}: owner of deferred L3 has no L3/L4 planning status")
        if curdata.get("planned_l4_count_m08") != m08_module_counts().get(incdata.get("id")):
            errors.append(f"{rel}: planned_l4_count_m08 "
                          f"{curdata.get('planned_l4_count_m08')!r} does not equal the "
                          f"M08 contribution {m08_module_counts().get(incdata.get('id'))!r}")
    # Every L3 the package defers must now be realised by the M08 package, and every
    # M08 contribution must target a deferred L3 of this milestone.
    for l3_id in deferred:
        if l3_id not in deferred_seen:
            errors.append(f"{l3_id}: deferred L3 was not checked")
        elif l3_id not in contributions:
            errors.append(f"{l3_id}: deferred to M08 but the M08 package has no L4 for it")
    for l3_id in contributions:
        if l3_id in deferred and l3_id not in deferred_seen:
            errors.append(f"{l3_id}: M08 contribution present but the L3 was not checked")
    shipped = {p.relative_to(inc).as_posix() for p in inc.rglob("*") if p.is_file()}
    kept = [p for p in cur.rglob("*")
            if p.is_file() and p.relative_to(cur).as_posix() not in shipped]
    print(json.dumps({"package": sys.argv[1], "checked": checked,
                      "local_only_files_preserved": len(kept),
                      "errors": errors}, indent=1))
    raise SystemExit(bool(errors))


if __name__ == "__main__":
    main()
