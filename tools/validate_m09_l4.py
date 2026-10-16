#!/usr/bin/env python3
"""Check the merged M09 release-plan structure; intentionally NOT an app/release test.

Derived from the package's own ``planning/M09`` validator, with two changes forced by
this checkout:

1. the canonical L3 inventory is read from the milestone trees
   (``OM9_Milestones/**/l3.json``) instead of ``planning/EXECUTION_MANIFEST.json``
   and ``specs/_meta/l3-registry.json``, neither of which exists here; every L3 the
   30 release tasks trace must exist locally with the recorded module, capability
   and execution stage;
2. the merged package lives at
   ``<M09 milestone>/reference/m09-l4`` and its manifest paths point there.

Deliberately NOT kept: the source plan asserted that
``implementation/evidence/M09`` does not exist. That was correct for a planning ZIP,
but this is a checkout in which release verification will legitimately create real
evidence, so the check would forbid the one directory the plan exists to fill.

Every other assertion of the source validator is preserved, including the DAG,
the 177 identified test cases and that no record claims a release decision.
"""
import csv
import json
import sys
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
B = ROOT / "OM9_Milestones" / "M09 - Acceptance and Release Gates" / "reference" / "m09-l4"
GATES = ("G-MVP", "G-FULL")
MAINLINE_STAGES = ["E" + str(i).zfill(2) for i in range(1, 14)]


def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def build_canonical_index():
    """id -> (module_id, capability_id, execution_stage, occurrences) for local L3.

    A capability may be declared in two milestone trees (M01 and M07 both carry
    OM9-L3-01-12-008 for the STL tessellation codec); that is allowed only when the
    declarations agree on module, capability and stage.
    """
    index, duplicates = {}, []
    for path in sorted((ROOT / "OM9_Milestones").rglob("l3.json")):
        data = read(path)
        spec = data.get("specification") or {}
        record = {
            "module_id": spec.get("module_id") or data.get("parent_l2_id"),
            "capability_id": spec.get("capability_id"),
            "execution_stage": spec.get("execution_stage") or data.get("execution_stage"),
            "paths": [path],
        }
        known = index.get(data["id"])
        if known is None:
            index[data["id"]] = record
            continue
        duplicates.append(data["id"])
        for field in ("module_id", "capability_id", "execution_stage"):
            if known[field] != record[field]:
                raise SystemExit(
                    f"{data['id']}: duplicate declarations disagree on {field}: "
                    f"{known[field]!r} vs {record[field]!r}")
        known["paths"].extend(record["paths"])
    return index, duplicates


def main():
    failures = []

    def check(condition, message):
        if not condition:
            failures.append(message)

    canon, duplicates = build_canonical_index()
    check(len(duplicates) <= 1,
          f"more duplicate L3 declarations than the known M01/M07 codec pair: {duplicates}")
    stages = Counter(record["execution_stage"] for record in canon.values())
    mainline = [record for record in canon.values()
                if record["execution_stage"] in MAINLINE_STAGES]
    check(stages["E13"] == 73, f"E13 L3 count is {stages['E13']}, expected 73")
    check(sum(stages[s] for s in MAINLINE_STAGES[:12]) == 529,
          f"E01-E12 L3 count is {sum(stages[s] for s in MAINLINE_STAGES[:12])}, expected 529")
    check(len(mainline) == 602, f"mainline L3 count is {len(mainline)}, expected 602")

    plan = read(B / "M09_EXECUTION_MANIFEST.json")
    items = plan["items"]
    check(plan["schema_version"] == "om9.m09.release_plan.v1",
          "unexpected manifest schema_version")
    check(plan["canonical_l2_added"] == plan["canonical_l3_added"] == 0,
          "manifest declares added canonical L2/L3")
    check(len(items) == plan["release_l4_count"] == 30,
          f"release L4 count {len(items)} != 30")
    check(Counter(v["parent_gate_id"] for v in items) == {"G-MVP": 18, "G-FULL": 12},
          "gate histogram is not 18 G-MVP / 12 G-FULL")
    check(plan["app_source_in_zip"] is False and plan["source_revision"] is None,
          "manifest claims app source or a source revision")
    check(plan["gate_dependency_graph"]["G-FULL"] == ["G-MVP", "E13"],
          "G-FULL dependency graph changed")
    check(plan["gate_dependency_graph"]["G-MVP"] == ["G00"] + MAINLINE_STAGES[:12],
          "G-MVP dependency graph changed")
    ids = {x["task_id"] for x in items}
    check(len(ids) == 30, "task ids are not unique")
    check(len({x["path"] for x in items}) == 30 and len({x["task_path"] for x in items}) == 30,
          "manifest paths are not unique")
    for item in items:
        check((B / Path(item["path"]).relative_to("reference/m09-l4")).is_file()
              if item["path"].startswith("reference/m09-l4") else True,
              f"{item['task_id']}: manifest path does not resolve: {item['path']}")

    cases, graph, total_refs, case_ids = [], {}, set(), set()
    for item in items:
        l4_path = B / Path(item["path"]).relative_to("reference/m09-l4")
        task_path = B / Path(item["task_path"]).relative_to("reference/m09-l4")
        check(l4_path.is_file(), f"{item['task_id']}: {l4_path} missing")
        check(task_path.is_file(), f"{item['task_id']}: {task_path} missing")
        if not (l4_path.is_file() and task_path.is_file()):
            continue
        record = read(l4_path)
        text = task_path.read_text("utf8")
        for key, wanted in [("task_id", item["task_id"]), ("title", item["title"]),
                            ("parent_gate_id", item["parent_gate_id"]),
                            ("task_kind", item["task_kind"]),
                            ("status", "PLANNED_NOT_EXECUTED"),
                            ("owner_milestone", "M09"),
                            ("owner_type", "RELEASE_GATE_ONLY"),
                            ("evidence_target", item["evidence_target"]),
                            ("depends_on_task_ids", item["depends_on_task_ids"])]:
            check(record.get(key) == wanted,
                  f"{item['task_id']}: {key} is {record.get(key)!r}, expected {wanted!r}")
        check(record["gate_status"] == "NOT_TESTED" and record["release_decision"] == "UNDECIDED",
              f"{item['task_id']}: gate status or release decision was promoted")
        check(record["source_revision"] is None
              and record["source_paths_verified_locally"] == []
              and record["toolchain_verified"] is False,
              f"{item['task_id']}: source provenance was asserted without a checkout")
        check(record["estimated_effort"] is None and record["evidence_state"] == "NOT_CREATED",
              f"{item['task_id']}: effort estimate or evidence state was promoted")
        check("parent_l3_id" not in record and "owner_module_id" not in record
              and "execution_stage" not in record,
              f"{item['task_id']}: release record invented an L2/L3 owner")
        expected_pre = (["G00_PASS", "E01..E12_PASS"] if record["parent_gate_id"] == "G-MVP"
                        else ["G_MVP_SIGNED_PASS", "E13_PASS"])
        check(record["external_gate_preconditions"] == expected_pre,
              f"{item['task_id']}: external preconditions changed")
        check(item["status"] == "PLANNED_NOT_EXECUTED" and item["source_revision"] is None,
              f"{item['task_id']}: manifest status or revision was promoted")
        check([r["l3_id"] for r in record["canonical_traceability"]] == item["l3_refs"],
              f"{item['task_id']}: canonical_traceability disagrees with the manifest refs")
        for entry in record["canonical_traceability"]:
            local = canon.get(entry["l3_id"])
            check(local is not None,
                  f"{item['task_id']}: traced L3 {entry['l3_id']} is absent from the trees")
            if local is None:
                continue
            total_refs.add(entry["l3_id"])
            check((entry["module_id"], entry["capability_id"], entry["execution_stage"])
                  == (local["module_id"], local["capability_id"], local["execution_stage"]),
                  f"{item['task_id']}: {entry['l3_id']} module/capability/stage drift")
            recorded = entry.get("l3_path", "")
            # The archive records its own spec-package path, which uses slug folder
            # names; this checkout names the same folders by title. Only the record
            # identity is asserted here, never the folder spelling.
            check(recorded.endswith("/l3.json") and recorded.startswith("specs/"),
                  f"{item['task_id']}: {entry['l3_id']} l3_path is not a spec l3.json "
                  f"reference: {recorded!r}")
            check(local["module_id"] == entry["module_id"],
                  f"{item['task_id']}: {entry['l3_id']} module drift")
            check(local["execution_stage"] in MAINLINE_STAGES,
                  f"{item['task_id']}: {entry['l3_id']} is {local['execution_stage']}; "
                  "AFTER cannot gate release")
            check(all(p.is_file() for p in local["paths"]),
                  f"{item['task_id']}: {entry['l3_id']} tree location does not resolve")
        for nav in record["origin_manual_navigation"]:
            check(nav["filename"] in {"matrix8_book_1.pdf", "matrix_8_manual_book2.pdf",
                                      "rhino5_manual.pdf", "matrix9.0_addendum_manual.pdf"},
                  f"{item['task_id']}: unknown manual {nav['filename']}")
        check(len(record["execution_steps"]) >= 3,
              f"{item['task_id']}: fewer than 3 execution steps")
        check(len(record["negative_cases_and_blockers"]) >= 2,
              f"{item['task_id']}: fewer than 2 negative cases")
        check(len(record["test_cases"]) == len(record["execution_steps"])
              + len(record["negative_cases_and_blockers"]),
              f"{item['task_id']}: test case count does not match steps + negative cases")
        check(record["specific_pass_oracle"]
              and all(c["status"] == "NOT_EXECUTED" for c in record["test_cases"]),
              f"{item['task_id']}: test case was executed or the oracle is missing")
        for case in record["test_cases"]:
            check(case["case_id"] not in case_ids,
                  f"duplicate case id {case['case_id']}")
            case_ids.add(case["case_id"])
        check(record["evidence_target"].startswith(
            "implementation/evidence/M09/" + record["parent_gate_id"] + "/"),
            f"{item['task_id']}: evidence target is outside its gate")
        check(set(record["depends_on_task_ids"]) <= ids
              and item["task_id"] not in record["depends_on_task_ids"],
              f"{item['task_id']}: invalid internal dependency")
        check(all(item["parent_gate_id"] == next(u["parent_gate_id"] for u in items
                                                 if u["task_id"] == d)
                  for d in record["depends_on_task_ids"]),
              f"{item['task_id']}: cross-gate internal dependency")
        for marker in [item["task_id"], "## Thứ tự làm", "## Acceptance", "## Bằng chứng",
                       "## Ca kiểm thử có mã định danh", "**Oracle cụ thể:**"]:
            check(marker in text, f"{item['task_id']}: TASK.md is missing {marker!r}")
        check(not any(x in text for x in ["**Result:** PASS", "**Trạng thái:** `DONE`"]),
              f"{item['task_id']}: TASK.md claims a result")
        graph[item["task_id"]] = record["depends_on_task_ids"]
        cases.extend((item, c) for c in record["test_cases"])

    check(len(total_refs) >= 50, f"only {len(total_refs)} distinct canonical L3 refs")

    visited, busy, top = set(), set(), []

    def walk(node):
        if node in visited:
            return
        check(node not in busy, f"dependency cycle at {node}")
        busy.add(node)
        for dep in graph[node]:
            walk(dep)
        busy.discard(node)
        visited.add(node)
        top.append(node)

    for node in graph:
        walk(node)
    check(len(top) == 30, f"topological order covers {len(top)} of 30 tasks")
    check(graph.get("OM9-M09-L4-GMVP-018")
          == ["OM9-M09-L4-GMVP-001", "OM9-M09-L4-GMVP-017"],
          "G-MVP signoff dependencies changed")
    check("OM9-M09-L4-GFULL-010" in graph.get("OM9-M09-L4-GFULL-012", []),
          "G-FULL signoff no longer depends on the MVP regression task")

    with (B / "M09_EXECUTION_MANIFEST.csv").open(encoding="utf-8-sig", newline="") as handle:
        rows = list(csv.DictReader(handle))
    check(len(rows) == len(items), f"manifest CSV has {len(rows)} rows, expected {len(items)}")
    index = {x["task_id"]: x for x in items}
    for row in rows:
        item = index.get(row["task_id"])
        check(item is not None, f"CSV row {row['task_id']} is not in the manifest")
        if item is None:
            continue
        for field in ["task_id", "parent_gate_id", "task_kind", "title", "status", "path",
                      "task_path", "evidence_target"]:
            check(row[field] == item[field], f"{item['task_id']}: CSV {field} drift")
        check(row["l3_refs"].split(";") == item["l3_refs"],
              f"{item['task_id']}: CSV l3_refs drift")
        check([x for x in row["depends_on_task_ids"].split(";") if x]
              == item["depends_on_task_ids"], f"{item['task_id']}: CSV dependency drift")

    with (B / "M09_TEST_CASES.csv").open(encoding="utf-8-sig", newline="") as handle:
        case_rows = list(csv.DictReader(handle))
    check(len(case_rows) == len(cases) == 177,
          f"test case CSV has {len(case_rows)} rows against {len(cases)} cases")
    expected = {c["case_id"]: (t, c) for t, c in cases}
    check(len(expected) == len(case_rows), "test case ids are not unique across the CSVs")
    for row in case_rows:
        pair = expected.get(row["case_id"])
        check(pair is not None, f"unknown case id in the CSV: {row['case_id']}")
        if pair is None:
            continue
        task, case = pair
        check(row["task_id"] == task["task_id"] and row["gate_id"] == task["parent_gate_id"],
              f"{row['case_id']}: CSV task/gate drift")
        check(row["case_kind"] == case["case_kind"] and row["action"] == case["action"],
              f"{row['case_id']}: CSV kind/action drift")
        check(row["status"] == "NOT_EXECUTED"
              and row["evidence_target"] == task["evidence_target"],
              f"{row['case_id']}: CSV status or evidence target drift")
        check(row["expected_oracle"] == case["expected"],
              f"{row['case_id']}: CSV oracle drift")

    for name in ["M09_EVIDENCE_TEMPLATE.json", "M09_GATE_SIGNOFF_TEMPLATE.json"]:
        check(read(B / name)["template_only"] is True, f"{name} is not marked template_only")
    check(read(B / "M09_GATE_SIGNOFF_TEMPLATE.json")["decision"] == "UNDECIDED",
          "signoff template already carries a decision")
    for name in ["G_MVP_CHECKLIST.md", "G_FULL_CHECKLIST.md"]:
        check("NOT_TESTED" in (B / name).read_text("utf8"),
              f"{name} does not state NOT_TESTED")
    check("M09/README.md" in (B / "M09_RELEASE_GATES.md").read_text("utf8"),
          "release gate document no longer points at the M09 plan")

    l1 = read(ROOT / "OM9_Milestones" / "M09 - Acceptance and Release Gates" / "L1.json")
    check(l1["l4_count"] == 30 and l1["module_count"] == 0 and l1["l3_count"] == 0,
          "M09 L1 no longer reports 30 release L4 over zero L2/L3 owners")
    check(l1["release_gate_plan"]["l4_by_gate"]["G-MVP"]["l4"] == 18
          and l1["release_gate_plan"]["l4_by_gate"]["G-FULL"]["l4"] == 12,
          "M09 L1 gate histogram drift")
    check(all(state == "NOT_TESTED" for state in l1["gate_status"].values())
          and l1["release_decision"] == "UNDECIDED",
          "M09 L1 promoted a gate status")

    if failures:
        print(json.dumps({"failures": failures[:40], "failure_count": len(failures)}, indent=1))
        for entry in failures[:10]:
            print("FAIL:", entry)
        raise SystemExit(1)
    print(f"PASS M09 planning validation: 30 L4 / 177 cases / {len(total_refs)} unique canonical "
          f"L3 refs / 2 release gates / acyclic DAG")
    print("NO CLAIMS: G00/E12/E13 signed, actual Rust/OCCT build, geometry test PASS, human "
          "signoff, release ready")


if __name__ == "__main__":
    main()
