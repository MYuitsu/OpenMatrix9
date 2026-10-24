//! Arc Blend (`ArcBlend`, OM9-CURVE-056): tangent arc between two curve ends.

use om9_core::geometry::{Curve, Geometry, Point3};

use super::{Controller, Error};

pub const ARC_BLEND: &str = "OM9-CURVE-056";

#[derive(Debug, Clone)]
pub struct ArcBlendPending {
    pub first: Curve,
    pub second: Curve,
}

impl ArcBlendPending {
    pub fn prompt(&self) -> &'static str {
        "Enter to blend with an arc"
    }
}

impl Controller {
    pub fn dispatch_arc_blend(&mut self) -> Result<(), Error> {
        let ordered = self.selected_ordered();
        if ordered.len() != 2 {
            return Err(Error::Invalid("two curve selection"));
        }
        let mut curves = Vec::with_capacity(2);
        for id in ordered {
            let entity = self
                .document()
                .entities
                .get(&id)
                .ok_or(Error::Missing("selected curve"))?;
            let Geometry::Curve(curve) = &entity.geometry else {
                return Err(Error::Invalid("curve selection"));
            };
            curves.push(curve.clone());
        }
        let endpoint = |curve: &Curve, end: bool| {
            om9_core::surface_kernel::sample_curve(curve, 16, self.coordinator.limits())
                .ok()
                .and_then(|samples| {
                    if end {
                        samples.last().copied()
                    } else {
                        samples.first().copied()
                    }
                })
        };
        let gap = |a: &Curve, b: &Curve| {
            match (endpoint(a, true), endpoint(b, false)) {
                (Some(a), Some(b)) => distance(a, b),
                _ => f64::INFINITY,
            }
        };
        let (first, second) = if gap(&curves[0], &curves[1]) <= gap(&curves[1], &curves[0]) {
            (curves[0].clone(), curves[1].clone())
        } else {
            (curves[1].clone(), curves[0].clone())
        };
        self.arc_blend_pending = Some(ArcBlendPending { first, second });
        Ok(())
    }

    pub fn submit_arc_blend_text(&mut self, text: &str) -> Option<Result<(), Error>> {
        let pending = self.arc_blend_pending.take()?;
        let option = text.trim().to_ascii_lowercase();
        if option == "cancel" || option == "esc" {
            return Some(Ok(()));
        }
        if !text.trim().is_empty() {
            self.arc_blend_pending = Some(pending);
            return Some(Err(Error::Invalid("ArcBlend accepts Enter or Cancel")));
        }
        let first = match om9_core::surface_kernel::sample_curve(
            &pending.first,
            16,
            self.coordinator.limits(),
        ) {
            Ok(samples) => samples,
            Err(error) => return Some(Err(error)),
        };
        let second = match om9_core::surface_kernel::sample_curve(
            &pending.second,
            16,
            self.coordinator.limits(),
        ) {
            Ok(samples) => samples,
            Err(error) => return Some(Err(error)),
        };
        if first.len() < 2 || second.len() < 2 {
            return Some(Err(Error::Invalid("arc blend samples")));
        }
        let a = first[first.len() - 1];
        let before = first[first.len() - 2];
        let b = second[0];
        let tangent = unit([a[0] - before[0], a[1] - before[1], a[2] - before[2]]).unwrap_or([1.0, 0.0, 0.0]);
        let plane = self.input_plane();
        let plane_normal = [
            plane.x[1] * plane.y[2] - plane.x[2] * plane.y[1],
            plane.x[2] * plane.y[0] - plane.x[0] * plane.y[2],
            plane.x[0] * plane.y[1] - plane.x[1] * plane.y[0],
        ];
        let mut normal = [
            tangent[1] * plane_normal[2] - tangent[2] * plane_normal[1],
            tangent[2] * plane_normal[0] - tangent[0] * plane_normal[2],
            tangent[0] * plane_normal[1] - tangent[1] * plane_normal[0],
        ];
        let norm = (normal[0].powi(2) + normal[1].powi(2) + normal[2].powi(2)).sqrt();
        if norm <= 1e-12 {
            normal = plane.y;
        } else {
            normal = [normal[0] / norm, normal[1] / norm, normal[2] / norm];
        }
        let delta = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
        let denominator = 2.0 * (normal[0] * delta[0] + normal[1] * delta[1] + normal[2] * delta[2]);
        if denominator.abs() <= 1e-12 {
            return Some(Err(Error::Invalid("arc blend direction is collinear")));
        }
        let length_squared = delta[0].powi(2) + delta[1].powi(2) + delta[2].powi(2);
        let offset = -length_squared / denominator;
        let center = [
            a[0] + normal[0] * offset,
            a[1] + normal[1] * offset,
            a[2] + normal[2] * offset,
        ];
        let radius = distance(center, a);
        let x = match unit([a[0] - center[0], a[1] - center[1], a[2] - center[2]]) {
            Ok(x) => x,
            Err(error) => return Some(Err(error)),
        };
        let y = match unit(cross(plane_normal, x)) {
            Ok(y) => y,
            Err(error) => return Some(Err(error)),
        };
        let end_angle = signed_angle(x, y, b, center);
        let sweep = if end_angle <= 0.0 { end_angle + std::f64::consts::TAU } else { end_angle };
        let curve = match om9_core::curve_build::arc_by_center(
            center, x, y, radius, 0.0, sweep,
        ) {
            Ok(curve) => curve,
            Err(error) => return Some(Err(error)),
        };
        Some(self.commit_geometry(
            ARC_BLEND,
            "ArcBlend",
            Geometry::Curve(curve),
        ))
    }

    pub fn arc_blend_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        if self.arc_blend_pending.is_some() {
            vec![("Cancel", "Cancel")]
        } else {
            Vec::new()
        }
    }
}

fn unit(vector: Point3) -> Result<Point3, Error> {
    let length = (vector[0].powi(2) + vector[1].powi(2) + vector[2].powi(2)).sqrt();
    if length <= 1e-12 {
        return Err(Error::Invalid("arc blend frame"));
    }
    Ok([vector[0] / length, vector[1] / length, vector[2] / length])
}

fn cross(a: Point3, b: Point3) -> Point3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn distance(a: Point3, b: Point3) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

fn signed_angle(x: Point3, y: Point3, point: Point3, center: Point3) -> f64 {
    let dx = point[0] - center[0];
    let dy = point[1] - center[1];
    let dz = point[2] - center[2];
    let cos = dx * x[0] + dy * x[1] + dz * x[2];
    let sin = dx * y[0] + dy * y[1] + dz * y[2];
    sin.atan2(cos)
}
