// SPDX-License-Identifier: LGPL-2.1-or-later
//! From Lines operates on the wire graph before any face exists. Keep unused edges so
//! MaxAutoFace=0 and the manual edge picker see exactly the same input.
use super::{Edge, Poly, edge};
use om9_addon_abi::session::{Vec3, cross, dot, norm, sub};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

type Result<T> = std::result::Result<T, &'static str>;

#[derive(Debug)]
pub struct LineNetwork {
    pub vertices: Vec<Vec3>,
    pub edges: BTreeSet<Edge>,
    /// Number of source path endpoints at each vertex, used by LineSnapping.
    pub endpoint_counts: Vec<usize>,
    neighbors: Vec<Vec<u32>>,
    closed_wires: Vec<Vec<u32>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkFace {
    pub outer: Vec<u32>,
    pub holes: Vec<Vec<u32>>,
}

/// An authored cage and the logical From Lines face for every polygon. Box and
/// Smooth displays can each contain several pieces of a single selectable face.
#[derive(Debug, PartialEq)]
pub struct LineCage {
    poly: Poly,
    face_ids: Vec<usize>,
}

impl LineCage {
    /// Reopen the actual control cage, including n-gons which were tessellated for display.
    pub fn from_polygons(poly: Poly, face_ids: Vec<usize>) -> Result<Self> {
        if poly.faces.is_empty()
            || face_ids.len() != poly.faces.len()
            || poly.verts.iter().flatten().any(|v| !v.is_finite())
            || poly
                .faces
                .iter()
                .any(|f| f.len() < 3 || f.iter().any(|&i| i as usize >= poly.verts.len()))
        {
            return Err("invalid From Lines control polygons");
        }
        Ok(Self { poly, face_ids })
    }
    pub fn poly(&self) -> &Poly {
        &self.poly
    }
    pub fn face_ids(&self) -> &[usize] {
        &self.face_ids
    }
    pub fn into_parts(self) -> (Poly, Vec<usize>) {
        (self.poly, self.face_ids)
    }
    /// Zero subdivisions is the Box display. The document mesh accepts triangles
    /// and quads, so the source keeps n-gons even when Box needs display triangles.
    pub fn display(
        &self,
        levels: usize,
        work: &mut impl FnMut(usize) -> Result<()>,
    ) -> Result<Self> {
        if self.poly.faces.len() != self.face_ids.len() {
            return Err("cage face count");
        }
        work(
            self.poly
                .verts
                .len()
                .saturating_add(self.poly.faces.iter().map(Vec::len).sum::<usize>()),
        )?;
        let mut output = Self {
            poly: self.poly.clone(),
            face_ids: self.face_ids.clone(),
        };
        for _ in 0..levels {
            let corners = output
                .poly
                .faces
                .iter()
                .try_fold(0usize, |n, f| n.checked_add(f.len()))
                .ok_or("too many cage corners")?;
            work(corners.saturating_mul(4))?;
            if output
                .poly
                .verts
                .len()
                .saturating_add(corners)
                .saturating_add(output.poly.faces.len())
                >= u32::MAX as usize
            {
                return Err("too many subdivided cage vertices");
            }
            // catmull_clark emits one quad per corner, ordered by parent polygon.
            let ids = output
                .poly
                .faces
                .iter()
                .zip(&output.face_ids)
                .flat_map(|(f, id)| std::iter::repeat_n(*id, f.len()))
                .collect();
            output.poly = super::catmull_clark(&output.poly, &BTreeSet::new());
            output.face_ids = ids;
        }
        if levels == 0 {
            let mut faces = Vec::new();
            let mut ids = Vec::new();
            for (polygon, id) in output.poly.faces.into_iter().zip(output.face_ids) {
                work(polygon.len())?;
                let source = NetworkFace {
                    outer: polygon,
                    holes: Vec::new(),
                };
                let triangles = source.triangles(&output.poly.verts, work)?;
                // The document/scene splits a quad along its 0-2 diagonal. Retain a
                // concave quad only if that diagonal is inside its boundary.
                let direct = source.outer.len() == 3
                    || source.outer.len() == 4
                        && Projection::of(&output.poly.verts, &source.outer).is_some_and(|p| {
                            let sign = crate::shapes::area2(&p.polygon).signum();
                            [[0, 1, 2], [0, 2, 3]].iter().all(|idx| {
                                sign * crate::shapes::area2(&idx.map(|i| p.polygon[i])) > 0.
                            })
                        });
                if direct {
                    faces.push(source.outer);
                    ids.push(id);
                } else {
                    for tri in triangles {
                        faces.push(tri.to_vec());
                        ids.push(id);
                    }
                }
            }
            output.poly.faces = faces;
            output.face_ids = ids;
        }
        Ok(output)
    }
}

/// Topology of the currently chosen preview faces. The wire itself stays immutable
/// while the user removes a face or picks a candidate from an edge.
#[derive(Debug)]
pub struct FaceLayout {
    network: Arc<LineNetwork>,
    faces: Vec<NetworkFace>,
    triangles: Vec<Vec<[u32; 3]>>,
    pub unused_edges: Vec<Edge>,
    pub nonmanifold_vertices: Vec<u32>,
    incidence: BTreeMap<Edge, Vec<usize>>,
    /// Separate disconnected fans at commit, retaining the original positions.
    fan: BTreeMap<(u32, usize), usize>,
}

impl FaceLayout {
    pub fn new(
        network: Arc<LineNetwork>,
        faces: Vec<NetworkFace>,
        work: &mut impl FnMut(usize) -> Result<()>,
    ) -> Result<Self> {
        let mut incidence = BTreeMap::<Edge, Vec<usize>>::new();
        let mut at_vertex = BTreeMap::<u32, BTreeSet<usize>>::new();
        let mut triangles = Vec::with_capacity(faces.len());
        for (i, face) in faces.iter().enumerate() {
            triangles.push(face.triangles(&network.vertices, work)?);
            let mut seen = BTreeSet::new();
            for ring in std::iter::once(&face.outer).chain(&face.holes) {
                work(ring.len())?;
                if ring.len() < 3 || ring.iter().any(|&v| !seen.insert(v)) {
                    return Err("invalid wire face ring");
                }
                for e in boundary(ring) {
                    if !network.edges.contains(&e) {
                        return Err("face boundary is not in the wire");
                    }
                    incidence.entry(e).or_default().push(i);
                }
            }
            for v in seen {
                at_vertex.entry(v).or_default().insert(i);
            }
        }
        let unused_edges = network
            .edges
            .iter()
            .filter(|e| !incidence.contains_key(e))
            .copied()
            .collect();
        let mut fan = BTreeMap::new();
        let mut nonmanifold_vertices = Vec::new();
        for (vertex, mut remaining) in at_vertex {
            let mut adjacent = BTreeMap::<usize, Vec<usize>>::new();
            let mut ambiguous = false;
            for &v in &network.neighbors[vertex as usize] {
                if let Some(list) = incidence.get(&edge(vertex, v)) {
                    match list[..] {
                        [a, b] => {
                            adjacent.entry(a).or_default().push(b);
                            adjacent.entry(b).or_default().push(a);
                        }
                        [_, _, _, ..] => ambiguous = true,
                        _ => {}
                    }
                }
            }
            let mut group = 0;
            while let Some(start) = remaining.pop_first() {
                let mut stack = vec![start];
                while let Some(i) = stack.pop() {
                    work(1)?;
                    fan.insert((vertex, i), group);
                    for &j in adjacent.get(&i).into_iter().flatten() {
                        if remaining.remove(&j) {
                            stack.push(j);
                        }
                    }
                }
                group += 1;
            }
            if group > 1 || ambiguous {
                nonmanifold_vertices.push(vertex);
            }
        }
        Ok(Self {
            network,
            faces,
            triangles,
            unused_edges,
            nonmanifold_vertices,
            incidence,
            fan,
        })
    }

    pub fn network(&self) -> &Arc<LineNetwork> {
        &self.network
    }
    pub fn faces(&self) -> &[NetworkFace] {
        &self.faces
    }
    pub fn triangles(&self) -> &[Vec<[u32; 3]>] {
        &self.triangles
    }

    pub fn pick_face(&self, origin: Vec3, direction: Vec3) -> Option<usize> {
        self.triangles
            .iter()
            .enumerate()
            .flat_map(|(id, triangles)| {
                triangles.iter().filter_map(move |tri| {
                    let [a, b, c] = tri.map(|i| self.network.vertices[i as usize]);
                    let (t, _) = crate::path::ray_triangle(origin, direction, a, b, c)?;
                    (t >= 0.).then_some((t, id))
                })
            })
            .min_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)))
            .map(|(_, id)| id)
    }

    /// In addition to the manual edge-count limit, the original offers a simple
    /// border cap (for example a cylinder end) whenever its naked loop is unambiguous.
    pub fn choices(
        &self,
        seed: Edge,
        max_edges: usize,
        work: &mut impl FnMut(usize) -> Result<()>,
    ) -> Result<Vec<NetworkFace>> {
        let network = &self.network;
        let seed = edge(seed.0, seed.1);
        let mut choices = network.manual_faces(seed, max_edges, work)?;
        let naked = |a, b| {
            self.incidence
                .get(&edge(a, b))
                .is_some_and(|f| f.len() == 1)
        };
        if naked(seed.0, seed.1) {
            let mut ring = vec![seed.0, seed.1];
            let mut seen = BTreeSet::from([seed.0, seed.1]);
            loop {
                let current = *ring.last().unwrap();
                let previous = ring[ring.len() - 2];
                work(network.neighbors[current as usize].len())?;
                let mut next = network.neighbors[current as usize]
                    .iter()
                    .copied()
                    .filter(|&v| v != previous && naked(current, v));
                let Some(v) = next.next() else { break };
                if next.next().is_some() {
                    break;
                }
                if v == seed.0 {
                    if ring.len() > max_edges
                        && network.neighbors[v as usize]
                            .iter()
                            .filter(|&&w| naked(v, w))
                            .count()
                            == 2
                    {
                        choices.push(ring);
                    }
                    break;
                }
                if !seen.insert(v) {
                    break;
                }
                ring.push(v);
            }
        }
        let mut result = Vec::new();
        for ring in choices {
            // Only a face on the seed can already have this same outer boundary.
            let used = self.incidence.get(&seed).into_iter().flatten().any(|&id| {
                let outer = &self.faces[id].outer;
                outer.len() == ring.len()
                    && boundary(outer).collect::<BTreeSet<_>>() == boundary(&ring).collect()
            });
            if !used {
                work(ring.len().saturating_pow(2))?;
                let Some(projection) = Projection::of(&network.vertices, &ring) else {
                    continue;
                };
                if !simple(&projection.polygon) {
                    continue;
                }
                let mut rings = vec![ring];
                let mut shapes = vec![projection];
                if shapes[0].planar {
                    for wire in &network.closed_wires {
                        work(wire.len().saturating_mul(rings[0].len()))?;
                        if wire
                            .iter()
                            .all(|&v| shapes[0].contains(network.vertices[v as usize]))
                            && let Some(p) = Projection::of(&network.vertices, wire)
                            && p.planar
                            && simple(&p.polygon)
                        {
                            rings.push(wire.clone());
                            shapes.push(p);
                        }
                    }
                }
                let nested = nest(&network.vertices, &rings, &shapes, work)?;
                if let Some(face) = nested.into_iter().find(|f| f.outer == rings[0]) {
                    result.push(face);
                }
            }
        }
        Ok(result)
    }

    /// Retain n-gons in the cage. A multiply-connected control region needs multiple
    /// polygons; the corresponding source-face IDs remain available for picking.
    pub fn cage(&self, work: &mut impl FnMut(usize) -> Result<()>) -> Result<LineCage> {
        if self.faces.is_empty() {
            return Err("no From Lines faces selected");
        }
        let mut cage = Poly::default();
        let mut source_faces = Vec::new();
        let mut ids = BTreeMap::<(u32, usize), u32>::new();
        for (i, face) in self.faces.iter().enumerate() {
            let mut insert = |ring: &[u32]| -> Result<()> {
                work(ring.len())?;
                let mut polygon = Vec::with_capacity(ring.len());
                for &v in ring {
                    let fan = self.fan[&(v, i)];
                    let id = if let Some(&id) = ids.get(&(v, fan)) {
                        id
                    } else {
                        let id = u32::try_from(cage.verts.len())
                            .map_err(|_| "too many cage vertices")?;
                        cage.verts.push(self.network.vertices[v as usize]);
                        ids.insert((v, fan), id);
                        id
                    };
                    polygon.push(id);
                }
                cage.faces.push(polygon);
                source_faces.push(i);
                Ok(())
            };
            if face.holes.is_empty() {
                insert(&face.outer)?;
            } else {
                for tri in &self.triangles[i] {
                    insert(tri)?;
                }
            }
        }
        let incidence = cage.edge_faces();
        if incidence.values().any(|owners| owners.len() > 2) {
            return Err("could not separate non-manifold edge");
        }
        let mut cage = cage.oriented();
        // Consistent winding still allows a whole closed shell to point inward.
        // Decide each component independently, relative to a point on that shell.
        let mut seen = vec![false; cage.faces.len()];
        for first in 0..cage.faces.len() {
            if seen[first] {
                continue;
            }
            let origin = cage.verts[cage.faces[first][0] as usize];
            let mut volume6 = 0.;
            let mut closed = true;
            let mut shell = vec![first];
            seen[first] = true;
            let mut index = 0;
            while index < shell.len() {
                let face = &cage.faces[shell[index]];
                work(face.len())?;
                let a = sub(cage.verts[face[0] as usize], origin);
                for pair in face[1..].windows(2) {
                    let b = sub(cage.verts[pair[0] as usize], origin);
                    let c = sub(cage.verts[pair[1] as usize], origin);
                    volume6 += dot(a, cross(b, c));
                }
                for e in boundary(face) {
                    let adjacent = &incidence[&e];
                    closed &= adjacent.len() == 2;
                    for &i in adjacent {
                        if !std::mem::replace(&mut seen[i], true) {
                            shell.push(i);
                        }
                    }
                }
                index += 1;
            }
            if closed && volume6 < 0. {
                for i in shell {
                    cage.faces[i].reverse();
                }
            }
        }
        Ok(LineCage {
            poly: cage,
            face_ids: source_faces,
        })
    }
}

fn distance(a: Vec3, b: Vec3) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1]).hypot(a[2] - b[2])
}

impl LineNetwork {
    /// Paths are ordered corner positions in the same world units as tolerance.
    /// The caller admits input storage and can cancel or limit combinatorial work.
    pub fn from_paths(
        paths: &[Vec<Vec3>],
        tolerance: f64,
        work: &mut impl FnMut(usize) -> Result<()>,
    ) -> Result<Self> {
        if !tolerance.is_finite() || tolerance <= 0. {
            return Err("From Lines needs positive tolerance");
        }
        let mut endpoints = Vec::new();
        for (i, path) in paths.iter().enumerate() {
            work(path.len())?;
            if path.len() < 2 || path.iter().flatten().any(|x| !x.is_finite()) {
                return Err("From Lines needs finite linear paths");
            }
            endpoints.extend([(path[0], i, 0), (*path.last().unwrap(), i, 1)]);
        }
        // Fixed representatives keep the result independent of selection order. A chain
        // of near misses cannot pull two vertices farther than tolerance onto one end.
        endpoints.sort_by(|a, b| {
            (0..3)
                .map(|i| a.0[i].total_cmp(&b.0[i]))
                .find(|o| !o.is_eq())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let mut joins = vec![[0; 2]; paths.len()];
        let mut buckets = BTreeMap::<[i64; 3], Vec<u32>>::new();
        let mut vertices = Vec::new();
        let mut endpoint_counts = Vec::new();
        for (p, path, end) in endpoints {
            let cell = p.map(|v| (v / tolerance).floor() as i64);
            let mut best = None::<(f64, u32)>;
            for x in cell[0].saturating_sub(1)..=cell[0].saturating_add(1) {
                for y in cell[1].saturating_sub(1)..=cell[1].saturating_add(1) {
                    for z in cell[2].saturating_sub(1)..=cell[2].saturating_add(1) {
                        if let Some(ids) = buckets.get(&[x, y, z]) {
                            work(ids.len())?;
                            for &id in ids {
                                let d = distance(vertices[id as usize], p);
                                if d <= tolerance && best.is_none_or(|pair| (d, id) < pair) {
                                    best = Some((d, id));
                                }
                            }
                        }
                    }
                }
            }
            let id = if let Some((_, id)) = best {
                id
            } else {
                let id = u32::try_from(vertices.len()).map_err(|_| "too many wire vertices")?;
                vertices.push(p);
                endpoint_counts.push(0);
                buckets.entry(cell).or_default().push(id);
                id
            };
            joins[path][end] = id;
            endpoint_counts[id as usize] += 1;
        }
        let mut edges = BTreeSet::new();
        for (path, ends) in paths.iter().zip(joins) {
            let mut last = ends[0];
            // Repeated last poles are common in explicitly closed polylines.
            let mut inner = &path[1..path.len() - 1];
            while inner.last() == path.last() && !inner.is_empty() {
                inner = &inner[..inner.len() - 1];
            }
            for &point in inner {
                if point == vertices[last as usize] {
                    continue;
                }
                let id = u32::try_from(vertices.len()).map_err(|_| "too many wire vertices")?;
                vertices.push(point);
                endpoint_counts.push(0);
                edges.insert(edge(last, id));
                last = id;
            }
            if last != ends[1] && vertices[last as usize] != vertices[ends[1] as usize] {
                edges.insert(edge(last, ends[1]));
            }
        }
        let mut neighbors = vec![Vec::new(); vertices.len()];
        for &(a, b) in &edges {
            neighbors[a as usize].push(b);
            neighbors[b as usize].push(a);
        }
        let mut visited = vec![false; vertices.len()];
        let mut closed_wires = Vec::new();
        for first in 0..neighbors.len() {
            if visited[first] || neighbors[first].len() != 2 {
                continue;
            }
            let mut ring = vec![first as u32];
            visited[first] = true;
            let mut next = neighbors[first][0];
            while neighbors[next as usize].len() == 2 && !visited[next as usize] {
                work(1)?;
                let pair = &neighbors[next as usize];
                let after = if pair[0] == *ring.last().unwrap() {
                    pair[1]
                } else {
                    pair[0]
                };
                visited[next as usize] = true;
                ring.push(next);
                next = after;
            }
            if next == first as u32 && ring.len() >= 3 {
                closed_wires.push(ring);
            }
        }
        Ok(Self {
            vertices,
            closed_wires,
            endpoint_counts,
            edges,
            neighbors,
        })
    }

    /// Enumerate simple cycles through a picked wire edge. The stack is explicit so even
    /// a long degree-one polyline can be inspected without deep machine recursion.
    pub fn manual_faces(
        &self,
        seed: Edge,
        max_edges: usize,
        work: &mut impl FnMut(usize) -> Result<()>,
    ) -> Result<Vec<Vec<u32>>> {
        let seed = edge(seed.0, seed.1);
        if !self.edges.contains(&seed) {
            return Err("unknown wire edge");
        }
        let mut faces = Vec::new();
        self.cycles(seed, max_edges, false, work, &mut |face| {
            faces.push(face.to_vec())
        })?;
        faces.sort_by(|a, b| a.len().cmp(&b.len()).then(a.cmp(b)));
        Ok(faces)
    }

    fn cycles(
        &self,
        (start, second): Edge,
        limit: usize,
        automatic: bool,
        work: &mut impl FnMut(usize) -> Result<()>,
        accept: &mut impl FnMut(&[u32]),
    ) -> Result<()> {
        if limit < 3 {
            return Ok(());
        }
        let mut path = vec![start, second];
        let mut stack = vec![0];
        let mut seen = BTreeSet::from([start, second]);
        while let Some(i) = stack.last_mut() {
            work(1)?;
            let current = *path.last().unwrap();
            let Some(&next) = self.neighbors[current as usize].get(*i) else {
                seen.remove(&path.pop().unwrap());
                stack.pop();
                continue;
            };
            *i += 1;
            if seen.contains(&next) || path.len() >= limit || (automatic && next < start) {
                continue;
            }
            let neighbors = &self.neighbors[next as usize];
            // A chord divides the automatic guess into smaller faces. Manual selection
            // still offers a larger polygon around a shared interior edge.
            if automatic
                && path[1..path.len() - 1]
                    .iter()
                    .any(|v| neighbors.binary_search(v).is_ok())
            {
                continue;
            }
            path.push(next);
            let closes = neighbors.binary_search(&start).is_ok();
            if closes && (!automatic || second < next) {
                work(path.len())?;
                accept(&path);
            }
            if automatic && closes || path.len() == limit {
                path.pop();
            } else {
                seen.insert(next);
                stack.push(0);
            }
        }
        Ok(())
    }

    /// Prefer smaller faces, as in the cube, and preserve nested coplanar boundaries as
    /// holes. Dropping MaxAutoFace to zero leaves the complete wire available for editing.
    pub fn automatic_faces(
        &self,
        max_edges: usize,
        work: &mut impl FnMut(usize) -> Result<()>,
    ) -> Result<Vec<NetworkFace>> {
        if max_edges < 3 {
            return Ok(Vec::new());
        }
        let mut loops = Vec::<Vec<u32>>::new();
        for &seed in &self.edges {
            self.cycles(seed, max_edges, true, work, &mut |face| {
                loops.push(face.to_vec())
            })?;
        }
        loops.sort_by(|a, b| a.len().cmp(&b.len()).then(a.cmp(b)));
        let mut incidence = BTreeMap::<Edge, usize>::new();
        let mut accepted = Vec::<Vec<u32>>::new();
        let cycle_edges: BTreeSet<_> = loops.iter().flat_map(|ring| boundary(ring)).collect();
        let mut shapes = Vec::new();
        for face in loops {
            if boundary(&face).any(|e| incidence.get(&e) == Some(&2)) {
                continue;
            }
            let Some(projection) = Projection::of(&self.vertices, &face) else {
                continue;
            };
            work(face.len().saturating_mul(face.len()))?;
            if !simple(&projection.polygon) {
                continue;
            }
            if projection.planar {
                let plane = &projection;
                // A chordless cycle may still surround several coplanar faces, as on
                // the perimeter of a 2x2 grid. Reject the envelope even when it has
                // fewer corners than a heavily subdivided interior region.
                let mut envelope = false;
                for &v in &face {
                    work(self.neighbors[v as usize].len())?;
                    for &w in &self.neighbors[v as usize] {
                        if self.neighbors[w as usize].len() < 2 || face.contains(&w) {
                            continue;
                        }
                        if cycle_edges.contains(&edge(v, w))
                            && plane.contains(self.vertices[w as usize])
                        {
                            envelope = true;
                            break;
                        }
                    }
                    if envelope {
                        break;
                    }
                }
                if envelope {
                    continue;
                }
            }
            for e in boundary(&face) {
                *incidence.entry(e).or_default() += 1;
            }
            shapes.push(projection);
            accepted.push(face);
        }
        nest(&self.vertices, &accepted, &shapes, work)
    }
}

fn nest(
    vertices: &[Vec3],
    accepted: &[Vec<u32>],
    shapes: &[Projection],
    work: &mut impl FnMut(usize) -> Result<()>,
) -> Result<Vec<NetworkFace>> {
    let mut parent = vec![None::<usize>; accepted.len()];
    let mut children = vec![Vec::new(); accepted.len()];
    let mut by_area: Vec<_> = (0..accepted.len()).filter(|&i| shapes[i].planar).collect();
    by_area.sort_by(|&i, &j| shapes[i].area.total_cmp(&shapes[j].area));
    for (i, inner) in accepted.iter().enumerate() {
        if !shapes[i].planar {
            continue;
        }
        let first = by_area.partition_point(|&j| shapes[j].area <= shapes[i].area);
        for &j in &by_area[first..] {
            let outer = &shapes[j];
            work(inner.len().saturating_mul(outer.polygon.len()))?;
            if inner.iter().all(|&v| outer.contains(vertices[v as usize])) {
                let projected: Vec<_> = inner
                    .iter()
                    .map(|&v| outer.project(vertices[v as usize]))
                    .collect();
                if segments(&projected)
                    .any(|(a, b)| segments(&outer.polygon).any(|(c, d)| intersects(a, b, c, d)))
                {
                    continue;
                }
                parent[i] = Some(j);
                children[j].push(inner.clone());
                break;
            }
        }
    }
    let mut result = Vec::new();
    for (i, ring) in accepted.iter().enumerate() {
        let mut depth = 0;
        let mut p = parent[i];
        while let Some(id) = p {
            work(1)?;
            depth += 1;
            if depth >= accepted.len() {
                return Err("ambiguous wire nesting");
            }
            p = parent[id];
        }
        if depth % 2 == 0 {
            result.push(NetworkFace {
                outer: ring.clone(),
                holes: std::mem::take(&mut children[i]),
            });
        }
    }
    Ok(result)
}

fn boundary(face: &[u32]) -> impl Iterator<Item = Edge> + '_ {
    face.iter()
        .zip(face.iter().cycle().skip(1))
        .map(|(&a, &b)| edge(a, b))
}

/// Local normalized coordinates avoid letting the world origin determine whether a
/// point belongs to a planar loop. The winding test excludes points on its boundary.
struct Projection {
    origin: Vec3,
    span: f64,
    normal: Vec3,
    axes: [usize; 2],
    polygon: Vec<[f64; 2]>,
    area: f64,
    planar: bool,
}

impl Projection {
    fn of(vertices: &[Vec3], ids: &[u32]) -> Option<Self> {
        let origin = vertices[ids[0] as usize];
        let mut local: Vec<_> = ids
            .iter()
            .map(|&i| sub(vertices[i as usize], origin))
            .collect();
        let span = local.iter().flatten().map(|v| v.abs()).fold(0., f64::max);
        if !span.is_finite() || span == 0. {
            return None;
        }
        for p in &mut local {
            *p = p.map(|v| v / span);
        }
        let (_, normal) = crate::shapes::loop_plane(&local)?;
        let planar = local.iter().all(|&p| dot(p, normal).abs() <= 1e-10);
        let k = (0..3).max_by(|&a, &b| normal[a].abs().total_cmp(&normal[b].abs()))?;
        let axes = [(k + 1) % 3, (k + 2) % 3];
        let polygon: Vec<_> = local.iter().map(|p| [p[axes[0]], p[axes[1]]]).collect();
        // Only used for ordering nesting candidates. Log area avoids overflow.
        let area = (crate::shapes::area2(&polygon).abs() / normal[k].abs()).ln() + 2. * span.ln();
        Some(Self {
            origin,
            span,
            normal,
            axes,
            polygon,
            area,
            planar,
        })
    }

    fn project(&self, p: Vec3) -> [f64; 2] {
        self.axes.map(|k| (p[k] - self.origin[k]) / self.span)
    }

    fn contains(&self, p: Vec3) -> bool {
        let local = sub(p, self.origin).map(|x| x / self.span);
        if dot(local, self.normal).abs() > 1e-10 {
            return false;
        }
        let [x, y] = self.axes.map(|k| local[k]);
        let mut inside = false;
        for (a, b) in self.polygon.iter().zip(self.polygon.iter().cycle().skip(1)) {
            let d = [b[0] - a[0], b[1] - a[1], 0.];
            let q = [x - a[0], y - a[1], 0.];
            if cross(d, q)[2].abs() <= 1e-12 * norm(d)
                && x >= a[0].min(b[0]) - 1e-12
                && x <= a[0].max(b[0]) + 1e-12
                && y >= a[1].min(b[1]) - 1e-12
                && y <= a[1].max(b[1]) + 1e-12
            {
                return false;
            }
            if (a[1] > y) != (b[1] > y) && x < a[0] + (y - a[1]) * d[0] / d[1] {
                inside = !inside;
            }
        }
        inside
    }
}

fn segments(points: &[[f64; 2]]) -> impl Iterator<Item = ([f64; 2], [f64; 2])> + '_ {
    points
        .iter()
        .copied()
        .zip(points.iter().copied().cycle().skip(1))
}

fn intersects(a: [f64; 2], b: [f64; 2], c: [f64; 2], d: [f64; 2]) -> bool {
    if (0..2).any(|k| a[k].max(b[k]) < c[k].min(d[k]) || c[k].max(d[k]) < a[k].min(b[k])) {
        return false;
    }
    let side = |p: [f64; 2], q: [f64; 2], r: [f64; 2]| {
        (q[0] - p[0]) * (r[1] - p[1]) - (q[1] - p[1]) * (r[0] - p[0])
    };
    side(a, b, c) * side(a, b, d) <= 0. && side(c, d, a) * side(c, d, b) <= 0.
}

fn simple(p: &[[f64; 2]]) -> bool {
    segments(p).enumerate().all(|(i, (a, b))| {
        segments(p)
            .enumerate()
            .skip(i + 2)
            .all(|(j, (c, d))| i == 0 && j + 1 == p.len() || !intersects(a, b, c, d))
    })
}

impl NetworkFace {
    /// Indexed display triangles for one logical control face. A concave contour or
    /// a hole must remain pickable as that one face across all of its triangles.
    pub fn triangles(
        &self,
        vertices: &[Vec3],
        work: &mut impl FnMut(usize) -> Result<()>,
    ) -> Result<Vec<[u32; 3]>> {
        if self.outer.len() < 3
            || self.holes.iter().any(|h| h.len() < 3)
            || self
                .outer
                .iter()
                .chain(self.holes.iter().flatten())
                .any(|&i| i as usize >= vertices.len())
        {
            return Err("invalid wire face boundary");
        }
        let plane = Projection::of(vertices, &self.outer).ok_or("degenerate wire face")?;
        let project = |id: u32| plane.project(vertices[id as usize]);
        let mut indices = BTreeMap::new();
        let key = |p: [f64; 2]| p.map(|v| if v == 0. { 0 } else { v.to_bits() });
        for &id in self.outer.iter().chain(self.holes.iter().flatten()) {
            if indices
                .insert(key(project(id)), id)
                .is_some_and(|old| old != id)
            {
                return Err("wire face overlaps in projection");
            }
        }
        let outer = &plane.polygon;
        work(outer.len().saturating_pow(2))?;
        if !simple(outer) {
            return Err("wire face crosses itself");
        }
        let holes: Vec<Vec<_>> = self
            .holes
            .iter()
            .map(|h| h.iter().copied().map(project).collect())
            .collect();
        for (i, hole) in holes.iter().enumerate() {
            work(hole.len().saturating_mul(hole.len() + outer.len()))?;
            if !simple(hole)
                || !self.holes[i]
                    .iter()
                    .all(|&v| plane.contains(vertices[v as usize]))
                || segments(hole).any(|(a, b)| segments(outer).any(|(c, d)| intersects(a, b, c, d)))
            {
                return Err("invalid wire face hole");
            }
        }
        let expected = crate::shapes::area2(outer).abs()
            - holes
                .iter()
                .map(|h| crate::shapes::area2(h).abs())
                .sum::<f64>();
        let polygon = crate::shapes::bridge_holes(outer, &holes);
        work(polygon.len().saturating_pow(3))?;
        let mut triangles = Vec::new();
        let mut area = 0.;
        let reverse = crate::shapes::area2(outer) < 0.;
        for tri in crate::shapes::triangulate(&polygon) {
            let corners = tri.map(|i| polygon[i]);
            let a = crate::shapes::area2(&corners);
            if a < -1e-14 {
                return Err("wire face has overlapping triangles");
            }
            if a <= 0. {
                continue;
            }
            let mut ids = corners.map(|p| indices[&key(p)]);
            if reverse {
                ids.swap(1, 2);
            }
            triangles.push(ids);
            area += a;
        }
        if expected <= 0. || (area - expected).abs() > 1e-10 * expected || triangles.is_empty() {
            return Err("wire face triangulation did not cover its boundaries");
        }
        Ok(triangles)
    }
}

#[cfg(test)]
mod tests;
