// Test tool only: installed Rhino5 metadata and its actual IronPython compiler.
using System;
using System.IO;
using System.Reflection;
using System.Text;
using IronPython.Hosting;
class Rhino5LayerDirectSdkCheck {
    static void Readable(Type type,string name) {
        PropertyInfo value=type.GetProperty(name);
        if(value==null || !value.CanRead)throw new MissingMemberException(type.FullName,name);
        Console.WriteLine("SDK readable property: "+value);
    }
    static int Main(string[] args) {
        try {
            AppDomain.CurrentDomain.ReflectionOnlyAssemblyResolve += delegate(object sender,ResolveEventArgs e) { return Assembly.ReflectionOnlyLoad(e.Name); };
            Assembly sdk=Assembly.ReflectionOnlyLoadFrom(args[0]);
            Type doc=sdk.GetType("Rhino.RhinoDoc",true);
            Readable(doc,"DocumentId");Readable(doc,"Modified");Readable(doc,"Layers");
            Type table=sdk.GetType("Rhino.DocObjects.Tables.LayerTable",true);
            Readable(table,"Count");Readable(table,"CurrentLayerIndex");
            Type plugin=sdk.GetType("Rhino.PlugIns.PlugIn",true);
            if(plugin.GetMethod("Find",new Type[]{typeof(Guid)})==null)throw new MissingMethodException("Rhino5 PlugIn.Find(Guid)");
            Type utils=sdk.GetType("Rhino.Runtime.HostUtils",true);
            if(utils.GetMethod("RegisterDynamicCommand",new Type[]{plugin,sdk.GetType("Rhino.Commands.Command",true)})==null)throw new MissingMethodException("Rhino5 HostUtils.RegisterDynamicCommand(PlugIn,Command)");
            Readable(sdk.GetType("Rhino.RhinoApp",true),"CommandHistoryWindowText");
            if(sdk.GetType("Rhino.RhinoApp",true).GetEvent("Idle")==null)throw new MissingMemberException("Rhino5 RhinoApp.Idle");
            Console.WriteLine("SDK native dynamic-command registration available; actual command runtime still pending.");
            if(doc.GetProperty("RuntimeSerialNumber")!=null)throw new Exception("Installed Rhino5 contract changed: inspect identity before accepting test");
            var engine=Python.CreateEngine();
            for(int i=1;i<args.Length;i++) {
                engine.CreateScriptSourceFromFile(args[i]).Compile();
                Console.WriteLine("Installed Rhino5 IronPython syntax PASS: "+args[i]);
            }
            Console.WriteLine("SDK/compile checks only; actual Matrix runtime and full handoff remain separate.");
            return 0;
        }catch(Exception e){Console.Error.WriteLine(e);return 1;}
    }
}
