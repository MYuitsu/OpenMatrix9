// SPDX-License-Identifier: LGPL-2.1-or-later
//! Polygon cages for SubD modelling: faces of any size, Catmull–Clark subdivision (with
//! boundaries and creases), topology queries (edges, loops, rings, naked edges), face edits
//! (extrude, inset, delete, split) and conversion to and from triangle meshes.
use crate::mesh::Mesh;
use om9_addon_abi::session::{Vec3, add, cross, dist, dot, norm, scale, sub, unit};
use std::collections::{BTreeMap, BTreeSet, HashMap};

/// A polygon mesh: vertices and faces (counter-clockwise seen from outside).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Poly {
    pub verts: Vec<Vec3>,
    pub faces: Vec<Vec<u32>>,
}

pub type Edge = (u32, u32);

pub fn edge(a: u32, b: u32) -> Edge {
    (a.min(b), a.max(b))
}

impl Poly {
    /// Triangles: quads as [a, b, c], [a, c, d] (so [`Poly::from_mesh`] finds them again),
    /// other faces as fans.
    pub fn to_mesh(&self) -> Mesh {
        let mut tris = Vec::new();
        for f in &self.faces {
            for k in 1..f.len().saturating_sub(1) {
                tris.push([f[0], f[k], f[k + 1]]);
            }
        }
        Mesh {
            vertices: self.verts.clone(),
            triangles: tris,
        }
    }

    /// Faces again from triangles: pairs written by [`Poly::to_mesh`] become quads; other
    /// triangle pairs sharing their longest edge and nearly flat become quads too.
    pub fn from_mesh(m: &Mesh) -> Poly {
        let t = &m.triangles;
        let mut used = vec![false; t.len()];
        let mut faces = Vec::new();
        let mut k = 0;
        // Our own quads: consecutive [a, b, c], [a, c, d].
        while k + 1 < t.len() {
            let (p, q) = (t[k], t[k + 1]);
            if p[0] == q[0] && p[2] == q[1] && !p.contains(&q[2]) {
                faces.push(vec![p[0], p[1], p[2], q[2]]);
                used[k] = true;
                used[k + 1] = true;
                k += 2;
            } else {
                k += 1;
            }
        }
        // Others: pair across the longest edge when flat enough.
        let mut by_edge: HashMap<Edge, Vec<usize>> = HashMap::new();
        for (i, tri) in t.iter().enumerate() {
            if !used[i] {
                for j in 0..3 {
                    by_edge
                        .entry(edge(tri[j], tri[(j + 1) % 3]))
                        .or_default()
                        .push(i);
                }
            }
        }
        let v = |i: u32| m.vertices[i as usize];
        let normal =
            |tri: [u32; 3]| unit(cross(sub(v(tri[1]), v(tri[0])), sub(v(tri[2]), v(tri[0]))));
        for i in 0..t.len() {
            if used[i] {
                continue;
            }
            let tri = t[i];
            // Longest edge.
            let (j, _) = (0..3)
                .map(|j| (j, dist(v(tri[j]), v(tri[(j + 1) % 3]))))
                .max_by(|a, b| a.1.total_cmp(&b.1))
                .unwrap();
            let (a, b) = (tri[j], tri[(j + 1) % 3]);
            let c = tri[(j + 2) % 3];
            let mate = by_edge
                .get(&edge(a, b))
                .and_then(|l| l.iter().copied().find(|&o| o != i && !used[o]));
            if let Some(o) = mate {
                let ot = t[o];
                let d = *ot.iter().find(|&&x| x != a && x != b).unwrap();
                let longest_too = {
                    let l = dist(v(a), v(b));
                    (0..3).all(|q| dist(v(ot[q]), v(ot[(q + 1) % 3])) <= l + 1e-12)
                };
                if longest_too && dot(normal(tri), normal(ot)) > 0.95 {
                    // Quad c → a → d → b keeps the winding of tri (a → b → c).
                    faces.push(vec![b, c, a, d]);
                    used[i] = true;
                    used[o] = true;
                    continue;
                }
            }
            faces.push(tri.to_vec());
            used[i] = true;
        }
        Poly {
            verts: m.vertices.clone(),
            faces,
        }
    }

    /// Edge → faces using it.
    pub fn edge_faces(&self) -> BTreeMap<Edge, Vec<usize>> {
        let mut map: BTreeMap<Edge, Vec<usize>> = BTreeMap::new();
        for (i, f) in self.faces.iter().enumerate() {
            for k in 0..f.len() {
                map.entry(edge(f[k], f[(k + 1) % f.len()]))
                    .or_default()
                    .push(i);
            }
        }
        map
    }

    /// Edges used by one face only (the open border).
    pub fn naked_edges(&self) -> Vec<Edge> {
        self.edge_faces()
            .into_iter()
            .filter(|(_, f)| f.len() == 1)
            .map(|(e, _)| e)
            .collect()
    }

    pub fn face_center(&self, f: usize) -> Vec3 {
        let face = &self.faces[f];
        scale(
            face.iter()
                .fold([0.0; 3], |a, &i| add(a, self.verts[i as usize])),
            1.0 / face.len() as f64,
        )
    }

    pub fn face_normal(&self, f: usize) -> Vec3 {
        let face = &self.faces[f];
        let mut n = [0.0; 3];
        for k in 0..face.len() {
            let (a, b) = (
                self.verts[face[k] as usize],
                self.verts[face[(k + 1) % face.len()] as usize],
            );
            n = add(n, cross(a, b));
        }
        unit(n)
    }

    /// The face whose center is nearest to the line through `p` along `dir` (picking); of
    /// faces as near, the one highest along `dir` (the one the viewer sees).
    pub fn pick_face(&self, p: Vec3, dir: Vec3) -> Option<usize> {
        let d = unit(dir);
        let off = |f: usize| {
            let c = sub(self.face_center(f), p);
            norm(sub(c, scale(d, dot(c, d))))
        };
        let best = (0..self.faces.len()).map(off).fold(f64::MAX, f64::min);
        (0..self.faces.len())
            .filter(|&f| off(f) <= best + 1e-6 * (1.0 + best))
            .max_by(|&a, &b| dot(self.face_center(a), d).total_cmp(&dot(self.face_center(b), d)))
    }

    /// The edge nearest to the line through `p` along `dir`; of edges as near, the highest
    /// along `dir`.
    pub fn pick_edge(&self, p: Vec3, dir: Vec3) -> Option<Edge> {
        let d = unit(dir);
        let mid = |e: Edge| scale(add(self.verts[e.0 as usize], self.verts[e.1 as usize]), 0.5);
        let off = |e: Edge| {
            let c = sub(mid(e), p);
            norm(sub(c, scale(d, dot(c, d))))
        };
        let edges: Vec<Edge> = self.edge_faces().keys().copied().collect();
        let best = edges.iter().map(|&e| off(e)).fold(f64::MAX, f64::min);
        edges
            .into_iter()
            .filter(|&e| off(e) <= best + 1e-6 * (1.0 + best))
            .max_by(|&a, &b| dot(mid(a), d).total_cmp(&dot(mid(b), d)))
    }

    /// Faces across from each other through quads, starting at the faces on `e` (a ring of
    /// faces crossing the edge ring of `e`).
    pub fn face_ring(&self, e: Edge) -> Vec<usize> {
        let ef = self.edge_faces();
        let mut out: Vec<usize> = Vec::new();
        let mut seen = BTreeSet::new();
        for &start in ef.get(&e).into_iter().flatten() {
            let mut f = start;
            let mut through = e;
            loop {
                if !seen.insert(f) {
                    break;
                }
                out.push(f);
                let face = &self.faces[f];
                if face.len() != 4 {
                    break;
                }
                // The opposite edge of the quad.
                let k = (0..4)
                    .find(|&k| edge(face[k], face[(k + 1) % 4]) == through)
                    .unwrap();
                let opp = edge(face[(k + 2) % 4], face[(k + 3) % 4]);
                let Some(next) = ef
                    .get(&opp)
                    .and_then(|l| l.iter().copied().find(|&g| g != f))
                else {
                    break;
                };
                f = next;
                through = opp;
            }
        }
        out
    }

    /// The faces along `e` (a face loop running along the edge's direction).
    pub fn face_loop(&self, e: Edge) -> Vec<usize> {
        // A loop along e is the ring across an edge perpendicular to it on the adjacent quad.
        let ef = self.edge_faces();
        let Some(&f) = ef.get(&e).and_then(|l| l.first()) else {
            return Vec::new();
        };
        let face = &self.faces[f];
        if face.len() != 4 {
            return vec![f];
        }
        let k = (0..4)
            .find(|&k| edge(face[k], face[(k + 1) % 4]) == e)
            .unwrap();
        self.face_ring(edge(face[(k + 1) % 4], face[(k + 2) % 4]))
    }

    /// The edge loop through `e` (edges continuing straight across valence-4 vertices).
    pub fn edge_loop(&self, e: Edge) -> Vec<Edge> {
        let ef = self.edge_faces();
        let mut vert_edges: HashMap<u32, Vec<Edge>> = HashMap::new();
        for &k in ef.keys() {
            vert_edges.entry(k.0).or_default().push(k);
            vert_edges.entry(k.1).or_default().push(k);
        }
        let mut out = vec![e];
        for (from, at) in [(e.0, e.1), (e.1, e.0)] {
            let mut cur = at;
            let _ = from;
            let mut prev_edge = e;
            for _ in 0..10_000 {
                let es = &vert_edges[&cur];
                if es.len() != 4 {
                    break;
                }
                // The edge sharing no face with the one we came along.
                let faces_prev: BTreeSet<usize> = ef[&prev_edge].iter().copied().collect();
                let next = es
                    .iter()
                    .copied()
                    .find(|&x| x != prev_edge && ef[&x].iter().all(|f| !faces_prev.contains(f)));
                let Some(n) = next else { break };
                if out.contains(&n) {
                    break;
                }
                out.push(n);
                let other = if n.0 == cur { n.1 } else { n.0 };
                cur = other;
                prev_edge = n;
            }
        }
        out
    }

    /// The same faces wound consistently (each edge run both ways by its two faces), face by
    /// face from the first face of every connected piece.
    pub fn oriented(&self) -> Poly {
        let ef = self.edge_faces();
        let mut faces = self.faces.clone();
        let mut seen = vec![false; faces.len()];
        for start in 0..faces.len() {
            if seen[start] {
                continue;
            }
            seen[start] = true;
            let mut stack = vec![start];
            while let Some(f) = stack.pop() {
                let face = faces[f].clone();
                for k in 0..face.len() {
                    let (a, b) = (face[k], face[(k + 1) % face.len()]);
                    for &g in &ef[&edge(a, b)] {
                        if g == f || seen[g] {
                            continue;
                        }
                        // g must run b → a.
                        let gf = &faces[g];
                        let runs_same =
                            (0..gf.len()).any(|j| gf[j] == a && gf[(j + 1) % gf.len()] == b);
                        if runs_same {
                            faces[g].reverse();
                        }
                        seen[g] = true;
                        stack.push(g);
                    }
                }
            }
        }
        Poly {
            verts: self.verts.clone(),
            faces,
        }
    }

    /// Weld vertices closer than `tol` and drop faces that collapse.
    pub fn welded(&self, tol: f64) -> Poly {
        let k = 1.0 / tol.max(1e-12);
        let mut map: HashMap<[i64; 3], u32> = HashMap::new();
        let mut verts = Vec::new();
        let mut remap = Vec::with_capacity(self.verts.len());
        for v in &self.verts {
            let key = [
                (v[0] * k).round() as i64,
                (v[1] * k).round() as i64,
                (v[2] * k).round() as i64,
            ];
            let id = *map.entry(key).or_insert_with(|| {
                verts.push(*v);
                (verts.len() - 1) as u32
            });
            remap.push(id);
        }
        let faces = self
            .faces
            .iter()
            .filter_map(|f| {
                let mut g: Vec<u32> = f.iter().map(|&i| remap[i as usize]).collect();
                g.dedup();
                if g.len() > 1 && g[0] == *g.last().unwrap() {
                    g.pop();
                }
                (g.len() >= 3).then_some(g)
            })
            .collect();
        Poly { verts, faces }
    }

    /// Remove faces and the vertices nobody uses any more.
    pub fn without_faces(&self, drop: &BTreeSet<usize>) -> Poly {
        let faces: Vec<Vec<u32>> = self
            .faces
            .iter()
            .enumerate()
            .filter(|(i, _)| !drop.contains(i))
            .map(|(_, f)| f.clone())
            .collect();
        Poly {
            verts: self.verts.clone(),
            faces,
        }
        .compacted()
    }

    pub fn compacted(&self) -> Poly {
        let mut remap: HashMap<u32, u32> = HashMap::new();
        let mut verts = Vec::new();
        let faces = self
            .faces
            .iter()
            .map(|f| {
                f.iter()
                    .map(|&i| {
                        *remap.entry(i).or_insert_with(|| {
                            verts.push(self.verts[i as usize]);
                            (verts.len() - 1) as u32
                        })
                    })
                    .collect()
            })
            .collect();
        Poly { verts, faces }
    }

    pub fn flipped(&self) -> Poly {
        Poly {
            verts: self.verts.clone(),
            faces: self
                .faces
                .iter()
                .map(|f| f.iter().rev().copied().collect())
                .collect(),
        }
    }

    /// Vertices used by the faces.
    pub fn face_verts(&self, faces: &BTreeSet<usize>) -> BTreeSet<u32> {
        faces
            .iter()
            .flat_map(|&f| self.faces[f].iter().copied())
            .collect()
    }
}

/// One Catmull–Clark step. Boundary edges and `creases` (vertex pairs) stay sharp.
pub fn catmull_clark(p: &Poly, creases: &BTreeSet<Edge>) -> Poly {
    let nv = p.verts.len();
    let ef = p.edge_faces();
    let fp: Vec<Vec3> = (0..p.faces.len()).map(|f| p.face_center(f)).collect();
    let edges: Vec<Edge> = ef.keys().copied().collect();
    let eidx: HashMap<Edge, usize> = edges.iter().enumerate().map(|(i, e)| (*e, i)).collect();
    let sharp = |e: &Edge| ef[e].len() != 2 || creases.contains(e);
    let ep: Vec<Vec3> = edges
        .iter()
        .map(|e| {
            let (a, b) = (p.verts[e.0 as usize], p.verts[e.1 as usize]);
            if sharp(e) {
                scale(add(a, b), 0.5)
            } else {
                let f = &ef[e];
                scale(add(add(a, b), add(fp[f[0]], fp[f[1]])), 0.25)
            }
        })
        .collect();
    // Per vertex: incident faces and edges.
    let mut vf: Vec<Vec<usize>> = vec![Vec::new(); nv];
    for (i, f) in p.faces.iter().enumerate() {
        for &v in f {
            vf[v as usize].push(i);
        }
    }
    let mut ve: Vec<Vec<Edge>> = vec![Vec::new(); nv];
    for e in &edges {
        ve[e.0 as usize].push(*e);
        ve[e.1 as usize].push(*e);
    }
    let vp: Vec<Vec3> = (0..nv)
        .map(|v| {
            let pv = p.verts[v];
            let es = &ve[v];
            if es.is_empty() {
                return pv;
            }
            let sharp_es: Vec<&Edge> = es.iter().filter(|e| sharp(e)).collect();
            match sharp_es.len() {
                0 => {
                    let n = es.len() as f64;
                    let f = scale(
                        vf[v].iter().fold([0.0; 3], |a, &i| add(a, fp[i])),
                        1.0 / vf[v].len() as f64,
                    );
                    let r = scale(
                        es.iter().fold([0.0; 3], |a, e| {
                            add(
                                a,
                                scale(add(p.verts[e.0 as usize], p.verts[e.1 as usize]), 0.5),
                            )
                        }),
                        1.0 / n,
                    );
                    scale(add(add(f, scale(r, 2.0)), scale(pv, n - 3.0)), 1.0 / n)
                }
                1 => pv,
                // A corner of an open border (one face) stays put.
                2 if vf[v].len() == 1 => pv,
                2 => {
                    let other =
                        |e: &Edge| p.verts[if e.0 as usize == v { e.1 } else { e.0 } as usize];
                    add(
                        scale(pv, 0.75),
                        scale(add(other(sharp_es[0]), other(sharp_es[1])), 0.125),
                    )
                }
                _ => pv,
            }
        })
        .collect();
    let mut verts = vp;
    let e_base = verts.len() as u32;
    verts.extend(ep.iter().copied());
    let f_base = verts.len() as u32;
    verts.extend(fp.iter().copied());
    let mut faces = Vec::new();
    for (i, f) in p.faces.iter().enumerate() {
        let n = f.len();
        for k in 0..n {
            let (prev, cur, next) = (f[(k + n - 1) % n], f[k], f[(k + 1) % n]);
            faces.push(vec![
                cur,
                e_base + eidx[&edge(cur, next)] as u32,
                f_base + i as u32,
                e_base + eidx[&edge(prev, cur)] as u32,
            ]);
        }
    }
    Poly { verts, faces }
}

/// `levels` subdivision steps; creases stay creased on the new edges along them.
pub fn subdivide(p: &Poly, levels: usize, creases: &BTreeSet<Edge>) -> Poly {
    let mut cur = p.clone();
    let mut cr = creases.clone();
    for _ in 0..levels {
        let next = catmull_clark(&cur, &cr);
        // A creased edge (a, b) becomes (a', mid) and (mid, b'): find the mid by position.
        if !cr.is_empty() {
            let ef = cur.edge_faces();
            let edges: Vec<Edge> = ef.keys().copied().collect();
            let base = cur.verts.len() as u32;
            let mut ncr = BTreeSet::new();
            for (i, e) in edges.iter().enumerate() {
                if cr.contains(e) {
                    let m = base + i as u32;
                    ncr.insert(edge(e.0, m));
                    ncr.insert(edge(m, e.1));
                }
            }
            cr = ncr;
        }
        cur = next;
    }
    cur
}

/// Grid cage from rows of points (rows along u); closed directions wrap.
pub fn grid(rows: &[Vec<Vec3>], closed_u: bool, closed_v: bool) -> Poly {
    let nu = rows.len();
    let nv = rows.first().map_or(0, Vec::len);
    let verts: Vec<Vec3> = rows.iter().flatten().copied().collect();
    let id = |i: usize, j: usize| ((i % nu) * nv + (j % nv)) as u32;
    let mut faces = Vec::new();
    let iu = if closed_u { nu } else { nu - 1 };
    let iv = if closed_v { nv } else { nv - 1 };
    for i in 0..iu {
        for j in 0..iv {
            faces.push(vec![id(i, j), id(i + 1, j), id(i + 1, j + 1), id(i, j + 1)]);
        }
    }
    Poly { verts, faces }
}

/// Extrude faces along their (averaged) normals by `d`: the faces move out, side walls join
/// their old borders. Returns the cage and the moved faces.
pub fn extrude_faces(p: &Poly, sel: &BTreeSet<usize>, d: f64) -> (Poly, BTreeSet<usize>) {
    let mut out = p.clone();
    let region_edges: BTreeMap<Edge, usize> = {
        let mut m = BTreeMap::new();
        for &f in sel {
            let face = &p.faces[f];
            for k in 0..face.len() {
                *m.entry(edge(face[k], face[(k + 1) % face.len()]))
                    .or_insert(0) += 1;
            }
        }
        m
    };
    let verts = p.face_verts(sel);
    let mut normal: HashMap<u32, Vec3> = HashMap::new();
    for &f in sel {
        let n = p.face_normal(f);
        for &v in &p.faces[f] {
            let e = normal.entry(v).or_insert([0.0; 3]);
            *e = add(*e, n);
        }
    }
    let mut newv: HashMap<u32, u32> = HashMap::new();
    for &v in &verts {
        out.verts
            .push(add(p.verts[v as usize], scale(unit(normal[&v]), d)));
        newv.insert(v, (out.verts.len() - 1) as u32);
    }
    for &f in sel {
        let face = p.faces[f].clone();
        // Side walls along the region's border (edges used once in the region).
        for k in 0..face.len() {
            let (a, b) = (face[k], face[(k + 1) % face.len()]);
            if region_edges[&edge(a, b)] == 1 {
                out.faces.push(vec![a, b, newv[&b], newv[&a]]);
            }
        }
        out.faces[f] = face.iter().map(|v| newv[v]).collect();
    }
    (out, sel.clone())
}

/// Inset each face by `amount` (toward its center, as a fraction 0..1 when `relative`).
pub fn inset_faces(
    p: &Poly,
    sel: &BTreeSet<usize>,
    amount: f64,
    relative: bool,
) -> (Poly, BTreeSet<usize>) {
    let mut out = p.clone();
    let mut inner = BTreeSet::new();
    for &f in sel {
        let face = p.faces[f].clone();
        let c = p.face_center(f);
        let start = out.verts.len() as u32;
        for &v in &face {
            let pv = p.verts[v as usize];
            let toward = sub(c, pv);
            let t = if relative {
                amount.clamp(0.0, 0.95)
            } else {
                (amount / norm(toward).max(1e-12)).clamp(0.0, 0.95)
            };
            out.verts.push(add(pv, scale(toward, t)));
        }
        let n = face.len() as u32;
        for k in 0..n {
            let (a, b) = (face[k as usize], face[((k + 1) % n) as usize]);
            out.faces.push(vec![a, b, start + (k + 1) % n, start + k]);
        }
        out.faces[f] = (0..n).map(|k| start + k).collect();
        inner.insert(f);
    }
    (out, inner)
}

/// Split every quad of the face ring across `e` at fraction `t` (an edge loop insertion);
/// `t` runs from `e.0` toward `e.1` and keeps that side all along the ring.
pub fn insert_loop(p: &Poly, e: Edge, t: f64) -> Poly {
    let ef = p.edge_faces();
    let mut out = p.clone();
    let mut mid: HashMap<Edge, u32> = HashMap::new();
    let mut split = |out: &mut Poly, from: u32, to: u32| -> u32 {
        let key = edge(from, to);
        if let Some(&m) = mid.get(&key) {
            return m;
        }
        let (pa, pb) = (out.verts[from as usize], out.verts[to as usize]);
        out.verts.push(add(pa, scale(sub(pb, pa), t)));
        let id = (out.verts.len() - 1) as u32;
        mid.insert(key, id);
        id
    };
    let mut done = BTreeSet::new();
    let mut new_faces: Vec<(usize, Vec<u32>, Vec<u32>)> = Vec::new();
    for &start in ef.get(&e).into_iter().flatten() {
        let (mut f, mut from, mut to) = (start, e.0, e.1);
        loop {
            if !done.insert(f) {
                break;
            }
            let face = &p.faces[f];
            if face.len() != 4 {
                break;
            }
            let k = (0..4)
                .find(|&k| edge(face[k], face[(k + 1) % 4]) == edge(from, to))
                .unwrap();
            let (a, b, c, d) = (
                face[k],
                face[(k + 1) % 4],
                face[(k + 2) % 4],
                face[(k + 3) % 4],
            );
            // The opposite edge runs d → c beside a → b.
            let (ofrom, oto) = if a == from { (d, c) } else { (c, d) };
            let m1 = split(&mut out, from, to);
            let m2 = split(&mut out, ofrom, oto);
            // Keep the winding: [a, m1, m2', d] and [m1, b, c, m2'] with m2' on c-d.
            new_faces.push((f, vec![a, m1, m2, d], vec![m1, b, c, m2]));
            let opp = edge(c, d);
            let Some(next) = ef
                .get(&opp)
                .and_then(|l| l.iter().copied().find(|&g| g != f))
            else {
                break;
            };
            f = next;
            from = ofrom;
            to = oto;
        }
    }
    for (f, q1, q2) in new_faces {
        out.faces[f] = q1;
        out.faces.push(q2);
    }
    out
}

/// Fill each open border loop with one face (a cap).
pub fn fill_holes(p: &Poly) -> Poly {
    let naked = p.naked_edges();
    // Directed border edges (as the faces run them), reversed for the caps.
    let mut next: HashMap<u32, u32> = HashMap::new();
    let naked_set: BTreeSet<Edge> = naked.iter().copied().collect();
    for f in &p.faces {
        for k in 0..f.len() {
            let (a, b) = (f[k], f[(k + 1) % f.len()]);
            if naked_set.contains(&edge(a, b)) {
                next.insert(b, a);
            }
        }
    }
    let mut out = p.clone();
    let mut done = BTreeSet::new();
    for &start in next.keys().copied().collect::<Vec<_>>().iter() {
        if done.contains(&start) {
            continue;
        }
        let mut lp = vec![start];
        done.insert(start);
        let mut cur = start;
        while let Some(&n) = next.get(&cur) {
            if n == start || lp.len() > 100_000 {
                break;
            }
            if !done.insert(n) {
                break;
            }
            lp.push(n);
            cur = n;
        }
        if lp.len() >= 3 {
            out.faces.push(lp);
        }
    }
    out
}

/// Vertex normals (area weighted).
pub fn vertex_normals(p: &Poly) -> Vec<Vec3> {
    let mut n = vec![[0.0; 3]; p.verts.len()];
    for (i, f) in p.faces.iter().enumerate() {
        let fnm = p.face_normal(i);
        for &v in f {
            n[v as usize] = add(n[v as usize], fnm);
        }
    }
    n.into_iter().map(unit).collect()
}

/// A closed shell: the surface offset by `t` inward, both joined along open borders.
pub fn shell(p: &Poly, t: f64) -> Poly {
    let n = vertex_normals(p);
    let k = p.verts.len() as u32;
    let mut out = p.clone();
    out.verts
        .extend(p.verts.iter().zip(&n).map(|(v, nn)| sub(*v, scale(*nn, t))));
    for f in &p.faces {
        out.faces.push(f.iter().rev().map(|i| i + k).collect());
    }
    let naked: BTreeSet<Edge> = p.naked_edges().into_iter().collect();
    for f in &p.faces {
        for j in 0..f.len() {
            let (a, b) = (f[j], f[(j + 1) % f.len()]);
            if naked.contains(&edge(a, b)) {
                out.faces.push(vec![b, a, a + k, b + k]);
            }
        }
    }
    out
}

/// Box cage of `n` × `n` × `n` faces per side.
pub fn box_cage(min: Vec3, max: Vec3, n: [usize; 3]) -> Poly {
    let mut p = Poly::default();
    let lerp = |a: f64, b: f64, t: f64| a + (b - a) * t;
    // Six grids, each wound outward, then welded.
    let sides: [(usize, usize, usize, f64, bool); 6] = [
        (0, 1, 2, 0.0, false),
        (0, 1, 2, 1.0, true),
        (1, 2, 0, 0.0, false),
        (1, 2, 0, 1.0, true),
        (2, 0, 1, 0.0, false),
        (2, 0, 1, 1.0, true),
    ];
    for (a, b, c, at, outward_pos) in sides {
        let (na, nb) = (n[a], n[b]);
        let rows: Vec<Vec<Vec3>> = (0..=na)
            .map(|i| {
                (0..=nb)
                    .map(|j| {
                        let mut q = [0.0; 3];
                        q[a] = lerp(min[a], max[a], i as f64 / na as f64);
                        q[b] = lerp(min[b], max[b], j as f64 / nb as f64);
                        q[c] = lerp(min[c], max[c], at);
                        q
                    })
                    .collect()
            })
            .collect();
        let mut g = grid(&rows, false, false);
        // grid() winds a×b counter-clockwise around +c; the low side must face −c.
        if !outward_pos {
            g = g.flipped();
        }
        let base = p.verts.len() as u32;
        p.verts.extend(g.verts);
        p.faces.extend(
            g.faces
                .into_iter()
                .map(|f| f.into_iter().map(|i| i + base).collect()),
        );
    }
    p.welded(1e-9)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_subdivided_cube_is_closed_and_round() {
        let c = box_cage([-1.0; 3], [1.0; 3], [1, 1, 1]);
        assert_eq!(c.verts.len(), 8);
        assert_eq!(c.faces.len(), 6);
        assert!(
            c.to_mesh().volume() > 7.9,
            "outward: {}",
            c.to_mesh().volume()
        );
        let s = subdivide(&c, 3, &BTreeSet::new());
        assert_eq!(s.faces.len(), 6 * 64);
        let m = s.to_mesh();
        assert!(m.is_closed());
        // Catmull–Clark shrinks a cube toward a rounded shape inside it.
        let r: Vec<f64> = s.verts.iter().map(|v| norm(*v)).collect();
        let (lo, hi) = (
            r.iter().cloned().fold(f64::MAX, f64::min),
            r.iter().cloned().fold(f64::MIN, f64::max),
        );
        assert!(hi < 1.0 && lo > 0.5 && hi - lo < 0.3, "{lo} {hi}");
        // The quads survive the triangle round trip.
        assert_eq!(Poly::from_mesh(&m).faces.len(), s.faces.len());
    }

    #[test]
    fn creases_and_boundaries_stay_sharp() {
        let rows: Vec<Vec<Vec3>> = (0..3)
            .map(|i| (0..3).map(|j| [i as f64, j as f64, 0.0]).collect())
            .collect();
        let g = grid(&rows, false, false);
        let s = subdivide(&g, 2, &BTreeSet::new());
        // Boundary corners stay put.
        assert!(s.verts.iter().any(|v| dist(*v, [0.0; 3]) < 1e-12));
        let c = box_cage([-1.0; 3], [1.0; 3], [1, 1, 1]);
        let all: BTreeSet<Edge> = c.edge_faces().keys().copied().collect();
        let s = subdivide(&c, 2, &all);
        // Fully creased: the cube keeps its corners.
        assert!(s.verts.iter().any(|v| dist(*v, [1.0, 1.0, 1.0]) < 1e-9));
    }

    #[test]
    fn extrude_inset_loops_and_fill() {
        let c = box_cage([-1.0; 3], [1.0; 3], [2, 2, 2]);
        let top = c.pick_face([0.5, 0.5, 5.0], [0.0, 0.0, 1.0]).unwrap();
        assert!((c.face_center(top)[2] - 1.0).abs() < 1e-12);
        let (e, sel) = extrude_faces(&c, &BTreeSet::from([top]), 1.0);
        assert!((e.face_center(*sel.iter().next().unwrap())[2] - 2.0).abs() < 1e-12);
        assert!(e.to_mesh().is_closed());
        let (i, _) = inset_faces(&c, &BTreeSet::from([top]), 0.25, true);
        assert_eq!(i.faces.len(), c.faces.len() + 4);
        assert!(i.to_mesh().is_closed());
        let ed = c.pick_edge([0.0, -1.0, 1.0], [0.0, 0.0, 1.0]).unwrap();
        let ring = c.face_ring(ed);
        assert!(ring.len() >= 4, "{}", ring.len());
        let lp = c.edge_loop(ed);
        assert!(lp.len() >= 4, "{}", lp.len());
        let split = insert_loop(&c, ed, 0.5);
        assert_eq!(split.faces.len(), c.faces.len() + ring.len());
        assert!(
            split.to_mesh().is_closed(),
            "the new loop keeps the cage closed"
        );
        // Remove the top faces and fill the hole again.
        let tops: BTreeSet<usize> = (0..c.faces.len())
            .filter(|&f| (c.face_center(f)[2] - 1.0).abs() < 1e-9)
            .collect();
        let open = c.without_faces(&tops);
        assert!(!open.naked_edges().is_empty());
        let filled = fill_holes(&open);
        assert!(filled.naked_edges().is_empty());
        let sh = shell(&open, 0.1);
        assert!(sh.to_mesh().is_closed());
    }
}
