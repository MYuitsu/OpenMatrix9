import importlib.util,json,shutil
from pathlib import Path

build=Path('H:/FreeCAD-src/build')
target=build/'opennurbs-standalone-reference-acceptance-20261008'
spec=importlib.util.spec_from_file_location('api',build/'om9-dev/tools/verify_rhino5_probe_api.py')
api=importlib.util.module_from_spec(spec);spec.loader.exec_module(api)
metadata=json.loads((target/'rhino5-gui-api-metadata.json').read_text(encoding='utf-8-sig'))
source=(target/'rhino5-standalone-reference-gui-probe.py').read_text(encoding='utf-8')
# The original checker binds obj to File3dmObject and cannot resolve a chained
# instance-method receiver. This script iterates document RhinoObjects.
api.ALIASES['obj']='Rhino.DocObjects.RhinoObject'
api.ALIASES['document_objects']='Rhino.DocObjects.Tables.ObjectTable'
source=source.replace('Rhino.RhinoDoc.ActiveDoc.Objects.GetSelectedObjects','document_objects.GetSelectedObjects')
result=api.verify(source,metadata['types'])
assert result['ok'],result
assert not api.verify(source.replace('obj.Id','obj.NonexistentId'),metadata['types'])['ok']
assert not api.verify(source.replace('GetSelectedObjects(False,False)','GetSelectedObjects(False)'),metadata['types'])['ok']
result.update(scope='GUI RhinoObject alias and ObjectTable instance receiver; API names/arity only, not actual geometry or Rhino execution',assembly_sha256=metadata['assembly_sha256'],negative_controls=2)
(target/'rhino5-api-preflight.json').write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8')
frozen=build/'om9-dev/docs/validation/rhino5-standalone-reference-20261008'
for name in ['rhino5-api-preflight.json','rhino5-gui-api-metadata.json']:
    shutil.copy2(target/name,frozen/name)
shutil.copy2(Path(__file__),frozen/Path(__file__).name)
print(json.dumps(result,indent=2))
