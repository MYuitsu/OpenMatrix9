"""Source-bound Layers controller/widgets/scene admission checkpoint, not full acceptance."""
import datetime as dt
import hashlib
import json
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]
RECEIPT = ROOT / "docs/validation/2026-10-14-owned-layers-ui.json"


def digest(path):
    with path.open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def source_paths():
    return {p for p in (ROOT / "standalone").rglob("*") if p.is_file()
            and "target" not in p.parts
            and p.suffix in {".rs", ".toml", ".lock", ".json", ".cpp", ".hpp"}} | {
                Path(__file__).resolve(), ROOT / "tools/validate_standalone_ui.py",
                ROOT / "tools/validate_standalone_foundation.py",
                ROOT / "native/CMakeLists.txt", ROOT / "rust/src/layers.rs"}


def main():
    before = {p: digest(p) for p in source_paths()}
    scratch = ROOT / "build/standalone-validation"
    scratch.mkdir(parents=True, exist_ok=True)
    commands = [
        ["rtk", "proxy", "cargo", "fmt", "--manifest-path", "Cargo.toml", "--all", "--", "--check"],
        ["rtk", "proxy", "cargo", "clippy", "--manifest-path", "Cargo.toml", "--workspace", "--all-targets", "--locked", "--", "-D", "warnings"],
    ]
    focused = [
        ["-p", "om9-core", "--test", "lazy_layer_assignment"],
        ["-p", "om9-app", "--test", "layer_controller"],
        ["-p", "om9-app", "--test", "layer_widgets"],
        ["-p", "om9-scene", "--test", "metadata_cache"],
        ["-p", "om9-scene", "--test", "instance_admission"],
        ["-p", "om9-app", "--test", "scene_allocation_peak"],
        ["-p", "om9-app", "--test", "viewport_title_menu"],
    ]
    commands += [["rtk", "proxy", "cargo", "test", "--locked", "--manifest-path", "Cargo.toml", *args, "--", "--nocapture"]
                 for args in focused + [["--workspace"]]]
    commands += [
        ["rtk", "proxy", "cargo", "build", "--release", "--locked", "--manifest-path", "Cargo.toml", "-p", "om9-app", "--bin", "om9"],
        ["rtk", "proxy", "python", "tools/validate_standalone_ui.py", "--layers", "--output", "docs/validation/2026-10-14-owned-layers-ui"],
        ["rtk", "proxy", "python", "tools/validate_standalone_ui.py", "--viewport-menu", "--output", "docs/validation/2026-10-14-owned-layers-viewport-regression"],
    ]
    runs = []
    cold = []
    peaks = []
    for index, command in enumerate(commands):
        result = subprocess.run(command, cwd=ROOT, capture_output=True, timeout=300)
        output = result.stdout.decode("utf-8", errors="strict") + result.stderr.decode("utf-8", errors="replace")
        log = scratch / f"owned-layers-ui-{index}.log"
        log.write_text(output, encoding="utf-8")
        if result.returncode:
            raise RuntimeError(output)
        counts = re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;", output)
        assert all(int(failed) == 0 for _, failed, _ in counts), output
        row = {"command": command, "passed": sum(int(p) for p, _, _ in counts),
               "ignored": sum(int(i) for _, _, i in counts), "suites": len(counts),
               "log": {"path": log.relative_to(ROOT).as_posix(), "sha256": digest(log)}}
        runs.append(row)
        for match in re.finditer(r"OM9_SCENE_PEAK nested_leaves=(\d+) nested_peak=(\d+) nested_limit=(\d+) cloud_points=(\d+) cloud_peak=(\d+) cloud_limit=(\d+)", output):
            leaves, nested_peak, nested_limit, points, cloud_peak, cloud_limit = map(int, match.groups())
            assert leaves == 4096 and points == 65537
            assert nested_peak <= nested_limit and cloud_peak <= cloud_limit
            peaks.append({"run": index, "nested_leaves": leaves, "nested_peak": nested_peak,
                          "nested_limit": nested_limit, "cloud_points": points,
                          "cloud_peak": cloud_peak, "cloud_limit": cloud_limit,
                          "allocator": "test_only_forced_moving_System_realloc"})
        if index == 9:
            for line in output.splitlines():
                if "OM9_LAYER_RECEIPT " in line:
                    cold.append(json.loads(line.split("OM9_LAYER_RECEIPT ", 1)[1]))
        print(json.dumps({"step": index, "passed": row["passed"], "ignored": row["ignored"]}), flush=True)
    assert [row["passed"] for row in runs[2:9]] == [1, 9, 6, 2, 1, 1, 17], runs
    assert len(peaks) == 2 and {row["run"] for row in peaks} == {7, 9}, peaks
    assert len(cold) == 4 and all(r["exited"] and r["active_processes"] == 0 for r in cold), cold
    ui_receipts = []
    for name, expected in [("2026-10-14-owned-layers-ui", 13), ("2026-10-14-owned-layers-viewport-regression", 6)]:
        path = ROOT / "docs/validation" / name / "ui-receipt.json"
        ui = json.loads(path.read_text(encoding="utf-8"))
        assert len(ui["checks"]) == expected and ui["native_window_observed"] and not ui["children"]
        assert not ui["goal_complete"]
        assert digest(ROOT / ui["binary"]["path"]) == ui["binary"]["sha256"]
        assert all(before[ROOT / row["path"]] == row["sha256"] for row in ui["sources"])
        ui_receipts.append({"path": path.relative_to(ROOT).as_posix(), "sha256": digest(path),
                            "checks": expected, "binary": ui["binary"], "screenshot": ui["screenshot"],
                            "loaded_modules": len(ui["loaded_modules"]), "children": ui["children"]})
    assert source_paths() == set(before) and all(digest(p) == sha for p, sha in before.items()), "source changed during validation"
    receipt = {"schema_version": 1, "status": "validated_owned_layers_controller_widgets_scene_slice",
               "measured_at": dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds"),
               "goal_complete": False, "full_layers_acceptance": False,
               "runs": runs, "workspace_passed": runs[9]["passed"], "workspace_ignored": runs[9]["ignored"],
               "focused_tests": sum(row["passed"] for row in runs[2:9]), "focused_checks_are_workspace_subset": True,
               "scene_allocation_peaks": peaks,
               "cold_fixtures": cold, "cold_om9_processes": len(cold), "native_ui": ui_receipts,
               "source_sha256": {p.relative_to(ROOT).as_posix(): sha for p, sha in sorted(before.items())},
               "limitations": [
                   "Advanced rename/color and selective-Show delayed dialogs are visible but unavailable; their cancellation/stale lifecycle remains open.",
                   "Current-layer output acceptance covers foundation Point/Line only; other command families remain unaccepted.",
                   "Cold archive tests cover owned metadata/schema migration, not a completed interactive save/reopen workflow.",
                   "Older SDK/native receipts retain their historical source checkpoints; no fresh combined SDK count is claimed.",
                   "Full DPI/IME, large-scene latency, GPU/global cross-job memory, complete Layers/M9IMG/RCORE gates remain open."]}
    RECEIPT.write_text(json.dumps(receipt, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"status": receipt["status"], "workspace_passed": receipt["workspace_passed"],
                      "workspace_ignored": receipt["workspace_ignored"], "focused_subset": receipt["focused_tests"],
                      "cold_om9_processes": len(cold), "native_ui_checks": [r["checks"] for r in ui_receipts],
                      "goal_complete": False}))


if __name__ == "__main__":
    main()
