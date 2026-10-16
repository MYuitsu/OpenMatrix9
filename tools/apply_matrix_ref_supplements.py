"""Apply the M00-M02 reference crosswalk to the L4 task records.

Only reference anchors, provenance metadata and verification instructions are
written. No reference source body, image or binary is copied, and no status is
promoted: every record keeps ``requirements_only_not_runtime_evidence``.

The JSON files are edited as text so the existing key order, indentation and
every other field stay byte-identical; the edit is idempotent.
"""

import argparse
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import matrix_ref_crosswalk as cw  # noqa: E402

ROOT = Path(__file__).resolve().parents[1]
INVENTORY = ROOT / "build" / "matrix-ref-inventory.json"
RHINO_INDEX_REL = "ref/rhino5/CORE_REFERENCE_INDEX.json"
SHEET_INDEX = ROOT / "build" / "matrix-ref-sources.json"


def load_sheet_paths() -> dict:
    """Map authored reference-sheet id -> milestone spec path."""
    if not SHEET_INDEX.is_file():
        return {}
    data = json.loads(SHEET_INDEX.read_text(encoding="utf-8"))
    paths = {}
    for row in data.get("milestone_reference_sheets", []):
        paths.setdefault(row["sheet"], row["path"])
    return paths


SHEET_PATHS = load_sheet_paths()

MATRIX_SOURCE_IDS = {
    "ref/matrix/decompiled-ui/export-manifest.json": "mx-export-manifest",
    "ref/matrix/vb6-lite/source-manifest.json": "mx-vb6-manifest",
}

REF_KEY = "reference_supplements"
POLICY_FIELD = "reference_source_policy"
RUNBOOK_HEADING = "## Nguồn tham chiếu bắt buộc đối chiếu (Matrix9 / Rhino 5) — v0.16"


def rcore_entry(chapter_id: str) -> dict:
    chapter = cw.RCORE[chapter_id]
    return {
        "anchor_id": chapter_id,
        "kind": "rhino5_core_chapter",
        "title": chapter["title"],
        "spec_source": RHINO_INDEX_REL,
        "guide_pdf_pages": chapter["guide_pdf"],
        "capability_ids": chapter["capabilities"],
        "fixture_ids": chapter["fixtures"],
        "official_help": [{"label": label, "url": url} for label, url in chapter["help"]],
        "verification_state": "INDEX_READ_NOT_EXECUTED",
        "index_runtime_snapshot": "2026-10-09",
    }


def source_root(source_id: str, source: dict) -> str:
    """Resolve the locator root for a recovered Matrix9 reference artifact."""
    project = source["project"]
    if source_id in ("mx-ghidra", "mx-native-inspection"):
        return "ref/matrix"
    if project.startswith("Matrix90") or project.startswith("MatrixCmdBar"):
        return "ref/matrix/vb6-lite"
    if project == "reference export":
        return "ref/matrix"
    return "ref/matrix/decompiled-ui"


def matrix_entry(source_id: str, role: str) -> dict:
    source = cw.MX[source_id]
    return {
        "anchor_id": source_id,
        "kind": "matrix9_recovered_source",
        "source_class": "PRIVATE_REFERENCE_NOT_FOR_PUBLICATION",
        "project": source["project"],
        "path": source["path"],
        "root": source_root(source_id, source),
        "unit": source["unit"],
        "role_for_this_task": role,
        "source_role": source["role"],
        "detail": source["detail"],
        "observed_identifiers": source["observed"],
        "verification_state": source["state"],
    }


def manual_entry(source_id: str, role: str) -> dict:
    sheet = cw.MANUAL[source_id]
    return {
        "anchor_id": source_id,
        "kind": "matrix_manual_reference_sheet",
        "source_class": "IN_REPOSITORY_AUTHORED_REFERENCE_SHEET",
        "manual": sheet["manual"],
        "pages": sheet["pages"],
        "section": sheet["section"],
        "topic": sheet["topic"],
        "sheet_id": sheet["sheet"],
        "path": SHEET_PATHS.get(sheet["sheet"]),
        "role_for_this_task": role,
        "verification_state": "SHEET_PRESENT_IN_MILESTONE_SPECS",
        "note": "Sheet documents the command path, options and stated defaults; it is documentation, "
                "not runtime evidence, and defaults must be re-read from the sheet before use.",
    }


def rhino5_entry(anchor_id: str, role: str) -> dict:
    src = cw.RHINO5[anchor_id]
    return {
        "anchor_id": anchor_id,
        "kind": "rhino5_geometry_source",
        "source_class": "PRIVATE_REFERENCE_NOT_FOR_PUBLICATION",
        "project": src["project"],
        "root": src["root"],
        "path": src["path"],
        "unit": src["unit"],
        "role_for_this_task": role,
        "source_role": src["role"],
        "detail": src["detail"],
        "observed_identifiers": src["observed"],
        "verification_state": src["state"],
        "does_not_establish": [
            "Không chứng minh thuật toán hình học native (RhinoCommon là wrapper P/Invoke mỏng).",
            "Không chứng minh dung sai, giá trị mặc định hay độ chính xác.",
            "Không chứa thuật toán domain của tập lệnh gem.",
        ],
    }


def build_supplements(l3_id: str) -> dict | None:
    row = cw.L3_ANCHORS.get(l3_id)
    if not row:
        return None
    rcore = [rcore_entry(c) for c in row.get("rcore", [])]
    manual = [manual_entry(sid, role) for sid, role in row.get("mx", {}).items()
              if sid in cw.MANUAL]
    matrix = [matrix_entry(sid, role) for sid, role in row.get("mx", {}).items()
              if sid not in cw.MANUAL]
    rhino5 = [rhino5_entry(aid, role) for aid, role in (row.get("rhino5") or {}).items()]
    if not rcore and not matrix and not manual and not rhino5:
        return None
    supplements = {
        "schema_version": cw.SCHEMA_VERSION,
        "policy_id": cw.REFERENCE_POLICY_ID,
        "crosswalk_date": cw.CROSSWALK_DATE,
        "status": cw.REQUIREMENTS_ONLY,
        "focus": row["focus"],
        "rhino5_core_chapters": rcore,
        "matrix9_reference_sources": matrix,
        "manual_reference_sheets": manual,
        "usage_rules": [
            "Chỉ dùng để định vị nguồn và đối chiếu hành vi; không thay thế test/đo runtime.",
            "Không copy mã, ảnh, resource hay nội dung manual vào repository.",
            "Identifier quan sát được chỉ là tên/đường dẫn; symbol phải kiểm lại tại checkout trước khi dùng.",
            "Nguồn Matrix9 là private reference: không đưa path private vào tài liệu public hay gói phát hành.",
            "Sheet manual: đọc lại đúng sheet trước khi dùng option/default; sheet không phải model hay solver.",
            "Nguồn Rhino 5: chỉ là bề mặt type/thao tác và C ABI; không chứng minh thuật toán, "
            "dung sai hay giá trị mặc định của native.",
        ],
        "does_not_establish": [
            "Không chứng minh Matrix9 parity cho capability này.",
            "Không xác nhận API Rust nào đã tồn tại trong QymCAD.",
            "Không nâng trạng thái L4/M00/M01/M02/M04 thành PASS.",
            "Không chứng minh default/option đã đúng cho bản Matrix9 đang cài.",
        ],
    }
    # Only present when this L3 actually uses a Rhino 5 geometry source, so the
    # already-applied records without one are not rewritten for an empty field.
    if rhino5:
        supplements["rhino5_geometry_anchors"] = rhino5
    return supplements


def build_policy() -> dict:
    return {
        "policy_id": cw.REFERENCE_POLICY_ID,
        "reference_families": [
            {
                "family": "rhino5_core",
                "source_id": "ref/rhino5/CORE_REFERENCE_INDEX.json",
                "access_class": "public_official_documentation_user_approved_exception",
                "use": "Định tuyến chương RCORE, dải trang Guide, Help URL chính thức, capability/fixture ID.",
            },
            {
                "family": "matrix9_recovered",
                "source_id": "ref/matrix",
                "access_class": "private_reference_export",
                "projects": ["Matrix (managed plugin)", "MatrixWPFControls", "MatrixControls",
                             "MatrixCommon", "PIMData", "MatrixTranslation",
                             "Matrix90 (VB6 EXE)", "MatrixCmdBar (VB6 OCX)"],
                "use": "Đối chiếu bề mặt UI, tên form/class/control/procedure và export provenance.",
            },
            {
                "family": "rhino5_geometry",
                "source_id": "ref/rhino5/dnspy/Rhino_DotNet + ref/rhino5/dnspy/RhinoCommon + ref/rhino5/ghidra",
                "access_class": "private_reference_export",
                "sources": ["Rhino 5 Rhino_DotNet (RMA.OpenNURBS / RMA.Rhino)",
                            "Rhino 5 RhinoCommon",
                            "Rhino 5 ghidra export"],
                "use": "Bề mặt type/thao tác hình học openNURBS và C ABI native: dùng để ràng buộc "
                       "tên và phân rã thao tác cho lớp FFI/geometry của OM9.",
                "does_not_contain": "Thuật toán domain của tập lệnh gem (prong/pave/halo/azure/"
                                    "milgrain). Những thứ đó nằm ở code Matrix9, không ở đây.",
                "known_limit": "Struct trong listing Ghidra là placeholder: không thu hồi được "
                               "layout member hay offset. RhinoCommon là wrapper P/Invoke mỏng: "
                               "không chứng minh đánh giá NURBS, giao, boolean hay trim.",
            },
        ],
        "forbidden_in_deliverables": [
            "Reference source bodies, decompiler listings, native resource blobs and recovered images.",
            "Private archive paths in public documentation, manifests or release packages.",
        ],
        "status_effect": "NONE_REQUIREMENTS_ONLY",
    }


def render_field(value, indent: str, level: int) -> str:
    """Serialise a value to JSON text with a caller-chosen base indent."""
    if isinstance(value, dict):
        if not value:
            return "{}"
        pad = indent * (level + 1)
        parts = []
        for key, item in value.items():
            parts.append(f"{pad}{json.dumps(key, ensure_ascii=False)}: "
                         f"{render_field(item, indent, level + 1)}")
        return "{\n" + ",\n".join(parts) + "\n" + indent * level + "}"
    if isinstance(value, list):
        if not value:
            return "[]"
        if all(not isinstance(x, (dict, list)) for x in value):
            flat = ", ".join(json.dumps(x, ensure_ascii=False) for x in value)
            if len(flat) + len(indent) * (level + 1) <= 110:
                return "[" + flat + "]"
        pad = indent * (level + 1)
        parts = [f"{pad}{render_field(x, indent, level + 1)}" for x in value]
        return "[\n" + ",\n".join(parts) + "\n" + indent * level + "]"
    if value is None:
        return "null"
    if isinstance(value, bool):
        return "true" if value else "false"
    return json.dumps(value, ensure_ascii=False)


def runbook_block(supplements: dict) -> str:
    chapters = supplements["rhino5_core_chapters"]
    matrix = supplements["matrix9_reference_sources"]
    manual = supplements.get("manual_reference_sheets", [])
    lines = [RUNBOOK_HEADING, "",
             f"**Trạng thái bổ sung:** `{supplements['status']}` — chỉ là yêu cầu đối chiếu, không phải nghiệm thu.",
             "",
             f"**Trọng tâm đối chiếu:** {supplements['focus']}", ""]
    if chapters:
        lines += ["### Chương Rhino 5 core phải đọc trước", ""]
        for chapter in chapters:
            helps = "; ".join(f"[{h['label']}]({h['url']})" for h in chapter["official_help"])
            lines.append(f"- `{chapter['anchor_id']}` — {chapter['title']} · Guide PDF {chapter['guide_pdf_pages']} "
                         f"· fixture đại diện `{', '.join(chapter['fixture_ids'])}`")
            lines.append(f"  - Help chính thức: {helps}")
            lines.append(f"  - Capability: `{', '.join(chapter['capability_ids'])}`")
        lines.append("")
    if matrix:
        lines += ["### Nguồn Matrix9 đã index phải mở (private reference, không copy)", ""]
        for source in matrix:
            lines.append(f"- `{source['path']}` ({source['project']}, `{source['unit']}`, "
                         f"`{source['verification_state']}`) — {source['role_for_this_task']}")
            lines.append(f"  - Vai trò nguồn: {source['source_role']}. {source['detail']}")
            lines.append(f"  - Identifier đã index: `{', '.join(source['observed_identifiers'])}`")
        lines.append("")
    if manual:
        lines += ["### Sheet manual Matrix phải đọc (trong repo)", ""]
        for sheet in manual:
            lines.append(f"- `{sheet['sheet_id']}` — {sheet['manual']}, trang {sheet['pages']} "
                         f"· mục “{sheet['section']}”")
            lines.append(f"  - File: `{sheet['path']}`")
            lines.append(f"  - Dùng cho: {sheet['role_for_this_task']} (chủ đề: {sheet['topic']})")
        lines.append("")
    rhino5 = supplements.get("rhino5_geometry_anchors", [])
    if rhino5:
        lines += ["### Nguồn Rhino 5 hình học / C ABI phải đối chiếu (private reference, không copy)", ""]
        for src in rhino5:
            lines.append(f"- `{src['root']}/{src['path']}` ({src['project']}, "
                         f"`{src['verification_state']}`) — {src['role_for_this_task']}")
            lines.append(f"  - Vai trò nguồn: {src['source_role']}. {src['detail']}")
            lines.append(f"  - Identifier đã index: `{', '.join(src['observed_identifiers'])}`")
        lines.append("")
        lines += ["**Nguồn Rhino 5 KHÔNG chứng minh:** " +
                  " ".join(rhino5[0]["does_not_establish"]), ""]
    lines += ["### Quy tắc sử dụng nguồn tham chiếu", ""]
    for rule in supplements["usage_rules"]:
        lines.append(f"- {rule}")
    lines += ["", "### Điều các nguồn này KHÔNG chứng minh", ""]
    for item in supplements["does_not_establish"]:
        lines.append(f"- {item}")
    lines += ["",
              "> Symbol/identifier ở trên là điểm tra cứu; phải kiểm lại tại checkout/manifest trước khi viết mã. "
              "PASS vẫn chỉ đến từ command receipt và oracle positive/negative đo được.", ""]
    return "\n".join(lines)


def root_object_end(text: str) -> int:
    """Index of the closing brace of the top-level JSON object."""
    start = text.find("{")
    if start < 0:
        raise ValueError("no root object")
    depth = 0
    in_string = False
    escaped = False
    for index in range(start, len(text)):
        char = text[index]
        if in_string:
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == '"':
                in_string = False
            continue
        if char == '"':
            in_string = True
        elif char in "{[":
            depth += 1
        elif char in "}]":
            depth -= 1
            if depth == 0:
                return index
    raise ValueError("unterminated root object")


def inject_object(text: str, field: str, value) -> str:
    """Add ``field`` as a new member at the end of the root object.

    Appending at root level is deterministic, avoids matching same-named nested
    members, and keeps the original member order untouched.
    """
    end = root_object_end(text)
    head = text[:end].rstrip()
    tail = text[end:]
    if not head.endswith("{"):
        head += ","
    rendered = render_field(value, "  ", 1)
    return head + "\n" + f'  "{field}": ' + rendered + tail


def inject_markdown(text: str, block: str) -> str:
    if RUNBOOK_HEADING in text:
        return text
    value = json.loads(text)
    current = value.get("agent_runbook_markdown")
    if not isinstance(current, str) or not current:
        return text
    combined = current.rstrip() + "\n\n" + block
    if not combined.endswith("\n"):
        combined += "\n"
    rendered = json.dumps(combined, ensure_ascii=False)
    marker = '"agent_runbook_markdown": '
    if marker not in text:
        raise ValueError("agent_runbook_markdown member not found")
    start = text.index(marker) + len(marker)
    end = start
    in_string = False
    escaped = False
    for char in text[start:]:
        if in_string:
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == '"':
                break
        elif char == '"':
            in_string = True
        end += 1
    end += 1  # include closing quote
    return text[:start] + rendered + text[end:]


def strip_reference_section(runbook: str) -> str:
    """Remove only the reference-supplement section from the runbook.

    The reference section is always written last, but other authors append their own
    sections after it (the M01/M02 code-sample block does). Cutting at the heading
    would destroy those, so this removes just the section body: from its heading up to
    the next same-level ``## `` heading, or to the end when it is last.
    """
    start = runbook.find(RUNBOOK_HEADING)
    if start < 0:
        return runbook
    # the next level-2 heading after this one starts the following section
    nxt = runbook.find("\n## ", start + len(RUNBOOK_HEADING))
    if nxt < 0:
        return runbook[:start].rstrip() + "\n"
    return (runbook[:start].rstrip() + "\n" + runbook[nxt + 1:]).strip("\n") + "\n"


def strip_applied(text: str) -> str:
    """Remove previously applied reference fields and the reference runbook section."""
    data = json.loads(text)
    data.pop(REF_KEY, None)
    data.pop(POLICY_FIELD, None)
    runbook = data.get("agent_runbook_markdown")
    if isinstance(runbook, str) and RUNBOOK_HEADING in runbook:
        data["agent_runbook_markdown"] = strip_reference_section(runbook)
    return json.dumps(data, ensure_ascii=False, indent=2) + "\n"


def process(path: Path, l3_id: str, apply: bool, refresh: bool = False) -> str:
    text = path.read_text(encoding="utf-8")
    supplements = build_supplements(l3_id)
    if supplements is None:
        return "skipped_no_anchor"
    had = REF_KEY in text
    if had and not refresh:
        return "already_applied"
    if had:
        text = strip_applied(text)
    updated = inject_object(text, REF_KEY, supplements)
    updated = inject_object(updated, POLICY_FIELD, build_policy())
    updated = inject_markdown(updated, runbook_block(supplements))
    json.loads(updated)  # fail closed on malformed output
    if apply:
        path.write_text(updated, encoding="utf-8")
    return "refreshed" if had else "applied"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--apply", action="store_true", help="write the files (default: dry run)")
    parser.add_argument("--refresh", action="store_true",
                        help="re-generate records that already carry supplements")
    parser.add_argument("--milestone", action="append", default=None,
                        help="limit to a milestone (M00/M01/M02); repeatable")
    parser.add_argument("--l3", action="append", default=None,
                        help="limit to specific parent L3 ids; repeatable")
    parser.add_argument("--limit", type=int, default=0, help="stop after N files (debugging)")
    args = parser.parse_args()

    inventory = json.loads(INVENTORY.read_text(encoding="utf-8"))
    counts = {}
    processed = 0
    for row in inventory["l4"]:
        if args.milestone and row["milestone"] not in args.milestone:
            continue
        if args.l3 and row["parent_l3_id"] not in args.l3:
            continue
        path = ROOT / row["file"]
        result = process(path, row["parent_l3_id"], args.apply, args.refresh)
        counts[result] = counts.get(result, 0) + 1
        processed += 1
        if args.limit and processed >= args.limit:
            break
    print(json.dumps({"apply": args.apply, "refresh": args.refresh, "processed": processed,
                      "results": counts}, ensure_ascii=False))


if __name__ == "__main__":
    main()
