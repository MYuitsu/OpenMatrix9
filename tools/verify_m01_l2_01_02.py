#!/usr/bin/env python
"""OM9-L2-01-02 (Be Command Dispatch Undo Redo) — one automated verification pass.

Runs every test group that carries the eleven L3 items of the module, the
workspace check, the lints and the real-window harness, then prints one table and
exits non-zero if anything failed.

    python tools/verify_m01_l2_01_02.py                # full pass
    python tools/verify_m01_l2_01_02.py --quick        # no real-window harness
    python tools/verify_m01_l2_01_02.py --self-test    # check the table wiring
"""
from __future__ import annotations

import argparse
import json
import pathlib
import subprocess
import sys
import time

ROOT = pathlib.Path(__file__).resolve().parents[1]
STANDALONE = 
EVIDENCE = ROOT / "implementation/evidence/M01"

# One entry per L3: the test target that carries it, and its evidence folder.
ITEMS = [
    ("OM9-L3-01-02-001", "Command IDs And Alias Registry", ["-p", "om9-core", "--test", "command_registry"]),
    ("OM9-L3-01-02-002", "Typed Command Descriptor", ["-p", "om9-core", "--test", "command_descriptor"]),
    ("OM9-L3-01-02-003", "Command Context And Active Document", ["-p", "om9-app", "--test", "command_context"]),
    ("OM9-L3-01-02-004", "Interactive Command State Machine", ["-p", "om9-app", "--test", "command_state_machine"]),
    ("OM9-L3-01-02-005", "Command Option Parser", ["-p", "om9-app", "--test", "command_options_and_repeat"]),
    ("OM9-L3-01-02-006", "Atomic Command Transactions", ["-p", "om9-app", "--test", "command_transactions"]),
    ("OM9-L3-01-02-007", "Undo Redo Journal", ["-p", "om9-app", "--test", "command_transactions"]),
    ("OM9-L3-01-02-008", "Command Cancel Rollback", ["-p", "om9-app", "--test", "command_transactions"]),
    ("OM9-L3-01-02-009", "Command Repeat And Dispatch", ["-p", "om9-app", "--test", "command_options_and_repeat"]),
    ("OM9-L3-01-02-010", "Command Event And Observer API", ["-p", "om9-app", "--test", "command_events_surface"]),
    ("OM9-L3-01-02-011", "Command Availability Preconditions", ["-p", "om9-core", "--test", "command_availability"]),
]

SUITES = [
    ("lib", ["test", "-p", "om9-app", "--lib"], "the session and parser unit tests"),
    ("core", ["test", "-p", "om9-core"], "the registry and coordinator contract"),
    ("workspace", ["test", "--workspace", "-j", "2"], "every crate (‑j 2: the host paging file cannot hold the default parallelism)"),
    ("check", ["check", "--workspace", "--all-targets"], "everything still compiles"),
    ("fmt", ["fmt", "--check"], "formatting"),
    ("clippy", ["clippy", "--workspace", "--all-targets", "--", "-D", "warnings"], "lints, warnings are errors"),
]

WINDOW = ("real-window", ["python", str(ROOT / "tools/validate_m01_l3_003_panels.py"),
                          "--output", str(ROOT / "build/standalone-validation/ui-artifacts/l2-01-02-verify")],
          "the build still starts, docks and dispatches in a real window")


def run(command: list[str], cwd: pathlib.Path) -> tuple[int, str]:
    started = time.time()
    finished = subprocess.run(command, cwd=str(cwd), capture_output=True, text=True)
    text = (finished.stdout or "") + (finished.stderr or "")
    return finished.returncode, text if finished.returncode else f"{time.time() - started:.1f}s"


def summarise(text: str) -> str:
    for line in text.splitlines():
        if line.startswith("test result:"):
            return line.strip()
    for line in text.splitlines():
        if line.startswith("panels-summary") or line.startswith("matrix-summary"):
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
        if not any("ui-review" in name and name.endswith(".json") for name in names):
            gaps.append(f"{l3}: no ui-review file")
        if not any(name.endswith("regression.log") for name in names):
            gaps.append(f"{l3}: no -05 regression log")
    return gaps


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--quick", action="store_true", help="skip the real-window harness")
    parser.add_argument("--self-test", action="store_true", help="check the wiring without running cargo")
    args = parser.parse_args()

    if args.self_test:
        assert len(ITEMS) == 11, "the module has eleven L3"
        assert len({l3 for l3, _, _ in ITEMS}) == 11, "every L3 appears once"
        assert all(len(command) >= 4 for _, _, command in ITEMS), "each item names a test target"
        print("self-test: 11 L3 wired, evidence folders:", len(list(EVIDENCE.glob('OM9-L3-01-02-*'))))
        return 0

    rows = []
    failures = 0
    for l3, title, command in ITEMS:
        code, detail = run(["cargo", "test", *command], STANDALONE)
        rows.append((l3, title, "ok" if code == 0 else "FAILED", summarise(detail)))
        failures += code != 0

    for label, command, why in SUITES:
        cwd = STANDALONE
        code, detail = run(["cargo", *command], cwd)
        rows.append((label, why, "ok" if code == 0 else "FAILED", summarise(detail)))
        failures += code != 0

    if not args.quick:
        label, command, why = WINDOW
        code, detail = run(command, ROOT)
        rows.append((label, why, "ok" if code == 0 else "FAILED", summarise(detail)))
        failures += code != 0

    width = max(len(row[0]) for row in rows)
    print()
    print("OM9-L2-01-02 (Be Command Dispatch Undo Redo) — automated verification")
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
