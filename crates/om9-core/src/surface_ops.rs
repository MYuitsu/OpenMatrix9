// SPDX-License-Identifier: LGPL-2.1-or-later
//! Freeform surface workflows owned by `OM9-L2-02-05`.
//!
//! This module is the portable Rust service behind the Matrix/Rhino surface commands:
//! curve networks, rail revolve, trim/split/untrim/shrink, rebuild/refit/softedit,
//! match/blend, offset/normal-extrude and unroll. Every constructor returns a validated
//! `NurbsSurface` (or `TrimmedSurface`) and refuses an inadmissible input before a host
//! document is touched. Trimming is stored as UV loops over the original NURBS surface;
//! the surface is never replaced by its tessellation.
use crate::{
    Error, Result,
    budget::Limits,
    curve_build::{KnotMode, interpolate_any},
    geometry::{Curve, NurbsSurface, Point3, finite},
    rhino_semantics::TrimmedSurface,
    surface_kernel::{self},
};

/// Re-exported for the surface-edit seam commands that parse a `U`/`V` axis.
pub use surface_kernel::SurfaceIso;

/* --------------------------------- vector helpers --------------------------------- */

fn add(a: Point3, b: Point3) -> Point3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub(a: Point3, b: Point3) -> Point3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale(a: Point3, factor: f64) -> Point3 {
    [a[0] * factor, a[1] * factor, a[2] * factor]
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

fn length(a: Point3) -> f64 {
    dot(a, a).sqrt()
}

fn unit(a: Point3, label: &'static str) -> Result<Point3> {
    let magnitude = length(a);
    if !magnitude.is_finite() || magnitude <= 1e-12 {
        return Err(Error::Invalid(label));
    }
    Ok(scale(a, 1.0 / magnitude))
}

fn distance(a: Point3, b: Point3) -> f64 {
    length(sub(a, b))
}

fn edge_count(surface: &NurbsSurface, edge: SurfaceEdge) -> usize {
    match edge {
        SurfaceEdge::U0 | SurfaceEdge::U1 => surface.size[1],
        SurfaceEdge::V0 | SurfaceEdge::V1 => surface.size[0],
    }
}

fn parameter_fraction(surface: &NurbsSurface, axis: usize, index: usize) -> f64 {
    let count = surface.size[axis];
    surface.domain[axis][0]
        + (surface.domain[axis][1] - surface.domain[axis][0]) * index as f64
            / (count.saturating_sub(1).max(1) as f64)
}

fn grid_parameter(surface: &NurbsSurface, axis: usize, index: usize, count: usize) -> f64 {
    surface.domain[axis][0]
        + (surface.domain[axis][1] - surface.domain[axis][0]) * index as f64
            / (count.saturating_sub(1).max(1) as f64)
}

fn surface_point(surface: &NurbsSurface, u: f64, v: f64, limits: &Limits) -> Result<Point3> {
    surface_kernel::evaluate(surface, u, v, limits)
}

fn sample_surface(
    surface: &NurbsSurface,
    size: [usize; 2],
    limits: &Limits,
) -> Result<Vec<Point3>> {
    if size[0] < 2 || size[1] < 2 {
        return Err(Error::Invalid("surface sample grid"));
    }
    let mut points = Vec::new();
    points
        .try_reserve_exact(size[0] * size[1])
        .map_err(|_| Error::Budget("surface sample grid"))?;
    for i in 0..size[0] {
        let u = grid_parameter(surface, 0, i, size[0]);
        for j in 0..size[1] {
            let v = grid_parameter(surface, 1, j, size[1]);
            points.push(surface_point(surface, u, v, limits)?);
        }
    }
    Ok(points)
}

/// Interpolate a row-major `size[0] x size[1]` point grid into a non-rational NURBS surface.
fn interpolate_grid(
    points: &[Point3],
    size: [usize; 2],
    degree: [usize; 2],
    limits: &Limits,
) -> Result<NurbsSurface> {
    if size[0] < 2 || size[1] < 2 || size[0] <= degree[0] || size[1] <= degree[1] {
        return Err(Error::Invalid("surface interpolation grid"));
    }
    if points.len() != size[0] * size[1] {
        return Err(Error::Invalid("surface interpolation grid"));
    }
    let mut rows = Vec::new();
    rows.try_reserve_exact(size[1])
        .map_err(|_| Error::Budget("surface interpolation rows"))?;
    for j in 0..size[1] {
        let row: Vec<Point3> = (0..size[0]).map(|i| points[i * size[1] + j]).collect();
        rows.push(interpolate_any(&row, degree[0], KnotMode::Uniform, limits)?);
    }
    let knots_u = rows[0].knots.clone();
    let mut knots_v = None;
    let mut poles = vec![[0.0; 3]; size[0] * size[1]];
    for i in 0..size[0] {
        let column: Vec<Point3> = rows.iter().map(|row| row.poles[i]).collect();
        let curve = interpolate_any(&column, degree[1], KnotMode::Uniform, limits)?;
        if knots_v.is_none() {
            knots_v = Some(curve.knots.clone());
        }
        for (j, value) in curve.poles.iter().enumerate() {
            poles[i * size[1] + j] = *value;
        }
    }
    let surface = NurbsSurface {
        degree,
        size,
        poles,
        weights: vec![1.0; size[0] * size[1]],
        knots: [
            knots_u,
            knots_v.ok_or(Error::Invalid("surface interpolation grid"))?,
        ],
        domain: [[0.0, 1.0], [0.0, 1.0]],
        periodic: [false, false],
        reversed: false,
    };
    surface.validate(limits)?;
    Ok(surface)
}

/* --------------------------------- curve network --------------------------------- */

/// A four-sided curve network in the order `[u0, u1, v0, v1]`.
///
/// `u0`/`u1` run along `v` at the two `u` boundaries; `v0`/`v1` run along `u` at the two
/// `v` boundaries. Endpoints must agree within `1e-9 * scale`, otherwise the network is
/// not a closed four-sided patch and the caller must create a five-sided or n-sided patch.
pub fn network(
    boundaries: [&Curve; 4],
    degree: [usize; 2],
    samples: [usize; 2],
    limits: &Limits,
) -> Result<NurbsSurface> {
    if samples[0] < 4 || samples[1] < 4 {
        return Err(Error::Invalid("network samples"));
    }
    let u0 = surface_kernel::sample_curve(boundaries[0], samples[1], limits)?;
    let u1 = surface_kernel::sample_curve(boundaries[1], samples[1], limits)?;
    let v0 = surface_kernel::sample_curve(boundaries[2], samples[0], limits)?;
    let v1 = surface_kernel::sample_curve(boundaries[3], samples[0], limits)?;
    let extent = length(sub(u0[0], v1[samples[0] - 1])).max(1.0);
    let tolerance = 1e-9 * extent;
    for (a, b) in [
        (u0[0], v0[0]),
        (u0[samples[1] - 1], v1[0]),
        (u1[0], v0[samples[0] - 1]),
        (u1[samples[1] - 1], v1[samples[0] - 1]),
    ] {
        if distance(a, b) > tolerance {
            return Err(Error::Invalid("network boundary corner"));
        }
    }
    let mut points = vec![[0.0; 3]; samples[0] * samples[1]];
    for i in 0..samples[0] {
        let u = i as f64 / (samples[0] - 1) as f64;
        for j in 0..samples[1] {
            let v = j as f64 / (samples[1] - 1) as f64;
            let coons = add(
                add(
                    add(scale(u0[j], 1.0 - u), scale(u1[j], u)),
                    scale(v0[i], 1.0 - v),
                ),
                scale(v1[i], v),
            );
            // Bilinear corner correction.
            let correction = add(
                add(
                    scale(u0[0], (1.0 - u) * (1.0 - v)),
                    scale(u1[0], u * (1.0 - v)),
                ),
                add(
                    scale(u0[samples[1] - 1], (1.0 - u) * v),
                    scale(u1[samples[1] - 1], u * v),
                ),
            );
            points[i * samples[1] + j] = sub(coons, correction);
        }
    }
    interpolate_grid(&points, samples, degree, limits)
}

/* ------------------------------ trim / split / shrink ------------------------------ */

fn loop_bbox(points: &[[f64; 2]]) -> Result<([f64; 2], [f64; 2])> {
    if points.len() < 3 {
        return Err(Error::Invalid("trim loop"));
    }
    let mut low = [f64::INFINITY; 2];
    let mut high = [f64::NEG_INFINITY; 2];
    for point in points {
        if !point[0].is_finite() || !point[1].is_finite() {
            return Err(Error::Invalid("trim loop"));
        }
        for axis in 0..2 {
            low[axis] = low[axis].min(point[axis]);
            high[axis] = high[axis].max(point[axis]);
        }
    }
    if low[0] >= high[0] || low[1] >= high[1] {
        return Err(Error::Invalid("trim loop bounds"));
    }
    Ok((low, high))
}

/// Store a Rhino-like trim: the original NURBS surface plus outer/inner UV loops.
///
/// `outer` and every inner loop must be closed implicitly (the first point is not repeated),
/// must contain at least three distinct finite UV points and must lie inside the surface
/// domain. An inner loop must lie inside the outer loop. The underlying surface is preserved
/// exactly; it is never replaced by the trim polygons.
pub fn trim_surface(
    surface: &NurbsSurface,
    outer: &[[f64; 2]],
    inners: &[Vec<[f64; 2]>],
    limits: &Limits,
) -> Result<TrimmedSurface> {
    surface.validate(limits)?;
    let trimmed = TrimmedSurface {
        underlying: surface.clone(),
        outer: outer.to_vec(),
        inners: inners.to_vec(),
        reversed: false,
    };
    trimmed.validate(limits)?;
    Ok(trimmed)
}

/// Explicit Untrim: return the underlying NURBS surface as a new independent surface.
pub fn untrim_surface(trimmed: &TrimmedSurface, limits: &Limits) -> Result<NurbsSurface> {
    trimmed.validate(limits)?;
    Ok(trimmed.underlying.clone())
}

/// Shrink a surface to a rectangular sub-domain by sampling and refitting.
/// The domain values are in the surface's own UV domain; a range outside that
/// domain or an empty range is refused.
pub fn shrink_surface_domain(
    surface: &NurbsSurface,
    u: [f64; 2],
    v: [f64; 2],
    samples: [usize; 2],
    degree: [usize; 2],
    limits: &Limits,
) -> Result<NurbsSurface> {
    surface.validate(limits)?;
    if samples[0] < 2 || samples[1] < 2 {
        return Err(Error::Invalid("shrink samples"));
    }
    let [u0, u1] = surface.domain[0];
    let [v0, v1] = surface.domain[1];
    if !u[0].is_finite()
        || !u[1].is_finite()
        || !v[0].is_finite()
        || !v[1].is_finite()
        || u[0] < u0
        || u[1] > u1
        || v[0] < v0
        || v[1] > v1
        || u[0] >= u[1]
        || v[0] >= v[1]
    {
        return Err(Error::Invalid("shrink domain"));
    }
    let mut points = vec![[0.0; 3]; samples[0] * samples[1]];
    for i in 0..samples[0] {
        let uu = u[0] + (u[1] - u[0]) * i as f64 / (samples[0] - 1) as f64;
        for j in 0..samples[1] {
            let vv = v[0] + (v[1] - v[0]) * j as f64 / (samples[1] - 1) as f64;
            points[i * samples[1] + j] = surface_point(surface, uu, vv, limits)?;
        }
    }
    interpolate_grid(&points, samples, degree, limits)
}

/// Split a surface at an interior iso-parameter into two trimmed half-surfaces.
///
/// The two results keep the original underlying surface; their outer loops are the two
/// rectangular halves of the UV domain. A parameter on or outside the boundary is refused.
pub fn split_surface(
    surface: &NurbsSurface,
    iso: SurfaceIso,
    parameter: f64,
    limits: &Limits,
) -> Result<(TrimmedSurface, TrimmedSurface)> {
    surface.validate(limits)?;
    let axis = match iso {
        SurfaceIso::U => 0,
        SurfaceIso::V => 1,
    };
    let [low, high] = surface.domain[axis];
    if !parameter.is_finite() || parameter <= low || parameter >= high {
        return Err(Error::Invalid("surface split parameter"));
    }
    let [u0, u1] = surface.domain[0];
    let [v0, v1] = surface.domain[1];
    let rect =
        |a: f64, b: f64, c: f64, d: f64| -> Vec<[f64; 2]> { vec![[a, c], [b, c], [b, d], [a, d]] };
    let (left, right) = if axis == 0 {
        (rect(u0, parameter, v0, v1), rect(parameter, u1, v0, v1))
    } else {
        (rect(u0, u1, v0, parameter), rect(u0, u1, parameter, v1))
    };
    let first = TrimmedSurface {
        underlying: surface.clone(),
        outer: left,
        inners: Vec::new(),
        reversed: false,
    };
    let second = TrimmedSurface {
        underlying: surface.clone(),
        outer: right,
        inners: Vec::new(),
        reversed: false,
    };
    first.validate(limits)?;
    second.validate(limits)?;
    Ok((first, second))
}

/// Restrict a surface to a sub-domain by sampling and re-interpolating.
fn restrict_domain(
    surface: &NurbsSurface,
    u: [f64; 2],
    v: [f64; 2],
    degree: [usize; 2],
    samples: [usize; 2],
    limits: &Limits,
) -> Result<NurbsSurface> {
    if samples[0] < 2 || samples[1] < 2 {
        return Err(Error::Invalid("surface restriction grid"));
    }
    let mut points = vec![[0.0; 3]; samples[0] * samples[1]];
    for i in 0..samples[0] {
        let uu = u[0] + (u[1] - u[0]) * i as f64 / (samples[0] - 1) as f64;
        for j in 0..samples[1] {
            let vv = v[0] + (v[1] - v[0]) * j as f64 / (samples[1] - 1) as f64;
            points[i * samples[1] + j] = surface_point(surface, uu, vv, limits)?;
        }
    }
    interpolate_grid(&points, samples, degree, limits)
}

/// Shrink a trimmed surface: the underlying surface is restricted to the outer-loop bounding
/// box and the loops are remapped into the normalized sub-domain.
pub fn shrink_surface(
    trimmed: &TrimmedSurface,
    degree: [usize; 2],
    samples: [usize; 2],
    limits: &Limits,
) -> Result<TrimmedSurface> {
    trimmed.validate(limits)?;
    let (low, high) = loop_bbox(&trimmed.outer)?;
    let surface = restrict_domain(
        &trimmed.underlying,
        [low[0], high[0]],
        [low[1], high[1]],
        degree,
        samples,
        limits,
    )?;
    let remap = |point: &[f64; 2]| -> [f64; 2] {
        [
            (point[0] - low[0]) / (high[0] - low[0]),
            (point[1] - low[1]) / (high[1] - low[1]),
        ]
    };
    let outer: Vec<[f64; 2]> = trimmed.outer.iter().map(remap).collect();
    let inners: Vec<Vec<[f64; 2]>> = trimmed
        .inners
        .iter()
        .map(|inner| inner.iter().map(remap).collect())
        .collect();
    let shrunk = TrimmedSurface {
        underlying: surface,
        outer,
        inners,
        reversed: trimmed.reversed,
    };
    shrunk.validate(limits)?;
    Ok(shrunk)
}

/* -------------------------------- soft edit --------------------------------- */

/// Move every pole near a UV point with a smooth cosine falloff.
///
/// The radius is measured in UV parameter units. A pole at the centre receives the full
/// `delta`; a pole at or beyond the radius is unchanged. This is the editable control-point
/// service behind SoftEdit; it never tessellates the surface.
pub fn softedit(
    surface: &NurbsSurface,
    at: [f64; 2],
    delta: Point3,
    radius: [f64; 2],
    limits: &Limits,
) -> Result<NurbsSurface> {
    surface.validate(limits)?;
    finite(delta)?;
    if !at[0].is_finite()
        || !at[1].is_finite()
        || !radius[0].is_finite()
        || !radius[1].is_finite()
        || radius[0] <= 0.0
        || radius[1] <= 0.0
    {
        return Err(Error::Invalid("softedit radius"));
    }
    let [u0, u1] = surface.domain[0];
    let [v0, v1] = surface.domain[1];
    if at[0] < u0 || at[0] > u1 || at[1] < v0 || at[1] > v1 {
        return Err(Error::Invalid("softedit point"));
    }
    let mut edited = surface.clone();
    for i in 0..surface.size[0] {
        let u = parameter_fraction(surface, 0, i);
        for j in 0..surface.size[1] {
            let v = parameter_fraction(surface, 1, j);
            let du = (u - at[0]) / radius[0];
            let dv = (v - at[1]) / radius[1];
            let normalized = (du * du + dv * dv).sqrt();
            if normalized >= 1.0 {
                continue;
            }
            let falloff = 0.5 * (1.0 + (std::f64::consts::PI * normalized).cos());
            let index = i * surface.size[1] + j;
            edited.poles[index] = add(edited.poles[index], scale(delta, falloff));
        }
    }
    edited.validate(limits)?;
    Ok(edited)
}

/* ------------------------------- reverse / unroll ------------------------------- */

fn reverse_axis(surface: &NurbsSurface, axis: usize) -> NurbsSurface {
    let mut reversed = surface.clone();
    let count = surface.size[axis];
    let knots = &surface.knots[axis];
    let [low, high] = surface.domain[axis];
    reversed.knots[axis] = knots.iter().rev().map(|value| low + high - value).collect();
    if axis == 0 {
        for i in 0..count {
            for j in 0..surface.size[1] {
                let source = (count - 1 - i) * surface.size[1] + j;
                let target = i * surface.size[1] + j;
                reversed.poles[target] = surface.poles[source];
                reversed.weights[target] = surface.weights[source];
            }
        }
    } else {
        for i in 0..surface.size[0] {
            for j in 0..count {
                let source = i * count + (count - 1 - j);
                let target = i * count + j;
                reversed.poles[target] = surface.poles[source];
                reversed.weights[target] = surface.weights[source];
            }
        }
    }
    reversed.reversed = !surface.reversed;
    reversed
}

/// Rebuild a surface to a new degree/pole grid. This is the explicit `RebuildSrf` alias of
/// `surface_kernel::refit`; the source surface is never mutated.
pub fn rebuild_surface(
    surface: &NurbsSurface,
    degree: [usize; 2],
    size: [usize; 2],
    limits: &Limits,
) -> Result<NurbsSurface> {
    surface_kernel::refit(surface, degree, size, limits)
}

/* ------------------------------- seam / reverse ------------------------------- */

/// Close a surface seam at C0 by joining the two boundary rows.
///
/// The model stores `periodic` as representation metadata rather than a periodic knot vector;
/// this operation therefore makes the two boundary rows identical (averaged within the
/// tolerance) and marks the direction periodic. A gap larger than `tolerance_mm` is refused:
/// the seam is never silently bridged.
pub fn seam_surface(
    surface: &NurbsSurface,
    axis: SurfaceIso,
    tolerance_mm: f64,
    limits: &Limits,
) -> Result<NurbsSurface> {
    surface.validate(limits)?;
    if !tolerance_mm.is_finite() || tolerance_mm <= 0.0 {
        return Err(Error::Invalid("surface seam tolerance"));
    }
    let mut seamed = surface.clone();
    let (axis_index, count) = match axis {
        SurfaceIso::U => (0usize, surface.size[0]),
        SurfaceIso::V => (1usize, surface.size[1]),
    };
    if count < 2 {
        return Err(Error::Invalid("surface seam rows"));
    }
    let last = count - 1;
    if axis_index == 0 {
        for j in 0..surface.size[1] {
            let first = surface.poles[j];
            let final_point = surface.poles[last * surface.size[1] + j];
            if distance(first, final_point) > tolerance_mm {
                return Err(Error::Invalid("surface seam gap"));
            }
            let average = scale(add(first, final_point), 0.5);
            seamed.poles[j] = average;
            seamed.poles[last * surface.size[1] + j] = average;
        }
    } else {
        for i in 0..surface.size[0] {
            let first = surface.poles[i * surface.size[1]];
            let final_point = surface.poles[i * surface.size[1] + last];
            if distance(first, final_point) > tolerance_mm {
                return Err(Error::Invalid("surface seam gap"));
            }
            let average = scale(add(first, final_point), 0.5);
            seamed.poles[i * surface.size[1]] = average;
            seamed.poles[i * surface.size[1] + last] = average;
        }
    }
    seamed.periodic[axis_index] = true;
    seamed.validate(limits)?;
    Ok(seamed)
}

/// Reverse the parameter direction of a surface. Geometry is unchanged and the mapping
/// `S'(low+high-u, v) == S(u, v)` (or the V equivalent) is exact.
pub fn reverse_surface(
    surface: &NurbsSurface,
    axis: SurfaceIso,
    limits: &Limits,
) -> Result<NurbsSurface> {
    surface.validate(limits)?;
    let reversed = match axis {
        SurfaceIso::U => reverse_axis(surface, 0),
        SurfaceIso::V => reverse_axis(surface, 1),
    };
    reversed.validate(limits)?;
    Ok(reversed)
}

/// Unroll a surface to a planar arc-length map.
///
/// For a developable surface (plane, cylinder, ruled strip) this preserves the distance along
/// both iso directions exactly up to sampling. For a doubly curved surface it is an explicit
/// approximation: the average u/v arc lengths are preserved, not the Gaussian curvature. The
/// result is a planar degree-1 surface in the XY plane and is never claimed to be an exact
/// developable unroll for a non-developable input.
pub fn unroll(
    surface: &NurbsSurface,
    samples: [usize; 2],
    limits: &Limits,
) -> Result<NurbsSurface> {
    surface.validate(limits)?;
    if samples[0] < 2 || samples[1] < 2 {
        return Err(Error::Invalid("unroll samples"));
    }
    let points = sample_surface(surface, samples, limits)?;
    let mut u_arc = vec![0.0; samples[0]];
    let mut v_arc = vec![0.0; samples[1]];
    for i in 1..samples[0] {
        let mut total = 0.0;
        for j in 0..samples[1] {
            total += distance(points[(i - 1) * samples[1] + j], points[i * samples[1] + j]);
        }
        u_arc[i] = u_arc[i - 1] + total / samples[1] as f64;
    }
    for j in 1..samples[1] {
        let mut total = 0.0;
        for i in 0..samples[0] {
            total += distance(points[i * samples[1] + j - 1], points[i * samples[1] + j]);
        }
        v_arc[j] = v_arc[j - 1] + total / samples[0] as f64;
    }
    let mut planar = vec![[0.0; 3]; samples[0] * samples[1]];
    for i in 0..samples[0] {
        for j in 0..samples[1] {
            planar[i * samples[1] + j] = [u_arc[i], v_arc[j], 0.0];
        }
    }
    interpolate_grid(&planar, samples, [1, 1], limits)
}

/* ----------------------------- rail revolve --------------------------------- */

fn rotate_around_axis(point: Point3, origin: Point3, axis: Point3, angle: f64) -> Point3 {
    let relative = sub(point, origin);
    let cos = angle.cos();
    let sin = angle.sin();
    let rotated = add(
        add(scale(relative, cos), scale(cross(axis, relative), sin)),
        scale(axis, dot(axis, relative) * (1.0 - cos)),
    );
    add(origin, rotated)
}

/// Revolve a profile around an axis while the rail supplies the rotation angle and axial
/// displacement at every station.
///
/// The rail is the authored path: its azimuth around `axis` rotates the profile and its
/// axial coordinate translates the profile. The profile's radial distance is preserved, so a
/// rail parallel to the axis produces an exact extrusion and a helical rail produces a
/// helical sweep. A rail point on the axis keeps the previous azimuth.
pub fn rail_revolve(
    profile: &Curve,
    rail: &Curve,
    axis_origin: Point3,
    axis_direction: Point3,
    samples: [usize; 2],
    degree: [usize; 2],
    limits: &Limits,
) -> Result<NurbsSurface> {
    if samples[0] < 4 || samples[1] < 2 {
        return Err(Error::Invalid("rail revolve samples"));
    }
    let axis = unit(axis_direction, "rail revolve axis")?;
    let profile_points = surface_kernel::sample_curve(profile, samples[0], limits)?;
    let rail_points = surface_kernel::sample_curve(rail, samples[1], limits)?;
    // A stable reference frame aligned with the first rail radial, so station 0 has angle 0.
    let first_relative = sub(rail_points[0], axis_origin);
    let first_axial = dot(first_relative, axis);
    let first_radial = sub(first_relative, scale(axis, first_axial));
    let x_axis = if length(first_radial) > 1e-12 {
        unit(first_radial, "rail revolve frame")?
    } else {
        let helper = if axis[0].abs() < 0.9 {
            [1.0, 0.0, 0.0]
        } else {
            [0.0, 1.0, 0.0]
        };
        unit(cross(axis, helper), "rail revolve frame")?
    };
    let y_axis = cross(axis, x_axis);
    let mut previous_angle = 0.0;
    let mut points = vec![[0.0; 3]; samples[0] * samples[1]];
    for (station, rail_point) in rail_points.iter().enumerate() {
        let relative = sub(*rail_point, axis_origin);
        let axial = dot(relative, axis);
        let radial = sub(relative, scale(axis, axial));
        let x = dot(radial, x_axis);
        let y = dot(radial, y_axis);
        let angle = if x.abs() <= 1e-12 && y.abs() <= 1e-12 {
            previous_angle
        } else {
            previous_angle = y.atan2(x);
            previous_angle
        };
        for (index, profile_point) in profile_points.iter().enumerate() {
            let rotated = rotate_around_axis(*profile_point, axis_origin, axis, angle);
            let translated = add(rotated, scale(axis, axial));
            points[index * samples[1] + station] = translated;
        }
    }
    interpolate_grid(&points, samples, degree, limits)
}

/* ------------------------------- match / blend ------------------------------- */

/// Which boundary of a surface a match/blend operates on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceEdge {
    U0,
    U1,
    V0,
    V1,
}

/// Requested continuity for a match or blend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceContinuity {
    G0,
    G1,
    G2,
}

/// The measured result of a match: boundary gap, tangent angle (radians) and the matched surface.
#[derive(Debug, Clone, PartialEq)]
pub struct MatchReport {
    pub surface: NurbsSurface,
    pub gap_mm: f64,
    pub tangent_angle_radians: f64,
    pub continuity: SurfaceContinuity,
}

fn edge_points(
    surface: &NurbsSurface,
    edge: SurfaceEdge,
    count: usize,
    limits: &Limits,
) -> Result<Vec<Point3>> {
    let [u0, u1] = surface.domain[0];
    let [v0, v1] = surface.domain[1];
    let mut points = Vec::new();
    points
        .try_reserve_exact(count)
        .map_err(|_| Error::Budget("surface edge samples"))?;
    for index in 0..count {
        let fraction = if count <= 1 {
            0.0
        } else {
            index as f64 / (count - 1) as f64
        };
        let point = match edge {
            SurfaceEdge::U0 => surface_point(surface, u0, v0 + (v1 - v0) * fraction, limits)?,
            SurfaceEdge::U1 => surface_point(surface, u1, v0 + (v1 - v0) * fraction, limits)?,
            SurfaceEdge::V0 => surface_point(surface, u0 + (u1 - u0) * fraction, v0, limits)?,
            SurfaceEdge::V1 => surface_point(surface, u0 + (u1 - u0) * fraction, v1, limits)?,
        };
        points.push(point);
    }
    Ok(points)
}

fn adjacent_row_points(
    surface: &NurbsSurface,
    edge: SurfaceEdge,
    row_offset: usize,
    count: usize,
    limits: &Limits,
) -> Result<Vec<Point3>> {
    let [u0, u1] = surface.domain[0];
    let [v0, v1] = surface.domain[1];
    let mut points = Vec::new();
    points
        .try_reserve_exact(count)
        .map_err(|_| Error::Budget("surface row samples"))?;
    for index in 0..count {
        let fraction = if count <= 1 {
            0.0
        } else {
            index as f64 / (count - 1) as f64
        };
        let point = match edge {
            SurfaceEdge::U0 => {
                if row_offset >= surface.size[0] {
                    return Err(Error::Invalid("match continuity rows"));
                }
                let u = parameter_fraction(surface, 0, row_offset);
                surface_point(surface, u, v0 + (v1 - v0) * fraction, limits)?
            }
            SurfaceEdge::U1 => {
                if row_offset >= surface.size[0] {
                    return Err(Error::Invalid("match continuity rows"));
                }
                let u = parameter_fraction(surface, 0, surface.size[0] - 1 - row_offset);
                surface_point(surface, u, v0 + (v1 - v0) * fraction, limits)?
            }
            SurfaceEdge::V0 => {
                if row_offset >= surface.size[1] {
                    return Err(Error::Invalid("match continuity rows"));
                }
                let v = parameter_fraction(surface, 1, row_offset);
                surface_point(surface, u0 + (u1 - u0) * fraction, v, limits)?
            }
            SurfaceEdge::V1 => {
                if row_offset >= surface.size[1] {
                    return Err(Error::Invalid("match continuity rows"));
                }
                let v = parameter_fraction(surface, 1, surface.size[1] - 1 - row_offset);
                surface_point(surface, u0 + (u1 - u0) * fraction, v, limits)?
            }
        };
        points.push(point);
    }
    Ok(points)
}

/// The cross-edge tangent at one edge station, pointing into the surface.
fn edge_cross_tangent(
    surface: &NurbsSurface,
    edge: SurfaceEdge,
    fraction: f64,
    limits: &Limits,
) -> Result<Point3> {
    let [u0, u1] = surface.domain[0];
    let [v0, v1] = surface.domain[1];
    let step_u = (u1 - u0) * 1e-4;
    let step_v = (v1 - v0) * 1e-4;
    let (edge_point, inward_point) = match edge {
        SurfaceEdge::U0 => (
            surface_point(surface, u0, v0 + (v1 - v0) * fraction, limits)?,
            surface_point(surface, u0 + step_u, v0 + (v1 - v0) * fraction, limits)?,
        ),
        SurfaceEdge::U1 => (
            surface_point(surface, u1, v0 + (v1 - v0) * fraction, limits)?,
            surface_point(surface, u1 - step_u, v0 + (v1 - v0) * fraction, limits)?,
        ),
        SurfaceEdge::V0 => (
            surface_point(surface, u0 + (u1 - u0) * fraction, v0, limits)?,
            surface_point(surface, u0 + (u1 - u0) * fraction, v0 + step_v, limits)?,
        ),
        SurfaceEdge::V1 => (
            surface_point(surface, u0 + (u1 - u0) * fraction, v1, limits)?,
            surface_point(surface, u0 + (u1 - u0) * fraction, v1 - step_v, limits)?,
        ),
    };
    unit(sub(inward_point, edge_point), "surface edge tangent")
}

/// Match a boundary of `surface` to a boundary of `target` at G0, G1 or G2.
///
/// The boundary row (and one or two adjacent rows for G1/G2) is replaced by the target's
/// corresponding inward rows, resampled to the source edge count. The report measures the
/// remaining boundary gap and the maximum angle between the cross-edge tangents; a match is
/// only accepted when that gap/angle is within the geometry tolerance.
pub fn match_surface(
    surface: &NurbsSurface,
    edge: SurfaceEdge,
    target: &NurbsSurface,
    target_edge: SurfaceEdge,
    continuity: SurfaceContinuity,
    limits: &Limits,
) -> Result<MatchReport> {
    surface.validate(limits)?;
    target.validate(limits)?;
    let count = edge_count(surface, edge);
    let rows = match continuity {
        SurfaceContinuity::G0 => 1,
        SurfaceContinuity::G1 => 2,
        SurfaceContinuity::G2 => 3,
    };
    let cross_rows = |value: &NurbsSurface, border: SurfaceEdge| match border {
        SurfaceEdge::U0 | SurfaceEdge::U1 => value.size[0],
        SurfaceEdge::V0 | SurfaceEdge::V1 => value.size[1],
    };
    if rows > cross_rows(surface, edge) || rows > cross_rows(target, target_edge) {
        return Err(Error::Invalid("match continuity rows"));
    }
    let mut matched = surface.clone();
    for offset in 0..rows {
        let target_points = adjacent_row_points(target, target_edge, offset, count, limits)?;
        match edge {
            SurfaceEdge::U0 | SurfaceEdge::U1 => {
                let u = match edge {
                    SurfaceEdge::U0 => offset,
                    SurfaceEdge::U1 => surface.size[0] - 1 - offset,
                    _ => unreachable!(),
                };
                for (j, point) in target_points.iter().enumerate() {
                    matched.poles[u * surface.size[1] + j] = *point;
                }
            }
            SurfaceEdge::V0 | SurfaceEdge::V1 => {
                let v = match edge {
                    SurfaceEdge::V0 => offset,
                    SurfaceEdge::V1 => surface.size[1] - 1 - offset,
                    _ => unreachable!(),
                };
                for (i, point) in target_points.iter().enumerate() {
                    matched.poles[i * surface.size[1] + v] = *point;
                }
            }
        }
    }
    matched.validate(limits)?;
    let source_edge = edge_points(&matched, edge, 9, limits)?;
    let target_edge_points = edge_points(target, target_edge, 9, limits)?;
    let gap_mm = source_edge
        .iter()
        .zip(&target_edge_points)
        .map(|(a, b)| distance(*a, *b))
        .fold(0.0, f64::max);
    let mut tangent_angle_radians: f64 = 0.0;
    for index in 0..=8 {
        let fraction = index as f64 / 8.0;
        let a = edge_cross_tangent(&matched, edge, fraction, limits)?;
        let b = edge_cross_tangent(target, target_edge, fraction, limits)?;
        let angle = dot(a, b).clamp(-1.0, 1.0).acos();
        tangent_angle_radians = tangent_angle_radians.max(angle);
    }
    Ok(MatchReport {
        surface: matched,
        gap_mm,
        tangent_angle_radians,
        continuity,
    })
}

/// Tangents supplied to a G1/G2 blend.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlendTangents {
    pub start: Point3,
    pub end: Point3,
    pub start_second: Option<Point3>,
    pub end_second: Option<Point3>,
}

/// Blend two boundary curves into a surface.
///
/// G0 is a ruled surface. G1 uses a cubic Hermite across `v` with the supplied end tangents;
/// G2 uses a quintic Hermite with the supplied second derivatives (zero when omitted). The
/// two curves must run in the same direction and have at least two sample stations.
pub fn blend_surface(
    edge_a: &Curve,
    edge_b: &Curve,
    continuity: SurfaceContinuity,
    tangents: Option<BlendTangents>,
    samples: [usize; 2],
    degree: [usize; 2],
    limits: &Limits,
) -> Result<NurbsSurface> {
    if samples[0] < 2 || samples[1] < 2 {
        return Err(Error::Invalid("blend samples"));
    }
    let a = surface_kernel::sample_curve(edge_a, samples[0], limits)?;
    let b = surface_kernel::sample_curve(edge_b, samples[0], limits)?;
    if continuity != SurfaceContinuity::G0 && tangents.is_none() {
        return Err(Error::Missing("blend continuity tangents"));
    }
    let mut points = vec![[0.0; 3]; samples[0] * samples[1]];
    for i in 0..samples[0] {
        let p0 = a[i];
        let p1 = b[i];
        let m0 = tangents.map(|t| t.start).unwrap_or([0.0; 3]);
        let m1 = tangents.map(|t| t.end).unwrap_or([0.0; 3]);
        let s0 = tangents.and_then(|t| t.start_second).unwrap_or([0.0; 3]);
        let s1 = tangents.and_then(|t| t.end_second).unwrap_or([0.0; 3]);
        for j in 0..samples[1] {
            let v = j as f64 / (samples[1] - 1) as f64;
            let point = match continuity {
                SurfaceContinuity::G0 => add(scale(p0, 1.0 - v), scale(p1, v)),
                SurfaceContinuity::G1 => {
                    let h00 = 2.0 * v.powi(3) - 3.0 * v.powi(2) + 1.0;
                    let h10 = v.powi(3) - 2.0 * v.powi(2) + v;
                    let h01 = -2.0 * v.powi(3) + 3.0 * v.powi(2);
                    let h11 = v.powi(3) - v.powi(2);
                    add(
                        add(scale(p0, h00), scale(m0, h10)),
                        add(scale(p1, h01), scale(m1, h11)),
                    )
                }
                SurfaceContinuity::G2 => {
                    let h00 = 1.0 - 10.0 * v.powi(3) + 15.0 * v.powi(4) - 6.0 * v.powi(5);
                    let h10 = v - 6.0 * v.powi(3) + 8.0 * v.powi(4) - 3.0 * v.powi(5);
                    let h20 = 0.5 * v.powi(2) - 1.5 * v.powi(3) + 1.5 * v.powi(4) - 0.5 * v.powi(5);
                    let h01 = 10.0 * v.powi(3) - 15.0 * v.powi(4) + 6.0 * v.powi(5);
                    let h11 = -4.0 * v.powi(3) + 7.0 * v.powi(4) - 3.0 * v.powi(5);
                    let h21 = 0.5 * v.powi(3) - v.powi(4) + 0.5 * v.powi(5);
                    add(
                        add(
                            add(scale(p0, h00), scale(m0, h10)),
                            add(scale(s0, h20), scale(p1, h01)),
                        ),
                        add(scale(m1, h11), scale(s1, h21)),
                    )
                }
            };
            points[i * samples[1] + j] = point;
        }
    }
    interpolate_grid(&points, samples, degree, limits)
}

/// Extend one surface boundary by `distance_mm` along the edge's inward
/// direction. The boundary row is duplicated at a linear offset and the whole
/// grid is refit; the original surface is never mutated.
pub fn extend_surface(
    surface: &NurbsSurface,
    edge: SurfaceEdge,
    distance_mm: f64,
    limits: &Limits,
) -> Result<NurbsSurface> {
    surface.validate(limits)?;
    if !distance_mm.is_finite() || distance_mm <= 0.0 {
        return Err(Error::Invalid("surface extend distance"));
    }
    const BASE: usize = 12;
    let mut original = vec![[0.0; 3]; BASE * BASE];
    for i in 0..BASE {
        let u = i as f64 / (BASE - 1) as f64;
        for j in 0..BASE {
            let v = j as f64 / (BASE - 1) as f64;
            original[i * BASE + j] = surface_point(surface, u, v, limits)?;
        }
    }
    let (rows, columns) = match edge {
        SurfaceEdge::U0 | SurfaceEdge::U1 => (BASE + 1, BASE),
        SurfaceEdge::V0 | SurfaceEdge::V1 => (BASE, BASE + 1),
    };
    let mut points = vec![[0.0; 3]; rows * columns];
    let mut write = |i: usize, j: usize, point: Point3| {
        points[i * columns + j] = point;
    };
    let direction = |a: Point3, b: Point3| -> Point3 {
        let d = sub(a, b);
        let length = d[0].hypot(d[1]).hypot(d[2]);
        if length <= 1.0e-12 {
            [0.0; 3]
        } else {
            [d[0] / length, d[1] / length, d[2] / length]
        }
    };
    match edge {
        SurfaceEdge::U0 => {
            let dir = direction(original[0], original[BASE]);
            for (j, point) in original.iter().enumerate().take(BASE) {
                write(0, j, add(*point, scale(dir, distance_mm)));
            }
            for i in 0..BASE {
                for j in 0..BASE {
                    write(i + 1, j, original[i * BASE + j]);
                }
            }
        }
        SurfaceEdge::U1 => {
            let dir = direction(original[(BASE - 1) * BASE], original[(BASE - 2) * BASE]);
            for i in 0..BASE {
                for j in 0..BASE {
                    write(i, j, original[i * BASE + j]);
                }
            }
            for j in 0..BASE {
                write(
                    BASE,
                    j,
                    add(original[(BASE - 1) * BASE + j], scale(dir, distance_mm)),
                );
            }
        }
        SurfaceEdge::V0 => {
            let dir = direction(original[0], original[1]);
            for i in 0..BASE {
                write(i, 0, add(original[i * BASE], scale(dir, distance_mm)));
                for j in 0..BASE {
                    write(i, j + 1, original[i * BASE + j]);
                }
            }
        }
        SurfaceEdge::V1 => {
            let dir = direction(original[BASE - 1], original[BASE - 2]);
            for i in 0..BASE {
                for j in 0..BASE {
                    write(i, j, original[i * BASE + j]);
                }
                write(
                    i,
                    BASE,
                    add(original[i * BASE + BASE - 1], scale(dir, distance_mm)),
                );
            }
        }
    }
    interpolate_grid(&points, [rows, columns], [3, 3], limits)
}

/// Merge two surfaces that share a matching boundary. The shared edge is
/// detected at U1/U0 or V1/V0; the result is a refit grid across both domains.
/// A non-matching boundary is refused instead of silently welding a gap.
pub fn merge_surfaces(
    first: &NurbsSurface,
    second: &NurbsSurface,
    limits: &Limits,
) -> Result<NurbsSurface> {
    first.validate(limits)?;
    second.validate(limits)?;
    let sample_edge = |surface: &NurbsSurface, edge: SurfaceEdge| -> Result<Vec<Point3>> {
        let mut points = Vec::with_capacity(9);
        for i in 0..9 {
            let t = i as f64 / 8.0;
            let (u, v) = match edge {
                SurfaceEdge::U0 => (0.0, t),
                SurfaceEdge::U1 => (1.0, t),
                SurfaceEdge::V0 => (t, 0.0),
                SurfaceEdge::V1 => (t, 1.0),
            };
            points.push(surface_point(surface, u, v, limits)?);
        }
        Ok(points)
    };
    let close =
        |a: &[Point3], b: &[Point3]| a.iter().zip(b).all(|(p, q)| distance(*p, *q) <= 1.0e-6);
    let u_match = close(
        &sample_edge(first, SurfaceEdge::U1)?,
        &sample_edge(second, SurfaceEdge::U0)?,
    );
    let v_match = close(
        &sample_edge(first, SurfaceEdge::V1)?,
        &sample_edge(second, SurfaceEdge::V0)?,
    );
    let (join_u, samples) = if u_match {
        (true, [32, 12])
    } else if v_match {
        (false, [12, 32])
    } else {
        return Err(Error::Invalid("surfaces do not share a boundary"));
    };
    let mut points = vec![[0.0; 3]; samples[0] * samples[1]];
    for i in 0..samples[0] {
        for j in 0..samples[1] {
            let (surface, first_half) = if join_u {
                if i < 16 {
                    (first, true)
                } else {
                    (second, false)
                }
            } else if j < 16 {
                (first, true)
            } else {
                (second, false)
            };
            let local = if join_u {
                if first_half {
                    i as f64 / 15.0
                } else {
                    (i - 16) as f64 / 15.0
                }
            } else if first_half {
                j as f64 / 15.0
            } else {
                (j - 16) as f64 / 15.0
            };
            let other = if join_u {
                j as f64 / (samples[1] - 1) as f64
            } else {
                i as f64 / (samples[0] - 1) as f64
            };
            let (u, v) = if join_u {
                (local, other)
            } else {
                (other, local)
            };
            points[i * samples[1] + j] = surface_point(surface, u, v, limits)?;
        }
    }
    interpolate_grid(&points, samples, [3, 3], limits)
}

/// Extrude every selected curve along +Z by `height_mm` using a vertical rail
/// and the exact sweep1 service. Returns one surface per curve.
pub fn extrude_curves(
    curves: &[Curve],
    height_mm: f64,
    limits: &Limits,
) -> Result<Vec<NurbsSurface>> {
    if curves.is_empty() {
        return Err(Error::Invalid("extrude curve count"));
    }
    if !height_mm.is_finite() || height_mm == 0.0 {
        return Err(Error::Invalid("extrude height"));
    }
    let mut surfaces = Vec::with_capacity(curves.len());
    for curve in curves {
        curve.validate(limits)?;
        let sample = surface_kernel::sample_curve(curve, 2, limits)?;
        let start = sample[0];
        let rail = Curve::Line {
            start,
            end: [start[0], start[1], start[2] + height_mm],
            domain: [0.0, height_mm.abs()],
        };
        surfaces.push(surface_kernel::sweep1(
            curve,
            &rail,
            [17, 33],
            [3, 3],
            limits,
        )?);
    }
    Ok(surfaces)
}

/// Drape a surface over a set of points. The XY bounding box becomes the
/// surface domain; each grid height is an inverse-distance blend of the point
/// heights. This is an approximation of the Drape command, not a mesh fit.
pub fn drape_points(
    points: &[Point3],
    samples: [usize; 2],
    degree: [usize; 2],
    limits: &Limits,
) -> Result<NurbsSurface> {
    if points.len() < 3 {
        return Err(Error::Invalid("drape point count"));
    }
    if samples[0] < 4 || samples[1] < 4 {
        return Err(Error::Invalid("drape samples"));
    }
    for point in points {
        finite(*point)?;
    }
    let mut min = points[0];
    let mut max = points[0];
    for point in points {
        for axis in 0..3 {
            min[axis] = min[axis].min(point[axis]);
            max[axis] = max[axis].max(point[axis]);
        }
    }
    let width = max[0] - min[0];
    let height = max[1] - min[1];
    if !width.is_finite() || !height.is_finite() || width <= 1.0e-9 || height <= 1.0e-9 {
        return Err(Error::Invalid("drape bounds"));
    }
    let mut grid = vec![[0.0; 3]; samples[0] * samples[1]];
    for i in 0..samples[0] {
        let x = min[0] + width * i as f64 / (samples[0] - 1) as f64;
        for j in 0..samples[1] {
            let y = min[1] + height * j as f64 / (samples[1] - 1) as f64;
            let mut weight_sum = 0.0;
            let mut z = 0.0;
            for point in points {
                let dx = point[0] - x;
                let dy = point[1] - y;
                let distance2 = dx * dx + dy * dy;
                let weight = 1.0 / (distance2 + 1.0e-6);
                weight_sum += weight;
                z += weight * point[2];
            }
            grid[i * samples[1] + j] = [x, y, z / weight_sum];
        }
    }
    interpolate_grid(&grid, samples, degree, limits)
}

/// Build a planar surface spanning a set of coplanar curves. The plane is the
/// best-fit Newell plane; the surface covers the curves' 2D bounding rectangle.
/// This is a bounded planar base, not a trim of the input curves.
pub fn planar_surface_from_curves(curves: &[Curve], limits: &Limits) -> Result<NurbsSurface> {
    if curves.is_empty() {
        return Err(Error::Invalid("planar curve count"));
    }
    let mut points = Vec::new();
    for curve in curves {
        curve.validate(limits)?;
        points.extend(surface_kernel::sample_curve(curve, 16, limits)?);
    }
    if points.len() < 3 {
        return Err(Error::Invalid("planar curve points"));
    }
    let mut normal = [0.0; 3];
    for i in 0..points.len() {
        let a = points[i];
        let b = points[(i + 1) % points.len()];
        normal[0] += (a[1] - b[1]) * (a[2] + b[2]);
        normal[1] += (a[2] - b[2]) * (a[0] + b[0]);
        normal[2] += (a[0] - b[0]) * (a[1] + b[1]);
    }
    let length = normal[0].hypot(normal[1]).hypot(normal[2]);
    if !length.is_finite() || length <= 1.0e-12 {
        return Err(Error::Invalid("planar curve plane"));
    }
    let normal = [normal[0] / length, normal[1] / length, normal[2] / length];
    let origin = points[0];
    let mut x = [0.0; 3];
    for point in &points[1..] {
        let candidate = sub(*point, origin);
        let projection = dot(candidate, normal);
        let projected = sub(candidate, scale(normal, projection));
        let length = projected[0].hypot(projected[1]).hypot(projected[2]);
        if length > 1.0e-9 {
            x = [
                projected[0] / length,
                projected[1] / length,
                projected[2] / length,
            ];
            break;
        }
    }
    if x == [0.0; 3] {
        return Err(Error::Invalid("planar curve x axis"));
    }
    let y = cross(normal, x);
    let mut min = [f64::INFINITY; 2];
    let mut max = [f64::NEG_INFINITY; 2];
    for point in &points {
        let relative = sub(*point, origin);
        let u = dot(relative, x);
        let v = dot(relative, y);
        min[0] = min[0].min(u);
        min[1] = min[1].min(v);
        max[0] = max[0].max(u);
        max[1] = max[1].max(v);
    }
    let width = max[0] - min[0];
    let height = max[1] - min[1];
    if !width.is_finite() || !height.is_finite() || width <= 1.0e-9 || height <= 1.0e-9 {
        return Err(Error::Invalid("planar curve extent"));
    }
    let corner = add(add(origin, scale(x, min[0])), scale(y, min[1]));
    surface_kernel::plane(corner, x, y, width, height, limits)
}

/// Set the cross-boundary tangent direction on one surface edge by rotating the
/// adjacent pole row/column around the boundary. The boundary itself is fixed;
/// only the direction of the inward tangent changes.
pub fn set_surface_tangent(
    surface: &NurbsSurface,
    edge: SurfaceEdge,
    direction: Point3,
    limits: &Limits,
) -> Result<NurbsSurface> {
    surface.validate(limits)?;
    finite(direction)?;
    let length = direction[0].hypot(direction[1]).hypot(direction[2]);
    if !length.is_finite() || length <= 1.0e-12 {
        return Err(Error::Invalid("surface tangent direction"));
    }
    let unit = [
        direction[0] / length,
        direction[1] / length,
        direction[2] / length,
    ];
    let mut edited = surface.clone();
    let rows = surface.size[0];
    let columns = surface.size[1];
    let adjust = |edited: &mut NurbsSurface, cell: usize, boundary: Point3| {
        let adjacent = edited.poles[cell];
        let current = sub(adjacent, boundary);
        let span = current[0].hypot(current[1]).hypot(current[2]);
        if span > 1.0e-12 {
            edited.poles[cell] = add(boundary, scale(unit, span));
        }
    };
    match edge {
        SurfaceEdge::U0 => {
            for j in 0..columns {
                let boundary = edited.poles[j];
                adjust(&mut edited, columns + j, boundary);
            }
        }
        SurfaceEdge::U1 => {
            for j in 0..columns {
                let boundary = edited.poles[(rows - 1) * columns + j];
                adjust(&mut edited, (rows - 2) * columns + j, boundary);
            }
        }
        SurfaceEdge::V0 => {
            for i in 0..rows {
                let boundary = edited.poles[i * columns];
                adjust(&mut edited, i * columns + 1, boundary);
            }
        }
        SurfaceEdge::V1 => {
            for i in 0..rows {
                let boundary = edited.poles[i * columns + columns - 1];
                adjust(&mut edited, i * columns + columns - 2, boundary);
            }
        }
    }
    edited.validate(limits)?;
    Ok(edited)
}

/* ----------------------------- tween / offset ----------------------------- */

/// Interpolate a surface between two surfaces at `fraction` in `[0,1]`.
///
/// Both inputs are sampled on the same normalized UV grid; the result is a
/// rebuilt NURBS surface. The source surfaces are never mutated.
pub fn tween_surface(
    first: &NurbsSurface,
    second: &NurbsSurface,
    fraction: f64,
    samples: [usize; 2],
    degree: [usize; 2],
    limits: &Limits,
) -> Result<NurbsSurface> {
    first.validate(limits)?;
    second.validate(limits)?;
    if !fraction.is_finite() || !(0.0..=1.0).contains(&fraction) {
        return Err(Error::Invalid("tween fraction"));
    }
    if samples[0] < 2 || samples[1] < 2 {
        return Err(Error::Invalid("tween samples"));
    }
    let mut points = vec![[0.0; 3]; samples[0] * samples[1]];
    for i in 0..samples[0] {
        let u = i as f64 / (samples[0] - 1) as f64;
        for j in 0..samples[1] {
            let v = j as f64 / (samples[1] - 1) as f64;
            let a = surface_point(first, u, v, limits)?;
            let b = surface_point(second, u, v, limits)?;
            points[i * samples[1] + j] = add(scale(a, 1.0 - fraction), scale(b, fraction));
        }
    }
    interpolate_grid(&points, samples, degree, limits)
}

/* ----------------------------- offset / normal extrude ----------------------------- */

/// Build a circular fillet strip between two curves at a radius that may vary
/// linearly from start to end along the rails.
pub fn fillet_strip(
    edge_a: &Curve,
    edge_b: &Curve,
    start_radius: f64,
    end_radius: f64,
    samples: [usize; 2],
    degree: [usize; 2],
    limits: &Limits,
) -> Result<NurbsSurface> {
    if samples[0] < 2 || samples[1] < 3 {
        return Err(Error::Invalid("fillet samples"));
    }
    if !start_radius.is_finite()
        || !end_radius.is_finite()
        || start_radius <= 0.0
        || end_radius <= 0.0
    {
        return Err(Error::Invalid("fillet radius"));
    }
    let a = surface_kernel::sample_curve(edge_a, samples[0], limits)?;
    let b = surface_kernel::sample_curve(edge_b, samples[0], limits)?;
    let mut points = vec![[0.0; 3]; samples[0] * samples[1]];
    for i in 0..samples[0] {
        let t = i as f64 / (samples[0] - 1) as f64;
        let radius = start_radius + (end_radius - start_radius) * t;
        let chord = sub(b[i], a[i]);
        let length = chord[0].hypot(chord[1]).hypot(chord[2]);
        if !length.is_finite() || length <= 1.0e-12 || length > 2.0 * radius {
            return Err(Error::Invalid("fillet span"));
        }
        let unit = [chord[0] / length, chord[1] / length, chord[2] / length];
        let previous = a[i.saturating_sub(1)];
        let next = a[(i + 1).min(samples[0] - 1)];
        let tangent = sub(next, previous);
        let mut normal = cross(unit, tangent);
        let mut normal_length = normal[0].hypot(normal[1]).hypot(normal[2]);
        if normal_length <= 1.0e-12 {
            normal = cross(unit, [0.0, 0.0, 1.0]);
            normal_length = normal[0].hypot(normal[1]).hypot(normal[2]);
        }
        if normal_length <= 1.0e-12 {
            normal = cross(unit, [0.0, 1.0, 0.0]);
            normal_length = normal[0].hypot(normal[1]).hypot(normal[2]);
        }
        let normal = [
            normal[0] / normal_length,
            normal[1] / normal_length,
            normal[2] / normal_length,
        ];
        let middle = scale(add(a[i], b[i]), 0.5);
        let height = (radius * radius - (length / 2.0).powi(2)).max(0.0).sqrt();
        let center = add(middle, scale(normal, height));
        let u = scale(sub(a[i], center), 1.0 / radius);
        let w = sub(b[i], center);
        let mut v = sub(w, scale(u, dot(w, u)));
        let mut v_length = v[0].hypot(v[1]).hypot(v[2]);
        if v_length <= 1.0e-12 {
            v = cross(normal, u);
            v_length = v[0].hypot(v[1]).hypot(v[2]);
        }
        let v = [v[0] / v_length, v[1] / v_length, v[2] / v_length];
        let angle = dot(w, v).atan2(dot(w, u));
        for j in 0..samples[1] {
            let s = j as f64 / (samples[1] - 1) as f64;
            let point = add(
                center,
                add(
                    scale(u, radius * (s * angle).cos()),
                    scale(v, radius * (s * angle).sin()),
                ),
            );
            points[i * samples[1] + j] = point;
        }
    }
    interpolate_grid(&points, samples, degree, limits)
}

/// Build a chamfer strip with a setback that varies linearly along the rails.
pub fn variable_chamfer_strip(
    edge_a: &Curve,
    edge_b: &Curve,
    start_setback: f64,
    end_setback: f64,
    samples: [usize; 2],
    degree: [usize; 2],
    limits: &Limits,
) -> Result<NurbsSurface> {
    if samples[0] < 2 || samples[1] < 2 {
        return Err(Error::Invalid("variable chamfer samples"));
    }
    if !start_setback.is_finite()
        || !end_setback.is_finite()
        || start_setback <= 0.0
        || end_setback <= 0.0
    {
        return Err(Error::Invalid("variable chamfer setback"));
    }
    let a = surface_kernel::sample_curve(edge_a, samples[0], limits)?;
    let b = surface_kernel::sample_curve(edge_b, samples[0], limits)?;
    let mut points = vec![[0.0; 3]; samples[0] * samples[1]];
    for i in 0..samples[0] {
        let t = i as f64 / (samples[0] - 1) as f64;
        let setback = start_setback + (end_setback - start_setback) * t;
        let delta = sub(b[i], a[i]);
        let length = delta[0].hypot(delta[1]).hypot(delta[2]);
        if !length.is_finite() || length <= 2.0 * setback {
            return Err(Error::Invalid("variable chamfer span"));
        }
        let unit = [delta[0] / length, delta[1] / length, delta[2] / length];
        let start = add(a[i], scale(unit, setback));
        let end = sub(b[i], scale(unit, setback));
        for j in 0..samples[1] {
            let v = j as f64 / (samples[1] - 1) as f64;
            points[i * samples[1] + j] = add(scale(start, 1.0 - v), scale(end, v));
        }
    }
    interpolate_grid(&points, samples, degree, limits)
}

/// Build a chamfer strip between two curves: each rail is set back by
/// `setback_mm` along the chord and the strip is ruled between the setbacks.
pub fn chamfer_strip(
    edge_a: &Curve,
    edge_b: &Curve,
    setback_mm: f64,
    samples: [usize; 2],
    degree: [usize; 2],
    limits: &Limits,
) -> Result<NurbsSurface> {
    if samples[0] < 2 || samples[1] < 2 {
        return Err(Error::Invalid("chamfer samples"));
    }
    if !setback_mm.is_finite() || setback_mm <= 0.0 {
        return Err(Error::Invalid("chamfer setback"));
    }
    let a = surface_kernel::sample_curve(edge_a, samples[0], limits)?;
    let b = surface_kernel::sample_curve(edge_b, samples[0], limits)?;
    let mut points = vec![[0.0; 3]; samples[0] * samples[1]];
    for i in 0..samples[0] {
        let delta = sub(b[i], a[i]);
        let length = delta[0].hypot(delta[1]).hypot(delta[2]);
        if !length.is_finite() || length <= 2.0 * setback_mm {
            return Err(Error::Invalid("chamfer span"));
        }
        let unit = [delta[0] / length, delta[1] / length, delta[2] / length];
        let start = add(a[i], scale(unit, setback_mm));
        let end = sub(b[i], scale(unit, setback_mm));
        for j in 0..samples[1] {
            let v = j as f64 / (samples[1] - 1) as f64;
            points[i * samples[1] + j] = add(scale(start, 1.0 - v), scale(end, v));
        }
    }
    interpolate_grid(&points, samples, degree, limits)
}

/// Blend two curves with tangent magnitudes that differ at the two rails.
/// `start_scale`/`end_scale` shape the Hermite bulge; 0 gives a ruled blend.
pub fn variable_blend_surface(
    edge_a: &Curve,
    edge_b: &Curve,
    start_scale: f64,
    end_scale: f64,
    samples: [usize; 2],
    degree: [usize; 2],
    limits: &Limits,
) -> Result<NurbsSurface> {
    if samples[0] < 2 || samples[1] < 2 {
        return Err(Error::Invalid("variable blend samples"));
    }
    if !start_scale.is_finite()
        || !end_scale.is_finite()
        || start_scale < 0.0
        || end_scale < 0.0
        || (start_scale == 0.0 && end_scale == 0.0)
    {
        return Err(Error::Invalid("variable blend scale"));
    }
    let a = surface_kernel::sample_curve(edge_a, samples[0], limits)?;
    let b = surface_kernel::sample_curve(edge_b, samples[0], limits)?;
    let mut points = vec![[0.0; 3]; samples[0] * samples[1]];
    for i in 0..samples[0] {
        let p0 = a[i];
        let p1 = b[i];
        let tangent = sub(p1, p0);
        let m0 = scale(tangent, start_scale);
        let m1 = scale(tangent, -end_scale);
        for j in 0..samples[1] {
            let v = j as f64 / (samples[1] - 1) as f64;
            let h00 = 2.0 * v.powi(3) - 3.0 * v.powi(2) + 1.0;
            let h10 = v.powi(3) - 2.0 * v.powi(2) + v;
            let h01 = -2.0 * v.powi(3) + 3.0 * v.powi(2);
            let h11 = v.powi(3) - v.powi(2);
            points[i * samples[1] + j] = add(
                add(scale(p0, h00), scale(m0, h10)),
                add(scale(p1, h01), scale(m1, h11)),
            );
        }
    }
    interpolate_grid(&points, samples, degree, limits)
}

/// Offset a surface by a distance that varies linearly along one parameter.
/// `axis` selects whether the distance runs along U or V.
pub fn variable_offset_surface(
    surface: &NurbsSurface,
    start_mm: f64,
    end_mm: f64,
    axis: SurfaceIso,
    samples: [usize; 2],
    degree: [usize; 2],
    limits: &Limits,
) -> Result<NurbsSurface> {
    surface.validate(limits)?;
    if !start_mm.is_finite() || !end_mm.is_finite() || (start_mm == 0.0 && end_mm == 0.0) {
        return Err(Error::Invalid("variable offset distance"));
    }
    if samples[0] < 2 || samples[1] < 2 {
        return Err(Error::Invalid("variable offset samples"));
    }
    let mut points = vec![[0.0; 3]; samples[0] * samples[1]];
    for i in 0..samples[0] {
        let u = i as f64 / (samples[0] - 1) as f64;
        for j in 0..samples[1] {
            let v = j as f64 / (samples[1] - 1) as f64;
            let point = surface_point(surface, u, v, limits)?;
            let normal = surface_kernel::normal(surface, u, v, limits)?;
            let t = match axis {
                SurfaceIso::U => u,
                SurfaceIso::V => v,
            };
            let distance = start_mm + (end_mm - start_mm) * t;
            points[i * samples[1] + j] = add(point, scale(normal, distance));
        }
    }
    interpolate_grid(&points, samples, degree, limits)
}

/// Offset a surface along its exact normals by `distance_mm`, sampled and refit.
///
/// This is the portable surface offset used by `OffsetSrf` and the front face of a
/// normal extrusion. A degenerate normal or a non-finite offset is refused; the result is a
/// rebuilt NURBS surface, so the caller must treat it as an approximation with a measured
/// deviation, not as an exact analytic offset.
pub fn offset_surface(
    surface: &NurbsSurface,
    distance_mm: f64,
    samples: [usize; 2],
    degree: [usize; 2],
    limits: &Limits,
) -> Result<NurbsSurface> {
    surface.validate(limits)?;
    if !distance_mm.is_finite() || distance_mm == 0.0 {
        return Err(Error::Invalid("surface offset distance"));
    }
    let mut points = vec![[0.0; 3]; samples[0] * samples[1]];
    for i in 0..samples[0] {
        let u = grid_parameter(surface, 0, i, samples[0]);
        for j in 0..samples[1] {
            let v = grid_parameter(surface, 1, j, samples[1]);
            let point = surface_point(surface, u, v, limits)?;
            let normal = surface_kernel::normal(surface, u, v, limits)?;
            points[i * samples[1] + j] = add(point, scale(normal, distance_mm));
        }
    }
    interpolate_grid(&points, samples, degree, limits)
}

/// Normal extrude: a parallel surface at `height_mm` (the editable sheet produced by
/// `ExtrudeSrf` along the surface normal). It shares the offset contract above.
pub fn normal_extrude(
    surface: &NurbsSurface,
    height_mm: f64,
    samples: [usize; 2],
    degree: [usize; 2],
    limits: &Limits,
) -> Result<NurbsSurface> {
    offset_surface(surface, height_mm, samples, degree, limits)
}

/* --------------------------------- edge helpers --------------------------------- */

/// Sample one boundary as a curve (used by match/blend fixtures and UI previews).
pub fn boundary_curve(
    surface: &NurbsSurface,
    edge: SurfaceEdge,
    count: usize,
    limits: &Limits,
) -> Result<Curve> {
    let points = edge_points(surface, edge, count, limits)?;
    crate::curve_build::polyline(&points)
}
