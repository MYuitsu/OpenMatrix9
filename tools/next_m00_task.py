#!/usr/bin/env python
"""Print the single M00 task the protocol allows next.

Order is gate G00-01..G00-07, then L2, L3, L4 as the tree lists them. A task is **done** when it has an
accepted `om9.m00.evidence.v013` record (PASS + APPROVED + artifacts) or when a gate closeout marks it
EVIDENCED. The first task that is not done is printed with the runbook paths the protocol asks for, and
the command exits:

* 0 — an actionable task was printed;
* 2 — the next task is blocked (approval, dependency, source, environment, fixture);
* 3 — nothing is left in M00.

The tool never runs the task and never grants PASS.
"""
from __future__ import annotations

import sys

import m00_common as m00


def main() -> int:
    accepted = m00.accepted_l4()
    blocked: dict[str, str] = {}
    for _path, record in m00.evidence_records():
        task = str(record.get("task_id") or "")
        if task and not m00.record_accepted(record):
            blocked[task] = m00.record_status(record)

    for entry in m00.iter_l4():
        if entry["id"] in accepted:
            continue
        document = entry["l4"]
        status = blocked.get(entry["id"], "NOT_RUN")
        approval = entry["id"] in m00.HUMAN_APPROVAL
        print(f"NEXT {entry['id']} — {entry['title']}")
        print(f"  gate: {entry['gate']}  status: {status}{'  (human approval required)' if approval else ''}")
        print(f"  runbook: {entry['l4_path'].relative_to(m00.ROOT) / 'RUNBOOK.md'}")
        print(f"  contract: {entry['l4_path'].relative_to(m00.ROOT) / 'execution_contract.json'}")
        print(f"  record: {entry['l4_path'].relative_to(m00.ROOT) / 'l4.json'}")
        print(f"  evidence dir: implementation/evidence/M00/{entry['gate']}/")
        if status in m00.BLOCKED or approval:
            print(f"  BLOCKED: {status if status != 'NOT_RUN' else 'not started'}"
                  f"{'; the reviewer must sign' if approval else ''}")
            return 2
        return 0
    print("NOTHING LEFT: every M00 L4 is either accepted or marked EVIDENCED by a closeout")
    return 3


if __name__ == "__main__":
    sys.exit(main())
