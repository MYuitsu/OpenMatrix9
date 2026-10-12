"""Build-only provenance cannot certify runtime tests or another checkout."""
import importlib.util
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

ROOT=Path(__file__).resolve().parents[2]
spec=importlib.util.spec_from_file_location('record_build',ROOT/'tools/ci/record-build.py')
module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)


class BuildRecordTests(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory();self.addCleanup(self.temp.cleanup)
        self.root=Path(self.temp.name);self.source=self.root/'source';self.source.mkdir()
        self.plugin=self.root/'plugin';(self.plugin/'bin').mkdir(parents=True)
        (self.source/'release.json').write_text(json.dumps({'version':'0.3.4-202610120606','freecad':'1.1.4'}))
        (self.root/'build-paths.json').write_text(json.dumps({'plugin':str(self.plugin)}))
        for name in ('OpenMatrix9Gui.pyd','OM9ThreeDmImportWorker.exe'):
            (self.plugin/'bin'/name).write_bytes(('fixture '+name).encode())
        self.env={'GITHUB_SHA':'a'*40,'GITHUB_REPOSITORY':'MYuitsu/OpenMatrix9','GITHUB_RUN_ID':'123'}

    def record(self):
        with patch.object(module.subprocess,'check_output',return_value='a'*40+'\n'):
            return module.record_build(self.root,self.source,self.env)

    def test_manifest_records_exact_source_hashes_and_no_runtime_verdict(self):
        record=self.record()
        self.assertEqual(record['source_commit'],'a'*40)
        self.assertEqual(record['build_url'],'https://github.com/MYuitsu/OpenMatrix9/actions/runs/123')
        self.assertEqual(record['plugin_sha256'],module.sha(self.plugin/'bin/OpenMatrix9Gui.pyd'))
        self.assertEqual(record['release_metadata_sha256'],module.sha(self.source/'release.json'))
        self.assertIs(record['tests_executed'],False)
        self.assertEqual(record['validation_kind'],'build_only')
        self.assertFalse({'gumball','rust_tests','brep_fixtures','native_reports'} & record.keys())
        self.assertEqual(json.loads((self.root/'evidence/summary.json').read_text()),record)

    def test_changed_checkout_is_rejected_before_writing_provenance(self):
        self.env['GITHUB_SHA']='b'*40
        with self.assertRaisesRegex(RuntimeError,'source commit'):
            self.record()
        self.assertFalse((self.plugin/'release-validation.json').exists())

    def test_missing_run_identity_is_rejected(self):
        self.env['GITHUB_RUN_ID']=''
        with self.assertRaisesRegex(RuntimeError,'run identity'):
            self.record()

    def test_empty_worker_is_rejected(self):
        (self.plugin/'bin/OM9ThreeDmImportWorker.exe').write_bytes(b'')
        with self.assertRaisesRegex(RuntimeError,'compiled binary'):
            self.record()


if __name__=='__main__':unittest.main()
