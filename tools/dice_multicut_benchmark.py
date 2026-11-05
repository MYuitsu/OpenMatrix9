"""MCP dice comparison with normal Undo; use a fresh output directory per run."""
import argparse
import hashlib
import json
import math
import os
import sys
import time
from pathlib import Path

sys.path.insert(0, os.path.abspath('tools'))
from om9_mcp_client import Client

parser = argparse.ArgumentParser()
parser.add_argument('--exe', required=True)
parser.add_argument('--mode', choices=['batch', 'serial'], required=True)
parser.add_argument('--output', required=True)
args = parser.parse_args()
out = Path(args.output).resolve()
out.mkdir(parents=True, exist_ok=False)
start = time.perf_counter()
log = open(out / 'timings.jsonl', 'w', encoding='utf-8', buffering=1)

def record(event, **data):
    row = {'event': event, 'elapsed_s': round(time.perf_counter() - start, 3), **data}
    line = json.dumps(row)
    print(line, flush=True)
    print(line, file=log, flush=True)

def check(reply):
    data = reply['data']
    if reply['is_error'] or not data.get('result', {}).get('ok', True):
        raise RuntimeError(str(reply))
    return data

c = Client(args.exe)
def cmd(text): return check(c.command(text))
def pt(point): return check(c.point(point))
def state(): return check(c.state(100))
def select(ids): return check(c.select(ids=ids))

patterns = {1:[(10,10)], 2:[(6,6),(14,14)], 3:[(6,6),(10,10),(14,14)],
            4:[(6,6),(6,14),(14,6),(14,14)],
            5:[(6,6),(6,14),(10,10),(14,6),(14,14)],
            6:[(6,5),(6,10),(6,15),(14,5),(14,10),(14,15)]}
faces = [(1,lambda a,b:[a,b,20]),(6,lambda a,b:[a,b,0]),
         (2,lambda a,b:[a,0,b]),(5,lambda a,b:[a,20,b]),
         (3,lambda a,b:[20,a,b]),(4,lambda a,b:[0,a,b])]
try:
    check(c.new('Dice 20mm - 21 pips'))
    cmd('Box'); pt([0,0,0]); pt([20,20,0]); cmd('20')
    body = state()['objects'][0]['id']
    cutters = []
    for face, loc in faces:
        for a, b in patterns[face]:
            previous = {o['id'] for o in state()['objects']}
            cmd('Sphere'); pt(loc(a,b)); cmd('1.5')
            cutter = next(o['id'] for o in state()['objects'] if o['id'] not in previous)
            if args.mode == 'serial':
                select([body, cutter]); cmd('Boolean2Objects'); cmd('DeleteInput=Yes')
                cmd('Difference'); cmd('')
                body = state()['objects'][0]['id']
            else:
                cutters.append(cutter)
            record('cavity_prepared', face=face, index=len(cutters) if args.mode == 'batch' else None)
    if args.mode == 'batch':
        before = state()
        assert len(before['objects']) == 22 and len(cutters) == 21
        select([body]); cmd('BooleanDifference'); cmd('DeleteInput=Yes'); cmd('')
        select(cutters)
        cut_start = time.perf_counter(); cmd('')
        record('boolean_committed', seconds=round(time.perf_counter()-cut_start, 3))
    after = state()
    assert len(after['objects']) == 1 and not after['benchmark_no_undo']
    check(c.tool('om9_save', path=str(out / 'dice.om9')))
    record('saved', undo=after['undo'])
    check(c.tool('om9_report', path=str(out / 'report.json')))
    report = json.loads((out / 'report.json').read_text(encoding='utf-8'))
    brep = report['objects'][0]['geometry']['Brep']
    digest = hashlib.sha256(brep['source'].encode()).hexdigest()
    mesh = brep['display']
    points = mesh['points']
    volume6 = 0.0
    for triangle in mesh['triangles']:
        p, q, r = (points[i] for i in triangle[:3])
        volume6 += p[0]*(q[1]*r[2]-q[2]*r[1]) + p[1]*(q[2]*r[0]-q[0]*r[2]) + p[2]*(q[0]*r[1]-q[1]*r[0])
    expected = 8000.0 - 21 * 2 * math.pi * 1.5**3 / 3
    record('geometry', sha256=digest, faces=len(mesh['faces']), mesh_volume=abs(volume6)/6, ideal_volume=expected)
    undo = check(c.tool('om9_undo'))
    undone = state()
    assert len(undone['objects']) == (22 if args.mode == 'batch' else 2)
    assert undone['undo'] == after['undo'] - 1
    if args.mode == 'batch': assert {o['id'] for o in undone['objects']} == {body, *cutters}
    record('undo_verified', objects=len(undone['objects']), undo=undone['undo'])
    check(c.tool('om9_redo'))
    redone = state()
    assert len(redone['objects']) == 1 and redone['objects'][0]['id'] == after['objects'][0]['id']
    assert redone['undo'] == after['undo']
    check(c.tool('om9_report', path=str(out / 'redo-report.json')))
    redo_report = json.loads((out / 'redo-report.json').read_text(encoding='utf-8'))
    assert hashlib.sha256(redo_report['objects'][0]['geometry']['Brep']['source'].encode()).hexdigest() == digest
    record('redo_verified', objects=1, sha256=digest)
finally:
    c.close()
    log.close()
