"""Run actual Rust-owned current-state 3DM conversion/publication tests.

Native geometry proof comes from cold openNURBS inventory, compared against
typed immutable OM9 masters. This receipt does not accept retained-source merge,
full import/export UI, structural clipboard or Rhino 5 actual open/edit.
"""
import argparse
import datetime as dt
import hashlib
import json
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def digest(path):
    with path.open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, default=ROOT / "build/standalone-native/Release/om9-exchange-native.exe")
    parser.add_argument("--receipt", type=Path, default=ROOT / "docs/validation/2026-10-14-owned-layer-state-exchange.json")
    args = parser.parse_args()
    binary = args.binary.resolve(strict=True)
    before = digest(binary)
    def source_paths():
        return {p for p in (ROOT / "standalone").rglob("*") if p.is_file()
                and "target" not in p.parts and p.suffix in {".rs", ".toml", ".lock", ".json", ".cpp", ".hpp"}} | {ROOT / "standalone/native/CMakeLists.txt", Path(__file__).resolve()}
    source_before = {p:digest(p) for p in source_paths()}
    env = os.environ.copy()
    env["OM9_EXCHANGE_NATIVE"] = str(binary)
    command = ["rtk", "proxy", "cargo", "test", "--manifest-path", "standalone/Cargo.toml",
               "-p", "om9-exchange", "--test", "staged_export", "--", "--ignored", "--test-threads=1", "--nocapture"]
    result = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, timeout=120)
    output = result.stdout.decode("utf-8", errors="strict")
    if result.returncode:
        raise RuntimeError(output + result.stderr.decode("utf-8", errors="replace"))
    assert "8 passed; 0 failed" in output, output
    assert digest(binary) == before, "native executable changed during validation"
    rows = [json.loads(line.split("OM9_EXCHANGE_RECEIPT ", 1)[1])
            for line in output.splitlines() if "OM9_EXCHANGE_RECEIPT " in line]
    expected = {"accepted_current_point", "stale_commit_undo_rejected", "late_cancel_rejected",
                "readonly_rejected", "accepted_all_masters", "accepted_current_edit_delete_new",
                "closed_owner_rejected", "changed_staging_rejected", "hidden_preset_loss_projection"}
    assert len(rows) == len(expected) and {r["case"] for r in rows} == expected, rows
    for row in rows:
        assert len(row["sha256"]) == 64 and row["bytes"] > 0
        for operation in ("write", "inspect"):
            stats = row[operation]
            assert stats["pid"] > 0 and stats["exited"] and stats["active_processes"] == 0, row
            assert 0 < stats["peak_committed_bytes"] <= 128 * 1024 * 1024, row
    hidden = next(row for row in rows if row["case"] == "hidden_preset_loss_projection")
    assert hidden["dropped_om9_hidden_distinctions"] == 1 and hidden["dropped_om9_preset_slots"] == 1, hidden
    assert source_paths() == set(source_before) and all(digest(p) == sha for p,sha in source_before.items()), "source changed during validation"
    receipt = {
        "schema_version": 1, "validated_at": dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds"),
        "status": "validated_owned_current_geometry_export_slice", "goal_complete": False,
        "command": command, "rust_lifecycle_tests_passed": 8, "native_conversion_jobs": 18,
        "native_binary": binary.relative_to(ROOT).as_posix() if binary.is_relative_to(ROOT) else str(binary),
        "native_sha256": before, "process_native_commit_cap_bytes": 128 * 1024 * 1024,
        "process_stdout_cap_bytes": 16 * 1024 * 1024, "process_stderr_cap_bytes": 16 * 1024,
        "process_deadline_seconds": 30, "native_file_limit_bytes": 16 * 1024 * 1024,
        "peak_committed_bytes": max(r[op]["peak_committed_bytes"] for r in rows for op in ("write", "inspect")),
        "source_sha256": {p.relative_to(ROOT).as_posix(): sha for p,sha in sorted(source_before.items())},
        "cases": rows,
        "limitations": ["Geometry-only conversion folds OM9 Hidden into native visibility and omits preset identities, History/results/runtime/Undo/document metadata with diagnostics and explicit receipt counts.",
                        "No retained-source merge/import namespace/clipboard/full archive migration acceptance.",
                        "No Matrix9 import/export dialog, application native capability or Rhino 5 actual open/edit accepted.",
                        "No OS security sandbox, physical RAM or native memory-safety certification; cross-job global reservation remains open."]
    }
    args.receipt.parent.mkdir(parents=True, exist_ok=True)
    args.receipt.write_text(json.dumps(receipt, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"status": receipt["status"], "rust_tests": 8, "native_jobs": 18,
                      "peak_committed_bytes": receipt["peak_committed_bytes"], "completion_claim": False}))


if __name__ == "__main__":
    main()
