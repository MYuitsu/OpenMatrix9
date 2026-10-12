"""Bind actual GUI lifecycle to separate read-only precise measurements."""
import json, shutil, tempfile, unittest
from pathlib import Path
import sys
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from curved_exchange_evidence import compose
ROOT=Path(__file__).resolve().parents[2]/'docs/validation/rhino5-curved-shells-20261007'

class CurvedEvidenceTests(unittest.TestCase):
    def test_actual_reports_bind_without_rewriting_failed_raw_comparison(self):
        row=compose(ROOT)
        self.assertTrue(row['scoped_exchange_verified'])
        self.assertFalse(row['raw_gui_ok'])
        self.assertEqual(row['precise_measurements'],8)
        self.assertFalse(row['full_exchange_complete'])
    def test_modified_saved_bytes_are_rejected(self):
        with tempfile.TemporaryDirectory() as folder:
            root=Path(folder)/'proof';shutil.copytree(ROOT,root)
            path=next(root.glob('gui-resaved-*.3dm'))
            path.write_bytes(path.read_bytes()+b'changed')
            with self.assertRaises(ValueError):compose(root)
    def test_changed_numeric_measurement_is_rejected_even_when_passed_flag_true(self):
        with tempfile.TemporaryDirectory() as folder:
            root=Path(folder)/'proof';shutil.copytree(ROOT,root)
            path=root/'rhino5-curved-precision-results.json'
            report=json.loads(path.read_text())
            report['cases'][0]['saved']['quadrature'][-1]['volume_mm3']+=1e-5
            path.write_text(json.dumps(report))
            with self.assertRaises(ValueError):compose(root)

if __name__=='__main__':unittest.main()
