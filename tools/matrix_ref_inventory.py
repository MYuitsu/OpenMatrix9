"""Inventory L2/L3/L4 records of M00-M02 and the private Matrix9 reference exports.

Read-only: writes a JSON inventory under build/ for crosswalk authoring.
Does not copy reference source content into the repository.
"""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MILESTONES = ROOT / "OM9_Milestones"
TARGETS = ("M00", "M01", "M02", "M03", "M04", "M05", "M06", "M07")
OUT = ROOT / "build" / "matrix-ref-inventory.json"


def load(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def milestone_dir(m):
    for d in MILESTONES.iterdir():
        if d.is_dir() and d.name.startswith(m + " "):
            return d
    raise SystemExit("missing milestone " + m)


def main():
    l4_rows = []
    l2_rows = []
    l3_rows = []
    for m in TARGETS:
        root = milestone_dir(m)
        for l2dir in sorted(p for p in root.iterdir() if p.is_dir()):
            l2file = l2dir / "L2.json"
            if l2file.is_file():
                data = load(l2file)
                definition = data.get("definition", {})
                l2_rows.append({
                    "id": data.get("id"),
                    "milestone": m,
                    "title": data.get("title"),
                    "scope": data.get("scope"),
                    "original_l1": definition.get("original_l1"),
                    "manual_references": definition.get("manual_references", []),
                    "l3_children": [c["id"] for c in data.get("children_l3", [])],
                })
            for l3dir in sorted(p for p in l2dir.iterdir() if p.is_dir()):
                l3file = l3dir / "l3.json"
                spec = {}
                if l3file.is_file():
                    data = load(l3file)
                    spec = data.get("specification", {})
                    l3_rows.append({
                        "id": data.get("id"),
                        "parent_l2_id": data.get("parent_l2_id"),
                        "milestone": m,
                        "title": data.get("title"),
                        "goal": spec.get("goal"),
                        "capability_id": spec.get("capability_id"),
                        "tag": spec.get("tag"),
                        "origin": spec.get("origin_source_taxonomy"),
                        "origin_manual_refs": spec.get("origin_manual_refs", []),
                        "l4_children": [c["id"] for c in data.get("children_l4", [])],
                    })
                for l4dir in sorted(p for p in l3dir.iterdir() if p.is_dir()):
                    l4file = l4dir / "l4.json"
                    if not l4file.is_file():
                        continue
                    data = load(l4file)
                    task = data.get("task", {})
                    l4_rows.append({
                        "file": l4file.relative_to(ROOT).as_posix(),
                        "id": data.get("id"),
                        "milestone": m,
                        "parent_l3_id": data.get("parent_l3_id"),
                        "owner_l2_id": data.get("owner_l2_id") or data.get("parent_l2_id"),
                        "dir": l4dir.name,
                        "title": data.get("title"),
                        "role": task.get("role") or task.get("work_kind") or data.get("task_role"),
                        "purpose": task.get("purpose") or task.get("goal") or data.get("scope_goal"),
                        "symbol_hints": task.get("candidate_symbol_hints_require_local_verification"),
                    })
    payload = {
        "schema_version": 1,
        "counts": {"l2": len(l2_rows), "l3": len(l3_rows), "l4": len(l4_rows)},
        "l2": l2_rows,
        "l3": l3_rows,
        "l4": l4_rows,
    }
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(payload, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")
    print(json.dumps(payload["counts"]))
    for m in TARGETS:
        print(m, sum(1 for r in l4_rows if r["milestone"] == m))
    for r in l3_rows:
        print(f"{r['id']}\t{r['parent_l2_id']}\t{r['title']}")


if __name__ == "__main__":
    main()
