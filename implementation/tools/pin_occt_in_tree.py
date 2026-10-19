#!/usr/bin/env python
"""Record the OCCT 8.0.1 decision inside the milestone tree.

The tree is a specification: this script adds the *pin reference* and the measured state to the
records that own OCCT, rewrites the version strings that named 7.8.1/7.9.3 in the task intent, and
leaves every `planning_state` untouched. Historical findings and runbook text keep their original
wording, with a dated note appended instead of being rewritten.

    python implementation/tools/pin_occt_in_tree.py            # dry run
    python implementation/tools/pin_occt_in_tree.py --apply
"""
from __future__ import annotations

import argparse
import json
import pathlib

ROOT = pathlib.Path(__file__).resolve().parents[2]
TREE = ROOT / "OM9_Milestones"
M00 = TREE / "M00 - Feasibility, Legal"
M01 = TREE / "M01 - CAD Shell, Viewport"
M03 = next(TREE.glob("M03*"))

PIN = {
    "version": "8.0.1",
    "tag": "V8_0_1",
    "commit": "b8f597c677811d1f9f4d8a97f5ae2825c0353a42",
    "pin_file": "native/occt/pin.json",
    "fetch_tool": "implementation/tools/fetch_occt.py",
    "accuracy_patch": "tools/patches/occt-8.0.1-offset-d3.patch",
    "upgrade_gate": "implementation/tools/occt_upgrade_gate.py",
    "gate_evidence": "implementation/evidence/M00/OM9-L2-02/OCCT-CONFORMANCE-8.0.1-stock.json",
    "gate_record": "implementation/design/M00/OM9-L2-02/OCCT-UPGRADE-GATE.md",
}

STATE = {
    "OM9-M00-L3-02-01": {
        "l4_done": ["OM9-M00-L4-02-01-01", "OM9-M00-L4-02-01-02", "OM9-M00-L4-02-01-03"],
        "l4_open": ["OM9-M00-L4-02-01-04"],
        "evidence": [
            "implementation/evidence/M00/OM9-L4-02-01-01/qymcad-buildrs-audit.md",
            "implementation/evidence/M00/OM9-L4-02-01-02/qymcad-kernel-build-against-occt-8.0.1.log",
            "implementation/evidence/M00/OM9-L4-02-01-03/occt-probe.json",
            "implementation/design/M00/OM9-L2-02/GATE-L3-01-PROOF.md",
        ],
        "next": "L4-04: a minimal Rust application calls OCCT through crates/om9-kernel (bridge authored for the 8.0 API)",
    },
    "OM9-M00-L3-02-02": {
        "l4_done": [],
        "l4_open": [],
        "evidence": [],
        "next": "lifetime and threading: create/destroy 10k shapes, one call at a time through KernelPolicy, report RSS and handle counts",
    },
    "OM9-M00-L3-02-03": {
        "l4_done": [],
        "l4_open": [],
        "evidence": [],
        "next": "numeric invariants: tolerance sweep with BRepCheck plus analytic volume/area comparison, including the patched offset D3 case",
    },
}

REWRITE = {
    "Kiểm tra OCCT 7.8+, libs, MSVC C++17, linker, flags và đường dẫn hệ thống":
        "Kiểm tra OCCT 8.0.1 (pin: native/occt/pin.json), libs, MSVC C++17, linker, flags và đường dẫn hệ thống",
    "Không trộn OCCT 7.8.1 của tài liệu với 7.9.3 CI; mỗi version có source line":
        "Chỉ dùng OCCT 8.0.1 theo pin native/occt/pin.json; mọi bằng chứng trích version + commit của pin",
}


def load(path: pathlib.Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def write(path: pathlib.Path, data: dict) -> None:
    path.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def rewrite_strings(value, changed: list[str]):
    if isinstance(value, str):
        if value in REWRITE:
            changed.append(value)
            return REWRITE[value]
        return value
    if isinstance(value, list):
        return [rewrite_strings(item, changed) for item in value]
    if isinstance(value, dict):
        return {key: rewrite_strings(item, changed) for key, item in value.items()}
    return value


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--apply", action="store_true")
    args = parser.parse_args()
    print("pin OCCT 8.0.1 in the tree", "(apply)" if args.apply else "(dry run)")

    # 1. M00 L2-02 and its L3 records get the pin + measured state.
    l2_path = M00 / "L2-02 - OCCT" / "L2.json"
    l2 = load(l2_path)
    print(f"\n[1] {l2['id']} kernel_pin + gate state")
    if args.apply:
        l2["kernel_pin"] = PIN
        l2["gate_state"] = {
            "as_of": "2026-10-17",
            "l3_01": "3/4 L4 done (audit, reference-bridge compile attempt, C-ABI/exception probe)",
            "l3_02": "not started",
            "l3_03": "not started",
            "note": "the tree stays SPEC_ONLY_NOT_IMPLEMENTED; measured evidence lives under implementation/evidence/M00",
        }
        write(l2_path, l2)

    for l3_dir in sorted(p for p in (M00 / "L2-02 - OCCT").iterdir() if p.is_dir() and p.name.startswith("L3-")):
        l3_path = l3_dir / "l3.json"
        l3 = load(l3_path)
        state = STATE.get(l3["id"])
        if not state:
            continue
        print(f"  {l3['id']}: pin + execution_evidence")
        if args.apply:
            l3["kernel_pin"] = {k: PIN[k] for k in ("version", "tag", "commit", "pin_file")}
            l3["execution_evidence"] = state
            l3["version_history"] = [
                {
                    "date": "2026-10-17",
                    "change": "OCCT 8.0.1 pinned; 7.8.1 (FreeCAD bundle) and 7.9.3 (workflow CI) references are historical",
                    "reason": "official release ships headers, import libraries, DLLs and CMake config; C++17 baseline; 8.0.1 carries the modelling/meshing/healing fixes the gate needs",
                    "pin": "native/occt/pin.json",
                }
            ]
            write(l3_path, l3)

    # 2. Task intent strings that named 7.8.1 / 7.9.3 become 8.0.1 + pin.
    print("\n[2] rewrite version strings in the gate L4 records")
    changed_total = 0
    for l4_path in sorted((M00 / "L2-02 - OCCT").rglob("l4.json")):
        data = load(l4_path)
        changed: list[str] = []
        updated = rewrite_strings(data, changed)
        if changed:
            changed_total += len(changed)
            print(f"  {l4_path.parent.name}: {len(changed)} string(s)")
            if args.apply:
                write(l4_path, updated)
    print(f"  rewritten: {changed_total}")

    # 3. M00 L1 risk item: the version reconciliation is now decided.
    print("\n[3] M00 L1 known_risks: record the resolution")
    l1_path = M00 / "L1.json"
    l1 = load(l1_path)
    old = "Some documentation pins OCCT 7.8.1 while workflows reference 7.9.3: reconcile by platform/revision"
    new = ("RESOLVED 2026-10-17: OCCT 8.0.1 is the single pinned kernel (native/occt/pin.json, tag V8_0_1); "
           "the 7.8.1 FreeCAD bundle and the 7.9.3 workflow references are historical and must not be used")
    if args.apply:
        # The list lives under a nested key; rewrite wherever the exact item appears.
        def walk(value):
            if isinstance(value, list):
                return [new if item == old else walk(item) for item in value]
            if isinstance(value, dict):
                return {key: walk(item) for key, item in value.items()}
            return value
        l1 = walk(l1)
        write(l1_path, l1)
        print(f"  rewritten: {old[:48]}…")

    # 4. M01 L2-01-05 and M03 OCCT modules reference the same pin.
    print("\n[4] M01 L2-01-05 + M03 OCCT modules: kernel_pin")
    targets = [next(M01.glob("L2-01-05*")) / "L2.json"]
    targets += [path / "L2.json" for path in sorted(M03.glob("L2-01-0[9]*"))]
    targets += [path / "L2.json" for path in sorted(M03.glob("L2-01-1[01]*"))]
    for path in targets:
        data = load(path)
        print(f"  {data['id']}")
        if args.apply:
            data["kernel_pin"] = {k: PIN[k] for k in ("version", "tag", "commit", "pin_file")}
            write(path, data)

    if not args.apply:
        print("\ndry run only; pass --apply to write")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
