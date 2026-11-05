// SPDX-License-Identifier: LGPL-2.1-or-later
//! Closed triangle meshes for add-ons (gems, prongs, bezels, cutters, primitives), built in a
//! local frame and placed with [`Frame`]. Every builder returns outward-facing triangles, so the
//! enclosed volume is positive.
use om9_addon_abi::session::{Geo, Vec3, add, cross, dot, norm, scale, sub, unit};
use std::f64::consts::TAU;

/// A triangle mesh in local or world millimetres.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Mesh {
    pub vertices: Vec<Vec3>,
    pub triangles: Vec<[u32; 3]>,
}

impl Mesh {
    /// Enclosed volume (mm³).
    pub fn volume(&self) -> f64 {
        self.triangles
            .iter()
            .map(|t| {
                let p = |i: u32| self.vertices[i as usize];
                dot(p(t[0]), cross(p(t[1]), p(t[2])))
            })
            .sum::<f64>()
            / 6.0
    }
    /// Every edge shared by exactly two triangles, in opposite directions.
    pub fn is_closed(&self) -> bool {
        let mut edges = std::collections::BTreeMap::<(u32, u32), i32>::new();
        for t in &self.triangles {
            for k in 0..3 {
                let (a, b) = (t[k], t[(k + 1) % 3]);
                *edges.entry((a.min(b), a.max(b))).or_insert(0) += if a < b { 1 } else { -1 };
            }
        }
        let mut count = std::collections::BTreeMap::<(u32, u32), u32>::new();
        for t in &self.triangles {
            for k in 0..3 {
                let (a, b) = (t[k], t[(k + 1) % 3]);
                *count.entry((a.min(b), a.max(b))).or_insert(0) += 1;
            }
        }
        count.values().all(|&n| n == 2) && edges.values().all(|&d| d == 0)
    }
    pub fn bbox(&self) -> [Vec3; 2] {
        let mut b = [[f64::INFINITY; 3], [f64::NEG_INFINITY; 3]];
        for v in &self.vertices {
            for i in 0..3 {
                b[0][i] = b[0][i].min(v[i]);
                b[1][i] = b[1][i].max(v[i]);
            }
        }
        b
    }
    /// Append another mesh (indices shifted).
    pub fn append(&mut self, other: &Mesh) {
        let base = self.vertices.len() as u32;
        self.vertices.extend_from_slice(&other.vertices);
        self.triangles.extend(
            other
                .triangles
                .iter()
                .map(|t| [t[0] + base, t[1] + base, t[2] + base]),
        );
    }
    pub fn placed(&self, f: &Frame) -> Mesh {
        Mesh {
            vertices: self.vertices.iter().map(|&p| f.point(p)).collect(),
            triangles: self.triangles.clone(),
        }
    }
    /// The mesh as add-on geometry.
    pub fn geo(self, name: &str, meta: Option<serde_json::Value>, layer: Option<&str>) -> Geo {
        Geo::Mesh {
            name: name.to_owned(),
            vertices: self.vertices,
            triangles: self.triangles,
            meta,
            layer: layer.map(str::to_owned),
        }
    }
    /// Feature edges for previews: every triangle edge (meshes here are small).
    pub fn wire(&self) -> Vec<Geo> {
        let mut seen = std::collections::BTreeSet::new();
        let mut out = Vec::new();
        for t in &self.triangles {
            for k in 0..3 {
                let (a, b) = (t[k], t[(k + 1) % 3]);
                if seen.insert((a.min(b), a.max(b))) {
                    out.push(Geo::Line {
                        start: self.vertices[a as usize],
                        end: self.vertices[b as usize],
                    });
                }
            }
        }
        out
    }
}

/// An orthonormal frame: origin, x, y and z (z = up / table axis / prong axis).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frame {
    pub origin: Vec3,
    pub x: Vec3,
    pub y: Vec3,
    pub z: Vec3,
}

impl Frame {
    pub const WORLD: Frame = Frame {
        origin: [0.0; 3],
        x: [1.0, 0.0, 0.0],
        y: [0.0, 1.0, 0.0],
        z: [0.0, 0.0, 1.0],
    };
    /// A frame from an origin, a z axis and a hint for x (made perpendicular to z).
    pub fn new(origin: Vec3, z: Vec3, x_hint: Vec3) -> Frame {
        let z = unit(z);
        let mut x = sub(x_hint, scale(z, dot(x_hint, z)));
        if norm(x) < 1e-9 {
            let h = if z[0].abs() < 0.9 {
                [1.0, 0.0, 0.0]
            } else {
                [0.0, 1.0, 0.0]
            };
            x = sub(h, scale(z, dot(h, z)));
        }
        let x = unit(x);
        Frame {
            origin,
            x,
            y: cross(z, x),
            z,
        }
    }
    pub fn point(&self, p: Vec3) -> Vec3 {
        add(
            self.origin,
            add(
                add(scale(self.x, p[0]), scale(self.y, p[1])),
                scale(self.z, p[2]),
            ),
        )
    }
    pub fn vector(&self, v: Vec3) -> Vec3 {
        add(
            add(scale(self.x, v[0]), scale(self.y, v[1])),
            scale(self.z, v[2]),
        )
    }
    /// World point → local coordinates.
    pub fn local(&self, p: Vec3) -> Vec3 {
        let d = sub(p, self.origin);
        [dot(d, self.x), dot(d, self.y), dot(d, self.z)]
    }
    /// The same frame rotated by `angle` radians about its z axis.
    pub fn rotated(&self, angle: f64) -> Frame {
        let (s, c) = angle.sin_cos();
        let x = add(scale(self.x, c), scale(self.y, s));
        Frame {
            origin: self.origin,
            x,
            y: cross(self.z, x),
            z: self.z,
        }
    }
    pub fn moved(&self, local: Vec3) -> Frame {
        Frame {
            origin: self.point(local),
            ..*self
        }
    }
}

/// Stack of closed rings (each `n` points, counter-clockwise seen from +z) from top to bottom,
/// capped at both ends by a fan or a point. `top` / `bottom` are the cap points (`None`: a
/// flat cap at the ring's centroid). Rings must have equal length.
pub fn ring_stack(rings: &[Vec<Vec3>], top: Option<Vec3>, bottom: Option<Vec3>) -> Mesh {
    let n = rings[0].len();
    let centroid = |r: &Vec<Vec3>| {
        let s = r.iter().fold([0.0; 3], |a, &p| add(a, p));
        scale(s, 1.0 / r.len() as f64)
    };
    let mut m = Mesh::default();
    let top_p = top.unwrap_or_else(|| centroid(&rings[0]));
    m.vertices.push(top_p);
    for r in rings {
        m.vertices.extend_from_slice(r);
    }
    let bottom_p = bottom.unwrap_or_else(|| centroid(&rings[rings.len() - 1]));
    let bi = m.vertices.len() as u32;
    m.vertices.push(bottom_p);
    let at = |k: usize, i: usize| 1 + (k * n + i % n) as u32;
    for i in 0..n {
        m.triangles.push([0, at(0, i), at(0, i + 1)]);
        for k in 0..rings.len() - 1 {
            let (a, b, c, d) = (at(k, i), at(k + 1, i), at(k + 1, i + 1), at(k, i + 1));
            m.triangles.push([a, b, c]);
            m.triangles.push([a, c, d]);
        }
        let last = rings.len() - 1;
        m.triangles.push([at(last, i), bi, at(last, i + 1)]);
    }
    m
}

/// Revolve a profile of `(radius, z)` stations from top to bottom about local z.
pub fn revolve(profile: &[(f64, f64)], segments: usize) -> Mesh {
    let n = segments.max(3);
    let mut rings: Vec<Vec<Vec3>> = Vec::new();
    let (mut top, mut bottom) = (None, None);
    for (k, &(r, z)) in profile.iter().enumerate() {
        if r <= 1e-12 {
            if k == 0 {
                top = Some([0.0, 0.0, z]);
            } else if k == profile.len() - 1 {
                bottom = Some([0.0, 0.0, z]);
            }
            continue;
        }
        rings.push(circle(r, z, n));
    }
    if top.is_none() {
        top = Some([0.0, 0.0, profile[0].1]);
    }
    if bottom.is_none() {
        bottom = Some([0.0, 0.0, profile[profile.len() - 1].1]);
    }
    ring_stack(&rings, top, bottom)
}

/// `n` points of a circle of radius `r` at height `z`, counter-clockwise.
pub fn circle(r: f64, z: f64, n: usize) -> Vec<Vec3> {
    (0..n)
        .map(|i| {
            let a = TAU * i as f64 / n as f64;
            [r * a.cos(), r * a.sin(), z]
        })
        .collect()
}

/// Closed cylinder of radius `r` from z0 (bottom) to z1 (top).
pub fn cylinder(r: f64, z0: f64, z1: f64, segments: usize) -> Mesh {
    revolve(&[(0.0, z1), (r, z1), (r, z0), (0.0, z0)], segments)
}

/// Cylinder with a hemispherical (`dome`) top: a prong or a pin.
pub fn domed_cylinder(r: f64, z0: f64, z1: f64, segments: usize) -> Mesh {
    let mut profile = vec![(0.0, z1 + r)];
    let steps = (segments / 4).max(3);
    for k in 1..=steps {
        let a = std::f64::consts::FRAC_PI_2 * k as f64 / steps as f64;
        profile.push((r * a.sin(), z1 + r * a.cos()));
    }
    profile.push((r, z0));
    profile.push((0.0, z0));
    revolve(&profile, segments)
}

/// UV sphere of radius `r` centered at the origin.
pub fn sphere(r: f64, segments: usize) -> Mesh {
    let rings = (segments / 2).max(3);
    let mut profile = vec![(0.0, r)];
    for k in 1..rings {
        let a = std::f64::consts::PI * k as f64 / rings as f64;
        profile.push((r * a.sin(), r * a.cos()));
    }
    profile.push((0.0, -r));
    revolve(&profile, segments)
}

/// Ellipsoid with semi-axes (a, b, c).
pub fn ellipsoid(a: f64, b: f64, c: f64, segments: usize) -> Mesh {
    let mut m = sphere(1.0, segments);
    for v in &mut m.vertices {
        *v = [v[0] * a, v[1] * b, v[2] * c];
    }
    m
}

/// Torus about local z: major radius `big`, tube radius `small`.
pub fn torus(big: f64, small: f64, segments: usize, tube_segments: usize) -> Mesh {
    let (n, k) = (segments.max(3), tube_segments.max(3));
    let mut m = Mesh::default();
    for i in 0..n {
        let a = TAU * i as f64 / n as f64;
        for j in 0..k {
            let b = TAU * j as f64 / k as f64;
            let r = big + small * b.cos();
            m.vertices.push([r * a.cos(), r * a.sin(), small * b.sin()]);
        }
    }
    let at = |i: usize, j: usize| ((i % n) * k + (j % k)) as u32;
    for i in 0..n {
        for j in 0..k {
            let (p, q, r, s) = (at(i, j), at(i + 1, j), at(i + 1, j + 1), at(i, j + 1));
            m.triangles.push([p, q, r]);
            m.triangles.push([p, r, s]);
        }
    }
    m
}

/// Box from min to max corner.
pub fn cuboid(min: Vec3, max: Vec3) -> Mesh {
    let (a, b) = (min, max);
    let v = vec![
        [a[0], a[1], a[2]],
        [b[0], a[1], a[2]],
        [b[0], b[1], a[2]],
        [a[0], b[1], a[2]],
        [a[0], a[1], b[2]],
        [b[0], a[1], b[2]],
        [b[0], b[1], b[2]],
        [a[0], b[1], b[2]],
    ];
    let t = vec![
        [0, 2, 1],
        [0, 3, 2],
        [4, 5, 6],
        [4, 6, 7],
        [0, 1, 5],
        [0, 5, 4],
        [1, 2, 6],
        [1, 6, 5],
        [2, 3, 7],
        [2, 7, 6],
        [3, 0, 4],
        [3, 4, 7],
    ];
    Mesh {
        vertices: v,
        triangles: t,
    }
}

/// A closed tube between an outer and an inner ring stack: outer rings top→bottom and
/// inner rings top→bottom (same counts). Used for bezels and halo channels.
pub fn tube(outer: &[Vec<Vec3>], inner: &[Vec<Vec3>]) -> Mesh {
    let n = outer[0].len();
    let mut m = Mesh::default();
    for r in outer.iter().chain(inner) {
        m.vertices.extend_from_slice(r);
    }
    let (o, i_) = (outer.len(), inner.len());
    let at = |k: usize, i: usize| (k * n + i % n) as u32;
    for i in 0..n {
        for k in 0..o - 1 {
            let (a, b, c, d) = (at(k, i), at(k + 1, i), at(k + 1, i + 1), at(k, i + 1));
            m.triangles.push([a, b, c]);
            m.triangles.push([a, c, d]);
        }
        for k in o..o + i_ - 1 {
            // inner wall faces inward: reversed winding
            let (a, b, c, d) = (at(k, i), at(k + 1, i), at(k + 1, i + 1), at(k, i + 1));
            m.triangles.push([a, c, b]);
            m.triangles.push([a, d, c]);
        }
        // top rim: outer top ring → inner top ring
        let (a, b, c, d) = (at(0, i), at(0, i + 1), at(o, i + 1), at(o, i));
        m.triangles.push([a, b, c]);
        m.triangles.push([a, c, d]);
        // bottom rim
        let (a, b, c, d) = (at(o - 1, i), at(o - 1, i + 1), at(o + i_ - 1, i + 1), at(o + i_ - 1, i));
        m.triangles.push([a, c, b]);
        m.triangles.push([a, d, c]);
    }
    m
}

/// Sweep a closed circular section of radius `r` along a polyline path (open or closed):
/// wires, beads strings, rails. Open paths get flat caps.
pub fn pipe(path: &[Vec3], closed: bool, r: f64, segments: usize) -> Option<Mesh> {
    let n = segments.max(3);
    if path.len() < 2 {
        return None;
    }
    let count = path.len();
    let tangent = |i: usize| -> Vec3 {
        let (a, b) = if closed {
            (path[(i + count - 1) % count], path[(i + 1) % count])
        } else {
            (path[i.saturating_sub(1)], path[(i + 1).min(count - 1)])
        };
        unit(sub(b, a))
    };
    // rotation-minimising frames by projection
    let t0 = tangent(0);
    let mut x = Frame::new(path[0], t0, [0.0, 0.0, 1.0]).x;
    let mut rings = Vec::new();
    for (i, &p) in path.iter().enumerate() {
        let t = tangent(i);
        x = unit(sub(x, scale(t, dot(x, t))));
        let y = cross(t, x);
        rings.push(
            (0..n)
                .map(|j| {
                    let a = TAU * j as f64 / n as f64;
                    add(p, add(scale(x, r * a.cos()), scale(y, r * a.sin())))
                })
                .collect::<Vec<_>>(),
        );
    }
    let mut m = Mesh::default();
    for ring in &rings {
        m.vertices.extend_from_slice(ring);
    }
    let rc = rings.len();
    let at = |k: usize, j: usize| ((k % rc) * n + j % n) as u32;
    let spans = if closed { rc } else { rc - 1 };
    for k in 0..spans {
        for j in 0..n {
            let (a, b, c, d) = (at(k, j), at(k, j + 1), at(k + 1, j + 1), at(k + 1, j));
            m.triangles.push([a, b, c]);
            m.triangles.push([a, c, d]);
        }
    }
    if !closed {
        let s = m.vertices.len() as u32;
        m.vertices.push(path[0]);
        m.vertices.push(path[count - 1]);
        for j in 0..n {
            m.triangles.push([s, at(0, j + 1), at(0, j)]);
            m.triangles.push([s + 1, at(rc - 1, j), at(rc - 1, j + 1)]);
        }
    }
    // Flip if the winding came out inward.
    if m.volume() < 0.0 {
        for t in &mut m.triangles {
            t.swap(1, 2);
        }
    }
    Some(m)
}

/// Offset a closed planar outline (counter-clockwise, in the xy plane) by `d` (outward > 0),
/// moving each point along its averaged edge normal.
pub fn offset_outline(outline: &[[f64; 2]], d: f64) -> Vec<[f64; 2]> {
    let n = outline.len();
    (0..n)
        .map(|i| {
            let p = outline[(i + n - 1) % n];
            let q = outline[i];
            let r = outline[(i + 1) % n];
            let e1 = [q[0] - p[0], q[1] - p[1]];
            let e2 = [r[0] - q[0], r[1] - q[1]];
            let n1 = normalize2([e1[1], -e1[0]]);
            let n2 = normalize2([e2[1], -e2[0]]);
            let m = normalize2([n1[0] + n2[0], n1[1] + n2[1]]);
            // keep the edges at distance d: divide by cos(half angle), bounded for spikes
            let cos = (m[0] * n1[0] + m[1] * n1[1]).max(0.3);
            [q[0] + m[0] * d / cos, q[1] + m[1] * d / cos]
        })
        .collect()
}

fn normalize2(v: [f64; 2]) -> [f64; 2] {
    let l = (v[0] * v[0] + v[1] * v[1]).sqrt();
    if l < 1e-15 { [0.0, 0.0] } else { [v[0] / l, v[1] / l] }
}

/// Outline points lifted to height z.
pub fn ring(outline: &[[f64; 2]], z: f64) -> Vec<Vec3> {
    outline.iter().map(|p| [p[0], p[1], z]).collect()
}

/// Outline scaled about its origin.
pub fn scaled(outline: &[[f64; 2]], sx: f64, sy: f64) -> Vec<[f64; 2]> {
    outline.iter().map(|p| [p[0] * sx, p[1] * sy]).collect()
}

/// Length of a closed outline.
pub fn perimeter(outline: &[[f64; 2]]) -> f64 {
    let n = outline.len();
    (0..n)
        .map(|i| {
            let (a, b) = (outline[i], outline[(i + 1) % n]);
            ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt()
        })
        .sum()
}

/// Join rings (equal point counts, each counter-clockwise looking along the run) into a
/// closed mesh: end caps for an open run, a joined loop for a closed one.
pub fn loft_rings(rings: &[Vec<Vec3>], closed: bool) -> Option<Mesh> {
    let count = rings.len();
    if count < 2 || rings[0].len() < 3 || rings.iter().any(|r| r.len() != rings[0].len()) {
        return None;
    }
    let mut m = if closed {
        let n = rings[0].len();
        let mut m = Mesh::default();
        for r in rings {
            m.vertices.extend_from_slice(r);
        }
        let at = |k: usize, j: usize| ((k % count) * n + j % n) as u32;
        for k in 0..count {
            for j in 0..n {
                let (a, b, c, d) = (at(k, j), at(k + 1, j), at(k + 1, j + 1), at(k, j + 1));
                m.triangles.push([a, b, c]);
                m.triangles.push([a, c, d]);
            }
        }
        m
    } else {
        ring_stack(rings, None, None)
    };
    if m.volume() < 0.0 {
        for t in &mut m.triangles {
            t.swap(1, 2);
        }
    }
    Some(m)
}

/// The frame of a run of points at index `i`: (tangent, up projected, side = up × tangent).
pub fn run_frame(path: &[Vec3], closed: bool, i: usize, up: Vec3) -> (Vec3, Vec3, Vec3) {
    let count = path.len();
    let (a, b) = if closed {
        (path[(i + count - 1) % count], path[(i + 1) % count])
    } else {
        (path[i.saturating_sub(1)], path[(i + 1).min(count - 1)])
    };
    let t = unit(sub(b, a));
    let u = unit(sub(up, scale(t, dot(up, t))));
    (t, u, cross(u, t))
}

/// Sweep a closed 2D section along a path. The section is given as `(side, up)` offsets,
/// counter-clockwise looking along the path; `up` is the direction the section's +up follows
/// (projected perpendicular to the path). Open paths get flat end caps; closed paths join.
pub fn sweep(path: &[Vec3], closed: bool, up: Vec3, section: &[[f64; 2]]) -> Option<Mesh> {
    let count = path.len();
    if count < 2 || section.len() < 3 {
        return None;
    }
    let rings: Vec<Vec<Vec3>> = (0..count)
        .map(|i| {
            let (_, u, side) = run_frame(path, closed, i, up);
            section
                .iter()
                .map(|q| add(path[i], add(scale(side, q[0]), scale(u, q[1]))))
                .collect()
        })
        .collect();
    loft_rings(&rings, closed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn primitives_are_closed_with_the_right_volume() {
        let s = sphere(2.0, 96);
        assert!(s.is_closed());
        let exact = 4.0 / 3.0 * PI * 8.0;
        assert!((s.volume() / exact - 1.0).abs() < 0.01, "{}", s.volume());
        let c = cylinder(1.0, 0.0, 3.0, 128);
        assert!(c.is_closed());
        assert!((c.volume() / (PI * 3.0) - 1.0).abs() < 0.002);
        let t = torus(5.0, 1.0, 96, 48);
        assert!(t.is_closed());
        assert!((t.volume() / (2.0 * PI * PI * 5.0) - 1.0).abs() < 0.01);
        let b = cuboid([0.0; 3], [1.0, 2.0, 3.0]);
        assert!(b.is_closed());
        assert!((b.volume() - 6.0).abs() < 1e-12);
        let d = domed_cylinder(0.5, 0.0, 2.0, 32);
        assert!(d.is_closed() && d.volume() > 0.0);
    }

    #[test]
    fn tube_and_pipe_are_closed() {
        let outer = vec![circle(2.0, 1.0, 48), circle(2.0, 0.0, 48)];
        let inner = vec![circle(1.5, 1.0, 48), circle(1.5, 0.0, 48)];
        let m = tube(&outer, &inner);
        assert!(m.is_closed());
        let exact = PI * (4.0 - 2.25);
        assert!((m.volume() / exact - 1.0).abs() < 0.01, "{}", m.volume());
        let path: Vec<Vec3> = circle(8.0, 0.0, 64);
        let p = pipe(&path, true, 0.5, 16).unwrap();
        assert!(p.is_closed() && p.volume() > 0.0);
        let rect = [[1.0, -1.0], [1.0, 1.0], [-1.0, 1.0], [-1.0, -1.0]];
        let s = sweep(&[[0.0; 3], [5.0, 0.0, 0.0], [10.0, 0.0, 0.0]], false, [0.0, 0.0, 1.0], &rect).unwrap();
        assert!(s.is_closed());
        assert!((s.volume() - 40.0).abs() < 1e-9, "{}", s.volume());
        let ring = sweep(&circle(10.0, 0.0, 128), true, [0.0, 0.0, 1.0], &rect).unwrap();
        assert!(ring.is_closed() && ring.volume() > 0.0);
        let p = pipe(&[[0.0; 3], [10.0, 0.0, 0.0]], false, 1.0, 64).unwrap();
        assert!(p.is_closed());
        assert!((p.volume() / (PI * 10.0) - 1.0).abs() < 0.01);
    }

    #[test]
    fn frames_and_offsets() {
        let f = Frame::new([1.0, 2.0, 3.0], [0.0, 0.0, 2.0], [1.0, 1.0, 0.0]);
        assert!((norm(f.x) - 1.0).abs() < 1e-12 && dot(f.x, f.z).abs() < 1e-12);
        let p = f.point([1.0, 0.0, 0.0]);
        assert!((f.local(p)[0] - 1.0).abs() < 1e-12);
        let square = [[1.0, 1.0], [-1.0, 1.0], [-1.0, -1.0], [1.0, -1.0]];
        let o = offset_outline(&square, 0.5);
        assert!((o[0][0] - 1.5).abs() < 1e-9 && (o[0][1] - 1.5).abs() < 1e-9);
    }
}
