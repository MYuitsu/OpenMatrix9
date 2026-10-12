"""Offline host-boundary checks; actual Matrix API evidence needs the live app."""
import json
import io
import ast
import os
from pathlib import Path
import runpy
import sys
import tempfile
import types
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parent


class DirectApiTests(unittest.TestCase):
    def test_first_use_probe_loads_by_path_in_current_session(self):
        tree=ast.parse((ROOT/'rhino5_layer_api_prerequisite.py').read_text())
        functions=[node for node in tree.body if isinstance(node,ast.FunctionDef) and node.name=='load_owned_probe']
        self.assertEqual(len(functions),1,'Current-session path loader is required; a new registry entry needs a Rhino restart')
        calls=[];state={'owner':None}
        def run(command,echo):
            calls.append(command);state['owner']='native-owner';return True
        namespace={'Rhino':types.SimpleNamespace(RhinoApp=types.SimpleNamespace(RunScript=run),
            PlugIns=types.SimpleNamespace(PlugIn=types.SimpleNamespace(Find=lambda identity:state['owner'],
                LoadPlugIn=lambda identity:self.fail('First-use current session must not rely on cached registry registration'))))}
        exec(compile(ast.Module(body=functions,type_ignores=[]),'current-path-loader','exec'),namespace)
        result=namespace['load_owned_probe']('own-guid',r'H:\own probe.rhp',True)
        self.assertTrue(result['loaded']);self.assertTrue(result['native_load_return'])
        self.assertEqual(calls,['_-Options _PlugIns _Load "H:\\own probe.rhp" _Enter'])
        calls.clear()
        self.assertTrue(namespace['load_owned_probe']('own-guid',r'H:\own probe.rhp',True)['loaded'])
        self.assertFalse(calls)

    def test_current_blank_guard_rejects_saved_modified_and_populated_documents(self):
        tree=ast.parse((ROOT/'rhino5_verify_matrix_current.py').read_text())
        function=next(node for node in tree.body if isinstance(node,ast.FunctionDef) and node.name=='require_blank')
        reads=[]
        namespace={'Rhino':types.SimpleNamespace(DocObjects=types.SimpleNamespace(ObjectEnumeratorSettings=types.SimpleNamespace))}
        exec(compile(ast.Module(body=[function],type_ignores=[]),'current-blank-guard','exec'),namespace)
        objects=types.SimpleNamespace(GetObjectList=lambda settings:reads.append(settings) or [])
        for modified,path in ((True,''),(False,'user.3dm')):
            with self.assertRaises(RuntimeError):namespace['require_blank'](types.SimpleNamespace(Modified=modified,Path=path,Objects=objects))
        self.assertFalse(reads)
        doc=types.SimpleNamespace(Modified=False,Path='',Objects=objects,DocumentId=42)
        self.assertEqual(namespace['require_blank'](doc),42)
        doc.Objects=types.SimpleNamespace(GetObjectList=lambda settings:iter([object()]))
        with self.assertRaises(RuntimeError):namespace['require_blank'](doc)

    def test_current_entry_keeps_environment_until_idle_completion(self):
        tree=ast.parse((ROOT/'rhino5_verify_matrix_current.py').read_text())
        function=next(node for node in tree.body if isinstance(node,ast.FunctionDef) and node.name=='main')
        env={'OM9_LAYER_API_OUTPUT':'original-output'};before=dict(env);calls=[];pending=[];writes=[]
        run_id='a'*32
        def execute(path,namespace):
            self.assertEqual(env['OM9_LAYER_API_HOST_MODE'],'current_blank')
            self.assertTrue(namespace['OM9_LAYER_API_CURRENT_COMMAND_TEST'])
            self.assertEqual(namespace['OM9_LAYER_API_EXPECTED_DOCUMENT_ID'],42)
            pending.append(namespace['OM9_LAYER_API_ON_COMPLETE'])
        def stream(path,mode):
            if mode=='r':return io.StringIO(json.dumps({'run_id':run_id,'pid':123,'full_api_gate_passed':True,'checks':[]}))
            return io.StringIO()
        namespace={'os':types.SimpleNamespace(environ=env,path=os.path,makedirs=lambda path:None),
            'System':types.SimpleNamespace(Guid=types.SimpleNamespace(NewGuid=lambda:types.SimpleNamespace(ToString=lambda _:run_id))),
            'Rhino':types.SimpleNamespace(RhinoDoc=types.SimpleNamespace(ActiveDoc=object())),
            'Process':types.SimpleNamespace(GetCurrentProcess=lambda:types.SimpleNamespace(Id=123)),
            'require_blank':lambda doc:42,'registry':lambda mode,run:calls.append((mode,run)),
            'execfile':execute,'write_utf8':lambda path,text:writes.append(text),'open':stream,
            'json':json,'traceback':__import__('traceback'),'print':lambda *args:None}
        exec(compile(ast.Module(body=[function],type_ignores=[]),'current-entry','exec'),namespace)
        namespace['main']()
        self.assertEqual(calls,[('Prepare',run_id)])
        self.assertNotEqual(env,before)
        pending[0]()
        self.assertEqual(calls,[('Prepare',run_id),('Cleanup',run_id)])
        self.assertEqual(env,before);self.assertTrue(writes)
        # A missing terminal report must not strand this run's registration.
        def missing_report(path,mode):raise OSError('terminal report missing')
        namespace['open']=missing_report;calls.clear();pending.clear()
        namespace['main']();pending[0]()
        self.assertEqual(calls,[('Prepare',run_id),('Cleanup',run_id)])
        self.assertEqual(env,before)

    def test_current_session_driver_finishes_without_exiting_matrix(self):
        class Idle:
            def __init__(self):self.handlers=[]
            def __iadd__(self,callback):self.handlers.append(callback);return self
            def __isub__(self,callback):self.handlers.remove(callback);return self
        idle=Idle();completed=[];exits=[]
        def work():yield None
        tree=ast.parse((ROOT/'rhino5_layer_api_prerequisite.py').read_text())
        start=next(node for node in tree.body if isinstance(node,ast.FunctionDef) and node.name=='start')
        namespace={'main':work,'os':os,'report':{},'checkpoint':lambda:None,
            'OM9_LAYER_API_CURRENT_COMMAND_TEST':True,'OM9_LAYER_API_ON_COMPLETE':lambda:completed.append(True),
            'Rhino':types.SimpleNamespace(Commands=types.SimpleNamespace(Command=types.SimpleNamespace(InCommand=lambda:False)),RhinoApp=types.SimpleNamespace(Idle=idle,WriteLine=lambda _:None,RunScript=lambda cmd,echo:exits.append(cmd))),
            'System':types.SimpleNamespace(EventHandler=lambda callback:callback),
            'Process':types.SimpleNamespace(GetCurrentProcess=lambda:types.SimpleNamespace(Id=123))}
        with patch.dict(os.environ,{'OM9_LAYER_API_RUN_ID':'offline-run','OM9_LAYER_API_OUTPUT':str(ROOT)}):
            exec(compile(ast.Module(body=[start],type_ignores=[]),'current-idle-driver','exec'),namespace)
            namespace['start']();callback=idle.handlers[0]
            callback(None,None);self.assertFalse(completed)
            callback(None,None)
            self.assertEqual(completed,[True]);self.assertFalse(exits);self.assertFalse(idle.handlers)

    def test_native_probe_yields_between_idle_turns_and_exits_only_after_completion(self):
        class Idle:
            def __init__(self):self.handlers=[]
            def __iadd__(self,callback):self.handlers.append(callback);return self
            def __isub__(self,callback):self.handlers.remove(callback);return self
        idle=Idle();state={'in_command':True};steps=[];exits=[]
        def work():
            steps.append('Undo dispatched');yield None
            steps.append('native projection observed');yield None
            steps.append('terminal report written')
        tree=ast.parse((ROOT/'rhino5_layer_api_prerequisite.py').read_text())
        start=next(node for node in tree.body if isinstance(node,ast.FunctionDef) and node.name=='start')
        rhino=types.SimpleNamespace(Commands=types.SimpleNamespace(Command=types.SimpleNamespace(InCommand=lambda:state['in_command'])),
            RhinoApp=types.SimpleNamespace(Idle=idle,WriteLine=lambda _:None,RunScript=lambda command,echo:exits.append(command)))
        namespace={'main':work,'os':os,'report':{},'checkpoint':lambda:None,'Rhino':rhino,
            'System':types.SimpleNamespace(EventHandler=lambda callback:callback),
            'Process':types.SimpleNamespace(GetCurrentProcess=lambda:types.SimpleNamespace(Id=123))}
        with patch.dict(os.environ,{'OM9_LAYER_API_RUN_ID':'offline-run','OM9_LAYER_API_OUTPUT':str(ROOT)}):
            exec(compile(ast.Module(body=[start],type_ignores=[]),'actual-idle-driver','exec'),namespace)
            namespace['start']();callback=idle.handlers[0]
            callback(None,None);self.assertFalse(steps)
            state['in_command']=False
            callback(None,None);self.assertEqual(steps,['Undo dispatched']);self.assertFalse(exits)
            callback(None,None);self.assertEqual(steps[-1],'native projection observed');self.assertFalse(exits)
            callback(None,None);self.assertEqual(steps[-1],'terminal report written')
            self.assertEqual(exits,['_-Exit _No']);self.assertFalse(idle.handlers)

    def test_owned_gate_waits_for_bootstrap_end_and_writes_terminal_report_before_exit(self):
        # Boundary test only: no fake layer/geometry success is claimed.
        class Idle:
            def __init__(self):self.handlers=[]
            def __iadd__(self,callback):self.handlers.append(callback);return self
            def __isub__(self,callback):self.handlers.remove(callback);return self
        idle=Idle();state={'in_command':True};exits=[]
        system=types.ModuleType('System');system.EventHandler=lambda callback:callback
        diagnostics=types.ModuleType('System.Diagnostics')
        diagnostics.Process=types.SimpleNamespace(GetCurrentProcess=lambda:types.SimpleNamespace(Id=123))
        rhino=types.ModuleType('Rhino')
        rhino.Commands=types.SimpleNamespace(Command=types.SimpleNamespace(InCommand=lambda:state['in_command']))
        rhino.RhinoDoc=types.SimpleNamespace(ActiveDoc=None)
        with tempfile.TemporaryDirectory(dir=str(ROOT.parent)) as folder:
            def exit_owned(command,echo):
                self.assertFalse(idle.handlers)
                report=json.loads((Path(folder)/'matrix-api-prerequisite.json').read_text())
                self.assertFalse(report['ok'])
                self.assertFalse(report['full_api_gate_passed'])
                exits.append(command);return True
            rhino.RhinoApp=types.SimpleNamespace(Idle=idle,WriteLine=lambda _:None,
                CommandHistoryWindowText='offline boundary',RunScript=exit_owned)
            with patch.dict(sys.modules,{'Rhino':rhino,'System':system,'System.Diagnostics':diagnostics,'clr':types.ModuleType('clr')}),patch.dict(os.environ,{'OM9_LAYER_API_RUN_ID':'offline-run','OM9_LAYER_API_OUTPUT':folder}):
                runpy.run_path(str(ROOT/'rhino5_layer_api_prerequisite.py'))
                self.assertEqual(len(idle.handlers),1)
                callback=idle.handlers[0]
                callback(None,None)
                self.assertFalse(exits)
                self.assertFalse((Path(folder)/'matrix-api-prerequisite.json').exists())
                state['in_command']=False
                callback(None,None)
                self.assertEqual(exits,['_-Exit _No'])

    def test_document_stamp_reads_rhino5_document_id_without_geometry_access(self):
        tree=ast.parse((ROOT/'rhino5_layer_api_prerequisite.py').read_text())
        function=next(node for node in ast.walk(tree) if isinstance(node,ast.FunctionDef) and node.name=='document_stamp')
        module=ast.Module(body=[function],type_ignores=[])
        layers=types.SimpleNamespace(Count=7,CurrentLayerIndex=3)
        # This matches the installed Rhino5 primary SDK: DocumentId exists;
        # RuntimeSerialNumber does not. No object/geometry API is supplied.
        doc=types.SimpleNamespace(DocumentId=42,Modified=True,Layers=layers)
        namespace={'Rhino':types.SimpleNamespace(RhinoDoc=types.SimpleNamespace(ActiveDoc=doc))}
        exec(compile(module,'document-stamp','exec'),namespace)
        self.assertEqual(namespace['document_stamp'](),{'document_id':42,'modified':True,'layer_count':7,'current_layer_index':3})

    def test_direct_entry_uses_current_host_restores_environment_and_writes_unicode(self):
        writes = []
        output = {}
        system = types.ModuleType('System')
        system.Guid = types.SimpleNamespace(NewGuid=lambda: types.SimpleNamespace(ToString=lambda _: 'offline-run'))
        system.IO = types.SimpleNamespace(File=types.SimpleNamespace(
            WriteAllText=lambda path, text, encoding: writes.append((path, text, encoding))))
        system.Text = types.SimpleNamespace(UTF8Encoding=lambda bom: ('utf8', bom))
        system.ArgumentException = LookupError
        diagnostics = types.ModuleType('System.Diagnostics')
        def absent(pid):
            raise LookupError(pid)
        diagnostics.Process = types.SimpleNamespace(GetProcessById=absent,
            Start=lambda _: self.fail('Direct diagnostic must not launch another app'))
        diagnostics.ProcessStartInfo = lambda: types.SimpleNamespace(EnvironmentVariables={})
        diagnostics.ProcessWindowStyle = types.SimpleNamespace(Hidden=0)
        threading = types.ModuleType('System.Threading')
        threading.Thread = types.SimpleNamespace()
        env_before = {key: os.environ.get(key) for key in ('OM9_LAYER_API_RUN_ID', 'OM9_LAYER_API_OUTPUT')}

        def execute(path, namespace):
            self.assertTrue(namespace.get('OM9_LAYER_API_DIRECT_READ_ONLY'))
            self.assertEqual(os.environ['OM9_LAYER_API_RUN_ID'], 'offline-run')
            folder = Path(os.environ['OM9_LAYER_API_OUTPUT'])
            output['path'] = folder
            folder.mkdir(parents=True, exist_ok=True)
            (folder / 'matrix-api-prerequisite.json').write_text(json.dumps({
                'ok': False, 'full_api_gate_passed': False,
                'error': 'Lỗi thử nghiệm — màu lớp', 'run_id': 'offline-run'
            }), encoding='utf-8')

        with tempfile.TemporaryDirectory(dir=str(ROOT.parent)) as folder:
            script = ROOT / 'rhino5_verify_layer_api.py'
            # Keep all temporary reports inside the test directory. The host
            # entry supports explicit test output; no real app can be launched.
            real_open = open
            def safe_open(path, *args, **kwargs):
                if str(path).endswith('rhino5-layer-api-last-launch.log'):
                    return io.StringIO()
                return real_open(path, *args, **kwargs)
            with patch.dict(sys.modules, {'System': system, 'System.Diagnostics': diagnostics, 'System.Threading': threading}), patch.dict(os.environ, {'OM9_LAYER_API_DIRECT_ROOT': folder}), patch('builtins.print'), patch('builtins.open', safe_open):
                runpy.run_path(str(script), init_globals={'execfile': execute})
            self.assertEqual(env_before, {key: os.environ.get(key) for key in env_before})
            self.assertTrue(writes)
            self.assertTrue(any('Lỗi thử nghiệm' in text for _, text, _ in writes))
            self.assertTrue(all(encoding == ('utf8', False) for _, _, encoding in writes))


if __name__ == '__main__':
    unittest.main()
