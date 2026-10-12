"""Recovery ownership checks against this actual failed-run fixture, without Rhino edits."""
import copy,json,types,unittest
from pathlib import Path
root=Path('H:/FreeCAD-src/build/om9-layer-edits')
failed=Path('H:/FreeCAD-src/build/matrix-om9-handoff/dc77ce9d849842ce86b027e9a19d2198')
class RecoveryOwnership(unittest.TestCase):
    def setUp(self):
        self.fixture=json.loads((failed/'matrix-before-paste.json').read_text())
        self.current=copy.deepcopy(self.fixture)
        for row in self.current['raw']['layers']:
            row['locked']=row.get('native_locked',row['locked']);row['visible']=row.get('native_visible',row['visible'])
        self.doc=types.SimpleNamespace(DocumentId=self.fixture['document_id'],Path='')
        self.ns=dict(Rhino=types.SimpleNamespace(RhinoDoc=types.SimpleNamespace(ActiveDoc=self.doc)),
          workflow=types.SimpleNamespace(snapshot=lambda:self.current),before=json.loads((failed/'matrix-original.json').read_text()),
          fixture=self.fixture,failure=json.loads((failed/'handoff-results.json').read_text()))
        source=(root/'tests/rhino5_recover_matrix_palette.py').read_text()
        exec(source[source.index('def validate():'):source.index('def finish(')],self.ns)
    def test_actual_fixture_accepted_and_only_five_owned_layers(self):
        doc,current,owned=self.ns['validate']();self.assertIs(doc,self.doc);self.assertEqual(len(owned),5)
    def test_new_object_refused(self):
        self.current['raw']['objects'].append(dict(self.current['raw']['objects'][0],id='foreign'))
        with self.assertRaisesRegex(RuntimeError,'Fixture geometry'):self.ns['validate']()
    def test_new_layer_refused(self):
        self.current['raw']['layers'].append(dict(self.current['raw']['layers'][0],id='foreign'))
        with self.assertRaisesRegex(RuntimeError,'Foreign/missing layer'):self.ns['validate']()
    def test_changed_native_child_state_refused(self):
        row=next(r for r in self.current['raw']['layers'] if len(r['path'])==2)
        row['persistent_locked']=not row['persistent_locked']
        with self.assertRaisesRegex(RuntimeError,'Layer native state changed'):self.ns['validate']()
    def test_saved_document_refused(self):
        self.doc.Path='user-model.3dm'
        with self.assertRaisesRegex(RuntimeError,'saved'):self.ns['validate']()
if __name__=='__main__':unittest.main()
