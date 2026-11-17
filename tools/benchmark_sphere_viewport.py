"""Run the opt-in real-window OM9 sphere/Boolean audit; no desktop privileges needed.

Only disposable audit processes are launched. Python supplies Windows process/environment
orchestration and analysis; CAD, UI replay, timing markers and geometry stay in Rust.
"""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import subprocess


def percentile(values, q):
    values = sorted(values)
    return values[round((len(values) - 1) * q)]


def gap_summary(frames):
    return {"max_ms":max((r["data"]["interval_ms"] for r in frames),default=0),
        **{f"over_{threshold}_ms":sum(r["data"]["interval_ms"]>threshold for r in frames) for threshold in [50,100,200,500,1000]},
        "gaps_over_100_ms":[{"elapsed_ms":r["elapsed_ms"],"stage":r["stage"],"interval_ms":r["data"]["interval_ms"]}
                            for r in frames if r["data"]["interval_ms"]>100]}


def analyze(path):
    rows = [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines()]
    assert not any(r.get("event") == "sphere_audit_failure" for r in rows), "audit reported failure"
    assert any(r.get("event") == "sphere_audit_end" for r in rows), "incomplete audit"
    assert max(r.get("dropped_records", 0) for r in rows) == 0, "diagnostic records dropped"
    assert any(r.get("event")=="gpu_configuration" for r in rows), "native GPU was not configured"
    start = next(r["data"] for r in rows if r.get("event") == "sphere_audit_start")
    if start.get("viewport_only"):
        return analyze_viewport(rows,path,start)
    architecture = start.get("architecture", "separated")
    pairs = start.get("pairs", 1)
    many = pairs > 1
    result = next(r["data"] for r in rows if r.get("event") == "sphere_audit_result")
    assert result["objects"] == 1 and len(result["bodies"]) == 1, result
    body = result["bodies"][0]
    expected = pairs * 1125 * math.pi / 4
    assert abs(body["volume_mm3"] - expected) / expected < 1e-6, body
    assert body["closed"] and body["solids"] == pairs and body["naked_edges"] == 0, body
    columns = math.ceil(math.sqrt(pairs))
    grid_rows = math.ceil(pairs / columns)
    bounds = [-5-(columns-1)*10, -5-(grid_rows-1)*8, -5,
              10+(columns-1)*10, 5+(grid_rows-1)*8, 5] if many else [-5,-5,-5,10,5,5]
    assert all(abs(a-b)<1e-5 for a,b in zip(body["bounds_mm"],bounds)),body
    inputs = next(r["data"] for r in rows if r.get("event") == "sphere_audit_inputs")
    assert inputs["objects"] == pairs * 2, inputs
    if "architecture" in start:
        witnesses = inputs["bodies"] + [body]
        assert all(b["coupled_display"] == (architecture == "coupled") for b in witnesses), "architecture path was not exercised"
        if architecture == "coupled":
            assert all(b["coupled_validations"] > 0 for b in witnesses), "coupled validation path was not exercised"
    summary = {"path": str(path), "pairs": pairs, "commit_baseline": start.get("commit_baseline"),
               "architecture":architecture,"source_sha256":body.get("source_sha256"),
               "coupled_validations":body.get("coupled_validations",0),
               "gpu_configuration": next(r for r in rows if r.get("event")=="gpu_configuration"),
               "presentation_policy": next((r for r in rows if r.get("event")=="presentation_policy"),None),
               "input_triangles": sum(b["triangles"] for b in inputs["bodies"]),
               "result_triangles": body["triangles"], "volume_mm3": body["volume_mm3"], "stages": {}}
    labels = [(0,"warmup"),(1,"creation" if many else "one_sphere"),(2,"scene" if many else "two_spheres"),(3,"boolean"),(4,"result"),(6,"selected_scene")]
    # Attribute every interval by its starting stage, including the interval
    # spanning a command. Thus a slow command cannot disappear at a boundary.
    frames = [r for r in rows if r.get("event") == "sphere_audit_frame"]
    for stage, label in labels:
        values = [b["data"]["interval_ms"] for a,b in zip(frames,frames[1:]) if a["stage"]==stage]
        if not values:
            continue
        summary["stages"][label] = {"intervals":len(values), "fps_app":1000*len(values)/sum(values),
            "p50_ms":percentile(values,.5),"p95_ms":percentile(values,.95),"p99_ms":percentile(values,.99),
            "max_ms":max(values),"over_50_ms":sum(v>50 for v in values),
            "over_100_ms":sum(v>100 for v in values),"over_200_ms":sum(v>200 for v in values)}
    markers = {r["event"]:r for r in rows if r.get("event") in ["boolean_command_start","boolean_preview_ready","boolean_commit_end"]}
    returned=next((r for r in rows if r.get("event")=="boolean_request_return"),None)
    if returned:
        summary["request_ms"]=returned["elapsed_ms"]-markers["boolean_command_start"]["elapsed_ms"]
        summary["request_capture"]=returned["data"]["capture"]
        assert summary["request_capture"][1]==pairs*2
        next_frame=next(r for r in frames if r["elapsed_ms"]>returned["elapsed_ms"])
        summary["request_transition_ms"]=next_frame["data"]["interval_ms"]
    selection=[r for r in rows if r.get("event")=="selection_frame_rebuild"]
    if selection:
        summary["selection_frame_rebuilds"]=selection
        assert len({r["baseline"] for r in selection})==1
        summary["selection_baseline"]=selection[0]["baseline"]
        if returned:
            selection_start=next((r for r in rows if r.get("event")=="sphere_selection_start"),None)
            low=selection_start["t_ms"] if selection_start else returned["t_ms"]
            high=next((r["t_ms"] for r in frames if r["t_ms"]>low),next_frame["t_ms"])
            transition=[r for r in selection if low<=r["t_ms"]<=high]
            assert len({r["view"] for r in transition})==4,"four-pane selection rebuild missing"
            summary["selection_transition_ms"]=sum(r["elapsed_ms"] for r in transition)
            summary["selection_transition_points"]=sum(r["points"] for r in transition)
    summary["boolean_total_ms"] = markers["boolean_commit_end"]["elapsed_ms"]-markers["boolean_command_start"]["elapsed_ms"]
    for event,key in [("boolean_temporary_ready","temporary_ready_ms"),("boolean_temporary_presented","temporary_presented_ms")]:
        row=next((r for r in rows if r.get("event")==event),None)
        summary[key]=None if row is None else row["elapsed_ms"]-markers["boolean_command_start"]["elapsed_ms"]
        if row is not None: summary[key+"_receipt"]=row["data"]
    if start.get("temporary_preview_expected"):
        assert summary["temporary_ready_ms"] is not None,"temporary mesh was not produced"
        assert summary["temporary_presented_ms"] is not None,"temporary mesh was not encoded by the GPU"
        assert summary["temporary_presented_ms"]<summary["boolean_total_ms"],"preview did not precede exact publication"
    if many:
        completed = [r for r in rows if r.get("event") == "boolean_worker_complete"]
        assert len(completed) == 1 and completed[0]["ok"], completed
        summary["boolean_commit_ms"] = completed[0]["final_poll_ms"]
        assert summary["boolean_commit_ms"] is not None, "final poll timing missing"
    else:
        summary["boolean_preview_ms"] = markers["boolean_preview_ready"]["elapsed_ms"]-markers["boolean_command_start"]["elapsed_ms"]
        summary["boolean_commit_ms"] = markers["boolean_commit_end"]["elapsed_ms"]-markers["boolean_preview_ready"]["elapsed_ms"]
    summary["sphere_ms"] = [b["elapsed_ms"]-a["elapsed_ms"] for a,b in zip(rows,rows[1:])
        if a.get("event")=="sphere_command_start" and b.get("event")=="sphere_command_end"]
    phase=0
    submitted={}
    published={}
    seen=set()
    seen_published=set()
    for row in rows:
        if row.get("event")=="sphere_audit_frame":phase=row["stage"]
        if row.get("event")=="nav_frame" and row.get("view")=="Perspective":
            frame=row.get("published_frame_id")
            if frame is not None and frame not in seen_published:
                seen_published.add(frame);published.setdefault(phase,[]).append(row["t_ms"])
        if row.get("event")=="nav_frame" and row.get("view")=="Perspective" and row.get("render_due"):
            frame=row.get("encoded_frame_id")
            if frame is not None and frame == row.get("frame_id") and frame not in seen:
                seen.add(frame);submitted.setdefault(phase,[]).append(row["t_ms"])
    for phase,label in labels:
        stamps=submitted.get(phase,[])
        if len(stamps)>1 and stamps[-1]>stamps[0]:
            summary["stages"][label]["fps_encoded_perspective"]=1000*(len(stamps)-1)/(stamps[-1]-stamps[0])
    # Keep pure camera motion separate from command-crossing intervals. Freeze
    # accounting below still includes every creation/Boolean transition.
    summary["steady"] = {}
    for stage, label in [(2, "scene"), (3, "pending_boolean"), (4, "result"), (6,"selected_scene")]:
        phase_frames = [r for r in frames if r["stage"] == stage]
        if len(phase_frames) < 2: continue
        low, high = phase_frames[0]["t_ms"] + 1000, phase_frames[-1]["t_ms"] - 250
        gaps = [b["data"]["interval_ms"] for a,b in zip(frames,frames[1:])
                if a["stage"]==stage and b["stage"]==stage and low<=a["t_ms"] and b["t_ms"]<=high]
        stamps = [t for t in submitted.get(stage,[]) if low <= t <= high]
        if gaps and len(stamps)>1:
            encoded_gaps=[stamps[0]-low,*[b-a for a,b in zip(stamps,stamps[1:])],high-stamps[-1]]
            summary["steady"][label] = {"fps_encoded":1000*len(stamps)/(high-low),
                "fps_between_encoded_frames":1000*(len(stamps)-1)/(stamps[-1]-stamps[0]),
                "window_ms":high-low,"fresh_encoded_frames":len(stamps),
                "first_frame_lag_ms":stamps[0]-low,"last_frame_lag_ms":high-stamps[-1],
                "p95_ms":percentile(gaps,.95), "max_ms":max(gaps), "over_100_ms":sum(v>100 for v in gaps),
                "encoded_p50_ms":percentile(encoded_gaps,.5),"encoded_p95_ms":percentile(encoded_gaps,.95),
                "encoded_max_ms":max(encoded_gaps),"encoded_over_100_ms":sum(v>100 for v in encoded_gaps)}
            presented=[t for t in published.get(stage,[]) if low<=t<=high]
            published_gaps=([presented[0]-low,*[b-a for a,b in zip(presented,presented[1:])],high-presented[-1]]
                            if presented else [high-low])
            summary["steady"][label].update(fps_published=1000*len(presented)/(high-low),
                fresh_published_frames=len(presented),published_max_ms=max(published_gaps),
                published_over_100_ms=sum(v>100 for v in published_gaps))
    if many and start.get("temporary_preview_expected"):
        expected_phases=[(2,"scene",pairs*2),(3,"pending_boolean",1),(4,"result",1)]
        if any(r.get("event")=="sphere_selection_start" for r in rows):
            expected_phases.append((6,"selected_scene",pairs*2))
        for stage,label,expected_objects in expected_phases:
            motion=summary["steady"].get(label)
            assert motion is not None,f"no fresh GPU motion frames in {label}"
            # Edge holds are measured freezes, not grounds for dropping the run.
            # The fixed-window denominator above includes their time even when
            # no frame was encoded, instead of inflating FPS between two frames.
            phase_frames=[r for r in frames if r["stage"]==stage]
            low,high=phase_frames[0]["t_ms"]+1000,phase_frames[-1]["t_ms"]-250
            assert any(r.get("event")=="nav_frame" and r.get("view")=="Perspective"
                and low<=r["t_ms"]<=high and r.get("encoded_frame_id")==r.get("frame_id")
                and r.get("scene_objects")==expected_objects and (r.get("visible") or 0)>0
                and (r.get("draw_calls") or 0)>0 for r in rows),f"missing rendered geometry in {label}"
    first_command = next(r["elapsed_ms"] for r in rows if r.get("event")=="sphere_command_start")
    # Keep active command/navigation gaps separate from final settling/idle:
    # a deliberate idle repaint wait is not an application freeze.
    # Native geometry verification happens after the last recorded frame.
    gaps = [r for r in frames if r["elapsed_ms"]>=first_command]
    summary["freeze"] = gap_summary(gaps)  # all callbacks, retained for older reports
    summary["active_freeze"] = gap_summary([r for r in gaps if r["stage"]<=4 or r["stage"]==6])
    summary["boolean_gaps"] = gap_summary([r for r in gaps if r["stage"]==3])
    summary["settling_gaps"] = gap_summary([r for r in gaps if r["stage"]==5])
    phase=0
    gpu={}
    targets=[]
    for row in rows:
        if row.get("event")=="sphere_audit_frame":phase=row["stage"]
        if phase in [2,4] and row.get("event")=="nav_frame" and row.get("view")=="Perspective" and row.get("phase")=="dragging":
            if isinstance(row.get("gpu_ms"),(float,int)) and row.get("gpu_frame_id",-10)>=(row.get("frame_id",0)-4):
                gpu.setdefault(phase,[]).append(row["gpu_ms"])
        if phase==4 and row.get("event")=="viewport_counters":
            view=row.get("views",{}).get("Perspective")
            if view:targets.append([view["target_width"],view["target_height"],view["target_samples"]])
    if gpu.get(4):summary["moving_gpu_p50_ms"]=percentile(gpu[4],.5)
    if gpu.get(2):summary["scene_gpu_p50_ms"]=percentile(gpu[2],.5)
    if targets:summary["result_motion_targets"]=sorted(set(map(tuple,targets)))
    settled=[r for r in rows if r.get("event")=="sphere_audit_settled"]
    if settled:
        summary["settled_targets"]=settled[-1]["data"]["views"]
        assert all(v and v["target_samples"]==4 for v in summary["settled_targets"].values()),"final MSAA was not restored"
    summary["native_timing"] = {}
    for stage,label in [(2,"scene"),(4,"result")]:
        phase_frames=[r for r in frames if r["stage"]==stage]
        if len(phase_frames)<2: continue
        low,high=phase_frames[0]["t_ms"]+1000,phase_frames[-1]["t_ms"]-250
        timings=[r for r in rows if r.get("event")=="native_frame_timing" and low<=r["t_ms"]<=high]
        if timings:
            summary["native_timing"][label]={key:{"p50_ms":percentile([r[key] for r in timings],.5),
                "p95_ms":percentile([r[key] for r in timings],.95)} for key in
                ["ui_ms","platform_and_tessellation_ms","paint_and_wait_ms","interval_ms"]}
    resources=[r for r in rows if r.get("event")=="sphere_audit_resources"]
    if resources:
        summary["resources"]=resources
        assert all(not r["data"]["undo_disabled"] for r in resources),"Undo was disabled"
        assert all(r["data"]["process"] for r in resources),"process RAM sample unavailable"
        before=next(r["data"] for r in resources if r["data"]["point"]=="before_boolean")
        after=next(r["data"] for r in resources if r["data"]["point"]=="after_boolean")
        end=next(r["data"] for r in resources if r["data"]["point"]=="performance_end")
        assert before["objects"]==pairs*2 and after["objects"]==1
        if start.get("temporary_preview_expected"):
            assert before["undo_steps"]==pairs*2 and after["undo_steps"]==pairs*2+1,"retained Undo count changed"
            assert after["redo_steps"]==0
        summary["resource_summary"]={"before_boolean":before,"after_boolean":after,"performance_end":end,
            "after_verification":next(r["data"] for r in resources if r["data"]["point"]=="after_verification")}
    return summary


def analyze_viewport(rows,path,start):
    assert not any(r.get('event') in ['boolean_command_start','boolean_worker_complete'] for r in rows),'Boolean must not run'
    count=start['pairs']*2
    created=0;phase=0;progress={};fresh={};seen={};frames={};creation=[];last_shown=0;max_lag=0
    for r in rows:
        event=r.get('event')
        if event=='sphere_command_end':created=r['data']['index'];creation.append(r)
        if event=='sphere_audit_frame':phase=r['stage'];frames.setdefault(phase,[]).append(r)
        if event=='viewport_publication' and r.get('frame_id') is not None:
            view=r['view'];frame=r['frame_id']
            if frame in seen.setdefault(view,set()):continue
            seen[view].add(frame)
            if view=='Perspective':fresh.setdefault(phase,[]).append(r['t_ms'])
            objects=r.get('objects') or 0
            if objects>0 and (r.get('visible') or 0)>0:
                progress.setdefault(view,[]).append({'created':created,'shown':objects,'t_ms':r['t_ms']})
                if view=='Perspective':last_shown=objects
        if event=='sphere_audit_frame' and phase==1:max_lag=max(max_lag,created-last_shown)
    assert created==count
    presented=[r['data'] for r in rows if r.get('event')=='sphere_viewport_presented']
    assert [r['objects'] for r in presented]==list(range(1,count+1)),'each sphere must be presented before the next command'
    assert len(progress)==4,'four viewports must publish geometry'
    for view,observations in progress.items():
        during=[p for p in observations if 0<p['created']<count]
        assert during,f'{view}: no nonempty published image during creation'
        assert len({p['shown'] for p in during})>=2,f'{view}: published geometry did not progress'
    result=next(r['data'] for r in rows if r.get('event')=='sphere_audit_result')
    assert result['objects']==count and len(result['bodies'])==count
    for body in result['bodies']:
        assert body['closed'] and body['solids']==1 and body['naked_edges']==0
        assert abs(body['volume_mm3']-4*math.pi*125/3)<1e-6,'exact sphere volume changed'
    expected=sorted(tuple(r['data']['center_mm']) for r in rows if r.get('event')=='sphere_command_start' and 'center_mm' in r['data'])
    measured=[]
    for body in result['bodies']:
        m=body['placement'];c=[(body['bounds_mm'][i]+body['bounds_mm'][i+3])/2 for i in range(3)]
        assert all(abs(m[i][j]-(1 if i==j else 0))<1e-9 for i in range(3) for j in range(3))
        measured.append(tuple(sum(m[i][j]*c[j] for j in range(3))+m[i][3] for i in range(3)))
    assert len(expected)==count and all(abs(a-b)<1e-6 for x,y in zip(expected,sorted(measured)) for a,b in zip(x,y)),'source placement changed'
    resources=[r['data'] for r in rows if r.get('event')=='sphere_audit_resources']
    end=next(r for r in resources if r['point']=='performance_end')
    assert end['objects']==count and end['undo_steps']==count and end['redo_steps']==0 and not end['undo_disabled']
    assert any(r.get('event')=='sphere_viewport_undo_redo' for r in rows)
    steady={}
    for stage,label in [(2,'scene'),(6,'selected_scene')]:
        pf=frames[stage];lo=pf[0]['t_ms']+1000;hi=pf[-1]['t_ms']-250
        stamps=[t for t in fresh.get(stage,[]) if lo<=t<=hi]
        assert len(stamps)>1,f'{label}: no GPU publication during navigation'
        gaps=[stamps[0]-lo,*[b-a for a,b in zip(stamps,stamps[1:])],hi-stamps[-1]]
        steady[label]={'fps_published':len(stamps)*1000/(hi-lo),'max_gpu_hold_ms':max(gaps),
            'ui_max_ms':max(r['data']['interval_ms'] or 0 for r in pf if lo<=r['t_ms']<=hi)}
    shot=next((r['data'] for r in rows if r.get('event')=='sphere_viewport_screenshot'),None)
    return {'path':str(path),'viewport_only':True,'objects':count,'source_sha256':result['bodies'][0]['source_sha256'],
        'per_sphere_presentation':presented,'commit_to_present_max_ms':max(r['commit_to_present_ms'] for r in presented),
        'source_fingerprints':sorted(b['source_sha256'] for b in result['bodies']),
        'steady':steady,'creation_progress':progress,'max_creation_lag_objects':max_lag,
        'ui_creation_max_ms':max(r['data']['interval_ms'] or 0 for r in frames[1]),
        'ram_peak_mib':end['process']['peak_working_set_bytes']/2**20,'ram_settled_mib':end['process']['working_set_bytes']/2**20,
        'undo_steps':end['undo_steps'],'resources':resources,'screenshot':shot,
        'input_triangles':sum(b['triangles'] for b in result['bodies'])}


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--exe", type=Path, default=Path("L:/om9-target/release/om9.exe"))
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--analyze-only", action="store_true")
    parser.add_argument("--motion-comparison", action="store_true", help="compare baseline Light with optimized pointer-event handling and transient render targets")
    parser.add_argument("--pairs", type=int, default=1, help="independent intersecting pairs; >1 builds all spheres then runs BooleanUnion")
    parser.add_argument("--architecture-comparison", action="store_true", help="same-binary document-coupled versus independent viewport representations")
    parser.add_argument("--request-comparison", action="store_true", help="same-binary former serial versus batched Boolean source capture")
    parser.add_argument("--selection-comparison", action="store_true", help="same-binary per-point versus prepared-camera selection frame projection")
    parser.add_argument("--profile-request", action="store_true", help="diagnostic cumulative request-phase timings to stderr")
    parser.add_argument("--resources", action="store_true", help="sample process RAM and retained Undo/Redo without changing retention policy")
    parser.add_argument("--viewport-only",action="store_true",help="create and navigate spheres with full Undo; never invoke Boolean")
    parser.add_argument("--timeout-seconds", type=int, help="opt-in audit deadline, 30..14400 seconds; default unchanged")
    parser.add_argument("--profiles", nargs="+", choices=["high","light"], default=["high","light"], help="profiles for architecture comparison")
    parser.add_argument("--repeats", type=int, choices=[1,2,3], default=3, help="paired Shaded repetitions for a many-sphere scene")
    parser.add_argument("--profile-commit", action="store_true", help="emit diagnostic phase timing to each run stderr")
    parser.add_argument("--commit-baseline", action="store_true", help="retain the former UI wire preparation for a same-binary counterfactual")
    parser.add_argument("--present-mode", choices=["auto-vsync","auto-no-vsync"], help="same-binary presentation override; default uses the detected session")
    args=parser.parse_args()
    if not 1 <= args.pairs <= 512: parser.error("--pairs must be 1..512")
    if args.timeout_seconds is not None and not 30<=args.timeout_seconds<=14400: parser.error("--timeout-seconds must be 30..14400")
    if args.resources and args.pairs==1: parser.error("--resources requires --pairs >1")
    if args.pairs > 1 and args.motion_comparison: parser.error("many-sphere comparison uses product Light and High")
    if args.architecture_comparison and (args.pairs == 1 or args.motion_comparison or args.commit_baseline):
        parser.error("architecture comparison requires --pairs >1 and retains product motion/commit paths")
    if args.request_comparison and (args.pairs==1 or args.architecture_comparison or args.motion_comparison or args.commit_baseline):
        parser.error("request comparison requires --pairs >1 with production architecture/motion/commit")
    if args.selection_comparison and (args.pairs==1 or args.request_comparison or args.architecture_comparison or args.motion_comparison or args.commit_baseline):
        parser.error("selection comparison requires --pairs >1 with production request/architecture/motion/commit")
    if not args.analyze_only:
        args.output.mkdir(parents=True, exist_ok=False)
        executable=args.exe.resolve(strict=True)
        fingerprint=hashlib.sha256(executable.read_bytes()).hexdigest()
        source_paths=["crates/om9-app/src/ui/sphere_boolean_audit.rs","crates/om9-app/src/ui.rs",
                      "crates/om9-app/src/display.rs","crates/om9-app/src/gpu_viewport.rs",
                      "crates/om9-app/src/session.rs","crates/om9-app/src/session/solid.rs",
                      "crates/om9-app/src/session/topology_remesh.rs","crates/om9-app/src/session/plugins.rs",
                      "crates/om9-scene/src/scene.rs",
                      "crates/om9-native-owner/src/lib.rs","crates/om9-app/src/frame_scheduler.rs",
                      "crates/om9-core/src/brep.rs","crates/om9-core/src/transaction.rs","crates/om9-scene/src/scene/admission.rs",
                      "crates/om9-app/src/session/boolean_union_worker.rs","crates/om9-scene/src/camera.rs",
                      "crates/om9-core/src/transaction/undo_memory.rs","crates/om9-native-owner/src/system_memory.rs",
                      "crates/om9-app/src/ui/undo_settings.rs","tools/benchmark_sphere_viewport.py",
                      "crates/om9-app/src/session/boolean_union_worker/mesh_preview.rs",
                      "crates/om9-kernel/src/editing.rs","crates/om9-kernel/src/editing_bridge.cpp",
                      "crates/om9-kernel/src/transform_bridge.cpp","crates/om9-kernel/src/brep_snapshot.hxx",
                       "crates/om9-kernel/src/brep/intermediate.rs","crates/om9-kernel/src/brep.rs",
                       "crates/om9-render/src/lib.rs","crates/om9-render/src/scene.rs","crates/om9-render/src/normals.rs",
                       "crates/om9-core/src/transaction/retained_projection_cache.rs",
                       "crates/om9-core/src/transaction/cache_pressure.rs",
                       "crates/om9-core/src/transaction/memory_diagnostics.rs"]
        receipt={"exe":str(executable),"sha256":fingerprint,"pairs":args.pairs,"repeats":args.repeats,
                 "profile_commit":args.profile_commit,"profile_request":args.profile_request,
                 "viewport_only":args.viewport_only,
                 "commit_baseline":args.commit_baseline,
                 "motion_comparison":args.motion_comparison,"frame_hz":30,
                 "architecture_comparison":args.architecture_comparison,"profiles":args.profiles,
                 "request_comparison":args.request_comparison,
                 "selection_comparison":args.selection_comparison,
                 "resources":args.resources,"timeout_seconds":args.timeout_seconds,
                 "profile_commit":args.profile_commit,"profile_request":args.profile_request,
                 "present_mode_override":args.present_mode,"wgpu_backend_override":os.environ.get("WGPU_BACKEND"),
                 "frame_latency_override":os.environ.get("OM9_FRAME_LATENCY"),
                 "source_sha256":{p:hashlib.sha256(Path(p).read_bytes()).hexdigest() for p in source_paths}}
        (args.output/"binary.json").write_text(json.dumps(receipt,indent=2),encoding="utf-8")
        runs=([(p,"shaded") for p in ["baseline","motion","motion","baseline","baseline","motion"]]
              if args.motion_comparison else [(p,"shaded") for p in ["high","light","light","high","high","light"]]+[(p,"wireframe") for p in ["high","light"]])
        if args.pairs > 1:
            runs=[(p,"shaded") for repeat in range(args.repeats)
                  for p in (args.profiles if repeat%2==0 else reversed(args.profiles))]
        if args.architecture_comparison:
            runs=[(profile,"shaded",architecture) for repeat in range(args.repeats)
                  for offset,profile in enumerate(args.profiles)
                  for architecture in (["coupled","separated"] if (repeat+offset)%2==0 else ["separated","coupled"])]
        if args.request_comparison:
            runs=[(profile,"shaded","separated",capture) for repeat in range(args.repeats)
                  for offset,profile in enumerate(args.profiles)
                  for capture in (["serial","batch"] if (repeat+offset)%2==0 else ["batch","serial"])]
        if args.selection_comparison:
            runs=[(profile,"shaded","separated","batch",selection) for repeat in range(args.repeats)
                  for offset,profile in enumerate(args.profiles)
                  for selection in (["baseline","prepared"] if (repeat+offset)%2==0 else ["prepared","baseline"])]
        for index,run in enumerate(runs,1):
            profile,display=run[:2]
            architecture=run[2] if len(run)>2 else "separated"
            suffix=f"-{architecture}" if args.architecture_comparison else ""
            capture=run[3] if len(run)>3 else "batch"
            if args.request_comparison: suffix=f"-request-{capture}"
            selection=run[4] if len(run)>4 else "prepared"
            if args.selection_comparison: suffix=f"-selection-{selection}"
            path=(args.output/f"{index:02d}-{profile}{suffix}-{display}.jsonl").resolve()
            env=os.environ.copy()
            env.pop("OM9_SPHERE_VIEWPORT_AUDIT",None)
            if args.viewport_only:env['OM9_SPHERE_VIEWPORT_AUDIT']='1'
            if args.viewport_only:env['OM9_VIEWPORT_SCREENSHOT_PATH']=str(path.with_suffix('.png'))
            env.pop("OM9_SPHERE_BOOLEAN_AUDIT_RESOURCES",None)
            if args.resources:env["OM9_SPHERE_BOOLEAN_AUDIT_RESOURCES"]="1"
            env.pop("OM9_SPHERE_BOOLEAN_AUDIT_TIMEOUT_SECS",None)
            if args.timeout_seconds is not None:env["OM9_SPHERE_BOOLEAN_AUDIT_TIMEOUT_SECS"]=str(args.timeout_seconds)
            env.pop("OM9_SELECTION_FRAME_BASELINE",None)
            if selection=="baseline": env["OM9_SELECTION_FRAME_BASELINE"]="1"
            env["OM9_SPHERE_BOOLEAN_AUDIT_ARCHITECTURE"]=architecture
            env.pop("OM9_BOOLEAN_REQUEST_BASELINE",None)
            if capture=="serial": env["OM9_BOOLEAN_REQUEST_BASELINE"]="1"
            env.pop("OM9_BOOLEAN_REQUEST_PROFILE",None)
            if args.profile_request: env["OM9_BOOLEAN_REQUEST_PROFILE"]="1"
            env.pop("OM9_PRESENT_MODE",None)
            if args.present_mode: env["OM9_PRESENT_MODE"]=args.present_mode
            env.pop("OM9_COMMIT_PROFILE",None)
            if args.profile_commit: env["OM9_COMMIT_PROFILE"]="1"
            env.pop("OM9_BOOLEAN_COMMIT_BASELINE",None)
            if args.commit_baseline: env["OM9_BOOLEAN_COMMIT_BASELINE"]="1"
            for key in ["OM9_CAMERA_AUDIT","OM9_NAVIGATION_AUDIT","OM9_GPU_STRESS","OM9_MESH_STRESS","OM9_LIGHT_MOTION_BASELINE","OM9_SPHERE_BOOLEAN_AUDIT_POINTER","OM9_SPHERE_BOOLEAN_AUDIT_PAIRS"]:env.pop(key,None)
            if args.motion_comparison:
                env["OM9_SPHERE_BOOLEAN_AUDIT_POINTER"]="1"
                if profile=="baseline":env["OM9_LIGHT_MOTION_BASELINE"]="1"
            if args.pairs > 1:
                env["OM9_SPHERE_BOOLEAN_AUDIT_POINTER"]="1"
                env["OM9_SPHERE_BOOLEAN_AUDIT_PAIRS"]=str(args.pairs)
            env.update(OM9_SPHERE_BOOLEAN_AUDIT="light" if args.motion_comparison else profile,OM9_INTERACTION_LOG=str(path),OM9_STARTUP_DIALOG="0",
                OM9_FRAME_HZ="30",OM9_SPHERE_BOOLEAN_AUDIT_DISPLAY=display,
                OM9_SPHERE_BOOLEAN_AUDIT_ORIGIN="cmd" if args.motion_comparison or args.pairs>1 else ("menu" if index%2==0 else "cmd"))
            env["PATH"]="L:/om9-target/debug;"+env.get("PATH","")
            print(f"running {path.name}",flush=True)
            with path.with_suffix(".stderr.txt").open("w",encoding="utf-8") as stderr:
                proc=subprocess.Popen([str(executable)],env=env,stderr=stderr)
                try:
                    code=proc.wait(timeout=args.timeout_seconds+60 if args.timeout_seconds is not None else (620 if args.pairs>1 else 100))
                except subprocess.TimeoutExpired:
                    proc.terminate();proc.wait(timeout=10)
                    raise RuntimeError(f"audit timed out: {path}")
            assert code==0,(path,code)
            analyze(path)
    results=[dict(run=path.stem,**analyze(path)) for path in sorted(args.output.glob("*.jsonl"))]
    if not results:raise RuntimeError("no audit traces")
    if args.viewport_only:
        (args.output/'summary.json').write_text(json.dumps(results,indent=2),encoding='utf-8')
        print(json.dumps([{k:r[k] for k in ['objects','steady','ram_peak_mib','ram_settled_mib','undo_steps','max_creation_lag_objects']} for r in results],indent=2))
        return
    if args.architecture_comparison:
        for profile in args.profiles:
            paired=[r for r in results if f"-{profile}-" in r["run"]]
            assert len(paired)==2*args.repeats
            assert len({(r["input_triangles"],r["result_triangles"]) for r in paired})==1, "architecture changed tessellation"
            assert len({tuple(r["source_sha256"]) for r in paired})==1, "architecture changed exact result source bytes"
            assert {r["architecture"] for r in paired}=={"coupled","separated"}
    if args.request_comparison:
        for profile in args.profiles:
            paired=[r for r in results if f"-{profile}-" in r["run"]]
            assert len(paired)==2*args.repeats
            assert len({(r["input_triangles"],r["result_triangles"]) for r in paired})==1
            assert len({tuple(r["source_sha256"]) for r in paired})==1,"capture changed exact result source bytes"
            assert all(r["request_capture"][0]==("-request-batch-" in r["run"]) for r in paired),"request capture path not exercised"
    if args.selection_comparison:
        for profile in args.profiles:
            paired=[r for r in results if f"-{profile}-" in r["run"]]
            assert len(paired)==2*args.repeats
            assert len({(r["input_triangles"],r["result_triangles"],r["selection_transition_points"]) for r in paired})==1,"selection changed geometry coverage"
            assert len({tuple(r["source_sha256"]) for r in paired})==1,"selection changed exact source"
            assert all(r["selection_baseline"]==("-selection-baseline-" in r["run"]) for r in paired)
            assert all(r["request_capture"]==[True,args.pairs*2] for r in paired),"request batch must stay enabled"
    if args.motion_comparison:
        assert len({(r["input_triangles"],r["result_triangles"]) for r in results})==1,"motion comparison changed display geometry"
        for r in results:
            targets=r.get("result_motion_targets",[])
            assert targets,"pointer navigation did not exercise render targets"
            if "-motion-" in r["run"]:assert any(t[2]==1 for t in targets),"Light motion tier was not exercised"
            else:assert all(t[2]==4 for t in targets),"baseline render tier changed"
    high=[r for r in results if "-high-" in r["run"]]
    light=[r for r in results if "-light-" in r["run"]]
    if high and light:
        assert len({tuple(r["source_sha256"]) for r in results})==1,"display profile changed canonical Boolean source"
        assert max(r["input_triangles"] for r in light)<min(r["input_triangles"] for r in high)
        assert max(r["result_triangles"] for r in light)<min(r["result_triangles"] for r in high)
        assert max(r["volume_mm3"] for r in results)-min(r["volume_mm3"] for r in results)<1e-6*min(r["volume_mm3"] for r in results)
    (args.output/"summary.json").write_text(json.dumps(results,indent=2),encoding="utf-8")
    print(json.dumps([{k:r[k] for k in ["run","input_triangles","result_triangles","boolean_total_ms","boolean_commit_ms"]}
        | {"result_fps_app":r["stages"]["result"]["fps_app"],
           "result_fps_encoded":r["steady"].get("result",{}).get("fps_encoded"),
           "max_active_gap_ms":r["active_freeze"]["max_ms"],"active_gaps_over_100_ms":r["active_freeze"]["over_100_ms"]} for r in results],indent=2))


if __name__=="__main__":main()
