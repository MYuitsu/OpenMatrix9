"""Reproducible source/runtime coverage without class-wide inference from slices."""
from pathlib import Path
import argparse
import hashlib
import json
import re

AXES = ('read', 'preserve', 'display', 'edit', 'write', 'rhino5_write')
COMMENTS = re.compile(r'("(?:\\.|[^"\\])*"|\'(?:\\.|[^\'\\])*\')|//[^\n]*|/\*[\s\S]*?\*/')
REGISTRATION = re.compile(r'\b(ON_OBJECT_IMPLEMENT(?:_NO_COPYCTOR|_NO_COPY)?|ON_VIRTUAL_OBJECT_IMPLEMENT)\s*\(\s*(\w+)\s*,\s*(\w+)\s*,\s*"([0-9a-fA-F-]+)"')
MANUAL = re.compile(r'\bconst\s+ON_ClassId\s+[\w:]+\s*\(\s*"(\w+)"\s*,\s*"(\w+)"\s*,\s*[^,]+,\s*"([0-9a-fA-F-]+)"')
NON_ARCHIVAL_PROXIES = {'ON_CurveProxy', 'ON_SurfaceProxy'}
# These are reviewed named slices, not automated interpretations of report prose.
CLAIM_AXES = {
    '2026-10-07-3dm-text-dot.md': ('read', 'preserve', 'display', 'edit', 'write', 'rhino5_write'),
    '2026-10-07-3dm-point-cloud.md': ('read', 'preserve', 'display', 'edit'),
    '2026-10-07-3dm-cloud-geometry-legacy5.md': ('read', 'preserve', 'display', 'write', 'rhino5_write'),
    '2026-10-07-3dm-hatch-current.md': ('read', 'preserve', 'display', 'edit', 'write'),
    '2026-10-07-3dm-hatch-loop-edit.md': ('read', 'preserve', 'display', 'edit', 'write'),
    '2026-10-07-3dm-hatch-loop-ui.md': ('display', 'edit'),
    '2026-10-07-3dm-hatch-loop-rows.md': ('edit', 'write'),
    '2026-10-07-3dm-hatch-legacy5.md': ('read', 'preserve', 'write', 'rhino5_write'),
    '2026-10-07-3dm-hatch-legacy-loops.md': ('read', 'preserve', 'write', 'rhino5_write'),
    '2026-10-07-curve-on-surface-archive-recovery.md': ('read', 'preserve'),
    '2026-10-07-curve-on-surface-native-schema.md': ('read', 'preserve'),
    '2026-10-07-native-reference-resolution.md': ('read',),
    '2026-10-07-reference-graph-integrity.md': ('read',),
    '2026-10-07-trim-domain-correspondence.md': ('read',),
}


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def property_slices(project, names):
    path=project/'docs/3dm-capability-slices.json'
    if not path.is_file():
        return {}
    document=json.loads(path.read_text(encoding='utf-8'))
    if document['schema_version']!=1:
        raise ValueError('Unsupported property capability schema')
    result={}
    allowed={'verified','preserved_only','incompatible','not_applicable','unverified'}
    for item in document['slices']:
        if item['class_name'] not in names or not item['scope']:
            raise ValueError('Unaccounted class or empty property scope')
        if not item['axes'] or any(axis not in AXES or status not in allowed for axis,status in item['axes'].items()):
            raise ValueError('Unknown property capability axis or status')
        evidence=[]
        for name in item['evidence']:
            target=(project/name).resolve()
            if not target.is_relative_to(project.resolve()) or not target.is_file():
                raise ValueError('Missing or non-project property evidence: '+name)
            evidence.append(dict(file=name,sha256=sha(target)))
        if not evidence:
            raise ValueError('Property capability requires evidence')
        result.setdefault(item['class_name'],[]).append(dict(item,evidence=evidence,class_wide_completion=False))
    return result


def discover(sdk):
    found = {}
    for path in sorted(sdk.glob('*.cpp')):
        text = path.read_text(encoding='utf-8-sig', errors='strict')
        clean = COMMENTS.sub(lambda m: m.group(1) if m.group(1) else '\n' * m.group(0).count('\n'), text)
        for match in REGISTRATION.finditer(clean):
            name = match[2]
            if name in found:
                raise ValueError('Duplicate registration: ' + name)
            found[name] = dict(file=path.name, line=clean.count('\n', 0, match.start()) + 1,
                               macro=match[1], base_class=match[3], class_uuid=match[4].lower(),
                               source_sha256=sha(path))
        for match in MANUAL.finditer(clean):
            name = match[1]
            if name in found:
                raise ValueError('Duplicate manual registration: ' + name)
            found[name] = dict(file=path.name, line=clean.count('\n', 0, match.start()) + 1,
                               macro='manual_ON_ClassId', base_class=match[2], class_uuid=match[3].lower(),
                               source_sha256=sha(path))
    if not found:
        raise ValueError('No source registrations found')
    return found


def normalize(project, sdk, historical, runtime):
    if runtime.get('ok') is not True:
        raise ValueError('Runtime probe failed')
    found = discover(sdk)
    properties=property_slices(project,set(found))
    original = {r['class_name']: r for r in historical['classes']}
    live = {r['class_name']: r for r in runtime['classes']}
    if len(original) != len(historical['classes']) or len(live) != len(runtime['classes']):
        raise ValueError('Duplicate catalog/runtime identity')
    if set(live) - set(found):
        raise ValueError('Unaccounted runtime registrations: ' + str(sorted(set(live) - set(found))))
    # These three source registrations are intentionally absent from pinned SDK CMake.
    # Account for them as unavailable, not as successful exchange adapters.
    source_only = set(found) - set(live)
    excluded_build = {'ON_Internal_ObsoleteUserData', 'ON_OBSOLETE_IDefLayerSettingsUserData', 'ON_OBSOLETE_LayerSettingsUserData'}
    if source_only != excluded_build:
        raise ValueError('Unexpected source/runtime registration difference: ' + str(sorted(source_only)))
    cmake = (sdk / 'CMakeLists.txt').read_text(encoding='utf-8')
    if 'opennurbs_userdata_obsolete.cpp' in cmake:
        raise ValueError('SDK obsolete build exclusion changed; audit required')
    rows = []
    for name in sorted(found):
        source = found[name]
        old = original.get(name, dict(class_name=name, class_uuid=source['class_uuid'], base_class=source['base_class'],
            geometry_class=False, classification='obsolete-compatibility' if 'OBSOLETE' in name else 'concrete'))
        actual = live.get(name)
        if actual and any(source[k] != actual[k] for k in ('base_class', 'class_uuid')):
            raise ValueError('Runtime class identity mismatch: ' + name)
        if source['class_uuid'] != old['class_uuid'].lower() or source['base_class'] != old['base_class']:
            raise ValueError('Historical class identity mismatch: ' + name)
        abstract = actual is not None and not actual['factory_available']
        helper = name in NON_ARCHIVAL_PROXIES
        applicability_evidence = dict(source_sha256=source['source_sha256'],
                                      file=source['file'], line=source['line'],
                                      runtime_factory_available=actual['factory_available'] if actual else False)
        if helper:
            text = (sdk/source['file']).read_text(encoding='utf-8-sig')
            for method in ('Read', 'Write'):
                if not re.search(re.escape(name+'::'+method)+r'\s*\([^)]*\)\s*(?:const\s*)?\{\s*return false;\s*\}', text):
                    raise ValueError('Proxy archival applicability changed: '+name+'::'+method)
            applicability_evidence['reason'] = 'Base proxy Read and Write both return false; reference ownership belongs to a concrete owner adapter'
        claims = []
        for item in old.get('tested_slices', []):
            evidence = project / item['evidence']
            if not evidence.is_file():
                raise ValueError('Missing scoped evidence: ' + str(evidence))
            claims.append(dict(item, evidence_sha256=sha(evidence), class_wide_completion=False,
                               axes=list(CLAIM_AXES.get(evidence.name, ())),
                               actual_rhino5_application='not_inferred_from_native_slice'))
        capabilities = {}
        for axis in AXES:
            supported = [c['evidence'] for c in claims if axis in c['axes']]
            for item in properties.get(name,[]):
                if item['axes'].get(axis)=='verified':
                    supported.extend(e['file'] for e in item['evidence'])
            capabilities[axis] = dict(status='not_applicable' if abstract or helper else 'scoped' if supported else 'unverified',
                                      evidence=sorted(set(supported)), class_wide_completion=False)
        if name not in original:
            parent = source['base_class']
            while parent in found:
                if parent == 'ON_Geometry':
                    old['geometry_class'] = True
                    break
                parent = found[parent]['base_class']
        rows.append(dict(class_name=name, class_uuid=source['class_uuid'], base_class=source['base_class'],
                         source_registration=source, classification='helper' if helper else 'abstract' if abstract else old['classification'],
                         legacy_compatibility='OBSOLETE' in name, applicability_evidence=applicability_evidence,
                         applicability='source_only_not_built' if actual is None else 'abstract_base' if abstract else 'runtime_proxy_non_archival' if helper else 'legacy_upgrade' if old['classification']=='obsolete-compatibility' else 'owned_brep_subobject' if name in ('ON_BrepVertex','ON_BrepEdge','ON_BrepTrim','ON_BrepLoop','ON_BrepFace','ON_BrepFaceSide','ON_BrepRegion') else 'native_record_or_owned_subobject',
                         geometry_class=old['geometry_class'], runtime_registration=dict(actual, verified=True) if actual else dict(verified=False, reason='opennurbs_userdata_obsolete.cpp absent from pinned SDK CMake', cmake_sha256=sha(sdk/'CMakeLists.txt')),
                         capabilities=capabilities, scoped_claims=claims, property_slices=properties.get(name,[]), class_wide_completion=False))
    excluded = []
    for name in sorted(set(original) - set(found)):
        row = original[name]
        path = sdk / row['registration']['file']
        line = path.read_text(encoding='utf-8-sig').splitlines()[row['registration']['line'] - 1]
        if not line.lstrip().startswith('//') or name not in line:
            raise ValueError('Unexplained historical registration: ' + name)
        excluded.append(dict(class_name=name, reason='commented registration; no linked runtime ClassId',
                             registration=row['registration'], source_sha256=sha(path)))
    return dict(schema_version=2, pinned_revision=historical['pinned_revision'], classes=rows,
                excluded_comment_registrations=excluded, runtime_unaccounted=[], runtime_count=len(live),
                runtime_probe_sha256=runtime['probe_sha256'], runtime_scope=runtime['scope'],
                component_categories=historical['component_categories'], document_categories=historical['document_categories'],
                historical_scope_flags_authoritative=False, full_exchange_complete=False, completion_percentage=None)


def bind_installed_registry(runtime_path, installed_path):
    runtime = json.loads(runtime_path.read_text(encoding='utf-8'))
    installed = json.loads(installed_path.read_text(encoding='utf-8'))
    expected = {row['class_name']: row for row in runtime['classes']}
    actual = {row['class_name']: row for row in installed['classes']}
    if (not runtime['ok'] or not installed['ok'] or actual != expected
            or len(actual) != len(installed['classes'])
            or installed['runtime_count'] != len(actual)
            or installed['standalone_report_sha256'] != sha(runtime_path)):
        raise ValueError('Installed registry does not match the standalone runtime evidence')
    binary = Path(installed['probe_binary'])
    if not binary.is_file() or installed['probe_sha256'] != sha(binary):
        raise ValueError('Installed module changed after the application registry probe')
    return dict(report=str(installed_path), report_sha256=sha(installed_path),
                module=str(binary), module_sha256=sha(binary),
                freecad_binary_sha256=installed['freecad_binary_sha256'],
                runtime_count=len(actual), scope=installed['scope'], exchange_inferred=False)


def main():
    project = Path(__file__).resolve().parents[1]
    parser = argparse.ArgumentParser()
    parser.add_argument('--sdk-root', type=Path, default=project.parent / 'dependencies/opennurbs')
    parser.add_argument('--runtime', type=Path, default=project.parent / '3dm-preserve-native/coverage-runtime.json')
    parser.add_argument('--installed-runtime', type=Path)
    parser.add_argument('--output', type=Path, default=project / 'docs/3dm-capabilities.json')
    args = parser.parse_args()
    historical_path = project / 'docs/3dm-coverage.json'
    result = normalize(project, args.sdk_root, json.loads(historical_path.read_text(encoding='utf-8')),
                       json.loads(args.runtime.read_text(encoding='utf-8')))
    result['historical_coverage_sha256'] = sha(historical_path)
    result['runtime_report_sha256'] = sha(args.runtime)
    if args.installed_runtime:
        result['installed_runtime_evidence'] = bind_installed_registry(args.runtime, args.installed_runtime)
    args.output.write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print(f"Coverage PASS: {len(result['classes'])} source classes, {result['runtime_count']} linked runtime classes; {len(result['excluded_comment_registrations'])} comments excluded; full support remains unproven")


if __name__ == '__main__':
    main()
