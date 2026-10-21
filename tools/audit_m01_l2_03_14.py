#!/usr/bin/env python3
"""Integrity audit for OM9-L2-03-14: every declared artefact exists.

The eight L3 records each declare design files, fixtures, an evidence review and
the artefacts that review cites. This tool checks that all of it is on disk (so a
record can never cite a file that was renamed or pruned) and that every L3 has a
ledger entry. It is a companion to tools/verify_m01_l2_03_14.py: that one proves
behaviour, this one proves the paperwork matches the tree.
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
L3S = [f"OM9-L3-03-14-{n:03d}" for n in range(1, 9)]
PREFIXES = [Path(""), Path("crates/om9-app"), Path("implementation/evidence/M01")]


def resolve(relative: str, extra: tuple[Path, ...] = ()) -> Path | None:
    """Find a cited file: repo root, the app crate, the module evidence, then the
    L3's own evidence/design folders (records cite neighbours by bare name)."""
    for base in (*extra, *(ROOT / prefix for prefix in PREFIXES)):
        candidate = base / relative
        if candidate.exists():
            return candidate
    return None


def main() -> int:
    ledger = json.loads((ROOT / "docs/openmatrix9-progress.json").read_text(encoding="utf-8"))
    ids = {item["id"] for item in ledger["items"] if item.get("id")}
    problems: list[str] = []
    for l3 in L3S:
        design = ROOT / "implementation/design/M01" / l3
        evidence = ROOT / "implementation/evidence/M01" / l3
        fixtures = ROOT / "implementation/fixtures/M01" / l3
        for path in (design / "api.md", design / "source_crosswalk.json"):
            if not path.exists():
                problems.append(f"{l3}: missing {path.relative_to(ROOT)}")
        for name in ("positive.json", "negative.json"):
            if not (fixtures / name).exists():
                problems.append(f"{l3}: missing fixture {name}")
        reviews = sorted(evidence.glob("*.ui-review.json"))
        if not reviews:
            problems.append(f"{l3}: no ui-review")
        for review_path in reviews:
            review = json.loads(review_path.read_text(encoding="utf-8"))
            where = f"{l3}/{review_path.name}"
            if not review.get("manual_reviewer_checklist", {}).get("items"):
                problems.append(f"{where}: no manual items")
            if not review.get("checks_run"):
                problems.append(f"{where}: empty checks_run")
            if not review.get("reviewer", {}).get("status"):
                problems.append(f"{where}: no reviewer.status")
            for artefact in review.get("artefacts", []):
                if resolve(artefact, (evidence, design)) is None:
                    problems.append(f"{where}: artefact missing: {artefact}")
        if not any(i.startswith(f"m01-l3-03-14-{l3[-3:]}-") for i in ids):
            problems.append(f"{l3}: no ledger entry")
    for problem in problems:
        print("-", problem)
    print(f"integrity: {len(problems)} problem(s) across {len(L3S)} L3")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
