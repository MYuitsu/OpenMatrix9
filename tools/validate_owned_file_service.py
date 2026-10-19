"""Freeze Task1 File lifecycle service; File UI/codecs/full RCORE gates stay open."""
import datetime as dt
import hashlib
import json
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]
RECEIPT = ROOT / "docs/validation/2026-10-14-owned-file-service.json"


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def source_paths():
    return {p for p in (ROOT / "standalone").rglob("*") if p.is_file()
            and "target" not in p.parts and p.suffix in {".rs", ".toml", ".lock", ".json", ".cpp", ".hpp"}} | set(
                (ROOT / "Resources/icons").rglob("*.svg")) | {
                    Path(__file__).resolve(), ROOT / "tools/validate_standalone_ui.py",
                    ROOT / "tools/validate_standalone_foundation.py",
                    ROOT / "native/CMakeLists.txt", ROOT / "rust/src/layers.rs"}


def main():
    before = {p: digest(p) for p in source_paths()}
    scratch = ROOT / "build/standalone-validation"
    scratch.mkdir(parents=True, exist_ok=True)
    commands = [
        ("format", ["cargo", "fmt", "--all", "--", "--check"]),
        ("clippy", ["cargo", "clippy", "--workspace", "--all-targets", "--locked", "-j", "2", "--", "-D", "warnings"]),
    ]
    focused = [("om9-app", "layer_dialogs", 11), ("om9-app", "layer_widgets", 19),
               ("om9-app", "layer_controller", 9), ("om9-core", "lazy_layer_assignment", 1),
               ("om9-scene", "metadata_cache", 2), ("om9-scene", "instance_admission", 1),
               ("om9-app", "scene_allocation_peak", 1), ("om9-app", "viewport_title_menu", 17), ("om9-app", "file_lifecycle", 17),
               ("om9-core", "coordinator_held_startup", 1), ("om9-app", "file_lifecycle_cold", 1)]
    commands += [(name, ["cargo", "test", "--locked", "-j", "2", "-p", package, "--test", name, "--", "--nocapture"])
                 for package, name, _ in focused]
    commands += [("workspace", ["cargo", "test", "--workspace", "--locked", "-j", "2", "--", "--nocapture"]),
                 ("release", ["cargo", "build", "--release", "--locked", "-j", "2", "-p", "om9-app", "--bin", "om9"])]
    ui_modes = [("--layer-dialogs", "owned-file-service-layer-dialogs-regression", 10),
                ("--layers", "owned-file-service-layers-regression", 13),
                ("--viewport-menu", "owned-file-service-viewport-regression", 6)]
    runs, cold, scene_peaks, modal_peaks = [], [], [], []
    for name, command in commands:
        result = subprocess.run(["rtk", "proxy", *command], cwd=ROOT / "standalone", capture_output=True, timeout=300)
        output = result.stdout.decode("utf-8") + result.stderr.decode("utf-8", errors="replace")
        log = scratch / f"owned-file-service-{name}.log"
        log.write_text(output, encoding="utf-8")
        if result.returncode:
            raise RuntimeError(output)
        counts = re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;", output)
        assert all(int(failed) == 0 for _, failed, _ in counts)
        runs.append({"name": name, "command": ["rtk", "proxy", *command],
                     "passed": sum(int(p) for p, _, _ in counts), "ignored": sum(int(i) for _, _, i in counts),
                     "suites": len(counts), "log": {"path": log.relative_to(ROOT).as_posix(), "sha256": digest(log)}})
        if name == "workspace":
            cold = [json.loads(line.split("OM9_LAYER_RECEIPT ", 1)[1]) for line in output.splitlines()
                    if "OM9_LAYER_RECEIPT " in line]
            cold += [json.loads(line.split("OM9_FILE_RECEIPT ", 1)[1]) for line in output.splitlines()
                     if "OM9_FILE_RECEIPT " in line]
        for match in re.finditer(r"OM9_SCENE_PEAK nested_leaves=(\d+) nested_peak=(\d+) nested_limit=(\d+) cloud_points=(\d+) cloud_peak=(\d+) cloud_limit=(\d+)", output):
            leaves, nested, nested_limit, points, cloud, cloud_limit = map(int, match.groups())
            assert leaves == 4096 and points == 65537 and nested <= nested_limit and cloud <= cloud_limit
            scene_peaks.append({"run": name, "nested_leaves": leaves, "nested_peak": nested,
                                "nested_limit": nested_limit, "cloud_points": points, "cloud_peak": cloud,
                                "cloud_limit": cloud_limit, "allocator": "test_only_forced_moving_System_realloc"})
        for match in re.finditer(r"OM9_MODAL_PEAK publication_peak=(\d+) publication_limit=(\d+) hidden_roots=(\d+) label_bytes=(\d+) snapshot_peak=(\d+) snapshot_limit=(\d+) refused=true epoch_unchanged=true", output):
            publication, limit, roots, label_bytes, snapshot, snapshot_limit = map(int, match.groups())
            assert publication <= limit and snapshot <= snapshot_limit and roots == 1000 and label_bytes == 8192
            modal_peaks.append({"run": name, "publication_peak": publication, "publication_limit": limit,
                                "hidden_roots": roots, "label_bytes": label_bytes, "snapshot_peak": snapshot,
                                "snapshot_limit": snapshot_limit, "refused": True, "epoch_unchanged": True})
        print(json.dumps({"step": name, "passed": runs[-1]["passed"], "ignored": runs[-1]["ignored"]}), flush=True)
    by_name = {row["name"]: row for row in runs}
    assert all(by_name[name]["passed"] == count for _, name, count in focused)
    assert by_name["workspace"]["passed"] == 458 and by_name["workspace"]["ignored"] == 24
    assert len(cold) == 7 and all(row["exited"] and row["active_processes"] == 0 for row in cold)
    assert {row["run"] for row in scene_peaks} == {"scene_allocation_peak", "workspace"} and len(scene_peaks) == 2
    assert {row["run"] for row in modal_peaks} == {"scene_allocation_peak", "workspace"} and len(modal_peaks) == 2
    binary = ROOT / "target/release/om9.exe"
    binary_sha = digest(binary)
    native_ui = []
    for mode, name, expected in ui_modes:
        directory = f"docs/validation/2026-10-14-{name}"
        command = ["rtk", "proxy", "python", "tools/validate_standalone_ui.py", mode, "--output", directory]
        result = subprocess.run(command, cwd=ROOT, capture_output=True, timeout=90)
        if result.returncode:
            raise RuntimeError(result.stdout.decode(errors="replace") + result.stderr.decode(errors="replace"))
        path = ROOT / directory / "ui-receipt.json"
        ui = json.loads(path.read_text(encoding="utf-8"))
        assert len(ui["checks"]) == expected and ui["native_window_observed"] and not ui["children"]
        assert not ui["goal_complete"] and ui["binary"]["sha256"] == binary_sha
        assert all(before[ROOT / row["path"]] == row["sha256"] for row in ui["sources"])
        assert digest(ROOT / ui["screenshot"]["path"]) == ui["screenshot"]["sha256"]
        if mode == "--layer-dialogs":
            assert {row["dialog"] for row in ui["dialog_screenshots"]} == {"rename", "color", "selective-show"}
            assert len(ui["dialog_capture_metrics"]) == 3 and all(row["saved"] for row in ui["dialog_capture_metrics"])
            for row in ui["dialog_screenshots"]:
                assert row["size"] == [1600, 1000] and digest(ROOT / row["path"]) == row["sha256"]
        native_ui.append({"path": path.relative_to(ROOT).as_posix(), "sha256": digest(path), "checks": expected,
                          "binary": ui["binary"], "screenshot": ui["screenshot"],
                          "dialog_screenshots": ui["dialog_screenshots"],
                          "loaded_modules": len(ui["loaded_modules"]), "children": ui["children"]})
        print(result.stdout.decode().strip(), flush=True)
    assert digest(binary) == binary_sha and source_paths() == set(before)
    assert all(digest(path) == sha for path, sha in before.items()), "source changed during verification"
    receipt = {"schema_version": 1, "status": "validated_owned_file_lifecycle_service_slice",
               "measured_at": dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds"),
               "goal_complete": False, "full_layers_acceptance": False, "full_file_acceptance": False, "file_ui_connected": False,
               "workspace_passed": by_name["workspace"]["passed"], "workspace_ignored": by_name["workspace"]["ignored"],
               "focused_tests": sum(count for _, _, count in focused), "focused_checks_are_workspace_subset": True,
               "runs": runs, "scene_allocation_peaks": scene_peaks, "modal_allocation_peaks": modal_peaks,
               "cold_fixtures": cold, "cold_om9_processes": len(cold), "native_ui": native_ui,
               "source_sha256": {p.relative_to(ROOT).as_posix(): sha for p, sha in sorted(before.items())},
               "limitations": ["Task1 File lifecycle service is wired to controller APIs only; File menu/CMD/picker UI is not enabled or accepted.",
                               "Cold fixtures validate controller Save/New/Open/SaveAs and archive state; interactive native picker/save/open, complete view/material persistence and .3dm remain pending.",
                               "No fresh SDK jobs; previous SDK evidence retains historical source checkpoints.",
                               "Global egui/GPU caches, async capture cancellation, broad OS IME/DPI, latency and full Layers/M9IMG/RCORE gates remain open."]}
    RECEIPT.write_text(json.dumps(receipt, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"status": receipt["status"], "workspace": receipt["workspace_passed"],
                      "focused_subset": receipt["focused_tests"], "native_checks": [10, 13, 6],
                      "cold_om9_processes": len(cold), "goal_complete": False}))


if __name__ == "__main__":
    main()
