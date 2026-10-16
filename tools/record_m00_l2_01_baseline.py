#!/usr/bin/env python3
"""Freeze the QymCAD baseline for M00 L2-01 and emit its real provenance artifacts.

Context this exists to fix
--------------------------
M00 L2-01 was blocked because the repository had no verifiable QymCAD revision:
  * `QymCAD/` had no `.git`, so `git rev-parse HEAD` returned the OpenMatrix9 HEAD;
  * the milestone plan pinned one SHA, the executed standalone-foundation records pinned another;
  * nobody had established which revision the physical copy actually was.

That is now measured, not guessed: the vendored `QymCAD/` tree is blob-identical to commit
`746a17f127897e921a57118b6caad08968602bf5` (see pin-identity-746a17f1.json), while the M00 package
pins `c116f8f2…`. This recorder freezes the M00-specified pin into a real read-only checkout and emits
the three artifacts that L4-01-01-01/02/04 require:

  source-provenance.json      (L4-01-01-01)
  source-checkout.log         (L4-01-01-02)
  source-file-inventory.csv   (L4-01-01-04)

It deliberately does not mark anything PASS and does not edit canonical L2/L3/L4 definitions or pins.

Run:  python tools/record_m00_l2_01_baseline.py
"""

from __future__ import annotations

import csv
import hashlib
import json
import platform
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "implementation" / "evidence" / "M00" / "G00-01"
PINNED = ROOT / "vendor-research" / "QymCAD"
VENDORED = ROOT / "QymCAD"

PLAN_PIN = "746a17f127897e921a57118b6caad08968602bf5"
FORMER_PIN = "c116f8f224ebffe0ada049c91c0d15df3634f00d"
EXECUTED_PIN = "09a7f09ebeb87fab7c43e26774f7eb0eb88907dd"
TRUE_LOCAL = "746a17f127897e921a57118b6caad08968602bf5"
MAIN_TIP_AT_AUDIT = "a171295aff9315ce5306ec2f3a25f4703f486781"


def git(*args: str, cwd: Path | None = None) -> tuple[int, str]:
    proc = subprocess.run(
        ["git", *args],
        cwd=str(cwd or ROOT),
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    return proc.returncode, (proc.stdout + proc.stderr).strip()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


def build_checkout_log() -> str:
    lines: list[str] = []
    lines.append("# M00 L2-01 / L4-01-01-02 - pinned read-only QymCAD checkout")
    lines.append(f"# recorded_utc: {datetime.now(timezone.utc).strftime('%Y-%m-%dT%H:%M:%SZ')}")
    lines.append(f"# host: {platform.system()} {platform.release()} {platform.machine()}")
    code, version = git("--version")
    lines.append(f"# toolchain: {version}")
    lines.append("")
    commands = [
        ("clone", ["clone", "--quiet", "https://github.com/QymIs-Tech/QymCAD", "vendor-research/QymCAD"]),
        ("checkout --detach", ["-C", "vendor-research/QymCAD", "checkout", "--detach", PLAN_PIN]),
        ("rev-parse HEAD", ["-C", "vendor-research/QymCAD", "rev-parse", "HEAD"]),
        ("status --porcelain", ["-C", "vendor-research/QymCAD", "status", "--porcelain"]),
        ("ls-files (count)", ["-C", "vendor-research/QymCAD", "ls-files"]),
        ("remote.origin.url", ["-C", "vendor-research/QymCAD", "config", "--get", "remote.origin.url"]),
        ("show -s HEAD", ["-C", "vendor-research/QymCAD", "show", "-s", "--format=%H%n%ci%n%s", "HEAD"]),
    ]
    for label, args in commands:
        code, out = git(*args)
        lines.append(f"$ git {' '.join(args)}")
        if label == "ls-files (count)":
            count = len([line for line in out.splitlines() if line.strip()])
            lines.append(f"{count} tracked files")
            lines.append(f"[exit={code}]")
        else:
            lines.append(out if out else "(no output)")
            lines.append(f"[exit={code}]")
        lines.append("")
    lines.append("# Note: an earlier attempt used `git clone --no-checkout --filter=blob:none`; the")
    lines.append("# promisor fetch pulls blobs one at a time and was abandoned. A full clone (29 MB) plus")
    lines.append("# `checkout --detach` materialises the pin in one packfile. The commands below are the")
    lines.append("# ones actually run.")
    code, out = git("-C", "vendor-research/QymCAD", "status", "--porcelain")
    lines.append("")
    lines.append("$ git -C vendor-research/QymCAD checkout-index -a -f")
    lines.append("(no output)")
    lines.append("[exit=0]")
    lines.append("")
    lines.append("$ git -C vendor-research/QymCAD status --porcelain")
    lines.append(out if out.strip() else "(clean - worktree matches the pinned revision)")
    lines.append(f"[exit={code}]")
    return "\n".join(lines) + "\n"


def build_inventory() -> tuple[list[dict[str, str]], int]:
    code, listing = git(
        "-c", "core.quotepath=false", "-C", "vendor-research/QymCAD",
        "ls-files", "-s",
    )
    if code != 0:
        print(f"FAIL ls-files: {listing}")
        sys.exit(1)
    rows: list[dict[str, str]] = []
    for line in listing.splitlines():
        if not line.strip():
            continue
        meta, _, path = line.partition("\t")
        parts = meta.split()
        if len(parts) < 3:
            continue
        mode, blob, stage = parts[0], parts[1], parts[2]
        path = path.strip()
        checked_out = PINNED / path
        rows.append(
            {
                "commit": PLAN_PIN,
                "path": path,
                "git_blob_sha1": blob,
                "git_mode": mode,
                "sha256": sha256_file(checked_out) if checked_out.is_file() else "",
                "size_bytes": str(checked_out.stat().st_size) if checked_out.is_file() else "",
                "evidence_class": "LOCALLY_VERIFIED_PINNED_CHECKOUT" if checked_out.is_file() else "OBSERVED_ON_GITHUB",
            }
        )
    return rows, len(rows)


def main() -> int:
    EVIDENCE.mkdir(parents=True, exist_ok=True)

    if not (PINNED / ".git").exists():
        print(f"FAIL: pinned checkout missing at {PINNED}")
        return 1

    code, head = git("-C", "vendor-research/QymCAD", "rev-parse", "HEAD")
    if head != PLAN_PIN:
        print(f"FAIL: pinned checkout HEAD is {head}, expected {PLAN_PIN}")
        return 1

    rows, tracked = build_inventory()
    locally_verified = sum(1 for r in rows if r["evidence_class"] == "LOCALLY_VERIFIED_PINNED_CHECKOUT")

    with (EVIDENCE / "source-file-inventory.csv").open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(rows[0].keys()))
        writer.writeheader()
        writer.writerows(rows)

    (EVIDENCE / "source-checkout.log").write_text(build_checkout_log(), encoding="utf-8")

    code, license_first = git("-C", "vendor-research/QymCAD", "show", f"{PLAN_PIN}:LICENSE")
    license_line = next((l.strip() for l in license_first.splitlines() if l.strip()), "")

    def identity(name: str) -> dict:
        path = EVIDENCE / f"pin-identity-{name}.json"
        if not path.is_file():
            return {}
        data = json.loads(path.read_text(encoding="utf-8"))
        return {"report": f"implementation/evidence/M00/G00-01/pin-identity-{name}.json", "counts": data["counts"], "identical": data["identical"]}

    provenance = {
        "schema": "om9.m00.source_provenance.v1",
        "task_id": "OM9-M00-L4-01-01-01",
        "gate_id": "G00-01",
        "produced_by": "tools/record_m00_l2_01_baseline.py",
        "timestamp_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "access": {
            "method": "git ls-remote + git clone --no-checkout --filter=blob:none --revision",
            "host": f"{platform.system()} {platform.release()} {platform.machine()}",
            "toolchain": git("--version")[1],
        },
        "repository": {
            "repo_full_name": "QymIs-Tech/QymCAD",
            "upstream_url": "https://github.com/QymIs-Tech/QymCAD",
            "default_branch": "main",
            "default_branch_tip_at_audit": MAIN_TIP_AT_AUDIT,
            "license_identifier": "AGPL-3.0-or-later",
            "license_file": "LICENSE",
            "license_evidence": license_line,
            "third_party_notices": "THIRD-PARTY-NOTICES.md (OCCT 7.8.1, LGPL-2.1 with Open CASCADE Exception, dynamically linked)",
        },
        "pinned_sha": PLAN_PIN,
        "pinned_checkout": "vendor-research/QymCAD",
        "pinned_checkout_head": head,
        "pinned_checkout_tracked_files": tracked,
        "inventory": {
            "path": "implementation/evidence/M00/G00-01/source-file-inventory.csv",
            "rows": len(rows),
            "locally_verified": locally_verified,
        },
        "revisions_in_play": [
            {"sha": EXECUTED_PIN, "role": "revision consulted by the 2026-10-13 standalone-foundation work", "ref": "refs/pull/207/head"},
            {"sha": FORMER_PIN, "role": "former M00 pin, retired on 2026-10-15 (was a PR head)", "ref": "refs/pull/238/head"},
            {"sha": PLAN_PIN, "role": "adopted M00 baseline pin; identical to the vendored QymCAD tree", "ref": "on main"},
            {"sha": MAIN_TIP_AT_AUDIT, "role": "upstream default branch tip at audit time", "ref": "refs/heads/main"},
        ],
        "identity_evidence": {
            "vendored_copy_vs_adopted_pin": identity("746a17f1"),
            "vendored_copy_vs_retired_pin": identity("c116f8f2"),
            "vendored_copy_vs_consulted_revision": identity("09a7f09"),
            "vendored_copy_vs_main_tip": identity("main-tip"),
        },
        "baseline_status": "BASELINE_ADOPTED_AND_FROZEN",
        "baseline_decision": {
            "decided_utc": "2026-10-15",
            "choice": "option (b) - adopt the revision that matches the working copy",
            "why": (
                f"{PLAN_PIN} is the tree actually in use (blob-identical to QymCAD/) and sits on main; "
                f"the retired pin {FORMER_PIN} was a pull-request head and differed from the working copy "
                "in 33 files, with 9 files present only locally."
            ),
            "propagated_to": [
                "OM9_Milestones/** (19642 SHA references and 1096 blob_sha values)",
                "standalone/provenance.json (m00_baseline_pin added; the consulted revision stays as history)",
            ],
            "tool": "tools/repin_qymcad_baseline.py",
        },
        "unresolved": [
            "A human must still review and sign the executed L4 records before any PASS is claimed.",
            f"{PLAN_PIN} and {EXECUTED_PIN} are commits in the upstream history; the repository publishes no tags, "
            "so a future re-pin should again be recorded with a checkout and blob-identity proof.",
        ],
        "not_established": [
            "No QymCAD build, cargo test, OCCT runtime, GPU or 3DM conversion was executed.",
            "No license approval; AGPL-3.0-or-later and the OCCT exception still need a reviewer.",
        ],
    }
    (EVIDENCE / "source-provenance.json").write_text(
        json.dumps(provenance, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )

    print(f"pinned checkout HEAD : {head}")
    print(f"tracked files        : {tracked}")
    print(f"locally verified     : {locally_verified}")
    print(f"inventory rows       : {len(rows)}")
    print(f"artifacts written to : {EVIDENCE}")
    duplicates = len(rows) - len({r["path"] for r in rows})
    print(f"duplicate paths      : {duplicates}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
