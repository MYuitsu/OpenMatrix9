"""Shared readers for the M00 gate records.

Used by `validate_m00_v013.py`, `next_m00_task.py`, `validate_m00_execution.py` and the two capture
helpers, so all five agree on what a task, an evidence record and a closeout entry are.
"""
from __future__ import annotations

import hashlib
import json
import pathlib
import re

ROOT = pathlib.Path(__file__).resolve().parents[1]
M00 = ROOT / "OM9_Milestones/M00 - Feasibility, Legal"
EVIDENCE = ROOT / "implementation/evidence/M00"
QYMCAD_SHA = "746a17f127897e921a57118b6caad08968602bf5"
L4_ID = re.compile(r"^OM9-M00-L4-\d{2}-\d{2}-\d{2}$")
GATE_ORDER = [f"G00-0{index}" for index in range(1, 8)]
HUMAN_APPROVAL = {
    "OM9-M00-L4-01-01-01",
    "OM9-M00-L4-01-03-03",
    "OM9-M00-L4-06-01-04",
    "OM9-M00-L4-06-02-04",
}
STATUSES = {
    "NOT_RUN",
    "PASS",
    "FAIL",
    "BLOCKED_SOURCE",
    "BLOCKED_DEPENDENCY",
    "BLOCKED_ENV",
    "BLOCKED_FIXTURE",
    "BLOCKED_APPROVAL",
    "NOT_TARGETED",
}
BLOCKED = {
    "NOT_RUN",
    "BLOCKED_SOURCE",
    "BLOCKED_DEPENDENCY",
    "BLOCKED_ENV",
    "BLOCKED_FIXTURE",
    "BLOCKED_APPROVAL",
    "FAIL",
}


def load(path: pathlib.Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def sha256(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def resolve(recorded: str) -> pathlib.Path:
    """`children_*.file` is recorded relative to OM9_Milestones, not to the repository root."""
    return (M00.parent / recorded).resolve()


def gate_of(l2_path: pathlib.Path) -> str:
    return str(load(l2_path).get("gate_id") or "")


def iter_l4():
    """Yield every L4 of M00 in gate, L2, L3, L4 order."""
    gates = []
    for l2_dir in sorted(p for p in M00.iterdir() if p.is_dir() and p.name.startswith("L2-")):
        gate = gate_of(l2_dir / "L2.json")
        if not gate:
            continue
        l3_dirs = sorted(p for p in l2_dir.iterdir() if p.is_dir() and p.name.startswith("L3-"))
        gates.append((gate, l2_dir, l3_dirs))
    order = {gate: index for index, gate in enumerate(GATE_ORDER)}
    for gate, l2_dir, l3_dirs in sorted(gates, key=lambda entry: order.get(entry[0], 99)):
        for l3_dir in l3_dirs:
            l3 = load(l3_dir / "l3.json")
            l4_dirs = sorted(p for p in l3_dir.iterdir() if p.is_dir() and p.name.startswith("L4-"))
            for l4_dir in l4_dirs:
                path = l4_dir / "l4.json"
                if not path.exists():
                    continue
                document = load(path)
                yield {
                    "gate": gate,
                    "l2": l2_dir,
                    "l3": l3,
                    "l3_path": l3_dir,
                    "l4": document,
                    "l4_path": l4_dir,
                    "id": str(document.get("id") or ""),
                    "title": str(document.get("title") or l4_dir.name),
                }


def evidence_records() -> list[tuple[pathlib.Path, dict]]:
    records = []
    for path in sorted(EVIDENCE.rglob("*.evidence.json")):
        try:
            records.append((path, load(path)))
        except json.JSONDecodeError:
            records.append((path, {}))
    return records


def closeout_evidenced() -> dict[str, str]:
    """L4 ids that a gate closeout marks EVIDENCED, mapped to the gate that says so."""
    found: dict[str, str] = {}
    for path in sorted(EVIDENCE.glob("G00-0*/GATE-CLOSEOUT.md")):
        gate = path.parent.name
        for line in path.read_text(encoding="utf-8").splitlines():
            if not line.startswith("| L4-") or "EVIDENCED" not in line:
                continue
            short = line.split("|")[1].strip()
            found[f"OM9-M00-{short}"] = gate
    return found



def signed_l3() -> set[str]:
    """L3 whose review record is signed (or AUTO_ONLY with automated evidence)."""
    signed: set[str] = set()
    for path in EVIDENCE.rglob("*.ui-review.json"):
        try:
            document = json.loads(path.read_text(encoding="utf-8"))
        except json.JSONDecodeError:
            continue
        reviewer = document.get("reviewer") or {}
        status = str(reviewer.get("status", "")).upper()
        if status == "SIGNED" or (status == "AUTOMATED_EVIDENCE_ONLY" and str(document.get("manual_test_gate", "")).upper() == "AUTO_ONLY"):
            l3_id = str(document.get("l3_id") or path.parent.name)
            signed.add(l3_id)
    return signed


def tree_evidenced() -> set[str]:
    """L4 that an L3 record lists in `execution_evidence.l4_done`."""
    done: set[str] = set()
    for l3_path in M00.rglob("L3-*/l3.json"):
        l3 = load(l3_path)
        evidence_block = l3.get("execution_evidence") or {}
        done.update(str(item) for item in (evidence_block.get("l4_done") or []))
    return done


def accepted_l4() -> set[str]:
    """Everything the protocol counts as evidenced: a real record, a closeout row, or a tree list."""
    accepted: set[str] = set()
    for _path, record in evidence_records():
        if record_accepted(record):
            accepted.add(str(record.get("task_id") or ""))
    accepted |= set(closeout_evidenced())
    accepted |= tree_evidenced()
    for l3_path in M00.rglob("L3-*/l3.json"):
        l3 = load(l3_path)
        if str(l3.get("id") or "") in signed_l3():
            accepted.update(str(item.get("id")) for item in (l3.get("children_l4") or []))
    return accepted


def record_status(record: dict) -> str:
    return str(record.get("status") or "NOT_RUN")


def record_accepted(record: dict) -> bool:
    """A record counts only when it claims PASS, ran for real, and a reviewer approved it."""
    review = record.get("review") or {}
    return (
        record_status(record) == "PASS"
        and record.get("actual_execution") is True
        and review.get("decision") == "APPROVED"
        and bool(review.get("reviewer"))
        and bool(record.get("artifacts"))
    )


def schema_errors(record: dict) -> list[str]:
    """The subset of `om9.m00.evidence.v013` the protocol actually enforces."""
    errors: list[str] = []
    required = ["task_id", "source_sha", "status", "actual_execution", "commands", "artifacts", "expected", "actual", "review"]
    for key in required:
        if key not in record:
            errors.append(f"missing {key}")
    task_id = str(record.get("task_id") or "")
    if not L4_ID.match(task_id):
        errors.append(f"task_id does not match the L4 pattern: {task_id!r}")
    if record.get("source_sha") != QYMCAD_SHA:
        errors.append("source_sha is not the pinned QymCAD revision")
    if str(record.get("status")) not in STATUSES:
        errors.append(f"status is not one of the protocol values: {record.get('status')!r}")
    if not isinstance(record.get("actual_execution"), bool):
        errors.append("actual_execution must be a boolean")
    commands = record.get("commands")
    if not isinstance(commands, list):
        errors.append("commands must be an array")
    else:
        for index, command in enumerate(commands):
            for key in ("command", "exit_code", "stdout_path", "stderr_path"):
                if key not in command:
                    errors.append(f"commands[{index}] missing {key}")
            if "exit_code" in command and not isinstance(command["exit_code"], int):
                errors.append(f"commands[{index}].exit_code must be an integer")
    artifacts = record.get("artifacts")
    if not isinstance(artifacts, list):
        errors.append("artifacts must be an array")
    else:
        for index, artifact in enumerate(artifacts):
            path = str(artifact.get("path") or "")
            digest = str(artifact.get("sha256") or "")
            if not path:
                errors.append(f"artifacts[{index}] missing path")
            if not re.fullmatch(r"[a-f0-9]{64}", digest):
                errors.append(f"artifacts[{index}].sha256 is not a lowercase sha256")
    review = record.get("review")
    if not isinstance(review, dict):
        errors.append("review must be an object")
    else:
        for key in ("decision", "reviewer", "evidence_checked"):
            if key not in review:
                errors.append(f"review missing {key}")
        if review.get("decision") not in {"PENDING", "APPROVED", "REJECTED"}:
            errors.append("review.decision is not PENDING/APPROVED/REJECTED")
    expected = record.get("expected")
    actual = record.get("actual")
    if not isinstance(expected, list):
        errors.append("expected must be an array")
    if not isinstance(actual, list):
        errors.append("actual must be an array")
    if isinstance(expected, list) and isinstance(actual, list):
        if expected and actual and expected == actual:
            errors.append("actual equals expected: an oracle cannot be copied from the result")
    return errors
