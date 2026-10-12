import copy
import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('evidence', Path(__file__).with_name('witness_evidence.py'))
evidence = importlib.util.module_from_spec(spec)
spec.loader.exec_module(evidence)


class WitnessEvidenceTest(unittest.TestCase):
    def sample(self):
        return {'relation': 'Interior', 'point': [-5.65698859, 0.0, 8.5635492],
                'native_point': [-5.65698859, 0.0, 8.5635492], 'face': 0, 'uv': [5.7, .177]}

    def test_contained_point_disproves_box(self):
        row = evidence.verify_witness(self.sample(), [-5.6548028, -.3, 6.3, -4.34, .32, 8.5625054])
        self.assertTrue(row['independently_verified'])
        self.assertAlmostEqual(row['outside_bounds_mm'], .00218579)
        self.assertTrue(row['disproves_containment'])

    def test_noninterior_cannot_disprove_box(self):
        for relation in ['Exterior', 'Boundary', 'Unset', None]:
            sample = self.sample(); sample['relation'] = relation
            self.assertFalse(evidence.verify_witness(sample, [-5.65, -.3, 6.3, -4.34, .32, 8.56])['independently_verified'])

    def test_native_point_mismatch_cannot_certify(self):
        sample = self.sample(); sample['native_point'][0] += .002
        self.assertFalse(evidence.verify_witness(sample, [-5.65, -.3, 6.3, -4.34, .32, 8.56])['independently_verified'])

    def test_missing_and_nonfinite_rejected(self):
        for field in ['point', 'native_point', 'uv']:
            sample = self.sample(); del sample[field]
            with self.assertRaises(ValueError): evidence.verify_witness(sample, [-6, -1, 6, -4, 1, 9])
        sample = self.sample(); sample['point'][0] = float('nan')
        with self.assertRaises(ValueError): evidence.verify_witness(sample, [-6, -1, 6, -4, 1, 9])

    def test_interior_point_inside_box_is_not_failure(self):
        row = evidence.verify_witness(self.sample(), [-6, -1, 6, -4, 1, 9])
        self.assertTrue(row['independently_verified'])
        self.assertFalse(row['disproves_containment'])

    def test_invalid_bounds_rejected(self):
        with self.assertRaises(ValueError): evidence.verify_witness(self.sample(), [-4, -1, 6, -6, 1, 9])


if __name__ == '__main__': unittest.main()
