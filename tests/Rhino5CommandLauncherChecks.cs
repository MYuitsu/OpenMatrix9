// Test only: execute launcher logging with the actual installed IronPython.
using System;
using System.IO;
using IronPython.Hosting;
class Rhino5CommandLauncherChecks {
 static int Main(string[] args) {
  try {
   var engine=Python.CreateEngine();var scope=engine.CreateScope();
   string source=File.ReadAllText(args[0]);int start=source.IndexOf("def write_utf8(");int end=source.IndexOf("\ndef ",start+1);
   if(start<0||end<0)throw new Exception("Launcher UTF8 writer function not found");
   engine.Execute("import System\n"+source.Substring(start,end-start),scope);
   scope.SetVariable("output",args[1]);
   engine.Execute("message=u'L\\u1ed7i th\\u1eed nghi\\u1ec7m \\u2014 m\\u00e0u l\\u1edbp'\nwrite_utf8(output,message)\n",scope);
   if(File.ReadAllText(args[1])!="Lỗi thử nghiệm — màu lớp")throw new Exception("UTF8 content changed");
   Console.WriteLine("Installed Rhino5 IronPython UTF8 launcher log PASS");return 0;
  }catch(Exception e){Console.Error.WriteLine(e);return 1;}
 }
}
