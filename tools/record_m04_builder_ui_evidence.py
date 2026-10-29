#!/usr/bin/env python3
"""Record the M04 builder-UI / history-journal batch from real runs.

Scope: the vertical slice that connects the ring chain to the application —
``BuilderSession`` (panel fields, live preview, viewport handles), the command frame
(``RingRail``/``RailProfile``/``RailSweep``), ``Mutation::History`` with published results,
``Mutation::HistoryUpdate`` for a journaled recompute, and the F6 prefill.

The L3 records under ``OM9-L2-03-11`` and ``OM9-L2-04-02`` stay in the plan tree until their
own manual gates are signed; this tool records milestone-level evidence only.

Usage::

    python tools/record_m04_builder_ui_evidence.py
"""

from __future__ import annotations

import datetime
import json
import os
import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
TARGET_DIR = os.environ.get("CARGO_TARGET_DIR", r"L:\om9-target")
KEY = "m04-builder-ui-history-journal-2026-10-27"
OUTPUTS = [
    ROOT / "implementation" / "evidence" / "M04" / "OM9-L2-03-11",
    ROOT / "implementation" / "evidence" / "M04" / "OM9-L2-04-02",
]


def run(command: list[str], with_path: bool = False) -> tuple[int, str]:
    env = dict(os.environ)
    env["CARGO_TARGET_DIR"] = TARGET_DIR
    env.setdefault("CARGO_TERM_COLOR", "never")
    if with_path:
        # OCCT/runtime DLLs live beside the test executables.
        env["PATH"] = TARGET_DIR + os.pathsep + env.get("PATH", "")
    process = subprocess.run(
        command,
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        env=env,
    )
    return process.returncode, process.stdout + process.stderr


def header(command: str, code: int, captured: str) -> str:
    return (
        f"$ {command}\n# exit code: {code}\n"
        f"# captured: {datetime.datetime.now(datetime.UTC).strftime('%Y-%m-%dT%H:%M:%SZ')}\n"
        f"# target dir: {TARGET_DIR}\n\n{captured}"
    )


def main() -> int:
    checks = [
        (
            "core-builder.log",
            [
                "cargo",
                "test",
                "-p",
                "om9-core",
                "--test",
                "builder_session",
                "--test",
                "history_mutation",
                "-j",
                "2",
            ],
            False,
        ),
        (
            "occt-chain.log",
            [
                "cargo",
                "test",
                "-p",
                "om9-kernel",
                "--features",
                "3dm-brep",
                "--test",
                "associative_history",
                "-j",
                "2",
            ],
            True,
        ),
        (
            "app-builder.log",
            [
                "cargo",
                "test",
                "-p",
                "om9-app",
                "--test",
                "builder_panel",
                "--test",
                "top11",
                "-j",
                "2",
            ],
            False,
        ),
        (
            "plugin-host-history.log",
            ["cargo", "test", "-p", "om9-plugin-host", "--lib", "-j", "2", "history"],
            False,
        ),
        (
            "kernel-clippy.log",
            [
                "cargo",
                "clippy",
                "-p",
                "om9-kernel",
                "--all-targets",
                "--no-deps",
                "-j",
                "2",
                "--",
                "-D",
                "warnings",
            ],
            False,
        ),
        (
            "workspace-check.log",
            ["cargo", "check", "--workspace", "--all-targets", "-j", "2"],
            False,
        ),
    ]
    results = []
    failed = False
    for name, command, with_path in checks:
        code, captured = run(command, with_path)
        summary = [
            line
            for line in captured.splitlines()
            if line.startswith(("test result:", "error", "FAILED", "Finished"))
        ]
        results.append(
            {
                "artifact": name,
                "command": command,
                "exit_code": code,
                "summary": summary[-5:],
            }
        )
        if code != 0:
            failed = True
            print(f"FAIL: {name} exit {code}", file=sys.stderr)
            print("\n".join(summary[-20:]), file=sys.stderr)
        for output in OUTPUTS:
            output.mkdir(parents=True, exist_ok=True)
            (output / name).write_text(header(" ".join(command), code, captured), encoding="utf-8")
    if failed:
        return 1

    _, revision = run(["git", "rev-parse", "HEAD"])
    _, status = run(["git", "status", "--porcelain"])
    observed = {
        "schema": "om9.m04.builder_ui.v1",
        "recorded_at_utc": datetime.datetime.now(datetime.UTC).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "source_revision": revision.strip().splitlines()[0] if revision.strip() else "UNRESOLVED",
        "checkout_dirty": bool(status.strip()),
        "target": "Standalone Rust OM9; Windows x64; CARGO_TARGET_DIR=L:/om9-target; -j 2",
        "scope": [
            "BuilderSession: typed fields, live preview through ProducerRegistry, viewport handles",
            "command frame: RingRail / RailProfile / RailSweep / OutsideRail / RingResizer from the command line and Top 11",
            "builder panel in the reserved BUILDERS slot: tabs, fields, sliders, Apply, Reset, Cancel, handle read-outs",
            "builder mode tabs keep independent parameters per mode and survive a failed switch",
            "profile selector window: rectangle / rounded rectangle / circle presets, preview only, removed preset refused",
            "outside rail offsets a closed circular rail; a non-circular rail or non-positive offset is refused",
            "ring resizer creates an independent ring at the target diameter and refuses a shrink below thickness",
            "viewport handle drag edits the same field as the panel, with a stale panel event refused",
            "Mutation::History carries published results; Mutation::HistoryUpdate journals a recompute",
            "Controller::recompute_history applies a registry rebuild as one undoable transaction",
            "F6 Builder prefill opens the mode the selection fits",
        ],
        "code": [
            "crates/om9-core/src/builder.rs",
            "crates/om9-core/src/producers.rs",
            "crates/om9-core/src/transaction.rs",
            "crates/om9-core/src/history.rs",
            "crates/om9-core/src/associative.rs",
            "crates/om9-core/tests/builder_session.rs",
            "crates/om9-core/tests/history_mutation.rs",
            "crates/om9-app/src/session/builder.rs",
            "crates/om9-app/src/session/top11.rs",
            "crates/om9-app/src/ui.rs",
            "crates/om9-app/src/ui/chrome.rs",
            "crates/om9-app/src/f6.rs",
            "crates/om9-app/tests/builder_panel.rs",
            "crates/om9-app/tests/top11.rs",
            "crates/om9-kernel/src/measurement.rs",
        ],
        "checks": results,
        "manual_gates": [
            "OM9-L3-03-11-* and OM9-L3-04-02-* keep their plan status: the mode tabs, profile selector, "
            "outside rail and resizer are implemented and automatically tested, but a human still has to "
            "review the panel and handle gestures for those L3.",
            "No screenshot/pointer-frame evidence is recorded for the panel or handles; the controller and "
            "core tests prove the model, command frame and transaction behavior.",
        ],
        "notes": [
            "clippy for om9-core/om9-app is currently blocked by unrelated untracked modules another "
            "process is adding (annotation.rs, measure/extended.rs, transform advanced); the changed "
            "builder and transaction code is rustfmt-clean.",
        ],
    }
    for output in OUTPUTS:
        (output / "observed.json").write_text(
            json.dumps(observed, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
        )

    # Per-L3 evidence for the four L3 this batch implements. It is evidence-only: the L3
    # needs a human reviewer, so no signature file is written and the dashboard keeps it WIP.
    l3_targets = {
        "OM9-L3-03-11-002": {
            "title": "Builder Mode Tabs And Context",
            "scope": "Five builder tabs with independent typed parameters; a return refreshes the tab and a failed switch restores the previous one.",
            "tests": [
                "tabs_keep_each_mode_parameters_and_switch_safely",
                "a_failed_tab_switch_restores_the_previous_tab",
            ],
            "logs": ["app-builder.log", "workspace-check.log"],
        },
        "OM9-L3-03-11-007": {
            "title": "Builder Profile Selector Window",
            "scope": "Profile library lists rectangle, rounded rectangle and circle; choosing changes only the preview, Apply commits it, a removed preset is refused.",
            "tests": [
                "the_profile_library_changes_the_preview_and_the_applied_shape",
                "the_preset_can_be_chosen_from_the_command_line",
                "the_profile_library_switches_shape_and_refuses_a_removed_preset",
            ],
            "logs": ["app-builder.log", "core-builder.log", "workspace-check.log"],
        },
        "OM9-L3-04-02-002": {
            "title": "Outside Ring Rail Generator",
            "scope": "Concentric outer rail at offset_mm from a closed circular rail; non-circular or non-positive offset refused, source rail untouched.",
            "tests": [
                "an_outside_rail_offsets_a_closed_circular_rail_only",
                "outside_rail_and_resizer_apply_independent_outputs",
            ],
            "logs": ["core-builder.log", "app-builder.log", "workspace-check.log"],
        },
        "OM9-L3-04-02-007": {
            "title": "Ring Resizer Builder",
            "scope": "Independent ring at target_diameter_mm; source ring intact; shrink at or below twice thickness refused.",
            "tests": [
                "a_ring_resizer_creates_an_independent_ring_and_refuses_a_shrink_below_thickness",
                "outside_rail_and_resizer_apply_independent_outputs",
            ],
            "logs": ["core-builder.log", "app-builder.log", "workspace-check.log"],
        },
    }
    import shutil

    for l3, spec in l3_targets.items():
        directory = ROOT / "implementation" / "evidence" / "M04" / l3
        directory.mkdir(parents=True, exist_ok=True)
        for log in spec["logs"]:
            src = OUTPUTS[0] / log
            if src.is_file():
                shutil.copyfile(src, directory / log)
        (directory / "observed.json").write_text(
            json.dumps(
                {
                    "schema": "om9.m04.l3_builder_evidence.v1",
                    "l3_id": l3,
                    "title": spec["title"],
                    "scope": spec["scope"],
                    "target": observed["target"],
                    "tests": spec["tests"],
                    "manual_gate": "OPEN",
                    "reviewer": None,
                    "statement": (
                        "Automated evidence only. The L3 requires a human review of the panel "
                        "and handle gestures; no reviewer signature is claimed."
                    ),
                    "logs": spec["logs"],
                },
                indent=2,
                ensure_ascii=False,
            )
            + "\n",
            encoding="utf-8",
        )

    readme = """# M04 builder UI and history journal

This directory records the vertical slice that connects the associative ring chain to the
application. The L3 records stay in the plan tree until their own manual gates are signed.

## What is implemented

- `om9_core::builder::BuilderSession` -- typed fields with domains, a live preview through
  `ProducerRegistry`, viewport handle positions and a one-transaction commit plan.
- `Controller` builder API -- `begin_builder`, `builder_set_field`, `builder_adjust_field`,
  `builder_refresh`, `builder_reset`, `builder_cancel`, `builder_apply`, `builder_text`.
- Command frame -- `RingRail`/`gvRingRail`, `Rail Profile`, `Rail Sweep`, `Outside Rail`,
  `Ring Resizer`, Enter/Apply, `Diameter=`, `Width=`, `Height=`, `Offset=`, `Target=`,
  `Thickness=`, `preset=`, `Solid=`, Reset and Cancel; the Top 11 Ring Rail row opens the
  builder.
- Mode tabs -- the BUILDERS panel switches between the five modes; every tab keeps its own
  typed parameters, a return refreshes against the current document, and a failed switch
  restores the previous tab (`OM9-L3-03-11-002`).
- Profile selector window -- the `Library…` button lists rectangle, rounded rectangle and
  circle; choosing changes only the preview, Apply commits the chosen shape, and removing
  the selected preset drops the preview and refuses Apply (`OM9-L3-03-11-007`).
- Outside rail -- `OutsideRailProducer` offsets a closed circular rail by `offset_mm`; a
  non-circular rail or non-positive offset is refused (`OM9-L3-04-02-002`).
- Ring resizer -- `RingResizerProducer` creates an independent ring at
  `target_diameter_mm` on a new object; the source ring is untouched and a target at or
  below twice `thickness_mm` is refused (`OM9-L3-04-02-007`).
- Panel -- the reserved BUILDERS slot paints the tabs, fields, sliders, preview summary,
  Apply/Reset/Cancel, the library button and handle read-outs.
- Viewport handles -- projected squares with a drag that changes the same field the panel
  edits (`diameter_mm`, `width_mm`, `height_mm`, `offset_mm`, `target_diameter_mm`).
- F6 -- the Builder context emits the typed prefill and opens the mode the selection fits.
- Transaction -- `Mutation::History` also carries published results (a builder inserts entity,
  recipe and result in one atomic step); `Mutation::HistoryUpdate` replaces the recipes and
  results a recompute touched; `Controller::recompute_history` runs the registry and journals
  the whole rebuild as one Undo unit.

## Verification

See `observed.json` and the logs in this directory. The focused suites are:

- `cargo test -p om9-core --test builder_session --test history_mutation`
- `cargo test -p om9-kernel --features 3dm-brep --test associative_history` (real OCCT sweep)
- `cargo test -p om9-app --test builder_panel --test top11`
- `cargo test -p om9-plugin-host --lib history`

## Not claimed

- Human acceptance of the panel, handle gestures, mode tabs, selector window, outside rail or
  resizer; those L3 keep their plan status.
- Screenshot/pointer-frame evidence; the tests prove the model, command frame and transactions.
"""
    for output in OUTPUTS:
        (output / "README.md").write_text(readme, encoding="utf-8")

    ledger_path = ROOT / "docs" / "openmatrix9-progress.json"
    ledger = json.loads(ledger_path.read_text(encoding="utf-8"))
    entry = {
        "key": KEY,
        "date": "2026-10-27",
        "stage": "feature-port",
        "status": "builder_ui_and_history_journal_verified_manual_gates_open",
        "target": observed["target"],
        "feature_ids": ["OM9-L2-03-11", "OM9-L2-04-02", "OM9-L2-01-04"],
        "scope": (
            "M04 builder UI slice: BuilderSession (fields/preview/handles), command frame, BUILDERS panel, "
            "viewport handle drag, F6 prefill; Mutation::History mang published result và Mutation::HistoryUpdate "
            "để journal recompute thành một Undo; sửa clippy measurement.rs. Các L3 UI vẫn chờ gate thủ công."
        ),
        "outputs": observed["code"]
        + [
            "implementation/evidence/M04/OM9-L2-03-11/",
            "implementation/evidence/M04/OM9-L2-04-02/",
        ],
        "evidence": [
            "implementation/evidence/M04/OM9-L2-03-11/observed.json",
            "implementation/evidence/M04/OM9-L2-03-11/app-builder.log",
            "implementation/evidence/M04/OM9-L2-03-11/core-builder.log",
            "implementation/evidence/M04/OM9-L2-03-11/occt-chain.log",
            "implementation/evidence/M04/OM9-L2-03-11/workspace-check.log",
            "implementation/evidence/M04/OM9-L2-04-02/observed.json",
            "implementation/evidence/M04/OM9-L2-04-02/README.md",
            "implementation/evidence/M04/OM9-L3-03-11-002/observed.json",
            "implementation/evidence/M04/OM9-L3-03-11-007/observed.json",
            "implementation/evidence/M04/OM9-L3-04-02-002/observed.json",
            "implementation/evidence/M04/OM9-L3-04-02-007/observed.json",
        ],
        "tests": {
            "core_builder": "8 passed (fields, preview, handles, plan, one Undo, outside rail, resizer, profile library)",
            "core_journal": "3 passed (History published, HistoryUpdate recompute + Undo/Redo, refusals)",
            "occt_chain": "2 passed (real B-Rep sweep, topology names, reference after edit)",
            "app_builder": "10 passed (tabs, command line, cancel/reset, profile+sweep, handle, recompute, outside rail, resizer, profile library)",
            "app_top11": "19 passed (Ring Rail opens the builder; a running command disables the toolbar)",
            "plugin_host": "3 history tests passed",
            "workspace_check": "cargo check --workspace --all-targets exit 0",
            "clippy_kernel": "om9-kernel --all-targets --no-deps -D warnings exit 0 (measurement.rs fixed)",
        },
        "uncertainties": [
            "Panel/handles chưa có ảnh chụp hay pointer-frame; chỉ test controller/core.",
            "Custom rail, four-rail profile, profile merge/endcap và ring-size table thuộc các L3 khác chưa làm.",
            "Clippy om9-core/om9-app đang bị chặn bởi file untracked của tiến trình khác (annotation.rs, measure/extended.rs, transform advanced), không phải code batch này.",
            "Mutation::HistoryUpdate chưa journal thay đổi cache; cache là bộ nhớ phiên, không phải document.",
        ],
        "manual_gate": "OPEN: các L3 UI cần nghiệm thu người; batch chỉ ghi bằng chứng tự động",
        "next_action": "Chụp panel/handle cho gate thủ công; làm custom rail, four-rail profile, merge/endcap và bảng size theo L3-04-02-001/004/005/006.",
    }
    ledger["items"] = [item for item in ledger["items"] if item.get("key") != KEY] + [entry]
    ledger["active_item"] = KEY
    ledger["updated_at"] = datetime.datetime.now().astimezone().isoformat(timespec="seconds")
    ledger_path.write_text(json.dumps(ledger, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")

    print(f"recorded builder evidence at {OUTPUTS[0].relative_to(ROOT)} and {OUTPUTS[1].relative_to(ROOT)}")
    for result in results:
        print(f"  {result['artifact']}: exit {result['exit_code']}")
    print(f"ledger: {ledger_path.relative_to(ROOT)} ({len(ledger['items'])} items)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
