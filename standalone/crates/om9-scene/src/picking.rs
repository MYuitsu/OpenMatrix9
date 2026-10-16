// SPDX-License-Identifier: LGPL-2.1-or-later
use crate::{
    camera::{Camera, Ray, Viewport, add, cross, dot, mul, sub},
    scene::{PickHit, PickIntent, RenderBuffers, Scene, SceneObject, Subelement},
};
use om9_core::{
    Error, Result,
    evaluation::{CurveEvaluator, Parameter},
    geometry::{Curve, Geometry, Point3, Transform},
};
fn distance(a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}
fn triangle(ray: Ray, a: Point3, b: Point3, c: Point3) -> Option<Point3> {
    let e1 = sub(b, a);
    let e2 = sub(c, a);
    let p = cross(ray.direction, e2);
    let determinant = dot(e1, p);
    if !determinant.is_finite() || determinant == 0. {
        return None;
    }
    let t = sub(ray.origin, a);
    let u = dot(t, p) / determinant;
    if !(0. ..=1.).contains(&u) {
        return None;
    }
    let q = cross(t, e1);
    let v = dot(ray.direction, q) / determinant;
    if v < 0. || u + v > 1. {
        return None;
    }
    let depth = dot(e2, q) / determinant;
    if !depth.is_finite() || depth < 0. {
        return None;
    }
    Some(add(ray.origin, mul(ray.direction, depth)))
}
#[allow(clippy::too_many_arguments)] // Explicit source basis + view-space query.
fn curve_parameter(
    c: &Curve,
    evaluator: &CurveEvaluator<'_>,
    transform: Transform,
    screen: [f64; 2],
    camera: &Camera,
    viewport: Viewport,
    points: &[Point3],
    parameters: &[f64],
) -> Result<Option<f64>> {
    if let Curve::Line { start, end, domain } = c {
        let a = transform.point(*start)?;
        let b = transform.point(*end)?;
        let ray = camera.ray(screen, viewport)?;
        let d = sub(b, a);
        let v = sub(a, ray.origin);
        let aa = dot(d, d);
        let bb = dot(d, ray.direction);
        let cc = dot(ray.direction, ray.direction);
        let denominator = aa * cc - bb * bb;
        if !denominator.is_finite() || denominator <= 0. {
            return Ok(None);
        }
        let t = ((bb * dot(ray.direction, v) - cc * dot(d, v)) / denominator).clamp(0., 1.);
        return Ok(Some(domain[0] + (domain[1] - domain[0]) * t));
    }
    let mut best = None;
    for (i, p) in points.iter().enumerate() {
        if let Some(projected) = camera.project(transform.point(*p)?, viewport)? {
            let d = distance(screen, projected.screen);
            if best.is_none_or(|(_, old)| d < old) {
                best = Some((i, d));
            }
        }
    }
    let Some((i, _)) = best else {
        return Ok(None);
    };
    let mut left = parameters[i.saturating_sub(1)];
    let mut right = parameters[(i + 1).min(parameters.len() - 1)];
    let metric = |u| -> Result<f64> {
        let p = evaluator.evaluate(Parameter::Native(u), 0, transform)?[0];
        Ok(camera
            .project(p, viewport)?
            .map_or(f64::INFINITY, |p| distance(screen, p.screen)))
    };
    for _ in 0..36 {
        let a = left + (right - left) / 3.;
        let b = right - (right - left) / 3.;
        if metric(a)? <= metric(b)? {
            right = b;
        } else {
            left = a;
        }
    }
    Ok(Some(left + (right - left) / 2.))
}
#[allow(clippy::too_many_arguments)] // Keep world hit and view/provenance inputs explicit.
fn hit(
    scene: &Scene,
    object: &SceneObject,
    camera: &Camera,
    viewport: Viewport,
    screen: [f64; 2],
    world_point: Point3,
    subset: Subelement,
    tolerance: f64,
) -> Result<Option<PickHit>> {
    let Some(p) = camera.project(world_point, viewport)? else {
        return Ok(None);
    };
    let d = distance(screen, p.screen);
    if d > tolerance {
        return Ok(None);
    }
    Ok(Some(PickHit {
        root: object.root,
        path: object.path.clone(),
        subset,
        world_point,
        source: object.source,
        world_epoch: scene.epoch,
        screen_distance: d,
        depth: p.depth,
        camera: camera.clone(),
        viewport,
    }))
}
pub(crate) fn pick(
    scene: &Scene,
    screen: [f64; 2],
    camera: &Camera,
    viewport: Viewport,
    tolerance: f64,
    intent: PickIntent,
) -> Result<Option<PickHit>> {
    if !tolerance.is_finite() || tolerance < 0. {
        return Err(Error::Invalid("pixel pick tolerance"));
    }
    let ray = camera.ray(screen, viewport)?;
    let mut best: Option<PickHit> = None;
    for object in &scene.objects {
        let allowed = object.visible
            && match intent {
                PickIntent::Select => object.selectable,
                PickIntent::Reference { snap_locked: true } => object.reference_snappable,
                PickIntent::Reference { snap_locked: false } => object.reference_unlocked,
            };
        if !allowed {
            continue;
        }
        let transform = object.display_transform()?;
        let mut candidates = vec![];
        match object.buffers.as_ref() {
            RenderBuffers::Points {
                points,
                native_indices,
            } => {
                for (i, p) in points.iter().enumerate() {
                    let subset = if matches!(scene.entity_for(object)?.geometry, Geometry::Cloud(_))
                    {
                        Subelement::CloudPoint(native_indices[i])
                    } else {
                        Subelement::Whole
                    };
                    if let Some(h) = hit(
                        scene,
                        object,
                        camera,
                        viewport,
                        screen,
                        transform.point(*p)?,
                        subset,
                        tolerance,
                    )? {
                        candidates.push(h);
                    }
                }
            }
            RenderBuffers::Curve { points, parameters } => {
                let Geometry::Curve(curve) = &scene.entity_for(object)?.geometry else {
                    return Err(Error::Invalid("curve cache source"));
                };
                let evaluator = CurveEvaluator::capture(curve, &scene.limits)?;
                if let Some(parameter) = curve_parameter(
                    curve, &evaluator, transform, screen, camera, viewport, points, parameters,
                )? {
                    let p = evaluator.evaluate(Parameter::Native(parameter), 0, transform)?[0];
                    if let Some(h) = hit(
                        scene,
                        object,
                        camera,
                        viewport,
                        screen,
                        p,
                        Subelement::CurveParameter(parameter),
                        tolerance,
                    )? {
                        candidates.push(h);
                    }
                }
            }
            RenderBuffers::Mesh {
                vertices,
                triangles,
                ..
            } => {
                for (indices, face) in triangles {
                    let a = transform.point(vertices[indices[0] as usize])?;
                    let b = transform.point(vertices[indices[1] as usize])?;
                    let c = transform.point(vertices[indices[2] as usize])?;
                    if let Some(p) = triangle(ray, a, b, c)
                        && let Some(h) = hit(
                            scene,
                            object,
                            camera,
                            viewport,
                            screen,
                            p,
                            Subelement::MeshFace(*face),
                            tolerance,
                        )?
                    {
                        candidates.push(h);
                    }
                }
            }
        }
        for candidate in candidates {
            if best.as_ref().is_none_or(|old| {
                candidate.screen_distance < old.screen_distance
                    || candidate.screen_distance == old.screen_distance
                        && candidate.depth < old.depth
            }) {
                best = Some(candidate);
            }
        }
    }
    Ok(best)
}
