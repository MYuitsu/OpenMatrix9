//! Screen-side gumball: where the handles are on screen, which one the pointer is on, and what a drag
//! means in world space (`OM9-L3-01-01-0xx`, viewport selection and gumball interaction).
//!
//! `om9_core::gumball` owns the transforms; this module owns the conversation between a pointer and that
//! model. It is deliberately free of UI types: the painter reads [`GumballView::segments`], the input
//! layer calls [`GumballView::hit`] and [`GumballView::drag`], and `om9_app` only draws and forwards.
//!
//! The rules mirror the legacy gumball: an axis handle only produces motion along its axis, a plane
//! handle drops the normal component, rotation produces an angle about the handle's axis, and scale
//! produces a factor along it.

use crate::camera::{Camera, Viewport};
use om9_core::{
    Result,
    geometry::Point3,
    gumball::{Axis, Drag, Frame, Handle, Plane},
};

/// Which handle set is shown. Each mode shows its own handles, as the legacy gumball does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Move,
    Rotate,
    Scale,
}

/// Segments drawn for one handle, in world space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Segment {
    pub handle: Handle,
    pub start: Point3,
    pub end: Point3,
}

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

fn normalise(a: Point3) -> Option<Point3> {
    let l = length(a);
    if l <= 1e-12 || !l.is_finite() {
        return None;
    }
    Some(scale(a, 1.0 / l))
}

/// Where the closest point on the line `origin + t·direction` to the ray is, as `t`.
///
/// The two lines are usually skew, so this is the least-squares solution; when they are parallel the ray
/// origin is projected instead, which keeps a drag usable rather than returning `None`.
fn axis_parameter(
    origin: Point3,
    direction: Point3,
    ray_origin: Point3,
    ray_direction: Point3,
) -> Option<f64> {
    let a = dot(direction, direction);
    if a <= 1e-12 {
        return None;
    }
    let b = dot(direction, ray_direction);
    let c = dot(ray_direction, ray_direction);
    let w0 = sub(origin, ray_origin);
    let d = dot(direction, w0);
    let e = dot(ray_direction, w0);
    let determinant = a * c - b * b;
    if determinant.abs() <= 1e-12 {
        // Parallel: the ray gives no depth, so use where it starts relative to the axis.
        return Some(-d / a);
    }
    Some((b * e - c * d) / determinant)
}

/// Intersection of a ray with the plane through `origin` with `normal`.
fn plane_hit(
    origin: Point3,
    normal: Point3,
    ray_origin: Point3,
    ray_direction: Point3,
) -> Option<Point3> {
    let denominator = dot(normal, ray_direction);
    if denominator.abs() <= 1e-12 {
        return None;
    }
    let t = dot(normal, sub(origin, ray_origin)) / denominator;
    if !t.is_finite() {
        return None;
    }
    Some(add(ray_origin, scale(ray_direction, t)))
}

/// The gumball as the viewport sees it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GumballView {
    pub frame: Frame,
    pub mode: Mode,
    /// World length of an axis handle; the rings use it as a radius.
    pub size: f64,
}

impl GumballView {
    pub fn new(frame: Frame, mode: Mode) -> Self {
        Self {
            frame,
            mode,
            size: 1.0,
        }
    }

    pub fn with_size(frame: Frame, mode: Mode, size: f64) -> Self {
        Self {
            frame,
            mode,
            size: size.max(1e-6),
        }
    }

    /// Handles this mode shows, in a stable order.
    pub fn handles(&self) -> Vec<Handle> {
        match self.mode {
            Mode::Move => vec![
                Handle::MoveAxis(Axis::X),
                Handle::MoveAxis(Axis::Y),
                Handle::MoveAxis(Axis::Z),
                Handle::MovePlane(Plane::XY),
                Handle::MovePlane(Plane::YZ),
                Handle::MovePlane(Plane::ZX),
                Handle::MoveFree,
            ],
            Mode::Rotate => vec![
                Handle::Rotate(Axis::X),
                Handle::Rotate(Axis::Y),
                Handle::Rotate(Axis::Z),
            ],
            Mode::Scale => vec![
                Handle::Scale(Axis::X),
                Handle::Scale(Axis::Y),
                Handle::Scale(Axis::Z),
            ],
        }
    }

    /// The handle's screen-space geometry, as world segments the painter turns into lines.
    pub fn segments(&self) -> Vec<Segment> {
        let mut segments = Vec::new();
        match self.mode {
            Mode::Move | Mode::Scale => {
                for axis in [Axis::X, Axis::Y, Axis::Z] {
                    let direction = self.frame.axis(axis);
                    let handle = if self.mode == Mode::Move {
                        Handle::MoveAxis(axis)
                    } else {
                        Handle::Scale(axis)
                    };
                    // Move arrows run along +axis. The scale leader is one dotted line across the whole axis,
                    // from the arrow tip through the pivot to the box on the negative side, which is how the
                    // Matrix 8 figure draws it.
                    let (from, to) = if self.mode == Mode::Move {
                        (
                            self.frame.origin,
                            add(self.frame.origin, scale(direction, self.size)),
                        )
                    } else {
                        (
                            add(self.frame.origin, scale(direction, self.size)),
                            add(self.frame.origin, scale(direction, -self.size)),
                        )
                    };
                    segments.push(Segment {
                        handle,
                        start: from,
                        end: to,
                    });
                }
                if self.mode == Mode::Move {
                    for plane in [Plane::XY, Plane::YZ, Plane::ZX] {
                        let (u, v) = self.frame.plane_basis(plane);
                        // The plane indicator sits in the positive quadrant between its two axes.
                        let corner = add(
                            self.frame.origin,
                            add(scale(u, self.size * 0.32), scale(v, self.size * 0.32)),
                        );
                        segments.push(Segment {
                            handle: Handle::MovePlane(plane),
                            start: corner,
                            end: corner,
                        });
                    }
                }
            }
            Mode::Rotate => {
                // Quarter arcs on the negative side of their two axes, so they bow out behind the arrows
                // instead of crossing them, which is the gumball the Matrix 8 manual draws.
                for (axis, first, second) in [
                    (Axis::X, Axis::Y, Axis::Z),
                    (Axis::Y, Axis::Z, Axis::X),
                    (Axis::Z, Axis::X, Axis::Y),
                ] {
                    let u = scale(self.frame.axis(first), -1.0);
                    let v = scale(self.frame.axis(second), -1.0);
                    let radius = self.size * 0.6;
                    let steps = 24;
                    for step in 0..steps {
                        let start = step as f64 / steps as f64 * std::f64::consts::FRAC_PI_2;
                        let end = (step + 1) as f64 / steps as f64 * std::f64::consts::FRAC_PI_2;
                        let point = |angle: f64| {
                            add(
                                self.frame.origin,
                                add(
                                    scale(u, radius * angle.cos()),
                                    scale(v, radius * angle.sin()),
                                ),
                            )
                        };
                        segments.push(Segment {
                            handle: Handle::Rotate(axis),
                            start: point(start),
                            end: point(end),
                        });
                    }
                }
            }
        }
        segments
    }

    /// The segments that make sense in this view.
    ///
    /// A handle whose axis points at the camera projects to a dot, so its arrow and its scale box would pile
    /// up on the pivot: a single (parallel) view must show two boxes, not three. Plane indicators are kept
    /// only when the plane faces the camera, which is why the top view shows one grid icon.
    pub fn visible_segments(&self, camera: &Camera, viewport: Viewport) -> Vec<Segment> {
        let Some(direction) = self.view_direction(camera) else {
            return self.segments();
        };
        let mut kept = Vec::new();
        for segment in self.segments() {
            let keep = match segment.handle {
                Handle::MoveAxis(axis) | Handle::Scale(axis) => {
                    // Hide the axis that points at the camera (its box would sit on the pivot).
                    dot(self.frame.axis(axis), direction).abs() < 0.93
                }
                Handle::MovePlane(plane) => {
                    // Keep the indicator that faces the camera, drop the ones seen edge-on.
                    dot(self.frame.plane_normal(plane), direction).abs() > 0.5
                }
                Handle::Rotate(axis) => {
                    // An arc seen edge-on collapses to a line; drop it when it is shorter than a few pixels.
                    dot(self.frame.axis(axis), direction).abs() > 0.15
                        && self
                            .screen_point(camera, viewport, segment.start)
                            .ok()
                            .flatten()
                            .zip(
                                self.screen_point(camera, viewport, segment.end)
                                    .ok()
                                    .flatten(),
                            )
                            .is_some_and(|(start, end)| {
                                (start[0] - end[0]).hypot(start[1] - end[1]) >= 1.0
                            })
                }
                Handle::MoveFree => true,
            };
            if keep {
                kept.push(segment);
            }
        }
        kept
    }

    /// The scale box under the pointer, within `radius` pixels of its centre.
    ///
    /// The dotted leader runs the whole axis, so a distance-to-segment test cannot tell the box from the
    /// arrow; the box is the small square at the negative end and deserves its own, precise hit test.
    pub fn box_at(
        &self,
        camera: &Camera,
        viewport: Viewport,
        pointer: [f64; 2],
        radius: f64,
    ) -> Result<Option<Handle>> {
        let mut best: Option<(f64, Handle)> = None;
        for segment in self.visible_segments(camera, viewport) {
            if !matches!(segment.handle, Handle::Scale(_)) {
                continue;
            }
            let Some(centre) = self.screen_point(camera, viewport, segment.end)? else {
                continue;
            };
            let distance = (pointer[0] - centre[0]).hypot(pointer[1] - centre[1]);
            if distance <= radius && best.is_none_or(|(best, _)| distance < best) {
                best = Some((distance, segment.handle));
            }
        }
        Ok(best.map(|(_, handle)| handle))
    }

    /// Hit testing restricted to what the view actually draws, so an invisible handle cannot be grabbed.
    pub fn nearest_in_view(
        &self,
        camera: &Camera,
        viewport: Viewport,
        pointer: [f64; 2],
        tolerance_px: f64,
    ) -> Result<Option<(f64, Handle)>> {
        let mut best: Option<(f64, Handle)> = None;
        if self.mode == Mode::Move
            && let Some(centre) = self.screen_point(camera, viewport, self.frame.origin)?
        {
            let d = (pointer[0] - centre[0]).hypot(pointer[1] - centre[1]);
            if d <= tolerance_px.min(5.0) {
                return Ok(Some((d, Handle::MoveFree)));
            }
        }
        for segment in self.visible_segments(camera, viewport) {
            let Some(start) = self.screen_point(camera, viewport, segment.start)? else {
                continue;
            };
            let Some(end) = self.screen_point(camera, viewport, segment.end)? else {
                continue;
            };
            let distance = if matches!(segment.handle, Handle::Scale(_)) {
                (pointer[0] - end[0]).hypot(pointer[1] - end[1])
            } else {
                point_to_segment(pointer, start, end)
            };
            if distance <= tolerance_px && best.is_none_or(|(best, _)| distance < best) {
                best = Some((distance, segment.handle));
            }
        }
        Ok(best)
    }

    /// Unit vector from the camera towards the pivot, the direction a handle pointing at the viewer has.
    fn view_direction(&self, camera: &Camera) -> Option<Point3> {
        normalise(sub(self.frame.origin, camera.eye))
            .or_else(|| normalise(sub(camera.target, camera.eye)))
    }

    /// Projects a handle to screen space; `None` when it is behind the camera.
    pub fn screen_point(
        &self,
        camera: &Camera,
        viewport: Viewport,
        point: Point3,
    ) -> Result<Option<[f64; 2]>> {
        Ok(camera
            .project(point, viewport)?
            .map(|projected| projected.screen))
    }

    /// Distance from the pointer to the nearest of this view's handles, with the handle itself.
    ///
    /// The Matrix9 gumball shows move, rotate and scale handles at the same time, so the application needs
    /// to know which group is nearest instead of asking one mode at a time.
    pub fn nearest(
        &self,
        camera: &Camera,
        viewport: Viewport,
        pointer: [f64; 2],
        tolerance_px: f64,
    ) -> Result<Option<(f64, Handle)>> {
        let mut best: Option<(f64, Handle)> = None;
        for segment in self.segments() {
            let Some(start) = self.screen_point(camera, viewport, segment.start)? else {
                continue;
            };
            let Some(end) = self.screen_point(camera, viewport, segment.end)? else {
                continue;
            };
            let distance = point_to_segment(pointer, start, end);
            if distance <= tolerance_px
                && best.is_none_or(|(best_distance, _)| distance < best_distance)
            {
                best = Some((distance, segment.handle));
            }
        }
        Ok(best)
    }

    /// The handle under the pointer, if any. Hit testing runs in screen space against the segments that
    /// are actually drawn, so what the user sees is what they can grab.
    pub fn hit(
        &self,
        camera: &Camera,
        viewport: Viewport,
        pointer: [f64; 2],
        tolerance_px: f64,
    ) -> Result<Option<Handle>> {
        let mut best: Option<(f64, Handle)> = None;
        for segment in self.segments() {
            let Some(start) = self.screen_point(camera, viewport, segment.start)? else {
                continue;
            };
            let Some(end) = self.screen_point(camera, viewport, segment.end)? else {
                continue;
            };
            let distance = point_to_segment(pointer, start, end);
            if distance <= tolerance_px
                && best.is_none_or(|(best_distance, _)| distance < best_distance)
            {
                best = Some((distance, segment.handle));
            }
        }
        Ok(best.map(|(_, handle)| handle))
    }

    /// Turns the pointer motion into the drag the core model consumes.
    pub fn drag(
        &self,
        camera: &Camera,
        viewport: Viewport,
        handle: Handle,
        from: [f64; 2],
        to: [f64; 2],
    ) -> Result<Option<Drag>> {
        let start_ray = camera.ray(from, viewport)?;
        let end_ray = camera.ray(to, viewport)?;
        let origin = self.frame.origin;
        let drag = match handle {
            Handle::MoveAxis(axis) => {
                let direction = self.frame.axis(axis);
                let Some(start) =
                    axis_parameter(origin, direction, start_ray.origin, start_ray.direction)
                else {
                    return Ok(None);
                };
                let Some(end) =
                    axis_parameter(origin, direction, end_ray.origin, end_ray.direction)
                else {
                    return Ok(None);
                };
                Drag::WorldDelta(scale(direction, end - start))
            }
            Handle::MovePlane(plane) => {
                let normal = self.frame.plane_normal(plane);
                let Some(start) = plane_hit(origin, normal, start_ray.origin, start_ray.direction)
                else {
                    return Ok(None);
                };
                let Some(end) = plane_hit(origin, normal, end_ray.origin, end_ray.direction) else {
                    return Ok(None);
                };
                Drag::WorldDelta(sub(end, start))
            }
            Handle::MoveFree => {
                // Screen-parallel motion: the plane faces the camera through the pivot.
                let Some(forward) = normalise(sub(camera.target, camera.eye)) else {
                    return Ok(None);
                };
                let Some(start) = plane_hit(origin, forward, start_ray.origin, start_ray.direction)
                else {
                    return Ok(None);
                };
                let Some(end) = plane_hit(origin, forward, end_ray.origin, end_ray.direction)
                else {
                    return Ok(None);
                };
                Drag::WorldDelta(sub(end, start))
            }
            Handle::Rotate(axis) => {
                let normal = self.frame.axis(axis);
                let Some(start) = plane_hit(origin, normal, start_ray.origin, start_ray.direction)
                else {
                    return Ok(None);
                };
                let Some(end) = plane_hit(origin, normal, end_ray.origin, end_ray.direction) else {
                    return Ok(None);
                };
                let v0 = sub(start, origin);
                let v1 = sub(end, origin);
                if length(v0) <= 1e-9 || length(v1) <= 1e-9 {
                    return Ok(None);
                }
                let angle = dot(cross(v0, v1), normal).atan2(dot(v0, v1));
                Drag::Angle(angle)
            }
            Handle::Scale(axis) => {
                let direction = self.frame.axis(axis);
                let Some(start) =
                    axis_parameter(origin, direction, start_ray.origin, start_ray.direction)
                else {
                    return Ok(None);
                };
                if start.abs() <= 1e-9 {
                    return Ok(None);
                }
                let Some(end) =
                    axis_parameter(origin, direction, end_ray.origin, end_ray.direction)
                else {
                    return Ok(None);
                };
                let factor = end / start;
                if !factor.is_finite() || factor <= 0.0 {
                    return Ok(None);
                }
                Drag::Factor(factor)
            }
        };
        Ok(Some(drag))
    }
}

/// Any orthonormal pair spanning the plane with `normal`. Kept for callers that need a ring basis.
#[allow(dead_code)]
fn basis_for(normal: Point3) -> (Point3, Point3) {
    let helper = if normal[0].abs() < 0.9 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    let u = normalise(cross(normal, helper)).unwrap_or([0.0, 1.0, 0.0]);
    let v = normalise(cross(normal, u)).unwrap_or([1.0, 0.0, 0.0]);
    (u, v)
}

/// Distance from a point to a segment, in screen pixels.
fn point_to_segment(point: [f64; 2], start: [f64; 2], end: [f64; 2]) -> f64 {
    let ab = [end[0] - start[0], end[1] - start[1]];
    let ap = [point[0] - start[0], point[1] - start[1]];
    let length_squared = ab[0] * ab[0] + ab[1] * ab[1];
    if length_squared <= 1e-12 {
        return (ap[0] * ap[0] + ap[1] * ap[1]).sqrt();
    }
    let t = ((ap[0] * ab[0] + ap[1] * ab[1]) / length_squared).clamp(0.0, 1.0);
    let closest = [start[0] + ab[0] * t, start[1] + ab[1] * t];
    ((point[0] - closest[0]).powi(2) + (point[1] - closest[1]).powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::camera::{Camera, Viewport};

    fn viewport() -> Viewport {
        Viewport {
            origin: [0.0, 0.0],
            size: [800.0, 600.0],
        }
    }

    fn camera() -> Camera {
        Camera::perspective(
            [0.0, 0.0, 10.0],
            [0.0; 3],
            [0.0, 1.0, 0.0],
            45f64.to_radians(),
        )
        .expect("camera")
    }

    fn screen_of(camera: &Camera, point: Point3) -> [f64; 2] {
        camera
            .project(point, viewport())
            .expect("project")
            .expect("in front")
            .screen
    }

    #[test]
    fn an_axis_handle_is_hit_where_it_is_drawn() {
        let view = GumballView::new(Frame::world(), Mode::Move);
        let camera = camera();
        let pointer = screen_of(&camera, [0.5, 0.0, 0.0]);
        let hit = view
            .hit(&camera, viewport(), pointer, 6.0)
            .expect("hit test");
        assert_eq!(hit, Some(Handle::MoveAxis(Axis::X)));
    }

    #[test]
    fn empty_space_is_not_a_handle() {
        let view = GumballView::new(Frame::world(), Mode::Move);
        let camera = camera();
        let hit = view
            .hit(&camera, viewport(), [790.0, 590.0], 6.0)
            .expect("hit test");
        assert_eq!(hit, None);
    }

    #[test]
    fn dragging_an_axis_handle_moves_only_along_that_axis() {
        let view = GumballView::new(Frame::world(), Mode::Move);
        let camera = camera();
        let from = screen_of(&camera, [0.5, 0.0, 0.0]);
        let to = screen_of(&camera, [1.5, 0.0, 0.0]);
        let drag = view
            .drag(&camera, viewport(), Handle::MoveAxis(Axis::X), from, to)
            .expect("drag")
            .expect("some drag");
        let Drag::WorldDelta(delta) = drag else {
            panic!("expected a translation, got {drag:?}")
        };
        assert!((delta[0] - 1.0).abs() < 1e-6, "x {delta:?}");
        assert!(
            delta[1].abs() < 1e-6 && delta[2].abs() < 1e-6,
            "no side motion: {delta:?}"
        );
    }

    #[test]
    fn dragging_a_plane_handle_keeps_the_normal_component() {
        let view = GumballView::new(Frame::world(), Mode::Move);
        let camera = camera();
        let from = screen_of(&camera, [0.2, 0.2, 0.0]);
        let to = screen_of(&camera, [0.7, 0.4, 0.0]);
        let drag = view
            .drag(&camera, viewport(), Handle::MovePlane(Plane::XY), from, to)
            .expect("drag")
            .expect("some drag");
        let Drag::WorldDelta(delta) = drag else {
            panic!("expected a translation")
        };
        assert!(
            delta[2].abs() < 1e-6,
            "no z motion out of the plane: {delta:?}"
        );
        assert!(
            delta[0] > 0.4 && delta[1] > 0.1,
            "in-plane motion: {delta:?}"
        );
    }

    #[test]
    fn rotating_about_z_reports_the_screen_angle() {
        let view = GumballView::new(Frame::world(), Mode::Rotate);
        let camera = camera();
        let from = screen_of(&camera, [1.0, 0.0, 0.0]);
        let to = screen_of(&camera, [0.0, 1.0, 0.0]);
        let drag = view
            .drag(&camera, viewport(), Handle::Rotate(Axis::Z), from, to)
            .expect("drag")
            .expect("some drag");
        let Drag::Angle(angle) = drag else {
            panic!("expected an angle")
        };
        assert!(
            (angle - std::f64::consts::FRAC_PI_2).abs() < 1e-6,
            "angle {angle}"
        );
    }

    #[test]
    fn scaling_reports_the_ratio_along_the_axis() {
        let view = GumballView::new(Frame::world(), Mode::Scale);
        let camera = camera();
        let from = screen_of(&camera, [1.0, 0.0, 0.0]);
        let to = screen_of(&camera, [2.0, 0.0, 0.0]);
        let drag = view
            .drag(&camera, viewport(), Handle::Scale(Axis::X), from, to)
            .expect("drag")
            .expect("some drag");
        let Drag::Factor(factor) = drag else {
            panic!("expected a factor")
        };
        assert!((factor - 2.0).abs() < 1e-6, "factor {factor}");
    }

    #[test]
    fn an_orthographic_viewport_hits_the_same_handles() {
        let view = GumballView::new(Frame::world(), Mode::Move);
        let camera = Camera::orthographic([0.0, 0.0, 10.0], [0.0; 3], [0.0, 1.0, 0.0], 10.0)
            .expect("camera");
        let pointer = screen_of(&camera, [0.5, 0.0, 0.0]);
        let hit = view
            .hit(&camera, viewport(), pointer, 6.0)
            .expect("hit test");
        assert_eq!(hit, Some(Handle::MoveAxis(Axis::X)));
    }

    #[test]
    fn every_mode_exposes_the_handles_it_draws() {
        assert_eq!(
            GumballView::new(Frame::world(), Mode::Move).handles().len(),
            7
        );
        assert_eq!(
            GumballView::new(Frame::world(), Mode::Rotate)
                .handles()
                .len(),
            3
        );
        assert_eq!(
            GumballView::new(Frame::world(), Mode::Scale)
                .handles()
                .len(),
            3
        );
        let segments = GumballView::new(Frame::world(), Mode::Rotate).segments();
        assert_eq!(
            segments.len(),
            3 * 24,
            "three quarter arcs of twenty-four segments each"
        );
    }
}

#[cfg(test)]
mod view_tests {
    use super::*;

    fn frame() -> Frame {
        Frame {
            origin: [0.0, 0.0, 0.0],
            x: [1.0, 0.0, 0.0],
            y: [0.0, 1.0, 0.0],
            z: [0.0, 0.0, 1.0],
        }
    }

    fn viewport() -> Viewport {
        Viewport {
            origin: [0.0, 0.0],
            size: [400.0, 400.0],
        }
    }

    /// Looking straight down the Z axis, as the top view does.
    fn top_view() -> Camera {
        Camera::orthographic([0.0, 0.0, 100.0], [0.0; 3], [0.0, 1.0, 0.0], 40.0).expect("camera")
    }

    fn perspective() -> Camera {
        Camera::perspective(
            [35.0, -35.0, 35.0],
            [0.0; 3],
            [0.0, 0.0, 1.0],
            35f64.to_radians(),
        )
        .expect("camera")
    }

    #[test]
    fn a_single_view_hides_the_axis_pointing_at_the_camera() {
        let gumball = GumballView::with_size(frame(), Mode::Move, 10.0);
        let kept = gumball.visible_segments(&top_view(), viewport());
        let axes: Vec<Axis> = kept
            .iter()
            .filter_map(|segment| match segment.handle {
                Handle::MoveAxis(axis) => Some(axis),
                _ => None,
            })
            .collect();
        assert_eq!(
            axes,
            vec![Axis::X, Axis::Y],
            "no Z arrow when looking down Z"
        );
        let planes: Vec<Plane> = kept
            .iter()
            .filter_map(|segment| match segment.handle {
                Handle::MovePlane(plane) => Some(plane),
                _ => None,
            })
            .collect();
        assert_eq!(
            planes,
            vec![Plane::XY],
            "only the plane that faces the camera"
        );
    }

    #[test]
    fn a_single_view_keeps_two_scale_boxes_not_three() {
        let gumball = GumballView::with_size(frame(), Mode::Scale, 10.0);
        let boxes: Vec<Axis> = gumball
            .visible_segments(&top_view(), viewport())
            .iter()
            .filter_map(|segment| match segment.handle {
                Handle::Scale(axis) => Some(axis),
                _ => None,
            })
            .collect();
        assert_eq!(boxes, vec![Axis::X, Axis::Y], "two boxes in the top view");
    }

    #[test]
    fn a_perspective_view_keeps_all_three_axes() {
        let gumball = GumballView::with_size(frame(), Mode::Move, 10.0);
        let axes = gumball
            .visible_segments(&perspective(), viewport())
            .iter()
            .filter(|segment| matches!(segment.handle, Handle::MoveAxis(_)))
            .count();
        assert_eq!(axes, 3);
    }

    #[test]
    fn a_hidden_handle_cannot_be_grabbed() {
        let gumball = GumballView::with_size(frame(), Mode::Scale, 10.0);
        let camera = top_view();
        // The Z box sits on the pivot in this view; a click right on the pivot must not take it.
        let hit = gumball
            .nearest_in_view(&camera, viewport(), [200.0, 200.0], 8.0)
            .expect("hit test");
        assert!(
            hit.is_none_or(|(_, handle)| handle != Handle::Scale(Axis::Z)),
            "the invisible Z box is not grabbable: {hit:?}"
        );
    }
}

#[cfg(test)]
mod box_tests {
    use super::*;
    use crate::camera::Projection;

    #[test]
    fn a_scale_box_is_hit_and_a_drag_of_it_scales() {
        let frame = Frame {
            origin: [0.0, 0.0, 0.0],
            x: [1.0, 0.0, 0.0],
            y: [0.0, 1.0, 0.0],
            z: [0.0, 0.0, 1.0],
        };
        let camera = Camera {
            eye: [35.0, -35.0, 35.0],
            target: [0.0; 3],
            up: [0.0, 0.0, 1.0],
            projection: Projection::Perspective {
                vertical_fov_rad: 35f64.to_radians(),
            },
            near_mm: 0.1,
            far_mm: 1000.0,
            revision: 1,
        };
        let viewport = Viewport {
            origin: [0.0, 0.0],
            size: [480.0, 260.0],
        };
        let gumball = GumballView::with_size(frame, Mode::Scale, 10.0);
        let box_screen = gumball
            .visible_segments(&camera, viewport)
            .into_iter()
            .find(|segment| segment.handle == Handle::Scale(Axis::X))
            .and_then(|segment| {
                gumball
                    .screen_point(&camera, viewport, segment.end)
                    .ok()
                    .flatten()
            })
            .expect("the X box is drawn");
        let hit = gumball
            .box_at(&camera, viewport, box_screen, 8.0)
            .expect("hit test")
            .expect("the box is under the pointer");
        assert_eq!(hit, Handle::Scale(Axis::X));
        // Dragging the box from its position outwards doubles the factor.
        let outer = gumball
            .screen_point(&camera, viewport, [-20.0, 0.0, 0.0])
            .ok()
            .flatten()
            .expect("projection");
        let drag = gumball
            .drag(&camera, viewport, hit, box_screen, outer)
            .expect("drag")
            .expect("a factor");
        let Drag::Factor(factor) = drag else {
            panic!("expected a factor, got {drag:?}");
        };
        assert!((factor - 2.0).abs() < 0.05, "factor {factor}");
    }
}
