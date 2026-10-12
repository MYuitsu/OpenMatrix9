"""Certify contained point witnesses, not global extrema or roundtrip acceptance."""
from pathlib import Path
import hashlib
import json
import math


def vector(value, size):
    if not isinstance(value, list) or len(value) != size or not all(
            isinstance(x, (int, float)) and math.isfinite(x) for x in value):
        raise ValueError('Missing/non-finite vector')
    return value


def verify_witness(witness, bounds):
    bounds = vector(bounds, 6)
    if any(bounds[i] > bounds[i+3] for i in range(3)):
        raise ValueError('Inverted bounding box')
    point = vector(witness.get('point'), 3)
    native = vector(witness.get('native_point'), 3)
    vector(witness.get('uv'), 2)
    deviation = math.dist(point, native)
    outside = max([0.0] + [bounds[i]-point[i] for i in range(3)] +
                  [point[i]-bounds[i+3] for i in range(3)])
    verified = witness.get('relation') == 'Interior' and deviation <= 1e-8
    return dict(witness, native_point_deviation_mm=deviation,
                independently_verified=verified, outside_bounds_mm=outside,
                disproves_containment=verified and outside > 1e-8)


def analyze(report):
    if not report.get('ok') or not report.get('input_hashes_unchanged'):
        raise ValueError('Incomplete diagnostic or changed inputs')
    if not report.get('rhino_version', '').startswith('5.'):
        raise ValueError('Rhino 5 evidence required')
    cases = []
    if len(report.get('cases', [])) != 2:
        raise ValueError('Expected two independently diagnosed mirror cases')
    for case in report['cases']:
        rows = []
        if [g['label'] for g in case['geometry']] != ['original', 'export']:
            raise ValueError('Missing source/export geometry')
        for geometry in case['geometry']:
            if not geometry['source_unchanged'] or not geometry['before']['valid']:
                raise ValueError('Mutated/invalid geometry')
            if len(geometry.get('extrema_witnesses', [])) != 6:
                raise ValueError('Missing coordinate witnesses')
            rows.append(dict(label=geometry['label'],
                witnesses=[verify_witness(w, geometry['before']['bounds'])
                           for w in geometry['extrema_witnesses']]))
        raw_delta = max(abs(a-b) for a, b in zip(
            case['geometry'][0]['before']['bounds'], case['geometry'][1]['before']['bounds']))
        mesh_delta = max(abs(a-b) for a, b in zip(
            vector(case['geometry'][0]['diagnostic_mesh']['bounds'], 6),
            vector(case['geometry'][1]['diagnostic_mesh']['bounds'], 6)))
        cases.append(dict(source_uuid=case['source_uuid'], geometry=rows,
            raw_bbox_deviation_mm=raw_delta, raw_bbox_pass=raw_delta < .001,
            diagnostic_mesh_deviation_mm=mesh_delta,
            source_bbox_containment_disproved=any(w['disproves_containment'] for w in rows[0]['witnesses'])))
    return dict(scope='Independent contained-point evidence; does not certify global extrema or full roundtrip acceptance.',
                cases=cases, source_boxes_disproved=all(c['source_bbox_containment_disproved'] for c in cases))


if __name__ == '__main__':
    root = Path(__file__).parent
    path = root/'results.json'
    report = json.loads(path.read_text(encoding='utf-8'))
    result = analyze(report)
    result['actual_report_sha256'] = hashlib.sha256(path.read_bytes()).hexdigest()
    (root/'witness-analysis.json').write_text(json.dumps(result, indent=2)+'\n')
    print(json.dumps(result, indent=2))
