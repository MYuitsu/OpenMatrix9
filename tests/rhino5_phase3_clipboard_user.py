# Actual Rhino 5 / IronPython 2.7, launched in an owned empty document.
import Rhino,scriptcontext as sc,System,os,json,hashlib,uuid,ntpath,time,traceback,math,ctypes,struct,sys
root='H:/FreeCAD-src/build/om9-phase3-dev'
sys.path.insert(0,ntpath.join(root,'tests'))
from phase12_timing import Timing
out=os.environ.get('OM9_RHINO_CLIPBOARD_OUTPUT') or 'H:/FreeCAD-src/build/rhino5-clipboard/'+uuid.uuid4().hex
if not os.path.isdir(out):os.makedirs(out)
report={'checks':[],'cases':[],'rhino_version':str(Rhino.RhinoApp.Version),'pid':os.getpid(),'ok':False}
owned_document=False
def check(name,value):
    report['checks'].append({'name':name,'passed':bool(value)})
    with open(ntpath.join(out,'progress.json'),'w') as stream:json.dump(report,stream,indent=2)
    if not value:raise RuntimeError(name)
def digest(path):
    with open(path,'rb') as stream:return hashlib.sha256(stream.read()).hexdigest()
def clipboard_state():
    user=ctypes.windll.user32
    user.GetClipboardOwner.restype=ctypes.c_void_p
    user.GetWindowThreadProcessId.argtypes=[ctypes.c_void_p,ctypes.POINTER(ctypes.c_ulong)]
    owner=user.GetClipboardOwner();pid=ctypes.c_ulong()
    if owner:user.GetWindowThreadProcessId(owner,ctypes.byref(pid))
    state={'sequence':user.GetClipboardSequenceNumber(),'owner_pid':pid.value,'formats':[]}
    if pid.value:
        try:state['owner_process']=str(System.Diagnostics.Process.GetProcessById(int(pid.value)).ProcessName)
        except:pass
    if user.OpenClipboard(None):
        try:
            value=0
            for unused in range(200):
                value=user.EnumClipboardFormats(value)
                if not value:break
                name=ctypes.create_unicode_buffer(256);user.GetClipboardFormatNameW(value,name,256)
                state['formats'].append({'id':value,'name':name.value})
        finally:user.CloseClipboard()
    else:state['busy']=True
    return state
# Existing audited Rhino bounds resolver (including instance member transforms).
with open(ntpath.join(root,'tests/rhino5_modeling_exchange.py'),'rb') as stream:source=stream.read()
helpers=dict(Rhino=Rhino,System=System,os=os,json=json,hashlib=hashlib,ntpath=ntpath,math=math,report=report)
exec(compile(source[source.index('def check('):source.index('\ntry:\n')],'<audited-rhino-measurements>','exec'),helpers)
document_helpers={}
execfile(ntpath.join(root,'tests/rhino5_modeling_document.py'),document_helpers)
helpers['document_helpers']=document_helpers;helpers['check']=check
original_facts=helpers['facts']
def mesh_facts(geometry):
    if isinstance(geometry,Rhino.Geometry.Brep):
        box=geometry.GetBoundingBox(True)
        check('Rhino valid CAD bounds',box.IsValid and geometry.IsValid)
        row={'class':str(geometry.GetType().FullName),'bounds':[box.Min.X,box.Min.Y,box.Min.Z,box.Max.X,box.Max.Y,box.Max.Z],'faces':geometry.Faces.Count,'solid':geometry.IsSolid}
        # Measure only the scalar quantities required by the gate. Rhino5's
        # all-moments mass APIs can return null on valid complex Breps.
        row['area']=geometry.GetArea(1e-10,1e-10)
        check('Rhino scalar area is finite',not math.isnan(row['area']) and not math.isinf(row['area']))
        if row['area']==0:
            report.setdefault('unavailable_scalar_probes',[]).append(dict(field='area',faces=geometry.Faces.Count,bounds=row['bounds']))
            del row['area']
        if geometry.IsSolid:
            row['volume']=geometry.GetVolume(1e-10,1e-10)
            check('Rhino scalar volume is finite',not math.isnan(row['volume']) and not math.isinf(row['volume']))
            if row['volume']==0:
                report.setdefault('unavailable_scalar_probes',[]).append(dict(field='volume',faces=geometry.Faces.Count,bounds=row['bounds']))
                del row['volume']
    else:row=original_facts(geometry)
    if isinstance(geometry,Rhino.Geometry.Mesh):
        triangles=[]
        for face in geometry.Faces:
            indices=[(face.A,face.B,face.C)]
            if face.D!=face.C:indices.append((face.A,face.C,face.D))
            for ids in indices:
                triangles.append(tuple(sorted(struct.pack('<fff',geometry.Vertices[i].X,geometry.Vertices[i].Y,geometry.Vertices[i].Z) for i in ids)))
        row['mesh_geometry_sha256']=hashlib.sha256(''.join(''.join(triangle) for triangle in sorted(triangles))).hexdigest()
    return row
helpers['facts']=mesh_facts
def capture():
    user=ctypes.windll.user32;kernel=ctypes.windll.kernel32
    user.RegisterClipboardFormatW.argtypes=[ctypes.c_wchar_p];user.GetClipboardData.restype=ctypes.c_void_p
    kernel.GlobalSize.argtypes=[ctypes.c_void_p];kernel.GlobalSize.restype=ctypes.c_size_t
    kernel.GlobalLock.argtypes=[ctypes.c_void_p];kernel.GlobalLock.restype=ctypes.c_void_p;kernel.GlobalUnlock.argtypes=[ctypes.c_void_p]
    fmt=user.RegisterClipboardFormatW(u'Rhino 5.0 3DM Clip global mem')
    opened=user.OpenClipboard(None)
    retries=0
    while not opened and retries<5:
        retries+=1
        Rhino.RhinoApp.Wait();System.Threading.Thread.CurrentThread.Join(20)
        opened=user.OpenClipboard(None)
    report.setdefault('clipboard_observer_retries',[]).append(retries)
    check('Rhino native clipboard opens',opened)
    try:
        handle=user.GetClipboardData(fmt);check('Rhino V5 HGLOBAL available',bool(handle));size=kernel.GlobalSize(handle)
        check('bounded Rhino clipboard size',32<size<=512*1024*1024);pointer=kernel.GlobalLock(handle);check('Rhino global memory locks',bool(pointer))
        try:return ctypes.string_at(pointer,size)
        finally:kernel.GlobalUnlock(handle)
    finally:user.CloseClipboard()
def wait_process(process,seconds):
    started=time.time()
    while not process.HasExited and time.time()-started<seconds:
        Rhino.RhinoApp.Wait();System.Threading.Thread.Sleep(50)
    return process.HasExited
def launch_freecad(case_dir,config_file):
    start=System.Diagnostics.ProcessStartInfo();start.FileName='H:/FreeCAD-src/build/om9-phase3-sdk/bin/FreeCAD.exe'
    start.UseShellExecute=False;start.CreateNoWindow=True;start.WindowStyle=System.Diagnostics.ProcessWindowStyle.Hidden
    start.Arguments='-u "'+ntpath.join(case_dir,'user.cfg')+'" -s "'+ntpath.join(case_dir,'system.cfg')+'" "'+ntpath.join(root,'tests/modeling_phase3_clipboard_roundtrip.FCMacro')+'"'
    start.EnvironmentVariables['OM9_SMOKE_OUTPUT']=case_dir;start.EnvironmentVariables['OM9_CLIPBOARD_CASE_JSON']=config_file
    start.EnvironmentVariables['FREECAD_USER_HOME']=ntpath.join(case_dir,'profile')
    start.EnvironmentVariables['PATH']='H:/FreeCAD-src/.pixi/envs/default/Library/bin;H:/FreeCAD-src/.pixi/envs/default;'+os.environ['PATH']
    process=System.Diagnostics.Process.Start(start);check('owned FreeCAD roundtrip finished',wait_process(process,180))
    check('owned FreeCAD process exit zero',process.ExitCode==0)
    with open(ntpath.join(case_dir,'results.json'),'r') as stream:data=json.load(stream)
    check('owned FreeCAD clipboard gate '+case_dir,data.get('ok'));data['owned_application']={'pid':process.Id,'exit_code':process.ExitCode,'executable':start.FileName};
    with open(ntpath.join(case_dir,'results.json'),'w') as stream:json.dump(data,stream,indent=2)
    return data
try:
    check('Rhino5 actual process',str(Rhino.RhinoApp.Version).startswith('5.'))
    check('empty test document required',len(document_helpers['document_objects'](sc.doc))==0 and not sc.doc.Modified)
    owned_document=True
    with open(ntpath.join(root,'docs/validation/modeling-phase-1/application-evidence.json'),'r') as stream:accepted=json.load(stream)
    entries=[{'name':c['name'],'input':c['input']} for c in accepted['accepted_case_bindings']]
    entries.sort(key=lambda entry:entry['name']!='current-ring')
    entries.append({'name':'native-cm','input':'H:/FreeCAD-src/build/clipboard-native/modeling-fixtures/cm.3dm'})
    case_filter=os.environ.get('OM9_RHINO_CLIPBOARD_CASE_FILTER')
    if case_filter:
        wanted=set(case_filter.split(','));entries=[entry for entry in entries if entry['name'] in wanted]
        check('requested diagnostic fixtures exist',set(entry['name'] for entry in entries)==wanted)
    report['module_sha256']=digest('H:/FreeCAD-src/build/om9-phase3-sdk/bin/OpenMatrix9Gui.pyd')
    script_files=['tests/rhino5_phase3_clipboard_user.py','tests/rhino5_phase3_clipboard_bootstrap.py','tests/rhino5_modeling_exchange.py','tests/rhino5_modeling_document.py','tests/modeling_phase3_clipboard_roundtrip.FCMacro','tests/modeling_rhino5_saved_reimport.FCMacro','tests/phase12_timing.py']
    report['measurement']='Rhino5 GetArea/GetVolume relative/absolute integration tolerance1e-10; native adaptive SaveAs reimport oracle remains mandatory'
    report['acceptance_scope']='Actual native Copy/Paste/current geometry commands, validity and bounds; unavailable Rhino5 scalar probes recorded separately, all SaveAs outputs require independent native mass/topology reimport before acceptance'
    report['script_sha256']={name:digest(ntpath.join(root,name)) for name in script_files}
    for entry in entries:
        case_dir=ntpath.join(out,entry['name']);os.makedirs(case_dir);source=entry['input'];original_hash=digest(source)
        case_timings={}
        trace=Timing(case_timings,entry['name'],'Rhino source','Open')
        sc.doc.Modified=False
        check('actual Rhino Open '+entry['name'],trace.call('total',Rhino.RhinoApp.RunScript,helpers['rhino_file_command']('Open',source),False))
        all_roots=[o for o in document_helpers['document_objects'](sc.doc) if not o.Attributes.Mode==Rhino.DocObjects.ObjectMode.InstanceDefinitionObject]
        settings=document_helpers['document_settings']()
        settings.LockedObjects=False;settings.HiddenObjects=False;settings.VisibleFilter=True
        roots=list(sc.doc.Objects.GetObjectList(settings))
        sc.doc.Objects.UnselectAll()
        for obj in roots:check('Rhino selects visible unlocked source object',obj.Select(True)>0)
        trace=Timing(case_timings,entry['name'],'Rhino -> OM9','Copy')
        check('actual Rhino CopyToClipboard '+entry['name'],trace.call('total',Rhino.RhinoApp.RunScript,'_CopyToClipboard',False))
        payload=capture();check('actual raw V5 archive header',payload[:24]=='3D Geometry File Format ')
        source_payload=ntpath.join(case_dir,'rhino-copy.3dm')
        with open(source_payload,'wb') as stream:stream.write(payload)
        clipboard_model=Rhino.FileIO.File3dm.Read(source_payload)
        clipboard_roots=[o for o in clipboard_model.Objects if o.Attributes.Mode!=Rhino.DocObjects.ObjectMode.InstanceDefinitionObject]
        check('Rhino clipboard contains exactly selected UUIDs',set(str(o.Id) for o in roots)==set(str(o.Attributes.ObjectId) for o in clipboard_roots))
        clipboard_model.Dispose()
        config={'name':entry['name'],'source_count':len(roots),'source_all_root_count':len(all_roots),'selected_ids':[str(o.Id) for o in roots],'rhino_payload_sha256':hashlib.sha256(payload).hexdigest(),'rhino_payload_file':source_payload}
        config_file=ntpath.join(case_dir,'case.json')
        with open(config_file,'w') as stream:json.dump(config,stream,indent=2)
        trace=Timing(case_timings,entry['name'],'Rhino <-> OM9','Owned OM9 session')
        host=trace.call('startup_workflow_validation_exit',launch_freecad,case_dir,config_file)
        check('FreeCAD bound candidate binary',host['module_sha256']==report['module_sha256'])
        check('OM9 Copy survives source application exit',hashlib.sha256(capture()).hexdigest()==host['payload_sha256'])
        # Clear only this owned document using Rhino commands, retain clipboard.
        sc.doc.Modified=False
        check('Rhino new owned document',Rhino.RhinoApp.RunScript('_-New _None _Enter',False))
        sc.doc.AdjustModelUnitSystem(Rhino.UnitSystem.Millimeters,False)
        trace=Timing(case_timings,entry['name'],'OM9 -> Rhino','Paste')
        check('actual Rhino Paste '+entry['name'],trace.call('total',Rhino.RhinoApp.RunScript,'_Paste',False))
        saved=ntpath.join(case_dir,'rhino-pasted-saved.3dm')
        trace=Timing(case_timings,entry['name'],'Rhino output','SaveAs')
        check('Rhino actual SaveAs '+entry['name'],trace.call('total',Rhino.RhinoApp.RunScript,helpers['rhino_file_command']('SaveAs',saved),False))
        received=helpers['root_rows'](helpers['object_rows'](document_helpers['document_objects'](sc.doc),helpers['document_definitions'](sc.doc)))
        expected=host['expected'];check('Rhino Paste object count '+entry['name'],len(received)==len(expected))
        by_name={r['name']:r for r in received};check('unique current geometry names',len(by_name)==len(received))
        for reference in expected:
            actual=by_name[reference['label']]
            if 'bounds' in reference:check('Rhino Paste bounds '+reference['label'],all(abs(a-b)<.001 for a,b in zip(reference['bounds'],helpers['geometry_bounds'](actual))))
            for field in ['area','volume','length']:
                if field in reference and field in actual:check('Rhino Paste '+field+' '+reference['label'],abs(reference[field]-actual[field])<=max(.001,.0001*abs(reference[field])))
            if 'points' in reference:check('Rhino Paste PointCloud fields',reference['points']==actual['points'])
            if reference['kind']==4:check('Rhino Paste Mesh triangle count',reference['faces']==actual['mesh_faces'])
            if reference['kind']==4:check('Rhino Paste Mesh canonical geometry',reference['mesh_geometry_sha256']==actual['mesh_geometry_sha256'])
        check('input source immutable '+entry['name'],digest(source)==original_hash)
        report['cases'].append(dict(name=entry['name'],ok=True,input=source,input_sha256=original_hash,output=saved,output_sha256=digest(saved),freecad_report=ntpath.join(case_dir,'results.json'),rhino_payload_sha256=config['rhino_payload_sha256'],host_payload_sha256=host['payload_sha256'],rhino_rows=received,timings=case_timings.get('timings',[]),timing_log_errors=case_timings.get('timing_log_errors',[])))
    after_script_sha256={name:digest(ntpath.join(root,name)) for name in script_files}
    check('test scripts immutable during gate',report['script_sha256']==after_script_sha256)
    report['ok']=True
except:
    report['error']=traceback.format_exc()
    if 'case_timings' in locals():report['interrupted_case_timings']=case_timings
    try:report['clipboard_failure_state']=clipboard_state()
    except:report['clipboard_diagnostic_error']=traceback.format_exc()
finally:
    report['termination_mode']='owned process exits after durable report' if owned_document and os.environ.get('OM9_RHINO_CLIPBOARD_OUTPUT') else 'manual Rhino remains open'
    report['command_history_tail']=unicode(Rhino.RhinoApp.CommandHistoryWindowText)[-8000:]
    with open(ntpath.join(out,'rhino5-results.json'),'w') as stream:json.dump(report,stream,indent=2)
    if owned_document:sc.doc.Modified=False
    print('Rhino5 clipboard: '+str(report['ok'])+'; cases='+str(len(report['cases']))+'; '+out)
    if owned_document and os.environ.get('OM9_RHINO_CLIPBOARD_OUTPUT'):
        # Only the dedicated launcher process, after all owned files/reports
        # have closed. Rhino's script thread has no WinForms timer message loop.
        System.Environment.Exit(0)
