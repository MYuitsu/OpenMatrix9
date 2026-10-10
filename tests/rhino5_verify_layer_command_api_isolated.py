"""Run the isolated prerequisite launcher from the user's Rhino token; no live-doc mutation."""
import os,json,re,System,traceback
from System.Diagnostics import Process,ProcessStartInfo
from System.Threading import Thread

try:text_type=unicode
except NameError:text_type=str

def write_utf8(path,text):
    # IronPython 2 in Rhino5 may ASCII-decode even unicode(CLR-string).
    # Pass the already UTF-16 CLR/Python string directly to the .NET writer.
    System.IO.File.WriteAllText(path,text,System.Text.UTF8Encoding(False))
def main():
    # This user-run retry deliberately cancels the exact test process that is
    # still stuck loading the diagnostic plugin. Never discover/kill user apps.
    owned_manifest=r'H:\FreeCAD-src\build\matrix-layer-api-probe\10957398142148aaa19746f9c909375c\launch.json'
    if os.path.isfile(owned_manifest):
        with open(owned_manifest,'r') as stream:owned=json.load(stream)
        if owned.get('run_id')!='10957398142148aaa19746f9c909375c' or owned.get('pid')!=6916 or not owned.get('timed_out'):
            raise RuntimeError('Unexpected current timed-out probe identity; no process stopped')
        try:old_probe=Process.GetProcessById(6916)
        except System.ArgumentException:old_probe=None
        if old_probe is not None:
            if old_probe.Id==Process.GetCurrentProcess().Id:raise RuntimeError('Run from your working Matrix window, not the owned probe')
            stamp=System.DateTime.Parse(owned['utc'],System.Globalization.CultureInfo.InvariantCulture,System.Globalization.DateTimeStyles.RoundtripKind)
            expected=os.path.normcase(os.path.abspath(owned['executable']))
            actual=os.path.normcase(os.path.abspath(old_probe.MainModule.FileName))
            if old_probe.ProcessName.lower()!='rhino' or expected!=actual or abs((old_probe.StartTime.ToUniversalTime()-stamp.ToUniversalTime()).TotalSeconds)>5:
                raise RuntimeError('Timed-out probe PID was reused; refusing cleanup')
            print('Closing only owned timed-out probe PID=6916 before your requested retry.')
            old_probe.Kill()
            if not old_probe.WaitForExit(5000):raise RuntimeError('Owned probe is still stopping; no replacement started')
    # Cleanup is restricted to the exact owned, timed-out probe manifest.
    manifest=r'H:\FreeCAD-src\build\matrix-layer-api-probe\c00ba4dd83414402889325c3b4af377f\launch.json'
    if os.path.isfile(manifest):
        with open(manifest,'r') as stream:previous=json.load(stream)
        if previous.get('run_id')!='c00ba4dd83414402889325c3b4af377f' or previous.get('pid')!=54872:raise RuntimeError('Unexpected old probe identity')
        try:old=Process.GetProcessById(54872)
        except System.ArgumentException:old=None
        if old is not None:
            if old.Id==Process.GetCurrentProcess().Id:raise RuntimeError('Run this entrypoint in your working Rhino, not the old owned probe window')
            stamp=System.DateTime.Parse(previous['utc'],System.Globalization.CultureInfo.InvariantCulture,System.Globalization.DateTimeStyles.RoundtripKind)
            if old.ProcessName.lower()!='rhino' or abs((old.StartTime.ToUniversalTime()-stamp.ToUniversalTime()).TotalSeconds)>5:raise RuntimeError('Old probe PID was reused; refusing cleanup')
            print('Closing only the timed-out owned probe PID=54872; user Rhino is untouched.')
            old.Kill()
            if not old.WaitForExit(5000):raise RuntimeError('Old owned probe is still stopping; no replacement was started')
    launcher=r'C:\Users\nguye\.codex\worktrees\layer-session-handoff\FreeCAD-src\Mod\OpenMatrix9\tests\run_rhino5_layer_api_prerequisite.ps1'
    shell=r'C:\Program Files\PowerShell\7\pwsh.exe'
    if not os.path.isfile(shell):shell=os.path.join(System.Environment.GetFolderPath(System.Environment.SpecialFolder.System),'WindowsPowerShell','v1.0','powershell.exe')
    if not os.path.isfile(launcher):raise RuntimeError('Missing isolated probe launcher: '+launcher)
    info=ProcessStartInfo();info.FileName=shell;info.Arguments='-NoProfile -File "'+launcher+'" -TimeoutSeconds 180 -ShowProbeUI'
    info.UseShellExecute=False;info.CreateNoWindow=True;info.WindowStyle=System.Diagnostics.ProcessWindowStyle.Hidden
    info.RedirectStandardOutput=True;info.RedirectStandardError=True
    info.EnvironmentVariables['NO_COLOR']='1'
    info.EnvironmentVariables['TERM']='dumb'
    print('Starting isolated Matrix/Rhino API prerequisite; your current document is not used.')
    process=Process.Start(info)
    while not process.HasExited:Thread.CurrentThread.Join(250)
    # Rhino's exception dialog prints escape sequences literally. Keep the
    # complete diagnostic readable even with a colored PowerShell error view.
    plain=lambda value:re.sub(r'\x1b\[[0-?]*[ -/]*[@-~]','',value)
    stdout=plain(process.StandardOutput.ReadToEnd());stderr=plain(process.StandardError.ReadToEnd())
    logfile=r'H:\FreeCAD-src\build\om9-layer-edits\tests\rhino5-layer-api-last-launch.log'
    write_utf8(logfile,'Exit code: '+str(process.ExitCode)+'\n'+stdout+'\n'+stderr)
    if process.ExitCode:
        print('Matrix API diagnostic FAIL; full message saved: '+logfile)
        return
    if stdout:print(stdout.encode('ascii','backslashreplace'))
    print('Matrix command API diagnostic completed; product two-way layer handoff still requires its own tests.')
try:main()
except Exception:
    logfile=r'H:\FreeCAD-src\build\om9-layer-edits\tests\rhino5-layer-api-last-launch.log'
    write_utf8(logfile,traceback.format_exc())
    print('Matrix API diagnostic FAIL; full message saved: '+logfile)
