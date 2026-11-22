// SPDX-License-Identifier: LGPL-2.1-or-later
use super::*;

fn xy(x: f64, y: f64) -> Vec3 {
    [x, y, 0.]
}
fn network(paths: &[Vec<Vec3>]) -> Arc<LineNetwork> {
    Arc::new(LineNetwork::from_paths(paths, 0.001, &mut |_| Ok(())).unwrap())
}
fn autos(net: &LineNetwork, max: usize) -> Vec<NetworkFace> {
    net.automatic_faces(max, &mut |_| Ok(())).unwrap()
}
#[test]
fn joins_real_endpoint_distances_without_splitting_interiors() {
    let paths = vec![
        vec![xy(-3., 0.), xy(0.99, 0.)],
        vec![xy(1.01, 0.), xy(6., 0.)],
        vec![xy(10.01, 0.01), xy(3., 4.)],
        vec![xy(10.99, 0.99), xy(12., 5.)],
        vec![xy(3., 0.), xy(3., 6.)],
    ];
    let net = LineNetwork::from_paths(&paths, 1., &mut |_| Ok(())).unwrap();
    assert_eq!(net.edges.len(), 5);
    assert_eq!(net.endpoint_counts.iter().filter(|&&n| n == 2).count(), 1);
    let mid = net.vertices.iter().position(|&p| p == xy(3., 0.)).unwrap();
    assert_eq!(net.neighbors[mid].len(), 1);
    let mut reversed = paths.clone();
    reversed.reverse();
    let other = LineNetwork::from_paths(&reversed, 1., &mut |_| Ok(())).unwrap();
    assert_eq!(net.vertices, other.vertices);
    assert_eq!(net.edges, other.edges);
    assert!(LineNetwork::from_paths(&paths, f64::NAN, &mut |_| Ok(())).is_err());
}
#[test]
fn cube_selects_six_quads_manual_works_with_auto_off_and_work_is_cancellable() {
    let p = |i: u32| [(i & 1) as f64, ((i >> 1) & 1) as f64, ((i >> 2) & 1) as f64];
    let mut paths = Vec::new();
    for i in 0..8 {
        for bit in [1, 2, 4] {
            if i & bit == 0 {
                paths.push(vec![p(i), p(i | bit)]);
            }
        }
    }
    let net = network(&paths);
    assert_eq!(net.vertices.len(), 8);
    assert_eq!(net.edges.len(), 12);
    assert!(net.endpoint_counts.iter().all(|&n| n == 3));
    assert!(autos(&net, 0).is_empty());
    let faces = autos(&net, 8);
    assert_eq!(faces.len(), 6);
    assert!(
        faces
            .iter()
            .all(|f| f.outer.len() == 4 && f.holes.is_empty())
    );
    let seed = *net.edges.first().unwrap();
    let preview = FaceLayout::new(net.clone(), faces, &mut |_| Ok(())).unwrap();
    let source = preview.cage(&mut |_| Ok(())).unwrap();
    for i in 0..source.poly().faces.len() {
        let normal = source.poly().face_normal(i);
        let center = source.poly().face_center(i);
        assert!(dot(normal, sub(center, [0.5; 3])) > 0.49);
    }
    let choices = net.manual_faces(seed, 4, &mut |_| Ok(())).unwrap();
    assert_eq!(choices.len(), 2);
    assert!(net.manual_faces(seed, 6, &mut |_| Ok(())).unwrap().len() > 2);
    let mut left = 20usize;
    let result = net.automatic_faces(100, &mut |n| {
        left = left.checked_sub(n).ok_or("cancelled")?;
        Ok(())
    });
    assert_eq!(result, Err("cancelled"));
}
#[test]
fn nested_boundaries_leave_an_actual_hole_in_a_tilted_concave_face() {
    let place = |x: f64, y: f64| [1e8 + x, 10. + 0.6 * y, -4. + 0.8 * y];
    let ring = |points: &[[f64; 2]]| {
        points
            .iter()
            .chain(points.first())
            .map(|p| place(p[0], p[1]))
            .collect()
    };
    let paths = vec![
        ring(&[[0., 0.], [8., 0.], [8., 3.], [6., 3.], [6., 7.], [0., 7.]]),
        ring(&[[1., 1.], [3., 1.], [3., 3.], [1., 3.]]),
    ];
    let net = network(&paths);
    let faces = autos(&net, 100);
    assert_eq!(faces.len(), 1);
    assert_eq!(faces[0].outer.len(), 6);
    assert_eq!(faces[0].holes.len(), 1);
    let mut area = 0.;
    for tri in faces[0].triangles(&net.vertices, &mut |_| Ok(())).unwrap() {
        let [a, b, c] = tri.map(|i| net.vertices[i as usize]);
        area += 0.5 * norm(cross(sub(b, a), sub(c, a)));
        let x = (a[0] + b[0] + c[0]) / 3. - 1e8;
        let y = ((a[2] + b[2] + c[2]) / 3. + 4.) / 0.8;
        assert!(!(x > 1. && x < 3. && y > 1. && y < 3.));
    }
    assert!((area - 44.).abs() < 1e-8, "{area}");
    let preview = FaceLayout::new(net.clone(), faces, &mut |_| Ok(())).unwrap();
    let normal = [0., -0.8, 0.6];
    let pick = |x, y| {
        let p = place(x, y);
        preview.pick_face(
            std::array::from_fn(|i| p[i] + 2. * normal[i]),
            normal.map(|v| -v),
        )
    };
    assert_eq!(pick(2., 2.), None, "ray through hole");
    assert_eq!(pick(7., 5.), None, "ray through concavity");
    assert_eq!(pick(0.5, 2.), Some(0), "ray through material");
    let outer_edge = boundary(&preview.faces()[0].outer).next().unwrap();
    let empty = FaceLayout::new(net.clone(), vec![], &mut |_| Ok(())).unwrap();
    let restored = empty.choices(outer_edge, 6, &mut |_| Ok(())).unwrap();
    assert_eq!(restored.len(), 1);
    assert_eq!(
        restored[0].holes.len(),
        1,
        "manual re-adding preserves the hole"
    );
}
#[test]
fn envelope_of_four_cells_is_not_an_extra_face() {
    let mut paths = Vec::new();
    for i in 0..3 {
        for j in 0..2 {
            paths.push(vec![xy(i as f64, j as f64), xy(i as f64, (j + 1) as f64)]);
            paths.push(vec![xy(j as f64, i as f64), xy((j + 1) as f64, i as f64)]);
        }
    }
    let net = network(&paths);
    let faces = autos(&net, 12);
    assert_eq!(faces.len(), 4);
    assert!(faces.iter().all(|f| f.outer.len() == 4));
}
#[test]
fn red_edges_and_disconnected_vertex_fans_follow_preview_edits() {
    let o = xy(0., 0.);
    let net = network(&[
        vec![o, xy(3., 0.), xy(0., 3.), o],
        vec![o, xy(-3., 0.), xy(0., -3.), o],
        vec![xy(10., 9.), xy(12., 9.)],
    ]);
    let mut faces = autos(&net, 4);
    let both = FaceLayout::new(net.clone(), faces.clone(), &mut |_| Ok(())).unwrap();
    assert_eq!(both.unused_edges.len(), 1);
    assert_eq!(both.nonmanifold_vertices.len(), 1);
    assert_eq!(net.vertices[both.nonmanifold_vertices[0] as usize], o);
    let (cage, ids) = both.cage(&mut |_| Ok(())).unwrap().into_parts();
    assert_eq!(ids, [0, 1]);
    assert_eq!(cage.verts.len(), 6);
    assert_eq!(cage.verts.iter().filter(|&&p| p == o).count(), 2);
    let removed = faces.pop().unwrap();
    let seed = boundary(&removed.outer).next().unwrap();
    let one = FaceLayout::new(net.clone(), faces.clone(), &mut |_| Ok(())).unwrap();
    assert_eq!(one.unused_edges.len(), 4);
    assert!(one.nonmanifold_vertices.is_empty());
    let choices = one.choices(seed, 3, &mut |_| Ok(())).unwrap();
    assert_eq!(choices.len(), 1);
    faces.extend(choices);
    assert_eq!(
        FaceLayout::new(net.clone(), faces, &mut |_| Ok(()))
            .unwrap()
            .nonmanifold_vertices,
        both.nonmanifold_vertices
    );
}
#[test]
fn cylinder_end_offers_one_pentagon_above_manual_limit() {
    let p = |i: usize, z: f64| {
        let angle = (i % 5) as f64 * std::f64::consts::TAU / 5.;
        [angle.cos(), angle.sin(), z]
    };
    let mut paths = Vec::new();
    for i in 0..5 {
        paths.push(vec![p(i, 0.), p(i + 1, 0.)]);
        paths.push(vec![p(i, 1.), p(i + 1, 1.)]);
        paths.push(vec![p(i, 0.), p(i, 1.)]);
    }
    let net = network(&paths);
    let side_faces = autos(&net, 4);
    assert_eq!(side_faces.len(), 5);
    let preview = FaceLayout::new(net.clone(), side_faces, &mut |_| Ok(())).unwrap();
    let edge = *net
        .edges
        .iter()
        .find(|&&(a, b)| net.vertices[a as usize][2] == 1. && net.vertices[b as usize][2] == 1.)
        .unwrap();
    let caps = preview.choices(edge, 3, &mut |_| Ok(())).unwrap();
    assert_eq!(caps.len(), 1);
    assert_eq!(caps[0].outer.len(), 5);
    let mut with_cap = preview.faces;
    with_cap.extend(caps);
    let updated = FaceLayout::new(net.clone(), with_cap, &mut |_| Ok(())).unwrap();
    let source = updated.cage(&mut |_| Ok(())).unwrap();
    let display = source.display(0, &mut |_| Ok(())).unwrap();
    assert_eq!(display.face_ids().iter().filter(|&&i| i == 5).count(), 3);
    let smooth = source.display(1, &mut |_| Ok(())).unwrap();
    assert_eq!(smooth.face_ids().iter().filter(|&&i| i == 5).count(), 5);
    assert!(smooth.poly().faces.iter().all(|p| p.len() == 4));
    let (cage, owners) = source.into_parts();
    assert_eq!(cage.faces.len(), 6);
    assert_eq!(cage.faces[5].len(), 5);
    assert_eq!(owners[5], 5);
    assert!(
        updated
            .choices(edge, 4, &mut |_| Ok(()))
            .unwrap()
            .is_empty()
    );
}
#[test]
fn high_face_limit_keeps_concave_polyline_sides_as_distinct_faces() {
    // Like the branched outline on p94: two long polygonal paths share a short edge.
    let a = xy(0., 0.);
    let b = xy(1., 2.);
    let net = network(&[
        vec![
            a,
            xy(-1., 1.),
            xy(-2., 4.),
            xy(-3., 5.),
            xy(-2., 6.),
            xy(0., 6.),
            xy(-1., 4.),
            b,
        ],
        vec![
            a,
            xy(2., 0.),
            xy(4., 2.),
            xy(6., 6.),
            xy(5., 6.),
            xy(3., 3.),
            b,
        ],
        vec![a, b],
    ]);
    let low = FaceLayout::new(net.clone(), autos(&net, 4), &mut |_| Ok(())).unwrap();
    assert_eq!(
        low.unused_edges,
        net.edges.iter().copied().collect::<Vec<_>>()
    );
    let high = FaceLayout::new(net.clone(), autos(&net, 100), &mut |_| Ok(())).unwrap();
    assert_eq!(high.faces.len(), 2);
    assert!(high.unused_edges.is_empty());
    let (cage, _) = high.cage(&mut |_| Ok(())).unwrap().into_parts();
    assert_eq!(
        cage.edge_faces()
            .values()
            .filter(|faces| faces.len() == 2)
            .count(),
        1
    );
}
