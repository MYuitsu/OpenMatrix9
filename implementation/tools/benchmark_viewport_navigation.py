#!/usr/bin/env python3
"""Bounded native OM9 replay; reports app/submit/GPU timing, never presentation acceptance.

Python is a process/JSON evidence harness only. Runtime navigation and render orchestration are Rust.
Fixtures stay private under ignored build/. Each process warms all requested viewport resources before
injecting input. DPI factors simulate egui scale; they do not certify physical monitor DPI or RDP.
"""
import argparse
import hashlib
import csv
from datetime import datetime, timezone, timedelta
import itertools
import json
import math
import os
from pathlib import Path
import subprocess
import shutil
import time

ROOT = Path(__file__).resolve().parents[2]
FIXTURES = {125: "bong-125-whole-stone.om9", 729: "bong-729-whole.om9", 27000: "bong-27k.om9"}


def percentiles(values):
    values = sorted(v for v in values if isinstance(v, (int, float)) and math.isfinite(v))
    return {"count": len(values), **{f"p{p}": values[max(0, math.ceil(len(values)*p/100)-1)] if values else None for p in (50, 95, 99)}, "max": max(values, default=None)}


def analyze(records):
    frames = [r for r in records if r.get("event") == "nav_frame"]
    active = [r for r in frames if r.get("phase") in ("dragging", "settling") and r.get("encoded_frame_id") == r.get("frame_id")]
    # Scheduler dt also advances while a pane is waiting for resources. Measure actual geometry
    # submission gaps separately, retaining skipped frames inside each continuous moving phase.
    submitted_at={};submit_gaps=[]
    for r in frames:
        view=r["view"]
        if r.get("phase") not in ("dragging","settling"):
            submitted_at.pop(view,None);continue
        if r.get("encoded_frame_id")!=r.get("frame_id"):continue
        stamp=r["t_ms"]  # monotonic Instant clock; preserve its explicit 1 ms resolution
        if view in submitted_at:submit_gaps.append(stamp-submitted_at[view])
        submitted_at[view]=stamp
    # Each GPU result belongs to its original submitted frame, not the later readback frame.
    active_keys={(r["view"],r["frame_id"]) for r in active}
    gpu = {(r["view"],r["gpu_frame_id"]): r["gpu_ms"] for r in frames if (r["view"],r.get("gpu_frame_id")) in active_keys and r.get("gpu_ms") is not None}
    finals = {(r["view"],r["revision"],r["generation"]): r["refinement_ms"] for r in frames if r.get("generation",0)>0 and r.get("final_complete") and r.get("published_frame_id",r.get("encoded_frame_id")) == r.get("frame_id") and r.get("refinement_ms") is not None}
    return {"active_frames": len(active),"active_app_records":sum(r.get("phase") in ("dragging","settling") for r in frames),"active_frame_scope":"OM9 geometry submissions; not every app/present frame","geometry_timestamp_resolution_ms":1,"geometry_submit_interval_ms":percentiles(submit_gaps), "application_frame_interval_ms": percentiles(r.get("interval_ms") for r in active), "cpu_encode_ms": percentiles(r.get("submit_cpu_ms") for r in active), "receive_to_submit_ms": percentiles(r.get("receive_to_submit_ms") for r in active), "gpu_ms": percentiles(gpu.values()), "refinement_ms": percentiles(finals.values()), "qualities": sorted({r["quality"] for r in active}), "hz": sorted({r["hz"] for r in active}), "closed_cleanly": any(r.get("event")=="diagnostic_end" for r in records), "dropped_records": max((r.get("dropped_records",0) for r in records),default=0), "errors": [r for r in records if r.get("event")=="viewport_prepare" and r.get("error")], "fingerprint": next((r for r in records if r.get("event")=="diagnostic_start"),None), "gpu_configuration": next((r for r in records if r.get("event")=="gpu_configuration"),None), "presentation": "NOT_MEASURED", "human_acceptance": "PENDING"}


def analyze_presentation(path,records,clock=None):
    moving=[r for r in records if r.get("event")=="nav_frame" and r.get("phase") in ("dragging","settling")]
    if not path.exists() or not moving:return {"state":"NOT_MEASURED"}
    def wall(record):
        if record.get("wall_unix_ns"):return int(record["wall_unix_ns"])/1e9
        return int(record["session_id"].split("-",1)[1])/1e9+record["t_ms"]/1000
    start=min(map(wall,moving));end=max(map(wall,moving))
    rows=list(csv.DictReader(path.open(encoding="utf-8-sig",newline="")))
    if clock is None:return {"state":"UNCORRELATED_CLOCK","reason":"Presentation samples retained, no trusted wall/QPC calibration"}
    offset=clock["qpc_minus_wall_ms"]
    active=[]
    for row in rows:
        try:when=(float(row["CPUStartQPCTime"])-offset)/1000
        except (KeyError,ValueError,TypeError):continue
        if start<=when<=end:active.append(row)
    if active:
        with path.with_name("presentmon-active.csv").open("w",encoding="utf-8",newline="") as output:
            writer=csv.DictWriter(output,fieldnames=active[0]);writer.writeheader();writer.writerows(active)
    def numbers(field):
        for row in active:
            try:yield float(row[field])
            except (KeyError,ValueError,TypeError):pass
    return {"state":"MEASURED" if active else "NO_ACTIVE_ROWS","scope":"continuous activity envelope, including release/refinement gaps; no outlier removal","clock":clock,"rows":len(active),"displayed_frame_interval_ms":percentiles(numbers("MsBetweenDisplayChange")),"present_interval_ms":percentiles(numbers("MsBetweenPresents")),"rows_without_display_interval":sum(r.get("MsBetweenDisplayChange") in (None,"NA","") for r in active)}


def calibrate_clock():
    samples=[]
    for _ in range(5):
        before=time.perf_counter_ns();wall=time.time_ns();after=time.perf_counter_ns()
        samples.append((after-before,(before+after)/2-wall))
    uncertainty,offset=min(samples)
    return {"qpc_minus_wall_ms":offset/1e6,"pair_uncertainty_ms":uncertainty/2e6,"source":"Python perf_counter_ns (Windows QPC) paired with time_ns; app SystemTime is wall clock"}


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--exe",type=Path,default=Path("L:/om9-target/debug/om9.exe"))
    parser.add_argument("--counts",type=int,nargs="+",choices=FIXTURES,default=[125,729,27000])
    parser.add_argument("--hz",type=int,nargs="+",choices=[30,60],default=[30,60])
    parser.add_argument("--auto",action="store_true",help="Exercise default adaptive cadence; no mixed-Hz SLA certification")
    parser.add_argument("--views",type=int,nargs="+",choices=[1,4],default=[1,4])
    parser.add_argument("--dpi",type=float,nargs="+",choices=[1.,1.25,1.5],default=[1.,1.25,1.5])
    parser.add_argument("--repeats",type=int,default=3)
    parser.add_argument("--latencies",type=int,nargs="+",choices=[1,2],default=[1])
    parser.add_argument("--frames",type=int,default=180)
    parser.add_argument("--timeout",type=float,default=120)
    parser.add_argument("--output",type=Path)
    parser.add_argument("--presentmon",type=Path)
    args=parser.parse_args()
    if args.auto:args.hz=[60]
    output=args.output or ROOT/"build/standalone-validation/ui-artifacts"/time.strftime("navigation-%Y%m%d-%H%M%S")
    output.mkdir(parents=True,exist_ok=False)
    # Freeze the executable for the entire batch: a concurrent Cargo link must not change later runs
    # or hold the developer's build output open. SDK DLLs resolve from the original binary directory.
    executable=output/"om9-navigation-benchmark.exe"
    shutil.copy2(args.exe,executable)
    report={"schema_version":2,"binary":str(args.exe),"binary_sha256":hashlib.sha256(args.exe.read_bytes()).hexdigest(),"session_name":os.environ.get("SESSIONNAME"),"timing_scope":"app/OM9-submit/GPU; not presentation","dpi_scope":"egui scale simulation","runs":[]}
    for count,hz,views,dpi,latency,repeat in itertools.product(args.counts,args.hz,args.views,args.dpi,args.latencies,range(1,args.repeats+1)):
        name=f"n{count}-h{'auto' if args.auto else hz}-v{views}-dpi{dpi:g}-q{latency}-r{repeat}"
        directory=output/name;directory.mkdir()
        fixture=ROOT/"build/stress-27000"/FIXTURES[count]
        original_fixture=hashlib.sha256(fixture.read_bytes()).hexdigest()
        env=os.environ.copy()
        env["PATH"]=str(args.exe.parent)+os.pathsep+env.get("PATH","")
        for key in ("OM9_GPU_STRESS","OM9_GPU_CLUSTER","OM9_CAMERA_AUDIT","OM9_VIEWPORT_LOD"):
            env.pop(key,None)
        env.update(OM9_NAVIGATION_AUDIT=str(args.frames),OM9_FRAME_HZ=str(hz),OM9_FRAME_LATENCY=str(latency),OM9_NAV_SPEED="1",OM9_NAVIGATION_AUDIT_STDIN_GATE="0",OM9_NAVIGATION_AUDIT_VIEWS=str(views),OM9_NAVIGATION_AUDIT_DPI=str(dpi),OM9_NAVIGATION_AUDIT_SCREENSHOT=str(directory/"final.png"),OM9_INTERACTION_LOG=str(directory/"interaction.jsonl"),OM9_STARTUP_DIALOG="0")
        if args.auto:env["OM9_FRAME_HZ"]="auto"
        if args.presentmon:env["OM9_NAVIGATION_AUDIT_STDIN_GATE"]="1"
        started=time.monotonic();timed_out=False;clock=calibrate_clock()
        with (directory/"stdout.log").open("w") as stdout,(directory/"stderr.log").open("w") as stderr:
            child=subprocess.Popen([str(executable),"--ui",str(fixture)],cwd=args.exe.parent,env=env,stdin=subprocess.PIPE if args.presentmon else subprocess.DEVNULL,stdout=stdout,stderr=stderr)
            monitor=None
            monitor_log=(directory/"presentmon.log").open("w")
            session=f"OM9-nav-{child.pid}-{time.time_ns()}"
            if args.presentmon:
                monitor=subprocess.Popen([str(args.presentmon),"--process_id",str(child.pid),"--session_name",session,"--output_file",str(directory/"presentmon.csv"),"--qpc_time_ms","--timed",str(math.ceil(args.timeout)),"--terminate_after_timed","--terminate_on_proc_exit","--no_console_stats"],stdout=monitor_log,stderr=monitor_log)
                time.sleep(2)
                child.stdin.write(b"ready\n");child.stdin.flush();child.stdin.close()
            try:
                code=child.wait(timeout=args.timeout)
            except subprocess.TimeoutExpired:
                timed_out=True;child.terminate();code=child.wait(timeout=10)
            except BaseException:
                child.terminate();child.wait(timeout=10);raise
            finally:
                if monitor:
                    # Close only this harness's unique ETW session, allowing the collector to flush.
                    subprocess.run([str(args.presentmon),"--session_name",session,"--terminate_existing_session"],stdout=monitor_log,stderr=monitor_log,timeout=10,check=False)
                    try:monitor.wait(timeout=10)
                    except subprocess.TimeoutExpired:monitor.terminate();monitor.wait(timeout=10)
                monitor_log.close()
        records=[]
        path=directory/"interaction.jsonl"
        if path.exists():
            for line in path.read_text(encoding="utf-8").splitlines():
                try:records.append(json.loads(line))
                except json.JSONDecodeError:pass
        run={"name":name,"count":count,"views":views,"dpi":dpi,"fixed_hz":hz,"frame_latency":latency,"repeat":repeat,"fixture_sha256":hashlib.sha256(fixture.read_bytes()).hexdigest(),"exit_code":code,"timeout":timed_out,"elapsed_seconds":time.monotonic()-started,"log":path.relative_to(ROOT).as_posix(),"screenshot_saved":(directory/"final.png").exists(),"presentmon_exit":monitor.returncode if monitor else None,"presentmon_csv":str(directory/"presentmon.csv") if (directory/"presentmon.csv").exists() else None,**analyze(records)}
        if args.auto:run["fixed_hz"]=None
        run["fixture_sha256_before"]=original_fixture;run["fixture_unchanged"]=run["fixture_sha256"]==original_fixture
        moves=[r["t_ms"] for r in records if r.get("event")=="navigation_input" and r.get("raw_event","").startswith("PointerMoved")]
        run["raw_pointer_move_interval_ms"]=percentiles(b-a for a,b in zip(moves,moves[1:]) if b>a)
        clock["end_offset_drift_ms"]=calibrate_clock()["qpc_minus_wall_ms"]-clock["qpc_minus_wall_ms"]
        run["presentation"]=analyze_presentation(directory/"presentmon.csv",records,clock)
        report["runs"].append(run);(directory/"receipt.json").write_text(json.dumps(run,indent=2)+"\n",encoding="utf-8")
        (output/"summary.json").write_text(json.dumps(report,indent=2)+"\n",encoding="utf-8")
        print(json.dumps({"run":name,"exit":code,"timeout":timed_out,"active_frames":run["active_frames"],"interval":run["application_frame_interval_ms"],"gpu":run["gpu_ms"],"final":run["refinement_ms"]}),flush=True)
    print(output,flush=True)


if __name__ == "__main__":
    main()
