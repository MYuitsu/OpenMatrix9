# Owned application test only: this harness exits its own Rhino process.
import Rhino, scriptcontext as sc, System, os, json, ntpath, traceback, warnings

out = os.environ['OM9_PHASE2_VERIFY_SESSION_TEST_OUTPUT']
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
    script = 'H:/FreeCAD-src/build/om9-dev/tests/rhino5_verify_phase2.py'
    check('manual verification entry point exists', os.path.isfile(script))
    with open(script, 'rb') as stream:
        code = stream.read().replace('\r\n', '\n') + '\n'
    with warnings.catch_warnings(record=True) as captured:
        warnings.simplefilter('always')
        exec(compile(code, script, 'exec'), {'__name__': '__main__', '__file__': script})
    report['warnings'] = [str(item.message) for item in captured]
    check('caller document geometry attributes selection units and dirty state preserved', snapshot() == before)
    result_path = sc.sticky.get('OM9_PHASE2_VERIFY_LAST_REPORT')
    check('manual script returns a fresh report', result_path and os.path.isfile(result_path))
    with open(result_path, 'r') as stream:
        result = json.load(stream)
    check('manual workflow succeeds', result.get('ok'))
    check('actual Rhino three fixture names', sorted(result['fixtures']) == ['periodic', 'placed', 'rational'])
    check('actual Rhino assertions executed', result['rhino_checks'] >= 125)
    check('actual saved geometry reread executed', result['saved_reread_checks'] >= 55)
    check('manual reentry guard released', not sc.sticky.get('OM9_PHASE2_VERIFY_RUNNING'))
    check('manual wait does not warn about blocking STA message pumping', not any('STA thread' in str(item.message) for item in captured))
    report.update(ok=True, before=before, after=snapshot(), verification_report=result_path)
except:
    report['error'] = traceback.format_exc()
with open(ntpath.join(out, 'results.json'), 'w') as stream:
    json.dump(report, stream, indent=2)
System.Environment.Exit(0 if report['ok'] else 1)
