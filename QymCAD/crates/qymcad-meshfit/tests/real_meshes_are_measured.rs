//! Preparation measured on real files. Ignored: it reads the directory named by QYM_SAMPLES and prints, per
//! mesh, what preparation found and how long it took.

use qymcad_core::geom::Mesh;
use qymcad_kernel::recognise::recognise as solid;
use qymcad_meshfit::{boundaries, curves, prepare, regions, weld_tolerance, Curve, Surface, Tolerance, OPEN};

fn meshes(path: &std::path::Path) -> Vec<(String, Mesh)> {
    let s = path.to_string_lossy().to_string();
    let ext = path.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
    let one = |r: Result<Mesh, String>| r.map(|m| vec![(String::new(), m)]).unwrap_or_default();
    let named = |r: Result<Vec<qymcad_io::NamedMesh>, String>| r.map(|v| v.into_iter().map(|n| (n.name, n.mesh)).collect()).unwrap_or_default();
    match ext.as_str() {
        "stl" => one(qymcad_io::import_stl(&s)),
        "ply" => one(qymcad_io::import_ply(&s)),
        "obj" => named(qymcad_io::import_obj(&s)),
        "glb" | "gltf" => named(qymcad_io::import_gltf(&s)),
        "3mf" => named(qymcad_io::import_3mf(&s)),
        "amf" => named(qymcad_io::import_amf(&s)),
        _ => Vec::new(),
    }
}

#[test]
#[ignore = "reads the files under QYM_SAMPLES"]
fn real_meshes_are_measured() {
    let Ok(dir) = std::env::var("QYM_SAMPLES") else { return };
    let mut files: Vec<_> = std::fs::read_dir(dir).expect("the samples read").flatten().map(|e| e.path()).collect();
    files.sort();
    for f in files {
        for (name, mesh) in meshes(&f) {
            let started = std::time::Instant::now();
            let p = prepare(&mesh, weld_tolerance(&mesh));
            let prepared_ms = started.elapsed().as_secs_f64() * 1e3;
            let split = std::time::Instant::now();
            let found = regions(&p, &Tolerance::for_mesh(&p));
            let kinds = found.iter().fold([0usize; 6], |mut k, r| {
                k[match r.surface {
                    Some(Surface::Plane { .. }) => 0,
                    Some(Surface::Cylinder { .. }) => 1,
                    Some(Surface::Sphere { .. }) => 2,
                    Some(Surface::Free { .. }) => 4,
                    Some(Surface::Helix { .. } | Surface::RoundHelix { .. } | Surface::Coil { .. }) => 5,
                    _ => 3,
                }] += 1;
                k
            });
            let single = found.iter().filter(|r| r.tris.len() == 1).count();
            println!(
                "    regions {} (planes {}, cylinders {}, spheres {}, other {}, free forms {}, screws {}; of one triangle {single}), {:.1} ms",
                found.len(),
                kinds[0],
                kinds[1],
                kinds[2],
                kinds[3],
                kinds[4],
                kinds[5],
                split.elapsed().as_secs_f64() * 1e3
            );
            let walked = std::time::Instant::now();
            let b = boundaries(&p, &found);
            let closed = b.edges.iter().filter(|e| e.ends.is_none()).count();
            let open = b.edges.iter().filter(|e| e.faces[1] == OPEN).count();
            let holed = b.loops.iter().filter(|l| l.len() > 1).count();
            let spent = walked.elapsed().as_secs_f64() * 1e3;
            let tol = Tolerance::for_mesh(&p);
            let moved: Vec<f64> = (0..b.vertices.len())
                .map(|v| {
                    let (q, m) = (b.vertices[v], p.mesh.verts[b.corners[v] as usize]);
                    ((q[0] - m.x).powi(2) + (q[1] - m.y).powi(2) + (q[2] - m.z).powi(2)).sqrt()
                })
                .collect();
            let far = moved.iter().filter(|&&d| d > 10.0 * tol.distance).count();
            println!("    corners {}, edges {} (closed {closed}, along a border {open}), faces of more than one loop {holed}, {spent:.1} ms", b.vertices.len(), b.edges.len());
            println!("    corners moved off their mesh corner: at most {:.3e} mm, farther than 10 tolerances ({:.1e}) {far}", moved.iter().copied().fold(0.0, f64::max), 10.0 * tol.distance);
            let traced = std::time::Instant::now();
            let found_curves = curves(&b, &found, &tol);
            let (lines, circles) = (found_curves.iter().filter(|c| matches!(c, Curve::Line { .. })).count(), found_curves.iter().filter(|c| matches!(c, Curve::Circle { .. })).count());
            println!("    curves: lines {lines}, circles {circles}, points {}, {:.1} ms", found_curves.len() - lines - circles, traced.elapsed().as_secs_f64() * 1e3);
            // the cap is lifted by `QYM_MAX_REGIONS`, for the measure of its own that the big meshes are
            if found.len() > std::env::var("QYM_MAX_REGIONS").ok().and_then(|v| v.parse().ok()).unwrap_or(3000) {
                println!("    body: not built here - {} regions, and sewing that many faces is a measure of its own", found.len());
            } else {
                let sewn = std::time::Instant::now();
                match solid(&p, &found, &b, &found_curves, &tol) {
                    Some(made) => {
                        let (body, areas, free, free_at, dropped) = (made.shape, made.areas, made.free_edges, made.free_at, made.loose_dropped);
                        println!(
                            "    body: {} of {} faces built, {}, free sides {free}, triangles dropped {dropped}, volume {:.3} against the mesh's {:.3}, {:.1} ms",
                            areas.iter().flatten().count(),
                            found.len(),
                            if body.is_sheet() { "a shell" } else { "a solid" },
                            body.volume(),
                            p.mesh.volume(),
                            sewn.elapsed().as_secs_f64() * 1e3
                        );
                        // HOW SHORT THE SIDES ARE WHERE A REGION LEFT AS MESH MEETS A FACE, against the ten tolerances the
                        // sewing works to: a side shorter than that is one the sewing may take for a point
                        {
                            let beside: Vec<usize> = (0..b.edges.len()).filter(|&e| b.edges[e].faces.iter().any(|&f| f != OPEN && areas[f as usize].is_none())).collect();
                            let mut shortest: Vec<f64> = beside
                                .iter()
                                .flat_map(|&e| b.edges[e].points.windows(2).map(|w| ((w[1][0] - w[0][0]).powi(2) + (w[1][1] - w[0][1]).powi(2) + (w[1][2] - w[0][2]).powi(2)).sqrt()))
                                .collect();
                            shortest.sort_by(f64::total_cmp);
                            let under = shortest.iter().filter(|&&d| d < 10.0 * tol.distance).count();
                            println!(
                                "      edges beside a region left as mesh: {}, their sides {} - shortest {:.3e}, tenth {:.3e}, median {:.3e} mm; under the sewing's {:.3e}: {under}",
                                beside.len(),
                                shortest.len(),
                                shortest.first().copied().unwrap_or(0.0),
                                shortest.get(shortest.len() / 10).copied().unwrap_or(0.0),
                                shortest.get(shortest.len() / 2).copied().unwrap_or(0.0),
                                10.0 * tol.distance
                            );
                        }
                        // WHERE THE SHELL STAYS OPEN: the mesh corner nearest each side left free, and the regions that
                        // corner belongs to - the ones left as mesh marked, since those are the seams to suspect
                        if !free_at.is_empty() {
                            let mut owner: Vec<Vec<usize>> = vec![Vec::new(); p.mesh.verts.len()];
                            for (r, region) in found.iter().enumerate() {
                                for &t in &region.tris {
                                    for &i in &p.mesh.tris[t as usize] {
                                        if !owner[i as usize].contains(&r) {
                                            owner[i as usize].push(r);
                                        }
                                    }
                                }
                            }
                            for at in free_at.iter().take(6) {
                                let away = |i: usize| {
                                    let q = p.mesh.verts[i];
                                    (q.x - at[0]).powi(2) + (q.y - at[1]).powi(2) + (q.z - at[2]).powi(2)
                                };
                                let near = (0..p.mesh.verts.len()).min_by(|&i, &j| away(i).total_cmp(&away(j))).unwrap_or(0);
                                let who: Vec<String> = owner[near].iter().map(|&r| format!("{r}{} ({} tris)", if areas[r].is_none() { " as mesh" } else { "" }, found[r].tris.len())).collect();
                                println!("      a side left free at [{:.3}, {:.3}, {:.3}], {:.3e} mm from corner {near} of regions {}", at[0], at[1], at[2], away(near).sqrt(), who.join(", "));
                                // and the edge of the graph it lies nearest: its two faces are the ones that were meant to meet there
                                let to_edge = |e: usize| {
                                    b.edges[e]
                                        .points
                                        .windows(2)
                                        .map(|w| {
                                            let (from, along) = (w[0], [w[1][0] - w[0][0], w[1][1] - w[0][1], w[1][2] - w[0][2]]);
                                            let len = along[0] * along[0] + along[1] * along[1] + along[2] * along[2];
                                            let t = if len > 0.0 { (((at[0] - from[0]) * along[0] + (at[1] - from[1]) * along[1] + (at[2] - from[2]) * along[2]) / len).clamp(0.0, 1.0) } else { 0.0 };
                                            (from[0] + along[0] * t - at[0]).powi(2) + (from[1] + along[1] * t - at[1]).powi(2) + (from[2] + along[2] * t - at[2]).powi(2)
                                        })
                                        .fold(f64::MAX, f64::min)
                                };
                                if let Some(side) = (0..b.edges.len()).min_by(|&i, &j| to_edge(i).total_cmp(&to_edge(j))) {
                                    let kind = match &found_curves[side] {
                                        Curve::Line { .. } => "a line",
                                        Curve::Circle { .. } => "a circle",
                                        Curve::Ellipse { .. } => "an ellipse",
                                        Curve::Points(_) => "points",
                                    };
                                    let whose =
                                        b.edges[side].faces.map(|f| if f == OPEN { "along a border".to_string() } else { format!("{f}{}", if areas[f as usize].is_none() { " as mesh" } else { "" }) });
                                    println!(
                                        "        nearest edge {side} ({kind}, {} points, {:.3e} mm away), between {} and {}",
                                        b.edges[side].points.len(),
                                        to_edge(side).sqrt(),
                                        whose[0],
                                        whose[1]
                                    );
                                }
                            }
                        }
                        // A FACE THAT TOOK A WRONG PIECE OF ITS SURFACE: its area against its region's triangles
                        let tri_area = |t: u32| {
                            let [a, q, c] = p.mesh.triangle(t as usize);
                            let (u, v) = ([q.x - a.x, q.y - a.y, q.z - a.z], [c.x - a.x, c.y - a.y, c.z - a.z]);
                            let n = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]];
                            (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt() / 2.0
                        };
                        let kind = |s: &Option<Surface>| match s {
                            Some(Surface::Plane { .. }) => "plane",
                            Some(Surface::Cylinder { .. }) => "cylinder",
                            Some(Surface::Cone { .. }) => "cone",
                            Some(Surface::Sphere { .. }) => "sphere",
                            Some(Surface::Torus { .. }) => "torus",
                            Some(Surface::Free { .. } | Surface::Spline(_)) => "free",
                            Some(Surface::Helix { .. }) => "helix",
                            Some(Surface::Coil { .. }) => "coil",
                            Some(Surface::RoundHelix { .. }) => "round helix",
                            None => "none",
                        };
                        let mut wrong: Vec<(f64, usize, f64)> = Vec::new();
                        let mut tally: std::collections::BTreeMap<(&str, bool), usize> = std::collections::BTreeMap::new();
                        let mut lost: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
                        for (r, region) in found.iter().enumerate() {
                            let Some(face) = areas[r] else {
                                if region.surface.is_some() {
                                    *lost.entry(kind(&region.surface)).or_default() += 1;
                                }
                                continue;
                            };
                            let own: f64 = region.tris.iter().map(|&t| tri_area(t)).sum();
                            let ratio = face / own.max(1e-300);
                            if !(0.5..=2.0).contains(&ratio) {
                                let pointed = b.loops[r].iter().flatten().any(|&(e, _)| matches!(found_curves[e], Curve::Points(_)));
                                *tally.entry((kind(&region.surface), pointed)).or_default() += 1;
                                wrong.push((ratio, r, own));
                            }
                        }
                        wrong.sort_by(|x, y| y.0.ln().abs().total_cmp(&x.0.ln().abs()));
                        println!("    faces not built, by kind: {lost:?}; off their region's area (outside 0.5..2): {} - by kind and an edge of points: {tally:?}", wrong.len());
                        for &(ratio, r, own) in wrong.iter().take(5) {
                            println!("      region {r}: {} triangles, face {:.3e} of its {own:.3e} (x{ratio:.2e}), {:?}", found[r].tris.len(), areas[r].unwrap_or(0.0), found[r].surface);
                        }
                    }
                    None => println!("    body: none - {:?}, {:.1} ms", qymcad_kernel::last_kernel_refusal(), sewn.elapsed().as_secs_f64() * 1e3),
                }
            }
            let _ = prepared_ms;
            let file = f.file_name().unwrap_or_default().to_string_lossy().to_string();
            println!(
                "{file} [{name}]: {} tris, corners {} -> {}, dropped {}, turned {}, holes {} (sides {}), shared by 3+ {}, {:.1} ms",
                mesh.tris.len(),
                mesh.verts.len(),
                p.mesh.verts.len(),
                p.dropped,
                p.flipped,
                p.holes.len(),
                p.holes.iter().map(Vec::len).sum::<usize>(),
                p.non_manifold,
                started.elapsed().as_secs_f64() * 1e3
            );
        }
    }
}

/// THE OWNER'S HEAD, PART BY PART, against its STEP: `QYM_CONDOR` names the STEP, `QYM_CONDOR_MESH` a mesh file of the
/// same head split into parts (3MF, glTF, OBJ). The head of the mesh file is shifted onto the STEP's by their boxes -
/// a slicer's project stands it on its plate - and every piece is matched to the occurrence of a part whose box it
/// shares; its regions are counted against that part's faces. `QYM_TOL_FACTOR` widens the distance a corner may lie from its surface, to see how the
/// split answers the tolerance. Printed, not judged: the regions against the part's faces, before any face is built.
#[test]
#[ignore = "the owner's files"]
fn the_owners_head_is_measured_against_its_step() {
    let (Ok(step), Ok(file)) = (std::env::var("QYM_CONDOR"), std::env::var("QYM_CONDOR_MESH")) else { return };
    let qymcad_kernel::ExactTree { bodies, nodes, .. } = qymcad_kernel::read_exact_tree(qymcad_kernel::ExactFormat::Step, &step, 0.1).expect("the STEP reads");
    let then = |a: &[f64; 12], b: &[f64; 12]| -> [f64; 12] {
        let mut m = [0.0; 12];
        for r in 0..3 {
            for c in 0..4 {
                m[r * 4 + c] = (0..3).map(|k| a[r * 4 + k] * b[k * 4 + c]).sum::<f64>() + if c == 3 { a[r * 4 + 3] } else { 0.0 };
            }
        }
        m
    };
    let world = |mut i: usize| {
        let mut m = nodes[i].place;
        while let Some(p) = nodes[i].parent {
            m = then(&nodes[p].place, &m);
            i = p;
        }
        m
    };
    let boxed = |m: &Mesh| m.bounds().map(|b| [b.min.x, b.min.y, b.min.z, b.max.x, b.max.y, b.max.z]);
    // every occurrence of a part: its box in the world, the faces of its body, its name
    let parts: Vec<([f64; 6], usize, String)> = nodes
        .iter()
        .enumerate()
        .filter_map(|(i, n)| {
            let qymcad_core::geom::Built { mesh, faces } = &bodies[n.solid.or(n.repeat_of)?];
            let mut m = mesh.clone();
            m.transform(&world(i));
            Some((boxed(&m)?, faces.len(), n.name.clone()))
        })
        .collect();
    let pieces = match file.rsplit('.').next().unwrap_or("").to_lowercase().as_str() {
        "3mf" => qymcad_io::import_3mf(&file),
        "glb" | "gltf" => qymcad_io::import_gltf(&file),
        "obj" => qymcad_io::import_obj(&file),
        _ => Err("not a file of parts".into()),
    }
    .expect("the mesh file reads");
    let factor: f64 = std::env::var("QYM_TOL_FACTOR").ok().and_then(|s| s.parse().ok()).unwrap_or(1.0);
    // every piece where it stands, and the shift that lays the mesh file's head onto the STEP's
    let boxes: Vec<Option<[f64; 6]>> = pieces
        .iter()
        .map(|piece| {
            let mut placed = piece.mesh.clone();
            placed.transform(&piece.place);
            for place in piece.within.iter().rev().map(|g| &g.place) {
                placed.transform(place);
            }
            boxed(&placed)
        })
        .collect();
    let whole = |bs: &mut dyn Iterator<Item = [f64; 6]>| {
        bs.fold([f64::MAX, f64::MAX, f64::MAX, f64::MIN, f64::MIN, f64::MIN], |a, b| [a[0].min(b[0]), a[1].min(b[1]), a[2].min(b[2]), a[3].max(b[3]), a[4].max(b[4]), a[5].max(b[5])])
    };
    let (head_step, head_mesh) = (whole(&mut parts.iter().map(|p| p.0)), whole(&mut boxes.iter().flatten().copied()));
    let shift = [0, 1, 2].map(|k| head_step[k] - head_mesh[k]);
    println!("the STEP's head {:?} mm across, the mesh file's {:?}; shifted by {shift:?}", [0, 1, 2].map(|k| head_step[k + 3] - head_step[k]), [0, 1, 2].map(|k| head_mesh[k + 3] - head_mesh[k]));
    let mut rows = Vec::new();
    for (piece, b) in pieces.iter().zip(&boxes) {
        let Some(b) = b else { continue };
        let b: [f64; 6] = std::array::from_fn(|k| b[k] + shift[k % 3]);
        let (off, faces, part) = parts.iter().map(|(pb, f, n)| (pb.iter().zip(&b).map(|(x, y)| (x - y).abs()).fold(0.0, f64::max), *f, n.clone())).min_by(|x, y| x.0.total_cmp(&y.0)).expect("parts");
        let p = prepare(&piece.mesh, weld_tolerance(&piece.mesh));
        let mut tol = Tolerance::for_mesh(&p);
        tol.distance *= factor;
        let found = regions(&p, &tol);
        let single = found.iter().filter(|r| r.tris.len() == 1).count();
        let mut kinds = [0usize; 8];
        for r in &found {
            kinds[match r.surface {
                Some(Surface::Plane { .. }) => 0,
                Some(Surface::Cylinder { .. }) => 1,
                Some(Surface::Cone { .. }) => 2,
                Some(Surface::Sphere { .. }) => 3,
                Some(Surface::Torus { .. }) => 4,
                None => 5,
                Some(Surface::Free { .. } | Surface::Spline(_)) => 6,
                Some(Surface::Helix { .. } | Surface::RoundHelix { .. } | Surface::Coil { .. }) => 7,
            }] += 1;
        }
        rows.push((p.mesh.tris.len(), part, faces, found.len(), single, kinds, off));
    }
    rows.sort_by_key(|a| std::cmp::Reverse(a.0));
    println!("{:>7} {:>6} {:>7} {:>7} {:>6}  {:<34} kinds p/c/k/s/t/none, box off", "tris", "faces", "regions", "x faces", "ones", "part");
    for (tris, part, faces, found, single, k, off) in &rows {
        println!(
            "{tris:>7} {faces:>6} {found:>7} {:>7.1} {single:>6}  {:<34} {}/{}/{}/{}/{}/{}, {off:.3}",
            *found as f64 / (*faces).max(1) as f64,
            part.chars().take(34).collect::<String>(),
            k[0],
            k[1],
            k[2],
            k[3],
            k[4],
            k[5]
        );
    }
    let tris: usize = rows.iter().map(|r| r.0).sum();
    let exact = rows.iter().filter(|r| r.3 == r.2).count();
    let near = rows.iter().filter(|r| (r.3 as f64 - r.2 as f64).abs() <= 0.2 * r.2 as f64).count();
    let (faces, found, ones): (usize, usize, usize) = rows.iter().fold((0, 0, 0), |a, r| (a.0 + r.2, a.1 + r.3, a.2 + r.4));
    let worst = rows.iter().map(|r| r.6).fold(0.0, f64::max);
    println!("distance tolerance x{factor}: {} pieces of {tris} triangles (every occurrence in the STEP: {} faces); matched parts {faces} faces, {found} regions ({:.1} x), {ones} of one triangle; as many regions as faces in {exact}, within a fifth in {near}; boxes off by up to {worst:.3} mm", rows.len(), parts.iter().map(|p| p.1).sum::<usize>(), found as f64 / faces.max(1) as f64);
}

/// THE WORST FACE OF ONE MESH, SHOWN: `QYM_SAMPLES` names the folder, `QYM_SAMPLE` the file and `QYM_PIECE` the piece; the
/// region whose built face is farthest off its triangles' area is printed with its triangles, its loops - every edge's
/// faces, corners, curve and points - and its neighbours. Printed, not judged: it is where a probe starts from.
#[test]
#[ignore = "reads a file under QYM_SAMPLES"]
fn the_worst_face_of_a_mesh_is_shown() {
    let (Ok(dir), Ok(file)) = (std::env::var("QYM_SAMPLES"), std::env::var("QYM_SAMPLE")) else { return };
    let piece = std::env::var("QYM_PIECE").unwrap_or_default();
    let path = std::path::Path::new(&dir).join(&file);
    for (name, mesh) in meshes(&path) {
        if !name.contains(piece.as_str()) {
            continue;
        }
        let p = prepare(&mesh, weld_tolerance(&mesh));
        let tol = Tolerance::for_mesh(&p);
        let found = regions(&p, &tol);
        let b = boundaries(&p, &found);
        let c = curves(&b, &found, &tol);
        let Some(areas) = solid(&p, &found, &b, &c, &tol).map(|made| made.areas) else { continue };
        let tri_area = |t: u32| {
            let [a, q, r] = p.mesh.triangle(t as usize);
            let (u, v) = ([q.x - a.x, q.y - a.y, q.z - a.z], [r.x - a.x, r.y - a.y, r.z - a.z]);
            let n = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]];
            (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt() / 2.0
        };
        let own = |r: usize| found[r].tris.iter().map(|&t| tri_area(t)).sum::<f64>();
        let Some(worst) = (0..found.len()).filter(|&r| areas[r].is_some()).max_by(|&x, &y| (areas[x].unwrap_or(0.0) / own(x)).ln().abs().total_cmp(&(areas[y].unwrap_or(0.0) / own(y)).ln().abs()))
        else {
            continue;
        };
        println!(
            "{name}: tolerance {:.3e}; region {worst}: {} triangles of {:.3e} mm^2, face {:.3e}, {:?}",
            tol.distance,
            found[worst].tris.len(),
            own(worst),
            areas[worst].unwrap_or(0.0),
            found[worst].surface
        );
        for &t in &found[worst].tris {
            println!("  triangle {t}: {:?}", p.mesh.triangle(t as usize).map(|q| [q.x, q.y, q.z]));
        }
        for (k, l) in b.loops[worst].iter().enumerate() {
            println!("  loop {k}:");
            for &(e, forward) in l {
                let edge = &b.edges[e];
                let other = if edge.faces[0] as usize == worst { edge.faces[1] } else { edge.faces[0] };
                println!(
                    "    edge {e} ({}) to region {other}: corners {:?}, {} points {:?} .. {:?}, curve {:?}",
                    if forward { "along" } else { "back" },
                    edge.ends.map(|[a, z]| [b.vertices[a], b.vertices[z]]),
                    edge.points.len(),
                    edge.points.first(),
                    edge.points.last(),
                    match &c[e] {
                        Curve::Points(pts) => format!("points x{}", pts.len()),
                        other => format!("{other:?}"),
                    }
                );
                if (other as usize) < found.len() {
                    println!("      region {other}: {} triangles, {:?}, face {:?}", found[other as usize].tris.len(), found[other as usize].surface, areas[other as usize]);
                }
            }
        }
    }
}
