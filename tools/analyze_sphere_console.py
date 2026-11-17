"""Summarize native replay and driver/process samples without altering the audit."""
import argparse
import bisect
import collections
import json
from pathlib import Path
import re


def stats(values):
    values = sorted(v for v in values if v is not None)
    if not values:
        return None
    return {"count": len(values), "sum": sum(values), "mean": sum(values) / len(values),
            **{label: values[round((len(values) - 1) * q)]
               for label, q in [("p50", .5), ("p95", .95), ("p99", .99), ("max", 1)]}}


def interrupted_summary(rows):
    """Observed performance only; never substitute for the strict source/history audit."""
    assert not any(r.get("event", "").startswith("boolean_") for r in rows)
    checkpoint = next(r["data"] for r in rows if r.get("event") == "sphere_audit_resources" and r["data"]["point"] == "performance_end")
    count = checkpoint["objects"]
    assert [r["data"]["index"] for r in rows if r.get("event") == "sphere_command_end"] == list(range(1, count + 1))
    assert [r["data"]["objects"] for r in rows if r.get("event") == "sphere_viewport_presented"] == list(range(1, count + 1))
    phase = 0
    seen = set()
    fresh = collections.defaultdict(list)
    frames = collections.defaultdict(list)
    for row in rows:
        if row.get("event") == "sphere_audit_frame":
            phase = row["stage"]
            frames[phase].append(row)
        elif row.get("event") == "viewport_publication" and row.get("view") == "Perspective":
            frame = row.get("frame_id")
            if frame is not None and frame not in seen:
                seen.add(frame)
                fresh[phase].append(row["t_ms"])
    steady = {}
    for stage, label in [(2, "scene"), (6, "selected_scene")]:
        low = frames[stage][0]["t_ms"] + 1000
        high = frames[stage][-1]["t_ms"] - 250
        times = [t for t in fresh[stage] if low <= t <= high]
        assert len(times) > 1
        gaps = [times[0] - low, *[b - a for a, b in zip(times, times[1:])], high - times[-1]]
        steady[label] = {"fps_published": len(times) * 1000 / (high - low), "max_gpu_hold_ms": max(gaps),
                         "ui_max_ms": max(r["data"]["interval_ms"] or 0 for r in frames[stage] if low <= r["t_ms"] <= high)}
    inputs = next(r["data"] for r in rows if r.get("event") == "sphere_audit_inputs")
    return {"objects": count, "steady": steady, "input_triangles": sum(b["triangles"] for b in inputs["bodies"]),
            "ram_peak_mib": checkpoint["process"]["peak_working_set_bytes"] / 2**20,
            "ram_settled_mib": checkpoint["process"]["working_set_bytes"] / 2**20,
            "undo_steps": checkpoint["undo_steps"],
            "resources": [r["data"] for r in rows if r.get("event") == "sphere_audit_resources"]}


def analyze(folder, allow_incomplete=False):
    trace = next(folder.glob("*.jsonl"))
    rows = [json.loads(line) for line in trace.read_text().splitlines()]
    complete = (folder / "summary.json").exists()
    if complete:
        summary = json.loads((folder / "summary.json").read_text())[0]
    else:
        assert allow_incomplete, "No strict audit summary; pass --include-incomplete for diagnostic-only extraction"
        summary = interrupted_summary(rows)
    ends = [r for r in rows if r.get("event") == "sphere_command_end"]
    starts = {r["data"]["index"]: r for r in rows if r.get("event") == "sphere_command_start"}
    presented = {r["data"]["objects"]: r for r in rows if r.get("event") == "sphere_viewport_presented"}
    commands = [{"index": r["data"]["index"], "command_ms": r["elapsed_ms"] - starts[r["data"]["index"]]["elapsed_ms"],
                 "commit_to_present_ms": presented[r["data"]["index"]]["data"]["commit_to_present_ms"]}
                for r in ends]
    for r in commands:
        r["command_to_present_ms"] = r["command_ms"] + r["commit_to_present_ms"]
    phases = collections.defaultdict(list)
    command_phases = collections.defaultdict(lambda: collections.defaultdict(float))
    command_phase_counts = collections.defaultdict(lambda: collections.defaultdict(int))
    outside = collections.defaultdict(list)
    command_index = None
    for line in trace.with_suffix(".stderr.txt").read_text(errors="replace").splitlines():
        if line.startswith("OM9_SPHERE_PROFILE begin"):
            command_index = int(re.search(r"index=(\d+)", line)[1])
        elif line.startswith("OM9_SPHERE_PROFILE end"):
            command_index = None
        elif line.startswith("OM9_COMMIT_PROFILE"):
            for name, value in re.findall(r"\b(\w+_ms)=([\d.]+)", line):
                value = float(value)
                phases[name].append(value)
                if command_index is None:
                    outside[name].append(value)
                else:
                    command_phases[command_index][name] += value
                    command_phase_counts[command_index][name] += 1
    blocks = []
    for low in range(1, len(commands) + 1, 32):
        block = [r for r in commands if low <= r["index"] < low + 32]
        names = sorted({k for r in block for k in command_phases[r["index"]]})
        blocks.append({"first": low, "last": block[-1]["index"],
                       **{key: stats([r[key] for r in block]) for key in ["command_ms", "commit_to_present_ms", "command_to_present_ms"]},
                       "phase_calls_per_command": {name: stats([command_phase_counts[r["index"]].get(name, 0) for r in block]) for name in ["retained_total_ms", "undo_inventory_ms"]},
                       "phase_ms_per_command": {name: stats([command_phases[r["index"]].get(name, 0) for r in block]) for name in names}})
    frame_stages = collections.defaultdict(list)
    native_stages = collections.defaultdict(list)
    gpu_stages = collections.defaultdict(list)
    seen_gpu = set()
    stage = 0
    for row in rows:
        event = row.get("event")
        if event == "sphere_audit_frame":
            # Attribute an interval to its beginning, including a command crossing a stage boundary.
            frame_stages[stage].append(row["data"].get("interval_ms"))
            stage = row["stage"]
        elif event == "native_frame_timing":
            native_stages[stage].append(row)
        elif event == "nav_frame" and row.get("gpu_ms") is not None:
            key = (row["view"], row.get("gpu_frame_id"))
            if key not in seen_gpu:
                seen_gpu.add(key)
                gpu_stages[stage].append(row["gpu_ms"])
    def frame_summary(values):
        values = [v for v in values if v is not None]
        return {"interval_ms": stats(values),
                "over_ms": {str(t): sum(v > t for v in values) for t in [50, 100, 200, 500, 1000]}}
    measured_end = next(r for r in rows if r.get("event") == "sphere_audit_resources" and r["data"]["point"] == "performance_end")
    samples_path = folder.parent / (folder.name + ".system.ndjson")
    samples = [json.loads(line) for line in samples_path.read_text().splitlines()]
    deadline = int(measured_end["wall_unix_ns"]) / 1e9
    import datetime
    samples = [r for r in samples if datetime.datetime.fromisoformat(r["utc"]).timestamp() <= deadline]
    processes = [p for r in samples for p in r["processes"]]
    markers = [(int(r["wall_unix_ns"]) / 1e9, r["stage"]) for r in rows if r.get("event") == "sphere_audit_frame"]
    marker_times = [r[0] for r in markers]
    by_stage = collections.defaultdict(lambda: collections.defaultdict(list))
    gpu = []
    for row in samples:
        stamp = datetime.datetime.fromisoformat(row["utc"]).timestamp()
        offset = bisect.bisect_right(marker_times, stamp) - 1
        stage = markers[offset][1] if offset >= 0 else -1
        for proc in row["processes"]:
            by_stage[stage]["cpu_cores_used"].append(proc.get("cpu_cores_used"))
            by_stage[stage]["private_mib"].append(proc["memory_bytes"]["private"] / 2**20)
        try:
            values = [float(x) for x in row["gpu"]["stdout"].splitlines()[0].split(",")]
            gpu.append(values)
            by_stage[stage]["gpu_util_percent"].append(values[0])
            by_stage[stage]["adapter_vram_mib"].append(values[2])
        except (KeyError, ValueError, IndexError):
            pass
    window_states = collections.defaultdict(list)
    window_path = folder.parent / "window-state.ndjson"
    if window_path.exists():
        pids = {p["pid"] for p in processes}
        audit_start = next(int(r["wall_unix_ns"]) / 1e9 for r in rows if r.get("event") == "sphere_audit_start")
        for line in window_path.read_text().splitlines():
            row = json.loads(line)
            stamp = datetime.datetime.fromisoformat(row["utc"]).timestamp()
            if not audit_start <= stamp <= deadline:
                continue
            candidates = [w for w in row["windows"] if w["pid"] in pids and min(w["client_pixels"]) > 200]
            if candidates:
                window = max(candidates, key=lambda w: (w["visible"], w["client_pixels"][0] * w["client_pixels"][1]))
                stage = markers[max(0, bisect.bisect_right(marker_times, stamp) - 1)][1]
                window_states[stage].append(window)
    return {"status": "strict_audit_passed" if complete else "interrupted_performance_only_source_and_undo_redo_unverified",
            "objects": summary["objects"], "profile": ends[0]["profile"],
            "trace": str(trace), "gpu_configuration": next(r for r in rows if r.get("event") == "gpu_configuration"),
            "presentation_policy": next(r for r in rows if r.get("event") == "presentation_policy"),
            "steady": summary["steady"], "input_triangles": summary["input_triangles"],
            "ram_peak_mib": summary["ram_peak_mib"], "ram_settled_mib": summary["ram_settled_mib"],
            "undo_steps": summary["undo_steps"], "resources": summary["resources"],
            "creation_elapsed_s": (presented[len(commands)]["elapsed_ms"] - starts[1]["elapsed_ms"]) / 1000,
            "commands": {key: stats([r[key] for r in commands]) for key in ["command_ms", "commit_to_present_ms", "command_to_present_ms"]},
            "slowest_commands": [dict(r, phase_ms=dict(command_phases[r["index"]]),
                                      phase_calls=dict(command_phase_counts[r["index"]]))
                                 for r in sorted(commands, key=lambda r: r["command_ms"], reverse=True)[:10]],
            "command_blocks": blocks,
            "audit_phase_ms": {k: stats(v) for k, v in sorted(phases.items())},
            "command_phase_ms": {name: stats([values.get(name, 0) for values in command_phases.values()])
                                 for name in sorted(phases)},
            "outside_command_phase_ms": {k: stats(v) for k, v in sorted(outside.items())},
            "phase_note": "Nested/cumulative phase totals overlap. Audit-wide and outside-command phases include post-performance verification; command phases exclude it. Command assignment is the stderr begin/end interval, not exclusive thread ownership. Legacy binaries thresholded retained logs at 50ms; current binaries log every call when profiling is enabled. Performance runs may disable profiling.",
            "frame_stages": {str(k): frame_summary(v) for k, v in frame_stages.items()},
            "native_frame_stages": {str(k): {name: stats([r.get(name) for r in records]) for name in ["ui_ms", "platform_and_tessellation_ms", "paint_and_wait_ms", "interval_ms"]} for k, records in native_stages.items()},
            "native_ui_work_over_ms_by_stage": {str(k): {str(threshold): sum((r.get("ui_ms") or 0) > threshold for r in records)
                for threshold in [33.3, 50, 100, 500, 1000]} for k, records in native_stages.items()},
            "gpu_timestamp_ms_by_stage": {str(k): stats(v) for k, v in gpu_stages.items()},
            "selection_projection_ms": stats([r["elapsed_ms"] for r in rows if r.get("event") == "selection_frame_rebuild"]),
            "settled_gpu_counters": next(r["data"]["views"] for r in rows if r.get("event") == "sphere_audit_settled"),
            "system": {"samples_before_verification": len(samples),
                       "window_samples_by_stage": {str(stage): {"samples": len(windows),
                            **{key: sum(bool(w[key]) for w in windows) for key in ["visible", "minimized", "foreground", "cloaked"]}}
                            for stage, windows in window_states.items()},
                       "window_note": "Read-only sample coverage starts during the 512 Light run. Visible/uncloaked does not prove the window is unoccluded or physically scanned out.",
                       "by_stage": {str(stage): {key: stats(values) for key, values in measurements.items()}
                                    for stage, measurements in by_stage.items()},
                       "sampler_ms": stats([r["sampler_total_ms"] for r in samples]),
                       "cpu_cores_used": stats([p.get("cpu_cores_used") for p in processes]),
                       "thread_count": stats([p["thread_count"] for p in processes]),
                       "handles": stats([p["handles"] for p in processes]),
                       "private_mib": stats([p["memory_bytes"]["private"] / 2**20 for p in processes]),
                       "peak_pagefile_mib": max((p["memory_bytes"]["peak_pagefile"] / 2**20 for p in processes), default=None),
                       "gpu_util_percent": stats([r[0] for r in gpu]),
                       "adapter_vram_mib": stats([r[2] for r in gpu]),
                       "gpu_temperature_c": stats([r[4] for r in gpu]),
                       "gpu_power_w": stats([r[5] for r in gpu]),
                       "gpu_graphics_mhz": stats([r[6] for r in gpu]),
                       "last_process_sample": processes[-1] if processes else None}}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("root", type=Path)
    parser.add_argument("--include-incomplete", action="store_true", help="extract observed performance without claiming strict source/Undo verification")
    args = parser.parse_args()
    results = []
    for folder in sorted(p for p in args.root.glob("native-*") if p.is_dir()):
        if not (folder / "summary.json").exists() and not args.include_incomplete:
            continue
        result = analyze(folder, args.include_incomplete)
        (folder / "metrics.json").write_text(json.dumps(result, indent=2), encoding="utf-8")
        results.append(result)
    (args.root / "metrics.json").write_text(json.dumps(results, indent=2), encoding="utf-8")
    print(json.dumps([{k: r[k] for k in ["objects", "profile", "steady", "creation_elapsed_s", "commands", "ram_peak_mib", "undo_steps"]} for r in results], indent=2))


if __name__ == "__main__":
    main()
