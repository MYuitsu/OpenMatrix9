# -*- coding: utf-8 -*-
"""Bounded recovery of the dc77 palette test in the same Matrix document."""
import os,sys,ntpath,traceback,types
import Rhino,System
TESTS=r'C:\Users\nguye\.codex\worktrees\layer-session-handoff\FreeCAD-src\Mod\OpenMatrix9\tests'
sys.path.insert(0,TESTS)
import matrix_om9_handoff_support as support
import rhino5_matrix_layer_bridge as bridge
bridge.load()
FAILED=r'H:\FreeCAD-src\build\matrix-om9-handoff\dc77ce9d849842ce86b027e9a19d2198'
source=open(ntpath.join(TESTS,'rhino5_verify_matrix_om9_handoff.py'),'r').read()
workflow=types.ModuleType('om9_dc77_owned_recovery')
exec(source[:source.rindex('\ntry:start()')],workflow.__dict__)
before=support.read_json(ntpath.join(FAILED,'matrix-original.json'))
fixture=support.read_json(ntpath.join(FAILED,'matrix-before-paste.json'))
failure=support.read_json(ntpath.join(FAILED,'handoff-results.json'))
output=ntpath.join(FAILED,'recovery-'+System.Guid.NewGuid().ToString('N'));os.makedirs(output)
result=dict(ok=False,application_accepted=False,scope='Only verified dc77 two objects/five layers; exact original34-layer state')
handler=None
def validate():
    doc=Rhino.RhinoDoc.ActiveDoc
    if doc is None or int(doc.DocumentId)!=before['document_id'] or doc.Path:
        raise RuntimeError('Current document differs or was saved; no recovery edits')
    current=workflow.snapshot()
    original_ids=set(row['id'] for row in before['raw']['layers'])
    known={row['id']:row for row in fixture['raw']['layers']}
    owned=set(known)-original_ids
    if len(original_ids)!=34 or len(owned)!=5 or len(fixture['raw']['objects'])!=2:
        raise RuntimeError('Failed-run ownership evidence differs; no recovery edits')
    if set(row['id'] for row in current['raw']['layers'])!=set(known):
        raise RuntimeError('Foreign/missing layer; no recovery edits')
    for row in current['raw']['layers']:
        expected=known[row['id']]
        if any(row[key]!=expected[key] for key in ('path','rgb','persistent_locked','persistent_visible')) or row['locked']!=expected.get('native_locked',expected['locked']) or row['visible']!=expected.get('native_visible',expected['visible']):
            raise RuntimeError('Layer native state changed since failed fixture; no recovery edits: '+str(row['path']))
    expected=fixture['raw']['objects']
    if current['raw']['objects']!=expected or current['raw']['active']!=fixture['raw']['active']:
        raise RuntimeError('Fixture geometry/active layer changed; no recovery edits')
    if failure['directions'].get('om9_to_matrix',{}).get('paste'):
        raise RuntimeError('Failed run passed reverse Paste; unexpected recovery scope')
    return doc,current,owned
def finish(sender,args):
    if Rhino.Commands.Command.InCommand():return
    Rhino.RhinoApp.Idle-=handler
    try:
        doc,current,owned=validate()
        support.write_json(ntpath.join(output,'recovery-before.json'),current)
        workflow.output=output;workflow.original_doc=int(doc.DocumentId)
        workflow.original=before;workflow.owned_layers=owned
        workflow.owned_objects=set(row['id'] for row in fixture['raw']['objects'])
        for row in before['raw']['layers']:
            bridge.call('NativeLayerWitness','Restore',doc,System.Guid(row['id']),row.get('persistent_witness'))
        # Base cleanup compares native facts. Witness fields are checked separately.
        native_before=dict(before);native_before['raw']=dict(before['raw'])
        native_before['raw']['layers']=[dict(dict((key,value) for key,value in row.items() if key not in ('native_locked','native_visible','persistent_witness')),locked=row.get('native_locked',row['locked']),visible=row.get('native_visible',row['visible'])) for row in before['raw']['layers']]
        workflow.original=native_before
        result['ok']=bool(workflow.cleanup())
        for row in before['raw']['layers']:
            layer=doc.Layers[doc.Layers.Find(System.Guid(row['id']),True)]
            observed=layer.GetUserString('OpenMatrix9.LayerPersistent.v1')
            if (str(observed) if observed is not None else None)!=row.get('persistent_witness'):
                raise RuntimeError('Original witness reread differs')
        result['after']=workflow.snapshot()
        result['removed_object_ids']=sorted(workflow.owned_objects);result['removed_layer_ids']=sorted(owned)
    except Exception:
        result['ok']=False;result['error']=traceback.format_exc()
    support.write_json(ntpath.join(output,'recovery-results.json'),result)
    Rhino.RhinoApp.WriteLine('Owned dc77 palette fixture recovery: '+('PASS' if result['ok'] else 'FAIL')+'; '+ntpath.join(output,'recovery-results.json'))
try:
    validate()
    handler=System.EventHandler(finish);Rhino.RhinoApp.Idle+=handler
    Rhino.RhinoApp.WriteLine('Queued recovery of two verified dc77 objects/five test layers in current Matrix. No applications/plugins are loaded.')
except Exception:
    result['error']=traceback.format_exc()
    support.write_json(ntpath.join(output,'recovery-results.json'),result)
    Rhino.RhinoApp.WriteLine('Owned dc77 palette fixture recovery: FAIL; no edits; '+ntpath.join(output,'recovery-results.json'))
