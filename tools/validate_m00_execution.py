#!/usr/bin/env python
"""Validate M00 execution evidence against `om9.m00.evidence.v013`.

    python tools/validate_m00_execution.py               # validate every record
    python tools/validate_m00_execution.py --self-test   # prove the validator rejects forged records
    python tools/validate_m00_execution.py --require-ready  # the gate-readiness decision

Readiness asks for more than "a file exists": every M00 L4 must be either an accepted v0.13 record
(PASS, real execution, artifacts, a reviewer who approved) or marked EVIDENCED by a gate closeout, and
every gate whose L4 are all evidenced must carry a review record for each of its L3. Anything else is
reported, and the command exits 1.
"""
from __future__ import annotations

import argparse
import copy
import sys
import tempfile

import m00_common as m00


def self_test() -> int:
    """A validator that cannot reject a forged record is worthless; prove each rejection."""
    good = {
        "task_id": "OM9-M00-L4-01-01-02",
        "source_sha": m00.QYMCAD_SHA,
        "status": "PASS",
        "actual_execution": True,
        "timestamp_utc": None,
        "os_arch": None,
        "toolchain": {},
        "commands": [
            {
                "command": "git -C QymCAD rev-parse HEAD",
                "exit_code": 0,
                "stdout_path": "implementation/evidence/M00/G00-01/x.log",
                "stderr_path": "implementation/evidence/M00/G00-01/x.log",
            }
        ],
        "artifacts": [{"path": "implementation/evidence/M00/G00-01/x.log", "sha256": "a" * 64}],
        "expected": ["HEAD == 746a17f1"],
        "actual": ["HEAD == 746a17f1 (observed)"],
        "review": {"decision": "APPROVED", "reviewer": "reviewer", "evidence_checked": True},
    }
    cases = []
    baseline = copy.deepcopy(good)
    cases.append(("baseline", baseline, True))

    no_artifacts = copy.deepcopy(good)
    no_artifacts["artifacts"] = []
    cases.append(("PASS without artifacts", no_artifacts, False))

    pending = copy.deepcopy(good)
    pending["review"] = {"decision": "PENDING", "reviewer": None, "evidence_checked": False}
    cases.append(("PASS with a pending review", pending, False))

    copied = copy.deepcopy(good)
    copied["actual"] = list(copied["expected"])
    cases.append(("actual copied from expected", copied, False))

    wrong_sha = copy.deepcopy(good)
    wrong_sha["source_sha"] = "c" * 40
    cases.append(("wrong source_sha", wrong_sha, False))

    bad_artifact = copy.deepcopy(good)
    bad_artifact["artifacts"] = [{"path": "x", "sha256": "not-a-digest"}]
    cases.append(("bad artifact digest", bad_artifact, False))

    failed = 0
    for name, record, should_pass in cases:
        errors = m00.schema_errors(record)
        accepted = m00.record_accepted(record)
        passes = not errors and accepted
        verdict = "accepted" if passes else "rejected"
        expected = "accepted" if should_pass else "rejected"
        ok = verdict == expected
        print(f"{'ok  ' if ok else 'FAIL'} {name}: {verdict} (expected {expected})")
        if not ok:
            failed += 1
    print(f"self-test: {len(cases) - failed}/{len(cases)} correct")
    return 0 if failed == 0 else 1


def validate_all() -> int:
    invalid = 0
    accepted = 0
    for path, record in m00.evidence_records():
        errors = m00.schema_errors(record)
        if m00.record_accepted(record):
            accepted += 1
        if errors:
            invalid += 1
            print(f"INVALID {path.relative_to(m00.ROOT)}")
            for error in errors:
                print("   -", error)
    print(f"records {len(m00.evidence_records())} | accepted {accepted} | invalid {invalid}")
    return 0 if invalid == 0 else 1


def require_ready() -> int:
    accepted = m00.accepted_l4()
    missing = [entry["id"] for entry in m00.iter_l4() if entry["id"] not in accepted]
    # A gate whose L4 are all evidenced still needs a review record per L3.
    unsigned = []
    for gate_dir in sorted(m00.M00.iterdir()):
        if not gate_dir.is_dir() or not gate_dir.name.startswith("L2-"):
            continue
        l2 = m00.load(gate_dir / "L2.json")
        gate = l2.get("gate_id")
        if not gate:
            continue
        for child in l2.get("children_l3") or []:
            l3_dir = m00.M00 / child.get("file", "")
            if not l3_dir.exists():
                continue
            l3 = m00.load(l3_dir)
            l3_id = str(l3.get("id") or "")
            l4_ids = [str(item.get("id")) for item in (l3.get("children_l4") or [])]
            if l4_ids and all(item in accepted for item in l4_ids):
                if not list((m00.EVIDENCE / l3_id).glob("*.ui-review.json")) if (m00.EVIDENCE / l3_id).is_dir() else True:
                    unsigned.append(l3_id)
    print(f"accepted L4: {len(accepted)}/{len(list(m00.iter_l4()))}")
    if missing:
        print(f"OPEN L4 ({len(missing)}):")
        for item in missing[:20]:
            print("  -", item)
    if unsigned:
        print(f"L3 that need a review record ({len(unsigned)}):")
        for item in unsigned[:20]:
            print("  -", item)
    ready = not missing and not unsigned
    print("READY" if ready else "NOT READY")
    return 0 if ready else 1


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--require-ready", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        return self_test()
    if args.require_ready:
        return require_ready()
    return validate_all()


if __name__ == "__main__":
    sys.exit(main())
