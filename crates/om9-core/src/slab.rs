// SPDX-License-Identifier: LGPL-2.1-or-later
//! Host-independent geometry foundation for the straight-line subset of Slab.
//! The interactive command and general-curve offset are intentionally not provided here.
use crate::{
    Error, Result,
    geometry::{Point3, finite},
};

/// A planar closed perimeter and an extrusion vector in world millimeters.
#[derive(Debug, Clone, PartialEq)]
pub struct LineSlab {
    pub perimeter: [Point3; 4],
    pub extrusion: Point3,
}

/// Build the straight-line slab cross section in a supplied plane.
/// `offset` is signed in the direction line × normal; `height` is signed along normal.
/// When `both_sides` is true, the solid spans `-height..height` along normal.
pub fn line_slab(
    start: Point3,
    end: Point3,
    normal: Point3,
    offset: f64,
    height: f64,
    both_sides: bool,
) -> Result<LineSlab> {
    finite(start)?;
    finite(end)?;
    finite(normal)?;
    if !offset.is_finite()
        || !height.is_finite()
        || offset.abs() < 1e-7
        || height.abs() < 1e-7
        || offset.abs() > 1e9
        || height.abs() > 1e9
    {
        return Err(Error::Invalid(
            "Slab offset and height must be finite and nonzero",
        ));
    }
    let direction = std::array::from_fn::<_, 3, _>(|i| end[i] - start[i]);
    let length = direction[0].hypot(direction[1]).hypot(direction[2]);
    let norm = normal[0].hypot(normal[1]).hypot(normal[2]);
    if !length.is_finite() || !norm.is_finite() || length < 1e-7 || norm < 1e-12 {
        return Err(Error::Invalid(
            "Slab line and plane normal must be nondegenerate",
        ));
    }
    let n = normal.map(|x| x / norm);
    if (0..3).map(|i| direction[i] * n[i]).sum::<f64>().abs() > length * 1e-9 {
        return Err(Error::Invalid("Slab line must lie in the offset plane"));
    }
    let side = [
        direction[1] * n[2] - direction[2] * n[1],
        direction[2] * n[0] - direction[0] * n[2],
        direction[0] * n[1] - direction[1] * n[0],
    ]
    .map(|x| x / length * offset);
    let shift = if both_sides { -height } else { 0. };
    let a = std::array::from_fn(|i| start[i] + shift * n[i]);
    let b = std::array::from_fn(|i| end[i] + shift * n[i]);
    let c = std::array::from_fn(|i| b[i] + side[i]);
    let d = std::array::from_fn(|i| a[i] + side[i]);
    let extrusion = n.map(|x| x * height * if both_sides { 2. } else { 1. });
    for point in [a, b, c, d, extrusion] {
        finite(point)?;
    }
    Ok(LineSlab {
        perimeter: [a, b, c, d],
        extrusion,
    })
}

/// Derive an open polyline's plane normal. A collinear source uses the preferred
/// normal projected perpendicular to its direction; truly ambiguous directions fail.
pub fn source_plane_normal(points: &[Point3], preferred: Point3) -> Result<Point3> {
    if points.len() < 2 || points.len() > 1024 {
        return Err(Error::Invalid("Slab source points"));
    }
    source_plane_normal_impl(points, preferred)
}

/// Infer a plane from the entire validated NURBS control hull, rather than
/// samples that can miss short knot spans. Conservative for trimmed curves.
pub fn nurbs_source_plane_normal(
    nurbs: &crate::geometry::NurbsCurve,
    preferred: Point3,
    limits: &crate::budget::Limits,
) -> Result<Point3> {
    nurbs.validate(limits)?;
    source_plane_normal_impl(&nurbs.poles, preferred)
}

fn source_plane_normal_impl(points: &[Point3], preferred: Point3) -> Result<Point3> {
    if points.len() < 2 {
        return Err(Error::Invalid("Slab source points"));
    }
    finite(preferred)?;
    let base = points[0];
    let sub = |a: Point3, b: Point3| std::array::from_fn::<_, 3, _>(|i| a[i] - b[i]);
    let mut direction = [0.; 3];
    let mut len = 0.;
    for p in points {
        finite(*p)?;
        let d = sub(*p, base);
        let l = d[0].hypot(d[1]).hypot(d[2]);
        if l > len {
            direction = d;
            len = l;
        }
    }
    if !len.is_finite() || len < 1e-7 {
        return Err(Error::Invalid("Slab source is degenerate"));
    }
    let u = direction.map(|x| x / len);
    let cross = |a: Point3, b: Point3| {
        [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ]
    };
    let mut normal = [0.; 3];
    let mut magnitude = 0.;
    for p in points {
        let v = cross(u, sub(*p, base));
        let m = v[0].hypot(v[1]).hypot(v[2]);
        if m > magnitude {
            normal = v;
            magnitude = m;
        }
    }
    if magnitude <= len * 1e-9 {
        let dot = (0..3).map(|i| preferred[i] * u[i]).sum::<f64>();
        normal = std::array::from_fn(|i| preferred[i] - dot * u[i]);
        magnitude = normal[0].hypot(normal[1]).hypot(normal[2]);
    }
    if !magnitude.is_finite() || magnitude < 1e-12 {
        return Err(Error::Invalid("Slab source plane is ambiguous"));
    }
    normal = normal.map(|x| x / magnitude);
    if (0..3).map(|i| normal[i] * preferred[i]).sum::<f64>() < 0. {
        normal = normal.map(|x| -x);
    }
    for p in points {
        if (0..3)
            .map(|i| (p[i] - base[i]) * normal[i])
            .sum::<f64>()
            .abs()
            > len.max(1.) * 1e-9
        {
            return Err(Error::Invalid("Slab source must be planar"));
        }
    }
    Ok(normal)
}

/// Signed distance from a pick to the closest segment, measured along segment × plane normal.
/// A tied nearest corner with inconsistent signed distances is ambiguous and is refused.
pub fn signed_offset_from_pick(points: &[Point3], normal: Point3, pick: Point3) -> Result<f64> {
    finite(pick)?;
    if points.len() < 2 || points.len() > 1024 {
        return Err(Error::Invalid("Slab source points"));
    }
    let nlen = normal[0].hypot(normal[1]).hypot(normal[2]);
    if !nlen.is_finite() || nlen < 1e-12 {
        return Err(Error::Invalid("Slab plane normal"));
    }
    let n = normal.map(|x| x / nlen);
    let mut best: Option<(f64, f64)> = None;
    let mut ambiguous = false;
    for pair in points.windows(2) {
        let _ = line_slab(pair[0], pair[1], n, 1., 1., false)?;
        let d = std::array::from_fn::<_, 3, _>(|i| pair[1][i] - pair[0][i]);
        let length2 = (0..3).map(|i| d[i] * d[i]).sum::<f64>();
        let t =
            ((0..3).map(|i| (pick[i] - pair[0][i]) * d[i]).sum::<f64>() / length2).clamp(0., 1.);
        let delta = std::array::from_fn::<_, 3, _>(|i| pick[i] - (pair[0][i] + t * d[i]));
        let distance2 = (0..3).map(|i| delta[i] * delta[i]).sum::<f64>();
        if !distance2.is_finite() {
            return Err(Error::Invalid("Slab pick distance"));
        }
        let length = length2.sqrt();
        let side = [
            (d[1] * n[2] - d[2] * n[1]) / length,
            (d[2] * n[0] - d[0] * n[2]) / length,
            (d[0] * n[1] - d[1] * n[0]) / length,
        ];
        let signed = (0..3).map(|i| delta[i] * side[i]).sum::<f64>();
        match best {
            None => best = Some((distance2, signed)),
            Some((distance, _)) if distance2 < distance - 1e-12 * distance.max(1.) => {
                best = Some((distance2, signed));
                ambiguous = false;
            }
            Some((distance, old))
                if (distance2 - distance).abs() <= 1e-12 * distance.max(1.)
                    && (signed - old).abs() > 1e-7 =>
            {
                ambiguous = true;
            }
            _ => {}
        }
    }
    if ambiguous {
        return Err(Error::Invalid("Slab pick is ambiguous at a corner"));
    }
    let (_, offset) = best.ok_or(Error::Invalid("Slab source points"))?;
    if !offset.is_finite() || offset.abs() < 1e-7 {
        return Err(Error::Invalid("Slab offset must be nonzero"));
    }
    Ok(offset)
}

/// World-space exact annular slab parameters for a fully closed circular source.
#[derive(Debug, Clone, PartialEq)]
pub struct CircleSlab {
    pub outer_radius: f64,
    pub inner_radius: f64,
    pub height: f64,
    pub placement: crate::geometry::Transform,
}

/// Validated world-space frame for a full circular Slab source.
#[derive(Debug, Clone, PartialEq)]
pub struct CircleSource {
    pub center: Point3,
    pub x: Point3,
    pub y: Point3,
    pub normal: Point3,
    pub radius: f64,
    pub angles: [f64; 2],
}

pub fn circle_source(
    curve: &crate::geometry::Curve,
    placement: crate::geometry::Transform,
    limits: &crate::budget::Limits,
) -> Result<CircleSource> {
    curve.validate(limits)?;
    let crate::geometry::Curve::Circle {
        center,
        x,
        y,
        radius,
        domain,
        angle_domain,
    } = curve
    else {
        return Err(Error::Invalid("Slab needs a circle"));
    };
    let angles = angle_domain.unwrap_or(*domain);
    let span = angles[1] - angles[0];
    if !span.is_finite() || span <= 1e-9 || span > std::f64::consts::TAU + 1e-8 {
        return Err(Error::Invalid("Slab circular domain"));
    }
    let origin = placement.point(*center)?;
    let px = placement.point(std::array::from_fn(|i| center[i] + x[i]))?;
    let py = placement.point(std::array::from_fn(|i| center[i] + y[i]))?;
    let raw_x = std::array::from_fn::<_, 3, _>(|i| px[i] - origin[i]);
    let raw_y = std::array::from_fn::<_, 3, _>(|i| py[i] - origin[i]);
    let sx = raw_x[0].hypot(raw_x[1]).hypot(raw_x[2]);
    let sy = raw_y[0].hypot(raw_y[1]).hypot(raw_y[2]);
    if !sx.is_finite()
        || !sy.is_finite()
        || sx < 1e-12
        || sy < 1e-12
        || (sx - sy).abs() > sx.max(sy) * 1e-9
    {
        return Err(Error::Unsupported(
            "Slab circle placement must preserve circularity",
        ));
    }
    let ux = raw_x.map(|v| v / sx);
    let uy = raw_y.map(|v| v / sy);
    if (0..3).map(|i| ux[i] * uy[i]).sum::<f64>().abs() > 1e-9 {
        return Err(Error::Unsupported(
            "Slab circle placement must preserve circularity",
        ));
    }
    let normal = [
        ux[1] * uy[2] - ux[2] * uy[1],
        ux[2] * uy[0] - ux[0] * uy[2],
        ux[0] * uy[1] - ux[1] * uy[0],
    ];
    let radius = radius * sx;
    if !radius.is_finite() || !(1e-7..=1e9).contains(&radius) {
        return Err(Error::Invalid("Slab circle radius"));
    }
    Ok(CircleSource {
        center: origin,
        x: ux,
        y: uy,
        normal,
        radius,
        angles,
    })
}

/// Signed radial offset at a picked point for a validated full circle.
pub fn circle_offset_from_pick(source: &CircleSource, normal: Point3, pick: Point3) -> Result<f64> {
    finite(pick)?;
    finite(normal)?;
    let delta = std::array::from_fn::<_, 3, _>(|i| pick[i] - source.center[i]);
    let u = (0..3).map(|i| delta[i] * source.x[i]).sum::<f64>();
    let v = (0..3).map(|i| delta[i] * source.y[i]).sum::<f64>();
    let alignment = (0..3).map(|i| normal[i] * source.normal[i]).sum::<f64>();
    if (alignment.abs() - 1.).abs() > 1e-9 {
        return Err(Error::Invalid("Slab circle plane"));
    }
    let angle = v.atan2(u);
    let span = source.angles[1] - source.angles[0];
    let radial = if span >= std::f64::consts::TAU - 1e-8 {
        u.hypot(v)
    } else {
        let tau = std::f64::consts::TAU;
        let k = ((source.angles[0] - angle) / tau).round();
        let mut nearest = source.angles[0];
        let mut best = f64::INFINITY;
        for step in -1..=2 {
            let candidate =
                (angle + (k + step as f64) * tau).clamp(source.angles[0], source.angles[1]);
            let distance = (u - source.radius * candidate.cos()).powi(2)
                + (v - source.radius * candidate.sin()).powi(2);
            if distance < best {
                best = distance;
                nearest = candidate;
            }
        }
        u * nearest.cos() + v * nearest.sin()
    };
    let offset = (radial - source.radius) * alignment.signum();
    if !offset.is_finite() || offset.abs() < 1e-7 {
        return Err(Error::Invalid("Slab offset must be nonzero"));
    }
    Ok(offset)
}

/// Build an annulus from an exactly circular source under a circularity-preserving placement.
pub fn circle_slab(
    curve: &crate::geometry::Curve,
    placement: crate::geometry::Transform,
    normal: Point3,
    offset: f64,
    height: f64,
    both_sides: bool,
    limits: &crate::budget::Limits,
) -> Result<CircleSlab> {
    use crate::geometry::Curve;
    curve.validate(limits)?;
    finite(normal)?;
    let Curve::Circle {
        center,
        x,
        y,
        radius,
        domain,
        angle_domain,
    } = curve
    else {
        return Err(Error::Invalid("Slab needs a circle"));
    };
    let angles = angle_domain.unwrap_or(*domain);
    if ((angles[1] - angles[0]) - std::f64::consts::TAU).abs() > 1e-8 {
        return Err(Error::Unsupported("Slab currently requires a full circle"));
    }
    if !offset.is_finite()
        || !height.is_finite()
        || offset.abs() < 1e-7
        || height.abs() < 1e-7
        || offset.abs() > 1e9
        || height.abs() > 1e9
    {
        return Err(Error::Invalid(
            "Slab offset and height must be finite and nonzero",
        ));
    }
    let origin = placement.point(*center)?;
    let axis = |v: Point3| -> Result<Point3> {
        let p = placement.point(std::array::from_fn(|i| center[i] + v[i]))?;
        Ok(std::array::from_fn(|i| p[i] - origin[i]))
    };
    let raw_x = axis(*x)?;
    let raw_y = axis(*y)?;
    let sx = raw_x[0].hypot(raw_x[1]).hypot(raw_x[2]);
    let sy = raw_y[0].hypot(raw_y[1]).hypot(raw_y[2]);
    if !sx.is_finite()
        || !sy.is_finite()
        || sx < 1e-12
        || sy < 1e-12
        || (sx - sy).abs() > sx.max(sy) * 1e-9
    {
        return Err(Error::Unsupported(
            "Slab circle placement must preserve circularity",
        ));
    }
    let ux = raw_x.map(|v| v / sx);
    let uy = raw_y.map(|v| v / sy);
    if (0..3).map(|i| ux[i] * uy[i]).sum::<f64>().abs() > 1e-9 {
        return Err(Error::Unsupported(
            "Slab circle placement must preserve circularity",
        ));
    }
    let source_normal = [
        ux[1] * uy[2] - ux[2] * uy[1],
        ux[2] * uy[0] - ux[0] * uy[2],
        ux[0] * uy[1] - ux[1] * uy[0],
    ];
    let nl = normal[0].hypot(normal[1]).hypot(normal[2]);
    if nl < 1e-12 || !nl.is_finite() {
        return Err(Error::Invalid("Slab normal"));
    }
    let n = normal.map(|v| v / nl);
    let alignment = (0..3).map(|i| n[i] * source_normal[i]).sum::<f64>();
    if (alignment.abs() - 1.).abs() > 1e-9 {
        return Err(Error::Invalid("Slab circle must lie in offset plane"));
    }
    let r = radius * sx;
    let other = r + offset * alignment.signum();
    if !other.is_finite() || other < 1e-7 || other > 1e9 || r > 1e9 {
        return Err(Error::Invalid("Slab offset circle radius"));
    }
    let (inner, outer) = (r.min(other), r.max(other));
    let base_shift = if both_sides { -height } else { 0. };
    let extrude = height * if both_sides { 2. } else { 1. };
    let sign = extrude.signum();
    let z = n.map(|v| v * sign);
    let yaxis = [
        z[1] * ux[2] - z[2] * ux[1],
        z[2] * ux[0] - z[0] * ux[2],
        z[0] * ux[1] - z[1] * ux[0],
    ];
    let mut m = crate::geometry::Transform::identity().0;
    for i in 0..3 {
        m[i][0] = ux[i];
        m[i][1] = yaxis[i];
        m[i][2] = z[i];
        m[i][3] = origin[i] + base_shift * n[i];
    }
    let placement = crate::geometry::Transform(m);
    placement.validate()?;
    Ok(CircleSlab {
        outer_radius: outer,
        inner_radius: inner,
        height: extrude.abs(),
        placement,
    })
}

/// Exact closed profile, extrusion and base translation for an open circular arc.
#[derive(Debug, Clone, PartialEq)]
pub struct ArcSlab {
    pub profile: crate::geometry::Curve,
    pub direction: Point3,
    pub base_shift: Point3,
}

pub fn arc_slab(
    curve: &crate::geometry::Curve,
    normal: Point3,
    offset: f64,
    height: f64,
    both_sides: bool,
    limits: &crate::budget::Limits,
) -> Result<ArcSlab> {
    use crate::geometry::Curve;
    curve.validate(limits)?;
    finite(normal)?;
    let Curve::Circle {
        x,
        y,
        domain,
        angle_domain,
        ..
    } = curve
    else {
        return Err(Error::Invalid("Slab needs an arc"));
    };
    let angles = angle_domain.unwrap_or(*domain);
    let span = angles[1] - angles[0];
    if !span.is_finite() || span <= 1e-9 || span >= std::f64::consts::TAU - 1e-8 {
        return Err(Error::Unsupported("Slab arc must be open"));
    }
    if !height.is_finite()
        || height.abs() < 1e-7
        || height.abs() > 1e9
        || !offset.is_finite()
        || offset.abs() < 1e-7
        || offset.abs() > 1e9
    {
        return Err(Error::Invalid("Slab offset and height"));
    }
    let nl = normal[0].hypot(normal[1]).hypot(normal[2]);
    if !nl.is_finite() || nl < 1e-12 {
        return Err(Error::Invalid("Slab plane"));
    }
    let n = normal.map(|v| v / nl);
    let source_normal = [
        x[1] * y[2] - x[2] * y[1],
        x[2] * y[0] - x[0] * y[2],
        x[0] * y[1] - x[1] * y[0],
    ];
    if ((0..3).map(|i| n[i] * source_normal[i]).sum::<f64>().abs() - 1.).abs() > 1e-9 {
        return Err(Error::Invalid("Slab arc must lie in plane"));
    }
    let other = crate::curve_modify::offset_curve(curve, offset, n, 16, limits)?;
    let a = crate::curve_ops::point_at(curve, 0., limits)?;
    let b = crate::curve_ops::point_at(curve, 1., limits)?;
    let c = crate::curve_ops::point_at(&other, 1., limits)?;
    let d = crate::curve_ops::point_at(&other, 0., limits)?;
    let reverse = crate::curve_ops::reverse_curve(&other, limits)?;
    let profile = Curve::Polycurve {
        segments: vec![
            curve.clone(),
            Curve::Line {
                start: b,
                end: c,
                domain: [0., 1.],
            },
            reverse,
            Curve::Line {
                start: d,
                end: a,
                domain: [0., 1.],
            },
        ],
        reversed: false,
    };
    profile.validate(limits)?;
    let direction = n.map(|v| v * height * if both_sides { 2. } else { 1. });
    let base_shift = n.map(|v| if both_sides { -height * v } else { 0. });
    Ok(ArcSlab {
        profile,
        direction,
        base_shift,
    })
}

/// Construct a bounded sampled-fit strip for an open planar nonperiodic NURBS.
/// The returned fit is checked at 33 samples against a nearest-point tolerance;
/// this is not a proof of global deviation or topology between samples.
pub fn nurbs_slab(
    curve: &crate::geometry::Curve,
    normal: Point3,
    offset: f64,
    height: f64,
    both_sides: bool,
    tolerance: f64,
    limits: &crate::budget::Limits,
) -> Result<ArcSlab> {
    use crate::geometry::Curve;
    curve.validate(limits)?;
    finite(normal)?;
    let Curve::Nurbs(nurbs) = curve else {
        return Err(Error::Invalid("Slab needs NURBS"));
    };
    if nurbs.periodic || nurbs.seam.is_some() {
        return Err(Error::Unsupported("Slab needs nonperiodic NURBS"));
    }
    if !height.is_finite()
        || height.abs() < 1e-7
        || height.abs() > 1e9
        || !offset.is_finite()
        || offset.abs() < 1e-7
        || offset.abs() > 1e9
        || !tolerance.is_finite()
        || tolerance <= 0.
    {
        return Err(Error::Invalid("Slab NURBS parameters"));
    }
    let nl = normal[0].hypot(normal[1]).hypot(normal[2]);
    if !nl.is_finite() || nl < 1e-12 {
        return Err(Error::Invalid("Slab plane"));
    }
    let n = normal.map(|v| v / nl);
    let a = crate::curve_ops::point_at(curve, 0., limits)?;
    let b = crate::curve_ops::point_at(curve, 1., limits)?;
    // Positive NURBS weights keep the entire curve in its control-polygon hull.
    // Checking that hull is conservative, but prevents narrow out-of-plane knot
    // spans from being missed by the fit samples below.
    if nurbs.poles.iter().any(|p| {
        (0..3).map(|j| (p[j] - a[j]) * n[j]).sum::<f64>().abs() > tolerance
    }) {
        return Err(Error::Invalid("Slab NURBS control hull must be planar"));
    }
    if (0..3).map(|i| (a[i] - b[i]).powi(2)).sum::<f64>().sqrt() < 1e-7 {
        return Err(Error::Unsupported("Slab closed NURBS"));
    }
    let other = if nurbs.reversed {
        let mut forward = nurbs.clone();
        forward.reversed = false;
        let fitted =
            crate::curve_modify::offset_curve(&Curve::Nurbs(forward), -offset, n, 128, limits)?;
        crate::curve_ops::reverse_curve(&fitted, limits)?
    } else {
        crate::curve_modify::offset_curve(curve, offset, n, 128, limits)?
    };
    for i in 0..=32 {
        let fraction = i as f64 / 32.;
        let point = crate::curve_ops::point_at(curve, fraction, limits)?;
        if (0..3).map(|j| (point[j] - a[j]) * n[j]).sum::<f64>().abs() > tolerance {
            return Err(Error::Invalid("Slab NURBS must be planar"));
        }
        let t = crate::curve_ops::tangent_at(curve, fraction, limits)?;
        let side = [
            t[1] * n[2] - t[2] * n[1],
            t[2] * n[0] - t[0] * n[2],
            t[0] * n[1] - t[1] * n[0],
        ];
        let norm = side[0].hypot(side[1]).hypot(side[2]);
        if norm < 1e-12 {
            return Err(Error::Invalid("Slab NURBS tangent"));
        }
        let expected = std::array::from_fn(|j| point[j] + offset * side[j] / norm);
        let (_, _, distance) =
            crate::curve_surface_ops::closest_point_on_curve(&other, expected, limits)?;
        if !distance.is_finite() || distance > tolerance {
            return Err(Error::Invalid("Slab offset fit exceeds tolerance"));
        }
    }
    let c = crate::curve_ops::point_at(&other, 1., limits)?;
    let d = crate::curve_ops::point_at(&other, 0., limits)?;
    let reverse = crate::curve_ops::reverse_curve(&other, limits)?;
    let profile = Curve::Polycurve {
        segments: vec![
            curve.clone(),
            Curve::Line {
                start: b,
                end: c,
                domain: [0., 1.],
            },
            reverse,
            Curve::Line {
                start: d,
                end: a,
                domain: [0., 1.],
            },
        ],
        reversed: false,
    };
    profile.validate(limits)?;
    Ok(ArcSlab {
        profile,
        direction: n.map(|v| v * height * if both_sides { 2. } else { 1. }),
        base_shift: n.map(|v| if both_sides { -height * v } else { 0. }),
    })
}

/// Closed NURBS Slab rings. Offset and topology checks are sampled, not certified.
pub struct ClosedNurbsSlab {
    pub outer: crate::geometry::Curve,
    pub inner: crate::geometry::Curve,
    pub direction: Point3,
    pub base_shift: Point3,
}

/// Convert a certified full-domain periodic basis to an equivalent closed
/// nonperiodic record, retaining the source unchanged. This supports the
/// explicit repeated-control-point representation, not arbitrary metadata.
pub fn normalize_closed_nurbs_for_slab(
    source:&crate::geometry::NurbsCurve,limits:&crate::budget::Limits,
) -> Result<crate::geometry::NurbsCurve> {
    source.validate(limits)?;
    if (source.periodic || source.seam.is_some()) && source.domain != [source.knots[source.degree],source.knots[source.poles.len()]] {
        return Err(Error::Unsupported("Slab shifted seam needs a full NURBS domain"));
    }
    if source.periodic {
        source.certify_repeated_periodic(limits)?;
    }
    let mut base=source.clone();base.periodic=false;base.seam=None;
    if (source.periodic || source.seam.is_some())
        && !crate::curve_ops::is_closed(&crate::geometry::Curve::Nurbs(base.clone()),1e-7,limits)? {
        return Err(Error::Invalid("Slab seam requires a closed basis"));
    }
    Ok(base)
}

pub fn closed_nurbs_slab(
    curve: &crate::geometry::Curve, normal: Point3, offset: f64, height: f64,
    both_sides: bool, tolerance: f64, limits: &crate::budget::Limits,
) -> Result<ClosedNurbsSlab> {
    use crate::geometry::Curve;
    curve.validate(limits)?;finite(normal)?;
    let Curve::Nurbs(source)=curve else {return Err(Error::Invalid("Slab needs NURBS"));};
    if source.degree<2 {return Err(Error::Unsupported("Slab needs a smooth closed NURBS"));}
    // Seam changes parameter origin only. Work with the same closed basis at its
    // original seam; this avoids depending on a shifted-seam native extrusion.
    let normalized=Curve::Nurbs(normalize_closed_nurbs_for_slab(source,limits)?);
    let curve=&normalized;
    let Curve::Nurbs(nurbs)=curve else {unreachable!()};
    if !height.is_finite() || height.abs()<1e-7 || height.abs()>1e9 || !offset.is_finite()
        || offset.abs()<1e-7 || offset.abs()>1e9 || !tolerance.is_finite() || tolerance<=0. {
        return Err(Error::Invalid("Slab closed NURBS parameters"));
    }
    let nl=normal[0].hypot(normal[1]).hypot(normal[2]);
    if !nl.is_finite() || nl<1e-12 {return Err(Error::Invalid("Slab plane"));}
    let n=normal.map(|v|v/nl);let start=crate::curve_ops::point_at(curve,0.,limits)?;
    let end=crate::curve_ops::point_at(curve,1.,limits)?;
    if (0..3).map(|i|(start[i]-end[i]).powi(2)).sum::<f64>().sqrt()>1e-7 {
        return Err(Error::Invalid("Slab NURBS must be closed"));
    }
    if nurbs.poles.iter().any(|p|(0..3).map(|j|(p[j]-start[j])*n[j]).sum::<f64>().abs()>tolerance) {
        return Err(Error::Invalid("Slab NURBS control hull must be planar"));
    }
    let samples=crate::curve_ops::sample_curve(curve,65,limits)?;
    // Reject sampled crossings, collapses and non-nested mitered approximation.
    let _=closed_polyline_slab(&samples,n,offset,height,both_sides,limits)?;
    let fitted=if nurbs.reversed {
        let mut forward=nurbs.clone();forward.reversed=false;
        let result=crate::curve_modify::offset_curve(&Curve::Nurbs(forward),-offset,n,128,limits)?;
        crate::curve_ops::reverse_curve(&result,limits)?
    } else {crate::curve_modify::offset_curve(curve,offset,n,128,limits)?};
    if !crate::curve_ops::is_closed(&fitted,1e-7,limits)? {return Err(Error::Invalid("Slab offset must close"));}
    for i in 0..=32 {
        let f=i as f64/32.;let p=crate::curve_ops::point_at(curve,f,limits)?;
        let t=crate::curve_ops::tangent_at(curve,f,limits)?;
        let side=[t[1]*n[2]-t[2]*n[1],t[2]*n[0]-t[0]*n[2],t[0]*n[1]-t[1]*n[0]];
        let length=side[0].hypot(side[1]).hypot(side[2]);if length<1e-12 {return Err(Error::Invalid("Slab NURBS tangent"));}
        let expected=std::array::from_fn(|j|p[j]+offset*side[j]/length);
        let (_,_,distance)=crate::curve_surface_ops::closest_point_on_curve(&fitted,expected,limits)?;
        if !distance.is_finite() || distance>tolerance {return Err(Error::Invalid("Slab offset fit exceeds tolerance"));}
    }
    let mut area=0.;for pair in samples.windows(2) {
        let a=std::array::from_fn::<_,3,_>(|i|pair[0][i]-start[i]);
        let b=std::array::from_fn::<_,3,_>(|i|pair[1][i]-start[i]);
        area+=(a[1]*b[2]-a[2]*b[1])*n[0]+(a[2]*b[0]-a[0]*b[2])*n[1]+(a[0]*b[1]-a[1]*b[0])*n[2];
    }
    if !area.is_finite() || area.abs()<1e-12 {return Err(Error::Invalid("Slab NURBS area"));}
    let (outer,inner)=if offset*area>0. {(fitted,curve.clone())} else {(curve.clone(),fitted)};
    Ok(ClosedNurbsSlab {outer,inner,direction:n.map(|v|v*height*if both_sides {2.} else {1.}),
        base_shift:n.map(|v|if both_sides {-height*v} else {0.})})
}

/// Slab perimeter for an open planar polyline. Straight offsets are joined at mitered
/// corners; degenerate, self-intersecting or excessive-miter boundaries are refused.
/// The result is a foundation, not an interactive Slab command.
pub fn polyline_slab(
    points: &[Point3],
    normal: Point3,
    offset: f64,
    height: f64,
    both_sides: bool,
    limits: &crate::budget::Limits,
) -> Result<(Vec<Point3>, Point3)> {
    if points.len() < 2 {
        return Err(Error::Invalid("Slab needs an open polyline"));
    }
    if points.len() > 1024 || points.len() as u64 > limits.max_points / 2 {
        return Err(Error::Budget("Slab polyline points"));
    }
    crate::budget::Usage::array::<Point3>(points.len() * 3)?.check(limits)?;
    let first = line_slab(points[0], points[1], normal, offset, height, both_sides)?;
    let n_len = normal[0].hypot(normal[1]).hypot(normal[2]);
    let n = normal.map(|x| x / n_len);
    let shift = if both_sides { -height } else { 0. };
    let mut sides = Vec::new();
    sides
        .try_reserve_exact(points.len() - 1)
        .map_err(|_| Error::Budget("Slab polyline sides"))?;
    for pair in points.windows(2) {
        let part = line_slab(pair[0], pair[1], normal, offset, height, both_sides)?;
        sides.push(std::array::from_fn::<_, 3, _>(|i| {
            (part.perimeter[3][i] - part.perimeter[0][i]) / offset
        }));
        if (0..3)
            .map(|i| (pair[0][i] - points[0][i]) * n[i])
            .sum::<f64>()
            .abs()
            > 1e-7
        {
            return Err(Error::Invalid("Slab polyline must be planar"));
        }
    }
    let mut boundary = Vec::new();
    boundary
        .try_reserve_exact(points.len() * 2)
        .map_err(|_| Error::Budget("Slab polyline boundary"))?;
    boundary.extend(
        points
            .iter()
            .map(|p| std::array::from_fn(|i| p[i] + shift * n[i])),
    );
    let last = points.len() - 1;
    boundary.push(std::array::from_fn(|i| {
        points[last][i] + shift * n[i] + offset * sides[last - 1][i]
    }));
    for index in (1..last).rev() {
        let a = sides[index - 1];
        let b = sides[index];
        let denominator = 1. + (0..3).map(|i| a[i] * b[i]).sum::<f64>();
        if denominator < 0.0002 {
            return Err(Error::Invalid("Slab miter is degenerate"));
        }
        let miter = std::array::from_fn::<_, 3, _>(|i| offset * (a[i] + b[i]) / denominator);
        if miter[0].hypot(miter[1]).hypot(miter[2]) > offset.abs() * 100. {
            return Err(Error::Invalid("Slab miter exceeds limit"));
        }
        boundary.push(std::array::from_fn(|i| {
            points[index][i] + shift * n[i] + miter[i]
        }));
    }
    boundary.push(std::array::from_fn(|i| {
        points[0][i] + shift * n[i] + offset * sides[0][i]
    }));
    let cross = |a: Point3, b: Point3| {
        (a[1] * b[2] - a[2] * b[1]) * n[0]
            + (a[2] * b[0] - a[0] * b[2]) * n[1]
            + (a[0] * b[1] - a[1] * b[0]) * n[2]
    };
    let sub = |a: Point3, b: Point3| std::array::from_fn::<_, 3, _>(|i| a[i] - b[i]);
    let count = boundary.len();
    let drop_axis = (0..3).max_by(|a,b| n[*a].abs().total_cmp(&n[*b].abs())).unwrap();
    for i in 0..count {
        let a = boundary[i];
        let b = boundary[(i + 1) % count];
        for j in i + 1..count {
            if j == i + 1 || i == 0 && j == count - 1 {
                continue;
            }
            let c = boundary[j];
            let d = boundary[(j + 1) % count];
            let ab = sub(b, a);
            let cd = sub(d, c);
            let o1 = cross(ab, sub(c, a));
            let o2 = cross(ab, sub(d, a));
            let o3 = cross(cd, sub(a, c));
            let o4 = cross(cd, sub(b, c));
            // Orientation tests alone classify disjoint collinear segments as
            // intersecting. Require overlap of their finite segment bounds too.
            let overlaps = (0..3).filter(|axis| *axis != drop_axis).all(|axis| {
                a[axis].min(b[axis]) <= c[axis].max(d[axis]) + 1e-9
                    && c[axis].min(d[axis]) <= a[axis].max(b[axis]) + 1e-9
            });
            // Collinear/tangent non-neighbours are deliberately refused as ambiguous.
            if overlaps
                && (o1.abs() < 1e-12 || o2.abs() < 1e-12 || o1.signum() != o2.signum())
                && (o3.abs() < 1e-12 || o4.abs() < 1e-12 || o3.signum() != o4.signum())
            {
                return Err(Error::Invalid("Slab boundary self-intersects or touches"));
            }
        }
    }
    Ok((boundary, first.extrusion))
}

/// Compute the nested mitered rings for a closed planar polyline Slab.
/// Closedness is represented by the last point repeating the first.
pub fn closed_polyline_slab(
    points: &[Point3], normal: Point3, offset: f64, height: f64, both_sides: bool,
    limits: &crate::budget::Limits,
) -> Result<(Vec<Point3>, Vec<Point3>, Point3)> {
    if points.len()<4 || points.len()>1025 || (points.len()*2) as u64>limits.max_points {
        return Err(Error::Invalid("Slab closed polygon points"));
    }
    crate::budget::Usage::array::<Point3>(points.len()*3)?.check(limits)?;
    let count=points.len()-1;
    let distance=|a:Point3,b:Point3| (0..3).map(|i|(a[i]-b[i]).powi(2)).sum::<f64>().sqrt();
    if distance(points[0],points[count])>1e-7 {return Err(Error::Invalid("Slab polygon must be closed"));}
    let first=line_slab(points[0],points[1],normal,offset,height,both_sides)?;
    let n_len=normal[0].hypot(normal[1]).hypot(normal[2]);let n=normal.map(|v|v/n_len);
    let shift=if both_sides {-height} else {0.};
    let mut sides=Vec::new();sides.try_reserve_exact(count).map_err(|_|Error::Budget("Slab closed sides"))?;
    for pair in points.windows(2) {
        let part=line_slab(pair[0],pair[1],normal,offset,height,both_sides)?;
        if (0..3).map(|i|(pair[0][i]-points[0][i])*n[i]).sum::<f64>().abs()>1e-7 {
            return Err(Error::Invalid("Slab polygon must be planar"));
        }
        sides.push(std::array::from_fn::<_,3,_>(|i|(part.perimeter[3][i]-part.perimeter[0][i])/offset));
    }
    let mut source=Vec::new();let mut shifted=Vec::new();
    source.try_reserve_exact(count).map_err(|_|Error::Budget("Slab closed source"))?;
    shifted.try_reserve_exact(count).map_err(|_|Error::Budget("Slab closed offset"))?;
    for i in 0..count {
        let a=sides[(i+count-1)%count];let b=sides[i];
        let denominator=1.+(0..3).map(|j|a[j]*b[j]).sum::<f64>();
        if denominator<0.0002 {return Err(Error::Invalid("Slab miter is degenerate"));}
        let m=std::array::from_fn::<_,3,_>(|j|offset*(a[j]+b[j])/denominator);
        if m[0].hypot(m[1]).hypot(m[2])>offset.abs()*100. {return Err(Error::Invalid("Slab miter exceeds limit"));}
        let p=std::array::from_fn::<_,3,_>(|j|points[i][j]+shift*n[j]);
        source.push(p);shifted.push(std::array::from_fn(|j|p[j]+m[j]));
    }
    for i in 0..count {
        let j=(i+1)%count;
        let direction=(0..3).map(|k|(source[j][k]-source[i][k])*(shifted[j][k]-shifted[i][k])).sum::<f64>();
        if direction<=1e-12 {return Err(Error::Invalid("Slab closed offset edge collapsed"));}
    }
    let drop=(0..3).max_by(|a,b|n[*a].abs().total_cmp(&n[*b].abs())).unwrap();
    let axes:Vec<usize>=(0..3).filter(|i|*i!=drop).collect();let x=axes[0];let y=axes[1];
    let orient=|a:Point3,b:Point3,c:Point3| (b[x]-a[x])*(c[y]-a[y])-(b[y]-a[y])*(c[x]-a[x]);
    let intersects=|a:Point3,b:Point3,c:Point3,d:Point3| {
        let box_overlap=[x,y].into_iter().all(|i|a[i].min(b[i])<=c[i].max(d[i])+1e-9 && c[i].min(d[i])<=a[i].max(b[i])+1e-9);
        let (o1,o2,o3,o4)=(orient(a,b,c),orient(a,b,d),orient(c,d,a),orient(c,d,b));
        box_overlap && (o1.abs()<1e-12 || o2.abs()<1e-12 || o1.signum()!=o2.signum())
            && (o3.abs()<1e-12 || o4.abs()<1e-12 || o3.signum()!=o4.signum())
    };
    for ring in [&source,&shifted] {
        for i in 0..count {for j in i+1..count {
            if j==i+1 || i==0 && j==count-1 {continue;}
            if intersects(ring[i],ring[(i+1)%count],ring[j],ring[(j+1)%count]) {
                return Err(Error::Invalid("Slab closed boundary self-intersects"));
            }
        }}
    }
    for i in 0..count {for j in 0..count {
        if intersects(source[i],source[(i+1)%count],shifted[j],shifted[(j+1)%count]) {
            return Err(Error::Invalid("Slab closed boundaries intersect"));
        }
    }}
    let area=|ring:&Vec<Point3>| (0..count).map(|i|orient(ring[0],ring[i],ring[(i+1)%count])).sum::<f64>();
    let a=area(&source);let b=area(&shifted);
    if a.abs()<1e-12 || b.abs()<1e-12 || a.signum()!=b.signum() {return Err(Error::Invalid("Slab closed offset collapsed"));}
    let inside=|p:Point3,ring:&Vec<Point3>| {
        let mut inside=false;
        for i in 0..count {let a=ring[i];let b=ring[(i+1)%count];
            if (a[y]>p[y])!=(b[y]>p[y]) && p[x]<(b[x]-a[x])*(p[y]-a[y])/(b[y]-a[y])+a[x] {inside=!inside;}
        } inside
    };
    let signed_area=a*n[drop]*if drop==1 {-1.} else {1.};
    let (outer,inner)=if offset*signed_area>0. {
        if !inside(source[0],&shifted) || b.abs()<=a.abs() {return Err(Error::Invalid("Slab closed outward offset is not nested"));}
        (shifted,source)
    } else {
        if !inside(shifted[0],&source) || a.abs()<=b.abs() {return Err(Error::Invalid("Slab closed inward offset is not nested"));}
        (source,shifted)
    };
    Ok((outer,inner,first.extrusion))
}

/// Resolve an open line or flat polycurve of connected lines in placed world coordinates.
/// Smooth, nested, closed and disconnected curves are deliberately rejected.
pub fn placed_open_polyline(
    curve: &crate::geometry::Curve,
    placement: crate::geometry::Transform,
    limits: &crate::budget::Limits,
) -> Result<Vec<Point3>> {
    placed_polyline(curve,placement,limits,false)
}

/// Resolve a closed polyline or a continuous degree-one NURBS in world coordinates.
pub fn placed_closed_polyline(
    curve: &crate::geometry::Curve,
    placement: crate::geometry::Transform,
    limits: &crate::budget::Limits,
) -> Result<Vec<Point3>> {
    placed_polyline(curve,placement,limits,true)
}

fn placed_polyline(
    curve: &crate::geometry::Curve, placement: crate::geometry::Transform,
    limits: &crate::budget::Limits, want_closed: bool,
) -> Result<Vec<Point3>> {
    use crate::geometry::Curve;
    curve.validate(limits)?;
    if let Curve::Nurbs(n) = curve {
        if n.degree != 1 || n.periodic || n.seam.is_some() {
            return Err(Error::Unsupported("Slab requires an open degree-one NURBS"));
        }
        if n.knots.windows(2).any(|k| k[0] == k[1] && k[0] > n.domain[0] && k[0] < n.domain[1]) {
            return Err(Error::Unsupported("Slab NURBS has a discontinuity"));
        }
        let count=n.knots.iter().filter(|k| **k > n.domain[0] && **k < n.domain[1]).count()+2;
        if count>1024 || count as u64>limits.max_points {return Err(Error::Budget("Slab source points"));}
        let mut usage=crate::budget::Usage::array::<Point3>(count)?;
        usage.add(crate::budget::Usage::array::<f64>(count)?,limits)?;
        let mut fractions=Vec::new();fractions.try_reserve_exact(count).map_err(|_|Error::Budget("Slab source fractions"))?;
        fractions.push(0.);
        for k in n.knots.iter().filter(|k| **k>n.domain[0] && **k<n.domain[1]) {
            let f=(*k-n.domain[0])/(n.domain[1]-n.domain[0]);
            fractions.push(if n.reversed {1.-f} else {f});
        }
        fractions.push(1.);fractions.sort_by(f64::total_cmp);
        let mut points=Vec::new();points.try_reserve_exact(count).map_err(|_|Error::Budget("Slab source points"))?;
        for f in fractions {points.push(placement.point(crate::curve_ops::point_at(curve,f,limits)?)?);}
        let closed=(0..3).map(|i|(points[0][i]-points[count-1][i]).powi(2)).sum::<f64>().sqrt()<=1e-7;
        if closed != want_closed {return Err(Error::Unsupported("Slab source closedness mismatch"));}
        return Ok(points);
    }
    let (segments, reversed): (&[Curve], bool) = match curve {
        Curve::Line { .. } => (std::slice::from_ref(curve), false),
        Curve::Polycurve { segments, reversed } => (segments, *reversed),
        _ => {
            return Err(Error::Unsupported(
                "Slab currently requires a line or open line polycurve",
            ));
        }
    };
    if segments.is_empty() || segments.len() >= 1024 {
        return Err(Error::Budget("Slab source segments"));
    }
    let count = segments.len() + 1;
    if count as u64 > limits.max_points {
        return Err(Error::Budget("Slab source points"));
    }
    crate::budget::Usage::array::<Point3>(count)?.check(limits)?;
    let mut points = Vec::new();
    points
        .try_reserve_exact(count)
        .map_err(|_| Error::Budget("Slab source points"))?;
    for (index, segment) in segments.iter().enumerate() {
        let Curve::Line { start, end, .. } = segment else {
            return Err(Error::Unsupported("Slab polycurve must contain only lines"));
        };
        let a = placement.point(*start)?;
        let b = placement.point(*end)?;
        if index == 0 {
            points.push(a);
        } else if (0..3)
            .map(|i| (points[index][i] - a[i]).powi(2))
            .sum::<f64>()
            .sqrt()
            > 1e-7
        {
            return Err(Error::Invalid("Slab polycurve is disconnected"));
        }
        points.push(b);
    }
    if reversed {
        points.reverse();
    }
    let closed = (0..3)
        .map(|i| (points[0][i] - points[count - 1][i]).powi(2))
        .sum::<f64>()
        .sqrt()
        <= 1e-7;
    if closed != want_closed {return Err(Error::Unsupported("Slab source closedness mismatch"));}
    Ok(points)
}
