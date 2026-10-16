"""Freeze and measure durable preset/Hidden/schema policy, without Layers UI acceptance."""
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
    return {p for p in (ROOT / "standalone").rglob("*") if p.is_file()
            and "target" not in p.parts and p.suffix in {".rs", ".toml", ".lock", ".json", ".cpp", ".hpp"}} | {
                Path(__file__).resolve(), ROOT / "tools/validate_standalone_exchange.py",
                ROOT / "standalone/native/CMakeLists.txt", ROOT / "rust/src/layers.rs"}


def main():
    before = {p: digest(p) for p in source_paths()}
    scratch = ROOT / "build/standalone-validation"
    scratch.mkdir(parents=True, exist_ok=True)
    commands = [
        ["rtk", "proxy", "cargo", "fmt", "--manifest-path", "standalone/Cargo.toml", "--all", "--", "--check"],
        ["rtk", "proxy", "cargo", "clippy", "--manifest-path", "standalone/Cargo.toml", "--workspace", "--all-targets", "--locked", "--", "-D", "warnings"],
    ]
    tests = [
        ["-p", "om9-core", "--test", "layer_transactions"],
        ["-p", "om9-core", "--test", "layer_state"],
        ["-p", "om9-core", "--test", "retained_imports", "layers_assignment"],
        ["-p", "om9-archive", "--test", "layer_schema"],
        ["-p", "om9-scene", "--test", "layer_colors"],
        ["-p", "om9-exchange", "--test", "layer_wire"],
        ["-p", "om9-exchange", "--lib", "diagnostics_reserve"],
        ["-p", "om9-app", "--test", "cold_layers"],
        ["--workspace"],
    ]
    commands += [["rtk", "proxy", "cargo", "test", "--locked", "--manifest-path", "standalone/Cargo.toml", *args, "--", "--nocapture"] for args in tests]
    commands += [["rtk", "proxy", "python", "tools/validate_standalone_exchange.py"]]
    runs = []
    cold = []
    for n, command in enumerate(commands):
        result = subprocess.run(command, cwd=ROOT, capture_output=True, timeout=240)
        output = result.stdout.decode("utf-8", errors="strict") + result.stderr.decode("utf-8", errors="replace")
        log = scratch / f"owned-layer-state-{n}.log"
        log.write_text(output, encoding="utf-8")
        if result.returncode:
            raise RuntimeError(output)
        counts = re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;", output)
        assert all(int(f) == 0 for _, f, _ in counts), output
        run = {"command": command, "passed": sum(int(p) for p, _, _ in counts),
               "ignored": sum(int(i) for _, _, i in counts), "suites": len(counts),
               "log": {"path": log.relative_to(ROOT).as_posix(), "sha256": digest(log)}}
        runs.append(run)
        if n == 9:
            for line in output.splitlines():
                if "OM9_LAYER_RECEIPT " in line:
                    cold.append(json.loads(line.split("OM9_LAYER_RECEIPT ", 1)[1]))
        print(json.dumps({"step": n, "passed": run["passed"], "ignored": run["ignored"]}), flush=True)
    assert [r["passed"] for r in runs[2:10]] == [12, 7, 1, 6, 2, 2, 1, 3], runs
    assert len(cold) == 4 and {r["case"] for r in cold} == {
        "cold_layer_metadata", "cold_preset_hidden_metadata", "cold_schema3_cli_migration", "cold_schema4_cli_reopen"}, cold
    assert all(r["exited"] and r["active_processes"] == 0 and not r["full_layers_acceptance"] for r in cold)
    binary = Path(cold[0]["binary"]).resolve(strict=True)
    assert all(r["binary_sha256"] == digest(binary) for r in cold)
    native_path = ROOT / "docs/validation/2026-10-14-owned-layer-state-exchange.json"
    native = json.loads(native_path.read_text(encoding="utf-8"))
    assert native["rust_lifecycle_tests_passed"] == 8 and native["native_conversion_jobs"] == 18
    assert not native["goal_complete"]
    assert source_paths() == set(before) and all(digest(p) == sha for p, sha in before.items()), "source changed during checks"
    receipt = {"schema_version": 1, "status": "validated_owned_layer_state_schema4_policy_projection_slice",
               "measured_at": dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds"),
               "goal_complete": False, "full_layers_acceptance": False, "archive_schema": 4,
               "runs": runs, "workspace_passed": runs[10]["passed"], "workspace_ignored": runs[10]["ignored"],
               "focused_tests": sum(r["passed"] for r in runs[2:10]), "focused_checks_are_workspace_subset": True,
               "cold_fixtures": cold, "cold_om9_processes": 4,
               "binary": {"path": binary.relative_to(ROOT).as_posix(), "sha256": digest(binary)},
               "native_export_receipt": {"path": native_path.relative_to(ROOT).as_posix(), "sha256": digest(native_path), "tests": 8, "jobs": 18},
               "source_sha256": {p.relative_to(ROOT).as_posix(): sha for p, sha in sorted(before.items())},
               "limitations": [
                   "Actual Layers panel/CMD/current-output/selection/modal/Advanced/Lights gestures remain open.",
                   "Cold native Hidden/preset loss fixture covers a root Point, not complete native family/definition-member/locked-hidden combinations.",
                   "Other ignored SDK batches and older native UI/Rhino receipts retain historical source checkpoints.",
                   "No full feature/capability, large-scene latency, cross-job global memory or full RCORE acceptance claimed."]}
    path = ROOT / "docs/validation/2026-10-14-owned-layer-state.json"
    path.write_text(json.dumps(receipt, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"status": receipt["status"], "workspace_passed": receipt["workspace_passed"],
                      "workspace_ignored": receipt["workspace_ignored"], "focused_subset": receipt["focused_tests"],
                      "cold_om9_processes": 4, "native_jobs": 18, "goal_complete": False}))


if __name__ == "__main__":
    main()
