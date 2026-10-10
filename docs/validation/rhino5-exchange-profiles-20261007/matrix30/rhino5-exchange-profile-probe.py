# -*- coding: utf-8 -*-
"""Read/write test copies using Rhino5 File3dm; active document is untouched."""
import Rhino
import hashlib
import json
import os
import traceback

HERE = os.path.dirname(__file__)
OUTPUT = os.path.join(HERE, 'rhino5-exchange-profile-results.json')


def checksum(path):
    with open(path, 'rb') as stream:
        return hashlib.sha256(stream.read()).hexdigest()


def inspect(model, expected):
    objects = []
    for obj in model.Objects:
        geometry = obj.Geometry
        row = {'uuid': str(obj.Attributes.ObjectId), 'valid': bool(geometry.IsValid),
               'managed_type': str(geometry.GetType().FullName)}
        if isinstance(geometry, Rhino.Geometry.Curve):
            first, last = float(geometry.Domain.T0), float(geometry.Domain.T1)
            row['domain'] = [first, last]
            row['samples'] = []
            for i in range(17):
                point = geometry.PointAt(first + (last-first)*i/16.0)
                row['samples'].append([float(point.X), float(point.Y), float(point.Z)])
        elif isinstance(geometry, Rhino.Geometry.Brep):
            row['faces'] = int(geometry.Faces.Count)
            row['solid'] = bool(geometry.IsSolid)
            row['orientation'] = str(geometry.SolidOrientation)
            mass = Rhino.Geometry.VolumeMassProperties.Compute(geometry)
            if mass is not None:
                try:
                    row['volume_mm3'] = float(mass.Volume)
                finally:
                    mass.Dispose()
        objects.append(row)
    actual = dict((row['uuid'].lower(), row) for row in objects)
    checks = []
    for native in expected['objects']:
        row = actual.get(native['uuid'].lower())
        passed = row is not None and row['valid']
        comparison = {'uuid': native['uuid'], 'present': row is not None}
        if row is not None and 'samples' in native:
            samples = row.get('samples', [])
            deviation = None
            if len(samples) == len(native['samples']):
                deviation = max(sum((float(a)-float(b))**2 for a, b in zip(p, q))**0.5
                                for p, q in zip(samples, native['samples']))
            comparison['maximum_sample_deviation_mm'] = deviation
            passed = passed and row.get('domain') == native['domain'] and deviation is not None and deviation <= 1e-9
        elif row is not None and 'faces' in native:
            passed = (passed and row.get('faces') == native['faces'] and row.get('solid')
                      and row.get('orientation') == 'Outward'
                      and 'volume_mm3' in row and abs(row['volume_mm3']-native['volume_mm3']) <= 1e-6)
        comparison['passed'] = bool(passed)
        checks.append(comparison)
    return {'objects': objects, 'comparisons': checks,
            'passed': len(objects) == len(expected['objects']) and all(row['passed'] for row in checks)}


report = {'ok': False, 'active_document_untouched': True,
          'rhino_version': str(Rhino.RhinoApp.Version), 'cases': []}
print('OM9 Rhino5 exchange profile probe started (writes test copies only)')
try:
    with open(os.path.join(HERE, 'oracle.json'), 'rb') as stream:
        oracle_bytes = stream.read()
    oracle = json.loads(oracle_bytes)
    report['oracle_sha256'] = hashlib.sha256(oracle_bytes).hexdigest()
    for expected in oracle['cases']:
        path = os.path.join(HERE, expected['fixture'])
        before = checksum(path)
        if before != expected['source_sha256']:
            raise RuntimeError('Source hash differs from native oracle: ' + path)
        model = Rhino.FileIO.File3dm.Read(path)
        if model is None:
            raise RuntimeError('Rhino5 cannot read: ' + path)
        target = os.path.join(HERE, 'resaved-'+expected['fixture'])
        try:
            first = inspect(model, expected)
            written = bool(model.Write(target, 5))
        finally:
            model.Dispose()
        if not written:
            raise RuntimeError('Rhino5 cannot write test copy: ' + target)
        reread = Rhino.FileIO.File3dm.Read(target)
        if reread is None:
            raise RuntimeError('Rhino5 cannot reread test copy: ' + target)
        try:
            second = inspect(reread, expected)
        finally:
            reread.Dispose()
        unchanged = checksum(path) == before
        passed = first['passed'] and second['passed'] and unchanged
        report['cases'].append({'fixture': expected['fixture'], 'source_sha256': before,
                                'saved_fixture': os.path.basename(target), 'saved_sha256': checksum(target),
                                'first_read': first, 'reread': second,
                                'input_unchanged': unchanged, 'passed': bool(passed)})
        print('OM9 exchange profile '+expected['fixture']+': '+str(passed))
    report['ok'] = len(report['cases']) == len(oracle['cases']) and all(row['passed'] for row in report['cases'])
except Exception:
    report['error'] = traceback.format_exc()
with open(OUTPUT, 'wb') as stream:
    stream.write(json.dumps(report, ensure_ascii=True, indent=2).encode('ascii'))
print('OM9 Rhino5 exchange profile output: '+OUTPUT)
print('OM9 Rhino5 exchange profile probe completed: '+str(report['ok']))
