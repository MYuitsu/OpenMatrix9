"""Measured original-wire native preparation -> strict Rust current-state inputs.

This batch never writes or publishes a merged 3DM. Native executions use the
Rust process owner; authored sources, temporary files and logs stay on H:.
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


def main():
    binaries = {
        "OM9_EXCHANGE_NATIVE": ROOT / "build/standalone-native/Release/om9-exchange-native.exe",
        "OM9_SOURCE_FIXTURE": ROOT / "build/native-source-fixture/Release/om9-source-fixture.exe",
        "OM9_RETAINED_SOURCE_FIXTURE": ROOT / "build/native-retained-source-fixture/Release/om9-retained-source-fixture.exe",
    }
    scratch = ROOT / "build/standalone-validation"
    scratch.mkdir(parents=True, exist_ok=True)
    environment = os.environ.copy()
    environment.update({key: str(path.resolve(strict=True)) for key, path in binaries.items()})
    environment["OM9_NATIVE_IMPORT_SCRATCH"] = str(scratch.resolve(strict=True))
    before = {path: digest(path) for path in binaries.values()}
    sources = [p for p in (ROOT / "standalone").rglob("*") if p.is_file()
               and "target" not in p.parts and p.suffix in {".rs", ".toml", ".lock"}]
    sources += [ROOT / "standalone/native" / name for name in
                ["native_exchange.cpp", "retained_merge.hpp", "source_inventory_fixture.cpp", "retained_source_fixture.cpp", "CMakeLists.txt"]]
    sources += [Path(__file__).resolve()]
    source_before = {p: digest(p) for p in sorted(set(sources))}
    command = ["rtk", "proxy", "cargo", "test", "--locked", "--manifest-path", "standalone/Cargo.toml",
               "-p", "om9-exchange", "--test", "retained_native_inputs", "--", "--ignored", "--nocapture", "--test-threads=1"]
    run = subprocess.run(command, cwd=ROOT, env=environment, capture_output=True, timeout=120)
    output = run.stdout.decode("utf-8", errors="strict") + run.stderr.decode("utf-8", errors="replace")
    log = scratch / "retained-input-tests.log"
    log.write_text(output, encoding="utf-8")
    assert run.returncode == 0, output
    assert re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;", output) == [("4", "0", "0")]
    jobs = [json.loads(row) for row in re.findall(r"OM9_RETAINED_INPUT_JOB (\{[^\r\n]+\})", output)]
    modes = re.findall(r"OM9_RETAINED_INPUT_SOURCE ([^\r\n]+)", output)
    assert len(jobs) == 36 and len(modes) == 11 and len(set(modes)) == 11, (len(jobs), modes)
    assert all(job["stats"]["exited"] and job["stats"]["active_processes"] == 0
               and 0 < job["stats"]["peak_committed_bytes"] <= 128 * 1024 * 1024 for job in jobs)
    assert all(digest(path) == sha for path, sha in before.items()), "native binary changed during batch"
    assert all(digest(path) == sha for path, sha in source_before.items()), "source changed during batch"
    assert not list(scratch.glob("om9-retained-inputs-*")), "retained preparation staging cleanup incomplete"
    receipt = {
        "schema_version": 1, "validated_at": dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds"),
        "status": "validated_original_wire_current_inputs_slice", "goal_complete": False,
        "command": command, "rust_tests_passed": 4, "failed": 0, "fixtures": 15,
        "fixture_count_definition": "Six unit files, three base files plus three derived sparse/light/copy-count-zero files, one ownership file, and an additional base/sparse pair for new wire rows; eleven source scenarios.",
        "source_scenarios": modes, "native_runs": len(jobs), "native_preparations": 10,
        "expected_prelaunch_rejections": 4, "peak_committed_bytes": max(j["stats"]["peak_committed_bytes"] for j in jobs),
        "runs": jobs, "source_schema": 4, "decoder_policy": "om9-retained-source-v1",
        "binary_sha256": {p.relative_to(ROOT).as_posix(): sha for p, sha in before.items()},
        "source_sha256": {p.relative_to(ROOT).as_posix(): sha for p, sha in source_before.items()},
        "log": {"path": log.relative_to(ROOT).as_posix(), "sha256": digest(log)},
        "proven": [
            "Actual owned original bytes and model/page/definition declarations correspond to the persisted Rust import namespace.",
            "Original record and full table/start/end spans match their SHA256 and complete decoded source keys.",
            "Sparse Layer wire7/reference1/decoded1, Group wire5/reference5/decoded0, and Light wire7/reference0/no decoded index remain distinct.",
            "Copy-count-zero legacy style attachment remains visible on the original wire owner after decoded conversion drops it.",
            "Primary wire contexts remain original; fresh Layer serial8/reference2 differs from decoded intent2, generated unassigned Layer serial9/reference3, and fresh Definition has no serialized index with ordinal4 after all four source definitions, including the unbound orphan.",
            "New root/instance/local member UUIDs occur once in the planned geometry table with no geometry index; generated native layer does not fabricate a current host layer.",
            "Borrowed writer requests forward all original wire hashes/contexts and resolved model/page/definition factors, including null current payloads for retained-only/orphan rows.",
            "Embedded source bytes create namespace-named held staging inputs after original external imports are deleted; packet/source writes and deletion are denied, held hashes match, and scope cleanup removes private inputs.",
            "Current output intents remain frozen to the same coordinator; cancel, budgets, Undo and foreign owner reject before preparation; staging is cleaned and target unchanged.",
        ],
        "limitations": [
            "Output wire allocation is a closed intent, not an actual native file. No retained merge writer or final cold output geometry/bounds proof.",
            "False preservation flags do not certify arbitrary identity, reference or length remapping.",
            "Only original V5 version50 wire preparation is admitted; full native families, resources, clipboard, global memory pool and Rhino5 application acceptance remain open.",
        ],
    }
    path = ROOT / "docs/validation/2026-10-14-standalone-retained-inputs.json"
    path.write_text(json.dumps(receipt, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"status": receipt["status"], "tests_passed": 4, "fixtures": 15,
                      "source_scenarios": 11, "native_runs": 36, "failed": 0, "goal_complete": False}))


if __name__ == "__main__":
    main()
