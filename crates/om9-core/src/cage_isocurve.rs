// SPDX-License-Identifier: LGPL-2.1-or-later
//! Single-span isocurves of uncreased, regular quad cages, including open grid borders.
use crate::{
    Error, Result,
    budget::{Limits, Usage},
    geometry::{Mesh, NurbsCurve, NurbsSurface, Point3},
    surface_kernel::{self, SurfaceIso},
};

const IRREGULAR: Error = Error::Unsupported("Extract Edges requires a regular quad neighborhood");

#[derive(Clone, Default)]
struct Fan {
    faces: [usize; 4],
    count: u8,
}

/// Incidence is built once per source cage. High-valence vertices elsewhere in the
/// same mesh do not prevent extraction from an ordinary neighborhood.
pub struct CagePatches<'a> {
    mesh: &'a Mesh,
    fans: Vec<Fan>,
}

impl<'a> CagePatches<'a> {
    pub fn new(mesh: &'a Mesh, limits: &Limits) -> Result<Self> {
        mesh.validate(limits)?
            .add(Usage::array::<Fan>(mesh.vertices.len())?, limits)?;
        let mut fans = Vec::new();
        fans.try_reserve_exact(mesh.vertices.len())
            .map_err(|_| Error::Budget("cage incidence"))?;
        fans.resize(mesh.vertices.len(), Fan::default());
        for (id, face) in mesh.faces.iter().enumerate() {
            for &v in face {
                let fan = &mut fans[v as usize];
                if let Some(slot) = fan.faces.get_mut(usize::from(fan.count)) {
                    *slot = id;
                }
                fan.count = fan.count.saturating_add(1);
            }
        }
        Ok(Self { mesh, fans })
    }

    fn incident(&self, v: u32) -> Result<&[usize]> {
        let fan = self
            .fans
            .get(v as usize)
            .ok_or(Error::Invalid("cage vertex"))?;
        fan.faces.get(..usize::from(fan.count)).ok_or(IRREGULAR)
    }

    // The target edge is oriented a -> b. Accept either polygon winding and
    // return its other two vertices in matching order (a, b, c, d).
    fn quad(&self, id: usize, a: u32, b: u32) -> Result<Option<([u32; 4], bool)>> {
        let face = &self.mesh.faces[id];
        let at = face.iter().position(|v| *v == a).ok_or(IRREGULAR)?;
        let n = face.len();
        let forward = face[(at + 1) % n] == b;
        if !forward && face[(at + n - 1) % n] != b {
            return Ok(None);
        }
        if n != 4 {
            return Err(IRREGULAR);
        }
        let step = if forward { 1 } else { 3 };
        Ok(Some((
            std::array::from_fn(|i| face[(at + step * i) % 4]),
            forward,
        )))
    }

    fn patch(&self, id: usize, q: [u32; 4]) -> Result<NurbsSurface> {
        // Indexed along the selected edge (u), then inward (v).
        let mut net = [[[0.; 3]; 4]; 4];
        let corners = [(1, 1), (2, 1), (2, 2), (1, 2)];
        let wings = [
            ((1, 0), (2, 0)),
            ((3, 1), (3, 2)),
            ((2, 3), (1, 3)),
            ((0, 2), (0, 1)),
        ];
        let diagonals = [(0, 0), (3, 0), (3, 3), (0, 3)];
        let mut neighbors = [None; 4];
        let mut wing_ids = [[0u32; 2]; 4];
        for i in 0..4 {
            let a = q[i];
            let b = q[(i + 1) % 4];
            let (u, v) = corners[i];
            net[u][v] = self.mesh.vertices[a as usize];
            for &other in self.incident(a)? {
                if other == id {
                    continue;
                }
                if let Some(([_, _, c, d], _)) = self.quad(other, a, b)? {
                    if neighbors[i].replace(other).is_some() {
                        return Err(IRREGULAR);
                    }
                    wing_ids[i] = [d, c];
                    for ((u, v), vertex) in [(wings[i].0, d), (wings[i].1, c)] {
                        net[u][v] = self.mesh.vertices[vertex as usize];
                    }
                }
            }
        }
        for i in 0..4 {
            let fan = self.incident(q[i])?;
            let prev = neighbors[(i + 3) % 4];
            let next = neighbors[i];
            let expected = (1 + usize::from(prev.is_some())) * (1 + usize::from(next.is_some()));
            if fan.len() != expected {
                return Err(IRREGULAR);
            }
            if let (Some(left), Some(right)) = (prev, next) {
                let diag = fan
                    .iter()
                    .copied()
                    .find(|f| ![id, left, right].contains(f))
                    .ok_or(IRREGULAR)?;
                let face = &self.mesh.faces[diag];
                if face.len() != 4 {
                    return Err(IRREGULAR);
                }
                let k = face.iter().position(|v| *v == q[i]).ok_or(IRREGULAR)?;
                let adjacent = [face[(k + 3) % 4], face[(k + 1) % 4]];
                let across = [wing_ids[(i + 3) % 4][1], wing_ids[i][0]];
                if adjacent != across && adjacent != [across[1], across[0]] {
                    return Err(IRREGULAR);
                }
                let (u, v) = diagonals[i];
                net[u][v] = self.mesh.vertices[face[(k + 2) % 4] as usize];
            }
        }
        // Linear extension across each open border gives the cubic boundary
        // spline and fixes a one-face corner. At two borders extend both axes.
        let reflect = |a: Point3, b: Point3| std::array::from_fn(|i| a[i] + (a[i] - b[i]));
        for (side, out, near, far) in [(3, 0, 1, 2), (1, 3, 2, 1)] {
            if neighbors[side].is_none() {
                net[out] = std::array::from_fn(|v| reflect(net[near][v], net[far][v]));
            }
        }
        for (side, out, near, far) in [(0, 0, 1, 2), (2, 3, 2, 1)] {
            if neighbors[side].is_none() {
                for row in &mut net {
                    row[out] = reflect(row[near], row[far]);
                }
            }
        }
        Ok(NurbsSurface {
            degree: [3; 2],
            size: [4; 2],
            poles: net.into_iter().flatten().collect(),
            weights: vec![1.; 16],
            knots: std::array::from_fn(|_| (-3..=4).map(f64::from).collect()),
            domain: [[0., 1.]; 2],
            periodic: [false; 2],
            reversed: false,
        })
    }

    /// Extract one selected edge interval. Positive offsets go into the face
    /// whose winding follows ascending edge IDs. A lone face is the inward side.
    pub fn edge_span(&self, edge: [u32; 2]) -> Result<EdgeSpan> {
        if edge[0] == edge[1] {
            return Err(Error::Invalid("cage edge"));
        }
        let a = edge[0].min(edge[1]);
        let b = edge[0].max(edge[1]);
        let mut sides = [None; 2];
        for &id in self.incident(a)? {
            if let Some((q, forward)) = self.quad(id, a, b)? {
                if sides[usize::from(!forward)].replace((id, q)).is_some() {
                    return Err(IRREGULAR);
                }
            }
        }
        let (pos, neg) = match sides {
            [Some(pos), neg] => (pos, neg),
            [None, Some(pos)] => (pos, None),
            [None, None] => return Err(Error::Invalid("select a source cage edge")),
        };
        Ok(EdgeSpan {
            positive: self.patch(pos.0, pos.1)?,
            negative: neg.map(|(id, q)| self.patch(id, q)).transpose()?,
        })
    }
}

#[derive(Debug)]
pub struct EdgeSpan {
    positive: NurbsSurface,
    negative: Option<NurbsSurface>,
}

impl EdgeSpan {
    pub fn curve(&self, offset: f64, limits: &Limits) -> Result<NurbsCurve> {
        if !(-1.0..=1.0).contains(&offset) {
            return Err(Error::Invalid("enter an offset between -1 and 1"));
        }
        let surface = if offset < 0. {
            self.negative.as_ref().ok_or(Error::Unsupported(
                "no adjacent patch outside this cage border",
            ))?
        } else {
            &self.positive
        };
        surface_kernel::isocurve(surface, SurfaceIso::U, offset.abs(), limits)
    }

    /// Actual pickable Box-mode cage segments at both ends of the selected interval.
    pub fn perpendicular_edges(&self) -> impl Iterator<Item = (f64, [Point3; 2])> + '_ {
        std::iter::once((1., &self.positive))
            .chain(self.negative.as_ref().map(|s| (-1., s)))
            .flat_map(|(sign, s)| [1, 2].map(|u| (sign, [s.poles[u * 4 + 1], s.poles[u * 4 + 2]])))
    }
}
