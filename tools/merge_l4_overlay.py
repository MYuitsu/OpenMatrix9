"""Merge an overlay-style L4 planning package into its milestone tree.

Two package families use this shape: a flat ``planning/<Mxx>`` layout plus an
execution manifest listing every task, instead of mirroring the milestone tree.
The manifest is the authoritative index, so this merger:

- places each overlay L4 at
  ``<Mxx milestone>/<L2 dir>/<L3 dir>/<L4-nn - Label>/l4.json``
  together with its ``TASK.md``;
- rewrites the overlay id (``OM9-M05-L4-09-001-01`` / ``OM9-M06-L4-14-001-01``)
  to the milestone form (``OM9-L4-09-001-01`` / ``OM9-L4-14-001-01``), keeping the
  overlay id on the record and rewriting intra-milestone dependencies;
- fills ``children_l4`` / ``l4_status`` / planning state on the parents and the
  L1 count;
- copies the package manifest, coverage/review documents and runbook into
  ``<milestone>/reference/<tag>/`` for audit.

No status is promoted: every L4 keeps the package's planned/blocked status.
"""
import argparse
import json
import re
import shutil
from pathlib import Path

ROOT = Path("H:/OpenMatrix9")
MILESTONES = ROOT / "OM9_Milestones"

PACKAGES = {
    "m08": {
        "milestone_dir": "M08 - Advanced Jewelry Modeling",
        # 22 of the 73 E13 L3 are OM9-L2-04-01 transform toolbars owned by M03 and
        # explicitly executed in M08 without relocation, so their L4 land in the
        # M03 tree; the M08 tree holds the remaining 51 L3.
        "extra_milestone_dirs": ["M03 - Curve, Surface, Solid and 3DM Exchange"],
        "incoming": "build/m08-incoming/OpenMatrix9",
        "planning": "planning/M08",
        "manifest": "M08_L4_EXECUTION_MANIFEST.json",
        "manifest_out": "build/m08-merge-manifest.json",
        "reference_tag": "reference/m08-l4",
        # M08 labels follow the overlay's own "nn-nn-<slug>" phase folder names.
        "label_map": {
            "SOURCE_AND_CONTRACT": ("CONTRACT_FIXTURE", "Source And Interface Contract"),
            "DOMAIN_IMPLEMENTATION": ("IMPLEMENT", "Owner Operation Or Dispatch"),
            "MENU_BINDING_ACCEPTANCE": ("ACCEPTANCE", "Menu Binding Acceptance"),
            "FEBE_ACCEPTANCE": ("ACCEPTANCE", "Transaction And UI Acceptance"),
            "ADVERSARIAL_GATE": ("ADVERSARIAL", "Adversarial History Regression"),
        },
        "label_key": "work_kind",
        "label_order": ["SOURCE_AND_CONTRACT", "DOMAIN_IMPLEMENTATION",
                        "MENU_BINDING_ACCEPTANCE", "FEBE_ACCEPTANCE",
                        "ADVERSARIAL_GATE"],
        "l3_key": "parent_l3_id",
        "l2_key": "owner_module_id",
        "l4_path_key": "path",
        "task_md_key": "path",
        "planned_count_field": "planned_l4_count_m08",
        "reference_files": [
            "M08_L4_EXECUTION_MANIFEST.json", "M08_L4_EXECUTION_MANIFEST.csv",
            "PACKAGING_REPORT.json", "SELF_REVIEW.md", "RELEASE_GATE.md",
            "LUNA_START_HERE.md", "GPT6_LUNA_TASK_PROMPT.md", "README.md",
        ],
    },
    "m05": {
        "milestone_dir": "M05 - Gems and Layout",
        "incoming": "build/m05-incoming",
        "planning": "planning/M05",
        "manifest": "M05_L4_EXECUTION_MANIFEST.json",
        "manifest_out": "build/m05-merge-manifest.json",
        "reference_tag": "reference/m05-l4",
        # overlay phase/work-kind -> milestone L4 role and folder label
        "label_map": {
            "SOURCE_AUDIT_CONTRACT": ("SOURCE_MAP", "Source Map"),
            "DOMAIN_IMPLEMENTATION": ("IMPLEMENT", "Implementation"),
            "UI_INTEGRATION_AND_ACCEPTANCE": ("WIRE_UI", "UI Integration And Acceptance"),
        },
        "label_key": "phase",
        "label_order": ["SOURCE_AUDIT_CONTRACT", "DOMAIN_IMPLEMENTATION",
                        "UI_INTEGRATION_AND_ACCEPTANCE"],
        "l3_key": "parent_l3_id",
        "l2_key": "module_id",
        "l4_path_key": "l4_json_path",
        "task_md_key": "task_md_path",
        "planned_count_field": "planned_l4_count_m05",
        "reference_files": [
            "M05_L4_EXECUTION_MANIFEST.json", "M05_L4_EXECUTION_MANIFEST.csv",
            "M05_L4_COVERAGE_MATRIX.csv", "M05_L4_EXECUTION_WAVES.json",
            "M05_L4_REVIEW.md", "README.md", "RUNBOOK_GPT6_LUNA.md",
        ],
    },
    "m06": {
        "milestone_dir": "M06 - Settings and Prongs",
        "incoming": "build/m06-incoming/OpenMatrix9",
        "planning": "planning/M06",
        "manifest": "M06_EXECUTION_MANIFEST.json",
        "manifest_out": "build/m06-merge-manifest.json",
        "reference_tag": "reference/m06-l4",
        "label_map": {
            "CONTRACT": ("CONTRACT_FIXTURE", "Contract And Fixture"),
            "BE": ("IMPLEMENT", "Backend Implementation"),
            "FEBE": ("WIRE_UI", "Frontend Integration"),
            "QA": ("REGRESSION", "Regression And Acceptance"),
        },
        "label_key": "work_kind",
        "label_order": ["CONTRACT", "BE", "FEBE", "QA"],
        "l3_key": "l3_id",
        "l2_key": "module_id",
        "l4_path_key": "path",
        "task_md_key": "task_path",
        "planned_count_field": "planned_l4_count_m06",
        "reference_files": [
            "M06_EXECUTION_MANIFEST.json", "M06_EXECUTION_MANIFEST.csv",
            "M06_MANUAL_TRACE.csv", "M06_REVIEW_REPORT.md",
            "M06_IMPLEMENTATION_CONTRACT.md", "M06_GATE_CHECKLIST.md",
            "M06_EVIDENCE_TEMPLATE.json", "M06_EXECUTION_TREE.txt",
            "GPT6_LUNA_EXECUTION_PROMPT.md", "README.md",
        ],
    },
    "m07": {
        "milestone_dir": "M07 - Cutters and Manufacturing Validation",
        "incoming": "build/m07-incoming/OpenMatrix9",
        "planning": "planning/M07",
        "manifest": "M07_L4_EXECUTION_MANIFEST.json",
        "manifest_out": "build/m07-merge-manifest.json",
        "reference_tag": "reference/m07-l4",
        "label_map": {
            "SOURCE_AND_CONTRACT": ("CONTRACT_FIXTURE", "Contract And Source Preflight"),
            "OWNER_INTEGRATION": ("IMPLEMENT", "Domain Integration"),
            "BE_EXPORT_ACCEPTANCE": ("ACCEPTANCE", "Export Acceptance"),
            "FEBE_ACCEPTANCE": ("ACCEPTANCE", "UI Or External Acceptance"),
            "ADVERSARIAL_GATE": ("ADVERSARIAL", "Adversarial Release Evidence"),
        },
        "label_key": "work_kind",
        "label_order": ["SOURCE_AND_CONTRACT", "OWNER_INTEGRATION", "BE_EXPORT_ACCEPTANCE",
                        "FEBE_ACCEPTANCE", "ADVERSARIAL_GATE"],
        "l3_key": "parent_l3_id",
        # OM9-L3-aa-bb-ccc -> OM9-L2-aa-bb (the L3 id carries its owner module).
        "l2_key": lambda item: "OM9-L2-" + "-".join(item["parent_l3_id"].split("-")[2:4]),
        "l4_path_key": "path",
        "task_md_key": "path",
        "planned_count_field": "planned_l4_count_m07",
        "reference_files": [
            "M07_L4_EXECUTION_MANIFEST.json", "M07_L4_EXECUTION_MANIFEST.csv",
            "PACKAGING_REPORT.json", "SELF_REVIEW.md", "RELEASE_GATE.md",
            "LUNA_START_HERE.md", "GPT6_LUNA_TASK_PROMPT.md", "README.md",
        ],
    },
}

OVERLAY_ID_RE = re.compile(r"^OM9-M\d\d-(L4-\d{2}-\d{3}-\d{2})$")
# M07 already ships milestone-form ids and keeps the module ordinal, e.g.
# OM9-L4-01-12-008-01; both forms normalise to OM9-L4-<...>.
MILESTONE_ID_RE = re.compile(r"^OM9-(L4(?:-\d{2})?-\d{2}-\d{3}-\d{2})$")


def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def write(path, data):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def normalise_id(overlay_id):
    """Map either id form onto the milestone form OM9-L4-... ."""
    match = OVERLAY_ID_RE.match(overlay_id or "") or MILESTONE_ID_RE.match(overlay_id or "")
    return "OM9-" + match.group(1) if match else None


def dep_list(value):
    if not value:
        return []
    if isinstance(value, str):
        return [part for part in value.split(";") if part]
    return list(value)


def find_milestone_dir(name):
    for path in MILESTONES.iterdir():
        if path.is_dir() and path.name.startswith(name.split(" ")[0] + " "):
            return path
    raise SystemExit(f"milestone dir not found: {name}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--package", required=True, choices=sorted(PACKAGES))
    parser.add_argument("--apply", action="store_true")
    args = parser.parse_args()

    spec = PACKAGES[args.package]
    l2_key = spec["l2_key"]
    if not callable(l2_key):
        l2_key = (lambda key: (lambda item: item[key]))(l2_key)
    current = find_milestone_dir(spec["milestone_dir"])
    incoming = ROOT / spec["incoming"]
    planning = incoming / spec["planning"]
    manifest = read(planning / spec["manifest"])
    items = manifest["items"]
    report = {
        "schema": f"om9.{args.package}.l4_overlay_merge.v1",
        "package": args.package,
        "manifest_declared": {k: v for k, v in manifest.items() if k != "items"},
        "manifest_items": len(items),
        "strategy": "place L4+TASK.md into the milestone tree; normalise ids; fill children_l4/planning state",
        "l4_written": [], "id_rewrites": [], "updated": [], "reference_files": [],
        "conflicts": [], "skipped": [],
    }

    # A package may own L3 that live in another milestone's tree (M08 executes the
    # M03 transform toolbars), so index every declared milestone and remember which
    # one holds each L3 before writing anything.
    l2_dirs, l3_dirs, l3_owner, l2_owner = {}, {}, {}, {}
    for milestone_name in [spec["milestone_dir"], *spec.get("extra_milestone_dirs", [])]:
        milestone_dir = find_milestone_dir(milestone_name)
        for l2dir in sorted(p for p in milestone_dir.iterdir()
                            if p.is_dir() and not p.name.startswith("reference")):
            l2 = read(l2dir / "L2.json")
            if l2["id"] in l2_dirs:
                raise SystemExit(f"duplicate L2 id across milestones: {l2['id']}")
            l2_dirs[l2["id"]] = l2dir
            l2_owner[l2["id"]] = milestone_dir
            for l3dir in sorted(p for p in l2dir.iterdir() if p.is_dir()):
                f = l3dir / "l3.json"
                if not f.is_file():
                    continue
                l3_id = read(f)["id"]
                if l3_id in l3_dirs:
                    raise SystemExit(f"duplicate L3 id across milestones: {l3_id}")
                l3_dirs[l3_id] = l3dir
                l3_owner[l3_id] = milestone_dir
    if not l2_dirs or not l3_dirs:
        raise SystemExit("milestone tree not found or incomplete")

    def milestone_rel(path):
        """Path relative to OM9_Milestones, for reports."""
        return path.relative_to(MILESTONES).as_posix()

    def milestone_code(path):
        """Milestone directory name -> its short code (M08, M03, ...)."""
        return path.name.split(" ")[0]

    per_l3 = {}
    for item in items:
        per_l3.setdefault(item[spec["l3_key"]], []).append(item)

    l3_children = {}
    for l3_id, entries in per_l3.items():
        if l3_id not in l3_dirs:
            report["conflicts"].append(f"{l3_id}: overlay parent L3 not present locally")
            continue
        entries.sort(key=lambda r: (spec["label_order"].index(r[spec["label_key"]]),
                                    r["task_id"]))
        children = []
        for index, item in enumerate(entries, start=1):
            overlay_id = item["task_id"]
            new_id = normalise_id(overlay_id)
            if new_id is None:
                report["conflicts"].append(f"{overlay_id}: unrecognised overlay id form")
                continue
            role, label = spec["label_map"][item[spec["label_key"]]]
            l4dir = l3_dirs[l3_id] / f"L4-{index:02d} - {label}"
            src = incoming / item[spec["l4_path_key"]]
            task_md = incoming / item[spec["task_md_key"]]
            if not src.is_file() or not task_md.is_file():
                report["conflicts"].append(f"{overlay_id}: overlay task files missing")
                continue
            record = read(src)
            if record.get("task_id") != overlay_id:
                report["conflicts"].append(f"{overlay_id}: overlay task_id mismatch")
                continue
            # Never clobber an L4 that is already in the tree (it may carry anchor
            # supplements added by tools/apply_matrix_ref_supplements.py).
            if (l4dir / "l4.json").is_file():
                report["skipped"].append(f"{overlay_id}: L4 already present")
                continue
            for key in ("id", "task_id"):
                record[key] = new_id
            record["level"] = "L4"
            record["task_role"] = role
            record["overlay_task_id"] = overlay_id
            record["overlay_" + spec["label_key"]] = item[spec["label_key"]]
            if "owner_module_id" in record:
                record["owner_l2_id"] = record["owner_module_id"]
            deps = [d for d in (normalise_id(x) for x in dep_list(item.get("depends_on_task_ids")))
                    if d]
            record["depends_on_l4_ids"] = deps
            for key in ("evidence_path", "manual_signoff_required", "depends_on_module_ids",
                        "owner_capability_id", "risk", "execution_wave", "execution_stage",
                        "phase"):
                if key in item:
                    record[key] = item[key]
            record["reference_policy"] = (
                "Planning-only record: no build, runtime or geometry evidence is claimed. "
                "Anchor supplements are added by tools/apply_matrix_ref_supplements.py."
            )
            rel_file = f"{milestone_rel(l4dir)}/l4.json"
            children.append({
                "id": new_id,
                "file": rel_file,
                "title": f"{label} — {record['title'].split(' — ')[0]}",
                "role": role,
                "depends_on_l4_ids": deps,
            })
            if overlay_id != new_id:
                report["id_rewrites"].append({"overlay": overlay_id, "milestone": new_id})
            report["l4_written"].append(rel_file)
            if args.apply:
                write(l4dir / "l4.json", record)
                shutil.copy2(task_md, l4dir / "TASK.md")
        if children:
            l3_children[l3_id] = children

    for l3_id, children in sorted(l3_children.items()):
        path = l3_dirs[l3_id] / "l3.json"
        data = read(path)
        before = [len(data.get("children_l4") or []), data.get("l4_status")]
        # Re-running the merge must not rewrite what an earlier run already recorded.
        if (data.get("children_l4") or []) == children:
            report["updated"].append({"file": milestone_rel(path),
                                      "children_l4": len(children), "was": before,
                                      "change": "none"})
            continue
        data["children_l4"] = children
        data["l4_status"] = "PLANNED_NOT_EXECUTED"
        if data.get("planning_state") == "SPEC_ONLY_NOT_IMPLEMENTED":
            data["planning_state"] = "SPECIFIED_L4_PLANNED_NOT_IMPLEMENTED"
        spec_block = data.setdefault("specification", {})
        if "l4_status" in spec_block:
            spec_block["l4_status"] = "L4_CREATED_PLANNING_ONLY"
        if "status" in spec_block and str(spec_block["status"]).startswith("L3_SPEC_ONLY"):
            spec_block["status"] = "L3_SPECIFIED_L4_PLANNED_NOT_EXECUTED"
        if "execution_status" in spec_block:
            spec_block["execution_status"] = "L4_PLANNED_NOT_EXECUTED"
        report["updated"].append({"file": f"{milestone_rel(path)}",
                                  "children_l4": len(children), "was": before})
        if args.apply:
            write(path, data)

    per_l2 = {}
    for l3_id, children in l3_children.items():
        parent = read(l3_dirs[l3_id] / "l3.json")["parent_l2_id"]
        bucket = per_l2.setdefault(parent, {"milestone": l2_owner[parent], "count": 0})
        bucket["count"] += len(children)
    for l2_id, bucket in sorted(per_l2.items()):
        count = bucket["count"]
        path = l2_dirs[l2_id] / "L2.json"
        data = read(path)
        data["l4_status"] = "PLANNED_NOT_EXECUTED"
        data[spec["planned_count_field"]] = count
        if data.get("planning_state") == "SPEC_ONLY_NOT_IMPLEMENTED":
            data["planning_state"] = "SPECIFIED_L4_PLANNED_NOT_IMPLEMENTED"
        definition = data.setdefault("definition", {})
        definition["l3_l4_status"] = "L3_DEFINED_L4_PLANNED_NOT_IMPLEMENTED"
        if args.apply:
            write(path, data)
        report["updated"].append({"file": f"{milestone_rel(path)}",
                                  spec["planned_count_field"]: count})

    l1_path = current / "L1.json"
    l1 = read(l1_path)
    # L4 placed in an extra milestone's tree still belong to this package, so count
    # them here and say where they live. Counting the L3 records (not just the ones
    # written on this run) keeps a re-run idempotent.
    package_l3 = {}
    for l3_id in per_l3:
        path = l3_dirs.get(l3_id)
        if path is None:
            continue
        children = (read(path / "l3.json").get("children_l4") or [])
        if children:
            package_l3[l3_id] = children
    total_children = sum(len(c) for c in package_l3.values())
    l1["l4_count"] = total_children
    l1["l4_count_in_tree"] = sum(
        len(children) for l3_id, children in package_l3.items()
        if l3_owner[l3_id] == current
    )
    cross = sorted({milestone_code(l3_owner[l3_id])
                    for l3_id in package_l3 if l3_owner[l3_id] != current})
    if cross:
        l1["l4_executed_in_milestones"] = [milestone_code(current), *cross]
        l3_home = {}
        l4_home = {}
        for l3_id, children in package_l3.items():
            home = milestone_code(l3_owner[l3_id])
            l3_home[home] = l3_home.get(home, 0) + 1
            l4_home[home] = l4_home.get(home, 0) + len(children)
        l1["l3_spec_tree_by_milestone"] = dict(sorted(l3_home.items()))
        l1["l4_placed_by_milestone"] = dict(sorted(l4_home.items()))
        l1["cross_milestone_note"] = (
            f"E13 executes these L3 in {milestone_code(current)} while each keeps its "
            "canonical owner milestone and L3 spec tree (no relocation)."
        )
    l1["planning_state"] = "L1_L4_SPECIFIED_NOT_IMPLEMENTED"
    # M07's manifest declares no explicit precondition list; its stage/gate
    # markers carry the same meaning.
    blocked = manifest.get("preconditions")
    if not blocked:
        blocked = ["G00 PASS", f"prior stage gate before {manifest.get('stage', 'this stage')} PASS",
                   "source/OCCT/legal approved"]
    readiness = {
        "target": "Planning-only L4 records; no binary, runtime receipts or changelog",
        "blocked_until": blocked,
        "l4_planned": l1["l4_count"],
        "l3_covered": len(package_l3),
    }
    # Packages differ in how they declare the gate and the upstream baseline.
    for key in ("gate", "execution_stage", "stage", "source_audit_status",
                "qymcad_reviewed_revision", "upstream_baseline_status", "status"):
        if key in manifest:
            readiness[key] = manifest[key]
    if "manual_basis" in manifest:
        readiness["manual_basis"] = manifest["manual_basis"]
    if items and "source_revision_expected_from_M00" in items[0]:
        readiness.setdefault("source_pin", "746a17f127897e921a57118b6caad08968602bf5")
    l1["execution_readiness"] = readiness
    if args.apply:
        write(l1_path, l1)
    report["l1"] = {"l4_count": l1["l4_count"], "l3_covered": len(package_l3)}
    if args.apply:
        (current / spec["reference_tag"]).mkdir(parents=True, exist_ok=True)
    for name in spec["reference_files"]:
        src = planning / name
        if not src.is_file():
            report["skipped"].append(name)
            continue
        report["reference_files"].append(f"{spec['reference_tag']}/{name}")
        if args.apply:
            shutil.copy2(src, current / spec["reference_tag"] / name)

    out = ROOT / spec["manifest_out"]
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(report, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")
    print(json.dumps({
        "package": args.package, "apply": args.apply,
        "overlay_records": len(items),
        "l4_written": len(report["l4_written"]),
        "id_rewrites": len(report["id_rewrites"]),
        "l3_with_children": len(package_l3),
        "l2_updated": len(per_l2),
        "l1_l4_count": l1["l4_count"],
        "reference_files": len(report["reference_files"]),
        "conflicts": len(report["conflicts"]),
        "skipped": report["skipped"],
    }, indent=1))
    for entry in report["conflicts"][:10]:
        print("CONFLICT:", entry)


if __name__ == "__main__":
    main()
