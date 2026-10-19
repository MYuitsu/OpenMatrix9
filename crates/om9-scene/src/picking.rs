// SPDX-License-Identifier: LGPL-2.1-or-later
use crate::{
    camera::{Camera, Ray, Viewport, add, cross, dot, mul, sub},
    scene::{PickHit, PickIntent, PickKind, RenderBuffers, Scene, SceneObject, Subelement},
};

/// Screen-space distance from `point` to the segment `a`-`b`.
fn segment_distance(point: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let ab = [b[0] - a[0], b[1] - a[1]];
    let ap = [point[0] - a[0], point[1] - a[1]];
    let length2 = ab[0] * ab[0] + ab[1] * ab[1];
    if length2 <= 1e-18 {
        return distance(point, a);
    }
    let t = ((ap[0] * ab[0] + ap[1] * ab[1]) / length2).clamp(0.0, 1.0);
    distance(point, [a[0] + ab[0] * t, a[1] + ab[1] * t])
}
use om9_core::{
    Error, Result,
    evaluation::{CurveEvaluator, Parameter},
    geometry::{Curve, Geometry, Point3, Transform},
};
/// The world point on the edge `a`-`b` that projects nearest the pointer, so the hit reports a real point
/// on the edge instead of the label's own guess.
fn edge_point(
    pointer: [f64; 2],
    start_screen: [f64; 2],
    end_screen: [f64; 2],
    start: om9_core::geometry::Point3,
    end: om9_core::geometry::Point3,
    depths: [f64; 2],
) -> Option<om9_core::geometry::Point3> {
    let ab = [end_screen[0] - start_screen[0], end_screen[1] - start_screen[1]];
    let ap = [pointer[0] - start_screen[0], pointer[1] - start_screen[1]];
    let length2 = ab[0] * ab[0] + ab[1] * ab[1];
    let t = if length2 <= 1e-18 {
        0.0
    } else {
        ((ap[0] * ab[0] + ap[1] * ab[1]) / length2).clamp(0.0, 1.0)
    };
    // Perspective divides by depth: linear screen interpolation is not linear in world space.
    let t = (t / depths[1]) / ((1.0 - t) / depths[0] + t / depths[1]);
    Some([
        start[0] + (end[0] - start[0]) * t,
        start[1] + (end[1] - start[1]) * t,
        start[2] + (end[2] - start[2]) * t,
    ])
}

/// Closest point of a 2D triangle to `p`, as barycentric `(u, v)` with the third weight `1 - u - v`.
///
/// Used as the screen-space fallback when the exact ray test misses: a ray through the shared diagonal of a
/// quad used to fall between both triangles, which left a hole exactly along every diagonal.
fn closest_on_triangle_2d(
    p: [f64; 2],
    a: [f64; 2],
    b: [f64; 2],
    c: [f64; 2],
) -> Option<(f64, f64)> {
    // A triangle seen edge-on projects to a segment, so the divisions below can divide by zero.
    let checked = |u: f64, v: f64| -> Option<(f64, f64)> {
        if u.is_finite() && v.is_finite() && u >= -1e-9 && v >= -1e-9 && u + v <= 1.0 + 1e-9 {
            Some((u, v))
        } else {
            None
        }
    };
    let ab = [b[0] - a[0], b[1] - a[1]];
    let ac = [c[0] - a[0], c[1] - a[1]];
    let ap = [p[0] - a[0], p[1] - a[1]];
    let d1 = ab[0] * ap[0] + ab[1] * ap[1];
    let d2 = ac[0] * ap[0] + ac[1] * ap[1];
    if d1 <= 0.0 && d2 <= 0.0 {
        return checked(0.0, 0.0);
    }
    let bp = [p[0] - b[0], p[1] - b[1]];
    let d3 = ab[0] * bp[0] + ab[1] * bp[1];
    let d4 = ac[0] * bp[0] + ac[1] * bp[1];
    if d3 >= 0.0 && d4 <= d3 {
        return checked(1.0, 0.0);
    }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        if (d1 - d3).abs() <= 1e-18 {
            return None;
        }
        return checked(d1 / (d1 - d3), 0.0);
    }
    let cp = [p[0] - c[0], p[1] - c[1]];
    let d5 = ab[0] * cp[0] + ab[1] * cp[1];
    let d6 = ac[0] * cp[0] + ac[1] * cp[1];
    if d6 >= 0.0 && d5 <= d6 {
        return checked(0.0, 1.0);
    }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        if (d2 - d6).abs() <= 1e-18 {
            return None;
        }
        return checked(0.0, d2 / (d2 - d6));
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let denominator = (d4 - d3) + (d5 - d6);
        if denominator.abs() <= 1e-18 {
            return None;
        }
        let w = (d4 - d3) / denominator;
        return checked(1.0 - w, w);
    }
    let denominator = va + vb + vc;
    if denominator.abs() <= 1e-18 {
        return None;
    }
    let v = vb / denominator;
    let w = vc / denominator;
    checked(v, w)
}

fn distance(a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}
/// Barycentric slack: a ray through the shared edge of two triangles used to miss both, which left a hole
/// along every quad diagonal in picking.
const TRIANGLE_EPSILON: f64 = 1e-9;

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
    if u < -TRIANGLE_EPSILON || u > 1. + TRIANGLE_EPSILON {
        return None;
    }
    let q = cross(t, e1);
    let v = dot(ray.direction, q) / determinant;
    if v < -TRIANGLE_EPSILON || u + v > 1. + TRIANGLE_EPSILON {
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
    Ok(pick_candidates(scene, screen, camera, viewport, tolerance, intent, PickKind::Object)?
        .into_iter()
        .next())
}

/// Every object under the pointer, nearest first. One hit per object: the ambiguity popup offers objects,
/// not individual faces of the same object.
pub(crate) fn pick_candidates(
    scene: &Scene,
    screen: [f64; 2],
    camera: &Camera,
    viewport: Viewport,
    tolerance: f64,
    intent: PickIntent,
    kind: PickKind,
) -> Result<Vec<PickHit>> {
    if !tolerance.is_finite() || tolerance < 0. {
        return Err(Error::Invalid("pixel pick tolerance"));
    }
    let ray = camera.ray(screen, viewport)?;
    let mut hits: Vec<PickHit> = Vec::new();
    for object in &scene.objects {
        let mut best: Option<PickHit> = None;
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
                if matches!(kind, PickKind::Face | PickKind::Edge) { continue; }
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
                if matches!(kind, PickKind::Face | PickKind::Vertex) { continue; }
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
                edges,
            } => {
                if kind == PickKind::Vertex {
                    for (index, vertex) in vertices.iter().enumerate() {
                        if let Some(h) = hit(
                            scene,
                            object,
                            camera,
                            viewport,
                            screen,
                            transform.point(*vertex)?,
                            Subelement::MeshVertex(index as u32),
                            tolerance,
                        )? {
                            candidates.push(h);
                        }
                    }
                } else if kind == PickKind::Edge {
                    // Use document polygon boundaries, never tessellation diagonals.
                    for [first, second] in edges {
                            let (a, b) = ((*first).min(*second) as usize, (*first).max(*second) as usize);
                            let Some([world_start,world_end])=camera.clip_segment(transform.point(vertices[a])?,transform.point(vertices[b])?)? else { continue; };
                            let (Ok(Some(start)), Ok(Some(end))) = (
                                camera.project(world_start, viewport),
                                camera.project(world_end, viewport),
                            ) else {
                                continue;
                            };
                            // The edge is picked at the point on it nearest the pointer.
                            let Some(midpoint) = edge_point(
                                screen, start.screen, end.screen,
                                world_start,
                                world_end,
                                match camera.projection {
                                    crate::camera::Projection::Perspective {..} => [start.depth,end.depth],
                                    crate::camera::Projection::Orthographic {..} => [1.,1.],
                                },
                            ) else {
                                continue;
                            };
                            if segment_distance(screen, start.screen, end.screen) > tolerance {
                                continue;
                            }
                            if let Some(h) = hit(
                                scene,
                                object,
                                camera,
                                viewport,
                                screen,
                                midpoint,
                                Subelement::MeshEdge { a: a as u32, b: b as u32 },
                                tolerance,
                            )? {
                                candidates.push(h);
                            }
                    }
                }
                // A vertex or edge pick never falls back to the whole face test.
                if matches!(kind, PickKind::Object | PickKind::Face) {
                    let mut face_hit = false;
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
                            face_hit = true;
                        }
                    }
                    if !face_hit {
                        // Screen-space fallback for a ray that threaded exactly through a shared edge.
                        let mut nearest: Option<(f64, PickHit)> = None;
                        for (indices, face) in triangles {
                            let a = transform.point(vertices[indices[0] as usize])?;
                            let b = transform.point(vertices[indices[1] as usize])?;
                            let c = transform.point(vertices[indices[2] as usize])?;
                            let (Ok(Some(pa)), Ok(Some(pb)), Ok(Some(pc))) = (
                                camera.project(a, viewport),
                                camera.project(b, viewport),
                                camera.project(c, viewport),
                            ) else {
                                continue;
                            };
                            let Some((u, v)) = closest_on_triangle_2d(screen, pa.screen, pb.screen, pc.screen)
                            else {
                                continue;
                            };
                            let weights = match camera.projection {
                                crate::camera::Projection::Perspective {..} => [(1.0 - u - v) / pa.depth, u / pb.depth, v / pc.depth],
                                crate::camera::Projection::Orthographic {..} => [1.0-u-v,u,v],
                            };
                            let sum: f64 = weights.iter().sum();
                            let world = std::array::from_fn(|i| (a[i]*weights[0] + b[i]*weights[1] + c[i]*weights[2]) / sum);
                            if !world.iter().all(|value| value.is_finite()) {
                                continue;
                            }
                            if let Some(h) = hit(
                                scene,
                                object,
                                camera,
                                viewport,
                                screen,
                                world,
                                Subelement::MeshFace(*face),
                                tolerance,
                            )? && nearest
                                .as_ref()
                                .is_none_or(|(best, _)| h.screen_distance < *best)
                            {
                                nearest = Some((h.screen_distance, h));
                            }
                        }
                        if let Some((_, h)) = nearest {
                            candidates.push(h);
                        }
                    }
                }
            }
        }
        if let Geometry::Brep(shape)=&scene.entity_for(object)?.geometry {
            let display=&shape.display;
            candidates.retain_mut(|candidate| {
                let mapped=match candidate.subset {
                    Subelement::MeshFace(face)=>display.faces.get(face as usize).copied().map(Subelement::BrepFace),
                    Subelement::MeshVertex(vertex)=>display.vertices.iter().position(|offset|*offset==vertex)
                        .and_then(|index|display.vertex_ids.get(index)).copied().map(Subelement::BrepVertex),
                    Subelement::MeshEdge{a,b}=>display.segments.iter().find(|[x,y,_]|(*x==a && *y==b)||(*x==b && *y==a))
                        .and_then(|[_,_,edge]|display.edges.get(*edge as usize)).copied().map(Subelement::BrepEdge),
                    _=>None,
                };
                if let Some(subset)=mapped {candidate.subset=subset;true} else {false}
            });
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
        if let Some(best) = best {
            hits.push(best);
        }
    }
    hits.sort_by(|left, right| left.depth.total_cmp(&right.depth).then(left.root.cmp(&right.root)).then(left.path.cmp(&right.path)));
    let mut roots = std::collections::BTreeSet::new();
    hits.retain(|hit| roots.insert(hit.root));
    Ok(hits)
}
