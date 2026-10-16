#!/usr/bin/env python3
"""Validate the M00 L2-01 blocked-status evidence records against the documented M00 schema.

The M00 package (`OM9_Milestones/M00 .../L1.json`) declares an execution-evidence schema
`om9.m00.evidence.v013` and its runbook references `tools/validate_m00_execution.py`. That tool is
not present in this checkout, so this is a deliberately narrow substitute for one gate only
(`OM9-M00-L2-01`, gate `G00-01`). It checks structure and honesty markers, not scientific truth.

It refuses to accept a PASS record, because no execution evidence for this gate exists.

Run:  python tools/validate_m00_l2_01_evidence.py
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "implementation" / "evidence" / "M00" / "G00-01"
PIN = "746a17f127897e921a57118b6caad08968602bf5"

REQUIRED = ["task_id", "source_sha", "status", "actual_execution", "commands", "artifacts", "expected", "actual", "review"]
STATUSES = {
    "NOT_RUN", "PASS", "FAIL", "BLOCKED_SOURCE", "BLOCKED_DEPENDENCY",
    "BLOCKED_ENV", "BLOCKED_FIXTURE", "BLOCKED_APPROVAL", "NOT_TARGETED",
}
EXPECTED_TASKS = [f"OM9-M00-L4-01-{block:02d}-{index:02d}" for block in (1, 2, 3) for index in (1, 2, 3, 4)]
HEX64 = re.compile(r"^[a-f0-9]{64}$")


def check(path: Path) -> tuple[list[str], str | None, str | None]:
    problems: list[str] = []
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:  # noqa: BLE001 - report any parse failure as a problem
        return [f"unreadable json: {exc}"], None, None

    for key in REQUIRED:
        if key not in data:
            problems.append(f"missing required field '{key}'")
    if data.get("source_sha") != PIN:
        problems.append("source_sha is not the M00 pinned revision")
    if data.get("status") not in STATUSES:
        problems.append(f"unknown status {data.get('status')!r}")
    if data.get("status") == "PASS":
        problems.append("PASS record rejected: no reviewed evidence exists for this gate")
    if data.get("actual_execution") is True:
        if data.get("status") == "NOT_RUN":
            problems.append("actual_execution=true while status is NOT_RUN")
        if not data.get("probe_result"):
            problems.append("actual_execution=true without a probe_result")
        if not [a for a in (data.get("artifacts") or []) if a.get("sha256")]:
            problems.append("actual_execution=true without any digested artifact")
    if not re.match(r"^OM9-M00-L4-\d{2}-\d{2}-\d{2}$", str(data.get("task_id", ""))):
        problems.append("task_id does not match ^OM9-M00-L4-\\d{2}-\\d{2}-\\d{2}$")

    review = data.get("review") or {}
    if review.get("decision") not in ("PENDING", "APPROVED", "REJECTED"):
        problems.append("review.decision is not PENDING/APPROVED/REJECTED")
    if review.get("decision") == "APPROVED" and not review.get("reviewer"):
        problems.append("approved review without a reviewer")
    if review.get("decision") == "APPROVED" and not review.get("evidence_checked"):
        problems.append("approved review without evidence_checked")

    for command in data.get("commands") or []:
        missing = [k for k in ("command", "exit_code", "stdout_path", "stderr_path") if k not in command]
        if missing:
            problems.append(f"command entry missing {missing}")
            break

    for artifact in data.get("artifacts") or []:
        if not HEX64.match(str(artifact.get("sha256", ""))):
            problems.append(f"artifact {artifact.get('path')!r} has no sha256 digest")
            break

    return problems, str(data.get("task_id")), str(data.get("status"))


def main() -> int:
    if not EVIDENCE.is_dir():
        print(f"FAIL: evidence directory missing: {EVIDENCE}")
        return 1

    found: dict[str, str] = {}
    failures = 0
    for path in sorted(EVIDENCE.glob("*.evidence.json")):
        problems, task_id, status = check(path)
        label = task_id or path.name
        if problems:
            failures += 1
            print(f"FAIL {label}")
            for problem in problems:
                print(f"       - {problem}")
        else:
            found[task_id] = status
            print(f"OK   {label}  {status}")

    for task_id in EXPECTED_TASKS:
        if task_id not in found:
            failures += 1
            print(f"FAIL {task_id}  no evidence record")

    promoted = [t for t, s in found.items() if s == "PASS"]
    if promoted:
        failures += 1
        print(f"FAIL unexpected PASS: {promoted}")

    print("")
    print(f"records={len(found)}/{len(EXPECTED_TASKS)} failures={failures}")
    print("gate G00-01 remains open; no status was promoted")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
