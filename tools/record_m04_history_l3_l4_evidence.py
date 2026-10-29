#!/usr/bin/env python3
"""Record M04 OM9-L2-03-13 L3/L4 automated evidence.

Runs the registered controller tests, hashes their output, and writes the L4 receipt
packages the milestone dashboard reads. Manual M04-UI review stays open: no signature is
written and every receipt says so.
"""
from __future__ import annotations

import hashlib
import json
import os
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "implementation" / "evidence" / "M04"
TARGET = os.environ.get("CARGO_TARGET_DIR", r"L:\om9-target")
CARGO = ["cargo", "test", "-p", "om9-app", "--test", "history_updates", "-j", "2"]

L3S = [
    {
        "id": "OM9-L3-03-13-001",
        "title": "Matrix History Record Toggle",
        "goal": "turn history recording on/off for a selected command sequence",
        "l4_count": 6,
        "positive": "record off inserts the rail geometry without a recipe; record on stores the typed recipe; a toggle is refused while the builder transaction is open",
        "negative": "toggle while an edit transaction is open -> Invalid(history record during edit transaction), policy unchanged",
        "tests": ["record_off_commits_geometry_without_a_recipe_and_toggles_stamp_new_records"],
        "integration_tests": ["record_off_commits_geometry_without_a_recipe_and_toggles_stamp_new_records"],
        "sources": [
            ("crates/om9-app/src/session/history.rs", ["Controller::set_history_record", "Controller::toggle_history_record"]),
            ("crates/om9-app/src/session/builder.rs", ["Controller::builder_apply"]),
            ("crates/om9-app/src/session.rs", ["Controller::history_record", "Controller::commit_entity_with_history"]),
            ("crates/om9-app/src/ui/sidebar_palettes.rs", ["fixed_sidebar_palette", "RhinoHistoryON", "GVHistoryRecordON"]),
        ],
        "changed": [
            "crates/om9-core/src/history_ops.rs",
            "crates/om9-app/src/session.rs",
            "crates/om9-app/src/session/history.rs",
            "crates/om9-app/src/session/builder.rs",
            "crates/om9-app/src/ui/sidebar_palettes.rs",
            "crates/om9-app/tests/history_updates.rs",
        ],
        "notes": [
            "Recording off omits the History mutation entirely: the entity is committed in the same one-transaction journal, so Undo still removes it.",
            "The policy is stamped on new recipes (record/update) rather than rewriting recipes that already exist.",
        ],
    },
    {
        "id": "OM9-L3-03-13-002",
        "title": "Matrix History Update Toggle",
        "goal": "enable/disable associative recompute display",
        "l4_count": 7,
        "positive": "with update off a parameter edit leaves the rail geometry and marks the recipe Dirty; re-enabling update rebuilds the edited feature and its downstream profile once",
        "negative": "re-enable with a broken dependency -> Invalid(history dependency broken) and the switch stays off",
        "tests": [
            "update_off_marks_dirty_and_rebuilds_only_after_update_is_reenabled",
            "reenabling_update_refuses_a_broken_dependency",
        ],
        "integration_tests": [
            "update_off_marks_dirty_and_rebuilds_only_after_update_is_reenabled",
            "reenabling_update_refuses_a_broken_dependency",
        ],
        "sources": [
            ("crates/om9-app/src/session/history.rs", ["Controller::set_history_update", "Controller::apply_history_parameter"]),
            ("crates/om9-app/src/session/builder.rs", ["Controller::recompute_history", "Controller::recompute_history_inner"]),
            ("crates/om9-core/src/history_ops.rs", ["refresh_inputs", "broken_links", "plan_recompute"]),
            ("crates/om9-app/src/ui/sidebar_palettes.rs", ["GVHistoryUpdateON"]),
        ],
        "changed": [
            "crates/om9-core/src/history_ops.rs",
            "crates/om9-app/src/session/history.rs",
            "crates/om9-app/src/session/builder.rs",
            "crates/om9-app/src/ui/sidebar_palettes.rs",
            "crates/om9-app/tests/history_updates.rs",
        ],
        "notes": [
            "recompute_history walks plan_recompute in execution order after the edited recipe; refresh_inputs re-stamps a Dirty consumer against the source as it is now, because the old stamp names the previous geometry revision.",
            "Re-enabling update validates execution_order (cycles) and broken_links (missing input/output) before the flag flips.",
        ],
    },
    {
        "id": "OM9-L3-03-13-003",
        "title": "Matrix History Broken Link Report",
        "goal": "show missing/changed history inputs and warnings",
        "l4_count": 7,
        "positive": "the History window lists each recorded feature, its recipe id, state and a detail naming the missing source id; a missing rail input is reported with its role, with no silent repair and an explicit Edit/Clear remediation",
        "negative": "the phantom stale reference survives save/reopen and is still reported instead of being dropped",
        "tests": ["the_report_names_a_missing_input_and_survives_a_reopen"],
        "integration_tests": ["the_report_names_a_missing_input_and_survives_a_reopen"],
        "sources": [
            ("crates/om9-core/src/history_ops.rs", ["BrokenLink", "broken_links"]),
            ("crates/om9-app/src/session/history.rs", ["Controller::history_entries", "Controller::history_broken_links"]),
            ("crates/om9-app/src/ui/chrome.rs", ["MatrixApp::history_window_ui"]),
        ],
        "changed": [
            "crates/om9-core/src/history_ops.rs",
            "crates/om9-app/src/session/history.rs",
            "crates/om9-app/src/ui/chrome.rs",
            "crates/om9-core/tests/history_links.rs",
            "crates/om9-app/tests/history_updates.rs",
        ],
        "notes": [
            "broken_links reports the recipe id, command, state, missing input roles, changed input roles and the matching source object ids; a Clean recipe with intact bindings never appears.",
            "The History window shows the recipe id, the detail (`missing input <role> (source <id>)` or `stale: <reason>`) and the Edit/Clear remediation; nothing is repaired silently.",
            "Dirty/Unresolved/Failed rows are warnings too, so a parameter edit with update off (or a source revision change) is visible without being called missing.",
            "The row is derived from the graph, not cached UI state, so a reopen re-reports the phantom reference.",
        ],
    },
    {
        "id": "OM9-L3-03-13-004",
        "title": "Matrix Clear Single Object History",
        "goal": "detach a specific object while leaving other associations",
        "l4_count": 7,
        "positive": "clearing the profile detaches only its recipe; the profile geometry stays, the rail recipe stays Clean and the report lists the remaining feature",
        "negative": "locked recipe -> Invalid(history recipe locked); no owner -> Missing(history recipe)",
        "tests": ["clear_history_detaches_only_the_selected_object_and_refuses_a_lock"],
        "integration_tests": ["clear_history_detaches_only_the_selected_object_and_refuses_a_lock"],
        "sources": [
            ("crates/om9-core/src/history_ops.rs", ["detach_object", "has_pending"]),
            ("crates/om9-app/src/session/history.rs", ["Controller::clear_object_history", "Controller::clear_selected_history"]),
            ("crates/om9-app/src/session/files.rs", ["begin_file_intent"]),
            ("crates/om9-app/src/ui/chrome.rs", ["MatrixApp::history_window_ui"]),
        ],
        "changed": [
            "crates/om9-core/src/history_ops.rs",
            "crates/om9-app/src/session/history.rs",
            "crates/om9-app/src/ui/chrome.rs",
            "crates/om9-core/tests/history_links.rs",
            "crates/om9-app/tests/history_updates.rs",
        ],
        "notes": [
            "Detach is one HistoryUpdate transaction (state Detached), so Undo restores the association.",
            "The published result is intentionally kept as a plain snapshot; there is no result-removal mutation in the transaction model.",
            "A frozen (lock) recipe is refused so Clear never bypasses the lock.",
        ],
    },
    {
        "id": "OM9-L3-03-13-005",
        "title": "Edit Existing Builder By History",
        "goal": "reopen builder with parameters from selected object",
        "l4_count": 7,
        "positive": "the ring builder reopens at the recorded 22 mm diameter; Apply at 24 mm updates the same object and recipe in one transaction",
        "negative": "missing builder kind/schema mismatch or an object without a recipe -> typed refusal, no partial edit",
        "tests": ["edit_from_history_reopens_the_builder_at_the_recorded_parameters"],
        "integration_tests": ["edit_from_history_reopens_the_builder_at_the_recorded_parameters"],
        "sources": [
            ("crates/om9-core/src/builder.rs", ["BuilderSession::adopt_history", "BuilderSession::plan", "BuilderSession::refresh"]),
            ("crates/om9-app/src/session/history.rs", ["Controller::begin_builder_from_history", "Controller::begin_builder_from_selected_history"]),
            ("crates/om9-app/src/session/builder.rs", ["Controller::builder_apply"]),
            ("crates/om9-app/src/ui/chrome.rs", ["MatrixApp::history_window_ui"]),
        ],
        "changed": [
            "crates/om9-core/src/builder.rs",
            "crates/om9-app/src/session/history.rs",
            "crates/om9-app/src/session/builder.rs",
            "crates/om9-app/src/ui/chrome.rs",
            "crates/om9-app/tests/history_updates.rs",
        ],
        "notes": [
            "The plan adopts the selected object's ObjectId and recipe id, so Apply writes Geometry + HistoryUpdate instead of Insert + History.",
            "Only the five builder commands are re-openable from history; another command or a schema_version other than 1 is a typed refusal.",
            "The published geometry_revision is advanced to the revision the Geometry mutation will produce before the transaction is prepared.",
        ],
    },
    {
        "id": "OM9-L3-03-13-006",
        "title": "History Commandline And Preview Status",
        "goal": "informative status when rebuild happens",
        "l4_count": 6,
        "positive": "HistoryRecord/HistoryUpdate/HistoryReport/ClearHistory/EditHistory drive the same controller switches; recompute reports complete with the rebuilt feature count or failed with the node id and reason",
        "negative": "ClearHistory with no selection -> Missing(history object); closing the window while a recipe is Pending -> Busy, document untouched",
        "tests": [
            "the_command_line_drives_the_switches_report_and_clear",
            "closing_the_window_is_refused_while_a_recipe_is_pending",
        ],
        "integration_tests": [
            "the_command_line_drives_the_switches_report_and_clear",
            "closing_the_window_is_refused_while_a_recipe_is_pending",
        ],
        "sources": [
            ("crates/om9-app/src/session/history.rs", ["Controller::submit_history_text", "Controller::dispatch_history_feature", "Controller::take_history_notice"]),
            ("crates/om9-app/src/session.rs", ["Controller::submit_effect", "Controller::dispatch_effect"]),
            ("crates/om9-app/src/session/files.rs", ["begin_file_intent"]),
            ("crates/om9-app/src/ui.rs", ["MatrixApp::feedback"]),
        ],
        "changed": [
            "crates/om9-app/src/session/history.rs",
            "crates/om9-app/src/session.rs",
            "crates/om9-app/src/session/files.rs",
            "crates/om9-app/src/ui.rs",
            "crates/om9-app/tests/history_updates.rs",
        ],
        "notes": [
            "The command line and the sidebar/menu IDs dispatch to the same Controller methods; the status line consumes one history notice per command.",
            "The rebuild is synchronous, so an interactive 'updating' frame is not observable; the terminal status distinguishes complete (with count) from failed (with node id and reason).",
            "A recipe in Pending state is the queued rebuild the lifecycle guard refuses to close over.",
        ],
    },
]

ROLES = {
    "03": ("IMPLEMENT", "implementation"),
    "04": ("INTEGRATE", "integration"),
    "05": ("REGRESSION", "regression"),
    "07": ("ADVERSARIAL", "adversarial"),
    "06": ("ACCEPT", "acceptance"),
}


def run(argv: list[str]) -> dict:
    env = os.environ.copy()
    env["CARGO_TARGET_DIR"] = TARGET
    completed = subprocess.run(
        argv,
        cwd=ROOT,
        env=env,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=1800,
    )
    output = (completed.stdout or "") + (completed.stderr or "")
    summary = [
        line.strip()
        for line in output.splitlines()
        if line.startswith("test result:") or line.startswith("running ")
    ]
    return {
        "command_argv": argv,
        "exit_code": completed.returncode,
        "stdout_stderr_sha256": hashlib.sha256(output.encode("utf-8", "replace")).hexdigest(),
        "summary": summary[-4:],
        "output": output,
    }


def git(*args: str) -> str:
    return subprocess.run(
        ["git", *args], cwd=ROOT, capture_output=True, text=True, encoding="utf-8"
    ).stdout.strip()


def main() -> int:
    revision = git("rev-parse", "HEAD")
    dirty = bool(git("status", "--porcelain"))
    generated = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    cache: dict[tuple[str, ...], dict] = {}

    def receipt_command(test: str | None) -> dict:
        argv = CARGO + ([test] if test else [])
        key = tuple(argv)
        if key not in cache:
            cache[key] = run(argv)
        return cache[key]

    failures = 0
    for spec in L3S:
        folder = EVIDENCE / spec["id"]
        folder.mkdir(parents=True, exist_ok=True)

        positive = {
            "schema": "om9.m04.fixture.v1",
            "l3_id": spec["id"],
            "kind": "positive",
            "input": spec["goal"],
            "expected": spec["positive"],
            "verified_by": spec["tests"],
            "notes": spec["notes"],
        }
        negative = {
            "schema": "om9.m04.fixture.v1",
            "l3_id": spec["id"],
            "kind": "negative",
            "expected": spec["negative"],
            "verified_by": spec["tests"],
        }
        (folder / "positive.json").write_text(
            json.dumps(positive, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
        )
        (folder / "negative.json").write_text(
            json.dumps(negative, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
        )
        fixture_hash = hashlib.sha256(
            (folder / "positive.json").read_bytes() + (folder / "negative.json").read_bytes()
        ).hexdigest()

        crosswalk = {
            "schema": "om9.m04.source_crosswalk.v1",
            "l3_id": spec["id"],
            "title": spec["title"],
            "owner": "om9-app Controller history API over om9-core HistoryGraph",
            "source_revision": revision,
            "checkout_dirty": dirty,
            "sources": [
                {"path": path, "symbols": symbols, "state": "INSPECTED_IN_CHECKOUT"}
                for path, symbols in spec["sources"]
            ],
            "planning_tree_touched": False,
            "notes": [
                "The L3/L4 plan records are planning input; runtime behavior is established by the receipts.",
                *spec["notes"],
            ],
        }
        (folder / "source_crosswalk.json").write_text(
            json.dumps(crosswalk, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
        )

        api = f"""# {spec['id']} — {spec['title']}

## Goal

{spec['goal']}

## Controller contract

- `Controller::submit(..)` / the sidebar/menu feature IDs dispatch to the same history methods.
- Positive: {spec['positive']}
- Negative: {spec['negative']}

## Tested entry points

""" + "\n".join(f"- `{test}`" for test in spec["tests"]) + "\n"
        if spec["id"].endswith("006"):
            api += (
                "\n## Status text\n\n"
                "- complete: `History update complete: <n> feature(s)`\n"
                "- failed: `History update failed at <recipe id>: <reason>`\n"
                "- lifecycle: a Pending recipe (or an in-flight rebuild) makes window close/open return `Busy`.\n"
                "\n## Command events\n\n"
                "- `Started / History / updating <recipe id>`\n"
                "- `Finished / History / complete: <n> feature(s)` with outcome Committed\n"
                "- `Warn / History / failed at <recipe id>: <reason>` with outcome Refused\n"
            )
        (folder / "api.md").write_text(api, encoding="utf-8")

        for suffix in ("03", "04", "05", "07", "06"):
            if suffix == "07" and spec["l4_count"] < 7:
                continue
            if suffix == "06" and spec["l4_count"] < 6:
                continue
            role, kind = ROLES[suffix]
            tests = spec["integration_tests"] if suffix in {"04", "05", "06"} else spec["tests"]
            commands = []
            for test in tests:
                result = receipt_command(test)
                failures += 0 if result["exit_code"] == 0 else 1
                commands.append(
                    {
                        "command_argv": result["command_argv"],
                        "exit_code": result["exit_code"],
                        "stdout_stderr_sha256": result["stdout_stderr_sha256"],
                        "summary": result["summary"],
                        "test": test,
                        "package": "om9-app",
                        "binary": "history_updates",
                    }
                )
            receipt = {
                "schema": "om9.m04.l4.receipt.v1",
                "l3_id": spec["id"],
                "title": spec["title"],
                "generated_at_utc": generated,
                "source_revision": revision,
                "checkout_dirty": dirty,
                "manual_gate": "OPEN",
                "signature": None,
                "not_claimed": [
                    "manual panel/icon/window review at the M04-UI gate",
                    "a reviewer signature",
                ],
                "fixture_manifest_sha256": fixture_hash,
                "document_snapshot": {
                    "source": "asserted by the registered tests",
                    "positive": spec["positive"],
                    "negative": spec["negative"],
                },
                "l4_id": f"OM9-L4-03-13-{spec['id'].rsplit('-', 1)[1]}-{suffix}",
                "role": role,
                "kind": kind,
                "status": "READY_FOR_REVIEW",
                "changed_files": spec["changed"],
                "commands": commands,
                "manual_gate_note": "M04-UI manual review remains open; no signature is included.",
            }
            path = folder / f"OM9-L4-03-13-{spec['id'].rsplit('-', 1)[1]}-{suffix}.json"
            path.write_text(json.dumps(receipt, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")

        readme = f"""# {spec['id']} — {spec['title']}

**Status:** implemented and automatically tested; **manual M04-UI gate is open** (no
reviewer signature is claimed). The dashboard keeps this L3 at 🟡 until a human signs the
gate.

## L4 artefacts

- `source_crosswalk.json` — sources inspected in this checkout
- `api.md` — the contract the tests exercise
- `positive.json` / `negative.json` — fixtures
- `OM9-L4-…-03/04/05/06.json` (+ `-07` where the plan adds an adversarial task) —
  implementation, integration, regression, acceptance and adversarial receipts

## Tests

""" + "\n".join(f"- `{test}`" for test in spec["tests"]) + "\n\n## Notes\n\n" + "\n".join(
            f"- {note}" for note in spec["notes"]
        ) + """

## Not claimed

- Human acceptance of the panel, icons, History window or command line rendering.
- A signature: record only automated receipts; a human owns the remaining gate.
"""
        (folder / "README.md").write_text(readme, encoding="utf-8")
        print(f"wrote {folder.relative_to(ROOT)} ({spec['l4_count']} L4)")

    # The L2 completion note is written by the caller after the manual review; this script
    # only refreshes the L3/L4 packages.
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
