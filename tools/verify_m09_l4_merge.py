"""Re-prove the M09 release-gate merge against the package archive.

Fails closed when:

- a merged file's bytes differ from the staged package copy, or a package file is
  missing from the merge;
- the manifest no longer lists 30 tasks (18 G-MVP / 12 G-FULL) with resolvable
  ``path``/``task_path`` inside the reference root;
- the regenerated CSV disagrees with the manifest;
- ``M09/L1.json`` no longer summarises 30 L4 over zero canonical L2/L3 owners, or
  reports a gate decision;
- any record claims execution, a release decision, provenance or effort.
"""
import csv
import hashlib
import json
from pathlib import Path

ROOT = Path("H:/OpenMatrix9")
MILESTONE = ROOT / "OM9_Milestones" / "M09 - Acceptance and Release Gates"
REFERENCE = MILESTONE / "reference" / "m09-l4"
INCOMING = ROOT / "build/m09-incoming/OpenMatrix9"
ARCHIVE_SHA256 = "ff962d452f8e081321b3ff25d9d9b382d71665fe9f911420c44968cd86094249"
# Staged package path -> merged path, relative to the two roots.
LAYOUT = {
    "planning/M09": "reference/m09-l4",
    "planning/M09_RELEASE_GATES.md": "reference/m09-l4/M09_RELEASE_GATES.md",
    "planning/README.md": "reference/m09-l4/PLANNING_README.md",
    "tools/validate_m09_l4.py": "reference/m09-l4/tools/validate_m09_l4.py",
    "tools/m09_next_task.py": "reference/m09-l4/tools/m09_next_task.py",
}


def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    failures = []

    def check(condition, message):
        if not condition:
            failures.append(message)

    # 1. Byte fidelity of every package file, both directions.
    manifest = read(REFERENCE / "M09_EXECUTION_MANIFEST.json")
    expected = {}
    for source_name, target_root in LAYOUT.items():
        source = INCOMING / source_name
        if source.is_dir():
            for path in source.rglob("*"):
                if path.is_file():
                    expected[REFERENCE / path.relative_to(source)] = path
        elif source.is_file():
            expected[MILESTONE / target_root] = source
    # Path-rewritten indexes and the regenerated CSV are checked separately.
    for path, source in sorted(expected.items(), key=lambda kv: str(kv[0])):
        if path.name in ("M09_EXECUTION_MANIFEST.json", "M09_EXECUTION_MANIFEST.csv"):
            continue
        check(path.is_file(), f"missing merged file: {path.relative_to(MILESTONE)}")
        if path.is_file():
            check(sha256(path) == sha256(source),
                  f"bytes differ from the package: {path.relative_to(MILESTONE)}")
    check((REFERENCE / "M09_EXECUTION_MANIFEST.json").is_file(),
          "reference manifest is missing")
    check((REFERENCE / "M09_EXECUTION_MANIFEST.csv").is_file(),
          "reference manifest CSV is missing")

    # 2. Manifest integrity and path resolution.
    items = manifest["items"]
    l3_ids = set()
    cited = set()
    check(len(items) == 30, f"manifest lists {len(items)} tasks, expected 30")
    gates = {}
    for item in items:
        gates[item["parent_gate_id"]] = gates.get(item["parent_gate_id"], 0) + 1
        for key in ("path", "task_path"):
            check(item[key].startswith("reference/m09-l4/"),
                  f"{item['task_id']}: {key} was not re-pointed into the milestone tree: "
                  f"{item[key]}")
            target = MILESTONE / item[key]
            check(target.is_file(), f"{item['task_id']}: {key} does not resolve: {item[key]}")
        record_path = MILESTONE / item["path"]
        if record_path.is_file():
            record = read(record_path)
            check(record["task_id"] == item["task_id"], f"{item['task_id']}: id mismatch in body")
            check(record["status"] == "PLANNED_NOT_EXECUTED",
                  f"{item['task_id']}: status was promoted")
            check(record["gate_status"] == "NOT_TESTED",
                  f"{item['task_id']}: gate_status was promoted")
            check(record["release_decision"] == "UNDECIDED",
                  f"{item['task_id']}: release_decision was promoted")
            check(record["evidence_state"] == "NOT_CREATED",
                  f"{item['task_id']}: evidence_state was promoted")
            check(record["source_revision"] is None and record["toolchain_verified"] is False
                  and record["estimated_effort"] is None,
                  f"{item['task_id']}: provenance or effort was asserted")
            check("parent_l3_id" not in record and "owner_module_id" not in record,
                  f"{item['task_id']}: a canonical owner was invented")
            l3_ids.update(entry["l3_id"] for entry in record["canonical_traceability"])
            cited.update(case["case_id"] for case in record["test_cases"])
    check(gates == {"G-MVP": 18, "G-FULL": 12},
          f"gate histogram is {gates}, expected 18 G-MVP / 12 G-FULL")
    check(len(l3_ids) == 59, f"{len(l3_ids)} canonical L3 refs, expected 59")
    check(len(cited) == 177, f"{len(cited)} cited test cases, expected 177")

    # 3. The CSV must agree with the manifest it was regenerated from.
    with (REFERENCE / "M09_EXECUTION_MANIFEST.csv").open(encoding="utf-8-sig", newline="") as h:
        rows = list(csv.DictReader(h))
    check(len(rows) == 30, f"manifest CSV has {len(rows)} rows, expected 30")
    index = {x["task_id"]: x for x in items}
    for row in rows:
        item = index.get(row["task_id"])
        check(item is not None, f"CSV row {row['task_id']} is not in the manifest")
        if item is None:
            continue
        for field in ("parent_gate_id", "task_kind", "title", "status", "path", "task_path",
                      "evidence_target"):
            check(row[field] == item[field], f"{item['task_id']}: CSV {field} drift")
        check(row["l3_refs"].split(";") == item["l3_refs"],
              f"{item['task_id']}: CSV l3_refs drift")
        check([x for x in row["depends_on_task_ids"].split(";") if x]
              == item["depends_on_task_ids"], f"{item['task_id']}: CSV dependency drift")

    # 4. The L1 release-gate contract.
    l1 = read(MILESTONE / "L1.json")
    plan = l1["release_gate_plan"]
    check(l1["l4_count"] == 30 and l1["module_count"] == 0 and l1["l3_count"] == 0,
          "L1 does not report 30 release L4 over zero L2/L3 owners")
    check(plan["path"] == "reference/m09-l4/M09_EXECUTION_MANIFEST.json",
          "L1 points at the wrong release plan")
    check(plan["test_cases"] == 177 and plan["distinct_canonical_l3_refs"] == 59,
          "L1 test case or canonical reference count drift")
    check(plan["l4_by_gate"]["G-MVP"]["l4"] == 18 and plan["l4_by_gate"]["G-FULL"]["l4"] == 12,
          "L1 gate histogram drift")
    check(plan["canonical_l2_added"] == plan["canonical_l3_added"] == 0,
          "L1 does not state that no canonical owner was added")
    check(l1["planning_source_sha256"] == ARCHIVE_SHA256,
          "L1 records a different archive hash than the delivered package")
    check(all(state == "NOT_TESTED" for state in l1["gate_status"].values())
          and l1["release_decision"] == "UNDECIDED",
          "L1 promoted a gate status or decision")

    # 5. No evidence was fabricated by the merge.
    evidence = ROOT / "implementation" / "evidence" / "M09"
    check(not evidence.exists(),
          "implementation/evidence/M09 exists; the planning merge must not create it")

    if failures:
        print(json.dumps({"failures": failures[:40], "failure_count": len(failures)}, indent=1))
        raise SystemExit(1)
    print("verify_m09_l4_merge: OK (30 release L4 / 177 cases / 59 canonical L3 refs / "
          "byte-identical to the package / no promoted status)")


if __name__ == "__main__":
    main()
