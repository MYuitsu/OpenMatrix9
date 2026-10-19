"""Measured Rust source -> retained current 3DM -> cold proof -> publication.

This is a selected SDK/codec integration batch, not full RCORE-00 acceptance.
Sources, outputs, logs, staging and scratch stay on H:. Source/binary snapshots
must remain unchanged for the entire run before a public receipt is recorded.
"""
import datetime as dt
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def source_paths():
    root = 
    paths = {p for p in root.rglob("*") if p.is_file() and "target" not in p.parts
             and p.suffix in {".rs", ".toml", ".lock", ".cpp", ".hpp", ".h"}}
    paths.add(root / "native/CMakeLists.txt")
    paths.add(Path(__file__).resolve())
    return paths


def main():
    binaries = {
        "OM9_EXCHANGE_NATIVE": ROOT / "build/standalone-native/Release/om9-exchange-native.exe",
        "OM9_SOURCE_FIXTURE": ROOT / "build/native-source-fixture/Release/om9-source-fixture.exe",
    }
    scratch = ROOT / "build/standalone-validation"
    scratch.mkdir(parents=True, exist_ok=True)
    environment = os.environ.copy()
    environment.update({key: str(path.resolve(strict=True)) for key, path in binaries.items()})
    environment["OM9_NATIVE_IMPORT_SCRATCH"] = str(scratch.resolve(strict=True))
    before = {p: digest(p) for p in binaries.values()}
    source_before = {p: digest(p) for p in sorted(source_paths())}
    command = ["rtk", "proxy", "cargo", "test", "--locked", "--manifest-path", "Cargo.toml",
               "-p", "om9-exchange", "--test", "retained_native_write", "--", "--ignored", "--nocapture", "--test-threads=1"]
    run = subprocess.run(command, cwd=ROOT, env=environment, capture_output=True, timeout=120)
    output = run.stdout.decode("utf-8", errors="strict") + run.stderr.decode("utf-8", errors="replace")
    log = scratch / "retained-current-tests.log"
    log.write_text(output, encoding="utf-8")
    assert run.returncode == 0, output
    assert re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;", output) == [("4", "0", "0")]
    jobs = []
    for kind, row in re.findall(r"OM9_RETAINED_(WRITE|COLD|SOURCE)_JOB (\{[^\r\n]+\})", output):
        job = json.loads(row)
        job["stage"] = kind.lower()
        jobs.append(job)
    source_modes = re.findall(r"OM9_RETAINED_CURRENT_SOURCE ([^\r\n]+)", output)
    edit_modes = re.findall(r"OM9_RETAINED_CURRENT_EDIT ([^\r\n]+)", output)
    publication_cases = re.findall(r"OM9_RETAINED_PUBLICATION_CASE ([^\r\n]+)", output)
    assert len(jobs) == 78 and len(source_modes) == 7 and len(edit_modes) == 5 and len(publication_cases) == 8
    assert len(set(source_modes)) == 7 and len(set(edit_modes)) == 5 and len(set(publication_cases)) == 8
    assert all(j["exited"] and j["active_processes"] == 0 and
               0 < j["peak_committed_bytes"] <= 128 * 1024 * 1024 for j in jobs)
    assert all(digest(p) == sha for p, sha in before.items()), "native binary changed during batch"
    assert source_paths() == set(source_before), "source file universe changed during batch"
    assert all(digest(p) == sha for p, sha in source_before.items()), "source bytes changed during batch"
    assert not list(scratch.glob("retained-write-test-*")), "retained current staging cleanup incomplete"
    receipt = {
        "schema_version": 1, "validated_at": dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds"),
        "status": "validated_selected_retained_current_cold_publication", "goal_complete": False,
        "command": command, "rust_tests_passed": 4, "failed": 0, "fixtures": 33,
        "fixture_count_definition": "Twelve SDK-authored native sources and twenty-one private writer outputs; thirteen outputs are atomically renamed to destinations. Twelve immutable embedded source staging copies are not new semantic fixtures.",
        "authored_source_files": 12, "successful_private_output_files": 21,
        "published_output_files": 13, "embedded_source_stage_copies": 12, "semantic_scenarios": 21,
        "source_scenarios": source_modes, "current_edit_scenarios": edit_modes, "publication_cases": publication_cases,
        "expected_publication_refusals": 7, "native_runs": len(jobs), "native_preparations": 12,
        "native_writes": 21, "native_cold_inspections": 21,
        "independently_checked_dirty_represented_bounds": 12,
        "peak_committed_bytes": max(j["peak_committed_bytes"] for j in jobs), "runs": jobs,
        "source_schema": 4, "writer_policy": "om9-retained-current-v1", "bounds_policy": "cold_leaf_hull_v1",
        "binary_sha256": {p.relative_to(ROOT).as_posix(): sha for p, sha in before.items()},
        "source_sha256": {p.relative_to(ROOT).as_posix(): sha for p, sha in source_before.items()},
        "log": {"path": log.relative_to(ROOT).as_posix(), "sha256": digest(log)},
        "proven": [
            "Rust stages complete original bytes and mapped current requests under read-only deny-write/delete ownership after original external imports are removed.",
            "The writer and fresh inspector share the admitted executable hash, and all native jobs are reaped within their process limit.",
            "Independent Rust verification streams the same held output hashes/raw spans and compares current UUIDs, three wire contexts, attributes, layer hierarchy, definition members, local frames, units and current-layer intent.",
            "Original mm/inch/custom/mixed/None/Unset declarations and unselected native tables/opaque original rows retain their admitted source correspondence.",
            "Edited nested master members in mm and mixed definition units refresh the leaf, both ancestor definitions and both root instance caches; new definition/instance caches are checked independently from captured represented geometry.",
            "Current root deletion, new Definition/Layer/member/instance and supported root label/color/locked attribute changes survive cold validation without reviving old source rows.",
            "The sealed held output changes to a final verified OS handle for same-parent atomic replacement; exact loss acknowledgement precedes publication.",
            "Absent/wrong loss acknowledgement, read-only destination, original/write cancellation, owner close and aggregate budget refuse while preserving the existing destination and immediate coordinator snapshot.",
        ],
        "limitations": [
            "Selected native codecs and conservative cold leaf hulls only; no independent tight BRep bound certificate.",
            "Current member/layer edit source scenarios bootstrap owned test snapshots after actual import; separate core transaction suites cover publication/Undo semantics.",
            "OM9 runtime ownership, History/results/resource containers require the explicit loss report; this is not native OM9 persistence or full 3DM capability acceptance.",
            "General opaque remapping, full native families, clipboard, global memory pool, heavy GPU performance and independent Rhino5 acceptance remain open; M01-M12 remain unchecked.",
        ],
    }
    path = ROOT / "docs/validation/2026-10-14-standalone-retained-current.json"
    path.write_text(json.dumps(receipt, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"status": receipt["status"], "tests_passed": 4, "authored_source_files": 12,
                      "successful_output_files": 21, "native_runs": 78,
                      "publication_refusals": 7, "failed": 0, "goal_complete": False}))


if __name__ == "__main__":
    main()
