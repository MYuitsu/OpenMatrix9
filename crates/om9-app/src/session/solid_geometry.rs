// SPDX-License-Identifier: LGPL-2.1-or-later
//! Pure geometry helpers for the solid commands: text outlines, tangent spheres, cube axes,
//! slab smoothing and polygon tests. Moved unchanged from `solid.rs`; `Controller` methods there
//! call these through the `pub(super)` re-export.
use super::solid::SolidPending;
use super::*;
#[cfg(feature = "occt-brep")]
use om9_kernel::editing;
#[cfg(feature = "occt-brep")]
#[derive(Default)]
pub(super) struct TextPen {
    pub(super) contours: Vec<Vec<om9_core::geometry::Point3>>,
    pub(super) current: Vec<om9_core::geometry::Point3>,
}

#[cfg(feature = "occt-brep")]
impl skrifa::outline::OutlinePen for TextPen {
    fn move_to(&mut self, x: f32, y: f32) {
        if self.current.len() > 2 {
            self.contours.push(std::mem::take(&mut self.current));
        } else {
            self.current.clear();
        }
        self.current.push([x as f64, y as f64, 0.0]);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.current.push([x as f64, y as f64, 0.0]);
    }
    fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        let Some(start) = self.current.last().copied() else {
            return;
        };
        for step in 1..=12 {
            let t = step as f64 / 12.0;
            let mt = 1.0 - t;
            self.current.push([
                mt * mt * start[0] + 2.0 * mt * t * cx as f64 + t * t * x as f64,
                mt * mt * start[1] + 2.0 * mt * t * cy as f64 + t * t * y as f64,
                0.0,
            ]);
        }
    }
    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        let Some(start) = self.current.last().copied() else {
            return;
        };
        for step in 1..=16 {
            let t = step as f64 / 16.0;
            let mt = 1.0 - t;
            self.current.push([
                mt * mt * mt * start[0]
                    + 3.0 * mt * mt * t * cx0 as f64
                    + 3.0 * mt * t * t * cx1 as f64
                    + t * t * t * x as f64,
                mt * mt * mt * start[1]
                    + 3.0 * mt * mt * t * cy0 as f64
                    + 3.0 * mt * t * t * cy1 as f64
                    + t * t * t * y as f64,
                0.0,
            ]);
        }
    }
    fn close(&mut self) {
        if self.current.len() > 2 {
            self.contours.push(std::mem::take(&mut self.current));
        } else {
            self.current.clear();
        }
    }
}

#[cfg(feature = "occt-brep")]
pub(super) fn box_wire_edges(
    dims: [f64; 3],
    axes: [[f64; 3]; 3],
    origin: om9_core::geometry::Point3,
) -> Vec<[om9_core::geometry::Point3; 2]> {
    let corner = |bits: usize| {
        [
            origin[0]
                + axes[0][0] * dims[0] * (bits & 1) as f64
                + axes[1][0] * dims[1] * ((bits >> 1) & 1) as f64
                + axes[2][0] * dims[2] * ((bits >> 2) & 1) as f64,
            origin[1]
                + axes[0][1] * dims[0] * (bits & 1) as f64
                + axes[1][1] * dims[1] * ((bits >> 1) & 1) as f64
                + axes[2][1] * dims[2] * ((bits >> 2) & 1) as f64,
            origin[2]
                + axes[0][2] * dims[0] * (bits & 1) as f64
                + axes[1][2] * dims[1] * ((bits >> 1) & 1) as f64
                + axes[2][2] * dims[2] * ((bits >> 2) & 1) as f64,
        ]
    };
    let corners: Vec<_> = (0..8).map(corner).collect();
    [
        (0, 1),
        (1, 3),
        (3, 2),
        (2, 0),
        (4, 5),
        (5, 7),
        (7, 6),
        (6, 4),
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ]
    .iter()
    .map(|(a, b)| [corners[*a], corners[*b]])
    .collect()
}

pub(super) fn box_edge_axes(
    first: [f64; 3],
    second: [f64; 3],
    plane: super::CPlane,
    vertical: bool,
) -> Result<(f64, [[f64; 3]; 3])> {
    let up = plane.normal();
    let d = std::array::from_fn::<_, 3, _>(|i| second[i] - first[i]);
    let normal_component = (0..3).map(|i| d[i] * up[i]).sum::<f64>();
    let edge = std::array::from_fn::<_, 3, _>(|i| d[i] - normal_component * up[i]);
    let length = edge[0].hypot(edge[1]).hypot(edge[2]);
    if !length.is_finite() || length <= 1e-9 {
        return Err(Error::Invalid("box needs a non-zero base edge"));
    }
    let x = edge.map(|v| v / length);
    let cross = |a: [f64; 3], b: [f64; 3]| {
        [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ]
    };
    let y = if vertical { up } else { cross(up, x) };
    let z = cross(x, y);
    Ok((length, [x, y, z]))
}

#[cfg(feature = "occt-brep")]
pub(super) fn point_polyline_distance(
    point: om9_core::geometry::Point3,
    curve: &[om9_core::geometry::Point3],
) -> f64 {
    curve
        .windows(2)
        .map(|window| {
            let a = window[0];
            let b = window[1];
            let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let ap = [point[0] - a[0], point[1] - a[1], point[2] - a[2]];
            let length = ab[0] * ab[0] + ab[1] * ab[1] + ab[2] * ab[2];
            let t = if length > 1e-20 {
                ((ap[0] * ab[0] + ap[1] * ab[1] + ap[2] * ab[2]) / length).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let nearest = [a[0] + ab[0] * t, a[1] + ab[1] * t, a[2] + ab[2] * t];
            ((point[0] - nearest[0]).powi(2)
                + (point[1] - nearest[1]).powi(2)
                + (point[2] - nearest[2]).powi(2))
            .sqrt()
        })
        .fold(f64::INFINITY, f64::min)
}

#[cfg(feature = "occt-brep")]
pub(super) fn tangent_sphere_three_curves(
    first: &[om9_core::geometry::Point3],
    second: &[om9_core::geometry::Point3],
    third: &[om9_core::geometry::Point3],
    radius: f64,
) -> Result<om9_core::geometry::Point3> {
    let curves = [first, second, third];
    let mut candidates = Vec::new();
    for i in 0..3 {
        for j in (i + 1)..3 {
            for a in curves[i].windows(2) {
                for b in curves[j].windows(2) {
                    let da = [a[1][0] - a[0][0], a[1][1] - a[0][1]];
                    let db = [b[1][0] - b[0][0], b[1][1] - b[0][1]];
                    let cross = da[0] * db[1] - da[1] * db[0];
                    if cross.abs() <= 1e-12 {
                        continue;
                    }
                    let na = [
                        da[1] / (da[0].powi(2) + da[1].powi(2)).sqrt(),
                        -da[0] / (da[0].powi(2) + da[1].powi(2)).sqrt(),
                    ];
                    let nb = [
                        db[1] / (db[0].powi(2) + db[1].powi(2)).sqrt(),
                        -db[0] / (db[0].powi(2) + db[1].powi(2)).sqrt(),
                    ];
                    for sa in [1.0, -1.0] {
                        for sb in [1.0, -1.0] {
                            let p = [a[0][0] + na[0] * radius * sa, a[0][1] + na[1] * radius * sa];
                            let q = [b[0][0] + nb[0] * radius * sb, b[0][1] + nb[1] * radius * sb];
                            let delta = [q[0] - p[0], q[1] - p[1]];
                            let t = (delta[0] * db[1] - delta[1] * db[0]) / cross;
                            let center = [p[0] + da[0] * t, p[1] + da[1] * t, a[0][2]];
                            if curves.iter().all(|curve| {
                                (point_polyline_distance(center, curve) - radius).abs()
                                    < 1e-6 * radius.max(1.0)
                            }) {
                                candidates.push(center);
                            }
                        }
                    }
                }
            }
        }
    }
    let mut unique = Vec::new();
    for candidate in candidates {
        if !unique.iter().any(|other: &om9_core::geometry::Point3| {
            (candidate[0] - other[0]).abs() < 1e-7 && (candidate[1] - other[1]).abs() < 1e-7
        }) {
            unique.push(candidate);
        }
    }
    unique
        .into_iter()
        .min_by(|a, b| (a[0].powi(2) + a[1].powi(2)).total_cmp(&(b[0].powi(2) + b[1].powi(2))))
        .ok_or(Error::Invalid("no tangent sphere to the three curves"))
}

#[cfg(feature = "occt-brep")]
pub(super) fn tangent_sphere_two_curves_point(
    first: &[om9_core::geometry::Point3],
    second: &[om9_core::geometry::Point3],
    point: om9_core::geometry::Point3,
) -> Result<om9_core::geometry::Point3> {
    let mut candidates = Vec::new();
    for a in first.windows(2) {
        for b in second.windows(2) {
            let da = [a[1][0] - a[0][0], a[1][1] - a[0][1]];
            let db = [b[1][0] - b[0][0], b[1][1] - b[0][1]];
            let cross = da[0] * db[1] - da[1] * db[0];
            if cross.abs() <= 1e-12 {
                continue;
            }
            let na = [
                da[1] / (da[0].powi(2) + da[1].powi(2)).sqrt(),
                -da[0] / (da[0].powi(2) + da[1].powi(2)).sqrt(),
            ];
            let nb = [
                db[1] / (db[0].powi(2) + db[1].powi(2)).sqrt(),
                -db[0] / (db[0].powi(2) + db[1].powi(2)).sqrt(),
            ];
            for sa in [1.0, -1.0] {
                for sb in [1.0, -1.0] {
                    let delta0 = [b[0][0] - a[0][0], b[0][1] - a[0][1]];
                    let delta_n = [nb[0] * sb - na[0] * sa, nb[1] * sb - na[1] * sa];
                    let t0 = (delta0[0] * db[1] - delta0[1] * db[0]) / cross;
                    let t1 = (delta_n[0] * db[1] - delta_n[1] * db[0]) / cross;
                    let c0 = [a[0][0] + da[0] * t0, a[0][1] + da[1] * t0];
                    let c1 = [na[0] * sa + da[0] * t1, na[1] * sa + da[1] * t1];
                    let ox = c0[0] - point[0];
                    let oy = c0[1] - point[1];
                    let qa = c1[0].powi(2) + c1[1].powi(2) - 1.0;
                    let qb = 2.0 * (ox * c1[0] + oy * c1[1]);
                    let qc = ox.powi(2) + oy.powi(2);
                    let mut roots = Vec::new();
                    if qa.abs() < 1e-12 {
                        if qb.abs() > 1e-12 {
                            roots.push(-qc / qb);
                        }
                    } else {
                        let disc = qb * qb - 4.0 * qa * qc;
                        if disc >= 0.0 {
                            let root = disc.sqrt();
                            roots.push((-qb + root) / (2.0 * qa));
                            roots.push((-qb - root) / (2.0 * qa));
                        }
                    }
                    for radius in roots {
                        if radius <= 1e-9 {
                            continue;
                        }
                        let center = [c0[0] + c1[0] * radius, c0[1] + c1[1] * radius, a[0][2]];
                        let da_distance = point_polyline_distance(center, first);
                        let db_distance = point_polyline_distance(center, second);
                        let point_distance = ((center[0] - point[0]).powi(2)
                            + (center[1] - point[1]).powi(2))
                        .sqrt();
                        if (da_distance - radius).abs() < 1e-6 * radius.max(1.0)
                            && (db_distance - radius).abs() < 1e-6 * radius.max(1.0)
                            && (point_distance - radius).abs() < 1e-6 * radius.max(1.0)
                        {
                            candidates.push(center);
                        }
                    }
                }
            }
        }
    }
    let mut unique = Vec::new();
    for candidate in candidates {
        if !unique.iter().any(|other: &om9_core::geometry::Point3| {
            (candidate[0] - other[0]).abs() < 1e-6 && (candidate[1] - other[1]).abs() < 1e-6
        }) {
            unique.push(candidate);
        }
    }
    unique
        .into_iter()
        .min_by(|a, b| (a[0].powi(2) + a[1].powi(2)).total_cmp(&(b[0].powi(2) + b[1].powi(2))))
        .ok_or(Error::Invalid(
            "no tangent sphere to the two curves and point",
        ))
}

#[cfg(feature = "occt-brep")]
pub(super) fn tangent_sphere_curve_two_points(
    curve: &[om9_core::geometry::Point3],
    first: om9_core::geometry::Point3,
    second: om9_core::geometry::Point3,
) -> Result<om9_core::geometry::Point3> {
    let mid = [
        (first[0] + second[0]) * 0.5,
        (first[1] + second[1]) * 0.5,
        (first[2] + second[2]) * 0.5,
    ];
    let dx = second[0] - first[0];
    let dy = second[1] - first[1];
    let length = (dx * dx + dy * dy).sqrt();
    if length <= 1e-12 {
        return Err(Error::Invalid("the two points must be distinct"));
    }
    let perp = [-dy / length, dx / length];
    let error_at = |offset: f64| {
        let center = [mid[0] + perp[0] * offset, mid[1] + perp[1] * offset, mid[2]];
        let radius = ((center[0] - first[0]).powi(2)
            + (center[1] - first[1]).powi(2)
            + (center[2] - first[2]).powi(2))
        .sqrt();
        (point_polyline_distance(center, curve) - radius, center)
    };
    let span = curve
        .iter()
        .chain([&first, &second])
        .fold(1.0_f64, |span, point| {
            span.max(point[0].abs()).max(point[1].abs())
        })
        * 8.0;
    let step = span / 4000.0;
    let mut roots = Vec::new();
    let mut previous = error_at(-span);
    for index in 1..=8000 {
        let offset = -span + step * index as f64;
        let current = error_at(offset);
        if current.0.abs() < 1e-9 {
            roots.push(current.1);
        } else if previous.0 * current.0 < 0.0 {
            let mut low = -span + step * (index - 1) as f64;
            let mut high = offset;
            let mut low_value = previous.0;
            for _ in 0..80 {
                let middle = (low + high) * 0.5;
                let value = error_at(middle).0;
                if low_value * value <= 0.0 {
                    high = middle;
                } else {
                    low = middle;
                    low_value = value;
                }
            }
            roots.push(error_at((low + high) * 0.5).1);
        }
        previous = current;
    }
    roots
        .into_iter()
        .min_by(|a, b| (a[0].powi(2) + a[1].powi(2)).total_cmp(&(b[0].powi(2) + b[1].powi(2))))
        .ok_or(Error::Invalid("no tangent sphere through the two points"))
}

#[cfg(feature = "occt-brep")]
pub(super) fn tangent_sphere_curve_point(
    curve: &[om9_core::geometry::Point3],
    point: om9_core::geometry::Point3,
    radius: f64,
) -> Result<om9_core::geometry::Point3> {
    let error_at = |angle: f64| {
        let candidate = [
            point[0] + radius * angle.cos(),
            point[1] + radius * angle.sin(),
            point[2],
        ];
        (point_polyline_distance(candidate, curve) - radius).abs()
    };
    let step = std::f64::consts::TAU / 3600.0;
    let mut best: Option<(f64, f64)> = None;
    for index in 0..3600 {
        let angle = step * index as f64;
        let error = error_at(angle);
        if error < 1e-2 * radius.max(1.0)
            && best
                .as_ref()
                .is_none_or(|(best_error, _)| error < *best_error)
        {
            best = Some((error, angle));
        }
    }
    let Some((_, coarse)) = best else {
        return Err(Error::Invalid("no tangent sphere for curve and point"));
    };
    let mut low = coarse - step;
    let mut high = coarse + step;
    for _ in 0..80 {
        let first = low + (high - low) / 3.0;
        let second = high - (high - low) / 3.0;
        if error_at(first) < error_at(second) {
            high = second;
        } else {
            low = first;
        }
    }
    let angle = (low + high) * 0.5;
    Ok([
        point[0] + radius * angle.cos(),
        point[1] + radius * angle.sin(),
        point[2],
    ])
}

#[cfg(feature = "occt-brep")]
pub(super) fn tangent_sphere_two_curves(
    first: &[om9_core::geometry::Point3],
    second: &[om9_core::geometry::Point3],
    radius: f64,
) -> Result<om9_core::geometry::Point3> {
    let mut candidates = Vec::new();
    for a in first.windows(2) {
        for b in second.windows(2) {
            let da = [a[1][0] - a[0][0], a[1][1] - a[0][1]];
            let db = [b[1][0] - b[0][0], b[1][1] - b[0][1]];
            let cross = da[0] * db[1] - da[1] * db[0];
            if cross.abs() <= 1e-12 {
                continue;
            }
            let na = [
                da[1] / (da[0].powi(2) + da[1].powi(2)).sqrt(),
                -da[0] / (da[0].powi(2) + da[1].powi(2)).sqrt(),
            ];
            let nb = [
                db[1] / (db[0].powi(2) + db[1].powi(2)).sqrt(),
                -db[0] / (db[0].powi(2) + db[1].powi(2)).sqrt(),
            ];
            for sa in [1.0, -1.0] {
                for sb in [1.0, -1.0] {
                    let p = [a[0][0] + na[0] * radius * sa, a[0][1] + na[1] * radius * sa];
                    let q = [b[0][0] + nb[0] * radius * sb, b[0][1] + nb[1] * radius * sb];
                    let delta = [q[0] - p[0], q[1] - p[1]];
                    let t = (delta[0] * db[1] - delta[1] * db[0]) / cross;
                    let center = [p[0] + da[0] * t, p[1] + da[1] * t, a[0][2]];
                    let da_distance = point_polyline_distance(center, first);
                    let db_distance = point_polyline_distance(center, second);
                    if (da_distance - radius).abs() < 1e-6 * radius.max(1.0)
                        && (db_distance - radius).abs() < 1e-6 * radius.max(1.0)
                    {
                        candidates.push(center);
                    }
                }
            }
        }
    }
    let mut unique = Vec::new();
    for candidate in candidates {
        if !unique.iter().any(|other: &om9_core::geometry::Point3| {
            (candidate[0] - other[0]).abs() < 1e-7 && (candidate[1] - other[1]).abs() < 1e-7
        }) {
            unique.push(candidate);
        }
    }
    unique
        .into_iter()
        .min_by(|a, b| (a[0].powi(2) + a[1].powi(2)).total_cmp(&(b[0].powi(2) + b[1].powi(2))))
        .ok_or(Error::Invalid("no tangent sphere between the two curves"))
}

#[cfg(feature = "occt-brep")]
pub(super) fn cube_axes(
    axes: [om9_core::geometry::Point3; 3],
    delta: om9_core::geometry::Point3,
) -> Result<([om9_core::geometry::Point3; 3], f64)> {
    let length = (delta[0].powi(2) + delta[1].powi(2) + delta[2].powi(2)).sqrt();
    if !length.is_finite() || length <= 0.0 {
        return Err(Error::Invalid("cube diagonal"));
    }
    let target = [delta[0] / length, delta[1] / length, delta[2] / length];
    let source = [
        (axes[0][0] + axes[1][0] + axes[2][0]) / 3.0_f64.sqrt(),
        (axes[0][1] + axes[1][1] + axes[2][1]) / 3.0_f64.sqrt(),
        (axes[0][2] + axes[1][2] + axes[2][2]) / 3.0_f64.sqrt(),
    ];
    let cosine =
        (source[0] * target[0] + source[1] * target[1] + source[2] * target[2]).clamp(-1.0, 1.0);
    let v = [
        source[1] * target[2] - source[2] * target[1],
        source[2] * target[0] - source[0] * target[2],
        source[0] * target[1] - source[1] * target[0],
    ];
    let sine = (v[0].powi(2) + v[1].powi(2) + v[2].powi(2)).sqrt();
    let axis = if sine > 1e-15 {
        [v[0] / sine, v[1] / sine, v[2] / sine]
    } else {
        let cross = [
            source[1] * axes[0][2] - source[2] * axes[0][1],
            source[2] * axes[0][0] - source[0] * axes[0][2],
            source[0] * axes[0][1] - source[1] * axes[0][0],
        ];
        let norm = cross[0].hypot(cross[1]).hypot(cross[2]);
        if norm <= 1e-15 {
            return Err(Error::Invalid("cube rotation axis"));
        }
        cross.map(|v| v / norm)
    };
    let rotate = |point: om9_core::geometry::Point3| {
        let cross = [
            axis[1] * point[2] - axis[2] * point[1],
            axis[2] * point[0] - axis[0] * point[2],
            axis[0] * point[1] - axis[1] * point[0],
        ];
        let dot = axis[0] * point[0] + axis[1] * point[1] + axis[2] * point[2];
        [
            point[0] * cosine + cross[0] * sine + axis[0] * dot * (1.0 - cosine),
            point[1] * cosine + cross[1] * sine + axis[1] * dot * (1.0 - cosine),
            point[2] * cosine + cross[2] * sine + axis[2] * dot * (1.0 - cosine),
        ]
    };
    Ok((axes.map(rotate), length / 3.0_f64.sqrt()))
}

#[cfg(feature = "occt-brep")]
pub(super) fn sphere_chord_direction(
    a: [f64; 3],
    b: [f64; 3],
    radius: f64,
    point: [f64; 3],
) -> Result<[f64; 3]> {
    let chord = std::array::from_fn::<_, 3, _>(|i| b[i] - a[i]);
    let length = chord[0].hypot(chord[1]).hypot(chord[2]);
    if !length.is_finite() || length <= 1e-9 || !radius.is_finite() || radius < length * 0.5 {
        return Err(Error::Invalid(
            "sphere radius must be at least half a non-zero chord",
        ));
    }
    let axis = chord.map(|v| v / length);
    let mid = std::array::from_fn::<_, 3, _>(|i| (a[i] + b[i]) * 0.5);
    let v = std::array::from_fn::<_, 3, _>(|i| point[i] - mid[i]);
    let along = (0..3).map(|i| v[i] * axis[i]).sum::<f64>();
    let normal = std::array::from_fn::<_, 3, _>(|i| v[i] - along * axis[i]);
    let norm = normal[0].hypot(normal[1]).hypot(normal[2]);
    if !norm.is_finite() || norm <= 1e-9 {
        return Err(Error::Invalid("choose direction away from the chord"));
    }
    let ratio = (length * 0.5) / radius;
    let offset = radius * (1.0 - ratio * ratio).max(0.0).sqrt();
    if !offset.is_finite() {
        return Err(Error::Invalid("sphere radius overflow"));
    }
    Ok(std::array::from_fn(|i| mid[i] + normal[i] * offset / norm))
}

pub(super) fn sphere_radius_input(text: &str) -> Option<f64> {
    let text = text.trim();
    let raw = if let Some((key, value)) = text.split_once('=') {
        if !key.trim().eq_ignore_ascii_case("radius") {
            return None;
        }
        value.trim()
    } else {
        text
    };
    raw.parse::<f64>()
        .ok()
        .filter(|v| v.is_finite() && *v > 0.0)
}

pub(super) enum SlabSourceGeometry {
    Polyline(Vec<[f64; 3]>),
    ClosedPolyline(Vec<[f64; 3]>),
    Circle(om9_core::slab::CircleSource),
    Smooth {
        curve: om9_core::geometry::Curve,
        points: Vec<[f64; 3]>,
        fractions: Vec<f64>,
        closed: bool,
    },
}
#[cfg(feature = "occt-brep")]
pub(super) fn circle_offset(
    circle: &om9_core::slab::CircleSource,
    pending: &SolidPending,
    point: [f64; 3],
) -> Result<f64> {
    let picked = om9_core::slab::circle_offset_from_pick(circle, pending.slab_normal, point)?;
    Ok(pending
        .slab_distance
        .map_or(picked, |distance| picked.signum() * distance))
}

/// Refine the best sampled station and every active knot span. This is not a
/// certified global closest-point algorithm for arbitrary NURBS.
pub(super) fn slab_smooth_nearest(
    curve: &om9_core::geometry::Curve,
    points: &[[f64; 3]],
    fractions: &[f64],
    pick: [f64; 3],
    normal: [f64; 3],
    limits: &om9_core::budget::Limits,
) -> Result<(f64, [f64; 3])> {
    if !pick.iter().all(|x| x.is_finite()) || points.len() != fractions.len() || points.len() < 2 {
        return Err(Error::Invalid("Slab NURBS pick"));
    }
    let distance = |p: [f64; 3]| (0..3).map(|i| (p[i] - pick[i]).powi(2)).sum::<f64>();
    let mut best = 0;
    let mut best_distance = distance(points[0]);
    for (i, p) in points.iter().enumerate().skip(1) {
        let d = distance(*p);
        if d < best_distance {
            best = i;
            best_distance = d;
        }
    }
    if !best_distance.is_finite() {
        return Err(Error::Invalid("Slab NURBS pick distance"));
    }
    let om9_core::geometry::Curve::Nurbs(n) = curve else {
        return Err(Error::Invalid("Slab smooth source"));
    };
    let spans = n
        .knots
        .windows(2)
        .filter(|pair| pair[0].max(n.domain[0]) < pair[1].min(n.domain[1]))
        .count();
    let work = spans
        .checked_add(1)
        .and_then(|v| v.checked_mul(67))
        .and_then(|v| v.checked_mul(n.knots.len()))
        .and_then(|v| u64::try_from(v).ok())
        .ok_or(Error::Overflow("Slab NURBS pick work"))?;
    if work > limits.max_nodes {
        return Err(Error::Budget("Slab NURBS pick work"));
    }
    let mut answer = (fractions[best], points[best]);
    let mut ambiguous = false;
    let signed = |fraction: f64, nearest: [f64; 3]| -> Result<f64> {
        let t = om9_core::curve_ops::tangent_at(curve, fraction, limits)?;
        let side = [
            t[1] * normal[2] - t[2] * normal[1],
            t[2] * normal[0] - t[0] * normal[2],
            t[0] * normal[1] - t[1] * normal[0],
        ];
        let length = side[0].hypot(side[1]).hypot(side[2]);
        if length < 1e-12 {
            return Err(Error::Invalid("Slab NURBS tangent"));
        }
        Ok((0..3)
            .map(|i| (pick[i] - nearest[i]) * side[i] / length)
            .sum())
    };
    let mut refine = |mut low: f64, mut high: f64| -> Result<()> {
        let golden = 0.618_033_988_749_895;
        let mut c = high - golden * (high - low);
        let mut d = low + golden * (high - low);
        let mut pc = om9_core::curve_ops::point_at(curve, c, limits)?;
        let mut pd = om9_core::curve_ops::point_at(curve, d, limits)?;
        for _ in 0..64 {
            if distance(pc) < distance(pd) {
                high = d;
                d = c;
                pd = pc;
                c = high - golden * (high - low);
                pc = om9_core::curve_ops::point_at(curve, c, limits)?;
            } else {
                low = c;
                c = d;
                pc = pd;
                d = low + golden * (high - low);
                pd = om9_core::curve_ops::point_at(curve, d, limits)?;
            }
        }
        let fraction = low + (high - low) * 0.5;
        let refined = om9_core::curve_ops::point_at(curve, fraction, limits)?;
        let measured = distance(refined);
        let tolerance = 1e-12 * best_distance.max(1.);
        if measured < best_distance - tolerance {
            best_distance = measured;
            answer = (fraction, refined);
            ambiguous = false;
        } else if (measured - best_distance).abs() <= tolerance {
            let separation = (0..3)
                .map(|i| (refined[i] - answer.1[i]).powi(2))
                .sum::<f64>()
                .sqrt();
            if separation > 1e-5
                && (signed(fraction, refined)? - signed(answer.0, answer.1)?).abs() > 1e-7
            {
                ambiguous = true;
            }
        }
        Ok(())
    };
    refine(
        fractions[best.saturating_sub(1)],
        fractions[(best + 1).min(fractions.len() - 1)],
    )?;
    for pair in n.knots.windows(2) {
        let lo = pair[0].max(n.domain[0]);
        let hi = pair[1].min(n.domain[1]);
        if hi <= lo {
            continue;
        }
        let a = (lo - n.domain[0]) / (n.domain[1] - n.domain[0]);
        let b = (hi - n.domain[0]) / (n.domain[1] - n.domain[0]);
        if n.reversed {
            refine(1. - b, 1. - a)?;
        } else {
            refine(a, b)?;
        }
    }
    if ambiguous {
        Err(Error::Invalid("Slab NURBS pick is ambiguous"))
    } else {
        Ok(answer)
    }
}

impl SlabSourceGeometry {
    pub(super) fn capture(
        entity: &om9_core::model::Entity,
        limits: &om9_core::budget::Limits,
    ) -> Result<Self> {
        let Geometry::Curve(curve) = &entity.geometry else {
            return Err(Error::Invalid("Slab requires a curve"));
        };
        if matches!(curve, om9_core::geometry::Curve::Circle { .. }) {
            Ok(Self::Circle(om9_core::slab::circle_source(
                curve,
                entity.placement,
                limits,
            )?))
        } else if matches!(curve,om9_core::geometry::Curve::Nurbs(n) if n.degree==1) {
            match om9_core::slab::placed_closed_polyline(curve, entity.placement, limits) {
                Ok(points) => Ok(Self::ClosedPolyline(points)),
                Err(_) => Ok(Self::Polyline(om9_core::slab::placed_open_polyline(
                    curve,
                    entity.placement,
                    limits,
                )?)),
            }
        } else if let om9_core::geometry::Curve::Nurbs(n) = curve {
            n.validate(limits)?;
            let shifted_seam = n.seam.is_some() || n.periodic;
            let mut n = if shifted_seam {
                om9_core::slab::normalize_closed_nurbs_for_slab(n, limits)?
            } else {
                n.clone()
            };
            for p in &mut n.poles {
                *p = entity.placement.point(*p)?;
            }
            let world = om9_core::geometry::Curve::Nurbs(n);
            world.validate(limits)?;
            if shifted_seam && !om9_core::curve_ops::is_closed(&world, 1e-7, limits)? {
                return Err(Error::Invalid("Slab shifted seam requires a closed curve"));
            }
            let om9_core::geometry::Curve::Nurbs(n) = &world else {
                unreachable!()
            };
            let capacity = n
                .knots
                .len()
                .checked_mul(2)
                .and_then(|v| v.checked_add(65))
                .ok_or(Error::Overflow("Slab preview samples"))?;
            let mut usage = om9_core::budget::Usage::array::<f64>(capacity)?;
            usage.add(
                om9_core::budget::Usage::array::<[f64; 3]>(capacity)?,
                limits,
            )?;
            let mut fractions = Vec::new();
            fractions
                .try_reserve_exact(capacity)
                .map_err(|_| Error::Budget("Slab preview samples"))?;
            fractions.extend((0..=64).map(|i| i as f64 / 64.));
            for pair in n.knots.windows(2) {
                let lo = pair[0].max(n.domain[0]);
                let hi = pair[1].min(n.domain[1]);
                if hi <= lo {
                    continue;
                }
                for u in [lo, lo + (hi - lo) * 0.5] {
                    let f = (u - n.domain[0]) / (n.domain[1] - n.domain[0]);
                    fractions.push(if n.reversed { 1. - f } else { f });
                }
            }
            fractions.sort_by(f64::total_cmp);
            fractions.dedup_by(|a, b| *a == *b);
            let mut points = Vec::new();
            points
                .try_reserve_exact(fractions.len())
                .map_err(|_| Error::Budget("Slab preview points"))?;
            for f in &fractions {
                points.push(om9_core::curve_ops::point_at(&world, *f, limits)?);
            }
            let closed = (0..3)
                .map(|i| (points[0][i] - points[points.len() - 1][i]).powi(2))
                .sum::<f64>()
                .sqrt()
                <= 1e-7;
            Ok(Self::Smooth {
                curve: world,
                points,
                fractions,
                closed,
            })
        } else {
            match om9_core::slab::placed_closed_polyline(curve, entity.placement, limits) {
                Ok(points) => Ok(Self::ClosedPolyline(points)),
                Err(_) => Ok(Self::Polyline(om9_core::slab::placed_open_polyline(
                    curve,
                    entity.placement,
                    limits,
                )?)),
            }
        }
    }
    pub(super) fn normal(
        &self,
        preferred: [f64; 3],
        limits: &om9_core::budget::Limits,
    ) -> Result<[f64; 3]> {
        match self {
            Self::Polyline(points) | Self::ClosedPolyline(points) => {
                om9_core::slab::source_plane_normal(points, preferred)
            }
            Self::Smooth {
                curve: om9_core::geometry::Curve::Nurbs(n),
                ..
            } => om9_core::slab::nurbs_source_plane_normal(n, preferred, limits),
            Self::Smooth { .. } => Err(Error::Invalid("Slab smooth source")),
            Self::Circle(circle) => Ok(
                if (0..3).map(|i| circle.normal[i] * preferred[i]).sum::<f64>() < 0. {
                    circle.normal.map(|v| -v)
                } else {
                    circle.normal
                },
            ),
        }
    }
    pub(super) fn anchor(&self) -> [f64; 3] {
        match self {
            Self::Polyline(p) | Self::ClosedPolyline(p) | Self::Smooth { points: p, .. } => p[0],
            Self::Circle(c) => c.center,
        }
    }
    pub(super) fn axis(&self) -> [f64; 3] {
        match self {
            Self::Polyline(p) | Self::ClosedPolyline(p) | Self::Smooth { points: p, .. } => {
                let d = std::array::from_fn::<_, 3, _>(|i| p[1][i] - p[0][i]);
                let l = d[0].hypot(d[1]).hypot(d[2]);
                d.map(|v| v / l)
            }
            Self::Circle(c) => c.x,
        }
    }
    pub(super) fn offset(
        &self,
        normal: [f64; 3],
        point: [f64; 3],
        limits: &om9_core::budget::Limits,
    ) -> Result<f64> {
        match self {
            Self::Polyline(p) | Self::ClosedPolyline(p) => {
                om9_core::slab::signed_offset_from_pick(p, normal, point)
            }
            Self::Circle(c) => om9_core::slab::circle_offset_from_pick(c, normal, point),
            Self::Smooth {
                curve,
                points,
                fractions,
                ..
            } => {
                let (fraction, nearest) =
                    slab_smooth_nearest(curve, points, fractions, point, normal, limits)?;
                let t = om9_core::curve_ops::tangent_at(curve, fraction, limits)?;
                let side = [
                    t[1] * normal[2] - t[2] * normal[1],
                    t[2] * normal[0] - t[0] * normal[2],
                    t[0] * normal[1] - t[1] * normal[0],
                ];
                let length = side[0].hypot(side[1]).hypot(side[2]);
                if length < 1e-12 {
                    return Err(Error::Invalid("Slab NURBS tangent"));
                }
                let offset = (0..3)
                    .map(|i| (point[i] - nearest[i]) * side[i] / length)
                    .sum::<f64>();
                if !offset.is_finite() || offset.abs() < 1e-7 {
                    return Err(Error::Invalid("Slab offset must be nonzero"));
                }
                Ok(offset)
            }
        }
    }
    pub(super) fn supports_plane(&self, n: [f64; 3]) -> Result<()> {
        match self {
            Self::Polyline(p) | Self::ClosedPolyline(p) => {
                for pair in p.windows(2) {
                    om9_core::slab::line_slab(pair[0], pair[1], n, 1., 1., false)?;
                }
                Ok(())
            }
            Self::Smooth {
                curve: om9_core::geometry::Curve::Nurbs(curve),
                ..
            } => {
                let base = curve.poles[0];
                if curve
                    .poles
                    .iter()
                    .any(|p| (0..3).map(|i| (p[i] - base[i]) * n[i]).sum::<f64>().abs() > 1e-7)
                {
                    Err(Error::Invalid("Slab NURBS must lie in CPlane"))
                } else {
                    Ok(())
                }
            }
            Self::Smooth { .. } => Err(Error::Invalid("Slab smooth source")),
            Self::Circle(c) => {
                if ((0..3).map(|i| c.normal[i] * n[i]).sum::<f64>().abs() - 1.).abs() <= 1e-9 {
                    Ok(())
                } else {
                    Err(Error::Invalid("Slab circle must lie in CPlane"))
                }
            }
        }
    }
}

#[cfg(feature = "occt-brep")]
pub(super) fn sphere_curve_source(
    entity: &om9_core::model::Entity,
    edge: Option<usize>,
    limits: &om9_core::budget::Limits,
) -> Result<om9_core::model::Entity> {
    let mut result = entity.clone();
    if let Geometry::Brep(brep) = &entity.geometry {
        let index = match edge {
            Some(index) => index,
            None if brep.display.edges.len() == 1 => 0,
            None => return Err(Error::Invalid("B-Rep path requires EdgeN")),
        };
        result.geometry = Geometry::Curve(
            match editing::straight_edge_endpoints(brep, index, limits) {
                Ok((start, end)) => om9_core::geometry::Curve::Line {
                    start,
                    end,
                    domain: [0., 1.],
                },
                Err(_) => editing::circular_edge(brep, index, limits)
                    .map_err(|error| Error::Io(format!("Sphere AroundCurve edge: {error}")))?,
            },
        );
    } else if !matches!(entity.geometry, Geometry::Curve(_)) {
        return Err(Error::Invalid(
            "Sphere AroundCurve requires a curve or B-Rep edge",
        ));
    }
    Ok(result)
}

#[cfg(feature = "occt-brep")]
pub(super) fn sphere_world_curve_samples(
    entity: &om9_core::model::Entity,
    limits: &om9_core::budget::Limits,
) -> Result<Vec<[f64; 3]>> {
    let Geometry::Curve(curve) = &entity.geometry else {
        return Err(Error::Invalid("sphere requires curve"));
    };
    let mut points = om9_core::surface_kernel::sample_curve(curve, 128, limits)?;
    for point in &mut points {
        *point = entity.placement.point(*point)?;
    }
    Ok(points)
}

#[cfg(feature = "occt-brep")]
pub(super) fn sphere_closest_world_curve_point(
    entity: &om9_core::model::Entity,
    pick: [f64; 3],
    limits: &om9_core::budget::Limits,
) -> Result<[f64; 3]> {
    let Geometry::Curve(curve) = &entity.geometry else {
        return Err(Error::Invalid("sphere requires curve"));
    };
    let evaluate = |fraction| -> Result<([f64; 3], f64)> {
        let point = entity
            .placement
            .point(om9_core::curve_ops::point_at(curve, fraction, limits)?)?;
        let distance = (0..3).map(|i| (point[i] - pick[i]).powi(2)).sum::<f64>();
        if !distance.is_finite() {
            return Err(Error::Invalid("sphere curve pick"));
        }
        Ok((point, distance))
    };
    const SAMPLES: usize = 128;
    let mut best = 0;
    let mut best_point = evaluate(0.0)?;
    for i in 1..=SAMPLES {
        let candidate = evaluate(i as f64 / SAMPLES as f64)?;
        if candidate.1 < best_point.1 {
            best = i;
            best_point = candidate;
        }
    }
    let mut low = best.saturating_sub(1) as f64 / SAMPLES as f64;
    let mut high = (best + 1).min(SAMPLES) as f64 / SAMPLES as f64;
    let golden = 0.618_033_988_749_895;
    let mut a = high - golden * (high - low);
    let mut b = low + golden * (high - low);
    let mut va = evaluate(a)?;
    let mut vb = evaluate(b)?;
    for _ in 0..80 {
        if va.1 < vb.1 {
            high = b;
            b = a;
            vb = va;
            a = high - golden * (high - low);
            va = evaluate(a)?;
        } else {
            low = a;
            a = b;
            va = vb;
            b = low + golden * (high - low);
            vb = evaluate(b)?;
        }
    }
    for candidate in [va, vb] {
        if candidate.1 < best_point.1 {
            best_point = candidate;
        }
    }
    Ok(best_point.0)
}

#[cfg(feature = "occt-brep")]
pub(super) fn polygon_area(points: &[om9_core::geometry::Point3]) -> f64 {
    let mut area = 0.0;
    for index in 0..points.len() {
        let a = points[index];
        let b = points[(index + 1) % points.len()];
        area += a[0] * b[1] - b[0] * a[1];
    }
    0.5 * area
}

#[cfg(feature = "occt-brep")]
pub(super) fn inside(
    point: om9_core::geometry::Point3,
    polygon: &[om9_core::geometry::Point3],
) -> bool {
    let mut inside = false;
    let mut j = polygon.len() - 1;
    for i in 0..polygon.len() {
        let a = polygon[i];
        let b = polygon[j];
        if (a[1] > point[1]) != (b[1] > point[1])
            && point[0] < (b[0] - a[0]) * (point[1] - a[1]) / (b[1] - a[1]) + a[0]
        {
            inside = !inside;
        }
        j = i;
    }
    inside
}
