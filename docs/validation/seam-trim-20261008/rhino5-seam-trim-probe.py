# -*- coding: utf-8 -*-
"""Actual Rhino5 document Open/SaveAs/reopen; run after saving current work."""
import Rhino
import rhinoscriptsyntax as rs
import datetime, hashlib, json, math, os, traceback

HERE=os.path.dirname(__file__)
OUTPUT=os.path.join(HERE,'gui-output-'+datetime.datetime.now().strftime('%Y%m%d-%H%M%S'))
report={'ok':False,'cases':[],'rhino_version':str(Rhino.RhinoApp.Version)}

def checksum(path):
    with open(path,'rb') as stream:
        return hashlib.sha256(stream.read()).hexdigest()

def command(text):
    if not rs.Command(text,False):
        raise RuntimeError('Rhino command failed: '+text)

def finite_number(value):
    return not math.isnan(value) and not math.isinf(value)

def report_number(value):
    value=float(value)
    return value if finite_number(value) else str(value)

def sample_deviation(samples,expected):
    maximum=0.0
    for point,wanted in zip(samples,expected):
        if len(point)!=3 or len(wanted)!=3:
            return None
        delta=[float(a)-float(b) for a,b in zip(point,wanted)]
        if not all(finite_number(value) for value in delta):
            return None
        scale=max(abs(value) for value in delta)
        if scale:
            # Squaring a raw UnsetPoint or huge finite coordinate can overflow.
            # Scale first; the three squared normalized values are <=1 each.
            distance=scale*math.sqrt(sum((value/scale)*(value/scale) for value in delta))
            if not finite_number(distance):
                return None
            maximum=max(maximum,distance)
    return maximum

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
            row['domain']=[report_number(first),report_number(last)]
            row['samples']=[]
            row['sample_validity']=[]
            row['sample_errors']=[]
            for i in range(17):
                try:
                    if not finite_number(first) or not finite_number(last):
                        raise ValueError('Nonfinite curve domain')
                    point=geometry.PointAt(first+(last-first)*i/16.0)
                    coordinates=[float(point.X),float(point.Y),float(point.Z)]
                    valid=bool(point.IsValid) and all(finite_number(value) for value in coordinates)
                    row['samples'].append([report_number(value) for value in coordinates])
                    row['sample_validity'].append(valid)
                    if not valid:
                        row['sample_errors'].append(dict(index=i,point_is_valid=bool(point.IsValid),reason='Invalid Point3d or nonfinite coordinate'))
                except Exception:
                    row['samples'].append([])
                    row['sample_validity'].append(False)
                    row['sample_errors'].append(dict(index=i,error=traceback.format_exc()))
        elif isinstance(geometry,Rhino.Geometry.Brep):
            row['faces']=int(geometry.Faces.Count)
            row['solid']=bool(geometry.IsSolid)
            row['orientation']=str(geometry.SolidOrientation)
            mass=Rhino.Geometry.VolumeMassProperties.Compute(geometry)
            if mass is None:
                raise RuntimeError('Rhino failed to compute default BRep mass properties')
            try:
                row['default_volume_mm3']=float(mass.Volume)
                row['default_volume_error_mm3']=float(mass.VolumeError)
            finally:
                mass.Dispose()
            # Actual Rhino5 eight-file convergence probe confirms this public
            # overload. Improve integration, retaining the1e-6mm3 threshold
            # and the original default measurement as separate evidence.
            row['volume_mm3']=float(geometry.GetVolume(1e-13,1e-13))
            row['volume_relative_tolerance']=1e-13
            row['volume_absolute_tolerance']=1e-13
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
                deviation=sample_deviation(samples,wanted['samples'])
            comparison['maximum_sample_deviation_mm']=deviation
            comparison['sample_threshold_mm']=1e-9
            comparison['samples_valid']=bool(row.get('sample_validity')) and all(row['sample_validity'])
            passed=passed and comparison['samples_valid'] and row.get('domain')==wanted['domain'] and deviation is not None and deviation<=1e-9
        elif row is not None and 'faces' in wanted:
            comparison['volume_threshold_mm3']=1e-6
            comparison['volume_deviation_mm3']=abs(row['volume_mm3']-wanted['volume_mm3']) if 'volume_mm3' in row else None
            comparison['default_volume_deviation_mm3']=abs(row['default_volume_mm3']-wanted['volume_mm3']) if 'default_volume_mm3' in row else None
            passed=passed and row.get('faces')==wanted['faces'] and row.get('solid') and row.get('orientation')=='Outward' and comparison['volume_deviation_mm3'] is not None and comparison['volume_deviation_mm3']<=1e-6
        comparison['passed']=bool(passed)
        comparisons.append(comparison)
    return {'objects':list(actual.values()),'comparisons':comparisons,'passed':len(actual)==len(expected['objects']) and all(c['passed'] for c in comparisons)}

def dump():
    with open(os.path.join(OUTPUT,'rhino5-gui-profile-results.json'),'wb') as stream:
        stream.write(json.dumps(report,ensure_ascii=True,allow_nan=False,indent=2).encode('ascii'))

def run_case(expected):
    source=os.path.join(HERE,expected['fixture'])
    row=dict(fixture=expected['fixture'],source_sha256=expected['source_sha256'],passed=False)
    try:
        if checksum(source)!=expected['source_sha256']:
            raise RuntimeError('Source differs from native oracle: '+source)
        command('_-Open "'+source+'"')
        if os.path.normcase(Rhino.RhinoDoc.ActiveDoc.Path)!=os.path.normcase(source):
            raise RuntimeError('Rhino opened another source')
        row['first_read']=document_measurements(expected)
        command('_SelNone')
        command('_SelBadObjects')
        row['bad_objects']=len(list(Rhino.RhinoDoc.ActiveDoc.Objects.GetSelectedObjects(False,False)))
        command('_SelNone')
        target=os.path.join(OUTPUT,'gui-resaved-'+os.path.basename(source))
        command('_-SaveAs "'+target+'" _Enter')
        row['saved_fixture']=target
        row['saved_sha256']=checksum(target)
        with open(target,'rb') as stream:
            if stream.read(32)!='3D Geometry File Format       50':
                raise RuntimeError('GUI output is not Rhino5')
        command('_-Open "'+target+'"')
        if os.path.normcase(Rhino.RhinoDoc.ActiveDoc.Path)!=os.path.normcase(target):
            raise RuntimeError('Rhino opened another saved copy')
        row['reread']=document_measurements(expected)
        row['source_unchanged']=checksum(source)==expected['source_sha256']
        row['passed']=bool(row['first_read']['passed'] and row['reread']['passed'] and row['bad_objects']==0 and row['source_unchanged'])
    except Exception:
        row['error']=traceback.format_exc()
    return row

def run_cases(cases):
    for expected in cases:
        try:
            row=run_case(expected)
        except Exception:
            row=dict(fixture=expected['fixture'],passed=False,error=traceback.format_exc())
        report['cases'].append(row)
        dump()
        if row.get('error'):
            print(row['error'])
        print('OM9 GUI profile '+expected['fixture']+': '+str(row['passed']))
    report['ok']=len(report['cases'])==len(cases) and all(c['passed'] for c in report['cases'])

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
    report['probe_sha256']=checksum(__file__)
    report['expected_cases']=len(oracle['cases'])
    for expected in oracle['cases']:
        source=os.path.join(HERE,expected['fixture'])
        if checksum(source)!=expected['source_sha256']:
            raise RuntimeError('Source differs from native oracle: '+source)
    run_cases(oracle['cases'])
except Exception:
    report['error']=traceback.format_exc()
    print(report['error'])
finally:
    if not os.path.isdir(OUTPUT):
        os.mkdir(OUTPUT)
    dump()
    print('OM9 Rhino5 GUI exchange output: '+OUTPUT)
    print('OM9 Rhino5 GUI exchange acceptance completed: '+str(report['ok']))
