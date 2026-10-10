# -*- coding: utf-8 -*-
"""Existing RhinoPython native command owns Undo; no plugin registration."""
import sys,os,json,traceback,ntpath
import Rhino
sys.path.insert(0,r'C:\Users\nguye\.codex\worktrees\layer-session-handoff\FreeCAD-src\Mod\OpenMatrix9\tests')
import rhino5_matrix_layer_bridge as bridge
import matrix_om9_handoff_support as support
output=os.environ.get('OM9_MATRIX_LAYER_RECEIVE_OUTPUT')
if not output or not ntpath.isabs(output) or not os.path.isdir(output):raise RuntimeError('Missing owned handoff output')
report=dict(ok=False,application_accepted=False,document_id=int(Rhino.RhinoDoc.ActiveDoc.DocumentId))
report['fault']=os.environ.get('OM9_MATRIX_LAYER_RECEIVE_FAULT','')
try:
    report['runtime']=bridge.load()
    bridge.receive(Rhino.RhinoDoc.ActiveDoc,os.environ.get('OM9_MATRIX_LAYER_RECEIVE_FAULT',''))
    report['ok']=True
except Exception:
    report['error']=traceback.format_exc()
    if not os.environ.get('OM9_MATRIX_LAYER_RECEIVE_FAULT'):Rhino.RhinoApp.WriteLine(report['error'])
finally:
    support.write_json(ntpath.join(output,'matrix-layer-receive.json'),report)
