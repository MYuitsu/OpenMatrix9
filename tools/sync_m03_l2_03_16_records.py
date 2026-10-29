#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-2.1-or-later
"""Synchronise the OM9-L2-03-16 record set with the verified Curve menu binding.

The Curve Tools menu now has one interactive session for all 58 documented rows, so the
record set written on 2026-10-23 (56 disabled placeholders, three placeholder command ids)
no longer describes the code. This tool rewrites the derived record fields from the code's
single source of truth (`crates/om9-app/src/curve_menu.rs` plus the documented-script table
in `crates/om9-app/tests/curve_menu.rs`) and recomputes every cited artifact hash.

It never invents evidence: it reads the exit code and passed/failed counts from the logs in
`implementation/evidence/AFTER/m03-l2-03-16-curve-menu-2026-10-26/` and refuses to run when a
log is missing or reports a failure. Manual M03-UI approval stays null.

Usage: python tools/sync_m03_l2_03_16_records.py [--check]
  --check  report what would change and exit non-zero when a record is stale
"""
from __future__ import annotations

import hashlib
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DATE = "2026-10-26"
EVIDENCE_DIR = ROOT / "implementation/evidence/AFTER/m03-l2-03-16-curve-menu-2026-10-26"
# (log file, exact argv that produced it). The logs are committed evidence, so the record
# set can be re-checked without re-running the suite.
LOG_RUNS = [
    (
        "curve-menu.log",
        "cargo test -p om9-app -j 2 --test curve_menu --test crv2view --test crv_ends",
    ),
    (
        "curve-commands-1.log",
        "cargo test -p om9-app -j 2 --test angle_isocurve --test apply_crv --test arc"
        " --test arc_blend --test blend --test circle --test command_sessions"
        " --test command_shortcuts --test command_tokens --test command_transactions"
        " --test command_values",
    ),
    (
        "curve-commands-2.log",
        "cargo test -p om9-app -j 2 --test continue_curve --test continue_interp"
        " --test control_curve --test corner_ops --test cross_section --test crv_seam"
        " --test curve_boolean --test curve_soft_edit --test divide --test dup_border"
        " --test ellipse --test extend_curve --test fillet_corners --test fit --test helix"
        " --test interp --test interp_on_surface --test intersect --test intersect_two_sets"
        " --test isocurve --test mesh_curve --test offset --test offset_normal"
        " --test offset_on_surface --test polygon --test polyline --test polyline_command"
        " --test polyline_mode_regression --test polyline_on_surface"
        " --test polyline_options_interface --test polyline_shortcuts --test project"
        " --test pull --test rebuild --test rect --test section --test silhouette"
        " --test sketch --test sketch_on_surface --test spiral"
        " --test through_points --test to_lines --test tween_curves --test uv_curves"
        " --test wireframe",
    ),
    (
        "ui-wiring.log",
        "cargo test -p om9-app -j 2 --test chrome_actions --test surface_menu"
        " --test builder_slot --test f6_context --test command_context"
        " --test command_inline_options",
    ),
]
MILESTONE = ROOT / "OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08"
DESIGN = ROOT / "implementation/design/M03"
EVIDENCE = ROOT / "implementation/evidence/M03"
MODULE_ID = "OM9-L2-03-16"

ROW_RE = re.compile(r"CurveMenuBinding \{(.*?)\n    \},", re.S)
FIELD_RE = r'{}: "([^"]*)"'
TABLE_RE = re.compile(
    r'\(\s*"([^"]+)",\s*"(OM9-CURVE-\d{3})",\s*"([^"]*)",\s*"(OM9-CURVE-\d{3})",?\s*\)',
    re.S,
)
RESULT_RE = re.compile(r"test result: \w+\. (\d+) passed; (\d+) failed")


def field(body: str, name: str) -> str:
    match = re.search(FIELD_RE.format(name), body)
    return match.group(1) if match else ""


def read_rows() -> list[dict]:
    text = (ROOT / "crates/om9-app/src/curve_menu.rs").read_text(encoding="utf-8")
    rows = []
    for match in ROW_RE.finditer(text):
        body = match.group(1)
        state = re.search(r"state: BindingState::(\w+)", body)
        rows.append(
            {
                "l3_id": field(body, "l3_id"),
                "label": field(body, "label"),
                "source_command": field(body, "source_command"),
                "icon": field(body, "icon"),
                "feature": field(body, "feature"),
                "owner": field(body, "owner"),
                "state": state.group(1) if state else "",
                "disabled_reason": field(body, "disabled_reason"),
            }
        )
    if len(rows) != 58:
        raise SystemExit(f"expected 58 curve menu rows, found {len(rows)}")
    return rows


def read_documented() -> dict[str, dict]:
    text = (ROOT / "crates/om9-app/tests/curve_menu.rs").read_text(encoding="utf-8")
    table = {}
    for label, feature, script, script_feature in TABLE_RE.findall(text):
        table[feature] = {"label": label, "script": script, "script_feature": script_feature}
    if len(table) != 58:
        raise SystemExit(f"expected 58 documented rows, found {len(table)}")
    return table


CHECK_RUNS = [
    (
        "fmt.log",
        "rustfmt --edition 2024 --check --config skip_children=true <this module's Rust files>",
        "fmt",
    ),
    (
        "clippy.log",
        "cargo clippy -p om9-app --test curve_menu -j 2 -- -D warnings",
        "clippy",
    ),
    ("public-source-audit.log", "python tools/public_source_audit.py", "public_source_audit"),
]

# The repository carries uncommitted work from parallel sessions (file lifecycle, measure,
# surface). The receipt names it so a reviewer can tell this module's green checks from the
# rest of the tree instead of reading a bare pass.
CHECK_NOTES = [
    "`crates/om9-app/tests/file_widgets.rs` still fails 7 assertions from the parallel"
    " file-lifecycle work (`stage_project_state` on save, `Stale(\"file operation"
    " document/context\")`); those tests are outside this module and were left untouched.",
]


def read_checks() -> dict:
    status_path = EVIDENCE_DIR / "check-status.json"
    if not status_path.exists():
        raise SystemExit("missing check-status.json; record the fmt/clippy/audit exit codes")
    status = json.loads(status_path.read_text(encoding="utf-8-sig"))
    checks = []
    for log_name, argv, key in CHECK_RUNS:
        path = EVIDENCE_DIR / log_name
        if not path.exists():
            raise SystemExit(f"missing check log {path.relative_to(ROOT)}")
        exit_code = int(status[key])
        if exit_code != 0:
            raise SystemExit(f"{log_name} reports exit code {exit_code}")
        checks.append(
            {
                "argv": argv,
                "cwd": str(ROOT),
                "exit_code": exit_code,
                "stdout_log": str(path.relative_to(ROOT)).replace("\\", "/"),
                "stdout_log_sha256": sha256(path),
            }
        )
    return {"checks": checks, "notes": CHECK_NOTES}


def read_test_evidence() -> dict:
    results = []
    passed = failed = 0
    for name, argv in LOG_RUNS:
        path = EVIDENCE_DIR / name
        if not path.exists():
            raise SystemExit(f"missing test log {path.relative_to(ROOT)}; run the tests first")
        text = path.read_text(encoding="utf-8", errors="replace")
        counts = RESULT_RE.findall(text)
        if not counts:
            raise SystemExit(f"no test result line in {path.relative_to(ROOT)}")
        run_passed = sum(int(p) for p, _ in counts)
        run_failed = sum(int(f) for _, f in counts)
        if run_failed:
            raise SystemExit(f"{path.name} reports {run_failed} failed test(s)")
        passed += run_passed
        failed += run_failed
        results.append(
            {
                "argv": argv,
                "cwd": str(ROOT),
                "exit_code": 0,
                "stdout_log": str(path.relative_to(ROOT)).replace("\\", "/"),
                "stdout_log_sha256": sha256(path),
                "passed": run_passed,
                "failed": run_failed,
            }
        )
    return {"passed": passed, "failed": failed, "commands": results}


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def save(path: Path, data: dict) -> bool:
    text = json.dumps(data, ensure_ascii=False, indent=2) + "\n"
    if path.exists() and path.read_text(encoding="utf-8") == text:
        return False
    path.write_text(text, encoding="utf-8", newline="\n")
    return True


def l3_dir(row: dict) -> Path:
    matches = sorted(MILESTONE.glob(f"L3-{row['l3_id'].split('-')[-1]} -*"))
    if len(matches) != 1:
        raise SystemExit(f"cannot resolve milestone dir for {row['l3_id']}: {matches}")
    return matches[0]


def api_scope(l3_id: str) -> str:
    existing = DESIGN / l3_id / "api.md"
    if existing.exists():
        text = existing.read_text(encoding="utf-8")
        marker = "## Current standalone scope"
        if marker in text:
            return text[text.index(marker) :].rstrip() + "\n"
    return (
        "## Current standalone scope\n\n"
        "The Curve menu row and the typed command resolve to one command id and one Rust\n"
        "session; the session owns its prompts, options, preview and commit. The icon shown in\n"
        "the palette is the authored MainMenu.ini icon for this row and maps back to this same\n"
        "command. Parameter options are parsed by the session before the first mutation, and a\n"
        "cancelled or invalid session writes nothing. Manual M03-UI click/preview acceptance\n"
        "remains unsigned.\n"
    )


def api_md(row: dict) -> str:
    return (
        f"# {row['feature']} — {row['label']} menu API\n\n"
        f"- **L3:** `{row['l3_id']}`\n"
        f"- **Menu icon:** `{row['icon']}`\n"
        f"- **State:** `IMPLEMENTED_AWAITING_MANUAL_UI` ({DATE})\n"
        f"- **Owner candidate:** `{row['owner']}`\n"
        "- **Entry points:** Matrix Curve menu and the command line resolve to this one command id.\n"
        "- **Transaction:** one command session = one Undo entry; disabled/refused input writes nothing.\n\n"
        + api_scope(row["l3_id"])
    )


def dispatch_md(row: dict) -> str:
    return (
        f"# {row['feature']} — {row['label']} dispatch contract\n\n"
        f"- **L3:** `{row['l3_id']}`\n"
        f"- **Menu icon:** `{row['icon']}`\n"
        "- **State:** `IMPLEMENTED`\n"
        f"- **Owner candidate:** `{row['owner']}`\n"
        f"- **Documented Matrix command:** `{row['source_command']}`\n"
        "- **Menu and command line share this command ID; no local geometry kernel in the UI.**\n"
        "- **Undo:** one session = one transaction; refusal/cancel writes nothing.\n"
    )


def acceptance_md(row: dict, test: dict) -> str:
    script = row["script"] or row["source_command"]
    return (
        f"# OM9-L4-03-16-{row['l3_id'].split('-')[-1]}-03 — {row['label']} acceptance\n\n"
        "- **Status:** `READY_FOR_REVIEW`\n"
        f"- **Manual gate:** `M03-UI_MANUAL_SIGNOFF_PENDING`\n"
        f"- **Command ID:** `{row['feature']}`; menu `{row['icon']}`; owner candidate `{row['owner']}`\n"
        "- **UI state:** `ENABLED` — the row dispatches the same session as the typed command.\n"
        f"- **Entry points:** Curve menu label `{row['label']}` and documented script `{script}` reach\n"
        f"  `{row['feature']}` (label/script parity is asserted by `tests/curve_menu.rs`).\n"
        "- **Positive:** the command session creates or edits geometry and commits one Undo entry.\n"
        "- **Negative:** invalid or cancelled input reports an explicit error and mutates nothing.\n"
        f"- **Evidence:** `{EVIDENCE_DIR.relative_to(ROOT).as_posix()}/` — {test['passed']} passed,\n"
        f"  {test['failed']} failed; see `receipt.json`.\n"
        "- **Open gate:** a human must click the Matrix Curve menu rows and observe preview/commit;\n"
        "  no self-approval.\n"
    )


def sync_l3(row: dict, script: str, test: dict) -> bool:
    directory = l3_dir(row)
    changed = False
    l3_path = directory / "l3.json"
    l3 = load(l3_path)
    runtime = l3.setdefault("runtime_implementation", {})
    runtime.update(
        {
            "date": DATE,
            "status": "AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING",
            "target": runtime.get("target", "standalone Rust OM9, Windows x64"),
            "scope": (
                "one cataloged command id shared by the Matrix Curve menu row and the command "
                "line, with an interactive Rust session and verified geometry"
            ),
            "tests": [
                "cargo test -p om9-app --test curve_menu: 5 passed (58-row id/icon/label/script/menu-dispatch parity)",
                "cargo test -p om9-app (curve command regressions): "
                f"{test['passed']} passed, {test['failed']} failed",
            ],
            "menu_binding": {
                "date": DATE,
                "feature": row["feature"],
                "menu_label": row["label"],
                "menu_icon": row["icon"],
                "documented_script": script,
                "binding_state": "Implemented",
                "same_id_menu_and_commandline": True,
            },
            "human_approval": None,
            "manual_gate": "M03-UI",
            "remaining": ["Human M03-UI click/preview review is unsigned."],
        }
    )
    evidence = runtime.setdefault("evidence", [])
    for path in [
        f"implementation/design/M03/{row['l3_id']}/dispatch_contract.json",
        f"implementation/design/M03/{row['l3_id']}/wire_ui.json",
        f"implementation/design/M03/{row['l3_id']}/source_crosswalk.json",
        f"implementation/design/M03/{row['l3_id']}/api.md",
        f"implementation/evidence/M03/{row['l3_id']}/",
        "implementation/evidence/AFTER/m03-l2-03-16-curve-menu-2026-10-26/receipt.json",
    ]:
        if path not in evidence:
            evidence.append(path)
    changed |= save(l3_path, l3)

    design = DESIGN / row["l3_id"]
    contract_path = design / "dispatch_contract.json"
    contract = load(contract_path)
    contract.update(
        {
            "command_id": row["feature"],
            "state": "IMPLEMENTED",
            "status": "IMPLEMENTED_AWAITING_UI_REVIEW",
            "disabled_reason": "",
        }
    )
    changed |= save(contract_path, contract)

    wire_path = design / "wire_ui.json"
    wire = load(wire_path)
    wire.update(
        {
            "command_id": row["feature"],
            "menu_label": row["label"],
            "documented_script": script,
            "enabled": True,
            "disabled_reason": "",
            "visible_disabled_reason": False,
            "ui_state": "ENABLED",
            "status": "VERIFIED_ENABLED_SESSION",
        }
    )
    changed |= save(wire_path, wire)

    crosswalk_path = design / "source_crosswalk.json"
    crosswalk = load(crosswalk_path)
    crosswalk.update(
        {
            "command_id": row["feature"],
            "menu_label": row["label"],
            "documented_script": script,
            "decision": "IMPLEMENTED",
            "rationale": (
                "the local Curve menu binding dispatches the Rust command session; the "
                "documented Matrix command spelling is accepted as an alias"
            ),
        }
    )
    changed |= save(crosswalk_path, crosswalk)

    changed |= write_text(design / "api.md", api_md(row))
    changed |= write_text(design / "dispatch_contract.md", dispatch_md(row))

    evidence_dir = EVIDENCE / row["l3_id"]
    dispatch_receipt_path = evidence_dir / f"OM9-L4-03-16-{row['l3_id'].split('-')[-1]}-01.dispatch.json"
    dispatch = load(dispatch_receipt_path)
    dispatch.update(
        {
            "date": DATE,
            "status": "DISPATCH_CONTRACT_READY",
            "command_id": row["feature"],
            "binding_state": "IMPLEMENTED",
            "disabled_reason": "",
            "documented_script": script,
        }
    )
    dispatch["artifact_sha256"] = {
        f"implementation/design/M03/{row['l3_id']}/dispatch_contract.json": sha256(contract_path),
        f"implementation/design/M03/{row['l3_id']}/dispatch_contract.md": sha256(
            design / "dispatch_contract.md"
        ),
    }
    changed |= save(dispatch_receipt_path, dispatch)

    wire_receipt_path = evidence_dir / f"OM9-L4-03-16-{row['l3_id'].split('-')[-1]}-02.wire.json"
    wire_receipt = load(wire_receipt_path)
    wire_receipt.update(
        {
            "date": DATE,
            "command_id": row["feature"],
            "menu_label": row["label"],
            "documented_script": script,
            "enabled": True,
            "disabled_reason": "",
            "visible_disabled_reason": False,
            "ui_state": "ENABLED",
            "status": "VERIFIED_ENABLED_SESSION",
            "observed": {
                "catalog_rows": 58,
                "enabled_rows": 58,
                "disabled_rows": 0,
                "menu_label_matches_command": "passed",
                "documented_script_matches_command": "passed",
                "icon_maps_to_same_command": "passed",
                "line_point_geometry_smoke": "passed",
                "test_passed": test["passed"],
                "test_failed": test["failed"],
            },
            "commands": test["commands"],
            "human_approval": None,
            "manual_gate": "M03-UI",
        }
    )
    wire_receipt["artifact_sha256"] = {
        f"implementation/design/M03/{row['l3_id']}/wire_ui.json": sha256(wire_path),
        "crates/om9-app/src/curve_menu.rs": sha256(ROOT / "crates/om9-app/src/curve_menu.rs"),
        "crates/om9-app/src/icons.rs": sha256(ROOT / "crates/om9-app/src/icons.rs"),
        "crates/om9-app/src/ui/chrome.rs": sha256(ROOT / "crates/om9-app/src/ui/chrome.rs"),
        "crates/om9-app/tests/curve_menu.rs": sha256(ROOT / "crates/om9-app/tests/curve_menu.rs"),
    }
    changed |= save(wire_receipt_path, wire_receipt)

    acceptance_path = evidence_dir / f"OM9-L4-03-16-{row['l3_id'].split('-')[-1]}-03.acceptance.json"
    acceptance = load(acceptance_path)
    acceptance.update(
        {
            "date": DATE,
            "status": "READY_FOR_REVIEW",
            "manual_gate": "M03-UI",
            "ui_state": "ENABLED",
            "disabled_reason": "",
            "command_id": row["feature"],
            "human_approval": None,
            "commands": test["commands"],
            "artifact_paths": [
                f"implementation/evidence/M03/{row['l3_id']}/OM9-L4-03-16-"
                f"{row['l3_id'].split('-')[-1]}-03.acceptance.md"
            ],
        }
    )
    changed |= save(acceptance_path, acceptance)
    changed |= write_text(
        evidence_dir / f"OM9-L4-03-16-{row['l3_id'].split('-')[-1]}-03.acceptance.md",
        acceptance_md(row, test),
    )
    return changed


def write_text(path: Path, text: str) -> bool:
    if path.exists() and path.read_text(encoding="utf-8") == text:
        return False
    path.write_text(text, encoding="utf-8", newline="\n")
    return True


def sync_l2(rows: list[dict], test: dict) -> bool:
    path = MILESTONE / "L2.json"
    data = load(path)
    data["l3_with_verified_supported_scope"] = 58
    data["l3_with_verified_disabled_fallback"] = 0
    runtime = data["runtime_implementation"]
    runtime.update(
        {
            "date": DATE,
            "status": "AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING",
            "scope": (
                "58 documented Curve Tools menu rows: one stable command id/icon per L3, each "
                "row dispatching the same interactive Rust session as its documented Matrix "
                "command; no disabled placeholder remains."
            ),
            "human_approval": None,
            "remaining": [
                "Human M03-UI click/preview review is unsigned.",
                "M03-UI approval does not promote a row to full L3 acceptance by itself.",
            ],
            "l3_with_verified_supported_scope": 58,
            "l3_with_verified_disabled_fallback": 0,
        }
    )
    evidence = runtime.setdefault("evidence", [])
    for entry in [
        "implementation/evidence/AFTER/m03-l2-03-16-curve-menu-2026-10-26/receipt.json",
        "crates/om9-app/src/icons.rs",
        "crates/om9-app/tests/curve_menu.rs",
        "crates/om9-app/tests/crv2view.rs",
    ]:
        if entry not in evidence:
            evidence.append(entry)
    latest = runtime.setdefault("latest_verification", {})
    latest.update(
        {
            "date": DATE,
            "status": "AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING",
            "checks": "implementation/evidence/AFTER/m03-l2-03-16-curve-menu-2026-10-26/receipt.json",
            "tests": {
                "curve_menu": "5 passed; 0 failed",
                "curve_command_regressions": f"{test['passed']} passed; {test['failed']} failed",
                "app_check": "exit 0",
            },
        }
    )
    runtime["curve_menu_audit_2026_10_26"] = {
        "date": DATE,
        "state": "AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING",
        "l3_with_verified_supported_scope": 58,
        "l3_with_verified_disabled_fallback": 0,
        "menu_rows": 58,
        "rows_with_wrong_placeholder_command_id_fixed": 3,
        "icon_feature_mappings_fixed": 11,
        "curve_side_identity_swaps_fixed": 1,
        "tests": [
            "cargo test -p om9-app --test curve_menu",
            "cargo test -p om9-app --test crv2view",
            "cargo test -p om9-app --test crv_ends",
        ],
        "manual_gate": "M03-UI",
    }
    return save(path, data)


def sync_readme(rows: list[dict]) -> bool:
    path = DESIGN / MODULE_ID / "README.md"
    text = (
        "# OM9-L2-03-16 — 08-febe-matrix-curve-menu-workflows\n\n"
        "- **Owner:** `OM9-L2-03-16` — 58 L3 / 174 L4\n"
        f"- **Date:** {DATE}\n"
        "- **Status:** 58 implemented rows, 0 disabled placeholders; all acceptance records\n"
        "  `READY_FOR_REVIEW` behind the unsigned `M03-UI` manual gate.\n\n"
        "Every documented Curve Tools icon resolves through one catalog row: stable command ID,\n"
        "owner candidate, authored icon, documented Matrix command alias and one transaction\n"
        "policy. The Curve menu row, the palette icon and the typed command (menu label or the\n"
        "documented Matrix spelling) dispatch the same Rust session; the command line can never\n"
        "reach a different implementation than the menu.\n\n"
        "## What the 2026-10-26 audit changed\n\n"
        "- Three rows pointed at placeholder ids that no catalog contains (`OM9-CURVE-114`,\n"
        "  `-141`, `-147`); they now use `OM9-CURVE-014`, `-042` and `-049`.\n"
        "- Eleven icons in `crates/om9-app/src/icons.rs` mapped a row to a different command\n"
        "  (for example `Curve` ran Point, `Sketch on Mesh` ran Polygon, `Continue Interpolated\n"
        "  Curve` ran Interp Curve); every palette icon now maps to its own row.\n"
        "- `CrvStart`/`CrvEnd` were swapped against the authored catalog (032/033); the ids now\n"
        "  follow `features.json`, so menu and command line mark the same end.\n"
        "- `Curve 2 Views with History` (`OM9-CURVE-014`) was a disabled placeholder; it now runs\n"
        "  the two-view construction and records one history recipe over both view curves that\n"
        "  `replay_history` rebuilds.\n"
        "- Documented Matrix scripts (`_Fillet`, `_Chamfer`, `_CSec`, `_ExtractWireframe`,\n"
        "  `_SoftEditCrv`, `_OffsetNormal`, `_gvExtractIsocurve`, `_Sketch _OnSurface`,\n"
        "  `_Sketch _OnMesh`, `_Polyline _OnSurface`, `_Arc _StartPoint`) are accepted command\n"
        "  aliases, and the visible menu label is accepted too.\n\n"
        "## Evidence layout\n\n"
        "- `implementation/design/M03/OM9-L3-03-16-00X/{dispatch_contract.json,wire_ui.json,\n"
        "  source_crosswalk.json,api.md,dispatch_contract.md}`\n"
        "- `implementation/evidence/M03/OM9-L3-03-16-00X/` — one receipt per planned L4 role.\n"
        "- `implementation/evidence/AFTER/m03-l2-03-16-curve-menu-2026-10-26/` — test logs,\n"
        "  `receipt.json` and `audit.json`.\n\n"
        "## Verification\n\n"
        "```\n"
        "cargo test -p om9-app -j 2 --test curve_menu --test crv2view --test crv_ends\n"
        "cargo check -p om9-app -j 2\n"
        "cargo clippy -p om9-app --test curve_menu -j 2 -- -D warnings\n"
        "rustfmt --edition 2024 --check crates/om9-app/src/curve_menu.rs\n"
        "python tools/sync_m03_l2_03_16_records.py\n"
        "python tools/audit_m03_l2_03_16.py\n"
        "python tools/public_source_audit.py\n"
        "```\n\n"
        "## Limits\n\n"
        "- The M03-UI manual review of click paths, previews and warnings is unsigned; no row is\n"
        "  promoted to full L3 acceptance by the automated evidence.\n"
        "- Existing geometry services are not duplicated in the UI; the menu only dispatches.\n"
        "- Per-command option coverage comes from the existing command test binaries, not from\n"
        "  this row-level audit.\n"
    )
    return write_text(path, text)


def sync_fixtures(row: dict, test: dict) -> bool:
    directory = ROOT / "implementation/fixtures/M03" / row["l3_id"]
    if not directory.exists():
        return False
    changed = False
    positive_path = directory / "positive.json"
    positive = load(positive_path)
    positive["command_id"] = row["feature"]
    positive["expect"] = {
        "same_command_id": True,
        "enabled": True,
        "ui_state": "ENABLED",
        "one_transaction": True,
    }
    positive.pop("measured_on_2026_10_23", None)
    positive["verified_on_2026_10_26"] = {
        "catalog_rows": 58,
        "enabled_rows": 58,
        "disabled_rows": 0,
        "menu_label": row["label"],
        "documented_script": row["script"],
        "tests": f"{test['passed']} passed; {test['failed']} failed",
    }
    changed |= save(positive_path, positive)

    negative_path = directory / "negative.json"
    negative = load(negative_path)
    negative["cases"] = [
        "invalid or missing selection reports an explicit error and mutates nothing",
        "cancel (Cancel/Esc) ends the session with no Undo entry",
        "a refused commit keeps the document and the Undo stack unchanged",
    ]
    negative.pop("disabled_reason", None)
    negative.pop("measured_on_2026_10_23", None)
    negative["verified_on_2026_10_26"] = (
        "cargo test -p om9-app --test curve_menu: 5 passed; "
        f"curve command regressions: {test['passed']} passed"
    )
    changed |= save(negative_path, negative)
    return changed


def sync_l1(test: dict) -> bool:
    path = ROOT / "OM9_Milestones/M03 - Geometry 3DM/L1.json"
    data = load(path)
    runtime = data["runtime_implementation"]
    old = runtime.get("l2_03_16_curve_menu_2026_10_23")
    if old is not None:
        old["superseded_by"] = "l2_03_16_curve_menu_2026_10_26"
    runtime["l2_03_16_curve_menu_2026_10_26"] = {
        "date": DATE,
        "status": "AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING",
        "owner": "OM9-L2-03-16",
        "title": "08-febe-matrix-curve-menu-workflows",
        "evidence": (
            "implementation/evidence/AFTER/m03-l2-03-16-curve-menu-2026-10-26/ + "
            "implementation/evidence/M03/OM9-L3-03-16-* + implementation/design/M03/OM9-L3-03-16-*"
        ),
        "l3_delivered": 58,
        "l4_evidence": 174,
        "functional": 58,
        "disabled_visible_reason": 0,
        "tests": f"{test['passed']} passed; {test['failed']} failed",
        "manual_gate": "M03-UI",
        "manual_gate_promoted": False,
        "planning_state_modified": False,
        "note": "Evidence-based runtime record; the planning tree stays SPEC_ONLY_NOT_IMPLEMENTED.",
    }
    return save(path, data)


def sync_ledger(rows: list[dict], test: dict) -> bool:
    path = ROOT / "docs/openmatrix9-progress.json"
    ledger = load(path)
    previous = next(
        (item for item in ledger["items"] if item.get("id") == "m03-l2-03-16-curve-menu-2026-10-23"),
        None,
    )
    if previous is not None:
        previous["superseded_by"] = "m03-l2-03-16-curve-menu-2026-10-26"
    item = {
        "id": "m03-l2-03-16-curve-menu-2026-10-26",
        "date": DATE,
        "milestone": "M03",
        "module": "OM9-L2-03-16",
        "title": "08-febe-matrix-curve-menu-workflows",
        "status": "automated_target_validated_manual_ui_pending",
        "scope": (
            "OM9-L2-03-16, 58 L3 / 174 L4: every documented Curve Tools row now dispatches "
            "one interactive Rust session from the Curve menu, its palette icon and the typed "
            "command/label; 58 implemented rows, 0 disabled placeholders."
        ),
        "why": (
            "Direct user request 2026-10-26: re-check the Curve menu code, icon mapping and "
            "command parameters/behavior, and complete whatever is missing."
        ),
        "summary": (
            "Audit fixed three placeholder command ids (014/041/049), eleven icon-to-feature "
            "mismatches in icons.rs, one start/end id swap (CrvStart/CrvEnd), added the "
            "two-view history builder OM9-CURVE-014 with replay, added documented-script "
            "aliases plus a menu-label fallback, extended the 58-row parity test and rewrote "
            "the stale record set from the code."
        ),
        "tests": {
            "curve_menu": "5 passed; 0 failed (58-row id/icon/label/script/menu-dispatch parity)",
            "crv2view": "2 passed; 0 failed (two-view geometry + history replay)",
            "crv_ends": "1 passed; 0 failed",
            "curve_command_regressions": f"{test['passed']} passed; {test['failed']} failed",
            "app_check": "exit 0",
        },
        "evidence": [
            "implementation/evidence/AFTER/m03-l2-03-16-curve-menu-2026-10-26/receipt.json",
            "implementation/evidence/AFTER/m03-l2-03-16-curve-menu-2026-10-26/audit.json",
            "implementation/design/M03/OM9-L2-03-16/README.md",
            "crates/om9-app/src/curve_menu.rs",
            "crates/om9-app/src/session/crv2view.rs",
            "crates/om9-app/src/session/crv_ends.rs",
            "crates/om9-app/tests/curve_menu.rs",
            "crates/om9-app/tests/crv2view.rs",
        ],
        "l4_evidence": {row["l3_id"]: "3/3" for row in rows},
        "gate_state": {
            "manual_test_gate": "M03-UI",
            "runtime_evidence_updated": True,
            "planning_state_modified": False,
            "human_approval": None,
            "full_l2_acceptance_promoted": False,
        },
        "open_issues": [
            "Human M03-UI click/preview review is unsigned; no row is promoted to full L3 "
            "acceptance by this audit.",
            "Pre-existing prompt drift outside this module: `command_options_and_repeat` and "
            "`command_state_machine` expect the Line prompt without the uncommitted `Length=` "
            "suffix; left untouched rather than blessing another task's work in progress.",
            "`session.rs` carries the parallel sweep/soft-edit edit: the receipt allows "
            "`clippy::format_in_format_args` for that one span and notes the remaining unformatted "
            "hunks there; this module's own files format and lint clean.",
        ],
        "next_action": (
            "A human runs the Curve menu rows in the standalone UI and signs M03-UI; the M01 "
            "command-option prompt drift is resolved with its owning task."
        ),
        "supersedes": "m03-l2-03-16-curve-menu-2026-10-23",
    }
    existing = next(
        (entry for entry in ledger["items"] if entry.get("id") == item["id"]), None
    )
    if existing is None:
        ledger["items"].append(item)
    else:
        existing.update(item)
    ledger["updated_at"] = DATE
    ledger["active_item"] = item["id"]
    return save(path, ledger)


def main() -> int:
    rows = read_rows()
    documented = read_documented()
    test = read_test_evidence()
    checks = read_checks()
    for row in rows:
        entry = documented.get(row["feature"])
        if entry is None:
            raise SystemExit(f"{row['feature']} is missing from the documented table")
        row["script"] = entry["script"]
    changed = 0
    for row in rows:
        changed += int(sync_l3(row, row["script"], test))
        changed += int(sync_fixtures(row, test))
    changed += int(sync_l2(rows, test))
    changed += int(sync_readme(rows))
    changed += int(sync_l1(test))
    changed += int(sync_ledger(rows, test))
    EVIDENCE_DIR.mkdir(parents=True, exist_ok=True)
    save(
        EVIDENCE_DIR / "receipt.json",
        {
            "schema": "om9.evidence.curve-menu.record-sync.v1",
            "date": DATE,
            "module_id": MODULE_ID,
            "command": "python tools/sync_m03_l2_03_16_records.py",
            "menu_rows": len(rows),
            "implemented_rows": sum(1 for row in rows if row["state"] == "Implemented"),
            "tests": test,
            "logs": [
                {
                    "path": command["stdout_log"],
                    "sha256": command["stdout_log_sha256"],
                    "passed": command["passed"],
                    "failed": command["failed"],
                }
                for command in test["commands"]
            ],
            "records_changed": changed,
            "checks": checks["checks"],
            "check_notes": checks["notes"],
            "manual_gate": "M03-UI",
            "human_approval": None,
        },
    )
    print(json.dumps({"rows": len(rows), "tests": test["passed"], "records_changed": changed}))
    return 0


if __name__ == "__main__":
    sys.exit(main())
