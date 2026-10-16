"""Add the M07-local L2-01-12 module and its M07-targeted L3-008 child.

The M07 L4 manifest declares 20 canonical L3 for stage E12, but L2-01-12
(Be Native Project Codec Stl Export) lives in the M01 tree because that is its
owner: its L3-008 carries `milestone_id: M01` and `target_milestone: M07`.

M07 needs the module inside its own tree so the E12 queue is self-contained, so
this tool writes an M07-local copy built from:

- the wrapper style of the existing milestone L2 records,
- the patch's enriched v0.13 `module.json` definition for L2-01-12,
- the existing M01 L3-008 record, with the child L3 list narrowed to the L3 that
  actually targets M07.

Nothing in the M01 tree is modified; ownership stays with M01 and the record says so.
"""
import argparse
import json
from pathlib import Path

ROOT = Path("H:/OpenMatrix9")
M01 = (ROOT / "OM9_Milestones" /
       "M01 - CAD Shell, OCCT Bridge, Document, Undo, GPU and Viewport Foundation")
M07 = ROOT / "OM9_Milestones" / "M07 - Cutters and Manufacturing Validation"
PATCH = ROOT / "build" / "m07-incoming" / "OpenMatrix9" / "specs"
L2_SRC = (PATCH / "01-ms01-workbench-shell" /
          "05-be-native-project-codec-stl-export")
L2_ID = "OM9-L2-01-12"
L3_ID = "OM9-L3-01-12-008"
L2_DIRNAME = "L2-01-12 - Be Native Project Codec Stl Export"
L3_DIRNAME = "L3-008 - STL Tessellation Export Codec"
MANIFEST_OUT = ROOT / "build" / "m07-module-add-manifest.json"


def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def write(path, data):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--apply", action="store_true")
    args = parser.parse_args()

    m01_l2 = read(M01 / L2_DIRNAME / "L2.json")
    m01_l3 = read(M01 / L2_DIRNAME / L3_DIRNAME / "l3.json")
    patch_definition = read(L2_SRC / "module.json")

    if m01_l3.get("target_implementation_milestone") != "M07":
        raise SystemExit(f"{L3_ID} no longer targets M07; refusing to copy")

    l2 = {
        "level": "L2",
        "id": L2_ID,
        "milestone": "M07",
        "title": m01_l2["title"],
        "planning_state": "SPEC_ONLY_NOT_IMPLEMENTED",
        "scope": m01_l2["scope"],
        "ownership": {
            "owner_milestone": "M01",
            "owner_module_record": (f"M01 - CAD Shell, OCCT Bridge, Document, Undo, GPU and "
                                    f"Viewport Foundation/{L2_DIRNAME}/L2.json"),
            "m07_role": "execution host for the M07-targeted child L3 only",
            "note": ("This M07 copy exists so the E12 queue is self-contained. L2-01-12 is owned "
                     "and specified in M01; M07 only executes its target_milestone=M07 child "
                     "L3. Do not diverge the two copies: edit the M01 record and re-derive here."),
        },
        "source_original_path": patch_definition.get("module_path"),
        "definition": patch_definition,
        "children_l3": [
            {"id": L3_ID,
             "file": f"M07 - Cutters and Manufacturing Validation/{L2_DIRNAME}/{L3_DIRNAME}/l3.json"}
        ],
        "m07_delegated_l3_count": 1,
    }

    l3 = dict(m01_l3)
    l3["milestone"] = "M07"
    l3["parent_l2_id"] = L2_ID
    l3["children_l4"] = l3.get("children_l4") or []
    l3["l4_status"] = l3.get("l4_status") or "NOT_PLANNED_YET"
    l3["ownership"] = {
        "owner_milestone": "M01",
        "target_milestone": "M07",
        "owner_record": (f"M01 - CAD Shell, OCCT Bridge, Document, Undo, GPU and Viewport "
                         f"Foundation/{L2_DIRNAME}/{L3_DIRNAME}/l3.json"),
        "note": "Delegated L3: M01 owns the specification, M07 hosts the E12 execution queue.",
    }

    report = {
        "schema": "om9.m07.module_add.v1",
        "l2": L2_ID, "l3": L3_ID,
        "l2_dir": L2_DIRNAME, "l3_dir": L3_DIRNAME,
        "source_module_json": str(L2_SRC.relative_to(ROOT).as_posix()),
        "source_l3_record": str((M01 / L2_DIRNAME / L3_DIRNAME / "l3.json").relative_to(ROOT).as_posix()),
        "m01_tree_modified": False,
        "files": [f"{L2_DIRNAME}/L2.json", f"{L2_DIRNAME}/{L3_DIRNAME}/l3.json"],
    }
    if args.apply:
        write(M07 / L2_DIRNAME / "L2.json", l2)
        write(M07 / L2_DIRNAME / L3_DIRNAME / "l3.json", l3)
    MANIFEST_OUT.write_text(json.dumps(report, ensure_ascii=False, indent=1) + "\n",
                            encoding="utf-8")
    print(json.dumps({"apply": args.apply, **report}, ensure_ascii=False, indent=1))


if __name__ == "__main__":
    main()
