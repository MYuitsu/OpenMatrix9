"""Sample an owned native sphere audit on the physical Windows console.

Python is the Windows process/driver telemetry adapter; the Rust native replay
owns CAD commands, full Undo, exact-source checks and GPU publication timing.
"""
import argparse
import datetime
import json
import os
from pathlib import Path
import subprocess
import time

import psutil


GPU_FIELDS = ["utilization.gpu", "utilization.memory", "memory.used", "memory.total",
              "temperature.gpu", "power.draw", "clocks.current.graphics"]


def command(args):
    started = time.perf_counter()
    try:
        result = subprocess.run(args, capture_output=True, text=True, timeout=8,
                                creationflags=subprocess.CREATE_NO_WINDOW)
        return {"code": result.returncode, "stdout": result.stdout.strip(),
                "stderr": result.stderr.strip(),
                "sampler_ms": (time.perf_counter() - started) * 1000}
    except (OSError, subprocess.TimeoutExpired) as error:
        return {"error": str(error), "sampler_ms": (time.perf_counter() - started) * 1000}


def console_active(result):
    return any("console" in line and "Active" in line
               for line in result.get("stdout", "").splitlines())


def audit_failure(output, cursor):
    """Consume only new complete trace lines; a blocked Close must not hide failure."""
    traces = list(output.glob("*.jsonl"))
    if not traces:
        return None
    path = traces[0]
    offset = cursor.get(path, 0)
    with path.open("rb") as stream:
        stream.seek(offset)
        while line := stream.readline():
            if not line.endswith(b"\n"):
                break
            offset = stream.tell()
            if b'"sphere_audit_failure"' in line:
                record = json.loads(line)
                if record.get("event") == "sphere_audit_failure":
                    cursor[path] = offset
                    return record
    cursor[path] = offset
    return None


def process_sample(proc, previous, now):
    with proc.oneshot():
        cpu = proc.cpu_times()._asdict()
        memory = proc.memory_info()._asdict()
        io = proc.io_counters()._asdict()
        threads = {r.id: r.user_time + r.system_time for r in proc.threads()}
        total = cpu["user"] + cpu["system"]
        sample = {"pid": proc.pid, "cpu_seconds": cpu, "memory_bytes": memory,
                  "page_faults_total": memory.pop("num_page_faults", None),
                  "io_cumulative": io, "thread_count": len(threads),
                  "handles": proc.num_handles(),
                  "context_switches": proc.num_ctx_switches()._asdict()}
    if previous:
        elapsed = now - previous["time"]
        cores = (total - previous["cpu"]) / elapsed
        sample["cpu_cores_used"] = cores
        sample["cpu_machine_percent"] = 100 * cores / psutil.cpu_count()
        sample["top_threads"] = sorted(
            [{"tid": tid, "cpu_cores_used": max(0, value - previous["threads"].get(tid, value)) / elapsed}
             for tid, value in threads.items()],
            key=lambda row: row["cpu_cores_used"], reverse=True)[:8]
    return sample, {"time": now, "cpu": total, "threads": threads}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--exe", type=Path, default=Path("L:/om9-target/release/om9.exe"))
    parser.add_argument("--objects", nargs="+", type=int, required=True)
    parser.add_argument("--profiles", nargs="+", choices=["light", "high"], default=["light", "high"])
    parser.add_argument("--no-profile-commit", action="store_true", help="measure desktop without detailed admission stderr timing")
    parser.add_argument("--max-points", type=int, help="explicit diagnostic scene-complexity allowance; does not disable validation")
    args = parser.parse_args()
    assert all(4 <= n <= 1024 and n % 2 == 0 for n in args.objects)
    assert args.max_points is None or 1 <= args.max_points <= 500000000
    args.output.mkdir(parents=True, exist_ok=True)
    session = command(["query", "session"])
    assert console_active(session), session
    receipt = {"utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
               "session": session, "logical_cpus": psutil.cpu_count(),
               "physical_cpus": psutil.cpu_count(logical=False),
               "ram_bytes": psutil.virtual_memory().total,
               "gpu_fields": GPU_FIELDS,
               "gpu_units": ["percent", "percent", "MiB", "MiB", "C", "W", "MHz"],
               "gpu_scope": "whole adapter, not process VRAM",
               "executable": str(args.exe.resolve(strict=True)),
               "max_points_override": args.max_points,
               "notes": ["1 Hz samples; process CPU in cores, then normalized to machine percent",
                         "page faults are total faults, not hard-fault counts",
                         f"commit profiling enabled: {not args.no_profile_commit}; one run per configuration; four viewports; no Boolean",
                         "current application replay cap 30 Hz, not a monitor or RDP cap"],
               "hardware": command(["powershell", "-NoProfile", "-Command",
                    "Get-CimInstance Win32_VideoController | Select-Object Name,DriverVersion,CurrentHorizontalResolution,CurrentVerticalResolution,CurrentRefreshRate | ConvertTo-Json"]),
               "cpu": command(["powershell", "-NoProfile", "-Command",
                    "Get-CimInstance Win32_Processor | Select-Object Name,NumberOfCores,NumberOfLogicalProcessors,MaxClockSpeed | ConvertTo-Json"]),
               "power_plan": command(["powercfg", "/getactivescheme"]),
               "driver": command(["nvidia-smi", "--query-gpu=name,driver_version", "--format=csv,noheader"])}
    (args.output / ("console-" + "-".join(map(str, args.objects)) + ".json")).write_text(
        json.dumps(receipt, indent=2), encoding="utf-8")
    for count in args.objects:
        for profile in args.profiles:
            output = args.output / f"native-{count}-{profile}"
            assert not output.exists(), output
            log = args.output / f"native-{count}-{profile}.log"
            samples = args.output / f"native-{count}-{profile}.system.ndjson"
            print(f"starting {count} {profile}", flush=True)
            env = os.environ.copy()
            # Matches the previous full-history fixture's retained-object admission allowance.
            # No Undo memory, history depth, or geometry point budget override is introduced.
            env["OM9_MAX_OBJECTS"] = str(100000 if count <= 16 else 400000)
            if args.max_points is not None:
                env["OM9_MAX_POINTS"] = str(args.max_points)
            with log.open("w", encoding="utf-8") as stream, samples.open("w", encoding="utf-8") as telemetry:
                runner = subprocess.Popen([
                    "python", "tools/benchmark_sphere_viewport.py", "--output", str(output),
                    "--exe", str(args.exe),
                    "--pairs", str(count // 2), "--profiles", profile, "--repeats", "1",
                    "--viewport-only", "--resources", *([] if args.no_profile_commit else ["--profile-commit"]), "--timeout-seconds", "14400"],
                    env=env, stdout=stream, stderr=subprocess.STDOUT)
                owner = psutil.Process(runner.pid)
                previous = {}
                audit_cursor = {}
                epoch = time.perf_counter()
                next_session = epoch
                try:
                    while runner.poll() is None:
                        tick = time.perf_counter()
                        row = {"utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
                               "elapsed_s": tick - epoch,
                               "system_available_bytes": psutil.virtual_memory().available,
                               "processes": []}
                        failure = audit_failure(output, audit_cursor)
                        if failure is not None:
                            (output / "audit-failure.json").write_text(json.dumps(failure, indent=2), encoding="utf-8")
                            raise RuntimeError(f"Owned OM9 audit failed: {failure.get('data')}; stopped before further workloads")
                        try:
                            children = owner.children(recursive=True)
                        except psutil.NoSuchProcess:
                            children = []
                        for child in children:
                            try:
                                if child.name().lower() == args.exe.name.lower():
                                    sample, previous[child.pid] = process_sample(child, previous.get(child.pid), tick)
                                    from om9_window_metrics import windows
                                    sample["windows"] = windows({child.pid})
                                    row["processes"].append(sample)
                                    if any(w["visible"] and w["hung"] for w in sample["windows"]):
                                        telemetry.write(json.dumps(row) + "\n")
                                        telemetry.flush()
                                        raise RuntimeError("Owned OM9 audit window is hung; stopped before larger workloads")
                            except (psutil.NoSuchProcess, psutil.AccessDenied):
                                pass
                        row["gpu"] = command(["nvidia-smi", "--query-gpu=" + ",".join(GPU_FIELDS),
                                              "--format=csv,noheader,nounits"])
                        if tick >= next_session:
                            row["session"] = command(["query", "session"])
                            next_session = tick + 30
                            if not console_active(row["session"]):
                                telemetry.write(json.dumps(row) + "\n")
                                telemetry.flush()
                                raise RuntimeError("Physical console is no longer Active; measurement interrupted")
                        row["sampler_total_ms"] = (time.perf_counter() - tick) * 1000
                        telemetry.write(json.dumps(row) + "\n")
                        telemetry.flush()
                        time.sleep(max(0, 1 - (time.perf_counter() - tick)))
                finally:
                    if runner.poll() is None:
                        for child in owner.children(recursive=True):
                            if child.name().lower() == args.exe.name.lower():
                                child.terminate()
                        runner.terminate()
                        runner.wait(timeout=10)
                assert runner.returncode == 0, f"{output}: runner exit {runner.returncode}; see {log}"
            print(f"completed {count} {profile}", flush=True)


if __name__ == "__main__":
    main()
