# -*- coding: utf-8 -*-
"""Rhino5 read-only mass quadrature diagnostic. No document Open/Save/edit."""
import Rhino
import hashlib, json, os, traceback, math

HERE=os.path.dirname(__file__)
REPORT=os.path.join(HERE,'rhino5-curved-precision-results.json')
GUI_REPORT=os.path.join(HERE,'gui-output-20261007-223723','rhino5-gui-profile-results.json')
THRESHOLD=1e-6
PRECISIONS=[1e-8,1e-10,1e-12,1e-13]
result={'ok':False,'cases':[],'rhino_version':str(Rhino.RhinoApp.Version),'volume_threshold_mm3':THRESHOLD}

def checksum(path):
    with open(path,'rb') as stream:
        return hashlib.sha256(stream.read()).hexdigest()

def load(path):
    with open(path,'rb') as stream:
        return json.loads(stream.read())

def measure(path,wanted):
    before=checksum(path)
    model=Rhino.FileIO.File3dm.Read(path)
    if model is None:raise RuntimeError('Rhino failed to read '+path)
    try:
        objects=list(model.Objects)
        if len(objects)!=1:raise RuntimeError('Expected one native BRep '+path)
        obj=objects[0];brep=obj.Geometry
        if not isinstance(brep,Rhino.Geometry.Brep):raise RuntimeError('Native class changed '+path)
        row={'path':path,'source_sha256':before,'uuid':str(obj.Attributes.ObjectId),'valid':bool(brep.IsValid),'solid':bool(brep.IsSolid),'faces':int(brep.Faces.Count),'orientation':str(brep.SolidOrientation),'analytical_volume_mm3':float(wanted['volume_mm3']),'quadrature':[]}
        mass=Rhino.Geometry.VolumeMassProperties.Compute(brep)
        if mass is None:raise RuntimeError('Default volume measurement failed '+path)
        try:
            row['default_volume_mm3']=float(mass.Volume)
            row['default_volume_error_mm3']=float(mass.VolumeError)
        finally:mass.Dispose()
        for epsilon in PRECISIONS:
            print('OM9 precision '+os.path.basename(path)+' tolerance '+str(epsilon))
            # This overload exists in the installed Rhino5 RhinoCommon.xml.
            # Improve integration accuracy; do not alter the acceptance threshold.
            volume=float(brep.GetVolume(epsilon,epsilon))
            if math.isnan(volume) or math.isinf(volume):raise RuntimeError('Nonfinite volume '+path)
            row['quadrature'].append({'relative_tolerance':epsilon,'absolute_tolerance':epsilon,'volume_mm3':volume,'analytical_deviation_mm3':abs(volume-float(wanted['volume_mm3']))})
        row['convergence_delta_mm3']=abs(row['quadrature'][-1]['volume_mm3']-row['quadrature'][-2]['volume_mm3'])
        row['input_hash_unchanged']=checksum(path)==before
        row['passed']=row['valid'] and row['solid'] and row['orientation']=='Outward' and row['faces']==wanted['faces'] and row['uuid'].lower()==wanted['uuid'].lower() and row['input_hash_unchanged'] and row['quadrature'][-1]['analytical_deviation_mm3']<=THRESHOLD and row['convergence_delta_mm3']<=THRESHOLD
        return row
    finally:model.Dispose()

print('OM9 Rhino5 curved precision started (read-only)')
try:
    active=Rhino.RhinoDoc.ActiveDoc
    before_document=(active.DocumentId,active.Modified,active.Path,len(list(active.Objects)))
    oracle_path=os.path.join(HERE,'oracle.json');oracle=load(oracle_path);gui=load(GUI_REPORT)
    result['oracle_sha256']=checksum(oracle_path);result['gui_report_sha256']=checksum(GUI_REPORT)
    if gui['oracle_sha256']!=result['oracle_sha256'] or len(gui['cases'])!=len(oracle['cases']):raise RuntimeError('GUI report is not bound to this oracle')
    for expected,saved in zip(oracle['cases'],gui['cases']):
        source=os.path.join(HERE,expected['fixture'])
        if expected['fixture']!=saved['fixture'] or checksum(source)!=expected['source_sha256'] or checksum(saved['saved_fixture'])!=saved['saved_sha256']:raise RuntimeError('Input differs from actual GUI evidence')
        original=measure(source,expected['objects'][0])
        reopened=measure(saved['saved_fixture'],expected['objects'][0])
        delta=abs(original['quadrature'][-1]['volume_mm3']-reopened['quadrature'][-1]['volume_mm3'])
        result['cases'].append({'fixture':expected['fixture'],'original':original,'saved':reopened,'saveas_volume_delta_mm3':delta,'passed':original['passed'] and reopened['passed'] and delta<=THRESHOLD})
    active=Rhino.RhinoDoc.ActiveDoc
    result['active_document_unchanged']=before_document==(active.DocumentId,active.Modified,active.Path,len(list(active.Objects)))
    result['ok']=len(result['cases'])==4 and result['active_document_unchanged'] and all(row['passed'] for row in result['cases'])
except Exception:
    result['error']=traceback.format_exc();print(result['error'])
finally:
    with open(REPORT,'wb') as stream:stream.write(json.dumps(result,ensure_ascii=True,indent=2).encode('ascii'))
    print('OM9 Rhino5 curved precision output: '+REPORT)
    print('OM9 Rhino5 curved precision completed: '+str(result['ok']))
