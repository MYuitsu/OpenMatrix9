// SPDX-License-Identifier: LGPL-2.1-or-later
//! Closed boundaries of the filled projection of selected render meshes.
use crate::{
    camera::{CPlane, Camera, Projection, add, dot, mul, sub},
    scene::{RenderBuffers, SceneObject},
};
use i_overlay::{core::fill_rule::FillRule, float::simplify::SimplifyShape};
use om9_core::{
    Error, Result,
    budget::Limits,
    geometry::{Point3, finite},
};

pub fn outlines(
    camera: &Camera,
    cplane: CPlane,
    objects: &[&SceneObject],
    limits: &Limits,
) -> Result<Vec<Vec<Point3>>> {
    camera.validate()?;
    cplane.validate()?;
    let [right, up, forward] = camera.basis()?;
    let perspective = matches!(camera.projection, Projection::Perspective { .. });
    let near = camera.near_mm;
    let far = camera.far_mm;
    // Rhino 5 extracts in scaled world-to-clip coordinates at clip z=0, then
    // applies the inverse transform. Its perspective depth is the harmonic mean.
    let depth = if perspective {
        near / (0.5 + 0.5 * near / far)
    } else {
        0.5 * near + 0.5 * far
    };
    let (origin, right, up) =
        if !perspective && dot(cplane.normal(), forward).abs() >= 1.0_f64.to_radians().cos() {
            (cplane.origin, cplane.x, cplane.y)
        } else {
            (add(camera.eye, mul(forward, depth)), right, up)
        };
    let mut faces = Vec::new();
    for object in objects {
        let RenderBuffers::Mesh {
            vertices,
            triangles,
            ..
        } = object.buffers.as_ref()
        else {
            return Err(Error::Missing("MeshOutline render mesh"));
        };
        if object.buffers.is_trim_display_fallback() {
            return Err(Error::Unsupported(
                "MeshOutline needs a trimmed render mesh",
            ));
        }
        if triangles.len() as u64 > limits.max_nodes.saturating_sub(faces.len() as u64)
            || vertices.len() as u64 > limits.max_points
        {
            return Err(Error::Budget("MeshOutline input"));
        }
        let mut view = Vec::new();
        view.try_reserve_exact(vertices.len())
            .map_err(|_| Error::Budget("MeshOutline vertices"))?;
        for &point in vertices {
            let world = object.transform.point(point)?;
            let delta = sub(world, if perspective { camera.eye } else { origin });
            // This models whole objects, including faces outside the near/far slab.
            let scale = if perspective {
                let z = dot(delta, forward);
                if z == 0. {
                    return Err(Error::Invalid("MeshOutline vertex on eye plane"));
                }
                depth / z
            } else {
                1.
            };
            let xy = [dot(delta, right) * scale, dot(delta, up) * scale];
            if !xy.iter().all(|v| v.is_finite()) {
                return Err(Error::Invalid("MeshOutline projection"));
            }
            view.push(xy);
        }
        faces
            .try_reserve(triangles.len())
            .map_err(|_| Error::Budget("MeshOutline faces"))?;
        for &(tri, _) in triangles {
            let vertex = |i: u32| {
                view.get(i as usize)
                    .copied()
                    .ok_or(Error::Invalid("MeshOutline face index"))
            };
            let [a, b, c] = [vertex(tri[0])?, vertex(tri[1])?, vertex(tri[2])?];
            let area = (b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1]);
            if !area.is_finite() {
                return Err(Error::Invalid("MeshOutline projected area"));
            }
            if area != 0. {
                faces.push(if area > 0. {
                    vec![a, b, c]
                } else {
                    vec![a, c, b]
                });
            }
        }
    }
    if faces.is_empty() {
        return Ok(vec![]);
    }
    let mut contours = Vec::new();
    let mut count = 0usize;
    for path in faces
        .simplify_shape_as::<i64>(FillRule::NonZero)
        .into_iter()
        .flatten()
    {
        count = count
            .checked_add(path.len() + 1)
            .ok_or(Error::Overflow("MeshOutline output"))?;
        if count as u64 > limits.max_points || contours.len() as u64 >= limits.max_outputs {
            return Err(Error::Budget("MeshOutline output"));
        }
        let mut points = Vec::with_capacity(path.len());
        for [x, y] in path {
            let world = add(origin, add(mul(right, x), mul(up, y)));
            finite(world)?;
            points.push(world);
        }
        contours.push(points);
    }
    Ok(contours)
}
