//! Fillet Corners (`FilletCorners`, OM9-CURVE-023).

use om9_core::geometry::{Curve, Geometry};

use super::{Controller, Error};

pub const FILLET_CORNERS: &str = "OM9-CURVE-023";

#[derive(Debug, Clone)]
pub struct FilletCornersPending {
    pub curve: Curve,
    pub radius: f64,
}

impl FilletCornersPending {
    pub fn prompt(&self) -> &'static str {
        "Radius=, or Enter for radius 1"
    }
}

impl Controller {
    pub fn dispatch_fillet_corners(&mut self) -> Result<(), Error> {
        let id = self
            .selection
            .iter()
            .next()
            .copied()
            .ok_or(Error::Missing("polyline selection"))?;
        let entity = self
            .document()
            .entities
            .get(&id)
            .ok_or(Error::Missing("selected polyline"))?;
        let Geometry::Curve(curve) = &entity.geometry else {
            return Err(Error::Invalid("polyline selection"));
        };
        if !matches!(curve, Curve::Polycurve { .. }) {
            return Err(Error::Invalid("polyline selection"));
        }
        self.fillet_corners_pending = Some(FilletCornersPending {
            curve: curve.clone(),
            radius: 1.0,
        });
        Ok(())
    }

    pub fn submit_fillet_corners_text(&mut self, text: &str) -> Option<Result<(), Error>> {
        let mut pending = self.fillet_corners_pending.take()?;
        let option = text.trim().to_ascii_lowercase();
        if option == "cancel" || option == "esc" {
            return Some(Ok(()));
        }
        if let Some(value) = text.trim().strip_prefix("Radius=") {
            let Some(radius) = value
                .parse::<f64>()
                .ok()
                .filter(|value| value.is_finite() && *value > 0.0)
            else {
                self.fillet_corners_pending = Some(pending);
                return Some(Err(Error::Invalid("radius must be positive")));
            };
            pending.radius = radius;
            self.fillet_corners_pending = Some(pending);
            return Some(Ok(()));
        }
        if !text.trim().is_empty() {
            self.fillet_corners_pending = Some(pending);
            return Some(Err(Error::Invalid(
                "FilletCorners accepts Radius= or Enter",
            )));
        }
        let Curve::Polycurve { segments, .. } = &pending.curve else {
            return Some(Err(Error::Invalid("polyline selection")));
        };
        let mut points = Vec::with_capacity(segments.len() + 1);
        for segment in segments {
            let Curve::Line { start, end, .. } = segment else {
                return Some(Err(Error::Invalid("polyline must contain lines")));
            };
            if points.is_empty() {
                points.push(*start);
            }
            points.push(*end);
        }
        if points.len() < 3 {
            return Some(Err(Error::Invalid("polyline needs corners")));
        }
        let plane = self.input_plane();
        let normal = [
            plane.x[1] * plane.y[2] - plane.x[2] * plane.y[1],
            plane.x[2] * plane.y[0] - plane.x[0] * plane.y[2],
            plane.x[0] * plane.y[1] - plane.x[1] * plane.y[0],
        ];
        let closed = distance(points[0], *points.last().expect("points")) < 1e-9;
        let unique = if closed {
            points.len() - 1
        } else {
            points.len()
        };
        let corners: Vec<usize> = if closed {
            (0..unique).collect()
        } else {
            (1..unique - 1).collect()
        };
        let mut made = 0;
        let mut last_error = None;
        for index in corners {
            let previous = if closed {
                points[(index + unique - 1) % unique]
            } else {
                points[index - 1]
            };
            let next = if closed {
                points[(index + 1) % unique]
            } else {
                points[index + 1]
            };
            let first = Curve::Line {
                start: points[index],
                end: previous,
                domain: [0.0, 1.0],
            };
            let second = Curve::Line {
                start: points[index],
                end: next,
                domain: [0.0, 1.0],
            };
            match om9_core::curve_modify::fillet_lines(
                &first,
                &second,
                pending.radius,
                normal,
                1e-7,
                self.coordinator.limits(),
            ) {
                Ok(fillet) => {
                    if let Err(error) = self.commit_geometry(
                        FILLET_CORNERS,
                        "FilletCorners",
                        Geometry::Curve(fillet.arc),
                    ) {
                        return Some(Err(error));
                    }
                    made += 1;
                }
                Err(error) => last_error = Some(error),
            }
        }
        if made == 0 {
            return Some(Err(last_error.unwrap_or(Error::Invalid("no corners"))));
        }
        Some(Ok(()))
    }

    pub fn fillet_corners_option_tokens(&self) -> Vec<(&'static str, &'static str)> {
        if self.fillet_corners_pending.is_some() {
            vec![("Radius=", "Radius="), ("Cancel", "Cancel")]
        } else {
            Vec::new()
        }
    }
}

fn distance(a: om9_core::geometry::Point3, b: om9_core::geometry::Point3) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}
