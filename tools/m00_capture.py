#!/usr/bin/env python
"""Capture one real command as M00 evidence.

    python tools/m00_capture.py --task OM9-M00-L4-01-01-02 --cwd QymCAD --label head -- git rev-parse HEAD

Writes `<stdout>.log` / `<stderr>.log` under `implementation/evidence/M00/<gate>/capture/` and prints the
command, exit code and the sha256 of both files, so an evidence record can quote real digests.
"""
from __future__ import annotations

import argparse
import pathlib
import subprocess
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import m00_common as m00


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--task", required=True)
    parser.add_argument("--cwd", default=".")
    parser.add_argument("--label", default="capture")
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    command = args.command[1:] if args.command and args.command[0] == "--" else args.command
    if not command:
        print("no command given")
        return 2
    entry = next((item for item in m00.iter_l4() if item["id"] == args.task), None)
    if entry is None:
        print(f"{args.task} is not an M00 L4")
        return 2
    folder = m00.EVIDENCE / entry["gate"] / "capture"
    folder.mkdir(parents=True, exist_ok=True)
    safe = "".join(character if character.isalnum() or character in "-_" else "_" for character in args.label)
    stdout_path = folder / f"{args.task}.{safe}.stdout.log"
    stderr_path = folder / f"{args.task}.{safe}.stderr.log"
    result = subprocess.run(command, cwd=args.cwd, capture_output=True, text=True, encoding="utf-8")
    stdout_path.write_text(result.stdout, encoding="utf-8")
    stderr_path.write_text(result.stderr, encoding="utf-8")
    print(f"command: {' '.join(command)}")
    print(f"exit_code: {result.returncode}")
    print(f"stdout: {stdout_path.relative_to(m00.ROOT)} sha256={m00.sha256(stdout_path)}")
    print(f"stderr: {stderr_path.relative_to(m00.ROOT)} sha256={m00.sha256(stderr_path)}")
    return result.returncode
