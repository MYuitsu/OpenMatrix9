using System;
using System.IO;
using IronPython.Hosting;
class IronPythonHandoffChecks {
    static int Main(string[] args) {
        try {
            var engine=Python.CreateEngine();
            var paths=engine.GetSearchPaths();
            paths.Add(@"C:\Program Files\Rhinoceros 5 (64-bit)\Plug-ins\IronPython\Lib");
            paths.Add(Path.GetDirectoryName(args[0]));
            engine.SetSearchPaths(paths);
            engine.ExecuteFile(args[0]);
            return 0;
        } catch(Exception e) {Console.Error.WriteLine(e);return 1;}
    }
}
