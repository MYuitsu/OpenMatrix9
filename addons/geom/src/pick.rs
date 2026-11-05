// SPDX-License-Identifier: LGPL-2.1-or-later
//! Curves and surfaces the commands pick from the selection (Line Perpendicular / Tangent /
//! Extension / Normal, Circle AroundCurve / Tangent, Ellipse AroundCurve), kept in the
//! command state between steps.
use om9_addon_abi::session::{
    Event, Geo, Selected, SelectedMesh, Vec3, add, cross, dist, dot, norm, scale, sub, unit,
};
use crate as g;
use serde::{Deserialize, Serialize};

/// Samples kept in the state when the exact curve is not available.
const KEEP: usize = 2048;

/// A picked curve.
#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
pub struct CurveRef {
    pub id: String,
    /// Samples (closed curves without the repeated end point), at most [`KEEP`] + 1.
    pub pts: Vec<Vec3>,
    pub closed: bool,
    /// The exact curve when OM9 sends it; evaluation then uses it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exact: Option<Geo>,
}

impl CurveRef {
    pub fn of(s: &Selected) -> Option<CurveRef> {
        if s.kind != "curve" || s.points.len() < 2 {
            return None;
        }
        let n = s.points.len();
        let step = n.div_ceil(KEEP).max(1);
        let mut pts: Vec<Vec3> = s.points.iter().step_by(step).copied().collect();
        if !s.closed && pts.last() != s.points.last() {
            pts.push(*s.points.last()?);
        }
        let exact = s.exact.clone().filter(|x| {
            matches!(
                x,
                Geo::Line { .. }
                    | Geo::Polyline { .. }
                    | Geo::Circle { .. }
                    | Geo::Arc { .. }
                    | Geo::Nurbs { .. }
            )
        });
        Some(CurveRef {
            id: s.id.clone(),
            pts,
            closed: s.closed,
            exact,
        })
    }

    /// The curves of the selection, in order.
    pub fn all(e: &Event) -> Vec<CurveRef> {
        e.selection.iter().filter_map(CurveRef::of).collect()
    }

    /// The curve as geometry: the exact curve, or a polyline through the samples.
    pub fn geo(&self) -> Geo {
        self.exact.clone().unwrap_or_else(|| Geo::Polyline {
            points: self.pts.clone(),
            closed: self.closed,
        })
    }

    /// The part between `t0` and `t1` (closed curves wrap when `t1 < t0`; open curves give
    /// the reversed part).
    pub fn sub(&self, t0: f64, t1: f64) -> Option<Geo> {
        g::join::sub_geo(&self.geo(), t0, t1)
    }

    /// The curve the user just picked: a selected curve not in `taken` (by id), the one under
    /// the last click when there are several, with the parameter of the click on it.
    pub fn picked(e: &Event, taken: &[&str]) -> Option<(CurveRef, Option<f64>)> {
        let fresh: Vec<CurveRef> = CurveRef::all(e)
            .into_iter()
            .filter(|c| !taken.contains(&c.id.as_str()))
            .collect();
        match e.pick {
            Some(p) => fresh
                .into_iter()
                .map(|c| {
                    let (t, q) = c.closest(p);
                    (dist(q, p), c, t)
                })
                .min_by(|a, b| a.0.total_cmp(&b.0))
                .map(|(_, c, t)| (c, Some(t))),
            None => fresh.into_iter().next().map(|c| (c, None)),
        }
    }

    /// Point at normalized `t ∈ [0, 1]`.
    pub fn at(&self, t: f64) -> Vec3 {
        if let Some(p) = self.exact.as_ref().and_then(|x| g::eval_geo(x, t)) {
            return p;
        }
        let mut pts = self.pts.clone();
        if self.closed {
            pts.push(pts[0]);
        }
        let segs = pts.len() - 1;
        let x = t.clamp(0.0, 1.0) * segs as f64;
        let i = (x.floor() as usize).min(segs - 1);
        add(pts[i], scale(sub(pts[i + 1], pts[i]), x - i as f64))
    }

    /// Unit tangent at `t` (along the curve).
    pub fn tangent(&self, t: f64) -> Vec3 {
        let h = if self.exact.is_some() {
            1e-6
        } else {
            0.5 / self.pts.len() as f64
        };
        let (a, b) = if self.closed {
            (t - h, t + h)
        } else {
            ((t - h).max(0.0), (t + h).min(1.0))
        };
        let wrap = |u: f64| if self.closed { u.rem_euclid(1.0) } else { u };
        let d = sub(self.at(wrap(b)), self.at(wrap(a)));
        if norm(d) < 1e-15 {
            [1.0, 0.0, 0.0]
        } else {
            unit(d)
        }
    }

    /// Closest point: (t, point).
    pub fn closest(&self, p: Vec3) -> (f64, Vec3) {
        let n = 512;
        let d = |t: f64| dist(self.at(t), p);
        let best =
            (0..=n).min_by(|&a, &b| d(a as f64 / n as f64).total_cmp(&d(b as f64 / n as f64)));
        let best = best.unwrap_or(0) as f64 / n as f64;
        let t = refine(
            &d,
            best - 1.0 / n as f64,
            best + 1.0 / n as f64,
            self.closed,
        );
        (t, self.at(t))
    }

    /// Feet of the perpendiculars from `p` (parameters where `(C − p) · C' = 0`).
    pub fn feet(&self, p: Vec3) -> Vec<f64> {
        let f = |t: f64| dot(sub(self.at(t), p), self.tangent(t));
        roots(&f, self.closed)
    }

    /// Parameters where the line from `p` touches the curve (`(C − p) × C' = 0` in the plane
    /// with normal `n`).
    pub fn tangent_points(&self, p: Vec3, n: Vec3) -> Vec<f64> {
        let f = |t: f64| dot(cross(sub(self.at(t), p), self.tangent(t)), n);
        roots(&f, self.closed)
    }

    /// The two ends (open curves) as (point, outward tangent).
    pub fn ends(&self) -> [(Vec3, Vec3); 2] {
        [
            (self.at(0.0), scale(self.tangent(0.0), -1.0)),
            (self.at(1.0), self.tangent(1.0)),
        ]
    }
}

/// Golden-section minimum of `d` on `[lo, hi]` (wrapped into [0, 1] for closed curves).
fn refine(d: &dyn Fn(f64) -> f64, lo: f64, hi: f64, closed: bool) -> f64 {
    let wrap = |u: f64| {
        if closed {
            u.rem_euclid(1.0)
        } else {
            u.clamp(0.0, 1.0)
        }
    };
    let (mut lo, mut hi) = if closed {
        (lo, hi)
    } else {
        (lo.max(0.0), hi.min(1.0))
    };
    for _ in 0..60 {
        let (m1, m2) = (lo + (hi - lo) / 3.0, hi - (hi - lo) / 3.0);
        if d(wrap(m1)) < d(wrap(m2)) {
            hi = m2;
        } else {
            lo = m1;
        }
    }
    wrap((lo + hi) / 2.0)
}

/// Sign changes of `f` on [0, 1], refined by bisection.
fn roots(f: &dyn Fn(f64) -> f64, closed: bool) -> Vec<f64> {
    let n = 600;
    let mut out = Vec::new();
    let mut prev = f(0.0);
    for i in 1..=n {
        let t = i as f64 / n as f64;
        let v = f(t.min(if closed { 1.0 - 1e-9 } else { 1.0 }));
        if prev == 0.0 {
            out.push((i - 1) as f64 / n as f64);
        } else if prev.signum() != v.signum() {
            let (mut a, mut b) = ((i - 1) as f64 / n as f64, t);
            let fa = prev;
            for _ in 0..60 {
                let m = (a + b) / 2.0;
                if f(m).signum() == fa.signum() {
                    a = m;
                } else {
                    b = m;
                }
            }
            out.push((a + b) / 2.0);
        }
        prev = v;
    }
    out
}

/// A picked surface, polysurface or mesh, for normals.
#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
pub struct SurfRef {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exact: Option<Geo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mesh: Option<SelectedMesh>,
}

impl SurfRef {
    pub fn of(s: &Selected) -> Option<SurfRef> {
        let exact = s.exact.clone().filter(|x| matches!(x, Geo::Surface { .. }));
        let mesh = s.mesh.clone().filter(|m| !m.triangles.is_empty());
        (exact.is_some() || mesh.is_some()).then(|| SurfRef {
            id: s.id.clone(),
            exact,
            mesh,
        })
    }

    /// Closest point and unit normal there.
    pub fn closest(&self, p: Vec3) -> Option<(Vec3, Vec3)> {
        if let Some(srf) = self.exact.as_ref().and_then(g::nurbs::Srf::of) {
            let (s, t) = srf.closest_st(p);
            return Some((srf.at_st(s, t), unit(srf.normal_st(s, t))));
        }
        let m = self.mesh.as_ref()?;
        let mut best: Option<(f64, Vec3, Vec3)> = None;
        for tri in &m.triangles {
            let [a, b, c] = tri.map(|i| m.vertices[i as usize]);
            let q = g::sdf::closest_on_triangle(p, a, b, c);
            let d = dist(p, q);
            if best.as_ref().is_none_or(|(bd, _, _)| d < *bd) {
                let nrm = cross(sub(b, a), sub(c, a));
                if norm(nrm) > 1e-18 {
                    best = Some((d, q, unit(nrm)));
                }
            }
        }
        best.map(|(_, q, n)| (q, n))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn circle_sel() -> Selected {
        let pts: Vec<Vec3> = (0..1024)
            .map(|i| {
                let a = std::f64::consts::TAU * i as f64 / 1024.0;
                [10.0 * a.cos(), 10.0 * a.sin(), 0.0]
            })
            .collect();
        Selected {
            id: "c".into(),
            kind: "curve".into(),
            points: pts,
            closed: true,
            exact: Some(Geo::Circle {
                center: [0.0; 3],
                x: [1.0, 0.0, 0.0],
                y: [0.0, 1.0, 0.0],
                radius: 10.0,
            }),
            ..Default::default()
        }
    }

    #[test]
    fn closest_tangent_feet_on_a_circle() {
        let c = CurveRef::of(&circle_sel()).unwrap();
        assert!(c.pts.len() <= KEEP + 1);
        let (_, q) = c.closest([20.0, 0.0, 0.0]);
        assert!(dist(q, [10.0, 0.0, 0.0]) < 1e-6, "{q:?}");
        let t = c.tangent(0.25);
        assert!(dist(t, [-1.0, 0.0, 0.0]) < 1e-5, "{t:?}");
        // Two perpendicular feet from an outside point: the near and far sides.
        let feet = c.feet([20.0, 0.0, 0.0]);
        assert_eq!(feet.len(), 2, "{feet:?}");
        // Tangent points from (20, 0): x = r²/d = 5.
        let tp = c.tangent_points([20.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
        assert_eq!(tp.len(), 2, "{tp:?}");
        for t in tp {
            assert!((c.at(t)[0] - 5.0).abs() < 1e-5);
        }
    }

    #[test]
    fn samples_only_curves_work_too() {
        let mut s = circle_sel();
        s.exact = None;
        let c = CurveRef::of(&s).unwrap();
        let (_, q) = c.closest([0.0, 20.0, 0.0]);
        assert!(dist(q, [0.0, 10.0, 0.0]) < 1e-3, "{q:?}");
    }

    #[test]
    fn mesh_normal() {
        let s = Selected {
            id: "m".into(),
            kind: "mesh".into(),
            mesh: Some(SelectedMesh {
                vertices: vec![[0.0; 3], [10.0, 0.0, 0.0], [0.0, 10.0, 0.0]],
                triangles: vec![[0, 1, 2]],
            }),
            ..Default::default()
        };
        let (q, n) = SurfRef::of(&s).unwrap().closest([2.0, 2.0, 5.0]).unwrap();
        assert!(dist(q, [2.0, 2.0, 0.0]) < 1e-9);
        assert!(dist(n, [0.0, 0.0, 1.0]) < 1e-9);
    }
}
