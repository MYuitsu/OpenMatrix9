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
    geometry::{Curve, Geometry, Point3, Transform, finite},
    identity::{ObjectId, SourceStamp},
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
    /// Source of a B-rep surface candidate. Its point and normal are display-mesh
    /// approximations until an owning kernel verifies the same source and face.
    pub surface_source: Option<SurfaceCandidateSource>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SurfaceCandidateSource {
    pub path: Vec<ObjectId>,
    pub leaf: ObjectId,
    pub source: SourceStamp,
    pub scene_epoch: u64,
    pub placement: Transform,
    pub camera: Camera,
    pub viewport: Viewport,
    pub cursor: [f64; 2],
    /// None when the retained native display does not provide a B-rep face binding.
    pub face: Option<uuid::Uuid>,
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
            surface_source: None,
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
    choose_iter(candidates.iter())
}

/// Select without building another candidate array; preserves iterator order for ties.
pub fn choose_iter<'a>(candidates: impl Iterator<Item = &'a SnapCandidate>) -> Option<SnapCandidate> {
    candidates.min_by(|left, right| snap_order(left, right)).cloned()
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
        let object = &scene.objects[index];
        if target_source.is_some_and(|source| object.root == source.source.object && object.path == source.path && object.leaf == source.leaf) {
            continue;
        }
        if !object.visible || excluded.contains(&object.root)
            || !matches!(scene.entity_for(object)?.geometry, Geometry::Mesh(_) | Geometry::Hatch(_) | Geometry::Surface(_) | Geometry::Brep(_) | Geometry::NativeBrep(_)) {
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
        .then(left.root.cmp(&right.root))
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
    let mut surface_broad_phase = None;
    // The pointer ray and one a snap radius beside it, for the NURBS broad phase (made once).
    let mut cull_rays: Option<(crate::camera::Ray, crate::camera::Ray)> = None;
    for (object_index, object) in scene.objects.iter().enumerate() {
        if !object.visible || !(object.selectable || object.reference_snappable) {
            continue;
        }
        let transform = object.display_transform()?;
        let entity = scene.entity_for(object)?;
        if matches!(entity.geometry, Geometry::Brep(_) | Geometry::NativeBrep(_) | Geometry::Surface(_)) {
            if !settings.is_enabled(SnapKind::Surface) { continue; }
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
            Geometry::Curve(Curve::Nurbs(curve)) => {
                if let crate::scene::RenderBuffers::Curve { points, .. } = object.buffers.as_ref() {
                    if cull_rays.is_none() {
                        cull_rays = Some((
                            camera.ray(cursor, viewport)?,
                            camera.ray([cursor[0] + settings.radius_px, cursor[1]], viewport)?,
                        ));
                    }
                    nurbs_snaps(
                        curve, object.root, transform, points, camera, viewport, cursor, settings,
                        from, cull_rays.as_ref().expect("set above"), &mut out, &mut segments,
                    )?;
                }
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
                let ray = surface_broad_phase.as_ref().expect("initialized for surfaces").0;
                let mut best = std::collections::BTreeMap::<u32, (f64, Point3, Point3, Option<uuid::Uuid>)>::new();
                for triangle in &brep.display.triangles {
                    let a = transform.point(brep.display.points[triangle[0] as usize])?;
                    let b = transform.point(brep.display.points[triangle[1] as usize])?;
                    let c = transform.point(brep.display.points[triangle[2] as usize])?;
                    if let Some((point, normal)) = ray_triangle(ray, a, b, c) {
                        let distance = distance(camera.eye, point);
                        if best.get(&triangle[3]).is_none_or(|(old, _, _, _)| distance < *old) {
                            best.insert(triangle[3], (distance, point, normal, brep.display.faces.get(triangle[3] as usize).copied()));
                        }
                    }
                }
                for (_, (_, point, normal, face)) in best {
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
                            path: object.path.clone(), leaf: object.leaf, source: object.source, scene_epoch: scene.epoch,
                            placement: transform, camera: camera.clone(), viewport, cursor, face,
                        });
                    }
                }
            }
            Geometry::NativeBrep(_) => {
                let ray = surface_broad_phase.as_ref().expect("initialized for surfaces").0;
                let mut best: Option<(f64, Point3, Point3)> = None;
                // Use the same triangulation as the displayed image, including
                // the fan diagonal chosen for quad faces.
                let crate::scene::RenderBuffers::Mesh { vertices, triangles, .. } = object.buffers.as_ref() else { continue };
                for (triangle, _) in triangles {
                    let a = transform.point(vertices[triangle[0] as usize])?;
                    let b = transform.point(vertices[triangle[1] as usize])?;
                    let c = transform.point(vertices[triangle[2] as usize])?;
                    if let Some((point, normal)) = ray_triangle(ray, a, b, c) {
                        let distance = distance(camera.eye, point);
                        if best.is_none_or(|(old, _, _)| distance < old) {
                            best = Some((distance, point, normal));
                        }
                    }
                }
                if let Some((_, point, normal)) = best {
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
                            path: object.path.clone(), leaf: object.leaf, source: object.source, scene_epoch: scene.epoch,
                            placement: transform, camera: camera.clone(), viewport, cursor, face: None,
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
                            path: object.path.clone(), leaf: object.leaf, source: object.source, scene_epoch: scene.epoch,
                            placement: transform, camera: camera.clone(), viewport, cursor, face: None,
                        });
                    }
                }
            }
            _ => {}
        }
    }
    // An Intersection candidate must lie within the snap radius of the pointer, and it lies on
    // both of its segments: only segments passing that close on screen can form one. Testing
    // just those keeps the pair test small in a large file (every pair of every curve was
    // tested on each pointer move). A segment that cannot be projected is kept.
    let screen = |p: Point3| camera.project(p, viewport).ok().flatten().map(|v| v.screen);
    let segments: Vec<Segment> = segments
        .into_iter()
        .filter(|segment| match (screen(segment.a), screen(segment.b)) {
            (Some(a), Some(b)) => {
                let ab = [b[0] - a[0], b[1] - a[1]];
                let length = ab[0] * ab[0] + ab[1] * ab[1];
                let t = if length > 0. {
                    (((cursor[0] - a[0]) * ab[0] + (cursor[1] - a[1]) * ab[1]) / length).clamp(0., 1.)
                } else {
                    0.
                };
                (a[0] + ab[0] * t - cursor[0]).hypot(a[1] + ab[1] * t - cursor[1])
                    <= settings.radius_px + 1.
            }
            _ => true,
        })
        .collect();
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
    out.sort_by(snap_order);
    out.dedup_by(|left, right| {
        left.kind == right.kind
            && left.root == right.root
            && left.parameter == right.parameter
            && (left.kind != SnapKind::Surface || left.surface_source == right.surface_source)
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

/// Snaps on a NURBS curve, as Rhino gives them: End at both ends (every vertex of a degree-1
/// curve, i.e. a polyline), Mid (each segment of a polyline, else half the length), Near and
/// Perpendicular along it. Positions along the curve use its display polyline (the exact
/// evaluation at the samples); the ends and polyline vertices are exact. A curve whose control
/// hull is far from the pointer ray is skipped before any sample is looked at.
#[allow(clippy::too_many_arguments)]
fn nurbs_snaps(
    curve: &om9_core::geometry::NurbsCurve,
    root: ObjectId,
    transform: Transform,
    display: &[Point3],
    camera: &Camera,
    viewport: Viewport,
    cursor: [f64; 2],
    settings: SnapSettings,
    from: Option<Point3>,
    (ray, side): &(crate::camera::Ray, crate::camera::Ray),
    out: &mut Vec<SnapCandidate>,
    segments: &mut Vec<Segment>,
) -> Result<()> {
    if display.len() < 2 || curve.poles.is_empty() {
        return Ok(());
    }
    let poles: Vec<Point3> = curve
        .poles
        .iter()
        .map(|p| transform.point(*p))
        .collect::<Result<_>>()?;
    // Broad phase: the curve lies in the convex hull of its poles (positive weights), so a
    // sphere around their box that misses the pointer ray by more than the snap radius cannot
    // hold a candidate (every candidate, Perpendicular included, must be near the pointer).
    if curve.weights.iter().all(|w| *w > 0.) {
        let (mut lo, mut hi) = (poles[0], poles[0]);
        for p in &poles {
            for axis in 0..3 {
                lo[axis] = lo[axis].min(p[axis]);
                hi[axis] = hi[axis].max(p[axis]);
            }
        }
        let center = midpoint(lo, hi);
        let radius = distance(lo, hi) * 0.5;
        let off_ray = |r: &crate::camera::Ray, p: Point3| -> Point3 {
            let d = r.direction;
            let len = dot(d, d);
            let t = if len > 0. { dot(sub(p, r.origin), d) / len } else { 0. };
            add(r.origin, [d[0] * t, d[1] * t, d[2] * t])
        };
        let near = off_ray(ray, center);
        let pixel_radius = distance(near, off_ray(side, center));
        if pixel_radius.is_finite() && distance(near, center) > radius + 2. * pixel_radius + 1e-9 {
            return Ok(());
        }
    }
    let world: Vec<Point3> = display
        .iter()
        .map(|p| transform.point(*p))
        .collect::<Result<_>>()?;
    let (first, last) = (world[0], world[world.len() - 1]);
    let put = |kind, point, parameter: f64, out: &mut Vec<SnapCandidate>| {
        push(kind, Some(root), point, None, Some(parameter), camera, viewport, cursor, settings.radius_px, out)
    };
    put(SnapKind::End, first, 0., out)?;
    if distance(first, last) > 1e-9 {
        put(SnapKind::End, last, 1., out)?;
    }
    if curve.degree == 1 {
        // A polyline: its vertices are ends and each segment has a middle.
        let count = (poles.len() - 1).max(1) as f64;
        for (index, pole) in poles.iter().enumerate() {
            put(SnapKind::End, *pole, index as f64 / count, out)?;
        }
        for (index, pair) in poles.windows(2).enumerate() {
            put(SnapKind::Mid, midpoint(pair[0], pair[1]), (index as f64 + 0.5) / count, out)?;
        }
    } else {
        let total: f64 = world.windows(2).map(|w| distance(w[0], w[1])).sum();
        let mut walked = 0.;
        for pair in world.windows(2) {
            let step = distance(pair[0], pair[1]);
            if walked + step >= total * 0.5 && step > 0. {
                let t = (total * 0.5 - walked) / step;
                let mid = add(pair[0], [
                    (pair[1][0] - pair[0][0]) * t,
                    (pair[1][1] - pair[0][1]) * t,
                    (pair[1][2] - pair[0][2]) * t,
                ]);
                put(SnapKind::Mid, mid, 0.5, out)?;
                break;
            }
            walked += step;
        }
    }
    // Near: the display segment closest to the pointer on screen; segments near the pointer
    // also take part in Intersection snaps (only those, so the pair test stays small).
    let screen = |p: Point3| -> Option<[f64; 2]> { camera.project(p, viewport).ok().flatten().map(|v| v.screen) };
    let reach = settings.radius_px * 2.;
    let mut best: Option<(f64, Point3, f64)> = None;
    let mut previous = screen(world[0]);
    let count = (world.len() - 1) as f64;
    for (index, pair) in world.windows(2).enumerate() {
        let next = screen(pair[1]);
        if let (Some(a), Some(b)) = (previous, next) {
            let ab = [b[0] - a[0], b[1] - a[1]];
            let length = ab[0] * ab[0] + ab[1] * ab[1];
            let t = if length > 0. {
                (((cursor[0] - a[0]) * ab[0] + (cursor[1] - a[1]) * ab[1]) / length).clamp(0., 1.)
            } else {
                0.
            };
            let gap = (a[0] + ab[0] * t - cursor[0]).hypot(a[1] + ab[1] * t - cursor[1]);
            if gap <= reach {
                segments.push(Segment { root, a: pair[0], b: pair[1] });
            }
            if best.is_none_or(|(old, _, _)| gap < old) {
                let point = add(pair[0], [
                    (pair[1][0] - pair[0][0]) * t,
                    (pair[1][1] - pair[0][1]) * t,
                    (pair[1][2] - pair[0][2]) * t,
                ]);
                best = Some((gap, point, (index as f64 + t) / count));
            }
        }
        previous = next;
    }
    if let Some((_, point, parameter)) = best {
        put(SnapKind::Near, point, parameter, out)?;
    }
    if let Some(from) = from {
        let mut foot: Option<(f64, Point3, f64)> = None;
        for (index, pair) in world.windows(2).enumerate() {
            let (point, t) = closest_on_segment(from, pair[0], pair[1]);
            // Only a foot inside a segment (or at a smooth joint) is perpendicular.
            let interior = t > 0. && t < 1.;
            let d = distance(from, point);
            if interior && foot.is_none_or(|(old, _, _)| d < old) {
                foot = Some((d, point, (index as f64 + t) / count));
            }
        }
        if let Some((_, point, parameter)) = foot {
            put(SnapKind::Perpendicular, point, parameter, out)?;
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
            surface_source: None,
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
