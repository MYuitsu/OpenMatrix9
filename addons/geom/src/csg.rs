// SPDX-License-Identifier: LGPL-2.1-or-later
//! Boolean operations on closed triangle meshes (union, difference, intersection) with a
//! binary space partition of the polygons: each solid clips the other's polygons against its
//! planes, and the surviving pieces make the result. Used by the Boolean Builder, the Solid
//! menu booleans, holes and cut-to-finger-rail.
//!
//! The result is a closed polygon set re-triangulated and welded; T-junctions can remain
//! along cut seams (fine for display, rendering and printing).
use crate::mesh::Mesh;
use om9_addon_abi::session::{Vec3, add, cross, dot, norm, scale, sub};

const EPS: f64 = 1e-7;

#[derive(Clone, Copy, Debug)]
struct Plane {
    n: Vec3,
    w: f64,
}

impl Plane {
    fn from_points(a: Vec3, b: Vec3, c: Vec3) -> Option<Plane> {
        let n = cross(sub(b, a), sub(c, a));
        let l = norm(n);
        (l > 1e-14).then(|| {
            let n = scale(n, 1.0 / l);
            Plane { n, w: dot(n, a) }
        })
    }
    fn flip(&mut self) {
        self.n = scale(self.n, -1.0);
        self.w = -self.w;
    }
}

#[derive(Clone, Debug)]
struct Polygon {
    points: Vec<Vec3>,
    plane: Plane,
}

impl Polygon {
    fn new(points: Vec<Vec3>) -> Option<Polygon> {
        let plane = Plane::from_points(points[0], points[1], points[2])?;
        Some(Polygon { points, plane })
    }
    fn flip(&mut self) {
        self.points.reverse();
        self.plane.flip();
    }
}

const COPLANAR: u8 = 0;
const FRONT: u8 = 1;
const BACK: u8 = 2;
const SPANNING: u8 = 3;

/// Split `poly` by `plane` into the four buckets.
fn split(
    plane: &Plane,
    poly: &Polygon,
    coplanar_front: &mut Vec<Polygon>,
    coplanar_back: &mut Vec<Polygon>,
    front: &mut Vec<Polygon>,
    back: &mut Vec<Polygon>,
) {
    let mut kind = 0u8;
    let types: Vec<u8> = poly
        .points
        .iter()
        .map(|&p| {
            let t = dot(plane.n, p) - plane.w;
            let k = if t < -EPS {
                BACK
            } else if t > EPS {
                FRONT
            } else {
                COPLANAR
            };
            kind |= k;
            k
        })
        .collect();
    match kind {
        COPLANAR => {
            if dot(plane.n, poly.plane.n) > 0.0 {
                coplanar_front.push(poly.clone());
            } else {
                coplanar_back.push(poly.clone());
            }
        }
        FRONT => front.push(poly.clone()),
        BACK => back.push(poly.clone()),
        _ => {
            let (mut f, mut b) = (Vec::new(), Vec::new());
            let n = poly.points.len();
            for i in 0..n {
                let j = (i + 1) % n;
                let (ti, tj) = (types[i], types[j]);
                let (vi, vj) = (poly.points[i], poly.points[j]);
                if ti != BACK {
                    f.push(vi);
                }
                if ti != FRONT {
                    b.push(vi);
                }
                if (ti | tj) == SPANNING {
                    let t = (plane.w - dot(plane.n, vi)) / dot(plane.n, sub(vj, vi));
                    let v = add(vi, scale(sub(vj, vi), t));
                    f.push(v);
                    b.push(v);
                }
            }
            if f.len() >= 3 {
                front.push(Polygon {
                    points: f,
                    plane: poly.plane,
                });
            }
            if b.len() >= 3 {
                back.push(Polygon {
                    points: b,
                    plane: poly.plane,
                });
            }
        }
    }
}

#[derive(Default)]
struct Node {
    plane: Option<Plane>,
    front: Option<Box<Node>>,
    back: Option<Box<Node>>,
    polygons: Vec<Polygon>,
}

impl Node {
    fn new(polys: Vec<Polygon>) -> Node {
        let mut n = Node::default();
        n.build(polys);
        n
    }
    fn invert(&mut self) {
        for p in &mut self.polygons {
            p.flip();
        }
        if let Some(pl) = self.plane.as_mut() {
            pl.flip();
        }
        if let Some(f) = self.front.as_mut() {
            f.invert();
        }
        if let Some(b) = self.back.as_mut() {
            b.invert();
        }
        std::mem::swap(&mut self.front, &mut self.back);
    }
    /// Remove the parts of `polys` inside this solid.
    fn clip_polygons(&self, polys: Vec<Polygon>) -> Vec<Polygon> {
        let Some(plane) = self.plane else {
            return polys;
        };
        let (mut f, mut b) = (Vec::new(), Vec::new());
        for p in &polys {
            let (mut cf, mut cb) = (Vec::new(), Vec::new());
            split(&plane, p, &mut cf, &mut cb, &mut f, &mut b);
            f.extend(cf);
            b.extend(cb);
        }
        let f = match &self.front {
            Some(n) => n.clip_polygons(f),
            None => f,
        };
        let b = match &self.back {
            Some(n) => n.clip_polygons(b),
            None => Vec::new(),
        };
        f.into_iter().chain(b).collect()
    }
    fn clip_to(&mut self, other: &Node) {
        self.polygons = other.clip_polygons(std::mem::take(&mut self.polygons));
        if let Some(f) = self.front.as_mut() {
            f.clip_to(other);
        }
        if let Some(b) = self.back.as_mut() {
            b.clip_to(other);
        }
    }
    fn all(&self) -> Vec<Polygon> {
        let mut out = self.polygons.clone();
        if let Some(f) = &self.front {
            out.extend(f.all());
        }
        if let Some(b) = &self.back {
            out.extend(b.all());
        }
        out
    }
    fn build(&mut self, polys: Vec<Polygon>) {
        if polys.is_empty() {
            return;
        }
        let plane = *self.plane.get_or_insert(polys[0].plane);
        let (mut f, mut b) = (Vec::new(), Vec::new());
        for p in &polys {
            let (mut cf, mut cb) = (Vec::new(), Vec::new());
            split(&plane, p, &mut cf, &mut cb, &mut f, &mut b);
            self.polygons.extend(cf);
            self.polygons.extend(cb);
        }
        if !f.is_empty() {
            self.front.get_or_insert_with(Default::default).build(f);
        }
        if !b.is_empty() {
            self.back.get_or_insert_with(Default::default).build(b);
        }
    }
}

fn polygons(m: &Mesh) -> Vec<Polygon> {
    m.triangles
        .iter()
        .filter_map(|t| {
            Polygon::new(vec![
                m.vertices[t[0] as usize],
                m.vertices[t[1] as usize],
                m.vertices[t[2] as usize],
            ])
        })
        .collect()
}

/// Fan-triangulate and weld (vertices closer than 1e-6 mm become one).
fn to_mesh(polys: Vec<Polygon>) -> Mesh {
    let mut m = Mesh::default();
    let mut index = std::collections::HashMap::<[i64; 3], u32>::new();
    let mut id = |p: Vec3, m: &mut Mesh| -> u32 {
        let k = [
            (p[0] * 1e6).round() as i64,
            (p[1] * 1e6).round() as i64,
            (p[2] * 1e6).round() as i64,
        ];
        *index.entry(k).or_insert_with(|| {
            m.vertices.push(p);
            (m.vertices.len() - 1) as u32
        })
    };
    for p in polys {
        let ids: Vec<u32> = p.points.iter().map(|&q| id(q, &mut m)).collect();
        for i in 1..ids.len() - 1 {
            let t = [ids[0], ids[i], ids[i + 1]];
            if t[0] != t[1] && t[1] != t[2] && t[0] != t[2] {
                m.triangles.push(t);
            }
        }
    }
    m
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    Union,
    Difference,
    Intersection,
}

/// `a ∪ b`, `a − b` or `a ∩ b`.
///
/// The BSP trees are as deep as the meshes have faces (thousands for a metal piece cut by a
/// channel), and building, clipping and dropping them recurse that deep. The host calls the
/// add-on on its UI thread, whose stack is 1 MiB on Windows: OM9 closed without a word when
/// Boolean Builder cut a ring of 18 stones. The work runs on a thread with a large stack.
pub fn boolean(a: &Mesh, b: &Mesh, op: Op) -> Mesh {
    const STACK: usize = 512 << 20;
    std::thread::scope(|scope| {
        let worker = std::thread::Builder::new()
            .name("om9-csg".into())
            .stack_size(STACK)
            .spawn_scoped(scope, || boolean_on_this_thread(a, b, op));
        match worker {
            Ok(handle) => handle
                .join()
                .unwrap_or_else(|panic| std::panic::resume_unwind(panic)),
            // No thread (resources exhausted): try on the caller's stack.
            Err(_) => boolean_on_this_thread(a, b, op),
        }
    })
}

fn boolean_on_this_thread(a: &Mesh, b: &Mesh, op: Op) -> Mesh {
    let mut na = Node::new(polygons(a));
    let mut nb = Node::new(polygons(b));
    match op {
        Op::Union => {
            na.clip_to(&nb);
            nb.clip_to(&na);
            nb.invert();
            nb.clip_to(&na);
            nb.invert();
            na.build(nb.all());
        }
        Op::Difference => {
            na.invert();
            na.clip_to(&nb);
            nb.clip_to(&na);
            nb.invert();
            nb.clip_to(&na);
            nb.invert();
            na.build(nb.all());
            na.invert();
        }
        Op::Intersection => {
            na.invert();
            nb.clip_to(&na);
            nb.invert();
            na.clip_to(&nb);
            nb.clip_to(&na);
            na.build(nb.all());
            na.invert();
        }
    }
    to_mesh(na.all())
}

/// Fold a list of meshes with one operation (`first op second op third …`).
pub fn fold(meshes: &[Mesh], op: Op) -> Option<Mesh> {
    let mut it = meshes.iter();
    let first = it.next()?.clone();
    Some(it.fold(first, |acc, m| boolean(&acc, m, op)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::{cuboid, cylinder, sphere};
    use std::f64::consts::PI;

    #[test]
    fn box_operations_have_the_right_volume() {
        let a = cuboid([0.0; 3], [2.0, 2.0, 2.0]);
        let b = cuboid([1.0, 1.0, 1.0], [3.0, 3.0, 3.0]);
        let u = boolean(&a, &b, Op::Union);
        assert!((u.volume() - 15.0).abs() < 1e-6, "{}", u.volume());
        let d = boolean(&a, &b, Op::Difference);
        assert!((d.volume() - 7.0).abs() < 1e-6, "{}", d.volume());
        let i = boolean(&a, &b, Op::Intersection);
        assert!((i.volume() - 1.0).abs() < 1e-6, "{}", i.volume());
    }

    #[test]
    fn drilling_a_hole() {
        let plate = cuboid([-5.0, -5.0, 0.0], [5.0, 5.0, 1.0]);
        let drill = cylinder(1.0, -1.0, 2.0, 64);
        let r = boolean(&plate, &drill, Op::Difference);
        let hole = std::f64::consts::PI * 1.0 * 64.0 / (2.0 * PI) * (2.0 * PI / 64.0).sin();
        assert!((r.volume() - (100.0 - hole)).abs() < 1e-3, "{}", r.volume());
        // A ball through the plate: the union grows by the two caps.
        let ball = sphere(1.5, 32);
        let u = boolean(&plate, &ball, Op::Union);
        assert!(u.volume() > plate.volume());
    }

    #[test]
    fn deep_trees_do_not_overflow_a_small_stack() {
        // A torus cut by a fine tube: thousands of faces, BSP trees thousands deep. Called
        // from a thread with the 1 MiB stack the Windows UI thread has.
        let ring = crate::mesh::torus(10.0, 2.0, 48, 12);
        let cutter = crate::mesh::torus(10.0, 1.0, 48, 12);
        let small = std::thread::Builder::new()
            .stack_size(1 << 20)
            .spawn(move || boolean(&ring, &cutter, Op::Difference))
            .unwrap();
        let cut = small.join().expect("no stack overflow");
        assert!(cut.triangles.len() > 500);
    }
}
