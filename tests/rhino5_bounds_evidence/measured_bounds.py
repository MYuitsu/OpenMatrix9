"""Bounded numerical measurement fallback, not a certified global extrema solver."""
import math
from witness_evidence import verify_witness

THRESHOLD_MM = .001


def vector(value, size):
    if not isinstance(value, list) or len(value) != size or not all(
            isinstance(x, (int, float)) and math.isfinite(x) for x in value):
        raise ValueError('Missing/non-finite measurement')
    return value


def bbox(value):
    result = vector(value, 6)
    if any(result[i] > result[i+3] for i in range(3)): raise ValueError('Inverted bounds')
    return result


def outside(point, bounds):
    return max([0.] + [bounds[i]-point[i] for i in range(3)] +
               [point[i]-bounds[i+3] for i in range(3)])


def measurement(geometry):
    if not geometry.get('valid') or not geometry.get('unchanged'): raise ValueError('Invalid/mutated geometry')
    raw = bbox(geometry['raw_bounds'])
    levels = geometry['levels']
    if len(levels) != 2 or [l['tolerance'] for l in levels] != [1e-5, 1e-6]:
        raise ValueError('Missing prescribed convergence levels')
    for level in levels:
        bbox(level['bounds'])
        if level['vertices'] <= 0: raise ValueError('Empty mesh')
        if len(level['projection_points']) != 6: raise ValueError('Missing projection witnesses')
        for point in level['projection_points']: vector(point, 3)
        for field in ['projection_errors', 'cross_distances']:
            if min(vector(level[field], 6)) < 0: raise ValueError('Negative distance')
    coarse, fine = levels
    convergence = max(abs(a-b) for a, b in zip(coarse['bounds'], fine['bounds']))
    if convergence > THRESHOLD_MM/100: raise ValueError('Mesh bounds have not converged')
    float_resolution = max(abs(x) for x in fine['bounds'])*2**-23
    budget = convergence + fine['tolerance'] + float_resolution + max(fine['projection_errors'])
    if budget > THRESHOLD_MM/10: raise ValueError('Measurement uncertainty too large')
    witnesses = geometry.get('independent_witnesses', [])
    if not witnesses: raise ValueError('Missing independent contained points')
    verified = [verify_witness(w, raw) for w in witnesses]
    if not all(w['independently_verified'] for w in verified): raise ValueError('Unverified independent point')
    known_point_omission = max(outside(w['point'], fine['bounds']) for w in verified)
    if known_point_omission > budget: raise ValueError('Mesh misses independently verified point')
    return dict(bounds=fine['bounds'], convergence_mm=convergence,
                numerical_budget_mm=budget, cross_distance_mm=max(fine['cross_distances']),
                known_point_omission_mm=known_point_omission,
                raw_containment_disproved=any(outside(p, raw) > THRESHOLD_MM
                                             for p in fine['projection_points']))


def compare(source, exported):
    try:
        a, b = measurement(source), measurement(exported)
        deviation = max(abs(x-y) for x, y in zip(a['bounds'], b['bounds']))
        budget = a['numerical_budget_mm'] + b['numerical_budget_mm']
        disproved = a['raw_containment_disproved'] or b['raw_containment_disproved']
        passed = disproved and deviation+budget < THRESHOLD_MM and max(
            a['cross_distance_mm'], b['cross_distance_mm'])+budget < THRESHOLD_MM
        return dict(passed=passed, raw_containment_disproved=disproved,
                    measured_deviation_mm=deviation, combined_numerical_budget_mm=budget,
                    source=a, exported=b, threshold_mm=THRESHOLD_MM,
                    scope='Converged diagnostic meshing and trimmed-BRep projection corroborated by independent contained points; empirical numerical verification. Budget is an observed screening allowance, not a certified extrema error bound.')
    except (KeyError, ValueError, TypeError) as exc:
        return dict(passed=False, error=str(exc), threshold_mm=THRESHOLD_MM)
