# SPDX-License-Identifier: LGPL-2.1-or-later
"""Read-only source/document checks for this handoff; never runs product tests."""
import argparse
import csv
import hashlib
import json
from pathlib import Path
import re


def validate(project_root: Path, folder: Path) -> dict:
    required = [
        "README.md", "SPEC.vi.md", "COMMAND-MATRIX.vi.md", "OWNER-CARDS.vi.md",
        "TASKS.vi.md", "LUNA-PROMPT.vi.md", "ACCEPTANCE.csv", "source-inventory.json",
    ]
    errors = []
    for name in required:
        if not (folder / name).is_file():
            errors.append(f"missing bundle file: {name}")
    if errors:
        return {"status": "FAIL", "errors": errors}

    inventory = json.loads((folder / "source-inventory.json").read_text(encoding="utf-8"))
    groups = inventory["groups"]
    adapter = (project_root / "crates/om9-app/src/session/command_options.rs").read_text(encoding="utf-8")
    providers = re.findall(r'\(\s*"(\w+)",\s*Controller::(\w+_option_tokens),?\s*\)', adapter)
    if len(providers) != inventory["registered_provider_count"]:
        errors.append("registered provider count changed; refresh and review inventory")
    if {name for name, _ in providers} != {g["owner"] for g in groups} - {"line", "polyline"}:
        errors.append("registered owners differ from inventory")
    if len(groups) != inventory["existing_option_group_count"] or len({g["owner"] for g in groups}) != len(groups):
        errors.append("group count/uniqueness mismatch")

    session = (project_root / "crates/om9-app/src/session.rs").read_text(encoding="utf-8")
    fields = re.search(r"pub struct Controller \{(.*?)\n\}", session, re.S).group(1)
    pending = set(re.findall(r"(\w+_pending): Option<[^\n]+>", fields))
    covered = {g["pending_field"] for g in groups} - {"session"}
    extras = {p["field"] for p in inventory["additional_pending_owners"]}
    if pending != covered | extras or covered & extras:
        errors.append("pending coverage differs or is duplicated")
    if len(pending) != inventory["controller_pending_field_count"] or len(extras) != inventory["additional_pending_owner_count"]:
        errors.append("pending counts do not match source")
    for name, expected in inventory["source_sha256"].items():
        path = project_root / name
        if not path.is_file() or hashlib.sha256(path.read_bytes()).hexdigest() != expected:
            errors.append(f"source changed/missing: {name}")

    cards = (folder / "OWNER-CARDS.vi.md").read_text(encoding="utf-8")
    for g in groups:
        if not re.search(r"\b" + re.escape(g["owner"]) + r"\b", cards, re.I):
            errors.append(f"owner card missing: {g['owner']}")
    for field in extras:
        if field not in cards:
            errors.append(f"additional pending card missing: {field}")
    tasks = (folder / "TASKS.vi.md").read_text(encoding="utf-8")
    task_ids = re.findall(r"^## (P\d\d) —", tasks, re.M)
    if task_ids != [f"P{i:02}" for i in range(13)]:
        errors.append("task sequence is not P00..P12")

    with (folder / "ACCEPTANCE.csv").open(encoding="utf-8", newline="") as handle:
        cases = list(csv.DictReader(handle))
    if [c.get("case_id") for c in cases] != [f"AC{i:03}" for i in range(1, 51)]:
        errors.append("acceptance case IDs/count mismatch")
    for case in cases:
        if case.get("status") != "NOT_RUN_TEMPLATE" or any(v is None or not str(v).strip() for v in case.values()):
            errors.append(f"invalid acceptance template: {case.get('case_id')}")

    links = 0
    for doc in folder.glob("*.md"):
        text = doc.read_text(encoding="utf-8")
        for target in re.findall(r"\[[^\]]+\]\(([^)]+)\)", text):
            if "://" in target or target.startswith(("mailto:", "#")):
                continue
            target = target.split("#", 1)[0].strip("<>")
            links += 1
            if target and not (doc.parent / target).resolve().exists():
                errors.append(f"broken link in {doc.name}: {target}")

    return {
        "status": "PASS" if not errors else "FAIL",
        "scope": "Documentation/source inventory consistency only; no product implementation or runtime acceptance claimed.",
        "date": "2026-10-26",
        "group_count": len(groups), "registered_provider_count": len(providers),
        "controller_pending_fields": len(pending), "additional_pending_fields": len(extras),
        "owner_cards_checked": len(groups), "task_packages": len(task_ids),
        "acceptance_templates": len(cases), "acceptance_executions": 0,
        "source_hashes_checked": len(inventory["source_sha256"]),
        "local_links_checked": links, "errors": errors,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--project-root", type=Path, default=Path(__file__).resolve().parents[4])
    parser.add_argument("--write-report", action="store_true")
    args = parser.parse_args()
    folder = Path(__file__).resolve().parent
    report = validate(args.project_root.resolve(), folder)
    if args.write_report:
        (folder / "documentation-validation.json").write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(report, ensure_ascii=False, indent=2))
    return 0 if report["status"] == "PASS" else 1


if __name__ == "__main__":
    raise SystemExit(main())
