# -*- coding: utf-8 -*-
"""Read independent coherent reference fixtures without touching the active document."""
import Rhino
import hashlib
import json
import os
import traceback

ROOT = r'H:\FreeCAD-src\build\3dm-preserve-native\reference-target-evidence'
OUTPUT = os.path.join(os.path.dirname(__file__), 'rhino5-coherent-reference-results.json')


def checksum(path):
    with open(path, 'rb') as stream:
        return hashlib.sha256(stream.read()).hexdigest()


report = {'ok': False, 'read_only': True, 'cases': [],
          'rhino_version': str(Rhino.RhinoApp.Version)}
print('OM9 Rhino5 coherent reference probe started (read-only)')
try:
    with open(os.path.join(ROOT, 'native-results.json'), 'rb') as stream:
        oracle_bytes = stream.read()
    oracle = json.loads(oracle_bytes)
    if not oracle.get('ok') or len(oracle.get('cases', [])) != 4:
        raise RuntimeError('Native coherent fixture oracle is incomplete')
    report['native_oracle_sha256'] = hashlib.sha256(oracle_bytes).hexdigest()
    for expected in oracle['cases']:
        path = os.path.join(ROOT, expected['fixture'])
        before = checksum(path)
        if before != expected['source_sha256']:
            raise RuntimeError('Fixture changed after native verification: ' + path)
        model = Rhino.FileIO.File3dm.Read(path)
        if model is None:
            raise RuntimeError('Rhino5 cannot read fixture: ' + path)
        try:
            objects = []
            for obj in model.Objects:
                geometry = obj.Geometry
                row = {'uuid': str(obj.Attributes.ObjectId),
                       'managed_type': str(geometry.GetType().FullName),
                       'valid': bool(geometry.IsValid), 'object_type': str(geometry.ObjectType)}
                if isinstance(geometry, Rhino.Geometry.Curve):
                    first, last = float(geometry.Domain.T0), float(geometry.Domain.T1)
                    row['domain'] = [first, last]
                    row['samples'] = []
                    for i in range(17):
                        point = geometry.PointAt(first + (last - first)*i/16.0)
                        row['samples'].append([float(point.X), float(point.Y), float(point.Z)])
                objects.append(row)
            by_id = dict((row['uuid'].lower(), row) for row in objects)
            missing = []
            comparisons = []
            for native in expected['objects']:
                actual = by_id.get(native['uuid'].lower())
                if actual is None:
                    missing.append(native)
                    continue
                comparison = {'uuid': native['uuid'], 'valid': actual['valid']}
                if 'samples' in native:
                    comparison['curve_present'] = 'samples' in actual
                    comparison['domain_equal'] = actual.get('domain') == native['domain']
                    deviation = None
                    if 'samples' in actual:
                        deviation = max(sum((float(a)-float(b))**2 for a, b in zip(p, q))**0.5
                                        for p, q in zip(actual['samples'], native['samples']))
                    comparison['maximum_sample_deviation_mm'] = deviation
                    comparison['passed'] = (actual['valid'] and comparison['domain_equal']
                                             and deviation is not None and deviation <= 1e-9)
                else:
                    comparison['passed'] = actual['valid'] and isinstance(
                        next(obj.Geometry for obj in model.Objects
                             if str(obj.Attributes.ObjectId).lower() == native['uuid'].lower()),
                        Rhino.Geometry.Brep)
                comparisons.append(comparison)
            unchanged = checksum(path) == before
            passed = (unchanged and not missing and len(objects) == expected['expected_objects']
                      and all(row['passed'] for row in comparisons))
            report['cases'].append({'fixture': expected['fixture'], 'source_sha256': before,
                                    'objects': objects, 'missing_objects': missing,
                                    'comparisons': comparisons, 'input_unchanged': unchanged,
                                    'passed': passed})
            print('OM9 coherent fixture ' + expected['fixture'] + ': ' + str(passed))
        finally:
            model.Dispose()
    report['ok'] = len(report['cases']) == 4 and all(row['passed'] for row in report['cases'])
except Exception:
    report['error'] = traceback.format_exc()
with open(OUTPUT, 'wb') as stream:
    stream.write(json.dumps(report, ensure_ascii=True, indent=2).encode('ascii'))
print('OM9 Rhino5 coherent reference probe output: ' + OUTPUT)
print('OM9 Rhino5 coherent reference probe completed: ' + str(report['ok']))
