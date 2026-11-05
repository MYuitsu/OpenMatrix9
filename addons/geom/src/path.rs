// SPDX-License-Identifier: LGPL-2.1-or-later
//! Polylines as paths (arc length, points and frames along them) and meshes as targets
//! (rays down onto a surface, closest points), for flows, arrays and projections.
use crate::mesh::Frame;
use om9_addon_abi::session::{SelectedMesh, Vec3, add, cross, dist, dot, norm, scale, sub, unit};

/// A sampled curve with its cumulative arc length.
#[derive(Debug, Clone)]
pub struct Path {
    pub points: Vec<Vec3>,
    pub closed: bool,
    lengths: Vec<f64>,
}

impl Path {
    pub fn new(points: &[Vec3], closed: bool) -> Option<Path> {
        let mut pts = points.to_vec();
        pts.dedup_by(|a, b| dist(*a, *b) < 1e-12);
        if closed && pts.len() > 2 && dist(pts[0], *pts.last()?) > 1e-12 {
            pts.push(pts[0]);
        }
        if pts.len() < 2 {
            return None;
        }
        let mut lengths = vec![0.0];
        for w in pts.windows(2) {
            let l = lengths.last().copied()? + dist(w[0], w[1]);
            lengths.push(l);
        }
        (lengths.last().copied()? > 1e-12).then_some(Path {
            points: pts,
            closed,
            lengths,
        })
    }
    pub fn length(&self) -> f64 {
        *self.lengths.last().unwrap()
    }
    fn segment(&self, s: f64) -> (usize, f64) {
        let s = if self.closed {
            s.rem_euclid(self.length())
        } else {
            s.clamp(0.0, self.length())
        };
        let i = match self.lengths.binary_search_by(|l| l.total_cmp(&s)) {
            Ok(i) => i.min(self.points.len() - 2),
            Err(i) => i.saturating_sub(1).min(self.points.len() - 2),
        };
        let l = self.lengths[i + 1] - self.lengths[i];
        (
            i,
            if l > 0.0 {
                (s - self.lengths[i]) / l
            } else {
                0.0
            },
        )
    }
    /// Point at arc length `s`.
    pub fn at(&self, s: f64) -> Vec3 {
        let (i, t) = self.segment(s);
        add(
            self.points[i],
            scale(sub(self.points[i + 1], self.points[i]), t),
        )
    }
    /// Unit tangent at arc length `s` (averaged over a short window so corners turn smoothly).
    pub fn tangent(&self, s: f64) -> Vec3 {
        let h = (self.length() * 1e-3).max(1e-6);
        let (a, b) = if self.closed {
            (self.at(s - h), self.at(s + h))
        } else {
            (
                self.at((s - h).max(0.0)),
                self.at((s + h).min(self.length())),
            )
        };
        unit(sub(b, a))
    }
    /// The centroid of the samples (for "outward" directions of closed curves).
    pub fn centroid(&self) -> Vec3 {
        let n = self.points.len() as f64;
        scale(
            self.points.iter().fold([0.0; 3], |a, &p| add(a, p)),
            1.0 / n,
        )
    }
    /// Frame along the path: x = tangent, z = `up` made perpendicular (for closed curves
    /// `None` means outward from the centroid), y = z × x.
    pub fn frame(&self, s: f64, up: Option<Vec3>) -> Frame {
        let p = self.at(s);
        let t = self.tangent(s);
        let up = up.unwrap_or_else(|| sub(p, self.centroid()));
        let mut z = sub(up, scale(t, dot(up, t)));
        if norm(z) < 1e-9 {
            z = cross(t, [0.0, 0.0, 1.0]);
        }
        Frame::new(p, z, t)
    }
    /// Arc length of the point on the path nearest to `p`.
    pub fn closest(&self, p: Vec3) -> f64 {
        let mut best = (f64::MAX, 0.0);
        for i in 0..self.points.len() - 1 {
            let (a, b) = (self.points[i], self.points[i + 1]);
            let ab = sub(b, a);
            let l2 = dot(ab, ab);
            let t = if l2 > 0.0 {
                (dot(sub(p, a), ab) / l2).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let d = dist(p, add(a, scale(ab, t)));
            if d < best.0 {
                best = (
                    d,
                    self.lengths[i] + t * (self.lengths[i + 1] - self.lengths[i]),
                );
            }
        }
        best.1
    }
}

/// A selected surface or mesh as a target for rays, with a coarse grid over its triangles
/// seen along `axis` (usually the CPlane normal).
pub struct Target<'a> {
    mesh: &'a SelectedMesh,
    frame: Frame,
    cell: f64,
    lo: [f64; 2],
    dims: [usize; 2],
    cells: Vec<Vec<u32>>,
}

impl<'a> Target<'a> {
    pub fn new(mesh: &'a SelectedMesh, axis: Vec3) -> Option<Target<'a>> {
        if mesh.triangles.is_empty() {
            return None;
        }
        let frame = Frame::new([0.0; 3], axis, [1.0, 0.0, 0.0]);
        let local: Vec<[f64; 2]> = mesh
            .vertices
            .iter()
            .map(|&v| {
                let l = frame.local(v);
                [l[0], l[1]]
            })
            .collect();
        let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
        for p in &local {
            for i in 0..2 {
                lo[i] = lo[i].min(p[i]);
                hi[i] = hi[i].max(p[i]);
            }
        }
        let side = ((hi[0] - lo[0]).max(hi[1] - lo[1])).max(1e-6);
        let cell = side / (mesh.triangles.len() as f64).sqrt().clamp(1.0, 256.0);
        let dims = [
            ((hi[0] - lo[0]) / cell) as usize + 1,
            ((hi[1] - lo[1]) / cell) as usize + 1,
        ];
        let mut cells = vec![Vec::new(); dims[0] * dims[1]];
        for (ti, t) in mesh.triangles.iter().enumerate() {
            let pts: Vec<[f64; 2]> = t
                .iter()
                .filter_map(|&i| local.get(i as usize).copied())
                .collect();
            if pts.len() != 3 {
                continue;
            }
            let (mut a, mut b) = ([f64::MAX; 2], [f64::MIN; 2]);
            for p in &pts {
                for i in 0..2 {
                    a[i] = a[i].min(p[i]);
                    b[i] = b[i].max(p[i]);
                }
            }
            let i0 = ((a[0] - lo[0]) / cell) as usize;
            let i1 = (((b[0] - lo[0]) / cell) as usize).min(dims[0] - 1);
            let j0 = ((a[1] - lo[1]) / cell) as usize;
            let j1 = (((b[1] - lo[1]) / cell) as usize).min(dims[1] - 1);
            for i in i0..=i1 {
                for j in j0..=j1 {
                    cells[i * dims[1] + j].push(ti as u32);
                }
            }
        }
        Some(Target {
            mesh,
            frame,
            cell,
            lo,
            dims,
            cells,
        })
    }

    /// Where the line through `p` along the grid axis meets the surface (the hit highest
    /// along the axis), and the surface normal there (facing +axis).
    /// Every crossing of the line through `p` along the axis with the mesh, from the top
    /// (largest along the axis) down: (point, unit normal facing the axis).
    pub fn hits(&self, p: Vec3) -> Vec<(Vec3, Vec3)> {
        let l = self.frame.local(p);
        let i = ((l[0] - self.lo[0]) / self.cell).floor();
        let j = ((l[1] - self.lo[1]) / self.cell).floor();
        if i < 0.0 || j < 0.0 || i as usize >= self.dims[0] || j as usize >= self.dims[1] {
            return Vec::new();
        }
        let dir = self.frame.z;
        let mut all: Vec<(f64, Vec3, Vec3)> = Vec::new();
        for &ti in &self.cells[i as usize * self.dims[1] + j as usize] {
            let t = self.mesh.triangles[ti as usize];
            let v = |k: usize| self.mesh.vertices[t[k] as usize];
            if let Some((h, n)) = ray_triangle(p, dir, v(0), v(1), v(2)) {
                let n = if dot(n, dir) < 0.0 { scale(n, -1.0) } else { n };
                all.push((h, add(p, scale(dir, h)), n));
            }
        }
        all.sort_by(|a, b| b.0.total_cmp(&a.0));
        // A crossing on a shared edge shows up once per triangle: keep one.
        all.dedup_by(|a, b| (a.0 - b.0).abs() < 1e-9);
        all.into_iter().map(|x| (x.1, x.2)).collect()
    }

    pub fn drop(&self, p: Vec3) -> Option<(Vec3, Vec3)> {
        let l = self.frame.local(p);
        let i = ((l[0] - self.lo[0]) / self.cell).floor();
        let j = ((l[1] - self.lo[1]) / self.cell).floor();
        if i < 0.0 || j < 0.0 || i as usize >= self.dims[0] || j as usize >= self.dims[1] {
            return None;
        }
        let dir = self.frame.z;
        let mut best: Option<(f64, Vec3, Vec3)> = None;
        for &ti in &self.cells[i as usize * self.dims[1] + j as usize] {
            let t = self.mesh.triangles[ti as usize];
            let v = |k: usize| self.mesh.vertices[t[k] as usize];
            if let Some((h, n)) = ray_triangle(p, dir, v(0), v(1), v(2))
                && best.is_none_or(|b| h > b.0)
            {
                let n = if dot(n, dir) < 0.0 { scale(n, -1.0) } else { n };
                best = Some((h, add(p, scale(dir, h)), n));
            }
        }
        best.map(|b| (b.1, b.2))
    }
}

/// Signed distance `h` along `dir` from `o` to the triangle (any sign), and its unit normal.
fn ray_triangle(o: Vec3, dir: Vec3, a: Vec3, b: Vec3, c: Vec3) -> Option<(f64, Vec3)> {
    let e1 = sub(b, a);
    let e2 = sub(c, a);
    let pv = cross(dir, e2);
    let det = dot(e1, pv);
    if det.abs() < 1e-14 {
        return None;
    }
    let inv = 1.0 / det;
    let tv = sub(o, a);
    let u = dot(tv, pv) * inv;
    if !(-1e-9..=1.0 + 1e-9).contains(&u) {
        return None;
    }
    let qv = cross(tv, e1);
    let v = dot(dir, qv) * inv;
    if v < -1e-9 || u + v > 1.0 + 1e-9 {
        return None;
    }
    Some((dot(e2, qv) * inv, unit(cross(e1, e2))))
}

/// The rigid (optionally scaled) matrix taking frame `from` onto frame `to`.
pub fn frame_matrix(from: &Frame, to: &Frame, scale_by: f64) -> [f64; 16] {
    // M = T(to.origin) · R_to · S · R_fromᵀ · T(−from.origin)
    let a = [from.x, from.y, from.z];
    let b = [to.x, to.y, to.z];
    let mut r = [[0.0; 3]; 3];
    for (i, row) in r.iter_mut().enumerate() {
        for (j, cell) in row.iter_mut().enumerate() {
            *cell = (0..3).map(|k| b[k][i] * a[k][j]).sum::<f64>() * scale_by;
        }
    }
    let mut m = om9_addon_abi::session::IDENTITY;
    for i in 0..3 {
        for j in 0..3 {
            m[i * 4 + j] = r[i][j];
        }
        m[i * 4 + 3] = to.origin[i] - (0..3).map(|j| r[i][j] * from.origin[j]).sum::<f64>();
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    use om9_addon_abi::session::apply;

    #[test]
    fn paths_measure_and_frame() {
        let p = Path::new(&[[0.0; 3], [10.0, 0.0, 0.0], [10.0, 10.0, 0.0]], false).unwrap();
        assert_eq!(p.length(), 20.0);
        assert_eq!(p.at(15.0), [10.0, 5.0, 0.0]);
        assert!((p.closest([12.0, 3.0, 0.0]) - 13.0).abs() < 1e-9);
        let ring: Vec<Vec3> = (0..360)
            .map(|i| {
                let a = std::f64::consts::TAU * i as f64 / 360.0;
                [5.0 * a.cos(), 5.0 * a.sin(), 0.0]
            })
            .collect();
        let r = Path::new(&ring, true).unwrap();
        let f = r.frame(0.0, None);
        assert!(
            (f.z[0] - 1.0).abs() < 1e-3,
            "outward at the start: {:?}",
            f.z
        );
        assert!((r.length() - std::f64::consts::TAU * 5.0).abs() < 0.01);
    }

    #[test]
    fn rays_drop_onto_a_mesh_and_frames_map() {
        let m = SelectedMesh {
            vertices: vec![
                [-5.0, -5.0, 2.0],
                [5.0, -5.0, 2.0],
                [5.0, 5.0, 4.0],
                [-5.0, 5.0, 4.0],
            ],
            triangles: vec![[0, 1, 2], [0, 2, 3]],
        };
        let t = Target::new(&m, [0.0, 0.0, 1.0]).unwrap();
        let (hit, n) = t.drop([0.0, 0.0, 0.0]).unwrap();
        assert!((hit[2] - 3.0).abs() < 1e-9 && n[2] > 0.9);
        assert!(t.drop([20.0, 0.0, 0.0]).is_none());
        let a = Frame::new([1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]);
        let b = Frame::new([0.0, 5.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        let mm = frame_matrix(&a, &b, 2.0);
        assert!(dist(apply(&mm, [1.0, 0.0, 1.0]), [2.0, 5.0, 0.0]) < 1e-9);
        assert!(dist(apply(&mm, [2.0, 0.0, 0.0]), [0.0, 7.0, 0.0]) < 1e-9);
    }
}
