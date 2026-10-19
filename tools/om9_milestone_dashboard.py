#!/usr/bin/env python3
"""Sinh dashboard trạng thái cho cây kế hoạch OM9_Milestones.

Chạy lại sau mỗi mục hoàn thành:

    python tools/om9_milestone_dashboard.py

Dashboard đọc **bằng chứng thật** trong ``implementation/`` (design, fixtures,
evidence) chứ không tin trường trạng thái trong cây kế hoạch: mọi bản ghi
canonical vẫn ở ``planning_state = SPEC_ONLY_NOT_IMPLEMENTED`` và dashboard
không được phép nâng trạng thái nào thay cho việc nghiệm thu thật.

Quy ước trạng thái cho một L3:

* ``DONE``  - có ``*ui-review*.json`` với ``reviewer.status == "SIGNED"``;
* ``WIP``   - có bằng chứng (evidence/design/fixtures) nhưng chưa ký;
* ``TODO``  - mới ở mức kế hoạch.

Đầu ra: ``MILESTONE_DASHBOARD.md`` ở gốc checkout.
"""

from __future__ import annotations

import datetime
import json
import pathlib
import re
import subprocess
from dataclasses import dataclass, field

ROOT = pathlib.Path(__file__).resolve().parents[1]
TREE = ROOT / "OM9_Milestones"
OUTPUT = ROOT / "MILESTONE_DASHBOARD.md"
EVIDENCE = ROOT / "implementation" / "evidence"
DESIGN = ROOT / "implementation" / "design"
FIXTURES = ROOT / "implementation" / "fixtures"
LEDGER = ROOT / "docs" / "openmatrix9-progress.json"

DONE, WIP, TODO = "DONE", "WIP", "TODO"
MARK = {DONE: "✅", WIP: "🟡", TODO: "⬜"}
LABEL = {DONE: "đã nghiệm thu", WIP: "có bằng chứng, chưa ký", TODO: "mới ở mức kế hoạch"}


@dataclass
class Level3:
    id: str
    title: str
    stage: str
    risk: str
    manual: bool
    task_ids: list[str]
    path: pathlib.Path
    status: str = TODO
    l4_evidence: int = 0
    evidence: list[str] = field(default_factory=list)
    signed_by: str = ""


@dataclass
class Level2:
    id: str
    title: str
    scope: str
    path: pathlib.Path
    children: list[Level3] = field(default_factory=list)

    def counts(self) -> dict[str, int]:
        result = {DONE: 0, WIP: 0, TODO: 0}
        for child in self.children:
            result[child.status] += 1
        return result


@dataclass
class Milestone:
    key: str
    folder: pathlib.Path
    id: str
    title: str
    planning_state: str
    depends: list[str]
    counts: dict[str, int]
    modules: list[Level2] = field(default_factory=list)
    gate_evidence: list[str] = field(default_factory=list)

    def rollup(self) -> dict[str, int]:
        result = {DONE: 0, WIP: 0, TODO: 0}
        for module in self.modules:
            for key, value in module.counts().items():
                result[key] += value
        return result


def load(path: pathlib.Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def resolve(value: str, *bases: pathlib.Path) -> pathlib.Path | None:
    """Đường dẫn trong bản ghi có thể tính từ gốc cây, từ milestone hoặc từ L2."""
    for base in (TREE, *bases):
        candidate = base / value
        if candidate.exists():
            return candidate
    return None


class Index:
    """Khớp L2/L3 theo ID khi đường dẫn ghi trong bản ghi đã cũ.

    Cây đã được rút ngắn tên thư mục bên ngoài tool (PATH_RENAMES.json) và một
    phần bản ghi vẫn giữ đường dẫn cũ; dashboard không được vì thế mà bỏ sót
    mục nào, nên nó đọc thẳng ``L2.json``/``l3.json`` và khớp theo ID.
    """

    def __init__(self) -> None:
        self.l2_by_id: dict[str, pathlib.Path] = {}
        self.l3_by_id: dict[str, pathlib.Path] = {}
        self.stale: list[str] = []

    def build(self, folder: pathlib.Path) -> None:
        for l2_path in folder.rglob("L2.json"):
            try:
                document = load(l2_path)
            except (OSError, json.JSONDecodeError):
                continue
            identifier = str(document.get("id", ""))
            if identifier:
                self.l2_by_id.setdefault(identifier, l2_path)
        for l3_path in folder.rglob("l3.json"):
            try:
                document = load(l3_path)
            except (OSError, json.JSONDecodeError):
                continue
            identifier = str(document.get("id", ""))
            if identifier:
                self.l3_by_id.setdefault(identifier, l3_path)

    def note_stale(self, kind: str, identifier: str, recorded: str) -> None:
        self.stale.append(f"{kind} {identifier}: {recorded}")


def is_l3_directory(name: str) -> bool:
    """`OM9-L3-...` and the milestone-qualified `OM9-M00-L3-...` both name an L3 bucket."""
    return name.startswith("OM9-L3-") or (len(name) > 10 and name.startswith("OM9-M") and "-L3-" in name)


def evidence_files(directory: pathlib.Path) -> list[pathlib.Path]:
    if not directory.is_dir():
        return []
    return sorted(path for path in directory.rglob("*") if path.is_file())


def signed_reviewer(files: list[pathlib.Path]) -> str:
    for path in files:
        if "ui-review" not in path.name or path.suffix.lower() != ".json":
            continue
        try:
            document = json.loads(path.read_text(encoding="utf-8"))
        except json.JSONDecodeError:
            continue
        reviewer = document.get("reviewer") or {}
        status = str(reviewer.get("status", "")).upper()
        if status == "SIGNED":
            return str(reviewer.get("role") or reviewer.get("name") or "người review")
        # A plan item whose manual_test_gate is AUTO_ONLY asks for no human test:
        # its acceptance rests on the automated checks, and the review file says so.
        if status == "AUTOMATED_EVIDENCE_ONLY" and str(
            document.get("manual_test_gate", "")
        ).upper() == "AUTO_ONLY":
            return "cổng AUTO_ONLY (bằng chứng tự động)"
    return ""


def collect_evidence(milestone_key: str) -> dict[str, dict]:
    """Bằng chứng theo L3: evidence/, design/, fixtures/ dưới implementation/."""
    result: dict[str, dict] = {}
    for root in (EVIDENCE, DESIGN, FIXTURES):
        base = root / milestone_key
        if not base.is_dir():
            continue
        for entry in base.iterdir():
            if not entry.is_dir() or not is_l3_directory(entry.name):
                continue
            bucket = result.setdefault(entry.name, {"files": [], "sources": []})
            files = evidence_files(entry)
            bucket["files"].extend(files)
            if files:
                bucket["sources"].append(root.relative_to(ROOT).as_posix())
    return result


def l4_evidence_count(task_ids: list[str], files: list[pathlib.Path]) -> int:
    """Đếm L4 có artefact thật, theo đúng loại artefact mà gói kế hoạch yêu cầu.

    - ``-01`` SOURCE_AUDIT: ``design/.../source_crosswalk.json``
    - ``-02`` CONTRACT_TEST: ``design/.../api.md`` + ``fixtures/.../{positive,negative}.json``
    - ``-03`` IMPLEMENT: ``evidence/.../<L4>-03.*``
    - ``-05`` ACCEPTANCE: ``evidence/.../<L4>-05.*``
    """
    names = {path.name for path in files}
    hits = 0
    for task_id in task_ids:
        suffix = task_id.rsplit("-", 1)[-1]
        role = task_id.rsplit("-", 2)[-2]
        covered = False
        if suffix == "01":
            covered = "source_crosswalk.json" in names
        elif suffix == "02":
            covered = "api.md" in names and {"positive.json", "negative.json"} <= names
        elif suffix in {"03", "05"}:
            covered = any(name.startswith(task_id) for name in names)
        else:
            covered = any(name.startswith(task_id) for name in names)
        if not covered and role == "API":
            covered = "api.md" in names
        hits += 1 if covered else 0
    return hits


def build() -> tuple[list[Milestone], Index]:
    milestones: list[Milestone] = []
    index = Index()
    for folder in sorted(p for p in TREE.iterdir() if p.is_dir() and p.name[:1] in {"M", "Z"}):
        l1_path = folder / "L1.json"
        if not l1_path.exists():
            continue
        index.build(folder)
        l1 = load(l1_path)
        milestone = Milestone(
            key=folder.name.split(" ")[0],
            folder=folder,
            id=str(l1.get("id", "")),
            title=str(l1.get("title", "")),
            planning_state=str(l1.get("planning_state", "")),
            depends=list(l1.get("depends_on_milestones", [])),
            counts={DONE: 0, WIP: 0, TODO: 0},
        )
        evidence = collect_evidence(milestone.key)
        for module_row in l1.get("modules", []):
            module_id = str(module_row.get("id", ""))
            recorded = str(module_row.get("file", ""))
            l2_path = resolve(recorded, folder)
            if l2_path is None:
                l2_path = index.l2_by_id.get(module_id)
                if l2_path is not None:
                    index.note_stale("L2", module_id, recorded)
            if l2_path is None or not l2_path.exists():
                continue
            l2_document = load(l2_path)
            definition = l2_document.get("definition", {})
            module = Level2(
                id=str(l2_document.get("id", module_id)),
                title=str(l2_document.get("title", "")),
                scope=str(definition.get("scope", "")),
                path=l2_path,
            )
            for child in l2_document.get("children_l3", []):
                child_id = str(child.get("id", ""))
                recorded = str(child.get("file", ""))
                l3_path = resolve(recorded, folder, l2_path.parent)
                if l3_path is None:
                    l3_path = index.l3_by_id.get(child_id)
                    if l3_path is not None:
                        index.note_stale("L3", child_id, recorded)
                if l3_path is None or not l3_path.exists():
                    continue
                l3_document = load(l3_path)
                specification = l3_document.get("specification", {})
                level3 = Level3(
                    id=str(l3_document.get("id", child_id)),
                    title=str(l3_document.get("title", "")),
                    stage=str(specification.get("execution_stage", "")),
                    risk=str(specification.get("planning_risk", "")),
                    manual=bool(specification.get("requires_manual_test", False)),
                    task_ids=[str(task) for task in l3_document.get("task_ids", [])],
                    path=l3_path,
                )
                bucket = evidence.get(level3.id)
                if bucket:
                    files = bucket["files"]
                    level3.evidence = sorted(set(bucket["sources"]))
                    level3.l4_evidence = l4_evidence_count(level3.task_ids, files)
                    level3.signed_by = signed_reviewer(files)
                    level3.status = DONE if level3.signed_by else (WIP if files else TODO)
                module.children.append(level3)
            milestone.modules.append(module)
        gate_dir = EVIDENCE / milestone.key
        if gate_dir.is_dir():
            milestone.gate_evidence = sorted(
                entry.name
                for entry in gate_dir.iterdir()
                if entry.is_dir()
                and not is_l3_directory(entry.name)
                and evidence_files(entry)
            )
        # Bản ghi L1 có thể thiếu module đang có thật trên đĩa; dashboard phản ánh
        # cây thư mục, nên bổ sung phần còn thiếu theo chính L2.json.
        listed = {module.id for module in milestone.modules}
        for l2_path in sorted(folder.rglob("L2.json")):
            document = load(l2_path)
            identifier = str(document.get("id", ""))
            if not identifier or identifier in listed:
                continue
            definition = document.get("definition", {})
            module = Level2(
                id=identifier,
                title=str(document.get("title", "")),
                scope=str(definition.get("scope", "")) + " (không có trong L1.json)",
                path=l2_path,
            )
            for child in document.get("children_l3", []):
                child_id = str(child.get("id", ""))
                l3_path = resolve(str(child.get("file", "")), folder, l2_path.parent) or index.l3_by_id.get(child_id)
                if l3_path is None or not l3_path.exists():
                    continue
                l3_document = load(l3_path)
                specification = l3_document.get("specification", {})
                level3 = Level3(
                    id=str(l3_document.get("id", child_id)),
                    title=str(l3_document.get("title", "")),
                    stage=str(specification.get("execution_stage", "")),
                    risk=str(specification.get("planning_risk", "")),
                    manual=bool(specification.get("requires_manual_test", False)),
                    task_ids=[str(task) for task in l3_document.get("task_ids", [])],
                    path=l3_path,
                )
                bucket = evidence.get(level3.id)
                if bucket:
                    files = bucket["files"]
                    level3.evidence = sorted(set(bucket["sources"]))
                    level3.l4_evidence = l4_evidence_count(level3.task_ids, files)
                    level3.signed_by = signed_reviewer(files)
                    level3.status = DONE if level3.signed_by else (WIP if files else TODO)
                module.children.append(level3)
            milestone.modules.append(module)
        milestone.counts = milestone.rollup()
        milestones.append(milestone)
    return milestones, index


def recent_ledger(limit: int = 8) -> tuple[list[dict], str]:
    if not LEDGER.exists():
        return [], ""
    document = json.loads(LEDGER.read_text(encoding="utf-8"))
    items = [item for item in document.get("items", []) if isinstance(item, dict)]
    recent = list(reversed(items))[:limit]
    next_action = ""
    for item in reversed(items):
        if item.get("next_action"):
            next_action = str(item["next_action"])
            break
    return recent, next_action


def source_revision() -> str:
    try:
        result = subprocess.run(
            ["git", "rev-parse", "--short", "HEAD"],
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=False,
        )
        return result.stdout.strip() or "unknown"
    except OSError:
        return "unknown"


def render(milestones: list[Milestone], index: Index) -> str:
    generated = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    total = {DONE: 0, WIP: 0, TODO: 0}
    for milestone in milestones:
        for key, value in milestone.counts.items():
            total[key] += value

    lines: list[str] = []
    lines.append("# OM9 Milestone Dashboard")
    lines.append("")
    lines.append(
        f"> Sinh tự động lúc **{generated}** · mã nguồn `{source_revision()}` · "
        f"nguồn: `OM9_Milestones/` + `implementation/{{evidence,design,fixtures}}/`"
    )
    lines.append(">")
    lines.append("> Cập nhật: chạy lại `python tools/om9_milestone_dashboard.py` sau mỗi mục hoàn thành.")
    lines.append(">")
    lines.append(
        f"> **{total[DONE]}** L3 đã nghiệm thu · **{total[WIP]}** có bằng chứng chưa ký · "
        f"**{total[TODO]}** mới ở mức kế hoạch (tổng {sum(total.values())} L3)."
    )
    lines.append(">")
    lines.append(
        "> Dashboard chỉ đọc bằng chứng thật; cây kế hoạch vẫn giữ "
        "`SPEC_ONLY_NOT_IMPLEMENTED` và không trạng thái nào được nâng thay cho nghiệm thu."
    )
    if index.stale:
        lines.append(">")
        lines.append(
            f"> ⚠️ {len(index.stale)} bản ghi còn đường dẫn cũ sau đợt rút ngắn tên thư mục; "
            "dashboard khớp theo ID nên vẫn hiển thị đủ. Xem `OM9_Milestones/PATH_RENAMES.json`."
        )
    lines.append("")
    lines.append(f"Chú giải: {MARK[DONE]} {LABEL[DONE]} · {MARK[WIP]} {LABEL[WIP]} · {MARK[TODO]} {LABEL[TODO]}")
    lines.append("")

    lines.append("## Tổng quan theo milestone")
    lines.append("")
    lines.append("| Milestone | Thư mục | L2 | L3 | ✅ | 🟡 | ⬜ | Phụ thuộc |")
    lines.append("|---|---|---:|---:|---:|---:|---:|---|")
    for milestone in milestones:
        lines.append(
            f"| **{milestone.key}** {milestone.title} | `{milestone.folder.name}` | "
            f"{len(milestone.modules)} | {sum(milestone.counts.values())} | "
            f"{milestone.counts[DONE]} | {milestone.counts[WIP]} | {milestone.counts[TODO]} | "
            f"{', '.join(milestone.depends) or '—'} |"
        )
    lines.append("")

    lines.append("## Cấu trúc chi tiết (Mxx → L1 → L2 → L3)")
    lines.append("")
    for milestone in milestones:
        counts = milestone.counts
        lines.append(
            f"<details><summary><b>{milestone.key} — {milestone.title}</b> · "
            f"{MARK[DONE]}{counts[DONE]} {MARK[WIP]}{counts[WIP]} {MARK[TODO]}{counts[TODO]} · "
            f"`{milestone.id}` · {milestone.planning_state}</summary>"
        )
        lines.append("")
        lines.append(f"**L1** — {milestone.title} · phụ thuộc: {', '.join(milestone.depends) or '—'}")
        if milestone.gate_evidence:
            lines.append("")
            lines.append("Bằng chứng cấp milestone: " + ", ".join(f"`{name}`" for name in milestone.gate_evidence))
        for module in milestone.modules:
            counts = module.counts()
            lines.append("")
            lines.append(
                f"**{module.id} — {module.title}** · "
                f"{MARK[DONE]}{counts[DONE]} {MARK[WIP]}{counts[WIP]} {MARK[TODO]}{counts[TODO]}"
            )
            if module.scope:
                lines.append("")
                lines.append(f"_{module.scope}_")
            lines.append("")
            lines.append("| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |")
            lines.append("|---|---|---|---|---:|---|---|")
            for child in module.children:
                evidence = ", ".join(f"`{source}`" for source in child.evidence) or "—"
                signed = f" — ký bởi {child.signed_by}" if child.signed_by else ""
                lines.append(
                    f"| `{child.id}` | {child.title}{' *(thủ công)*' if child.manual else ''} | "
                    f"{child.stage} | {child.risk} | {child.l4_evidence}/{len(child.task_ids)} | "
                    f"{MARK[child.status]} {LABEL[child.status]}{signed} | {evidence} |"
                )
        lines.append("")
        lines.append("</details>")
        lines.append("")

    recent, next_action = recent_ledger()
    lines.append("## Việc đã ghi gần đây (live ledger)")
    lines.append("")
    lines.append("| Ngày | Mục | Trạng thái | Phạm vi |")
    lines.append("|---|---|---|---|")
    for item in recent:
        scope = str(item.get("scope", "")).replace("|", "/")
        lines.append(
            f"| {item.get('date', '')} | `{item.get('id', '')}` | {item.get('status', '')} | {scope[:120]} |"
        )
    lines.append("")
    if next_action:
        lines.append("## Việc kế tiếp")
        lines.append("")
        lines.append(f"{next_action}")
        lines.append("")
    return "\n".join(lines)


def main() -> int:
    milestones, index = build()
    OUTPUT.write_text(render(milestones, index), encoding="utf-8")
    total = {DONE: 0, WIP: 0, TODO: 0}
    for milestone in milestones:
        for key, value in milestone.counts.items():
            total[key] += value
    print(
        f"wrote {OUTPUT.relative_to(ROOT).as_posix()}: "
        f"{len(milestones)} milestone, {sum(len(m.modules) for m in milestones)} L2, "
        f"{sum(total.values())} L3 — {total[DONE]} done / {total[WIP]} wip / {total[TODO]} todo"
    )
    print(f"  bản ghi có đường dẫn cũ (khớp theo ID): {len(index.stale)}")
    for row in index.stale[:5]:
        print("   ", row)
    for milestone in milestones:
        if milestone.counts[DONE] or milestone.counts[WIP]:
            print(
                f"  {milestone.key}: done={milestone.counts[DONE]} wip={milestone.counts[WIP]} "
                f"todo={milestone.counts[TODO]}"
            )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
