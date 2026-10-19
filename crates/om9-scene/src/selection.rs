// SPDX-License-Identifier: LGPL-2.1-or-later
//! Window/crossing queries against displayed primitives, including near/far clipping.
use crate::{
    camera::{Camera, Viewport, dot, sub},
    scene::{RenderBuffers, Scene},
};
use om9_core::{Error, Result, geometry::Point3, identity::ObjectId};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy)]
pub struct SelectionRect {
    pub min: [f64; 2],
    pub max: [f64; 2],
}
impl SelectionRect {
    fn contains(self, p: [f64; 2]) -> bool {
        (0..2).all(|i| p[i] >= self.min[i] && p[i] <= self.max[i])
    }
    fn segment(self, a: [f64; 2], b: [f64; 2]) -> bool {
        let (mut lo, mut hi) = (0.0_f64, 1.0_f64);
        for i in 0..2 {
            let delta = b[i] - a[i];
            if delta.abs() <= f64::EPSILON {
                if a[i] < self.min[i] || a[i] > self.max[i] {
                    return false;
                }
            } else {
                let t0 = (self.min[i] - a[i]) / delta;
                let t1 = (self.max[i] - a[i]) / delta;
                lo = lo.max(t0.min(t1));
                hi = hi.min(t0.max(t1));
                if lo > hi {
                    return false;
                }
            }
        }
        true
    }
    fn polygon(self, points: &[[f64; 2]]) -> bool {
        if points.iter().any(|p| self.contains(*p)) {
            return true;
        }
        if points.is_empty() {
            return false;
        }
        for i in 0..points.len() {
            if self.segment(points[i], points[(i + 1) % points.len()]) {
                return true;
            }
        }
        // The rectangle may lie entirely inside a face, without meeting an edge or vertex.
        let p = self.min;
        let mut inside = false;
        for i in 0..points.len() {
            let (a, b) = (points[i], points[(i + 1) % points.len()]);
            if (a[1] > p[1]) != (b[1] > p[1])
                && p[0] < (b[0] - a[0]) * (p[1] - a[1]) / (b[1] - a[1]) + a[0]
            {
                inside = !inside;
            }
        }
        inside
    }
}

/// Clip a convex world polygon against one camera depth plane before projection.
fn clip(points: &[Point3], depth: &impl Fn(Point3) -> f64, plane: f64, near: bool) -> Vec<Point3> {
    let mut result = Vec::new();
    if points.is_empty() {
        return result;
    }
    let mut a = *points.last().expect("nonempty");
    let mut da = depth(a) - plane;
    for &b in points {
        let db = depth(b) - plane;
        let (ia, ib) = if near {
            (da >= 0., db >= 0.)
        } else {
            (da <= 0., db <= 0.)
        };
        if ia != ib {
            let t = da / (da - db);
            result.push(std::array::from_fn(|i| a[i] + t * (b[i] - a[i])));
        }
        if ib {
            result.push(b);
        }
        a = b;
        da = db;
    }
    result
}

impl Scene {
    pub fn select_band(
        &self,
        camera: &Camera,
        viewport: Viewport,
        rect: SelectionRect,
        window: bool,
    ) -> Result<Vec<ObjectId>> {
        camera.validate()?;
        viewport.validate()?;
        if !(0..2).all(|i| {
            rect.min[i].is_finite() && rect.max[i].is_finite() && rect.min[i] <= rect.max[i]
        }) {
            return Err(Error::Invalid("selection rectangle"));
        }
        let forward = crate::camera::unit(sub(camera.target, camera.eye))?;
        let depth = |p| dot(sub(p, camera.eye), forward);
        let projected =
            |p| -> Result<Option<[f64; 2]>> { Ok(camera.project(p, viewport)?.map(|p| p.screen)) };
        let intersects = |points: &[Point3]| -> Result<bool> {
            let clipped = clip(
                &clip(points, &depth, camera.near_mm, true),
                &depth,
                camera.far_mm,
                false,
            );
            let mut screen = Vec::with_capacity(clipped.len());
            for p in clipped {
                // Roundoff at an intersection must not drop a vertex on the clipping plane.
                let z = depth(p);
                let correction = z.clamp(camera.near_mm, camera.far_mm) - z;
                let p = std::array::from_fn(|i| p[i] + forward[i] * correction);
                if let Some(p) = projected(p)? {
                    screen.push(p);
                }
            }
            Ok(rect.polygon(&screen))
        };
        let mut selected = BTreeSet::new();
        for object in &self.objects {
            if !object.visible || !object.selectable || !object.has_pick_geometry() {
                continue;
            }
            let transform = object.display_transform()?;
            let points = match object.buffers.as_ref() {
                RenderBuffers::Points { points, .. } | RenderBuffers::Curve { points, .. } => {
                    points
                }
                RenderBuffers::Mesh { vertices, .. } => vertices,
            };
            let mut all = !points.is_empty();
            let mut any = false;
            for p in points {
                let inside = projected(transform.point(*p)?)?.is_some_and(|p| rect.contains(p));
                all &= inside;
                any |= inside;
            }
            let mut hit = if window { all } else { any };
            if !window && !hit {
                match object.buffers.as_ref() {
                    RenderBuffers::Points { .. } => {}
                    RenderBuffers::Curve { points, .. } => {
                        for pair in points.windows(2) {
                            if intersects(&[transform.point(pair[0])?, transform.point(pair[1])?])?
                            {
                                hit = true;
                                break;
                            }
                        }
                    }
                    RenderBuffers::Mesh {
                        vertices,
                        triangles,
                        edges,
                    } => {
                        for (indices, _) in triangles {
                            if intersects(&[
                                transform.point(vertices[indices[0] as usize])?,
                                transform.point(vertices[indices[1] as usize])?,
                                transform.point(vertices[indices[2] as usize])?,
                            ])? {
                                hit = true;
                                break;
                            }
                        }
                        if !hit {
                            for [a, b] in edges {
                                if intersects(&[
                                    transform.point(vertices[*a as usize])?,
                                    transform.point(vertices[*b as usize])?,
                                ])? {
                                    hit = true;
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            if hit {
                selected.insert(object.root);
            }
        }
        Ok(selected.into_iter().collect())
    }
}
