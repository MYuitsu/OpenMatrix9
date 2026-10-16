"""Merge the M09 release-gate L4 package into the M09 milestone tree.

M09 is not an implementation milestone: the package ships 30 release-verification
L4 (18 G-MVP after E12, 12 G-FULL after G-MVP + E13) that only read and verify
canonical L3 owned by M01-M08. There is therefore no L2/L3 to hang an L4 directory
under, so the package is merged as the milestone's own release board:

- every package file keeps its relative layout under
  ``<M09 milestone>/reference/m09-l4/`` (``g-mvp/NN``, ``g-full/NN``, documents,
  and the package's own ``tools/`` helper + validator);
- the manifest and CSV ``path``/``task_path`` members are re-pointed to that
  reference root so they resolve inside the checkout;
- ``M09/L1.json`` gains the release-gate plan summary while keeping its
  "no L2/L3 owner" contract;
- every copied file's SHA256 is recorded, and a file that already exists with
  identical bytes is reported as already merged rather than rewritten.

No status is promoted: all 30 L4 keep ``PLANNED_NOT_EXECUTED``,
``gate_status=NOT_TESTED``, ``release_decision=UNDECIDED`` and
``evidence_state=NOT_CREATED``.
"""
import argparse
import csv
import hashlib
import json
import shutil
from pathlib import Path

ROOT = Path("H:/OpenMatrix9")
MILESTONES = ROOT / "OM9_Milestones"
MILESTONE_DIR = "M09 - Acceptance and Release Gates"
INCOMING = ROOT / "build/m09-incoming/OpenMatrix9"
PACKAGE = INCOMING / "planning/M09"
ARCHIVE_ROOT = INCOMING / "planning"
REFERENCE_TAG = "reference/m09-l4"
MANIFEST_NAME = "M09_EXECUTION_MANIFEST.json"
REPORT_OUT = ROOT / "build/m09-merge-report.json"

# Archive layout -> milestone-tree layout. The gate folders keep the package's own
# names so its manifest paths stay recognisable; only the root moves.
LAYOUT = {
    "planning/M09": REFERENCE_TAG,
    "planning/M09_RELEASE_GATES.md": f"{REFERENCE_TAG}/M09_RELEASE_GATES.md",
    "planning/README.md": f"{REFERENCE_TAG}/PLANNING_README.md",
    "tools/validate_m09_l4.py": f"{REFERENCE_TAG}/tools/validate_m09_l4.py",
    "tools/m09_next_task.py": f"{REFERENCE_TAG}/tools/m09_next_task.py",
}
# Files copied straight through, relative to the reference root.
PACKAGE_FILES = [
    "README.md", "GPT6_LUNA_EXECUTION_PROMPT.md", "M09_EXECUTION_MANIFEST.json",
    "M09_EXECUTION_MANIFEST.csv", "M09_EXECUTION_TREE.txt", "M09_TEST_CASES.csv",
    "M09_REVIEW_REPORT.md", "M09_EVIDENCE_TEMPLATE.json",
    "M09_GATE_SIGNOFF_TEMPLATE.json", "M09_RUNTIME_STATE_TEMPLATE.json",
    "G_MVP_CHECKLIST.md", "G_FULL_CHECKLIST.md",
]


def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def write(path, data):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def milestone_dir():
    for path in MILESTONES.iterdir():
        if path.is_dir() and path.name.startswith("M09 "):
            return path
    raise SystemExit("M09 milestone directory not found")


def rewrite_path(value):
    """Map an archive-relative package path onto the merged reference layout."""
    for prefix, target in LAYOUT.items():
        if value == prefix or value.startswith(prefix + "/"):
            return target + value[len(prefix):]
    raise SystemExit(f"package path outside the known layout: {value}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--apply", action="store_true")
    args = parser.parse_args()

    current = milestone_dir()
    reference = current / REFERENCE_TAG
    manifest = read(PACKAGE / MANIFEST_NAME)
    items = manifest["items"]
    report = {
        "schema": "om9.m09.l4_package_merge.v1",
        "source_archive": "C:/Users/nguye/Downloads/OpenMatrix9_M09_L4_Patch.zip",
        "milestone_dir": MILESTONE_DIR,
        "reference_root": f"{MILESTONE_DIR}/{REFERENCE_TAG}",
        "manifest_declared": {k: v for k, v in manifest.items() if k != "items"},
        "manifest_items": len(items),
        "files": [], "copied": 0, "unchanged": 0, "path_rewrites": [], "conflicts": [],
    }

    # ---- 1. re-point the manifest and CSV indexes -------------------------------
    for item in items:
        for key in ("path", "task_path"):
            new = rewrite_path(item[key])
            if new != item[key]:
                report["path_rewrites"].append({"task_id": item["task_id"], key: new})
            item[key] = new
    # The CSV is regenerated from this manifest below, so the two cannot drift.
    if args.apply:
        write(reference / MANIFEST_NAME, manifest)
    report["files"].append(f"{REFERENCE_TAG}/{MANIFEST_NAME}")

    # ---- 2. copy the package documents and the gate folder layout ---------------
    targets = {}
    for name in PACKAGE_FILES:
        if name == MANIFEST_NAME:
            continue
        targets[PACKAGE / name] = reference / name
    for gate in ("g-mvp", "g-full"):
        for path in sorted((PACKAGE / gate).rglob("*")):
            if path.is_file():
                targets[path] = reference / gate / path.relative_to(PACKAGE / gate)
    for source_name, target_name in LAYOUT.items():
        source = INCOMING / source_name
        if source.is_file():
            targets[source] = current / target_name

    for source, target in sorted(targets.items(), key=lambda kv: str(kv[1])):
        if not source.is_file():
            report["conflicts"].append(f"package file missing: {source.relative_to(INCOMING)}")
            continue
        digest = sha256(source)
        note = "copied"
        if target.is_file():
            if sha256(target) == digest:
                note = "unchanged"
            else:
                report["conflicts"].append(f"local file differs: {target.relative_to(current)}")
                continue
        if args.apply and note == "copied":
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source, target)
        report[note if note == "unchanged" else "copied"] += 1
        report["files"].append({
            "path": target.relative_to(current).as_posix(),
            "sha256": digest,
            "action": note,
        })

    # ---- 3. the L1 release-gate contract ---------------------------------------
    l1 = read(current / "L1.json")
    per_gate = {}
    for item in items:
        bucket = per_gate.setdefault(item["parent_gate_id"], {"l4": 0, "kinds": {}})
        bucket["l4"] += 1
        bucket["kinds"][item["task_kind"]] = bucket["kinds"].get(item["task_kind"], 0) + 1
    case_count = 0
    trace_refs = set()
    for item in items:
        local = reference / Path(item["path"]).relative_to(REFERENCE_TAG)
        source = PACKAGE / Path(item["path"]).relative_to(REFERENCE_TAG)
        record = read(local if local.is_file() else source)
        case_count += len(record["test_cases"])
        trace_refs.update(entry["l3_id"] for entry in record["canonical_traceability"])
    l1.update({
        "planning_state": "L1_RELEASE_GATE_L4_PLANNED_NOT_EXECUTED",
        "planning_only": True,
        "module_count": 0,
        "l3_count": 0,
        "l4_count": len(items),
        "planning_source_archive": report["source_archive"],
        "planning_source_sha256": ARCHIVE_SHA256,
        "release_gate_plan": {
            "path": f"{REFERENCE_TAG}/{MANIFEST_NAME}",
            "schema_version": manifest["schema_version"],
            "owner_type": manifest["owner_type"],
            "release_gate_ids": manifest["release_gate_ids"],
            "gate_dependency_graph": manifest["gate_dependency_graph"],
            "release_l4_count": manifest["release_l4_count"],
            "canonical_l2_added": manifest["canonical_l2_added"],
            "canonical_l3_added": manifest["canonical_l3_added"],
            "app_source_in_zip": manifest["app_source_in_zip"],
            "source_revision": manifest["source_revision"],
            "status": manifest["status"],
            "l4_by_gate": per_gate,
            "test_cases": case_count,
            "distinct_canonical_l3_refs": len(trace_refs),
            "documents": sorted({f["path"] for f in report["files"]
                                 if isinstance(f, dict) and f["path"].endswith(".md")}),
        },
        "gate_status": {gate: "NOT_TESTED" for gate in per_gate},
        "release_decision": "UNDECIDED",
        "evidence_state": "NOT_CREATED",
        "note": ("Nguồn gốc chỉ quy định hai release gate G-MVP và G-FULL; không có L2/L3 với ID "
                 "gốc. Không tự phát sinh L2/L3 mới. 30 L4 kiểm định nằm trong "
                 f"{REFERENCE_TAG}/ và chỉ đọc L3 canonical của M01-M08."),
    })
    if args.apply:
        write(current / "L1.json", l1)
    report["l1"] = {"l4_count": l1["l4_count"], "gates": per_gate,
                    "test_cases": case_count, "trace_refs": len(trace_refs)}

    # ---- 4. the CSV index, regenerated so it can never drift from the manifest --
    csv_path = reference / "M09_EXECUTION_MANIFEST.csv"
    if args.apply:
        with csv_path.open("w", encoding="utf-8-sig", newline="") as handle:
            writer = csv.writer(handle)
            fields = ["task_id", "parent_gate_id", "task_kind", "title", "status", "l3_refs",
                      "depends_on_task_ids", "path", "task_path", "evidence_target"]
            writer.writerow(fields)
            for item in items:
                writer.writerow([
                    item["task_id"], item["parent_gate_id"], item["task_kind"], item["title"],
                    item["status"], ";".join(item["l3_refs"]),
                    ";".join(item["depends_on_task_ids"]), item["path"], item["task_path"],
                    item["evidence_target"],
                ])
        report["csv_regenerated"] = f"{REFERENCE_TAG}/M09_EXECUTION_MANIFEST.csv"

    REPORT_OUT.parent.mkdir(parents=True, exist_ok=True)
    REPORT_OUT.write_text(json.dumps(report, ensure_ascii=False, indent=1) + "\n",
                          encoding="utf-8")
    print(json.dumps({
        "apply": args.apply,
        "manifest_items": len(items),
        "copied": report["copied"],
        "unchanged": report["unchanged"],
        "path_rewrites": len(report["path_rewrites"]),
        "conflicts": len(report["conflicts"]),
        "l1_l4_count": l1["l4_count"],
        "test_cases": case_count,
        "distinct_refs": len(trace_refs),
    }, indent=1))
    for entry in report["conflicts"][:10]:
        print("CONFLICT:", entry)


ARCHIVE_SHA256 = "ff962d452f8e081321b3ff25d9d9b382d71665fe9f911420c44968cd86094249"


if __name__ == "__main__":
    main()
