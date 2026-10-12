"""Harness boundary tests; native application acceptance is separate."""
import importlib.util,json,tempfile,unittest,ast,types,traceback,ntpath
from pathlib import Path

ROOT=Path(__file__).parent

class SupportTests(unittest.TestCase):
    def support(self):
        path=ROOT/'matrix_om9_handoff_support.py'
        self.assertTrue(path.is_file(),'Two-application harness support is missing')
        spec=importlib.util.spec_from_file_location('handoff_support',path)
        module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
        return module

    def test_wrong_run_sequence_or_action_never_dispatches(self):
        s=self.support();run='a'*32
        good={'version':1,'run_id':run,'seq':3,'action':'paste'}
        self.assertEqual(s.validate_request(good,run,3),'paste')
        for field,value in [('run_id','b'*32),('seq',2),('action','exec'),('version',99)]:
            bad=dict(good);bad[field]=value
            with self.assertRaises(ValueError):s.validate_request(bad,run,3)

    def test_missing_empty_layer_and_wrong_rgb_are_real_transfer_failures(self):
        s=self.support()
        source={'layers':[{'path':['Metal'],'rgb':[1,2,3],'locked':False,'visible':True,'persistent_locked':False,'persistent_visible':True},
                          {'path':['Empty'],'rgb':[4,5,6],'locked':True,'visible':False,'persistent_locked':True,'persistent_visible':False}],
                'active':['Metal'],'objects':[]}
        received=json.loads(json.dumps(source));self.assertFalse(s.transfer_errors(source,received))
        received['layers'].pop();self.assertIn('missing layer Empty',s.transfer_errors(source,received))
        received=json.loads(json.dumps(source));received['layers'][0]['rgb']=[7,8,9]
        self.assertIn('layer Metal rgb',s.transfer_errors(source,received))

    def test_geometry_is_compared_with_tolerance_and_object_ids_are_not_reused(self):
        s=self.support()
        obj={'name':'Curve','layer':['Metal'],'locked':False,'visible':True,'color_source':'ByObject','rgb':[11,22,33],'bounds':[1,2,3,4,5,6]}
        source={'layers':[],'active':None,'objects':[obj]};target=json.loads(json.dumps(source))
        target['objects'][0]['bounds'][0]+=0.00001
        self.assertFalse(s.transfer_errors(source,target,tolerance=0.001))
        target['objects'][0]['bounds'][0]+=1
        self.assertIn('object Curve bounds',s.transfer_errors(source,target))
        target=json.loads(json.dumps(source));target['objects'][0]['locked']=True
        self.assertIn('object Curve locked',s.transfer_errors(source,target))

    def test_incomplete_report_can_never_be_a_two_application_pass(self):
        s=self.support();report={'checks':[{'name':'API ready','passed':True}],'directions':{},'cleanup_ok':True}
        self.assertFalse(s.accepted(report))
        report['directions']={direction:dict(copy=True,paste=True,undo=True,redo=True,transfer=True) for direction in ('matrix_to_om9','om9_to_matrix')}
        self.assertTrue(s.accepted(report))
        report['checks'].append({'name':'empty layer missing','passed':False});self.assertFalse(s.accepted(report))

    def test_user_scope_requires_om9_undo_but_matrix_transfer_only(self):
        s=self.support();report=dict(undo_scope='om9_only',checks=[dict(name='handoff',passed=True)],cleanup_ok=True,
            directions=dict(matrix_to_om9=dict(copy=True,paste=True,transfer=True,undo=True,redo=True),
                om9_to_matrix=dict(copy=True,paste=True,transfer=True)))
        self.assertTrue(s.accepted(report),'Matrix Undo/Redo is outside the requested scope')
        report['directions']['matrix_to_om9']['undo']=False;self.assertFalse(s.accepted(report))
        report['directions']['matrix_to_om9']['undo']=True
        report['directions']['om9_to_matrix']['transfer']=False;self.assertFalse(s.accepted(report))

    def test_repeated_matrix_script_does_not_reuse_cached_acceptance_policy(self):
        import sys
        cached=types.ModuleType('matrix_om9_handoff_support');cached.accepted=lambda report:False
        old=sys.modules.get('matrix_om9_handoff_support');sys.modules['matrix_om9_handoff_support']=cached
        try:
            tree=ast.parse((ROOT/'rhino5_verify_matrix_om9_handoff.py').read_text(encoding='utf-8'))
            definition=next((node for node in tree.body if isinstance(node,ast.FunctionDef) and node.name=='load_support'),None)
            if definition is None:
                import importlib
                loaded=importlib.import_module('matrix_om9_handoff_support')
            else:
                namespace={'types':types}
                exec(compile(ast.Module(body=[definition],type_ignores=[]),'current-support-loader','exec'),namespace)
                loaded=namespace['load_support'](str(ROOT/'matrix_om9_handoff_support.py'))
            report=dict(undo_scope='om9_only',checks=[dict(name='all real checks',passed=True)],cleanup_ok=True,
                directions=dict(matrix_to_om9=dict(copy=True,paste=True,transfer=True,undo=True,redo=True),
                    om9_to_matrix=dict(copy=True,paste=True,transfer=True)))
            self.assertTrue(loaded.accepted(report),'Stale cached policy must not require Matrix Undo')
            report['directions']['matrix_to_om9']['redo']=False;self.assertFalse(loaded.accepted(report))
            self.assertIs(sys.modules['matrix_om9_handoff_support'],cached,'Do not mutate unrelated engine module cache')
        finally:
            if old is None:sys.modules.pop('matrix_om9_handoff_support',None)
            else:sys.modules['matrix_om9_handoff_support']=old

    def test_current_handoff_never_runs_matrix_undo_commands(self):
        tree=ast.parse((ROOT/'rhino5_verify_matrix_om9_handoff.py').read_text(encoding='utf-8'))
        main=next(node for node in tree.body if isinstance(node,ast.FunctionDef) and node.name=='main')
        calls=[node.args[0].value for node in ast.walk(main) if isinstance(node,ast.Call) and isinstance(node.func,ast.Name)
            and node.func.id=='run_native' and node.args and isinstance(node.args[0],ast.Constant)]
        self.assertNotIn('_Undo',calls);self.assertNotIn('_Redo',calls)
        self.assertIn('_CopyToClipboard',calls);self.assertIn('_Paste',calls)

    def test_atomic_messages_are_bounded_and_owned(self):
        s=self.support()
        with tempfile.TemporaryDirectory(dir=str(ROOT)) as directory:
            path=Path(directory)/'request.json';data={'version':1,'run_id':'a'*32,'seq':1,'action':'snapshot'}
            s.write_json(str(path),data);self.assertEqual(s.read_json(str(path)),data)
            path.write_bytes(b'x'*(s.MAX_MESSAGE+1))
            with self.assertRaises(ValueError):s.read_json(str(path))

    def test_rhino5_model_iterator_includes_normal_locked_and_hidden_models(self):
        # Installed Rhino5 XML: IdefObjects=True means ONLY definition members,
        # not "include definitions". A real model Paste must not disappear.
        class Settings:
            IdefObjects=False
        models=['normal BRep','locked curve','hidden BRep']
        def enumerate_models(settings):
            if settings.IdefObjects:return iter(['definition member'])
            self.assertTrue(settings.NormalObjects and settings.LockedObjects and settings.HiddenObjects)
            return iter(models)
        doc=types.SimpleNamespace(Objects=types.SimpleNamespace(GetObjectList=enumerate_models))
        tree=ast.parse((ROOT/'rhino5_verify_matrix_om9_handoff.py').read_text(encoding='utf-8'))
        definition=next(node for node in tree.body if isinstance(node,ast.FunctionDef) and node.name=='objects')
        namespace={'Rhino':types.SimpleNamespace(DocObjects=types.SimpleNamespace(ObjectEnumeratorSettings=Settings))}
        exec(compile(ast.Module(body=[definition],type_ignores=[]),'actual-rhino5-model-iterator','exec'),namespace)
        self.assertEqual(list(namespace['objects'](doc)),models)

    def test_populated_or_saved_matrix_stops_before_launch_or_geometry_read(self):
        s=self.support();tree=ast.parse((ROOT/'rhino5_verify_matrix_om9_handoff.py').read_text(encoding='utf-8'))
        main=next(node for node in tree.body if isinstance(node,ast.FunctionDef) and node.name=='main')
        for modified,path,populated in ((True,'',False),(False,'model.3dm',False),(False,'',True)):
            report={'checks':[],'directions':{}};written=[]
            doc=types.SimpleNamespace(Modified=modified,Path=path)
            def check(name,value,details=None):report['checks'].append(dict(name=name,passed=bool(value)))
            namespace={'original':None,'original_doc':None,'companion_pid':None,'finished':False,'report':report,
                'document':lambda:doc,'os':types.SimpleNamespace(environ={}),
                'objects':lambda doc:iter([object()] if populated else []),
                'launch':lambda:self.fail('Unsafe document must stop before application launch'),
                'fixture':lambda:self.fail('Unsafe document must stop before geometry mutation'),
                'cleanup':lambda:True,'check':check,'traceback':traceback,'ntpath':ntpath,'output':'unused',
                'log':lambda text:None,'checkpoint':lambda name,value:None,'support':types.SimpleNamespace(accepted=s.accepted,write_json=lambda path,value:written.append(value))}
            exec(compile(ast.Module(body=[main],type_ignores=[]),'actual-handoff-guard','exec'),namespace)
            self.assertFalse(list(namespace['main']()))
            self.assertEqual(len(written),1);self.assertFalse(written[0]['application_accepted'])
            self.assertIn('fresh blank unsaved',written[0]['error'])

    def test_matrix_idle_driver_returns_control_without_exiting_application(self):
        class Idle:
            def __init__(self):self.handlers=[]
            def __iadd__(self,callback):self.handlers.append(callback);return self
            def __isub__(self,callback):self.handlers.remove(callback);return self
        idle=Idle();steps=[];state={'command':True}
        def work():steps.append('first');yield None;steps.append('finished')
        tree=ast.parse((ROOT/'rhino5_verify_matrix_om9_handoff.py').read_text(encoding='utf-8'))
        start=next(node for node in tree.body if isinstance(node,ast.FunctionDef) and node.name=='start')
        namespace={'os':types.SimpleNamespace(makedirs=lambda path:None),'output':'unused','handler':None,'bootstrap_queued':False,'main':work,
            'log':lambda text:None,'last_progress':0,'resuming':False,'time':types.SimpleNamespace(time=lambda:100),
            'System':types.SimpleNamespace(EventHandler=lambda callback:callback),
            'Rhino':types.SimpleNamespace(Commands=types.SimpleNamespace(Command=types.SimpleNamespace(InCommand=lambda:state['command'])),
                RhinoApp=types.SimpleNamespace(Idle=idle,RunScript=lambda *args:self.fail('Idle driver must not exit/load application')))}
        exec(compile(ast.Module(body=[start],type_ignores=[]),'actual-matrix-driver','exec'),namespace)
        namespace['start']();callback=idle.handlers[0];callback(None,None);self.assertFalse(steps)
        state['command']=False;callback(None,None);callback(None,None)
        self.assertEqual(steps,['first','finished']);self.assertFalse(idle.handlers)

    def test_idle_reentrancy_and_failure_leave_a_terminal_failure_report(self):
        class Idle:
            def __init__(self):self.handlers=[]
            def __iadd__(self,callback):self.handlers.append(callback);return self
            def __isub__(self,callback):self.handlers.remove(callback);return self
        idle=Idle();writes=[];steps=[]
        def work():
            steps.append('entered')
            idle.handlers[0](None,None)
            raise RuntimeError('final report writer failed')
            yield None
        tree=ast.parse((ROOT/'rhino5_verify_matrix_om9_handoff.py').read_text(encoding='utf-8'))
        start=next(node for node in tree.body if isinstance(node,ast.FunctionDef) and node.name=='start')
        namespace={'os':types.SimpleNamespace(makedirs=lambda path:None),'output':'unused','handler':None,'bootstrap_queued':False,'main':work,
            'log':lambda text:None,'last_progress':0,'time':types.SimpleNamespace(time=lambda:100),'traceback':traceback,
            'terminal_failure':lambda error:writes.append(error),'resuming':False,
            'System':types.SimpleNamespace(EventHandler=lambda callback:callback),
            'Rhino':types.SimpleNamespace(Commands=types.SimpleNamespace(Command=types.SimpleNamespace(InCommand=lambda:False)),
                RhinoApp=types.SimpleNamespace(Idle=idle))}
        exec(compile(ast.Module(body=[start],type_ignores=[]),'actual-matrix-driver','exec'),namespace)
        namespace['start']();idle.handlers[0](None,None)
        self.assertEqual(steps,['entered']);self.assertFalse(idle.handlers)
        self.assertEqual(len(writes),1);self.assertIn('final report writer failed',writes[0])

    def test_failed_run_recovery_refuses_foreign_or_changed_objects_before_cleanup(self):
        failed=Path('H:/FreeCAD-src/build/matrix-om9-handoff/303c98e676534ffb91c894c130fba7b7')
        observed=json.loads((failed/'read-only-a8ad89a847b44092a223ac6bbce4ae70/matrix-observation.json').read_text())
        expected=json.loads((failed/'response-001.json').read_text())['result']['semantic']['objects']+json.loads((failed/'response-004.json').read_text())['result']['semantic']['objects']
        current=json.loads(json.dumps(observed['current_state']))
        current['raw']['objects']=[dict(row,id=str(i)) for i,row in enumerate(expected)]
        tree=ast.parse((ROOT/'rhino5_recover_matrix_handoff.py').read_text(encoding='utf-8'))
        definition=next(node for node in tree.body if isinstance(node,ast.FunctionDef) and node.name=='validate')
        by_id=dict((row['id'],row) for row in current['raw']['objects'])
        def find_native(identity):
            row=by_id[identity]
            return types.SimpleNamespace(Attributes=types.SimpleNamespace(GetUserString=lambda key:row['name'].replace(' ','')))
        doc=types.SimpleNamespace(DocumentId=observed['current_state']['document_id'],Path='',Objects=types.SimpleNamespace(Find=find_native))
        namespace={'Rhino':types.SimpleNamespace(RhinoDoc=types.SimpleNamespace(ActiveDoc=doc)),
            'observed':observed,'result':{},'namespace':{'snapshot':lambda:current},'support':self.support(),
            'ntpath':ntpath,'FAILED':str(failed),'System':types.SimpleNamespace(Guid=lambda value:value)}
        exec(compile(ast.Module(body=[definition],type_ignores=[]),'actual-owned-recovery-preflight','exec'),namespace)
        self.assertEqual(len(namespace['validate']()[3]),30)
        for mutation in ('foreign','bounds','saved','palette','tag'):
            saved=json.loads(json.dumps(current));doc.Path=''
            if mutation=='foreign':current['raw']['objects'].append(dict(current['raw']['objects'][0],id='foreign'))
            elif mutation=='bounds':current['raw']['objects'][0]['bounds'][0]+=10
            elif mutation=='saved':doc.Path='user-model.3dm'
            elif mutation=='palette':current['raw']['layers'][0]['rgb']=[0,0,0]
            else:doc.Objects.Find=lambda identity:types.SimpleNamespace(Attributes=types.SimpleNamespace(GetUserString=lambda key:'wrong-owner'))
            with self.assertRaises(RuntimeError,msg=mutation):namespace['validate']()
            current.clear();current.update(saved);doc.Path='';doc.Objects.Find=find_native

if __name__=='__main__':unittest.main()
