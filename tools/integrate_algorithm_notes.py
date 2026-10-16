"""Merge completed algorithm notes into the authored notes document.

Two inputs are supported:

1. ``--json FILE``  a JSON file holding a list of note objects with the same schema as
   the notes document. This is how a subagent report is integrated once it has been
   converted into the authored schema (facts / steps / boundaries).
2. ``--list``       prints the current notes and their coverage of the build chain.

The point of this tool is that no note enters the document without a source citation,
a fact list and a boundary list, so ``validate_algorithm_notes.py`` can always check it.
"""
import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path("H:/OpenMatrix9")
NOTES = ROOT / "docs" / "private-algorithm-notes.json"
REQUIRED = {"id", "stage", "om9_backend", "source", "verified_facts", "algorithm",
            "inputs_outputs", "boundaries"}

STAGES = [
    "0. Hợp đồng builder",
    "1. Vành và profile",
    "2. Thân và mặt",
    "3. Đặt đá theo curve",
    "4. Đặt đá theo surface",
    "5. Pave",
    "6. Halo / cluster / channel",
    "7. Prong / head / setting",
    "8. Cutter",
]


def load():
    return json.loads(NOTES.read_text(encoding="utf-8"))


def write(payload):
    NOTES.write_text(json.dumps(payload, ensure_ascii=False, indent=1) + "\n",
                     encoding="utf-8")


def cmd_list():
    payload = load()
    notes = payload["notes"]
    print(f"notes: {len(notes)}")
    covered = set()
    for n in notes:
        stage = n["stage"]
        # notes may cover several stages at once ("1/2", "6/7"), so match any stage
        # number mentioned rather than requiring a prefix match
        numbers = set(re.findall(r"\d+", stage.split(" ", 1)[0]))
        if not numbers:
            numbers = set(re.findall(r"\b(\d)\.", stage))
        for s in STAGES:
            if s.split(".")[0] in numbers:
                covered.add(s)
        print(f"  {n['id']:34s} {stage[:56]}")
    print()
    missing = [s for s in STAGES if s not in covered]
    print("stages without a note:", missing if missing else "(none)")
    return missing


def cmd_merge(path):
    incoming = json.loads(Path(path).read_text(encoding="utf-8"))
    if isinstance(incoming, dict):
        incoming = incoming.get("notes", [])
    payload = load()
    have = {n["id"] for n in payload["notes"]}
    added, replaced, problems = [], [], []
    for note in incoming:
        missing = REQUIRED - set(note)
        if missing:
            problems.append(f"{note.get('id', '?')}: missing fields {sorted(missing)}")
            continue
        if not note["source"].get("file"):
            problems.append(f"{note['id']}: source has no file")
            continue
        if note["id"] in have:
            payload["notes"] = [n for n in payload["notes"] if n["id"] != note["id"]]
            replaced.append(note["id"])
        else:
            added.append(note["id"])
        payload["notes"].append(note)
    write(payload)
    print(json.dumps({"added": added, "replaced": replaced, "problems": problems,
                      "total": len(payload["notes"])}, ensure_ascii=False, indent=1))
    if problems:
        sys.exit(1)


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--json", help="note objects to merge")
    ap.add_argument("--list", action="store_true")
    args = ap.parse_args()
    if args.list or not args.json:
        cmd_list()
    if args.json:
        cmd_merge(args.json)


if __name__ == "__main__":
    main()
