// SPDX-License-Identifier: LGPL-2.1-or-later
//! Object snaps and SmartTrack inference (`OM9-L2-02-03`).
//!
//! The module is pure geometry over a captured [`Scene`]; it never mutates the document and
//! never runs a second picking engine. The viewport supplies the camera, rectangle and cursor.
use crate::{
    camera::{CPlane, Camera, Viewport},
    picking::PickScene,
    scene::{OcclusionSnapshot, Scene},
};
use om9_core::{
    Error, Result,
    geometry::{Curve, Geometry, Point3, Transform, finite},
    identity::{ObjectId, SourceStamp},
};

mod apparent_curves;
pub use apparent_curves::{apparent_curve_intersections, apparent_curve_intersections_with_parameter_map};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SnapKind {
    End,
    Mid,
    Center,
    Quadrant,
    Point,
    Intersection,
    Perpendicular,
    Tangent,
    Near,
    Surface,
    SmartTrack,
}

impl SnapKind {
    pub const ALL: [Self; 11] = [
        Self::End,
        Self::Mid,
        Self::Center,
        Self::Quadrant,
        Self::Point,
        Self::Intersection,
        Self::Perpendicular,
        Self::Tangent,
        Self::Near,
        Self::Surface,
        Self::SmartTrack,
    ];
    pub fn bit(self) -> u16 {
        1 << self as u16
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::End => "End",
            Self::Mid => "Mid",
            Self::Center => "Center",
            Self::Quadrant => "Quadrant",
            Self::Point => "Point",
            Self::Intersection => "Intersection",
            Self::Perpendicular => "Perpendicular",
            Self::Tangent => "Tangent",
            Self::Near => "Near",
            Self::Surface => "Surface",
            Self::SmartTrack => "SmartTrack",
        }
    }
    pub fn tooltip(self) -> &'static str {
        match self {
            Self::End => "End point",
            Self::Mid => "Midpoint",
            Self::Center => "Center",
            Self::Quadrant => "Quadrant",
            Self::Point => "Point",
            Self::Intersection => "Intersection",
            Self::Perpendicular => "Perpendicular",
            Self::Tangent => "Tangent",
            Self::Near => "Near",
            Self::Surface => "On surface",
            Self::SmartTrack => "SmartTrack alignment",
        }
    }
    pub fn priority(self) -> u8 {
        match self {
            Self::End => 0,
            Self::Mid => 1,
            Self::Center => 2,
            Self::Quadrant => 3,
            Self::Point => 4,
            Self::Intersection => 5,
            Self::Perpendicular => 6,
            Self::Tangent => 7,
            Self::Near => 8,
            Self::Surface => 9,
            Self::SmartTrack => 10,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SnapSettings {
    /// Bit set from [`SnapKind::bit`].
    pub enabled: u16,
    /// Suspend persistent object snaps without changing the selected kinds.
    pub master_enabled: bool,
    pub radius_px: f64,
    /// A one-shot kind restricts exactly the next pick; `accept` clears it.
    pub one_shot: Option<SnapKind>,
    /// Allow an isolated view crossing to supply a point on one source.
    pub apparent_intersections: bool,
}

impl Default for SnapSettings {
    fn default() -> Self {
        Self {
            enabled: SnapKind::ALL.iter().fold(0, |bits, kind| bits | kind.bit()),
            master_enabled: true,
            radius_px: 8.0,
            one_shot: None,
            apparent_intersections: false,
        }
    }
}

impl SnapSettings {
    pub fn set(&mut self, kind: SnapKind, enabled: bool) {
        if enabled {
            self.enabled |= kind.bit();
        } else {
            self.enabled &= !kind.bit();
        }
    }
    pub fn is_enabled(self, kind: SnapKind) -> bool {
        self.enabled & kind.bit() != 0
    }
    pub fn is_effective(self, kind: SnapKind) -> bool {
        self.one_shot.map_or(self.master_enabled && self.is_enabled(kind), |one_shot| one_shot == kind)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SnapCandidate {
    pub kind: SnapKind,
    pub root: Option<ObjectId>,
    pub world: Point3,
    pub screen: [f64; 2],
    pub distance_px: f64,
    /// The two source witnesses met in the view while remaining separate in 3D.
    pub apparent: bool,
    pub normal: Option<Point3>,
    pub parameter: Option<f64>,
    /// Direction of the evaluated source curve at an End snap.
    pub tracking_tangent: Option<Point3>,
    /// Source identity for a point/curve candidate, bound to one scene publication.
    pub source: Option<SnapSource>,
    /// Second source for an intersection; absent for a single-source candidate.
    pub second_source: Option<SnapSource>,
    /// Source of a B-rep surface candidate. Its point and normal are display-mesh
    /// approximations until an owning kernel verifies the same source and face.
    pub surface_source: Option<SurfaceCandidateSource>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SnapSource {
    pub root: ObjectId,
    pub path: Vec<ObjectId>,
    pub leaf: ObjectId,
    pub leaf_generation: u64,
    pub leaf_geometry_revision: u64,
    pub stamp: SourceStamp,
    pub scene_epoch: u64,
    pub scene_publication: uuid::Uuid,
}

impl SnapSource {
    pub fn validate(&self, scene: &Scene) -> Result<()> {
        if scene.epoch != self.scene_epoch || scene.publication_id() != self.scene_publication
            || scene.document.resolve(&self.stamp).is_err() {
            return Err(Error::Stale("snap source publication"));
        }
        if !scene.objects.iter().any(|object| object.root == self.root && object.path == self.path
            && object.leaf == self.leaf && object.source == self.stamp
            && object.leaf_generation == self.leaf_generation && object.leaf_geometry_revision == self.leaf_geometry_revision
            && object.visible && (object.selectable || object.reference_snappable)) {
            return Err(Error::Stale("snap source instance"));
        }
        Ok(())
    }
}

fn snap_source(scene: &Scene, object: &crate::scene::SceneObject) -> SnapSource {
    SnapSource { root: object.root, path: object.path.clone(), leaf: object.leaf,
        leaf_generation: object.leaf_generation, leaf_geometry_revision: object.leaf_geometry_revision,
        stamp: object.source, scene_epoch: scene.epoch, scene_publication: scene.publication_id() }
}

/// Bind a scene occurrence to source-curve work retained outside the scene.
pub fn occurrence_source(scene: &Scene, index: usize) -> Result<SnapSource> {
    let object = scene.objects.get(index).ok_or(Error::Invalid("snap occurrence index"))?;
    let source = snap_source(scene, object);
    source.validate(scene)?;
    Ok(source)
}

pub fn contains_nurbs(curve: &Curve) -> bool {
    match curve {
        Curve::Nurbs(_) => true,
        Curve::Polycurve { segments, .. } => segments.iter().any(contains_nurbs),
        _ => false,
    }
}

/// Whether a source contains a circular segment eligible for deferred midpoint work.
pub fn contains_circle(curve: &Curve) -> bool {
    match curve {
        Curve::Circle { .. } => true,
        Curve::Polycurve { segments, .. } => segments.iter().any(contains_circle),
        _ => false,
    }
}

/// Evaluate extracted source curves using the ordinary object-snap solver.
/// The caller must validate `source` against the current scene before using the
/// returned candidates. Intersections between sources are handled separately.
#[allow(clippy::too_many_arguments)]
pub fn source_curve_candidates(
    curves: &[Curve], vertices: &[Point3], source: &SnapSource, transform: Transform, camera: &Camera,
    viewport: Viewport, cursor: [f64; 2], mut settings: SnapSettings, from: Option<Point3>,
    limits: &om9_core::budget::Limits, cplane: CPlane, tolerance: f64,
    canceled: impl Fn() -> bool,
) -> Result<Vec<SnapCandidate>> {
    if !settings.radius_px.is_finite() || settings.radius_px < 0.
        || !tolerance.is_finite() || tolerance <= 0. {
        return Err(Error::Invalid("source curve snap parameters"));
    }
    if settings.one_shot == Some(SnapKind::Intersection) { return Ok(Vec::new()); }
    settings.set(SnapKind::Intersection, false);
    let mut out = Vec::new();
    let mut segments = Vec::new();
    let mut ellipses = Vec::new();
    let mut work = 0usize;
    let mut end_directions = settings.is_effective(SnapKind::End).then(EndDirections::default);
    for curve in curves {
        if canceled() { return Err(Error::Canceled); }
        let start = out.len();
        if let Curve::Nurbs(nurbs) = curve
            && let Some(&pole) = nurbs.poles.first()
            && nurbs.poles.iter().all(|point| *point == pole) {
            let point = transform.point(pole)?;
            if settings.is_effective(SnapKind::End) {
                push_curve_end(source.root, point, None, None, camera, viewport, cursor,
                    settings.radius_px, &mut out, &mut end_directions, |_| None)?;
            }
            if settings.is_effective(SnapKind::Near) {
                push(SnapKind::Near, Some(source.root), point, None, None, camera, viewport, cursor, settings.radius_px, &mut out)?;
            }
        } else {
            curve_snaps(curve, [0., 1.], source.root, 0, transform, camera, viewport, cursor, settings, from,
                limits, cplane, tolerance, &mut out, &mut end_directions, &mut segments, &mut ellipses, &mut work, &canceled, false)?;
        }
        if out.len() as u64 > limits.max_outputs {
            return Err(Error::Budget("source curve snap candidates"));
        }
        for candidate in &mut out[start..] { candidate.source = Some(source.clone()); }
    }
    out.retain(|candidate| settings.is_effective(candidate.kind));
    if let Some(end_directions) = end_directions {
        let mut vertex_hits = source_point_candidates(vertices, SnapKind::End, source, transform,
            camera, viewport, cursor, settings.radius_px, limits.max_outputs.saturating_sub(out.len() as u64), &canceled)?;
        end_directions.resolve(&mut out, &mut vertex_hits, tolerance, &canceled)?;
        out.try_reserve(vertex_hits.len()).map_err(|_| Error::Budget("source End candidates"))?;
        out.extend(vertex_hits);
    }
    out.sort_by(snap_order);
    out.dedup_by(|left, right| left.kind == right.kind && left.parameter == right.parameter
        && distance(left.world, right.world) <= 1e-9);
    Ok(out)
}

type EndWitness = (Point3, Option<Point3>);
const MAX_END_WITNESSES: usize = 4096;

/// Maximum temporary witness storage for one source query.
pub const END_DIRECTION_SCRATCH_BYTES: usize = MAX_END_WITNESSES * std::mem::size_of::<EndWitness>();

/// Keep source endpoints independently of the pointer aperture. A partial
/// witness list cannot establish that a position has one common direction.
struct EndDirections {
    witnesses: Option<Vec<EndWitness>>,
    derivative_work: usize,
}

impl Default for EndDirections {
    fn default() -> Self { Self { witnesses: Some(Vec::new()), derivative_work: 0 } }
}

impl EndDirections {
    fn record(&mut self, world: Point3, tangent: Option<Point3>) {
        let Some(witnesses) = &mut self.witnesses else { return; };
        let count = witnesses.len();
        let growth = count.max(32).min(MAX_END_WITNESSES - count);
        if count == MAX_END_WITNESSES
            || count == witnesses.capacity() && witnesses.try_reserve_exact(growth).is_err()
        {
            self.witnesses = None;
            return;
        }
        witnesses.push((world, tangent));
    }

    fn resolve(self, curves: &mut [SnapCandidate], vertices: &mut [SnapCandidate],
        tolerance: f64, canceled: &dyn Fn() -> bool) -> Result<()> {
        if !curves.iter().chain(vertices.iter()).any(|hit| hit.kind == SnapKind::End) { return Ok(()); }
        let mut remaining = 1_000_000usize;
        if let Some(mut witnesses) = self.witnesses {
            witnesses.sort_unstable_by(|a, b| a.0[0].total_cmp(&b.0[0]));
            'resolve: for hit in curves.iter_mut().chain(vertices.iter_mut()).filter(|hit| hit.kind == SnapKind::End) {
                if canceled() { return Err(Error::Canceled); }
                let low = witnesses.partition_point(|(world, _)| world[0] < hit.world[0] - tolerance);
                let mut direction = None;
                for &(world, tangent) in &witnesses[low..] {
                    if world[0] > hit.world[0] + tolerance { break; }
                    if remaining == 0 { break 'resolve; }
                    remaining -= 1;
                    if distance(world, hit.world) > tolerance { continue; }
                    let Some(tangent) = tangent else { direction = None; break; };
                    if direction.is_some_and(|other| dot(other, tangent).abs() < 1. - 1e-12) {
                        direction = None;
                        break;
                    }
                    direction = Some(tangent);
                }
                hit.tracking_tangent = direction;
            }
            if remaining > 0 { return Ok(()); }
        }
        // Optional guide work must not prevent these End snaps.
        for hit in curves.iter_mut().chain(vertices) { hit.tracking_tangent = None; }
        Ok(())
    }
}

/// Turn source-local topological vertices or analytic edge centers into source-bound snaps.
pub fn source_point_candidates(points: &[Point3], kind: SnapKind, source: &SnapSource, transform: Transform,
    camera: &Camera, viewport: Viewport, cursor: [f64; 2], radius: f64,
    max_candidates: u64, canceled: impl Fn() -> bool) -> Result<Vec<SnapCandidate>> {
    if !matches!(kind, SnapKind::End | SnapKind::Center) { return Err(Error::Invalid("source point snap kind")); }
    let mut out = Vec::new();
    for &point in points {
        if canceled() { return Err(Error::Canceled); }
        let world = transform.point(point)?;
        let Some(projected) = camera.project(world, viewport)? else { continue; };
        let distance_px = (projected.screen[0] - cursor[0]).hypot(projected.screen[1] - cursor[1]);
        if !distance_px.is_finite() || !(distance_px <= radius) { continue; }
        if out.len() as u64 >= max_candidates { return Err(Error::Budget("source point snap candidates")); }
        out.try_reserve(1).map_err(|_| Error::Budget("source point snap candidates"))?;
        out.push(SnapCandidate { kind, root: Some(source.root), world, screen: projected.screen,
            distance_px, normal: None, parameter: None, tracking_tangent: None, source: Some(source.clone()),
            second_source: None, surface_source: None, apparent: false });
    }
    Ok(out)
}

/// Evaluate one pair of independently captured source curves in world space.
/// Sampled fallback intersections are verified against both sources within the
/// document tolerance, but the underlying solver does not certify completeness.
#[allow(clippy::too_many_arguments)]
pub fn source_curve_intersection_candidates(
    first: &Curve, first_transform: Transform, first_source: &SnapSource,
    second: &Curve, second_transform: Transform, second_source: &SnapSource,
    camera: &Camera, viewport: Viewport, cursor: [f64; 2], radius: f64,
    tolerance: f64, limits: &om9_core::budget::Limits, canceled: &dyn Fn() -> bool,
) -> Result<Vec<SnapCandidate>> {
    if canceled() { return Err(Error::Canceled); }
    if !radius.is_finite() || radius < 0. || !tolerance.is_finite() || tolerance <= 0. {
        return Err(Error::Invalid("source curve intersection parameters"));
    }
    for curve in [first, second] {
        match curve {
            Curve::Nurbs(nurbs) if nurbs.knots.len() > 383 => return Err(Error::Budget("intersection source spans")),
            Curve::Polycurve { .. } => return Err(Error::Unsupported("intersection requires individual source segments")),
            _ => {}
        }
        curve.validate(limits)?;
    }
    let first_baked = om9_core::top11::bake_curve(first, first_transform)?;
    if canceled() { return Err(Error::Canceled); }
    let second_baked = om9_core::top11::bake_curve(second, second_transform)?;
    let hits = om9_core::curve_surface_ops::curve_curve_intersections_cancellable(
        &first_baked, &second_baked, tolerance, limits, canceled)?;
    let mut candidates = Vec::new();
    for hit in hits {
        if canceled() { return Err(Error::Canceled); }
        let (first_point, parameter) = source_intersection_witness(first, first_transform, hit.first_fraction, hit.point, tolerance, limits)?;
        let (second_point, _) = source_intersection_witness(second, second_transform, hit.second_fraction, hit.point, tolerance, limits)?;
        let separation = distance(first_point, second_point);
        if !separation.is_finite() || separation > tolerance {
            return Err(Error::Unsupported("intersection source witness mismatch"));
        }
        let world = midpoint(first_point, second_point);
        let before = candidates.len();
        push(SnapKind::Intersection, Some(first_source.root), world, None, Some(parameter),
            camera, viewport, cursor, radius, &mut candidates)?;
        if candidates.len() > before {
            let candidate = candidates.last_mut().expect("intersection candidate appended");
            candidate.source = Some(first_source.clone());
            candidate.second_source = Some(second_source.clone());
        }
    }
    Ok(candidates)
}

/// Find an isolated crossing of two visible source segments in the active view.
/// The result carries the point and parameter of the selected original segment.
pub fn apparent_segment_intersection(
    first: ([Point3; 2], &SnapSource),
    second: ([Point3; 2], &SnapSource),
    camera: &Camera,
    viewport: Viewport,
    cursor: [f64; 2],
    radius_px: f64,
    tolerance_mm: f64,
) -> Result<Option<SnapCandidate>> {
    apparent_segment_intersection_with_parameter_map(
        first, second, camera, viewport, cursor, radius_px, tolerance_mm,
        &|_, parameter| parameter,
    )
}

fn apparent_segment_intersection_with_parameter_map(
    (first, first_source): ([Point3; 2], &SnapSource),
    (second, second_source): ([Point3; 2], &SnapSource),
    camera: &Camera,
    viewport: Viewport,
    cursor: [f64; 2],
    radius_px: f64,
    tolerance_mm: f64,
    map_parameter: &dyn Fn(usize, f64) -> f64,
) -> Result<Option<SnapCandidate>> {
    if !cursor.iter().all(|coordinate| coordinate.is_finite())
        || !radius_px.is_finite() || radius_px < 0.
        || !tolerance_mm.is_finite() || tolerance_mm <= 0.
    {
        return Err(Error::Invalid("apparent intersection parameters"));
    }
    let Some(first_visible) = camera.clip_segment(first[0], first[1])? else { return Ok(None) };
    let Some(second_visible) = camera.clip_segment(second[0], second[1])? else { return Ok(None) };
    let projector = camera.projector(viewport)?;
    let (Some(a), Some(b), Some(c), Some(d)) = (
        projector.project(first_visible[0])?, projector.project(first_visible[1])?,
        projector.project(second_visible[0])?, projector.project(second_visible[1])?,
    ) else { return Ok(None) };
    let delta = |start: [f64; 2], end: [f64; 2]| [end[0] - start[0], end[1] - start[1]];
    let cross = |a: [f64; 2], b: [f64; 2]| a[0] * b[1] - a[1] * b[0];
    let r = delta(a.screen, b.screen);
    let s = delta(c.screen, d.screen);
    let denominator = cross(r, s);
    if !denominator.is_finite()
        || denominator.abs() <= 64. * f64::EPSILON * r[0].hypot(r[1]) * s[0].hypot(s[1])
    {
        return Ok(None);
    }
    let offset = delta(a.screen, c.screen);
    let t = cross(offset, s) / denominator;
    let u = cross(offset, r) / denominator;
    let parameter_slack = 64. * f64::EPSILON;
    if [t, u].iter().any(|fraction| {
        !(-parameter_slack..=1. + parameter_slack).contains(fraction)
    }) {
        return Ok(None);
    }
    let on_segment = |segment: [Point3; 2], fraction: f64, depths| -> Result<Point3> {
        let t = camera.projection.segment_source_fraction(fraction.clamp(0., 1.), depths)?;
        Ok(add(segment[0], mul(sub(segment[1], segment[0]), t)))
    };
    let first_point = on_segment(first_visible, t, [a.depth, b.depth])?;
    let second_point = on_segment(second_visible, u, [c.depth, d.depth])?;
    let world_separation = distance(first_point, second_point);
    if !world_separation.is_finite() || world_separation <= tolerance_mm {
        return Ok(None);
    }
    let (Some(first_hit), Some(second_hit)) = (
        projector.project(first_point)?, projector.project(second_point)?,
    ) else { return Ok(None) };
    let separation = delta(first_hit.screen, second_hit.screen);
    if separation[0].hypot(separation[1]) > 1e-7 {
        return Ok(None);
    }
    let screen_distance = |start, direction: [f64; 2]| {
        let offset = delta(start, cursor);
        let t = ((offset[0] * direction[0] + offset[1] * direction[1])
            / (direction[0] * direction[0] + direction[1] * direction[1])).clamp(0., 1.);
        (offset[0] - t * direction[0]).hypot(offset[1] - t * direction[1])
    };
    let first_distance = screen_distance(a.screen, r);
    let second_distance = screen_distance(c.screen, s);
    let screen_slack = 64. * f64::EPSILON * cursor[0].abs().max(cursor[1].abs()).max(1.);
    let first_wins = apparent_source_precedes(
        (first_distance, first_hit.depth), (second_distance, second_hit.depth), screen_slack,
    );
    let (world, hit, segment, source, other) = if first_wins {
        (first_point, first_hit, first, first_source, second_source)
    } else {
        (second_point, second_hit, second, second_source, first_source)
    };
    let offset = delta(hit.screen, cursor);
    let distance_px = offset[0].hypot(offset[1]);
    if distance_px > radius_px {
        return Ok(None);
    }
    let (_, source_parameter) = closest_on_segment(world, segment[0], segment[1]);
    Ok(Some(SnapCandidate {
        kind: SnapKind::Intersection,
        root: Some(source.root),
        world,
        screen: hit.screen,
        distance_px,
        apparent: true,
        normal: None,
        parameter: Some(map_parameter(usize::from(!first_wins), source_parameter)),
        tracking_tangent: None,
        source: Some(source.clone()),
        second_source: Some(other.clone()),
        surface_source: None,
    }))
}

fn apparent_source_precedes(first: (f64, f64), second: (f64, f64), screen_slack: f64) -> bool {
    if first.0 + screen_slack < 0.95 * second.0 {
        true
    } else if second.0 + screen_slack < 0.95 * first.0 {
        false
    } else {
        first.1 <= second.1
    }
}

/// Keep a previously accepted pair when two computed pixel distances coincide.
pub fn intersection_pick_is_closer(new: &SnapCandidate, accepted: &SnapCandidate, cursor: [f64; 2]) -> bool {
    let slack = 64. * f64::EPSILON * cursor[0].abs().max(cursor[1].abs()).max(1.);
    new.distance_px + slack < accepted.distance_px
}

/// Resolve a solver witness back onto the original placed source. A baked
/// rational ellipse does not use the original circle's angular parameter.
fn source_intersection_witness(curve: &Curve, transform: Transform, fraction: f64, witness: Point3,
    tolerance: f64, limits: &om9_core::budget::Limits) -> Result<(Point3, f64)> {
    let (point, parameter) = if let Curve::Circle { center, x, y, radius, domain, angle_domain } = curve {
        let center = transform.point(*center)?;
        let x = transform_vector(transform, *x)?; let y = transform_vector(transform, *y)?;
        let delta = sub(witness, center);
        let (xx, xy, yy) = (dot(x, x), dot(x, y), dot(y, y));
        let determinant = xx * yy - xy * xy;
        if !determinant.is_finite() || determinant <= f64::EPSILON * xx * yy {
            return Err(Error::Unsupported("intersection ellipse source frame"));
        }
        let (dx, dy) = (dot(delta, x), dot(delta, y));
        let mut angle = ((dy * xx - dx * xy) / determinant).atan2((dx * yy - dy * xy) / determinant);
        let angles = angle_domain.unwrap_or(*domain);
        if !angle_on_arc(angle, angles) {
            let endpoint = |a: f64| add(center, add(mul(x, *radius * a.cos()), mul(y, *radius * a.sin())));
            let first = distance(endpoint(angles[0]), witness);
            let last = distance(endpoint(angles[1]), witness);
            if first.min(last) > tolerance { return Err(Error::Unsupported("intersection outside source arc")); }
            angle = if first <= last { angles[0] } else { angles[1] };
        }
        let point = add(center, add(mul(x, *radius * angle.cos()), mul(y, *radius * angle.sin())));
        (point, angles[0] + (angle - angles[0]).rem_euclid(std::f64::consts::TAU))
    } else {
        let evaluator = om9_core::evaluation::CurveEvaluator::capture(curve, limits)?;
        let point = evaluator.evaluate(om9_core::evaluation::Parameter::Fraction(fraction), 0, transform)?[0];
        (point, fraction)
    };
    let separation = distance(point, witness);
    if !parameter.is_finite() || !separation.is_finite() || separation > tolerance {
        return Err(Error::Unsupported("intersection source parameterization"));
    }
    Ok((point, parameter))
}

#[derive(Debug, Clone, PartialEq)]
pub struct SurfaceCandidateSource {
    pub path: Vec<ObjectId>,
    pub leaf: ObjectId,
    pub leaf_generation: u64,
    pub leaf_geometry_revision: u64,
    pub brep_display_fingerprint: Option<crate::scene::BrepDisplayReceipt>,
    pub source: SourceStamp,
    pub scene_epoch: u64,
    pub scene_publication: uuid::Uuid,
    pub placement: Transform,
    pub camera: Camera,
    pub viewport: Viewport,
    pub cursor: [f64; 2],
    /// None when the retained native display does not provide a B-rep face binding.
    pub face: Option<uuid::Uuid>,
    /// Local-space triangle from the exact scene display and its deflection.
    pub face_witness: Option<([Point3; 3], f64)>,
    pub face_witness_triangle: Option<usize>,
    /// Source-backed NativeBrep face count, filled only by exact refinement.
    pub native_face_count: Option<usize>,
}

impl SnapCandidate {
    pub fn tooltip(&self) -> String {
        if self.apparent { "Apparent intersection" } else { self.kind.tooltip() }.to_owned()
    }
    pub fn marker_label(&self) -> &'static str {
        if self.apparent { "Apparent Int" } else { self.kind.label() }
    }
}

/// A transient anchor captured from an accepted snap. It lives only in the viewport session
/// and is never written to the document.
#[derive(Debug, Clone, PartialEq)]
pub struct SnapAnchor {
    pub kind: SnapKind,
    pub world: Point3,
    pub screen: [f64; 2],
    tangent: Option<Point3>,
}

pub const MAX_SMARTTRACK_GUIDES: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SnapGuide {
    pub start: Point3,
    pub end: Point3,
    /// CPlane X, CPlane Y, source tangent, or perpendicular (0 through 3).
    pub axis: u8,
}

impl SnapGuide {
    /// Extend the guide beyond the view depth before camera clipping.
    pub fn segment_for_view(&self, camera: &Camera) -> Option<[Point3; 2]> {
        let center = midpoint(self.start, self.end);
        let direction = sub(self.end, self.start);
        let length = dot(direction, direction).sqrt();
        let extent = (camera.far_mm + distance(center, camera.eye)).min(1e12);
        if !length.is_finite() || length <= 0. || !extent.is_finite() {
            return None;
        }
        let delta = mul(direction, extent / length);
        Some([sub(center, delta), add(center, delta)])
    }
}

#[derive(Debug, Clone, Default)]
pub struct SmartTrack {
    anchors: Vec<SnapAnchor>,
    guides: Vec<SnapGuide>,
}

impl SmartTrack {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn anchors(&self) -> &[SnapAnchor] {
        &self.anchors
    }
    pub fn guides(&self) -> &[SnapGuide] {
        &self.guides
    }
    /// Keep at most two reference anchors and rebuild their ephemeral alignment lines.
    pub fn push(&mut self, candidate: &SnapCandidate, plane: CPlane) -> Result<()> {
        self.acquire(candidate, plane, 0.).map(|_| ())
    }

    /// Acquire a distinct source point without repeatedly replacing it on each hover frame.
    pub fn acquire(&mut self, candidate: &SnapCandidate, plane: CPlane, tolerance: f64) -> Result<bool> {
        if !tolerance.is_finite() || tolerance < 0. { return Err(Error::Invalid("SmartTrack tolerance")); }
        if candidate.kind == SnapKind::SmartTrack
            || candidate.kind == SnapKind::Perpendicular && candidate.source.is_none()
            || self.anchors.iter().any(|anchor|
            anchor.kind == candidate.kind && distance(anchor.world, candidate.world) <= tolerance) {
            return Ok(false);
        }
        if self.anchors.len() == 2 {
            self.anchors.remove(0);
        }
        self.anchors.push(SnapAnchor {
            kind: candidate.kind,
            world: candidate.world,
            screen: candidate.screen,
            tangent: candidate.tracking_tangent.filter(|_| candidate.kind == SnapKind::End),
        });
        self.guides.clear();
        let extent = 1000.0;
        for anchor in &self.anchors {
            let perpendicular = anchor.tangent.and_then(|tangent| unit_direction(cross(plane.normal(), tangent)));
            let directions = [Some(plane.x), Some(plane.y), anchor.tangent, perpendicular];
            let first = self.guides.len();
            for (axis, direction) in directions.into_iter().enumerate() {
                let Some(direction) = direction else { continue; };
                if self.guides[first..].iter().any(|guide| {
                    let previous = mul(sub(guide.end, guide.start), 0.5 / extent);
                    dot(direction, previous).abs() >= 1. - 1e-12
                }) { continue; }
                self.guides.push(SnapGuide {
                    start: add(anchor.world, mul(direction, -extent)),
                    end: add(anchor.world, mul(direction, extent)),
                    axis: axis as u8,
                });
            }
        }
        Ok(true)
    }
    /// The nearest smart point, guide projection, or guide intersection within tolerance.
    /// The result is ephemeral and never a document point.
    pub fn infer(&self, cursor: Point3, tolerance: f64) -> Option<SnapCandidate> {
        self.infer_with_source_tolerance(cursor, tolerance, 1e-6)
    }
    pub fn infer_with_source_tolerance(&self, cursor: Point3, tolerance: f64, source_tolerance: f64) -> Option<SnapCandidate> {
        if !tolerance.is_finite() || tolerance < 0. || !source_tolerance.is_finite() || source_tolerance <= 0. {
            return None;
        }
        let mut best: Option<(f64, Point3)> = None;
        let consider = |best: &mut Option<(f64, Point3)>, point: Point3| {
            let separation = distance(point, cursor);
            if separation <= tolerance && best.is_none_or(|(old, _)| separation < old) {
                *best = Some((separation, point));
            }
        };
        for point in self.reference_points(source_tolerance) {
            consider(&mut best, point);
        }
        // A guide projection is always as close as a crossing on that guide.
        // Only compare projections after testing nearby points and crossings.
        if best.is_none() {
            for guide in &self.guides {
                let direction = sub(guide.end, guide.start);
                let length_squared = dot(direction, direction);
                if length_squared > 0. {
                    let t = dot(sub(cursor, guide.start), direction) / length_squared;
                    consider(&mut best, add(guide.start, mul(direction, t)));
                }
            }
        }
        best.map(|(_, world)| transient_tracking_hit(SnapKind::SmartTrack, world, [0.; 2], 0.))
    }

    fn reference_points(&self, source_tolerance: f64) -> impl Iterator<Item = Point3> + '_ {
        let crossings = self.guides.iter().enumerate().flat_map(move |(index, left)| {
            self.guides[index + 1..].iter().filter_map(move |right| {
                if left.axis == right.axis && left.axis < 2 {
                    return None;
                }
                let (point, separation) = line_intersection(left.start, left.end, right.start, right.end)?;
                (separation <= source_tolerance).then_some(point)
            })
        });
        self.anchors.iter().map(|anchor| anchor.world).chain(crossings)
    }

    /// Pick the displayed guide in pixels while retaining its world-space height.
    pub fn infer_in_view(&self, camera: &Camera, viewport: Viewport, cursor: [f64; 2],
        radius_px: f64, source_tolerance: f64) -> Result<Option<SnapCandidate>> {
        if !radius_px.is_finite() || radius_px < 0. || !source_tolerance.is_finite() || source_tolerance <= 0. {
            return Err(Error::Invalid("SmartTrack view query"));
        }
        let mut best: Option<SnapCandidate> = None;
        let consider = |best: &mut Option<SnapCandidate>, point| -> Result<()> {
            consider_tracking_view_hit(best, SnapKind::SmartTrack, point, camera, viewport, cursor, radius_px)
        };
        for point in self.reference_points(source_tolerance) {
            consider(&mut best, point)?;
        }
        if best.is_none() {
            for guide in &self.guides {
                if let Some([start, end]) = guide.segment_for_view(camera)
                    && let Some((point, _)) = closest_segment_on_screen(camera, viewport, cursor, start, end)?
                {
                    consider(&mut best, point)?;
                }
            }
        }
        Ok(best)
    }

    /// Find a true perpendicular foot on a temporary infinite guide near the pointer.
    pub fn infer_perpendicular_in_view(&self, base: Point3, camera: &Camera, viewport: Viewport,
        cursor: [f64; 2], radius_px: f64) -> Result<Option<SnapCandidate>> {
        if !radius_px.is_finite() || radius_px < 0. {
            return Err(Error::Invalid("SmartTrack perpendicular aperture"));
        }
        let mut best = None;
        for guide in &self.guides {
            let origin = midpoint(guide.start, guide.end);
            let direction = sub(guide.end, guide.start);
            let length_squared = dot(direction, direction);
            if !length_squared.is_finite() || length_squared <= 0. { continue; }
            let foot = add(origin, mul(direction, dot(sub(base, origin), direction) / length_squared));
            consider_tracking_view_hit(&mut best, SnapKind::Perpendicular, foot, camera, viewport,
                cursor, radius_px)?;
        }
        Ok(best)
    }

    /// Intersect tracking guides with source lines and circular/affine arcs.
    /// Unsupported curve families are left for the source-curve worker.
    pub fn infer_on_curves(&self, scene: &Scene, camera: &Camera, viewport: Viewport,
        cursor: [f64; 2], radius_px: f64, tolerance: f64) -> Result<Option<SnapCandidate>> {
        if !radius_px.is_finite() || radius_px < 0. || !tolerance.is_finite() || tolerance <= 0. {
            return Err(Error::Invalid("SmartTrack curve query"));
        }
        let mut hits = Vec::new();
        let mut work = 0usize;
        for (index, object) in scene.objects.iter().enumerate() {
            if !object.visible || !(object.selectable || object.reference_snappable) { continue; }
            let entity = scene.entity_for(&scene.document, object)?;
            let Geometry::Curve(curve) = &entity.geometry else { continue };
            let start = hits.len();
            for guide in &self.guides {
                track_curve_hits(curve, object.root, index, object.display_transform()?, *guide,
                    camera, viewport, cursor, radius_px, tolerance, &mut work, &mut hits)?;
            }
            if hits.len() > start {
                let source = snap_source(scene, object);
                for hit in &mut hits[start..] { hit.source = Some(source.clone()); }
            }
        }
        Ok(choose(&hits))
    }
    pub fn clear(&mut self) {
        self.anchors.clear();
        self.guides.clear();
    }
}

fn transient_tracking_hit(kind: SnapKind, world: Point3, screen: [f64; 2], distance_px: f64) -> SnapCandidate {
    SnapCandidate {
        kind, root: None, world, screen, distance_px,
        normal: None, parameter: None, tracking_tangent: None, source: None, second_source: None, surface_source: None, apparent: false,
    }
}

fn consider_tracking_view_hit(best: &mut Option<SnapCandidate>, kind: SnapKind, world: Point3,
    camera: &Camera, viewport: Viewport, cursor: [f64; 2], radius_px: f64) -> Result<()> {
    let Some(projected) = camera.project(world, viewport)? else { return Ok(()); };
    let distance_px = (projected.screen[0] - cursor[0]).hypot(projected.screen[1] - cursor[1]);
    if distance_px <= radius_px && best.as_ref().is_none_or(|hit| distance_px < hit.distance_px) {
        *best = Some(transient_tracking_hit(kind, world, projected.screen, distance_px));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn track_curve_hits(curve: &Curve, root: ObjectId, object_index: usize, transform: Transform,
    guide: SnapGuide, camera: &Camera, viewport: Viewport, cursor: [f64; 2], radius_px: f64,
    tolerance: f64, work: &mut usize, hits: &mut Vec<SnapCandidate>) -> Result<()> {
    *work = work.checked_add(1).ok_or(Error::Overflow("SmartTrack curve work"))?;
    // ponytail: cap synchronous analytic pairs; use a scene-revision worker for larger models.
    if *work > 50_000 { return Err(Error::Budget("SmartTrack curve work")); }
    match curve {
        Curve::Line { start, end, .. } => {
            let (a, b) = (transform.point(*start)?, transform.point(*end)?);
            if let Some((point, separation)) = line_intersection(guide.start, guide.end, a, b)
                && separation <= tolerance {
                let (on_curve, _) = closest_on_segment(point, a, b);
                if distance(point, on_curve) <= tolerance {
                    push(SnapKind::SmartTrack, Some(root), on_curve, None, None, camera, viewport, cursor, radius_px, hits)?;
                }
            }
        }
        Curve::Circle { center, x, y, radius, domain, angle_domain } => {
            let center = transform.point(*center)?;
            let x = transform_vector(transform, *x)?;
            let y = transform_vector(transform, *y)?;
            let axis = sub(guide.end, guide.start);
            let length = dot(axis, axis).sqrt();
            if !length.is_finite() || length <= 0. { return Ok(()); }
            let origin = midpoint(guide.start, guide.end);
            let extent = distance(origin, center) + radius.abs() * (dot(x, x).sqrt() + dot(y, y).sqrt()) + tolerance;
            if !extent.is_finite() || extent <= 0. { return Err(Error::Invalid("SmartTrack guide extent")); }
            let delta = mul(axis, extent / length);
            let segment = Segment { root, object_index, a: sub(origin, delta), b: add(origin, delta),
                source_range: [0., 1.], source_weights: [1.; 2] };
            let ellipse = Ellipse { object_index, center, x, y, radius: *radius, angles: angle_domain.unwrap_or(*domain),
                source_range: [0., 1.] };
            for point in segment_ellipse_intersections(segment, ellipse, tolerance) {
                push(SnapKind::SmartTrack, Some(root), point, None, None, camera, viewport, cursor, radius_px, hits)?;
            }
        }
        Curve::Polycurve { segments, .. } => for part in segments {
            track_curve_hits(part, root, object_index, transform, guide, camera, viewport, cursor, radius_px, tolerance, work, hits)?;
        },
        Curve::Nurbs(_) => {},
    }
    Ok(())
}

#[derive(Debug, Clone, Default)]
pub struct SnapState {
    pub settings: SnapSettings,
}

impl SnapState {
    pub fn best(
        &self,
        scene: &Scene,
        camera: &Camera,
        viewport: Viewport,
        cursor: [f64; 2],
        from: Option<Point3>,
    ) -> Result<Option<SnapCandidate>> {
        let mut all = candidates(scene, camera, viewport, cursor, self.settings, from)?;
        if let Some(one_shot) = self.settings.one_shot {
            all.retain(|candidate| candidate.kind == one_shot);
        }
        Ok(choose(&all))
    }
    /// A committed pick consumes the one-shot override exactly once.
    pub fn accept(&mut self) {
        self.settings.one_shot = None;
    }
    /// Cancelling a prompt also consumes the one-shot override; the next pick cannot leak it.
    pub fn cancel(&mut self) {
        self.settings.one_shot = None;
    }
}

pub fn choose(candidates: &[SnapCandidate]) -> Option<SnapCandidate> {
    choose_iter(candidates.iter())
}

/// Select without building another candidate array; preserves iterator order for ties.
pub fn choose_iter<'a>(candidates: impl Iterator<Item = &'a SnapCandidate>) -> Option<SnapCandidate> {
    choose_ref(candidates).cloned()
}

/// Select a candidate without copying its source path.
pub fn choose_ref<'a>(candidates: impl Iterator<Item = &'a SnapCandidate>) -> Option<&'a SnapCandidate> {
    candidates.min_by(|left, right| snap_order(left, right))
}

/// True if a visible filled display surface intersects the view ray before the target.
/// Vertices/triangles are the same displayed buffers used by scene picking.
pub fn visible_display_occludes(
    scene: &Scene,
    camera: &Camera,
    viewport: Viewport,
    cursor: [f64; 2],
    target: Point3,
    tolerance: f64,
    excluded: &[ObjectId],
    target_source: Option<&SurfaceCandidateSource>,
) -> Result<bool> {
    visible_display_occludes_with_margin(scene, camera, viewport, cursor, target, tolerance, excluded, target_source, &mut |_| false)
}

/// Like `visible_display_occludes`, but lets the caller prove that an
/// unverified trim fallback cannot occlude this ray. False keeps the fallback.
pub fn visible_display_occludes_with_fallback_proof(
    scene: &Scene, camera: &Camera, viewport: Viewport, cursor: [f64;2], target: Point3,
    tolerance: f64, excluded: &[ObjectId], target_source: Option<&SurfaceCandidateSource>,
    mut prove_clear: impl FnMut(&crate::scene::SceneObject) -> bool,
) -> Result<bool> {
    visible_display_occludes_with_margin(scene,camera,viewport,cursor,target,tolerance,excluded,target_source,&mut prove_clear)
}

/// Whether a visible filled display surface lies strictly in front of a displayed point.
pub fn visible_display_occludes_before(
    scene: &Scene, camera: &Camera, viewport: Viewport, cursor: [f64; 2], target: Point3, clearance: f64,
) -> Result<bool> {
    if !clearance.is_finite() || clearance < 0.0 { return Err(Error::Invalid("occlusion clearance")); }
    visible_display_occludes_with_margin(scene, camera, viewport, cursor, target, -clearance, &[], None, &mut |_| false)
}

fn visible_display_occludes_with_margin(
    scene: &Scene, camera: &Camera, viewport: Viewport, cursor: [f64; 2], target: Point3,
    tolerance: f64, excluded: &[ObjectId], target_source: Option<&SurfaceCandidateSource>,
    prove_clear: &mut dyn FnMut(&crate::scene::SceneObject) -> bool,
) -> Result<bool> {
    visible_display_occludes_in_scene(scene, camera, viewport, cursor, target, tolerance, excluded, target_source,
        prove_clear, &mut |object| Ok(matches!(scene.entity_for(&scene.document, object)?.geometry,
            Geometry::Mesh(_) | Geometry::Hatch(_) | Geometry::Surface(_) | Geometry::Brep(_) | Geometry::NativeBrep(_))))
}

impl OcclusionSnapshot {
    /// Conservative display occlusion strictly before a source point, using
    /// retained display buffers without retaining the document.
    pub fn visible_display_occludes_before(&self, camera: &Camera, viewport: Viewport,
        cursor: [f64; 2], target: Point3, clearance: f64) -> Result<bool> {
        if !clearance.is_finite() || clearance < 0. { return Err(Error::Invalid("occlusion clearance")); }
        visible_display_occludes_in_scene(&self.0, camera, viewport, cursor, target, -clearance,
            &[], None, &mut |_| false, &mut |_| Ok(true))
    }

    /// Query captured display buffers without retaining a document. A fallback
    /// proof must itself use independently captured source data; returning
    /// false preserves the conservative occluder.
    pub fn visible_display_occludes_with_fallback_proof(
        &self, camera: &Camera, viewport: Viewport, cursor: [f64; 2], target: Point3,
        tolerance: f64, excluded: &[ObjectId], target_source: Option<&SurfaceCandidateSource>,
        mut prove_clear: impl FnMut(&crate::scene::SceneObject) -> bool,
    ) -> Result<bool> {
        visible_display_occludes_in_scene(&self.0, camera, viewport, cursor, target, tolerance, excluded,
            target_source, &mut prove_clear, &mut |_| Ok(true))
    }
}

fn visible_display_occludes_in_scene(
    scene: &impl PickScene, camera: &Camera, viewport: Viewport, cursor: [f64; 2], target: Point3,
    tolerance: f64, excluded: &[ObjectId], target_source: Option<&SurfaceCandidateSource>,
    prove_clear: &mut dyn FnMut(&crate::scene::SceneObject) -> bool,
    valid_surface: &mut dyn FnMut(&crate::scene::SceneObject) -> Result<bool>,
) -> Result<bool> {
    let ray = camera.ray(cursor, viewport)?;
    let target_distance = distance(ray.origin, target);
    let mut candidates = scene.spatial_index()?.ray_candidates(ray.origin, ray.direction);
    // Unusable bounds cannot establish a miss. Include them conservatively,
    // even if the index has already returned the same object.
    candidates.extend(scene.object_spheres()?.iter().enumerate().filter_map(|(index, (center, radius))| {
        (!radius.is_finite() || *radius <= 0. || center.iter().any(|value| !value.is_finite())).then_some(index)
    }));
    candidates.sort_unstable();
    candidates.dedup();
    for index in candidates {
        let object = &scene.objects()[index];
        if target_source.is_some_and(|source| object.root == source.source.object && object.path == source.path && object.leaf == source.leaf) {
            continue;
        }
        if !object.visible || excluded.contains(&object.root)
            || !valid_surface(object)? {
            continue;
        }
        let crate::scene::RenderBuffers::Mesh { vertices, triangles, .. } = object.buffers.as_ref() else { continue };
        let transform = object.display_transform()?;
        for (indices, _) in triangles {
            let a = transform.point(vertices[indices[0] as usize])?;
            let b = transform.point(vertices[indices[1] as usize])?;
            let c = transform.point(vertices[indices[2] as usize])?;
            if let Some((point, _)) = ray_triangle(ray, a, b, c)
                && camera.project(point, viewport)?.is_some()
                && distance(ray.origin, point) <= target_distance + tolerance {
                if object.buffers.is_trim_display_fallback() && prove_clear(object) { break; }
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn snap_order(left: &SnapCandidate, right: &SnapCandidate) -> std::cmp::Ordering {
    let surface_depth = |candidate: &SnapCandidate| {
        candidate.surface_source.as_ref()
            .map(|source| distance(source.camera.eye, candidate.world))
            .unwrap_or(f64::INFINITY)
    };
    left.kind.priority().cmp(&right.kind.priority())
        // Surface candidates are intersections at the same cursor. Their
        // projected distances can differ by floating-point rounding; use the
        // captured display depth before pixel distance or object identity.
        .then_with(|| {
            if left.kind == SnapKind::Surface && right.kind == SnapKind::Surface {
                surface_depth(left).total_cmp(&surface_depth(right))
            } else {
                std::cmp::Ordering::Equal
            }
        })
        .then(left.distance_px.total_cmp(&right.distance_px))
        .then(left.apparent.cmp(&right.apparent))
        .then(left.root.cmp(&right.root))
}

#[derive(Debug, Clone, Copy)]
struct Segment {
    root: ObjectId,
    object_index: usize,
    a: Point3,
    b: Point3,
    source_range: [f64; 2],
    source_weights: [f64; 2],
}

impl Segment {
    fn source_parameter(self, fraction: f64) -> f64 {
        let [start, end] = self.source_range;
        if fraction <= 0. { return start; }
        if fraction >= 1. { return end; }
        // Invert rational interpolation on a degree-one NURBS span.
        // For a Line the endpoints have equal weights.
        let [a, b] = self.source_weights;
        let first = (1. - fraction) * b;
        let last = fraction * a;
        start + (end - start) * (last / (first + last))
    }
}

#[derive(Debug, Clone, Copy)]
struct Ellipse {
    object_index: usize,
    center: Point3,
    x: Point3,
    y: Point3,
    radius: f64,
    angles: [f64; 2],
    source_range: [f64; 2],
}

pub fn candidates(
    scene: &Scene,
    camera: &Camera,
    viewport: Viewport,
    cursor: [f64; 2],
    settings: SnapSettings,
    from: Option<Point3>,
) -> Result<Vec<SnapCandidate>> {
    candidates_in_plane(scene, camera, viewport, cursor, settings, from, CPlane::world_xy())
}

pub fn candidates_in_plane(
    scene: &Scene,
    camera: &Camera,
    viewport: Viewport,
    cursor: [f64; 2],
    settings: SnapSettings,
    from: Option<Point3>,
    cplane: CPlane,
) -> Result<Vec<SnapCandidate>> {
    candidates_in_plane_impl(scene, camera, viewport, cursor, settings, from, cplane, false)
}

/// The caller must merge the separately evaluated NURBS, circular Near/Mid,
/// surface-boundary and intersection candidates before ranking or accepting a point.
pub fn candidates_in_plane_deferred(
    scene: &Scene, camera: &Camera, viewport: Viewport, cursor: [f64; 2],
    settings: SnapSettings, from: Option<Point3>, cplane: CPlane,
) -> Result<Vec<SnapCandidate>> {
    candidates_in_plane_impl(scene, camera, viewport, cursor, settings, from, cplane, true)
}

fn candidates_in_plane_impl(
    scene: &Scene, camera: &Camera, viewport: Viewport, cursor: [f64; 2],
    settings: SnapSettings, from: Option<Point3>, cplane: CPlane, defer_nurbs: bool,
) -> Result<Vec<SnapCandidate>> {
    if !settings.radius_px.is_finite() || settings.radius_px < 0. {
        return Err(Error::Invalid("snap radius"));
    }
    let mut out = Vec::new();
    let mut segments = Vec::new();
    let mut ellipses = Vec::new();
    let mut curve_work = 0usize;
    let mut surface_broad_phase = None;
    let tolerance = scene.document.effective_tolerance()?.absolute_mm;
    for (object_index, object) in scene.objects.iter().enumerate() {
        if !object.visible || !(object.selectable || object.reference_snappable) {
            continue;
        }
        // The fallback surface contains points that may be outside the trim.
        // Do not let such a candidate prevent refinement of valid objects behind it.
        if object.buffers.is_trim_display_fallback() {
            continue;
        }
        let transform = object.display_transform()?;
        let entity = scene.entity_for(&scene.document, object)?;
        if !defer_nurbs && let Geometry::Surface(surface) = &entity.geometry
            && !entity.extensions.contains_key(om9_core::model::TRIM_SURFACE_EXTENSION_SLOT)
            && ([SnapKind::End, SnapKind::Mid, SnapKind::Near, SnapKind::Quadrant]
                .into_iter().any(|kind| settings.is_effective(kind))
                || from.is_some() && (settings.is_effective(SnapKind::Tangent)
                    || settings.is_effective(SnapKind::Perpendicular))) {
            // An untrimmed NURBS surface owns its boundary isocurves. Evaluate
            // those source curves rather than snapping to the display mesh.
            // ponytail: move this bounded extraction to a revision-keyed worker/cache.
            use om9_core::surface_kernel::SurfaceIso;
            let start = out.len();
            let mut end_directions = settings.is_effective(SnapKind::End).then(EndDirections::default);
            for (iso, fixed_axis) in [(SurfaceIso::U, 1usize), (SurfaceIso::V, 0usize)] {
                for side in 0..if surface.periodic[fixed_axis] { 1 } else { 2 } {
                    let degree = surface.degree[fixed_axis].checked_add(1).ok_or(Error::Overflow("surface edge snap work"))?;
                    let cost = surface.size[0].checked_mul(surface.size[1])
                        .and_then(|value| value.checked_mul(degree))
                        .and_then(|value| value.checked_mul(degree))
                        .ok_or(Error::Overflow("surface edge snap work"))?;
                    curve_work = curve_work.checked_add(cost).ok_or(Error::Overflow("surface edge snap work"))?;
                    if curve_work > 1_000_000 { return Err(Error::Budget("surface edge snap work")); }
                    let boundary = om9_core::surface_kernel::isocurve(
                        surface, iso, surface.domain[fixed_axis][side], &scene.limits)?;
                    if let Some(&pole) = boundary.poles.first()
                        && boundary.poles.iter().all(|point| *point == pole) {
                        let point = transform.point(pole)?;
                        if settings.is_effective(SnapKind::End) {
                            push_curve_end(object.root, point, None, None, camera, viewport, cursor,
                                settings.radius_px, &mut out, &mut end_directions, |_| None)?;
                        }
                        if settings.is_effective(SnapKind::Near) {
                            push(SnapKind::Near, Some(object.root), point, None, None, camera, viewport,
                                cursor, settings.radius_px, &mut out)?;
                        }
                        continue;
                    }
                    let edge = Curve::Nurbs(boundary);
                    curve_snaps(&edge, [0., 1.], object.root, object_index, transform, camera, viewport, cursor,
                        settings, from, &scene.limits, cplane, tolerance, &mut out, &mut end_directions,
                        &mut segments, &mut ellipses, &mut curve_work, &|| false, false)?;
                }
            }
            if out.len() > start {
                if let Some(end_directions) = end_directions {
                    end_directions.resolve(&mut out[start..], &mut [], tolerance, &|| false)?;
                }
                let source = snap_source(scene, object);
                for candidate in &mut out[start..] { candidate.source = Some(source.clone()); }
            }
        }
        if matches!(entity.geometry, Geometry::Brep(_) | Geometry::NativeBrep(_) | Geometry::Surface(_)) {
            if !settings.is_effective(SnapKind::Surface) { continue; }
            if surface_broad_phase.is_none() {
                let ray = camera.visible_ray(cursor, viewport)?;
                let mut included = vec![false; scene.objects.len()];
                for index in scene.spatial_index()?.ray_candidates(ray.origin, ray.direction) { included[index] = true; }
                // Unknown or degenerate bounds must never cause a false negative.
                for (index, (_, radius)) in scene.object_spheres()?.iter().enumerate() {
                    if !radius.is_finite() || *radius <= 0. { included[index] = true; }
                }
                surface_broad_phase = Some((ray, included));
            }
            if !surface_broad_phase.as_ref().expect("initialized above").1[object_index] { continue; }
        }
        let start = out.len();
        let mut end_directions = (settings.is_effective(SnapKind::End)
            && matches!(entity.geometry, Geometry::Curve(Curve::Polycurve { .. })))
            .then(EndDirections::default);
        match &entity.geometry {
            Geometry::Point(point) => {
                let world = transform.point(*point)?;
                push(
                    SnapKind::Point,
                    Some(object.root),
                    world,
                    None,
                    None,
                    camera,
                    viewport,
                    cursor,
                    settings.radius_px,
                    &mut out,
                )?;
            }
            Geometry::Curve(curve) => curve_snaps(
                curve,
                [0., 1.],
                object.root,
                object_index,
                transform,
                camera,
                viewport,
                cursor,
                settings,
                from,
                &scene.limits,
                cplane,
                tolerance,
                &mut out,
                &mut end_directions,
                &mut segments,
                &mut ellipses,
                &mut curve_work,
                &|| false,
                defer_nurbs,
            )?,
            Geometry::Brep(_) => {
                let display = object.pick_brep_display(entity)?.ok_or(om9_core::Error::Stale("scene BRep snap display"))?;
                let ray = surface_broad_phase.as_ref().expect("initialized for surfaces").0;
                let mut best = std::collections::BTreeMap::<u32, (f64, Point3, Point3, Option<uuid::Uuid>, [Point3; 3], usize)>::new();
                for (triangle_index, triangle) in display.triangles.iter().enumerate() {
                    let a = transform.point(display.points[triangle[0] as usize])?;
                    let b = transform.point(display.points[triangle[1] as usize])?;
                    let c = transform.point(display.points[triangle[2] as usize])?;
                    if let Some((point, normal)) = ray_triangle(ray, a, b, c) {
                        let distance = distance(camera.eye, point);
                        if best.get(&triangle[3]).is_none_or(|(old, _, _, _, _, _)| distance < *old) {
                            best.insert(triangle[3], (distance, point, normal, display.topology.faces.get(triangle[3] as usize).copied(),
                                [display.points[triangle[0] as usize], display.points[triangle[1] as usize], display.points[triangle[2] as usize]], triangle_index));
                        }
                    }
                }
                for (_, (_, point, normal, face, witness, triangle_index)) in best {
                    let previous = out.len();
                    push(
                        SnapKind::Surface,
                        Some(object.root),
                        point,
                        Some(normal),
                        None,
                        camera,
                        viewport,
                        cursor,
                        settings.radius_px,
                        &mut out,
                    )?;
                    if out.len() > previous {
                        out.last_mut().expect("surface candidate was appended").surface_source = Some(SurfaceCandidateSource {
                            path: object.path.clone(), leaf: object.leaf, source: object.source, scene_epoch: scene.epoch, scene_publication: scene.publication_id(),
                            leaf_generation: object.leaf_generation, leaf_geometry_revision: object.leaf_geometry_revision,
                            brep_display_fingerprint: object.brep_display_fingerprint.clone(),
                            placement: transform, camera: camera.clone(), viewport, cursor, face, face_witness: Some((witness, display.deflection_mm)), face_witness_triangle: Some(triangle_index), native_face_count: None,
                        });
                    }
                }
            }
            Geometry::NativeBrep(_) => {
                let ray = surface_broad_phase.as_ref().expect("initialized for surfaces").0;
                let mut best: Option<(f64, Point3, Point3, [Point3; 3], usize)> = None;
                // Use the same triangulation as the displayed image, including
                // the fan diagonal chosen for quad faces.
                let crate::scene::RenderBuffers::Mesh { vertices, triangles, .. } = object.buffers.as_ref() else { continue };
                for (triangle_index, (triangle, _)) in triangles.iter().enumerate() {
                    let a = transform.point(vertices[triangle[0] as usize])?;
                    let b = transform.point(vertices[triangle[1] as usize])?;
                    let c = transform.point(vertices[triangle[2] as usize])?;
                    if let Some((point, normal)) = ray_triangle(ray, a, b, c) {
                        let distance = distance(camera.eye, point);
                        if best.is_none_or(|(old, _, _, _, _)| distance < old) {
                            best = Some((distance, point, normal, [vertices[triangle[0] as usize], vertices[triangle[1] as usize], vertices[triangle[2] as usize]], triangle_index));
                        }
                    }
                }
                if let Some((_, point, normal, witness, triangle_index)) = best {
                    let previous = out.len();
                    push(
                        SnapKind::Surface,
                        Some(object.root),
                        point,
                        Some(normal),
                        None,
                        camera,
                        viewport,
                        cursor,
                        settings.radius_px,
                        &mut out,
                    )?;
                    if out.len() > previous {
                        out.last_mut().expect("surface candidate was appended").surface_source = Some(SurfaceCandidateSource {
                            path: object.path.clone(), leaf: object.leaf, source: object.source, scene_epoch: scene.epoch, scene_publication: scene.publication_id(),
                            leaf_generation: object.leaf_generation, leaf_geometry_revision: object.leaf_geometry_revision,
                            brep_display_fingerprint: object.brep_display_fingerprint.clone(),
                            placement: transform, camera: camera.clone(), viewport, cursor, face: None, face_witness: Some((witness, 0.)), face_witness_triangle: Some(triangle_index), native_face_count: None,
                        });
                    }
                }
            }
            Geometry::Surface(_) => {
                let ray = surface_broad_phase.as_ref().expect("initialized for surfaces").0;
                let crate::scene::RenderBuffers::Mesh { vertices, triangles, .. } = object.buffers.as_ref() else { continue };
                let mut best: Option<(f64, Point3, Point3)> = None;
                for (indices, _) in triangles {
                    let a = transform.point(vertices[indices[0] as usize])?;
                    let b = transform.point(vertices[indices[1] as usize])?;
                    let c = transform.point(vertices[indices[2] as usize])?;
                    if let Some((point, normal)) = ray_triangle(ray, a, b, c) {
                        let depth = distance(ray.origin, point);
                        if best.is_none_or(|(old, _, _)| depth < old) { best = Some((depth, point, normal)); }
                    }
                }
                if let Some((_, point, normal)) = best {
                    let previous = out.len();
                    push(SnapKind::Surface, Some(object.root), point, Some(normal), None,
                        camera, viewport, cursor, settings.radius_px, &mut out)?;
                    if out.len() > previous {
                        out.last_mut().expect("surface candidate was appended").surface_source = Some(SurfaceCandidateSource {
                            path: object.path.clone(), leaf: object.leaf, source: object.source, scene_epoch: scene.epoch, scene_publication: scene.publication_id(),
                            leaf_generation: object.leaf_generation, leaf_geometry_revision: object.leaf_geometry_revision,
                            brep_display_fingerprint: object.brep_display_fingerprint.clone(),
                            placement: transform, camera: camera.clone(), viewport, cursor, face: None, face_witness: None, face_witness_triangle: None, native_face_count: None,
                        });
                    }
                }
            }
            _ => {}
        }
        if out.len() > start {
            if let Some(end_directions) = end_directions {
                if defer_nurbs && matches!(&entity.geometry, Geometry::Curve(curve) if contains_nurbs(curve)) {
                    // The worker has every segment. A partial result cannot attest a join direction.
                    for candidate in &mut out[start..] { candidate.tracking_tangent = None; }
                } else {
                    end_directions.resolve(&mut out[start..], &mut [], tolerance, &|| false)?;
                }
            }
            let source = snap_source(scene, object);
            for candidate in &mut out[start..] { candidate.source = Some(source.clone()); }
        }
    }
    if settings.is_effective(SnapKind::Intersection) {
    // ponytail: bound synchronous pair work; move exact queries to a revision-keyed worker for larger scenes.
    let pair_count = (segments.len().saturating_mul(segments.len().saturating_sub(1)) / 2)
        .saturating_add(segments.len().saturating_mul(ellipses.len()))
        .saturating_add(ellipses.len().saturating_mul(ellipses.len().saturating_sub(1)) / 2);
    if pair_count > 100_000 { return Err(om9_core::Error::Budget("intersection snap pairs")); }
    let mut accepted = None;
    if settings.apparent_intersections {
        segments.sort_by_key(|segment| std::cmp::Reverse(
            scene.document.entities.get(&segment.root).and_then(|entity| entity.creation_order),
        ));
    }
    let world_line = |segment: &Segment| Curve::Line {
        start: segment.a, end: segment.b, domain: [0., 1.],
    };
    for (index, left) in segments.iter().enumerate() {
        let first = world_line(left);
        for right in &segments[index + 1..] {
            // Distinct members can share both a root instance and a leaf definition.
            if left.object_index == right.object_index {
                continue;
            }
            let hits = match om9_core::curve_surface_ops::curve_curve_intersections(
                &first, &world_line(right), tolerance, &scene.limits,
            ) {
                Err(Error::Unsupported("overlapping line segments")) => continue,
                result => result?,
            };
            let before_pair = out.len();
            for hit in hits {
                let previous = out.len();
                push(SnapKind::Intersection, Some(left.root), hit.point, None,
                    Some(left.source_parameter(hit.first_fraction)), camera, viewport, cursor, settings.radius_px, &mut out)?;
                if out.len() > previous {
                    let candidate = out.last_mut().expect("intersection candidate appended");
                    candidate.source = Some(snap_source(scene, &scene.objects[left.object_index]));
                    candidate.second_source = Some(snap_source(scene, &scene.objects[right.object_index]));
                }
            }
            if settings.apparent_intersections && accepted.is_none() && out.len() == before_pair
                && let Some(hit) = apparent_segment_intersection_with_parameter_map(
                    ([left.a, left.b], &snap_source(scene, &scene.objects[left.object_index])),
                    ([right.a, right.b], &snap_source(scene, &scene.objects[right.object_index])),
                    camera, viewport, cursor, settings.radius_px, tolerance,
                    &|source, fraction| [left, right][source].source_parameter(fraction),
                )?
            {
                out.push(hit);
            }
            if settings.apparent_intersections {
                for hit in out.drain(before_pair..) {
                    if accepted.as_ref().is_none_or(|best| intersection_pick_is_closer(&hit, best, cursor)) {
                        accepted = Some(hit);
                    }
                }
            }
        }
    }
    out.extend(accepted);
    for segment in &segments {
        for ellipse in &ellipses {
            for point in segment_ellipse_intersections(*segment, *ellipse, tolerance) {
                let previous = out.len();
                let fraction = closest_on_segment(point, segment.a, segment.b).1;
                let parameter = segment.source_parameter(fraction);
                push(SnapKind::Intersection, Some(segment.root), point, None, Some(parameter), camera,
                    viewport, cursor, settings.radius_px, &mut out)?;
                if out.len() > previous {
                    let candidate = out.last_mut().expect("intersection candidate appended");
                    candidate.source = Some(snap_source(scene, &scene.objects[segment.object_index]));
                    candidate.second_source = Some(snap_source(scene, &scene.objects[ellipse.object_index]));
                }
            }
        }
    }
    // Reuse the source solver's analytic circle/circle path only when both
    // transformed curves remain circles in parallel planes. Other ellipse and
    // non-coplanar pairs need the general asynchronous curve solver.
    for (index, left) in ellipses.iter().enumerate() {
        let Some(first) = ellipse_as_circle(*left) else { continue };
        for right in &ellipses[index + 1..] {
            let Some(second) = ellipse_as_circle(*right) else { continue };
            let Curve::Circle { x: x1, y: y1, center: c1, .. } = &first else { unreachable!() };
            let Curve::Circle { x: x2, y: y2, center: c2, .. } = &second else { unreachable!() };
            let n1 = cross(*x1, *y1);
            let n2 = cross(*x2, *y2);
            if dot(cross(n1, n2), cross(n1, n2)).sqrt() > 8. * f64::EPSILON
                || dot(sub(*c2, *c1), n1).abs() > tolerance { continue; }
            for hit in om9_core::curve_surface_ops::curve_curve_intersections(&first, &second, tolerance, &scene.limits)? {
                let previous = out.len();
                let [start, end] = left.source_range;
                let parameter = start + (end - start) * hit.first_fraction;
                push(SnapKind::Intersection, Some(scene.objects[left.object_index].root), hit.point, None,
                    Some(parameter), camera, viewport, cursor, settings.radius_px, &mut out)?;
                if out.len() > previous {
                    let candidate = out.last_mut().expect("intersection candidate appended");
                    candidate.source = Some(snap_source(scene, &scene.objects[left.object_index]));
                    candidate.second_source = Some(snap_source(scene, &scene.objects[right.object_index]));
                }
            }
        }
    }
    }
    out.retain(|candidate| settings.is_effective(candidate.kind));
    out.sort_by(snap_order);
    out.dedup_by(|left, right| {
        left.kind == right.kind
            && left.root == right.root
            && left.source == right.source && left.second_source == right.second_source
            && left.parameter == right.parameter
            && (left.kind != SnapKind::Surface || left.surface_source == right.surface_source)
            && distance(left.world, right.world) <= 1.0e-9
    });
    Ok(out)
}

fn ellipse_as_circle(ellipse: Ellipse) -> Option<Curve> {
    let xx = dot(ellipse.x, ellipse.x);
    let yy = dot(ellipse.y, ellipse.y);
    let xy = dot(ellipse.x, ellipse.y);
    if !xx.is_finite() || !yy.is_finite() || !xy.is_finite() || xx <= f64::MIN_POSITIVE || yy <= f64::MIN_POSITIVE
        || (xx - yy).abs() > 8. * f64::EPSILON * xx.max(yy)
        || (xy / xx.sqrt() / yy.sqrt()).abs() > 8. * f64::EPSILON { return None; }
    let (sx, sy) = (xx.sqrt(), yy.sqrt());
    Some(Curve::Circle { center: ellipse.center, x: mul(ellipse.x, 1. / sx), y: mul(ellipse.y, 1. / sy),
        radius: ellipse.radius * sx, domain: ellipse.angles, angle_domain: Some(ellipse.angles) })
}

#[allow(clippy::too_many_arguments)]
fn curve_snaps(
    curve: &Curve,
    source_range: [f64; 2],
    root: ObjectId,
    object_index: usize,
    transform: om9_core::geometry::Transform,
    camera: &Camera,
    viewport: Viewport,
    cursor: [f64; 2],
    settings: SnapSettings,
    from: Option<Point3>,
    limits: &om9_core::budget::Limits,
    cplane: CPlane,
    tolerance: f64,
    out: &mut Vec<SnapCandidate>,
    end_directions: &mut Option<EndDirections>,
    segments: &mut Vec<Segment>,
    ellipses: &mut Vec<Ellipse>,
    curve_work: &mut usize,
    canceled: &dyn Fn() -> bool,
    defer_nurbs: bool,
) -> Result<()> {
    if canceled() { return Err(Error::Canceled); }
    match curve {
        Curve::Line { start, end, .. } => {
            let a = transform.point(*start)?;
            let b = transform.point(*end)?;
            if !defer_nurbs && settings.is_effective(SnapKind::Intersection) {
                segments.push(Segment { root, object_index, a, b, source_range, source_weights: [1.; 2] });
            }
            if settings.is_effective(SnapKind::End) {
                let tangent = unit_direction(sub(b, a));
                for (point, fraction) in [(a, 0.), (b, 1.)] {
                    push_curve_end(root, point, None, Some(fraction), camera, viewport, cursor,
                        settings.radius_px, out, end_directions, |_| tangent)?;
                }
            }
            push(
                SnapKind::Mid,
                Some(root),
                midpoint(a, b),
                None,
                Some(0.5),
                camera,
                viewport,
                cursor,
                settings.radius_px,
                out,
            )?;
            if settings.is_effective(SnapKind::Near)
                && let Some((nearest, t)) = closest_segment_on_screen(camera, viewport, cursor, a, b)? {
                push(
                    SnapKind::Near,
                    Some(root),
                    nearest,
                    None,
                    Some(t),
                    camera,
                    viewport,
                    cursor,
                    settings.radius_px,
                    out,
                )?;
            }
            if settings.is_effective(SnapKind::Perpendicular) && let Some(from) = from {
                let direction = sub(b, a);
                let length_squared = dot(direction, direction);
                let parameter = dot(sub(from, a), direction) / length_squared;
                // A nearest endpoint is a valid Near snap, but Perp needs
                // the actual foot to lie on the bounded source segment.
                if length_squared.is_finite() && (0.0..=1.0).contains(&parameter) {
                    push(
                        SnapKind::Perpendicular,
                        Some(root),
                        add(a, mul(direction, parameter)),
                        None,
                        Some(parameter),
                        camera,
                        viewport,
                        cursor,
                        settings.radius_px,
                        out,
                    )?;
                }
            }
        }
        Curve::Circle {
            center,
            x,
            y,
            radius,
            domain,
            angle_domain,
        } => {
            let center = transform.point(*center)?;
            let x = transform_vector(transform, *x)?;
            let y = transform_vector(transform, *y)?;
            if !defer_nurbs && settings.is_effective(SnapKind::Intersection) {
                ellipses.push(Ellipse { object_index, center, x, y, radius: *radius, angles: angle_domain.unwrap_or(*domain), source_range });
            }
            circle_snaps(
                root, center, x, y, *radius, angle_domain.unwrap_or(*domain), camera, viewport, cursor, settings, from, cplane, tolerance, out, end_directions, curve_work, canceled, defer_nurbs,
            )?;
        }
        Curve::Polycurve { segments: parts, reversed } => {
            let ranges = om9_core::curve_ops::polycurve_segment_ranges(source_range, parts.len(), *reversed);
            for (part, range) in parts.iter().zip(ranges) {
                curve_snaps(
                    part, range, root, object_index, transform, camera, viewport, cursor, settings, from, limits, cplane, tolerance, out, end_directions, segments, ellipses, curve_work, canceled, defer_nurbs,
                )?;
            }
        }
        Curve::Nurbs(nurbs) => {
            if defer_nurbs { return Ok(()); }
            // Degree-one, continuous NURBS spans are exact straight segments.
            // Keep the source-aware solver below for curved and other snap kinds.
            if nurbs.degree == 1 && !nurbs.periodic && nurbs.seam.is_none()
                && nurbs.weights.iter().all(|weight| *weight > 0.)
                && nurbs.knots.windows(2).all(|pair| pair[0] != pair[1]
                    || pair[0] <= nurbs.domain[0] || pair[0] >= nurbs.domain[1]) {
                let evaluator = om9_core::evaluation::CurveEvaluator::capture(curve, limits)?;
                for (index, knots) in nurbs.knots.windows(2).enumerate().take(nurbs.poles.len()).skip(1) {
                    let [domain_start, domain_end] = nurbs.domain;
                    let a = knots[0].max(domain_start);
                    let b = knots[1].min(domain_end);
                    if b <= a { continue; }
                    *curve_work = curve_work.checked_add(2).ok_or(Error::Overflow("NURBS snap work"))?;
                    if *curve_work > 1_000_000 { return Err(Error::Budget("NURBS snap work")); }
                    let source_range = [a, b].map(|parameter| {
                        let offset = if nurbs.reversed { domain_end - parameter } else { parameter - domain_start };
                        let fraction = offset / (domain_end - domain_start);
                        source_range[0] + (source_range[1] - source_range[0]) * fraction
                    });
                    let point_a = evaluator.evaluate(om9_core::evaluation::Parameter::Native(a), 0, transform)?[0];
                    let point_b = evaluator.evaluate(om9_core::evaluation::Parameter::Native(b), 0, transform)?[0];
                    let first = out.len();
                    let first_segment = segments.len();
                    curve_snaps(&Curve::Line { start: point_a, end: point_b, domain: [0., 1.] }, source_range, root, object_index,
                        Transform::identity(), camera, viewport, cursor, settings, from, limits, cplane,
                        tolerance, out, end_directions, segments, ellipses, curve_work, canceled, false)?;
                    if let Some(segment) = segments.get_mut(first_segment) {
                        let first = nurbs.weights[index - 1];
                        let last = nurbs.weights[index];
                        let scale = first.max(last);
                        let (first, last) = (first / scale, last / scale);
                        // Clipping to the active domain changes the span endpoint weights.
                        segment.source_weights = [a, b].map(|parameter| {
                            let fraction = (parameter - knots[0]) / (knots[1] - knots[0]);
                            (1. - fraction) * first + fraction * last
                        });
                    }
                    // A linear segment fraction is not generally the rational NURBS parameter.
                    for candidate in &mut out[first..] { candidate.parameter = None; }
                }
            }
            if settings.is_effective(SnapKind::End) || settings.is_effective(SnapKind::Near) || settings.is_effective(SnapKind::Mid) || settings.is_effective(SnapKind::Quadrant)
                || from.is_some() && (settings.is_effective(SnapKind::Tangent) || settings.is_effective(SnapKind::Perpendicular)) {
                // ponytail: bound synchronous source evaluation; use a revision-keyed worker/cache for larger scenes.
                let degree = nurbs.degree.checked_add(1).ok_or(Error::Overflow("NURBS snap work"))?;
                let evaluations = if settings.is_effective(SnapKind::Near) { 128usize } else { 2 };
                let work = degree.checked_mul(degree).and_then(|value| value.checked_mul(evaluations))
                    .and_then(|value| value.checked_add(nurbs.poles.len())).ok_or(Error::Overflow("NURBS snap work"))?;
                *curve_work = curve_work.checked_add(work).ok_or(Error::Overflow("NURBS snap work"))?;
                if *curve_work > 1_000_000 { return Err(Error::Budget("NURBS snap work")); }
                let evaluator = om9_core::evaluation::CurveEvaluator::capture(curve, limits)?;
                if settings.is_effective(SnapKind::End) {
                    for fraction in [0., 1.] {
                        let parameter = om9_core::evaluation::Parameter::Fraction(fraction);
                        let point = evaluator.evaluate(parameter, 0, transform)?[0];
                        push_curve_end(root, point, None, Some(fraction), camera, viewport,
                            cursor, settings.radius_px, out, end_directions, |directions| {
                                let work = directions.map_or(&mut *curve_work, |capture| &mut capture.derivative_work);
                                *work = work.checked_add(degree * degree).filter(|&next| next <= 1_000_000)?;
                                let jet = evaluator.evaluate(parameter, 1, transform).ok()?;
                                unit_direction(*jet.get(1)?)
                            })?;
                    }
                }
                if settings.is_effective(SnapKind::Near)
                    && let Some((point, fraction)) = nurbs_near_screen(curve, &evaluator, transform, camera, viewport, cursor,
                        settings.radius_px, degree, limits, curve_work, canceled)? {
                    push(SnapKind::Near, Some(root), point, None, Some(fraction), camera, viewport,
                        cursor, settings.radius_px, out)?;
                }
                if settings.is_effective(SnapKind::Mid) {
                    let spans = nurbs.knots.windows(2).filter(|pair| pair[1] > pair[0]
                        && pair[1] > nurbs.domain[0] && pair[0] < nurbs.domain[1]).count();
                    let minimum_samples = spans.checked_mul(degree).and_then(|value| value.checked_mul(4))
                        .ok_or(Error::Overflow("NURBS midpoint samples"))?;
                    let (point, fraction) = nurbs_midpoint(curve, &evaluator, transform, degree, minimum_samples, tolerance, limits, curve_work, canceled)?;
                    push(SnapKind::Mid, Some(root), point, None, Some(fraction), camera, viewport,
                        cursor, settings.radius_px, out)?;
                }
                if settings.is_effective(SnapKind::Quadrant) {
                    for (point, fraction) in nurbs_quadrants(curve, &evaluator, nurbs, transform, cplane, degree, tolerance, limits, curve_work, canceled)? {
                        push(SnapKind::Quadrant, Some(root), point, None, Some(fraction), camera, viewport,
                            cursor, settings.radius_px, out)?;
                    }
                }
                if let Some(from) = from
                    && (settings.is_effective(SnapKind::Tangent) || settings.is_effective(SnapKind::Perpendicular)) {
                    for (kind, point, fraction) in nurbs_directional_hulls(curve, &evaluator, nurbs, transform, from,
                        settings, degree, tolerance, limits, curve_work, canceled)? {
                        push(kind, Some(root), point, None, Some(fraction), camera, viewport,
                            cursor, settings.radius_px, out)?;
                    }
                }
            }
        }
    }
    Ok(())
}

/// Subdivide Bézier spans and bound the homogeneous tangent equations. This
/// retains intervals whose bounds contain zero, including even-multiplicity roots.
#[allow(clippy::too_many_arguments)]
fn nurbs_directional_hulls(curve: &Curve, evaluator: &om9_core::evaluation::CurveEvaluator<'_>,
    nurbs: &om9_core::geometry::NurbsCurve, transform: Transform, from: Point3, settings: SnapSettings,
    degree: usize, tolerance: f64, limits: &om9_core::budget::Limits, work: &mut usize,
    canceled: &dyn Fn() -> bool) -> Result<Vec<(SnapKind, Point3, f64)>> {
    type Range = [f64; 2];
    fn widen(low: f64, high: f64) -> Range {
        let margin = 64. * f64::EPSILON * low.abs().max(high.abs()).max(1.);
        [low - margin, high + margin]
    }
    fn add(a: Range, b: Range) -> Range { widen(a[0] + b[0], a[1] + b[1]) }
    fn subtract(a: Range, b: Range) -> Range { widen(a[0] - b[1], a[1] - b[0]) }
    fn multiply(a: Range, b: Range) -> Range {
        let values = [a[0] * b[0], a[0] * b[1], a[1] * b[0], a[1] * b[1]];
        widen(values.iter().copied().fold(f64::INFINITY, f64::min),
            values.iter().copied().fold(f64::NEG_INFINITY, f64::max))
    }
    fn contains_zero(range: Range) -> bool { range[0] <= 0. && range[1] >= 0. }
    let cost = degree.checked_mul(degree).and_then(|n| n.checked_mul(nurbs.poles.len().saturating_mul(2).saturating_add(50)))
        .ok_or(Error::Overflow("NURBS directional work"))?;
    let breaks = nurbs_breakpoints(nurbs, 8, limits)?;
    let mut stack = Vec::new();
    stack.try_reserve_exact(breaks.len()).map_err(|_| Error::Budget("NURBS directional intervals"))?;
    for pair in breaks.windows(2) { if pair[1] > pair[0] { stack.push((pair[0], pair[1])); } }
    let mut results: Vec<(SnapKind, Point3, f64)> = Vec::new();
    while let Some((low, high)) = stack.pop() {
        if canceled() { return Err(Error::Canceled); }
        *work = work.checked_add(cost).ok_or(Error::Overflow("NURBS directional work"))?;
        if *work > 1_000_000 { return Err(Error::Budget("NURBS directional work")); }
        let piece = om9_core::curve_ops::trim_curve_by_parameter(curve, low, high, limits)?;
        let local_mid = om9_core::evaluation::CurveEvaluator::capture(&piece, limits)?
            .evaluate(om9_core::evaluation::Parameter::Fraction(0.5), 0, transform)?[0];
        let source_mid = evaluator.evaluate(om9_core::evaluation::Parameter::Fraction((low + high) * 0.5), 0, transform)?[0];
        if distance(local_mid, source_mid) > tolerance * 0.0625 {
            return Err(Error::Unsupported("NURBS directional subcurve parameterization"));
        }
        let Curve::Nurbs(piece) = &piece else { return Err(Error::Unsupported("NURBS directional subcurve")) };
        let p = piece.degree;
        if p == 0 || piece.poles.len() != p + 1 || piece.weights.len() != p + 1
            || piece.knots[..=p].iter().any(|knot| *knot != piece.domain[0])
            || piece.knots[piece.knots.len() - p - 1..].iter().any(|knot| *knot != piece.domain[1]) {
            return Err(Error::Unsupported("NURBS directional Bézier span"));
        }
        let weight_scale = piece.weights.iter().copied().fold(0f64, f64::max);
        if !weight_scale.is_finite() || weight_scale <= 0. || piece.weights.iter().any(|w| !w.is_finite() || *w <= 0.) {
            return Err(Error::Unsupported("NURBS directional weights"));
        }
        let mut homogeneous = Vec::new();
        homogeneous.try_reserve_exact(p + 1).map_err(|_| Error::Budget("NURBS directional controls"))?;
        let mut low_point = [f64::INFINITY; 3];
        let mut high_point = [f64::NEG_INFINITY; 3];
        for (&pole, &weight) in piece.poles.iter().zip(&piece.weights) {
            let point = transform.point(pole)?;
            let w = weight / weight_scale;
            let mut h = [0.; 4];
            for axis in 0..3 {
                low_point[axis] = low_point[axis].min(point[axis]); high_point[axis] = high_point[axis].max(point[axis]);
                h[axis] = (point[axis] - from[axis]) * w;
            }
            h[3] = w;
            if h.iter().any(|value| !value.is_finite()) { return Err(Error::Invalid("NURBS directional controls")); }
            homogeneous.push(h);
        }
        let mut h = [[f64::INFINITY, f64::NEG_INFINITY]; 4];
        let mut dh = [[f64::INFINITY, f64::NEG_INFINITY]; 4];
        for control in &homogeneous { for axis in 0..4 {
            h[axis][0] = h[axis][0].min(control[axis]); h[axis][1] = h[axis][1].max(control[axis]);
        }}
        for pair in homogeneous.windows(2) { for axis in 0..4 {
            let value = p as f64 * (pair[1][axis] - pair[0][axis]);
            dh[axis][0] = dh[axis][0].min(value); dh[axis][1] = dh[axis][1].max(value);
        }}
        for axis in 0..4 { h[axis] = widen(h[axis][0], h[axis][1]); dh[axis] = widen(dh[axis][0], dh[axis][1]); }
        let tangent = std::array::from_fn::<_, 3, _>(|axis|
            subtract(multiply(dh[axis], h[3]), multiply(h[axis], dh[3])));
        let mut perpendicular = [0., 0.];
        for axis in 0..3 { perpendicular = add(perpendicular, multiply(h[axis], tangent[axis])); }
        let cross_bounds = [
            subtract(multiply(h[1], tangent[2]), multiply(h[2], tangent[1])),
            subtract(multiply(h[2], tangent[0]), multiply(h[0], tangent[2])),
            subtract(multiply(h[0], tangent[1]), multiply(h[1], tangent[0])),
        ];
        if perpendicular.iter().chain(cross_bounds.iter().flatten()).any(|value| !value.is_finite()) {
            return Err(Error::Unsupported("NURBS directional interval precision"));
        }
        let possible_perp = settings.is_effective(SnapKind::Perpendicular) && contains_zero(perpendicular);
        let possible_tangent = settings.is_effective(SnapKind::Tangent) && cross_bounds.into_iter().all(contains_zero);
        if !possible_perp && !possible_tangent { continue; }
        let diameter = (high_point[0] - low_point[0]).hypot(high_point[1] - low_point[1]).hypot(high_point[2] - low_point[2]);
        let middle = (low + high) * 0.5;
        if middle == low || middle == high { return Err(Error::Unsupported("NURBS directional parameter precision")); }
        if diameter.is_finite() && diameter <= tolerance * 0.25 {
            let values = evaluator.evaluate(om9_core::evaluation::Parameter::Fraction(middle), 1, transform)?;
            let (point, tangent) = (values[0], values[1]);
            let delta = sub(point, from);
            let length = tangent[0].hypot(tangent[1]).hypot(tangent[2]);
            let from_distance = delta[0].hypot(delta[1]).hypot(delta[2]);
            if length > f64::EPSILON && length.is_finite() && from_distance > tolerance && from_distance.is_finite() {
                let cross = cross(delta, tangent);
                let perp_error = dot(delta, tangent).abs() / length;
                let tangent_error = cross[0].hypot(cross[1]).hypot(cross[2]) / length;
                let mut unresolved = false;
                for (kind, possible, error) in [(SnapKind::Perpendicular, possible_perp, perp_error),
                    (SnapKind::Tangent, possible_tangent, tangent_error)] {
                    if possible && error.is_finite() && error <= tolerance * 0.5 {
                        if !results.iter().any(|(old_kind, old_point, _)| *old_kind == kind && distance(*old_point, point) <= tolerance) {
                            if results.len() >= 256 || results.len() as u64 >= limits.max_outputs { return Err(Error::Budget("NURBS directional candidates")); }
                            results.try_reserve(1).map_err(|_| Error::Budget("NURBS directional candidates"))?;
                            results.push((kind, point, middle));
                        }
                    } else if possible { unresolved = true; }
                }
                if !unresolved { continue; }
            }
        }
        if stack.len() + 2 > 2048 || ((stack.len() + 2) as u64).saturating_mul(16) > limits.max_bytes {
            return Err(Error::Budget("NURBS directional intervals"));
        }
        stack.try_reserve(2).map_err(|_| Error::Budget("NURBS directional intervals"))?;
        stack.push((low, middle)); stack.push((middle, high));
    }
    Ok(results)
}

fn nurbs_breakpoints(nurbs: &om9_core::geometry::NurbsCurve, samples: usize,
    limits: &om9_core::budget::Limits) -> Result<Vec<f64>> {
    const MAX_BREAKS: usize = 2048;
    let mut breaks = Vec::new();
    let capacity = samples.checked_add(1).and_then(|n| n.checked_add(nurbs.knots.len()))
        .ok_or(Error::Overflow("NURBS snap breaks"))?;
    if capacity > MAX_BREAKS || capacity as u64 > limits.max_points
        || (capacity as u64).saturating_mul(std::mem::size_of::<f64>() as u64) > limits.max_bytes {
        return Err(Error::Budget("NURBS snap breaks"));
    }
    breaks.try_reserve_exact(capacity).map_err(|_| Error::Budget("NURBS snap breaks"))?;
    breaks.extend((0..=samples).map(|i| i as f64 / samples as f64));
    let span = nurbs.domain[1] - nurbs.domain[0];
    let origin = nurbs.seam.map(|s| (s - nurbs.domain[0]) / span);
    for &knot in &nurbs.knots {
        if knot <= nurbs.domain[0] || knot >= nurbs.domain[1] { continue; }
        let raw = (knot - nurbs.domain[0]) / span;
        let f = match origin {
            Some(o) => if nurbs.reversed { (o - raw).rem_euclid(1.) } else { (raw - o).rem_euclid(1.) },
            None => if nurbs.reversed { 1. - raw } else { raw },
        };
        breaks.push(f);
    }
    breaks.sort_by(f64::total_cmp);
    breaks.dedup_by(|a, b| (*a - *b).abs() < 1e-14);
    Ok(breaks)
}

/// Bound CPlane extrema by subdividing positive-weight source control hulls.
/// Floating-point margins and source trimming still require numerical validation.
fn nurbs_quadrants(curve: &Curve, evaluator: &om9_core::evaluation::CurveEvaluator<'_>,
    nurbs: &om9_core::geometry::NurbsCurve, transform: Transform, cplane: CPlane,
    degree: usize, tolerance: f64, limits: &om9_core::budget::Limits, work: &mut usize, canceled: &dyn Fn() -> bool) -> Result<Vec<(Point3, f64)>> {
    use om9_core::evaluation::Parameter;
    #[derive(Clone, Copy)]
    struct Interval { from: f64, to: f64, bounds: [f64; 4] }
    fn consider(extrema: &mut [(f64, Point3, f64); 4], point: Point3, fraction: f64, cplane: CPlane) {
        for (axis_index, axis) in [cplane.x, cplane.y].into_iter().enumerate() {
            let value = dot(sub(point, cplane.origin), axis);
            for (offset, is_min) in [(0, true), (1, false)] {
                let slot = &mut extrema[axis_index * 2 + offset];
                if if is_min { value < slot.0 } else { value > slot.0 } {
                    *slot = (value, point, fraction);
                }
            }
        }
    }
    let cost = degree.checked_mul(degree).and_then(|n| n.checked_mul(nurbs.poles.len().saturating_mul(2).saturating_add(50)))
        .ok_or(Error::Overflow("NURBS quadrant work"))?;
    let mut bounds = |from: f64, to: f64| -> Result<[f64; 4]> {
        if canceled() { return Err(Error::Canceled); }
        *work = work.checked_add(cost).ok_or(Error::Overflow("NURBS quadrant work"))?;
        if *work > 1_000_000 { return Err(Error::Budget("NURBS quadrant work")); }
        let owned = if from == 0. && to == 1. { None }
            else { Some(om9_core::curve_ops::trim_curve_by_parameter(curve, from, to, limits)?) };
        let piece = owned.as_ref().unwrap_or(curve);
        let Curve::Nurbs(piece_nurbs) = piece else { return Err(Error::Unsupported("NURBS quadrant subcurve")) };
        if piece_nurbs.weights.iter().any(|weight| !weight.is_finite() || *weight <= 0.) {
            return Err(Error::Unsupported("NURBS quadrant weights"));
        }
        if owned.is_some() {
            let local = om9_core::evaluation::CurveEvaluator::capture(piece, limits)?
                .evaluate(Parameter::Fraction(0.5), 0, transform)?[0];
            let original = evaluator.evaluate(Parameter::Fraction((from + to) * 0.5), 0, transform)?[0];
            if distance(local, original) > tolerance * 0.0625 {
                return Err(Error::Unsupported("NURBS quadrant subcurve parameterization"));
            }
        }
        let mut range = [f64::INFINITY, f64::NEG_INFINITY, f64::INFINITY, f64::NEG_INFINITY];
        for &pole in &piece_nurbs.poles {
            if canceled() { return Err(Error::Canceled); }
            let point = transform.point(pole)?;
            for (index, axis) in [cplane.x, cplane.y].into_iter().enumerate() {
                let value = dot(sub(point, cplane.origin), axis);
                if !value.is_finite() { return Err(Error::Invalid("NURBS quadrant bounds")); }
                range[index * 2] = range[index * 2].min(value);
                range[index * 2 + 1] = range[index * 2 + 1].max(value);
            }
        }
        for axis in 0..2 {
            let margin = 128. * f64::EPSILON * range[axis * 2].abs().max(range[axis * 2 + 1].abs()).max(1.);
            range[axis * 2] -= margin; range[axis * 2 + 1] += margin;
        }
        Ok(range)
    };
    let first = evaluator.evaluate(Parameter::Fraction(0.), 0, transform)?[0];
    let values = [dot(sub(first, cplane.origin), cplane.x), dot(sub(first, cplane.origin), cplane.y)];
    let mut extrema = [(values[0], first, 0.), (values[0], first, 0.), (values[1], first, 0.), (values[1], first, 0.)];
    for fraction in [0.5, 1.] {
        if canceled() { return Err(Error::Canceled); }
        let point = evaluator.evaluate(Parameter::Fraction(fraction), 0, transform)?[0];
        consider(&mut extrema, point, fraction, cplane);
    }
    let mut intervals = vec![Interval { from: 0., to: 1., bounds: bounds(0., 1.)? }];
    while let Some(interval) = intervals.pop() {
        if canceled() { return Err(Error::Canceled); }
        let may_improve = interval.bounds[0] < extrema[0].0 - tolerance * 0.5
            || interval.bounds[1] > extrema[1].0 + tolerance * 0.5
            || interval.bounds[2] < extrema[2].0 - tolerance * 0.5
            || interval.bounds[3] > extrema[3].0 + tolerance * 0.5;
        if !may_improve { continue; }
        let middle = (interval.from + interval.to) * 0.5;
        if middle == interval.from || middle == interval.to { return Err(Error::Unsupported("NURBS quadrant parameter precision")); }
        for (from, to) in [(interval.from, middle), (middle, interval.to)] {
            let fraction = (from + to) * 0.5;
            let point = evaluator.evaluate(Parameter::Fraction(fraction), 0, transform)?[0];
            consider(&mut extrema, point, fraction, cplane);
            if intervals.len() >= 2048 || ((intervals.len() + 1) as u64)
                .saturating_mul(std::mem::size_of::<Interval>() as u64) > limits.max_bytes {
                return Err(Error::Budget("NURBS quadrant intervals"));
            }
            intervals.try_reserve(1).map_err(|_| Error::Budget("NURBS quadrant intervals"))?;
            intervals.push(Interval { from, to, bounds: bounds(from, to)? });
        }
    }
    let mut result = Vec::with_capacity(4);
    for pair in extrema.chunks_exact(2) {
        let (low, high) = (pair[0], pair[1]);
        let scale = low.0.abs().max(high.0.abs()).max(1.);
        if high.0 - low.0 > f64::EPSILON * 64. * scale {
            result.extend([(low.1, low.2), (high.1, high.2)]);
        }
    }
    Ok(result)
}

/// Approximate the world-space half-length angle for an affinely placed arc.
/// Simpson refinement and bisection are bounded; lack of convergence is explicit.
fn ellipse_mid_angle(x: Point3, y: Point3, radius: f64, angles: [f64; 2], tolerance: f64,
    work: &mut usize, canceled: &dyn Fn() -> bool) -> Result<f64> {
    let span = angles[1] - angles[0];
    let max_speed = radius * span.abs() * (dot(x, x).sqrt() + dot(y, y).sqrt());
    if !span.is_finite() || span <= 0. || !max_speed.is_finite() || max_speed <= 0.
        || !tolerance.is_finite() || tolerance <= 0. { return Err(Error::Invalid("ellipse midpoint input")); }
    let mut integrate = |end: f64| -> Result<f64> {
        if end == 0. { return Ok(0.); }
        let mut previous: Option<f64> = None;
        let mut count = 16usize;
        loop {
            *work = work.checked_add(count + 1).ok_or(Error::Overflow("ellipse midpoint work"))?;
            if *work > 1_000_000 { return Err(Error::Budget("ellipse midpoint work")); }
            let step = end / count as f64;
            let mut sum = 0.;
            for index in 0..=count {
                if index % 64 == 0 && canceled() { return Err(Error::Canceled); }
                let angle = angles[0] + span * step * index as f64;
                let velocity = add(mul(x, -angle.sin()), mul(y, angle.cos()));
                let speed = radius * span * dot(velocity, velocity).sqrt();
                if !speed.is_finite() { return Err(Error::Invalid("ellipse midpoint speed")); }
                let weight = if index == 0 || index == count { 1. } else if index % 2 == 0 { 2. } else { 4. };
                sum += weight * speed;
            }
            let value = sum * step / 3.;
            if !value.is_finite() { return Err(Error::Invalid("ellipse midpoint length")); }
            if let Some(old) = previous && (value - old).abs() <= tolerance / 32. { return Ok(value); }
            if count >= 4096 { return Err(Error::Unsupported("ellipse midpoint integration did not converge")); }
            previous = Some(value);
            count *= 2;
        }
    };
    let total = integrate(1.)?;
    let (mut low, mut high) = (0., 1.);
    for _ in 0..64 {
        if canceled() { return Err(Error::Canceled); }
        let middle = (low + high) * 0.5;
        if middle == low || middle == high { break; }
        if integrate(middle)? < total * 0.5 { low = middle; } else { high = middle; }
        if max_speed * (high - low) <= tolerance * 0.25 {
            return Ok(angles[0] + span * (low + high) * 0.5);
        }
    }
    Err(Error::Unsupported("ellipse midpoint parameter did not converge"))
}

#[allow(clippy::too_many_arguments)]
fn circle_snaps(
    root: ObjectId,
    center: Point3,
    x: Point3,
    y: Point3,
    radius: f64,
    angles: [f64; 2],
    camera: &Camera,
    viewport: Viewport,
    cursor: [f64; 2],
    settings: SnapSettings,
    from: Option<Point3>,
    cplane: CPlane,
    tolerance: f64,
    out: &mut Vec<SnapCandidate>,
    end_directions: &mut Option<EndDirections>,
    work: &mut usize,
    canceled: &dyn Fn() -> bool,
    defer_circle_eval: bool,
) -> Result<()> {
    if !radius.is_finite() || radius <= 0. {
        return Ok(());
    }
    let point_at = |angle: f64| add(center, add(mul(x, radius * angle.cos()), mul(y, radius * angle.sin())));
    if settings.is_effective(SnapKind::End) {
        for angle in angles {
            push_curve_end(root, point_at(angle), Some(normal(x, y)), Some(angle), camera,
                viewport, cursor, settings.radius_px, out, end_directions,
                |_| unit_direction(add(mul(x, -angle.sin()), mul(y, angle.cos()))))?;
        }
    }
    let (xx, xy, yy) = (dot(x, x), dot(x, y), dot(y, y));
    if settings.is_effective(SnapKind::Mid) && !defer_circle_eval {
        let angle = if (xx - yy).abs() <= 1e-12 * xx.max(yy) && xy.abs() <= 1e-12 * xx.max(yy) {
            0.5 * (angles[0] + angles[1])
        } else {
            ellipse_mid_angle(x, y, radius, angles, tolerance, work, canceled)?
        };
        push(SnapKind::Mid, Some(root), point_at(angle), Some(normal(x, y)), Some(angle), camera,
            viewport, cursor, settings.radius_px, out)?;
    }
    push(
        SnapKind::Center,
        Some(root),
        center,
        Some(normal(x, y)),
        None,
        camera,
        viewport,
        cursor,
        settings.radius_px,
        out,
    )?;
    if settings.is_effective(SnapKind::Quadrant) {
        for axis in [cplane.x, cplane.y] {
            let (a, b) = (dot(x, axis), dot(y, axis));
            if a.hypot(b) <= f64::EPSILON { continue; }
            let angle = b.atan2(a);
            for angle in [angle, angle + std::f64::consts::PI] {
                if !angle_on_arc(angle, angles) { continue; }
                let point = add(center, add(mul(x, radius * angle.cos()), mul(y, radius * angle.sin())));
                push(SnapKind::Quadrant, Some(root), point, Some(normal(x, y)), Some(angle), camera,
                    viewport, cursor, settings.radius_px, out)?;
            }
        }
        if (xx - yy).hypot(2. * xy) > 1e-12 * xx.max(yy) {
            let principal = 0.5 * (2. * xy).atan2(xx - yy);
            for index in 0..4 {
                let angle = principal + index as f64 * std::f64::consts::FRAC_PI_2;
                if !angle_on_arc(angle, angles) { continue; }
                let point = add(center, add(mul(x, radius * angle.cos()), mul(y, radius * angle.sin())));
                push(SnapKind::Quadrant, Some(root), point, Some(normal(x, y)), Some(angle), camera,
                    viewport, cursor, settings.radius_px, out)?;
            }
        }
    }
    if settings.is_effective(SnapKind::Near) && !defer_circle_eval
        && let Some((near, angle)) = circle_near_screen(center, x, y, radius, angles, camera, viewport, cursor,
            settings.radius_px, work, canceled)?
    {
        push(SnapKind::Near, Some(root), near, Some(normal(x, y)), Some(angle), camera,
            viewport, cursor, settings.radius_px, out)?;
    }

    let Some(from) = from else {
        return Ok(());
    };
    let delta = sub(from, center);
    let (xx, xy, yy) = (dot(x, x), dot(x, y), dot(y, y));
    let determinant = xx * yy - xy * xy;
    if determinant <= f64::EPSILON * xx * yy { return Ok(()); }
    let (dx, dy) = (dot(delta, x), dot(delta, y));
    let u = (dx * yy - dy * xy) / determinant;
    let v = (dy * xx - dx * xy) / determinant;
    let distance = u.hypot(v);
    if settings.is_effective(SnapKind::Tangent) && distance >= radius
        && dot(delta, normal(x, y)).abs() <= tolerance {
        let base = v.atan2(u);
        let offset = (radius / distance).acos();
        for sign in [-1.0, 1.0] {
            let angle = base + sign * offset;
            if !angle_on_arc(angle, angles) { continue; }
            let point = add(
                center,
                add(mul(x, radius * angle.cos()), mul(y, radius * angle.sin())),
            );
            push(
                SnapKind::Tangent,
                Some(root),
                point,
                Some(normal(x, y)),
                Some(angle),
                camera,
                viewport,
                cursor,
                settings.radius_px,
                out,
            )?;
        }
    }
    if settings.is_effective(SnapKind::Perpendicular) && !defer_circle_eval {
        for (point, angle) in ellipse_perpendiculars(center, x, y, radius, angles, from, tolerance, work, canceled)? {
            push(SnapKind::Perpendicular, Some(root), point, Some(normal(x, y)), Some(angle),
                camera, viewport, cursor, settings.radius_px, out)?;
        }
    }
    Ok(())
}

/// Bound the trigonometric normal equation by a derivative Lipschitz bound,
/// retaining even-multiplicity roots that do not cross zero.
fn ellipse_perpendiculars(center: Point3, x: Point3, y: Point3, radius: f64, angles: [f64; 2],
    from: Point3, tolerance: f64, work: &mut usize, canceled: &dyn Fn() -> bool) -> Result<Vec<(Point3, f64)>> {
    let delta = sub(from, center);
    let (a, b) = (dot(delta, x), dot(delta, y));
    let (xx, xy, yy) = (dot(x, x), dot(x, y), dot(y, y));
    let derivative_bound = a.abs() + b.abs() + radius * ((xx - yy).abs() + 2. * xy.abs());
    let speed_bound = radius * (xx.sqrt() + yy.sqrt());
    let span = (angles[1] - angles[0]).min(std::f64::consts::TAU);
    if !derivative_bound.is_finite() || !speed_bound.is_finite() || speed_bound <= 0.
        || !span.is_finite() || span <= 0. || !tolerance.is_finite() || tolerance <= 0. {
        return Err(Error::Invalid("ellipse perpendicular input"));
    }
    // Every point is perpendicular in this degenerate case, so there is no
    // discrete candidate; other snap kinds should still be available.
    if derivative_bound == 0. { return Ok(Vec::new()); }
    let residual = |angle: f64| {
        -a * angle.sin() + b * angle.cos() + radius * (0.5 * (xx - yy) * (2. * angle).sin() - xy * (2. * angle).cos())
    };
    let mut stack = vec![(angles[0], angles[0] + span)];
    let mut hits: Vec<(Point3, f64)> = Vec::new();
    while let Some((low, high)) = stack.pop() {
        if canceled() { return Err(Error::Canceled); }
        *work = work.checked_add(1).ok_or(Error::Overflow("ellipse perpendicular work"))?;
        if *work > 1_000_000 { return Err(Error::Budget("ellipse perpendicular work")); }
        let middle = low * 0.5 + high * 0.5;
        if middle == low || middle == high { return Err(Error::Unsupported("ellipse perpendicular precision")); }
        let value = residual(middle);
        let margin = 128. * f64::EPSILON * derivative_bound.max(value.abs()).max(1.);
        if !value.is_finite() || !margin.is_finite() { return Err(Error::Invalid("ellipse perpendicular residual")); }
        if value.abs() > derivative_bound * (high - low) * 0.5 + margin { continue; }
        if speed_bound * (high - low) <= tolerance * 0.25 {
            let (sin, cos) = middle.sin_cos();
            let point = add(center, add(mul(x, radius * cos), mul(y, radius * sin)));
            let derivative = add(mul(x, -radius * sin), mul(y, radius * cos));
            let speed = dot(derivative, derivative).sqrt();
            let error = dot(sub(from, point), derivative).abs() / speed;
            if speed.is_finite() && speed > 0. && error.is_finite() && error <= tolerance {
                if !hits.iter().any(|(old, _)| distance(*old, point) <= tolerance) {
                    if hits.len() >= 256 { return Err(Error::Budget("ellipse perpendicular candidates")); }
                    hits.try_reserve(1).map_err(|_| Error::Budget("ellipse perpendicular candidates"))?;
                    hits.push((point, middle));
                }
                continue;
            }
        }
        if stack.len() >= 2048 { return Err(Error::Budget("ellipse perpendicular intervals")); }
        stack.try_reserve(2).map_err(|_| Error::Budget("ellipse perpendicular intervals"))?;
        stack.push((middle, high)); stack.push((low, middle));
    }
    Ok(hits)
}

/// Bounded screen-space search, followed by source-parameter refinement. The returned
/// point is evaluated on the transformed analytic curve, never on display tessellation.
fn angle_on_arc(angle: f64, angles: [f64; 2]) -> bool {
    let span = angles[1] - angles[0];
    span.is_finite() && span > 0. && (span >= std::f64::consts::TAU
        || (angle - angles[0]).rem_euclid(std::f64::consts::TAU) <= span + 1e-12)
}

/// Subdivide source intervals using projected positive-weight control hulls.
/// The screen-distance tolerance is numerical, not a formal floating-point proof.
fn nurbs_near_screen(curve: &Curve, evaluator: &om9_core::evaluation::CurveEvaluator<'_>, transform: Transform,
    camera: &Camera, viewport: Viewport, cursor: [f64; 2], radius: f64, degree: usize,
    limits: &om9_core::budget::Limits, work: &mut usize, canceled: &dyn Fn() -> bool) -> Result<Option<(Point3, f64)>> {
    #[derive(Clone, Copy)]
    struct Interval { from: f64, to: f64, lower: f64 }
    let Curve::Nurbs(nurbs) = curve else { return Err(Error::Invalid("NURBS near source")) };
    let cost = degree.checked_mul(degree).and_then(|n| n.checked_mul(nurbs.poles.len().saturating_mul(2).saturating_add(50)))
        .ok_or(Error::Overflow("NURBS near work"))?;
    let projector = camera.projector(viewport)?;
    let sample = |fraction: f64| -> Result<(f64, Point3)> {
        if canceled() { return Err(Error::Canceled); }
        let point = evaluator.evaluate(om9_core::evaluation::Parameter::Fraction(fraction), 0, transform)?[0];
        let score = projector.project(point)?.map_or(f64::INFINITY, |projected|
            (projected.screen[0] - cursor[0]).hypot(projected.screen[1] - cursor[1]));
        Ok((score, point))
    };
    let forward = sub(camera.target, camera.eye);
    let magnitude = dot(forward, forward).sqrt();
    if !magnitude.is_finite() || magnitude <= 0. { return Err(Error::Invalid("NURBS near camera")); }
    let forward = mul(forward, 1. / magnitude);
    let mut lower_bound = |from: f64, to: f64| -> Result<Option<f64>> {
        if canceled() { return Err(Error::Canceled); }
        *work = work.checked_add(cost).ok_or(Error::Overflow("NURBS near work"))?;
        if *work > 1_000_000 { return Err(Error::Budget("NURBS near work")); }
        let owned = if from == 0. && to == 1. { None }
            else { Some(om9_core::curve_ops::trim_curve_by_parameter(curve, from, to, limits)?) };
        let piece = owned.as_ref().unwrap_or(curve);
        let Curve::Nurbs(piece) = piece else { return Err(Error::Unsupported("NURBS near subcurve")) };
        if piece.weights.iter().any(|weight| !weight.is_finite() || *weight <= 0.) {
            return Err(Error::Unsupported("NURBS near weights"));
        }
        let mut min_depth = f64::INFINITY;
        let mut max_depth = f64::NEG_INFINITY;
        let mut low = [f64::INFINITY; 2];
        let mut high = [f64::NEG_INFINITY; 2];
        let mut clipped = false;
        for &pole in &piece.poles {
            if canceled() { return Err(Error::Canceled); }
            let point = transform.point(pole)?;
            let depth = dot(sub(point, camera.eye), forward);
            min_depth = min_depth.min(depth); max_depth = max_depth.max(depth);
            if let Some(projected) = projector.project(point)? {
                for axis in 0..2 { low[axis] = low[axis].min(projected.screen[axis]); high[axis] = high[axis].max(projected.screen[axis]); }
            } else { clipped = true; }
        }
        if !min_depth.is_finite() || !max_depth.is_finite() { return Err(Error::Invalid("NURBS near hull")); }
        if max_depth < camera.near_mm || min_depth > camera.far_mm { return Ok(None); }
        if clipped { return Ok(Some(0.)); }
        let scale = low.iter().chain(high.iter()).map(|value| value.abs()).fold(1f64, f64::max);
        let margin = 128. * f64::EPSILON * scale;
        let dx = (low[0] - margin - cursor[0]).max(0.).max(cursor[0] - high[0] - margin);
        let dy = (low[1] - margin - cursor[1]).max(0.).max(cursor[1] - high[1] - margin);
        Ok(Some(dx.hypot(dy)))
    };
    let mut best = (f64::INFINITY, 0., [0.; 3]);
    for fraction in [0., 0.5, 1.] {
        let (score, point) = sample(fraction)?;
        if score < best.0 { best = (score, fraction, point); }
    }
    let mut intervals = Vec::new();
    if let Some(lower) = lower_bound(0., 1.)? { intervals.push(Interval { from: 0., to: 1., lower }); }
    while !intervals.is_empty() {
        if canceled() { return Err(Error::Canceled); }
        let index = intervals.iter().enumerate().min_by(|(_, a), (_, b)| a.lower.total_cmp(&b.lower))
            .map(|(index, _)| index).expect("nonempty intervals");
        let interval = intervals.swap_remove(index);
        if interval.lower > radius && best.0 > radius { continue; }
        if best.0.is_finite() && interval.lower >= (best.0 - 0.05).max(0.) { continue; }
        let middle = (interval.from + interval.to) * 0.5;
        if middle == interval.from || middle == interval.to { return Err(Error::Unsupported("NURBS near parameter precision")); }
        for (from, to) in [(interval.from, middle), (middle, interval.to)] {
            let fraction = (from + to) * 0.5;
            let (score, point) = sample(fraction)?;
            if score < best.0 { best = (score, fraction, point); }
            if let Some(lower) = lower_bound(from, to)? {
                if intervals.len() >= 2048 || ((intervals.len() + 1) as u64)
                    .saturating_mul(std::mem::size_of::<Interval>() as u64) > limits.max_bytes {
                    return Err(Error::Budget("NURBS near intervals"));
                }
                intervals.try_reserve(1).map_err(|_| Error::Budget("NURBS near intervals"))?;
                intervals.push(Interval { from, to, lower });
            }
        }
    }
    Ok((best.0.is_finite() && best.0 <= radius).then_some((best.2, best.1)))
}

/// Bound half world-space arc length using the endpoint chord and the positive-
/// weight control polygon of each source subcurve. Floating-point arithmetic and
/// subcurve construction still require independent numerical validation.
fn nurbs_midpoint(curve: &Curve, evaluator: &om9_core::evaluation::CurveEvaluator<'_>, transform: Transform,
    degree: usize, minimum_samples: usize, tolerance: f64, limits: &om9_core::budget::Limits,
    work: &mut usize, canceled: &dyn Fn() -> bool) -> Result<(Point3, f64)> {
    fn polygon_length(curve: &Curve, transform: Transform) -> Result<f64> {
        match curve {
            Curve::Nurbs(nurbs) => {
                if nurbs.weights.iter().any(|weight| !weight.is_finite() || *weight <= 0.) {
                    return Err(Error::Unsupported("NURBS midpoint weights"));
                }
                let mut length = 0.;
                for pair in nurbs.poles.windows(2) {
                    length += distance(transform.point(pair[0])?, transform.point(pair[1])?);
                }
                Ok(length)
            }
            Curve::Line { start, end, .. } => Ok(distance(transform.point(*start)?, transform.point(*end)?)),
            Curve::Polycurve { segments, .. } => segments.iter().try_fold(0., |sum, part| Ok(sum + polygon_length(part, transform)?)),
            Curve::Circle { .. } => Err(Error::Unsupported("NURBS midpoint subcurve")),
        }
    }
    let poles = match curve { Curve::Nurbs(nurbs) => nurbs.poles.len(), _ => return Err(Error::Invalid("NURBS midpoint source")) };
    let cost = degree.checked_mul(degree).and_then(|n| n.checked_mul(poles.saturating_mul(2).saturating_add(50)))
        .ok_or(Error::Overflow("NURBS midpoint work"))?;
    for count in [8usize, 16, 32, 64, 128, 256, 512, 1024] {
        if canceled() { return Err(Error::Canceled); }
        if count < minimum_samples { continue; }
        if (count as u64 + 1) > limits.max_points
            || (count as u64 + 1).saturating_mul(std::mem::size_of::<Point3>() as u64) > limits.max_bytes {
            return Err(Error::Budget("NURBS midpoint points"));
        }
        let step_work = count.checked_mul(cost).ok_or(Error::Overflow("NURBS midpoint work"))?;
        *work = work.checked_add(step_work).ok_or(Error::Overflow("NURBS midpoint work"))?;
        if *work > 1_000_000 { return Err(Error::Budget("NURBS midpoint work")); }
        let mut lower = [0.; 1025];
        let mut upper = [0.; 1025];
        let mut previous = evaluator.evaluate(om9_core::evaluation::Parameter::Fraction(0.), 0, transform)?[0];
        for index in 0..count {
            if canceled() { return Err(Error::Canceled); }
            let from = index as f64 / count as f64;
            let to = (index + 1) as f64 / count as f64;
            let next = evaluator.evaluate(om9_core::evaluation::Parameter::Fraction(to), 0, transform)?[0];
            let subcurve = om9_core::curve_ops::trim_curve_by_parameter(curve, from, to, limits)?;
            let local_mid = om9_core::evaluation::CurveEvaluator::capture(&subcurve, limits)?
                .evaluate(om9_core::evaluation::Parameter::Fraction(0.5), 0, transform)?[0];
            let source_mid = evaluator.evaluate(om9_core::evaluation::Parameter::Fraction((from + to) * 0.5), 0, transform)?[0];
            if distance(local_mid, source_mid) > tolerance * 0.0625 {
                return Err(Error::Unsupported("NURBS midpoint subcurve parameterization"));
            }
            let chord = distance(previous, next);
            let polygon = polygon_length(&subcurve, transform)?;
            let roundoff = f64::EPSILON * 128. * chord.max(polygon).max(1.);
            lower[index + 1] = lower[index] + (chord - roundoff).max(0.);
            upper[index + 1] = upper[index] + polygon.max(chord) + roundoff;
            previous = next;
        }
        if !lower[count].is_finite() || !upper[count].is_finite() || upper[count] <= 0. {
            return Err(Error::Invalid("NURBS midpoint length"));
        }
        let earliest = (1..=count).find(|&index| upper[index] >= lower[count] * 0.5).unwrap_or(count) - 1;
        let latest = (1..=count).find(|&index| lower[index] >= upper[count] * 0.5).unwrap_or(count);
        if upper[count] - lower[count] > tolerance * 0.5 { continue; }
        let (mut left, mut right) = (earliest as f64 / count as f64, latest as f64 / count as f64);
        for _ in 0..40 {
            if canceled() { return Err(Error::Canceled); }
            let fraction = (left + right) * 0.5;
            let index = ((fraction * count as f64).floor() as usize).min(count - 1);
            let start = index as f64 / count as f64;
            let point = evaluator.evaluate(om9_core::evaluation::Parameter::Fraction(fraction), 0, transform)?[0];
            let base = evaluator.evaluate(om9_core::evaluation::Parameter::Fraction(start), 0, transform)?[0];
            let (mut partial_low, mut partial_high) = (0., 0.);
            if fraction > start {
                *work = work.checked_add(cost).ok_or(Error::Overflow("NURBS midpoint work"))?;
                if *work > 1_000_000 { return Err(Error::Budget("NURBS midpoint work")); }
                let partial = om9_core::curve_ops::trim_curve_by_parameter(curve, start, fraction, limits)?;
                let chord = distance(base, point);
                let polygon = polygon_length(&partial, transform)?;
                let roundoff = f64::EPSILON * 128. * chord.max(polygon).max(1.);
                partial_low = (chord - roundoff).max(0.);
                partial_high = polygon.max(chord) + roundoff;
            }
            let (prefix_low, prefix_high) = (lower[index] + partial_low, upper[index] + partial_high);
            let (target_low, target_high) = (lower[count] * 0.5, upper[count] * 0.5);
            let worst = (prefix_low - target_high).abs().max((prefix_high - target_low).abs());
            if worst <= tolerance * 0.5 { return Ok((point, fraction)); }
            if prefix_high < target_low { left = fraction; }
            else if prefix_low > target_high { right = fraction; }
            else { break; } // Refine the length bounds before choosing a side.
        }
    }
    Err(Error::Unsupported("NURBS midpoint bounds did not converge within budget"))
}

fn circle_near_screen(center: Point3, x: Point3, y: Point3, radius: f64, angles: [f64; 2], camera: &Camera,
    viewport: Viewport, cursor: [f64; 2], radius_px: f64, work: &mut usize,
    canceled: &dyn Fn() -> bool) -> Result<Option<(Point3, f64)>> {
    let point_at = |angle: f64| add(center, add(mul(x, radius * angle.cos()), mul(y, radius * angle.sin())));
    let span = (angles[1] - angles[0]).min(std::f64::consts::TAU);
    if !span.is_finite() || span <= 0. { return Ok(None); }
    let projector = camera.projector(viewport)?;
    let forward = sub(camera.target, camera.eye);
    let forward_length = dot(forward, forward).sqrt();
    if !forward_length.is_finite() || forward_length <= 0. { return Err(Error::Invalid("arc near camera")); }
    let forward = mul(forward, 1. / forward_length);
    let mut best: Option<(f64, f64)> = None;
    let consider = |angle: f64, best: &mut Option<(f64, f64)>| -> Result<()> {
        if let Some(p) = projector.project(point_at(angle))? {
            let score = (p.screen[0] - cursor[0]).hypot(p.screen[1] - cursor[1]);
            if score.is_finite() && best.is_none_or(|(old, _)| score < old) { *best = Some((score, angle)); }
        }
        Ok(())
    };
    consider(angles[0], &mut best)?; consider(angles[0] + span, &mut best)?;
    let mut stack = vec![(angles[0], angles[0] + span)];
    while let Some((low, high)) = stack.pop() {
        if canceled() { return Err(Error::Canceled); }
        *work = work.checked_add(1).ok_or(Error::Overflow("arc near work"))?;
        if *work > 1_000_000 { return Err(Error::Budget("arc near work")); }
        let middle = low * 0.5 + high * 0.5;
        if middle == low || middle == high { return Err(Error::Unsupported("arc near parameter precision")); }
        consider(middle, &mut best)?;
        let mut lower = [0.; 3]; let mut upper = [0.; 3];
        for axis in 0..3 {
            let a = radius * x[axis]; let b = radius * y[axis];
            let value = |angle: f64| center[axis] + a * angle.cos() + b * angle.sin();
            let (mut lo, mut hi) = (value(low).min(value(high)), value(low).max(value(high)));
            let phase = b.atan2(a);
            for critical in [phase, phase + std::f64::consts::PI] {
                let turn = ((low - critical) / std::f64::consts::TAU).ceil();
                let angle = critical + turn * std::f64::consts::TAU;
                if angle >= low && angle <= high { let v = value(angle); lo = lo.min(v); hi = hi.max(v); }
            }
            let margin = 256. * f64::EPSILON * lo.abs().max(hi.abs()).max(1.);
            lower[axis] = lo - margin; upper[axis] = hi + margin;
        }
        let corners: [Point3; 8] = std::array::from_fn(|index| std::array::from_fn(|axis|
            if index & (1 << axis) == 0 { lower[axis] } else { upper[axis] }));
        let depths = corners.map(|p| dot(sub(p, camera.eye), forward));
        if depths.iter().all(|depth| *depth < camera.near_mm)
            || depths.iter().all(|depth| *depth > camera.far_mm) { continue; }
        let bounds = projector.project_box(corners)?;
        if let Some(bounds) = bounds {
            let dx = (bounds[0][0] - cursor[0]).max(0.).max(cursor[0] - bounds[1][0]);
            let dy = (bounds[0][1] - cursor[1]).max(0.).max(cursor[1] - bounds[1][1]);
            let lower_bound = dx.hypot(dy);
            let threshold = best.map_or(radius_px, |(score, _)| score.min(radius_px));
            if lower_bound > threshold + 1e-6 { continue; }
            let diagonal = (bounds[1][0] - bounds[0][0]).hypot(bounds[1][1] - bounds[0][1]);
            if diagonal <= 0.05 && best.is_some_and(|(score, _)| score <= radius_px && score <= lower_bound + 0.05) {
                continue;
            }
        }
        if stack.len() >= 2048 { return Err(Error::Budget("arc near intervals")); }
        stack.try_reserve(2).map_err(|_| Error::Budget("arc near intervals"))?;
        stack.push((middle, high)); stack.push((low, middle));
    }
    Ok(best.filter(|(score, _)| *score <= radius_px).map(|(_, angle)| (point_at(angle), angle)))
}

#[allow(clippy::too_many_arguments)]
fn push_curve_end(root: ObjectId, world: Point3, normal: Option<Point3>, parameter: Option<f64>,
    camera: &Camera, viewport: Viewport, cursor: [f64; 2], radius: f64, out: &mut Vec<SnapCandidate>,
    end_directions: &mut Option<EndDirections>, tangent: impl FnOnce(Option<&mut EndDirections>) -> Option<Point3>,
) -> Result<()> {
    let first = out.len();
    push(SnapKind::End, Some(root), world, normal, parameter, camera, viewport, cursor, radius, out)?;
    let hit = out.get_mut(first);
    if hit.is_some() || end_directions.as_ref().is_some_and(|ends| ends.witnesses.is_some()) {
        let direction = tangent(end_directions.as_mut());
        if let Some(ends) = end_directions { ends.record(world, direction); }
        if let Some(hit) = hit { hit.tracking_tangent = direction; }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn push(
    kind: SnapKind,
    root: Option<ObjectId>,
    world: Point3,
    normal: Option<Point3>,
    parameter: Option<f64>,
    camera: &Camera,
    viewport: Viewport,
    cursor: [f64; 2],
    radius: f64,
    out: &mut Vec<SnapCandidate>,
) -> Result<()> {
    finite(world)?;
    let Some(projected) = camera.project(world, viewport)? else {
        return Ok(());
    };
    let distance = (projected.screen[0] - cursor[0]).hypot(projected.screen[1] - cursor[1]);
    if distance.is_finite() && distance <= radius {
        out.push(SnapCandidate {
            kind,
            root,
            world,
            screen: projected.screen,
            distance_px: distance,
            normal,
            parameter,
            tracking_tangent: None,
            source: None,
            second_source: None,
            surface_source: None,
            apparent: false,
        });
    }
    Ok(())
}

fn transform_vector(transform: om9_core::geometry::Transform, vector: Point3) -> Result<Point3> {
    let a = transform.point([0.; 3])?;
    let b = transform.point(vector)?;
    Ok(sub(b, a))
}

fn normal(x: Point3, y: Point3) -> Point3 {
    let n = [
        x[1] * y[2] - x[2] * y[1],
        x[2] * y[0] - x[0] * y[2],
        x[0] * y[1] - x[1] * y[0],
    ];
    let length = dot(n, n).sqrt();
    if length > 0. {
        n.map(|v| v / length)
    } else {
        n
    }
}

fn unit_direction(direction: Point3) -> Option<Point3> {
    let length = dot(direction, direction).sqrt();
    (length.is_finite() && length > 0.).then(|| mul(direction, length.recip()))
}

fn closest_on_segment(point: Point3, a: Point3, b: Point3) -> (Point3, f64) {
    let ab = sub(b, a);
    let length = dot(ab, ab);
    if length <= 0. {
        return (a, 0.);
    }
    let t = (dot(sub(point, a), ab) / length).clamp(0., 1.);
    (add(a, mul(ab, t)), t)
}

fn closest_segment_on_screen(camera: &Camera, viewport: Viewport, cursor: [f64; 2], a: Point3, b: Point3)
    -> Result<Option<(Point3, f64)>> {
    let Some([start, end]) = camera.clip_segment(a, b)? else { return Ok(None) };
    let (Some(first), Some(last)) = (camera.project(start, viewport)?, camera.project(end, viewport)?) else { return Ok(None) };
    let delta = [last.screen[0] - first.screen[0], last.screen[1] - first.screen[1]];
    let denominator = delta[0] * delta[0] + delta[1] * delta[1];
    let s = if denominator > 0. {
        (((cursor[0] - first.screen[0]) * delta[0] + (cursor[1] - first.screen[1]) * delta[1]) / denominator).clamp(0., 1.)
    } else if first.depth <= last.depth { 0. } else { 1. };
    let local = camera.projection.segment_source_fraction(s, [first.depth, last.depth])?;
    let point = add(start, mul(sub(end, start), local));
    let (_, fraction) = closest_on_segment(point, a, b);
    Ok(Some((point, fraction)))
}

fn line_intersection(a0: Point3, a1: Point3, b0: Point3, b1: Point3) -> Option<(Point3, f64)> {
    let p = a0;
    let r = sub(a1, a0);
    let q = b0;
    let s = sub(b1, b0);
    let rxs = cross(r, s);
    let denominator = dot(rxs, rxs);
    if denominator <= 1.0e-24 {
        return None;
    }
    let qp = sub(q, p);
    let t = dot(cross(qp, s), rxs) / denominator;
    let u = dot(cross(qp, r), rxs) / denominator;
    let left_point = add(p, mul(r, t));
    let right_point = add(q, mul(s, u));
    Some((
        midpoint(left_point, right_point),
        distance(left_point, right_point),
    ))
}

fn segment_ellipse_intersections(segment: Segment, ellipse: Ellipse, tolerance: f64) -> Vec<Point3> {
    let (xx, xy, yy) = (dot(ellipse.x, ellipse.x), dot(ellipse.x, ellipse.y), dot(ellipse.y, ellipse.y));
    let determinant = xx * yy - xy * xy;
    if determinant <= f64::EPSILON * xx * yy || ellipse.radius <= 0. { return Vec::new(); }
    let coordinates = |point: Point3| {
        let dx = dot(point, ellipse.x);
        let dy = dot(point, ellipse.y);
        [(dx * yy - dy * xy) / determinant, (dy * xx - dx * xy) / determinant]
    };
    let offset = sub(segment.a, ellipse.center);
    let direction = sub(segment.b, segment.a);
    let n = normal(ellipse.x, ellipse.y);
    let height = dot(offset, n);
    let slope = dot(direction, n);
    let mut parameters = [f64::NAN; 2];
    if slope.abs() > 1e-12 * dot(direction, direction).sqrt() {
        parameters[0] = -height / slope;
    } else if height.abs() <= tolerance {
        let [u, v] = coordinates(offset);
        let [du, dv] = coordinates(direction);
        let (a, b, c) = (du * du + dv * dv, 2. * (u * du + v * dv), u * u + v * v - ellipse.radius * ellipse.radius);
        let discriminant = b * b - 4. * a * c;
        if a > 0. && discriminant >= 0. {
            let root = discriminant.sqrt();
            parameters = [(-b - root) / (2. * a), (-b + root) / (2. * a)];
        }
    }
    let mut points = Vec::new();
    for t in parameters {
        if !(0.0..=1.0).contains(&t) { continue; }
        let on_line = add(segment.a, mul(direction, t));
        let [u, v] = coordinates(sub(on_line, ellipse.center));
        let angle = v.atan2(u);
        if !angle_on_arc(angle, ellipse.angles) { continue; }
        let on_ellipse = add(ellipse.center, add(mul(ellipse.x, ellipse.radius * angle.cos()), mul(ellipse.y, ellipse.radius * angle.sin())));
        if distance(on_line, on_ellipse) <= tolerance && points.iter().all(|point| distance(*point, on_ellipse) > tolerance) {
            points.push(on_ellipse);
        }
    }
    points
}

fn ray_triangle(
    ray: crate::camera::Ray,
    a: Point3,
    b: Point3,
    c: Point3,
) -> Option<(Point3, Point3)> {
    let edge1 = sub(b, a);
    let edge2 = sub(c, a);
    // Normalize edge lengths before the parallelism test: an absolute
    // determinant threshold rejects small, perfectly visible triangles.
    let length1 = edge1[0].hypot(edge1[1]).hypot(edge1[2]);
    let length2 = edge2[0].hypot(edge2[1]).hypot(edge2[2]);
    let ray_length = ray.direction[0].hypot(ray.direction[1]).hypot(ray.direction[2]);
    if !length1.is_finite() || !length2.is_finite() || !ray_length.is_finite()
        || length1 == 0. || length2 == 0. || ray_length == 0. {
        return None;
    }
    let unit1 = edge1.map(|value| value / length1);
    let unit2 = edge2.map(|value| value / length2);
    let h = cross(ray.direction, unit2);
    let determinant = dot(unit1, h);
    if !determinant.is_finite() || determinant.abs() <= 1.0e-12 * ray_length {
        return None;
    }
    let inverse = 1.0 / determinant;
    let s = sub(ray.origin, a);
    let u = (inverse * dot(s, h)) / length1;
    // Allow roundoff at a shared edge: exact refinement still verifies
    // surface candidates against the canonical geometry.
    const BARYCENTRIC_EPSILON: f64 = 1.0e-12;
    if !(-BARYCENTRIC_EPSILON..=1.0 + BARYCENTRIC_EPSILON).contains(&u) {
        return None;
    }
    let q = cross(s, unit1);
    let v = (inverse * dot(ray.direction, q)) / length2;
    if !v.is_finite() || v < -BARYCENTRIC_EPSILON || u + v > 1.0 + BARYCENTRIC_EPSILON {
        return None;
    }
    let t = inverse * dot(unit2, q);
    if !t.is_finite() || t < 0. {
        return None;
    }
    Some((add(ray.origin, mul(ray.direction, t)), normal(unit1, unit2)))
}

fn midpoint(a: Point3, b: Point3) -> Point3 {
    [
        (a[0] + b[0]) * 0.5,
        (a[1] + b[1]) * 0.5,
        (a[2] + b[2]) * 0.5,
    ]
}
fn add(a: Point3, b: Point3) -> Point3 {
    std::array::from_fn(|i| a[i] + b[i])
}
fn sub(a: Point3, b: Point3) -> Point3 {
    std::array::from_fn(|i| a[i] - b[i])
}
fn mul(a: Point3, s: f64) -> Point3 {
    a.map(|v| v * s)
}
fn dot(a: Point3, b: Point3) -> f64 {
    (0..3).map(|i| a[i] * b[i]).sum()
}
fn cross(a: Point3, b: Point3) -> Point3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn distance(a: Point3, b: Point3) -> f64 {
    dot(sub(a, b), sub(a, b)).sqrt()
}
