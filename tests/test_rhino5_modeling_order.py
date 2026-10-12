"""Exercise the probe's real assertions without starting a Rhino document."""
import os
import math
import copy
import unittest
import ntpath
import json

SCRIPT = os.path.join(os.path.dirname(os.path.abspath(__file__)), 'rhino5_modeling_exchange.py')
with open(SCRIPT, 'r') as stream:
    SOURCE = stream.read()


def check(name, value):
    if not value:
        raise AssertionError(name)


def namespace(rows, received=None):
    scope = {'math': math, 'ntpath': ntpath, 'check': check, 'rows': rows,
             'received': received, 'all_rows': rows, 'saved_all': received,
             'entry': {'name': 'edited-and-new'}}
    helpers = SOURCE[SOURCE.index('def digest(path):'):SOURCE.index('\ntry:')]
    exec(compile(helpers, SCRIPT, 'exec'), scope, scope)
    return scope


def execute_block(start, end, scope):
    block = SOURCE[SOURCE.index(start):SOURCE.index(end)]
    block = '\n'.join(line[8:] if line.startswith('        ') else line
                      for line in block.splitlines())
    exec(compile(block, SCRIPT, 'exec'), scope, scope)


def curve_assertions(rows):
    execute_block("        if entry['name']=='edited-and-new':",
                  "        case['save_command']=", namespace(rows))


def saved_assertions(rows, received):
    execute_block('        for a,b in ', "        if config.get('export_selected'):",
                  namespace(rows, received))


class ProbeObjectOrderTests(unittest.TestCase):
    def setUp(self):
        self.line = {'object_id': 'line-id', 'class': 'Rhino.Geometry.LineCurve',
                     'bounds': [20, 0, 0, 30, 0, 0], 'length': 10.0}
        self.circle = {'object_id': 'circle-id', 'class': 'Rhino.Geometry.ArcCurve',
                       'bounds': [-3, 4, 0, 3, 10, 0], 'length': 6 * math.pi}

    def test_document_object_order_does_not_change_curve_assertions(self):
        curve_assertions([self.line, self.circle])
        curve_assertions([self.circle, self.line])

    def test_wrong_curve_placement_still_fails(self):
        shifted = copy.deepcopy(self.line)
        shifted['bounds'] = [0, 0, 0, 10, 0, 0]
        with self.assertRaises(AssertionError):
            curve_assertions([self.circle, shifted])

    def test_wrong_circle_length_still_fails(self):
        bad = copy.deepcopy(self.circle)
        bad['length'] = 4 * math.pi
        with self.assertRaises(AssertionError):
            curve_assertions([bad, self.line])

    def test_save_reread_matches_identity_despite_order_change(self):
        saved_assertions([self.circle, self.line], [self.line, self.circle])

    def test_saved_geometry_change_still_fails(self):
        bad = copy.deepcopy(self.line)
        bad['bounds'][3] = 31
        with self.assertRaises(AssertionError):
            saved_assertions([self.line, self.circle], [bad, self.circle])

    def test_saved_duplicate_identity_still_fails(self):
        with self.assertRaises(AssertionError):
            saved_assertions([self.line, self.circle], [self.line, self.line])

    def test_saved_missing_identity_still_fails(self):
        with self.assertRaises(AssertionError):
            saved_assertions([self.line, self.circle], [self.line])

    def file_command(self, action, path):
        scope = namespace([])
        self.assertTrue(callable(scope.get('rhino_file_command')), 'Rhino5 command paths must use Windows separators')
        return scope['rhino_file_command'](action, path)

    def test_open_accepts_all_forward_slash_fixture_directory(self):
        path = 'H:/FreeCAD-src/build/3dm-preserve-native/modeling-fixtures/mm-current.3dm'
        self.assertEqual(self.file_command('Open', path),
                         '_-Open "H:\\FreeCAD-src\\build\\3dm-preserve-native\\modeling-fixtures\\mm-current.3dm" _Enter')

    def test_saveas_accepts_mixed_separators(self):
        self.assertEqual(self.file_command('SaveAs', 'H:/FreeCAD-src/build/modeling-rhino5/run\\mm.rhino5.3dm'),
                         '_-SaveAs "H:\\FreeCAD-src\\build\\modeling-rhino5\\run\\mm.rhino5.3dm" _Enter')

    def test_unicode_and_spaces_are_preserved_in_command_path(self):
        self.assertEqual(self.file_command('Open', u'H:/CAD model/nh\u1eabn.3dm'),
                         u'_-Open "H:\\CAD model\\nh\u1eabn.3dm" _Enter')

    def test_definition_member_is_not_counted_as_model_root(self):
        scope = namespace([])
        self.assertTrue(callable(scope.get('root_rows')))
        member = {'object_id': 'member-id', 'definition_member': True}
        self.assertEqual(scope['root_rows']([self.line, member, self.circle]),
                         [self.line, self.circle])

    def test_saved_brep_area_change_still_fails(self):
        before = dict(self.line, area=20.0, faces=1)
        after = dict(before, area=21.0)
        with self.assertRaises(AssertionError):
            saved_assertions([before], [after])

    def test_saved_solid_volume_change_still_fails(self):
        before = dict(self.line, volume=20.0, solid=True)
        after = dict(before, volume=21.0)
        with self.assertRaises(AssertionError):
            saved_assertions([before], [after])

    def test_ring_selected_export_uses_new_circle_subset(self):
        scope = namespace([])
        self.assertTrue(callable(scope.get('export_rows')))
        new_circle = dict(self.circle, name='NewRingCircle', length=4*math.pi,
                          bounds=[15,2,1,19,6,1])
        self.assertEqual(scope['export_rows']('current-ring', [self.line, new_circle]), [new_circle])

    def instance_fixture(self):
        # Recorded actual Rhino5 failure: identical member BRep and reflection,
        # differing instance cache. Reflection maps the member's exact bounds.
        proof = os.path.join(os.path.dirname(SCRIPT), '..', 'docs', 'validation',
                             'modeling-phase-1', 'rhino5-ring-instance-bounds-failure.json')
        with open(proof) as stream:
            case = json.load(stream)['cases'][0]
        groups = []
        for field in ['all_rows', 'saved_all_rows']:
            rows = copy.deepcopy(case[field])
            member = next(row for row in rows if row.get('definition_member'))
            instance = next(row for row in rows if row.get('transform'))
            self.assertEqual(instance['transform'], [-1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.])
            box = member['bounds']
            instance['instance_geometry_bounds'] = [-box[3],box[1],box[2],-box[0],box[4],box[5]]
            instance['instance_definition_objects'] = [member['object_id']]
            groups.append(rows)
        return groups

    def test_actual_ring_instance_cache_change_is_not_geometry_change(self):
        before, after = self.instance_fixture()
        saved_assertions(before, after)

    def test_actual_ring_instance_resolved_bounds_change_still_fails(self):
        before, after = self.instance_fixture()
        next(row for row in after if row.get('transform'))['instance_geometry_bounds'][0] += .01
        with self.assertRaises(AssertionError) as failure:
            saved_assertions(before, after)
        self.assertIn('Rhino saved bounds', str(failure.exception))

    def test_actual_ring_instance_definition_membership_change_still_fails(self):
        before, after = self.instance_fixture()
        next(row for row in after if row.get('transform'))['instance_definition_objects'] = ['wrong-member']
        with self.assertRaises(AssertionError) as failure:
            saved_assertions(before, after)
        self.assertIn('Rhino saved instance_definition_objects', str(failure.exception))

    def test_instance_without_resolved_geometry_bounds_fails(self):
        before, after = self.instance_fixture()
        for rows in [before, after]:
            instance = next(row for row in rows if row.get('transform'))
            instance['bounds'] = [0,0,0,1,1,1]
            del instance['instance_geometry_bounds']
        with self.assertRaises(AssertionError) as failure:
            saved_assertions(before, after)
        self.assertIn('Rhino instance resolved geometry bounds available', str(failure.exception))

    def test_actual_ring_instance_transform_change_still_fails(self):
        before, after = self.instance_fixture()
        next(row for row in after if row.get('transform'))['transform'][0] = 1.
        with self.assertRaises(AssertionError) as failure:
            saved_assertions(before, after)
        self.assertIn('Rhino saved instance transform', str(failure.exception))


if __name__ == '__main__':
    unittest.main()
