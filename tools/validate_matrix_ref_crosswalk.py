"""Validate the M00-M02 reference crosswalk before it is applied to L4 specs.

Checks:
- every L3 in M00/M01/M02 has an anchor row (no silent gaps);
- every referenced Matrix9 source id and RCORE chapter exists;
- every Matrix9 source path exists under ref/matrix (when the private export is
  present in this workspace);
- every RCORE chapter/capability/fixture id matches ref/rhino5/CORE_REFERENCE_INDEX.json.
"""
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import matrix_ref_crosswalk as cw  # noqa: E402
import apply_matrix_ref_supplements as apply_mod  # noqa: E402

ROOT = Path(__file__).resolve().parents[1]
INVENTORY = ROOT / "build" / "matrix-ref-inventory.json"
RHINO_INDEX = ROOT / "ref" / "rhino5" / "CORE_REFERENCE_INDEX.json"
REF_MATRIX = ROOT / "ref" / "matrix"


def main():
    errors = []
    inventory = json.loads(INVENTORY.read_text(encoding="utf-8"))
    l3_ids = {row["id"] for row in inventory["l3"]}
    missing = sorted(l3_ids - set(cw.L3_ANCHORS))
    extra = sorted(set(cw.L3_ANCHORS) - l3_ids)
    if missing:
        errors.append("L3 without crosswalk row: " + ", ".join(missing))
    if extra:
        errors.append("crosswalk row without L3: " + ", ".join(extra))

    for l3_id, row in cw.L3_ANCHORS.items():
        for chapter in row.get("rcore", []):
            if chapter not in cw.RCORE:
                errors.append(f"{l3_id}: unknown RCORE chapter {chapter}")
        for source_id in row.get("mx", {}):
            if source_id not in cw.MX and source_id not in cw.MANUAL:
                errors.append(f"{l3_id}: unknown Matrix9 source {source_id}")
        if not row.get("focus"):
            errors.append(f"{l3_id}: missing focus line")

    sheet_paths = apply_mod.SHEET_PATHS
    if not sheet_paths:
        errors.append("reference sheet index missing; run tools/matrix_ref_sources.py first")
    for source_id, sheet in cw.MANUAL.items():
        path = sheet_paths.get(sheet["sheet"])
        if path is None:
            errors.append(f"{source_id}: reference sheet not present: {sheet['sheet']}")
            continue
        if not (ROOT / path).is_file():
            errors.append(f"{source_id}: sheet locator does not resolve: {path}")

    for source_id, source in cw.MX.items():
        base = ROOT / apply_mod.source_root(source_id, source)
        path = base / source["path"]
        if not path.exists():
            errors.append(f"{source_id}: locator does not resolve: "
                          f"{base.relative_to(ROOT).as_posix()}/{source['path']}")
        if not source.get("state"):
            errors.append(f"{source_id}: missing verification state")
        if source_id not in apply_mod.MATRIX_SOURCE_IDS.values():
            entry = apply_mod.matrix_entry(source_id, "validation")
            if entry["verification_state"] not in {
                    "CONTENT_READ_IDENTIFIERS", "IDENTIFIER_INDEXED",
                    "PATH_INDEXED_SYMBOL_CANDIDATE", "RESOURCE_MANIFEST_READ"}:
                errors.append(f"{source_id}: unexpected verification state {entry['verification_state']}")

    if RHINO_INDEX.is_file():
        index = json.loads(RHINO_INDEX.read_text(encoding="utf-8"))
        chapters = {c["id"]: c for c in index["chapters"]}
        for chapter_id, chapter in cw.RCORE.items():
            actual = chapters.get(chapter_id)
            if actual is None:
                errors.append(f"unknown RCORE chapter in reference index: {chapter_id}")
                continue
            if chapter["title"] != actual["title"]:
                errors.append(f"{chapter_id}: title drift vs reference index")
            expected_caps = set(actual["capability_ids"])
            expected_fix = set(actual["fixture_ids"])
            for cap in chapter["capabilities"]:
                if cap not in expected_caps:
                    errors.append(f"{chapter_id}: capability not in index: {cap}")
            for fix in chapter["fixtures"]:
                if fix not in expected_fix:
                    errors.append(f"{chapter_id}: fixture not in index: {fix}")

    summary = {
        "l3_total": len(l3_ids),
        "crosswalk_rows": len(cw.L3_ANCHORS),
        "matrix_sources": len(cw.MX),
        "rcore_chapters": len(cw.RCORE),
        "errors": errors,
    }
    print(json.dumps(summary, ensure_ascii=False, indent=1))
    raise SystemExit(bool(errors))


if __name__ == "__main__":
    main()
