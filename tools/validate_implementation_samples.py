"""Verify the implementation-sample blocks across every milestone that carries them.

Fails closed on: a missing block, a promoted status, an incorrect honesty flag, a
sketch that no longer matches the template for its role, a duplicated member, a
missing runbook warning section, or a milestone whose records have no sample at all.

The milestone list is taken from the applier so this cannot silently drift out of
scope when another milestone is added there.
"""
import json
import sys
from pathlib import Path

ROOT = Path("H:/OpenMatrix9")
sys.path.insert(0, str(ROOT / "tools"))
import apply_implementation_samples as samples  # noqa: E402

PROMOTED = ("VERIFIED", "IMPLEMENTED", "PASSED")


def main():
    errors = []
    per_milestone = {}
    roles = {}
    for milestone, dirname in samples.MILESTONES.items():
        base = ROOT / "OM9_Milestones" / dirname
        if not base.is_dir():
            errors.append(f"{milestone}: milestone directory not found: {dirname}")
            continue
        stats = per_milestone.setdefault(milestone, {"files": 0, "with_samples": 0,
                                                     "roles": {}})
        for path in sorted(base.rglob("l4.json")):
            text = path.read_text(encoding="utf-8")
            data = json.loads(text)
            stats["files"] += 1
            block = data.get(samples.SAMPLES_KEY)
            if block is None:
                errors.append(f"{path}: missing {samples.SAMPLES_KEY}")
                continue
            stats["with_samples"] += 1
            role = block.get("role")
            roles[role] = roles.get(role, 0) + 1
            stats["roles"][role] = stats["roles"].get(role, 0) + 1
            rel = path.relative_to(base).as_posix()
            if text.count(f'"{samples.SAMPLES_KEY}"') != 1:
                errors.append(f"{rel}: duplicated samples member")
            if block.get("status") != "author_skeleton_untrusted":
                errors.append(f"{rel}: unexpected sample status {block.get('status')!r}")
            for flag in ("sketch_is_untested", "not_runtime_evidence",
                         "must_not_be_reported_as_implemented"):
                if block.get(flag) is not True:
                    errors.append(f"{rel}: honesty flag {flag} is not true")
            if block.get("language") != "rust":
                errors.append(f"{rel}: unexpected language {block.get('language')!r}")
            if not block.get("sketch"):
                errors.append(f"{rel}: empty sketch")
            if any(word in str(block.get("status", "")).upper() for word in PROMOTED):
                errors.append(f"{rel}: sample status claims completion")
            # drift: the block must equal what the template produces today, using the
            # same task lookup the applier uses (nested where present, else the record
            # root) and the same resolved role.
            expected = samples.build_block(data, data.get("task") or data, milestone)
            if expected is None:
                errors.append(f"{rel}: role has no template")
            elif block != expected:
                diff = sorted(k for k in set(block) | set(expected)
                              if block.get(k) != expected.get(k))
                errors.append(f"{rel}: sample drift vs template: {diff}")
            runbook = data.get("agent_runbook_markdown")
            guide = data.get(samples.GUIDE_KEY)
            if isinstance(runbook, str):
                if runbook.count(samples.RUNBOOK_HEADING) != 1:
                    errors.append(f"{rel}: code-sample runbook section count != 1")
                if guide is not None:
                    errors.append(f"{rel}: both a runbook section and a {samples.GUIDE_KEY} member")
            else:
                # overlay records (M04-M07) have no runbook field, so the warning must
                # live in its own member; count the heading there instead
                if not isinstance(guide, str) or not guide:
                    errors.append(f"{rel}: missing {samples.GUIDE_KEY} warning content")
                elif guide.count(samples.RUNBOOK_HEADING) != 1:
                    errors.append(f"{rel}: code-sample guide heading count != 1")
                elif "author_skeleton_untrusted" not in guide:
                    errors.append(f"{rel}: guide does not state the untrusted status")
    print(json.dumps({"per_milestone": per_milestone, "roles": roles,
                      "errors": errors}, ensure_ascii=False, indent=1))
    raise SystemExit(bool(errors))


if __name__ == "__main__":
    main()
