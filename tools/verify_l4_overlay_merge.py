"""Verify an overlay-style L4 merge: merged tree == overlay + anchors.

Usage: python tools/verify_l4_overlay_merge.py <package>   (m05 | m06)

Per overlay record: the milestone L4 exists at the expected path, its body equals
the overlay record apart from the two anchor members and the fields the merger
deliberately rewrites, the id/dependency rewrite is consistent, TASK.md is
present, and every parent L3 lists the child.
"""
import json
import sys
from pathlib import Path

ROOT = Path("H:/OpenMatrix9")
sys.path.insert(0, str(ROOT / "tools"))
import apply_matrix_ref_supplements as mod  # noqa: E402
import merge_l4_overlay as mg  # noqa: E402

REWRITTEN = {"id", "level", "task_id", "task_role", "overlay_task_id", "owner_l2_id",
             "depends_on_l4_ids", "evidence_path", "manual_signoff_required",
             "reference_policy"}
# overlay_phase / overlay_work_kind are merger-authored provenance markers.
REWRITTEN_PREFIX = ("overlay_",)
# Authored enrichment applied on top of the shipped overlay package. These are not
# part of the incoming body, so they are excluded when proving the merge preserved it.
ENRICHMENT = {"implementation_samples", "implementation_sample_guide"}


def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def main():
    spec = mg.PACKAGES[sys.argv[1]]
    current = mg.find_milestone_dir(spec["milestone_dir"])
    incoming = ROOT / spec["incoming"]
    planning = incoming / spec["planning"]
    manifest = read(planning / spec["manifest"])
    errors = []
    children_seen = 0
    covered_l3 = set()

    for l3file in sorted(current.glob("*/*/l3.json")):
        l3 = read(l3file)
        covered_l3.add(l3["id"])
        children_seen += len(l3.get("children_l4") or [])
        for child in l3.get("children_l4") or []:
            target = ROOT / "OM9_Milestones" / child["file"]
            if not target.is_file():
                errors.append(f"{child['id']}: child file missing: {child['file']}")
                continue
            if not (target.parent / "TASK.md").is_file():
                errors.append(f"{child['id']}: TASK.md missing beside {target}")
            record = read(target)
            if record.get("id") != child["id"]:
                errors.append(f"{child['id']}: id mismatch in {target}")
            if record.get("parent_l3_id") != l3["id"]:
                errors.append(f"{child['id']}: parent_l3_id mismatch")
            if record.get("task_role") != child.get("role"):
                errors.append(f"{child['id']}: role mismatch vs children_l4")

    by_id = {}
    for candidate in current.rglob("l4.json"):
        by_id.setdefault(read(candidate).get("id"), candidate)

    checked = 0
    phases = set()
    for item in manifest["items"]:
        checked += 1
        phases.add(item[spec["label_key"]])
        overlay = read(incoming / item[spec["l4_path_key"]])
        new_id = mg.normalise_id(item["task_id"])
        target = by_id.get(new_id)
        if target is None:
            errors.append(f"{new_id}: milestone record not found")
            continue
        record = read(target)
        stripped = {k: v for k, v in record.items()
                    if k not in REWRITTEN and k not in (mod.REF_KEY, mod.POLICY_FIELD)
                    and k not in ENRICHMENT
                    and not k.startswith(REWRITTEN_PREFIX)}
        expected = {k: v for k, v in overlay.items() if k not in REWRITTEN}
        if stripped != expected:
            diff = sorted(k for k in set(stripped) | set(expected)
                          if stripped.get(k) != expected.get(k))
            errors.append(f"{new_id}: body differs from overlay: {diff}")
        if record.get("overlay_task_id") != item["task_id"]:
            errors.append(f"{new_id}: overlay_task_id not preserved")
        expected_deps = [mg.normalise_id(d) for d in mg.dep_list(item.get("depends_on_task_ids"))]
        if record.get("depends_on_l4_ids") != [d for d in expected_deps if d]:
            errors.append(f"{new_id}: dependency rewrite mismatch")
        if record[mod.REF_KEY]["status"] != "requirements_only_not_runtime_evidence":
            errors.append(f"{new_id}: anchor status not requirements-only")
        if record.get(mod.POLICY_FIELD, {}).get("status_effect") != "NONE_REQUIREMENTS_ONLY":
            errors.append(f"{new_id}: policy status_effect wrong")

    statuses = {read(p).get("status") for p in current.rglob("l4.json")}
    print(json.dumps({
        "package": sys.argv[1],
        "overlay_records": checked,
        "children_l4_entries": children_seen,
        "l3_covered_locally": len(covered_l3),
        "distinct_work_labels": sorted(phases),
        "distinct_l4_status": sorted(s for s in statuses if s),
        "errors": errors,
    }, ensure_ascii=False, indent=1))
    raise SystemExit(bool(errors))


if __name__ == "__main__":
    main()
