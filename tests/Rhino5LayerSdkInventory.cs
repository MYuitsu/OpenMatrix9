// Diagnostic tool only: reflection of the installed primary SDK, no host mutation.
using System;
using System.IO;
using System.Reflection;
using System.Collections.Generic;
using System.Web.Script.Serialization;
public static class Rhino5LayerSdkInventory {
    public static int Main(string[] args) {
        try {
            if(args.Length!=2) throw new ArgumentException("SDK path and output path required");
            AppDomain.CurrentDomain.ReflectionOnlyAssemblyResolve += delegate(object sender,ResolveEventArgs e) { return Assembly.ReflectionOnlyLoad(e.Name); };
            Assembly sdk=Assembly.ReflectionOnlyLoadFrom(Path.GetFullPath(args[0]));
            string[] types={"Rhino.RhinoDoc","Rhino.RhinoApp","Rhino.DocObjects.Layer","Rhino.DocObjects.Tables.LayerTable","Rhino.DocObjects.Tables.ObjectTable","Rhino.DocObjects.ObjectAttributes","Rhino.PlugIns.PlugIn","Rhino.Commands.Command"};
            HashSet<string> names=new HashSet<string>(new string[]{"BeginUndoRecord","EndUndoRecord","Undo","Redo","AddCustomUndoEvent","UndoRecordingEnabled","UndoRecordingIsActive","UndoSerialNumber","CurrentUndoRecordSerialNumber","NextUndoRecordSerialNumber","RuntimeSerialNumber","Version","ExeVersion","ExeServiceRelease","BuildDate","GetPersistentLocking","SetPersistentLocking","UnsetPersistentLocking","GetPersistentVisibility","SetPersistentVisibility","UnsetPersistentVisibility","IsLocked","IsVisible","ParentLayerId","Id","LayerIndex","ColorSource","ObjectColor","Mode","Name","FullPath","SetCurrentLayerIndex","CurrentLayerIndex","Modify","CommitChanges","Add","Delete","Find","FindByFullPath","LoadPlugIn","GetLoadedPlugIn","GetInstalledPlugInNames","GetInstalledPlugIns","PlugInExists","IdFromName","GetEnglishCommandNames","GetPlugInObject","IsCommand","InCommand","CommandStyle","EnglishName"});
            List<object> results=new List<object>();
            foreach(string name in types) {
                Type type=sdk.GetType(name,true);List<object> members=new List<object>();
                foreach(MethodInfo method in type.GetMethods(BindingFlags.Public|BindingFlags.Instance|BindingFlags.Static))
                    if(names.Contains(method.Name) || method.Name=="Duplicate" || method.Name=="DuplicateAttributes") members.Add(new {kind="method",name=method.Name,signature=method.ToString(),is_static=method.IsStatic});
                foreach(PropertyInfo prop in type.GetProperties(BindingFlags.Public|BindingFlags.Instance|BindingFlags.Static))
                    if(names.Contains(prop.Name)) members.Add(new {kind="property",name=prop.Name,signature=prop.ToString(),readable=prop.CanRead,writable=prop.CanWrite});
                results.Add(new {type=name,members=members});
            }
            Type table=sdk.GetType("Rhino.DocObjects.Tables.ObjectTable",true);
            Type plugin=sdk.GetType("Rhino.PlugIns.PlugIn",true);
            MethodInfo exists=plugin.GetMethod("PlugInExists",new Type[]{typeof(Guid),typeof(bool).MakeByRefType(),typeof(bool).MakeByRefType()});
            if(exists==null || exists.ReturnType!=typeof(bool)) throw new MissingMethodException("Rhino5 PlugIn.PlugInExists(Guid, out bool, out bool)");
            List<string> existsParameters=new List<string>();foreach(ParameterInfo parameter in exists.GetParameters())existsParameters.Add(parameter.Name);
            if(existsParameters.Count!=3 || existsParameters[1]!="loaded" || existsParameters[2]!="loadProtected") throw new MissingMethodException("Unexpected Rhino5 PlugInExists output meaning");
            if(plugin.GetMethod("LoadPlugIn",new Type[]{typeof(Guid)})==null) throw new MissingMethodException("Rhino5 PlugIn.LoadPlugIn(Guid)");
            Type settings=sdk.GetType("Rhino.DocObjects.ObjectEnumeratorSettings",true);
            List<object> enumeration=new List<object>();
            foreach(PropertyInfo prop in settings.GetProperties()) enumeration.Add(new {name=prop.Name,signature=prop.ToString(),readable=prop.CanRead,writable=prop.CanWrite});
            List<string> objectLists=new List<string>();
            foreach(MethodInfo method in table.GetMethods()) if(method.Name=="GetObjectList") objectLists.Add(method.ToString());
            if(table.GetMethod("GetObjectList",new Type[]{settings})==null) throw new MissingMethodException("Rhino5 ObjectTable.GetObjectList(ObjectEnumeratorSettings)");
            foreach(string property in new string[]{"NormalObjects","LockedObjects","HiddenObjects","ActiveObjects","ReferenceObjects","IncludeLights","IncludeGrips"}) {
                PropertyInfo prop=settings.GetProperty(property);
                if(prop==null || !prop.CanWrite || prop.PropertyType!=typeof(bool)) throw new MissingMemberException("Rhino5 ObjectEnumeratorSettings."+property);
            }
            object report=new {schema_version=1,ok=true,scope="installed SDK metadata only; actual Matrix host architecture and runtime APIs unverified",sdk_path=sdk.Location,sdk_name=sdk.FullName,sdk_runtime=sdk.ImageRuntimeVersion,sdk_pe_architecture=sdk.GetName().ProcessorArchitecture.ToString(),actual_matrix_loaded=false,types=results,object_table_count_available=table.GetProperty("Count")!=null,object_list_overloads=objectLists,object_enumerator_settings=enumeration,plugin_exists_byref_available=true,plugin_exists_parameters=existsParameters,plugin_load_by_guid_available=true};
            File.WriteAllText(args[1],new JavaScriptSerializer().Serialize(report));return 0;
        }catch(Exception e){Console.Error.WriteLine(e.ToString());return 1;}
    }
}
