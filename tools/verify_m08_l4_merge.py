"""Re-prove the M08 L4 overlay merge against its manifest.

Checks, failing closed on the first inconsistency:

- the staged package manifest declares 221 tasks over 73 parent L3;
- every manifest task exists in a milestone tree as ``l4.json`` + ``TASK.md``,
  with matching id/task_id/level/role/overlay id/dependency list;
- every L3 named by the manifest carries exactly its manifest children in
  ``children_l4`` and reports ``l4_status`` PLANNED_NOT_EXECUTED;
- the L4 spec-tree location follows the manifest ``milestone_id`` (M03-owned
  transform adapters stay in the M03 tree);
- the package documents were copied into ``<milestone>/reference/m08-l4``;
- no L4 record claims execution: every status stays PLANNED_NOT_EXECUTED.
"""
import argparse
import json
from pathlib import Path

ROOT = Path("H:/OpenMatrix9")
MILESTONES = ROOT / "OM9_Milestones"
STAGING = ROOT / "build/m08-incoming/OpenMatrix9"
PACKAGE_MANIFEST = STAGING / "planning/M08/M08_L4_EXECUTION_MANIFEST.json"
M08_DIR = "M08 - Jewelry Modeling"
M03_DIR = "M03 - Geometry 3DM"
REFERENCE_FILES = [
    "M08_L4_EXECUTION_MANIFEST.json", "M08_L4_EXECUTION_MANIFEST.csv",
    "PACKAGING_REPORT.json", "SELF_REVIEW.md", "RELEASE_GATE.md",
    "LUNA_START_HERE.md", "GPT6_LUNA_TASK_PROMPT.md", "README.md",
]
LABELS = {
    "SOURCE_AND_CONTRACT": ("CONTRACT_FIXTURE", "Source And Interface Contract"),
    "DOMAIN_IMPLEMENTATION": ("IMPLEMENT", "Owner Operation Or Dispatch"),
    "MENU_BINDING_ACCEPTANCE": ("ACCEPTANCE", "Menu Binding Acceptance"),
    "FEBE_ACCEPTANCE": ("ACCEPTANCE", "Transaction And UI Acceptance"),
    "ADVERSARIAL_GATE": ("ADVERSARIAL", "Adversarial History Regression"),
}
ORDER = ["SOURCE_AND_CONTRACT", "DOMAIN_IMPLEMENTATION",
         "MENU_BINDING_ACCEPTANCE", "FEBE_ACCEPTANCE", "ADVERSARIAL_GATE"]


def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--quiet", action="store_true")
    args = parser.parse_args()

    failures = []

    def check(condition, message):
        if not condition:
            failures.append(message)

    manifest = read(PACKAGE_MANIFEST)
    items = manifest["items"]
    check(len(items) == 221, f"manifest items {len(items)} != 221")
    check(manifest.get("l4_count") == 221, "manifest l4_count is not 221")
    check(manifest.get("canonical_l3_count") == 73,
          "manifest canonical_l3_count is not 73")
    check(manifest.get("implementation_count") == 0,
          "manifest no longer declares zero implementation")

    # Index every L3 in the two trees the package touches.
    l3_dirs = {}
    for milestone in (M08_DIR, M03_DIR):
        base = MILESTONES / milestone
        for l2dir in sorted(p for p in base.iterdir()
                            if p.is_dir() and not p.name.startswith("reference")):
            for l3dir in sorted(p for p in l2dir.iterdir() if p.is_dir()):
                f = l3dir / "l3.json"
                if f.is_file():
                    l3_id = read(f)["id"]
                    check(l3_id not in l3_dirs, f"duplicate L3 id {l3_id}")
                    l3_dirs[l3_id] = l3dir

    per_l3 = {}
    for item in items:
        per_l3.setdefault(item["parent_l3_id"], []).append(item)

    checked_records = 0
    placed = {"M03": 0, "M08": 0}
    for l3_id, entries in sorted(per_l3.items()):
        l3dir = l3_dirs.get(l3_id)
        check(l3dir is not None, f"{l3_id}: parent L3 missing from both trees")
        if l3dir is None:
            continue
        l3 = read(l3dir / "l3.json")
        check(l3.get("l4_status") == "PLANNED_NOT_EXECUTED",
              f"{l3_id}: l4_status is {l3.get('l4_status')!r}")
        children = l3.get("children_l4") or []
        expected_home = MILESTONES / (
            M03_DIR if entries[0]["milestone_id"] == "M03" else M08_DIR)
        check(l3dir.is_relative_to(expected_home),
              f"{l3_id}: placed in {l3dir}, manifest says {entries[0]['milestone_id']}")
        entries = sorted(entries, key=lambda r: (ORDER.index(r["work_kind"]), r["task_id"]))
        check(len(children) == len(entries),
              f"{l3_id}: children_l4 {len(children)} != manifest {len(entries)}")
        for index, item in enumerate(entries, start=1):
            role, label = LABELS[item["work_kind"]]
            l4dir = l3dir / f"L4-{index:02d} - {label}"
            record_path = l4dir / "l4.json"
            check(record_path.is_file(), f"{item['task_id']}: {record_path} missing")
            check((l4dir / "TASK.md").is_file(),
                  f"{item['task_id']}: TASK.md missing")
            if not record_path.is_file():
                continue
            record = read(record_path)
            checked_records += 1
            check(record.get("id") == item["task_id"],
                  f"{item['task_id']}: id is {record.get('id')!r}")
            check(record.get("task_id") == item["task_id"],
                  f"{item['task_id']}: task_id mismatch")
            check(record.get("parent_l3_id") == l3_id,
                  f"{item['task_id']}: parent_l3_id mismatch")
            check(record.get("level") == "L4", f"{item['task_id']}: level is not L4")
            check(record.get("task_role") == role,
                  f"{item['task_id']}: task_role {record.get('task_role')!r} != {role}")
            check(record.get("status") == "PLANNED_NOT_EXECUTED",
                  f"{item['task_id']}: status {record.get('status')!r} is not planned")
            check(record.get("milestone_id") == item["milestone_id"],
                  f"{item['task_id']}: milestone_id drift")
            check(record.get("depends_on_l4_ids") == list(item["depends_on_task_ids"]),
                  f"{item['task_id']}: depends_on_l4_ids drift")
            check(record.get("reference_policy", "").startswith("Planning-only record"),
                  f"{item['task_id']}: reference_policy missing")
            placed[item["milestone_id"]] += 1
            if index <= len(children):
                child = children[index - 1]
                check(child.get("id") == item["task_id"],
                      f"{l3_id}: children_l4[{index}] is {child.get('id')!r}")
                check(child.get("role") == role,
                      f"{l3_id}: children_l4[{index}] role drift")
                check(Path(MILESTONES / child["file"]).is_file(),
                      f"{l3_id}: children_l4[{index}] file does not resolve")

    # Every dependency target must exist as a merged L4 record.
    ids = set()
    for item in items:
        ids.add(item["task_id"])
    for item in items:
        for dep in item["depends_on_task_ids"]:
            check(dep in ids, f"{item['task_id']}: dependency {dep} is not in the manifest")

    for name in REFERENCE_FILES:
        check((MILESTONES / M08_DIR / "reference/m08-l4" / name).is_file(),
              f"reference/m08-l4/{name} missing")
        check((STAGING / "planning/M08" / name).is_file(),
              f"staged planning/M08/{name} missing")

    l1 = read(MILESTONES / M08_DIR / "L1.json")
    check(l1.get("l4_count") == 221, f"M08 L1 l4_count is {l1.get('l4_count')!r}")
    check(l1.get("l4_count_in_tree") == placed["M08"],
          "M08 L1 l4_count_in_tree drift")
    check(l1.get("l4_placed_by_milestone") == {"M03": placed["M03"], "M08": placed["M08"]},
          "M08 L1 l4_placed_by_milestone drift")
    check(l1.get("l3_spec_tree_by_milestone") == {"M03": 22, "M08": 51},
          "M08 L1 l3_spec_tree_by_milestone drift")

    total_l4 = len(list(MILESTONES.rglob("l4.json")))
    summary = {
        "manifest_items": len(items),
        "l3_covered": len(per_l3),
        "records_checked": checked_records,
        "placed": placed,
        # Every l4.json in the milestone trees, including the M09 release-board plan
        # records that live under M09/reference and owners no L2/L3.
        "workspace_l4_total": total_l4,
        "failures": failures,
    }
    if not args.quiet:
        print(json.dumps(summary, indent=1))
    if failures:
        for entry in failures[:20]:
            print("FAIL:", entry)
        raise SystemExit(1)
    print(f"verify_m08_l4_merge: OK ({checked_records} L4 over {len(per_l3)} L3)")


if __name__ == "__main__":
    main()
