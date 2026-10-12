import copy
import unittest
from measured_bounds import compare


def geometry():
    points = [[-5.657, 0, 8], [-5.5, -.28, 8], [-4.34, 0, 6.3],
              [-4.34, 0, 8], [-5.5, .32, 8], [-5.5, 0, 8.564]]
    mesh = {'bounds': [-5.657, -.28, 6.3, -4.34, .32, 8.564], 'vertices': 100,
            'projection_points': points, 'projection_errors': [1e-7]*6,
            'cross_distances': [1e-6]*6}
    return {'raw_bounds': [-5.6548, -.28, 6.3, -4.34, .32, 8.5625],
            'unchanged': True, 'valid': True,
            'independent_witnesses': [{'point': [-5.65698859, 0., 8.5635492],
                'native_point': [-5.65698859, 0., 8.5635492], 'relation': 'Interior', 'uv': [5.7, .177]}],
            'levels': [dict(copy.deepcopy(mesh), tolerance=1e-5), dict(copy.deepcopy(mesh), tolerance=1e-6)]}


class MeasuredBoundsTest(unittest.TestCase):
    def test_measurement_recovers_invalid_raw_box(self):
        result = compare(geometry(), geometry())
        self.assertTrue(result['passed'])
        self.assertTrue(result['raw_containment_disproved'])

    def test_shift_of_two_microns_rejected(self):
        source = geometry(); target = geometry()
        for level in target['levels']:
            level['bounds'][0] += .002; level['bounds'][3] += .002
            for point in level['projection_points']: point[0] += .002
        for witness in target['independent_witnesses']:
            witness['point'][0] += .002; witness['native_point'][0] += .002
        result = compare(source, target)
        self.assertAlmostEqual(result['measured_deviation_mm'], .002)
        self.assertFalse(result['passed'])

    def test_accurate_raw_box_cannot_use_fallback(self):
        source = geometry(); target = geometry()
        for g in [source, target]: g['raw_bounds'] = g['levels'][1]['bounds'][:]
        self.assertFalse(compare(source, target)['passed'])

    def test_converged_mesh_missing_certified_point_rejected(self):
        source = geometry(); target = geometry()
        witness = source['independent_witnesses'][0]
        witness['point'][0] -= .002; witness['native_point'][0] -= .002
        self.assertFalse(compare(source, target)['passed'])

    def test_missing_or_exterior_independent_witness_rejected(self):
        for change in ['missing', 'exterior']:
            source = geometry(); target = geometry()
            if change == 'missing': source['independent_witnesses'] = []
            else: source['independent_witnesses'][0]['relation'] = 'Exterior'
            self.assertFalse(compare(source, target)['passed'])

    def test_nonconverged_missing_invalid_and_mutated_rejected(self):
        for change in ['convergence', 'missing', 'invalid', 'mutated', 'nonfinite', 'cross', 'projection']:
            source = geometry(); target = geometry()
            if change == 'convergence': target['levels'][0]['bounds'][0] += .0005
            if change == 'missing': target['levels'] = target['levels'][:1]
            if change == 'invalid': target['valid'] = False
            if change == 'mutated': target['unchanged'] = False
            if change == 'nonfinite': target['levels'][1]['bounds'][0] = float('nan')
            if change == 'cross': target['levels'][1]['cross_distances'][0] = .002
            if change == 'projection': target['levels'][1]['projection_errors'][0] = .002
            with self.subTest(change=change): self.assertFalse(compare(source, target)['passed'])


if __name__ == '__main__': unittest.main()
