import collections
import hashlib
import json
import math
import os
import statistics
import sys
from datetime import datetime, timezone, timedelta
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
if len(sys.argv) > 1:
    ROOT = Path(sys.argv[1]).resolve()
SOURCE = ROOT / 'build/stress-27000/nav-pointer-125.jsonl'
OUT = ROOT / 'implementation/evidence/M01/2026-10-21-navigation-forensics-review.json'
TZ = timezone(timedelta(hours=7))

def stamp(ns):
    return datetime.fromtimestamp(ns / 1e9, TZ).isoformat()

def distribution(values):
    values = sorted(values)
    if not values:
        return {'n': 0}
    def quantile(q):
        index = (len(values) - 1) * q
        lo = math.floor(index)
        hi = math.ceil(index)
        return values[lo] + (values[hi] - values[lo]) * (index - lo)
    return {'n': len(values), 'min': values[0], 'median': statistics.median(values),
            'p95_linear': quantile(.95),
            'p95_floor_rank': values[max(0, math.floor(len(values)*.95)-1)],
            'p99_linear': quantile(.99), 'max': values[-1]}

def metrics(rows):
    result = {}
    for key in ['dt_ms', 'frame_ms', 'gap_ms', 'prepare_ms', 'draw_ms', 'wheel', 'radius_mm']:
        result[key] = distribution([r[key] for r in rows if isinstance(r.get(key), (int, float))])
    result['dt_exact_target'] = sum(abs(r['dt_ms'] - r['target_ms']) < 1e-6 for r in rows)
    result['pace_late_true'] = sum(r['pace_late'] for r in rows)
    return result

excluded = {'.git', 'QymCAD', 'ref', 'node_modules', 'target', '.venv', '__pycache__'}
files = []
for base, dirs, names in os.walk(ROOT):
    dirs[:] = [d for d in dirs if d not in excluded]
    for name in names:
        if name.endswith('.jsonl'):
            p = Path(base) / name
            s = p.stat()
            files.append({'path': p.relative_to(ROOT).as_posix(), 'bytes': s.st_size,
                          'mtime_ns': s.st_mtime_ns, 'modified_at': stamp(s.st_mtime_ns)})
files.sort(key=lambda x: x['mtime_ns'], reverse=True)
raw = SOURCE.read_bytes()
rows = []
errors = []
for line, value in enumerate(raw.decode('utf-8').splitlines(), 1):
    try:
        r = json.loads(value)
        r['_line'] = line
        rows.append(r)
    except Exception as error:
        errors.append({'line': line, 'error': str(error)})
camera = [dict(r) for r in rows if r.get('event') == 'camera']
for index, row in enumerate(camera):
    row['radius_mm'] = math.dist(row['eye'], row['target'])
    if index:
        row['gap_ms'] = row['t_ms'] - camera[index - 1]['t_ms']
paired = camera[1:]
short = [r for r in paired if r['gap_ms'] <= 100]
outliers = [r for r in short if r['dt_ms'] > 40]
big = [r for r in paired if r['gap_ms'] > 100]
result = {
    'schema_version': 1,
    'scope': 'Existing-log analysis and source review only; no new application/runtime benchmark',
    'recorded_at': datetime.now(TZ).isoformat(),
    'input': {'path': SOURCE.relative_to(ROOT).as_posix(), 'sha256': hashlib.sha256(raw).hexdigest(),
              'bytes': len(raw), 'modified_at': stamp(SOURCE.stat().st_mtime_ns),
              'lines': len(rows), 'parse_errors': errors,
              'event_types': dict(collections.Counter(r.get('event') for r in rows)),
              'first': rows[0], 'last_t_ms': rows[-1]['t_ms']},
    'inventory_scope': {'root': str(ROOT), 'excluded_directory_names': sorted(excluded),
                        'note': 'JSONL inventory in checkout; no claim about external AppData or other drives'},
    'jsonl_files_found': len(files),
    'newest_jsonl': files[:20],
    'all_camera': metrics(camera),
    'by_action': {k: metrics([r for r in camera if r['action'] == k])
                  for k in sorted({r['action'] for r in camera})},
    'by_view_action': {v + ' / ' + a: metrics([r for r in camera if r['view'] == v and r['action'] == a])
                       for v, a in sorted({(r['view'], r['action']) for r in camera})},
    'inter_camera_log_gaps': distribution([r['gap_ms'] for r in paired]),
    'gaps_gt_100_ms': len(big),
    'dt_gap_within_2ms_for_large_gaps': sum(abs(r['dt_ms'] - r['gap_ms']) <= 2 for r in big),
    'gap_le_100_proxy': {'warning': 'Adjacent camera records <=100 ms, NOT confirmed continuous input/frame capture',
                        'metrics': metrics(short), 'outliers_dt_gt_40': outliers},
    'largest_dt': sorted(camera, key=lambda r:r['dt_ms'], reverse=True)[:12],
    'last_rows_by_action_view': {v + ' / ' + a: [r for r in camera if r['view']==v and r['action']==a][-1]
                               for v,a in sorted({(r['view'],r['action']) for r in camera})},
    'counter_values': {k: sorted({r.get(k) for r in camera}, key=lambda v:(v is None,v))
                       for k in ['view','scene_objects','visible','culled','draw_calls','speed','target_ms']},
}
source_paths = [
    'implementation/design/M01/VIEWPORT-NAVIGATION-FORENSICS.md',
    'crates/om9-app/src/ui.rs', 'crates/om9-app/src/frame_pace.rs',
    'crates/om9-app/src/camera_rig.rs', 'crates/om9-app/src/camera_transition.rs',
    'crates/om9-app/src/interaction_log.rs', 'crates/om9-app/src/gpu_viewport.rs',
    'crates/om9-scene/src/camera.rs', 'crates/om9-app/tests/navigation_input.rs',
    'ref/matrix/decompiled-ui/Matrix/Commands/SetCameraTargetCommand.cs',
    'ref/matrix/decompiled-ui/Matrix/Commands/MouseCallBackHandler.cs',
    'ref/matrix/decompiled-ui/Matrix/Commands/Test/TestVRayDisplay.cs',
    'ref/rhino5/dnspy/RhinoCommon/RhinoCommon/Display/RhinoViewport.cs',
    'ref/rhino5/ghidra/rhcommon_c.dll.c', 'ref/rhino5/ghidra/Rhino.exe.c',
    'QymCAD/crates/qymcad-ui-state/src/navigation.rs',
]
result['source_fingerprints'] = []
for name in source_paths:
    p = ROOT / name
    with p.open('rb') as stream:
        digest = hashlib.file_digest(stream, 'sha256').hexdigest()
    result['source_fingerprints'].append({'path': name, 'bytes': p.stat().st_size, 'sha256': digest})
result['analytic_checks'] = {
    'label': 'Independent algebra for current interpolation equations; not an application render test',
    'midpoint_eased_fraction': .5,
    'zoom_factor': .81,
    'cursor_initial_offset_px': 200,
    'linear_pivot_log_distance_midpoint_drift_px': 200 * ((.5 + .5*.81) / math.sqrt(.81) - 1),
    'scale_consistent_pivot_midpoint_drift_px': 0.,
    'wheel_252_points_at_40_points_per_step': 252 / 40,
    'min_drain_frames_at_2_steps': math.ceil((252/40)/2),
    'nominal_drain_time_at_30hz_ms': math.ceil((252/40)/2) * (1000/30),
    'ease_initial_derivative': .1/.25,
}
result['preceding_logs'] = []
for filename in ['nav-hold-125.jsonl', 'nav-rig-125.jsonl']:
    p = SOURCE.parent / filename
    data = p.read_bytes()
    records = [json.loads(line) for line in data.decode('utf-8').splitlines()]
    cams = [r for r in records if r.get('event') == 'camera']
    result['preceding_logs'].append({
        'path': p.relative_to(ROOT).as_posix(), 'sha256': hashlib.sha256(data).hexdigest(),
        'modified_at': stamp(p.stat().st_mtime_ns), 'camera_events': len(cams),
        'by_view_action': {v+' / '+a: {
            'dt_ms': distribution([r['dt_ms'] for r in cams if r['view']==v and r['action']==a]),
            'dt_exact_target': sum(abs(r['dt_ms']-r['target_ms']) < 1e-6 for r in cams
                                   if r['view']==v and r['action']==a)}
            for v,a in sorted({(r['view'],r['action']) for r in cams})},
        'comparability': 'Different captures; no controlled paired A/B or build identity',
    })
OUT.parent.mkdir(parents=True, exist_ok=True)
OUT.write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
print(json.dumps({'output': str(OUT), 'input': result['input'],
                  'jsonl_files_found': len(files), 'newest_jsonl': files[0],
                  'all_dt_ms': result['all_camera']['dt_ms'],
                  'gap_le_100_dt_ms': result['gap_le_100_proxy']['metrics']['dt_ms'],
                  'gaps_gt_100_ms': len(big), 'continuous_proxy_outliers': len(outliers),
                  'analytic_checks': result['analytic_checks']}, ensure_ascii=False, indent=2))
