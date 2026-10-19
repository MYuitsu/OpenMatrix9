#!/usr/bin/env python
"""Move the M00 closeout artefacts into the per-L3 folders the dashboard reads.

The dashboard buckets evidence by a directory named exactly like the L3 id, and merges
`implementation/{evidence,design,fixtures}/M00/<L3>`. The closeout script writes gate-level documents, so
this pass distributes them and adds the two artefacts the counting rules need:

* `-01` role: a `source_crosswalk.json` in the L3 bucket;
* `-02` role: `api.md` plus `positive.json`/`negative.json` in the L3 bucket.

Only evidenced L4s are listed in a crosswalk, so an OPEN L4 stays uncounted on purpose.
"""
from __future__ import annotations

import json
import pathlib
import re
import shutil

ROOT = pathlib.Path(__file__).resolve().parents[2]
EVIDENCE = ROOT / "implementation/evidence/M00"
DESIGN = ROOT / "implementation/design/M00"
FIXTURES = ROOT / "implementation/fixtures/M00"
GATES = ("G00-04", "G00-05", "G00-06", "G00-07")
L4 = re.compile(r"(OM9-M00-L4-(\d{2})-(\d{2})-(\d{2}))")


def l3_of(l4_id: str) -> str:
    match = L4.match(l4_id)
    assert match is not None, l4_id
    return f"OM9-M00-L3-{match.group(2)}-{match.group(3)}"


def main() -> int:
    # 1. move the per-L4 notes next to their L3
    moved = 0
    for gate in GATES:
        for note in sorted((EVIDENCE / gate).glob("OM9-M00-L4-*.note.md")):
            l3 = l3_of(note.name.split(".")[0])
            target = EVIDENCE / l3
            target.mkdir(parents=True, exist_ok=True)
            shutil.move(str(note), str(target / note.name))
            moved += 1
    print("notes moved:", moved)

    # 2. per L3: crosswalk (if the -01 L4 is evidenced) and api/fixtures (if the -02 L4 is evidenced)
    for gate in GATES:
        for l3_dir in sorted((ROOT / "OM9_Milestones/M00 - Feasibility, Legal").glob(f"L2-{gate[-2:]}-*")):
            document = json.loads((l3_dir / "L2.json").read_text(encoding="utf-8"))
            for child in document.get("children_l3", []):
                l3_id = child.get("id")
                path = EVIDENCE / l3_id
                if not path.is_dir():
                    continue
                evidenced = sorted(p.name.split(".")[0] for p in path.glob("OM9-M00-L4-*.note.md"))
                # newline-free JSON so the crosswalk stays readable
                crosswalk = {
                    "schema": "om9.source_crosswalk.v1",
                    "l3_id": l3_id,
                    "entries": [
                        {
                            "l4": l4,
                            "evidence": f"implementation/evidence/M00/{l3_id}/{l4}.note.md",
                        }
                        for l4 in evidenced
                    ],
                    "open": "L4 without an evidence file here is OPEN; see implementation/evidence/M00/"
                    + gate
                    + "/GATE-CLOSEOUT.md",
                }
                (path / "source_crosswalk.json").write_text(
                    json.dumps(crosswalk, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
                )
                design = DESIGN / l3_id
                design.mkdir(parents=True, exist_ok=True)
                source_api = DESIGN / gate / "api.md"
                if source_api.exists() and not (design / "api.md").exists():
                    shutil.copy2(source_api, design / "api.md")
                fixtures = FIXTURES / l3_id
                fixtures.mkdir(parents=True, exist_ok=True)
                for name in ("positive.json", "negative.json"):
                    source = FIXTURES / gate / name
                    if source.exists() and not (fixtures / name).exists():
                        shutil.copy2(source, fixtures / name)
                print(f"{l3_id}: {len(evidenced)} evidenced L4")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
