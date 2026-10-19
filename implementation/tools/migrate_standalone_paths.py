"""Một lần: đổi mọi đường dẫn sống từ layout `standalone/` sang layout gốc repo.

Chỉ sửa tham chiếu **đang dùng** (code, build config, tool, CI, README/AGENTS).
KHÔNG sửa bằng chứng/validation lịch sử trong `implementation/evidence`,
`implementation/design`, `docs/validation`, `docs/features`, `docs/superpowers`,
`.agents/**` — những tệp đó ghi lại việc đã xảy ra ở đường dẫn cũ tại thời điểm đó.

    python implementation/tools/migrate_standalone_paths.py --check   # chỉ báo cáo
    python implementation/tools/migrate_standalone_paths.py           # ghi
"""
from __future__ import annotations

import argparse
import pathlib
import re

ROOT = pathlib.Path(__file__).resolve().parents[2]

# Depth changed by one: crates/<crate> -> crates/<crate>.
RUST_RULES = [
    # ancestors().nth(N): the crate now sits one level higher.
    (re.compile(r"\.ancestors\(\)\s*\n(\s*)\.nth\(3\)"), r".ancestors()\n\1.nth(2)"),
    (re.compile(r"\.ancestors\(\)(\s*)\.nth\(3\)"), r".ancestors()\1.nth(2)"),
    # Four levels up becomes three; three becomes two. Sentinels prevent cascades.
    (re.compile(r"(?<!\.)\.\./\.\./\.\./\.\./"), "\x00L4\x00"),
    (re.compile(r'"\.\./\.\./\.\."'), '"\x00J3\x00"'),
    (re.compile(r"\.\./\.\./\.\./"), "../../"),
    (re.compile("\x00L4\x00"), "../../../"),
    (re.compile("\x00J3\x00"), "../.."),
]

PATH_RULES = [
    (re.compile(r"standalone[/\\]crates[/\\]"), "crates/"),
    (re.compile(r"standalone[/\\]native[/\\]"), "native/"),
    (re.compile(r"standalone[/\\]build[/\\]"), "build/"),
    (re.compile(r"standalone[/\\]target[/\\]"), "target/"),
    (re.compile(r"standalone[/\\]Cargo\.toml"), "Cargo.toml"),
    (re.compile(r"standalone[/\\]Cargo\.lock"), "Cargo.lock"),
    (re.compile(r"standalone[/\\]provenance\.json"), "native/provenance.json"),
    (re.compile(r"standalone[/\\]README\.md"), "docs/standalone-foundation.md"),
    # A bare "standalone" path component used as a directory join.
    (re.compile(r"""ROOT / "standalone" """.strip() + r"\n"), "\n"),
]

CMAKELISTS_RULES = [
    (re.compile(r"\$\{CMAKE_CURRENT_SOURCE_DIR\}/\.\./\.\./build/"), "${CMAKE_CURRENT_SOURCE_DIR}/../build/"),
]


def rust_files() -> list[pathlib.Path]:
    return [p for p in (ROOT / "crates").rglob("*.rs")]


def text_files() -> list[pathlib.Path]:
    targets: list[pathlib.Path] = [ROOT / ".gitignore", ROOT / "AGENTS.md", ROOT / "README.md",
                                   ROOT / "README.en.md", ROOT / "README.vi.md",
                                   ROOT / "docs/standalone-foundation.md"]
    for pattern in ("tools/*.py", "implementation/tools/*.py", "implementation/tools/*.ps1"):
        targets.extend(sorted(ROOT.glob(pattern)))
    return [path for path in targets if path.is_file()]


def apply(path: pathlib.Path, rules, dry: bool) -> list[str]:
    original = path.read_text(encoding="utf-8")
    updated = original
    for pattern, replacement in rules:
        updated = pattern.sub(replacement, updated)
    if updated == original:
        return []
    if not dry:
        path.write_text(updated, encoding="utf-8", newline="")
    changed = [
        f"{path.relative_to(ROOT).as_posix()}"
        for _ in [0]
    ]
    return changed


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()

    # tools/verify_m01_l2_01_03.py keeps an explicit checkout constant.
    verify = ROOT / "tools/verify_m01_l2_01_03.py"
    if verify.is_file():
        text = verify.read_text(encoding="utf-8")
        if 'STANDALONE = ROOT / "standalone"' in text:
            if not args.check:
                verify.write_text(
                    text.replace('STANDALONE = ROOT / "standalone"', "STANDALONE = ROOT"),
                    encoding="utf-8",
                    newline="",
                )
            print("tools/verify_m01_l2_01_03.py: STANDALONE -> ROOT")

    changed: list[str] = []
    for path in rust_files():
        changed += apply(path, RUST_RULES, args.check)
    cmake = ROOT / "native/CMakeLists.txt"
    if cmake.is_file():
        changed += apply(cmake, CMAKELISTS_RULES + PATH_RULES, args.check)
    for path in text_files():
        changed += apply(path, PATH_RULES, args.check)

    print(f"{'would change' if args.check else 'changed'}: {len(changed)} files")
    for name in sorted(set(changed)):
        print("  ", name)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
