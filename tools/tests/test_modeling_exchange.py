"""Detached preflight must reject unsupported geometry before host mutation."""
import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('modeling', Path(__file__).parents[2] / 'ThreeDmModeling.py')
modeling = importlib.util.module_from_spec(spec)
spec.loader.exec_module(modeling)

class ModelingPreflightTests(unittest.TestCase):
    def test_unknown_geometry_is_not_dropped(self):
        with self.assertRaisesRegex(RuntimeError, 'Mystery'):
            modeling.validate_prepared([{'geometry_kind':2, 'brep':'curve.brep'},
                                        {'geometry_kind':9, 'name':'Mystery', 'class_name':'PluginGeometry'}])

    def test_cad_with_display_mesh_stays_cad(self):
        modeling.validate_prepared([{'geometry_kind':3, 'brep':'solid.brep', 'display_mesh':True}])

    def test_ambiguous_geometry_cannot_bypass_preflight(self):
        with self.assertRaises(RuntimeError):
            modeling.validate_prepared([{'geometry_kind':3, 'brep':'solid.brep','vertices':[],'faces':[]}])

    def test_empty_and_retained_selection_refused(self):
        for rows in [[], [{'geometry_kind':8,'name':'Opaque'}], [{'geometry_kind':6,'name':'SubD'}]]:
            with self.subTest(rows=rows), self.assertRaises(RuntimeError):
                modeling.validate_prepared(rows)

if __name__ == '__main__': unittest.main()
