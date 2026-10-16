"""Regression guard: a supplements refresh must not destroy another runbook section.

This is the bug that was found on 2026-10-16: strip_applied cut the runbook at the
reference heading, so re-running the supplements applier deleted the M01/M02
code-sample section that had been appended after it. The guard asserts the property
directly on the strip helper, so it fails even when no M01/M02 record is present.

It also asserts the ordering contract: build_supplements never emits the rhino5 key
for an L3 that has no Rhino 5 anchor, so unaffected records are not rewritten.
"""
import json
import sys
from pathlib import Path

ROOT = Path("H:/OpenMatrix9")
sys.path.insert(0, str(ROOT / "tools"))
import apply_matrix_ref_supplements as A  # noqa: E402
import matrix_ref_crosswalk as cw  # noqa: E402

SAMPLE = (
    "## 7. Nghiệm thu và dừng\n\nnội dung cũ\n\n"
    f"{A.RUNBOOK_HEADING}\n\nphần tham chiếu phải bị thay\n\n"
    "## Code mẫu / thuật toán mẫu (sketch, không phải bằng chứng)\n\n"
    "```rust\nfn om9_x() {}\n```\n"
)

checks = []
problems = []

out = A.strip_reference_section(SAMPLE)
checks.append(("reference heading removed", A.RUNBOOK_HEADING not in out))
checks.append(("following section kept", "## Code mẫu" in out))
checks.append(("earlier section kept", "## 7. Nghiệm thu" in out))
checks.append(("earlier body kept", "nội dung cũ" in out))
checks.append(("reference body dropped", "phần tham chiếu phải bị thay" not in out))

# when the reference section is last, the tail must still be trimmed safely
tail_only = "## 1. a\n\nbody\n\n" + A.RUNBOOK_HEADING + "\n\nref body\n"
out2 = A.strip_reference_section(tail_only)
checks.append(("tail reference removed", A.RUNBOOK_HEADING not in out2))
checks.append(("tail earlier section survives", "## 1. a" in out2))

# a runbook without the section is returned unchanged
plain = "## 1. a\n\nbody\n"
checks.append(("no-op when section absent", A.strip_reference_section(plain) == plain))

# the rhino5 key is only emitted when the L3 actually carries an anchor
with_anchor = [lid for lid, row in cw.L3_ANCHORS.items() if row.get("rhino5")]
without = [lid for lid, row in cw.L3_ANCHORS.items() if not row.get("rhino5")]
if with_anchor:
    has = A.build_supplements(with_anchor[0])
    checks.append(("rhino5 key present when anchored",
                   "rhino5_geometry_anchors" in has))
if without:
    absent = A.build_supplements(without[0])
    if absent is not None:
        checks.append(("rhino5 key absent when unanchored",
                       "rhino5_geometry_anchors" not in absent))

for label, ok in checks:
    if not ok:
        problems.append(label)

print(json.dumps({"checks": [{"check": c, "ok": o} for c, o in checks],
                  "problems": problems,
                  "l3_with_rhino5": len(with_anchor),
                  "l3_without_rhino5": len(without)},
                 ensure_ascii=False, indent=1))
if problems:
    sys.exit(1)
