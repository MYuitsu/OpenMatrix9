"""Actual schema3 native producer -> owned Rust import -> cold .om9 batch.

This measures the selected current Rust import slice. It does not promote full
retained-source merge, clipboard, Rhino 5, or application acceptance.
"""
import datetime as dt
import hashlib
import json
from pathlib import Path
import re
import subprocess
import os

ROOT = Path(__file__).resolve().parents[1]
RECEIPT = ROOT / "docs/validation/2026-10-13-standalone-import.json"


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def main():
    native = ROOT / "build/standalone-native/Release/om9-exchange-native.exe"
    fixture = ROOT / "build/native-source-fixture/Release/om9-source-fixture.exe"
    scratch = ROOT / "build/standalone-validation"
    scratch.mkdir(parents=True, exist_ok=True)
    environment = os.environ.copy()
    environment.update(OM9_EXCHANGE_NATIVE=str(native.resolve(strict=True)),
                       OM9_SOURCE_FIXTURE=str(fixture.resolve(strict=True)),
                       OM9_NATIVE_IMPORT_SCRATCH=str(scratch.resolve(strict=True)))
    before = {str(p): digest(p) for p in [native, fixture]}
    command = ["rtk", "proxy", "cargo", "test", "--locked", "--manifest-path",
               "standalone/Cargo.toml", "-p", "om9-exchange", "--test",
               "retained_native_import", "--", "--ignored", "--nocapture"]
    run = subprocess.run(command, cwd=ROOT, env=environment, capture_output=True, timeout=120)
    output = run.stdout.decode("utf-8", errors="strict") + run.stderr.decode("utf-8", errors="replace")
    log = scratch / "native-import-tests.log"
    log.write_text(output, encoding="utf-8")
    assert run.returncode == 0, output
    result = re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;", output)
    assert result == [("8", "0", "0")], result
    names = re.findall(r"^test (real_\w+) \.\.\. ok$", output, flags=re.MULTILINE)
    assert len(names) == 8 and len(set(names)) == 8, names
    assert all(digest(Path(path)) == sha for path, sha in before.items()), "binary changed during batch"
    assert not list(scratch.glob("om9-source-import-*")), "test scratch cleanup incomplete"
    sources = [p for p in (ROOT / "standalone").rglob("*") if p.is_file()
               and "target" not in p.parts and p.suffix in {".rs", ".toml", ".lock"}]
    sources += [Path(__file__).resolve(), ROOT / "standalone/native/native_exchange.cpp",
                ROOT / "standalone/native/retained_merge.hpp",
                ROOT / "standalone/native/source_inventory_fixture.cpp", ROOT / "standalone/native/CMakeLists.txt"]
    receipt = {
        "schema_version": 1, "validated_at": dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds"),
        "status": "validated_native_to_host_import_slice", "goal_complete": False,
        "command": command, "rust_native_import_tests_passed": 8, "failed": 0,
        "test_names": names, "fixtures": 15,
        "fixture_count_definition": "Actual source files authored in parent tests; repeats and child reopen are separate executions, not new fixture kinds.",
        "fixture_modes": ["clean-mm", "clean-inch", "clean-custom", "clean-mixed", "clean-unitless", "clean-unset", "clean-unresolved", "mm", "clean-color-parent", "clean-color-material"],
        "source_schema": 3, "policy": "om9-source-projection-v1", "container_schema": 3,
        "binary_sha256": {Path(path).relative_to(ROOT).as_posix(): sha for path, sha in before.items()},
        "source_sha256": {p.relative_to(ROOT).as_posix(): digest(p) for p in sorted(set(sources))},
        "log": {"path": log.relative_to(ROOT).as_posix(), "sha256": digest(log)},
        "proven": ["Real native source bytes/digest retained with fresh document-local native namespaces; repeated import and one Undo/Redo.",
                   "Millimeter/inch/custom/explicit unknown units and nested/shared/mixed-unit definition scene coordinates.",
                   "Unresolved closure retained without hiding independently admissible roots; original/explicit definition units durable.",
                   "Preserve is default; explicit Geometry names omitted layer/member/object fields including unsupported color selectors.",
                   "Target closure, original operation cancellation, caller ownership and admission budgets preserve public state on rejection.",
                   "Fresh-process .om9 restore keeps actual native-source bytes and explicit definition contexts after source file removal."],
        "limitations": ["No retained merged writer or preservation acceptance for edits/new/delete to every native family.",
                        "Current unit/dependency/attribute admission is limited to represented exact geometry and default or directly represented attributes.",
                        "General document settings, arbitrary attachments/resources/history, clipboard, Rhino 5 actual runtime and app import dialog remain open.",
                        "Conservative local reservations do not certify a concurrent global memory pool or full performance targets."]
    }
    RECEIPT.write_text(json.dumps(receipt, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"status": receipt["status"], "tests_passed": 8, "fixtures": 15, "failed": 0, "goal_complete": False}))


if __name__ == "__main__":
    main()
