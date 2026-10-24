// SPDX-License-Identifier: LGPL-2.1-or-later
//! Object snaps and SmartTrack inference (`OM9-L2-02-03`).
//!
//! The module is pure geometry over a captured [`Scene`]; it never mutates the document and
//! never runs a second picking engine. The viewport supplies the camera, rectangle and cursor.
use crate::{
    camera::{CPlane, Camera, Viewport},
    scene::Scene,
};
use om9_core::{
    Error, Result,
    geometry::{Curve, Geometry, Point3, finite},
    identity::ObjectId,
};

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

#[derive(Debug, Clone, Copy)]
pub struct SnapSettings {
    /// Bit set from [`SnapKind::bit`].
    pub enabled: u16,
    pub radius_px: f64,
    /// A one-shot kind restricts exactly the next pick; `accept` clears it.
    pub one_shot: Option<SnapKind>,
}

impl Default for SnapSettings {
    fn default() -> Self {
        Self {
            enabled: SnapKind::ALL.iter().fold(0, |bits, kind| bits | kind.bit()),
            radius_px: 8.0,
            one_shot: None,
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
}

#[derive(Debug, Clone, PartialEq)]
pub struct SnapCandidate {
    pub kind: SnapKind,
    pub root: Option<ObjectId>,
    pub world: Point3,
    pub screen: [f64; 2],
    pub distance_px: f64,
    pub normal: Option<Point3>,
    pub parameter: Option<f64>,
}

impl SnapCandidate {
    pub fn tooltip(&self) -> String {
        self.kind.tooltip().to_owned()
    }
    pub fn marker_label(&self) -> &'static str {
        self.kind.label()
    }
}

/// A transient anchor captured from an accepted snap. It lives only in the viewport session
/// and is never written to the document.
#[derive(Debug, Clone, PartialEq)]
pub struct SnapAnchor {
    pub kind: SnapKind,
    pub world: Point3,
    pub screen: [f64; 2],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SnapGuide {
    pub start: Point3,
    pub end: Point3,
    /// `0` was the first CPlane axis, `1` the second.
    pub axis: u8,
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
        if self.anchors.len() == 2 {
            self.anchors.remove(0);
        }
        self.anchors.push(SnapAnchor {
            kind: candidate.kind,
            world: candidate.world,
            screen: candidate.screen,
        });
        self.guides.clear();
        let extent = 1000.0;
        for anchor in &self.anchors {
            for (axis, direction) in [(0, plane.x), (1, plane.y)] {
                self.guides.push(SnapGuide {
                    start: add(anchor.world, mul(direction, -extent)),
                    end: add(anchor.world, mul(direction, extent)),
                    axis,
                });
            }
        }
        Ok(())
    }
    /// The intersection of two inferred alignment lines nearest the cursor, if the cursor is
    /// within the supplied world tolerance. The result is ephemeral and never a document point.
    pub fn infer(&self, cursor: Point3, tolerance: f64) -> Option<SnapCandidate> {
        if !tolerance.is_finite() || tolerance < 0. {
            return None;
        }
        let mut best: Option<(f64, Point3)> = None;
        for (index, left) in self.guides.iter().enumerate() {
            for right in &self.guides[index + 1..] {
                if left.axis == right.axis {
                    continue;
                }
                let Some((point, separation)) =
                    line_intersection(left.start, left.end, right.start, right.end)
                else {
                    continue;
                };
                if separation > 1.0e-6 {
                    continue;
                }
                let distance = distance(point, cursor);
                if distance <= tolerance && best.is_none_or(|(old, _)| distance < old) {
                    best = Some((distance, point));
                }
            }
        }
        best.map(|(_, world)| SnapCandidate {
            kind: SnapKind::SmartTrack,
            root: None,
            world,
            screen: [0.; 2],
            distance_px: 0.,
            normal: None,
            parameter: None,
        })
    }
    pub fn clear(&mut self) {
        self.anchors.clear();
        self.guides.clear();
    }
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
    candidates
        .iter()
        .min_by(|left, right| {
            left.kind
                .priority()
                .cmp(&right.kind.priority())
                .then(left.distance_px.total_cmp(&right.distance_px))
                .then(left.root.cmp(&right.root))
        })
        .cloned()
}

#[derive(Debug, Clone, Copy)]
struct Segment {
    root: ObjectId,
    a: Point3,
    b: Point3,
}

pub fn candidates(
    scene: &Scene,
    camera: &Camera,
    viewport: Viewport,
    cursor: [f64; 2],
    settings: SnapSettings,
    from: Option<Point3>,
) -> Result<Vec<SnapCandidate>> {
    if !settings.radius_px.is_finite() || settings.radius_px < 0. {
        return Err(Error::Invalid("snap radius"));
    }
    let mut out = Vec::new();
    let mut segments = Vec::new();
    for object in &scene.objects {
        if !object.visible || !(object.selectable || object.reference_snappable) {
            continue;
        }
        let transform = object.display_transform()?;
        let entity = scene.entity_for(object)?;
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
                object.root,
                transform,
                camera,
                viewport,
                cursor,
                settings,
                from,
                &mut out,
                &mut segments,
            )?,
            Geometry::Brep(brep) => {
                let ray = camera.ray(cursor, viewport)?;
                let mut best: Option<(f64, Point3, Point3)> = None;
                for triangle in &brep.display.triangles {
                    let a = transform.point(brep.display.points[triangle[0] as usize])?;
                    let b = transform.point(brep.display.points[triangle[1] as usize])?;
                    let c = transform.point(brep.display.points[triangle[2] as usize])?;
                    if let Some((point, normal)) = ray_triangle(ray, a, b, c) {
                        let distance = distance(camera.eye, point);
                        if best.is_none_or(|(old, _, _)| distance < old) {
                            best = Some((distance, point, normal));
                        }
                    }
                }
                if let Some((_, point, normal)) = best {
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
                }
            }
            Geometry::NativeBrep(brep) => {
                let ray = camera.ray(cursor, viewport)?;
                let mut best: Option<(f64, Point3, Point3)> = None;
                for face in &brep.display.faces {
                    for triangle in face.windows(3) {
                        let a = transform.point(brep.display.vertices[triangle[0] as usize])?;
                        let b = transform.point(brep.display.vertices[triangle[1] as usize])?;
                        let c = transform.point(brep.display.vertices[triangle[2] as usize])?;
                        if let Some((point, normal)) = ray_triangle(ray, a, b, c) {
                            let distance = distance(camera.eye, point);
                            if best.is_none_or(|(old, _, _)| distance < old) {
                                best = Some((distance, point, normal));
                            }
                        }
                    }
                }
                if let Some((_, point, normal)) = best {
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
                }
            }
            _ => {}
        }
    }
    for (index, left) in segments.iter().enumerate() {
        for right in &segments[index + 1..] {
            if left.root == right.root {
                continue;
            }
            if let Some((point, distance)) = segment_intersection(*left, *right) {
                let scale = segment_length(*left).max(segment_length(*right)).max(1.0);
                if distance <= 1.0e-9 * scale {
                    push(
                        SnapKind::Intersection,
                        Some(left.root),
                        point,
                        None,
                        None,
                        camera,
                        viewport,
                        cursor,
                        settings.radius_px,
                        &mut out,
                    )?;
                }
            }
        }
    }
    out.retain(|candidate| settings.is_enabled(candidate.kind));
    out.sort_by(|left, right| {
        left.kind
            .priority()
            .cmp(&right.kind.priority())
            .then(left.distance_px.total_cmp(&right.distance_px))
            .then(left.root.cmp(&right.root))
    });
    out.dedup_by(|left, right| {
        left.kind == right.kind
            && left.root == right.root
            && left.parameter == right.parameter
            && distance(left.world, right.world) <= 1.0e-9
    });
    Ok(out)
}

#[allow(clippy::too_many_arguments)]
fn curve_snaps(
    curve: &Curve,
    root: ObjectId,
    transform: om9_core::geometry::Transform,
    camera: &Camera,
    viewport: Viewport,
    cursor: [f64; 2],
    settings: SnapSettings,
    from: Option<Point3>,
    out: &mut Vec<SnapCandidate>,
    segments: &mut Vec<Segment>,
) -> Result<()> {
    match curve {
        Curve::Line { start, end, .. } => {
            let a = transform.point(*start)?;
            let b = transform.point(*end)?;
            segments.push(Segment { root, a, b });
            push(
                SnapKind::End,
                Some(root),
                a,
                None,
                Some(0.),
                camera,
                viewport,
                cursor,
                settings.radius_px,
                out,
            )?;
            push(
                SnapKind::End,
                Some(root),
                b,
                None,
                Some(1.),
                camera,
                viewport,
                cursor,
                settings.radius_px,
                out,
            )?;
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
            let ray_point = screen_ray_point(camera, viewport, cursor)?;
            let (nearest, t) = closest_on_segment(ray_point, a, b);
            if (0.0..=1.0).contains(&t) {
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
            if let Some(from) = from {
                let (foot, foot_t) = closest_on_segment(from, a, b);
                if (0.0..=1.0).contains(&foot_t) {
                    push(
                        SnapKind::Perpendicular,
                        Some(root),
                        foot,
                        None,
                        Some(foot_t),
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
            ..
        } => {
            let center = transform.point(*center)?;
            let x = transform_vector(transform, *x)?;
            let y = transform_vector(transform, *y)?;
            circle_snaps(
                root, center, x, y, *radius, camera, viewport, cursor, settings, from, out,
            )?;
        }
        Curve::Polycurve {
            segments: parts, ..
        } => {
            for part in parts {
                curve_snaps(
                    part, root, transform, camera, viewport, cursor, settings, from, out, segments,
                )?;
            }
        }
        Curve::Nurbs(_) => {
            // The display samples are exact enough for an object snap; a native branch would
            // introduce a second curve solver here.
            let _ = from;
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn circle_snaps(
    root: ObjectId,
    center: Point3,
    x: Point3,
    y: Point3,
    radius: f64,
    camera: &Camera,
    viewport: Viewport,
    cursor: [f64; 2],
    settings: SnapSettings,
    from: Option<Point3>,
    out: &mut Vec<SnapCandidate>,
) -> Result<()> {
    if !radius.is_finite() || radius <= 0. {
        return Ok(());
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
    for (index, direction) in [x, y, [-x[0], -x[1], -x[2]], [-y[0], -y[1], -y[2]]]
        .into_iter()
        .enumerate()
    {
        let point = add(center, mul(direction, radius));
        push(
            SnapKind::Quadrant,
            Some(root),
            point,
            Some(normal(x, y)),
            Some(index as f64),
            camera,
            viewport,
            cursor,
            settings.radius_px,
            out,
        )?;
    }
    // Near: project the cursor ray onto the circle plane and clamp to the radius.
    let ray_point = screen_ray_point(camera, viewport, cursor)?;
    let local_u = dot(sub(ray_point, center), x);
    let local_v = dot(sub(ray_point, center), y);
    let angle = local_v.atan2(local_u);
    let near = add(
        center,
        add(mul(x, radius * angle.cos()), mul(y, radius * angle.sin())),
    );
    push(
        SnapKind::Near,
        Some(root),
        near,
        Some(normal(x, y)),
        Some(angle),
        camera,
        viewport,
        cursor,
        settings.radius_px,
        out,
    )?;

    let Some(from) = from else {
        return Ok(());
    };
    let delta = sub(from, center);
    let u = dot(delta, x);
    let v = dot(delta, y);
    let distance = u.hypot(v);
    if distance > radius {
        let base = v.atan2(u);
        let offset = (radius / distance).acos();
        for sign in [-1.0, 1.0] {
            let angle = base + sign * offset;
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
    if distance > 0. {
        let direction = add(mul(x, u / distance), mul(y, v / distance));
        for sign in [-1.0, 1.0] {
            let point = add(center, mul(direction, sign * radius));
            push(
                SnapKind::Perpendicular,
                Some(root),
                point,
                Some(normal(x, y)),
                None,
                camera,
                viewport,
                cursor,
                settings.radius_px,
                out,
            )?;
        }
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
        });
    }
    Ok(())
}

fn screen_ray_point(camera: &Camera, viewport: Viewport, cursor: [f64; 2]) -> Result<Point3> {
    let ray = camera.ray(cursor, viewport)?;
    if let Ok(Some(point)) = ray.intersect_plane(CPlane::world_xy()) {
        return Ok(point);
    }
    Ok(add(ray.origin, ray.direction))
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

fn closest_on_segment(point: Point3, a: Point3, b: Point3) -> (Point3, f64) {
    let ab = sub(b, a);
    let length = dot(ab, ab);
    if length <= 0. {
        return (a, 0.);
    }
    let t = (dot(sub(point, a), ab) / length).clamp(0., 1.);
    (add(a, mul(ab, t)), t)
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

fn segment_intersection(left: Segment, right: Segment) -> Option<(Point3, f64)> {
    let p = left.a;
    let r = sub(left.b, left.a);
    let q = right.a;
    let s = sub(right.b, right.a);
    let rxs = cross(r, s);
    let denominator = dot(rxs, rxs);
    if denominator <= 1.0e-24 {
        return None;
    }
    let qp = sub(q, p);
    let t = dot(cross(qp, s), rxs) / denominator;
    let u = dot(cross(qp, r), rxs) / denominator;
    if !(0.0..=1.0).contains(&t) || !(0.0..=1.0).contains(&u) {
        return None;
    }
    let left_point = add(p, mul(r, t));
    let right_point = add(q, mul(s, u));
    Some((
        midpoint(left_point, right_point),
        distance(left_point, right_point),
    ))
}

fn segment_length(segment: Segment) -> f64 {
    distance(segment.a, segment.b)
}

fn ray_triangle(
    ray: crate::camera::Ray,
    a: Point3,
    b: Point3,
    c: Point3,
) -> Option<(Point3, Point3)> {
    let edge1 = sub(b, a);
    let edge2 = sub(c, a);
    let h = cross(ray.direction, edge2);
    let determinant = dot(edge1, h);
    if determinant.abs() <= 1.0e-12 {
        return None;
    }
    let inverse = 1.0 / determinant;
    let s = sub(ray.origin, a);
    let u = inverse * dot(s, h);
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = cross(s, edge1);
    let v = inverse * dot(ray.direction, q);
    if v < 0. || u + v > 1.0 {
        return None;
    }
    let t = inverse * dot(edge2, q);
    if t < 0. {
        return None;
    }
    Some((add(ray.origin, mul(ray.direction, t)), normal(edge1, edge2)))
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
