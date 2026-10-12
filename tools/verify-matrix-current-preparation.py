"""Record preparation checks; never represent offline evidence as Matrix PASS."""
from pathlib import Path
import datetime,hashlib,json,subprocess

root=Path('H:/FreeCAD-src/build/om9-layer-edits')
proof=root/'docs/validation/layer-session'
steps=[
 ('native-current-sdk-build',['pwsh','-NoProfile','-File',str(root/'tools/build-matrix-current-probe.ps1')]),
 ('python-host-boundaries',['H:/FreeCAD-src/.pixi/envs/default/python.exe',str(root/'tests/test_layer_api_direct.py')]),
 ('registry-ownership',['pwsh','-NoProfile','-File',str(root/'tests/test_probe_registry.ps1')]),
 ('installed-ironpython-sdk',['pwsh','-NoProfile','-File',str(root/'tools/check-layer-direct-sdk.ps1')]),
 ('powershell-syntax',['pwsh','-NoProfile','-File',str(root/'tools/check-layer-launcher-syntax.ps1')]),
 ('installed-ironpython-utf8',['pwsh','-NoProfile','-File',str(root/'tools/check-layer-command-launcher.ps1')]),
 ('native-rollback-boundary',['H:/FreeCAD-src/.pixi/envs/default/python.exe',str(root/'tools/check-probe-rollback.py')]),
]
log=proof/'matrix-current-preparation.log'
results=[]
with log.open('w',encoding='utf-8') as stream:
 for name,args in steps:
  result=subprocess.run(['rtk','proxy',*args],capture_output=True,text=True)
  stream.write(name+'\n'+result.stdout+result.stderr+'\n');stream.flush()
  results.append({'name':name,'exit_code':result.returncode})
  print(name+': '+('PASS' if result.returncode==0 else 'FAIL'))
  if result.returncode:raise SystemExit(result.returncode)

files=[
 'tests/Rhino5LayerCommandProbe.cs','tests/rhino5_layer_api_prerequisite.py',
 'tests/rhino5_verify_layer_command_api.py','tests/rhino5_verify_layer_command_api_isolated.py',
 'tests/rhino5_verify_matrix_current.py','tests/register_matrix_current_probe.ps1',
 'tests/rhino5_probe_registry.ps1','tests/test_layer_api_direct.py','tests/test_probe_registry.ps1',
 'tests/bin/current/Rhino5LayerCurrentProbe.dll','tests/bin/current/Rhino5LayerCurrentProbe.rhp',
 'tests/Rhino5CommandLauncherChecks.cs','tools/check-layer-command-launcher.ps1',
 'tools/check-layer-direct-sdk.ps1','tools/check-layer-launcher-syntax.ps1',
 'tools/build-matrix-current-probe.ps1','tools/verify-matrix-current-preparation.py',
]
digest=lambda path:hashlib.sha256(path.read_bytes()).hexdigest()
document={
 'created_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),
 'scope':'Preparation only: current Matrix process, dedicated blank unsaved document',
 'user_choice':'Current blank Matrix document; no additional Rhino application',
 'actual_current_matrix_command_gate_passed':False,'application_accepted':False,
 'native_replay_pending':True,'checks':results,
 'source_and_binary_sha256':{name:digest(root/name) for name in files},
 'log_sha256':digest(log),
 'shared_freecad_host_sha256':digest(Path('H:/FreeCAD-src/build/relWithDebInfo/bin/FreeCAD.exe')),
 'native_plugin_guid':'28a7966b-2c55-4fe4-8cb6-0394316dc4ae',
 'load_method':'Rhino5 _-Options _PlugIns _Load for first use, exact native GUID/command verification',
 'references':['https://discourse.mcneel.com/t/rhinoplugin-rhp-in-rhino-c/46419',
               'https://developer.rhino3d.com/guides/rhinocommon/registering-plugins-windows/'],
 'limits':['Offline tests cannot establish native Matrix command or rollback acceptance.',
           'Only dedicated blank-document test Undo/Redo history is cleared after exact original-state comparison.',
           'Diagnostic assembly remains loaded in current process after its temporary registration is removed.',
           'Old isolated probe binary/process are untouched; full product layer handoff remains incomplete.'],
}
(proof/'matrix-current-preparation.json').write_text(json.dumps(document,indent=2)+'\n',encoding='utf-8')
print('Preparation evidence recorded; actual current Matrix replay required.')
