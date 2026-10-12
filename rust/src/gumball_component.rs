// SPDX-License-Identifier: LGPL-2.1-or-later
//! OM9-INFO-021 Mesh component frame policy, using immutable world observations.
//!
//! This is an explicit OM9 convention, not evidence of Rhino's exact axis roll.
//! Edges point from the lower to higher native point ID. Vertex X uses the
//! lowest-ID usable incident edge projected onto the area-weighted normal's
//! tangent plane. Z preserves that normal; Y completes a right-handed frame.
//! The selected origin is the vertex or edge midpoint, never owner bounds.
//!
//! Full snapshots are bounded/validated before allocating a vertex fan. Only
//! the selected incident neighborhood must be one consistently oriented
//! manifold fan; unrelated valid components do not determine its frame.

use crate::gumball::{Frame, GumballError, Point};
use std::collections::BTreeMap;

pub const MAX_POINTS: usize = 131072;
pub const MAX_TRIANGLES: usize = 131072;

/// Native point IDs, zero based and valid only for the captured source stamp.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Component {
    Vertex(u32),
    Edge(u32, u32),
}

#[derive(Default)]
struct FanNode {
    previous: Option<u32>,
    next: Option<u32>,
}

/// Resolve one native indexed Mesh component without guessing missing axes.
///
/// `points` already include the object's world transform; indices refer to
/// this exact point snapshot. Returned frames are independent of view cameras.
pub fn mesh_component_frame(
    points: &[Point],
    triangles: &[[u32; 3]],
    component: Component,
) -> Result<Frame, GumballError> {
    if points.len() > MAX_POINTS || triangles.len() > MAX_TRIANGLES {
        return Err(GumballError::Limit);
    }
    if points.is_empty() || triangles.is_empty() {
        return Err(GumballError::InvalidSources);
    }
    if points
        .iter()
        .flatten()
        .any(|v| !v.is_finite() || v.abs() > 1e12)
    {
        return Err(GumballError::InvalidInput);
    }
    let (first, second) = match component {
        Component::Vertex(id) => (id, None),
        Component::Edge(a, b) if a != b => (a.min(b), Some(a.max(b))),
        Component::Edge(_, _) => return Err(GumballError::InvalidSources),
    };
    if first as usize >= points.len() || second.is_some_and(|id| id as usize >= points.len()) {
        return Err(GumballError::InvalidSources);
    }
    // No fan allocation occurs until every triangle index and geometry is safe.
    for &triangle in triangles {
        if triangle.iter().any(|&id| id as usize >= points.len())
            || triangle[0] == triangle[1]
            || triangle[1] == triangle[2]
            || triangle[2] == triangle[0]
        {
            return Err(GumballError::InvalidSources);
        }
        triangle_normal(points, triangle)?;
    }

    let mut normal = [0.; 3];
    let mut area = 0.;
    let mut incident = 0_usize;
    let mut edge_orientation = 0_i32;
    let mut fan = BTreeMap::<u32, FanNode>::new();
    for &triangle in triangles {
        let Some(position) = triangle.iter().position(|&id| id == first) else {
            continue;
        };
        let a = triangle[(position + 1) % 3];
        let b = triangle[(position + 2) % 3];
        if let Some(second) = second {
            if a != second && b != second {
                continue;
            }
            incident += 1;
            edge_orientation += if a == second { 1 } else { -1 };
            if incident > 2 || (incident == 2 && edge_orientation != 0) {
                return Err(GumballError::InvalidFrame);
            }
        } else {
            // At a selected vertex, every incident triangle contributes an
            // oriented link a -> b. A manifold fan has no branching links.
            if fan.entry(a).or_default().next.replace(b).is_some()
                || fan.entry(b).or_default().previous.replace(a).is_some()
            {
                return Err(GumballError::InvalidFrame);
            }
            incident += 1;
        }
        let contribution = triangle_normal(points, triangle)?;
        normal = add(normal, contribution);
        area += length(contribution);
    }
    if incident == 0 {
        return Err(GumballError::InvalidSources);
    }
    if length(normal) <= area * 1e-12 {
        return Err(GumballError::InvalidFrame);
    }
    let normal = unit(normal)?;

    let origin = points[first as usize];
    if let Some(second) = second {
        let endpoint = points[second as usize];
        let x = unit(sub(endpoint, origin))?;
        let midpoint = std::array::from_fn(|i| origin[i] + (endpoint[i] - origin[i]) / 2.);
        return Frame::from_directions(midpoint, x, normal);
    }
    validate_fan(&fan)?;
    for &neighbor in fan.keys() {
        let edge = sub(points[neighbor as usize], origin);
        let projected = sub(edge, mul(normal, dot(edge, normal)));
        if length(projected) > length(edge) * 1e-12 {
            return Frame::from_directions(origin, unit(projected)?, normal);
        }
    }
    Err(GumballError::InvalidFrame)
}

fn validate_fan(fan: &BTreeMap<u32, FanNode>) -> Result<(), GumballError> {
    let starts = fan
        .iter()
        .filter(|(_, node)| node.previous.is_none())
        .collect::<Vec<_>>();
    let ends = fan.values().filter(|node| node.next.is_none()).count();
    let closed = starts.is_empty() && ends == 0;
    if !closed && (starts.len() != 1 || ends != 1) {
        return Err(GumballError::InvalidFrame);
    }
    let start = if closed {
        *fan.first_key_value().ok_or(GumballError::InvalidFrame)?.0
    } else {
        *starts[0].0
    };
    let mut current = start;
    for visited in 1..=fan.len() {
        match fan.get(&current).ok_or(GumballError::InvalidFrame)?.next {
            None if !closed && visited == fan.len() => return Ok(()),
            Some(next) if closed && next == start && visited == fan.len() => return Ok(()),
            Some(next) if next != start => current = next,
            _ => return Err(GumballError::InvalidFrame),
        }
    }
    Err(GumballError::InvalidFrame)
}

fn triangle_normal(points: &[Point], triangle: [u32; 3]) -> Result<Point, GumballError> {
    let origin = points[triangle[0] as usize];
    let a = sub(points[triangle[1] as usize], origin);
    let b = sub(points[triangle[2] as usize], origin);
    let normal = cross(a, b);
    let scale = length(a).max(length(b)).max(length(sub(a, b)));
    if scale <= 1e-12 || length(normal) <= scale * scale * 1e-12 {
        return Err(GumballError::InvalidFrame);
    }
    Ok(normal)
}
fn add(a: Point, b: Point) -> Point {
    std::array::from_fn(|i| a[i] + b[i])
}
fn sub(a: Point, b: Point) -> Point {
    std::array::from_fn(|i| a[i] - b[i])
}
fn mul(a: Point, factor: f64) -> Point {
    a.map(|v| v * factor)
}
fn dot(a: Point, b: Point) -> f64 {
    a.into_iter().zip(b).map(|(a, b)| a * b).sum()
}
fn cross(a: Point, b: Point) -> Point {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn length(a: Point) -> f64 {
    a[0].hypot(a[1]).hypot(a[2])
}
fn unit(a: Point) -> Result<Point, GumballError> {
    let length = length(a);
    if !length.is_finite() || length <= 0. {
        return Err(GumballError::InvalidFrame);
    }
    Ok(mul(a, 1. / length))
}
