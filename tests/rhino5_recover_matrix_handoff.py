# -*- coding: utf-8 -*-
"""Remove only fingerprinted fixtures of failed303c; current Matrix only."""
import os,ntpath,json,traceback
import Rhino,System
BASE=r'H:\FreeCAD-src\build\om9-layer-edits\tests'
FAILED=r'H:\FreeCAD-src\build\matrix-om9-handoff\303c98e676534ffb91c894c130fba7b7'
with open(ntpath.join(BASE,'rhino5_verify_matrix_om9_handoff.py'),'r') as stream:source=stream.read()
namespace={};exec(source[:source.rindex('\ntry:start()')],namespace)
support=namespace['support']
observed=support.read_json(ntpath.join(FAILED,'read-only-a8ad89a847b44092a223ac6bbce4ae70','matrix-observation.json'))
output=ntpath.join(FAILED,'recovery-'+System.Guid.NewGuid().ToString('N'));os.makedirs(output)
namespace['output']=output
result=dict(ok=False,application_accepted=False,scope='Remove owned fixtures; preserve observed original34-layer palette',
    original_before_run_snapshot_unavailable=True)
handler=None

def validate():
    doc=Rhino.RhinoDoc.ActiveDoc
    if int(doc.DocumentId)!=observed['current_state']['document_id'] or doc.Path:
        raise RuntimeError('Failed-run document is no longer active/unsaved; no recovery edits')
    current=namespace['snapshot']();raw=current['raw'];old=observed['current_state']['raw']
    result['observed_model_rows']=raw['objects']
    if raw['layers']!=old['layers'] or raw['active']!=old['active']:
        raise RuntimeError('Layer palette changed after read-only observation; no recovery edits')
    original_ids=set(row['id'] for row in observed['layer_indices'] if row['index']<34)
    if len(original_ids)!=34:raise RuntimeError('Original palette index evidence differs')
    fixture_paths=set([('Matrix Transfer 303c98e6',),('Matrix Transfer 303c98e6','Detail'),
        ('Matrix Empty 303c98e6',),('Matrix Restricted 303c98e6',),('Matrix Restricted 303c98e6','Own state')])
    om9=support.read_json(ntpath.join(FAILED,'response-004.json'))['result']['semantic']
    allowed_paths=fixture_paths|set(tuple(row['path']) for row in om9['layers'])
    own_layers=[row for row in raw['layers'] if row['id'] not in original_ids]
    if len(own_layers)!=30 or any(tuple(row['path']) not in allowed_paths for row in own_layers):
        raise RuntimeError('Layer ownership does not match failed fixtures; no recovery edits')
    matrix=support.read_json(ntpath.join(FAILED,'response-001.json'))['result']['semantic']
    expected=matrix['objects']+om9['objects'];actual=raw['objects']
    if len(actual)!=5 or len(expected)!=5:raise RuntimeError('Expected exactly five failed-run test objects; no recovery edits')
    for row in expected:
        matches=[item for item in actual if item['name']==row['name']]
        if len(matches)!=1:raise RuntimeError('Object ownership/name ambiguous: '+row['name'])
        item=matches[0]
        if item['layer']!=row['layer'] or any(abs(a-b)>0.001 for a,b in zip(item['bounds'],row['bounds'])):
            raise RuntimeError('Fixture geometry/layer changed: '+row['name']+'; no recovery edits')
        if row in om9['objects']:
            native=doc.Objects.Find(System.Guid(item['id']))
            tag=native.Attributes.GetUserString('OpenMatrix9.LayerObjectId')
            result.setdefault('source_tags',{})[item['id']]=str(tag or '')
            if str(tag or '')!=row['name'].replace(' ',''):
                raise RuntimeError('OM9 source-object tag differs: '+row['name']+'; no recovery edits')
    return doc,current,original_ids,own_layers

def finish(sender,args):
    global handler
    if Rhino.Commands.Command.InCommand():return
    Rhino.RhinoApp.Idle-=handler
    try:
        doc,current,original_ids,own_layers=validate()
        support.write_json(ntpath.join(output,'recovery-before.json'),current)
        namespace['original_doc']=int(doc.DocumentId)
        namespace['original']={'raw':dict(layers=[row for row in current['raw']['layers'] if row['id'] in original_ids],
            objects=[],active=current['raw']['active'])}
        namespace['owned_objects']=set(row['id'] for row in current['raw']['objects'])
        namespace['owned_layers']=set(row['id'] for row in own_layers)
        result['ok']=bool(namespace['cleanup']())
        result['after']=namespace['snapshot']()
        result['removed_object_ids']=sorted(namespace['owned_objects'])
        result['removed_layer_ids']=sorted(namespace['owned_layers'])
    except Exception:
        result['error']=traceback.format_exc()
    support.write_json(ntpath.join(output,'recovery-results.json'),result)
    Rhino.RhinoApp.WriteLine('Owned failed-run fixture recovery: '+('PASS' if result['ok'] else 'FAIL')+'; '+ntpath.join(output,'recovery-results.json'))

try:
    validate()  # Refuse a changed/working document before queuing any edits.
    handler=System.EventHandler(finish);Rhino.RhinoApp.Idle+=handler
    Rhino.RhinoApp.WriteLine('Queued cleanup of five verified test objects and30 owned layers after this command. No applications/plugins are loaded.')
except Exception:
    result['error']=traceback.format_exc()
    support.write_json(ntpath.join(output,'recovery-results.json'),result)
    Rhino.RhinoApp.WriteLine('Owned failed-run fixture recovery: FAIL; no edits; '+ntpath.join(output,'recovery-results.json'))
