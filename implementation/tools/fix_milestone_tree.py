#!/usr/bin/env python
"""Fix the four planning-tree defects found by `audit_milestone_tree.py`.

1. The M07 "L2-01-12" folder is a *placement shell*: its four L4 records belong to the canonical
   `OM9-L2-01-12` (owner_module_id says so) and must stay in M07, but the copied `L2.json`/`l3.json`
   duplicate ids that M01 already owns. Move the four L4 folders under
   `M07 - Cutters/placements/OM9-L3-01-12-008/` and delete the duplicated shells; record the
   placement in M07's L1 the way M08 records cross-milestone placements.
2. M03 hosts M08's 44 L4 records without saying so; record `hosted_l4_from`.
3. M03..M08, M09 and AFTER declare no `depends_on_milestones`, so nothing in the tree prevents
   executing geometry before the OCCT bridge; add the real dependencies.
4. Layer clipboard/exchange has no L3 owner (only one unrelated AFTER record matches); record the gap
   on the nearest owner (`OM9-L2-03-14`) instead of inventing plan items.

    python implementation/tools/fix_milestone_tree.py            # dry run, prints the plan
    python implementation/tools/fix_milestone_tree.py --apply
"""
from __future__ import annotations

import argparse
import json
import pathlib
import shutil

ROOT = pathlib.Path(__file__).resolve().parents[2]
TREE = ROOT / "OM9_Milestones"
M01 = TREE / "M01 - CAD Shell, Viewport"
M03 = TREE / "M03 - Geometry 3DM"
M07 = TREE / "M07 - Cutters"
M09 = TREE / "M09 - Acceptance and Release Gates"
AFTER = TREE / "ZAFTER - Extensions"

DUPLICATE = M07 / "L2-01-12 - Be Native Project Codec Stl Export"
PLACEMENT = M07 / "placements" / "OM9-01-12-008"

DEPENDENCIES = {
    "M03": ["M01"],
    "M04": ["M01"],
    "M05": ["M03"],
    "M06": ["M03", "M05"],
    "M07": ["M03"],
    "M08": ["M03", "M05", "M06"],
    "M09": ["M00", "M01", "M02", "M03", "M04", "M05", "M06", "M07", "M08"],
    "AFTER": ["M09"],
}


def read(path: pathlib.Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def write(path: pathlib.Path, data: dict) -> None:
    path.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def assert_intended(path: pathlib.Path, *needles: str) -> None:
    resolved = path.resolve()
    assert str(resolved).startswith(str(TREE.resolve())), f"outside the plan tree: {resolved}"
    for needle in needles:
        assert needle in str(resolved), f"{needle} missing from {resolved}"
    print(f"  ok path: {resolved.relative_to(ROOT)}")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--apply", action="store_true")
    args = parser.parse_args()
    print("milestone tree fix", "(apply)" if args.apply else "(dry run)")

    # ---------------------------------------------------------------- fix 1
    print("\n[1] M07 placement shell -> placements/OM9-L3-01-12-008")
    assert_intended(DUPLICATE, "M07 - Cutters", "L2-01-12")
    l2 = DUPLICATE / "L2.json"
    l3 = DUPLICATE / "L3-008 - STL Tessellation Export Codec"
    assert l2.exists(), l2
    assert (l3 / "l3.json").exists(), l3
    l4_dirs = sorted(p for p in l3.iterdir() if p.is_dir() and p.name.startswith("L4-"))
    print(f"  L4 placement records to keep: {[p.name for p in l4_dirs]}")
    assert len(l4_dirs) == 4, "expected the four delegated L4 records"
    duplicate_l2 = read(l2)
    owner = read(l3 / "l3.json").get("ownership", {})
    print(f"  ownership: {owner}")
    if args.apply:
        PLACEMENT.mkdir(parents=True, exist_ok=True)
        for directory in l4_dirs:
            shutil.move(str(directory), str(PLACEMENT / directory.name))
        # Only the duplicated shell may be removed; the L4 records now live under placements/.
        assert_intended(DUPLICATE, "M07 - Cutters", "L2-01-12")
        shutil.rmtree(DUPLICATE)
        write(
            PLACEMENT / "placement.json",
            {
                "schema": "om9.placement.v1",
                "l3_id": "OM9-L3-01-12-008",
                "owner_milestone": "M01",
                "owner_l2_id": duplicate_l2.get("id"),
                "target_milestone": "M07",
                "owner_record": owner.get("owner_record"),
                "note": owner.get("note"),
                "moved_from": "M07 - Cutters/L2-01-12 - Be Native Project Codec Stl Export/",
                "reason": "a delegated L3 keeps the owner's ids; the execution queue is placed here",
            },
        )
        m07_l1 = M07 / "L1.json"
        data = read(m07_l1)
        data["l3_placed_by_milestone"] = {"OM9-L3-01-12-008": "M01"}
        data["l4_placed_by_milestone"] = {"M01": len(l4_dirs)}
        data["placement_records"] = ["placements/OM9-01-12-008"]
        data["cross_milestone_note"] = (
            "M01 owns L2-01-12/L3-008 and its ids; M07 hosts the delegated E12 execution queue as "
            "four L4 placement records under placements/OM9-L3-01-12-008 (no duplicated L2/L3 ids)."
        )
        write(m07_l1, data)
        print(f"  deleted {DUPLICATE.relative_to(ROOT)} (duplicated ids removed)")

    # ---------------------------------------------------------------- fix 2
    print("\n[2] M03 records the L4 records it hosts for M08")
    m03_l1 = M03 / "L1.json"
    data = read(m03_l1)
    hosted = data.get("l4_placed_by_milestone")
    print(f"  M03 before: hosted_l4_from={hosted}")
    if args.apply:
        data["hosted_l4_from"] = {"M08": 44}
        data["hosted_l4_note"] = (
            "M08 keeps its canonical L3 ids and its L1 records l4_placed_by_milestone {M03: 44, "
            "M08: 177}; the 44 M08 L4 records are physically hosted in M03, so M03's physical L4 "
            "count is declared + 44."
        )
        write(m03_l1, data)

    # ---------------------------------------------------------------- fix 3
    print("\n[3] milestone dependencies")
    for key, depends in DEPENDENCIES.items():
        folder = {"M03": M03, "M09": M09, "AFTER": AFTER}.get(key)
        if folder is None:
            folder = next(p for p in TREE.iterdir() if p.is_dir() and p.name.startswith(key))
        path = folder / "L1.json"
        data = read(path)
        print(f"  {key}: {data.get('depends_on_milestones')!r} -> {depends}")
        if args.apply:
            data["depends_on_milestones"] = depends
            write(path, data)

    # ---------------------------------------------------------------- fix 3b
    # M01/M02 store the dependency as a bare string while M03.. use a list; one shape only.
    print("\n[3b] normalise a string dependency into a list")
    for folder in sorted(p for p in TREE.iterdir() if p.is_dir()):
        path = folder / "L1.json"
        if not path.exists():
            continue
        data = read(path)
        value = data.get("depends_on_milestones")
        if isinstance(value, str):
            print(f"  {folder.name}: {value!r} -> [{value!r}]")
            if args.apply:
                data["depends_on_milestones"] = [value]
                write(path, data)

    # ---------------------------------------------------------------- fix 4
    print("\n[4] layer clipboard gap on the nearest owner")
    layers = next(M01.glob("L2-03-14*"))
    path = layers / "L2.json"
    data = read(path)
    print(f"  {data.get('id')}: gap_note={data.get('gap_note')!r}")
    if args.apply:
        data["gap_note"] = (
            "Layer clipboard/exchange has no L3 owner in this tree while the portable kernel exists "
            "(implementation/design/M01/W5-LAYER-CLIPBOARD.md, 10 tests). This L2 is the nearest "
            "owner; authoring the owning L3 (and its L4 queue) needs an explicit plan decision."
        )
        write(path, data)

    if not args.apply:
        print("\ndry run only; pass --apply to write the changes")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
