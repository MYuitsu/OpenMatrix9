"""Exercise actual before-image code against a constrained native child boundary."""
from pathlib import Path
import subprocess
root=Path('H:/FreeCAD-src/build/om9-layer-edits')
source=(root/'tests/Rhino5LayerCommandProbe.cs').read_text(encoding='utf-8-sig')
start=source.index('    sealed class BeforeLayer {')
end=source.index('    internal static Result Apply(',start)
actual=source[start:end]
folder=root/'docs/validation/layer-session/rollback-boundary'
folder.mkdir(parents=True,exist_ok=True)
fixture='''using System;
using System.Drawing;
class Layer {
 internal Color Color;
 internal bool IsLocked=true;
 bool visible=false;
 internal bool IsVisible {get{return visible;} set{if(value)throw new InvalidOperationException("Constrained child cannot be turned visible while parent is hidden");visible=value;}}
 internal bool desiredLock=false,desiredVisible=true;
 internal bool GetPersistentLocking(){return desiredLock;}
 internal bool GetPersistentVisibility(){return desiredVisible;}
 internal void SetPersistentLocking(bool value){desiredLock=value;}
 internal void SetPersistentVisibility(bool value){desiredVisible=value;}
}
class RhinoDoc {internal Layer[] Layers;}
class RollbackTests {
 static void Commit(Layer layer){}
'''+actual+'''
 static int Main(){try{
  var child=new Layer{Color=Color.FromArgb(71,72,73)};
  var doc=new RhinoDoc{Layers=new Layer[]{child}};
  var before=new BeforeLayer(doc,0);
  child.Color=Color.FromArgb(211,221,231);
  before.Restore(doc);
  if(!child.IsLocked||child.IsVisible||child.GetPersistentLocking()||!child.GetPersistentVisibility()||child.Color.R!=71)throw new Exception("Native and desired before-states not restored separately");
  Console.WriteLine("Actual before-image rollback boundary PASS: constrained native flags and desired own flags restored separately.");return 0;
 }catch(Exception e){Console.Error.WriteLine(e.Message);return 1;}}
}
'''
path=folder/'RollbackTests.cs';path.write_text(fixture,encoding='utf-8')
compiler='C:/Windows/Microsoft.NET/Framework64/v4.0.30319/csc.exe'
binary=folder/'RollbackTests.exe'
subprocess.run(['rtk','proxy',compiler,'/nologo','/reference:System.Drawing.dll','/out:'+str(binary),str(path)],check=True)
result=subprocess.run(['rtk','proxy',str(binary)],capture_output=True,text=True)
print(result.stdout+result.stderr,end='')
raise SystemExit(result.returncode)
