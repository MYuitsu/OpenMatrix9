"""Kiem thu co hoc cho phan doi ten thu muc (chay tren cay gia trong implementation/).

Chay: python implementation/tools/test_om9_shorten_milestone_paths.py
"""

from __future__ import annotations

import json
import os
import shutil
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

import om9_shorten_milestone_paths as tool  # noqa: E402

SCRATCH = os.path.join(os.path.dirname(HERE), "tmp-path-rename-test")
MS = os.path.join(SCRATCH, "OM9_Milestones")

M02 = "M02 - Viewport Picking, Snapping, Navigation, Gumball and Matrix Interaction"
M03 = "M03 - Curve, Surface, Solid and 3DM Exchange"
M05 = "M05 - Gems and Layout"

L3_SELECT = "L3-004 - Selection Window And Crossing"
L4_DISPATCH = "L4-01 - Command Dispatch Contract"
L3_OTHER = "L3-005 - Object Selection Disambiguation"
L3_MENU = "L3-016 - Transform \u00b7 Orient Perpendicular to Curve - menu binding"
L4_MENU = "L4-02 - Menu and Command UI Integration"
L2_PICK = "L2-01-07 - Febe Picking Selection Generic Handles"
L2_XFORM = "L2-04-01 - 13-febe-matrix-transform-toolbar-bridges"


def build_tree() -> None:
    if os.path.isdir(SCRATCH):
        shutil.rmtree(SCRATCH)
    leafs = [
        (M02, L2_PICK, L3_SELECT, L4_DISPATCH, "l4.json"),
        (M02, L2_PICK, L3_OTHER, None, os.path.join("reference", "text", "note.md")),
        (M02, L2_PICK, L3_SELECT, None, os.path.join("L2.json")),
        (M03, L2_XFORM, L3_MENU, L4_MENU, "l4.json"),
        (M03, L2_XFORM, L3_MENU, "L4-01 - Command Dispatch Contract", "l4.json"),
        (M05, "L2-04-11 - 03-febe-gem-halo-cluster-baguette-layouts",
         "L3-004 - Tapered Baguettes Between Two Curves", None, "l3.json"),
    ]
    for module, l2, l3, l4, leaf in leafs:
        parts = [MS, module, l2, l3] + ([l4] if l4 else [])
        directory = os.path.join(*parts)
        os.makedirs(directory, exist_ok=True)
        path = os.path.join(directory, leaf)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w", encoding="utf-8") as fh:
            fh.write("{}\n")


def snapshot() -> set[str]:
    out = set()
    for dp, dn, fn in os.walk(MS):
        for name in list(dn) + list(fn):
            out.add(os.path.relpath(os.path.join(dp, name), MS))
    return out


def check(label: str, cond: bool) -> bool:
    print(f"  {'ok  ' if cond else 'FAIL'}  {label}")
    return cond


def main() -> int:
    build_tree()
    ok = True
    renames = {
        os.path.join(MS, M02, L2_PICK, L3_SELECT): "L3-004 - Selection Window",
        os.path.join(MS, M02, L2_PICK, L3_SELECT, L4_DISPATCH): "L4-01 - Command Dispatch",
        os.path.join(MS, M03, L2_XFORM, L3_MENU):
            "L3-016 - Transform \u00b7 Orient Perpendicular to Curve",
    }
    alias = {M02: "M02 - Viewport, Gumball", M03: "M03 - Curve, Surface, 3DM",
             M05: "M05 - Gems and Layout"}

    print("1. doi ten thu muc (module + L2/L3/L4 long nhau)")
    done = tool.execute_renames(MS, renames, alias)
    ok &= check("doi 5 thu muc (2 module + 3 long nhau)", len(done) == 5)
    ok &= check("module M02 doi ten",
                os.path.isdir(os.path.join(MS, "M02 - Viewport, Gumball")))
    ok &= check("module M03 doi ten",
                os.path.isdir(os.path.join(MS, "M03 - Curve, Surface, 3DM")))
    ok &= check("M05 giu nguyen", os.path.isdir(os.path.join(MS, M05)))
    deep = os.path.join(MS, "M02 - Viewport, Gumball", L2_PICK, "L3-004 - Selection Window",
                        "L4-01 - Command Dispatch", "l4.json")
    ok &= check("file la van con (L4 long nhau)", os.path.isfile(deep))
    deep2 = os.path.join(MS, "M03 - Curve, Surface, 3DM", L2_XFORM,
                         "L3-016 - Transform \u00b7 Orient Perpendicular to Curve",
                         "L4-02 - Menu and Command UI Integration", "l4.json")
    ok &= check("L4 khong doi ten nam dung cho", os.path.isfile(deep2))
    ok &= check("ten cu da bien mat",
                not os.path.exists(os.path.join(MS, M02))
                and not os.path.exists(os.path.join(MS, M03)))
    ok &= check("thu muc khong doi ten giu nguyen ten",
                os.path.isdir(os.path.join(MS, "M02 - Viewport, Gumball", L2_PICK, L3_OTHER)))

    print("2. chay lai (idempotent)")
    before = snapshot()
    done2 = tool.execute_renames(MS, renames, alias)
    ok &= check("lan 2 khong doi gi", done2 == [] and snapshot() == before)

    print("3. loi -> tu hoan tac")
    before = snapshot()
    bad = dict(renames)
    bad[os.path.join(MS, "M02 - Viewport, Gumball", L2_PICK, L3_OTHER)] = "L3-005 - Object Selection"
    bad[os.path.join(MS, "M05 - Gems and Layout", "khong-ton-tai")] = "L3-999 - Khong Co"
    raised = False
    try:
        tool.execute_renames(MS, bad, {})
    except Exception as exc:  # noqa: BLE001
        raised = True
        print(f"     (loi mong doi: {exc})")
    ok &= check("nem loi khi thieu thu muc nguon", raised)
    ok &= check("cay khong bi thay doi sau khi hoan tac", snapshot() == before)

    print("4. thong ke")
    stats = tool.tree_stats(MS)
    ok &= check("tree_stats dem duoc file", stats["files"] >= 5 and stats["max"] > 0)
    print(f"     {json.dumps(stats)}")

    shutil.rmtree(SCRATCH, ignore_errors=True)
    print("\nKET QUA:", "PASS" if ok else "FAIL")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
