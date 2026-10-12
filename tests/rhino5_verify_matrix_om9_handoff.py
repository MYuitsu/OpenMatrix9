# -*- coding: utf-8 -*-
"""Two actual apps: current Matrix + owned OM9, native clipboard/local Undo."""
import os,sys,json,time,traceback,hashlib,ntpath
import Rhino,System
import clr
clr.AddReference('System.Drawing')
from System.Diagnostics import Process,ProcessStartInfo
TESTS=r'C:\Users\nguye\.codex\worktrees\layer-session-handoff\FreeCAD-src\Mod\OpenMatrix9\tests'
sys.path.insert(0,TESTS)
def load_support(path):
    import types,hashlib
    module=types.ModuleType('om9_handoff_support_run')
    module.__file__=path
    with open(path,'rb') as stream:source=stream.read()
    exec(compile(source,path,'exec'),module.__dict__)
    module.__source_sha256__=hashlib.sha256(source).hexdigest()
    return module
support=load_support(ntpath.join(TESTS,'matrix_om9_handoff_support.py'))
report={'version':1,'ok':False,'checks':[],'directions':{},'timings':[],'scope':'Current Matrix transfer <-> OM9 public clipboard; OM9 receiving Undo/Redo only','undo_scope':'om9_only','test_support_sha256':support.__source_sha256__}
run_id=System.Guid.NewGuid().ToString('N')
output=ntpath.join(r'H:\FreeCAD-src\build\matrix-om9-handoff',run_id)
original=None;original_doc=None;owned_objects=set();owned_layers=set();sequence=0;companion_pid=None;finished=False
handler=None;last_progress=0;bootstrap_queued=False;resuming=False

def log(text):
    Rhino.RhinoApp.WriteLine(text)
    with open(ntpath.join(output,'handoff.log'),'ab') as stream:stream.write((text+'\n').encode('utf-8'))

def check(name,value,details=None):
    row={'name':name,'passed':bool(value)}
    if details is not None:row['details']=details
    report['checks'].append(row)
    if not value:log('FAIL: '+name+(': '+json.dumps(details,ensure_ascii=True) if details else ''))

def checkpoint(name,value):
    support.write_json(ntpath.join(output,name+'.json'),value)

def terminal_failure(error):
    global finished
    # Small JSON-native fail record remains writable if the full report fails.
    value=dict(ok=False,application_accepted=False,run_id=str(run_id),error=str(error),
        scope='Harness terminal failure; no two-app acceptance',cleanup_verified=False)
    support.write_json(ntpath.join(output,'handoff-terminal-error.json'),value)
    finished=True
    log('Matrix <-> OM9 handoff: FAIL; terminal error saved. Report directory: '+output)

def document():
    doc=Rhino.RhinoDoc.ActiveDoc
    if doc is None or original_doc is not None and int(doc.DocumentId)!=original_doc:
        raise RuntimeError('Current Matrix test document changed; stopped before further commands')
    return doc

def objects(doc):
    settings=Rhino.DocObjects.ObjectEnumeratorSettings()
    for field in ('NormalObjects','LockedObjects','HiddenObjects','ActiveObjects','ReferenceObjects','IncludeLights','IncludeGrips'):setattr(settings,field,True)
    # Rhino5: true means ONLY instance-definition members, not include them.
    settings.IdefObjects=False
    return doc.Objects.GetObjectList(settings)

def snapshot():
    doc=document();layers=[];rows=[];ids=[]
    for i in range(doc.Layers.Count):
        layer=doc.Layers[i]
        if layer is None or layer.IsDeleted:continue
        ids.append(str(layer.Id))
        layers.append(dict(id=str(layer.Id),path=str(layer.FullPath).split('::'),rgb=[int(layer.Color.R),int(layer.Color.G),int(layer.Color.B)],
            locked=bool(layer.IsLocked),visible=bool(layer.IsVisible),persistent_locked=bool(layer.GetPersistentLocking()),persistent_visible=bool(layer.GetPersistentVisibility())))
    for obj in objects(doc):
        attrs=obj.Attributes;box=obj.Geometry.GetBoundingBox(True)
        if not box.IsValid:raise RuntimeError('Invalid native geometry bounding box')
        rows.append(dict(id=str(obj.Id),name=str(attrs.Name or ''),layer=str(doc.Layers[attrs.LayerIndex].FullPath).split('::'),
            locked=attrs.Mode==Rhino.DocObjects.ObjectMode.Locked,visible=bool(attrs.Visible),
            color_source='ByObject' if attrs.ColorSource==Rhino.DocObjects.ObjectColorSource.ColorFromObject else 'ByLayer',
            rgb=[int(attrs.ObjectColor.R),int(attrs.ObjectColor.G),int(attrs.ObjectColor.B)],bounds=[float(box.Min.X),float(box.Min.Y),float(box.Min.Z),float(box.Max.X),float(box.Max.Y),float(box.Max.Z)]))
    rows.sort(key=lambda row:row['id']);layers.sort(key=lambda row:row['id'])
    raw=dict(layers=layers,objects=rows,active=str(doc.Layers[doc.Layers.CurrentLayerIndex].Id))
    semantic=dict(layers=[dict((k,v) for k,v in row.items() if k!='id') for row in layers],
        objects=[dict((k,v) for k,v in row.items() if k!='id') for row in rows],active=str(doc.Layers[doc.Layers.CurrentLayerIndex].FullPath).split('::'))
    return dict(raw=raw,semantic=semantic,document_id=int(doc.DocumentId))

def clipboard_sequence():
    import ctypes
    return int(ctypes.windll.user32.GetClipboardSequenceNumber())

def run_native(command):
    document();started=time.time();result=bool(Rhino.RhinoApp.RunScript(command,False))
    report['timings'].append(dict(app='Matrix',operation=command,seconds=time.time()-started,returned=result,in_command=bool(Rhino.Commands.Command.InCommand())))
    document();return result

def add_layer(name,rgb,parent=None,locked=False,visible=True):
    doc=document();layer=Rhino.DocObjects.Layer();layer.Name=name
    layer.Color=System.Drawing.Color.FromArgb(*rgb)
    if parent is not None:layer.ParentLayerId=doc.Layers[parent].Id
    layer.IsLocked=locked;layer.IsVisible=visible
    index=doc.Layers.Add(layer)
    if index<0:raise RuntimeError('Cannot create owned Matrix fixture layer')
    owned_layers.add(str(doc.Layers[index].Id));return index

def fixture():
    doc=document();record=doc.BeginUndoRecord('Matrix/OM9 dedicated test fixture')
    try:
        root=add_layer('Matrix Transfer '+run_id[:8],[21,71,131])
        child=add_layer('Detail',[13,93,173],root)
        add_layer('Matrix Empty '+run_id[:8],[173,19,211],None,True,False)
        constrained=add_layer('Matrix Restricted '+run_id[:8],[51,61,71],None,True,False)
        own=add_layer('Own state',[91,101,111],constrained,True,False)
        layer=doc.Layers[own];layer.SetPersistentLocking(False);layer.SetPersistentVisibility(True)
        if not layer.CommitChanges():raise RuntimeError('Cannot set fixture child desired flags')
        if not doc.Layers.SetCurrentLayerIndex(root,True):raise RuntimeError('Cannot set fixture active layer')
        a=Rhino.DocObjects.ObjectAttributes();a.Name='Matrix Point';a.LayerIndex=child
        identity=doc.Objects.AddPoint(Rhino.Geometry.Point3d(10,20,30),a);owned_objects.add(str(identity))
        b=Rhino.DocObjects.ObjectAttributes();b.Name='Matrix Curve';b.LayerIndex=root
        b.ColorSource=Rhino.DocObjects.ObjectColorSource.ColorFromObject;b.ObjectColor=System.Drawing.Color.FromArgb(221,31,51)
        identity=doc.Objects.AddCurve(Rhino.Geometry.LineCurve(Rhino.Geometry.Point3d(20,0,0),Rhino.Geometry.Point3d(24,1,0)),b);owned_objects.add(str(identity))
        if str(System.Guid.Empty) in owned_objects:raise RuntimeError('Fixture object insertion failed')
    finally:
        if record:doc.EndUndoRecord(record)
    doc.Views.Redraw()

def cleanup():
    if original is None:return True
    doc=document()
    if doc.Path:raise RuntimeError('Test document was saved; cleanup stopped')
    current=snapshot()['raw']
    if any(row['id'] not in owned_objects for row in current['objects']):raise RuntimeError('Foreign geometry appeared; refusing cleanup')
    if any(row['id'] not in owned_layers and row['id'] not in set(r['id'] for r in original['raw']['layers']) for row in current['layers']):raise RuntimeError('Foreign layer appeared; refusing cleanup')
    old=System.Guid(original['raw']['active']);index=doc.Layers.Find(old,True)
    if index<0 or not doc.Layers.SetCurrentLayerIndex(index,True):raise RuntimeError('Original active layer unavailable')
    for row in sorted(current['layers'],key=lambda r:len(r['path'])):
        if row['id'] in owned_layers:
            layer=doc.Layers[doc.Layers.Find(System.Guid(row['id']),True)];layer.IsLocked=False;layer.IsVisible=True
            if not layer.CommitChanges():raise RuntimeError('Own layer unlock during cleanup failed')
    for row in current['objects']:
        identity=System.Guid(row['id']);obj=doc.Objects.Find(identity)
        if obj.IsLocked:doc.Objects.Unlock(identity,True)
        if obj.IsHidden:doc.Objects.Show(identity,True)
        if not doc.Objects.Delete(identity,True):raise RuntimeError('Own object cleanup failed')
    for row in sorted(current['layers'],key=lambda r:len(r['path']),reverse=True):
        if row['id'] in owned_layers:
            index=doc.Layers.Find(System.Guid(row['id']),True)
            if index>=0 and not doc.Layers.Delete(index,True):
                failure=dict(layer=row,current_layer_index=int(doc.Layers.CurrentLayerIndex),failed_index=int(index),state=snapshot())
                checkpoint('cleanup-layer-failure',failure)
                raise RuntimeError('Own layer cleanup failed: '+str(doc.Layers[index].FullPath)+'; see cleanup-layer-failure.json')
    for row in sorted(original['raw']['layers'],key=lambda r:len(r['path'])):
        layer=doc.Layers[doc.Layers.Find(System.Guid(row['id']),True)]
        layer.Color=System.Drawing.Color.FromArgb(*row['rgb']);layer.IsLocked=row['locked'];layer.IsVisible=row['visible']
        layer.SetPersistentLocking(row['persistent_locked']);layer.SetPersistentVisibility(row['persistent_visible'])
        if not layer.CommitChanges():raise RuntimeError('Original palette restoration failed')
    if snapshot()['raw']!=original['raw']:raise RuntimeError('Original Matrix document state was not restored exactly')
    doc.ClearUndoRecords(False);doc.ClearRedoRecords();doc.Modified=False;doc.Views.Redraw()
    return True

def wait_json(path,timeout=180):
    deadline=time.time()+timeout
    while not os.path.isfile(path):
        if time.time()>deadline:raise RuntimeError('Timed out waiting for '+path)
        yield None

def request(action):
    global sequence
    sequence+=1;seq=sequence
    support.write_json(ntpath.join(output,'request-%03d.json'%seq),dict(version=1,run_id=run_id,seq=seq,action=action))
    path=ntpath.join(output,'response-%03d.json'%seq)
    for pending in wait_json(path):yield pending
    response=support.read_json(path)
    if response.get('version')!=1 or response.get('run_id')!=run_id or response.get('seq')!=seq or response.get('pid')!=companion_pid:raise RuntimeError('Foreign OM9 response')
    report['timings'].append(dict(app='OM9',operation=action,seconds=response.get('seconds')))
    if not response.get('ok'):raise RuntimeError('OM9 '+action+' failed: '+response.get('error','unknown'))
    # Python2 generators cannot return a value. Preserve only this response.
    report['_response']=response['result']

def launch():
    plugin=r'C:\Users\nguye\.codex\worktrees\layer-session-handoff\FreeCAD-src\build\om9-plugin-runtime\bin\OpenMatrix9Gui.pyd'
    host=r'H:\FreeCAD-src\build\relWithDebInfo\bin\FreeCAD.exe'
    def digest(path):
        with open(path,'rb') as stream:return hashlib.sha256(stream.read()).hexdigest()
    config=dict(version=1,run_id=run_id,output=output,plugin=plugin,plugin_sha256=digest(plugin),host_sha256=digest(host))
    support.write_json(ntpath.join(output,'session.json'),config);report['runtime']=config
    info=ProcessStartInfo();info.FileName=r'C:\Program Files\PowerShell\7\pwsh.exe'
    if not os.path.isfile(info.FileName):info.FileName=ntpath.join(System.Environment.GetFolderPath(System.Environment.SpecialFolder.System),'WindowsPowerShell','v1.0','powershell.exe')
    info.Arguments='-NoProfile -File "'+ntpath.join(TESTS,'launch_matrix_om9_handoff.ps1')+'" -RunId '+run_id
    info.UseShellExecute=False;info.CreateNoWindow=True;info.WindowStyle=System.Diagnostics.ProcessWindowStyle.Hidden
    process=Process.Start(info);report['launcher_pid']=process.Id
    return process

def main():
    global original,original_doc,companion_pid,finished
    launched=None
    try:
        doc=document()
        if os.environ.get('OM9_LAYER_API_HOST_MODE')=='current_blank':raise RuntimeError('Previous diagnostic is still pending; press Esc to finish its menu before running this test')
        if doc.Modified or doc.Path or any(True for obj in objects(doc)):raise RuntimeError('Use a fresh blank unsaved document in CURRENT Matrix; no automatic New/Open is used')
        original_doc=int(doc.DocumentId);original=snapshot();checkpoint('matrix-original',original)
        report.update(run_id=run_id,matrix_pid=Process.GetCurrentProcess().Id,document_id=original_doc)
        check('actual Rhino5 Matrix version',str(Rhino.RhinoApp.Version).startswith('5.'))
        import clr
        plugin_type=clr.GetClrType(Rhino.PlugIns.PlugIn);exists=plugin_type.GetMethod('PlugInExists')
        for name,identity in [('DotMatrix','e9c738e7-3303-405d-a42b-881d52b01bdb'),('gvMatrixCore','7faef691-c68e-462a-af89-899bff11a80f')]:
            args=System.Array[System.Object]([System.Guid(identity),System.Boolean(False),System.Boolean(False)])
            loaded=bool(exists.Invoke(None,args) and args[1]);check('current Matrix component already loaded '+name,loaded)
            if not loaded:raise RuntimeError('Run inside Matrix9 with its frontend/core already loaded')
        log('Two-app handoff: current Matrix + owned OM9. Uses the system clipboard; please leave both test documents alone.')
        launched=launch();ready_path=ntpath.join(output,'companion-ready.json')
        for pending in wait_json(ready_path):yield pending
        ready=support.read_json(ready_path)
        if ready.get('run_id')!=run_id or not ready.get('ok'):raise RuntimeError('OM9 companion startup failed: '+ready.get('error','ownership mismatch'))
        companion_pid=ready['pid'];report['om9_pid']=companion_pid
        if ready.get('plugin_sha256')!=report['runtime']['plugin_sha256']:raise RuntimeError('Wrong OM9 native runtime')
        check('two actual applications ready',companion_pid!=report['matrix_pid'])
        before_om9=ready['state'];fixture();matrix_source=snapshot();report['matrix_source']=matrix_source;checkpoint('matrix-source',matrix_source)
        log('Matrix -> OM9: Copy/Paste, palette/geometry, Undo, Redo.')
        doc.Objects.UnselectAll()
        for identity in owned_objects:
            if not doc.Objects.Select(System.Guid(identity)):raise RuntimeError('Cannot select owned Matrix copy fixture')
        before_sequence=clipboard_sequence();copied=run_native('_CopyToClipboard') and clipboard_sequence()!=before_sequence
        direction=dict(copy=copied);report['directions']['matrix_to_om9']=direction;check('Matrix native Copy publishes clipboard',copied)
        if not copied:raise RuntimeError('Matrix Copy failed; OM9 Paste was not attempted')
        for pending in request('paste'):yield pending
        after_om9=report.pop('_response');report['om9_received']=after_om9
        direction['paste']=len(after_om9['semantic']['objects'])-len(before_om9['semantic']['objects'])==2
        check('OM9 Paste adds the two Matrix objects',direction['paste'])
        differences=support.transfer_errors(matrix_source['semantic'],after_om9['semantic']);direction['transfer']=not differences
        check('Matrix -> OM9 geometry/full palette/locks/active match',not differences,differences)
        for pending in request('undo'):yield pending
        undone=report.pop('_response');direction['undo']=undone['raw']==before_om9['raw'] and undone['semantic']==before_om9['semantic']
        check('one OM9 Undo restores destination state and removes paste',direction['undo'])
        for pending in request('redo'):yield pending
        redone=report.pop('_response');direction['redo']=redone['raw']==after_om9['raw'] and redone['semantic']==after_om9['semantic']
        check('one OM9 Redo restores geometry and layer state',direction['redo'])
        log('OM9 -> Matrix: Copy Session/Paste, palette/geometry. Undo/Redo scope: OM9 only.')
        for pending in request('fixture'):yield pending
        om9_source=report.pop('_response');report['om9_source']=om9_source
        for pending in request('copy'):yield pending
        report['om9_copy']=report.pop('_response');direction=dict(copy=True);report['directions']['om9_to_matrix']=direction
        check('OM9 public Copy Session publishes current models/full palette',True)
        before_matrix=snapshot();before_ids=set(row['id'] for row in before_matrix['raw']['objects']);before_layers=set(row['id'] for row in before_matrix['raw']['layers'])
        checkpoint('matrix-before-paste',before_matrix)
        try:
            pasted=run_native('_Paste')
            yield None
        finally:
            observed=snapshot()
            owned_objects.update(row['id'] for row in observed['raw']['objects'] if row['id'] not in before_ids)
            owned_layers.update(row['id'] for row in observed['raw']['layers'] if row['id'] not in before_layers)
        after_matrix=snapshot();report['matrix_received']=after_matrix;checkpoint('matrix-after-paste',after_matrix)
        direction['paste']=pasted and len(after_matrix['semantic']['objects'])-len(before_matrix['semantic']['objects'])==3
        direction['paste_observation']=dict(returned=pasted,before_count=len(before_matrix['raw']['objects']),after_count=len(after_matrix['raw']['objects']),
            added=[row for row in after_matrix['raw']['objects'] if row['id'] not in before_ids])
        check('Matrix native Paste adds the three OM9 objects',direction['paste'],direction['paste_observation'])
        differences=support.transfer_errors(om9_source['semantic'],after_matrix['semantic']);direction['transfer']=not differences
        check('OM9 -> Matrix geometry/full palette/locks/active match',not differences,differences)
        if not direction['paste']:raise RuntimeError('Matrix Paste failed')

    except Exception:
        report['error']=traceback.format_exc();check('two-app workflow completed without exception',False,report['error'])
    finally:
        checkpoint('workflow-before-cleanup',report)
        try:report['cleanup_ok']=cleanup();check('original blank Matrix document restored',report['cleanup_ok'])
        except Exception:report['cleanup_ok']=False;report['cleanup_error']=traceback.format_exc();check('original blank Matrix document restored',False,report['cleanup_error'])
        if companion_pid:
            try:
                for pending in request('finish'):yield pending
                report['companion_finished']=report.pop('_response').get('finished',False)
                check('owned OM9 companion finished',report['companion_finished'])
            except Exception:check('owned OM9 companion finished',False,traceback.format_exc())
        elif launched is not None:
            report['companion_cleanup']='Startup did not provide owned ready PID; no application was force-killed. See host logs.'
        report.pop('_response',None);report['ok']=support.accepted(report);report['application_accepted']=report['ok']
        support.write_json(ntpath.join(output,'handoff-results.json'),report)
        log('Matrix <-> OM9 handoff: '+('PASS' if report['ok'] else 'FAIL')+'; checks='+str(len(report['checks'])))
        log('Report: '+ntpath.join(output,'handoff-results.json'))
        finished=True

def start():
    global handler,bootstrap_queued
    os.makedirs(output)
    log('Report directory: '+output)
    work=main();bootstrap_queued=True
    def resume(sender,args):
        global last_progress,resuming
        if resuming or Rhino.Commands.Command.InCommand():return
        resuming=True
        try:next(work)
        except StopIteration:
            Rhino.RhinoApp.Idle-=handler
            return
        except Exception:
            Rhino.RhinoApp.Idle-=handler
            terminal_failure(traceback.format_exc())
            return
        finally:resuming=False
        if time.time()-last_progress>30:
            last_progress=time.time();log('Two-app handoff is running; timing details are recorded in the report.')
    handler=System.EventHandler(resume);Rhino.RhinoApp.Idle+=handler
    log('Queued after RunPythonScript ends. Matrix stays open; no Rhino application or .rhp is loaded.')

try:start()
except Exception:
    print(traceback.format_exc())
