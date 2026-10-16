"""Record actual Rust schema/resource/cold-process evidence for a partial slice.

Fixtures and assertions run in Rust, including independent executable opens;
Python only invokes the tests and fingerprints their emitted receipts.
"""
import datetime as dt
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def digest(path):
    with path.open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def run(arguments, expected):
    command = ["rtk", "proxy", "cargo", "test", "--locked", "--manifest-path",
               "standalone/Cargo.toml", *arguments, "--", "--test-threads=1", "--nocapture"]
    result = subprocess.run(command, cwd=ROOT, capture_output=True, timeout=120)
    output = result.stdout.decode("utf-8", errors="strict")
    if result.returncode:
        raise RuntimeError(output + result.stderr.decode("utf-8", errors="replace"))
    assert f"{expected} passed; 0 failed" in output, output
    return command, output


def main():
    commands = []
    command, output = run(["-p", "om9-app", "--test", "cold_reopen"], 9)
    commands.append(command)
    rows = [json.loads(line.split("OM9_RESOURCE_RECEIPT ", 1)[1])
            for line in output.splitlines() if "OM9_RESOURCE_RECEIPT " in line]
    expected = {"embedded_resource_cold_reopen", "explicit_schema1_migration",
                "migration_counter_overflow_preserves_file", "persistent_parameter_fingerprint",
                "explicit_schema2_migration", "schema3_namespace_tombstone_cold_reopen"}
    assert len(rows) == 6 and {row["case"] for row in rows} == expected, rows
    binaries = {Path(row["binary"]).resolve(strict=True) for row in rows}
    assert len(binaries) == 1
    binary = binaries.pop()
    for row in rows:
        assert row["details"]["exited"]
        pids = row["details"].get("pids", [row["details"].get("pid")])
        assert all(isinstance(pid, int) and pid > 0 for pid in pids), row
        row["binary"] = binary.relative_to(ROOT).as_posix()
    by_case = {row["case"]: row["details"] for row in rows}
    embedded = by_case["embedded_resource_cold_reopen"]
    assert embedded["original_removed"] and embedded["resource"]["bytes"] == 32768
    assert len(embedded["resource"]["payload_sha256"]) == 64
    migration = by_case["explicit_schema1_migration"]
    assert migration["from_schema"] == 1 and migration["to_schema"] == 3
    assert migration["default_open_rejected"] and migration["resources_empty"]
    assert migration["restoration"] == "unresolved" and migration["diagnostics"]
    assert migration["geometry"] == {"Point": [1.0, 2.0, 3.0]}
    overflow = by_case["migration_counter_overflow_preserves_file"]
    assert overflow["original_preserved"] and overflow["stdout_empty"]
    assert not overflow["exit_success"] and overflow["directory_entries"] == 1
    hashes = by_case["persistent_parameter_fingerprint"]["persistent_sha256"]
    assert len(hashes) == 2 and hashes[0] != hashes[1] and all(len(h) == 64 for h in hashes)
    migration2=by_case["explicit_schema2_migration"]
    assert migration2["from_schema"] == 2 and migration2["to_schema"] == 3
    assert migration2["empty_provenance"] and migration2["default_open_rejected"]
    ledger=by_case["schema3_namespace_tombstone_cold_reopen"]
    assert ledger["dispositions"] == ["Edited","Deleted"] and len(ledger["objects"]) == 2
    assert ledger["original_model_mm_per_unit"] == 25.4
    tests = 9
    for package, suite, count in [("om9-archive", "resource_chunks", 8),
                                  ("om9-archive", "restore_modules", 6),
                                  ("om9-core", "embedded_resources", 3),
                                  ("om9-core", "resource_transactions", 12),
                                  ("om9-core", "retained_imports", 16),
                                  ("om9-core", "retained_peak_review", 3),
                                  ("om9-exchange", "source_preparation_review", 4)]:
        command, _ = run(["-p", package, "--test", suite], count)
        commands.append(command)
        tests += count
    command,_=run(["-p","om9-app","--lib","report::tests"],1)
    commands.append(command)
    tests+=1
    sources = [p for p in (ROOT / "standalone").rglob("*") if p.is_file()
               and "target" not in p.parts and p.suffix in {".rs", ".toml", ".lock", ".json"}]
    sources.append(Path(__file__).resolve())
    receipt = {
        "schema_version": 1, "validated_at": dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds"),
        "status": "validated_resource_schema3_source_ledger_slice", "goal_complete": False,
        "commands": commands, "rust_tests_passed": tests, "cold_process_cases": rows,
        "binary": {"path": binary.relative_to(ROOT).as_posix(), "sha256": digest(binary)},
        "source_sha256": {p.relative_to(ROOT).as_posix(): digest(p) for p in sorted(set(sources))},
        "limitations": ["Owned source/namespace/tombstone metadata is not native import correspondence or merged preservation export.",
                        "Native tables/userdata payload preservation and Rhino interoperability acceptance remain pending.",
                        "No full archive fixture acceptance, global reservations, allocation deduplication or OS clipboard acceptance."]
    }
    path = ROOT / "docs/validation/2026-10-13-standalone-resources.json"
    path.write_text(json.dumps(receipt, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"status": receipt["status"], "tests": tests, "cold_cases": len(rows),
                      "binary_sha256": receipt["binary"]["sha256"], "completion_claim": False}))


if __name__ == "__main__":
    main()
