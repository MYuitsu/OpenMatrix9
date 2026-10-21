#!/usr/bin/env python3
"""Check runtime evidence identities/lifecycle without promoting a manual/performance gate."""
import argparse
import json
import math
from pathlib import Path


def check(summary_path,root):
    report=json.loads(summary_path.read_text(encoding="utf-8"))
    errors=[];app_target_failures=[];seen=set();details=[];measurements=[]
    metrics=("geometry_submit_interval_ms","application_frame_interval_ms","raw_pointer_move_interval_ms","receive_to_submit_ms","cpu_encode_ms","gpu_ms","refinement_ms")
    for run in report["runs"]:
        name=run["name"]
        if name in seen:errors.append([name,"duplicate run"])
        seen.add(name)
        if run["exit_code"]!=0 or run["timeout"]:errors.append([name,"unclean process exit"])
        if not run["screenshot_saved"]:errors.append([name,"missing final screenshot"])
        if run.get("fixture_unchanged") is False:errors.append([name,"canonical fixture file changed"])
        if run.get("errors"):errors.append([name,"viewport preparation reported an error"])
        if not run["refinement_ms"]["count"]:errors.append([name,"no warm final image completion observed"])
        rows=[json.loads(line) for line in (root/run["log"]).read_text(encoding="utf-8").splitlines()]
        start=next((r for r in rows if r.get("event")=="diagnostic_start"),{})
        if start.get("executable_sha256")!=report["binary_sha256"]:errors.append([name,"binary identity changed"])
        if not any(r.get("event")=="diagnostic_end" for r in rows):errors.append([name,"logger did not close"])
        if max((r.get("dropped_records",0) for r in rows),default=0):errors.append([name,"diagnostic records lost"])
        sequences=[r["input_seq"] for r in rows if r.get("event")=="navigation_input"]
        if any(a>=b for a,b in zip(sequences,sequences[1:])):errors.append([name,"input sequence repeated or reversed"])
        frames=[r for r in rows if r.get("event")=="nav_frame"]
        keys=[(r["frame_id"],r["view"]) for r in frames]
        if len(keys)!=len(set(keys)):errors.append([name,"multiple frame records for one pane/pass token"])
        for r in frames:
            if r.get("phase") in ("dragging","settling") and r.get("zoom_factor") is not None and abs(r["zoom_factor"]-run["dpi"])>1e-6:errors.append([name,"requested DPI simulation was not applied"])
            if r.get("phase")=="dragging" and r.get("owner_view")!=r["view"]:errors.append([name,"dragging pane does not own capture"])
            if r.get("published_frame_id")==r["frame_id"] and r.get("final_complete") and r.get("displayed_revision")!=r["revision"]:errors.append([name,"published final camera is stale"])
            if r.get("quality") not in (0,1,2,3):errors.append([name,"invalid preview profile"])
            for field in ("interval_ms","work_ms","gpu_ms","refinement_ms"):
                value=r.get(field)
                if value is not None and (not math.isfinite(value) or value<0):errors.append([name,f"invalid {field}"])
        dt=run.get("geometry_submit_interval_ms",run["application_frame_interval_ms"]);hz=run["fixed_hz"]
        for metric,limit in [("p95",18.5 if hz==60 else 35.),("p99",25. if hz==60 else 40.)]:
            if hz is not None and dt[metric] is not None and dt[metric]>limit:app_target_failures.append([name,"geometry_submit_interval",metric,dt[metric],limit])
        latency=run["receive_to_submit_ms"]["p95"]
        if hz is not None and latency is not None and latency>(20. if hz==60 else 37.):app_target_failures.append([name,"receive_to_submit","p95",latency,20. if hz==60 else 37.])
        refine=run["refinement_ms"]["p95"]
        if run["count"] in (125,729) and refine is not None and refine>250:app_target_failures.append([name,"warm_refinement","p95",refine,250.])
        details.append({"name":name,"navigation_events":len(sequences),"frame_records":len(frames),"source_fingerprint":start.get("source_fingerprint"),"final_snapshots":sum(bool(r.get("final_complete") and r.get("published_frame_id")==r["frame_id"]) for r in frames)})
        measurements.append({key:run.get(key) for key in ("name","count","views","dpi","fixed_hz","frame_latency","repeat","active_frames","active_app_records","qualities","hz",*metrics)})
    groups=[]
    for count,hz,views,latency in sorted({(r["count"],r["fixed_hz"],r["views"],r["frame_latency"]) for r in report["runs"]},key=lambda row:(row[0],row[1] or 0,row[2],row[3])):
        runs=[r for r in measurements if (r["count"],r["fixed_hz"],r["views"],r["frame_latency"])==(count,hz,views,latency)]
        groups.append({"count":count,"fixed_hz":hz,"views":views,"frame_latency":latency,"runs":len(runs),"scope":"maximum of per-run percentiles; not pooled percentiles",**{metric:{f"max_per_run_{stat}":max((r[metric][stat] for r in runs if r.get(metric) and r[metric][stat] is not None),default=None) for stat in ("p95","p99","max")} for metric in metrics}})
    return {"schema_version":2,"summary":summary_path.relative_to(root).as_posix(),"binary_sha256":report["binary_sha256"],"runs":len(report["runs"]),"integrity_errors":errors,"application_target_failures":app_target_failures,"runs_without_application_target_failures":len(seen-{r[0] for r in app_target_failures}),"source_fingerprints":sorted({r["source_fingerprint"] for r in details if r["source_fingerprint"]}),"presentation_acceptance":"NOT_PROVED","physical_dpi_acceptance":"PENDING","local_rdp_acceptance":"PENDING","manual_gate_promoted":False,"details":details,"measurements":measurements,"groups":groups}


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument("summary",type=Path);parser.add_argument("--output",type=Path,required=True);args=parser.parse_args();root=Path(__file__).resolve().parents[2]
    result=check(args.summary.resolve(),root);args.output.write_text(json.dumps(result,indent=2)+"\n",encoding="utf-8")
    print(json.dumps({k:result[k] for k in ("runs","source_fingerprints","presentation_acceptance")}));print("integrity_errors",len(result["integrity_errors"]),"application_target_failures",len(result["application_target_failures"]))
    raise SystemExit(bool(result["integrity_errors"]))


if __name__=="__main__":main()
