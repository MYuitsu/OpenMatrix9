#!/usr/bin/env python
"""OM9-L2-03-14 — one automated verification pass (all eight L3 + lints + real window)."""
from __future__ import annotations
import argparse, pathlib, subprocess, sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
TESTS = [
    ("OM9-L3-03-14-001", "Matrix Layer Category Palette", "layer_category_palette"),
    ("OM9-L3-03-14-002", "Current Layer And Assignment UI", "layer_current_and_assign"),
    ("OM9-L3-03-14-003", "Layer Visible And Locked Controls", "layer_visible_locked"),
    ("OM9-L3-03-14-004", "Select All Objects By Layer", "layer_select_by_layer"),
    ("OM9-L3-03-14-005", "Matrix Object Info Overlay", "object_info_overlay"),
    ("OM9-L3-03-14-006", "Superselect By Jewelry Object Type", "superselect_jewelry"),
    ("OM9-L3-03-14-007", "Object Property Editor Adapter", "property_editor_adapter"),
    ("OM9-L3-03-14-008", "Layer State Persistence Visual Check", "layer_state_persistence"),
    ("L3-006 UI", "Tools menu picks the semantic gem", "super_select_ui"),
    ("L3-007 UI", "Edit menu edits one property in one transaction", "property_menu_ui"),
]
SUITES = [
    ("lib", ["test", "-p", "om9-app", "--lib"]),
    ("core", ["test", "-p", "om9-core"]),
    ("fmt", ["fmt", "--check"]),
    ("clippy", ["clippy", "-p", "om9-core", "-p", "om9-app", "--all-targets", "-j", "2", "--", "-D", "warnings"]),
]
WINDOW = ["python", "tools/validate_m01_l3_003_panels.py", "--output",
          "build/standalone-validation/ui-artifacts/l2-03-14-verify"]

def run(args):
    f = subprocess.run(args, cwd=str(ROOT), capture_output=True, text=True)
    text = (f.stdout or "") + (f.stderr or "")
    line = next((l for l in text.splitlines() if l.startswith("test result")), "")
    return f.returncode, line or text.strip().splitlines()[-1][:90] if text.strip() else ""

def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--quick", action="store_true")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        assert len(TESTS) == 10
        print("self-test: 10 L3 wired")
        return 0
    rows, failures = [], 0
    for l3, title, test in TESTS:
        code, line = run(["cargo", "test", "-p", "om9-app", "--test", test, "-j", "2"])
        rows.append((l3, title, code == 0, line)); failures += code != 0
    for label, command in SUITES:
        code, line = run(["cargo", *command])
        rows.append((label, "", code == 0, line)); failures += code != 0
    if not args.quick:
        code, line = run(WINDOW)
        rows.append(("real-window", "", code == 0, line)); failures += code != 0
    width = max(len(r[0]) for r in rows)
    print("\nOM9-L2-03-14 — automated verification")
    for name, title, ok, line in rows:
        print(f"{name.ljust(width)}  {'ok' if ok else 'FAILED':6}  {title}")
        if line:
            print(f"{' ' * width}          {line}")
    print(f"\n{len(rows) - failures}/{len(rows)} groups ok")
    return 1 if failures else 0

if __name__ == "__main__":
    sys.exit(main())
