# Owned application test only: this harness exits its own Rhino process.
import Rhino, scriptcontext as sc, System, os, json, ntpath, traceback, warnings

out = os.environ['OM9_PHASE12_VERIFY_SESSION_TEST_OUTPUT']
report = {'ok': False, 'checks': [], 'pid': os.getpid()}

def check(name, value):
    report['checks'].append({'name': name, 'passed': bool(value)})
    if not value:
        raise AssertionError(name)

def snapshot():
    return {
        'identity': sc.doc.GetHashCode(),
        'modified': sc.doc.Modified,
        'path': sc.doc.Path,
        'units': str(sc.doc.ModelUnitSystem),
        'tolerance': sc.doc.ModelAbsoluteTolerance,
        'objects': sorted((str(o.Id), o.Attributes.Name, str(o.Geometry.GetBoundingBox(True)), bool(o.IsSelected(False))) for o in sc.doc.Objects)
    }

try:
    sc.doc.ModelUnitSystem = Rhino.UnitSystem.Centimeters
    sc.doc.ModelAbsoluteTolerance = 0.002
    identifier = sc.doc.Objects.AddLine(Rhino.Geometry.Point3d(1, 2, 3), Rhino.Geometry.Point3d(9, 8, 7))
    obj = sc.doc.Objects.Find(identifier)
    attr = obj.Attributes.Duplicate()
    attr.Name = 'USER_DOCUMENT_SENTINEL'
    sc.doc.Objects.ModifyAttributes(identifier, attr, True)
    sc.doc.Objects.Select(identifier)
    before = snapshot()
    script = 'H:/FreeCAD-src/build/om9-perf-dev/tests/rhino5_verify_phase12_timing.py'
    check('manual verification entry point exists', os.path.isfile(script))
    with open(script, 'rb') as stream:
        code = stream.read().replace('\r\n', '\n') + '\n'
    with warnings.catch_warnings(record=True) as captured:
        warnings.simplefilter('always')
        exec(compile(code, script, 'exec'), {'__name__': '__main__', '__file__': script})
    report['warnings'] = [str(item.message) for item in captured]
    check('caller document geometry attributes selection units and dirty state preserved', snapshot() == before)
    result_path = sc.sticky.get('OM9_PHASE12_VERIFY_LAST_REPORT')
    check('manual script returns a fresh report', result_path and os.path.isfile(result_path))
    with open(result_path, 'r') as stream:
        result = json.load(stream)
    check('manual workflow succeeds', result.get('ok'))
    check('actual Rhino three fixture names', sorted(result['phase2']['fixtures']) == ['periodic', 'placed', 'rational'] and len(result['phase1']['fixtures']) == 11)
    check('actual Rhino assertions executed', result['rhino_checks'] >= 621)
    check('actual saved geometry reread executed', result['saved_reread_checks'] >= 175 and result['host_report_count'] == 38)
    check('candidate runtime is the optimized build', result['runtime'].replace('\\', '/').endswith('/om9-perf-sdk') and result['manifest_sha256'] == '5d06e5d1fd9070a14afb69800aa53f99c6001ddc1dc4bc159959f3e7476eac6d')
    check('manual reentry guard released', not sc.sticky.get('OM9_PHASE12_VERIFY_RUNNING'))
    check('manual wait does not warn about blocking STA message pumping', not any('STA thread' in str(item.message) for item in captured))
    report.update(ok=True, before=before, after=snapshot(), verification_report=result_path)
except:
    report['error'] = traceback.format_exc()
with open(ntpath.join(out, 'results.json'), 'w') as stream:
    json.dump(report, stream, indent=2)
System.Environment.Exit(0 if report['ok'] else 1)
