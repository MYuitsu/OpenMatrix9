import json, os, sys, time
sys.path.insert(0, os.path.abspath('tools'))
from om9_mcp_client import Client

out = os.path.abspath('build/dice-21-undo-optimized-2026-11-02')
os.makedirs(out, exist_ok=True)
if os.path.exists(os.path.join(out, 'dice.om9')):
    raise SystemExit('Output already exists; refusing to overwrite')
started = time.perf_counter()
log = open(os.path.join(out, 'timings.jsonl'), 'w', encoding='utf-8', buffering=1)
def record(event, **values):
    row = {'event': event, 'elapsed_s': round(time.perf_counter() - started, 3), **values}
    print(json.dumps(row), flush=True)
    print(json.dumps(row), file=log, flush=True)
def chk(r):
    if r['is_error'] or ('result' in r['data'] and not r['data']['result'].get('ok', True)):
        raise RuntimeError(str(r))
    return r['data']
c = Client(r'L:\om9-target\debug\om9.exe')
def cmd(s): return chk(c.command(s))
def point(p): return chk(c.point(p))
def objects(): return c.state(100)['data']['objects']
patterns = {1:[(10,10)], 2:[(6,6),(14,14)], 3:[(6,6),(10,10),(14,14)],
            4:[(6,6),(6,14),(14,6),(14,14)],
            5:[(6,6),(6,14),(10,10),(14,6),(14,14)],
            6:[(6,5),(6,10),(6,15),(14,5),(14,10),(14,15)]}
faces = [(1,lambda a,b:[a,b,20]),(6,lambda a,b:[a,b,0]),
         (2,lambda a,b:[a,0,b]),(5,lambda a,b:[a,20,b]),
         (3,lambda a,b:[20,a,b]),(4,lambda a,b:[0,a,b])]
try:
    record('start', binary=r'L:\om9-target\debug\om9.exe')
    chk(c.new('Dice 20mm - 21 recessed pips'))
    cmd('Box'); point([0,0,0]); point([20,20,0]); cmd('20')
    record('box')
    count = 0
    for number, loc in faces:
        face_started = time.perf_counter()
        for a,b in patterns[number]:
            pip_started = time.perf_counter()
            body = objects()[0]['id']
            cmd('Sphere'); point(loc(a,b)); cmd('1.5')
            objs = objects(); cutter = next(o['id'] for o in objs if o['id'] != body)
            chk(c.select(ids=[body,cutter]))
            cmd('Boolean2Objects'); cmd('DeleteInput=Yes'); cmd('Difference'); cmd('')
            state = chk(c.state(100))
            if len(state['objects']) != 1 or state['undo'] != 2 * (count + 1) + 1 or state['benchmark_no_undo']:
                raise RuntimeError('unexpected object count or Undo state after cavity')
            count += 1
            record('cavity', face=number, index=count, seconds=round(time.perf_counter()-pip_started,3))
        record('face', face=number, seconds=round(time.perf_counter()-face_started,3))
    assert count == 21
    save_started = time.perf_counter()
    saved = chk(c.tool('om9_save', path=os.path.join(out,'dice.om9')))
    record('saved', seconds=round(time.perf_counter()-save_started,3), cavities=count, result=saved)
    chk(c.tool('om9_report', path=os.path.join(out,'dice-report.json')))
    record('report_written')
    undo = chk(c.tool('om9_undo'))
    assert undo['state']['undo'] == 42 and undo['state']['redo'] == 1
    redo = chk(c.tool('om9_redo'))
    assert redo['state']['undo'] == 43 and redo['state']['redo'] == 0
    chk(c.tool('om9_report', path=os.path.join(out,'dice-redo-report.json')))
    record('undo_redo_verified')
except Exception as error:
    record('failure', error=str(error))
    raise
finally:
    c.close()
    log.close()
