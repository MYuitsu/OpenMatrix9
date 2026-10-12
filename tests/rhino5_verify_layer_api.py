"""Read installed layer APIs in the Matrix session the user opened directly.

No app launch, plugin load, document mutation or command Undo/Redo is performed.
The isolated command gate remains in rhino5_verify_layer_command_api.py.
"""
import os,json,traceback,System

try:text_type=unicode
except NameError:text_type=str

def write_utf8(path,text):
    System.IO.File.WriteAllText(path,text,System.Text.UTF8Encoding(False))

def main():
    root=os.environ.get('OM9_LAYER_API_DIRECT_ROOT',r'H:\FreeCAD-src\build\matrix-layer-api-direct')
    run_id=System.Guid.NewGuid().ToString('N')
    output=os.path.join(root,run_id)
    os.makedirs(output)
    logfile=os.path.join(output,'matrix-layer-api.log')
    last_log=r'H:\FreeCAD-src\build\om9-layer-edits\tests\rhino5-layer-api-last-launch.log'
    source=r'C:\Users\nguye\.codex\worktrees\layer-session-handoff\FreeCAD-src\Mod\OpenMatrix9\tests\rhino5_layer_api_prerequisite.py'
    keys=('OM9_LAYER_API_RUN_ID','OM9_LAYER_API_OUTPUT')
    previous=dict((key,os.environ.get(key)) for key in keys)
    print('Inspecting APIs in this Matrix session; no plugin loading or document edits.')
    print('Report directory: '+output)
    try:
        os.environ[keys[0]]=run_id;os.environ[keys[1]]=output
        execfile(source,{'__name__':'__main__','OM9_LAYER_API_DIRECT_READ_ONLY':True})
        report_file=os.path.join(output,'matrix-api-prerequisite.json')
        with open(report_file,'r') as stream:report=json.load(stream)
        message=json.dumps(report,ensure_ascii=False,indent=2)
        write_utf8(logfile,message)
        write_utf8(last_log,output+'\n'+message)
        if report.get('read_only_api_passed'):
            print('Matrix read-only layer API: PASS. Command Undo/Redo and full handoff remain unverified.')
        else:
            print('Matrix read-only layer API: FAIL. Full diagnostic: '+logfile)
            print(json.dumps(report.get('error','No diagnostic was returned.'),ensure_ascii=True))
    except Exception:
        message=traceback.format_exc()
        write_utf8(logfile,message)
        write_utf8(last_log,output+'\n'+message)
        print('Matrix layer API: FAIL. Full diagnostic: '+logfile)
        print(json.dumps(message,ensure_ascii=True))
    finally:
        for key,value in previous.items():
            if value is None:os.environ.pop(key,None)
            else:os.environ[key]=value

main()
