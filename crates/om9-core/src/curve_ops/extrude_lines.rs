// SPDX-License-Identifier: LGPL-2.1-or-later
//! Control wires and exact translated profiles for the Extrude Lines command. Coordinates, offsets and
//! tolerance are all in document space. Display tessellation never supplies a corner.
use super::linear_path::placed_vertices;
use crate::{
    Error, Result,
    budget::{Limits, Usage},
    evaluation::{CurveEvaluator, Parameter},
    geometry::{Curve, Point3, Transform},
};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DraftCorners {
    #[default]
    Sharp,
    Round,
    Smooth,
}

/// Bézier handle length of a smooth outside draft corner, measured from the two rail endpoints.
pub fn smooth_draft_handle(chord: f64, tangent_cosine: f64) -> f64 {
    chord * (3. - tangent_cosine.clamp(-1., 1.)) / 6.
}

type Segment = [Point3; 2];
type Key = [[u64; 3]; 2];

fn key(a: Point3, b: Point3) -> Key {
    let bits = |p: Point3| p.map(|x| if x == 0. { 0 } else { x.to_bits() });
    let mut pair = [bits(a), bits(b)];
    pair.sort_unstable();
    pair
}

fn wire_usage(edges: usize) -> Result<Usage> {
    let mut usage = Usage::btree::<Key, Segment>(edges)?;
    usage.points = (edges as u64)
        .checked_mul(2)
        .ok_or(Error::Overflow("Extrude Lines endpoints"))?;
    Ok(usage)
}

pub fn is_piecewise_linear(curve: &Curve) -> bool {
    match curve {
        Curve::Line { .. } => true,
        Curve::Polycurve { segments, .. } => segments.iter().all(is_piecewise_linear),
        Curve::Nurbs(n) => n.degree == 1,
        Curve::Circle { .. } => false,
    }
}

/// A closed polygon drafted obliquely has independent side rails in Rhino 5.
pub fn is_inclined_draft(normal: Point3, unit_axis: Point3) -> bool {
    let cosine = (0..3).map(|i| normal[i] * unit_axis[i]).sum::<f64>().abs();
    cosine > 1e-9 && cosine < 1. - 1e-12
}

/// A common plane of the selected source curves determines the default direction.
/// The complete NURBS hull is conservative for a trimmed curve. Inspect the hull
/// instead of deciding planarity from the preview polyline.
pub fn default_axis<'a>(
    profiles: impl IntoIterator<Item = (&'a Curve, Transform)>,
    preferred: Point3,
    limits: &Limits,
) -> Result<Point3> {
    Ok(profile_plane(profiles, limits)?.unwrap_or(preferred))
}

/// Return the common plane normal when the complete placed control hull defines one.
pub fn profile_plane<'a>(
    profiles: impl IntoIterator<Item = (&'a Curve, Transform)>,
    limits: &Limits,
) -> Result<Option<Point3>> {
    fn append(
        curve: &Curve,
        world: Transform,
        hull: &mut Vec<Point3>,
        limits: &Limits,
    ) -> Result<()> {
        let mut point = |p: Point3| -> Result<()> {
            let mut usage = Usage::array::<Point3>(hull.len() + 1)?;
            usage.points = (hull.len() + 1) as u64;
            usage.check(limits)?;
            hull.push(world.point(p)?);
            Ok(())
        };
        match curve {
            Curve::Line { start, end, .. } => {
                point(*start)?;
                point(*end)?;
            }
            Curve::Circle {
                center,
                x,
                y,
                radius,
                ..
            } => {
                point(*center)?;
                for axis in [x, y] {
                    point(std::array::from_fn(|i| center[i] + axis[i] * radius))?;
                }
            }
            Curve::Nurbs(n) => {
                for p in &n.poles {
                    point(*p)?;
                }
            }
            Curve::Polycurve { segments, .. } => {
                for s in segments {
                    append(s, world, hull, limits)?;
                }
            }
        }
        Ok(())
    }
    let mut hull = Vec::new();
    for (curve, world) in profiles {
        curve.validate(limits)?;
        append(curve, world, &mut hull, limits)?;
    }
    if hull.len() < 3 {
        return Ok(None);
    }
    let Ok(normal) = crate::slab::planar_hull_normal(&hull, None) else {
        return Ok(None);
    };
    // Rhino 5 orients a planar profile toward its positive dominant world axis.
    // Equal magnitudes prefer Z, then X before Y.
    let mut dominant = 2;
    for i in [0, 1] {
        if normal[i].abs() > normal[dominant].abs() {
            dominant = i;
        }
    }
    Ok(Some(if normal[dominant] < 0. {
        normal.map(|v| -v)
    } else {
        normal
    }))
}

/// Enumerate the original primitive segments in path order, including nested
/// and reversed polycurves. A rational segment stays exact.
pub fn profile_segments(curve: &Curve, limits: &Limits) -> Result<Vec<Curve>> {
    fn visit(
        curve: &Curve,
        reverse: bool,
        parts: &mut Vec<Curve>,
        held: &mut Usage,
        limits: &Limits,
    ) -> Result<()> {
        if let Curve::Polycurve { segments, reversed } = curve {
            let backwards = reverse ^ reversed;
            for i in 0..segments.len() {
                let index = if backwards { segments.len() - 1 - i } else { i };
                visit(&segments[index], backwards, parts, held, limits)?;
            }
        } else {
            held.add(curve.validate(limits)?, limits)?;
            held.add(Usage::array::<Curve>(1)?, limits)?;
            parts.push(if reverse {
                super::reverse_curve(curve, limits)?
            } else {
                curve.clone()
            });
        }
        Ok(())
    }
    curve.validate(limits)?;
    let mut parts = Vec::new();
    visit(curve, false, &mut parts, &mut Usage::default(), limits)?;
    Ok(parts)
}

/// Preserve corners as separate prism faces. Smooth, joined segments become
/// a single rational edge when SplitAtTangents is off.
pub fn prism_profile<'a>(
    curve: &'a Curve,
    world: Transform,
    split: bool,
    tolerance: f64,
    angular_tolerance: f64,
    limits: &Limits,
) -> Result<std::borrow::Cow<'a, Curve>> {
    use std::borrow::Cow;
    if split || !matches!(curve, Curve::Polycurve { .. }) {
        return Ok(Cow::Borrowed(curve));
    }
    let mut parts = profile_segments(curve, limits)?;
    if parts.len() < 2 {
        return Ok(Cow::Borrowed(curve));
    }
    // Begin a closed profile at a corner, so a smooth run can cross its
    // original seam. A wholly smooth profile retains the source seam.
    for i in 0..parts.len() {
        let prev = (i + parts.len() - 1) % parts.len();
        if !super::tangent_joint(
            &parts[prev],
            &parts[i],
            world,
            tolerance,
            angular_tolerance,
            limits,
        )? {
            parts.rotate_left(i);
            break;
        }
    }
    let mut sections = Vec::new();
    let mut first = 0;
    for end in 1..=parts.len() {
        if end < parts.len()
            && super::tangent_joint(
                &parts[end - 1],
                &parts[end],
                world,
                tolerance,
                angular_tolerance,
                limits,
            )?
        {
            continue;
        }
        let run = &mut parts[first..end];
        if run.len() == 1 {
            sections.push(run[0].clone());
        } else {
            // The shared rational join takes clamped pieces. Materialize trimmed knot
            // intervals exactly before grouping them, preserving the traversal order.
            for part in run.iter_mut() {
                if let Curve::Nurbs(n) = part
                    && n.seam.is_none()
                    && (n.periodic
                        || n.knots[..=n.degree].iter().any(|k| *k != n.domain[0])
                        || n.knots[n.knots.len() - n.degree - 1..]
                            .iter()
                            .any(|k| *k != n.domain[1]))
                {
                    let first = super::clamped_active_nurbs(n, true, limits)?;
                    *n = super::clamped_active_nurbs(&first, false, limits)?;
                }
            }
            let mut scale = 1.0_f64;
            for part in run.iter() {
                for v in super::curve_endpoints(part, limits)?.into_iter().flatten() {
                    scale = scale.max(v.abs());
                }
            }
            let fused = super::fuse_clamped_segments(run, 64.0 * f64::EPSILON * scale, limits)?
                .ok_or(Error::Unsupported(
                    "Cannot join these tangent extrusion segments exactly",
                ))?;
            sections.push(fused);
        }
        first = end;
    }
    if sections.len() == 1 {
        return Ok(Cow::Owned(sections.pop().unwrap()));
    }
    Ok(Cow::Owned(Curve::Polycurve {
        segments: sections,
        reversed: false,
    }))
}

/// Connector stations come from actual segment ends and polyline corners. Samples
/// for displaying an arc or spline are kept separately from these stations.
fn curved_path(
    curve: &Curve,
    world: Transform,
    tolerance: f64,
    limits: &Limits,
    check: &mut impl FnMut() -> Result<()>,
) -> Result<(Vec<Point3>, Vec<Point3>)> {
    let mut anchors = Vec::new();
    let mut display = Vec::new();
    let parts = profile_segments(curve, limits)?;
    let mut held = Usage::array::<Curve>(parts.capacity())?;
    for part in &parts {
        held.add(part.validate(limits)?, limits)?;
    }
    let remaining = limits.remaining_after(held)?;
    for part in &parts {
        check()?;
        let mut retained = Usage::array::<Point3>(display.capacity() + anchors.capacity())?;
        retained.points = (display.len() + anchors.len()) as u64;
        let available = remaining.remaining_after(retained)?;
        let (stations, samples) = if is_piecewise_linear(part) {
            let pts = placed_vertices(part, world, tolerance, &available, check)?;
            (pts.clone(), pts)
        } else {
            let evaluator = CurveEvaluator::capture(part, &available)?;
            const PREVIEW_STEPS: usize = 64;
            let mut usage = Usage::array::<Point3>(PREVIEW_STEPS + 1)?;
            usage.points = (PREVIEW_STEPS + 1) as u64;
            usage.check(&available)?;
            let mut samples = Vec::with_capacity(PREVIEW_STEPS + 1);
            for i in 0..=PREVIEW_STEPS {
                check()?;
                samples.push(
                    evaluator.evaluate(
                        Parameter::Fraction(i as f64 / PREVIEW_STEPS as f64),
                        0,
                        world,
                    )?[0],
                );
            }
            let first = samples[0];
            let last = samples[PREVIEW_STEPS];
            let closed = match part {
                Curve::Circle {
                    domain,
                    angle_domain,
                    ..
                } => {
                    let angles = angle_domain.unwrap_or(*domain);
                    let turns = (angles[1] - angles[0]) / std::f64::consts::TAU;
                    turns.round() >= 1.
                        && (turns - turns.round()).abs() <= 8. * f64::EPSILON * turns
                }
                _ => first == last,
            };
            let stations = if closed {
                vec![first]
            } else {
                vec![first, last]
            };
            (stations, samples)
        };
        let n = display.len() + anchors.len() + samples.len() + stations.len();
        let mut usage = Usage::array::<Point3>(2 * n)?;
        usage.points = n as u64;
        usage.check(&remaining)?;
        if let Some(prev) = display.last().copied() {
            let prev: Point3 = prev;
            let d: Point3 = std::array::from_fn(|i| prev[i] - samples[0][i]);
            if d[0].hypot(d[1]).hypot(d[2]) > tolerance {
                return Err(Error::Invalid("disconnected Extrude Lines polycurve"));
            }
        }
        display.extend(samples);
        anchors.extend(stations);
    }
    Ok((anchors, display))
}

#[derive(Debug)]
pub struct CurvedProfile {
    /// Index into the selection in the order passed to capture.
    pub source: usize,
    /// Preview only. Commit copies the exact source curve with a translated placement.
    pub display: Vec<Point3>,
    pub stations: Vec<Point3>,
}

/// Exact world-space profile edges captured once at the end of object selection.
#[derive(Debug)]
pub struct ControlProfiles {
    sources: BTreeMap<Key, Segment>,
    pub start: Point3,
    pub curved: Vec<CurvedProfile>,
    curved_usage: Usage,
}
impl ControlProfiles {
    pub fn capture<'a>(
        profiles: impl IntoIterator<Item = (&'a Curve, Transform)>,
        tolerance: f64,
        limits: &Limits,
        check: &mut impl FnMut() -> Result<()>,
    ) -> Result<Self> {
        let mut start = None;
        let mut sources = BTreeMap::new();
        let mut curved = Vec::new();
        let mut curved_usage = Usage::default();
        for (index, (profile, world)) in profiles.into_iter().enumerate() {
            check()?;
            if index as u64 >= limits.max_objects {
                return Err(Error::Budget("Extrude Lines source count"));
            }
            let available = limits.remaining_after(curved_usage)?;
            let remaining = available.remaining_after(wire_usage(sources.len())?)?;
            profile.validate(&remaining)?;
            if !is_piecewise_linear(profile) {
                let (stations, display) =
                    curved_path(profile, world, tolerance, &remaining, check)?;
                start.get_or_insert(
                    *display
                        .first()
                        .ok_or(Error::Invalid("empty curved profile"))?,
                );
                let mut usage = Usage::array::<Point3>(stations.capacity() + display.capacity())?;
                usage.points = (stations.len() + display.len()) as u64;
                usage.add(Usage::array::<CurvedProfile>(2)?, &remaining)?;
                curved_usage.add(usage, limits)?;
                curved.push(CurvedProfile {
                    source: index,
                    stations,
                    display,
                });
                continue;
            }
            let corners = placed_vertices(profile, world, tolerance, &remaining, check)?;
            start.get_or_insert(corners[0]);
            let mut path_usage = Usage::array::<Point3>(corners.capacity())?;
            path_usage.points = corners.len() as u64;
            let available = available.remaining_after(path_usage)?;
            let mut has_edge = false;
            for edge in corners.windows(2) {
                check()?;
                let (a, b) = (edge[0], edge[1]);
                if a == b {
                    continue;
                }
                has_edge = true;
                let k = key(a, b);
                if !sources.contains_key(&k) {
                    wire_usage(sources.len() + 1)?.check(&available)?;
                    sources.insert(k, [a, b]);
                }
            }
            if !has_edge {
                return Err(Error::Invalid("Extrude Lines needs a nonzero profile edge"));
            }
        }
        Ok(Self {
            sources,
            curved,
            curved_usage,
            start: start.ok_or(Error::Missing("Extrude Lines profiles"))?,
        })
    }
    /// Corners of straight segments and representative points on curved rails for a fan preview.
    pub fn apex_preview_stations(&self) -> impl Iterator<Item = Point3> + '_ {
        self.sources
            .values()
            .flatten()
            .copied()
            .chain(self.curved.iter().flat_map(|rail| {
                rail.stations.iter().copied().chain(
                    rail.display
                        .iter()
                        .step_by((rail.display.len() / 4).max(1))
                        .copied(),
                )
            }))
    }

    pub fn source_wires(&self) -> impl Iterator<Item = [Point3; 2]> + '_ {
        self.sources.values().copied().chain(
            self.curved
                .iter()
                .flat_map(|rail| rail.display.windows(2).map(|pts| [pts[0], pts[1]])),
        )
    }

    /// Return the new line objects required to complete the control polygon. With a
    /// retained profile on the middle plane, split each connector there so From Lines
    /// can find faces on both sides. An input end touching an unsplit edge stays separate.
    pub fn wires(
        &self,
        offset: Point3,
        both_sides: bool,
        retain_input: bool,
        limits: &Limits,
        check: &mut impl FnMut() -> Result<()>,
    ) -> Result<Vec<Segment>> {
        let sources = &self.sources;
        if offset == [0.; 3] || offset.iter().any(|x| !x.is_finite()) {
            return Err(Error::Invalid(
                "Extrude Lines needs a finite nonzero offset",
            ));
        }
        let top = Transform::translation(offset);
        let bottom = Transform::translation(offset.map(|x| -x));
        let available = limits
            .remaining_after(self.curved_usage)?
            .remaining_after(wire_usage(sources.len())?)?;
        let mut wires = BTreeMap::new();
        let mut insert = |p: Point3, q: Point3| -> Result<()> {
            if p == q {
                return Err(Error::Invalid(
                    "Extrude Lines offset below coordinate precision",
                ));
            }
            let k = key(p, q);
            if (retain_input && sources.contains_key(&k)) || wires.contains_key(&k) {
                return Ok(());
            }
            let n = wires.len() + 1;
            let mut usage = wire_usage(n)?;
            usage.outputs = n as u64;
            usage.check(&available)?;
            wires.insert(k, [p, q]);
            Ok(())
        };
        for &[a, b] in sources.values() {
            check()?;
            let (ta, tb) = (top.point(a)?, top.point(b)?);
            let (ba, bb) = if both_sides {
                (bottom.point(a)?, bottom.point(b)?)
            } else {
                (a, b)
            };
            insert(ta, tb)?;
            insert(ba, bb)?;
            if both_sides && retain_input {
                insert(ba, a)?;
                insert(bb, b)?;
                insert(a, ta)?;
                insert(b, tb)?;
            } else {
                insert(ba, ta)?;
                insert(bb, tb)?;
            }
        }
        for profile in &self.curved {
            for &p in &profile.stations {
                check()?;
                let a = if both_sides { bottom.point(p)? } else { p };
                let b = top.point(p)?;
                if both_sides && retain_input {
                    insert(a, p)?;
                    insert(p, b)?;
                } else {
                    insert(a, b)?;
                }
            }
        }
        let mut usage = wire_usage(wires.len())?;
        let mut result_usage = Usage::array::<Segment>(wires.len())?;
        result_usage.points = usage.points;
        usage.add(result_usage, &available)?;
        let mut result = Vec::new();
        result
            .try_reserve_exact(wires.len())
            .map_err(|_| Error::Budget("Extrude Lines output"))?;
        for wire in wires.into_values() {
            check()?;
            result.push(wire);
        }
        Ok(result)
    }
}
