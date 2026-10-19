"""Record frozen selected native measurements without promoting the full goal."""
import datetime as dt
import hashlib
import json
from pathlib import Path
import shutil

ROOT = Path(__file__).resolve().parents[1]
# Counts are declared semantic scenarios/files, not inferred from execution totals.
BATCHES = [
    ("selected", 107, 34, 8, 23, 7, 57, "native-retained-current-validation.json"),
    ("conversion", 50, 40, 0, 5, 0, 45, "native-conversion-retained-current-compatibility.json"),
    ("inventory", 68, 17, 18, 6, 0, 28, "native-inventory-retained-current-compatibility.json"),
    ("projection", 82, 32, 22, 0, 0, 49, "native-projection-retained-current-compatibility.json"),
    ("preparation", 65, 20, 28, 0, 0, 32, "native-preparation-retained-current-compatibility.json"),
]


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def main():
    binary = digest(ROOT / "build/standalone-native/Release/om9-exchange-native.exe")
    cpp = digest(ROOT / "native/native_exchange.cpp")
    header = digest(ROOT / "native/retained_merge.hpp")
    selected = json.loads((ROOT / "build/standalone-native" / BATCHES[0][-1]).read_text())
    assert (selected["binary_sha256"], selected["source_sha256"], selected["header_sha256"]) == (binary, cpp, header)
    stamp = dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds")
    rows = []
    for name, runs, refused, inputs, outputs, negatives, scenarios, filename in BATCHES:
        source = ROOT / "build/standalone-native" / filename
        measured = json.loads(source.read_text())
        assert measured["binary_sha256"] == binary
        if name == "selected":
            assert (measured["total_runs"], measured["expected_rejections"]) == (runs, refused)
            assert (measured["sdk_authored_source_files"], measured["successful_writer_output_files"],
                    measured["authored_valid_stale_cache_output_files"], measured["semantic_scenarios"]) == (inputs, outputs, negatives, scenarios)
        elif name == "inventory" or name == "projection":
            assert len(measured["operation_receipts"]) == runs and measured["rejections"] == refused
        elif name == "preparation":
            assert len(measured["runs_detail"]) == runs and measured["rejections"] == refused
        else:
            assert len(measured["runs"]) == runs and len(measured["rejection_cases"]) == refused
        for key, sha in [("source_sha256", cpp), ("header_sha256", header)]:
            if key in measured:
                assert measured[key] == sha, (name, key)
        for row in measured.get("authored_sources", []):
            assert row["sha256"] == digest(ROOT / row["path"]), (name, row["path"])
        target = ROOT / "docs/validation" / f"2026-10-14-native-retained-current-{name}.json"
        shutil.copyfile(source, target)
        rows.append({
            "name": f"native-retained-current-{name}-2026-10-14",
            "status": "passed_supported_slice", "recorded_at": stamp,
            "native_runs": runs, "expected_rejections": refused,
            "authored_input_files": inputs, "successful_output_files": outputs,
            "authored_negative_output_files": negatives, "semantic_scenarios": scenarios,
            "binary_sha256": binary, "source_sha256": cpp, "header_sha256": header,
            "evidence": target.relative_to(ROOT).as_posix(), "evidence_sha256": digest(target),
            "goal_complete": False,
        })
    roadmap = ROOT / "docs/validation/opennurbs-test-roadmap.json"
    data = json.loads(roadmap.read_text(encoding="utf-8"))
    names = {row["name"] for row in rows}
    total_runs = sum(row["native_runs"] for row in rows)
    total_refusals = sum(row["expected_rejections"] for row in rows)
    data["completed_batches"] = [row for row in data["completed_batches"] if row.get("name") not in names] + rows
    data["updated_at"] = stamp
    data["retained_current_checkpoint"] = {
        "native_runs": total_runs, "expected_rejections": total_refusals, "batches": rows,
        "limits": "Selected strict SDK codecs. Rust independent cold/capture acceptance and publication, general opaque remap, full families and Rhino acceptance remain open.",
        "goal_complete": False,
    }
    roadmap.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    ledger = ROOT / "docs/validation/2026-10-13-standalone-foundation-progress.md"
    text = ledger.read_text(encoding="utf-8")
    marker = f"### Native retained-current adapter {binary[:7]} — 2026-10-14"
    if marker not in text:
        text += f"\n\n{marker}\n\n" + (
            f"Frozen adapter binary {binary[:7]}: selected 107/34 expected refusals (8 SDK source files, 23 successful outputs, 7 authored cache-negative files, 57 scenarios); "
            "conversion 50/40 (5 output files, 45 scenarios); inventory 68/17 (18 inputs, 6 outputs, 28 scenarios); "
            "projection 82/32 (22 inputs, 49 scenarios); preparation 65/20 (28 inputs, 32 scenarios). "
            f"Total {total_runs} owned jobs / {total_refusals} expected refusals; all children reaped. Files/scenarios/executions are separate counts. "
            "The selected batch grew by two successful files and two valid stale-cache negative files after actual tiny-coordinate/huge-instance and translated thin-hull regressions exposed the absolute tolerance floor.\n\n"
            "Selected writer/cold SDK tests cover A edit/B deletion/C addition, appended layers/definitions/members/mirrored instances, shared/nested cache arithmetic, "
            "typed secondary mm/inch regeneration, raw userdata/copy-count-zero chunks, decimal V5 mesh and smaller exact EOF after deletion. "
            "Cold raw_spans expose actual held output hashes/ranges for independent Rust checks. Receipts: 2026-10-14-native-retained-current-{selected,conversion,inventory,projection,preparation}.json. "
            "General opaque semantics, tight CAD bounds and Rhino application acceptance are unproved. Root Rust independent cold/current capture acceptance and atomic publication remain in progress. "
            "Full RCORE-00 and packages 2–8 remain open; total future batches unknown.\n"
        )
        ledger.write_text(text, encoding="utf-8")
    print(json.dumps({"batches": 5, "native_runs": total_runs, "expected_rejections": total_refusals, "goal_complete": False}))


if __name__ == "__main__":
    main()
