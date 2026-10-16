"""Fail-closed check that the algorithm notes contain no copied Matrix source.

The notes are supposed to be authored prose derived from reading the decompile. This
proves it mechanically: every note is compared against the source files it cites, and
any run of consecutive words long enough to constitute copying fails the check.

It also verifies that each cited source file and method name actually exists, so a
note cannot cite an operation that was never there.
"""
import json
import re
import sys
from pathlib import Path

ROOT = Path("H:/OpenMatrix9")
CORE = ROOT / "ref/matrix/decompiled-ui/Matrix"
NOTES = ROOT / "docs/private-algorithm-notes.json"

# consecutive-word run that counts as copying. 12 words is well below any legitimate
# shared phrasing between prose and C# code, and well above incidental overlap.
RUN = 12


def words(text):
    return re.findall(r"[A-Za-z_][A-Za-z0-9_]*", text.lower())


def shingles(seq, n):
    return {" ".join(seq[i:i + n]) for i in range(max(0, len(seq) - n + 1))}


def main():
    payload = json.loads(NOTES.read_text(encoding="utf-8"))
    errors = []
    checked_sources = 0

    for note in payload["notes"]:
        src = note["source"]
        path = CORE / src["file"]
        if not path.is_file():
            errors.append(f"{note['id']}: cited file does not exist: {src['file']}")
            continue
        checked_sources += 1
        text = path.read_text(encoding="utf-8", errors="replace")
        src_words = words(text)
        src_shingles = shingles(src_words, RUN)

        # every prose field of the note must be free of long verbatim runs
        prose = []
        for key in ("verified_facts", "algorithm", "boundaries"):
            prose.extend(note.get(key) or [])
        for line in prose:
            note_shingles = shingles(words(line), RUN)
            overlap = note_shingles & src_shingles
            if overlap:
                sample = next(iter(overlap))
                errors.append(f"{note['id']}: {RUN}-word verbatim run from "
                              f"{src['file']}: {sample[:70]!r}")

        # every cited method name must appear in the cited file
        for method in src.get("methods") or []:
            base = re.split(r"[ (:]", method)[0]
            if base and base not in text:
                errors.append(f"{note['id']}: method not found in {src['file']}: {method}")
        for entry in src.get("also") or []:
            other, _, meth = entry.partition(":")
            other_path = CORE / other
            if not other_path.is_file():
                errors.append(f"{note['id']}: 'also' file missing: {other}")
                continue
            other_text = other_path.read_text(encoding="utf-8", errors="replace")
            for name in (m.strip() for m in meth.split(",") if m.strip()):
                if name not in other_text:
                    errors.append(f"{note['id']}: 'also' method not found in {other}: {name}")

    # the notes must carry their own boundaries; an unbounded claim is not admissible
    for note in payload["notes"]:
        if not note.get("boundaries"):
            errors.append(f"{note['id']}: no boundaries recorded")
        if not note.get("verified_facts"):
            errors.append(f"{note['id']}: no verified facts recorded")

    print(json.dumps({
        "notes": len(payload["notes"]),
        "sources_checked": checked_sources,
        "verbatim_run_threshold_words": RUN,
        "errors": errors,
    }, ensure_ascii=False, indent=1))
    if errors:
        sys.exit(1)


if __name__ == "__main__":
    main()
