#!/usr/bin/env python
"""Giữ lại N thư mục build/validation gần nhất, xoá phần cũ hơn.

Rule của dự án: chỉ giữ **10** run gần nhất để dung lượng không phình (mỗi run UI
artifact có thể vài chục MB; `build/` là thư mục gitignored, không phải bằng chứng
bền vững). Bản ghi bằng chứng trong `implementation/evidence/**` chỉ *dẫn chiếu*
đường dẫn artifact và không chứa artifact; run cũ bị xoá thì dẫn chiếu đó thành
lịch sử, không phải nguồn sống.

    python tools/prune_build_artifacts.py             # xem sẽ xoá gì (dry-run)
    python tools/prune_build_artifacts.py --apply     # xoá thật
    python tools/prune_build_artifacts.py --keep 5 --apply
"""
from __future__ import annotations

import argparse
import pathlib
import shutil

ROOT = pathlib.Path(__file__).resolve().parents[1]
DEFAULT_KEEP = 10

# Each entry: a directory whose *children* are independent runs; keep the newest ones.
RUN_ROOTS = [
    ROOT / "build/standalone-validation/ui-artifacts",
]

# Cargo target directories: keep only the newest N (normally there is exactly one).
TARGET_GLOBS = ["om9-target*"]


def size_mb(path: pathlib.Path) -> float:
    total = 0
    for item in path.rglob("*"):
        try:
            if item.is_file():
                total += item.stat().st_size
        except OSError:
            continue
    return total / (1024 * 1024)


def newest_first(entries: list[pathlib.Path]) -> list[pathlib.Path]:
    return sorted(entries, key=lambda path: path.stat().st_mtime, reverse=True)


def cited_runs() -> set[str]:
    """Run names that current evidence/design records still cite.

    A run cited from `implementation/**` is never pruned: the retention rule is
    about disk space, not about invalidating a record that points at it.
    """
    cited: set[str] = set()
    for base in (ROOT / "implementation", ROOT / "docs"):
        for path in base.rglob("*"):
            if not path.is_file() or path.suffix.lower() not in {".json", ".md", ".txt"}:
                continue
            try:
                text = path.read_text(encoding="utf-8", errors="ignore")
            except OSError:
                continue
            for marker in ("ui-artifacts/", "ui-artifacts\\"):
                start = 0
                while (at := text.find(marker, start)) != -1:
                    start = at + len(marker)
                    name = ""
                    for char in text[start:]:
                        if char in "/\\\"' \n\r\t),;]":
                            break
                        name += char
                    if name:
                        cited.add(name)
    return cited


def prune(directory: pathlib.Path, keep: int, apply: bool, protect: set[str]) -> tuple[int, float]:
    if not directory.is_dir():
        return 0, 0.0
    runs = newest_first([entry for entry in directory.iterdir() if entry.is_dir()])
    stale = [path for path in runs[keep:] if path.name not in protect]
    freed = sum(size_mb(path) for path in stale)
    for path in stale:
        print(("removing " if apply else "would remove ") + str(path.relative_to(ROOT)))
        if apply:
            shutil.rmtree(path, ignore_errors=True)
    kept_cited = [path.name for path in runs[keep:] if path.name in protect]
    if kept_cited:
        print(f"kept {len(kept_cited)} old run(s) still cited by a record: {', '.join(sorted(kept_cited))}")
    return len(stale), freed


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--keep", type=int, default=DEFAULT_KEEP, help="how many newest runs to keep")
    parser.add_argument("--apply", action="store_true", help="delete instead of listing")
    args = parser.parse_args()
    if args.keep < 1:
        parser.error("--keep must be at least 1")

    total_runs = 0
    total_mb = 0.0
    protect = cited_runs()
    for directory in RUN_ROOTS:
        runs, freed = prune(directory, args.keep, args.apply, protect)
        total_runs += runs
        total_mb += freed
    for pattern in TARGET_GLOBS:
        targets = newest_first([path for path in ROOT.parent.glob(pattern) if path.is_dir()])
        for path in targets[args.keep:]:
            print(("removing " if args.apply else "would remove ") + str(path))
            if args.apply:
                shutil.rmtree(path, ignore_errors=True)
            total_runs += 1

    verb = "removed" if args.apply else "would remove"
    print(f"{verb} {total_runs} run(s), {total_mb:.1f} MB; kept the newest {args.keep} per root")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
