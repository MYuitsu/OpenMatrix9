#!/usr/bin/env python3
"""Sign the OM9-L2-03-14 manual reviews with the owner's name.

The manual gate M01-UI is not something an agent may satisfy: the eight reviews
record what a HUMAN looked at. This tool only writes the signature the owner asks
for, after they have run the checklist:

    python tools/sign_m01_l2_03_14.py --reviewer "Tên" --dry-run   # show what changes
    python tools/sign_m01_l2_03_14.py --reviewer "Tên"             # sign all eight

It refuses to sign an item when the review has no manual items or no machine
evidence, so it cannot paper over a missing record.
"""
from __future__ import annotations

import argparse
import json
import sys
from datetime import date
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
EVIDENCE = ROOT / "implementation/evidence/M01"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--reviewer", required=True, help="who signed (recorded verbatim)")
    parser.add_argument("--date", default=date.today().isoformat())
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()
    if not args.reviewer.strip():
        print("a reviewer name is required", file=sys.stderr)
        return 2
    reviews = sorted(EVIDENCE.glob("OM9-L3-03-14-*/OM9-L4-*-05.ui-review.json"))
    if not reviews:
        print("no reviews found", file=sys.stderr)
        return 1
    signed, refused = 0, 0
    for path in reviews:
        review = json.loads(path.read_text(encoding="utf-8"))
        items = review.get("manual_reviewer_checklist", {}).get("items", [])
        checks = review.get("checks_run", [])
        if not items or not checks:
            print(f"refused {path.name}: no manual items or no machine evidence")
            refused += 1
            continue
        review["reviewer"] = {
            "name": args.reviewer,
            "date": args.date,
            "status": "SIGNED",
            "manual_gate": "M01-UI",
        }
        print(f"{'would sign' if args.dry_run else 'signed'} {path.parent.name}/{path.name}")
        if not args.dry_run:
            path.write_text(json.dumps(review, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        signed += 1
    print(f"{signed} review(s) {'to sign' if args.dry_run else 'signed'}, {refused} refused")
    return 1 if refused else 0


if __name__ == "__main__":
    sys.exit(main())
