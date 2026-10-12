# -*- coding: utf-8 -*-
"""Current Matrix + owned OM9, shared Rust full-palette adapter, no Rhino/RHP launch."""
import os,sys,types
sys.path.insert(0,r'C:\Users\nguye\.codex\worktrees\layer-session-handoff\FreeCAD-src\Mod\OpenMatrix9\tests')
import rhino5_matrix_layer_bridge as bridge
bridge.load() # ABI/hash validation before creating fixtures or changing clipboard
workflow=types.ModuleType('om9_current_palette_workflow')
execfile(r'C:\Users\nguye\.codex\worktrees\layer-session-handoff\FreeCAD-src\Mod\OpenMatrix9\tests\rhino5_verify_matrix_om9_handoff.py',workflow.__dict__)
original_run_native=workflow.run_native
def run_adapter(command):
    if command=='_CopyToClipboard':
        started=workflow.time.time();bridge.copy_selected(workflow.document())
        workflow.report['timings'].append(dict(app='Matrix',operation='Rust full-palette Copy Selected',seconds=workflow.time.time()-started))
        return True
    if command=='_Paste':
        os.environ['OM9_MATRIX_LAYER_RECEIVE_OUTPUT']=workflow.output
        path=workflow.ntpath.join(workflow.TESTS,'rhino5_matrix_layer_receive.py')
        ack=workflow.ntpath.join(workflow.output,'matrix-layer-receive.json')
        if os.path.isfile(ack):os.remove(ack)
        before=workflow.snapshot()
        try:
            for fault in ('after-layer','after-geometry'):
                if os.path.isfile(ack):os.remove(ack)
                os.environ['OM9_MATRIX_LAYER_RECEIVE_FAULT']=fault
                original_run_native('_-RunPythonScript "'+path+'"')
                receipt=workflow.support.read_json(ack)
                observed=workflow.snapshot()
                workflow.checkpoint('matrix-after-rollback-'+fault,observed)
                unchanged=observed['raw']==before['raw']
                injected=receipt.get('fault')==fault and 'Injected '+('layer' if fault=='after-layer' else 'geometry')+' failure' in receipt.get('error','')
                workflow.check('Matrix injected '+fault+' restores exact before state',injected and not receipt['ok'] and unchanged,receipt.get('error'))
                if not injected or receipt['ok'] or not unchanged:raise RuntimeError('Injected receive rollback failed; successful Paste was not attempted')
        finally:os.environ.pop('OM9_MATRIX_LAYER_RECEIVE_FAULT',None)
        if os.path.isfile(ack):os.remove(ack)
        returned=original_run_native('_-RunPythonScript "'+path+'"')
        if not os.path.isfile(ack):raise RuntimeError('Matrix receive command produced no terminal report')
        receipt=workflow.support.read_json(ack);workflow.report['matrix_adapter_receive']=receipt
        return returned and receipt.get('ok',False) and receipt.get('document_id')==int(workflow.document().DocumentId) and receipt.get('fault')==''
    return original_run_native(command)
workflow.run_native=run_adapter
workflow.report['scope']='Current Matrix transfer adapter + shared Rust policy <-> actual OM9 public clipboard; full palette, OM9 Undo/Redo only'
workflow.report['undo_scope']='om9_only'
base_snapshot=workflow.snapshot
def own_state_snapshot():
    value=base_snapshot();doc=workflow.document()
    for group in ('raw','semantic'):
        for row in value[group]['layers']:
            if group=='raw':row['native_locked']=row['locked'];row['native_visible']=row['visible']
            path='::'.join(row['path'])
            for i in range(doc.Layers.Count):
                layer=doc.Layers[i]
                if layer is not None and not layer.IsDeleted and str(layer.FullPath)==path:
                    if group=='raw':row['persistent_witness']=str(layer.GetUserString('OpenMatrix9.LayerPersistent.v1')) if layer.GetUserString('OpenMatrix9.LayerPersistent.v1') is not None else None
                    if layer.ParentLayerId!=workflow.System.Guid.Empty:
                        row['locked']=bool(layer.GetPersistentLocking());row['visible']=bool(layer.GetPersistentVisibility())
                    break
        for row in value[group]['objects']:
            for obj in workflow.objects(doc):
                if str(obj.Attributes.Name or '')==row['name'] and obj.Attributes.GetUserString('OpenMatrix9.Locked')=='1':
                    row['locked']=True
    return value
workflow.snapshot=own_state_snapshot
base_cleanup=workflow.cleanup
def cleanup():
    if workflow.original is not None:
        doc=workflow.document();current=workflow.snapshot()['raw']
        if doc.Path or any(row['id'] not in workflow.owned_objects for row in current['objects']):raise RuntimeError('Foreign/saved document; no witness cleanup attempted')
        for row in workflow.original['raw']['layers']:
            bridge.call('NativeLayerWitness','Restore',doc,workflow.System.Guid(row['id']),row.get('persistent_witness'))
    return base_cleanup()
workflow.cleanup=cleanup
