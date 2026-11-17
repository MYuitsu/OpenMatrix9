use om9_render::{normals, shapes};

fn faces(indices: &[u32]) -> Vec<([u32; 3], u32)> {
    indices
        .chunks_exact(3)
        .map(|t| ([t[0], t[1], t[2]], 0))
        .collect()
}
#[test]
fn sphere_normals_follow_curvature_without_moving_display_triangles() {
    for (rings, segments) in [(16, 16), (64, 64)] {
        let input = shapes::sphere(rings, segments, 5.);
        let smooth = normals::smooth_faces(input.positions.clone(), &faces(&input.indices));
        assert_eq!(smooth.indices.len(), input.indices.len());
        for (before, after) in input.indices.iter().zip(&smooth.indices) {
            assert_eq!(
                input.positions[*before as usize],
                smooth.positions[*after as usize]
            );
        }
        for (p, n) in smooth.positions.iter().zip(&smooth.normals) {
            let length = n.iter().map(|v| v * v).sum::<f32>().sqrt();
            assert!(length.is_finite());
            // Degenerate pole triangles have zero area. Every used sphere vertex
            // also touches a non-degenerate triangle and receives a unit normal.
            assert!(length > 0.99, "{p:?}: {n:?}");
            let cosine = p.iter().zip(n).map(|(a, b)| a * b).sum::<f32>().abs() / 5.;
            assert!(
                cosine > 0.97,
                "normal deviates from radial direction: {p:?} {n:?}"
            );
        }
    }
}
#[test]
fn face_boundaries_and_sharp_temporary_mesh_edges_stay_sharp() {
    let positions = vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.], [0., 0., 1.]];
    for second_face in [0, 1] {
        let mesh = normals::smooth_faces(
            positions.clone(),
            &[([0, 1, 2], 0), ([0, 3, 1], second_face)],
        );
        assert_eq!(mesh.positions.len(), 6);
        assert_eq!(mesh.normals[mesh.indices[0] as usize], [0., 0., 1.]);
        assert_eq!(mesh.normals[mesh.indices[3] as usize], [0., 1., 0.]);
    }
}
#[test]
fn cofactor_normal_remains_perpendicular_under_scale_shear_and_mirror() {
    let model = [
        [2., 1., 0., 0.],
        [0., 3., 0.5, 0.],
        [0., 0., -4., 0.],
        [0., 0., 0., 1.],
    ];
    let normal = normals::transform([0., 0., 1.], model);
    for tangent in &model[..2] {
        let dot = (0..3).map(|i| normal[i] * tangent[i]).sum::<f32>();
        assert!(dot.abs() < 1e-6);
    }
}
