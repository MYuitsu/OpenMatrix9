"""Exercise request size/geometry values and atomic protocol without a CAD host."""
import importlib.util
import json
from pathlib import Path
import sys
import tempfile
import types
import unittest
from unittest.mock import patch


class Writer:
    def __init__(self, fail=False):
        self.request = None
        self.fail = fail

    def write3dm(self, request, path):
        self.request = request
        Path(path).write_bytes(b'owned staged archive')
        if self.fail:
            raise RuntimeError('native writer refused')


class ModelingWriterPayloadTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        # Only module import needs host names; this detached function takes its writer.
        spec = importlib.util.spec_from_file_location('detached_three_dm', Path(__file__).parents[2] / 'ThreeDm.py')
        cls.module = importlib.util.module_from_spec(spec)
        with patch.dict(sys.modules, {'FreeCAD': types.ModuleType('FreeCAD'), 'FreeCADGui': types.ModuleType('FreeCADGui')}):
            spec.loader.exec_module(cls.module)

    def prepared(self):
        return {'items': [{'name': 'Đá oval / 中文\nline', 'vertices': [[i + 0.125, i - 0.5, 0.0] for i in range(256)],
                           'faces': [[i, i+1, i+2, i+2] for i in range(254)]}], 'tolerance': 1e-9}

    def test_large_request_smaller_without_changing_geometry_or_unicode(self):
        prepared = self.prepared()
        writer = Writer()
        with tempfile.TemporaryDirectory() as directory:
            target = Path(directory) / 'current.3dm'
            self.module._write_geometry_atomic(writer, prepared, target)
            self.assertEqual(json.loads(writer.request), prepared)
            self.assertLess(len(writer.request.encode()), len(json.dumps(prepared).encode()) * 0.9)
            self.assertEqual(target.read_bytes(), b'owned staged archive')
            self.assertEqual(list(Path(directory).glob('.om9-*')), [])

    def test_failed_writer_preserves_destination_and_cleans_staging(self):
        with tempfile.TemporaryDirectory() as directory:
            target = Path(directory) / 'current.3dm'
            target.write_bytes(b'previous archive')
            with self.assertRaisesRegex(RuntimeError, 'native writer refused'):
                self.module._write_geometry_atomic(Writer(fail=True), self.prepared(), target)
            self.assertEqual(target.read_bytes(), b'previous archive')
            self.assertEqual(list(Path(directory).glob('.om9-*')), [])


if __name__ == '__main__':
    unittest.main()
