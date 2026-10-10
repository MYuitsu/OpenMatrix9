"""Numerical probe regression. Native calls are simulated, not Rhino evidence."""
import ast,contextlib,io,json,math,traceback,types,unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
SCRIPT=ROOT/'tests/rhino5_gui_profiles_probe.py'

class Curve:
    def __init__(self,coordinate=0.0,point_valid=True):
        self.Domain=types.SimpleNamespace(T0=31,T1=47)
        self.IsValid=True;self.coordinate=coordinate;self.point_valid=point_valid
    def GetType(self):return types.SimpleNamespace(FullName='Rhino.Geometry.PolyCurve')
    def PointAt(self,t):
        return types.SimpleNamespace(X=self.coordinate,Y=0.0,Z=0.0,IsValid=self.point_valid)

def measurement(curve):
    obj=types.SimpleNamespace(Id='root',IsDeleted=False,Geometry=curve)
    rhino=types.SimpleNamespace(RhinoDoc=types.SimpleNamespace(ActiveDoc=types.SimpleNamespace(Objects=[obj])),
        Geometry=types.SimpleNamespace(Curve=Curve,Brep=type('Brep',(),{})))
    nodes=[n for n in ast.parse(SCRIPT.read_text(encoding='utf-8')).body if isinstance(n,ast.FunctionDef)]
    scope={'Rhino':rhino,'math':math,'traceback':traceback}
    exec(compile(ast.Module(body=nodes,type_ignores=[]),str(SCRIPT),'exec'),scope)
    expected={'objects':[{'uuid':'root','domain':[31,47],'samples':[[0.0,0.0,0.0]]*17}]}
    return scope['document_measurements'](expected)

class CurveMeasurementTests(unittest.TestCase):
    def measure_without_crashing(self,curve):
        try:return measurement(curve)
        except (OverflowError,ValueError) as error:self.fail('Probe must record failure instead of stopping: '+str(error))
    def test_unset_point_is_retained_and_fails_without_overflow(self):
        value=-1.23432101234321e308
        row=self.measure_without_crashing(Curve(value,False))
        self.assertFalse(row['passed'])
        self.assertEqual(row['objects'][0]['samples'][0][0],value)
        self.assertFalse(row['objects'][0]['sample_validity'][0])
    def test_huge_finite_point_does_not_overflow_or_pass(self):
        row=self.measure_without_crashing(Curve(1e200))
        self.assertFalse(row['passed'])
        self.assertEqual(row['comparisons'][0]['maximum_sample_deviation_mm'],1e200)
    def test_nonfinite_sample_fails_and_report_is_strict_json(self):
        for value in [float('inf'),float('-inf'),float('nan')]:
            with self.subTest(value=value):
                row=self.measure_without_crashing(Curve(value,False));self.assertFalse(row['passed'])
                try:json.dumps(row,allow_nan=False)
                except ValueError as error:self.fail('Nonfinite coordinate needs explicit diagnostic: '+str(error))
    def test_invalid_point_cannot_pass_even_with_correct_coordinates(self):
        self.assertFalse(measurement(Curve(0.0,False))['passed'])
    def test_original_sample_threshold_stays_strict(self):
        self.assertTrue(measurement(Curve(0.5e-9))['passed'])
        self.assertFalse(measurement(Curve(1.1e-9))['passed'])
    def test_diagonal_norm_is_not_replaced_by_coordinate_maximum(self):
        curve=Curve();curve.PointAt=lambda t:types.SimpleNamespace(X=0.7e-9,Y=0.7e-9,Z=0.7e-9,IsValid=True)
        self.assertFalse(measurement(curve)['passed'])
    def test_case_exception_is_recorded_and_later_cases_still_run(self):
        nodes=[n for n in ast.parse(SCRIPT.read_text(encoding='utf-8')).body if isinstance(n,ast.FunctionDef)]
        report={'cases':[],'ok':False};scope={'report':report,'traceback':traceback}
        exec(compile(ast.Module(body=nodes,type_ignores=[]),str(SCRIPT),'exec'),scope)
        self.assertIn('run_cases',scope,'Probe needs per-case failure records instead of whole-matrix abort')
        calls=[];dumps=[]
        def run_case(case):
            calls.append(case['fixture'])
            if case['fixture']=='bad':raise RuntimeError('Rhino transport failure')
            return dict(fixture=case['fixture'],passed=True)
        scope.update(run_case=run_case,dump=lambda:dumps.append(len(report['cases'])))
        with contextlib.redirect_stdout(io.StringIO()):
            scope['run_cases']([{'fixture':'bad'},{'fixture':'good'}])
        self.assertEqual(calls,['bad','good']);self.assertEqual(dumps,[1,2])
        self.assertFalse(report['cases'][0]['passed']);self.assertIn('Rhino transport failure',report['cases'][0]['error'])
        self.assertTrue(report['cases'][1]['passed']);self.assertFalse(report['ok'])

if __name__=='__main__':unittest.main()
