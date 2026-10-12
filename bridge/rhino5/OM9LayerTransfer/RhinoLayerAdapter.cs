// Native RhinoCommon facts/commit only; no independent merge policy.
using System;
using System.IO;
using System.Text;
using System.Drawing;
using System.Collections.Generic;
using System.Web.Script.Serialization;
using Rhino;
using Rhino.Commands;
using Rhino.DocObjects;
using Rhino.FileIO;
using Rhino.Geometry;

namespace OM9LayerTransfer {
public static class RhinoLayerAdapter {
    public static void RetirePreviousDiagnostics() {
        // Remove only our earlier test hooks from already loaded content-addressed
        // adapters. Never load/unload Matrix plugins or touch its native history.
        foreach(var assembly in AppDomain.CurrentDomain.GetAssemblies()) {
            if(assembly==typeof(RhinoLayerAdapter).Assembly||!assembly.GetName().Name.StartsWith("OM9LayerTransfer_",StringComparison.Ordinal))continue;
            var type=assembly.GetType("OM9LayerTransfer.RhinoLayerAdapter",false);if(type==null)continue;
            string[] methods={"CursorIdle","DocumentClosed"};Type[] owners={typeof(RhinoApp),typeof(RhinoDoc)};string[] events={"Idle","CloseDocument"};
            for(int i=0;i<methods.Length;i++) {
                var method=type.GetMethod(methods[i],System.Reflection.BindingFlags.NonPublic|System.Reflection.BindingFlags.Static);
                if(method==null)continue;
                var nativeEvent=owners[i].GetEvent(events[i]);
                nativeEvent.RemoveEventHandler(null,Delegate.CreateDelegate(nativeEvent.EventHandlerType,method));
            }
        }
    }
    static readonly JavaScriptSerializer Json=new JavaScriptSerializer {MaxJsonLength=256*1024*1024,RecursionLimit=256};
    public static byte[] Encode(object value){return Encoding.UTF8.GetBytes(Json.Serialize(value));}
    static Dictionary<string,object> Map(object value){return (Dictionary<string,object>)value;}
    static object[] Array(object value){return (object[])value;}
    static string Text(object value){return value==null?null:Convert.ToString(value);}
    static bool Flag(object value){return Convert.ToBoolean(value);}
    static Color Rgb(object value){var a=Array(value);return Color.FromArgb(Convert.ToInt32(a[0]),Convert.ToInt32(a[1]),Convert.ToInt32(a[2]));}
    static void Require(bool v,string message){if(!v)throw new InvalidOperationException(message);}
    public static IEnumerable<RhinoObject> Models(RhinoDoc doc) {
        return doc.Objects.GetObjectList(new ObjectEnumeratorSettings {NormalObjects=true,LockedObjects=true,HiddenObjects=true,
            ActiveObjects=true,ReferenceObjects=true,IncludeLights=true,IncludeGrips=true,IdefObjects=false});
    }
    public static byte[] Snapshot(RhinoDoc doc,bool selected) {
        var layers=new List<object>();var objects=new List<object>();
        for(int i=0;i<doc.Layers.Count;i++) {
            var l=doc.Layers[i];if(l==null||l.IsDeleted)continue;bool child=l.ParentLayerId!=Guid.Empty;
            // Persistent getters expose desired state beneath a constrained parent.
            bool locked=child?l.GetPersistentLocking():l.IsLocked,visible=child?l.GetPersistentVisibility():l.IsVisible;
            var persistent=Map(Json.DeserializeObject(Encoding.UTF8.GetString(NativeLayerApi.Persistent(Encode(new {child=child,locked=locked,visible=visible,witness=l.GetUserString("OpenMatrix9.LayerPersistent.v1")??""}),0))));
            layers.Add(new {source_id=l.Id.ToString(),parent_id=child?l.ParentLayerId.ToString():null,name=l.Name,
                path_components=l.FullPath.Split(new string[]{"::"},StringSplitOptions.None),rgb=new int[]{l.Color.R,l.Color.G,l.Color.B},
                locked=locked,visible=visible,persistent_locked=persistent["persistent_locked"],persistent_visible=persistent["persistent_visible"]});
        }
        foreach(var o in Models(doc)) {
            if(selected&&o.IsSelected(false)==0)continue;
            Require(!o.IsReference&&!o.IsInstanceDefinitionGeometry,"Reference/definition objects require retained-archive transfer, not a silent skip");
            var a=o.Attributes;Require(a.ColorSource==ObjectColorSource.ColorFromLayer||a.ColorSource==ObjectColorSource.ColorFromObject,"Unsupported object color source");
            objects.Add(new {id=o.Id.ToString(),layer_id=doc.Layers[a.LayerIndex].Id.ToString(),locked=a.Mode==ObjectMode.Locked||a.GetUserString("OpenMatrix9.Locked")=="1",
                visible=a.Visible,color_source=a.ColorSource==ObjectColorSource.ColorFromObject?"ByObject":"ByLayer",rgb=new int[]{a.ObjectColor.R,a.ObjectColor.G,a.ObjectColor.B}});
        }
        return NativeLayerApi.Validate(Encode(new {version=1,snapshot=new {document_id="Matrix:"+doc.DocumentId,generation="0",layers=layers,objects=objects,
            active_layer=doc.Layers[doc.Layers.CurrentLayerIndex].Id.ToString()}}));
    }
    static Guid ArchiveAdd(File3dm file,GeometryBase g,ObjectAttributes a) {
        if(g is Rhino.Geometry.Point)return file.Objects.AddPoint(((Rhino.Geometry.Point)g).Location,a);
        if(g is Curve)return file.Objects.AddCurve((Curve)g,a);
        if(g is Brep)return file.Objects.AddBrep((Brep)g,a);
        if(g is Mesh)return file.Objects.AddMesh((Mesh)g,a);
        if(g is PointCloud)return file.Objects.AddPointCloud((PointCloud)g,a);
        if(g is Surface)return file.Objects.AddSurface((Surface)g,a);
        if(g is Extrusion)return file.Objects.AddExtrusion((Extrusion)g,a);
        throw new InvalidOperationException("Native archive conversion unavailable: "+g.GetType().FullName);
    }
    public static byte[][] Serialize(RhinoDoc doc,bool session) {
        byte[] snapshot=Snapshot(doc,!session);string path=Path.Combine(Path.GetTempPath(),"om9-matrix-"+Guid.NewGuid().ToString("N")+".3dm");
        try {using(var file=new File3dm()) {
            file.Settings.ModelUnitSystem=doc.ModelUnitSystem;file.Settings.ModelAbsoluteTolerance=doc.ModelAbsoluteTolerance;
            var indices=new Dictionary<int,int>();
            for(int i=0;i<doc.Layers.Count;i++){var l=doc.Layers[i];if(l==null||l.IsDeleted)continue;
                file.Layers.Add(l);indices.Add(i,file.Layers.Count-1);}
            foreach(var o in Models(doc)) {if(!session&&o.IsSelected(false)==0)continue;
                using(var attrs=o.Attributes.Duplicate()) {attrs.LayerIndex=indices[attrs.LayerIndex];
                    Require(attrs.SetUserString("OpenMatrix9.LayerObjectId",o.Id.ToString()),"Cannot write object provenance");
                    Require(ArchiveAdd(file,o.Geometry,attrs)!=Guid.Empty,"Native archive object write failed");}}
            file.Polish();Require(file.Write(path,5),"Native V5 archive write failed");
        }
        var info=new FileInfo(path);Require(info.Length<=512L*1024*1024,"Geometry byte budget exceeded");
        byte[] geometry=File.ReadAllBytes(path);NativeLayerApi.CheckCurrent(snapshot,Snapshot(doc,!session));
        return new byte[][]{geometry,NativeLayerApi.Prepare(snapshot,session?2U:1U,geometry)};
        } finally {if(File.Exists(path))File.Delete(path);}
    }
    static Dictionary<string,Guid> Existing(RhinoDoc doc) {
        var result=new Dictionary<string,Guid>();for(int i=0;i<doc.Layers.Count;i++){var l=doc.Layers[i];if(l!=null&&!l.IsDeleted)result.Add(l.Id.ToString(),l.Id);}return result;
    }
    static void Palette(RhinoDoc doc,Dictionary<string,object> state,Dictionary<string,Guid> ids,List<Guid> added) {
        var rows=new List<Dictionary<string,object>>();foreach(var value in Array(state["layers"]))rows.Add(Map(value));
        rows.Sort(delegate(Dictionary<string,object> a,Dictionary<string,object> b){return Array(a["path_components"]).Length.CompareTo(Array(b["path_components"]).Length);});
        // Resolve native IDs and create missing layers before native state projection.
        foreach(var row in rows){string id=Text(row["source_id"]);if(ids.ContainsKey(id))continue;
            string parent=Text(row["parent_id"]);var layer=new Layer {Name=Text(row["name"]),ParentLayerId=parent==null?Guid.Empty:ids[parent]};
            int index=doc.Layers.Add(layer);Require(index>=0,"Cannot add mapped layer");ids.Add(id,doc.Layers[index].Id);added.Add(doc.Layers[index].Id);}
        // Release parent constraints first. Never try to show a child under a hidden parent.
        foreach(var row in rows){var l=doc.Layers[doc.Layers.Find(ids[Text(row["source_id"])],true)];
            l.IsLocked=false;l.IsVisible=true;Require(l.CommitChanges(),"Cannot stage layer availability");}
        string active=Text(state["active_layer"]);int current=doc.Layers.Find(ids[active],true);
        // Source active has already been validated/fallback-selected by Rust.
        var work=doc.Layers[current];work.IsLocked=false;work.IsVisible=true;Require(work.CommitChanges(),"Cannot prepare current layer");
        Require(doc.Layers.SetCurrentLayerIndex(current,true),"Cannot select mapped current layer");
        rows.Reverse(); // desired child state must be written while ancestors are still available
        foreach(var row in rows){var l=doc.Layers[doc.Layers.Find(ids[Text(row["source_id"])],true)];
            l.Name=Text(row["name"]);l.Color=Rgb(row["rgb"]);l.IsLocked=Flag(row["locked"]);l.IsVisible=Flag(row["visible"]);
            if(row["persistent_locked"]==null)l.UnsetPersistentLocking();else l.SetPersistentLocking(Flag(row["persistent_locked"]));
            if(row["persistent_visible"]==null)l.UnsetPersistentVisibility();else l.SetPersistentVisibility(Flag(row["persistent_visible"]));
            Require(l.SetUserString("OpenMatrix9.LayerPersistent.v1",Encoding.UTF8.GetString(NativeLayerApi.Persistent(Encode(row),1))),"Cannot write native persistent presence witness");
            Require(l.CommitChanges(),"Cannot commit mapped layer state");}
    }
    public static void Receive(RhinoDoc doc,byte[] geometry,byte[] metadata,string fault) {
        Require(Command.InCommand(),"Receive requires the current native command context");
        NativeLayerApi.Capture(metadata,geometry); // reject unknown/truncated/mismatched metadata before native parsing
        byte[] before=Snapshot(doc,false);string path=Path.Combine(Path.GetTempPath(),"om9-receive-"+Guid.NewGuid().ToString("N")+".3dm");
        var addedObjects=new List<Guid>();var addedLayers=new List<Guid>();
        var witnesses=new Dictionary<Guid,string>();foreach(var l in doc.Layers){if(l!=null&&!l.IsDeleted)witnesses.Add(l.Id,l.GetUserString("OpenMatrix9.LayerPersistent.v1"));}
        try {Require(geometry.Length<=512*1024*1024,"Geometry byte budget exceeded");File.WriteAllBytes(path,geometry);
            using(var file=File3dm.Read(path)) {Require(file!=null,"Cannot stage native archive");
                var bindings=new List<object>();foreach(var o in file.Objects){Require(o.Geometry!=null&&o.Geometry.IsValid,"Invalid staged native geometry");
                    bindings.Add(new {physical_id=o.Attributes.ObjectId.ToString(),source_id=o.Attributes.GetUserString("OpenMatrix9.LayerObjectId")});}
                var plan=Map(Json.DeserializeObject(Encoding.UTF8.GetString(NativeLayerApi.Receive(metadata,geometry,before,Encode(bindings)))));
                NativeLayerApi.CheckCurrent(before,Snapshot(doc,false));var after=Map(plan["after"]);var ids=Existing(doc);
                try {
                    Palette(doc,after,ids,addedLayers);if(fault=="after-layer")throw new InvalidOperationException("Injected layer failure");
                    var mapped=Map(plan["object_mapping"]);var rows=new Dictionary<string,Dictionary<string,object>>();
                    foreach(var row in Array(after["objects"])){var a=Map(row);rows.Add(Text(a["id"]),a);}
                    foreach(var o in file.Objects) {var row=rows[Text(mapped[o.Attributes.ObjectId.ToString()])];
                        using(var a=o.Attributes.Duplicate()){a.LayerIndex=doc.Layers.Find(ids[Text(row["layer_id"])],true);
                            a.ColorSource=Text(row["color_source"])=="ByObject"?ObjectColorSource.ColorFromObject:ObjectColorSource.ColorFromLayer;
                            a.ObjectColor=Rgb(row["rgb"]);a.Mode=Flag(row["locked"])?ObjectMode.Locked:ObjectMode.Normal;a.Visible=Flag(row["visible"]);
                            a.SetUserString("OpenMatrix9.Locked",Flag(row["locked"])?"1":"0");
                            Guid id=doc.Objects.Add(o.Geometry,a);Require(id!=Guid.Empty,"Native object add failed");addedObjects.Add(id);}
                        if(fault=="after-geometry")throw new InvalidOperationException("Injected geometry failure");}

                } catch {
                    foreach(var l in doc.Layers){if(l==null||l.IsDeleted)continue;l.IsLocked=false;l.IsVisible=true;Require(l.CommitChanges(),"Rollback layer release failed");}
                    foreach(Guid id in addedObjects){doc.Objects.Unlock(id,true);doc.Objects.Show(id,true);Require(doc.Objects.Delete(id,true),"Receive rollback object delete failed");}
                    var original=Map(Map(Json.DeserializeObject(Encoding.UTF8.GetString(before)))["snapshot"]);
                    Palette(doc,original,ids,new List<Guid>());
                    addedLayers.Reverse();foreach(Guid id in addedLayers){int i=doc.Layers.Find(id,true);var l=doc.Layers[i];l.IsLocked=false;l.IsVisible=true;Require(l.CommitChanges(),"Rollback layer release failed");Require(doc.Layers.Delete(i,true),"Rollback layer delete failed");}
                    foreach(var row in witnesses)NativeLayerWitness.Restore(doc,row.Key,row.Value);
                    NativeLayerApi.CheckCurrent(before,Snapshot(doc,false));throw;
                }
            }
        } finally {if(File.Exists(path))File.Delete(path);doc.Views.Redraw();}
    }
}
}
