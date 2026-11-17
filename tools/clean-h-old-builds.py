#!/usr/bin/env python3
"""Preview or remove old Cargo artifacts from two legacy OM9 targets on H:.

    python tools/clean-h-old-builds.py                 # preview
    python tools/clean-h-old-builds.py --apply         # remove listed artifacts
    python tools/clean-h-old-builds.py --days 30 --apply
    python tools/clean-h-old-builds.py --list          # show each candidate

L:\\om9-target, native SDKs, project files, executables outside deps, and build/
validation evidence are not selected. Cargo may rebuild removed artifacts.
"""
from __future__ import annotations

import argparse
import csv
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import time

ROOTS = (Path(r'H:\OpenMatrix9\target'), Path(r'H:\OpenMatrix9\cargo-target'))
REPARSE = getattr(os.stat_result, 'FILE_ATTRIBUTE_REPARSE_POINT', 0x400)
ARTIFACT = re.compile(r'^.+-[0-9a-f]{16,64}\.(?:d|dll|exe|exp|lib|pdb|rlib|rmeta|o|obj)$', re.I)


def redirected(path: Path) -> bool:
    return any(p.is_symlink() or (p.stat(follow_symlinks=False).st_file_attributes & REPARSE)
               for p in (path, *path.parents) if p.exists())


def cargo_running() -> bool:
    result = subprocess.run(['tasklist', '/FO', 'CSV', '/NH'], capture_output=True, text=True, check=True)
    return any(row and row[0].lower() in {'cargo.exe', 'rustc.exe'} for row in csv.reader(result.stdout.splitlines()))


def old_tree(path: Path, cutoff: float) -> tuple[int, float] | None:
    """Refuse a tree if it contains a link, unreadable item, or recent item."""
    try:
        if redirected(path):
            return None
        newest = path.stat(follow_symlinks=False).st_mtime
        size = 0
        for directory, names, files in os.walk(path, followlinks=False, onerror=lambda error: (_ for _ in ()).throw(error)):
            for name in names + files:
                item = Path(directory) / name
                info = item.stat(follow_symlinks=False)
                if info.st_file_attributes & REPARSE:
                    return None
                newest = max(newest, info.st_mtime)
                if name in files:
                    size += info.st_size
        return (size, newest) if newest < cutoff else None
    except OSError as error:
        print(f'SKIP {path}: {error}', file=sys.stderr)
        return None


def candidates(days: int):
    cutoff = time.time() - days * 86400
    for root in ROOTS:
        if not root.is_dir() or redirected(root):
            continue
        for profile in ('debug', 'release'):
            incremental = root / profile / 'incremental'
            if incremental.is_dir() and not redirected(incremental):
                for entry in incremental.iterdir():
                    if entry.is_dir() and (info := old_tree(entry, cutoff)) is not None:
                        yield 'incremental', entry, info[0], info[1]
            deps = root / profile / 'deps'
            if deps.is_dir() and not redirected(deps):
                for entry in deps.iterdir():
                    try:
                        info = entry.stat(follow_symlinks=False)
                        if entry.is_file() and not info.st_file_attributes & REPARSE and ARTIFACT.fullmatch(entry.name) and info.st_mtime < cutoff:
                            yield 'deps', entry, info.st_size, info.st_mtime
                    except OSError as error:
                        print(f'SKIP {entry}: {error}', file=sys.stderr)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('--days', type=int, default=7, help='minimum age in days (default: 7)')
    parser.add_argument('--apply', action='store_true', help='remove selected artifacts')
    parser.add_argument('--list', action='store_true', help='print every candidate')
    args = parser.parse_args()
    if not 1 <= args.days <= 36500:
        parser.error('--days must be between 1 and 36500')
    if args.apply and cargo_running():
        raise SystemExit('Cargo or rustc is running; stop builds before cleanup.')
    items = list(candidates(args.days))
    for category in ('incremental', 'deps'):
        group = [item for item in items if item[0] == category]
        print(f'{category}: {len(group)} candidates, {sum(item[2] for item in group) / 2**30:.2f} GiB logical size')
    if args.list:
        for _, path, size, _ in items:
            print(f'{size:>14}  {path}')
    if not args.apply:
        print('PREVIEW ONLY. Use --apply to remove the selected artifacts.')
        return 0
    cutoff = time.time() - args.days * 86400
    removed = 0
    for kind, path, size, _ in items:
        try:
            # Restrict all mutations to the same explicit roots and recheck
            # redirection and age immediately before removal.
            resolved = path.resolve(strict=True)
            if not any(root.resolve(strict=True) in resolved.parents for root in ROOTS if root.exists()) or redirected(path):
                raise RuntimeError('path is outside the allowed targets or redirected')
            if kind == 'incremental':
                info = old_tree(path, cutoff)
                if info is None:
                    continue
                size = info[0]
                shutil.rmtree(path)
            else:
                info = path.stat(follow_symlinks=False)
                if info.st_file_attributes & REPARSE or info.st_mtime >= cutoff or not ARTIFACT.fullmatch(path.name):
                    continue
                size = info.st_size
                path.unlink()
            removed += size
        except (OSError, RuntimeError) as error:
            print(f'SKIP {path}: {error}', file=sys.stderr)
    print(f'Removed {removed / 2**30:.2f} GiB logical size; free H: {shutil.disk_usage("H:/").free / 2**30:.2f} GiB')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
