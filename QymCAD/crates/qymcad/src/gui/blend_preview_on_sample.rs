//! THE PREVIEW OF A FILLET ON A REAL PART LIES ON ITS FACES: on the private sample of a frame with bosses at its corners,
//! the lines where the R8.6 fillet meets the faces - drawn by the same function the window draws the preview with - lie
//! on the part's surface, curved faces too. Reported behaviour, with a picture of that frame: the arcs of the preview
//! lay on the other side of the edge, in the body; and beside the round corners R5 the line ran straight along the
//! face's tangent, 2.1 mm off the curved face, into the air. Reported behaviour, with a picture of that frame: the violet arcs of the
//! preview lay on the other side of the edge, in the body, while the fillet itself was built outwards.
#[cfg(test)]
mod tests {
    use super::super::App;

    /// How far `p` is from the nearest triangle of `mesh`.
    fn distance_to(mesh: &qymcad_core::geom::Mesh, p: [f64; 3]) -> f64 {
        (0..mesh.tris.len())
            .map(|t| {
                let [a, b, c] = mesh.triangle(t);
                let q = qymcad_ui_state::closest_on_triangle(p, &[[a.x, a.y, a.z], [b.x, b.y, b.z], [c.x, c.y, c.z]]);
                ((q[0] - p[0]).powi(2) + (q[1] - p[1]).powi(2) + (q[2] - p[2]).powi(2)).sqrt()
            })
            .fold(f64::MAX, f64::min)
    }

    /// Whether `p` is inside the closed `mesh`: rays in three slanted directions, each crossing its triangles an odd number
    /// of times, by the majority - a point in the plane of a face makes one ray's count unreliable.
    fn inside(mesh: &qymcad_core::geom::Mesh, p: [f64; 3]) -> bool {
        [[0.577_1, 0.577_5, 0.577_3], [-0.4, 0.8, 0.44], [0.7, -0.3, -0.64]].into_iter().filter(|d| odd(mesh, p, *d)).count() >= 2
    }

    fn odd(mesh: &qymcad_core::geom::Mesh, p: [f64; 3], d: [f64; 3]) -> bool {
        let mut crossings = 0;
        for t in 0..mesh.tris.len() {
            let [a, b, c] = mesh.triangle(t);
            let (a, b, c) = ([a.x, a.y, a.z], [b.x, b.y, b.z], [c.x, c.y, c.z]);
            let sub = |x: [f64; 3], y: [f64; 3]| [x[0] - y[0], x[1] - y[1], x[2] - y[2]];
            let cross = |x: [f64; 3], y: [f64; 3]| [x[1] * y[2] - x[2] * y[1], x[2] * y[0] - x[0] * y[2], x[0] * y[1] - x[1] * y[0]];
            let dot = |x: [f64; 3], y: [f64; 3]| x[0] * y[0] + x[1] * y[1] + x[2] * y[2];
            let (e1, e2) = (sub(b, a), sub(c, a));
            let h = cross(d, e2);
            let det = dot(e1, h);
            if det.abs() < 1e-12 {
                continue;
            }
            let s = sub(p, a);
            let u = dot(s, h) / det;
            let q = cross(s, e1);
            let v = dot(d, q) / det;
            let t = dot(e2, q) / det;
            if (0.0..=1.0).contains(&u) && v >= 0.0 && u + v <= 1.0 && t > 1e-9 {
                crossings += 1;
            }
        }
        crossings % 2 == 1
    }

    #[test]
    fn the_fillet_preview_on_the_frame_sample_lies_on_its_faces() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../samples/vacuumCleaner.qcad");
        if !std::path::Path::new(path).exists() {
            eprintln!("PASSED OVER: the private sample vacuumCleaner.qcad is not in this tree");
            return;
        }
        let mut app = App { project: qymcad_io::load_project(path).expect("the sample opens"), ..Default::default() };
        app.project.mark_all_dirty();
        qymcad_ui_state::rebuild_if_dirty(&mut app.rebuild_ctx());
        let Some((src, radius, picked)) = app.project.timeline.iter().find(|n| n.id == 177).and_then(|n| match &n.kind {
            qymcad_core::feature::FeatureKind::Fillet { src, radius, edges, .. } => Some((*src, *radius, edges.query.picked_descs())),
            _ => None,
        }) else {
            panic!("the sample has no fillet 177");
        };
        let mesh = app.project.mesh_index(src).map(|mi| app.project.bodies[mi].mesh.clone()).expect("the body the fillet stands on");
        let edges = app.project.regen_edges.get(&src).cloned().unwrap_or_default();
        let mut far = Vec::new();
        let mut drawn = 0;
        for e in edges.iter().filter(|e| picked.contains(&e.id)) {
            let poly = [[e.a[0] as f32, e.a[1] as f32, e.a[2] as f32], [e.b[0] as f32, e.b[1] as f32, e.b[2] as f32]];
            let out = qymcad_ui_state::edge_blend_outline(&mesh, &poly, qymcad_ui_state::Blend::Round(radius));
            for rail in out.iter().take(2) {
                for p in rail {
                    drawn += 1;
                    let d = distance_to(&mesh, *p);
                    if d > 0.1 {
                        far.push(format!("edge {} at {p:?}: {d:.2} mm {} the part", e.id, if inside(&mesh, *p) { "inside" } else { "off" }));
                    }
                }
            }
        }
        assert!(drawn > 0, "the preview drew nothing for the R{radius} of the sample");
        assert!(far.is_empty(), "the preview of R{radius} leaves the faces of the part:\n{}", far.join("\n"));
    }
}
