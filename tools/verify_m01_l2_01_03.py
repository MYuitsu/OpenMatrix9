#!/usr/bin/env python
"""OM9-L2-01-03 (Be Document Scene Entities) — one automated verification pass.

Runs the test target that carries each of the thirteen L3 items, then the crate
suites, the workspace test, the compile check, the formatting and the lints, and
prints one table. Exits non-zero if anything failed or an evidence log is
missing.

    python tools/verify_m01_l2_01_03.py                # full pass
    python tools/verify_m01_l2_01_03.py --quick        # skip the workspace test
    python tools/verify_m01_l2_01_03.py --self-test    # check the table wiring only

Set CARGO_TARGET_DIR to build outside the checkout when the repository target
directory is not writable (the pass inherits the environment unchanged).
"""
from __future__ import annotations

import argparse
import pathlib
import subprocess
import sys
import time

ROOT = pathlib.Path(__file__).resolve().parents[1]
STANDALONE = ROOT
EVIDENCE = ROOT / "implementation/evidence/M01"

# One entry per L3: the test targets that carry it (the module's oracle per item).
ITEMS = [
    ("OM9-L3-01-03-001", "Document Entity IDs", [
        ["-p", "om9-core", "--test", "document_graph"],
        ["-p", "om9-archive", "--test", "document_identity_round_trip"],
    ]),
    ("OM9-L3-01-03-002", "Typed Scene Entity Records", [
        ["-p", "om9-core", "--test", "document_graph"],
    ]),
    ("OM9-L3-01-03-003", "Geometry Handle And Attributes", [
        ["-p", "om9-core", "--test", "document_graph"],
    ]),
    ("OM9-L3-01-03-004", "Layer Domain Records", [
        ["-p", "om9-core", "--test", "document_graph"],
    ]),
    ("OM9-L3-01-03-005", "Entity To Layer Membership", [
        ["-p", "om9-core", "--test", "document_graph"],
    ]),
    ("OM9-L3-01-03-006", "Layer Selection Visibility Rules", [
        ["-p", "om9-core", "--test", "document_graph"],
    ]),
    ("OM9-L3-01-03-007", "Group Membership Model", [
        ["-p", "om9-core", "--test", "document_graph"],
    ]),
    ("OM9-L3-01-03-008", "Block Definition And Instance Graph", [
        ["-p", "om9-core", "--test", "document_graph"],
        ["-p", "om9-scene", "--test", "shared_instances"],
    ]),
    ("OM9-L3-01-03-009", "Jewelry Semantic Extension Record", [
        ["-p", "om9-core", "--test", "document_graph"],
    ]),
    ("OM9-L3-01-03-010", "Selection State Document Contract", [
        ["-p", "om9-core", "--test", "selection_state"],
        ["-p", "om9-app", "--test", "document_contract"],
    ]),
    ("OM9-L3-01-03-011", "Document Change Notification", [
        ["-p", "om9-core", "--test", "document_changes"],
        ["-p", "om9-app", "--test", "document_contract"],
    ]),
    ("OM9-L3-01-03-012", "Document Snapshot And Restore", [
        ["-p", "om9-archive", "--test", "document_snapshot"],
        ["-p", "om9-app", "--test", "document_contract"],
    ]),
    ("OM9-L3-01-03-013", "Import Source Provenance Metadata", [
        ["-p", "om9-core", "--test", "provenance"],
    ]),
]

SUITES = [
    ("workspace", ["test", "--workspace", "--no-fail-fast"], "every crate together (covers the four crate suites)"),
    ("check", ["check", "--workspace", "--all-targets"], "everything still compiles"),
    ("fmt", ["fmt", "--check"], "formatting"),
    ("clippy", ["clippy", "--workspace", "--all-targets", "--", "-D", "warnings"], "lints, warnings are errors"),
]

# Used by --quick: the same coverage as the workspace run, crate by crate.
QUICK_SUITES = [
    ("core", ["test", "-p", "om9-core"], "IDs, records, layers, groups, blocks, extensions, changes, selection, provenance"),
    ("archive", ["test", "-p", "om9-archive"], "container admission, snapshot codec, identity round trip"),
    ("scene", ["test", "-p", "om9-scene"], "scene capture and picking of shared block instances"),
    ("app", ["test", "-p", "om9-app"], "the controller contract and every existing app suite"),
]

# The acceptance L4 of each L3; the module's evidence must exist for every one.
ACCEPTANCE = {l3: f"OM9-L4-01-03-{l3.split('-')[-1]}-05.regression.log" for l3, _, _ in ITEMS}


def run(command: list[str], cwd: pathlib.Path) -> tuple[int, str]:
    started = time.time()
    finished = subprocess.run(command, cwd=str(cwd), capture_output=True, text=True)
    text = (finished.stdout or "") + (finished.stderr or "")
    return finished.returncode, text if finished.returncode else f"{time.time() - started:.1f}s"


def summarise(text: str) -> str:
    failed = [line.strip() for line in text.splitlines() if line.startswith("test result: FAILED")]
    if failed:
        return failed[0]
    for line in text.splitlines():
        if line.startswith("test result:"):
            return line.strip()
    return text.strip().splitlines()[-1] if text.strip() else ""


def evidence_gaps() -> list[str]:
    gaps = []
    for l3, _, _ in ITEMS:
        folder = EVIDENCE / l3
        if not folder.is_dir():
            gaps.append(f"{l3}: no evidence folder")
            continue
        names = {path.name for path in folder.iterdir()}
        if ACCEPTANCE[l3] not in names:
            gaps.append(f"{l3}: no {ACCEPTANCE[l3]}")
    integration = EVIDENCE / "OM9-L3-01-03-008" / "OM9-L4-01-03-008-04.integration.log"
    if not integration.is_file():
        gaps.append("OM9-L3-01-03-008: no OM9-L4-01-03-008-04.integration.log")
    return gaps


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--quick",
        action="store_true",
        help="crate suites only (no workspace-wide run); fastest full-coverage pass",
    )
    parser.add_argument(
        "--evidence-only",
        action="store_true",
        help="13 focused L3 checks only (used while writing evidence logs)",
    )
    parser.add_argument(
        "--jobs",
        type=int,
        default=2,
        help="cargo build jobs; default 2 because this host's 8 GiB paging file cannot hold the "
        "default parallelism (rustc dies with 'os error 1455'). Test binaries still run one at a time.",
    )
    parser.add_argument("--self-test", action="store_true", help="check the wiring without running cargo")
    args = parser.parse_args()
    jobs = ["-j", str(args.jobs)] if args.jobs else []

    def with_jobs(command: list[str]) -> list[str]:
        """Only cargo build-like subcommands take -j, and never after `--`."""
        if not jobs or command[0] not in ("test", "check", "clippy", "build"):
            return command
        if "--" in command:
            split = command.index("--")
            return command[:split] + jobs + command[split:]
        return command + jobs

    if args.self_test:
        assert len(ITEMS) == 13, "the module has thirteen L3"
        assert len({l3 for l3, _, _ in ITEMS}) == 13, "every L3 appears once"
        assert all(targets for _, _, targets in ITEMS), "each item names at least one test target"
        assert len(ACCEPTANCE) == 13, "each L3 names its acceptance log"
        print("self-test: 13 L3 wired, evidence folders:", len(list(EVIDENCE.glob("OM9-L3-01-03-*"))))
        return 0

    rows = []
    failures = 0
    # The focused check is the per-L3 oracle; after the first build each of these
    # costs well under a second.
    for l3, title, targets in ITEMS:
        for index, target in enumerate(targets):
            code, detail = run(["cargo", "test", *target, *jobs], STANDALONE)
            rows.append((
                l3 if index == 0 else "",
                title if index == 0 else "",
                "ok" if code == 0 else "FAILED",
                summarise(detail),
            ))
            failures += code != 0

    # The workspace run already contains every crate suite, so running the four
    # crates separately as well would execute the same binaries twice (the app
    # suite alone is ~1/3 of the whole pass).
    suites = QUICK_SUITES if args.quick else SUITES
    if args.evidence_only:
        suites = []
    for label, command, why in suites:
        code, detail = run(["cargo", *with_jobs(command)], STANDALONE)
        rows.append((label, why, "ok" if code == 0 else "FAILED", summarise(detail)))
        failures += code != 0

    width = max(len(row[0]) for row in rows)
    print()
    print("OM9-L2-01-03 (Be Document Scene Entities) — automated verification")
    print("=" * (width + 46))
    for name, title, state, detail in rows:
        print(f"{name.ljust(width)}  {state:6}  {title}")
        if detail:
            print(f"{' ' * width}          {detail[:120]}")
    gaps = evidence_gaps()
    if gaps:
        print()
        print("evidence gaps:")
        for gap in gaps:
            print("  -", gap)
    print()
    print(f"{len(rows) - failures}/{len(rows)} groups ok, {failures} failed, {len(gaps)} evidence gaps")
    return 1 if failures or gaps else 0


if __name__ == "__main__":
    sys.exit(main())
