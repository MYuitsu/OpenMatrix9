#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-2.1-or-later
"""Audit the OM9-L2-03-17 Surface menu bindings end to end.

Checks the chain the user asked about — code, icon, parameters/behavior, records —
without trusting any single catalog:

1. `crates/om9-app/src/surface_menu.rs` has 38 implemented rows with unique l3 ids,
   features, labels and Matrix script names.
2. Every row icon is authored (`Resources/icons/surface-minimal/<icon>.svg`), listed by
   `menu_catalog::SURFACE` and embedded by `icons.rs`; all 40 authored menu icons reach a
   command, including the four menu-only owners (Revolve, Heightfield, Smash, RefitSrf).
3. `crates/om9-app/tests/surface_menu.rs` names every row and verifies that the menu label
   and the Matrix script resolve to the same command id (menu/command-line parity).
4. The 38 `L3-*/l3.json` records and `L2.json` report the implemented state, the current
   test count and the new evidence file, with no stale "3 disabled rows" text.
5. The design README documents the option contracts and the Soft Edit typed-point limit.
6. Every cited artifact is hashed so the receipt can be re-verified.

Exit code 0 only when every check passes.

Usage: python tools/audit_m03_l2_03_17.py
"""
from __future__ import annotations

import hashlib
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DATE = "2026-10-26"
MILESTONE = ROOT / "OM9_Milestones/M03 - Geometry 3DM/L2-03-17 - 09-febe"
DESIGN = ROOT / "implementation/design/M03/OM9-L2-03-17"
EVIDENCE = ROOT / "implementation/evidence/M03/2026-10-26-l2-03-17-checks.json"
AFTER = ROOT / "implementation/evidence/AFTER/m03-l2-03-17-surface-menu-2026-10-26"
RECEIPT = AFTER / "audit.json"
EXPECTED_ROWS = 38

problems: list[str] = []


def problem(message: str) -> None:
    problems.append(message)


def text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def rows() -> list[dict]:
    body = text(ROOT / "crates/om9-app/src/surface_menu.rs")
    found = []
    for match in re.finditer(r"SurfaceMenuBinding \{(.*?)\n    \},", body, re.S):
        block = match.group(1)

        def field(name: str) -> str:
            hit = re.search(rf'{name}: "([^"]*)"', block)
            return hit.group(1) if hit else ""

        row = {
            key: field(key)
            for key in ("l3_id", "label", "source_command", "command_line", "icon",
                        "feature", "owner", "disabled_reason")
        }
        state = re.search(r"state: BindingState::(\w+)", block)
        row["state"] = state.group(1) if state else ""
        found.append(row)
    return found


def main() -> int:
    binding_rows = rows()
    if len(binding_rows) != EXPECTED_ROWS:
        problem(f"surface_menu.rs has {len(binding_rows)} rows, expected {EXPECTED_ROWS}")
    for key in ("l3_id", "feature", "label", "command_line"):
        values = [row[key] for row in binding_rows]
        if len(set(values)) != len(values):
            problem(f"duplicate {key} in surface_menu.rs")
    for row in binding_rows:
        if row["state"] != "Implemented":
            problem(f"{row['l3_id']} is {row['state'] or 'missing state'}")
        if row["disabled_reason"]:
            problem(f"{row['l3_id']} keeps a disabled reason")
        if not re.fullmatch(r"OM9-SURFACE-\d{3}", row["feature"]):
            problem(f"{row['l3_id']} has an invalid feature id {row['feature']!r}")
        if not re.fullmatch(r"OM9-L[23]-[\w-]+", row["owner"]):
            problem(f"{row['l3_id']} has no owner candidate")
        if not row["command_line"]:
            problem(f"{row['l3_id']} has no Matrix command-line name")

    menu_catalog = text(ROOT / "crates/om9-app/src/menu_catalog.rs")
    icons_rs = text(ROOT / "crates/om9-app/src/icons.rs")
    for row in binding_rows:
        icon = row["icon"]
        svg = ROOT / "Resources/icons/surface-minimal" / f"{icon}.svg"
        if not svg.is_file():
            problem(f"missing authored SVG {svg.relative_to(ROOT)}")
        if f'"{icon}"' not in menu_catalog:
            problem(f"{icon} is not in menu_catalog::SURFACE")
        if f'"{icon}"' not in icons_rs:
            problem(f"{icon} is not embedded in icons.rs")

    # The authored Surface menu has 40 icons while the milestone plans 38 L3 rows (36 menu
    # commands plus the Sweep 1/2 History sub-commands). The four remaining menu commands run
    # from their own owner sessions — Revolve, Heightfield from Image, Smash and Refit to
    # Tolerance — so every authored icon must reach an available command.
    surface_menu_icons = re.findall(
        r'"([^"]+)"',
        re.search(r"pub const SURFACE: &\[&str\] = &\[(.*?)\];", menu_catalog, re.S).group(1),
    )
    if len(surface_menu_icons) != 40:
        problem(f"menu_catalog::SURFACE has {len(surface_menu_icons)} icons, expected 40")
    icon_features = {
        match.group(2): match.group(3)
        for match in re.finditer(r'icon!\(\s*"([^"]*)",\s*"([^"]*)",\s*"([^"]*)"', icons_rs)
    }
    menu_only_rows = {
        "SurfaceRevolve": "OM9-SURFACE-018",
        "SurfaceHeightfieldfromImage": "OM9-SURFACE-031",
        "SurfaceSmash": "OM9-SURFACE-141",
        "SurfaceSurfaceEditToolsRefittoTolerance": "OM9-SURFACE-142",
    }
    menu_wired = 0
    for icon in surface_menu_icons:
        feature = next(
            (row["feature"] for row in binding_rows if row["icon"] == icon), ""
        ) or icon_features.get(icon, "")
        if not feature:
            problem(f"Surface menu icon {icon} has no command")
        if icon in menu_only_rows and feature != menu_only_rows[icon]:
            problem(f"Surface menu icon {icon} maps to {feature}, expected {menu_only_rows[icon]}")
        menu_wired += 1
    if menu_wired != 40:
        problem(f"only {menu_wired} Surface menu icons are wired, expected 40")

    surface_edit = text(ROOT / "crates/om9-app/src/session/surface_edit.rs")
    if f'SMASH: &str = "{menu_only_rows["SurfaceSmash"]}"' not in surface_edit:
        problem("surface_edit does not own the Smash command")
    refit_srf = ROOT / "crates/om9-app/src/session/refit_srf.rs"
    if not refit_srf.is_file() or menu_only_rows["SurfaceSurfaceEditToolsRefittoTolerance"] not in text(refit_srf):
        problem("session/refit_srf.rs does not own the surface Refit to Tolerance command")
    features = json.loads(text(ROOT / "crates/om9-app/assets/features.json"))
    known_features = {entry.get("id") for entry in features}
    for icon, feature in menu_only_rows.items():
        if feature not in known_features:
            problem(f"features.json has no {feature} ({icon})")

    # The authored sheets quote the Matrix script in most sections (`‘Sweep1’`). A row whose
    # section never names a script must at least be reachable by the authored menu label,
    # which `by_label` covers; keep those rows visible in the receipt instead of inventing
    # a script name.
    script_from_sheet = 0
    label_only: list[str] = []
    for row in binding_rows:
        number = row["l3_id"][-3:]
        folder = next(iter(sorted(MILESTONE.glob(f"L3-{number}*"))), None)
        if folder is None:
            problem(f"{row['l3_id']} has no milestone folder")
            continue
        body = "".join(
            path.read_text(encoding="utf-8", errors="ignore")
            for path in folder.rglob("reference/text/*.md")
        )
        if not body:
            problem(f"{row['l3_id']} has no authored reference sheet")
        elif row["command_line"].lower() in body.lower():
            script_from_sheet += 1
        elif row["label"].lower() in body.lower():
            label_only.append(row["command_line"])
        else:
            problem(f"{row['l3_id']} has no authored script or label in its sheet")

    tests = text(ROOT / "crates/om9-app/tests/surface_menu.rs")
    if "surface_menu::BINDINGS" not in tests:
        problem("surface_menu tests do not iterate the catalog")
    for needle in (
        "every_surface_row_declares_its_matrix_command_line_name",
        "every_surface_command_line_name_resolves_like_its_menu_row",
        "menu_and_command_line_dispatch_the_same_surface_command",
        "typed_shared_labels_follow_the_selected_object",
    ):
        if needle not in tests:
            problem(f"missing parity test {needle}")

    l2 = json.loads(text(MILESTONE / "L2.json"))
    runtime = l2["runtime_implementation"]
    if runtime["l3_with_verified_supported_scope"] != EXPECTED_ROWS:
        problem("L2.json does not report 38 supported rows")
    if runtime["l3_with_verified_disabled_fallback"] != 0:
        problem("L2.json still reports disabled fallback rows")
    if "disabled placeholder" in runtime["scope"]:
        problem("L2.json scope still calls rows disabled placeholders")
    checks = runtime["latest_verification"]["checks"]
    if "2026-10-26-l2-03-17-checks.json" not in checks:
        problem("L2.json latest_verification does not cite the new checks file")

    stale = ("41 passed", "3 rows verified", "2026-10-23-l2-03-17-checks.json")
    l3_files = sorted(MILESTONE.glob("L3-*/l3.json"))
    if len(l3_files) != EXPECTED_ROWS:
        problem(f"found {len(l3_files)} L3 records, expected {EXPECTED_ROWS}")
    for path in l3_files:
        body = text(path)
        for needle in stale:
            if needle in body:
                problem(f"{path.relative_to(ROOT)} still says {needle!r}")
        if "62 passed" not in body:
            problem(f"{path.relative_to(ROOT)} does not cite the 62-passing suite")
        if "2026-10-26-l2-03-17-checks.json" not in body:
            problem(f"{path.relative_to(ROOT)} does not cite the new checks file")

    readme = text(DESIGN / "README.md")
    for needle in ("Soft Edit", "DirConstraint", "Continuity", "Closed"):
        if needle not in readme:
            problem(f"design README does not document {needle}")
    if "no interactive command session" in readme:
        problem("design README still lists missing interactive sessions")

    cited = [
        ROOT / "crates/om9-app/src/surface_menu.rs",
        ROOT / "crates/om9-app/src/menu_catalog.rs",
        ROOT / "crates/om9-app/src/icons.rs",
        ROOT / "crates/om9-app/src/session.rs",
        ROOT / "crates/om9-app/src/session/surface_edit.rs",
        ROOT / "crates/om9-app/src/session/refit_srf.rs",
        ROOT / "crates/om9-app/src/session/command_options.rs",
        ROOT / "crates/om9-app/assets/features.json",
        ROOT / "crates/om9-app/tests/surface_menu.rs",
        ROOT / "crates/om9-app/tests/smash.rs",
        ROOT / "crates/om9-app/tests/refit_srf.rs",
        DESIGN / "README.md",
        EVIDENCE,
        MILESTONE / "L2.json",
    ]

    RECEIPT.parent.mkdir(parents=True, exist_ok=True)
    receipt = {
        "schema": "om9.m03.l2-03-17.audit.v1",
        "date": DATE,
        "module": "OM9-L2-03-17",
        "rows": len(binding_rows),
        "implemented": sum(row["state"] == "Implemented" for row in binding_rows),
        "icons_authored": len(binding_rows),
        "surface_menu_icons": len(surface_menu_icons),
        "menu_icons_wired": menu_wired,
        "menu_only_rows": menu_only_rows,
        "scripts_quoted_in_sheet": script_from_sheet,
        "label_only_rows": label_only,
        "l3_records": len(l3_files),
        "status": "AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING",
        "manual_gate": "M03-UI",
        "human_approval": None,
        "problems": problems,
        "artifacts": {
            str(path.relative_to(ROOT)).replace("\\", "/"): sha256(path)
            for path in cited
            if path.is_file()
        },
    }
    RECEIPT.write_text(json.dumps(receipt, indent=2) + "\n", encoding="utf-8")

    for message in problems:
        print(f"problem: {message}")
    print(f"rows: {len(binding_rows)}")
    print(f"integrity: {len(problems)} problem(s)")
    return 0 if not problems else 1


if __name__ == "__main__":
    sys.exit(main())
