"""Test THIS Matrix process with a dedicated blank document; never launch Rhino."""
import os,json,traceback,System,Rhino
from System.Diagnostics import Process,ProcessStartInfo

def write_utf8(path,text):
    System.IO.File.WriteAllText(path,text,System.Text.UTF8Encoding(False))

def require_blank(doc):
    if doc is None or doc.Modified or doc.Path:
        raise RuntimeError('Use a fresh blank unsaved document in this Matrix session; the working document was not changed.')
    settings=Rhino.DocObjects.ObjectEnumeratorSettings()
    for name in ('NormalObjects','LockedObjects','HiddenObjects','ActiveObjects','ReferenceObjects','IncludeLights','IncludeGrips','IdefObjects'):
        setattr(settings,name,True)
    for obj in doc.Objects.GetObjectList(settings):
        raise RuntimeError('Document contains model objects; test stopped before changes.')
    return int(doc.DocumentId)

def registry(mode,run_id):
    helper=r'C:\Users\nguye\.codex\worktrees\layer-session-handoff\FreeCAD-src\Mod\OpenMatrix9\tests\register_matrix_current_probe.ps1'
    shell=r'C:\Program Files\PowerShell\7\pwsh.exe'
    if not os.path.isfile(shell):shell=os.path.join(System.Environment.GetFolderPath(System.Environment.SpecialFolder.System),'WindowsPowerShell','v1.0','powershell.exe')
    info=ProcessStartInfo();info.FileName=shell
    info.Arguments='-NoProfile -File "'+helper+'" -Mode '+mode+' -RunId '+run_id
    info.UseShellExecute=False;info.CreateNoWindow=True
    info.WindowStyle=System.Diagnostics.ProcessWindowStyle.Hidden
    info.RedirectStandardOutput=True;info.RedirectStandardError=True
    process=Process.Start(info)
    if not process.WaitForExit(10000):
        process.Kill()
        raise RuntimeError('Owned diagnostic registry helper timed out; no Rhino application was started.')
    stdout=process.StandardOutput.ReadToEnd();stderr=process.StandardError.ReadToEnd()
    if process.ExitCode:raise RuntimeError('Current-session diagnostic registration '+mode+' failed: '+stderr)
    return stdout

def main():
    if os.environ.get('OM9_LAYER_API_HOST_MODE')=='current_blank':
        raise RuntimeError('Current-session test is already active; let it finish first.')
    run_id=System.Guid.NewGuid().ToString('N')
    output=os.path.join(r'H:\FreeCAD-src\build\matrix-layer-api-current',run_id)
    os.makedirs(output)
    last_log=r'H:\FreeCAD-src\build\om9-layer-edits\tests\rhino5-layer-api-last-launch.log'
    keys=('OM9_LAYER_API_RUN_ID','OM9_LAYER_API_OUTPUT','OM9_LAYER_API_HOST_MODE')
    previous=dict((key,os.environ.get(key)) for key in keys)
    prepared=False
    def restore_environment():
        for key,value in previous.items():
            if value is None:os.environ.pop(key,None)
            else:os.environ[key]=value
    def complete():
        try:
            path=os.path.join(output,'matrix-api-prerequisite.json')
            try:
                with open(path,'r') as stream:result=json.load(stream)
                if result.get('run_id')!=run_id or result.get('pid')!=Process.GetCurrentProcess().Id:
                    raise RuntimeError('Current-session report ownership changed')
            except Exception:
                result={'run_id':run_id,'pid':Process.GetCurrentProcess().Id,'ok':False,
                    'full_api_gate_passed':False,'terminal_report_error':traceback.format_exc()}
            # Registry cleanup has its own exact ownership proof. A missing
            # native report must not strand the temporary diagnostic GUID.
            try:
                registry('Cleanup',run_id)
                result['temporary_registration_removed']=True
            except Exception:
                result.update(ok=False,full_api_gate_passed=False,registration_cleanup_error=traceback.format_exc())
            with open(path,'w') as stream:json.dump(result,stream,indent=2)
            write_utf8(last_log,output+'\n'+json.dumps(result,ensure_ascii=False,indent=2))
            print('Matrix CURRENT SESSION command API: '+('PASS' if result.get('full_api_gate_passed') else 'FAIL')+'; checks='+str(len(result.get('checks',[])))+'; report='+path)
            print('Full OM9/Matrix layer handoff still requires its own application tests.')
        except Exception:
            write_utf8(last_log,output+'\n'+traceback.format_exc())
            print('Matrix CURRENT SESSION test FAIL; full diagnostic: '+last_log)
        finally:restore_environment()
    try:
        document_id=require_blank(Rhino.RhinoDoc.ActiveDoc)
        os.environ[keys[0]]=run_id;os.environ[keys[1]]=output;os.environ[keys[2]]='current_blank'
        registry('Prepare',run_id);prepared=True
        print('Testing THIS Matrix session, PID='+str(Process.GetCurrentProcess().Id)+'; blank document='+str(document_id)+'. No Rhino process is launched.')
        print('Report directory: '+output)
        print('Dedicated blank test: fixtures and test Undo/Redo records will be cleaned; original palette/active layer restored.')
        source=r'C:\Users\nguye\.codex\worktrees\layer-session-handoff\FreeCAD-src\Mod\OpenMatrix9\tests\rhino5_layer_api_prerequisite.py'
        execfile(source,{'__name__':'__main__','OM9_LAYER_API_CURRENT_COMMAND_TEST':True,
            'OM9_LAYER_API_EXPECTED_DOCUMENT_ID':document_id,'OM9_LAYER_API_ON_COMPLETE':complete})
        # Environment/registration live until asynchronous Idle completion.
    except Exception:
        message=traceback.format_exc()
        if prepared:
            try:registry('Cleanup',run_id)
            except Exception:message+='\n'+traceback.format_exc()
        restore_environment();write_utf8(last_log,output+'\n'+message)
        print('Matrix CURRENT SESSION test stopped; full diagnostic: '+last_log)

try:main()
except Exception:
    write_utf8(r'H:\FreeCAD-src\build\om9-layer-edits\tests\rhino5-layer-api-last-launch.log',traceback.format_exc())
    print('Matrix CURRENT SESSION test stopped; see rhino5-layer-api-last-launch.log')
