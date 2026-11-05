// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_scene::spatial::SpatialIndex;

#[test]
fn near_parallel_ray_is_not_treated_as_exactly_parallel() {
    let index = SpatialIndex::build(&[[1., 0., 1.0e13]], &[0.1]);
    let candidates = index.ray_candidates([0., 0., 0.], [1.0e-13, 0., 1.]);
    assert_eq!(candidates, vec![0]);
}

#[test]
fn indexed_candidates_include_every_ray_hit_sphere() {
    let centres: Vec<_> = (0..240).map(|i| [((i % 16) as f64 - 8.) * 3., ((i / 16) as f64 - 7.) * 3., -(i as f64) * 0.01]).collect();
    let radii = vec![1.; centres.len()];
    let index = SpatialIndex::build(&centres, &radii);
    for centre in &centres {
        let origin = [centre[0] + 0.5, centre[1], 20.];
        let direction = [0., 0., -1.];
        let result = index.ray_candidates(origin, direction);
        for (i, other) in centres.iter().enumerate() {
            let dx = other[0] - origin[0];
            let dy = other[1] - origin[1];
            if dx * dx + dy * dy <= radii[i] * radii[i] && other[2] <= origin[2] {
                assert!(result.contains(&i), "ray {origin:?} missed sphere {i}");
            }
        }
    }
}
