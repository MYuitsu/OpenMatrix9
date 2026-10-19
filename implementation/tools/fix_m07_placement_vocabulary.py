#!/usr/bin/env python
"""Follow-up: make M07's hosted placement use the tree's own vocabulary.

`l4_placed_by_milestone` (M08) answers "where do *my* L4 records physically sit", so M07 writing
`{"M01": 4}` claimed the opposite of reality. The host side is `hosted_l4_from` (M03 already uses it
for M08's 44 records), so M07 records the four delegated L4 records that way and keeps its own
logical count separate from the physical tree count.
"""
from __future__ import annotations

import json
import pathlib

ROOT = pathlib.Path(__file__).resolve().parents[2]
M07 = ROOT / "OM9_Milestones" / "M07 - Cutters" / "L1.json"

data = json.loads(M07.read_text(encoding="utf-8"))
data.pop("l4_placed_by_milestone", None)
data.pop("l3_placed_by_milestone", None)
data["l4_count"] = 65
data["l4_count_in_tree"] = 69
data["hosted_l4_from"] = {"M01": 4}
data["delegated_l3_from_milestone"] = {"OM9-L3-01-12-008": "M01"}
data["cross_milestone_note"] = (
    "M01 owns OM9-L2-01-12 and OM9-L3-01-12-008, including their ids; M07 executes the delegated "
    "STL export queue as four L4 placement records under placements/OM9-01-12-008. M07's own L4 "
    "count is 65; the tree holds 69 because those four are hosted here."
)
M07.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
print("M07 L1:", {key: data[key] for key in ("l4_count", "l4_count_in_tree", "hosted_l4_from")})
