"""Prove the algorithm-notes validator actually detects copying.

Takes a real 14-word run out of a cited decompiled file, injects it into a copy of the
notes, and confirms the validator fails. Then restores the notes. This is the probe
that makes a clean validator result meaningful.
"""
import json
import re
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path("H:/OpenMatrix9")
CORE = ROOT / "ref/matrix/decompiled-ui/Matrix"
NOTES = ROOT / "docs/private-algorithm-notes.json"
BACKUP = ROOT / "build" / "private-algorithm-notes.probe-backup.json"

sys.path.insert(0, str(ROOT / "tools"))
import validate_algorithm_notes as V  # noqa: E402


def main():
    payload = json.loads(NOTES.read_text(encoding="utf-8"))
    note = payload["notes"][0]
    src_path = CORE / note["source"]["file"]
    src_words = V.words(src_path.read_text(encoding="utf-8", errors="replace"))

    # pick a 14-word run from the source that is plain code/identifier text
    run = None
    for i in range(len(src_words) - 14):
        candidate = src_words[i:i + 14]
        if len(set(candidate)) >= 6:
            run = " ".join(candidate)
            break
    assert run, "could not build a probe run"

    shutil.copy2(NOTES, BACKUP)
    try:
        note["verified_facts"].append(run)
        NOTES.write_text(json.dumps(payload, ensure_ascii=False, indent=1) + "\n",
                         encoding="utf-8")
        proc = subprocess.run([sys.executable, "tools/validate_algorithm_notes.py"],
                              cwd=ROOT, capture_output=True, text=True)
        caught = proc.returncode != 0
        detail = ""
        out = proc.stdout or ""
        if "verbatim run" in out:
            detail = "reported a verbatim run"
        print(json.dumps({
            "probe_run_words": 14,
            "probe_text_prefix": run[:60],
            "validator_exit": proc.returncode,
            "violation_detected": caught,
            "how": detail or "(exit non-zero)",
        }, ensure_ascii=False, indent=1))
        if not caught:
            print("PROBE FAILED: validator did not detect injected copying")
            sys.exit(1)
        print("PROBE OK: validator detects injected copying")
    finally:
        shutil.copy2(BACKUP, NOTES)
        BACKUP.unlink(missing_ok=True)
        after = subprocess.run([sys.executable, "tools/validate_algorithm_notes.py"],
                              cwd=ROOT, capture_output=True, text=True)
        print(f"restored; validator after restore exit={after.returncode}")
        if after.returncode != 0:
            sys.exit(1)


if __name__ == "__main__":
    main()
