import json, os, sys, time
sys.path.insert(0, os.path.abspath('tools'))
from om9_mcp_client import Client
c=Client(r'L:\om9-target\debug\om9.exe')
def call(label, fn, *args, **kw):
    t=time.perf_counter(); r=fn(*args,**kw); elapsed=time.perf_counter()-t
    print(json.dumps({'step':label,'seconds':round(elapsed,4),'error':r.get('is_error',False),'result':r.get('data',{}).get('result')}),flush=True)
    if r.get('is_error') or not r.get('data',{}).get('result',{}).get('ok',True): raise RuntimeError(r)
    return r
def cmd(s): return call('cmd '+repr(s),c.command,s)
def pt(p): return call('point',c.point,p)
try:
    call('new',c.new,'timing')
    cmd('Box'); pt([0,0,0]); pt([20,20,0]); cmd('20')
    for i in range(4):
        body=call('state',c.state,100)['data']['objects'][0]['id']
        cmd('Sphere'); pt([4+4*i,10,20]); cmd('1.5')
        objs=call('state',c.state,100)['data']['objects']; other=next(o['id'] for o in objs if o['id']!=body)
        call('select',c.select,ids=[body,other]); cmd('Boolean2Objects'); cmd('DeleteInput=Yes'); cmd('Difference'); cmd('')
finally: c.close()
