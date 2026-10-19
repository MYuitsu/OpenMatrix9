#!/usr/bin/env python3
"""Re-pin the QymCAD baseline across the OpenMatrix9 planning packages.

Decision (2026-10-15): the M00 package pinned `746a17f2…`, a *pull-request head*, while the QymCAD
tree actually used on disk is blob-identical to `746a17f1…`, a commit on `main`. The two differ by 42
files. Keeping both would preserve the "floating baseline" defect that blocked M00 L2-01, so the pin
moves to the revision that matches reality.

`746a17f2…` is an ancestor of `746a17f1…`, so every path referenced at the old pin still exists at the
new one and the links stay valid when the SHA segment is swapped.

Deliberately text-only
----------------------
An earlier draft re-serialised every touched JSON through `json.dumps`. That rewrites the whole file -
key order, spacing, escaping - for what is a 40-character change, and would bury the real edit in a
3000-file diff. This version performs literal substitutions instead:

  1. the full SHA, and its 7-character short form;
  2. `"blob_sha"` values, but only for the handful of paths whose content actually differs between the
     two revisions. Blob hashes are read from the pinned checkout, never guessed.

Narrative records under `docs/validation/` are left alone: they describe which pin was used *when*, and
that history stays true.

Usage
-----
  python tools/repin_qymcad_baseline.py                 # dry run
  python tools/repin_qymcad_baseline.py --apply
  python tools/repin_qymcad_baseline.py --verify        # check recorded blob_sha against the pin
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OLD_SHA = "746a17f127897e921a57118b6caad08968602bf5"
NEW_SHA = "746a17f127897e921a57118b6caad08968602bf5"

INCLUDE_DIRS = [ROOT / "OM9_Milestones", ROOT / "tools"]
EXCLUDE_PARTS = {".git", "QymCAD", "vendor-research", "ref", "build", "__pycache__"}
EXCLUDE_FILES = {
    ROOT / "tools" / "record_m00_l2_01_baseline.py",
    ROOT / "tools" / "record_m00_l2_01_status.py",
    ROOT / "tools" / "validate_m00_l2_01_evidence.py",
}
SUFFIXES = {".json", ".md", ".csv", ".py"}


def ls_tree(checkout: Path, sha: str) -> dict[str, str]:
    proc = subprocess.run(
        ["git", "-C", str(checkout), "-c", "core.quotepath=false", "ls-tree", "-r", sha,
         "--format=%(objectname) %(path)"],
        capture_output=True, text=True, encoding="utf-8", errors="replace",
    )
    if proc.returncode != 0:
        print(f"FAIL ls-tree {sha}: {proc.stderr.strip()}")
        sys.exit(2)
    table: dict[str, str] = {}
    for line in proc.stdout.splitlines():
        blob, _, path = line.partition(" ")
        if path:
            table[path] = blob
    return table


def changed_blob_map(checkout: Path, old: str, new: str) -> dict[str, str]:
    before, after = ls_tree(checkout, old), ls_tree(checkout, new)
    return {
        before[path]: after[path]
        for path in before
        if path in after and before[path] != after[path]
    }


def targets() -> list[Path]:
    found: list[Path] = []
    for base in INCLUDE_DIRS:
        for path in base.rglob("*"):
            if not path.is_file() or path.suffix not in SUFFIXES:
                continue
            if any(part in EXCLUDE_PARTS for part in path.parts):
                continue
            if path in EXCLUDE_FILES:
                continue
            found.append(path)
    return sorted(found)


def scan_blob_sha(checkout: Path, new: str, apply: bool) -> int:
    """Compare every recorded (path, blob_sha) pair against the pinned revision.

    With `apply`, a mismatching blob_sha is rewritten in place. The rewrite is scoped by the regex
    match, so only the blob literal inside that one evidence object changes - a bogus hash that happens
    to equal another file's real hash cannot corrupt that other file's record.
    """
    truth = ls_tree(checkout, new)
    pattern = re.compile(r'("path":\s*"([^"]+)"[^{}]*?"blob_sha":\s*")([a-f0-9]{40})(")', re.S)
    counters = {"checked": 0, "stale": 0, "non_qymcad": 0, "fixed": 0}
    touched: list[Path] = []

    for path in sorted((ROOT / "OM9_Milestones").rglob("*.json")):
        text = path.read_text(encoding="utf-8")
        if "blob_sha" not in text:
            continue

        def replace(match: re.Match[str]) -> str:
            named, blob = match.group(2), match.group(3)
            counters["checked"] += 1
            if named not in truth:
                counters["non_qymcad"] += 1
                return match.group(0)
            if truth[named] == blob:
                return match.group(0)
            counters["stale"] += 1
            if apply:
                counters["fixed"] += 1
                return f"{match.group(1)}{truth[named]}{match.group(4)}"
            return match.group(0)

        new_text = pattern.sub(replace, text)
        if apply and new_text != text:
            path.write_text(new_text, encoding="utf-8")
            touched.append(path)

    print(f"blob_sha checked           : {counters['checked']}")
    print(f"  {'fixed' if apply else 'mismatching the pin'}: {counters['fixed'] if apply else counters['stale']}")
    print(f"  non-QymCAD path (skipped): {counters['non_qymcad']}  (openNURBS/Matrix9 sources share this package)")
    if apply:
        print(f"  files rewritten          : {len(touched)}")
    return 0 if (apply or not counters["stale"]) else 1


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--apply", action="store_true")
    parser.add_argument("--verify", action="store_true")
    parser.add_argument("--fix-blob-sha", action="store_true",
                        help="rewrite recorded blob_sha values that do not match the pinned revision")
    parser.add_argument("--checkout", default="vendor-research/QymCAD")
    parser.add_argument("--old", default=OLD_SHA)
    parser.add_argument("--new", default=NEW_SHA)
    args = parser.parse_args()

    checkout = ROOT / args.checkout
    if not (checkout / ".git").exists():
        print(f"FAIL: pinned checkout not found at {checkout}")
        return 1
    head = subprocess.run(["git", "-C", str(checkout), "rev-parse", "HEAD"],
                          capture_output=True, text=True).stdout.strip()
    if head != args.new:
        print(f"FAIL: checkout HEAD is {head}, expected {args.new}")
        print(f"      git -C {args.checkout} checkout --detach {args.new}")
        return 1

    if args.verify:
        return scan_blob_sha(checkout, args.new, apply=False)
    if args.fix_blob_sha:
        return scan_blob_sha(checkout, args.new, apply=True)

    blobs = changed_blob_map(checkout, args.old, args.new)
    print(f"paths whose blob differs between pins: {len(blobs)}")

    short_old, short_new = args.old[:7], args.new[:7]
    counters = {"files": 0, "sha_full": 0, "sha_short": 0, "blob_sha": 0}

    for path in targets():
        try:
            text = path.read_text(encoding="utf-8")
        except (UnicodeDecodeError, OSError):
            continue
        original = text

        counters["sha_full"] += text.count(args.old)
        text = text.replace(args.old, args.new)

        counters["sha_short"] += text.count(short_old)
        text = text.replace(short_old, short_new)

        for old_blob, new_blob in blobs.items():
            needle = f'"blob_sha": "{old_blob}"'
            if needle in text:
                counters["blob_sha"] += text.count(needle)
                text = text.replace(needle, f'"blob_sha": "{new_blob}"')

        if text != original:
            counters["files"] += 1
            if args.apply:
                path.write_text(text, encoding="utf-8")

    mode = "APPLY" if args.apply else "DRY RUN"
    print(f"[{mode}] {args.old} -> {args.new}")
    print(f"  files changed        : {counters['files']}")
    print(f"  full SHA occurrences : {counters['sha_full']}")
    print(f"  short SHA occurrences: {counters['sha_short']}")
    print(f"  blob_sha occurrences : {counters['blob_sha']}")
    if not args.apply:
        print("  (nothing written; re-run with --apply)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
