"""Render the L3 ``sdk_export_plan`` annotations into one reviewable index.

The milestone L3 records carry a planning-only ``sdk_export_plan`` field: which surface
belongs in SDK v1, which is planned, and which stays host-internal. This tool reads those
fields (it never writes the milestones) and prints a Markdown table so the decision can be
reviewed without opening dozens of JSON files.

Usage:
    python tools/l3_sdk_export_plan.py            # print Markdown to stdout
    python tools/l3_sdk_export_plan.py --output docs/validation/2026-10-21-l3-sdk-export-plan.md
"""

from __future__ import annotations

import argparse
import json
from collections import Counter
from pathlib import Path

STATUS_ORDER = ("exported_in_sdk_v1", "planned", "not_in_sdk")


def collect(root: Path) -> list[dict]:
    rows: list[dict] = []
    for path in sorted(root.glob("OM9_Milestones/*/*/L3-*/l3.json")):
        doc = json.loads(path.read_text(encoding="utf-8"))
        plan = doc.get("sdk_export_plan")
        if not plan:
            continue
        rows.append(
            {
                "id": doc["id"],
                "title": doc.get("title", ""),
                "owner": path.parent.parent.name,
                "milestone": path.parent.parent.parent.parent.name,
                "status": plan["status"],
                "api": plan.get("api", []),
                "note": plan.get("note", ""),
                "candidate_api": plan.get("candidate_api", []),
                "host_only": plan.get("host_only", ""),
            }
        )
    return rows


def render(rows: list[dict]) -> str:
    counts = Counter(row["status"] for row in rows)
    lines = [
        "# L3 SDK export plan index",
        "",
        "Planning only: the milestone L3 records decide which surface belongs in SDK v1.",
        "This index is generated from those records; edit the L3 JSON, not this file.",
        "",
        "| status | count | meaning |",
        "|---|---|---|",
        f"| `exported_in_sdk_v1` | {counts['exported_in_sdk_v1']} | Delivered, with a measured receipt. |",
        f"| `planned` | {counts['planned']} | Needed by plugins, not delivered yet. |",
        f"| `not_in_sdk` | {counts['not_in_sdk']} | Host-internal codec/state/fixtures. |",
        "",
        f"Total annotated L3: **{len(rows)}** across {len({row['owner'] for row in rows})} owners.",
        "",
    ]
    by_owner: dict[str, list[dict]] = {}
    for row in rows:
        by_owner.setdefault(row["owner"], []).append(row)
    for owner in sorted(by_owner):
        lines.append(f"## {owner}")
        lines.append("")
        lines.append("| L3 | title | status | SDK surface / note |")
        lines.append("|---|---|---|---|")
        for row in sorted(by_owner[owner], key=lambda r: r["id"]):
            if row["api"]:
                detail = ", ".join(f"`{api}`" for api in row["api"])
            elif row["candidate_api"]:
                detail = "đề xuất: " + ", ".join(f"`{api}`" for api in row["candidate_api"])
                if row["host_only"]:
                    detail += " — " + row["host_only"]
            else:
                detail = row["note"]
            lines.append(f"| `{row['id']}` | {row['title']} | `{row['status']}` | {detail} |")
        lines.append("")
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path("."))
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    text = render(collect(args.root.resolve()))
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(text, encoding="utf-8", newline="\n")
        print(f"wrote {args.output}")
    else:
        print(text)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
