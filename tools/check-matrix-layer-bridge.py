"""Real managed/native ABI proof; native host commit is a separate user run."""
import json,subprocess
from pathlib import Path
root=Path('H:/FreeCAD-src/build/om9-layer-edits');runtime=root/'bridge/rhino5/runtime';proof=root/'docs/validation/layer-session'
manifest=json.loads((runtime/'manifest.json').read_text())
layers=[dict(source_id='layer-'+str(i),parent_id=None,name='Slot '+str(i),path_components=['Slot '+str(i)],rgb=[i,125,251],locked=i==1,visible=i!=1,persistent_locked=None,persistent_visible=None) for i in range(32)]
layers.append(dict(source_id='child',parent_id='layer-1',name='Own state',path_components=['Slot 1','Own state'],rgb=[31,71,131],locked=False,visible=True,persistent_locked=False,persistent_visible=True))
source=dict(document_id='Matrix',generation='0',layers=layers,objects=[dict(id='ring',layer_id='child',locked=True,visible=False,color_source='ByObject',rgb=[211,19,81])],active_layer='layer-0')
destination=dict(document_id='OM9',generation='9007199254740993',layers=[dict(layers[0],source_id='existing',rgb=[1,2,3])],objects=[],active_layer='existing')
for name,value in [('matrix-abi-source.json',dict(version=1,snapshot=source)),('matrix-abi-destination.json',dict(version=1,snapshot=destination)),('matrix-abi-bindings.json',[dict(physical_id='archive-ring',source_id='ring')])]:
    (proof/name).write_text(json.dumps(value),encoding='utf-8')
exe=runtime/'MatrixLayerAbiProof.exe'
command=['rtk','proxy','C:/Windows/Microsoft.NET/Framework64/v4.0.30319/csc.exe','/nologo','/platform:x64','/reference:System.Web.Extensions.dll','/reference:'+str(runtime/manifest['adapter']),'/out:'+str(exe),str(root/'tests/MatrixLayerAbiProof.cs')]
subprocess.run(command,check=True)
result=subprocess.run(['rtk','proxy',str(exe),str(proof/'matrix-abi-source.json'),str(proof/'matrix-abi-destination.json'),str(proof/'matrix-abi-bindings.json'),str(proof/'matrix-abi-plan.json')],text=True,capture_output=True)
(proof/'matrix-managed-abi.log').write_text(result.stdout+result.stderr,encoding='utf-8');print(result.stdout);result.check_returncode()
plan=json.loads((proof/'matrix-abi-plan.json').read_text())
assert len(plan['after']['layers'])==33 and plan['layer_mapping']['layer-0']=='existing'
assert plan['after']['generation']=='9007199254740994' and plan['after']['objects'][0]['locked'] and not plan['after']['objects'][0]['visible']
assert plan['after']['layers'][0]['rgb']==[0,125,251] and plan['before']['layers'][0]['rgb']==[1,2,3]
print('Managed/native ABI output semantic assertions PASS; native two-app run pending.')
