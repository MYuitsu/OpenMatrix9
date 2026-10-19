"""Verify the applied L4 reference supplements across M00-M02.

Checks per file: valid JSON, single reference_supplements/reference_source_policy
member, runbook heading present exactly once, no planning/runtime status was
promoted, and the appended content matches what the crosswalk produces today.
"""
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
import matrix_ref_crosswalk as cw  # noqa: E402
import apply_matrix_ref_supplements as apply_mod  # noqa: E402

PROMOTED = ("VERIFIED", "PASS")


def main():
    inventory = json.loads((ROOT / "build" / "matrix-ref-inventory.json").read_text(encoding="utf-8"))
    errors = []
    per_milestone = {}
    promoted = []
    no_anchor = []
    for row in inventory["l4"]:
        path = ROOT / row["file"]
        text = path.read_text(encoding="utf-8")
        try:
            data = json.loads(text)
        except Exception as exc:  # noqa: BLE001
            errors.append(f"{row['file']}: invalid JSON: {exc!r}")
            continue
        milestone = row["milestone"]
        stats = per_milestone.setdefault(milestone, {"files": 0, "with_anchors": 0})
        stats["files"] += 1
        if "reference_supplements" not in data:
            errors.append(f"{row['file']}: missing reference_supplements")
            continue
        has_content = any(data["reference_supplements"].get(key) for key in
                          ("rhino5_core_chapters", "matrix9_reference_sources",
                           "manual_reference_sheets"))
        if not has_content:
            no_anchor.append(row["file"])
        if text.count('"reference_supplements"') != 1:
            errors.append(f"{row['file']}: duplicate reference_supplements member")
        if text.count('"reference_source_policy"') != 1:
            errors.append(f"{row['file']}: duplicate reference_source_policy member")
        # M00-M02 records embed an agent runbook; the M04 package uses its own
        # schema (work_plan/handoff) and carries the anchors in JSON only.
        has_runbook = "agent_runbook_markdown" in data
        if has_runbook and text.count(apply_mod.RUNBOOK_HEADING) != 1:
            errors.append(f"{row['file']}: runbook heading count != 1")
        supplements = data["reference_supplements"]
        if supplements["status"] != cw.REQUIREMENTS_ONLY:
            errors.append(f"{row['file']}: unexpected supplement status")
        expected = apply_mod.build_supplements(row["parent_l3_id"])
        if expected is None:
            no_anchor.append(row["file"])
        elif supplements != expected:
            errors.append(f"{row['file']}: supplement drift vs crosswalk")
        if has_runbook:
            runbook = data["agent_runbook_markdown"]
            # Only the reference section belongs to this validator; other sections
            # (e.g. the M01/M02 code-sample block) legitimately follow it.
            head = runbook.split(apply_mod.RUNBOOK_HEADING)[0].rstrip() + "\n"
            expected_book = json.loads(apply_mod.inject_markdown(
                json.dumps({"agent_runbook_markdown": head}, ensure_ascii=False),
                apply_mod.runbook_block(expected)))["agent_runbook_markdown"]
            if not runbook.startswith(expected_book):
                errors.append(f"{row['file']}: runbook reference section drift vs crosswalk")
        stats["with_anchors"] += 1
        # A planning state that claims execution/verification is a promotion.
        # The *_NOT_EXECUTED / *_NOT_IMPLEMENTED / *_NOT_CREATED markers are the
        # package's own "planned only" wording and are fine.
        planning = str(data.get("planning_state") or data.get("validation", {}).get("status") or "")
        neutral = ("NOT_EXECUTED", "NOT_IMPLEMENTED", "NOT_CREATED", "NOT_RUN", "NOT_STARTED")
        if any(word in planning.upper() for word in ("EXECUTED", "VERIFIED_PASS", "PASSED")) and \
                not any(marker in planning.upper() for marker in neutral):
            promoted.append(f"{row['file']}: planning_state={planning}")
        policy = data.get("execution_policy", {})
        status = str(policy.get("status", ""))
        if any(word in status.upper() for word in PROMOTED):
            promoted.append(f"{row['file']}: execution_policy.status={status}")
        # The merge must never flip a package record to a runtime/verified state.
        for field in ("validation", "interaction_policy"):
            value = data.get(field)
            if isinstance(value, dict):
                inner = str(value.get("status", ""))
                if any(word in inner.upper() for word in ("VERIFIED_PASS", "IMPLEMENTED")):
                    promoted.append(f"{row['file']}: {field}.status={inner}")
    summary = {
        "per_milestone": per_milestone,
        "files_without_anchor_content": len(no_anchor),
        "promoted_status": len(promoted),
        "errors": errors,
    }
    print(json.dumps(summary, ensure_ascii=False, indent=1))
    for entry in promoted[:10]:
        print("PROMOTED:", entry)
    for entry in no_anchor[:10]:
        print("NO-ANCHOR:", entry)
    raise SystemExit(bool(errors) or bool(promoted))


if __name__ == "__main__":
    main()
