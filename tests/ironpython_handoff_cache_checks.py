# -*- coding: utf-8 -*-
"""Execute the actual fresh loader under the installed Rhino 5 IronPython engine."""
import os,sys,json,types
ROOT=r'H:\FreeCAD-src\build\om9-layer-edits'
path=os.path.join(ROOT,'tests','rhino5_verify_matrix_om9_handoff.py')
source=open(path,'rb').read()
start=source.index('def load_support(');end=source.index('\nsupport=',start)
namespace={};exec(compile(source[start:end],path,'exec'),namespace)
cached=types.ModuleType('matrix_om9_handoff_support')
def old_acceptance(report):
    return all(report['directions'].get(name,{}).get(key) is True for name in ('matrix_to_om9','om9_to_matrix') for key in ('copy','paste','undo','redo','transfer'))
cached.accepted=old_acceptance
previous=sys.modules.get('matrix_om9_handoff_support');sys.modules['matrix_om9_handoff_support']=cached
try:
    native_path=r'H:\FreeCAD-src\build\matrix-om9-handoff\7c9f839b33e348cba0aca6235b387f42\handoff-results.json'
    report=json.loads(open(native_path,'rb').read())
    assert len(report['checks'])==16 and all(row['passed'] for row in report['checks'])
    assert not __import__('matrix_om9_handoff_support').accepted(report),'Old cached criteria must reproduce the false FAIL'
    fresh=namespace['load_support'](os.path.join(ROOT,'tests','matrix_om9_handoff_support.py'))
    assert fresh.accepted(report),'Fresh scope criteria must accept all16 native checks'
    report['directions']['matrix_to_om9']['undo']=False
    assert not fresh.accepted(report),'A real OM9 Undo failure must remain rejected'
    assert sys.modules['matrix_om9_handoff_support'] is cached
    print('Installed Rhino5 IronPython: cached old policy reproduces false FAIL; actual fresh loader PASS; real OM9 Undo failure rejected.')
finally:
    if previous is None:sys.modules.pop('matrix_om9_handoff_support',None)
    else:sys.modules['matrix_om9_handoff_support']=previous
