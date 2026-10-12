using System;
using Rhino;
using Rhino.DocObjects;
namespace OM9LayerTransfer {
public static class NativeLayerWitness {
    const string Key="OpenMatrix9.LayerPersistent.v1";
    public static void Restore(RhinoDoc doc,Guid id,string value) {
        int index=doc.Layers.Find(id,true);
        if(index<0)throw new InvalidOperationException("Persistent witness layer missing: "+id);
        var live=doc.Layers[index];
        if(String.Equals(live.GetUserString(Key),value,StringComparison.Ordinal))return;
        // Rhino 5 SetUserString uses ConstPointer and does not create a writable
        // layer copy. Edit a detached duplicate; Modify owns the native Undo change.
        using(var settings=doc.Layers[index]) {
            settings.EnsurePrivateCopy();
            if(!settings.SetUserString(Key,value))throw new InvalidOperationException("Persistent witness write failed: "+id);
            if(!doc.Layers.Modify(settings,index,true))throw new InvalidOperationException("Persistent witness table commit failed: "+id);
        }
        if(!String.Equals(doc.Layers[index].GetUserString(Key),value,StringComparison.Ordinal))
            throw new InvalidOperationException("Persistent witness reread differs: "+id);
    }
}
}
