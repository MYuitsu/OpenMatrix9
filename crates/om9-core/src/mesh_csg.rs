// SPDX-License-Identifier: LGPL-2.1-or-later
//! Bounded floating-point surface-arrangement CSG for oriented, closed triangle meshes.
//! Owned Rust data only. No document mutation and no face-deletion fallback.
//!
//! Split by supporting planes, classify both sides by the signed solid-angle
//! winding number, retain the material boundary, then conform and triangulate it.
//! This is a tolerance kernel, not an exact-predicate or production-scale solver.
use std::collections::{BTreeMap, BTreeSet};

use crate::{
    Error, Result,
    budget::Limits,
    geometry::{Mesh, Point3},
};
mod spatial;
use spatial::{Bounds, Index};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BooleanOp {
    Union,
    Difference,
    Intersection,
}

#[derive(Debug, Clone, Copy)]
pub struct CsgOptions {
    /// Absolute world-space distance. Features below this scale are unsupported.
    pub tolerance: f64,
    pub max_triangles: usize,
    pub max_fragments: usize,
    /// Shared operation budget, including classification and shell reconstruction.
    pub max_work: usize,
}
impl Default for CsgOptions {
    fn default() -> Self {
        Self {
            tolerance: 1.0e-8,
            max_triangles: 25_000,
            max_fragments: 50_000,
            max_work: 20_000_000,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CsgReport {
    pub input_triangles: usize,
    pub split_events: usize,
    pub classified_fragments: usize,
    pub internal_fragments: usize,
    pub duplicate_fragments: usize,
    pub output_triangles: usize,
    pub work: usize,
    pub candidate_pairs: usize,
    pub ray_triangle_tests: usize,
    pub winding_fallbacks: usize,
}

#[derive(Clone, Copy)]
struct Corner {
    p: Point3,
    uv: [f64; 2],
}
impl Corner {
    fn lerp(self, b: Self, t: f64) -> Self {
        Self {
            p: add(self.p, mul(sub(b.p, self.p), t)),
            uv: [
                self.uv[0] + (b.uv[0] - self.uv[0]) * t,
                self.uv[1] + (b.uv[1] - self.uv[1]) * t,
            ],
        }
    }
}
#[derive(Clone, Copy)]
struct Plane {
    n: Point3,
    p: Point3,
}
impl Plane {
    fn of(a: Point3, b: Point3, c: Point3, eps: f64) -> Result<Self> {
        let n = cross(sub(b, a), sub(c, a));
        let length = norm(n);
        if !length.is_finite() || length <= eps * eps {
            return Err(Error::Invalid("CSG degenerate triangle"));
        }
        Ok(Self {
            n: mul(n, 1.0 / length),
            p: a,
        })
    }
    fn distance(self, p: Point3) -> f64 {
        dot(self.n, sub(p, self.p))
    }
}
#[derive(Clone)]
struct Polygon {
    corners: Vec<Corner>,
    plane: Plane,
}

struct Budget {
    remaining: usize,
    initial: usize,
    ray_triangle_tests: usize,
    winding_fallbacks: usize,
}
impl Budget {
    fn new(work: usize) -> Self {
        Self {
            remaining: work,
            initial: work,
            ray_triangle_tests: 0,
            winding_fallbacks: 0,
        }
    }
    fn spend(&mut self, count: usize) -> Result<()> {
        self.remaining = self
            .remaining
            .checked_sub(count)
            .ok_or(Error::Invalid("CSG work budget exceeded"))?;
        Ok(())
    }
}

fn add(a: Point3, b: Point3) -> Point3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn sub(a: Point3, b: Point3) -> Point3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn mul(a: Point3, s: f64) -> Point3 {
    [a[0] * s, a[1] * s, a[2] * s]
}
fn dot(a: Point3, b: Point3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn cross(a: Point3, b: Point3) -> Point3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn norm(a: Point3) -> f64 {
    a[0].hypot(a[1]).hypot(a[2])
}

fn split(poly: Polygon, plane: Plane, eps: f64) -> (Vec<Polygon>, bool) {
    let distances: Vec<_> = poly.corners.iter().map(|c| plane.distance(c.p)).collect();
    if !distances.iter().any(|d| *d > eps) || !distances.iter().any(|d| *d < -eps) {
        return (vec![poly], false);
    }
    let mut front = Vec::new();
    let mut back = Vec::new();
    for i in 0..poly.corners.len() {
        let j = (i + 1) % poly.corners.len();
        let a = poly.corners[i];
        let b = poly.corners[j];
        let da = distances[i];
        let db = distances[j];
        if da >= -eps {
            front.push(a);
        }
        if da <= eps {
            back.push(a);
        }
        if (da > eps && db < -eps) || (da < -eps && db > eps) {
            let cut = a.lerp(b, da / (da - db));
            front.push(cut);
            back.push(cut);
        }
    }
    (
        vec![
            Polygon {
                corners: front,
                plane: poly.plane,
            },
            Polygon {
                corners: back,
                plane: poly.plane,
            },
        ],
        true,
    )
}

fn topology(mesh: &Mesh, allow_empty: bool) -> Result<()> {
    if mesh.faces.is_empty() {
        return if allow_empty && mesh.vertices.is_empty() {
            Ok(())
        } else {
            Err(Error::Invalid("CSG empty input"))
        };
    }
    let mut edges: BTreeMap<(u32, u32), Vec<(usize, bool)>> = BTreeMap::new();
    let mut incident = vec![Vec::new(); mesh.vertices.len()];
    for (fi, f) in mesh.faces.iter().enumerate() {
        if f.len() != 3 {
            return Err(Error::Invalid("CSG requires triangle faces"));
        }
        for i in 0..3 {
            let a = f[i];
            let b = f[(i + 1) % 3];
            if a as usize >= mesh.vertices.len() || b as usize >= mesh.vertices.len() || a == b {
                return Err(Error::Invalid("CSG triangle indices"));
            }
            edges
                .entry((a.min(b), a.max(b)))
                .or_default()
                .push((fi, a < b));
            incident[a as usize].push(fi);
        }
    }
    if edges.values().any(|e| e.len() != 2 || e[0].1 == e[1].1) {
        return Err(Error::Invalid(
            "CSG requires closed consistently oriented mesh",
        ));
    }
    // Edge incidence alone misses two shells pinched at a vertex: require one fan.
    for (vi, faces) in incident.iter().enumerate() {
        if faces.is_empty() {
            return Err(Error::Invalid("CSG isolated vertex"));
        }
        let mut reached = BTreeSet::new();
        let mut stack = vec![faces[0]];
        while let Some(fi) = stack.pop() {
            if !reached.insert(fi) {
                continue;
            }
            for &v in &mesh.faces[fi] {
                if v as usize == vi {
                    continue;
                }
                let a = vi as u32;
                for &(neighbor, _) in &edges[&(a.min(v), a.max(v))] {
                    if !reached.contains(&neighbor) {
                        stack.push(neighbor);
                    }
                }
            }
        }
        if reached.len() != faces.len() {
            return Err(Error::Invalid("CSG nonmanifold vertex"));
        }
    }
    Ok(())
}

fn polygons(mesh: &Mesh, origin: Point3, scale: f64, eps: f64) -> Result<Vec<Polygon>> {
    let mut output = Vec::with_capacity(mesh.faces.len());
    let mut offset = 0;
    for face in &mesh.faces {
        let corners: Vec<_> = face
            .iter()
            .enumerate()
            .map(|(i, &v)| Corner {
                p: mul(sub(mesh.vertices[v as usize], origin), 1.0 / scale),
                uv: mesh
                    .corner_uvs
                    .get(offset + i)
                    .copied()
                    .unwrap_or([0.0, 0.0]),
            })
            .collect();
        offset += 3;
        let plane = Plane::of(corners[0].p, corners[1].p, corners[2].p, eps)?;
        output.push(Polygon { corners, plane });
    }
    Ok(output)
}

/// Nonzero winding fill. Negative orientation is accepted; opposite nested shells
/// define a cavity. Fractions away from an integer signal numerical ambiguity.
fn inside_solid_angle(p: Point3, triangles: &[Polygon], budget: &mut Budget) -> Result<bool> {
    budget.spend(triangles.len())?;
    let mut sum = 0.0;
    for t in triangles {
        let a = sub(t.corners[0].p, p);
        let b = sub(t.corners[1].p, p);
        let c = sub(t.corners[2].p, p);
        let la = norm(a);
        let lb = norm(b);
        let lc = norm(c);
        let numerator = dot(a, cross(b, c));
        let denominator = la * lb * lc + dot(a, b) * lc + dot(b, c) * la + dot(c, a) * lb;
        sum += 2.0 * numerator.atan2(denominator);
    }
    let w = sum / (4.0 * std::f64::consts::PI);
    if !w.is_finite() || (w - w.round()).abs() > 1.0e-4 {
        return Err(Error::Invalid("CSG ambiguous winding classification"));
    }
    Ok(w.abs() > 0.5)
}
struct Solid<'a> {
    triangles: &'a [Polygon],
    index: Index,
}
impl<'a> Solid<'a> {
    fn new(triangles: &'a [Polygon], eps: f64, budget: &mut Budget) -> Result<Self> {
        let boxes = triangles
            .iter()
            .map(|t| Bounds::of(t.corners.iter().map(|c| c.p)).expanded(eps))
            .collect();
        Ok(Self {
            triangles,
            index: Index::new(boxes, budget)?,
        })
    }
    fn inside(&self, p: Point3, eps: f64, budget: &mut Budget) -> Result<bool> {
        // For a closed oriented surface the signed ray-crossing sum equals its
        // integer winding number. No far-field approximation or parity fill.
        for direction in [
            [1., 0.371390676, 0.694746591],
            [-0.512716, 1., 0.238917],
            [0.192783, -0.414271, 1.],
        ] {
            let mut winding = 0_i64;
            let mut ambiguous = false;
            for id in self.index.along_ray(p, direction, budget)? {
                budget.spend(1)?;
                budget.ray_triangle_tests += 1;
                let t = &self.triangles[id];
                let a = t.corners[0].p;
                let e1 = sub(t.corners[1].p, a);
                let e2 = sub(t.corners[2].p, a);
                let n = cross(e1, e2);
                let area = norm(n);
                let h = cross(direction, e2);
                let det = dot(e1, h);
                if det.abs() <= 1e-12 * area {
                    ambiguous = true;
                    break;
                }
                let offset = sub(p, a);
                let u = dot(offset, h) / det;
                let q = cross(offset, e1);
                let v = dot(direction, q) / det;
                let distance = dot(e2, q) / det;
                let margin = eps * norm(e1).max(norm(e2)) / area;
                if u < -margin || v < -margin || u + v > 1. + margin || distance < -eps {
                    continue;
                }
                if u <= margin || v <= margin || 1. - u - v <= margin || distance <= eps {
                    ambiguous = true;
                    break;
                }
                winding += if dot(n, direction) > 0. { 1 } else { -1 };
            }
            if !ambiguous {
                return Ok(winding != 0);
            }
        }
        budget.winding_fallbacks += 1;
        inside_solid_angle(p, self.triangles, budget)
    }
}
fn material(
    p: Point3,
    a: &Solid<'_>,
    b: &Solid<'_>,
    op: BooleanOp,
    eps: f64,
    budget: &mut Budget,
) -> Result<bool> {
    let ia = a.inside(p, eps, budget)?;
    let ib = b.inside(p, eps, budget)?;
    Ok(match op {
        BooleanOp::Union => ia || ib,
        BooleanOp::Intersection => ia && ib,
        BooleanOp::Difference => ia && !ib,
    })
}

/// Regularized union, intersection or A-minus-B. Inputs must have triangle faces,
/// closed oriented index topology, finite coordinates, and matching UV presence.
/// Normals are invalidated; vertex colours are explicitly unsupported.
pub fn mesh_boolean(
    a: &Mesh,
    b: &Mesh,
    op: BooleanOp,
    options: CsgOptions,
) -> Result<(Mesh, CsgReport)> {
    run(a, Some(b), op, options)
}

/// Resolve self-overlap using the nonzero winding solid, without deleting a
/// triangle merely because it intersects another. Open surfaces are refused.
pub fn repair_self_intersections(mesh: &Mesh, options: CsgOptions) -> Result<(Mesh, CsgReport)> {
    run(mesh, None, BooleanOp::Union, options)
}

fn run(a: &Mesh, b: Option<&Mesh>, op: BooleanOp, o: CsgOptions) -> Result<(Mesh, CsgReport)> {
    if !o.tolerance.is_finite()
        || o.tolerance <= 0.0
        || o.max_triangles == 0
        || o.max_fragments == 0
        || o.max_work == 0
    {
        return Err(Error::Invalid("CSG options"));
    }
    let count = a
        .faces
        .len()
        .checked_add(b.map_or(0, |m| m.faces.len()))
        .ok_or(Error::Invalid("CSG triangle budget"))?;
    if count > o.max_triangles || count > o.max_fragments {
        return Err(Error::Invalid("CSG triangle budget exceeded"));
    }
    for m in std::iter::once(a).chain(b) {
        m.validate(&Limits::default())?;
        topology(m, false)?;
        if !m.colors.is_empty() {
            return Err(Error::Invalid("CSG vertex colours unsupported"));
        }
    }
    if b.is_some_and(|b| a.corner_uvs.is_empty() != b.corner_uvs.is_empty()) {
        return Err(Error::Invalid("CSG UV channel mismatch"));
    }
    let mut lo = [f64::INFINITY; 3];
    let mut hi = [f64::NEG_INFINITY; 3];
    for p in a
        .vertices
        .iter()
        .chain(b.into_iter().flat_map(|b| &b.vertices))
    {
        for i in 0..3 {
            lo[i] = lo[i].min(p[i]);
            hi[i] = hi[i].max(p[i]);
        }
    }
    let origin = add(mul(lo, 0.5), mul(hi, 0.5));
    let scale = norm(sub(hi, lo));
    let eps = o.tolerance / scale;
    let world_magnitude = lo.iter().chain(&hi).map(|x| x.abs()).fold(0.0, f64::max);
    if !scale.is_finite()
        || scale <= 0.0
        || !eps.is_finite()
        || !(1.0e-13..=1.0e-4).contains(&eps)
        || world_magnitude * f64::EPSILON * 4.0 > o.tolerance
    {
        return Err(Error::Invalid(
            "CSG coordinate scale or tolerance unsupported",
        ));
    }
    let ap = polygons(a, origin, scale, eps)?;
    let bp = match b {
        Some(b) => polygons(b, origin, scale, eps)?,
        None => Vec::new(),
    };
    let sources: Vec<_> = ap.iter().chain(&bp).cloned().collect();
    let mut report = CsgReport {
        input_triangles: count,
        ..CsgReport::default()
    };
    let mut budget = Budget::new(o.max_work);
    let a_solid = Solid::new(&ap, eps, &mut budget)?;
    let b_solid = Solid::new(&bp, eps, &mut budget)?;
    let source_bounds: Vec<_> = sources
        .iter()
        .map(|t| Bounds::of(t.corners.iter().map(|c| c.p)))
        .collect();
    let source_index = Index::new(source_bounds.clone(), &mut budget)?;
    let mut boundary = Vec::new();
    for (source_id, source) in sources.iter().enumerate() {
        let mut fragments = vec![source.clone()];
        let mut covers = Vec::new();
        for cutter_id in
            source_index.overlapping(source_bounds[source_id].expanded(eps), &mut budget)?
        {
            if cutter_id == source_id {
                continue;
            }
            budget.spend(1)?;
            report.candidate_pairs += 1;
            let cutter = &sources[cutter_id];
            let points = |t: &Polygon| [t.corners[0].p, t.corners[1].p, t.corners[2].p];
            let contact = triangle_intersection(points(source), points(cutter), eps)?;
            let planes = if matches!(contact, TriangleIntersection::Coplanar(_)) {
                if cutter_id < source_id {
                    covers.push(cutter_id);
                }
                (0..3)
                    .map(|i| {
                        let p = cutter.corners[i].p;
                        let e = sub(cutter.corners[(i + 1) % 3].p, p);
                        let n = cross(e, cutter.plane.n);
                        Plane {
                            n: mul(n, 1.0 / norm(n)),
                            p,
                        }
                    })
                    .collect::<Vec<_>>()
            } else if matches!(contact, TriangleIntersection::Segment(_)) {
                vec![cutter.plane]
            } else {
                continue;
            };
            for plane in planes {
                let mut next = Vec::new();
                for fragment in fragments {
                    budget.spend(fragment.corners.len())?;
                    let (pieces, did_split) = split(fragment, plane, eps);
                    report.split_events += usize::from(did_split);
                    next.extend(pieces);
                    if next.len() + boundary.len() > o.max_fragments {
                        return Err(Error::Invalid("CSG fragment budget exceeded"));
                    }
                }
                fragments = next;
            }
        }
        for mut f in fragments {
            report.classified_fragments += 1;
            let center = mul(
                f.corners.iter().fold([0.0; 3], |a, c| add(a, c.p)),
                1.0 / f.corners.len() as f64,
            );
            // Local arrangements can triangulate coincident overlays differently.
            // Ownership by lowest source index avoids requiring identical polygon
            // keys, and is safe because every actual overlay boundary was split.
            if covers
                .iter()
                .any(|&id| point_in_triangle(center, &sources[id], eps))
            {
                report.duplicate_fragments += 1;
                continue;
            }
            let nearby =
                source_index.overlapping(Bounds::of([center]).expanded(eps * 16.), &mut budget)?;
            let delta = probe_distance(
                center,
                f.plane.n,
                eps,
                nearby.into_iter().map(|id| &sources[id]),
                &mut budget,
            )?;
            let negative = material(
                sub(center, mul(f.plane.n, delta)),
                &a_solid,
                &b_solid,
                op,
                eps,
                &mut budget,
            )?;
            let positive = material(
                add(center, mul(f.plane.n, delta)),
                &a_solid,
                &b_solid,
                op,
                eps,
                &mut budget,
            )?;
            if negative == positive {
                report.internal_fragments += 1;
                continue;
            }
            if positive {
                f.corners.reverse();
                f.plane.n = mul(f.plane.n, -1.0);
            }
            boundary.push(f);
            if boundary.len() > o.max_fragments {
                return Err(Error::Invalid("CSG fragment budget exceeded"));
            }
        }
    }
    let mut mesh = rebuild(
        boundary,
        eps,
        !a.corner_uvs.is_empty(),
        o.max_fragments,
        &mut budget,
        &mut report,
    )?;
    topology(&mesh, true)?;
    verify_embedding(&mesh, eps, &mut budget)?;
    for p in &mut mesh.vertices {
        *p = add(origin, mul(*p, scale));
    }
    mesh.validate(&Limits::default())?;
    report.output_triangles = mesh.faces.len();
    report.work = budget.initial - budget.remaining;
    report.ray_triangle_tests = budget.ray_triangle_tests;
    report.winding_fallbacks = budget.winding_fallbacks;
    Ok((mesh, report))
}

/// Pick a displacement within a certified empty normal segment. If another
/// triangle is hit at signed distance t, both probes stay at most |t|/4 away
/// from the center. Thus they cannot cross it; infinite-plane proximity alone
/// is not a material wall. The distance tolerance and minimum probe stay intact.
fn probe_distance<'a>(
    center: Point3,
    normal: Point3,
    eps: f64,
    nearby: impl Iterator<Item = &'a Polygon>,
    budget: &mut Budget,
) -> Result<f64> {
    let mut delta = eps * 8.;
    for t in nearby {
        budget.spend(1)?;
        let distance = t.plane.distance(center);
        let alignment = dot(t.plane.n, normal);
        if distance.abs() > eps && alignment.abs() > 1e-12 {
            let hit_distance = -distance / alignment;
            let hit = add(center, mul(normal, hit_distance));
            if point_in_triangle(hit, t, eps) {
                delta = delta.min(hit_distance.abs() * 0.25);
            }
        }
    }
    if delta < eps * 2. {
        return Err(Error::Invalid("CSG feature below tolerance"));
    }
    Ok(delta)
}

fn point_in_triangle(p: Point3, t: &Polygon, eps: f64) -> bool {
    let a = t.corners[0].p;
    let u = sub(t.corners[1].p, a);
    let v = sub(t.corners[2].p, a);
    let w = sub(p, a);
    let uu = dot(u, u);
    let uv = dot(u, v);
    let vv = dot(v, v);
    let wu = dot(w, u);
    let wv = dot(w, v);
    let denominator = uu * vv - uv * uv;
    let s = (vv * wu - uv * wv) / denominator;
    let r = (uu * wv - uv * wu) / denominator;
    let margin = eps * norm(u).max(norm(v)) / norm(cross(u, v));
    s >= -margin && r >= -margin && s + r <= 1. + margin
}

struct Welder {
    points: Vec<Point3>,
    cells: BTreeMap<[i64; 3], Vec<u32>>,
    eps: f64,
}
impl Welder {
    fn new(eps: f64) -> Self {
        Self {
            points: Vec::new(),
            cells: BTreeMap::new(),
            eps,
        }
    }
    fn insert(&mut self, p: Point3) -> Result<u32> {
        let key = p.map(|x| (x / self.eps).floor() as i64);
        for x in -1..=1 {
            for y in -1..=1 {
                for z in -1..=1 {
                    if let Some(ids) = self.cells.get(&[key[0] + x, key[1] + y, key[2] + z]) {
                        for &id in ids {
                            if norm(sub(self.points[id as usize], p)) <= self.eps {
                                return Ok(id);
                            }
                        }
                    }
                }
            }
        }
        let id =
            u32::try_from(self.points.len()).map_err(|_| Error::Invalid("CSG vertex budget"))?;
        self.points.push(p);
        self.cells.entry(key).or_default().push(id);
        Ok(id)
    }
}

fn rebuild(
    polygons: Vec<Polygon>,
    eps: f64,
    uv: bool,
    max_faces: usize,
    budget: &mut Budget,
    report: &mut CsgReport,
) -> Result<Mesh> {
    let mut weld = Welder::new(eps);
    let mut rings = Vec::new();
    let mut seen = BTreeSet::new();
    for poly in polygons {
        let mut ring = Vec::new();
        for c in poly.corners {
            let id = weld.insert(c.p)?;
            if ring.last().is_none_or(|(i, _)| *i != id) {
                ring.push((id, c.uv));
            }
        }
        if ring.len() > 1 && ring[0].0 == ring[ring.len() - 1].0 {
            ring.pop();
        }
        if ring.len() < 3 {
            return Err(Error::Invalid("CSG collapsed boundary fragment"));
        }
        let mut key: Vec<_> = ring.iter().map(|(id, _)| *id).collect();
        key.sort_unstable();
        if seen.insert(key) {
            rings.push(ring);
        } else {
            report.duplicate_fragments += 1;
        }
    }
    let boundary_points = weld.points.clone();
    let point_index = Index::new(
        boundary_points.iter().map(|&p| Bounds::of([p])).collect(),
        budget,
    )?;
    let mut faces = Vec::new();
    let mut uvs = Vec::new();
    for ring in rings {
        // Insert all boundary vertices on every edge to eliminate T junctions.
        let mut conform = Vec::new();
        for i in 0..ring.len() {
            let (ai, au) = ring[i];
            let (bi, bu) = ring[(i + 1) % ring.len()];
            let a = boundary_points[ai as usize];
            let b = boundary_points[bi as usize];
            let e = sub(b, a);
            let length2 = dot(e, e);
            let mut edge = vec![(0.0, ai, au)];
            for id in point_index.overlapping(Bounds::of([a, b]).expanded(eps), budget)? {
                let p = boundary_points[id];
                if id == ai as usize || id == bi as usize {
                    continue;
                }
                let t = dot(sub(p, a), e) / length2;
                if t > 0.0 && t < 1.0 && norm(sub(p, add(a, mul(e, t)))) <= eps {
                    edge.push((
                        t,
                        id as u32,
                        [au[0] + (bu[0] - au[0]) * t, au[1] + (bu[1] - au[1]) * t],
                    ));
                }
            }
            edge.sort_by(|a, b| a.0.total_cmp(&b.0));
            conform.extend(edge.into_iter().map(|(_, id, uv)| (id, uv)));
        }
        if conform.len() == 3 {
            let ids: Vec<_> = conform.iter().map(|&(id, _)| id).collect();
            let p = |i| boundary_points[ids[i] as usize];
            if norm(cross(sub(p(1), p(0)), sub(p(2), p(0)))) <= eps * eps {
                return Err(Error::Invalid("CSG degenerate output triangle"));
            }
            faces.push(ids);
            if uv {
                uvs.extend(conform.iter().map(|&(_, uv)| uv));
            }
            if faces.len() > max_faces {
                return Err(Error::Invalid("CSG output budget exceeded"));
            }
            continue;
        }
        let count = conform.len() as f64;
        let mut center = [0.0; 3];
        let mut center_uv = [0.0; 2];
        for &(id, u) in &conform {
            center = add(center, mul(boundary_points[id as usize], 1.0 / count));
            for i in 0..2 {
                center_uv[i] += u[i] / count;
            }
        }
        let ci = weld.insert(center)?;
        for i in 0..conform.len() {
            let (ai, au) = conform[i];
            let (bi, bu) = conform[(i + 1) % conform.len()];
            if ci == ai
                || ci == bi
                || norm(cross(
                    sub(weld.points[ai as usize], center),
                    sub(weld.points[bi as usize], center),
                )) <= eps * eps
            {
                return Err(Error::Invalid("CSG degenerate output triangle"));
            }
            faces.push(vec![ai, bi, ci]);
            if uv {
                uvs.extend([au, bu, center_uv]);
            }
            if faces.len() > max_faces {
                return Err(Error::Invalid("CSG output budget exceeded"));
            }
        }
    }
    Ok(Mesh {
        vertices: weld.points,
        faces,
        corner_uvs: uvs,
        normals: Vec::new(),
        colors: Vec::new(),
    })
}

/// Intersection segment of two non-coplanar triangles. Coplanar contact has no
/// unique segment: the intersection polygon is returned for positive-area
/// coplanar overlap. Coplanar point/edge contact uses Point/Segment.
#[derive(Debug, Clone, PartialEq)]
pub enum TriangleIntersection {
    Disjoint,
    Point(Point3),
    Segment([Point3; 2]),
    Coplanar(Vec<Point3>),
}

pub fn triangle_intersection(
    a: [Point3; 3],
    b: [Point3; 3],
    tolerance: f64,
) -> Result<TriangleIntersection> {
    if !tolerance.is_finite()
        || tolerance <= 0.0
        || a.iter().chain(&b).flatten().any(|x| !x.is_finite())
    {
        return Err(Error::Invalid("triangle intersection input"));
    }
    let pa = Plane::of(a[0], a[1], a[2], tolerance)?;
    let pb = Plane::of(b[0], b[1], b[2], tolerance)?;
    let direction = cross(pa.n, pb.n);
    let length = norm(direction);
    if a.iter().all(|&p| pb.distance(p).abs() <= tolerance)
        && b.iter().all(|&p| pa.distance(p).abs() <= tolerance)
    {
        let mut points = a.to_vec();
        for i in 0..3 {
            let n = cross(sub(b[(i + 1) % 3], b[i]), pb.n);
            let plane = Plane {
                n: mul(n, 1.0 / norm(n)),
                p: b[i],
            };
            let mut next = Vec::new();
            for j in 0..points.len() {
                let p = points[j];
                let q = points[(j + 1) % points.len()];
                let dp = plane.distance(p);
                let dq = plane.distance(q);
                if dp <= tolerance {
                    next.push(p);
                }
                if (dp > tolerance && dq < -tolerance) || (dp < -tolerance && dq > tolerance) {
                    next.push(add(p, mul(sub(q, p), dp / (dp - dq))));
                }
            }
            points = next;
        }
        let mut unique = Vec::new();
        for p in points {
            if !unique.iter().any(|&q| norm(sub(p, q)) <= tolerance) {
                unique.push(p);
            }
        }
        if unique.is_empty() {
            return Ok(TriangleIntersection::Disjoint);
        }
        let area = (1..unique.len().saturating_sub(1))
            .map(|i| {
                norm(cross(
                    sub(unique[i], unique[0]),
                    sub(unique[i + 1], unique[0]),
                ))
            })
            .sum::<f64>();
        let perimeter_scale = unique
            .iter()
            .map(|&p| norm(sub(p, unique[0])))
            .fold(0.0, f64::max);
        if area > tolerance * perimeter_scale * 2.0 {
            return Ok(TriangleIntersection::Coplanar(unique));
        }
        let mut far = (unique[0], unique[0], 0.0);
        for &p in &unique {
            for &q in &unique {
                let length = norm(sub(p, q));
                if length > far.2 {
                    far = (p, q, length);
                }
            }
        }
        return Ok(if far.2 <= tolerance {
            TriangleIntersection::Point(far.0)
        } else {
            TriangleIntersection::Segment([far.0, far.1])
        });
    }
    if length <= 1.0e-12 {
        return Ok(TriangleIntersection::Disjoint);
    }
    let direction = mul(direction, 1.0 / length);
    let section = |tri: [Point3; 3], plane: Plane| {
        let mut points = Vec::new();
        for i in 0..3 {
            let j = (i + 1) % 3;
            let da = plane.distance(tri[i]);
            let db = plane.distance(tri[j]);
            if da.abs() <= tolerance {
                points.push(tri[i]);
            }
            if da * db < 0.0 {
                points.push(add(tri[i], mul(sub(tri[j], tri[i]), da / (da - db))));
            }
        }
        points.sort_by(|a, b| dot(*a, direction).total_cmp(&dot(*b, direction)));
        points
    };
    let sa = section(a, pb);
    let sb = section(b, pa);
    if sa.is_empty() || sb.is_empty() {
        return Ok(TriangleIntersection::Disjoint);
    }
    let origin = sa[0];
    let project = |p: Point3| dot(sub(p, origin), direction);
    let lo = project(sa[0]).max(project(sb[0]));
    let hi = project(*sa.last().unwrap()).min(project(*sb.last().unwrap()));
    if hi < lo - tolerance {
        return Ok(TriangleIntersection::Disjoint);
    }
    if hi - lo <= tolerance {
        return Ok(TriangleIntersection::Point(add(
            origin,
            mul(direction, (hi + lo) * 0.5),
        )));
    }
    Ok(TriangleIntersection::Segment([
        add(origin, mul(direction, lo)),
        add(origin, mul(direction, hi)),
    ]))
}

/// Check a triangle pair for intersection beyond their shared indexed boundary.
/// Used for publication validation and the existing repair diagnostic.
pub(crate) fn improper_intersection(
    a: [Point3; 3],
    b: [Point3; 3],
    shared: &[Point3],
    eps: f64,
) -> Result<bool> {
    for i in 0..3 {
        let amin = a.iter().map(|p| p[i]).fold(f64::INFINITY, f64::min);
        let amax = a.iter().map(|p| p[i]).fold(f64::NEG_INFINITY, f64::max);
        let bmin = b.iter().map(|p| p[i]).fold(f64::INFINITY, f64::min);
        let bmax = b.iter().map(|p| p[i]).fold(f64::NEG_INFINITY, f64::max);
        if amin > bmax + eps || bmin > amax + eps {
            return Ok(false);
        }
    }
    Ok(match triangle_intersection(a, b, eps)? {
        TriangleIntersection::Disjoint => false,
        TriangleIntersection::Coplanar(_) => true,
        TriangleIntersection::Point(p) => !shared.iter().any(|&q| norm(sub(p, q)) <= eps * 2.0),
        TriangleIntersection::Segment([p, q]) => {
            if shared.len() != 2 {
                true
            } else {
                let e = sub(shared[1], shared[0]);
                let length2 = dot(e, e);
                [p, q].iter().any(|&p| {
                    let t = dot(sub(p, shared[0]), e) / length2;
                    t < -eps || t > 1.0 + eps || norm(sub(p, add(shared[0], mul(e, t)))) > eps * 2.0
                })
            }
        }
    })
}

fn verify_embedding(mesh: &Mesh, eps: f64, budget: &mut Budget) -> Result<()> {
    let boxes: Vec<_> = mesh
        .faces
        .iter()
        .map(|f| Bounds::of(f.iter().map(|&id| mesh.vertices[id as usize])))
        .collect();
    let index = Index::new(boxes.clone(), budget)?;
    for (i, a) in mesh.faces.iter().enumerate() {
        for j in index.overlapping(boxes[i].expanded(eps), budget)? {
            if j <= i {
                continue;
            }
            let b = &mesh.faces[j];
            budget.spend(1)?;
            let shared: Vec<_> = a
                .iter()
                .filter(|v| b.contains(v))
                .map(|&v| mesh.vertices[v as usize])
                .collect();
            let points = |f: &Vec<u32>| {
                [
                    mesh.vertices[f[0] as usize],
                    mesh.vertices[f[1] as usize],
                    mesh.vertices[f[2] as usize],
                ]
            };
            if improper_intersection(points(a), points(b), &shared, eps)? {
                return Err(Error::Invalid("CSG output still intersects"));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod acceleration_tests {
    use super::*;

    fn triangle(points: [Point3; 3], eps: f64) -> Polygon {
        Polygon {
            plane: Plane::of(points[0], points[1], points[2], eps).unwrap(),
            corners: points.map(|p| Corner { p, uv: [0.; 2] }).to_vec(),
        }
    }

    #[test]
    fn finite_triangle_probe_clearance_fixes_the_curved_witness() {
        // Extracted from source 3192 of the unchanged authored 16K sphere fixture.
        let eps = 2.581481256071081e-9;
        let center = [
            0.06189469512490915,
            -0.24103943921394494,
            0.023789793795686097,
        ];
        let normal = [0.469686035165928, -0.8723570509411301, 0.13560311221872878];
        let wall = triangle(
            [
                [
                    0.07333153767319336,
                    -0.24105754671979965,
                    0.011616665652319853,
                ],
                [0.073346776768188, -0.24074472052469947, 0.02429863290283747],
                [
                    0.0606648095176704,
                    -0.24105754671979965,
                    0.02428339380784283,
                ],
            ],
            eps,
        );
        let plane_distance = wall.plane.distance(center);
        assert!(plane_distance.abs() > eps * 12.);
        assert!(plane_distance.abs() * 0.125 < eps * 2.); // The old false rejection.
        let mut budget = Budget::new(100);
        let delta =
            probe_distance(center, normal, eps, std::iter::once(&wall), &mut budget).unwrap();
        let hit_distance = -plane_distance / dot(wall.plane.n, normal);
        assert!(delta >= eps * 2. && delta <= eps * 8.);
        assert!(delta <= hit_distance.abs() * 0.25 * (1. + f64::EPSILON));
        for sign in [-1., 1.] {
            let start = wall.plane.distance(center);
            let end = wall.plane.distance(add(center, mul(normal, sign * delta)));
            assert!(
                start * end > 0.,
                "probe must stay on its starting side of the wall"
            );
        }
    }

    #[test]
    fn probe_ignores_remote_plane_but_still_refuses_inadequate_clearance() {
        let eps = 1e-8;
        let remote = triangle(
            [
                [10., 10., 3. * eps],
                [11., 10., 3. * eps],
                [10., 11., 3. * eps],
            ],
            eps,
        );
        let close = triangle(
            [
                [-1., -1., 4. * eps],
                [1., -1., 4. * eps],
                [0., 1., 4. * eps],
            ],
            eps,
        );
        let mut budget = Budget::new(100);
        assert_eq!(
            probe_distance(
                [0.; 3],
                [0., 0., 1.],
                eps,
                std::iter::once(&remote),
                &mut budget
            )
            .unwrap(),
            eps * 8.
        );
        assert!(
            probe_distance(
                [0.; 3],
                [0., 0., 1.],
                eps,
                std::iter::once(&close),
                &mut budget
            )
            .is_err()
        );
    }

    #[test]
    fn signed_bvh_rays_match_solid_angles_with_overlap_and_cavity_orientation() {
        let eps = 1e-9;
        for reverse in [false, true] {
            let mut triangles = Vec::new();
            for (shift, inward) in [([0., 0., 0.], false), ([0.3, 0.2, 0.1], reverse)] {
                let points =
                    [[0., 0., 0.], [2., 0., 0.], [0., 2., 0.], [0., 0., 2.]].map(|p| add(p, shift));
                for mut ids in [[0, 2, 1], [0, 1, 3], [0, 3, 2], [1, 2, 3]] {
                    if inward {
                        ids.reverse();
                    }
                    let corners: Vec<_> = ids
                        .map(|i| Corner {
                            p: points[i],
                            uv: [0.; 2],
                        })
                        .to_vec();
                    let plane = Plane::of(corners[0].p, corners[1].p, corners[2].p, eps).unwrap();
                    triangles.push(Polygon { corners, plane });
                }
            }
            let mut budget = Budget::new(1_000_000);
            let solid = Solid::new(&triangles, eps, &mut budget).unwrap();
            for i in 0..9 {
                for j in 0..9 {
                    for k in 0..9 {
                        let p = [
                            i as f64 * 0.31 - 0.123,
                            j as f64 * 0.29 - 0.119,
                            k as f64 * 0.27 - 0.113,
                        ];
                        assert_eq!(
                            solid.inside(p, eps, &mut budget).unwrap(),
                            inside_solid_angle(p, &triangles, &mut budget).unwrap(),
                            "point {p:?}, reverse={reverse}"
                        );
                    }
                }
            }
        }
    }
}
