#!/usr/bin/env python3
"""Compare the local vendored QymCAD tree against a pinned upstream revision, by git blob identity.

Why: M00 L2-01 was blocked with "which revision is the local QymCAD copy?". The copy has no VCS
metadata, and the repository carried two different SHAs (a plan pin and an executed baseline). This
tool settles the question with byte-level identity instead of guessing.

Method (no upstream blobs are downloaded):
  1. `git ls-tree -r <sha> --format="%(objectname) %(path)"` in the pinned filtered clone gives the
     authoritative blob SHA-1 for every path at that revision (tree objects only).
  2. `git hash-object --stdin-paths` on the local working tree gives the same kind of SHA-1 for every
     local file.
  3. Paths are joined and compared.

`git hash-object` applies the checkout's attribute filters, so a match means "this content would be
committed as exactly this blob" - the same notion `git status`/`git ls-files` uses.

Usage:
  python tools/verify_qymcad_pin.py --pin-tree <listing.txt> --local-blobs <listing.txt> [--out <json>]
"""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def read_pin_tree(path: Path) -> dict[str, str]:
    table: dict[str, str] = {}
    for line in path.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line:
            continue
        sha, _, name = line.partition(" ")
        table[name] = sha
    return table


def read_local_blobs(path: Path) -> dict[str, str]:
    table: dict[str, str] = {}
    for line in path.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line:
            continue
        sha, _, name = line.partition(" ")
        table[name] = sha
    return table


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--pin-tree", required=True, type=Path)
    parser.add_argument("--local-blobs", required=True, type=Path)
    parser.add_argument("--pin-sha", default="")
    parser.add_argument("--out", default="", type=Path)
    args = parser.parse_args()

    pin = read_pin_tree(args.pin_tree)
    local = read_local_blobs(args.local_blobs)

    common = sorted(set(pin) & set(local))
    matched = [p for p in common if pin[p] == local[p]]
    drifted = [p for p in common if pin[p] != local[p]]
    local_only = sorted(set(local) - set(pin))
    pin_only = sorted(set(pin) - set(local))

    report = {
        "schema": "om9.m00.l2_01.pin_identity.v1",
        "pin_sha": args.pin_sha,
        "timestamp_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "method": "git ls-tree blob sha (pinned revision) vs git hash-object (local working tree)",
        "counts": {
            "pin_files": len(pin),
            "local_files": len(local),
            "common": len(common),
            "matched": len(matched),
            "content_drift": len(drifted),
            "local_only": len(local_only),
            "pin_only": len(pin_only),
        },
        "identical": not drifted and not local_only and not pin_only,
        "content_drift": [{"path": p, "pin": pin[p], "local": local[p]} for p in drifted],
        "local_only": local_only,
        "pin_only": pin_only,
    }

    print(f"pin files        : {len(pin)}")
    print(f"local files      : {len(local)}")
    print(f"blob-identical   : {len(matched)}")
    print(f"content drift    : {len(drifted)}")
    print(f"local-only files : {len(local_only)}")
    print(f"pin-only files   : {len(pin_only)}")
    if drifted:
        print("  drift sample:")
        for row in report["content_drift"][:10]:
            print(f"    {row['path']}")
    if local_only:
        print("  local-only sample: " + ", ".join(local_only[:10]))
    if pin_only:
        print("  pin-only sample: " + ", ".join(pin_only[:10]))
    verdict = "IDENTICAL" if report["identical"] else "NOT IDENTICAL"
    print(f"verdict          : {verdict}")

    if args.out:
        args.out.parent.mkdir(parents=True, exist_ok=True)
        args.out.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        print(f"written          : {args.out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
