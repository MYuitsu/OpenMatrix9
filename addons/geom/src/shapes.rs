// SPDX-License-Identifier: LGPL-2.1-or-later
//! Planar polygons and solids built from them: ear-clipping triangulation, prisms from closed
//! curves (with holes), capping open mesh boundaries, and a simple stroke font for text.
use crate::mesh::{Frame, Mesh};
use om9_addon_abi::session::{Vec3, add, cross, dot, norm, scale, sub, unit};

/// Signed area of a 2D polygon (counter-clockwise > 0).
pub fn area2(p: &[[f64; 2]]) -> f64 {
    let n = p.len();
    (0..n)
        .map(|i| {
            let (a, b) = (p[i], p[(i + 1) % n]);
            a[0] * b[1] - b[0] * a[1]
        })
        .sum::<f64>()
        / 2.0
}

fn inside_tri(p: [f64; 2], a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> bool {
    let s = |p: [f64; 2], q: [f64; 2], r: [f64; 2]| {
        (q[0] - p[0]) * (r[1] - p[1]) - (q[1] - p[1]) * (r[0] - p[0])
    };
    let (d1, d2, d3) = (s(a, b, p), s(b, c, p), s(c, a, p));
    !(d1 < -1e-12 || d2 < -1e-12 || d3 < -1e-12)
}

/// Ear-clipping triangulation of a simple polygon (any orientation); triangles are returned
/// counter-clockwise as indices into `poly`.
pub fn triangulate(poly: &[[f64; 2]]) -> Vec<[usize; 3]> {
    let n = poly.len();
    if n < 3 {
        return Vec::new();
    }
    let mut idx: Vec<usize> = (0..n).collect();
    if area2(poly) < 0.0 {
        idx.reverse();
    }
    let mut out = Vec::new();
    let mut guard = 0;
    while idx.len() > 3 && guard < 10 * n * n {
        guard += 1;
        let m = idx.len();
        let mut clipped = false;
        for i in 0..m {
            let (ia, ib, ic) = (idx[(i + m - 1) % m], idx[i], idx[(i + 1) % m]);
            let (a, b, c) = (poly[ia], poly[ib], poly[ic]);
            let convex = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]) > 1e-14;
            if !convex {
                continue;
            }
            // Points at a corner of the ear (the doubled ends of a hole's bridge) do not
            // block it; anything else on or inside the ear does.
            let same = |p: [f64; 2], q: [f64; 2]| {
                (p[0] - q[0]).abs() < 1e-12 && (p[1] - q[1]).abs() < 1e-12
            };
            let blocked = idx.iter().any(|&j| {
                j != ia
                    && j != ib
                    && j != ic
                    && !same(poly[j], a)
                    && !same(poly[j], b)
                    && !same(poly[j], c)
                    && inside_tri(poly[j], a, b, c)
            });
            if blocked {
                continue;
            }
            out.push([ia, ib, ic]);
            idx.remove(i);
            clipped = true;
            break;
        }
        if !clipped {
            // Degenerate rest (collinear points): fan it.
            for i in 1..idx.len() - 1 {
                out.push([idx[0], idx[i], idx[i + 1]]);
            }
            return out;
        }
    }
    if idx.len() == 3 {
        out.push([idx[0], idx[1], idx[2]]);
    }
    out
}

/// Join an outer polygon and its holes into one simple polygon by bridges from each hole's
/// rightmost point to the nearest visible outer point (the classic hole-bridging).
pub fn bridge_holes(outer: &[[f64; 2]], holes: &[Vec<[f64; 2]>]) -> Vec<[f64; 2]> {
    let mut poly: Vec<[f64; 2]> = if area2(outer) < 0.0 {
        outer.iter().rev().copied().collect()
    } else {
        outer.to_vec()
    };
    let mut hs: Vec<Vec<[f64; 2]>> = holes
        .iter()
        .map(|h| {
            if area2(h) > 0.0 {
                h.iter().rev().copied().collect()
            } else {
                h.clone()
            }
        })
        .collect();
    // Rightmost holes first.
    hs.sort_by(|a, b| {
        let ra = a.iter().map(|p| p[0]).fold(f64::MIN, f64::max);
        let rb = b.iter().map(|p| p[0]).fold(f64::MIN, f64::max);
        rb.total_cmp(&ra)
    });
    for h in hs {
        let (hi, hp) = h
            .iter()
            .enumerate()
            .max_by(|a, b| a.1[0].total_cmp(&b.1[0]))
            .map(|(i, p)| (i, *p))
            .unwrap();
        let (oi, _) = poly
            .iter()
            .enumerate()
            .filter(|(_, p)| p[0] >= hp[0])
            .min_by(|a, b| {
                let d = |p: &[f64; 2]| (p[0] - hp[0]).powi(2) + (p[1] - hp[1]).powi(2);
                d(a.1).total_cmp(&d(b.1))
            })
            .or_else(|| {
                poly.iter().enumerate().min_by(|a, b| {
                    let d = |p: &[f64; 2]| (p[0] - hp[0]).powi(2) + (p[1] - hp[1]).powi(2);
                    d(a.1).total_cmp(&d(b.1))
                })
            })
            .unwrap();
        let mut merged = poly[..=oi].to_vec();
        for k in 0..=h.len() {
            merged.push(h[(hi + k) % h.len()]);
        }
        merged.push(poly[oi]);
        merged.extend_from_slice(&poly[oi + 1..]);
        poly = merged;
    }
    poly
}

/// The best plane of a loop of points: centroid and Newell normal.
pub fn loop_plane(points: &[Vec3]) -> Option<(Vec3, Vec3)> {
    let n = points.len();
    if n < 3 {
        return None;
    }
    let c = scale(
        points.iter().fold([0.0; 3], |a, &p| add(a, p)),
        1.0 / n as f64,
    );
    let mut normal = [0.0; 3];
    for i in 0..n {
        let (a, b) = (points[i], points[(i + 1) % n]);
        normal[0] += (a[1] - b[1]) * (a[2] + b[2]);
        normal[1] += (a[2] - b[2]) * (a[0] + b[0]);
        normal[2] += (a[0] - b[0]) * (a[1] + b[1]);
    }
    (norm(normal) > 1e-14).then(|| (c, unit(normal)))
}

/// A closed prism from a planar outline (world points, with optional holes) swept along
/// `dir`. Faces point outward.
pub fn prism(outline: &[Vec3], holes: &[Vec<Vec3>], dir: Vec3) -> Option<Mesh> {
    let (c, mut n) = loop_plane(outline)?;
    if dot(n, dir) < 0.0 {
        n = scale(n, -1.0);
    }
    let f = Frame::new(c, n, sub(outline[0], c));
    let flat = |pts: &[Vec3]| -> Vec<[f64; 2]> {
        pts.iter()
            .map(|&p| {
                let l = f.local(p);
                [l[0], l[1]]
            })
            .collect()
    };
    let outer2 = flat(outline);
    let holes2: Vec<Vec<[f64; 2]>> = holes.iter().map(|h| flat(h)).collect();
    let cap = bridge_holes(&outer2, &holes2);
    let tris = triangulate(&cap);
    let mut m = Mesh::default();
    let base: Vec<Vec3> = cap.iter().map(|q| f.point([q[0], q[1], 0.0])).collect();
    let k = cap.len() as u32;
    m.vertices.extend(base.iter().copied());
    m.vertices.extend(base.iter().map(|&p| add(p, dir)));
    for t in &tris {
        // bottom faces down (reversed), top faces up
        m.triangles.push([t[0] as u32, t[2] as u32, t[1] as u32]);
        m.triangles
            .push([t[0] as u32 + k, t[1] as u32 + k, t[2] as u32 + k]);
    }
    // Side walls along every loop (outer counter-clockwise, holes clockwise in `cap` order).
    let walls = |pts: &[[f64; 2]], m: &mut Mesh| {
        let s = m.vertices.len() as u32;
        let ring: Vec<Vec3> = pts.iter().map(|q| f.point([q[0], q[1], 0.0])).collect();
        let nn = ring.len() as u32;
        m.vertices.extend(ring.iter().copied());
        m.vertices.extend(ring.iter().map(|&p| add(p, dir)));
        for i in 0..nn {
            let j = (i + 1) % nn;
            let (a, b, c2, d) = (s + i, s + j, s + j + nn, s + i + nn);
            // Material is on the left of every loop (outer counter-clockwise, holes clockwise).
            m.triangles.push([a, b, c2]);
            m.triangles.push([a, c2, d]);
        }
    };
    let outer_ccw: Vec<[f64; 2]> = if area2(&outer2) < 0.0 {
        outer2.iter().rev().copied().collect()
    } else {
        outer2
    };
    walls(&outer_ccw, &mut m);
    for h in &holes2 {
        let cw: Vec<[f64; 2]> = if area2(h) > 0.0 {
            h.iter().rev().copied().collect()
        } else {
            h.clone()
        };
        walls(&cw, &mut m);
    }
    let mut m = weld(&m, 1e-9);
    if m.volume() < 0.0 {
        for t in &mut m.triangles {
            t.swap(1, 2);
        }
    }
    Some(m)
}

/// Merge vertices closer than `tol` and drop degenerate triangles.
pub fn weld(m: &Mesh, tol: f64) -> Mesh {
    let mut out = Mesh::default();
    let mut map = std::collections::HashMap::<[i64; 3], u32>::new();
    let inv = 1.0 / tol.max(1e-12);
    let ids: Vec<u32> = m
        .vertices
        .iter()
        .map(|&p| {
            let k = [
                (p[0] * inv).round() as i64,
                (p[1] * inv).round() as i64,
                (p[2] * inv).round() as i64,
            ];
            *map.entry(k).or_insert_with(|| {
                out.vertices.push(p);
                (out.vertices.len() - 1) as u32
            })
        })
        .collect();
    for t in &m.triangles {
        let t = [ids[t[0] as usize], ids[t[1] as usize], ids[t[2] as usize]];
        if t[0] != t[1] && t[1] != t[2] && t[0] != t[2] {
            out.triangles.push(t);
        }
    }
    out
}

/// Boundary loops of a mesh (edges used once), as vertex index loops.
pub fn boundary_loops(m: &Mesh) -> Vec<Vec<u32>> {
    let mut count = std::collections::HashMap::<(u32, u32), i32>::new();
    for t in &m.triangles {
        for k in 0..3 {
            let (a, b) = (t[k], t[(k + 1) % 3]);
            *count.entry((a.min(b), a.max(b))).or_insert(0) += 1;
        }
    }
    // Directed boundary edges, oriented as in their triangle.
    let mut next = std::collections::HashMap::<u32, u32>::new();
    for t in &m.triangles {
        for k in 0..3 {
            let (a, b) = (t[k], t[(k + 1) % 3]);
            if count[&(a.min(b), a.max(b))] == 1 {
                next.insert(a, b);
            }
        }
    }
    let mut loops = Vec::new();
    let mut used = std::collections::HashSet::new();
    let mut starts: Vec<u32> = next.keys().copied().collect();
    starts.sort_unstable();
    for s in starts {
        if used.contains(&s) {
            continue;
        }
        let mut l = vec![s];
        used.insert(s);
        let mut cur = s;
        while let Some(&n) = next.get(&cur) {
            if n == s || used.contains(&n) {
                break;
            }
            l.push(n);
            used.insert(n);
            cur = n;
        }
        if l.len() >= 3 {
            loops.push(l);
        }
    }
    loops
}

/// Close every boundary loop with a planar-ish cap (Cap Planar Holes, Fill Hole).
pub fn cap_holes(m: &Mesh) -> (Mesh, usize) {
    let mut out = m.clone();
    let loops = boundary_loops(m);
    for l in &loops {
        let pts: Vec<Vec3> = l.iter().map(|&i| m.vertices[i as usize]).collect();
        let Some((c, n)) = loop_plane(&pts) else {
            continue;
        };
        let f = Frame::new(c, n, sub(pts[0], c));
        let flat: Vec<[f64; 2]> = pts
            .iter()
            .map(|&p| {
                let q = f.local(p);
                [q[0], q[1]]
            })
            .collect();
        // The loop runs along the boundary as its triangles see it; the cap runs the other way.
        let ccw = area2(&flat) > 0.0;
        for t in triangulate(&flat) {
            let tri = [l[t[0]], l[t[1]], l[t[2]]];
            out.triangles
                .push(if ccw { [tri[0], tri[2], tri[1]] } else { tri });
        }
    }
    (out, loops.len())
}

/// Strokes of a small block font (capitals, digits, a few signs) in a 0..1 × 0..1 cell,
/// for text that is engraved, embossed or set along a curve.
pub fn glyph(c: char) -> Vec<Vec<[f64; 2]>> {
    let s = |pts: &[(f64, f64)]| pts.iter().map(|&(x, y)| [x, y]).collect::<Vec<_>>();
    match c.to_ascii_uppercase() {
        'A' => vec![
            s(&[(0.0, 0.0), (0.5, 1.0), (1.0, 0.0)]),
            s(&[(0.25, 0.5), (0.75, 0.5)]),
        ],
        'B' => vec![
            s(&[
                (0.0, 0.0),
                (0.0, 1.0),
                (0.7, 1.0),
                (0.85, 0.85),
                (0.85, 0.65),
                (0.7, 0.5),
                (0.0, 0.5),
            ]),
            s(&[
                (0.7, 0.5),
                (0.9, 0.35),
                (0.9, 0.15),
                (0.75, 0.0),
                (0.0, 0.0),
            ]),
        ],
        'C' => vec![s(&[
            (1.0, 0.85),
            (0.8, 1.0),
            (0.2, 1.0),
            (0.0, 0.8),
            (0.0, 0.2),
            (0.2, 0.0),
            (0.8, 0.0),
            (1.0, 0.15),
        ])],
        'D' => vec![s(&[
            (0.0, 0.0),
            (0.0, 1.0),
            (0.6, 1.0),
            (1.0, 0.7),
            (1.0, 0.3),
            (0.6, 0.0),
            (0.0, 0.0),
        ])],
        'E' => vec![
            s(&[(1.0, 1.0), (0.0, 1.0), (0.0, 0.0), (1.0, 0.0)]),
            s(&[(0.0, 0.5), (0.7, 0.5)]),
        ],
        'F' => vec![
            s(&[(1.0, 1.0), (0.0, 1.0), (0.0, 0.0)]),
            s(&[(0.0, 0.5), (0.7, 0.5)]),
        ],
        'G' => vec![s(&[
            (1.0, 0.85),
            (0.8, 1.0),
            (0.2, 1.0),
            (0.0, 0.8),
            (0.0, 0.2),
            (0.2, 0.0),
            (0.8, 0.0),
            (1.0, 0.2),
            (1.0, 0.45),
            (0.55, 0.45),
        ])],
        'H' => vec![
            s(&[(0.0, 0.0), (0.0, 1.0)]),
            s(&[(1.0, 0.0), (1.0, 1.0)]),
            s(&[(0.0, 0.5), (1.0, 0.5)]),
        ],
        'I' => vec![
            s(&[(0.5, 0.0), (0.5, 1.0)]),
            s(&[(0.2, 1.0), (0.8, 1.0)]),
            s(&[(0.2, 0.0), (0.8, 0.0)]),
        ],
        'J' => vec![s(&[
            (1.0, 1.0),
            (1.0, 0.2),
            (0.8, 0.0),
            (0.2, 0.0),
            (0.0, 0.2),
        ])],
        'K' => vec![
            s(&[(0.0, 0.0), (0.0, 1.0)]),
            s(&[(1.0, 1.0), (0.0, 0.45), (1.0, 0.0)]),
        ],
        'L' => vec![s(&[(0.0, 1.0), (0.0, 0.0), (1.0, 0.0)])],
        'M' => vec![s(&[
            (0.0, 0.0),
            (0.0, 1.0),
            (0.5, 0.45),
            (1.0, 1.0),
            (1.0, 0.0),
        ])],
        'N' => vec![s(&[(0.0, 0.0), (0.0, 1.0), (1.0, 0.0), (1.0, 1.0)])],
        'O' | '0' => vec![s(&[
            (0.2, 0.0),
            (0.8, 0.0),
            (1.0, 0.2),
            (1.0, 0.8),
            (0.8, 1.0),
            (0.2, 1.0),
            (0.0, 0.8),
            (0.0, 0.2),
            (0.2, 0.0),
        ])],
        'P' => vec![s(&[
            (0.0, 0.0),
            (0.0, 1.0),
            (0.8, 1.0),
            (1.0, 0.8),
            (1.0, 0.6),
            (0.8, 0.45),
            (0.0, 0.45),
        ])],
        'Q' => vec![
            s(&[
                (0.2, 0.0),
                (0.8, 0.0),
                (1.0, 0.2),
                (1.0, 0.8),
                (0.8, 1.0),
                (0.2, 1.0),
                (0.0, 0.8),
                (0.0, 0.2),
                (0.2, 0.0),
            ]),
            s(&[(0.6, 0.3), (1.0, -0.1)]),
        ],
        'R' => vec![
            s(&[
                (0.0, 0.0),
                (0.0, 1.0),
                (0.8, 1.0),
                (1.0, 0.8),
                (1.0, 0.6),
                (0.8, 0.45),
                (0.0, 0.45),
            ]),
            s(&[(0.4, 0.45), (1.0, 0.0)]),
        ],
        'S' => vec![s(&[
            (1.0, 0.85),
            (0.8, 1.0),
            (0.2, 1.0),
            (0.0, 0.8),
            (0.0, 0.62),
            (0.2, 0.5),
            (0.8, 0.5),
            (1.0, 0.38),
            (1.0, 0.2),
            (0.8, 0.0),
            (0.2, 0.0),
            (0.0, 0.15),
        ])],
        'T' => vec![s(&[(0.0, 1.0), (1.0, 1.0)]), s(&[(0.5, 1.0), (0.5, 0.0)])],
        'U' => vec![s(&[
            (0.0, 1.0),
            (0.0, 0.2),
            (0.2, 0.0),
            (0.8, 0.0),
            (1.0, 0.2),
            (1.0, 1.0),
        ])],
        'V' => vec![s(&[(0.0, 1.0), (0.5, 0.0), (1.0, 1.0)])],
        'W' => vec![s(&[
            (0.0, 1.0),
            (0.25, 0.0),
            (0.5, 0.6),
            (0.75, 0.0),
            (1.0, 1.0),
        ])],
        'X' => vec![s(&[(0.0, 0.0), (1.0, 1.0)]), s(&[(0.0, 1.0), (1.0, 0.0)])],
        'Y' => vec![
            s(&[(0.0, 1.0), (0.5, 0.5), (1.0, 1.0)]),
            s(&[(0.5, 0.5), (0.5, 0.0)]),
        ],
        'Z' => vec![s(&[(0.0, 1.0), (1.0, 1.0), (0.0, 0.0), (1.0, 0.0)])],
        '1' => vec![
            s(&[(0.25, 0.75), (0.55, 1.0), (0.55, 0.0)]),
            s(&[(0.25, 0.0), (0.85, 0.0)]),
        ],
        '2' => vec![s(&[
            (0.0, 0.8),
            (0.2, 1.0),
            (0.8, 1.0),
            (1.0, 0.8),
            (1.0, 0.6),
            (0.0, 0.0),
            (1.0, 0.0),
        ])],
        '3' => vec![
            s(&[
                (0.0, 0.85),
                (0.2, 1.0),
                (0.8, 1.0),
                (1.0, 0.8),
                (1.0, 0.65),
                (0.7, 0.5),
                (1.0, 0.35),
                (1.0, 0.2),
                (0.8, 0.0),
                (0.2, 0.0),
                (0.0, 0.15),
            ]),
            s(&[(0.35, 0.5), (0.7, 0.5)]),
        ],
        '4' => vec![s(&[(0.75, 0.0), (0.75, 1.0), (0.0, 0.3), (1.0, 0.3)])],
        '5' => vec![s(&[
            (1.0, 1.0),
            (0.05, 1.0),
            (0.0, 0.55),
            (0.75, 0.6),
            (1.0, 0.4),
            (1.0, 0.2),
            (0.8, 0.0),
            (0.2, 0.0),
            (0.0, 0.15),
        ])],
        '6' => vec![s(&[
            (0.9, 1.0),
            (0.3, 1.0),
            (0.0, 0.7),
            (0.0, 0.2),
            (0.2, 0.0),
            (0.8, 0.0),
            (1.0, 0.2),
            (1.0, 0.4),
            (0.8, 0.55),
            (0.0, 0.5),
        ])],
        '7' => vec![s(&[(0.0, 1.0), (1.0, 1.0), (0.4, 0.0)])],
        '8' => vec![s(&[
            (0.2, 0.5),
            (0.0, 0.7),
            (0.0, 0.85),
            (0.2, 1.0),
            (0.8, 1.0),
            (1.0, 0.85),
            (1.0, 0.7),
            (0.8, 0.5),
            (0.2, 0.5),
            (0.0, 0.3),
            (0.0, 0.15),
            (0.2, 0.0),
            (0.8, 0.0),
            (1.0, 0.15),
            (1.0, 0.3),
            (0.8, 0.5),
        ])],
        '9' => vec![s(&[
            (1.0, 0.5),
            (0.2, 0.45),
            (0.0, 0.6),
            (0.0, 0.8),
            (0.2, 1.0),
            (0.8, 1.0),
            (1.0, 0.8),
            (1.0, 0.3),
            (0.7, 0.0),
            (0.1, 0.0),
        ])],
        '-' => vec![s(&[(0.2, 0.5), (0.8, 0.5)])],
        '.' => vec![s(&[
            (0.45, 0.0),
            (0.55, 0.0),
            (0.55, 0.1),
            (0.45, 0.1),
            (0.45, 0.0),
        ])],
        '&' => vec![s(&[
            (1.0, 0.0),
            (0.2, 0.75),
            (0.35, 1.0),
            (0.6, 0.9),
            (0.6, 0.7),
            (0.0, 0.3),
            (0.1, 0.0),
            (0.6, 0.0),
            (1.0, 0.45),
        ])],
        '♥' => vec![s(&[
            (0.5, 0.0),
            (0.0, 0.55),
            (0.1, 0.9),
            (0.3, 1.0),
            (0.5, 0.8),
            (0.7, 1.0),
            (0.9, 0.9),
            (1.0, 0.55),
            (0.5, 0.0),
        ])],
        _ => Vec::new(),
    }
}

/// Text as stroke polylines in a frame: letters `height` tall, `spacing` between letters
/// (fraction of the height), laid along +x from the frame origin.
pub fn text_strokes(text: &str, frame: &Frame, height: f64, spacing: f64) -> Vec<Vec<Vec3>> {
    let mut out = Vec::new();
    let w = height * 0.65;
    let mut x = 0.0;
    for ch in text.chars() {
        if ch == ' ' {
            x += w + height * spacing;
            continue;
        }
        for stroke in glyph(ch) {
            out.push(
                stroke
                    .iter()
                    .map(|p| frame.point([x + p[0] * w, p[1] * height, 0.0]))
                    .collect(),
            );
        }
        x += w + height * spacing;
    }
    out
}

/// Width of a text line from [`text_strokes`].
pub fn text_width(text: &str, height: f64, spacing: f64) -> f64 {
    let n = text.chars().count() as f64;
    (n * (height * 0.65 + height * spacing) - height * spacing).max(0.0)
}

/// Unit normal of a triangle.
pub fn tri_normal(a: Vec3, b: Vec3, c: Vec3) -> Vec3 {
    unit(cross(sub(b, a), sub(c, a)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triangulates_concave_polygons_and_holes() {
        let l = [
            [0.0, 0.0],
            [2.0, 0.0],
            [2.0, 1.0],
            [1.0, 1.0],
            [1.0, 2.0],
            [0.0, 2.0],
        ];
        let t = triangulate(&l);
        assert_eq!(t.len(), 4);
        let a: f64 = t.iter().map(|t| area2(&[l[t[0]], l[t[1]], l[t[2]]])).sum();
        assert!((a - 3.0).abs() < 1e-12);
        let outer = [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]];
        let hole = vec![[1.0, 1.0], [3.0, 1.0], [3.0, 3.0], [1.0, 3.0]];
        let p = bridge_holes(&outer, &[hole]);
        let a: f64 = triangulate(&p)
            .iter()
            .map(|t| area2(&[p[t[0]], p[t[1]], p[t[2]]]))
            .sum();
        assert!((a - 12.0).abs() < 1e-9, "{a}");
    }

    #[test]
    fn prisms_and_caps_are_closed() {
        let sq: Vec<Vec3> = vec![
            [0.0, 0.0, 0.0],
            [4.0, 0.0, 0.0],
            [4.0, 4.0, 0.0],
            [0.0, 4.0, 0.0],
        ];
        let hole: Vec<Vec3> = vec![
            [1.0, 1.0, 0.0],
            [3.0, 1.0, 0.0],
            [3.0, 3.0, 0.0],
            [1.0, 3.0, 0.0],
        ];
        let m = prism(&sq, &[hole], [0.0, 0.0, 2.0]).unwrap();
        assert!(m.is_closed(), "closed prism with a hole");
        assert!((m.volume() - 24.0).abs() < 1e-9, "{}", m.volume());
        // An open box (no top) gets its cap back.
        let mut open = crate::mesh::cuboid([0.0; 3], [1.0, 1.0, 1.0]);
        open.triangles
            .retain(|t| !t.iter().all(|&i| open.vertices[i as usize][2] == 1.0));
        let (capped, n) = cap_holes(&open);
        assert_eq!(n, 1);
        assert!(capped.is_closed() && (capped.volume() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn text_has_strokes() {
        let s = text_strokes("OM9 love", &Frame::WORLD, 2.0, 0.2);
        assert!(s.len() >= 8);
        assert!((text_width("AB", 2.0, 0.2) - (2.0 * 1.3 + 2.0 * 0.4 - 0.4)).abs() < 1e-9);
    }
}

#[cfg(test)]
mod hole_tests {
    use super::*;

    #[test]
    fn bridged_polygon_triangulates_to_the_ring_area() {
        let circ = |r: f64, n: usize| -> Vec<[f64; 2]> {
            (0..n)
                .map(|k| {
                    let a = std::f64::consts::TAU * k as f64 / n as f64;
                    [r * a.cos(), r * a.sin()]
                })
                .collect()
        };
        let outer = circ(5.0, 64);
        let hole = circ(2.0, 32);
        let poly = bridge_holes(&outer, &[hole.clone()]);
        let tris = triangulate(&poly);
        let area: f64 = tris
            .iter()
            .map(|t| {
                let (a, b, c) = (poly[t[0]], poly[t[1]], poly[t[2]]);
                ((b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])).abs() / 2.0
            })
            .sum();
        let want = area2(&outer).abs() - area2(&hole).abs();
        assert!(
            (area - want).abs() < 1e-6,
            "{area} vs {want} ({} points, {} triangles)",
            poly.len(),
            tris.len()
        );
    }
}
