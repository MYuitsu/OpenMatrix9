import sys,os,json,clr
sys.path.insert(0,r'H:\FreeCAD-src\build\om9-layer-edits\tests')
import matrix_om9_handoff_support as support
clr.AddReference('System.Drawing')
import System
color=System.Drawing.Color.FromArgb(21,71,131)
class Stub(object):
    pass
layer=Stub();layer.Id=System.Guid.NewGuid();layer.FullPath='Metal';layer.Color=color
layer.IsDeleted=False;layer.IsLocked=False;layer.IsVisible=True
layer.GetPersistentLocking=lambda:False;layer.GetPersistentVisibility=lambda:True
class Layers(object):
    Count=1;CurrentLayerIndex=0
    def __getitem__(self,index):return layer
doc=Stub();doc.DocumentId=3;doc.Layers=Layers()
path_source=r'H:\FreeCAD-src\build\om9-layer-edits\tests\rhino5_verify_matrix_om9_handoff.py'
with open(path_source,'r') as stream:source=stream.read()
definition=source[source.index('def snapshot():'):source.index('def clipboard_sequence():')]
namespace={'document':lambda:doc,'objects':lambda doc:[]}
exec(definition,namespace)
value=namespace['snapshot']()
value['pid']=System.Diagnostics.Process.GetCurrentProcess().Id
assert all(type(x)==int for x in value['raw']['layers'][0]['rgb']), 'Snapshot contains CLR Byte instead of JSON integers'
root=r'H:\FreeCAD-src\build\om9-layer-edits\docs\validation\layer-session'
path=os.path.join(root,'ironpython-handoff-json-'+System.Guid.NewGuid().ToString('N')+'.json')
support.write_json(path,value)
assert support.read_json(path)==value
print('Installed IronPython CLR RGB report serialization PASS')
