"""Native diagnostic in an owned test document; current or isolated host mode."""
import os,json,traceback,hashlib,struct,time
import Rhino,System,clr
from System.Diagnostics import Process
report={'checks':[],'scope':'actual Matrix load/architecture, persistent/current layer and command-owned Undo/rollback API diagnostic; no product handoff claim'}
checkpoint_output=None
def checkpoint():
    if checkpoint_output:
        with open(os.path.join(checkpoint_output,'probe-checkpoint.json'),'w') as stream:json.dump(report,stream,indent=2)
def check(name,value):
    report['checks'].append({'name':name,'passed':bool(value)})
    checkpoint()
    if not value:raise RuntimeError(name)

def load_owned_probe(identity,path,current_test):
    if current_test:
        if Rhino.PlugIns.PlugIn.Find(identity) is not None:
            return {'loaded':True,'method':'already loaded diagnostic owner'}
        # Rhino5 caches installed identities at startup. Registry preparation
        # alone cannot make a first-use plugin available in an existing host.
        # Use Rhino5's native path loader outside the bootstrap command.
        result=bool(Rhino.RhinoApp.RunScript('_-Options _PlugIns _Load "'+path+'" _Enter',False))
        return {'loaded':Rhino.PlugIns.PlugIn.Find(identity) is not None,
                'native_load_return':result,'method':'_-Options _PlugIns _Load'}
    return {'loaded':bool(Rhino.PlugIns.PlugIn.LoadPlugIn(identity)),
            'method':'Rhino.PlugIns.PlugIn.LoadPlugIn(Guid)'}
def main():
    global checkpoint_output
    read_only=bool(globals().get('OM9_LAYER_API_DIRECT_READ_ONLY',False))
    current_test=bool(globals().get('OM9_LAYER_API_CURRENT_COMMAND_TEST',False))
    if current_test:
        report['mode']='current_matrix_blank_command_test'
        report['scope']='actual current Matrix process and dedicated blank document; command Undo/rollback only, no product handoff claim'
    if read_only:
        report['scope']='read-only actual Matrix host/loaded modules and detached layer APIs; no command Undo/rollback or product handoff claim'
        report['mode']='direct_matrix_read_only'
    run_id=os.environ.get('OM9_LAYER_API_RUN_ID');output=os.environ.get('OM9_LAYER_API_OUTPUT')
    if not run_id or not output:raise RuntimeError('Requires owned diagnostic entrypoint')
    checkpoint_output=output
    # Identity is available even when an early API check fails.
    report.update(run_id=run_id,pid=Process.GetCurrentProcess().Id)
    def progress(stage):
        report['stage']=stage
        checkpoint()
        with open(os.path.join(output,'progress.json'),'w') as stream:json.dump({'run_id':run_id,'pid':Process.GetCurrentProcess().Id,'stage':stage},stream)
    try:
        progress('Rhino Python bootstrap entered')
        doc=Rhino.RhinoDoc.ActiveDoc
        check('current Matrix document exists' if read_only or current_test else 'fresh isolated document exists',doc is not None)
        if current_test:
            check('same current Matrix document as entrypoint',int(doc.DocumentId)==globals().get('OM9_LAYER_API_EXPECTED_DOCUMENT_ID'))
            check('current Matrix document is fresh blank and unsaved',not doc.Modified and not doc.Path)
        def document_stamp():
            current=Rhino.RhinoDoc.ActiveDoc
            return {'document_id':int(current.DocumentId),'modified':bool(current.Modified),
                    'layer_count':int(current.Layers.Count),'current_layer_index':int(current.Layers.CurrentLayerIndex)}
        if read_only:report['document_before']=document_stamp()
        settings=Rhino.DocObjects.ObjectEnumeratorSettings()
        settings.NormalObjects=True;settings.LockedObjects=True;settings.HiddenObjects=True
        settings.ActiveObjects=True;settings.ReferenceObjects=True;settings.IncludeLights=True;settings.IncludeGrips=True
        # Rhino 5 ObjectTable has no Count. Stop at the first object without
        # extracting geometry, tessellating or materializing a large list.
        if not read_only:
            has_objects=False
            for obj in doc.Objects.GetObjectList(settings):
                has_objects=True
                break
            check('test document contains no user geometry',not has_objects and not doc.Modified)
        process=Process.GetCurrentProcess();report['pid']=process.Id;report['executable']=process.MainModule.FileName
        with open(report['executable'],'rb') as stream:
            if struct.unpack('<H',stream.read(2))[0]!=0x5a4d:raise RuntimeError('Host executable is not a PE image')
            stream.seek(0x3c);offset=struct.unpack('<I',stream.read(4))[0];stream.seek(offset)
            if struct.unpack('<I',stream.read(4))[0]!=0x4550:raise RuntimeError('Host PE signature missing')
            machine=struct.unpack('<H',stream.read(2))[0]
        report['actual_host_pe_machine']=machine;report['actual_host_architecture']={0x8664:'AMD64',0x14c:'I386',0xaa64:'ARM64'}.get(machine,'unknown')
        report['run_id']=run_id;report['pointer_bytes']=System.IntPtr.Size;report['clr_version']=str(System.Environment.Version)
        report['rhino_version']=str(Rhino.RhinoApp.Version);report['rhino_build_date']=str(Rhino.RhinoApp.BuildDate)
        assembly=clr.GetClrType(Rhino.RhinoDoc).Assembly
        report['rhino_common']={'location':assembly.Location,'name':assembly.FullName,'image_runtime':assembly.ImageRuntimeVersion}
        installed=Rhino.PlugIns.PlugIn.GetInstalledPlugIns()
        report['matrix_installations']=[{'id':str(key),'name':installed[key]} for key in installed.Keys if 'matrix' in installed[key].lower()]
        core=System.Guid('7faef691-c68e-462a-af89-899bff11a80f')
        frontend=System.Guid('e9c738e7-3303-405d-a42b-881d52b01bdb')
        check('registered Matrix frontend is DotMatrix',installed.ContainsKey(frontend) and installed[frontend]=='DotMatrix')
        # This identity is from the installed native plugin registry, not a folder-bitness guess.
        if read_only or current_test:
            progress('Inspect already loaded Matrix components without LoadPlugIn')
            exists=clr.GetClrType(Rhino.PlugIns.PlugIn).GetMethod('PlugInExists')
            core_arguments=System.Array[System.Object]([core,System.Boolean(False),System.Boolean(False)])
            report['core_load_result']=bool(exists.Invoke(None,core_arguments) and core_arguments[1])
        else:
            progress('Loading registered gvMatrixCore in owned empty host')
            report['core_load_result']=bool(Rhino.PlugIns.PlugIn.LoadPlugIn(core))
            progress('Loading registered DotMatrix frontend in owned empty host')
            report['frontend_load_result']=bool(Rhino.PlugIns.PlugIn.LoadPlugIn(frontend))
        check('registered gvMatrixCore is loaded',report['core_load_result'])
        # GetLoadedPlugIn is absent from this installed RhinoCommon SDK. Use
        # its public PlugInExists(Guid, out loaded, out loadProtected) contract.
        exists=clr.GetClrType(Rhino.PlugIns.PlugIn).GetMethod('PlugInExists')
        arguments=System.Array[System.Object]([frontend,System.Boolean(False),System.Boolean(False)])
        present=bool(exists.Invoke(None,arguments))
        if read_only or current_test:report['frontend_load_result']=bool(present and arguments[1])
        report['frontend_registration']={'id':str(frontend),'present':present,'loaded':bool(arguments[1]),'load_protected':bool(arguments[2])}
        progress('Matrix load returned; inspect actual native modules')
        modules=[]
        for module in process.Modules:
            if os.path.basename(module.FileName).lower() in ('matrix.rhp','gvmatrixcore.rhp'):
                path=module.FileName
                with open(path,'rb') as stream:digest=hashlib.sha256(stream.read()).hexdigest()
                modules.append({'path':path,'sha256':digest})
        report['matrix_modules']=modules
        # Matrix.rhp is the managed DotMatrix frontend. Managed assembly load
        # evidence comes from the actual CLR, not just native module enumeration.
        assemblies=[]
        for loaded in System.AppDomain.CurrentDomain.GetAssemblies():
            if loaded.IsDynamic:continue
            path=loaded.Location
            if path and os.path.basename(path).lower()=='matrix.rhp':
                with open(path,'rb') as stream:digest=hashlib.sha256(stream.read()).hexdigest()
                assemblies.append({'path':path,'name':loaded.FullName,'sha256':digest})
        report['matrix_frontend_assemblies']=assemblies
        names=set(os.path.basename(m['path']).lower() for m in modules)
        expected=os.path.normcase(os.path.abspath(r'C:\Program Files (x86)\Matrix90\Plugins\Matrix.rhp'))
        frontend_file=any(os.path.normcase(os.path.abspath(a['path']))==expected for a in assemblies)
        report['actual_matrix_loaded']=bool(report['core_load_result'] and report['frontend_load_result'] and present and arguments[1] and frontend_file and 'gvmatrixcore.rhp' in names)
        check('actual registered DotMatrix frontend and gvMatrixCore loaded',report['actual_matrix_loaded'])
        report['matrix_command_counts']={'core':len(Rhino.PlugIns.PlugIn.GetEnglishCommandNames(core)),'frontend':len(Rhino.PlugIns.PlugIn.GetEnglishCommandNames(frontend))}
        check('actual host pointer width is reported',System.IntPtr.Size in (4,8))
        layer=Rhino.DocObjects.Layer();layer.Name='OM9 detached API probe';layer.ParentLayerId=System.Guid.NewGuid()
        initial={'locked':layer.IsLocked,'visible':layer.IsVisible,'persistent_locked':layer.GetPersistentLocking(),'persistent_visible':layer.GetPersistentVisibility()}
        # Persistent getters report the current state for an unlocked/visible
        # child. Test the desired parent-restoration state while constrained.
        layer.IsLocked=True;layer.IsVisible=False
        layer.SetPersistentLocking(True);layer.SetPersistentVisibility(False)
        report['detached_persistent']={'initial':initial,'constrained':{'locked':layer.IsLocked,'visible':layer.IsVisible,'persistent_locked':layer.GetPersistentLocking(),'persistent_visible':layer.GetPersistentVisibility()},'presence_bit_proven':False}
        check('installed persistent setters and getters operate',layer.GetPersistentLocking() and not layer.GetPersistentVisibility())
        layer.SetPersistentLocking(False);layer.SetPersistentVisibility(True)
        report['detached_persistent']['desired_opposite']={'locked':layer.IsLocked,'visible':layer.IsVisible,'persistent_locked':layer.GetPersistentLocking(),'persistent_visible':layer.GetPersistentVisibility()}
        check('persistent child desired state differs from constrained state',not layer.GetPersistentLocking() and layer.GetPersistentVisibility())
        layer.UnsetPersistentLocking();layer.UnsetPersistentVisibility()
        report['detached_persistent']['after_unset']={'locked':layer.IsLocked,'visible':layer.IsVisible,'persistent_locked':layer.GetPersistentLocking(),'persistent_visible':layer.GetPersistentVisibility()}
        check('unset persistent child state falls back to current state',layer.GetPersistentLocking()==layer.IsLocked and layer.GetPersistentVisibility()==layer.IsVisible)
        if read_only:
            report['document_after']=document_stamp()
            check('read-only inspection leaves current document stamp unchanged',report['document_before']==report['document_after'])
            report.update(ok=True,read_only_api_passed=True,command_undo_verified=False,full_api_gate_passed=False,
                pending=['command-owned Undo/Redo and injected rollback','full two-way Matrix/OM9 layer handoff'])
            return
        record=doc.BeginUndoRecord('OM9 API ownership observation')
        report['gate_command_in_progress']=bool(Rhino.Commands.Command.InCommand())
        check('command gate runs outside the bootstrap ScriptRunner',not report['gate_command_in_progress'])
        report['bootstrap_begin_undo_record']=int(record);report['bootstrap_undo_recording_active']=bool(doc.UndoRecordingIsActive)
        if record:check('owned observation record closes',doc.EndUndoRecord(record))
        # The diagnostic is compiled against this installed primary SDK after
        # AMD64/CLR4 were proven by an actual Matrix run. It is not product logic.
        probe=r'H:\FreeCAD-src\build\om9-layer-edits\tests\bin\current\Rhino5LayerCurrentProbe.rhp' if current_test else r'H:\FreeCAD-src\build\om9-layer-edits\tests\bin\Rhino5LayerCommandProbe.rhp'
        check('native command diagnostic exists',os.path.isfile(probe))
        command_names=['OM9LayerApiPrepareFixture','OM9LayerApiApplyProbe','OM9LayerApiFailLayerProbe','OM9LayerApiFailGeometryProbe','OM9LayerApiSample']
        expected_ids=[System.Guid(value) for value in ['569d2757-27f8-4d90-a694-22e46a7ce008','381fd2c0-28ed-47b5-aa2d-1ee387e3d4bc','e63a8c93-494f-4b79-9da7-8209c5b5987d','29cfe9f2-b107-4d94-b2b7-d9f15b7d5f94','b4d9b9f5-b2d7-4b1d-badc-796df5fc2232']]
        if current_test:
            command_names=[name.replace('OM9LayerApi','OM9LayerDirectApi') for name in command_names]+['OM9LayerDirectApiCleanup']
            expected_ids=[System.Guid(value) for value in ['84934f3c-8079-4935-ad93-29eb40d9a90d','23c7d980-04ce-4bde-97d7-5905b7d3dbe1','81e74825-ea49-4c78-8cda-17fb44aad986','2969b19c-e4f0-46a7-879b-4661c49044bc','42838e2e-cea6-4738-9722-08ff08aec4dc','b241b6e4-e7e7-4669-a04a-1cf4a7143e1a']]
        # First-use plugin registration may load it before this bootstrap. An
        # existing command is allowed only with our exact expected native ID.
        check('diagnostic command names do not collide',all(Rhino.Commands.Command.LookupCommandId(name,True) in (System.Guid.Empty,identity) for name,identity in zip(command_names,expected_ids)))
        progress('Load independent diagnostic command owner through installed public SDK')
        probe_id=System.Guid('28a7966b-2c55-4fe4-8cb6-0394316dc4ae' if current_test else 'd2f0f5d0-4e65-4b75-b4ef-8d6ca72be839')
        report['diagnostic_load']=load_owned_probe(probe_id,probe,current_test)
        check('owned diagnostic plugin loads with native identity',report['diagnostic_load']['loaded'])
        owner=Rhino.PlugIns.PlugIn.Find(probe_id)
        check('independent actual native diagnostic owner exists',owner is not None and owner.Id==probe_id)
        report['native_command_registration']={'owner_id':str(owner.Id),'owner_type':owner.GetType().FullName,'commands':list(Rhino.PlugIns.PlugIn.GetEnglishCommandNames(probe_id)),'method':report['diagnostic_load']['method'],'scope':'own diagnostic GUID; temporary registry entries removed after completion; Matrix registration unchanged'}
        loaded_probe=[]
        for loaded in System.AppDomain.CurrentDomain.GetAssemblies():
            if loaded.IsDynamic:continue
            path=loaded.Location
            if path and os.path.normcase(os.path.abspath(path))==os.path.normcase(os.path.abspath(probe)):
                with open(path,'rb') as stream:digest=hashlib.sha256(stream.read()).hexdigest()
                loaded_probe.append({'path':path,'name':loaded.FullName,'sha256':digest})
        report['command_probe_assemblies']=loaded_probe
        check('actual C# diagnostic assembly is loaded',len(loaded_probe)==1)
        if current_test:
            with open(probe,'rb') as stream:expected_hash=hashlib.sha256(stream.read()).hexdigest()
            check('current loaded diagnostic matches prepared binary',loaded_probe[0]['sha256']==expected_hash)
        check('all diagnostic commands register',all(Rhino.Commands.Command.IsCommand(name) for name in command_names))
        check('all diagnostic native IDs match',all(Rhino.Commands.Command.LookupCommandId(name,True)==identity for name,identity in zip(command_names,expected_ids)))
        def command(name,filename):
            if current_test:name=name.replace('OM9LayerApi','OM9LayerDirectApi')
            check('native command still targets owned test document '+name,Rhino.RhinoDoc.ActiveDoc.DocumentId==doc.DocumentId)
            path=os.path.join(output,filename+'.json')
            if os.path.isfile(path):os.remove(path)
            result=bool(Rhino.RhinoApp.RunScript('_'+name,False))
            check('command writes owned report '+name,os.path.isfile(path))
            with open(path,'r') as stream:data=json.load(stream)
            check('command report belongs to owned process '+name,data.get('run_id')==run_id and data.get('pid')==process.Id)
            data['run_script_return']=result
            check('command diagnostic passed '+name,data.get('ok',False))
            return data
        prepared=command('OM9LayerApiPrepareFixture','command-prepare')
        report['command_prepare']=prepared
        progress('Actual command apply and whole Undo/Redo')
        applied=command('OM9LayerApiApplyProbe','command-apply')
        report['command_apply']=applied
        check('actual command owns active Undo recording',applied['command_in_progress'] and applied['undo_recording_active'])
        check('command before matches existing fixture',json.loads(applied['before'])==json.loads(prepared['state']))
        after=json.loads(applied['after']);before=json.loads(prepared['state'])
        paths=dict((l['path'],l) for l in after['layers'])
        check('current switches before source is locked and hidden',paths['OM9 API Parent']['locked'] and not paths['OM9 API Parent']['visible'] and after['active']==paths['OM9 API Work']['id'])
        check('child desired flags survive inherited parent restriction',not paths['OM9 API Parent::Detail']['persistent_locked'] and paths['OM9 API Parent::Detail']['persistent_visible'] and paths['OM9 API Parent::Own locked child']['persistent_locked'])
        check('new geometry and palette are in actual receive command',len(after['objects'])==len(before['objects'])+1 and len(after['layers'])==len(before['layers'])+1)
        def wait_cursor_projection(events,created_by_redo):
            # Wait across actual GUI Idle turns. Never set the layer from the
            # test or pump/block the host thread to make a native test pass.
            path=os.path.join(output,'cursor-projection.json');deadline=time.time()+20
            while True:
                if os.path.isfile(path):
                    with open(path,'r') as stream:projection=json.load(stream)
                    check('cursor projection belongs to owned process',projection.get('run_id')==run_id and projection.get('pid')==process.Id)
                    if projection.get('events')==events:
                        check('native cursor projection passed',projection.get('ok',False))
                        check('cursor projects only outside commands and Undo recording',not projection['command_in_progress'] and not projection['undo_recording_active'])
                        check('cursor inverse follows Undo/Redo',projection['created_by_redo']==created_by_redo)
                        report.setdefault('cursor_projections',[]).append(projection)
                        return
                    check('cursor projection sequence is not ahead',projection.get('events',0)<events)
                if time.time()>deadline:raise RuntimeError('Native cursor projection did not complete within 20 seconds')
                yield None
        check('one native Undo dispatch returns',Rhino.RhinoApp.RunScript('_Undo',False))
        for pending in wait_cursor_projection(1,False):yield pending
        undone=command('OM9LayerApiSample','command-sample')
        report['command_undo']=undone
        check('one Undo restores existing full layer/geometry/active state',json.loads(undone['state'])==before)
        check('one native Redo dispatch returns',Rhino.RhinoApp.RunScript('_Redo',False))
        for pending in wait_cursor_projection(2,True):yield pending
        redone=command('OM9LayerApiSample','command-sample')
        report['command_redo']=redone
        check('one Redo restores layer update plus geometry together',json.loads(redone['state'])==after)
        # A second cycle detects a spurious Undo record or lost Redo caused by
        # the deferred active-layer projection itself.
        check('repeat native Undo dispatch returns',Rhino.RhinoApp.RunScript('_Undo',False))
        for pending in wait_cursor_projection(3,False):yield pending
        repeated_undo=command('OM9LayerApiSample','command-sample')
        check('repeat Undo preserves one-command ownership',json.loads(repeated_undo['state'])==before)
        check('repeat native Redo dispatch returns',Rhino.RhinoApp.RunScript('_Redo',False))
        for pending in wait_cursor_projection(4,True):yield pending
        repeated_redo=command('OM9LayerApiSample','command-sample')
        check('repeat Redo preserves full state',json.loads(repeated_redo['state'])==after)
        progress('Injected failure after layer update and geometry add')
        failures=[]
        for name,filename in [('OM9LayerApiFailLayerProbe','command-fail-layer'),('OM9LayerApiFailGeometryProbe','command-fail-geometry')]:
            failed=command(name,filename);failures.append(failed)
            check('failure rolls back before-image without blind Undo '+name,failed['rollback_equal'] and not failed['used_blind_undo'] and json.loads(failed['before'])==after and json.loads(failed['after'])==after)
            sample=command('OM9LayerApiSample','command-sample')
            check('failure leaves previous command content intact '+name,json.loads(sample['state'])==after)
        report['command_failures']=failures
        report.update(ok=True,command_undo_verified=True,full_api_gate_passed=True)
    except Exception as error:
        # IronPython traceback omits TargetInvocationException.InnerException.
        detail=error.ToString() if hasattr(error,'ToString') else str(error)
        report.update(ok=False,error=traceback.format_exc()+'\n'+detail,full_api_gate_passed=False)
    finally:
        if current_test:
            try:
                cleanup_id=System.Guid('b241b6e4-e7e7-4669-a04a-1cf4a7143e1a')
                if Rhino.Commands.Command.LookupCommandId('OM9LayerDirectApiCleanup',True)==cleanup_id:
                    Rhino.RhinoApp.RunScript('_OM9LayerDirectApiCleanup',False)
                    with open(os.path.join(output,'command-cleanup.json'),'r') as stream:cleanup=json.load(stream)
                    valid=cleanup.get('ok') and cleanup.get('run_id')==run_id and cleanup.get('pid')==Process.GetCurrentProcess().Id
                    report['current_document_cleanup']=cleanup
                    report['checks'].append({'name':'current blank Matrix fixture cleaned','passed':bool(valid)})
                    if not valid:raise RuntimeError('Current blank document cleanup failed')
                else:report['current_document_cleanup']={'skipped':True,'reason':'diagnostic command was not loaded; fixture cannot have been created'}
            except Exception:
                report.update(ok=False,full_api_gate_passed=False,cleanup_error=traceback.format_exc())
        if not read_only:report['command_history']=Rhino.RhinoApp.CommandHistoryWindowText
        with open(os.path.join(output,'matrix-api-prerequisite.json'),'w') as stream:json.dump(report,stream,indent=2)
        status='READ-ONLY PASS; command gate pending' if report.get('read_only_api_passed') else ('PASS' if report.get('full_api_gate_passed') else 'FAIL')
        Rhino.RhinoApp.WriteLine('OM9 Matrix API diagnostic: '+status+'; product handoff still unverified; '+output)
def start():
    global checkpoint_output
    if bool(globals().get('OM9_LAYER_API_DIRECT_READ_ONLY',False)):
        for pending in main():
            raise RuntimeError('Read-only inspection unexpectedly yielded native work')
        return
    # The owned isolated probe continues one step per native Idle turn, allowing
    # the diagnostic plugin to project its private cursor after Undo/Redo ends.
    run_id=os.environ.get('OM9_LAYER_API_RUN_ID');output=os.environ.get('OM9_LAYER_API_OUTPUT')
    if not run_id or not output:raise RuntimeError('Requires owned diagnostic entrypoint')
    checkpoint_output=output
    report.update(run_id=run_id,pid=Process.GetCurrentProcess().Id,
                  deferred_until_bootstrap_command_ends=True)
    checkpoint()
    work=main()
    def resume(sender,args):
        if Rhino.Commands.Command.InCommand():return
        try:next(work)
        except StopIteration:
            Rhino.RhinoApp.Idle-=handler
            if bool(globals().get('OM9_LAYER_API_CURRENT_COMMAND_TEST',False)):
                globals()['OM9_LAYER_API_ON_COMPLETE']()
            else:Rhino.RhinoApp.RunScript('_-Exit _No',False)
    handler=System.EventHandler(resume)
    Rhino.RhinoApp.Idle+=handler
    Rhino.RhinoApp.WriteLine('Owned diagnostic queued until bootstrap command ends.')
start()
