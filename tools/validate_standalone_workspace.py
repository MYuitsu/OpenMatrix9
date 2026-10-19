"""Record actual workspace test counts and their source fingerprints.

Explicitly ignored native exchange tests run through their separate validator.
This evidence does not promote any complete RCORE or application capability.
"""
import datetime as dt
import hashlib
import json
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def digest(path):
    with path.open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def source_paths():
    paths = {p for p in (ROOT / "standalone").rglob("*") if p.is_file()
             and "target" not in p.parts and p.suffix in {".rs", ".toml", ".lock", ".json"}}
    paths.add(Path(__file__).resolve())
    return paths


def main():
    source_before = {p: digest(p) for p in sorted(source_paths())}
    command = ["rtk", "proxy", "cargo", "test", "--locked", "--manifest-path",
               "Cargo.toml", "--workspace"]
    result = subprocess.run(command, cwd=ROOT, capture_output=True, timeout=180)
    output = result.stdout.decode("utf-8", errors="strict")
    errors = result.stderr.decode("utf-8", errors="replace")
    log = ROOT / "build/standalone-validation/workspace-tests.log"
    log.parent.mkdir(parents=True, exist_ok=True)
    log.write_text(output + errors, encoding="utf-8")
    if result.returncode:
        raise RuntimeError(output + errors)
    counts = re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;", output)
    assert counts and all(int(failed) == 0 for _, failed, _ in counts), output
    assert source_paths() == set(source_before), "workspace source file universe changed during tests"
    assert all(digest(p) == sha for p, sha in source_before.items()), "workspace source changed during tests"
    receipt = {
        "schema_version": 1, "validated_at": dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds"),
        "status": "validated_workspace_tests", "goal_complete": False,
        "command": command, "suites": len(counts),
        "passed": sum(int(p) for p, _, _ in counts),
        "ignored": sum(int(i) for _, _, i in counts),
        "rustc": subprocess.check_output(["rtk", "proxy", "rustc", "--version"], cwd=ROOT, text=True).strip(),
        "cargo": subprocess.check_output(["rtk", "proxy", "cargo", "--version"], cwd=ROOT, text=True).strip(),
        "log": {"path": log.relative_to(ROOT).as_posix(), "sha256": digest(log)},
        "source_sha256": {p.relative_to(ROOT).as_posix(): sha for p, sha in source_before.items()},
        "limitations": ["Ignored native exchange lifecycle tests require separate actual native validation.",
                        "Tests cover implementation slices; complete M01–M12 and Matrix9 acceptance remain open."]
    }
    path = ROOT / "docs/validation/2026-10-13-standalone-workspace.json"
    path.write_text(json.dumps(receipt, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"status": receipt["status"], "passed": receipt["passed"],
                      "ignored": receipt["ignored"], "suites": receipt["suites"], "completion_claim": False}))


if __name__ == "__main__":
    main()
