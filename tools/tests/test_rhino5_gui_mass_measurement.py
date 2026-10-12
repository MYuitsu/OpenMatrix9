"""Adapter regression using captured real Rhino5 numeric measurements.

Native calls are replayed; this does not replace actual Rhino GUI execution.
"""
import ast,json,types,unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]

class Brep:
    def __init__(self,row):
        self.row=row;self.IsValid=True;self.IsSolid=True
        self.SolidOrientation='Outward';self.Faces=types.SimpleNamespace(Count=row['faces'])
        self.calls=[];self.delta=0
    def GetType(self):return types.SimpleNamespace(FullName='Rhino.Geometry.Brep')
    def GetVolume(self,relative,absolute):
        self.calls.append((relative,absolute))
        return self.row['quadrature'][-1]['volume_mm3']+self.delta

class Mass:
    def __init__(self,brep):
        self.Volume=brep.row['default_volume_mm3']
        self.VolumeError=brep.row['default_volume_error_mm3']
        self.disposed=False
    def Dispose(self):self.disposed=True

def measurements(row):
    brep=Brep(row);mass=Mass(brep)
    obj=types.SimpleNamespace(IsDeleted=False,Id=row['uuid'],Geometry=brep)
    rhino=types.SimpleNamespace(RhinoDoc=types.SimpleNamespace(ActiveDoc=types.SimpleNamespace(Objects=[obj])),
        Geometry=types.SimpleNamespace(Curve=type('Curve',(),{}),Brep=Brep,
        VolumeMassProperties=types.SimpleNamespace(Compute=lambda geometry:mass)))
    tree=ast.parse((ROOT/'tests/rhino5_gui_profiles_probe.py').read_text())
    function=next(node for node in tree.body if isinstance(node,ast.FunctionDef) and node.name=='document_measurements')
    scope={'Rhino':rhino};exec(compile(ast.Module(body=[function],type_ignores=[]),'rhino5_gui_profiles_probe.py','exec'),scope)
    return scope['document_measurements'],brep,mass

class MassMeasurementTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.proof=json.loads((ROOT/'docs/validation/rhino5-curved-shells-20261007/rhino5-curved-precision-results.json').read_text())
        assert cls.proof['ok'] and len(cls.proof['cases'])==4
    def test_precise_measurement_accepts_all_four_real_cases_and_retains_raw(self):
        for case in self.proof['cases']:
            row=case['original']
            with self.subTest(fixture=case['fixture']):
                measure,brep,mass=measurements(row)
                expected={'objects':[dict(uuid=row['uuid'],faces=row['faces'],volume_mm3=row['analytical_volume_mm3'])]}
                actual=measure(expected)
                self.assertTrue(actual['passed'])
                self.assertEqual(actual['objects'][0]['volume_mm3'],row['quadrature'][-1]['volume_mm3'])
                self.assertEqual(actual['objects'][0]['default_volume_mm3'],row['default_volume_mm3'])
                self.assertEqual(actual['objects'][0]['default_volume_error_mm3'],row['default_volume_error_mm3'])
                self.assertEqual(brep.calls,[(1e-13,1e-13)])
                self.assertTrue(mass.disposed)
    def test_bad_precise_volume_still_fails_original_threshold(self):
        row=self.proof['cases'][0]['original']
        measure,brep,mass=measurements(row);brep.delta=1e-5
        expected={'objects':[dict(uuid=row['uuid'],faces=row['faces'],volume_mm3=row['analytical_volume_mm3'])]}
        self.assertFalse(measure(expected)['passed'])
        self.assertEqual(brep.calls,[(1e-13,1e-13)])
        self.assertTrue(mass.disposed)

if __name__=='__main__':unittest.main()
