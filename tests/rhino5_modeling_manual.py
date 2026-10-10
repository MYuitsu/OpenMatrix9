# Run inside Rhino 5 with _-RunPythonScript; IronPython 2 compatible.
import os
import json
import codecs
import hashlib
import uuid
import traceback
import datetime

ROOT = 'H:/FreeCAD-src/build'
PROBE = ROOT + '/om9-dev/tests/rhino5_modeling_exchange.py'
MANUAL = ROOT + '/om9-dev/tests/rhino5_modeling_manual.py'
MODULE = ROOT + '/om9-modeling-sdk/bin/OpenMatrix9Gui.pyd'
DOCUMENT_HELPER = ROOT + '/om9-dev/tests/rhino5_modeling_document.py'


def digest(path):
    with open(path, 'rb') as stream:
        return hashlib.sha256(stream.read()).hexdigest()


def verified_directory(suite):
    base = ROOT + '/om9-dev/build/' + suite + '-1'
    candidates = []
    for name in os.listdir(base):
        report_path = os.path.join(base, name, 'results.json')
        if not os.path.isfile(report_path):
            continue
        try:
            with codecs.open(report_path, 'r', 'utf-8-sig') as stream:
                passed = json.load(stream).get('ok', False)
        except (ValueError, IOError):
            continue
        if passed:
            candidates.append((os.path.getmtime(report_path), report_path))
    if not candidates:
        raise RuntimeError('No passing report for ' + suite)
    return os.path.dirname(max(candidates)[1])


def prepare_config(include_acceptance=False):
    source_dir = verified_directory('modeling_exchange_smoke')
    fixtures = [
        ('edited-and-new', os.path.join(source_dir, 'edited-and-new.3dm'), 2),
        ('edited-cloud', os.path.join(source_dir, 'edited-cloud.3dm'), 1),
        ('placed-cad', os.path.join(source_dir, 'placed-cad.3dm'), 1),
        ('mm', ROOT + '/3dm-preserve-native/modeling-fixtures/mm-current.3dm', 2),
        ('cm-converted-mm', ROOT + '/3dm-preserve-native/modeling-fixtures/cm-current.3dm', 2),
    ]
    if include_acceptance:
        primitives = verified_directory('modeling_primitives_smoke')
        performance = verified_directory('modeling_performance_smoke')
        ring = verified_directory('modeling_ring_roundtrip_smoke')
        fixtures += [
            ('source-and-link', os.path.join(source_dir, 'source-and-link.3dm'), 2),
            ('current-point', os.path.join(primitives, 'point-current.3dm'), 1),
            ('current-extrusion', os.path.join(primitives, 'extrusion-current.3dm'), 1),
            ('current-mesh', os.path.join(performance, 'current-1.3dm'), 1),
            ('current-ring', os.path.join(ring, 'ring-current-and-new-v5.3dm'), 29),
        ]
    files = []
    for name, path, count in fixtures:
        if not os.path.isabs(path) or not os.path.isfile(path):
            raise RuntimeError('Missing absolute Rhino5 fixture: ' + path)
        files.append({'name': name, 'input': path, 'count': count,
                      'input_sha256': digest(path)})
    return {'files': files, 'script': PROBE, 'script_sha256': digest(PROBE),
            'launcher_sha256': digest(MANUAL),
            'host_module_sha256': digest(MODULE),
            'document_helper': DOCUMENT_HELPER, 'document_helper_sha256': digest(DOCUMENT_HELPER),
            'launch_mode': 'manual-user-started', 'fixture_directory': source_dir}


def run(only_names=None, include_acceptance=False):
    import Rhino
    import System
    output = None
    previous_output = os.environ.get('OM9_RHINO5_OUTPUT')
    launch = {'launch_mode': 'manual-user-started', 'ok': False,
              'utc': datetime.datetime.utcnow().isoformat() + 'Z'}
    try:
        document_helpers = {'__name__': 'document_helpers', '__file__': DOCUMENT_HELPER}
        execfile(DOCUMENT_HELPER, document_helpers, document_helpers)
        doc = Rhino.RhinoDoc.ActiveDoc
        if doc is None or doc.Path or document_helpers['document_objects'](doc):
            raise RuntimeError('Open a NEW, EMPTY, UNSAVED document before running this test.')
        config = prepare_config(include_acceptance)
        if only_names is not None:
            config['files'] = [entry for entry in config['files'] if entry['name'] in only_names]
            if len(config['files']) != len(only_names):
                raise RuntimeError('Requested diagnostic cases are missing')
            config['diagnostic_subset'] = list(only_names)
        config['echo_commands'] = True
        config['export_selected'] = include_acceptance
        output = ROOT + '/modeling-rhino5/' + uuid.uuid4().hex
        os.makedirs(output)
        pid = System.Diagnostics.Process.GetCurrentProcess().Id
        launch['pid'] = pid
        launch['initial_document_empty'] = True
        launch['executable'] = System.Diagnostics.Process.GetCurrentProcess().MainModule.FileName
        launch['rhino_version'] = str(Rhino.RhinoApp.Version)
        with open(os.path.join(output, 'config.json'), 'w') as stream:
            json.dump(config, stream, indent=2)
        with open(os.path.join(output, 'owned-pid.txt'), 'w') as stream:
            stream.write(str(pid))
        Rhino.RhinoApp.WriteLine('OpenMatrix9 Rhino5 test output: ' + output)
        os.environ['OM9_RHINO5_OUTPUT'] = output
        scope = {'__name__': '__main__', '__file__': PROBE}
        execfile(PROBE, scope, scope)
        report_path = os.path.join(output, 'rhino5-results.json')
        with open(report_path, 'r') as stream:
            result = json.load(stream)
        launch['report_exists'] = True
        launch['ok'] = bool(result.get('ok'))
        Rhino.RhinoApp.WriteLine('Rhino5 test: {0}; checks={1}; cases={2}'.format(
            'PASS' if launch['ok'] else 'FAIL', len(result.get('checks', [])),
            len(result.get('cases', []))))
        Rhino.RhinoApp.WriteLine('Report: ' + report_path)
        if not launch['ok']:
            Rhino.RhinoApp.WriteLine(result.get('error', 'Probe failed'))
    except Exception:
        launch['error'] = traceback.format_exc()
        Rhino.RhinoApp.WriteLine(launch['error'])
    finally:
        if previous_output is None:
            os.environ.pop('OM9_RHINO5_OUTPUT', None)
        else:
            os.environ['OM9_RHINO5_OUTPUT'] = previous_output
        if output:
            with open(os.path.join(output, 'manual-launch.json'), 'w') as stream:
                json.dump(launch, stream, indent=2)


if __name__ == '__main__':
    run()
