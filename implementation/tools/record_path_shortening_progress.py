"""Ghi mot muc vao live ledger docs/openmatrix9-progress.json cho cong viec
rut ngan duong dan OM9_Milestones.

Chen o muc TEXT (khong dump lai ca file) de giu nguyen dinh dang CRLF/indent
cua 596 KB ledger.
"""

from __future__ import annotations

import json
import os
import sys

ROOT = r"H:\OpenMatrix9"
LEDGER = os.path.join(ROOT, "docs", "openmatrix9-progress.json")
INDEX = os.path.join(ROOT, "OM9_Milestones", "PATH_RENAMES.json")
REPORT = os.path.join(ROOT, "implementation", "design", "om9-milestone-path-shortening.md")

ITEM_ID = "om9-milestones-path-shortening-maxpath-2026-10-16"


def find_array_end(text: str, start: int) -> int:
    depth = 0
    i = start
    in_str = False
    esc = False
    while i < len(text):
        ch = text[i]
        if in_str:
            if esc:
                esc = False
            elif ch == "\\":
                esc = True
            elif ch == '"':
                in_str = False
        elif ch == '"':
            in_str = True
        elif ch == "[":
            depth += 1
        elif ch == "]":
            depth -= 1
            if depth == 0:
                return i
        i += 1
    raise ValueError("items array khong can bang")


def build_item() -> dict:
    index = json.load(open(INDEX, encoding="utf-8"))
    applied = [row for row in index["modules"] if row.get("applied")]
    blocked = [row for row in index["modules"] if not row.get("applied")]
    return {
        "id": ITEM_ID,
        "date": "2026-10-16",
        "status": "applied_verified_two_modules_blocked_by_windows_handle",
        "scope": (
            "OM9_Milestones path budget (Windows MAX_PATH) + dong bo tham chieu trong "
            "tools/ va ledger. Khong nang planning_state/gate nao."
        ),
        "why": (
            "Nguoi dung bao loi 'Filename too long' cho "
            "OM9_Milestones/M00 .../L2-04 .../L3-03 .../L4-01 .../l4.json (242 ky tu) va yeu cau "
            "rut ngan cac duong dan qua dai nhung khong mat thong tin."
        ),
        "source_audit": {
            "measured_before": {
                "files": 11881,
                "max_path": 249,
                "files_ge_240": 214,
                "files_ge_230": 889,
                "files_ge_220": 1815,
                "longest_workspace_path": "build/m03-incoming/OpenMatrix9/... = 256",
            },
            "measured_after": {
                "files": 11883,
                "max_path": 210,
                "files_over_210": 0,
                "json_parse_errors": 0,
                "dangling_references_real": 0,
            },
            "root_cause": (
                "Ten thu muc dat theo title: module M01/M02 dai 73/76 ky tu, L2 toi 63, L3 toi 65, "
                "cong reference/text (14) va ten file toi 42. Duong dan chi 242 ky tu nen loi chi xay ra "
                "khi co tien to cong them (staging build/mNN-incoming +31, worktree agent +~50)."
            ),
            "no_information_lost": (
                "Tieu de day du cua moi cap van nam trong L1.json / L2.json / l3.json / l4.json; "
                "OM9_Milestones/PATH_RENAMES.{json,md} ghi lai anh xa ten cu -> ten moi."
            ),
        },
        "code_added": [
            "implementation/tools/om9_shorten_milestone_paths.py",
            "implementation/tools/test_om9_shorten_milestone_paths.py",
            "OM9_Milestones/PATH_RENAMES.json",
            "OM9_Milestones/PATH_RENAMES.md",
            "implementation/design/om9-milestone-path-shortening.md",
        ],
        "validation_commands": [
            "python implementation/tools/test_om9_shorten_milestone_paths.py -> PASS (doi ten long nhau, idempotent, tu hoan tac khi loi)",
            "python implementation/tools/om9_shorten_milestone_paths.py --apply -> VERIFY: PASS (max 210, 0 file qua ngan sach, 0 JSON loi, 0 tham chieu treo that, ledger JSON hop le)",
            "python tools/validate_implementation_samples.py -> exit 0, errors: []",
            "python tools/validate_m09_l4.py -> exit 0, PASS 30 L4 / 177 cases",
        ],
        "behaviour_changes_pending_consent": [],
        "gate_state": {
            "promoted": "none",
            "note": "Chi doi ten thu muc/duong dan va dong bo tham chieu; khong doi planning_state hay gate.",
        },
        "not_yet_done": [
            "M00 - Feasibility, QymCAD Source, OCCT, Legal and Environment chua doi duoc ten module (Windows giu handle) -> da bu bang cach cat 18 thu muc con.",
            "M03 - Curve, Surface, Solid and 3DM Exchange chua doi duoc ten module (Windows giu handle) -> da bu bang cach cat cac thu muc con.",
            "Sau khi giai phong handle: python implementation/tools/om9_shorten_milestone_paths.py --finish-module \"<ten cu>=<ten moi>\"",
            "Chua git add/commit: cay dang o trang thai rename chua stage.",
        ],
        "final_reconciliation": {
            "modules_renamed": [row["old"] + " -> " + row["new"] for row in applied],
            "modules_blocked": [row["old"] for row in blocked],
            "inner_dirs_renamed": len(index["dirs"]),
            "budget": index["budget"],
            "report": "implementation/design/om9-milestone-path-shortening.md",
        },
        "next_action": (
            "Dong moi handle dang mo tren M00/M03 (khoi dong lai phien/editor), chay --finish-module cho hai "
            "module nay, roi git add -A OM9_Milestones tools va review diff rename."
        ),
        "artefacts": [
            "OM9_Milestones/PATH_RENAMES.json",
            "OM9_Milestones/PATH_RENAMES.md",
            "implementation/design/om9-milestone-path-shortening.md",
            "implementation/tools/om9_path_plan.json",
        ],
    }


def main() -> int:
    text = open(LEDGER, "rb").read().decode("utf-8")
    data = json.loads(text)
    if any(item.get("id") == ITEM_ID for item in data["items"]):
        print("da co muc nay trong ledger, khong chen lai")
        return 0
    marker = '"items": ['
    start = text.index(marker) + len(marker) - 1
    end = find_array_end(text, start)
    head = text[:end]
    stripped = head.rstrip(" \r\n\t")
    tail_ws = head[len(stripped):]
    blob = json.dumps(build_item(), ensure_ascii=False, indent=2)
    blob = "\r\n".join("  " + line for line in blob.split("\n"))
    new_text = stripped + ",\r\n" + blob + tail_ws + text[end:]
    parsed = json.loads(new_text)
    if len(parsed["items"]) != len(data["items"]) + 1 or parsed["items"][-1]["id"] != ITEM_ID:
        print("KIEM TRA THAT BAI: muc moi khong nam cuoi items")
        return 2
    open(LEDGER, "wb").write(new_text.encode("utf-8"))
    print(f"ledger updated: items {len(data['items'])} -> {len(parsed['items'])} (id={ITEM_ID})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
