// SPDX-License-Identifier: LGPL-2.1-or-later
//! Display-only midpoint isocurves of the planar faces of a round brilliant.
use crate::geometry::Point3;
use crate::settings::{cross, dot, lerp, norm, sub, unit};
use std::collections::BTreeMap;

/// Recover each convex plane from its display triangles. Mesh and B-rep
/// displays can split the same facet into different triangles.
pub fn round_facet_isocurves(
    vertices: &[Point3],
    triangles: &[([u32; 3], u32)],
) -> Vec<[Point3; 2]> {
    let mut guides = Vec::new();
    let radius = vertices.iter().map(|p| p[0].hypot(p[1])).fold(0., f64::max);
    if radius <= 0. || !radius.is_finite() {
        return guides;
    }
    let tol = radius * 1e-6;
    let mut planes = BTreeMap::<[i64; 4], (Point3, Vec<Point3>)>::new();
    for &(ids, _) in triangles {
        let [Some(&a), Some(&b), Some(&c)] = ids.map(|i| vertices.get(i as usize)) else {
            continue;
        };
        let Ok(mut n) = unit(cross(sub(b, a), sub(c, a))) else {
            continue;
        };
        let dominant = (0..3)
            .max_by(|&i, &j| n[i].abs().total_cmp(&n[j].abs()))
            .unwrap();
        if n[dominant] < 0. {
            n = n.map(|x| -x);
        }
        let key = [n[0], n[1], n[2], dot(n, a) / radius].map(|x| (x * 1e6).round() as i64);
        let (_, polygon) = planes.entry(key).or_insert_with(|| (n, Vec::new()));
        for point in [a, b, c] {
            if !polygon.iter().any(|&p| norm(sub(p, point)) < tol) {
                polygon.push(point);
            }
        }
    }
    let top = vertices
        .iter()
        .map(|p| p[2])
        .fold(f64::NEG_INFINITY, f64::max);
    for (_, (n, mut polygon)) in planes {
        let center = std::array::from_fn(|i| {
            polygon.iter().map(|p| p[i]).sum::<f64>() / polygon.len() as f64
        });
        let Ok(x) = unit(sub(polygon[0], center)) else {
            continue;
        };
        let y = cross(n, x);
        let angle = |p: Point3| dot(sub(p, center), y).atan2(dot(sub(p, center), x));
        polygon.sort_by(|a, b| angle(*a).total_cmp(&angle(*b)));
        let radial = |p: Point3| p[0].hypot(p[1]);
        let mut by_radius = polygon.clone();
        by_radius.sort_by(|a, b| radial(*a).total_cmp(&radial(*b)));
        let inner = by_radius[0];
        let outer = *by_radius.last().unwrap();
        let table: Vec<_> = polygon
            .iter()
            .copied()
            .filter(|p| (p[2] - top).abs() < tol)
            .collect();
        let axis = if table.len() == polygon.len() {
            [1., 0., 0.]
        } else if table.len() == 2 {
            let axis = sub(table[1], table[0]);
            // Cardinal star facets consist of two trimmed halves.
            if outer[0].abs().min(outer[1].abs()) < tol {
                let mid = lerp(table[0], table[1], 0.5);
                guides.push([outer, mid]);
                for end in table {
                    midpoint_guides(&[end, mid, outer], n, axis, tol, &mut guides);
                }
                continue;
            }
            axis
        } else if table.len() == 1 || radial(inner) < tol {
            // Orient a main facet by one of its two rim-to-junction edges.
            let junctions = by_radius
                .iter()
                .copied()
                .filter(|p| radial(*p) > radial(inner) + tol && radial(*p) < radial(outer) - tol);
            let score = |p: Point3| {
                if table.is_empty() {
                    -p[0].abs().min(p[1].abs())
                } else {
                    p[1].abs()
                }
            };
            let Some(junction) = junctions.max_by(|a, b| score(*a).total_cmp(&score(*b))) else {
                continue;
            };
            sub(junction, outer)
        } else if radial(inner) < radius * 0.9 {
            let end = polygon
                .iter()
                .copied()
                .max_by(|a, b| norm(sub(*a, inner)).total_cmp(&norm(sub(*b, inner))))
                .unwrap();
            sub(end, inner)
        } else {
            // Girdle strips use a vertical parameter along their side.
            [0., 0., 1.]
        };
        midpoint_guides(&polygon, n, axis, tol, &mut guides);
    }
    guides
}

fn midpoint_guides(
    polygon: &[Point3],
    normal: Point3,
    direction: Point3,
    tol: f64,
    guides: &mut Vec<[Point3; 2]>,
) {
    let Ok(u) = unit(direction) else { return };
    for axis in [u, cross(normal, u)] {
        let params = polygon.iter().map(|&p| dot(p, axis));
        let mid = (params.clone().fold(f64::INFINITY, f64::min)
            + params.fold(f64::NEG_INFINITY, f64::max))
            * 0.5;
        let mut ends = Vec::with_capacity(2);
        for (&a, &b) in polygon.iter().zip(polygon.iter().cycle().skip(1)) {
            let (da, db) = (dot(a, axis) - mid, dot(b, axis) - mid);
            if da.abs() < tol {
                ends.push(a);
            }
            if (da > tol && db < -tol) || (da < -tol && db > tol) {
                ends.push(lerp(a, b, da / (da - db)));
            }
        }
        if ends.len() == 2 && norm(sub(ends[0], ends[1])) > tol {
            guides.push([ends[0], ends[1]]);
        }
    }
}
