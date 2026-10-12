// Diagnostic fixture only. No product layer merge or portable policy in C#.
using System;
using System.IO;
using System.Drawing;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Web.Script.Serialization;
using System.Reflection;
using Rhino;
using Rhino.Commands;
using Rhino.DocObjects;
using Rhino.Geometry;

[assembly: System.Reflection.AssemblyTitle(ProbeIdentity.Title)]
[assembly: System.Reflection.AssemblyDescription("Rhino5 native command API diagnostic; not product layer handoff")]
[assembly: System.Reflection.AssemblyVersion("1.0.0.0")]
[assembly: Guid(ProbeIdentity.Plugin)]

static class ProbeIdentity {
#if OM9_CURRENT_SESSION_PROBE
    internal const string Plugin="28a7966b-2c55-4fe4-8cb6-0394316dc4ae",Title="OM9 Matrix Current Session Probe",Prefix="OM9LayerDirectApi";
    internal const string Prepare="84934f3c-8079-4935-ad93-29eb40d9a90d",Apply="23c7d980-04ce-4bde-97d7-5905b7d3dbe1",FailLayer="81e74825-ea49-4c78-8cda-17fb44aad986",FailGeometry="2969b19c-e4f0-46a7-879b-4661c49044bc",Sample="42838e2e-cea6-4738-9722-08ff08aec4dc";
#else
    internal const string Plugin="d2f0f5d0-4e65-4b75-b4ef-8d6ca72be839",Title="OM9 Layer API Command Probe",Prefix="OM9LayerApi";
    internal const string Prepare="569d2757-27f8-4d90-a694-22e46a7ce008",Apply="381fd2c0-28ed-47b5-aa2d-1ee387e3d4bc",FailLayer="e63a8c93-494f-4b79-9da7-8209c5b5987d",FailGeometry="29cfe9f2-b107-4d94-b2b7-d9f15b7d5f94",Sample="b4d9b9f5-b2d7-4b1d-badc-796df5fc2232";
#endif
}
[Guid(ProbeIdentity.Plugin)]
public sealed class OM9LayerCommandProbePlugin : Rhino.PlugIns.PlugIn { }

// Diagnostic-only registration in the owned empty Matrix-loaded host. The
// actual loaded Matrix frontend owns these native commands for this process;
// no PlugInManager interaction, plugin installation or registry mutation.
public static class OM9LayerProbeRegistration {
    public static int Register(Rhino.PlugIns.PlugIn owner) {
        if(owner==null)throw new ArgumentNullException("owner");
        string root=Environment.GetEnvironmentVariable("OM9_LAYER_API_OUTPUT");
        if(String.IsNullOrEmpty(root)||!Directory.Exists(root))throw new InvalidOperationException("Requires owned isolated launcher");
        var rows=new List<object>();
        var pointerMethod=typeof(Rhino.PlugIns.PlugIn).GetMethod("NonConstPointer",BindingFlags.NonPublic|BindingFlags.Instance);
        if(pointerMethod==null)throw new MissingMethodException("Installed diagnostic SDK NonConstPointer");
        // Read-only reflection of the installed SDK is diagnostic evidence only;
        // no private registration method or native pointer mutation is used.
        var pointer=(IntPtr)pointerMethod.Invoke(owner,null);
        Action save=delegate {
            LayerProbe.Write("command-registration",new {owner_id=owner.Id.ToString(),owner_type=owner.GetType().FullName,owner_assembly=owner.GetType().Assembly.FullName,native_owner_pointer=pointer.ToInt64(),commands=rows});
        };
        save();
        if(pointer==IntPtr.Zero)throw new InvalidOperationException("Loaded managed Matrix owner has no native RhinoCommon plugin pointer; see command-registration.json");
        var commands=new Command[]{new OM9LayerApiPrepareFixture(),new OM9LayerApiApplyProbe(),new OM9LayerApiFailLayerProbe(),new OM9LayerApiFailGeometryProbe(),new OM9LayerApiSample()};
        foreach(Command command in commands) {
            if(Command.IsCommand(command.EnglishName))throw new InvalidOperationException("Diagnostic command collision: "+command.EnglishName);
            bool sdkReturn=Rhino.Runtime.HostUtils.RegisterDynamicCommand(owner,command);
            Guid nativeId=Command.LookupCommandId(command.EnglishName,true);
            rows.Add(new {name=command.EnglishName,local_name=command.LocalName,id=command.Id.ToString(),sdk_return=sdkReturn,native_lookup_id=nativeId.ToString(),is_command=Command.IsCommand(command.EnglishName)});
            save();
            // Rhino5's wrapper discards the native create return. A managed
            // true alone is never evidence that a command actually registered.
            if(!sdkReturn||nativeId!=command.Id)throw new InvalidOperationException("Native command lookup does not resolve the registered ID: "+command.EnglishName+"; see command-registration.json");
        }
        return commands.Length;
    }
}

static class LayerProbe {
    internal static int Parent=-1,Child=-1,OwnChild=-1,Work=-1;
    internal static RhinoDoc FixtureDoc;
    static string OriginalState,FixtureRun;
    static Guid OriginalCurrent;
    static bool OriginalModified;
    static readonly List<Guid> FixtureLayers=new List<Guid>();
    static readonly HashSet<Guid> FixtureObjects=new HashSet<Guid>();
    // Private diagnostic cursor, not portable layer policy. The custom Undo
    // callback changes only this private value; native projection happens on
    // a later host Idle after Undo/Redo has ended.
    static Guid Cursor;
    static bool CursorPending;
    static int CursorEvents;
    static string CursorError;
    static bool CursorCreatedByRedo;
    static void OnCursorUndo(object sender,CustomUndoEventArgs e) {
        if(!Object.ReferenceEquals(e.Document,FixtureDoc)||!(e.Tag is Guid)) {
            CursorError="Custom cursor event does not belong to fixture";
        } else {
            Guid inverse=Cursor;
            if(!e.Document.AddCustomUndoEvent("OM9 diagnostic active cursor",OnCursorUndo,inverse))
                CursorError="Cannot register inverse private cursor event";
            Cursor=(Guid)e.Tag;
        }
        CursorCreatedByRedo=e.CreatedByRedo;CursorEvents++;CursorPending=true;
    }
    static void OnCursorIdle(object sender,EventArgs e) {
        if(!CursorPending||Command.InCommand())return;
        if(FixtureDoc!=null&&FixtureDoc.UndoRecordingIsActive)return;
        CursorPending=false;
        var values=new Dictionary<string,object>();
        values["events"]=CursorEvents;values["created_by_redo"]=CursorCreatedByRedo;
        values["desired"]=Cursor.ToString();
        try {
            Require(CursorError==null,CursorError);
            Require(FixtureDoc!=null&&Object.ReferenceEquals(FixtureDoc,RhinoDoc.ActiveDoc),"Cursor fixture document changed");
            Owned(FixtureDoc);
            int index=-1;
            for(int i=0;i<FixtureDoc.Layers.Count;i++) {
                var layer=FixtureDoc.Layers[i];
                if(layer!=null&&!layer.IsDeleted&&layer.Id==Cursor) {index=i;break;}
            }
            Require(index>=0,"Restored cursor layer missing");
            var target=FixtureDoc.Layers[index];
            Require(!target.IsLocked&&target.IsVisible,"Restored cursor layer unavailable");
            values["command_in_progress"]=Command.InCommand();
            values["undo_recording_active"]=FixtureDoc.UndoRecordingIsActive;
            Require(FixtureDoc.Layers.SetCurrentLayerIndex(index,true),"Deferred cursor projection failed");
            values["actual"]=FixtureDoc.Layers[FixtureDoc.Layers.CurrentLayerIndex].Id.ToString();
            values["ok"]=values["actual"].Equals(Cursor.ToString());
            FixtureDoc.Views.Redraw();
        } catch(Exception error) {values["ok"]=false;values["error"]=error.ToString();}
        Write("cursor-projection",values);
    }
    internal static readonly JavaScriptSerializer Json=new JavaScriptSerializer();
    internal static void Require(bool value,string message) { if(!value)throw new InvalidOperationException(message); }
    internal static void Owned(RhinoDoc doc) {
        string root=Environment.GetEnvironmentVariable("OM9_LAYER_API_OUTPUT");
        Require(!String.IsNullOrEmpty(root)&&Directory.Exists(root),"Requires owned isolated launcher");
        Require(FixtureDoc==null||Object.ReferenceEquals(FixtureDoc,doc),"Fixture document changed");
        Require(FixtureDoc==null||FixtureRun==Environment.GetEnvironmentVariable("OM9_LAYER_API_RUN_ID"),"Fixture run changed");
    }
    internal static void Write(string name,object report) {
        var values=Json.Deserialize<Dictionary<string,object>>(Json.Serialize(report));
        values["run_id"]=Environment.GetEnvironmentVariable("OM9_LAYER_API_RUN_ID");
        values["pid"]=System.Diagnostics.Process.GetCurrentProcess().Id;
        File.WriteAllText(Path.Combine(Environment.GetEnvironmentVariable("OM9_LAYER_API_OUTPUT"),name+".json"),Json.Serialize(values));
    }
    internal static ObjectEnumeratorSettings Enumeration() {
        return new ObjectEnumeratorSettings { NormalObjects=true,LockedObjects=true,HiddenObjects=true,ActiveObjects=true,ReferenceObjects=true,IncludeLights=true,IncludeGrips=true };
    }
    internal static string State(RhinoDoc doc) {
        var layers=new List<object>();var objects=new List<object>();
        for(int i=0;i<doc.Layers.Count;i++) {
            Layer l=doc.Layers[i];if(l==null||l.IsDeleted)continue;
            layers.Add(new { id=l.Id.ToString(),name=l.Name,path=l.FullPath,parent=l.ParentLayerId.ToString(),rgb=new int[]{l.Color.R,l.Color.G,l.Color.B},locked=l.IsLocked,visible=l.IsVisible,persistent_locked=l.GetPersistentLocking(),persistent_visible=l.GetPersistentVisibility() });
        }
        var rows=new List<RhinoObject>();foreach(var o in doc.Objects.GetObjectList(Enumeration()))rows.Add(o);
        rows.Sort(delegate(RhinoObject a,RhinoObject b){return a.Id.CompareTo(b.Id);});
        foreach(var o in rows) {
            var a=o.Attributes;BoundingBox b=o.Geometry.GetBoundingBox(true);
            objects.Add(new { id=o.Id.ToString(),layer=doc.Layers[a.LayerIndex].Id.ToString(),mode=(int)a.Mode,visible=a.Visible,color_source=(int)a.ColorSource,rgb=new int[]{a.ObjectColor.R,a.ObjectColor.G,a.ObjectColor.B},bounds=new double[]{b.Min.X,b.Min.Y,b.Min.Z,b.Max.X,b.Max.Y,b.Max.Z} });
        }
        return Json.Serialize(new { active=doc.Layers[doc.Layers.CurrentLayerIndex].Id.ToString(),layers=layers,objects=objects });
    }
    internal static void Commit(Layer layer) { Require(layer.CommitChanges(),"Layer CommitChanges failed"); }
    internal static int AddLayer(RhinoDoc doc,string name,Guid parent,Color color,bool locked) {
        var layer=new Layer { Name=name,ParentLayerId=parent,Color=color,IsLocked=locked,IsVisible=true };
        int index=doc.Layers.Add(layer);Require(index>=0,"Layer insertion failed");FixtureLayers.Add(doc.Layers[index].Id);return index;
    }
    static Guid AddFixturePoint(RhinoDoc doc,Point3d point,ObjectAttributes attributes) {
        Guid id=doc.Objects.AddPoint(point,attributes);Require(id!=Guid.Empty,"Fixture point insertion failed");FixtureObjects.Add(id);return id;
    }
    internal static void ObserveCommand(RhinoDoc doc,Dictionary<string,object> report) {
        report["command_in_progress"]=Command.InCommand();report["undo_recording_active"]=doc.UndoRecordingIsActive;
        uint record=doc.BeginUndoRecord("OM9 diagnostic record observation");
        report["begin_undo_record"]=record;
        // A zero record is the host's command-owned record, never an explicit
        // record owned by this call. Never close a record we didn't create.
        if(record!=0) { Require(doc.EndUndoRecord(record),"Own observation record did not close");report["closed_only_explicit_owned_record"]=true; }
        Require(doc.UndoRecordingIsActive,"Actual command has no active Undo recording");
    }
    internal static void Setup(RhinoDoc doc) {
        Owned(doc);foreach(var obj in doc.Objects.GetObjectList(Enumeration()))throw new InvalidOperationException("Fixture host already contains geometry");
        Require(FixtureDoc==null,"Previous fixture has not been cleaned");
#if OM9_CURRENT_SESSION_PROBE
        Require(!doc.Modified&&String.IsNullOrEmpty(doc.Path),"Requires a fresh blank unsaved Matrix test document");
#endif
        OriginalState=State(doc);OriginalCurrent=doc.Layers[doc.Layers.CurrentLayerIndex].Id;OriginalModified=doc.Modified;
        FixtureRun=Environment.GetEnvironmentVariable("OM9_LAYER_API_RUN_ID");FixtureLayers.Clear();FixtureObjects.Clear();
        FixtureDoc=doc;
        Parent=AddLayer(doc,"OM9 API Parent",Guid.Empty,Color.FromArgb(61,62,63),false);
        Child=AddLayer(doc,"Detail",doc.Layers[Parent].Id,Color.FromArgb(71,72,73),false);
        OwnChild=AddLayer(doc,"Own locked child",doc.Layers[Parent].Id,Color.FromArgb(81,82,83),true);
        Work=AddLayer(doc,"OM9 API Work",Guid.Empty,Color.FromArgb(91,92,93),false);
        Require(doc.Layers.SetCurrentLayerIndex(Parent,true),"Cannot set fixture source active layer");
        Cursor=doc.Layers[Parent].Id;CursorPending=false;CursorEvents=0;CursorError=null;
        RhinoApp.Idle-=OnCursorIdle;RhinoApp.Idle+=OnCursorIdle;
        var child=doc.Layers[Child];child.SetPersistentLocking(false);child.SetPersistentVisibility(true);Commit(child);
        var own=doc.Layers[OwnChild];own.SetPersistentLocking(true);own.SetPersistentVisibility(true);Commit(own);
        AddFixturePoint(doc,new Point3d(1,2,3),new ObjectAttributes { LayerIndex=Child,ColorSource=ObjectColorSource.ColorFromLayer });
        AddFixturePoint(doc,new Point3d(4,5,6),new ObjectAttributes { LayerIndex=Work,Mode=ObjectMode.Locked,ColorSource=ObjectColorSource.ColorFromObject,ObjectColor=Color.FromArgb(13,23,33) });
        doc.Views.Redraw();Write("command-prepare",new { ok=true,state=State(doc) });
    }
    sealed class BeforeLayer {
        internal int Index;internal Color Color;internal bool Locked,Visible,PersistentLocked,PersistentVisible;
        internal BeforeLayer(RhinoDoc doc,int index) {Index=index;var l=doc.Layers[index];Color=l.Color;Locked=l.IsLocked;Visible=l.IsVisible;PersistentLocked=l.GetPersistentLocking();PersistentVisible=l.GetPersistentVisibility();}
        internal void Restore(RhinoDoc doc) {
            var l=doc.Layers[Index];l.Color=Color;l.IsLocked=Locked;l.IsVisible=Visible;
            // A constrained child may be hidden/locked while its desired own
            // flags are visible/unlocked. Restoring desired visibility into
            // IsVisible would attempt to show a child under a hidden parent.
            l.SetPersistentLocking(PersistentLocked);l.SetPersistentVisibility(PersistentVisible);Commit(l);
        }
    }
    internal static Result Apply(RhinoDoc doc,string stage,bool fail,bool addBeforeFailure) {
        var report=new Dictionary<string,object>();var saved=new List<BeforeLayer>();Guid added=Guid.Empty;int addedLayer=-1,oldCurrent=-1;string before=null;
        try {
            Owned(doc);Require(FixtureDoc!=null,"Prepare command not run");
            Write("command-progress",new {stage=stage,step="capture before-image"});
            before=State(doc);oldCurrent=doc.Layers.CurrentLayerIndex;
            saved.Add(new BeforeLayer(doc,Parent));saved.Add(new BeforeLayer(doc,Child));saved.Add(new BeforeLayer(doc,OwnChild));
            ObserveCommand(doc,report);report["before"]=before;
            Require(doc.Layers.SetCurrentLayerIndex(Work,true),"Cannot switch current before restricting source layer");
            var parent=doc.Layers[Parent];parent.Color=Color.FromArgb(111,121,131);parent.IsLocked=true;parent.IsVisible=false;Commit(parent);
            // Change again on failure so the layer-update fault is real even
            // when the successful apply state already locked/hidden the parent.
            if(fail) {parent=doc.Layers[Parent];parent.Color=Color.FromArgb(211,221,231);Commit(parent);}
            report["after_layer_update"]=State(doc);Require((string)report["after_layer_update"]!=before,"No actual layer update observed");
            if(!fail||addBeforeFailure) {
                addedLayer=AddLayer(doc,"OM9 Probe Added "+Guid.NewGuid().ToString("N"),Guid.Empty,Color.FromArgb(141,142,143),false);
                added=AddFixturePoint(doc,new Point3d(100,200,300),new ObjectAttributes { LayerIndex=addedLayer,ColorSource=ObjectColorSource.ColorFromLayer });
                Require(added!=Guid.Empty,"Added geometry insertion failed");report["added_object"]=added.ToString();report["after_geometry_add"]=State(doc);
            }
            if(fail) {
                Write("command-progress",new {stage=stage,step="begin owned before-image rollback"});
                if(added!=Guid.Empty)Require(doc.Objects.Delete(added,true),"Own added geometry rollback failed");
                Require(doc.Layers.SetCurrentLayerIndex(Work,true),"Cannot select work layer for rollback");
                foreach(var row in saved) {
                    Write("command-progress",new {stage=stage,step="restore saved native layer",index=row.Index,locked=row.Locked,visible=row.Visible,persistent_locked=row.PersistentLocked,persistent_visible=row.PersistentVisible});
                    row.Restore(doc);
                }
                if(addedLayer>=0)Require(doc.Layers.Delete(addedLayer,true),"Own added layer rollback failed");
                Require(doc.Layers.SetCurrentLayerIndex(oldCurrent,true),"Old current layer rollback failed");
                doc.Views.Redraw();report["after"]=State(doc);report["rollback_equal"]=before==(string)report["after"];report["ok"]=report["rollback_equal"];
                report["injected_failure"]=addBeforeFailure?"after geometry add":"after existing layer update";
                report["used_blind_undo"]=false;Write(stage,report);return Result.Failure;
            }
            Require(doc.AddCustomUndoEvent("OM9 diagnostic active cursor",OnCursorUndo,doc.Layers[oldCurrent].Id),"Private cursor Undo event registration failed");
            Cursor=doc.Layers[doc.Layers.CurrentLayerIndex].Id;
            doc.Views.Redraw();report["after"]=State(doc);report["ok"]=true;Write(stage,report);return Result.Success;
        } catch(Exception e) {report["ok"]=false;report["error"]=e.ToString();Write(stage,report);return Result.Failure;}
    }
#if OM9_CURRENT_SESSION_PROBE
    internal static Result Cleanup(RhinoDoc doc) {
        try {
            Owned(doc);
            if(FixtureDoc==null){Write("command-cleanup",new {ok=true,skipped=true,reason="no fixture created"});return Result.Success;}
            Require(Environment.GetEnvironmentVariable("OM9_LAYER_API_HOST_MODE")=="current_blank", "Current blank test ownership missing");
            RhinoApp.Idle-=OnCursorIdle;CursorPending=false;
            int originalIndex=doc.Layers.Find(OriginalCurrent,true);
            Require(originalIndex>=0&&doc.Layers.SetCurrentLayerIndex(originalIndex,true),"Original active layer unavailable");
            foreach(Guid id in FixtureLayers) {
                int index=doc.Layers.Find(id,true);if(index<0)continue;
                var layer=doc.Layers[index];layer.IsLocked=false;layer.IsVisible=true;Commit(layer);
            }
            foreach(Guid id in FixtureObjects) {
                var obj=doc.Objects.Find(id);if(obj==null||obj.IsDeleted)continue;
                if(obj.IsLocked)Require(doc.Objects.Unlock(id,true),"Own fixture object unlock failed");
                if(obj.IsHidden)Require(doc.Objects.Show(id,true),"Own fixture object show failed");
                Require(doc.Objects.Delete(id,true),"Own fixture object cleanup failed");
            }
            for(int i=FixtureLayers.Count-1;i>=0;i--) {
                int index=doc.Layers.Find(FixtureLayers[i],true);
                if(index>=0)Require(doc.Layers.Delete(index,true),"Own fixture layer cleanup failed");
            }
            string after=State(doc);Require(after==OriginalState,"Blank Matrix document state changed outside fixture");
            // User chose a dedicated fresh blank test document. Rhino5 lacks
            // selective record removal; never use this on a working document.
            Require(!OriginalModified&&String.IsNullOrEmpty(doc.Path),"Blank test history ownership changed");
            doc.ClearUndoRecords(false);doc.ClearRedoRecords();doc.Modified=OriginalModified;
            doc.Views.Redraw();
            Write("command-cleanup",new {ok=true,original=OriginalState,after=after,document_id=doc.DocumentId,dedicated_blank_test_history_cleared=true});
            FixtureDoc=null;FixtureRun=null;FixtureLayers.Clear();FixtureObjects.Clear();return Result.Success;
        } catch(Exception error){Write("command-cleanup",new {ok=false,error=error.ToString()});return Result.Failure;}
    }
#endif
}
[Guid(ProbeIdentity.Prepare)]
public sealed class OM9LayerApiPrepareFixture : Command {
    public override string EnglishName {get{return ProbeIdentity.Prefix+"PrepareFixture";}}
    protected override Result RunCommand(RhinoDoc doc,RunMode mode) {try{LayerProbe.Setup(doc);return Result.Success;}catch(Exception e){LayerProbe.Write("command-prepare",new {ok=false,error=e.ToString()});return Result.Failure;}}
}
[Guid(ProbeIdentity.Apply)]
public sealed class OM9LayerApiApplyProbe : Command {
    public override string EnglishName {get{return ProbeIdentity.Prefix+"ApplyProbe";}}
    protected override Result RunCommand(RhinoDoc doc,RunMode mode) {return LayerProbe.Apply(doc,"command-apply",false,false);}
}
[Guid(ProbeIdentity.FailLayer)]
public sealed class OM9LayerApiFailLayerProbe : Command {
    public override string EnglishName {get{return ProbeIdentity.Prefix+"FailLayerProbe";}}
    protected override Result RunCommand(RhinoDoc doc,RunMode mode) {return LayerProbe.Apply(doc,"command-fail-layer",true,false);}
}
[Guid(ProbeIdentity.FailGeometry)]
public sealed class OM9LayerApiFailGeometryProbe : Command {
    public override string EnglishName {get{return ProbeIdentity.Prefix+"FailGeometryProbe";}}
    protected override Result RunCommand(RhinoDoc doc,RunMode mode) {return LayerProbe.Apply(doc,"command-fail-geometry",true,true);}
}
[Guid(ProbeIdentity.Sample)]
public sealed class OM9LayerApiSample : Command {
    public override string EnglishName {get{return ProbeIdentity.Prefix+"Sample";}}
    protected override Result RunCommand(RhinoDoc doc,RunMode mode) {try{LayerProbe.Owned(doc);LayerProbe.Write("command-sample",new {ok=true,state=LayerProbe.State(doc)});return Result.Success;}catch(Exception e){LayerProbe.Write("command-sample",new {ok=false,error=e.ToString()});return Result.Failure;}}
}
#if OM9_CURRENT_SESSION_PROBE
[Guid("b241b6e4-e7e7-4669-a04a-1cf4a7143e1a")]
[CommandStyle(Style.NotUndoable)]
public sealed class OM9LayerDirectApiCleanup : Command {
    public override string EnglishName {get{return "OM9LayerDirectApiCleanup";}}
    protected override Result RunCommand(RhinoDoc doc,RunMode mode){return LayerProbe.Cleanup(doc);}
}
#endif
