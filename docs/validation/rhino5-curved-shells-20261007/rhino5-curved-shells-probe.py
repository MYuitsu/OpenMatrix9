# -*- coding: utf-8 -*-
"""Actual Rhino5 document Open/SaveAs/reopen; run after saving current work."""
import Rhino
import rhinoscriptsyntax as rs
import datetime, hashlib, json, os, traceback

HERE=os.path.dirname(__file__)
OUTPUT=os.path.join(HERE,'gui-output-'+datetime.datetime.now().strftime('%Y%m%d-%H%M%S'))
report={'ok':False,'cases':[],'rhino_version':str(Rhino.RhinoApp.Version)}

def checksum(path):
    with open(path,'rb') as stream:
        return hashlib.sha256(stream.read()).hexdigest()

def command(text):
    if not rs.Command(text,False):
        raise RuntimeError('Rhino command failed: '+text)

def document_measurements(expected):
    doc=Rhino.RhinoDoc.ActiveDoc
    actual={}
    for obj in doc.Objects:
        if obj.IsDeleted:
            continue
        geometry=obj.Geometry
        row={'uuid':str(obj.Id),'valid':bool(geometry.IsValid),'managed_type':str(geometry.GetType().FullName)}
        if isinstance(geometry,Rhino.Geometry.Curve):
            first,last=float(geometry.Domain.T0),float(geometry.Domain.T1)
            row['domain']=[first,last]
            row['samples']=[]
            for i in range(17):
                point=geometry.PointAt(first+(last-first)*i/16.0)
                row['samples'].append([float(point.X),float(point.Y),float(point.Z)])
        elif isinstance(geometry,Rhino.Geometry.Brep):
            row['faces']=int(geometry.Faces.Count)
            row['solid']=bool(geometry.IsSolid)
            row['orientation']=str(geometry.SolidOrientation)
            mass=Rhino.Geometry.VolumeMassProperties.Compute(geometry)
            if mass is not None:
                try:
                    row['volume_mm3']=float(mass.Volume)
                finally:
                    mass.Dispose()
        actual[row['uuid'].lower()]=row
    comparisons=[]
    for wanted in expected['objects']:
        row=actual.get(wanted['uuid'].lower())
        passed=row is not None and row['valid']
        comparison={'uuid':wanted['uuid'],'present':row is not None}
        if row is not None and 'samples' in wanted:
            samples=row.get('samples',[])
            deviation=None
            if len(samples)==len(wanted['samples']):
                deviation=max(sum((float(a)-float(b))**2 for a,b in zip(p,q))**0.5 for p,q in zip(samples,wanted['samples']))
            comparison['maximum_sample_deviation_mm']=deviation
            passed=passed and row.get('domain')==wanted['domain'] and deviation is not None and deviation<=1e-9
        elif row is not None and 'faces' in wanted:
            passed=passed and row.get('faces')==wanted['faces'] and row.get('solid') and row.get('orientation')=='Outward' and 'volume_mm3' in row and abs(row['volume_mm3']-wanted['volume_mm3'])<=1e-6
        comparison['passed']=bool(passed)
        comparisons.append(comparison)
    return {'objects':list(actual.values()),'comparisons':comparisons,'passed':len(actual)==len(expected['objects']) and all(c['passed'] for c in comparisons)}

def dump():
    with open(os.path.join(OUTPUT,'rhino5-gui-profile-results.json'),'wb') as stream:
        stream.write(json.dumps(report,ensure_ascii=True,indent=2).encode('ascii'))

print('OM9 Rhino5 GUI exchange acceptance started (opens test documents)')
try:
    if Rhino.RhinoDoc.ActiveDoc.Modified:
        raise RuntimeError('Save current Rhino document before running GUI acceptance')
    os.mkdir(OUTPUT)
    oracle_path=os.path.join(HERE,'oracle.json')
    with open(oracle_path,'rb') as stream:
        oracle_bytes=stream.read()
    oracle=json.loads(oracle_bytes)
    report['oracle_sha256']=hashlib.sha256(oracle_bytes).hexdigest()
    for expected in oracle['cases']:
        source=os.path.join(HERE,expected['fixture'])
        if checksum(source)!=expected['source_sha256']:
            raise RuntimeError('Source differs from native oracle: '+source)
        command('_-Open "'+source+'"')
        if os.path.normcase(Rhino.RhinoDoc.ActiveDoc.Path)!=os.path.normcase(source):
            raise RuntimeError('Rhino opened another source')
        first=document_measurements(expected)
        command('_SelNone')
        command('_SelBadObjects')
        bad_count=len(list(Rhino.RhinoDoc.ActiveDoc.Objects.GetSelectedObjects(False,False)))
        command('_SelNone')
        target=os.path.join(OUTPUT,'gui-resaved-'+os.path.basename(source))
        command('_-SaveAs "'+target+'" _Enter')
        with open(target,'rb') as stream:
            if stream.read(32)!='3D Geometry File Format       50':
                raise RuntimeError('GUI output is not Rhino5')
        command('_-Open "'+target+'"')
        if os.path.normcase(Rhino.RhinoDoc.ActiveDoc.Path)!=os.path.normcase(target):
            raise RuntimeError('Rhino opened another saved copy')
        second=document_measurements(expected)
        passed=first['passed'] and second['passed'] and bad_count==0 and checksum(source)==expected['source_sha256']
        report['cases'].append(dict(fixture=expected['fixture'],source_sha256=expected['source_sha256'],saved_fixture=target,saved_sha256=checksum(target),first_read=first,reread=second,bad_objects=bad_count,passed=bool(passed)))
        dump()
        print('OM9 GUI profile '+expected['fixture']+': '+str(passed))
    report['ok']=len(report['cases'])==len(oracle['cases']) and all(c['passed'] for c in report['cases'])
except Exception:
    report['error']=traceback.format_exc()
    print(report['error'])
finally:
    if not os.path.isdir(OUTPUT):
        os.mkdir(OUTPUT)
    dump()
    print('OM9 Rhino5 GUI exchange output: '+OUTPUT)
    print('OM9 Rhino5 GUI exchange acceptance completed: '+str(report['ok']))
