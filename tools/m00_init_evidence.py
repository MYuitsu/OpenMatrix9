#!/usr/bin/env python
"""Create a NOT_RUN draft evidence record for one M00 L4.

    python tools/m00_init_evidence.py OM9-M00-L4-01-01-02

The draft satisfies the v0.13 shape but claims nothing: status NOT_RUN, empty actual, PENDING review.
`validate_m00_execution.py` accepts the shape and refuses the claim, which is the point.
"""
from __future__ import annotations

import json
import sys
import pathlib

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import m00_common as m00


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: m00_init_evidence.py <OM9-M00-L4-xx-xx-xx>")
        return 2
    task = sys.argv[1]
    entry = next((item for item in m00.iter_l4() if item["id"] == task), None)
    if entry is None:
        print(f"{task} is not an M00 L4")
        return 2
    folder = m00.EVIDENCE / entry["gate"]
    folder.mkdir(parents=True, exist_ok=True)
    path = folder / f"{task}.evidence.json"
    if path.exists():
        print(f"{path.relative_to(m00.ROOT)} already exists; refusing to overwrite")
        return 1
    document = {
        "task_id": task,
        "source_sha": m00.QYMCAD_SHA,
        "status": "NOT_RUN",
        "actual_execution": False,
        "timestamp_utc": None,
        "os_arch": None,
        "toolchain": {},
        "commands": [],
        "artifacts": [],
        "fixtures": [],
        "expected": [],
        "actual": [],
        "review": {"decision": "PENDING", "reviewer": None, "evidence_checked": False, "signed_record_path": None},
        "blockers": [],
        "note": "draft created by m00_init_evidence.py; not a PASS",
    }
    path.write_text(json.dumps(document, indent=2) + "\n", encoding="utf-8")
    print(f"wrote {path.relative_to(m00.ROOT)} (NOT_RUN draft)")
    return 0
