#!/usr/bin/env python
"""Regenerate every M00 gate record from one status engine.

The problem this solves: M00 status used to be written by hand in four places (`L1.gate_status`, the
gate registry, each `STATUS-SUMMARY.md` and the dashboard), so they drifted. The rule now is:

    spec      OM9_Milestones/M00/**            never holds execution status (only planning_state)
    computed  this script, from the tree + implementation/evidence + review records
    curated   implementation/evidence/M00/gate-notes.json   facts and next actions only a human adds
    derived   G00-INDEX.json, G00-0X/STATUS-SUMMARY.md, L1.gate_status, MILESTONE_DASHBOARD.md

`computed` reuses the dashboard's own functions, so the registry and the dashboard can never disagree.

    python implementation/tools/update_m00_gates.py            # print the table
    python implementation/tools/update_m00_gates.py --write    # rewrite the records
"""
from __future__ import annotations

import argparse
import datetime
import importlib.util
import json
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
TREE = ROOT / "OM9_Milestones"
M00 = TREE / "M00 - Feasibility, Legal"
EVIDENCE = ROOT / "implementation/evidence/M00"
NOTES = EVIDENCE / "gate-notes.json"
REGISTRY = EVIDENCE / "G00-INDEX.json"


def load_dashboard():
    path = ROOT / "tools/om9_milestone_dashboard.py"
    spec = importlib.util.spec_from_file_location("om9_dashboard", path)
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    # dataclasses resolve `cls.__module__` through sys.modules, so register the module first.
    sys.modules["om9_dashboard"] = module
    spec.loader.exec_module(module)
    return module



def l4_artefacts() -> dict[str, int]:
    """Count files that name an L4 id, across evidence, design and fixtures.

    Two naming schemes exist in M00: `OM9-M00-L4-02-01-01.*` and the older `L4-01-01-01.evidence.json`.
    Both are counted here so the registry can separate "an artefact exists" from "acceptance evidence
    exists" (the dashboard's stricter rule).
    """
    pattern = re.compile(r"L4-(\d{2})-(\d{2})-(\d{2})")
    counts: dict[str, int] = {}
    bases = [
        EVIDENCE,
        ROOT / "implementation/design" / "M00",
        ROOT / "implementation/fixtures" / "M00",
    ]
    for base in bases:
        if not base.is_dir():
            continue
        for path in base.rglob("*"):
            if not path.is_file():
                continue
            match = pattern.search(path.name)
            if match is None:
                continue
            short = "L4-" + "-".join(match.groups())
            counts[short] = counts.get(short, 0) + 1
    return counts



def closeout_counts(gate: str) -> tuple[int, int] | None:
    """Read the evidenced/open table of a gate closeout, which states each L4 explicitly.

    The dashboard's per-bucket rule is coarser: one `source_crosswalk.json` covers every `-01` role in the
    bucket and one `api.md` covers every `-02`, so a bucket can report more L4 than are actually proven.
    When a closeout exists its table wins, and the dashboard number is kept as `l4_dashboard` for audit.
    """
    path = EVIDENCE / gate / "GATE-CLOSEOUT.md"
    if not path.exists():
        return None
    evidenced = 0
    total = 0
    for line in path.read_text(encoding="utf-8").splitlines():
        if not line.startswith("| L4-"):
            continue
        total += 1
        if "EVIDENCED" in line:
            evidenced += 1
    return (evidenced, total) if total else None


def gate_status(signed: int, total_l3: int, evidenced: int) -> str:
    if total_l3 > 0 and signed == total_l3:
        return "CLOSED"
    if signed > 0:
        return "PARTIAL"
    if evidenced > 0:
        return "IN_PROGRESS"
    return "NOT_STARTED"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    dashboard = load_dashboard()
    milestones, _ = dashboard.build()
    m00 = next(m for m in milestones if m.key == "M00")
    notes = json.loads(NOTES.read_text(encoding="utf-8")).get("gates", {}) if NOTES.exists() else {}
    as_of = datetime.datetime.now(datetime.UTC).strftime("%Y-%m-%dT%H:%M:%SZ")

    gates = []
    for module in m00.modules:
        l2_path = module.path
        l2 = json.loads(l2_path.read_text(encoding="utf-8"))
        gate = l2.get("gate_id")
        if not gate:
            continue
        signed = sum(1 for l3 in module.children if l3.status == dashboard.DONE)
        evidenced = sum(1 for l3 in module.children if l3.l4_evidence > 0)
        l4_total = 0
        for l3 in module.children:
            l3_document = json.loads(l3.path.read_text(encoding="utf-8"))
            l4_total += len(l3_document.get("children_l4") or l3_document.get("task_ids") or [])
        l4_evidenced = sum(l3.l4_evidence for l3 in module.children)
        l4_dashboard = l4_evidenced
        closeout = closeout_counts(gate)
        if closeout is not None:
            l4_evidenced, l4_total = closeout
        artefacts = l4_artefacts()
        l4_artefacts_count = 0
        for l3 in module.children:
            l3_document = json.loads(l3.path.read_text(encoding="utf-8"))
            for l4 in l3_document.get("children_l4") or []:
                short = str(l4.get("id", "")).replace("OM9-M00-", "")
                l4_artefacts_count += 1 if artefacts.get(short) else 0
        note = notes.get(gate, {})
        entry = {
            "gate": gate,
            "l2": l2.get("id"),
            "title": l2.get("title"),
            "status": gate_status(signed, len(module.children), l4_evidenced),
            "l3_total": len(module.children),
            "l3_signed": signed,
            "l4_total": l4_total,
            "l4_evidenced": l4_evidenced,
            "l4_dashboard": l4_dashboard,
            "l4_artefacts": l4_artefacts_count,
            "l3": [
                {
                    "id": l3.id,
                    "title": l3.title,
                    "status": l3.status,
                    "l4_evidenced": l3.l4_evidence,
                    "signed_by": l3.signed_by,
                }
                for l3 in module.children
            ],
            "evidence_roots": sorted(
                {
                    f"implementation/evidence/M00/{l3.id}"
                    for l3 in module.children
                    if (EVIDENCE / l3.id).is_dir()
                }
            ),
            "verified_facts": note.get("verified_facts", []),
            "blockers": note.get("blockers", []),
            "next": note.get("next", ""),
            "human_approval_required": note.get("human_approval_required", []),
        }
        if entry["status"] == "CLOSED":
            entry["closed_at_utc"] = note.get("closed_at_utc") or as_of
            entry["closed_by"] = note.get("closed_by", "")
        gates.append(entry)

    registry = {
        "schema": "om9.m00.gate_registry.v2",
        "milestone": "M00",
        "generated_by": "implementation/tools/update_m00_gates.py",
        "as_of_utc": as_of,
        "source_of_truth": "implementation/design/M00/M00-SOURCE-OF-TRUTH.md",
        "rule": "status is computed from implementation/evidence + review records; facts and next actions come from gate-notes.json",
        "totals": {
            "gates": len(gates),
            "closed": sum(1 for gate in gates if gate["status"] == "CLOSED"),
            "l4_total": sum(gate["l4_total"] for gate in gates),
            "l4_evidenced": sum(gate["l4_evidenced"] for gate in gates),
        },
        "gates": gates,
    }

    print(f"{'gate':7} {'status':12} {'L3 signed':10} {'L4':10}")
    for gate in gates:
        print(
            f"{gate['gate']:7} {gate['status']:12} "
            f"{gate['l3_signed']}/{gate['l3_total']:<8} evidence {gate['l4_evidenced']}/{gate['l4_total']}  "
            f"artefacts {gate['l4_artefacts']}"
        )

    if args.write:
        REGISTRY.write_text(json.dumps(registry, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
        for gate in gates:
            folder = EVIDENCE / gate["gate"]
            folder.mkdir(parents=True, exist_ok=True)
            lines = [
                f"# {gate['gate']} — {gate['title']}",
                "",
                f"Trạng thái: **{gate['status']}** · sinh bởi `implementation/tools/update_m00_gates.py` lúc {as_of}",
                f"L3 đã ký: {gate['l3_signed']}/{gate['l3_total']} · L4 có bằng chứng nghiệm thu: {gate['l4_evidenced']}/{gate['l4_total']} · L4 có artefact: {gate['l4_artefacts']}/{gate['l4_total']}",
                "",
                "## L3",
                "",
                "| L3 | Trạng thái | L4 | Ký bởi |",
                "| --- | --- | ---: | --- |",
            ]
            for l3 in gate["l3"]:
                lines.append(
                    f"| `{l3['id']}` {l3['title']} | {l3['status']} | {l3['l4_evidenced']} | {l3['signed_by'] or '—'} |"
                )
            if gate["verified_facts"]:
                lines += ["", "## Sự thật đã kiểm", ""] + [f"- {fact}" for fact in gate["verified_facts"]]
            if gate["blockers"]:
                lines += ["", "## Còn chặn", ""] + [f"- {blocker}" for blocker in gate["blockers"]]
            if gate["next"]:
                lines += ["", "## Việc tiếp theo", "", f"- {gate['next']}"]
            (folder / "STATUS-SUMMARY.md").write_text("\n".join(lines) + "\n", encoding="utf-8")
        l1_path = M00 / "L1.json"
        l1 = json.loads(l1_path.read_text(encoding="utf-8"))
        l1["gate_status"] = {
            gate["gate"]: {
                key: value
                for key, value in gate.items()
                if key in {"status", "l3_total", "l3_signed", "l4_total", "l4_evidenced", "closed_at_utc", "closed_by", "next"}
            }
            for gate in gates
        }
        l1["gate_registry"] = "implementation/evidence/M00/G00-INDEX.json"
        l1_path.write_text(json.dumps(l1, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
        print(f"\nwrote {REGISTRY.relative_to(ROOT)}, the STATUS-SUMMARY files and L1.gate_status")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
