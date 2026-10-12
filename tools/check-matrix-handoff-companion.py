"""Real OM9 service smoke only; never represent it as native Matrix acceptance."""
from pathlib import Path
import datetime,hashlib,json,subprocess,time,uuid,sys
root=Path('H:/FreeCAD-src/build/om9-layer-edits')
sys.path.insert(0,str(root/'tests'))
import matrix_om9_handoff_support as support
attach=sys.argv[1] if len(sys.argv)>1 else None
run_id=attach or uuid.uuid4().hex
assert len(run_id)==32 and all(c in '0123456789abcdef' for c in run_id)
output=Path('H:/FreeCAD-src/build/matrix-om9-handoff')/run_id
if not attach:output.mkdir(parents=True)
plugin=Path('C:/Users/nguye/.codex/worktrees/layer-session-handoff/FreeCAD-src/build/om9-plugin-runtime/bin/OpenMatrix9Gui.pyd')
host=Path('H:/FreeCAD-src/build/relWithDebInfo/bin/FreeCAD.exe')
digest=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
config=dict(version=1,run_id=run_id,output=str(output),plugin=str(plugin),plugin_sha256=digest(plugin),host_sha256=digest(host))
if attach:
 assert support.read_json(str(output/'session.json'))==config
else:
 support.write_json(str(output/'session.json'),config)
 # Persistent GUI child can retain inherited pipe handles; write directly to
 # an owned log and wait for the service-ready file instead of pipe EOF.
 with (output/'smoke-launcher.log').open('w',encoding='utf-8') as stream:
  launch=subprocess.Popen(['rtk','proxy','pwsh','-NoProfile','-File',str(root/'tests/launch_matrix_om9_handoff.ps1'),'-RunId',run_id],stdout=stream,stderr=stream)
def wait(path):
 deadline=time.monotonic()+120
 while not path.is_file():
  if time.monotonic()>deadline:raise RuntimeError('Companion smoke timeout; owned output '+str(output))
  time.sleep(.1)
 return support.read_json(str(path))
ready=wait(output/'companion-ready.json')
if not ready.get('ok'):raise RuntimeError(ready.get('error','Companion startup failed'))
seq=0;checks=[];responses={}
def request(action):
 global seq
 seq+=1;support.write_json(str(output/('request-%03d.json'%seq)),dict(version=1,run_id=run_id,seq=seq,action=action))
 response=wait(output/('response-%03d.json'%seq))
 assert response['run_id']==run_id and response['seq']==seq and response['pid']==ready['pid']
 responses[str(seq)]=response
 if not response.get('ok'):raise RuntimeError(response.get('error','Owned action failed'))
 return response['result']
def check(name,passed):
 checks.append(dict(name=name,passed=bool(passed)))
 if not passed:raise AssertionError(name)
error=None
try:
 check('actual selected native plugin',ready['plugin_sha256']==config['plugin_sha256'])
 initial=request('snapshot');check('owned target is empty',not initial['semantic']['objects'])
 seeded=request('fixture');check('canonical fixture has three objects and34 layers',len(seeded['semantic']['objects'])==3 and len(seeded['semantic']['layers'])==34)
 copied=request('copy');check('public Copy Session finished',copied['clipboard_result']['ok'])
 pasted=request('paste');check('real public clipboard Paste adds three objects',len(pasted['semantic']['objects'])==6)
 undone=request('undo');check('one actual OM9 Undo restores pre-Paste canonical state and geometry',undone['raw']==seeded['raw'] and undone['semantic']==seeded['semantic'])
 redone=request('redo');check('one actual OM9 Redo restores received canonical state and geometry',redone['raw']==pasted['raw'] and redone['semantic']==pasted['semantic'])
except Exception as e:error=str(e)
finally:
 try:check('owned companion finished',request('finish')['finished'])
 except Exception as e:error=(error or '')+'\nfinish: '+str(e)
 report=dict(scope='Actual OM9 companion pipeline only; Matrix native roundtrip pending',created_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),
  run_id=run_id,output=str(output),ready=ready,checks=checks,error=error,ok=error is None,responses=responses,
  matrix_application_accepted=False,host_sha256=digest(host),plugin_sha256=digest(plugin))
 support.write_json(str(output/'companion-smoke-results.json'),report)
 (root/'docs/validation/layer-session/matrix-handoff-companion-smoke.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
print(('PASS' if not error else 'FAIL')+' OM9 companion smoke; checks='+str(len(checks))+'; '+str(output))
if error:print(error);raise SystemExit(1)
