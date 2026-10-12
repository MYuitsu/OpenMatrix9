# -*- coding: utf-8 -*-
"""Rhino5 read-only prerequisite probe. Never opens/saves the active document."""
import Rhino
import json
import os
import hashlib
import traceback

ROOT = r'H:\FreeCAD-src\build\3dm-preserve-native\trim-domain-evidence'
OUTPUT = os.path.join(os.path.dirname(__file__), 'rhino5-reference-results.json')


def checksum(path):
    with open(path, 'rb') as stream:
        return hashlib.sha256(stream.read()).hexdigest()


report = {'ok': False, 'read_only': True, 'cases': [], 'rhino_version': str(Rhino.RhinoApp.Version)}
print('OM9 Rhino5 native reference probe started (read-only)')
try:
    for name in ('shape-0-edge-0-trim-0-segment-0-consistent-1.3dm',
                 'shape-1-edge-1-trim-1-segment-1-consistent-1.3dm'):
        path = os.path.join(ROOT, name)
        before = checksum(path)
        model = Rhino.FileIO.File3dm.Read(path)
        if model is None:
            raise RuntimeError('Rhino5 cannot read fixture: ' + name)
        try:
            objects = []
            for obj in model.Objects:
                geometry = obj.Geometry
                row = {'uuid': str(obj.Attributes.ObjectId), 'name': str(obj.Attributes.Name or ''),
                       'managed_type': str(geometry.GetType().FullName), 'valid': bool(geometry.IsValid),
                       'object_type': str(geometry.ObjectType)}
                curve = geometry if isinstance(geometry, Rhino.Geometry.Curve) else None
                if curve is not None:
                    domain = curve.Domain
                    row['domain'] = [float(domain.T0), float(domain.T1)]
                    row['samples'] = []
                    for i in range(17):
                        point = curve.PointAt(float(domain.T0) + (float(domain.T1)-float(domain.T0))*i/16.0)
                        row['samples'].append([float(point.X), float(point.Y), float(point.Z)])
                objects.append(row)
            passed = len(objects) == 2 and all(row['valid'] for row in objects) and checksum(path) == before
            report['cases'].append({'fixture': name, 'source_sha256': before, 'objects': objects, 'passed': passed})
        finally:
            model.Dispose()
    report['ok'] = all(case['passed'] for case in report['cases']) and len(report['cases']) == 2
except Exception:
    report['error'] = traceback.format_exc()
with open(OUTPUT, 'wb') as stream:
    stream.write(json.dumps(report, ensure_ascii=True, indent=2).encode('ascii'))
print('OM9 Rhino5 native reference probe output: ' + OUTPUT)
print('OM9 Rhino5 native reference probe completed: ' + str(report['ok']))
