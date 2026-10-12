import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

TOOL = Path(__file__).resolve().parents[1] / 'opennurbs_coverage.py'


class CoverageTests(unittest.TestCase):
    def load(self):
        spec = importlib.util.spec_from_file_location('coverage_tool', TOOL)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        return module

    def test_cpp_comments_not_registrations(self):
        module = self.load()
        text = '''// ON_OBJECT_IMPLEMENT(ON_Comment, ON_Object, "00000000-0000-0000-0000-000000000001")
/* ON_OBJECT_IMPLEMENT(ON_Block, ON_Object, "00000000-0000-0000-0000-000000000002") */
const char* note="// not a comment";
ON_OBJECT_IMPLEMENT(ON_Real, ON_Object, "00000000-0000-0000-0000-000000000003")
'''
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)
            (path / 'test.cpp').write_text(text)
            registrations = module.discover(path)
        self.assertEqual(set(registrations), {'ON_Real'})
        self.assertEqual(registrations['ON_Real']['line'], 4)

    def test_manual_and_non_copy_registration_are_discovered(self):
        module = self.load()
        text = '''ON_OBJECT_IMPLEMENT_NO_COPYCTOR(ON_Child, ON_Geometry, "00000000-0000-0000-0000-000000000001")
const ON_ClassId ON_Legacy::m_rtti("ON_Legacy", "ON_ArcCurve", MakeLegacy, "00000000-0000-0000-0000-000000000002");
'''
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)
            (path / 'test.cpp').write_text(text)
            registrations = module.discover(path)
        self.assertEqual(set(registrations), {'ON_Child', 'ON_Legacy'})

    def test_actual_sdk_catalog_matches_runtime_without_promoting_exchange(self):
        module = self.load()
        project = TOOL.parents[1]
        historical = json.loads((project / 'docs/3dm-coverage.json').read_text())
        sdk = project.parent / 'dependencies/opennurbs'
        runtime = json.loads((project.parent / '3dm-preserve-native/coverage-runtime.json').read_text())
        result = module.normalize(project, sdk, historical, runtime)
        self.assertEqual(len(result['classes']), 131)
        self.assertEqual(result['runtime_count'], 128)
        self.assertEqual(len(result['excluded_comment_registrations']), 5)
        self.assertEqual(result['runtime_unaccounted'], [])
        self.assertFalse(result['full_exchange_complete'])
        self.assertIsNone(result['completion_percentage'])
        self.assertEqual(len(result['component_categories']), 16)
        self.assertEqual(len(result['document_categories']), 6)
        for row in result['classes']:
            self.assertEqual(row['runtime_registration']['verified'], row['applicability'] != 'source_only_not_built')
            self.assertFalse(row['class_wide_completion'])
            self.assertNotEqual(row['capabilities']['rhino5_write']['status'], 'validated')
        text = next(row for row in result['classes'] if row['class_name'] == 'ON_TextDot')
        self.assertTrue(any('current-field editing' in c['name'] for c in text['scoped_claims']))
        self.assertEqual(text['capabilities']['edit']['status'], 'scoped')
        self.assertTrue(all(c['evidence_sha256'] for c in text['scoped_claims']))
        for name in ('ON_CurveProxy', 'ON_SurfaceProxy'):
            helper = next(row for row in result['classes'] if row['class_name'] == name)
            self.assertEqual(helper['classification'], 'helper')
            self.assertEqual(helper['applicability'], 'runtime_proxy_non_archival')
            self.assertEqual(helper['capabilities']['write']['status'], 'not_applicable')
            self.assertTrue(helper['applicability_evidence']['source_sha256'])

    def test_missing_and_mismatched_runtime_registration_are_errors(self):
        module = self.load()
        project = TOOL.parents[1]
        coverage = json.loads((project / 'docs/3dm-coverage.json').read_text())
        sdk = project.parent / 'dependencies/opennurbs'
        for runtime in ({'ok': True, 'classes': []}, {'ok': False, 'classes': []}):
            with self.assertRaises(ValueError):
                module.normalize(project, sdk, coverage, runtime)

    def test_installed_registry_evidence_requires_matching_classes_and_binary(self):
        module = self.load()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            runtime = root/'runtime.json'
            binary = root/'OpenMatrix9Gui.pyd'
            binary.write_bytes(b'actual loaded binary')
            rows = [{'class_name': 'ON_Test', 'class_uuid': 'test', 'factory_available': True}]
            runtime.write_text(json.dumps({'classes': rows, 'ok': True}))
            installed = dict(ok=True, classes=rows, runtime_count=1,
                             standalone_report_sha256=module.sha(runtime),
                             probe_binary=str(binary), probe_sha256=module.sha(binary),
                             freecad_binary_sha256='a'*64, scope='actual isolated FreeCAD')
            report = root/'installed.json'
            report.write_text(json.dumps(installed))
            proof = module.bind_installed_registry(runtime, report)
            self.assertEqual(proof['report_sha256'], module.sha(report))
            self.assertEqual(proof['module_sha256'], module.sha(binary))
            self.assertFalse(proof['exchange_inferred'])
            for field, bad in [('classes', []), ('probe_sha256', '0'*64),
                               ('standalone_report_sha256', '0'*64), ('ok', False)]:
                report.write_text(json.dumps(dict(installed, **{field: bad})))
                with self.assertRaises(ValueError):
                    module.bind_installed_registry(runtime, report)

    def test_property_slices_bind_evidence_without_promoting_incompatibility(self):
        module=self.load()
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory)
            (root/'docs').mkdir()
            (root/'proof.json').write_text('{"ok":false,"observed":"target loses object"}')
            entry=dict(class_name='ON_Test',scope='optional child',axes=dict(read='verified',rhino5_write='incompatible'),
                       evidence=['proof.json'],limitations=['target drops child'],evidence_kind='actual_rhino5')
            path=root/'docs/3dm-capability-slices.json'
            path.write_text(json.dumps(dict(schema_version=1,slices=[entry])))
            slices=module.property_slices(root,{'ON_Test'})
            self.assertEqual(slices['ON_Test'][0]['evidence'][0]['sha256'],module.sha(root/'proof.json'))
            self.assertFalse(slices['ON_Test'][0]['class_wide_completion'])
            for changed in (dict(entry,class_name='ON_Unknown'),dict(entry,evidence=['missing.json']),dict(entry,axes=dict(write='full'))):
                path.write_text(json.dumps(dict(schema_version=1,slices=[changed])))
                with self.assertRaises(ValueError):
                    module.property_slices(root,{'ON_Test'})


if __name__ == '__main__':
    unittest.main()
