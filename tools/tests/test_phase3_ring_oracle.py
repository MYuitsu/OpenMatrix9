import unittest,sys
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[2]/'tests'))
from phase3_ring_expectations import expected_ring, matches

class RingOracle(unittest.TestCase):
    def test_complement_with_same_volume_and_area_is_rejected(self):
        half,_=expected_ring()
        wrong=dict(half,bounds=[-12,0,-2,12,12,2])
        self.assertFalse(matches(wrong,half))
        self.assertTrue(matches(dict(half),half))
    def test_invalid_topology_or_missing_dimension_is_rejected(self):
        half,_=expected_ring()
        self.assertFalse(matches(dict(half,solids=0),half))
        self.assertFalse(matches(dict(half,bounds=half['bounds'][:3]),half))
        self.assertFalse(matches(dict(half,area=half['area']+2),half))
