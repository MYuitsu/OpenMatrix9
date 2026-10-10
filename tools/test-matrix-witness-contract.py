from pathlib import Path
import subprocess
root=Path('H:/FreeCAD-src/build/om9-layer-edits')
source=(root/'bridge/rhino5/OM9LayerTransfer/RhinoLayerAdapter.cs').read_text()
helper=root/'bridge/rhino5/OM9LayerTransfer/NativeLayerWitness.cs'
if not helper.exists():
    # Run the existing rollback logic as the RED baseline, without native pointers.
    helper=root/'docs/validation/layer-session/MatrixWitnessRed.cs'
    start=source.index('foreach(var row in witnesses)')
    end=source.index('\n',start)
    logic=source[start:end]
    logic=logic[logic.index('if(l.GetUserString'):logic.rindex('}')]
    logic=logic.replace('row.Value','value').replace('continue;','return;')
    helper.write_text('using System; namespace OM9LayerTransfer { public static class NativeLayerWitness { static void Require(bool v,string m){if(!v)throw new InvalidOperationException(m);} public static void Restore(Rhino.RhinoDoc doc,Guid id,string value){var l=doc.Layers[doc.Layers.Find(id,true)];'+logic+'} } }')
exe=root/'docs/validation/layer-session/MatrixWitnessContract.exe'
subprocess.run(['rtk','proxy','C:/Windows/Microsoft.NET/Framework64/v4.0.30319/csc.exe','/nologo','/out:'+str(exe),str(root/'tests/MatrixWitnessContract.cs'),str(helper)],check=True)
result=subprocess.run(['rtk','proxy',str(exe)],capture_output=True,text=True)
(exe.with_name('matrix-witness-contract-'+('green' if result.returncode==0 else 'red')+'.log')).write_text(result.stdout+result.stderr)
print(result.stdout+result.stderr);result.check_returncode()
