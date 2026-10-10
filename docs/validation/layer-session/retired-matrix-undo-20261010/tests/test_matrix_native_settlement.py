"""Test the native-ready gate in the actual generator, without native Rhino edits."""
from pathlib import Path
import types,unittest
source=Path('H:/FreeCAD-src/build/om9-layer-edits/tests/rhino5_verify_matrix_om9_handoff.py').read_text()
class NativeSettlement(unittest.TestCase):
    def prepare(self,ready):
        self.now=0.0
        ns=dict(time=types.SimpleNamespace(time=lambda:self.now),native_settled=ready)
        start=source.find('def wait_native_settled(')
        if start<0:
            # RED: the currently installed harness simply yields once.
            exec('def wait_native_settled():\n    yield None\n',ns)
        else:exec(source[start:source.index('\ndef ',start+4)],ns)
        return ns['wait_native_settled']()
    def test_observation_waits_for_deferred_native_handler(self):
        pending=[True];gate=self.prepare(lambda:not pending[0]);next(gate)
        self.now=0.1
        try:next(gate)
        except StopIteration:self.fail('Harness finished before deferred native handler')
        pending[0]=False
        with self.assertRaises(StopIteration):next(gate)
    def test_native_error_stops_before_comparison(self):
        def ready():raise RuntimeError('Native projection failed')
        gate=self.prepare(ready);next(gate)
        try:
            with self.assertRaisesRegex(RuntimeError,'Native projection'):next(gate)
        except StopIteration:self.fail('Harness ignored native projection error')
    def test_pending_native_projection_times_out(self):
        gate=self.prepare(lambda:False);next(gate);self.now=11
        try:
            with self.assertRaisesRegex(RuntimeError,'settle'):next(gate)
        except StopIteration:self.fail('Harness compared a still-pending projection')
if __name__=='__main__':unittest.main()
