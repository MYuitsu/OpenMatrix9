#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-2.1-or-later
"""Real-window evidence for OM9-L3-01-01-003 (menu reorder, collapse and float).

Runs the application with `--ui-audit` and `OM9_UI_AUDIT_PANELS=<scenario>`; the
application drives a real menu drag with real pointer events, records what the
preview showed and what the release applied, writes `ui-receipt.json` and
`workspace.png` itself. This tool checks those receipts against the same
expectations the fixtures state, and writes `panels-summary.json`.

Scenarios: reorder (Projects onto the DISPLAY bar), float (DISPLAY into a
viewport), dock-right (DISPLAY into the 48 px strip) and escape (cancel).

Usage:
  python tools/validate_m01_l3_003_panels.py [--binary PATH] [--output DIR]
                                             [--only NAME,NAME] [--self-test]
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import struct
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
WINDOW = "1600x1000"
WORK_AREA = [8.0, 29.0, 1584.0, 943.0]
DOCUMENTED_LEFT = [
    "icon-history",
    "main-menu",
    "display",
    "snaps",
    "info-settings",
    "layers",
    "projects",
    "builder-slot",
]


def without(order: list[str], *drop: str) -> list[str]:
    return [key for key in order if key not in drop]


def with_at(order: list[str], key: str, index: int, *drop: str) -> list[str]:
    rest = without(order, key, *drop)
    rest.insert(index, key)
    return rest


SCENARIOS: dict[str, dict] = {
    "reorder": {
        "drag": "projects",
        "label": "drag PROJECTS onto the upper half of the DISPLAY bar",
        "fixture": "positive.json P3 / P4 (drop index rule)",
        "expect": {
            "preview": {"Place": {"side": "left", "index": 2}},
            "indicator_line": True,
            "left_order": with_at(DOCUMENTED_LEFT, "projects", 2),
            "right_order": [],
            "floating": [],
            "status": "Projects moved to menu position 3",
            "op": {"Place": {"section": "projects", "side": "left", "index": 2}},
        },
    },
    "float": {
        "drag": "display",
        "label": "drag DISPLAY into a viewport and float it",
        "fixture": "positive.json P7 (float keeps a home), negative.json N5 (recovery)",
        "expect": {
            "preview": {"Float": None},
            "indicator_line": False,
            "left_order": without(DOCUMENTED_LEFT, "display"),
            "right_order": [],
            "floating": ["display"],
            "status": None,
            "op": {"Float": {"section": "display"}},
            "notes_include": "Recovered",
        },
    },
    "dock-right": {
        "drag": "display",
        "label": "drag DISPLAY into the 48 px strip at the right edge",
        "fixture": "positive.json P6 (dock right), derived_geometry right_drop_strip",
        "expect": {
            "preview": {"Place": {"side": "right", "index": 0}},
            "indicator_line": True,
            "left_order": without(DOCUMENTED_LEFT, "display"),
            "right_order": ["display"],
            "floating": [],
            "status": "Display docked to the right menu column",
            "op": {"Place": {"section": "display", "side": "right", "index": 0}},
        },
    },
    "escape": {
        "drag": "display",
        "label": "press DISPLAY, drag into a viewport and press Escape",
        "fixture": "negative.json N19 (Escape cancels and keeps the draft)",
        "expect": {
            "preview": {"Float": None},
            "indicator_line": False,
            "left_order": DOCUMENTED_LEFT,
            "right_order": [],
            "floating": [],
            "status": "Display drag cancelled (Esc)",
            "op": None,
        },
    },
    "roll-up": {
        "drag": "projects",
        "label": "click the PROJECTS title bar, then click it again",
        "fixture": "positive.json P1 (collapse then expand restores the state), P2",
        "expect": {
            "preview": None,
            "indicator_line": None,
            "left_order": DOCUMENTED_LEFT,
            "right_order": [],
            "floating": [],
            "status": "Projects expanded",
            "op": {"ToggleCollapsed": None},
            "extra": [
                ("after_first.collapsed", True),
                ("after_first.status", "Projects rolled up"),
                ("after_second.collapsed", False),
                ("after_second.status", "Projects expanded"),
            ],
        },
    },
    "float-drop": {
        "drag": "display",
        "label": "drag DISPLAY into a viewport: it floats and keeps its home",
        "fixture": "positive.json P7 (float records home), N5 (recovery in the drop)",
        "expect": {
            "preview": {"Float": None},
            "indicator_line": False,
            "left_order": without(DOCUMENTED_LEFT, "display"),
            "right_order": [],
            "floating": ["display"],
            "status": None,
            "op": {"Float": {"section": "display"}},
            "extra": [
                ("display_home.side", "left"),
                ("display_home.index", 2),
                ("floating_after_drop", ["display"]),
                ("display_float.2", 300.0),
            ],
            "limitations": "the X of the floating window is not scripted (a 16 px target): closing it is p7 plus the owner's item M3",
        },
    },
    "object-info": {
        "drag": None,
        "label": "draw one line and select it: the Object Info label the receipt carries",
        "fixture": "positive.json I1..I5 (L3-005): type, size and validity from the model",
        "expect": {
            "preview": None,
            "indicator_line": None,
            "left_order": DOCUMENTED_LEFT,
            "right_order": [],
            "floating": [],
            "status": None,
            "op": None,
            "draws": {"entities": 1, "undo_len": 1},
            "extra": [
                ("object_info_kind", "Line"),
                ("object_info_label", "Line \u00b7 10.000 \u00d7 4.000 \u00d7 0.000 \u00b7 valid"),
                ("selected", 1),
            ],
        },
    },
    "readback-recover": {
        "drag": "display",
        "label": "float DISPLAY, then let the real window report a rectangle that cannot fit",
        "fixture": "negative.json N14 (a shrink recovers the floating menu), M5",
        "expect": {
            "preview": None,
            "indicator_line": None,
            "left_order": without(DOCUMENTED_LEFT, "display"),
            "right_order": [],
            "floating": ["display"],
            "status": "Display was moved back inside the window",
            "op": {"SetFloatRect": {"section": "display"}},
            # The forced readback of a 300x40 rectangle is the thing that must clamp:
            # the state one frame later is the REAL window's own 132 px height, which
            # is above the 120 px minimum and legitimately accepted (round 14).
            "extra": [
                ("float_after_force.2", 300.0),
                ("float_after_force.3", 120.0),
                ("message_after_force", "Display was moved back inside the window"),
            ],
            "limitations": "the window keeps its audit size, so the repair comes from the rectangle the real window reports; the shrinking-window case is the owner's item M5",
        },
    },
}


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    digest.update(path.read_bytes())
    return digest.hexdigest()


def png_size(path: Path) -> list[int] | None:
    header = path.read_bytes()[:24]
    if header[:8] != b"\x89PNG\r\n\x1a\n":
        return None
    return list(struct.unpack(">II", header[16:24]))


def dig(value: object, path: str) -> object:
    """Follow a dotted path: `a.b.2` reads a list index too."""
    for part in path.split("."):
        if isinstance(value, list):
            try:
                value = value[int(part)]
            except (ValueError, IndexError):
                return None
        elif isinstance(value, dict):
            value = value.get(part)
        else:
            return None
    return value


def target_matches(actual: object, expected: dict) -> bool:
    """`{"Place": {"side": "left", "index": 2}}`; a null value means any."""
    if not isinstance(actual, dict) or len(actual) != 1:
        return False
    (kind, detail), = actual.items()
    if kind not in expected:
        return False
    want = expected[kind]
    if want is None:
        return True
    if not isinstance(detail, dict):
        return False
    return all(detail.get(key) == value for key, value in want.items())


def op_matches(actual: object, expected: dict | None) -> bool:
    if expected is None:
        return actual is None
    if not isinstance(actual, dict) or len(actual) != 1:
        return False
    (kind, detail), = actual.items()
    if kind not in expected:
        return False
    want = expected[kind]
    if want is None:
        return True
    if not isinstance(detail, dict):
        return False
    return all(detail.get(key) == value for key, value in want.items())


def notes_contain(notes: object, kind: str) -> bool:
    return isinstance(notes, list) and any(
        isinstance(note, dict) and kind in note for note in notes
    )


def run_scenario(binary: Path, output: Path, name: str, timeout: float) -> dict:
    scenario = SCENARIOS[name]
    audit = output / name
    if audit.exists():
        shutil.rmtree(audit)
    audit.mkdir(parents=True)
    environment = dict(os.environ)
    environment["OM9_UI_AUDIT_PANELS"] = name
    environment["OM9_UI_AUDIT_WINDOW"] = WINDOW
    environment.pop("OM9_UI_AUDIT_HOLD", None)
    problems: list[str] = []
    started = time.time()
    # Files, not pipes: the application must own a real console handle, and a
    # piped stdio under the harness sandbox is a documented failure mode.
    stdout_path = audit / "app-stdout.txt"
    stderr_path = audit / "app-stderr.txt"
    attempts: list[int] = []
    for _attempt in range(2):
        with stdout_path.open("w", encoding="utf-8") as stdout, stderr_path.open(
            "w", encoding="utf-8"
        ) as stderr:
            completed = subprocess.run(
                [str(binary), "--ui-audit", str(audit)],
                # The tree was merged to the repository root (AGENTS.md, 2026-10-17):
                # the old standalone/ directory no longer exists.
                cwd=str(ROOT),
                env=environment,
                stdout=stdout,
                stderr=stderr,
                timeout=timeout,
                check=False,
            )
        attempts.append(completed.returncode)
        if completed.returncode == 0 or (audit / "ui-receipt.json").exists():
            break
        # Windows occasionally kills a freshly launched audit process (seen twice
        # on this host, never reproducible alone): one retry, recorded in the
        # summary so the evidence says it happened.
        shutil.rmtree(audit, ignore_errors=True)
        audit.mkdir(parents=True, exist_ok=True)
    elapsed = round(time.time() - started, 2)
    stderr_text = stderr_path.read_text(encoding="utf-8", errors="replace").strip()
    receipt_path = audit / "ui-receipt.json"
    png_path = audit / "workspace.png"
    observed: dict = {
        "exit_code": completed.returncode,
        "elapsed_seconds": elapsed,
        "attempts": attempts,
        "retried": len(attempts) > 1,
    }
    if completed.returncode != 0:
        problems.append(f"exit code {completed.returncode}")
    if stderr_text:
        problems.append(f"stderr: {stderr_text[:200]}")
    if not receipt_path.exists() or not png_path.exists():
        problems.append("receipt or screenshot missing")
        return {
            "scenario": name,
            "label": scenario["label"],
            "fixture": scenario["fixture"],
            "passed": False,
            "problems": problems,
            "observed": observed,
        }
    receipt = json.loads(receipt_path.read_text(encoding="utf-8"))
    panels = receipt.get("panels") or {}
    entries = panels.get("checks") or []
    # The audit may record more than one entry; take the one this scenario wrote.
    checks = next((entry for entry in entries if entry.get("scenario") == name), entries[0] if entries else None)
    if not checks:
        problems.append("the receipt carries no panel checks")
        checks = {}
    expect = scenario["expect"]

    preview = checks.get("observed_preview")
    if expect.get("preview") is not None and not preview:
        problems.append("no preview was observed while the drag was pressed")
    elif expect.get("preview") is not None:
        if not target_matches(preview.get("target"), expect["preview"]):
            problems.append(
                f"preview target {preview.get('target')} != {expect['preview']}"
            )
        indicator = preview.get("indicator")
        if expect["indicator_line"]:
            if not isinstance(indicator, dict) or not indicator.get("line"):
                problems.append("the preview did not draw the 3 px insertion line")
            elif abs(float(indicator["rect"]["height"]) - 3.0) > 0.01:
                problems.append("the insertion line is not 3 points high")
        elif indicator is not None and indicator.get("line"):
            problems.append("a line indicator was drawn for a float drop")

    for path, wanted in expect.get("extra", []):
        actual = dig(checks.get("extra"), path)
        if actual != wanted:
            problems.append(f"extra {path} {actual!r} != {wanted!r}")

    last_op = checks.get("last_op")
    if not op_matches((last_op or {}).get("op"), expect["op"]):
        problems.append(f"applied {(last_op or {}).get('op')} != {expect['op']}")
    if expect["op"] is not None and not (last_op or {}).get("ok"):
        problems.append("the applied operation is not reported as ok")
    if expect.get("notes_include") and not notes_contain(
        (last_op or {}).get("notes"), expect["notes_include"]
    ):
        problems.append(f"notes {((last_op or {}).get('notes'))} lack {expect['notes_include']}")

    for key in ("left_order", "right_order", "floating"):
        if checks.get(key) != expect[key]:
            problems.append(f"{key} {checks.get(key)} != {expect[key]}")
    if checks.get("drag_in_progress") is not False:
        problems.append("a drag is still in progress after the script")
    if expect["status"] is not None and checks.get("status_message") != expect["status"]:
        problems.append(
            f"status {checks.get('status_message')!r} != {expect['status']!r}"
        )
    # A panel drag must not touch the document. The one scenario that draws on
    # purpose (object-info) declares `draws` and is exempt, with the exact counts
    # it expects instead.
    if not expect.get("draws"):
        if checks.get("entities") not in (0, None):
            problems.append("a menu drag changed document entities")
        if checks.get("undo_len") not in (0, None):
            problems.append("a menu drag changed the undo journal")
    else:
        if checks.get("entities") != expect["draws"].get("entities"):
            problems.append(
                f"drew {checks.get('entities')!r} entities, expected {expect['draws'].get('entities')!r}"
            )
        if checks.get("undo_len") != expect["draws"].get("undo_len"):
            problems.append(
                f"journal {checks.get('undo_len')!r} != {expect['draws'].get('undo_len')!r}"
            )
    if checks.get("work_area") != WORK_AREA:
        problems.append(f"work area {checks.get('work_area')} != {WORK_AREA}")
    if not receipt.get("screenshot_saved"):
        problems.append("the application reported the screenshot as not saved")
    if png_size(png_path) != [1600, 1000]:
        problems.append(f"screenshot size {png_size(png_path)} != [1600, 1000]")
    if receipt.get("passes", 0) < 8:
        problems.append(f"the script only ran {receipt.get('passes')} passes")

    observed.update(
        {
            "preview_target": (preview or {}).get("target"),
            "indicator": (preview or {}).get("indicator"),
            "last_op": last_op,
            "left_order": checks.get("left_order"),
            "right_order": checks.get("right_order"),
            "floating": checks.get("floating"),
            "status_kind": checks.get("status_kind"),
            "status_message": checks.get("status_message"),
            "history_tail": checks.get("history_tail"),
            "work_area": checks.get("work_area"),
            "entities": checks.get("entities"),
            "undo_len": checks.get("undo_len"),
            "passes": receipt.get("passes"),
            "screenshot_px": png_size(png_path),
        }
    )
    return {
        "scenario": name,
        "label": scenario["label"],
        "fixture": scenario["fixture"],
        "limitations": scenario["expect"].get("limitations"),
        "env": {"OM9_UI_AUDIT_PANELS": name, "OM9_UI_AUDIT_WINDOW": WINDOW},
        "receipt": str(receipt_path.relative_to(ROOT)).replace("\\", "/"),
        "receipt_sha256": sha256(receipt_path),
        "screenshot": str(png_path.relative_to(ROOT)).replace("\\", "/"),
        "screenshot_sha256": sha256(png_path),
        "observed": observed,
        "problems": problems,
        "passed": not problems,
    }


def self_test() -> int:
    """The comparators must accept the recorded shapes and reject drift."""
    assert target_matches({"Place": {"side": "left", "index": 2}},
                          {"Place": {"side": "left", "index": 2}})
    assert not target_matches({"Place": {"side": "left", "index": 3}},
                              {"Place": {"side": "left", "index": 2}})
    assert target_matches({"Float": {"pointer": [1, 2]}}, {"Float": None})
    assert not target_matches({"Cancel": {"reason": "NoTarget"}}, {"Float": None})
    assert op_matches(None, None)
    assert not op_matches({"Dock": {"section": "display", "side": "left"}}, None)
    assert op_matches({"Place": {"section": "display", "side": "right", "index": 0}},
                      {"Place": {"section": "display", "side": "right", "index": 0}})
    assert notes_contain([{"Recovered": {"section": "display"}}], "Recovered")
    assert not notes_contain([], "Recovered")
    assert with_at(DOCUMENTED_LEFT, "projects", 2)[2] == "projects"
    assert without(DOCUMENTED_LEFT, "display") == [
        "icon-history", "main-menu", "snaps", "info-settings", "layers", "projects", "builder-slot"
    ]
    print("self-test: comparators and expectations are consistent")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", default=None)
    parser.add_argument("--output", default=None)
    parser.add_argument("--only", default=None, help="comma separated scenario names")
    parser.add_argument("--timeout", type=float, default=120.0)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        return self_test()

    # The workspace builds with CARGO_TARGET_DIR outside the checkout (AGENTS.md,
    # 2026-10-17): honour it, then fall back to the in-tree target.
    if args.binary:
        binary = Path(args.binary)
    else:
        target = os.environ.get("CARGO_TARGET_DIR")
        binary = Path(target) / "debug/om9.exe" if target else ROOT / "target/debug/om9.exe"
    binary = binary.resolve()
    if not binary.exists():
        print(f"binary not found: {binary}", file=sys.stderr)
        return 2
    stamp = time.strftime("%Y-%m-%d")
    output = Path(args.output) if args.output else (
        ROOT / "build/standalone-validation/ui-artifacts" / f"{stamp}-m01-l3-003-panels"
    )
    # Absolute: the application runs with the workspace root as its cwd.
    output = output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    names = list(SCENARIOS)
    if args.only:
        names = [name.strip() for name in args.only.split(",") if name.strip()]
        unknown = [name for name in names if name not in SCENARIOS]
        if unknown:
            print(f"unknown scenario(s): {unknown}", file=sys.stderr)
            return 2

    cases = [run_scenario(binary, output, name, args.timeout) for name in names]
    for case in cases:
        state = "PASS" if case["passed"] else "FAIL"
        print(f"{state} [{case['scenario']}] {case['label']}")
        for problem in case["problems"]:
            print(f"     - {problem}")
    summary = {
        "schema": "om9.m01.l3_003.panels-summary.v1",
        "l3_id": "OM9-L3-01-01-003",
        "binary": str(binary),
        "window": WINDOW,
        "timestamp": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "cases": cases,
        "all_passed": all(case["passed"] for case in cases),
    }
    target = output / "panels-summary.json"
    target.write_text(json.dumps(summary, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"panels-summary: {target}")
    return 0 if summary["all_passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
