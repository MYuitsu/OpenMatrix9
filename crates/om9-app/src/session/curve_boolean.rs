//! Curve Boolean (`CurveBoolean`, OM9-CURVE-037).
//!
//! For two closed planar curves the command computes their union boundary:
//! intersections split each curve, arcs outside the other curve are kept and
//! joined. Other selections fall back to joining connected boundary curves.

use om9_core::geometry::{Curve, Geometry, Point3};

use super::{Controller, Error};

pub const CURVE_BOOLEAN: &str = "OM9-CURVE-037";

impl Controller {
    pub fn dispatch_curve_boolean(&mut self) -> Result<(), Error> {
        let ordered = self.selected_ordered();
        if ordered.len() < 2 {
            return Err(Error::Invalid("at least two curves"));
        }
        let mut curves = Vec::with_capacity(ordered.len());
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
        if curves.len() == 2
            && let Some(result) = self.union_two_closed(&curves[0], &curves[1])
        {
            return result;
        }
        let curve = om9_core::curve_ops::join_curves(&curves, 1e-6, self.coordinator.limits())?;
        self.commit_geometry(CURVE_BOOLEAN, "CurveBoolean", Geometry::Curve(curve))
    }

    fn union_two_closed(&mut self, first: &Curve, second: &Curve) -> Option<Result<(), Error>> {
        let limits = self.coordinator.limits();
        if !om9_core::curve_ops::is_closed(first, 1e-6, limits).ok()?
            || !om9_core::curve_ops::is_closed(second, 1e-6, limits).ok()?
        {
            return None;
        }
        let first_samples = om9_core::surface_kernel::sample_curve(first, 128, limits).ok()?;
        let second_samples = om9_core::surface_kernel::sample_curve(second, 128, limits).ok()?;
        let intersections =
            om9_core::curve_surface_ops::curve_curve_intersections(first, second, 1e-6, limits)
                .ok()?;
        if intersections.is_empty() {
            let first_mid = om9_core::curve_ops::point_at(first, 0.5, limits).ok()?;
            let second_mid = om9_core::curve_ops::point_at(second, 0.5, limits).ok()?;
            let first_inside_second = inside_polygon(&second_samples, first_mid);
            let second_inside_first = inside_polygon(&first_samples, second_mid);
            let chosen: Vec<&Curve> = if first_inside_second {
                vec![second]
            } else if second_inside_first {
                vec![first]
            } else {
                vec![first, second]
            };
            for curve in chosen {
                if let Err(error) = self.commit_geometry(
                    CURVE_BOOLEAN,
                    "CurveBoolean",
                    Geometry::Curve(curve.clone()),
                ) {
                    return Some(Err(error));
                }
            }
            return Some(Ok(()));
        }
        let mut first_fractions: Vec<f64> =
            intersections.iter().map(|hit| hit.first_fraction).collect();
        let mut second_fractions: Vec<f64> = intersections
            .iter()
            .map(|hit| hit.second_fraction)
            .collect();
        first_fractions.sort_by(f64::total_cmp);
        second_fractions.sort_by(f64::total_cmp);
        first_fractions.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
        second_fractions.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
        if first_fractions.len() < 2 || second_fractions.len() < 2 {
            return None;
        }
        let mut pieces = Vec::new();
        collect_outside_pieces(
            first,
            &first_fractions,
            &second_samples,
            &mut pieces,
            limits,
        );
        collect_outside_pieces(
            second,
            &second_fractions,
            &first_samples,
            &mut pieces,
            limits,
        );
        if pieces.is_empty() {
            return None;
        }
        for piece in pieces {
            if let Err(error) =
                self.commit_geometry(CURVE_BOOLEAN, "CurveBoolean", Geometry::Curve(piece))
            {
                return Some(Err(error));
            }
        }
        Some(Ok(()))
    }
}

fn collect_outside_pieces(
    curve: &Curve,
    fractions: &[f64],
    other: &[Point3],
    pieces: &mut Vec<Curve>,
    limits: &om9_core::budget::Limits,
) {
    let count = fractions.len();
    for index in 0..count {
        let start = fractions[index];
        let end = if index + 1 < count {
            fractions[index + 1]
        } else {
            fractions[0] + 1.0
        };
        let middle = ((start + end) * 0.5).min(1.0);
        let Ok(point) = om9_core::curve_ops::point_at(curve, middle, limits) else {
            continue;
        };
        if inside_polygon(other, point) {
            continue;
        }
        if end <= 1.0 {
            if let Ok(piece) = om9_core::curve_ops::trim_curve(curve, start, end, limits) {
                pieces.push(piece);
            }
        } else {
            if let Ok(piece) = om9_core::curve_ops::trim_curve(curve, start, 1.0, limits) {
                pieces.push(piece);
            }
            if let Ok(piece) = om9_core::curve_ops::trim_curve(curve, 0.0, end - 1.0, limits) {
                pieces.push(piece);
            }
        }
    }
}

fn inside_polygon(points: &[Point3], point: Point3) -> bool {
    let mut inside = false;
    let mut previous = points.len() - 1;
    for current in 0..points.len() {
        let a = points[current];
        let b = points[previous];
        if (a[1] > point[1]) != (b[1] > point[1]) {
            let x = (b[0] - a[0]) * (point[1] - a[1]) / (b[1] - a[1]) + a[0];
            if point[0] < x {
                inside = !inside;
            }
        }
        previous = current;
    }
    inside
}
