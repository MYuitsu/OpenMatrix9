# Verify measured source/binary correspondence without promoting full gates.
from pathlib import Path
import json
import hashlib

root = Path(__file__).resolve().parents[1]
def digest(path):
    with path.open('rb') as file:
        return hashlib.file_digest(file, 'sha256').hexdigest()

native = root / 'docs/validation/2026-10-14-native-retained-current-conversion.json'
report = json.loads(native.read_text())
assert len(report['runs']) == 50 and len(report['rejection_cases']) == 40
assert report['binary_sha256'] == digest(root / 'build/standalone-native/Release/om9-exchange-native.exe')
assert report['runner_sha256'] == digest(root / 'standalone/target/release/om9-native-runner.exe')
for row in report['authored_sources']:
    assert row['sha256'] == digest(root / row['path']), row['path']
checked = 0
for name in ['docs/validation/2026-10-13-standalone-ui/ui-receipt.json',
             'docs/validation/2026-10-14-viewport-title-menu/ui-receipt.json',
             'docs/validation/2026-10-13-standalone-process.json']:
    data = json.loads((root / name).read_text())
    for row in data['sources'] + [data['binary']]:
        assert row['sha256'] == digest(root / row['path']), (name, row['path'])
        checked += 1
    if 'screenshot' in data:
        assert data['screenshot']['sha256'] == digest(root / data['screenshot']['path'])
    if name.endswith('2026-10-14-viewport-title-menu/ui-receipt.json'):
        assert data['status'] == 'validated_viewport_title_menu_slice' and len(data['checks']) == 6
        assert data['objects'] == 0 and data['undo'] == 0 and not data['goal_complete']
        assert data['harness']['sha256'] == digest(root/data['harness']['path'])
exchange = json.loads((root / 'docs/validation/2026-10-13-standalone-exchange.json').read_text())
for path, sha in exchange['source_sha256'].items():
    assert sha == digest(root / path), path
    checked += 1
assert exchange['native_sha256'] == report['binary_sha256']
assert exchange['rust_lifecycle_tests_passed'] == 7
resource = json.loads((root / 'docs/validation/2026-10-13-standalone-resources.json').read_text())
for path, sha in resource['source_sha256'].items():
    assert sha == digest(root / path), path
    checked += 1
assert resource['binary']['sha256'] == digest(root / resource['binary']['path'])
assert resource['rust_tests_passed'] == 62 and len(resource['cold_process_cases']) == 6
workspace = json.loads((root/'docs/validation/2026-10-13-standalone-workspace.json').read_text())
assert workspace['status'] == 'validated_workspace_tests' and workspace['ignored'] > 0
for path, sha in workspace['source_sha256'].items():
    assert sha == digest(root/path), path
    checked += 1
assert workspace['log']['sha256'] == digest(root/workspace['log']['path'])
source = json.loads((root/'docs/validation/2026-10-14-native-retained-current-inventory.json').read_text())
assert source['status'] == 'validated_source_inventory_and_circle_correspondence_slice'
assert source['runs'] == 68 and source['rejections'] == 17 and not source['goal_complete']
assert source['binary_sha256'] == report['binary_sha256']
assert source['runner_sha256'] == report['runner_sha256']
for path, key in [('standalone/native/native_exchange.cpp','source_sha256'),
                  ('standalone/native/retained_merge.hpp','header_sha256'),
                  ('standalone/native/source_inventory_fixture.cpp','fixture_source_sha256'),
                  ('tools/validate_native_source_inventory.py','validator_sha256'),
                  ('build/native-source-fixture/Release/om9-source-fixture.exe','fixture_binary_sha256')]:
    assert source[key] == digest(root/path), path
for row in source['owner_sources']:
    assert row['sha256'] == digest(root/row['path']), row['path']
    checked += 1
assert len(source['operation_receipts']) == 68
assert all(r['stats']['exited'] and r['stats']['active_processes'] == 0 for r in source['operation_receipts'])
imported = json.loads((root/'docs/validation/2026-10-13-standalone-import.json').read_text())
assert imported['rust_native_import_tests_passed'] == 8 and imported['fixtures'] == 15 and not imported['goal_complete']
for path, sha in imported['source_sha256'].items():
    assert sha == digest(root/path), path
    checked += 1
for path, sha in imported['binary_sha256'].items():
    assert sha == digest(root/path), path
    checked += 1
assert imported['log']['sha256'] == digest(root/imported['log']['path'])
projection = json.loads((root/'docs/validation/2026-10-14-native-retained-current-projection.json').read_text())
assert projection['runs'] == 82 and projection['rejections'] == 32 and not projection['goal_complete']
assert projection['binary_sha256'] == report['binary_sha256'] and projection['runner_sha256'] == report['runner_sha256']
for path, key in [('standalone/native/native_exchange.cpp','source_sha256'),
                  ('standalone/native/retained_merge.hpp','header_sha256'),
                  ('standalone/native/source_inventory_fixture.cpp','fixture_source_sha256'),
                  ('tools/validate_native_source_projection.py','validator_sha256'),
                  ('build/native-source-fixture/Release/om9-source-fixture.exe','fixture_binary_sha256')]:
    assert projection[key] == digest(root/path), path
assert len(projection['operation_receipts']) == 82
assert all(r['stats']['exited'] and r['stats']['active_processes'] == 0 for r in projection['operation_receipts'])
retained = json.loads((root/'docs/validation/2026-10-14-standalone-retained-inputs.json').read_text())
assert retained['rust_tests_passed'] == 4 and retained['fixtures'] == 15 and retained['native_runs'] == 36
assert not retained['goal_complete'] and retained['source_schema'] == 4
for group in ['source_sha256','binary_sha256']:
    for path,sha in retained[group].items():
        assert sha == digest(root/path), path
        checked += 1
assert retained['log']['sha256'] == digest(root/retained['log']['path'])
assert all(r['stats']['exited'] and r['stats']['active_processes'] == 0 for r in retained['runs'])
current = json.loads((root/'docs/validation/2026-10-14-standalone-retained-current.json').read_text())
assert current['status'] == 'validated_selected_retained_current_cold_publication'
assert current['rust_tests_passed'] == 4 and current['native_runs'] == 78 and not current['goal_complete']
assert (current['authored_source_files'],current['successful_private_output_files'],current['published_output_files']) == (12,21,13)
assert current['expected_publication_refusals'] == 7
for group in ['source_sha256','binary_sha256']:
    for path,sha in current[group].items():
        assert sha == digest(root/path), path
        checked += 1
assert current['log']['sha256'] == digest(root/current['log']['path'])
assert len(current['runs']) == current['native_runs']
assert all(r['exited'] and r['active_processes'] == 0 for r in current['runs'])
selected = json.loads((root/'docs/validation/2026-10-14-native-retained-current-selected.json').read_text())
assert selected['total_runs'] == len(selected['runs']) == 107 and selected['expected_rejections'] == 34
assert (selected['sdk_authored_source_files'],selected['successful_writer_output_files'],selected['authored_valid_stale_cache_output_files'],selected['semantic_scenarios']) == (8,23,7,57)
for path,key in [('build/standalone-native/Release/om9-exchange-native.exe','binary_sha256'),
                 ('standalone/native/native_exchange.cpp','source_sha256'),
                 ('standalone/native/retained_merge.hpp','header_sha256'),
                 ('standalone/native/retained_current_fixture.cpp','fixture_source_sha256'),
                 ('build/native-retained-current-fixture/Release/om9-retained-current-fixture.exe','fixture_binary_sha256'),
                 ('tools/validate_native_retained_current.py','validator_sha256'),
                 ('standalone/target/release/om9-native-runner.exe','runner_sha256')]:
    assert selected[key] == digest(root/path), path
    checked += 1
for path,sha in selected['dependencies'].items():
    assert sha == digest(root/path), path
    checked += 1
assert all(r['stats']['exited'] and r['stats']['active_processes'] == 0 for r in selected['runs'])
assert selected['binary_sha256'] == report['binary_sha256']
ledger = json.loads((root/'OpenMatrix9_Codex_Spec_v2/MIGRATION_LEDGER.json').read_text())
assert ledger['foundation_implementation']['rust_tests_passed'] == workspace['passed']+workspace['ignored']
assert not ledger['foundation_implementation']['feature_completion_claim']
assert ledger['target_implementation_count'] == 0
assert workspace['ignored'] == exchange['rust_lifecycle_tests_passed']+imported['rust_native_import_tests_passed']+retained['rust_tests_passed']+current['rust_tests_passed']
print(json.dumps({'source_binary_hashes_verified': checked, 'native_jobs': 50,
                  'native_rejections': 40, 'native_lifecycle_tests': 7, 'resource_tests':62,
                  'workspace_tests':workspace['passed'], 'workspace_suites':workspace['suites'],
                  'native_source_jobs':68, 'native_source_rejections':17,
                  'native_import_tests':8, 'native_projection_jobs':82, 'native_projection_rejections':32,
                  'native_retained_input_tests':retained['rust_tests_passed'], 'native_retained_input_jobs':retained['native_runs'],
                  'native_retained_current_tests':current['rust_tests_passed'], 'native_retained_current_jobs':current['native_runs'],
                  'native_selected_jobs':selected['total_runs'], 'native_selected_rejections':selected['expected_rejections'],
                  'distinct_rust_tests':workspace['passed']+workspace['ignored'],
                  'cold_resource_cases':6, 'completion_claim': False}))
