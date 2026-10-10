"""Static packaging gate; behavioral ownership is verified by host macros."""
from pathlib import Path
import argparse
import hashlib
import json
from phase2_evidence import verify


def check(runtime):
    runtime = Path(runtime)
    module = runtime / 'Mod/OpenMatrix9'
    if (module / 'ModelingCurveEditor.py').exists():
        raise RuntimeError('Production Python PointsOn editor remains packaged')
    manifest = runtime / 'phase2-build.json'
    data = verify(manifest)
    if data.get('migration') != 'phase2-rust-owned-core':
        raise RuntimeError('Missing migrated runtime manifest')
    root = Path(data['source_root'])
    for name in ['phase2_curve.rs', 'phase2_session.rs', 'phase2_snap.rs']:
        if '#![forbid(unsafe_code)]' not in (root/'rust/src'/name).read_text():
            raise RuntimeError('Safe core guard missing: '+name)
    if (root/'ModelingCurveEditor.py').exists():
        raise RuntimeError('Production Python PointsOn source remains')
    return {'ok': True, 'scope': 'packaging and safe-core compiler guards; host macros prove behavior',
            'manifest': str(manifest), 'manifest_sha256': hashlib.sha256(manifest.read_bytes()).hexdigest(),
            'native_module_sha256': data['runtime_sha256']['bin/OpenMatrix9Gui.pyd'],
            'python_points_on_removed': True, 'safe_core_guards': 3}


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('runtime')
    parser.add_argument('--output')
    args = parser.parse_args()
    result = check(args.runtime)
    if args.output:
        Path(args.output).write_text(json.dumps(result, indent=2))
    print(json.dumps(result))
