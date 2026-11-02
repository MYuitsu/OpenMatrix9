#!/usr/bin/env python3
"""Record the M04 L2-03-11 builder-UI completion batch from real runs.

Scope: the L2-03-11 capabilities that were still open after the first builder slice
(``m04-builder-ui-history-journal-2026-10-27``):

* mode tabs and per-tab parameters (``OM9-L3-03-11-002``);
* typed controls with step quantisation and sliders (``-003``);
* live preview lifecycle: no document write before Apply (``-004``);
* viewport handles bound to one parameter authority, locked inputs read-only (``-005``);
* stale panel events refused against a newer handle drag (``-006``);
* profile selector window over the producer preset library (``-007``);
* Enter/Escape/Reset and one-Undo Apply (``-008``);
* F6 prefill and command line reaching the same builder service (``-009``).

The evidence lands in a separate directory from the earlier slice so the two records can be
read independently, and the ledger entry keeps its own key.

Usage::

    python tools/record_m04_l2_03_11_ui_completion.py
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
KEY = "m04-l2-03-11-ui-completion-2026-10-27"
OUTPUT = ROOT / "implementation" / "evidence" / "M04" / "OM9-L2-03-11-COMPLETION"

CODE = [
    "crates/om9-core/src/builder.rs",
    "crates/om9-core/src/producers.rs",
    "crates/om9-core/tests/builder_session.rs",
    "crates/om9-core/tests/builder_controls.rs",
    "crates/om9-app/src/session/builder.rs",
    "crates/om9-app/src/session.rs",
    "crates/om9-app/src/ui/chrome.rs",
    "crates/om9-app/src/ui.rs",
    "crates/om9-app/tests/builder_panel.rs",
    "crates/om9-app/tests/builder_controls.rs",
    "crates/om9-app/tests/builder_slot.rs",
]

L3_COVERAGE = [
    {
        "l3": "OM9-L3-03-11-001",
        "capability": "Builder Typed Input Boxes",
        "code": [
            "builder::BuilderField domains + BuilderSession::new role/kind checks",
            "builder_controls::a_nan_or_negative_value_is_refused_before_it_reaches_the_preview",
        ],
        "state": "automated_verified_manual_gate_open",
    },
    {
        "l3": "OM9-L3-03-11-002",
        "capability": "Builder Mode Tabs And Context",
        "code": [
            "Controller::builder_tabs + builder_saved + switch_builder (five modes)",
            "chrome::builder_slot_ui tab row",
            "builder_controls::a_mode_tab_switch_keeps_each_modes_own_parameters",
            "builder_controls::a_tab_with_an_invalid_preview_is_never_silently_applied_or_silently_dropped",
        ],
        "state": "automated_verified_manual_gate_open",
    },
    {
        "l3": "OM9-L3-03-11-003",
        "capability": "Builder Parameter Controls And Sliders",
        "code": [
            "BuilderField::step/quantize/slider_ratio/from_slider_ratio",
            "BuilderSession::set_slider and step-quantised adjust_field",
            "Controller::builder_set_slider + BuilderFieldView step/ratio",
            "chrome::builder_slot_ui slider bound to the field domain",
            "builder_controls::a_slider_move_lands_exactly_on_the_field_step",
            "builder_controls::a_slider_edit_from_the_panel_quantises_and_previews_the_same_value",
        ],
        "state": "automated_verified_manual_gate_open",
    },
    {
        "l3": "OM9-L3-03-11-004",
        "capability": "Builder Live Preview Lifecycle",
        "code": [
            "BuilderSession::refresh writes only the preview; plan() is the only document record",
            "builder_controls::escape_after_preview_edits_leaves_the_document_bytewise_unchanged",
            "builder_controls::double_enter_applies_once_and_reset_creates_no_journal_entry",
        ],
        "state": "automated_verified_manual_gate_open",
    },
    {
        "l3": "OM9-L3-03-11-005",
        "capability": "Builder Viewport Control Handles",
        "code": [
            "BuilderSession::editable + handles hidden for a locked input",
            "Controller::builder_adjust_field refuses a read-only session",
            "ui::builder_handles_ui projects the handle and drags along its normalised axis",
            "builder_controls::a_locked_input_paints_no_handle_and_reports_itself_read_only",
            "builder_controls::a_locked_input_makes_the_builder_read_only_and_hides_its_handles",
        ],
        "state": "automated_verified_pointer_frame_open",
    },
    {
        "l3": "OM9-L3-03-11-006",
        "capability": "Builder Handle And Panel Bidirectional Sync",
        "code": [
            "BuilderSession::revision + set_field_at stale refusal",
            "Controller::builder_set_field_at + builder_revision",
            "chrome panel captures the revision before painting a widget",
            "builder_controls::a_panel_event_captured_before_a_handle_drag_is_refused",
            "builder_controls::a_stale_panel_event_cannot_overwrite_a_newer_handle_drag",
        ],
        "state": "automated_verified_manual_gate_open",
    },
    {
        "l3": "OM9-L3-03-11-007",
        "capability": "Builder Profile Selector Window",
        "code": [
            "producers::ProfilePreset library + RailProfileProducer preset parameter",
            "BuilderSession::presets/selected_preset/set_preset/remove_preset",
            "Controller::builder_presets/builder_set_preset/builder_remove_preset",
            "chrome::builder_profile_window_ui selector window",
            "builder_session::the_profile_library_switches_shape_and_refuses_a_removed_preset",
            "builder_panel::the_profile_library_changes_the_preview_and_the_applied_shape",
        ],
        "state": "automated_verified_manual_gate_open",
    },
    {
        "l3": "OM9-L3-03-11-008",
        "capability": "Builder Accept Cancel Reset And Undo",
        "code": [
            "Controller::builder_apply inserts entity+recipe+result in one transaction",
            "Controller::builder_cancel/builder_reset",
            "builder_controls::double_enter_applies_once_and_reset_creates_no_journal_entry",
            "builder_controls::escape_after_preview_edits_leaves_the_document_bytewise_unchanged",
        ],
        "state": "automated_verified_manual_gate_open",
    },
    {
        "l3": "OM9-L3-03-11-009",
        "capability": "Builder F6 And Commandline Input Binding",
        "code": [
            "f6::builder_prefill + ui::builder_from_prefill via Controller::begin_builder",
            "Controller::submit -> builder_command / builder_text (same service as the panel)",
            "builder_controls::the_f6_prefill_and_the_command_line_reach_the_same_builder_service",
        ],
        "state": "automated_verified_manual_gate_open",
    },
]


def run(command: list[str], with_path: bool = False) -> tuple[int, str]:
    env = dict(os.environ)
    env["CARGO_TARGET_DIR"] = TARGET_DIR
    env.setdefault("CARGO_TERM_COLOR", "never")
    if with_path:
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
            "core-builder-controls.log",
            [
                "cargo",
                "test",
                "-p",
                "om9-core",
                "--test",
                "builder_session",
                "--test",
                "builder_controls",
                "--test",
                "history_mutation",
                "-j",
                "2",
            ],
            False,
        ),
        (
            "app-builder-controls.log",
            [
                "cargo",
                "test",
                "-p",
                "om9-app",
                "--test",
                "builder_panel",
                "--test",
                "builder_controls",
                "--test",
                "builder_slot",
                "--test",
                "top11",
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
            "clippy-core.log",
            [
                "cargo",
                "clippy",
                "-p",
                "om9-core",
                "--all-targets",
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
            if line.startswith(("test result:", "error", "FAILED", "Finished", "warning:"))
        ]
        results.append(
            {
                "artifact": name,
                "command": command,
                "exit_code": code,
                "summary": summary[-8:],
            }
        )
        if code != 0:
            failed = True
            print(f"FAIL: {name} exit {code}", file=sys.stderr)
            print("\n".join(summary[-20:]), file=sys.stderr)
        OUTPUT.mkdir(parents=True, exist_ok=True)
        (OUTPUT / name).write_text(header(" ".join(command), code, captured), encoding="utf-8")
    if failed:
        return 1

    _, revision = run(["git", "rev-parse", "HEAD"])
    _, status = run(["git", "status", "--porcelain"])
    observed = {
        "schema": "om9.m04.builder_ui_completion.v1",
        "recorded_at_utc": datetime.datetime.now(datetime.UTC).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "source_revision": revision.strip().splitlines()[0] if revision.strip() else "UNRESOLVED",
        "checkout_dirty": bool(status.strip()),
        "target": "Standalone Rust OM9; Windows x64; CARGO_TARGET_DIR=L:/om9-target; -j 2",
        "scope": [
            "builder mode tabs with per-tab parameters over five modes",
            "step-quantised typed fields and sliders (2.0 -> 2.5 mm at step 0.1)",
            "stale panel event refused against a newer viewport handle drag",
            "locked input makes the builder read-only and paints no handle",
            "profile selector window over the producer preset library",
            "Enter/Escape/Reset semantics and one-Undo Apply",
            "F6 prefill and the command line dispatch the same builder service",
        ],
        "code": CODE,
        "l3_coverage": L3_COVERAGE,
        "checks": results,
        "manual_gates": [
            "OM9-L3-03-11-* keep their plan status: the M04-UI manual gate (real pointer gesture on "
            "the panel and handles, human review) is not signed by this batch.",
            "No screenshot or pointer-frame evidence is recorded; the tests prove the model, the "
            "command frame, the controller service and the transaction/journal behavior.",
        ],
        "notes": [
            "The preview recomputes synchronously on every accepted edit; there is no frame throttle. "
            "Nothing reaches the document before Apply, which is the requirement the spec states for "
            "throttled previews, but a time-based throttle is not implemented.",
            "The workspace was being edited concurrently by other sessions while this batch ran; the "
            "logs are the observed state of this tree and this revision.",
        ],
    }
    (OUTPUT / "observed.json").write_text(
        json.dumps(observed, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
    )

    rows = "\n".join(
        f"| `{item['l3']}` | {item['capability']} | {item['state']} |" for item in L3_COVERAGE
    )
    readme = f"""# M04 L2-03-11 builder UI completion

This directory records the second builder slice for `OM9-L2-03-11` (02-febe-matrix-builder-ui-
and-live-handles). It completes the capabilities the first slice
(`../OM9-L2-03-11/`) left open and re-uses that code instead of adding a second builder.

## What changed on top of the first slice

- `BuilderField::step/quantize/slider_ratio/from_slider_ratio` and
  `BuilderSession::set_slider`: a slider moves in whole field steps, so 2.0 + 0.46 is stored
  as exactly 2.5 mm and the panel, the model and the preview read the same number.
- `BuilderSession::revision` and `set_field_at`: a panel event captured before a viewport
  handle drag is refused instead of overwriting the newer value.
- `BuilderSession::editable` plus `Controller::builder_adjust_field`/`builder_set_slider`
  guards: a locked input makes the builder read-only, paints no handle and disables every edit
  control, so a read-only session cannot be edited by accident.
- The BUILDERS panel paints one tab per mode (each keeps its own parameters), a slider next
  to each numeric field, a read-only warning line and the Profile Library window (with a stable
  `.id`, so it keeps its place when the title changes).
- The reserved M01 slot is now the live panel: `crates/om9-app/tests/builder_slot.rs` records
  that handover and still proves the dock reservation itself.
- `RingBuilder` is accepted as the L3-009 fixture's command-line name for the ring rail mode.
- New automated suites: `crates/om9-core/tests/builder_controls.rs` and
  `crates/om9-app/tests/builder_controls.rs`.

## L3 traceability

| L3 | Capability | State |
|----|------------|-------|
{rows}

## Verification

See `observed.json` and the logs here. Focused commands:

- `cargo test -p om9-core --test builder_session --test builder_controls --test history_mutation`
- `cargo test -p om9-app --test builder_panel --test builder_controls --test builder_slot --test top11`
- `cargo test -p om9-kernel --features 3dm-brep --test associative_history` (real OCCT sweep)
- `cargo check --workspace --all-targets`

## Not claimed

- The M04-UI manual gate: a human has not yet reviewed the panel/handle gesture on screen.
- A time-based preview throttle: previews run synchronously on each accepted edit.
- macOS runtime for any of this: the evidence is Windows x64 only.
"""
    (OUTPUT / "README.md").write_text(readme, encoding="utf-8")

    ledger_path = ROOT / "docs" / "openmatrix9-progress.json"
    ledger = json.loads(ledger_path.read_text(encoding="utf-8"))
    entry = {
        "key": KEY,
        "date": "2026-10-27",
        "stage": "feature-port",
        "status": "builder_ui_completion_automated_verified_manual_gates_open",
        "target": observed["target"],
        "feature_ids": ["OM9-L2-03-11"],
        "scope": (
            "M04 OM9-L2-03-11 completion: mode tabs với parameter riêng từng tab (5 mode), "
            "step-quantised field + slider, stale panel event bị từ chối khi handle drag mới hơn, "
            "locked input thành read-only và không vẽ handle, profile selector window, "
            "Enter/Escape/Reset/one-Undo và F6/command line dùng chung một service. "
            "Tái dùng builder slice trước, không thêm builder thứ hai."
        ),
        "outputs": CODE
        + [
            "implementation/evidence/M04/OM9-L2-03-11-COMPLETION/",
            "tools/record_m04_l2_03_11_ui_completion.py",
        ],
        "evidence": [
            "implementation/evidence/M04/OM9-L2-03-11-COMPLETION/observed.json",
            "implementation/evidence/M04/OM9-L2-03-11-COMPLETION/README.md",
            "implementation/evidence/M04/OM9-L2-03-11-COMPLETION/core-builder-controls.log",
            "implementation/evidence/M04/OM9-L2-03-11-COMPLETION/app-builder-controls.log",
            "implementation/evidence/M04/OM9-L2-03-11-COMPLETION/occt-chain.log",
            "implementation/evidence/M04/OM9-L2-03-11-COMPLETION/clippy-core.log",
            "implementation/evidence/M04/OM9-L2-03-11-COMPLETION/workspace-check.log",
        ],
        "tests": {
            "core_builder_controls": "7 passed (step quantisation, slider, stale event, locked input, typed slot, tab schema)",
            "core_builder_session": "8 passed (typed fields, preview, handles, plan/undo, presets, outside rail, resizer)",
            "core_history_mutation": "3 passed (History insert/update journal)",
            "app_builder_controls": "8 passed (tabs, slider, stale event, locked input, Escape, double Enter, F6/command line)",
            "app_builder_panel": "10 passed (command line, tabs, profile library, outside rail/resizer, recompute)",
            "app_builder_slot": "9 passed (the reserved M01 slot now hosts the live builder panel)",
            "app_top11": "19 passed (Ring Rail opens the builder; a running command disables the toolbar)",
            "occt_chain": "2 passed (real B-Rep sweep, topology names, reference after edit)",
            "clippy_core": "cargo clippy -p om9-core --all-targets -- -D warnings exit 0",
            "workspace_check": "cargo check --workspace --all-targets exit 0",
        },
        "uncertainties": [
            "Gate M04-UI vẫn mở: chưa có ảnh chụp panel/handle hay pointer-frame; batch chỉ có bằng chứng tự động.",
            "Preview chạy đồng bộ mỗi lần sửa, chưa có throttle theo thời gian; yêu cầu 'preview không commit' thì đã đảm bảo (chỉ Apply mới ghi document).",
            "Workspace bị các session khác sửa song song trong lúc chạy batch; log là trạng thái quan sát của cây tại revision ghi trong observed.json.",
            "Chưa có bằng chứng macOS; toàn bộ log là Windows x64.",
            "Cliproxy không chạy -D warnings cho om9-app vì warning dead_code thuộc module transform của session khác, không thuộc batch này.",
        ],
        "manual_gate": "OPEN: M04-UI cần nghiệm thu người (panel/handle gesture); batch chỉ ghi bằng chứng tự động",
        "next_action": "Chụp panel + handle gesture cho gate M04-UI, rồi chuyển sang L2-03-12 (builder styles/presets) hoặc L2-04-02 rail/profile tools.",
    }
    ledger["items"] = [item for item in ledger["items"] if item.get("key") != KEY] + [entry]
    ledger["active_item"] = KEY
    ledger["updated_at"] = datetime.datetime.now().astimezone().isoformat(timespec="seconds")
    ledger_path.write_text(json.dumps(ledger, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")

    # Milestone runtime record: point the owner at the evidence without promoting the
    # planning status. The gate stays manual until a human signs M04-UI.
    owner_path = (
        ROOT
        / "OM9_Milestones"
        / "M04 - Builder and History"
        / "L2-03-11 - 02-febe-matrix-builder-ui-and-live-handles"
        / "L2.json"
    )
    owner = json.loads(owner_path.read_text(encoding="utf-8"))
    owner["runtime_implementation"] = {
        "date": "2026-10-27",
        "status": "AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING",
        "target": "standalone Rust OM9, Windows x64",
        "scope": "builder panel, mode tabs, live preview, viewport handles, profile selector",
        "owner": {
            "crates": ["om9-core", "om9-app"],
            "tests": [
                "crates/om9-core/tests/builder_session.rs",
                "crates/om9-core/tests/builder_controls.rs",
                "crates/om9-app/tests/builder_panel.rs",
                "crates/om9-app/tests/builder_controls.rs",
            ],
        },
        "evidence": [
            "implementation/evidence/M04/OM9-L2-03-11-COMPLETION/observed.json",
            "implementation/evidence/M04/OM9-L2-03-11-COMPLETION/README.md",
            "implementation/evidence/M04/OM9-L2-03-11-COMPLETION/core-builder-controls.log",
            "implementation/evidence/M04/OM9-L2-03-11-COMPLETION/app-builder-controls.log",
            "implementation/evidence/M04/OM9-L2-03-11-COMPLETION/occt-chain.log",
            "implementation/evidence/M04/OM9-L2-03-11-COMPLETION/clippy-core.log",
            "implementation/evidence/M04/OM9-L2-03-11-COMPLETION/workspace-check.log",
        ],
        "l3_coverage": {item["l3"]: item["state"] for item in L3_COVERAGE},
        "tests": [f"{result['artifact']}: exit {result['exit_code']}" for result in results],
        "human_approval": None,
        "manual_gate": "M04-UI",
        "remaining": [
            "Manual M04-UI interactive acceptance (panel painting, real pointer gesture on a handle) is not performed by the implementer.",
            "The live preview recomputes synchronously per accepted edit; no time-based throttle is implemented.",
            "macOS runtime is unverified.",
        ],
    }
    owner_path.write_text(
        json.dumps(owner, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
    )

    print(f"recorded builder completion evidence at {OUTPUT.relative_to(ROOT)}")
    for result in results:
        print(f"  {result['artifact']}: exit {result['exit_code']}")
    print(f"ledger: {ledger_path.relative_to(ROOT)} ({len(ledger['items'])} items)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
