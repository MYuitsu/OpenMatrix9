#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-2.1-or-later
"""Audit the OM9-L2-03-16 Curve menu binding end to end.

Checks the chain the user asked about — code, icon, parameters/behavior, records — without
trusting any single catalog:

1. `crates/om9-app/src/curve_menu.rs` has 58 implemented rows with unique ids, features,
   labels and icons.
2. Icon order matches the authored `Resources/menu/MainMenu.ini` `[Menu7]` list, every
   authored SVG exists, and `Resources/menu/icons.ini` marks it `status = resolved`.
3. `menu_catalog::CURVE`, `crates/om9-app/src/icons.rs` and `crates/om9-app/assets/features.json`
   name the same icon, the same command id and the same command.
4. `crates/om9-app/tests/curve_menu.rs` documents a menu label and a Matrix script for
   every row, and `session.rs` accepts both.
5. The design/evidence record set describes this state and every cited artifact hash
   matches the file on disk.
6. `L2.json` reports 58 supported rows and no disabled fallback.

Exit code 0 only when `integrity: 0 problem(s)`.

Usage: python tools/audit_m03_l2_03_16.py
"""
from __future__ import annotations

import hashlib
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DATE = "2026-10-26"
MILESTONE = ROOT / "OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08"
DESIGN = ROOT / "implementation/design/M03"
EVIDENCE = ROOT / "implementation/evidence/M03"
AFTER = ROOT / "implementation/evidence/AFTER/m03-l2-03-16-curve-menu-2026-10-26"
RECEIPT = AFTER / "audit.json"
EXPECTED_ROWS = 58

problems: list[str] = []


def problem(message: str) -> None:
    problems.append(message)


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def rows() -> list[dict]:
    body = text(ROOT / "crates/om9-app/src/curve_menu.rs")
    found = []
    for match in re.finditer(r"CurveMenuBinding \{(.*?)\n    \},", body, re.S):
        block = match.group(1)

        def field(name: str) -> str:
            hit = re.search(rf'{name}: "([^"]*)"', block)
            return hit.group(1) if hit else ""

        found.append({key: field(key) for key in (
            "l3_id", "label", "source_command", "icon", "feature", "owner",
            "disabled_reason")})
        state = re.search(r"state: BindingState::(\w+)", block)
        found[-1]["state"] = state.group(1) if state else ""
    return found


def documented() -> dict[str, dict]:
    body = text(ROOT / "crates/om9-app/tests/curve_menu.rs")
    table = {}
    for label, feature, script, script_feature in re.findall(
        r'\(\s*"([^"]+)",\s*"(OM9-CURVE-\d{3})",\s*"([^"]*)",\s*"(OM9-CURVE-\d{3})",?\s*\)',
        body,
        re.S,
    ):
        table[feature] = {"label": label, "script": script, "script_feature": script_feature}
    return table


def check_catalog(found: list[dict]) -> None:
    if len(found) != EXPECTED_ROWS:
        problem(f"curve_menu.rs has {len(found)} rows, expected {EXPECTED_ROWS}")
        return
    for index, row in enumerate(found):
        expected_l3 = f"OM9-L3-03-16-{index + 1:03d}"
        if row["l3_id"] != expected_l3:
            problem(f"row {index + 1}: {row['l3_id']} should be {expected_l3}")
        if row["state"] != "Implemented":
            problem(f"{row['l3_id']}: state {row['state']!r} is not Implemented")
        if row["disabled_reason"]:
            problem(f"{row['l3_id']}: implemented row keeps a disabled reason")
        for key in ("label", "source_command", "icon", "feature", "owner"):
            if not row[key]:
                problem(f"{row['l3_id']}: empty {key}")
    for key in ("l3_id", "feature", "label", "icon"):
        values = [row[key].lower() for row in found]
        if len(set(values)) != EXPECTED_ROWS:
            duplicates = sorted({v for v in values if values.count(v) > 1})
            problem(f"duplicate {key}: {duplicates}")


def check_icon_order(found: list[dict]) -> None:
    menu = text(ROOT / "Resources/menu/MainMenu.ini")
    block = re.search(r"\[Menu7\]\n(.*?)(?:\n\[|\Z)", menu, re.S)
    if not block:
        problem("MainMenu.ini has no [Menu7] Curve menu")
        return
    count = re.search(r"IconCount=(\d+)", block.group(1))
    if not count or int(count.group(1)) != EXPECTED_ROWS:
        problem("MainMenu.ini [Menu7] IconCount is not 58")
    icons = re.findall(r"Icon\d+=(.+)", block.group(1))
    icons = [icon.strip() for icon in icons]
    if icons != [row["icon"] for row in found]:
        problem("MainMenu.ini [Menu7] icon order differs from curve_menu.rs")
    catalog = text(ROOT / "crates/om9-app/src/menu_catalog.rs")
    match = re.search(r"pub const CURVE: &\[&str\] = &\[(.*?)\];", catalog, re.S)
    if not match:
        problem("menu_catalog.rs has no CURVE palette")
    else:
        names = re.findall(r'"([^"]+)"', match.group(1))
        if names != [row["icon"] for row in found]:
            problem("menu_catalog::CURVE order differs from curve_menu.rs")


def check_icons(found: list[dict]) -> None:
    icons_ini = text(ROOT / "Resources/menu/icons.ini")
    palette = text(ROOT / "crates/om9-app/src/icons.rs")
    entries = {}
    for match in re.finditer(
        r'icon!\(\s*"([^"]*)",\s*"([^"]*)",\s*"([^"]*)",\s*"([^"]*)"\s*,?\s*\)', palette
    ):
        name, key, feature, source = match.groups()
        entries[key] = (name, feature, source)
    for row in found:
        icon = row["icon"]
        svg = ROOT / "Resources/icons/curve-minimal" / f"{icon}.svg"
        if not svg.exists():
            problem(f"{row['l3_id']}: missing authored icon {svg.relative_to(ROOT)}")
        section = re.search(rf"\[{re.escape(icon)}\]\n(.*?)(?:\n\[|\Z)", icons_ini, re.S)
        if not section:
            problem(f"{row['l3_id']}: icons.ini has no [{icon}] entry")
        else:
            if "status = resolved" not in section.group(1):
                problem(f"{row['l3_id']}: icons.ini [{icon}] is not resolved")
            if f"icons/curve-minimal/{icon}.svg" not in section.group(1):
                problem(f"{row['l3_id']}: icons.ini [{icon}] points at another image")
        entry = entries.get(icon)
        if entry is None:
            problem(f"{row['l3_id']}: icons.rs palette has no {icon}")
            continue
        name, feature, source = entry
        if feature != row["feature"]:
            problem(
                f"{row['l3_id']}: icons.rs maps {icon} to {feature or '(empty)'}, "
                f"menu row uses {row['feature']}"
            )
        if not source.endswith(f"/{icon}.svg"):
            problem(f"{row['l3_id']}: icons.rs {icon} embeds {source}")
        if not name:
            problem(f"{row['l3_id']}: icons.rs {icon} has no UI name")


def check_features(found: list[dict]) -> None:
    features = json.loads(text(ROOT / "crates/om9-app/assets/features.json"))
    by_id = {entry["id"]: entry for entry in features}
    for row in found:
        entry = by_id.get(row["feature"])
        if entry is None:
            problem(f"{row['l3_id']}: features.json has no {row['feature']}")
            continue
        if not entry.get("name"):
            problem(f"{row['l3_id']}: features.json {row['feature']} has no name")
        elif entry["name"].strip().lower() != row["label"].strip().lower():
            problem(
                f"{row['l3_id']}: features.json {row['feature']} is {entry['name']!r}, "
                f"menu row is {row['label']!r}"
            )
        if not entry.get("module"):
            problem(f"{row['l3_id']}: features.json {row['feature']} has no module")


def check_session_aliases(found: list[dict], docs: dict[str, dict]) -> None:
    session = text(ROOT / "crates/om9-app/src/session.rs")
    lowered = session.lower()
    for row in found:
        entry = docs.get(row["feature"])
        if entry is None:
            problem(f"{row['l3_id']}: tests/curve_menu.rs documents no row for {row['feature']}")
            continue
        if entry["label"] != row["label"]:
            problem(
                f"{row['l3_id']}: documented label {entry['label']!r} != "
                f"menu label {row['label']!r}"
            )
        if entry["script"].lower() not in lowered:
            problem(
                f"{row['l3_id']}: session.rs never mentions documented script "
                f"{entry['script']!r}"
            )
        if row["feature"] not in session and not any(
            row["feature"] in text(path)
            for path in (ROOT / "crates/om9-app/src/session").glob("*.rs")
        ):
            problem(f"{row['l3_id']}: {row['feature']} is not referenced by the session")
    expected = {row["feature"] for row in found}
    if set(docs) != expected:
        problem("tests/curve_menu.rs documented table does not cover the 58 rows")


def check_records(found: list[dict]) -> None:
    for row in found:
        l3_id = row["l3_id"]
        directory = sorted(MILESTONE.glob(f"L3-{l3_id.split('-')[-1]} -*"))
        if len(directory) != 1:
            problem(f"{l3_id}: cannot resolve the milestone L3 directory")
            continue
        l3 = json.loads(text(directory[0] / "l3.json"))
        runtime = l3.get("runtime_implementation") or {}
        binding = runtime.get("menu_binding") or {}
        if binding.get("feature") != row["feature"]:
            problem(f"{l3_id}: l3.json menu_binding.feature is stale")
        if binding.get("menu_icon") != row["icon"]:
            problem(f"{l3_id}: l3.json menu_binding.menu_icon is stale")
        if binding.get("same_id_menu_and_commandline") is not True:
            problem(f"{l3_id}: l3.json does not claim one shared command id")
        if runtime.get("human_approval") is not None:
            problem(f"{l3_id}: l3.json runtime human_approval must stay null")
        if runtime.get("manual_gate") != "M03-UI":
            problem(f"{l3_id}: l3.json runtime manual gate is not M03-UI")

        design = DESIGN / l3_id
        contract = json.loads(text(design / "dispatch_contract.json"))
        if contract.get("command_id") != row["feature"]:
            problem(f"{l3_id}: dispatch_contract.json command_id is stale")
        if contract.get("state") != "IMPLEMENTED":
            problem(f"{l3_id}: dispatch_contract.json state is not IMPLEMENTED")
        wire = json.loads(text(design / "wire_ui.json"))
        if wire.get("command_id") != row["feature"] or wire.get("enabled") is not True:
            problem(f"{l3_id}: wire_ui.json does not describe the enabled binding")
        crosswalk = json.loads(text(design / "source_crosswalk.json"))
        if crosswalk.get("command_id") != row["feature"]:
            problem(f"{l3_id}: source_crosswalk.json command_id is stale")

        evidence = EVIDENCE / l3_id
        number = l3_id.split("-")[-1]
        dispatch = json.loads(text(evidence / f"OM9-L4-03-16-{number}-01.dispatch.json"))
        if dispatch.get("command_id") != row["feature"]:
            problem(f"{l3_id}: dispatch receipt command_id is stale")
        for path, digest in (dispatch.get("artifact_sha256") or {}).items():
            target = ROOT / path
            if not target.exists():
                problem(f"{l3_id}: dispatch receipt cites missing {path}")
            elif sha256(target) != digest:
                problem(f"{l3_id}: dispatch receipt hash is stale for {path}")
        wire_receipt = json.loads(text(evidence / f"OM9-L4-03-16-{number}-02.wire.json"))
        if wire_receipt.get("enabled") is not True or wire_receipt.get("command_id") != row["feature"]:
            problem(f"{l3_id}: wire receipt does not describe the enabled binding")
        for path, digest in (wire_receipt.get("artifact_sha256") or {}).items():
            target = ROOT / path
            if not target.exists():
                problem(f"{l3_id}: wire receipt cites missing {path}")
            elif sha256(target) != digest:
                problem(f"{l3_id}: wire receipt hash is stale for {path}")
        acceptance = json.loads(text(evidence / f"OM9-L4-03-16-{number}-03.acceptance.json"))
        if acceptance.get("status") != "READY_FOR_REVIEW":
            problem(f"{l3_id}: acceptance status is not READY_FOR_REVIEW")
        if acceptance.get("manual_gate") != "M03-UI":
            problem(f"{l3_id}: acceptance manual gate is not M03-UI")
        if acceptance.get("human_approval") is not None:
            problem(f"{l3_id}: acceptance human_approval must stay null")

        fixtures = ROOT / "implementation/fixtures/M03" / l3_id
        positive_path = fixtures / "positive.json"
        negative_path = fixtures / "negative.json"
        if not positive_path.exists() or not negative_path.exists():
            problem(f"{l3_id}: missing positive/negative fixture")
        else:
            positive = json.loads(text(positive_path))
            if positive.get("command_id") != row["feature"]:
                problem(f"{l3_id}: positive fixture command_id is stale")
            expect = positive.get("expect") or {}
            if expect.get("enabled") is not True or expect.get("ui_state") != "ENABLED":
                problem(f"{l3_id}: positive fixture still expects a disabled row")
            negative = json.loads(text(negative_path))
            if negative.get("disabled_reason"):
                problem(f"{l3_id}: negative fixture still carries a disabled reason")
            if not negative.get("cases"):
                problem(f"{l3_id}: negative fixture has no cases")


def check_l2(found: list[dict]) -> None:
    l2 = json.loads(text(MILESTONE / "L2.json"))
    runtime = l2.get("runtime_implementation") or {}
    if runtime.get("l3_with_verified_supported_scope") != EXPECTED_ROWS:
        problem("L2.json supported scope is not 58")
    if runtime.get("l3_with_verified_disabled_fallback") != 0:
        problem("L2.json still reports disabled fallback rows")
    if l2.get("l3_with_verified_supported_scope") != EXPECTED_ROWS:
        problem("L2.json top-level supported count is stale")
    if l2.get("l3_with_verified_disabled_fallback") != 0:
        problem("L2.json top-level disabled count is stale")
    audit = runtime.get("curve_menu_audit_2026_10_26") or {}
    if audit.get("l3_with_verified_supported_scope") != EXPECTED_ROWS:
        problem("L2.json audit block is stale")
    readme = text(DESIGN / "OM9-L2-03-16/README.md")
    if DATE not in readme:
        problem("design README does not describe the 2026-10-26 state")


def check_evidence_logs() -> None:
    receipt = AFTER / "receipt.json"
    if not receipt.exists():
        problem(f"missing {receipt.relative_to(ROOT)}")
        return
    data = json.loads(text(receipt))
    for log in data.get("logs", []):
        path = ROOT / log["path"]
        if not path.exists():
            problem(f"receipt cites missing log {log['path']}")
        elif sha256(path) != log["sha256"]:
            problem(f"receipt log hash is stale for {log['path']}")
    if data.get("human_approval") is not None:
        problem("record-sync receipt must not claim human approval")
    if data.get("tests", {}).get("failed"):
        problem("record-sync receipt reports failing tests")
    checks = data.get("checks") or []
    if len(checks) != 3:
        problem("record-sync receipt must record fmt, clippy and public-source checks")
    for check in checks:
        path = ROOT / check["stdout_log"]
        if not path.exists():
            problem(f"receipt cites missing check log {check['stdout_log']}")
        elif sha256(path) != check["stdout_log_sha256"]:
            problem(f"receipt check hash is stale for {check['stdout_log']}")
        if check.get("exit_code") != 0:
            problem(f"receipt check exited {check.get('exit_code')}: {check['argv']}")
    if not (data.get("check_notes") or []):
        problem("record-sync receipt must explain the checks it qualifies")


def main() -> int:
    found = rows()
    docs = documented()
    check_catalog(found)
    check_icon_order(found)
    check_icons(found)
    check_features(found)
    check_session_aliases(found, docs)
    if len(found) == EXPECTED_ROWS:
        check_records(found)
        check_l2(found)
    check_evidence_logs()
    RECEIPT.parent.mkdir(parents=True, exist_ok=True)
    RECEIPT.write_text(
        json.dumps(
            {
                "schema": "om9.m03.l2-03-16.audit.v1",
                "date": DATE,
                "module_id": "OM9-L2-03-16",
                "menu_rows": len(found),
                "implemented_rows": sum(1 for row in found if row["state"] == "Implemented"),
                "disabled_rows": sum(1 for row in found if row["state"] != "Implemented"),
                "documented_rows": len(docs),
                "problems": problems,
                "manual_gate": "M03-UI",
                "human_approval": None,
            },
            ensure_ascii=False,
            indent=2,
        )
        + "\n",
        encoding="utf-8",
        newline="\n",
    )
    for message in problems:
        print(f"problem: {message}")
    print(
        f"integrity: {len(problems)} problem(s); {len(found)} menu row(s), "
        f"{len(docs)} documented row(s)"
    )
    return 0 if not problems else 1


if __name__ == "__main__":
    sys.exit(main())
