"""Record CI build identity and binary integrity without claiming runtime tests."""
import argparse
import hashlib
import json
import os
import re
import subprocess
from pathlib import Path


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def record_build(root, source, environ):
    paths = json.loads((root / 'build-paths.json').read_text(encoding='utf-8-sig'))
    plugin = Path(paths['plugin'])
    commit = subprocess.check_output(['git', '-C', str(source), 'rev-parse', 'HEAD'], text=True).strip()
    repository = environ.get('GITHUB_REPOSITORY', '')
    run_id = environ.get('GITHUB_RUN_ID', '')
    if not re.fullmatch(r'[0-9a-f]{40}', commit) or environ.get('GITHUB_SHA') != commit:
        raise RuntimeError('CI source commit does not match the checked-out build')
    if not re.fullmatch(r'[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+', repository) or not run_id.isdecimal():
        raise RuntimeError('CI repository and run identity required')
    release_path = source / 'release.json'
    release = json.loads(release_path.read_text(encoding='utf-8'))
    if release['freecad'] != '1.1.4':
        raise RuntimeError('Unsupported stock host')
    binaries = {'plugin_sha256': plugin / 'bin/OpenMatrix9Gui.pyd',
                'worker_sha256': plugin / 'bin/OM9ThreeDmImportWorker.exe'}
    for binary in binaries.values():
        if binary.is_symlink() or not binary.is_file() or binary.stat().st_size == 0:
            raise RuntimeError('Missing or linked compiled binary')
    record = dict(ok=True, validation_kind='build_only', tests_executed=False,
                  build=dict(ok=True, target='OpenMatrix9Gui'), version=release['version'],
                  release_metadata_sha256=sha(release_path),
                  host='Official FreeCAD 1.1.4 Windows x64', source_commit=commit,
                  build_url=f'https://github.com/{repository}/actions/runs/{run_id}',
                  **{key: sha(path) for key, path in binaries.items()})
    raw = json.dumps(record, indent=2) + '\n'
    (plugin / 'release-validation.json').write_text(raw, encoding='utf-8')
    evidence = root / 'evidence'
    evidence.mkdir(exist_ok=True)
    (evidence / 'summary.json').write_text(raw, encoding='utf-8')
    return record


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, required=True)
    args = parser.parse_args()
    print(json.dumps(record_build(args.root.resolve(), Path(__file__).resolve().parents[2], os.environ), indent=2))
