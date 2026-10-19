"""Rut ngan duong dan cay OM9_Milestones ma khong mat thong tin.

Quy tac:
  * 11 thu muc module duoc doi sang alias ngan (ID + vai tu khoa).
  * Thu muc L2/L3/L4 chi bi doi khi can thiet de moi duong dan <= BUDGET.
    Uu tien bo hau to boilerplate " - menu binding", sau do cat tieu de theo
    ranh gioi tu (khong de lai tu noi/huu tu o cuoi).
  * Tieu de day du KHONG mat: no da nam trong L1.json / L2.json / l3.json /
    l4.json, va duoc ghi lai trong OM9_Milestones/PATH_RENAMES.{json,md}.

Mac dinh la dry-run (khong ghi gi). Dung --apply de thuc thi.

Pham vi viet lai tham chieu (theo quyet dinh cua nguoi dung):
  * OM9_Milestones/**  (ban ghi JSON + md)
  * tools/*.py         (tool dang chay)
  * docs/openmatrix9-progress.json (live ledger)
  * KHONG sua tai lieu lich su: docs/validation/**, .agents/**
"""

from __future__ import annotations

import argparse
import collections
import datetime as dt
import io
import json
import os
import re
import sys
import urllib.parse

ROOT = r"H:\OpenMatrix9"
MS = os.path.join(ROOT, "OM9_Milestones")
BUDGET = 210
PLAN_OUT = os.path.join(ROOT, "implementation", "tools", "om9_path_plan.json")
REPORT_OUT = os.path.join(ROOT, "implementation", "design", "om9-milestone-path-shortening.md")

ALIAS = {
    # M00 bi khoa handle o cap thu muc module (khong doi ten duoc trong phien nay);
    # ten dai cua no duoc bu bang cach cat cac thu muc con ben trong.
    "M00 - Feasibility, QymCAD Source, OCCT, Legal and Environment": "M00 - Feasibility, QymCAD Source, OCCT, Legal and Environment",
    "M01 - CAD Shell, OCCT Bridge, Document, Undo, GPU and Viewport Foundation": "M01 - CAD Shell, Viewport",
    "M02 - Viewport Picking, Snapping, Navigation, Gumball and Matrix Interaction": "M02 - Viewport, Gumball",
    "M03 - Curve, Surface, Solid and 3DM Exchange": "M03 - Geometry 3DM",
    "M04 - Builder, History and Ring": "M04 - Builder and History",
    "M05 - Gems and Layout": "M05 - Gems and Layout",
    "M06 - Settings and Prongs": "M06 - Settings and Prongs",
    "M07 - Cutters and Manufacturing Validation": "M07 - Cutters",
    "M08 - Advanced Jewelry Modeling": "M08 - Jewelry Modeling",
    "M09 - Acceptance and Release Gates": "M09 - Acceptance and Release Gates",  # <=113 ky tu, khong can doi
    "ZAFTER - Post-Release Extensions": "ZAFTER - Extensions",
}

# Ten module du kien (M00 chua lam duoc trong phien nay; xem --finish-module)
MODULE_TARGET = {k: v for k, v in ALIAS.items() if k != v}
MODULE_TARGET["M00 - Feasibility, QymCAD Source, OCCT, Legal and Environment"] = "M00 - Feasibility, Legal"

# Sua thu cong trong tools/ (khong quet bua, vi mot so tool dung ten thu muc cua
# goi incoming `build/mNN-incoming/...` — ten do KHONG doi).
# (duong dan tuong doi, chuoi cu, chuoi moi, so lan mong doi)
# M00/M03 giu nguyen ten (thu muc bi khoa) nen cap old==new la co y.
TOOL_PATCHES: list[tuple[str, str, str, int]] = [
    ("tools/apply_implementation_samples.py",
     '"M00 - Feasibility, QymCAD Source, OCCT, Legal and Environment"',
     '"M00 - Feasibility, QymCAD Source, OCCT, Legal and Environment"', 1),
    ("tools/apply_implementation_samples.py",
     '"M01 - CAD Shell, OCCT Bridge, Document, Undo, GPU and Viewport Foundation"', '"M01 - CAD Shell, Viewport"', 1),
    ("tools/apply_implementation_samples.py",
     '"M02 - Viewport Picking, Snapping, Navigation, Gumball and Matrix Interaction"', '"M02 - Viewport, Gumball"', 1),
    ("tools/apply_implementation_samples.py",
     '"M03 - Curve, Surface, Solid and 3DM Exchange"',
     '"M03 - Curve, Surface, Solid and 3DM Exchange"', 1),
    ("tools/apply_implementation_samples.py",
     '"M04 - Builder, History and Ring"', '"M04 - Builder and History"', 1),
    ("tools/apply_implementation_samples.py",
     '"M07 - Cutters and Manufacturing Validation"', '"M07 - Cutters"', 1),
    ("tools/apply_implementation_samples.py",
     '"M08 - Advanced Jewelry Modeling"', '"M08 - Jewelry Modeling"', 1),
    ("tools/record_m00_l2_01_status.py",
     "OM9_Milestones/M00 - Feasibility, QymCAD Source, OCCT, Legal and Environment/L1.json",
     "OM9_Milestones/M00 - Feasibility, QymCAD Source, OCCT, Legal and Environment/L1.json", 1),
    ("tools/add_m07_delegated_module.py",
     '"M01 - CAD Shell, OCCT Bridge, Document, Undo, GPU and Viewport Foundation"', '"M01 - CAD Shell, Viewport"', 1),
    ("tools/add_m07_delegated_module.py",
     '"M07 - Cutters and Manufacturing Validation"', '"M07 - Cutters"', 1),
    ("tools/add_m07_delegated_module.py",
     "M07 - Cutters and Manufacturing Validation/{L2_DIRNAME}", "M07 - Cutters/{L2_DIRNAME}", 1),
    ("tools/sync_qymcad_revision.py",
     '"OM9_Milestones/M03 - Curve, Surface, Solid and 3DM Exchange"',
     '"OM9_Milestones/M03 - Curve, Surface, Solid and 3DM Exchange"', 1),
    ("tools/sync_qymcad_revision.py",
     '"OM9_Milestones/M04 - Builder, History and Ring"', '"OM9_Milestones/M04 - Builder and History"', 1),
    ("tools/sync_qymcad_revision.py",
     '"OM9_Milestones/M07 - Cutters and Manufacturing Validation"', '"OM9_Milestones/M07 - Cutters"', 1),
    ("tools/sync_m07_source_revision.py",
     '"OM9_Milestones/M07 - Cutters and Manufacturing Validation"', '"OM9_Milestones/M07 - Cutters"', 1),
    ("tools/verify_m08_l4_merge.py",
     'M08_DIR = "M08 - Advanced Jewelry Modeling"', 'M08_DIR = "M08 - Jewelry Modeling"', 1),
    ("tools/verify_m08_l4_merge.py",
     'M03_DIR = "M03 - Curve, Surface, Solid and 3DM Exchange"',
     'M03_DIR = "M03 - Curve, Surface, Solid and 3DM Exchange"', 1),
    ("tools/verify_merge.py",
     'ROOT / "OM9_Milestones" / "M08 - Advanced Jewelry Modeling"', 'ROOT / "OM9_Milestones" / "M08 - Jewelry Modeling"', 2),
]

NAMERE = re.compile(r"^L[234]-")
STOP = {
    "and", "or", "the", "of", "to", "for", "with", "in", "on", "between",
    "plus", "from", "a", "an", "at", "by", "into", "over", "under", "its",
}
SUFFIXES = (" - menu binding", " - menu-binding", " - Menu Binding")

REWRITE_SUFFIXES = (".json", ".md", ".txt", ".py")
INDEX_NAMES = ("PATH_RENAMES.json", "PATH_RENAMES.md")
REWRITE_EXCLUDE_DIRS = (
    os.path.join("docs", "validation"),
    ".agents",
)


# --------------------------------------------------------------------------
# plan
# --------------------------------------------------------------------------
def rel_parts(path: str) -> list[str]:
    return os.path.relpath(path, MS).split(os.sep)


def truncate_title(title: str, keep: int) -> str:
    """Bo dan tung tu (hoac tung doan sau dau '-') cho den khi du ngan.

    Khong bao gio cat giua tu. Van co the bo doan cuoi cua mot tu ghep
    (vi du ``06-febe-ring-builder`` -> ``06-febe``) va luon giu it nhat mot doan.
    """
    words = title.split()
    while len(" ".join(words)) > keep:
        segments = words[-1].split("-")
        if len(segments) > 1:
            segments.pop()
            words[-1] = "-".join(segments)
        elif len(words) > 1:
            words.pop()
        else:
            break
    changed = True
    while changed and words:
        changed = False
        segments = words[-1].split("-")
        if len(segments) > 1 and segments[-1].lower().strip(".,;:\u00b7") in STOP:
            segments.pop()
            words[-1] = "-".join(segments)
            changed = True
        elif len(words) > 1 and words[-1].lower().strip(".,;:\u00b7") in STOP:
            words.pop()
            changed = True
    return " ".join(words).rstrip(" -\u2013\u2014\u00b7,;:")


def suffix_candidate(name: str) -> str | None:
    if " - " not in name:
        return None
    idpart, title = name.split(" - ", 1)
    for suf in SUFFIXES:
        if title.endswith(suf):
            return idpart + " - " + title[: -len(suf)]
    return None


def prose_candidate(name: str, need: int) -> str | None:
    if " - " not in name:
        return None
    idpart, title = name.split(" - ", 1)
    sep = " - "
    target = max(len(name) - need, len(idpart) + len(sep) + 8)
    keep = target - len(idpart) - len(sep)
    short = truncate_title(title, keep)
    if short == title:
        # nhu cau nho hon do "tho" cua mot tu/doan: bo bot mot don vi nua
        for smaller in range(keep - 1, 0, -1):
            candidate = truncate_title(title, smaller)
            if candidate != title:
                short = candidate
                break
    if not short:
        return None
    cand = idpart + sep + short
    return cand if len(cand) < len(name) else None


def build_plan(
    files: list[str],
    alias: dict[str, str] | None = None,
    skip_dirs: set[str] | None = None,
) -> dict:
    """Chon tap thu muc can doi ten nho nhat de moi duong dan <= BUDGET.

    Moi vong: dem so duong qua ngan sach di qua tung thu muc, chon thu muc
    "ganh" nhieu nhat (uu tien cap cao hon khi bang nhau), cat ten vua du.
    Thu muc da cat het muc (chi con ID + 1-2 tu) duoc danh dau la ket.
    """
    alias = ALIAS if alias is None else alias
    skip_dirs = set() if skip_dirs is None else set(skip_dirs)
    renames: dict[str, str] = {}

    def new_parts(path: str) -> list[str]:
        parts = rel_parts(path)
        out = list(parts)
        out[0] = alias.get(parts[0], parts[0])
        for i in range(1, len(parts)):
            key = os.path.join(MS, *parts[: i + 1])
            if key in renames:
                out[i] = renames[key]
        return out

    def new_len(path: str) -> int:
        return len(os.path.join(MS, *new_parts(path)))

    stuck: set[str] = set()
    guard = 0
    while guard < 4000:
        guard += 1
        offending = [p for p in files if new_len(p) > BUDGET]
        if not offending:
            break
        score: collections.Counter = collections.Counter()
        for path in offending:
            parts = rel_parts(path)
            keys = []
            for lvl in (1, 2, 3):
                if lvl >= len(parts) - 1 or not NAMERE.match(parts[lvl]):
                    continue
                key = os.path.join(MS, *parts[: lvl + 1])
                if key not in stuck and key not in skip_dirs:
                    keys.append(key)
            for key in set(keys):
                score[key] += 1
        if not score:
            break
        best = max(score, key=lambda k: (score[k], -len(rel_parts(k)), -len(k)))
        lvl = len(rel_parts(best)) - 1
        need = max(
            new_len(p) - BUDGET
            for p in offending
            if os.path.join(MS, *rel_parts(p)[: lvl + 1]) == best
        )
        current = renames.get(best, os.path.basename(best))
        cand = suffix_candidate(current)
        if not (cand and len(current) - len(cand) >= need):
            cand = prose_candidate(current, need)
        if not cand or len(cand) >= len(current):
            stuck.add(best)
            continue
        renames[best] = cand

    unresolved = [(new_len(p), os.path.relpath(p, MS)) for p in files if new_len(p) > BUDGET]
    return {"renames": renames, "unresolved": unresolved}


def renamed_path(path: str, renames: dict[str, str], alias: dict[str, str] | None = None) -> str:
    """Duong dan tuong lai cua mot file/thu muc sau khi doi ten."""
    alias = ALIAS if alias is None else alias
    parts = rel_parts(path)
    out = list(parts)
    out[0] = alias.get(parts[0], parts[0])
    for i in range(1, len(parts)):
        key = os.path.join(MS, *parts[: i + 1])
        if key in renames:
            out[i] = renames[key]
    return os.path.join(MS, *out)


# --------------------------------------------------------------------------
# reference rewrite
# --------------------------------------------------------------------------
def scan_excluded(pairs: list[tuple[str, str]]) -> dict[str, int]:
    """Dem ten cu con sot lai o khu vuc KHONG tu dong sua (lich su + tools)."""
    roots = [
        os.path.join(ROOT, "tools"),
        os.path.join(ROOT, "docs", "validation"),
        os.path.join(ROOT, ".agents"),
    ]
    hits: dict[str, int] = {}
    for root in roots:
        for dp, _, fn in os.walk(root):
            for f in fn:
                path = os.path.join(dp, f)
                try:
                    if os.path.getsize(path) > 8_000_000:
                        continue
                    text = read_text(path)
                except OSError:
                    continue
                count = sum(text.count(old) for old, new in pairs if old != new)
                if count:
                    hits[os.path.relpath(path, ROOT)] = count
    return hits


def execute_renames(
    ms: str,
    renames: dict[str, str],
    alias: dict[str, str],
    skip_locked: bool = False,
) -> list[tuple[str, str, bool, str]]:
    """Doi ten thu muc that su, tu tren xuong; tu hoan tac neu loi.

    ``renames``: duong dan thu muc GOC -> ten moi. Tra ve danh sach
    ``(src, dst, ok, loi)``. Khi ``skip_locked``, thu muc bi he dieu hanh tu choi
    se duoc bo qua thay vi lam hong ca lo.
    """

    def parts_of(path: str) -> list[str]:
        return os.path.relpath(path, ms).split(os.sep)

    def dest_of(old: str) -> str:
        parts = parts_of(old)
        out = list(parts)
        out[0] = alias.get(parts[0], parts[0])
        for i in range(1, len(parts)):
            key = os.path.join(ms, *parts[: i + 1])
            if key in renames:
                out[i] = renames[key]
        return os.path.join(ms, *out)

    def live_of(old: str) -> str:
        """Duong dan hien tai: cha da doi ten, ten cua chinh no van la ten goc."""
        parts = parts_of(old)
        live = [alias.get(parts[0], parts[0])]
        for i in range(1, len(parts) - 1):
            key = os.path.join(ms, *parts[: i + 1])
            live.append(renames.get(key, parts[i]))
        return os.path.join(ms, *live, parts[-1])

    done: list[tuple[str, str, bool, str]] = []
    locked: list[tuple[str, str, bool, str]] = []
    try:
        for old, new in alias.items():
            if old == new or not os.path.isdir(os.path.join(ms, old)):
                continue
            src, dst = os.path.join(ms, old), os.path.join(ms, new)
            if os.path.exists(dst):
                raise RuntimeError(f"dich da ton tai: {dst}")
            try:
                os.rename(src, dst)
            except OSError as exc:
                if not skip_locked:
                    raise
                locked.append((src, dst, False, str(exc).splitlines()[0]))
                continue
            done.append((src, dst, True, ""))
        for old in sorted(renames, key=lambda p: len(parts_of(p))):
            src, dst = live_of(old), dest_of(old)
            if src == dst:
                continue
            if not os.path.isdir(src):
                if os.path.isdir(dst):
                    continue  # da doi tu lan chay truoc
                if skip_locked:
                    locked.append((src, dst, False, "khong thay thu muc nguon"))
                    continue
                raise RuntimeError(f"khong thay thu muc nguon: {src}")
            if os.path.exists(dst):
                raise RuntimeError(f"dich da ton tai: {dst}")
            try:
                os.rename(src, dst)
            except OSError as exc:
                if not skip_locked:
                    raise
                locked.append((src, dst, False, str(exc).splitlines()[0]))
                continue
            done.append((src, dst, True, ""))
    except Exception:
        for src, dst, _ok, _err in reversed(done):
            try:
                os.rename(dst, src)
            except OSError as undo_exc:
                print(f"    KHONG HOAN TAC DUOC {dst} -> {src}: {undo_exc}")
        raise
    return done + locked


def tree_stats(ms: str) -> dict:
    files = [os.path.join(dp, f) for dp, _, fn in os.walk(ms) for f in fn]
    lens = sorted(len(p) for p in files)
    return {"files": len(files), "max": max(lens) if lens else 0}


def write_index(index: dict) -> None:
    with open(os.path.join(MS, "PATH_RENAMES.json"), "w", encoding="utf-8") as fh:
        json.dump(index, fh, ensure_ascii=False, indent=1)
    with io.StringIO() as buf:
        buf.write("# OM9_Milestones — ánh xạ tên thư mục sau khi rút ngắn đường dẫn\n\n")
        buf.write(f"- Tạo: {index['created_utc']} | ngân sách: mọi đường dẫn ≤ **{index['budget']}** ký tự.\n")
        buf.write("- Lý do: giới hạn MAX_PATH 260 của Windows khi cây bị sao chép vào tiền tố dài hơn"
                  " (staging `build/...`, worktree agent, thư mục tạm).\n")
        buf.write("- **Không mất thông tin:** tiêu đề đầy đủ của mỗi cấp vẫn nằm trong `L1.json`,"
                  " `L2.json`, `l3.json`, `l4.json` tương ứng; bảng dưới chỉ là ánh xạ tên thư mục.\n")
        if index.get("notes"):
            for note in index["notes"]:
                buf.write(f"- {note}\n")
        buf.write("\n## Thư mục module\n\n| Trước | Sau | Trạng thái |\n|---|---|---|\n")
        for row in index["modules"]:
            state = "đã đổi" if row.get("applied") else "giữ nguyên"
            buf.write(f"| `{row['old']}` | `{row['new']}` | {state} |\n")
        buf.write("\n## Thư mục L2/L3/L4 đã đổi tên\n\n"
                  "| Cấp | Đường dẫn cũ | Tên mới | ID | Tiêu đề đầy đủ |\n|---|---|---|---|---|\n")
        for row in index["dirs"]:
            buf.write(
                f"| {row['level']} | `{row['old_path']}` | `{row['new_name']}` | {row['id'] or ''} |"
                f" {row['full_title'] or ''} |\n"
            )
        open(os.path.join(MS, "PATH_RENAMES.md"), "w", encoding="utf-8").write(buf.getvalue())


def tool_files() -> list[str]:
    seen: list[str] = []
    for rel, *_rest in TOOL_PATCHES:
        path = os.path.join(ROOT, rel.replace("/", os.sep))
        if path not in seen and os.path.isfile(path):
            seen.append(path)
    return seen


def sync_tools(apply: bool = True) -> int:
    """Dong bo hang so module trong tools/ voi ten thu muc THUC TE tren dia.

    Can thiet vi M00/M03 co the khong doi ten duoc (thu muc bi giu handle),
    khi do hang so phai giu ten goc de tool con chay dung.
    """
    changed = 0
    for path in tool_files():
        text = read_text(path)
        new_text = text
        for original, planned in MODULE_TARGET.items():
            if planned == original:
                continue
            actual = planned if os.path.isdir(os.path.join(MS, planned)) else original
            for name in {original, planned} - {actual}:
                if name in new_text:
                    new_text = new_text.replace(name, actual)
        if new_text != text:
            if apply:
                write_text(path, new_text)
            changed += 1
            print(f"  tool dong bo        : {os.path.relpath(path, ROOT)}")
    return changed


def finish_module(old: str, new: str) -> int:
    """Doi ten mot thu muc module + viet lai tham chieu (dung cho M00 sau khi het khoa)."""
    src, dst = os.path.join(MS, old), os.path.join(MS, new)
    if os.path.isdir(dst) and not os.path.isdir(src):
        print(f"module da o ten moi: {new}")
    else:
        if not os.path.isdir(src):
            print(f"khong thay module: {src}")
            return 1
        if os.path.exists(dst):
            print(f"dich da ton tai: {dst}")
            return 1
        os.rename(src, dst)
        print(f"renamed module       : {old} -> {new}")
    pairs = [(old, new)]
    changed = 0
    for path in target_files():
        text = read_text(path)
        new_text, hits = rewrite_text(text, pairs)
        if hits and new_text != text:
            write_text(path, new_text)
            changed += 1
    print(f"files rewritten      : {changed}")
    patched = sync_tools()
    print(f"tool patches applied : {patched}")
    index_path = os.path.join(MS, "PATH_RENAMES.json")
    if os.path.isfile(index_path):
        index = json.load(open(index_path, encoding="utf-8"))
        for row in index["modules"]:
            if row["old"] == old:
                row["new"] = new
                row["applied"] = True
        index["notes"] = [n for n in index.get("notes", []) if "M00" not in n]
        write_index(index)
        print("index updated        : OM9_Milestones/PATH_RENAMES.json + .md")
    total, dangling, samples = audit_references(target_files())
    print(f"references           : {total}  dangling: {dangling}")
    for s in samples[:3]:
        print(f"    DANGLING {s}")
    return 0 if dangling == 0 else 4


def finalize() -> int:
    """Chuan hoa ten module (ke ca sua khoang trang) + viet lai tham chieu + dong bo tools.

    Dung khi thu muc module da duoc doi ten ben ngoai tool nay (vi du bang
    Explorer) hoac khi ten bi lech khoang trang.
    """

    def norm(value: str) -> str:
        return " ".join(value.split()).casefold()

    drift_pairs: list[tuple[str, str]] = []
    actual_names: dict[str, str | None] = {}
    for original, planned in MODULE_TARGET.items():
        if not os.path.isdir(os.path.join(MS, original)) and os.path.isdir(os.path.join(MS, planned)):
            actual_names[original] = planned
            continue
        if planned == original:
            actual_names[original] = original if os.path.isdir(os.path.join(MS, original)) else None
            continue
        target = os.path.join(MS, planned)
        if os.path.isdir(target):
            actual_names[original] = planned
            continue
        match = None
        for name in sorted(os.listdir(MS)):
            candidate = os.path.join(MS, name)
            if os.path.isdir(candidate) and norm(name) == norm(planned):
                match = (candidate, name)
                break
        if match:
            try:
                os.rename(match[0], target)
            except OSError as exc:
                print(f"  KHONG SUA DUOC TEN {match[1]!r}: {exc}")
                actual_names[original] = match[1]
                continue
            print(f"  chuan hoa ten module: {match[1]!r} -> {planned!r}")
            drift_pairs.append((match[1], planned))
            actual_names[original] = planned
        elif os.path.isdir(os.path.join(MS, original)):
            actual_names[original] = original
            print(f"  module giu ten goc   : {original}")
        else:
            actual_names[original] = None
            print(f"  CANH BAO: khong thay module {original!r} (ke ca {planned!r})")

    pairs = drift_pairs + [
        (original, actual)
        for original, actual in actual_names.items()
        if actual and actual != original
    ]
    changed = 0
    for path in target_files():
        text = read_text(path)
        new_text, hits = rewrite_text(text, pairs)
        if hits and new_text != text:
            write_text(path, new_text)
            changed += 1
    print(f"  tham chieu da sua    : {changed} file ({len(pairs)} cap ten)")
    print(f"  tools dong bo        : {sync_tools()} file")

    index_path = os.path.join(MS, "PATH_RENAMES.json")
    if os.path.isfile(index_path):
        index = json.load(open(index_path, encoding="utf-8"))
        for row in index["modules"]:
            actual = actual_names.get(row["old"])
            if actual:
                row["new"] = actual
                row["applied"] = actual == row.get("planned_new", actual)
                row["blocked_reason"] = None if row["applied"] else "giu ten goc"
        index["notes"] = [note for note in index.get("notes", []) if "--finish-module" not in note]
        index["notes"].append(
            "Ten module con lai da doi ben ngoai tool; dung "
            "`python implementation/tools/om9_shorten_milestone_paths.py --finalize` de dong bo lai."
        )
        index["created_utc"] = dt.datetime.now(dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
        write_index(index)
        print("  index cap nhat       : OM9_Milestones/PATH_RENAMES.json + .md")

    files_after = [os.path.join(dp, f) for dp, _, fn in os.walk(MS) for f in fn]
    lens = sorted(len(p) for p in files_after)
    over = [p for p in files_after if len(p) > BUDGET]
    bad_json = []
    for dp, _, fn in os.walk(MS):
        for f in fn:
            if f.lower().endswith(".json"):
                p = os.path.join(dp, f)
                try:
                    json.load(open(p, encoding="utf-8"))
                except ValueError as exc:
                    bad_json.append(f"{os.path.relpath(p, ROOT)}: {exc}")
    stale = []
    for path in target_files():
        text = read_text(path)
        for old, new in pairs:
            if old != new and old in text:
                stale.append(f"{os.path.relpath(path, ROOT)}: {old}")
                break
    total_ref, dangling, samples = audit_references(target_files())
    print(f"  files={len(files_after)} max={max(lens)} over_budget={len(over)}")
    print(f"  json_errors={len(bad_json)} stale={len(stale)} references={total_ref} dangling={dangling}")
    for item in stale[:5]:
        print(f"    STALE {item}")
    for item in samples[:5]:
        print(f"    DANGLING {item}")
    ok = not over and not bad_json and not stale and dangling == 0
    print("FINALIZE:", "PASS" if ok else "CHECK")
    return 0 if ok else 4


def target_files() -> list[str]:
    out: list[str] = []
    for dp, dn, fn in os.walk(MS):
        for f in fn:
            if f.lower().endswith(REWRITE_SUFFIXES) and f not in INDEX_NAMES:
                out.append(os.path.join(dp, f))
    out.append(os.path.join(ROOT, "docs", "openmatrix9-progress.json"))
    return [p for p in out if os.path.isfile(p)]


def apply_tool_patches(apply: bool) -> tuple[int, list[str]]:
    """Sua cac hang so tro vao cay cuc bo trong tools/ (xem TOOL_PATCHES).

    Chay lai duoc: neu chuoi cu da duoc thay truoc do thi tinh la da xong.
    """
    problems: list[str] = []
    changed = 0
    for rel, old, new, expected in TOOL_PATCHES:
        path = os.path.join(ROOT, rel.replace("/", os.sep))
        text = read_text(path)
        count = text.count(old)
        if count == expected:
            if apply:
                write_text(path, text.replace(old, new))
            changed += 1
        elif count == 0 and new in text:
            changed += 1  # da ap dung o lan chay truoc
        else:
            problems.append(f"{rel}: mong doi {expected} lan, thay {count} lan: {old[:60]}")
    return changed, problems


def replacement_pairs(renames: dict[str, str]) -> list[tuple[str, str]]:
    pairs: list[tuple[str, str]] = []
    for old, new in ALIAS.items():
        if old != new:
            pairs.append((old, new))
    for old_path, new_name in renames.items():
        old_name = os.path.basename(old_path)
        if old_name != new_name:
            pairs.append((old_name, new_name))
    return pairs


def rewrite_text(text: str, pairs: list[tuple[str, str]]) -> tuple[str, int]:
    hits = 0
    for old, new in pairs:
        if old in text:
            hits += text.count(old)
            text = text.replace(old, new)
        quoted = urllib.parse.quote(old)
        if quoted != old and quoted in text:
            hits += text.count(quoted)
            text = text.replace(quoted, urllib.parse.quote(new))
    return text, hits


# --------------------------------------------------------------------------
# dangling reference audit
# --------------------------------------------------------------------------
REF_RE = re.compile(r"OM9_Milestones[/\\]([^\"'<>\r\n|*?]+)")


def read_text(path: str) -> str:
    """Doc dang bytes -> utf-8, giu nguyen BOM va kieu xuong dong."""
    return open(path, "rb").read().decode("utf-8", errors="surrogateescape")


def write_text(path: str, text: str) -> None:
    with open(path, "wb") as fh:
        fh.write(text.encode("utf-8", errors="surrogateescape"))


def referenced_paths(text: str) -> list[str]:
    found = []
    for m in REF_RE.finditer(text):
        raw = m.group(1)
        # cat phan van xuoi dinh sau duong dan (": 8 tai lieu...", " (manifest...")
        for marker in (": ", " (", "``", ", "):
            cut = raw.find(marker)
            if cut > 0:
                raw = raw[:cut]
        raw = raw.rstrip(" \\,;:).]")
        if raw:
            found.append(raw)
    return found


def audit_references(files: list[str]) -> tuple[int, int, list[str]]:
    """(so tham chieu that, so tham chieu treo, vi du).

    Chi tinh tham chieu ma phan doan dau tien ton tai that trong cay, de bo qua
    van xuoi / vi du viet tat (``M09 .../reference/...``).
    """
    total = 0
    dangling: list[str] = []
    for path in files:
        try:
            text = read_text(path)
        except OSError:
            continue
        for raw in referenced_paths(text):
            first = re.split(r"[/\\]", raw, maxsplit=1)[0]
            if not os.path.exists(os.path.join(MS, first)):
                continue
            total += 1
            full = os.path.join(MS, raw.replace("\\", os.sep).replace("/", os.sep))
            if not os.path.exists(full):
                dangling.append(f"{os.path.relpath(path, ROOT)} -> {raw}")
    return total, len(dangling), dangling[:10]


# --------------------------------------------------------------------------
# main
# --------------------------------------------------------------------------
def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--apply", action="store_true")
    ap.add_argument(
        "--finish-module",
        metavar="OLD=NEW",
        help="doi ten mot thu muc module (vi du M00) va viet lai tham chieu",
    )
    ap.add_argument(
        "--sync-tools",
        action="store_true",
        help="dong bo hang so ten module trong tools/ voi ten thu muc thuc te",
    )
    ap.add_argument(
        "--finalize",
        action="store_true",
        help="chuan hoa ten module (sua khoang trang), viet lai tham chieu, dong bo tools",
    )
    args = ap.parse_args()

    if args.finalize:
        return finalize()

    if args.sync_tools:
        changed = sync_tools(apply=True)
        print(f"tool files synced: {changed}")
        return 0

    if args.finish_module:
        old, _, new = args.finish_module.partition("=")
        if not old or not new:
            print("can dang OLD=NEW")
            return 2
        return finish_module(old, new)

    files = [os.path.join(dp, f) for dp, _, fn in os.walk(MS) for f in fn]
    before = sorted(len(p) for p in files)
    plan = build_plan(files)
    renames = plan["renames"]

    def buckets(vals):
        b = collections.Counter()
        for v in vals:
            k = (">=240" if v >= 240 else "230-239" if v >= 230 else "220-229" if v >= 220
                 else "210-219" if v >= 210 else "<210")
            b[k] += 1
        return {k: b[k] for k in ("<210", "210-219", "220-229", "230-239", ">=240") if b[k]}

    projected = sorted(len(renamed_path(p, renames)) for p in files)
    print(f"files                : {len(files)}")
    print(f"before               : max={max(before)} {buckets(before)}")
    print(f"after (projected)    : max={max(projected)} {buckets(projected)}")
    print(f"renamed dirs         : {len(renames)} (+11 module aliases)")
    per_module = collections.Counter(rel_parts(k)[0].split(" ")[0] for k in renames)
    print(f"per module           : {dict(sorted(per_module.items()))}")
    print(f"unresolved paths     : {len(plan['unresolved'])}")
    for item in plan["unresolved"][:5]:
        print(f"    UNRESOLVED {item}")

    collisions = []
    siblings = collections.defaultdict(list)
    for old, new in renames.items():
        siblings[(os.path.dirname(old).casefold(), new.casefold())].append(old)
    collisions = [k for k, v in siblings.items() if len(v) > 1]
    print(f"name collisions      : {len(collisions)}")
    for c in collisions[:5]:
        print(f"    {c}")

    pairs = replacement_pairs(renames)
    targets = target_files()
    affected, total_hits = [], 0
    for path in targets:
        try:
            text = read_text(path)
        except OSError:
            continue
        new_text, hits = rewrite_text(text, pairs)
        if hits:
            affected.append((os.path.relpath(path, ROOT), hits))
            total_hits += hits
    print(f"files to rewrite     : {len(affected)}  (replacements: {total_hits})")
    per_top = collections.Counter(a.split(os.sep)[0] for a, _ in affected)
    print(f"  by top dir         : {dict(per_top)}")

    # tham chieu treo: hien tai va sau khi doi ten (mo phong)
    total_ref, dangling_now, samples_now = audit_references(targets)
    print(f"references audited   : {total_ref}  dangling now: {dangling_now}")
    for s in samples_now[:5]:
        print(f"    DANGLING(now) {s}")

    if not args.apply:
        changed_tools, tool_problems = apply_tool_patches(apply=False)
        print(f"tool patches         : {changed_tools}/{len(TOOL_PATCHES)} khop")
        for p in tool_problems:
            print(f"    TOOL PROBLEM {p}")
        excluded = scan_excluded(pairs)
        print(f"ten cu con lai (lich su/tools): {len(excluded)} file")
        for k, v in sorted(excluded.items(), key=lambda kv: -kv[1])[:12]:
            print(f"    {v:4d}x {k}")
        # mo phong: sau khi doi ten + viet lai, moi tham chieu co con giai duoc khong?
        future = set()
        for dp, dn, fn in os.walk(MS):
            for name in list(fn) + list(dn):
                future.add(renamed_path(os.path.join(dp, name), renames).casefold())
        dangling_proj = 0
        examples_proj = []
        for path in targets:
            try:
                new_text, hits = rewrite_text(read_text(path), pairs)
            except OSError:
                continue
            if not hits:
                new_text = read_text(path)
            for raw in referenced_paths(new_text):
                first = re.split(r"[/\\]", raw, maxsplit=1)[0]
                if not os.path.exists(os.path.join(MS, first)) and first not in ALIAS:
                    continue
                full = os.path.join(MS, raw.replace("\\", os.sep).replace("/", os.sep))
                if not os.path.exists(full) and full.casefold() not in future:
                    dangling_proj += 1
                    if len(examples_proj) < 5:
                        examples_proj.append(f"{os.path.relpath(path, ROOT)} -> {raw}")
        print(f"references dangling after rename (projected): {dangling_proj}")
        for s in examples_proj:
            print(f"    DANGLING(after) {s}")
        print("\n[dry-run] khong ghi gi. Dung --apply de thuc thi.")
        os.makedirs(os.path.dirname(PLAN_OUT), exist_ok=True)
        with open(PLAN_OUT, "w", encoding="utf-8") as fh:
            json.dump(
                {
                    "budget": BUDGET,
                    "aliases": ALIAS,
                    "renames": {os.path.relpath(k, MS).replace(os.sep, "/"): v for k, v in renames.items()},
                },
                fh,
                ensure_ascii=False,
                indent=1,
            )
        print(f"plan -> {PLAN_OUT}")
        return 0

    # ---------------- apply: phase 1 — doi ten module (thich ung voi thu muc bi khoa) ----------------
    applied_modules: dict[str, str] = {}
    failed_modules: dict[str, str] = {}
    locked_dirs: set[str] = set()
    renamed_now: list[tuple[str, str]] = []
    module_targets = [(o, n) for o, n in ALIAS.items() if o != n]
    module_result = execute_renames(MS, {}, dict(module_targets), skip_locked=True)
    for src, dst, ok, err in module_result:
        old = os.path.basename(src)
        if ok:
            applied_modules[old] = os.path.basename(dst)
            renamed_now.append((src, dst))
            print(f"  module renamed      : {old} -> {os.path.basename(dst)}")
        else:
            failed_modules[old] = err
            locked_dirs.add(src)
            print(f"  module BI KHOA      : {old}\n      ly do: {err}")
    for old, new in module_targets:
        src, dst = os.path.join(MS, old), os.path.join(MS, new)
        if os.path.isdir(dst) and not os.path.isdir(src):
            applied_modules.setdefault(old, new)
            print(f"  module da o ten moi : {new}")

    # ---------------- phase 2/3 — lap ke hoach va cat thu muc con, bo qua thu muc bi khoa ----------------
    attempts = 0
    remaining: list[tuple[int, str]] = []
    inner_applied: list[tuple[str, str]] = []   # (duong dan luc doi, duong dan moi)
    while attempts < 8:
        attempts += 1
        files_now = [os.path.join(dp, f) for dp, _, fn in os.walk(MS) for f in fn]
        plan = build_plan(files_now, alias={}, skip_dirs=locked_dirs)
        renames = plan["renames"]
        remaining = plan["unresolved"]
        if not renames:
            break
        result = execute_renames(MS, renames, {}, skip_locked=True)
        applied = [(s, d) for s, d, ok, _e in result if ok]
        newly_locked = [s for s, _d, ok, _e in result if not ok]
        inner_applied.extend(applied)
        print(f"  vong {attempts}: doi {len(applied)} thu muc, khoa them {len(newly_locked)}, "
              f"con {len(remaining)} duong qua ngan sach")
        if not newly_locked:
            break
        locked_dirs.update(newly_locked)
    print(f"  thu muc con da cat  : {len(inner_applied)}")
    if remaining:
        print(f"  CON DAI (khong cat them duoc vi thu muc bi khoa): {len(remaining)}")
        for item in remaining[:10]:
            print(f"    {item[0]:4d}  {item[1]}")

    files = [os.path.join(dp, f) for dp, _, fn in os.walk(MS) for f in fn]

    # cap doi ten de viet lai tham chieu: gop voi index cu (chay lai an toan)
    index_path = os.path.join(MS, "PATH_RENAMES.json")
    prior = {}
    if os.path.isfile(index_path):
        try:
            prior = json.load(open(index_path, encoding="utf-8"))
        except ValueError:
            prior = {}
    merged_modules: dict[str, dict] = {row["old"]: dict(row) for row in prior.get("modules", [])}
    for key, target in MODULE_TARGET.items():
        row = merged_modules.setdefault(
            key, {"old": key, "new": key, "applied": False, "planned_new": target}
        )
        row["planned_new"] = target
        if applied_modules.get(key) == target:
            row["applied"] = True
            row["new"] = target
            row["blocked_reason"] = None
        elif target != key and not row.get("applied"):
            row["new"] = key
            row["blocked_reason"] = failed_modules.get(key) or "thu muc bi giu handle"
    merged_dirs: list[dict] = [dict(row) for row in prior.get("dirs", [])]
    known_dir_keys = {(row.get("old_path"), row.get("new_name")) for row in merged_dirs}

    def original_path(live: str) -> str:
        inverse: dict[str, str] = {}
        for row in list(merged_modules.values()) + [
            {"old": os.path.basename(r["old_path"]), "new": r["new_name"], "applied": True}
            for r in merged_dirs
            if r.get("old_path")
        ]:
            if row.get("applied") and row.get("old") != row.get("new"):
                inverse.setdefault(row["new"], row["old"])
        parts = os.path.relpath(live, MS).split(os.sep)
        return "/".join(inverse.get(p, p) for p in parts)

    titles_now = {}
    for dp, _, fn in os.walk(MS):
        for f in fn:
            if f in ("L1.json", "L2.json", "l3.json", "l4.json"):
                try:
                    data = json.load(open(os.path.join(dp, f), encoding="utf-8"))
                except (OSError, ValueError):
                    continue
                titles_now[dp] = {"id": data.get("id"), "title": data.get("title")}

    for src, dst in sorted(inner_applied, key=lambda item: item[0]):
        meta = titles_now.get(dst, {})
        old_path = original_path(src)
        new_name = os.path.basename(dst)
        if (old_path, new_name) in known_dir_keys:
            continue
        known_dir_keys.add((old_path, new_name))
        merged_dirs.append(
            {
                "old_path": old_path,
                "new_name": new_name,
                "level": os.path.basename(src).split("-")[0],
                "id": meta.get("id"),
                "full_title": meta.get("title"),
            }
        )

    pairs = [
        (row["old"], row["new"])
        for row in merged_modules.values()
        if row.get("applied") and row.get("old") != row.get("new")
    ]
    pairs += [
        (os.path.basename(row["old_path"]), row["new_name"])
        for row in merged_dirs
        if row.get("old_path")
        and os.path.basename(row["old_path"]) != row["new_name"]
    ]

    patched, tool_problems = apply_tool_patches(apply=True)
    print(f"tool patches applied : {patched}/{len(TOOL_PATCHES)}")
    for p in tool_problems:
        print(f"    TOOL PROBLEM {p}")
    if tool_problems:
        print("ABORT: hang so trong tools/ khong khop, kiem tra lai truoc khi viet tiep")
        return 5

    changed = 0
    for path in target_files():
        try:
            text = read_text(path)
        except OSError:
            continue
        new_text, hits = rewrite_text(text, pairs)
        if hits and new_text != text:
            write_text(path, new_text)
            changed += 1
    print(f"files rewritten      : {changed}")

    # ---------------- index ----------------
    index = {
        "schema": "om9.path_renames.v1",
        "created_utc": dt.datetime.now(dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "reason": "Windows MAX_PATH: giu moi duong dan trong OM9_Milestones <= 210 ky tu",
        "budget": BUDGET,
        "note": "Tieu de day du khong mat: van nam trong L1.json / L2.json / l3.json / l4.json tuong ung.",
        "notes": [],
        "modules": sorted(merged_modules.values(), key=lambda row: row["old"]),
        "dirs": sorted(merged_dirs, key=lambda row: (row.get("old_path") or "")),
    }
    for row in index["modules"]:
        if not row.get("applied") and row.get("planned_new") != row["old"]:
            index["notes"].append(
                f"`{row['old']}` chua doi duoc trong phien nay (thu muc bi giu handle); "
                f"ten du kien: `{row['planned_new']}`. Hoan tat sau: "
                f'`python implementation/tools/om9_shorten_milestone_paths.py '
                f'--finish-module "{row["old"]}={row["planned_new"]}"`.')
    if remaining:
        index["notes"].append(
            f"{len(remaining)} duong dan van dai hon {BUDGET} ky tu vi thu muc cha bi giu handle.")
    write_index(index)
    print("index written        : OM9_Milestones/PATH_RENAMES.json + .md")

    # ---------------- verify ----------------
    files_after = [os.path.join(dp, f) for dp, _, fn in os.walk(MS) for f in fn]
    lens = sorted(len(p) for p in files_after)
    over = [p for p in files_after if len(p) > BUDGET]
    bad_json = []
    for dp, _, fn in os.walk(MS):
        for f in fn:
            if f.lower().endswith(".json"):
                p = os.path.join(dp, f)
                try:
                    json.load(open(p, encoding="utf-8"))
                except ValueError as exc:
                    bad_json.append(f"{os.path.relpath(p, ROOT)}: {exc}")
    stale = 0
    stale_examples: list[str] = []
    for path in target_files():
        text = read_text(path)
        for old, new in pairs:
            if old != new and old in text:
                stale += 1
                if len(stale_examples) < 5:
                    stale_examples.append(f"{os.path.relpath(path, ROOT)}: {old}")
                break
    total_ref, dangling_after, samples_after = audit_references(target_files())
    ledger = os.path.join(ROOT, "docs", "openmatrix9-progress.json")
    ledger_ok = True
    try:
        json.load(open(ledger, encoding="utf-8"))
    except ValueError as exc:
        ledger_ok = False
        print(f"LEDGER JSON LOI: {exc}")
    index_paths = {os.path.join(MS, name) for name in INDEX_NAMES}
    new_files = sorted(set(files_after) - set(files))
    lost_files = sorted(set(files) - set(files_after))
    report = {
        "applied_utc": dt.datetime.now(dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "files_before": len(before),
        "files_after": len(files_after),
        "max_path_before": max(before),
        "max_path_after": max(lens),
        "buckets_after": buckets(lens),
        "paths_over_budget": [os.path.relpath(p, MS) for p in over[:20]],
        "inner_dirs_renamed": len(inner_applied),
        "modules_renamed": len(applied_modules),
        "modules_blocked": {k: v for k, v in failed_modules.items()},
        "files_rewritten": changed,
        "json_parse_errors": bad_json,
        "files_with_stale_names": stale,
        "stale_examples": stale_examples[:5],
        "files_added": [os.path.relpath(p, ROOT) for p in new_files],
        "files_lost": [os.path.relpath(p, ROOT) for p in lost_files],
        "references_total": total_ref,
        "references_dangling_before": dangling_now,
        "references_dangling_after": dangling_after,
        "dangling_examples_after": samples_after,
        "ledger_json_ok": ledger_ok,
    }
    print(json.dumps(report, ensure_ascii=False, indent=1))
    os.makedirs(os.path.dirname(REPORT_OUT), exist_ok=True)
    with open(REPORT_OUT, "w", encoding="utf-8") as fh:
        fh.write("# Rút ngắn đường dẫn OM9_Milestones (MAX_PATH)\n\n")
        fh.write("```json\n" + json.dumps(report, ensure_ascii=False, indent=1) + "\n```\n")
    ok = (
        not over
        and not bad_json
        and ledger_ok
        and stale == 0
        and dangling_after <= dangling_now
        and not lost_files
        and set(new_files) <= index_paths
    )
    print("VERIFY:", "PASS" if ok else "FAIL")
    return 0 if ok else 4


if __name__ == "__main__":
    sys.exit(main())

