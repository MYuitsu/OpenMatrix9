// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    geometry::Mesh,
    mesh_csg::{self, BooleanOp, CsgOptions, TriangleIntersection},
};
use std::collections::BTreeMap;

fn cube(lo: [f64; 3], hi: [f64; 3]) -> Mesh {
    Mesh {
        vertices: vec![
            [lo[0], lo[1], lo[2]],
            [hi[0], lo[1], lo[2]],
            [hi[0], hi[1], lo[2]],
            [lo[0], hi[1], lo[2]],
            [lo[0], lo[1], hi[2]],
            [hi[0], lo[1], hi[2]],
            [hi[0], hi[1], hi[2]],
            [lo[0], hi[1], hi[2]],
        ],
        faces: vec![
            vec![0, 2, 1],
            vec![0, 3, 2],
            vec![4, 5, 6],
            vec![4, 6, 7],
            vec![0, 1, 5],
            vec![0, 5, 4],
            vec![1, 2, 6],
            vec![1, 6, 5],
            vec![2, 3, 7],
            vec![2, 7, 6],
            vec![3, 0, 4],
            vec![3, 4, 7],
        ],
        normals: vec![],
        colors: vec![],
        corner_uvs: vec![],
    }
}
fn join(a: &Mesh, b: &Mesh) -> Mesh {
    let mut out = a.clone();
    let offset = out.vertices.len() as u32;
    out.vertices.extend(&b.vertices);
    out.faces.extend(
        b.faces
            .iter()
            .map(|f| f.iter().map(|v| v + offset).collect::<Vec<_>>()),
    );
    out.corner_uvs.extend(&b.corner_uvs);
    out
}
fn volume(m: &Mesh) -> f64 {
    // Translate first: independent signed tetrahedron sum stable far from origin.
    let origin = m.vertices.first().copied().unwrap_or([0.0; 3]);
    m.faces
        .iter()
        .map(|f| {
            let p: Vec<_> = f
                .iter()
                .map(|&i| {
                    let v = m.vertices[i as usize];
                    [v[0] - origin[0], v[1] - origin[1], v[2] - origin[2]]
                })
                .collect();
            let (a, b, c) = (p[0], p[1], p[2]);
            (a[0] * (b[1] * c[2] - b[2] * c[1])
                + a[1] * (b[2] * c[0] - b[0] * c[2])
                + a[2] * (b[0] * c[1] - b[1] * c[0]))
                / 6.0
        })
        .sum()
}
fn closed(m: &Mesh) {
    let mut edges = BTreeMap::<(u32, u32), (usize, i32)>::new();
    for f in &m.faces {
        assert_eq!(f.len(), 3);
        for i in 0..3 {
            let (a, b) = (f[i], f[(i + 1) % 3]);
            let e = edges.entry((a.min(b), a.max(b))).or_default();
            e.0 += 1;
            e.1 += if a < b { 1 } else { -1 };
        }
    }
    assert!(
        edges
            .values()
            .all(|&(count, balance)| count == 2 && balance == 0),
        "boundary/nonmanifold edges: {:?}",
        edges
            .iter()
            .filter(|(_, e)| e.0 != 2 || e.1 != 0)
            .collect::<Vec<_>>()
    );
}
fn check(m: &Mesh, expected: f64) {
    closed(m);
    let observed = volume(m);
    assert!(
        (observed - expected).abs() <= 1e-7 * expected.max(1.0),
        "volume {observed} != {expected}"
    );
}

#[test]
fn crossing_triangles_have_an_actual_intersection_segment() {
    let a = [[-1., 0., 0.], [1., 0., 0.], [0., 1., 0.]];
    let b = [[0., -1., -1.], [0., -1., 1.], [0., 1., 0.]];
    let TriangleIntersection::Segment(mut segment) =
        mesh_csg::triangle_intersection(a, b, 1e-9).unwrap()
    else {
        panic!("segment expected")
    };
    segment.sort_by(|a, b| a[1].total_cmp(&b[1]));
    assert_eq!(segment, [[0., 0., 0.], [0., 1., 0.]]);
    assert!(matches!(
        mesh_csg::triangle_intersection(a, a, 1e-9).unwrap(),
        TriangleIntersection::Coplanar(_)
    ));
    let far = a.map(|p| [p[0], p[1], p[2] + 2.]);
    assert_eq!(
        mesh_csg::triangle_intersection(a, far, 1e-9).unwrap(),
        TriangleIntersection::Disjoint
    );
}

#[test]
fn overlapping_boxes_union_difference_intersection_are_closed_and_measured() {
    let a = cube([0.; 3], [2.; 3]);
    let b = cube([1.; 3], [3.; 3]);
    for (op, expected) in [
        (BooleanOp::Union, 15.),
        (BooleanOp::Difference, 7.),
        (BooleanOp::Intersection, 1.),
    ] {
        let (m, r) = mesh_csg::mesh_boolean(&a, &b, op, CsgOptions::default()).unwrap();
        check(&m, expected);
        assert!(r.split_events > 0);
        assert!(r.internal_fragments > 0);
        assert_eq!(r.output_triangles, m.faces.len());
        println!(
            "{op:?}: {} triangles, volume {}, work {}",
            m.faces.len(),
            volume(&m),
            r.work
        );
    }
}

#[test]
fn repair_rebuilds_the_union_of_crossing_shells_without_holes() {
    let a = cube([0.; 3], [2.; 3]);
    let b = cube([1.; 3], [3.; 3]);
    let input = join(&a, &b);
    let before = input.clone();
    let (out, r) = mesh_csg::repair_self_intersections(&input, CsgOptions::default()).unwrap();
    assert_eq!(input, before);
    check(&out, 15.);
    assert!(r.split_events > 0);
    assert!(out.vertices.iter().any(|p| {
        p.iter()
            .zip([1., 2., 2.])
            .all(|(a, b)| (a - b).abs() < 1e-9)
    }));
    // The intersection edge is retained exactly in the output arrangement.
    assert!(out.faces.len() > input.faces.len());
}

#[test]
fn coplanar_overlays_touching_faces_and_identical_solids_are_regularized() {
    let a = cube([0.; 3], [2.; 3]);
    for (b, vol) in [
        (cube([1., 0., 0.], [3., 2., 2.]), 12.),
        (cube([2., 0., 0.], [4., 2., 2.]), 16.),
        (a.clone(), 8.),
    ] {
        let (m, _) =
            mesh_csg::mesh_boolean(&a, &b, BooleanOp::Union, CsgOptions::default()).unwrap();
        check(&m, vol);
    }
    for op in [BooleanOp::Difference, BooleanOp::Intersection] {
        let (m, _) = mesh_csg::mesh_boolean(&a, &a, op, CsgOptions::default()).unwrap();
        check(&m, if op == BooleanOp::Difference { 0. } else { 8. });
    }
}

#[test]
fn disjoint_contained_and_inward_cavity_shells() {
    let outer = cube([0.; 3], [3.; 3]);
    let inner = cube([1.; 3], [2.; 3]);
    let far = cube([4.; 3], [5.; 3]);
    for (b, op, v) in [
        (&inner, BooleanOp::Union, 27.),
        (&inner, BooleanOp::Difference, 26.),
        (&inner, BooleanOp::Intersection, 1.),
        (&far, BooleanOp::Union, 28.),
        (&far, BooleanOp::Intersection, 0.),
        (&far, BooleanOp::Difference, 27.),
    ] {
        check(
            &mesh_csg::mesh_boolean(&outer, b, op, CsgOptions::default())
                .unwrap()
                .0,
            v,
        );
    }
    let mut cavity = inner;
    for f in &mut cavity.faces {
        f.reverse();
    }
    check(
        &mesh_csg::repair_self_intersections(&join(&outer, &cavity), CsgOptions::default())
            .unwrap()
            .0,
        26.,
    );
}

#[test]
fn new_corners_interpolate_primary_uvs_and_retain_seams() {
    let mut a = cube([0.; 3], [2.; 3]);
    let mut b = cube([1.; 3], [3.; 3]);
    // Each face has a distinct UV chart offset: welded vertices must not merge charts.
    for m in [&mut a, &mut b] {
        for (fi, f) in m.faces.iter().enumerate() {
            for &v in f {
                let p = m.vertices[v as usize];
                m.corner_uvs.push([p[0] + fi as f64 * 10., p[1] + p[2]]);
            }
        }
    }
    let (m, _) = mesh_csg::mesh_boolean(&a, &b, BooleanOp::Union, CsgOptions::default()).unwrap();
    check(&m, 15.);
    assert_eq!(m.corner_uvs.len(), m.faces.len() * 3);
    let mut offsets = Vec::new();
    for (fi, f) in m.faces.iter().enumerate() {
        let mut chart = Vec::new();
        for (ci, &v) in f.iter().enumerate() {
            let p = m.vertices[v as usize];
            let uv = m.corner_uvs[fi * 3 + ci];
            assert!((uv[1] - p[1] - p[2]).abs() < 1e-8);
            chart.push(uv[0] - p[0]);
            offsets.push((v, uv[0] - p[0]));
        }
        assert!(chart.iter().all(|x| (x - chart[0]).abs() < 1e-8));
    }
    assert!(
        offsets
            .iter()
            .any(|&(v, x)| offsets.iter().any(|&(w, y)| v == w && (x - y).abs() > 1.))
    );
}

#[test]
fn invalid_open_nonmanifold_and_budget_inputs_leave_sources_unchanged() {
    let a = cube([0.; 3], [2.; 3]);
    let b = cube([1.; 3], [3.; 3]);
    let mut open = a.clone();
    open.faces.pop();
    assert!(mesh_csg::repair_self_intersections(&open, CsgOptions::default()).is_err());
    let mut invalid = a.clone();
    invalid.vertices[0][0] = f64::NAN;
    assert!(mesh_csg::repair_self_intersections(&invalid, CsgOptions::default()).is_err());
    let mut wrong = a.clone();
    wrong.faces[0].reverse();
    assert!(mesh_csg::repair_self_intersections(&wrong, CsgOptions::default()).is_err());
    let mut colour = a.clone();
    colour.colors = vec![[255; 4]; colour.vertices.len()];
    assert!(mesh_csg::repair_self_intersections(&colour, CsgOptions::default()).is_err());
    for o in [
        CsgOptions {
            max_work: 1,
            ..CsgOptions::default()
        },
        CsgOptions {
            max_fragments: 30,
            ..CsgOptions::default()
        },
        CsgOptions {
            max_triangles: 1,
            ..CsgOptions::default()
        },
        CsgOptions {
            tolerance: 0.,
            ..CsgOptions::default()
        },
    ] {
        assert!(mesh_csg::mesh_boolean(&a, &b, BooleanOp::Union, o).is_err());
    }
    assert_eq!(a, cube([0.; 3], [2.; 3]));
    assert_eq!(b, cube([1.; 3], [3.; 3]));
    // Union of edge-touching cubes has a nonmanifold edge; return an error.
    let edge_touch = cube([2., 2., 0.], [4., 4., 2.]);
    assert!(
        mesh_csg::mesh_boolean(&a, &edge_touch, BooleanOp::Union, CsgOptions::default()).is_err()
    );
}

#[test]
fn scale_translation_and_reversed_winding_preserve_geometry() {
    for s in [0.01, 1., 1000.] {
        let t = [1000., -2000., 3000.];
        let transform = |mut m: Mesh| {
            for p in &mut m.vertices {
                for i in 0..3 {
                    p[i] = t[i] + p[i] * s;
                }
            }
            m
        };
        let mut a = transform(cube([0.; 3], [2.; 3]));
        let mut b = transform(cube([1.; 3], [3.; 3]));
        for m in [&mut a, &mut b] {
            for f in &mut m.faces {
                f.reverse();
            }
        }
        let (m, _) = mesh_csg::mesh_boolean(
            &a,
            &b,
            BooleanOp::Union,
            CsgOptions {
                tolerance: s * 1e-8,
                ..CsgOptions::default()
            },
        )
        .unwrap();
        check(&m, 15. * s * s * s);
    }
}

#[test]
fn connected_self_crossing_pentagram_prism_rebuilds_material_boundary() {
    let mut m = Mesh {
        vertices: vec![],
        faces: vec![],
        normals: vec![],
        colors: vec![],
        corner_uvs: vec![],
    };
    for z in [0., 1.] {
        for i in [0, 2, 4, 1, 3] {
            let angle = i as f64 * std::f64::consts::TAU / 5.;
            m.vertices.push([2. * angle.cos(), 2. * angle.sin(), z]);
        }
        m.vertices.push([0., 0., z]);
    }
    for i in 0..5 {
        let j = (i + 1) % 5;
        m.faces.extend([
            vec![5, j, i],
            vec![11, i + 6, j + 6],
            vec![i, j, j + 6],
            vec![i, j + 6, i + 6],
        ]);
    }
    closed(&m);
    let (out, r) = mesh_csg::repair_self_intersections(&m, CsgOptions::default()).unwrap();
    let inner = 2. * (std::f64::consts::TAU / 5.).cos() / (std::f64::consts::PI / 5.).cos();
    let area = 5. * 2. * inner * (std::f64::consts::PI / 5.).sin();
    check(&out, area);
    assert!(r.split_events > 0);
    println!(
        "connected pentagram: input signed volume {}, repaired {}, expected {}, triangles {}",
        volume(&m),
        volume(&out),
        area,
        out.faces.len()
    );
}

#[test]
fn sloped_tetrahedra_boolean_matches_analytic_volumes() {
    let a = Mesh {
        vertices: vec![[0., 0., 0.], [2., 0., 0.], [0., 2., 0.], [0., 0., 2.]],
        faces: vec![vec![0, 2, 1], vec![0, 1, 3], vec![0, 3, 2], vec![1, 2, 3]],
        normals: vec![],
        colors: vec![],
        corner_uvs: vec![],
    };
    let mut b = a.clone();
    for p in &mut b.vertices {
        p[0] += 0.5;
    }
    let vi = 1.5_f64.powi(3) / 6.;
    let va = 8. / 6.;
    for (op, v) in [
        (BooleanOp::Union, 2. * va - vi),
        (BooleanOp::Intersection, vi),
        (BooleanOp::Difference, va - vi),
    ] {
        check(
            &mesh_csg::mesh_boolean(&a, &b, op, CsgOptions::default())
                .unwrap()
                .0,
            v,
        );
    }
}

#[test]
fn rotated_offset_solid_pairs_keep_expected_volumes() {
    let mut tested = 0;
    for k in 1..=8 {
        let angle = k as f64 * 0.17;
        let (s, c) = angle.sin_cos();
        let d = k as f64 / 10.;
        let transform = |mut m: Mesh| {
            for p in &mut m.vertices {
                let [x, y, z] = *p;
                *p = [c * x - s * y + 12., s * x + c * y - 9., z + 1.];
            }
            m
        };
        let a = transform(cube([0.; 3], [2.; 3]));
        let b = transform(cube([d; 3], [2. + d; 3]));
        let vi = (2. - d).powi(3);
        for (op, v) in [
            (BooleanOp::Union, 16. - vi),
            (BooleanOp::Intersection, vi),
            (BooleanOp::Difference, 8. - vi),
        ] {
            check(
                &mesh_csg::mesh_boolean(&a, &b, op, CsgOptions::default())
                    .unwrap()
                    .0,
                v,
            );
            tested += 1;
        }
    }
    println!("rotated translated pairs: {tested} analytic boolean fixtures passed");
}

#[test]
fn detection_ignores_shared_boundaries_and_finds_coplanar_area_overlap() {
    let a = cube([0.; 3], [2.; 3]);
    assert!(om9_core::mesh_repair::self_intersections(&a, 100).is_empty());
    let m = Mesh {
        vertices: vec![
            [0., 0., 0.],
            [2., 0., 0.],
            [0., 2., 0.],
            [0.2, 0.2, 0.],
            [2.2, 0.2, 0.],
            [0.2, 2.2, 0.],
        ],
        faces: vec![vec![0, 1, 2], vec![3, 4, 5]],
        normals: vec![],
        colors: vec![],
        corner_uvs: vec![],
    };
    assert_eq!(
        om9_core::mesh_repair::self_intersections(&m, 1),
        vec![(0, 1)]
    );
    assert!(om9_core::mesh_repair::self_intersections(&m, 0).is_empty());
}

#[test]
fn independently_rotated_operands_satisfy_volume_conservation() {
    let a = cube([0.; 3], [2.; 3]);
    let mut b = a.clone();
    let (s, c) = 0.31_f64.sin_cos();
    let (u, v) = 0.27_f64.sin_cos();
    for p in &mut b.vertices {
        let [x, y, z] = *p;
        *p = [
            c * x - s * y + 0.3,
            v * (s * x + c * y) - u * z + 0.5,
            u * (s * x + c * y) + v * z + 0.4,
        ];
    }
    let union = mesh_csg::mesh_boolean(&a, &b, BooleanOp::Union, CsgOptions::default())
        .unwrap()
        .0;
    let intersection =
        mesh_csg::mesh_boolean(&a, &b, BooleanOp::Intersection, CsgOptions::default())
            .unwrap()
            .0;
    let difference = mesh_csg::mesh_boolean(&a, &b, BooleanOp::Difference, CsgOptions::default())
        .unwrap()
        .0;
    for m in [&union, &intersection, &difference] {
        closed(m);
    }
    assert!((volume(&union) + volume(&intersection) - 16.).abs() < 1e-7);
    assert!((volume(&difference) + volume(&intersection) - 8.).abs() < 1e-7);
    assert!(volume(&intersection) > 0. && volume(&intersection) < 8.);
    let encoded = serde_json::to_vec(&union).unwrap();
    let decoded: Mesh = serde_json::from_slice(&encoded).unwrap();
    check(&decoded, volume(&union));
}

fn subdivide(mut mesh: Mesh, levels: usize) -> Mesh {
    for _ in 0..levels {
        let mut midpoints = BTreeMap::new();
        let mut faces = Vec::new();
        for f in &mesh.faces {
            let mut mid = [0; 3];
            for i in 0..3 {
                let a = f[i];
                let b = f[(i + 1) % 3];
                mid[i] = *midpoints.entry((a.min(b), a.max(b))).or_insert_with(|| {
                    let p = mesh.vertices[a as usize];
                    let q = mesh.vertices[b as usize];
                    mesh.vertices.push([
                        (p[0] + q[0]) * 0.5,
                        (p[1] + q[1]) * 0.5,
                        (p[2] + q[2]) * 0.5,
                    ]);
                    (mesh.vertices.len() - 1) as u32
                });
            }
            let [a, b, c] = [f[0], f[1], f[2]];
            let [ab, bc, ca] = mid;
            faces.extend([
                vec![a, ab, ca],
                vec![ab, b, bc],
                vec![ca, bc, c],
                vec![ab, bc, ca],
            ]);
        }
        mesh.faces = faces;
    }
    mesh
}

#[test]
#[ignore = "explicit release-mode scaling measurement"]
fn mesh_csg_scaling_measurement() {
    let mut passed = 0;
    for level in [0, 2, 3, 4, 5] {
        let a = subdivide(cube([0.; 3], [2.; 3]), level);
        let b = subdivide(cube([1.; 3], [3.; 3]), level);
        let input = a.faces.len() + b.faces.len();
        let start = std::time::Instant::now();
        let result = mesh_csg::mesh_boolean(&a, &b, BooleanOp::Union, CsgOptions::default());
        match result {
            Ok((m, r)) => {
                passed += 1;
                check(&m, 15.);
                println!(
                    "SCALING input={input} output={} work={} elapsed_ms={:.3} result=PASS",
                    m.faces.len(),
                    r.work,
                    start.elapsed().as_secs_f64() * 1000.
                );
            }
            Err(e) => println!(
                "SCALING input={input} elapsed_ms={:.3} result=ERR error={e:?}",
                start.elapsed().as_secs_f64() * 1000.
            ),
        }
    }
    assert_eq!(passed, 5, "all five levels must pass using default budgets");
}

fn curved_sphere(level: usize, center: [f64; 3]) -> Mesh {
    let mut mesh = Mesh {
        vertices: vec![
            [1., 0., 0.],
            [-1., 0., 0.],
            [0., 1., 0.],
            [0., -1., 0.],
            [0., 0., 1.],
            [0., 0., -1.],
        ],
        faces: vec![
            vec![0, 2, 4],
            vec![2, 1, 4],
            vec![1, 3, 4],
            vec![3, 0, 4],
            vec![2, 0, 5],
            vec![1, 2, 5],
            vec![3, 1, 5],
            vec![0, 3, 5],
        ],
        normals: vec![],
        colors: vec![],
        corner_uvs: vec![],
    };
    for _ in 0..level {
        mesh = subdivide(mesh, 1);
        for p in &mut mesh.vertices {
            let length = p[0].hypot(p[1]).hypot(p[2]);
            for x in p {
                *x /= length;
            }
        }
    }
    for p in &mut mesh.vertices {
        for i in 0..3 {
            p[i] += center[i];
        }
    }
    mesh
}

#[test]
#[ignore = "explicit release-mode curved mesh scaling measurement"]
fn mesh_csg_curved_scaling_measurement() {
    for level in [3, 4, 5] {
        let a = curved_sphere(level, [0.; 3]);
        let b = curved_sphere(level, [0.47, 0.13, 0.09]);
        let start = std::time::Instant::now();
        let union =
            mesh_csg::mesh_boolean(&a, &b, BooleanOp::Union, CsgOptions::default()).unwrap();
        println!(
            "CURVED input={} output={} work={} rays={} fallbacks={} elapsed_ms={:.3}",
            a.faces.len() + b.faces.len(),
            union.0.faces.len(),
            union.1.work,
            union.1.ray_triangle_tests,
            union.1.winding_fallbacks,
            start.elapsed().as_secs_f64() * 1000.
        );
        let intersection =
            mesh_csg::mesh_boolean(&a, &b, BooleanOp::Intersection, CsgOptions::default())
                .unwrap()
                .0;
        closed(&union.0);
        closed(&intersection);
        assert!((volume(&union.0) + volume(&intersection) - volume(&a) - volume(&b)).abs() < 1e-7);
    }
}

#[test]
fn dense_curved_fixture_repairs_at_the_original_tolerance() {
    let a = curved_sphere(5, [0.; 3]);
    let b = curved_sphere(5, [0.47, 0.13, 0.09]);
    let before_a = a.clone();
    let before_b = b.clone();
    let options = CsgOptions::default();
    assert_eq!(options.tolerance, 1e-8);
    assert_eq!(a.faces.len() + b.faces.len(), 16_384);
    let start = std::time::Instant::now();
    let union =
        mesh_csg::mesh_boolean(&a, &b, BooleanOp::Union, options).expect("16K curved union");
    let intersection = mesh_csg::mesh_boolean(&a, &b, BooleanOp::Intersection, options)
        .expect("16K curved intersection");
    let difference = mesh_csg::mesh_boolean(&a, &b, BooleanOp::Difference, options)
        .expect("16K curved difference");
    for m in [&union.0, &intersection.0, &difference.0] {
        closed(m);
    }
    assert!((volume(&union.0) + volume(&intersection.0) - volume(&a) - volume(&b)).abs() < 1e-7);
    assert!((volume(&difference.0) + volume(&intersection.0) - volume(&a)).abs() < 1e-7);
    let mut joined = join(&a, &b);
    for (fi, f) in joined.faces.iter().enumerate() {
        for &id in f {
            let p = joined.vertices[id as usize];
            joined
                .corner_uvs
                .push([p[0] + fi as f64 * 10., p[1] + p[2]]);
        }
    }
    let before_joined = joined.clone();
    let repaired = mesh_csg::repair_self_intersections(&joined, options)
        .expect("16K curved self-overlap repair");
    closed(&repaired.0);
    assert!((volume(&repaired.0) - volume(&union.0)).abs() < 1e-7);
    assert_eq!(repaired.0.corner_uvs.len(), repaired.0.faces.len() * 3);
    for (fi, f) in repaired.0.faces.iter().enumerate() {
        let mut offsets = Vec::new();
        for (ci, &id) in f.iter().enumerate() {
            let p = repaired.0.vertices[id as usize];
            let uv = repaired.0.corner_uvs[fi * 3 + ci];
            assert!((uv[1] - p[1] - p[2]).abs() < 1e-8);
            offsets.push(uv[0] - p[0]);
        }
        assert!(offsets.iter().all(|x| (x - offsets[0]).abs() < 1e-8));
    }
    assert_eq!(a, before_a);
    assert_eq!(b, before_b);
    assert_eq!(joined, before_joined);
    println!(
        "CURVED16K union_volume={:.14} intersection_volume={:.14} difference_volume={:.14} repair_volume={:.14} union_triangles={} repair_triangles={} work={} elapsed_all_ms={:.3}",
        volume(&union.0),
        volume(&intersection.0),
        volume(&difference.0),
        volume(&repaired.0),
        union.0.faces.len(),
        repaired.0.faces.len(),
        repaired.1.work,
        start.elapsed().as_secs_f64() * 1000.
    );
}

#[test]
fn mismatched_dense_coplanar_triangulations_rebuild_without_seam_holes() {
    let mut a = subdivide(cube([0.; 3], [2.; 3]), 3);
    let mut b = subdivide(cube([1.; 3], [3.; 3]), 2);
    // Primary UV channel is affine within each face, with explicit chart seams.
    for m in [&mut a, &mut b] {
        for (fi, f) in m.faces.iter().enumerate() {
            for &id in f {
                let p = m.vertices[id as usize];
                m.corner_uvs.push([p[0] + fi as f64 * 10., p[1] + p[2]]);
            }
        }
    }
    for (op, expected) in [
        (BooleanOp::Union, 15.),
        (BooleanOp::Difference, 7.),
        (BooleanOp::Intersection, 1.),
    ] {
        let (mesh, report) = mesh_csg::mesh_boolean(&a, &b, op, CsgOptions::default()).unwrap();
        check(&mesh, expected);
        assert_eq!(mesh.corner_uvs.len(), mesh.faces.len() * 3);
        assert!(report.ray_triangle_tests > 0);
        assert!(report.work < CsgOptions::default().max_work);
    }
}
